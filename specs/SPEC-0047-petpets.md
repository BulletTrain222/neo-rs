# SPEC-0047: Petpets (companion creatures)

- Answers: proactive, none requested (companion creature); R-0035 (in part: world changes, the leave rule, the intake rounding and open questions 41 to 44)
- Status: Draft
- Disc build: SCUS-97367 v1.00 (same fingerprint as SPEC-0001). The data words below are in the boot program's data and are read by virtual address. Values were checked against the disc image.
- Confidence: per section. Kind table and hit-base values: High (words checked). Membership sets: High (words checked). Package numbers and package pet ids: Medium-High. Acquisition (joining by feeding a wild petpet) and scene-start restore: High. Multipliers: Medium (code constants, not data). State selection: High for the rules, Medium for the roam and hover details. Strike and score: Medium; the 3600 factor in the score (tick rate 60 x 60 seconds) is High. Spawn and package rules: Medium. Save fields 0x214, 0x218 and 0x21C: High for the offsets (owned by SPEC-0025), High for the no-pet value -1. Second kind table: Medium. Script query presence gate: High. Class 20 (second petpet behaviour): High for the four states and the choice rules; Medium for the hero reaction, the object consumption and the effects; the item test is Medium-Low. Hero-attached companion follow (section "Hero-attached companion follow"): Low-Medium; its owner, SPEC-0047, is low-medium, inferred from behaviour only. Behaviour classes: High that the summoned petpet is requested into class 34 and that class 33 is the row default of the placed pet rows; class 34 controller: High for its structure and the remaining-time countdown, Medium for the state bodies. Passive bonuses: High for the kind tests, the four data words and the per-tick test; High for which hero products the two damage factors enter (SPEC-0036); High for the heal rate of 1 point per second; Medium-High for the freeze quirk.

## Summary

A petpet is a companion creature that belongs to the hero. Each hero has at most one active petpet, and the petpet follows the active hero. The petpet's kind is a number from 0 to 9, read from its pet type id. Level scripts can read that kind through the host query GetPlayerPetPetType (SPEC-0030, row 199).

A petpet joins the hero by being fed. Wild petpets are placed characters of the petpet kinds in the world packs; when the hero feeds one a food item, it eats the food and becomes the hero's live petpet (see "Acquisition (joining)"). A petpet also becomes active when a scene starts after a save was loaded whose pet type field holds a pet type id. A fresh game starts with no petpet.

The petpet keeps a loose distance to its hero. When it is 10 or more units away it roams to a random point within 40 units of the hero. When it is within 10 units of the hero and, after it has been fed, a food item is within 10 units, it goes for it (state 4, Strike). It eats once its eat animation ends; a 5 s fallback timer applies the food only when no petpet is in its eating step. The hit value is a base value set by the target's type code, times a multiplier set by the petpet kind. The type codes are food item ids, and the hit value is the food's value in minutes (see "Hit value and score"). The petpet keeps a remaining companion time T in ticks and the full time F set at its last meal. T and F are saved in the save slot (SPEC-0025, offsets 0x218 and 0x21C) and restored on load.

The active petpet also gives its hero passive bonuses by kind: a 1.25 factor on a hero damage product (kinds 0 and 7), a 0.9 factor on the hero's damage-intake product (kinds 1 and 7), extra hit points (kinds 6 and 7) and extra magic (kinds 3 and 7). See "Passive bonuses to the hero".

The summoned (live) petpet runs behaviour class 34, a seven-state controller that stays until its remaining time T falls below 1 and then leaves. Class 33, whose six-state loop the State selection, States and Strike sections describe, is the default class of the placed pet rows. See "Class 34 (summoned petpet controller)".

A second petpet behaviour (class 20) reacts to the hero, approaches and consumes nearby world objects, and wanders. It is described in its own section below.

## Where it lives on the disc

- Behaviour classes. The summoned petpet is requested into behaviour class 34 at priority 2 when it spawns (High). Class 33 is the default class of the placed pet rows 244, 245, 247, 250 and 251 of the monster table (resource 530, SPEC-0029, Layout 4); the six-state loop described under State selection and States is class 33. The class table is 40 consecutive words at virtual address 0x312C10 (word index = class id, see SPEC-0029, Layout 3). The clean side does not copy the table entries; it implements the rules below.
- The second petpet behaviour (class 20) is word 20 of the same table; see the class 20 section.
- Base hit values: nine 32-bit float words, 4 bytes apart, from virtual address 0x2DB1DC to 0x2DB1FC (addresses 0x2DB1DC, 0x2DB1E0, 0x2DB1E4, 0x2DB1E8, 0x2DB1EC, 0x2DB1F0, 0x2DB1F4, 0x2DB1F8, 0x2DB1FC). Their values are 2.0, 5.0, 10.0, 2.0, 5.0, 10.0, 10.0, 30.0, 60.0 in that order. Read them at runtime; do not copy them into the clean code. Which word belongs to which target type code is open (open question 4).
- Membership sets: three arrays of eight 32-bit signed integers, one per set. Set A starts at virtual address 0x315C88, set C at 0x315CA8 and set B at 0x315CC8. An entry of -1 is unused. Contents are in Layout.
- Summoned petpet low-time mark: the float word at virtual address 0x2DB1D8 (value 0.5, in minutes), read by class 34; 0.5 x 60 x 60 = 1,800 ticks (High). It is not a lifetime. Whether the 0.5 hit multiplier also reads this word is open (open question 12). The 0.75 and 1.5 multipliers are not data words; they are constants of the behaviour (see Layout).
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

One character-eligibility gate uses the second-table check as a rejection condition: if the character's type id maps to any of these four kinds, that gate returns false. For example, a character with type id 805 (kind 2) fails the gate. The higher-level action that consumes this eligibility result is not established, and this does not identify all users of the primary table.

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

Medium: the values are the nine words listed under Where it lives; the type-to-word assignment is open.

### Kind multipliers

| Kind | Multiplier 1.5 | Multiplier 0.75 | Multiplier 0.5 | Otherwise |
| --- | --- | --- | --- | --- |
| 0 | target in A or C | - | - | 1.0 |
| 1 | target in B or C | - | - | 1.0 |
| 3 | target in B or C | target in A or C | - | 1.0 |
| 6 | target in A or C | target in B or C | - | 1.0 |
| 7 | - | - | target in A, B or C | 1.0 |
| 2, 4, 5, 8, 9 and others | - | - | - | 1.0 |

The multipliers 0.75 and 1.5 are constants of the behaviour (Medium). The 0.5 multiplier may be read from a data word (open question 12).

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

Times are in ticks at 60 ticks per second (SPEC-0007). Distances are world units, measured on the horizontal plane between the petpet and the reference point (the hero, or the anchor or target named below). The distance measure is Medium.

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

1. If the petpet is 10 or more units from its hero, it goes to roam (state 3), unless it is already in strike (4) or hover (5). Those two keep their state. (A test for state 2 is made first on this branch and always fails.)
2. Otherwise (within 10 units of the hero):
   - if a non-anchor target is within 10 units of the petpet, it goes to strike (4);
   - else if an anchor (type code 145) is within 5 units of the petpet, it goes to hover (5);
   - else it goes to idle-with-animation (1), unless it is already in state 1 or 4.

The selector is High for these rules. The roam threshold (10) is the "return" rule: a petpet that wanders beyond 10 units comes back toward the hero by roaming to a point near it.

### States

- **0, Idle.** Plays animation 0 each tick.
- **1, Idle with one-shot.** Plays animation 9 once. When it finishes, the petpet is idle (animation 0). The state keeps its place until the selector changes it (Medium).
- **2, Idle.** Plays animation 0, as state 0. The selector never chooses state 2: its test for state 2 always returns false (High).
- **3, Roam.** If no roam point is chosen, pick a random point within 40 units of the hero and draw a wait counter (in ticks; the draw range is open). While the wait counter is above zero, play animation 0 and count it down. When the wait ends, walk to the point. On arrival, clear the point and draw a new wait.
- **4, Strike.** See "Strike" below.
- **5, Hover.** Move to a point about 10 units from the anchor and hold there. The rule for choosing a new hover point after arrival is not recorded (Medium-Low).

### Strike

Confidence: High for the steps, the eat tests and the reservation rule.

There is one shared food reservation: a reserved food index (or -1), a timer in seconds and a slow flag. Reserving a food sets the index, and sets the timer to 0 and the slow flag to 0.

1. Each tick in state 4 the petpet finds the nearest world item of item category 12. If there is none, or it is the Bagguss (145), nothing more happens that tick. Otherwise it reserves that food (so the timer is reset every tick while the step runs). Anchors are never eaten.
2. Wild petpet (class 33), eat test: it eats when it faces the food (dot of its facing with the unit direction to the food above 0.8) and the food is less than 2.0 units away. Its timer test (more than 2.5 s) reads the timer after the reservation of the same tick, so it is 0 and never passes. Otherwise, when the food is 2.0 or more units away the petpet paths toward it (only when it can reach it); when it is nearer but not facing it, it turns toward it (turn rate 0.1).
3. Live petpet (class 34 state 4), eat test: it eats when the food is less than 2.0 units away or the timer is below 2.5 s. Its timer is read before the reservation of the same tick, so it holds at most one tick's step and the test always passes: the live petpet eats at once wherever it stands; its walk toward the food (speed 0.3) is never used.
4. Eating: the petpet requests animation 4; when the clip has ended, the food's value is applied (Hit value and score), the food is removed and the reservation is cleared (-1). A wild petpet then calls the activation by its own id and becomes the hero's live petpet (Activation by id, below). A live petpet also clears its low-time latch.
5. Fallback timer (main loop, once per tick, while a food is reserved): the timer grows by the tick length 1/60 s, or by 1/180 s while the slow flag is set (a class 20 creature sets it in its object-seek step; a reservation clears it). When it passes 5.0 s: the timer and slow flag are cleared, the reserved food is removed from the world, the live petpet (when present, T at least 1) gets the food's value with its own kind multiplier as in step 4, and the reservation is cleared. Because every eating step re-reserves the food each tick, the fallback only runs after 5 s in which no petpet ran its eating step, for example food dropped with no petpet going for it. A wild petpet in its eat clip cannot lose the food to the fallback.

### Hit value and score

The targets of state 4 are food items: the type codes are item ids 145 to 157 (SPEC-0021), the anchor 145 is the Bagguss, which is never eaten, and a wild petpet enters state 4 only after it has been fed (the fed flag is set) and the nearest food item other than 145 is within 10 units and reachable (High). The hit value is the food's value in minutes, and the score candidate is the remaining time it gives, in ticks.

For a target of type code t and a petpet of kind k:

1. base = the base value of t from the table above. For t = 145 (anchor), the hit is 0.
2. multiplier = the kind multiplier from the table above. Kinds without a rule use 1.0.
3. hit value = base x multiplier.
4. score candidate = floor(hit value x 3600). The factor 3600 is the tick rate (the run-time value 60 of SPEC-0007) times 60 seconds per minute, because hit values are in minutes (High; open question 2a).
5. Time update: eating food of value v (the base value of the food id times the kind multiplier, in minutes) sets T and F to trunc(v x 60 x 60) when that is larger than T (High). Times are integers. A candidate of 0 never replaces T.

T then falls by 1 per tick (Class 34), so T and F differ in play: F is the full time of the last meal, and the HUD petpet meter is round(60 x T / F) pixels (SPEC-0023) (High). The field is shared with other creatures (open question 9).

### Spawn

- A fresh spawn (the activation's id argument is -1) places the petpet 7 units ahead of the hero, along the hero's facing direction (Medium).
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
- If the flag is set, the routine also resets a hero flag (its use for the petpet is not recorded). Then it reads the pending pet type, which is also the third value passed to the activation call (High):
  - if the pending type is below 0 (signed 32-bit), the current petpet is deactivated;
  - otherwise the petpet is activated by type with the pending type as its type value and -1 as its id value. The spawn is fresh, so the pending T and F are copied and the pending values are cleared (High for the argument order; Medium for the rest).

Pending values stay as loaded when the no-pet path runs (they are not cleared).

### Save

The save writes the live petpet's type id at 0x214, T at 0x218 and F at 0x21C. With no petpet, or T below 1, the type is written as -1 (0xFFFFFFFF). On load a type of -1 removes any petpet at scene start; a valid type spawns it 7 units in front of the hero with T and F from the slot. (High)

### Area changes, resets and hero swaps

- Level or game teardown and mode switches deactivate the petpet (Medium).
- Every TriggerActStart call and the world reset remove the live petpet (High).
- A character helper also deactivates the petpet when the hero is far away or a flag is set (Medium-Low; the condition is not fully described).
- A hero swap or reset moves the petpet's owner to the active hero without spawning it. A switch to another kind is a deactivate followed by an activation; there is no separate switch path (Medium).

### Script query

GetPlayerPetPetType (SPEC-0030, row 199) returns the kind of the active petpet. It returns -1 when there is no active petpet, and when the type is not in the primary table (for example a second-table id such as 805 or 2335). It applies the presence gate: it returns -1 when the petpet's remaining time T is below 1 (High). Two level scripts read the kind to choose camera, conversation or message lines (Medium). Both test kinds 1, 3, 6 and 7 (one also tests kind 0); they see -1 until a petpet has joined (Medium).

### Level-script pet branches

These branches are in the level scripts (SPEC-0045 owns the scripts). They read the pet-kind query above and choose a line or a scene. They are the only in-game readers of the kind that were found.

Cinthia conversation (level pack 758):
- The first visit sets flag 539 when flags 539 and 540 are both clear. It shows no pet line (Medium; the line shown is not identified).
- On later visits, while flag 539 is set and flag 540 is clear, the conversation reads the active pet kind:
  - Kind 6 sets flag 540 and gives a thank-you line. This completes her request.
  - Kinds 0, 1, 3 and 7 each give their own refusal line. No flag changes.
  - Any other value, including -1 when no pet is active, gives a waiting line. Hero 0 and hero 1 get different waiting lines.
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

1. If the pet's body touched something this tick and the pet is not leaving: state 3 (bumped).
2. Else, if the pet's pending-event word is set: state 4, and the word is cleared.
3. Else, if the pet is not in state 4 or 6:
   - when T is at least the low-time mark, or the low-time state has already run since the last meal: if T < 1, state 5 (leave); otherwise state 1 (follow, with a new target near the hero) when the hero is idle, else state 2;
   - when T is below the low-time mark and the low-time state has not run since the last meal: state 6, once.

State bodies (Medium):

- **0**: plays animation 0.
- **1, idle-hero follow and nearby-character response:** the pet scans character entries in order for the first eligible character within 10.0 horizontal units whose id differs from its last recorded response. On accepting a new character it records that id and requests animation 8; while animation 8 is current it waits, and after the clip finishes it requests animation 0. The eligibility filter's meaning is open. When no new character response takes priority, the pet moves toward its stored point while horizontal distance is at least 1.0, aiming toward it with weight 0.1 and requesting animation 2; this also clears the wait timer. Below 1.0, an unset timer is drawn from 180 through 600 ticks and animation 0 is requested. On following updates the timer decreases by one; when it drops below 1, the pet chooses a new point and unsets the timer. Each new point is the hero's current position offset on two horizontal axes by independent random magnitudes from 3.0 to 10.0 and independent random signs; the axis identities are open. During an active wait, animation selection uses r = T / 1,800: if r < 1, request animation 7; otherwise draw integer q from 0 through 100 and request animation 1 when q > 200 × |1 − r|, or animation 6 otherwise. T is decremented after the state body on that tick. (High for the thresholds and update order; Medium-High for the candidate-filter and random-target details.)
- **2, moving follow:** chosen instead of state 1 while the hero is in the movement/jump predicate. The target is the hero's position plus 4.0 units along one hero basis vector and 1.5 along another; which basis vectors these are remains open. The pet measures horizontal distance to this point and uses a three-gait hysteresis rule. Gait 0 uses animation 0 at distance 3.0 or less; above 3.0 it uses animation 2 and changes to gait 1. Gait 1 uses animation 3 and changes to gait 2 above 7.0; below 1.5 it uses animation 0 and returns to gait 0; from 1.5 through 7.0 it uses animation 2 and stays in gait 1. Gait 2 uses animation 2 and returns to gait 1 below 5.0; at 5.0 or more it uses animation 3 and stays in gait 2. Gaits 1 and 2 also aim toward the target with weight 0.1 each tick; gait 0 does not run this aim update. (High for the distances, transitions and aim call; the world-axis identity of the two hero basis vectors remains open.)
- **3, bumped:** requests animation 0 each tick. The state remains 3 unless both recovery gates pass: the dot product between the camera's view vector and the normalized direction from the camera position to the pet is below 0.5, and the ground height at a candidate point is less than 2.0 units from the hero's current height. The candidate point is the hero's position plus 10 units along the normalized horizontal negative camera-view vector; the ground query samples from 8 units above that point. On success the pet is placed at the candidate facing horizontally toward the hero and state 1 is selected. The world-axis interpretation of the camera vector and the broader purpose of this recovery remain open. (High for the comparisons and position/facing writes; Medium for the vector's camera-relative role.)
- **4**: entered when the pending-event word is set; the petpet eats the nearest food at once, where it stands (Strike, step 3); eating here resets the low-time state.
- **5, leave**: the pet is removed when it faces away from the hero (facing dot below 0.5) or touches something.
- **6, low time**: plays animation 7 once.

So the petpet stays until T falls below 1 and then leaves (state 5, removed). The first time T drops below 1,800 after a meal, it plays animation 7 once (High).

### Class 20 (second petpet behaviour)

Class 20 is a second petpet behaviour. It has four state values: 0 (idle), 1 (hero reaction), 2 (approach and consume a nearby object) and 3 (wander). Each tick the state is chosen first, then the chosen state's rules run. Confidence: High for the states and the choice rules; Medium for the effects.

Choice rules (each tick):
- A state is re-chosen only while it is 0, 2 or 3. State 1 runs to its end.
- If the item test passes, the state becomes 1 and the sub-step becomes 0.
- Else, if the state is not 2 and a non-anchor world object is within 25 units, the state becomes 2 and the latch is cleared.
- Else, if the state is not 3, the state becomes 3 and the sub-step becomes 1.
- While the state is 2, the object rule does not restart it; only the state 2 routine ends the state (Medium).

Item test (Medium-Low): reads three checks on the hero. The first is the hero's attack-state query. The third is the shared reach/contact test, which first applies a 65.0-unit range gate and then checks character bone shapes. The second calls an accessor that returns a word from its receiver; its exact relation to the hero's attack-window field remains uncertain. If all three checks are non-zero, a short message is shown at the hero's position and the test passes. Otherwise it scans the hero's six shape slots: if one of the pet's shapes overlaps an occupied slot, that slot is cleared and the test passes. The contents of those slots and the meaning of the second check remain open.

State 0 (idle): plays animation 0 each tick.

State 1 (hero reaction): three sub-steps.
- Sub-step 0: plays animation 14 once. When it finishes, draws a wait of 3 to 4 seconds at random (180 to 240 ticks).
- Sub-step 1: plays animation 15 for the wait, one count per tick. Then goes to sub-step 2.
- Sub-step 2: plays animation 16 once. The hero check is made once, on the first tick of animation 16, and the state's latch is set whether or not the check passes. The check passes when all three hold: the pet is within 2.5 units of the hero; the vertical separation between the hero and the pet is more than 0.5 units; and the hero's current animation id is not 7. On a pass: the hero's effect position is nudged by 8.0 on one coordinate (which coordinate is open), a hero method is called with the value 7 (its purpose is open), and a hero flag is set. When animation 16 finishes, the state becomes 0.
- During sub-steps 1 and 2 a small effect is placed on the hero each tick (Low-Medium; the visual is not described).

State 2 (approach and consume): each tick, while the pet is not yet latched:
- Finds the nearest non-anchor world object within 25 units. If there is none, or the target is not valid (the stored target has moved more than 2.0 units from its stored point; Medium), the state becomes 3.
- If the object is 2.5 or more units away, the pet moves toward it and stays in state 2.
- If it is within 2.5 units, the pet latches and takes the object's type code. The matching world-pickup record has its attached resources detached and released, then is reset; this is active-object cleanup, not a visibility-only change. The pool record itself is not shown to be deallocated, and its later reuse is open (Medium-High for cleanup; Low for slot lifecycle). It stays in state 2 that tick.
- Once latched, the pet plays animation 4; when that finishes, the state becomes 3 with sub-step 0 and a wait of 4 to 5 seconds at random.

State 3 (wander): sub-step 0 plays animation 0 and counts the wait down; at 0 it picks a random point within 40 units of the pet's reference object and sets sub-step 1. Sub-step 1 walks to the point; within 5.0 units it draws a new wait of 4 to 5 seconds and returns to sub-step 0. The reference object is set at start-up (Medium).

Constants (seconds at 60 ticks per second): state 1 first wait 3.0 to 4.0 (3.0 at virtual address 0x2DB290, 4.0 at 0x2DB294); hero effect nudge 8.0 (0x2DB298); the wait after state 2 and each state 3 wait 4.0 to 5.0 (0x2DB2AC and 0x2DB2B0). Other values are fixed constants: 25 units (object range), 2.5 units (latch and hero check distance), 0.5 units (height separation), 2.0 units (target validity), 40 units (wander radius), 5.0 units (wander arrival). The class 20 values are taken from the disc; the constants are as the behaviour uses them.

Edge cases:
- A state entered at sub-step 1 (from the choice rules) walks to the point stored in the state record before it picks a new one. The first wander after start-up may walk to a stored point that was never set (Low; open question 21).
- State 2 with an anchor as the nearest candidate stays in state 2 (Low).
- The item test reads the hero's six shape slots; a slot holding a shape that the pet overlaps is cleared, so the item test can consume a hero-side shape (Medium-Low; open question 18).

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

### Hero-attached companion follow (owner low-medium)

Confidence: Low-Medium overall. The owner, SPEC-0047, is inferred from behaviour only (Open question 23). The companion stays with the active hero. The state numbers in this section belong to this companion; they are not the petpet states listed above, and the overlap is open (Open question 23).

States covered: 1, 2 and 5. The other states were not described.

- **State 1, align.** The companion turns its facing to match the hero's facing. The alignment is complete when the facing dot product passes 0.99.
- **State 2, turn.** The companion turns toward a target by a fixed step each tick. The step size and the target were not recorded (Open question 26).
- **State 5, follow.** Each tick, the companion forms a target point 1.5 units above the hero and the direction from its current position to that point. When the current facing direction has a dot product with the target direction greater than 0.5 and less than 0.99, it blends 0.9 of its current facing direction with 0.1 of the target direction, then normalizes the result. It advances 0.25 units along its current facing direction each tick; outside that dot-product interval, this step uses the existing facing direction without the blend. A counter counts ticks in this state. At 200, the companion sends animation 1 to the hero together with an idle-state value (Open question 27). Confidence: Medium-High for the recovered state-5 calculation; the companion's identity remains open (Open question 23).
- **Follow timer.** When the timer-enable field is nonzero, each update subtracts the selected speed value multiplied by 0.008333334 (approximately 1/120) from the timer. This calculation uses a fixed factor; the equal-valued boot-image word at virtual address 0x3245B4 is not read by this update. The timer's purpose and the selected speed value's runtime units remain open (Open question 25).

Test vector (synthetic): in state 5, let the companion face +X and let the target point 1.5 units above the hero lie along +Y from the companion. The normalized target direction is +Y and the dot product is 0, so the blend gate does not run. The companion advances 0.25 units along +X for that tick. If the target direction instead has dot product 0.75 with the current facing, one blend step forms normalize(0.9 × currentFacing + 0.1 × targetDirection), then advances 0.25 units along that normalized direction.

## Edge cases and error handling

- A petpet whose remaining time T is below 1 reads as "no pet" to the script query, the passive bonuses, the save and the HUD meter (High).
- A save made with no pet, or with T below 1, stores -1 in the type field (High). Loading that value removes any petpet at scene start. See "Save".
- A saved type that appears only in the second table (805, 1350, 1368 or 2335) gives no kind on load, so nothing spawns (Medium).
- A kind 8 petpet saves as 2090 and loads back as kind 7 (Medium; open question 10).
- Pending values are one-shot. The first spawn that takes them clears them. A no-pet load leaves them as loaded.
- A kind outside the tables (for example pet type 4660) gives no kind. Activation by type then does nothing.
- Anchor type 145 is never a strike target and always gives a hit of 0.
- Path B of the strike runs at one third speed when its second flag is set (Medium-Low).
- Feeding another wild petpet while one is live sets the live petpet's T to -1; it leaves, and the script query returns -1 until the fed one joins (High).
- A wild petpet that has eaten is removed from its world list and the live petpet is created at its place (Acquisition (joining)).
- Class 20: a state entered at sub-step 1 walks to a stored point before picking a new one (see the class 20 section; open question 21).
- Class 20: an anchor as the nearest object keeps the pet in state 2 (Low).
- Class 20: the item test's slot clearing consumes a hero-side shape (Medium-Low; open question 18).
- Passive bonuses: the petpet hit point is not gated on the hero being alive or unblocked, so it is added on ticks where the item-based regeneration is gated off (Medium; reproduce).
- Passive bonuses: with no live petpet, or T below 1, there is no kind and nothing is added.
- Passive bonuses: a reloaded kind 8 petpet is kind 7 and gains all four bonuses.
- Passive bonuses: a live petpet whose T was set to -1 gives none of the four bonuses.
- Feeding (hero state 49) with no candidate in range after the clip: no fed flag is set, and the hero goes to state 1 after the gate (Medium).

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
27. Class 20, hero check (sub-step 2 of state 1): pet 2.0 units from the hero, vertical separation 0.6, the hero's animation id 3: the check passes. The same pet with the hero's animation id 7: the check fails. Vertical separation exactly 0.5 with animation id 3: the check fails (the separation must be more than 0.5).
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

## Open questions

1. Origin of the first petpet. Resolved: a petpet joins in normal play when the hero feeds a wild petpet (Acquisition (joining); High).
2. Resolved: the "creature flag 0x22" set on spawn is the request of behaviour class 34 at priority 2 (0x22 = 34), not a flag (High). Still open: the object-effect call used by strike and class 20.
2a. Resolved: the score and the remaining time are trunc(v x tick rate x 60), with the tick rate the run-time value 60 (SPEC-0007) and 60 the seconds per minute, so 3600 = 60 x 60 (High).
3. Partly resolved: one character-eligibility gate calls the second-table check and rejects a character whose type id maps to kind 2, 4, 5 or 9 (e.g. type id 805 / kind 2; Second kind table). Still open: the higher-level action using this gate and the broader callers and effects of the primary-table check.
4. Class of placed pet characters. Resolved: class 33, the row default of the placed pet rows 244, 245, 247, 250 and 251; the summoned petpet is requested into class 34 (High).
5. What sets the live petpet's time field to -1. Resolved: feeding another wild petpet while one is live (Feeding a petpet; High).
6. Resolved in part: kind 8 (type 2094) placements run class 20; they never call the activation, so they cannot join this way (High; "Feeding a petpet"). The specific creature or species identity represented by kind 8 is not established.
7. Resolved: the live petpet's runtime type record is character record ordinal 0 from the loaded package; each package has one kind-3 character record, so this is its first and only character record (High; Packages by kind).
8. Whether any path sets state 2 for the petpet. Resolved: no; the class 33 test for state 2 always fails, so state 2 (idle, animation 0) is never chosen (State selection; High).
9. Use of the remaining-time field outside the petpet. Other creatures read and write the same field.
10. Resolved in part: the activation by id after a landed hit is the wild petpet joining, not the live petpet re-creating itself (Activation by id; High). Still open: whether kind 8 is intentionally saved as 2090 (package 542 carries 2090, not 2094).
11. The identity of the anchor type 145 (the owning character or a companion). Not confirmed.
12. The 0.5 multiplier: whether it is read from the word at 0x2DB1D8 (value 0.5) or is a code constant. The word is read by class 34 as the petpet's low-time mark in minutes (High); whether the multiplier also reads it is open.
13. Argument of the script query's gate. Resolved: the query applies the presence gate, so it returns -1 when T is below 1 (Script query; High).
14. Whether the petpet has its own row in the monster table (resource 530, SPEC-0029, Layout 4). None was found: the petpet code does not read that table. Medium-Low.
15. Resolved: pet activation loads archive resource 534 plus the kind and instantiates character record ordinal 0 from that package (High; Activation by id and Packages by kind).
16. Score-below-1 gate of the script query. Resolved: the gate applies; T below 1 gives -1 (Script query; High).
17. The per-tick step of the fallback strike timer. Resolved: the tick length 1/60 s, or 1/180 s while the slow flag set by a class 20 creature is on; every reservation resets the timer and the flag (Strike, step 5; High).
18. Partly answered: the first item-test check is the hero's attack-state query, and the third is the shared reach/contact test with a 65.0-unit range gate followed by character bone-shape checks. The second calls an accessor that returns a word from its receiver; its exact relation to the hero's attack-window field remains uncertain. What the six hero-side shape slots hold (for example, projectiles or carried objects), and the second check's exact role, remain open. Medium-Low.
19. Which coordinate of the hero's effect position the 8.0 nudge (class 20, state 1) changes. Low.
20. The class-20 state-2 pickup helper detaches/releases the matched record's attached resources and resets the record; it does not establish pool-slot deallocation. What later owns or reuses that reset slot is open. Medium-High for cleanup; Low for slot lifecycle.
21. Whether the first class 20 wander walks to an unset point. Low.
22. What the hero method called with the value 7 (class 20, state 1) does, and what the hero flag it sets is used for. Low.
23. Owner of the hero-attached follow: whether the companion of states 1, 2 and 5 is the petpet (same kind and save fields) or a separate companion. Its state numbers overlap with the petpet states, where state 1 and state 5 mean other things. Low-Medium.
24. Resolved: the 0.9/0.1 blend is between the current facing direction and the normalized direction toward the point 1.5 units above the hero; position advances 0.25 units along the facing direction. The blend runs only when their dot product is strictly between 0.5 and 0.99. Medium-High.
25. Partly answered: when enabled, the timer decreases by the selected speed value times 1/120 per update, using a fixed factor rather than reading the equal-valued boot word at 0x3245B4. The timer's purpose and the runtime meaning and range of the selected speed value remain open.
26. State 2 target and step of the hero-attached companion: the turn's target and fixed step were not recorded.
27. Animation 1 message: which hero message receives animation 1 at the 200-tick mark, and what idle state value it carries.
28. The use of the scene-start hero flag that the scene start restore resets (Behavior, Scene start).
29. Resolved: item 126 (the Tattered Cloak) has no PetPet effect. The item-screen and quick-panel use paths share the dispatcher, which refuses the cloak without consuming it; the item-screen Square action equips it instead. Host row 353 invokes the same use path. The separate petpet-feeding interaction uses category-12 food. The only PetPet-specific host API is a read-only kind query, and none of the recovered item-use callbacks calls it (High).
30. Writers of flags 539, 540 and 594. The level scripts set 539 and 540 only in the Cinthia conversation, and only the trigger room itself sets 594: its per-tick update sets it when the ritual fade completes (5.0 s after the trigger starts the ritual). Whether another writer exists outside the level scripts (boot program, front end or host layer) is not established. Whether flag 539 is ever set before the first Cinthia visit is also open; that decides whether the waiting lines can show at all.
31. The item-use name table. The boot program's data holds a four-entry table of names for item-use categories (food, potion, cloak and a failure case). A direct search found no reference to it beyond the table itself. Whether the item-use routine reads it through a computed base is not known, so the role of the cloak entry is open (see open question 29).
32. Passive damage factors. Answered: the 1.25 factor is factor E of the hero's outgoing damage (SPEC-0036, Damage), and the 0.9 factor applies to enemy hits taken by the hero, after the shield block and the armour terms (SPEC-0036, Enemy attack damage to a hero).
33. Whether the petpet stays while the hero is in the death state. Resolved: it stays; nothing on the death path removes it and its state choice has no health test (Passive bonuses; High). The point is still added, but the death state has no health exit (Medium; SPEC-0055).
34. Regeneration rate. Resolved: the test reads the remaining time T, which falls by 1 per tick in game-state code 0, so the bonus is 1 point per second (Passive bonuses, Heal rate; High).
35. Partly answered: class 34 state 1's nearby-character response, 1.0-unit walk threshold, 180-to-600-tick wait, random target offsets and remaining-time animation selection are specified (Class 34; High for the thresholds and update order, Medium-High for candidate and target details). The eligibility filter meaning and the two target-axis identities remain open. State 2's follow point and three-gait horizontal-distance controller are specified (Class 34; High); the basis-vector world-axis identities remain open. State 3's conditional camera-relative reposition after contact is now specified (Class 34; High for the gates and writes, Medium for the camera-vector interpretation); its broader purpose remains open. Further state 4 detail and state-6 selection interactions remain open; the pending-event writer is unknown. Whether the hero-attached companion follow section describes some of the class 34 states remains open.
36. Whether the region-slot wild petpet copies are created in play. Resolved: the scripts of 1184, 1300, 1350, 1358 and 1414 load the regions that hold them; 706, 800 and 886 load theirs by story flag (flags 63 to 67, 55 to 58 and 52), 753 would load the type 2335 region but its gate flag 427 is never set, and 851 never loads its regions (Acquisition; High for the call sites).
37. Whether a wild petpet record comes back after its companion leaves. Resolved: yes; its record keeps no kill bit, so once no live petpet has its type a streaming block binds it again at the spot where it ate (Acquisition; Medium-High).
38. The petpet during the hero's death state. Resolved: see open question 33 (it stays).
39. The exact eat condition of class 34 state 4. Resolved: distance below 2.0 or timer below 2.5 s; the timer is read before the same tick's reservation, so the test always passes and the live petpet eats where it stands (Strike, step 3; High).
40. Whether the reservation fallback can take the food during a wild petpet's eat clip. Resolved: no; the eating step re-reserves the food every tick, which resets the timer, so the fallback runs only after 5 s with no eating step (Strike, step 5; High).
41. Resolved (Medium-High): the feed reads the raw quick-item selection marker of the active hero (SPEC-0021; the marker value, not its effective value). If the marker is an item id (0 or more), the food is that item, taken from the party inventory's selection lookup. If the marker is -1, the food is the first slot of the party's general inventory list (count 20, see SPEC-0021 Party inventory lists) whose item type is 12, searched in slot order. The dropped food is that item's id. Test vectors: marker 150 held: the food is 150 even if other food is held. Marker -1 with type-12 items in slots 3 and 7 (ids 149 and 153): the food is the item in slot 3 (149). The item-screen marker is the same marker (SPEC-0021), so there is no second source.
42. Resolved (High): the food id to word map is: 146 and 149 read the word at 0x2DB1E0 (5); 147 reads 0x2DB1F0 (10); 150 reads 0x2DB1DC (2); 152 reads 0x2DB1FC (60); 153 reads 0x2DB1F4 (10); 155 reads 0x2DB1F8 (30); 156 reads 0x2DB1E4 (10); 157 reads 0x2DB1EC (5). Ids 145, 148, 151 and 154 read no word (0). The word at 0x2DB1E8 (2) is read by no food id, so the table above is correct and the nine words are not all used. Test vector: food 150 gives 2 units, food 152 gives 60.
43. Resolved (High): the HUD petpet panel gates are not petpet tests. Predicate slot 38 tests the current music track's index (SPEC-0035, section 9c): the panel's nodes are shown for the music tracks whose index is 0, 1, 3, 6 or 7 (the argument), or for any track with an index (argument 255). Slot 39, used by the nodes under the meter, always fails, so its nodes are never shown by it. The two slots read no petpet kind, so the panel's visibility does not follow the live petpet's kind (High for the slots; the other gates of the panel are not described here).
44. Resolved (High): the petpet's regeneration gates read its remaining time T (the petpet word that falls by one each tick while the game-state code is 0), not a score word. The gate passes when the live petpet's kind is 6 or 7 (hit points) or 3 or 7 (magic), T is at least 1 and T modulo 60 is 0 (Regeneration test above). So a passing tick occurs once per second in play, and while the game-state code is 1 to 8 T stays fixed, so a multiple of 60 passes every tick of that stretch. SPEC-0033 Behavior 24 describes a different gate and does not apply to the petpet's point. Test vector: T = 120 at game-state code 0 passes on that tick; T = 119 does not.
