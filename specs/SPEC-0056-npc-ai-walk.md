# SPEC-0056: NPC AI modes, walk-to-point routines and proximity items

- Answers: none (new topic; the script calls that drive these routines are specified in SPEC-0030); R-0079 (the block contact pass for region characters: place in the tick and response); R-0088 (navigation containment and fresh-block arrival as used by the Ixi Chief); R-0127 (where a path comes from, how the path walk starts and ends); proactive (path walkers: every way a non-enemy character walks, walking speeds, walkers on the disc, port checklist); proactive (NPC pass: the NPC build guide, hero contact, hit and status rules for townsfolk, and the non-fighting creatures); proactive (placement-facing survey and NPC orientation lifecycle); proactive (timing pass); proactive (verify pass: placement facing write-back, bin convention); proactive (verify pass: facing decode, runtime overrides, port faults and test vectors)
- Status: Draft
- Disc build: SCUS-97367 v1.00 (boot program and level modules; no fingerprint check is stated in this spec, see SPEC-0001)
- Confidence: per section. Mode numbers: High. Mode names: labels only (the number-to-name map is not established). Townsfolk class, the mode-from-code rule and the per-character values: High. Walk and path arrival distances: High. All distances are horizontal (ground-plane) distances: High for the walk, hold, greeting and villager tests, Medium-High for the navigation arrival test. Steering weights and final orientation: High. Hold branch and greeting: High. Villager timer ranges: High in ticks and in seconds (60 ticks per second, set once at start-up; see SPEC-0007). Pairing rules: High, and no pair forms on disc. Base states and script control: High; the park-and-resume of a script walk and the script hold during a walk: Medium-High. Talk facing: High. Travel by root motion and facing: High. Place of the AI step in the tick and the one-tick delay from clip request to first root displacement: High. Townsfolk speeds at R = 1: High. Movement-type to animation mapping in the walk classes: Medium-High. Proximity items: High that some callbacks give items; Medium for their gates; High for the 5.8 firing distance and the Action press, Medium for which characters can be the hero's candidate. Wall collision of region characters (navmesh confinement, its edge-reach quirk and the off-mesh restore): High; the blocked flag's setter, clearers and readers: High. Route table: High for the layout and for the read by the navigation step, Medium-High for its meaning (the next edge on a shortest route). Route step through the chosen edge: Low. Spawn point search, its retry rule, its cursor quirk and its callers: High. Movable-block contact pass: High for the block selection, Medium-High for the push. Values marked "(confirmed on disc)" match the boot program's code and data.

## Summary

A non-player character (NPC) runs one **mode** at a time. The mode decides what the character does on each tick: stand idle, wander, patrol a route, pair with another villager, greet the passing hero, hold a seated or lying pose, or one of the enemy-style modes (flee, search, stunned, guard, ambush, follow, agro, appear). Townsfolk are the characters of **behavior class 6**. Each townsfolk character has a behavior code, stored on the disc in its placement record. Whenever its mode is 0, the next class tick sets the mode to the code. The level script call SetAI_Behavior replaces the code and the character's AI values, and resets the mode to 0 when the code changes. The enemy-style modes have bodies only in the common-enemy class (SPEC-0038). In class 6 they do nothing.

Separately, a character runs one **behavior class** at a time. The walk calls of SPEC-0030 select one of five walk classes: four walk to a point and one walks a sequence of path points. Each class has an arrival rule, and on every tick each class turns the character toward its direction of travel by a fixed fraction of the remaining difference.

Level scripts also register proximity callbacks. Eleven callbacks in ten level modules can give the hero an item on approach, and most of them give it only when a story flag or a prompt answer passes.

Apart from the arrival step of walk classes 23 and 25 (Behavior classes), none of these routines moves the character itself. A walking NPC travels by the root motion of the locomotion animation it plays, exactly as the heroes do: the routines choose the animation and steer, and the motion record turns the animation's root step into a displacement each tick. Townsfolk walk at 5.00 units/s (1.71 for the 214 of 666 townsfolk placements whose animation table names the slower walk clip on id 1; patrol always uses a 5.00 clip), run at 13.50 and fast-run at 19.30 units/s.

A walking character on a navigation mesh finds its way by a route table that is stored with the mesh: for each pair of triangles it names the edge to leave by. There is no run-time route search.

Region characters (townsfolk, enemies and bosses) do not collide with the world's wall segments. Each tick the border edges of the navigation-mesh triangle they stand on act as their walls, and a character found off the mesh is put back where it last stood on it. The reach of an edge wall follows a quirk of the original that must be reproduced (see "Wall confinement of region characters").

A level script takes control of a character by putting it into a script base state, and gives control back by returning it to base state 1. While a character is in conversation, under script control or in a looped script animation, its AI is frozen and later resumes where it stopped.

The clean side can build the mode switch, the walk classes, the villager pair, the greeting, NPC travel and the item callbacks from this spec. There is no townsfolk follower, escort or teleport-if-far behavior (see "What NPCs do not do").

Related specs: SPEC-0030 (script calls: SetAI_Behavior, the walk calls, GrabEntityControl, ReleaseEntityControl, RegisterEntityProximityCallback and the level prompt), SPEC-0040 (character state requests, base states, movement classes, steering, the motion record and the character push), SPEC-0006 (placement records), SPEC-0018 (navigation-mesh locate, cell lists, wall segments and the contact response), SPEC-0005 (clip playback, root records and clip speed), SPEC-0022 (conversations and the hero's interaction target), SPEC-0026 (game-state code), SPEC-0029 (monster rows and enemy-state modes), SPEC-0036 (status effects), SPEC-0038 (the common-enemy class 17), SPEC-0053 (the parked hero has no companion AI).

## Where it lives on the disc

- The mode switch, the walk classes, the villager pair and the item callbacks are code. The clean side writes its own code from the behavior described here. None of it is read from the disc.
- Run-time values that the behavior depends on:
  - The frame rate is 60.0 ticks per second. It is written once at start-up into the frame-rate word at data address 0x368FA0 (the rate field of the frame object at data address 0x368F00) from the boot table entry at data address 0x30C598 (row 0, column 0). See SPEC-0007. The villager timers below are computed from this word, so the tick counts and their seconds conversions (60 ticks per second) are confirmed.
  - The blend weight 0.1 used for steering off the navigation mesh is a boot data word at data address 0x2DD86C (confirmed on disc: value 0.1).
  - The greeting and hold distance is a boot data word at data address 0x2DD8CC (confirmed on disc: value 6.0). The exit distance is twice this word (12.0).
  - Each character's behavior code and AI values are in its placement record (kind-3 data, SPEC-0006): the code at +0x84, the patrol route at +0xB8 and the 28-byte AI block at +0xBC to +0xD7 (see "Placement values").
  - The behavior class, the pair radius, the pair turn weight and the mode 0 remap switch come from the character's monster row (resource 530, SPEC-0029) and its companion row (resource 531) (see "Monster row values").
  - NPC travel speeds are not stored as values. They come from the per-key root records of each character's locomotion clips (table A of SPEC-0005) in its animation bank. For townsfolk that is the shared townsfolk body bank (body 526, SPEC-0040 Behavior 3C).
- The tables that map behavior class numbers and mode numbers to routines are boot data. This spec gives every class and mode in words, so the clean side needs neither table.
- The level modules that register item callbacks are listed in the proximity section.

## Layout

### Character AI fields used by this behavior

Offsets inside the runtime character record are not part of this spec. The offsets in "Placement values" and "Monster row values" are offsets in disc records.

| Field | Type | Meaning |
| --- | --- | --- |
| Behavior code | signed 16-bit int | Starts as the placement record's code (kind-3 data +0x84) and is replaced by SetAI_Behavior. |
| AI block | 28 bytes | The placement record's bytes +0xBC to +0xD7. SetAI_Behavior copies its data block over them. The wander and patrol modes read them as listed in "Placement values". |
| Townsfolk flag | flag | 1 exactly when the character's default behavior class is 6. Cleared by the NPC reset and set by the default class request at every bind; constant while the character is bound (see "Townsfolk class"). The same word as the talker flag of SPEC-0022. |
| Fidget state | flag | Wander and patrol: whether the one-time fidget request of the current pause has been made. The NPC reset sets it to "not started" and the fidget timer to 0. |
| Mode | int | The current mode (see the Modes table). |
| Greeting counter, done flag, pose flag | int, flag, flag | Used by the greeting (mode 13) and the walk hold. |
| Pause timer, fidget timer | int, ticks | Used by wander and patrol. |
| Home point | position | The centre of the wander square and the base of the spawn point search (form H). The bind sets it to the record's placement position; for a character whose row is not a flyer (Can Fly 0) the height is first replaced by the ground found by a downward probe from 8.0 above that position, when the probe finds ground. The saved position of the confinement starts at the same point. A character released in mode 7 (guard) has its home and saved position set to its current position, and that position and facing are written back into its record (High). |
| Social gate | flag | 0 allows pairing. EnableVillagerSocialize sets 0, DisableVillagerSocialize sets 1, the NPC reset sets 0. |
| Role, talk timer, cooldown | int, ticks, ticks | Villager pair state. Role 0 approach, roles 1 and 2 the two talk roles, role 3 ending. |
| Mode timer | int, ticks | Drawn on each call that sets the mode and never read (see "Mode change timer"). |
| Behavior class | int, 0 to 39 | The walk or NPC class now running. 0 means none. |
| Walk target | position, optional orientation, stop distance | Where a walk class moves the character. |
| Navigation state | nav index, next nav index, arrival flag | Used by the per-tick walk step. |
| Navigation-mesh index | int | Which mesh of the world the character walks on. -1 (no mesh) unless a level script or a boss controller assigns one (see "Region characters with no navigation mesh"). |
| Partner id | int | Villagers only: the character id of the paired villager. |
| Move type | int | The movement type set by the script call SetAI_MovementType: 0 walk, 1 run, 2 fast run (3 and 4 also accepted). Default 0. Kept in the character's navigation block. |
| Blocked flag | flag | Set when the navmesh confinement pushes, clips or restores the character. Nothing else sets it (see "Blocked flag"). |
| Blocked timer | int, ticks | Set to 120 with the blocked flag; counts down by 1 per tick while the flag is set. |
| Saved position | position | The character's last position found on the navigation mesh. |
| Wall-type value | int | Reset to -1 (none) by the pre-step of each motion-record tick. |
| Lock value | int | A one-tick flag of the character's motion record; when it is not 0 the off-mesh restore is skipped. Placement clears it, and the motion-record step clears it at the end of every tick. Only the Gnarfas boss sets it (to 1, during its leap at the hero and on a tick when its height differs from its reference by more than 1.0), so for every other character it is 0 at the restore (High for the writers; Medium for the order within a tick). |

### Placement values (kind-3 data, SPEC-0006; High)

| Offset | Type | Meaning |
| --- | --- | --- |
| +0x10 | vector | Placement facing: a direction vector with lanes x (+0x10), y (+0x14, up) and z (+0x18), heading = atan2(x, z) degrees from +Z toward +X (decode rule: SPEC-0006, "Character placement: position and facing decode"). Mode 0 with code 0 turns toward it. A release of the character writes its current facing back here (see the orientation survey below). Which rules show it and which replace it: "Authored facing: where it shows and what replaces it". |
| +0x84 | signed 16-bit int | Behavior code. |
| +0xB8 | signed 32-bit int | Path index: the number of a path in the area's path list (SPEC-0006, Tagged tables, tag 0x0A), -1 for none. It is the patrol route of mode 6 and the route of a script path walk (class 35), which overwrites it. Outside the AI block, so SetAI_Behavior does not change it. |
| +0xBC | 32-bit float (wander) or signed 32-bit int (patrol) | Wander: radius r. Patrol: loop mode (0 loop, any other value back-and-forth). |
| +0xC0 | signed 32-bit int | Wander: shortest pause, ticks. Not used by patrol. |
| +0xC4 | signed 32-bit int | Wander: longest pause, ticks. Patrol: shortest point pause, ticks. |
| +0xC8 | signed 32-bit int | Wander: pause chance, percent. Patrol: longest point pause, ticks. |
| +0xCC | signed 32-bit int | Patrol: random-stop chance, percent. |
| +0xD0 | signed 32-bit int | Patrol: shortest random stop, ticks. |
| +0xD4 | signed 32-bit int | Patrol: longest random stop, ticks. |

#### Authored orientation survey (class 6, High)

The facing in each character placement is its own unit vector, separate from the world's entry-0 hero spawn facing. In the 666 class-6 placements across 100 world packs and 328 character resources, the horizontal projection of facing occupies all 24 fifteen-degree heading bins; 8 vectors also have a nonzero vertical component. Farm world 909 has 20 records containing 12 distinct facing vectors. World 1023 has 48 records across its region resources containing 37 distinct vectors (rounded to four decimal places) and 22 five-degree heading bins (bins centred on multiples of 5 degrees; 23 with bins that start at multiples of 5). Heading is measured from +Z toward +X, and the 48 vectors are all horizontal. Some characters recur in multiple region resources, so record counts are not counts of unique visible actors. Runtime AI can replace the current heading as described below; these counts describe authored placement data. The facing that a code-0 character turns back to is the facing held in its record at that moment: the disc vector at the first bind and after any reload of its region, but a release of the character (distance more than 280 units from the hero in a streaming block, or a cell-flag release) writes its current position and facing into the record unless its AI mode is 7, so the next bind of the same record starts from and turns back to that written-back facing (SPEC-0006, Release and rebind; High for the write-back, Medium-High for its effect on the idle target).

Every range draw is a whole number drawn uniformly with both ends included. The two ends may be stored in either order.

#### Authored facing: where it shows and what replaces it (class 6, High)

The decode of the vector is in SPEC-0006 ("Character placement: position and facing decode") and SPEC-0040 ("Character facing: how to read it"). Heading below means atan2(x, z) in degrees from +Z toward +X.

| Situation | Heading of the townsperson |
| --- | --- |
| Bind (every mode, every code) | Exactly the record's facing (y dropped). Nothing turns the character on the bind tick. |
| Mode 0 with record code 0 (idle) | Stays at the authored heading. Every tick it blends toward the record's facing with weight 0.05 (v = 0.05 t + 0.95 c, then normalised); the turn never ends, so any displaced heading returns: from 90 degrees away it is within 10 degrees on tick 48 and at a dot product of 0.99 on tick 52; from 170 degrees, on ticks 93 and 97. The test is on the record's code word at that moment (the word SetAI_Behavior writes), not on a saved copy; a character whose code was changed to 5, 6 or 7 never turns back. t is the record's current +0x10, which a release may have overwritten (see the survey above). |
| Mode 13 (greeting) | No turn. The character keeps its heading and plays idle clips; it does not face the hero. |
| Modes 7 and 9 (statues that greet) | Bodies are empty: the bind heading is kept for good, except for a talk turn, which is never undone. |
| Modes 10 and 12 (seated, lying) | No turn of any kind, no greeting, no talk turn (pose flag). The authored heading is the pose's heading. |
| Mode 5 (wander) and 6 (patrol) while walking | The move direction (steering weight 0.1 per tick). The authored heading is visible only before the first step. During pauses the last move direction stays. They never turn back to the record's facing. |
| Base state 25 with the hero as partner and no pose | Turns toward the horizontal direction to the hero with weight 0.3 per tick (within 10 degrees from 90 on tick 7). With no partner (SetConversationTarget argument 0, or a LevelMsg line alone) or in a pose, it does not turn. After the talk: code 0 turns back at 0.05; code 5 or 6 resumes walking; code 7 or 9 stays facing where the talk left it. |
| Script walks (classes 23, 24 and 35) | The move direction. At the end the heading is the last move direction; a code-0 character then turns back at 0.05. |
| WalkEntityAndOrient (classes 25, 26) | After arrival, turns at weight 0.1 until the dot product with the locator's flattened direction is at least 0.99. |
| MoveEntity, locator form | Face option on: takes the locator record's facing. Off: keeps the heading. Coordinate form: keeps the heading. |
| Base states 33 and 55 (script hold, scripted loop) | Heading kept (no turn). |
| Release and rebind | A release outside default mode 7 stores the current heading in the record; a rebind starts from it. Mode 7 characters rebind at the disc heading. |

There is no script call that sets or aims a character's heading directly: no SetFacing or LookAt exists for characters. The animated-object rotation calls rotate scene objects, not characters, and the head look-at (SPEC-0040, Behavior 23A) is used by the hero and the Plague Serpent only. Townsfolk never look at the hero with their heads.

#### Why every townsperson faces the same way (port faults and their symptoms)

Authored data does not have one heading: farm pack 909 has 12 distinct headings in 20 townsfolk records and pack 1023 has 37 in 48. About a third of class-6 records (203 of 666) face within 0.0005 of +Z or −Z on every lane, so groups of neighbours with one heading (a shop interior, a council hall) are normal. If a build shows one heading across unrelated places, check these in order:

1. Facing field ignored at bind: every character keeps the body's default heading (+Z, or +X for a body whose reset facing is +X). Symptom: one heading everywhere, including characters that never turn.
2. Turn-back target taken from the wrong source (the hero's spawn facing, the world's entry-0 facing, the camera): code-0 characters drift to that one heading in about 50 ticks and stay there; patrol and wander characters look fine. Symptom: right at bind, wrong a second later.
3. Wrong record offset (position lanes read as facing, the 16-byte chunk header added to the base, or the name bytes at +0x20): vectors point along the position or are noise. Neighbours then share one heading (they point from the origin toward themselves).
4. Wrong lanes: atan2(z, x) mirrors and rotates every heading by 90 degrees (distinct but wrong); atan2(y, x), atan2(x, y) or a w lane give only 0, 90, 180 or 270 because y and w are 0. Symptom for the latter: everyone faces one of two or four directions.
5. Vector treated as an angle (lane x used as radians or degrees): every heading is within about 1 radian or 1 degree of +Z, so everyone looks the same.
6. Degrees against radians: atan2 output in radians fed to a degree API gives headings within ±3.14 degrees (all near +Z); degrees fed to a radian API gives scrambled but never-matching headings.
7. Basis built from world axes (side row fixed at (1, 0, 0), or a look-at toward the hero): ignores the vector entirely.
8. Turn-back weight used as an angle (yaw = 0.05 or yaw += 0.05) instead of a vector blend v = 0.05 t + 0.95 c: all idle characters collapse to a heading near 0.
9. Handedness flip on import (z negated): every yaw becomes 180 − yaw (Hawker Gamzan 12.90 becomes 167.10; +Z shopkeepers face away from their counters). An x flip gives 360 − yaw.
10. The summon placeholder (1, 0, 0), position zero, applied to ordinary records: everyone faces +X.
11. Release before the first tick writing a default heading back: after the first stream-out and rebind every character shares it. The write-back is the current forward row, which for a bound character equals the authored heading until something turns it.
12. Idle or pose code that snaps to the move direction every tick: a standing character has a zero move direction, so it falls to the default.

A quick self-test: decode all 666 class-6 records; there must be 24 occupied 15-degree bins, and pack 909 must show 12 distinct headings among its 20 records.

#### Facing test vectors (class 6, High)

Columns: pack; region resource (emb = the pack's embedded block); entity id in the pack's entity table; name; default behaviour code (record +0x84); facing lanes (x, y, z); stored position (x, y, z); yaw in degrees; look direction (dominant axis, then the lesser one). Packs: 1023 Meridell, 909 Farm, 1098 Brightvale, 1300 Faerieland. The side row is (z, 0, −x) over √(x² + z²).

| Pack | Region | Entity | Name | Code | Facing (x, y, z) | Position | Yaw | Looks toward |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| 1023 | 1035 | 65 | Hawker Gamzan | 0 | (0.2232, 0.0000, 0.9748) | (99.30, 27.54, 366.55) | 12.90 | +Z (+X) |
| 1023 | 1035 | 66 | Shanra | 0 | (-0.1192, 0.0000, 0.9929) | (176.62, 27.54, 372.20) | 353.15 | +Z (-X) |
| 1023 | 1031 | 57 | Sir Aldwin | 6 | (0.0000, 0.0000, 1.0000) | (-86.44, 27.54, 493.93) | 0.00 | +Z |
| 1023 | 1026 | 4 | Guardsman Ulfric | 6 | (0.4664, 0.0000, -0.8846) | (-21.56, 27.47, 797.25) | 152.20 | -Z (+X) |
| 1023 | 1026 | 2 | Guardsman Darol | 0 | (-1.0000, 0.0000, -0.0085) | (18.87, 29.64, 603.06) | 269.51 | -X |
| 1023 | 1024 | 0 | Meerouladen | 0 | (-0.6581, 0.0000, -0.7529) | (-34.94, 29.38, 634.24) | 221.16 | -Z (-X) |
| 1023 | 1033 | 61 | Bermund | 0 | (0.6751, 0.0000, 0.7377) | (-121.44, 27.49, 486.09) | 42.46 | +Z (+X) |
| 909 | 914 | 31 | Lucy | 0 | (-0.2421, 0.0000, 0.9702) | (-1860.44, 45.51, 1131.45) | 345.99 | +Z (-X) |
| 909 | 916 | 33 | Lucy | 0 | (-0.4465, 0.0000, 0.8948) | (-1887.32, 45.51, 1166.39) | 333.48 | +Z (-X) |
| 909 | 919 | 36 | Lucy | 0 | (0.3606, 0.0000, 0.9327) | (-1835.18, 50.00, 1182.70) | 21.14 | +Z (+X) |
| 909 | 923 | 40 | Patricia | 7 | (0.9632, 0.0000, 0.2687) | (-1877.73, 50.00, 1176.79) | 74.41 | +X (+Z) |
| 909 | 924 | 41 | Patricia | 6 | (-0.7718, 0.0000, -0.6358) | (-1938.15, 50.00, 1165.28) | 230.52 | -X (-Z) |
| 909 | 927 | 44 | Hubert | 0 | (0.9885, 0.0000, 0.1512) | (-1961.99, 45.51, 1064.68) | 81.30 | +X (+Z) |
| 909 | 920 | 37 | Patricia | 0 | (0.0000, 0.0000, 1.0000) | (-1937.53, 50.00, 1170.61) | 0.00 | +Z |
| 1098 | 1101 | 12 | Sir Earwobble | 0 | (0.9991, 0.0000, -0.0416) | (-74.22, -106.20, 67.63) | 92.39 | +X |
| 1098 | 1101 | 13 | Master Tailwag | 0 | (-0.0246, 0.0000, -0.9997) | (-69.85, -106.16, 71.91) | 181.41 | -Z |
| 1098 | 1101 | 14 | Lord Fatbat | 0 | (-0.9999, 0.0000, 0.0102) | (-65.95, -106.12, 67.65) | 270.58 | -X |
| 1098 | 1101 | 15 | Lord Adelheit | 6 | (-1.0000, 0.0000, 0.0000) | (-98.61, -107.88, -37.30) | 270.00 | -X |
| 1098 | 1105 | 19 | King Skarl | 0 | (0.0000, 0.0000, -1.0000) | (0.00, -124.24, 100.79) | 180.00 | -Z |
| 1098 | 1106 | 20 | Sir Lawrence | 0 | (1.0000, 0.0000, 0.0000) | (-9.04, -128.05, 73.10) | 90.00 | +X |
| 1098 | 1104 | 18 | Squire Grayson | 0 | (0.6840, 0.0000, 0.7295) | (95.76, -126.21, 95.52) | 43.16 | +Z (+X) |
| 1300 | 1305 | 8 | Seradar | 0 | (-0.7619, 0.0000, -0.6477) | (174.29, 1531.21, -48.19) | 229.63 | -X (-Z) |
| 1300 | 1308 | 11 | Council Herald Kekou | 0 | (-0.9726, 0.0000, 0.2324) | (4.02, 1536.38, -216.40) | 283.44 | -X (+Z) |
| 1300 | 1309 | 12 | Council Herald Kekou | 0 | (-0.0262, 0.0000, 0.9997) | (-3.17, 1536.38, -213.05) | 358.50 | +Z |
| 1300 | 1310 | 13 | Council Herald Kekou | 0 | (0.9847, 0.0000, -0.1742) | (-92.47, 1533.35, -102.14) | 100.03 | +X (-Z) |
| 1300 | 1314 | 17 | Old Lady | 0 | (0.9838, 0.0000, -0.1793) | (-92.47, 1533.35, -102.14) | 100.33 | +X (-Z) |
| 1300 | 1322 | 74 | Old Lady | 0 | (-0.0872, 0.0000, -0.9962) | (3.00, 1523.72, 223.00) | 185.00 | -Z (-X) |
| 855 | emb | 0 | Ingmar the Armourer | 6 | (0.0000, 0.0000, 1.0000) | (-1.20, -0.22, -30.14) | 0.00 | +Z |
| 880 | 881 | 3 | Gord the Merchant | 6 | (0.0000, 0.0000, 1.0000) | (1824.80, 164.70, -937.69) | 0.00 | +Z |
| 1131 | emb | 0 | Armourer Inga | 0 | (0.0000, 0.0000, 1.0000) | (0.17, 0.00, -7.83) | 0.00 | +Z |
| 1407 | 1408 | 0 | Smithy Magnar | 0 | (0.9998, 0.0000, 0.0175) | (-4.92, 0.00, -4.31) | 89.00 | +X |
| 1405 | 1406 | 0 | Armourer Tharbad | 0 | (0.8677, 0.0000, 0.4971) | (-7.22, 1.90, -0.90) | 60.19 | +X (+Z) |
| 1020 | 1021 | 0 | Alchemist Joku | 6 | (-0.7697, 0.0000, -0.6384) | (7.63, 2.64, 12.62) | 230.33 | -X (-Z) |
| 1150 | emb | 0 | Pat the Bartender | 0 | (-0.9998, 0.0000, -0.0184) | (4.54, 0.00, -6.33) | 268.95 | -X |
| 1417 | 1418 | 0 | Potion-Maker Symond | 0 | (0.4976, 0.0000, 0.8674) | (-8.60, 0.00, -7.06) | 29.84 | +Z (+X) |
| 956 | 958 | 1 | Illusen the Earth Faerie | 7 | (-0.6034, 0.0000, -0.7974) | (-582.57, 121.48, -493.09) | 217.11 | -Z (-X) |
| 1251 | emb | 0 | Sophie the Swamp Witch | 0 | (0.9767, 0.0000, -0.2147) | (-22.29, 0.00, 5.61) | 102.40 | +X (-Z) |
| 678 | 679 | 3 | Kell the Beggar | 0 | (0.4918, 0.0000, 0.8707) | (-1596.08, 49.32, 1092.96) | 29.46 | +Z (+X) |
| 1122 | 1123 | 0 | King Skarl | 0 | (0.9707, 0.0000, 0.2404) | (18.55, -25.30, -197.13) | 76.09 | +X (+Z) |
| 842 | 844 | 19 | Mayor Jurgin | 7 | (-0.8676, 0.0000, -0.4973) | (40.87, 103.86, 1724.19) | 240.18 | -X (-Z) |
| 1350 | 1352 | 29 | Tharius the Sage | 6 | (-0.7024, 0.0000, -0.7118) | (108.34, 0.00, -300.71) | 224.62 | -Z (-X) |
| 1411 | 1412 | 4 | King Altador | 0 | (0.9325, 0.0000, -0.3613) | (-44.09, 129.63, 54.32) | 111.18 | +X (-Z) |
| 1411 | 1412 | 3 | Fauna the Petpet Handler | 0 | (-0.4758, 0.0000, -0.8795) | (-31.42, 129.63, 68.75) | 208.41 | -Z (-X) |
| 1176 | 1177 | 2 | Dorf | 7 | (-0.3714, -0.0049, -0.9285) | (214.50, 32.41, 345.27) | 201.80 | -Z (-X) |
| 1176 | 1177 | 4 | Jeffrey | 7 | (-0.9419, 0.0029, 0.3360) | (226.43, 33.68, 301.20) | 289.63 | -X (+Z) |
| 1327 | 1333 | 43 | Seradar | 7 | (-0.9923, -0.0294, -0.1200) | (3.42, 4.86, 10.94) | 263.10 | -X (-Z) |

Notes on the vectors. Position y is the stored value; a ground walker's bind may replace it with the probed ground height. Characters with the same name in two regions (Kekou and the Old Lady at the same spot in pack 1300) are alternative loads, not two people. Rows with a small y lane (Dorf, Jeffrey, Seradar in 1327) have a flat steering heading: for Dorf the flat vector is (-0.3714, 0, -0.9285) over its length 1.0000 (yaw unchanged at 201.80). Basis check for Hawker Gamzan: forward (0.2232, 0, 0.9748), side (0.9748, 0, -0.2232), up (0, 1, 0). Mirror check: a flipped z gives yaw 167.10 for Gamzan; a flipped x gives 347.10.

| Source | Type | Meaning |
| --- | --- | --- |
| Monster row (resource 530) +0x36 | signed int | Default behavior class. A negative value means none. |
| Companion row (resource 531) +0x10 | int | Default behavior class used when the monster row gives none. |
| Monster row +0x44 | 32-bit float | Pair radius (the social approach distance). |
| Monster row +0x82 | 16-bit int | Mode 0 remap switch (the row's default-display word): 0 allows the remap to mode 23. |
| Monster row +0x84 / +0x88 | 32-bit float | Pair turn weight for ground characters / flyers. |

On disc, every class-6 placement has +0x44 = 0.0, +0x84 = 0.0 and +0x82 = 1.

### Modes

Names are this spec's labels. The boot program holds debug names for twelve modes, but the mapping from mode number to name is not established. The mode numbers are the reference, and the names are labels only. Animation ids are slots in the character's own animation table (SPEC-0040). The table gives the per-tick body in class 6 (townsfolk). The bodies of the enemy-style modes in the common-enemy class 17 are in SPEC-0038. Confidence: High.

| Mode | Name | Per-tick behavior in class 6 |
| --- | --- | --- |
| 0 | IDLE | Requests idle id 0. If the behavior code is 0, turns toward the placement facing (kind-3 data +0x10) with blend weight 0.05 each tick (confirmed on disc). A character with a nonzero code leaves mode 0 in the pre-step before this body runs (see "Townsfolk routine"). |
| 1 | FLEE | Nothing. |
| 2 | SEARCH | Nothing. SetAI_Agro sets this mode (SPEC-0030). |
| 3 | SOCIALIZE | Villager pair (see "Villager pair"). Not reached on disc. |
| 4 | STUNNED | Nothing. A stun sets this mode (see Edge cases). |
| 5 | MEANDER | Wander (see "Wander"). The pair search runs first, in the pre-step. |
| 6 | PATROL | Patrol with pauses (see "Patrol"). The pair search runs first, in the pre-step. |
| 7 | GUARD | Nothing. The character keeps the animation it is playing. |
| 8 | AMBUSH | Nothing. |
| 9 | FOLLOW | Nothing. The character keeps the animation it is playing. |
| 10 | (no name here) | Requests id 65 every tick (a seated pose for townsfolk). |
| 11 | (no name here) | Requests id 97 every tick (lying). |
| 12 | (no name here) | Requests id 98 every tick (lying) and applies status 2 (poison; SPEC-0036) to itself every tick. The shared status apply accepts it for a townsfolk character: the timer is set to 7200 ticks (120 seconds) on every tick, so it never runs down, and the status visual of poison is started. No hit points are lost: the poison damage test never fires while the timer is refreshed every tick, and the damage would go through the take-hit, which a townsfolk ignores (High for the timer and the absence of damage; Medium for how the visual looks). |
| 13 | (no name here) | Greeting pause (see "Greeting"). |
| 14 to 23 | (no name here) | Nothing (confirmed on disc). |

"Nothing" means the class-6 body makes no request, so the character keeps the animation it is playing.

### Behavior classes (walk routines)

Class 6 is the townsfolk class (see "Townsfolk class"). Class 17 is the common-enemy class (SPEC-0038). The walk classes below are started by script walk calls.

| Class | Set by | Gate (walk argument) | Arrival rule | After arrival |
| --- | --- | --- | --- | --- |
| 23 | WalkEntity | zero or less | distance below 1.0 | Set the position to the walk target; play idle animation; end |
| 24 | WalkEntity | above zero (the gate is the stop radius) | distance below the stop radius | Play idle animation; end |
| 25 | WalkEntityAndOrient | zero or less | distance below 1.0 | Set the position to the walk target; turn to the target orientation; then end step |
| 26 | WalkEntityAndOrient | above zero (the gate is the stop radius) | distance below the stop radius | Turn to the target orientation; then end step |
| 35 | WalkEntityAlongPath | (path index) | distance to the current path point below 0.5 | Advance to the next path point; at the last point the class ends (Path walk) |

The stop radius of classes 24 and 26 is the gate value of the walk call (SPEC-0030 rows 106 and 108). On arrival, classes 23 and 25 put the character exactly on the walk target point (all three coordinates, so a jump of less than 1.0 horizontally and a change of height to the target's height); classes 24 and 26 leave it where it stopped (High).

## Behavior

### Townsfolk class (behavior class 6) (High)

- A region character's default behavior class is the monster row's class (resource 530 +0x36). When that is none (negative), it is the companion row's class (resource 531 +0x10).
- A character whose default class is 6 is a townsfolk character. The default class request at bind sets the townsfolk flag to 1 exactly when the class is 6.
- On disc, 666 of the 3,036 kind-3 placements (280 distinct names) resolve to class 6. Their monster rows all give none, and their companion rows give 6. Every placement with main-family value -7 (SPEC-0005) is among them.

Townsfolk flag lifecycle (High):

- The NPC reset clears the flag.
- Every bind path then sets it through the default class request: flag = 1 when the default class is 6, else 0. The streamed bind and the forced bind make this request when the record's summon byte is 0; spawn by key always makes it.
- Script class requests (the walk classes 23 to 26 and 35, and the class-34 request) never write the flag. So the flag is constant while the character is bound.
- No class-6 record on disc has a summon byte, so the flag is 1 on all 666 class-6 placements and on no other placement. That is 592 placements on the shared townsfolk body bank (main family -7), 46 of family 0, 16 of family -6 (in regions the game never reaches), 8 of family 1 and 4 of family 2.
- It is the same word as the talker flag of SPEC-0022.

What reads the townsfolk flag:

- The idle picker (SPEC-0040) returns id 0 for a flagged character and does nothing else: no fidget draw, no hold-timer change, no random draw and no flying idle. Only the mode-19 test runs before this test. (High)
- The talker list (SPEC-0022) holds exactly the flagged characters. An entry whose character's flag is clear is dropped on the next update. (High)
- The walk animations. The townsfolk walk slots pick the townsfolk animation ids by speed code. The walk classes 23 to 26 give a character without the flag walk slot 2 (see "Walk speed and movement type"). (High)
- The pose test. For a flagged character, modes 10 to 12 count as poses for the greeting. (High)
- Take-hit. The NPC take-hit returns at once for a flagged character, so townsfolk ignore hits. (High)
- Hit candidate. The hit-candidate query answers "no" for a flagged character, so the hero's swing, Roberta's shots and the Mote walks never select a townsfolk character: no impact cue, no hit-list entry, no on-hit hook and no magic cost for the swing. The class-6 routine also makes its character invulnerable (the invulnerable word +0x7F0 is set to 1 before the first tick and the class never clears it) and removes it from lock-on, so the flag is not the only reason. (High)
- Status apply. Nothing in the shared status apply tests the flag. A townsfolk character accepts every status; see mode 12 and the stun note in Edge cases. (High)
- The class-17 target filter (SPEC-0038) excludes flagged characters. (High)
- The Juppie death sequence (SPEC-0015) spares flagged characters: its damage goes only to the other characters whose flag is clear. (High)
- The variant set-up. At bind, a flagged character's variant set-up word is set to 1 when progress flag 216 is on and the world's variant key check gives 1 (SPEC-0022, Variant key check). The world's variant key is the pack's sound selector word (the third word of its type-1001 record, 0 to 11 on disc; SPEC-0009, Area to music row), so the check passes in the same story span as the alternate music of that selector. What it changes on screen: the character's two skin colours are replaced by the purple pair of SPEC-0040, Behavior 26 (High).

### SetAI_Behavior

1. When the call's data pointer is not null, the 28 bytes of its data block are copied over the placement record's AI block (kind-3 data +0xBC to +0xD7). This happens on every call.
2. If the code differs from the stored code, the code is stored and the mode is set to 0 directly. The mode setter is not used, so there is no remap and no timer draw. If the code is the same, only the block copy happens.
3. New values act at their next use, for example at the next wander point.
4. In the level scripts, SetAI_Behavior uses codes 0, 5, 6, 7 and 8. Code 7 is used once, by the road-fight intro of the module shared by packs 678, 686 and 691 (SPEC-0042): after the scene it sets entities 1 and 2, the two road enemies, to code 7 and then turns their aggression on.
### Mode from code (High)

1. The behavior code starts as the placement record's code (kind-3 data +0x84). Every placement has one, and SetAI_Behavior replaces it.
2. Each class-6 tick begins with a pre-step (see "Townsfolk routine"). While the mode is 0, the pre-step calls the mode setter with the code. So the mode becomes the code in the same tick, before the mode body runs.
3. A code-0 character stays in mode 0. A character with code c is never in mode 0 when its mode body runs. It returns to c on the first tick after anything sets mode 0.
4. On-disc codes of the 666 class-6 placements: code 0 in 414, code 6 in 121, code 7 in 67 (mostly guards and officials), code 5 in 51, code 10 in 6, code 9 in 5 and code 12 in 2. Codes 7, 9, 10 and 12 therefore come from the disc for townsfolk, not from scripts (the one script use of code 7 is on two enemies). No placement uses code 11.
5. Two other routines set the mode from the code: one sets mode 8 for code 8 and mode 0 otherwise, and one waits for animation 81 and then sets the code. Both belong to the common-enemy class 17 (its start-up and its mode 23; SPEC-0038). Neither runs for townsfolk.

### Mode 0 remap (High)

A request through the mode setter for mode 0 gives mode 23 instead only when the monster row's remap switch (+0x82) is 0 and the code is not 0. When the switch is not 0, the mode becomes 0. The switch is 1 on all 666 class-6 placements, so townsfolk never enter mode 23. Mode 23 is the class-17 wait for animation 81 (SPEC-0038).

### Mode change timer

Each call of the mode setter also draws the mode timer as an integer uniformly from 60 to 180 ticks (1 to 3 seconds) and clears two neighbouring words. The timer is unused (High): no routine reads it or counts it down, and the per-mode tick hook that the mode bodies call has an empty body. One of the two cleared words is a flag that no routine sets; a mode body requests clip 7 instead of the idle picker only while that flag is set, so that request never happens. A rebuild may drop the timer and the flag. The draw still uses the shared random stream (one draw per mode change), which matters only when matching a recorded random sequence.

### Townsfolk routine (per tick)

A class-6 character runs one routine each tick:

1. It updates its navigation index.
2. Pre-step:
   - In every mode except 10, 11 and 12, it runs the greeting entry test (see "Greeting").
   - In mode 0, it sets the mode from the code.
   - In modes 5 and 6, it runs the pair search (see "Villager pair").
3. It sets the movement class from the navigation move type.
4. It runs the per-tick body of the current mode (Modes table).
5. It yields until the next tick.

A mode with no body does nothing on that tick, so the character keeps the animation it is playing.

### Greeting (mode 13) (High)

Entry. In every mode except 10, 11 and 12, the pre-step tests these conditions on each tick. The greeting starts when all of them hold:

- the hero is not in base state 33, 50, 51, 52 or 55;
- the game-state code (SPEC-0026) is 2 or less (normal play, the six-pass wait after a load, or the one pass after a game-over request; Open question 24);
- the character's own base state is not 13 (only hero routines ever set base state 13, so for townsfolk this condition always holds in play; Medium-High);
- the hero's horizontal distance is below 6.0 (the boot word at data address 0x2DD8CC);
- the dot product of the unit horizontal direction from the character to the hero and the character's facing is above 0 (the hero is in the front half).

On entry, the mode is set to 13 and the counter and the done flag are cleared. The pose flag is set when the current animation id is one of 65 to 74, 78, 97, 98, 146 or 147, or when a townsfolk character is in mode 10, 11 or 12. Mode 13 is not excluded from the test, so the entry repeats on every tick the conditions hold. Each repeat restarts the counter.

Body, without the pose flag:

1. While the counter is below 180 ticks (3.0 seconds), the counter goes up by 1 and the idle picker id (SPEC-0040) is requested.
2. Then id 30 is requested until it has played to 0.99.
3. Then the done flag is set and the idle picker is requested again.

With the pose flag set, the body makes no animation request.

Exit, tested every tick: the mode ends when the hero's horizontal distance is above 12.0 (twice the boot word), or when the dot product is below -0.1. On exit the mode is set to 0, and the next tick's pre-step sets the code again. The character does not turn toward the hero in this mode.

### Walk-to-point routine (classes 23, 24, 25 and 26)

Once started, the class repeats these steps on each tick until it ends. All distance tests here are horizontal (ground-plane) distances, ignoring height.

1. Hold test, on each tick while the class is not holding: if the hero's horizontal distance is below 6.0 (confirmed on disc) and the dot product of the unit horizontal direction from the walker to the hero and the walker's facing is above 0 (confirmed on disc), the hold starts: its counter is set to 0, steering is skipped, and the hold branch (step 5) runs from this tick.
2. Otherwise the navigation target is set to the walk target through the plain setter ("Walk target setters"). When the new point is outside the stored target triangle and more than 1.0 from the stored walk target (horizontal, strict), the goal locate starts its search again from triangle 0. The arrival flag is cleared. No position is written by this step.
3. The distance to the walk target is measured. If the class's arrival rule holds, the class goes to its after-arrival step.
4. Otherwise the navigation step runs (see "Navigation step").
5. Hold branch: the same body as the greeting (see "Greeting"): the idle picker for 180 ticks (3 seconds, confirmed on disc), then id 30 once, then the idle picker. Because the hold test runs only while not holding, the counter does not restart during a hold. The hold ends when the hero's horizontal distance is above 12.0 or the dot product is below -0.1 (both confirmed on disc). On exit, walking resumes on the next tick toward the same walk target. The hold does not end the class; only arrival does. Confidence: High.

End step: play the idle animation, clear the navigation wait, and end the class.

Orientation (classes 25 and 26), after arrival: each tick, if the dot product of the facing and the target orientation is below 0.99 (confirmed on disc), the character turns toward the target orientation with blend weight 0.1 (confirmed on disc). When the dot product is 0.99 or more, the class goes to its end step.

### Navigation step (per tick, not yet arrived)

Each tick the step works on these values (High). P is the character's position. c is the triangle that contains P: the stored current triangle is tested first, then its three neighbours in edge order, then the outward search of SPEC-0018 ("Locate with a last triangle"). G is this tick's goal point and t its triangle, both from the goal latch of SPEC-0029 Behavior 2b, which runs the steering look-ahead (SPEC-0029 Behavior 2a) on the walk target. The step keeps a stored current triangle C0, a stored target triangle T0 and a steering point K. All distances are horizontal. Triangle containment tests the point's x-z projection against the triangle edges and does not compare its height with the triangle plane (R-0088, High).

1. Off the mesh (c = -1): the nearest-triangle search of SPEC-0029 Behavior 2b runs from P. It also sets the current triangle. The character steers toward that triangle's centroid with weight 0.1 and plays the movement clip of its mode.
2. No goal triangle (t = -1): the idle picker runs (its clip plays), and there is no steering. For the Ixi Chief the idle picker requests animation 0 (the rearing clip).
3. Otherwise G is stored as the latch's fallback point for the next tick. Then:
   - Settled (c = C0 and t = c): T0 becomes c. Let d = |G - P|. If d < 0.25, or the block is fresh (the blocked flag is set and the blocked timer is exactly 120; see "Blocked flag"), the walk arrives: the arrival flag is set, the base state becomes 1 (through the same set-state method that puts a held Miamouse in base state 33; High), the idle picker runs, and the character turns toward the current hero's direction with weight 0.1. Otherwise it steers toward G with weight 0.3.
   - Any other case: if d < 0.25, the walk arrives as above, but without the turn toward the hero. A fresh block does not end the walk here. Otherwise:
     - Same triangle, same target (c = C0, t = T0): the character steers toward K with weight 0.1 and plays the movement clip. Then it runs the route part below, so it is steered twice in this tick.
     - Same triangle, new target (c = C0, t differs from T0): T0 becomes t.
     - New triangle (c differs from C0): T0 becomes t and C0 becomes c.
     - Route part: if C0 differs from T0, the step reads e = table(C0, T0) of the character's mesh ("Route table") and runs the route step (below) on triangle c and edge e. A character with no mesh gets -1 and does nothing more. If C0 equals T0 (it has just entered the goal's triangle), it steers toward G with weight 0.1 and plays the movement clip. It then settles on the next tick.

The table is read on every tick that is not settled, so a change of either triangle changes the edge at once. There is no separate re-plan step. The Undead Draik's walking attack sub-steps call this step without setting a locomotion clip; the clip is the one stored by its approach state.

### Route step (High)

Input: triangle c and edge e (0, 1 or 2). Let A be vertex e of c, B vertex e + 1 and C vertex e + 2 (indices mod 3), so the edge runs from A to B and D = B - A. Let m be the character's motion-record radius plus 0.25.

1. K starts as the edge midpoint (A + B) / 2.
2. If the 3D distance from P to G is below 100.0, K becomes the crossing point. Let s be the parameter on the line A + sD where the horizontal line from P toward G meets it. Let dx = G.x - P.x and dz = G.z - P.z. If dx is exactly 0, s = (P.x - A.x) / D.x. Otherwise, with q = dz / dx, s = ((P.z - A.z) + q A.x - q P.x) / (D.z - q D.x).
3. Clearance clamp. Let k1 be the horizontal distance of B from the line through C and A, and k2 the horizontal distance of A from the line through C and B. On the disc every non-degenerate triangle gives positive values. lo = min(m / k1, 0.5) and hi = max(1 - m / k2, 0.5). The parameter used is s clamped to [lo, hi], and K = A + (clamped s) D. So K stays at least m from the triangle's other two sides, unless that would push it past the midpoint. For a degenerate triangle (k = 0) K is the midpoint.
4. Behind test: if the dot product of the unit horizontal directions from P to G and from P to K is below 0, K goes back to the midpoint.
5. The character steers toward K with weight 0.1 and plays the movement clip of its mode.

The step never reads the neighbour across edge e. A table entry that names a boundary edge (876 reachable pairs on the disc) sends the walker at that edge. The confinement then pushes it back and sets the blocked flag, so it does not cross. Because only the settled case treats a fresh block as arrival, such a walk ends only through its owner's own test (for example the class-17 wander's fresh-block rule, SPEC-0038) or a new target (Medium).

### Walk target setters (High)

- Plain setter (the walk calls): if the new point lies inside the stored target triangle, nothing is reset. Otherwise, if its horizontal distance from the stored walk target is above 1.0, the goal locate's scan cursor is cleared, so the goal's triangle is searched again from triangle 0. Then the new point is stored and the arrival flag is cleared. For a flying character a height adjust follows (SPEC-0040).
- Fresh setter: clears the steering point, the stored current and target triangles (-1) and the arrival flag, then stores the point. With no navigation-mesh list in the world it sets base state 1 instead. With no mesh index, or one out of range, it raises a warning (text not specified).
- Mesh assignment: when the mesh index changes to a valid one, the current and target triangles are located again from P, and the walk target becomes P.

### Route table (High for the layout and the read; Medium-High for the meaning)

Each navigation mesh carries a route table (the mesh's "bit block" in the navigation-mesh layout of SPEC-0018). It was built offline. The clean side reads it from the disc and does not compute routes at run time.

- Layout. A mesh of T triangles holds 2 bits for every ordered pair (a, b) of triangles, a and b in [0, T). Pair (a, b) is at bit k = 2(aT + b) counted from the table start: byte k div 8, bits (k mod 8) and (k mod 8) + 1, low bit first.
- Value. The 2-bit value is the edge of triangle a (0, 1 or 2; edge v runs from vertex v to vertex v + 1) to leave by on the way to triangle b.
- (a, a) holds 0 in every mesh on the disc. Value 3 never occurs. A pair with no route over the neighbour links still holds 0, 1 or 2 (there is no "no route" code).
- Disc checks. Every one of the 998 navigation meshes on the disc has a table of at least ceil(2T²/8) bytes (together 2,830,968 bytes against 2,823,111 needed); the bytes past the pairs are fill. In all meshes of up to 60 triangles, 420,289 of 420,452 reachable pairs (99.96%) name a neighbour on a shortest route when the cost of a step is the 3D distance between the two triangles' centroids; 92 of the other 163 are near-ties. Counted in steps (one per triangle crossed), the share is 97.8% (meshes of up to 120 triangles).
- Following the table from a to b: leave a by edge table(a, b), arrive in that edge's neighbour a', and repeat with table(a', b) until the triangle is b.

Test vector (from the disc: pack 722, the type-21 mesh 2, kind 1, T = 5). Neighbours over edges 0, 1, 2 (-1 = boundary): triangle 0: (-1, 2, -1); 1: (3, -1, 2); 2: (0, 1, -1); 3: (4, -1, 1); 4: (-1, 3, -1). Table rows "from a" over b = 0 to 4: a = 0: 0, 1, 1, 1, 1; a = 1: 2, 0, 2, 0, 0; a = 2: 0, 1, 0, 1, 1; a = 3: 2, 2, 2, 0, 0; a = 4: 1, 1, 1, 1, 0. Route 0 to 4: edge 1 into triangle 2, edge 1 into 1, edge 0 into 3, edge 0 into 4. The bit of pair (1, 3) is k = 2(1 × 5 + 3) = 16: byte 2, bits 0 and 1, value 0.

### Spawn point search (High)

A routine picks a random walkable point near a base point. It has two forms. Form H: the base point B is the character's home point (its start position, SPEC-0040). Form P: B is passed by the caller. Inputs: radius R, character C. The result is a point with fourth component w = 1.

1. n = trunc(R). Two draws, each u mod n for a random draw u (0, with no draw made, when n = 0; SPEC-0008). The candidate is B + (d1 - 0.5R, 0, d2 - 0.5R): the first draw goes on x, the second on z, and y is B's.
2. Probe: a full-scan locate of the candidate on C's own navigation mesh (SPEC-0018). The probe fails when no triangle of that mesh contains the point (the x, z inside test), when C has no mesh (index -1 or out of range, or the world has no navigation-mesh chunk), or through the cursor quirk below. Any other result accepts the candidate, which is the result.
3. Retries k = 1 to 10, after a failed probe: the candidate becomes Q + 0.5 (candidate - Q), where Q is C's current position (not B; all four components, so y moves too), and is probed again. On retry 10 the result is the home point, whatever the probe returns. On an earlier retry an accepted candidate is the result. No draws are made on retries.

Cursor quirk (reproduce). The full-scan locate keeps a scan cursor (SPEC-0018). Before scanning, a cursor of -1 becomes 0, and a cursor at T or more is cleared to -1 and the call fails without testing anything. A scan that finds the point returns its triangle and clears the cursor. A scan that misses stores the old cursor + 50. A point inside the triangle the locate has cached (the walk target's triangle) is accepted at once, before the cursor rules. So on a mesh of 50 triangles or fewer, every second retry after a miss is rejected untested, unless the candidate lies in the cached triangle.

Callers (High for the arguments):

- Form H: the townsfolk wander (mode 5, R = the wander radius r, used only when r is 0.1 or more; below 0.1 the random-centroid pick of "Wander" is used); the class-17 wander (state 5, SPEC-0038); the petpet roam point (R from its own state; 40.0 in one routine).
- Form P: the class-17 agro wait in line (outer sub-state 12, SPEC-0038), first with R = the monster row's value at +0xB0 around its passed point, then, after each hold of 600 to 800 ticks, with R = 50.0 around the current hero's position.
- In every case the result becomes the walk target or a stored roam point. A home-point result is used like any other point: the character walks home.

Far centroid pick (Medium), a related routine: it draws a triangle index from 0 to T - 1 (both included) on a given mesh and takes that triangle's centroid (the mean of its three vertices, w = 1). It accepts the centroid when its horizontal distance from a point P is greater than D. It makes up to T draws and returns the last centroid even if none was accepted, or P when there is no mesh. Users: the Faerie Sisters' alternate-loop move (D = 10 from the hero, SPEC-0011) and three boss or class-17 variant routines (D = 30 and 3).

### Steering and turn rate

Each steering step sets the new facing to v / |v|, where v = w t + (1 - w) c. Here c is the current facing (unit vector), t is the unit direction to steer toward, and w is the blend weight. When the dot product of c and t is below -0.99 (almost opposite), 0.1 times the character's side axis is added to v before normalising, so the turn picks a side (Medium-High). The weight is:

- 0.3 for steering toward the goal point once the character is settled in the goal's triangle (confirmed on disc);
- 0.1 for the route step's steering point and the steering point held from the last tick, for the first tick in the goal's triangle, for steering off the mesh, for the turn toward the hero on arrival, and for the final orientation of classes 25 and 26 (confirmed on disc);
- 0.1 in the strafing modes 5 and 6 whatever weight the caller passes, or 1.0 (a snap) when the strafe direction is already within a dot product of 0.9 of the facing (SPEC-0029 Behavior 2);
- 0.05 for the IDLE turn of mode 0 (confirmed on disc);
- 0.3 for an NPC's turn toward its conversation partner (see "Talk facing");
- the monster row's pair turn weight for the villager pair (0.0 on every townsfolk row).

The direction t depends on the movement class: some classes steer toward the reverse of the move direction, and the strafing movement classes use their own rule (SPEC-0040, section 8a). The turn is a fixed fraction of the remaining difference each tick, not a rate in degrees per second.

The body's facing follows the steered direction directly (High). On each motion-record tick an NPC's facing is set to its move direction (the direction the steering last wrote, flattened to the ground plane and normalised). There is no turn floor: the hero's 0.95 facing floor (SPEC-0033) applies only to characters driven by a pad, which NPCs never are. So the only limit on an NPC's turning is the steering blend above. The facing is kept instead (not set from the move direction) in base states 16 (death), 33, 51 and 55. Example, a 90-degree turn: with w = 0.3 the remaining angle after ticks 1 to 7 is 66.80, 48.18, 34.23, 24.13, 16.96, 11.89 and 8.33 degrees; with w = 0.1 it is 83.66, 77.43 and 71.39 degrees after ticks 1 to 3 and first drops below 10 degrees (9.77) on tick 23.

### Travel by root motion (High)

The walk classes and the wander, patrol and pair modes do not move the character. They choose the locomotion animation and steer. The character travels by the root motion of that animation, the same way the heroes do (SPEC-0040, Behavior 14D).

1. **Objects.** Every character of a region other than the heroes (townsfolk, enemies and bosses) is created as an object of the shared NPC class, with a motion record of the same kind as the heroes' motion records.
2. **Order in a tick.** For each active region character, the NPC update runs first, including the per-character behavior arbiter that steps its active AI coroutine in eligible base states. The motion-record tick follows (facing, basis, displacement, velocity, driver and integrator; SPEC-0040 Behaviors 14C and 14A), then the animation pass starts a pending clip request, evaluates the playing clip and stores layer 0's root step for the next tick. The two global behavior schedulers after the region-character, selected-hero and petpet passes do not step these NPC class coroutines: they were created with auto-run disabled and the schedulers receive a zero mask. An AI change to a movement goal or steering can affect that character's motion-record tick in the same pass. A clip request made by its coroutine can be started in that character's animation pass.
3. **Step.** The per-tick step V of a ground NPC is layer 0's root step: the root record of the current key k (the whole part of the clip cursor) times the clip speed c = R x 0.5 / S (SPEC-0005, Playback 1a and 5), where S is the clip's playback divisor and R the rate the clip was started with. For townsfolk locomotion clips S = 3, so at R = 1 the step is one sixth of a key's root record per tick.
4. **Displacement.** With the record basis B1 (side), B2 (up), B3 (facing) of SPEC-0040 Behavior 14D item 3, D = Vx B1 + Vy B2 + Vz B3.
5. **Ground following.** As for the heroes: while the record is within 1.0 of the ground, D is projected onto the ground plane; then the record's gravity and integrator steps run (SPEC-0040, Behavior 14A). D is added to the record position.
6. **Timing (High).** A behavior coroutine requests a clip during the NPC update on tick n. The motion-record update on that tick uses the root step stored on the previous tick. The animation pass then starts the requested clip and stores its first root step, which the motion-record update consumes on tick n + 1. Thus a townsfolk walk requested from idle on tick 0 moves 0 on tick 0 and 0.08337 on each tick from tick 1; after T ticks including tick 0, distance is 0.08337 x (T - 1) (T = 2: 0.0834; T = 60: 4.919). A goal or steering change made by the coroutine is available to the same tick's motion-record update.
7. **Clip speed of townsfolk.** Townsfolk are in base state 1 from the bind onward, not in the parked state 42. In base state 1 the NPC update re-times the playing clip on every update, at c = R x 0.5 / S. At R = 1 this gives the same speeds as the start rate.

### Wall confinement of region characters (High)

Region characters (townsfolk, enemies and bosses) collide with the world differently from the heroes. They do not test the world's wall segments (the frame records of SPEC-0018) and they do not test cell list A. Their wall collision is the navmesh confinement step below, together with cell list B, the movable-block contact pass and the character push. The live petpet runs a branch of its own (SPEC-0018).

Order inside the motion-record tick of a region character (SPEC-0040, Behavior 14A, driver step): clear the touched flag and the floor flag; the pre-step (the wall-type value becomes -1); the navmesh confinement step; the character push (SPEC-0040); cell list B through the contact walk (SPEC-0018); the movable-block contact pass (below). All characters also get the moving-object contacts of the ground step (SPEC-0018). The heroes run the same movable-block pass after their cell list B (SPEC-0018).

**Navmesh confinement step.** Step 1 (the blocked timer) runs each tick for every region character. Steps 2 and 3 run only for a character that has a navigation-mesh index and a current triangle. Let P be the character's position, r its motion-record radius (body sizes: SPEC-0040), and L this tick's root step (Travel by root motion, item 3). All tests are in the ground plane.

1. **Blocked timer.** If the blocked flag is set, the blocked timer drops by 1. When it is below 1 the flag is cleared.
2. **Boundary edges.** For each edge k = 0, 1, 2 of the current triangle whose neighbour index is -1 (an edge with no triangle beyond it), the edge acts as a wall. The wall runs along the edge from its first vertex v0, with length len, horizontal unit normal n pointing into the triangle, and a height of 20 units around the character's height; the height is not tested.
   - **Reach.** The edge's reach is rho = r when both of these hold: the triangle's opposite vertex lies at least 2r inside the edge line, and n · v0 ≥ r, where v0 is taken as a position measured from the world origin. Otherwise rho = 0. The second condition is the edge-reach quirk (Edge cases): it is equivalent to the world origin lying at least r outside the edge line.
   - **Test at the position.** d = the signed distance of P from the edge line, positive inside; u = the coordinate of P along the edge, 0 at v0. If d ≤ rho and 0 ≤ u ≤ len, the contact response of SPEC-0018 runs with depth |d − rho| along n, push on; the blocked flag is set and the blocked timer is set to 120 ticks.
   - **Predicted test.** Otherwise, when |L| > 0.1, the same test is made at P + 0.5 L. If it passes, the contact response runs with push off (no position push; the part of this tick's step, displacement and velocity that points into the wall is still removed); the blocked flag is set and the timer set to 120.
3. **Off-mesh restore.** The character's triangle is located again (the hinted locate of SPEC-0018). If the position is on no triangle and the lock value is 0, the position is set to the saved position, the blocked flag is set and the timer set to 120. Otherwise the saved position is set to the current position.

The contact response (SPEC-0018), in short: with push on, the position moves by the depth along n; then the part of this tick's displacement that points into the wall is removed, together with the matching part of this tick's root step (taken back from the position when |L| > 0.1), and the into-wall part of the velocity; the touched flag is set.

Cross-references: the motion-record driver and the character push are in SPEC-0040; the navigation mesh, its locate, the cell lists and the contact response are in SPEC-0018.

### Blocked flag (High)

- **Setter.** Only the navmesh confinement step sets the blocked flag, always together with blocked timer 120: on a boundary-edge push (the test at the position or the predicted test) and on the off-mesh restore. No level module touches the flag or the timer.
- **Clearers.** The confinement step (when the timer drops below 1), the character's second initialisation, and the class-7 chase (SPEC-0038).
- **Fresh block.** The AI step runs before the motion-record tick (Travel by root motion, item 2). So an AI routine that sees "flag set and timer = 120" knows the character was pushed back during the previous tick's motion step. "Flag set" alone means a push within the last 120 ticks.

Readers:

| Reader | Rule |
| --- | --- |
| Navigation step (this spec) | A fresh block counts as arrival, like a distance below 0.25. |
| Class-17 wander, state 5 (SPEC-0038) | The point counts as reached when its distance is below 0.5, or on a fresh block, or when the route-found value is 0; then the pause roll. |
| Class-17 agro (SPEC-0038) | Blocked: strafe (inner sub-state 10) with a 120-tick lock (twice the tick rate). A second site sets the same lock. |
| Class-17 charge, state 8 (SPEC-0038) | Blocked ends the charge. |
| Class-7 chase (SPEC-0038) | As stated there; it also clears the flag. |
| Black Knight enraged charge (SPEC-0012) | Blocked gives sub-state 3, the knee stun (SPEC-0012's "edge-contact flag" is this flag). |
| Ixi Chief run past (SPEC-0014) | Blocked ends the run past. |
| Gelert Assassin (SPEC-0013) | On each tick the flag is set, the shadow-slip request latch is set. |
| Faerie Sisters orbit (SPEC-0011) | On each tick with body contact or the flag set, the orbit direction flips. |
| Faerie Sisters alternate-loop move (SPEC-0011) | Blocked ends the hold countdown early and forces a new point (Medium). |

The townsfolk (class 6) wander and patrol do not read the flag.

### Region characters with no navigation mesh (High)

A region character's navigation-mesh index is -1 by default; the script call CharacterNavmeshAssign gives it one (SPEC-0030), and a boss controller can also assign its own at start-up (the Gelert Assassin assigns mesh 0, and the pack 985 script assigns mesh 0 to entities 0 to 4 again; SPEC-0013). With index -1 the confinement step runs only its timer countdown. Such a character's only world walls are cell list B, the movable-block contact pass, the character push and the moving-object collision of the ground step. It passes through the world's wall segments (frame records) and the primitives of cell list A. Every probe of its spawn point searches fails, so the search always ends at retry 10 with the home point.

### Movable-block contact pass (High for the selection, Medium-High for the push)

This is the closing step of the region-character branch of the motion-record driver. It works on the character's motion record, which keeps a nearest-block index and a nearest-block distance (their readers are open).

1. The nearest-block distance is set to 1.0e9. If the owner's state word is 18 or 19, the pass ends here.
2. Each active movable block of the world (SPEC-0048) is tested by the squared horizontal distance d² from the record position to the block's point:
   - d² < 100.0 (horizontal distance below 10): the block is a candidate (at most 100 are kept).
   - When d² is below the best so far and the block's top (its y plus its height) is above the record's reference y + 0.1: the nearest-block index becomes this block and the best becomes d².
3. The candidates are sorted by ascending d². Each one that is still active gets the box push:
   - It is skipped when the owner's state word is 58, 41 or 30, or when the block has no box.
   - The box's reference point is first set from the block.
   - Test 1: point against box at the record's reference point, radius 0.75 (a fixed constant), with no growth at the top, using the pass's preferred face.
   - Test 2, only when test 1 misses: the same test at the predicted point, which is the reference point plus (velocity + force ÷ weight × 1/30) × 1/30, plus 0.5 × the stored displacement of the tick when that displacement is longer than 0.1. The vertical bound is not raised in either test.
   - On a hit with a push distance below 0.1: a contact event (code 7) with the block object's id. Then, if the block's top minus the reference y is strictly between 1.0 and 5.0, an owner hook runs with argument 1 (its effect is open).
   - Then the contact response of SPEC-0018 runs (Slide response): push on for test 1 and for side and bottom hits of test 2, off for a test-2 top hit; the velocity and force into the face are removed. The pass sits after the confinement, the character push and the cell list B walk and before the integrator (SPEC-0048, Region characters). A test-1 response that reports no change resets the preferred face to 0.
4. The nearest-block distance becomes sqrt(best d²). When no block qualified this is sqrt(1.0e9), about 31622.78 (quirk, reproduce). It stays 1.0e9 when step 1 ended the pass.

### Walk speed and movement type

The walk classes (23 to 26) set the character's movement class to its move type (the value of SetAI_MovementType, default 0) and request the animation for that class. Confidence: Medium-High for the mapping; High for the speeds.

- Townsfolk (the shared townsfolk body bank): movement class 0 plays animation id 1, class 1 id 6, class 2 id 5, classes 3 and 4 id 3 (the idle clip, so no travel).
- Other characters: movement class 0 plays animation id 2 (the enemy-style walk); the other classes request no animation, so the character keeps travelling with the clip it already plays. Their speed is the root of their own walk clip.

Townsfolk speeds at R = 1, flat ground (S = 3 for every locomotion clip; per tick = root record of key k / 6):

| Move type | Animation id | Per tick | Units per second |
| --- | --- | --- | --- |
| 0 walk (walk classes, path walk, wander) | 1 | 0.08337 (table variant B: 0.02850) | 5.00 (table variant B: 1.71) |
| 0 walk in patrol (mode 6) | 8 | 0.08337, every table variant | 5.00 |
| 1 run | 6 | 0.22504 forward, with a small side sway (mean -0.0065 along B1) | 13.50 (side drift -0.39) |
| 2 fast run | 5 | 0.32160 | 19.30 |
| 3, 4 | 3 (idle) | 0 | 0 |
| carry walk | 59 | 0.02084 | 1.25 |

Table variant B is the 148-entry townsfolk animation table (named by the placement's main-table word at +0x9E) whose id 1 names the slower walk clip (7 keys, root 175 per key, 1.197 units per 0.7 s pass); its id 8 still names the 5.00 clip, and its ids 118 and 119 have no clip. It is used by every placement of monster types 16, 18, 20, 22, 24, 27, 28, 32, 35, 41, 43 and 47: 214 of the 666 townsfolk placements (117 of code 0, 41 of code 6, 24 of code 5, 25 of code 7). The other tables give id 1 the clips 104 (types 25, 26, 44, 45 and 21, 23, 30, 33, 37, 50) or 107 (female types 17, 19, 34 and most type 29), both at 5.00. So script walks, path walks and wander of variant B characters run at 1.71 units/s, and their patrols at 5.00. Patrol uses id 8 only for movement class 0; with move type 1 or 2 a patrolling character uses the run or fast-run clip (the three Cogham villagers, entities 3, 5 and 7 of pack 842, patrol at 19.30 units/s). The walk, run and fast-run speeds equal the heroes' slow walk, walk and run speeds (SPEC-0040, Behavior 14C). At another rate R the speed scales by R.

### Path walk (class 35)

- Route (High). The path is path number +0xB8 of the character's placement in the area's path list (SPEC-0006, Tagged tables, tag 0x0A): a run of entry records of the pack's entry list. A walk call with a path index of 0 or more first writes that index into the placement word (SPEC-0030 row 111); a negative index leaves the placement's own value, which is how a script walks a character along the path its placement names. The route is valid when 0 ≤ index < the path count: the first point is entry A[index] and the point count is B[index] − A[index] + 1. A route is set up once in the life of a character (the NPC reset clears it): a later walk call with another index changes the placement word but not the route already set up (Medium-High).
- Start (High). The point index starts at 0 and the reached flag is clear, so the first target is point 0 (unlike the patrol of mode 6, whose first target is point 1). A character that stands elsewhere first walks to point 0. The class does not set the movement class itself (unlike classes 23 to 26): it keeps the one the townsfolk routine or the previous walk class last set, which is the character's move type (0 unless a script called SetAI_MovementType). With move type 0 it plays townsfolk walk id 1: 5.00 units per second at R = 1, 1.71 for table variant B (see Walk speed and movement type).
- Each tick that is not a hold (High). With the reached flag clear, the navigation target is the current point, the navigation step runs, and when the horizontal distance to the point is then below 0.5 the reached flag is set. With the reached flag set at the start of the tick: if the current point is the last one, the class ends (End, below); otherwise the index goes up by 1, the flag is cleared and the navigation target becomes the new point (the navigation step runs from the next tick). The loop word +0xBC is not read by the path walk: loop and back-and-forth placements walk alike. There is no pause at a point, no random stop and no position snap.
- End (Medium-High). On the tick after the last point is reached the class ends with no further animation request (unlike the classes 23 and 25, it does not snap or play an idle): the character's class and priority reset to 0 and the parked townsfolk class resumes (Class requests have priorities, below). From then on IsEntityWalkToPointActive is false for it and a WAIT loop of the walk call stops waiting. A path of one point is walked to and ends the same way. The character is not removed: the cycle zones of pack 1023 deactivate their finished walkers (SPEC-0006, Cycle load).
- No route (Medium-Low). With an invalid index no route is set up, the reached flag stays clear, and the class steers toward entry 0 of the pack's entry list (a point past the list also reads entry 0), never advances and never ends. No shipped walk call names an invalid path.
- Hold (High). The hold test and hold branch are those of the walk-to-point classes and run before the arrival test each tick.
- The script query IsEntityWalkToPointActive (SPEC-0030) is true while class 35 is the character's current requested class, as it is for classes 23 to 26 (High).

### Wander (mode 5) (High)

Values from the AI block: radius r, pause range [Pmin, Pmax] and pause chance (see "Placement values").

1. Each tick: request the townsfolk walk animations for speed code 5 (walk id 1). If the pause timer is above 0, it drops by 1.
2. Point choice, when no point is held:
   - If r is below 0.1, the point is the centroid of a uniformly chosen cell of the character's current navigation mesh.
   - Otherwise the point is home + (a - r/2, 0, b - r/2). Here a and b are each a whole number drawn uniformly from 0 to trunc(r) - 1 (0 when trunc(r) is 0), and home is the character's home point.
   - If that point is off the mesh, it moves halfway toward the character's current position and is tested again, up to 10 times. The 10th retry gives the home point whatever its test returns. This is form H of the spawn point search (see "Spawn point search", including its cursor quirk).
3. Walking (pause timer below 1): the navigation target is the point, and the navigation step runs. Let d be the horizontal distance to the point. Walking continues while all three of these hold:
   - d is 1.5 or more;
   - the character is not in the last cell of its route within 2.5 of an open (boundary) edge (Medium);
   - the goal latch of SPEC-0029 Behavior 2b is set. The latch clears when the goal locate has tested every triangle of the mesh without finding the goal point, so "no route" means "the goal point is on no triangle" (High for the latch; Medium for the reading of this test).
   Otherwise the point counts as reached. Then a roll from 1 to 100 is made. If the roll is below the chance, the pause timer and the fidget timer each draw from [Pmin, Pmax]. Otherwise there is no pause, and a new point is picked on the next tick.
4. Pausing (pause timer 1 or more). The pause timer and the fidget timer were drawn together from [Pmin, Pmax] (step 3). Each pausing tick does one of these:
   - Fidget timer 1 or more: the idle picker is requested (for a townsfolk always id 0). When id 2 has an entry in the character's animation table, the fidget timer drops by 1.
   - Fidget timer below 1, fidget not yet requested: id 2 is requested once, and the fidget state becomes "requested".
   - Fidget requested: nothing is requested until id 2's clip reaches progress 0.99. On that tick the fidget timer is drawn again from [Pmin, Pmax], the fidget state is cleared and the idle picker is requested.
   The NPC reset sets the fidget timer to 0 and the fidget state to "not started".

   On the townsfolk body bank (body 526) the fidget is a no-op. Ids 0, 2 and 11 all name clip 48, the standing idle. The request for id 2 starts nothing while clip 48 already plays, and the wait ends at clip 48's own loop end (within one loop, 132 ticks at R = 1). So the character shows the standing idle throughout the pause. (High)

   Warren is the exception (region resource 999, family 0, wander, pause 60 to 150 ticks, chance 50). His id 2 names his clip 1 and his id 0 his clip 0, so he is the only class-6 placement whose pause fidget shows a different clip. Clip 1 is a 30-frame one-shot bow, lasting 116 ticks (1.93 s) at R = 1. Its torso and head fold forward (maximum torso angle about 53 degrees; head height falls from about 2.23 to 1.81). Its root x, y and z records are zero on every key, so the fidget adds no root travel and Warren does not drift. The bow label is Medium confidence from the decoded pose measurements; exact style is not confirmed in an in-game preview. The other four class-6 wander and patrol placements outside body 526 have all-zero pause values and never pause. (High for timing, clip choice and zero root travel; Medium for the bow identification)

On disc there are 51 code-5 placements. Their radii run from 2 to 200; 10 is the most common (22 placements), then 100 (11). Most pause ranges lie between 30 and 600 ticks, and the usual pause chance is 50. 17 placements have chance 0 and never pause.

### Patrol (mode 6) (High)

Values: the route (+0xB8), the loop mode, the point-pause range, the random-stop chance and the random-stop range (see "Placement values").

These timing values are the same spawn-record block, +0xC4 to +0xD4, that the class-17 default state 6 reads (SPEC-0038 open question 20): +0xC4 and +0xC8 are the hold (point-pause) range, +0xCC the chance (a stop or move happens when a roll from 1 to 100 is below it) and +0xD0 and +0xD4 the move (random-stop) range (High).

1. Each tick: request the townsfolk walk animations for speed code 6 (walk id 8). The route is started once, if it is valid (the placement's +0xB8 names a path of the area's path list: the first point is the path's first entry and the count is its point count; SPEC-0006, Tagged tables, tag 0x0A), and the reached flag is set to 1. If the pause timer is above 0, it drops by 1.
2. When the reached flag is set, the path step advances the index and the new point becomes the target. The first step happens on the start tick, so a patrol's first target is route point 1, not point 0. In loop mode, the index goes up by 1 and wraps to 0 after the last point. In back-and-forth mode, it starts forward, runs to the last point, then back to point 0, then forward again.
3. Walking (pause timer below 1): the navigation step runs toward the point. When the horizontal distance is below 0.5, the reached flag is set, and the pause timer and the fidget timer draw from the point-pause range. Otherwise a roll from 1 to 100 is made, only on a tick on which the character's awareness gate word is 0 (the roll is skipped while it is not); if the roll is below the random-stop chance, both timers draw from the random-stop range (+0xD0 to +0xD4). A random stop does not set the reached flag, so the same point stays the target afterwards. A pause draw P holds the walker for P - 1 ticks, because the timer drops by 1 at the start of the next tick. The next point is chosen on the first tick after arrival (the index advances while the pause runs), so the pause only delays the walk. With an invalid path index no route is set up and the walker steers to entry 0 of the pack's entry list and rests there (Medium-Low). The facing does not change during a pause.
4. Pausing: as wander step 4, with fidget id 11 in place of id 2. After the fidget's clip reaches progress 0.99, the fidget timer is drawn again from the point-pause range. On body 526, id 11 names clip 48, so the fidget is a no-op as in wander. Id 7 is never requested, because its gate is only ever cleared.

On disc there are 121 code-6 placements, and every one has a route index of 0 or more. 87 loop and 34 go back and forth. 63 have every timing value (+0xC4 to +0xD4) 0, so they never pause (61 if the unused +0xC0 word is counted too). Patrol walking is id 8 at 5.00 units per second in every table variant (clip root 512 per key, 8 keys of 6 ticks, 4.002 units per 0.8 s). Point pauses of a few seconds are typical, for example 300 to 450 ticks or 120 to 300 ticks. 13 placements have a random-stop chance, from 5 to 50.

### Villager pair (mode 3) (High)

1. Search. It runs in the pre-step, in modes 5 and 6 only. Each tick the pair's two counters drop by 1, the cooldown among them. When the cooldown is below 1, the search walks the region's character list in order and takes the first other character that:
   - is eligible;
   - has the same navigation index;
   - lies at a horizontal distance below the searcher's pair radius (monster row +0x44).
   On a match, both characters get mode 3 (role 0) through the mode setter, and each stores the other's id as its partner. If there is no match, the search runs again on the next tick.
2. Eligible. All of these must hold:
   - the social gate is 0;
   - the character is playing one of the animation ids 0, 1, 2, 8, 9, 10, 11, 59, 60, 107 or 112 to 116;
   - its mode is not 3.
3. Each tick in mode 3, the pair ends if the partner is missing, is not in mode 3, or has another navigation index.
4. Role 0 (approach): movement class 0, walk to the partner's current position. When the pair is closer than 6.0 (confirmed on disc) and the partner's role is still 0, a draw of 0 or 1 gives roles 1 and 2, or 2 and 1. When a villager's role is set from 0, it draws its talk timer, an integer from 300 to 720 ticks (confirmed on disc; 5 to 12 seconds), and picks a gesture (SPEC-0040).
5. Role 1: when its gesture finishes, it becomes role 2 and its partner becomes role 1, so the turn passes. Role 2: when its gesture finishes, it plays idle id 0.
6. In roles 1 and 2, the villager turns toward its partner each tick with the pair turn weight (monster row +0x84, or +0x88 for flyers) and counts its own talk timer down. When the timer reaches 0, both villagers go to role 3.
7. End (role 3 or any end condition): the cooldown draws an integer from 600 to 1800 ticks (confirmed on disc; 10 to 30 seconds) and the role becomes 0. A mode 0 request follows, so the code returns on the next tick, and the idle picker is requested.
8. On disc the pair radius and the pair turn weight are 0.0 on every class-6 placement. No pair ever forms, so the game never shows a villager pair. A clean side keeps these rules for completeness.

This is a social pairing between two villagers. It never involves the hero.

### Animation ids a townsfolk plays, by mode (High)

For a townsfolk on the shared body bank (main family -7). Clip numbers are clips of that bank.

| Mode or routine | Ids requested |
| --- | --- |
| 0 | 0 (clip 48) |
| 5 (wander) | walking 1 (clip 104, 108 or 107 by table variant); pausing 0; fidget 2 (clip 48) |
| 6 (patrol) | walking 8 (clip 104 or 107); pausing 0; fidget 11 (clip 48); never 7 |
| 13 (greeting) | 0 for 180 ticks, then 30 once (clip 71), then 0 |
| 3 (villager pair) | the gestures of SPEC-0040, then 0 (no pair forms on disc) |
| 10, 11, 12 | 65, 97, 98 |
| 1, 2, 4, 7, 8, 9 | nothing |
| Script walks (classes 23 to 26 and 35) | walk slots 1, 6, 5 or 3 by move type; the hold as mode 13; 0 on arrival (classes 23 to 26; class 35 requests nothing when it ends) |

A townsfolk never requests ids 115 to 119, 141 or 142, because the idle picker returns 0 for it. The picker's mode-19 test runs before the flag test: with mode word 19 it gives id 1 (a walk clip), never 141, since no class-6 row can fly. That happens only if a script sets behavior 19 on a townsfolk that then runs a routine that calls the picker (Medium; no such script is known).

### Base states and script control (High)

- Townsfolk are in base state 1 from the bind onward. Every bind (streamed, forced or spawn by key) sets base state 1 before the first draw. Base state 42 is the parked state: creation, reset, release and inactive slots. A parked character is neither updated nor drawn.
- The other base states townsfolk take:
  - 25: conversation, from SetConversationTarget until EndConversation, which restores the saved state;
  - 33: script control, set by GrabEntityControl, PlayEntityAnim, MoveEntity when its control argument is not 1, and the end of a LoopEntityAnim when the control flag is 0;
  - 55: LoopEntityAnim;
  - 16: death, only if a script kills the character.
- Scripts take and give back control. GrabEntityControl sets base state 33. ReleaseEntityControl and ReturnEntityAnimToIdle set base state 1. The walk calls also set base state 1 and store the control flag.
- The behavior coroutine is stepped in every base state except 16, 25, 33, 36, 42, 53 and 55. Base states 17 and 31 step it after their own step. In the excluded states, the mode machine, the walks and the greeting are frozen where they stopped, and they resume from there when the state returns to one that steps them.
- Script hold during a walk (Medium-High; SPEC-0030 section 4). A character whose walk class (23 to 26 or 35) is applied and that is then put in base state 33:
  - The walk freezes. The walk stays the applied class, but its routine is not stepped: no steering, no arrival test and no hold test. The facing is not updated in base state 33.
  - The clip already playing keeps playing at its start rate, and its root motion keeps carrying the character straight along its frozen facing.
  - On the first tick that clip's progress is 0.99 or more, the character requests its default idle animation. The idle has no root travel, so the character stands. With a looping walk clip the drift lasts at most one clip cycle.
  - When the state returns to 1, the walk routine resumes where it yielded, requests the movement clip again and goes on to the same target.
  - IsEntityWalkToPointActive stays true throughout, and a WAIT loop of the walk call keeps waiting.
- Class requests have priorities: a script walk is priority 2 and the default class is priority 1. A higher-priority request made while a coroutine runs parks that coroutine and starts the new class. When the new class's coroutine ends, the class and its priority reset to 0 and the parked coroutine resumes where it yielded. So a townsfolk character keeps its mode and its wander point across a script walk, although its class reads 0 during the walk (Medium-High).

### Talk facing (High)

- SetConversationTarget puts the hero and the character into base state 25 (SPEC-0022). The character's partner is the hero only when the call's second argument is 1.
- In base state 25, an NPC with a partner and without the pose flag (the animation list of "Greeting") turns each tick toward the horizontal direction to its partner. It uses the steering rule with weight 0.3: v = 0.3 t + 0.7 c, with the anti-parallel side nudge, then normalised. An NPC with no partner, or in a pose, does not turn.
- The hero in base state 25 uses its own routine with the same rule and weight 0.1. The "0.1 of the remaining angle" of SPEC-0022 is this hero weight. It equals the vector blend only for small angles. Both weights are confirmed on disc.

### Proximity callbacks and items

- A level script registers a callback for an entity index with RegisterEntityProximityCallback (SPEC-0030). The registration stores the callback and the index in the world's callback table. An index outside the table is ignored.
- A callback body runs its own tests (story flags, possession of an item, a level prompt) and may call GiveItemToHero.
- Eleven callbacks in ten modules call GiveItemToHero within their call tree (table below).
- A level prompt returns true when the player answers yes (answer 1). Where the table says "prompt", the item is given only on the yes path.
- The registration record holds the callback, the index (the descriptor id) and a flag (the registration's bool argument).
- A registered callback fires through the hero's interaction scan for the hero's current candidate character (the target choice of SPEC-0022). It fires only while the hero is not busy and not in base state 25.
  - With the flag set, the candidate's distance from the hero must be below 5.8 and Action must be pressed. Within 5.8 and without Action, hint 8 is shown.
  - With the flag clear, it fires with no distance test whenever that character is the candidate.
  - A callback does not fire again while its previous run is still going.
- Confidence: High for these rules, Medium for which characters can be the candidate. The 400-unit proximity test of SPEC-0030 section 3 belongs to a different record.

| Module | Callbacks | Item calls in the callback's tree | First-argument item values seen | Gates (in words) |
| --- | --- | --- | --- | --- |
| 855 | 1 | 1 | 122 | Two story flags; possession of item 22 |
| 880 | 1 | 8 | 175 | Treasure-map flags; prompt on some paths |
| 909 | 1 | 1 | 86 | Two story flags; hero not talking |
| 951 | 1 | 1 | 32 | Two story flags; possession of item 98 |
| 1023 | 1 | 6 | 175 | Cloak and treasure-map flags; prompt on some paths |
| 1133 | 1 | 1 | 60 | Seven story-flag conditions |
| 1139 | 1 | 6 | -1 (a call with no item), 50, 54, 55, 56, 152 | Prompt 10 gates item 50; prompts 11, 12, 13 and 14 gate items 152, 54, 55 and 56 in that order; flags on some paths |
| 1148 | 2 | 3 (first callback), 8 (second callback) | 39, 120 (first); 42, 98, 99 (second) | Story flags; possession test; prompt on some paths |
| 1230 | 1 | 6 | 175 | Treasure-map flags; prompt on some paths |
| 1300 | 1 | 1 | 61 | Two story flags |

The second argument of GiveItemToHero varies by call and is not named here. The first-argument value 175 appears to be a map award (Medium).

Modules that register a proximity callback but give no item on approach:

- Modules 722 and 758: their callback trees contain no item call.
- Module 678: its item calls come from debug-mode and button-gated routines, not from its proximity callback.

### Every way a non-enemy character moves (High unless marked)

- Path walk, class 35 (WalkEntityAlongPath): the Meridell and Brightvale cycle-zone villagers (28 walkers in pack 1023, 3 in pack 758) and five scripted walkers of pack 1300. Speed: walk id 1 (5.00 or 1.71 units/s). Ends after the last point (Path walk).
- Walk to a locator, classes 23 to 26 (WalkEntity, WalkEntityAndOrient): 76 and 67 call sites in 38 and 34 modules, and the waypoint walks of the shared entity-control routine of 27 modules (stop radius 1.1, so class 24 or 26). Speed: the movement class's walk id. Interior townsfolk of packs 1008, 1011, 1014 and 1020 are also mode-6 patrollers; a script walk parks the patrol and the patrol goes on afterwards.
- MoveEntity: places the character at once (116 call sites, 26 modules); no walk.
- Autonomous: wander (mode 5, 51 placements), patrol (mode 6, 121), greeting hold (mode 13). The villager pair never forms. Codes 7, 9, 10 and 12 stand or pose.
- Followers: only the three Ixi Spearmen of Cogham Village (class-17 state 9, SPEC-0038), which follow the villagers Rose, Gershwin and Giovanni the Explorer at a stay distance of 5.0 and copy their gait. Those three villagers patrol at fast run (move type 2 set once by the pack's slot thread). Townsfolk follow nobody. The petpet follows the hero (SPEC-0047, class 34).
- Heroes in cutscenes: WalkHero and WalkHeroAndOrient walk at the slow-walk speed (5.0 units/s), RunHero at the run speed (19.29); arrival below 0.25 (SPEC-0033).
- There is no acceleration or deceleration, no run-versus-walk switch by distance (only SetAI_MovementType changes the gait), no teleport-if-far and no escort.

### Walker interruptions and navigation limits (High)

- Mesh. A townsfolk character gets its navigation mesh once, when its townsfolk routine first runs: the first mesh of kind 1, in the world's list order, with a triangle that contains its position (x and z only). A character outside every kind-1 mesh keeps no mesh and never walks (2 of the 172 wander and patrol placements). All 28 cycle walkers and the 9 plain-zone patrollers of Meridell Village stand inside a kind-1 mesh, and every point of their paths lies in the same mesh.
- Hero in front: a class 23 to 26 or 35 walker stops (idle for 180 ticks, then the id 30 gesture, which has no root travel) while the hero is within 6.0 horizontally and in its front half, and goes on toward the same point when the hero is farther than 12.0 or behind it (dot below -0.1). A hero behind a walker never stops it.
- Conversation: SetConversationTarget puts the character in base state 25. Its walk routine is not stepped, it requests idle id 0 every tick (the walk clip's last root step is applied once), and it turns toward the hero with weight 0.3. After EndConversation the routine resumes where it yielded, with the same point index.
- Script hold (base state 33): as in Base states and script control.
- Other characters and the hero: the character push moves a region character out of any overlapping character, the hero included, along the overlap direction; it does not stop or re-route. Doors and walls obey Wall confinement.

### Path end by caller (Medium-High)

- Cycle-zone threads (packs 758, 1023): the thread's next pass sees the walk as finished and deactivates the villager; when none is left the next candidate group loads.
- Pack 1300 scripted walks: the walker stays at the last point; a code-0 character then idles and turns back to its placement facing at 0.05 per tick.
- A code-6 or code-5 character goes back to its patrol or wander. Classes 23 and 25 snap to the target, 24 and 26 do not (Behavior classes).

### Walkers on the disc (High for counts)

- 666 townsfolk placements: code 0 in 414, code 5 in 51, code 6 in 121 (87 loop, 34 back and forth), code 7 in 67, code 9 in 5, code 10 in 6, code 12 in 2. Patrol speed: 120 at 5.00. Wander speed: 23 at 5.00, 24 at 1.71, 4 on other banks.
- Pack 1023 cycle walkers: 14 at 5.00 and 14 at 1.71 (SPEC-0045 lists each). A 1.71 walker needs up to 239 s for its route (Grizwuld, 408.4 units); a 5.00 walker 27 to 71 s, a 1.71 walker 107 to 239 s.
- Pack 758: the one cycle zone loads region 762 (Lilly, path 4, 11 points, 216.0 units, 5.00; Corwill, path 5, 9 points, 226.3 units, 1.71, standing 1.4 from point 0) or region 763 (Minerva, path 7, 15 points, 231.3 units, 5.00). Lilly and Corwill are also patrollers.
- Pack 1300: Seradar (path 1, 4 points, 23.1 units) and four Old Ladies (paths 6, 7, 9 and 10: 2, 3, 3 and 3 points; 11.5, 26.9, 17.7 and 17.9 units), all 5.00 units/s, set walking by the scenes of that pack.

### What a port most likely gets wrong (Meridell townsfolk)

1. No path list: the villager's own path number (placement +0xB8) must index the pack's path list; WalkEntityAlongPath with -1 uses it.
2. IsEntityWalkToPointActive must be true in the same script tick as the walk call, before the walker's next AI step. The cycle thread asks right after issuing the walks, and a false answer deactivates the villager at once.
3. The class must end after the last point (class back to 0), or the group never reloads; it must not end earlier and must not loop.
4. The first target is point 0 (the patrol starts at point 1). Loop mode is ignored by the path walk.
5. The 150-tick idle before the walks, and the single start of all walkers of the group together.
6. No mesh: without the kind-1 containment rule nobody has a mesh and nobody moves.
7. Speed: root motion of walk id 1, which is 1.71 for 14 of the 28 Meridell walkers; id 8 (5.00) for patrols; do not use the heroes' speed or one speed for all; move type changes the gait only through SetAI_MovementType.
8. The hold: villagers stop in front of a hero within 6.0 and go on once the hero is beyond 12.0 or behind; a hold that never ends looks like a frozen village.
9. Only cycle-zone villagers (spheres 31 to 34) walk paths by script; the plain-zone villagers stand, wander or patrol by their codes.
10. One-tick lag between a clip request and the first root step.

## Edge cases and error handling

- Mode 0 remap: a mode 0 request through the mode setter, with a nonzero code, gives mode 23 only on a monster row whose remap switch is 0. No townsfolk row has that, so townsfolk get mode 0 and then their code.
- Modes with no per-tick body: nothing happens per tick, and the character keeps the animation it is playing. Townsfolk with code 7 or 9 therefore stand in their current animation.
- A hero standing within 6.0 in front of a townsfolk character restarts the greeting counter on every tick, so id 30 is never requested while the hero stays there.
- The greeting runs in every mode except 10 to 12, including wander and patrol, and on exit the character goes back to its code's mode on the next tick.
- Stun and mode 4. The mode word of a townsfolk character is the same word as the class state of the common enemies. The shared status apply (the one every region character uses) would therefore set mode 4 for status 0 on a townsfolk character, and the status tick would freeze its clip rate (factor 0) every tick while the timer runs. Nothing reaches it on disc: the hero's swing, shots and Mote walks skip townsfolk (they are not hit candidates), no level-script call applies a status to a character, and no on-disc SetAI_Behavior code selects 4. Once in mode 4 a townsfolk character would stay there, because no class-6 body leaves it. A rewrite that never applies stun to townsfolk matches the disc (High for the shared apply and the shared word; High that no source exists; see Open question 26).
- Villager pairs never form on disc, because the pair radius is 0.0 and the distance test is strict.
- Arrival tests are strict. A distance exactly equal to the threshold does not arrive.
- The timer ranges are 5 to 12 and 10 to 30 times the rate variable (confirmed on disc). At 60 ticks per second they are the integers 300 to 720 and 600 to 1800.
- A class 35 walk with no valid path stays on entry 0 of the pack's entry list and never ends (Path walk, No route).
- A proximity registration with an index outside the callback table is ignored.
- A walk call that targets a point beyond the world's locator count does nothing (SPEC-0030).
- Arrival versus step length: the per-tick steps (walk 0.0834, run 0.2250, fast run 0.3216) are all shorter than the diameters of the arrival circles (2.0, 1.0 and 0.5), so a walker heading straight at its point always has at least one tick inside the circle. Only an off-centre pass at fast run can miss the 0.25 circle of the navigation step.
- Move types 3 and 4 play the idle clip: a character given one of these types and a walk call steers but does not travel.
- The step uses the clip of the previous tick. A walker that arrives and switches to idle still moves one more step with the walk clip.
- **Edge-reach quirk (reproduce).** A boundary edge gets the full reach r only when the world origin lies at least r outside its line (n · v0 ≥ r) and the opposite vertex is at least 2r inside. For every other boundary edge the reach is 0: a character walking into it is pushed back exactly onto the edge line, so half its body stands outside the mesh. On a mesh whose edges are far from the world origin with the origin outside them, the same character stops a full radius short of the edge. The cause: the original measures the distance of the edge's own vertices with a fourth component of -1.0 (the stored neighbour index) instead of 1, which gives 2 n · v0 instead of 0. The clean side must reproduce the resulting rule, not the intended one.
- Region characters never touch the world's wall segments. Where a navigation-mesh boundary does not follow a wall, they can walk through that wall; where it does, only the confinement holds them.
- The predicted test (P + 0.5 L) does not push. It only takes back the into-wall part of the step, so a character just inside an edge with reach 0 stops before crossing it.
- Off the mesh with a lock value that is not 0, no restore happens and the off-mesh position becomes the saved position.
- A character with no navigation-mesh index (or no current triangle) gets only the blocked-timer countdown of the confinement step; its only world walls are cell list B, the movable-block contact pass, the character push and the ground step's moving-object collision (see "Region characters with no navigation mesh").
- **Spawn search cursor quirk (reproduce).** After a full-scan miss the locate's cursor is 50 higher; on a mesh of 50 triangles or fewer the next probe is then rejected without a test (unless the point lies in the cached triangle), so every second retry is wasted.
- **Spawn search fallback.** Retry 10 returns the home point even when its own probe would accept the candidate. A character with no mesh always gets its home point.
- The spawn retries move toward the character's current position, not toward the base point, and they move y as well.
- **Nearest-block distance quirk (reproduce).** With no qualifying movable block the motion record's nearest-block distance is sqrt(1.0e9), about 31622.78; when the owner's state word is 18 or 19 it stays 1.0e9.
- A movable block at a horizontal distance of exactly 10.0 is not a candidate (the test is strict).
- **Repeated blocked readings.** The blocked flag stays set for 120 ticks after one push. A reader that acts on "flag set" (not on a fresh block) acts on each of those ticks; the Faerie Sisters' orbit flips its direction on every one of them (SPEC-0011).
- The edge wall's 20-unit height is not tested, so the confinement acts at any height.

## Test vectors

Synthetic values; they are not copied from the disc.

- TV-0056-1 (class 23): distance 0.99 at a decision point: arrived. Distance 1.00: not arrived.
- TV-0056-1a (class 23 arrival): character at (10.0, 2.0, 5.0), walk target (10.5, 2.0, 5.5): horizontal distance 0.707 is below 1.0, so the character is set to (10.5, 2.0, 5.5) and the class ends. The same walk with class 24 and stop radius 1.1 ends with the character still at (10.0, 2.0, 5.0).
- TV-0056-R127a (path walk, disc values from pack 1023; High): Wymond (placement +0xB8 = 8) is walked with path -1. Path 8 is entries 146 to 156, 11 points. The first target is point 0 at (-8.57, 29.37, 655.11), the point he stands on (horizontal distance 0, below 0.5), so on the next tick the reached flag is seen set, the index goes to 1 and the target becomes (-25.10, 29.37, 654.89). He covers 173.3 units of segments (horizontal) at 5.00 units per second, about 34.7 s plus the turns of the navigation route, and when point 10 is reached the class ends one tick later and IsEntityWalkToPointActive turns false.
- TV-0056-R127b (patrol start; High): a Skeleton Archer of pack 790 (default state 6, +0xB8 = 4, +0xBC = 1) has a route of 2 points (entries 27 and 28). The start tick sets the reached flag, so the first step moves the index from 0 to 1 and the first target is point 1 at (2154.02, 56.42, -940.91), not point 0 at (2237.60, 56.42, -941.08). On reaching it the index goes back to 0, then 1, and so on (back and forth). With +0xBC = 0 it would go 1, 0, 1, 0.
- TV-0056-R127c (path walk timeline, pack 1023, straight-line model; Medium): Wymond stands at (-8.573, 27.474, 655.108), 0.004 from point 0. Tick 0 is the first step of the walk class: the flag is set at once. Tick 1: the index goes to 1 (target (-25.10, 29.37, 654.89), 16.53 away). Tick 2: first navigation step, walk clip requested. Tick 3: first displacement 0.08337. Points 1 to 10 are flagged on ticks 196, 606, 880, 998, 1021, 1038, 1156, 1431, 1841 and 2035; tick 2036 ends the class (33.9 s). Steering arcs add a few ticks.
- TV-0056-R127d (scripted path walk, pack 1300, straight-line model; Medium): an Old Lady placed at (-92.474, 1533.348, -102.140), facing (0.984, 0, -0.179), is sent along path 10 with WAIT: points (-83.719, 1535.248, -107.290), (-84.944, 1535.248, -117.876), (-91.995, 1535.248, -119.354). Flagged on ticks 117, 242 and 325; the class ends on tick 326 (5.4 s) and the script resumes. Variant B contrast: the same 28.0 units take 982 ticks at 1.71.
- TV-0056-R127e (patrol speed): a type 20 guard (table variant B) walks a script walk at 0.02850 per tick (1.71) and patrols at 0.08337 (5.00).
- TV-0056-2 (class 24, stop radius 1.1): distance 1.09: arrived. Distance 1.11: keeps walking.
- TV-0056-3 (class 35): point distance 0.49: reached. Distance 0.50: not reached.
- TV-0056-4 (steering on the mesh, w = 0.3): facing c = (0, 0, 1), direction t = (1, 0, 0). v = (0.3, 0, 0.7), so facing = (0.39392, 0, 0.91915).
- TV-0056-5 (orientation, classes 25 and 26): dot 0.98 keeps turning with weight 0.1. Dot 0.99 ends the orientation.
- TV-0056-6 (code from the placement): a townsfolk placement with code 7 binds in mode 0. On its first class tick the pre-step sets mode 7, and it keeps the animation it is playing. Codes 0, 5 and 6 give modes 0, 5 and 6 the same way.
- TV-0056-7 (SetAI_Behavior change): a character in mode 5 receives code 6. The code changes, so the mode is 0 directly after the call, and on the next class tick it is 6.
- TV-0056-8 (mode 0 remap): a mode 0 request through the mode setter while the code is 5, on a row whose remap switch is 0, gives mode 23. On a townsfolk row (switch 1) it gives mode 0, and the next tick gives 5.
- TV-0056-9 (villager timers at 60 ticks per second): a talk timer draws an integer in 300 to 720. A cooldown draws an integer in 600 to 1800.
- TV-0056-10 (re-plan): stored walk target (0, 0, 0), new points outside the stored target triangle. New target (1.2, 0, 0): the goal locate restarts from triangle 0. New target (0.9, 0, 0): its scan position is kept. A new target 5.0 away that lies inside the stored target triangle: also kept.
- TV-0056-11 (proximity item): the callback in module 1139 that gives item 50 gives it only when prompt 10 answers yes and the flag test passes. With prompt 10 answered no, that path gives nothing.
- TV-0056-12 (townsfolk walk timing): walk requested during the NPC update on tick 0 from idle, R = 1, flat ground, facing straight at a far point. The motion-record tick on tick 0 uses the idle root step, so distance after tick 0 is 0. After tick 1: 0.0834. After tick 2: 0.1667. After tick 60: 4.919.
- TV-0056-13 (displacement): step (0.0184, 0, 0.2250) with facing (1, 0, 0): B1 = (0, 0, -1), B3 = (1, 0, 0), so D = (0.2250, 0, -0.0184).
- TV-0056-14 (steering, 90-degree turn, w = 0.3): remaining angle 66.80 degrees after tick 1 and 8.33 degrees after tick 7.

Navmesh confinement (synthetic; r = 1.175, no velocity; L is this tick's root step):

- TV-0056-15 (near triangle (-50, 0, -50), (50, 0, -50), (-50, 0, 50), all three edges boundary; every edge has reach 0 because the world origin is not at least r outside any edge line: it lies inside the two straight edges and on the diagonal): positions 1.0 or 0.5 inside the edge x = -50: no push. (-51, 0, 0): pushed to (-50, 0, 0); blocked flag set, timer 120. (0.5, 0, 0.5), just past the diagonal: pushed to (0, 0, 0).
- TV-0056-16 (same triangle, predicted test): (-49.9, 0, 0) with L = (-0.3, 0, 0): no hit at the position (0.1 inside, reach 0); P + 0.5 L = (-50.05, 0, 0) hits; no push, the step is taken back: position (-49.6, 0, 0).
- TV-0056-17 (same triangle, off-mesh restore): saved position (-45, 0, 0), lock value 0, the character at (-51, 0, -51), outside the span of both straight edges and far inside the diagonal: no edge hit; off the mesh, so the position is restored to (-45, 0, 0), blocked flag set, timer 120.
- TV-0056-18 (far triangle (100, 0, 100), (200, 0, 100), (100, 0, 200)): the edges x = 100 and z = 100 have reach 1.175 (the world origin is 100 outside each). (101, 0, 150): pushed to (101.175, 0, 150). (99.5, 0, 150): pushed to (101.175, 0, 150). The diagonal edge has reach 0 (the origin is inside it): a position 0.99 inside the diagonal is not pushed.

AI modes, greeting and conversation (synthetic):

- TV-0056-19 (greeting entry): townsfolk facing (0, 0, 1). Hero at (0, 3, 5) relative: horizontal distance 5.0, dot 1, so the greeting starts. Hero at (5.9, 0, 0): dot 0, so it does not. Hero at (0, 0, -4): dot -1, so it does not. Hero at (0, 0, 5) but in base state 33: it does not.
- TV-0056-20 (greeting timing): after a greeting starts, the hero stays 8.0 in front. Ticks 1 to 180 request the idle picker, and tick 181 requests id 30. If the hero instead stays 5.0 in front, the counter restarts every tick and id 30 is never requested.
- TV-0056-21 (greeting exit): hero at 10.0 with dot -0.05: the greeting goes on. Dot -0.2: it ends. Hero at 12.5 with dot 1: it ends.
- TV-0056-22 (greeting round trip): a code-7 townsfolk character is in mode 7. The hero comes within 5.0 in front, so the mode becomes 13. The hero walks away to 12.5, so the mode becomes 0, and on the next tick 7 again.
- TV-0056-23 (wander point): r = 10, home (100, 0, 50): the point's x is one of 95, 96, ..., 104 and its z one of 45, ..., 54. With r = 0.05 the point is a random cell centroid.
- TV-0056-24 (wander pause): chance 50 and range [90, 150]. Roll 49: pause drawn in [90, 150]. Roll 50: no pause.
- TV-0056-25 (patrol order, 4 points): loop mode 0 targets points 1, 2, 3, 0, 1, ... Loop mode 1 targets 1, 2, 3, 2, 1, 0, 1, ...
- TV-0056-26 (pair radius): two eligible villagers 0.5 apart with pair radius 0.0: no pair (0.5 < 0.0 is false). With pair radius 1.0 they pair, and since they are closer than 6.0 they take roles on their first mode-3 tick.
- TV-0056-27 (talk turn): NPC facing (0, 0, 1), direction to its partner (1, 0, 0), weight 0.3: facing (0.39392, 0, 0.91915). The hero with weight 0.1: facing (0.11043, 0, 0.99388).
- TV-0056-28 (SetAI_Behavior block): SetAI_Behavior(code 5, block with r = 20.0, Pmin 60, Pmax 120, chance 50) on a code-0 villager: the mode is 0 at once, and on the next tick 5, wandering with radius 20.
- TV-0056-29 (proximity firing): flag set, candidate at 5.79 with Action: fires. At 5.80 with Action: does not. Flag clear: fires at 15.0 as soon as that character is the candidate.
- TV-0056-30 (script control): a wandering townsfolk character is grabbed by a script (base state 33) with its pause timer at 40. During the script it does not change. After ReleaseEntityControl (base state 1), the pause timer goes on from 40.
- TV-0056-31 (wander pause, body 526): a pause starts with pause timer 300 and fidget timer 100. Ticks 1 to 100 request id 0, and the fidget timer reaches 0. Tick 101 requests id 2; clip 48 keeps playing and does not restart. Later ticks request nothing until clip 48's progress reaches 0.99. On that tick the fidget timer is drawn again and id 0 is requested. The visible clip is 48 on every tick.

Route table, spawn search, blocked flag and movable blocks (the route table vector is in "Route table"; the rest are synthetic):

- TV-0056-32 (spawn search): R = 10, base point B = (100, 0, 50), character at (90, 0, 50), draws 7 and 2: candidate (102, 0, 47). If it is off the mesh, the retries probe (96, 0, 48.5), then (93, 0, 49.25), and so on. Retry 10 gives the home point, not the last candidate.
- TV-0056-33 (spawn draws): R = 10.9 gives n = 10, so the x offset is one of -5.45, -4.45, ..., 3.55.
- TV-0056-34 (cursor quirk): T = 40, cursor -1, three full-scan probes at off-mesh points. Probe 1 scans and misses (cursor 50). Probe 2 is rejected untested (cursor -1). Probe 3 scans again (cursor 0, then 50 on a miss). A point on the mesh at probe 2 is still rejected unless it lies in the cached triangle.
- TV-0056-35 (fresh block): a push in the motion step of tick n sets the flag and timer 120. The AI of tick n + 1 sees timer 120 (fresh block: a walk in the navigation step arrives). The motion step of tick n + 1 makes it 119, so the AI of tick n + 2 sees flag set but no fresh block. With no further push, the AI sees the flag set on ticks n + 1 to n + 120, and it is clear from tick n + 121.
- TV-0056-36 (movable blocks): a block at horizontal distance 9.99 is a candidate; one at 10.0 is not. An owner whose state word is 18 leaves the nearest-block distance at 1.0e9. With no qualifying block it is 31622.78.
- TV-0056-37 (route step; triangle V0 (0, 0, 0), V1 (10, 0, 0), V2 (5, 0, 5), edge 0, radius 0.5 so m = 0.75; k1 = k2 = 7.0711, lo = 0.10607, hi = 0.89393; walker P (5, 0, 2)): goal (1, 0, -10) gives s = 0.43333 and K = (4.3333, 0, 0). Goal (-3, 0, -2) gives s = 0.1, clamped to 0.10607, so K = (1.0607, 0, 0). Goal (6, 0, 10) gives (4.75, 0, 0), which lies behind the walker, so K = (5, 0, 0). Goal (40, 0, -200) is more than 100 away, so K = (5, 0, 0). P (3, 0, 2) with goal (3, 0, -5): dx = 0, s = 0.3, K = (3, 0, 0). Radius 4.0 (m = 4.25): lo = hi = 0.5, so K is always the midpoint.
- TV-0056-38 (tick flow): in a tick spent inside one corridor triangle with an unchanged goal triangle, the facing is blended twice with weight 0.1: first toward last tick's K, then toward the new K. On the tick that enters the goal's triangle it is blended once toward G (0.1). From the next tick it is blended toward G with 0.3. A fresh block arrives only in that settled case.
- TV-0056-39 (mode timer): a townsperson changes mode from 0 to 7 and the draw gives 95. Its mode timer holds 95 and still holds 95 after 500 ticks in mode 7; nothing in its behaviour depends on the value. A mode body that tests the cleared flag always takes the idle-picker branch.
- TV-0056-40 (lock value): a townsperson is pushed off its mesh to (-51, 0, -51) with saved position (-45, 0, 0). Its lock value is 0 (cleared at the end of the previous tick), so it is restored to (-45, 0, 0), blocked flag set, timer 120 (as TV-0056-17). The same setup on the Gnarfas boss on a tick of its leap (lock value 1): no restore, and the saved position becomes (-51, 0, -51).
- TV-0056-41 (home point): a record placed at (10.0, 5.0, 20.0) on a row with Can Fly 0, over ground at height 2.0: the home point and the saved position are (10.0, 2.0, 20.0). The probe starts at height 13.0. On a row with Can Fly 1 the home point is (10.0, 5.0, 20.0). A wander of radius 6 with draws 4 and 1 then aims at (11.0, 2.0, 18.0) when that point is on the mesh.

### Test vectors: a real route (disc)

A complete route over one real navigation mesh: the type-21 mesh 1 of the first level pack (909; kind 1, 9 triangles, near the new-game start). The route table is the mesh's bit block, which starts 28 bytes into the mesh record; with 9 triangles it holds 2 x 81 = 162 bits (21 bytes). Neighbours over edges 0, 1, 2 (-1 = boundary): triangle 0: (6, 1, -1); 1: (0, 2, -1); 2: (-1, 3, 1); 3: (-1, 4, 2); 4: (-1, 5, 3); 5: (4, -1, -1); 6: (7, -1, 0); 7: (6, 8, -1); 8: (-1, -1, 7). The triangles form a chain 8 - 7 - 6 - 0 - 1 - 2 - 3 - 4 - 5.

- Table rows toward triangle 5 (value of table(a, 5) for a = 0 to 8): 1, 1, 1, 1, 1, 0, 2, 0, 2. Rows toward triangle 8: from 5, 4, 3, 2, 1, 0, 6, 7 the values are 0, 2, 2, 2, 0, 0, 0, 1 (the route back runs through the same nine triangles in the opposite order).
- Reading one pair: pair (8, 5) is at bit k = 2 x (8 x 9 + 5) = 154: byte 19 (value 170, binary 10101010), bits 2 and 3, which give 2. Pair (4, 5): k = 82, byte 10, bits 2 and 3 of the value 164, giving 1. Pair (7, 5): k = 136, byte 17, bits 0 and 1 of the value 64, giving 0.
- Route from triangle 8 to triangle 5 (eight edge crossings). The walker has radius 1.175, so m = 1.425. To make each step reproducible the walker is placed at the centroid of the triangle it is leaving, and the goal G is the centroid of triangle 5, (-1859.651, 53.499, 1177.061). All triangles have y = 53.499. Each row gives the table value e, the vertices A = vertex e and B = vertex e + 1 of the edge, the neighbour entered, the crossing parameter s and the steering point K (route step, with the 3D distance to G below 100.0 on every row).

| Step | Triangle c | e = table(c, 5) | Edge A to B (x, z) | Enters | P (x, z) | s (lo to hi) | K (x, z) |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | 8 | 2 | (-1883.086, 1113.304) to (-1888.098, 1140.512) | 7 | (-1888.620, 1127.088) | 0.65070 (0.10659 to 0.91473) | (-1886.347, 1131.008) |
| 2 | 7 | 0 | (-1867.753, 1127.619) to (-1888.098, 1140.512) | 6 | (-1879.645, 1127.145) | 0.45876 (0.06114 to 0.91937) | (-1877.086, 1133.534) |
| 3 | 6 | 2 | (-1851.182, 1131.359) to (-1888.098, 1140.512) | 0 | (-1869.011, 1133.163) | 0.46861 (0.08355 to 0.88153) | (-1868.481, 1135.648) |
| 4 | 0 | 1 | (-1851.182, 1131.359) to (-1877.682, 1159.886) | 1 | (-1872.321, 1143.919) | 0.69348 (0.06686 to 0.96133) | (-1869.559, 1151.142) |
| 5 | 1 | 1 | (-1851.182, 1131.359) to (-1867.737, 1162.732) | 2 | (-1865.534, 1151.326) | 0.79725 (0.15450 to 0.95895) | (-1864.380, 1156.371) |
| 6 | 2 | 1 | (-1842.212, 1137.097) to (-1867.737, 1162.732) | 3 | (-1853.710, 1143.729) | 0.49229 (0.04031 to 0.86572) | (-1854.777, 1149.717) |
| 7 | 3 | 1 | (-1833.809, 1158.351) to (-1867.737, 1162.732) | 4 | (-1847.919, 1152.727) | 0.52872 (0.04297 to 0.93198) | (-1851.747, 1160.667) |
| 8 | 4 | 1 | (-1848.067, 1183.448) to (-1867.737, 1162.732) | 5 | (-1849.871, 1168.177) | 0.43824 (0.05213 to 0.93822) | (-1856.687, 1174.369) |

- Step 1 in full: triangle 8 has vertices (-1888.098, 1140.512), (-1894.675, 1127.447) and (-1883.086, 1113.304); edge 2 runs from vertex 2 to vertex 0, so A = (-1883.086, 1113.304), B = (-1888.098, 1140.512), C = vertex 1, D = B - A = (-5.012, 27.208). The distance of B from the line through C and A is k1 = 13.3685 and the distance of A from the line through C and B is k2 = 16.7116, so lo = min(1.425 / 13.3685, 0.5) = 0.10659 and hi = max(1 - 1.425 / 16.7116, 0.5) = 0.91473. The horizontal line from P toward G meets the edge line at s = 0.65070, inside [lo, hi], so K = A + 0.65070 x D = (-1886.347, 1131.008). The directions from P to G and from P to K agree (dot product 1.0), so the behind test leaves K alone. The walker steers toward K with weight 0.1. The edge's neighbour is triangle 7, and the next tick reads table(7, 5).
- No row is clamped and no row falls back to the midpoint. The table value names the edge the walker leaves by; the walker reaches triangle 5 after the eighth crossing, where C0 = T0 and the settled case of the navigation step applies.

## What NPCs do not do

- Region characters do not collide with the world's wall segments (frame records) or cell list A; only the navmesh confinement, cell list B, the movable-block contact pass and the character push hold them (see "Wall confinement of region characters").
- No NPC or enemy hops or jumps across navigation-mesh gaps. The game has no navmesh jump-link system (SPEC-0018), so a reimplementation must not add one.
- No route is searched at run time. Routes come from the route table stored with each mesh (see "Route table").
- No script walk target is the hero's position. Every walk call in the level scripts takes a locator index or a path index (SPEC-0030), and no townsfolk routine retargets a walk to the hero. The greeting and the talk turn face or wait for the hero, but do not walk toward it.
- Townsfolk have no follower, escort or ally behavior. Mode 9 (FOLLOW) is used on disc: 5 townsfolk placements have code 9, but class 6 has no body for mode 9, so they stand in their current animation. A real follow exists only in the common-enemy class 17 (9 placements; SPEC-0038). The villager pair joins two villagers, never a villager and the hero, and it never forms on disc.
- There is no teleport-if-far. A walk changes the walk target and the navigation state only. The only script call that sets a position is MoveEntity (SPEC-0030).
- In the level modules that call the game-over call, the condition reads the hero's state. None reads an NPC's death (SPEC-0053, section 3.11, says the same of the parked hero).

## NPC build guide (townsfolk, shopkeepers, quest givers, villagers and the creatures that do not fight)

Confidence: High unless marked. This guide gathers, in the order a rewrite needs them, the rules above and the rules of the non-fighting creature sections of SPEC-0029 and SPEC-0047. It adds the contact rules (what the hero's blows, status effects, enemies and pushes do to an NPC), the state tables, the placement counts and four worked examples. The enemy kinds (class 17 and the bosses) are not covered here.

### 1. Kinds and placement counts

| Kind | Class | Row | Hit points | Placed on disc | Moves by |
| --- | --- | --- | --- | --- | --- |
| Townsfolk, shopkeepers, quest givers, guards, officials, Illusen | 6 | companion row 531 (monster row gives none) | not used (hits ignored) | 666 placements, 280 names (123 placements unnamed) | modes 0, 5, 6, 7, 9, 10, 11, 12, 13 below; script walks |
| Whinny | 7 | 253 | 7 | 17 in play plus 4 plausible | SPEC-0029 Behavior 5 |
| Mortog | 39 | 249 | 20 | 21 | SPEC-0029 Behavior 3 |
| Miamouse | 21 | 248 | 5 | 21 (resources 538, 759, 1164, 1281, 1400) | SPEC-0029 Behavior 28 |
| Target Dummy | 36 | 254 | 50 | 7 (all in the castle courtyard pack) | SPEC-0029 Behavior 29 |
| Wild petpets | 33 | 244, 245, 247, 250, 251 | 5, 5, 5, 20, 5 | 8, 8, 5, 7, 5 | SPEC-0047 |
| Second petpet kind | 20 | 252 | 20 | 5 plus 1 plausible | SPEC-0047, Class 20 |
| Queen Fyora | 38 | 132 | 50 (not used) | 4 records, 2 named | SPEC-0029 Behavior 30 (finale and duel only) |

Townsfolk by code (666 placements): code 0 stand 414 (299 named, 115 unnamed), code 5 wander 51, code 6 patrol 121 (87 loop, 34 back and forth), code 7 guard-stand 67, code 9 stand 5, code 10 seated 6, code 12 lying 2. Code 7 holds guards, officials and some villagers: Mayor Jurgin (5), the Cogham villagers Stanley, Rose, Giovanni the Explorer, Ingmar Anvilmaster and Serene, Guardsmen Marrok and Dubric, Tiff, Jeffrey and Dorf in the arena pack, and Illusen in three packs. Code 9 is Ruffie (3, Cogham), Cinthia (1) and Lucy (1, the farm). Code 10 is Lord Cormac, Sir Harlag, Guardsman Henry, Lord Darius and Lady Anabel (the court pack) and Seradar. Code 12 is Arken and Hank. No placement uses code 11 or any code above 12. The clip behind walk id 1 is the 5.00 units/s clip for 378 placements (268 on clip 104, 110 on clip 107) and the 1.71 units/s clip for 214; the other 74 are on the other families. Shopkeepers are ordinary townsfolk placements: the stock is world items bought through the Buy prompt (SPEC-0050), and there is no counter object and no shop-specific NPC behavior.

### 2. One tick of a townsfolk character

Order inside the world tick: the NPC update (re-time of the playing clip, status tick, then the class routine through the arbiter when the base state allows it), the motion-record tick, the animation pass. The class routine does the following in this order every tick it is allowed to run (base state 1, or 17 or 31; frozen in 16, 25, 33, 36, 42, 53 and 55):

1. Update the navigation index; on the very first run of a character with no mesh, search for one (kind 1 mesh containing the position).
2. Pre-step: the greeting entry test (all modes except 10, 11, 12); in mode 0 set mode := code; in modes 5 and 6 the pair search (never matches on disc).
3. Set the movement class from the move type.
4. Run the body of the current mode (table below).
5. Yield.

Before the very first tick the class start makes the character invulnerable and removes it from lock-on; it makes no animation request, so the first clip is whatever the animation player holds after bind (idle id 0 is the default if a rewrite needs one).

### 3. Mode state tables (what each mode does on every tick)

| Mode | How a character gets there | Body, each tick | Timers and numbers | Leaves by |
| --- | --- | --- | --- | --- |
| 0, code 0 | bind; end of a greeting; SetAI_Behavior with a new code | idle id 0; turn toward the placement facing with weight 0.05 | none; the turn never "finishes" | greeting entry; SetAI_Behavior |
| 0, code not 0 | same | none: the pre-step has already set mode := code in this tick | one tick at most | the pre-step |
| 5 wander | code 5 | walk id 1 (move type 0) or the move-type clip; pick a point; walk or pause (Wander) | pause 1 to Pmax ticks; arrival 1.5; chance roll 1 to 100 below the chance | greeting; SetAI_Behavior |
| 6 patrol | code 6 | walk id 8 (the move type's clip when not 0); route from point 1; point pauses; random stops (Patrol) | arrival 0.5; pause ranges from the record | greeting; SetAI_Behavior |
| 7 | code 7 | nothing; keeps its clip and facing | none | greeting (it comes back to 7) |
| 9 | code 9 | nothing; same as 7 | none | greeting |
| 10 | code 10 | request id 65 (seated) | none; no greeting test | SetAI_Behavior only |
| 11 | no placement | request id 97 | none | SetAI_Behavior only |
| 12 | code 12 | request id 98 (lying); apply poison to itself | status timer 7200 re-set each tick | SetAI_Behavior only |
| 13 greeting | the entry test | idle picker for 180 ticks, id 30 once, idle picker; no turn | counter 0 to 180 | hero beyond 12.0, or dot below -0.1 |
| 1, 2, 4, 8, 14 to 23 | not reachable on disc | nothing | none | none |
| 3 pair | not reached on disc | Villager pair | talk timer 300 to 720, cooldown 600 to 1800 | pair end |

Notes. A mode 7 or 9 character is only a statue that greets: it never turns on its own. A character in modes 10 to 12 never greets and never turns. Mode 5 and mode 6 characters walk by root motion at the clip speeds of Walk speed and movement type; there is no run-or-walk choice except SetAI_MovementType. No mode reads the awareness word except the patrol's random-stop roll.

Greeting window. The entry test needs the hero within 6.0 and in the front half. While those hold, the counter restarts every tick, so the id 30 gesture is never reached. The gesture plays only after 180 ticks in which the entry test fails but the exit test has not fired: the hero between 6.0 and 12.0 away in any direction with the dot at least -0.1, or within 6.0 with the dot between -0.1 and 0. The clip for id 30 has no root travel and lasts 144 ticks at R = 1. On exit the mode is 0 and the next tick restores the code's mode.

### 4. Base states, scripts and cutscenes

| Base state | Set by | AI routine | Clip | Facing | Hits |
| --- | --- | --- | --- | --- | --- |
| 1 | bind; ReleaseEntityControl; walk calls; arrival; end of conversation (restores the saved state) | runs | by mode | by mode | ignored |
| 25 | SetConversationTarget | frozen where it yielded (mode, counters and hold counter keep their values) | idle id 0 each tick (the last walk root step still applies once), or no request for a pose | turns toward the hero with weight 0.3 when the hero is its partner (call argument 1) and it is not in a pose | ignored |
| 33 | GrabEntityControl; PlayEntityAnim; MoveEntity with control; end of LoopEntityAnim | frozen | the playing clip keeps its drift until its progress is 0.99, then the default idle | kept | ignored |
| 55 | LoopEntityAnim | frozen | the scripted loop | kept | ignored |
| 16 | a script kill only | none | death clip | kept | ignored |
| 42 | bind slot, release, reset | none; not drawn | none | none | ignored |

Cutscenes use only these states: a scene grabs the actor (33), places it with MoveEntity or walks it (classes 23 to 26, 35), plays clips (33, 55) and releases it (1). The NPC's AI does not run in 25, 33 and 55, so a patrol or wander stops during a talk or a scene and resumes where it stopped (TV-0056-30). A scene that releases a code-0 character leaves it turning back to its placement facing at 0.05 per tick.

### 5. Turning times (steering, 90 and 170 degree turns)

Remaining angle after n ticks, with v = w t + (1 - w) c each tick (the table gives the first tick at which a threshold is passed):

| Weight and use | Start angle | After tick 1 | After tick 10 | Below 10 degrees | Dot 0.99 (8.11 degrees) | Below 1 degree |
| --- | --- | --- | --- | --- | --- | --- |
| 0.3, talk turn | 90 | 66.80 | 2.86 | tick 7 | tick 8 | tick 13 |
| 0.3, talk turn | 170 | 162.66 | 15.24 | tick 12 | tick 12 | tick 18 |
| 0.1, walk and route turns | 90 | 83.66 | 37.36 | tick 23 | tick 25 | tick 45 |
| 0.1, walk and route turns | 170 | 168.76 | 148.40 | tick 44 | tick 46 | tick 66 |
| 0.05, idle turn back to the placement facing | 90 | 86.99 | 61.31 | tick 48 | tick 52 | tick 93 |
| 0.05, idle turn back | 170 | 169.45 | 162.92 | tick 93 | tick 97 | tick 138 |

An exactly opposite direction gets the side nudge of Steering and turn rate. A walker travels along its facing while it turns (root motion along the facing), so a 90-degree turn at walking pace with weight 0.1 sweeps an arc: about 23 ticks of the 5.00 clip, 1.9 units, before the heading is within 10 degrees.

### 6. Contact rules (hero, enemies, hits, status)

- Hero blows. A townsfolk character is not a hit candidate (the query answers no, and the class makes it invulnerable): the swing, quick shots, full-release shots and the Mote walks pass through it with no cue, no flash, no hit-list entry and no cost. Its take-hit would also return at once.
- Status effects. The shared status apply accepts them, but no source reaches townsfolk except the mode 12 self-poison (no damage).
- Pushing. Region characters push themselves out of any overlapping character (centre distance below 20, body cylinders overlap), the hero included; the hero's own push moves the hero because the townsfolk flag cancels the "push the other one" choice. So a hero walking into a townsfolk character is stopped by the pair of pushes without either stopping its routine. Two townsfolk push each other apart the same way, without re-routing.
- Enemies. Enemies take the selected hero as the only target, their weapon cylinders test only the hero's bone capsules, and the class-17 target filter excludes townsfolk. Enemy projectiles and boss effects do not test townsfolk (Medium-High). Townsfolk therefore never react to a fight, never flee and never take damage; the Juppie death sequence spares them.
- Lock-on. A townsfolk character is never on the lock-on list; it is on the talker list (within 20.0 and in the hero's front half, cosine above 0.5) and can be the talk candidate (SPEC-0022).
- Sounds. Townsfolk make none: no footsteps, no barks, no greeting voice (SPEC-0031). The talk emote voice clips belong to the dialogue box (SPEC-0009).
- Head. No head look-at: only the heroes and the Plague Serpent request one (SPEC-0040). A greeting changes the clip only.

### 7. Draw, release and streaming

A bound townsfolk character is drawn whenever its base state is not 42 and its parts pass the frustum test; there is no fade and no distance fade. It is released when the hero is more than 280 units away (squared distance above 78,400, SPEC-0006), and bound again from its placement when the hero comes back within 280: position back at the placement, mode 0, route and fidget state cleared (the NPC reset), patrol index at its start, no memory of an earlier wander point. A walker mid-path that is released therefore starts again at the placement (High for the release rule; Medium-High that a rebound character carries no state).

### 8. Non-fighting creatures: contact summary

| Kind | Hit by the hero? | What a hit does | Notes |
| --- | --- | --- | --- |
| Whinny | yes (base hit-candidate rule) | shared take-hit: first damaging hit doubled, hit points 7 minus the damage, face the attacker once, at zero hit points the death hook (base state 16, loop frozen, corpse 510 ticks) | no undamageable call in the farm pack; a dead Whinny cannot be herded (Open question 35) |
| Mortog | yes | same; hit points 20; its state lives in its own block, so the class-state write of the hit does not disturb it | a Mortog killed while holding leaves the zone count raised |
| Miamouse | yes unless carried (base state 33) | same; hit points 5 | the pack that needs all five does not protect them |
| Target Dummy | yes | the shared take-hit runs, but the courtyard script makes all seven undamageable every tick, so hit points never fall; the dummy faces the hero and its own routine resets hit points and the twitch counter | worked example 4 |
| Wild petpet (class 33) | no (pet type rule) | none | feeding is the only interaction |
| Class 20 creature | no (pet type rule) | none | reacts to the hero's attack state and shots (SPEC-0047) |
| Queen Fyora | no controller reaction | her orbs and deflect follow the duel controller only | no talk, walk or idle behavior of her own |

### 9. Worked examples

Example 1: a Meridell villager talked to mid-walk (pack 1023 path walker at 5.00 units/s, class 35, hero approaching from the front).

1. The villager walks at 0.08337 per tick along its path, facing +z. The hero comes within 6.0 in the villager's front half on tick h. The hold test fires on tick h: counter 0, no steering, idle id 0 requested. The previous tick's walk step still moves the villager once (0.08337), then it stands.
2. While the hero stays within 6.0 in front, the counter restarts every tick. The hero presses Square within 5.8: the talk callback calls SetConversationTarget. Base state 25 on tick h + k: the walk routine is frozen inside its hold branch with its counter kept; the villager requests idle id 0 every tick and turns toward the hero with weight 0.3 (from 20 degrees off, dot 0.99 on tick 3; from 90 degrees, on tick 8).
3. EndConversation restores base state 1 on the end tick. The routine resumes inside the hold branch. The villager now faces the hero, so the dot is near 1: the hold continues (the hero is nearer than 12.0) with the counter continuing from its old value. The gesture id 30 plays only if the counter reaches 180 without a restart (the hero stays 6.0 to 12.0 away).
4. When the hero is more than 12.0 away or goes behind (dot below -0.1), the hold ends on that tick and walking resumes on the next tick toward the same path point. The route and the point index are untouched. The villager first turns from the hero direction to the route direction with weight 0.1 while walking along its facing: for a 90-degree difference the heading is within 10 degrees after 23 ticks, with about 1.9 units of arc.
5. Variant: the hero talks from behind. The villager keeps walking (the hold needs the hero in its front half) until base state 25 starts on tick s. On tick s the walk clip's last step (0.08337) still applies once, then idle. The villager turns toward the hero from 170 degrees with weight 0.3: 162.66 degrees after tick 1, below 10 degrees on tick 12, dot 0.99 on tick 12. After the talk the hold test starts at once (hero within 6.0 and now in front).
6. A 1.71 units/s villager (table variant B) behaves the same; only its step is 0.02850 per tick, and a villager released beyond 280 units starts at its placement again (the cycle zones of pack 1023 deactivate their finished walkers instead).

Example 2: a Whinny fleeing (SPEC-0029 Behavior 5, farm pack). The Whinny at (-1845.754, 1143.807) stands in wander. The hero comes within 5.0: flee, run clip (13.51 units/s, 0.2252 per tick, first step on the tick after the request). Sub-step A plans a point 4.0 directly away from the hero, runs at most 15 ticks or until within 2.5 (about 7 ticks in practice), then plans again; when the point is off its mesh it takes sub-step B (a random far triangle centroid at least 10 from the hero, run at most 25 ticks or until within 2). The entrance test runs at the top of every flee tick: the first tick the Whinny stands in the pen's entrance area (about tick 23 with a hero 4 behind it) it becomes lead, runs to the pen point, and when within 3.0 of the pen point and inside the inner mesh becomes penned, then walks (4.16 units/s) to random points and idles 120 to 400 ticks between them. If the hero's blow lands on it first, the Whinny takes the shared hit (7 hit points, first damaging hit doubled), faces the hero for one steer and goes on running its own routine until its hit points reach 0; then it falls into base state 16 and stays.

Example 3: a Mortog hold (SPEC-0029 Behavior 3, the zone puzzle; there is no race). Zone centre A = (0, 0, 0), L = 100 (a zone of radius 10). Mortog idles 2 to 8 seconds, then wanders to a random point. The hero comes within 5.0 (squared 25): flee. On the first flee tick the zone test runs first: Mortog at (6, 0, 5) has squared distance 61, below 100, so it enters hold with round(60 x 27.3) = 1638 ticks (hold range 25 to 30 seconds in the puzzle pack) and the zone count goes 0 to 1; the same tick the flee step still runs (hero within 15: it moves toward a point 8 away). Hold plays idle clip 0 and counts down. At the end it draws k from 4 to 8, takes the goal k x unit(P - A) (a point within 8 units of the world origin, not beside Mortog), goes to wander and the count falls by one. The puzzle pack widens the range by 5 seconds at each odd rise of the count and, when the count reaches 4, sets the range to 0 and 0: no hold ends and the count stays at 4. A Mortog the hero kills while it holds (20 hit points) leaves the count raised.

Example 4: the Target Dummy taking a hit (courtyard pack). The dummy's counter is 237 and its interval is 650 (drawn from 400 to 800). On tick T the hero's swing resolves in the hero update (after the NPC update of tick T). The swing passes the hit-candidate query and the dummy is added to the swing's hit list. The shared take-hit runs: damage doubled once (the class state word becomes 19), no hit points lost because the script sets the dummy undamageable every tick, the dummy is steered to face the hero with weight 1.0, the pending-hit flag is set. Tick T + 1, the dummy routine: hit branch: animation 3, current hit points set to the maximum (50), counter 1, a new interval is drawn. While animation 3 plays the counter does not move. When it ends, each tick adds 1; the twitch (animation 1) starts on the tick the counter equals the interval, about 650 ticks later. A second hit during animation 3 sets the flag again: counter back to 1, a new interval, and the clip is not restarted (same id requested). The dummy keeps the facing of its last attacker (Medium-High: nothing turns it back).

### 10. Edge cases for implementers

- A townsfolk character with code 7 or 9 never changes clip until a greeting; after a greeting it plays id 0 and stays on id 0 (a seated or lying code-7 character does not exist; poses are codes 10 and 12).
- Greeting and conversation are different things: the greeting never turns the character, the conversation does (weight 0.3, hero as partner only).
- The hold counter and the gesture belong to the walk routine and to mode 13 alike; a conversation freezes both.
- Talk freezes the AI, not the position: the last walk root step still applies once on the first tick of base state 25.
- Wander and patrol pause timers do not run in base states 25, 33 and 55.
- A code-0 character turns back toward its placement facing for as long as it stands in mode 0, also after a script walk that moved it; the turn never ends.
- Clip speed is re-timed every update, so a status that slows or hastens a clip changes a walker's speed at once.
- Release beyond 280 resets everything except the character's record (no state is saved).
- No townsfolk is ever hit, stunned, poisoned for damage, knocked back, lock-on targeted or attacked; do not add reactions.
- Mode 12 characters show poison and take no damage; their poison timer is 7200 ticks and is re-set every tick.
- A hero-versus-townsfolk push moves both; neither stops walking its routine.

### 11. What a port most likely gets wrong (NPC)

1. Letting the hero's swing hit townsfolk (a cue, a flash or a magic cost). They are not hit candidates.
2. Turning townsfolk toward the hero in the greeting. Only the conversation turns them.
3. Treating modes 7 and 9 as guard or follow. They are statues that greet.
4. Playing fidget clips or idle variants. A townsfolk idle is id 0 forever; only Warren's pause shows a different clip.
5. Missing the 180-tick rule: the gesture needs a hero who stays 6 to 12 units away.
6. Stopping the walk for a hero behind the walker, or never ending the hold.
7. Forgetting that a code-0 character turns back to its placement facing at 0.05 per tick after any talk or script walk.
8. Running wander and patrol timers during a talk or scene.
9. Giving townsfolk voices, footsteps or head tracking.
10. Using one walking speed for everyone (1.71 for 214 placements, 5.00 patrol on every table).
11. Making Whinnies, Mortogs and Miamice immune to the hero (they are killable in the original), and the Target Dummy mortal (the courtyard makes it undamageable).
12. Releasing a character by distance with any state kept.

### 12. Open guess points and defaults

Default if unresolved (rewrite choice): the first clip of a code 7 or 9 character before any greeting is idle id 0. The poison visual of a mode 12 character is the ordinary poison status visual of enemies. A dead Whinny, Mortog or Miamouse stays dead for the visit and nothing in the level script is adjusted; the courtyard dummies are undamageable only while the courtyard script runs.

### 13. Timing table (timing pass)

Confidence: High (from the mode tables, the turning table and SPEC-0007). Units: ticks of 1/60 s. A townsfolk routine runs once per tick in the region pass (item 6 of SPEC-0007), before the hero and before the level script; its counters (greeting counter, pause and fidget timers, pair talk and cooldown timers, poison timer of mode 12) are whole ticks lowered once per run. None runs while the character is frozen (base states 16, 25, 33, 36, 42, 53, 55) or while the world is paused, so a talk or a scene stops every NPC timer where it is.

| Item | Timing |
| --- | --- |
| Turn to the hero in a conversation | starts on the tick after the script's conversation call (the script runs after the NPC in the same tick): weight 0.3 per tick, 90 degrees reach a dot of 0.99 after 8 ticks and 1 degree after 13; 170 degrees after 12 and 18 |
| Turn back to the placement facing (code 0) | weight 0.05 per tick: 52 ticks from 90 degrees, 97 from 170 to a dot of 0.99 |
| Walk and route turns | weight 0.1 per tick: 25 ticks from 90 degrees, 46 from 170 |
| Greeting | entry: hero within 6.0 and in the front half; the gesture id 30 needs 180 consecutive ticks of the window state (the entry test restarts the counter every tick it holds); gesture clip 144 ticks; no turn |
| Wander pause | pause and fidget timers drawn together from the record's [shortest, longest] pause in ticks when a point is reached and the roll (1 to 100) is below the chance; the fidget is a no-op on the townsfolk body (the standing idle loop, 132 ticks) |
| Patrol | point pauses and random stops drawn from the record's ranges in ticks |
| Villager pair (not reached on disc) | talk 300 to 720, cooldown 600 to 1800 |
| Lying (mode 12) | its poison timer is set to 7200 every tick |
| Flee or react to hits | none for townsfolk (hits are ignored; mode 1 does nothing); Whinny and Mortog flee per SPEC-0029, the first flee step on the tick after the request |

Worked timeline (a code-0 shopkeeper facing 90 degrees away from the hero; the level script calls the conversation start on tick T): T the NPC has already run (idle, turning back to its placement facing at 0.05); the script writes base state 25 in item 10. T + 1 the NPC's update sees base state 25 and turns at 0.3 (66.8 degrees left). T + 8 its facing is within a dot of 0.99 of the hero. When the conversation ends on tick E (base state restored by the script), the NPC turns back at 0.05 from E + 1 and is within 0.99 of its placement facing 52 ticks later.

What a port most likely gets wrong (timing): turning the NPC on the same tick as the script call; running pause or greeting counters during a talk; counting the greeting from the first tick the hero is near (the entry test holds the counter at 0 while the hero stays within 6.0 in front).

## Open questions

Default if unresolved (rewrite choice): a class 35 walk with an invalid path index steers toward entry 0 of the entry list and never ends, as in Path walk, No route.

Default if unresolved (rewrite choice): a townsfolk character with no mesh stays where it is; steering of the real route is approximated by the route step, so arrival ticks may differ from the straight-line vectors by a few ticks.

1. Resolved: the townsfolk pre-step sets the mode to the code whenever the mode is 0, in the same tick (see "Mode from code"); the mode 8 or 0 setters and the animation-81 wait belong to class 17 (SPEC-0038).
2. Resolved: townsfolk never enter mode 23, because every townsfolk row has remap switch 1; mode 23 is the class-17 wait for animation 81.
3. Resolved: the displacement is the root motion of the locomotion animation (see "Travel by root motion" and "Walk speed and movement type").
4. Which characters can be the hero's candidate for a registered callback (Medium). Answered: the firing rules are in "Proximity callbacks and items" and talk facing is in "Talk facing".
5. Resolved: the hold and exit tests use the hero's horizontal distance and the facing dot product; the wait animation is the idle picker, and leaving the hold resumes the walk (walk step 5).
6. Resolved: walking continues while the distance is 1.5 or more, the character is not in the last route cell within 2.5 of an open edge, and a route was found (see "Wander"). Medium for the two navigation tests.
7. Resolved: the wander and patrol values are in the placement record's AI block and the pair values are monster-row values (see "Placement values" and "Monster row values"); the social gate is set by EnableVillagerSocialize and DisableVillagerSocialize and is 0 after the NPC reset.
8. Resolved: eligibility needs the villager to be playing one of a list of animation ids (see "Villager pair"). There is no busy-state list.
9. Resolved: the mode change timer (60 to 180 ticks) is never read or counted down; the per-mode tick hook is empty. It costs one random draw per mode change and has no other effect (High; "Mode change timer", TV-0056-39).
10. Resolved: SetAI_Behavior copies its data block over the placement record's AI block (+0xBC to +0xD7); the patrol route (+0xB8) lies outside it (see "Placement values"; SPEC-0030).
11. Answered: mode 9 (FOLLOW) has no body in class 6 and is a class-17 behavior; see SPEC-0038. On disc 5 townsfolk placements use code 9.
12. The names of modes 10 to 12, and what status 2 shows on an NPC. Answered: modes 10, 11 and 12 request ids 65, 97 and 98 every tick, mode 12 also applies status 2 to itself, and modes 1, 2, 4, 7, 8, 9 and 14 to 23 have no body in class 6.
13. Answered: class 6 has no body for modes 7 and 8; their behavior belongs to class 17, see SPEC-0038.
14. Resolved: the boot word at data address 0x2DD8CC is 6.0, so the exit distance is 12.0; mode 13 is the greeting pause with no turn (see "Greeting").
15. Answered: the frame rate word is 60.0, written once at start-up; see SPEC-0007. The seconds conversions in this spec hold for that value.
16. Resolved: the 1.0 threshold is a fixed constant of the plain walk-target setter, compared horizontally and strictly. It is skipped when the new point lies inside the stored target triangle, and what it resets is the goal locate's scan position ("Walk target setters"; High).
17. Move type 7 is used once in a level script on a ground character. It names an unset animation slot, so the character should keep its current clip and its travel; this is not confirmed in play.
18. Resolved: table variant B is used by every placement of monster types 16, 18, 20, 22, 24, 27, 28, 32, 35, 41, 43 and 47 (214 of 666 townsfolk placements; Walk speed and movement type). The 28 Meridell cycle walkers split 14 and 14 (SPEC-0045).
19. Resolved: a region character with no navigation-mesh index runs only the timer countdown of the confinement step (see "Region characters with no navigation mesh").
20. The effect of the owner hook run for a block top 1 to 5 above the reference, and the readers of the nearest-block index and distance. Answered: the closing step of the region-character branch is the movable-block contact pass (see "Movable-block contact pass").
21. Resolved: the readers of the blocked flag and their rules are in "Blocked flag"; its only setter is the confinement step (timer 120).
22. Resolved: the lock value is a one-tick motion-record flag, cleared at placement and at the end of every motion-record step, and set only by the Gnarfas boss during its leap at the hero and on large height changes. For townsfolk and enemies the off-mesh restore always runs (High; Layout row "Lock value", TV-0056-40).
23. Resolved: the bind stores the placement position as the home point, ground-snapped for characters that do not fly; a release in guard mode moves it to the current position (High; Layout row "Home point", TV-0056-41).
24. Resolved (High): code 0 is normal play with no pending rule; code 1 holds while the six-pass wait that follows a loaded save or a new game runs (SPEC-0026, flag G); code 2 holds for at most one pass after a game-over request. The greeting can start in any of these three, provided no rule of a higher code (3 to 9, the priority table of SPEC-0026) matches first. So a townsperson greets the hero during the six passes after a load, and during the single pass after a game-over request, as in ordinary play. Test vector: a hero in state 1 within 6.0 units in front of a townsperson, game-state code 1 on the pass after a load: the greeting starts; with code 4 (a dialogue box up) it does not.
25. Resolved: base state 13 is the hero's state 13 (SPEC-0033); every routine that sets it is a hero routine, and the script calls set an entity only to 1, 25 or 33. The greeting gate on the character's own base state therefore never blocks a townsperson in play (Medium-High).
26. Resolved (High): the status apply is one shared routine for every region character and has no townsfolk test, so status 0 on a townsfolk character would set mode 4 (the mode word and the enemy class state are one word). No source applies a status to a townsfolk character on disc except the mode 12 self-poison, because the hero's hit paths skip townsfolk and no script call applies a status. On-disc class-6 placement codes are 0, 5, 6, 7, 9, 10 and 12; the level-script SetAI_Behavior calls use 0, 5, 6 and 8. So mode 4 never occurs on a townsfolk character.
27. Resolved: eligible NPC class coroutines are stepped by the per-character arbiter in the NPC update, before that character's motion-record tick and animation pass. The later global schedulers do not admit these auto-run-disabled coroutines with their zero mask. Steering changes can affect the current motion-record tick; a requested clip starts in the current animation pass and its first root step moves the character next tick (Travel, items 2 and 6; SPEC-0040, Behavior 3; High).
28. Resolved (Medium for the pose label): Warren's clip 1 is a bow; it is a 30-frame one-shot with a 116-tick duration at R = 1. Its root x, y and z are zero on all keys, so Warren does not drift during the fidget. The exact style has not been confirmed in an in-game preview (see "Wander", Warren's exception).
29. Resolved: the variant set-up word turns on the colour override of SPEC-0040, Behavior 26: the two skin colours become RGB (155, 123, 193) and (101, 81, 126) and the other colours keep their placement values. It is set at bind when progress flag 216 is on (after the Gnarfas aftermath, walkthrough step 20, never cleared) and the pack's sound selector passes its story-span test (SPEC-0009 table of selectors). The variant key of SPEC-0022 is that selector word; no run-time code writes it. Test vector: pack 1217 (selector 1) with flags 216 and 83 on and flag 5 off: every character of a block loaded there takes the purple skin (SPEC-0040, Behavior 26); with flag 5 on, or flag 216 off, none does. Packs with selector 0 (58 of the 192) never do (High).
30. The drift of a walking character put in base state 33 (Base states and script control) is not confirmed in play (Medium-High).
31. Resolved: the route step aims at the point where the straight line to the goal crosses the named edge. That point is clamped to keep m = radius + 0.25 from the triangle's other two sides, but never past the midpoint (the 0.5 clamp). It falls back to the midpoint for a goal 100 or more away, or one behind the walker. A boundary edge is aimed at like any other, so the walker presses against it ("Route step"; High for the geometry, Medium for the boundary-edge consequence).
32. Narrowed: counted over directed links in meshes of 2 to 60 triangles (420,890 reachable pairs), the share of pairs on a shortest route is 420,408 by 3D centroid distance, 420,440 by horizontal centroid distance, 415,734 by centroid-to-edge-midpoint-to-centroid distance and 414,430 by step count. 14 of the 27,813 neighbour links are one-way. No tested cost explains every pair. This does not matter to a clean side that reads the table.
33. Resolved: arrival calls the character's set-state method with 1, the same method that sets base state 33 for a held Miamouse (High).
34. Resolved (R-0127): a path is a run of entry records of the pack's entry list, named by the placement's +0xB8 (SPEC-0006, Tagged tables). The path walk starts at point 0, steps one point each time the point is reached (within 0.5 horizontal) with no pause, ends after the last point, and then reads as not walking; the mode 6 patrol is the looping or back-and-forth version with pauses (High).
35. Resolved for the rules (Medium-High; proactive (open close-out)): a Whinny, Mortog or Miamouse can be killed by the hero, and the pen, zone and cage counts do not account for a dead creature; the code has no exemption. How the farm level script behaves when a Whinny is dead is a script-flow question that only play can show (needs a runtime check). Default if unresolved (rewrite choice): reproduce, with the counts unchanged by deaths.
36. Resolved (High): townsfolk are not hit candidates (the query answers no for the townsfolk flag, and the class start sets the invulnerable word and clears lock-on), the shared status apply accepts every status for them, and no on-disc source applies one except the mode 12 self-poison (NPC build guide, Contact rules).
37. Runtime-only (proactive (open close-out); needs a runtime check): the clip a code 7 or 9 townsfolk character shows before its first greeting. The class start and the NPC reset make no clip request, so it depends on the animation player's start clip. Default if unresolved (rewrite choice): idle id 0.
