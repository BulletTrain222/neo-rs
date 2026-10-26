# SPEC-0034: Visual effects and particles

- Answers: proactive (effects, sparks and particle bursts that the animation tables, world placements and script calls start), R-0043, R-0044, R-0045, R-0047 (parts 1 and 2; part 3 in SPEC-0051), R-0051 (answered; the run-time write of the material word is Medium), R-0052 (answered for the code path; play check in the user checks)
- Status: Draft
- Disc build: SCUS-97367 v1.00 (same fingerprint as SPEC-0001)
- Confidence: per section. Sections 14 to 20 (with their constants and edge cases): the frame object of section 14 Medium-High (lifetime constant High); UV scroll Medium-High (an animator kind that no material on disc selects, so it is unused); particle sprite batch packets Medium-Low; trail slots Medium-Low (Medium-High that nothing starts or updates them); the eight static effect records (the swing-trail colour gradients) High for the address, count, stride, field types, values and the roles of +0x00 to +0x40, Medium for +0x44 and +0x48 (the values are given in section 18 by user decision); anchor chain and trigger High for the owner (the Gelert Assassin's light arc), Medium-High for the trigger test; position jitter Medium-High (the ramp moves the y component; nothing updates its records, so it never runs). preset selection (High); the 27 presets' source tables, record layouts and start-up fill (High; the five header words of each preset are given in 3.9 by user decision); the Mote-eater drain with eight slots (3.13; High for the steps and words, Medium-High for the charge use); the hover machine is the item showcase of SPEC-0021 (3.14, High). Sections 21 to 25: triangle oscillator High for the counter, the fraction and the gates, High for the float store and the lane mask, Medium for the 1/30 step being the word at 0x31F5F0; sections 22 to 25 are not effects (High for the grid being unreached and for the world record step; Medium for the sound-cue pool; Medium-High for the tween-node match). Disc counts and layouts: High (from the disc). Effect table loader fix-ups (links, A, B and C references, child and definition references, definition emitter slots): High for the rules and the disc counts; High for the external child lookup of the 116 nodes with a low child byte (common-object models, 2.1). Spawner gate and node pool word: High. Negative base period: High (one burst, then the spawner drains and dies). Pulse effect (kind 3, section 12): phase steps and boot constants High (from the disc); the wave arithmetic and the test vectors High (computed); the update rate Medium (60 updates per second assumed); the reader of the output byte Low. Effect lists and clip windows: Medium-High for the list layout; High for the window units and arithmetic (1.2). Hero footstep dust (3.16): High. Spawn timing, the per-tick clock and expiry: High (test vectors 70 to 72). The owner slot manager: Medium. The shared effect manager, its slot-3 group, effect objects and placed-effect activation (3.16): High. The 8.8 field write rule: High for the rule, Medium-High for the wrap on overflow, Medium for the field meanings. Particle birth, motion (velocity, drag, vertical acceleration, kicks) and curves of sprite particles: High; the model-particle branch: Medium. Status tint routines (5.4, once called owner frame presets): High. Drawing (4.7): Medium-High for the sprite batches and the light publish; Medium for the packet details. Effect group and its driver (Behavior 13, the hero's Mote power and charm object set): High for the set, the owner, the gate and the counts; Medium for the per-object roles (SPEC-0036 owns them). Section 8 (the reed-cut coin roll, not an effect): High for the ranges and the item spawn, Medium-Low for the two points. Section 9 (the map icon pulse, not an effect): Medium-High for the role, High for the constants. Frame tilt (the shot bounce of section 10): High. Uniform draw: Medium-High. Debug vibration tester (section 11): High for the settings, the falloff rule and the rumble request; it is not reachable in retail. Start-up initialisers (the routines that take effect id 0xFFFF): High. Effect definition 162 is absent: High. Swing-trail start (3.8): High for the flow, the gate, the kind source and the record mapping; Medium-High for the sample and colour rules; Medium for the draw. Trail ageing gate (3.8, 4.6): High. Hit-flash keys (section 32): High for the values, Medium-High for the ramp; Medium-High that the part draw reads colour slot 1, so the flash tint is shown. One-shot effect at a point (3.16): High for the steps, Medium for the callers' roles. Type setup (3.9, world-object emitter presets): High for the arithmetic, the role, the table locations and the copy rules; Medium for the launch speed taken from the direction length. Timed emitter (3.10): High for the steps; Medium-High for its meaning (an enemy showing a stolen item). Foliage sparkle 139 (section 38): High for the values, Medium for the picture. Timed entry list (3.11): High for the steps, Medium for the stale-slot detail. Segment join (3.12 rumble envelope pieces): High for representation, state gate, join and slot release; Medium-High for selected output. Clip-to-list lookup (1.2) and the sub-slot, buffer and id rules (7): High for the returns and formulas, Medium for the field use. Definition lookup, counts and the launch, lifetime and gravity facts (2.7): High for the counts, Medium for the meaning. Material, texture and blend (2.8): High for the values and runtime group base, Medium for the meaning. Test vectors 66 to 69: High (disc values). Animation effect-list index (1.2, Where it lives, test vector 49): High for the field position (first word, bits 14 to 21) and the counts. Use of definitions (2.9): Medium for the reference counts; Low for any reading that an id with no known reference never plays. Item effect mapping (3.15): High for the mapping and the item use; Medium for the second caller. Save-point glow (section 36): High for the definitions, the load, start, move, update and stop rules and for the emitter values of its look; Medium for the picture. Hazard-surface effects 129 and 188 (section 37): High for the values, Medium for the picture. Attack-family effect tables as an effect-id source (section 2.9): High for the layout and the counts; Medium-High for the readers' callers. Placed-effect zone number (+0x44) and the per-pass zone gate (section 6): High.

## Summary

Effects are short-lived visual sources with no gameplay effect of their own. Three things start them: a character's animation clip (an effect list attached to the clip), a placed effect in a world pack, and a script call. Each started effect is an **effect instance** that takes one of 10 pool slots. An instance looks up its **definition** (a record with an id) in one fixed definition table, and runs one or more **emitters**. An emitter repeats cycles of a fixed period, and each cycle launches a burst of **particles**. Each particle moves with a velocity, a per-tick drag, a vertical acceleration along world up and optional periodic upward kicks, lives for a fixed life drawn at birth, and drives two curve channels (colour and size) over that life. An owner (a character or world object) holds its effects as effect objects (copies of definitions, 3.16). The character status display also tints its owner through the render object's colour-override slot (5.4). Particles with a material are drawn as screen-facing sprites in batches keyed by material; particles of a light-emitter emitter are not drawn and publish a point light instead (4.7). All timings run on the 60 Hz game tick (SPEC-0007), so a time t in seconds is t × 60 ticks.

A clip's effect-list entry can also start a swing trail instead of a particle effect: a ribbon between two attach points, coloured by the attacker's element (3.8). Effect kind 3 in the shared effect list is a pulse. It is not a particle emitter: it runs two phase counters (one that wraps in 0 to 1, one that grows without a wrap) and writes a single intensity byte, which is a wave between 64 and 250, into a 16-byte colour preset. It also writes a phase offset into each item of the owning object. The wave is a fixed truncated power series of a folded phase, so it is not an exact sine (section 12).

## Where it lives on the disc

- **Character effect lists**: type-36 records of tag 0x10 inside character regions (SPEC-0006 kind records). The animation tables (SPEC-0005) name an effect list per clip: the list index is in bits 14 to 21 of the first 32-bit word (word A) of each table entry, the same field that SPEC-0005 gives as the effect index. 575 regions contain type-36 records; 1,451 tag-0x10 lists sit in 263 of them (maximum 19 in one region); 1,863 entries in total.
- **Placed effects**: type-39 chunks in world packs (SPEC-0004 world pack, resources whose first chunk is type 31). 158 chunks, 5,139 records.
- **Definition table**: type-34 chunk inside GAMEFILE.DAT resource 9 (the item stream number 3 in the boot's loader order), chunk offset 0x337B0 within the resource, 837,856 bytes. This single chunk holds the whole effect group used by character effects, placed effects and script calls. Its header holds 50 child chunks, 1,651 emitter nodes and 666 definitions (section 2.1). The definitions carry 665 distinct ids, from 101 to 6005; id 137 appears twice and the first match is used, and id 162 is absent (section 2.7). The world loader's type-34 branch calls the effect table loader for this chunk; the loader's fix-ups are in section 2.1. Other type-34 chunks (resources 906, 1246, 1250, 1254, all world packs) are separate groups that the list path does not use.
- Constants that the clean side reads from the boot program's data (virtual addresses, 32-bit IEEE floats unless noted; the file offset is the virtual address minus 1 MiB plus 0x80):
  - Cone limit, squared: 0x2ECCC0 = 225.0 (15 units).
  - Per-tick step: 0x2ECCC4 = 1/60 (stored as 0.016666668). The effect table load writes the same 1/60 into it again, so the effect step is always 1/60 s; use dt = 1/60 s.
  - Effects-enabled word: the emitter step runs only while the word at 0x2D7520 is non-zero. It is 1 on disc and has no known writer; treat effects as always on (Medium-High).
  - Pools (run-time, sizes fixed): shared group 3 uses global pools of 300 spawners and 800 particles; each character has 30 spawners and 100 particles; each world has 40 and 200. A spawner element is 0x130 bytes and a particle element 0x170 bytes of run-time layout (High).
  - Sway constants (Behavior 9), all on disc: 0x2DD310 = u16 4300 (upper bound of the sway phase); 0x2DD312 = u16 3900 (lower bound of the phase); 0x2DD314 = u16 50 (initial phase step); 0x2DD31C = f32 0.8 (lower bound of the amplitude); 0x2DD320 = f32 1.5 (upper bound of the amplitude); 0x2DD324 = f32 0.03 (amplitude step); 0x2DD328 = u16 4096 (initial phase, updated in place); 0x2DD32C = f32 1.0 (initial amplitude, updated in place).
  - Debug vibration tester settings (Behavior 11), on disc: 0x2D7568 = 0 (the test trigger, set from the debug page); 0x2D756C = 0 (vibration type, the rumble preset); 0x2D7570 = f32 1.0 (duration in seconds); 0x2D7574 = f32 0.5 (magnitude); 0x2D7578 = f32 1.0 (inner falloff radius); 0x2D757C = f32 30.0 (outer falloff radius). These are rumble settings, not effect constants.
  - Envelope presets: five records of 8 floats at 0x2DD270, stride 0x20 (four times, then four levels). The first row reads 0, 0.3333, 0.6666, 1.0 for the times. They are the rumble presets of SPEC-0033 Rumble; no effect behaviour in this spec depends on them (Open question 10).
  - Basis block (High): five constant vectors written once at boot by a start-up initialiser and never changed: 0x3932C0 = (0, 0, 0, 0), 0x3932D0 = (1, 0, 0, 0) (X), 0x3932E0 = (0, 1, 0, 0) (world up), 0x3932F0 = (0, 0, 1, 0) (Z), 0x393300 = (0, 0, 0, 1). They lie beyond the end of the file image, so they are not on disc; the clean side uses these values.
  - The shared effect manager (3.7, 3.16) is a run-time block at 0x382240, built the first time it is used. The swing-trail manager pointer at 0x35C8A0 is set once at start-up to the block at 0x35C6F0 (12 trails of 18 samples) and is never null afterwards (3.8). Neither is on disc.
  - Common-object table (High; run-time, in the shared effect manager block): global model ids 0 to 322 in four ranges, 0 to 97 (98 ids), 98 to 190 (93), 191 to 282 (92) and 283 to 322 (40). The fixed words written when the block is built are this range table (per range: the count, the first id and the last id). Range r is filled by item stream r at load: each type-12 or type-13 chunk (also each child of a type-8 chunk) registers its model under the index in its u16 at chunk +0x0E. A type-34 chunk of stream r binds group slot r. Resource 9 (stream 3) holds 38 such models, indices 0 to 37 (ids 283 to 320).
  - Pulse constants (Behavior 12), all on disc as 32-bit floats: 0x2DCA98 = 0.0 (initial phase A); 0x2DCA9C = 0.06 (phase A step); 0x2DCAA0 = 0.2 (phase B step); 0x2DCAA4 = 64.0 (intensity low); 0x2DCAA8 = 250.0 (intensity high); 0x2DCAAC = 0.0 (initial phase B).
  - Pulse presets and floor word: the two 16-byte preset records (at 0x374D10 and 0x374D20) and the floor word at 0x374D00 are run-time memory beyond the file-backed part of the program image, so they are zero on disc. A start-up initialiser sets 0x374D00 to 64.0 (a copy of the boot word 0x2DCAA4). The two presets stay zero until a draw copies 16 bytes into them from the drawn object's +0x3E0 and +0x3F0 (Medium-High). The clean side must not read them from the disc.
  - Read rule for start-up tables (High for the rule): no table values that the boot program builds at start-up are copied into this spec. The clean side reads them from the user's own boot executable at build or start-up time, like disc data: file offset = VA - 0x100000 + 0x80. The file-backed image ends at VA 0x32B214. A run-time table above that address is zero in the file, so the clean side reads the source tables that start-up fills it from, and then performs the same fill (3.9).
  - Run-time tables (beyond the end of the file image; memory above it is zero-filled at load, and start-up initialisers fill these once at boot): the trail colour table of 3.8 is the eight static records of section 18, 8 records of 0x50 bytes at 0x370DF0 to 0x37106F. The type setup of 3.9 uses 27 preset records of 0x80 bytes at 0x3711B0 to 0x371F2F (the cap is the s32 at +8 of each record) and three first-time tables: 27 records of 0x50 bytes at 0x371F30 (C), 27 of 0x90 bytes at 0x3727A0 (A) and 27 of 0x90 bytes at 0x3736D0 (B). Record i of each table belongs to preset type i + 1.
  - Preset source tables (inside the file image, so read by the rule above; all six are 27 records long and lie back to back from 0x2DB4E0 to 0x2DC98B):
    - Size start rows: 27 records of 0x10 bytes (four f32) at 0x2DB4E0, file offset 1,946,976 (decimal). Row i fills preset i at +0x60.
    - Size rate rows 1, 2 and 3: 27 records of 0x10 bytes each at 0x2DB690 (file offset 1,947,408), 0x2DB840 (1,947,840) and 0x2DB9F0 (1,948,272). Row i fills preset i at +0x20, +0x30 and +0x40.
    - C sources: 27 records of 0x50 bytes at 0x2DBBA0, file offset 1,948,704. Record i fills first-time C record i and a present C record of type i + 1.
    - B sources: 27 records of 0x34 bytes at 0x2DC410, file offset 1,950,864. Record i fills first-time B record i and a present B record of type i + 1.
    The layouts and the fill steps are in 3.9.
  - Footstep dust table (3.16): 28 signed 32-bit effect definition ids at 0x3150A8, one per dust entry, -1 = none.
  - Immediates: the timed emitter scale of 0.45 (3.10) and the segment tolerance of 0.001 (3.12) are fixed constants, not data words.
  - Preset header words: the first 0x14 bytes of each 0x80-byte preset record (base period, period jitter, cap, the word for A +0x68 and the life; 3.9) have no source table in the file image: a start-up initialiser writes them from values held in its own code. By user decision their values are published in 3.9 (preset header values table), and the clean side uses them as constants. The rest of each preset comes from the source tables above, which stay read from the boot executable.
  - Mote-eater drain words (3.13; on disc): 0x2DCAC8 = f32 30.0 (duration in seconds); 0x2DCACC = f32 4.5 (body effect height above the hero); 0x2DCAD0 = f32 4.0 (slot height); 0x2DCAD4 = f32 0.0 (slot spread a); 0x2DCAD8 = f32 0.5 (slot effect scale); 0x2DCADC = s32 1 (charge-use switch); 0x2DCAE0 = s32 20 (charge used per revealed slot). The per-Mote effect id table is 14 s32 words at 0x2DCAE8 (read from the boot executable). The eight-slot cap is a fixed constant. Leach hit: 0x2DD458 = s32 50 (MP drained).
  - Per-joint effect id table (2.9): 24 s32 words at 0x3132A0 holding four distinct ids, 453, 455, 456 and 457; none is in the definition table.
  - Hit-flash force word (section 32): 0x2D7600 = 0 on disc; non-zero forces the hit flash on the hero.
  - Timing words (boot data words, 32-bit floats, on disc): billboard lifetime in seconds at 0x2DCB34 = 5.0; trail countdown in seconds at 0x2DB498 = 1.0; jitter ramp ceiling at 0x2DD41C = 0.075; jitter ramp step at 0x2DD420 = 0.005; UV scroll divisor word at 0x2DCB24 = 0.025 (the scroll divides time by 100 / 0.025 = 4000).
  - Run-time words beyond the file image (not on disc; values from start-up): ticks per second at 0x368FA0 = 60.0; the time step at 0x368FA8 = 1/60; the UV scale word K at 0x368F98 is the main-loop pass counter (an integer, 0 at start, plus 1 at the end of every pass; section 15); the reference direction used by the quad and anchor routines at 0x3932E0 is world up (0, 1, 0, 0) (the basis block of this section); the random generator state at 0x39BE50 (used by the jitter draws of section 20 and by every uniform draw, 3.3); the static effect record area at 0x370DF0 to 0x371070 (section 18), written once at boot; the two trail colour clamp vectors at 0x42BE30 (low) and 0x42BE20 (high) (3.8; a start-up initialiser sets the high vector to (255, 255, 255, 255) and the low vector to (0, 0, 0, 0)); the hit-flash key block at 0x374830 (section 32).
  - Triangle oscillator (section 21): the step 1/30 is the boot data word at 0x31F5F0, which holds 0.033333335 as a 32-bit float on disc and is the only aligned word with that value in the file image. It is a separate word from the physics-block time scale at 0x3A1A74 (SPEC-0033, Physics block), which has the same value but is written at start-up and is not on disc (Medium). The index displacement word at 0x2EC830 is zero on disc. At run time it holds the current display buffer index (0 or 1): every model draw stores its buffer argument there before its per-buffer lists are used, so the disc value holds only until the first draw (Medium-High). The oscillator's slot step (+0x1340 + 8j for buffer 0, +0x1344 + 8j for buffer 1, section 21.4) uses it.
  - Fixed constants (not data words, so they are not disc data): the field values of the eight static records (section 18; they have no source table in the file image and are given in section 18 by user decision), and the plus or minus 0.01 jitter range constants (sections 18 and 20; the ramp limits are data words, above); the trigger radii 2.0 and 1.0 (section 19); the 24-sample packet limit and the 0xC0 budget reserve (section 16); the sample record stride 0x30 and the 0xB0-byte block count (section 16).

Run-time tuning words the clean side reads from the boot program's data (on disc; 32-bit floats unless noted):

- Sub-effect timer (section 33): duration 10.0 seconds at 0x2DCBE8; step 1.8 at 0x2DCBE4; scale from the word 3.0 at 0x2DCBEC, halved to 1.5; first argument 2.0 at 0x2DCBDC; second argument 25.0 at 0x2DCBE0.
- Second sub-effect timer (section 33): duration divisor 12.8 at 0x2DCC08; first argument 1.0 at 0x2DCBF8; scale from the word 3.0 at 0x2DCBFC, halved to 1.5.
- Placement distance 5.0 at 0x2DCABC (section 34).
- Degree-to-radian factor, pi / 180 (about 0.0174533), at 0x2DB480 (section 35).

The tick rate of 60 per second is the tick rule of SPEC-0007; burst lifetimes (section 26) and the timers multiply by it. The hit-flash key block of section 32 is written once at boot by a start-up initialiser; it is not disc data.

## Layout

### 1. Character effect list (type 36, tag 0x10)

Record: 16-byte header (u32 tag 0x10 at +0). The list data starts at record +0x10.

| List offset | Size | Field | Remarks |
| --- | --- | --- | --- |
| +0x00 | u32 | list value | Unknown role (values 2022 to 6002). Not read by the walk. |
| +0x04 | u32 | entry count n | 1 to 5 on disc (1,105 lists with 1, 286 with 2, 57 with 3, 3 with 5). |
| +0x08 | 8 | constants | The word at +0x08 is 1 and the word at +0x0C is 2 in every list on disc (1,663 lists over all resources). Never read; write them as given. |
| +0x10 + 0x50 i | 0x50 | entry i | see 1.1. |

Record span: 0x10 + round up to 16 of (0x50 × n + 0x10). This holds for all 1,451 lists.

#### 1.1 Entry (0x50 bytes)

| Off | Type | Meaning | Disc distribution | Confidence |
| --- | --- | --- | --- | --- |
| +0x00 | vec4 | point offset in attach slot A's frame (used with w = 1): gives point 1, the effect position and the first trail edge (3.8, 5.1) | first float 0 (1,797), -0.2 (57), others | Medium-High |
| +0x10 | vec4 | point offset in attach slot B's frame (used with w = 1): gives point 2, the second trail edge (3.8, 5.1) | non-zero xyz in 266 entries | Medium-High |
| +0x20 | u32 | attach slot A: low 3 bits = matrix slot 0 to 7. The value 0x4FFFFFFF is "none"; it occurs once on disc (see Edge cases) | slot 7 (1,147), 0 (362), 4 (150), 6 (108), 1 (63), 3 (21), 2 (11), none (1) | High |
| +0x24 | u32 | attach slot B, same encoding; 0x4FFFFFFF = none | none (1,531), slot 0 (212), 4 (49), 7 (46), 1 (25). Total 1,863 | High |
| +0x28 | f32 | start, as a fraction of the clip duration | 0 to 0.97 | High (values), Medium (role) |
| +0x2C | f32 | end, as a fraction of the clip duration | 0.17 to 1.0; 1.0 in 1,629 | High (values), Medium (role) |
| +0x30 | i32 | effect definition id; -1 = none | -1 (341); 76 distinct ids; every id is in the definition table | High |
| +0x34 | u32 | duration in seconds for rumble from positional mode 0; it is not passed to audio setup on that path | 0.4 as a 32-bit float (1,397), 0 (285) | High for mode-0 rumble duration and the audio-argument boundary; Low for other use |
| +0x38 | f32 | maximum intensity for the positional path | 1.0 (1,819); 2.5, 15.0 others | Medium |
| +0x3C | u8 | when +0x3D is clear, 1 replaces point 1 height with a ground probe 8.0 units below the point | 1 in 580 | High (read), Medium (ground) |
| +0x3D | u8 | when 1, replace point 1 height with the owner character's ground height; if the character has no ground height, probe 8.0 units below point 1, regardless of +0x3C | 1 in 45 | High (read), Medium (ground) |
| +0x3E | u8 | positional mode; 255 = none | 255 (1,699), 0 (164). No other value occurs on disc (see Edge cases) | High (none and 0), Medium (modes) |
| +0x3F | u8 | rumble gate of the positional path: after the sound and shake request of 5.3, the controller rumble request (preset 0, duration +0x34, magnitude equal to the computed intensity) runs unless this byte is 0xFE (254) | 0 in all, so the rumble always runs | High (read), Medium (what 0xFE was meant for) |
| +0x40 | f32 | inner radius (full intensity) | 10 (1,829), others 15 to 60 | Medium |
| +0x44 | f32 | outer radius | 50 (1,829), others 60 to 200 | Medium |
| +0x48 | u8 | trail flag: 0 = start a swing trail (3.8); any other value starts none | 0 (325), 255 (1,538) | High |
| +0x49 | u8 | full-frame flag: non-zero = the effect object gets the full frame (slot A's matrix with point 1 as its translation) instead of only its position | 1 in 5 (the 32-bit word at +0x48 reads 255 in 1,533 entries, 0 in 325 and 511 in 5) | High (read), Medium (use) |
| +0x4C | u16 | never read by the pool update, the window walk, the trail code or the positional path; bytes +0x4A, +0x4B, +0x4E and +0x4F are zero in every entry and are not read either | 0xFFFF (1,695), 0 (168) | High (no reader) |

Ground-snap pairs (+0x3C, +0x3D) on disc: (0, 0) in 1,238 entries, (1, 0) in 580, (0, 1) in 45; none has both set. The 325 trail entries (+0x48 = 0) all have effect id -1 and attach slot B set; they lie in 120 character regions.

#### 1.2 Clip windows (per character, per pass)

The windows are measured in frames on the character's main layer 0 (SPEC-0005). t is the layer-0 cursor in frames, D is the frame count F of the main clip, and L is the layer-0 speed in frames per tick, so each window is exactly one tick wide. Each entry has a start window and an end window:

- start = entry +0x28 × D. End = entry +0x2C × D; if end ≥ D, end = D − 1 (one frame before the clip end).
- If start (entry +0x28 × D) is 0, the start is L instead, so an entry with start fraction 0 starts its effect on the second tick of the clip, not the first (see Edge cases).
- While t is in [start, start + L), the entry starts its effect (if none is running). While t is in [end, end + L), it stops the running effect with the same key.
- A cursor of exactly 0 raises a diagnostic message only; it has no effect on play.
- The list is walked in order each pass.
- Clip-to-list lookup (Medium-High): the lookup reads a table with a count word and 8-byte entries. For index i it takes the 8-bit field at bits 14 to 21 of the first 32-bit word (word A) of table entry i. The second word of the entry does not hold this field. It returns that value, or 0xFFFF when i is at or past the count or the field is 0xFF (0xFF means none). This is the rule that turns the clip's effect field (SPEC-0005, word A) into a list index. On disc, read this way, the field gives 6,202 valid indices and no value out of range.
- When the clip changes (the new clip's list index differs from the old), every effect started by the old list is stopped first, then the new list is used.

Confidence: High for the structure, the units and the window arithmetic.

### 2. Definition table (type-34 chunk, resource 9)

#### 2.1 Chunk header, arrays and loader fix-ups

Payload-relative, six 32-bit signed integers, then arrays in this order:

| Off | Field | Disc value |
| --- | --- | --- |
| +0x00 | child chunk count | 50 |
| +0x04 | emitter node count | 1,651 |
| +0x08 | definition count | 666 |
| +0x0C | emitter record A count (0x90 each) | 952 |
| +0x10 | emitter record B count (0x90 each) | 1,227 |
| +0x14 | emitter record C count (0x50 each) | 664 |

Arrays: A at +0x20, then B, then C, then a table of child chunk offsets (one per child, relative to each table entry), then (aligned to 16) a table of definition offsets (relative to each entry), then (aligned to 16) the node array (0x70 each). The 50 child chunks follow the node array; each is a standard chunk of type 12 (all 50 confirmed) and is referenced by exactly one node. Definition records are reached through the definition offset table.

Loader (High for the rules). The effect table loader runs from the world loader's type-34 branch. It copies the six header counts, then converts stored indices into references in the order below. A stored index becomes a reference when it is in range, and none (zero) when it is not.

1. Child chunk offsets (50): each becomes the start of a child chunk (type 12), which is then loaded.
2. Definition offsets (666): each becomes the start of a definition record.
3. Each emitter node (1,651): the link word (+0x4C) becomes a reference to the node when it is in range; -1 and any other out-of-range value become none. The C, A and B indices (+0x50, +0x54, +0x58) become references to their records (strides 0x50, 0x90 and 0x90). The child index (+0x60) becomes a child chunk reference when the byte at +0x67 is 0x80 or more and the value is 0 to 49; otherwise it becomes none. When the byte at +0x67 is below 0x80, the value is resolved through the loader's context with key (byte + 0x11B) if a context is present, and by the index rule above if no context is given (Medium; the context lookup is not yet identified).
4. Each definition: five runtime child words (+0x58, +0x5C, +0x60, +0x64, +0x68) are cleared. The first n emitter node indices (+0x6C, n at +0x54) become node references; other slots become none.
5. The material index (+0x5C) is not changed by the table load. It is resolved when the group is bound to its pools (3.16): -1 becomes none; otherwise it selects a 0x270-byte shader group record.
6. The load also writes the effect step word (1/60, Where it lives).

Because the loader turns every index into a pointer, the stored form is not kept in memory. The clean side keeps indices and resolves them itself.

On disc, after the loader rules: all 1,651 link words are -1 (none after load); all pool words (+0x40, +0x44) and context words (+0x48) are 0 (they are written when the group is bound, 3.16); all A, B and C indices are in range; of the 1,535 nodes whose child byte is 0xFF, 50 have a child word from 0 to 49 (these become the 50 child chunk references, one per chunk) and 1,485 have a negative child word (none); the other 116 nodes have a child byte below 0x80 and a child word of -1, which go through the context lookup (the child rule below the field table).

#### 2.2 Definition (0x90 bytes, 666 on disc)

| Off | Size | Field | Disc value / distribution | Confidence |
| --- | --- | --- | --- | --- |
| +0x00 | 64 | local 4x4 transform (identity in all) | identity | High |
| +0x40 | i32 | effect id (lookup key) | 101 to 6005; 665 distinct; id 137 twice (first match is used) | High |
| +0x44 | u32 | 0 | all | High |
| +0x48 | u32 | copied into the effect object; for a placed record (section 6) it is the active flag | 1 in all definitions; 0 in all placed records | High |
| +0x4C | u32 | multi-emitter flag: 0 = stop after the first emitter that starts; non-zero = start every matching emitter (3.16) | 1 in all | Medium-High |
| +0x50 | f32 | near/far threshold, squared distance | 10000 (100 units) in 555; 22500 (150) in 46; 1225 (35) in 20; 5625 in 17; others | High (value), Medium (role) |
| +0x54 | i32 | number of emitters n (1 to 5) | 1: 188, 2: 201, 3: 141, 4: 41, 5: 95 | High |
| +0x58 to +0x6B | five words (+0x58, +0x5C, +0x60, +0x64, +0x68) | the effect object's five live spawner slots (3.16), cleared by the loader (2.1 step 4) | zero on disc | High |
| +0x6C | 5 × i32 | emitter node indices; the first n become node references, other slots none (2.1 step 4) | all valid (0 to 1,650); every node used at least once; node 0 used twice | Medium |
| +0x80 | f32 | motion scale S, copied into every spawner the effect object starts (4.1, 4.2) | 1.0 in all | High |
| +0x84 | f32 | cull distance, squared; -1 = never culled | -1 (649); 90000 (300 units) in 8; 22500 in 4; others | High (value), Medium (role) |
| +0x88 | u8 | cone/direction test enabled | 1 (643), 0 (23) | High (value), Medium (role) |

#### 2.3 Emitter node (0x70 bytes, 1,651 on disc)

| Off | Size | Field | Disc value / distribution | Confidence |
| --- | --- | --- | --- | --- |
| +0x00 | 64 | local transform | identity in most; 45-degree rotations in some | Medium |
| +0x40 | u32 | spawner pool used by this node; written for every node when its group is bound (3.16); zero means "no spawner" (the gate of 3.4) | zero on disc | High |
| +0x44 | u32 | particle pool used by this node; written at the same bind | zero on disc | High |
| +0x48 | u32 | shader-list context passed to its particles (part of the sprite batch key, 4.7); written at the same bind | zero on disc | High |
| +0x4C | i32 | link | -1 in all 1,651 on disc; the loader turns every out-of-range value, including -1, into none | High (value and loader rule) |
| +0x50 | i32 | index of the C record (0 to 663) | all valid | High |
| +0x54 | i32 | index of the A record (0 to 951) | all valid | High |
| +0x58 | i32 | index of the B record (0 to 1,226) | all valid | High |
| +0x5C | i32 | material index into the shader list (see 2.1 step 5); -1 = none | -1 (336); 18 to 121 otherwise (86 distinct) | High (range), Medium-High (meaning, section 2.8) |
| +0x60 | i32 | child chunk index. Read by the loader only when the byte at +0x67 is 0x80 or more (2.1 step 3). After load, 50 nodes hold a child chunk, one for each of 0 to 49. Of the rest, 1,485 nodes hold a negative value (none after load) and 116 nodes hold -1 with a low child byte (context lookup: common object byte + 283) | 50 nodes with a child | High |
| +0x67 | u8 | loader gate for the child index. 0xFF on 1,535 nodes: the 50 child nodes and 1,485 others. The other 116 nodes hold values from 1 to 37 (20 distinct) | 0xFF (1,535); 1 to 37 (116) | High (value), Medium (meaning) |
| +0x64 | u16 | preset type: 0 = none; 1 to 27 = world-object emitter preset (3.9) | 0 on all 1,651 nodes of resource 9 | High |
| +0x66 | u8 | ring fill: 0 = random radius in [0, r]; non-zero = exactly r (4.1 step 2) | 0 on 66 of the 71 ring nodes, 1 on 5 | High |
| +0x68 | s8.8 | spawn ring radius r; 0 = no ring (4.1 step 2) | 0 on 1,580 nodes; non-zero on 71 | High |
| +0x6A, +0x6C, +0x6E | s8.8 each | sprite half-extent factors (128 = 0.5): the half-extent lane x takes the factor at +0x6C, lane y the factor at +0x6E and lane z the factor at +0x6A, each times the size lane and S (4.1 step 7, 4.6; 16.4 gives which lane is the width and which the heights). Not colour | (128,128,128) in 1,208; (0,0,0) in 336 (the no-material nodes); others from 82 to 302 | High |

The child rule after load: a node's child is the chunk named by its +0x60 value, when the byte at +0x67 is 0x80 or more and the value is 0 to 49. On disc this selects exactly the 50 child nodes. Nodes whose byte is below 0x80 use the context lookup (High): when the table is loaded with a context, a byte b of 0x27 or less makes the node's child the model of common object b + 283 (0x11B, the common-object table of Where it lives), and a byte above 0x27 gives none; without a context the index rule above applies to +0x60. Both load paths pass the shared common-object registry as the context, so in play the lookup always applies. On disc the 116 nodes hold 20 distinct bytes, 1 to 13 and 30 to 37, so they name common objects 284 to 296 and 313 to 320. Examples: the model particles of the one-shot hit effects 220 and 222 and of the Black Knight spark 305 are common object 292; the near model of the save-point glow 404 is common object 295. Common objects 283 to 320 are the models of resource 9 in index order, so 292 is resource-9 model 9 (a type-13 chunk of 1,712 bytes) and 295 is resource-9 model 12 (a type-13 chunk of 1,760 bytes). The disc stores no name for either model, or for any other resource-9 model (High). No node holds both a material and a child (all 1,651 nodes).

#### 2.4 Emitter record A (0x90 bytes, 952 on disc): timing and gates

| Off | Field | Disc distribution | Use | Confidence |
| --- | --- | --- | --- | --- |
| +0x00 | size start vector (x, y, z, w) | x, y, z in 0 to 40 (common 0, 0.3, 0.5, 1.0); w = 1.0 in all 952 | initial value of curve channel 2 (4.6); x is the sprite size, y and z the per-axis sizes. A copy is kept for the loop restart | High |
| +0x0C | w of the start vector | 1.0 in all 952 | part of channel 2's start value; not the motion scale (S comes from definition +0x80) | High |
| +0x10, +0x20, +0x30 | three size rate rows (vec4) | range -65 to 150 | rate of channel 2 per unit of life (4.6 step 6): row +0x10 while the life fraction f is at or below +0x40, row +0x20 while f is above +0x40 and at or below +0x50, row +0x30 after | High |
| +0x40 | threshold vector, per component | typical 2.0 | first life-fraction threshold of the size rows | High |
| +0x50 | threshold vector, per component | typical 0 or 2.0 | second life-fraction threshold of the size rows | High |
| +0x60 | i32 | -1 (852), 1 (82), 1000 (10), others | end gate: negative = never end by cycle count (3.2). For a preset node it is the total particle count (3.9) | High |
| +0x64 | i32 | particles per burst, 1 to 45 | count of births per burst | High |
| +0x6C | f32 | base burst period in seconds; negative = a single burst (3.2) | see 3.2 | High |
| +0x70 | f32 | period jitter in seconds, 0 to 1; symmetric (the period is base + U(-jitter, +jitter)) | see 3.2 | High |
| +0x74 | i32 | cycle limit (3.2 step 3) | see 3.2 | High |
| +0x78 | f32 | particle life in seconds, 0.1 to 100 (2.0 is the most common) | the life the curves run over and expiry tests against (4.1 step 3) | High |
| +0x7C | f32 | life jitter in seconds, 0 to 5; symmetric | life = A +0x78 + U(-A +0x7C, +A +0x7C) | High |
| +0x80 | i32 | birth position mode: 0 (791), 1 (114), 2 (35), 3 (12) | modes 2 and 3 place the particle on the segment from the spawner to its target point (4.1 step 2); mode 2 is used by effects 436, 437, 474, 509 to 511, 530, 552, 554, 556, 558, 567, 577 to 583, 585 to 591, 1430 and 1500 to 1506, mode 3 by effects 410, 411, 429 to 431, 433, 434 and 5000 | High (counts and arithmetic) |
| +0x84 | u8 | state gate: the emitter is started only while the effect object's state equals this value (0 = near, 1 = far; 3.16) | 0 (909), 1 (43) | High |
| +0x85 | u8 | follow flag: particles add the spawner's per-tick displacement to their position, and do not inherit the spawner velocity at birth | 1 in 302 | High |
| +0x86 | u8 | loop flag: while the spawner runs, a particle at its life end restarts at the emitter instead of ending; a new burst comes only when the spawner has no live particle (3.2, 4.6) | 0 (793), 1 (159) | High |
| +0x87 | u8 | light emitter: its particles are never drawn; each tick each publishes a point light (4.5) | 1 in 37 | High |

#### 2.5 Emitter record B (0x90 bytes, 1,227 on disc): curve rows and clocks

| Off | Field | Meaning | Confidence |
| --- | --- | --- | --- |
| +0x00 to +0x0F | vec4 (4 floats) | row 0: start value of colour channel 1 (r, g, b, a), copied when a particle is born and at a loop restart; also the rate row for key index 0. Values 0 to 1 | High |
| +0x08 | f32 | row 0, blue component | High |
| +0x0C | f32 | row 0, alpha component; 0, 1.0, 0.5 or 0.16 on disc | High |
| +0x10 to +0x5F | 5 vec4 | colour rate rows 1 to 5 (row k at B +0x10 k). The RGB part is taken from the row of the RGB key index and the alpha from the row of the alpha key index (4.6 step 5) | High |
| +0x60 to +0x77 | 6 f32 | RGB key times, as life fractions (the RGB key index advances past them) | High |
| +0x78 to +0x8F | 6 f32 | alpha key times, as life fractions | High |

Disc values: +0x00 is 1.0 in 390, 0.5 in 121, 0 in 83. +0x08 is 1.0 in 230, 0 in 142, 0.5 in 141.

#### 2.6 Emitter record C (0x50 bytes, 664 on disc): motion parameters

Fields are signed 16-bit values in 8.8 fixed point (value = s16 / 256). Pairs are (base, spread): a value is base plus a uniform random number in [-spread, +spread] unless noted.

| Off | Field | Range on disc | Role (used in) | Confidence |
| --- | --- | --- | --- | --- |
| +0x10 | vec4 | zero | extra acceleration term, divided by the acceleration clock T (4.1 step 6) | High |
| +0x20 | s8.8 | -20 to 20 | vertical acceleration along world up, in units/s^2 (negative = falls). Per tick the velocity gains C +0x20 x dt along (0, 1, 0) (4.1 step 6) | High |
| +0x22 | s8.8 | -20 to 20 | drag in 1/s; per tick k = C +0x22 x dt (4.2) | High |
| +0x24, +0x26 | s8.8 | 0 to 3.16 | cone polar angle from the launch axis, upper / lower, in radians | High |
| +0x28, +0x2A | s8.8 | -20 to 90; 0 to 15 | launch speed and spread, units/s | High |
| +0x2C, +0x2E | s8.8 | 0 to 20; 0 to 10 | acceleration clock T and spread, seconds. Its only effect: a T of 0 cancels the acceleration (2 records) | High |
| +0x30, +0x32 | s8.8 | -20 to 90; 0 to 15 | sprite spin rate and spread, rad/s | High |
| +0x34, +0x36 | s8.8 | 0 to 2 each | kick strength R and spread, units/s (4.2 step 2) | High |
| +0x38, +0x3A | s8.8 | -1 to 5; 0 to 1 | kick interval and spread in seconds; 0 = a kick every tick; negative = never (section 2.7). Not a lifetime | High |
| +0x3C | u8 | 1 in 15 | model-particle orientation mode; zero applies spin about the configured axis, nonzero selects a velocity-based frame update. Sprite particles do not use it (Open question 5) | Medium-High for branch selection; exact alignment math remains open |
| +0x3D | u8 | 1 in 2 | zero the velocity at each kick | High |
| +0x3F | u8 | 0 (604), 2 (37), 5 (21), 4 (1), 6 (1) | spin axis of model particles (particles without a material): 0 random, 1 X, 2 world up, 3 Z, 4, 5, 6 = rows 0, 1, 2 of the particle's own frame. Sprite particles never use it | High (counts and codes), Medium (model rotation) |
| +0x40 | u8 | 1 (633), 0 (31) | flag | Low |

Write rule (High for the rule). To store a real value v in an 8.8 field: multiply v by 256; add one half if the product is zero or positive, or subtract one half if it is negative; truncate toward zero to an integer; then keep the result as a signed 16-bit value. A result outside the signed 16-bit range wraps around (it is not clamped). Reading the field gives the stored integer divided by 256. The rule applies to every 8.8 field in the table above. The rule is the game's own conversion code (one setter per field, each multiplying by 256, adding or subtracting one half by the sign, truncating and storing the low 16 bits), used for the fields at offsets 0x20, 0x22, 0x24, 0x28, 0x2A, 0x2C, 0x2E, 0x30, 0x32, 0x34, 0x36, 0x38 and 0x3A (Open question 37, resolved). The setter for the field at 0x26 has the same body but nothing calls it, so that field is never written by the conversion and stays at its cleared value (0).

#### 2.7 Id lookup, counts and life, kick interval and acceleration fields

- Lookup key: the id at definition +0x40. The definition table is scanned in order and the first definition with the requested id is used. The 666 definitions carry 665 distinct ids. Id 137 is at definition 0 and at definition 204; the first (0) is used. Id 162 is at no definition, so a start with that id has no emitters (test vector 16). Sample results: id 101 is definition 211, id 159 is definition 542, id 1201 is definition 161, id 938 is definition 403 and id 4007 is definition 496.
- Particles per burst: A +0x64, from 1 to 45 (2.4).
- Life: the particle life is A +0x78 + U(-A +0x7C, +A +0x7C) seconds (2.4). A +0x78 lies from 0.1 to 100 s on disc; 2.0 is the most common value. Every particle has a finite life.
- Kick interval: C +0x38 (base) and C +0x3A (spread), in seconds, set how often a particle receives a kick (4.2 step 2). They lie from -1.0 to 5.0 on disc. 0 means a kick on every tick (278 records; only 2 of them have a non-zero kick strength). A negative base means no kick ever (199 of 664 records). The raw value 5 (5/256 = 0.0195 s) is about one tick.
- Birth position mode: the mode at A +0x80 has four values, 0 in 791 records, 1 in 114, 2 in 35 and 3 in 12 (sum 952; section 4.1 gives the rules). The spin-axis code at C +0x3F is 0 in 604 records, 2 in 37, 5 in 21, and 4 and 6 in one each; it is used only by model particles, and its fixed axes are the basis block of Where it lives.
- No separate gravity field. The C +0x10 vector is zero in all 664 records. The vertical acceleration C +0x20 along world up (units/s^2, negative = falls) is the gravity-like term; the other motion terms are the launch speed (2.6), the drag at C +0x22 and the kick at C +0x34 and +0x36. A C +0x2C of 0 cancels the acceleration (2 of the 373 records with a non-zero acceleration).

#### 2.8 Material, texture and blend

- Material index: the index at emitter node +0x5C (2.3) selects one group of the shader list at the start of resource 9. That list holds 123 groups, and each group is 0x270 bytes (SPEC-0003 gives the group addressing). A value of -1 means the node has no material (336 nodes). At start-up the type-2 payload from resource 9 is retained as the shared manager's slot-3 shader-list pointer and passed to the node binder. The selected group pointer is `shader_list + 0x10 + index × 0x270`; the shader-list loader has already relocated +0x10 to the group table. **[High for the static load and bind path]**
- Texture: each group has a texture-chunk index at group +0x28. Index 0 selects the first texture chunk after the list and index 1 the second. Over the 123 groups, 45 use index 0 and 78 use index 1. The group's first state block holds a texture setting word at group +0xD0, in the PS2 GS texture layout. From that word:
  - pixel format: PSMT8 (indexed, 8-bit) in 77 groups and PSMT4 (4-bit) in 46 groups;
  - texture size: 64 x 64 in 92 groups, 32 x 32 in 14, 128 x 128 in 9, 32 x 64 in 4, 64 x 128 in 2, 128 x 64 in 1 and 8 x 8 in 1;
  - texture base and palette base: the texture base and the palette base fields of the same word.
- Blend: the group's alpha-blend setting is in the same state block (its field offset is not given here). Its four factor fields A, B, C and D and a fixed value field give the colour rule result = ((A - B) x C) / 128 + D. A and B select the source colour (0), the destination colour (1) or zero (2). C selects the source alpha (0). D selects the source colour (0) or the destination colour (1). The fixed value is 0 in all groups.
  - Alpha blend (A 0, B 1, C 0, D 1; 77 groups): result = (source - destination) x source alpha / 128 + destination.
  - Additive (A 0, B 2, C 0, D 1; 46 groups): result = source x source alpha / 128 + destination.
  - All 123 resource-9 effect material groups use the same alpha-test/depth state in context 2: alpha test enabled, GEQUAL 120, AFAIL = FB_ONLY, and depth test enabled with GEQUAL. An alpha-test failure still writes frame-buffer colour and suppresses only the depth write, so this is not a colour-discard mask. Confidence: High for the disc values and register decode; Medium-High for the FB_ONLY rendering consequence.
- Categories: no field on disc names a category. Counting each definition by the blend and material types of its nodes gives these derived classes (counts over the 666 definitions):

| Derived class | Definitions |
| --- | --- |
| alpha blend only | 197 |
| additive only | 156 |
| alpha and additive | 67 |
| alpha, additive and no-material nodes | 101 |
| additive and no-material nodes | 59 |
| alpha and no-material nodes | 20 |
| no-material nodes only | 66 |

These classes are counts for description. They are not names taken from the disc (Open question 133).

#### 2.9 Use of definitions

The definition table holds 666 definitions with 665 distinct ids (id 137 appears twice; section 2.7). Counted against every known reference in the disc data and the boot program, the ids fall into three groups:

- 358 ids are referenced in the static play-reach set: by a character clip's effect list, by a placed record in a world pack in the static reach graph, or by a program path that passes the id to an effect start or to the per-owner slot. An id can have more than one kind of reference. Story gates and physical access are not proven by this count.
- 3 ids are referenced only by data in statically unreached world packs (Open question 139).
- 304 ids have no known reference in these counts. 87 of them are held in the attack-family effect tables (below), which the hero attack routines read at run time; the counts here do not include those tables.

Counts per block of ids (distinct ids):

| Block | Referenced in static reach set | Only in statically unreached packs | No known reference | Total |
| --- | --- | --- | --- | --- |
| 0 to 999 | 199 | 0 | 180 | 379 |
| 1000 to 1999 | 68 | 2 | 104 | 174 |
| 2000 to 2999 | 52 | 0 | 7 | 59 |
| 3000 to 3999 | none | | | 0 |
| 4000 to 4999 | 37 | 1 | 6 | 44 |
| 5000 to 5999 | 2 | 0 | 1 | 3 |
| 6000 to 6005 | 0 | 0 | 6 | 6 |
| Total | 358 | 3 | 304 | 665 |

Of the 358 ids in the static reach set, 105 are referenced only by placed records. One now has a known script route: module 1249 uses slots 0 through 15, all of which map to definition 4006 in pack 1249. The other per-world slot-to-record mappings remain open (Open question 135). Program-side references account for 178 of the 358 ids (fixed arguments to an effect start or to the keyed lookup, or values chosen per case).

"No known reference" is not proof that an effect never plays. Two gaps decide most of it:

- A script StartEffect or StopEffect call names a placed-effect slot, not an effect id. For most calls the slot-to-record mapping is not resolved, so a placed record can be started without any visible id reference.
- An effect id computed at run time (for example from a table index or from an item, hazard or breakable record) does not appear as a fixed constant in the boot program, so it has no visible reference. The attack-family effect tables below are one such source.

An id with no known reference may therefore still play. The clean side must not treat these ids as dead data (Low for any claim that one never plays).

Attack-family effect tables (High for the layout and the counts; Medium-High for the readers' callers). The boot program holds ten tables of fourteen effect-definition ids at 0x315280 + 0x38 x k (k = 0 to 9), indexed by Mote id minus 160, and six tables of eight ids at 0x3154C8, 0x3154E8, 0x315508, 0x315528, 0x315548 and 0x315568, indexed by a slot and a variant (SPEC-0036, Attack-family effect tables, which gives the indexing, the Tor and Roberta pairs and the values for attack id -1). The hero attack routines read an id from them and start it through the id lookup of 2.7; 3.17 gives which routine reads which table and when. Together they hold 92 distinct definition ids; the fourteen-entry tables also hold the value -1 (no effect). Of the 92, 87 are in the no-known-reference group of the table above (48 in block 0 to 999 and 39 in block 1000 to 1999, ids 529 to 1513) and 5 are among the static-reach ids. Counting these tables as references gives 445 ids referenced by the static reach set and 217 with no known reference. These counts do not include the references of 3.17: a further fourteen-entry table at 0x315588 (ids 1300 to 1306) and the immediate ids 239 to 246 that the same driver starts (244 and 246 are Tor's swing glows).

Notable groups with no known reference, described in words (the definitions themselves are on disc):

- Ids 6000 to 6005: six near-identical copies of one effect. Each copy is a set of five emitters: three additive, one alpha and one with no material. Each emitter fires a single burst with a small particle count, and the particle lives are 2 to 3 s. The set looks like a multi-part burst or glow. They are the per-Mote effects of the Mote-eater drain (3.13), read from its id table at 0x2DCAE8 (one id per pair of Motes); the counts above do not include that table.
- Ids 147 to 181 (the range is not confirmed one id at a time): a family of dense short spark bursts. Each effect has three emitters, mostly alpha, each firing a single burst with many particles, particle lives of 0.5 to 3 s, and quick deceleration.
- Ids 244, 246 and 506 to 511: short additive glows. The emitters are mostly additive with particle lives of 0.1 to 2.5 s. Several of their emitters have a kick interval of 5 s, longer than their particle lives, so those particles are never kicked. Ids 506 to 511 are two-emitter effects.
- Other unreferenced effects are single or few-emitter pulses and bright streaks, for example ids 412, 432, 435, 4007, 4028, 4022, 4043 and 5000.

Three values that appear in program-side references are not definitions in this table: 297, 193 and 453 (High):
- 297 and 298, and 193 and 194, are not effect ids. They are ids in the common-object table of the shared effect manager (Where it lives). 297 and 298 are resource-9 models 14 and 15 (range 3). Their users take only the model's material group (the first mesh node's group word, SPEC-0004): group 11 for 297 (additive, PSMT4, 32 x 64) and group 12 for 298 (additive, PSMT4, 64 x 64). The swing trail (3.8) uses object 297's material and nothing else of it. 193 and 194 are range-2 indices 2 and 3, used by a world record kind (SPEC-0028).
- 453 is a real group-3 definition lookup, but resource 9 has no definition 453, so the lookup fails. The per-joint effect set of the character model object (12 pairs of effect objects, 24 in all; ids from the 24-word table at 0x3132A0, Where it lives) therefore binds nothing, and its update does nothing.
A fixed constant that matches an id in this table is still not proof that it names this definition; decide per reference.

Confidence: Medium for the reference counts; Low for the start routes and for any reading that an unreferenced id is never played.

### Pulse effect state (effect kind 3)

Run-time state of one pulse, per effect instance (the boot constants are in Where it lives):

| Field | Type | Meaning |
| --- | --- | --- |
| Phase A | float32 | Wrapping phase, 0 to 1. Starts at 0.0. Step 0.06 per update. |
| Phase B | float32 | Growing phase, no wrap. Starts at 0.0. Step 0.2 per update. |
| Item base records | 16 bytes each, stride 16 | Per item, the first float is the base value of that item's phase offset. Run-time data of the owning object. |
| Preset record | 16 bytes | Two of them. Flag zero selects the first; flag not zero selects the second. Byte 12 is the intensity written by the pulse. Other bytes are not written by the pulse. |
| Intensity floor word | float32 | Run-time word at 0x374D00, set to 64.0 once at boot by a start-up initialiser (Behavior 12.5). The pulse does not read it. |

## Behavior

### 3. Start, clock and spawning

#### 3.1 Starting an effect instance

1. A start request names an effect id (from a clip's list entry or a placed record). A script call names a placed-effect slot instead, not an id (section 6).
2. The instance takes the first free slot of the 10-slot pool. If none is free, the start is dropped and the attempt is logged; there is no retry.
3. The id is looked up in the definition table (first match). If there is no match, the instance is created without emitters and never produces particles. The clean side should log and continue.
4. The definition is copied into the instance as an effect object (the whole 0x90-byte record, 3.16): id, threshold, cull distance (kept at the instance's own value when the definition's value is 0 or less), cone flag, emitter list, multi-emitter flag and motion scale.
5. The instance's origin is computed from its attach slots (5.1). The instance's owner transform is set from the owner (5.1).
6. The distance and cone checks (5.2) run for the new instance on the first pass.
7. Spawners: the effect object's update (3.16) starts a spawner for each emitter slot that passes the gate (3.4). The nodes' pool words were written when the group was bound (3.16), not at the start.

Order (Medium): pool slot, definition lookup, copy of the definition, attach position and owner transform, then the update (distance check and spawners). For list entries, +0x3D selects the owner character's ground height, with an 8.0-unit probe fallback; when +0x3D is clear, +0x3C controls the probe (1.1; Open question 25 resolved).

Instance fields at creation: state near; elapsed 0; the cone direction from the attach slot; and the owner position from the attach matrix.

#### 3.2 Emitter cycle clock

Each emitter of an instance has a spawner with: state (1 init, 2 running, 3 paused, 4 dead, 5 draining), elapsed time (seconds), phase (fraction of a cycle, 0 to 1), current period p (seconds, or -1), cycle count, a count of live particles, its frame and position, a "previous" point and a "target" point, a per-tick displacement (delta), the motion scale S, an owner pointer and two hooks (a birth hook called for each particle born, and an end hook called when the spawner finishes, 3.16).

Init: the frame and position come from the effect (the effect frame times the node's local transform); the position is also copied to the previous and target points; S is copied from the effect (definition +0x80); p = A +0x6C + U(-A +0x70, +A +0x70) when A +0x6C >= 0, else p = -1. Start sets state 2. Resume turns state 3 back into 2. An effect-object position update copies the new position to each live spawner but leaves the separate target point unchanged. The target point (spawner +0x80) is written by the burst rule (a mode 3 spawner copies its position to it after each burst) and by two loops of the hero attack-effect driver that write it into every live spawner of an effect object (3.17); the standalone target-vector setter has no caller and no data reference. The inherited-velocity setter copies its vector to each live spawner; particle birth adds it when the definition's follow flag is clear (4.1), while the follow path adds the spawner's per-tick displacement instead (4.2).

Each tick (dt = 1/60 s), for a running spawner with an A record, in 32-bit float arithmetic:

1. **Negative period** (p < 0): launch one burst; then, if A +0x86 = 0, drain. Every disc record with a negative base period has A +0x86 = 0 (330 records), so such an emitter fires once and its spawner dies when its particles have expired (fire-and-forget).
2. **Otherwise**: the elapsed time grows by dt and the phase by dt / p. A new cycle starts when the phase reaches 1 or more, or on the first running tick (elapsed was 0 before this tick). At a new cycle: a new period p' = max(0, A +0x6C + U(-A +0x70, +A +0x70)) is drawn; the phase is rescaled by p / p' and raised to 1 if it is below 1; its whole part is added to the cycle count and it keeps its fraction; p' becomes the current period. A burst is then launched if A +0x86 = 0 or the spawner has no live particle.
3. **End rule** (every tick, not only at a boundary): if the cycle count is below A +0x74, or A +0x60 is negative, the spawner keeps running. Otherwise, if A +0x86 = 0, it drains. Otherwise it keeps running.
4. Always: delta = position - previous point; previous point = position.

Drain: with no live particle the spawner is dead at once; otherwise it enters state 5 (delta zeroed, no more bursts) and, if A +0x86 = 1, the last-born particle's elapsed time is set to its life, so it ends on the next tick. A draining spawner with no live particle becomes dead on its next tick and is freed; the end hook runs then.

Burst: A +0x64 births, stopping early when the particle pool is empty; each birth calls the birth hook. If A +0x80 = 3, the target point is then set to the current position.

Each burst particle is born as in 4.1. The spawner's state gate (A +0x84) is applied when the effect object starts its emitters (3.16).

Confidence: High for the steps (test vectors 70 to 72). Float32 matters: a 0.1 s period bursts on ticks 1, 8, 14, 20, ... (Edge cases).

#### 3.3 Random draws

The random source is a Mersenne Twister (MT19937, 32-bit output). Uniform draws are U(lo, hi) = lo + (hi − lo) × (u / 2^32), with lo = hi returning lo. The generator is seeded on its first draw from the console clock, so sequences differ per boot (SPEC-0008). The clean side should use any good generator; sequences cannot be reproduced exactly. The random parts of particle birth and motion each take one uniform draw: the range is from -spread to +spread for every jitter and spread (symmetric).

Each call to the uniform helper takes exactly one 32-bit draw from the shared generator and maps it by the formula above, so the result always lies in the half-open range from lo up to hi (Medium-High). A draw of u = 0 gives lo.

#### 3.4 Spawner creation per node

Pool words (High). When a group is bound (3.16), every node of the group receives its spawner pool (+0x40), its particle pool (+0x44) and its shader-list context (+0x48), and its material index is resolved (2.1 step 5). For shared group 3 these are the global pools (300 spawners, 800 particles). The words are zero on disc and in an unbound group.

Gate (High for the gate). The emitter slots are tried in order, only while the slot index is below the definition's emitter count n. A slot is tried when it has a node, that node's spawner pool word is non-zero, and its A +0x84 gate equals the effect object's state (3.16). When the multi-emitter flag (definition +0x4C) is 0, the walk stops after the first emitter that starts; when it is non-zero (all disc definitions), every matching slot is tried. Then:

1. Take a spawner from the node's spawner pool. If none is free, that emitter is skipped.
2. Initialise it (3.2) from the node's records (A, B and C, 2.4 to 2.6), the node's transform, the effect position and frame, and the effect's motion scale S.
3. Give it the owner pointer, the birth hook and the end hook. The end hook clears the effect object's slot when the spawner finishes.
4. Start it (state 2). The start happens before the handle test below.

Handle (Medium-High). The effect object keeps the spawner in one of its five live slots only when the node's A record base period (A +0x6C) is zero or positive. For a negative base the spawner is still started, fires its single burst and dies on its own (3.2 step 1); the object does not track it. If at least one spawner is kept, the instance state becomes running (1). If none is kept, the state stays idle (0).

#### 3.5 Single emitter start with a callback

A helper starts one emitter from a node with a callback (Medium): it takes a spawner, initialises it from the node, sets one hook slot, two data words and an owner pointer, and starts it. The hook names and behaviour are not known (Open question 26). The helper is used by the callback path of an effect start; the list path uses 3.4.

#### 3.6 Node-block start (stride 0x70)

An effect-list event may start a block of emitter nodes directly (Medium). The block uses the same node stride as a definition's node array (0x70 bytes). For each node in use, the clean side creates one spawner from the node's records (as 3.4), gives it the owner, and starts it.

#### 3.7 Per-owner slot manager

The checked character effect-request path uses one cached effect-instance slot at +0x570 of the effect-request object; the character is a separate input to the request, not the base of that slot (Medium-High for this path). When the request has a valid effect id:

- If the slot is empty, the instance is bound and started (3.1).
- If the slot holds an instance whose id equals the requested id, it is refitted with the current request transform (5.1), and the distance check (5.2) runs.
- If the slot holds a different id, the old instance is released before the new definition is bound into the slot.
- The no-id sentinel is -1; it releases the current instance. A failed lookup also follows the release path.

The checked request path obtains its effect-request object from one initialized singleton accessor and passes the character separately. This establishes a shared slot for this path, not a per-character slot. Other request paths and any relationship between this slot and the separate effect-object pool remain open (Open question 28).

Shared effect manager (High). The effect manager is one shared run-time block (Where it lives), built once when it is first used. It holds four group slots of 0x38 bytes each. Only slot 3 is loaded and bound: it holds the resource-9 definition table, and binding ties every node of that group to the global pools (3.4, 3.16). The same start-up item-stream load retains resource 9's type-2 shader-list payload in the shared manager's slot-3 shader-list pointer. When slot 3 is bound, that pointer is passed as the material context; each nonnegative node material index is replaced by the corresponding group pointer at the relocated group table plus index × 0x270 (2.8). A request by id scans the group's definitions in order and takes the first match. Slots 0 to 2 have no known use. When the block is built, fixed words are written into a table in the block: they are the range table of the common-object table (Where it lives), global model ids, not effect ids. The checked character request path's cached instance slot is described above; its relationship to the separate effect-object pool remains open (Open question 28). The swing-trail start (3.8) keeps its trail in a per-holder slot of the animation-effect pool element, which is a separate slot.

#### 3.8 Swing trail (element-coloured ribbon)

Confidence: High for the flow, the gate, the kind source, the record mapping and the ageing gate; Medium-High for the sample and colour rules; Medium for the draw.

This start path does not start a particle effect. It starts a swing trail: a ribbon drawn between two edge points (a weapon or limb swing) that are sampled once per pass. The kind is the attacker's element, and it picks the trail's colour gradient (section 18).

- Holder: an element of the animation-effect pool (10 elements of 0xB0 bytes, 5.1). Element +0x08 is the clip effect-list entry, +0x0C the character, +0xA0 the trail active word and +0xA4 the trail handle.
- Caller: the pool start of a list entry (1.2), for every entry, after the definition bind and the positional step. The kind is passed down through the clip walk:
  - character base class: kind 0 always;
  - enemy class (monster row in use, SPEC-0029): kind = the row's target type T (u16 at row +0x3C); while an attack slot k (0 to 3) is selected, kind = that slot's damage alignment (s32 at row +0x3BC + 4k).

Steps:
1. The trail manager always exists: its pointer word 0x35C8A0 is set once at start-up (Where it lives). It holds 12 trails, each a ring of 18 two-point samples of 0x50 bytes.
2. Record by kind: kinds 0, 1, 2, 3, 4, 5, 6 and 7 take records 0, 1, 2, 6, 3, 4, 5 and 7 of the eight records of section 18 (0x370DF0 to 0x37106F). Any other kind takes record 0.
3. Gate: the trail starts only when the entry's byte +0x48 is 0 (1.1). On disc this is 325 of the 1,863 entries, in 120 character regions, and all 325 have effect id -1 and attach slot B set. Other values (255 in 1,533 entries, and 255 with +0x49 = 1 in 5) start no trail; the entry's effect id path is unchanged.
4. If the holder's active word (+0xA0) is non-zero, nothing is created.
5. A trail is taken from the manager's 12. If none is free, nothing happens. Its material is the material group of common object 297 (2.9: group 11, additive, PSMT4, 32 x 64). Its handle goes to +0xA4; the trail keeps a link to the holder's active word and clears it at creation.
6. Start: the first sample is taken with the two edge points, and the trail writes 1 into the holder's +0xA0.

- Attach point for the held-weapon trail (High): "matrix slot 0 of the model" is bank entry 0, which the per-frame render update fills from pose-table entry 0, so it is attach point 0 (section 5.1). The reach L is given in the held-weapon paragraph below.
- Edge points: point 1 = attach slot A's matrix (entry +0x20) applied to the entry's vector at +0x00 with w = 1; point 2 = attach slot B's matrix (entry +0x24) applied to the entry's vector at +0x10 with w = 1 (5.1).
- Companion update: each pass, while the holder has both words set, one sample is appended with the two current edge points.
- Release: when the entry's stop window is reached (1.2), the pool element is released and its +0xA0 and +0xA4 are cleared. The trail gets no more samples; its samples age out and the trail ends by itself. The release does not end the trail. The trail ends in the first pass in which none of its sample pairs has a live sample (a sample whose age has been set to -1 on retirement): the draw step then clears the trail's active word and the link to the holder's active word, and the trail is freed in that same pass. With cap n = 18 and one step per normal-play pass, the last sample retires after 19 steps and the trail ends in that pass (Medium for the count of passes).

Samples and colour (record fields as in section 18: colours A, B and C, count c at +0x30, cap n at +0x34, fade distances d0 and d1 at +0x38 and +0x3C, steps at +0x40, u step at +0x44):
- A new sample holds the two points, width = |p1 - p2|, colour = A, age 0, and u = the previous sample's u + D x (+0x44), with D from the fade rule below.
- Fade: when sample s + 1 is appended, D = max(|p1(s+1) - p1(s)|, |p2(s+1) - p2(s)|). Sample s gets the fade F = 0 when D <= d0, 1 when D >= d1, else (D - d0) / (d1 - d0).
- Ageing, once per trail update: the step count is 1 when a sample was appended since the last update, else the record's +0x40 (1 in every record). Each step, for each live sample: age += 1. If age > n, the sample is retired: its colour becomes C and it ages no further. Otherwise colour += (B - A) / c while age <= c, or += (C - B) / (n - c) after. Then alpha = alpha x F (every step, so it compounds), and the colour is clamped between the two run-time limit vectors (0x42BE30 low, 0x42BE20 high). A start-up initialiser sets them to (0, 0, 0, 0) and (255, 255, 255, 255) and nothing else writes them, so the clamp only keeps each channel within 0 to 255 (High). If the trail's override word (+0x68) is non-zero, R, G and B are replaced by the bytes at trail +0x64, +0x65 and +0x66; the word is cleared when a trail is created and nothing sets it, so the override never applies (Medium-High).
- Ageing gate (High for the gate; Medium for the step's place in the pass): the trail pass runs inside the effects render pass on every main-loop pass. For each live trail, the age-and-colour step runs before drawing only when the game-state code is 8 or below, which is normal play. At code 9 or above (pause and the other freezing screens, SPEC-0026) the step is skipped, so samples keep their age and colour while the game is frozen. Drawing happens in both cases; each draw also rebuilds the sample texture coordinates and fade, which is not the colour step. The trail remains drawn (4.6).
- Draw (Medium): the trail is drawn as quads between consecutive live samples with the sample colours and the additive material of object 297; texture u comes from the sample u, with a wrap that uses the record's +0x48 (0 in every record).

The trail kinds (elements) are unrelated to effect kind 3 of the shared list (section 12).

Held-weapon and other trails (High for the records and the requests; Medium for the edge points). The hero never runs the clip walk (Edge cases), so the hero's visible swing trail comes from the character model instead. Each tick the owner may raise a trail request on its character model; the model's tick consumes the request (clears it) and, when the model has a held object, keeps one trail of the manager going with record 0x392790 and the material of common object 297. Its edge points are: point 1 = B + 0.25 D and point 2 = point 1 + L D, with B the weapon base point (the translation of matrix slot 0 of the model, attach point 0), D the held object's direction vector (the third axis of attach point 0, the same vector as Z0 in 3.17) and L the reach length. For a hero owner L = 0.6 x W, with W the weapon reach of SPEC-0021 (1.5 with no weapon, so L = 0.9); for an enemy owner L = 1.5, whatever the enemy (Medium-High: the enemy classes use the base class value). Example: hero Tor with item 98 (W = 1.7, L = 1.02), B = (10, 0, 5), D = (0, 0, 1): point 1 = (10, 0, 5.25), point 2 = (10, 0, 6.27). With no weapon: point 2 = (10, 0, 6.15). Enemy: point 2 = (10, 0, 6.75). The Black Knight's own trail update uses the same two-step form with its own length (SPEC-0012). When no trail is held yet (or the old one ended) a new one is taken; otherwise one sample is appended.
- Hero request: on every tick in which the hero's attack-active word (the animation event bit 0x1 of any record, SPEC-0036) is set, during normal hero control. Enemy request: on every tick in which any of the animation event bits 0x1, 0x2, 0x4, 0x8 or 0x10 is set. The character base class never requests one.
- The Black Knight's sword uses the same record. A two-sprite marker (two trails) uses record 0x3704F0 and the enemy projectile uses record 0x3857C0, both with the material of common object 298.
- Records (all written by start-up initialisers; colours as (R, G, B, alpha) for A, B and C; then c, n, d0, d1, steps, u step):

| Record | A | B | C | c | n | d0 | d1 | steps | u step |
|---|---|---|---|---|---|---|---|---|---|
| 0x392790 (held weapon) | (128, 128, 128, 32) | (128, 128, 128, 10) | (128, 128, 128, 0) | 2 | 14 | 0.02 | 0.3 | 1 | 2.8 |
| 0x3704F0 (marker) | (255, 0, 0, 0) | (255, 0, 0, 78) | (255, 0, 0, 0) | 2 | 9 | 0.02 | 0.3 | 2 | 0.09 |
| 0x3857C0 (projectile) | (128, 128, 128, 0) | (128, 128, 128, 78) | (128, 128, 128, 0) | 2 | 9 | 0.02 | 0.3 | 2 | 0.09 |

Test vectors (computed):
- Enemy with row T = 3 and no attack slot: record 6 (0x370FD0, yellow). The same enemy with slot 1 selected, whose alignment is 4: record 3 (0x370EE0, blue). Kind 9: record 0. Character base class: record 0.
- List entry with +0x48 = 255: no trail; the effect id path is unchanged.
- Colour (record 0: c = 2, n = 14, alpha A 48, B 40, C 2, F = 1, one step per update): alpha 44 at age 1, 40 at age 2, 21 at age 8, 2 at age 14; at age 15 the sample is retired with alpha 2.
- Fade with d0 = 0.02 and d1 = 0.3: D = 0.16 gives F = 0.5; D = 0.02 gives 0; D = 0.3 gives 1. With F = 0.5, alpha 48 at age 0 becomes (48 - 4) x 0.5 = 22 at age 1 and (22 - 4) x 0.5 = 9 at age 2 (the factor compounds; the clamp limits are taken not to bind).
- Ageing gate: with game-state code 8, a sample's age and colour are unchanged after the next pass; with code 9, they advance, and the trail is still drawn in both cases.

#### 3.9 Effect type setup (27 world-object emitter presets)

Confidence: High for the arithmetic, the role, the table locations, the record layouts and the copy rules; Medium for the launch speed taken from the direction length. The size rows, C sources and B sources are not given here: the clean side reads them from the user's boot executable (Where it lives). The five header words of each preset are given below (preset header values), by user decision.

The 27 presets are emitter types for the emitter nodes of world-object parts (type-29 objects, SPEC-0004). A node whose u16 at +0x64 (2.3) is 1 to 27 takes its C, A and B records from the preset of that type, at load. The resource-9 table never uses them: the u16 at +0x64 is 0 on all 1,651 of its nodes.

Disc census of serialized type-29 emitter nodes (all 192 world-pack resources): 121 resources contain one type-29 chunk each. Their secondary owner tables contain 2,029 nonempty node arrays, holding 14,774 node records. Every array and node record fits within its containing chunk. All 14,774 type words are in the applied range 1 to 27; observed values are 1, 2, 3 and 5. The counts below are node records, distinct owner records with at least one such node, and packs containing at least one such node. They describe serialized emitter nodes; they do not establish a one-to-one correspondence with rendered mesh parts or live play reachability.

| Type word | Emitter nodes | Owner records | Packs |
| --- | ---: | ---: | ---: |
| 1 | 11,025 | 2,017 | 79 |
| 2 | 1,965 | 1,965 | 78 |
| 3 | 1,610 | 1,610 | 68 |
| 4 | 0 | 0 | 0 |
| 5 | 174 | 174 | 6 |
| 6-27 (each) | 0 | 0 | 0 |

A-record total census for the 14,774 serialized type-29 preset nodes: every referenced total is nonzero. Type 1 has total 1 (11,025 nodes); type 2 has total 9 (1,937) or 10 (28); types 3 and 5 have total 20 (1,610 and 174). Thus no shipped world-object preset node reaches the cycle-limit division with a zero total. This is a serialized-node fact; it does not claim which packs are loaded during play.

1. Index i = type word minus 1. If the index is outside 0 to 26, nothing is done and the node keeps its records.
2. The node's three record references (+0x50 C, +0x54 A, +0x58 B) are set for that index. A missing record (reference zero) is set to record i of the matching first-time table (record sizes 0x50 for C, 0x90 for A and 0x90 for B). A present record keeps its storage and is overwritten for the index: C by the C conversion of C source i, A by the A copy of preset i, and B by the B copy of B source i (the rules below; Where it lives gives the tables).
3. For a preset node, A +0x60 holds the total particle count. A +0x64 (particles per burst) becomes the smaller of A +0x60 and the type's cap. The cap is the s32 at +8 of the type's 0x80-byte preset record.
4. A +0x74 (the cycle limit) becomes A +0x60 divided by A +0x64, with whole-number division.

Example (synthetic cap): type word 5 (index 4) with a cap of 5. A total of 30 gives 5 particles per burst and a cycle limit of 6. A total of 3 gives 3 per burst and a cycle limit of 1. A total of 0 divides by zero (Edge cases).

Example (game cap): type word 5 (index 4) has the cap 4 (preset header values). A total of 30 gives 4 particles per burst and a cycle limit of 7 (30 / 4 = 7 in whole numbers).

A node with a missing A record shares first-time A record i with every other such node of the same type, and steps 3 and 4 write into that shared record. Its total count A +0x60 is the default -1, so A +0x64 becomes -1 and A +0x74 becomes 1 (Medium).

Each preset is an ordinary emitter description: a C record (motion), an A record (timing, life and size rows, with a cap) and B curve rows (colour), used exactly as sections 2.4 to 2.6 and 3.2 to 4.6 describe.

**Preset record (0x80 bytes; record i at 0x3711B0 + 0x80 i; run-time, filled at start-up).**

| Off | Type | Role | Filled from | Confidence |
| --- | --- | --- | --- | --- |
| +0x00 | f32 | base burst period in seconds; copied to A +0x6C | start-up code (preset header values below) | High |
| +0x04 | f32 | period jitter in seconds; copied to A +0x70 | start-up code | High |
| +0x08 | s32 | cap: copied to A +0x64, then the bound of step 3 | start-up code | High |
| +0x0C | s32 | copied to A +0x68 (role not given in 2.4) | start-up code | High |
| +0x10 | f32 | particle life in seconds; copied to A +0x78 | start-up code | High |
| +0x14 to +0x1F | - | never read: the only readers of a preset are the A-record copy and the type setup, which take the fields listed here | start-up code or zero | High |
| +0x20, +0x30, +0x40 | vec4 each | size rate rows 1, 2 and 3; copied to A +0x10, +0x20 and +0x30 | size rate row tables 1, 2 and 3, row i | High |
| +0x50 to +0x5F | - | never read (see above) | start-up code or zero | High |
| +0x60 | vec4 | size start vector; copied to A +0x00 | size start row table, row i | High |
| +0x70 to +0x7F | - | never read (see above) | zero | High |

**Preset header values (High; words +0x00 to +0x13 of each preset record).** These values are published here by the user's approval, as a limited exception to the rule against copied tables; the clean side uses them as constants. Each float is written with enough decimal digits to give back the exact 32-bit float. Row i is preset record i at 0x3711B0 + 0x80 i, used by type word i + 1. Base period -1.0 means a single burst (3.2).

| Type word | Index i | +0x00 base period (s) | +0x04 period jitter (s) | +0x08 cap | +0x0C A +0x68 word | +0x10 life (s) |
| --- | --- | --- | --- | --- | --- | --- |
| 1 | 0 | -1.0 | 0.0 | 1 | 0 | 5.0 |
| 2 | 1 | -1.0 | 0.0 | 50 | 0 | 1.2 |
| 3 | 2 | 0.12 | 0.11 | 6 | 3 | 0.8 |
| 4 | 3 | 0.0 | 0.0 | 1 | 0 | 0.9 |
| 5 | 4 | -1.0 | 0.0 | 4 | 0 | 2.2 |
| 6 | 5 | -1.0 | 0.0 | 1 | 0 | 0.22 |
| 7 | 6 | -1.0 | 0.0 | 1 | 0 | 0.4 |
| 8 | 7 | 0.11 | 0.03 | 6 | 3 | 1.4 |
| 9 | 8 | 0.11 | 0.03 | 6 | 3 | 0.6 |
| 10 | 9 | 0.11 | 0.03 | 6 | 3 | 0.7 |
| 11 | 10 | -1.0 | 0.0 | 1 | 0 | 1.7 |
| 12 | 11 | -1.0 | 0.0 | 1 | 0 | 0.7 |
| 13 | 12 | -1.0 | 0.0 | 1 | 0 | 0.9 |
| 14 | 13 | -1.0 | 0.0 | 1 | 0 | 1.9 |
| 15 | 14 | -1.0 | 0.0 | 1 | 0 | 1.9 |
| 16 | 15 | -1.0 | 0.0 | 1 | 0 | 1.9 |
| 17 | 16 | -1.0 | 0.0 | 1 | 0 | 1.9 |
| 18 | 17 | -1.0 | 0.0 | 1 | 0 | 1.9 |
| 19 | 18 | -1.0 | 0.0 | 1 | 0 | 1.9 |
| 20 | 19 | -1.0 | 0.0 | 1 | 0 | 1.1 |
| 21 | 20 | -1.0 | 0.0 | 50 | 0 | 0.4 |
| 22 | 21 | -1.0 | 0.0 | 50 | 0 | 1.2 |
| 23 | 22 | -1.0 | 0.0 | 50 | 0 | 0.5 |
| 24 | 23 | -1.0 | 0.0 | 10 | 0 | 0.4 |
| 25 | 24 | -1.0 | 0.0 | 10 | 0 | 0.4 |
| 26 | 25 | 0.1 | 0.0 | 20 | 1 | 0.6 |
| 27 | 26 | -1.0 | 0.0 | 4 | 0 | 7.2 |

Value summary: 21 presets have the base period -1.0 (single burst), one (type word 4) has 0.0, and five (type words 3, 8, 9, 10 and 26) have 0.1 to 0.12 s. Lives run from 0.22 s to 7.2 s.

**A copy (preset to A record).** A +0x00 = preset +0x60; A +0x10 = preset +0x20; A +0x20 = preset +0x30; A +0x30 = preset +0x40 (16 bytes each); A +0x64 = preset +0x08; A +0x68 = preset +0x0C; A +0x6C = preset +0x00; A +0x70 = preset +0x04; A +0x78 = preset +0x10. Every other A field keeps its value.

**C source record (0x50 bytes, twenty 32-bit words; record i at 0x2DBBA0 + 0x50 i).** All words are f32 except +0x3C. The C conversion writes each float into its 8.8 field by the write rule of 2.6.

| Source off | Type | Written to | Confidence |
| --- | --- | --- | --- |
| +0x00, +0x04, +0x08 | 3 f32 | direction vector v. C +0x00 becomes the unit vector v / length(v) with w = 1.0, and C +0x28 (launch speed) becomes length(v) | High for C +0x00, Medium for C +0x28 |
| +0x0C | f32 | C +0x2A (launch speed spread) | High |
| +0x10 | f32 | C +0x24 (cone polar angle, upper) | High |
| +0x14, +0x18, +0x1C | 3 f32 | C +0x10 = (x, y, z, 0), the extra acceleration term | High |
| +0x20 | f32 | C +0x20 (vertical acceleration) | High |
| +0x24 | f32 | C +0x22 (drag) | High |
| +0x28 | f32 | C +0x2C (acceleration clock T) | High |
| +0x2C | f32 | C +0x2E (clock spread) | High |
| +0x30 | f32 | not read | High |
| +0x34 | f32 | C +0x30 (spin rate) | High |
| +0x38 | f32 | C +0x32 (spin spread) | High |
| +0x3C | s32 | byte C +0x3E: the value when it is 2 or 3, otherwise 0. Byte C +0x3F (spin axis) is set to 0 | High |
| +0x40 | f32 | C +0x34 (kick strength R) | High |
| +0x44 | f32 | C +0x36 (kick spread) | High |
| +0x48 | f32 | C +0x38 (kick interval) | High |
| +0x4C | f32 | C +0x3A (kick interval spread) | High |

C +0x26, +0x3C, +0x3D and +0x40 are not written by the conversion. Two of the 27 C sources (indices 5 and 6) have a zero direction vector. The conversion then divides by a zero length; on the original hardware the result is C +0x00 = (0, 0, 0, 1) and a launch speed of 0, and the clean side must give the same result instead of failing (Medium).

**B source record (0x34 bytes, thirteen f32; record i at 0x2DC410 + 0x34 i).** +0x00 = key time t1 (a life fraction); +0x04, +0x08, +0x0C = alpha a0, a1, a2; +0x10, +0x14, +0x18 = red r0, r1, r2; +0x1C, +0x20, +0x24 = green g0, g1, g2; +0x28, +0x2C, +0x30 = blue b0, b1, b2.

**B copy (B source to B record; High).** Row 0 (B +0x00) = (r0, g0, b0, a0). Row 1 (B +0x10) = (r1, g1, b1, a1). Rows 2 to 5 (B +0x20, +0x30, +0x40, +0x50) all = (r2, g2, b2, a2). The six RGB key times (B +0x60 to +0x77) become 0, t1, 1.0, -1.0, -1.0, -1.0. The alpha key times (B +0x78 to +0x8F) are not written.

**Start-up fill (High).** Once at boot, in this order:
1. A start-up initialiser (one of the start-up table entries of SPEC-0026, called with (1, 0xFFFF)) clears each of the 27 preset records and writes its header words (+0x00 to +0x13) and some further words, from values held in its own code. It then builds the 81 first-time records with their defaults. First-time C: C +0x00 = (0, 0, 0, 1), C +0x10 = (0, 0, 0, 0), the 8.8 field C +0x38 = raw -1, every other 8.8 field and byte zero. First-time A: A +0x00 = (1, 1, 1, 1), the three size rate rows (0, 0, 0, 1), A +0x60 = -1, A +0x64 = 1, A +0x6C = -1.0, and A +0x68, +0x70 to +0x80 and the four flag bytes zero (the threshold vectors stay zero). First-time B: all six rows (0.5, 0.5, 0.5, 1.0) and all twelve key times -1.0.
2. Later in start-up, the effect start-up step converts C source i into first-time C record i, for i = 0 to 26.
3. It then writes, for each i, preset i's +0x60 from size start row i and its +0x20, +0x30 and +0x40 from size rate rows 1, 2 and 3 (row i), and applies the A copy of preset i to first-time A record i.
4. It then applies the B copy of B source i to first-time B record i, for each i.

Rule for the clean side: read the six source tables from the user's boot executable (Where it lives) and perform steps 1 to 4 with them. The header words of step 1 are the constants of the preset header values table above.

#### 3.10 Timed emitter

Confidence: High for the steps; Medium-High for the meaning. It is the enemy class's stolen-item display: the enemy shows the item it took from the hero for 3 seconds.

An object has an enable word at +0xC24 and a remaining count at +0xC2C. While the enable word is non-zero, each tick does this:

1. If the count is below 1, the enable word is cleared and nothing else happens on that tick.
2. Otherwise three 16-byte vectors are read from a sub-record that the object reaches through its link at +0x170 (at that sub-record's +0x1400, +0x1410 and +0x1420). Each vector is multiplied by 0.45. The id at +0xC30 is an item id. It is looked up in the item model registry (SPEC-0021), not in the definition table of this spec. If a model is found, it is drawn once on that tick with the frame made of the three scaled vectors and a scale of 1.0, so the item appears at 0.45 of its size on the enemy's frame. No particles are made.
3. The count goes down by 1, whether or not an entry was found.

A count of N therefore gives N spawn attempts on N consecutive ticks. The enable word is cleared on the tick after the last one.

Start (Medium-High). The only writer is the enemy class's hit on the hero. When the enemy's kind has its steal flag and a random draw passes the kind's steal chance, the enemy either takes Neopoints (dropped again as coin items, SPEC-0021) or takes one item from the hero's inventory. For an item it sets the enable word to 1, stores the item id at +0xC30 and sets the count to trunc(3.0 x 60) = 180 ticks. Which kinds carry the steal flag is SPEC-0029's.

#### 3.11 Timed entry list

Confidence: High for the steps, the follow-camera owner and the 19-entry maximum; Medium-High for the stale-slot detail (Edge cases).

The follow-camera object keeps a list of contact normals from its collision pass. Its focus-character link is at +0x70, count at +0x500, and each entry is a 0x20-byte record whose countdown/tag is at +0x290 plus 0x20 times its index. The camera update visits entries in order:

1. The focus character's owner callback is queried. If the answer is non-zero, or the camera's flag at +0x26C is non-zero, the entry's countdown goes down by 1. The flag is on the camera itself: the follow update sets it to 1 when the shaped right-stick drift base is nonzero, and to 0 when that base is zero, including the streaming-load freeze path.
2. If the countdown is now below 1, the entry is removed. The entries after it move down one place, and the count goes down by 1.

The contact-normal accumulator refreshes an existing entry when its normal matches above 0.999; otherwise it appends only while the count is below 19. At 19 entries it logs and leaves the list unchanged. Thus the static maximum is 19. The loop details that matter for a rebuild are in Edge cases.

#### 3.12 Rumble envelope segments (High for representation, gate, join and release; Medium-High for output selection)

A rumble request turns its selected preset into up to three linear magnitude segments, each stored in one of six fixed 0x18-byte manager slots. A segment has an in-use word at +0, a state word at +4, remaining duration in ticks at +8, magnitude change per tick at +0xC, current magnitude at +0x10 and a link to the next segment at +0x14. The first preset interval is marked enabled when it receives a slot; later linked intervals wait until their predecessor expires. Each update advances enabled segments, and expiry enables the linked successor. If several requests have enabled segments together, the updater selects the greatest current magnitude and passes it with that segment's remaining ticks to the vibration state. The magnitude is clamped to 0 through 1 and converted to a byte.

Joining segment A into the segment B that A links to:

1. Both segments must be in use, A must link to B, A's start value plus its slope times its duration must be within 0.001 of B's start value, and the two slopes must be within 0.001 of each other.
2. If so, B takes A's start value, its duration becomes A's duration plus its own, and its state becomes 1 if either state was set (0 otherwise). A's fixed slot is cleared to all zero words and becomes available for reuse; B keeps its successor link. The join returns 1.
3. Otherwise nothing changes and the join returns 0.

The state word gates per-tick progression and output. The first preset interval is enabled during construction when allocated; linked successors are enabled on predecessor expiry. The fixed six-slot scan and clear-to-zero release are part of the segment lifecycle.

#### 3.13 Mote-eater drain (eight slots)

Confidence: High for the steps and the words; Medium-High for the charge use. The missile that triggers it belongs to the Faerie Sisters (SPEC-0011, enemy-pool missile kind 14); the Mote charge rules are SPEC-0021's.

The drain is one object (built on first use) with a body effect and up to eight Mote slots of 0xB0 bytes. When a Mote-eater missile hits the hero and is not deflected, and the drain is not already active:

1. Pick 8 Mote indices from a 12-entry list (Mote ids 160 to 171). The first is a uniform whole number from 0 to 11. Each next pick is drawn the same way; while it equals an earlier pick it becomes (pick mod 12) + 1. Quirk: a walk that passes 11 gives 12, which reads one past the end of the list and yields a value that is not a Mote; that pick adds no slot. So the drain has 7 or 8 slots.
2. Each picked Mote gets a slot (at most 8) with the effect from the per-Mote id table at 0x2DCAE8 (14 s32 words indexed by Mote id - 160; one id per pair of Motes, definitions 6000 to 6005, and -1 for the last pair), at the scale word 0x2DCAD8 = 0.5.
3. Start: the body effect, definition 464, at the hero position + (0, 4.5, 0) (word 0x2DCACC), with no listener. Countdown N = trunc(30.0 x 60) + 1 = 1801 (duration word 0x2DCAC8, ticks per second word 0x368FA0). Period P = N / slots in whole numbers, at least 1: 225 for 8 slots, 257 for 7.

Each tick call:
- countdown -= 1. At 0: everything stops and the drain becomes inactive.
- Otherwise the body follows the hero, and every revealed slot follows the hero + its offset.
- When countdown mod P = 0 and fewer than 8 slots are revealed: the next slot is revealed at the offset (U(-a, a), 4.0, U(-a, a)), with a = word 0x2DCAD4 = 0.0 and 4.0 = word 0x2DCAD0, and its effect starts. If the switch word 0x2DCADC (1 on disc) is non-zero, 20 charge (word 0x2DCAE0) of that Mote is used (SPEC-0021 Mote charge; a Mote that is not held or is spent is cleared from the equipment words).
- Caller: each live Faerie Sister's per-tick thread calls the tick once. Quirk: with k sisters alive the drain advances k counts per tick, so it runs k times faster (Medium-High).

Leach hit (enemy-pool missile kind 15, SPEC-0011): effect 4039 plays once at the hero's model point (no listener), then 50 MP is drained (word 0x2DD458; MP is floored at 0 and the MP-empty hook runs).

Test vectors (computed):
- 8 slots: reveals on calls 1, 226, 451, 676, 901, 1126, 1351 and 1576; everything stops on call 1801.
- 7 slots (P = 257): the first reveal is on call 2 (countdown 1799 = 7 x 257), the last on call 1544; the stop is still on call 1801.
- Two sisters alive (8 slots): the last reveal is on tick 788 and the stop on tick 901.

#### 3.14 Hover state machine (pointer)

The hover machine is the item showcase of SPEC-0021 (Item showcase): the same object, rates and bounds. State 4 is the full showcase. Coins (item ids 135 to 144) stop the grow at 0.4 and skip the hold (High). SPEC-0021 owns the rules.

#### 3.15 Item effect mapping

Confidence: High for the mapping and the world-item use; Medium for the second caller. The number mapped is an item id (SPEC-0021), not an attack kind, and is unrelated to the eight kinds of section 3.8. (Earlier text called these attack kinds; the combat spec, SPEC-0036, covers the attack records).

When a world item (a dropped or placed pickup, SPEC-0021) is set up, a routine maps its item id to an effect id in the definition table (section 2) and binds that definition into the item's own effect object (3.16):

- Sixteen item ids map to ids in the range 759 to 786: 13 to 778, 14 to 783, 19 to 776, 20 to 777, 31 to 786, 57 to 771, 58 to 772, 59 to 773, 62 to 780, 135 to 761, 136 to 760, 137 to 759, 138 to 767, 147 to 770, 151 to 775 and 154 to 774.
- Fourteen item ids, 0xA0 through 0xAD (the Motes, 160 to 173), map to the ids 951 to 964, in order: the first gives 951, the next 952, and so on up to 964.
- Any other item id takes the default id 792 (the routine's second test always passes).
- Item 0x8B (139) maps to no effect.

Use in play (High). Each tick of a world item: while the item is active its effect object is placed at the item position raised by the item model's bound offset plus half its height (y only), its motion scale S is set to 1.5, and it is updated with the camera as the listener; when the item is no longer active the object is stopped. The family equipment change also binds the mapped effect into its own effect object (Medium).

Look (resource 9): 761 (item 135) is one additive 64 x 64 sprite emitter, size 1.5, one particle every 0.05 s, life 0.4 s, thrown in all directions at 4 +- 1 units/s with drag 5, colour (1, 0.64, 0.47); 759 (item 137) is the same in (1, 1, 0.64). Both have the near/far threshold 10000 (100 units), no cull and the cone test on. 792 is a slow grey (0.75) puff of size 2.5 every 4 +- 0.2 s, life 2 s. The Mote effects (951 to 964) have five emitters, for example 951: near and far sparks, two alpha discs of size 3 and a looping light emitter of radius 15 and colour (1, 0.5, 0), threshold 22500.

Test vectors: item 0xA3 gives id 954. Item 0x8B gives no effect. Item 135 gives 761; item 50 gives 792 (computed).

The same Mote ids 0xA0 to 0xAD (160 to 173) also index the attack-family effect tables (2.9; SPEC-0036, Attack-family effect tables), which are a separate source of effect ids.

#### 3.16 Effect objects, groups and their life cycle

Confidence: High, except where noted.

- **Groups.** The shared manager (3.7) has four group slots; only slot 3 is loaded and bound. Binding a group writes every node's spawner pool, particle pool and shader-list context and resolves each material index (-1 = none) (3.4, 2.1 step 5). Shared group 3 holds the resource-9 table and is bound to the global pools of 300 spawners and 800 particles. A lookup by id scans the group's definitions in order (first match).
- **Effect object.** An owner holds a 0x90-byte copy of a definition (bind = copy; binding starts nothing). Its five words at +0x58 to +0x68 are its live spawner slots. Calls on it: set position (copied to every live spawner), set frame (copied to every live spawner), set inherited velocity of its spawners, update, stop.
- **Update** (the owner calls it, with an optional listener: the camera or another point). Nothing happens for an object with 0 emitters. Let d^2 be the squared distance from the effect to the listener.
  1. Cull: when a listener is given and the cull distance (+0x84) is above 0 and d^2 > cull; or when a listener is given, the cone flag (+0x88) is set, the dot product of the listener's forward vector with the unit vector from the listener to the effect is below 0.5, and d^2 > 225. A culled object is stopped.
  2. State: 1 (far) when a listener is given and d^2 is above the threshold (+0x50); else 0 (near). Without a listener the state is always 0 and the object is never culled.
  3. If the object has no live spawner: start every emitter whose A +0x84 gate equals the state (3.4; up to five). Else, if the state differs from the running state: stop, then start the emitters of the new state.
  Consequence: an object updated every tick restarts its emitters each time all of them have finished. Spawners of a negative base period are not kept in the slots (3.4), so they do not count as live here; an object whose emitters all have a negative base period therefore fires again on every update (Medium; the known users update such objects once, for example StartEffect).
- **Stop.** Every live spawner is drained (3.2) and the slots are cleared; the spawners finish on their own (particles already in flight live out their lives). A spawner that dies clears its slot through its end hook.
- **Hero footstep dust (High).** On a footstep frame of the grounded hero (table-B bit 0x80 or 0x100; SPEC-0005 and SPEC-0031), the hero starts a dust effect at joint 28 when the frame has bit 0x80, otherwise at joint 31. The surface under the hero (ground material code, SPEC-0031) picks a dust entry: dirt (codes 0 and 18) entry 1, grass (1) entry 2, muck (5) entry 5, water (8 and 11) entry 8, wood (9) entry 9; other codes request no dust. The table at 0x3150A8 gives definitions 163 (dirt), 181 (grass) and 162 (water), and -1 (none) for muck and wood. Id 162 has no definition (2.7): the lookup finds nothing and the footstep sets up no effect object, so water footsteps show no dust (High). The table's other non-empty entries (7, 10 and 11 to 17, ids 184, 171 and 164 to 170) are not used by the footstep; an entry index outside 0 to 27 gives none.
- **Fixed users of group 3.** HUD loss bar: ids 4002 (health), 4003 (pool) and 4001 (a third slot), bound when the bar's owner changes (SPEC-0023). Hero swap: ids 4025, 4026 and 4027 (SPEC-0053). Save-point glow: ids 400 and 404 (section 36). All eight ids are present in resource 9.
- **One-shot effect at a point (High for the steps; Medium for the callers' roles).** A shared helper asks the shared effect manager for group 3 and does nothing if the group is absent. It looks up definition 222 when its flag argument is 0, or definition 220 when the flag is 1, and does nothing if the definition is missing. It then sets up an effect object from the definition, places it at the given position, starts it and detaches it with mode 2 (section 31), so the effect plays once and frees itself. There are 16 callers: 14 pass 0 and 2 pass 1; they are world-object and character routines (Medium). Its users include the Gelert Assassin's flinch, the Ixi Chieftain's incoming hits, Juppie's head slam, the Plague Serpent's bite, the Faerie Sisters' damaging hit and the Spyder Nest valve state 5, which normal play does not reach (High for these users; Medium for the roles of the others). Both definitions are in resource 9 and are short single-burst flashes (High for the values): 220 has three emitters with a life of 0.3 s (an alpha sprite of colour (1, 0.5, 0), child models, and a light emitter of radius 15); 222 has four emitters with lives of 0.2 to 0.4 s (an alpha sprite, child models, 15 additive sparks at 8 units/s, and a light emitter of radius 15).
- **Black Knight hit spark (High).** On each weak-point hit of the Black Knight (SPEC-0012), effect 305 of group 3 starts once at the world position of the hit part's frame. 305 is a one-shot spark burst of four single-burst emitters: 3 pink model particles and 1 sprite (0.7 s), 15 white sparks (1 s) and 15 dark sprites (2 s). Because every emitter fires one burst, each hit restarts it.
- **Placed records** (type 39, section 6) are effect objects in place; their activation is in section 6.

#### 3.17 Hero attack-effect driver: who reads each attack-family table, and when

Confidence: High for which routine reads which table and for the trigger conditions; Medium for the on-screen role names; Medium for the tip and near-hand points (the camera offset in particular).

The driver runs once per main-loop pass for the active hero, only while the game-state code is below 9 (SPEC-0036, Attack family driver). Its inputs: the equipped Mote m from the hero's equipment word 2 (-1 for none, otherwise 160 to 173; the table index is m - 160); the hero index h (0 Tor, 1 Roberta); the attack sub-state s (hero word +0xF90) and its tick counter F (+0xF8C); the full-press threshold Theta (50 for Tor, 80 for Roberta); "charged" means F > Theta; and the hero frame (its translation is the hero position). Two points are derived from the hero: the near-hand point N = position + 0.48 Z0 and the weapon tip T = position + R Z0, where Z0 is the third axis of attach point 0 (section 5.1) and R the weapon reach of SPEC-0021 (1.5 with no weapon); both points are then moved 0.2 units toward the camera's eye (Medium). The effects below that use birth mode 2 are given T as their target point every pass (section 4.1 step 2). The per-Mote effects below also need a gate word of the party inventory object to be nonzero (it is on a normal save; Medium-Low for its meaning).

Which routine reads which table (tables are at the addresses of Where it lives; ids are s32 and -1 means none):

| Table (14 entries, index m - 160) | Hero | When it is read | What it starts |
| --- | --- | --- | --- |
| 0x315280 (ids 1500 to 1506, each twice) | Tor | every pass while m is not -1 | the Mote weapon line: started with its frame origin at N and its spawners' target at T (mode 2 birth), restarted when m changes, stopped when m becomes -1; N and T are rewritten every pass |
| 0x3153D0 (ids 1507 to 1513) | Roberta | every pass while m is not -1 | the Mote aura: started at the hero frame, replaced when m changes, stopped when m becomes -1 |
| 0x315398 (ids 531 to 537) | Roberta | every pass while m is not -1 and F is not above Theta | the Mote charge-up effect at the hero frame; stopped when charged or when m is -1 |
| 0x315328 (Tor) and 0x315360 (Roberta), equal contents (ids 570 to 576) | both | once, on the first pass of sub-state 1 in which F is above Theta | the fully-charged burst at the hero frame, detached so it plays out (m = -1: 243 for Tor, 240 for Roberta) |
| 0x3152B8 (Tor) and 0x3152F0 (Roberta), equal contents (ids 577 to 591, mode 2 birth) | both | started with the burst above when not already live, then followed every pass | the charged-hold loop: frame origin at N, target T, until the sub-state leaves 1 (m = -1: effect 245 for Roberta, none for Tor) |
| 0x315588 (ids 1300 to 1306, light emitters) | Tor | every pass while m is not -1 | a coloured point light on the weapon tip T, replaced when m changes |

Further effects that the same driver starts from immediate ids (no table): Tor's swing glows (below); Roberta's spin glow, effect 242 at T on every pass of sub-states 6 and 7; Roberta's charge-up glow, effect 239 at T: it starts when she has been in sub-state 1 for 0.8 s (49 passes at 60 per second, the single-precision sum of the tick length) and F is not yet above Theta, and it stops when the charge completes or the sub-state ends; and the leap glow 241 (SPEC-0036, Timed glow).

Tor's swing glows. While s is 2, 3 or 4 (the first, second and third swing), a timer adds the tick length each pass; when it reaches 0.25 s (s = 2), 0.2 s (s = 3) or 0.25 s (s = 4) it starts effect 244, 243 or 246 respectively at T, once, and then moves the effect to T every pass while s stays the same. Leaving that sub-state resets the timer and the started flag but does not stop the effect, which plays out. That is about 15, 12 and 15 ticks after the swing starts (the single-precision sum decides between neighbouring passes). These glows, the held-weapon swing ribbon (section 3.8) and the hero's two hit effects (SPEC-0036, incoming hits) are the swing and hit visuals; none of them reads an attack-family table.

Roberta's shot visuals (the remaining tables). Each time Roberta launches a magic shot (SPEC-0036, Roberta shot projectile), one of six shot-visual records is filled with three effect ids, unless the launch is a full release (s = 9) with Mote 161, 162, 164, 165, 166, 169, 171 or 173, whose Mote object supplies the visuals instead. A is the flight effect, B a second flight layer and C the impact effect:
- Not a full release: A = the eight-entry table chosen by the shot kind (0x3154C8 for kind 0, 0x3154E8 for kind 1, 0x315508 for kind 2) at the shot step 0 to 7 (the attack clip's position in the family); any other kind gives 402. B = the Mote aura table 0x3153D0 at m when m is not -1, otherwise none.
- Full release (kind 3): A = the table 0x315440 at m (many entries are -1, which means no visual at all), or the 0x315508 table at the step when m is -1. B = none.
- C = the table 0x315478 at m when m is not -1; otherwise the eight-entry table chosen by the kind (0x315528, 0x315548 or 0x315568 for kinds 0, 1, 2) at the step; any other kind gives 403.
The flight effects follow the shot every pass; when the shot ends they stop and C plays once at the shot's last position. Two table readers have no caller: the one for 0x315408 (so that table is never read) and a second one for 0x315440 (the live read of 0x315440 is the shot-visual builder above).

Test vectors (real table values). Tor, Mote 164 (index 4), F = 51 in sub-state 1: the weapon line 1506 is already running; the first such pass starts the burst 576 and the loop 581, both at the hero's near-hand frame, and 581's spawners get target T; the pass that leaves sub-state 1 stops 581. Tor with Mote -1 in the same pass starts 243 and no loop. Roberta, Mote 163 (index 3), full release: A = 527 (table 0x315440), B none, C = 999 (table 0x315478). Roberta, Mote 169 (index 9): the aura is 1507, the charge-up effect 535, the burst 570 and the loop 587; a full release with 169 spawns the Mote object and no shot visual. Roberta quick shot of kind 1 at step 2 with m = -1: A = 1107 and C = 993.

### 4. Particles

#### 4.1 Birth (init)

Confidence: High for sprite particles (test vectors 70 to 72); Medium for modes 2 and 3 and for the model branch. All "x + spread" pairs mean x + U(-spread, +spread).

1. The particle copies the spawner frame and records its spawner, the node's shader-list context and the material.
2. Position P = spawner position. Ring: if the node's ring radius r (+0x68) is above 0, alpha = U(0, 2 pi), radius = r (or U(0, r) when node +0x66 = 0), and P moves by radius cos(alpha) X + radius sin(alpha) Z, with X and Z the world axes of the basis block. Mode A +0x80 = 2 or 3 (launch modes 2 and 3; High for the arithmetic): P is rebuilt from the spawner position S alone, so the ring offset above is dropped. Let T be the spawner's target point and d = T − S over the three position components, L = |d| and u = d / L. Then P = S + u x t with one draw t = U(0, L), a point uniformly spread along the segment from S to T. When L = 0, u is the zero vector (the division by zero saturates and the product with a zero d is zero) and P = S, and the draw still returns 0. All particles of one burst use the same S and T and each draws its own t. Modes 2 and 3 differ only in what happens to T after a burst (3.2). Where T comes from: it equals S at the spawner's creation; an effect-object position update moves S and leaves T alone; mode 3 sets T to S after each burst; and the hero's attack-effect driver writes T directly, every pass, for the effects it keeps running (the equipped Mote's weapon-line effects and the charged-hold loop effects, section 3.17): T is then the weapon tip and S the near-hand point, so the particles are born along the blade. Without such a write, a mode 2 spawner spreads its particles from the point where it was created to wherever it has been moved to, and a mode 3 spawner spreads them over the path travelled since its previous burst. Test vector: S = (0, 1, 0), T = (3, 1, 4), so d = (3, 0, 4), L = 5, u = (0.6, 0, 0.8); a draw t = 2 gives P = (1.2, 1, 1.6), t = 0 gives (0, 1, 0) and t = 5 gives T. With T = S every particle starts at S.
3. Life = A +0x78 + U(-A +0x7C, +A +0x7C) seconds. Colour channel 1 = B row 0 (B +0x00). Size channel 2 = A +0x00 (a copy is kept for the loop restart). Both colour key indices start at 0. Elapsed = 0.
4. If the node has a C record: acceleration clock T = C +0x2C + U(-C +0x2E, +C +0x2E); kick interval K = C +0x38 + U(-C +0x3A, +C +0x3A) when C +0x38 >= 0, else K = -1 (never).
5. Direction: axis = normalise((1 - w) x row 1 of the spawner frame + w x the spawner direction vector), where w is the spawner's direction weight (w = 0 for effect objects, whose frame row 1 is the local up). theta = U(C +0x26, C +0x24) and phi = U(-pi, pi). Take two vectors e1 and e2 perpendicular to the axis, each with a random sign. d = normalise(cos theta x axis + sin theta x (sin phi x e1 + |cos phi| x e2)). The sines and cosines use the folded series of section 12 (copy it). Perpendiculars (High for the construction order, Medium for the choice of helper): e1 is the cross product of the axis with a coordinate unit vector chosen as the one least aligned with the axis, normalised; e2 is the cross product of the axis with e1, normalised. Each of e1 and e2 is then negated with probability one half, by one draw of a uniform integer 0 or 1 (a 0 negates it). Because phi is uniform and the second perpendicular's sign is random, the direction is uniform on the circle of perpendiculars whichever helper is used. Angles (High): cos theta is the folded series at theta itself; sin phi is the same series at phi minus pi/2 (the cosine is not a series at phi plus pi/2); |cos phi| is the square root of one minus sin squared phi; and sin theta is the square root of one minus cos squared theta, which is non-negative because theta lies in 0 to pi. Velocity v = (C +0x28 + U(-C +0x2A, +C +0x2A)) x d, plus the spawner velocity when A +0x85 = 0.
6. Spin rate = C +0x30 + U(-C +0x32, +C +0x32), spin angle 0. Drag per tick k = C +0x22 x dt. Acceleration per tick a = (C +0x20 x T x up + C +0x10) / T x dt, with up = (0, 1, 0). This equals C +0x20 x dt along world up when T > 0 (C +0x10 is zero on disc), and zero when T = 0.
7. Sprite (material present): colour bytes = trunc(channel 1 x 127) per component (R, G, B, A); sprite half-extent lanes (x, y, z) = (factor at +0x6C, factor at +0x6E, factor at +0x6A, each as s8.8) x (size x, y, z) x S (the lane order is not the order of the fields; High). In the sprite quad, lane z is the half-width, lane x the half-height above the centre and lane y the half-height below it (16.4). Example from disc: a node with factors 144, 289 and 0 at +0x6A, +0x6C and +0x6E and size (0.5, 0.5, 0.5) gives lanes (0.5645, 0, 0.28125) x S at birth, so it is a sprite of width 0.5625 x S that grows upward only. Model (no material): the spin axis comes from C +0x3F (2.6) and the frame rows are scaled by size x S (Medium).

#### 4.2 Motion (per tick)

Skipped when the node has no C record.

1. Velocity: v = v + a; then v = v - v x k (drag). So v becomes (v + a) x (1 - C +0x22 x dt).
2. Kick: when K = 0, or when K > 0 and trunc((e - dt) / K) < trunc(e / K), where e is the elapsed time before this tick's age step: K is drawn again (as in 4.1 step 4); if C +0x3D is set, v = 0; R = C +0x34 + U(-C +0x36, +C +0x36); the kick (U(-R, R), R, U(-R, R)) is added to v. The vertical component of a kick is always +R (quirk: the range passed for it is (R, R)). A kick takes two draws.
3. Sprite: when the spin rate is not 0, the spin angle grows by spin rate x dt. P moves by v x dt x S. If A +0x85 = 1, P also moves by the spawner's delta (3.2 step 4). Model particles rotate about their spin axis instead (not specified further; Open question 5).

#### 4.3 Motion summary and checks

- Speed and direction are fixed at birth. Velocity changes only through the acceleration, the drag and the kicks.
- Drag: with C +0x22 positive, velocity decays; with C +0x22 negative (present on disc), velocity grows. Keep the sign as stored.
- Vertical acceleration: C +0x20 is in units/s^2 along world up; the per-tick gain a already contains dt and is added once per tick. A negative value makes particles fall.
- Kick interval 0 gives a kick on every tick (278 records; only 2 have a non-zero kick strength, which gives a constant +R upward push each tick).

#### 4.4 Curves over life

Two channels, both run over the life fraction f = elapsed / life (life from 4.1 step 3):

- Colour channel 1 (r, g, b, a): start value B row 0; rows k = 0 to 5 at B +0x10 k act as rates; an RGB key index (key times at B +0x60) and an alpha key index (key times at B +0x78), each 0 to 5. Each component is clamped to [0, 2] after every update.
- Size channel 2 (x, y, z): start value A +0x00; three rate rows at A +0x10, +0x20 and +0x30, picked per axis by the thresholds at A +0x40 and A +0x50. Channel 2 is not clamped; only the sprite half-extents derived from it are kept at 0 or more.

The per-tick rules are steps 5 and 6 of 4.6.

#### 4.5 Light emitters (A +0x87)

Runs for the particles of an emitter whose A +0x87 flag is set (37 records on disc). Such particles are never drawn as sprites. Each tick, while their spawner has a material (the spawner's material word is non-zero, and the word at +0x54 of that material names a definition whose light flag is set). The spawner's material word is set when its world binds the world's level-script module (SPEC-0030), which is the only record of its kind in a pack. The light-emitter definitions carry no such record: a definition is a fixed table entry with no chunk list. On disc the word at +0x54 of every level-script module is zero (173 modules, including the 12 packs that hold the placed spawns with effects 1300, 2202 to 2207, 4012, 4023, 4030 and 4031), so the test never passes and no light is listed in normal play (High for the disc values; Medium for the test, since a run-time write of that word is not ruled out), each particle writes into its spawner's light record: the particle position; colour = channel 1 x 255 (r, g, b; alpha lane 0); intensity = channel 1 alpha; light type 3 (point); radius = size (channel 2 x); and the square of the radius. The record is appended to the frame's light list, which holds 200 entries; more are dropped. The light consumer belongs to SPEC-0046.

#### 4.6 Per-tick order, curves and expiry

Order within a tick: all spawners step first (3.2), then each live particle runs its motion (steps 1 to 3 = 4.2) and then its curves (steps 4 to 6):

4. Age and end: elapsed grows by dt. If elapsed > life: when A +0x86 = 1 and the spawner is not draining, the particle loop-restarts (P = spawner position, elapsed = 0, channels 1 and 2 = their start values, key indices 0) and continues this tick with the fraction computed before the restart (quirk). Otherwise the particle ends (the spawner's live count goes down by 1). Before it ends, the emitter node's link word is checked; a non-none link would start a child spawner at the particle's position. The loader makes every link on disc none (2.1 step 3), so no child is started from disc data, and no run-time writer of a non-zero link is known (Open question 23).
5. Colour: f = elapsed / life (the pre-restart value on a restart tick). The RGB key index advances by one when f is above the RGB key time at the current index and the index is below 5; the alpha key index does the same with its own key times. Rate = (rows[RGB index].rgb, rows[alpha index].a). Channel 1 grows by rate x dt / life, and each component is then clamped to [0, 2]. Colour bytes = trunc(channel 1 x 127).
6. Size: per axis, rate = A +0x10 if f <= A +0x40, A +0x20 if A +0x40 < f <= A +0x50, else A +0x30. If the rate vector is not zero: channel 2 grows by rate x dt / life, and the half-extent lanes = max(0, node factor x channel 2 x S) per lane, with the factor of each lane taken as in 4.1 step 7.

A draining spawner (state 5) launches no bursts (3.2), but its particles keep running until their life ends; looping particles of a draining spawner end at their next life end.

Place in the pass and freezing (High). The shared effect manager is advanced inside the effects render pass of the main loop's draw phase, and only when the game-state code is 8 or less (SPEC-0026, main loop step 11). In code 9 (pause and the other freezing screens of SPEC-0026) the effect manager is not advanced: spawners do not step and particles neither move nor age, but the effects are still drawn, frozen in place. The placed-effect lists and the animation-effect pool are stepped in the world branch, so they freeze with it. The screen fade (the fade object) is not part of this and steps on every pass. The swing-trail pass in the same render pass draws on every pass and ages its samples only while the code is 8 or less (3.8). The hero attack-effect group (Behavior 13) is driven only while the code is below 9, so it freezes too. The world effect group at world +0x2CC8 (50 particle records of 0x170 bytes, created at world setup) has its own particle step, a four-state oscillator. Its particles step only while the game-state code is below 9; under code 9 they are drawn frozen in place (Medium-High).

#### 4.7 Drawing

Confidence: Medium-High for the batches; Medium for the packet details.

- Each frame the live particles (state 2) of a pool are drawn, except those of light emitters (4.5).
- Material present: the particle's 0x40-byte sprite record (the RGBA bytes, position, half-extents, shader context and material, spin angle and a link) is added to one of 20 batches keyed by (shader context, material, depth group). At most 2000 sprites are drawn per frame. A 21st key is dropped, with a log.
- The depth-group component is the current renderer context selector, an index of 0 or 1 in a two-entry context table. It is copied into the upper 24 bits of the sprite record's 64-bit word at +0x08 before batch lookup. Two sprites with the same shader context and material share a batch only when this selector also matches. No camera-depth calculation for this selector is present in the recovered batch path. Synthetic check: equal shader context, material and selector 0 share a batch; changing only the selector to 1 creates a different key.
- Material absent: the node's child model is drawn with the particle frame, and channel 1 as its colour (Medium).
- The batch packet builder is section 16. Its per-sample angle is the sprite spin angle (4.2 step 3), so Q1 and Q2 there are the sine and cosine of the sprite rotation.
- The GS colour convention applies: a colour byte of 128 is 1.0, so a channel value of 1.0 gives 127, just under full.

### 5. Attachment, owner transform, culling and positional effects

#### 5.1 Attachment and owner transform

The instance's origin comes from up to two attach slots (entry +0x20 and +0x24 of the list entry, or the placed record's position). Slot k is one of the eight matrix slots of the owner (4x4 matrices in the owner's model state), stored at a 0x40 stride. Each slot is four 16-byte vectors X, Y, Z and T in memory order: X, Y and Z are the axes and T is the translation, and a point v maps to X v.x + Y v.y + Z v.z + T v.w (High). For a list entry, point 1 is slot A's matrix applied to the entry's vector at +0x00 with w = 1 (so the slot's translation is added), and point 2 is slot B's matrix applied to the entry's vector at +0x10 with w = 1. Point 1, after applying the ground-height rule selected by +0x3C and +0x3D (1.1), is the effect object's position; when +0x49 is non-zero the effect object instead gets slot A's whole matrix with point 1 as its translation. Points 1 and 2 are the two trail edges of 3.8 (High).

An instance with an owner takes its attach point from the attach slot, or from the owner origin when no slot is set. The list path keeps its started entries in the animation-effect pool: 10 elements of 0xB0 bytes (+0x08 the list entry, +0x0C the character, +0xA0 and +0xA4 the trail words of 3.8), updated once per pass and released at the entry's stop window (Medium-High).

The owner transform (position and frame) of an instance is set from the owner when the instance starts, when it is refitted (3.7), and when the owner moves the attach point. The effect object's set-position and set-frame calls copy the new position or frame to every live spawner (3.16); particles already born keep their own position unless A +0x85 makes them follow the spawner (4.2 step 3) (Medium-High).

#### 5.2 Distance and cone (per instance, per pass)

Let d² be the squared distance from the instance's position to the listener (the camera or the player; the caller decides).

1. Cull: if a listener is given, the cull distance is greater than 0 and d² > cull, stop the instance.
2. Cone: if the cone flag is set and the listener is given, compute the unit vector from the instance to the listener, take its dot product with the instance's direction. If the dot product is below 0.5 and d² > 225 (the cone limit at 0x2ECCC0), stop the instance.
3. State: near (0) when there is no listener or d² ≤ threshold (definition +0x50), far (1) otherwise. A change of state stops (drains) the running spawners and starts the emitters with A +0x84 equal to the new state. When no spawner is live, the emitters of the current state are started again (3.16).

Units: world units (the same units as positions). Medium.

Placed-effect records apply the same rules; their cull and cone values come from the record (see 6).

#### 5.3 Positional attenuation (list entries with a positional mode)

For a list entry with a positional mode (not 255), each pass:

- Let d² be the squared distance from the object to the listener. Let r_in = entry +0x40 and r_out = entry +0x44.
- If d² ≤ r_in², intensity = entry +0x38 (full).
- If d² > r_out², intensity = 0.
- Otherwise intensity = entry +0x38 × (1 − (d² − r_in²) / (r_in² + r_out²)).
- If intensity > 0, the entry's sound id at +0x30 is resolved through the loaded sound list; when present, its sound instance is configured at the computed effect point and started. The sound setup does not receive the entry's +0x34 word on this path. For mode 0, d² is the squared distance from the character to the camera. The mode also starts its camera-shake request and requests rumble preset 0 with duration +0x34 seconds and magnitude equal to intensity. On disc the 164 mode-0 entries use durations from 0.3 to 2.0 seconds (0.4 in 81 entries, 0.5 in 61, 0.8 in 17, 0.3 in 4 and 2.0 in 1); intensities are 0.2 to 15.0, and 130 entries use radii 10 and 50. Mode 4 passes 0.8 and mode 5 passes π/2 as fixed values (Medium; not confirmed).

Edge case: the zero test is strict: d² > r_out² sets intensity to zero, while d² = r_out² still uses the interpolation formula. For r_out > r_in > 0, the exact boundary weight is 2r_in² / (r_in² + r_out²), so the positive value proceeds to the request path when the base intensity is positive; just beyond it the result drops to zero. With the common radii 10 and 50, the boundary is 1/13 of base intensity. This endpoint step is part of the original rule (Open question 9 resolved). Only modes 0 and 255 occur in list entries on disc (see Edge cases). The meaning of the modes is not settled; the spec does not define them beyond the constants above.

#### 5.4 Status tint routines (once called owner frame presets)

Confidence: High. These six routines do not set a frame. They are the per-tick steps of the character status display (SPEC-0036, Status display and cues), one per shown status. Each tick the display runs the step of the shown status code:

| Shown code | Status | Emitter placement | Stored colour (R, G, B) |
| --- | --- | --- | --- |
| 0 | stun | main attach point | none in this step (the stun start sets its own tint) |
| 1 | curse | main attach point plus (0, 1.5, 0) | Tor (0.4, 0, 0.5); Roberta (0.5, 0, 0.4); any other hero slot uses Tor's |
| 2 | poison | main attach point | (0.2, 1.5, 0.6) |
| 3 | slow | main attach point | (0, 0.5, 1.5) |
| 4 | strength | main attach point | (1.8, 0.3, 0.2) |
| 5 | speed | main attach point | (2.0, 2.0, 0.3) |
| 6 | confusion | main attach point | none |
| 7 | fire | all three emitters: the main point and two more attach points | (1.5, 1.5, 1.0) |
| 8 to 10 | (one-shot effects only) | nothing | none |
| none (-1) | | the three emitters stop | the tint is cleared |

Attach points (High): the main attach point is attach point 7, the translation of the owner's attach-point bank entry 7 (the bank of section 5.1); the status starts and the per-tick follow read that translation. Fire's two more points are attach points 0 and 1 (their translations). The stun start sets slot 0 of the colour override with mode 0 (SPEC-0046, Colour-override slots), so its flash colour holds at full weight, steady, until the display changes to another status or clears the tint (Medium for the end of the flash, which follows the display's clear rule).

The colours are run-time words at 0x374C50 to 0x374CEF, written once by a start-up initialiser from constants in its code (the curse offset (0, 1.5, 0, 0) is at 0x374C50 and the stun colour (0, 0.5, 1.7) at 0x374C60).

Tint step, for codes 1 to 5 and 7:
1. Three channel records are built, one per colour channel c (R, G, B): low = min(c, 1.0), high = max(c, 1.0), and a rate word (1.0, or -1.0 for the green channel of poison). The records are stored on the character but nothing reads them except this step.
2. The tint target is (low_R, low_G, low_B, 1.0). When all three lows are 1.0 the colour-override slot 0 of the character's render object is cleared; otherwise slot 0 is set to the target in mode 1 with both lane masks on (SPEC-0046, Colour-override slots; section 21 of this spec). Because the slot is already on after the first tick, the 60-tick counter is not reset by later ticks.

So every colour channel above 1.0 is cut to 1.0 before it is used. The targets on screen are: curse (0.4, 0, 0.5) or (0.5, 0, 0.4), poison (0.2, 1, 0.6), slow (0, 0.5, 1), strength (1, 0.3, 0.2), speed (1, 1, 0.3). Fire gives (1, 1, 1), so its step clears slot 0 every tick and a burning character shows the fire effect with no tint (reproduce). The stun start is a different routine: it sets slot 0 once in mode 0 with (0, 0.5, 1.7, 1.0), not capped. The rate words do not change the picture.

These routines and their colour words are separate from the 27 presets of section 3.9 (different code, different tables).

### 6. Placed effects (world type 39)

- Payload: u32 count at +0x00; records at +0x10, stride 0x90. 158 chunks, 5,139 records. Records per chunk: 1 to 274 (sum 5,139).
- Record fields: direction vec4 at +0x20 (xyz, with w = 0 at +0x2C on all records); the direction is unit length in all 5,139 records within a tolerance of 1e-3, so treat the stored value as already normalised (Open question 13). World position vec4 at +0x30 (w = 1 at +0x3C on all records). Effect id i32 at +0x40 (116 distinct, 101 to 4042). u32 at +0x44: the placed effect's zone number, that is, the column of the type-35 zone query (SPEC-0006, Binary grids); the per-pass check uses its low byte (below; High). Values 0 to 59 (0 on 1,926 records, 1 on 544, 3 on 481, 2 on 405, 4 to 59 on the rest); in all 4,559 records of packs with a type-35 grid the value is below block 0's width. +0x50 threshold = 10000 in all. +0x54 child count 0 in all (filled at runtime). +0x84 cull, squared: -2 in 4,869 records, -1 in 3, and the rest 225 or more (see Edge cases). +0x88 cone flag = 1 in all.
- At world load, each record's effect id is resolved in the definition table (first match) and the record takes the definition's values, then restores its own placement fields (the transform block, the word at +0x44 and the word at +0x80). Its own cull value is kept only when it is positive or -1 (Edge cases; High). Every one of the 116 ids on disc resolves to a definition (High); a record whose id did not resolve would stay unbound. The bind is a copy and starts nothing, so placed effects do not start at load.
- Activation (High). A placed record is an effect object in place (3.16). The active flag at +0x48 is zero in all 5,139 records on disc.
  - StartEffect(slot, flag) runs one update of the record (3.16). With the flag set the camera is the listener, so cull, cone and near/far apply; with it clear there is no listener (state 0, never culled). It does not set +0x48.
  - StopEffect(slot) sets +0x48 to 0 and stops the record (drain).
  - MoveEffect sets the record's position, or its frame from a direction (Medium).
  - Switch on (a type-40 entry; only its first listed placed index is used; SPEC-0028) sets +0x48 to 1 and starts nothing. Switch off sets +0x48 to 0 and stops the record.
  - Zone gate (High). Each pass, for the current world and for the second world, every active record (id other than -1 and +0x48 non-zero) is zone-queried with column = the low byte of +0x44 and key = that world's id (SPEC-0006, Binary grids, Query). When the query fails (the zone is hidden from the camera's zone), the record is stopped (drained). When it passes, the record is updated with the camera as the listener, so cull, cone and near/far apply (3.16). Records with +0x48 = 0 are not touched by the pass: emitters started by StartEffect alone keep running until stopped (looping ones) or until they finish (one-shot ones). On world-buffer release, world reset visits every placed record, drains and releases each non-null emitter instance, and clears its slot. Leaving the effect's zone alone does not stop an unflagged looping effect; a retained second world keeps it allocated until that buffer is released.

- **Script calls (High).** StartEffect and StopEffect treat their first integer argument as a zero-based index into the selected world's placed-effect table. Both reject a negative index and an index at or above the table count; a valid index selects record n at the table base + n × 0x90. The selected record carries its definition id at +0x40, which the world-load binder resolves separately. Therefore values 159, 162, 790, 938 and 4007 in these script calls are slot arguments even when the same number is an effect-definition id; 162 has no definition in resource 9, but remains a slot value at this API. A separate program path can look up definition 938 directly and start a detached effect object; it does not pass 938 to this script slot entry point (Open question 125 resolved).

### 7. Pools

- Effect instances: 10 slots per pool (each slot holds the instance and a handle to an audio or emitter helper). Full pool = dropped start.
- Emitter spawners and particles: global pools of 300 spawners and 800 particles for shared group 3; pools of 30 and 100 per character, and 40 and 200 per world (High). A particle that cannot be allocated is not spawned, and the emitter's cycle continues. A spawner that cannot be allocated is skipped (3.4).
- Pool slots are reset on release (zero for handles, -1 for keys).
- Rumble segments (High): the shared rumble manager owns six fixed slots of 0x18 bytes, indexed 0 to 5; these are not sub-slots of an effect record. A five-entry preset request API constructs up to three linked segments per request. The allocator returns the first free slot and marks it in use; if all six are occupied, no segment is allocated. Each update scans the slots, advances enabled segments and sends the greatest current magnitude and its remaining duration to the vibration state (3.12).
- Buffer for N records (High for the formula): the requested size is N x 0x60 + floor(N / 2) x 0x10 + 0x1000 bytes, rounded up to a multiple of 256. The renderer owns two shared packet-batch objects, with capacities 700 and 1,000; each stores the aligned size, buffer base and second-half pointer. The buffer stages renderer-facing DMA/GIF packets from linked draw records: the builder writes a 0x10-byte packet header and 0x30-byte samples. This establishes the packet stream's role and write stride, not the full meaning or layout of all packet/header fields. The arena allocator aligns the returned base to 128 bytes; 256-byte alignment applies to the size, not the base. Setup fills exactly the rounded size with byte 0xDE. Drawing passes a selected base plus the current write offset and a remaining-capacity value to the packet builder; the selector's live domain and its capacity relationship to the second-half pointer remain open (Open question 79). A second setup on an object that already has a buffer logs and returns failure and keeps the old buffer.
- DLL class-instance registry (High; not an effect-definition id table): the candidate is a runtime DLL class-instance address and must be a multiple of 128. A misaligned address returns -1 with no log. An occupied per-instance registration record returns -1 and logs. Otherwise the DLL manager validates and links/registers the image; on failure the call logs and returns 0 without storing, and on success the caller's record stores the address and the call returns 0 (Open question 80).
- Owner effect state packing for the remote path: up to four owners' effect state blocks, each 0x70 (112) bytes, are copied into one local buffer and passed to the send routine of the remote effect path. The count (four) and the block size are established. The whole path is debug-only and never runs: the packing routine is called only by a statistics-print routine that nothing calls, and the send goes to a host link (Open question 42).

### 8. Reed-cut coin roll (not an effect)

Confidence: High for the ranges and the spawn route; Medium-Low for what the two points are. This routine was once read as an effect burst. It spawns world items, not effects; the ids are item ids (SPEC-0021 coins), not definition ids.

When the hero's weapon cuts a reed (SPEC-0060, reeds), the cut routine runs once:

1. The launch direction is taken from two points held by the hero's render object, each moved 2.5 times along its own direction vector; the direction is the first point minus the second. Its y component is set to 0 and the rest is normalised, so the launch is horizontal.
2. Draw r, a whole number uniform from 0 to 100 (101 values, both ends inclusive).
3. Pick an item by r:
   - r = 0: item 138 (1 of 101 draws).
   - r = 1 to 5: item 136 (5 of 101 draws).
   - r = 6 to 19: item 135 (14 of 101 draws).
   - r = 20 to 100: nothing (81 of 101 draws).
4. A picked item is spawned at the reed through the item drop spawner (SPEC-0021, Spawn routes).

No effect definition is started by this routine. Definitions 135, 136 and 138 do exist in the definition table, but nothing here plays them.

### 9. Map icon pulse (not an effect)

Confidence: Medium-High for the role; High for the constants (Where it lives). These two oscillators belong to the View Map page of the front end (SPEC-0035), not to an effect. The routine draws the map's icon list for the current world slot and runs once per drawn frame of the page.

- Amplitude: starts at 1.0. Each update adds the amplitude step (0.03). When the amplitude is above 1.5 it is set to 1.5, and when it is below 0.8 it is set to 0.8; the step's sign flips at each of these bounds. The hero's own marker (an arrow turned to the hero's heading) takes its colour bytes from a base colour times the amplitude, each rounded half away from zero, so the marker brightens and dims.
- Phase: a 16-bit value that starts at 4096 and is updated in place. The phase step starts at 50, set once on the first update. Each update adds the step. When the phase is above 4300 it is set to 4300 and the step becomes negative; when it is below 3900 it is set to 3900 and the step becomes positive. A value exactly at a bound does not turn.
- Selected icon. The page keeps a selected index among its icons. One pair of direction buttons moves it to the next icon and the other pair to the previous one, wrapping at both ends. The selected icon's two 16-bit scale fields take the phase, and every other icon takes 4096. The scale fields are 4.12 fixed point (4096 = 1.0), so the selected icon pulses between 0.952 and 1.050 of its size. The selected icon also gets its label drawn 14 screen units above it.

The constants keep their boot values in Where it lives; only the role changes.

### 10. Random frame tilt (the shot bounce)

Confidence: High. The tilt acts on a shot frame of three rows: side axis S (row 0), up axis U (row 1) and direction D (row 2), as in the hero shot slots of SPEC-0036. Steps:
1. S and D are negated (the shot turns back).
2. Draw t1 uniformly from 0 to 0.5, then a fair coin makes it negative. D1 = t1 S + (1 - |t1|) D. D1 is stored as is, not normalised.
3. S becomes unit(D1 x U).
4. The slot's arming counter (slot word +0x7C) is set to 15.
5. Draw t2 the same way. D2 = t2 U + (1 - |t2|) D1, stored as is, not normalised. D2 is the new direction.
6. U becomes unit(D1 x D2). Since D1 x D2 = t2 (D1 x U), the new up axis equals the new side axis times the sign of t2 (quirk: the frame is left degenerate; with t2 exactly 0 the cross product has zero length).

The direction keeps its shortened length, so a shot that moves by its direction times its speed is slowed to between about 61 and 100 percent (SPEC-0036). Users (High): the hero's shot hit tests when the target's guard method is true (the bounce of SPEC-0036, Roberta shot projectile), and the Juppie boss's hit helper, which bounces a shot always in its first mode and, in its second mode, only when the shot's alignment value is at most 0.9 (otherwise the boss takes the hit). No effect kind uses the tilt.

### 11. Debug vibration tester (not an effect)

Confidence: High for the settings, the falloff rule and the request; Medium for the timer gate. This routine belongs to a debug page that tests the controller rumble. It draws nothing and is not reachable in retail play. It is listed here only because its settings sit among the effect data words. The rumble itself (presets, entries, the motor level and the Vibration option gate) is SPEC-0033 Rumble.

- Settings (data words, on disc): trigger at 0x2D7568 (0); vibration type, the rumble preset, at 0x2D756C (0); duration in seconds at 0x2D7570 (1.0); magnitude at 0x2D7574 (0.5); inner falloff radius r at 0x2D7578 (1.0); outer falloff radius R at 0x2D757C (30.0).
- Trigger: when the trigger word is nonzero it is cleared, and one rumble test runs for that tick.
- Distance: s is the squared distance from the camera position to the hero position.
- Falloff weight: if R ≤ r, no rumble is requested and an error is reported. Otherwise:
  - if s ≤ r², the weight w = 1;
  - if s > R², the weight w = 0;
  - otherwise w = 1 − (s − r²) / (r² + R²).
  The strength is w × magnitude. If the strength is above zero, a rumble request is made with the vibration type as the preset, the duration setting as the duration and the strength as the magnitude (SPEC-0033 Rumble).
- Edge drop (original behaviour): the middle formula does not reach zero at the outer radius. At s = R², w = 2r² / (r² + R²) = 2/901 (about 0.0022), so the strength is about 0.0011; just beyond R the weight is 0. Reproduce this as written.
- Timer: each tick the page's base update runs first, then its frame counter goes up by 1. The period is clamped to 0 to 60 ticks; 0 disables this action. For a nonzero period, the update clears the shared input-enable word (the selector/input object's +0x04 field; SPEC-0033, Behavior 1A) unless either the previous raw-held word has L3 set, or the incremented counter is divisible by the period and the current raw-held word has L3 set. Thus a previously held L3 suppresses every clear; a current L3 press with no previous hold suppresses only on a period boundary. Otherwise the input-enable word is set to zero. This page is debug-only and is not reached in retail (Open question 40).

### 12. Pulse effect (effect kind 3)

Confidence: phase steps, constants and the wave arithmetic High (from the disc and by computation); the update rate Medium (60 updates per second assumed, Open question 44); the output byte as the alpha of a packed colour record Medium.

The pulse is effect kind 3 in the shared effect list. Kinds 1 and 2 are the other two entries. The kind's update runs once per update of the owning object. Each update:

12.1 Phase A. A = A + 0.06, then reduced to its fraction in 0 to 1 (fmod by 1). A starts at 0. A therefore wraps once every 17 updates: after 16 updates A is 0.96, and after 17 updates A is 0.02.

12.2 Phase B. B = B + 0.2. B is not reduced. B starts at 0.

12.3 Wave and intensity. x = fold(B). Intensity I = 64 + 186 × (1 + P(x)) / 2, rounded half away from zero to a whole number, and stored as one byte at byte 12 of the chosen preset record (flag zero selects the first preset, flag not zero the second). The intensity is therefore always between 64 and 250 (the boot words 64.0 and 250.0 are the low and high ends; 250 - 64 = 186).

12.4 Items. For each item i of the owning object: write the float (base_i + A) at the item's phase-offset float, where base_i is the first float of that item's base record; and copy the 16-byte chosen preset into the item's colour record.

Output meaning (Medium). The items' phase-offset floats and colour records lie in the owner's per-buffer draw data, so the pulse edits a draw that is already built rather than a game value. The preset and each colour record are packed colour records of four 32-bit words (red, green, blue, alpha, in that order, as the graphics chip takes a colour in packed form). Byte 12 is therefore the low byte of the alpha word: the intensity is the alpha of the drawn items, from 64 to 250 on the scale where 128 is 1.0 (the colour scale of SPEC-0046). The first 12 bytes (red, green and blue) come from the drawn object's preset colour and are not changed by the pulse. The phase-offset float (base_i + A) is a texture coordinate offset, so the item's texture scrolls by 0.06 of its size per update.

Definitions.
- fold(B): let m = |B - pi/2| / (2 pi). Away from fold boundaries, let k be the nearest nonnegative integer to m; x = k + 0.25 - m. The interval changes at m = k + 0.5, equivalently |B - pi/2| = (2k + 1) pi. On opposite sides of each boundary x approaches -0.25 and 0.75; both represent the same ideal sine phase, but the truncated polynomial can produce different pulse values. The exact 32-bit result at a boundary depends on the console floating-point reduction and is not established here. The formula follows from the reducer and fold expression in the pulse update; it does not replace the stored-coefficient polynomial with an ideal sine.
- P(x) = c1 x - c3 x^3 + c5 x^5 - c7 x^7 + c9 x^9, a truncated odd power series in x with the constants c1 = 6.283185, c3 = 41.34167, c5 = 81.60223, c7 = 76.57496 and c9 = 39.71066. These are the stored constants of the original. They are fixed program constants of the boot program (the same five values, with the same signs, also appear as constants of a vector-unit program in the boot image); no game data file holds them, so this spec is their source (High). They are not the exact Taylor coefficients of sine in 2 pi x (which would be about 41.34170, 81.60525, 76.70586 and 42.05869). The clean side must use the stored constants: with the exact coefficients the vector at phase 4.8 gives 107, not 94.
- Accuracy. P matches sine closely for x from -0.25 to 0.25 (the largest difference is about 5e-8). For x from 0.25 to 0.75 it does not: the largest difference is about 0.396 at x = 0.75, where P is -0.604 and sine is -1.0. So the intensity is the ideal 157 + 93 sin(B) only where x is between -0.25 and 0.25, and it is higher than the ideal near x = 0.75 (the phase near 3 pi / 2), by up to about 37 byte units. This is a reproducible quirk of the original: the clean side must copy the series, not the ideal sine.

12.5 Start-up value. A start-up initialiser (one of the start-up table entries of SPEC-0026, called once at boot with the arguments (1, 0xFFFF)) sets the run-time intensity floor word at 0x374D00 to 64.0, a copy of the boot word 0x2DCAA4. Nothing sets it again. The pulse does not read that word: it takes its low end from the boot word 0x2DCAA4 directly. No routine in the boot program reads the floor word at 0x374D00, so the clean side may omit it (Medium-High).

Timing. The pulse advances once per update of its owner. The update is assumed to run once per game tick, 60 times per second (SPEC-0007). At that rate phase B grows by 12 units per second, and phase A wraps about every 17 ticks, roughly every 0.28 s (Open question 44).

### 13. Hero Mote power and charm effect objects (the group of about 50 objects) and its driver

Confidence: High for the set, the owner, the gate and the counts; Medium for the per-object roles, which SPEC-0036 owns (Charm attack hits; Attack family driver). The objects take fixed arguments, not definitions picked from list entries; they are separate from the 10-slot instance pool of 3.1.

The group is the hero's attack-effect object set: the Mote power-attack objects and the weapon or charm effect objects of SPEC-0036. The spawn dispatch by Mote id (160 to 173) starts objects of this set.

1. **Init.** A start-up initialiser (a start-up table entry of SPEC-0026, called once at boot with the arguments (1, 0xFFFF)) builds 22 top-level objects, which lie between 0x375F40 and 0x378220, and writes three anchor offsets: (0, 2.0, 0, 0) at 0x375F30 and at 0x376230, and (0, 1.5, 0, 0) at 0x376F90. The set is built at boot, not on first use.
2. **Teardown.** The same routine with first argument 0 releases 30 effect objects with mode 2 (section 31). It is not reached in play.
3. **Driver.** Once per main-loop pass, only while the game-state code is below 9 (so it is frozen in pause), for the active hero only, and only when that hero has a model, a skeleton and a non-zero model-state word at +0xFA0. Order:
   1. the helper set picked by the hero's flag at +0x14C: 0 = Tor (stop the bundle, reset the follow slot, then Tor's attack step); non-zero = Roberta (stop the helper, Roberta's attack step, then the follow step);
   2. the per-owner slot manager (3.7) and the six-slot update;
   3. the group update.
4. **Group update.** All 22 objects are updated each pass. The anchor is the hero's position. Three objects use the anchor + their offset from step 1. The second input to the objects is the hero's per-tick displacement x 60 (the ticks per second word 0x368FA0), a velocity in units per second; it is not a scaled offset. Five objects (0x3774E0 to 0x377720) and three others are updated only while they are active; one starts a positional cue (mode 5) when it has a live spawner.

Test vector (computed): hero displacement (0.05, 0, -0.1) units in one tick gives the second input (3.0, 0, -6.0) units per second; with the hero at (10, 0, 5), the anchors are (10, 2, 5) for the two 2.0 offsets and (10, 1.5, 5) for the 1.5 offset.

### 14. Flat-facing frame object with a lifetime

Confidence: Medium-High for the frame; High for the lifetime constant (read from the boot data); Medium for the owner. Two routines handle one object: a creator and a per-tick refresher. They build a frame (a transform), not four corner points. The object is one of the hero's Mote power objects of section 13 (the one at 0x378220); the group driver's request code 0xA2 creates it (Medium; SPEC-0036 owns the request codes). Both routines take an anchor point and a direction vector.

14.1 Frame. Let D be the direction normalised (a zero direction stays zero), then with its y component set to 0 and its w to 0. D is not normalised again, so its length is the horizontal part of the unit direction. Let up be world up (0, 1, 0) (Where it lives). X = normalise(up x D). U = normalise(D x X). The rows of the frame are row 0 = U, row 1 = D, row 2 = X and row 3 = the anchor with w = 1. For any direction that is not vertical, U is world up, so the object stands upright and faces along the horizontal part of the direction.

14.2 Creation. If the object is already live, the creator does nothing. Otherwise it resets the object, writes the frame as the object's transform, clears its other placement values, sets the lifetime counter (14.3), submits the object to the draw list in draw mode 5 and stores an owner id.

14.3 Lifetime. The creator sets the counter to the whole part of 5.0 x ticks per second, which is 300 ticks at 60 ticks per second (5.0 seconds). Each refresh first lowers the counter by 1. If the counter is then below 1, the object is released and nothing else is written that tick. Otherwise the refresher rebuilds the frame from the current anchor and direction by the same rule, writes it and submits the object again. The object is therefore released on the 300th refresh after creation (Medium; the constant is High).

### 15. UV scroll by time

Confidence: High for the arithmetic, the wrap and the scale word K being the main-loop pass counter (a run-time word, not on disc); Medium-High that it never runs. This routine is kind 2 of the model vertex-animator registry (SPEC-0003). No material on disc selects it, so it does not run in play; the UV scrolling that players see (sky domes, water) is the draw's own material scroll of SPEC-0003.

A routine rewrites texture coordinates so that a texture appears to scroll. Inputs: a count N; a source table of N base coordinate pairs (u, v) with a stride of 16 bytes; a list of N destination byte offsets into the vertex data; and two time values, one for the u axis (t_u) and one for the v axis (t_v).

Offsets. The u offset is wrap(K x t_u / 4000) and the v offset is wrap(K x t_v / 4000). The divisor 4000 is 100 divided by 0.025, where 0.025 is the boot word at 0x2DCB24. K is the main-loop pass counter at 0x368F98 (an integer, 0 at start, plus 1 at the end of every pass, converted to a float here), which is outside the file image. wrap(x) is the floating-point remainder of x by 1.0, which keeps the sign of x: a positive x gives an offset in [0, 1), and a negative x gives an offset in (-1, 0]. t_u and t_v are the two per-axis speed values that the animator receives, in texture widths per 4000 passes.

Writes. For each of the N listed vertices, the u value at its destination becomes the base u plus the u offset, and the v value becomes the base v plus the v offset. Other vertices are not touched. A count of 0 writes nothing.

### 16. Particle sprite batch packets

Confidence: High for the sample stride, 24-sample packet cap, packet-byte formula and output-budget reserve; Medium-High for how the caller shares the frame budget across batches. The builder rule is static; this does not establish that a normal-play frame contains enough visible same-batch sprites and remaining budget to open a second packet.

This builder turns one sprite batch of 4.7 into draw packets; it is not a ribbon or trail builder. A batch is a linked chain of samples, one per particle sprite. Each sample record holds three 16-byte vectors at offsets 0x00, 0x10 and 0x20, a float angle at 0x38 (the sprite spin angle, driven by C +0x30, 4.2 step 3), and a link to the next sample at 0x3C. Vector 0 is the sprite colour (four 32-bit words R, G, B and A, of which the low byte of each is used), vector 1 is the sprite centre in world space (its w is replaced by the sine below) and vector 2 holds the three half-extents (its w is replaced by the cosine); 16.4 gives how the draw program turns a sample into a quad (High for the roles; Medium-High for the corner layout). The builder returns the number of bytes written.

16.1 Sample output (0x30 bytes per sample). Vector 0 is copied unchanged. Vector 1 is copied, and its w component is replaced by Q1. Vector 2 is copied, and its w component is replaced by Q2. Their other components are copied unchanged. With a the sample's angle:
- Q2 = P(fold(t2)) with t2 = -|a| / (2 pi);
- Q1 = P(fold(t1)) with t1 = -|a - pi/2| / (2 pi);
- fold(t) = |d - 0.5| - 0.25, where d = round(t) - t and round is the nearest whole number;
- P is the series of section 12 with the same stored constants.
If a is exactly 0, Q1 is 0 and Q2 is 1.0. For positive a, Q2 approximates cos(a) and Q1 approximates sin(a). Test vector 52 gives the check. The series is accurate only where the fold lies between -0.25 and 0.25, the same limit as section 12. The clean side must use the series, not the ideal cosine or sine.

16.2 Packet layout. A packet is a DMA tag quadword, then one tag quadword, then the samples (three quadwords each).
- The DMA tag carries the caller's 16-bit parameter at byte 12. Its quadword count (the number of quadwords after it) is written at byte 2 when the packet closes. The count is 1 + 3n for n samples.
- The tag quadword has a 15-bit sample counter in its low bits, raised by 1 for each sample. Its flag bit is set while the packet is open and cleared when it closes.
- Each sample occupies three quadwords (0x30 bytes).

16.3 Packet boundaries. A new packet starts at the first sample of a call, and again whenever the current packet already holds 24 samples, so the 25th sample opens a new packet. Each closed packet is submitted to the draw list with the caller's flags (the draw path is outside this spec). The last packet is closed when the call ends (Medium-Low).

16.4 Quad layout and texture orientation (the draw program that consumes the packets; High for the corner and texture-coordinate order, Medium for the screen directions, Medium-Low for the extent scale). Each sample becomes one textured quad of four vertices drawn as a triangle strip, in the packet's colour-then-(texture coordinate, position) order. Let X and Y be the transformed centre's x and y (vector 1 through the view-projection matrix, before the perspective divide) and Z its depth, with c = Q2 (cosine) and s = Q1 (sine) of the sprite spin angle, and hx, hy and hz the x, y and z lanes of vector 2 (which the particle stores as node factor x size x S per lane, 4.1 step 7). The four corners and their texture coordinates (S, T) are:

| Vertex | (S, T) | x | y |
|---|---|---|---|
| 0 | (0, 0) | X - hz c - hx s | Y + hz s - hx c |
| 1 | (0, 1) | X - hz c + hy s | Y + hz s + hy c |
| 2 | (1, 0) | X + hz c - hx s | Y - hz s - hx c |
| 3 | (1, 1) | X + hz c + hy s | Y - hz s + hy c |

All four vertices take the centre's depth Z and a texture Q of 1.0 (no perspective correction of the texture). Each corner is divided by the centre's w after the offsets are added, so the on-screen half-size is the lane value times a draw-setup scale, divided by w: sprites shrink with distance and always face the camera. The half-extents are multiplied by that scale constant of the draw setup before the offsets are added (the constant is not a property of the particle; Low for its value, which acts as the projection scale).

Reading of the table, with the GS convention that screen y grows downward: at spin angle 0 the sprite is axis-aligned; S = 0 is the left edge and S = 1 the right edge; T = 0 is the top edge and T = 1 the bottom edge, so the picture is upright and not mirrored. hz is the half-width (horizontal, each side of the centre); hx is the distance from the centre up to the top edge (T = 0); hy is the distance from the centre down to the bottom edge (T = 1). A node whose lane for hy is zero therefore grows only upward from its point. A positive spin angle turns the picture counter-clockwise on screen (the right edge swings up, the bottom edge swings right). The lane order of the node factors is in 4.1 step 7.

Drawing flags and dropping (Medium): the first two vertices of every quad carry the no-draw bit of the position register, so each quad is its own strip of two triangles and quads do not join. A sprite whose centre is at or behind the camera plane (w at or below 0) has all four vertices marked no-draw and is not drawn; a second pair of range tests drops a sprite with corners far outside the screen window (Low for the exact limits).

Test vector (synthetic, scale 1): centre (X, Y) = (100, 50), hz = 2, hx = 3, hy = 1. Spin angle 0 (c = 1, s = 0): vertex 0 (98, 47), vertex 1 (98, 51), vertex 2 (102, 47), vertex 3 (102, 51). Spin angle pi/2 (c = 0, s = 1): vertex 0 (97, 52), vertex 1 (101, 52), vertex 2 (97, 48), vertex 3 (101, 48).

16.5 Budget. The builder continues only while the bytes written so far plus 0xC0 (192 bytes) is strictly less than the caller's remaining budget. This is a reserve check, not a packet-size limit; a small remaining budget can stop the builder before it reaches 24 samples. It returns the bytes written, including the packet headers. The batch flush passes its remaining frame-buffer budget to each of the up to twenty sprite batches of 4.7, in order, subtracts each returned byte count, and reports the number of 0xB0 (176-byte) blocks used. High for the reserve comparison and batch budget flow.

### 17. Trail slots with a countdown

Confidence: Medium-Low for the steps; Medium-High that they never run. Nothing calls the per-tick routine of this trail object or the routines that build it, and no table in the boot program holds their addresses. So the trail slots never run in play, and nothing sets the head speed. The clean side may leave the object out; the steps below are kept for reference.

A trail object has three slots. Each slot follows one segment of a sample chain, and its state is an 0x80-byte record. Each tick the three slots are processed in order, and then the trail's countdown runs.

17.1 Slot test. A slot is processed only when its active flag is set and both of its sample pointers (the segment's start sample and its target sample) are non-null.

17.2 Head movement. The slot's head point starts at the segment start. While the slot is not reached, the head moves toward the target sample's point by at most the trail's speed per tick (a float in the trail record, Open question 98). When the head would reach or pass the target, it is set to the target and the slot is marked reached. A reached slot copies the target point each tick.

17.3 Advance. A reached slot that is not the last segment of its chain is flagged to advance, and the chain then moves the slot on to the next segment (Medium-Low).

17.4 Countdown. The trail's countdown is one word, applied once per tick after the three slots:
- If it is above 0, it is reduced by 1.
- If it is 0, it is set to -1 and the trail is ended (released).
- If it is -1, the trail's re-arm test is run. If the test passes, the countdown is set to the whole part of 1.0 x 60 = 60 ticks (1.0 second, the constant at 0x2DB498).

### 18. Eight static effect records (swing-trail colour gradients)

Confidence: High for the address, the count, the stride, the field types, the start-up write and the roles of +0x00 to +0x40; Medium for the roles of +0x44 and +0x48. The field values are given below by user decision.

Table: 8 records of 0x50 bytes, stored back to back in the run-time area from 0x370DF0 to 0x37106F; record k (k = 0 to 7) starts at 0x370DF0 + 0x50 k. The area lies beyond the end of the file image (VA 0x32B214), so it is zero in the file. These records are the colour gradients of the swing trail (3.8): the trail start picks one by the attacker's element (the kind), and the trail's sample and colour rules read the fields below.

Start-up write. A start-up initialiser (one of the start-up table entries of SPEC-0026, called once at boot with the arguments (1, 0xFFFF), before the main body; nothing calls it again) clears each record and then writes its fields. It reads no source table: every value comes from its own code, so no data address in the file image holds these records. Their values are given in the static record values table below.

| Offset | Size | Type | Role | Confidence |
| --- | --- | --- | --- | --- |
| +0x00 | 16 | 4 f32 | colour A, the head colour of a new sample: R, G, B, A in GS units (128 = 1.0) | High |
| +0x10 | 16 | 4 f32 | colour B, the middle colour, reached after c updates | High |
| +0x20 | 16 | 4 f32 | colour C, the tail colour, reached at the cap and kept by a retired sample | High |
| +0x30 | 4 | i32 | count c: updates from colour A to colour B | High |
| +0x34 | 4 | i32 | cap n: sample life in updates; at age n + 1 the sample is retired | High |
| +0x38 | 4 | f32 | fade distance d0: at or below it a sample's alpha factor is 0 (units moved per update) | High |
| +0x3C | 4 | f32 | fade distance d1: at or above it the alpha factor is 1 | High |
| +0x40 | 4 | i32 | colour steps per update when no sample was appended that update | High |
| +0x44 | 4 | f32 | texture u per unit of trail length | Medium |
| +0x48 | 4 | f32 | u offset used by the draw's wrap; 0 in every record | Medium |
| +0x4C | 4 | zero | unused (cleared) | High |

In seven records the R, G and B of colours A, B and C are the same, so those trails only fade; record 7 (kind 7) runs from red through green to blue. The alpha differs between A, B and C (48, 40, 2) and is the same in all eight records. The sample, fade and ageing rules that use these fields are in 3.8.

Static record values: swing-trail colour gradients (High for the values; High for the roles of +0x00 to +0x40, Medium for +0x44). Each record holds the head, middle and tail RGBA colours of a weapon swing trail in GS colour units (128 = 1.0), the number of updates from the head colour to the middle colour, the sample life cap in updates, two fade distances, the colour steps per update and the texture-u step per unit of trail length. The kind column is the kind of 3.8 that selects the record; the kind is the attacker's element. These values are published here by the user's approval, as a limited exception to the rule against copied tables; the clean side uses them as constants. Each float is written with enough decimal digits to give back the exact 32-bit float. The 8 bytes at +0x48 to +0x4F are zero in every record. Integer fields are written in decimal.

| Record k | Kind | Address | Head R | Head G | Head B | Head A | Middle R | Middle G | Middle B | Middle A | Tail R | Tail G | Tail B | Tail A | +0x30 head-to-middle updates | +0x34 sample life cap (updates) | +0x38 fade distance d0 | +0x3C fade distance d1 | +0x40 colour steps per update | +0x44 texture u per unit of trail length |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| 0 | 0 | 0x370DF0 | 128.0 | 128.0 | 128.0 | 48.0 | 128.0 | 128.0 | 128.0 | 40.0 | 128.0 | 128.0 | 128.0 | 2.0 | 2 | 14 | 0.02 | 0.3 | 1 | 2.8 |
| 1 | 1 | 0x370E40 | 255.0 | 0.0 | 0.0 | 48.0 | 255.0 | 0.0 | 0.0 | 40.0 | 255.0 | 0.0 | 0.0 | 2.0 | 2 | 14 | 0.02 | 0.3 | 1 | 2.8 |
| 2 | 2 | 0x370E90 | 0.0 | 128.0 | 0.0 | 48.0 | 0.0 | 128.0 | 0.0 | 40.0 | 0.0 | 128.0 | 0.0 | 2.0 | 2 | 14 | 0.02 | 0.3 | 1 | 2.8 |
| 3 | 4 | 0x370EE0 | 0.0 | 0.0 | 128.0 | 48.0 | 0.0 | 0.0 | 128.0 | 40.0 | 0.0 | 0.0 | 128.0 | 2.0 | 2 | 14 | 0.02 | 0.3 | 1 | 2.8 |
| 4 | 5 | 0x370F30 | 0.0 | 128.0 | 128.0 | 48.0 | 0.0 | 128.0 | 128.0 | 40.0 | 0.0 | 128.0 | 128.0 | 2.0 | 2 | 14 | 0.02 | 0.3 | 1 | 2.8 |
| 5 | 6 | 0x370F80 | 128.0 | 0.0 | 128.0 | 48.0 | 128.0 | 0.0 | 128.0 | 40.0 | 128.0 | 0.0 | 128.0 | 2.0 | 2 | 14 | 0.02 | 0.3 | 1 | 2.8 |
| 6 | 3 | 0x370FD0 | 128.0 | 128.0 | 0.0 | 48.0 | 128.0 | 128.0 | 0.0 | 40.0 | 128.0 | 128.0 | 0.0 | 2.0 | 2 | 14 | 0.02 | 0.3 | 1 | 2.8 |
| 7 | 7 | 0x371020 | 128.0 | 0.0 | 0.0 | 48.0 | 0.0 | 128.0 | 0.0 | 40.0 | 0.0 | 0.0 | 128.0 | 2.0 | 8 | 14 | 0.02 | 0.3 | 1 | 2.8 |

### 19. Effect anchor chain and trigger

Confidence: High for the owner, the scale value, the hit value and the target; Medium-High for the candidate order and the test sense; Medium for the sweep length.

An effect may hold a chain of anchors: two end anchors A and B with velocity vectors (A at +0x20, B at +0x40 of the chain), a mirror slot for each end that receives the same point, and a per-chain scale value at +0x70. A trigger that fires the effect once is attached. The only user in the boot program is the Gelert Assassin's light arc (SPEC-0013, light arc): its set-up is the only code that builds a chain (High).

19.1 Chain update (each tick). Point A becomes A plus its velocity times the scale value, and point B likewise with its own velocity. The mirror slots copy the new points. The scale value is set once by the light arc's set-up: the light-ball speed (the whole number 20 at boot data word 0x2DAD48) divided by the frame-rate word (60, SPEC-0007), so 1/3, and the velocities are directions. Each ball therefore moves 20 units per second, 1/3 unit per tick (High). If the chain has a frame record, the frame is rebuilt from the two end points: the first vector is the normalised direction from A to B, and three companion vectors are built from it with the reference direction, world up (section 14.1 style).

19.2 Trigger. Once the trigger has fired, nothing else happens. Until then, each tick it tests the chain's three links in a fixed order, skipping a link that is absent:
- First and second links: a contact test between the hero's body (the radius and height of the hero's motion record, SPEC-0040) and a sphere of radius 2.0 at the link's position. A non-zero result means that they overlap, and the trigger fires.
- Third link: a sweep of radius 1.0 from the hero's mid-body point (the hero's position plus half its height straight up) along the link's normalised direction vector, over the link's stored length. A non-zero result means that the sweep meets the link, and the trigger fires.
So "success" means contact: the trigger fires when a ball touches the hero, not when a path is clear (Medium-High).

19.3 Firing. On any success the chain calls the hero's take-hit method (SPEC-0040, Method slots), never the link's object, with the chain's value at +0x78 as the damage, attacker id 0, no flags and mode 3. For the light arc that value is the light-arc damage, the whole number 20 at boot data word 0x2DAD50 (SPEC-0013). The fired flag is then set and never cleared by this trigger, so one arc deals its damage once (High).

### 20. Position jitter

Confidence: High for the two modes, the constants and the ramp component; Medium-High that it never runs; Low for the starting value of the ramp. The jitter belongs to a set of twelve 0xD0-byte sprite records. The routine that updates the twelve records has no caller and no table holds its address, and the routines that set the records up have no callers either, so the jitter never runs in play.

An effect with a jitter mode (u32 at +0x04 of its record) changes its position vector (at +0x40) on each tick:
- Mode 1 (random): each of x, y and z gets a uniform random value in the range -0.01 to +0.01 added (three draws). The range is set by fixed constants; the draws use the run-time random helper (table at 0x39BE50).
- Mode 2 (ramp): the ramp value R (a float at +0xC0) is raised by 0.005 while it is below 0.075. Then R is subtracted from the y (vertical) component of the position, so the record sinks. In 32-bit float steps from 0, R is just below 0.075 after 15 steps, so a 16th step runs and R ends at 0.08.
- Other modes do nothing.
After either mode, a frame builder rebuilds the record's frame from its vectors and the new position (Medium).

### 21. Triangle oscillator blend (60-tick cycle)

Confidence: High for the counter, the fraction and the gates; High for the float store and the lane mask.

An owner object with a set of slots can move the vectors of the objects in those slots toward a reference vector, following a triangle wave with a period of 60 ticks (one second). This wave is separate from the sway oscillators of section 9.

21.1 Counter. The owner holds a signed 32-bit counter. At the start of each tick, if its counter-enable word equals 1, the counter goes up by 1. The blend in the same tick uses the new value.

21.2 Fraction. Let t be the counter modulo 60. The fraction is f = t / 30 when t is from 0 to 30, and f = (60 − t) / 30 when t is from 31 to 59. So f rises from 0 at t = 0 to 1 at t = 30 and falls back to 0 at t = 60. The factor 1/30 that scales t is the boot data word at 0x31F5F0, not the physics-block word at 0x3A1A74 (Where it lives; Medium).

21.3 Blend gate. The blend runs only when the blend-enable word equals 1. This gate does not depend on the counter-enable word. When it is off, nothing is blended.

21.4 Slots and blend. The owner has 15 slots at 8-byte steps from offset 0x1340 (slots 0 to 14), each holding a reference to an object. These are the character's 15 colour slot records: the character colour push of SPEC-0040 writes colour j, as (byte0/255, byte1/255, byte2/255, 1.0), to +0x10 of slot record j, which is the vector this blend moves (High). The push reaches record j through +0x1340 + 8j for display buffer 0 and +0x1344 + 8j for buffer 1. The slots are handled in order:
1. The slot object's lane mask (the word at +0x20 of the object) is rebuilt: bits 0, 1 and 2 (x, y and z) are set when the owner's word at +0x12A0 is non-zero, and bit 3 (w) is set when the owner's word at +0x12A4 is non-zero. Every caller passes 1 for both, so the mask is 15.
2. The object's vector at +0x10 moves toward the owner's reference vector at +0x1290. Its x, y and z components each become (1 − f) times the old component plus f times the reference component. Each result is stored as a 32-bit float; there is no whole-number conversion. The w component is not changed.

21.5 Accumulation. Each tick's blend starts from the value stored by the previous tick, not from a fixed base. The stored value therefore depends on the earlier ticks as well as on t, and over many cycles it drifts toward the reference. The clean side must keep the stored value between ticks. A clear (21.6) does not write the colour records: they keep their last blended values, and the draw uses those values while the slot is off. So after a poison or stun ends, the hero keeps the drifted tint until a later push rewrites the records (a variant select, a bind push or a region-wide override switch). This is what the code does (High for the writes; Medium-High that the original shows it, as no other path restores the records during play).

21.6 Writers (High). This blend is colour-override slot 0 of the render object (SPEC-0046 draw step 6). The slot-0 set writes the reference vector at +0x1290, the two lane words at +0x12A0 and +0x12A4 and the mode at +0x12AC (the counter-enable word), and turns the slot on (+0x1280 = 1, the blend-enable word). When the slot was not already on, it also sets the counter: 0 for mode 1, 30 for any other mode. The clear turns the slot off. With mode 1 the colours swing toward the reference and back once per second. With another mode the counter stays at 30, so f = 1 and every slot colour is set to the reference on the first tick. Two routines set slot 0 (one with mode 1, one with mode 0). Two others set slot 1 at +0x12B0 (the hit flash among them); slot 1 is read by the part draw as the object colour (section 32; SPEC-0040, Behavior 26, item 9).

### 22. Twelve-sprite grid (dormant)

Confidence: High. This is not an effect that plays. The routine has no caller, no code pointer and no address that reaches it in the boot program, so it is unreached; the clean side need not implement it.

For reference only: the owner record holds a state word at +0x00, a centre vector at +0x10, an axis vector at +0x20, and twelve sprite records of 0xD0 bytes; sprite (row r = 0 to 2, column c = 0 to 3) is at +0x30 + 0x340 r + 0xD0 c. The sprites form a 3 x 4 grid, not a ring: position = centre + 8.0 c x axis + 8.0 r x normalise(cross(up, axis)), with up = (0, 1, 0). The tested length is the full 3D distance from the centre to the reference object's point (its +0x50). Beyond 50.0 the grid is built once (state 1); at 50.0 or less the state becomes 2 and each sprite is reset.

### 23. Spark spawner (pointer)

Confidence: High. This is not an effect. It is one step of the world's +0x29F0 record list (0x160-byte records: leashed or floating world objects), owned by the world spec (SPEC-0028, with the pass in SPEC-0026). The step's loop returns after item 0, so only item 0, a world object found by the record index, is placed (its frame and visibility); the tests for items 1 to 4 (thresholds 2, 4, 6 and 8) never run.

### 24. Timed slot pool of 48 (pointer)

Confidence: Medium. The 48-slot pool is the cue voice pool of the sound-bank spec (48 voice records), not an effect pool. SPEC-0031 (Cue playback) owns its rules.

### 25. Step tween (pointer)

Confidence: High for the step operations and modes 0 to 3; Medium for the scalar's unit. The dormant step tween is the world's +0x3850 tween-node list (0x140-byte records), owned by the world spec (SPEC-0028). Its +0x130 word is an active flag, not an elapsed timer. On each world update while active, it increments the completed-step byte once and adds source × (+0x134 / total steps) to position +0xA0. The scalar +0x134 is initialized to 1.75, but its unit is unknown. Modes 0 and 2 use the vector at +0x20; modes 1 and 3 use +0x40; modes 2 and 3 negate the source. When completed steps already equal the total, the active flag is cleared with no movement. Other mode values have no established source rule. See SPEC-0028 section 15.1 for the full rule and a synthetic check. The attached-part variant is a separate movable-block update path (SPEC-0048), not another effect tween-node mode. The world update walks its movable-block records; while a record is active, its update processes the attached-part list (a count and 32-byte entries) and refreshes each non-null part transform using the block record's updated pose. SPEC-0048 owns the block motion and attached-effect behavior; this effects spec does not assign those parts to the +0x3850 tween-node pool.

### 26. Particle burst (ten pieces)

Confidence: High for the start, lifetime, ten-entry target list, positional cue request, and the three update-pass call paths. Medium-High for the cell-primitive contact response (the selected primitive tests and the response effects are observed; exact geometry semantics remain owned by SPEC-0018). Other burst callers and whether they alter these behaviors remain open.

1. A burst starts from a start point and a duration s in seconds.
2. Lifetime: L = s x 60, truncated to whole ticks (rounded toward zero, so a negative s gives a negative L). A lifetime of zero or less ends the burst on its first update.
3. The burst's origin is set to the start point, its age is set to 0, and its tracked value is set to the second component of the start point.
4. At the origin, the burst requests positional sound cue 5 through the sound service. It does not start an emitter effect on this path. It also clears ten 8-byte recent-target records, numbered 0 to 9.
5. On the actor-event A7 path, the burst also performs an area-hit scan. A qualifying target takes damage equal to its A7 attack-attribute value multiplied by the shared damage scale. The burst records the target's actor identity so it cannot be hit again by that burst while the record remains active.
6. The ten 8-byte records form a recent-target list. A successful hit claims the first free entry and stores the target identity with a 60-tick countdown. Each burst update lowers occupied countdowns by one and frees an entry at expiry. A target can therefore be hit again by the same burst after its record expires, if the burst is still active and the collision test succeeds. The handler path observed for this attack starts the burst with a 10-second lifetime; this duration is not established for every possible caller.
7. Each tick, while the burst runs:
   a. The age rises by 1.
   b. The new origin is read from the burst's owner. Jump guard: if the new second component differs from the tracked value by more than 1e8 (absolute difference, strictly greater), the tracked value is used in place of the new second component. The origin is then stored, and the tracked value is set to the stored second component.
   c. The positional cue-5 voice is refreshed at the current origin.
   d. If the age is below L, it ages the ten target-suppression records, scans actor candidates for area hits, then checks the current cell's primitive list for contacts. Kinds 1 and 3 use their respective contact tests; kinds 4 and 7 select a test by whether their radius is positive; kind 2's test always reports no contact. For a successful contact, when the world-busy check is clear and the primitive state is 0 or 5, the response applies a 0.25-unit normalized push. It also requests cue 143, 144, 145 or 146 according to the primitive's short contact field (codes 2, 3 and 4 select 143, 145 and 146; other values select 144). It does not update ten visual pieces. Otherwise the burst finishes.

### 27. Effect bank reset (record and entry sizes)

Confidence: High for sizes, entry clearing and record-reset child cleanup; Low for the meaning of the entry byte at +0x67.

A bank is one contiguous block, in this order:
- Four effect records, 0x2C0 bytes apart, from the start of the bank. Each is reset by the record reset.
- Two more effect records, 0x2C0 bytes apart, starting right after the fourth.
- Five entries of 0x70 bytes, starting right after the sixth record.
- Five more entries of 0x70 bytes, starting right after the first five.

The six effect records are reset at offsets 0, +0x2C0, +0x580, +0x840, +0xB00 and +0xDC0. A record reset operates on its effect sub-object at +0x230: it visits the five child links, conditionally clears a child's backlink and parent link when the backlink points to this sub-object, invokes the child's stop routine, and clears the parent link. For a child whose live-child count is below 1, the stop routine sets state 4 if the child is not already in state 4; in that transition it runs the callback when present and releases the buffer when present. A child with a live-child count of 1 or more enters state 5 and has its vector at +0x50 cleared. The sub-object is then reinitialised. The callback's external effect is not established.

Entry reset: the 48 bytes from entry offset 0x40 to 0x6F are cleared to zero, except the byte at offset 0x67, which is set to 255. The bank's table pointer field is set to a fixed built-in table (its contents are not described here).

### 28. Keyed claim over five slots

Confidence: Medium-High; the busy test rule is Medium.

Five slots of 0xA0 bytes lie in a row. Each slot holds a key pair (two 32-bit words, first and second), a data pointer word at offset 0x10, and five child pointers at offsets +0x68, +0x6C, +0x70, +0x74 and +0x78. For the keyed claim and bank slot checks, a slot is busy if any child pointer is non-null and the pointed object's 32-bit word at +0x90 is not 4. The data pointer at +0x10 is not itself the busy-test value.

Claim with key pair (A, B):
1. Scan the five slots in order, noting the first free slot seen. Stop at the first busy slot whose key pair equals (A, B).
2. If such a duplicate was found, the claim is refused and the result is the null pointer (0), even when a free slot exists.
3. Otherwise the first free slot takes the key pair (A, B) and its data pointer (offset 0x10) is returned. If no slot is free, the result is null (0).

### 29. First-free search over ten entries

Confidence: High for the pool size, order and countdown busy test.

Ten entries of 0x3D0 bytes (976) lie in a row. Each entry is tested once, in order. Its busy test reads the 32-bit countdown at offset 0x300: non-zero means the entry is occupied, and zero means free. Starting the controller sets this countdown to duration × the frame rate; each active update decrements it by one, and when it reaches zero the entry is reset. The controller contains its own effect instance at offset 0x330. This ten-entry pool has a 0x3D0-byte stride and is distinct from the clip-animation effect pool in section 3.1, whose ten 0xB0-byte slots contain an effect instance at +0x10 and use a separate boolean active word. The first free entry is returned. When all ten are occupied the result is null (0). Test vector: at 60 ticks/s, starting with duration 2 seconds sets the countdown to 120; after 119 active updates it is 1, and the next update reaches zero and frees the entry.

### 30. Find-or-create key table (groups of eight slots)

Confidence: Medium overall. The search/write rules, blackboard-message role, and the specific type-1, type-2, and type-3 uses described below are High; a general meaning for any message type or payload remains open.

The table is a sequence of groups. Each group is 0x40 bytes and holds eight slots of 8 bytes; a slot holds a key word (first 4 bytes) and a value word (next 4 bytes). The caller names the table, a group index, a key and a value. The backing store is a global 193-group block of 0x3040 bytes at 0x0034A810, selected through a global table pointer and cleared during initialization and the manager reset path. A matching-key read returns the slot's value word; a miss returns -1. The script-facing wrappers identify this as the blackboard message store (SPEC-0030 rows 149–150): the group is a world identifier, the key is a message type, and the value is its payload. Level-script callers demonstrate type 2 as an arrival tag carried across a world change. For example, a script writes type 2/value 30 for world 31 before leaving for world 44; when world 31 activates, it reads that entry, moves the hero to locator 0 when the value is 30, and clears the value to -1.

Other types also carry script-specific transition information. In the world-129 return route, pack 1217 writes type 2/value 2 and type 3/value 2 before loading world 129; its activation callback in pack 1184 reads type 3/value 2, moves the hero to locator 50, sets the camera to normal, then clears both messages for world 129. In the same callback, type 3/value 1 instead moves the hero to locator 45, walks to locator 46 and restores the normal camera. Pack 1220 writes type 3/value 1 for world 129 on its return route. On the forward route, pack 1184 writes type 3/value 1 for world 131 before entering pack 1220; that pack reads the value on activation and starts the hero’s long fall at locator 25 with camera track 4. Its subsequent clear is addressed to world 132, so the type-3 entry for world 131 is not shown being cleared there. Pack 1184 also writes type 3/value 1 or 0 for world 130 on separate replacement-load paths, with no matching type-3 reader recovered for that world. These examples establish route-specific arrival behavior, not a global type-3 value dictionary.

Type 1 has demonstrated arena-result uses. Pack 1176 writes type 1/value 1 for world 90 when the watched AI is dead and the arena-result gate is clear, alongside type 2/value 10; pack 1023 reads type 1 on that return and sends value 1 through the win-rank progression. On the hero-death route, pack 1176 writes type 1/value -1 alongside the same type 2/value 10 before returning to world 90; pack 1023 sends -1 through the loss route. A lookup miss also returns -1. A second arena return uses world 157: pack 1371 writes type 1/value 1 for a win or -1 for a loss alongside type 2/value 20 before loading world 157. Pack 1350 reads type 1 when the type-2 value is 20; value 1 selects the win progression, while any other value selects the loss progression. The activation path then clears type 1 to 0. These examples establish result payloads on two arena routes, not a global meaning for type 1. The meanings of other message types and payloads are not uniform or established by these examples.

Find-or-create:
1. Search the group for the first slot whose key equals the key. If found, use it.
2. Otherwise search the same group for the first slot whose key is 0 (an empty slot). If found, use it.
3. If a slot was used, write the key and the value into it and report success. Otherwise report failure and change nothing.

A found slot has its value overwritten. A request for key 0 matches in step 1 at the first empty slot.

### 31. Detach (mode 2) and stop sweeps

Confidence: Medium-High for mode 2; Medium for the sweeps.

An effect object has five child-link words in a row (offsets +0x58 to +0x68). A detach call with argument A:
1. It visits all five links in order. A null link is ignored. For a non-null child, it clears the child's +0xC4 and +0xBC words only when the child's +0xC4 backlink equals this effect object.
2. If bit 0 of A is set, after visiting the links it calls the effect object's release helper. The helper does not release linked children.

Mode 2 (bit 1 only) therefore detaches the five links and keeps the object. Mode 1 or 3 detaches and releases.

Stop sweep one: nine effect objects are detached with mode 2 in one pass, walked from the highest slot down: eight objects spaced 0xB0 bytes apart, then one more object at a separate place. Stop sweep two: two sub-objects are detached per step, down to a fixed end (the step count is not described).

**Group stop (partly resolved).** The group stop handles three structural sets: one effect object at +0x5D0; a controller at +0x280 whose stop path detaches its linked effect object at +0x10; and four effect records at +0, +0xA0, +0x140 and +0x1E0. The group initializer resets the four records and initializes the controller. The update path steps the controller timer and updates only active members among the four records. The gameplay category of each set and the producer of the separate +0x5D0 object are not established. The loop across +0x3A0 through +0x560 has no effect on these records. When bit 0 of the stop flag is set, the group object is released. **[High for layout and dispatch; Low for gameplay categories]**

### 32. Hit-flash keys (start-up initialiser)

Confidence: High for the values; Medium-High for the ramp; Medium for which hits start it; Medium-High that the draw shows it as a constant reddish scale (SPEC-0046 owns the draw).

The block is the key set of the hit flash: a short colour ramp on a character's render object when it is hit. It is a run-time block of 80 bytes at 0x374830. A start-up initialiser (a start-up table entry of SPEC-0026, called once at boot with the arguments (1, 0xFFFF)) clears it, then writes:
- Key vector 0 at offset 0x00: (0.5, 0.5, 0.5, 0.0).
- Key vector 1 at offset 0x10: (1.9, 1.7, 1.9, 0.0).
- Key vector 2 at offset 0x20: (1.0, 0.8, 1.0, 0.0).
- Key colours, four bytes each (R, G, B, A): 0x30 = (200, 0, 0, 128); 0x34 = (200, 0, 0, 128); 0x38 = (0, 0, 0, 128).
- Stage lengths in frames: +0x3C = 4 (key 0 to key 1) and +0x40 = 10 (key 1 to key 2). Each is a signed 32-bit integer read as an integer and converted to float when forming its reciprocal duration.

Called with any other arguments it writes nothing; in play it is never called again after boot.

Flash (Medium-High). A hit on the hero, or a hit through the enemy class's hit method (Medium for that role), starts a 14-frame two-stage ramp on the hit character's render object: the vector goes from key 0 to key 1 over 4 frames, then to key 2 over 10 frames, and the colour bytes go from key colour 0 to key colour 1, then to key colour 2 (red fading out). While the ramp runs, the render object's colour override slot 1 holds (1.5, 0.5, 0.5, 1.0), a reddish tint.

The flash tint is visible (Medium-High). The blend of section 21 reads slot 0 only, but the part draw reads slot 1 as the object colour: while slot 1 is on, every part of the character is drawn with the slot-1 vector (1.5, 0.5, 0.5, 1.0) as its colour scale in place of its own colour slot (SPEC-0046 draw step 6; SPEC-0040, Behavior 26, item 9). Nothing outside the ramp's own start, step and stop reads the ramp's vector or its colour bytes, so the ramp values below are state only. The clean side draws the whole character with the red-biased scale for as long as the ramp keeps slot 1 on, then returns to the slot colours. The separate one-shot effect 222 or 220 (3.16) is not affected. The data word 0x2D7600 (0 on disc) forces the flash on the hero when non-zero.

Test vector (computed, taking each stage as linear): frame 2 of the flash gives the vector (1.2, 1.1, 1.2); frame 4 gives key 1 (1.9, 1.7, 1.9); frame 9 gives (1.45, 1.25, 1.45); frame 14 gives key 2 (1.0, 0.8, 1.0), and the flash ends. These are state values only; the ramp vector itself is not drawn, and the tint on screen is the constant slot-1 vector during these frames.

### 33. Sub-effect timer and its step

Confidence: High for the parameter roles and per-tick scalar update; Low for the child field’s physical or visual meaning and for whether the target is reached on every run.

Starting a sub-effect with duration D seconds, step S, scale K, initial scalar A and upper target B:
1. If the sub-effect is already active (its active count is nonzero), nothing changes.
2. Otherwise the active count is set to floor(D x 60) ticks; the per-tick scalar increment is set to S / 60; the scale is set to K; the initial scalar is set to A; the upper target is B; and the sub-effect is bound and started.

Update rule: while active, the countdown decreases by one tick first. If it reaches zero, the timer resets immediately and that tick skips the scalar advance and child updates. Otherwise, the scalar advances by its stored per-tick increment toward the upper target, and the updated scalar is passed to the linked child records as round(scalar x 256) in each child signed 16-bit field at +0x68. The reset clears the active count, scalar, target and step, restores the base value and calls the associated stop/unbind helper; the helper visible effect is not established. The upper target is a clamp, not proof that every run reaches it. When the sub-effect is not active, the base update starts it again with duration 10.0 seconds, step 1.8, scale 1.5, initial scalar 2.0 and upper target 25.0.

Second timer variant: when the sub-effect is not active, it starts with duration T / 12.8 seconds, step 12.8 (stored as 12.8 / 60 per tick), scale supplied separately, initial scalar 1.0 and upper target T, where T is the input of that update (High for roles; physical meaning of T and the child value remains open).

### 34. Placement along a direction

Confidence: Medium for the source move; Low-Medium for the child rule.

Let P be the source point and G the reference point (the position of the global object used for targeting).
1. If the source's object lookup finds an object, compute the unit direction u = (G - P) / |G - P|.
2. Move the source: P = P + 5.0 u.
3. For each child point in the object's child list (up to the child count, skipping empty entries), set the child point to 5.0 units along the same unit vector u from the children's base point. The exact base point is an open question.

The routine returns a status value whose meaning is not established. A zero distance has no defined direction (open question).

### 35. Rotation by angle (degrees) and the axis variant

Confidence: High for the angle formulas, helper roles and axis variant assignment.

Rotating an effect object by angle p (degrees) about axis (x, y, z):
1. Half angle h = p x (pi / 180) x 0.5, in radians.
2. Quaternion = (x sin h, y sin h, z sin h, cos h). The axis is used as given; this routine does not normalise it.
3. The quaternion is written as the object's orientation, if the object is found.

Axis variant: the axis is (cos q, 0, sin q), where q is the first parameter used directly as radians (no conversion), and the angle is the second parameter in degrees; steps 1 to 3 then apply.

### 36. Save-point glow

Confidence: High for the definitions, the rules and the emitter values below; Medium for the picture they make.

The world pack's own character region owns one save-point glow (SPEC-0025, Save points; SPEC-0006 kind 4). The glow is a pair of emitters made from effect definitions 400 and 404 of shared effect group 3, the slot-3 definition list of the shared effect manager (3.7), which is the definition table of resource 9. Both definitions are present in that table.

- Load. The pair is loaded once, the first time it is needed. If it is already loaded, nothing happens. Otherwise the shared effect manager is asked for group 3; if the group is found, one emitter is set up from definition 400 and the other from definition 404, and the glow is marked loaded. If the group is not found, the glow stays unloaded and every later step does nothing.
- Start. If the glow is loaded and not running, both emitters are started and the glow is marked running. Starting a running glow does nothing.
- Move. Both emitters are moved to the same position (the chosen save point's position).
- Update. Every tick, if the glow is loaded and running, both emitters are updated with a 64-byte block (4 x 4 floats) that the region passes in; it is taken to be the frame's camera matrix (Medium).
- Stop. If the glow is running, both emitters are stopped and the glow is marked not running.
- Use (SPEC-0025). When the nearest save point changes, the region stops the glow, moves it to the new nearest save point and starts it again. The glow is not distance-gated, so it sits at the nearest save point at any distance. A region reset stops it.
- The glow is not a placed effect (type 39) and no placed record is tied to a save point (SPEC-0028).

Look (values from resource 9). Both definitions have 5 emitters, the near/far threshold 625 (25 units from the listener, 5.2), no cull (-1) and the cone test on.
- Definition 400, all emitters near only (state 0): an additive puff every 0.25 s (life 1.5 s, drifting up); a slowly rising, spinning child model every 0.4 s (life 4 s); additive sparks at 5 per second (speed 5 units/s, up to 0.7 rad off straight up, life 2 to 4 s); a looping alpha disc of 9.2 x 9.2 x 3.8 with colour (0.5, 1, 1) at alpha 0.15 (node 446, test vector 72); and a spinning child model every 0.25 s.
- Definition 404: near, a child model every 0.7 s and a looping light emitter (4.5) of cyan (0, 1, 1) with radius 20; far (state 1, beyond 25 units), additive sparks, cyan sparks of colour (0.27, 0.89, 0.94) and a looping alpha disc of 10.2 x 10.2 x 4.0.
So within 25 units the save point shows the full glow (defs 400 and 404 near) with a cyan point light of radius 20; farther away it shows only 404's sparks and its larger disc.

### 37. Hazard-surface effects 129 and 188

Confidence: High for the values (resource 9); Medium for the picture. These are the effects of two deadly-ground surfaces (SPEC-0055).
- Effect 129 (0x81, surface 2): three single bursts of 3 particles each: an alpha blob thrown up at 4 units/s with vertical acceleration -7, an additive glow, and additive sparks at 8 units/s with vertical acceleration -10. A hot burst.
- Effect 188 (0xBC, surface 8, water): two child-model emitters: a burst of 35 thrown at 7 units/s, 36 to 63 degrees off vertical, with vertical acceleration -9; then 3 every 0.2 s for 3 s. A water splash.

### 38. Foliage sparkle effect 139

Confidence: High for the values (resource 9); Medium for the picture. Definition 139 of shared effect group 3 is the sparkle that a grown coin bush or an unclaimed Negg bush starts at itself every 18 to 42 ticks (SPEC-0060). It is one definition with one emitter: near/far threshold 4900 (70 units), cull 4900, cone test on, near only.

- Emitter: one burst of 1 particle, then the spawner drains (negative base period, 3.2). So each start makes exactly one sprite.
- Material: an additive group with a 64 x 64 texture; sprite half-extent factor 0.5 on each axis.
- Life: 2.0 s plus U(-0.3, +0.3) s.
- Motion: launch speed 2.5 plus U(-0.5, +0.5) units/s at 1.555 to 1.57 rad from the launch axis (almost at right angles to it); vertical acceleration +1.5 units/s^2 (up); drag 3.0 per second; spin rate U(-1.5, +1.5) rad/s; kicks of strength 0.2 plus U(-0.1, +0.1) every 0.2 plus U(-0.1, +0.1) s.
- Colour: start (1.0, 1.0, 0.643), pale yellow, constant over the life. Alpha starts at 0. Its keys are at life fractions 0, 0.338, 0.389, 0.935 and 1.0: alpha stays 0 until 0.338, rises at 19.53 per life to about 1.0 by 0.389, holds, then falls at 15.42 per life from 0.935 to 0 at the end.
- Size: starts at 0, grows at 24/7 (about 3.43) per life to 1.0 at life fraction 0.2917, then shrinks at 12/17 (about 0.706) per life to 0.5 at the end. The third size rate is never reached (its threshold is 2.0).

Look: a small pale-yellow additive spark. It stays invisible for about the first third of its life, then appears at full brightness at about its largest size, drifts sideways and slightly upward, slowing fast under the drag, with small upward kicks and a slow spin. It shrinks to half size and fades out in about its last 0.13 s. With a new one every 0.3 to 0.7 s, the bush twinkles.

## Edge cases and error handling

- Impact draw boundaries: the ranges are inclusive at 0, 5, 6, 19 and 20. A draw of 0 selects definition 138 alone, which is 1 draw in 101.
- Effect definition 162 does not exist (on disc the table holds 135, 136, 138 and 160, 161, 163 to 171, but not 162). The water footstep (3.16) asks for 162: the lookup finds nothing and no effect object is set up, so nothing is shown (High). The cue bag of SPEC-0021 is the Faerie Sisters' mote eater, not an item path.
- Sway bounds are strict: a phase or amplitude exactly at its bound does not turn the step.
- Sway with fewer than six attached nodes updates only the nodes that are present.
- Effect id not in the definition table: the instance is created empty and has no particles. Log and continue.
- Effect id 137 occurs twice in the table; the first match wins.
- Clip windows: a start fraction of 0 makes the start window [L, 2L) instead of [0, L), so the effect starts on the second tick of the clip. An end fraction whose product with D reaches D is moved to D − 1 frames; on a one-shot held at its last key (cursor F − 1) that stop window holds every tick. Reproduce both.
- A list entry's end fraction at or past the clip duration is set to one second before the duration.
- Attach slot sentinel: the value 0x4FFFFFFF is "none", and it occurs only once on disc (in slot A). Its low three bits are 7, so a decoder that takes the low bits without checking for the sentinel reads slot 7. Check for the sentinel first. Slot 7 otherwise appears 1,147 times in slot A, so slot counts must exclude the sentinel.
- A cycle period below 0 after jitter is clamped to 0.
- A negative kick interval base (C +0x38 < 0) means the particle is never kicked. It does not affect the life: every particle has the finite life of 4.1 step 3.
- Period 0.1 s: bursts on ticks 1, 8, 14, 20, ... (the first gap is 7 ticks, because six float32 additions of dt / p land just under 1). Reproduce the clock in 32-bit floats.
- Acceleration clock T = 0 with a non-zero acceleration: no acceleration (2 of 373 records).
- Kick interval 0: a kick on every tick (278 records; only 2 have a non-zero kick strength, giving a constant +R per tick upward).
- Kick vertical component: always +R, never a random value in [-R, R] (4.2 step 2).
- Loop restart tick: a particle that restarts uses the old life fraction for that tick's curves once (4.6 step 4).
- Light emitters (A +0x87) are never drawn as sprites (4.5).
- More than 2000 sprites in a frame, or a 21st batch key: the excess is dropped (4.7). More than 200 lights in a frame: the excess is dropped (4.5).
- Node links: the loader makes every stored link none (2.1 step 3). All 1,651 disc links are -1 and become none, so no expiring particle starts a child from disc data (4.6 step 5). Do not start a child for a none link.
- Node child byte: the child index of a node is read only when its byte at +0x67 is 0x80 or more (2.1 step 3). Use the 50 nodes that pass this gate and hold 0 to 49 as the only child nodes. The 116 nodes with a byte below 0x80 use the context lookup (2.1): their child is the model of common object byte + 283, and a byte above 0x27 gives none (High).
- Node spawner pool word: zero on disc and in an unbound group. A spawner is never created for a node whose pool word is zero, so an effect of an unbound group produces no particles (High).
- Negative base period (A +0x6C < 0) on an emitter: the spawner is started, fires one burst on its first running tick, then drains and dies when its particles have expired (3.2 step 1; all 330 such records have A +0x86 = 0). The effect object does not keep its handle (3.4). If no other slot keeps a handle, the instance stays idle (state 0).
- Placed record cull values: -2 on 4,869 records and -1 on 3. Under the rule in Section 6, only a positive value or -1 is kept; a -2 record takes the definition's cull value (High).
- Placed record direction: already unit length (w = 0). No renormalisation is needed on disc.
- Stop requests on an instance with an invalid handle are ignored (with a log).
- 8.8 write wrap: a real value of 128.0 or more, or of -128.01 or less, is stored as a wrapped 16-bit value. For example 128.0 reads back as -128.0, and -128.01 reads back as 127.98828125. The clean side must reproduce the wrap; it is the original's behaviour (Open question 37).
- Routines that take effect id 0xFFFF (High): these are the start-up initialisers, the entries of the start-up table walk (SPEC-0026), each called once at boot with the arguments (1, 0xFFFF) before the main body. Each sets its table to its default values (sections 12.5, 18 and 32 are three of them). Nothing calls them again, and their teardown branch (first argument 0) is never reached in play.
- Vibration tester with the outer radius at or below the inner radius: no rumble is requested and an error is reported. The clean side should log and skip the request.
- Pulse wave, folded phase near 0.75 (phase near 3 pi / 2): the series gives about 101 where the ideal sine gives 64. This is the largest error of the pulse (Behavior 12.4). Copy the series.
- Pulse phase A: the wrap is at 1.0 exactly; a value of 1.0 would be reduced to 0. A is always in 0 to 1.
- Pulse phase B: never reduced, so it grows without bound while the effect runs. Accumulate it in 32-bit float as the stored step is a 32-bit float; the exact float rounding the original uses is not confirmed over long runs.
- Pulse intensity rounding: the intensity is rounded half away from zero to a whole byte. The value is always in 64 to 250 for a folded phase in -0.25 to 0.75.
- Pulse presets: only byte 12 of the chosen preset is written by the pulse. The other fifteen bytes are zero until a draw copies 16 bytes into the preset (Where it lives; Open question 46).
- Pulse start-up value: the start-up initialiser sets the intensity floor word to 64.0 once at boot and does nothing else that this spec describes.
- Swing trail with a kind outside 0 to 7: the record for kind 0 is used (High).
- Swing trail with the entry's byte +0x48 non-zero: no trail is created and the holder is unchanged (High).
- Swing trail with the holder's active word already set, or with all 12 trails in use: nothing is created (High).
- Swing trail below game-state code 9 (normal play): samples do not age and the trail is still drawn; at code 9 or above, samples advance and the trail remains drawn (High).
- Hero clip lists: the hero class's per-tick update does not run the clip effect-list walk (only the character base class and the enemy class do), so the trail start of 3.8 is reached through those two classes and the hero's clip lists start no effect and no trail (High). The hero's swing trail is the held-weapon trail of 3.8.
- Type setup with a type word outside 1 to 27: nothing is done (High).
- Type setup with a total count (A +0x60) of 0: the per-burst count becomes 0 and the cycle-limit calculation divides by zero. No shipped world-object preset node has total 0 (3.9); what the original stores for a synthetic or mutated zero remains unknown (Open question 71). The clean side must not divide by zero.
- Timed emitter with no entry for its id: the count still goes down, so that attempt spawns nothing (High).
- Timed emitter with a count below 1: the enable word is cleared on that tick and nothing is spawned (High).
- Timed entry list, slot after a removal: the entry that moves into the removed slot is not updated in that tick (High; Test vector 45).
- Timed entry list, stale slot: the loop also tests the slot just past the last live entry, which holds stale data. The countdown there is decremented when the test passes. If it then reaches the removal test, the count drops without a shift, and the last live entry is lost (Medium; Open question 75).
- Timed entry list, empty: with a count of 0 the same stale slot is tested. If the test passes and its countdown is 1 or less before the decrement, the count becomes -1. A count below 0 stops the list until something else sets the count (Medium; the value of that word on an empty list is not established).
- Buffer fields: the size field is first given the formula result and then the result rounded up to 256; it remains the rounded size. The base is stored separately, and the second-half pointer is base + rounded-size/2. The fill covers exactly the rounded size. The draw routine passes the selected base plus current write offset and rounded-size minus current offset. The builder writes 0x10-byte headers and 0x30-byte samples and requires more than 0xC0 bytes remaining before proceeding. The selector's live domain and its relationship to the second half's available capacity are unresolved; full packet/header field meanings remain renderer-facing details.
- Id registry: a misaligned id returns -1 without a log; an id for an occupied slot returns -1 and logs; a failed check logs and returns 0 without storing, so a caller that tests only for zero sees success (High).

- **Mote-eater drain pick**: a pick that walks past index 11 reads one past the 12-entry Mote list and adds no slot, so the drain can have 7 slots (period 257) instead of 8 (3.13). Reproduce it.
- **Mote-eater drain speed**: each live Faerie Sister ticks the drain once per tick, so with k sisters the drain runs k times faster (3.13). Reproduce it.
- **Mote-eater drain at its cap**: with eight slots revealed, a period adds no slot; the countdown still runs (3.13).
- **Frame object lifetime at zero**: a counter that is below 1 when the refresher starts releases the object at once, with no frame written (Medium).
- **Frame object with a vertical direction**: D becomes zero after its y is removed, so X and U are zero too and the frame keeps only the anchor row (Low for the draw result).
- **Series outside its range**: in the sprite batch values, the series is off by up to about 0.4 where the fold lies between 0.25 and 0.75 (as in section 12). Copy the series; do not substitute the ideal cosine or sine.
- **Sprite zero angle**: an angle of exactly 0 gives Q1 = 0 and Q2 = 1.0 by a special case, which matches the series at zero.
- **Batch packet overflow**: a packet closes before sample 25, which starts the next packet if the caller budget permits. A full packet is 0x4A0 bytes total: 16-byte DMA tag, 16-byte GIF tag, and 24 samples of 0x30 bytes each (16.3).
- **Batch reserve**: the builder continues only while bytes written plus 192 is strictly less than the remaining budget. Equality also stops the builder; unused space can exceed 192 bytes when the batch ends early.
- **Trail countdown on re-arm**: a countdown at -1 waits, with no effect, until the re-arm test passes (Medium-Low).
- **Ramp overshoot**: the ramp stops at the first value that is not below 0.075. In 32-bit floats the 15th value is just below 0.075, so the ramp ends at 0.08 after 16 steps (High; the jitter never runs in play, section 20).
- **Trigger fires once**: after the fired flag is set, the trigger does nothing, even if the tests would pass again (Medium-Low).
- **Static record initialiser with other arguments**: it does nothing; in play it runs only once at boot (section 18) (High).
- **Preset C source with a zero direction**: C +0x00 becomes (0, 0, 0, 1) and the launch speed 0; no failure (3.9) (Medium).
- **Preset node without an A record**: it shares the first-time A record of its type, whose total count is -1, so the per-burst count becomes -1 and the cycle limit 1 (3.9) (Medium).

- **Triangle oscillator gates**: with the counter-enable word at 0 the counter does not move. With the blend-enable word at 0 nothing is blended, and the counter still moves if its own word is 1 (High).
- **Triangle oscillator fractions**: a blend that produces a fraction stores the fraction as a float; a small move from 0 is kept (High).
- **Triangle oscillator repeated blends**: the stored value moves toward the reference on every tick, so repeated cycles do not repeat exactly (21.5).
- **Twelve-sprite grid**: unreached in the original (section 22); nothing needs to reproduce it.
- **World record step (section 23)**: only item 0 is placed, because the loop returns after it (SPEC-0028).
- **Tween with a zero total**: the timer clears on the first tick with no step, and the apply still runs (Medium).
- **Tween source for other modes**: the source is not set on that path, so the step is not defined by this spec. The clean side should not step for modes other than 0 to 3 (Low; Open question 108).

### Edge cases: bursts, tables, timers, placement and rotation

- A burst whose lifetime is zero or less finishes on its first update, and its pieces are not updated.
- The jump guard applies only when the new second component is more than 1e8 away from the tracked value. A difference of exactly 1e8 is accepted.
- The keyed claim refuses a duplicate key pair even when a free slot exists. The scan stops at the duplicate, so no slot is taken.
- Keyed slot busy test: five null child pointers make the slot not busy. A non-null child with its +0x90 word equal to 4 also leaves it not busy; changing that word to any other value makes the slot busy.
- Find-or-create with key 0 writes into the first empty slot.
- Find-or-create with no matching key and no empty slot in the group reports failure and changes nothing.
- A first-free search with all ten entries busy returns null.
- Detach: a null child link is ignored; a non-null child whose +0xC4 backlink does not match the parent is unchanged. With bit 0 clear, children are conditionally unlinked and the effect object is kept; with bit 0 set, the same five-link walk is followed by the effect object's release helper. Linked children are not released by this path.
- Effect-bank record reset: a child in state 2 with live-child count 0 enters state 4 and runs its callback/buffer-release path when those hooks are present; with count 1 it enters state 5 and has the vector at +0x50 cleared. The reset then reinitialises the containing sub-object.
- A sub-effect start while the sub-effect is active changes nothing, including its step and scale.
- A negative duration gives a negative active count. Whether a negative count counts as active is not established.
- The rotation axis is not normalised; a non-unit axis scales the vector part of the quaternion.
- The axis variant reads its first parameter in radians and its second in degrees. Reproduce this mix if the original behaviour is wanted.
- Placement with coincident points has no defined direction.
- The hit-flash key block is written only by its start-up initialiser, once at boot (section 32).
- Save-point glow: when shared effect group 3 is not available, the glow never loads, and the start, move-and-start and update steps do nothing; the save-point rules of SPEC-0025 still run (High).

## Test vectors

1. Clip window (synthetic): F = 20, layer-0 speed 0.25 frames per tick, start fraction 0.5, end fraction 1.0. Start window = [10.0, 10.25): the entry starts on the one tick its cursor is 10.0. End: 1.0 × 20 = 20 ≥ 20, so the end is 19 and the stop window is [19.0, 19.25). With start fraction 0 the start window is [0.25, 0.5), the second tick of the clip.
2. Cone: dot 0.4, d² = 400 (above 225), cone flag 1. Stop. With d² = 200, no stop.
3. State gate: an emitter with gate 1 is kept only while the instance state is 1. The listener moves from d² = 9000 to d² = 10100 (threshold 10000): the instance state changes from 0 to 1, the gate-0 spawners are drained (their particles live out their lives) and the gate-1 emitters start.
4. Period: base 0.2 s, jitter 0.05 s. Each new period is in [0.15, 0.25) s (symmetric jitter); a draw at the midpoint gives 0.2 s.
5. Curve accumulation: life 1.0 s, row 0 = (0, 0, 0, 0), no key time passed during the life, so the index stays at 0 and the rate is row 0. Set row 0 = (0.5, 0, 0, 0) instead and the channel 1 x component after the full life is 0.5.
6. Drag: velocity (1, 0, 0), drag per second 0.1, one tick: velocity x = (1 + acceleration) × (1 − 0.1/60).
7. Positional attenuation: entry intensity 1.0, r_in 10, r_out 50. At d = 5 (d² = 25) intensity = 1.0. At d = 30 (d² = 900) intensity = 1 − (900 − 100)/(100 + 2500) = 0.6923. At d = 60, intensity = 0.
8. Particle life fraction and curve (computed): life 0.5 s, row 0 channel 1 = (0.8, 0, 0, 0), RGB and alpha key times all 1.0 (not reached in the test), dt = 1/60 s. Each tick adds 0.8 × (1/60 ÷ 0.5) = 0.026666667 to the channel 1 x accumulator. With the console's truncating adds (SPEC-0008, Platform arithmetic): after 15 ticks elapsed = 0.24999999 s, life fraction = 0.49999997 and the x accumulator = 0.3999998 (nominally 0.25 s, 0.5 and 0.4). After 30 ticks elapsed = 0.49999976 s, which is not above the life, so the particle is still alive; life fraction = 0.99999952 and the x accumulator = 0.7999994 (nominally 1.0 and 0.8).
9. Curve clamp (computed): the same life, row 0 channel 1 = (3.0, 0, 0, 0). Each tick adds 3.0 × (1/60 ÷ 0.5) = 0.1. After 20 ticks the x accumulator is 1.9999989 (below 2.0, so no clamp yet). On tick 21 the sum is 2.0999987 and is clamped to 2.0, and it stays at 2.0 on ticks 21 to 30 (the clamp).
10. Attach slot decode (synthetic): raw slot A value 0x00000015 gives matrix slot 5 (low 3 bits). Raw 0x4FFFFFFF gives none, without interpretation its low bits as slot 7.
11. Impact burst (synthetic): draw r = 3 starts definition 136 with the burst direction; r = 6 starts definition 135; r = 0 starts definition 138; r = 20 starts nothing. Probabilities over the 101 draws: 138 in 1, 136 in 5, 135 in 14, nothing in 81.
12. Impact boundary (synthetic): r = 19 gives 135 and r = 20 gives no effect.
13. Sway, first update (synthetic, boot values): the phase goes from 4096 to 4146 (the step of 50 is set on this first update) and the amplitude from 1.0 to 1.03.
14. Sway bounds (synthetic): the phase after five updates would be 4346, which is above 4300, so it is set to 4300 and the step becomes -50; update six gives 4250. The amplitude after 17 updates would be 1.51, so it is set to 1.5 and the step becomes -0.03.
15. Frame tilt (synthetic, section 10): S = (1, 0, 0), U = (0, 1, 0), D = (0, 0, 1). Step 1 gives S = (-1, 0, 0), D = (0, 0, -1). With t1 = 0.2: D1 = (-0.2, 0, -0.8) (length 0.8246, kept). S = unit(D1 x U) = (0.9701, 0, -0.2425). With t2 = -0.5: D2 = (0, -0.5, 0) + 0.5 D1 = (-0.1, -0.5, -0.4), length 0.6481, so the shot moves at 64.8 percent of its speed. U = unit(D1 x D2) = (-0.9701, 0, 0.2425), the new S negated (t2 < 0). Arming counter 15.
15a. Status tint (synthetic, section 5.4): strength (1.8, 0.3, 0.2) gives the slot 0 target (1.0, 0.3, 0.2, 1.0) in mode 1; speed (2.0, 2.0, 0.3) gives (1.0, 1.0, 0.3, 1.0); fire (1.5, 1.5, 1.0) gives (1, 1, 1), so slot 0 is cleared and no tint shows. With mode 1 and the counter at 15 the blend weight is 0.5 (section 21).
16. Missing definition (synthetic): a start with id 162 finds no definition and creates an instance with no emitters, which never produces particles. A water footstep (id 162) sets up no effect object at all.
17. 8.8 write, positive (computed): value 0.7 × 256 = 179.2, plus 0.5 = 179.7, truncated to 179. Stored 179; reads back as 179/256 = 0.69921875.
18. 8.8 write, negative (computed): value -0.7 gives -179.2, minus 0.5 = -179.7, truncated to -179. Reads back as -0.69921875.
19. 8.8 write, half step (computed): value 1.5/256 = 0.005859375 gives 1.5 + 0.5 = 2.0, stored 2, reads back 0.0078125. The negative value -1.5/256 gives -2, reads back -0.0078125.
20. 8.8 write, zero and tiny (computed): value 0.001 gives 0.256 + 0.5 = 0.756, truncated to 0; value 0.0 gives 0.5, truncated to 0. Both read back as 0.
21. 8.8 wrap, high side (computed): value 200.0 gives 51,200 + 0.5, truncated to 51,200. As a signed 16-bit value this is 51,200 − 65,536 = −14,336, which reads back as -56.0. Value 128.0 gives 32,768 + 0.5, truncated to 32,768, which wraps to −32,768 and reads back as -128.0.
22. 8.8 wrap, low side (computed): value -128.01 gives -32,770.56 − 0.5 = -32,771.06, truncated to -32,771. As a signed 16-bit value this wraps to 32,765, which reads back as 127.98828125. The value 127.99 also stores 32,765 (32,765.44 + 0.5, truncated).
23. Uniform draw (synthetic): lo = 2, hi = 6. A draw u = 2^31 gives 2 + 4 × 0.5 = 4.0. A draw u = 2^32 − 1 gives just under 6 (about 5.99999999907).
24. Vibration tester, inside the inner radius (synthetic, boot values): squared distance s = 0.5 ≤ 1. Weight w = 1, strength 0.5: a rumble request with preset 0, duration 1.0 s and magnitude 0.5.
25. Vibration tester, middle band (synthetic, boot values): s = 100 (distance 10). w = 1 − (100 − 1)/(1 + 900) = 802/901 ≈ 0.8901, magnitude ≈ 0.4451.
26. Vibration tester, at the outer radius (synthetic, boot values): s = 900 (distance 30). w = 2/901 ≈ 0.0022, magnitude ≈ 0.0011. The request is still made (strength above zero).
27. Vibration tester, just outside (synthetic, boot values): s = 900.5. s > R², so w = 0 and no request is made. This is the drop described in Behavior 11.
28. Vibration tester, equal radii (synthetic): an outer radius of 1 with an inner radius of 1 gives R ≤ r. No request is made and an error is reported.
29. Pulse, phase A (computed): after 16 updates A = 0.96; after 17 updates A = 0.02 (the wrap at 1.0).
30. Pulse, phase B and intensity (computed with the stored constants, float32 accumulation of B; byte = 64 + 186 (1 + P) / 2, rounded half away from zero):
    - n = 1 (B = 0.2, x = 0.0318): byte 175 (the ideal sine gives 175.48).
    - n = 2 (B = 0.4): byte 193 (ideal 193.22).
    - n = 8 (B = 1.6, the peak): byte 250 (ideal 249.96).
    - n = 10 (B = 2.0): byte 242 (ideal 241.57).
    - n = 16 (B = 3.2, A = 0.96): byte 152 (ideal 151.57).
    - n = 17 (B = 3.4, A = 0.02): byte 133 (ideal 133.24).
    - n = 20 (B = 4.0): byte 87 (ideal 86.62).
    - n = 24 (B = 4.8, folded x = 0.736): byte 94. An ideal sine gives 64. With the exact Taylor coefficients instead of the stored constants, the same phase gives 107, so the stored constants are part of the test.
31. Pulse fold boundary (synthetic): at B = 3 pi / 2, m = 0.5, a fold boundary. The one-sided values are x = -0.25 and x = 0.75; the exact console value at the boundary is unresolved, so no intensity byte is asserted. At B = pi / 2, away from a boundary, x = 0.25, P is about 1.0 and the intensity rounds to 250.
32. Pulse, item offset (synthetic): an item with base 0.5 and phase A = 0.96 gets the float 1.46 at its phase offset; after the wrap (A = 0.02) it gets 0.52.
33. Loader links (synthetic, 1,651 nodes): stored link -1 becomes none. Stored link 1,650 becomes a reference to node 1,650. Stored link 1,651 is out of range and becomes none.
34. Loader child (synthetic): child byte 0xFF with child word 7 gives child chunk 7. Child byte 0xFF with word -1 gives none. Child byte 0x80 with word 49 gives child chunk 49 (the gate is 0x80 or more).
35. Loader definition (synthetic): definition with n = 3 and stored emitter indices (4, 9, 1,700, 2, 3). Result: node 4, node 9, none (1,700 is out of range), and slots 4 and 5 are none (beyond n).
36. Spawner gate, multi-emitter flag (synthetic): definition with n = 2, bound nodes in all five slots, all gates 0, state 0. Flag 1 starts slots 0 and 1 (limited by n = 2). Flag 0 starts slot 0 only (the walk stops after the first emitter that starts).
37. Spawner gate, pool word (synthetic): a node present with spawner pool word 0 (group not bound) gets no spawner. The same node after its group is bound (non-zero pool word) and with a free pool slot gets a spawner.
38. Spawner handle (synthetic): one slot with base period -1, a free pool slot and a bound node. The spawner is started, the handle is discarded, and the instance state stays 0 (idle). With a second slot of base period 0.5 that keeps its handle, the instance state becomes 1.
39. Expiry without a child (synthetic): a particle with life 1.0 s and elapsed 1.2 s on a node with link none and A +0x86 = 0 ends, and no child is started.
40. Swing-trail record mapping (synthetic): kind 3 selects record 6 of the 0x50-byte table (byte offset 480, the yellow gradient); kind 4 selects record 3 (offset 240, blue); kind 9 selects record 0 (offset 0). An enemy whose row type T is 3 with no attack slot selected gives kind 3.
41. Swing-trail gate (synthetic): a list entry with byte +0x48 = 0, slot B set and a free holder starts a trail whose material is the material group of common object 297 (group 11); with +0x48 = 255 no trail starts and the entry's effect id path runs as usual.
42. Type setup (synthetic cap): type word 5 gives index 4; with a cap of 5 in preset record 4, a total count of 30 gives 5 particles per burst and a cycle limit of 6. A total of 3 gives 3 per burst and a cycle limit of 1. Type word 28 leaves the node unchanged.
43. Timed emitter (synthetic): enable set, count 2. Tick 1 spawns and the count is 1. Tick 2 spawns and the count is 0. Tick 3 clears the enable word and spawns nothing.
44. Timed emitter, lookup miss (synthetic): count 1 and no entry for the id. The tick spawns nothing and the count becomes 0.
45. Timed entry list (synthetic): three entries with countdown words 1, 5 and 2, and every test passes. After one tick the list has two entries: the first is the old second entry (countdown 5, not decremented this tick) and the second is the old third (countdown 1). The count is 2.
46. Buffer size (synthetic): N = 10 gives 10 x 96 + 5 x 16 + 4096 = 5136 bytes. N = 1 gives 96 + 0 + 4096 = 4192 bytes.
47. Id registry (synthetic): id 256 into an empty slot is stored and the call returns 0. Id 300 returns -1 with no log. Id 128 into an occupied slot returns -1. Id 384 when the check fails returns 0 and stores nothing.
48. Segment join (synthetic): A has start 0, slope 1, duration 2 and links to B. B has start 2, slope 1, duration 3. The join returns 1; B becomes start 0, slope 1, duration 5, flag 0.
49. Clip-to-list lookup (synthetic): a table with count 3; the second entry's first word holds 42 shifted left by 14 bits. Index 1 returns 42. Index 3 returns 0xFFFF. An entry whose field is 0xFF returns 0xFFFF.
50. Quad lifetime (synthetic, 60 ticks per second): created at tick 0, the counter is 300. It is still alive after 299 refreshes (counter 1). The 300th refresh lowers it to 0, and the quad is released on that refresh.
51. UV scroll (synthetic; K = 1 is a test assumption, not the disc value): one vertex with base (u, v) = (0.25, 0.5), t_u = 2000 and t_v = 1000. The offsets are wrap(2000 / 4000) = 0.5 and wrap(1000 / 4000) = 0.25. The result is (0.75, 0.75).
52. Sprite batch values (computed from the series of section 12 and the fold of 16.1): for a = 0.5, Q2 is about 0.877583 (cosine of 0.5 is 0.877583) and Q1 is about 0.479426 (sine of 0.5 is 0.479426). For a = 0, Q1 = 0 and Q2 = 1.0.
53. Trail countdown (synthetic): the countdown is set to 60 at tick 0 and reduced by 1 on each of ticks 1 to 60, so it is 0 after tick 60. On tick 61 it is set to -1 and the trail is ended.
54. Packet split (synthetic, large budget): 30 samples in one call. The first packet holds 24 samples and its quadword count is 1 + 3 x 24 = 73. The second holds 6 samples with a count of 1 + 18 = 19.
57. Packet reserve (synthetic): with 192 bytes remaining before a sample, 0 + 192 is not strictly below the budget, so the builder stops; with 193 bytes remaining it may write the first sample.
55. Ramp (computed in 32-bit floats): starting from 0, the ramp is 0.005 after one tick and 0.07499999 after 15 ticks, which is below 0.075, so tick 16 takes it to 0.08 and it stops there.
56. Static record decoding (synthetic bytes; section 18): a 0x50-byte record whose bytes at +0x00 are 00 00 80 3F 00 00 00 40 00 00 40 40 00 00 80 40, at +0x30 are 05 00 00 00 and at +0x38 are 00 00 00 3F decodes as vector A = (1.0, 2.0, 3.0, 4.0), count 5 and the +0x38 scalar 0.5. Kind 3 selects record 6, which starts at 0x370DF0 + 6 x 0x50 = 0x370FD0; kind 7 selects record 7 at 0x371020.

57. Triangle fraction (synthetic): counter values t = 15, 30, 31, 45, 59 and 60 give f = 0.5, 1.0, 29/30, 0.5, 1/30 and 0. (t = 60 is t = 0 after the modulo.)
58. Triangle blend (synthetic): a slot object with x = 10 and a reference x = 40. At t = 15 the stored x becomes 25. From x = 10 at t = 30 it becomes 40. From x = 10 at t = 45 it becomes 25.
59. Triangle fraction store (synthetic): x = 0 with reference x = 1 at t = 1 (f = 1/30). The stored value is 0.0333 (a float), not 0.
60. Twelve-sprite grid (synthetic, unreached code; section 22): with axis (1, 0, 0) and centre (0, 0, 0), sprite (r = 1, c = 2) is at 16.0 x (1, 0, 0) + 8.0 x normalise(cross((0, 1, 0), (1, 0, 0))) = (16, 0, -8).
61. Not used (section 22 is unreached code).
62. World record step (synthetic; section 23): a list with items 0 to 4 places item 0 only; items 1 to 4 keep their earlier state.
63. Tween, mode 0 (synthetic; section 25): duration 0.5, total 5, source (10, 0, 0), position starting at 0. Each step adds 10 × 0.5 / 5 = 1.0. After ticks 1 to 5 the x component of the position is 1, 2, 3, 4 and 5. Tick 6 clears the timer and x stays 5.
64. Tween, mode 3 (synthetic): duration 1.0, total 2, source (0, 4, 0), position starting at 0. Each step adds (0, −2, 0). After two steps y is −4, and the next tick clears the timer.
65. Tween, zero total (synthetic): a total of 0 clears the timer on the first tick with the position unchanged.
66. Effect id 101 (definition 211; disc values): four emitter nodes, threshold 1225 (35 units), cull 90000 (300 units), cone flag 1. Node 521: A period 0.25 s, jitter 0, burst count 1, cycle limit 0, state 0, repeat 0, launch rule 0; C kick interval base -1.0 (never kicked); material 71 (additive). Node 523: A state 1 (far), period 0.5 s, burst count 1, repeat 0, material 71. Expected: node 523 is started only while the effect object's state is 1 (3.16), and node 521 only while it is 0.
67. Effect id 1201 (definition 161; disc values): three emitter nodes sharing A record 193 and C record 110. A: period 0.1 s, jitter 0, burst count 10, loop flag 0, life A +0x78 = 4.0 s with life jitter A +0x7C = 1.0 s. C: launch speed base 2.0 (raw 0x0200) with spread 4.0 (raw 0x0400); polar angle range from 0 up to raw 0x0324 (804 / 256, about 3.14); vertical acceleration 1.5 units/s^2 (raw 0x0180, 0.025 per tick); kick interval 0.5 s (raw 0x0080), spread 0. Material 22 (additive, texture index 0). Expected: each burst launches 10 particles, each with a life from 3.0 to 5.0 s and a kick every 0.5 s (test vector 70).
68. Effect id 159 (definition 542; disc values): one emitter node (node index 1371) with A period -1.0, burst count 30, launch rule 0, and the field at A +0x68 equal to 5 (role not known). C launch speed base 3.0; kick interval raw 0x0005 (0.0195 s, about one tick). Life A +0x78 = 6.0 s, jitter 0.02 s. Material 27 (alpha, texture index 1). Expected: one burst of 30 on the first running tick, then the spawner drains and dies after its particles expire (test vector 71).
69. Material test vectors (disc values; texture base and palette base are the fields of section 2.8): material 71 uses texture index 1, PSMT8, 64 x 64, texture base 496, palette base 884, additive. Material 24 uses texture index 0, PSMT8, 128 x 128, texture base 0, palette base 540, alpha blend. Material 22 uses texture index 0, PSMT8, 64 x 64, texture base 320, palette base 568, additive. Material 27 uses texture index 1, PSMT8, 64 x 64, texture base 192, palette base 880, alpha blend.
70. Per-tick, effect id 1201, node 414 (A record 193, B record 343, C record 110; disc values; every uniform draw at the midpoint of its range; emitter at the origin with an identity frame; values after the tick's motion and curves): 10 births per burst; bursts on ticks 1, 8, 14, 20, ...; life 4.0 s; kick interval 0.5 s; acceleration per tick (0, 0.025, 0); drag per tick 1/30. Particle 0 after tick 1: P = (-0.9678, 0.0004, 0), v = (1.9333, 0.0251, 0), colour channel (1, 1, 1, 0.0359), colour bytes (127, 127, 127, 4), size 0.4990, half-extent 0.2495. Tick 30: P = (-0.3829, 0.1391, 0), v = (0.7233, 0.4631, 0). Tick 31: v = (0.6992, 0.4719, 0). Tick 32 (kick, +R = 0.19921875 on y only): v = (0.6759, 0.6795, 0). The alpha rate stops on tick 28, when f passes 0.116; alpha is then 0.9695 (27 ticks of 8.6176 / 240). Tick 240: size 0.25, half-extent 0.125, P = (-0.0336, 3.1891, 0). The particle ends on tick 241. The population settles at 390 to 400. With all draws at the low end the life is 3.0 s; at the high end 5.0 s.
71. Per-tick, effect id 159, node 1371 (same conditions): tick 1 bursts 30 particles and the spawner is already draining; life 6.0 s; acceleration per tick (0, 0.00833, 0); drag per tick 0.05; spin 1 rad/s; colour bytes (95, 95, 95, 24) after tick 1. The particles end on tick 361; the spawner is dead on tick 362.
72. Per-tick, effect id 400, node 446 (loop flag set; same conditions): one particle. On tick 61 its elapsed returns to 0 and channel 1 restarts to the start colour (63, 127, 127, 18 as bytes). The same tick then applies one colour growth step with the pre-restart fraction (4.6 step 5), which gives the bytes 64, 129, 129, 18 (Medium: the single step is consistent with the start colour, but the key-row rates of node 446 were not recomputed here). No new birth while it lives. Half-extents (5.175, 5.175, 2.1375) = 0.5625 x (9.2, 9.2, 3.8).
73. Preset source locations (3.9; file offset = VA - 0x100000 + 0x80): for index 4 (type word 5), size start row 4 is at VA 0x2DB520 (file offset 1,947,040 decimal); size rate rows 1, 2 and 3 of index 4 are at VA 0x2DB6D0, 0x2DB880 and 0x2DBA30 (file offsets 1,947,472, 1,947,904 and 1,948,336); C source 4 is at VA 0x2DBCE0 (file offset 1,949,024); B source 4 is at VA 0x2DC4E0 (file offset 1,951,072); preset record 4 is at the run-time address 0x3713B0, and its cap at 0x3713B8.
74. Size row decoding (synthetic bytes): the 16 bytes 00 00 80 3F 00 00 00 40 00 00 40 40 00 00 00 00 at size start row i decode as (1.0, 2.0, 3.0, 0.0). After start-up step 3 this is preset i +0x60 and first-time A record i +0x00.
75. C conversion (synthetic source record; other words 0): direction words (0.0, 3.0, 4.0) (bytes 00 00 00 00, 00 00 40 40, 00 00 80 40), +0x0C = 0.5, +0x10 = 1.0, +0x20 = -9.8, +0x3C = 3, +0x48 = -1.0. Result: C +0x00 = (0, 0.6, 0.8, 1.0); C +0x28 raw 1280 (5.0); C +0x2A raw 128 (0.5); C +0x24 raw 256 (1.0); C +0x20 raw -2509 (-9.80078); C +0x38 raw -256 (-1.0, never kick); C +0x3E = 3; C +0x3F = 0. With +0x3C = 1 instead, C +0x3E = 0.
76. B copy (synthetic source record): t1 = 0.25; alpha (1.0, 0.5, 0.0); red (0.9, 0.8, 0.7); green (0.6, 0.5, 0.4); blue (0.3, 0.2, 0.1). Result: row 0 = (0.9, 0.6, 0.3, 1.0); row 1 = (0.8, 0.5, 0.2, 0.5); rows 2 to 5 = (0.7, 0.4, 0.1, 0.0); RGB key times (0, 0.25, 1.0, -1.0, -1.0, -1.0); a first-time record keeps alpha key times of -1.0.
77. A copy (synthetic preset): preset +0x00 = -1.0, +0x04 = 0.0, +0x08 = 9, +0x0C = 7, +0x10 = 2.5, +0x60 = (1.0, 2.0, 3.0, 0.0). Into a first-time A record: A +0x6C = -1.0 (single burst), A +0x70 = 0.0, A +0x64 = 9, A +0x68 = 7, A +0x78 = 2.5, A +0x00 = (1.0, 2.0, 3.0, 0.0); A +0x60 stays -1 and A +0x7C stays 0.
78. Swing-trail colour (computed; record 0, one step per update, F = 1): a sample's alpha is 48 at age 0, 44 at age 1, 40 at age 2, 21 at age 8 and 2 at age 14; at age 15 it is retired with colour C. Record 7 (kind 7, c = 8): R goes from 128 down by 16 per update to 0 at age 8, while G rises by 16 per update to 128.
79. Swing-trail ageing gate (synthetic): game-state code 8: sample ages and colours stay as they were, and the trail is still drawn. Code 9 (pause): samples advance and the trail is still drawn.
80. Mote-eater drain (computed): 8 slots, N = 1801, P = 225: reveals on calls 1, 226, ..., 1576, stop on call 1801; with two sisters alive the last reveal is on tick 788 and the stop on tick 901. 7 slots: P = 257, first reveal on call 2.
81. One-shot effect at a point (synthetic): flag 0 starts definition 222 at the point and detaches it (mode 2); flag 1 starts definition 220; with group 3 absent nothing happens.
82. Placed-effect zone gate (synthetic): an active record (+0x48 = 1) with +0x44 = 3 in the current world, while the camera stands in zone R and cell (R, 3) of block 0 is 0: the record is stopped that pass. With cell (R, 3) = 1 it is updated with the camera as the listener. A record with +0x48 = 0 is neither stopped nor updated by the pass.
83. Slot matrix (synthetic): X = (0, 0, -1, 0), Y = (0, 1, 0, 0), Z = (1, 0, 0, 0), T = (10, 0, 5, 1) and the vector (1, 2, 3) with w = 1 give the point (13, 2, 4).
84. Context lookup (disc values): Black Knight spark 305, first node, byte 9: child = common object 292. Save-point glow 404, first node, byte 12: common object 295. A byte of 0x28: no child.
85. Item effect (disc values): item 135 binds 761; item 160 binds 951; item 173 binds 964; item 139 binds nothing. The bound object's motion scale is 1.5.
86. Placed-record cull (synthetic): record cull 50.0 is kept; -1.0 is kept; -2.0 and 0.0 take the definition's value.
87. Sprite quad (synthetic, scale 1): centre (100, 50), lanes hx = 3, hy = 1, hz = 2. Angle 0: corners (98, 47), (98, 51), (102, 47), (102, 51) with texture coordinates (0, 0), (0, 1), (1, 0), (1, 1). Angle pi/2: (97, 52), (101, 52), (97, 48), (101, 48). A centre with w of 0 or less gives no quad (16.4).
88. Half-extent lanes (disc values): a node with factors 144, 289 and 0 at +0x6A, +0x6C and +0x6E and size start (0.5, 0.5, 0.5) has lanes (hx, hy, hz) = (0.5645, 0, 0.28125) x S: width 0.5625 x S, height 0.5645 x S above the centre only. Held-weapon trail points (section 3.8): hero Tor with item 98, base (10, 0, 5), direction (0, 0, 1): (10, 0, 5.25) and (10, 0, 6.27); no weapon: (10, 0, 5.25) and (10, 0, 6.15); enemy owner: (10, 0, 5.25) and (10, 0, 6.75).

### Test vectors: bursts, tables, timers, placement and rotation (synthetic)

- Burst lifetimes: s = 0.5 gives L = 30; s = 2.0 gives L = 120; s = 0.01 gives L = 0; s = 0.1 gives L = 6.
- Burst guard: tracked value 0.0. A new second component of 1.0e8 is accepted. A new second component of -1.5e8 is refused, and the tracked value stays 0.0.
- Keyed claim on an empty bank: (A, 1) takes slot 0; (B, 2) takes slot 1; (A, 1) returns null; (A, 3) takes slot 2.
- Find-or-create in a group whose third slot holds key 5 with value 40 and whose other slots are empty: (5, 60) finds the third slot, sets its value to 60 and reports success; (9, 1) takes the first empty slot; (0, 7) takes the next empty slot. In a full group of eight nonzero keys without 9, (9, 1) reports failure.
- First-free over ten entries with the first four busy: returns the fifth entry, 4 x 0x3D0 bytes from the first. All busy: null.
- Detach: argument 2 detaches five links and keeps the object. Argument 3 detaches five links and releases the object.
- Hit-flash keys: the start-up initialiser called with (1, 0xFFFF) writes the three key vectors, the three key colours and the two stage lengths (4 and 10 frames) of section 32; called with other arguments it writes nothing.
- Sub-effect: duration 10.0 gives 600 ticks; duration 0.5 gives 30 ticks; step 1.8 gives 0.03 per tick; scale 1.5. Second timer with T = 12.8 gives duration 1.0 s (60 ticks); T = 6.4 gives 30 ticks.
- Placement: P = (0, 0, 0), G = (0, 0, 10) gives P = (0, 0, 5). P = (1, 2, 3), G = (1, 2, -7) gives P = (1, 2, -2).
- Rotation: angle 90 about (0, 1, 0) gives (0, 0.70711, 0, 0.70711); angle 180 about (0, 0, 1) gives about (0, 0, 1, 0); angle 0 gives (0, 0, 0, 1).
- Axis variant: first parameter 0 gives axis (1, 0, 0); second parameter 90 gives (0.70711, 0, 0, 0.70711).
- Frame object (computed): direction (1, 0, 0) gives row 0 = (0, 1, 0), row 1 = (1, 0, 0), row 2 = (0, 0, -1). Direction (1, 1, 0) gives row 1 = (0.70711, 0, 0) (not renormalised), row 2 = (0, 0, -1) and row 0 = (0, 1, 0).
- UV scroll wrap (synthetic; K = 1 is a test assumption): base (0.25, 0.5), t_u = -2000 and t_v = -5000 give offsets -0.5 and -0.25, so the result is (-0.25, 0.25).
- Reed-cut coin roll: r = 0 spawns item 138; r = 5 spawns item 136; r = 19 spawns item 135; r = 20 spawns nothing.
- Map icon pulse: phase 4250 with step +50 becomes 4300 (no turn); the next update gives 4350, which is set to 4300 with step -50. Amplitude 1.49 with step +0.03 becomes 1.52, which is set to 1.5 with step -0.03.
- Stolen-item display: count 180 at the start gives draws on ticks 1 to 180; on tick 181 the enable word is cleared and nothing is drawn.
- Hit flash: on every frame of the 14-frame ramp, every part of the character is drawn with the colour scale (1.5, 0.5, 0.5, 1.0) in place of its slot colour; on the first frame after the ramp ends it uses its slot colours again.
- Foliage sparkle 139 (computed, life exactly 2.0 s = 120 ticks): alpha is 0 through tick 40; it rises by 19.5333 / 120 per tick on ticks 41 to 46 to 0.97667 (alpha byte 124); it holds until tick 112; from tick 113 it falls by 15.4210 / 120 per tick and is clamped to 0 on tick 120. The colour bytes stay (127, 127, 81). The size is about 1.0 after 35 ticks and about 0.5 after 120.
- Save-point glow (synthetic positions): save points at (0, 0, 0) and (100, 0, 0), hero at (10, 0, 0). The glow pair (definitions 400 and 404) is started at (0, 0, 0). The hero moves to (60, 0, 0): the nearest save point changes to index 1, so both emitters are stopped, moved to (100, 0, 0) and started again. While the hero stays nearer to index 1, the glow is only updated each tick, not restarted.
- Pulse output record (synthetic): drawn object preset colour words (100, 80, 60, 0) and an update that gives intensity 94. Each item colour record then holds the four words 100, 80, 60 and 94: red, green and blue are unchanged and the alpha is 94 (about 0.73 on the 128 = 1.0 scale).
- Light arc chain (synthetic positions): A = (0, 0, 0) with direction (1, 0, 0) and scale 20 / 60. After 3 ticks A is (1, 0, 0) and after 60 ticks (20, 0, 0), both within float rounding.
- Light arc trigger: on the first tick a ball touches the hero, the hero's take-hit method receives damage 20, attacker 0 and mode 3, and the fired flag is set. On later ticks with the ball still touching, nothing more happens.
- Display buffer word: a model draw for buffer 1 sets the word at 0x2EC830 to 1, so oscillator slot 2 is read at +0x1344 + 16 = +0x1354; a draw for buffer 0 sets it to 0 and slot 2 is read at +0x1350.

## Open questions

- Resolved (R-0043, parts 1 to 4): (1) test vector 72: the restart tick applies one colour growth step after the restart, so the start colour gives 63, 127, 127, 18 and the step gives 64, 129, 129, 18 (Medium; see vector 72). (2) The perpendiculars and sign draws are in section 4.1 step 5 (High for the order and signs, Medium for the helper choice). (3) The birth-angle cosine is the folded series at the angle itself, and the sine is the same series at the angle minus pi/2, not plus pi/2 (section 4.1 step 5; High). (4) The footstep dust frame: the footstep passes the hero joint's world point (joint 28 or 31) as its position, the frame is set to the identity with zero offset, and the object is the dust definition's object already in the hero's group 3, found by its definition id, not a new one (Medium).
- Resolved (R-0043, part 5): the sprite quad's corner order, texture coordinates and half-extent lanes are in section 16.4. Half-extent lane z is the half-width (screen right), lane x the half-height above the centre (toward T = 0) and lane y the half-height below it; the node factor at +0x6A feeds lane z, +0x6C lane x and +0x6E lane y (4.1 step 7). The picture is upright and a positive spin angle turns it counter-clockwise (High for the corner and texture-coordinate order, Medium for the screen directions). The child model of a model particle is drawn with the particle's frame (four rows, the last being its position) and the channel 1 colour (section 4.7; open question 16, Resolved; Medium); the velocity-alignment variant of its orientation is the part of open question 5 that stays open.
- Resolved (R-0044, parts 1 to 4): (1) the owner's matrix slots are the nine entries of its attach-point bank, entry k being pose-table entry k (the per-frame render update writes entries up to the table's count), so slot k of a list entry is bank entry k (the low bits of the entry's slot A word). Slot 7 of a character with fewer pose entries is never written by that update and keeps the value from the render-object reset, which is the identity (Medium for the reset value). (2) The animation-effect pool's per-pass update does not call the effect object's own update. For each active element it sets the effect object's position from point 1 (or, with the frame flag +0x49 set, its frame from slot A's whole matrix), and then appends the element's trail sample; the effect object's update runs from its owner's pass (3.16), and no listener is passed (High for the calls; Medium for the owner pass). (3) On the positional path the entry's +0x30 word is a sound id only, as section 5.3 states; the effect id that SPEC-0015 lists for mode 0 is the entry's separate effect field, not this word (Medium). (4) Hero clip lists are never walked (Resolved, Edge cases), so the hero tables' bits 14 to 21 start no effect or trail (High for the non-walk; the table locations are the clip records of SPEC-0005).
- Resolved (R-0045, parts 1 to 3): (1) the main attach point is attach point 7; fire adds attach points 0 and 1 (section 5.4). (2) Mode 0 holds the colour-override weight at 1, so the stun flash colour (0, 0.5, 1.7) is a steady tint, not a pulse; its end is the display's clear rule (section 5.4; Medium for the end). (3) Re-applying a status runs the display start whenever the apply's duration is at least one tick, but the display changes only when the code differs from the shown one; a re-apply of the shown code leaves the display and its rotation timer as they are (Medium-High; SPEC-0036 Status display and cues).
- Resolved (R-0045, part 4): the readers and their triggers are in section 3.17. The fourteen-entry tables serve the equipped Mote's weapon effects (the weapon line and light for Tor, the aura and charge-up effect for Roberta), the fully-charged burst and the charged-hold loop; the eight-entry tables and two of the fourteen-entry tables (0x315440 and 0x315478) serve Roberta's shot visuals, read when a shot is launched. None serves a swing ribbon or a hit effect. The table at 0x315408 is never read (its only reader has no caller). Tor's swing glows (effects 244, 243 and 246 at 0.25, 0.2 and 0.25 s into sub-states 2, 3 and 4) come from immediate ids. High for the readers and triggers; Medium for the on-screen role names.
- Resolved (R-0047, part 1): swing-trail samples age in normal play (game-state code 8 or below) and freeze at code 9 or above (section 3.8, Ageing gate); the draw step only rebuilds texture coordinates and fade. A trail ends in the first pass in which none of its samples is live, and the release does not end it on its own (section 3.8, Release). Medium for the pass count. (2) Part 2 is settled in section 3.8: the base point is attach point 0, and the length L is 0.6 times the hero's weapon reach (SPEC-0021) or 1.5 for an enemy owner, measured from a point 0.25 of the weapon direction vector beyond the base point (High for the hero, Medium-High for enemies). (3) Open (R-0047, part 3; SPEC-0051 Open question 12): front-end database record 210 supplies the map page's art pointers when the map opens. The icon art (primary, secondary and treasure icons) is a set of sub-sprites of front-end record 26, and the map's marker draw uses front-end record 214 (Medium). The sub-sprite indices inside those records, the hero locator sprite and the hint art are not settled, so no icon record ids are given; the sizes are in SPEC-0051.

- Resolved: the 167 light-emitter definitions carry no material record, and the type-39 spawns with effects 1300, 2202 to 2207, 4012, 4023, 4030 and 4031 reach the material only through their pack's level-script module, whose word at +0x54 is zero in every pack on disc. No light is listed in normal play, unless a run-time write of that word exists (Medium; see section 4.5). Test vector: a particle whose spawner's material word is set but whose material word at +0x54 is zero writes no light (High).
- Resolved (High for the code path): after a status ends, the hero's colour records keep their last blended values. The per-frame colour upload reads all 15 records with no slot test, and the blend that changes them runs only while the slot is on; the clear writes none of the records, and no push restores them until a bind, variant select or override caller runs (SPEC-0040 Behavior 26, item 6). Whether the original shows the persisting tint is a play check, listed in the user checks.

1. Window length L. Resolved: L is the layer-0 speed in frames per tick, t the layer-0 cursor and D the clip's frame count, so the windows are in frames and one tick wide, and the end clamp is D − 1 frames (1.2; High). Resolved (the acceleration-unit question that 4.1 used to file under this number): C +0x20 is a vertical acceleration in units/s^2 along world up; per tick the velocity gains C +0x20 x dt along (0, 1, 0) (4.1 step 6).
2. Resolved: the basis block holds constant vectors written once at boot: 0x3932D0 = (1, 0, 0, 0), 0x3932E0 = (0, 1, 0, 0) (world up), 0x3932F0 = (0, 0, 1, 0) (Where it lives). Codes 1, 2 and 3 of C +0x3F select them, for the spin axis of model particles only (2.6).
3. Resolved: the motion scale S of a spawner is copied from the effect's definition +0x80 (1.0 in all 666 definitions) at spawner init (3.2); the A record's w component is not the scale.
4. Resolved (launch modes 2 and 3): birth puts the particle at S + u x U(0, L) on the segment from the spawner's position S to its target point T, and drops the ring offset (4.1 step 2). Init copies the starting position to T; a position update moves S but leaves T unchanged; mode 3 resets T to S after a burst (3.2). T is also written directly, every pass, by the hero attack-effect driver for the weapon-line and charged-hold effects (3.17); the standalone target setter has no caller. Which effects use each mode is in the table of section 2 (birth position mode). **[High]** Still open, and not about the segment: the inherited-velocity setter copies an owner-supplied vector to each live spawner (4.1, 4.2); its three direct callers are a per-tick animated-object update, a helper that creates or refreshes an effect from arguments and a routine that applies one vector across a five-slot child-effect list, and a common gameplay meaning of their vectors is not established. Default if unresolved (a rewrite choice): inherited velocity zero. **[Medium for the callers]**
5. Resolved in part: C +0x3C is not a world/local position flag; it selects the model-particle orientation branch. Zero rotates the existing model frame about the axis selected by C +0x3F using the particle spin rate; nonzero uses the particle velocity to update the model frame. Sprite particles do not read this flag. The exact velocity-alignment interpolation and full rotation math remain open.
6. Resolved: the value the curves run over and expiry tests against is the particle life, A +0x78 + U(-A +0x7C, +A +0x7C) seconds, drawn at birth (4.1 step 3). The colour that reaches the screen is channel 1 clamped to [0, 2] x 127 in the sprite's RGBA bytes (GS scale, 128 = 1.0). The x 255 path is the light publish of light-emitter emitters (4.5), not a sprite colour.
7. Resolved: the size rate row per axis is A +0x10 while f <= A +0x40, A +0x20 while A +0x40 < f <= A +0x50, and A +0x30 after (4.6 step 6).
8. Resolved: a negative base period gives one burst on the first running tick; with A +0x86 = 0 (all 330 such records on disc) the spawner then drains and dies when its particles expire (3.2 step 1).
9. Resolved: the outer-radius comparison is strict. At d² = r_out², interpolation returns 2r_in² / (r_in² + r_out²) times base intensity; only d² > r_out² sets intensity to zero. For the common radii 10 and 50, the boundary is 1/13 of base intensity and proceeds to the request path when positive (section 5.3; High).
10. Partly resolved: the envelope presets at 0x2DD270 are the rumble presets of SPEC-0033 Rumble. Positional mode 0 also starts a camera shake and rumble preset 0 when its attenuated intensity is positive; the separate sound id at +0x30 resolves through the loaded sound list and its sound instance is configured at the computed effect point. The +0x34 word supplies the rumble duration and is not passed into audio setup on this path (5.3; High). Still open: any other role of +0x34 and the behavior of modes other than 0, which have no list-entry example on disc.
11. Resolved: the generator is seeded on its first draw from the console clock, so sequences differ per boot (3.3; SPEC-0008 owns the details).
12. Resolved: each slot matrix is four vectors in memory order, the x, y and z axes then the translation, and a point maps to X x + Y y + Z z + T w (5.1; test vector 83).
13. Answered: section 6 (the stored direction is used as is).
14. Resolved (High): placed records are bound (copied) at world load and start nothing. StartEffect runs one update and does not set +0x48; StopEffect and switch off clear +0x48 and drain; switch on sets +0x48 and starts nothing; each pass, a flagged record is updated while its zone (+0x44) is visible and stopped while it is hidden (section 6, Zone gate). A looping effect started only by StartEffect remains untouched by the zone pass while its world stays loaded, even after the player leaves its visible zone. World-buffer release drains and releases every non-null placed-record emitter instance and clears the slot; a retained second world keeps the effect allocated until that buffer is actually released. One-shot emitters may finish naturally sooner.
15. Resolved: the u16 values at node +0x6A, +0x6C and +0x6E are s8.8 sprite half-extent factors (128 = 0.5), not colour: half-extent lane x = factor at +0x6C, lane y = factor at +0x6E, lane z = factor at +0x6A, each times the size lane and S (2.3, 4.1 step 7, 16.4).
16. Resolved: no node holds both a material and a child; a child chunk is an ordinary type-12 model chunk drawn with the particle frame (2.1, 4.7).
17. The material index (+0x5C) selects one of the 123 groups of the resource-9 shader list. The group gives the texture (section 2.8) and the blend (section 2.8). Partly settled: the resolved material is part of the sprite batch key (4.7). The runtime base is Open question 128.
18. Resolved: A +0x10 to +0x30 are the three size rate rows; A +0x78 and +0x7C are the particle life and its jitter; B +0x78 to +0x8F (which includes B +0x88) are the alpha key times (2.4, 2.5).
19. Resolved: C +0x38 is the kick interval, not a lifetime. The life is A +0x78 + U(-A +0x7C, +A +0x7C), drawn once at birth; a loop restart keeps the same life. Expiry and the curves use that life.
20. Resolved in part: particles with a material are drawn as sprites in up to 20 batches keyed by shader context, material and a renderer context selector (0 or 1), at most 2000 per frame; light-emitter particles are not drawn. The selector is stored in the upper 24 bits of the sprite record's 64-bit word at +0x08 and participates in batch equality (4.7). Section 16 establishes the packet shape: three 16-byte input vectors become three output quadwords, with vector 0 copied unchanged, vector 1's and vector 2's w components replaced by the angle-derived Q1/Q2 values, and their other components copied. The roles of the three vectors (colour, centre, half-extents) and the quad built from them are in 16.4 (resolved). Still open: the broader purpose of the two renderer contexts.
21. Resolved: Owner frame presets (5.4). They are the status display's per-tick tint routines, not frames: the three vectors are the red, green and blue channel records, the constants are the status colours, and the slot 0 target is each channel cut to 1.0 (section 5.4). **[High]**
22. Resolved: the node word at +0x40 that the gate reads is the node's spawner pool, written when its group is bound (3.4, 3.16); +0x44 is the particle pool and +0x48 the shader-list context.
23. Answered: 2.1 step 3 and 4.6 step 5 (every disc link becomes none).
24. Resolved: the per-particle phase clock is the sprite spin angle, advanced by the spin rate (C +0x30) x dt (4.2 step 3). It is separate from the elapsed time and the life fraction, and it is the per-sample angle of the sprite batch packets (section 16).
25. Resolved for the list-entry start and refresh paths: +0x3D selects the owner character's ground height, falling back to an 8.0-unit probe when none is available; +0x3C controls the probe only when +0x3D is clear. No separate start/stop condition depending on +0x3D was found in the checked callers. Other unrecovered code paths remain outside this result (1.1; High for branch use, Medium-High for ground meaning).
26. Hooks: partly settled. A spawner has a birth hook (called for each particle born) and an end hook (called when it finishes; for effect objects it clears the object's slot, 3.16). Still open: the callbacks used by the single-emitter start (3.5).
27. Resolved: only channel 1 (colour) is clamped to [0, 2]. Channel 2 (size) is not clamped; the sprite half-extents derived from it are kept at 0 or more (4.4, 4.6 step 6).
28. **Effect request slot (3.7).** Resolved for the checked character request path: -1 is the no-id release request; when a different id is requested, the prior instance is released before the new definition is bound; the request slot is at +0x570 of an effect-request object returned by a shared singleton accessor, while the character is passed separately. Still open: whether other request paths use the same slot/object, and how it relates to the separate effect-object pool. The swing trail (3.8) uses its own per-holder slot in the animation-effect pool element. **[Medium-High for the checked caller and slot transitions; open for other paths]**
29. Resolved: the effect object's set-position and set-frame calls copy the value to every live spawner (5.1, 3.16). The infinite-lifetime reading that 4.6 used to file under this number is also resolved: every particle has a finite life, and a negative C +0x38 means a particle is never kicked (Open question 19). The frame built from a direction by MoveEffect is Medium.
30. Resolved: no unbounded life is stored. A negative C +0x38 is a kick interval of never; the life comes from A +0x78 and is always finite.
31. Likely-effect helpers and the two effect-core helpers named in the spawner path are not described beyond the behaviour in sections 3 and 5. Their details are not specified here.
32. Resolved: the point helper of section 8 is the reed-cut coin roll. Its caller is the reed cut routine (SPEC-0060); it spawns items 135, 136 or 138 through the item drop spawner, not effects, and the direction is horizontal from two points of the hero's render object.
33. Resolved: no kind-based effect helper starts one or four sprites with random offsets. The one-or-four-with-offsets pattern is the cut-bush drop of SPEC-0060 (four coins of one kind, offset in x and z), which spawns items, not effects.
34. Resolved: the sway is the View Map icon pulse (section 9). The selected icon takes the phase as its two 4.12 scale fields; the amplitude scales the hero marker's colour. It is not an effect.
35. Resolved: Frame tilt. It is the shot bounce, used by the hero shot hit tests and the Juppie boss's hit helper; no effect kind uses it. The second mix pairs the up axis with the first mixed direction, and the up axis is then rebuilt from the two directions (section 10). **[High]**
36. Resolved: 162 is requested by the water footstep; the lookup finds nothing and nothing is set up (3.16, Edge cases). The SPEC-0021 cue bag is the mote eater, not an item path.
37. Resolved: 8.8 write rule. The rounding (multiply by 256, add or subtract one half by the sign, truncate toward zero) and the wrap (the low 16 bits are stored, nothing is clamped) are the game's own conversion code, one setter per field. The field at 0x3A is written by the same conversion from the last source word; the setter of the field at 0x26 is never called. Test vectors 17 to 19 and the wrap example in the edge cases hold. **[High]**
38. Resolved: the fixed words written when the shared effect manager block is built are the range table of the common-object table (per range: the count, the first id and the last id of the global model ids 0 to 322); they are model ids, not effect ids (Where it lives). **[High]**
39. Resolved: the routines that take effect id 0xFFFF are the start-up initialisers, the entries of the start-up table walk (SPEC-0026), each called once at boot with the arguments (1, 0xFFFF) and never again (Edge cases). Each belongs to the spec of the table it fills; the 0x50-byte records of section 18 are the swing-trail colour gradients of 3.8.
40. Resolved: Behavior 11 is the debug vibration tester, not a glow. The centre is the camera position and the distance is to the hero; the value formerly read as a draw scale is the rumble duration (1.0 s) and the zero argument is the rumble preset. It requests a rumble (SPEC-0033 Rumble), draws nothing and is not reachable in retail, so the edge drop is never seen in play.
41. Resolved: the timer clears the shared input-enable word at selector/input object +0x04 except when previous raw-held L3 is set, or when current raw-held L3 is set on a counter multiple of the clamped period. The counter increments before the modulo check; period 0 disables the action (section 11; SPEC-0033, Controller map and Behavior 1A). **[High for the gate and cleared field]**
42. Resolved: remote effect path. The packing routine is called only by a debug statistics routine that has no caller, and the send it makes goes to a host link, so the path never runs in play; implement nothing for it. **[High]**
43. Answered: the effect step is 1/60 s and the UV scale K is the main-loop pass counter (Where it lives); the per-tick scale of the other effect and sine routines is that of SPEC-0007.
44. Resolved: the pulse phases advance once for each eligible mesh draw that uses CPU vertex animator 3, not once per game tick. Multiple such mesh draws in one main-loop pass advance the shared phases multiple times; a pass with no eligible draw does not advance them. See SPEC-0003, CPU vertex animators. High.
45. Resolved: the intensity byte is the alpha byte of a packed colour record (red, green, blue, alpha as four 32-bit words). The pulse copies the whole preset into colour records of the owner's built draw data, so the intensity is the alpha of those drawn items (64 to 250, with 128 as 1.0); the phase float is a texture scroll offset (Behavior 12.4). **[Medium]**
46. Partly resolved: the floor word 0x374D00 is set to 64.0 once at boot; the two 16-byte presets at 0x374D10 and 0x374D20 are zero until a draw copies 16 bytes into them from the drawn object's +0x3E0 and +0x3F0 (Medium-High). Still open: the base records' writer and the drawn object's role.
47. Resolved: the five series constants are fixed program constants of the boot program (they also appear in a vector-unit program in the boot image). No game data file holds them, so this spec is their source (Behavior 12). **[High]**
48. Resolved: no routine in the boot program reads the floor word; the pulse takes its low end from the boot word 0x2DCAA4 (Behavior 12.5). **[Medium-High]**
49. Pulse fold branch: partly resolved. For phases away from fold boundaries, the reducer selects the nearest interval k and gives x = k + 0.25 − |B − pi/2|/(2 pi); intervals change at |B − pi/2| = (2k + 1) pi. The two one-sided fold values at a boundary are −0.25 and 0.75 (section 12; test vectors 31 and 151). Exact console arithmetic at a boundary remains open.
50. Resolved: for bytes 1 to 0x27 the child is the model of common object byte + 283 from the common-object registry, which both load paths pass; a byte above 0x27 gives none; without a context the index rule applies (2.1; test vector 84).
51. Resolved: the gate reads the node's spawner pool word, written for every node when its group is bound (3.16). A node of an unbound group has a zero pool word and gets no spawner; no per-start owner allocation is involved.
52. Resolved: the spawner of a negative base period fires one burst, then drains and dies when its particles expire (3.2 step 1). It needs no tracking by the effect object.
53. Resolved: the bind keeps the record's cull only when it is above 0 or exactly -1; -2 and 0 take the definition's value. Node +0x5C holds 86 distinct material values (87 counting -1) (section 6; test vector 86).
54. Resolved: switch on (a type-40 entry, first listed placed index only) sets +0x48 = 1 and starts nothing; switch off clears +0x48 and drains (section 6).
55. Resolved: the bind is a copy of the whole 0x90-byte definition, after which the record restores its own placement fields (section 6); the five live slots start cleared. Stop drains every live spawner (3.16). The per-pass check reads the low byte at +0x44 as the record's zone number for the zone query (section 6, Zone gate; High). After the bind, +0x54 holds the definition's emitter count, since it is not among the restored fields (Medium).
56. Reset for an unknown effect id: the exact effect of the reset on the record is not confirmed.
57. Resolved for the switch path: it only sets or clears +0x48 (section 6). No other writer of the flag is known.
58. Resolved: all 116 distinct ids of the 5,139 placed records resolve to definitions (section 6).
59. **Effect group owner.** Resolved: the group is the hero's Mote power-attack and charm effect object set (Behavior 13; SPEC-0036 owns the per-object roles). **[High]**
60. **Effect group counts.** Resolved: the start-up initialiser builds 22 top-level objects, the teardown releases 30 effect objects, and the group update updates all 22 each pass (Behavior 13). **[High]**
61. **Driver helper sets.** Resolved: the hero flag at +0x14C picks Tor's helper set (0) or Roberta's (non-zero); their steps are listed in Behavior 13, and SPEC-0036 owns them. **[High]**
62. **Driver call rate.** Resolved: the driver runs once per main-loop pass for the active hero only, inside the block gated by a game-state code below 9, so it is frozen in pause (Behavior 13). **[High]**
63. **Group update constants and tests.** Resolved: the anchor offsets are (0, 2.0, 0, 0) twice and (0, 1.5, 0, 0) once; the second input is the hero's per-tick displacement x 60, a velocity; eight objects update only while active (Behavior 13). **[High]**
64. **Kind start meaning.** Resolved: the kind is the attacker's element (0 for the character base class; the monster row's type T, or the selected attack slot's damage alignment, for the enemy class), and it picks the swing trail's colour gradient (3.8, section 18). It is unrelated to effect kind 3 of the shared list. **[High]**
65. **Kind start tables and manager.** Resolved: the trail manager pointer 0x35C8A0 is set once at start-up to the block at 0x35C6F0 and is never null afterwards; the record table is the eight static records of section 18 at 0x370DF0, written once at boot (3.8).
66. **Effect code 297.** Resolved: 297 is not an effect id. It is an id in the common-object table (resource-9 model 14); the swing trail takes only that model's material group (group 11: additive, PSMT4, 32 x 64) (2.9, 3.8). **[High]**
67. **Kind start vectors.** Resolved: the two vectors are the trail's two edge points (slot A applied to entry +0x00, slot B applied to entry +0x10); the holders are animation-effect pool elements, and only list entries with byte +0x48 = 0 start a trail (3.8). **[High]**
68. **Type setup units.** Resolved: A +0x60 of a preset node is a total particle count, A +0x64 the particles per burst and A +0x74 the cycle limit (3.9); none of them is a lifetime or a frame value.
69. **Type setup tables.** Resolved: the 27 preset records (0x80 bytes, cap at +8) are at 0x3711B0 and the first-time tables at 0x371F30, 0x3727A0 and 0x3736D0, all filled once at boot; they are filled from six source tables in the file image (size start rows 0x2DB4E0, size rate rows 0x2DB690, 0x2DB840 and 0x2DB9F0, C sources 0x2DBBA0, B sources 0x2DC410) and from the preset header words (Where it lives, 3.9). The header words are given in 3.9 (preset header values) by user decision. **[High for the locations]**
70. **Type setup use.** Resolved: the three sub-records are the node's C, A and B records; the types are used by the emitter nodes of world-object parts whose u16 at +0x64 is 1 to 27, never by resource 9 (3.9). Still open: how many world-object parts carry each type (Open question 141).
71. **Total count 0.** No shipped world-object preset node has a zero total (3.9). What the original stores if a synthetic or mutated preset node has total 0, causing the cycle-limit division by zero, remains unknown. **[Low]**
72. Resolved: the timed emitter is carried by the enemy class (SPEC-0029). It shows the item an enemy stole from the hero for 180 ticks (3.10). The 0.45 is the size at which the item model is drawn on the enemy's frame.
73. Resolved: the id at +0xC30 is an item id, looked up in the item model registry (SPEC-0021), not in the definition table or the id registry.
74. **Contact-normal list owner and limit.** Resolved: this is the character collision block's contact-normal list, owned by the ground/collision path (SPEC-0018); its owner link supplies the callback test. The accumulator refreshes matching normals and appends only while the count is below 19, so the maximum is 19 (3.11). **[High for the maximum; Medium-High for the owner path]**
75. Answered: Edge cases, “Timed entry list, stale slot,” describes the skipped-entry and stale-slot removal behavior (Medium).
76. Resolved: the +0x26C flag belongs to the follow camera. Its normal follow update sets it to 1 when the shaped right-stick drift base is nonzero, otherwise 0; the streaming-load freeze also zeros the drift base and flag. The camera collision update uses the flag as an alternative to a nonzero focus-character owner callback when aging contact-normal entries (3.11). **[High]**
77. Resolved: these are linear pieces of rumble magnitude envelopes (3.12). The state word gates progression/output; expiry enables a linked successor. Joining compatible adjacent pieces copies the earlier start and summed duration into the later piece and ORs their states, then clears the earlier fixed slot to zero for reuse. The updater outputs the greatest current magnitude among enabled pieces, clamped to [0,1] and converted to a byte alongside the selected remaining duration. **[High for representation, gate, join and release; Medium-High for selected output]**
78. Answered: the renderer's two shared packet-batch objects own these buffers; each stages renderer-facing draw packets from linked draw records. The size rounds to 256 bytes, while the arena allocator aligns the returned base to 128 bytes. The role is established; the full packet/header field layout is not. Open question 79 retains the selector/capacity boundary. **[Medium-High]**
79. Partly resolved: the size field holds the rounded size (the base is separate), the fill helper writes exactly that many bytes, and the second-half pointer is base + rounded-size/2. Drawing passes a selected base plus the current write offset and rounded-size minus that offset; the builder writes 0x10-byte headers and 0x30-byte samples and requires more than 0xC0 bytes remaining. The selector's live domain and its capacity relationship to the second-half pointer are not established, and packet/header field meanings remain renderer-facing. **[Medium]**
80. Resolved: the 128-aligned registry stores runtime DLL class-instance addresses, not effect-definition ids. The per-instance registration record is supplied by the caller; the shared DLL manager checks the candidate image's header, links/relocates it and attempts module registration. An occupied record or misaligned address is rejected before that check; a nonzero checker result logs failure and leaves the record empty. The record stores the address only after a successful check (section 7; SPEC-0030's level-module attach path). **[High]**
81. Resolved: these six 0x18-byte slots belong to the shared rumble manager, not an effect record. Each slot holds an in-use word, an enabled state, remaining duration, magnitude change per tick, current magnitude and a link to the next segment. Five presets can request up to three linked segments each; the active updater scans the six slots and sends the greatest enabled magnitude plus its remaining duration to vibration output (sections 3.12 and 7). **[High]**
82. **Preset contents and source.** Resolved: the 27 presets are the world-object emitter types of 3.9. Their size rows, C records and B rows come from source tables in the file image, which the clean side reads from the user's boot executable (Where it lives); their header words come from the start-up code and are given in 3.9 by user decision. **[High]**
83. Resolved: **Preset tables and owner frame presets.** Different tables: the 27 presets of section 3.9 are the world-object emitter presets, and the six routines of section 5.4 are status tints with their own colour words (0x374C50 to 0x374CEF). **[High]**
84. **Preset record sizes.** Resolved: the C records are 0x50 bytes, the A records come from the 0x80-byte preset records (cap at +8) into 0x90-byte A records, and the B curve rows come from 0x34-byte source records into 0x90-byte B records (3.9).
85. **Preset selector copies.** Resolved: the blocks are a C record (converted from a 0x50-byte C source), an A record (the A copy of the 0x80-byte preset) and B colour rows (the B copy of a 0x34-byte B source); the stored ratio divides the total count A +0x60 by the per-burst count A +0x64. The 0x34 bytes give three colour keys: rows 0 and 1 and a third row repeated in rows 2 to 5, with RGB key times 0, t1 and 1.0 (3.9). **[High]**
86. **Static table ownership.** Resolved: all the run-time tables at 0x3711B0 to 0x371F2F and at 0x371F30, 0x3727A0 and 0x3736D0 belong to this spec (3.9); they are written once at boot.
87. **Burst emitter.** Resolved: the eight-slot emitter is the Faerie Sisters' Mote-eater drain (3.13): 7 or 8 slots, one revealed every 225 (or 257) ticks over 30 s, 20 charge used per slot, ticked once per live sister; the kind-15 hit is the Leach MP drain. **[High]**
88. **Hover machine.** Resolved: it is the item showcase of SPEC-0021, which owns its rules (3.14). **[High]**
89. Resolved: section 14 builds a frame, not quad corners: row 0 = normalise(D x X), row 1 = D with its y removed, row 2 = normalise(up x D), row 3 = the anchor (14.1). The object is drawn in draw mode 5 as one of the hero's Mote power objects.
90. Resolved: the wrap is the floating-point remainder by 1.0, so negative values give offsets in (-1, 0]. The two time inputs are the animator's per-axis speeds, in texture widths per 4000 passes. No material on disc selects this animator, so it never runs.
91. Resolved: a packet holds at most 24 samples; sample 25 starts a new packet if the caller's remaining output budget permits. The 0xC0 comparison is a reserve check, not a packet-size limit. A full packet is 0x4A0 bytes: two 16-byte headers and 24 three-quadword samples. Static builder behavior is High confidence; normal-play visibility of a packet split is not established.
92. **Batch count and packet parameter.** Resolved: the twenty pointers walked by the batch flush are the twenty sprite batches of 4.7. The three-slot trail object of section 17 is separate. The 16-bit packet-header parameter is the material group index: the batch flush reads it from the selected 0x270-byte shader group at +0x20, whose byte is that group index (SPEC-0003, Group layout). The packet builder copies that value into its DMA tag. **[High for the field flow and group-index identity]**
93. **Static effect records.** Resolved: the eight records are the swing-trail colour gradients (section 18): colours A, B and C, count c, cap n, fade distances d0 and d1, colour steps per update, u per unit length and the u offset of the draw wrap; the trail start reads them as its record table (3.8). **[High for +0x00 to +0x40; Medium for +0x44 and +0x48]**
94. Resolved: the only chain is the Gelert Assassin's light arc. Its set-up sets the scale to the light-ball speed (20) divided by the frame rate (60), 1/3 per tick, with direction vectors as velocities (section 19.1). **[High]**
95. Resolved: a non-zero result means contact, so the trigger fires when a ball touches the hero. The first and second links use a contact test of a 2.0 sphere against the hero's body; the third uses a 1.0 sweep from the hero's mid-body point along the link's direction. Every success hits the hero, with the chain's value (the light-arc damage, 20) (sections 19.2 and 19.3). **[Medium-High]**
96. Resolved: the ramp is subtracted from the y component (the record sinks), it ends at 0.08 after 16 float steps, and a frame builder then rebuilds the record's frame. The jitter records are never updated in play (section 20). The starting value of the ramp stays unknown and does not matter for play.
97. **Reference direction.** Resolved: 0x3932E0 is world up (0, 1, 0, 0), written once at boot and never changed (Where it lives). It is the reference direction of the quad (section 14), the chain frame (section 19) and the ring (section 22).
98. Resolved: nothing builds or updates the trail object of section 17, so its head speed is never set.
99. Resolved: the word is the current display buffer index (0 or 1). Every model draw stores its buffer argument there, so the run-time value is 0 or 1 by buffer and the disc value of 0 holds only until the first draw (Where it lives; section 21.4). **[Medium-High]**
100. Resolved: the blended components are stored as floats with no whole-number conversion, and w is not blended.
101. Resolved: the per-slot helper rebuilds the slot object's lane mask from the words at +0x12A0 (x, y, z) and +0x12A4 (w). The slot-0 set writes the reference vector at +0x1290, those two words, the mode and the counter (21.6).
102. **Oscillator slot count.** Resolved: there are 15 slots (indices 0 to 14), the same records that the character colour push fills from the 15 colour words of the character's spawn record (21.4). **[High]**
103. **Ring distance and placement.** Resolved: the "ring" is a 3 x 4 grid with 8-unit spacing, its tested length is the full 3D distance, and the routine is unreached (section 22). **[High]**
104. **Spark item count.** Resolved: the routine is one step of the world's +0x29F0 record list and its loop returns after item 0, so only item 0 is placed (section 23; SPEC-0028). **[High]**
105. **Spark item vectors.** Resolved as Open question 104: the vectors are the world object's frame, owned by SPEC-0028 (section 23). **[Medium-High]**
106. **Spark hit test.** Resolved as Open question 104: the tests for items 1 to 4 never run; the routine is not the kind-based impact helper of Open question 33. **[High]**
107. **Timed pool.** Resolved: the 48-slot pool is the sound-cue voice pool. SPEC-0031 owns its cue-playback rules; section 24 points to that sound system and classifies the pool as non-effect data. **[High for the owner and count]**
108. **Tween scalar and modes.** Resolved in part: +0x130 is an active flag; the step adds the selected source vector multiplied by +0x134 / total steps once per world update, with modes 0 to 3 selecting or negating the vectors as described in section 25 and SPEC-0028 section 15.1. The scalar's unit and source behavior for other modes remain open. **[High for operations and supported mode mapping; Medium for scalar interpretation]**
109. **Tween apply and parts.** Resolved: the attached-part updater belongs to the movable-block records in SPEC-0048, not the +0x3850 effect tween-node list. On the world update, an active block record visits its counted list of 32-byte part entries and refreshes non-null part transforms from the updated block pose (section 25; SPEC-0048, Movable blocks). **[High for ownership, dispatch, and transform refresh]**
110. **Owners of the new routines.** Resolved: the grid is unreached, the spark spawner is the world +0x29F0 record step, the timed pool is the sound-cue pool and the tween is the world tween-node list (sections 22 to 25). **[High]**

111. Answered: the guarded value is the exact tracked second component, copied into the origin before the tracked value is refreshed from the stored origin (section 26, steps 3 and 7b; Edge cases).
112. **Burst records, contact pass and cue.** Resolved for the observed start/update path: the ten 8-byte records are a recent-target suppression list for the A7 area-hit path, with 60-tick identity holds; the update then scans actor candidates and the current cell's primitive list. Successful primitive contacts can push the primitive and play a cue selected from its contact field. The start requests positional sound cue 5 and active updates refresh that voice; this is not an emitter-effect start. Other callers and any different behavior they select remain open. **[High for record use, ordering, and cue request/refresh; Medium-High for primitive-contact effects]**
113. Resolved: the keyed claim and bank slot checks use the same five-child predicate. A slot is busy when at least one of its five child pointers is non-null and that child's +0x90 word is not 4. Null pointers and children whose word equals 4 do not make the slot busy; the slot's +0x10 data pointer is not the tested value. **[High]**
114. **Find-or-create table owner.** Resolved in part: the backing store is the blackboard message store exposed by the level-script APIs (SPEC-0030 rows 149–150), grouped by world identifier and keyed by message type. The value word is a payload. Type 2/value 30 selects a return locator after a world change; type 3 values 1 and 2 select script-specific arrival sequences on the linked world-129 and world-131 routes; type 1 carries arena results on the world-90 and world-157 return routes. For world 90, value 1 selects the win progression and the stored value -1 follows the loss route, paired with type 2/value 10; a missing entry also reads as -1. For world 157, value 1 selects the win progression and the writer stores -1 for a loss, paired with type 2/value 20; the arrival clears type 1 to 0. These examples do not establish a global type/value mapping. The type-3/value-0 write for world 130 has no matched reader in the recovered census. **[High for the specific reads, writes, and actions; unresolved for unmatched writes and any general type/value dictionary]**
115. Resolved: the block is the hit-flash key set: three key vectors, three key colours of four bytes, and signed 32-bit integer stage lengths 4 and 10 frames at +0x3C and +0x40 (section 32; High).
116. Resolved in part: all five child slots are visited; nulls are skipped, and a non-null child is unlinked only when its +0xC4 backlink equals the parent, clearing the child's +0xC4 and +0xBC words. Bit 0 calls the parent effect object's release helper after the link walk; it does not release the children. In this disc build, the parent-release shim performs no allocator request or effect-object state write. Whether an external pool retains, reclaims or reuses the detached parent remains open (section 31; High for the call chain, unresolved for ownership and reuse).
117. Resolved in part: for the first timer, 2.0 is the initial scalar and 25.0 its upper target; for the second variant, 1.0 is the initial scalar and T is the upper target. The child receives round(scalar x 256) in its signed 16-bit +0x68 field. The physical or visual meaning of that child field and whether a run reaches its target remain open. **[High for parameter roles and writes; Low for downstream meaning]**
118. Answered in part: the active count decrements first; on the zero-count tick the timer reset runs and skips scalar/child updates. Reset clears the active count, scalar, target and step, restores the base value and calls the stop/unbind helper. The helper visible effect remains open (section 33). [High for ordering and reset writes; unresolved for the helper outcome]
119. Resolved: the second timer uses the same numeric value 12.8 in two distinct roles: as the duration divisor for T and as the numerator of the per-second scalar increment (12.8 / frame rate per tick). This does not establish the child field’s physical unit. **[High for the operational roles]**
120. Resolved in part: when the source object is found, it moves 5.0 units along the normalized direction to the global reference point; each non-empty child point is set 5.0 units along that same direction from the child-list base. The exact child-list base point and behavior when source and reference coincide remain open (section 34). [High for the source move and offset; open for the base and zero-distance case]
121. Resolved: in the quaternion form, sine multiplies the axis components and cosine supplies the scalar component. In the axis variant, the first parameter is used directly as a radian angle to form (cos q, 0, sin q), without degree conversion (section 35). [High]
122. Resolved: **Pool identity and busy test.** The ten-entry pool in section 29 is a separate 0x3D0-stride controller pool, not the 0xB0-stride clip-animation instance pool of section 3.1. Its +0x300 busy field is a countdown in ticks: non-zero is occupied; updates decrement it and zero triggers reset. Each controller contains an effect instance at +0x330. **[High]**
123. Resolved in part: each of the bank's six records resets the effect sub-object at +0x230 by stopping and clearing its five child links, then reinitialising the sub-object. Children with live-child count below 1 are moved to state 4 if needed, invoking any callback and buffer-release hooks; children with count at least 1 enter state 5 and have the +0x50 vector cleared. The meaning of the entry-reset byte at +0x67 (set to 255) remains open. **[High for the reset and child-state branches; Low for the byte's meaning]**
124. **Stop-group routine.** Partly resolved: the group stop covers one effect object at +0x5D0, a controller at +0x280 with a linked effect at +0x10, and four active-checked effect records from +0 through +0x1E0 in 0xA0-byte steps (section 31). Their exact gameplay categories and the producer of the separate +0x5D0 object remain open. **[High for layout and dispatch; Low for gameplay categories]**
125. **Script start and stop argument.** Resolved: both host entry points range-check the first argument against the selected world's type-39 placed-effect count and use it as a zero-based record index; the chosen record supplies the definition id. This applies even when the numeric slot argument also equals a valid effect-definition id, and even when it does not (definition 162 is absent). The separate program path that starts definition 938 first looks it up by definition id and builds a detached effect object; it does not pass that id to the StartEffect slot entry point. **[High]**
126. Answered: **Alpha-test setting.** All 123 resource-9 effect material groups use context-2 alpha test GEQUAL 120 with AFAIL = FB_ONLY and depth GEQUAL (section 2.8). Alpha failures still write colour and suppress depth, so the state does not discard texels from the frame buffer. Synthetic check: alpha 119 fails and writes colour but not depth; alpha 120 passes alpha test and proceeds to the depth test. Confidence: High for the disc values; Medium-High for the rendering consequence.
127. **Negative base period.** Resolved as Open question 52: one burst, then drain and die. Definition 542 (id 159, test vectors 68 and 71) shows it.
128. Resolved: **Runtime base of the group table.** The item-stream loader retains resource 9's type-2 shader-list payload in the shared manager's slot-3 shader-list pointer. The final slot-3 binder passes that pointer as the node material context; the node binder resolves a nonnegative material index to the relocated group table at shader-list +0x10 plus index × 0x270 (sections 2.8 and 3.7). The type-2 payload and the shader-list expected by the disc material indices are the same object. **[High for the static startup load and bind path]**
129. **Lifetime units.** Resolved: C +0x38 and +0x3A are a kick interval in seconds, not a lifetime (0.0195 s = 5/256, about every tick; 0 = every tick; negative = never). The particle life is A +0x78 +/- A +0x7C seconds.
130. **Gravity.** Resolved: there is no separate gravity field; C +0x20 is the vertical acceleration along world up in units/s^2 (negative = falls), and a C +0x2C of 0 cancels it (2.7).
131. **Launch basis codes.** Resolved: the C +0x3F codes are the spin axis of model particles only (0 random, 1 X, 2 up, 3 Z, 4 to 6 the particle's own frame rows), with the axes of the basis block. Sprite particles and the acceleration never use them (2.6).
132. **Node colour values.** Resolved as Open question 15: sprite half-extent factors, not colour.
133. Answered: section 2.8; these classes are descriptive counts derived from materials, not game-level names stored on disc.
134. Resolved: see Open question 36.
135. **Placed-record start route.** Resolved in part: a script StartEffect or StopEffect slot argument is an index into a world's type-39 placed-effect table; slot n addresses record n (SPEC-0028, Type 39; SPEC-0030, rows 5 and 6). One concrete mapping is established: module 1249 uses slots 0 through 15, all mapping to definition 4006 in pack 1249; its StartEffect calls request slots 0 through 7, and its StopEffect calls request slots 0 through 15. Still open: the world table and definition mapping for other calls, and whether their callback gates are reached in play. The remaining placed-only ids and per-call mappings are not established as started effects (section 2.9). **[High for the host slot rule and pack 1249 mapping; Low for the remaining mappings and play reachability]**
136. **Boot-data runs of effect-like ids.** Partly resolved: the paired run of ids 6000 to 6005 is the Mote-eater drain's per-Mote id table at 0x2DCAE8 (3.13). The proposed 16-bit sequence 101 to 113 followed by 115 to 118 does not occur contiguously in the preserved boot image; similar contiguous runs include 114 and are character-code lookup data. The proposed sequence's location, source and reader remain unknown, as do split or computed forms. **[Low-Medium]**
137. **Effect group for codes 297, 193 and 453.** Resolved: 297, 298, 193 and 194 are common-object ids, not effect ids; 453 is a group-3 lookup whose definition is absent, so the per-joint effect set (ids from the table at 0x3132A0) binds nothing (2.9). **[High]**
138. Resolved: the routine maps item ids, not attack kinds; it runs for every world item set-up and for the family equipment change, and the item updates the bound effect each tick (3.15; test vector 85).
139. **World-pack reach for section 2.9.** Resolved in part: pack 1382 is in the static reach set through pack 1391's pending-area setter, which has no flag test (SPEC-0026, Reach and pending-area entries). Its 18 definition references, ids 2202 to 2219, therefore move from the only-unreached group into the static reach group. Packs 742 and 1259 are among the 17 statically unreached packs (SPEC-0026, Reach); the three remaining ids are 1442 and 1445 in the pack-1259 character region, and 4015 in pack 742. This is a pack-graph result, not proof of story-gated or physical access. **[High for the static graph and count; Medium for practical play access]**
140. **Save-point glow look.** Resolved: within 25 units definitions 400 and 404 show the full glow and a cyan point light of radius 20; farther away only 404's sparks and its larger disc show (section 36). **[High for the values; Medium for the picture]**
141. **Preset values and serialized-node counts.** Resolved in part: by the user's decision the clean side reads the preset values from the user's own boot executable, not from this spec. The size rows, the C sources and the B sources are read from the six source tables of Where it lives (file offset = VA - 0x100000 + 0x80) and filled as 3.9 describes. The five header words of each preset are given in 3.9 by user decision (Open question 143). Section 3.9 gives the disc-wide counts of serialized type-29 emitter nodes, owner records and packs by preset type. Whether these nodes correspond one-to-one to distinct rendered mesh parts, and which are reached during play, remains open. **[High for the serialized counts; open for mesh mapping and play reachability]**
142. **Other effect lists under code 9.** Resolved: the swing trails draw but do not age, and the hero attack-effect group is not driven, while the game-state code is 9 or above (3.8, Behavior 13). The world group at +0x2CC8 (50 particle records, stepped by its four-state oscillator) steps only while the code is below 9 and is drawn frozen under code 9 (4.6, Place in the pass and freezing). **[Medium-High for the world group]**
143. **Values held only in the start-up code.** Resolved: values published in the spec by user decision. The field values of the eight static records (0x370DF0 to 0x37106F) are in section 18, and the five header words (+0x00 to +0x13: base period, period jitter, cap, the A +0x68 word, life) of each of the 27 preset records (0x3711B0) are in 3.9. The clean side uses them as constants; the rest of each preset still comes from the six source tables of Where it lives. **[High]**
144. Resolved: the hero never runs the clip walk, so its clip lists start no effect or trail; its swing trail is the held-weapon trail of 3.8, requested while the attack-active word is set. The footstep dust uses its own path (3.16).
145. Resolved: the clamp vectors are set once at start-up to (255, 255, 255, 255) and (0, 0, 0, 0); the override word is cleared at trail creation and never set (3.8).
146. Resolved: the three records are written by start-up initialisers with the values of the table in 3.8; their users are the held-weapon trail (hero, enemies, Black Knight), the two-sprite marker and the enemy projectile.
147. **Drain words without a reader.** The word at 0x2DCAE4 is zero in boot data, beside the Mote-eater drain amount 20 and the per-Mote effect table; its owner and meaning remain unknown. The separate drain number at 0x2DAB78 is 8, but the eight-slot cap is fixed and this word has no known reader. **[Low]**
148. Resolved: definition 400 uses child chunks 8 and 9, 404 uses common object 295, 188 uses child chunks 16 and 17 (2.1).
149. Segment join and release (synthetic): A is in use with state 1, duration 2, slope 1 and start 0; A links to B. B is in use with state 0, duration 3, slope 1 and start 2. Their endpoint and slope differences are both 0, so the join succeeds: B has start 0, duration 5 and state 1, retains its own successor link, and A's six words become zero.
150. Positional attenuation at outer boundary (synthetic): base intensity 1, r_in = 10, r_out = 50, listener distance d = 50. Then d² = r_out² = 2500, so interpolation returns 1 − (2500 − 100)/(100 + 2500) = 1/13 and the positive-intensity request path runs. For d just greater than 50, d² > 2500, intensity is zero and no request is made.
151. Pulse fold away from a boundary (synthetic): B = 0.7, not a multiple of the 0.2 phase step. m ≈ 0.13859, k = 0, so x ≈ 0.11141. The stored-coefficient series gives P(x) ≈ 0.64422 and the intensity is about 216.91, which rounds to 217.
152. Vibration tester period gate (synthetic): period 4, previous counter 3 (so the incremented counter is 4), previous L3 clear, current L3 set: the input-enable word is preserved. With the same counter and both L3 snapshots clear, it is set to zero. If the incremented counter is 3 and only current L3 is set, it is also set to zero. If the slope difference is 0.001 or greater, the strict tolerance fails and neither segment changes.

