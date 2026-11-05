# SPEC-0028: World lights, effect spawns, object records and collision primitives

- Answers: proactive (open close-out); proactive (open question sweep, method slot audit: the wall pass offers every visited frame item to the face recoil callback); R-0006 (extension: world-pack light, effect-spawn and object records), R-0050, R-0076 (breakable volume point roles, in sections 6.1 and 6.4), R-0063 (movable block pass: when it runs and the block's shape), R-0040 (in part: the mesh a second-list breakable draws), R-0078 (the five faces of the moving-rectangle test), R-0081 (pack 951 fireplace records and zone gate), R-0084 (atmosphere-zone gate flags, in section 11), R-0115 (in part: Tor blade-recoil gates, hit window, shape geometry, dynamic-pool producer boundary and all three candidate caller paths; runtime pool population, transform provenance, exact set of wall frame items in reach, accepted count and visible recoil count per swing remain open); R-0118 (blade triangle input sources and effective reach; busy gate answered under Blade recoil); R-0125 (DeactivateObj and InitObjOn effect on ground and walls); R-0157 (sections 7 and 13: breakable cylinders are the solid of a lock chest); R-0191 (which projectiles run the world projectile step); R-0205 (the walk carries nothing; riders are carried by the ground attach); R-0221 (pack 1299 pad trigger, landing, mask tree, resets, walking route); R-0226 (section 7.2: type-10 groups owned by a placed object, pack 777 ladder 0); proactive (open question sweep, world details); proactive (open question sweep, round 5); proactive (spec consistency pass); proactive (anticipated questions); proactive (flag and walk survey); proactive (open question sweep, round 9); proactive (open question sweep, round 12); R-0230 (answered from the code; not run in the original)
- Status: Draft
- Disc build: SCUS-97367 v1.00 (same fingerprint as SPEC-0001)
- Confidence: per section. Object pools (section 15): High for sizes, counts and allocation sizes; Medium-High for the pool roles (script callback tables); Medium for initial words and the release rule; Low for the table and scene-list layouts. Record layouts and counts: High (measured on disc, all 192 world packs). Effect spawn fields: High. Placed-object record layout and counts: High; placed-object setup, draw gate and reset: Medium. Lighting (section 8): High that resource 527 is never read and that type 30 light sets reach characters only; Medium-High for the light-type roles (type 1 ambient, type 3 direction from the light to the object). Section 10.1 is a pointer only: the two rotation steps once listed there belong to the sky dome (SPEC-0046), High. Second transform block of a placed object (section 7): High; it is the rest block (section 7.1). World geometry is unlit, High (section 8). Placed objects in the object lists (section 10): Medium-Low for list membership; draw distance: Low (the draw path does not use the designer values). Type 28 records are the camera tracks of SPEC-0052: High. Meaning of the type 30 header flag: Low. Type 39 +0x44 zone number and the per-pass zone gate: High. Type 29 part records (copy of the bound frame record): High. Type 11 extra lists n1 to n4: High. Dialogue line table: Medium-High. Type 40: layout High (counted over the disc), load-time steps Medium, list-entry fields, kinds and keys High; the never-filled built-in pools (section 15.1): High. Type 41: layout and load-time steps High; per-frame zone selection Medium-High; field roles Medium-High except the render scalar (Low). Type 41 is a per-area atmosphere zone table selected from the camera eye; it is not a camera volume and not a light table. Type 11 collision primitives: header counts High (189 chunks); the 0x20-byte header at the start of every primitive record, and the volume kind byte of each list (spheres 1, cones 2, cylinders 3, kind-4 rectangles 4, kind-7 rectangles 7, fourth list 8), High (every record of all 189 chunks); sphere records (0x40 bytes, equal extents) High for the layout and count (189 of 189 chunks, 6,678 in total); cone records (0x40 bytes) High for the count (83 in 7 packs), Medium for the field meanings; rectangle records (0x70 bytes, kind-4 list at +0x40 and kind-7 list at +0x3C) High for the layout and counts, extent signs Medium; cylinder records (0x50 bytes) High for the layout and count (the count matches in all 189 chunks); other 0x50-byte records (the fourth header list): layout and count High, role Low; primitive-array pointer order High (all 189 chunks); order of auxiliary arrays varies. Doors (section 13): High for the records and the count, Medium-High for the rule and for the script/default split, High for the door prompt and confirm (SPEC-0006). Trigger volumes (section 13): kind 2 never fires in play, High; trigger pass actors, cell lists, fire paths and block words, High; the dispatch and the kind split, Medium; proximity and finite-cylinder geometry, Low-Medium; the rectangle test, Medium; the apply rules (script hook, element gate, break, cue and the hero melee mismatch reaction), High (SPEC-0036 and SPEC-0048); the blade recoil (the former press variant), High for hook gates, writes, mappings and tested geometry; Medium-High for event-cache ordering; the callback route, candidate-pass order and type-10 source are High; exact live candidate sets, accepted counts and visible repeat frequency remain open; the disc sources of the volumes (every type 11 primitive record is a volume: its own header plus its body, section 6), High; the entry list at the type 11 pointer +0x4C (count at +0x24), High; the script trigger fire gates by flag bit, Medium-High; save points are not trigger volumes, placed objects or placed effects, High. Owner primitive list walk order (section 6.6): Medium-High (SPEC-0018 holds the dispatch behaviour). Loader handling of this spec's chunk types (Layout 0): High for dispatch and order; Medium for the type 40 load-time steps, High for type 41. Contact-time crossing solver (Behavior): Medium-Low. The blade recoil multiplier readers (section 13): High. Animated placed objects (section 7.1): High for the fields, the stream, the script states and the per-tick steps; Medium for the edge branches of the sampler. Pack 1299 looping-door puzzle (Behavior): High for the rules. Riding, carrying and collision with moving placed objects: owned by SPEC-0018 (Behavior, pointer only). Six-slot look-ahead (Behavior): it is the foliage bush shot test of SPEC-0060, High. Object record accessors (Layout 14): High for the offsets and widths, names open. Type 16 foliage patches (section 16): High; a summary only, SPEC-0060 owns them. Open items are listed under Open questions.
- Related specs: SPEC-0025 (save game; owner of the save-point rule), SPEC-0027 (camera; its eye position selects the type 41 atmosphere zones), SPEC-0046 (rendering: draw records, far state, light slots, model-part visibility, the screen post pass and the sky dome that receive the type 41 output), SPEC-0006 (areas; world buffers, doors and transitions), SPEC-0026 (game flow; world buffers and travel), SPEC-0018 (object collision and the rectangle and box rules), SPEC-0040 (character side of collision), SPEC-0030 (script trigger callbacks), SPEC-0036 (Tor's blade recoil, attack sub-state 13; contact routines and cues on world breakables), SPEC-0048 (world breakables and their break routes), SPEC-0004 (vertex colour and lit-vertex arithmetic), SPEC-0060 (type 16 foliage patches), SPEC-0052 (camera tracks, type 28), SPEC-0031 (ambient sound loops, the type 11 fourth list).

## Summary

Area packs (SPEC-0006) hold several small record tables besides geometry: camera tracks (type 28), light sets (type 30), placed effect spawns (type 39), a dialogue line table (type 1000), object headers with collision primitives (type 11), placed objects (type 29) with their part records, and two tables whose fields are only partly established (types 40 and 41). The type 41 table holds per-area atmosphere zones (a colour for the screen post pass, a screen overlay colour, a render scalar and the blend factor of the sky dome's role-10 layer), chosen each frame from the camera eye position; it is not a camera volume and not a light table. A separate text resource (527) describes three directional lights, but the game never reads it; the same three directional lights, together with an ambient light that exists only in the program, make up the scene's default lights, built from fixed constants (SPEC-0046). This spec gives the layouts and counts of each, the roles of the light-set entries (which light characters only; world geometry is unlit), the rules for placed effect spawns, the draw gate of placed objects, which asks the level's binary grid (SPEC-0006) whether an object's zone is active, and the layouts of the collision primitives (spheres, cones, cylinders, boxes and planes) that the contact and move rules of SPEC-0018 and SPEC-0040 use. Effect definitions (what a spawn plays) are the shared effect table described in SPEC-0034; whether every spawn's id resolves to one is open (Open questions 4). The same pack-level data is also the source of trigger volumes: section 13 gives the volume records that the hero and other actors are tested against each tick, the kinds of test they use, and the rule for applying a hit (a script hook, an element gate, a break with a piece push and a hit cue), together with the press (use) variant. Every type 11 primitive record is such a volume, with its own 0x20-byte header. Doors are kind-7 rectangle volumes with flag bit 1 and a class other than 500 and 501; section 13 gives their records and the rule that takes the hero to another world. The game's moving platforms, lifts and bridges are animated placed objects: type 29 objects with a recorded single-transform motion that level scripts start and stop (section 7.1). Characters ride them through the ground attach of SPEC-0018, and moving objects push characters through their own inline primitives (SPEC-0018). The one six-state "platform" is the looping-door puzzle of pack 1299 (Behavior). The "six-slot look-ahead" once listed here is the foliage bush shot test of SPEC-0060 (Behavior, pointer). The type 16 foliage patches are summarised in section 16 and owned by SPEC-0060. Save points are not among the records of this spec: they are not placed objects, trigger volumes or placed effect spawns (section 13, Save points). Times are in seconds; the game runs at 60 ticks per second (SPEC-0007).

## Where it lives on the disc

- World packs are the resources whose first chunk is type 31. There are 192 of them: 191 area packs and the front-end pack (resource 532). Each chunk below is a top-level chunk of a world pack, except the inline type 5 chunk and the inline mesh and type-11 header that a type 29 object carries.
- Counts over the 192 packs: type 28 in 191 packs (2,010 records); type 30 in 140 packs (2,449 records, 3,265 entries); type 39 in 158 packs (5,139 records); type 1000 in 131 packs; type 11 in 189 packs (of which 164 have kind-4 rectangles, 142 kind-7 rectangles, 175 cylinders, 131 spheres, 7 cones); type 29 in 121 packs (3,495 objects in all, of which 642 have part records, with 7,201 part records); type 40 in all 191 level packs; type 41 in 138 packs (185 records).
- Trigger volume records (section 13): every type 11 primitive record is a volume. Each record starts with its own 0x20-byte header (kind, mode, flags, gate mask, number, class or book value) followed by its body. The kind byte is fixed per list: 1 for spheres, 2 for cones, 3 for cylinders, 4 for the rectangle list at +0x40, 7 for the rectangle list at +0x3C and 8 for the fourth list (High, every record of all 189 chunks). Doors are kind-7 records with flags 0x0002 and a class other than 500 and 501 (section 13, Doors). Types 40 and 41 were measured and carry no trigger kind byte.
- Resource 527 of the archive is a 2,048-byte plain-text file (SPEC-0002 lists it as text). The game never reads it (High); its values are not copied here.
- Offsets below are relative to a chunk's payload unless a table says "in object" or "in record". The payload starts 16 bytes plus the chunk's extra field after the chunk header. The one exception is the second list of type 29: its entries are measured from the start of their offset table (section 7).

## Layout

### 0. Loader steps for this spec's chunk types

The loader's handler for each chunk type, and the walk order it needs, are in SPEC-0006 ("How the loader handles each chunk"). This section gives only the load-time facts that the layouts below depend on.

- **Type 10 before type 29.** Type 29 reads the type-10 table at load and writes ownership into it (section 7.2), so type 10 must come earlier in the chain (SPEC-0006). Types 5 and 10 also come before type 29 there.
- **Type 11.** After the header offsets are rewritten, the loader clears byte +0x4F of each 0x50-byte record in the table at payload +0x54 (the count is the u32 at payload +0x2C, section 6; the offset is from the record start, which is the start of its header, section 6). Only that byte is cleared; it is the start-delay counter of the ambient sound loops (section 6.5). High.
- **Type 21.** The loader rebases the navigation mesh table and its meshes. The layout is in SPEC-0018 (this spec does not describe it). High.
- **Type 28.** Each record's offsets are rewritten and the record is stored in its fixed slot (SPEC-0006). High.
- **Type 30.** The loader walks N records, each record's size taken from its own header (section 2). High.
- **Type 39.** At load each record is bound to its effect definition (the definition list is held in the shared effect manager; SPEC-0034). No object is created. A record with an unknown id is reset and logged. The bind copies the definition's active word into the record; all 666 definitions on disc have this word set, so every valid placed record becomes active at load even though the source record starts with zero. High.
- **Type 40.** The 21 fixed records and the counted list are rebased at load (seven offset fields each). Record 3 then gets two runtime blocks (0x80 bytes and 4 bytes). Medium.
- **Type 41.** Each 0x50-byte record is rebased at load: its point-list offset at +0x44 is rebased when its point count at +0x40 is nonzero, otherwise cleared. Then every record whose gate key (+0x24) is nonzero gets the value 28.0 written to the alpha lane of its vector B (+0x1C); other records keep their disc value (section 11). High.
- **Type 1000.** Relocated and stored (section 5; SPEC-0006). Medium-High.

### 1. Type 28: camera tracks (191 packs, 2,010 records)

Type 28 records are the camera tracks of SPEC-0052 and SPEC-0027: the track id is the record index, and the value at +0x2C is the focal length. SPEC-0052 owns the record meaning and playback (High). A finder routine returns the last track within 500 units of a point (squared distance below 250,000), skipping tracks whose +0x28 byte is set unless asked to include them; nothing calls it (High).

Payload:

| Offset | Size | Field | Notes |
| --- | --- | --- | --- |
| +0x00 | s32 | N, record count | 4 to 43 per pack |
| +0x04 | u32 | offset of the table | 0x10 in all 191 packs |
| +0x10 | N × u32 | payload-relative offsets to records | all point inside the payload |

Record (0x40 bytes):

| Offset | Size | Field | Measured on disc |
| --- | --- | --- | --- |
| +0x00 | 4 × f32 | unit quaternion (x, y, z, w) | unit length within 1e-3 in 1,996 of 2,010 (14 are off) |
| +0x10 | 4 × f32 | position, w = 1 | w = 1 in all 2,010 |
| +0x20 | s32 | count | above 0 in 329 records |
| +0x28 | u32 | flag | 0 in 1,246 records, 1 in 764 |
| +0x2C | f32 | focal length (SPEC-0052) | about 791.4 in about 1,850 records, 0.0 in 72 |
| +0x34 | u32 | pointer-like word | nonzero in exactly the 329 records where the count is above 0, and then inside the payload |

### 2. Type 30: light sets (140 packs, 2,449 records, 3,265 entries)

Payload:

| Offset | Size | Field | Notes |
| --- | --- | --- | --- |
| +0x00 | s32 | N, record count | 1 to 106 per pack |
| +0x04 | u32 | offset of the table | 0x10 in all 140 packs |
| +0x10 | N × u32 | table | Values on disc are not record offsets. Do not use them. |

Records follow the table in sequence, not through the table. The first record starts at the table end rounded up to 16 bytes. Each record is 16-byte aligned, and its size is 0x20 + 0x50 × (A + B + C), rounded up to 16. The chain ends exactly at the end of the payload in all 140 packs.

Record header (0x20 bytes):

| Offset | Size | Field | Measured on disc |
| --- | --- | --- | --- |
| +0x00 | s16 | unknown flag | 0 in 488 records, 1 in 1,961. Character lighting does not use it. Meaning not encoded (class D; ignore it). |
| +0x02 | s16 | gate | 1 means character lighting skips the record in both passes. 0 in 2,298 records, 1 in 151. All 151 hold A and C entries. |
| +0x04 | s16 | A, count of entries in array A | A sum 535 |
| +0x06 | s16 | B, count of entries in array B | B sum 2,507 |
| +0x08 | s16 | C, count of entries in array C | C sum 223 |
| +0x0A | u16 | fill | 0xCDCD in all 2,449 records; not used |
| +0x0C | u32 | fill | 0xCDCDCDCD in all 2,449 records; not used |
| +0x10, +0x14, +0x18 | u32 each | pointers to arrays A, B and C | zero on disc; the loader writes run-time values here. Do not read them from the disc. |
| +0x1C | u32 | unused | zero on disc |

Arrays follow the header in the order A, B, C. Each entry is 0x50 bytes:

| Offset | Size | Field | Measured on disc |
| --- | --- | --- | --- |
| +0x00 | 4 × f32 | position x, y, z, w | w = 1 in all 3,265 entries |
| +0x10 | 3 × f32, then 0 | unit direction, w = 0 | length 1.0 in all 3,265 entries |
| +0x20 | 4 × f32 | colour R, G, B (0 to 255) and alpha | alpha 255 and channels 0 to 255 in all 3,265 entries |
| +0x30 | s32 | light type | 2 for array A, 3 for array B, 1 for array C, in all 3,265 entries. The array decides the type, so readers should use the array. Roles (Medium-High): type 1 is the ambient term (its direction is not used); type 2 is directional and uses the entry direction (+0x10); type 3 is a point light whose direction is normalize(object position − light position), with the falloff of SPEC-0046 (Character lights); its stored direction is not used. |
| +0x34 | f32 | strength | A 0.0 to 1.0; B 0.2 to 12.232; C 0.0 to 1.1 |
| +0x38 | s32 | flag | A: 0 in 354 entries, 1 in 181. B and C: 0 in all. The first A entry of every record that holds A and C entries is 1. |
| +0x3C | f32 | radius | A: −1.0 (445 entries) or 0.0 (90). B: 0.625 to 117.803 (2,506 entries) and 0.0 (1 entry). C: −1.0 or 0.0 (181), or 15.025 to 34.617 (42). The values −1.0 and 0.0 mean no radius. |
| +0x40 | f32 | radius squared | the square of the radius when the radius is above zero; 0.0 when the radius is 0.0; −1.0 when the radius is −1.0 |
| +0x44 | s32 | zone key | only the low byte is used: 0 to 20 in arrays A and C, 0 to 44 in array B. Bytes +0x45 to +0x47 are zero in all entries. |
| +0x48 | 8 bytes | fill | 0xCD in all 3,265 entries; not used |

Which record kinds occur: records with only B entries (2,226), B and C (40), A and C (181), C only (2). No record has both A and B entries.

The one array B (type 3) entry with radius 0.0 is in pack 1174, record 44: gate 0, zone key 3, colour (255, 119, 9), strength 0.65. It has no radius gate, and in the original it breaks the cross-fade of SPEC-0046 (a division by zero); the clean side treats it as absent (SPEC-0046, Character lights and Edge cases).

Counts on disc: 140 packs, 2,449 records, 3,265 entries (A 535, B 2,507, C 223). Each record holds at most one array C entry on disc. Character lighting reads only the first array C entry of a record (SPEC-0046, Character lights). Light sets reach characters only: no world-geometry or placed-object reader exists, and world geometry is unlit (High).

### 3. Resource 527: directional light text

The resource is 2,048 bytes. It starts with seven ASCII text lines, followed by bytes left over from an earlier sector. The game never reads this resource (High), so a clean implementation does not need it. Its lines describe the three directional lights of the scene's default lights, which the program builds from fixed constants (SPEC-0046, Scene default lights; the lit-vertex arithmetic is in SPEC-0004). The default lights are four lights: one ambient light, which exists only in the program (colour (50, 50, 50)), and these three directional lights. The line forms are kept here only to identify the file.

The seven lines are, in order: direction0, direction1, direction2, color0, color1, color2 and ambient. Each line is its key, then its values separated by single spaces; every value is written as a decimal number with six digits after the point. Lines 1 to 6 end in CR LF; the seventh (ambient) line has no line end.

| Line key | Values | Meaning (High) |
| --- | --- | --- |
| direction0, direction1, direction2 | three decimal numbers each | travel direction of directional light i (pointing down); the lit path uses its negation. Length 1.0 within 6e-4 on disc. The same three directions are the built-in directional lights (SPEC-0046). |
| color0, color1, color2 | four decimal numbers each: three whole numbers 0 to 255, then one fraction | RGB colour of directional light i (the same colours as the built-in lights), then its strength: 0.25, 0.25 and 0.20 in the text, while the built-in lights use 0.25 for all three. |
| ambient | four decimal numbers | On disc all four are the same value, −107374176.000000. That is the float interpretation of four 0xCC fill bytes, not a colour. The file sets no ambient colour; the built-in ambient (50, 50, 50) exists only in the program (SPEC-0046). |

The ambient line ends at byte 350 of the resource. Byte 351 is a stray byte and byte 352 is the first zero byte. Readers must stop at the end of the seventh line and must not parse the stray byte.

### 4. Type 39: placed effect spawns (158 packs, 5,139 records)

Payload: u32 count at +0x00 (1 to 274 per chunk). Records start at +0x10 and are 0x90 bytes each. Every chunk fits its payload.

Record (0x90 bytes):

| Offset | Size | Field | Measured on disc |
| --- | --- | --- | --- |
| +0x00 | 4 × f32 | column 0 of a 4 × 4 transform (column-major) | columns 0 to 2 form a rotation: unit length and mutually orthogonal in all 5,139 records (1e-3) |
| +0x10 | 4 × f32 | column 1 of the transform | as above |
| +0x20 | 4 × f32 | column 2 of the transform: unit direction (forward axis) | unit length in all 5,139; the 4th value is 0 |
| +0x30 | 4 × f32 | column 3: position, w = 1 | w = 1 in all 5,139 |
| +0x3C | f32 | 1.0 | all 5,139 |
| +0x40 | s32 | effect id | 116 distinct ids, 101 to 4042 |
| +0x44 | u32 | zone number: the type-35 column of the placed effect (SPEC-0006, Binary grids); only the low byte is used | 0 to 59 on disc: 0 in 1,926, 1 in 544, 3 in 481, 2 in 405, 4 to 59 in the rest; below block 0's width in 4,559 of 4,559 records in packs with a grid |
| +0x48 | u32 | 0 | all 5,139 |
| +0x50 | f32 | threshold for near or far, stored as a squared distance | 10000.0 in all 5,139 (100 squared) |
| +0x54 | u32 | child count | 0 in all 5,139 |
| +0x84 | f32 | squared cull distance | −2.0 (no cull) in 4,869 records. Others: 225 (15 squared), 2,500, 5,625, 22,500, 40,000, 90,000, 1,000,000. |
| +0x88 | u8 | cone-test enable | 1 in all 5,139 |

Bytes +0x89 to +0x8B are a fill pattern and carry no data. Bytes +0x58 to +0x7B are zero on disc and are filled by the game at run time.

**Pack 951 fireplace (R-0081).** The farmhouse interior has ten type-39 records: records 0–2 are effect 108 at the fireplace, and records 3–9 are effect 103. Their source active words (+0x48) are zero, but the world-load bind copies the matching definition over that word. Definitions 108 and 103 both have it set, so records 0–9 become active when pack 951 is loaded. On the first applicable zone pass, visible records update; records outside a visible zone are stopped.

Records are placed effect instances. The loader binds each to a definition by id, copying the definition's active word (+0x48) along with its other fields, then restores the record's placement fields. Every definition on disc has this word set, so every valid placed record becomes active at load; the source records have zero in that word. Scripts can start, stop and move a record by its index in the chunk; a switch id in a type 40 list can also set the flag (section 9).

### 5. Type 1000: dialogue line table (131 packs)

Header:

| Offset | Size | Field | Measured on disc |
| --- | --- | --- | --- |
| +0x00 | 4 bytes | tag "IDMX" | 131 of 131 |
| +0x0C | u16 | entry count | 1 to 1,043. The value 24 occurs in five packs: 678, 686, 691, 855 and 984. |
| +0x0E | u16 | entry stride | 12 in all 131 |

Entries start at +0x10 and are 12 bytes each:

| Offset | Size | Field | Measured on disc |
| --- | --- | --- | --- |
| +0x00 | u32 | speaker-name pointer (payload-relative); 0 means none | non-zero for all but a few; 7,482 of 7,547 resolve to readable names |
| +0x04 | u32 | dialogue-text pointer (payload-relative); 0 means none | points to text with embedded control bytes in most records |
| +0x08 | 3 bytes | colour R, G, B | 11 distinct triples; (0, 0, 0) in 7,019 of 7,547 |
| +0x0B | 1 byte | alpha half of the colour word: 0x80 in the 528 entries whose RGB is not (0, 0, 0), 0 in the other 7,019. Never read by any code (the text box reads only bytes +0x08 to +0x0A) | |

The loader takes the entry count from +0x0C and the record array from +0x10; it does not use payload words +0x04 or +0x08. The known line-display path reads the three colour bytes at entry +0x08 through +0x0A; entry byte +0x0B does not affect that colour. The original authoring purpose of the unused header words and fourth entry byte is unknown, and no other consumer has been established. The text format and its control bytes belong to the dialogue spec (SPEC-0022, not yet in the outbox). An index outside 0 to count minus 1 yields no entry.

### 6. Type 11: object header and collision primitives (189 chunks)

Header (payload offsets). Counts are s32; offsets are u32 and payload-relative.

| Offset | Size | Field | Measured on disc |
| --- | --- | --- | --- |
| +0x00 | s32 | trigger count: the size of the script manager's trigger pool (pool A, section 15) | not counted |
| +0x08 | s32 | count of sphere records (0x40 bytes each) | positive in 131 chunks. The count equals the number of 6.2 records in all 189 chunks (6,678 in total). |
| +0x0C | s32 | count of cylinder records (0x50 bytes each) | positive in 175 chunks; the header counts sum to 10,038. Under the established on-disc layout, 10,028 complete cylinder shape records fit their chunks. Resource 721 declares 10, but only 9 complete records and 48 bytes of the tenth fit in its payload; the declared array extends 32 bytes past the chunk end. |
| +0x10 | s32 | count of cone records (0x40 bytes each) | positive in 7 chunks. The count equals the 6.3 count in all 7 packs (83 cones in total). |
| +0x14 | s32 | count of kind-7 rectangle records (0x70 bytes each; list pointer +0x3C) | positive in 142 chunks; 675 records in all |
| +0x18 | s32 | count of kind-4 rectangle records (0x70 bytes each; list pointer +0x40) | positive in 164 chunks; 5,849 records in all; pack 678 = 12, pack 684 = 100 |
| +0x1C | s32 | n1: count of 0x24-byte entries at the table at +0x44 (movable-block sources) | positive in 17 chunks |
| +0x20 | s32 | n2: count of 0x0C-byte entries at the table at +0x48 (rope-gun aim targets) | positive in 19 chunks |
| +0x24 | s32 | count of entry (locator) records (0x50 bytes each) at the table at +0x4C | positive in 179 chunks, exactly the chunks whose +0x4C pointer holds a kind-1 header; 4,964 records in all |
| +0x28 | s32 | n3: count of 0x1C-byte records at the table at +0x50 (empty on disc) | 0 in all 189 |
| +0x2C | s32 | n4: count of 0x50-byte records at the table at +0x54 (kind-8 ambient sound loops, section 6.5) | positive in 139 chunks |
| +0x30, +0x38, +0x3C, +0x40, +0x44, +0x48, +0x4C, +0x50, +0x54 | u32 | payload-relative offsets to further tables; rewritten at load | all inside the chunk in 189 of 189 |

Notes on the offsets:

- The table at +0x44 holds the n1 entries, +0x48 the n2 entries and +0x50 the n3 records; +0x54 holds the n4 records. These four pairings match the counts above.
- Record rule (High): each loaded primitive has a 0x20-byte runtime header followed by its shape data. In the type-11 payload, the primitive-list pointer is 0x20 bytes before the first shape record; on-disc record i's shape data starts at pointer + 0x20 + i times the record size. The record sizes are 0x40 (spheres and cones), 0x50 (cylinders and the fourth list) and 0x70 (rectangles). The kind byte is supplied in the loaded header from list membership; the all-ones body tag begins each on-disc shape record. Pack 678 has its sphere pointer at 0x100, so its two sphere shape records start at 0x120 and 0x160. Pack 684 has its cylinder pointer at 0x60, so its first cylinder shape record starts at 0x80.
- The offsets at +0x30, +0x34, +0x38, +0x3C, +0x40 and +0x54 are the list pointers of sections 6.1 to 6.5. The kind-4 rectangles (pointer +0x40, count +0x18) and the kind-7 rectangles (pointer +0x3C, count +0x14) are two lists of the same 0x70-byte record. In 131 of the 142 chunks with kind-7 records the kind-7 list directly follows the kind-4 list (+0x3C = +0x40 plus 0x70 times the kind-4 count); in the other 11 the kind-4 count is 0. Packs 678 and 684: 678 has 12 kind-4 records from +0x1D0 and no kind-7 record; 684 has 100 kind-4 records from +0x380 and one kind-7 record at +0x2F40.
- The offset at +0x4C is the entry (locator) list of SPEC-0006: its count is the s32 at +0x24, and its records are 0x50 bytes, each with a kind-1 header (kind 1 in all 4,964 records; flags 0x0020 in 4,924 and 0x0021 in 40). The 179 chunks with a positive count are exactly those whose pointer holds a kind-1 header; in the other 10 the count is 0 and the kind byte at the pointer is 0. Pack 1432 has 5 entry records with flags 0x0020; the first has a body with three equal extents of 0.625 (section 13), and its entry 2 lies on the trigger-1 cylinder (section 13). In pack 684 the list starts at +0x2FB0, directly after the kind-7 record. High.

Extra lists (High):
- **n1 entries (0x24 bytes): movable-block sources.** +0x10 step value, +0x18 registry model id (−1 on disc), +0x1C inline mesh, +0x20 the block's primitive (a kind-4 rectangle record). They are the disc source of the movable blocks of SPEC-0048. An entry becomes a block only when its primitive's flags have bit 1 (value 0x0002) set (SPEC-0006, Type 11 movable-block table).
- **n2 entries (0x0C bytes): rope-gun aim targets.** +0x00 registry model id (used when +0x04 is 0), +0x04 inline marker mesh, +0x08 a locator-like record whose +0x30 is the target point. At load they become the world's aim-target list (world +0x5824, count +0x5820) of SPEC-0030 row 126, which the hero's rope-gun aim scan reads (SPEC-0033).
- **n3 records (0x1C bytes):** none on disc (the count is 0 in all 189 chunks).
- **n4 records (0x50 bytes, list pointer +0x54, count +0x2C):** the fourth list of section 6.5, the kind-8 ambient sound loops and adventure zones of SPEC-0031. The six-list walk of section 6.6 does not use it.

Pointers at +0x30 to +0x54 are rewritten at load. The load-time clear of byte +0x4F in each 0x50-byte record of the table at +0x54 is in section 0 (High).

**Record header (0x20 bytes, at the start of every primitive record; High for the positions, roles as noted).**

| Offset | Size | Field | Measured on disc |
| --- | --- | --- | --- |
| +0x00 | 16 bytes | unused padding (no reader known) | zero in every sphere, cone, cylinder and fourth-list record; the rewrite may store zeros or omit it |
| +0x10 | u8 | volume kind (section 13): 1 sphere, 2 cone, 3 cylinder, 4 or 7 rectangle, 8 fourth list | fixed per list (record rule above) |
| +0x11 | u8 | mode byte (blade recoil material input, section 13) | varies |
| +0x12 | u16 | flags (section 13) | varies per record (not per list); on a kind-7 rectangle, 0x0002 with a class other than 500 and 501 marks a door |
| +0x14 | u32 | serial number | cylinders of pack 683: 3 to 6 (Medium for the width and for other lists) |
| +0x18 | u16 | index of the record within its list | cylinders of pack 683: 0 to 3 (Medium for the width and for other lists) |
| +0x1A | u16 | gate mask (section 13) | nonzero in 44 records (8 spheres, 9 cylinders, 23 kind-4 and 4 kind-7 rectangles); 0 on every door |
| +0x1C | u16 | number: the script trigger index; on a door, the door number (rewritten at load, section 13) | doors 0 to 10 |
| +0x1E | u16 | class selector or book value | book values on the 134 book-station cylinders (section 13); otherwise nonzero only in 14 sphere and 12 rectangle records |

Class selector: the values 500 and 501 form one class, the same-world pads. The class list is built at load from the kind-7 list (+0x3C) only: a kind-7 record with bit 1 set and selector 500 or 501 gets, as its trigger index (+0x1C), the running count of such class records so far. On the disc, selectors 500 and 501 occur on two kind-7 records of pack 1299 (records 2 and 3, flags 0x0002), which get indices 0 and 1, and 500 on one sphere record of pack 1246, which is never read (the class step reads only the kind-7 list). The two pack 1299 records are not doors: the static rectangle handler sends a bit-1 record of class 500 or 501, and only for the selected hero, to the same-world pad branch (Behavior, Pack 1299 looping-door puzzle) before and instead of the door branch. The pad pair is built at load only when a world word is nonzero; the display-tree load stores the node with id 401 in that word, so the pair needs a display node 401 (Medium). Only pack 1299 qualifies, and there the pads drive the looping-door puzzle (Behavior). High for the branch order; Medium for the pad pair.

### 6.1 Rectangle records (0x70 bytes): kind-4 list (+0x40) and kind-7 list (+0x3C), High

Both rectangle lists use the same record. Record layout (offsets from the record start, which is the start of its header):

| Offset | Size | Field | Measured on disc |
| --- | --- | --- | --- |
| +0x00 | 0x20 bytes | record header (section 6): kind 4 or 7 at +0x10 | every record of both lists |
| +0x20 | u32 | tag, all bits set (0xFFFFFFFF) | all 6,524 rectangle records |
| +0x24 | f32 | extent along the axis at +0x60 (vertical in the sampled boxes) | signed (see Extent signs below); one extent of 0.0 in the 300 extents of pack 684 |
| +0x28 | f32 | extent along the axis at +0x50 | as above |
| +0x2C | f32 | extent along the axis at +0x40 | as above |
| +0x30 | 4 × f32 | reference point x, y, z; w = 1 | sampled, w = 1 |
| +0x40 | 4 × f32 | unit axis (the B axis for the extent at +0x2C): x, y, z; w = −(axis · reference point) | unit length within 1e-7 over 112 sampled records; w agrees within 6e-5 |
| +0x50 | 4 × f32 | unit axis (the A axis for the extent at +0x28); w as above | as above |
| +0x60 | 4 × f32 | unit axis (the axis for the extent at +0x24); w as above | as above; (0, 1, 0) in the sampled boxes of pack 684 |

The three axes are mutually perpendicular (dot products zero to float precision in all 112 sampled records). The w component of each axis is the offset of its plane through the reference point. These are the run-time rectangle fields of section 13 (extents at +0x24 and +0x28, the switch or third extent at +0x2C, normal at +0x40, in-plane axes at +0x50 and +0x60). The static kind-4/7 rectangle test uses the +0x40 vector as its plane normal and the +0x50 and +0x60 vectors as the in-plane axes; their tested bounds use the extents at +0x28 and +0x24, respectively. This is an operational field mapping, without an editor-axis interpretation. The rectangle volumes of the second-list breakables (SPEC-0048; 115 on disc) are read the same way: +0x30 is a corner and the extents are full non-negative lengths from it, which puts the box centre at the corner plus half of each extent along its axis (confirmed against the inline meshes of packs 758 and 989).

The reference point is the box's low corner in the sense of SPEC-0018 (Object collision). The containment bounds in SPEC-0018 use the extents as full lengths from the reference point, so this spec does not use the word "half-extent" for them. Medium-High.

Extent signs. The 112 sampled boxes all have positive extents, but this is not general. Over all 5,849 kind-4 records the extent at +0x24 is negative in 3 records (minimum -27.35), the extent at +0x28 in 59 (minimum -10.07) and the extent at +0x2C in 70 (minimum -118.6). No kind-7 record has a negative extent; the extent at +0x2C is 0.0 or above in all 675 (the static switch value of section 13). Runtime box contact uses the stored signed extents directly: it does not replace them with their absolute values. For either in-plane axis, a radius-r query is accepted only when its coordinate is strictly between -r and extent + r. Thus a negative extent narrows that interval and can leave a small overlap when the query radius is large enough; with zero radius, a negative in-plane extent admits no coordinate. For example, extent -2 with radius 1.25 admits coordinate -1 on that axis, while extent -2 with radius 1.0 admits none. The vertical extent uses the same signed value with the separate 0.1 top offset and optional top growth described in SPEC-0018. This establishes runtime use, not why the authoring data contains negative values. Do not assume that an extent is positive.

First kind-4 record of pack 684 (record start +0x380): extents 75.2, 190.2 and 11.0; reference (1440.8, 59.6, 726.4).

The records of one rectangle list are stored one after another. Pack 684 holds 100 kind-4 records from +0x380, then its one kind-7 record at +0x2F40, then the entry list (+0x4C) from +0x2FB0. In all 101 rectangle records of pack 684 the shape tests pass. In pack 678 the 12 kind-4 records start at +0x1D0 and pass the same tests.

Synthetic example (a layout example, not disc data): a box with reference point (10, 0, −5), extents 6 (along the axis at +0x60), 4 (along the axis at +0x50) and 2 (along the axis at +0x40), with axes (0, 1, 0) at +0x60, (1, 0, 0) at +0x50 and (0, 0, 1) at +0x40. Its w values are 0 at +0x60 (−(0 + 0 + 0) = 0), −10 at +0x50 and 5 at +0x40.

### 6.2 Sphere records (0x40 bytes): High for the layout and count

Sphere records are the records of the sphere list (list pointer +0x30, count +0x08): 0x40 bytes, a tag of all ones and three equal extents. The list is read by its count, not by shape. The number of records in the list equals the sphere count at +0x08 in all 189 chunks (6,678 in total). The size question (0x40 or 0x50) is resolved: the 0x40-byte records are the spheres.

| Offset | Size | Field | Measured on disc |
| --- | --- | --- | --- |
| +0x00 | 0x20 bytes | record header (section 6): kind 1 | every counted record |
| +0x20 | u32 | tag, all bits set | every counted record |
| +0x24 | 3 × f32 | radius, repeated three times (equal values) | all counted records; the three values are equal |
| +0x30 | 3 × f32 | centre x, y, z | varies per record |
| +0x3C | f32 | radius (the w of the centre vector) | equal to the radius above in all counted records |

Pack 678 holds two sphere records, starting at +0x100 and +0x140, with radii about 14.44 and 19.11. Its two 0x50-byte records with radii 65 and 75 are not spheres (see 6.5).

### 6.3 Cone records (0x40 bytes): High for the count, Medium for the field meanings

Cone records are the records of the cone list (list pointer +0x34, count +0x10): 0x40 bytes with a tag of all ones. They are classified by list membership, not by shape. The list count equals the cone count at +0x10 in all 7 chunks that contain cones (83 records in total).

| Offset | Size | Field | Measured on disc |
| --- | --- | --- | --- |
| +0x00 | 0x20 bytes | record header (section 6): kind 2, flags 0 in all 83 | every counted record |
| +0x20 | u32 | tag, all bits set | every counted record |
| +0x24 | f32 | height H along the vertical axis | positive in all 83 (High for the value; Medium for the use) |
| +0x28, +0x2C | 2 × f32 | equal pair: base radius R | every counted record |
| +0x30 | 3 × f32 | base point x, y, z (the reference point; its y component is the base height) | varies |
| +0x3C | f32 | radius copy, equal to the radius pair | every counted record |

The body holds no axis: the cone axis is vertical. The 16 bytes after each cone body are zero in all 83; where another record follows, they are the first 16 bytes of its header.

The cone formula in SPEC-0018 (object collision) uses this record: height H, base radius R, and the base point with its base height. The weapon-contact pass does not reach the cone routine; the cell list walks route kind 2 to the cone handler (open question 76, resolved). Use the count, not the chunk length, to find cones.

### 6.4 Cylinder records (0x50 bytes): Medium-High

Cylinder records are the records of the cylinder list (list pointer +0x38, count +0x0C): 0x50 bytes, a 0x20-byte header with kind 3, then a body with a tag of all ones and a radius copy at +0x3C. They are classified by list membership, not by shape: 96 cylinders have a length equal to their radius.

Counts: the header count is positive in 175 chunks and sums to 10,038 declared entries. On-disc cylinder shape records begin 0x20 bytes after the list pointer and advance by 0x50 bytes. Of the declared entries, 10,028 complete shape records fit within their chunk payloads; resource 721 declares 10, but its payload holds 9 complete records plus 48 bytes of the tenth, leaving a 32-byte shortfall. Packs 1176 and 1245 are not short: all 11 and all 85 records fit. In each pack, one valid record has a length equal to its radius (0.625), which caused the earlier shape-based counter to miss it.

| Offset | Size | Field | Measured on disc |
| --- | --- | --- | --- |
| +0x00 | 0x20 bytes | loaded record header (section 6): kind 3; flags, number and book value per record (section 13) | 10,038 declared entries, assigned from the cylinder list |
| +0x20 | u32 | tag, all bits set | records counted in the packs above |
| +0x24 | f32 | full length along the axis at +0x40 (one value is negative, see the edge cases) | 12.242 in all 10 records of pack 684 |
| +0x28 | f32 | radius | 2.579 in all 10 records of pack 684 |
| +0x2C | f32 | radius (same value) | 2.579 in all 10 records of pack 684 |
| +0x30 | 3 × f32 | base point x, y, z: the base, not the centre (the centre is the base plus half the full length along the axis) | varies per record |
| +0x3C | f32 | copy of the radius | 2.579 in all 10 records of pack 684 |
| +0x40 | 4 × f32 | unit axis x, y, z; w = 0 | unit length and w = 0 in all 10,028 complete on-disc shape records (tolerance 1e-4); (0, 1, 0) in all 10 records of pack 684 |

In pack 684 the cylinder-list pointer is +0x60; the 10 on-disc shape records start at +0x80 and advance by 0x50. The radius at +0x28 equals +0x2C and +0x3C in all 10,028 complete on-disc shape records. The axis is unit length, so the length at +0x24 is not a radius except where the two values happen to be equal. These are the run-time kind 3 fields of section 13. The cylinder volumes of the second-list breakables (SPEC-0048) use this same record and the same reading: +0x30 is the base, and the length at +0x24 runs one way along the axis from it (confirmed on 1,915 breakable cylinders, including pack 732).

Run-time form (High): each cylinder is 0x50 bytes, its own 0x20-byte header (kind +0x10, flags +0x12, serial +0x14, index in the list +0x18, trigger index +0x1C, book value +0x1E) followed by the 0x30 shape bytes (length +0x24, radius pair +0x28 and +0x2C, base +0x30, radius +0x3C, axis +0x40). All 9,947 references to cylinders in the grid cell lists (cell list A, SPEC-0018) point at a header start. Flags vary per cylinder, not per list (survey in section 13). When bit 3, 6 or 8 of the flags is set, a character overlap tests the cylinder against the character's body cylinder (radius 1.0, from the feet to 3.5 above, along the body's up vector); with only bit 0 set, the character's own radius and length are used instead (SPEC-0018). High.

### 6.5 Other 0x50-byte records (the fourth header list): layout High, role High

The fourth header list (list pointer +0x54, count +0x2C) declares 2,589 records; 2,588 complete records fit within their chunks. The final record in resource 720 extends 32 bytes beyond its payload, and the game's read bound for it is not encoded (class D; Default if unresolved (rewrite choice): skip the one record that does not fit inside its payload). Pack 678 has 2 records, pack 1245 has 25, pack 1246 has 24 and pack 1176 has 4. Each record is 0x50 bytes: a 0x20-byte header (kind 8, flags 0x0004 in all 2,589) and a body with a tag of all ones at +0x20, three equal extents E at +0x24, a centre at +0x30, and a copy of E at +0x3C. E is 0 in 2 records and positive in the rest. Fields from +0x40 on are: a first value that is not a unit component, a value of 1.0 at +0x44 (2,545 records), a small integer at +0x48 and a u16 pair at +0x4C. The records are the ambient sound loops and adventure zones of SPEC-0031, which owns their field meanings; byte +0x4F is their start-delay counter, which the loader clears (section 0). The six-list walk of section 6.6 does not use this list, and no collision path in SPEC-0018 uses it.

### 6.6 Record order

Classify each record by the header list whose pointer and count place it, not by its position or shape. Across all 189 chunks, the nonempty primitive arrays are laid out in pointer order as spheres, cones, cylinders, kind-4 rectangles, then kind-7 rectangles, with absent arrays omitted (High; 183 chunks contain at least one primitive array). Every record header kind agrees with the list that points to it. The auxiliary type-11 arrays can interleave with this primitive sequence, so this is not a physical-order rule for every type-11 array.

Pack 684 is one example: cylinders begin at +0x60, kind-4 rectangles at +0x380, the kind-7 rectangle at +0x2F40, and the entry list (+0x4C) at +0x2FB0. Pack 1176 shows auxiliary interleaving: its N4 list starts at +0x60, followed by cones at +0x1A0, cylinders at +0x320, kind-4 rectangles at +0x690, kind-7 at +0x1260, and then N1 and entry lists. [High for counted primitive arrays; auxiliary array order varies]

**Walk order of an owner's primitives (Medium-High).** A loaded owner walks its primitives in six lists, in this order: a list of 0x70-byte records, a list of 0x40-byte records (spheres or cones), a list of 0x50-byte records (cylinders), a sub-list of 0x24-byte records, a second list of 0x70-byte records, and a list of 0x40-byte records (spheres or cones). Every record of each list is dispatched in that order. The dispatch behaviour is in SPEC-0018. The mapping from these runtime lists to the header counts in section 6 is not established (Open question 41). The disc has two lists of 0x70-byte records, the kind-7 list (+0x3C, count +0x14) and the kind-4 list (+0x40, count +0x18); which of them is walk list 0 and which is walk list 4 is part of that question. The fourth header list (section 6.5) is not one of the six lists.

### 7. Type 29: placed objects and their part records (121 packs, 3,495 objects, 642 with parts)

Object table: count N at +0x00 and table offset at +0x04 give the objects, which are payload-relative offsets. Second list: count at +0x08 and table offset at +0x0C (payload-relative). Each second-list table entry is a u32 offset measured from the start of the offset table, not from the payload start (High; the relocation at load uses this base). Both fit in every chunk. The loader reads the type-10 table when it sets these objects up, so type 10 comes earlier in the chain (section 0). The object fields are below, with offsets **in object**:

| Offset in object | Size | Field | Measured on disc |
| --- | --- | --- | --- |
| 0x100 | 4 × 4 floats | transform block used to draw the inline mesh | |
| 0x1A0 | s32 | object state | 3 in 3,491 objects on disc, 7 in 4; written by the script calls of section 7.1; see the draw gate |
| 0x1A4 | 4 bytes | colour: three channels, then a fourth byte | see the draw gate |
| 0x1B4 | s16 | instance index | −1 in all 3,495 objects on disc: the index is not used and no object links to an instanced mesh (type 20) through it |
| 0x1B6 | byte | draw-test result (written by the draw gate) | reset value 0 |
| 0x1B7 | byte | draw-gate byte, read as stored | 1 in 3,489 objects; 0 in 6 (pack 1298, objects 1 to 6) |
| 0x1B8 | byte | zone column for the binary-grid query (SPEC-0006), read as stored | 0 in 2,774 objects and 1 to 25 in the other 721 (values 1: 120, 2: 41, 3: 99, 4: 101, 5: 41, 6: 32, 7: 19, 8: 34, 9: 11, 10: 4, 11: 98, 12: 5, 13: 5, 15: 3, 16: 2, 17: 4, 21: 2, 25: 100) |
| 0x1D0 | u32 | inline mesh chunk (type 12 or 13), payload-relative offset | 3,305 objects; all type 12 |
| 0x1D4 | u32 | inline model set (type 5), payload-relative offset | 650 objects have a non-zero pointer, and all 650 point at a type-5 chunk; 642 of them have parts and 8 have a model set but no parts. The object tables also hold 29 zero entries, which are not objects. |
| 0x1D8 | u32 | inline type-11 header, payload-relative offset | 357 objects; all type 11 |
| 0x1DC | s32 | part count | total 7,201 |
| 0x1E0 | u32 | part array of 0x70-byte records, payload-relative offset | 642 objects; all fit |
| 0x1BC | s32 | number of type-10 ladder or climb-panel groups the object owns (section 7.2) | 0 in all objects but three: pack 777 object 12, pack 800 object 0 and pack 1098 object 6 (1 each); High |
| 0x1C0 | u32 | payload-relative offset of that many u32 indices into the pack's type-10 group table | index 0 (777 / 12), 3 (800 / 0), 0 (1098 / 6); no group has two owners; High |
| 0x1C8 | u32 | payload-relative offset of an array of group pointers; the array is filled at load with the addresses of the listed type-10 groups | disc bytes are filler; High |
| 0x1CC | u32 | payload-relative offset of the object's own copy of each listed group (0x20-byte group header, then its 0x60-byte records in the object's rest frame, section 7.2) | each copy equals the table group in the header; High |
| 0xC0 to 0xFF | 4 × 4 floats | second transform block: columns at 0xC0, 0xD0, 0xE0 and 0xF0 (the last one a translation). Read by the ground probe to map a query point into the object's frame (SPEC-0018, Ground probe, step 4). It is the rest block: setup builds the current block from it (section 7.1), and the moving-object walks of SPEC-0018 map into its frame. | High for the layout and the rest role |

The inline type 5 chunk's type field at +4 must be 5. Its payload starts 16 bytes into its chunk. Its count is at +0x04 and the offset of its entry table at +0x08 (both relative to its payload). Each table entry is a payload-relative offset to a type 5 entry, which holds: +0x20 count A, +0x24 count B, +0x50 count C, and +0x54 the offset of an array of C records of 0x70 bytes. The entry kinds, the full entry header and the 0x70 record layout are in SPEC-0018 (Model-set entries, chunk type 5).

Part record (0x70 bytes, in the part array; High):

| Offset | Size | Field | Measured on disc |
| --- | --- | --- | --- |
| +0x00 to +0x3F | 4 × 4 f32 | copy of the bound model-set frame record's start point and rows e0, e1 and e2 (SPEC-0018, Model-set entries, frame record) | equal to the bound record in 7,201 of 7,201 |
| +0x40 | f32 | run length L, a copy of the bound frame record's +0x40 | 0 to about 1,451; equal to the bound record in 7,201 of 7,201 |
| +0x60 | pointer | model-set record it is bound to, set at setup | |
| +0x64 | s32 | index into the type 5 entry table | in range for all 7,201 |
| +0x68 | s32 | index of the record inside that entry's record array | less than the entry's count for all 7,201 |

Parts at run time (High; Medium-High for the hide path):
- Setup binds each part to its frame record (part +0x60) and writes into **the bound record**, not the part: its run length (+0x40) becomes −200.0, and its self pointer (+0x48) and owner pointer (+0x64) are set. The −200.0 is not stored on disc.
- Each object update maps the record's start from the object's rest frame (section 7.1) to its current frame, and sets record L = part L and record +0x68 = e1.z × L.
- Hiding the object writes −200.0 back into the bound record's run length, so the object's walls are off while it is hidden.

So a moving placed object's model-set walls follow the object, and the part keeps the rest-frame copy.

Second-list entries are the breakables of SPEC-0048 (layout there). Their volumes are the only collision of the objects a type-40 chest group draws: the 146 kind-1 groups (lock chests) list placed objects with no model set and no inline type 11 header, and each group's breakable 0 cylinder is what stops characters (section 13, Cell list B walk; SPEC-0048, Lock chests). The 39 kind-2 groups (element-sealed chests) give object 0 an inline header with one cylinder as well. Their drop ids are plain fields of the disc entry, set by no record or script (High): the drop placement ordinal (+0x56) is 0 or more in 1,239 of the 2,073 entries on disc, and the character id (+0x54) is 0 or more in 20 (for example 1960 in pack 732). Also on disc: the break float (+0x5A) is above 0 in 99 entries, the piece count (+0x4C) is above 0 in 2,029, and the volume sub-record (+0x60) is present in 2,030. Type-40 script groups carry the same (placement, character) pair in their own drop list (section 11).

### 7.1 Animated placed objects (recorded motion; High unless stated)

The game's moving platforms, lifts, sliding and rotating bridges, moving doors and gates are type 29 placed objects with a recorded single-transform motion. Level scripts start and stop the motion. Characters ride them through the ground attach of SPEC-0018 (Ground attach of a character); objects with inline primitives push characters through the push at the tail of the character ground-contact step (SPEC-0033, Behavior 43; the object is never moved, only the character); projectiles are tested against them in SPEC-0018 (Projectile versus placed objects).

Fields (offsets in object):

| Offset | Size | Field |
| --- | --- | --- |
| +0x80 | 4 × f32 | root offset (run time) |
| +0x90 | 4 × f32 | script position offset |
| +0xA0 | 4 × f32 | extra rotation quaternion (set by a script call, with the mode byte) |
| +0xB0 | pointer | locator record (run time) |
| +0xB4 | pointer | animated locator (run time) |
| +0xB8 | f32 | play speed (signed) |
| +0xBC | f32 | bounding radius (setup) |
| +0x100 to +0x13F | 4 × 4 f32 | current block: columns X, Y, Z, then the translation at +0x130 |
| +0x140 | 4 × f32 | scale; (1, 1, 1) in 289 of 343 sampled objects |
| +0x150 | 4 × f32 | translation change of the last update (w = 0) |
| +0x160 | 4 × f32 | current rotation quaternion |
| +0x180 | 4 × f32 | reference centre |
| +0x1A0 | s32 | state |
| +0x1B0 | f32 | time in seconds |
| +0x1B9 | u8 | offset and rotation mode byte: 1 adds the +0x90 offset, 2 adds the +0xA0 rotation, 3 both |
| +0x1C4 | 4 bytes | advance of the last tick, in keys |
| +0x1E4 | u16 | key count F |
| +0x1E8 | u32 | motion stream, payload-relative offset on disc (relocated at load) |

Disc: state 3 in 3,491 objects (7 in 4), speed 0 and time 0 in all. So no object moves or draws until a script calls InitObjOn. Survey over all 192 world packs: 3,495 objects decode a stream; 2,798 move (translation span above 0.01, or rotating); 430 moving objects carry height patches (they carry riders); 252 moving objects carry inline primitives (they push: doors, gates, chests); 108 moving objects with height patches, in 32 packs, are named by a fixed index in a PlayAnim or LoopAnim call of their pack's script. 31 of the 32 packs (107 objects) are in the set of 174 packs the player can enter (SPEC-0006, Reach; the count ignores story-flag gates); the remaining pack is 1077, an unused copy of 1129 with one such object.

**Stream.** Ten channels in order: rotation x, y, z, w; translation x, y, z; scale x, y, z. Each channel starts with an s16 flag. Flag 0: one s16 follows (a constant). Any other flag: F s16 samples follow, one per key. Value = s16 / 1024. Keys run at 15 per second. Rotation and scale samples are absolute per key. Translation samples are per-key changes: the offset at key k is the sum of samples 0 to k (sample 0 is 0 in every object checked).

**Setup.** Current block = rest block (+0xC0 to +0xF0, section 7) with its columns scaled by +0x140. Reference centre +0x180 = rest translation + mesh corner (mesh child +0x10) + half the mesh size (mesh child +0x20). The corner is added without rotation, so the centre of a rotated object is off (reproduce it). Radius +0xBC = max(50.0, |size.x|, |size.y|, |size.z|); 276 of 302 sampled meshes give 50.

**Script states.** Duration D = (F − 1) / 15 s.
- InitObjOn(i, b): drawn, state 9, time 0; with b true, time = D and state 8.
- PlayAnim(i, wait, v): state 2, speed v, root reset; a negative v first sets time = D.
- LoopAnim(i, v): state 0, speed v.
- StopAnim: state 10, unless the state is 8.
- DeactivateObj: state 4, hidden. The drawn byte (0x1B6) is cleared on the call itself, not at the next draw pass, so the object's height patches stop being ground candidates at once (SPEC-0018, Ground probe, step 2), and the run length of every part and bound frame record is set to −200.0, which switches the object's walls off (and the length of every type-10 record of a group the object owns, section 7.2). InitObjOn and ActivateObj set the drawn byte, and evaluate the pose at time zero, which restores the frame records' run lengths. An object left in its disc state 3 (never activated) has neither ground nor walls. Test vector: a static object (one key) with a height patch at 111.3 over deadly ground at 108.4: the probe returns 111.3 after InitObjOn and 108.4 after DeactivateObj, on the same tick as the call.

**Object poses at load versus in a visit (rule; High for the rule and the lists, which were gathered over all 161 distinct level-module images).** A flag decides an object's pose only at the moment a script tests it. The load-time calls (InitObjOn from Init or its helpers) test the flag once and draw the object at its start pose (second argument false) or its end pose (true); nothing watches the flag afterwards. Changing the pose inside a visit takes an explicit call (PlayAnim, LoopAnim, ActivateObj, DeactivateObj, SetObjToTime, StopAnim or a later InitObjOn). So a door that a scene opens by setting a flag is open in that visit only if the same scene also animates it; otherwise it changes at the next load of the pack, which is why a flag set by a scene and a door drawn by the same flag can disagree for the rest of the visit. An object never initialised stays in state 3: not drawn, no walls, no ground.
- Pose-only objects whose own pack sets the controlling flag in a scene that does not animate them (the pose changes at the next load): pack 784 object 3 (flag 543, set by script trigger 1, which plays object 2 instead); pack 951 object 0 (flag 121, set by the item-98 talk); pack 1247 object 21 (flag 78, set by trigger 4); pack 1363 object 2 (flag 692: the Mortog scene sets 692 and animates object 0 and deactivates object 1, but object 2 shares the 692 load pose and is not touched); pack 1129 object 12 (flag 38, set by lever 22).- Flag-posed objects that the same pack animates in the visit by other means (not a gap, but the load pose follows the flag, so a reload shows the end pose at once): pack 1248 objects 0 to 5 (flag 79 is set by the Exit routine, triggers 16 to 18 animate them in the visit); pack 1371 objects 1 to 3 (flags 100 and 342 to 344; the wave triggers pass the object number as an argument); pack 1398 objects 1, 2, 5, 6, 7, 9, 10 and 15, 17, 19, 21 (flags 106 and 724; Update helpers and triggers animate them from module words); pack 909 objects 2, 3, 10 and 11 (flag 110, animated by the farm threads).
- Pose-only objects whose flag is set in another pack, so they change at the next load of their own pack: 696 object 0 (flag 137); 706 object 0 (248); 772 object 1 (543); 822 objects 1, 2, 3 (55) and 6 (57); 956 and its copy 970 objects 1 and 2 (214, 283); 1023 object 5 (10, 15, 150); 1098 object 1 (30, 31); 1139 object 0 (35); 1163 object 0 (155); 1244 object 0 (273); 1327 objects 0 to 2 (92).
- Objects that a module animates only in scenes (never posed at load; invisible and passable until the first call) and that carry ground patches and wall parts: 678 objects 0, 1; 757 objects 0, 1; 934 object 0; 1066 object 0; 1081 object 4; 1096 objects 0, 11, 14; 1122 objects 0, 3, 5; 1244 object 6; 1246 objects 1, 2, 4; 1247 objects 6, 13, 14, 22, 30; 1249 objects 11, 14, 18, 22; 1353 objects 5, 6; 1398 objects 3, 4, 8, 11; 1428 object 3; 1432 objects 0 to 2. Seven packs that also pose other objects at load through computed object numbers are listed separately below.
- First load of seven packs with the normal first-visit flags (from the code; not run in the original; High). Pack 777 (flags 252 to 256 clear): objects 3, 4, 5, 6 and 14 are posed at load, visible and solid at the start pose; object 15 is hidden and passable until flag 255 is on. Pack 1381 (no flag tests): objects 38 to 43, 172, 175, 178, 181, 184 and 187 are posed at load, visible and solid; they are not invisible. Pack 735 (flag 263 clear): objects 0 and 13 are hidden and passable; with 263 on they are posed at the start pose, visible and solid. Pack 956 (flags 214 and 283 clear): objects 3, 4 and 5 are hidden and passable. Pack 1324: objects 22 to 28 are hidden and passable. Pack 1382: objects 0, 1 and 21 to 27 are hidden and passable. Pack 1397 (flags 687, 698, 699 and 700 clear): objects 32, 33, 34 and 46 are hidden and passable.
- Test vector: pack 1363 loaded with 692 clear: objects 0 and 2 at their start pose, object 1 never initialised. The fourth Mortog lands: 692 is set, object 0 plays its animation, object 1 is deactivated, object 2 is unchanged. Reload the pack: objects 0 and 2 are both drawn at their end pose.

Arena note: pack 1432's objects 0 to 2 are solid boxes that the module's activate-world callback makes solid (SPEC-0039, module 1432; the arena guide is in SPEC-0054).

**Per tick.** The world tick runs before the hero update (SPEC-0018, Who walks what), and only objects in states 0, 2, 5 and 6 are updated. With dt = 1/60 s:
1. Unless the state is 4 or 10: time += dt × speed, and the tick's advance a = 15 × dt × speed keys.
2. If time < 0: time = 0, and the end is reached when speed < 0. If time ≥ D: time = D, and the end is reached when speed > 0.
3. At the end: state 6 becomes 4; state 2 becomes 8 (finished); states 0 and 5 loop: n = trunc(time / D), minus 1 when speed < 0; time −= n × D; the root is reset to (0, 0, 0, 1).
4. Sample: k = trunc(time × 14.999999), f = (time − k × 0.06666667) × 14.999999.
   - Rotation and scale: lerp of keys k and min(k + 1, F − 1) by f. When the two rotations' dot product is negative the second is negated. The result is not normalised.
   - Translation, forward play (speed > 0, not an absolute call): this tick's change = s[k+1] × a, except: when k = 0 and f < a (the first tick after a loop wrap) the root is set (not added) to s[0] × f; when f − a < 0 (the tick crossed into key k) the change is s[k] × (a − f) + s[k+1] × f, with the s[k] term only when k > 0. The root accumulates the change.
   - Translation, absolute (InitObjOn, set-to-time, or speed ≤ 0): root = s[0] + … + s[k] + f × s[k+1] (the sum stops at F − 1).
   - The delta gate is evaluated only for per-tick playback: the linked locator record's f32 at record +0x08 must be positive and the global override word at ELF data address 0x2EB1C4 must be zero. The global word is 0 on disc; no writer was found in the boot program. The per-instance pointer at +0xB0 is set by the link forms of MoveObjToLocator and MoveObjToAnimatedLocator. The semantic meaning and source of the linked record's +0x08 value are unknown. A failed gate, an absolute evaluation, or a seek uses the absolute translation sample. The root accumulator at instance +0x80 feeds this same object's transform output (+0x100 through +0x180); it is separate from the character playback-layer accumulator in SPEC-0005.
5. Position P = root + base. The base is the animated locator's position, else the locator record's +0x30, else the rest translation (+0xF0). The +0x90 offset is added when the mode byte is 1 or 3; w = 1.
6. Outputs: +0x180 = P + corner + size / 2. Rotation matrix R(q) (columns, local to world); when the mode byte is 2 or 3, R = R × R(+0xA0). Then +0x150 = P − the old +0x130 (w = 0); +0x160 = q; +0x100, +0x110 and +0x120 = the columns of R times the scale; +0x130 = P.

Inside a key interval an object moves |s[k+1]| × |speed| / 4 units per tick.

**Clip duration in ticks (the one statement of the rule; other specs point here; High for speed 1.0 and for the sign rule, Medium-High for other speeds).** The clip length is D = (F − 1) × 0.06666667 and the step is dt × speed, both in single precision, with dt the single-precision 1/60 (bit pattern 0x3C888889). The console's FPU truncates every product and sum toward zero (SPEC-0008), so the running time stays a hair below the exact sum, and a play-once clip (state 2) is finished on the first advance whose time is at or above D.
- Forward play (speed > 0). The number of advances is floor(4 (F − 1) ÷ speed) + 1: the plain round-up of the ideal count 4 (F − 1) ÷ speed when that is not a whole number, and one more than the ideal when it is a whole number. At speed 1.0 this is exactly 4 (F − 1) + 1 for every F from 2 to 60 (none finishes on the ideal tick); at speeds 0.2, 0.25, 0.4 and 0.5 the same holds. One exception was found on the grid of speeds 0.2, 0.25, 0.4, 0.5, 0.7, 1.0, 1.5, 2.0 and 3.0 with F from 2 to 60: F = 2 at speed 2.0 takes 2 advances. A rewrite should run the truncating sum itself instead of the closed form.
- Backward play (speed < 0, from the end pose, finished when the time drops below 0): no extra tick; the count is the round-up of 4 (F − 1) ÷ |speed| (one exception: F = 2 at speed −2.0 takes 3).
- When a script waits for the end (PlayAnim with wait), the first advance is made by the world step of the tick after the call, the wait polls once per tick, and the thread resumes in the tick of the last advance: a clip started in tick T, waited on, resumes in tick T + n.
- Test vectors (n = advances): F = 6, speed 1.0: 21 (ideal 20). F = 11, speed 1.0: 41. F = 30, speed 1.0: 117 (1.95 s). F = 7, speed 0.7: 35 (ideal 34.3). F = 30, speed 3.0: 39. F = 20, speed 0.2: 381; F = 21: 401. F = 25, speed −1.0: 96. F = 23, speed −2.0: 44.
- Character clips use a different counter (SPEC-0005, Playback 1a): they are exact for the key spacings S = 0, 1, 2 and 4 and one tick late for S = 3.

Examples on disc (speed from the script; Medium-High for the reading of each script call):

| Kind | Pack / object | Path | Speed |
| --- | --- | --- | --- |
| Long lift (6 height patches) | 1391 / 0, PlayAnim(0, 1.0) | F 300, 19.93 s, y 0 to −218.41, no rotation | 0.7305 per key = 0.1826 per tick |
| Short lift (4 patches) | 1411 / 0, PlayAnim with wait, 1.0 | F 30, 1.93 s, y +19.88 | at most 1.106 per key = 0.277 per tick |
| Lift (4 patches) | 790 / 0, PlayAnim(1.0) | F 16, 1.0 s, y +9.0, yaw −90 degrees | peak 0.225 per tick |
| Sliding bridge (4 patches) | 757 / 2 to 5, PlayAnim(+1 or −1) | F 15 or 16, about 1 s; object 2 from (0, 0, 0) to (1.226, 0, −3.521), yaw −19.2 degrees | peak 0.143 per tick |
| Fast elevator | 864 / 15 and 16 | F 13, 0.8 s, y 74.24 | 1.55 per tick |
| Rotating bridge (yaw only) | 1122 / 1, 2, 4 and 5 | quaternion (0, 1, 0, 0) to about (0, −0.68, 0, 0.73) | the rotation rule of the SPEC-0018 ground attach applies |
| Door pusher (sphere primitive) | 699 / 0, 714 / 0, 732 / 0 | F 61, y +3.87 | pushes only |
| Pushed blocks | 1394 (3 blocks) | movable blocks, not placed objects (SPEC-0048) | step 2.7259 per push |

### 7.2 Type-10 ladder and climb-panel groups owned by a placed object (High unless stated)

A type-10 group (SPEC-0006: centre, squared radius, record count and 0x60-byte records) belongs to a placed object only when that object lists it. The link lives on the object: its group count (+0x1BC) and index list (+0x1C0) name groups of the pack's type-10 table (section 7 table). Nothing on a group or record says who owns it on disc (group +0x18 and record +0x48 are zero there), and no geometry decides it: a ladder whose face happens to lie along an object's wall is not owned unless the object lists it. On disc 3 of the 194 groups are owned: pack 777 object 12 owns group 0, pack 800 object 0 owns group 3 (one key, so it never moves) and pack 1098 object 6 owns group 0. Every other ladder and climb panel is static.

At load, for each listed group the group's +0x18 word is set to the object, and every record of the group gets +0x48 = the group and its length LU (+0x40) = −200.0, so the records are off until the object is activated. The object's own copy of the group (+0x1CC) holds the records in the object's rest frame. The ground attach of SPEC-0018 (The platform pointer) follows record, then group, then object to find the platform.

Every pose evaluation of the object (InitObjOn, ActivateObj and the per-tick motion of section 7.1) rewrites each owned record from the object's copy, on the same tick as the object and before the hero update: LU is restored from the copy; the start point is mapped as the offset from the rest translation split along the rest axes and rebuilt along the object's current columns, added to the current translation; the normal, U and V axes are rotated by the object's current rotation; the four plane offsets are recomputed from the new start so that each plane still passes through its record; LV and the neighbour links are those of the rest record. An owned record therefore moves and turns with its object. DeactivateObj sets LU to −200.0 again for every owned record. The group's centre and squared radius in the type-10 table are never rewritten: they stay at the rest pose. The hero's object-wall loop and some projectile and character walks use them as a distance pre-filter, so a moved ladder face is not tested as a wall when the actor is farther from the rest centre than the radius (Medium for the effect on play). The ladder mount and climb-panel grab tests walk every record without that pre-filter, so a moved ladder is still grabbed (High).

Test vector (pack 777, ladder 0 on object 12, real disc values). Rest face record: origin (−75.284, 45.313, −22.712), normal (0, 0, 1) with plane offset 22.712, U (1, 0, 0), V (0, 1, 0), LU 1.630, LV 14.874; its top strip has origin (−75.284, 45.313, −23.632). Object 12 has 17 keys (1.0667 s), no rotation and scale 1; its z samples in 1/1024 units are 0, 21, 59, 91, 117, 138, 153, 161, 164, 161, 153, 138, 117, 91, 59, 21, 0 (sum 1644, so 1.6055 in all). At time 0 the face is at z −22.712; at 8/15 s (key 8, sum 904) the offset is 0.8828, the face is at z −21.829 and its plane offset is 21.829; from 16/15 s (finished) the offset is 1.6055, the face is at z −21.107, the strip origin at z −22.027, and x, y, LU, LV are unchanged. The object's own front wall (normal +z) moves from z −22.849 to about −21.24 at the same time, so the face stays 0.137 in front of it and ladder 0 still works at the end pose. Before object 12 is activated the ladder does not exist (LU −200.0). The book of flag 253 plays object 12 at speed 1.0 with no wait; a later load with flag 253 set starts it at the end pose. Groups 1 and 2 of the pack are listed by no object and never move.

The other two owned groups (Medium-High; the disc values are High). Neither is a vertical ladder like pack 777's: each has two records, a horizontal strip with its normal pointing up and a small vertical plate.
- Pack 1098 object 6 (group 0; centre (106.431, −99.375, 103.700), squared radius 77.998): 22 keys at 15 keys a second, so 1.4 s; rest translation (106.415, −99.375, 103.685); yaw about −45.7 degrees; only the y translation is keyed. Cumulative y offsets by key: 0, −0.927, −1.868, −2.820, −3.780, −4.745, −5.712, −6.678, −7.640, −8.595, −9.540, −10.472, −11.388, −12.285, −13.160, −14.011, −14.834, −15.626, −15.729 (key 18, the lowest point), −15.240, −15.162, −15.378 (rest at the end). The strip is 1.63 by 17.576 along the horizontal direction (0.698, 0, 0.717); the plate is 1.63 high by 0.641 long. The group's distance pre-filter stays at the rest centre, so once the object has sunk 15.4 the wall pre-filter (radius 8.83 from the rest centre) no longer offers these records to an actor farther away than that, while the grab tests still find them.
- Pack 800 object 0 (group 3; centre (470.803, 301.614, 2363.819), squared radius 26.682): one key, no motion, yaw 62.0 degrees about (482.087, 296.310, 2387.563); it starts in disc state 3 (off), so its two records (a 1.773 by 10.067 strip and a 1.773 by 1.499 plate) exist only while the object is active.

### 8. Lighting: what the light-set records feed (High unless stated)

- World geometry (the level display tree) is unlit: no light set and no light rig is applied to it, and its vertex colours are used as stored (SPEC-0004, vertex colour). High.
- Type 30 light sets reach characters only. The character light walk of SPEC-0046 (Character lights) places light-set entries into the four light slots of the character's light context (SPEC-0046, Light slots); the lit-vertex arithmetic that consumes those slots is in SPEC-0004 and SPEC-0046. High.
- Entry roles by light type (section 2, +0x30; Medium-High):
  - type 1 (array C): the ambient term. It fills the ambient slot; its direction is never used.
  - type 2 (array A): a directional light along the entry direction (+0x10, the travel direction).
  - type 3 (array B): a point light. Its direction for a given object is normalize(object position − light position), computed per draw; the stored direction is not used. Its weight is the falloff of SPEC-0046 (Character lights). Between 0.9 r and r it does not take a slot: it cross-fades the directional light it would displace, from full strength at r down to 0 at 0.9 r (SPEC-0046, Character lights).
- Dynamic point lights come from effect particles (the light-emitter emitters of SPEC-0034), not from world records; they join the same character walk after the light set (SPEC-0046, Character lights).
- Lit objects that have no light context of their own (placed objects, items and other props with lit batches) use the scene's default lights, a fixed four-light rig built from program constants (SPEC-0046), never a light set. Resource 527 describes the rig's three directional lights but is never read (section 3). High.

Synthetic examples:
- Type 3 entry at (0, 10, 0), object at (0, 0, 0): direction = normalize((0, 0, 0) − (0, 10, 0)) = (0, −1, 0), the light travels straight down onto the object.
- Type 3 entry at (3, 0, 0), object at (0, 0, 4): direction = (−3, 0, 4) / 5 = (−0.6, 0, 0.8).
- A type 1 entry with any stored direction contributes only its colour, as ambient.

### 9. Placed effect spawns (type 39)

At load, each type 39 record is bound to the effect definition whose id matches its effect id (the first match in the shared definition list; SPEC-0034). No separate object is created at load: the record is the instance, and it keeps its position (column 3), its unit forward direction (column 2) and its effect id (+0x40).

Per-pass zone gate (High). Each pass, for the current world and for the second world, every active record (+0x48 nonzero, id not −1) is zone-queried (SPEC-0006, Binary grids, Query) with column = the low byte of its zone number (+0x44) and key = that world's id. When the query passes, the record is updated with the camera as the listener; when it fails, the record is stopped (drained) for that pass. Inactive records are not touched.

At load, binding copies the definition's active word into the record. Every definition on disc has this word set, so all valid placed records become active even when their source record has zero here. Type-40 switch-on also sets the flag; switch-off and StopEffect clear it. StartEffect runs one update without changing the flag (SPEC-0034, section 6). The per-pass zone gate updates active records in a visible zone and stops active records outside it. The update applies the cull rule below. The spawn's cull, threshold and cone settings come from the record:

- Each frame, let d2 be the squared distance from the spawn to the listener. If the cull value is above 0 and d2 is greater than it, the spawn stops.
- The spawn is in its near state while d2 is at most 10,000 and in its far state above that. A change of state restarts its child set. (Medium: read from the instance update path.)
- If the cone-test flag is 1, take the unit vector from the spawn to the listener and its dot product with the spawn's forward direction. If the dot is below 0.5 and d2 is greater than 225, the spawn stops.

The glow at a save point is not a type 39 spawn. It is a pair of effect emitters owned by the world pack's own character region (definitions 400 and 404 of shared effect group 3, SPEC-0034), which the region moves to the save point nearest the hero (section 13, Save points). No type 39 record is tied to a save point. High.

### 10. Placed object draw gate and setup (Medium)

**Setup.** At setup, an object takes its inline mesh's child offsets, binds each part to its model-set record (part +0x60), writes −200.0 into the bound record's run length (+0x40) and sets that record's self and owner pointers (section 7, Parts at run time; SPEC-0018, Model-set entries), and sets a bounding radius. The radius starts at 50.0 (a code constant). If the mesh child's values at offsets 0x20 and 0x24 are larger in absolute value, the radius becomes the larger one.

**Reset.** A reset routine exists (gate byte (0x1B7) 1, zone column (0x1B8) 0, drawn flag (0x1B6) 0, the mesh and model-set offsets cleared) but it is not run for the objects of a loaded type-29 chunk: loading only converts the offsets to addresses and binds the parts. It runs only when the object pools of section 15 are built, and the chunk's own objects keep the gate byte and the zone column stored on disc. When it runs it sets the gate byte 1, the zone column 0, the drawn flag (0x1B6) 0, the child pointers 0, several counters 0, and three 16-byte blocks at 0x90, 0xA0 and 0x160 are cleared. The reset does not set the state at 0x1A0.

**Draw gate.** Each frame, for each object in the list (skip zero entries):
1. The draw test is true when the object's draw-gate byte (0x1B7) is zero, or when its state (s32 at 0x1A0) is not 3 and not 4. The test result is stored in the byte at 0x1B6. The gate byte starts as stored on disc (1 in 3,489 objects, 0 in the six objects 1 to 6 of pack 1298), and every later write (InitObjOn and the type-40 group paths) stores 1; nothing in the boot program or the level modules stores 0. So in play the test is: the object draws unless its state is 3 or 4, except that the six pack-1298 objects, which start with gate byte 0, draw whatever their state until a script writes the gate byte (High). Keep the gate byte.
2. If the test is true, query the binary grid (SPEC-0006, type 35) with the object's zone key (byte at 0x1B8) as the column and the key −1 (the active block). An object whose query fails is not drawn. The zone column is the byte stored on disc (0 to 25; 721 of the 3,495 objects have a non-zero column; in every pack with a zone grid it is below the width of block 0, and packs without a grid hold only 0). No routine of the object code stores a value into that byte for the objects of a loaded chunk, so the column never changes (Medium-High: the only stores are zeros in the reset above, which does not run for them, and in effect-setup routines on a different record family). Test vector: an object with column 4 and a camera zone row that holds 0 at column 4 is not drawn; the same object with a row that holds 1 at column 4 is drawn.
3. The colour is the four bytes at 0x1A4 to 0x1A7. The first three are divided by 256; the fourth is the fourth byte plus 1, divided by 256 (a code constant, 2⁻⁸).
4. Draw the inline mesh with the object's transform block at 0x100.

The state is written by the script calls of section 7.1 (InitObjOn draws the object; DeactivateObj sets state 4 and hides it). On disc almost every object is in state 3, so it is not drawn until a script calls InitObjOn. No state other than 3 and 4 appears in the draw path.

**List membership (High).** Placed objects are not in the object lists of SPEC-0046: those lists hold character slot records (characters, the active petpet and the heroes), not placed objects. The draw gate above is the only draw of a placed object. The world draw pass walks the world's own placed-object list and the breakable list directly and submits each object's mesh with its transform, so a placed object gets no far flag (the 50-unit far set of SPEC-0046) and is not dropped by the 350-unit submit cull; the mesh draw applies only its own view-volume clipping. Test vector: an object with gate byte 1, state 0 and a passing zone query, 400 units from the view point: it is drawn. The same object in state 3 or 4 with gate byte 1: not drawn.

**Designer draw distances (High).** The two designer fields named draw distance and low-resolution distance belong to the monster table rows (row offsets 0x32 and 0x34), not to placed-object type data. 2500 is only the top of their editor range, not a default (the default is 0). No code reads either field, so no character or object uses its own value; the far threshold is a code constant (SPEC-0046, Designer properties). They change nothing that is drawn.

**Second list.** The second list has M entries, the world breakables of SPEC-0048. Each entry's state word (s32 at +0x40, within the entry) is tested. An entry in state 1 (intact) or 3 (intact, not breakable) is drawn with its own 4 x 4 matrix (the first 64 bytes of the entry) and intensity 1.0. The mesh is the entry's inline mesh at +0x44, or, when that is absent, the near mesh of record +0x48 of the first type-20 chunk (SPEC-0048, Breakable draw; High). Entries in state 0 (disabled) or 2 (broken) are not drawn on this path.

**Collision use.** Placed objects and the world's type-11 primitives are used by the contact and move rules of SPEC-0018 (Object collision). The draw gate above does not depend on them.

### 10.1 Rotation steps 0.0006 and 0.0009 (pointer; High)

The two rotation steps 0.0006 and 0.0009 radians (boot data words 0x2DB48C and 0x2DB488) are read only by the per-frame sky dome draw: they turn the dome's role-3 and role-2 layers about the world Y axis (SPEC-0046, Sky domes). No placed-object routine reads them. The dome keeps each angle unwrapped: each draw stores the angle minus its step, and the reduction by 1/(2π) happens only inside the sine and cosine evaluation, so the stored angle is never brought back into one turn. On disc no dome has a role-2 or role-3 layer, so nothing turns. The placed-object rotations of the pack 1299 puzzle (Behavior) and of recorded motion (section 7.1) are separate rules and do not use these steps.

### 11. Types 40 and 41 (partial)

**Type 40** (191 level packs, one chunk each). The payload begins with 21 fixed records of 0x44 bytes (0x000 to 0x533). Word 0 of fixed record k is k, and the word at +0x3C is 2 in every pack. Seven pointer fields per record are rebased at load when nonzero. Record 3 gets two runtime blocks at walk time (0x80 bytes at +0x20, 4 bytes at +0x30). Bytes 0x534 to 0x593 are nonzero in every pack and are not covered here. The u32 at 0x594 is the list count (nonzero in 81 packs, 458 entries in total). The u32 at 0x598 is the list offset; it is 0x59C in all 191 packs, so the list always starts at 0x59C. Each list entry is 0x44 bytes. Entries fit in every case. High.

List entries are **script groups**: the kind word picks a built-in behaviour that the game installs at each world load, and the script calls ReActivateScriptGroup and GetScriptGroupInstance act on an entry by its position in the list (SPEC-0048 gives the lock chest, sealed chest and chain behaviours; SPEC-0051 the dig site). Each list field below is a count followed by a payload-relative pointer to that many s32 indices (rebased at load when nonzero). Fields (record-relative):

| Offset | Size | Field | Range on disc | Confidence |
| --- | --- | --- | --- | --- |
| +0x00 | u32 | kind | see the kind table below | High |
| +0x04 | u32 | world reference (written at run time) | | High |
| +0x08 | u32 | key: a row of the shared collected table (SPEC-0051) | 0 in 227 entries; 1 to 199 on 145 lock chests, all 39 sealed chests and one dig site (165); 201 to 247 on 46 dig sites | High |
| +0x0C, +0x10 | count, pointer | locator indices (type-11 entry list) | | High |
| +0x14, +0x18 | count, pointer | script trigger indices | | High |
| +0x1C, +0x20 | count, pointer | placed-object indices (type 29) | | High |
| +0x24, +0x28 | count, pointer | camera indices | | High |
| +0x2C, +0x30 | count, pointer | breakable indices (type-29 second list) | | High |
| +0x34, +0x38 | count, pointer | placed effect indices (type 39; section 4) | count 0 (332 entries), 1 (94), 2 (9), 4 (12), 6 (5), 8 (3), 9 (2), 49 (1) | High |
| +0x3C, +0x40 | count, pointer | drop pair: index 0 is a placement ordinal to drop, index 1 a character id to activate | non-empty on every chest, sealed chest and dig site | High |

Kinds on disc (High):

| Kind | Built-in behaviour | Entries | Packs |
| --- | --- | --- | --- |
| 0 | none built in (script data only) | 49 | 15 |
| 1 | lock chest (Square press) | 146 | 40 |
| 2 | element-sealed chest | 39 | 39 |
| 3 | proximity fuse (live only with 32 objects and 1 breakable: 9 entries; the other 27 are empty) | 36 | 10 |
| 4 | breakable chain (switch group) | 39 | 8 |
| 5 | reaction object | 29 | 4 |
| 6 | dig site (SPEC-0051) | 47 | 28 |
| 7 | none built in | 12 | 5 |
| 12 | none built in | 61 | 2 |

Kind 3 counts down while the hero is near and not standing, slow-walking or sneaking, then breaks its breakable with sound cue 178 (Medium). Kind 5 hides its looping object, plays a second object once, and restores the loop each time its trigger fires (High). Level scripts do read them (SPEC-0048, Open question 40): kind 0 entries through the script-group instance query in 14 packs, kind 7 through the main-trigger query in the five packs that have it, and kind 12 through the type callbacks that pack 1171 registers (the kind-12 entries of pack 1369 are inert). [High]

Keys (High). The key is the entry's persistent mark in the collected table: lock chests and sealed chests use rows 1 to 199 as their "opened" mark, and dig sites use rows 200 + k. The row is tested when the entry is set up (only when its drop list is non-empty) and set to 1 when the group completes. Keys repeat only between packs that share a module payload (1077 and 1129; 956 and 970) and once between packs 1096 and 1122 (key 185, two chests), which therefore share one opened mark.

Switch path (High for the key arithmetic). A switch set with id k maps to a key: k + 200 when k is below 201, and k otherwise. The key is searched in the current world's list, then in the next world's list. When found, a nonzero value starts the one placed effect named by the entry's first index (its active flag is set to 1) and a zero value stops it (flag set to 0; runtime slots cleared); further indices in the entry's list are not used (32 entries on disc have more than one; SPEC-0006). Switch ids 0 to 200 therefore reach keys 200 to 400, that is, only dig-site rows. Keys 1 to 199 are not reached by this path, and they need none: they are chest rows, read and set by the chest behaviours themselves. The same setter is described in SPEC-0030, section 2.1.

**Type 41: atmosphere zones** (138 level packs, 185 records). The payload has a count N at +0x00 (1 to 6; 1 in 98 chunks) and 0x50-byte records from +0x10. Record fields are record-relative (the value at +0x44 is itself a payload-relative offset). The camera does not read these records; the selection below only uses the camera eye as its position.

| Offset | Size | Field | Range on disc | Confidence |
| --- | --- | --- | --- | --- |
| +0x00 | 4 × f32 | vector A: an RGBA colour (each lane rounded to a byte) for the screen post pass; used only while that pass is in its mode 1 (SPEC-0046). The renderer sets the post-pass mode to 3 when it is built and nothing changes it, so vector A has no visible effect in any area | | High |
| +0x10 | 4 × f32 | vector B: the screen overlay colour, RGB plus alpha; the overlay is drawn only when its alpha byte is nonzero. Gated records get alpha 28.0 at load | | Medium-High |
| +0x20 | f32 | depth range maximum: written to the render record's +0x34 only when it is not above that record's limit at +0x5C, and then the view-screen matrices are rebuilt. +0x30 and +0x34 of the render record are the minimum and maximum depth-buffer values that the near and far clip distances map to (the standard PS2 view-screen mapping), so the value sets the depth-buffer scale, not a draw distance; it does not move the clip planes | 100000.0 in 92 records; 1048576.0 in 29 | High (path); Medium-High (meaning) |
| +0x24 | u32 | gate key (0 = ungated) | 0 in 128 records; 1 to 13 otherwise | High (values); Medium-High (meaning) |
| +0x30 | 3 × f32 | bounding circle centre x, y, z of a polygon zone (y unused) | | Medium-High |
| +0x3C | f32 | bounding circle radius | 1.0 in 163 records | Medium-High |
| +0x40 | u32 | point count M (0 = zone without points) | 0 in 163 records; 4, 5, 7 and 9 most often | High (count) |
| +0x44 | u32 | payload-relative offset of the M points (rebased at load when M is nonzero, cleared otherwise) | | High |
| +0x49 | u8 | closed flag: nonzero means the points form a closed polygon, zero an open polyline | | Medium-High |

Chunks with points (11 chunks, 22 records) have M sixteen-byte points after the record array (four floats each; only x and z are used). A zone with points is a polygon zone; a zone without points is a base zone.

Per-frame atmosphere zone selection (Medium-High). Each frame, after the world update, the game picks the zone output from the camera eye position Q (SPEC-0027 gives the order in the tick):

1. World change crossfade. The world the output belongs to and the eye position at that time are kept. When the current world differs from the stored one, d = distance from the stored eye position to Q. If d > 50.0 the new world is stored and the crossfade ends; otherwise the crossfade weight is w = 1 − d / 50.0.
2. Base zone. The base zones (no points) are walked in order. The first ungated one is the fallback; a gated one whose key test passes is taken at once. If none is found, an all-zero default record is the base. The blend factor b starts at 1 if the base is gated, else 0. The working record is a copy of the base.
3. Polygon zones. For each zone with points whose key test passes, the distance g from Q to the polygon (horizontal, x and z):
   - g = −1 (zone ignored) when the circle distance (horizontal distance from Q to the centre minus the radius) exceeds 50.0;
   - inside (g = 0) when Q is within 0.5 of a vertex;
   - otherwise g is the distance to the nearer of the two edges that meet at the nearest vertex (at the ends of an open polyline there is one edge), and Q is inside when the y component of the cross product of (edge direction, Q − edge start) is positive.
   If Q is inside or g < 1, the zone replaces the working record. Else if g ≤ 50.0 the working record becomes lerp(working, zone, t) with t = 1 − (g − 1) / 49: vectors A and B and the render scalar are interpolated; the other fields stay those of the working record while t < 1.
   Blend term of the zone: a gated zone that Q is inside gives 1 when g > 25.0 or the base key is 1, else g / 25.0; outside, a term g / 25.0 is given only by an ungated zone over a gated base, else 0. b is the largest of the start value and the terms.
4. Output. Without a crossfade the output block becomes the working record; during a crossfade the output is lerp(working, previous output, w). The output goes to the colour blocks (vector A to the post-pass colour, vector B to the overlay colour), the render scalar to the renderer, and b is kept at +0x500 of the atmosphere state object (the game object's own slot at +0xE0, SPEC-0026; not the camera record, whose +0x500 is the eye pass's contact count), where the sky dome's role-10 layer reads it as its master opacity; that layer is drawn only when b > 0 (SPEC-0046, Sky domes).

Key test (High for the predicates and ids; flag story meanings remain unknown): key 0 is not a flag key; the base-zone scan treats it as ungated, while a polygon zone with key 0 is not a passing candidate. Key 13 always passes; unlisted nonzero keys fail. Keys 1 to 7 and 9 to 12 pass only when the first listed game flag is set and the second is clear; key 8 passes when its single flag is set. The flag ids are:

| Key | Required set flag | Required clear flag |
| --- | ---: | ---: |
| 1 | 83 | 5 |
| 2 | 10 | 322 |
| 3 | 64 | 283 |
| 4 | 57 | 58 |
| 5 | 74 | 263 |
| 6 | 85 | 298 |
| 7 | 92 | 94 |
| 8 | 248 | — |
| 9 | 332 | 100 |
| 10 | 332 | 102 |
| 11 | 332 | 104 |
| 12 | 332 | 106 |
| 13 | — | — |

Key 8 tests flag 248 alone; key 13 has no flag test. The ids do not establish narrative meanings for the flags. Constants (ELF data words; values High, use Medium-High): the zone range and world-change crossfade distance 50.0 at 0x2DB4CC, and the gated-zone blend distance 25.0 at 0x2DB4D0. The 0.5 vertex distance, the 1 and 49 of t and the alpha 28.0 are fixed constants.

Test vectors (synthetic): t at g = 1, 25.5 and 50 is 1.0, 0.5 and 0.0. World-change crossfade at d = 10: w = 0.8; at d = 60 the crossfade ends and the new world is stored. A gated zone with Q inside at g = 10 gives the term 0.4; at g = 30 it gives 1.0. A zone whose bounding circle is 60 units from Q (circle distance above 50) is ignored.

### 12. Type 10 and type 1000 cross references

Type 10 is described in SPEC-0006 (partial). Type 1000 is in section 5 of this spec.

### 13. Trigger volume records (runtime records; disc sources: the type 11 primitive records)

Each actor that takes part in triggers keeps a pointer to the grid cell under it, refreshed every tick (SPEC-0018, Grid cell pointer; Trigger pass in Behavior). The cell holds two lists, each a count and a pointer array: cell list A holds type 11 volume records directly (the records of section 6), and cell list B holds trigger objects. Each list B entry is a **trigger object**: a world breakable, that is an entry of the type 29 second list (section 7; SPEC-0048), read in place from the world pack. Their volume records are the type 11 primitive records: each record is its own 0x20-byte header followed by its body (section 6), and the kind byte in the header follows the list (spheres 1, cones 2, cylinders 3, kind-4 rectangles 4, kind-7 rectangles 7, fourth list 8; High). The records of the entry list at the type 11 pointer +0x4C (count at +0x24) also have kind-1 headers (High). The two tables of types 40 and 41 carry no trigger kind byte.

Trigger object (offsets in object):

| Offset | Size | Field | Use |
| --- | --- | --- | --- |
| +0x30 | 3 × f32 | position | direction of the push; position of the effect |
| +0x40 | s32 | state word | the breakable's state (SPEC-0048): 0 disabled, 1 intact, 2 broken, 3 intact but not breakable. An object is tested only when it is intact, state 1 or 3; a broken or disabled breakable is skipped. The same word gates the attack contact routines (SPEC-0036) and the draw of section 10 |
| +0x58 | s16 | script index | key of the script breakable hook (Behavior, Apply on a hit, step 1; SPEC-0048) |
| +0x5C | s16 | material word | chooses the hit cue (Behavior, Apply on a hit, step 3) |
| +0x5E | u8 | element byte | element class tested by the element gate (Behavior, Apply on a hit, step 2) |
| +0x60 | pointer | volume record | a null pointer is skipped |

Volume record (offsets in record, from the start of its header; the same offsets as the type 11 records of section 6):

| Offset | Size | Field | Use |
| --- | --- | --- | --- |
| +0x10 | u8 | volume kind | kinds 1, 2, 3, 4 and 7 are tested by every pass (Trigger pass in Behavior); the cell list A walk also tests kind 8, with the kind 1 rule, for the selected hero only; other values are not tested |
| +0x11 | u8 | mode byte | the volume's material for Tor's blade recoil (Behavior, Blade recoil) |
| +0x12 | u16 | flags | bit 0 (0x0001) marks a solid volume: it enables Tor's blade recoil and the contact push; bits 3, 5 and 8 enable the script trigger fire by kind (see Script trigger fire by flag bit below); on a kind-7 rectangle, flags 0x0002 mark a door (Doors below); the one kind-4 box of pack 1217 has flags 0x0080, a bit that no pass tests |
| +0x1A | u16 | gate mask | 0: always tested. Otherwise the volume is tested only when the mask shares a bit with a gate word of the current world record (+0x56B4). Medium |
| +0x1C | u16 | script trigger index; door number on a door | the index the fire step uses when this volume fires |
| +0x1E | u16 | class selector or book value | book value of a book station; class selectors 500 and 501 (section 6) |
| +0x24 | f32 | kinds 4 and 7: extent along the in-plane axis at +0x60. Kind 3: full length of the finite cylinder | kinds 3, 4 and 7 |
| +0x28 | f32 | extent along the in-plane axis at +0x50 | kinds 4 and 7 |
| +0x2C | f32 | kinds 4 and 7: taken as a switch (zero or below: static test; above zero: moving test). On disc the same field is the third extent of the rectangle record (section 6.1): 0.0 or above in all 675 kind-7 records (0.0 on every door) and negative in 70 of the 5,849 kind-4 records, so for kind 4 this interpretation is open (open question 32) | kinds 4 and 7 |
| +0x30 | 3 × f32 | kind 3: base point of the finite cylinder (its centre is the base plus half the length along the axis). Kind 1: centre of the sphere | kinds 1 and 3 |
| +0x34 | f32 | y of the kind-1 sphere centre; the kind-1 test reads no radius here | kind 1 |
| +0x3C | f32 | kind 3: radius of the finite cylinder. Kind 1: radius of the sphere (the w lane of the centre vector) | kinds 1 and 3 |
| +0x40 | 4 × f32 | kinds 4 and 7: rectangle normal (the fourth value's role is open). Kind 3: axis of the finite cylinder | kinds 3, 4 and 7 |
| +0x50 | 3 × f32 | in-plane axis 1 | kinds 4 and 7 |
| +0x60 | 3 × f32 | in-plane axis 2 | kinds 4 and 7 |

The kind 3 fields are the finite-cylinder fields listed above (radius, full length, base point and axis); kind 3 is a finite cylinder. For kinds 3, 4 and 7 the offsets above are the offsets of the type 11 cylinder and rectangle records (sections 6.1 and 6.4; High).

**Kind 2 (reconciled).** The weapon-contact pass's test for kind 2 reports no hit (High). The cell list A walk, the hero shot pass and the world projectile step route kind 2 to handlers that can fire the script trigger with flag bit 3. All 83 cones on the disc have flags 0x0000, so no kind 2 volume fires in play (High).

**Cylinder volumes (High for the structure and the measured values).** Every loaded cylinder record starts with its own 0x20-byte header (section 6). The list pointer (type 11 header +0x38) points at the first loaded header; the corresponding on-disc shape data starts 0x20 bytes later and advances by 0x50 per entry. The header counts sum to 10,038 declared entries, while 10,028 complete shape records fit in the on-disc payloads; resource 721's tenth entry is incomplete (section 6.4). Header fields for declared cylinder entries:

| Offset | Size | Field | Measured on disc |
| --- | --- | --- | --- |
| +0x10 | u8 | volume kind | 3 in all 10,038 records |
| +0x11 | u8 | mode byte (blade recoil material input) | 0 (6,627), 3 (1,470), 4 (1,106), 2 (464), 16 (211), 14 (68), 1 (45), 8 (23), 12 (15), 7 (6), 10 (3); 0 on every book station |
| +0x12 | u16 | flags | 0x0001 (8,707), 0x0008 (441), 0x0011 (256), 0x0010 (224), 0x0021 (192), 0x0040 (134), 0x0002 (45), 0x0020 (33), 0x0003 (2), 0x0000 (2), 0x0080 (1), 0x0031 (1) |
| +0x1A | u16 | gate mask | nonzero in 9 records; 0 on every book station; meaning Medium |
| +0x1C | u16 | script trigger index: the index the trigger fire step uses when this volume is hit. This is the field earlier named type 40 entry index | 0 on every book station; Medium-High for the fire use |
| +0x1E | u16 | book value, 1-based | nonzero only on the 134 records with flags 0x0040 (values 1 to 87) |

The runtime volume of a cylinder is the record itself: length at +0x24, base point at +0x30, radius at +0x3C and axis at +0x40 (section 6.4). These are the kind 3 fields above (High).

The volume's u16 at +0x1C is the script trigger index that the trigger fire step uses when the volume fires (Medium-High; the same field as the row above). Pack 1432 has 53 cylinders: 52 with flags 0x0001 and index 0, which never fire a script trigger (flag 0x0001 runs the press step and the contact push only; see Script trigger fire by flag bit), and one, record 8, with flags 0x0008 and index 1, which fires script trigger 1 when the test hits (High for the values). Record 8 has mode 0, gate mask 0, class word 0, base point (28.702, 270.608, −326.005), axis (0, 1, 0), length 3.0 and radius 6.0; it stands on the arena floor, the grid cells under it list it, and its centre is within 0.011 units of entry 2 of the pack's entry list. The selected hero's body cylinder touching it (cell list A walk) fires trigger 1 with object 1; a hero shot passing through it fires it with object 0 and shot word 1 (Script trigger fire by flag bit below; SPEC-0054). High.

**Other volume lists (kinds 1, 2, 4, 7 and 8).** Every record of the other type 11 lists has the same header (kind +0x10, flags +0x12, gate mask +0x1A, number +0x1C, class or book value +0x1E). The kind byte follows the list: spheres (+0x30) 1, cones (+0x34) 2, the kind-4 rectangles (+0x40) 4, the kind-7 rectangles (+0x3C) 7, the fourth list (+0x54) 8 (High, all 189 chunks). Flags measured over the disc:

| List (type 11 pointer) | Records | Kind | Flags (records) |
| --- | --- | --- | --- |
| spheres (+0x30) | 6,678 | 1 | 0x0001 (5,855), 0x0008 (587), 0x0010 (216), 0x0005 (7), 0x0002 (5), 0x0000 (4), 0x0200 (3), 0x0003 (1) |
| cones (+0x34) | 83 | 2 | 0x0000 (83) |
| kind-4 rectangles (+0x40) | 5,849 | 4 | 0x0001 (4,895), 0x0020 (659), 0x0080 (104), 0x0010 (85), 0x0008 (44), 0x0009 (31), 0x0120 (27), 0x0011 (3), 0x0050 (1) |
| kind-7 rectangles (+0x3C) | 675 | 7 | 0x0001 (300), 0x0002 (171: 169 doors and the two pads of pack 1299), 0x0020 (130), 0x0100 (22), 0x0040 (19), 0x0021 (18), 0x0061 (8), 0x0000 (5), 0x0004 (1), 0x0041 (1) |
| fourth list (+0x54) | 2,589 | 8 | 0x0004 (2,589) |
| entry list (+0x4C, count +0x24) | 4,964 | 1 | 0x0020 (4,924), 0x0021 (40) |

Values in pack 1432 (High): cylinders as above; 6 kind-4 rectangles with flags 0x0001 and index 0; one kind-7 rectangle with flags 0x0100 and index 0; two fourth-list records (kind 8, flags 0x0004, numbers 0 and 1); five entry-list records (kind 1, flags 0x0020).

The kind-7 rectangle of pack 1432 has its reference corner at (36.376, 269.199, -343.997), normal (-1, 0, -0.0017), extent 20.85 along the in-plane axis (0, 1, 0) and 37.25 along the in-plane axis (-0.0017, 0, 1), and switch value 0.0 (static). Pack 1432 has two volumes whose flags reach a fire path: this rectangle (bit 8), which fires script trigger index 0 (SPEC-0054), and the cylinder with flags 0x0008, which fires index 1 (High for the values; Medium-High for the fire paths).

**Script trigger fire by flag bit (High unless marked).** The fire step takes the trigger index from the volume (+0x1C) and the actor, object value and three block words from the path that calls it (SPEC-0030, Layout section 3). Which actors run each path, where, and in which order is in Behavior, Trigger pass. The paths and their gates:

| Path | Kind | Fires when | Object | Shot word (+0x20) | Attack word (+0x24) | Last argument (+0x28) |
| --- | --- | --- | --- | --- | --- | --- |
| Cell list A walk (body contact) | 3 | bit 3 (0x0008) set, bits 1 and 2 clear, and the test hits | from the owner's trigger-hit method | 0 | from the method | -1 |
| Cell list A walk | 1 and 2, and 8 for the selected hero | bit 0 clear, bit 3 set and the test hits | from the method | 0 | from the method | -1 |
| Cell list A walk | static 4 or 7 (switch zero or below) | bit 8 (0x0100) set, bits 0, 1, 4 and 6 clear, and the test hits | from the method | 0 | from the method | -1 |
| Cell list A walk | moving 4 or 7 (switch above zero) | bit 5 (0x0020) set, bits 1, 7 and 8 clear, and the test hits | from the method | 0 | from the method | -1 |
| Cell list B walk (trigger objects) | 1, 2, 3, 4, 7 | the same bits as the cell list A walk, on the object's volume record | from the method | 0 | from the method | -1 |
| Placed-object walk (inline volumes of animated placed objects) | as the cell list A walk | the same bits | from the method; the record is the object's own instance trigger | 0 | from the method | -1 |
| Hero shot pass | 1, 2, 3 | bit 3 set and the shot's test hits | 0 | 1 | 0 | -1 |
| Hero shot pass | static 4 or 7 | bit 8 set and the test hits | 0 | 1 | 0 | -1 |
| Hero shot pass | moving 4 or 7 | bit 5 set and the test hits | 0 | 1 | 0 | -1 |
| Movable block pass | 1, 3 | bit 3 set and the block's test hits | 0 (no actor) | 0 | 0 | the block's number |
| Movable block pass | static 4 (not 7) | bit 8 set and the test hits | 0 (no actor) | 0 | 0 | the block's number |
| Movable block pass | moving 4 (not 7) | bit 5 set and the test hits | 0 (no actor) | 0 | 0 | the block's number |
| World projectile step | 1, 2, 3 | bit 3 set, the test hits and the act-1 trigger map holds the index | 0 (no actor) | 0 | 0 | -1 |
| World projectile step | static 4 or 7 | bit 8 set, the test hits and the map holds the index | 0 (no actor) | 0 | 0 | -1 |
| World projectile step | moving 4 or 7 | bits 5 and 8 set, the test hits and the map holds the index | 0 (no actor) | 0 | 0 | -1 |

The trigger-hit method depends on the owner's class. The hero's method fires with object 1 and its attack-active word (1 while the current animation frame carries event bit 0x01, SPEC-0036) as the attack word. The world-character method (every non-hero character, the petpet included) fires with object 0 and, as the attack word, 1 when any of the five low bits of the character's motion state word is set. The base method does nothing. The hero's own body contact is therefore the only path that gives object 1. The method also writes the frame counter into the volume's +0x20 word. Its record-lookup argument is 0 on the cell list walks, so the trigger index is looked up plainly in the world's trigger table; on the placed-object walk it is the placed object, and the lookup goes to that object's own instance trigger list (Medium-High). A shot that hits an inline volume of an animated placed object fires the world trigger table entry of the volume's index instead, with no instance lookup (quirk; Medium).

Which kinds reach which test (High): the kind numbers 3, 5 and 8 that the dispatch tables name are flag bit numbers, not table numbers: bit 3 for kinds 1, 2 and 3, bit 8 for a static rectangle, bit 5 for a moving rectangle. The cell list A walk applies them only when its caller asks for a fire-bit filter (CollideEntityWithTriggers does; the per-tick trigger step does not, so its handlers apply their own gates of the table above). A volume with flags 0x0001 (bit 0, with bits 3, 6 and 8 clear) never fires a script trigger; it is a solid volume that gives Tor's blade recoil (Behavior, Blade recoil), the contact push, and stops a hero shot. Kind 8: all 2,589 records have flags 0x0004 and no handler acts on bit 2, so kind 8 never fires. The 27 kind-4 boxes with flags 0x0120 never fire from body contact (bit 8 blocks the moving test); they fire from hero shots, movable blocks and, with the act-1 map, world projectiles. Kind 2: see Kind 2 (reconciled) above. The weapon-contact pass (Behavior, Trigger volumes) never calls the fire step.

Side effects that are not script fires (High): on the cell list A walk, a kind-3 hit with bit 8 (any owner) and a kind-1 hit with bit 9 (0x0200, the selected hero only) set the boundary-contact word of the game-flow record (SPEC-0026), which the main loop clears each tick before the hero step; a kind-3 hit with bit 6 sets the pending book (Book station below).

**Object field seen by the script (High).** The fire step writes the object value it receives into the trigger block that it passes to the script callback (block +0x1C; the block's +0x10 is the trigger state and +0x14 the trigger number, SPEC-0030). The field is 1 only when the selected hero's own body touched the volume, and 0 on every other path. Because an object-0 fire from a path with last argument -1 rewrites only an idle record (SPEC-0030, Layout section 3), a hero who keeps the record active keeps the object at 1 while other firers touch the same volume.

**Load-time index writes (High).** Load steps write the trigger index (+0x1C) of bit-1 records. A kind-7 record (+0x3C list) with bit 1 set and class 500 or 501 gets the running count of such class records (section 6). A kind-7 record with bit 1 set and any other class, that is a door, gets the running count of doors so far in its list: its ordinal among the non-class bit-1 records. A cylinder with bit 1 set gets the running count of such cylinders. On the disc the written door ordinal equals the disc value of every one of the 169 doors, so the door number is unchanged. Pack 1432 has no door.

**Doors (High for the records and the count; Medium-High for the rule; High for the prompt and confirm).** A door is a static rectangle volume (kind 7, list +0x3C) whose flags have bit 1 (0x0002) set and whose class (+0x1E) is not 500 or 501; on the disc every door's flags are exactly 0x0002. There are 169 doors in 102 packs: one door in 77 packs, two in 12, three in 6, four, five and six in one pack each, eight in 3 and eleven in 1. Every door has mode byte 0, gate mask 0 and switch value (+0x2C) 0.0, so it takes the static rectangle test; its door number (+0x1C) is 0 to 10. The only other kind-7 records with bit 1 set are the two same-world pads of pack 1299 (class 500 and 501, section 6), which the static rectangle handler sends to the pad branch before the door branch. Sphere and cylinder records with bit 1 set (6 and 47) are not doors: the door branch belongs to the static rectangle handler only.

A door works only when all of these hold: the main hero touches it; the game-state code is 0; no screen fade is running; no enter-interior or leave-interior job is pending on the current world or on the exterior buffer; the hero is over a grid cell (SPEC-0006). Then:

1. **Script door.** If the current world's script registered a teleport callback for that door number (RegisterTeleportCallback, SPEC-0030), the callback decides and the default rule below is skipped. With a registered context above 0 the door posts that context as its prompt, and the callback starts as a script thread only on a tick with a Square press edge and the hero's door-confirm flag at 1; with context 0 it starts on contact, with no prompt and no press. This path never clears the confirm flag (SPEC-0006, Doors, item 5, and the stale-confirm edge case). The callback receives the trigger block with +0x10 = 1, +0x14 = the door number and +0x1C = 1. 124 doors have at least one teleport-callback registration for their number in their pack's script (Medium-High: the count takes registration sites regardless of their script conditions).
2. **Default door.** Otherwise (45 doors) the hero must confirm the door prompt: while the hero's door-confirm flag is 0, the door posts prompt 13 ("Enter") when the exterior is current, or 26 ("Exit") when the transition or the interior is current, and nothing else happens. The prompt ids are table-13 labels. The interaction chooser shows the posted prompt on the next tick and sets the flag on a Square press edge; the door, which is tested after the hero update in the same tick, then clears the flag and travels, so one press travels on the press tick (SPEC-0006, Door prompt and confirm). Once confirmed, let D be the pack that the exit value v of the grid cell under the hero names (entry v of the current pack's tag-0x0B list, then the world tables of SPEC-0006), and let e be entry (door number) of the current pack's tag-0x0E list, the arrival door:
   - current world is the exterior, D is a transition: the hero is placed at door e of the transition buffer, which becomes current;
   - current world is the exterior, D is not a transition and the two packs are not a listed area pair: D is entered as an interior (no door, locator 0);
   - current world is the exterior, D forms a listed area pair with it: the exterior is reloaded with D (a replacement load with "unload all"), arriving at door e;
   - current world is the transition: the hero is placed at door e of the exterior buffer, which becomes current;
   - current world is the interior: the interior is left. The transition buffer is reloaded with the pack it held when the interior was entered, and the hero arrives at door e of the exterior it was entered from (when no interior is loaded, the hero is only placed at door e of the exterior).

The buffers, kinds, area pairs, the tag-0x0B and tag-0x0E lists and the interior jobs are in SPEC-0006; the whole-game view is in SPEC-0026 (World buffers and travel). The two pack 1299 records with flags 0x0002 and class 500 and 501 are same-world pads, not doors (section 6).

**Grab-door records (dormant; High).** A kind-7 record with bit 1 clear, bits 2 and 3 clear and bit 7 (0x0080) set would build a 0x200-byte grab-door record (world list at +0x5808, count at +0x5804) for the hero's door-hold state (SPEC-0033). No kind-7 record on disc has bit 7 (over the 675 records: bit 0 in 327, bit 1 in 171, bit 2 in 1, bit 5 in 156, bit 6 in 28, bit 8 in 22), so the list is always empty and the door hold never occurs. Quirk: the counting pass excludes bits 2 and 3, but the building pass does not.

**Book station (High for the rule).** A kind 3 volume is a book station when its flags have bit 6 (0x0040) set and bits 1, 2, 3 and 8 (0x0002, 0x0004, 0x0008 and 0x0100) clear. The cylinder is tested against the character's body cylinder (radius 1.0, from the feet to 3.5 above; SPEC-0018), and the book event is raised on overlap, not on a miss (High); on each tick that the flags pass and the cylinders overlap, the pending book word receives the volume's book value (+0x1E), which is 1-based. On the disc every book station has flags exactly 0x0040, and no cylinder with other flags meets the rule. A book value of 0 means no book. The reader shows the book at index value minus 1 (SPEC-0035, section 6.1). The pending book word is one global word; its reader, the "Read Book" prompt and the book picker are in SPEC-0035.

On disc there are 134 book stations in 27 world packs (from 1 to 17 per pack; pack 777 has 17, pack 1414 has 14 and pack 1327 has 13). Their book values cover 1 to 87 except 78: 86 distinct values, so several books have more than one station. The book indices the reader shows (value minus 1) are therefore 0 to 86 except 77. No cylinder with other flags holds a book value (High for all 10,038 records). No other writer of the pending book index is known, so of the 88 books only indices 77 and 87 have no station (Medium-High).

Other cylinder flag values are not book stations:
- 0x0001 (8,707 records): solid volumes (flag bit 0; see Blade recoil in Behavior). These never fire a script trigger.
- 0x0008 (441 records): script trigger fire (bit 3; see Script trigger fire by flag bit). The volumes of breakables (type 29 second list, 1,915 cylinders) are stored with 0x0008 and become 0x0009 when the breakable is loaded or activated, which makes them solid as well (Cell list B walk; SPEC-0018, Breakable cylinders against characters).
- 0x0002 (45 records) and 0x0003 (2 records): skipped, because bit 1 is set.
- 0x0010 (224 records) and 0x0011 (256 records): book value 0; not save points. Bit 4 makes a cylinder solid for character collision (Cylinder flags in character collision, below). 0x0010 cylinders block characters but give no blade recoil; 0x0011 cylinders do both.
- 0x0020 (33 records), 0x0021 (192 records) and 0x0031 (1 record): not book stations and not save points. Bit 5 takes a cylinder out of character collision. 0x0020 cylinders do nothing in the rules of this section; 0x0021 and 0x0031 cylinders give the blade recoil (bit 0) but characters pass through them.

**Cylinder flags in character collision (High).** The three character collision passes (the per-record push, the per-tick contact pass and the blocking query; SPEC-0018) first apply the gate mask. A cylinder (kind 3) is then tested when bit 0 or bit 4 is set and bit 5 is clear. A static rectangle (kinds 4 and 7) in the same passes is tested when bit 0 or bit 5 is set. The blade recoil keys on bit 0 only. Test vectors: flags 0x0010, tested; 0x0011, tested; 0x0021, not tested; 0x0031, not tested; 0x0020, not tested; 0x0008, not tested (it only fires its script trigger).
- 0x0000 (2 records) and 0x0080 (1 record): no rule of this section uses them.

**Save points (High).** Save points are not trigger volumes, placed objects (type 29) or placed effect spawns (type 39). Each save point is a kind-4 checkpoint record of the world pack's own character region (SPEC-0006). A region keeps at most 8 of them; on the disc there are 88 save points in 60 world packs, and the flag word of every one is 0. They are handled by the region's per-tick update, not by the trigger volume pass of this section; SPEC-0025 gives the rule (nearest save point, sound, "Save Game" prompt, opening the save screen) and SPEC-0006 the record layout. The region also owns the save-point glow: a pair of effect emitters (definitions 400 and 404 of shared effect group 3, SPEC-0034) that the update moves to the save point nearest the hero. No cylinder flag value marks a save point; in particular the cylinders with flags 0x0010, 0x0011 and 0x0021 are not save points.

### 14. Shared object-setup descriptor fields (owner and names open)

The compact accessor group reads fields from a setup descriptor passed into shared object/render initialization. Its concrete owner is not established, so fields are named by offset only. The offsets and widths are High; the meanings of most fields are not needed for play (Open questions). Default if unresolved (rewrite choice): build each object's transform and render setup from the placed-object records of this spec and SPEC-0046; do not reproduce this descriptor.

| Offset | Size | Access |
| --- | --- | --- |
| +0x00, +0x04, +0x08, +0x0C | 4 × u32 | read |
| +0x10, +0x20, +0x30 | 3 × u64 | read |
| +0x40, +0x44, +0x48, +0x4C, +0x50, +0x54, +0x58 | 7 × u32 | read |
| +0x5C, +0x64, +0x6C | 3 × 8 bytes | read together into one 24-byte output block |
| +0x74, +0x78, +0x7C, +0x80, +0x84, +0x88, +0x8C, +0x90, +0x94 | 9 × u32 | read |


The compact accessor group reads the fields above from a setup descriptor passed into shared object/render initialization. The descriptor is built by several setup paths, but its concrete owner and most field meanings are not established. The selector at +0x04 changes initialization behavior; +0x10, +0x20 and +0x30 are vector-like inputs used in transform construction; the block at +0x5C, +0x64 and +0x6C is copied together; other words are copied or used as selectors without established individual meanings. A separate setter writes one word, one 64-bit value and two words into a larger runtime object at +0xC9C through +0xCAC. How that larger object relates to this descriptor is not needed for play: both feed render and transform setup, whose play-visible results are specified elsewhere (SPEC-0046). A list count is also read by a count accessor, and an indexed accessor on that kind of list returns zero when the index is outside the count; reproduce the zero for an out-of-range index. Default if unresolved (rewrite choice): see section 14's first paragraph; these accessors are render-setup helpers and need no separate rewrite structure.

### 15. Object pools (built per world)

When a world's object store is set up, one constructor builds seven pools, one table and one scene list: nine allocations in all. Each pool block starts with a 0x10-byte header whose first word is the entry count. Entries are linked through link words, and an empty link word is -1 in the layouts below (Medium).

| Pool | Entries | Entry size (bytes) | Allocation (bytes) | Source of the count |
| --- | --- | --- | --- | --- |
| A | caller-sized | 0x48 | count x 0x48 + 0x10 | caller argument |
| B | caller-sized | 0x48 | count x 0x48 + 0x10 | caller argument |
| C | caller-sized | 0x44 | count x 0x44 + 0x10 | caller argument |
| D | caller-sized | 0x18 | count x 0x18 + 0x10 | caller argument |
| E | 743 (0x2E7) | 0x18 | 0x45B8 | fixed |
| F | 64 (0x40) | 0x18 | 0x610 | fixed |
| G | 10 | 0x18 | 0x100 | fixed |

Initial entry words: most are 0, and several link words start at -1 (Medium). The constructor stores each pool's entry count in the owner's header, for use by the maintenance pass below.

Two more blocks are built:
- The world's script-thread slot table: 50 (0x32) thread slots, each with a 0x400-byte execution context (the 50-thread limit of SPEC-0030). The pools' link words hold handles of these threads, which is why the release rule frees a link when its thread has ended (Medium-High).
- A scene list with one 4-byte pointer per scene. Its count is the first word of the scene list kept in the scene object (at +0x5834). Each entry is then set to point at that scene's sub-list: an array of 0x48-byte link records of the same layout as pool A, with as many records as the first word of the scene's own table (the table at +0x1D8 of the scene record); a scene with no such table or a count of 0 keeps a null entry. Each record starts with its link word at +4 and its sub-record at +0x14, and gets the release rule and the state step of Pool maintenance (Medium-High).

Roles (Medium-High; owner SPEC-0030, which holds the sizes at High). The seven pools and the scene list are the script manager's callback tables:
- A: trigger callbacks (count = the first u32 of the type-11 payload, section 6);
- B: breakable callbacks;
- C: teleport-plane callbacks (world +0x57E4);
- D: entity proximity callbacks (count = the tag-0x12 entry count, world +0x3C; SPEC-0006);
- E: 743 game-flag callbacks;
- F: 64 level-flag callbacks;
- G: 10 item ask-about callbacks;
- the scene list: the per-placed-object instance triggers, each sized by the first word of that object's inline type-11 header;
- the 0x400 table with count 50: owner open.

The release rule below frees finished callback threads.

Confidence: High for the entry sizes, counts and allocation sizes; Medium-High for the roles; Medium for the initial words; Low for the table and the scene list layouts.

### 15.1 Built-in object pools that are never filled (High)

The world object store also builds six built-in pools that the world tick updates every tick:

| World offset | Records | Role |
| --- | --- | --- |
| +0xA0 | 30 records of 0x150 bytes from +0xB0 | puzzle objects; kind word at record +0x148: 1 statue, 2, 5 |
| +0x29F0 | 0x160-byte records from +0x2A00 (list header count at +0x29F0) | drifting, leashed objects with radius separation (below) |
| +0x2810 | 0xE0-byte records | secondary pool of the +0x29F0 pool: its add routine fills eight entries here for each spawn it makes; drawn like the other model pools below (Medium-High) |
| +0x2D40 | eight 0x160-byte slots | the plate manager (SPEC-0048) |
| +0x3850 | 0x140-byte records | tween nodes (the step tween of SPEC-0034 section 25; Medium-High) |
| +0x4260 | 0x160-byte records | model-instance pool of the same shape as +0x29F0; its add routine has no caller and no other pool feeds it (Medium) |

Their add routines have no caller in the boot program, no host record names them, and no data pointer reaches them, so every slot keeps its free mark and every update, contact test and completion step of these pools is skipped in play. A clean implementation may omit them. In particular: The world draw pass also walks the +0x29F0, +0x2810, +0x3850 and +0x4260 pools: it skips a record whose free mark is set, culls a record farther than 400 units from the draw reference point (distance squared below 160,000 is drawn) and draws the record's model object; nothing is drawn while the pools are empty. The only add routine that is called by another is the +0x2810 add, and its sole caller is the +0x29F0 add, which itself has no caller; so no pool is ever fed from outside (High).
- the plate manager of SPEC-0048 never runs; pressure plates, switches and levers in play are scripted trigger volumes (section 13; SPEC-0030);
- the statue count (four aligned kind-1 puzzle objects) never runs, and its completion step has an empty body; the hero's statue-turn state is never offered (SPEC-0033). The statue puzzle of pack 1353 is scripted (SPEC-0041);
- the +0x29F0 pool's update below never runs (High);
- the +0x3850 tween-node step never runs (dormant). For completeness, +0x130 is an active flag, not an elapsed timer: zero skips movement; when the completed-step byte reaches the total-step byte at +0x139, the flag is cleared and that update has no movement. Otherwise, one update increments the completed-step byte and adds source × (+0x134 / total steps) to position +0xA0. The scalar +0x134 is initialized to 1.75; its unit is unknown. Modes 0 and 2 select the vector at +0x20; modes 1 and 3 select +0x40; modes 2 and 3 negate the selected vector. No rule is established for other mode values. High for these operations; Medium for interpreting the scalar as distance-like.

**Synthetic check (dormant step).** With mode 0, source vector (8, 0, 0), scalar 1.75 and 7 total steps, each active update adds (2, 0, 0) to position. Seven active updates add (14, 0, 0); the following update clears the active flag without moving.

The three-state fall once listed here as a routine of the +0xB0 pool is the cut-reed fall of the foliage patches, and it runs in play (Behavior, Cut-reed fall; SPEC-0060).

**+0x29F0 pool update (dormant; High unless marked).** For completeness, each record update runs these steps in order. The pool's add routine has no call and no pointer, so none of this runs in play:
1. **Drift.** p ← p + 0.99 (p − p_prev) + (0.0003 i, 0.005, 0.0003 j), where i is a random whole number from 4 to 8 and j one from 5 to 9 (closed draws, SPEC-0008). Then p_prev ← p and w = 1. The previous sample p_prev is the last tick's position at record +0x150; 0.99 is a fixed program constant.
2. **Ground lift.** With g the ground height probed at the anchor (probe range 8.0) and r the radius at record +0x140: if y < g + r, set y = g + r + 0.25.
3. **Separation from records 0 and 1** of the pool, skipping the record itself and disabled records. If the centre distance d is below r_s + r_o, with overlap o = r_s + r_o − d and u = unit(self − other): self moves by u × o × r_o / (r_s + r_o), and the other by −u × o × r_s / (r_s + r_o). The smaller object moves more (SPEC-0040, Behavior 11A).
4. **Leash** (Medium-High). The point p − r × (the vector at record +0x80) must stay within the distance at record +0x144 of the anchor (record +0xB0, with y on the ground); the excess is pulled back.
5. An attachment step (not described).
6. The current position is copied to the previous sample.

Walk quirk (dormant): the pool's list walk advances its record pointer only after a record it has processed, so a disabled record stalls the walk on itself for the rest of that pass. The pool update is the second sub-table of the world tick (SPEC-0026).

Nothing in the game picks up, carries or throws a world object. The only caught-and-carried object is the Miamouse (SPEC-0040); characters ride moving placed objects only through the ground attach of SPEC-0018 (Behavior, Riding).

### 16. Type 16 foliage patches (summary; owner SPEC-0060)

Confidence: High. SPEC-0060 "Foliage patches (type 16)" owns the families, models, placement, fade bands, per-frame logic, bush cutting and drops; SPEC-0006 ("Box records") owns the chunk and record layout. This section is a summary only.

- Counts: 75 chunks in 74 packs (one pack has two), 4,205 records, 18,287 rows and 29,853 plants. Up to ten chunks per world are kept.
- Base: every offset in the chunk, stored ones included, is relative to the chunk header, 16 bytes before the payload (SPEC-0006).
- Record (0x40 bytes), field names as in SPEC-0006: box (x0, x1, z0, z1) at +0x00; pool count (rows) at +0x10; first-row offset at +0x14; key at +0x18; height at +0x1C; zone at +0x20; jitter at +0x24; draw-distance overrides at +0x28 to +0x34.
- Each record is one foliage patch: rows of plants (row at z = z0 + row offset, plant at x = x0 + plant offset, with an RGBA colour per plant; the colour tints only the far sprites, where 128 is full brightness and 64 grey is half, and near models ignore it, SPEC-0060). The key picks the family: grass (keys 0 and 12), flowers (1, 6 and 7), bushes (22 keys, among them the coin bush and the Negg bushes), trees (4, 13 and 25), reeds (8) and rustle and clover beds (10, 11, 14, 23, 24, 26, 27 and 35). Keys 15 and 40 and above build nothing; no record on disc has them.
- The flower models are the registry ids 194 (keys 1 and 7) and 193 (key 6): common-object ids of range 2 (indices 3 and 2), registered by the world pack's own type-8 children (SPEC-0004).
- Each frame a patch is considered only when its zone (+0x20) passes the zone query (SPEC-0006, Binary grids) and the camera eye is within its far distance; within 100 units, and while the game-state code is below 9, its family logic runs.
- Behaviour that other sections refer to: the "six-slot look-ahead" is the bush shot test, and the three-state fall is the cut-reed fall (Behavior, pointers). Key-4 trees put a solid trunk cylinder at each nearby tree (SPEC-0060, Trees).
- Type 16 patches are not trigger volumes of section 13 and share no field with them.

## Behavior

Sections 7.1 and 8 to 10 and the trigger volume tests below give the behavior. Sections 1 to 7, 11, 12 and 13 are layouts. The collision behavior that uses the type-11 primitives (contact pass, sweep, projectile step and move checks) is in SPEC-0018 (Object collision), and the character side is in SPEC-0040.

### Trigger volumes (section 13)

Confidence: Medium for the dispatch and the kind split; High for the fire gates by flag bit (section 13); High that kind 2 never fires in the weapon-contact pass; Low to Medium for the geometry of each kind (as marked).

**Per tick.** The pass is the weapon-contact pass: it runs from the hero's attack routines (both heroes' attack-step and air routines) on ticks whose current clip frame carries event bit 0x01, the hit window (Medium-High). For each trigger object of the attacking actor, the object is skipped unless its state word is 1 or 3 (an intact breakable, section 13) and its volume record is present. Otherwise the volume's kind selects the test:

- **Kind 1, sphere.** The volume is a sphere: centre at record +0x30, radius at +0x3C. Three points form the actor's contact triangle: P0 = the actor's contact point + 0.25 × the actor's up vector, P1 = P0 + r × the actor's contact direction, P2 = P0 + r × the up vector, with r the actor's contact radius (times 0.25 on the blade recoil hook). The volume is hit when any of the edges P0P1, P0P2 and P1P2 has its closest point (clamped to the edge) at a distance from the centre that is at most the radius; the boundary counts. High for the radius field and the inclusive test; Medium for the point sources.
- **Kind 2.** Never hit on this path. The test is a placeholder that reports no hit. High. Other passes route kind 2 to handlers that can fire (section 13, Kind 2 (reconciled)); no cone on the disc has a fire bit.
- **Kind 3, finite cylinder.** The volume is a finite cylinder: radius at +0x3C, full length at +0x24, axis at +0x40 and base point at +0x30 (table above). A stationary actor (zero motion) is tested by a point, and a moving actor is tested along its motion segment, against the finite-cylinder shape of SPEC-0018. The two code scalars have these roles: 0.5 halves the full length, so the cylinder's centre is the base point plus the axis times half the length; 0.25 raises the actor's contact point along its up vector to form the first test point, the same raise as the sphere test's P0 (the blade recoil hook scales the contact radius by 0.25 as for the sphere). The stationary point test uses the contact point raised by 0.5 × the up vector instead. [High for the roles of 0.5 and 0.25; Medium-Low for the exact test calls; the shape is High.]
- **Kinds 4 and 7, rectangle.** The switch in the volume record chooses the test:
  - switch zero or below (static): let Q be the hero contact point, U the contact-up direction, D the contact direction and r = 0.25 × the hero contact radius. The three points are P0 = Q + 0.25 × U, P1 = P0 + r × D and P2 = P0 + r × U. Segments P0P1, P0P2 and P1P2 are tested against the rectangle without margins; any hit counts. The radius scaling is High; the design intent of the stored switch has no effect on play, because the sign alone selects the test (open question 32; Default if unresolved (rewrite choice): keep the stored value and select the test by its sign);
  - switch above zero (moving): the record is treated as a box (corner at +0x30, extent at +0x2C along the +0x40 axis, +0x28 along the +0x50 axis, +0x24 along the +0x60 axis) and tested as five of its faces; a hit on any of the five counts. The five are the two faces perpendicular to the +0x40 axis (at 0 and at its extent), the two perpendicular to the +0x50 axis, and the bottom face (the face at 0 along the +0x60 axis); the top face is never built. They are not fractions of the volume's motion. The block pass uses the same five faces with a cylinder (SPEC-0048, The block's moving-rectangle test; High there, Medium-High for the weapon pass sharing them).
  The rectangle test takes a segment. It hits when the segment crosses the rectangle's plane at a parameter from 0 to 1 and the crossing point lies within the in-plane extents, measured from the rectangle's reference point along each in-plane axis (0 to the extent). No margins are applied. Medium for the test.
  Two rectangle tests exist, each with its own callers (High for the split by caller): this no-margin test serves the weapon-contact pass's static rectangle and several world-geometry callers; the margin test of SPEC-0018 (parameter −0.02 to 1.02, in-plane bounds widened by 0.25) serves the projectile step, the per-cell character contact, the camera and the world-face recoil. Do not swap them.
- Any other volume kind: no test.

The test gives yes or no for the tick. There is no enter or leave edge on this path. This pass never calls the script trigger fire step: a strike reaches a level script only through the object's breakable hook (Apply on a hit, step 1). A strike reaches a script trigger only through the hero's body contact during the hit window, which gives the attack word +0x24 = 1 (Trigger pass below; SPEC-0030). High.

**Apply on a hit.** The apply runs every tick on which the test is true. Its steps, in order:

1. **Script hook.** If a level script registered a breakable callback for the object's script index (+0x58; RegisterScriptBreakable, SPEC-0030), that callback is started as a script thread unless it is already running, the hook's two argument words are stored ((1, 0) on the hero melee and attack-family routes, (0, 1) on Roberta's shot), and nothing else happens: no break and no cue. Without a registered callback the apply goes on to step 2. High (SPEC-0036, Contact routines and cues; SPEC-0048, Break routes).
2. **Element gate.** Let E_t be the object's element byte (+0x5E) and E_a the attacker's Mote kind. The gate passes when E_t is 0 or E_t = E_a. Some handlers also pass on 7: the hero melee route (E_a = the Mote kind of the hero's slot-2 item) and Roberta's shot (E_a = the shot's element kind) pass when E_t is 7, and two of the attack-family handlers that take E_a from their owner pass when E_a is 7 (Nova). One attack-family handler and the boss route have no gate. No entry on disc has E_t = 7. High.
3. **Pass.** The object is a world breakable (a type 29 second-list entry, SPEC-0048), and the pass breaks it: the break effect of SPEC-0048 runs with a piece push of 0.25 along the unit direction between the actor and the object, handed to each piece the break spawns (the actor itself does not move). Then one positioned sound cue plays on the main sound slot at the object's position (+0x30), chosen by the object's material word (+0x5C, table above): cue 143 (0x8F) when it is 2, 145 (0x91) when it is 3, 146 (0x92) when it is 4, and 144 (0x90) otherwise; Roberta's shot always plays cue 144. No effect id is started. The cue plays even when the object did not break (SPEC-0036, Contact routines and cues). High.
4. **Mismatch.** When the element gate does not pass, the reaction depends on the route: on the hero melee route the hero's recoil material word becomes 4 and Tor's attack sub-state 13 is requested (the sword bounce of SPEC-0036, recoil cue 13; Blade recoil below), with no break and no breakable cue, and Roberta has no reaction; the attack-family handlers and Roberta's shot skip silently (no break, no cue, no state change). High.

**Blade recoil (the former "press (use) variant").** This is not a use button: it is Tor's blade striking a solid volume. When the main hero's character contact walk visits a primitive, the hero's hook for that primitive kind also tests the blade against it. There are hooks for the static rectangle, the moving rectangle, the cylinder and the sphere (using the volume tests of this section); the kind-2 hook never hits. A hook acts only when all of these hold:
- the hero is Tor (hero variant word 0); Roberta never recoils;
- event bit 0x01 is active in any of four playback records: main animation layers 0 or 1, the sub-part clip, or the face clip. The query has no selected-layer input; it checks the current frame and, when playback advances over multiple frames, the traversed frames. At a loop boundary exactly at the frame count it checks both the first and last frame. Layer 1 is a second main-skeleton layer; no shipped main clip selects it, so it has no effect on play. The query still includes the layer-1 playback record (Default if unresolved (rewrite choice): include it, as stated). The hero caches this any-record result early in its tick, and the recoil hooks use that cached value;
- the recoil busy gate is clear. It blocks only when the hero target-list count is positive and the first target record status is nonzero. The count is occupied tracked-opponent records (up to 15); the first record status marks a candidate that passes the list's range and direction checks and has a clear path to the hero. It is not a swing counter or recoil latch. The list can be rebuilt, compacted and sorted during target updates, so the first record can change;
- bit 0 (0x0001, solid) of the volume's flags is set;
- the blade test hits (the kind-specific geometry is described below).

**Kind-3 blade test (static inputs and branch).** The hero test cylinder has radius 0.5, full length L = 0.25 × the hero contact radius, base at the hero contact point, and axis along the hero contact-up direction; its centre is the base plus (L/2) × axis. The world cylinder keeps its own radius and signed full length H, with centre at its stored base plus (H/2) × axis. A zero derived motion offset selects the static cylinder-overlap test; a non-zero offset selects the moving-cylinder test. This describes the tested inputs and branch only; it does not establish how often either branch occurs during ordinary play. The fixed 0.5 test radius is distinct from the 0.5 used to place each cylinder centre. The static test preserves signed half-lengths. The moving test also retains signed H: its projected half-extent includes H × |axis alignment| × 0.5 without an absolute-value conversion or clamp. A negative H can therefore reduce or reverse that projected allowance; the final hit still depends on later edge checks. [High for inputs and this gate; exact final negative-H hit cases open]

**Blade triangle against other targets.** For kinds 1, 4 and 7, and the world-face callback, the blade is three line segments rather than the kind-3 cylinder. Let Q be the translation of the hero's first weapon attachment frame, U the stored vector at that frame's first basis row, D the stored secondary direction used by the contact helper, and W the equipped weapon reach (1.5 when no weapon is equipped). The contact helper scales W by 0.6 for this query, giving r = 0.6W. P0 = Q + 0.25 U, P1 = P0 + 0.25 r D, P2 = P0 + 0.25 r U; the tested edges are P0P1, P0P2 and P1P2. Thus the dimensions come from the weapon attachment and reach query, not the hero's body collision radius. For the default W=1.5, r=0.9 and each extension factor is 0.225. Kind 1 tests each edge against its sphere, with closest-point parameter clamped to [0, 1] and distance less than or equal to the stored radius; the stored radius is not enlarged. A static kind-4 or kind-7 rectangle tests those edges against its single plane rectangle with segment parameter [0, 1] and in-plane coordinates from zero through each extent. A moving rectangle tests the same edges against five such rectangles on the box (both faces for each of the first two axes and the bottom face); the top face is omitted. These primitive rectangle tests have no margin. [High for the points, weapon reach factor and attachment-frame sources; Medium for the physical labels of the two stored vectors]

The world-face callback tests the same triangle edges against a rectangle-like record, not a type-11 volume or an abstract cell boundary. It uses the margin rectangle test: segment parameter −0.02 through 1.02; both low in-plane bounds −0.25; first high bound extent +0.25; second high bound extent −0.25. It does not test a volume solid bit. In its ordinary contact-state branch, the hero contact update calls the frame-candidate routine after the grid-listed wall-contact pass and before later contact updates. Its two candidate passes are independent and ordered: dynamic-pool records first, then the world-frame list.

The first source is a 12-slot dynamic face pool at world +0x4270, scanned only when its active count is positive. Construction initializes all 12 slots free with active count zero, and no caller of the add routine is known, so this pass has no known producer. If populated, the pool is scanned in ascending slot order. It skips entries whose activity word at +0x04 is nonzero, accepts a strict center distance below 50 units from the supplied point, and passes the embedded record at +0xC0 to the helper once for each qualifying entry. The loop has no early exit, so the per-invocation offer count is at most the active count and therefore at most 12. The embedded payload is synthesized from a transform into three plane rows and two extents; it is not a pointer or copy of a named type-5 frame or type-11 volume. Its material byte at payload +0x4D is not written by the pool's initializer or add routine.

The second source is the world's type-10 list of ladder and climb-panel faces (SPEC-0006), checked whenever its list pointer is non-null and the contact actor is not one of the excluded actor kinds. Groups are visited in list order; a group must pass its distance/direction prefilter and have a positive record count. Its face records are then visited in stored order. A ladder record (class byte −3) is offered only when its first basis component is below 0.9; climb-panel records and any other classes in the list are offered without that orientation test. Each offered record is passed as the callback's second argument. The type-10 record list contains at most 40 records in any one shipped pack (pack 986); 453 records occur across 63 non-empty type-10 tables. Every shipped type-10 record has 205 in the callback's material-byte field; this maps to material 0 and recoil cue 11. Thus a shipped type-10 table offers at most 40 candidates before the group and orientation filters. The separate dynamic pool has 12 slots but no known producer; only if an unknown caller populated it could the combined storage ceiling rise to 52 offers per invocation. The group prefilter and ladder orientation condition can reduce the world-list offer count below its table size. Neither loop stops after an offer. The callback checks its cached blade event internally, so candidate dispatch itself is not attack-window gated. **The third and main source is the ordinary wall-contact walk.** Each tick, in the same motion-record update, the grid-listed wall-contact pass runs first and the dynamic-pool and type-10 passes above follow it. The wall pass visits every model-set frame item of the character's cell entries (entries whose mask passes the world mask, in entry order, items in stored order), and its per-item routine begins by offering that item to the same face recoil callback, before any wall or ledge handling. The per-item routine returns at once, with no offer, while the character's state is 21, 23, 30, 35, 37, 38, 41 or 47; in every other state each visited item is offered. The callback receives the frame item as its rectangle record and reads its material byte at +0x4D through the face-record map above. So Tor's blade bounces off ordinary wall frame records the blade overlaps, not only off ladders and climb panels; characters other than the hero have a do-nothing callback in that slot. Offer order within one tick is therefore: wall-pass items, then the dynamic pool, then the type-10 list. The three sources are the delivery routes; which face intersects Tor's blade in a given scene or swing is not established. [High for callback geometry, branch order, type-10 candidate identity, forwarding, material/cue mapping and for the wall pass offering every visited item in the listed states; Medium for the dynamic payload's role; unresolved for live candidate overlap and per-swing response count]
Default if unresolved (rewrite choice): offer every wall frame item the hero's wall pass visits to the face recoil test, in the order above, in every state except the eight listed.

The geometry callbacks have no one-hit-per-window latch and do not test the current attack sub-state before requesting recoil. Each qualifying callback clears the charge counter, requests sub-state 13 and writes the recoil material. The event bit is refreshed early in the hero update; the contact dispatch comes later in the same tick, because the wall pass runs inside the motion-record step, which follows the hero update and precedes the animation pass (SPEC-0040, motion-record update order; High). So every callback in a tick sees the event bit cached at the start of that tick, and a later eligible callback in a following tick can submit recoil again. How many times the contact traversal invokes the hook during one swing depends on the live overlap, which needs a running game to count. Default if unresolved (rewrite choice): run the wall pass after the event-bit refresh in the same tick, apply the callback rules above on every qualifying offer, and let the repeated sub-state 13 request within a tick count as one response.

On a hit, the charge counter is cleared, Tor's attack sub-state becomes 13 (SPEC-0036: the sword bounce, hero animation 26, rumble preset 2 for 0.8 s at strength 0.6, and a recoil cue), and the hero's recoil material word becomes M(mode byte at +0x11). The face callback uses the candidate record's byte at +0x4D through a second material map and does not test the volume solid bit. It can be offered more than once in one contact update: both candidate loops continue after each helper call, and the callback has no one-hit latch. Subsequent offers can rewrite the recoil state while the cached event and busy gates remain open. This permits repeated recoil writes, but does not establish that multiple records pass the geometry in a particular scene or that each write produces a separate visible response. Across attack windows, the routine's exact scheduling relative to event-cache refresh and the number of visible responses remain unknown.

The three maps are fixed program constants. In rule form:
- Mode byte m to material M: 0 to 0, 1 to 1, 2 to 4, 3 to 6, 4 to 9, 5 to 2, 6 to 8, 7 to 3; m from 8 to 16 gives m + 10 (18 to 26); 17 gives 10; 18 gives 27; any other byte gives 0. That is 19 mapped values.
- Face-record byte b to material: b + 1 for b from 0 to 3; b for b from 4 to 8; 9 for b = 9 or 10; 8 for b = 11; b − 1 for b from 12 to 18; 10 for b = 19; any other byte gives 0.
- Material to recoil cue (SPEC-0036 plays it): cue 11 for materials 1, 6 and 20; 20 for 2, 23 and 26; 18 for 3; 13 for 4; 15 for 5, 22 and 27; 17 for 7; 19 for 8; 12 for 9, 21 and 24; 29 for 10; 25 for 11; 21 for 12; 22 for 13; 23 for 14, 18 and 25; 26 for 15; 24 for 16; 28 for 17 and 19; any other material, 0 included, gives cue 11.

The hero's material word starts at 6. The element-mismatch recoil of Apply on a hit (step 4) and of SPEC-0048 sets material 4, so it plays cue 13.

### Trigger pass: who is tested, where, and when (High unless marked)

**Actors.** Each play tick the main loop runs these, in this order (SPEC-0026, main loop):
1. The current world's step: its pools, its animated placed objects, then each movable block (the movable block pass), then the region characters of the current world, then those of the exterior buffer. Each region character runs only while its slot entry is active, it is visible and its word +0xE3C is 0 (meaning open); it runs its update and then its trigger step.
2. The selected hero: its update (which holds the hero shot pass) and then its trigger step. Of the two hero objects only the selected one runs; the other hero has no trigger step and fires nothing.
3. The live petpet: its update and its trigger step.
4. The script step of the current world: for each trigger record, resume its callback thread (and free it when it has ended), then apply the decay (SPEC-0030, Layout section 3). The loaded second world runs only the special threads of the script step's first part, so its trigger callbacks are never resumed and its trigger records do not decay while it is not current (SPEC-0030, per-tick script step).
5. The frame counter goes up by 1 at the end of the pass.

World projectiles step with the effects, between step 1 and step 2 (Medium).

**Trigger step of one actor.** The actor's motion record supplies the tested position p (its feet point, motion record +0x50), its body height and its body radius. In order:
1. Cell refresh: the actor's cell pointer becomes the cell, under p, of the grid of the current world (a region character uses its own world when it has one); when p is off that grid, the exterior buffer's grid is tried; when both miss, the pointer is null and steps 3 and 4 do nothing.
2. Placed-object walk: each animated placed object whose reach (its radius word) contains p is tested with p moved into the object's frame, against all six volume lists of the object's inline type 11 header, with the cell list A rules.
3. By the owner's mode, a class method: the hero is mode 0, every world character is mode 1, and the petpet is mode 2.
   - Mode 0 (selected hero): cell list A walk, then cell list B walk.
   - Mode 1 (NPCs and enemies): cell list B walk only. These actors never test cell list A, so they never touch a type 11 volume, a book station or a trigger rectangle.
   - Mode 2 (petpet): cell list A walk only.

**Cell list A walk.** For each volume record in the cell's list A: skipped when its gate mask (+0x1A) is non-zero and shares no bit with the world's gate word (SetWorldCollisMask); otherwise the kind selects the handler and the handler applies its flag gate and test (section 13, Script trigger fire by flag bit). Kind 8 is tested only when the owner is the selected hero.

**Cell list B walk.** For each trigger object in list B: skipped unless its state word is 1 or 3 and its volume pointer is set; skipped when the owner's class predicate says so (the selected hero's predicate is its landing break: when the hero is falling onto the object from above it breaks the object (SPEC-0048) and the object is skipped; every other class passes all objects); no gate-mask test; kind 8 not tested. Otherwise the same handlers as list A run on the object's volume. This walk is also the solid collision of every breakable with a cylinder volume (1,915 of 2,030 volumes on disc): the loader sets such a volume's flags to 9 (bit 0 solid, bit 3 body-contact fire), so a hit fires the volume's number and then pushes the owner out with the slide response, the owner's body being a cylinder of radius 1.0 and height 3.5 (SPEC-0018, Breakable cylinders against characters). A lock chest's only collision is its breakable 0 (SPEC-0048, Lock chests); the unselected hero and the petpet do not run this walk (the petpet walks list A only; the unselected hero runs no trigger step), so they are not pushed (Medium-High).

**Tested shape (Medium-High).** Kind 3: the body cylinder of radius 1.0 and height 3.5 from p (centre p + 1.75 up) against the finite cylinder. Kind 1: the vertical flat-ended cylinder from p up by the body height, with the body radius (not a capsule), against the sphere, by the exact sphere-against-flat-ended-cylinder rule of SPEC-0018 (Finite cylinder tests: beside the cylinder the sphere hits when the axis distance is below R + r; beyond an end face by g it hits when g is at most r for an axis distance of at most R, else when the distance to the rim circle is at most r). Rectangles: section Trigger volumes and SPEC-0018. One test per volume per tick; there is no enter or leave edge in any pass.

**Fire step gate on the actor.** The fire step refuses an actor that is neither hero when the actor is the petpet (or a character whose type id is one of the activatable petpet kinds) and its type id is not in the second petpet kind table (SPEC-0047). In play this refuses every petpet fire, so a petpet never starts a script trigger. Its book-station and boundary-contact side effects do not use the fire step and still happen. NPCs and enemies are not refused. Medium-High for the table reading.

**Hero shot pass.** Inside the selected hero's update, each of the hero's six shot slots that is live, each tick, tests its point and its radius against: the volumes of cell list A at the shot point (all fire bits as in section 13, no bits-clear conditions; a solid volume, bit 0, ends the shot with an impact cue); the trigger objects of cell list B at the shot point (solid test and the breakable hook only); the movable blocks; the inline volumes of animated placed objects within 150 units. A shot can fire several triggers in one tick and the same trigger on several ticks.

**Movable block pass.** A movable block tests its own shape against the volume records of the **selected hero's** cell list A, not its own cell: a block fires only while the hero stands over a cell that lists the volume. The test runs only while the block is moving and on the tick its moving flag clears (n + 1 ticks per push); an idle block does not test, it repeats the volume it last hit on every tick. The block's shape is a vertical finite cylinder with its base at the block position, radius S and height H (the entry's +0x10 and +0x14; for the moving-rectangle kind the radius is 0.6 S). Kind 7 rectangles are not tested on this pass. The shapes by volume kind, the repeat rule and test vectors are in SPEC-0048 (Movable blocks, Trigger volumes).

**World projectile step.** Only an enemy-pool projectile with its breaks-objects flag set takes this step, and in the whole game only the Faerie Sisters' alternate-loop ball sets that flag (SPEC-0029, SPEC-0044 pack 1174). Such a projectile tests the volumes of the cell under it; on a fire it ends. It fires only indices that the act-1 trigger map holds (DFS_Act1_SetTriggerData, SPEC-0030), and only pack 1174 writes that map.

**Reach on the disc (High).** Over the 608 resolved (pack, index) registrations: 594 have a volume that passes the selected hero's body-contact gate and that at least one grid cell lists (list A, or list B for a trigger object), or a ForceTriggerScriptIfNotRunning call; 7 use a computed index; the other 7, pack 1174 triggers 5, 22, 23, 24, 36, 37 and 38, are boxes with flags 0x0120 and fire only from hero shots, movable blocks and world projectiles (SPEC-0044, the two-hit object and the floor tiles). No registered trigger lacks a volume.

### Riding, carrying and collision with moving placed objects (pointer)

Moving placed objects (section 7.1) have no proximity carry of their own. Three rules of SPEC-0018 cover them:

- **Ground attach of a character** (SPEC-0018). The ground probe maps the character into the object's rest frame, the object becomes the character's ground object, and from the next tick on the character is moved by the object's per-update translation change (+0x150) plus a rotation term, with x and z clamped to 1.5 per tick. The vertical follow is done by the motion-record integrator (SPEC-0018, SPEC-0040). This is the platform and lift riding rule.
- **Character versus moving placed objects** (SPEC-0018, Who walks what, item 4). For each object of the world's placed-object list whose active byte (+0x1B6) is nonzero, which has an inline type-11 header (primitive set, +0x1D8), and whose reference centre (+0x180) is closer to the character than the object's radius (+0xBC; squared values, strict), the character's position is taken into the object's rest frame, the object's six primitive lists are walked through the dispatcher, and the push is taken back to world space. This walk carries nothing; the carry of a rider is the separate ground attach, which names the object through the character's platform pointer (+0x190; SPEC-0018, Ground attach of a character, The platform pointer; R-0205).
- **Projectile versus placed objects** (SPEC-0018). The same mapping with a fixed gate of 150 units (squared 22,500, strict) from the projectile; the projectile's position is restored afterwards. Nothing is carried.

Settled (High): a platform carries a character only through the ground attach of SPEC-0018 (Ground attach of a character), with the conditions stated there: the same ground object as on the previous tick, the rider at most 0.25 above the ground (or climbing), x and z clamped to 1.5 per tick each, and the vertical term only with the record flag that SPEC-0018 names. The 150-unit gate is the projectile test and the radius walk only pushes.

### Pack 1299 looping-door puzzle (High for the rules)

The only six-state "platform" is a puzzle that is built at load when the world has a kind-7 record of class 500 (pad A) and one of class 501 (pad B), both in the +0x3C list, and a display node with id 401. Only pack 1299 qualifies: its kind-7 records 2 (class 500, base height 15.15) and 3 (class 501, base height 33.16), the same-world pads of section 6.

Puzzle record at build: active; state s = 0; pads A and B; mask target = display node 401; accumulator 0; object range 0 to the placed-object count minus 1; spin rates of placed objects 0 to 5: 15, −15, 22, −18, 10 and 18 degrees per second; midpoint height M = (15.1487 + 33.1583) / 2 = 24.1535 (stored at +0x68 of the puzzle record). The accumulator is zero only at program start: the world reset clears the state, the active word, the pad references and the mask target, but not the accumulator, so a later visit to the world starts with the old total (see Solution).

**Pad trigger (High).** Only the selected hero can fire a pad. Each pad is a rectangle with a corner point P, a through axis F (the plane normal), a width axis D and an up axis Q (0, 1, 0), a width w along D and a height h along Q (the plane layout of SPEC-0006, Teleport plane list). For a point p, the plane coordinates are f = F · (p − P), d = D · (p − P) and q = p.y − P.y. Let p be the hero's reference point (its position plus a small offset vector that is zero in ordinary movement) and r = 1.175 and H = 3.5 the hero's radius and height. The pad fires on a tick when all three hold: −10 < f ≤ 0 (the hero is on the back side of the pad, within 10 units), −H ≤ q < h, and −r < d < w + r. It fires on each such tick until the hero leaves the zone or the landing moves it out. Pack 1299 values: both pads have w = 20.987 and h = 10.687.

| Pad | P | F | D |
| --- | --- | --- | --- |
| A (class 500) | (−129.768, 15.149, −78.791) | (0.7370, 0, −0.6759) | (−0.6759, 0, −0.7370) |
| B (class 501) | (−143.953, 33.158, −94.258) | (−0.7370, 0, 0.6759) | (0.6759, 0, 0.7370) |

B is A turned half a turn about the vertical axis through the common centre line: B's corner is A's corner plus w times A's D (in x and z), and B's axes are A's reversed. So the two pads cover the same footprint on the ground (centre at x −136.86, z −86.52), 18.01 apart in height, and f_B = −f_A and d_B = w − d_A for any point. A fires on the downhill side of the line (f_A ≤ 0), B on the uphill side (f_A ≥ 0).

**Pad crossing (High).** The hero touches one pad T; the destination X is the other pad. There is no teleport when s > 5, when s = 0 and X = A, or when s = 5 and X = B; then nothing is written (the state, mask and offsets stay as they are) and the hero walks on. Otherwise the hero is moved as follows, then the step below runs with dy.

- Let H be the hero's position, d and f its plane coordinates on T, and a the hero's height above the ground: a = H.y − G1, where G1 is the ground height at (H.x, H.z) found with a ceiling 8.0 above H.y (SPEC-0018: the highest ground at or below the ceiling).
- The landing point L = X's corner + X's D × (w − d) − X's F × f, taken in x and z. X's D and F are horizontal, so L.y is X's corner height. The new height is G2 + a, where G2 is the ground height at (L.x, L.z) with a ceiling 8.0 above X's corner height.
- The hero's facing becomes −(facing · F_T) F_X − (facing · D_T) D_X, normalised. The hero's saved previous position is set to the new position (no speed is carried over), and the ground cell is refreshed.
- dy = new height − old height. The camera moves up by dy only; its x and z do not change.
- On this disc L has the same x and z as H, and the facing is unchanged (the pads share a footprint and are reversed). A crossing is therefore a pure vertical move to the ground of the other lap of the staircase at the same x and z, keeping the height above that ground. It is not the difference of the pad centres: that would add 18.01 to the height, but the real dy is the difference of the two floors at that spot (+16.83 and −17.20 at the pad line, see the test vector).
- Why the landing does not fire at once: the landing flips the side, so f on the destination is −f (positive when the hero was past the line), which fails −10 < f ≤ 0; and the origin pad's height band no longer contains the hero, who is about 17 units above or below it. The next crossing needs a deliberate step back across the line (see Solution).

Then the step runs with dy:

1. **State move.** The type of state s is the 32-bit word at ELF data address 0x2EB410 + 4s. On disc the words for s = 0 to 5 are 1, 0, 1, 1, 0, 1, and word 6 is 0. Type 0 needs X = A and type 1 needs X = B. On a pass s += 1 (the completion flag is set when s > 5). On a fail s −= 1 when s > 0.
2. **Display mask.** The mask word of display node 401 becomes 1 << (i − 1), with i = s + 2 when the type of the new s is 0 and i = s + 8 when it is 1; that is 1 << (s + 1) or 1 << (s + 7). An i below 1 gives 0. The mask at build is 0x80 (the build runs the step once with dy = 0 and no destination, which leaves s = 0). What the mask does: display node 401 is the root of its own small display tree, kept apart from the level display tree and drawn every frame the world is drawn. It holds twelve variants of one small object (16 mesh leaves each, material group 8, box origin (−126.6, 27.1, −74.9), size 6.5 by 7.3 by 6.5), as twelve layer-flag nodes with the single flag bits 0x2, 0x4, 0x8, ... 0x1000. The tree is walked with the usual layer rule (a type-4 node is skipped with its subtree when its layer flags are nonzero and share no bit with the draw mask, SPEC-0046), and the draw mask for this walk is the mask word the puzzle writes, not the world's display word (SetWorldDisplayMask has no effect on it). So exactly one variant shows at a time: s = 0 to 5 show the variants 0x80, 0x4, 0x200, 0x400, 0x20 and 0x1000. No variant has the flag 0x1. The tree is not moved by the accumulator. Which picture each variant holds is not settled (Low). Default if unresolved (rewrite choice): draw each variant's stored meshes as they are; the variant choice above is the whole rule.
3. **Offset.** accumulator += dy, and every placed object gets the position offset (0, accumulator, 0).
4. **Characters.** On a pad crossing, the step walks the current world's active character list from entry 0 to the first null entry, without a hero, boss or class filter. It moves an entry by (0, dy, 0) only when dy > 0 and its height is strictly below M, or dy < 0 and its height is strictly above M. An entry exactly at M does not move. This is a crossing-time adjustment; the per-tick accumulator offsets placed objects only. In pack 1299 the list is empty (Open question 94), so no character moves.

Per tick while the puzzle is active: after the selected hero's motion-record tick, accumulator += that tick's vertical displacement (current position minus the saved tick-start position); placed objects 0 to 5 spin about the vertical axis by rate × dt degrees, converted to radians with the factor 0.017453294 (ELF data address 0x2DB480); every placed object gets the offset (0, accumulator, 0). This ordering is established for the selected hero's main-loop path; no other character's tick moves the accumulator, so the order against other characters' motion-record ticks has no effect on the puzzle (Default if unresolved (rewrite choice): run the step after the selected hero's tick).

Solution: crossings to B, A, B, B, A take s through 1, 2, 3, 4 and 5 (touching pads A, B, A, A, B in turn); at s = 5 touching pad A (a crossing to B) is not teleported, so the hero simply walks on down past pad A to the exit. Touching pad B at s = 0 (a crossing to A) is also not teleported. State 6 cannot be reached from play. A touch that fails the type test still teleports the hero and then lowers s by 1.

Resets (High for what is cleared; Medium that the exit load releases the world). Nothing is reset when the hero walks past pad A at s = 5, because the gate refuses before any write: the state stays 5 until the world is released. The world reset (run when the world is released, for example by the load that follows the exit door) sets the state, the active word, both pad references and the mask target to zero. The next build starts at s = 0 with mask 0x80, rezeroes the spin angles of objects 0 to 5, and re-activates every placed object, but it never clears the accumulator: the accumulator is a program-wide static, so it keeps whatever total the last visit ended with (the sum of the hero's vertical moves while the puzzle was active, about the exit height minus the spawn height, near −54 after a full run) and the placed objects start offset by it. This only moves the placed objects (a visual quirk); no walkable ground depends on them in pack 1299, whose five placed objects carry no height patches (the staircase ground is the pack's static ground). Default if unresolved (rewrite choice): reproduce it (keep the accumulator between visits) only if the placed objects are ever drawn; otherwise zero it at build.

### Cut-reed fall (pointer; High)

The three-state routine earlier read as a mover, and later as a launched-object after-flight fall, is the fall of the cut piece of a reed (key 8) of the type 16 foliage patches. It is not a platform, and it runs in play, for the 24 reed records in packs 909, 934 and 1056. SPEC-0060 (Reeds, K, and the cut-piece fall) owns it. In brief: state 0 moves the piece by 0.05 × its velocity per tick until it is more than 0.5 from its anchor; state 1 sinks it 0.07 per tick until its height is below the ground probe result (ceiling = height + 8.0) minus 1.25 (SPEC-0018, Ceilings supplied by callers); state 2 turns its up axis toward its velocity, up = normalize(0.965 up + 0.035 velocity), until |up.y| ≤ 0.001.

The keyed walk earlier listed in this section is the effect trail-strip renderer of SPEC-0034 (Ribbon and trail packets); it is not a mover track.

### Six-slot look-ahead: the bush shot test (pointer; High)

The "six-slot look-ahead" is the shot test of the foliage bushes (SPEC-0060, Bushes, B, Cut triggers). The six slots are Roberta's six shot slots (SPEC-0036, Hero shot slots); only armed slots are tested. The test point is the bush position + s × the bush's up axis, with s = the boot data word 0x2EB340 = 0.0, so the point is the bush base. A slot cuts the bush when the distance from the test point to the shot is strictly below the shot's radius + m, with m = the boot data word 0x2EB344 = 1.5. The hit handler is the bush cut and drop of SPEC-0060 (Bush drops). It is not the trigger-volume proximity kind of section 13.

A separate character-construction path writes six per-slot values into nested auxiliary records at offsets 0x0, 0x2C0, 0x580, 0x840, 0xB00 and 0xDC0 (stride 0x2C0), and resets the last two to zero. These records are distinct from the shot slots used above: those are a separate per-hero array at +0x10D0 with stride 0xA0, and the bush test reads their armed flag, position and radius (SPEC-0036, Hero shot slots). Resolved (open close-out, Medium-High): they are the hero's six effect records with the same offsets, each holding an effect sub-object with five child links (SPEC-0034, effect record bank reset).

### Contact-time crossing solver (Medium-Low)

A moving point is tested against a circle in a plane, to find when it crosses the circle during a segment of motion. The circle radius is the sum of the two radii involved, and the comparison is made on squared values. The crossing times are the roots of a quadratic in the segment parameter. The solver returns zero, one or two crossings, with the crossing points, and only crossings on the segment are kept.

- If the motion is parallel to the plane (segment length below 1e-5), the roots are not used. The result is decided by whether the point is inside the circle.
- A near-parallel case is tested separately against 0.99999. It can return two crossings.

The solver is never run. Nothing in the boot program or in any of the 173 level modules calls it, jumps to it, stores its address in a table or builds its address, so no sweep rule of SPEC-0018 or contact pass of any character uses it. The clean side must not implement it; the sweep and contact rules of SPEC-0018 are the only crossing tests in play (Open questions 45 to 48). High for "never run"; the in-plane axes and any segment scaling stay unspecified because they cannot matter.

### Pool maintenance (per tick, current world only)

Confidence: Medium for the release rule and the order of the passes; Low for the per-record update and for the sub-list rules.

Each tick the owner maintains its pools. Only the current world's pools are maintained. Other worlds keep their links.

1. **UI gate.** When the UI flag at +0x44 of the UI block is set, no pass runs. The stored scene id at +0x6A (step 4) is written to the world's scene id field if it is 0 or more. Otherwise the pass continues with step 2.
2. **Scene id before.** The current scene id is read.
3. **Pool A.** Each link of pool A is handled by the release rule below, and then a per-record update runs over the pool. The update acts on the record's state word at +0x24 of the 0x48-byte record (offset +0x10 of the trailing sub-record at +0x14): it does nothing when that word is 0; otherwise, when the sub-record's stamp word (record +0x2C) is 0 or more and below the frame counter minus 1, a word equal to 3 becomes 0 and any other nonzero word becomes 3. So a stamped record steps to 3 and then to 0 on the following tick, once its stamp is more than one tick old (Medium; what the state values mean to the object that owns the record is not known). The scene sub-lists get the same update in step 5.
4. **Scene id after.** The scene id is read again. If it changed during the passes so far, the new id is stored at +0x6A. If it did not change, -1 is stored there.
5. **Other passes.** Pools B and C, the single link the owner holds, pool D, pool G, pool E, pool F, and the scene list with its sub-lists are handled by the release rule, in that order.
6. **Scheduled job.** The word at +0x88 of the world record is the level module's Update entry point (the entry stored with Init and Exit; SPEC-0030). When the world is the one being maintained, that entry is set and the owner's pending-job word is zero, one job is scheduled and its handle is stored in that word. The job calls the Update entry once, then ends; this is the Update thread of SPEC-0030 (High for the call, Medium for the owner's handling of the pending-job word once the job has ended).

Release rule. A link that holds no object is skipped. Otherwise it is released (it is released back to its pool and the link is set to 0) when either of these holds:
- the object is gone, meaning that the object lookup finds nothing;
- the object's first word, its state word, is nonzero.

Seven pools and the single link are handled each tick, so the release rule covers seven lists. Synthetic case: a pool A link whose object's state word is 0 and still exists is kept; the same link is released once its state word becomes 1.

## Edge cases and error handling

- Trigger objects whose state word is neither 1 nor 3 (a disabled or broken breakable) are never tested, even when they have a volume record.
- Volume kind 2 never fires on the weapon-contact pass: its test is empty there. The cell list A walk, the shot pass and the projectile step can fire a kind 2 volume with bit 3 (section 13); no cone on the disc has a fire bit. The cone routine of SPEC-0018 is reached by the cell list walks, not by the weapon-contact pass (open question 76).
- Volume kinds other than 1, 2, 3, 4 and 7 are never tested.
- No enter or leave edge: an overlap applies on every tick it lasts. When a script hook is registered for the object, every such tick goes to the hook (the callback is not restarted while it is still running). Reproduce the repetition; do not add a one-shot latch that the disc does not show.
- Element byte 7 is not a general wildcard: it passes on the hero melee route and on Roberta's shot; on an attack-family handler it passes only when the attacker's kind is 7. No disc entry holds 7.
- Blade recoil needs Tor, the hit window and a solid volume (flag bit 0): a blade hit by Roberta, outside the hit window, or on a volume with bit 0 clear gives no recoil. The hit-window query checks bit 0x01 on any of the four playback records (main layers 0 and 1, sub-part or face); it has no layer-0-only restriction (SPEC-0005, Table B frame bits).
- Mode bytes above 18 give material 0, which plays recoil cue 11.
- A moving rectangle (switch above zero) is tested as five faces of the plate box (all but the top) with a test point of radius 0.6 S; a hit on any face counts (SPEC-0048, The block's moving-rectangle test). Do not test it as a single rectangle.
- The rectangle test for the static case applies no margins; do not apply the SPEC-0018 margins to it.
- Door confirm left set (quirk): a script door never clears the hero's door-confirm flag, so after a confirmed script door the next default door travels on its first contact tick with no prompt (SPEC-0006, Edge cases). Medium-High.

- Type 28: 14 quaternions are not unit length within 1e-3. Use them as stored; do not renormalise silently.
- Type 30: do not use the table values to find records. Read records in sequence. A record count of 0 in A, B or C is valid. Negative counts were not found.
- Type 39: a spawn whose effect id has no definition is reset and plays nothing. The original logs this case. Medium.
- Type 1000: an index past the entry count gives no entry.
- Type 11: the loader clears byte +0x4F of each 0x50-byte record of the fourth list when it loads the chunk. Reproduce it: the byte is the start-delay counter of the ambient sound loops (SPEC-0031).
- Type 11 cylinders: resource 721's count of 10 extends 32 bytes beyond the chunk payload under the established pointer-plus-0x20 layout. The type-11 loader re-bases pointers but does not change this count; whether later readers bound the final partial entry is a run-time memory question with no disc data to decide it. Default if unresolved (rewrite choice): read only the complete entries that fit in the chunk and treat the partial entry as absent.
- Type 11 spheres and cones: the 0x40-byte records are read by their count. An equal-extent 0x50-byte record is not a sphere, even though its extents match.
- Type 11 boxes: one extent of 0.0 on disc (pack 684). A zero extent gives an empty range along that axis.
- Type 11 boxes: extents can be negative (section 6.1, extent signs). Do not assume a positive extent; runtime use of a negative extent is settled (it stays signed and narrows the accepted interval; open question 77); only the authors' reason for storing one is unknown, and the rewrite does not need it.
- Type 11 cylinders: one record (resource 1394) has a negative full length (−14.47). The kind-3 contact path preserves the sign: its centre is base + (H/2) × axis, and the overlap routine uses signed half-lengths without taking an absolute value or clamping. This is not a conventional positive-height cylinder interpretation. Whether resource 1394 reaches this contact path in play remains unestablished (open question 24); the blade-recoil branch is separate (open question 31).
- Type 11 primitive lists: the loaded record header is at the list pointer; on-disc shape data begins 0x20 bytes after that pointer and proceeds at the list's record stride. For resource 721, the declared ten-cylinder array is 32 bytes longer than the payload; only nine complete shapes and 48 bytes of the tenth are present. Do not infer a reader clamp from the loader's pointer rebase alone.
- Type 29: an object whose inline child does not have type 5 is skipped. Parts with an out-of-range entry index are skipped.
- Type 29 draw gate: a gate byte of 0 and the reset value 1 do not mean "visible". Keep the two-sided rule of section 10.
- Type 29 state: a state of 3 or 4 with a non-zero gate byte is never drawn. States other than 3 and 4 are never seen in the draw path.
- Type 29 far state: a placed object is not in the object lists of SPEC-0046, so it has no far flag and no 350-unit cull; the draw gate of section 10 is its only draw test. The designer draw-distance values are not applied by the draw path.
- Rotation steps 0.0006 and 0.0009: do not apply them to placed objects; they are sky dome layer steps (section 10.1, SPEC-0046).
- Lights: type 30 light sets feed only the light slots of character light contexts (SPEC-0046, Light slots and Character lights); never apply them to world geometry, which is unlit. Type 3 entries have a falloff, and between 0.9 r and r they cross-fade a displaced directional light instead of taking its slot (SPEC-0046, Character lights); type 1 entries are ambient and their direction must be ignored. Do not load resource 527; use the built-in default rig of SPEC-0046 instead.
- Type 40: the entry fields of section 11 are known; kinds 0, 7 and 12 have no built-in behaviour and must not be given one. Two packs that share a key (1096 and 1122, key 185) share one opened mark: opening one chest marks the other as opened. Type 41: the zone output is used as stated in section 11; the render scalar's meaning is Low, so the clean side should pass it through to the renderer without depending on it.
- Resource 527 is not read by the game. A tool that inspects it should stop at the end of the text lines; the stray byte after the ambient line is not data.
- Moving placed objects: no proximity carry exists. Riding is the ground attach of SPEC-0018; the radius (+0xBC) walk only pushes characters and the 150-unit gate only tests projectiles (Behavior, pointer section).
- Bush shot test (the former six-slot look-ahead): only armed shot slots are tested, the distance test is strict, and key-5 bushes have no shot test (SPEC-0060).
- Object accessors: a list read beyond its count returns zero; do not read past the count.
- Built-in pools (section 15.1): they stay empty on disc data; do not fill them from any disc record. The statue turn, the plate press, the tween nodes and the grab-door hold are never seen in play. The three-state fall is not one of them: it is the cut-reed fall, which runs (SPEC-0060).

- Animated placed objects: an object in its disc state (3) does not move or draw until a script calls InitObjOn (section 7.1). Do not start any motion at load.
- Animated placed objects: the reference centre adds the mesh corner without rotation, so it is off for rotated objects. Reproduce it; the radius walk of SPEC-0018 uses this centre.
- Animated placed objects: the rotation lerp is not normalised. Reproduce it.
- Animated placed objects: the first tick after a loop wrap sets the root to s[0] × f instead of adding to it (section 7.1, step 4).
- Pack 1299 puzzle: the state can move back. A state above 0 whose crossing fails steps down by one.
- Pack 1299 puzzle: state 6 cannot be reached, because at state 5 a crossing to pad B is not teleported; word 6 of the state table (0 on disc) is never used in play.

## Test vectors

- Type 39 record with count 2: the second record starts at 0x10 + 0x90 = 0xA0 from the payload start.
- Type 39 zone gate: an active spawn with zone number 3, while the camera stands in zone R and cell (R, 3) of the queried block is 0, is stopped on that pass; with cell (R, 3) = 1 it is updated.
- Type 30 record with A = 3, B = 0, C = 1: size = 0x20 + 0x50 × 4 = 0x120 bytes, already a multiple of 16. The next record starts 0x120 bytes later.
- Type 30 record with A = 0, B = 2, C = 0: size = 0x20 + 0xA0 = 0xC0 bytes.
- Type 28 record with count 0: its pointer-like word is 0 (true for all 1,681 such records on disc).
- Draw gate: an object with state 3 is drawn only if its gate byte is zero. An object with state 0 and the reset gate byte 1 is drawn, if its zone query passes.
- Placed object far from the view point (SPEC-0046 far and cull values do not apply): at 49.9, 50.1 and 350.0 units it is drawn the same way whenever its draw gate passes; there is no far flag and no distance drop.
- Lighting examples: see section 8.
- Rectangle record layout example: the synthetic box of section 6.1 has w = −10 at +0x50 and w = 5 at +0x40; a point (10, 0, −5) lies at the reference point (+0x30).
- Sphere record layout example (synthetic): a sphere of radius 2.0 centred at (1, 2, 3) has kind 1 at +0x10, the all-ones tag at +0x20, +0x24, +0x28 and +0x2C equal to 2.0, centre values at +0x30, +0x34, +0x38, and +0x3C equal to 2.0. A chunk with sphere count 1 must contain exactly one such 0x40-byte record, starting at the sphere list pointer.
- Cone record layout example (synthetic): kind 2 at +0x10, height 5.0 at +0x24, radius 2.0 at +0x28 and +0x2C, and a copy 2.0 at +0x3C. The record is a cone because its header pointer and count place it on the cone list; the extents alone do not decide it.

- Record walk (synthetic): a cylinder list pointer of 0x200 with count 3 gives records at 0x200, 0x250 and 0x2A0; each record's kind byte is at +0x10 (0x210, 0x260, 0x2B0) and its body tag at +0x20 (0x220, 0x270, 0x2C0).
- Door (synthetic): a kind-7 record with flags 0x0002, number 2, switch 0.0. The current world is the exterior, no callback is registered for door 2, and the current pack's tag-0x0E list has entry 2 = 5. The cell under the hero names a transition pack: prompt 13 ("Enter") shows from the tick after first contact, and on the tick of a Square press the hero is placed at door 5 of the transition buffer, which becomes current. With a callback registered for door 2, the callback runs instead and receives +0x14 = 2 and +0x1C = 1.
- Door (synthetic): the same record with the switch at +0x2C set to 1.0 takes the moving rectangle test, not the static handler, so the door rule does not apply (no door on the disc has a switch above 0).
- Rectangle branch threshold (synthetic): with all other fields unchanged, +0x2C = 0.0 and +0x2C = −0.01 select the static rectangle test; +0x2C = +0.01 selects the moving test. This is the runtime branch rule for both kinds 4 and 7, regardless of how the stored value is named in the layout.

Trigger volumes (synthetic; the actor, object and volumes are invented for these examples):

- Sphere, kind 1: centre (0, 0, 0), radius 1.0 at +0x3C (the value 5.0 at +0x34 is ignored). Triangle edge from (1, 0, −2) to (1, 0, 2): closest distance 1.0, a hit (the boundary counts). Edge from (1.01, 0, −2) to (1.01, 0, 2): a miss.
- Kind 2, weapon-contact pass: a point lying on the segment of the volume gives no hit.
- Static rectangle, kinds 4 and 7 with switch 0: normal (0, 1, 0); in-plane axis 1 (1, 0, 0) with extent 10; in-plane axis 2 (0, 0, 1) with extent 4; reference point at the origin. Segment from (5, 1, 2) to (5, −1, 2) crosses the plane at parameter 0.5, at (5, 0, 2), which lies inside: hit. Segment from (12, 1, 2) to (12, −1, 2) crosses at x = 12, outside the extent of 10: no hit. The second result holds with or without a margin of 0.25.
- Finite cylinder, kind 3 (synthetic layout example): radius 1.0 at +0x3C, full length 3.5 at +0x24, base point (0, 0, 0) at +0x30, axis (0, 1, 0) at +0x40. The centre used by the overlap test is the base plus half the length along the axis: (0, 1.75, 0). A stationary actor at the centre is inside the volume.
- Contact cue (synthetic): a breakable with material 2 gives cue 143 (0x8F), 3 gives 145 (0x91), 4 gives 146 (0x92), and 0, 1 or 5 give 144 (0x90).
- Break push (synthetic): actor at (0, 0, 0), breakable at (3, 0, 4). The unit direction is (0.6, 0, 0.8); the break hands the push 0.25 along it to each piece. The actor stays at (0, 0, 0).
- Element gate: element 0 passes for any attacker. Element 5 hit by Tor's melee with a slot-2 Mote of kind 4: no pass, Tor recoils with material 4 (attack sub-state 13, recoil cue 13), no break and no breakable cue. The same element 5 hit by the fixed kind-4 family handler: nothing happens. Element 7 hit by the hero melee: pass (break when in state 1, cue by material).
- Blade recoil radius: an actor contact radius of 4.0 gives a blade test radius of 1.0 (4.0 × 0.25).
- Recoil material and cue: mode byte 4 gives material 9 and cue 12; mode 17 gives material 10 and cue 29; mode 0 gives material 0 and cue 11; mode 19 gives material 0 and cue 11. Face byte 11 gives material 8 and cue 19.
- Recoil gate: Tor in the hit window against a volume with flags 0x0008 (bit 0 clear): no recoil. Flags 0x0001: recoil (attack sub-state 13). Roberta against flags 0x0001: no recoil.
- Repetition: the same overlap on ticks 100, 101 and 102 runs the apply three times. With a registered script hook, all three ticks go to the hook; the callback starts on tick 100 and is not restarted while it is still running.

Projectile versus placed objects, 150-unit gate (synthetic; object reference centre at the origin, projectile position given; SPEC-0018):

- Projectile at (149.0, 0, 0): squared distance 22201, less than 22500, so the object is tested.
- Projectile at (150.0, 0, 0): squared distance 22500, not less than 22500, so it is not tested.

Animated placed objects and the pack 1299 puzzle (synthetic unless stated):

- Playback: F = 5, translation y samples (0, 1, 1, 1, 1), speed 1. The advance is a = 0.25 keys per tick and D = 4 / 15 = 0.2667 s = 16 ticks. The translation change +0x150 y is 0.25 on every tick, the total is 4.0, and state 2 becomes 8 on tick 16.
- Duration: F = 300 gives D = 19.933 s, 1,196 ticks at speed 1. At speed −1 the object starts at time D and ends at time 0, with the absolute translation rule on every tick.
- Puzzle crossings (destination pads): B, A, B, B, A give s = 1, 2, 3, 4, 5 and display masks 0x4, 0x200, 0x400, 0x20 and 0x1000. At s = 5, a crossing to A gives s = 4 (with a teleport); a crossing to B gives no teleport. At s = 0, a crossing to A gives no teleport; a crossing to B gives s = 1.
- Puzzle spin: placed object 2 at 22 degrees per second turns 0.3667 degrees per tick.
- Puzzle character shift (M = 24.1535): with dy = +1, an entry at y = 24.0 moves to 25.0, while y = 24.2 and y = M do not move; with dy = −1, y = 24.2 moves to 23.2, while y = 24.0 and y = M do not move. The per-tick accumulator does not shift characters.
- Pad pair geometry (pack 1299, real values): the hero at pad A's centre line (d = 10.4935) with f = −0.3 stands at x −137.082, z −86.321; the same point has f_B = +0.3 and d_B = 10.4935. The ground there is 53.156, 35.077, 18.244 and 8.379 from the top lap down. At f = +0.3 (x −136.640, z −86.727) it is 53.469, 35.466, 18.266 and 8.379.
- Pad crossing, real values (the hero stands on the ground, a = 0): touch A at (−137.082, 18.244, −86.321) with s = 0: the destination is B, the ceiling at the landing is 41.158, the ground there is 35.077, so the hero lands at (−137.082, 35.077, −86.321), dy = +16.832, s = 1, display mask 0x4. At the landing f_A = −0.3, f_B = +0.3 (not in B's zone) and the height above A's base is 19.93 (above A's band h = 10.687), so nothing fires. Walk back 0.6 to f_A = +0.3 (y 35.466): B fires (f_B = −0.3, height above B's base 2.31, inside its band); the destination is A, the ceiling is 23.149, the ground is 18.266, so the hero lands at (−136.640, 18.266, −86.727), dy = −17.201, s = 2, mask 0x200; there f_A = +0.3 and the height below B's base is −14.89, so nothing fires.
- Pad crossing, wrong order: at s = 1, touching A (destination B, the state type wants A) still teleports the hero up, then s = 0 and the mask is 0x80. At s = 5, touching B (destination A) teleports down and gives s = 4.
- Pack 1299 walking route from the top of the staircase to the exit (computed from the pack's ground and pad data, not played; walls and rails of the stair are not tested, so treat the path as a centre-line, Medium). Staircase: a spiral about the vertical axis through x −127.99, z −76.38 (the common x and z of the five placed objects), walkable band radius 7.7 to 24.3, rising 17.5 per turn; going down means turning in the direction of decreasing angle atan2(z − z0, x − x0). The pad line is at angle −131.18 degrees; on the centre line (radius 13.47) it is the point x −136.86, z −86.52. Its floors, top lap to bottom, are y 53.45, 35.45, 18.25 and 8.38 (the bottom is a flat landing). Pad B's height band is y 29.66 to 43.85 and pad A's is y 11.65 to 25.84, so on the 35.45 lap only B can fire and on the 18.25 lap only A. Pad B's zone is the uphill side of the line (up to 42 degrees of arc), pad A's the downhill side. Lengths below are along the centre line.
  1. Start at the pack's first entry (−106.3, 62.3, −51.5); the ground there is the flat top landing at 61.64 (the hero drops 0.66). Walk 19.8 to the head of the spiral at (−117.67, 61.64, −67.72). The top door (door 0, y 61.89) is shut (it only shows a message) before Act 3.
  2. Descend the first lap: (−119.33, 57.35, −86.70) at 41.0 from the start; pad-line pass at y 53.45 after 60.0 (no pad can fire at that height); (−138.13, 48.85, −67.51) at 81.2; (−119.12, 44.47, −66.24) at 102.4; (−117.85, 39.90, −85.25) at 123.5.
  3. State 0: second pad-line pass at y 35.45 after 144.7. Pad B's zone is entered 42 degrees of arc before the line; B fires but the destination is A with s = 0, so nothing happens and the walk goes on. Continue (−138.13, 30.77, −67.51) at 165.8; (−119.12, 27.05, −66.24) at 187.0; (−117.85, 22.69, −85.25) at 208.1.
  4. Crossing 1: the third pass at y 18.25 after 229.3. Pad A fires on the first tick with f_A ≤ 0 (the sample point f_A = −0.3 is x −137.082, z −86.321); the hero is lifted to y 35.08 at the same x and z (s = 1, mask 0x4).
  5. Crossing 2: step back uphill 0.6 across the line (f_A = +0.3, x −136.640, z −86.727, y 35.47): pad B fires, the hero drops to y 18.27 (s = 2, mask 0x200).
  6. Crossing 3: walk down 0.6 across the line again: pad A fires, the hero rises to y 35.08 (s = 3, mask 0x400).
  7. Crossing 4: this lap is on the upper floor, where pad A cannot fire, so walk down one full lap (84.6): (−138.13, 30.8, −67.5), (−119.1, 27.1, −66.2), (−117.9, 22.7, −85.3), then the pad line at y 18.25: pad A fires, the hero rises to y 35.08 (s = 4, mask 0x20).
  8. Crossing 5: step back uphill 0.6: pad B fires, the hero drops to y 18.27 (s = 5, mask 0x1000).
  9. Walk down past the pad line: pad A fires but is refused (s = 5, destination B), so the hero keeps walking. Continue down the flat at 18.25 and the last half turn (43.2 from the line) to the foot of the stair at (−118.47, 8.38, −66.86) and then 13.7 across the bottom landing to the exit door's centre (−111.43, 8.37, −78.61), 8.8 wide (door index 1; its height is 11.2 above y 8.37). Total walk about 373 along the centre line, plus the four short back-and-forth steps.
  Running the laps in the wrong order loses a state: touching A at s = 1 (instead of stepping back to B) lowers s to 0 and the walk must be repeated.

Dormant +0x29F0 pool (section 15.1, synthetic):

- Separation: r_s = 1 at (0, 0, 0) and r_o = 3 at (2, 0, 0): self moves to (−1.5, 0, 0) and the other to (2.5, 0, 0).
- Drift: p = (0, 5, 0), p_prev = (0, 5, −0.1), draws i = 6 and j = 7: p becomes (0.0018, 5.005, 0.1011).

Trigger pass (Behavior, Trigger pass; values from the disc where a pack is named):

- Pack 1432, rectangle of index 0 (kind 7, flags 0x0100, static): the selected hero stands in it on ticks 100 to 104 and is outside from tick 105. Fires on ticks 100 to 104 with object 1, shot word 0, last argument -1, attack word = the hero's hit-window word on that tick. State after each tick's script step: 1, 2, 2, 2, 2, then 2 (tick 105), 3 (106), 0 (107).
- The same rectangle, the hero not touching it, a hero shot inside it on ticks 200 and 201, record idle before: tick 200 writes object 0, shot word 1, state 1, stamp 200; tick 201 writes nothing (state not 0, object 0, last argument -1); state 1 after tick 201, 3 after tick 202, 0 after tick 203.
- Pack 1174 floor tile, index 36 (kind 4 box, flags 0x0120, moving test): the hero walking through it fires nothing; a hero shot fires it (object 0, shot word 1); a world projectile fires it only while the act-1 trigger map holds 36.
- Petpet inside a cylinder with flags 0x0008: the handler runs and the fire step refuses it; no record change and no callback. The same petpet inside a book station (flags 0x0040, book value 12) sets the pending book word to 12.
- Region character (mode 1) inside a sphere with flags 0x0008 that a cell lists in list A: nothing; that character never walks list A.
- Movable block 3 inside a static kind-4 rectangle with flags 0x0100: fires with no actor, object 0, last argument 3 only while the selected hero's cell lists the rectangle; with the hero standing over a cell that does not list it, no fire.
- The selected hero over a kind-8 record (flags 0x0004): kind-1 handler, no fire, no push.
- A volume with gate mask 0x0002 while the world's gate word is 0x0004: skipped on the cell list A walk; the same volume is still fired by a hero shot (the shot pass has no gate-mask test).
- Hero holding a record (object 1, state 2), an NPC firing through CollideEntityWithTriggers on the same tick: the record keeps object 1 and the hero's stamp; the callback, if its thread is free, sees object 1.
- Tor in his hit window (event bit 0x01 on the current frame) with his body inside a sphere of flags 0x0008: attack word 1 on those ticks and 0 on the others; his blade inside a volume that his body does not touch fires nothing.
- Door callback lookup: pack 706 in T is current and the hero confirms door 0. The callback registered by 706 for door number 0 starts; its enter call into 984 is refused and nothing loads. If E were current at that moment, the door step would look in E's table at number 0 instead, not in 706's.

## Open questions
- Default if unresolved (rewrite choice): leave the type-40 interval uninterpreted and do not treat it as fixed-record data or script-group entries.

- Default if unresolved (rewrite choice): do not assign a gameplay purpose to the type-40 fixed-record dispatch. Preserve the guarded type-1 activation route and treat the observed type-2 to type-5 stubs as inactive unless further support identifies another handler contract.

Numbers of removed questions are not reused.

1. Resource 527. Resolved: it is never read, so there is no parser; its directions are travel directions and the fourth value of each colour line is the strength (section 3; SPEC-0046). [High for "never read"; Medium-High for the field roles]
2. Reach of type 30 light sets. Resolved: they reach characters only, through the character light walk into the four light slots of the character's light context; world geometry is unlit (section 8; SPEC-0046). [High for the reach; Medium-High for the type roles]
3. Falloff: only type 3 entries of a light set have a falloff (character walk, SPEC-0046); resource 527 is not read. Resolved for type 28: its +0x2C value is a camera focal length, not a light radius (section 1).
4. **Resolved: type-39 and character-region effect IDs.** The 116 distinct IDs stored in type-39 placed-effect records all resolve to definitions in resource 9; none is missing. The 76 distinct IDs in character-region effect lists also all resolve there; none is missing (SPEC-0034 sections 2.7, 2.9 and 6; High). Two values, 1416 and 1431, occur in both disc ID sets; the other 74 region-list IDs do not appear in the placed-record set. Both fields contain effect-definition IDs. Script `StartEffect` and `StopEffect` arguments are a separate domain: they select a placed-effect slot in the active world, whose record supplies the definition ID (SPEC-0034 section 6; SPEC-0030 rows 5 and 6). Thus a script argument must not be counted as a definition ID without resolving its slot to a record.
5. Resolved: type 39 +0x44 is the placed effect's zone number, 0 to 59 on disc (not a group); each pass an active record whose zone is not visible is stopped, otherwise it is updated (sections 4 and 9). [High]
6. Answered: see SPEC-0052 and SPEC-0027 (type 28 records are the camera tracks; section 1).
7. Type 30 entries of arrays B and C: their colour and strength are stored, but no consumer of the colour was found. Only the character walk reads these entries (SPEC-0046).
8. Type 29 part bytes from +0x44 to +0x5F (section 7).
9. Resolved: 650 objects have a model-set pointer, all of type 5; 642 have parts and 8 have a set but no parts. The object tables also hold 29 zero entries (section 7). [High]
10. Type 29 draw gate. Resolved: every write of the gate byte stores 1, so an object draws unless its state is 3 or 4; the six pack-1298 objects whose stored gate byte is 0 draw in any state until a script writes it. Placed objects of the first list have no broken state and are not breakable; the breakables are the second-list entries (SPEC-0048) (section 10). High.
11. **Resolved in part:** for kind-4 and kind-7 rectangles on the static path, only a flagged selector of 500 or 501 selects the same-world pad branch; other selector values use ordinary static-rectangle handling. Other nonzero values found in sphere and rectangle records have no consumer and no authored meaning is encoded (class D; ignore them) (section 6). [High for the rectangle branch]
12. **Answered in part:** the 0x60-byte interval from payload offsets 0x534 to 0x593 is outside the 21 fixed records and the counted script-group list. The loader walks and relocates those record blocks but not this interval. Its repeated sparse on-disc pattern includes two count-like values of 2, a value of 20 and two offset-like values that point into trailing payload data; their roles are not encoded anywhere readable (class D). Leave the interval uninterpreted and do not treat it as script-group entries (section 11). [High for boundaries and observed pattern; Low for the field roles]
13. Type 41 render scalar and post-pass mode. Resolved: the scalar is the maximum depth-buffer value of the view-screen mapping, not a draw distance. When accepted by the renderer's limit it changes the view-screen depth mapping and rebuilds its matrices; it does not move the near or far clip planes. A modern renderer that aims to preserve the original depth mapping must carry this value into its projection/depth conversion; treating it as a fade or color value is wrong. The post pass is in mode 3 in every area, so vector A is never shown (section 11). High for the mode and mapping path; Medium-High for the scalar's rendering effect.
14. **Answered in part: Type 1000 runtime reads.** The loader uses the count at +0x0C and records from +0x10; the known line-display path reads colour bytes +0x08 to +0x0A and ignores +0x0B. Header words +0x04/+0x08 and the fourth entry byte have no encoded authoring meaning (class D), and no other runtime consumer exists. High for the observed loader/display reads; Medium for the bounded no-consumer conclusion.
16. Whether the code constants used in section 10 (50.0 radius, −200.0 cull, 2⁻⁸ colour scale) are located in the boot program's data; they have only been taken from the code path.
17. Resolved for counted primitive arrays: nonempty lists follow spheres, cones, cylinders, kind-4 rectangles, kind-7 rectangles in pointer order in all 189 chunks, omitting empty lists. Auxiliary arrays interleave variably; no general total order is established (6.6).
18. Answered: see SPEC-0018 (sphere records, section 6.2, are pushed out of by the character contact pass when the id word has class 0 or class 1 and not class 2; they are not a plane test).
19. Cone records: count matches (83 in 7 packs). The first extent is the height H (positive in all 83). The cone routine of SPEC-0018 uses the record; the weapon-contact pass does not reach it, the cell list walks do (open question 76).
20. **Resolved in part.** The rectangle's +0x30 point is the runtime low-corner origin: the collision test projects points relative to it and compares against 0 through the stored in-plane extents, while the checked type-11 pointer-relocation routine does not transform or recenter the geometry. This resolves runtime use, not editor or authored intent. The first 16 bytes of the runtime primitive header are zero in every record and nothing uses them (class D; ignore them) (section 6). [High for runtime origin]
21. Resolved: the static kind-4/7 rectangle test treats the vector at +0x40 as the plane normal; +0x50 and +0x60 are the in-plane axes, paired with extents +0x28 and +0x24. This records runtime field use only; it does not assign an editor or authored axis name. (section 6.1 and section 13). [High]
22. Resolved: the fourth-list 0x50-byte records are the kind-8 ambient sound loops and adventure zones of SPEC-0031 (section 6.5). [High]
24. Resolved for the kind-3 contact routine: the signed full length is used directly. It shifts the tested cylinder centre from the stored base by (H/2) × axis, and the overlap test retains signed half-lengths; it does not convert negative H to |H| or clamp it. Resource 1394 has H = −14.47 (record 65 of its 124 cylinders; flags 0x0001, so it can only push, it never fires a script trigger; base (131.6, 4.27, −1424.1), axis up, radius 3.08), so its computed centre lies 7.235 units opposite its stored axis from the base. The pack's other placed points near it (level locators at heights of about −4 to 4 within 50 units) put its base at a floor-level height, so the record is probably within the hero's walking area, but whether the hero actually touches it depends on the player's position (needs a runtime check; proactive (open close-out)); Default if unresolved (rewrite choice): apply the signed arithmetic as stated. The distinct blade-recoil branch is tracked in open question 31. [High for routine and record arithmetic; reach open]
25. Resolved (High): placed objects are not in the object lists of SPEC-0046 (those lists hold characters). The draw gate of section 10 is their only draw, with no far flag and no 350-unit submit cull (see List membership).
26. Resolved (open close-out): no placed object uses a draw distance from the designer properties; the two monster-table fields are never read (SPEC-0046 open question 12). **[High]**
30. Kind 1 test. Resolved: kind 1 is a sphere (centre +0x30, radius +0x3C) tested against the three edges of the actor's weapon-attachment triangle, boundary included (Behavior, Trigger volumes; High for the radius field, test and input storage sources; Medium for the physical names of the stored direction vectors).
31. **Resolved in part — Kind-3 blade-recoil test.** The hook uses a fixed-radius 0.5 test body whose full length is the hero contact-radius getter × 0.25. It passes the world cylinder's own radius and signed full length; its test centre is base + (H/2) × axis. A zero derived motion offset selects static cylinder overlap; a non-zero offset selects the moving-cylinder helper. No route-frequency claim follows from this branch test. The static helper retains signed half-lengths. The moving helper also uses signed H in its projected half-extent: H × |axis alignment| × 0.5 is not made absolute or clamped, so negative H reduces or reverses that projection allowance. Later edge checks still determine the final hit. [High for inputs and projected gate; Medium for the derived motion test; final negative-H hit cases open]
32. Kinds 4 and 7 (rectangle). **Resolved in part:** runtime selects the static test when the float at +0x2C is zero or negative, and the moving test only when it is strictly positive. For the static test, Q is the translation of the hero's first weapon attachment frame; U and D are the stored vectors used by that frame's contact helper; and r = 0.15W, where W is equipped weapon reach (1.5 without a weapon). Thus P0 = Q + 0.25 × U, P1 = P0 + r × D, and P2 = P0 + r × U. The segments P0P1, P0P2 and P1P2 are tested without margins. The moving test is five faces of the plate box with a test point of radius 0.6 S (SPEC-0048, The block's moving-rectangle test). The stored layout also places +0x2C among the three extent words, so its design-time meaning is not established; negative kind-4 values on disc select the static branch. [High for runtime geometry, reach factor and storage sources; Medium for the physical vector labels; Low for designer intent.]
33. Resolved: the first gate is the script breakable hook keyed by the script index +0x58 (Behavior, Apply on a hit, step 1). It is not a latch: a registered callback takes every hit, and it is started only when it is not already running. [High]
34. Resolved: the byte at +0x5E is the element class of the breakable, compared directly with the attacker's Mote kind (Behavior, Apply on a hit, step 2); there is no side-code map. Its disc values are counted in SPEC-0048. [High]
35. Press variant. Resolved: it is Tor's blade recoil; the mode byte maps to a recoil material and the material to the recoil cue (Behavior, Blade recoil; High).
36. **Resolved: volume flag bits.** Section 13 covers the script-fire and side-effect gates for bits 1, 2, 3, 5, 6, 8 and 9; bit 2 has no firing handler. Bit 1 also marks door and pad records during loading. Character collision uses bits 0, 4 and 5 for cylinders and bits 0 and 5 for rectangles; bit 7 marks the dormant grab-door record. See Script trigger fire by flag bit, Doors, Book station, Grab-door records and Cylinder flags in character collision. No additional volume-flag behavior is established.
37. Resolved: the first word is the occupied count of the hero's 15-record tracked-opponent list; the second is the active/clear-path status in its first record. The gate blocks only when both are nonzero. Neither is a per-swing latch, and the recoil hooks do not change them. The dynamic pool can offer multiple candidates only if some caller populates it; its initialized-empty state and the absence of any known add caller leave shipped use unestablished. The separate world-frame branch also continues after offers. The callback has no one-shot latch, so multiple accepted offers can conditionally write recoil repeatedly while its cached attack-event and busy gates stay true; the exact accepted count per swing depends on the swing and the objects in reach (needs a runtime check); Default if unresolved (rewrite choice): follow the Blade recoil behaviour as written and apply each accepted offer. [High for fields and gate]
38. Answered: see Behavior, Script trigger fire by flag bit and Trigger pass. A firing volume supplies its +0x1C trigger index; grid-cell volumes look up the world's trigger table, while an inline placed-object volume uses that object's trigger list (with the documented shot-path exception).
39. Resolved: the 0.25 is the piece push that the break effect hands to each piece of the broken object (SPEC-0048); the actor does not move. How the pieces use the push value is tracked in SPEC-0048.
41. **Owner primitive lists at run time** (section 6.6): which runtime list holds each record kind (kind-4 rectangle, kind-7 rectangle, sphere, cone, cylinder and the 0x24-byte sub-list), and so how the six-list walk maps to the header counts in section 6. The order of the walk is as stated; the mapping does not change any result, since every record kind is tested by its own rule; Default if unresolved (rewrite choice): keep one list per kind and walk them in the stated order.
42. **Type 39 owner.** Partly resolved. The walk-time binding and the per-frame update are in SPEC-0034 (effects). SPEC-0028 owns the record layout. Owner of the run-time behaviour: SPEC-0034.
43. Answered: see SPEC-0018 (type 21 is the navigation mesh chunk).
45. Resolved: **Crossing solver caller.** There is none: no call, jump, table entry or built address in the boot program or in any level module refers to the solver, so it is unused code in the shipped build. [High]
46. Resolved: **Crossing solver and SPEC-0018 sweep.** No sweep rule uses it (see 45); the two cannot disagree in play. [High]
47. Resolved: **Crossing solver segment scaling.** Moot: the solver never runs (see 45). Do not implement it, so the factor of 0.5 does not matter. [High]
48. Resolved: **Crossing solver in-plane axes.** Moot for the same reason (see 45). [High]
49. **Confidence of types 40 and 41.** The loader dispatch is High. Type 40: load-time steps Medium; list-entry fields and kinds High (section 11); the fixed-record block Low. Type 41: load-time steps High, field roles Medium-High, the render scalar's meaning Medium-High (section 11).
53. Answered: see SPEC-0018 (no proximity carry list; the walk is the placed-object list).
54. Resolved: nothing is carried by that walk. The character or projectile is mapped into the object's rest frame, tested against the object's inline primitives, and mapped back (character push) or restored (projectile) (SPEC-0018).
55. Answered: see SPEC-0018 (riding is the ground attach carry; the radius walk only pushes).
57. Resolved: the two data words of the six-slot look-ahead are 0x2EB340 = 0.0 (the scale along the bush's up axis) and 0x2EB344 = 1.5 (the margin) (Behavior, bush shot test; SPEC-0060). [High]
58. Resolved: the hit handler is the bush cut and drop of the foliage patches (SPEC-0060, Bushes and Bush drops). [High]
59. **Resolved: no.** The six 0x2C0-stride auxiliary records are separate from Roberta's six shot slots, which the bush test reads from the per-hero array at +0x10D0 with stride 0xA0 (Behavior, Six-slot look-ahead; SPEC-0036, Hero shot slots). The auxiliary records are the hero's six effect records (SPEC-0034, effect record bank reset; same offsets and stride). [High for separation; Medium-High for their role]
56. Resolved: for blade triangle hooks against spheres, rectangles and world faces, Q is the first weapon attachment frame's translation, U and D are the stored vectors used by the contact helper, and the extension scalar is 0.15 times equipped weapon reach (W=1.5 with no equipped weapon). This is distinct from the body contact radius used by the separate cylinder hook. The busy gate is resolved under question 37. [High for source offsets and scale; Medium for physical names of the two vectors]
60. **Partly resolved: object accessor fields.** The compact accessor group reads a setup descriptor used by shared object/render initialization. The +0x04 field selects between initialization paths; +0x10, +0x20 and +0x30 are vector-like transform inputs; the block at +0x5C, +0x64 and +0x6C is copied together. Other fields are copied or used as selectors, but their individual meanings are open. A separate setter writes at +0xC9C through +0xCAC on a larger runtime object; no relationship to the compact descriptor is established. The concrete descriptor owner and exact meanings of most fields remain open. [High for accessor layout and observed uses; Medium for descriptor role; Low for owner and remaining field semantics]
61. **Partly resolved: type-40 fixed-record dispatch.** The owner is the type-40 world-chunk fixed-record table: setup visits its 21 records at 0x44-byte stride and codes 1 to 5 are the corresponding fixed-record indices. Type 1 has a guarded world/subrecord path that reaches the shared activation routine; types 2 to 5 have stub handlers in the available build. A separate variable-list pass at +0x598 includes code 6 and is not this dispatcher (SPEC-0006, Switch-keyed entries). There is one disc build, so the stubs are no-ops in the only build a rewrite targets (High). The type-1 subrecord's gameplay purpose is not encoded anywhere readable (class D); Default if unresolved (rewrite choice): as in the defaults at the top of this section. [High for owner/layout and handler bodies]
62. Resolved (proactive (open close-out), Medium): the routine is one step of a character's contact test against the bound collision faces of placed objects (the per-face contact the character contact pass runs for each non-skipped face of an overlapping object, SPEC-0018). It lowers the character's reference point by the absolute value of one face field along world up and asks the shared overlap test whether that lowered point touches the face; the true or false answer picks which of two follow-up contact branches runs. It is not a separate object routine and has no effect beyond the face contact rules of SPEC-0018, so the rewrite implements it through those rules.
63. Answered: see SPEC-0018 (the character versus moving placed objects rule; not a riding rule).
64. Resolved: the radius at +0xBC is the setup radius max(50.0, |size.x|, |size.y|, |size.z|) of section 7.1.
65. Resolved: +0x68 of the pack 1299 puzzle record is the midpoint height M = 24.1535 of the two pads; characters above or below it are moved with the accumulator (Behavior, Pack 1299 looping-door puzzle).
66. Resolved: state 6 cannot be reached from play; at state 5 a crossing to pad B is not teleported (Behavior).
67. Answered: see Behavior, Cut-reed fall, and SPEC-0060 (the fall of a cut reed piece, not a mover).
68. Resolved: the first state ends when the record is more than 0.5 from its anchor, so the record leaves its anchor (Behavior, Cut-reed fall).
69. Answered: see SPEC-0034 (the keyed walk is the effect trail-strip renderer).
70. The six-state, three-state and keyed routines. Resolved: they are the pack 1299 looping-door puzzle, the cut-reed fall (owner SPEC-0060, High) and the trail-strip renderer (SPEC-0034); moving platforms and lifts are the animated placed objects of section 7.1.
71. **Pool table.** Resolved: the 0x400-byte, 0x32-count table is the world's script-thread slot table (50 threads, SPEC-0030; Medium-High). Default if unresolved (rewrite choice): keep the seven pools of section 15 as plain lists and take the thread limit from SPEC-0030. The seven pools and the scene list are the script callback tables (section 15).
72. **Scene sub-lists.** Resolved: each is an array of pool-A-style 0x48-byte link records sized by the scene's own count (section 15; Medium-High), and the release rule and state step of Pool maintenance apply to each record. Default if unresolved (rewrite choice): apply the same release rule as the pools (a link is freed when its object is gone or its state word is nonzero); the rule only frees finished script callback threads and no player-visible effect is known.
73. **Per-record update.** Resolved: the update is the state step described under Pool maintenance, step 3 (Medium). Its effect on the object that owns the record is not known; Default if unresolved (rewrite choice): apply the step as written and let the release rule decide which links end.
74. Resolved: the pools are the script manager's callback tables: A triggers, B breakables, C teleport planes, D entity proximity, E 743 game flags, F 64 level flags, G 10 item ask-about callbacks (section 15; owner SPEC-0030). [Medium-High]
75. **Stored scene id.** Default if unresolved (rewrite choice): treat the value at +0x6A as used only for the write back to the scene id field (no other reader is known).
76. **Volume kind 2 and the cone routine.** Resolved: the weapon-contact pass never fires kind 2; its test reports no hit. The cell list walks, the shot pass and the projectile step route kind 2 to handlers that fire with bit 3; all 83 cones have flags 0x0000, so none fires (High).
77. Resolved for runtime behavior (section 6.1): negative rectangle extents remain signed in box contact and narrow the strict containment interval; they are not converted to positive sizes. The vertical field also retains its 0.1 top offset and optional top growth. The authoring intent of negative values is not encoded (class D). Only kind-4 records have negative extents (3, 59 and 70 records); no kind-7 record does. [High for runtime use; open for design intent]
78. **Kind 1 shape (section 13).** Resolved for the weapon-contact pass and the blade recoil: kind 1 is a sphere (centre +0x30, radius +0x3C) tested against the actor's contact-triangle edges (Behavior, Trigger volumes; High). The camera's eye pass pushes the eye out of the same record as a sphere grown by 1.25 (SPEC-0018, Sphere contact); that 1.25 push belongs to the camera only, not to characters.
80. Answered: see SPEC-0060 and section 16 (type 16 records are foliage patches, not trigger volumes).
82. **Class predicate.** Resolved: on the cell list B walk the owner's class predicate is the hero's landing break (it breaks the object and skips it) and passes every object for other classes; the fire step refuses petpets (Behavior, Trigger pass). High.
83. **Trigger pass for non-hero actors.** Resolved: region characters run the trigger step in mode 1 (cell list B only), so they never reach a type 11 volume or a book station; the petpet walks cell list A, so it can set the pending book word, but its script fires are refused (Behavior, Trigger pass). High.
84. **Cylinder flags 0x0010 (224 records), 0x0011 (256), 0x0020 (33), 0x0021 (192) and 0x0031 (1).** These cylinders are not save points (save points are region checkpoint records, section 13). Bit 1 is clear in all of them, so they do not fall under the bit-1 stop. Resolved: in character collision bit 4 makes a cylinder solid and bit 5 removes it (Cylinder flags in character collision, section 13). High.
86. **Rectangle lists at load.** Resolved in part: the type-11 lists are already part of the loaded type-11 payload; the level loader stores the payload pointer and relocates its list pointers, without allocating replacement primitive arrays. The kind-7 rectangle list remains at +0x3C with count +0x14, and kind-4 remains at +0x40 with count +0x18. A later world-activation builder runs only when the world grid is non-null. It reads qualifying kind-7 rectangles to make separate pad, door and grab-door products, and also reads cylinder records and movable-block source entries. These products are not the collision-volume lists; the builder does not read the kind-4 rectangle list. Default if unresolved (rewrite choice): the rewrite reads the loaded kind-4 and kind-7 lists directly as the collision-volume arrays. [High for load-time pointer relocation, list/count mapping and these builder boundaries]
87. **Kind 1 entry records at +0x4C.** Resolved (High for the field, Medium for reach): the kind 1 test reads its radius from +0x3C, never from +0x34, which is the y of the centre (open questions 30 and 78; the older wording that named +0x34 was wrong). In the sphere record layout the three extents at +0x24, +0x28 and +0x2C repeat that radius, so the first entry record of pack 1432 has radius 0.625 at centre (0.052, 273.838, -265.063). Entry records are the world's 0x50-byte locator records (section 6, entry list); with flags 0x0020 (4,924 records) or 0x0021 (40 records) none of them fires a script trigger. A kind 1 volume needs bit 3 to fire, which these flags lack.
89. **Hero per-tick pass input.** Resolved: the selected hero's trigger step tests cell list A (type 11 volumes) and cell list B (trigger objects) of the grid cell under its feet point, plus the inline volumes of animated placed objects in reach; a volume no cell lists can fire only through a shot or a movable block (Behavior, Trigger pass). High.
92. Resolved: **Door refusal in pack 706.** The door step finds a door's callback in the current world's teleport table by the touched door's number, so the callback of a T door runs only while T is current. 706's door 0 callback therefore always makes its enter call into 984 while T is current, and the call is refused: the door never works, and 984 is entered from exterior 956 (SPEC-0006, Kind refusals). (High)
93. **Object value on the hero per-tick pass.** Resolved: every hero body contact, of every kind, fires through the hero's trigger-hit method with object 1; the pass earlier called the hero per-tick volume pass is the hero shot pass, which fires with object 0 and shot word +0x20 = 1 (section 13). High.
94. **World character list at +0x58E8.** Resolved: on a pad crossing, the puzzle walks the current world's active character list from entry 0 until the first null entry, with no hero, boss or class filter. That list is the world's active characters, rebuilt compactly on the streaming pass (at most 15 entries, null after the last), so the walk visits every entry. It moves entries below M when dy > 0, or above M when dy < 0; the comparison is strict. The crossing-time adjustment uses dy, while the per-tick accumulator offsets placed objects only. For pack 1299 the list is empty: the pack holds no character records, its level module creates no character and spawns none, and both the streaming rebuild and the spawn helpers work from the world's own character slots, of which it has none, so a crossing moves no character. The hero is not in the list, and the companion petpet is created through the pet manager rather than the world's character slots (Medium for the petpet). Test vector: crossing from pad B to pad A in pack 1299 with dy = -9.0: no character is moved, only the placed objects take the accumulator offset. [High for the walk and movement rule and for the empty list; Medium for the petpet]
95. Resolved: the pack 1299 puzzle step runs after the selected hero's motion-record tick. It adds the selected hero's current vertical position minus the position saved at the start of that motion-record tick to the accumulator. Ordering relative to other characters' motion-record ticks only matters for the non-selected hero standing on the pads; Default if unresolved (rewrite choice): step the puzzle right after the selected hero's tick (needs a runtime check). [High for the selected hero order and displacement source]
95a. Resolved (High): a pad crossing puts the hero at the mirrored point of the other pad's plane (the same distance along the width counted from the far end, the through distance reversed), which on this disc is the same x and z, with the height re-solved on the other floor and the camera moved by the height change; the difference of the pad centres is not the rule (Behavior, Pack 1299 looping-door puzzle). The landing does not fire at once because the side flips and the height band changes.
95b. Resolved: the mask word of node 401 is the draw mask of that node's own small tree, not tested against the world's display word (Behavior; High). Neither the state nor the mask is reset by the refused touch at s = 5; the world reset clears the state at release, and the accumulator is never cleared (Medium-High).
96. **Reach of scripted moving objects.** Resolved (Medium-High): 108 moving placed objects with height patches, in 32 packs, are named by a fixed PlayAnim or LoopAnim index in their pack's script. 31 of the 32 packs (107 objects) can be entered; only 1077 cannot (section 7.1). Every one of the 108 has at least one such call in script code that is reached from the pack's init, update, a control thread, a hook or a registered callback: 95 are reached directly and 13 only through routines that are reached by a call or a registered address although they look unreferenced at first sight; none is dormant. For the 107 in enterable packs, the easiest call of each object is: 16 with no flag test (init, thread or helper code), 16 started by a button press, 29 behind a story or level flag (packs 757, 777, 799, 864, 1078, 1098, 1122, 1244, 1247, 1344, 1371, 1397 and 1411) and 46 behind a trigger argument or other state (volume entry, hero or character state). Which story states open each flag gate is in the level specs; the gate conditions were not evaluated against play state.





