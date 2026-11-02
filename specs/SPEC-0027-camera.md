# SPEC-0027: Camera and projection
- Answers: proactive (none requested) (camera and view), proactive (open close-out), R-0084 (the type-41 scalar's depth-mapping effect, in Zone record blend); R-0188 (the item showcase's camera call is SetHeroSceneCamera(0.9425, 10.0), a live use of mode 8)
- Status: Draft
- Disc build: SCUS-97367 v1.00 (same fingerprint as SPEC-0001)
- Confidence for the later sections: update order High (the meaning of the play-active flag High). Camera profile record and its selection High. Polar convention High. Normal follow High (hero-state names High). Ground clearance Medium-High. Walls and stuck timers Medium-High for the flow, Medium for the push geometry. Recentre pass and R3 High for the trigger and the shrink, Medium for the candidate order. Placement after a load or restart High. Scripted, cutscene and boss cameras High. The type 41 atmosphere zones are specified in SPEC-0028 section 11; the camera only supplies the eye position. Look-at play start Medium-High (the row senses Medium); right-stick aim mode and the camera High; view object reset Medium (the sentinel writes High). Follow speed offset: not a camera routine (it is the class-2 surface test of SPEC-0018), High. Eye pass per-primitive routines High for the owner and for which routine handles which primitive, Medium for the entry steps.
- Confidence: per section. Projection scalars: High (near and far planes High). Mode setter: High (accepted values, the mapping to mode 2, and which values the shipped game passes). Mode routing: High (modes 1 and 11 run the mode 3 handler; only modes 0 and 8 and the scripted plays run on this disc). Empty modes (2, 4, 6): High. Follow-camera order: High. Stick value and spin input: High for the arithmetic, the stick bytes and the excluded state. Chase and manual elevation: High for the arithmetic and for the recentring gate T. Hold and side lean: Medium-High for the arithmetic; High for the meaning (they frame the hero's current target), Medium for the target pick rules. Dormant eye-lag and rig path: Medium for what it computes; not in the live chain. Recentre shrink: High for the sequence and constants. Visibility tests: Medium. Blended transition (mode 8): High for the entry, the fractions, the update arithmetic and the lane names; its pivot step is not a defect. Death camera (mode 8 at game over): High for the trigger and the arguments; Medium for "in front". Blocked latch (contact pass): Medium-High. Smoothed follow (mode 9, dormant): Medium for the update; its entry has no caller. Fallback pose (mode 10, dormant): High for the entity-id lookup, Medium for the entry; Low for the stored target. Dead-band yaw (mode 12, dormant): High for the rule; Medium for its inputs. Mode 3 (debug free-fly, dormant): High for the arithmetic; Medium for the free-fly reading. Mode 7 (debug top-down, dormant): High for the entry and the rule; Medium for the debug reading and for the up-row vector. Pad-axis converters: High for the arithmetic; High for which stick each byte is. Camera reset: High for the writes. Camera point query by mode: High for the branches, the lane of the height and the callers. Pose setter pull-in: High for the rule, the lanes and its callers. Camera initialisation (default offset and forward): High. Fade writer: High. Fade ramp: High. Camera shake: High for the record layout, the presets and the arithmetic; Medium for the angle terms; High for the argument order of the mode call; per-frame use: High for the wave, the envelope, the axes and the aim step. Scripted cameras: High for the steps and the PlayCamera arguments. Camera wall push by primitive kind: see Walls and the stuck timers. Zone record blend (the former pose blend helper; type 41 zones): High. Side-swing event and the eye pass routine split: High. Screen record, quad setup and clip rectangle: not specified here (see Screen record). Items marked **[Low]** are uncertain and must not be implemented as settled.

## Summary

The game draws the world through a single perspective camera with a focal length of 512 pixels on a 640 x 480 screen (4:3); a 16:9 (800 x 450) variant exists but its flag is never set in the shipped game. The near plane is at 1.0 unit and the far planes at 5280, 600 and 10560 units (SPEC-0046). The camera updates once per tick, early in the tick and before the world update, so it follows the hero's pose of the previous tick. Its tuning comes from a camera profile (follow distance, default elevation, spring stiffness, spin gain and decay, minimum eye height) chosen by the kind of floor the hero stands on, or set by a script. The default camera (mode 0) is a leash: the eye keeps its place in the world and a critically damped spring pulls it toward a target point at the profile's distance and elevation from a focus point above the hero, so it swings behind a hero that turns. Right stick X spins the camera around the hero (yaw); right stick Y raises or lowers it (elevation, up to nearly overhead). The target distance shortens as the elevation rises. The eye is kept 2 units above the ground, pushed out of walls and level primitives, and recentred behind the hero when it is stuck, when its view of the hero stays blocked, or when R3 is pressed. The camera has several other modes: a blended transition (mode 8), which is the game-over death camera that swings round to the front of the dying hero; a debug free-fly camera (mode 3, also run by modes 1 and 11); a debug top-down camera (mode 7); a smoothed follow (mode 9); a fallback pose at a named stored target (mode 10); and a dead-band yaw follow with a fixed eye that turns toward the hero (mode 12). Modes 2, 4 and 6 do nothing. Only mode 0, mode 8 and the scripted plays run on this disc; the other modes are dormant (no caller or script reaches them) and are described for completeness and for debugging. After every mode except 0 the camera probes the ground straight down under itself. Scripts can take over the camera: play a recorded, keyframed track that also sets its own focal length (optionally aimed at an object, the hero or an AI character), return to normal, return to an orbit angle, or start a timed camera shake. Boss fights use the same tools; there is no boss camera mode. A dialog request does nothing. A fade ramp fades the character when the camera is very close. On initialisation the camera starts with a default offset of (0, 0, 50) and a forward direction of (0, 0, -1); after a load or restart it is placed behind the hero. The camera eye also selects the per-area atmosphere zones of SPEC-0028 (type 41); the camera itself does not read them. Times are in seconds at 60 ticks per second (SPEC-0007). "Per tick" means per camera update, which runs once per tick.

## Where it lives on the disc

The tuning values below are 32-bit words in the boot program's data, read at runtime by ELF virtual address (SPEC-0001 gives the fingerprint). Other values are fixed constants and are stated here directly. Camera state (eye, yaw, focus, zoom level and so on) is runtime state, not disc data. All values in the table below are read from the boot program's data.
| Constant | Value | ELF data address | Confidence |
| --- | --- | --- | --- |
| Manual elevation cap | 1.5 rad | 0x2DB3B8 | High (the cap can be overshot by one step; see Chase and manual elevation) |
| Dormant lean and eye-lag word (lean blend fraction per unit of stick; eye-lag stop threshold) | 0.05 | 0x2DB3BC | High (value); Medium (use). Dormant path; see Dormant eye-lag and rig path |
| Dormant eye-lag step (default) | 0.025 units per tick | 0x2DB3C0 | High (value). Dormant path |
| Elevation chase step (stick at most 0.99 in size) | 0.01 rad per tick | 0x2DB3CC | High |
| Elevation chase step and manual step (stick above 0.99 in size) | 0.0175 rad per tick | 0x2DB3D0 | High |
| Pull-in factor | 0.5 | 0x2DB3B4 | High (a fixed constant 0.5 is also used). Set-pose rule only |
| Drift hold length in seconds | 2.5 | 0x2DB3D4 | High (value). The drift hold (2.5 seconds, 150 ticks; see Hold and lean terms). The ground probe's lift of 2.5 units above the eye is a fixed constant of the same value (see Ground probe) |
| Transition stall threshold (yaw and elevation) | 1e-6 | 0x2DB414 | High (value); Medium (use) |
| Visibility wrapper flag | 1 (nonzero enables the second side test) | 0x2DB410 | Medium |
| Drift base divisor and stick value divisor | 87.5 | 0x319524 | High (value); Medium (use) |
| Lean side-offset unit divisor | 1.0 | 0x2DB3D8 | High (value); Medium (use) |
| Lean side-offset dead band | 2.0 units | 0x2DB3DC | High (value); Medium (use) |
| Lean side-offset easing step | 0.1 units per tick | 0x2DB3E0 | High (value); Medium (use) |
| Lean side-offset cap, and turn-lean scale (state 0x12 and 0x17) | 10.0 | 0x2DB3F4 | High (value); Medium (use) |
| Side lean scale | 2.5 | 0x2DB3F8 | High (value); Medium (use) |
| Target distance shortening factor (see Target distance) | 0.65 | 0x2DB3F0 | High |
| Ground clearance of the eye | 2.0 units | 0x2DB3C4 | High (value); Medium-High (use) |
| Eye push radius (box push) | 1.25 units | 0x2DB400 | High (value); Medium (geometry) |
| Stuck limit (consecutive failing ticks of the eye pass; trunc(0.05 s x 60) = 3) | 0.05 s | 0x2DB3E8 | High |
| Line-of-sight limit (consecutive blocked ticks; 1.0 s x 60 = 60) | 1.0 s | 0x2DB3EC | High |
| Contact-normal lifetime (32-bit integer) | 40 ticks | 0x2DB40C | High (value and countdown gate) |
| Camera profile tables (five entries each; see Camera profile record) | | 0x2DB308, 0x2DB320, 0x2DB338, 0x2DB350, 0x2DB368, 0x2DB380, 0x2DB398 | High |
| Shake strength scale (amplitude = this x strength) | 0.3 | 0x2DB2F0 | High (value); Medium (use) |
| Shake preset scale S (presets 0 to 5) | 12.0 | 0x2DB2F4 | High (value); Medium (use) |
| Shake preset value p4 (presets 0, 3, 4, 5) | 0.3 | 0x2DB2F8 | High (value); Medium (use) |
| Shake preset ramp R (presets 0, 3, 4, 5) | 0.5 | 0x2DB2FC | High (value); Medium (use) |
| Shake preset angle phi (presets 0, 1, 2, 4) | pi/2 (stored value 1.57079506) | 0x2DB300 | High (value); Medium (use) |
| Shake preset value Z (presets 0, 1, 2, 3) | 0.2 | 0x2DB304 | High (value); Medium (use) |
| Screen centre (GS coordinate centre) | 2048 | 0x322208 | High |
| Widescreen horizontal scale | 0.8 | 0x3223EC | High (value). Unused in the shipped game: the widescreen flag is never set |
| Widescreen vertical scale | 1.066667 | 0x3223F0 | High (value). Unused, as above |
Fixed constants (no address needed): focal length 512.0; screen 640 x 480; the dormant eye-lag fast step 0.1 and early-exit offset 1.5; the focus height factor 0.25 (hero state 21, follow step 5); the damping c = 2 sqrt(k), computed from the profile stiffness; the ground probe lift 2.5 above the eye; the camera-locked probe extra length 2.0 and step floor 0.1; the contact list size 19 and its match threshold 0.999; the restart placement distances 10 and 2.5; the pull-in probe height 5.0; zoom-ramp thresholds 1.7 and 5.0, the ramp divisor 3.3, the ramp maximum 128 and the ramp step 5.0; the pull-in distance 5.0; the orbit minimum radius 0.1; the recentre shrink step 0.25; the shrink floor 1.0; the recentre commit length 0.1; the initialisation offset 50.0 and forward -1.0; the visibility range cut 50 units (squared test against 2500); the distance snap 0.1 in the transition; the blend fractions 0.01 passed to the mode 8 entry (see Blended transition); the mode 8 entry height offset 1.5; the hold length 150 ticks; the mode 12 thresholds 0.1 and 0.2 and its factor 10; the mode 3 pad factor 1/64 (0.015625) and its D-pad value 1092.2667 (= 16384/15); the mode 3 shoulder-button height steps 0.1 and 0.01; the mode 3 turn threshold 0.92 and roll damping 0.15; the mode 7 entry height 100.0 above the ground; the pad-axis dead zones 40, 32 and 30 and centres 127.5 and 128 (see Pad-axis converters); the drift threshold 0.99 and drift value 1.3; the death camera radius 15.0 (see Death camera); the near plane 1.0 and the far planes 5280, 600 and 10560 (see Projection); the fade ramp limit 128; the shake preset value 0.8 (preset 4) and pi/2 (preset 5); and the 2 pi and pi constants.
Boot data words used below (32-bit floats, read from disc; the file offset is the virtual address minus 1 MiB plus 0x80):- Zone range and world-change crossfade distance: 0x2DB4CC = 50.
0. It is used by the type 41 atmosphere zone pass (SPEC-0028 section 11), not by the camera.
- Gated-zone blend distance: 0x2DB4D0 = 25.0 (the same pass).
- The reference direction used by the look-at basis and the restart placement is the run-time block at 0x3932E0, outside the file image. It is the up vector (0, 1, 0, 0), written once at start-up. The neighbouring run-time blocks are 0x3932C0 = (0, 0, 0, 0), 0x3932D0 = (1, 0, 0, 0), 0x3932F0 = (0, 0, 1, 0) and 0x393300 = (0, 0, 0, 1). High.

## Layout

### Projection (the render camera)

| Field | Type | Meaning | Value |
| --- | --- | --- | --- |
| Focal length S | float, pixels | Distance from eye to screen, in pixels | 512.0 (set every camera tick, and on level reset) |
| Screen width W | int | Canvas width | 640 (800 only in the unused 16:9 case) |
| Screen height H | int | Canvas height | 480 (450 only in the unused 16:9 case) |
| Widescreen flag | int | Nonzero selects 16:9 | 0; never set in the shipped game (High) |
| Near N | float | Near plane distance | 1.0 (High) |
| Far-like P | float | Far plane of each of the three clip matrix sets | 5280 (world and characters), 600 and 10560 (High) |
| GS projection far | float | Far plane of the depth-buffer mapping | 10560 (High) |
These are fixed start-up values of the render camera (SPEC-0046, Camera projection, gives the viewport, the GS split and the depth values). The camera does not change them. Depth row (Projection step 2) with N = 1: P = 5280 gives c = 1.000379, d = -2.000379; P = 600 gives c = 1.003339, d = -2.003339; P = 10560 gives c = 1.000189, d = -2.000189.

### Camera record fields used by the modes (data layout, runtime state)

| Field (offset) | Type | Meaning |
| --- | --- | --- |
| 0x30 | point | Base point: the camera position. Returned by the camera point query in modes other than 0, 8, 9 and 10 (see Camera point query by mode). Moved by modes 3 and 7, raised by the mode 7 entry, and pushed by the contact pass of Blocked latch. |
| 0x50 | int | Camera mode. 0 normal follow, 1 and 11 run the mode 3 handler, 2, 4 and 6 empty, 3 debug free-fly, 7 debug top-down, 8 blended transition (the death camera), 9 smoothed follow, 10 fallback pose, 12 dead-band yaw follow. The setter accepts 0 to 4, 7, 11 and 12 and sets mode 2 for any other value (see Mode routing). |
| 0x58 | pointer | Pointer to the attached character's position vector (its motion record position), set by the attach together with 0x5C (pointer to its facing vector) and the word 0x54 = 1. The camera point query reads it in mode 0. |
| 0x60 | pointer | The followed player record. |
| 0x68 | int | Enables the mode 3 height term. Zeroed at construction and never set, so the term never applies. |
| 0x70 | pointer | The focus object. The tick and mode 3 read its ground entry at its +0x178 field (the ground patch the focus character stands on). |
| 0x74 | pointer | Camera profile record (see Camera profile record below): follow distance, default elevation, spring, spin factors and minimum eye height. |
| 0x100 | vector | Unit direction of the eye's motion over the last follow tick, with its length in lane 3 (Medium for the readers). |
| 0x120 | point | Focus point F of the follow camera. |
| 0x130 to 0x13C | vector | Target polar P (distance, yaw, elevation in its first three lanes; see Polar convention). |
| 0x140 to 0x14C | vector | Current polar C = polar(eye - focus) with the spun yaw. The target distance rule reads its elevation lane (0x148), as stored by the previous tick. |
| 0x150 | float | Spin value I: a yaw rate in radians per tick, added to the eye's yaw each tick (see Spin). Zeroed by camera reset. |
| 0x154 | float | Elevation target t_e (see Chase and manual elevation). Set to the profile default elevation when a recentre pass starts; set to the eye's elevation by camera reset. |
| 0xC4 | float | Manual elevation m (see Chase and manual elevation). |
| 0xC8 | int | Manual flag (sticky; see Chase and manual elevation). |
| 0xE0 | point | Eye position. |
| 0xF0 | point | Previous tick's final eye: copied from the eye after the wall pass each tick; restored when the eye is stuck (see Walls and the stuck timers). |
| 0x198 | int | Reseat flag. Set to 1 by camera reset and by each orbit shrink. |
| 0x19C | int | Eye-lag flag (dormant path only). |
| 0x1C8 / 0x1CC | float / int | Side-swing increment and its tick counter. Written by the side-swing event (see Side-swing event); zeroed by camera reset and by the attach. While the counter is above 0 the increment is added to the spin value I each tick (see Spin). |
| 0x1D0 | int | R3 recentre latch (see Recentre pass and R3). |
| 0x1D4 | int | Ground entry under the camera, written by the ground probe (see Ground probe). In mode 3, while the focus has a ground entry, the tick copies that entry here instead. |
| 0x1D8 / 0x1DC | int / float | World of that ground entry / ground height under the camera. |
| 0x1E0 | float | Side offset (current). |
| 0x1E4 | float | Side offset target. |
| 0x1E8 | float | Side lean (player-state test passes; see Hold and lean terms). |
| 0x1EC | int | Hold counter (ticks). |
| 0x1F0 | float | Small bias (cleared by the hold branch). |
| 0x1F4 | float | Heading bias (cleared by the hold branch; read by the chase target). |
| 0x1F8 | float | Turn lean for player state 0x12. |
| 0x1FC | float | Turn lean for player state 0x17. |
| 0x200 | int | Target object handle (set by the mode 8 entry; read by mode 9). |
| 0x204 | int | Second target object handle (mode 9; used when the selector at 0x230 is 2). |
| 0x208 | float | Distance sqrt(R² + h²) from the mode 8 entry's radius R and height offset h. R itself is not stored. |
| 0x210 / 0x214 | float | Yaw offset o and height offset h given to the mode 8 entry. |
| 0x218 | float | Elevation (arcsine of h over R, the horizontal reach, not over the distance; see Blended transition). |
| 0x21C / 0x220 / 0x224 | float | Blend fractions for distance, yaw and elevation. |
| 0x228 | float | Pivot step per update (mode 8 entry argument; also read by mode 9). |
| 0x22C | int | Target-point selector of the mode 9 entry (values 1 or 2; the entry has no caller). |
| 0x230 | int | Selector for the second target object in mode 9 (value 2 selects it). |
| 0x240 / 0x250 | point | Pivot and target point (modes 8, 9 and 10). |
| 0x260 | int | Blocked latch for modes 8, 9 and 10 (see Blocked latch). |
| 0x264 | float | Spin input D (right-stick X and leans; see Stick value and spin input). |
| 0x268 | float | Stick value s (right-stick Y, the elevation input; see Stick value and spin input). |
| 0x26C | int | Hold word: 1 while the drift base is nonzero (right stick X outside its dead zone and not frozen). While nonzero, the contact normals count down (see Walls and the stuck timers). |
| 0x270 | int | R3 press-edge bit of this tick. |
| 0x500 | int | Contact-normal count of the eye pass (see Eye pass failure flag). |
| 0x504 | int | Previous tick's failure flag. |
| 0x508 | int | Failure flag of the eye pass: cleared at the start of each pass, set by a per-primitive routine that cannot resolve its contact. |
| 0x50C | int | Restore latch: set when the eye is put back at the previous eye. |
| 0x510 | int | Stuck counter (consecutive failing ticks). |

### Camera profile record (Confidence: High)

The camera reads its tuning from a profile record (one static run-time record at 0x370410; camera field 0x74 points at it). Its fields are loaded from five-entry tables of 32-bit floats in the boot data (entry i at table address + 4 i):
| Field | Meaning | Table (ELF data) | Profile 0 | 1 | 2 | 3 | 4 (script) |
| --- | --- | --- | --- | --- | --- | --- | --- |
| +0x00 | override flag (int) | none | | | | | |
| +0x04 | follow distance | 0x2DB308 | 10.0 | 9.0 | 10.0 | 15.0 | 10.0 |
| +0x08 | default elevation (rad) | 0x2DB320 | 0.2 | 0.25 | 0.2 | 0.6 | 0.2 |
| +0x0C | spring stiffness k | 0x2DB338 | 0.085 in all five | | | | |
| +0x10 | spring damping c | computed | 2 sqrt(k) = 0.583095 | | | | |
| +0x14 | no reader known | 0x2DB350 | 0.01 in all five | | | | |
| +0x18 | spin gain a | 0x2DB368 | 0.005 in all five | | | | |
| +0x1C | spin decay b | 0x2DB380 | 0.125 in all five | | | | |
| +0x20 | minimum eye height above the ground | 0x2DB398 | 1.0 in all five | | | | |
| +0x24 | current profile index (int; starts at -1, so the first selection always loads) | none | | | | | |**Selection (each normal-follow tick).
** The ground patch the focus character stands on (the grid entry of SPEC-0018 found by its ground probe; the u32 at entry +0x38) selects the profile: 3 gives profile 2; 4 gives profile 1; 5 or 6 give profile 3; any other value gives profile 0. Pack 735: the ring patch (kind 5) selects profile 3 (distance 15.0, elevation 0.6); pads, platforms, pillars and the pit island (kind 0) select profile 0. When the override flag is set the profile is 4. If the index equals the current one nothing is reloaded; otherwise all fields are loaded from the tables. When the character has no ground patch that tick, the profile is left as it was.
**Profile names (Medium).
** The boot data holds one debug name per profile, in profile order: the main camera (0), the interior camera (1), the small-room camera (2), a special camera (3) and the script camera of the level modules (4, the override). No reader of the names is known. The names match the disc: ground kind 4 (profile 1, the interior camera) appears in 33 of the 50 interior packs, against 12 of the 87 exterior packs and 3 of the 36 transition packs; kinds 5 and 6 (profile 3) appear only in exterior and transition packs.
**Script calls.
** OverRideCameraAttributes(distance, elevation, minimum height) writes the three profile-4 table entries (0x2DB318, 0x2DB330, 0x2DB3A8) and sets the override flag; RestoreCameraAttributes() clears the flag. The written values stay in the table. No shipped level script calls either.
**Disc survey** (top-level grid entries of all 192 world packs): +0x38 = 3 in 655 entries (41 packs), 4 in 136 (48 packs), 5 in 164 (21 packs), 6 in 86 (15 packs), 0 in 9,986, 0xFFFFFFFF in 713; other values seen are 1, 2, 7 and 17. Examples: kind 4 in pack 732, kind 5 in pack 735, kind 6 in pack 758.

### Followed player record fields used by the camera (data layout)

| Field (offset) | Type | Meaning |
| --- | --- | --- |
| 0x11F | unsigned byte | Left-stick Y (centre about 127.5). Used by mode 3 (factor -0.05) and mode 7. |
| 0x123 | unsigned byte | Left-stick X (centre about 127.5; mode 3 uses centre 128). Used by mode 3 (factor 0.05). |
| 0x127 | unsigned byte | Right-stick Y (centre about 127.5). The stick value s (the elevation input). Used by mode 3 (factor -34.13) and mode 7. |
| 0x12B | unsigned byte | Right-stick X (centre about 127.5). The spin input base (yaw). Used by mode 3 (factor -34.13) and mode 7. |
| 0x150 | 16-bit flags | Held-button word of the pad snapshot. Bit 0x400 (R3) clear releases the recentre latch; bit 0x0001 (L2) is read by the camera-locked state test; mode 3 reads the D-pad bits (Up 0x1000, Right 0x2000, Down 0x4000, Left 0x8000) and, only when its height term is enabled, the shoulder bits (L2 0x0001, R2 0x0002, L1 0x0004, R1 0x0008) (see Recentre pass and R3, Follow camera and Mode 3). |
| 0x154 | 16-bit flags | Press-edge word of the pad snapshot. Bit 0x400 (R3 pressed this tick) starts a recentre pass. |
| 0x162 | 16-bit state | Player state. The value 7 enables the mode 3 pad terms. The value 21 scales the focus height by 0.
25. The values 0x12 and 0x17 select the turn leans. The camera-locked states are listed in Follow camera step 9. |
The four stick bytes are unsigned values from 0 to 255. They are the followed player's input block: left-stick Y at 0x11F, left-stick X at 0x123, right-stick Y at 0x127 and right-stick X at 0x12B. The left-stick bytes are never inverted. Inversion complements the right-stick bytes (SPEC-0033): the right-stick X byte when the left-right invert option is on, and the right-stick Y byte when the up-down invert option is on. The camera reads the right-stick Y byte as its stick value (elevation) and the right-stick X byte as its spin input base (yaw).

### Camera shake record (runtime state; High for the fields the per-frame step reads)

One record holds the current shake. It is created on first use (in run-time memory at 0x370320) and reused. The envelope is a list of six keys: key k has a time (0x5C + 4k), an amplitude (0x84 + 4k) and a wave rate (0xA0 + 4k). A start fills keys 0 to 3; keys 4 and 5 stay zero, because every start clears the whole key list first. Starting a shake fills the record as follows (offsets are within the record):
| Field (offset) | Type | Set from | Meaning |
| --- | --- | --- | --- |
| 0x58 | float | duration T | End time in seconds (the shake ends when the elapsed time reaches it). |
| 0x5C, 0x60, 0x64, 0x68 | float | 0, p4, T minus R, T | Times of keys 0 to 3. So p4 is the time at which the shake reaches full amplitude (the attack time) and R is the fade-out time at the end. |
| 0x84, 0x88, 0x8C, 0x90 | float | 0, A, A, 0 | Amplitudes of keys 0 to 3. |
| 0xA0, 0xA4, 0xA8, 0xAC | float | scale S | Wave rates of keys 0 to 3 (all equal, so the rate never changes during a shake). |
| 0x7C | float | amplitude A | Start amplitude; used for the aim distance (see Per-frame use of the shake). |
| 0x78, 0x9C | float | 0, S | Current amplitude and current wave rate (written by the per-frame step). |
| 0xB8 | float | value Z | Aim factor: above 0 the shaken camera keeps looking at a fixed point ahead; at or below 0 the shake only moves the camera. |
| 0xBC | float | angle phi | Cosine of the absolute angle, cos(abs phi): the sideways share of the movement. |
| 0xC0 | float | angle phi | Cosine of abs(phi - pi/2). For phi between 0 and pi this equals sin(phi): the vertical share. |
| 0x44 | int | 0 | Finished flag (1 when the record is created; set to 1 when the shake ends). |
| 0x40, 0x4C, 0x54, 0x80 | int or float | 0 | Cleared at each start; 0x4C is the wave phase. |
| 0x48, 0x74 | float, int | 0 | Elapsed time and current key index. Every start clears both (the start first runs the same clearing step that creates the record), so each shake runs from its own beginning. |

## Behavior

### Projection

1. Scales (centred perspective, principal point at the screen centre):
- x scale a = 2S / W. At 640 wide: 1.
6.   - y scale b = 2S / H. At 480 high: 2.13333.
2. Depth row (column 2 and 3 of the z row), with P the far-like value and N the near value:
- c = (P + N) / (P - N)
- d = -2 P N / (P - N)   The matrix maps view-space z into clip depth through c and d. Whether the w component is +z or -z (that is, whether the camera looks along +Z or -Z) is a matrix-sign convention inside the draw program that does not change the image: any consistent choice, with the depth values mapped as above, draws the same picture (rewrite choice: keep the eye-space forward axis as the polar convention of this spec defines it) **[Medium]**.
3. Half-tangents of the field of view: tan(half fov_x) = W / (2S), tan(half fov_y) = H / (2S).   - 4:3 (640 x 480): horizontal 0.625, vertical 0.46875. Field of view 64.0 x 50.2 degrees.   - 16:9 (800 x 450, never used on this disc): horizontal 0.78125, vertical 0.43945. Field of view 76.0 x 47.45 degrees.   - While a scripted track plays, S is the track's focal length (field +0x2C of the type-28 track record, SPEC-0052), written to the render camera on every posed update; the normal camera writes 512. The common track value 791.45 gives a horizontal field of view of 2 atan(320 / 791.45) = 44.0 degrees and a vertical one of 2 atan(240 / 791.45) = 33.7 degrees at 640 x 480, against 64.0 x 50.2 degrees in normal play. High for the write; Medium-High for the field-of-view reading.
4. Widescreen remap of normalised screen coordinates (0 to 1). Used when the widescreen flag is nonzero:
- horizontal: u' = (u - 0.5) / 0.8 + 0.5   - vertical: v' = (v - 0.5) / 1.066667 + 0.5   This matches a canvas of 800 x 450 drawn into the 640 x 480 coordinate frame. The widescreen flag is never set in the shipped game (its setter has no caller), so the 640 x 480 case always applies; the 16:9 remap is kept for completeness only (High).
5. A second matrix family (4094 wide and high, centred at 2048) is also built. It is the guard-band clip set of SPEC-0046, one per far plane (High).

### GS screen offset and scissor

1. The GS screen offset for a viewport at (x, y) of size (w, h) is, per axis, (2048 - (origin + size / 2)) x 16. So for the full 640 x 480 viewport at (0, 0): horizontal (2048 - 320) x 16 = 27648, vertical (2048 - 240) x 16 = 28928. The GS subtracts the offset from each vertex's 12.4 fixed-point coordinates to get the window position (a hardware rule), so a vertex at window position p is submitted as p x 16 + offset and the viewport centre sits at the middle of the 4096-wide coordinate space **[High]**.
2. The scissor rectangle is the viewport: x from x to x + w - 1, y from y to y + h - 1 (inclusive). Medium.

### Mode routing

Each tick the camera update runs one handler, chosen by the mode number:
| Mode | Handler | Confidence |
|---|---|---|
| 0 | Normal follow (see Follow camera) | High |
| 1 and 11 | Debug free-fly (the same body as mode 3; dormant) | High |
| 2 | No action (empty) | High |
| 3 | Debug free-fly (dormant) | High |
| 4 | No action (empty) | High |
| 6 | No action (empty) | High |
| 7 | Debug top-down (dormant) | High |
| 8 | Blended transition (the death camera) | High |
| 9 | Smoothed follow (dormant) | Medium |
| 10 | Fallback pose at a named stored target (dormant) | Medium |
| 12 | Dead-band yaw follow (dormant) | High (rule) |
| 5 and any other value | No action | High |
The general dispatch handles modes 0, 2, 3, 4 and 5 (2 and 4 have empty bodies; 5 and other values do nothing); modes 1 and 11 call the mode 3 body directly (High). After the mode handler, in every mode other than 0, the ground probe runs (see Ground probe); mode 0 has its own ground clearance instead. During a scripted play, the play and the ground probe run and nothing else.
The public mode setter takes a mode number. It accepts 0 to 4, 7, 11 and 12 and sets that mode; any other value sets mode 2 instead, which does nothing. Setting mode 7 also raises the camera to at least 100 units above the ground (see Mode 7). Modes 8, 9 and 10 are set only by their own entries.
**Mode entry points in the shipped game (High).
** In the whole program the setter is called only with 0, 12 and 3, and the call with 3 comes from a routine that nothing references.
| Mode | Who sets it | Runs in play |
| --- | --- | --- |
| 0 | camera initialisation (every world load from the front end), EndCameraTrackHeroMode (only from mode 12), the 2 s quit hold, the hero handlers of hero state 21, and return-to-normal from modes 8 to 10 | yes |
| 1, 11 | no caller passes these values | no |
| 2, 4, 5, 6 | no caller passes these values (2 is also the setter's answer to an invalid value, which no caller passes) | no |
| 3 | only a debug routine that nothing references: it sets mode 3, poses the camera from a stored debug pose and copies a stored focal length | no (debug free-fly camera) |
| 7 | no caller passes 7 | no (debug top-down camera) |
| 8 | the hero death handler on the game-over branch (see Death camera), and the scene camera of SetHeroSceneCamera, which the full item showcase of SPEC-0021 calls as (yaw offset 0.9425, radius 10.0) every time it runs (the one script site, in a routine of pack 1250 that nothing references, is not used in play) | yes, at game over and during every full item showcase |
| 9 | its entry has no caller | no |
| 10 | SetAI_DistCamera and SetHeroDistCamera (no script sites) | no |
| 12 | SetCameraToTrackHeroMode (no script sites) | no |
So a clean build must implement mode 0, mode 8 and the scripted plays. Modes 1, 3, 7, 9, 10, 11 and 12 are dormant: they are never reached on this disc. Their bodies are kept below, marked dormant, and may be implemented as described (for example as debug cameras).
In the per-tick update, while the mode is 3 and the focus character has a ground entry (focus +0x178), that entry becomes the camera's ground entry (0x1D4), and the fade ramp and the ground probe are skipped. The fade ramp runs after every mode except 3.

### Camera point query by mode (Confidence: High for the branches; Medium for the lane)

The camera has a point query that returns one point from the camera record. The camera mode at record 0x50 chooses the point:1. Mode 0 (normal follow) returns the attached character's position (the vector that record 0x58 points to) plus (0, character height, 0), the height from the focus object record at +0x16C, +0xC. This is the follow focus without the small bias (0x1F0) and without the state 21 scaling.
2. Modes 8, 9 and 10 return the pivot (record 0x240).
3. All other modes return the base point at record 0x30.
The query has two callers, the mode 8 entry and the mode 9 entry; each takes the answer as its pivot. Only the mode 8 entry is live (the death camera enters from mode 0, so its pivot is the hero's position raised by its height). (High.)Test vector: in mode 0, hero at (10, 0, 5) with height 2.0: the query returns (10, 2, 5).

### Update order (Confidence: High for the order and for the meaning of the play-active flag)

The camera updates once per game tick, early in the tick: after the controller snapshot and the game-status update, and before the world update. The hero's own update, its motion step and its animation run later in the same tick, so the camera always works from the hero's pose of the previous tick (one tick of lag). After the camera update, the shake (if any) is applied to a copy of the camera matrix; only the copy goes to the renderer. Sound (the listener) and the other systems use the unshaken matrix. The per-area atmosphere selection (SPEC-0028 type 41) runs after the world update, from this tick's camera eye. The normal follow runs only while the game's play-active flag is set and the camera has a focus character. The play-active flag is the game-global word 0x35C980 + 0x04: it is cleared on every pass just before the front end runs and set again by the in-play screen, the dialogue-box host and the page stepper, so the camera reads the value left by the previous pass's front end (SPEC-0026, Pause and freezing screens). The camera mode dispatch runs on every pass, also while the world is frozen (game-state code 9), but the follow update is skipped while the flag is 0, so the camera holds still under every freezing screen (the pause page, the item screens, the map and the other pages listed in SPEC-0026). The shake and track applier and the view set-up still run. The camera volume blend runs once per pass after the world branch and the global task system (SPEC-0026, main loop step 10).

### Polar convention (Confidence: High)

The camera stores offsets from the focus as (r, yaw, elevation): r = |v|, yaw = atan2(v.z, v.x), elevation = asin(v.y / r) (computed as pi/2 - acos(v.y / r)). The inverse is v = (r cos(yaw) cos(e), r sin(e), r sin(yaw) cos(e)). Y is up. The camera matrix rows are right, up, forward and position; the camera looks along its third row, and right = up x forward. "sphere(r, yaw, e)" below means this inverse.
Test vector: eye (0, 3.9357, -9.5493) about focus (0, 2, 0) gives r 9.7435, yaw -pi/2 and elevation 0.2; the rows are right (1, 0, 0), up (0, 0.9801, 0.1987) and forward (0, -0.1987, 0.9801).

### Follow camera (mode 0) (Confidence: High for the steps; Medium for the hero-state names)

State: eye E, focus F, target polar P = (P.r, P.yaw, P.e), spring velocity v, spin value I, elevation target t_e, manual elevation m. Each tick, in order:1. **Scripted check.
** If a scripted play is active (see Scripted cameras), run the play and the ground probe (see Ground probe), and skip the rest.
2. **Profile selection** (see Camera profile record).
3. **Stick value and spin input.
** Compute the stick value s (right stick Y) and the spin input D (right stick X and the leans); see Stick value and spin input. Both are recomputed every tick.
4. **Target distance.
** P.r from the profile distance and the current eye elevation (see Target distance).
5. **Focus.
** F = the focus character's base position + (0, character height, 0); the height is multiplied by 0.25 in hero state 21; plus the small bias (0x1F0).
6. **Spin.
** I <- I + D a - I b (see Spin). Take the eye's offset from the new focus as a polar C = polar(E - F) and add I to its yaw (C.yaw <- wrap(C.yaw + I), with the angle-wrap rule of Edge cases).
7. **Elevation target.
** If the recentring gate T is 0 and the manual flag is set, run the manual step, else the chase step (see Chase and manual elevation).
8. **Eye on the sphere.
** E = F + sphere(C): distance and elevation unchanged, yaw spun.
9. **Camera-locked hero states.
** In hero state 32, states 43 to 46, states 17 and 20 with the sub-field 32, and state 13 with its mode word 1 while L2 (pad bit 0x0001) is held: P.yaw = the yaw of the camera's back direction (minus the forward row). A probe at F + sphere(P.r + 2.0, P.yaw, P.e) is tested for visibility; when it is blocked and the current distance is below P.r, P.r = current distance + max(hero step length, 0.1). If D = 0 the eye is placed at F + sphere(P) directly (the camera moves rigidly with the hero). In all other states P.yaw = atan2(E.z - F.z, E.x - F.x), the eye's own yaw, so the spring below pulls only the distance and the elevation.
10. **Spring.
** T = F + sphere(P). v <- v - k (E - T)
- c v, then E <- E + v, with k and c from the profile. Exception: while the hero is not moving (its body speed value at most 0.01, and not in state 28), the eye is not stuck and v points into a remembered contact normal n (v . n <= 0), the eye does not move this tick.
11. **Ground and walls.
** Ground clearance, then the wall push and its timers (see Walls and the stuck timers), then the R3 recentre (see Recentre pass and R3). Then the pose: the previous eye (0xF0) = E, and the camera looks from E at F.
12. **Fade ramp.
** See Fade ramp.
The dormant eye-lag stage is not in this chain (see Dormant eye-lag and rig path).
Consequences: only right stick X changes the yaw directly. The hero's motion swings the yaw because the eye trails like a leash: it stays where it was in the world and is pulled toward the target, so it swings behind a hero that turns. Right stick Y moves the camera up and down (pitch), not sideways.
Follow lag: per tick the spring is critically damped. For a unit offset with v = 0 the error is 0.915, 0.80179 and 0.68644 after ticks 1 to 3; 0.48633 after 5 ticks, 0.19347 after 10, 0.0297 after 20, 0.00455 after 30 and 0.00027 after 45.
Test vectors (the game's rules with character height 2.0, no ground, hero at the origin unless stated):- Settle after a restart placement: start eye (0, 2.5, -10), focus (0, 2, 0) (r 10.0125, e 0.04996). Distance and elevation after ticks 1, 5, 10, 20, 30 and 60: (10.0100, 0.0508), (9.9955, 0.0652), (9.9740, 0.1007), (9.8865, 0.1774), (9.7914, 0.1965), (9.7439, 0.2000). Final eye (0, 3.9357, -9.5493).
- Running straight away from the camera at 0.1 units per tick: the distance stretches to 10.3504 at elevation 0.1887 (steady by about tick 30). After a 90 degree turn to run along +x, the yaw moves from -90 degrees to -101.7 after 20 ticks, -123.2 after 60 and -155.2 after 150 (distance 9.85 to 10.29).
- Camera-locked state, hero moving along +x at 0.1 per tick: the eye moves rigidly with the hero (yaw and distance constant at 9.7435). Back in a normal state, the yaw swings by -0.104 rad in 10 ticks and -0.997 rad in 120 ticks.

### Stick value and spin input (Confidence: High for the arithmetic; High for which stick)

The stick value s moves the elevation (see Chase and manual elevation). The drift value D is the spin input: it turns the yaw through the spin integrator (see Spin).
Converter A (see Pad-axis converters) is applied to a pad byte.
- **Stick value** = -(converter A of byte 0x127) / 87.5, where byte 0x127 is the right-stick Y byte. Range -1 to 1. Stored at camera 0x268. It is the elevation input used by the chase target and the manual step.
- **Drift base** = B x |B| where B = -(converter A of byte 0x12B) / 87.5, and byte 0x12B is the right-stick X byte. The divisor 87.5 is the full output range of converter A (ELF word at 0x319524).
- **Excluded state.
** Both the drift base and the stick value are zero while the streaming-load freeze holds: the game-globals boundary-contact word (0x35C980 + 0x24) and the load-busy word (0x35C980 + 0x28) are both nonzero (SPEC-0026, game-state code 9). High.
- **Sum with the leans.
** S = base + side lean + turn lean (state 0x12) + turn lean (state 0x17).
- **Drift.
** D = S x |S|. Then: if D > 0.99, D = +1.3; if D < -0.99, D = -1.3; otherwise D is unchanged. This is a switch, not a clamp: a value just above 0.99 becomes exactly 1.3. The one constant 1.3 is loaded once and used for both signs (the negative side negates it), so the rule is symmetric; the behaviour is fixed, and the design intent does not change what to implement (Open question 9).
- **Reachable drift values (High).
** With no leans, D = B to the fourth power with B's sign. Only pad bytes 0 and 255 give |D| = 1.3.
Byte 254 gives base -0.977273 and D = -0.955063. Byte 250 gives D = -0.790285, byte 240 gives -0.471324 and byte 220 gives -0.1296. A full revolution with the stick held from rest takes 128 ticks (2.13 s) at byte 255, 165 ticks (2.75 s) if D were clamped to 1.0, and 172 ticks (2.87 s) at byte 254.
Byte 0x127 is not a spin input. It feeds the stick value only.

### Chase and manual elevation (Confidence: High for the arithmetic and for the recentring gate)

These steps set the elevation (pitch) of the camera, not its yaw. The chase value (0x154) is the elevation target t_e, the manual accumulator (0xC4) is the manual elevation m, and the chase "heading" is the profile's default elevation (+0x08).
**Recentring gate T.
** T is a test on the followed hero. Its stored reference is the hero's position at the start of its last motion step (each motion step copies the position there before it moves). T = 1 only when the hero's position differs from that reference, that is when the hero moved during its last motion step, and the hero state is not idle (state 1). Because the camera runs before the hero, that is the previous tick's step. Otherwise T = 0 (High). One exception also gives T = 0 even when the body has moved (High): Roberta in her attack state. That is state 13 (attack, SPEC-0036) with the hero variant word +0xF80 equal to 1 (Roberta; it is a hero selector, not a mode) and her attack sub-state (+0xF90) equal to 1 to 4 (hold and the three steps), 9 (full release), 10 (rapid shot) or 14 (full release of the Mote ids). Tor in state 13, and Roberta in any other sub-state, follow the general rule.
**Mode choice each tick.
** If T is 0 and the manual flag (0xC8) is set, the manual step runs and the chase step does not. Otherwise the chase step runs. So when T is 0 and the flag is clear, the camera returns to the chase target.
**Chase.
** Target = profile default elevation + stick value + heading bias (0x1F4). The target is negated only when the negation test holds (High): the hero is in state 21 (climbing a climbable panel, SPEC-0033) and the shared climb and ladder sub-mode word (+0x9BC) is 4, the stick-steered move on the panel that the arrival step and the ceiling-panel grab lead to. On the ground and in every other climb sub-mode there is no negation. Step per tick: 0.0175 if |stick value| > 0.99, else 0.01 (0.6 rad per second, or 1.05 rad per second for the larger step). t_e moves toward the target by at most the step, and snaps to the target when the remaining difference is less than the step. When P.r > 0.1, P.e = t_e and m = t_e. During the chase step, when T is 0 the manual flag is set to 1 if the stick value is nonzero and to 0 otherwise; when T is 1 it is cleared. So a right-stick push while the hero stands (T = 0) sets the flag, and the return to the chase target stops until the hero moves (T = 1).
**Manual.
** With s = stick value, each tick m increases by k x s x |s|, where k = 0.0175 if |s| > 0.99, else 0.
01. m is an absolute elevation. Its limit is +1.5 rad (about 86 degrees, nearly overhead) for positive s and -1.5 rad for negative s. While s > 0 and m is below +1.5, the step is added; when s > 0 and m is at or above +1.5, m is set to +1.5, and the same holds for s < 0 at -1.
5. One step can carry m past the limit once, because the test uses the previous value. P.e = t_e = m. With s = 0, m does not change; it does not decay.
**Manual flag is sticky.
** Once set, the flag stays on while T stays 0 and the manual step runs. Manual mode then continues with a zero stick until T changes or a recentre pass clears the flag.
**Range.
** Outside manual mode the target is at most the profile elevation + 1 + bias: 1.2 rad with profile 0. With profile 3 (0.6 rad) a full push gives 1.6 rad, and a manual tick then sets m to the 1.5 cap (a 0.1 rad step; Medium).
Test vectors:- Chase, target 1.0, start t_e 0, T = 1: t_e is 0.01 after 1 tick, 0.99 after 99 ticks, and 1.0 at tick 100 (snap; float rounding may move this to tick 101).
- Manual, stick 0.5 (k = 0.01): step = 0.01 x 0.5 x 0.5 = 0.0025 per tick. After 4 ticks m has risen by 0.01.
- Manual, full push (stick 1.0, k = 0.0175): 0.0175 rad per tick (1.05 rad per second).
- Manual overshoot, m = 1.499, stick 1.0 (k = 0.0175): after 1 tick m is 1.5165; after 2 ticks it is 1.5.
- Right stick Y full up (s = +1) while standing (T = 0), profile 0: the manual flag is set on tick 1 (elevation target 0.2175), then the target rises 0.0175 per tick, reaches the 1.5 cap at about tick 76 and stays there after release. Eye elevation and distance at ticks 30, 60 and 120: (0.6185, 8.5087), (1.1375, 5.3794), (1.5000, 3.5330). When the hero then moves (T = 1) the target returns by 0.01 per tick: 1.4 after 20 ticks, 0.9 after 60, 0.3 after 120.

### Lock-on (none) (Confidence: High)

There is no lock-on or target-lock camera. The camera has no target-lock mode, no input selects a lock target, and the camera writes no lock field. Every pad button of the hero action path already has a role (SPEC-0033: L1 shield guard, L2 strafe, R1 run, R2 or Circle attack, Cross jump, Square interaction, Triangle quick items, R3 recentre). The only button-held camera behaviour is the L2 strafe: in the strafe states (32 and 43 to 46) the follow uses the camera-locked rule of follow step 9, so the yaw stays on the camera's current back direction while the hero walks and turns toward the camera's forward (SPEC-0005); no target is involved. Test vector: camera yaw -pi/2 behind a hero who strafes sideways with L2 held: with the right stick centred, the target yaw stays -pi/2 on every tick. The nearest thing is automatic: the side lean and side target (see Hold and lean terms) turn and shift the normal follow toward the hero's current target (the nearest noticed enemy within 20 units) with no input and no mode change.

### Right-stick aim mode and the camera (Confidence: High)

The hero's right-stick aim mode (SPEC-0036, Right-stick aim mode) does not change the camera record. Its "camera mode" values 0 to 4 are not camera modes in the sense of record 0x50: the hero's update computes them only while the camera mode (0x50) is 0, which is the "lock word is 0" condition of SPEC-0036, and passes them, with the two stick values and the aim counter, to a separate aim view object (a static object in run-time memory at 0x382D90). That object blends its own values from the mode it is given and draws its model in the render pass; no camera routine reads the hero's aim word (+0x1490). So during aim mode:- the camera stays in mode 0 and the normal follow keeps running every tick;- right-stick Y, which starts aim mode, still drives the follow elevation through the manual step (see Chase and manual elevation), and right-stick X still turns the yaw through the spin integrator;- in any other camera mode (the death camera of mode 8, or mode 12) the value passed to the aim view object is 0.
The aim view object's own blend and drawing are outside this spec.

### Spin (yaw) integration (Confidence: High; Medium for the screen direction)

Each follow tick the spin value I (0x150) takes the spin input: I <- I + D x a - I x b, with a = the profile spin gain (+0x18, 0.005) and b = the profile spin decay (+0x1C, 0.125). I is a yaw rate: the eye's yaw about the new focus increases by I each tick (follow step 6). For a steady D the rate tends to D a / b = 0.04 D rad per tick (2.4 D rad per second); with D = 1.3 that is 0.052 rad per tick (3.12 rad per second, one turn in about 121 ticks). After release I decays by the factor 0.875 per tick. The drift D is recomputed every tick, so camera reset need not clear it. Side swing (High): before the spin step, while the side-swing counter (0x1CC) is above 0, it is decreased by 1 and the side-swing increment (0x1C8) is added to I; the decay term still uses the I of the start of the tick. So the tick's update is I <- I + k + D x a - I x b, with k = 0x1C8 on a counting tick and 0 otherwise (Side-swing event). Screen direction (Medium): right stick right gives D < 0, which moves the eye toward the camera's left, so the view turns right.
Test vector (profile 0, D = -1.3 held from rest): I = -0.0065, -0.012188, -0.017164, -0.021519, -0.025329 and -0.028663 on ticks 1 to 6; -0.048401 at tick 20; -0.051983 at tick 60 (limit -0.052). After release the camera coasts a further 0.364 rad (20.9 degrees).

### Hold and lean terms (Confidence: Medium-High for the arithmetic; Medium for the meaning)

**Hold.
** When the drift base is nonzero, the hold counter is set to 150 ticks (2.5 s x 60 ticks per second). While the counter is at least 1, it decrements and the hold branch runs. In the hold branch: the side offset is set to zero, the heading bias is set to zero, the 0x1F0 bias is set to zero, and the side lean is set to zero. The side offset is not eased during the hold.
- While the drift base stays nonzero, the counter is re-armed each tick and reads 149 after each tick.
- After the drift base last became nonzero, the hold lasts 149 more ticks with the base at zero. The normal branch runs on the tick after that (see the test vector).
**Side offset (outside the hold).
** The side offset (0x1E0) eases toward the side target (0x1E4) by at most 0.1 units per tick, and snaps to the target when within 0.1 (ELF word 0x2DB3E0 gives the step).
**The lean target (High).
** The side lean and the side target frame the hero's current target. The test is the hero's target flag (hero record +0x15CC), read as 0 whenever the boot word 0x2DD6BC is nonzero (it is 0 on disc). The hero's own update sets the flag and the target point (hero record +0x15D0, the picked character's position) each tick: the target is the nearest live enemy within 20.0 units (word 0x2DD6C0) that the hero has noticed (an enemy is noticed once it is in front of the hero, with the dot product of the hero's forward row and the horizontal direction to it above 0.25, and stays noticed until it is 20 or more units away). A script-forced target replaces the pick for one tick. With no target the flag is 0. (Medium for the pick rules.) Let u be the horizontal unit direction from the hero's position to the target point, F the camera's forward row (0x20) and X its first row (0x00), and d = u . F.
**Side target.
** Computed only when the target flag is set and d < 0.
5. Let s = (target point - hero position, with y set to 0) . X, the target's sideways offset across the view. When |s| > 2.0 the target is (|s| - 2.0) / 1.0, capped at 10.0; otherwise it is zero. The side target uses the magnitude of s, so it is never negative. With d at least 0.5 (the target well inside the view) the side target is zero.
**Side lean (field 0x1E8).
** Computed only when the target flag is set, and zero otherwise and during the hold. The side lean is (1 - d) / 2 x 2.5, negated when u . X > 0 (the target is on the camera's first-row side). It is part of the spin sum, so a target off to one side turns the camera toward it.
Test vectors (synthetic): hero at the origin, camera forward (0, 0, 1), first row (1, 0, 0). Target at (3, 0, 4): u = (0.6, 0, 0.8), d = 0.8, side lean -0.25, side target 0. Target at (5, 0, 1): d = 0.196116, side lean -1.004855, s = 5, side target 3.
0. No target: both 0.
**Turn lean for state 0x12 (field 0x1F8).
** In player state 0x12 only: (1 - d) / 2 x 10.0, with the same sign rule as the side lean. Zero in other states. Medium-High for the arithmetic.
**Turn lean for state 0x17 (field 0x1FC).
** In player state 0x17 only: the same form with scale 10.
0. In this state the direction used for d is a blend of two vectors (0.25 and 0.75 weights), and two animation-flag tests are read before it. Zero in other states. Medium-High for the arithmetic; Low for the blend and the flag tests.
Test vector (hold timing): a nonzero base at tick 0 gives counter 149 after tick 0. With the base zero after that, the hold runs for ticks 1 to 149 and the normal branch runs at tick 150. Equivalently, with the base nonzero in updates 1 to 10 and zero from update 11 on, the counter reads 149 after update 10, updates 11 to 159 are in the hold (149 updates), and update 160 is the first normal update.

### Pad-axis converters (Confidence: High for the arithmetic; Medium for which stick)

Three converters turn a pad byte (0 to 255) into a number. They are used in different places; keep them separate. Each takes a pad byte b.
| Converter | Centre | Dead zone | Rule | Output range | Used by |
| --- | --- | --- | --- | --- | --- |
| A | 127.5 | 40 | d = b - 127.
5. If d > 40, output d - 40. If d < -40, output d + 40. Otherwise 0. | -87.5 to +87.5 (no scale) | Stick value (byte 0x127); yaw drift base (byte 0x12B); camera zone test |
| B | 128 | 32 | d = b - 128. If |d| < 32, output 0. If d > 0, output d - 32; if d < 0, output d + 32. Then output = that value x (1/128) x factor, where the factor is passed by the caller. | about -0.75 to +0.742 per unit factor | Mode 3 (four axes; factors below) |
| C | 127.5 | 30 | t = 127.5 - b. If |t| > 30 (strictly), the move amount is t x (1/127.5) x 10. Otherwise 0. No dead-zone subtraction is made. | -10 to +10 | Mode 7 |
Converter A divided by 87.5 gives a value in the range -1 to +1. Mode 3's per-axis factors (with converter B) are: byte 0x123 by 0.05; byte 0x11F by -0.05; byte 0x12B by -34.1333; byte 0x127 by -34.1333. Mode 7 uses converter C, with no state test.

### Mode 3 (debug free-fly camera) (Confidence: High for the arithmetic; Medium for the free-fly reading)

DORMANT: mode 3 is entered only by a debug routine that nothing references, and no caller passes 1 or 11 (see Mode entry points in the shipped game). The body is kept for a clean-side debug camera.
Modes 1, 3 and 11 run this handler. Rows R0 (first row), R1 (up), R2 (forward), position p (the base point 0x30). Converter B as in Pad-axis converters. Each update reads the followed player's input block:

1. **Terms**, all 0 by default.   - Only in player state 7 (state word 0x162): a = B(byte 0x123) x 0.05, b = B(byte 0x11F) x (-0.05), x = B(byte 0x12B) x (-34.1333), y = B(byte 0x127) x (-34.1333).   - In any state, from the held word (0x150): D-pad Up sets y = +1092.2667 and Down sets y = -1092.2667 (Down wins); Right sets x = -1092.2667 and Left sets x = +1092.2667 (Left wins). These replace the stick values outright.   - A height term h (L1 +0.1, L2 -0.1, R1 +0.01, R2 -0.01; the buttons are tested in the order L1, L2, R1, R2 and the last one held wins) is read only when camera field 0x68 is nonzero. That field is zero for the whole game, so h is always 0.
2. **Move.
** p' = p + (x / 64) R0, then p' = p' + (y / 64) R2, then p'.y = p'.y + h (h is a plain float, added without rounding). So a held D-pad direction moves the camera 17.0667 units per tick (1024 units per second); a full right-stick push about 0.396 units per tick.
3. **Turn.
** If

|R2.y| > 0.92: forward' = R2 - b R1 and side' = R0 + a R1. Otherwise: negate a when R1.y > 0; forward' = R2 + a R0 - b R1; side' = R0 - a (R2.x, 0, R2.z), and then the y of side' is multiplied by 0.15 (this damps roll).
4. **Rebuild.
** F = normalise(forward'), S = normalise(side'), up U = normalise(F x S), first row = U x F. The camera is posed at p' with these rows.
Directions: right stick right, D-pad Right and left stick right all move or turn toward minus the first row, the same side as right stick right turns the normal camera (screen side per the Spin section). In the tick, while the mode is 3 and the focus character has a ground entry, that entry becomes the camera's ground entry (0x1D4) and the fade ramp and the ground probe are skipped (see Mode routing).
Test vectors (rows identity, p = (0, 0, 0), sticks centred at 128):- D-pad Up held (any state): p' = (0, 0, 17.0667); rows unchanged.
- Player state 7, right stick X byte 255: x = (95 / 128) x (-34.1333) = -25.3333; p' = (-0.395833, 0, 0).
- Player state 7, left stick X byte 255: a = 0.037109, negated to -0.037109; F = (-0.037084, 0, 0.999312), first row (0.999312, 0, 0.037084), up (0, 1, 0): a yaw turn of 0.0371 rad per tick.
- L1 held: no change (h needs field 0x68, which is 0).

### Mode 7 (debug top-down camera) (Confidence: High for the entry and the rule; Medium for the debug reading and for the up-row vector)

DORMANT: no caller passes 7 to the mode setter (see Mode entry points in the shipped game). The body is kept for a clean-side debug camera.
**Entry.
** Setting mode 7 raises the camera position p (0x30) so it is at least 100.0 units above the ground under it: g = the stored ground height (0x1DC), re-probed at p's x, z when the camera has a ground entry (0x1D4); if p.y < g + 100, then p.y = g + 1

00. The same rule with the profile minimum height (+0x20, 1.0) in place of 100 is the minimum-height rule of Ground clearance.
**Update.
** Let f = the heading of the focus character's locomotion block (block +0x40; the block is the one at character +0x7CC, SPEC-0040) and D = (0, -1, 0). The heading is a stored unit vector (fourth component 1). The block reset sets it to (1, 0, 0). The normal locomotion writes it only on ticks when the left stick is off centre: it becomes the unit stick direction turned into world space with the camera's forward row and the horizontal right axis (up x forward). On ticks with the stick centred nothing writes it, so f keeps the direction of the last stick push (Medium-High; other locomotion states also set it, by their own rules). While mode 7 is active the camera's forward row is D itself, so a left-stick push builds the next heading from a vertical forward row; up x D is zero, so a push with a side part gives a degenerate (zero-length) heading (Medium; mode 7 is dormant). The rows are set to first row = D x f, up row = f, forward row = D: the camera looks straight down. Then, for the pad bytes in the order 0x12B (right stick X), 0x127 (right stick Y) and 0x11F (left stick Y), with t = 127.5 - byte (converter C): if

|t| > 30, p moves by 10 t / 127.5 along, respectively, the first row, the up row (f) and D. So the right stick pans the camera and the left stick Y moves it up and down, up to 10 units per tick. Byte 0x123 is not used. The mode is not gated by player state. There is no height clamp after the entry, so the left stick Y can take the camera below 100. Converter C has no dead-zone subtraction: a byte just outside the dead band gives a jump, not a small step.
Test vectors: entry from p = (5, 3, 5) with stored ground 0 and no ground entry gives (5, 100, 5); from (5, 150, 5) nothing changes. Update with f = (0, 0, 1): first row (-1, 0, 0); right stick X byte 255 (t = -127.5) moves p by (+10, 0, 0); right stick Y byte 0 moves it by (0, 0, +10); left stick Y byte 0 moves it by (0, -10, 0); byte 97 moves it by 2.392157.

### Dead-band yaw follow (mode 12) (Confidence: High for the rule; Medium for its inputs)

DORMANT: its only entry is Set

CameraToTrackHeroMode, which no level script calls.
Let d be the target yaw minus the current yaw, with no wrap applied. The target yaw is taken from the followed object's direction (Medium for which two values form the error).
- If the absolute value of d is below 0.1 radians, the yaw does not change (the dead band).
- If the absolute value of d is from 0.1 to 0.2 radians, the yaw changes by d x 10 x (|d| - 0.1). This ramps the yaw in smoothly from zero.
- If the absolute value of d is above 0.2 radians, the yaw changes by d (it snaps to the target yaw).
The eye stays fixed; only the view direction turns. Entry: SetCameraToTrackHeroMode(track) poses the eye at the first key of the world camera track, looking at the hero, and sets mode 12; EndCameraTrackHeroMode() sets mode 0 only when the mode is 12. The mode setter can also set mode 12. No level script calls either entry.

### Blended transition (mode 8) (Confidence: High)

**Entry** (target object, radius R, yaw offset o, height h, fractions k_d, k_y, k_e, pivot step s, place-now flag):

1. Stores the target (0x200), o (0x210) and h (0x214); distance = sqrt(R² + h²) (0x208); elevation = asin(h / R) (0x218; the quotient uses R, not the distance, so the value is slightly above the angle of the point (R, h)); the fractions (0x21C, 0x220, 0x224) and s (0x228). R itself is not stored.
2. Pivot (0x240) = the camera point query at entry (in mode 0: the follow focus point). Target point (0x250) = target position + (0, target height, 0).
3. If place-now is set: eye = pivot + sphere(distance, yaw of the target's forward row + o, elevation), the pivot becomes the target point, and the camera looks from the eye at it.
4. Latch (0x260) = 0, mode = 8.
**Update** (latch 0 only; see Blocked latch), in order:1. **Pivot step.
** d =

|target point - pivot|. If s < d the pivot moves by s toward the target point; otherwise it is set to the target point. With s = 0 (both callers) the pivot does not move. The step is compared with the remaining distance; this is not a defect.
2. **Desired and current polar.
** Desired = (distance, yaw of the target's forward row + o, elevation). Current = polar(camera position - pivot) (Polar convention). Lane 0 is the distance, lane 1 the yaw and lane 2 the elevation.
3. **Distance.
** new = old x (1 - k_d) + desired x k_d. Snap to the desired value when |new - desired| < 0.1.
4. **Yaw.
** Before the blend, if the current yaw and the desired yaw have opposite signs, the desired yaw is shifted by one full turn (2 pi) toward the side it is on: a negative target gets +2 pi, a positive target gets -2 pi. Then new = old x (1 - k_y) + desired x k_y. Snap when within 1e-6 (ELF word 0x2DB414). This is a sign test, not a shortest-arc rule (see Edge cases).
5. **Elevation.
** new = old x (1 - k_e) + desired x k_e. Snap when within 1e-6.
6. **Pose.
** Eye = pivot + sphere(blended distance, yaw, elevation); the camera looks from the eye at the pivot.
Mode 8 does no ground clearance and no follow wall push; the contact pass of Blocked latch is its only collision, and the ground probe runs after it as after every mode other than 0. Callers: the death camera below (live), and SetHeroSceneCamera(a, b), which uses the same entry with o = a, R = b, h = 1.5, all fractions 0.01, s = 0 and no placement (no script sites). Return to normal (below) leaves this mode at once and sets mode 0.

### Death camera (mode 8 at game over) (Confidence: High for the trigger and the arguments; Medium-High for "in front")

In hero state 16 (dead), on the game-over branch of SPEC-0055 (both heroes dead, the hero switch refused, or deadly ground with no respawn index): when no game over is pending and the game-over timer is idle, the timer is started and, unless the camera is already in mode 8, mode 8 is entered on the dying hero with R = 15, o = 0, h = 1.5, all fractions 0.01, s = 0 and no placement. Entry values: distance 15.0748, elevation 0.10017 rad (5.74 degrees). The pivot is the follow focus at that moment and stays there (s = 0), so a body that moves keeps being framed at the death spot. The camera swings round to the hero's forward side (its forward row) and pulls back, ending 15.07 units from the pivot at 5.7 degrees elevation, in front of the hero. A contact push freezes it (see Blocked latch). Mode 0 returns with the next world load (camera initialisation).
Test vector (synthetic, exact arithmetic): pivot (0, 2, 0), hero forward +z (yaw +pi/2), camera at polar (9.7434, -pi/2, 0.2). Distance, yaw (degrees) and elevation after ticks 1, 30, 60, 90 and 180: (9.7967, -91.80, 0.19900), (11.1312, -136.85, 0.17401), (12.1577, -171.51, 0.15479), (12.9170, 162.85, 0.14057), (14.2015, 119.49, 0.11652). The distance snaps to 15.0748 on tick 396; the yaw tends to 90 degrees (eye (0, 3.5, 15.0) about the pivot origin). The yaw sign shift sends the target to -270 degrees, so the eye passes behind -x (yaw -180) and keeps turning; after the polar wraps past -pi the signs agree and the blend continues to +90.
Forward row (Medium-High). A character's world matrix is stored as four rows at +0x290 (first row), +0x2A0 (up row), +0x2B0 (third row) and +0x2C0 (fourth row), the same row order as the camera matrix, whose third row is the view direction (Polar convention). The death camera's target yaw is the yaw of the third row. Two other uses agree that this row is the character's forward: SetToNormalCamera places the eye at angle pi from it (behind the hero), and the hero's feed action spawns its object at the hero position plus 2.8 times this row, with this row as the object's direction. Test vector: hero at (0, 0, 0) with third row (0, 0, 1): the feed object starts at (0, 0, 2.8), and the death camera's target yaw is +90 degrees (eye on the +z side, as in the vector above).
Under the game-over page (High). The game-over timer starts on the death tick and sets the game-over flag on its 91st step (1.5 s, SPEC-0055); the page appears on the next pass. The camera mode update runs on every pass, also while the world is frozen under the page (Update order), so mode 8 keeps blending behind the page until the player leaves it. The world draws run in the frozen state too (SPEC-0026, draw phase), and the page's picture is 512 by 128, so the swinging view of the world stays visible around it. Test vector (the synthetic case above): at the page's first pass (about tick 92) the camera is close to its tick-90 pose (12.9170, 162.85 degrees, 0.14057); it keeps turning under the page and reaches the 15.0748 distance on tick 396 (6.6 s after the death).

### Blocked latch (modes 8, 9 and 10) (Confidence: Medium-High)

Each update of modes 8, 9 and 10 first checks the blocked latch (0x260). If the latch is 0, a contact pass runs on the camera position (0x30): it pushes the position out of the world's moving objects, the ground frames and collision primitives of its grid cell (kinds 1 plane, 3 cylinder, 4 and 7 box or zero-radius contact by the sign of +0x2C, 2 cone skipped) and the characters. It returns nonzero when any push happened; the pushed position is kept and the result is stored in the latch. If the latch is nonzero, the blend or follow step is skipped for this tick and from then on. The latch is cleared only when the mode is entered again (entry sets it to 0). So once the camera position touches something in these modes, the camera freezes in its last pose until the mode is re-entered or return-to-normal runs (see Edge cases). The pass uses the same four lists, in the same order, and the same push pieces as the follow wall push (see Walls and the stuck timers), so its push geometry is that of the wall push (eye radius 1.25). It differs only in its target and result: it works on a copy of the camera position (0x30), writes the copy back only when something pushed, and returns whether anything pushed. (Medium-High.)

### Smoothed follow (mode 9) (Confidence: Medium for the update; its entry has no caller)

DORMANT: the mode 9 entry has no caller, so the mode never runs on this disc.
The mode keeps a pivot and a target point.
1. **Target point.
** The target point comes from the target object (handle 0x200), or from a second object (handle 0x204) when the selector at 0x230 is 2.
2. **Pivot step.
** The pivot moves toward the target point by the step, using the rule of mode 8 step 1 (the step is compared with the remaining distance). The step and the three fractions (0x21C, 0x220, 0x224) are as in mode 8.
3. **Look blend.
** The look direction blends toward the target direction component by component, using the rules of mode 8 steps 2 to 5 (the same snaps).
Because the entry has no caller, it has no shipped step, fractions or selector values; the clean side leaves the mode out (Open question 5).

### Fallback pose (mode 10) (Confidence: High for the lookup; Medium for the entry; Low for the stored target)

DORMANT: the mode is entered only from SetAI_Dist

Camera and SetHeroDistCamera, and no level script calls either.
**Lookup and entry.
** The entry takes the camera, a target character and three numbers: the horizontal reach R, the height h and the yaw offset o. SetAI_DistCamera(id, a, b, c) resolves its first argument as an entity id of the current world (the tag 0x12 entity-id table of SPEC-0006) to a character record and passes R = a, o = b, h = c. SetHeroDistCamera(a, b, c) passes the selected hero and uses the three floats in a different order: R = a, h = b, o = c. If the id names no loaded character (an unknown id, or a slot whose region is not loaded), the mode does not change. Otherwise the entry stores the target (0x200), the yaw offset o (0x210), h (0x214), the distance sqrt(R^2 + h^2) (0x208) and the elevation asin(h / R) (0x218), the same formulas as the blended transition (R itself is not stored). The pivot (0x240) and the stored target point (0x250) are both set to the target's position plus (0, the target's height value, 0); the eye is placed at once at pivot + sphere(distance, yaw of the target's forward row + o, elevation), the camera looks from it at the pivot, the blocked latch is cleared, and the mode is set to 10 (High for the stores and formulas; Medium for the target-height term). Test vector: SetHeroDistCamera(8.0, 2.0, 0.0) with the hero at the origin facing +z and height value H gives distance 8.2462, elevation 0.25268, the eye at (0, H + 2.0616, 7.9844) and the camera looking at (0, H, 0).
**Update (blocked latch 0 only).
** The pivot is set again from the stored target point each tick, so it stays where the entry put it and does not follow the target. The eye is rebuilt as pivot + sphere(distance, yaw of the target's forward row + o, elevation), with no blend, and the camera looks from it at the pivot; only the yaw term follows the target. The contact pass of modes 8 to 10 runs first; when it pushes, the latch is set and the update does nothing further (see Blocked latch). **[High]**
There is no fallback record built from the player position and heading: the entry is the only writer of the mode 10 target.

### Return to normal (modes 8, 9 and 10 only) (Confidence: High)

The internal return routine acts only when the mode is 8, 9 or 10. It sets mode 0, then runs camera reset with the base point and the pivot (0x240). Modes 3, 7, 11 and 12 are left as they are by this routine; all four are dormant on this disc. Its callers are the three return-to-normal script calls (ReturnToNormalCamera and both forms of SetToNormalCamera), through which the full item showcase also runs it, by calling ReturnToNormalCamera(-1, true) as its last step; none of them handles mode 12. Mode 12 is left only by EndCameraTrackHeroMode or by the camera initialisation at a world load. So ReturnToNormalCamera in mode 12 still moves the eye (through the pull-in setter) but the dead-band yaw follow keeps running.

### Target distance (Confidence: High)

The target distance is P.r = profile distance x (1 - 0.65 sin^2(e)) + side offset (0x1E0), where e is the current eye elevation (the elevation lane, 0x148) and 0.65 is the data word 0x2DB3F0. When e <= 0 the 0.65 term is dropped: P.r = distance + side offset. This is the only rule that sets the follow distance; the 0.25 factor of follow step 5 belongs to the focus height in hero state 21.
Values (side offset 0): profile 0 at e = 0.2 gives 9.7434 (the eye 1.9357 above the focus and 9.5492 away horizontally); e = 1.2 gives 4.3535; e = 1.5 gives 3.5325; e <= 0 gives 10.
0. Profile 1 at 0.25 gives 8.6419. Profile 3 at 0.6 gives 11.8915 (the eye 6.7144 above the focus).

### Dormant eye-lag and rig path (DORMANT: no known live caller) (Confidence: Medium for what it computes)

These two paths are not in the live follow chain. Neither has a known caller, by direct call or by indirect call through a table. Do not implement them as part of the normal follow unless a caller is identified (Open questions). They are described so that the clean side can recognise them.
**Eye-lag stage.
** When the eye-to-base distance is above 0.05 and the elevation is at most 1.5, a lag flag (0x19C) is set to 1. While the flag is set, each tick the eye's distance offset moves toward its target by 0.025 units, or by 0.1 units when a second player condition holds. Movement stops when the remaining difference is at most 0.05 (the stop use of word 0x2DB3BC). The lag ends early when the offset is above 1.5 units. The lag stage runs the lean path below each tick.
**Eye placement with stick lean (the rig lean path).
** Each call, with two stick values (X and Y), the eye's lateral and forward vectors move toward their target vectors by a blend fraction k = 0.05 x |stick value| (the data word 0x2DB3BC), one fraction per stick axis:- The lateral blend runs when the X value is nonzero. A positive X blends one way and a negative X the other.
- The forward blend runs for a positive Y only when the current forward component is above -0.9, and for a negative Y only when it is below 0.
85. Otherwise it does not run. These gates are fixed constants (Open questions).
- Both vectors are then renormalised and the basis is rebuilt.
**Camera rig update.
** The rig frame is built from the two stick values, blended as above. Collision probes from the rig's focus toward the eye use the base-point blocked test and the probe distances 1.0, 2.0 and 5.0 along the lean vector. A caller threshold of 0.1 units applies. The rig runs only from the eye-lag stage, which is unreachable, so it never runs in the shipped game and the clean side leaves it out (Open question 1).

### Ground probe (per tick) (Confidence: High)

After the mode handler, in every mode other than 0, and after a scripted play's update, the camera probes the ground straight down at its position's x, z, starting 2.5 units (a fixed constant) above its eye height. The probe wrapper clears the camera's ground-entry slot before each query. It tries the current world first. If that finds nothing and the current world is not the interior buffer, it tries the other resident world. A hit stores the world that supplied the entry; if both worlds miss, the world field is set to the current world. When the current world is the interior buffer and the probe misses, the second-world query is skipped and the world field retains its previous value. If the current-world pointer is null, this helper leaves the entry, world and height fields untouched. In mode 3 with a focus ground entry the probe is skipped (see Mode 3). Mode 0 does not run this probe; it has its own ground clearance. When a current world is available, the end-of-tick room update uses the chosen entry's room byte when an entry exists; otherwise it uses the current world's room value. The end trigger pass uses any nonzero world still stored in the camera record, so an interior-world miss can leave it querying the previous world's trigger grid even though the current entry is null. It is a vertical ground probe, not a line from the eye toward the focus, so it is not the line-of-sight test of Walls and the stuck timers.
End of the update (High). After the probe the update does only two more things: it stores the room index, and, in every mode and when a ground world is stored at 0x1D8, it runs a trigger pass at the camera position. The trigger pass looks up the camera's grid cell in the world of its ground entry and, for each collision primitive there of kind 1 or kind 8, runs a point test with a 1.0 radius at the camera position and hands a hit to the world's trigger handler. It never moves the camera. There is no second orbit or shrink pass in the update; the only shrink is the one in Recentre pass and R3. Test vector: camera at (0, 5, 0) in a cell with one kind 1 and one kind 3 primitive: only the kind 1 primitive is tested, and the camera position is (0, 5, 0) after the pass whatever the result.

### Ground clearance (Confidence: Medium-High)

Each follow tick the ground height h under the eye is probed (from 2.5 above the eye). If the eye is lower than h + 2.0 (ELF word 0x2DB3C4): when the focus is above the eye, the focus character's flag test is 0 and h + 2.0 is below the focus height, the eye slides along the line toward the focus until it is exactly 2.0 above h, and the polar state and the elevation targets are rebuilt from the new eye. Otherwise the eye is simply raised to h + 2.
0. A separate minimum-height rule, used by some camera paths, raises the camera position (0x30) to at least the profile minimum height (+0x20, 1.0) above the stored ground height (0x1DC), re-probed at its x, z when the camera has a ground entry (0x1D4). The mode 7 entry applies the same rule with 100.0.

### Walls and the stuck timers (Confidence: Medium-High for the flow; Medium for the push geometry)

After the spring step the eye (0xE0) is pushed out of, in this order:1. the world's list of 0xB0-byte object records (count at world +0x580C, records from +0x5810): the moving objects whose active word is set;2. the ground frame records of the grid cell under the eye (entries whose mask passes; records of type 6, the low byte of +0x4C, are skipped). Each record is tested with the single-margin plane test, whose margin is the eye push radius 1.25 (0x2DB400): a wall hit reports the depth 1.25 - d inside the margin, a ground hit reports zero depth (SPEC-0018, Per-record contact test, camera form; Medium, partly described). This is not the characters' test: the heroes use the two-parameter plane test of SPEC-0040 (reach = body radius 1.175 only; the body height 3.5 only sets the vertical window), and they do test type 6 records, as two-sided walls (SPEC-0018, Frame-record type and type-6 survey). So the eye passes through type 6 walls that stop the hero;3. the collision primitives of that cell (cell list A of SPEC-0018), by the kind byte at +0x10 of the loaded primitive: 1 plane; 3 cylinder; 4 or 7 with the radius value +0x2C above 0 box push, 4 or 7 with +0x2C at or below 0 zero-radius contact; 2 (cone) skipped; other kinds ignored. The box push uses an eye radius of 1.25 (0x2DB400);4. the characters listed by the world's regions (their sub-shapes).
This is the pass that SPEC-0018 calls its contact pass (driver): it belongs to the camera, not to the characters.
Each push adds its normal to a contact list (at most 19 entries; a normal within 0.999 of a stored one only refreshes it). Each entry lives 40 ticks (word 0x2DB40C). The countdown runs on a tick only while the focus hero is moving (hero state 2, 3, 4, 5, 6 or 32, or sneak state 22, 39 or 40 while the hero's current animation request is 85) or the hold word (0x26C) is nonzero; with neither, entries do not age. The scan visits one slot more than the list holds (SPEC-0018, Follow-camera contact-normal countdowns).
Stuck timer: driven by the eye pass failure flag, not by every push (see Eye pass failure flag below). On a failing tick the counter increases; when two or more contacts are active, or once the restore latch is set, the eye is put back at the previous tick's final eye (0xF0). When the counter exceeds trunc(0.05 s x 60) = 3 (ELF word 0x2DB3E8), that is on the fourth consecutive failing tick, the recovery (recentre) pass runs and the state clears. A tick with no failure while the latch is clear resets the counter.
Line-of-sight timer: if the line from the eye to the focus is blocked (the world segment test or the camera visibility test), a counter increases; when it exceeds 1.0 s x 60 = 60 (word 0x2DB3EC), that is on the 61st consecutive blocked tick, the recentre pass runs. A clear tick resets it.

### Eye pass per-primitive routines (Confidence: High that they belong to the camera; Medium for the steps)

The eye pass has five per-primitive routines that share one entry pattern (SPEC-0018 points here for them; they serve the camera, not the characters), plus the box push. The moving body is the camera eye (0xE0). Which routine handles what (High):
| Routine | Handles | Class bits it reads (+0x12) | Posts the side-swing event |
| --- | --- | --- | --- |
| Frame-record routine | ground frame records of the eye's cell (type 6 skipped) | none | yes |
| Plane routine | cell list A primitives of kind 1 | 0, 1 and 2 | yes |
| Cylinder routine | cell list A primitives of kind 3 | 0, 4 and 5 | yes |
| Zero-radius routine | kinds 4 and 7 with the radius value +0x2C at or below 0 | 0 and 5 | yes |
| Character routine | each sub-shape of the region characters | none | yes |
Kind 2 (cone) and every other kind run no routine. Every routed primitive routine first applies the primitive mask gate in step 1. Its class gate is then: kind 1 requires class 0 or 1 and rejects class 2; kind 3 requires class 0 or 4 and rejects class 5; zero-radius kinds 4 and 7 require class 0 or 5; positive-radius kinds 4 and 7 require class 0 or 3. The character-subshape routine has no class-bit gate. The moving-object push uses the same geometric helper as positive-radius boxes but enters through a separate call path that does not apply the primitive mask or class gate. These are the complete static class predicates for the live eye-pass dispatch (High).
2. Class gate. The applicable predicate is the one for the dispatched primitive route listed above. A failed predicate returns with no eye change.
3. Contact test. The routine tests the eye as a sphere of radius 1.25 (0x2DB400) against the primitive: a plane test for kind 1, a cylinder test for kind 3, and for the zero-radius kinds a sweep from the eye along its per-tick velocity. With no contact it returns: nothing changes and the failure flag is not touched. The zero-radius routine hands faces whose normal has |y| above 0.85 to a separate floor and ceiling push that moves the eye out along the hit normal; that path posts no side-swing event and sets no failure flag (Medium).
4. Contact. The hit normal N is added to the contact list (a normal within 0.999 of a stored one refreshes that entry; at most 19 entries) and the eye is moved to the eye plus N times the contact depth. When the previous failure, the failure flag or the restore latch is already set (see Eye pass failure flag), the routine stops here.
5. Settling (Medium-High). The routine runs its clear test of the eye position: for the plane routine the camera's own shape-sweep test, for the zero-radius routine a segment test between the focus point, lowered by the primitive's own height term, and the eye. Let D be the settling direction: for the plane routine the horizontal unit vector perpendicular to the up vector and to the line from the eye to the primitive's reference point, reversed when it points against the eye's heading; for the zero-radius routine the primitive's own face normal, reversed when it points against the heading. If the clear test passes, the eye is nudged along D by the contact depth, but only when the stuck counter is 0, the focus hero is moving or the hold word is set, and D has a positive dot product with every normal already in the contact list (an empty list passes); no event is posted. If the clear test fails, the routine probes a segment of length 10.0 (0x2DB3FC) from the eye along D and then along minus D. When either is clear the contact is resolved: if the hold word (0x26C) is zero, the side-swing event is posted to the camera with the rate 0.01 (0x2DB404), the resolving direction, the duration 10 ticks (0x2DB408) and the direction flag 1; with the hold word nonzero nothing is posted and the eye position is kept. When neither is clear, the failure flag (0x508) is set to 1 and nothing else changes. At the end of a pass without failure, the zero-radius routine also removes the hit-normal component from the eye's velocity and restores its speed, so the eye slides along the surface (Medium). Test vectors (synthetic): a plane contact of depth 0.2 with the hero standing and the hold word 0: the eye moves out by 0.2 along N, and when the clear test passes nothing else happens; the same contact with the hero walking, a failing clear test, D clear and the hold word 0: the event is posted with rate 0.01, 10 ticks and flag 1; with the hold word 1 it is not posted; with D and minus D both blocked the failure flag is 1 and the eye has only been moved out along N.

### Side-swing event (Confidence: High)

The event takes a rate k, a vector N, a duration n (ticks) and a direction flag.
- Flag 0: the side-swing increment (0x1C8) = k and the counter (0x1CC) = n.
- Flag 1: only when N . F > 0, with F the camera's forward row (0x20): the increment = k, negated when N . R < 0 (R the camera's first row, 0x00); the counter = n. When N . F is at or below 0 nothing is written.
Each following follow tick adds the increment to the spin value I while the counter runs down (Spin), so the eye swings sideways about the focus for n ticks and then the extra rate decays with the spin decay. Every poster uses the flag 1 and the rate 0.01.
Posters: the five eye pass routines above (10 ticks); two character contact routines and one hero routine of the motion code (30 ticks each), among them the general case of the hero's ledge hand-off (SPEC-0018), which posts only when the hero's word +0x148 is set.
Test vector (exact arithmetic; spin input D = 0, I = 0 at the start, flag 1 event with N . F > 0 and N . R >= 0, k = 0.01, n = 10): I after ticks 1, 2 and 3 is 0.01, 0.01875 and 0.0264063 (I <- 0.875 I + 0.01); after tick 10 it is 0.0589539 (0.08 x (1 - 0.875^10)); from tick 11 on it decays by 0.875 per tick. With N . R < 0 the same values are negative. With N . F <= 0 nothing changes.
Failure is the last case of step 5: neither probe direction is clear, so the failure flag (0x508) is set to 1 and nothing else is written. The flag's effect is below.

### Eye pass failure flag (Confidence: High for the flag, the counter, the limit and which primitive kinds each routine handles)

The per-primitive routines of the follow camera's eye pass (above) share five words of the camera record: the contact-normal count (0x500), the previous failure (0x504), the failure flag (0x508), the restore latch (0x50C) and the stuck counter (0x510). The camera's contact-block reset clears all five.
1. At the start of each eye pass: failure = 0.
2. A per-primitive routine that cannot resolve its contact sets failure = 1. While any of the previous failure, the failure or the latch is set, the later routines of the same pass skip their resolution step.
3. At the end of the pass:
- if failure = 0 and latch = 0: counter = 0;   - otherwise: if the contact count is at least 2 or the latch is set, the eye is put back at the previous eye (0xF0) and latch = 1; then counter = counter + 1;   - if the counter is now greater than trunc(0.05 x 60) = 3 (0.05 from 0x2DB3E8, times the tick rate), the camera recovery routine (the recentre pass) runs and the counter, the latch, the failure and the previous failure are all cleared.
4. Previous failure = failure.
So recovery runs only after more than 3 consecutive failing ticks.

### Recentre pass and R3 (Confidence: High for the trigger, the shrink and the constants; Medium for the candidate order)

**Trigger.
** The pass runs on the R3 press edge (bit 0x400 of the player's press-edge word 0x154) when the latch (0x1D0) is 0, and from the two timers of Walls and the stuck timers. The latch is released (set to 0) on a tick when it is 1 and bit 0x400 of the held word (0x150) is clear, that is once R3 is released.
**Start.
** latch = 1, manual flag (0xC8) = 0; the eye is remembered.
1. **Behind.
** P = polar(-d x facing), with d the target distance and facing the focus character's facing vector; then P.e = the profile default elevation and t_e = that elevation. Candidate eye = F + sphere(P). If the line from it to F is clear, it is used.
2. **In front.
** Otherwise the candidate is F + d x facing (at focus height); if clear, P = its polar with the default elevation, and it is used.
3. **Shrink.
** From the chosen candidate, let r be its distance. While r > 1.0 and the segment from the focus to the eye is blocked by the level (either of the two tests below), r = r - 0.25, place the eye at r on the same yaw and elevation, set the reseat flag (0x198) to 1, and test again. The loop stops when the segment is clear, or when r <= 1.0 (the test is not repeated at 1.0 after the last shrink).
4. **Commit.
** The eye is placed at the result and the wall push runs. If the eye-to-focus length is at least 0.1, camera reset runs (see Camera reset). Otherwise the remembered eye is restored and steps 2 and 3 are redone from the front candidate (Medium).
The two blocked tests (Medium-High) are different tests on the same candidate, and either one blocks:- Line test: the segment query of SPEC-0018 between the focus and the candidate eye, with the restriction flag set, so two-sided walls (frame type 6) are ignored. Any hit blocks.
- Buried test: a ground probe (SPEC-0018) at the candidate eye's x and z with a ceiling 5.0 above the focus height. It blocks when the height returned is more than 5.0 above the focus height. That happens only through the probe's fallback, when every ground surface there lies above the ceiling, so the candidate is under a hill or ledge that rises more than 5 units above the focus.
Test vectors (buried test): focus (0, 2, 0), candidate eye at x 0, z -9.55 (ceiling 7.0). Only ground at height 8.0 there: the probe returns 8.0, which is above 7.0, so the candidate is blocked and shrinks. Ground at 6.5 (and 8.0): 6.5 is accepted, so the candidate is not blocked by this test.
Test vectors (shrink):- Distance 6.0, segment blocked for r > 3.0 (clear at 3.0). Tested distances: 6.0, 5.75, 5.5, ..., 3.0 (13 tests, 12 shrinks). Final distance 3.0, length at least 0.1, so the pass commits and resets.
- Distance 4.0, always blocked: tests at 4.0, 3.75, ..., 1.25 (12 tests, 12 shrinks). The loop stops at 1.0 without another test. Final distance 1.0.
- Distance 1.25, always blocked: one test at 1.25, shrink to 1.0, loop ends. Final 1.0, no test at 1.0.
- Distance 1.0 or less: no test and no shrink.
- Distance 0.05 (below the 0.1 commit length): no shrink; the final length 0.05 is below 0.1, so the eye is restored to its position before the pass.

### Pose setter with pull-in (set-pose rule, not per tick) (Confidence: High for the rule)

The pull-in setter is the camera object's pose hook. It is used by the boot and world-load attaches, the restart placement and the three return-to-normal script calls (see Placement after a load or restart and Scripted cameras), not per tick. Given a wanted eye E:1. The focus point F is recomputed (the follow focus).
2. The eye height is replaced by the focus height: E' = (E.x, F.y, E.z). This happens in every case, so the wanted eye's own height is never kept.
3. The ground is probed at (E.x, E.z) from F.y + 5.
0. If the ground there is higher than F.y + 5.0, the horizontal offset is halved: E' = (F.x + 0.5 (E.x - F.x), F.y, F.z + 0.5 (E.z - F.z)) (factor 0.5; ELF word 0x2DB3B4).
4. E' (with w = 1) is written to the camera position (0x30), the eye (0xE0) and the previous eye (0xF0); the eye motion direction (0x100) is cleared to (0, 0, 0, 1).
5. The wall push runs; the eye is set to the pushed position, the focus is recomputed and camera reset runs with the eye and the focus.
Because of step 2 the eye starts level with the focus (elevation 0); the follow chase then raises it toward the profile elevation over the next ticks.
Test vectors: focus (0, 2, 0), wanted eye (0, 4.5, -10). Ground at (0, -10) at 6.0: eye (0, 2, -10). Ground there at 9.0: eye (0, 2, -5).

### Camera reset (Confidence: High for the writes)

Camera reset:- zeroes the spin value I (0x150) and the side-swing counter (0x1CC);- sets the reseat flag (0x198) to 1;- sets the target and current polar vectors to the polar of (eye - focus), and the elevation target (0x154) and the manual elevation (0xC4) to that elevation.
It does not write the spin input D (0x264), the stick value (0x268) or the increment (0x1C8). D is recomputed every tick, so no stale value persists beyond the next tick. Reset runs after a level reset, after a return-to-normal, after a recentre commit and after each placement.

### Camera initialisation (Confidence: High)

When the camera is initialised, its offset is set to (0, 0, 50) and its forward direction to (0, 0, -1); both are fixed constants (50.0 and -1.0), not boot data words. Read together, the offset puts the eye 50 units along +Z from the focus and the forward vector points back at the focus along -Z (the frame convention is in Polar convention). The initialisation then attaches the camera to the player (see Placement after a load or restart). The camera's field reset at construction also sets the focal length to 512.

### Placement after a load or restart (Confidence: High)

On a restart the hero is put at the restart point P with facing f. The camera is posed at eye = P - 10 f + 2.5 (0, 1, 0), looking along f (right = normalise(up x f)). The pull-in setter then runs (see Pose setter with pull-in), which drops the eye to the focus height (so the 2.5 lift does not survive), then the wall push and a camera reset. The follow spring settles the camera in about 45 to 60 ticks (first test vector of Follow camera). Attaching the camera to a character (boot, new game, start point, hero switch) instead puts the eye at position - d x facing (d = the target distance at the current elevation, no lift); the boot and world-load attaches then run the pull-in setter, the hero switch does not (see below). The up vector used here is the run-time block at 0x3932E0, (0, 1, 0, 0).
**The attach in detail.
** The attach sets the focus object (0x70) to the character, poses the camera from the character's position and its locomotion facing, and sets the manual flag (0xC8), the side-swing counter (0x1CC), the R3 recentre latch (0x1D0) and the camera's ground entry (0x1D4) to 0. It takes a hook flag. With the flag 1 (the boot and world-load attaches) the camera object's own hook runs as part of the pose step; with the flag 0 the hook is skipped. The hook is the pull-in setter (the camera class entry that the restart placement and the return-to-normal calls also use). The camera reset runs in both cases. So after a hero switch the eye stays at the incoming hero's position height (no drop to the focus height and no pull-in), and after a boot or world-load attach it is level with the focus. (High.)**Hero switch (camera hand-off).
** On a hero switch (SPEC-0053 section 3.6b) the only camera action is this attach, made to the incoming hero at the swap's select step with the hook flag 0, followed by the camera reset. Then the followed player record (0x60), the camera's ground reference, is set to the incoming hero's ground-reference record (hero record +0x9A0). The incoming hero has already taken the outgoing hero's position and facing, so the eye lands behind the same spot. There is no camera turn and no blend: the view cuts on the selection tick. When the swap's second flag is set, the white flash (SPEC-0053 section 3.5) is opaque at that moment and hides the cut. (High for the calls; Medium for the visible result.)Test vector: P (10, 0, 5), f (0.6, 0, 0.8), hero height H: the posed eye is (4, 2.5, -3); after the pull-in setter it is (4, H, -3), or (7, H, 1) when the ground at (4, -3) is higher than H + 5 (focus (10, H, 5), no bias).
Test vector (hero switch): outgoing hero at (10, 0, 5) facing (0.6, 0, 0.8), target distance d = 10, switch accepted: on the selection tick the camera's focus object and followed player record are the incoming hero's, the eye is (4, 0, -3) before the reset, the manual flag and the R3 latch are 0, and no turn or blend runs on later ticks.

### Fade ramp (follow step 12) (Confidence: High for the rule; High for the sort rule and Medium-High for the composite draw in the renderer use)

Let v be the eye-to-focus distance (the length of focus minus eye):- v below 1.7: target fade = 0.
- 1.7 to 5.0: target = (v - 1.7) / 3.3 x 128.
- 5.0 or more: target = 128.
The current fade level moves toward the target by 5 per tick (snapping when within 5). It is clamped to 0 to 128 and written to the character as the fade level (see Fade writer). So the level falls when the camera comes closer than 5 units and reaches 0 below 1.7.
Renderer use (SPEC-0046): the renderer uses the hero's fade level as the composite fade level. Below 128 the hero is drawn off-screen and composited onto the screen as one sprite with alpha = level / 128, so the hero fades as a whole silhouette, and its draw record sorts as translucent. At 128 the composite path is off and the hero is drawn and sorted as opaque. Because the ramp clamps the level to 0 to 128, the hero is never in the flagged-but-opaque case (a level above 128). This fade is not material alpha (SPEC-0003): material alpha never enters the draw sort. Confidence: High for the sort rule and the 128 cut-off; Medium-High for the composite sprite and its alpha; Medium for the off-screen buffer sequence.

### Fade writer (confirmed)

The fade writer always stores its flag (field +0x40). It writes the fade level only when the flag is nonzero. The level is the argument truncated toward zero to a whole number, then clamped to the range 0 to 128, and is written to the character record (field +0x44). Confidence: High.

### Scripted camerasA running play has a kind and a run state.
- **Play kinds.
** Kind 1 (play by time): the eye and target follow the track sampled at the play's track time. Kind 3 (target play): the eye follows the sampled track position as in kind 1, and the view is turned to look at a target anchor chosen when the play starts (see Cutscene camera tracks, Target plays). Kind 4 (follow focus): the eye is placed relative to the focus object (its base position plus a vertical offset), used for scripted follow shots. Resolved: no play is ever started with kind 4. The six play calls start kinds 1 and 3, the hotspot call starts the one-pose kind 5, and the only routine that writes kind 4 has no caller in the boot program or in the camera call set, so the clean side leaves kind 4 out **[Medium-High]**.
- **Run states.
** 0 = idle (no play). 1 = playing. 2 = ended.
- **Each update (kinds 1 and 3):** the track is sampled at the play's track time (starting at 0). When the sample is valid it poses the camera and sets the focal length, and then the track time advances by (1/60) x the play's speed factor. If there is no sample, the play moves from playing to ended (state 2) and the end mode acts in the same update. While playing, a hero that is in none of the scripted states (0x21, 0x30, 0x32, 0x33, 0x34, 0x35, 0x37) and not in 0x12, 0x13 or 0x19 is put in state 0x21 and plays its animation 0. When the play is ended, its end mode decides what happens:
- end mode 1: nothing; the play stays ended, holding the last pose, until ReturnToNormalCamera, SetToNormalCamera or another play;  - end mode 0: if the hero is not in a scripted state it is set to state 1 (free); the play becomes idle;  - end mode 2: the hero is set to state 1 whatever its state; the play becomes idle.
- **Track sampling.
** See Cutscene camera tracks below for the sampling rule, the play timing, short tracks, the focal length and the finished query.
- **Starting a play (PlayCamera family).
** All six play calls share one core. Arguments, in this order: track id; for the targeted variants, the target (an animated object index or an AI id); the wait option; the end mode; the speed factor (High):  1. **Track id** (an integer). A value outside the world's track list is ignored: the call does nothing and does not wait.
2. **Wait option.
** When it is nonzero and the track has at least one key, the script yields each tick while the play is neither ended nor idle; the test is made before the first yield.
3. **End mode** (the script's camera-wait value: 0, 1 or 2; see Run states). It is not an entity index.
4. **Speed factor.
** The track time advances by the tick step x the factor per update, so a track of length L seconds lasts L / factor seconds. A factor of 0 freezes the play.  The target depends on the call: PlayCamera has no target (kind 1). PlayTrackHeroCamera and PlayLookAtPlayerCamera target the active hero's anchor and behave identically. PlayTrackAICamera targets the AI character with the given id. PlayLookAtAnimObjCamera targets the animated object's look-at anchor (object offset 0x190) and PlayTrackAnimObjCamera its track anchor (object offset 0x180). With a target the play is kind 3. A missing AI character, or an object index outside the list, gives a play with no target (kind 1).
- **ReturnToNormalCamera(preset, release).
** Stops any running play. Only if the camera is in the blended transition (mode 8), the smoothed follow (mode 9) or the fallback mode (mode 10), it sets mode 0 (normal follow) and runs camera reset from the current base point and pivot (the same rule as the internal return routine). Modes 3, 7, 11 and 12 are left alone: the hand-back does not clear them (all four are dormant). If the preset index is inside the world's locator table (the 80-byte records of the type 11 chunk, SPEC-0028), it hands that locator's position (record +0x30) to the pull-in setter, so the eye goes to the locator's (x, z) at the focus height (see Pose setter with pull-in). These calls never leave mode 12 (see Return to normal). A preset of -1 means no preset. When release is set, the active hero is put in state 1 (free). SetToNormalCamera(int preset, bool release) behaves the same.
- **Return to normal with an angle: SetToNormalCamera(distance, angle, release).
** Stops any play and resets the mode as above; release as for ReturnToNormalCamera. Then places the eye on an orbit around the focus base:
- eye = base + distance x unit(A x cos(theta) + B x sin(theta)),  where base is the hero's position, A is the forward row of the hero's world matrix (character world record +0x2B0) and B is its first row (+0x290), and theta is the angle argument in radians. So theta = 0 puts the eye in front of the hero, theta = pi behind it and theta = pi/2 on its first-row side. The eye is then handed to the pull-in setter, which drops it to the focus height (see Pose setter with pull-in). Level scripts pass distances 5, 10, 15 and 3 most often and angles 0 or about pi; one site passes 180.0, which is used as radians. The cos/sin pairing is High (cos multiplies the forward row). Within |theta| <= pi the exact sine and cosine are used (see Edge cases).  Test vector: hero at the origin with forward row (0, 0, 1) and first row (1, 0, 0), height H. SetToNormalCamera(10.0, pi/2, true): wanted eye (10, 0, 0), placed eye (10, H, 0) when the ground there is not above H + 5, else (5, H, 0). SetToNormalCamera(10.0, pi, true): placed eye (0, H, -10).
- **Other camera calls.
** SetCameraToTrackHeroMode(track) and EndCameraTrackHeroMode() enter and leave mode 12 (see Dead-band yaw follow). SetHeroSceneCamera(a, b) enters mode 8 on the hero with yaw offset a and radius b (see Blended transition); SetAI_DistCamera(id, f, f, f) and SetHeroDistCamera(f, f, f) enter mode 10 on an AI character or the hero; SetAiHotspotDistCamera(id, hotspot, f, f, f) starts a one-key play at an AI hotspot (hotspot index below 9, else 3 units above the AI). No level script calls any of these in play; the one script site of SetHeroSceneCamera sits in a routine of pack 1250 that no call, registration or table names. The boot program itself calls SetHeroSceneCamera(0.9425, 10.0) at the start of the full item showcase (SPEC-0021, Item showcase, Full-showcase task): mode 8 on the selected hero with yaw offset 0.9425 radians, horizontal reach 10.0, height 1.5, so distance 10.112 and elevation 0.1506, all fractions 0.01, pivot step 0, no placement. The showcase ends it with the internal return routine (mode 0 and a camera reset from the pivot), as ReturnToNormalCamera(-1, true) would.
- **Boss fights.
** There is no boss camera mode. The 12 boss modules (those that call BossFightTweakHero) use PlayCamera tracks, ReturnToNormalCamera, SetToNormalCamera, StartCameraShake or StartCameraShakeMode and IsAnimatedCameraActiveAndFinished; between plays the normal follow runs with the floor's profile (boss packs carry floor kind 5 in pack 735 and kind 3 in packs 735 and 1174; the rest use profile 0).
Track sampling (synthetic): 2 keys at interval 1.0 s, positions (0, 0, 0) and (10, 0, 0), quaternions (0, 0, 0, 1) and (0, 0, 0, -1). At t = 0.25 the position is (2.5, 0, 0) and, after the sign flip of the second quaternion, the rotation is (0, 0, 0, 1). At t = 1.0, i = 1 and i + 1 = N, so there is no sample and the play ends. A 30-key track at 1/15 s and speed 1.0 poses the camera on 117 updates and ends on the 118th when the track time is accumulated in single precision (exact arithmetic gives 116 and 117; see Cutscene camera tracks).
Test vectors: PlayCamera(5, 1, 1, 0.5) with track 5 of length 2.0 s: the script waits 4.0 s (240 ticks), then the play stays ended holding its last pose until ReturnToNormalCamera(-1, true), which also frees the hero. PlayCamera(5, 0, 2, 1.0): at the end of the 2.0 s track the hero is set to state 1 and the play goes idle with no further call. SetToNormalCamera(10.0, 3.14159, true): the eye at distance 10 on the far side of the orbit (angle pi), and the hero set to state 1.

### Cutscene camera tracks

Confidence: High for the sampling rule, the play timing, the counts, the short tracks, the focal write and the finished query; Medium-High for the field-of-view reading; Medium for the target-play orientation.
The scene side is SPEC-0052: the type-28 track record layout (head, key count N, key interval dt, focal byte and value, keys), the scene threads that start and wait on plays, the screen effect, and the worked scene of pack 985 with its sampled test vectors (Camera track record, Camera track playback, Sampling a track, Test vectors). This section gives the camera rules only.
**Sampling at track time t (seconds of track time).
**
1. If N < 1, the pose is the head quaternion and the head position, the focal length is the head's +0x2C value, and the sample is valid.
2. Otherwise i = trunc(t / dt). If i + 1 >= N there is no sample. Otherwise f = (t - i x dt) / dt, A = key i and B = key i + 1.
3. Rotation: if dot(qA, qB) < 0, qB is replaced by -qB. Then q = (1 - f) qA + f qB, component by component. There is no slerp, and q is not renormalised.
4. Matrix from q = (x, y, z, w), without normalising:
- row0 = (x² + w² - y² - z², 2(xy - wz), 2(xz + wy), 0)
- row1 = (2(xy + wz), y² + w² - x² - z², 2(yz - wx), 0)
- row2 = (2(xz - wy), 2(yz + wx), z² + w² - x² - y², 0)
- row3 = the position of step 5.   The camera pose is this matrix, with the row meanings of Polar convention (right, up, forward, position).
5. Position: (1 - f) pA + f pB, with w = 1.
6. Focal length: if the head byte +0x2B is nonzero, (1 - f) focalA + f focalB from the two keys; otherwise the head's +0x2C value. The byte is 0 on every track on disc, so the head value is always used.
**Play timing.
**- PlayCamera starts the play with track time t = 0 and run state 1 (playing).
- Each camera update (once per tick) samples the track at t. When the sample is valid, the camera is posed from it, the focal length is written to the render camera, and then t = t + (1/60) x speed, accumulated in single precision.
- When the sample is invalid, the run state becomes 2 (ended), and end mode 0, 1 or 2 acts in that same update (see Scripted cameras, Each update).
- For N of 2 or more, the posed updates are those on which trunc(t / dt) + 1 < N. Nominally there are ceil((N - 1) x dt x 60 / speed) of them; when that product is an exact integer, the float accumulation gives one more.
**Counts (single precision; the same with round-to-nearest and round-toward-zero arithmetic).
**| Track | N | dt | Speed | Posed updates | Ends on update |
| --- | --- | --- | --- | --- | --- |
| Pack 985, track 25 | 30 | 1/15 s | 1.0 | 117 | 118 |
| Pack 985, track 25 | 30 | 1/15 s | 0.5 | 233 | 234 |
| Pack 985, track 25 | 30 | 1/15 s | 2.0 | 59 | 60 |
| Pack 985, track 29 | 61 | 0.065574 s (4.0 s / 61) | 1.0 | 237 | 238 |
With exact arithmetic the first row would be 116 and 117; a clean build that accumulates t in single precision gets the original's 117 and 118.
**Short tracks.
**- 0 keys (static shot): every update returns the head pose as a valid sample, so the play never ends by itself. It stays in run state 1 until ReturnToNormalCamera, SetToNormalCamera or another play. PlayCamera does not wait on it (it waits only when N > 0), and IsAnimatedCameraActiveAndFinished stays false for it.
- 1 key: the first update has no valid sample, so the play ends on its first update without posing the camera. With the wait option the script resumes after one update.
**Focal length (+0x2C).
** The head's +0x2C value is the focal length S of Projection, pushed to the render camera on every posed update; it is not a radius. The normal camera pushes 512. Horizontal field of view = 2 atan(320 / S) and vertical = 2 atan(240 / S) at 640 x 480. The common track value 791.45 gives 44.0 degrees horizontal and 33.7 degrees vertical; the normal 512 gives 64.0 degrees horizontal (50.2 vertical). The other values on disc are listed in SPEC-0052 (Camera track record).
**IsAnimatedCameraActiveAndFinished(track id).
** True only when all of these hold: a play is active (run state not 0); the active play is playing that track id; and, for a plain play (kind 1) or a target play (kind 3), the run state is 2 (ended). For the other play kinds it is true whenever the play is active and is that track. Edge case (reproduce): with end mode 0 or 2 the play is already idle by the time a script can poll, so a poll loop on such a play never ends; the pack 985 scene polls only end-mode-1 plays.
**Target plays (kind 3) (Medium).
** The eye is the sampled track position. The orientation is rebuilt to look at the target's anchor point (the hero, an AI character, or an animated object's look-at or track anchor; see Starting a play). The side axis of the rebuilt basis is flattened to horizontal (its y set to 0) whenever its |x| + |z| exceeds 0.01, so the horizon stays level. The exact axis signs are not confirmed.

### Camera shake (Confidence: High for the record layout, the arithmetic and the argument order; Medium for the angle terms)A shake is one timed record, held by the camera system, that is started by one of two script calls (see Script calls below). A new shake replaces the old one: starting a shake clears the record's counters and then writes the new values.
**Record fields set when a shake starts.
** Parameters: duration T, amplitude A, per-axis scale S, fourth value p4, ramp R, angle phi, and a seventh value Z. The record layout is in Layout (Camera shake record). In short:- The envelope keys are (time, amplitude) = (0, 0), (p4, A), (T - R, A), (T, 0): the amplitude rises over the attack time p4, holds, and falls to 0 over the last R seconds. The wave rate of every key is S.
- Two angle terms are computed from phi: cos(|phi|) and cos(|phi - pi/2|). For phi between 0 and pi the second is sin(phi). They split the movement between the sideways and the vertical direction. The terms are computed by the shared folded sine series of SPEC-0034 section 12, which equals the ideal cosine to about 5e-8 plus the single-precision rounding of the angle (High; Resolved, proactive (open close-out)); exact cos may be used.
- Z is the aim factor (see Per-frame use of the shake).
**Script calls.
**- **StartCameraShake(T, A, S, p4, R, phi, Z).
** All seven values are passed through directly into the record. The camera system supplies the record internally.
- **StartCameraShakeMode(type, duration, strength).
** The first argument selects one of six presets (0 to 5), the second is the duration T in seconds, and the third is the strength (A = 0.3 x strength, the data word 0x2DB2F0). Every preset, including 0, fills the shake record. The preset supplies the other values:
| Preset | S | p4 | R | phi | Z | Notes |
| --- | --- | --- | --- | --- | --- | --- |
| 0 | 12.0 | 0.3 | 0.5 | pi/2 | 0.2 | |
| 1 | 12.0 | 0 | 0 | pi/2 | 0.2 | No ramp (ramp start equals T) |
| 2 | 12.0 | T | 0 | pi/2 | 0.2 | p4 equals the duration; no ramp |
| 3 | 12.0 | 0.3 | 0.5 | 0 | 0.2 | |
| 4 | 12.0 | 0.3 | 0.5 | pi/2 | 0.8 | |
| 5 | 12.0 | 0.3 | 0.5 | 0 | pi/2 | |
The constants are the data words in the Where it lives table (S from 0x2DB2F4, p4 from 0x2DB2F8, R from 0x2DB2FC, phi from 0x2DB300, Z from 0x2DB304). The value 0.8 for preset 4 and the value pi/2 for preset 5 are fixed constants with the same values. Any type outside 0 to 5 starts no shake (High).
**Argument order (High).
** The order is (type, duration, strength). For example StartCameraShakeMode(0, 0.5, 1.0) is a preset-0 shake of 0.5 s with amplitude 0.3 (S 12, p4 0.3, R 0.5, so the ramp starts at 0.0 s); StartCameraShakeMode(1, 5.0, 1.0) is a 5 s shake with amplitude 0.3 and no ramp.
**Per-frame use of the shake (Confidence: High for the wave, the timing, the envelope, the axes and the aim step; Medium for the senses of the rebuilt rows).
** Each frame the camera's matrix (its four rows) is copied into the record, and the shake works on that copy, which is then used as the frame's view; the camera record itself is not moved. The step is a fixed dt = 1/60 s (not wall-clock time). When the finished flag is set nothing else happens. Otherwise, each frame:1. The elapsed time e grows by dt.
2. Key search: starting from the current key index j, while j + 1 < 6 and the time of key j + 1 is at most e, j becomes j + 1.
3. Envelope: if j + 1 < 6, and both e - time(j) and time(j + 1)
- time(j) are above 0, the current amplitude a and the current rate are linear blends of keys j and j + 1 with the fraction (e - time(j)) / (time(j + 1)
- time(j)). Otherwise a and the rate keep their previous values.
4. End test: if e is at most 0 or e is at least T, the finished flag is set. That frame still applies its movement; later frames leave the view unchanged.
5. The phase grows by dt x rate (12.0 for presets 0 to 5). The wave is w = sin(2 pi x frac(phase)), where frac is the fractional part.
6. Movement: let h be the frame's first row with its y set to 0, normalised. The position row moves by a w cos(|phi|) along h and by a w cos(|phi - pi/2|) along the world up (0, 1, 0). So phi = pi/2 shakes up and down only, and phi = 0 shakes sideways only.
7. Aim (only when Z > 0): the aim point is the unshaken position plus the unshaken forward row times L, with L = 0.5 A + 100 (1 - Z) (A the start amplitude). The forward row becomes the unit vector from the moved position to the aim point; the up row becomes normalise(first row x forward), and the first row becomes forward x up. So the view turns slightly to keep the aim point in place: with Z = 0.2 the aim point is about 80 units ahead, with Z = 0.8 about 20 units ahead (a stronger turn).
With S = 12, one wave cycle takes 5 frames (0.0833 s).
**Every shake runs from the start (High).
** A start clears the elapsed time, the key index, the wave phase, the finished flag and the whole key list, and then fills keys 0 to 3, so a second or later shake plays exactly like the first one. A start while another shake is still running replaces it at once, and the new shake begins at elapsed time 0.
**Envelope examples.
** StartCameraShakeMode(0, 1.0, 1.0): keys (0, 0), (0.3, 0.3), (0.5, 0.3), (1.0, 0): rise over 0.3 s, hold to 0.5 s, fall to 0 at 1.0 s. StartCameraShakeMode(0, 0.5, 1.0): keys (0, 0), (0.3, 0.3), (0, 0.3), (0.5, 0); the third key time is earlier than the second, so the search passes it at 0.3 s and the amplitude drops at once from 0.3 to 0.3 x (1 - 0.3 / 0.5) = 0.12, then falls to 0 at 0.5 s. Preset 1 (p4 = 0, R = 0): full amplitude from the first frame to the end. Preset 2 (p4 = T, R = 0): the amplitude rises over the whole shake and stops at A.
Level-script usage: StartCameraShake (seven values) is called in 3 modules, once each. StartCameraShakeMode is called at 20 sites in 13 modules: type 0 at 16, type 1 at 2, type 2 at 2. Most common: (0, 0.5, 1.0) four times, and (0, 1.0, 1.0), (0, 1.0, 3.0) and (1, 5.0, 1.0) twice each (full list in SPEC-0030 row 36). 15 distinct modules call one or both shake calls.

### Screen record, quad setup and clip rectangle

These items belong to the screen or canvas object, not to the camera. SPEC-0023, Canvas mapping, describes the known quad and clip-rectangle setup; the remaining details are retained in Open questions 42 and 43.
Screen size (High). The screen record is filled once at boot by the display set-up of the boot display object (run-time data 0x368F00). Its display-mode word is taken from the boot settings record, whose mode entry is -1 (automatic) on this disc. Automatic picks mode 1 only when the video-standard query returns 2, and that query always returns 0 in this build, so the mode is 0 and the record is always 512 by 448. No other code writes the mode word. The 512 by 512 variant (mode 1) never appears. Test vector: settings mode -1, query 0: mode 0, width 512, height 448; a settings mode of 1 would give 512 by 512 (not on this disc).

### Level-script camera usage (counts)

Counts of host calls in the level scripts. "Mentions" are occurrences, not unique calls. Confidence: High for the counts.
- PlayCamera: 2,301 mentions in 63 modules.
- ReturnToNormalCamera: 887 mentions in 60 modules; of these, 833 pass preset -1 with notify set.
- SetToNormalCamera: 94 modules.
- IsAnimatedCameraActiveAndFinished: 3 modules.
- StartCameraShake (seven values): 3 modules. StartCameraShakeMode: 13 modules, types 0, 1 and 2.
- PlayLookAtAnimObjCamera: 26 sites in 4 modules; PlayTrackHeroCamera: 7 in 6; PlayLookAtPlayerCamera: 5 in 4; PlayTrackAICamera: 1 in module 864; SetHeroSceneCamera: 1 in module 1250. PlayTrackAnimObjCamera, SetCameraToTrackHeroMode, SetAI_DistCamera, SetHeroDistCamera, SetAiHotspotDistCamera, OverRideCameraAttributes and RestoreCameraAttributes: none.
- SetToNormalCamera(distance, angle, release): 132 sites in 99 modules; SetToNormalCamera(preset, release): 1 site.
- PlayCamera argument patterns (wait option, end mode, speed factor): (0, 1, 1.0) 1,772 times; (1, 1, 1.0) 294 times; (1, 2, 1.0) 66 times; (0, 1, 0.3333) 58 times.

### Camera volumes (none; Confidence: High)

The camera has no volume records. The kind-byte test over level primitives is the camera's own wall push over the collision primitives of the eye's grid cell (kinds 1 plane, 3 cylinder, 4 and 7 box push or zero-radius contact by the sign of +0x2C, 2 cone skipped; see Walls and the stuck timers). The distance blend with the thresholds 50.0 (0x2DB4CC) and 25.0 (0x2DB4D0) is the type 41 atmosphere zone pass of SPEC-0028 section 11, which takes the camera eye as its position but does not change the camera.

### Zone record blend (the former pose blend helper; Confidence: High)

This helper is not a camera routine. It is the record blend of the type 41 atmosphere zones (SPEC-0028 section 11), and the zone pass is its only caller (two calls). It takes a blend value t and two 0x50-byte zone records X and Y, and writes a target record (both calls pass X as the target):- t at or below 0: the whole 0x50-byte record X is copied to the target.
- 0 < t < 1: only three fields are blended, each as (1 - t) x X + t x Y: vector A (+0x00, four lanes), vector B (+0x10, four lanes) and the render scalar (+0x20). No lane is normalised. Every other field of the target (from +0x24 on: the gate key, the circle centre, the point count and list, and the bytes at +0x48 to +0x4F) is left as it was, so with X as the target they keep X's values.
- t at or above 1: the whole record Y is copied to the target.
It returns 1 - t when 0 < t < 1 and a meaningless value otherwise; both calls ignore it. The interpolated type-41 scalar is sent to the renderer as the view-screen depth-range maximum. When accepted by the renderer's limit, it changes the depth mapping and rebuilds the view-screen matrices; it does not change the camera eye or the world-space near and far clip distances. A modern renderer that preserves the original mapping must apply the scalar to its projection/depth conversion. It is not an opacity or fade input.
The two calls: (1) the zone blend, t = 1 - (d - 1) / (50 - 1) for a zone at distance d from the eye, with X the working record and Y the zone; (2) the world-change crossfade, t = w = 1 - d / 50, with X the working record and Y the record kept at the world change (SPEC-0028 section 11).

### Per-frame anchor scan (see SPEC-0028; Medium-High)

The per-frame update with the 50.0 and 25.0 thresholds is the type 41 atmosphere zone pass, specified in SPEC-0028 section 11. It runs after the world update and uses this tick's camera eye as its position. Its "anchor list" at world +0x5844 is the world's type 41 table; the stored target id (+0x510) and pose (+0x520) are the stored world and the eye position at the last world change, and the blend-active flag (+0x580) marks the world-change crossfade (w = 1 - d / 50) (Medium for these three fields). The "sweep" is the distance from the eye to a polygon zone, and the "fade fraction" (distance / 25) is the gated-zone blend term that becomes the alpha of the second sky-dome model. The camera does not read the result.

### Look-at play start toward the hero (Medium-High; Medium for the row senses)

This is one of the scripted camera play starts (it also runs the front-end HUD clear, like every play start). Its record's first word is the camera object. Inputs: the eye point E given to the start (stored at play record +0x80) and the active hero.
1. Target T = the hero's position + (0, character height, 0), the height from the hero's focus record (+0x16C, +0xC), the same height the follow focus uses.
2. Forward F = normalise(T - E).
3. A horizontal reference R = (-F.z, 0, F.x), normalised (F turned a quarter turn about the vertical). The up vector block at 0x3932E0 is not used here.
4. V = normalise(F x R), then W = normalise(F x V).
5. The camera's rows setter (the camera class entry that the restart placement also uses) is called with the position E, first row W, up row V and forward row F. The camera is then at E, looking at T.
Test vector (synthetic): E (0, 5, -10), hero at the origin with height 2: T (0, 2, 0), F (0, -0.287348, 0.957826), R (-1, 0, 0), up row V (0, -0.957826, -0.287348), first row W (1, 0, 0). For a level forward (0, 0, 1) the rows are first (1, 0, 0) and up (0, -1, 0); the sign of the up row follows the rows setter's convention and is Medium.

### View object reset (Medium)

Resetting the 0x160-byte view object:- Sets the words at +0x0C, +0x10, +0x78, +0xA4, +0xA8, +0xAC, +0xB0 and +0xB4 to -1 (0xFFFFFFFF). These are the "unset" sentinels; a reader must treat them as absent.
- Sets the orientation words at +0x50 and +0x60 to the identity rotation (0, 0, 0, 1).
- Zeroes the other fields, including +0x14 to +0x28, +0x34, +0x38, +0x3C, +0x40, +0x70, +0x74, +0x7C, +0x80, +0x84 to +0x94, +0xC4, +0xCC to +0xDC, +0xF4, +0xF8, +0x11C, +0x144, +0x150, +0x154 and +0x158.
- Clears the small arrays at +0x100, +0x108 and +0x110.
Confidence: High for the writes. Medium for the meaning of the sentinels (they have no known name).

### Follow speed offset: not a camera routine (Confidence: High)

This update does not belong to the camera. It is the class-2 surface test of SPEC-0018 (Second dispatcher, class-2 surface record): its first argument is a 0x90-byte world surface record (from the world record's +0x57F8 table), not the camera record, and its second argument is a character's motion record. Its "probe" is the object-direction footprint variant run against the surface; its "0x20-byte block" is the player class predicate asked about code 0x20 (it sets the surface's latch +0x84); the "speed" is k = |P . n| over four lanes, with P the body position (+0x50) and n the surface normal (+0x40); h is the body's vertical displacement (+0x64); setter A adds to the body's vertical velocity (+0x74) and setter B sets it; the two zero setters clear the x and z of the body's displacement (+0x60, +0x68). Its three callers are character collision routines (the box push, the cylinder routine and the contact walk). The rules, the latch and the test vectors are in SPEC-0018; no camera field is read or written.

## Edge cases and error handling- World change far from the old eye: when the eye is more than 50.0 units from where it was at the world change, the new world's zones are used at once with no crossfade (SPEC-0028 section 11; Medium-High).
- Crossfade at zero movement: d = 0 gives w = 1.0, so the previous output is kept in full (SPEC-0028 section 11).
- No base zone: an all-zero default record is used (SPEC-0028 section 11).
- Camera lag: the camera works from the hero's pose of the previous tick (Update order). A clean side that updates the camera after the hero shows one tick less lag than the original.
- Sentinel values: a -1 in the reset fields means "unset"; a caller that reads one as a number gets -1 (Medium).
- **Angle-wrap bug (reproduce it).
** Any angle above 2 pi is reduced by pi, not by 2 pi. The result is half a turn off from the correct value. Example: x = 7.0 gives 3.858407, not 0.716815. Only one subtraction is applied per call. In the normal follow the wrap is applied to the spun yaw (follow step 6); that yaw is an arctangent (at most pi) plus a spin of at most 0.052 rad, so the above-2-pi branch is not reached there. Confidence: High for the constant; Medium that it is reached in play.
- **Transition yaw takes the long way round** (reproduce it, Medium). When the current and target yaw have opposite signs, the sign test always shifts the target by a full turn. If the two yaws are close across zero (their sizes add to less than pi), the camera turns the long way round. Example: current 0.5, target -0.5, k = 0.5 gives about 3.141593 after one update, where the short way would give 0.
- **Transition distance snap.
** The distance snaps to its target within 0.1 units, not within 1e-6. Yaw and elevation snap within 1e-6.
- **Drift threshold jump (reproduce it, High for the rule).
** The drift D is not clamped to the range -0.99 to +0.99.
Any value above 0.99 becomes exactly +1.3, and any value below -0.99 becomes exactly -1.3.
So a squared sum just above 0.99 jumps from about 0.99 to 1.3.
With no leans only pad bytes 0 and 255 reach the jump (byte 254 gives 0.955); see Reachable drift values. The jump is the shipped rule and is reproduced as stated (Open question 9).
- **Drift base at the extremes.
** Pad byte 255 gives converter A the value 87.5, so B = -1 and the base is -1. Pad byte 0 gives the base +1. Pad byte 127 or 128 gives a base of 0 (inside the dead band).
- **Byte 0x127 is not a spin input.
** It is the stick value only. Its effect is on the elevation (the chase target and the manual step).
- **Side target sign.
** The side target uses the magnitude of the sideways displacement, so it is never negative. The side offset therefore eases in one direction only.
- **Mode 8 pivot step (not a defect).
** The pivot step is compared with the remaining distance to the target point. With step 0 (both callers) the pivot never moves, so the death camera keeps framing the death spot.
- **Contact freezes modes 8, 9 and 10.
** Once the contact pass on the camera position pushes it, the blend or follow step of that mode is skipped until the mode is re-entered (or return-to-normal runs). The camera holds its last pose during that time (see Blocked latch). So a death camera that swings into a wall stops there.
- **Manual cap snap.
** The manual elevation is absolute and limited to +/-1.5 rad; a value beyond the limit is cut to the limit on the next push tick. Outside manual mode the elevation target is at most the profile elevation + 1 + bias (1.2 rad with profile 0), so a large snap does not occur in normal play; with profile 3 a full push can reach 1.6 rad, and the next manual tick cuts it to 1.5 (Medium; see Chase and manual elevation).
- **Manual cap overshoot.
** The manual elevation can exceed its cap by one step once; the next tick sets it back to the cap (see Chase and manual elevation). Whether this is intended is open.
- **Setter mapping.
** The mode setter maps 5, 6, 8, 9 and 10 to mode 2 (no action), so these values do not run their handlers when set through the setter. Modes 8, 9 and 10 are reached only through their own entries. In the shipped game the setter is only ever called with 0, 12 and 3 (see Mode entry points in the shipped game).
- **Recentre shrink floor.
** The shrink stops at radius 1.
0. If the segment is still blocked there, the eye may sit at a blocked position.
- **Visibility range cut.
** Segments of 50 units or longer never report an obstruction.
- **Orbit sine/cosine approximation.
** The orbit, and the camera-shake angle terms, use the same folded sine series as SPEC-0034 section 12 (same five stored constants, same phase reduction that floors on the console's truncating adder; Resolved, proactive (open close-out), High). It matches the ideal sine and cosine at every angle: the series itself is within about 5e-8, and the single-precision rounding of the angle adds up to about 2e-6 for |theta| up to 20. There is no region of large error: cos(3.5) = -0.936457, cos(4.0) = -0.653644, cos(5.0) = 0.283662, and at theta = -pi/2 the sine is -1.000000. An earlier reading with errors of 0.16 near 3.5 rad and a value of -0.604 at -pi/2 assumed round-to-nearest in the reduction and is wrong.
0. Resolved (High): the two terms evaluate the same cosine polynomial at |theta| and at |theta - pi/2|. The polynomial is a cosine, an even function, so taking the absolute value mirrors nothing: the first term is cos(theta) and the second is cos(theta - pi/2) = sin(theta) for negative angles as for positive ones. Use the polynomial as described, with the absolute values.
- **Zoom ramp thresholds.
** v is the eye-to-focus distance, so the thresholds 1.7 and 5.0 are reached whenever the camera comes close (a wall push, a recentre shrink or the nearly overhead elevation at 3.53 units). High.
- **Missing play or preset.
** Out-of-range play or preset indices are ignored; nothing changes. For the PlayCamera family the track id is the index that is ignored (no play and no wait); a missing target object or AI character gives a play with no target (see Scripted cameras).
- **Mode 7 with a byte just outside the dead band.
** Converter C has no dead-zone subtraction, so a byte at 30.5 counts from centre moves the camera by about 2.39 units per tick, not by a small amount (mode 7 is dormant).
- **Mode 3 height term.
** The shoulder-button height term is gated by camera field 0x68, which is zeroed at construction and never set, so the term never applies. When enabled, its value is a plain float added to the height (no rounding). Mode 3 itself is dormant.
- **Shake preset 0 is a normal shake (High).
** It fills the shake record like presets 1 to 5.
- **Shake with out-of-range preset.
** A third argument other than 0 to 5 starts no shake.
- **Shake ramp with R equal to 0.
** Presets 1 and 2 have no ramp; the ramp start equals T, so the ramp runs over no time.
- **Camera reset and the drift value.
** Reset does not write the drift value D. D is recomputed every tick, so it does not persist past the next tick.
- **Fade flag zero.
** A fade write with flag zero stores the flag and leaves the fade level as it was.
- **Stick at neutral.
** A zero stick leaves the manual elevation unchanged and the chase target equal to the profile default elevation plus the heading bias.
- **Speed factor 0.
** A play started with speed factor 0 never advances; with end mode 1 it holds its first pose until a return-to-normal call or another play.
- **Type 6 frame records.
** The eye pass skips them, while the hero's wall contact treats them as two-sided walls (SPEC-0018). The eye can therefore pass through a type 6 wall that blocks the hero. Reproduce this; do not give the camera the hero's test.

## Test vectors

Projection (synthetic, from the formulas above):- S = 512, W = 640: a = 1.6; H = 480: b = 2.13333; half-tangents 0.625 and 0.46875.
- S = 512, W = 800, H = 450: half-tangent x 0.78125 (76.0 degrees), half-tangent y 0.43945 (47.45 degrees).
- P = 1000, N = 0.5: c = 1.001001, d = -1.0005.
- Viewport (0, 0, 640, 480): GS offsets 27648 and 28928 (sign per the Low item above).
Pad-axis converters (computed from the converter rules in Pad-axis converters; all values are exact to the digits shown):- Converter A (centre 127.5, dead zone 40): byte 0 gives -87.5; byte 87 gives -0.5; byte 88 gives 0; byte 127 gives 0; byte 160 gives 0; byte 167 gives 0; byte 168 gives +0.5; byte 180 gives +12.5; byte 255 gives +87.
5. Divided by 87.5: byte 0 gives -1.0 and byte 255 gives +1.0.
- Converter B (centre 128, dead zone 32, factor 1): byte 0 gives -0.75; bytes 96, 97, 128 and 160 give 0; byte 161 gives 1/128 = 0.0078125; byte 255 gives 95/128 = 0.7421875. With factor -34.1333 (axes 0x127 and 0x12B), byte 255 gives about -25.333 and byte 0 gives about +25.6.
- Converter C (centre 127.5, gate 30): byte 0 gives +10.0 (t = 127.5); byte 97 gives +2.392157 (t = 30.5); byte 98 gives 0 (t = 29.5); byte 157 gives 0; byte 158 gives -2.392157 (t = -30.5); byte 255 gives -10.0.
Stick value (synthetic): byte 255 at 0x127 gives converter A 87.5, so the stick value is -1.0 (elevation input); byte 0 at 0x127 gives +1.0; byte 127 gives 0.
Yaw drift base (synthetic, from the rule in "Stick value and yaw drift"):- Byte 255 at 0x12B: converter A gives 87.5, B = -1.0, base = -1.0.
- Byte 0 at 0x12B: converter A gives -87.5, B = +1.0, base = +1.0.
- Byte 127 at 0x12B: converter A gives 0, base 0.
- Byte 207 at 0x12B: converter A gives 39.5, B = -0.451429, base = -0.203789.
- Byte 215 at 0x12B: converter A gives 47.5, B = -0.542857, base = -0.294694 (D is the same, since there are no leans; below the 0.99 threshold).
Yaw drift switch (synthetic, from the rule in the drift section; S is the sum of the base and the three leans):- S = 0.5: D = 0.25 (unchanged, |D| is below 0.99).
- S = 0.8: D = 0.64 (unchanged).
- S = -0.9: D = -0.81 (unchanged, above -0.99).
- S = 0.98: D = 0.9604 (unchanged).
- S = 0.995: D = 0.990025, which is above 0.99, so D = +1.3 (the jump).
- S = 1.2: D = 1.44, above 0.99, so D = +1.3.
- S = -1.2: D = -1.44, below -0.99, so D = -1.3.
- Base term example: v such that -v / 87.5 = 0.5 gives base 0.25 (signed square of 0.5), with no leans, so S = 0.25 and D = 0.0625.
Spin integration (synthetic factors): D = 1.3, a = 0.1, b = 0.05, I = 0. After one tick I = 1.3 x 0.1 - 0 = 0.13 and the yaw has turned by 0.
13. After a second tick with D still 1.3: I = 0.13 + 1.3 x 0.1 - 0.13 x 0.05 = 0.2535, and the yaw has turned by 0.3835 in all. With the profile factors (a 0.005, b 0.125) see Spin.
Chase and manual (see Chase and manual elevation for the vectors above).
Negation (synthetic): with a profile elevation of 1.0 and stick 0.5, normal play gives target 1.
5. With the hero in state 21 and climb sub-mode 4, the target is -1.5; in state 21 with sub-mode 2 it stays 1.5.
Recentring gate exception (High): Roberta (+0xF80 = 1) in state 13 with attack sub-state 10, having moved last step: T = 0. Tor (+0xF80 = 0) in state 13 with sub-state 10, having moved: T = 1. Roberta in state 13 with sub-state 5 (near-target swat), having moved: T = 1. Any hero in state 1 (idle): T = 0.
Recentring gate (synthetic): the body has not moved, flag 0: the chase runs and the flag stays 0 with a zero stick. Stick +0.5 while the body is still: the chase step sets the flag to 1, and from the next tick the manual step runs and the camera holds. The hero then moves (T = 1): the flag is cleared and the chase runs again.
Hold timing (synthetic): see Hold and lean terms.
Dead-band yaw (mode 12, synthetic; d is the target minus current yaw):- d = 0.05: yaw unchanged.
- d = 0.15: yaw changes by 0.15 x 10 x (0.15 - 0.1) = 0.075.
- d = 0.3: yaw changes by 0.3 (snap to the target).
Fade writer (synthetic):- Argument 200.7 with flag 1: level 128. Argument 64.9 with flag 1: level 64. Argument -5 with flag 1: level 0. Argument 90 with flag 0: flag stored as 0, level unchanged.
Camera-near fade and renderer (synthetic): eye-to-focus distance v = 3.35 gives target level (3.35 - 1.7) / 3.3 x 128 = 64. With current level 58, the next update reaches 63; the following update snaps to 64 because it is within 5. The composite sprite alpha at level 64 is 64 / 128 = 0.5, and the character sorts translucent. At v = 5.0 the target is 128; at v = 1.7 it is 0.
Target distance (synthetic): profile distance 2.0 and side offset 0.5, elevation not positive: 2.5 (no 0.65 term). Elevation 0.5: 2.0 x (1 - 0.65 x sin^2(0.5)) + 0.5 = 2.0 x 0.850598 + 0.5 = 2.201196. Profile values: see Target distance.
Mode setter (synthetic): set 5 gives mode 2; set 8 gives mode 2; set 11 gives mode 11 (runs the mode 3 body); set 12 gives mode 12; set 7 gives mode 7 and raises the camera to at least 100 units above the ground (see Mode 7).
Blended transition (mode 8), entry values (synthetic, computed from the formulas):- Radius 15.0, height 1.5: distance = sqrt(15.0^2 + 1.5^2) = about 15.0748; elevation = arcsine(1.5 / 15.0) = arcsine(0.1) = about 0.10017 rad (5.74 degrees).
Blended transition (mode 8), synthetic update rules (unchanged formulas):- Distance from 4.0 toward 2.0 with k = 0.5: the remaining difference halves each update: 3.0, 2.5, 2.25, 2.125, then 2.0625 on update 5. That value is within 0.1 of 2.0, so the distance is set to exactly 2.0 on update 5.
- Distance from 4.0 toward 2.0 with k = 0.01 (the mode 8 entry fraction): after update 298 the value is 2.100073; on update 299 the value is 2.099073, within 0.1, so the distance is set to exactly 2.0.
- Distance from 15.0748 toward 10.0 with k = 0.01 (the mode 8 entry values): after 100 updates the remaining difference is about 1.857 (0.99^100 x 5.0748). It first falls within 0.1 at update 391 (remaining about 0.0997), and the distance is then set to exactly 10.0.
- Yaw from 2.0 toward 1.0 with k = 0.5 (same sign, no wrap): the remaining difference after update 19 is about 1.9 x 10^-6 (value about 1.0000019, not snapped). After update 20 it is about 9.5 x 10^-7, which is below 1e-6, so the yaw is set to exactly 1.0.
- Yaw shortest arc: current 3.0, target -3.0, k = 0.
5. The signs are opposite, so the target is shifted by +2 pi to about 3.283185. One update gives 3.0 + 0.5 x 0.283185 = 3.141593 (pi). This is also the short way round (the gap is 0.283 rad through pi).
- Yaw sign-test quirk: current 0.5, target -0.5, k = 0.
5. The target is shifted to about 5.783185, and one update gives 0.25 + 2.891593 = 3.141593 (pi). The short way would give 0.
0. Reproduce this quirk.
- Mode 8 pivot step: step 0 and remaining distance 3.0: the pivot stays. Step 0.5 and remaining distance 3.0: the pivot moves 0.5 toward the target point. Step 2.0 and remaining distance 1.5: the pivot is set to the target point.
- Death camera: see Death camera for the tick vector.
Blocked latch (synthetic): the contact pass pushes the camera position on the first update of mode 8: the pushed position is kept, the latch becomes nonzero, the blend is skipped, and the camera holds its pose on every later tick until the mode is entered again.
Projection depth rows of the shipped planes (N = 1): P = 5280 gives c = 1.000379, d = -2.000379; P = 600 gives c = 1.003339, d = -2.003339; P = 10560 gives c = 1.000189, d = -2.000189.
Angle wrap:- x = 7.0: returns 3.858407.
- x = -1.0: returns 5.283185.
- x = 3.0: returns 3.0.
Camera shake per frame (S = 12, dt = 1/60): the wave w after frames 1 to 5 is 0.951, 0.588, -0.588, -0.951 and 0 (one cycle every 5 frames). The phases are 0.2, 0.4, 0.6, 0.8 and 0.0 (fractional part).
Camera shake (synthetic; the presets use the record rules above; A = 0.3 x strength):- StartCameraShake(T = 2.0, A = 0.15, S = 12.0, p4 = 0.3, R = 0.5, phi = 0, Z = 0.2): the record gets duration 2.0, ramp start 1.5, amplitude 0.15, five scale copies of 12.0, and the angle terms cos(0) = 1.0 and cos(pi/2) = 0.0.
- StartCameraShakeMode(type 3, duration 2.0, strength 0.5): the same record values as the line above (preset 3 uses phi = 0 and Z = 0.2).
- Preset 4 with duration 2.0 and strength 0.5: Z = 0.8, phi = pi/2, so the angle terms are cos(pi/2) = 0.0 and cos(0) = 1.0; ramp start 1.5.
- Preset 2 with duration 2.0 and strength 0.5: p4 = 2.0 (the duration), R = 0, ramp start 2.0.
- Type 6: nothing starts.
- StartCameraShakeMode(0, 0.5, 1.0): duration 0.5 s, amplitude 0.3, ramp start 0.0 s.
Angle terms (the folded series of SPEC-0034 section 12 against the ideal cosine; the difference is below 1e-6 for phi up to 2):- phi = 0.5: term 1 = 0.877583 (cos 0.5), term 2 = 0.479426 (sin 0.5).
- phi = 1.0: term 1 = 0.540302, term 2 = 0.841471.
- phi = 2.0: term 1 = -0.416147, term 2 = 0.909297.
Scripted:- Orbit with distance 10, base (0, 0, 0), A = (1, 0, 0), B = (0, 0, 1): theta = 0 gives (10, 0, 0); theta = pi/2 gives (0, 0, 10).
Zone record blend (synthetic): X with vector A (0, 0, 0, 0), vector B (10, 10, 10, 0), render scalar 100 and gate key 3; Y with A (4, 0, 0, 0), B (30, 10, 10, 28), scalar 200 and gate key 7. t = 0.25 gives A (1, 0, 0, 0), B (15, 10, 10, 7), scalar 125 and gate key 3 (kept from the target), and returns 0.
75. t = 0 gives X unchanged; t = 1 gives a full copy of Y, gate key 7 included.
Eye pass failure (synthetic): a per-primitive routine fails on 4 consecutive ticks with a single contact each tick. The counter reads 1, 2 and 3 after ticks 1 to 3; on tick 4 it reaches 4 > 3 and the recovery pass runs. The eye is not put back on any of these ticks (count below 2, latch 0). With two contacts on tick 1, the eye is put back on tick 1 and the latch keeps it put back on ticks 2 and 3.
Wall push routing (synthetic): a kind-4 primitive with +0x2C = 0 gets the zero-radius contact; with +0x2C = 2.0 it gets the box push with eye radius 1.25; a kind-2 primitive is skipped; a frame record of type 6 is skipped (the hero's wall contact of SPEC-0018 does test it). A wall frame record 1.0 in front of the eye (d = 1.0, inside its lateral and vertical bounds) gives a depth of 0.25 with the 1.25 margin.
World-change crossfade (synthetic; SPEC-0028 section 11): the eye is 10 units from its position at the world change, so d = 10 and w = 1 - 10/50 = 0.
8. A distance of 25 gives w = 0.
5. A distance of 60 is above 50, so the crossfade ends at once.
Camera shake (any shake, counted from its start): StartCameraShakeMode(0, 1.0, 1.0), so A = 0.3, S = 12, keys (0, 0), (0.3, 0.3), (0.5, 0.3), (1.0, 0), phi = pi/2 (vertical only), Z = 0.2 (aim distance 80.15). Frame 9 (e = 0.15): amplitude 0.15, phase 1.8, w = -0.951057, the view moves down by 0.142659 and turns up toward the aim point by about 0.00178 rad. Frame 24 (e = 0.4): amplitude 0.3, w = -0.951057, down 0.285317. Frame 42 (e = 0.7): amplitude 0.18, w = 0.587785, up 0.105801. The shake ends on frame 60 or 61 (single-precision sums of 1/60). A second StartCameraShakeMode(0, 1.0, 1.0) later in the session moves nothing and ends on its first frame. Values to about 5 decimals.

### Test vectors: one real follow tick (new game, first level pack)

Values are in single precision, so positions near 1,858 are good to about 0.001 and the small terms below are shown to more places. The wall pass is taken as no contact (Medium): the nearest collision sphere surface of the eye's grid cell is 17.2 units from the eye, the nearest box 28.8 units and the nearest wall-panel start point 18.5 units. Idle sticks and a hero in state 1 (idle) are assumed (Medium).

Setup (High for the placement, Medium-High for the ground heights).
- A new game puts the hero at the first record of the pack's entry list: x -1852.802, z 1137.463, facing (0.59588, 0, -0.80307), and then runs the restart placement. The ground probe at that point with ceiling 58.0 meets patch 139 of the pack's model set (the other patches in the cell give no sample or fail the outline test), height 40.4989, so the hero stands at y 40.4989. The patch's kind word is 0, so the first tick selects profile 0: distance 10.0, default elevation 0.2, k 0.085, c 0.583095, spin gain a 0.005, decay b 0.125.
- The heroes' body height is 3.5, so the focus is F = (-1852.802, 43.9989, 1137.463).
- Restart placement: eye = P - 10 f + 2.5 up = (-1858.761, 42.9989, 1145.493). The pull-in setter keeps x and z and sets the eye height to the focus height 43.9989. The ground there, probed from F.y + 5 = 48.9989, is 39.3613 (patch 139; the other candidates fail the outline test), which is not above 48.9989, so the offset is not halved. Camera reset then sets the current and target polar to (10.0000, 2.20916, 0) (yaw 126.58 degrees), the elevation target and manual elevation to 0, the spin value I to 0 and the spring velocity to zero.

State before tick 1: eye (-1858.761, 43.9989, 1145.493); focus (-1852.802, 43.9989, 1137.463); C = P = (10.0000, 2.20916, 0); v = (0, 0, 0); I = 0; t_e = m = 0; manual flag 0; profile index -1 (nothing loaded yet).

Tick 1, step by step (the hero has not moved, so it is the pose of the placement):
1. No scripted play.
2. Profile selection: ground kind 0 gives profile 0 (values above).
3. Sticks centred (bytes 127 or 128 are inside converter A's dead zone): stick value s = 0, drift base 0, D = 0.
4. Target distance: the stored elevation lane is 0, which is not above 0, so the 0.65 term is dropped and P.r = 10.0 (side offset 0).
5. Focus: F as above (bias 0).
6. Spin: I = 0 + 0 x 0.005 - 0 x 0.125 = 0. The eye's polar about the new focus is (10.0000, 2.20916, 0) and its yaw stays 2.20916.
7. Elevation: the recentring gate T is 0 (the hero is idle and has not moved) and the manual flag is 0, so the chase step runs: target = 0.2 + 0 + 0 = 0.2, step 0.01 (|s| is not above 0.99), so t_e = 0.01, P.e = 0.01 and m = 0.01. The manual flag stays 0 (s is 0).
8. Eye on the sphere: E = F + sphere(10.0000, 2.20916, 0) = (-1858.761, 43.9989, 1145.493), unchanged because the yaw did not spin.
9. Hero state 1 is not camera-locked, so P.yaw = atan2(E.z - F.z, E.x - F.x) = 2.20916. P = (10.0, 2.20916, 0.01).
10. Spring: T = F + sphere(P) = (-1858.7607, 44.0989, 1145.4931). E - T = (-0.0003, -0.0999983, 0.0004). v = 0 - 0.085 x (E - T) - 0.583095 x 0 = (0.00003, 0.0085, -0.00003). E = E + v = (-1858.7609, 44.0074, 1145.4934). (The hero's speed is below 0.01 and no contact normal is remembered, so the eye is free to move.)
11. Ground and walls: the ground at the eye, probed from the eye height + 2.5 = 46.5, is 39.3613, so the minimum eye height h + 2.0 = 41.3613 is below the eye and nothing changes. No wall push (see above), no R3 press. The previous eye is set to E and the camera looks from E at F.
12. Fade ramp: the eye-to-focus distance is 10.0, which is above 5.0, so the target level is 128; the level stays 128.

State after tick 1: eye (-1858.761, 44.0074, 1145.493); polar of eye about focus (9.99996, 2.20916, 0.00085); v = (0.00003, 0.0085, -0.00003); I = 0; t_e = m = 0.01.

Ticks 2 and 3 with the same idle input (the elevation lane read by step 4 is the one stored by the tick before): tick 2 has P.r = 9.999995, t_e = 0.02, v.y = 0.01982 and ends with the eye at (-1858.761, 44.0272, 1145.493), elevation 0.00283; tick 3 has P.r = 9.99995, t_e = 0.03, v.y = 0.03135 and ends at (-1858.761, 44.0585, 1145.493), elevation 0.00597. The camera rises about 0.0085, 0.0198 and 0.0313 per tick while the elevation target climbs 0.01 per tick toward 0.2.

Same tick with the right stick fully right (byte 255 at the right-stick X byte, left stick and right-stick Y centred). Differences from the idle tick:
- Step 3: converter A gives 87.5, B = -1.0, base = -1.0 x 1.0 = -1.0, D = -1.0 x 1.0 = -1.0, which is below -0.99, so D = -1.3.
- Step 6: I = 0 + (-1.3) x 0.005 - 0 = -0.0065. The eye's polar yaw becomes 2.20916 - 0.0065 = 2.20266.
- Step 8: E = F + sphere(10.0000, 2.20266, 0) = (-1858.709, 43.9989, 1145.532), a horizontal move of 0.065.
- Step 9: P.yaw = 2.20266 (the eye's own yaw).
- Step 10: T = (-1858.7083, 44.0989, 1145.5316); v = (0.00003, 0.0085, -0.00003); E = (-1858.709, 44.0074, 1145.532).
- After ticks 2 and 3, I = -0.012188 and -0.017164, and the eye is at (-1858.610, 44.0272, 1145.603) and (-1858.469, 44.0585, 1145.701).

## Open questions

1. Resolved: **Dormant eye-lag and rig path.
** The known rig body performs its own segment and contact checks; it does not repeat the follow camera's vertical ground probe or pose pull-in setter. The eye-lag stage has no incoming call or jump in the boot program or in any of the 524 level modules, no word equal to its address (absolute, or relative to the load base) in the boot data or the module data, and no split high-half and low-half address load that forms it. The rig body (the lean path) is called only from the eye-lag stage. So neither routine can run in the shipped game, and a clean build leaves them out; only a computed address built some other way is not excluded. Resolved: not reachable. **[High for the known rig steps; Medium-High for non-reachability]** Default if unresolved (rewrite choice): omit the eye-lag stage and the rig lean path; the follow camera never sets the eye-lag flag.
2. **Player-object tests.** Resolved operationally: these are camera eye/contact collision filters on world primitive records, not class tests on the player character. The regular eye pass applies primitive class-bit gates before its geometric sweep; the secondary camera contact pass repeats the primitive and character-subshape routes and reports whether the camera eye moved. Camera movement handlers call the secondary pass, and the eye-lag rig path also calls it; that rig path is unreachable (Open question 1). A failed class gate leaves the eye unchanged. Frame-record and character-subshape routes use separate checks and no primitive class-bit gate. The runtime role and callers are established; the authored meaning of the class bits is developer intent and not encoded anywhere. See Eye pass per-primitive routines and Walls and the stuck timers. **[High for route and predicates; Medium for design meaning]**
3. Resolved: **Blocked tests.
** The contact pass pieces are item 60. The two orbit shrink tests are a line test (the segment query from the focus to the candidate eye, two-sided walls ignored) and a buried test (a ground probe at the candidate's x and z with a ceiling 5.0 above the focus height, blocking when only higher ground is found); see Recentre pass and R3. **[Medium-High]**
4. Resolved: **Camera field meanings.
** 0x120 is the focus point F, 0x130 the target polar P and 0x140 the current polar C (polar of eye minus focus, with the spun yaw); 0x100 is the eye motion direction (unit vector, length in lane 3). 0x1D4, 0x1D8 and 0x1DC are the camera's ground entry, its world and the ground height. The side-swing increment (0x1C8) and its counter (0x1CC) are written by the side-swing event (Side-swing event), and 0x1D0 is the R3 recentre latch (see Layout). **[High]**
5. Resolved: the mode 12 entry is SetCameraToTrackHeroMode (see Dead-band yaw follow); no level script calls it. The mode 9 entry has no caller, so mode 9 is dormant and its step, fractions and selector values (0x22C and 0x230) have no shipped values; the order of the fraction fields (0x21C to 0x224) comes from the mode 8 entry. **[High]**
6. Resolved: **Mode 3 and mode 7 bodies.
** Both bodies are described in full (see Mode 3 and Mode 7); both modes are dormant debug cameras. **[High for the arithmetic]**
7. Resolved: **Mode 7 setter step.
** Setting mode 7 raises the camera position to at least 100.0 units above the ground under it (see Mode 7, Entry). **[High]**
8. Resolved: the camera updates once per tick, after the pad snapshot and the game-status update and before the world update, so it follows the hero pose of the previous tick (see Update order). The tick rate is 60 Hz (SPEC-0007). **[High]**
9. Resolved: **Drift threshold.
** The rule switches the drift to +1.3 above 0.99 and to -1.3 below -0.99 (a jump, not a clamp). The instructions load the single constant 1.3 once and use it for the positive branch and, negated, for the negative branch, so the rule is symmetric and is not a misreading of a clamp to 1.0. With no leans the jump is reached only at the very end of stick travel (pad bytes 0 and 255 give 1.3; bytes 1 and 254 give 0.955). Reproduce it. Test vector: the stick held fully to one side from rest gives one full revolution in 128 ticks (2.13 s); a clamp to 1.0 would give 165 ticks, and byte 254 gives 172 ticks (2.87 s). Why the original authors chose a jump cannot be known and does not change the behaviour. **[High]**
10. **Sticky manual flag intent.** Resolved for shipped operational behavior: when T is 0 and the flag is already set, the manual step runs; zero stick leaves the manual elevation unchanged, and the flag remains set because the chase step does not run. Once T becomes 1, the chase step runs, clears the flag, and returns toward the chase target. The manual step tests the old elevation before adding its step, so it can overshoot the ±1.5-rad bound by one step and clamps on the following tick. Whether the sticky hold and one-step overshoot are intended is developer intent and not encoded anywhere; reproduce the behavior as stated. **[High]**
11. Resolved: **Mode 8 lane names.
** Lane 0 is the distance, lane 1 the yaw and lane 2 the elevation; 0x210 is a yaw offset, and the radius is not stored (see Blended transition). **[High]**
12. Resolved: **Mode 8 step comparison.
** The pivot step is compared with the remaining distance to the target point, not with a direction component, so it is not a defect. With step 0 the pivot never moves. Mode 9 uses the same comparison. **[High]**
13. Resolved: **Mode 8 callers.
** The live caller is the hero death handler on the game-over branch: radius 15, yaw offset 0, height 1.5, fractions 0.01, pivot step 0, no placement, only when the camera is not already in mode 8 (see Death camera). SetHeroSceneCamera(a, b) uses the same entry with yaw offset a and radius b; its only script call site is in a routine of pack 1250 that no call, registration or table names, so no script calls it in play. The second live caller is the full item showcase, which calls it as SetHeroSceneCamera(0.9425, 10.0) (radius 10.0, yaw offset 0.9425, height 1.5, fractions 0.01, step 0, no placement; SPEC-0021) and leaves the mode with the return routine. **[High]**
14. Resolved: **Near and far planes, widescreen flag writer, fade consumer.
** Near 1.0; clip far planes 5280 (world and characters), 600 and 10560; GS far 10560 (see Projection and SPEC-0046). The widescreen flag setter has no caller, so 16:9 is never on. The 4094-unit matrix family is the guard-band clip set. The fade consumer: The renderer uses the hero's fade level as the composite fade level (SPEC-0046). Below 128 the hero is drawn off-screen and composited as one sprite with alpha = level / 128, and sorts as translucent. At 128 the composite path is off. This is not material alpha (SPEC-0003). See Fade ramp. **[High]** for the planes, the widescreen flag and the sort rule, **[Medium-High]** for the composite draw.
15. Resolved: the wrap result is used; it is the spun yaw of follow step 6. **[High]**
16. Resolved: **Pull-in lane scaling.
** The setter always replaces the eye height with the focus height; when the ground at the eye is more than 5 above the focus it also halves the horizontal (x and z) offset from the focus. The fourth lane is set to 1 (see Pose setter with pull-in). **[High]**
17. Resolved: **Return-to-normal and mode 12.
** No caller handles mode 12: the return routine's only callers are ReturnToNormalCamera and the two SetToNormalCamera forms, and mode 12 is left only by EndCameraTrackHeroMode or a world-load initialisation (see Return to normal). **[High]**
18. Resolved: **Excluded state and stick value.
** The drift base and the stick value are zero during the streaming-load freeze: the game-globals words 0x35C980 + 0x24 (boundary contact) and 0x35C980 + 0x28 (load busy) are both set (see Stick value and spin input). **[High]**
19. Answered: **Physical stick for the pad bytes.
** Left-stick Y at 0x11F, left-stick X at 0x123, right-stick Y at 0x127 and right-stick X at 0x12B. Mode 3 and mode 7 read the pad bytes 0x11F, 0x123, 0x127 and 0x12B. Right-stick Y drives the elevation and right-stick X the yaw (item 53). **[High for the mapping]**
20. Resolved: **Side-lean variable s.
** s is the sideways offset, along the camera's first row, of the hero's current target from the hero; the state test is the hero's target flag (the nearest noticed enemy within 20 units). See Hold and lean terms. **[High for the camera side; Medium for the pick rules]**
21. Resolved: only the 0.65 rule sets the follow distance (see Target distance); the 0.25 rule scales the focus height in hero state 21 (follow step 5). **[High]**
22. Resolved: **Ground probe.
** The per-tick probe is a downward ground probe at the camera position (see Ground probe). Its entry slot is cleared on each query; a miss in the interior buffer skips the alternate world and retains the prior world pointer, which the end trigger pass can still use. A null current-world pointer leaves all three probe fields untouched. The update has no second orbit pass: after the probe it only stores the room index and runs a trigger pass at the camera position over the kind 1 and kind 8 primitives of its grid cell, which never moves the camera (End of the update). **[High for the known branch behavior]**
23. Resolved: **Camera point query.
** In mode 0 it returns the attached character's position (record 0x58 points to it; the attach sets it) plus the character height on y. Its callers are the mode 8 and mode 9 entries only. **[High]**
24. Resolved: **Mode 10 entry and fallback.
** The key is an entity id of the current world (SPEC-0006 tag 0x12), resolved to a loaded character; the entry is reached only from SetAI_DistCamera and SetHeroDistCamera; no fallback record exists. **[High]**
25. Resolved: **PlayCamera arguments.
** Track id, (target for the targeted variants), wait option, end mode (the camera-wait value 0, 1 or 2; not an entity index) and speed factor (not seconds). The target comes from the variant called (see Scripted cameras). **[High]**
26. Resolved: **Mode 6 and mode 7 directions.
** Mode 6 is empty. Mode 7 looks straight down (forward (0, -1, 0)); its up row is the focus character's camera-relative stick direction, its first row is down x that direction; right stick X moves along the first row, right stick Y along the up row and left stick Y along the down direction (see Mode 7). **[High]**
27. Resolved: **Track data and interpolation.
** Camera tracks are keyframed type-28 records read from the world pack (SPEC-0052). Position is lerped; the quaternion is lerped after a sign flip, without slerp or renormalising; the head's +0x2C value is the focal length pushed to the render camera; the play ends on the first update with no valid sample, a 0-key track never ends and a 1-key track ends on its first update (see Cutscene camera tracks). **[High]**
28. Answered: **Field of view.
** 64.0 degrees horizontal in normal play and the track's focal length during scripted plays (see Projection); planes in item 14; no boss camera mode (Scripted cameras); lock-on is item 55. **[High]**
29. Resolved: **Shake envelope and axes.
** Four keys (0, 0), (p4, A), (T - R, A), (T, 0) with linear blending; the movement is along the frame's horizontal first row (share cos|phi|) and the world up (share sin phi). See Per-frame use of the shake. **[High]**
30. Resolved: **Shake argument order.
** StartCameraShakeMode(type, duration, strength), amplitude 0.3 x strength (see Camera shake). **[High]**
31. Resolved: **Shake preset 0.
** Preset 0 fills the shake record like the others; it is a normal shake. **[High]**
32. Resolved: **Shake field meanings.
** p4 is the attack time, R the fade-out time, the copies of S are the per-key wave rates, the angle terms split the movement between sideways and vertical, and Z above 0 turns the shaken view toward an aim point 0.5 A + 100 (1 - Z) ahead. A start clears the elapsed time and the key index, so every shake plays from its beginning. **[High for the code]**
33. Resolved: **Mode 3 height term.
** The shoulder-button group (L1 +0.1, L2 -0.1, R1 +0.01, R2 -0.01) is gated by camera field 0x68, which is always 0, so it never applies. Its value is a plain float added to the height; there is no conversion to a whole number (see Mode 3). **[High]**
34. Resolved: **Mode 7 directions and mode 3 buttons.
** Y is the vertical axis; mode 7 looks along (0, -1, 0), and the sign of each move follows from t = 127.5 - byte along the rows given in Mode 7. Mode 3 reads the D-pad bits of the held word (Up 0x1000, Right 0x2000, Down 0x4000, Left 0x8000) and, only when enabled, the shoulder bits. No caller passes 7 or 3 in play, so both modes are dormant. **[High]**
35. Resolved: **Modes 2 and 4 dispatch.
** The general dispatch handles modes 0, 2, 3, 4 and 5 (2 and 4 have empty bodies; 5 and other values do nothing); modes 1 and 11 call the mode 3 body directly. No caller sets mode 2 or 4. **[High]**
36. Resolved: **Camera initialisation.
** The default offset (0, 0, 50) and forward (0, 0, -1) are fixed constants (50.0 and -1.0), not boot data words; the initialisation then attaches the camera to the player, and placement after a load or restart is described in its own section. The frame convention is in Polar convention. **[High]**
37. Resolved: **Camera volume kinds.
** There are no camera volumes. The kinds belong to the camera's wall push over the cell's collision primitives: 1 plane, 3 cylinder, 4 and 7 box push or zero-radius contact, 2 (cone) skipped, other kinds ignored (see Walls and the stuck timers). **[Medium-High]**
38. Resolved: **Camera volume list and kind byte.
** The kind byte is +0x10 of the loaded collision primitive in the grid cell's list (SPEC-0018). The type 41 records are atmosphere zones (SPEC-0028 section 11); the camera does not read them. **[Medium-High]**
39. Resolved: **Camera volume blend.
** The 50.0 (0x2DB4CC) and 25.0 (0x2DB4D0) thresholds belong to the type 41 atmosphere zone pass (SPEC-0028 section 11): 50.0 is the zone range and the world-change crossfade distance, 25.0 the gated-zone blend distance, and the reference point is the camera eye. **[Medium-High]**
40. Resolved: **Pose blend helper.
** It is the type 41 zone record blend and has one caller, the zone pass (two calls). Between 0 and 1 it blends only vector A (+0x00), vector B (+0x10) and the render scalar (+0x20) and leaves the rest of the target as it was; at or below 0 and at or above 1 it copies the whole 0x50-byte record. Both calls ignore its return value (Zone record blend). **[High]**
41. Resolved: **Screen record size.
** The record is filled once at boot and is always 512 by 448: the display-mode word comes from the boot settings entry -1 (automatic), which picks the 512 by 512 mode only when the video-standard query returns 2, and that query always returns 0 in this build (see Screen size). The owner is the boot display object; the field positions and the 144 and 168 values are not needed for play. **[High for the size; Medium for the owner]**
42. Resolved in part: **Quad setup.
** The caller is the screen/canvas initialiser, so this is not camera behavior. The four-corner quad uses 1/16-unit coordinates with extent 128, and a flag adds an 8-unit nudge. The flag meaning and nudge purpose are a presentation detail of the canvas initialiser with no effect on camera or play; the clean side follows SPEC-0023, Canvas mapping, and does not need them. **[Medium]**
43. Resolved in part: **Clip rectangle.
** The caller is the screen/canvas initialiser, not the camera. It converts four object fields to 1/64 units with truncation toward zero and writes clip value 16,380. The source fields and the clip value's purpose are a presentation detail of the canvas initialiser with no effect on camera or play; the clean side clips to the screen as SPEC-0023, Canvas mapping, describes. **[Low]**
44. Resolved: **View object and camera record.
** The 0x160-byte view object is not the camera record; it is the display object of the type 41 zone pass, which receives the zone output (SPEC-0028 section 11). **[Medium]**
45. Resolved: **Look-at scalar and owner.
** The scalar is the hero's character height, added to y; the routine is a scripted camera play start whose owner is the camera object, and the setter is the camera's rows setter. The reference direction is the horizontal quarter turn of the forward, not the up vector. See Look-at play start toward the hero. **[Medium-High]**
46. Resolved: **Blend and fade mapping.
** The weight 1 - d / 50 is the world-change crossfade (output = lerp(working, previous output, w)); the fraction distance / 25 is the gated-zone blend term that becomes the alpha of the second sky-dome model (SPEC-0028 section 11). That value is kept in the camera record at +0x500 and is the master opacity of the sky dome's role-10 layer, which is drawn only while the value is above 0 (SPEC-0046, Sky domes). **[Medium-High]**
47. Resolved: **Reference direction.
** The run-time block at 0x3932E0 is the up vector (0, 1, 0, 0), written once at start-up (see Where it lives on the disc). **[High]**
48. Resolved: **Follow speed offset.
** It is not a camera routine: it is the class-2 surface test of SPEC-0018, run on a 0x90-byte world surface record and a character motion record by three character collision routines (Follow speed offset: not a camera routine). **[High]**
49. Resolved: **Drift factors a and b.
** They are the spin gain a = 0.005 and the spin decay b = 0.125 of the camera profile record (see Camera profile record); the spin rate tends to D a / b = 0.04 D rad per tick. **[High]**
50. Resolved: **Which yaw writes the final eye yaw.
** Only right stick X turns the yaw directly: the eye is re-placed each tick on the sphere about the new focus with its yaw increased by the spin value I; the chase and manual steps set the elevation, and the spring pulls only the distance and the elevation (outside the camera-locked states). See Follow camera. **[High]**
51. Resolved: **Body reference position.
** The reference that T compares with is the hero's position at the start of its last motion step (the motion step copies the position there before it moves). So T means "the hero moved during its last motion step", which from the camera's side is the previous tick (see Chase and manual elevation). **[High]**
52. Resolved: **Hero state 1 and the sub-state exceptions.
** State 1 is idle. The two exceptions are one: state 13 (attack) with the hero variant word +0xF80 = 1 (Roberta) and attack sub-state 1 to 4, 9, 10 or 14 (Recentring gate T). **[High]**
53. Resolved: **Right-stick Y as yaw.
** Right-stick Y moves the elevation (pitch) and right-stick X turns the yaw through the spin integrator (see Chase and manual elevation and Spin). **[High]**
54. Resolved: **Negation test names.
** The current action is state 21 (climbing a climbable panel) and the zone phase is the shared climb and ladder sub-mode word +0x9BC, value 4 (the stick-steered move on the panel; Chase). **[High]**
55. Resolved: **Lock-on.
** There is none. Every hero pad button already has a role, and the only button-held camera behaviour is the L2 strafe, which holds the camera yaw on its back direction (camera-locked rule) with no target (see Lock-on). **[High]**
56. Resolved: **Attach hook.
** The hook is the pull-in setter. The boot and world-load attaches run it; the hero-switch attach does not, so after a switch the eye keeps the hero's position height with no pull-in (see Placement after a load or restart). **[High]**
57. Resolved: **Mode 7 up row.
** The vector is the heading of the focus character's locomotion block. It holds the last off-centre left-stick direction (unit, in world space), (1, 0, 0) after a block reset, and is not rewritten while the stick is centred (Mode 7). **[Medium-High]**
58. Resolved: **Character forward row.
** The character matrix rows are at +0x290, +0x2A0, +0x2B0 and +0x2C0 in the camera's row order, so +0x2B0 is the third (forward) row; SetToNormalCamera's angle pi and the feed action's spawn point 2.8 along that row agree (see Death camera, Forward row). **[Medium-High]**
59. Resolved: **Game-over screen and the death camera.
** The death camera swings for about 92 passes (timer E, 1.5 s) before the page, and keeps swinging under it, because the camera mode update runs on every pass and the world is still drawn around the 512 by 128 page picture (see Death camera, Under the game-over page). **[High]**
60. Resolved: **Contact pass pieces.
** The contact pass of modes 8 to 10 uses the same lists, order and push pieces as the follow wall push, on a copy of the camera position that is written back only when pushed (see Blocked latch). **[Medium-High]**
61. Resolved: **Eye pass routines.
** The five routines handle the ground frame records, kind 1, kind 3, kinds 4 and 7 with radius value at or below 0, and the region characters; the box push (kinds 4 and 7 with a positive radius value) posts nothing. The 0.01 event is the side-swing event: for 10 ticks it adds plus or minus 0.01 per tick to the spin value I when the push normal points along the camera's forward row, so the eye swings sideways off the wall (Side-swing event). **[High]**


