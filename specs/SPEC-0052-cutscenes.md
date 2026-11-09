# SPEC-0052: In-engine cutscenes

- Answers: proactive (open question sweep, round 10); none (proposed topic; cross-references SPEC-0027, SPEC-0028, SPEC-0030, SPEC-0033, SPEC-0040); proactive (open close-out); proactive (open question sweep, script details); proactive (open question sweep, round 6); proactive (spec consistency pass)
- Status: Draft
- Disc build: SCUS-97367 v1.00. Check the serial, and check that the level modules named in the worked examples (archive resources 985 and 1217) load as level modules.
- Confidence: High for the camera track layout, key sampling and play timing, the screen-effect object (fields, per-pass arithmetic, bar geometry, draw order), the request rules, frame counts as game ticks, the locator sample formula, the wait rules and the absence of any scene skip; Medium-High for the exact tick on which fades and bars finish (the add rule of SPEC-0008); Medium for the teleport curtain quirk, the pass on which a camera wait resumes, the look-at orientation of target plays, the tether and the control-lock restore; Low for the locator record fields other than the position. Movies in scenes: High for the fixed indices and for which routine of the final fight plays movies 78, 80 and 85; High for the finale gate; Medium-High for the movie 4 block; Medium for the reach of the other packs; Medium-High for movies 71 and 72 not playing. Per section.

## Summary

An in-engine cutscene is not a separate system. It is an ordinary level control thread (SPEC-0030): a script body that runs straight-line host calls in order. Examples are moving the hero to a locator, playing an entity animation, fading the screen, sweeping letterbox bars, playing a camera track, and showing dialogue lines. A blocking call gives up one tick per loop iteration and resumes when its condition holds. There is no timeout. There is no timeline asset. Animated-looking positions come from a locator table sampled when the call runs, and camera motion comes from keyframed camera tracks (SPEC-0027, SPEC-0028).

The clean side can build cutscenes as scripted sequences over these pieces: a per-tick thread runner that honours the wait rules, a locator reader and sampler, a camera track player that interpolates keys, one screen-effect object (fade and letterbox bars) updated and drawn once per pass over the world and under the HUD and menus (SPEC-0046, Per-frame draw order), and the control-state rules. The original has no player-facing skip of an in-engine scene (section "Skipping"). A scene can also queue a full-screen movie with the play command and a fixed movie index; the game flow plays the queue (SPEC-0026). Section "Movies in scenes" gives the scene movies that matter, including the three of the final fight.

## Where it lives on the disc

- Thread bodies that make these calls are in the level module archive resources. Worked examples use resource 985 (an exterior scene) and resource 1217 (a scripted scene with a fade and a camera play). Thread and host-call conventions are in SPEC-0030.
- Locator table: the object header chunk (type 11) of the world pack that owns the locators. Locators are indexed by id.
- Camera tracks: the type-28 table of the world pack (SPEC-0028, section 1). The track id is the record index.
- Screen-effect state, thread slots and control states are runtime state. They are not on disc.

## Layout

### Locator table (type 11 chunk, world locators)

Header fields (payload-relative offsets, relocated at load). Values shown are from the example pack 985.

| Offset | Size | Type | Meaning | Example (pack 985) |
| --- | --- | --- | --- | --- |
| +0x24 | u32 | count | number of locators in array A; valid ids are 0 to count minus 1 | 30 |
| +0x2C | u32 | count | number of records in array B | 6 |
| +0x4C | u32 | offset | start of array A | not stated |
| +0x54 | u32 | offset | start of array B | not stated |

Array A record (0x50 bytes, one per locator):

| Offset | Size | Type | Meaning | Confidence |
| --- | --- | --- | --- | --- |
| +0x30 | 16 bytes | 4 x float32 | position x, y, z, then w | High for position; Low for the meaning of w (seen values 0.625 and 4.76; it is carried into the sample's w, see Locator moves) |
| other bytes | | | no encoded meaning (unknowable); small integers and 0xFFFFFFFF words occur; ignore | Low |

Array B record (0x50 bytes, the ambient-sound and adventure-zone volumes of SPEC-0031 and SPEC-0009): byte +0x4F is the start-delay counter of an ambient loop. The loader clears it to 0 for each record. It has no effect on cutscenes (High).

### Camera track table (type 28 chunk)

Word 0 (u32 at +0x00) is the track count. On pack 985 it is 34. The 4-byte record offsets start at +0x10, one per track, in track id order. A track record is found through its offset in this table, not by a fixed stride from the start of the table.

### Camera track record (type 28; track id = record index)

A record is a 0x40-byte head. When its key count N is above 0, N keys of 0x30 bytes follow the head directly. Confidence High (all 191 packs: 2,010 records, 329 of them keyed, 15,429 keys).

Head (0x40 bytes):

| Offset | Size | Type | Meaning |
| --- | --- | --- | --- |
| +0x00 | 16 bytes | 4 x float32 | head quaternion (x, y, z, w). Used only when N = 0 (static shot). On every keyed track it is a placeholder, (0.7071, 0.7071, 0, 0) |
| +0x10 | 16 bytes | 4 x float32 | head position, w = 1. Used only when N = 0. (0, 0, 0, 1) on every keyed track |
| +0x20 | s32 | integer | N, key count. 0 on 1,681 records; 1 to 260 on the other 329 |
| +0x24 | float32 | seconds | dt, the key interval. 1/15 s on 95 keyed tracks; on the others it is D / N with D a whole number of 1/30 s steps (61 keys over 4.0 s gives 0.065574) |
| +0x28 | u8 | flag | 0 or 1 (1 on 764 records). No effect on play is known; see SPEC-0028 |
| +0x2B | u8 | flag | per-key focal select. 0 on every record, so the track focal (+0x2C) is used |
| +0x2C | float32 | pixels | focal length of the render camera while the track plays (640-wide frame, SPEC-0027). 791.45 on 299 keyed tracks; also 645.8 (8 tracks), 493.0 (7), 328.0 (5), 585.7 (4), 1071.1 (3), and 607.9, 513.4, 704.7 once each. 0.0 on 72 records, none of which the pack 985 scene plays |
| +0x34 | u32 | offset | key array offset, relative to the start of this record. 0x40 on every keyed record (the keys follow the head); 0 when N = 0. Relocated at load by adding the record's own address |

Key (0x30 bytes, N of them, contiguous):

| Offset | Size | Type | Meaning |
| --- | --- | --- | --- |
| +0x00 | 16 bytes | 4 x float32 | unit quaternion (x, y, z, w); 4 of 15,429 keys are off unit length by more than 0.001 |
| +0x10 | 16 bytes | 4 x float32 | camera position, w = 1 on every key |
| +0x20 | 4 bytes | u32 | 0 on every key |
| +0x24 | 4 bytes | float32 | per-key focal length; equal to the head's +0x2C on every track (constant within a track) |
| +0x28 | 8 bytes | | unused fill |

Example (pack 985, track 25): N 30, dt = 1/15 s (single-precision bit pattern 0x3D888889), focal 791.448. Key 0 position (-645.90, 128.38, -597.22); key 29 position (-644.31, 127.97, -589.21). The quaternions turn smoothly from (-0.023, -0.131, 0.003, 0.991) at key 0 to (-0.018, -0.226, 0.004, 0.974) at key 29. All 30 keys are part of one smooth path. Track 29 of the same pack: N 61, dt 0.065574 (4.0 s / 61).

### Screen-effect object (runtime, one object)

One static object holds the letterbox bars and the full-screen fade. Field offsets are within the object (0x80 bytes used). Confidence High.

| Offset | Type | Meaning |
| --- | --- | --- |
| +0x00 | float32 | top bar height at bar progress 1 (modes 0 and 2): 32.0, or 256.0 after a teleport landing (Edge cases) |
| +0x04 | float32 | bottom bar height at bar progress 1; same values |
| +0x08 | s32 | screen height H = 448 |
| +0x0C | s32 | screen width W = 512 |
| +0x10 | s32 | bar delay in ticks (0 in every caller) |
| +0x14 | s32 | bar done flag |
| +0x20, +0x24, +0x28 | u8 each, in 32-bit lanes | bar colour R, G, B (corrected 2026-11-09, High: the +0x2C lane is copied into the draw packet but never written; the bar alpha goes into the packet's own alpha byte at draw time) |
| +0x30 | s32 | bars held: 1 after LetterBoxOn, 0 after LetterBoxOff |
| +0x34 | s32 | draw depth (taken from the display object) |
| +0x38 | s32 | consecutive passes on which bars were drawn (0 when none) |
| +0x40, +0x44, +0x48 | u8 each, in 32-bit lanes | fade colour R, G, B |
| +0x50, +0x54, +0x58 | float32 each | fade alpha, fade target, fade step |
| +0x5C | s32 | bar mode: 0, 1 or 2 (Behavior) |
| +0x60, +0x64, +0x68 | float32 each | bar step, bar progress, bar target |
| +0x6C, +0x70, +0x74 | float32 each | bar colour alpha, its target, its step (the step is only ever 0) |

Start values (and the values after a full reset): bar heights 32, H 448, W 512, both colours black, fade alpha 0, bar colour alpha 128, bar progress 0, bar mode 1, held 0.

Alpha scale: 0 to 128, where 128 is fully opaque (the PS2 GS alpha scale, 128 = 1.0).

### Control states (entities and the hero)

| Value | Decimal | Meaning | Confidence |
| --- | --- | --- | --- |
| 1 | 1 | free: normal control | High |
| 0x21 | 33 | script hold: no input response, follows script orders | High |
| 0x32 | 50 | scripted walk or run to a point (WalkHero and RunHero wait on it) | High |
| 0x34 | 52 | walk and orient (WalkHeroAndOrient) | High |
| 0x33 | 51 | scripted one-shot animation (PlayHeroAnim): waits for the clip progress to pass 0.99, then goes to 1 when the hero's flag word at +0x3D0 is nonzero, otherwise to 0x21 | High |
| 0x37 | 55 | scripted looped animation (LoopHeroAnim): a tick counter runs down, then the state goes to 0x21 when the flag word is zero, otherwise to 1 | High |
| 0x19 | 25 | talking: entered by SetConversationTarget and by LevelMsg on a hero that is not busy; the dialogue emotes play only in this state (SPEC-0022, SPEC-0040) | High |

## Behavior

### Threads and waiting

- A cutscene is a control thread body. It runs its calls top to bottom and yields only inside blocking calls.
- Each tick (60 per second, SPEC-0007) a waiting thread is resumed once, its condition is tested, and it yields again if the condition is not met.
- Sleep(n) waits n ticks. n of 0 or less does nothing.
- Wait(f) waits trunc(f x 60) ticks, where f is in seconds. Wait(0.5) is 30 ticks.
- Ticks are the only clock. No call measures wall time.
- The factor is 60 ticks per second. The game sets it once at start-up and never changes it (SPEC-0007). The rules in this spec use 60.
- Waiting calls have no timeout. A condition that never becomes true keeps the thread waiting.

Blocking calls and their conditions:

- PlayCamera with wait set, on a track with at least one key: waits until the play has ended or gone idle (Camera track playback).
- Fade calls with wait: yield one tick per loop while fade alpha differs from its target (SPEC-0030 rows 136 and 137).
- Letterbox calls with wait: yield one tick per loop until the bar done flag is set (SPEC-0030 rows 139 and 140).
- WalkHero with wait: waits while the hero stays in the walk state.
- PlayEntityAnim with wait value 1: yields one tick, then one tick at a time until the entity's animation track reports finished (High). Other wait values return at once.
- LevelMsg with wait value 1: yields one tick at a time while the dialogue box is active, so it returns on the first pass after the box closes (the player dismisses the last page). Other wait values do not wait (High).

### Locator moves

MoveObjToAnimatedLocator(target, source object, locator id, flag, W):

1. The target and source are placed-object indices of the named world (the current world when none is named). Find the locator table of the world that owns the source object. If either index is out of range, the source has no locator table, or the locator id is outside 0 to count minus 1, the call does nothing (High).
2. If the flag is 0, take one sample at the moment the call runs (High). Let L be the locator vector (record +0x30), P the source object's position (object +0xF0), row0, row1 and row2 the source object's first three basis rows (object +0x00, +0x10, +0x20), and V the source object's vector at +0x80. With d = L - P (x, y, z only):
   - sample.xyz = P.xyz + d.x row0.xyz + d.y row1.xyz + d.z row2.xyz + V.xyz;
   - sample.w = L.w + V.w.
   The target's position (+0xF0) is set to the sample, and the target's second vector (+0x150) is set to (0, 0, 0, 1). The fourth basis row and P.w are not used.
3. If the flag is 1, store a tether on the target that points to the locator record. Each object update then recomputes the target's base from that locator, with the host's pose applied, so the target follows the source (SPEC-0028, section 7.1, step 5).

"Animated" means the transform is the source object's current pose at call time. It does not mean a keyframed route through the locator table. The source's animation time maps to a frame index at 15 frames per second with a fractional part (SPEC-0005). A script that wants a locator to keep following a moving object must repeat the call or use the tether.

The static form MoveObjToLocator(target, locator id, link) takes no source object. With link 0 it sets the target's position to the locator record's position point (four floats) and the target's second vector to (0, 0, 0, 1); with link nonzero it stores a locator link and the target follows the record on every object update. It does nothing when the locator id is outside the world's locator table.

### Camera track playback

Confidence High for the rules; Medium for the pass on which a waiting script resumes and for the target-play orientation.

- PlayCamera(track id, WAIT, end mode, speed) takes a record index of the type-28 table. The third argument (the camera-wait option) is the play's end mode 0, 1 or 2, not an entity index. The fourth is a playback speed factor, not a duration (SPEC-0030; SPEC-0027). A track id outside the world's list does nothing and does not wait.
- Start: the track time t is set to 0 and the run state to 1 (playing).
- Each camera update (once per pass) samples the track at t (Sampling a track, below). When the sample is valid, the camera is posed from it, the focal length is written to the render camera, and then t = t + (1/60) x speed. When the sample is not valid, the run state becomes 2 (ended) and the end mode acts in the same update:
  - end mode 1: nothing more happens; the camera holds the last pose until ReturnToNormalCamera, SetToNormalCamera or another play;
  - end mode 0: the hero is set to state 1 unless it is in a scripted state; the play becomes idle;
  - end mode 2: the hero is set to state 1 whatever its state; the play becomes idle.
- While a play runs, a hero that is in none of the scripted states (0x21, 0x30, 0x32 to 0x35, 0x37) and not in 0x12, 0x13 or 0x19 is put in state 0x21 and made to play its animation 0, on every update.
- Posed updates for N of 2 or more: the updates on which trunc(t / dt) + 1 < N. Nominally this is ceil((N - 1) x dt x 60 / speed); when that product is an exact integer, float accumulation gives one more. Single precision (the same with round-to-nearest and round-toward-zero): track 25 of pack 985 (N 30, dt 1/15) at speed 1.0 poses 117 updates and ends on update 118; at speed 0.5, 233 posed and ended on 234; at speed 2.0, 59 and 60. Track 29 (N 61) at speed 1.0: 237 and 238.
- PlayCamera with WAIT waits only when N is above 0. It yields each tick until the play is ended or idle, testing before the first yield. With WAIT on track 25, the script resumes on the pass of update 118 (Medium for the pass order).
- Static shot (N = 0): every update returns the head pose as a valid sample, so the play never ends by itself. It stays in run state 1 until ReturnToNormalCamera, SetToNormalCamera or another play. IsAnimatedCameraActiveAndFinished stays false for it.
- N = 1: the first update has no valid sample, so the play ends on its first update without posing the camera. With WAIT the script resumes after one update.
- IsAnimatedCameraActiveAndFinished(track id) is true only while a play is active (run state not 0), it plays that track, and the play is in run state 2 (for play kinds other than the plain and target track plays it is true whenever active). Scenes use it to gate later calls with a Sleep(1) loop.
- Field of view while a track plays: horizontal FOV = 2 atan(320 / focal). Focal 791.45 gives 44.0 degrees, against 64.0 degrees for the normal camera's focal of 512; vertical 2 atan(240 / focal) = 33.7 degrees.
- Target plays (the hero, AI and look-at variants, SPEC-0030): the eye is the sampled position; the orientation is rebuilt to look at the target's anchor point, with the side axis flattened to horizontal (y = 0) whenever its |x| + |z| exceeds 0.01, so the horizon stays level (Medium).

### Sampling a track

Given the track time t in seconds (High):

1. If N < 1: the pose is the head quaternion and head position, the focal length is the head's +0x2C, and the sample is valid.
2. Otherwise i = trunc(t / dt). If i + 1 >= N there is no sample (the play ends, above). Otherwise f = (t - i x dt) / dt, A = key i and B = key i + 1.
3. Rotation: if dot(qA, qB) < 0, replace qB by -qB. Then q = (1 - f) qA + f qB, component by component. There is no slerp and q is not renormalised.
4. Rotation matrix from q = (x, y, z, w), without normalising:
   - row0 = (x² + w² - y² - z², 2(xy - wz), 2(xz + wy), 0)
   - row1 = (2(xy + wz), y² + w² - x² - z², 2(yz - wx), 0)
   - row2 = (2(xz - wy), 2(yz + wx), z² + w² - x² - y², 0)
   - row3 = the position from step 5.
   The camera pose is this matrix, in the basis convention of SPEC-0027.
5. Position: (1 - f) pA + f pB, with w = 1.
6. Focal length: if the head byte +0x2B is nonzero, (1 - f) focalA + f focalB; otherwise the head's +0x2C (the only case on disc). It is written to the render camera's focal length on every posed update.

### Screen effect: per-pass update and draw

The screen effect is updated and drawn once per main-loop pass, after the world is drawn and before the HUD and the menus, so the HUD and menus are drawn over the fade and the bars (SPEC-0046, steps 11 and 12). Every frame count below is a count of these updates, which is one per game tick. Confidence High; float detail Medium (Test vectors).

1. **Delay.** If the bar delay is 1 or more, it goes down by 1; otherwise bar progress += bar step.
2. **Bar clamp.** If (step < 0 and progress < target) or (step > 0 and progress > target): progress = target, done flag = 1, step = 0. Equality does not clamp: a sum that lands exactly on the target sets the done flag only on the next update, when the sum overshoots.
3. **Fade.** If the fade step is not 0: alpha += step; if rising and alpha > target, or falling and alpha < target, alpha = target and step = 0. Then alpha is clamped to 0 to 128. When alpha is 0 the fade colour is reset to black.
4. **Bars.** If progress <= 0, no bars this pass (both heights 0). Otherwise the bar colour alpha moves by its step with the same clamp; if the bar colour alpha is 0 or less, the bars auto-clear (Screen effect: requests) and the bar colour alpha is set to 0; the bar colour alpha is capped at 128; at 0 the bar colour becomes black. Then the bar heights, with p = progress:
   - mode 0 (letterbox): top = p x height0, bottom = p x height1 (32 px each at p = 1);
   - mode 1 (full curtain): top = bottom = p x trunc(H / 2), so 224 px each at p = 1, which meets at the centre;
   - mode 2 (letterbox to full close, p from 1 to 2): with c = trunc(H / 2) + 1 = 225, top = height0 + (p - 1)(c - height0) and bottom = height1 - (p - 1)(height0 - c). With heights of 32 the bars grow from 32 to 225 px each; at p = 2 they overlap at the centre.
5. **Draw.** If the top or bottom height is not 0: the top bar is the rectangle from (0, 0) to (W, top) and the bottom bar the rectangle from (0, H - bottom) to (W, H), both in the bar colour with alpha round(bar colour alpha). Then, if the fade alpha is not 0: one full-screen rectangle from (0, 0) to (W, H) in the fade colour with alpha round(fade alpha). Bars are drawn first and the fade on top. A rectangle is drawn opaque when its alpha is 128 and blended (standard source-over, 128 = 1.0) when below. Added 2026-11-09 (High): a 4-word header that sets the alpha-blend register to source-over is written once per call, and only when something is drawn; each rectangle is a sprite quad whose colour R, G, B come from the object's colour lanes (bars 0x20 to 0x2F, fade 0x40 to 0x4F) and whose alpha byte is overwritten with round-to-nearest (add 0.5 for a non-negative alpha, subtract 0.5 otherwise, truncate); the bottom quad starts at y = H - bottom height with the bottom height as its height; after the two bar quads the "bars drawn" counter at +0x38 goes up by 1, and with no bars it is set to 0; the vertex build offsets the screen so its centre sits at GS coordinate 2048 (x + 2048 - W/2, y + 2048 - H/2, times 16, rounded to nearest). Nothing is written when both heights are 0 and the fade alpha is 0.

### Screen effect: requests

The dialogue box host re-initialises the screen-effect object on its first pass: bar heights return to 32 (including after a teleport landing's 256), fade alpha and fade colour clear, and the bar colour resets to black. Progress clears and bars-up is requested only when bars are not held. (Dialogue box states: SPEC-0035 section 2a.)

- Frames: frames = max(1, trunc(seconds x 60.0)) in single precision for every script call, the product formed by one truncating single-precision multiply (SPEC-0008, Seconds to ticks): 0.3 s = 18, 0.5 s = 30, 0.7 s = 41 (0.7 is stored as 0.699999988, and the product 41.999996 truncates to 41), 1.0 s = 60. Engine callers pass frame counts directly (for example the hero swap's 37 and 8, SPEC-0053).
- **Fade out** (colour, frames): the fade colour is set, alpha = 0 whatever its previous value, target 128, step 128 / frames (every step on this page is a single-precision divide to nearest; the per-update adds follow SPEC-0008, Platform arithmetic). **Fade in** (colour, frames): colour set, alpha = 128, target 0, step -128 / frames. A new fade replaces the old one. Fades do not check the held flag.
- **Bars up** (LetterBoxOn): nothing while bars are held. Otherwise target = 1; if progress is not already 1, done = 0, mode 0, step = 1 / frames.
- **Bars down** (LetterBoxOff): nothing while bars are held. Otherwise target = 0; if progress is not already 0, done = 0, mode 0, step = -1 / frames. The target is written even when the step is not.
- **OpenLetterBox**: nothing while held. Otherwise mode 1, progress 1, the delay, target 0, done 0, step -1 / frames; when a colour is given, the bar colour RGB and the bar colour alpha (current and target) are taken from it. The full curtain opens from the centre.
- **CloseLetterBox**: nothing while held. The colour is taken as above; target 1, step 1 / frames; if the mode is 0 and progress is not 0 (letterbox bars are showing), mode 2 and target 2 (the letterbox-to-full-close sweep); otherwise mode 1 (the full curtain closes from progress 0). Then the delay is set and done = 0.
- **Auto-clear**: if bars are not held, an OpenLetterBox request of 1 frame with no colour, then progress = 0. Added 2026-11-09 (High): the request leaves mode 1, step -1.0, target 0 and done 0, so on the next update progress clamps to 0 and done becomes 1. The bar heights of the update that ran the auto-clear use the progress after the clear, so that pass draws no bars. The close request (CloseLetterBox) does not touch progress itself, and the delay is 0 in every caller (the script wrappers pass 0).
- LetterBoxOn sets held = 1 after its request; LetterBoxOff clears held before its request. So a held state blocks every bar request except LetterBoxOff.
- Waits: fade calls yield one tick per loop while alpha differs from the target; bar calls yield until the done flag is set. Neither has a timeout. A new fade started by another thread restarts a fade wait. When a LetterBoxOn request is ignored (bars held), the done flag keeps its old value, and the wait follows it.
- Script calls: FadeScreenOut and FadeScreenIn (SPEC-0030 rows 136 and 137) pass the colour by value, with R in bits 0 to 7, G in bits 8 to 15 and B in bits 16 to 23 (the top byte is ignored). LetterBoxOn and LetterBoxOff are SPEC-0030 rows 140 and 139. Those rows give the call shapes and the per-duration site counts.
- **Moving query**: true when fade alpha differs from its target, or the bar colour alpha differs from its target, or bar progress differs from its target. Confirmed 2026-11-09 (High): the three comparisons are made in this order with exact float equality; HUD and hero-busy rules that use "moving" use exactly this test.
- Disc use (call sites in level code that run in play, each distinct module counted once; SPEC-0030, Disc call-site counts):
  - LetterBoxOn has 52 call sites in 21 modules (45 without wait, 7 with). LetterBoxOff has 54 in 22 modules (one at 0.7 s).
  - OpenLetterBox and CloseLetterBox have 2 call sites each, all in module 990's prompt handler for triggers 3 to 8 (High). They form two open-then-close pairs around one camera track, all with 0 s (1 frame) and no wait:
    - Reach: the hero is in trigger 3, 4 or 5; the prompt is showing, which needs object 12, 14 or 16 (in that order) to be inactive; and button 0 is pressed. Triggers 6, 7 and 8 never reach the pairs.
    - Track: trigger 3 plays camera track 5, trigger 4 track 4 and trigger 5 track 6, at speed 1.0 with wait, then the normal camera returns.
    - Lever word: the pairs run only while the module's lever word is +1. The word starts at -1 when the module loads, and each press at trigger 6, 7 or 8 negates it. So the pairs run after an odd number of lever presses since the module loaded, and the track plays untinted after an even number.
    - First pair (before the track): OpenLetterBox with colour (0, 0, 128) and alpha 20, then CloseLetterBox with no colour, in the same tick. The bars are in mode 1 at progress 1, so they cover the whole screen (224 px each on a 448-line screen). The camera track plays under a dark blue tint blended at 20/128.
    - Second pair (after the camera returns): OpenLetterBox with colour (0, 0, 0) and alpha 0, then CloseLetterBox with no colour. On the next update the bar colour alpha is 0, so the bars auto-clear and the tint is gone.
    - Each of triggers 3 to 5 runs at most once while the module stays loaded. Init turns object 13, 15 or 17 on, so the first press turns object 12, 14 or 16 on, and the prompt does not show again.
  - FadeScreenOut has 264 call sites in 54 modules (1.0 s at 82, 0.5 s at 77, 0.7 s at 49). FadeScreenIn has 246 in 51 modules (0.7 s at 44).
  - Fade colours, over all of these sites (High for black and white; Medium-High for the 4 sites that clear the colour with a fill):
    - FadeScreenOut, 264 sites: black 239, white 20, (62, 3, 73) purple 4, (255, 64, 32) orange 1.
    - FadeScreenIn, 246 sites: black 224, white 17, purple 4, orange 1.
    - Every fade has a fixed colour, either a colour word in the module's data or a cleared colour (black). No fade computes its colour.
    - White fades are in modules 735, 777, 842, 1017, 1023, 1257, 1371, 1382, 1394, 1397, 1398 and 1411. The purple fades are in module 956 (and its repeat 970), under its update routine, and the orange pair is in module 864.

### What the screen effect hides and locks

Confidence High.

- **HUD.** The monster bar, the D-pad hint and the journal icon are hidden while the effect is moving, or bar progress > 0, or trunc(fade alpha) > 0 (also while the quick panel is open; SPEC-0023). So bars held at rest also hide them.
- **Hero.** The hero busy predicate (SPEC-0033) is true while the effect is moving; while it is true the stick-walk step and the L2, L1, Cross and Square action routes do nothing. Bars held at rest do not lock the hero by themselves.
- **Doors.** A door touch is refused while the effect is moving or while the game status code is not 0 (SPEC-0026, Game-state code).
- **Camera plays.** While a camera play runs, the hero is held in state 0x21 with animation 0 (Camera track playback).
- **Script buttons.** IsButtonPressed returns false whenever its third argument is true, for button code 7, and while either of two game-status words is set (SPEC-0030).

### Control locks

- A lock is a state value, not a counter. It has no timeout. It lasts until a release or restore is run.
- GrabHeroControl: sets the hero state to 0x21, with no check of the current state.
- BlockPlayerControl(hero): the hero argument picks the hero (-1 means the current hero; any other value picks that entry of the two-hero table). If that hero has a control record, the call first plays the active hero's one-shot animation 0 with the control flag set to 1 (this yields one tick and puts the active hero in state 0x33), and then sets the picked hero's state to 0x21. Without a control record nothing changes and nothing yields. With the current hero (the usual case) the net result is state 0x21 after one tick (High). The disc calls are assumed to pass the current hero (Medium).
- RestorePlayerControl(hero): the hero argument picks the hero as for BlockPlayerControl. It sets that hero's state to 1 only if the hero has a loaded model and its state is currently 0x19, 0x37, 0x33 or 0x21. In any other state, including walking, nothing changes. So a block followed by a restore frees the hero, but a restore during a walk does nothing.
- ReleaseEntityControl(entity): sets the entity state to 1 when the entity exists, with no check of its current state.
- CTL argument of PlayEntityAnim, LoopEntityAnim, WalkHero and MoveHero: stored on the entity as a flag, set when CTL equals 1. For PlayEntityAnim, the state after the play is 1 when CTL equals 1, and 0x21 otherwise.
- The input lock during a scene has four layers: the script hold (state 0x21, from Block or Grab), the camera play's per-update hold (Camera track playback), the hero busy predicate while the screen effect moves, and the door refusal while the effect moves or the game status is not 0.

### Worked example: resource 985 (exterior scene)

Described in order. Sequence numbers are the order of calls on the main route. Calls are unconditional unless noted.

1. LetterBoxOn with 0 seconds and no wait: one frame, so the 32 px bars are at full height after one update, and they stay held.
2. MoveHero call to locator 9, with CTL value 0.
3. Sleep 1 tick.
4. Loop an entity animation with a negative seconds value. The tick budget is 1 tick.
5. Run a boss-fight tweak on the hero.
6. Fade in from the colour over 1.0 second with no wait (60 ticks).
7. Block the current hero's control (state 0x21).
8. Play camera track 25 with no wait, end mode 1 and speed 1.0. It poses 117 updates, ends on update 118 and then holds its last pose.
9. Sleep 30 ticks.
10. Play an animation on entity 0 with no wait.
11. Sleep 1 tick until the camera play for track 25 reports finished.
12. Sleep 30 ticks, play camera track 26 (a static shot with no keys, so no wait; it holds until the next play), then sleep 30 ticks.
13. Loop an animation on entity 0 with 0 seconds (no limit).
14. A dialogue section: a conversation target is set, then dialogue lines with waits alternate with camera tracks 27, 26, 30, 33 and 28 (end mode 1, speed 1.0) and entity animations. The conversation is ended at the end.
15. Play camera track 29 with wait, end mode 2 and speed 1.0 (237 posed updates). When it ends, the hero is set to state 1 and the play goes idle. Set the hero equipped state. Wait 0.5 second (30 ticks). LetterBoxOff over 1.0 second with no wait. Return to the normal camera. Release entity 0's control.

The chain never calls RestorePlayerControl for the hero blocked in step 7; the end-mode-2 play of step 15 frees the hero instead.

### Worked example: resource 1217 (scripted scene with a fade)

The scene is a chain of: fade out to the colour over 0.7 second with wait (41 frames; the wait ends on update 42), play camera track 14 at speed 1.0, then fade in over 0.4 second (24 ticks). The chain is gated by a game-state check. A second branch is guarded by a Select-button test (button code 5) whose third argument is true; IsButtonPressed always returns false when its third argument is true, so that branch (a character activation with no target) never runs.

### Movies in scenes

- A scene queues a movie with the level play command. The movie index is a constant written at the call; no call computes it. The game has 83 such calls in 37 level modules. The queue, playback and movie skip are game flow (SPEC-0026, Movies and Movie play sites, which lists every call by pack). High.
- **Final fight (pack 1432).** All three movies are under the fight's control thread (SPEC-0054 for the fight, SPEC-0055 for the ending).
  1. The entry sequence runs once. It plays movie 78, then starts the fight clock at 180 seconds (10,800 ticks).
  2. Timeout ending: if the clock reaches zero while the fight is not won, the ending plays movie 80, sets game over (SPEC-0026, Game over flag) and then holds. The full gate and step list are in SPEC-0055.
  3. Finale: if the boss is defeated and the finale trigger fires with an object present, the finale plays movie 85 instead. The object field is filled by the hero's own contact: the selected hero's trigger-hit path fires the trigger with a nonzero object value. So movie 85 plays in normal play once the boss is defeated and the hero enters the finale trigger (High).
  4. Movies 80 and 85 cannot both play in one fight.
- **Movies 71 and 72 (pack 1411).** They play at the end of the elemental-pedestal chain (High):
  - Flag 337 is set, with flag 98, in pack 1382 once all six elemental pedestals are lit (SPEC-0041). Movie 70 then plays and the hero returns to 1411.
  - In 1411, the movie 71 call runs while flag 337 is set and flag 338 is clear, and sets flag 338.
  - The movie 72 call runs while flag 338 is set. It returns the hero to the final-fight pack 1432 (SPEC-0049, Who writes the flags; SPEC-0058 step 46).
  - Both movies play in normal play.
- **Movie 4 (pack 1023).** It plays at the end of an escort thread (Medium-High for the structure; High that it plays at most once per game):
  - Start: the thread is started by script trigger 17 while flag 127 is off, or by the area's control thread while flag 127 is on, flags 10 and 11 are off, flag 136 is off and no world or character load is active.
  - The thread does nothing when flag 136 is set.
  - Legs: it resumes at the first of eight leg flags (a list in the module's data) that is clear, then runs legs from there to leg 8. In each leg, entity 0 walks to the leg's locator. The thread then loops the entity's waiting animation, one of two chosen at random, until the hero is within 30 units of that locator. Legs 0 to 7 then set their leg flag.
  - Leg 8 sets flag 136, then fades out to black over 1.0 s with wait and reloads a world slot. It gives the hero item 32, plays movie 4, restores player control and fades back in over 1.0 s.
  - Flag 136 has this one setter in all level code. So once the block has run, it does not run again in that game.
- **Movie 67 (pack 1398).** Plays from pack 1398, which a script transition enters (Medium).
- **Movie 70 (pack 1382).** Plays only from pack 1382, which is entered only through the pending-area setter (SPEC-0006) (Medium).
- **Movies in side packs.**
  - Movies 34 and 35 are played by pack 777 (King Hagan's castle interior), which is reached by a script load from pack 758 (SPEC-0058 step 30). Its trigger index 0 runs only while flag 252 is off:
    - The first fire plays movie 34 and sets a module latch, which is 0 when the module loads.
    - A later fire in the same module lifetime finds the latch set and plays movie 35. That call also sets flags 252 and 69.
    - Trigger index 4 sets flag 252 without testing it. When it fires first, movie 35 is skipped (SPEC-0058, open question 14).
  - Movie 65 is played by pack 1394's trigger 11 (the shrine room), on contact, while the room word is 0. The room word is clear at the start of every module lifetime, so a return visit plays movie 65 again (SPEC-0041, 1394).
  - Movies 33, 36 and 37 are played by pack 753, which the hero reaches through cell exits (SPEC-0058 step 29):
    - 33 at teleport index 0 while flag 68 is off;
    - 36 by trigger 1, once (only while movie 36 has not played);
    - 37 by the control thread while level state 1 is set and flag 258 is off.
  - Confidence: High for the calls and gates; Medium for the pack 753 cell route. The full list is in SPEC-0026, Movie play sites.
- **Act 2 movie sequences (High for the calls and order; from the code, not run in the original).**
  - Pack 1174 trigger 1 (the Act 2 start): fade out black 0.3 s with wait; movie 16 queued; the act 2 start call, which is synchronous and whose done test always answers yes, so the routine does not yield; movie 89 queued, flag 221 set, return world eW 151, movie 18 queued. The three movies play in order at that pass's movie step with the act 2 start already applied; no scene follows in the routine (pack 1300's own first-arrival thread runs after the reload).
  - Pack 1327 door 0 (the Act 3 start): no fade; movies 27 and 29 queued, the act 3 start run, movies 90 and 30 queued, return world eW 64. The four play in the order 27, 29, 90, 30.
  - Pack 1327 target-practice end: line, a character-set load, Wait 0.5 s, camera track 5 (a static shot), line, Wait 0.3 s, normal camera at distance 10.0, return world eW 151, flag 235, movie 19; no fade.
  - Pack 1335: the bed (Square) fades out black 1.0 s with wait, then (flag 235 set, 92 clear) sets flags 236 and 92 and queues movie 20, then Wait 1.0 s and fades in 1.0 s without wait. Trigger 7 fades out black 0.5 s with wait and queues movie 25. Trigger 8 queues movie 21 (flag 403). Teleport 1 queues movie 26 with return world eW 150.
  - Pack 1344: trigger 0 queues movie 22, sleeps 30 ticks, shows line 10; trigger 1 queues movie 24. No fade.
  - Pack 1324: the solved scene plays a looped cue pair, waits 2.0 s, sets flag 239, fades out black 2.0 s with wait, stops both loops, queues movie 23, fades in black 0.5 s without wait, then restarts the loop pair ten times 15 ticks apart.
- **What each Act 2 movie is (disc data; the pictures were not decoded, so only file, length and place are known).** Each movie is a 512 by 448 Bink file with one audio track and no subtitle track; the game shows no caption text over a movie, and the only words around them are the dialogue lines the scripts show before or after. Table index, file, frames, length and where it plays: the Act 2 start plays 16 (ACT1/115A1, 1,508 frames, 50.3 s; index 17 names the same file and is never called), then 89 (the Act 2 intro card, ACTINTRO/ACT2, 222 frames, 7.4 s), then 18 (ACT2/200A, 3,348 frames, 111.6 s); 19 ACT2/203A, 1,795 frames, 59.8 s, after the target practice; 20 ACT2/204A, 1,197 frames, 39.9 s, the night in the bed; 21 ACT2/205A, 933 frames, 31.1 s, the optional cries scene in the palace; 22 ACT2/205B, 1,499 frames, 50.0 s, the first view of the captive and the Darkest Faerie when the hero enters the north wing (Roberta then says she must make a noise to distract the Darkest Faerie; the captive's own rescue lines and character set exist in pack 1344 but are never loaded, so this movie carries the lure); 23 ACT2/206A1, 322 frames, 10.7 s, the noise made by the solved puzzle; 24 ACT2/206A2, 1,320 frames, 44.0 s, the rescue after the hero returns to the north wing; 25 ACT2/206B, 521 frames, 17.4 s, the west stairs breaking while the minions flee (the module's character-set label for the set loaded with it is "fleeing smashed staircase"); 26 ACT2/207A, 385 frames, 12.8 s, the east door and the transport to the Endless Staircase; 27 and 28 name ACT2/209A1 (380 frames, 12.7 s; 28 is never played) and 29 ACT2/210A (1,070 frames, 35.7 s, 29.97 fps, 48 kHz audio). The picture descriptions for 20 to 26 are inferred from the scripts' order, lines and labels (Medium for 20, 25 and 26, Medium-Low for 22 to 24); the 1.0 s black-to-clear transition after movie 20 is only the script's own fade-in call (below), so a rewrite that has no decoded picture can show a neutral card of the movie's length.
- **Fade across a movie (High; derived from the hand-off reset and the fade request rules).** The hand-off resets the screen effect (fade alpha 0, fade colour black, bars 32 px, bar progress 0). So a script fade-out before a queued movie does not leave the screen black afterwards, and a script that then calls FadeScreenIn (alpha 128 at once, falling to 0) shows one opaque frame followed by the fade. Test vector (pack 1335 bed): fade out ends opaque on update 61; the movie plays; the first picture after the movie is clear; 60 ticks later the fade-in call makes the screen opaque and it is clear again after 61 updates. Movie file lengths (frame count at file header offset 8, frame rate from offsets 28 and 32): the Act 2 intro movie (89) has 222 frames at 29.97 fps (7.41 s); movie 29 has 1,070 frames at 29.97 fps (35.70 s); the other Act 2 movies are 30 fps (SPEC-0026, Movie name table).

### Skipping

Confidence High.

- No in-engine scene can be skipped. The cutscene route has no button check that ends a scene early.
- The button tests in the cutscene modules that look like skip checks sit inside a debug-mode gate. That flag is constant false in this build (SPEC-0030, debug rule), so these tests never run.
- Module 1217's one non-debug button test (Select, button code 5) passes a true third argument, which makes IsButtonPressed constant false. Its branch never runs (Worked example: resource 1217).
- Movies: every queued movie has its skip check enabled, because the queue add always clears the no-skip word and nothing sets it. The button rule (Cross or Start, five counted frames) is in SPEC-0026 (Movie skip); playback is SPEC-0008.

Rule for the clean side: do not add a cutscene skip based on this spec. If one is wanted, it is a design decision for the owner.

## State-code discrepancy with SPEC-0033

Resolved: there is no discrepancy. SPEC-0033, SPEC-0040 and SPEC-0022 number hero states in decimal. The scripted walk this spec calls 0x32 is state 50 there (SPEC-0033, Behavior 4A.4), the walk and orient 0x34 is 52, the script hold 0x21 is 33, the one-shot animation 0x33 is 51, the looped animation 0x37 is 55 and the talking state 0x19 is 25. SPEC-0033's "state 32" is a different state: the stick walk and run locomotion state (decimal 32, hex 0x20), which scripts never wait on. A rewrite should store one numeric value per state and convert only when it reads these two specs.

## Edge cases and error handling

- Locator id out of range (for example id 30 when the count is 30): the lookup returns nothing, and the move call changes nothing.
- Source object with no world: the locator move changes nothing.
- Sleep with n of 0 or less: no yield.
- Wait with a very small value (below 1/60 second): 0 ticks.
- LoopEntityAnim with 0 seconds: no tick limit (loops until released). With a negative value: a budget of 1 tick; a positive time below 1/60 s also gives no limit (High).
- A fade with 0 seconds: 1 frame, so alpha reaches its target after one update. An engine request of 0 frames (the save load's instant white) is also opaque after the first update.
- PlayCamera on a track with 0 keys: no wait, even with the wait flag set, and the static shot never ends by itself.
- PlayCamera on a track with 1 key: the play ends on its first update without posing the camera.
- A camera play at speed 0.0 (3 disc sites) never advances its time, so a keyed track holds its first pose and never ends by itself.
- Poll loop on a play with end mode 0 or 2 (reproduce): such a play is idle by the time a script can poll, so a loop on IsAnimatedCameraActiveAndFinished never ends. Pack 985 polls only end-mode-1 plays.
- RestorePlayerControl while the hero is walking: no change.
- BlockPlayerControl without a control record on the hero: no change.
- A block with no restore leaves the hero held indefinitely, unless an end-mode-0 or end-mode-2 camera play, or a return to the normal camera, frees it.
- A bar request while bars are held: no change to progress.
- A fade request while a fade is running: the new request resets alpha to 0 (FadeScreenOut) or to 128 (FadeScreenIn) and replaces the old fade.
- Bar ramps and equality: a progress sum that lands exactly on the target does not clamp, so the done flag can come one update after the nominal count (Test vectors).
- **Teleport curtain (reproduce and flag; Medium).** When the focus hero is landed on a teleport plane (door touches into a resident world, and plane landings after a load; SPEC-0026, Teleport landing), the game sets both bar heights to 256 px, requests bars up over 10 ticks and then, in the same tick, bars down over 10 ticks. Progress is still 0, so the second request writes only the target 0, and the next update clamps progress back to 0: no curtain is ever visible. The 256 px heights stay until the next movie handoff, the game-over page or a return to the front end (each resets them to 32). A LetterBoxOn in that window sweeps two 256 px bars that together cover the whole 448 px screen. Not confirmed on screen.
- Final fight: the timeout ending (movie 80) and the finale (movie 85) exclude each other; one fight plays at most one of them.
- Movies 71 and 72: flag 337 is set in pack 1382 when all six pedestals are lit; the movie 71 call in pack 1411 (337 set, 338 clear) sets flag 338; movie 72 plays while 338 is set. Both play in normal play.

## Test vectors

Synthetic values unless a pack is named.

- Wait(0.5) yields 30 ticks. Wait(0.01) yields 0 ticks. Sleep(0) yields 0 ticks.
- Frames: 0.3 s gives 18, 0.5 s gives 30, 0.7 s gives 41, 0.0 s gives 1.
- FadeScreenOut(0.7 s, wait): 41 frames, step 128 / 41; the alpha is 127.9999 after update 41 and is clamped to 128 on update 42, where the wait ends. FadeScreenIn(0.7 s) and LetterBoxOff(0.7 s) also finish on update 42.
- FadeScreenOut(black, 0.5 s, wait): 30 frames, step 128 / 30 (about 4.2667). With round-to-nearest single precision: alpha after updates 1, 2 and 3: 4.27, 8.53, 12.8 (drawn with alpha 4, 9 and 13, blended); after update 29: 123.73; after update 30: 128, drawn opaque.
- Fade in from white over 8 frames: alpha 112, 96, 80, 64, 48, 32, 16 after updates 1 to 7; after update 8 it is 0, nothing is drawn and the fade colour is black again.
- FadeScreenIn(black, 1.0 s) starting from alpha 128: 60 frames, step −128 / 60 = −2.1333334. Alpha is 0.000039 after update 60, and the clamp to 0 comes on update 61, so alpha is 0 after 61 updates.
- LetterBoxOn(0.5 s) from progress 0 (round-to-nearest single precision): progress goes up by 1/30 per update; bars are 1.07 px after update 1, 16 px after update 15 and 32 px after update 30. LetterBoxOff(0.5 s): 30.93 px after update 1; 0 and not drawn after update 30.
- Updates to finish for n frames, as (fade out, fade in, bars up done, bars down done), first with round-to-nearest single precision and then with the console's round-toward-zero adds with one guard bit (SPEC-0008, Platform arithmetic): n = 5: (5, 6, 6, 6) and (6, 6, 6, 6); n = 6: (6, 6, 7, 6) and (7, 6, 7, 7); n = 8: (8, 8, 9, 9) and (8, 8, 9, 9); n = 10, 18, 20, 24, 30, 36 or 42: (n, n, n, n) and (n + 1, n, n + 1, n); n = 37, 41, 48, 60 or 120: (n + 1, n + 1, n + 1, n + 1) and (n + 1, n + 1, n + 1, n + 1). The console FPU adds this way, so the second set is the original timing (Medium-High). A clean build may use exact integer counts (n updates) and accept the one-tick difference (Open question 19).
- Bars held (after LetterBoxOn), then OpenLetterBox(0.5 s): progress does not change.
- Bar geometry: mode 1 at progress 0.5: both bars 112 px. Mode 2 at progress 1.5 with heights 32: both bars 128.5 px.
- Teleport sequence: heights 256, bars up over 10 frames, then bars down over 10 frames in the same tick: after three updates progress is 0, nothing has been drawn and done is 1. A LetterBoxOn(0.5 s) afterwards: 128 px bars after update 15 and 256 px after update 30, covering rows 0 to 256 and 192 to 448 (the whole screen).
- Track sampling, pack 985 track 25: t = 0 gives row0 (0.96582, 0, -0.2592), row3 (-645.898, 128.377, -597.216, 1) and focal 791.448; t = 1.0 gives row0 (0.93592, 0, -0.35221) and row3 (-645.100, 128.170, -593.178, 1); t = 1.93 is valid; t = 29/15 (1.9333334) and above give no sample.
- Synthetic tracks: N = 1, any t: no sample. N = 0: the head pose and head focal at any t.
- Quaternion lerp: qA = (0, 0, 0, 1), qB = (0, 0, 0, -1), f = 0.5: qB is flipped to (0, 0, 0, 1), so q = (0, 0, 0, 1) and the matrix is the identity.
- Locator sample (flag 0): identity basis, P (10, 20, 30), V 0, L (1, 2, 3, 1): result (1, 2, 3, 1). Basis rows (0, 0, -1), (0, 1, 0), (1, 0, 0), P (10, 20, 30), V 0, L (11, 22, 33, 1): result (13, 22, 29, 1). Identity basis, P (10, 20, 30), V (5, 6, 7, 8), L (1, 2, 3, 1): result (6, 8, 10, 9). Basis scaled by 2, P (10, 20, 30), V 0, L (11, 22, 33, 1): result (12, 24, 36, 1). L.w = 0.625 with V 0: result w 0.625. In every case the target's second vector becomes (0, 0, 0, 1).
- Hero state 0x37, RestorePlayerControl: state becomes 1. Hero state 0x32 (walking), RestorePlayerControl: state stays 0x32.
- Hero with a control record in state 1, BlockPlayerControl then RestorePlayerControl: state 0x21, then state 1.
- Locator count 30: id 29 is valid, id 30 is not.
- PlayCamera on a track with 0 keys, wait flag set: the thread does not yield for the wait.
- FOV: focal 791.45 gives 2 atan(320 / 791.45) = 44.0 degrees; focal 512 gives 64.0 degrees.
- Final fight (synthetic): entry plays movie 78 and sets the clock to 180 s (10,800 ticks). The clock reaches zero before the win: movie 80, then game over; movie 85 does not play. The boss is defeated first and the finale trigger fires with an object present: movie 85; movie 80 does not play.
- Movie 4 block (synthetic): flag 136 clear and leg flags 0 to 2 set: the thread starts at leg 3 and walks legs 3 to 8. After leg 8 flag 136 is set, item 32 is given and movie 4 is queued. On a later load with flag 136 set, the block does not run.
- Module 990 tint: lever word -1, trigger 3 pressed with object 12 inactive: camera track 5 plays with no tint. One press at trigger 6 (word +1), then trigger 4 pressed with object 14 inactive: track 4 plays under bars in mode 1 at progress 1, colour (0, 0, 128) and alpha 20. On the first update after the camera returns, the bars auto-clear and progress is 0.
- Fade colour word: bytes R 255, G 64, B 32 are passed as one colour word with R in the lowest byte, then G, then B, and 0 in the top byte.
- Pack 1411 (synthetic): flags 337 and 338 both clear: neither 71 nor 72 is queued.

## Open questions

Numbers of removed questions are not reused.

1. The meaning of the source vector at +0x80 in the locator sample. Answered: the sample rule and the target's second vector (0, 0, 0, 1) are in Locator moves.
2. Answered: PlayCamera takes (track, WAIT, end mode, speed); see SPEC-0030 (row 34) and Camera track playback.
3. Resolved: the screen effect draws the bars itself as two full-width rectangles, progress x 32 pixels high in mode 0, progress x 224 in mode 1, and swept from 32 to 225 in mode 2 (Screen effect: per-pass update and draw).
4. Resolved: the update runs once per main-loop pass, before the front end draws the HUD, so the HUD is drawn over it (Screen effect: per-pass update and draw).
5. Resolved: the tether is the target object's locator link (a locator record, plus a host object for the animated form). On every object update the base of the target's position is recomputed from that link: the locator's current position, with the host's pose applied when a host is named, and for the static form the record's position point; the object's root animation is added to the base, and the +0x90 offset as well when the mode byte is 1 or 3 (SPEC-0028, section 7.1, step 5; SPEC-0030 rows 124 and 130). The target therefore follows the locator for as long as the link stays (High).
7. Locator record bytes other than the position, the w component and the array B flag: no sampled move uses them (Open question 15), so they have no effect on cutscenes; their authored meaning is not encoded (unknowable). The rewrite stores them and ignores them.
8. Resolved: 0x19 is the talking state, 0x33 the scripted one-shot animation and 0x37 the scripted looped animation (Control states, above). The state-code discrepancy with SPEC-0033 does not exist; SPEC-0033 uses decimal numbers (High). Test vector: PlayHeroAnim sets state 51 (0x33); at progress 0.995 with the flag word +0x3D0 equal to 0 the state becomes 33 (0x21); with the flag word 2 it becomes 1. LoopHeroAnim with a counter of 3 sets state 55 (0x37); after three ticks the state becomes 33 (flag zero) or 1.
9. Resolved (Medium-High): module 985 does not use RestorePlayerControl; the end of track 29 sets the hero to state 1 (Worked example: resource 985).
10. PlayEntityAnim after its wait. Resolved (High): the second animation request is a restart into the entity's idle, not a hold of the end pose. When the played clip reaches 0.99 progress, the caller asks the shared idle picker for the entity's idle animation id and requests it with the default blend (no clip is held). The picker returns 0 for a ground entity (97 for a flyer) whose requests are not held; 1 (141 for a flyer) when the entity is in agro mode (class state 19); and, outside agro mode, 0 whatever the flyer state when the entity's request-hold word is set. Because the finished clip is not the idle, the picker also redraws the fidget countdown as a whole number from 120 to 600 ticks, and the idle variants (115 to 119, or 142 for a flyer) can only start after the plain idle has been the current clip for that long. With WAIT other than 1 the call returns at once and makes no second request; what follows the clip is then the rule of state 0x21 (SPEC-0030, row 105). Test vector: PlayEntityAnim(e, a, WAIT 1, CTL 0) on a ground entity in a normal state: when clip a reaches 0.99, animation 0 is requested and the state is 0x21; with CTL 1 the state is 1; a flyer gets animation 97; the same call on an entity in agro mode gets animation 1.
11. Resolved: LoopEntityAnim stores a tick budget n on the entity: n = trunc(seconds x 60) (one single-precision multiply, truncated) for seconds of 0 or more, and n = 1 for any negative seconds; a result of n = 0 becomes -1, which means no limit. So 0 seconds, and any positive time below 1/60 s, loops until released. The entity is put in the scripted looped state (55) (High). Test vectors: 0.5 s gives 30; 0.7 s gives 41; 0 s gives -1 (no limit); 0.005 s gives -1; -1.0 s gives 1.
12. Resolved: module 1217's Select branch never runs, because its button test is constant false (Skipping).
13. Resolved: LevelMsg with wait value 1 yields while the dialogue box is active (Threads and waiting).
14. Resolved: CloseLetterBox's target 2 is the mode 2 letterbox-to-full-close sweep (Screen effect: requests).
15. Resolved: no. The static form MoveObjToLocator(object, locator, link) has no source object and does no sampling: with link 0 it sets the target's position to the locator record's four position floats (w included) and its second vector to (0, 0, 0, 1); with link nonzero it stores the locator link instead (Open question 5). Nothing from a source object's basis rows, vector or locator w is used (High). Test vector: locator (10, 2, -3, 1) with link 0 puts the target at (10, 2, -3, 1).
16. Resolved: the object field comes from the selected hero's trigger-hit path, which fires with a nonzero object value, so movie 85 plays after the boss is defeated (Movies in scenes). **[High]**
17. Resolved: flag 337 has an ordinary setter with a fixed flag number. Pack 1382 sets it with a fixed argument, together with flag 98, once all six elemental pedestals are lit. The setter sits behind a module counter that is 0 when the module loads. Movies 71 and 72 then play in pack 1411 as described in Movies in scenes (High).
18. Resolved: the block ends an eight-leg escort thread, started by trigger 17 or by the area thread under flags 10, 11, 127 and 136. Flag 136 has no other setter, so it plays at most once per game (Movies in scenes). **[Medium-High]**
19. Resolved (Medium-High): fades and bars finish on the update given in Test vectors (round toward zero; SPEC-0008, Platform arithmetic).
20. The teleport curtain (Edge cases): the arithmetic of the two back-to-back requests is fixed by the screen-effect rules (progress stays 0, so no curtain is drawn; Medium-High), and only the on-screen look is unconfirmed (needs a runtime check). Default if unresolved (rewrite choice): implement the request arithmetic as specified in Edge cases and draw nothing for the curtain. Answers: proactive (open close-out).
21. Resolved (High for the construction, Medium-High for the sign because the cross-product order was confirmed against the machine code of the vector unit): the look-at orientation of target plays uses the camera frame of SPEC-0027 (rows right, up, forward; the camera looks along its third row). forward = (target anchor - eye) normalised. A first side axis is the cross product (sampled up row x forward), normalised, where the sampled up row is the second row of the orientation the track key gives at that time. When the sum of that axis's |x| and |z| exceeds 0.01, its y is set to 0 (it is not renormalised here); otherwise it is kept as it is. Then up = (forward x side) normalised, and the stored right row = up x forward, so the three rows are orthonormal; the stored rows are right, up, forward, each with fourth lane 0. With a sampled up row of (0, 1, 0) the right row is (forward z, 0, -forward x) normalised, and up has a positive y. The sampled up row is not forced to world up: a key whose orientation is tilted gives a tilted frame. The type-28 head byte +0x28 has no encoded meaning (unknowable); ignore it.
22. Resolved: every fade site has a fixed colour. Out of 510 sites, 463 are black, 37 white, 8 purple and 2 orange (Screen effect: requests). **[High]**
23. Resolved: triggers 3 to 5 with the prompt showing (object 12, 14 or 16 inactive), button 0 pressed and the lever word at +1. The first pair tints the camera track dark blue and the second clears it (Screen effect: requests). **[High]**
