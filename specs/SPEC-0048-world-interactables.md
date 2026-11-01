# SPEC-0048: World interactables: breakables, lock chests, switch groups and movable-block control

- Answers: proactive (open close-out); proactive (world interactables; no request identified); R-0040 (in part; open items under Open questions), R-0048 (in part: set-up poses by kind; the writers of object states 5 and 6 open in SPEC-0030), R-0063 (movable blocks: direction words, push-start tests, box axes and home, character collision and trigger-volume tests), R-0076 (breakable volume fields: cylinder base and rectangle corner roles), R-0077 (the hero during a block push; block boxes and the static lists), R-0078 (the block's five-face moving-rectangle test; inactive blocks), R-0079 (standing on a block: the strict top bound, the resting height and the floor flag; the pass for region characters and its response), R-0108 (clause 1 resolved for the normal push path; ladder and hang details are in SPEC-0018/0033/0040), R-0131 (what limits a push: the footprint patch written by every placement, its outline test, the hero-point test, push limits of every block on disc); proactive (farm crate boundaries: the pack 909 crate's outline polygon, centre limits, lattice, push and pull rules, nearby walls and purpose); proactive (climb pass); proactive (verify pass: reachable block positions, shortest push and pull routes per puzzle, hero grab spots, pack 1381 stations); proactive (verify pass: pack 1397 plate order, floor swaps and lifts); R-0157 (what stops a character at a lock chest or a sealed chest)
- Status: Draft
- Disc build: SCUS-97367 v1.00 (same fingerprint as SPEC-0001)
- Confidence: per section. Breakable entry layout and break and state rules: High for order and offsets, Medium-High for state names. The state word at +0x40 holds the value 1 in every entry on disc (2,073 of 2,073; states 0 to 3 occur at run time), under the table-relative offset reading (Where it lives). Piece-record links and setup inputs: High; full field meanings and final drawn transform: Low. Positive +0x5A effect-controller call: High for data flow, Low for its unit and perceptual meaning. Persistent broken bit and the start-mode gate: High. Attack contact routes (script hook, element gate, 0.25 push, hit cue by material): High; the +0x5A-gated route being unreachable: Medium. Switch group step: High for the code rules and data counts, Medium for meaning and authoring intent. Draik slot ownership and release callers: High. Lock table, lock add and lookup: High. Button masks: High for all 16 codes and for the button names (code 0 is Square; SPEC-0033 Controller map). Second-argument word (held or newly pressed, pad slot 0): High. Press rule: High for the gates, the prompt, the key rule and the open effects; Medium for the press height units and the busy counter's meaning. Movable-block list and setters: High for offsets and writes, Medium for meaning; mover per-tick motion, push start and placement probe: Medium-High; footprint patch (written by every placement) and the outline limit of a push: High; push limits per block on disc: High for pack 909 and the pack 1324 bells, Medium-High for the rest; push tick count n: High. Plate manager (three plates and one sinking object): Medium; High for its per-tick constants (0.1, -50, 1.5, 0.5); High that it is never populated in play. Element-sealed chest and breakable chain: High for the gates, hooks and waits; Medium for the start of effect 791 and the exact drop point. Breakable start mode on disc data: High. Doors: pointer to SPEC-0006 (High). Script groups (kinds and set-up): High. Proximity fuse: Medium for the rules, High for the thresholds and the hero-state gate. Reaction object: High. Movable-block disc source (the type-11 n1 list and the flag bit 1 rule): High.

## Summary

World interactables are the world-side objects that a level script or a hit can change: breakable objects that can be broken once, switch groups that break their members in order, lock chests that open with a key item, and movable blocks that a script can activate and move toward a locator and that the hero pushes one step at a time. A small per-world plate manager for hero-pressed plates and a sinking object also exists, but nothing fills it in play; plates in play are scripted trigger volumes (Plate manager). The chests, the chains, a proximity fuse that breaks its breakable after a countdown, and a reaction object that replays a clip when its trigger fires are all script groups: type-40 entries whose kind picks a built-in behaviour (Script groups). This spec describes their stored records, their state rules, and the order of the effects that follow a break or an open. It builds on SPEC-0028 (the type-29 placed-object chunk, whose second list holds breakables; the type-40 switch list), SPEC-0021 (the placement drop action that a breakable can trigger), SPEC-0030 (the script calls that act on these objects) and SPEC-0018 (the movable-block walk). Those facts are not repeated here.

A clean implementation can use this spec to build the breakable, switch and chest logic of a world. The lock table and the chest press are per world and per level; a breakable's start state comes from its own start-mode byte at each world load.

## Where it lives on the disc

- Breakables: the second list of the type-29 chunk in a world pack (SPEC-0028, section 7). The chunk gives the count at +0x08 and the table offset at +0x0C (both payload-relative). The table holds one 4-byte offset per breakable. Each offset is measured from the start of the offset table (not from the payload) and leads to that breakable's entry. The world record keeps this chunk at +0x5834.
- Switch groups: the type-40 chunk of a level pack (SPEC-0028, section 11). Its list starts at payload offset 0x59C; the count is at 0x594. The world record holds the count at +0x594 and the list pointer at +0x598.
- Lock table: runtime world record, not on disc. Two 12-byte entries at world +0x5914.
- Movable-block records: runtime world list at world +0x5810 (count at world +0x580C), built at level setup. Disc source (High): the n1 list of the world pack's type-11 chunk (0x24-byte entries; count at header +0x1C, table at header +0x44; SPEC-0028 section 6, SPEC-0006 Tool data). An entry becomes a block only when its box pointer (entry +0x20, a kind-4 rectangle record) is set and that box's flags word (+0x12) has bit 1 (value 0x0002) set. On disc 91 entries in 17 packs build 90 blocks; pack 1344's one entry has no box.
- Locator list: runtime world list at world +0x56B0.
- Plate manager: runtime world block at world +0x2D40 (count word), with eight 0x160-byte slots from world +0x2D50 to +0x3850. Not on disc. Nothing fills it in play: its add routine has no caller and no host call or data pointer reaches it (SPEC-0028, Built-in object pools that are never filled; High).
- Break source words: three world words at +0x5780, +0x5854 and +0x5850.
- Persistent broken set: one bit per breakable script index (+0x58), in a 17-byte row per world index; the row is chosen by the world index word at world +0x5900. Script indexes of 128 and above are not recorded (High). The set is saved with the game (SPEC-0025, table T1). The movable-block sound index is one word at world +0x2CCC.
- Script breakable hooks: the world's hook table, filled by the script call RegisterScriptBreakable (SPEC-0030 row 267), 0x48-byte records indexed by the breakable's script index (+0x58). The record that owns the table holds the record count at +0x94 and the table pointer at +0x98; an index below 0 or at or above the count has no hook (High). Runtime only, not on disc.

No data from these records is copied here.

## Layout

### Breakable entry (second list of the type-29 chunk)

Offsets are relative to the entry. The second list is a table of 4-byte offsets, one per breakable, and each entry is reached through its offset (not by a fixed stride). Each offset is measured from the start of the offset table.

| Offset | Size | Type | Field | Confidence |
| --- | --- | --- | --- | --- |
| +0x00 to +0x2F | 3 x 16 bytes | float blocks | The three axis columns of the entry's draw matrix (column-major 4 x 4 with the translation at +0x30, as for placed objects). On disc the axes are a turn about the vertical axis (for example the columns (-0.755, 0, 0.656), (0, 1, 0), (-0.656, 0, -0.755) in the first entry of pack 732) with 0 in the fourth lane. | High |
| +0x30 | 16 bytes | float vector | Translation column of the draw matrix (fourth lane 1.0 on disc in all 2,073 entries) and the break position. Passed to the drop and effect calls. | High |
| +0x40 | s32 | state word | Breakable state (section Behavior). SPEC-0028 section 10 reads the same word as the second-list kind and draws kinds 1 and 3. | High (offset and use); Medium-High (state meanings); value 0 to 3 in every entry on disc |
| +0x44 | pointer | inline mesh | The mesh the entry draws (Breakable draw below). The disc value is an offset from the entry's first byte to an inline mesh chunk (type 12 or 13) stored inside the entry's own data; the load replaces it with the address of that chunk's payload and prepares the mesh as it does for a placed object's inline mesh. 0 means none at load. Nonzero in 1,980 of 2,073 entries on disc (all offsets multiples of 16, all pointing at a type-12 chunk), 0 in the other 93. | High |
| +0x48 | s32 | instanced mesh index | Index of a record of the world's first instanced-mesh chunk (type 20, SPEC-0004). -1 in the 1,980 entries that carry an inline mesh; 0 or more in the 93 that do not (all in packs that hold one type-20 chunk; every value is below that chunk's record count: pack 816 has 2 records, and its entries use index 0). | High |
| +0x4C | s32 | piece count | Number of piece records. | High |
| +0x50 | pointer | piece array | Piece records of 0x70 bytes each. The disc value is an offset from the entry's first byte; the load adds the entry's address (0 means none). | High (base); Medium (record use) |
| +0x54 | s16 | spawn key | Character kind key of a world-level spawn by key at the break (SPEC-0029 Behavior 31); -1 for none. It is not a character activation. 0 or more in 20 of 2,073 entries on disc; for example key 1960 in pack 732, which matches the two summon-reserved Sludgy records of that pack. | High |
| +0x56 | s16 | drop placement id | Placement ordinal for the drop; -1 for none. A plain field of the disc entry: 0 or more in 1,239 of 2,073 entries on disc. | High |
| +0x58 | s16 | script index | Key of the script breakable hook (RegisterScriptBreakable; Break routes, step 1) and of the persistent broken bit (Break effect, step 6). It is not a sound index. Unique within a pack; below 128 in 2,069 of 2,073 entries on disc. | High |
| +0x5A | s16 | break float | When above 0, the break effect runs its timed effect with this value (Break effect, step 5). The attack contact routines do not test it; the only break route gated on it is never reached (Break routes). | Medium (use); Low (meaning) |
| +0x5C | s16 | contact material | Selects the hit cue of the attack contact routines: 2 -> cue 143, 3 -> cue 145, 4 -> cue 146, any other value -> cue 144 (SPEC-0031, Breakable contact cues). On disc (2,073 entries in 79 world packs): 0 in 1,557, 1 in 294, 3 in 19, 4 in 203, 2 in none. | High |
| +0x5E | u8 | element class (alignment) | 0 = any attack may act on the entry; 1 to 6 = the Mote kind an attack must have to act on it (see Break routes, step 2, and Element gate on hits); 7 is not used on disc. Returned by the alignment query. On disc (2,073 entries in 79 world packs; 121 packs have a type-29 chunk): 0 in 2,033 entries, 1 in 9, 2 in 6, 3 in 6, 4 in 6, 5 in 7, 6 in 6, 7 in none. | High (offset and counts); Medium (meaning of 1 to 6 as attack kinds) |
| +0x5F | u8 | start mode | 0 means the default mode, which is 0. Mode 1 and mode 2 have their own effects; any other value gives state 1 (section Behavior). Applied at world load only to an entry whose broken bit is set. On disc: 0 in 1,974 entries, 10 in 13, 20 in 3, 30 in 83; 1 and 2 in none | High |
| +0x60 | pointer | sub-record | Optional record: the breakable's type-11 volume (present in 2,030 of 2,073 entries on disc). Its type byte is at +0x10 (u8) and its volume flags word is the s16 at +0x12, which a break sets to 0. The disc value is an offset from the entry's first byte; the load adds the entry's address (0 means none) and then sets the flags word to 9, or to 33 when the type byte is 4. The shape fields (base or corner point, extents, axes) are in Breakable volume shapes. | High |

Word 0x16 of the entry, counted in 32-bit words (0x16 x 4 = 0x58), is the script index; it is not a break cue (High). The break effect itself plays no cue; the hit cue comes from the contact routines and +0x5C.

### Breakable volume shapes

Confidence: High for the cylinder and rectangle layouts and for the point roles (confirmed on the disc data below); Medium-High for the box faces of the moving rectangle test.

The volume record of a breakable (+0x60) is a type-11 primitive record with the layout of SPEC-0028 section 6. The fields that the melee walk uses:

- **Kind 3, cylinder** (1,915 of the 2,030 volumes on disc, including all 37 breakables of pack 732): the point at +0x30 is the centre of the base disc, not the middle of the solid, and the length at +0x24 runs one way. The solid is every point base + s x u + a lateral offset of at most r, with s from 0 to H, flat ends, where u is the unit axis at +0x40, H the length at +0x24 and r the radius (+0x3C; +0x28 and +0x2C hold the same value). Its centre, which the pair tests use, is base + 0.5 H u. Disc check: 1,894 of the 1,915 axes are vertical (0, 1, 0); in 1,837 the base height is within 0.5 of the breakable's own height (its draw translation); for the 1,822 with an inline mesh the median distance from the base height to the lowest point of the mesh is 0.02, while a middle reading (base - 0.5 H) would sit a median 1.53 below it (the base reading is the closer one in 1,580 of the 1,822).
- **Kinds 4 and 7, rectangle** (115 volumes on disc, all kind 4; packs 989, 1074, 1080, 1184, 758, 1225 and 1083): the point at +0x30 is a corner, not the centre. The extents are full lengths from the corner, never negative on the disc: the extent at +0x24 along the axis at +0x60, the extent at +0x28 along the axis at +0x50, the extent at +0x2C along the axis at +0x40 (the plane normal). The static test (extent at +0x2C of 0 or below) takes a segment, finds where it crosses the plane through the corner with normal +0x40, and accepts when the crossing has coordinate 0 to the extent at +0x28 along +0x50 and 0 to the extent at +0x24 along +0x60 (both ends included; a negative extent accepts nothing). When the extent at +0x2C is above 0 (all 115 on disc) the record is treated as a box and the same rectangle test is repeated on the faces that the three extents span (Medium-High). Disc check: in 90 of the 115 the centre corner + half of each extent along its axis lies within 0.5 of the centre of the entry's inline mesh (largest gap 4.8), and the point at +0x30 itself is never within 4.8 of it.

Worked tests (pack data; the swing numbers are those of SPEC-0036 Hit detection, step 6):
1. Pack 732, breakable 2. Translation (3816.401, -1.380, -8.106); cylinder base (3816.360, -1.380, -7.991), axis (0, 1, 0), H = 2.403, r = 1.335; the solid runs from height -1.380 to 1.023 and its centre is (3816.360, -0.178, -7.991). The entry's mesh spans heights -1.494 to 1.826. The pair test with a weapon whose axis equals the previous-tick axis (D1 = D2 = (0, 1, 0)), radius 0.5, length L = 3.5 (weapon reach W = 1.5, no weapon) and attachment point (3818.000, 1.5, -7.991): the weapon centre is (3818.000, 2.0, -7.991); the axial offset 2.179 is within (3.5 + 2.403) / 2 = 2.9514 and the lateral distance 1.640 is within 0.5 + 1.335 = 1.835, so it hits. Moving the attachment point to x = 3818.300 gives lateral 1.940 (miss); raising it to height 2.5 gives axial 3.179 (miss). A reading of +0x30 as the middle would put the centre at height -1.380 and turn the first case into a miss (axial 3.380).
2. Pack 758, breakable 17 (second-list index 17). Translation (2008.517, 101.374, 82.229). Corner C = (2003.644, 101.038, 89.533); normal n = (0.6466, 0, -0.7628); axis a (at +0x50) = (-0.7628, 0, -0.6466); axis b (at +0x60) = (0, 1, 0); extents 0.6726 along b, 2.0115 along a, 17.4453 along n. C + 0.5 x (0.6726 b + 2.0115 a + 17.4453 n) = (2008.517, 101.374, 82.229), the entry's own translation. The face on the plane through C: a segment from P = (2002.558, 101.338, 89.268) to Q = (2003.204, 101.338, 88.506) (that is, X - 0.5 n to X + 0.5 n for X = C + 1.0 a + 0.3 b) crosses the plane at t = 0.5 with coordinates (1.0, 0.3), inside 0 to 2.0115 and 0 to 0.6726, so it hits. The same segment shifted to coordinates (2.3, 0.3) or (-0.2, 0.3) misses.

### Piece record (0x70 bytes)

Each record is 0x70 bytes, and a break initializes one effect object from each record. The loader resolves the record's +0x60 link to a selected 0x70-byte part record in the breakable's model-set data, using the selector at +0x68, and stores the part-array base at +0x64. The piece-record +0x54 link is relocated and read by effect-object setup. Runtime setup also initializes +0x40 to −200 and +0x48 to point back to the piece record. The effect object receives the breakable's position transform and the break-source words; the record's complete field meanings and its final drawn transform are not established. [High for the listed links and setup; Low for full record semantics].

### Lock table (world +0x5914)

Two entries, stride 12 bytes, each with three signed 32-bit words (Medium on the signedness; the empty value is -1):

| Offset in entry | Field |
| --- | --- |
| +0 | id (the group index that the lock belongs to; -1 when empty) |
| +4 | key item id |
| +8 | message id |

### Switch group entry (type-40 list entry)

Each list entry is 0x44 bytes. The full field list and the kinds table are in SPEC-0028 section 11. The fields used here are (entry-relative):

| Offset | Size | Field | Confidence |
| --- | --- | --- | --- |
| +0x2C | u32 | member count | High |
| +0x30 | u32 | payload offset to the member index array; the indices are not inline. Each index names an entry of the second list of the same world (breakable). | High (count and offset); Medium (member meaning) |
| +0x08 | u32 | key: a row of the collected table (Script groups; SPEC-0028, section 11) | High; not read by the chain step |

The group object that drives a switch group carries an entry index at its +0x30 and a world reference at its +0x04.

### Movable block record (0xB0 bytes, world list at +0x5810)

| Offset | Size | Field | Confidence |
| --- | --- | --- | --- |
| +0x00 | vec4 | home position: the block's centre at its last placement (equal to the position then; w = 1) | High |
| +0x10 | vec4 | axis A (push directions 0 and 2): the box record's first axis vector (the vector at record +0x40), w = 0 | High |
| +0x20 | vec4 | vertical axis: the box record's third axis vector (record +0x60), w = 0 | High |
| +0x30 | vec4 | axis B (push directions 1 and 3): the box record's second axis vector (record +0x50), w = 0 | High |
| +0x40 | vec4 | current position: the box corner plus half the size along A and along B (the centre of the footprint at the ground height found at placement; w = 1) | High |
| +0x50 | vec4 | position minus home (w = 0); zero after a placement | High |
| +0x60 | pointer | the volume the block last fired (0 = none; Trigger volumes below) | High |
| +0x64 | pointer | display model: the entry's inline mesh, or the registry model named by the entry's +0x18 word when the entry has no inline mesh | High |
| +0x68 | pointer | the block's entry (91 entries on disc): +0x10 size S, +0x14 height H, +0x18 word, +0x1C inline mesh, +0x20 the box record | High |
| +0x6C | s32 | a value handed in at placement and kept for the next placement (role Low) | Low |
| +0x70 | pointer | the world record the block belongs to (a trigger fire needs it to be non-null) | High |
| +0x74 | s32 | active word: 1 after a placement; written by the activate call; the per-tick step runs only while it is non-zero | High |
| +0x78 to +0x84 | 4 x s32 | direction-allowed words, one per push direction 0 to 3: set to 1 by every placement and written by nothing else, so all four directions are always allowed | High |
| +0x88 | s32 | moving flag | High |
| +0x8C | f32 | step length L of one push = 0.5 S | High |
| +0x90 | u8 | tick counter i of the current push | High |
| +0x91 | u8 | tick count n of the current push (signed byte) | High |
| +0x94 | s32 | push direction D: 0 = +A, 1 = +B, 2 = −A, 3 = −B; −1 after a placement | High |
| +0x98 | s32 | the block's number (its index in the list; the fire step passes it as the trigger's last argument) | High |
| +0x9C | pointer | footprint patch: the ground patch entry (model-set entry) that the placement's ground probe chose at the box corner, written by every placement (Placement, step 1) through the probe's chosen-candidate slot. Its outline records (the entry's own frame records, count at +0x50 and list at +0x54 of the entry; SPEC-0018) give the footprint test of a push. It is 0 only when that probe found no grid cell or a cell with no entries. On disc all 90 blocks get a non-zero patch at load | High |
| +0xA0, +0xA4 | s32, pointer | attached-effect list: a count of 5 and the pointer to 5 entries of 32 bytes (an effect handle in the first word, an offset vector at +0x10); set up by the attach-effect call | High |

Index checks: every call tests the index against the list count.

### Locator record (0x50 bytes, locator list at world +0x56B0)

The list header holds the count at +0x24. Records start at +0x4C of the header, each 0x50 bytes. The locator position is the vector at +0x30 of its record. Medium.

### Plate manager (world +0x2D40)

Runtime only, and never filled in play (nothing adds a slot; see Plate manager, below). The block starts with a slot count word at +0x00 (the manager's reset sets it to 8); the slots are 0x160 bytes each and start at +0x10 of the block (world +0x2D50). The slot index decides the slot's role: slot 0 is the sinking object, slots 1 to 3 are the three plates, and slots 4 to 7 have no per-tick behavior.

| Offset in slot | Size | Field | Confidence |
| --- | --- | --- | --- |
| +0x04 | s32 | unused-slot word: a slot whose word is non-zero is skipped by the per-tick pass and by the completion count | Medium |
| +0xA0 | vec4 | position (y is the height that the plate press and the sinking change) | High |
| +0x140 | s32 | triggered flag: "pressed" for a plate, "sinking" for slot 0 | High |
| +0x150 | f32 | radius r (also the radius of the slot's bounding sphere, centred on the position) | Medium-High |
| +0x154 | s32 | solid word: when non-zero, the slot takes part in the character contact pass (its collision box block at +0xC0) | Medium-Low |

## Behavior

### Breakable states

Every breakable entry has one of four states, held in its state word:

| State | Name | Intact (query) | Invulnerable (query) | Drawn | Set by |
| --- | --- | --- | --- | --- | --- |
| 0 | disabled | no | yes | no | SetBreakableActive(false); start mode 1 |
| 1 | intact, breakable | yes | no | yes | SetBreakableActive(true); MakeBreakable(true); start mode 0 and 2; the group arm step |
| 2 | broken | no | yes | no | Break, an attack contact and the group step, from state 1 only |
| 3 | intact, not breakable | yes | yes | yes | MakeBreakable(false) from state 1 |

Transitions:
- 1 to 2: Break, an attack contact (Break routes), or the group step.
- 1 to 3 and 3 to 1: MakeBreakable with false and true. MakeBreakable changes only state 1 and state 3; it does nothing in states 0 and 2.
- 0 to 1 and 1 to 0: SetBreakableActive with true and false. SetBreakableActive(true) from state 2 also gives state 1 (see Edge cases).
- The group arm step sets state 1 directly (see Switch groups).

Queries: intact is true in states 1 and 3. Invulnerable is true in every state other than 1. The alignment query returns the byte at +0x5E, or 0 for an invalid index. Confidence: High for the code; Medium-High for the names.

### Breakable draw

Confidence: High.

Once per world draw, right after the placed objects of SPEC-0028 section 10 are drawn, every second-list entry in state 1 or 3 is drawn; entries in state 0 or 2 are skipped, so a broken entry (state 2) is hidden by its state alone and nothing else needs to change. There is no distance test, zone-grid query or draw-gate byte on this path.

- Mesh. The entry draws the one mesh named by its own +0x44 pointer. At world set-up (after all chunks are loaded), when the pack has a type-20 chunk and the entry's +0x48 index is 0 or more and below that chunk's record count, the entry's +0x44 is replaced by the near-mesh pointer of that record of the first type-20 chunk (the same mesh the instanced scenery of SPEC-0004 uses). With no type-20 chunk, or an index of -1 or out of range, +0x44 keeps its inline-mesh value. An entry whose +0x44 is still 0 draws nothing.
- Transform. The entry's first 64 bytes, used as stored, are the draw matrix: three axis columns, then the translation at +0x30. The mesh is not scaled by the entry. The "scale 1.0" that SPEC-0028 mentions is the intensity argument of the draw call (1.0, Medium); the entry carries no colour of its own.
- The piece records (+0x50) are not drawn here; they are the effect placements that a break spawns (Break effect, step 4).

Test vector: pack 732, entry 0 (state 1, +0x44 = 192 bytes from the entry start, +0x48 = -1, 10 pieces): drawn with its inline type-12 mesh at translation (3834.924, 6.345, 343.157), turned about the vertical axis by the matrix above. Pack 816, entries 0 to 7 (state 1, +0x44 = 0, +0x48 = 0, 8 pieces, a pack with a 2-record type-20 chunk): each draws record 0's near mesh at its own translation. After a break (state 2) neither is drawn.

### Break routes (script call and attacks)

Confidence: High for the steps, the push and the cue ids; Medium for which attack family uses each handler and for the unreachable +0x5A route.

A script Break call with an index runs the break effect on that entry, if the index is valid.

Attacks reach the break through the contact routines (SPEC-0036, Contact routines and cues). A contact routine runs when an attack family's volume, the hero's melee, Roberta's shot or a boss attack overlaps a breakable that is intact (state 1 or 3) and has a sub-record (+0x60, its volume record). The routines do not test +0x5A. Steps:

1. Script hook. The hook record for the entry's script index (+0x58) gets two argument words: (1, 0) on the melee and attack-family routes, (0, 1) on Roberta's shot. If a level script registered a callback there (RegisterScriptBreakable), the callback is started as a script thread (unless it is already running), and nothing else happens: no break and no cue.
2. Element gate. Let E_t be the entry's element class (+0x5E) and E_a the attacker's Mote kind. The contact passes when E_t = 0 or E_t = E_a. Five attack-family handlers use a fixed E_a (2, 3, 4, 5 and 6), three take E_a from their owner (two of these also pass when E_a = 7, Nova), and one has no gate. The hero melee route uses the Mote kind of the hero's slot-2 item and also passes when E_t = 7. Roberta's shot uses the shot's own kind and also passes when E_t = 7. The boss route has no gate. On a mismatch the family handlers and the shot do nothing; the hero melee route makes Tor recoil (Element recoil below), with no break and no breakable cue.

**Element recoil (High).** Where this spec says that Tor recoils with material 4, the hero's recoil material word is set to 4 and Tor's attack sub-state 13 is requested: the sword bounce of SPEC-0036 (hero animation 26, recoil cue 13 for material 4, and rumble preset 2 for 0.8 s at strength 0.6). The material-to-cue map is in SPEC-0028 (Blade recoil). This is not a hero action state. Roberta has no reaction: the request does nothing when she is the hero. The same recoil follows Tor's blade striking a solid volume (SPEC-0028, Blade recoil).
3. Break. The break effect runs with a piece push of 0.25 along d = unit(entry position (+0x30) - attacker reference point). Only a state-1 entry breaks; a state-3 entry stays intact.
4. Cue. One positioned cue plays on the main sound slot at the entry position (+0x30), chosen by the contact material (+0x5C): 2 -> cue 143, 3 -> cue 145, 4 -> cue 146, any other value -> cue 144 (SPEC-0031, Breakable contact cues). Roberta's shot always plays cue 144. The cue plays even when the entry did not break (state 3).

No damage, no status and no effect id is involved.

A separate break route gated on +0x5A also exists: when +0x5A is above 0 it copies the break source words (see below) and runs the break effect with a push of 0 and no cue; otherwise it does nothing. Nothing in the boot program or in the level modules reaches it, so it is dormant (Medium).

### Break effect (only in state 1)

The effect runs in this order. If the entry is not in state 1, nothing happens: no state change, no pieces, no drop, no sound.

1. The state becomes 2. The sub-record's s16 at +0x12 becomes 0 when the sub-record exists.
2. If the drop id (+0x56) is 0 or more, the drop action runs for that placement, at the break position (+0x30). Rules in Placed drops on break.
3. If the spawn key (+0x54) is 0 or more, a world-level spawn by key runs for that key at the break position (+0x30), with the entry's facing (SPEC-0029 Behavior 31): it searches the world's embedded character block first, then each created slot in slot order, for the first unbound summon-reserved record (summon byte nonzero) with that key, and binds it there. With no such record, no free character object or the active cap reached, nothing spawns. The request has its arrival flag set, so the spawned character plays animation 82 as it enters agro (SPEC-0029 Behavior 31). The spawn adds no placement (High).
4. For each piece record (count at +0x4C, array at +0x50, stride 0x70), one effect object is initialized from that record. Its setup receives the breakable position transform and the break-source words. The record links to a selected model-set part and supplies a relocated configuration link; the full meaning of its remaining fields and the final drawn transform are unknown (Piece record). The break's push value and direction are passed to each piece spawn: 0.25 along d from attack contacts (Break routes, step 3), or 0 from the dormant +0x5A route (Medium for the spawn; effect playback is in SPEC-0034).
5. If the signed value at +0x5A is above 0, a timed effect controller is started with the value converted to float and the break position. This value also gates the dormant hit-break route. Its precise time unit and perceptual meaning are unknown (Open question 11).
6. The entry's broken bit is set in the persistent broken set: the row of the current world index (world index word at world +0x5900), bit = script index (+0x58). Script indexes of 128 and above are not recorded (High).

The break effect plays no cue. The dormant +0x5A route copies the source words at world +0x5780, +0x5850 and +0x5854 into three globals that the piece spawns read, before its break (High for the copy; Low for its meaning). Whether the contact routines set these words is not described here.

### Draik breakable slots

The class-12 Draik controller owns nine 12-byte runtime slot records: a one-time release latch, a breakable index supplied by the level script, and a fixed bone index. In part order, the slots are left leg, right leg, tail, chest, left arm, right arm, left wing, right wing and head; their fixed bones are 25, 26, 4, 1, 9, 17, 38, 5 and 7. The level script supplies the breakable indices. Pack 1096 supplies 17, 20, 22, 15, 16, 19, 18, 21 and 23 in that order. [High for the record and fixed bones; Medium for which live boss instance consumes a pack's values].

On encounter setup, each valid part breakable is set to state 0. A release request checks the slot latch, sets it once, activates the associated breakable, copies the matching Draik bone transform to it and runs the normal break effect; it also requests effect 0xFC0 at the bone transform. The stage transitions release slots 0–2 (legs and tail), then slots 3–7 (chest, arms and wings), and death releases slot 8 (head). Level event 17 releases all nine slots once. [High for the calls and order].

The attack hit tests compare the hero's melee shape or projectiles against selected Draik bones and return a hit result; they do not directly release a breakable slot. The part releases are driven by the stage-transition and event paths above. The slot records belong to the active Draik controller, while the breakable indices are supplied by the level script. [High].

### Start mode and reset

Confidence: High for the rule and for the disc counts.

At each world load, after the type-29 chunk is stored, every second-list entry is set up:

1. The mode is the start-mode byte (+0x5F). If it is 0, the default mode is used. The default mode is always 0 (High). The original reads the default through the world's +0x58F8 word; the loader always sets that word (to a built-in default block at data address 0x2DA3D0, or to the pack's type-1001 payload), so the default is 0 on every world. With that word zero the original would read an unset word in low memory; the clean side uses 0.
2. The mode is applied only when the entry's persistent broken bit (Break effect, step 6) is set in the current world's row, tested with the script index (+0x58) (High). An entry whose bit is clear keeps its state from the pack (1 on disc).
   - mode 1: state 0 (start disabled).
   - mode 2: state 1 and drop id set to -1 (intact, no drop).
   - any other mode, including the default 0: state 1 (intact and breakable), with the volume flags word of its sub-record set to 9 (or 0x21 when the sub-record's type byte is 4).

Combined rule on disc data. The gate exists, but on disc it never changes the outcome: no entry has start byte 1 or 2 (the bytes are 0, 10, 20 and 30), so every entry with its broken bit set takes the "any other mode" branch, which gives the same intact state 1 that an entry with a clear bit keeps from the pack. A breakable recorded as broken therefore comes back intact and breakable, with its drop id unchanged, every time its world is loaded again; a new break runs the drop action again (what it yields follows SPEC-0021) (High). The only breakables that stay out of play after a reload are the two of an opened element-sealed chest, which that chest's set-up disables itself (Element-sealed chest, Set-up).

Broken pieces are not stored. Within a level the state stays until the level is unloaded (Medium). A break records the entry in the persistent broken set (Break effect, step 6), and the set is saved with the game (SPEC-0025, table T1). A clean implementation must still apply modes 1 and 2 as stated, for data that uses them. Entries with a script index of 128 or above are not recorded, so they always return in their pack state (High).

### Script groups (type-40 list entries)

Confidence: High, except where marked.

Every type-40 list entry is a script group. Its layout, the kinds found on disc and their counts are in SPEC-0028 section 11 (field table and kinds table); they are not repeated here.

#### Set-up poses by kind (at each world load)

Confidence: High for the poses and flags; Medium for the object roles. Object states used here: 9 seeks to a time, 8 is finished at the end of a seek, 4 is finished after deactivation and 10 is stopped. Object flags are the bytes at +0x1B6 and +0x1B7 of the object record. Setting a pose at time 0 is "start pose"; at the clip's last frame it is "end pose".

- Lock chest (kind 1): object 0 is set to state 9 at time 0 (start pose) when its collected row is clear. When the row is set, object 0 is at its last frame in state 8 (end pose). Both set its flag +0x1B6.
- Break chest (kind 2): object A is set to state 9 at time 0 and object B to state 10 at time 0. When the chest is collected, both of its breakables have their word at +0x40 cleared and a flag set, and object B is set to state 4 (finished, end pose). Object A stays at its start pose.
- Proximity fuse (kind 3): writes no pose. Each stage object below the trigger threshold gets its flag +0x1B7 set and its flag +0x1B6 cleared, and the breakable's word at +0x40 is cleared.
- Breakable chain (kind 4): the first member is left alone. Each later member has its word at +0x40 cleared. No pose is written.
- Reaction object (kind 5): object 0 is set to state 9 at time 0 (start pose), with flags +0x1B6 and +0x1B7 set.
- Dig site (kind 6): when its collected row is clear, object A is set to state 9 at time 0. When the row is set, object A is set to state 9 at time 0 and object B is at its last frame in state 8 (end pose).

Test vectors: a lock chest whose row is clear at load starts in state 9 at time 0; the same chest with the row set loads in state 8 at its last frame. A break chest collected at load has object B in state 4 and object A in state 9 at time 0. The word at +0x40 that kinds 2 to 4 clear is read as a breakable's active word (Medium).

- Set-up. At each world load every entry is set up by its kind word. The script call ReActivateScriptGroup(index) runs the same set-up again for one entry, and GetScriptGroupInstance(index) returns the entry; both find the entry by its position in the list (the group index), not by its key.
- Triggers. A group's script triggers are registered as script-instance triggers: each starts a callback thread keyed by the trigger index, and the trigger record carries the group index at its +0x30.
- Breakables. A group's breakables can get hooks with the semantics of RegisterScriptBreakable: a hit starts the hook instead of the default break and cue (Break routes, step 1).
- Key. The key (+0x08) is a row of the shared collected table (SPEC-0051). The row is tested at set-up only when the entry's drop pair list is non-empty (true for every chest, sealed chest and dig site on disc), and it is set to 1 when the group completes. The table is saved with the game (SPEC-0025).
- Built-in behaviour by kind: 1 lock chest (Lock chests), 2 element-sealed chest (Element gate on hits), 3 proximity fuse (Proximity fuse), 4 breakable chain (Switch groups), 5 reaction object (Reaction object), 6 dig site (SPEC-0051).
- Kinds 0, 7 and 12 have no built-in behaviour: set-up does nothing for them. They exist as data for level scripts, which act on them through the script-group calls and their trigger, object and camera lists. Readers on disc (High for the call sites; Medium-High for what each script does with the entry):
  - Kind 0 (49 entries in 15 packs): the scripts of 14 of these packs fetch entries with GetScriptGroupInstance. Fixed indices name kind-0 entries in 732 (entry 2), 735 (entry 8), 1077 and 1129 (entries 5 and 6), 1335 (entry 0), 1397 (entries 1 to 5) and 1398 (entries 1 and 2); 842, 1078, 1079, 1081, 1089, 1327 and 1411 pass a computed index. Pack 1247 has 6 kind-0 entries and its script makes no script-group call, so they are never read.
  - Kind 7 (12 entries: 758 has 5, 864 has 4, 790, 880 and 991 have 1 each): every one of these five scripts calls GetScriptGroupMainTrigger, which returns the collision block of the entry's first trigger. Kind 7 is therefore a trigger holder that the script uses as a named volume.
  - Kind 12 (61 entries: 1171 has 34, 1369 has 27): the script of 1171 registers a run callback and an initialize callback for group type 12 (RegisterScriptGroupType, SPEC-0030 row 274). The script of 1369 makes no script-group call and registers no type, so its 27 kind-12 entries do nothing.
  - Type registration (High): RegisterScriptGroupType(W, t, run, init) visits the script-group list in order. For each entry whose kind word is t: if the entry has at least one trigger, its first trigger index is registered as a script-instance trigger with the run callback and the entry's group index as owner (as RegisterScriptInstanceTrigger, SPEC-0030 row 261); an entry with no trigger gets no trigger registration. Then the init callback is called at once, during the registration call, with the entry and W. So init runs once per matching entry per world load, and run starts when the hero enters that entry's trigger. A trigger index registered earlier by RegisterScriptTrigger is replaced by this registration (both write the same trigger record).
  - 1171 kind-12 entries (High): the 34 entries are group indices 3 to 36. Entry 3 + k (k = 0 to 33) has one trigger, 29 + k, and three placed objects: a floating loop F = 17 + 3k, a sinking clip S = 18 + 3k and a rising clip R = 19 + 3k. They are floating objects that sink while the hero stands on them and rise again when the hero leaves.
  - 1171 init callback, per entry (High): InitObjOn(W, F, false); draw r = Random(0, 100) (both ends included); SetObjToTime(F, r / 100) (computed in double precision and stored as a float, so 0.00 to 1.00 s); LoopAnim(F, 1.0). The random start offsets the loops so that the 34 objects do not bob in step. S and R are not touched.
  - 1171 run callback (High for the steps; Medium-High that the collision record fields are the inside flag, the trigger index and the owner): it does nothing unless the hero is inside the trigger and the owner entry's first trigger is the firing trigger (always true for these entries). Then, as one script thread: DeactivateObj(F); ActivateObj(S, false); PlayAnim(S, wait, 1.0); then it waits one tick at a time while the hero is still inside; then DeactivateObj(S); ActivateObj(R, false); SetObjToTime(R, 0); PlayAnim(R, wait, 1.0); DeactivateObj(R); ActivateObj(F, false); LoopAnim(F, 1.0). The return to the loop starts at time 0 (no random offset).
  - Consequence in 1171 (High): its init first registers triggers 29, 30, 31 and 32 to the switch handler (SPEC-0042) and only then registers group type 12, which replaces those four trigger records with the kind-12 run callback. The switch handler is therefore never started from those triggers; triggers 29 to 32 sink and raise objects 17 to 28 like every other kind-12 entry.
  - No other script calls a script-group function on a kind-0, 7 or 12 entry.

### Switch groups (breakable chain, type-40 kind 4)

Confidence: High.

A switch group is a type-40 entry of kind 4 (39 entries in 8 packs on disc). Its members are the entry's breakables, in list order (2 or 3 on disc).

Set-up (at each world load): members 0 to n-2 get the chain hook; members 1 to n-1 are disabled (state 0: not drawn and not hit). The last member has no hook.

Chain hook (a hit on a member that has the hook; the hook replaces the contact route of Break routes, so the hit plays no breakable cue and gives no push):

1. Scan the entry's member list in order and find the first member whose state is 1 or 3. If none is found, or that member is in state 3, nothing happens.
2. Otherwise that member breaks (Break effect, with no push).
3. If a member follows it in the list, that member is armed: state 1 (drawn, with its volume flags word set to 9, or 0x21 for a type-4 volume), then state 3 at once, then a wait of 0.4 s (24 ticks), then state 1. The new stage is therefore shown at once but cannot be broken for 24 ticks.

The last member breaks through the normal contact route (Break routes), with its hit cue.

The lookup of the entry is by position: the group index selects the k-th entry of the type-40 list, for k at or above 0 and below the count. The entry's key is not compared on this route.

Timing: the 0.4 s wait is a code constant (not a disc word).

### Element gate on hits (element-sealed chest, type-40 kind 2)

Confidence: High for the gates, the hook and the order of the open; Medium for the start of effect 791 and the exact drop point.

This gate belongs to the element-sealed chest: a type-40 entry of kind 2 (39 entries, one each in 39 packs). Its breakable 0 is the chest body and its breakable 1 is the seal. The chain step above does not test elements. Attacks on other breakables use the contact routines' own gate (Break routes, step 2). Terms:
- E_a is the Mote kind of the hero's slot-2 item (0 when there is none; 7 for Nova).
- E_t is the element class (the byte at +0x5E) of breakable 1 of the entry. E_t is 0 when the entry has fewer than two breakables.

**Set-up** (at each world load): object 0 is drawn (state 9, time 0) and object 1 is drawn stopped (state 10, time 0); trigger 0 gets the prompt thread; breakable 0 gets a hook whose only effect is Tor's recoil with material 4 (Element recoil); breakable 1 gets the seal hook. When the entry's key row is already set (an opened seal), both breakables are disabled (state 0) and their start byte is set to 1, object 1 is hidden (state 4) and the entry's effect 0 is stopped.

**Seal hook** (a hit on breakable 1):
1. The hit fails if the entry is missing, or if the hook record's word at +0x24 is zero: Tor recoils with material 4 (Element recoil), and nothing else happens.
2. The hit passes if E_t = 0, or E_a = E_t, or E_a = 7.
3. Otherwise the hit fails: Tor recoils with material 4, with no break, no breakable cue and no effect. Roberta has no reaction.
4. On a pass the open runs.

**Open**, in order (waits in seconds):
1. Breakable 0 is made active (state 1); breakable 1 breaks (Break effect); effect 0 of the entry stops.
2. The hero is holstered (weapon put away) and the HUD is reset.
3. Wait 0.3. Camera 0 of the entry plays, and the thread waits for it. Sound cue 182 plays.
4. Wait 1.3. Object 0 is hidden; object 1 is shown and looped at speed 1.0.
5. Wait 2.0. The world display mask is set to 0x10000; wait 0.8; 0x20000; wait 0.8; 0x40000; wait 0.8; then 0.
6. A one-shot detached effect request uses definition 791 from shared effect group 3, with the transform at X + 2.9 d (locator 0; see step 8). When the group and definition are available, the effect system starts it and releases it in detach mode 2; if the group or definition is unavailable, no particles are produced (SPEC-0034, section 3.16; High for the request and mode, conditional on the effect data resolving). Breakable 0 breaks. Object 1 is hidden and object 0 shown. The hero plays animation 192.
7. Wait 2.3, then wait 1.0. Camera 1 of the entry plays when it has one. The hero plays animation 186.
8. When locator 0 exists, the placement of pair[0] is requested at X + 2.2 d and launched along −d, where X and d are the position and forward direction of locator 0. The separate effect transform is at X + 2.9 d. The character of pair[1] is activated (as for a lock chest). The locator-0 fallback never happens on disc: each of the 39 packs with an element-sealed chest (and each of the 146 with a lock chest) has at least one entry record, so locator 0 always exists; a rewrite may skip the placement if the list were ever empty. (High for the coordinates passed to the spawn and effect setup; effect 791 playback depends on the effect data resolving, step 6.)
9. Wait 2.0. The normal camera returns. The key row of the entry is set to 1.

The fixed waits add up to 11.3 s, plus the wait on camera 0.

**Prompt thread** (Square context prompts 29 and 30). The thread starts on a repeated fire of trigger 0 (trigger state 2) and runs each tick while the main hero stays in the volume and breakable 0 is intact. On each tick it uses the mismatch test E_t != 0 and E_a != E_t:
- while the mismatch holds, context prompt 29 is shown on every tick until Square has been held once; it shows again after E_a has matched;
- otherwise context prompt 30 is shown on every tick until Square has been held once.
These are context prompts (SPEC-0023), not sound cues. The thread does not exempt Nova (E_a = 7), so a hero with Nova at a seal with E_t = 3 sees prompt 29, while the seal hook passes Nova.

Counts on disc: a switch entry has 0 members in 224 of 458 entries, 1 member in 155, 2 in 55 and 3 in 24 (61 level packs have members). Across the 337 members, member index 0 is 0 in every case and member index 2 is 0 in every case. Member index 1 is nonzero in 40 of the 79 groups with two or more members (values 1: 9, 2: 6, 3: 6, 4: 6, 5: 7, 6: 6). All 40 are in two-member groups (15 of the 55 have class 0); every three-member group has member-1 class 0. These 40 nonzero member-1 references are exactly the 40 breakable entries with a nonzero element class; the other 39 distinct member-1 entries have class 0. Since setup hooks members 0 through n−2, member 1 is the final unhooked stage in a two-member chain, where its nonzero class gates ordinary contact after the 24-tick guard. In three-member chains member 1 is hook-controlled and class 0, so hits advance the chain without the ordinary element gate. (Open questions 22 and 23; High for the serialized data and runtime ordering.)

### Lock chests

There is no chest object type. A chest is a type-40 group entry (SPEC-0028, section 11) of kind 1 (Square-press chest) or kind 2 (break chest opened by a hit with a matching Mote element). A lock chest is a kind-1 group whose group index is also an id in the lock table.

**Shared collected table (High).** Kinds 1 and 2 use the same collected table as kind 6, the treasure dig (SPEC-0051). A chest's key (+0x08) is its collected entry, in the range 1 to 199 (one kind-1 chest has key 0); dig sites use 200 and above. Chests need no treasure map. Both chest kinds end their open with the drop action on their reward list and then set their collected entry to 1. On a later world load the set-up shows the opened pose for a collected chest: a lock chest's object starts at its last frame (High).

**Set-up (kind 1, at each world load; High).** Breakable 0 (the chest body) is made active and not breakable (state 3), with a hook whose only effect is Tor's recoil with material 4 (Element recoil). Trigger 0 gets the chest thread. Object 0 (the chest) is drawn in state 9 at time 0, or, when the key row is already set, at its last frame in the finished state 8. Object 1 is not used.

**What stops a character at a chest (collision; High for the rule, the shape and the numbers; Medium for the top of the cylinder and for characters other than the selected hero).** A chest has no collision record of its own and nothing bound to its objects' matrices:
- Neither placed object of a kind-1 entry has an inline model set or an inline type-11 header (all 146 kind-1 entries on disc; 143 list two placed objects and 3 list one), so no frame record, no inline primitive and no placed-object push (SPEC-0033, Behavior 43) belongs to them. No registry model is involved.
- The entry's trigger volume is not a solid. Trigger 0 of the entry is a sphere with flags 0x0008 (fire only, bit 0 clear; pack 951: centre (-14.97, 13.53, -21.07), radius 3.69). It starts the press thread when the hero is inside it and pushes nobody.
- The solid is the volume of breakable 0, the chest body. It is a kind-3 cylinder record (Breakable volume shapes), the same record that gives the breakable its melee test. The loader sets its flags word to 9 (bit 0 solid, bit 3 body-contact fire) and the set-up above leaves the entry in state 3. The motion step offers every state 1 or 3 breakable of the character's grid cell to the kind-3 volume handler (SPEC-0018, Breakable cylinders against characters). Because bit 3 is set, a hit fires the volume's script trigger number (+0x1C) with the character as actor, and because bit 0 is also set it then pushes the character out. The number is a stale authoring value: in all 224 chest breakable volumes on disc it is below the pack's trigger count and no fire-bit volume of the pack carries it, so the fire writes an unregistered trigger record and starts nothing (pack 951: number 24; its module registers indices 0 to 8 only; Medium).
- Shape. An upright cylinder with flat ends: the centre of its base disc is the volume's point (+0x30), it rises along the axis (0, 1, 0) for the length at +0x24, and its radius is at +0x3C. It is a fixed record: its base height equals the height of object 0 in every kind-1 entry on disc, and its base centre lies 0 to 1.34 (median 0.69) horizontally from object 0's origin. Of the 146 kind-1 entries, radius 2.59 occurs in most (range 1.96 to 3.14) and length 2.06 in 105 (the others 3.1 to 5.05; every axis is vertical).
- The character is tested as an upright cylinder of fixed radius 1.0 and height 3.5 standing on its position (not its own body radius). It touches the chest when its horizontal distance L from the axis is at most r + 1.0 and its feet height is from (base height - 3.5) to (base height + length). The response is the ordinary slide response (SPEC-0018, Slide response): the normal is the horizontal unit vector from the character toward the axis, the depth is L - (r + 1.0) (negative inside), and the character ends exactly r + 1.0 from the axis, losing its velocity and force along the normal. A grounded hero is therefore stopped at a ring of radius r + 1.0 round the cylinder axis, on every side, and slides along it.
- It does not move or go away. Nothing in the open sequence writes the breakable (the lid animation moves object 0 only), so an opened chest blocks exactly as a closed one, and it blocks again after a reload.
- The hero does not stand on it. The cylinder gives no ground candidate (the ground probe reads only model-set patches), and the contact sets no floor flag for a horizontal normal. A hero who falls in a jump state (7 to 11, 43 to 46) onto the top, with feet within 1.5 of the top, is handled by the landing break first: the chest's hook runs (Tor recoils with material 4 and enters attack sub-state 13; Roberta does nothing) and the push is skipped on that tick (SPEC-0036, Hero collision-kind contact tests). Lower than that he is inside the cylinder and the horizontal push moves him out to the ring. Quirk (Medium): when his axis is within 1.0 of the cylinder axis and his centre is within 0.4 of the top end of the overlap, the response uses the vertical axis and moves him down by the overlap (0 to 0.4); he never rises onto the top.
- Who is blocked. The selected hero and the other characters that walk the breakable list (region characters, enemies; they use the same fixed 1.0 by 3.5 body; Medium-High). The hero that is not selected and the petpet do not walk that list and pass through (Medium-High).
- Element-sealed chest (kind 2). Each of its two breakables (body, seal) is a flag-9 cylinder of this kind (body radius 1.17 and length 1.27 in 38 of 39 packs; seal radius 1.17 and length 5.17 in all 39), so both stop a character until the seal breaks (a break clears the volume flags, which removes its push). Object 0 of every sealed chest also carries an inline type-11 header with one cylinder of flags 0x0001 (radius 1.86, length 1.93, base at the object's origin); the placed-object push (SPEC-0033, Behavior 43) moves a character out of it with the character's own body radius (1.175 for a hero), so the reach round it is 3.035 (Medium).

Worked example (pack 951, the first-house chest; real disc values). Breakable 28 of the pack is breakable 0 of its type-40 entry: base centre (-14.9674, 11.9709, -24.7527), axis (0, 1, 0), length 2.0594, radius 2.5929; object 0 is drawn at (-14.97, 11.97, -24.06). The ring radius is 2.5929 + 1.0 = 3.5929; contact needs feet height from 8.4709 to 14.0303.
- Hero on the floor (feet 11.971) at (-14.967, -21.5): L = 3.2527, depth -0.3402, normal (0, -1): he is moved to z = -21.1598.
- Hero at (-13.0, -22.5): L = 2.9909, depth -0.6021, normal (-0.6578, -0.7532): he is moved to (-12.604, -22.0465).
- Hero at (-14.967, -20.7): L = 4.0527, no contact.
- Falling Tor, feet 14.0 over (-12.967, -24.753) (L = 2.0): landing break, chest hook, recoil, no push that tick; once his feet are at or below 12.53 the push moves him out to the ring.
Default if unresolved (rewrite choice): treat the breakable cylinder as a solid for the selected hero and for region characters exactly as above, and do not give the chest top a floor.

**Lock add (LockChest).** Stores (id, key item, message) in the first free lock slot. A slot is free when its id is -1 (High). If both slots are used, the call logs a message that the table holds at most two locks, and stores nothing. The call's three arguments are the stored values.

**Lock lookup.** For an id, returns the key item of the matching entry, and writes the message id through an output argument. Returns -1 when the id is not in the table.

**Lock remove.** Sets the matching entry's id to -1. The routine has no caller anywhere, so no lock entry is ever removed; the table keeps its entries until the world is rebuilt (High). LockChest has no duplicate check (Medium).

**Press rule.** Each tick, a press-activated group with a lock runs this sequence:
1. Gate: the thread runs while the trigger record's state word is nonzero and its object word is 1 (the main hero). The chest object (object 0) must be drawn, not animating (its animation state is not 0, 2, 5 or 6) and not finished (not 4, 6 or 8). The Square caption's shown count (SPEC-0023, section 33) must be zero: if any Square caption (for example "Talk" or "Save Game") was already set earlier in the same pass, the chest thread skips its prompt and its press test for that tick (High for the test; where the chest thread runs in the pass relative to the hero update is not settled). The hero-state test must be false (the hero is not occupied).
2. Show the prompt (context 6, "Open") on every tick that passes the gate, before the test. The chest thread sets it itself. The prompt is the Square prompt (High; see Button query).
3. Press test: code 0 (Square, mask 0x0080) on pad slot 0's newly-pressed word, so the press counts only on the pass in which Square goes down (see Button query). The hero's height above ground must be below 0.1 (units not confirmed). The pad-control lock must be clear.
4. Lock: look up the group's index (group object +0x30 is the id). If the lookup gives a key item whose id is above 0:
   - if the hero does not have the key item, show the lock message (the message id from the table) and stop;
   - if the hero has the key item, remove all of that item from the hero, then continue to step 5.
   If the lookup gives no key, or the key id is 0 or -1, go to step 5 directly.
5. Open (kind 1; High), in order: camera 0 of the entry plays when it has one; the hero is moved to locator 0 of the entry when it has one (placed on the ground and turned to the locator's facing). With that locator's position X and facing direction d, two points are used: A = X + 2.4 d + 0.4 up (the drop point) and B = X + 2.9 d (the effect point). The chest's open clip starts (the object is switched on and played once at speed 1.0). Effect 790 of shared effect group 3 starts at B, facing d, but only when that effect group is loaded. Sound cue 142 plays on the main slot. The hero plays one-shot animation 162, and the chest waits for it.
6. Reward (High for the order): if reward [0] of the entry (the same two-slot reward list as a dig site: placement id, character id) is 0 or more, the drop action (SPEC-0021; Placed drops on break) runs on that placement id at A, flying along -d, so coins appear 0.4 s apart. If reward [1] is 0 or more, that character is activated.
7. Close (High): wait 1.0 s; camera 1 of the entry plays when it has one; wait 1.5 s; when no dialog is active, the normal camera returns; then the chest's collected entry (its key) is set to 1.

No repeat press (High). After the open the chest object is first animating (its open clip) and then finished, and the gate of step 1 refuses both, so the open never runs again in this visit. After a reload the set-up starts the opened chest at its last frame (finished), so the gate refuses it there too and no prompt shows.

Rewards: the open sequence pays its reward placement (step 6), a placement of the same pack. Three chest rewards are treasure maps, each a placement with start flag 4 (never spawned at region load; SPEC-0021), so it appears only through its chest: map 20 in a kind-1 chest in pack 1122, and maps 23 and 34 in kind-2 chests in packs 947 and 1247 (SPEC-0050).

**Break chest (kind 2) (High).** The set-up registers the chest's trigger, links two breakables and shows the collected pose when the chest is collected. The chest opens on a hit on the seal whose Mote element matches the seal's element class (+0x5E); a mismatch makes Tor recoil with material 4 (Element recoil). The open plays sound cue 182, fetches effect 791, steps the world display mask, and plays hero animations 192 and 186. It then runs the drop action on reward [0] and sets its collected entry to 1, as kind 1 does. The full set-up, hook, open sequence and prompts are in Element gate on hits.

### Proximity fuse (type-40 kind 3)

Confidence: Medium for the rules; High for the thresholds and the hero-state gate.

A proximity fuse is a type-40 entry of kind 3 (36 entries in 10 packs on disc). It is live only when the entry has 32 placed objects (the stages) and 1 breakable: 9 entries, one each in packs 735, 783, 880, 1172, 1184, 1230, 1250, 1363 and 1409. The other 27 entries have empty lists and do nothing.

Set-up (at each world load): the 32 stage objects are hidden (gate byte 1, not drawn) and the breakable is disabled (state 0). The fuse is armed.

Each world tick, while the game state is below 9 (SPEC-0026) and the fuse is armed, let d2 be the squared distance from the fuse's position to the hero, T the countdown timer (seconds, 0 at set-up) and U a second timer:
1. d2 < 7.0: the fuse fires at once.
2. d2 >= 180.0 (distance about 13.4): T is reset to 0; when the countdown had started, stages 0 to 2 are shown again and the countdown stops.
3. Otherwise, when the countdown has not started, it starts only when the hero is not standing (state 1), not slow-walking (state 2) and not sneaking (states 22, 39 and 40). Once started it continues whatever the hero does.
4. On the first counting tick stages 3 to 5 are shown. Each counting tick T = T + dt (dt = the tick word, 1/60 s; SPEC-0007).
5. While T > 2.8 s: on ticks where floor(5 T) is even, a shake value of 0.5 times a sine of frac(5 T) is set; on other ticks it is 0. The active-stage draw pass consumes this value in the first component of a four-component render parameter, as shake + 0.5; the other components are 0.5, 0.5 and 1.0. The visible result of that parameter needs a runtime check. Default if unresolved (rewrite choice): treat it as a modulate tint on the stage objects in which the first component, between 0.5 and about 0.92, flickers while the other components stay at the neutral 0.5 and 1.0, so the shown stages flash reddish from 2.8 s on; no gameplay rule depends on it.
6. While T > 7.0 s: stages 6 to 30 are shown and U = U + dt. When U > 0.3 s the fuse fires.

Fire: an effect starts at the fuse position + (0, 2, 0); sound cue 178 plays on the main slot; the breakable takes the fuse's transform and runs the break effect (Break effect, so its drop and broken bit follow); the fuse is disarmed and does not run again in this visit.

The thresholds 7.0, 180.0, 2.8, 7.0, 0.3 and 0.5 are code constants.

### Reaction object (type-40 kind 5)

Confidence: High.

A reaction object is a type-40 entry of kind 5 (29 entries in 4 packs on disc).

- Set-up (at each world load): object 0 is switched on and looped at speed 1.0; trigger 0 gets the reaction thread.
- Reaction (each fire of trigger 0): object 0 is hidden; object 1 is shown, played once at speed 1.0 and waited for, then set to time 0 and hidden; object 0 is shown and looped at speed 1.0 again.
- A later fire runs the reaction again once the previous run has ended; a fire during a run does not start a second one.
- Its key is 0 in every kind-5 entry on disc (SPEC-0028 section 11), so the collected table is not involved.

### Button query (press input)

The button query takes a button code and two flags. All 16 codes map to the pad masks below (confirmed):

| Code | Pad mask | Code | Pad mask |
| --- | --- | --- | --- |
| 0 | 0x0080 (Square) | 9 | 0x0400 (R3) |
| 1 | 0x0010 (Triangle) | 10 | 0x0002 (R2) |
| 2 | 0x0020 (Circle) | 11 | 0x0008 (R1) |
| 3 | 0x0040 (Cross) | 12 | 0xFFFF (any) |
| 4 | 0x0800 (Start) | 13 | 0xF000 (D-pad) |
| 5 | 0x0100 (Select) | 14 | 0x00F0 (face buttons) |
| 6 | 0x0200 (L3) | 15 | 0x000F (shoulder buttons) |
| 7 | none (always false) | 8 | 0x0004 (L1) |

- Both words belong to pad slot 0's own record (which starts at +0x40 of the input block), not to the combined menu snapshot. The second argument false reads the held word, at +0x190 of the input block. True reads the newly-pressed word, at +0x194: the buttons that went down this pass (held now, not held at the previous read).
- The third argument nonzero makes the result false.
- A nonzero player-control word at +0x44 makes the whole query false. This check comes before the mask is used.
- A nonzero lock word at +0x20 of the player-control record forces the mask to 0 (the query is false).

The press rule uses code 0 on the newly-pressed word. Code 0 is Square (0x0080): the game builds its held word as NOT (packet byte 2 x 256 + packet byte 3), so packet byte 2 is the high byte and byte 3 bit 7 (Square) is 0x0080. The pad library's own little-endian word, in which 0x0080 would be Left, is not what the game uses (High). Button names for every bit are in SPEC-0033, Controller map.

### Placed drops on break

A breakable's drop id is a placement id. When it is 0 or more, the drop action runs for that placement on the break tick, in the break effect's second step (SPEC-0021, "When a drop roll runs"). The break route has these properties:
- no luck gate (the gate at placement +0x28 is used only at region start and by scripted adds);
- no equipment veto;
- no start-flag check, so placements with start flags 2 to 4 can drop through a break.

The drop id is mapped to a region index and a placement index. A region value of -1 means the current region. The drop then follows the SPEC-0021 flow for the placement's key:

1. If the key (placement +0x24) is below 0 (a group placement):
   - money roll: the group's money result (SPEC-0021, step 1). If that gives no amount, the placement's fixed count (+0x2A) is used when it is above 0. If an amount above 0 exists, it is split into coins (100, 25, 10, 5, 1, spawned in that order) and nothing else drops.
   - item roll: if there was no amount, the group in +0x22 is rolled (SPEC-0021, step 4). If it gives an item, that item spawns at the placement's position.
2. If the key is 0 or more: the item with that id spawns directly, at the placement's position.

Spawn parameters for the direct item and the group item: 0.25 and 1.0 are code constants; their meaning is Low.

### Movable blocks

Confidence: High for placement, the direction words, the push start, the second clearance test and the per-tick motion; High for the push tick count n; Medium-High for the character contact; Medium for the trigger-volume shapes.

A movable block moves only in whole pushes. Each push moves it by its step length L along one of four directions, spread evenly over n ticks. The 3.0 value is not a speed: it is the ceiling offset of the ground probe used when the block is placed.

**Source (High).** At world load each n1 entry of the type-11 chunk whose box primitive (entry +0x20) has flag bit 1 (0x0002) set becomes one 0xB0-byte block record, in entry order; entries without a box, or whose box has bit 1 clear, build nothing (Where it lives).

**Placement (init, move to locator, reacquire; High).** The block's geometry is its box record (the n1 entry's primitive, SPEC-0028 section 6.1): a reference corner (record +0x30), a first axis (record +0x40), a second axis (record +0x50), a vertical axis (record +0x60) and three extents (vertical at +0x24, along the second axis at +0x28, along the first axis at +0x2C). A placement does this, in order:
1. Probes the ground at the corner's x and z with a ceiling of the corner height + 3.0 (ground probe of SPEC-0018) and writes the found height into the corner. The block's footprint word (+0x9C) is first set to 0 and is the probe's chosen-candidate slot, so it receives the patch entry the probe chose: the accepted candidate, or, when none is accepted, the last rejected one (SPEC-0018, Ground probe, step 8); it stays 0 only when the probe returns early (no grid cell, or a cell without entries). This patch is what limits every later push (Push start, step 2). The probe is the full ground probe: a floor that belongs to the inline model set of a placed object counts while that object is drawn (SPEC-0018, Ground probe, step 4). In pack 1324 the three bells are placed at locators at height 33.08, and the probe from 36.08 finds 31.19 on placed object 21's patch, not the 0.0 of the pack's own patches (SPEC-0041, 1324).
2. Copies the three axes into the block record (A = the first axis, B = the second axis, vertical = the third), each with w = 0.
3. Sets the position and the home to corner + 0.5 S A + 0.5 S B, where S is the entry's size (+0x10); the displacement (position minus home) is zero.
4. Rewrites the w lane of each of the box record's three axes to minus the dot product of that axis with the new corner, so the plane constants of the box follow the placement.
5. Sets all four direction-allowed words to 1, the moving flag to 0, both counters to 0, the stored direction to -1 and the active word to 1. The step length is L = 0.5 S.

The 3.0 value is the probe ceiling, not a speed. Entry defaults (a rule that never applies on disc): when the entry has no inline mesh (its +0x1C word is 0) and its +0x18 word is negative, the entry first receives size 3.5, height 5.0 and the box extents 5.0 (vertical), 3.5 and 3.5, and its +0x18 word becomes the registry model id 206; L is then taken from the size read before that overwrite, while steps 3 and 4 use the new size 3.5. All 90 built blocks have an inline mesh (only the unbuilt entry of pack 1344 has none), so on disc S, H and the extents are always the stored values and a second placement gives the same L (High).

Entry words +0x00 to +0x0C (four floats, one per push direction; High that nothing reads them). The same default rule writes 5 S into any of them that is 0. On disc they hold 5 S in 86 blocks and other values in packs 1176 (0.551, 7.693, 0.451, 3.857), 1249 and 1298. They look like per-direction travel ranges, but no routine reads them: they do not limit a push. A clean implementation ignores them.

On disc A is the world +Z unit vector and B the world +X unit vector in 89 of the 90 blocks. The block of pack 1176 has A = (0, 0, -1) and B = (-1, 0, 0). The vertical axis is (0, 1, 0) in all of them. The box record's extents equal S and H in nearly all blocks (for example pack 1381 block 1 has 5.0 along its first axis against S = 5.5).

**Direction-allowed words (High).** Nothing but the placement writes them, and the placement sets all four to 1. No script call, hero state or other routine changes them, so every push direction is always allowed. The test in the push start is kept for completeness.

**Push start.** The hero's block state (SPEC-0033, Behavior 4A.2) asks for a push with the block, a direction D (0 = +A, 1 = +B, 2 = -A, 3 = -B), the face index f of the face the hero holds (0 to 3 in the order A, B, -A, -B; SPEC-0033 Behavior 15A) and the tick count n. The hero's actions give D from f: the push (action 42) uses D = f; the pull (action 41) uses D = (f + 2) mod 4; the side move of action 144 uses D = (f + 1) mod 4; the side move of action 145 uses D = (f + 3) mod 4. An invalid face (f = -1) gives D = -1 and nothing runs. The request repeats on each tick of the clip window in which SPEC-0033 allows the move, until a push is accepted; a refusal leaves the block and the hero's clip unchanged. A push is accepted when all of these hold, in order:
1. The block is not moving, and the direction-allowed word of D is non-zero (always true; see above).
2. Footprint step (High). It runs whenever the block's footprint patch (+0x9C) is non-zero, which on disc is every block (Placement, step 1); a block whose patch is 0 skips it. With D the unit vector of the direction (+A, +B, -A or -B):
   - Block front: F = position + D x (L + 0.5 S + 0.1) must pass the footprint patch's list test (the candidate list test of SPEC-0018, Ground probe, step 3, on x and z, with the patch's outline records; an empty outline passes). F is 0.1 beyond where the block's leading face will be after this push, so a push is accepted only when the whole step keeps the leading face at least 0.1 inside the outline. Failing it refuses the push.
   - Hero point: H = (selected hero's position) + D x (L + 0.75) x 1.1. If H passes the same list test the push is accepted (subject to step 3). Otherwise the ground heights (full ground probe, each point's own height + 8.0 as the ceiling) at the block's position and at H must differ by no more than the step limit 0.75 (ELF data word 0x2EB2C0); a larger difference refuses the push. With the hero on the grab spot (Solid for characters; SPEC-0033 Behavior 4A.2), H is position + D x (0.05 S - 0.675) for a push (inside the block), position + D x (1.05 S + 2.325) for a pull, and the grab spot + D x (0.55 S + 0.825) for a side move.
   This is the footprint variant of SPEC-0018 (Move check) and SPEC-0040 Behavior 11.
   What this means in play (High): a block can only be pushed within the outline of the floor patch it was placed on. The outline is a closed polygon of vertical frame records (walls) around that patch, so each block has authored borders; it refuses the first move whose leading face would end closer than 0.1 to the border (or beyond it), so it comes to rest between 0.1 and L + 0.1 inside the border, and the hero then plays the push clip in place. The outline is fixed at the last placement: a push does not re-probe, so the block keeps its patch while it moves. Walls of other entries inside that outline are not tested (see Push limits on disc).
3. Second clearance test (below): no other block may be in the way.
4. Then the moving flag is set to 1, i to 0, the direction D is stored and n is stored as a signed byte. n comes from the caller: n = (key count of the hero's current clip - the whole part of the clip cursor - 2), kept as a signed byte. Only for the push clip (action 42) is the count divided by the half-rate word 0x368FA4 (0.5), so a push lasts twice as many ticks. Pull and side moves (actions 41, 144 and 145) are not doubled, so they finish in about half the clip's remaining time (quirk; reproduce). The count is taken when the hero's block state starts the move, once per clip cycle (SPEC-0033, Behavior 4A.2).

**What does not limit a push (High).** Nothing else gates a push or a block's motion: the entry words +0x00 to +0x0C are not read; there is no limit relative to the home position, no step count, no region or cell bound and no navmesh test; the second clearance test reads only the world's block list (no cell list, frame record or primitive); the tick mover (Per tick) adds the scheduled step without any test; and the hero's block state (SPEC-0033, Behavior 4A.2) makes no wall, ledge or ground test in its grab phase (its only world-aware gate is the hero point of step 2). Trigger volumes that a moving block fires call level scripts, which can move, re-place or deactivate a block (pack 1324 freezes a bell on its plate and re-places the others; pack 1397 deactivates the block that fires its solve trigger; SPEC-0041, SPEC-0042), but no script on disc stops a block at a border.

**Second clearance test (High for the rule; Medium-High for the intended meaning).** The test scans every other block in the world's list (active or not; the pushed block is skipped) and refuses the push as soon as one block is in the way. It works on the world x and z of the block positions and assumes that A points along world +Z and B along world +X, as they do for 89 of the 90 disc blocks; the block of pack 1176, whose axes are reversed, still gets the +Z and +X tests (quirk; reproduce). Four relations are used. Each takes a first block, a second block and a flag w (0 or 1); r is the size S of the first block, dx = x(second) - x(first) and dz = z(second) - z(first):
- Forward-Z: true when dz >= 0, |dx| < r - 0.05 and dz <= r + 0.5 (w = 0) or dz <= 1.5 x (r + 0.5) (w = 1).
- Forward-X: the same with the roles of x and z exchanged (dx >= 0, |dz| < r - 0.05, dx <= r + 0.5 or 1.5 x (r + 0.5)).
- Side-Z: true when dz >= 0, dz <= r + 0.5 and the x window overlaps the open interval (-(r - 0.05), r - 0.05): with w = 0 the window is [dx, dx + r], with w = 1 it is [dx - r, dx].
- Side-X: the same with x and z exchanged.
With P the pushed block and Q another block, the relation chosen by D and f is:

| D | f = 0 | f = 1 | f = 2 | f = 3 |
| --- | --- | --- | --- | --- |
| 0 (+A) | Forward-Z(P, Q, 0) | Side-Z(P, Q, 0) | Forward-Z(P, Q, 1) | Side-Z(P, Q, 1) |
| 1 (+B) | Side-X(P, Q, 0) | Forward-X(P, Q, 0) | Side-X(P, Q, 1) | Forward-X(P, Q, 1) |
| 2 (-A) | Forward-Z(Q, P, 1) | Side-Z(Q, P, 1) | Forward-Z(Q, P, 0) | Side-Z(Q, P, 0) |
| 3 (-B) | Side-X(Q, P, 1) | Forward-X(Q, P, 1) | Side-X(Q, P, 0) | Forward-X(Q, P, 0) |

For D = 2 and 3 the roles are exchanged, so r is the size of Q. The push (f = D, w = 0) looks for a block within r + 0.5 ahead; the pull (D opposite to f, w = 1) looks 1.5 x (r + 0.5) ahead, which leaves room for the hero; the side moves look ahead by r + 0.5 over a side window that extends towards the hero's side by one block size. Any other pair (D or f outside 0 to 3) tests nothing and passes. The scan returns "blocked" at the first block for which the relation is true.

**Per tick.** The movable-block list is walked by the world tick, after the object animations of SPEC-0028 and before any character update (SPEC-0018, Movable-block list). For a block whose active word (+0x74) is non-zero: if it is moving and i < n, position += D × (L / n), the position-minus-home vector is updated and i += 1; if it is moving and i ≥ n, moving = 0 with no motion (so the flag clears on the tick after the n-th step). Effects attached to the block follow it. Per-tick speed is L / n; a push covers L in n ticks. With n of 0 or below the block does not move: the first tick finds i at or above n and clears the moving flag.

**Climb onto a block.** The hero's climb onto a block (state 58) is gated by the climb-onto-block fit test of SPEC-0033 (Behavior 15): the hero walking at a block face, the block top 0.2 to 3.5 above the hero's feet, the facing toward the face and the stick pushed into the block. The block top it reads is the box as loaded; a push does not update it. The approach and the clip are in SPEC-0033, Behavior 51 (state 58) and Behavior 13A, item 6: the hero is drawn to 0.75 outside the face and 1.2 below the top, and clip 134 lifts his feet to about 0.02 above the top and 1.72 inside the face (both heroes).

**Climbing (what starts a climb or a hang; High).** No placed object, object kind, trigger volume or script call starts a climb or a hang, and no interactable has a placement field for one. Climbs come only from:
- the world's type-10 records (SPEC-0006): ladders (type -3; a face record and a top record each) and climb panels (type -2; walls, top faces and ceilings). They are static world data, loaded with the world and never moved, switched or hidden by a script; the hero's grab tests read them every tick (SPEC-0033, Behaviors 22, 22A and 13A);
- the movable blocks of this spec: the climb onto a block (state 58, above) and the hang from a block's top edge (state 19, SPEC-0033 Behavior 51), both through the block face finder. An inactive block offers neither;
- the ledge edges of the frame records (SPEC-0033, Behaviors 37, 49 and 51), which give the ledge hang, the drop into a hang and the mantles.
The Farm crate (below) is a block placed as a step to a ladder.

**Activate.** Writes the int argument to the block's +0x74 word, for a valid index. A block is built active (the placement sets the word to 1), and a script makes it inactive with ActivateMovableBlock(k, false). An inactive block does not move or run its trigger test (the tick returns at once), is not drawn, is left out of the character contact pass and its current-block scan (so it is never a hero's current block and shows no grab prompt), the projectile pass and the pull-back clearance, and is still counted by the second clearance test of a push (that scan reads no active word; High). Pack 1381 keeps its blocks 0 to 5 inactive until their stations are solved (SPEC-0042).

**Move to locator.** For a valid block index and a valid locator index, centres the block on the locator position X (+0x30 of the locator record): the box record's reference corner becomes X - 0.5 S x (the vector at record +0x50) - 0.5 S x (the vector at record +0x40), where S is the entry's size (+0x10). Then the block is placed again (Placement), which re-probes the ground and sets home and position to X at the new ground height.

**Reacquire height patch (High).** Centres the block on its own current position and places it again: it runs the move-to-locator rule with X = the block's current position (so the corner becomes position - 0.5 S A - 0.5 S B), then the placement. The result is that the ground height under the block is probed again (ceiling = the position height + F, where F is the float the script passes to ReacquireMovableBlockHeightPatch: 3.0 in pack 1324, 1.0 at the three trigger sites of pack 1397 and 500.0 at its per-tick site; SPEC-0030), the home becomes the new position and the displacement from home is zero. The horizontal position does not change.

**Sound index.** Stores the int argument at world +0x2CCC. A later sound call reads that word as the movable-block sound index (Medium).

**Make unmovable.** The script call MakeMovableItemUnMovable has an empty body. It does nothing.

**Attach effect (High).** AttachEffectToMovableBlock(world, block, effect, a, b, c) attaches placed effect "effect" to block "block" at an offset. The block index is checked against the movable-block count and the effect index against the world's placed-effect count; an index out of range does nothing. The offset vector is (b, c, a, 0): the three floats are taken in the order second, third, first.

**Hero state query.** IsHeroMovingBlock is named. Its meaning belongs to the hero spec (SPEC-0036).

**Solid for characters (High for the geometry; Medium-High for the cue and latch details).** A block is a solid oriented box that moves with it. Its shape is the block's own box record (the same record as in the entry), with its stored extents. Because the record's axis plane constants were written at the last placement (Placement, step 4) and the contact test adds the block's displacement from home to the reference point, the box sits at the placement corner plus (position minus home): it follows the block exactly, with its base at the placement ground height. The box is not re-probed while the block moves.
Each tick, for every motion record whose owner calls the block contact pass (heroes and region characters; SPEC-0040 Behavior 14A), unless the owner is in state 18 or 19 (the hero's own push and grab states), the pass:
1. Lists the active blocks (active word non-zero) whose centre is within 10 units horizontally of the character (squared horizontal distance below 100), at most 100 of them, and sorts them nearest first. In the same scan it keeps, as the character's current block, the nearest active block whose top (position height + H) is more than 0.1 above the character's feet, with its horizontal distance (SPEC-0018).
2. Runs the contact for each listed block in that order. The contact is skipped for an owner in state 30, 41 or 58.
3. Contact. The character's reference point (its position) with radius 0.75 is tested against the box (the point-plus-radius box test of SPEC-0018: each horizontal coordinate in the box frame must lie strictly between -0.75 and the extent plus 0.75, and the vertical coordinate between -0.75 and the vertical extent minus 0.1). On a hit with a side normal (the vertical part of the contact normal below 0.1 in magnitude) the block-slide cue plays at the block and, when the block top is more than 1.0 and less than 5.0 above the character's position, the owner's side-touch method runs (for a hero it sets the side-touch latch of SPEC-0033 Behavior 15A). The shared contact response then moves the character out along the normal by the depth, removes the velocity and force components into the face, and sets the floor flag for a normal with a vertical part above 0.9.
4. If the first test misses, a second test uses the character's predicted position: position + (velocity + force / weight x k) x k, with k = 1/30, plus half the tick's displacement when that displacement is longer than 0.1. The bounds are those of the first test: the vertical bound is not raised by the radius in either test. On a hit the cue and the latch rule above run; a hit on a top face (normal vertical part above 0.1) runs only the velocity, force and floor part of the response, with no position push.
The face of a hit is the nearest of six: the distance to the top bound is (extent − 0.1) minus the vertical coordinate v, to the bottom v + 0.75, and to each horizontal face the coordinate or the extent minus the coordinate. Ties go to the earlier of the order top, bottom, low A, high A, high B, low B. When the point is within 0.375 outside the footprint on all four sides (every coordinate above −0.375 and every extent minus coordinate above −0.375) and the top distance is below 0.25, the face is the top whatever is nearer. The push depth is the distance plus the radius 0.75 for the four horizontal faces; for the bottom face it is v + 1.125 (so the point ends 1.125 below the box base); for the top face it is the distance alone, with no radius.
So a block stops a character on every side and a character may stand on top. The pass is skipped for the hero in states 18 and 19, so the block never pushes the hero who holds it. He is carried by his own state instead: every tick of the grab phase sets his position to the grab spot, 1.5 units from the face, computed from the block's position of that tick, so he moves with the block through a push, a pull (backwards) and a side move (SPEC-0033, Behavior 4A.2; High).

**Standing on a block and the floor flag (High).** A top-face hit moves the character straight up to v = extent − 0.1 exactly, so the resting point is 0.1 below the box's top. On the next tick that point lies on the strict bound, so the first test misses. The second test decides: the gravity step of the same record tick has already put the weight-scaled gravity into the force (it runs before the driver; SPEC-0040, Behavior 14A, items 1 and 5), the previous response left the velocity at zero, and the predicted point is therefore lower by (32 G / 900) = 0.0356 G (G the gravity gain, 1.0 when standing; the weight cancels), inside the bound. It hits the top face, and its response runs with no position push, sets the floor flag (+0x150) and the touched flag (+0x238), removes the force into the face and, as the vertical velocity is below 0.1, sets velocity and force to zero. The floor flag is therefore set on every tick for a character at rest on a block, by the second test, as long as gravity acts on its record (the no-gravity flag clear). The driver clears both flags at the start of each tick, and the gravity step reads the raw ground gap, not the flag, so nothing alternates. A build that rests the point a little inside the bound (0.01) gets the same flag from the first test; only the resting height differs from the original (extent − 0.1 above the box base). A character that has just landed from a fall is handled by the first test on the landing tick (depth to the bound, velocity into the face removed).

**Region characters (High for the place and the response).** The pass is the last step of the region-character branch of the motion-record driver, not a pass after all characters have moved. Within one record tick the order is: ground probe, root travel, the pre-step (edge guard, deadly ground, ground constraint), gravity, the driver (owner pre-step, navmesh confinement, character push, cell list B contact walk, block pass), then the integrator. The pass therefore runs after the confinement and before the integrator, and it sees the position after this tick's root step. A character pushed off the mesh by a block is put back by the next tick's confinement. The response is the shared one that heroes use (SPEC-0018, Slide response): it moves the character out along the normal by the depth (pushes: first test side and bottom faces; the top face with no push in the second test), removes the velocity component into the face, removes the force component into the face, and for a floor normal sets the floor flag and zeroes velocity and force when the vertical velocity is below 0.1. So a region character's velocity into the face is removed, not only its position moved. The root-travel displacement of the tick is part of the test (the predicted point adds half of it when it is longer than 0.1, and the early exit counts it in the velocity into the face). The pass is skipped when the owner's base state word is 18 or 19, and the contact when it is 58, 41 or 30. For a region character that word is the base action state; the common enemies' agro state 19 is a class state in another word (SPEC-0038), so those enemies are not skipped (Medium).

**Block boxes and the static volume lists (High).** A block's box record is a rectangle record that lies outside both rectangle lists of its pack, and no grid cell lists it: on all 90 disc blocks the record's offset appears in no cell's list A (SPEC-0018). The characters' static contact passes walk those cell lists, so they never test a block box, whatever its flags; the box takes part only through the block contact pass above, which reads no flag of the box. Flag bit 1 (0x0002) makes the record a block: 89 of the 90 boxes carry exactly 0x0002, and the box of pack 1298's block carries 0x0003. Bit 0 (solid) is read only by the world projectile step, which tests the box of every active block (the box as last written: once a block contact pass has visited the block it follows the block, before that it stands at its load corner). So the extra bit makes that one block stop projectiles; it does not make characters stop at the block's load position after it has moved (Medium-High for the projectile effect).

**Trigger volumes (High for the dispatch and the timing; Medium for the shapes).** A moving block fires script trigger volumes. The test runs inside the block's own tick, only while the block is moving and on the tick on which its moving flag clears (n + 1 ticks per push, counting the final tick at the end position). Each moving tick starts by clearing the block's last-fired word (+0x60) and every hit sets it again. While the block is idle and that word is non-zero, the block fires the stored volume again on every tick, so a block that comes to rest inside a volume keeps firing the volume it last hit until the next push starts. The volumes tested are the volume records of the selected hero's cell list A (not the block's own cell). The block's shape is a vertical finite cylinder with its base at the block position, radius S and height H (the entry's +0x10 and +0x14). By volume kind:
- kind 1 (sphere): needs flag bit 3 (0x0008); finite cylinder against sphere (centre at record +0x30, radius at +0x3C).
- kind 2: no action.
- kind 3 (cylinder): needs flag bit 3; finite cylinder against finite cylinder (base +0x30, axis +0x40, length +0x24, radius +0x3C).
- kind 4 with the switch value (+0x2C) zero or below: needs flag bit 8 (0x0100); the cylinder against the static rectangle test.
- kind 4 with the switch value above zero: needs flag bit 5 (0x0020); the five-face moving-rectangle test described after this list, with the block cylinder's radius set to 0.6 S.
- kind 7 and every other kind: no action.
A hit fires the volume with no actor, object 0 and the block's number as the last argument (+0x28 of the collision block; SPEC-0030 Layout section 3), provided the block's world pointer is set, and stores the volume in the last-fired word. A block can fire several volumes in one tick (each hit fires at once); the last one fired is the one repeated while idle.

**The block's moving-rectangle test (High for the faces and the ranges; Medium for the cylinder boundary at equality only).** The tested point is the block's position (the middle of its cylinder's base: the footprint centre at the ground height that the placement found, plus the displacement). The cylinder height is H (the entry's +0x14) and the radius is r = 0.6 S (S the entry's +0x10). The volume record gives a corner O (+0x30) and three unit axes with their extents: A (+0x40, extent eA at +0x2C), B (+0x50, extent eB at +0x28) and U (+0x60, extent eU at +0x24, vertical on the disc). It describes a box that runs from O by eA along A, eB along B and eU along U. Five thin rectangles are built from this box, one per face except the top: the two faces perpendicular to A (a = eA and a = 0), the two perpendicular to B (b = eB and b = 0) and the bottom (u = 0). A hit on any one of the five counts. With a, b and u the point's coordinates (P - O) along A, B and U:
- A side face (its normal is horizontal: the vertical part of the normal is not above 0.9) is hit when the point is within r of the face's plane (|a - eA| <= r, |a| <= r, |b - eB| <= r or |b| <= r), the coordinate along the face's own horizontal in-plane axis lies from -r to that axis' extent + r (b from -r to eB + r for the two A faces; a from -r to eA + r for the two B faces), and u lies from -H to eU. The upper bound eU has no r added.
- The bottom face (its normal points down) is hit when u > -H (strict: the block's top reaches the plane u = 0; there is no upper limit), b lies from -r to eB + r and a lies from -r to eA + r.
All range tests are inclusive except u > -H. The top face is never built, so a block that is above the box still fires it as long as u > -H. The bottom face covers the whole footprint widened by r, and every side-face region lies inside that footprint, so on the disc the net rule is: the volume fires when the block's position is inside the rectangle from O widened by r on every side (a from -r to eA + r, b from -r to eB + r) and u > -H; the four side faces add nothing the bottom face does not cover.

**Worked example (disc, pack 1324, plate with trigger 2 and bell block 0).** The plate record: O = (-20.221, 31.162, 1.266), A = (0, 0, 1) with eA = 0.340, B = (1, 0, 0) with eB = 5.553, U = (0, 1, 0) with eU = 10. The bell: S = 3.305 (L = 1.652), H = 9.2, so r = 0.6 x 3.305 = 1.983; its position starts at x = -17.429, y = 31.191 (the ground the placement found, Placement), z = 13.349 and it is pushed along -Z. The volume fires while x lies from -22.204 to -12.686, z from -0.717 to 3.590 and y > 31.162 - 9.2 = 21.962. After five pushes z = 5.087, so nothing fires; during the sixth push (38 steps of 0.0435 each, Tor's push) the position reaches z = 3.565 on step 35, where the far face (a = eA, distance 1.958 <= r) and the bottom face are hit, so the volume fires (the near face is 2.298 from its plane and is not; on step 34, z = 3.608, nothing is hit). With the ground height 0.0 instead of 31.191, u = -31.162 is below -H and nothing could ever fire.


### Push limits on disc

Confidence: High for the rule and for packs 909 and 1324; Medium-High for the other counts (they assume the load position, the drawn objects named below, and no script move); Medium for which crossed frame records are walls the player sees.

The rule of Push start, step 2, applied to the disc data. A block's reachable positions form a lattice of steps L along A and B from its placement; every push must keep the front point F inside the footprint patch's outline, and no other block may be in the way.

**Pack 909 (Ellis Family Farm), its single block (High).** S = 2.7587, H = 2.7409, L = 1.3793. Box corner (-1940.267, 44.075, 1074.289); the placement probe (ceiling 47.075) chooses the pack's model-set entry 132 at height 45.460, so the block stands at (-1938.887, 45.460, 1075.668). Entry 132 is a flat square tile whose outline is four vertical panels: x from -1941.257 to -1928.093 and z from 1071.342 to 1084.512 (the floor round it is entry 131 at 45.83 and entry 127 at 45.38, both within 0.75 of the tile). From the start, pushing straight:
- +A (+Z, north): 5 pushes (6.90 units), to z 1082.565; the 6th is refused (F at z 1085.423 is past the edge 1084.512).
- +B (+X, east): 6 pushes (8.28 units), to x -1930.611; the 7th is refused (F at x -1927.753, edge -1928.093).
- -A (-Z, south): 2 pushes (2.76 units), to z 1072.909; the 3rd is refused (F at z 1070.051, edge 1071.342).
- -B (-X, west): no push at all: the block starts 0.99 from the west edge, and F would be at x -1941.746 (edge -1941.257).
Pulls give the same counts (the hero point lies off the tile, but the floor heights differ by at most 0.38). Because the outline is a rectangle the two axes are independent: the block can reach 7 x 8 = 56 positions, x from -1938.887 to -1930.611 and z from 1072.909 to 1082.565, and it never leaves the tile. No other wall lies inside the tile. The subsection Farm crate (pack 909), at the end of this section, gives the outline polygon, the limits for the crate's centre, the full lattice, the pull rules, the walls around the tile and what the crate is for.

**Every pack with movable blocks (from the load position; pushes +A, +B, -A, -B straight from the start).**

| Pack | Block | Start (x, y, z) | Patch | Pushes +A, +B, -A, -B |
| --- | --- | --- | --- | --- |
| 758 | 0 | (1978.79, 90.62, 97.50) | entry 107 | 7, 13, 3, 4 |
| 784 | 0 to 3 | (-10.01, -0.52, -15.78), (-10.01, -0.52, -22.24), (-3.55, -0.52, -15.78), (2.91, -0.52, -28.71) | entry 8 | 0,0,0,0; 0,6,2,0; 0,4,4,0; 4,2,0,4 |
| 909 | 0 | (-1938.89, 45.46, 1075.67) | entry 132 | 5, 6, 2, 0 |
| 947 | 0 | (-27.59, -7.16, -87.18) | entry 0 | 1, 2, 4, 5 |
| 986 | 0 | (1050.82, 207.78, 258.82) | entry 91 | 2, 3, 4, 3 |
| 1056 | 0 | (0.00, 3.74, 0.00) | entry 52 (fallback) | 0, 0, 0, 0 |
| 1078 | 0 to 2 | (-526.83, 0.93, -433.20), (-276.49, 0.93, -644.14), (-397.27, 1.03, -668.08) | entries 1, 2, 3 | 14,5,8,31; 14,32,10,7; 11,26,20,21 |
| 1090 | 0, 1 | (-233.05, -88.49, -581.34), (-293.04, 129.83, -630.64) | entries 131, 26 | 0,1,8,2; 6,0,1,10 |
| 1176 | 0 | (16.44, 27.47, 535.34) | entry 7 | 28, 40, 15, 7 (its A and B are -Z and -X) |
| 1247 | 0 to 3 | (16.46, -0.99, 52.61), (-137.53, 1.22, 83.27), (114.78, 1.15, -84.57), (137.92, 8.88, 1.80) | entries 20, 8, 7, 6 | 4,17,3,26; 10,29,16,5; 13,23,14,16; 96,6,49,5 |
| 1249 | 0 | (209.87, -20.57, 14.38) | entry 53 | 9, 9, 9, 10 |
| 1298 | 0 | (-47.53, -0.68, 179.04) | entry 1 | 82, 10, 43, 19 |
| 1324 | 0 to 4 | bells (0, 0.25, 0) before the thread moves them; (51.02, 0.25, -45.04), (43.46, 0.25, 15.63) | entry 17; entry 18 | bells 0 (stacked); 3,0,0,1; 4,0,0,0 |
| 1381 | 0 to 56 | lower room blocks 0 to 5 (entry 15, inactive at first); upper grid blocks 6 to 56 (entry 2, y 34.98, 8.988 apart) | | 15 of the 57 cannot move at load (13 firmly; blocks 10 and 56 sit on an exact clearance boundary, see Reachable positions and shortest routes); at most 18 pushes in one direction |
| 1394 | 0 to 2 | (-90.75, -2.83, -1220.52), (-229.70, -3.90, -1342.74), (-229.70, -3.90, -1348.19) | entries 6, 7, 7 | 0,2,0,5; 2,0,0,0; 0,8,6,3 |
| 1397 | 0 to 3 | (98.60, 16.00, 33.36), (387.16, -50.11, -115.75), (406.97, -50.11, -96.35), (367.23, -50.11, -96.06) | block 0: the patch of object 6; blocks 1 to 3: the pack's own floor entry at load (objects 32 to 34 are not drawn until a plate fires) | 4,21,6,1; 14,11,12,11; 6,3,20,14; 6,14,20,3 |

Over the 360 straight runs, 163 end at the outline and 197 at another block (mostly the packed grid of pack 1381); none is unlimited. Pack 1056's block stands on a fallback patch whose outline is far from it (z 304 to 352), so it cannot be pushed in any direction from its load position. Positions are those at load; a script that moves a block to a locator or re-places it gives it a new patch (pack 1324 below).

**Pack 1324, the bells (High).** After the thread moves bells 0, 1 and 2 to locators 6, 7 and 8 (with placed object 21 shown and objects 22 to 28 hidden, as Init leaves them), each bell stands on object 21's inline patch, whose outline has three alcoves that reach the stage-1 plates. Pushed along -Z the bells take exactly 6, 4 and 7 pushes, the pushes that bring them onto their plates (Trigger volumes, worked example); one more push is refused by the end of the alcove. If the probe chose another object's patch (for example object 23's, whose alcoves end at z 5.06), bell 0 would stop after 3 pushes and its plate could never fire, so the drawn-object gate of the probe matters here.

**Walls a block can cross.** Only the block's own outline is tested, so frame records of other entries that lie inside it do not stop the block. On disc these are mostly the outlines of neighbouring floor patches and of pads carried by placed objects (packs 1078, 1247, 1381 lower room, 1397), which the blocks are meant to reach. Pure wall entries (entries with no height patch) that a straight run overlaps: pack 1090 block 1 along -B passes about 1.7 units through a wall panel near x -327 (frame type 3, height 109.1 to 148.1) on its 10th push; pack 1397 block 0 along +A and +B meets the type-3 panels of entry 9; the long side panels of pack 1381's upper room graze the edge of the grid (several already touch blocks at load). Character contact still treats these walls and the block separately (Solid for characters).

**Inactive blocks as stops.** The second clearance test reads no active word, so an inactive block still stops a push: pack 1381's blocks 0 to 5 while inactive, and pack 1324's five blocks in its solved layout (all made inactive by Init). No block on disc exists only as a hidden stop.

#### Reachable positions and shortest routes (all packs with blocks)

Confidence: High for the rules applied and for the counts of packs 909, 986 and 1078 and of pack 1324's bells and stage-2 blocks; Medium-High for the hero grab spots (floor, walls and solid shapes are tested at the spot only, not along the walk to it); Medium for pack 1397 (which floor objects are drawn is set by scripts that this spec does not follow) and for the clearance boundary cases of pack 1381.

**How the routes were searched.** The state is every block's position, footprint patch, active and frozen flags, and the pack's script state. One move is a push, a pull or a side move, given by the direction D and the held face f (Push start). A move is accepted by the footprint step (front point and hero point, with the hero on the grab spot of f) and by the second clearance test against every other block, moving or not. After an accepted move the moving block's volumes are tested on every tick of the move with the hero carried on the grab spot (Trigger volumes), and the pack's script effects are applied. The cost is the number of moves; the number of grabs (changes of the held face) is the second criterion. The searches count Tor's tick numbers; Roberta's differ only in the tick count and never in which positions are reached.

**Hero grab spot rule.** At the start of a grab the hero walks to the grab spot, which is the block position minus the face direction times (0.5 S + 1.5), and must come within the latch distance 0.25 of it. The spot therefore counts as free when: the floor under it is within 3.0 of the block's base; no wall panel that rises at least 1.5 above that floor and reaches the hero's head (3.5) lies closer than 1.175 - 0.25 = 0.925 to it (the hero is held 1.175 from a panel); no solid shape lies within that distance; the spot is outside every other block's box widened by 0.75; and the face finder (Behavior 15A of SPEC-0033, selector 1) passes. Once the grab has started nothing is tested again: the hero is placed from the block each tick, so he may be carried past walls and edges. A route that keeps the same held face needs the spot only once. Where a push needs a spot that fails this rule, the same move is usually still possible as a pull from the opposite face, or as a side move from a neighbouring face.

**Positions a block can reach alone** (the other blocks at their load positions; a position is a centre on the lattice of steps L from the placement, within the footprint patch outline; in the packs searched for traps (909, 986, 1078, 1324, the lower room of 1381 and 1397) every reachable position has a way back to the start, so no state traps a block; some single moves have no reverse move, for example three in pack 986, and a different path leads back):

| Pack | Block | Reachable positions | World x range | World z range |
| --- | --- | --- | --- | --- |
| 758 | 0 | 198 | 1972.33 to 1999.77 | 92.66 to 108.80 |
| 784 | 0 to 3 | 1, 21, 21, 23 | -10.01 to 9.38 | -28.71 to -15.78 |
| 909 | 0 | 56 (7 x 8) | -1938.89 to -1930.61 | 1072.91 to 1082.56 |
| 947 | 0 | 48 | -41.22 to -22.14 | -98.08 to -84.45 |
| 986 | 0 | 42 | 1044.85 to 1058.79 | 250.85 to 264.79 |
| 1056 | 0 | 1 | | |
| 1078 | 0, 1, 2 | 851, 1000, 1536 | -558.64 to -243.67 over the three | -688.60 to -418.83 over the three |
| 1090 | 0, 1 | 73, 66 | -328.68 to -209.81 | -633.88 to -581.34 |
| 1176 | 0 | 2402 | -62.96 to 35.80 | 467.57 to 564.39 |
| 1247 | 0 to 3 | 719, 753, 995, 2675 | -143.78 to 172.92 over the four | -102.07 to 121.80 over the four |
| 1249 | 0 | 380 | 194.25 to 223.93 | 0.32 to 28.45 |
| 1298 | 0 | 3780 | -123.34 to -7.63 | 7.47 to 506.22 |
| 1324 | 0, 1, 2 (bells, after the thread's moves) | 571, 542, 534 | -48.83 to 7.55 | 3.43 to 48.20 |
| 1324 | 3, 4 (stage 2) | 14, 15 | 43.46 to 52.68 | -45.04 to 34.07 |
| 1381 | 0 to 5 (lower room) | 1507, 1516, 1516, 1506, 1491, 1491 | -65.59 to 65.57 | -56.78 to 56.82 |
| 1381 | 6 to 56 (upper grid) | 1 to 35 each; 13 have one position | | |
| 1394 | 0, 1, 2 | 72, 13, 84 | -237.88 to -85.30 | -1364.55 to -1206.89 |
| 1397 | 0 | 573 | 41.98 to 150.29 | -45.41 to 45.67 |
| 1397 | 1, 2, 3 | 974 each | 359.84 to 414.35 | -207.13 to -81.28 |

In pack 1381, 15 of the 57 blocks cannot make any move at load, for any held face (the count of blocks with no straight push is the same). For 13 of them the refusal does not depend on rounding. Blocks 10 and 56 stand exactly one clearance window from their neighbour (the grid spacing 8.988 equals the block size plus 0.5, so the forward test compares two equal values); the original's single-precision rounding decides them and a double-precision model cannot (Medium). A rewrite should compute positions and the test in single precision, in the order of Second clearance test.

**Pack 909, the Farm crate to its ladder spot (High).** Target: the centre four lattice steps east and two north of the start, (-1933.3699, 1078.4266) (the crate under the lower ladder, Farm crate below). The shortest route is 6 moves. The hero holds the east face of the crate (a pull toward the hero is toward the east, so the hero stands east of it at the start): two side moves north, then four pulls east, with one grab. With push and pull only it is 6 moves with 2 grabs: two pushes north from the south face (spot free at the start column), then four pulls east from the east face. Pushing east from the west face is not available at the start column: that spot is 0.83 from the west wall. With pushes alone the route is 8 moves with 3 grabs (one push south, four pushes east along the row z 1074.29, three pushes north in column x -1933.37). The grab spots that fail the rule are the west spot at column 0 for rows 0 to 5, the south spot at rows -2 and -1 for columns 1 to 3 (the post), and the north spot at row 5 for columns 0 to 3; the others in Farm crate, Walls, posts and the house near the crate are borderline and pass the rule above.

**Pack 986, the stair block (High).** Target: three steps toward +x, the centre ending at x 1056.80 (1056.798 before rounding; top at y 210.61), z unchanged. The route is three pushes toward +x with the hero west of the block at x 1047.33, z 258.82; one grab. The spot has floor within 0.01 of the block base and no wall or solid shape near it. The pushes available from the start are +z 2, +x 3, -z 4, -x 3, as in the table; the lattice is 42 positions.

**Pack 1078, plate 7 (High).** The plate record's first axis is -Z with extent 7.0 and its second axis is -X with extent 7.0, from the corner (-536.23, -4.02, -417.21), so the plate covers x -543.23 to -536.23 and z -424.21 to -417.21; the crate fires it while its position lies in the rectangle widened by 0.6 S = 1.231 (x -544.46 to -535.00, z -425.44 to -415.98) and the hero's grid cell lists the plate. The shortest route for block 0 from (-526.83, -433.20) is 8 steps toward -X and 8 toward +Z (16 moves, exactly the minimum): after 8 west steps the centre is at x -535.04 (0.04 inside the widened edge) and after 8 north steps at z -424.99 (0.45 inside). With 10 and 10 the crate rests at (-537.09, -422.94), well inside. Two ways to hold it: pushes west from the east face, then walk to the south face and push north (16 moves, 2 grabs); or hold the south face throughout, taking the 8 west steps as side moves and the 8 north steps as pushes (16 moves, 1 grab). Do the west steps first: pushing north first brings the crate into plate 6's widened rectangle (x -531.9 to -522.4, z -425.4 to -416.0) and opens that record's door, which is harmless but not wanted. The other two crates reach only other plates: block 1 reaches plate 5 in 3 pushes toward -Z; block 2 reaches plate 3 in 14 pushes toward -X and plate 4 in 18 toward +X; block 0 reaches plate 6 in 8 moves, plate 7 in 16 and plate 8 in 31. All start spots are free.

**Pack 1324, the bells and the stage-2 blocks (High for the counts; Medium-High for the hero spots).**
- Each bell stands on the inline patch of the shown floor object (21 before any plate is pressed). When a stage-1 plate is pressed the pressed bell is frozen, the plate object is swapped, the floor object for the new pressed set is shown (26, 27 or 28 for one plate; 23, 24 or 25 for two; 22 for all three) and every other block is re-placed on the floor that is then drawn. Re-placed at their start positions, the two other bells keep their own straight counts under every pressed set (bell 0: 6, bell 1: 4, bell 2: 7 pushes toward -Z), so the swap never shortens or blocks a remaining bell. The pressed bell's own count under its own plate's floor is shorter (3, 2 and 5), which does not matter because it is frozen.
- Search over all three bells with the script effects, all interleavings and all six press orders: the shortest stage 1 is 17 moves, six pushes of bell 0 toward -Z (plate 2), four of bell 1 (plate 3) and seven of bell 2 (plate 4), in any order and with the hero north of the bell (spot on the platform, free). A seventh push of bell 0, a fifth of bell 1 and an eighth of bell 2 are refused by the outline, so each bell can make exactly the pushes that bring it onto its plate and the plate fires on the last accepted push. The margins at the last push are 0.156 (bell 0), 0.0019 (bell 1) and 0.066 (bell 2) in z against the widened plate edge (z 3.5898); the bell 1 margin is small enough that a rewrite must use the exact step and plate numbers, not rounded ones.
- A bell can also reach another plate: bell 1 reaches plate 4 in 10 moves and plate 2 in 10, bell 0 reaches plate 4 in 21; none of these is shorter than the direct pushes.
- Stage 2 (no pressed-set effect on these blocks): block 3 from (51.02, -45.04) fires plate 1 after 2 moves toward +Z, z -40.43 and -35.82 (the plate's widened rectangle starts at z -36.58). The hero must be north of the block and pull: the spot south of the block (z -51.15) lies 0.89 from a tall wall panel at z -52.04, so a push is not possible there. Block 4 from (43.46, 15.63) fires plate 0 after 5 moves: three steps toward +Z (z 29.46) and then two toward +X (x 52.68), with the hero west of the block at x 37.35 (three side moves then two pushes: 1 grab; as pulls from the north face then pushes: 2 grabs). The first east step is refused at the start row and at rows z 20.24 and 24.85 in the start column, because the front point leaves the outline; it is accepted only from z 29.46 up. The spots south and east of block 4 lie in a solid shape and are not usable.
- Shortest total for the pack: 2 + 5 + 17 = 24 moves. With the two stage-2 marks set before the third bell, the third bell's press runs the solved scene.

**Pack 1397, blocks 1 to 3 and the three plates (High for the rules; Medium for the floor objects).** The plates (triggers 9, 10 and 11) are boxes whose horizontal sides are 1.0 to 1.6 and whose height is 3.1, at (366.6, -51.6, -258.4), (406.6, -51.6, -197.8) and (366.3, -51.6, -106.1); a block fires one while its position is within 0.6 S = 2.954 of the box footprint, and the hero's grid cell lists it. Each plate acts only for one block: plate 11 for block 3, plate 10 for block 1 and plate 9 for block 2 (the plate handler compares the firing block's number; another block does nothing and leaves the plate armed; SPEC-0042, handler G). Placed objects 32, 33 and 34 are never drawn by Init, so at load they are not ground: blocks 1 to 3 stand at y -50.11 on the pack's own floor entry, whose outline spans x 339 to 435 and z -149.46 to -70.11 (the first dip begins at z -148). The plate scenes swap in the invisible block-floor patches: plate 11 draws placed object 33 (outline, a 44-edge chicane, to z -210.1, height -50.06) and re-places the three blocks on it; plate 10 hides it, draws placed object 34 (outline to z -270.75) and re-places the blocks again. Plate 11 also raises lift 29 and plate 10 lift 30, whose columns the floor patches cover. So the three plates form a chain, in the order 11, 10, 9:
- Plate 11 (z -106.1): block 3 in 3 pushes toward -Z (z -103.44), hero north of the block; block 1 in 10 moves (7 steps toward -X as side moves, 3 toward +Z); block 2 in 18.
- Plate 10 (z -197.8), after plate 11 has fired (before it, on the pack's own floor, no block gets past z -145.6, short of the first dip): block 1 in 47 moves (4 toward -X, 32 toward -Z, 11 toward +X; the outline is a chicane that forces the eastward leg to come last), with one grab if the 32 steps are side moves and three if only pushes and pulls are used. Block 2 would take 63 and block 3 55, but the plate ignores them. Block 3 parked on plate 11 does not change the count. The farthest any block gets on object 33's floor is z -207.1. Resolved (Medium).
- Plate 9 (z -258.4), after plate 10 has fired: block 2 in 82 moves (12 toward -X as side moves, 45 toward -Z as pushes, 1 toward +X, 15 toward -Z, 4 toward -X, 5 toward -Z, ending at (370.04, -256.37)), one grab if the sideways steps are side moves. Block 1 would take 39 and block 3 73, but the plate ignores them; with blocks 1 and 3 parked on plates 10 and 11 the count is unchanged. The farthest any block gets on object 34's floor is z -266.2. Resolved (Medium): plate 9 is reachable, and it is the plate that raises lift 31, the third dip lift that the walk to the Treasury box needs. The earlier statement that no block reaches plate 9 (farthest z -207.1) held only for the floor of object 33.
- Resolved (High): the plate handler raises the dip lifts (SPEC-0042, handler G): plate 11 lift 29, plate 10 lift 30, plate 9 lift 31, each rising 10.476 over about 6.3 to 6.7 s from about -60.6 to about -50.1. The walk to the box uses all three (SPEC-0058, goal 534). The hero's walk to each grab spot is tested at the spot only (as for the other packs), so the counts assume the hero can get behind the block, which the lifts and floor patches allow once the previous plate has fired.
- Block 0 (98.60, 16.00, 33.36) stands on the inline patch of object 6, the object that trigger 0's solve sequence later removes, so it is on y 16.0 (the patch height; setup makes an object's current block equal to its rest block, so no offset is added) and not on the pack's own floor at y 0 (the first update re-places it with a probe ceiling 500 above and gets the same patch). Trigger 0's box (corner (92.27, 16.82, 14.62), 7.82 along +Z, 12.74 along +X, 10 tall) is fired once the block's position is inside z 11.67 to 25.40 and x 89.31 to 107.96: four pushes toward -Z (z 23.52) with the hero north of it at z 29.94 to 37.32, one grab (Medium: it assumes object 6 is drawn at that time).

**Pack 1381, the six stations (optional; High for the counts, Medium for the boundary cases).** For station k the lower-room block k must touch the station's arm point and goal point together (their positions and radius 0.1 are in SPEC-0042, 1381) while the hero's grid cell lists both. The shortest routes from the start centres, with the hero holding the face named, are: block 0, 7 pushes toward -Z (the SPEC-0042 test vector); block 1, 4 steps toward -Z then 6 toward +X (10 moves, side moves for the first four, so one grab); block 2, 3 toward +Z then 6 toward +X (9); block 3, 7 pushes toward +Z; block 4, 6 toward -X then 3 toward +Z (9); block 5, 6 toward -X then 3 toward -Z (9). All the spots needed are free. A station block must first be active, which the station's target hit does.

Default if unresolved (rewrite choice): where a rule here depends on a hero position that the original's wall contact decides within 0.1 of the limit, use the normal hero wall collision on the approach and add no special rule to the blocks.

#### Farm crate (pack 909)

Confidence: High for the outline, the front-point rule, the centre limits, the lattice, the step length, the hero-point result and the grab-offer result; Medium-High for Tor's tick counts; Medium for Roberta's tick counts, for the walls that limit where the hero can stand, and for what the crate is for.

The Farm's only movable block, called the crate here. Its size is S = 2.7587, its height H = 2.7409 and its step length L = 0.5 S = 1.3793 (1.3793377 before rounding). Start centre (-1938.8873, 45.4599, 1075.6679); its top is at y = 48.2008. All coordinates below are world x and z.

**Boundary polygon (High).** The crate stands on the floor tile of model-set entry 132 (entry 144 is an identical twin; the placement probe takes 132 because the cell lists it first). The tile is flat at y = 45.4599. Its outline is four type-0 frame records, in record order. Each is a vertical panel whose top edge lies on the tile height and which hangs 120 below it, so a hero standing on the tile never touches it; only the push start reads these panels. The normals point out of the tile.

| Record | Starts at | Runs to | Run length | Plane normal | Span scalar |
| --- | --- | --- | --- | --- | --- |
| 0 (north edge) | (-1941.2573, 1084.5123) | (-1928.0925, 1084.5123) | 13.1648 | (0, 0, 1) | +0.00012 |
| 1 (east edge) | (-1928.0925, 1084.5125) | (-1928.0925, 1071.3423) | 13.1702 | (1, 0, 0) | -13.1702 |
| 2 (south edge) | (-1928.0925, 1071.3423) | (-1941.2573, 1071.3423) | 13.1648 | (0, 0, -1) | +0.00012 |
| 3 (west edge) | (-1941.2573, 1071.3424) | (-1941.2573, 1084.5123) | 13.1699 | (-1, 0, 0) | +13.1699 |

The list test on this outline (SPEC-0018, Ground probe, step 3) passes exactly the points with -1941.2573 <= x <= -1928.0925 and 1071.3424 < z < 1084.5123. The two x-running panels (records 0 and 2) have a span scalar of only 0.00012, so they count only inside a 0.0001-wide z sliver and the z limits are uncertain by 0.0002; no front point of the crate comes within 0.08 of them. A rewrite can therefore use this rectangle directly in place of the four records.

**Front-point rule and limits for the centre (High).** With D the unit vector of the move, the front point is F = centre + D x (S + 0.1) = centre + D x 2.8587 (this is L + 0.5 S + 0.1). A push, a pull or a side move in direction D is accepted by the footprint step exactly when F is inside the rectangle above; on the Farm nothing else refuses a move (next paragraphs). In terms of the centre:
- north (+Z): accepted only while centre z < 1081.6537;
- east (+X): only while centre x <= -1930.9512;
- south (-Z): only while centre z > 1074.2011;
- west (-X): only while centre x > -1938.3986.
The other coordinate of F equals the centre's and is always inside the tile. The centre therefore stays within x -1938.8873 to -1930.6112 and z 1072.9092 to 1082.5646, and a crate face at its extreme then lies 0.990 (west), 1.139 (east), 0.188 (south) and 0.568 (north) inside the tile edge. The crate never leaves the tile and never crosses an outline panel.

**Hero point and other blocks (High).** H = hero position + D x 1.1 x (L + 0.75). With the hero on the grab spot (Solid for characters) H = centre + D x (-0.5371) for a push, centre + D x 5.2216 for a pull, and grab spot + D x 2.3423 for a side move. When H is off the tile the floor there is 45.83 (entry 131: north, south, east) or 45.38 (entry 127: west), so the height difference to the crate's floor is at most 0.372 and the 0.75 limit never refuses. In 896 combinations of the 56 positions, four directions and four hero faces, the move is accepted exactly when F is inside. The world holds no other block, so the second clearance test has nothing to compare with.

**Step length and timing (High for the step; Medium-High for Tor's counts; Medium for Roberta's).** Every accepted move shifts the centre by exactly L = 1.3793377 along D; the data holds no rounding. The block adds L / n each tick for n ticks in single-precision floats, so the sum lands within 0.003 of L per move and within 0.01 after six moves in one direction; the smallest margin of any decision is 0.0875 (the second south move), so a rewrite may snap the centre to the lattice below after each move. The moving flag clears one tick after the last step (n + 1 ticks per move).

| Move | Tor n | Tor step per tick | Roberta n | Roberta step per tick |
| --- | --- | --- | --- | --- |
| push (action 42) | 38 | 0.036298 | 18 | 0.076630 |
| pull (action 41) | 23 | 0.059971 | 9 | 0.153260 |
| side (actions 144, 145) | 17 | 0.081138 | 7 | 0.197048 |

With the stick held, a new move can start once per clip cycle (SPEC-0033, Behavior 4A.2): Tor 44 ticks for a push, 52 for a pull, 40 for a side move; Roberta 44, 44 and 36.

**Lattice of centre positions (High).** Write x_k for k = 0 to 6 (k = 0 is the start column) and z_j for j = -2 to 5 (j = 0 is the start row):
- x_k = -1938.8873, -1937.5080, -1936.1286, -1934.7493, -1933.3699, -1931.9906, -1930.6112;
- z_j (j = -2 to 5) = 1072.9092, 1074.2886, 1075.6679, 1077.0473, 1078.4266, 1079.8059, 1081.1853, 1082.5646.
Every one of the 7 x 8 = 56 pairs (x_k, z_j) is a reachable centre, and no other position is. The start is (x_0, z_0). From (k, j) the four directions are allowed as follows, for the push and the pull alike (and for the side moves of that direction): north while j < 5, south while j > -2, east while k < 6, west while k > 0. Every refusal has the same reason, the front point lies outside the outline: north from j = 5 gives F z = 1085.4233 (edge 1084.5123); east from k = 6 gives F x = -1927.7526 (edge -1928.0925); south from j = -2 gives F z = 1070.0506 (edge 1071.3424); west from k = 0 gives F x = -1941.7459 (edge -1941.2573). The last accepted fronts are north 1084.0439, east -1929.1319, south 1071.4299 and west -1940.3666. From the start the straight runs are 5 north, 6 east, 2 south and 0 west, for pulls too.

**Pull specifics (High unless marked).**
- Where the hero stands: to pull in direction D he holds the face on the D side of the crate. His grab spot is centre + D x 2.8793 (1.5 from the face), facing the crate; he pulls the stick back and is placed relative to the crate on every tick, so he moves with it (SPEC-0033, Behavior 4A.2). At the start: pull north from (-1938.8873, 1078.5472), pull east from (-1936.0080, 1075.6679), pull south from (-1938.8873, 1072.7886), pull west from (-1941.7666, 1075.6679).
- What decides a pull: the same front-point rule as the push in that direction (a pull is refused exactly where the push of the same direction is) and the hero point (never refuses here). The pull-back clearance point P of the face finder (SPEC-0033, Behavior 15A) is not tested during a pull: state 18 does not run the finder, so it only decides whether the grab and the climb onto the crate are offered at a face.
- P on the Farm: P = centre minus the face direction x (S + 1.5), 4.2587 from the centre on the hero's side. It lies on the tile or off it by at most these margins: face 1 (hero west) at columns k = 0 and 1 (P west of the tile, floor 0.08 lower), face 0 (hero south) at rows j <= -1, face 2 (hero north) at rows j >= 4 and face 3 (hero east) at columns k >= 5 (the floor there is 0.37 higher). The ground difference is never above 0.372, so the finder with selector 1 passes at all 56 positions and all four faces: the grab is always offered.
- Selector 0 (the push-into-face counter that leads to the climb onto the crate) adds a segment test with the restriction flag clear, from the point 0.75 S = 2.0690 from the centre toward the hero to the point S + 3 = 5.7587 from the centre, both raised 1.5. On the Farm only the solid wall panels of the next paragraph can stop it (the tile's own panels end below the segment): it fails from face 1 at columns k = 0 and 1 (west wall) and from face 2 at rows j >= 3 in columns k = 0 to 3 (north wall). From those faces at those positions the climb onto the crate is not offered; grabbing, pushing and pulling still are.

**Walls, posts and the house near the crate (Medium).** None of these enters the push and pull rules, which read the outline only; they limit where the hero can stand and approach. All other frame records near the tile are floor outlines like the tile's own, hanging below the floor, and cannot stop him.
- West wall, a solid wall (type 3, solid from y 43.3 to 55.4) facing east: from (-1941.94, 1085.66) to (-1942.83, 1072.17), continuing to (-1942.94, 1070.54) and (-1943.82, 1057.34). It lies 0.7 to 1.6 west of the tile edge (1.34 at the start row).
- North wall, solid, facing south: from (-1934.48, 1085.16) to (-1941.94, 1085.66), a slab 1.0 thick whose north face (facing north) is at z about 1086.7; a short east-facing jamb from (-1934.37, 1086.71) to (-1934.48, 1085.16); further north the wall continues east-facing at x about -1935.4 to -1934.5 up to z 1100.07.
- A post (a solid vertical cylinder, radius 1.2034, y 45.30 to 57.09) at (-1936.071, 1069.862), just south of the tile edge.
- Two taller posts (radius 1.133) at (-1920.633, 1070.010) and (-1919.649, 1084.845), and the walls of a large solid building outline east of the tile (type 3, x from about -1923 to -1907, z 1053 to 1100, solid from y 43.3 to 58.6); its nearest wall is more than 5 units from the farthest east grab spot, and none of these touches a grab spot. No fence stands near the crate.
- A thin solid box on the ladder (base y 49.19) lies above the hero's head and the crate.
- The hero (radius 1.175) is held that far from a wall and must reach his grab spot (within 0.25) without touching anything: a contact during the approach makes him let go. This makes these grab spots unusable: the west spot (hero west of the crate, for a push east or a pull west) at column k = 0 for rows j >= 0 (wall plane 0.83 to 0.38 away; at rows j = -1 and -2 it is 0.93 and 1.02 away, borderline); the south spot (a push north or a pull south) for rows j = -2 and -1 at columns k = 1 to 3 (within the post's reach); the north spot (a push south or a pull north) at row j = 5 for columns k = 0 to 3 (at or behind the north wall; 1.09 from the jamb at column k = 4) and, borderline, at row j = 4 column k = 3 (1.11 from the north wall). All 56 positions stay reachable by a player with these limits: the east, south and north spots at the start are free, so the first moves east are made as pulls by a hero standing east of the crate. A rewrite that already collides the hero with these walls gets this result without special code. Default if unresolved (rewrite choice): apply the normal hero wall collision to the approach and add no special rule to the crate.

**What the player sees (High for the refusal; Medium for the rest).**
- At a limit the hero keeps playing the push or pull clip in place, cycle after cycle; the crate does not move and the hero stays on his grab spot. There is no message, no change of prompt and no refusal sound; whatever the hero's push or pull start plays (SPEC-0033, Behavior 4A.2) is played the same way, because the push start's answer is not read. The limit is only noticed by the crate not moving. The hero can let go (Square) and take another face.
- Because the grab is offered at every face (above), there is no face where the prompt is hidden at a limit. The crate rests at the lattice position, with a visible gap to the tile edge (0.19 to 1.14) that has no visible mark of its own.
- Climbing onto the crate (top 2.74 above the floor, inside the 0.2 to 3.5 window of SPEC-0033, Behavior 15) is possible from any face except the ones named under selector 0 above.

**What the crate is for (Medium; no script uses it).** The Farm's level script never moves, re-places, activates or deactivates this block; it only registers the movable-block sound index. The crate stands inside the trigger volume of the push/pull and box/ladder hint (SPEC-0045, trigger 7: x from -1941.4535 to -1936.3569, z from 1073.4445 to 1078.2642 widened by the body margin, y 43.85 to 49.70); the hint is started by the hero only. The crate's own trigger pass reaches that volume with no actor, which the hint ignores, so the crate sets nothing off. The crate is a tutorial step to a ladder: a ladder (SPEC-0006, Type 10) leans on the shelf above the tile with its face plane at x about -1933.9 to -1934.5, covering z 1077.45 to 1079.08, base y 50.383 and top y 59.56. The ladder grab needs the hero's feet at y >= 49.383, 1.175 behind to 1.469 in front of the plane, and the floor is 45.46. Corrected (High): standing on the crate is not enough, because its top (48.2008) is 1.18 below that bound (height on the ladder -2.18, below the -1.0 limit), so the ground grab is refused there too. The hero must jump from the crate with the stick pushed toward the ladder; the ladder test also runs in the air (SPEC-0033, Behavior 13A) and passes once his feet rise past 49.383, well inside a standing jump. The worked climb is SPEC-0033, Behavior 13A, item 7. The crate positions that put its top under that grab zone are the rows z = 1078.4266 (centre 2 steps north of the start) in columns x = -1934.7493, -1933.3699 and -1931.9906 (3 to 5 steps east); the ladder leads up to the shelf at y 58.67. There is no target position that any rule checks. Default if unresolved (rewrite choice): no crate scripting; the player may leave it anywhere on the lattice.

**Test vector (centres after each accepted move).** Start (-1938.8873, 1075.6679).
1. Push west: refused, F x = -1941.7459 is west of -1941.2573.
2. East six times (by pushes, or by pulls with the hero on the east side): x = -1937.5080, -1936.1286, -1934.7493, -1933.3699, -1931.9906, -1930.6112 (z unchanged). The seventh is refused: F x = -1927.7526 is east of -1928.0925.
3. Pull west twice: x = -1931.9906, then -1933.3699 (F x -1933.4699, then -1934.8493, both inside).
4. Push north five times: z = 1077.0473, 1078.4266, 1079.8059, 1081.1853, 1082.5646. The sixth is refused: F z = 1085.4233.
5. Push south seven times: z = 1081.1853, 1079.8059, 1078.4266, 1077.0473, 1075.6679, 1074.2886, 1072.9092. The eighth is refused: F z = 1070.0506.
6. Push west four times: x = -1934.7493, -1936.1286, -1937.5080, -1938.8873 (z 1072.9092). The fifth is refused: F x = -1941.7459.
After step 4 the crate stands at (-1933.3699, 1082.5646), its farthest north; after step 5 at (-1933.3699, 1072.9092), its farthest south; after step 6 it is back in the start column.

### Plate manager

Confidence: Medium for the roles (plates, sinking object) and the completion step; High for the per-tick rules and constants below, which are fixed program constants (no disc data word). These rules are not world items: world items (SPEC-0021) have their own pool, hover, flight and pickup rules.

Never populated (High): the manager's add routine has no caller, and no host call or data pointer reaches it (SPEC-0028, Built-in object pools that are never filled), so every slot keeps its unused mark and nothing in this section runs in play. Pressure plates, switches and levers seen in play are scripted trigger volumes (SPEC-0028, section 13; SPEC-0030). The rules below describe the engine part only; a clean implementation may omit it.

The world tick runs the manager once per tick while the world is active (alongside the movable-block list and the other world subsystems). Steps are per tick and are not scaled by dt. For each slot index k from 0 up to the slot count, skipping slots whose unused-slot word is non-zero:

1. **Slot 0, sinking object.** When its triggered flag is set, its height y goes down by 0.1 per tick (6 units per second at 60 ticks per second). When y is then below -50, the flag is cleared and the object stops where it is (it is not removed or moved back). When the flag is clear, nothing moves. What sets the flag is open.
2. **Slots 1 to 3, plates.** A plate that is already pressed does nothing. Otherwise let P be the plate's position, H the active hero's position and r the plate's radius. The plate is pressed when both hold:
   - H.y - P.y < 1.5 (the hero is less than 1.5 units above the plate; there is no lower bound), and
   - the horizontal distance sqrt((H.x - P.x)^2 + (H.z - P.z)^2) < r + 1.5.
   On the press, the triggered flag is set and the plate's height drops by 0.5 units at once. Nothing in the manager ever clears a plate's flag or raises it again.
3. **Slots 4 and up.** No behavior.
4. **Every slot** then refreshes its drawn transform from its stored orientation and its current position, and its bounding sphere becomes (position, r).
5. **Completion.** After the slot pass, the manager counts the slots 1 to 3 that are in use and pressed. When the count is 3 and bit 0 of the world's 64-bit flag set (the bit set whose pointer is at world +0x2C) is clear, the completion step runs. In this build the completion step has an empty body, so pressing all three plates has no effect through the manager (Medium). It is re-tested every tick while all three stay pressed.

All five values (0.1, -50, 1.5 twice and 0.5) are code constants.

### Doors

Doors are not world interactables of this spec. They are the kind-7 door volumes of SPEC-0028 (section 13, Doors), and their travel rules are in SPEC-0006 (Doors). Gates and moving doors are animated placed objects driven by level scripts (SPEC-0028, section 7.1). The grab-door hold (hero state 29) never occurs on disc data: no kind-7 record builds a grab-door record (SPEC-0028, section 13). High.

## Edge cases and error handling

- **Break on states 0, 2 or 3** does nothing. No state change, pieces, drop or broken bit. A gated entry (state 3) is not broken by Break, by an attack contact or by the group step. An attack contact on a state-3 entry still plays its hit cue (Break routes, step 4).
- **Script hook registered.** An attack contact on an entry whose script index has a registered callback starts the callback and does nothing else: no element test, no break, no cue.
- **Contact material other than 2, 3 or 4** gives cue 144. Material 2 (cue 143) occurs in no entry on disc.
- **Intact query in state 3** is true. A gated object still reports intact, while the invulnerable query is also true for it.
- **SetBreakableActive(true) on a broken entry** sets state 1 and the sub-record's s16 at +0x12 to 9 (or 0x21 when the sub-record's type byte is 4). The broken pieces are not restored, so the entry is breakable again but its pieces are gone (Medium).
- **Group arm revives a broken member.** Arming sets the next member's state to 1 unconditionally. A member that was broken (state 2) can therefore be made breakable again by a chain step (Medium).
- **Chain hit on a guarded stage.** If the first live member found is in state 3 (a stage armed less than 24 ticks ago), the hit does nothing: that member is not broken and no member is armed. A new stage therefore cannot be broken during the 0.4 s after it appears (High).
- **Lock table full.** The third lock is dropped with a log call. The log has no gameplay effect (Medium); a rewrite may omit it. Whether the retail build prints it is not recorded anywhere (class D).
- **Key id 0 or -1.** Counts as no key. The group opens without the lock message and without removing any item.
- **Repeat press after opening.** None. The open clip leaves the chest object animating and then finished, and the gate tests that animation state, so a later press does nothing, in this visit and after a reload (High).
- **Lock not removed on open.** The open sequence does not remove the lock entry, and the lock remove routine has no caller, so the entry stays in the table until the world is rebuilt. This has no effect on the opened chest, whose gate is already closed (High).
- **Hero in the air.** A press is ignored when the hero's height above ground is 0.1 or more. The units of the height value are not confirmed.
- **Flagged placements.** Placements with start flags 2 to 4 are not spawned at region start, but a break can still drop them (Medium).
- **Start mode 0.** The default mode is 0, so a broken entry comes back as a normal breakable after a reload. No disc entry has start byte 1 or 2, so on disc data every broken breakable comes back intact, with its drop (Start mode and reset). The original would read an unset low-memory word for the default if the world's default-mode word were zero; the loader always sets that word; use 0.
- **Pieces on reload.** Pieces are not stored. A broken entry's state after the level is loaded again follows its start mode, because the mode is applied only when its broken bit is set; on disc data that is always state 1 (Start mode and reset).
- **Element mismatch on a seal hit or a hero melee contact.** Tor recoils with material 4 (Element recoil: attack sub-state 13, cue 13, rumble), with no break and no breakable cue; Roberta has no reaction. The same recoil is the only effect of any hit on a chest body (breakable 0 of a lock chest or of an element-sealed chest). The Mote-family contact handlers and Roberta's shot skip silently on a mismatch (Break routes, step 2).
- **Unreachable +0x5A route.** The break route gated on +0x5A (push 0, no cue) is never reached; a clean implementation that breaks entries from attacks follows Break routes and ignores +0x5A as a gate (Medium).
- **Nova at a seal.** The seal hook passes Nova, but the prompt thread shows the mismatch prompt 29 to a hero with Nova when E_t is 1 to 6. Reproduce both rules as stated (High).
- **Entry defaults never apply on disc.** The size 3.5 / height 5.0 defaults are written only for an entry that has no inline mesh and a negative +0x18 word. Every built block has an inline mesh, so L is always 0.5 times the stored size, also for a block that is placed again (move to locator, reacquire). The earlier statement that a re-placed block gets L = 1.75 is wrong for disc data (High).
- **Push while moving.** A push request while the moving flag is set does nothing. The direction-allowed word is never zero, so it never refuses.
- **Idle block in a volume.** A block that finished a push inside a trigger volume fires that volume on every following tick until it is pushed again (Trigger volumes).
- **Pack 1176 block.** Its axes are reversed (A = -Z, B = -X) but the second clearance test still looks towards +Z and +X (Second clearance test).
- **Push tick count.** n is stored as a signed byte, so a count of 128 or more wraps. With n = 0 or negative the block does not move: the first step finds i ≥ n and clears the moving flag (Medium; Open questions).
- **Inactive block.** A block whose active word is zero is not stepped, so a push started on it does not advance until it is activated again.
- **Plate pressed from below (quirk).** The plate height test has no lower bound: a hero any distance below a plate, within r + 1.5 horizontally, presses it. Reproduce (Medium).
- **Plates stay pressed.** A pressed plate is never released or raised by the manager, and its 0.5 drop happens once (Medium).
- **Sinking object below -50.** The object stops just below -50 (the first height under -50 that the 0.1 steps reach) and stays there (Medium).
- **Completion does nothing.** Three pressed plates run an empty completion step; any visible result of the plates comes from elsewhere (a level script or another object), which is open (Medium).
- **Fuse with empty lists.** A kind-3 entry without its 32 objects and 1 breakable (27 entries on disc) does nothing (Proximity fuse).
- **Fuse and a sneaking hero.** A hero who stays in states 1, 2, 22, 39 or 40 inside the 180.0 squared-distance ring never starts the countdown, but coming closer than squared distance 7.0 fires the fuse at once in any state. Once the countdown has started, standing still or sneaking does not stop it; only leaving the ring resets it (Medium).
- **Kinds 0, 7 and 12.** Set-up does nothing for them; an implementation that adds built-in behaviour for these kinds would differ from the original (High).
- **Kind-12 entries of pack 1369.** No callback is registered for type 12 in 1369, so its 27 entries have no effect in play; reproduce that (Medium-High).

## Test vectors

Script groups (disc check values):
- Pack 1397: list kinds 2, 0, 0, 0, 0, 0. Its script fetches entries 1 to 5 by fixed index, all kind 0; set-up gives none of them any behaviour of its own.
- Pack 1369: 27 kind-12 entries and no script-group call: after load, no group callback runs for any of them.
- Pack 1023: its script calls GetScriptGroupMainTrigger, but its type-40 list is empty, so any such call returns 0.
- Pack 1171, load: the kind-12 init runs 34 times (entries 3 to 36). With Random returning 37 for entry 3: object 17 is active, set to 0.37 s and looping at speed 1.0; objects 18 and 19 are untouched. Random returning 100: 1.00 s (clamped to the clip length when that is shorter).
- Pack 1171, hero walks onto trigger 40 (entry 14, k = 11): object 50 is deactivated, object 51 is activated and plays once to its end and holds; after the hero leaves, object 51 is deactivated, object 52 plays once from 0, then object 50 loops from 0.
- Pack 1171, hero enters trigger 29: the kind-12 run starts for entry 3 (objects 17 to 19); the switch handler registered for 29 earlier in the same init does not run.
- Registration order: a kind-t entry with no trigger still gets its init call, and no trigger is registered for it.

Breakables (synthetic entry, state word as given):
- State 1, Break: state becomes 2. Intact query 0. Invulnerable query 1.
- State 2, Break: no change.
- State 1, MakeBreakable(false): state 3. Intact query 1. Break: no change. MakeBreakable(true): state 1.
- State 1, SetBreakableActive(false): state 0. Intact query 0. Invulnerable query 1.
- State 2, SetBreakableActive(true): state 1. Intact query 1. Invulnerable query 0.

Attack contact (synthetic entry, no script hook registered, sub-record present):
- State 1, element 0, material 3, kind-3 family handler: the entry breaks (state 2) with piece push 0.25 along d; cue 145 plays at the entry position; the broken bit for its script index is set.
- State 1, element 5, kind-3 family handler: skipped; no break and no cue.
- State 1, element 5, hero melee (Tor) with a slot-2 Mote of kind 2: no break, no breakable cue; Tor recoils with material 4 (attack sub-state 13, recoil cue 13). The same hit with Roberta as the hero: nothing happens.
- State 3, element 0, material 0, Nova family handler: no break; cue 144 plays.
- State 1, element 0, material 4, Roberta's shot: the entry breaks; cue 144 plays (not 146).
- Entry at position (4, 0, 0), attacker reference point (1, 0, 4): d = (0.6, 0, -0.8).
- Same entries with a callback registered for the script index: the callback starts; no break and no cue in every case.

Lock table (table starts empty, both ids -1):
- LockChest(id 4, key 1234, message 77), then LockChest(id 5, key 9, message 78), then LockChest(id 6, key 0, message 79). The third call stores nothing.
- Lookup id 4: key 1234 and message 77. Lookup id 6: -1 (not found).
- Pack 951: LockChest(group 0, key item 15, message 124). Pressing Square at the chest without item 15 shows line 124 of that world's dialogue table and the chest stays shut; with item 15 the item is removed and the chest opens.
- Piece push: a break with push 0.25 and own axis (0, 1, 0), direction away from the attacker d = (1, 0, 0): the emission axis before spread is normalize(0.75 x (0, 1, 0) + 0.25 x (1, 0, 0)) = (0.316, 0.949, 0).

Chest press (group index 4, key 1234):
- Hero without item 1234: message 77 shows; no opening.
- Hero with item 1234: all of item 1234 is removed from the hero; sound cue 142 plays and the hero plays animation 162; the chest opens. Effect 790 plays only if shared effect group 3 is loaded. The reward placement drops and the chest's collected entry (for example key 12) becomes 1.
- Chest press with key 0 (or -1): no key; the chest opens with no message and no item removed.
- Second press on the opened chest: nothing (the chest object is animating, then finished, and the gate refuses both).
- Key row clear, no lock, Square newly pressed with the hero's height above ground 0.05: the chest opens; at height 0.1: no opening.
- Reload of a world whose chest key row is set: the chest is at its last frame (finished); no prompt shows and a press does nothing.
- Locator 0 at X = (0, 0, 0) with facing d = (0, 0, 1): drop point A = (0, 0.4, 2.4), effect 790 point B = (0, 0, 2.9).
- Lock entry present (key 1234, message 77), key not held, two presses: message 77 on each press; the chest stays closed.

Start mode (synthetic entry, script index 5):
- Broken bit set, start byte 0: state 1, volume flags 9 (0x21 for a type-4 volume). Start byte 30: the same. Start byte 1: state 0. Start byte 2: state 1 with drop id -1.
- Broken bit clear, any start byte: the pack state (1) is kept.

Attach effect:
- AttachEffectToMovableBlock(world, block 2, effect 0, a = 2.0, b = 5.0, c = 1.5): offset (5.0, 1.5, 2.0, 0).

Button query:
- Code 0 with pad word 0x0080 set: true. Code 0 with only 0x0010 set: false.
- Packet byte 2 = 0xFF, byte 3 = 0x7F: held word 0x0080 (Square), so code 0 is true on the held word.
- Square down on pass n (up on pass n-1) and still down on pass n+1: code 0 with the second argument true is true on pass n and false on pass n+1; with the second argument false it is true on both. The chest press fires on pass n only.
- Any code with the player-control word at +0x44 nonzero: false.

Placed drop (key -5, fixed count 37, money roll gives no amount, item roll gives no item):
- Coins spawned in order: 25, 10, 1, 1. (37 = 25 + 10 + 1 + 1.)

Breakable chain (synthetic chain of three members; after set-up the states are 1, 0, 0):
- Hit on member 0: member 0 breaks; member 1 goes to state 1 and at once to state 3, so the states are (2, 3, 0) for 24 ticks and then (2, 1, 0).
- A hit on member 1 at tick 10 after the arm: nothing (it is in state 3). At tick 30: member 1 breaks and member 2 is armed, giving (2, 2, 3) and then (2, 2, 1) 24 ticks later.
- Hit on member 2 (no hook): it breaks through the contact route and plays its contact cue.
- A hit when no member is in state 1 or 3: nothing happens.

Element-sealed chest (synthetic; the seal, breakable 1, has element class 4; breakable 0 has class 0):
- Hero Mote kind 2: the hit fails; Tor recoils with material 4 (cue 13); the seal is not broken.
- Mote kind 4: the hit passes and the open runs (breakable 0 active, the seal broken, cue 182 after 0.3 s and camera 0).
- Mote kind 7: the hit passes.
- No Mote (kind 0): the hit fails.
- Same entry with a seal of element class 0: any hit passes, including kind 0.
- Hook record word +0x24 zero: the hit fails.
- Any hit on breakable 0 (the chest body) before the open: Tor recoils with material 4; nothing else happens.

Prompt thread at a seal of element class 3:
- Mote kind 7: context prompt 29 (mismatch).
- Mote kind 3: context prompt 30, shown each tick until Square is held once.

Proximity fuse (synthetic, 60 ticks per second):
- d2 = 6.9: fires at once, in any hero state.
- d2 = 100, hero running: the countdown starts; T passes 7.0 s after about 421 ticks and U passes 0.3 s about 18 ticks later, so the fuse fires after about 439 ticks.
- d2 = 100, hero sneaking (state 22) from the start: the countdown never starts.
- d2 = 200 after the countdown started: T = 0 and stages 0 to 2 are shown again.
- At T = 3.0 s, floor(5T) is 15, so shake is 0; at T = 3.25 s, floor(5T) is 16 and frac(5T) is 0.25, so shake is 0.5 and the active stage-object draw parameter's first component is 1.0.

Reaction object:
- Trigger 0 fires: object 0 hidden, object 1 plays once, then object 0 loops again. A second fire after that: the same again.

Movable blocks:
- Disc example (pack 1394, three blocks with entry value 5.4517): L = 0.5 × 5.4517 = 2.7259 per push. With n = 20 the block moves 0.1363 per tick for 20 ticks (2.7259 in total), and the moving flag clears on tick 21.
- Synthetic: entry value 4.0, axis A = (1, 0, 0), direction 2 (−A), n = 8, start position (10, 0, 0): L = 2.0, 0.25 per tick; after 8 ticks the position is (8, 0, 0).
- Push length (synthetic clip values): push clip with key count 22 and clip cursor 1.5 at the move: n = (22 − 1 − 2) / 0.5 = 38 ticks; a pack 1394 block (L = 2.7259) then moves about 0.0717 per tick. A pull with key count 26 and cursor 1.5: n = 23, not doubled.
- Placement (disc, pack 1056): size S = 5.452, box corner (-2.726, 0.0, -2.726), A = (0, 0, 1), B = (1, 0, 0). If the ground probe found height 0.0 at the corner, home = position = (0.0, 0.0, 0.0), L = 2.726 and all four direction words would be 1; placing the block again gives the same L. On the disc the corner's grid cell holds one entry, model-set entry 52, which samples 3.745 there, above the ceiling 3.0: the probe returns that fallback height and slot, so the block stands at (0, 3.745, 0) with footprint patch entry 52, whose outline (z 304 to 352) does not contain the block; every push from there is refused.
- Second clearance, push (synthetic, S = 5.452 for both blocks, P at (0, 0, 0), D = 0, f = 0): Q at (0, 0, 5.9) blocks the push (dz = 5.9 is not above r + 0.5 = 5.952); Q at (0, 0, 5.96) does not; Q at (5.5, 0, 3) does not (|dx| = 5.5 is not below r - 0.05 = 5.402).
- Second clearance, pull (D = 0, f = 2, same blocks): the limit is 1.5 x 5.952 = 8.928, so Q at (0, 0, 8.9) blocks and Q at (0, 0, 8.95) does not.
- Second clearance, side move (D = 0, f = 1, w = 0): Q at (-10, 0, 3) blocks (the window [dx, dx + r] = [-10, -4.548] overlaps (-5.402, 5.402)); Q at (-11, 0, 3) does not (its window ends at -5.548). With f = 3 (w = 1) the window is [dx - r, dx]: Q at (10, 0, 3) blocks and Q at (-5.5, 0, 3) does not.
- Second clearance, backwards push (D = 2, f = 2): P at (0, 0, 0), Q at (0, 0, -5.9): the relation is Forward-Z(Q, P, 0) with dz = z(P) - z(Q) = 5.9, so it blocks.
- Direction from the hero's action (S = any): face f = 1 gives D = 1 for the push, 3 for the pull, 2 for the side move of action 144 and 0 for the side move of action 145.
- Footprint step, pack 909 (disc; the worked push sequence). Block at (-1938.887, 45.460, 1075.668), S = 2.7587, L = 1.3793, footprint patch entry 132 (outline x -1941.257 to -1928.093, z 1071.342 to 1084.512). Push +B (east, f = 1), F = position + (2.8587, 0, 0) each time: push 1 F x -1936.029, accepted, block to x -1937.508; push 2 to -1936.129; push 3 to -1934.749; push 4 to -1933.370; push 5 to -1931.991; push 6 (F x -1929.132) to -1930.611; push 7: F x -1927.753 is east of the edge -1928.093, so the outline count is 0 (even), the push is refused, the block stays at x -1930.611 and the hero strains in place. Push -B (west) from the start: F x -1941.746, refused at once.
- Footprint step, pack 1324 bell 0 (disc): on object 21's patch at z 13.349, -Z pushes of 1.6524 are accepted six times (z 11.697, 10.045, 8.392, 6.740, 5.087, 3.435); the seventh has F z = 3.435 - 3.4048 = 0.030, south of the alcove end, and is refused.
- Hero point (synthetic): block on a patch, pull with S = 2.0 (L = 1.0): H = position + D x 4.425. H off the patch with floor heights 45.46 under the block and 46.30 at H: difference 0.84 > 0.75, refused even though F is inside; with 46.10 at H (0.64) accepted.
- Second clearance on disc: pack 784 block 1 at (-10.014, -0.521, -22.244) pushed +A (f = 0) with block 0 at (-10.014, -0.521, -15.780): Forward-Z with dz = 6.464 <= 6.464 + 0.5 and dx = 0, refused before any step (its F is inside the outline).
- Character contact (synthetic, pack 1056 block at home, S = H = 5.452): a character point at (3.0, 1.0, 0.0) is 0.274 outside the +X face, inside the 0.75 band, so the contact hits; at (3.6, 1.0, 0.0) it is 0.874 outside and misses (the second test only hits if the predicted position enters the band). After a push of 2.726 along +Z the box spans z from 0 to 5.452: a point at (0.0, 1.0, -0.5) hits and one at (0.0, 1.0, -1.0) misses.
- Trigger volumes (synthetic): a push with n = 20 runs the volume test on 21 ticks (20 steps and the final tick). With the block resting inside a sphere volume afterwards, that volume fires again on each following tick until the next push starts.
- Step limit (synthetic): ground heights 2.0 and 2.75 at the two points: allowed (difference 0.75 is not above 0.75). Heights 2.0 and 2.76: blocked.
- Move to locator (synthetic): locator X = (20, 0, 20), U = (1, 0, 0), V = (0, 0, 1), s = 3.5: the transform position becomes (18.25, 0, 18.25).
- Moving-rectangle test (disc, pack 1324, plate with trigger 3 and bell block 1): plate corner (-8.449, 31.162, 1.266), eA = 0.340, eB = 5.553, eU = 10; bell S = 3.305, H = 9.2, r = 1.983, start (-5.666, 31.191, 10.198), step 1.652 along -Z. Positions after 0, 1, 2 and 3 pushes (z = 10.198, 8.545, 6.893, 5.240) hit no face. During the fourth push the position is at z = 3.588 only on its last step (step 38 of 38), where the far face and the bottom face are hit (3.588 is below the limit 1.266 + 0.340 + 1.983 = 3.590 by 0.002), so four pushes fire the plate. Bell 0 needs six pushes and bell 2 seven (starts z = 13.349 and 15.091).
- Block box flags (disc, pack 1298): the block's box has flags 0x0003 and appears in no cell list A, so after the block has moved nothing stops a character at the box's load position; a projectile that reaches the block's current box ends there.
- Inactive block (pack 1381 blocks 0 to 5 right after Init): no grab prompt appears beside block 2 (it is never the hero's current block), the block is not drawn and runs no trigger test, and a push of an active neighbour still counts it in the second clearance test.

Plate manager (synthetic; plate at P = (0, 0, 0) with r = 1.0, not pressed):
- Hero at (2.0, 1.0, 1.0): height difference 1.0 < 1.5, horizontal distance 2.236 < 2.5: pressed; the plate's height becomes -0.5.
- Hero at (2.5, 0, 0): horizontal distance 2.5 is not below 2.5: not pressed.
- Hero at (0, 1.5, 0): height difference 1.5 is not below 1.5: not pressed.
- Hero at (0, -3.0, 0): height difference -3.0 < 1.5: pressed (no lower bound).
- Sinking object at height -49.95 with its flag set: after one tick the height is -50.05, below -50, so the flag clears and it stays at -50.05. At height 0 with the flag set: -0.1 after one tick, about -1.0 after ten.
- Plates 1 and 2 pressed, plate 3 not: completion count 2, nothing runs. All three pressed, world flag bit 0 clear: the (empty) completion step runs on every tick.
- Handler pairing (synthetic): a breakable with element class 5 overlapped by Tor Wind (167) through the hero-list walk passes the element gate and breaks (state 1); the same breakable under Tor Shadow (fixed kind 6) is skipped with no break, no hook and no cue. A breakable with element class 0 breaks under any of the nine.

Standing on a block (pack 1056 block: extent 5.452, base height b, so the strict top bound is b + 5.352; hero weight 5.5, gain 1.0, root displacement zero):
- Landing tick: feet at b + 5.30 with velocity (0, −10, 0), inside the footprint by more than 0.375. The first test hits (5.30 is below 5.352), the nearest face is the top (distance 0.052), depth 0.052: feet to b + 5.352, the velocity into the face is removed, the floor flag is set, and the vertical velocity (now 0) is below 0.1, so velocity and force become zero.
- Next tick: the gravity step makes the force (0, −176, 0) (5.5 × 32). The first test misses (b + 5.352 is not below the bound). The second test's point is b + 5.352 + (0 + (−176 / 5.5) / 30) / 30 = b + 5.3164, inside the bound: top face, no push, the floor flag is set again, force and velocity are zero. Every later tick repeats this; the feet stay at b + 5.352.
- A weight-2.0 region character (Spyder) gives the same predicted drop (force −64, weight 2.0): b + 5.3164.
- Bottom face (synthetic): feet at b + 0.2 at the middle of the footprint (about 2.7 from each side). The bottom distance 0.2 + 0.75 = 0.95 is the smallest, the push depth is 0.2 + 1.125 = 1.325 downward, and the feet end at b − 1.125.

## Open questions

- Resolved (R-0048, part 3, High for the program, Medium-High for the level scripts): nothing writes animation state 5 or 6 of a placed object. Every write of the state word stores 0 (loop), 2 (play once), 4 (finished and hidden), 8 (finished at the end), 9 (stopped at the start or at a set time) or 10 (stopped), through the script calls of SPEC-0030 category 1, the set-up routines by kind (Set-up poses by kind) and the end-of-clip step. The two values that the disc stores are 3 and 7 (3,491 and 4 objects). No script call takes a state number as an argument, and the level scripts do not write object records. So states 5 and 6 never occur: IsObjAnimating is true only in states 0 and 2, IsObjAnimFinished only in states 4 and 8, and the per-tick object step runs only for states 0 and 2 (the code that also handles 5 and 6 is never reached). A rewrite may leave them out.

Numbers of removed questions are not reused.

- Open (R-0040, in part): (1) Resolved: the disc values of the piece array (+0x50) and the volume link (+0x60) are offsets from the entry's own first byte. The load adds the entry's address to each nonzero value (a value of 0 means none). Test vector: an entry at position E with +0x60 = 0x80 has its volume record at E + 0x80. At load the volume's flags word (+0x12) is set to 9, or to 33 when the volume's type byte (+0x10) is 4. (2) Resolved (Medium-High; SPEC-0036 Hit detection step 6): the hero's melee walk runs on every tick that the character scan runs, with one test per volume kind (kind 1 sphere against the swing triangle's edges; kinds 4 and 7 rectangle; kind 3 cylinder, a static pair test while the swing tip has not moved and a swept test against the swing triangle once it has; kind 2 never). The swing shape is the reach of SPEC-0036 Hit detection step 2, not a radius 1.0 cylinder from attach point 0 alone; the third vertex is the previous-tick tip (SPEC-0036 Hit detection step 6; Medium-High) and the kind 3 and 4/7 internals remain Medium. Resolved (R-0076): the point and extent roles of kinds 3, 4 and 7 are in Breakable volume shapes (cylinder: base point, one-way length; rectangle: corner and full extents). (3) Resolved (High): a second-list entry in state 1 or 3 draws its own inline mesh (+0x44, a type-12 chunk stored in the entry) or, when it has none, the near mesh of record +0x48 of the pack's first type-20 chunk, with the entry's own 4 x 4 matrix; a broken entry (state 2) is simply not drawn. See Breakable draw and its test vector.

1. Resolved: **pad bit names.** Code 0 is Square (0x0080), with packet byte 2 in the high byte of the button word (High; SPEC-0033 Controller map).
2. Resolved: **second pad word.** True reads pad slot 0's newly-pressed word and false its held word; the chest press uses the newly-pressed word (High; SPEC-0033 Controller map).
3. **Default start mode.** Resolved: the default mode is always 0, gated at load by the entry's persistent broken bit; no disc entry has start byte 1 or 2 (Start mode and reset; High).
4. Resolved: **wait of 0.4 in the arm step.** It is a wait of 0.4 s (24 ticks) during which the newly shown stage is in state 3 and cannot be broken (Switch groups; High).
5. Resolved: **lock remove caller.** The lock remove routine has no caller; an opened chest refuses presses through its object's animation state (Press rule, No repeat press; High).
6. Resolved: **opening and repeat presses.** The chest's open clip makes the gate refuse later presses, and a reload starts the chest at its last frame (Press rule, step 1; High).
7. Resolved: **attach effect argument order.** AttachEffectToMovableBlock(world, block, effect, a, b, c) builds the offset (b, c, a, 0) (Movable blocks, Attach effect; High).
8. Resolved: attacks break entries through the contact routines (Break routes); the +0x5A-gated route has no caller and is dormant (Medium).
9. Resolved: **Draik part slots and release callers.** The active class-12 controller owns nine 12-byte slots with a release latch, script-supplied breakable index and fixed bone index. Stage transitions release slots 0–2 and then 3–7; death releases slot 8; level event 17 releases all nine. Melee/projectile hit tests return body-hit results but do not directly release these parts (Draik breakable slots; High for the runtime paths).
10. Resolved: **Mover update.** A block moves L / n per tick for n ticks, with L = 0.5 × the entry's +0x10 value and n as in 33 (Movable blocks, Per tick).
11. **Break float.** Resolved in part: positive signed +0x5A is converted to float and passed with the break position to a timed effect controller; it also gates the dormant hit-break route. Its time unit and perceptual meaning remain unknown. Answered for the cue: +0x58 is the script index, not a break cue; the contact cue comes from +0x5C.
12. **Second-list table base.** Answered: the offsets are measured from the start of the offset table, which gives state 1 in every disc entry; measuring them from the payload start is wrong (Layout, Breakable entry).
13. **Piece record fields.** Resolved in part: each 0x70 record initializes one effect object; the loader resolves its +0x60 model-set-part link from the +0x68 selector and stores the part-array base at +0x64. Effect setup relocates and reads the piece-record +0x54 link. The breakable transform and source words are supplied to the spawned object. The remaining field meanings and final drawn transform are unknown (Piece record; High for the links and call inputs, Low for the complete record interpretation).
14. Resolved: **switch lists with placed objects.** A script group entry has its own placed-object list, used by the chest, sealed chest, fuse, reaction and dig-site kinds; the breakable chain uses only its breakables (Script groups; SPEC-0028 section 11; High).
16. Resolved: **lock message meaning.** The message id is a line index of the world's dialogue line table (SPEC-0022), the same table that the script call LevelMsg draws from. The lock check shows it through the same table-line routine as LevelMsg, so the speaker name comes from the line's own entry (SPEC-0022). Only one script locks a chest: pack 951 locks group 0 with key item 15 and line 124 (SPEC-0041, section 951; SPEC-0058, step 24). A lock call with message 124 in a world whose table has no line 124 shows nothing (High).
17. **Breakable spawn key (+0x54).** The drop item the request passes to the spawned character. Answered: +0x54 is the key of a world-level spawn by key at the break point, run on every break whose key is 0 or more (Break effect, step 3; SPEC-0029 Behavior 31).
18. **Code constants.** The 0.1 height threshold, the 0.4 wait, the 0.25 and 1.0 spawn values, the effect id 790, and the 0.1 margin, 0.75 reach and 1.1 factor of the push start points are code constants with no known disc data address. Answered: the 3.0 mover value is the ceiling offset of the ground probe at placement (Movable blocks, Placement), and the push step limit 0.75 is the data word at ELF address 0x2EB2C0.
19. **RegisterScriptBreakable arguments.** The meaning to the callback of the two argument words, (1, 0) for the melee and attack families and (0, 1) for Roberta's shot. Answered: the call fills the hook record for a script index (+0x58), and an attack contact then starts the callback as a script thread instead of the default break and cue (SPEC-0030 row 267; Break routes, step 1).
20. Resolved: **users of the element gate.** The seal hook and prompt thread that each element-sealed chest's set-up installs (Element gate on hits; High).
21. **Pass effects.** Resolved (High for the request sequence; conditional on effect data): with locator 0, the reward placement is requested at X + 2.2 d along −d, and a one-shot detached effect 791 is requested at X + 2.9 d, where X and d are locator 0 position and forward direction (Element gate on hits; SPEC-0034, section 3.16). The request follows the shared one-shot start path and detaches in mode 2. Locator 0 exists in all 39 packs with an element-sealed chest (and in all 146 with a lock chest), so the absent-locator fallback never occurs on disc.
22. Resolved for retail data and runtime behavior; authoring intent remains unknown. Member 0 has element class 0 throughout. All 40 nonzero member-1 classes occur in two-member chains; all 24 three-member chains have member-1 class 0. In a two-member chain, member 1 is the final stage and uses the ordinary contact path after its 24-tick state-3 guard, so its own element class gates the hit. In a three-member chain, member 1 is chain-hooked and remains element class 0; its hit advances the chain without the ordinary element gate. Member 2 is element class 0 in all three-member chains. [High for disc data and runtime ordering; intent unknown].
23. Resolved: **Breakables and member 1.** All 40 breakable entries with a nonzero element class are exactly the entries referenced as member 1 by switch groups with two or more members. All 40 occur in two-member chains; the 39 other distinct member-1 entries have element class 0, including every three-member chain. The identities are within each pack's own type-29 second list. **[High for the serialized data]**
24. **Nova at a seal.** The prompt thread does not exempt Nova, while the seal hook does. Both rules are reproduced as stated (Edge cases); whether the difference was intended is not decided.
25. Resolved: **hold loop "cues".** 29 and 30 are Square context prompts, not sound cues, so there is no throttle (Element gate on hits; High).
26. Resolved: **gated groups per level.** The element gate belongs to the 39 element-sealed chests, one in each of 39 packs (Element gate on hits; High).
27. Resolved: **follow-up on the mismatch fail.** The request queued when the hero's stance sub-object is idle is Tor's attack sub-state 13, the sword bounce (Element recoil; High).
28. **Breakable drop ids.** Whether any break uses a placement with key -1 and group -1. Answered: the drop placement ordinal (+0x56) and spawn key (+0x54) are fields of each disc entry (Layout; Break effect, step 3; SPEC-0028, section 11; High).
29. Resolved: **hero action state 4.** There is none on this route; 4 is the recoil material word (Element recoil; High).
30. Resolved: **Handler owners.** Each of the nine attack-family handlers belongs to a Mote power-attack object (SPEC-0036, Contact routines and cues). Three take the attacker's Mote kind from their owner: the owner's +0x100 field for Roberta Lava (161), Tor Water (163), Tor Nova (172) and the three Tor Rock (165) sub-shots; the owner's +0x148 field for Tor Lava (161) and Roberta Supernova (173); the owner's first word for Roberta's Shadow (169) and Sun (171) beams. Three use a fixed kind: kind 6 for Tor Shadow (169), kind 3 for Tor Sun (171) and kind 2 for Roberta's three Rock (165) skipping shots. Three further hero-list walks use kind 4 for Roberta Water (162), no gate for Roberta Supernova (173) and kind 5 for Tor Wind (167). A breakable with element class 0 yields to any of them. This is the static dispatch; it does not show which encounter activates an eligible object. [High]
31. Resolved: **piece push unit.** The push value is neither a speed nor a distance: it is a blend weight p between zero and one. In the emission step of the piece spawn, the emitter's own emission axis (its transform's second row) is mixed with the push direction d as normalize((1 - p) x own axis + p x d) before the random spread is applied. With p = 0.25 the axis is 75 percent the piece's own axis and 25 percent the direction away from the attacker; with p = 0 (the dormant +0x5A route) the axis is unchanged. The speed of the piece particles comes from the effect definition, not from the push. Medium (the branch used by emitters whose definition carries a spread block; the plain branch ignores the push).
32. Resolved: **Shot cue float.** The two floats are the hearing radius R and the full-volume radius r0 (SPEC-0031 Open question 50); Roberta's shot passes a non-zero r0.
33. Resolved: **Push tick count n.** n = key count of the hero's current clip − whole part of the clip cursor − 2, doubled only for the push clip (Push start, step 4).
34. Resolved (R-0131, replaces the earlier answer): **block footprint test and second clearance test.** Every placement writes the footprint patch (+0x9C): the placement's ground probe stores the patch it chose into that word. The footprint step therefore runs for every block on disc and is what keeps a block inside its floor patch's outline (Push start, step 2; Push limits on disc). The second clearance test is the table of Second clearance test and tests other blocks only (High).
35. Resolved: **re-placement quirk.** There is none on disc data: the size defaults never apply to a block with an inline mesh, so L is always half the stored size (Placement).
36. Resolved: **plate manager use.** Nothing fills the plate manager's slots, so the sinking flag is never set in play (Plate manager; High).
37. Resolved: **plate completion.** It never runs in play; plates in play are scripted trigger volumes (Plate manager; High).
38. Resolved: **plate solid word.** Its contact rule never runs in play (Plate manager; High).
39. **Proximity fuse details.** Answered in part: the active stage-object draw pass reads the shake value as the first component of its render parameter (Proximity fuse; Medium). What the 32 stage objects depict (stage 31 is not shown by any step) has no name data on disc (class D). Which effect is started on fire and whether the nine live fuses sit in reachable areas need a runtime check or a geometry check against the hero routes (class C). Default if unresolved (rewrite choice): implement all nine fuses as specified, and start the fire effect through the effect request of the Fire paragraph above with the shared one-shot start path.
40. Resolved: **Kinds 0, 7 and 12.** Their readers are listed in Script groups: kind 0 through GetScriptGroupInstance in 14 packs (1247's entries are unread), kind 7 through GetScriptGroupMainTrigger in all five packs that have it, kind 12 through the type callbacks that 1171 registers (1369's entries are inert). Resolved: the 1171 type-12 init starts each floating object's loop at a random time of 0 to 1 s, and the run callback sinks the object while the hero stands in its trigger and raises it when the hero leaves; the registration calls init at once and replaces earlier trigger callbacks on the same index, including the switch handler on triggers 29 to 32 (Script groups). (High)
41. Resolved (R-0131): **what limits a pushed block.** Its footprint patch outline (Push start, step 2), the hero-point height step and the second clearance test; nothing else (What does not limit a push). Per-block limits on disc are in Push limits on disc. Whether the few wall panels a block can overlap inside its outline (pack 1090 block 1, pack 1397 block 0, the side panels of pack 1381's upper room) are visible walls in play needs a runtime check (class C, Medium). Default if unresolved (rewrite choice): apply only the rules above, so these overlaps happen as in the original.
42. Resolved (R-0157): **what stops a character at a chest.** The breakable-0 cylinder of the group (flags 9, state 3), pushed out of by the motion step's breakable walk with a fixed 1.0 by 3.5 body; no frame record, inline primitive, registry model or trigger-volume solid belongs to a lock chest, and the hero cannot stand on it (Lock chests, collision; High for the rule and the numbers, Medium for the top-of-cylinder quirk and for characters other than the selected hero). Default if unresolved (rewrite choice): as stated there.
