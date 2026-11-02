# SPEC-0053: Hero switching and the party state

- Answers: R-0042 (in part; the Act 2 start module 1174 is in SPEC-0044); extends the hero rows of SPEC-0030: rows 51, 162, 167, 173, 178, 180, 186, 187; proactive (open close-out); R-0178 (swap mode across a Continue)
- Status: Draft
- Disc build: SCUS-97367 v1.00 (same fingerprint as SPEC-0001). The HUD portrait words in section 2.5 and the hero resource table in section 2.6 were read from the boot program's data.
- Confidence: per section (High, Medium-High, Medium, Low).

## Summary

The game has two playable heroes, Tor (hero index 0) and Roberta (hero index 1). Each hero is a fixed record of 0x1680 bytes, and exactly one of them is selected (controlled) at a time. Only the selected hero is in the world: the other one (the parked hero) is not updated, not drawn and has no character data resident.

A switch is a request. The request checks the swap mode and the target hero, updates the HUD hero portraits (accepted or not), stores one shared request record, sets an in-progress flag and starts a swap task. The swap task carries the outgoing hero's action state to the incoming hero, reloads the shared hero memory pool with the incoming hero's resources, gives the incoming hero the outgoing hero's place, selects it, re-attaches the camera to it, and holds for 41 ticks (0.7 s nominal, about 0.683 s) before it clears the in-progress flag. A switch does not move the camera eye: the view stays where it was and the follow resumes from there.

The swap mode is a lock on switching. Mode 0 keeps control on Tor, mode 1 keeps it on Roberta, mode 2 allows switching both ways, modes 3 and 5 refuse every switch, and mode 4 allows switching but places the incoming hero at its own stored spot. Play starts in the mode given by story flags 220 and 248 (section 2.3).

Switches are started by level scripts (SwitchHero, ActivateHero and the act-start setup), by the player with the L1 and R1 chord (section 3.8 c), when the selected hero goes down with the partner alive (section 3.8 d), and by the Potion of Revival (section 3.8 b).

There is no companion AI for the parked hero and no co-op or second-controller control of a hero.

## Where it lives on the disc

- The HUD portrait words in section 2.5 are 32-bit IEEE singles and 32-bit integers in the boot program's data. They are read by virtual address (SPEC-0001 gives the mapping).
- The hero resource table (section 2.6) is in the boot program's data at 0x313388.
- Each hero's six character resources are archive resources (SPEC-0001): Tor 492 to 497, Roberta 498 to 503.
- The hero records, the selector, the swap-state object and the shared request record are run-time memory. They are created at start-up and are not on the disc. The clean side keeps its own equivalents with the layouts below.
- Level scripts reach this behavior through the host calls listed in section 3.3 (SPEC-0030).

## Layout

### 2.1 Hero records

Tor's record is the first record of the hero table. Roberta's record starts 0x1680 bytes after it. Offsets are from the start of a hero record.

| Offset | Size | Type | Meaning | Per hero | Confidence |
| --- | --- | --- | --- | --- | --- |
| +0x14C | u32 | integer | Hero index: 0 = Tor, 1 = Roberta. Used by the speaker name rule and the hero-name cue. | per hero, fixed | High |
| +0x144 | u32 | integer | Action state (see SPEC-0033 and SPEC-0036 for the state numbers). | per hero | High |
| +0x154 | s32 | integer | Current hit points. | per hero | High |
| +0x158 | s32 | integer | Maximum hit points. | per hero | High |
| +0x15C, +0x160 | s32 | integer | Current and maximum magic pool (the second pool beside health; SPEC-0036, Magic pool). The per-hero reset copies the maximum to the current. The HUD magic bar reads both words and is hidden when the maximum is 0. | per hero | High |
| +0x164 | pointer | table | Class method table. Used slots: set state, the dead test (true when the action state equals 16, the death state), a one-shot cue (object, cue id), and a hit-point-to-maximum hook. | per hero | High |
| +0x168 | pointer | record | Party inventory (one record shared by both heroes; SPEC-0021). Not a sound object. | same record in both | High |
| +0x16C | pointer | record | Placement and motion record: position and facing vectors; +0x24 height above the ground; +0x150 grounded word (set on landings, cleared on take-offs; SPEC-0040). | per hero | High (use), Medium (field names) |
| +0x170 | pointer | record | Locomotion sub-object. Its 64-byte block at +0x290 is the model matrix (rotation rows and position) that the draw hook uses; it is carried to the incoming hero on a switch. The locomotion enable word is at +0xFA0 (decimal 4000) of the hero record. | per hero | Medium-High |
| +0x3D0 | u32 | integer | End-of-action flag: 1 when the third argument of a script hero-animation call (play or loop) equals 1, else 0; scripted hero moves store their own argument; 1 after hero construction. The action-end resolver reads it: nonzero returns the hero to state 1, zero sends it to state 33 (SPEC-0033, Behavior 3A). | per hero | High |
| +0x7B4 | s16 | integer | Area key. Compared with the world's current key in mode 4. | per hero | Medium |
| +0x7B8 | pointer | record | Area resource pointer, set with the area key. | per hero | Medium |
| +0x7CC | block | record | Locomotion block (SPEC-0033): +0x0C stick speed value v, +0x40 heading. | per hero | High |
| +0x9A0 | block | record | Ground-reference record. The camera's followed player word points here while the hero is selected (SPEC-0027). | per hero | Medium |
| +0xA70 | 15 x 0x1C | records | Target list: up to 15 nearby characters, with the count of used records at +0xC14. Record: +0x00 character id (-1 = empty); +0x04 a counter raised each time that id is added to the hero's ten-entry list at +0x400; +0x08 distance from the hero; +0x0C tick counter (used in the first record as the rescan clock); +0x10 candidate flag (1 = in range, in front and with a clear path); +0x14 dot of the hero's forward with the direction to the character. The select step empties the list (count 0; every record id -1 and the other four words 0). Scan rule in section 3.9. | per hero | Medium (layout High) |
| +0xF80 | u32 | integer | Attack variant (0 Tor, 1 Roberta). See SPEC-0036. | per hero | High |
| +0xF90 | u32 | integer | Charge sub-state. See SPEC-0036. | per hero | High |
| +0xFA4 | u32 | integer | Stored charge power. See SPEC-0036. | per hero | High |
| +0x1544, +0x1548, +0x154C | s32 | integer | Equipment words 0, 1 and 2 (the three equipped Motes; SPEC-0021, Equipment slots): an item id, or -1 when empty. | per hero | High |
| +0x155C | s32 | integer | Quick-item selection marker (SPEC-0021). | per hero | Medium-High |

### 2.2 Selector

The selector is one run-time block: the game-globals block at data address 0x35C7B0, filled once by the start-up step that creates the global objects (High). It is outside the file image, so the clean side builds its own.

| Offset | Meaning |
| --- | --- |
| +0x20 | Base of the hero table (Tor's record), data address 0x35C7D0. It holds 0x335270 from start-up. |
| +0x24 | Selected pointer, data address 0x35C7D4: 0x335270 + 0x1680 × hero index. It equals the base when Tor is selected, and the base plus 0x1680 when Roberta is selected. Rewritten only by the hero-switch and load routines. |
| +0x04 | Pointer to the camera object, data address 0x35C7B4 (written once at start-up; it does not change). A switch re-attaches the camera to the incoming hero (section 3.6b). |

Index -1 in the host calls means "the selected hero".

### 2.3 Swap-state object

One run-time block holds the swap state. It is the game-globals record at data address 0x35C980, so the in-progress flag below is the game-global word 0x35C980 + 0x44 that the game-state code and the level scripts read (SPEC-0026, SPEC-0030) (High).

| Offset | Size | Meaning | Confidence |
| --- | --- | --- | --- |
| +0x08 | u32 | Swap mode (section 3.1). The first use of the object sets it to 2, but the boot set-up then sets it by rule M below, so play starts in mode 0. Rule M, run at boot and at every new-game and start-point world load: story flag 220 clear gives mode 0; flag 220 set and flag 248 clear gives mode 1; both set give mode 2. A load sets the mode from save word 0x307C (SPEC-0025); so does the Continue after a game over when a save or load is pending, which re-applies the session snapshot, so a boss fight that had changed the mode (a locked mode, for example) does not carry its mode over a Continue (SPEC-0055, section 4). Level scripts (SetHeroSwapMode) and the act-start setup write it at other times. | High |
| +0x20 | u32 | Flag D: set when a box or list picker ran this pass (it gives game-state code 4, SPEC-0026). Read by the in-play controls gate (section 3.13). | High |
| +0x44 | u32 | In-progress flag: 1 from acceptance of a request until the swap task ends, else 0. It gives game-state code 5 (SPEC-0026). | High |
| +0x4C | u32 | Set to 0 together with +0x44 when a request is accepted and when the swap ends. It is one of the accumulators of the per-pass status update, which adds dt to it on every pass unless it holds -1, so it counts seconds since the last accept or swap end (High for the update; Medium for the meaning). | Medium |
| +0x5C | u32 | Game-state code (SPEC-0026). The in-play controls gate needs it to be 0. | High |

### 2.4 Shared request record

Only one request record exists. Words are 32-bit, indexed from 0.

| Word | Meaning |
| --- | --- |
| 0 | Target hero record (the incoming hero). |
| 1 | Direction bit: 1 when the incoming hero is Roberta. |
| 2 | Extra flag from the caller: 0 from SwitchHero and ActivateHero; 1 from the chord (3.8 c), the hero-down request (3.8 d) and the Potion of Revival (3.8 b). |
| 3 | First boolean flag (start sound and the three swap effects, section 3.5). |
| 4 | Second boolean flag (white fade-out and fade-in, section 3.5). |

### 2.5 HUD portrait words (boot program data)

These words are not camera values. They belong to the HUD hero-portrait swing (SPEC-0023 section 12), which every switch request sets (section 3.6).

| Address | Type | Value | Use |
| --- | --- | --- | --- |
| 0x2DCEE8 | f32 | 0.5 | Swing length, in seconds. |
| 0x2DCEF0 | f32 | -0.5236 (-30 degrees) | Start angle of the swing when the incoming hero is Tor. |
| 0x2DCEF4 | f32 | 2.617993 (+150 degrees) | Start angle of the swing when the incoming hero is Roberta. |
| 0x2DCEF8 | f32 | 0.5 | Start blend when the incoming hero is Tor (for Roberta the blend starts at 1.0). |
| 0x2DCEFC | f32 | 2.0 | Hold length in seconds after an accepted request: 2.0 x 60 = 120 ticks. |
| 0x2DCF00 | f32 | 4.0 | Hold length in seconds after a refused request: 4.0 x 60 = 240 ticks. |
| 0x2DCF08 | u32 | 1 | Tor's portrait "available" word (0 = portrait dimmed). |
| 0x2DCF0C | u32 | 1 | Roberta's portrait "available" word. |

The run-time swing state (angle, end angle, angle step, blend, blend step, hold counter) is zero in the file image (words from 0x2DCED4 to 0x2DCEE4).

Tick rate: the game runs at 60 ticks per second (SPEC-0007). Times in seconds become ticks as trunc(seconds x 60); the stored hold is the word times the single-precision tick-rate word.

### 2.6 Hero resource table (boot program data)

At 0x313388. Rows are 8 bytes apart; each row starts with two u16 archive resource numbers, Tor's then Roberta's. The first row's loader also loads the next number. So the six resources of a hero are Tor 492 to 497 and Roberta 498 to 503 (High for the numbers; Medium-High for the per-row roles, which follow the model-set loaders of SPEC-0005).

## Behavior

### 3.1 Swap modes (the lock)

The mode is read when a request is made, and once more by the swap task at step 7 of section 3.4.

| Mode | Effect on a request | Label |
| --- | --- | --- |
| 0 | Refused while Tor is selected. A request from Roberta to Tor is allowed. | Locked to Tor |
| 1 | Refused while Roberta is selected. A request from Tor to Roberta is allowed. | Locked to Roberta |
| 2 | Allowed both ways. | Free |
| 3 | Every request refused. | Blocked |
| 4 | Allowed both ways. The swap task places the incoming hero at its own stored spot (section 3.4, step 7). | Free, own placement ("reactivate main player") |
| 5 | Every request refused. | Blocked |

The modes written by any routine are 0, 1, 2, 3 and 5: rule M (section 2.3), a load, SetHeroSwapMode, the act-start setup and the level scripts. Mode 4 has no writer: every SetHeroSwapMode call in the level scripts passes a constant 0, 1, 2, 3 or 5 (Open question 1). No routine forces a swap when the mode changes.

Confidence: High for the guard table and the writers; Medium-High for the labels and for mode 4 being unused.

### 3.2 Starting a switch (request)

The request has these steps, in order. Let S be the selected hero and T the other hero (the target).

1. If the mode is 3 or 5, the request is refused.
2. If the mode is 0 and S is Tor, it is refused. If the mode is 1 and S is Roberta, it is refused.
3. If T is dead (its action state is 16), it is refused.
4. Update the HUD portraits (section 3.6). This happens on every request, accepted or refused.
5. If accepted:
   - fill the shared request record (section 2.4): target T, direction bit, the extra flag, and the two flags;
   - set the in-progress flag to 1 and the word at +0x4C to 0;
   - start the swap task in the cooperative task slot 1 of the global task system, with task mask 3. If that slot is busy, the start fails silently (section 5, Edge cases).
6. Return 1 when accepted (also when the task start failed), 0 when refused.

SwitchHero(h, a, b) passes a and b as the two flags and 0 as the extra flag. If h is -1 (the selected hero) or already the selected hero, the call makes no request. After the request step, SwitchHero clears the party weapon-drawn flag on every call, also when it makes no request; when the flag was set this plays the sheathe sound (SPEC-0040, held items).

On a refusal the HUD portraits are still updated (section 3.6): the incoming portrait is dimmed and, in mode 2, the portrait group is shown for 4 s.

Confidence: High.

### 3.3 Script entry points

These are the host calls that touch switching. Their argument lists are in SPEC-0030.

| Call | Effect in plain words | SPEC-0030 row |
| --- | --- | --- |
| SetHeroSwapMode | Writes the swap mode (section 3.1). | 167 |
| GetHeroSwapMode | Returns the swap mode. | 186 |
| SwitchHero(h, a, b) | Requests a switch to h (section 3.2). Does nothing if h is selected. Always clears the party weapon-drawn flag. | 173 |
| ActivateHero(h, loc, block, fade, cue) | Moves the hero selected at the time of the call (not h) at once to locator loc: ground probe on, the locator's facing, state 1. Then requests a switch to h with fade and cue as SwitchHero's two flags (3.5). Then, when block is set, blocks h at once. Nothing is deferred in the call. If the switch is accepted, step 7 of 3.4 copies the outgoing hero's placement (the locator) to h (mode 4: h's own stored spot). Both disc calls (module 1244) pass false for block, fade and cue. | 187 |
| TriggerActStart(stage) | Act-start setup by stage (section 3.7). | 51 |
| IsHeroSwapInProgress | Returns 1 from acceptance until the swap ends, else 0. | 180 |
| WhichHeroIsActive | True when Roberta is the selected hero. | 162 |
| GrabHeroControl | Sets the selected hero's state to 0x21 (script control). | 164 |
| BlockPlayerControl(h) | For hero h, when it has a locomotion sub-object: runs the lock step (one-shot state 0x33, then a wait) and then sets state 0x21. | 160 |
| RestorePlayerControl(h) | Sets hero h's state to 1 when its state is 0x19, 0x21, 0x33 or 0x37. It does not return player input. | 166 |

### 3.4 The swap task, in order

The swap task is a cooperative task that yields once per tick. It runs in the global task system with mask 3, so it runs on every pass in every game-state code, also under code 9 (SPEC-0026, main loop step 9). The memory-card tasks use mask 3 too. While the in-progress flag is set the game-state code is 5 unless a higher rule matches: the world keeps running, the level scripts' steps 2 to 5 are skipped while their special threads still run (SPEC-0030), IsButtonPressed returns false, and the in-play controls gate fails, so the chord cannot start another switch (High). Let O be the selected hero when the task starts (the outgoing hero) and I the incoming hero, taken from word 0 of the request record at each use.

1. **Carry the state.** If O is alive and O's state is not 0x12 or 0x13, the carried state is O's current state. Otherwise the carried state is 1 (idle). A dead outgoing hero (hero down) therefore gives carried state 1.
2. **Start cue.** Reset all 17 hero attack-family pools (the projectiles and attack objects of SPEC-0036); this happens on every swap. If the first flag is set, play sound 0xB0, make the three swap effects at O's position and start the first one (section 3.5). If the second flag is set, start the white fade-out (section 3.5). (High for the calls; Medium for the meaning of the pools.)
3. **Outgoing one-shot.** If O is alive: when the first flag is set, sound 0xB0 is played a second time (Medium); then play O's one-shot cue 0xB1 and set O's state to 0x33 (one-shot). A dead O plays no exit one-shot.
4. **Wait, then locomotion off.** Wait while O's state is 0x33. Then clear O's locomotion enable word (+0xFA0) and release its locomotion at once, so O is no longer updated. Then wait at least three ticks, and until the transition-busy predicate is false and the fade alpha of the screen effect equals its target (SPEC-0052), so a running white fade-out has reached full opacity. The transition-busy predicate is true while any of the three world buffers (transition, exterior, interior; SPEC-0006) is loading, including its linked sub-world and any of its character slots, while any of the four held-model display slots (SPEC-0021) is loading, or while the disc file reader is not idle. It does not test the camera or the fade; the fade is the separate alpha test. (High for the order; Medium-High for the predicate terms.)
5. **Sound settle.** If the sound system is running, wait for one video field, then service the sound processor's command channel; repeat until the channel has reported no pending transfer on 10 fields in a row. The count restarts at 0 on any field with a pending transfer. The task does not yield here, so the whole game pass stops: at least 10 fields (about 0.17 s) of frozen picture, longer while sound commands are still being sent. There is no timeout; it can only stall if a transfer never completes. With the sound system off the step does nothing. The same wait is used before other loads into shared memory. (Medium-High.)
6. **Bind I.** Bind I by its index: the shared hero memory pool is rewound and I's six character resources (section 2.6) are loaded into it, then I is bound to the per-slot sub-objects. Only one hero's character data is resident at a time. (Medium-High.)
7. **Placement by mode.**
   - Mode 4: if I's area key equals the world's current area key, place I at its own stored placement. Otherwise nothing is placed; the only other action is a debug print, which has no effect.
   - Any other mode: I takes O's area key and area resource (set only when they differ), O's position and facing through the placement helper, O's placement vectors, the 64-byte model matrix at +0x290 of O's locomotion sub-object (rotation rows and position), and O's locomotion heading.
   (High.)
8. **Incoming one-shot prepared.** Set I's state to 0x33 (one-shot).
9. **Select.** If O is not I: deactivate O (section 3.9). Then attach the camera to I (section 3.6b), set the camera's ground reference to I's, set the selected pointer to I, clear all 14 of I's status timers (status apply with kind "all"), and empty I's target list at +0xA70 (section 2.1). (High.)
10. **End flags.** If the first flag is set, stop the second swap effect and start the third (section 3.5). If the second flag is set, start the fade-in from white back to clear over 8 ticks (section 3.5).
11. **Incoming one-shot.** Play I's one-shot cue 0xB0 and set I's state to 0x33 again.
12. **Wait for the outgoing one-shot.** Wait until O's state is not 0x33 or the fade alpha equals its target: the wait ends on either condition (High).
13. **Carried state.** If the extra flag in the request is set, the six HUD visibility timers C and G0 to G4 (SPEC-0023 section 4) are set to 240 ticks. Then set I's state to the carried state from step 1. (High.)
14. **Cloak.** If I wears the Tattered Cloak (item 126 in its body entry), the party weapon-drawn and shield-raised flags are cleared (the sheathe sound plays if the weapon-drawn flag was set), and for Tor the two held-item slots are cleared. (Medium-High.)
15. **Hold.** Wait trunc(0.7 x F) ticks, where F is the tick-rate word: 41 ticks. The 0.7 is stored as the single-precision constant 0.699999988 and F is the single-precision tick-rate word (60.0). The product is formed by one single-precision multiply, and the console's multiply truncates its result toward zero (SPEC-0008, Platform arithmetic): the exact product 41.99999928 becomes 41.999996, and the conversion to an integer truncates it to 41. The swap task calls no double-precision helper, so no double-precision arithmetic is involved. Round-to-nearest single precision would give 42.0 and 42 instead, so a clean build must apply the platform multiply rule here or use the fixed count 41. Confidence: High for single precision; Medium-High for the count of 41, which rests on the platform multiply rule. SPEC-0026 (Area transition job, step 9) describes the same hold.
16. **Finish.** Clear the in-progress flag (+0x44) and the word at +0x4C, release the three swap effect instances (whether or not the first flag was set), and end the task.

Notes:
- Steps 1 to 16 use the flags as described in section 3.5.
- The order of steps 9 to 11 is the order stated here, with the deactivation of O before the camera attach and the selection write (High).
- The 0.7 is a single-precision code constant. The 60 ticks per second come from the game tick (SPEC-0007), read as the single-precision tick-rate word.
- The request has an optional fifth argument (fade in from black with the guards skipped). No caller passes it.

Confidence: High for the order of waits and writes; Medium for the attack-pool reset of step 2 (what each of the 17 pools holds is in SPEC-0036) and for the repeated start sound of step 3 (whether the second play is audible needs a runtime check; the rewrite plays it).

### 3.5 The two boolean flags

The two booleans of SwitchHero are stored as the request's words 3 and 4.

| Flag | Set | Clear |
| --- | --- | --- |
| First (word 3) | At start, sound 0xB0 is played and three effect instances are made from the effect ids 0xFB9, 0xFBA and 0xFBB of effect bank 3 (effects: SPEC-0034). All three are placed at the outgoing hero's position, and the first (0xFB9) is started. At the end (step 10), the second (0xFBA) is stopped and the third (0xFBB) is started. The three instances are released when the task ends (step 16). | No sound, no swap effects. |
| Second (word 4) | At start, the screen fades to white (255, 255, 255) from clear to alpha 128 over 37 ticks. Alpha 128 is the top of the screen effect's 0 to 128 scale and is fully opaque (SPEC-0052), so the screen is solid white while the heroes are exchanged and the change of model is hidden (the camera does not move; section 3.6b). At the end, it fades from opaque white back to clear over 8 ticks. | No flash; the change of model is visible, with the same view. |

With both flags false the swap is silent: no sound, no swap effects and no flash. The silent form is what the level scripts use for the scripted story switches (pack 1394's lever triggers and the entry sequence of pack 1432). Four script call sites pass both flags true and so play the swap sound, the swap effects and the white flash: two in the round fight (pack 1371, after a hero's death) and two in the final fight (pack 1432, in its swap-after-death step).

Confidence: High for which word is read, for the three effect ids and the bank number, for the colour and the frame counts as arguments (the colour is read from the boot data at 0x320998); Medium for the start and stop roles of the effect calls and for the visible result of the effects. The keys 0xFB9 to 0xFBB are effect ids, not screen-tint records. The frame counts are game ticks: the screen effect is updated once per pass (SPEC-0052; High). In single precision the 37-tick fade-out reaches alpha 128 on the 38th update (37 with exact arithmetic) and the 8-tick fade-in reaches 0 on the 8th (Medium; SPEC-0052 Test vectors).

### 3.6 HUD portraits on every request

Every request updates the HUD hero portraits; it does not move the camera. Let d = 1 when the incoming hero is Roberta, and A = 1 when the request is accepted (the mode allows it and the partner is not dead).

- The outgoing hero's available word (0x2DCF08 for Tor, 0x2DCF0C for Roberta) is set to 1. The incoming hero's available word is set to 1 when A = 1 and to 0 otherwise.
- If the in-play controls gate (section 3.13) passes at the time of the request, the swing is made active.
- Accepted: start angle = -0.5236 (d = 0) or 2.617993 (d = 1); end angle = start + pi (3.141593); angle step = ((end - start) / 0.5) / 60 per tick; blend 0.5 with step +1/60 per tick (d = 0), or 1.0 with step -1/60 per tick (d = 1); hold = 2.0 x 60 = 120 ticks.
- Refused: the angles and the blend are not changed; hold = 4.0 x 60 = 240 ticks.
- The swing is drawn only while it is active and the swap mode is 2. A portrait is dimmed while its available word is 0. The per-tick swing update, the portrait placement and the drawing are in SPEC-0023 section 12.

HUD on a swap: the bars, dividers and panels read the selected hero every tick, so they show the incoming hero from the selection tick (step 9) on. When the extra flag is set (the chord, the hero-down request, the Potion of Revival), step 13 sets the six HUD visibility timers C and G0 to G4 to 240 ticks (SPEC-0023 section 4), so the bars and panels are shown for 4 s.

Confidence: High.

### 3.6b Camera hand-off

The only camera action of a swap is at step 9: the camera's followed hero is set to the incoming hero and its target point and facing inputs are taken from it, as in SPEC-0027 (Placement after a load or restart), but with the camera-object hook flag 0. The boot and world-load attaches pass 1. The camera reset follows, and the camera's ground-reference word is set to the incoming hero's ground-reference record (+0x9A0).

The hook is the camera's set-eye step (the pull-in of SPEC-0027): it refreshes the focus point, moves the eye to the attach point (position - d x facing), and pulls that point halfway toward the focus when a ground probe from 5 units above it finds ground higher than that probe point. It is also skipped whenever the camera word +0x50 is set, whatever the flag.

On a switch the hook does not run, so the eye is not moved. The attach point is computed into the camera's work vectors, but the reset then runs the eye's collision pass on the current eye and copies the current eye over those work vectors; the yaw words are recomputed from the current eye and focus. So the eye stays where it was on the tick before the selection, and the normal follow resumes from there. Since step 7 put the incoming hero on the outgoing hero's spot and facing, the view does not jump: there is no cut, no turn and no blend, with either flag setting. The white flash (section 3.5) hides only the change of model.

Confidence: High for the steps and for which word the reset copies; the hook's probe and pull rule are skipped on a switch and so do not affect the result (their detail belongs to SPEC-0027); Medium for the visible result (needs a runtime check of the on-screen look).

### 3.7 Act-start setup by stage (TriggerActStart)

The control flow is High.

| Stage | Swap mode set | Selected hero after | Story flag set | Heal | Notes |
| --- | --- | --- | --- | --- | --- |
| 1 | 0 (locked to Tor) | Tor, by a normal switch request (section 3.4 runs) | none | none | Does not set mode 2 first. The request passes both sound/effect flags clear (no swap sound, no effects, no white flash) and the extra flag clear (no HUD timer reset) (High). |
| 2 | 1 (locked to Roberta) | Roberta, set directly (placement and select, no task) | 220 | Roberta only | Also runs the scene-set helpers. |
| 3 | 2 (free) | Tor, set directly | 248 | both | |
| 4 | 2 (free) | unchanged | 332 | both | |
| other | 2 (free) | unchanged | none | both | |

The heal sets the current hit points of each hero in the mask to the maximum and clears that hero's status timers. Bit 1 of the mask is Tor and bit 2 is Roberta. So stage 2 heals Roberta only.

The values 220, 248 and 332 are story flag numbers (High). Flags 220 and 248 are the ones rule M reads (section 2.3), so after stage 2 and a reload of the start point the mode is 1, and after stage 3 it is 2.

### 3.8 Routes that request a switch in play

Four routes in play can make a switch request. Route (c) is the player's switch, made with the L1 and R1 chord. Route (a) is a set of triggers in one level module, route (b) is the Potion of Revival, and route (d) is the forced switch when the selected hero goes down.

(a) **Square-button triggers in one level module.** Four triggers (indices 15 to 18) each run a hero move, an object swap, a camera change, set mode 2, SwitchHero, set mode 3, and fade and timer calls. Their gate is a clear once-word, a clear busy word and a new Square press while the prompt shows (Open question 4). Hero argument 1 for indices 15 and 18 and 0 for 16 and 17. Confidence: High for the gate and the hero arguments; Medium for the sequence order.

(b) **Potion of Revival (item 50).** Using item 50 (use-item kind 0x32) with hero index 0 or 1, in mode 2 with the partner dead: the partner is set to state 1 and healed to its maximum, then a switch to it is requested with the extra flag 1 and both flags equal to the caller's flag. On success a fade and sound helper runs. The revived hero is always the partner of the selected hero. Confidence: High.

(c) **The L1 and R1 chord (the player's switch in play).** The in-play state runs this test once per tick when it reaches it. It reads the held word (+0x890) and the raw newly-pressed word (+0x894) of the menu snapshot (SPEC-0033 Controller map), and tests bit 0x0004 (L1) and bit 0x0008 (R1). The snapshot is the OR over the controlling set, which is always pad slot 0 (controller port 1). The rules are:
1. The chord is present when one of L1 or R1 is newly pressed on this tick and the other is held on this tick, or when both are newly pressed on this tick. Holding both without a new press does nothing. Holding one without a new press of the other does nothing.
2. If the chord is present, the guards of section 3.12 must all pass. If one fails, nothing happens. A refused press is not retried on a later tick, because the new press lasts one tick.
3. Then request a switch to the other hero with the extra flag set to 1 and both flags set to 1 (section 3.2). One chord makes one request.
4. A refused request makes no sound and no flash; it still updates the HUD portraits (section 3.6).

Confidence: High for the bits; Medium-High for the chord reading; High for the guards (section 3.12). Order in the tick (High for the order; Medium for the consequence): the chord is tested in the front-end in-play handler, after the hero's button chain (SPEC-0033 Behavior 1A) in the same tick. The chain may move the hero to state 0x14 (20, stance steering) on the L1 press; the switch-ready gate (section 3.12) accepts state 0x14, so the chord still passes. The in-play state refreshes the quick-item display every tick in any case, so the chord's own refresh call changes nothing.

Where the chord sits in the in-play state (High; SPEC-0035 section 6). On each pass the state runs, in order: the credits gate (story bit 368), the book reader gate, the game-over gate, the play-active write, then the menu openers on the rising-edge word: Start (item screen) first, then Select or pad slot 0 not usable (pause page). An opener returns only when the busy check is zero; otherwise the pass goes on. After the HUD work, when the overlay box is idle and the in-play controls gate (section 3.13) passes and both quick-panel flags are clear, the chord test runs, and then the D-pad Left (map) and Right (journal) shortcuts. The chord's result does not stop the pass, so a shortcut pressed on the same tick still opens its page. So:
- A pass that leaves through a gate or an opener never reaches the chord. Only a new press of Start or Select (or a pad that is not usable) does this; holding Start or Select does not, because the openers read new presses only.
- A chord press on the same tick as a new Start or Select press is lost when the busy check is zero (the menu opens instead) and is tested when the busy check is set.
- Holding L1 while standing still blocks nothing: no gate reads L1.
- Test vector: tick with L1 held, R1 newly pressed and Start newly pressed, busy check zero: the item screen opens and no switch is requested. The same tick with the busy check set: Start does nothing and the switch is requested.

Route (c) answers Open question 2: the player's switch in play is the L1 and R1 chord. A search of pad routes alone misses it, because the mask is applied through a helper on the snapshot and not to a single pad word.

(d) **Hero down.** While the selected hero is in the death state (16) with the death clip at progress 0.95 or more on any of its layers, its death handler requests a switch to the partner on every tick (both flags 1, extra flag 1), unless the partner is dead. The first accepted request starts the task; later ones overwrite the same record with the same target until step 9 deselects the dead hero. A refused request (modes 3 and 5, and modes 0 and 1 in the locked direction) or a dead partner starts game over (SPEC-0055 8A). The dead outgoing hero gives carried state 1 and plays no exit one-shot. Confidence: High for the calls; Medium for the per-tick repeat.

### 3.9 The parked hero

The parked hero is not in the world. It is not updated, not drawn and not followed.

- Only one hero's character data is resident: each swap rewinds the one shared hero memory pool and loads the incoming hero's six resources into it (step 6).
- The world loop updates only the selected hero, and only while its placement record, its locomotion sub-object and its locomotion enable word are set; the swap clears the enable word on the outgoing hero (step 4). Only the selected hero gets the draw hook (the other draw-hook calls are the second player slot and the actor list).
- Deactivating the outgoing hero (when it is not the incoming hero) stops its status display: the shown status is cleared, its emitters are cleared and its tint is set to white (1, 1, 1). Its status timers are not changed. Confidence: Medium-High.
- Its placement record keeps the place where it was swapped out; a mode-4 swap would bring it back there if the area is the same. Its status timers stay frozen and are cleared when it next comes in (step 9).
- Nothing can hit the parked hero. Every enemy and boss attack handler is given the selected hero as its target, the enemy target finders read only the selected hero, the module damage zones and DamageHero act on the selected hero, and only the selected hero runs the trigger and contact step. The parked hero's placement record keeps its spot but no hit routine looks it up there. Its hit points therefore cannot change while it is parked, except through the act-start setup (section 3.7), the revive route (section 3.8 b) and the boss-fight refill (the script call BossFightTweakHero at stage 0, SPEC-0030 row 165), which for most boss instances first revives a downed parked hero and then refills both heroes. Confidence: Medium-High.
- The parked hero's target list is kept but not used. The incoming hero's list is emptied at step 9, so its first target scan after a swap starts from nothing.
- Target scan (selected hero only, Medium). Range R and cone C come from boot data by attack variant: variant 0 uses R = 20.0 (0x2DD6A8) and C = 0.5 (0x2DD6B0); variant 1 uses R = 100.0 (0x2DD6AC) and C = 0.5 (0x2DD6B4). The rescan period is 90 ticks (0x2DD6B8). Characters in the actor list that pass the scan's own test are added when not yet listed. For each listed record: a character that is gone, dead or excluded is dropped. Otherwise d = distance and dot = forward . (direction to it). The candidate flag is 0 when dot < C and either the record's counter is below 1 or d > 6.0; it is 0 when d > R; otherwise it is 1 when the clear-path test from the hero passes and 0 when it fails. Candidates are then moved ahead of non-candidates.
- No routine makes the parked hero follow the active hero or act as a companion. The petpet follows the active hero (SPEC-0047); a petpet is not a hero.

Confidence: Medium-High.

### 3.10 What is shared and what is per hero

| Item | Shared or per hero | Confidence |
| --- | --- | --- |
| Hit points (current and maximum) | per hero (stat-ups raise both, SPEC-0036) | High; Medium-High for the stat-up rule |
| Magic pool | per hero (SPEC-0036 starting values per hero) | Medium-High |
| Action state, facing, position, locomotion block | per hero; the incoming hero receives the carried state, the area key, the position and facing, the placement vectors, the 64-byte model matrix and the locomotion heading (modes other than 4) | High |
| Statuses (the 14 status timers) | per hero, never carried: the select step clears all of the incoming hero's timers | High |
| Party inventory (+0x168) | shared: both heroes point at the same record | High |
| Hero index (+0x14C) | per hero, fixed | High |
| Attack variant and charge (+0xF80, +0xF90, +0xFA4) | per hero | High |
| Equipment | per hero. Each hero has its own entry in the weapon, shield, body and cloak lists of the party inventory (list index = hero index) and its own three equipment words (+0x1544, +0x1548, +0x154C; SPEC-0021). | High |
| Quick items | the four quick-item entries and the selection marker (+0x155C) are per hero; item counts are party-wide | High |
| Neopoint wallet, items, Motes | shared | High |
| Luck | shared (both heroes always equal, SPEC-0025) | High |
| Weapon-drawn and shield-raised flags | shared (party flags); SwitchHero clears weapon-drawn, and the Tattered Cloak clears both at step 14 | High |
| Attack-family pools | reset on every swap (step 2) | Medium |
| Progress and game flags | shared | High |
| Swap mode and in-progress flag | shared (one swap-state object) | High |
| Selected pointer | shared (one selector) | High |
| Camera focus and ground reference | shared, re-attached on each switch; the eye is kept (section 3.6b) | High |
| Target list (+0xA70) | per hero; the incoming hero's list is emptied on each switch | High |

The speaker name and colour rule reads the selected hero's index. Index 0 gives the name "Tor" with the first colour slot and index 1 gives "Roberta" with the second (SPEC-0022). The swap mode is not used by the speaker rule.

### 3.11 No companion AI, no co-op

- There is one selected hero. There is no second selected pointer.
- The second pad slot (controller port 2) is used only by debug paths, such as the button sequence of SPEC-0033. The snapshot that the in-play routines read is the OR over the controlling set, which is always pad slot 0, so the second pad slot never takes part in the chord (Medium-High).
- Conclusion: no co-op and no second-controller hero control. Medium.

### 3.12 Guards for the chord (switch-ready gate)

The chord passes only when all of these hold for the selected hero (S) and the other hero (the partner).

- The story flag 368 is clear. Pack 1300 (world 151, the epilogue) sets it in play: its script trigger 3, registered by the pack's init on every load, fades out, plays movie 83 and sets flag 368 when flag 367 is set and 368 is clear. On the next play pass the in-play state tests flag 368 before it reaches the chord and opens the credits instead (SPEC-0026), and the world teardown clears the flag. So in play this guard never refuses a chord (High for the setter, Medium-High for the order in the pass).
- No dialogue box is active.
- The quick-item panel is closed.
- The in-play state reaches the chord. Earlier returns in that state go to the credits, book reader, game-over, item-screen and pause openers described in section 3.8 c (Resolved: Open questions 16 and 17).
- The in-play controls gate of section 3.13 passes. During a swap the game-state code is 5, so this fails for the whole swap, including the 41-tick hold.
- S is not under script control. Group A (states 0x21, 0x30, 0x32 to 0x35 and 0x37) holds the scripted states: grab (0x21) and the one-shot lock (0x33) block the chord.
- S is in state 1 (idle) or 0x14 (20, stance steering). Every other state fails. Because state 0x14 passes, an L1 press that the hero's button chain turns into state 0x14 earlier in the same tick does not block the chord. This excludes walking, attacking, the leap (0x1C), the timed hop (0x38), state 0x17, the action states (0x2B to 0x2E) and all busy states.
- S is not in the busy list: states 0x7 to 0xA, 0xB, 0xC to 0xE, 0xF to 0x13, 0x15, 0x17 to 0x1E, 0x23, 0x25, 0x26, 0x29, 0x2B to 0x31, 0x35, 0x36 and 0x38 to 0x3C. With state 1 or 0x14 only, this test is automatic.
- S is on the ground: the height above the ground in its motion record is below 0.15, or its grounded word is set (set on landings, cleared on take-offs). So a hero in the air cannot switch even in state 1. (High for the test; Medium for the field names.)
- S is not attacking (class slot 0xE4). For attack variant 0 (Tor) the test is true when the action state is 13 with charge sub-state 2 to 12, or the state is 7, 8, 9, 10 or 11 with the hero word +0x1508 equal to 8 or 9. For attack variant 1 (Roberta) it is true when the state is 13 with sub-state 5 or 12. Any other variant gives false. Because S must already be in state 1 or 0x14, this term never refuses on its own (High).
- The stick speed value v of S's locomotion block is exactly 0.0. v is the length of the left-stick pair after the dead zone divided by 87.5 (data word 0x319524), capped at 1.0 (SPEC-0033). So the left stick must be at rest, inside the dead zone.
- S is not stunned: its status 0 (stun) timer is not above 0. A stunned hero stands in state 1 with v = 0, so this is the term that blocks the switch while stunned.
- The screen effect's moving query is false. It is true while fade alpha, bar colour alpha or bar progress differs from its target (SPEC-0052). So the chord is blocked while any fade or bar sweep is running, but not by bars held at rest.
- The swap mode allows the request (section 3.1): modes 3 and 5 refuse; mode 0 refuses while Tor is selected; mode 1 refuses while Roberta is selected; modes 2 and 4 allow.
- The partner is not dead (section 3.2, step 3).
- The swap task slot is free (section 3.2). The in-progress flag is not tested directly, but it fails the controls gate through game-state code 5.

Confidence: High for the state lists, the ground, stick and stun terms, and the mode and partner rules; Medium for the flag meanings.

### 3.13 In-play controls gate

The in-play controls gate passes when the state of the selected hero is not 0x19, the hero is not in group A (section 3.12), flag D (swap-state object +0x20: a box or list picker ran this pass) is clear, and the game-state code (+0x5C, SPEC-0026) is 0. The middle test of the panel-opening gate is this same group A test.

Because the in-progress flag gives code 5, the chord cannot fire from the pass after an acceptance until the pass after the in-progress flag clears.

The same gate also gates opening the quick-item panel (SPEC-0021). While the gate fails, an opening press is not handled. Once the panel is open, its direction, select, use and close actions are not gated. The use step taken with the panel closed (Triangle) is not gated either.

Confidence: High.

## Edge cases and error handling

- **Request while a swap runs.** The request record is the single shared record, so a new request (from the hero-down handler, a Potion of Revival used from the closed quick-item step, or in principle a level script, though no disc script makes one while a swap runs; Open question 11) overwrites it. The task start fails (slot 1 is busy), but the call still returns 1 and sets the in-progress flag again. The running swap then reads the new target at its next use of word 0 (Medium).
- **No chord during a swap.** The controls gate needs game-state code 0, and the in-progress flag gives code 5 until it clears, so the chord cannot make a second request during a swap or its hold (High).
- **Request for the selected hero.** Refused silently (SwitchHero makes no request). High.
- **Mode changes during a swap.** The swap task reads the mode only at step 7. A change during the swap has no other effect (Medium).
- **Fade never equal.** Step 4 waits for the two fade values to be equal, with no timeout. If they never match, the swap never ends (Medium). Step 12 also ends when O leaves state 0x33, so it does not hang on the fade.
- **Mode 4 with an incoming hero in another area.** Only a debug print runs; the hero is not moved (High). No routine writes mode 4.
- **Scene stage 2.** Only Roberta is healed; Tor's hit points are not changed (High).
- **A script sets mode 3 right after SwitchHero.** The swap task then reads mode 3 at step 7, not mode 4. Its placement branch is the copy branch (Medium). Mode 3 also refuses any request made while the swap runs.
- **Revive route (3.8 b) and the dead partner.** A switch to a dead hero is refused by the guard in section 3.2. The revive in route (b) happens first and then the switch request is made.
- **Potion of Revival used from the item screen.** The world is frozen (code 9) but the swap task runs; it waits at step 4 for O's one-shot, which does not advance while frozen, so the swap finishes after the screen closes (Medium).
- **Repeated hero-down requests.** They re-start the portrait swing every tick until the selection changes (Medium).
- **Status cure by swapping.** A swap clears the incoming hero's statuses, so swapping out and back in cures poison, curse and the rest. Stun cannot be escaped this way, because it blocks the chord (High).
- **Chord on the same tick as a D-pad press.** The chord is evaluated first and the D-pad then changes the state. Both happen (Medium).
- **Refused chord.** It is lost. No later tick retries it unless a new press occurs (High).
- **Effect bank 3 not loaded.** The three effect lookups are skipped, so the instances stay empty; they are still positioned, started, stopped and released, which shows nothing. Sound 0xB0 still plays (Medium).
- **ActivateHero with block set and an accepted switch.** The block does not survive the swap: the swap task later sets h's state to 0x33 and then to the carried state 1. No disc call sets block, so this is not reached in the shipped game (Medium).
- **The second swap effect is never started.** Step 10 stops the second effect, which no step started; the stop of an idle instance has no visible result (Medium).
- **Request with the fifth argument set.** No caller passes it, so its fade-in from black with the guards skipped is never used (High).

## Test vectors

Synthetic examples. Hero names are used only as labels.

- Mode 0, Tor selected, SwitchHero(Roberta, 0, 0): refused, returns 0. Mode 0, Roberta selected, SwitchHero(Tor, 0, 0) with Tor alive: accepted, returns 1.
- Mode 1, Roberta selected, SwitchHero(Tor, 0, 0): refused. Mode 1, Tor selected, SwitchHero(Roberta, 0, 0): accepted.
- Mode 2, Tor selected, Roberta dead (state 16), SwitchHero(Roberta, 0, 0): refused, returns 0.
- Mode 2, Tor selected, SwitchHero(Tor, 0, 0): no request, nothing changes except that the weapon-drawn flag is cleared.
- Mode 3 or 5, any selection: refused.
- Mode from story flags: new game (flags 220 and 248 clear): mode 0. After TriggerActStart(2) (sets flag 220) and a reload of the start point: mode 1. Flags 220 and 248 set: mode 2.
- Carry: outgoing Tor alive in state 5, switch accepted: Roberta's state is 5 at step 13. Outgoing Tor in state 0x12: Roberta's state is 1 at step 13.
- Script gating during a swap: a swap accepted on pass 10 whose task ends on pass 70: level-script steps 2 to 5 do not run on passes 11 to 70; the special threads run on every pass; the swap task also advances on any pass with code 9.
- Hold: the in-progress flag clears 41 ticks after the swap reaches step 15 (0.7 x 60.0 in single precision, truncated toward zero, is 41.999996, so trunc gives 41; round-to-nearest arithmetic would give 42).
- HUD, chord accepted, Tor to Roberta, mode 2: Tor available 1, Roberta available 1, the swing starts at 2.617993 rad with blend 1.0 and hold 120; the portrait group is visible.
- HUD, chord, Tor selected, Roberta dead, mode 2: Roberta available 0 (dimmed), swing angles unchanged, hold 240 ticks, group visible for 4 s.
- HUD, chord, Roberta selected, mode 0 (accepted): the swing runs but the group is not drawn (the mode is not 2).
- Camera: outgoing hero at (10, 0, 5) facing (0.6, 0, 0.8), d = 10, eye at (2, 3, -1) on the tick before the selection, switch accepted with either flag setting: the attach point (4, 0, -3) is computed and then overwritten; after the reset the eye is still (2, 3, -1) (before any push by the eye's collision pass) and the normal follow resumes from it. A world-load attach of the same hero instead moves the eye to (4, 0, -3) (or toward the focus, if the ground check fires).
- Sound settle: sound system running, the command channel reports pending on fields 1 to 3 and idle from field 4 on: the step ends after field 13 (10 idle fields in a row). Pending again on field 8: the count restarts and the step ends after field 17. Sound system off: no wait.
- Attacking term: variant 0 in state 13 with sub-state 4: true; variant 1 in state 13 with sub-state 4: false; any hero in state 1: false.
- Target list after a swap: Roberta had 3 listed targets when she was swapped out; Tor is swapped in and later out again; when Roberta comes back her count is 0 and every record id is -1.
- Target scan, variant 0, record counter 0: a character 8 units ahead (dot 1.0) with a clear path is a candidate; at 25 units it is not (R = 20); at 4 units directly behind (dot -1.0) it is not. Same with counter 1: the one 4 units behind is a candidate (d not above 6); at 7 units behind it is not.
- Parked hero: Tor is parked at (10, 0, 5) with 50 hit points; an enemy shot or a damage zone at (10, 0, 5) leaves Tor at 50.
- Act stage 2: Roberta is selected, mode is 1, Roberta's hit points are set to the maximum, Tor's are unchanged, and story flag 220 is set.
- Mode 4 placement: incoming hero's area key equals the world's key: placed at its stored placement. Keys differ: the hero is not moved.
- Chord, Tor selected, hero idle, mode 2, Roberta alive, no dialogue, no panel. Held 0x0004, newly pressed 0x0008 (R1 new while L1 held): a request to Roberta is made.
- Chord with L1 as the new press (held 0x0008, newly pressed 0x0004), Tor selected, hero idle, mode 2, other guards passing: the hero's button chain may move Tor to state 0x14 earlier in the tick; the gate accepts state 0x14, so in either case a request to Roberta is made. The same press made only on the second controller port: no request (the snapshot is pad slot 0).
- Same, held 0x000C, newly pressed 0x000C (both new on one tick): a request is made.
- Same, held 0x0008, newly pressed 0x0008 (R1 alone): no request.
- Same, held 0x000C, newly pressed 0x0000 (both held, no new press): no request.
- Roberta selected, state 0x14 (stance steering), held 0x0004, newly pressed 0x0008: a request to Tor is made.
- Tor selected, mode 0, chord: refused. No sound, no flash; Roberta's available word is 0 and the hold is 240 (not drawn, mode 0).
- Roberta selected, mode 0, chord: accepted, request to Tor.
- Tor selected, held 0x000C, newly pressed 0x0004 (L1 new while R1 held), hero walking (stick speed v above 0.0): no request.
- Roberta selected, Tor dead, chord: refused. No flash, no sound.
- Hero idle in state 1, v = 0.0, height above the ground 0.2, grounded word 0: no switch. Same with grounded word 1: switch.
- Hero stunned (status 0 timer 90): no switch.
- Swap accepted on pass N (code 0): the passes from N+1 to the pass where the in-progress flag clears have code 5; a chord pressed then is ignored; the next pass with code 0 accepts a new chord.
- Hero down: Tor in state 16, death clip progress 0.95, Roberta alive, mode 2: a request to Roberta with the extra flag 1 is made on that tick and on every later tick until step 9. Same in mode 0: refused, game over starts.
- Overlap (synthetic, Medium): the hero-down swap from Tor to Roberta is accepted on pass 100 and passes step 9 on pass 150. On pass 160 the player uses a Potion of Revival from the closed quick-item step, mode 2, Tor dead: Tor is revived at full health, the record's target becomes Tor, the in-progress flag stays 1 and the second task start fails. The running swap's later steps (one-shot, carried state, cloak check) act on Tor, and Roberta stays selected. No disc script makes a request during a swap.
- Incoming Roberta wearing item 126 with the weapon drawn: at step 14 the weapon-drawn and shield-raised flags clear and the sheathe sound plays.
- Tor poisoned (status 2 timer 600) swaps out; Roberta in; swap back: all of Tor's status timers are -1 after step 9.
- SwitchHero(Roberta, 1, 0) from Tor at (10, 0, 5): at step 2 sound 0xB0 plays and effects 0xFB9, 0xFBA and 0xFBB of bank 3 are made at (10, 0, 5); 0xFB9 starts. Tor is alive, so 0xB0 plays again at step 3. At step 10, 0xFBA is stopped and 0xFBB starts at (10, 0, 5), not at Roberta's new place. All three are released at step 16. SwitchHero(Roberta, 0, 0): no effect instance is filled or started.

## Open questions


- Resolved (R-0042, part 1; High for the order, Medium-High for the kit): stage 2 copies the party inventory and wallet into the stash, then clears the party inventory (every list and the wallet go to zero), then adds the kit through the normal add (SPEC-0021): item 150 x 3, item 149 x 3, item 127 x 1 (a dress), and coin item 135 x 1150, which the coin rule adds to the party wallet as 1150 Neopoints. Container A is therefore emptied after the copy and refilled, and the kit belongs to the shared party inventory, not to Roberta alone. Equipment: the dress is added through the normal add and its body or cloak entry is decided by that add's owner rule (SPEC-0021); no other equipment entry is granted.
- Resolved (R-0042, part 2/3; High for the position and facing, Medium-High for the state): the stage 2 and stage 3 placement does two things for the hero it names (Roberta, index 1, in stage 2; Tor, index 0, in stage 3). First it rebinds that hero's animation and skeleton sets and runs the hero's reset (the placement routine does not read any entry-index table; the table it uses holds resource numbers). Then it spawns the hero as at a new game (SPEC-0040, Behavior 1B): the position is the world's spawn vector (entry record 0 of the current world, height probed to the ground; SPEC-0040, Behavior 1C, World spawn vector), the facing is entry 0's facing (the default (1, 0, 0) when the world has no entries), and the same two vectors become the hero's start pose. Nothing in this path writes the hero's action state: the state is whatever the hero already holds. The reset does not touch it either. For Roberta at stage 2 that is her creation value, 0, which the hero update's default route handles (SPEC-0033, Behavior 3); a respawn at the start pose puts a hero in state 1. A rewrite should keep the hero's current state word and set only the position, facing and start pose.

Numbers of removed questions are not reused.

1. Resolved: mode 4 is never set. All 37 SetHeroSwapMode calls in the 10 level modules that use the call (1122, 1217, 1244, 1246, 1300, 1327, 1371, 1394, 1428, 1432) pass an immediate constant: 0, 1, 2, 3 or 5. The other writers (rule M, the load word, the act-start setup) produce 0, 1, 2 or 3 as well, and a saved word can only hold what a writer produced. So the mode-4 placement rule is never reached and the clean side may leave it out (High).
2. Resolved: the player's switch in play is the L1 and R1 chord, tested in the in-play state (section 3.8 c). The test applies the mask 0x000C to the newly-pressed word and the held word of the menu snapshot: it passes when both bits are newly pressed, or when one is newly pressed and the other is held; two held bits without a new press fail. Its guards are in section 3.12: the switch-ready gate takes the state (1 or 0x14 only), the busy and attacking tests, the ground test, the stick speed of exactly 0.0, the stun test and the moving-fade query, then the swap mode and the partner. Test vectors are in section 3.8 c and the Test vectors list. High for the mask, the held/new rule and the gate terms; Medium-High for the order inside the in-play pass.
4. Resolved: the gate of the square-button triggers in route 3.8 (a) is four conditions in order: the lever's once-word is clear, the module's busy word is clear, the prompt context 4 is shown, and Square (button 0) is newly pressed (IsButtonPressed with the newly-pressed form). Only then does the lever sequence start (SPEC-0041, module 1394). High.
7. Resolved: the dead test is "action state equals 16" (High).
11. Resolved: no level script requests a switch while the in-progress flag is 1 (High for the call sites; Medium-High for play). The disc has ten SwitchHero sites and two ActivateHero sites. Pack 1432's three sites are guarded: the director's two swaps are skipped while IsHeroSwapInProgress reports 1, and the entry swap is followed by a wait until it reports 0. Pack 1394's five sites are the four lever routines and the intro routine. A lever needs a new Square press, and IsButtonPressed returns false while the in-progress flag is set (SPEC-0030 row 95); each lever works once per load, and the mode-3 write that follows each switch refuses every other request. Pack 1371's two sites are in the death callback of its round: one call per round, and the fallen hero is not in the death state, so the hero-down route cannot fire. Pack 1244's two ActivateHero calls are in its Init and in one trigger routine, and no other request source is active then. Among the boot-side requesters, the chord cannot fire while the flag is 1 (the controls gate fails), and the item screen and pause openers do nothing (the in-progress word is part of their busy check, SPEC-0035 section 6). The hero-down handler repeats its request every tick with the same target, so that overwrite changes nothing. The one player route to a second request is the Potion of Revival used from the closed quick-item step (Triangle, which the controls gate does not cover) after a hero-down swap has passed its deselect step: the shared record then names the revived hero, the task start fails and the running swap's later steps (one-shot, carried state, cloak check) act on that hero instead of the incoming one (Medium; Edge cases).
12. Resolved: FMV 78 is played by the entry sequence of the final-fight pack (SPEC-0039 and SPEC-0054, pack 1432), which a control thread started by that pack's Init runs, so the movie swap is reached in play (High).
14. Resolved: the gate value 2 is compared inside the revive helper that the host row calls, not in the host row's own body. The helper acts only while the swap mode word equals 2 (free both ways); then it sets the dead partner of the selected hero to idle and restores its health to the maximum, and returns 1 (SPEC-0030 row 178). The revive of a dead partner with a switch is the route in 3.8 (b) (High). Test vector: swap mode 2, Tor selected and alive, Roberta dead: NeedHeroAlive for Roberta leaves her idle at full health and returns 1; with swap mode 0, 1 or 3 the same call changes nothing and returns 0.
15. Resolved: flag 368 is set in play by pack 1300 after movie 83, at the end of the epilogue, and the credits follow on the next play pass (section 3.12; High).
16. Resolved: the earlier returns are the credits gate (story bit 368, credits screen), the book reader gate, the game-over gate, and the Start (item screen) and Select or pad-not-usable (pause page) openers, which return only when the busy check is zero (section 3.8 c; SPEC-0035 section 6). None reads L1, so standing still with L1 held blocks nothing (High).
17. Resolved: only a new press of Start or Select, with the busy check zero, leaves the pass before the chord; a chord on that tick is lost because the menu opens. Held Start or Select does not matter, since the openers read new presses only (section 3.8 c; High).
20. Answered: the movement float is the stick speed value v of the locomotion block; see SPEC-0033. The guard needs exactly 0.0.
21. Resolved: class slot 0xE4 of the hero class table is the attacking test of section 3.12 (by attack variant and action state). Under the state 1 or 0x14 requirement it never refuses on its own (High).
24. Answered: the mutually exclusive flags identify the two quick panels. The first shows the four-direction item ring, its selected entry and owned-item counts (SPEC-0021, Quick-item panel; SPEC-0023 section 11). The second shows the Mote picker: Mote icons for owned Motes, a selection highlight, and its label/count text; its detailed selection and equip rules are in SPEC-0021, Second quick panel and Mote equipping (SPEC-0023 section 24). **[High]**
25. Resolved: the chord reads the menu snapshot of the controlling set, which is always pad slot 0 (section 3.8 c; Medium-High).
26. Resolved: the chord is tested in the front-end in-play handler after the hero's button chain of the same tick (SPEC-0033, Behavior 1A), and the switch-ready gate accepts the state 0x14 that an L1 press can set (High for the order; Medium for the consequence).
28. Resolved for recorded normal routes (High): only three level modules write mode 3 or 5. Pack 1394 (the timed lever course) sets mode 3 in its Init and after each lever's scripted switch (one lever also writes 5 and then 2); its shrine trigger and its exit-open trigger set mode 2. Pack 1371 (the round fight) sets mode 3 after the switch that follows a hero's death and mode 2 when the round is won or lost. Pack 1432 (the final fight) sets mode 3 at entry and after each of the director's swaps. In each case the lock is part of the scene: the chord is refused while the area runs and is freed by the same module's own mode-2 write. In pack 1394, the only recorded load out requires a room flag set by the shrine sequence, which writes mode 2 first; the exit-open sequence also writes mode 2, and the pack's cell records have no outbound transition. Thus no recorded normal route leaves pack 1394 with mode 3 active. Out-of-bounds movement, glitches or an unmodeled external transition have not been tested.
30. Resolved (High for the definitions; Medium for the look on screen): the three ids are visual effect definitions of SPEC-0034's table (effect ids 4025, 4026 and 4027), not sounds. An instance has room for five live spawners, but these definitions use 3, 1 and 4. All three have the near/far threshold 100 units, no cull distance and motion scale 1.0. 0xFB9 (4025, started at step 2) has a model node (the common-object model with index 9 + 283), a one-shot burst of 40 sprites of shader material 22 (start size 1.6, life 3 s plus or minus 1 s) and a point-light emitter (size 15, one burst, life 2 s, following the effect). 0xFBA (4026) is one repeating burst of 3 sprites of material 28 every 0.2 s with no end count (start size 4.07, life 2 s plus or minus 1 s); it is never started (Edge cases). 0xFBB (4027, started at step 10) has two model nodes (common-object models 1 + 283 and 9 + 283, life 1.5 s), a one-shot burst of 40 sprites of material 22 (start size 0.6, life 5 s plus or minus 1 s) and the same light emitter. On screen: a light flash and a sparkle burst at the outgoing hero's place when the swap starts, and a smaller, longer-lasting sparkle burst at the same place at the end. Test vector: at step 10 the 40-sprite burst of 0xFBB is born at the place stored at step 2, so with the outgoing hero at (10, 0, 5) the effect is placed at (10, 0, 5); the spawn ring radius of its nodes is 0.
31. Resolved: the hook is the camera's set-eye step with the pull-in. A switch skips it, so the eye stays where it was and the follow resumes from there (section 3.6b; High for the skip; the pull rule does not run on a switch, so it does not matter here). Bears on SPEC-0027 Open question 56.
32. Resolved: the 15 records are the hero's target list (section 2.1 and section 3.9; Medium for the field meanings, High for the layout).
33. Resolved: nothing can hit the parked hero; every attack, target finder and damage zone uses the selected hero (section 3.9; Medium-High). Bears on SPEC-0055 Open question 2.
34. Resolved: step 5 waits until the sound command channel is idle for 10 fields in a row, with no yield and no timeout; at least about 0.17 s of frozen picture when sound is on (Medium-High).
35. Resolved: there is no cut. The eye is not moved by a switch, so a silent swap shows the new model on the same spot with the same view, whatever the hero's facing (section 3.6b; Medium).
36. Resolved: the values are in SPEC-0023 section 12: pivot element 0x29 is 56 by 64 with pivot point (28, 32) under a parent at canvas (437, 17); Tor's group 0x2A has pivot (28, 29) and Roberta's group 0x2B has (29, 32).
37. Resolved: yes. The picker's story-flag calls go to a routine that does nothing, and none of them names flag 220 or 248 in any case. Rule M then reads the flags as they already are, so after a start point on a fresh session the mode is 0 (Tor only) until a script or TriggerActStart changes it, even for a late start point (Medium-High; SPEC-0035).
