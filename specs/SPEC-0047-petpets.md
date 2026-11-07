# SPEC-0047: Petpets (companion creatures)

- Answers: proactive (open question sweep, enemy details); proactive, none requested (companion creature); proactive (NPC pass); R-0035 (in part: world changes, the leave rule, the intake rounding and open questions 41 to 44); R-0094; proactive (petpet pass); proactive (verify pass: petpet spawn heading); proactive (open close-out); proactive (open question sweep, round 6); R-0245 (answered from the code; not run in the original)
- Status: Draft
- Disc build: SCUS-97367 v1.00 (same fingerprint as SPEC-0001). The data words below are in the boot program's data and are read by virtual address. Values were checked against the disc image.
- Confidence: per section. Kind table and hit-base values: High (words checked). Membership sets: High (words checked). Package numbers and package pet ids: Medium-High. Acquisition (joining by feeding a wild petpet) and scene-start restore: High. Multipliers: Medium (code constants, not data). State selection: High for the rules, Medium for the roam and hover details. Strike and score: Medium; the 3600 factor in the score (tick rate 60 x 60 seconds) is High. Spawn and package rules: Medium. Save fields 0x214, 0x218 and 0x21C: High for the offsets (owned by SPEC-0025), High for the no-pet value -1. Second kind table: Medium. Script query presence gate: High. Class 20 (second petpet behaviour): High for the four states and the choice rules; Medium for the hero reaction, the object consumption and the effects; the item test is Medium-High for shot-slot behavior and Medium for the second-check receiver relationship. Statue puzzle objects (section "Statue puzzle objects"): High that they are not petpets or companions and that none exists in play. Behaviour classes: High that the summoned petpet is requested into class 34 and that class 33 is the row default of the placed pet rows; class 34 controller: High for its structure and the remaining-time countdown, Medium for the state bodies. Passive bonuses: High for the kind tests, the four data words and the per-tick test; High for which hero products the two damage factors enter (SPEC-0036); High for the heal rate of 1 point per second; Medium-High for the freeze quirk. Petpet build guide: High for the follow rules, the clip ids, the speeds and the leave and teleport rules; Medium-High for the push, hit and eat-anywhere consequences; Medium for the sound-block residency and the spawn clip.

## Summary

A petpet is a companion creature that belongs to the hero. Each hero has at most one active petpet, and the petpet follows the active hero. The petpet's kind is a number from 0 to 9, read from its pet type id. Level scripts can read that kind through the host query GetPlayerPetPetType (SPEC-0030, row 199).

A petpet joins the hero by being fed. Wild petpets are placed characters of the petpet kinds in the world packs; when the hero feeds one a food item, it eats the food and becomes the hero's live petpet (see "Acquisition (joining)"). A petpet also becomes active when a scene starts after a save was loaded whose pet type field holds a pet type id. A fresh game starts with no petpet.

The hero's live petpet (class 34) follows the selected hero by root motion: while the hero moves it heads for a point 4 units ahead of the hero, walking or running by distance; while the hero stands it walks to a random point 3 to 10 units away on each world axis and idles there. It has no distance leash; a petpet that bumps a wall stands still until it is out of the camera's view and then reappears 10 units from the hero (see "Petpet build guide"). A wild petpet (class 33) keeps a loose distance to the hero: when it is 10 or more units away it roams to a random point around its own home point (its placement position): X and Z each offset by a whole number from -20 to 19 (Roam point rule, State 3); the hero is not the centre. When it is within 10 units of the hero and, after it has been fed, a food item is within 10 units, it goes for it (state 4, Strike). It eats once its eat animation ends; a 5 s fallback timer applies the food only when no petpet is in its eating step. The hit value is a base value set by the target's type code, times a multiplier set by the petpet kind. The type codes are food item ids, and the hit value is the food's value in minutes (see "Hit value and score"). The petpet keeps a remaining companion time T in ticks and the full time F set at its last meal. T and F are saved in the save slot (SPEC-0025, offsets 0x218 and 0x21C) and restored on load.

The active petpet also gives its hero passive bonuses by kind: a 1.25 factor on a hero damage product (kinds 0 and 7), a 0.9 factor on the hero's damage-intake product (kinds 1 and 7), extra hit points (kinds 6 and 7) and extra magic (kinds 3 and 7). See "Passive bonuses to the hero".

The summoned (live) petpet runs behaviour class 34, a seven-state controller that stays until its remaining time T falls below 1 and then leaves. Class 33, whose six-state loop the State selection, States and Strike sections describe, is the default class of the placed pet rows. See "Class 34 (summoned petpet controller)".

A second petpet behaviour (class 20) reacts to the hero, approaches and consumes nearby world objects, and wanders. It is described in its own section below.

## Where it lives on the disc

- Behaviour classes. The summoned petpet is requested into behaviour class 34 at priority 2 when it spawns (High). Class 33 is the default class of the placed pet rows 244, 245, 247, 250 and 251 of the monster table (resource 530, SPEC-0029, Layout 4); the six-state loop described under State selection and States is class 33. The class table is 40 consecutive words at virtual address 0x312C10 (word index = class id, see SPEC-0029, Layout 3). The clean side does not copy the table entries; it implements the rules below.
- The second petpet behaviour (class 20) is word 20 of the same table; see the class 20 section.
- Base hit values: nine 32-bit float words, 4 bytes apart, from virtual address 0x2DB1DC to 0x2DB1FC (addresses 0x2DB1DC, 0x2DB1E0, 0x2DB1E4, 0x2DB1E8, 0x2DB1EC, 0x2DB1F0, 0x2DB1F4, 0x2DB1F8, 0x2DB1FC). Their values are 2.0, 5.0, 10.0, 2.0, 5.0, 10.0, 10.0, 30.0, 60.0 in that order. Read them at runtime; do not copy them into the clean code. The word each target type code reads is in the table below and in open question 42; the word at 0x2DB1E8 is read by no code.
- Membership sets: three arrays of eight 32-bit signed integers, one per set. Set A starts at virtual address 0x315C88, set C at 0x315CA8 and set B at 0x315CC8. An entry of -1 is unused. Contents are in Layout.
- Summoned petpet low-time mark: the float word at virtual address 0x2DB1D8 (value 0.5, in minutes), read by class 34; 0.5 x 60 x 60 = 1,800 ticks (High). It is not a lifetime. The 0.5 food multiplier does not read this word (open question 12). The 0.75 and 1.5 multipliers are not data words; they are constants of the behaviour (see Layout).
- Model packages: a petpet uses one character package from the archive, by kind. Package for kind k is archive resource 534 plus k (SPEC-0001). The table is in Layout.
- Save fields: slot offsets 0x214 (type id), 0x218 (remaining time T) and 0x21C (full time F) of a save slot. These offsets are owned by SPEC-0025 (Petpet fields); this spec uses them as given. The offset 0x10C is not a petpet field.
- Host query: row 199 of the level script API (SPEC-0030).
- Passive bonus words: four 32-bit floats in the boot program's data, 0.25 at 0x2DD8DC, 0.1 at 0x2DD8E0, 1.0 at 0x2DD8E4 and 1.0 at 0x2DD8E8 (values checked). The tick rate that scales the last two is the run-time value of SPEC-0007 (60).
- Class 20 constants at virtual addresses 0x2DB290, 0x2DB294, 0x2DB298, 0x2DB2AC and 0x2DB2B0 (see the class 20 section).

## Layout

### Pet type id to kind

Pet type ids are not item ids (item ids run from 0 to 322; SPEC-0021). The companion code maps a pet type id to a kind through a primary table. Any other id gives no kind.

| Pet type id | Kind |
| --- | ---: |
| 450 | 0 |
| 703 | 1 |
| 1314 | 3 |
| 1460 | 6 |
| 2090 | 7 |
| 2094 | 8 |

Activation, restore and the script query use only this primary table (Medium-High: found in the code, not from a data table).

A second table gives ids for four more kinds. A character check reads it (Medium; open question 3). Activation, restore and the save never use it:

| Pet type id | Kind |
| --- | ---: |
| 805 | 2 |
| 1350 | 4 |
| 1368 | 5 |
| 2335 | 9 |

One character-eligibility gate uses the second-table check as a rejection condition: if the character's type id maps to any of these four kinds, that gate returns false. For example, a character with type id 805 (kind 2) fails the gate. That gate is the enemy-kind "can be hit" query (SPEC-0036, Hittable predicate, tests 4 and 5): hero swings, hero shots and the Mote and charm walks skip a character whose spawn-record pet type id has a kind in either table, and they skip the second actor record. The primary table's other users are activation, restore and the script query described above.

A saved type of 805, 1350, 1368 or 2335 gives no kind on load, so nothing spawns (Medium-High).

### Packages by kind

The activation path selects archive resource 534 plus the kind (High). Each package holds one character record whose pet type id is a 16-bit value at record offset 0xAC (the first kind-3 record of the package's type-36 chunk). The loaded package callback selects character record ordinal 0 from that same loaded resource and uses it to create the live petpet, so the runtime type record is that package record (High; see the selection chain in the).

| Kind | Package (archive resource) | Pet type id in package | Reachable by activation |
| ---: | ---: | ---: | --- |
| 0 | 534 | 450 | yes |
| 1 | 535 | 703 | yes |
| 2 | 536 | 805 | no (no kind on activation) |
| 3 | 537 | 1314 | yes |
| 4 | 538 | 1350 | no |
| 5 | 539 | 1368 | no |
| 6 | 540 | 1460 | yes |
| 7 | 541 | 2090 | yes |
| 8 | 542 | 2090 | yes, by id 2094; see the kind 8 note |
| 9 | 543 | 2335 | no |

Kind 8 note (Medium): activation by id 2094 loads package 542, but that package carries id 2090. A kind 8 petpet therefore saves as 2090 and loads back as kind 7. Packages 541 and 542 differ only in the placement values of the first record.

### Membership sets for hit multipliers

Each set lists type codes. Unused slots are -1 and are ignored. Values are taken from the disc.

| Set | Members (type codes) |
| --- | --- |
| A | 150, 146, 156, 58 |
| B | 149, 157, 147, 59 |
| C | 153, 155, 152, 57, 56, 55, 50 |

Set C counts as a member of both "in A" and "in B" tests (see Behavior). "In A or C" means the code is in set A or in set C. "In B or C" means it is in set B or in set C.

### Base hit value by target type code

| Target type code | Base value (units) |
| --- | ---: |
| 145 (anchor) | 0 |
| 146 | 5.0 |
| 147 | 10.0 |
| 148 | 0 |
| 149 | 5.0 |
| 150 | 2.0 |
| 151 | 0 |
| 152 | 60.0 |
| 153 | 10.0 |
| 154 | 0 |
| 155 | 30.0 |
| 156 | 10.0 |
| 157 | 5.0 |
| any other code | 0 |

High: the values are eight of the nine words listed under Where it lives, assigned to the codes as in open question 42.

### Kind multipliers

| Kind | Multiplier 1.5 | Multiplier 0.75 | Multiplier 0.5 | Otherwise |
| --- | --- | --- | --- | --- |
| 0 | target in A or C | - | - | 1.0 |
| 1 | target in B or C | - | - | 1.0 |
| 3 | target in B or C | target in A or C | - | 1.0 |
| 6 | target in A or C | target in B or C | - | 1.0 |
| 7 | - | - | target in A, B or C | 1.0 |
| 2, 4, 5, 8, 9 and others | - | - | - | 1.0 |

Set C belongs to both tests. For kinds 3 and 6 the 1.5 test is made first, so a set C item gives 1.5 to kinds 3 and 6 (never 0.75). An item that is in none of the tested sets gives 1.0. The multipliers 0.75 and 1.5 are constants of the behaviour (Medium). The 0.5 multiplier is also a constant of the behaviour: the routine that computes the food value uses the three multipliers as fixed values, and the word at 0x2DB1D8 is read only by class 34 (High; open question 12).

### Save fields (slot-relative, SPEC-0025)

The save field offsets are owned by SPEC-0025. They are repeated here so the petpet rules can be read on their own.

| Offset | Size | Field | Notes |
| --- | --- | --- | --- |
| 0x214 | 4, signed | Petpet type id | Written as -1 (0xFFFFFFFF) when there is no pet or its T is below 1, and as the pet's type id otherwise. An empty slot holds -1. A value with no kind spawns nothing |
| 0x218 | 4, signed | Remaining time T | The live pet's remaining companion time in ticks at save time; 0 with no pet |
| 0x21C | 4, signed | Full time F | The full time set at the pet's last meal; 0 with no pet |

At save time the three values are read from the live pet (type, T, F). On load they become pending values for the next scene start (Behavior, Scene start).

### Pet manager state (offsets from the pet manager block)

The pet manager is the per-hero block that owns the petpet. Offsets are from its base.

| Offset | Size | Field | Meaning |
| --- | --- | --- | --- |
| 0x28 | pointer | Live petpet | Zero when no petpet is active. Set only by a spawn; cleared by deactivate and by the subsystem reset |
| 0x2C | pointer | Child list head; asset loader record | The petpet lookup by id reads it. It also holds the asset loader record, whose busy word (+0x10) must be 0 before a new load starts |
| 0x30 | pointer | Child list head | Read by the petpet lookup by id |
| 0x40 | 4 | Pending load flag | Set to 1 by the save load. Cleared at scene start |
| 0x78 | 4, signed | Pending pet type | -1 when none |
| 0x7C | 4, signed | Pending remaining time T | 0 when none |
| 0x80 | 4, signed | Pending full time F | 0 when none |
| 0x84, 0x88 | pointers | Child list heads | Cleared by the subsystem reset |

The subsystem reset sets the pending type to -1, both pending times to 0, and clears the live petpet and the child list heads. The save load fills the pending values from the save slot (SPEC-0025 offsets 0x214, 0x218 and 0x21C).

### Petpet entity fields (offsets from the petpet entity base)

| Offset | Size | Field | Meaning |
| --- | --- | --- | --- |
| 0x14C | 4 | Object id | The character's id. Used by the lookup and by the joining activation of a wild petpet |
| 0xA44 | pointer | Type record | The character type record of the petpet |
| 0xAC (in the type record) | 2, unsigned | Pet type id | The id that the kind lookup reads |
| 0xE34 | 4, signed | Remaining time T | Remaining companion time in ticks. The presence gate, the regeneration test and the save read it |
| 0xE38 | 4, signed | Full time F | Set with T at each meal; the HUD meter's full value |

### Petpet runtime fields (conceptual)

| Field | Meaning |
| --- | --- |
| State | Current state, 0 to 5 (Behavior). The state is the first word of the petpet's per-pet block. |
| Owner | The hero the petpet belongs to. |
| Remaining time T and full time F | T counts down in ticks; a meal raises T and F together. They are not scores. |
| Pending target | The enemy target latched by a strike, applied when the strike lands. |
| Strike timer | Accumulated seconds on the pending target, for the fallback path. |
| Wait counter | Ticks left in the roam or hover wait. |

## Behavior

Times are in ticks at 60 ticks per second (SPEC-0007). Distances are world units. Distances to the hero (the 10-unit tests of the wild loop, the gait distances and idle points of the live petpet, and the hero checks of class 20) are measured on the horizontal plane, with the height difference zeroed. Every distance to a food item (the nearest-food search, the anchor test, the eat tests and the class 20 range) is three-dimensional, and "nearest" means the smallest three-dimensional distance (High).

### Acquisition (joining)

- A petpet joins the hero by being fed. Wild petpets are placed characters of the petpet kinds (monster rows 244, 245, 247, 250 and 251, behaviour class 33) in the world packs. The hero selects a food item and presses Square at a wild petpet (the "Feed petpet" prompt); the hero drops one of the item 2.8 units in front of itself (shown at model scale 0.45), reserved for the petpet and not collectable. The wild petpet walks to the food, eats it (animation 4) and becomes the hero's petpet: it is removed from the world and the live petpet is created at its place with its remaining time, then runs class 34. No host call, flag or item grant is involved. (High)
- Food items are item ids 145 to 157. The hero can feed 145, 146, 147, 149, 150, 152, 153, 155, 156 and 157. A petpet never eats 145 (Bagguss), which has value 0; ids 148, 151 and 154 cannot be fed. (High)
- On the main path, the first wild petpet is in Meridell Outskirts (pack 1066, pet type 450, kind 0). Others are in packs 1230, 758, 991, 1130, 1324, 934, 1250, 1257, 1353 and 1358 (High for the placements; Medium-High that they appear in play).
- While a petpet of type t is the live petpet, no placed character with pet type t is spawned (High).
- Script slots (High for the call sites; Medium that each call is reached in normal play): wild petpets in character regions that a level script loads with SlotLoad do appear: region 1215 of pack 1184 (type 2090, kind 7), region 1303 of pack 1300 (type 1314, kind 3), regions 1351 (types 703 and 1460) and 1352 (type 1460) of pack 1350, region 1360 of pack 1358 (type 703) and region 1416 of pack 1414 (type 1314). Packs 706, 800 and 886 also load regions that hold wild petpets, by the set numbers their scripts choose from story flags (a load of set j reads region pack + 1 + j): pack 706 loads set 1 (region 708, a type 450 copy) when flag 63 is the highest set of flags 63 to 67, set 2 (region 709) for 64 and set 5 (region 712) for 67; pack 800 loads set 1 (region 802, type 1460) when flag 55 is the highest set of flags 55 to 58 and set 2 (region 803, type 450) for 56; pack 886 loads set 3 (region 890, type 450) when flag 52 is set. Pack 753 loads set 2 (region 756, whose copy is a type 2335 creature of the second table) when flag 427 is set and flag 428 is clear; nothing sets flag 427 (no script, dialogue code or stage row; SPEC-0049, Who writes the flags), so that load does not run in play. Pack 851 never loads its regions either (852 to 854, type 703): its three loads sit in a routine nothing starts, and they use a slot word that no code stores (SPEC-0032) (High for the call sites; Medium that each gate is met in normal play).
- Return after leaving (Medium-High): the joining wild petpet is released from its block without its kill bit, so its record stays eligible, with its position set to where it ate. While a petpet of its type is the live petpet the record is not bound. Once that live petpet is gone (it left at T below 1, was replaced by feeding another wild petpet, or was removed at an act start), a streaming block binds the record again by its normal rules (within 280 units of the hero, nearest first, up to the block's cap; SPEC-0006), and the wild petpet is back at the spot where it ate, at full health, and can be fed again. A copy in a non-streaming slot comes back only when its slot is loaded again.
- Kind 8 (type 2094) placements run class 20, which never calls the activation, so a kind 8 petpet cannot join this way (High).
- A level script cannot create a petpet. The only pet host name is the query GetPlayerPetPetType (row 199), and no host call sets or spawns a pet (High). Every TriggerActStart call removes the live petpet (Area changes, resets and hero swaps).
- No item-use route operates on a petpet (High; open question 29 is resolved). Item 126 is the Tattered Cloak, a category-9 item with no use effect: the shared item-use dispatch refuses it without consuming it, while Square on the Items tab equips it. Feeding a wild petpet is a separate interaction that drops category-12 food. The only PetPet-specific host API is the read-only kind query, and no registered item-use callback calls it.
- A fresh game has no petpet. The pending pet type is -1 after the subsystem reset, so the script query returns -1 and every level branch that tests a pet kind is false until a petpet joins.
- No default save image exists on the disc. A fresh game starts from the empty slot that the code builds, which holds no pet type (Medium-High; SPEC-0025, empty-slot template).

### Per-tick loop

This loop and the State selection, States and Strike sections below are the class 33 loop (six states), the default class of the placed pet rows. The summoned petpet runs class 34 instead (see "Class 34 (summoned petpet controller)").

Each tick the petpet chooses a state from the current state (below), runs that state's rules, and then yields one tick.

### State selection

1. If the petpet is 10 or more units from its hero, it goes to roam (state 3), unless it is already in strike (4) or hover (5). Those two keep their state. (A test for state 2 is made first on this branch and always fails.) The fed flag is not read on this branch.
2. Otherwise (strictly within 10 units of the hero), in this order:
   - the food test runs. It reads and clears the fed flag every time it runs, whether or not it succeeds; only when the flag was set does it look at the nearest category-12 world item. It passes when that item exists, is not the anchor (type code 145), its position lies on the pet's navmesh (a point-location test) and it is less than 10 units from the petpet. On a pass the state becomes strike (4), or stays 4. So a feeding sends the pet to strike at most once, and a feeding whose food fails the test (for example the nearest item is the anchor) is lost. The hero's feed range is a little larger than 10 (10.5), so a pet fed while 10 to 10.5 units away keeps the flag until it first comes within 10;
   - else, if the nearest category-12 world item is the anchor (type code 145), lies on the pet's navmesh and is less than 5 units from the petpet, it goes to hover (5); entering hover from any other state clears the hover once-only mark. This test also takes the pet out of strike;
   - else it goes to idle-with-animation (1), unless it is already in state 1 or 4; entering state 1 sets the one-shot flag, so animation 9 plays once.

Strike (4) and hover (5) are never left by the 10-unit test: strike ends only through its own body (no food in the world sets state 0; eating ends in joining) or through the anchor test, and hover ends only through its own body (state 0) or by the selector sending the pet to strike. The selector is High for these rules. The roam threshold (10) selects the roam state; its target is generated around the pet's own home point rather than around the active hero (State 3, Roam).

### States

- **0, Idle.** Plays animation 0 each tick.
- **1, Idle with one-shot (High).** Each tick the petpet first turns toward the hero (the horizontal direction to the hero, blend weight 0.1, the shared aim rule). Then: when the one-shot flag is set (it starts set and is set again each time the selector enters this state), the flag is cleared and animation 9 is requested. Otherwise, if animation 9 is not the current clip, nothing more happens while the hero is within 10 units, and the state becomes 0 when the hero is not. Otherwise (animation 9 is playing), when it has finished animation 0 is requested. The state keeps its place until the selector changes it.
- **2, Idle.** Plays animation 0, as state 0. The selector never chooses state 2: its test for state 2 always returns false (High).
- **3, Roam.** Each tick the wait counter, when above zero, first falls by 1. If no roam point is held, one is picked (Roam point rule below); the pet also picks a first point with the same rule when it starts. The wait counter starts at 0, so the first walk begins at once. While the wait counter is still above zero after its decrement, play animation 0 and do nothing else. So after an arrival tick sets the wait to 60, the next 59 ticks idle and the walk request is made on the 60th tick after the arrival. When the wait ends, walk to the point with the shared goal-move routine (walk gait, animation 2). The walk ends when the horizontal distance to the point is below 2.5, when the body touched something, or when the point fails the navmesh-neighbourhood point test; then the point is cleared and the wait is set to exactly 60 ticks (1 s). The draw is made between the tick rate and itself, so it is always 60; the 300-tick value stored beside it is never used. (High for the point rule, the 60-tick wait and the exits.)

  **Roam point rule (High).** The picker takes the pet's home point (the placement position stored with the character; level-script moves do not change it) and a size of 40.0. It adds to X and to Z each an independent whole-number draw from 0 to 39 minus 20, so each offset is -20 to 19 with equal chance; Y is the home point's Y. The result is a square of side 40 around the home point, not a circle of radius 40. The candidate must lie on the pet's navmesh (a point-location test). If it does not, the picker retries up to 10 times; each retry moves the candidate half of the way from its position toward the pet's current position and tests again. If the 10th retry still fails, the home point itself is the result. The same picker serves class 20's wander (State 3 of class 20). The "reference object" that the start-up code looks up from the character's configured id is only the pet's navmesh polygon attachment; it is not a roam centre. Test vector: home point (100, 5, 200), draws 39 and 0: candidate (119, 5, 180); if it fails the test and the pet stands at (110, 5, 190), the first retry candidate is (114.5, 5, 185).
- **4, Strike.** See "Strike" below.
- **5, Hover (High for the point, the mark and the exit; Medium-High for the gait).** The state looks up the nearest category-12 world item (the same search as the strike); when there is none the state returns to 0 at once. While its once-only mark is clear and that item is an anchor (type 145), the petpet chooses one goal point: the anchor's position plus 10.0 units along the unit vector from the anchor toward the petpet's current position (all three axes), stores it as its goal and sets the mark. The mark is cleared only when the selector enters hover from another state, so the goal is not recomputed while the pet stays in hover. If the nearest item is not an anchor, or the mark is already set, the goal stays what it was. Each tick the petpet then moves toward the goal while its three-dimensional distance to the goal is 2.0 or more, using the third gait word of the wild petpet (the run clip, animation 3, not the walk clip used by roam and strike); below 2.0 the state returns to 0 and the selector chooses again. Test vector: anchor at (0, 0, 0), petpet at (3, 0, 4): goal (6, 0, 8), reached when the petpet is within 2.0 of it.

### Strike

Confidence: High for the steps, the eat tests and the reservation rule.

There is one shared food reservation: a reserved food index (or -1), a timer in seconds and a slow flag. Reserving a food sets the index, and sets the timer to 0 and the slow flag to 0.

1. Each tick in state 4 the petpet finds the nearest world item of item category 12. If there is none, the state becomes 0. If it is the Bagguss (145), nothing more happens that tick and the state stays 4. Otherwise it reserves that food (so the timer is reset every tick while the step runs). A wild petpet then also needs the item's position to pass the navmesh point-location test and does nothing more this tick if it fails (the reservation stays made); the live petpet has no such test. Anchors are never eaten.
2. Wild petpet (class 33), eat test: it eats when it faces the food (dot of its facing with the unit direction to the food above 0.8) and the food is less than 2.0 units away. Its timer test (more than 2.5 s) reads the timer after the reservation of the same tick, so it is 0 and never passes. Otherwise, when the food is 2.0 or more units away the petpet paths toward it (only when it can reach it); when it is nearer but not facing it, it turns toward it (turn rate 0.1).
3. Live petpet (class 34 state 4), eat test: it eats when the food is less than 2.0 units away or the timer is below 2.5 s. Its timer is read before the reservation of the same tick, so it holds at most one tick's step and the test passes: the live petpet eats at once wherever it stands. Its walk branch toward the food (turn weight 0.3 with animation 2) can run only on a tick where a reservation made by another caller has aged 2.5 seconds with the food 2.0 or more away (SPEC-0038 Behavior 33), which does not happen in play. State 5 (leave) turns toward the camera's reverse direction with weight 0.1 and plays animation 3 until it is out of the camera cone or touches something, then deactivates. The search has no range limit: the live petpet takes the nearest category-12 world item to itself, which need not be the item the hero dropped (see "Petpet build guide", G7).
4. Eating: the petpet requests animation 4; when the clip has ended, the food's value is applied (Hit value and score), the food is removed and the reservation is cleared (-1). A wild petpet then calls the activation by its own id and becomes the hero's live petpet (Activation by id, below). A live petpet also clears its low-time latch.
5. Fallback timer (main loop, once per tick, while a food is reserved): the timer grows by the tick length 1/60 s, or by 1/180 s while the slow flag is set (a class 20 creature sets it in its object-seek step; a reservation clears it). When it passes 5.0 s: the timer and slow flag are cleared, the reserved item is taken from the item pool of the current world and of the other loaded world (the same slot is tried in both), and the reservation is cleared. When an item was taken and a live petpet with T at least 1 exists, it gets the taken item's value with its own kind multiplier as in step 4. Because every eating step re-reserves the food each tick, the fallback only runs after 5 s in which no petpet ran its eating step, for example food dropped with no petpet going for it. A wild petpet in its eat clip cannot lose the food to the fallback.

### Hit value and score

The targets of state 4 are food items: the type codes are item ids 145 to 157 (SPEC-0021), the anchor 145 is the Bagguss, which is never eaten, and a wild petpet enters state 4 only after it has been fed (the fed flag is set) and the nearest category-12 world pickup other than 145 is within 10 units and passes the shared spatial-sector/polygon validity check (High for these conditions; Medium for the interpretation as reachability). This check is a point location only: no line or ray between pet and food is tested, and a failed check does not retry a farther candidate (open question 46; High). The hit value is the food's value in minutes, and the score candidate is the remaining time it gives, in ticks.

For a target of type code t and a petpet of kind k:

1. base = the base value of t from the table above. For t = 145 (anchor), the hit is 0.
2. multiplier = the kind multiplier from the table above. Kinds without a rule use 1.0.
3. hit value = base x multiplier.
4. score candidate = floor(hit value x 3600). The factor 3600 is the tick rate (the run-time value 60 of SPEC-0007) times 60 seconds per minute, because hit values are in minutes (High; open question 2a).
5. Time update: eating food of value v (the base value of the food id times the kind multiplier, in minutes) sets T and F to trunc(v x 60 x 60) when that is larger than T (High). Times are integers. A candidate of 0 never replaces T.

T then falls by 1 per tick (Class 34), so T and F differ in play: F is the full time of the last meal, and the HUD petpet meter is round(60 x T / F) pixels (SPEC-0023) (High). The field is shared with other creatures (open question 9).

### Spawn

- A fresh spawn (the activation's id argument is -1) places the petpet 7 units ahead of the hero, along the hero's facing direction (Medium).
- Spawn heading (Medium): the hero's facing is given to the petpet's body axes for the spawn tick only. The stored facing that every petpet turn blends from is the heading of the petpet's own placement record, which is +Z (heading 0) in all five live kinds' packages, so from the first motion tick the body faces +Z and the Class 34 turns blend from there (SPEC-0038, Spawn facing and what overrides it).
- A joining spawn (the activation's id argument names a wild petpet) takes the wild petpet's state, placement and remaining time (into both T and F), then removes the wild petpet from its world list (High).
- Both kinds of spawn play animation 6. The value 0x22 passed on spawn is not a creature flag: it is the class number 34 (0x22 = 34) of the class request below (High; open question 2).
- The spawn then requests behaviour class 34 for the petpet at priority 2 (High). The fresh-spawn position, 7.0 units in front of the hero, is used when no previous pet state is held (High).
- A fresh spawn copies the pending T and pending F into the new petpet's T and F. It then clears the pending values: pending type becomes -1 and both pending times become 0. Pending values are one-shot (Medium).
- A second asset-load callback exists, but no code registers it. A second petpet slot is therefore never loaded (High).

### Activation and restore

Activation first checks the pet manager's asset loader. If a load is already busy, activation does nothing.

- **Activation by id (joining).** Finds the character with the given id in the world's character lists. If there is none, nothing happens. Otherwise it takes the type from the found character's record and maps it to a kind (no kind: nothing spawns). It then deactivates the current petpet, loads the package, and spawns as a joining spawn, not a fresh one. Its only callers are the wild petpet that has eaten (with its own id) and the scene-start restore (High).
- **Activation by type (fresh).** Maps the pet type id to a kind (primary table). If the type has no kind, nothing spawns. The spawn is fresh (id -1).
- Both forms deactivate the current petpet (if any), then load the kind's package (archive resource 534 plus the kind). When the package is ready, the petpet is spawned.
- **Deactivate.** Removes the petpet from its owner, releases its load record and clears the live petpet.

### Scene start restore

When a scene starts, the pet subsystem checks the pending load flag (+0x40). The flag is cleared at scene start.

- If the flag is clear, nothing happens to the petpet.
- If the flag is set, the routine first puts the selected hero at the saved position and facing and clears them (the load's apply step; SPEC-0055; Open question 28). Then it reads the pending pet type, which is also the third value passed to the activation call (High):
  - if the pending type is below 0 (signed 32-bit), the current petpet is deactivated;
  - otherwise the petpet is activated by type with the pending type as its type value and -1 as its id value. The spawn is fresh, so the pending T and F are copied and the pending values are cleared (High for the argument order; Medium for the rest).

Pending values stay as loaded when the no-pet path runs (they are not cleared).

### Save

The save writes the live petpet's type id at 0x214, T at 0x218 and F at 0x21C. With no petpet, or T below 1, the type is written as -1 (0xFFFFFFFF). On load a type of -1 removes any petpet at scene start; a valid type spawns it 7 units in front of the hero with T and F from the slot. (High)

### Area changes, resets and hero swaps

- Level or game teardown and mode switches deactivate the petpet (Medium).
- Every TriggerActStart call and the world reset remove the live petpet (High).
- No helper removes the petpet for being far from the hero. The routine once described that way is the class 34 leave state: it removes the petpet when the petpet is outside the camera's view cone or touches something, and only after T has fallen below 1 (Class 34, state 5) (High).
- A hero swap or reset does not move or respawn the petpet: the class 34 controller reads the selected hero every tick, so from the next tick it follows the incoming hero. A switch to another kind is a deactivate followed by an activation; there is no separate switch path (Medium).

### Script query

GetPlayerPetPetType (SPEC-0030, row 199) returns the kind of the active petpet. It returns -1 when there is no active petpet, and when the type is not in the primary table (for example a second-table id such as 805 or 2335). It applies the presence gate: it returns -1 when the petpet's remaining time T is below 1 (High). Two level scripts read the kind to choose camera, conversation or message lines (Medium). Both test kinds 1, 3, 6 and 7 (one also tests kind 0); they see -1 until a petpet has joined (Medium).

### Level-script pet branches

These branches are in the level scripts (SPEC-0045 owns the scripts). They read the pet-kind query above and choose a line or a scene. They are the only in-game readers of the kind that were found.

Cinthia conversation (level pack 758):
- Flag 540 set (her request done): every visit shows line 0 of the pack's text table (the thank-you for the new petpet). Nothing changes (High).
- Flag 540 clear and flag 539 clear (the first visit): flag 539 is set and line 12 is shown (her "I am lonely, I want a petpet" request). It does not read the pet kind (High).
- On later visits, while flag 539 is set and flag 540 is clear, the conversation reads the active pet kind, tested in this order (High):
  - Kind 6 sets flag 540 and shows line 1 (the thank-you with the gift). This completes her request.
  - Kind 0 shows line 2, kind 1 line 3, kind 3 line 4 and kind 7 line 5 (each a refusal). No flag changes.
  - Any other value, including -1 when no pet is active, gives a waiting line that rotates with a counter shared by both heroes: hero 0 (Tor) shows lines 6, 7, 8 in turn and then again 6; hero 1 (Roberta) shows lines 9, 10, 11 in turn and then again 9. Starting from the stored counter 0, the sequence is first, second, third; the counter is cleared after the third line. Switching hero mid-sequence continues the same counter (so Roberta may start on 10 after Tor's first line). The counter is also reset to 0 by the third line only. Test vector: Tor, no pet, three visits after the first: lines 6, 7, 8; a fourth: 6.
- With no petpet the query returns -1, so the kind 0, 1, 3, 6 and 7 branches run only after a petpet has joined; a kind 6 petpet joins from a wild type 1460 petpet (Acquisition) (Medium; open question 30).

Trigger room (level pack 1300):
- When the trigger fires with flag 594 clear and the active pet kind 3, the scene hides one scene object, shows another, plays a hero animation and a screen fade, moves the hero, and runs a camera move.
- It needs a live kind-3 petpet (a wild type 1314 petpet that has joined). The same pack's per-tick update sets flag 594: while the ritual word is set it fades a scene object in over 5.0 s (colour multiplier from 0 to 255 by the system clock) and sets flag 594 when the value reaches 255 (High; SPEC-0045, pack 1300). No other writer of 594 exists in the level scripts.
- Confidence: High for the branch structure; Low for the meaning of the scene.

### Placed pet copies

A pet type record can also appear as a character placement in level packs. Such a placement spawns as an ordinary character through the character setup, which looks up its type record by the placement's type fields. That spawn does not set the live petpet, does not attach the character to the pet manager, and is not counted by the script query (High for the spawn path). Such a placed copy is a wild petpet: when it is fed and eats, it becomes the live petpet (Acquisition (joining)). The placed pet rows 244, 245, 247, 250 and 251 have class 33 as their default class, so such a placed character runs the class 33 loop unless something requests another class for it (High for the row default).

### Class 34 (summoned petpet controller)

Confidence: High for the request, the seven-state structure and the remaining-time countdown; Medium for the state meanings and bodies.

The summoned petpet is spawned 7.0 units in front of the hero (when no previous pet state is held), plays animation 6, and is requested into class 34 at priority 2. Class 34 keeps the remaining-time counter T (entity offset 0xE34). While the game-state code (SPEC-0026) is 0, T falls by 1 each tick. Codes 1 to 8 hold it; code 9 stops the pet's update (High). The petpet counts as present (for the passive bonuses, the script query, the save and the HUD meter) only while T is at least 1 (High). The low-time mark is the word 0x2DB1D8 (0.5 minute) x 60 x 60 = 1,800 ticks.

States are chosen each tick, in this order:

0. If the pet is in state 3 (bumped), nothing is chosen: state 3 is left only by its own recovery (High).
1. If the pet's body touched something on the previous tick's motion step (the motion record's touched flag) and the pet is not leaving: state 3 (bumped). A touched pet that is leaving (state 5) keeps state 5, and nothing else is chosen that tick (the pending-event word is not read).
2. Else, if the pet's pending-event word is set: state 4, and the word is cleared.
3. Else, if the pet is not in state 4 or 6:
   - when T is at least the low-time mark, or the low-time state has already run since the last meal: if T < 1, state 5 (leave); otherwise state 1 (follow, with a new target near the hero) when the hero is idle, else state 2;
   - when T is below the low-time mark and the low-time state has not run since the last meal: state 6, once.

State bodies (Medium):

- **0**: plays animation 0.
- **1, idle-hero follow and nearby-character response:** the pet scans character entries in order for the first eligible character within 10.0 horizontal units whose id differs from its last recorded response. On accepting a new character it records that id and requests animation 8; while animation 8 is current it waits, and after the clip finishes it requests animation 0. The eligibility filter is the test that the character's class state is 19, the agro state of SPEC-0038, so the pet reacts only to enemies that are fighting (High). When no new character response takes priority, the pet moves toward its stored point while horizontal distance is at least 1.0, aiming toward it with weight 0.1 and requesting animation 2; this also clears the wait timer. Below 1.0, an unset timer is drawn from 180 through 600 ticks and animation 0 is requested. On following updates the timer decreases by one; when it drops below 1, the pet chooses a new point and unsets the timer. Each new point is the hero's current position plus a world-X offset and a world-Z offset (not hero-relative): the X magnitude is drawn first (uniform from 3.0 to 10.0), then its sign (a 0-or-1 draw; 1 makes it negative), then the Z magnitude and the Z sign the same way (High). During an active wait, animation selection uses r = T / 1,800: if r < 1, request animation 7; otherwise draw integer q from 0 through 100 and request animation 1 when q > 200 × |1 − r|, or animation 6 otherwise. This choice is made and requested on every tick of the wait; a second rest timer of 1.5 s in the same routine can never start, because its counter is only ever set to 0 (High; see "Petpet build guide", G5). T is decremented after the state body on that tick. (High for the thresholds and update order; Medium-High for the candidate-filter and random-target details.)
- **2, moving follow:** chosen instead of state 1 while the hero is in the movement/jump predicate. The target is the hero's position plus 4.0 units along the hero's forward axis B3 and 1.5 units along the hero's side axis B1 (B1 = normalise(up × forward); for a hero facing +Z it is +X; SPEC-0040, Behavior 14C) (High). The gait word starts at 1 when the controller starts and is not reset when state 2 is entered. The pet measures horizontal distance to this point and uses a three-gait hysteresis rule. Gait 0 uses animation 0 at distance 3.0 or less; above 3.0 it uses animation 2 and changes to gait 1. Gait 1 uses animation 3 and changes to gait 2 above 7.0; below 1.5 it uses animation 0 and returns to gait 0; from 1.5 through 7.0 it uses animation 2 and stays in gait 1. Gait 2 uses animation 2 and returns to gait 1 below 5.0; at 5.0 or more it uses animation 3 and stays in gait 2. Gaits 1 and 2 also aim toward the target with weight 0.1 each tick; gait 0 does not run this aim update. (High for the distances, transitions, aim call and axes.)
- **3, bumped:** requests animation 0 each tick. The state remains 3 unless both recovery gates pass: the dot product between the camera's view vector and the normalized direction from the camera position to the pet is below 0.5, and the ground height at a candidate point is less than 2.0 units from the hero's current height. The candidate point is the hero's position plus 10 units along the normalized horizontal negative camera-view vector; the ground query samples from 8 units above that point. On success the pet is placed at the candidate (at the hero's height, not the ground height; gravity and the ground constraint settle it) facing horizontally toward the hero, and state 1 is selected directly, so the idle point stored before the bump is kept. The camera vector is the camera's forward row (camera record +0x20, the look direction from the eye toward the focus) and the camera position is its base point (+0x30; SPEC-0027). So the first gate means "the pet is more than 60 degrees off the camera's line of sight", and the candidate lies 10 units from the hero toward the camera. This is the only rule that brings a lagging petpet back to the hero: a bumped petpet stands still while it is in view and reappears near the hero once it is out of view (High).
- **4**: entered when the pending-event word is set; the petpet eats the nearest food at once, where it stands (Strike, step 3); eating here resets the low-time state.
- **5, leave** (High): each tick, the pet is removed (deactivated) when it is more than 60 degrees off the camera's line of sight (the same camera dot test as state 3: dot below 0.5) or when its body touched something on the previous motion step. Otherwise it turns toward the horizontal negative camera-forward direction (toward the viewer) with weight 0.1 and requests animation 3 (run), so it runs out of the picture toward the camera. It does not test the hero or its own facing. An older reading ("removed when it faces away from the hero") was wrong.
- **6, low time**: requests animation 7 until that clip has finished, then sets state 0; the pet stands still meanwhile (High).

So the petpet stays until T falls below 1 and then leaves (state 5, removed). The first time T drops below 1,800 after a meal, it plays animation 7 once (High). The latch for this starts clear, so a petpet created with T already below 1,800 (a loaded save) plays it on its first update; eating clears the latch even when the meal does not raise T (High).

### Class 20 (second petpet behaviour)

Class 20 is a second petpet behaviour. It has four state values: 0 (idle), 1 (hero reaction), 2 (approach and consume a nearby object) and 3 (wander). Each tick the state is chosen first, then the chosen state's rules run. Confidence: High for the states and the choice rules; Medium for the effects.

Choice rules (each tick):
- A state is re-chosen only while it is 0, 2 or 3. State 1 runs to its end.
- If the item test passes, the state becomes 1 and the sub-step becomes 0.
- Else, if the state is not 2 and the nearest category-12 world item (food) exists, is not the anchor (145), lies on the pet's navmesh (point-location test) and is less than 25 units away (three-dimensional), the state becomes 2 and the latch is cleared (High).
- Else, if the state is not 3, the state becomes 3 and the sub-step becomes 1.
- While the state is 2, the object rule does not restart it; only the state 2 routine ends the state (Medium).

Item test (Medium-High for the slot behavior, Medium for the second check): reads three checks on the hero. The first is the hero's variant-specific attack-state query. The second calls a method that returns the attack-active word (the event bit 0x01) from the receiver adjusted for that method; the call does not prove that this adjusted receiver is the selected hero object itself. The third is the shared reach/contact test, which first applies a 65.0-unit range gate and then checks character bone shapes. If all three checks are non-zero, a short message is shown at the hero's position and the test passes. Otherwise it scans the hero's six shot slots. An armed slot supplies a shot position and radius; if the pet's shape overlaps that shot sphere, the slot is disarmed and the test passes. The exact owner of the adjusted second-check receiver is not known; it does not matter, because these objects exist nowhere in play (Default if unresolved (rewrite choice): treat the second check as an opaque yes-or-no).

State 0 (idle): plays animation 0 each tick.

State 1 (hero reaction): three sub-steps.
- Sub-step 0: plays animation 14 once. When it finishes, draws a wait of 3 to 4 seconds at random (180 to 240 ticks).
- Sub-step 1: plays animation 15 for the wait, one count per tick. Then goes to sub-step 2.
- Sub-step 2: plays animation 16 once. The hero check is made once, on the first tick of animation 16, and the state's latch is set whether or not the check passes. The check passes when all three hold: the horizontal distance between the pet's and the hero's positions (their feet or origins, with the height difference zeroed, not a three-dimensional distance) is below 2.5 units; the absolute height difference between the hero and the pet is more than 0.5 units (either side counts); and the hero's action state is not 7. (From the code, not run in the original; High for the horizontal distance and the constants, Medium-High for the practical result.) So a hero touching the body is always inside 2.5, and a hero standing on the body or beside it passes the height test whenever his feet are more than 0.5 above or below the creature's origin. In play the hero is launched when he is in a moving jump (states 8 to 10, or a fall that is not state 7) or stands on ground differing by more than 0.5 in height when the animation starts; a standing jump (state 7) fails the state test. The original has no rule of "within 1.0 of the top, stand there": nothing makes the hero stand on this creature (SPEC-0040, the character push rule). On a pass the creature launches the hero: 8.0 is added to the vertical (Y) lane of the hero's velocity vector (units per 1/30 s step), the hero's action state is set to 7 (the standing jump), and a one-tick hold flag is set that makes the next hero pass skip its usual velocity reset and clear the flag instead (High). When animation 16 finishes, the state becomes 0.
- During sub-steps 1 and 2 the creature queues two contact shapes into the hero's pending contact shapes (SPEC-0040, Behavior 14A, item 2a). Open: what the hero's mover does with them is not known (Low-Medium).
- Body (from the code, not run in the original; High for the constants, Medium-High for "never changed"): radius 1.175, height 3.5 and weight 2.0, written once at placement and never changed by any state, including the shell state. The only body change is the collidable word: the hero-reaction routine, called every tick in sub-steps 1 and 2 (not sub-step 0), clears it, and class 20 never sets it again, so from the end of animation 14 the creature no longer pushes the hero and he passes through it, until the record is reset by re-placement. Before that it is collidable like any region character (SPEC-0040, the character push rule) and is not hidden, since it plays only animations 0, 4, 14, 15 and 16. Medium-High that no other writer restores the word.

State 2 (approach and consume): each tick, while the pet is not yet latched:
- Raises the shared slow flag on every tick the state runs, including ticks that do nothing else (High).
- Finds the nearest category-12 world item, with no range limit (an anchor, if it is the nearest, ends the tick with no action; see the edge cases). If there is none, or the item's position is not on the pet's navmesh (point-location test), the state becomes 3 with sub-step 0 and a wait of 4 to 5 seconds (High).
- If the object is 2.5 or more units away, the pet moves toward it and stays in state 2.
- If it is within 2.5 units, the pet latches and takes the object's type code. The matching world-pickup record has its attached resources detached and released, then is reset; this is active-object cleanup, not a visibility-only change. The pool record itself is not shown to be deallocated, and its later reuse is open (Medium-High for cleanup; Low for slot lifecycle). It stays in state 2 that tick.
- Once latched, the pet plays animation 4; when that finishes, the state becomes 3 with sub-step 0 and a wait of 4 to 5 seconds at random.

State 3 (wander): sub-step 0 plays animation 0 and counts the wait down; at 0 it picks a random point by the Roam point rule (class 33, State 3; a square of side 40 around the pet's home point) and sets sub-step 1. Sub-step 1 walks to the point; within 5.0 units it draws a new wait of 4 to 5 seconds and returns to sub-step 0. The pet's navmesh polygon attachment is set at start-up from its configured id (High).

Constants (seconds at 60 ticks per second): state 1 first wait 3.0 to 4.0 (3.0 at virtual address 0x2DB290, 4.0 at 0x2DB294); hero effect nudge 8.0 (0x2DB298); the wait after state 2 and each state 3 wait 4.0 to 5.0 (0x2DB2AC and 0x2DB2B0). Other values are fixed constants: 25 units (food range for choosing state 2), 2.5 units (latch and hero check distance), 0.5 units (height separation), 40 units (wander square side), 5.0 units (wander arrival). The class 20 values are taken from the disc; the constants are as the behaviour uses them.

Edge cases:
- A state entered at sub-step 1 (from the choice rules) walks to the point stored in the state record before it picks a new one. The stored point starts as the pet's home point, so a first wander entered at sub-step 1 walks to the home point (High; open question 21).
- State 2 with an anchor as the nearest candidate (High): the tick does nothing at all, apart from raising the slow flag. The pet stays in state 2, requests no animation, makes no move and consumes nothing, so it keeps whatever it was playing. This repeats every tick until the nearest object is no longer an anchor; the state chooser never takes the pet out of state 2 by itself. Nothing is consumed from beyond the anchor, however close.
- The item test reads the hero's six shape slots; a slot holding a shape that the pet overlaps is cleared, so the item test can consume a hero-side shape (Medium-High; open question 18).

### Passive bonuses to the hero

Confidence: High for the kind tests, the constants, the per-tick test and the hero products that the two damage factors enter (SPEC-0036).

The kind is the kind of the live petpet through the primary table (Layout). With no live petpet there is no kind and none of these bonuses applies. Each bonus tests the kind first:

| Kinds | Bonus | Constant |
| --- | --- | --- |
| 0 or 7 | A hero damage product (the product of the hero's attack factors) is multiplied by 1 + 0.25 = 1.25; otherwise by 1.0 | 0.25 at 0x2DD8DC |
| 1 or 7 | The hero's damage-intake product is multiplied by 1 - 0.1 = 0.9; otherwise by 1.0. Single precision: 1 minus the 0.1 word gives 0.89999998, and that product is rounded to nearest, so 20 times it gives 18.0 and 10 times it gives 9.0 (the product is not 17.999998). Only the final conversion to a whole number truncates. | 0.1 at 0x2DD8E0 |
| 6 or 7 | +1 hit point, capped at the hero's maximum hit points, on each hero update where the regeneration test below passes | 1.0 at 0x2DD8E4 |
| 3 or 7 | +1 to the hero's second pool (magic; SPEC-0036), capped at its maximum, on each hero update where the regeneration test below passes | 1.0 at 0x2DD8E8 |

Kind 7 has all four bonuses. A kind 8 petpet has none while it is live, but it saves as 2090 and loads back as kind 7 (Layout, kind 8 note), so after a reload it has all four.

Relation to SPEC-0036 (High): the 0.25 word is the word behind the factor E of SPEC-0036 (Damage), so "the global bonus condition" there is "the live petpet's kind is 0 or 7". The 0.1 word is the word behind SPEC-0036's 0.9 factor, which applies to enemy hits taken by the hero, after the shield block and the armour terms (SPEC-0036, Enemy attack damage to a hero). It does not apply to the hero's outgoing damage.

Regeneration test (High). Let s be the live petpet's remaining time T (entity offset 0xE34; Layout, Petpet entity fields) and n = trunc(60 x c), where 60 is the tick rate and c is the constant of the bonus (1.0, so n = 60). The test passes when T is at least 1, the kind test passes and the integer remainder of s divided by n is 0. The bonus goes to the hero whose update runs, which is only the selected hero (SPEC-0053). The 1.25 and 0.9 factors also need T at least 1 (High).

Order within the hero update (once per tick):
1. Hit points: the petpet point (0 or 1) is added to the item-based hit-point regeneration of the same tick (SPEC-0036, Regeneration); if the sum is above 0, it is added to the hit points and the result is capped at the maximum.
2. Magic: the same with the petpet magic point and the item-based magic regeneration.

The item-based hit-point regeneration has gates (the hero must be alive and not blocked; SPEC-0036). The petpet point has no distance, alive or poison gate; it is added to the item trickle of that tick before the cap (High). The point is added in the death state too, so a downed hero's health can rise while the petpet stays, but the death state has no health exit, so the hero does not get up from this (Medium; SPEC-0055 owns the death state). The petpet stays during the hero's death state (High): nothing on the death path removes it (it is removed only by its own leave state, a new activation, an act start, the world reset and the scene-start no-pet restore), and its state choice has no hero health or death test. It keeps following by the usual hero idle test (state 1 or 2), and T keeps falling while the game-state code is 0.

Heal rate (High). Because T falls by 1 per tick in game-state code 0, the test passes once in every 60 ticks: a kind 6 or 7 petpet adds 1 hit point per second, and a kind 3 or 7 petpet 1 magic point per second, each capped at the maximum.

Quirk (reproduce; Medium-High): while the game-state code is 1 to 8 (for example during a conversation, code 3) T does not change, so if it is a multiple of 60 the point is added on every tick of that stretch.

Other uses of the remaining-time field outside the petpet are open (open question 9).

### Feeding a petpet (hero state 49)

Hero state 49 is feeding a petpet (Medium-High): the Square prompt shows the feed-petpet label (label 14 of the UI label table, design table 13; SPEC-0022) when the hero's own petpet is within 2.5 units (4.5 while a timer is below 45 s), another petpet is within 4.5, or a class-20 creature (Class 20) is within 25, in front of the hero (dot above 0.5); Square with a food item enters state 49 (without one, cue 162 plays); the hero requests 138 (0x8A) and, at progress 0.99 (word 0x2DD854), sets the fed flag on the nearest of its own petpet within 10.5, another character within 10.5 or a class-20 creature within 25, plays an effect with the value 0.45 and the record's 16-bit id, and returns to state 1. The hero drops one of the selected food item 2.8 units in front of itself (shown at model scale 0.45) as a world item that cannot be collected, reserved for the petpet, and one is removed from the inventory (High). Feeding the own petpet raises its T as in Hit value and score. Feeding another wild petpet while one is live sets the live petpet's T to -1, so it leaves; the fed one joins when it has eaten (High).

### Statue puzzle objects (not companions; never present in play)

Confidence: High that these objects are not petpets or companions, that none is ever created in play, and for the rules below; Medium for the model they draw.

The statue puzzle-object pool is never filled in play. Its four-state records (states 1, 2 and 5 with a turn-to-align routine) can be mistaken for hero-attached companions, but they are the world's built-in puzzle objects: a pool of 30 records of 336 bytes (SPEC-0028, section 15.1; the hero's statue-turn state is in SPEC-0033). They have no connection to the petpet kind table, the save fields, the petpet behaviour classes or the hero's active petpet. The numbers 1, 2 and 5 are the kind word of a puzzle object (kind 1 is a statue); they are not petpet states. A clean implementation can omit this section entirely.

**Who creates them: nothing.** A record can enter the pool only through two add routines. No boot code calls either one, no level module calls or points to either one, and no data table holds their addresses. The pool is cleared at every world reset, so every slot stays free. Consequences in play (High): the per-tick update never runs on a record, the drawing pass never draws one, the hero's statue offer (Square, label 23) never finds one, and the completion count below never reaches four.

If a faithful port wants the dormant behaviour anyway, it is this.
- **Update order.** Each tick, for every used slot: the fall step (when its fall flag is set), then the kind step, then the render copy. The render copy always runs: the three basis vectors and the position are copied into the record's draw matrix.
- **Fall step.** The fall flag is set (with vertical speed 0) only by a start routine that nothing calls. While it is set, each tick first subtracts G x 0.008333334 from the vertical speed, with G the gravity of the physics block (32.0, word 0x3A1A7C), that is 0.26667 per tick. Then the ground height under the record is read with a probe from 8 units above the record. If that height is lower than the record's y, the speed is added to y. Otherwise y becomes the ground height and the fall flag clears. Test vector: speed 0, y = 10, ground far below: after ticks 1, 2, 3 and 4 the speed is -0.26667, -0.53333, -0.8 and -1.06667, and y is 9.73333, 9.2, 8.4 and 7.33333.
- **Kind 1, statue (High).** Each tick the unit direction from the record's position to the world origin is compared with the record's side vector by dot product. At 0.99 or above the aligned flag is 1, otherwise 0. The hero's facing is not read. The statue turn of SPEC-0033 rebuilds the side vector from the turned forward direction. Test vectors: position (0, 0, 10) with side vector (0, 0, -1): dot 1.0, aligned. Position (10, 0, 0) with side vector (0, 0, 1): dot 0, not aligned. Position (10, 0, 0) with side vector (-0.995, 0, 0.0999): dot 0.995, aligned. A record exactly at the origin has no defined direction.
- **Completion count (High).** After the kind step of every slot, the number of used slots that are kind 1 with the aligned flag set is counted. If level state flag 1 is clear and the count is exactly 4, a completion routine runs; its body is empty, so nothing happens. The statue puzzle of the levels is scripted instead (SPEC-0041).
- **Kind 2, turn toward +X (High for the target and step; Medium for the side of the up axis the rebuilt side vector points).** The target is the fixed world direction +X, (1, 0, 0). On every tick the aligned flag is set to 1 if it was 0. Then, while the dot product of the forward direction with +X (its x component) is below 0.99, the new forward direction is normalize(0.99 x forward + 0.01 x (1, 0, 0)); the up vector is set to the world up (0, 1, 0), and the side vector is rebuilt as the normalized cross product of the new forward direction and up, with a zero fourth lane. At 0.99 or above nothing is turned. The position never changes. Test vector: forward (0, 0, 1): after one tick it is (0.0101, 0, 0.99995); the dot product with +X first reaches 0.99 after 263 turning ticks, when the forward direction is (0.99000, 0, 0.14106), and from the next tick on it stays there.
- **Kind 5, homing strike on the hero (High for the flight and the hit; Medium for the hit arguments).** The record has a tick counter, starting at 0. Each tick the counter is incremented first. While it is below 200: the target point is the active hero's position plus 1.5 units along +Y; the direction to it from the record's position is normalized, and its length is kept. When the dot product of the current forward direction with that direction is above 0.5 and below 0.99, the new forward direction is normalize(0.9 x forward + 0.1 x direction), the up vector is set to the world up and the side vector is rebuilt as for kind 2. Then the position advances 0.25 units along the forward direction (the new one when it was blended). If the length measured at the start of the tick was below 0.5, the hero takes a hit (damage 1, attacker -1, blocked flag 0, the same take-hit call as other contact hits; SPEC-0036) and the counter is set to 200. On any tick where the counter is 200 or more, the aligned flag is set to 1 and nothing moves. So a kind 5 record flies for at most 199 ticks (at most 49.75 units) or until it ends a tick within 0.5 of the target point, hits the hero once, and then stops. Test vectors: record at (0, 0, 0) facing +X, hero at (3, 0, 0): the target point is (3, 1.5, 0), the length 3.3541 and the direction (0.8944, 0.4472, 0). The dot product 0.8944 is inside the blend interval, so the new forward direction is (0.9990, 0.0452, 0) and the position after the tick is (0.2497, 0.0113, 0). With the target direction +Y (dot 0) there is no blend: the record advances 0.25 along +X. A record 0.4 from the target point at the start of a tick hits the hero for 1, ends the tick 0.25 closer along its forward direction, and on the next tick sets the aligned flag and stays.
- **Other kinds (0, 3, 4 and above 5):** no kind behaviour; the record only falls (if its fall flag is set) and copies its basis.
- **Drawing (Medium).** The world draw pass draws every used record whose position is within 400 units of the view point (squared distance below 160000) as one model through the ordinary model path. The add routine writes no model index of its own, and since nothing creates a record (above) nothing is ever drawn: no model is needed in play (High).

## Edge cases and error handling

- A petpet whose remaining time T is below 1 reads as "no pet" to the script query, the passive bonuses, the save and the HUD meter (High).
- A save made with no pet, or with T below 1, stores -1 in the type field (High). Loading that value removes any petpet at scene start. See "Save".
- A saved type that appears only in the second table (805, 1350, 1368 or 2335) gives no kind on load, so nothing spawns (Medium).
- A kind 8 petpet saves as 2090 and loads back as kind 7 (Medium; open question 10).
- Pending values are one-shot. The first spawn that takes them clears them. A no-pet load leaves them as loaded.
- A kind outside the tables (for example pet type 4660) gives no kind. Activation by type then does nothing.
- Anchor type 145 is never a strike target and always gives a hit of 0.
- The strike's fallback timer (Strike, step 5) grows at one third of the normal rate while the shared slow flag is set; only a class 20 creature sets it, and each new reservation clears it (High).
- Feeding another wild petpet while one is live sets the live petpet's T to -1; it leaves, and the script query returns -1 until the fed one joins (High).
- A wild petpet that has eaten is removed from its world list and the live petpet is created at its place (Acquisition (joining)).
- Class 20: a state entered at sub-step 1 walks to a stored point before picking a new one (see the class 20 section; open question 21).
- Class 20: an anchor as the nearest object keeps the pet in state 2 with no action each tick (High; State 3 and State 2 notes above).
- Class 20: the item test's slot clearing consumes a hero-side shape (Medium-High; open question 18).
- Passive bonuses: the petpet hit point is not gated on the hero being alive or unblocked, so it is added on ticks where the item-based regeneration is gated off (Medium; reproduce).
- Passive bonuses: with no live petpet, or T below 1, there is no kind and nothing is added.
- Passive bonuses: a reloaded kind 8 petpet is kind 7 and gains all four bonuses.
- Passive bonuses: a live petpet whose T was set to -1 gives none of the four bonuses.
- Feeding (hero state 49) with no candidate in range after the clip: no fed flag is set, and the hero goes to state 1 after the gate (Medium).

### Feeding, wild-roam reference and state exits

- **Feeding selection (High):** Square uses the currently selected quick-item entry when its marker names an available item. If the marker is negative, the feed action searches the general inventory for its first available category-12 item. The directional selector supports four entries: Right, Up, Left and Down choose entries 0, 1, 2 and 3. A successful feed removes one item and creates a reserved, non-pickable pickup 2.8 units in front of the hero; no available category-12 item means no pickup. The four-entry selector is the quick-item ring and is unrelated to the HUD's three equipment counts (open question 48).
- **Class 33 roaming (High):** The roam centre is the pet's own home point (its placement position), not the active hero position. A roam target is generated by the Roam point rule (a square of side 40 around the home point, offsets -20 to 19 on X and Z). The wait is always 60 ticks (1 s): it is drawn between the tick rate and itself, and the 5-second bound stored beside it is never read; the first wait is 0. The pet waits, walks to its target (arrival below 2.5 units, or a touch, or a failed point test), and on arrival clears it and begins the next wait/target cycle. The separate type-145 anchor behavior is selected only through its own candidate and hero-distance gates; it is not the roam centre.
- **Food reach (High):** The fed wild pet selects the nearest category-12 world pickup within 10 units, excluding type 145. It then checks whether that candidate point can be located in the pet's current navmesh neighborhood. This is a point-location test, not a line-of-sight or segment test. If the nearest candidate fails point location, the pet does not retry with the next-nearest food. The pet approaches until it faces the food with dot product above 0.8 and is less than 2 units away, then eats. Ordinary hero pickup uses a separate path and can have its own visibility checks.
- **Prompt and meter (High for prompt gates; Medium-High for meter fade):** The live petpet prompt's strict range is 2.5 units normally, expanding to 4.5 while remaining time T is below 45 seconds (2,700 ticks); it also requires facing dot above 0.5 and no pending food reservation. Wild petpet prompts use 4.5 units. The meter fill itself is alpha 128 while its width is positive and alpha 0 below one pixel. Its node reveal uses the shared 60-frame fade coefficient and hide clears visibility immediately. The separate Feed petpet caption uses the shared Square-caption default colour (160, 160, 90, 127) and one-pass shown counter; it does not use the meter fade.
- **Slot 38 predicate (High):** It reads the current petpet-kind query, not the music track. Recognized pet type ids map to kinds 0, 1, 3, 6, 7 and 8; no petpet or an unmapped type returns -1. Argument 255 passes any recognized kind; otherwise the mapped kind must equal the argument.
- **State exits (High):** Class 33 completes eating by removing the wild actor and activating a live petpet from its own character id. Class 34's eat state consumes the reserved food after its animation, raises T and F only if the food value exceeds current T, clears the pending event and returns to state selection. A separate five-second reservation fallback can consume and credit food. Class 34's leave state deactivates the pet when it is more than 60 degrees off the camera's line of sight (camera dot below 0.5) or its touched flag is set; otherwise it turns toward the camera with weight 0.1 and runs (animation 3), and stays in state 5. Class 20's consume-and-wander behavior does not activate a live petpet.

### Wild petpet build notes (NPC pass)

High unless marked. A rewrite of the wild petpets (class 33 rows 244, 245, 247, 250, 251, and the class 20 row 252) needs these contact rules on top of the states above.

- Hero blows. A placed petpet is never a hit candidate: its pet type has a kind in one of the two petpet tables, and the hit-candidate query refuses such a character. The hero's swing, Roberta's shots and the Mote walks pass through it with no cue and no hit-list entry. Wild petpets cannot be damaged or killed.
- Reaction to the hero. The class 33 loop has no flee or notice rule: within 10 units of the hero it idles (state 1 plays animation 9 once, then idle); at 10 or more it roams around its home point (the Roam point rule, a 60-tick wait, walks to the point, new 60-tick wait). In state 1 it turns toward the hero every tick (horizontal direction, blend weight 0.1); it does not react to the hero's attacks. The class 20 creature reacts to the hero's attack state and shots through its item test (State 1).
- Reaction to enemies. None for wild petpets. The summoned (class 34) petpet notices only enemies in the agro state, within 10 units, one response per newly noticed enemy.
- Contact. A wild petpet is pushed by the character push like any region character; it does not push back or re-route.
- Sounds. The routines play no sound; any cue comes from the animation table's sound words (SPEC-0031, creature blocks).
- Streaming. A wild petpet is released beyond 280 units and rebound at its placement (SPEC-0006); the record that fed a live petpet comes back at the spot where it ate (Acquisition, Return after leaving).
- Draw. Drawn like any bound region character; the live petpet is drawn from the petpet list.

### Petpet build guide

Confidence: High unless marked. This guide gathers what a rewrite needs to build the hero's live petpet (class 34) so that it looks, moves and behaves like the original, then summarises the wild petpets. The game runs at 60 ticks per second. "Per tick" speeds are layer-0 root steps (SPEC-0005, Playback 5): root z of the playing key × 0.000977 × v, with v = 0.5 ÷ S keys per tick (v = 0.5 when the clip's speed byte S is 0) and play rate R = 1.0 for the petpet (nothing changes its speed scale or applies a status to it). Units per second = per tick × 60.

#### G1. Which petpets exist and how each is obtained

| Kind | Pet type id | Package | Wild copies (packs) | Obtained by | Can be live in play |
| ---: | ---: | ---: | --- | --- | --- |
| 0 | 450 | 534 | 1066 (the first on the main path), 758, 1353; script regions of 706, 800 and 886 | feeding a wild one | yes |
| 1 | 703 | 535 | 934, 1130; script regions of 1350 and 1358 | feeding a wild one | yes |
| 3 | 1314 | 537 | 1230, 1257; script regions of 1300 and 1414 | feeding a wild one | yes |
| 6 | 1460 | 540 | 1250, 1324; script regions of 800 and 1350 | feeding a wild one | yes |
| 7 | 2090 | 541 | 991, 1358; script region of 1184 | feeding a wild one | yes |
| 8 | 2094 | 542 | placed creatures of monster row 252 run class 20 | never: class 20 never joins, and a save stores 2090 | no |

There is no story grant, no shop and no item that gives a petpet (Acquisition (joining); SPEC-0021). Kinds 2, 4, 5 and 9 are other creatures (row 246, the Miamouse, the Mortog and the Whinny), never petpets.

#### G2. The active petpet, equipping and switching

- There is one live-petpet slot for the whole party. It is not per hero, and there is no equip or unequip menu. Feeding a different wild petpet while one is live sets the live one's T to -1, so it leaves, and the fed one joins when it has eaten.
- The class 34 controller reads the selected hero every tick. A hero switch (SPEC-0053) moves nothing: from the next tick the follow point is the incoming hero's. The incoming hero normally takes the outgoing hero's place, so nothing visible happens; in swap mode 4 the incoming hero stands at its own stored spot and the petpet walks there by the rules below.
- The passive bonuses go to the selected hero only (Passive bonuses).
- The petpet leaves only through: T below 1 (state 5), replacement by feeding another wild petpet, an act start, the world reset, and the scene-start no-pet restore.

#### G3. Model, body, clips and sounds

Model: character record 0 of package 534 + kind (Packages by kind), with no extra scale. Body cylinder (motion record): radius 1.175, height 3.5, from monster row 244 + kind (the package record's monster type is 100 + kind); weight 5.5, except kind 6 with 2.0 (weight cancels out of gravity). Can Fly is 0 on all petpet rows: every petpet is a ground walker under normal gravity; nothing hovers. Kind 7's hop and kind 6's run bounce are bone motion inside the clip, not travel.

Animation ids used by class 34 and their clips (one-shot length = (F − 1) ÷ v ticks; loop cycle = F ÷ v ticks):

| Id | Use | Kind 0 (S 0) | Kind 1 (S 0) | Kind 3 (S 2) | Kind 6 (S 2) | Kinds 7 and 8 (S 0) |
| ---: | --- | --- | --- | --- | --- | --- |
| 0 | idle | loop, 122 | loop, 100 | loop, 120 | loop, 80 | loop, 160 |
| 1 | fidget | 90 | 160 | 156 | 88 | 280 |
| 2 | walk | loop, 40 | loop, 40 (same clip as 3) | loop, 28 (same clip as 3) | loop, 40 | loop, 40 (same clip as 3) |
| 3 | run | loop, 24 | loop, 40 | loop, 28 | loop, 28 | loop, 40 |
| 4 | eat | 120 | 170 | 208 | 248 | 140 |
| 6 | happy (also the spawn request) | 38 | 128 | 120 | loop, 40 | loop, 60 |
| 7 | hungry (low time) | 190 | 130 | 136 | 188 | 290 |
| 8 | notice an enemy | 180 | 150 | 176 (moves the pet back about 0.58 units in all) | 148 | no entry: the lookup miss plays clip 0 of the family, which is the hungry clip, 290 |

Travel speeds (root motion only; no code velocity is added):

| Kind | Walk (id 2) per tick | Walk u/s | Run (id 3) per tick | Run u/s |
| ---: | --- | ---: | --- | ---: |
| 0 | 0.012213 | 0.73 | 0.43457 mean (0.4152 to 0.4499 by key) | 26.07 |
| 1 | 0.112355 | 6.74 | 0.112355 (same clip) | 6.74 |
| 3 | 0.225198 | 13.51 | 0.225198 (same clip) | 13.51 |
| 6 | 0.029554 | 1.77 | 0.225198 | 13.51 |
| 7, 8 | 0.32231 mean (0.32143 to 0.32387) | 19.34 | same clip | 19.34 |

For comparison the hero walks at 0.225198 and runs at 0.32143 per tick (SPEC-0040, Behavior 14C). Kinds 1 and 6 cannot keep up with a running hero, kind 3 only matches a walking one, kind 7 is 0.0009 per tick faster than the hero's run, and kind 0 alternates a 26 u/s dash with a 0.73 u/s waddle. When ids 2 and 3 name the same clip, a switch between them starts nothing (SPEC-0005, Clip start), so the travel does not hitch.

The wild copies placed in the packs carry their own clip copies. For kinds 1, 3, 6 and 7 they travel the same per tick as the package; the wild kind 0 differs (walk 0.0176, run 0.2326 per tick). A rewrite uses the package clips for the live petpet.

Sounds (High for the words; Medium that the creature block is resident while the petpet is live): only kinds 1, 3 and 6 have sounds, all in the creature-block group (SPEC-0031), block index 7, 8 and 9 respectively, each starting at frame 0 of its clip, none keep-alive; walk and run are silent; kinds 0, 7 and 8 are silent. Cue ids by animation id (0, 1, 4, 6, 7, 8): kind 1: 0, 1, 2, 3, 4, 5; kind 3: 0, 0, 1, 0, 2, 3; kind 6: 0, 4, 2, 0, 3, 4. A cue with start frame 0 plays once when its clip starts, not on each loop.

#### G4. Per-tick order

Inside one game tick the region characters run first, then the selected hero (its move and its regeneration, petpet bonus included), then the petpet: (1) the class 34 step: the state choice, the state body (turn and clip request), then T falls by 1 if the game-state code is 0; (2) the motion-record tick: the displacement is the root step stored by the previous tick's animation pass, turned by the new facing basis, then gravity, driver branch 2 (its own frame-record step and cell list A) and the integrator; (3) the animation pass: the requested clip starts and its root step is stored for the next tick. The petpet therefore sees the hero's position of the same tick, and a new clip moves the petpet from the next tick (SPEC-0040, Behavior 14D, item 5). In game-state codes 1 to 8 (conversations and the like) the petpet keeps moving with T held; in code 9 it does not update.

#### G5. Shared follow rules (exact numbers)

Choice of state each tick: see Class 34 (items 0 to 3). Two words that persist across states start as: gait 1, last noticed enemy id -1.

- **Hero moving** means the selected hero's action state is 2, 3, 4, 5, 6 (stance walk, run and reverse), 32 (walk and turn), 7 to 11 (jump and air), 43 to 46 (strafe jumps), or 22, 39 or 40 while the hero's animation is 85. Every other state counts as standing, including scripted walks (50 and 52), climbing, ladders, attacks, guard, hurt, feeding and death.
- **State 2 (hero moving).** Follow point Q = hero position + 4.0 × B3 + 1.5 × B1 of the hero, recomputed every tick. d = horizontal distance to Q. Gait 0: d ≤ 3.0 idle (id 0), else walk (id 2) and gait 1, no turn this tick. Gait 1: d > 7.0 run (id 3) and gait 2; d < 1.5 idle and gait 0; otherwise walk. Gait 2: d < 5.0 walk and gait 1; otherwise run. Gaits 1 and 2 turn toward Q every tick (after the gait test, even on the tick they change gait).
- **State 1 (hero standing).** On entry from any other state a new idle point is drawn (hero position ± 3.0 to 10.0 on world X and ± 3.0 to 10.0 on world Z) and the wait is unset. Each tick: first the enemy notice (below); then, if d (horizontal, to the idle point) ≥ 1.0: turn toward it with weight 0.1, request walk (id 2) and unset the wait. If d < 1.0 and the wait is unset: draw a wait from 180 to 600 ticks inclusive and request idle (id 0). On later ticks the wait falls by 1; when it reaches 0 a new idle point is drawn around the hero's current position, the wait is unset and nothing is requested that tick; otherwise the idle variant is requested: id 7 when T < 1,800; else id 1 or id 6 by the draw of Class 34, state 1. The variant is drawn and requested on every tick of the wait. Consequences: with T ≥ 2,700 the variant is always id 6; it plays once and holds its last key for the rest of the wait where id 6 is a one-shot (kinds 0, 1 and 3) and loops where it loops (kinds 6, 7 and 8). With T below 1,800 id 7 plays once and holds. With 1,800 ≤ T < 2,700 the variant can change from tick to tick (for example at T = 2,250, r = 1.25, id 1 has chance 50 in 101 each tick), and each change starts the other clip and its sound again, so the petpet twitches between the two poses. Reproduce this. A held one-shot keeps moving by the finished-clip rule of SPEC-0005 (Layer step, step 8: twice the last key's root step every tick): kind 0's happy clip ends on root x = 8, so a kind 0 petpet holding it slides sideways (along its own B1) by 2 × 8 × 0.000977 × 0.5 = 0.0078 per tick, up to about 4.4 units over a 600-tick wait; kind 6's fidget ends on root z = −75 and slides back 0.0366 per tick while held. The other held clips end on zero root (Medium-High for the visible effect).
- **Enemy notice (state 1 only).** The first region character, in list order, that is in the agro state (class state 19), within 10.0 horizontal units and not the last noticed one: its id is stored and id 8 is requested. While id 8 plays nothing else runs (no walk, no turn toward the enemy); when it has finished, id 0 is requested and the walk logic resumes on the next tick. The same enemy is never noticed twice in a row; the stored id survives state changes.
- **Turning.** Every turn is the shared aim rule with weight 0.1 (SPEC-0040, Behavior 8): facing = normalise(0.1 t + 0.9 c), with the 0.1 side nudge when the two are nearly opposite. There is no turn in place: the petpet walks along its current facing while it turns, so it moves on an arc. No turn runs while idling, in gait 0, in the notice, while eating, or in state 3.
- **Speed.** Always the clip's root step (G3). There is no acceleration, no speed matching and no leash distance; distance only selects walk, run or idle.
- **Teleport to the hero.** Only state 3: a petpet whose body touched a wall or shape on the previous motion step stops (id 0, no turning) and stays there while it is within 60 degrees of the camera's line of sight. Once it is outside (camera dot below 0.5) and the ground 10 units from the hero toward the camera is within 2.0 of the hero's height, it is placed there, facing the hero, and resumes state 1 with its old idle point. There is no rule for a petpet that is merely far away: a slow petpet that never touches anything walks after the hero indefinitely.

#### G6. Ground, walls, ledges, water, doors, world swaps, movies and hero switches

- **Ground following.** Gravity (gain 1.0: 128 units/s²), the ground constraint and the ground projection of the step are the shared motion-record rules (SPEC-0040, Behavior 14A items 5 to 7 and 14C item 7). Slopes and steps follow from them.
- **Walls.** Driver branch 2 runs the petpet's own frame-record step and cell list A (SPEC-0018). It runs no navmesh confinement, no cell list B and no character push, and the petpet is not in the region-character list, so the hero and other characters neither push it nor are pushed by it: they pass through each other (Medium-High). Any applied contact response sets the touched flag, which sends the petpet to state 3 on the next tick.
- **Ledges.** The edge guard of SPEC-0040, Behavior 14E applies only to region characters; the petpet is never guarded. It walks off any ledge and falls with no fall damage. Its frame-wall response is also skipped when the ground 2 units ahead is at least 2.0 lower (SPEC-0018), so a wall standing at a drop does not stop or bump it.
- **Water and pits.** The petpet has no deadly-ground, water or hazard test (those tests need the hero flag; SPEC-0040, Behavior 14E). Default if unresolved (rewrite choice): it walks on the ground under water like any ground, and one that falls where there is no ground keeps falling until a respawn, landing, door, world switch or act start moves or removes it.
- **Doors and interiors.** Entering and leaving an interior place the petpet with the hero, at the hero's pose (SPEC-0026).
- **Teleport landings and world switches.** The petpet gets the hero's world id and the hero's landing pose (or the world's start locator pose, when one is set), the same pose as the hero.
- **Respawn and movies.** Every respawn of the selected hero (death, deadly ground, and the reload after a movie, R-0141) gives the petpet the hero's world id and the hero's start pose, which is the hero's own position and facing, so the two overlap; with no push between them they stay overlapped until the petpet walks off (SPEC-0006, SPEC-0040 Behavior 1C).
- **Hero switches.** Nothing moves (G2).
- **Act starts.** Each act start removes the petpet (Area changes).

#### G7. Abilities, by kind

| Kind | Damage dealt by the hero | Damage taken by the hero | Hit points | Magic | Other |
| ---: | --- | --- | --- | --- | --- |
| 0 | × 1.25 | - | - | - | - |
| 1 | - | × 0.9 | - | - | - |
| 3 | - | - | - | +1 per second | - |
| 6 | - | - | +1 per second | - | - |
| 7 | × 1.25 | × 0.9 | +1 per second | +1 per second | - |
| 8 | (never live; would have none) | | | | |

All bonuses need T at least 1 and go to the selected hero (Passive bonuses). The petpet has no other ability: it does not attack, does not pick up coins, Neopoints or items, has no magnet, shield or heal for anyone but the selected hero, and the enemy notice is a look only. The only thing it takes from the world is food:

- **Eating.** Feeding it (hero state 49) sets its pending event; on its next update it enters state 4 and takes the nearest category-12 world item to itself, wherever that is (no range limit). The food it eats need not be the one the hero dropped: if another food item lies nearer, that one is eaten and the dropped one stays on the ground, uncollectable and no longer reserved (Medium-High). It eats where it stands, without walking or turning, plays id 4 to its end, then gets the food's value (Hit value and score) and returns to the state choice. A Bagguss (145) nearest to it is skipped and nothing is reserved that tick, so the petpet stands in state 4 until the 5-second fallback removes the dropped item (value 0) and the next search finds nothing (state 0) (Medium-High).
- **Invulnerable.** No hero blow, shot, Mote or enemy attack targets it (the hit tests use the region-character list and the hero, and the petpet kinds fail the hittable query). It has no health in play (Medium-High).
- **HUD.** While a recognised petpet is live the petpet panel shows its kind's portrait and the meter round(60 × T ÷ F) (SPEC-0023, sections 7 and 37).
- **Save.** Type, T and F (Save).

#### G8. Worked examples

1. **The hero runs off (kind 1).** Hero at the origin facing +Z, starts running on tick 1 (0.32143 per tick). The petpet stands at (−3, 0, −2) facing +Z, gait 0. Tick 1: Q = (1.5, 0, 4.32); d = 7.76 > 3.0, so walk (id 2), gait 1, no turn; it does not move this tick (the previous step was idle). Tick 2: d = 8.02 > 7.0, so run (id 3, the same clip), gait 2; it turns 10 % toward Q (facing (0.057, 0, 0.998)) and moves 0.11236. From then on it runs at 0.11236 per tick while the hero gains 0.20907 per tick: its distance to the hero is 3.79 on tick 1, 14.97 on tick 60 and 27.5 on tick 120. Hero stops on tick 120. Tick 121: state 1; say the draw gives the idle point hero + (6, 0, −4); the petpet walks (same clip, no hitch), turning 0.1 per tick, and first has d < 1.0 on tick 330 (209 ticks of walking); it then draws a wait of 180 to 600 ticks and requests id 0, and from the next tick the idle variant.
2. **A faster kind keeps up.** Same start with kind 7: from tick 2 it runs at 0.32231 per tick and settles about 3.3 units behind the hero (7.2 to 7.3 from Q, so it stays in gait 2). With kind 0: it dashes (id 3, mean 0.43457 per tick) from tick 2 to tick 26, drops to gait 1 on tick 27 (d 4.99 < 5.0) and waddles at 0.0122 per tick until d exceeds 7.0 on tick 35, then dashes again: a saw-tooth between about 4.9 and 7.3 from Q.
3. **A wall bump and the teleport.** A kind 1 petpet walks into a wall on tick n; the contact response sets its touched flag. Tick n + 1: state 3, id 0, no movement. The hero walks on and the camera follows. While the petpet is within 60 degrees of the camera's line of sight it stays. On the first tick where it is not (for example hero at (0, 0, 80), camera eye at (0, 10, 30) looking along +Z and down, petpet at (−30, 0, 20): dot = −0.24) and the ground at (0, y, 70) is within 2.0 of the hero's height, it is placed at (0, hero y, 70) facing +Z and enters state 1 with its old idle point. If T fell below 1 while it was stuck it still waits for this; on the next tick it then enters state 5.
4. **An ability firing (kind 6 heal).** T = 18,000 at the hero's update on tick k: 18,000 is a multiple of 60, so the hero gains 1 hit point; the petpet then makes T 17,999. The next gain is on tick k + 60 (T = 17,940). In a conversation (game-state code 3) starting with T = 1,200 the hero gains 1 point on every tick of the conversation (Passive bonuses, freeze quirk).
5. **Feeding the live petpet (kind 1, Chokato).** Tick n: the hero's feed reaches progress 0.99, drops the Chokato 2.8 units ahead (reserved) and sets the petpet's pending event. Tick n + 1: state 4 (event cleared); the Chokato is the nearest food, so it is reserved and id 4 is requested; the petpet does not move. Its eat clip lasts 170 ticks. On the first update after the clip has finished: value 5.0 minutes gives 18,000; if T was below 18,000, T and F become 18,000; the item is removed, the low-time latch is cleared and the state is 0. The next tick the state choice runs again.
6. **Low time and leaving.** T drops below 1,800 for the first time since the last meal: state 6, the hungry clip plays once (kind 1: 130 ticks) while the petpet stands; then state 0 and normal following, with id 7 as the idle variant. When T falls below 1: state 5; each tick it turns toward the viewer and runs (kind 1 at 6.74 u/s) until it is more than 60 degrees off the camera's line of sight or touches something; then it is removed and the panel disappears.

#### G9. What a port most likely gets wrong

1. A smoothed follow (lerp, speed matching, acceleration, leash). The original moves only by the clip's root step and picks idle, walk or run by distance; slow kinds fall behind a running hero for good.
2. A distance teleport. There is none; the only catch-up jump is the bumped-and-out-of-view rule of state 3.
3. The leave rule. The petpet runs toward the camera and vanishes when out of view or on a touch; it does not test the hero or its own facing.
4. Speeds: treating speed byte 0 as invalid (it means 0.5 key per tick), multiplying the per-key root by the key count, or using the wild kind 0's clips for the live one.
5. Turning by a fixed angle per tick, or turning while idle, in gait 0 or during the enemy notice. It is a 0.1 blend only in the moving branches.
6. The idle point relative to the hero's facing. It is ± 3 to 10 on world X and world Z.
7. The follow point's side. It is + 1.5 along the hero's B1 = normalise(up × forward), 4.0 ahead.
8. A rest between idle variants, or restarting a finished one-shot variant. The variant is re-requested every tick; one-shots hold their last key (and a held kind 0 happy clip slides sideways); between T = 1,800 and 2,700 the variant twitches.
9. Kind 7's (and kind 8's) notice. Id 8 is missing and plays the hungry clip.
10. The edge guard, navmesh confinement or character push applied to the petpet. It has none of them.
11. Following during scripted walks. States 50 and 52 count as standing, so the petpet heads for an old idle point instead.
12. Letting enemies or the hero hit it.
13. Eating only the dropped food. It eats the nearest food anywhere.
14. Freezing it in conversations. It keeps moving; only T is held.
15. The wild roam wait. It is always 60 ticks, not 1 to 5 seconds.

#### G10. Wild petpets and the class 20 creature

- Wild petpets (class 33) are region characters: they get the edge guard, navmesh confinement and character push of driver branch 1, and they roam with the shared goal-move routine (SPEC-0040, Behavior 8a) using animation slots 2 (walk) and 3 (run). Their speeds come from their own placed clips (G3). Roam wait: 60 ticks; arrival below 2.5 units (State 3, Roam).
- The class 20 creature (row 252, kind 8) is described in its own section; it never becomes the live petpet.

#### G11. Open points with defaults

- Default if unresolved (rewrite choice): keep the creature sound block of kinds 1, 3 and 6 loaded while the petpet is live.
- Default if unresolved (rewrite choice): the spawn request of id 6 shows for the spawn tick only; the first class 34 step (state 1 or 2) requests its own clip on the next tick (Medium).
- Default if unresolved (rewrite choice): no petpet update runs while a movie plays; after the movie the respawn places it on the hero.

## Test vectors

Synthetic and hand-checkable. Vectors 1 to 4 cover acquisition, scene start and save; vectors 30 to 33 and 37 to 40 cover the remaining time, the heal rate and joining. Vectors 5 to 27 use the base table and the multiplier table above.

1. Fresh game (no save loaded): the pending type is -1; a scene starts; the pending load flag is clear, so nothing happens; the script query returns -1. (High)
2. Save: live type 450, T = 27000, F = 27000 gives 450, 27000, 27000 at 0x214, 0x218, 0x21C. No live pet gives -1, 0, 0. A live pet with T = 0 gives -1, 0 and its F. (High)
3. Save with 0x214 = 703, 0x218 = 27000, 0x21C = 27000, loaded, then a scene starts: kind 1; package 535 loads; the spawn, 7 units in front of the hero, takes the pending T and F (27000 and 27000) and clears the pending values (type -1, times 0); the meter is full; the script query returns 1. (Medium)
4. Save with 0x214 = -1 (empty slot template), loaded, then a scene starts: the pending type is below 0, so the petpet is deactivated; no petpet; the script query returns -1. (High)
5. Pet type 703 gives kind 1. Pet type 4660 gives no kind.
6. Target 146, kind 0: 146 is in A, so multiplier 1.5; base 5.0; hit 7.5; score floor(7.5 x 3600) = 27000.
7. Target 146, kind 1: 146 is in neither B nor C, so multiplier 1.0; hit 5.0; score 18000.
8. Target 146, kind 7: 146 is in A, so multiplier 0.5; hit 2.5; score 9000.
9. Target 147, kind 3: 147 is in B, so multiplier 1.5; base 10.0; hit 15.0; score 54000.
10. Target 150, kind 3: 150 is in A only, so the 1.5 test fails and the 0.75 test passes; base 2.0; hit 1.5; score 5400.
11. Target 150, kind 6: 150 is in A, so the 1.5 test passes; hit 3.0; score 10800.
12. Target 152, kind 6: 152 is in C, so the 1.5 test passes; base 60.0; hit 90.0; score 324000.
13. Target 155, kind 7: 155 is in C, so multiplier 0.5; base 30.0; hit 15.0; score 54000.
14. Target 152, kind 8: kind 8 has multiplier 1.0; hit 60.0; score 216000.
15. Target 145 (anchor), any kind: hit 0.
16. Target 148 (base 0), any kind: hit 0.
17. Time update: T 27000, candidate 18000: T stays 27000. T 27000, candidate 54000: T and F become 54000.
18. Save: petpet type 703, T 27000, F 27000. The slot holds 703 at 0x214, 27000 at 0x218 and 27000 at 0x21C. With no petpet, -1 is written at 0x214.
19. Script query: petpet type 703 with T = 27000 gives kind 1. The same petpet with T = 0 gives -1 (High).
20. Second table: pet type 805 gives kind 2 in the second table only. A saved type of 805 spawns nothing.
21. State selection: distance 10.0 from the hero with no targets gives roam (3). Distance 9.9 with no targets and no anchor within 5 gives state 1. Distance 8 from the hero with a non-anchor target 9 units from the petpet gives strike (4).
22. Hover: distance 8 from the hero, no non-anchor target, an anchor 4 units from the petpet gives hover (5).
23. Spawn: a hero at the origin facing along +Z gives a fresh petpet at (0, 0, 7) (the facing convention is assumed).
24. Strike fallback: food reserved with no petpet in its eating step: the timer passes 5.0 seconds after about 300 ticks at 1/60 s per tick, so the food is applied on about the 301st tick. With a petpet in its eating step on every tick, the timer is reset each tick and the fallback never runs.
25. Class 20, state 2 latch: a non-anchor object 2.5 units from the pet, with the latch clear, is not yet within the latch distance: the pet moves toward it and stays in state 2. An object 2.4 units away latches: the latch is set, the object is consumed, and the state stays 2 for that tick. On the next tick animation 4 plays. When it finishes, the state is 3 with sub-step 0 and a wait of 240 to 300 ticks (4 to 5 seconds). (The boundary at exactly 2.5 units follows "2.5 or more units away moves"; Medium.)
26. Class 20, wander arrival: in state 3 sub-step 1, a pet 6.0 units from its stored point keeps walking. A pet 4.0 units from its stored point is within 5.0 units: it draws a new wait of 240 to 300 ticks and returns to sub-step 0. (The boundary at exactly 5.0 units is open.)
27. Class 20, hero check (sub-step 2 of state 1): pet 2.0 units from the hero, vertical separation 0.6, the hero's animation id 3: the check passes. The same pet with the hero's animation id 7: the check fails. Vertical separation exactly 0.5 with animation id 3: the check fails (the separation must be more than 0.5). Hero feet (1.0, 0.7, 0) against a pet origin (0, 0, 0), hero state 9: horizontal 1.0 is below 2.5 and the height difference 0.7 is above 0.5, so he is launched; the same with state 7 is not; feet at height 0.4 is not; feet 0.7 below the pet's origin also passes.
28. Passive damage factor: a hero damage product of 10.0 with a live kind 0 or kind 7 petpet becomes 12.5; with kind 1, 3 or 6, or no petpet, it stays 10.0.
29. Passive intake factor: a hero damage-intake product of 20.0 with a live kind 1 or kind 7 petpet becomes 18.0; with kind 0, 3 or 6 it stays 20.0.
30. Heal rate: kind 6, T = 18000, game-state code 0 on every tick, hero at 50 of 120 hit points, no item-based regeneration: exactly +1 hit point in every 60 consecutive ticks; after 600 ticks the hero has 60. Same for kind 3 on magic. Kind 7: +1 hit point and +1 magic per 60 ticks.
31. Whole stay: T counts from 18000 down to 1 with no feeding: 300 passing values (the multiples of 60 from 60 to 18000), so at most +300 hit points over 5 minutes.
32. Test values: T = 60 passes; T = 59 fails; T = 0 fails (no pet below 1, so no kind); T = -1 fails; T = 1 fails.
33. Freeze: T = 1200 when a 600-tick conversation starts (code 3): T stays 1200, and the hero gains 1 hit point on each of the 600 ticks (capped at the maximum). T = 1201 at the start: no heal during the talk.
34. Class 34 low-time mark: the word 0.5 gives 0.5 x 60 x 60 = 1,800 ticks. With T = 1 and no contact or pending event, the pet keeps following; on the next tick with game-state code 0, T = 0 (below 1) and the pet goes to state 5 (leave).
35. Class 34 order: body contact this tick and a pending event both set, pet not leaving: state 3 (contact is tested first); the pending event stays set.
36. Feeding (hero state 49; synthetic): own petpet at 9.0 and another character at 12.0 units after the clip: the petpet at 9.0 gets the fed flag (12.0 is beyond 10.5). A class-20 creature at 20.0 units (within 25.0) with nothing nearer qualifies.
37. Join: a wild kind 1 petpet (type 703) fed a Chokato (146; not in B or C): T = 5.0 x 3600 = 18000 ticks (5 minutes). A wild kind 0 petpet (450) fed a Chokato (146 in A): 7.5 x 3600 = 27000 (7.5 minutes). A wild kind 6 petpet fed a Silver Negg (152 in C): 90 x 3600 = 324000 (90 minutes). The Bagguss (145) can be fed but is never eaten, so no petpet joins.
38. Low time: T falls below 1800 for the first time since the last meal: state 6 (animation 7) runs once. At T < 1 the pet goes to state 5 and leaves; the script query then returns -1.
39. Load 450 / 27000 / 27000, scene start: a fresh spawn 7 units ahead of the hero, T = 27000, meter full. Load -1: deactivate, no pet.
40. Second feed: live kind 0 petpet, the hero feeds a wild kind 6 petpet: the live petpet's T becomes -1 and it leaves; the kind 6 petpet joins when it has eaten.
41. Live petpet eats in place: live kind 0 petpet 8.0 units from food it was fed (Chokato, 146): on the next tick it requests animation 4 without moving; when the clip ends T and F become 27000 (if T was lower) and the food is gone.
42. Wild petpet eat test: food 1.5 units away, facing dot 0.9: eats. Food 1.5 units away, dot 0.7: turns toward it, no eat. Food 3.0 units away, dot 1.0: walks toward it. A wild petpet whose path is blocked for 10 s never eats by the timer (its timer is reset each tick).
43. Fallback: food dropped with no petpet entering its eating step, live kind 0 petpet with T = 1000: after about 300 ticks (5 s) the food is removed and T and F become 27000. Same with no live petpet: the food is removed and nothing else changes.
44. Death: a hero enters the death state with a live kind 6 petpet, T = 6000: the petpet stays and T keeps falling by 1 per tick in game-state code 0.
45. Return: the wild type 450 petpet of pack 1066 joins and later leaves at T below 1; with the hero within 280 units of the spot where it ate and the block under its cap, a later tick binds it again there.
46. Slots: pack 706 with flag 63 the highest set of flags 63 to 67 loads set 1 (region 708): its type 450 copy appears; with flag 67 set it loads set 5 (region 712); pack 1414 loads region 1416, whose type 1314 copy appears.
47. State 2: no input to the class 33 selector gives state 2.
48. Class 34 moving-follow gait 0: horizontal target distance 3.0 keeps animation 0 and gait 0; distance 3.01 selects animation 2 and gait 1.
49. Class 34 bumped recovery: camera dot 0.49 and candidate ground height 1.99 units from the hero's height pass both gates; with the candidate 10 units along the normalized horizontal negative camera-view vector, the pet is placed there facing the hero and enters state 1. A dot of exactly 0.5 or a ground-height difference of exactly 2.0 fails its strict comparison, so the pet stays in state 3 and requests animation 0.
49. Class 34 moving-follow gait 1: distance 1.5 or 7.0 selects animation 2 and keeps gait 1; distance 1.49 selects animation 0 and gait 0; distance 7.01 selects animation 3 and gait 2. Class 34 moving-follow gait 2: distance 5.0 selects animation 3 and keeps gait 2; distance 4.99 selects animation 2 and gait 1. Gaits 1 and 2 aim toward the follow point with weight 0.1.
50. Class 34 state 1: no new character response, horizontal target distance 1.0 uses animation 2, aims at weight 0.1, and clears the wait timer. At distance 0.99 with an unset wait timer, a value from 180 through 600 is drawn and animation 0 is requested. If the active wait reaches 0 after its decrement, a new target point is selected.
51. Class 34 state 1, active wait with the low-time state already run: T = 900 gives r = 0.5, so any draw q in [0,100] fails q > 200 × |1 − r| and animation 6 is requested. T = 1,800 gives r = 1, so any positive q selects animation 1 and q = 0 selects animation 6. T = 1,799 takes the r < 1 branch and requests animation 7. T is decremented after the selection.

52. Negative feed marker: general inventory slots 3 and 7 contain category-12 food ids 149 and 153; the feed action selects slot 3, drops one id 149 pickup, and consumes one from that slot. With no category-12 item, it creates no pickup. Marker selecting an available quick-item entry uses that selected id.
53. Wild roam: pet at least 10 units from its hero, configured reference point at R, selected target 39.9 units from R: state 3 walks at once (first wait 0), stops below 2.5 units from the target, waits exactly 60 ticks, walks to that target, then chooses a new target around R. A target around the hero instead would not match this rule.
54. Slot 38: active type 2090 maps to kind 7; argument 255 and argument 7 pass, argument 6 fails. With no active petpet all three fail.
55. Food candidate selection: if the nearest category-12 pickup is outside the pet's current navmesh neighborhood and a farther candidate is inside it, the nearest candidate fails and the farther candidate is not tried. A candidate point in the neighborhood does not need a clear line to the pet; the separate eat gate still requires distance below 2 and facing dot above 0.8.
56. Class 34 state 1 idle point: hero at (10, 0, 20); draws 4.5 (sign draw 0), 7.25 (sign draw 1): point (14.5, 0, 12.75), whatever the hero's facing.
57. Class 34 state 2 follow point: hero at (0, 0, 0) facing +Z (B3 = (0, 0, 1), B1 = (1, 0, 0)): Q = (1.5, 0, 4.0). Facing +X (B3 = (1, 0, 0), B1 = (0, 0, −1)): Q = (4.0, 0, −1.5).
58. Turning: facing (0, 0, 1), target direction (1, 0, 0), weight 0.1: new facing normalise(0.1, 0, 0.9) = (0.1104, 0, 0.9939). Gait 0 (d ≤ 3.0): the facing is unchanged.
59. Kind speeds (per tick): kind 1 walk and run 0.112355; kind 3 0.225198; kind 6 walk 0.029554, run 0.225198; kind 7 run 0.32231 (key mean); kind 0 walk 0.012213, run 0.43457 (key mean).
60. Leave: T = 0, camera dot 0.7, untouched: the petpet turns toward the horizontal negative camera forward with weight 0.1 and requests id 3; camera dot 0.49: it is removed that tick. Touched with dot 0.9: removed.
61. Bumped petpet with T = −30 (stuck) whose low-time state has already run: it stays in state 3 while in view; after the out-of-view reposition it is in state 1 for that tick and enters state 5 on the next. If the low-time state has not run yet, it enters state 6 first (hungry clip), then state 0, then state 5.
62. Class 33 roam: arrival at 2.4 units: the next wait is 60 ticks (never 61 to 300).
63. Kind 7 enemy notice: id 8 has no entry, so clip 0 of its family (the hungry clip, 290 ticks) plays; the pet resumes walking on the tick after it has finished.
64. Wild selector, lost feeding: hero 9 units away, fed flag set, nearest category-12 item the Bagguss 3 units away. The food test fails and clears the flag; the anchor test passes: hover (5). A non-anchor food that is nearest on a later tick does not send the pet to strike unless the pet is fed again.
65. Wild selector, flag kept: hero 10.2 units away, flag set: roam (3), flag still set. After the pet walks to 9.5 units from the hero with a non-anchor food 6 units away (nearest, on the navmesh): strike (4), flag cleared. Once in strike, moving back out to 12 units from the hero leaves the state at 4.
66. Hover gait and range: anchor at (0, 0, 0), petpet at (3, 0, 4) in hover: goal (6, 0, 8); the pet runs (animation 3) until its three-dimensional distance to the goal is below 2.0, then the state is 0. A goal 1.9 horizontal units away but 2.5 units higher is 3.14 away and is not yet reached.
67. Wild roam timing: the pet arrives on tick A and sets its wait to 60. Ticks A+1 to A+59 play animation 0 (the wait counts 59 down to 1); on tick A+60 the wait is 0 and the walk request is made.
68. Wild state 1: the selector enters state 1 with the hero 9 units away. Tick 1: the pet turns 0.1 toward the hero and requests animation 9. Following ticks: it keeps turning at 0.1; when animation 9 has finished it requests animation 0 and keeps turning while the hero stays within 10 units.
69. Eat distance is three-dimensional: food 1.9 units away horizontally and 2.5 units higher than the pet is 3.14 units away; the eat test (below 2.0) fails and a wild pet walks toward it.
70. Class 20 food range: a food item 24.9 units away (three-dimensional) on the navmesh, state 0: state 2. The same item at 25.0: not chosen (strict), state 3. The anchor as nearest item: never chosen; while already in state 2 the pet then does nothing each tick.

## Open questions
- Default if unresolved (rewrite choice): keep the second item-test result as an opaque boolean from its adjusted receiver; do not assume the receiver is the selected hero.
- Default if unresolved (rewrite choice): after class-20 pickup, mark the matched world-pickup record inactive and let the world-pool manager own any reuse; do not infer a respawn.

1. Origin of the first petpet. Resolved: a petpet joins in normal play when the hero feeds a wild petpet (Acquisition (joining); High).
2. Resolved: the "creature flag 0x22" set on spawn is the request of behaviour class 34 at priority 2 (0x22 = 34), not a flag (High). Still open: the object-effect call used by strike and class 20.
2a. Resolved: the score and the remaining time are trunc(v x tick rate x 60), with the tick rate the run-time value 60 (SPEC-0007) and 60 the seconds per minute, so 3600 = 60 x 60 (High).
3. Resolved: the character gate that calls the second-table check is the enemy-kind "can be hit" query used by the hero's swing, the hero's shots and the Mote and charm walks (SPEC-0036, Hittable predicate; SPEC-0040, Hittable predicate). It rejects a character whose spawn-record pet type id maps to kind 2, 4, 5 or 9 (second table) or to a kind of the primary table (kinds 0, 1, 3, 6, 7 and 8), and it rejects the second actor record. So wild petpets and the ornamental second-table characters cannot be struck. The primary table is used by activation, restore, the script query and this query; a request queue that tests the primary table first never reaches the second table for a second-table-only id (High). Test vector: a swing overlapping a character with pet type id 703 (kind 1) or 805 (kind 2) strikes neither; the same swing on a character with an unlisted id such as 61 strikes it when its other hittable tests pass.
4. Class of placed pet characters. Resolved: class 33, the row default of the placed pet rows 244, 245, 247, 250 and 251; the summoned petpet is requested into class 34 (High).
5. What sets the live petpet's time field to -1. Resolved: feeding another wild petpet while one is live (Feeding a petpet; High).
6. Resolved in part: kind 8 (type 2094) placements run class 20; they never call the activation, so they cannot join this way (High; "Feeding a petpet"). The species that kind 8 depicts is developer intent and is not encoded anywhere (D).
7. Resolved: the live petpet's runtime type record is character record ordinal 0 from the loaded package; each package has one kind-3 character record, so this is its first and only character record (High; Packages by kind).
8. Whether any path sets state 2 for the petpet. Resolved: no; the class 33 test for state 2 always fails, so state 2 (idle, animation 0) is never chosen (State selection; High).
9. Resolved (D, not needed for the rewrite): the remaining-time word is a general per-character word that other creature classes use for their own timers; the petpet's use of it is fully specified here and no other class reads a petpet's copy.
10. Resolved in part: the activation by id after a landed hit is the wild petpet joining, not the live petpet re-creating itself (Activation by id; High). Resolved: kind 8 never becomes the live petpet in play, so the save path never writes it; the 2090-versus-2094 package entry is a data quirk with no play effect, and whether it was intended is developer intent (D).
11. Resolved: type 145 is item id 145 (Bagguss, SPEC-0021): a world pickup in the food category whose food value is 0 and which is never a strike target. The class 33 anchor behavior is a lure by that item placed in the world; it is not the owning character or a companion (High for the id and the value; the reason the designers chose it is not encoded anywhere).
12. The 0.5 multiplier. Resolved (High): it is a code constant, like 1.5 and 0.75. The word at 0x2DB1D8 (value 0.5) is read in two places only, both in class 34, as the petpet's low-time mark in minutes; the food-value routine loads no data word for its three multipliers. Test vector: a kind 7 petpet fed a food item whose type is in set A, B or C gets exactly half the base value (a base of 2.0 gives 1.0), and a kind 6 petpet fed a type in neither of its sets gets multiplier 1.0, whatever the word at 0x2DB1D8 holds.
13. Argument of the script query's gate. Resolved: the query applies the presence gate, so it returns -1 when T is below 1 (Script query; High).
14. Resolved: the petpet has no row of its own in the monster table (resource 530, SPEC-0029, Layout 4); the petpet code never reads that table, and its hit and score values come from its kind table (Medium-High).
15. Resolved: pet activation loads archive resource 534 plus the kind and instantiates character record ordinal 0 from that package (High; Activation by id and Packages by kind).
16. Score-below-1 gate of the script query. Resolved: the gate applies; T below 1 gives -1 (Script query; High).
17. The per-tick step of the fallback strike timer. Resolved: the tick length 1/60 s, or 1/180 s while the slow flag set by a class 20 creature is on; every reservation resets the timer and the flag (Strike, step 5; High).
18. Resolved (proactive (open close-out)): the first item-test check is the hero's variant-specific attack-state query; the second returns the attack-active event bit from the selected hero's own sub-object (the item test is handed the selected hero as its subject, and the receiver is that hero adjusted to its sub-object); and the third is the shared reach/contact test with a 65.0-unit range gate and character bone-shape checks. The six hero-side slots are shot slots: an armed slot provides a shot position and radius, and a successful overlap with the pet disarms it. High for the three checks and the receiver; Medium-High for the shot-slot behavior.
19. Resolved: the 8.0 is added to the vertical (Y) lane of the hero's velocity vector (class 20, state 1; see the sub-step 2 text) (High).
20. Resolved in part: class-20 pickup releases the matched record's attached resources and resets its transform, state, and five child-resource slots. The fixed-stride pool record itself is not freed by this path. Whether a later spawn reuses the cleared record has no visible effect, because the class-20 creatures are placed once per load. Medium-High for cleanup.
21. Resolved: the stored wander point starts as the pet's home point, so the first wander entered at sub-step 1 walks to the home point (High).
22. Resolved: the hero method is the action-state setter, called with state 7 (standing jump); the hero flag is a one-tick hold that stops the next hero pass from zeroing the hero's velocity, so the launch of Question 19 survives that pass (High).
23. Resolved (High): the hero-attached-looking records (states 1, 2, 5) are the world's statue puzzle objects, not petpets; no petpet state, save field or kind is shared with them, and none is ever created in play. See "Statue puzzle objects".
24. Resolved: the 0.9/0.1 blend belongs to kind 5 of the puzzle objects (a homing strike on the hero), between the current facing direction and the normalized direction toward the point 1.5 units above the hero; the position advances 0.25 units along the facing direction. The blend runs only when their dot product is strictly between 0.5 and 0.99. High (see "Statue puzzle objects").
25. Resolved (Medium-High): the timer with a speed value is a fall. The vertical speed drops by 32.0 x 0.008333334 per tick, is added to y, and the fall ends when the ground height (probe from 8 units above) is reached. See "Statue puzzle objects".
26. Resolved (High): kind 2 of the puzzle objects turns toward the fixed world direction +X by the blend 0.99 x forward + 0.01 x +X, normalized, each tick until the forward x component is 0.99 or more; see "Statue puzzle objects" for the up and side vectors and a test vector (263 ticks from forward (0, 0, 1)). It is not a companion state.
27. Resolved (Medium): the call at the end of kind 5 is the hero's take-hit (damage 1, attacker -1, blocked flag 0), made when the record ends a tick within 0.5 of its target point. It is not an animation message. See "Statue puzzle objects".
28. Resolved: the pending flag that the scene start checks is the same one as the save-load apply flag (SPEC-0055). With it set, the scene start puts the selected hero at the saved position and facing, re-runs the hero's pose set-up, clears the saved vectors and the flag, and then applies the petpet rule above. The "hero flag" reset in the Scene start text is this apply step (Medium-High).
29. Resolved: item 126 (the Tattered Cloak) has no PetPet effect. The item-screen and quick-panel use paths share the dispatcher, which refuses the cloak without consuming it; the item-screen Square action equips it instead. Host row 353 invokes the same use path. The separate petpet-feeding interaction uses category-12 food. The only PetPet-specific host API is a read-only kind query, and no item-use callback calls it (High).
30. Writers of flags 539, 540 and 594. The level scripts set 539 and 540 only in the Cinthia conversation, and only the trigger room itself sets 594: its per-tick update sets it when the ritual fade completes (5.0 s after the trigger starts the ritual). Resolved: the boot program names no flag above 332 by a fixed number (its fixed-number setters write only 220, 248 and 332, SPEC-0053 section 3.7); the other boot setters take a variable number from a stage table or a debug range command, and neither is reached with 539, 540 or 594 in normal play. So flag 539 is first set by the first Cinthia conversation, and the waiting lines can show only on visits after the first (High for the fixed writers; Medium-High for the stage table, whose entries are read at run time).
31. Resolved (D): the boot program's four-entry table of item-use category names (food, potion, cloak, failure) is a set of label strings with no gameplay effect; the cloak's behavior is settled by Question 29 (no petpet effect), so the table's role does not matter to the rewrite.
32. Passive damage factors. Answered: the 1.25 factor is factor E of the hero's outgoing damage (SPEC-0036, Damage), and the 0.9 factor applies to enemy hits taken by the hero, after the shield block and the armour terms (SPEC-0036, Enemy attack damage to a hero).
33. Whether the petpet stays while the hero is in the death state. Resolved: it stays; nothing on the death path removes it and its state choice has no health test (Passive bonuses; High). The point is still added, but the death state has no health exit (Medium; SPEC-0055).
34. Regeneration rate. Resolved: the test reads the remaining time T, which falls by 1 per tick in game-state code 0, so the bonus is 1 point per second (Passive bonuses, Heal rate; High).
35. Resolved (High): the state 1 idle point uses world X and world Z offsets; the state 2 follow point uses the hero's forward axis B3 (4.0) and side axis B1 (1.5); the state 3 vector is the camera's forward row and the reposition is the out-of-view catch-up of a bumped petpet; state 4 eats the nearest food in place; state 6 plays id 7 to its end and returns to state 0; state 5 is the camera-cone leave (Class 34; "Petpet build guide"). Earlier text kept for reference: class 34 state 1's nearby-character response, 1.0-unit walk threshold, 180-to-600-tick wait, random target offsets and remaining-time animation selection are specified (Class 34; High for the thresholds and update order, Medium-High for candidate and target details). The eligibility filter is the class-state-19 (agro) test (High); the two target-axis identities remain open. State 2's follow point and three-gait horizontal-distance controller are specified (Class 34; High); the basis-vector world-axis identities remain open. State 3's conditional camera-relative reposition after contact is now specified (Class 34; High for the gates and writes, Medium for the camera-vector interpretation); its broader purpose remains open. Further state 4 detail and state-6 selection interactions remain open. The pending-event word is written by the hero's completed feed interaction: it sets the live pet's word when feeding it, or sets the selected wild pet's word when feeding that pet; when a different wild pet is fed while a companion is live, the live pet's remaining time is set below zero so it leaves. The exact choice among close pet targets remains governed by the feeding path. The hero-attached-looking records are statue puzzle objects, not class 34 states (Resolved).
36. Whether the region-slot wild petpet copies are created in play. Resolved: the scripts of 1184, 1300, 1350, 1358 and 1414 load the regions that hold them; 706, 800 and 886 load theirs by story flag (flags 63 to 67, 55 to 58 and 52), 753 would load the type 2335 region but its gate flag 427 is never set, and 851 never loads its regions (Acquisition; High for the call sites).
37. Whether a wild petpet record comes back after its companion leaves. Resolved: yes; its record keeps no kill bit, so once no live petpet has its type a streaming block binds it again at the spot where it ate (Acquisition; Medium-High).
38. The petpet during the hero's death state. Resolved: see open question 33 (it stays).
39. The exact eat condition of class 34 state 4. Resolved: distance below 2.0 or timer below 2.5 s; the timer is read before the same tick's reservation, so the test always passes and the live petpet eats where it stands (Strike, step 3; High).
40. Whether the reservation fallback can take the food during a wild petpet's eat clip. Resolved: no; the eating step re-reserves the food every tick, which resets the timer, so the fallback runs only after 5 s with no eating step (Strike, step 5; High).
41. Resolved (Medium-High): the feed reads the raw quick-item selection marker of the active hero (SPEC-0021; the marker value, not its effective value). If the marker is an item id (0 or more), the food is that item, taken from the party inventory's selection lookup. If the marker is -1, the food is the first slot of the party's general inventory list (count 20, see SPEC-0021 Party inventory lists) whose item type is 12, searched in slot order. The dropped food is that item's id. Test vectors: marker 150 held: the food is 150 even if other food is held. Marker -1 with type-12 items in slots 3 and 7 (ids 149 and 153): the food is the item in slot 3 (149). The item-screen marker is the same marker (SPEC-0021), so there is no second source.
42. Resolved (High): the food id to word map is: 146 and 149 read the word at 0x2DB1E0 (5); 147 reads 0x2DB1F0 (10); 150 reads 0x2DB1DC (2); 152 reads 0x2DB1FC (60); 153 reads 0x2DB1F4 (10); 155 reads 0x2DB1F8 (30); 156 reads 0x2DB1E4 (10); 157 reads 0x2DB1EC (5). Ids 145, 148, 151 and 154 read no word (0). The word at 0x2DB1E8 (2) is read by no food id, so the table above is correct and the nine words are not all used. Test vector: food 150 gives 2 units, food 152 gives 60.
43. Resolved (High): predicate slot 38 tests the active petpet kind (SPEC-0035, section 9c). The current pet type maps to kinds 0, 1, 3, 6, 7 or 8; no live pet or an unmapped type returns -1. Argument 255 passes any mapped kind, otherwise the argument must equal the kind. The portrait group nodes use this predicate with their listed kind arguments. Slot 39 always fails, so nodes gated only by slot 39 are never shown by it.
44. Resolved (High): the petpet's regeneration gates read its remaining time T (the petpet word that falls by one each tick while the game-state code is 0), not a score word. The gate passes when the live petpet's kind is 6 or 7 (hit points) or 3 or 7 (magic), T is at least 1 and T modulo 60 is 0 (Regeneration test above). So a passing tick occurs once per second in play, and while the game-state code is 1 to 8 T stays fixed, so a multiple of 60 passes every tick of that stretch. SPEC-0033 Behavior 24 states the same gate. Test vector: T = 120 at game-state code 0 passes on that tick; T = 119 does not.

45. Resolved in part (High): the class 33 roam wait is always 60 ticks and the walk ends below 2.5 units, on a touch or on a failed point test (State 3, Roam). Resolved (High): the roam centre is the pet's own home point and the random point is a whole-number offset of -20 to 19 on X and Z with a halve-toward-the-pet retry (Roam point rule, State 3); the "reference object" is only the pet's navmesh polygon attachment.
46. Resolved (High): food selection validates only the nearest candidate point against the pet's current navmesh neighborhood. It does not test a line or ray between pet and food, and failure does not retry a farther candidate. The separate eat gate is distance below 2 and facing dot above 0.8.
47. Resolved (High): the Feed petpet caption uses the standard Square caption draw with the shared default colour (160, 160, 90, 127), so its alpha is 127. The Square shown counter is reasserted on each eligible chooser pass and consumed by one draw. It is suppressed while the quick-item or Mote panel is open. This caption path is separate from the petpet meter's alpha and low-time flashing.
48. Resolved: they are unrelated. The four directional entries are the quick-item ring (SPEC-0021, Quick-item panel); the three equipment-count slots show the equipped weapon, shield and armour Motes and their counts (SPEC-0021, Equipment slots; SPEC-0023 section 4). They share no data, and the petpet feed uses only the ring's selection marker (Question 41) (High).
