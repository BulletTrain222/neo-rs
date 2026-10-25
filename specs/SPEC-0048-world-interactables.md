# SPEC-0048: World interactables: breakables, lock chests, switch groups and movable-block control

- Answers: proactive (world interactables; no request identified)
- Status: Draft
- Disc build: SCUS-97367 v1.00 (same fingerprint as SPEC-0001)
- Confidence: per section. Breakable entry layout and break and state rules: High for order and offsets, Medium-High for state names. The state word at +0x40 holds the value 1 in every entry on disc (2,073 of 2,073; states 0 to 3 occur at run time), under the table-relative offset reading (Where it lives). Break effect pieces: Medium; persistent broken bit and the start-mode gate: High. Attack contact routes (script hook, element gate, 0.25 push, hit cue by material): High; the +0x5A-gated route being unreachable: Medium. Switch group step: High for the code rules, Medium for meaning. Lock table, lock add and lookup: High. Button masks: High for all 16 codes and for the button names (code 0 is Square; SPEC-0033 Controller map). Second-argument word (held or newly pressed, pad slot 0): High. Press rule: High for the gates, the prompt, the key rule and the open effects; Medium for the press height units and the busy counter's meaning. Movable-block list and setters: High for offsets and writes, Medium for meaning; mover per-tick motion, push start and placement probe: Medium-High; push tick count n: High. Plate manager (three plates and one sinking object): Medium; High for its per-tick constants (0.1, -50, 1.5, 0.5); High that it is never populated in play. Element-sealed chest and breakable chain: High for the gates, hooks and waits; Medium for the start of effect 791 and the exact drop point. Breakable start mode on disc data: High. Doors: pointer to SPEC-0006 (High). Script groups (kinds and set-up): High. Proximity fuse: Medium for the rules, High for the thresholds and the hero-state gate. Reaction object: High. Movable-block disc source (the type-11 n1 list and the flag bit 1 rule): High.

## Summary

World interactables are the world-side objects that a level script or a hit can change: breakable objects that can be broken once, switch groups that break their members in order, lock chests that open with a key item, and movable blocks that a script can activate and move toward a locator and that the hero pushes one step at a time. A small per-world plate manager also runs three hero-pressed plates and one sinking object (Plate manager). The chests, the chains, a proximity fuse that breaks its breakable after a countdown, and a reaction object that replays a clip when its trigger fires are all script groups: type-40 entries whose kind picks a built-in behaviour (Script groups). This spec describes their stored records, their state rules, and the order of the effects that follow a break or an open. It builds on SPEC-0028 (the type-29 placed-object chunk, whose second list holds breakables; the type-40 switch list), SPEC-0021 (the placement drop action that a breakable can trigger), SPEC-0030 (the script calls that act on these objects) and SPEC-0018 (the movable-block walk). Those facts are not repeated here.

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
| +0x00 to +0x2F | 3 x 16 bytes | float blocks | Vectors used by the effects. Not described. | Low |
| +0x30 | 16 bytes | float vector | Break position. Passed to the drop and effect calls. | Medium-High |
| +0x40 | s32 | state word | Breakable state (section Behavior). SPEC-0028 section 10 reads the same word as the second-list kind and draws kinds 1 and 3. | High (offset and use); Medium-High (state meanings); value 0 to 3 in every entry on disc |
| +0x4C | s32 | piece count | Number of piece records. | High |
| +0x50 | pointer | piece array | Piece records of 0x70 bytes each, rebased at load. | Medium |
| +0x54 | s16 | spawn key | Character kind key of a world-level spawn by key at the break (SPEC-0029 Behavior 31); -1 for none. It is not a character activation. 0 or more in 20 of 2,073 entries on disc; for example key 1960 in pack 732, which matches the two summon-reserved Sludgy records of that pack. | High |
| +0x56 | s16 | drop placement id | Placement ordinal for the drop; -1 for none. A plain field of the disc entry: 0 or more in 1,239 of 2,073 entries on disc. | High |
| +0x58 | s16 | script index | Key of the script breakable hook (RegisterScriptBreakable; Break routes, step 1) and of the persistent broken bit (Break effect, step 6). It is not a sound index. Unique within a pack; below 128 in 2,069 of 2,073 entries on disc. | High |
| +0x5A | s16 | break float | When above 0, the break effect runs its timed effect with this value (Break effect, step 5). The attack contact routines do not test it; the only break route gated on it is never reached (Break routes). | Medium (use); Low (meaning) |
| +0x5C | s16 | contact material | Selects the hit cue of the attack contact routines: 2 -> cue 143, 3 -> cue 145, 4 -> cue 146, any other value -> cue 144 (SPEC-0031, Breakable contact cues). On disc (2,073 entries in 79 world packs): 0 in 1,557, 1 in 294, 3 in 19, 4 in 203, 2 in none. | High |
| +0x5E | u8 | element class (alignment) | 0 = any attack may act on the entry; 1 to 6 = the Mote kind an attack must have to act on it (see Break routes, step 2, and Element gate on hits); 7 is not used on disc. Returned by the alignment query. On disc (2,073 entries in 79 world packs; 121 packs have a type-29 chunk): 0 in 2,033 entries, 1 in 9, 2 in 6, 3 in 6, 4 in 6, 5 in 7, 6 in 6, 7 in none. | High (offset and counts); Medium (meaning of 1 to 6 as attack kinds) |
| +0x5F | u8 | start mode | 0 means the default mode, which is 0. Mode 1 and mode 2 have their own effects; any other value gives state 1 (section Behavior). Applied at world load only to an entry whose broken bit is set. On disc: 0 in 1,974 entries, 10 in 13, 20 in 3, 30 in 83; 1 and 2 in none | High |
| +0x60 | pointer | sub-record | Optional record: the breakable's type-11 volume (present in 2,030 of 2,073 entries on disc). Its type byte is at +0x10 (u8) and its volume flags word is the s16 at +0x12, which a break sets to 0. | High |

Word 0x16 of the entry, counted in 32-bit words (0x16 x 4 = 0x58), is the script index; it is not a break cue (High). The break effect itself plays no cue; the hit cue comes from the contact routines and +0x5C.

### Piece record (0x70 bytes)

Each piece record has a transform and fields that are used as the placement of one spawned effect object. The layout is the frame-record shape of SPEC-0018 (model-set entries). Field use here: Low.

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
| +0x00 | vec4 | home position (the block's position when it was placed) | Medium-High |
| +0x10 | vec4 | axis A (push directions 0 and 2) | Medium-High |
| +0x30 | vec4 | axis B (push directions 1 and 3) | Medium-High |
| +0x40 | vec4 | current position | Medium-High |
| +0x50 | vec4 | position minus home | Medium-High |
| +0x68 | pointer | the block's entry (91 entries on disc): its +0x10 value sets the step length, its +0x18 word is −1 in all 91 entries (this selects the default sizes), and it links the transform record the mover reads | Medium |
| +0x6C, +0x70 | float | mover parameters | Low |
| +0x74 | s32 | active word, written by the activate call; the per-tick step runs only while it is non-zero | High |
| +0x78 to +0x84 | 4 x s32 | direction-allowed words, one per push direction 0 to 3 | Medium-High |
| +0x88 | s32 | moving flag | Medium-High |
| +0x8C | f32 | step length L of one push | Medium-High |
| +0x90 | u8 | tick counter i of the current push | Medium-High |
| +0x91 | u8 | tick count n of the current push | Medium-High |
| +0x94 | s32 | push direction D: 0 = +A, 1 = +B, 2 = −A, 3 = −B | Medium-High |
| +0x98 | pointer | world reference | Medium |

Index checks: every call tests the index against the list count.

### Locator record (0x50 bytes, locator list at world +0x56B0)

The list header holds the count at +0x24. Records start at +0x4C of the header, each 0x50 bytes. The locator position is the vector at +0x30 of its record. Medium.

### Plate manager (world +0x2D40)

Runtime only. The block starts with a slot count word at +0x00 (the manager's reset sets it to 8); the slots are 0x160 bytes each and start at +0x10 of the block (world +0x2D50). The slot index decides the slot's role: slot 0 is the sinking object, slots 1 to 3 are the three plates, and slots 4 to 7 have no per-tick behavior.

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
4. For each piece record (count at +0x4C, array at +0x50), one piece object is spawned at the record's transform, with the break source words as the source. The break's push value and direction are handed to each piece spawn as its push: 0.25 along d from the attack contact routines (Break routes, step 3), 0 from the dormant +0x5A route (Medium for the spawn; the effect spawn itself is in SPEC-0034).
5. If the break float (+0x5A) is above 0, the timed effect call runs with that float and the break position.
6. The entry's broken bit is set in the persistent broken set: the row of the current world index (world index word at world +0x5900), bit = script index (+0x58). Script indexes of 128 and above are not recorded (High).

The break effect plays no cue. The dormant +0x5A route copies the source words at world +0x5780, +0x5850 and +0x5854 into three globals that the piece spawns read, before its break (High for the copy; Low for its meaning). Whether the contact routines set these words is not described here.

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
8. When locator 0 exists, the placement of pair[0] is requested at X + 2.2 d and launched along −d, where X and d are the position and forward direction of locator 0. The separate effect transform is at X + 2.9 d. The character of pair[1] is activated (as for a lock chest). If locator 0 is absent, the fallback placement point is not established. (High for the coordinates passed to the spawn and effect setup; effect 791 playback remains open.)
9. Wait 2.0. The normal camera returns. The key row of the entry is set to 1.

The fixed waits add up to 11.3 s, plus the wait on camera 0.

**Prompt thread** (Square context prompts 29 and 30). The thread starts on a repeated fire of trigger 0 (trigger state 2) and runs each tick while the main hero stays in the volume and breakable 0 is intact. On each tick it uses the mismatch test E_t != 0 and E_a != E_t:
- while the mismatch holds, context prompt 29 is shown on every tick until Square has been held once; it shows again after E_a has matched;
- otherwise context prompt 30 is shown on every tick until Square has been held once.
These are context prompts (SPEC-0023), not sound cues. The thread does not exempt Nova (E_a = 7), so a hero with Nova at a seal with E_t = 3 sees prompt 29, while the seal hook passes Nova.

Counts on disc: a switch entry has 0 members in 224 of 458 entries, 1 member in 155, 2 in 55 and 3 in 24 (61 level packs have members). Across the 337 members, member index 0 is 0 in every case and member index 2 is 0 in every case. Member index 1 is nonzero in 40 of the 79 groups with two or more members (values 1: 9, 2: 6, 3: 6, 4: 6, 5: 7, 6: 6). Those 40 nonzero member-1 references are exactly the 40 breakable entries with a nonzero element class; the other 39 distinct member-1 entries have class 0 (Open question 23). So on disc the gate tests only member 1.

### Lock chests

There is no chest object type. A chest is a type-40 group entry (SPEC-0028, section 11) of kind 1 (Square-press chest) or kind 2 (break chest opened by a hit with a matching Mote element). A lock chest is a kind-1 group whose group index is also an id in the lock table.

**Shared collected table (High).** Kinds 1 and 2 use the same collected table as kind 6, the treasure dig (SPEC-0051). A chest's key (+0x08) is its collected entry, in the range 1 to 199 (one kind-1 chest has key 0); dig sites use 200 and above. Chests need no treasure map. Both chest kinds end their open with the drop action on their reward list and then set their collected entry to 1. On a later world load the set-up shows the opened pose for a collected chest: a lock chest's object starts at its last frame (High).

**Set-up (kind 1, at each world load; High).** Breakable 0 (the chest body) is made active and not breakable (state 3), with a hook whose only effect is Tor's recoil with material 4 (Element recoil). Trigger 0 gets the chest thread. Object 0 (the chest) is drawn in state 9 at time 0, or, when the key row is already set, at its last frame in the finished state 8. Object 1 is not used.

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
5. While T > 2.8 s: on ticks where floor(5 T) is even, a shake value of 0.5 times a sine of frac(5 T) is set; on other ticks it is 0. The active-stage draw pass consumes this value in the first component of a four-component render parameter, as shake + 0.5; the other components are 0.5, 0.5 and 1.0. The exact visible meaning of that parameter is not established.
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

Confidence: Medium-High for placement, push start and the per-tick motion; High for the push tick count n.

A movable block moves only in whole pushes. Each push moves it by its step length L along one of four directions, spread evenly over n ticks. The 3.0 value is not a speed: it is the ceiling offset of the ground probe used when the block is placed.

**Source (High).** At world load each n1 entry of the type-11 chunk whose box primitive (entry +0x20) has flag bit 1 (0x0002) set becomes one 0xB0-byte block record, in entry order; entries without a box, or whose box has bit 1 clear, build nothing (Where it lives).

**Placement (init, move to locator, reacquire).** The block's home and current position are set from its transform record, and its height is probed with the ground probe of SPEC-0018 with ceiling = its height + 3.0. The step length is L = 0.5 × the entry's +0x10 value, read before the defaults are applied. When the entry's +0x18 word is negative (−1 in all 91 entries on disc), the defaults are then written into the entry: size 3.5, extents 5.0, 3.5 and 3.5. The first placement therefore uses the disc value, and a block that is placed again later reads 3.5 and gets L = 1.75 (quirk, Edge cases).

**Push start.** A push starts only when the block is not moving and the push direction D is allowed (its direction-allowed word is non-zero). With D the unit vector of the direction (+A, +B, −A or −B):
1. Block front F = position + D × (L + 0.5 × box size + 0.1). Hero target H = hero position + D × (L + 0.75) × 1.1.
2. When the block has an area test, both points must pass it. Otherwise the ground heights at the two points (ground probe with ceiling + 8.0) must differ by no more than the step limit 0.75 (ELF data word 0x2EB2C0, the step limit of SPEC-0018's move check).
3. A second clearance test must also pass.
4. Then moving = 1, i = 0, the direction is stored, and n = (key count of the hero's current clip − the whole part of the clip cursor − 2), kept as a signed byte. Only for the push clip (action 42) is the count divided by the half-rate word 0x368FA4 (0.5), so a push lasts twice as many ticks. Pull and side moves (actions 41, 144 and 145) are not doubled, so they finish in about half the clip's remaining time (quirk; reproduce). The count is taken when the hero's block state starts the move, once per clip cycle (SPEC-0033, Behavior 4A.2).

**Per tick.** The movable-block list is walked by the world tick, after the object animations of SPEC-0028 and before any character update (SPEC-0018, Movable-block list). For a block whose active word (+0x74) is non-zero: if it is moving and i < n, position += D × (L / n), the position-minus-home vector is updated and i += 1; if it is moving and i ≥ n, moving = 0 with no motion (so the flag clears on the tick after the n-th step). Effects attached to the block follow it. Per-tick speed is L / n; a push covers L in n ticks.

**Climb onto a block.** The hero's climb onto a block (state 58) is gated by the climb-onto-block fit test of SPEC-0033 (Behavior 15): the hero walking at a block face, the block top 0.2 to 3.5 above the hero's feet, the facing toward the face and the stick pushed into the block. The block top it reads is the box as loaded; a push does not update it.

**Activate.** Writes the int argument to the block's +0x74 word, for a valid index.

**Move to locator.** For a valid block index and a valid locator index, centres the block on the locator position X (+0x30 of the locator record): the transform record's position becomes X − 0.5 × s × U − 0.5 × s × V, where U and V are the two horizontal axis vectors of the block's transform record (its vectors at +0x50 and +0x40) and s is the entry's size (+0x10; 3.5 once the defaults have been applied). Then the block is placed again (Placement).

**Reacquire height patch.** Places the block again (Placement): the home, the position and the ground height are taken anew from its transform record.

**Sound index.** Stores the int argument at world +0x2CCC. A later sound call reads that word as the movable-block sound index (Medium).

**Make unmovable.** The script call MakeMovableItemUnMovable has an empty body. It does nothing.

**Attach effect (High).** AttachEffectToMovableBlock(world, block, effect, a, b, c) attaches placed effect "effect" to block "block" at an offset. The block index is checked against the movable-block count and the effect index against the world's placed-effect count; an index out of range does nothing. The offset vector is (b, c, a, 0): the three floats are taken in the order second, third, first.

**Hero state query.** IsHeroMovingBlock is named. Its meaning belongs to the hero spec (SPEC-0036).

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
- **Lock table full.** The third lock is dropped with a log call. Whether the log prints in the retail build is not established (Medium).
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
- **Block placed a second time (quirk).** The step length is taken from the entry's +0x10 value before the defaults overwrite it with 3.5. A block that is placed again (move to locator, reacquire, or a later init) therefore gets L = 0.5 × 3.5 = 1.75, whatever its disc value. Reproduce (Medium-High).
- **Push while moving.** A push request while the moving flag is set, or in a direction whose allowed word is zero, does nothing.
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
- Disc example (pack 1394, three blocks with entry value 5.45): L = 0.5 × 5.45 = 2.725 per push. With n = 20 the block moves 0.13625 per tick for 20 ticks (2.725 in total), and the moving flag clears on tick 21.
- Synthetic: entry value 4.0, axis A = (1, 0, 0), direction 2 (−A), n = 8, start position (10, 0, 0): L = 2.0, 0.25 per tick; after 8 ticks the position is (8, 0, 0).
- Push length (synthetic clip values): push clip with key count 22 and clip cursor 1.5 at the move: n = (22 − 1 − 2) / 0.5 = 38 ticks; a pack 1394 block (L = 2.725) then moves about 0.0717 per tick. A pull with key count 26 and cursor 1.5: n = 23, not doubled.
- Re-placement quirk (synthetic): the same block placed again (reacquire) gets L = 1.75, so the next push with n = 8 moves 0.21875 per tick.
- Step limit (synthetic): ground heights 2.0 and 2.75 at the two points: allowed (difference 0.75 is not above 0.75). Heights 2.0 and 2.76: blocked.
- Move to locator (synthetic): locator X = (20, 0, 20), U = (1, 0, 0), V = (0, 0, 1), s = 3.5: the transform position becomes (18.25, 0, 18.25).

Plate manager (synthetic; plate at P = (0, 0, 0) with r = 1.0, not pressed):
- Hero at (2.0, 1.0, 1.0): height difference 1.0 < 1.5, horizontal distance 2.236 < 2.5: pressed; the plate's height becomes -0.5.
- Hero at (2.5, 0, 0): horizontal distance 2.5 is not below 2.5: not pressed.
- Hero at (0, 1.5, 0): height difference 1.5 is not below 1.5: not pressed.
- Hero at (0, -3.0, 0): height difference -3.0 < 1.5: pressed (no lower bound).
- Sinking object at height -49.95 with its flag set: after one tick the height is -50.05, below -50, so the flag clears and it stays at -50.05. At height 0 with the flag set: -0.1 after one tick, about -1.0 after ten.
- Plates 1 and 2 pressed, plate 3 not: completion count 2, nothing runs. All three pressed, world flag bit 0 clear: the (empty) completion step runs on every tick.

## Open questions

Numbers of removed questions are not reused.

1. Resolved: **pad bit names.** Code 0 is Square (0x0080), with packet byte 2 in the high byte of the button word (High; SPEC-0033 Controller map).
2. Resolved: **second pad word.** True reads pad slot 0's newly-pressed word and false its held word; the chest press uses the newly-pressed word (High; SPEC-0033 Controller map).
3. **Default start mode.** Resolved: the default mode is always 0, gated at load by the entry's persistent broken bit; no disc entry has start byte 1 or 2 (Start mode and reset; High).
4. Resolved: **wait of 0.4 in the arm step.** It is a wait of 0.4 s (24 ticks) during which the newly shown stage is in state 3 and cannot be broken (Switch groups; High).
5. Resolved: **lock remove caller.** The lock remove routine has no caller; an opened chest refuses presses through its object's animation state (Press rule, No repeat press; High).
6. Resolved: **opening and repeat presses.** The chest's open clip makes the gate refuse later presses, and a reload starts the chest at its last frame (Press rule, step 1; High).
7. Resolved: **attach effect argument order.** AttachEffectToMovableBlock(world, block, effect, a, b, c) builds the offset (b, c, a, 0) (Movable blocks, Attach effect; High).
8. Resolved: attacks break entries through the contact routines (Break routes); the +0x5A-gated route has no caller and is dormant (Medium).
9. **Draik part slots.** The boss uses nine breakable slots (SPEC-0030 row 78). The slot hit routine and its caller are not fully described, and the boss owner spec for these parts was not identified. The reset (slot flags to 0, parts to state 0) is High.
10. Resolved: **Mover update.** A block moves L / n per tick for n ticks, with L = 0.5 × the entry's +0x10 value and n as in 33; a block placed a second time gets L = 1.75 (quirk) (Movable blocks, Per tick).
11. **Break float.** The meaning of the break float at +0x5A (timed value, duration or intensity). Answered for the cue: word 0x16 of the entry is the script index +0x58, not a break cue; the hit cue comes from +0x5C.
12. **Second-list table base.** Answered: the offsets are measured from the start of the offset table, which gives state 1 in every disc entry; measuring them from the payload start is wrong (Layout, Breakable entry).
13. **Piece record fields.** The use of the fields in the 0x70 piece records is Low.
14. Resolved: **switch lists with placed objects.** A script group entry has its own placed-object list, used by the chest, sealed chest, fuse, reaction and dig-site kinds; the breakable chain uses only its breakables (Script groups; SPEC-0028 section 11; High).
16. **Lock message meaning.** Whether the message id in the lock table is a dialogue id or a level message id is open (SPEC-0022).
17. **Breakable spawn key (+0x54).** The drop item the request passes to the spawned character. Answered: +0x54 is the key of a world-level spawn by key at the break point, run on every break whose key is 0 or more (Break effect, step 3; SPEC-0029 Behavior 31).
18. **Code constants.** The 0.1 height threshold, the 0.4 wait, the 0.25 and 1.0 spawn values, the effect id 790, and the 0.1 margin, 0.75 reach and 1.1 factor of the push start points are code constants with no known disc data address. Answered: the 3.0 mover value is the ceiling offset of the ground probe at placement (Movable blocks, Placement), and the push step limit 0.75 is the data word at ELF address 0x2EB2C0.
19. **RegisterScriptBreakable arguments.** The meaning to the callback of the two argument words, (1, 0) for the melee and attack families and (0, 1) for Roberta's shot. Answered: the call fills the hook record for a script index (+0x58), and an attack contact then starts the callback as a script thread instead of the default break and cue (SPEC-0030 row 267; Break routes, step 1).
20. Resolved: **users of the element gate.** The seal hook and prompt thread that each element-sealed chest's set-up installs (Element gate on hits; High).
21. **Pass effects.** Resolved (High for the request sequence; conditional on effect data): with locator 0, the reward placement is requested at X + 2.2 d along −d, and a one-shot detached effect 791 is requested at X + 2.9 d, where X and d are locator 0 position and forward direction (Element gate on hits; SPEC-0034, section 3.16). The request follows the shared one-shot start path and detaches in mode 2. If locator 0 is absent, the fallback spawn point is not established.
22. **Member 1 only.** On disc the gate tests member 1 in every switch entry, and member 0 always has element class 0. Whether this is a design rule or only a property of the data is not decided.
23. Resolved: **Breakables and member 1.** All 40 breakable entries with a nonzero element class are exactly the entries referenced as member 1 by switch groups with two or more members. The 39 other distinct member-1 entries have element class 0. The identities were compared within each pack's type-29 second list; no cross-pack identity is implied. **[High for the serialized data]**
24. **Nova at a seal.** The prompt thread does not exempt Nova, while the seal hook does. Both rules are reproduced as stated (Edge cases); whether the difference was intended is not decided.
25. Resolved: **hold loop "cues".** 29 and 30 are Square context prompts, not sound cues, so there is no throttle (Element gate on hits; High).
26. Resolved: **gated groups per level.** The element gate belongs to the 39 element-sealed chests, one in each of 39 packs (Element gate on hits; High).
27. Resolved: **follow-up on the mismatch fail.** The request queued when the hero's stance sub-object is idle is Tor's attack sub-state 13, the sword bounce (Element recoil; High).
28. **Breakable drop ids.** Whether any break uses a placement with key -1 and group -1. Answered: the drop placement ordinal (+0x56) and spawn key (+0x54) are fields of each disc entry (Layout; Break effect, step 3; SPEC-0028, section 11; High).
29. Resolved: **hero action state 4.** There is none on this route; 4 is the recoil material word (Element recoil; High).
30. **Handler owners.** Which attack family owns each of the nine contact handlers (fixed kinds 2 to 6, the three owner-kind handlers, the ungated one) is Medium.
31. **Piece push unit.** Whether the 0.25 push handed to the piece spawns is a speed or a distance is not established (SPEC-0034).
32. Resolved: **Shot cue float.** The two floats are the hearing radius R and the full-volume radius r0 (SPEC-0031 Open question 50); Roberta's shot passes a non-zero r0.
33. Resolved: **Push tick count n.** n = key count of the hero's current clip − whole part of the clip cursor − 2, doubled only for the push clip (Push start, step 4).
34. **Block area test and second clearance test.** Which blocks have an area test, and the rule of the second clearance test at push start, are not described here (Medium).
35. **Re-placement quirk in play.** Whether any level places a block a second time (move to locator or reacquire after the first init), so that the 1.75 step length is seen in play, is not established.
36. Resolved: **plate manager use.** Nothing fills the plate manager's slots, so the sinking flag is never set in play (Plate manager; High).
37. Resolved: **plate completion.** It never runs in play; plates in play are scripted trigger volumes (Plate manager; High).
38. Resolved: **plate solid word.** Its contact rule never runs in play (Plate manager; High).
39. **Proximity fuse details.** Answered in part: the active stage-object draw pass reads the shake value as the first component of its render parameter (Proximity fuse; Medium). What the 32 stage objects depict (stage 31 is not shown by any step), which effect is started on fire, and whether the nine live fuses sit in reachable areas remain open (Medium).
40. Resolved: **Kinds 0, 7 and 12.** Their readers are listed in Script groups: kind 0 through GetScriptGroupInstance in 14 packs (1247's entries are unread), kind 7 through GetScriptGroupMainTrigger in all five packs that have it, kind 12 through the type callbacks that 1171 registers (1369's entries are inert). Resolved: the 1171 type-12 init starts each floating object's loop at a random time of 0 to 1 s, and the run callback sinks the object while the hero stands in its trigger and raises it when the hero leaves; the registration calls init at once and replaces earlier trigger callbacks on the same index, including the switch handler on triggers 29 to 32 (Script groups). (High)
