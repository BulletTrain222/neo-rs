# SPEC-0065: Common enemy build guides, part 2

- Answers: R-0278 (grab gate is the touched flag, not a stun test); proactive (open question sweep, round 5); R-0158; R-0184; proactive (weapon box); proactive (common enemy AI); R-0124; proactive (enemy claim check pass 3); proactive (Crokabek freeze); proactive (timing pass); proactive (enemy reach pass); proactive (reach pass 2); proactive (movement pass); proactive (enemy sweep); proactive (enemy sweep); R-0150 (Giant Slorg against the first sword); R-0149 (player's answer per ranged kind); proactive (open close-out); R-0199 (the Crokabek lunge on a hero standing still); R-0200 (when the Park Shrine Harris load); R-0223 (how a Shrine Harris reaches the stage-4 column); proactive (open question sweep, round 6); proactive (timeline and reward check); proactive (open question sweep, round 10); proactive (open question sweep, round 12)
- Status: Draft
- Disc build: SCUS-97367 v1.00 (same fingerprint as SPEC-0001)
- Confidence: High for the row values, animation tables, clip gate ticks, attack arithmetic and the notice, wake, hidden and alarm rules. Medium-High for the idle states (wander, patrol, sentry), the tick offsets of the sequencer and the grab machine. Medium for weapon contact geometry, hurt-shape geometry and the places a section marks Medium. Each kind states its own confidence.

## Summary

This spec continues SPEC-0064. It holds the build guides of five more batches of shipped class-17 common enemies: Bog and Sewer Creatures, Flyers, Crypt Spirits and Knights, Cogham Ixi Force, and Werelupe Burrows and Library Oddities (with the Altador Coliseum overview). The roster of all 49 kinds and the shared field behaviour that every guide here relies on (start states, idle states, noticing, alarm groups, hidden kinds, socialize, map search, grab machine, attack manager, missing animation ids) are in SPEC-0064; the base machine is SPEC-0038.

## Bog and Sewer Creatures

Four kinds of water-bound creature fill the swamps, sewers, dungeons and vaults: the Swamp Tentacle (monster type 97, variant 0, row 238), the Tentacle (type 96, variant 0, row 237; the Juppie fight uses a weaker Tentacle, type 98, row 239, specified in SPEC-0015), the Slorg (type 90, row 229) and the Giant Slorg (type 91, row 230). All are class-17 common enemies and follow SPEC-0038 exactly (think loop, pre-step, notice check, attack choosers, sequencer, attack manager, take-hit and death hooks). The two Tentacles are stationary ambushers: they hide at home, rise when the hero comes near, grab or smash, and sink again when the hero backs away. The two Slorgs are slow ground creatures that fight at range: they spit a straight bolt, charge in a straight line, and bite only when the hero is point blank. Confidence: High for the row values, clips, attack arithmetic, the hide and retreat cycle and the grab; Medium-High for the idle states; Medium for weapon contact geometry and hurt shapes. All times are ticks (60 per second). "s" is the tick on which the sequencer requests the strike clip (state 1 of the sequencer; one tick after the choice when the hero is already inside the slot's melee range), and every clip of these kinds has speed byte 2 unless stated (0.25 keys per tick at play rate 1.0, so a one-shot clip of F keys passes its 0.99 gate 4(F - 1) + 1 ticks after the request, one tick earlier from F = 25 keys (SPEC-0005: at 25 keys or more the progress term reaches 0.99 one tick before the clip finishes, so a 25-key clip passes on s + 96); a looping clip passes it when its progress first reaches 0.99).

### Shared rules for the four kinds

#### What they have in common

- **Visible kinds.** Every row has Display in default 1. The setter's default case therefore gives state 0 (not 23), the notice pass goes to the normal alerted search (state 2, never 21), and nothing is invisible by the Display rule. The only hiding is the ambush rule of state 8 and of the retreat state 14, below.
- **Row words common to all four.** Class 17, hostile, approach routine 1 (rush), can fly 0 (never airborne), blocking angle and block chances 0 (no block, so nothing here ever plays a block clip), no steal, no socialize flag, no map search, no anger, no ghost, no reanimation, flying hit 0. Carried Neopoints are always 0. Target type T: Tentacle 2 (nature), Swamp Tentacle, Slorg and Giant Slorg 4 (water). An enemy ignores the status of its own element (SPEC-0036), so the Tentacle ignores poison and the three water kinds ignore stun; a Mote whose kind matches T (1 with 4, 5 with 2, and kind 7 with everything) adds the Mote bonus of SPEC-0036.
- **Weights in the attack manager** (SPEC-0038 Behavior 45): Tentacle and Swamp Tentacle 8, Slorg and Giant Slorg 2. With the starting gear (capacity 0) only the first listed attacker may start an attack; every other enemy that would start one waits in line (outer sub-state 12). A waiting enemy walks with id 2 (the walk gait of gait set 2, not the rush id: Tentacle 7.15 u/s, Slorgs 2.90 u/s; the Swamp Tentacle has no id 2, so clip 0 plays, a one-shot with no root travel, and a waiting Swamp Tentacle stands where it is) to a stand-off point around the hero drawn in a square of side R = row +0xB0: 50 for both Tentacles (so a waiting Tentacle may wander up to 25 units from the hero), 10 for both Slorgs; it faces the hero, redraws its point every int[600, 800] ticks, and is released the tick the manager stops holding it.
- **Death.** Death hook, then base state 16 and the death clip once (id 58); the disappear countdown starts when the clip reaches progress 0.95 and the corpse is released when it passes below 0: Swamp Tentacle 180, Tentacle 200, Slorg and Giant Slorg 300. The corpse sinks 1 unit in 180 ticks once the countdown is 30 ticks below its start. The killed enemy never returns (SPEC-0038 Behavior 47). Death clip lengths are in each kind's table.
- **Drops.** The death hook pays the placement's drop ordinal (record offset 0xB0; -1 means none) by SPEC-0038 Behavior 34: -1 on 2 of 51 Swamp Tentacle, 4 of 25 Tentacle, 3 of 26 Slorg and 6 of 14 Giant Slorg placements in play.
- **Hit reactions.** On a non-flyer the take-hit hook starts hit-reaction kind 0 (id 57; kind 5, id 127, exists in no table here). A hit while the enemy is not in agro is doubled and puts it in agro at once (SPEC-0038 Behavior 37). A blocked charge (Slorgs) is kind 3 (ids 62 then 64).

#### Where they stand: default states and counts

Counts are the placements the player can meet (confirmed or plausibly loaded), without the CUT blocks (unreached packs and the test block of regions 1260 to 1309, a few of which hold the same kinds). Default-state word: 5 wander, 6 patrol, 7 guard, 8 hide at home, 0 stand idle.

| Kind | State 0 | State 5 (wander) | State 6 (patrol) | State 7 (guard) | State 8 (hide, emerge) | Main places |
|---|---|---|---|---|---|---|
| Swamp Tentacle (238) | 0 | 0 | 0 | 0 | 51 (all) | Bogshot Swamp 40, Bogshot Village 6, Werelupe Burrows Lower 4 and Upper 1 |
| Tentacle (237) | 0 | 5 | 0 | 0 | 20 | Meridell Sewers 10, Ancient Sewers 8, Secret Sewers 7 (5 wanderers) |
| Tentacle (239, Shadowglen Woods and the Juppie pack) | 0 | 0 | 0 | 0 | 8 | Shadowglen Woods 5, pack 1217 3 |
| Slorg (229) | 0 | 9 | 7 | 2 | 8 | Meridell Sewers 8, Ancient Sewers 6, Black Keep Dungeon 5, Meridell Treasure Vault 4, Rathbone Family Crypts 3 |
| Giant Slorg (230) | 0 | 6 | 6 | 2 | 0 | Secret Sewers 6, Meridell Treasure Vault 3, Black Keep Dungeon 2, Ancient Sewers 1, Bogshot Sinkholes 1, Werelupe Sinkholes 1 |

The Giant Slorg of the Missing Kid errand (module 742, region 744; SPEC-0041) sits in an unreached pack and does not appear. Of the Tentacle 239 placements, the five in Shadowglen Woods are plausible rather than confirmed (their region is listed by a reached module but no load of it was found); they are described under the Tentacle.

Alarm group byte (record offset 0xB7, 255 = none): Swamp Tentacle 35 of 51 have none; Bogshot Swamp group 1 holds 4 (emerge 3 or 4), group 2 holds 8 (emerge 5, 10 or 20) and group 5 holds 4 (emerge 12). Tentacle 237: Meridell Sewers group 2 holds 2 (emerge 1 and 40), group 3 holds 1 (emerge 5); Ancient Sewers group 3 holds 3 (emerge 1, 1, 5). Slorg: the two Treasure Vault state-8 Slorgs share group 1 (emerge 1). Giant Slorg: one Secret Sewers patrol Giant is in group 10. A group number only means something inside its area.

#### Spawn record values that drive the field states

| State | Record values | Meaning |
|---|---|---|
| 5 | float at +0xBC (R); integers at +0xC0 and +0xC4 (pause range, ticks); integer at +0xC8 (pause percent) | Wander square side R, pause chance and range |
| 6 | s16 at +0xB8 (path group); integer at +0xBC (0 loop, nonzero ping-pong); integers +0xC4, +0xC8 (pause range at each point); +0xCC (percent), +0xD0, +0xD4 (mid-path stop range) | Patrol of a locator group |
| 7 | integer at +0xC4 (0 on every guard here) | 0: stand and fidget; nonzero: face the placed direction |
| 8 | float at +0xC0 (emerge distance); integer +0xC4 (cue word); integer +0xC8 (emerge selector, 0 on every placement) | Wake rule of SPEC-0038 Behavior 40; the cue word 0 makes the hidden enemy untouchable (below) |

Values on disc. Tentacle 237 wanderers (Secret Sewers): R 20 with a pause of 60 to 240 at 40 percent (2), R 20 at 20 percent (1), R 30 at 50 percent (1), R 40 with the range but percent 0 (never pauses, 1). Slorg wanderers: R 10 with pause 240 to 600 at 25 percent (2) and 60 to 300 at 25 percent (2), R 20 or 30 with no pause (5). Giant wanderers: R 10 or 15 with 60 to 300 at 25 percent (2), R 20 with 30 to 260 at 70 percent (1), R 30 with 60 to 240 at 50 percent (1), R 20 with none (2). Slorg patrols (7: Meridell Sewers 5, Ancient Sewers 1, Rathbone crypts 1): point pause int[0, 0] on five, int[0, 240] on one and int[60, 240] on one; mid-path stops of chance 25 with 60 to 480, 120 to 480, 60 to 240 or 240 to 1800 ticks on four, chance 15 with 60 to 480 on one, and chance 25 or 50 with a zero range (which never holds it) on two; ping-pong on five and loop on two. Giant patrols (6: Secret Sewers 5, Ancient Sewers 1): point pause int[60, 240] (5) or int[60, 480] (1); the mid-path stop chances (25 to 70) all have a zero range, so they never hold; loop on three, ping-pong on three. Guards: Slorg 2 (Ancient Sewers), Giant 2 (Treasure Vault), all with +0xC4 = 0. Emerge distances in state 8: Swamp Tentacle 3 (2), 4 (2), 5 (14), 6 (2), 10 (6), 12 (12), 16 (6), 18 (5), 20 (2); Tentacle 237 1 (5), 3 (3), 5 (2), 10 (1), 15 (1), 16 (6), 40 (2); Slorg 1 (2), 5 (1), 10 (3), 15 (1), 20 (1). The cue word is 0 on every Swamp Tentacle and every Slorg except one Meridell Sewers Slorg (emerge 15), and on every Tentacle 237 except one Meridell Sewers Tentacle (group 2, emerge 40).

#### Start of life, notice, wake and return (applies to every kind here)

1. **Start.** The think loop enters hidden state 8 directly when the default word is 8, and otherwise runs the setter with 0, which for these visible kinds gives state 0; the pre-step turns state 0 into the default state on the same tick, so default 5, 6 and 7 are entered at once. Default 0 never leaves state 0 (it plays the idle pick) and wakes only by a hit or a group alarm.
2. **Notice check** (states 5, 6, 7 and 2; SPEC-0038 Behavior 3). The awareness countdown reloads with the row's timer and runs once per tick, so the check can pass on one tick per cycle: 32 ticks for all four kinds (timer 30). On that tick roll A int[1, 100] must be at most awareness (100 for the Tentacles and the Giant, 60 for the Slorg); closer than the near edge times F1 of the hero's action state (0.7 standing, 0.75 slow walk, 1.5 walk, 1.75 run, 1.0 otherwise) is noticed with no further roll; farther than the far edge times F1 is never noticed; between them roll B int[1, 100] must be below the interpolated chance (given per kind), halved when the hero is outside the view cone and halved again under the Tattered Cloak. A blocked sight line replaces F1 by F2.
3. **Wake from 5, 6 or 7.** A pass moves the enemy to state 2: it stops, turns to the last-seen point (turn weight = the row's ground turn value, 1.0 here, so it faces the point at once) and plays id 16 (look) when the table has it, otherwise the idle pick. The notice check runs again in the same tick and then once per cycle. A pass while id 18 (alert) is not playing requests id 18 and the enemy enters agro when that clip reaches its gate; the alert then runs to its end even if later checks fail. With no pass, the search countdown (int of float[row +0x60, row +0x5C)) runs down, only while id 18 is not playing; at zero the enemy requests id 17 and, when it finishes, runs the setter with 0 (default state); a kind whose table has no id 17 runs that setter at once (a missing id 16 plays the idle pick, and a missing id 18 would send a notice pass straight to agro; all four tables here have id 18). So noticing is two stages: the first pass puts the enemy in search, a second pass within the search window starts the alert. The second stage has the same probability as the first, and it can try again on every later cycle tick while the search lasts (search countdown plus the id-17 clip). Numbers per kind below.
4. **A hit at any time** (SPEC-0038 Behavior 37): damage doubled once when the state was not agro, state 19, hit reaction kind 0, the attacker is faced. This works from states 0, 2, 5, 6, 7, 8, 14 and 17, but not from a hidden state-8 enemy (below: it cannot be hit).
5. **Group alarm.** When any member of an alarm group enters agro (every time it enters agro), every other member runs its alarm check at its next check tick (every int[15, 45] ticks): members in states 0, 3, 5, 6, 7 and 14 go to state 2 and request id 18 at once, then enter agro when it ends (no second pass needed); members in state 8 play their emerge clip (id 10) and enter agro whatever the hero's distance; members in any other state do nothing. When the cue word is 0 the emerging member also starts the ambush camera and sound cue (Medium).
6. **Hidden state 8** (SPEC-0038 Behavior 40). Sub-state 0: if the row's distance-hide flag is nonzero (both Tentacles) the enemy stands where it is and plays the hide pick (id 9), never walking home; otherwise (both Slorgs) it walks home with the walk id when it is more than 0.35 from home, and stands there playing id 9. It wakes when the hero is closer than the emerge distance times F1 (a standing hero 0.7: distance 12 becomes 8.4, 16 becomes 11.2, 40 becomes 28; a walking hero 1.5; a running hero 1.75); a distance of 0.0 never wakes it. Sub-state 1 (the next tick) requests the emerge clip (id 10) and sets sub-state 2; the enemy enters agro when the emerge clip passes its gate. **Hidden and untouchable.** While the emerge sub-state is 0, the cue word is 0 and the playing animation is the hide pick (id 9), the enemy counts as hidden (SPEC-0029 Behavior 2b; SPEC-0036 Hit detection): it is not drawn, not hittable (hero swings, shots and Mote objects pass through it, so a hit cannot wake or double it), not auto-targetable, and it ignores the navmesh occupancy of others. It is visible and hittable from the first tick of the emerge request, through the whole emerge clip, and again for the whole of every other state except the second half of the retreat (below). With cue word 1 (one Tentacle 237 and one Slorg) it is never hidden: drawn and hittable while it hides, and a hit wakes it with doubled damage.
7. **Return to the default state after losing the hero.** Three exits leave agro for the default state through the setter with 0: the leash (no route to the hero and the walk waypoint 15.0 or more from the hero for more than 180 ticks), the lost-sight search timing out (search counter above 120) and the victory pose after a killing blow. The Tentacles also leave agro by the distance-hide gate (state 14, below). Default 5 resumes wandering from the current spot, 6 resumes the patrol, 7 walks back to the post, 8 walks home (Slorgs) or stays put (Tentacles) and hides again, so the whole wake and emerge sequence repeats.

#### Idle states in detail (default 5, 6 and 7)

Each tick the pre-step runs first (awareness countdown, notice check, then the state body). 
The wander (state 5), patrol (state 6) and guard (state 7) bodies are the common ones in "Shared field behaviour" at the top of this spec; the record values for this batch are in the table above. The requests for 6, 7 and 8 are made without checking the table.

#### Hero actions against the shared model

| Hero action | Result |
|---|---|
| Walks within the emerge distance of a hidden Tentacle or Slorg | After the one-tick test the enemy rises (emerge clip), visible and hittable from its first tick, and fights |
| Stands still near a hidden enemy | The emerge distance is scaled by 0.7 (walking 1.5, running 1.75): standing is the quietest way to pass |
| Swings or shoots at the spot of a hidden enemy | Passes through (cue word 0): no wake, no damage |
| Hits any non-agro enemy that is not hidden | Doubled damage, agro, hit reaction |
| Hits a hidden-cue-1 enemy | Same; it is hittable |
| Wears the Tattered Cloak | Halves the notice chance and blocks notice through a blocked sight line; it does not reveal a hidden Tentacle or Slorg (the cloak only matters to the Display-in-default-0 kinds) |
| Fights one member of an alarm group | Every other member of the group in states 0, 3, 5, 6, 7, 14 (alert) or 8 (emerge) joins within 15 to 45 ticks |

### Swamp Tentacle (row 238)

The Swamp Tentacle is the bog's ambusher: a 75-hit-point eight-bone tentacle that lies hidden at its post in a pit, rises when a hero comes within its emerge distance, and then fights for as long as the hero stays within 10 units. It has three attacks: a quick 15-damage lash, an unblockable 25-damage smash from a little farther away, and a rare grab that holds the hero for about a second and tosses him a short way without hurting him. Every placement in play starts hidden (default state 8) and has no wander, patrol or guard state. It dodges 99 of 100 melee swings, but only one swing at a time.

#### Identity and tunables

| Item | Value |
|---|---|
| Records | Monster type 97, variant 0, row 238, name "Swamp Tentacle". 53 records in the bog regions (Bogshot Swamp, Bogshot Village, Werelupe Burrows Lower and Upper) and the test block; 51 in play (49 confirmed, 2 plausible). One clip table serves every record in play |
| Class and approach | Class 17; approach routine 1 (rush); target type T = 4; Display in default 1; never flies |
| Hit points, body | 75; radius 2.0, height 4.0, weight 5.5; attack-manager weight 8; hostile |
| Hurt shape | Eight chained capsules on bones 0 to 7: lengths 1.57, 1.79, 1.7, 1.67, 1.34, 1.69, 1.13, 1.09 and radii 1.12, 0.81, 0.59, 0.63, 0.87, 1.05, 0.37, 0.28 (model units; Medium). Row radius and height are used only for the chase stop and spacing |
| Weapon volumes | Finite cylinders: weapon 1 radius 5.0 height 5.0; weapon 2 radius 2.0 height 1.5; weapon 3 radius 2.0 height 1.5; weapon 4 radius 1.5 height 3.0; weapon 5 radius 2.0 height 2.0. The lash and smash clips set all five weapon bits together; the grab does not use the weapon test |
| Notice | View cosine 0.0 (a 90 degree half cone), awareness 100, timer 30 (cycle 32 ticks), near range 10 with chance 75, far range 40 with chance 25, no always-has-line-of-sight; chance(d) = 75 - 50 (d - 10) / 30 between the edges. Search countdown int of float[60, 240). Used only in state 2 (see Start and wake) |
| Turn | Aim turn weight in the hold sub-states 1.0 |
| Agro bands | Rush band 3.0 to 12.0 (hold between them); desired distance 3 + 9 f for f from 0.20 to 0.80 (4.8 to 10.2), redrawn after every normal attack choice |
| Distance-hide gate | Flag 1, hide distance 10.0, hide chance 75: in the rush outer sub-state, each tick the hero is farther than 10.0 and int[0, 100] is below 75 (75 of 101), the Tentacle enters the retreat state 14 |
| Recovery after an attack | int[30, 120] ticks |
| Missile defence | Hold 60, dodge chance 75 (74 of 100), block 0 |
| Melee defence | Block 0, 0, 0; dodge 100, 100, 100 for quick, normal and heavy swings (99 of 100 each); the hold is set to 60 when a dodge starts and drawn int[15, 60] when it ends |
| Other | No steal, no socialize, no map search, knockback only on slot 0 (value 2, applied to the hero when the lash does damage), no projectiles, no summons |
| Death | Disappear timer 180; death clip 58 |
| Stand-off radius | 50 (wait in line) |

Attack slots (table slot 3 has type 0 but trigger band 0 to 0 and chance 0, so it never passes; tested order 3, 2, 1, 0). Speeds are the wind-up, strike and recovery words; the wind-up word also scales the request tick of a strike clip whose wind-up clip is missing (every slot here), and the recovery word is never used:

| Slot | Distance band (strict) | Chance | Facing arc (dot above) | Melee range | Delay after the choice, redrawn at the end path | Damage | Knockback | Flags | Speeds (wind-up / strike / recovery) | Counter |
|---|---|---|---|---|---|---|---|---|---|---|
| 0 | 0 to 17 | 25 | 0.0 | 5.0 | int[0, 0] = none | 15 | 2 | blockable | 1.5 / 1.5 / 1.0 | trigger 1 (hero recover window), chance 25 |
| 1 | 7 to 17 | 50 | 0.0 | 7.0 | int[0, 0] = none | 25 | 0 | unblockable (flag 0x1) | 1.0 / 1.5 / 1.0 | trigger 5 (blocked and L1 held), chance 50 |
| 2 | 5 to 17 | 50 | 0.0 | 6.0 | int[1800, 3600] | 0 (the grab deals none) | 0 | unblockable and grab-and-throw (flags 0x1 and 0x8); throw speed 5.0 | 1.5 / 1.5 / 1.0 | none |

Slots 0 and 1 have no cooldown, so a Swamp Tentacle in reach attacks again as soon as its recovery of int[30, 120] ticks has run out. Slot 2 is available once at the start and then only every 30 to 60 seconds. All three slots are melee, none has a charge or a loop, none projects anything, none applies a status.

#### Animation ids and clips (one table for every record in play; 166 entries)

| Id | Use | Clip |
|---|---|---|
| 0 | idle pick | clip 3 (21 keys, loops) |
| 115 | idle fidget | clip 10 (30 keys, loops); the idle picker picks it on one in five of its timer expiries (int[120, 600] ticks) |
| 9 | hide pick | clip 5 (1 key, loops) |
| 10 | emerge | clip 17 (40 keys, one-shot), gate s + 156 |
| 16 | search look | clip 20 (44 keys, one-shot), gate s + 172 |
| 18 | alert | clip 10 (30 keys, loops), gate s + 116 |
| 17, 68 | search end | clip 11 (50 keys, one-shot), gate s + 196 |
| 21, 24 | rush walk (gait 0 and 2) | clip 13 (20 keys, loops, root travel 20.25 units per second = 0.3375 per tick) |
| 22 | rush hold | clip 22 (16 keys, loops) |
| 23, 26, 27 | back off | clip 12 (12 keys, loops, root travel -23.90 units per second = 0.398 per tick backward) |
| 25, 1, 7 | keep-distance hold and unused holds | clip 15 (36 keys, speed byte 1, loops) |
| 125 | walk home (state 14) | clip 19 (20 keys, loops, 20.25 units per second) |
| 34 | slot 0 strike (B) | clip 23 (23 keys, one-shot, all five weapon bits on keys 4 to 7) |
| 39 | slot 1 strike (B) | clip 14 (30 keys, one-shot, all five bits on keys 9 to 14) |
| 44 | slot 2 strike (B, only a stub) | clip 26 (13 keys, one-shot, weapon-2 bit on key 12 only; unused by the grab path) |
| 45 | slot 2 follow-through (C, the grab hold) | clip 27 (43 keys, one-shot, bit 0x2 on keys 0 to 27, release bit 0x20 on key 28, forward 0.55 units per second) |
| 36, 41, 46 | blocked (D) | clip 8 (22 keys, one-shot), gate s + 85 |
| 55 | melee dodge | clip 24 (15 keys, one-shot, backward 11.27 units per second), gate s + 57 |
| 57 | hit reaction kind 0 | clip 2 (25 keys, one-shot), gate s + 96 |
| 58 | death | clip 9 (25 keys, one-shot, backward 4.49 units per second), gate s + 96, progress 0.95 at s + 92 |
| 73 | victory pose | clip 25 (30 keys), gate s + 116 |
| 81 | sink | clip 4 (23 keys, one-shot, forward 0.35), gate s + 89 |
| 82 | rise | clip 1 (25 keys, one-shot, forward 0.33), gate s + 96 |
| 2, 3, 4, 6 | wander and patrol walk and fidgets | none (unused: no Swamp Tentacle wanders) |
| 53, 54 | missile dodge | none: the request plays clip 0 (24 keys, one-shot, no travel), gate s + 93 |
| 33, 35, 37, 38, 40, 42, 43, 47 | wind-ups, follow-throughs of slots 0 and 1, recoveries | none: skipped in the same tick |

#### Start and wake

All 51 placements start in state 8 (hidden at home, hide pick id 9, never drawn or hit while the cue word is 0). The wake test (SPEC-0038 Behavior 40) compares the horizontal distance with the placement's emerge distance (3 to 20) times F1 of the hero's action state; sub-state 1 on the next tick requests the emerge clip 10 (156-tick gate), the Tentacle is visible and hittable from that request, and it enters agro when the gate passes. The first attack choice is possible two ticks after agro (set-up tick, then the first outer-1 tick). Emerge at wake tick w: request w + 1, agro w + 157, set-up w + 158, first choice w + 159.

Group alarm (Bogshot Swamp groups 1, 2 and 5): when any member enters agro, every other member still in state 8 plays the emerge clip within 15 to 45 ticks and enters agro whatever its distance; every time a member enters agro (including after a sink and rise) the broadcast repeats. A member in the sink or home-walk phase of state 14 goes to state 2 (alerted search), requests 18 and enters agro when its gate passes (116 ticks later), whatever the hero's distance.

State 2 can also arise after an alarm during a sink. Its clips: 16 (look, 44 keys, gate 172) plays while the search runs, 18 (alert, gate 116) starts on a notice pass and runs to its end, 17 (search end, gate 196) plays when the search countdown (60 to 239 ticks) runs out and the enemy then returns to the default state (hide at home, then emerge when the hero comes near). A Swamp Tentacle never wanders, patrols or guards, so these are the only idle states it has.

#### Per-tick order (agro)

As SPEC-0038 Behavior 44: recovery countdown, missile defence, melee defence, lost-sight test, normal choice, counter choice, distance-hide gate, outer switch, agro tail (route flag and leash). The outer switch starts at outer 0 (set-up, one tick), then outer 1 (rush). The distance-hide gate comes after the choice in the same tick and only works in outer 1, so a Tentacle farther than 10.0 from the hero attacks first when the chooser passes on that tick and sinks only when it did not (see the vector).

#### State table

| State | Action id and clip | Keys | Speed | Gate and exits |
|---|---|---|---|---|
| Hidden (class 8, sub 0) | 9 -> clip 5 | 1, loops | 1.0 | Wake rule |
| Emerge (sub 1 and 2) | 10 -> clip 17 | 40, one-shot | 1.0 | Gate s + 156, then agro |
| Agro set-up (outer 0) | agro idle pick 1 -> clip 15 (the table maps id 1 to its hold loop) | 36, loops | 1.0 | One tick, then outer 1 inner 1 |
| Rush hold (inner 1) | 22 -> clip 22 | 16, loops | 1.0 | d < 3.0: back off; d > 12.0: approach |
| Approach (inner 0) | 21 -> clip 13 | 20, loops | 20.25 units per second | Ends when d < desired (4.8 to 10.2) |
| Back off (inner 2) | 23 -> clip 12 | 12, loops | 23.9 units per second backward | Toward the point hero - dir x 14.0; ends when d > desired |
| Slot 0 lash (outer 2) | 34 -> clip 23 | 23, one-shot | wind-up 1.5 on the request tick, then 1.5 | Window opens s + 11; a still hero in front is first touched on s + 13 (swing started 3.2 to 4.0 away) or s + 14 (5.0 away); no contact: gate s + 59, end path s + 60; after a hit on s + 13: gate s + 82, end path s + 83 (hit on s + 14: gate s + 81, end path s + 82) |
| Slot 1 smash | 39 -> clip 14 | 30, one-shot | 1.0 on the request tick, then 1.5 | Window opens s + 25; first touch on a still hero s + 26 (5.0 away), s + 27 (6.0) or s + 27 to s + 28 (6.7 to 7.0, the usual start: the band begins at the range and the walk-in overshoots by one step); no contact: gate s + 78, end path s + 79; after a hit on s + 28: gate s + 102, end path s + 103 (s + 26: gate s + 103, end path s + 104) |
| Slot 2 grab | 44 -> clip 26 for one tick, then 45 -> clip 27 | 13 then 43 | 1.5 | Below |
| Blocked lash (state 4) | 36 -> clip 8 | 22 | 1.0 | Lash blocked at s + 11: state 4 at s + 12, gate s + 97, end path s + 98 |
| Melee dodge (outer 5) | 55 -> clip 24 | 15, one-shot | 1.0 | Gate s + 57, then the defence exit; the enemy is untouchable for the whole clip |
| Missile dodge (outer 6) | 53 or 54 -> clip 0 | 24, one-shot | 1.0 | Gate s + 93, then the defence exit |
| Hit reaction kind 0 (outer 10) | 57 -> clip 2 | 25, one-shot | 1.0 | Gate s + 96, then outer 0; every further hit restarts the clip |
| Retreat, phase 0 (state 14) | 81 -> clip 4 | 23, one-shot | 1.0 | Gate s + 89; visible and hittable |
| Retreat, phase 1 | 125 -> clip 19, or the idle pick when already home | 20, loops | 20.25 units per second | Hidden and untouchable; see Movement |
| Retreat, phase 2 | 82 -> clip 1 | 25, one-shot | 1.0 | Gate s + 96; hero within 20.0: agro, else setter 0 (default state 8: hide) |
| Victory pose | 73 -> clip 25 | 30 | 1.0 | Gate s + 116, then the default state |
| Wait in line (outer 12) | 2, which this table lacks: clip 0 (24 keys, one-shot, no root travel), so the Tentacle stays where it is; idle pick while the countdown runs | | | Stand-off points in a 50 x 50 square about the hero; it never reaches one it is not already on, so 73 plays only when the drawn point lies at its feet |
| Dead | 58 -> clip 9 | 25, one-shot | 1.0 | Plays once; disappear countdown 180 from progress 0.95 |

Defence exit: outer 0, base state 1, defence hold int[15, 60].

#### Attack chooser

Each tick before the outer switch, while the hero is attackable, the recovery countdown is below 1 and the outer sub-state is 1, 3, 4 or 11: slot 3 never passes; slot 2 (cooldown below 1, 5 < d < 17, facing cosine above 0, roll int[1, 100] below 50, and either a route to the hero or d < 10.0); slot 1 (7 < d < 17, roll below 50, route or d < 10); slot 0 (0 < d < 17, roll below 25, route or d < 10). The first slot to pass wins and the sequencer starts in state 0; the slot cooldown is drawn at the choice and again at the end path, and the desired distance is redrawn.

| d | Per-tick chance that some slot starts (all three off cooldown) | Which slot |
|---|---|---|
| 0 < d <= 5 | 24 % | slot 0 only |
| 5 < d <= 7 | 61 % | slot 2 (49 %), then slot 0 (12 %) |
| 7 < d < 17 | 80 % | slot 2 (49 %), slot 1 (25 %), slot 0 (6 %) |
| d >= 17 | 0 | |

While slot 2 is on cooldown (after its first use) the chances are slot 1 49 % and slot 0 24 % for 7 < d < 17 (61 % together), 24 % for d <= 7. Because the three arcs are all "dot above 0.0" the Tentacle attacks any hero in the front half plane; it turns to face the hero in the hold sub-states (turn weight 1.0) and in the sequencer only if the slot's orient flag is set (it is not).

#### Movement

Movement table (movement pass; shared rules in "Movement rules for every kind of this spec"; speed byte 2 throughout; play rate 1.0 except the dodge and death rows, which run at 1.0 too):

| Mode | Source | u/t | u/s | Turn | Clip (id: clip, keys, loop) | Exits |
|---|---|---|---|---|---|---|
| Field (default state 8 on all 51) | none | 0 | 0 | none (hides in place: distance-hide flag) | 9: clip 5, 1 key | wake test |
| Rush approach (inner 0, gaits 0 and 2) and sequencer approach (state 0) | root | 0.3376 on every key | 20.25 | steering blend 0.3 / 0.1 | 21: clip 13, 20, loop | d below desired (3 + 9f, 4.8 to 10.2) or within the slot range |
| Rush hold (inner 1) | none | 0 | 0 | aim weight 1.0 (snaps) | 22: clip 22, 16, loop | d below 3.0 or above 12.0 |
| Back off (inner 2, gait 3) | root, backward | 0.398 average; per key 0.171 to 0.764 | 23.90 | faces the hero | 23: clip 12, 12, loop | d above desired; aims at the point 14.0 from the hero |
| Retreat home (state 14 phase 1, hidden) | root | 0.3376 | 20.25 | steering | 125: clip 19, 20, loop | hero within 5.0, or home reached |
| Wait in line (outer 12) | none | 0 | 0 | faces the hero | 2 missing: clip 0, 24-key one-shot, no root | manager releases it |
| Melee dodge (outer 5) | root, backward | 11.27 units over the clip (gate 57 ticks) | 11.27 (clip average) | none | 55: clip 24, 15 | gate |
| Hit reaction kind 0 | none | 0 | 0 | facing snapped to the attacker | 57: clip 2, 25 | gate |
| Death | root | 7.48 backward over the clip, then 0.057 u/t sideways while held | - | none | 58: clip 9, 25 | removal 180 ticks after progress 0.95 |

Worked example (chase). An emerged Swamp Tentacle in agro, hero standing 9.0 away on flat ground in front, desired distance drawn 6.0, no attack roll passing. Agro tick 1: outer 0. Tick 2: rush entry, inner 1 (hold); d 9.0 is inside 3 to 12, so it holds. The approach starts only from a hold when d is above 12.0: from 9.0 it never walks in by itself; it waits for a slot (all three bands include 9) or for the hero to move. Hero steps back to 12.5: the hide gate (d above 10.0, roll int[0, 100] below 75) is tested every tick before the outer switch and sends it home on 75 of 101 draws; only on a tick where that roll fails does inner 0 request 21, and the gate keeps rolling on every later tick while d stays above 10. So the Swamp Tentacle in practice never chases: inside 12 it holds or attacks, beyond 10 it goes home within a tick or two. If the gate kept failing, the approach would cover 12.5 - 6.0 = 6.5 units in 20 ticks (0.3376 x 20 = 6.75), the first step landing one tick after the request, and it would hold after one more step.

What a port most likely gets wrong (movement): letting it walk in the field (its walk ids 2 and 4 have no clip; it never leaves its spot except in agro, the retreat walk and the back-off); approaching at walking speed (the rush and the walk-home both run at 20.25 u/s and the back-off at 23.9); making the back-off a fixed-speed slide (its keys run 0.17 to 0.76 u/t).

Rules in words. The approach runs at 0.3376 per tick (20.25 units per second) while d is above the desired distance; the step is discarded within the sum of the radii (2.0 plus the hero's radius) during sequencer states 0 to 3 (chase stop). For a melee slot with d above the slot's range (slot 0 5.0, slot 1 7.0, slot 2 6.0) the sequencer starts in state 0: gait 0 (id 21) toward the hero until d is within the range, then state 1 on the next tick. Slot 1's band starts at its own range (7 < d), so every smash first closes the last fraction of a unit. The rush hold faces the hero each tick (turn weight 1.0). Back off at 0.398 per tick to the point 14 units from the hero. With no route the waypoint walk and leash of SPEC-0038 Behavior 23 apply.

Retreat (state 14, after the gate fires). Phase 0 plays 81 (89 ticks) on the spot, still visible and hittable. Phase 1 (hidden and untouchable) sets the walk target to the home point and plays the walk-home id 125 (20.25 units per second). When the route test to the hero says no route and the Tentacle is within 3.0 of home, it jumps to the home point and enters state 8 (hidden, id 9) at once, skipping the rise. Otherwise phase 1 ends and phase 2 begins when the hero is within 5.0 (and the Tentacle stands on the mesh), or when the walker has reached home and played the idle pick (animation id 0 current): a Tentacle that never left home therefore moves on within one or two ticks of starting phase 1. Phase 2 plays 82 (96 ticks, visible), then enters agro when the hero is within 20.0, otherwise the default state (state 8: hide at home). Total from the gate to hiding again: about 89 + 2 + 96 = 187 ticks when the hero is far; with the hero between 10 and 20 units away the Tentacle goes sink, rise, agro, sink indefinitely (about 190 ticks per cycle), never retreating to hiding while the hero stays within 20.

#### Slot 2: grab and soft throw (Medium-High)

1. Choice tick c: the sequencer starts in state 0. If d is above 6.0 the Tentacle walks (id 21) toward the hero until d is 6.0 or less; the first state-1 tick is s (s = c + 1 if d was already within 6.0).
2. Tick s: state 1 (wind-up id 43 missing) requests B (id 44, clip 26) and enters state 2.
3. Tick s + 1 (the grab machine's choose step runs from the strike test; the machine starts in its choose state): if the horizontal distance to the hero is below 6.0 and the facing dot toward the hero is above 0.5, the hero's touched flag is clear (no stun test; R-0278) and the straight path to the grab point is open, the hero is put in base state 53 (held; the hero cannot act and plays animation 143 each tick), a rumble of magnitude 0.7 for 1 second starts and repeats every 61 ticks while he is held, and in the same tick the machine requests C (id 45, clip 27), replacing B; the grab point used for the path test is attach point 0 (no point bits on the first frame of clip 26). The held hero is carried: on the seize tick and each hold tick he is placed at the enemy's attach point, minus 1.25 along its second basis row (SPEC-0038 Behavior 15; SPEC-0040 Behavior 8c). Otherwise the machine abandons the grab and goes to its recover state, which would request the recovery clip E (id 47) from s + 2; the table has no id 47, so B stays the current clip, the hero is not touched and the swing finishes as a whiff: B passes its gate at s + 33 and the end path runs at s + 34.
4. Ticks s + 2 to s + 75: the clip plays at the strike word 1.5 (the sequencer stays in state 2, which writes the strike word each tick). Each tick the machine checks that the hero's touched flag is clear and the straight path from his last good position is open; if not, the hold ends (the hero returns to base state 1 at the last good position). The sequencer does not aim during the hold (the slot has no orient flag).
5. Tick s + 76: the release bit (0x20, key 28) is first seen. The hero's velocity is set to normalize(15 x facing + 8 x up) x 5.0 (a gentle toss; compare 15 to 17 for the Thug and the Brute), the machine ends, the strike test reports a hit and takes the hero into the take-hit with damage 0 (no health is lost; no knockback value is applied, the hero enters his landing state 48 with no velocity from the knockback rule, SPEC-0036), and no status is applied. In the next tick the sequencer is in state 3.
6. State 3 plays C to its end at rate 1.0 (the strike word no longer applies): the gate passes at s + 131, state 5 on that tick and, because the slot has the grab flag, no recovery clip: the end path runs at s + 132 (recovery int[30, 120], slot-2 cooldown int[1800, 3600]).
7. A hit on the Tentacle while it holds the hero frees the hero at once (take-hit step 4: grab phase 2 and hero in state 53) and starts the 96-tick reaction.

The hold lasts 75 ticks (about 1.25 seconds) and the whole cycle 132 ticks. The grab deals no damage and, because it starts only from 5 < d < 17 with the strict 6.0 test, a Tentacle that was chosen at 6.0 or more but could not close the last fraction wastes it.

#### Hit and hurt windows

| Attack | Window | Hits | Damage per hit before armour |
|---|---|---|---|
| Slot 0 lash | s + 11 to s + 21 | one, on the first touching tick: s + 13 for a still hero 3.2 to 4.0 away, s + 14 at 5.0 | 15, blockable, knockback 2 |
| Slot 1 smash | s + 25 to s + 40 | one, on the first touching tick: s + 26 at 5.0, s + 27 to s + 28 at 6.0 to 7.0 | 25, unblockable |
| Slot 2 grab | s + 1 (seize) and s + 76 (throw) | the throw counts as a hit | 0 |

The windows hold for a hero inside the strike volume (contact follows the general rule of SPEC-0064, Shared field behaviour: Weapon contact and the hero's hurt shape, with no facing or distance gate; the distances are in the Weapon reach table below, and the volumes are larger than the bodies: weapon 1 has radius 5.0; stand-in for a port without attachment frames: the "at the tick" range of the slot's row). The after-hit rest of each swing runs at R = 1.0 (state 3 does not replace the play rate), so a connecting swing is longer than a whiff. The Tentacle can be struck in every state except a melee or missile dodge (outer 5 and 6), the second half of the retreat (state 14 phase 1), while hidden with cue word 0, and while dead. A hit while the Tentacle swings cancels the swing (hit reaction), except in sequencer state 4 when blocked.

#### Weapon reach

Conventions are those of "Weapon reach tables" in SPEC-0064 (shared field behaviour). Row 238, Medium. The windows above are authoritative for the hit ticks. Every window carries all five weapons together (weapon 1 radius 5.0 height 5.0, weapons 2 and 3 radius 2.0 height 1.5, weapon 4 radius 1.5 height 3.0, weapon 5 radius 2.0 height 2.0), so the table gives the union of the five; the centre column is the union of the five volume centres.

| Slot (phase, clip) | Weapons | Window ticks (level contact) | Centre forward / up / side | Tor ahead, at the tick | Tor ahead, from the start | Tor feet vs origin |
|---|---|---|---|---|---|---|
| 0 (B, clip 23) | w1 to w5 | s+11 to s+21 (s+11 to s+21) | -3.0 to 9.0 / -0.1 to 4.9 / -0.9 to 1.6 | -6.0 to 14.6 | -6.1 to 15.1 | -6.5 to 9.5 |
| 1 (B, clip 14) | w1 to w5 | s+25 to s+40 (s+25 to s+40) | -2.1 to 9.0 / -0.4 to 6.6 / -0.1 to 0.3 | -3.6 to 15.0 | -3.6 to 15.4 | -7.5 to 11.5 |

Both blows reach 14.6 to 15.0 ahead of the Tentacle and 6.0 behind it; the Tentacle's own travel is about 0.5. Slot 2 (grab) has no weapon test.

#### Defence and counters

- Melee defence: once per hero swing, on the first tick the swing is active, only while the defence hold is below 1, the Tentacle is not in a hit reaction, not mid-sequence except sequencer states 0, 1 and 5, the hero is within 8.0 and inside 60 degrees in front. The block roll never passes. The dodge roll passes below 100 for every swing class (99 of 100): the dodge sets the hold to 60 and plays 55, an unhittable 57-tick hop of about 11 units backward away from the hero. After the exit the hold is int[15, 60], so the next swing is safe only if it starts when the hold is already below 1. A hero who swings in a rhythm of one swing per 70 ticks or slower is dodged on almost every swing.
- Missile defence: for each live hero shot whose path within 60 ticks meets the body, int[1, 100] below 75 gives a dodge that plays clip 0 for 93 ticks (the table has no 53 or 54).
- No guard: every swing that reaches the capsule hits.
- Hit reaction: kind 0 (id 57, 96 ticks); the reaction restarts on each further hit, so a combo can keep the Tentacle locked (it cannot attack while outer 10 runs unless a counter fires).
- Counters (SPEC-0038 Behavior 14, every tick of a reaction or dodge): slot 0 (trigger 1) is eligible on any tick on which the hero's recover window is open (Tor's swings carry it; Roberta's never do), with 0 < d < 17 and the hero in front: a roll int[0, 100] below 25 (25 of 101) starts the lash. Slot 1 (trigger 5) needs the sequencer in its blocked state with L1 held and 7 < d < 17; a blocked lash happens at close range, so this counter is practically unreachable.

#### Hero actions against this enemy

| Hero action | What the Swamp Tentacle does |
|---|---|
| Stands outside the emerge distance times F1 | Nothing; hidden and invisible |
| Stands still 8 units away | Rises (156 ticks), then grabs (slot 2) or smashes |
| Walks or runs near it | The wake distance is 1.5 or 1.75 times larger; the Tentacle rises sooner |
| Moves beyond 10 units after it has risen | Sinks (75 percent a tick, but not before the chooser has failed on that tick), walks home, rises again if the hero is within 20 |
| Quick or normal swing in front within 8 | Dodged 99 of 100 if the hold is below 1 |
| Heavy swing | Dodged 99 of 100 as well (the heavy chance is 100 too); never when the hold is positive |
| Tor's power step | Never dodged (it has no swing class) |
| Roberta's shots | Dodged 74 of 100 when the path meets the body within 60 ticks; the dodge clip is 93 ticks |
| Blocks with L1 while facing it | Blocks the 15-damage lash; the smash and the grab ignore the shield |
| A landed hit | Damage by SPEC-0036 (doubled if it was not in agro), 96-tick reaction, slot 0 counter on Tor |
| Hero stunned, knocked down or dead | No new choice; cooldowns stop; a running swing continues |

#### Edge cases for implementers

- The first outer-1 tick after the emerge decides between attacking and sinking when the hero is more than 10 units away: the chooser runs first and wins about 80 percent of the time at 7 < d < 17.
- A hero who dies to a lash puts the Tentacle in the victory pose (73, 116 ticks); it then runs the setter with 0 and returns to hiding.
- The hold word is set at the start of a defence reaction and replaced at its end; it also gates the missile dodge. Both dodges draw the same hold.
- The wind-up speed word applies on the request tick of a strike whose wind-up clip is missing: slot 1's first tick runs at 1.0, the rest at 1.5. That is why slot 1's window opens at s + 25, not s + 24 (the first touch on a still hero is one to three ticks later, Hit and hurt windows).
- The 1800 to 3600 tick cooldown of the grab starts at the choice and restarts at the end path; a fight rarely sees a second grab.
- Slot 2 needs d < 6.0 at the seize tick. A Tentacle that starts walking at d = 10 can end at d = 6.0 exactly and whiff.
- The retreat state does not run the attack manager entry; a Tentacle in state 14 is not in the manager's list (released the next frame) and does not take an attack turn.
- Hidden Tentacles in the same alarm group wake together, so a hero passing a group of eight Bogshot Swamp Tentacles is met by all of them within 45 ticks, yet only one of them may start an attack at a time with the starting gear; the others wait in line where they rose (a waiting Swamp Tentacle has no walk clip, so it does not move to its stand-off point).

#### What a port most likely gets wrong

1. Drawing or hit-testing the hidden Tentacle. With cue word 0 it is invisible and untouchable until the emerge request, and again while it walks home underwater.
2. Treating the wake distance as the fight distance. Emerge is up to 20 units (28 for a running hero) but the Tentacle only stays up while the hero is within 10 units.
3. Giving the grab damage or a hard throw: it does none (throw speed 5.0) and ends at s + 132.
4. Using wind-up and recovery clips. None exist; the strike clip starts on the request tick and the end path comes the tick after its gate.
5. Cooldowns: slots 0 and 1 have none, so the Tentacle swings again after int[30, 120] ticks.
6. The 99 percent dodge as a flat rate: it is once per hold, and the hold is only 15 to 60 ticks after a dodge.
7. Making it chase. It sinks and returns home, and keeps re-rising only while the hero is within 20.
8. Forgetting that the alarm group repeats on each re-entry to agro.

#### Timeline test vector

Default 8, emerge distance 12, cue word 0, no group; the hero stands still facing it 8.0 units away from tick 0 (8.0 < 12 x 0.7 = 8.4), guard off. Assume the walk is clear and a route exists.

| Tick | Event |
|---|---|
| 0 | Wake test passes (sub-state 1). Until now the Tentacle was hidden and the hero's swings passed through it |
| 1 | Emerge animation 10 (clip 17) requested (gate 156); visible and hittable from now |
| 157 | Gate passes: agro. Tick 158: set-up (outer 0). Tick 159: first choice |
| 159 | d = 8.0: slot 2 49 percent, else slot 1 25, slot 0 6, nothing 20. (With nothing chosen the hide gate cannot fire because d = 8 <= 10.) Call the choice tick c |
| c | Grab chosen (49 percent a tick): d = 8 > 6, so the Tentacle walks 0.3375 per tick for 6 ticks (2.0 units); s = c + 7 |
| s + 1 | Hero held (base state 53) if the Tentacle faces him within 60 degrees and d < 6.0 |
| s + 76 | Release bit: toss at 5.0, take-hit 0, hero in state 48 |
| s + 132 | End path: recovery int[30, 120]; slot-2 cooldown int[1800, 3600] |
| s + 133 + R to about s + 330 | Next choice with the hero at about 6 to 9 units: slot 1 (if d > 7) 49 percent a tick, slot 0 24 percent. Slot 0 (walks in to 5.0 first): hit at s' + 14 for 15, end path s' + 82. Slot 1 (walks in to 7.0): hit at s' + 27 to s' + 28 for 25, end path s' + 103 to s' + 104 |
| 600 | A motionless hero with no armour has taken the grab (0 damage) and 1 to 3 swings (15 or 25 each): about 25 to 55 damage in total, depending on when the choices fall. A hero who walks away to 11 units: on each outer-1 tick the chooser fails with probability 20 percent and the gate then fires with 74 percent, so the Tentacle sinks on about 15 percent of those ticks and attacks on the rest |

Hero at 11 units from the start (emerge 16, standing hero 11.2): wake, rise (156 ticks), first outer-1 tick 159: attack chosen 80 percent (slot 2 grab approach from 11 to 6), sink 15 percent (89 ticks, then phase 1, then 82 for 96 ticks, then agro again because the hero is within 20, and so on).

#### Open questions

1. Weapon contact geometry. The volumes hang from the clip's attach points; the sequencer only starts a swing at d at most the slot range. Resolved by the general rule (SPEC-0064, Shared field behaviour: Weapon contact and the hero's hurt shape); the distances are in the Weapon reach table; stand-in only for a port without attachment frames: a swing hits a level hero inside the "at the tick" range of the slot's row (6.0 behind to 14.6 ahead for slot 0, 3.6 behind to 15.0 ahead for slot 1).
2. Whether the grab's choose step can run later when the first tick fails. Resolved (Medium-High; proactive (open close-out)): no; the choose step runs once, on the first tick of the machine, and a failure sends the machine to its recover state, so a failed first tick wastes the attack (the swing ends at s + 34).
3. The tick on which the home-walk of state 14 sees animation 0 current. Resolved (Medium-High; proactive (open close-out)): the tick after the walker reaches home. A request made on tick t is started by that tick's animation pass, which follows the enemy's update, so the test sees the clip current from tick t + 1 (SPEC-0038 Behavior 50).
4. Whether the rise (82) is skipped when a Tentacle returns to state 8 because no route to the hero exists. Resolved (High; proactive (open close-out)): yes; the Tentacle jumps to its home point and enters state 8 at once, as the Retreat rule says.
5. The ambush camera and sound cue of an alarm emerge (cue word 0). Resolved (High; proactive (open close-out)): the cue is the shared stinger 4 only (SPEC-0009, Stinger and emote-clip playback), played at volume factor 1.0 and not again within 40 s of its last start; it starts when the group alarm wakes a state-8 member whose spawn record has cue word 0, and no camera is started. A wake by distance never starts it.

### Tentacle (row 237, and the field placements of row 239)

The sewer Tentacle is the same body and the same skeleton as the Juppie fight's minion (row 239, SPEC-0015) with a tougher row: 60 hit points instead of 18, smashes of 20 and 45 damage instead of 10, long cooldowns on the strong swings, a 99 percent melee dodge, and the search clips that the minion's table lacks. It lies hidden at home in the sewer channels (20 of its 25 placements), rises when the hero comes within its emerge distance, fights only while the hero is within 8 units, and otherwise sinks, walks home and hides. Five Secret Sewers Tentacles wander instead of hiding. This guide gives the row-237 numbers in full and a difference table for row 239 (which also has eight field placements: five in Shadowglen Woods and the three Juppie-fight bodies).

#### Identity and tunables

| Item | Row 237 (sewers) | Row 239 (Juppie pack and Shadowglen Woods) |
|---|---|---|
| Records | Monster type 96, variant 0, name "Tentacle"; 35 records (25 in play: Meridell Sewers 10, Ancient Sewers 8, Secret Sewers 7); one clip table | Type 98, variant 0; 10 records (8 in play); one clip table (SPEC-0015) |
| Class, approach | 17, rush; T = 2 (nature: ignores poison); Display in default 1 | same, T = 2 |
| Hit points | 60 | 18 |
| Body | Radius 2.0, height 3.0, weight 5.5; manager weight 8; hostile | same |
| Hurt shape | Seven chained capsules on bones 1 to 7: lengths 1.0, 0.98, 0.88, 0.8, 0.65, 0.57, 0.44; radii 0.33, 0.30, 0.27, 0.21, 0.18, 0.13, 0.10 (Medium) | same |
| Weapon volumes | Finite cylinders of height 1.5: weapons 1, 2, 3 and 5 radius 2.0, weapon 4 radius 1.5 | same |
| Notice | View cosine 0.0, awareness 100, timer 30 (cycle 32 ticks), near range 5 (chance 90), far range 40 (chance 25), chance(d) = 90 - 65 (d - 5) / 35; search countdown int of float[15, 60) | same, except the table has no search clips |
| Agro bands | Rush band 3.0 to 15.0, desired distance 3 + 12 f for f from 0.20 to 0.80 (5.4 to 12.6) | same |
| Distance-hide gate | Flag 1, hide distance 8.0, chance 100 (100 of 101 per tick in outer 1 once past the chooser) | same |
| Recovery after an attack | int[0, 0] = none | same |
| Missile defence | Hold 120, dodge 100 (99 of 100), block 0 | hold 60, dodge 100 |
| Melee defence | Block 0; dodge 100, 100, 100 (99 of 100 for every swing class); hold set to 120 when a dodge starts, drawn int[0, 120] when it ends | dodge 20, 20, 10 (19, 19, 9 of 100); hold 60; timer int[0, 120] |
| Death | Disappear 200; clip 58 | same |
| Stand-off radius | 50 | 50 |

Attack slots (slot 3 unused; tested 2, 1, 0; all speed words 2.0, so every clip of a swing runs at 0.5 keys per tick until the hit and the first request tick runs at 2.0 as well):

| Slot | Band (strict) | Chance | Arc (dot above) | Melee range | Cooldown int[...] (choice and end path) | Damage | Knockback | Counter | Row 239 differences |
|---|---|---|---|---|---|---|---|---|---|
| 0 | 3 to 17 | 50 | 0.9239 | 5.0 | 5 to 45 | 10 | 0 | none | same |
| 1 | 3 to 10 | 50 | 0.0 | 4.0 | 300 to 420 | 20 | 3 | trigger 6 (hit reaction), chance 5 | cooldown 0 to 45, damage 10, counter chance 10 |
| 2 | 3 to 13 | 25 | 0.0 | 6.0 | 15 to 60 | 45 | 12 | none | cooldown 0 to 60, damage 10, knockback 15 |

No slot has a status, a charge, a grab or a projectile. Flags are 0 (blockable) on all three.

Animation ids and clips (row 237 table; 166 entries). The clips are the same 30 as the Swamp Tentacle's skeleton family but mapped differently:

| Id | Use | Clip |
|---|---|---|
| 0 | idle pick | clip 3 (21 keys, loops) |
| 9 | hide pick | clip 5 (1 key, loops) |
| 10 | emerge | clip 17 (40 keys, one-shot), gate s + 156 |
| 2, 4 | wander and patrol walk (state 5, 6, 7) | clip 16 (21 keys, loops, 7.15 units per second = 0.119 per tick) |
| 3, 8, 18 | rest fidget, guard look, alert | clip 22 (16 keys, loops); the alert gate passes at s + 63 |
| 6, 13 | patrol fidget | clip 10 (30 keys, loops) |
| 14, 16 | chat, search look | clip 20 (44 keys, one-shot), gate s + 172 |
| 17, 68 | search end | clip 11 (50 keys, one-shot), gate s + 196 |
| 21, 24, 27 | rush walk | clip 18 (21 keys, loops, 7.14 units per second = 0.119 per tick) |
| 22, 25, 1, 7, 63 | rush hold and holds | clip 15 (36 keys, speed byte 1, loops) |
| 23, 26 | back off | clip 12 (12 keys, loops, 23.9 units per second backward = 0.398 per tick) |
| 125 | walk home | clip 19 (20 keys, loops, 20.25 units per second) |
| 34 / 39 / 44 | slot 0 / 1 / 2 strike (B) | clip 14 (30 keys, bits on keys 9 to 14) / clip 6 (18 keys, keys 6 to 8) / clip 23 (23 keys, keys 4 to 7); all five weapon bits set together |
| 36, 41 | blocked (D) of slots 0 and 1 | clip 8 (22 keys), gate s + 85; slot 2 has none (46 missing) |
| 53, 54, 81 | missile dodge, sink | clip 4 (23 keys, one-shot), gate s + 89 |
| 55, 62 | melee dodge | clip 24 (15 keys, one-shot, backward 11.27 units per second), gate s + 57 |
| 57 | hit reaction kind 0 | clip 2 (25 keys), gate s + 96 |
| 58 | death | clip 9 (25 keys), gate s + 96, progress 0.95 at s + 92 |
| 69 | confused | clip 28 (40 keys, loops, 5.87 units per second) |
| 73 | victory pose | clip 25 (30 keys), gate s + 116 |
| 82 | rise | clip 1 (25 keys), gate s + 96 |
| Missing | 33, 35, 37, 38, 40, 42, 43, 45, 46, 47 (all wind-ups, follow-throughs, recoveries, slot-2 blocked), 56, 60, 61, 64, 115 | skipped (attack phases) or clip 0 |

The row-239 table is the same except that it has no ids 16, 17 and 18 (so its notice pass goes straight to agro: SPEC-0015) and it has a fidget 115 (clip 10).

#### Start and wake

Row 237 default states: 8 (hidden) on 20, 5 (wander) on 5. Emerge distances in state 8: 1 (5), 3 (3), 5 (2), 10 (1), 15 (1), 16 (6, all in Meridell Sewers), 40 (2). Cue word 0 on all but one (Meridell Sewers, group 2, emerge 40: never hidden, hittable and drawn while it waits).

- **Hidden (state 8).** Stands where it is playing id 9 (the distance-hide flag keeps it from walking home), not drawn, not hittable. Wake test with F1 as in the shared rules; emerge clip 10 (156 ticks); agro at w + 157; first choice w + 159. Most sewer placements are close-range ambushes (emerge 1 to 5: they rise only when the hero stands within 0.7 to 3.5 units) or long-range ones (16: 11.2 standing, 24 walking, 28 running).
- **Wander (state 5, Secret Sewers).** Visible and hittable while it wanders (Display 1). Walks the wander rules of the shared section (walk id 2 at 0.119 per tick, fidget 3). The notice check runs every cycle tick (32 ticks): probability per cycle for a standing hero at distance d is 1 inside 3.5 (5 x 0.7) and chance(d) = 90 - 65 (d - 5) / 35 between 3.5 and 28 (before the cone halving, which does not apply to a cone of cosine 0.0 except for a hero behind it). A pass goes to state 2.
- **Search (state 2).** Look clip 16 (172 ticks) starts, the notice check repeats in the same tick and once per 32 ticks. A pass requests the alert 18 (clip 22, 16 keys, loops; gate 63 ticks) and the Tentacle enters agro 63 ticks later whatever the later checks say. With no pass the countdown int[15, 59] runs; at zero the search-end clip 17 (196 ticks) plays and the Tentacle returns to its default state when it ends; a pass during 17 still starts 18. So the search window is about 15 to 59 plus 196 ticks and holds about 7 awareness cycles. The first pass is therefore not the end: a hero who steps back out of the notice band during the look is lost.
- **Return to the default state** after losing the hero: the setter with 0 gives state 0 and then the default: 8 hides in place, 5 resumes wandering. The retreat of state 14 (below) is the normal way a Tentacle gives up.

#### Per-tick order (agro)

As the Swamp Tentacle and SPEC-0038 Behavior 44; the outer switch is rush (outer 1). Differences from the Swamp: the hide gate is at 8.0 with chance 100, no slot has a status or a grab, and the recovery is 0, so the next choice is possible on the tick after an end path.

#### State table

| State | Action id and clip | Keys | Speed | Gate and exits |
|---|---|---|---|---|
| Hidden (class 8, sub 0) | 9 -> clip 5 | 1, loops | 1.0 | Wake rule |
| Emerge | 10 -> clip 17 | 40 | 1.0 | Gate s + 156, then agro |
| Search (state 2) | 16 -> clip 20, 18 -> clip 22, 17 -> clip 11 | 44 / 16 / 50 | 1.0 | 18: gate s + 63 then agro; 17: gate s + 196 then the default state |
| Agro set-up (outer 0) | agro idle pick 1 -> clip 15 (the table maps id 1 to its hold loop) | 36, loops | 1.0 | One tick, then outer 1 |
| Rush approach (inner 0) | 21 -> clip 18 | 21, loops | 7.14 units per second | Until d < desired (5.4 to 12.6) |
| Rush hold (inner 1) | 22 -> clip 15 | 36, loops | 0.5 keys per tick | d < 3.0: back off; d > 15.0: approach |
| Back off (inner 2) | 23 -> clip 12 | 12, loops | 23.9 units per second backward | Toward the point hero - dir x 17.0; until d > desired |
| Slot 2 smash (outer 2) | 44 -> clip 23 | 23 | 2.0 | Window opens s + 8; first touch on a still hero s + 10 (3.2 away), s + 11 (4.0 to 5.0) or s + 12 (6.0, the range) (45); whiff: gate s + 45, end path s + 46; after a hit on s + 10, 11 or 12: gate s + 78, 77 or 76, end path one tick later; blocked on the touch tick t: state 4 at t + 1, no D, end path t + 2 |
| Slot 1 | 39 -> clip 6 | 18 | 2.0 | Window opens s + 12; first touch s + 12 (3.2 away) or s + 13 (4.0, the range) (20); whiff: gate s + 35, end path s + 36; after a hit on s + 12 or 13: gate s + 56 or 55, end path one tick later; blocked on t: D (clip 8) requested t + 1, gate t + 86, end path t + 87 |
| Slot 0 | 34 -> clip 14 | 30 | 2.0 | Window opens s + 18; first touch s + 20 (3.2 to 4.0 away) or s + 21 (5.0, the range) (10); whiff: gate s + 59, end path s + 60; after a hit on s + 20 or 21: gate s + 95 or 94, end path one tick later; blocked on t: D requested t + 1, gate t + 86, end path t + 87 |
| Melee dodge (outer 5) | 55 -> clip 24 | 15 | 1.0 | Gate s + 57, then the defence exit (hold int[0, 120]) |
| Missile dodge (outer 6) | 53 or 54 -> clip 4 | 23 | 1.0 | Gate s + 89, then the defence exit |
| Hit reaction kind 0 (outer 10) | 57 -> clip 2 | 25 | 1.0 | Gate s + 96, then outer 0 |
| Retreat (state 14) | 81 -> clip 4, then 125 -> clip 19, then 82 -> clip 1 | 23 / 20 / 25 | 1.0 | Phase 0 89 ticks (visible); phase 1 hidden, walk home at 20.25 units per second (or one tick if already home); phase 2 96 ticks (visible); then agro if the hero is within 20.0, else the default state |
| Dead | 58 -> clip 9 | 25 | 1.0 | Plays once; disappear 200 from progress 0.95 |

#### Attack chooser

Slots tested 2, 1, 0 each tick under the usual gates (hero attackable, recovery 0, outer 1, 3, 4 or 11). Slot 2: cooldown below 1, 3 < d < 13, facing cosine above 0.0, roll int[1, 100] below 25 (24 of 100), and a route or d < 10.0 for the charge-less rule. Slot 1: 3 < d < 10, roll below 50 (49 of 100), cosine above 0.0. Slot 0: 3 < d < 17, roll below 50, cosine above 0.9239 (22.5 degrees). With all three off cooldown the chance that some slot starts in a tick is:

| d | Start chance | Mix |
|---|---|---|
| 3 < d < 10 | 80 % | slot 2 24, slot 1 37, slot 0 19 |
| 10 <= d < 13 | 61 % | slot 2 24, slot 0 37 |
| 13 <= d < 17 | 49 % | slot 0 |
| d >= 17 or d <= 3 | 0 | |

The cooldowns are drawn at the choice and again at the end path: slot 2 15 to 60, slot 0 5 to 45, slot 1 300 to 420. So in a long fight slot 1 (20 damage, knockback 3) is rare, slot 2 (45 damage, knockback 12) comes back after a swing plus 15 to 60 ticks of cooldown, and slot 0 (10 damage) fills the gaps. The desired distance is redrawn after every normal choice.

#### Movement

Movement table (movement pass; speed byte 2 throughout):

| Mode | Source | u/t | u/s | Turn | Clip (id: clip, keys, loop) | Exits |
|---|---|---|---|---|---|---|
| Wander (state 5, five Secret Sewers records), wait in line | root | 0.1192 on every key | 7.15 | steering blend 0.3 / 0.1 | 2 (and 4): clip 16, 21, loop | wander goal within 0.5; pause rolls |
| Hidden (state 8) | none | 0 | 0 | none (hides in place) | 9: clip 5 | wake |
| Rush approach (inner 0) and sequencer approach | root | 0.1189 average; per key 0.080 to 0.176 | 7.14 | steering | 21: clip 18, 21, loop | d below desired (3 + 12f, 5.4 to 12.6) or within the slot range |
| Rush hold (inner 1) | none | 0 | 0 | aim 1.0 | 22: clip 15, 36 keys (speed byte 1), loop | d below 3.0 or above 15.0 |
| Back off (inner 2) | root, backward | 0.398 average; per key 0.171 to 0.764 | 23.90 | faces the hero | 23: clip 12, 12, loop | d above desired; point 17.0 from the hero |
| Retreat home (state 14 phase 1, hidden) | root | 0.3376 | 20.25 | steering | 125: clip 19, 20, loop | hero within 5.0 or home reached |
| Confused (outer 11) | root | 0.098 average | 5.87 | random-direction aim 0.05 | 69: clip 28, 40, loop | status ends |
| Melee dodge (55) and blocked charge (62) | root, backward | 11.27 units over 57 ticks | - | none | clip 24, 15 | gate |
| Hit reaction kind 0 | none | 0 | 0 | snapped to the attacker | 57: clip 2 | gate |
| Death | root | 7.48 backward over the clip, then 0.057 u/t sideways while held | - | none | 58: clip 9, 25 | removal 200 ticks after progress 0.95 |

Worked example. Hero standing 7.0 in front of an emerged Tentacle, desired distance drawn 5.4, no attack passing: inside the 3 to 15 hold band it does not walk in. With the hero at 16.0 the rush would approach at 0.119 u/t, but the hide gate (d above 8.0, roll below 100: 100 of 101 draws) sends it home on the first such tick, so the Tentacle's own approach covers at most the band 8.0 down to the desired distance (2.6 units at most, about 22 ticks). Its fast moves are the back-off (23.9 u/s) and the hidden walk home (20.25 u/s).

What a port most likely gets wrong (movement): one speed for all its gaits (approach 7.14, back-off 23.9, walk home 20.25); a smooth approach (the rush clip's keys run 0.080 to 0.176 u/t); chasing beyond the 8.0 hide distance.

Rules in words. Rush. Walk 0.119 per tick (7.14 units per second) while d is above the desired distance, hold facing the hero (turn weight 1.0), back off at 0.398 per tick to the point 17 units from the hero when d is below 3.0. The chase stop discards the step within 2.0 plus the hero's radius during sequencer states 0 to 3. Smash ranges: slot 2 6.0, slot 1 4.0, slot 0 5.0 (the sequencer walks until within the range). The retreat walks home at 20.25 units per second, much faster than the 7.14 of the approach.

#### Hit and hurt windows

| Attack | Window | Hits | Damage per hit before armour |
|---|---|---|---|
| Slot 2 | s + 8 to s + 15 | one, on the first touching tick: s + 10 to s + 12 for a still hero 3.2 to 6.0 away | 45, blockable, knockback 12 |
| Slot 1 | s + 12 to s + 17 | one, s + 12 to s + 13 (3.2 to 4.0 away) | 20, knockback 3 |
| Slot 0 | s + 18 to s + 29 | one, s + 20 to s + 21 (3.2 to 5.0 away) | 10 |

The hero is assumed inside the strike volume. A swing that connects lasts longer than a whiff because state 3 runs at R = 1.0. The Tentacle can be struck in every state except a melee or missile dodge, the hidden states (cue word 0), phase 1 of the retreat and while dead; a hit cancels a swing in progress.

#### Weapon reach

Conventions are those of "Weapon reach tables" in SPEC-0064 (shared field behaviour). Row 237, Medium. The windows above are authoritative for the hit ticks. Every window carries all five weapons together (weapons 1 to 3 and 5 radius 2.0, weapon 4 radius 1.5, all height 1.5), so the table gives the union of the five.

| Slot (phase, clip) | Weapons | Window ticks (level contact) | Centre forward / up / side | Tor ahead, at the tick | Tor ahead, from the start | Tor feet vs origin |
|---|---|---|---|---|---|---|
| 0 (B, clip 14) | w1 to w5 | s+18 to s+29 (s+18 to s+29) | -1.4 to 5.7 / -0.1 to 4.1 / -0.1 to 0.2 | -2.8 to 8.2 | -2.8 to 8.6 | -4.5 to 6.0 |
| 1 (B, clip 6) | w1 to w5 | s+12 to s+17 (s+12 to s+17) | 0.2 to 5.6 / -0.1 to 2.4 / 0.0 to 1.7 | -1.8 to 8.2 | -1.5 to 8.7 | -5.0 to 3.5 |
| 2 (B, clip 23) | w1 to w5 | s+8 to s+15 (s+8 to s+15) | -1.9 to 5.7 / -0.1 to 2.9 / -0.5 to 1.1 | -3.4 to 8.2 | -3.5 to 8.7 | -5.0 to 4.5 |

All three blows reach 8.2 ahead and 1.8 to 3.4 behind. The row 239 placements (field) have the same shape with the far reach 8.0 to 8.5.

#### Defence and counters

- Melee dodge: once per hero swing, on the first tick the swing is active, only with the hold below 1, not in a reaction, not in mid-sequence except states 0, 1 and 5, hero within 8.0 and in front. Block never; dodge below 100 for all three classes: 99 of 100 swings. A dodge sets the hold to 120 and plays 55 (57 ticks, an unhittable hop of 11 units back); at the exit the hold is int[0, 120]. The first swing at a fresh Tentacle (hold 0) is dodged 99 percent of the time; after a dodge the hero has on average 60 ticks (0 to 120) in which a swing cannot be dodged. Tor's power step is never dodged.
- Missile dodge: a shot whose path within 120 ticks meets the body: roll below 100 gives outer 6 and the hold 120 (99 of 100); clip 4, 89 ticks. Roberta's quick shots are therefore dodged nearly always while the hold is below 1.
- Counters: slot 1 (trigger 6, chance 5) is eligible on every tick of a hit reaction when its cooldown is below 1, 3 < d < 10 and the hero is in front: a roll int[0, 100] below 5 (5 of 101) per tick. A reaction lasts 96 ticks, so the 20-damage smash answers a hit with probability 99 percent (mean 20 ticks later) while slot 1 is off cooldown; its cooldown of 300 to 420 ticks means it does so about once per 6 seconds. Row 239 has chance 10 and no cooldown.
- No guard: a swing that reaches the capsule hits.

#### Hero actions against this enemy

| Hero action | What the Tentacle does |
|---|---|
| Stands within 3.5 (5 x 0.7) of a wandering one | Noticed on the next cycle tick with no roll B; the second check on the same tick passes too, so the alert clip starts at once and agro follows 63 ticks after the first pass |
| First quick swing at a fresh Tentacle | Dodged 99 of 100; 57-tick hop back, then a hold of 0 to 120 |
| Hits it twice inside the hold window | Both land; reaction 96 ticks, slot 1 counter 99 percent |
| Roberta's shots | Dodged 99 of 100 while the hold is below 1 |
| Steps back beyond 8 units | Sinks (100 of 101 once the chooser fails), walks home hidden at 20 units per second, rises again if the hero is within 20 |
| Guards (L1) facing it | Blocks all three slots (none is unblockable); a blocked slot-2 smash ends two ticks after the block (s + 12 to s + 14), blocked slots 0 and 1 play the 85-tick blocked clip |
| Hero hurt, knocked down or dead | No new choice; a running swing continues |

#### Edge cases for implementers

- A blocked slot-2 smash has no blocked clip, so the sequencer goes to state 5 in the tick after the block and the end path runs the tick after that (block on the first touching tick s + 10 to s + 12, end path s + 12 to s + 14).
- The id 22 hold clip is the 36-key idle at speed byte 1 (0.5 keys per tick), not the 16-key clip the Swamp Tentacle uses.
- The 45-damage smash fires at 24 percent a tick while the hero stands within 13 units; with the cooldown of 15 to 60 ticks redrawn at its end path it can repeat every 92 to 139 ticks (end path s + 77 to s + 79 after a hit, plus the cooldown).
- A Tentacle in state 14 phase 0 or 2 is visible and hittable; the hit goes to agro with the doubled damage and a 96-tick reaction.
- Row 239 placements in Shadowglen Woods: default 8, emerge 15, 20, 30, 30 and 50, cue word 0, group 3 on the emerge-15 and one emerge-30 placement; the three Juppie-pack bodies have emerge 200 and are placed by the level script (SPEC-0015). A row-239 notice pass goes to agro at once (no 16, 17 or 18).
- The 25 sewer placements outside the CUT block use the same clip table (one table), so there are no per-region differences.

#### What a port most likely gets wrong

1. Treating row 237 as row 239. It has 60 hit points, hits for 20 and 45, dodges 99 percent of swings and has a three-clip search.
2. Giving the Tentacle recovery, wind-up or blocked clips for slot 2: none exist.
3. Dodging every swing. The dodge is once per hold; the hold after a dodge is 0 to 120 ticks.
4. Making wanderers invisible: only the state-8 placements are hidden.
5. Ending the retreat at hiding: with the hero within 20 the Tentacle rises into agro again.
6. Using the Swamp Tentacle's clip assignments (hold 22 is a different clip, the rush walk is 7.14 units per second).
7. Forgetting that slot 1 (20 damage) is the counter and has a 300 to 420 tick cooldown, while slot 2 has no counter at all.

#### Timeline test vector

Default 8, emerge distance 5, cue word 0, no group; the hero stands still facing it 3.4 units away from tick 0 (3.4 < 5 x 0.7 = 3.5), guard off. d = 3.4 is inside every slot's band and range, so no walking is needed.

| Tick | Event |
|---|---|
| 0 | Wake test passes; 1 emerge clip requested; 157 agro; 158 set-up; 159 first choice |
| 159 | Per tick: slot 2 24 %, slot 1 37 %, slot 0 19 % (needs the Tentacle to face the hero within 22.5 degrees; it faces him from the hold), nothing 20 %. c = the choice tick, s = c + 1 |
| s + 10 | Slot 2 (window open from s + 8; first touch at 3.4 units on s + 10): 45 damage, knockback 12; end path s + 79 (the hero is knocked back, so the next swings may need a walk) |
| after the end path | Recovery 0: next choice on the next tick; slot 2 is cooling (15 to 60), so slot 1 (37 percent, only if the hero is again within 10) or slot 0 |
| 600 | A motionless hero with no armour has taken 5 to 6 hits (about 95 to 175 damage; mean 135) between ticks 159 and 600 |

Wander variant: a Secret Sewers wanderer (R 30, pause 60 to 240 at 50 percent), hero standing 12 units away in its view cone. The band for a standing hero is 3.5 to 28, so the chance at 12 units is 90 - 65 x 7 / 35 = 77 percent (roll B below 77, 76 of 100) per 32-tick cycle. The first pass moves it to state 2 (look clip 16 starts); the second check on that same tick passes with the same chance and, if it does, 18 is requested and agro follows 63 ticks later; otherwise the search runs for 15 to 59 ticks plus the 196-tick search-end clip with a new chance every 32 ticks.

#### Open questions

1. Whether a wanderer in the middle of a fidget (clip 22) is interrupted by the notice pass. Resolved (Medium-High; proactive (open close-out)): yes; the pass enters state 2 and the state-2 handler runs on that same tick and requests the search animation, which replaces the fidget request (SPEC-0038 Behavior 5, state 2).
2. Weapon contact geometry (four volumes radius 2.0 and one radius 1.5, height 1.5). Resolved by the general rule (SPEC-0064, Shared field behaviour: Weapon contact and the hero's hurt shape); the distances are in the Weapon reach table; stand-in only for a port without attachment frames: a swing hits a level hero inside the "at the tick" range of the slot's row (2.8 behind to 8.2 ahead for slot 0, 1.8 behind to 8.2 ahead for slot 1, 3.4 behind to 8.2 ahead for slot 2).
3. Whether the Meridell Sewers emerge-1 and emerge-3 placements can be woken by anything but the hero's standing within 0.7 or 2.1 units. Resolved (High; proactive (open close-out)): a waiting enemy leaves state 8 only by the emerge-distance test, the group alarm of its group, a script call that hands it to the alert state, a hit, or a socialize pull (SPEC-0038 Behavior 40). A placement with cue word 0 cannot be hit while it waits, the packs of these placements make no such script call, and these rows have no socialize flag, so only the hero's standing within the distance and the group alarm can wake them.
4. The tick on which phase 1 of state 14 ends when the Tentacle is already home. Resolved (Medium-High; proactive (open close-out)): the tick after the idle pick request (request on tick t, clip current from tick t + 1; SPEC-0038 Behavior 50).

### Slorg and Giant Slorg (rows 229 and 230)

**Player's answer (R-0149).** Fight the Giant Slorg in its open windows: each dodge (99 of 100, melee and shot) lasts 57 ticks and is followed by 60 to 120 ticks in which every swing and shot lands (about five swings at one per 18 ticks), and it cannot dodge a swing that lands while it is in its strike states; it rushes and spits rather than flees, so nothing makes it give up or hide (SPEC-0038 Behavior 44). The Slorg dodges only 49 swings and 74 shots in 100 with the same window.

One guide for both. The Slorg (monster type 90, row 229) and the Giant Slorg (type 91, row 230) share one skeleton, one 24-clip set, one 166-entry action table layout and one attack plan: a straight spit bolt from far away, a straight-line charge, and a bite that is only used at point-blank range. Both are slow walkers (2.9 units per second) that keep their distance and shoot; the Giant has more hit points (250), a damage reduction of 5, a 99 percent dodge, a heavier bolt, and a bite that stuns the hero for 6 to 8 seconds. Neither hides by distance, neither retreats, neither flies. Confidence: High for the rows, clips and arithmetic; Medium-High for the idle states; Medium for contact geometry and for the sentry shuffle.

#### Identity and tunables

| Item | Slorg (row 229) | Giant Slorg (row 230) |
|---|---|---|
| Records | Type 90, variant 0, name "Slorg"; 36 records (26 in play), one clip table in play | Type 91, variant 0, name "Giant Slorg"; 19 records (14 in play), one clip table in play |
| Class, approach | 17, rush; T = 4 (water: ignores stun); Display in default 1; never flies | same |
| Hit points | 75 | 250 |
| Body | Radius 1.175, height 3.5, weight 5.5; manager weight 2; hostile | Radius 3.0, height 5.0, weight 5.5; manager weight 2; hostile |
| Damage reduction | 0 | 5.0: every hero hit subtracts 5 from its damage product before the status scale and the result is cut to a whole number; a result below 1 deals 0, and the hit is absorbed only when the product is 5 or less (SPEC-0036; R-0150) |
| Hurt shape | Six capsules (bones 1, 2, 3, 23, 24, 25): lengths 0.78, 0.71, 1.37, 0.76, 0.78, 0.63, radii 1.36, 0.83, 0.97, 0.71, 0.60, 0.42 (model units; Medium) | the same capsule table (the row scale is not applied; Medium) |
| Weapon volumes | Weapon 1 radius 2.0 height 4.0; weapon 2 radius 3.0 height 1.0; weapon 3 radius 5.0 height 3.0; weapons 4 and 5 radius 2.0 height 1.0 | the same |
| Notice | View cosine 0.707 (45 degrees), awareness 60, timer 30 (cycle 32 ticks), near range 3 (chance 75), far range 40 (chance 10); search countdown int of float[30, 120) | View cosine 0.0, awareness 100, timer 30, near range 5 (chance 99), far range 70 (chance 20): chance(d) = 99 - 79 (d - 5) / 65 |
| Turn | Aim turn weight 1.0 | 1.0 |
| Agro bands | Rush band 5.0 to 20.0; desired distance 5 + 15 f (8.0 to 17.0) | Rush band 6.0 to 25.0; desired 6 + 19 f (9.8 to 21.2) |
| Recovery after an attack | int[60, 120] | int[60, 120] |
| Missile defence | Hold 60, dodge chance 75 (74 of 100), block 0 | hold 60, dodge 100 (99 of 100) |
| Melee defence | Block 0; dodge 50 for all three classes (49 of 100); hold 60 at the start, drawn int[60, 120] at the exit | dodge 100 for all three (99 of 100); same hold and timer |
| Other | No hide or distance-hide gate, no steal, no socialize, no map search | same |
| Hero sword damage (R-0150; High) | Defence 0: sword 99 deals 7 per plain swing, 14 on the finishing swing, 5 on the chained repeat | Defence 5: sword 99 deals 2, 9 and 0; sword 105 deals 35, 75 and 25 (first hit on an enemy that is not yet alert doubled) |
| Death | Disappear 300; clip 58 | same |
| Stand-off radius | 10 | 10 |

Giant Slorg against the Squire's Sword (R-0150; High for the numbers, Medium for the story position). The 0 is not a rule of the enemy: a plain swing of the Squire's Sword (stat A 7) leaves 7 - 5 = 2, the finishing swing 14 - 5 = 9, and only the 0.75 steps (near-target swat, chained repeat: 5.25 - 5 = 0.25) cut to 0. A build that deals 0 on every swing is subtracting too much (for example using the step factor 0.75 on every swing, or subtracting the defence twice). The design intent is that the first sword is a poor weapon here (250 hit points at 2 to 9 per swing, and the Giant's melee dodge chance is 99 in 100 per decision, so many swings never connect), so the player is expected to come with a Mote of kind 1 (items 160 and 161, matching target type 4: sword 99 with Mote 161 deals 12 per plain swing, with Mote 160 deals 7), with Roberta's wand shots, or with a better sword; the Altadorian Sword is not available in play at all. Placements in play, in the story order of the Act 3 chain (SPEC-0058): Bogshot Sinkholes (pack 732, one Giant Slorg beside Haplo; the optional Missing Kid errand reached from the swamp), Black Keep Dungeon (pack 990, two; reached after the Black Knight, flag 298), Werelupe Sinkholes (pack 1253, one), Ancient Sewers (pack 1129, one), Meridell Treasure Vault (pack 1096, three) and Secret Sewers (pack 1369, six). No Giant Slorg stands in Act 1 or Act 2. By then (Medium) Tor can hold the Squire's Sword, which is in Act 1 either paid for at the enlistment (pack 1148 returns item 99 for 100 Neopoints when no sword is traded in) or sold in pack 1133 (catalogue price 500 Neopoints); his chest sword from the first house is the Wooden Sword (98). Better swords are optional buys: the Knight's Sword (item 100, 1,500 Neopoints, pack 856) and the Battle Blade (item 102, 16,000 Neopoints before the Black Knight and 4,000 after, pack 1008). The Sword of the Ancients (103) is given in the Knights Crypts, in the same chapter as the Vault. Which sword the player brings to the first Giant Slorg depends on what he has bought, so only play can settle it (needs a runtime check). Default if unresolved (rewrite choice): treat the first Giant Slorg as the Bogshot Sinkholes one and give Tor sword 99 there.

Attack slots (slot 3 unused; tested 2, 1, 0; "d" is the distance to the hero):

| Slot | Role | Band (strict) | Chance | Arc (dot above) | Range | Cooldown int[...] | Damage | Knockback | Orient | Speeds | Counter |
|---|---|---|---|---|---|---|---|---|---|---|---|
| 0 | Missile (type 1), straight bolt | Slorg 5 to 20; Giant 7 to 100 | 75 | 0.0 | (not used: missile) | 60 to 180 | Slorg 15; Giant 25 | 0 | Slorg no; Giant yes | 1.0 / 1.0 / 1.0 | none |
| 1 | Melee bite | 0 to 6 | 75 | 0.9239 | Slorg 3.0; Giant 5.0 | Slorg 30 to 60; Giant 300 to 600 | Slorg 20; Giant 25 and a stun (below) | 0 | Slorg no; Giant yes | 1.0 / 2.0 / 1.0 | Slorg trigger 1, chance 50; Giant trigger 6, chance 20 |
| 2 | Charge, kind 0 | Slorg 6 to 30; Giant 5 to 30 | 50 | 0.9239 | 12.0 | 30 to 180 | 6 | 3 | Slorg no; Giant yes | Slorg 1.0 / 1.0 / 1.0; Giant 1.3 / 1.3 / 1.3 | trigger 5, chance 25 (practically unreachable) |

Slot 0 projectile: flight mode 0 (straight), launch speed 1.0 per tick (Giant 1.2), range 90 (life 90 ticks, Giant 75), radius 0.5, aim inaccuracy 0, spawn count 1, no area, no status, no spin, visual kind 0, element T = 4, damage 15 (Giant 25). The slot's flag word is 0x100, which no class-17 routine reads. Slot 2: charge distance float[12, 25) (see below), blocked-charge kind 4 (a blocked charge starts hit reaction kind 3), flags 0x100 (blockable). The Giant's slot 1 applies status 0 (stun) with chance 100 (100 of 101 on the int[0, 100] roll) for int[360, 480] / 60 = 6 to 8 seconds; the Slorg's statuses all have chance 0.

#### Animation ids and clips (same layout for both; entries marked G exist only on the Giant)

| Id | Use | Clip |
|---|---|---|
| 0 | idle pick | clip 1 (25 keys, loops) |
| 115, 116 | idle fidgets | clip 2 (30 keys), clip 3 (57 keys); each idle-picker expiry picks among 115 to 119 and uses the one it hits when present (2 of 5) |
| 9 | hide pick | clip 18 (2 keys, one-shot, gate s + 5) |
| 10 | emerge | clip 4 (20 keys, one-shot, forward 1.51 units per second), gate s + 77 |
| 2, 21, 4 (G) | wander walk, rush walk, patrol walk | clip 0 (21 keys, loops, forward 2.90 units per second = 0.048 per tick) |
| 3, 14 (G), 5 (G), 19 (G) | rest fidget | clip 3 (57 keys, one-shot), gate s + 224 |
| 1, 22, 7 (G) | holds, guard stand (G) | clip 7 (18 keys, loops) |
| 6, 8, 15 (G) | patrol fidget, guard look (G) | clip 2 (30 keys), gate s + 116 |
| 16 | search look, victory pose (Slorg 73) | clip 5 (32 keys, one-shot), gate s + 124 |
| 18, 13 (G) | alert | clip 6 (10 keys, one-shot), gate s + 37 |
| 23 | back off | clip 8 (21 keys, loops, backward 1.32 units per second = 0.022 per tick) |
| 34 | slot 0 strike (B, the spit) | clip 9 (23 keys, one-shot, event word 0x24 on key 8 only), gate s + 89 |
| 39 | slot 1 strike (B, the bite) | clip 10 (15 keys, one-shot, weapon-3 bit 0x4 on keys 5 to 7) |
| 43 | slot 2 wind-up (A) | clip 11 (11 keys, one-shot, forward 0.41 units per second), gate s + 41 (Giant s + 32) |
| 44 | slot 2 strike (B, the run) | clip 12 (7 keys, loops, weapon-1 bit 0x1 on every key, forward 27.56 units per second = 0.459 per tick; Giant at 1.3: 35.83 = 0.597 per tick) |
| 47 | slot 2 recovery (E) | clip 13 (25 keys, one-shot, forward 2.03 units per second), gate s + 96 (Giant s + 74) |
| 53 / 54 | missile dodge | clip 23 (15 keys, one-shot, sideways 3.23 units per second) / clip 14 (sideways -3.08), gate s + 57 |
| 55 | melee dodge | clip 15 (15 keys, one-shot, backward 5.60 units per second), gate s + 57 |
| 57, 62 | hit reaction kind 0, kind 3 first part | clip 16 (10 keys, one-shot), gate s + 37 |
| 64 | kind 3 second part | clip 20 (22 keys, one-shot, forward 0.39 units per second), gate s + 85 |
| 58 | death | clip 17 (40 keys, one-shot), gate s + 156, progress 0.95 at s + 151 |
| 69 | confused | clip 19 (42 keys, loops) |
| 63 | (unused, 1 key) | clip 22 |
| 73 | victory pose | Slorg clip 5 (gate s + 124); Giant clip 3 (gate s + 224) |
| 81, 82 (G) | disappear, appear (unused by these rows: no Display-0 setter, no retreat) | clip 18 / clip 4 |
| 17 | search end | none: when the search countdown runs out, the setter with 0 runs at once |
| 33, 35, 36, 37, 38, 40, 41, 42, 45, 46 | wind-ups of slots 0 and 1, follow-throughs, blocked clips, recoveries of slots 0 and 1, charge follow-through and blocked | none: skipped in the same tick |
| 4, 6, 7, 8 (Slorg only) | patrol walk, patrol fidget, guard stand and look | none: the request plays clip 0, the walk clip (see Edge cases) |

#### Start and wake

Slorg default states: 5 (wander) on 9, 6 (patrol) on 7, 7 (guard) on 2, 8 (hidden) on 8; Giant: 5 on 6, 6 on 6, 7 on 2, none hidden. Emerge distances (Slorg, state 8): 1 (2, the Treasure Vault pair in group 1), 5, 10 (3), 15 (cue word 1: never hidden) and 20; cue word 0 on the others.

- **Hidden (state 8).** The Slorg stands at home playing id 9 (it walks home first at 2.9 units per second if displaced by more than 0.35), not drawn and not hittable. Wake test (SPEC-0038 Behavior 40) against emerge distance x F1 (a standing hero 0.7: 10 becomes 7.0, 20 becomes 14); emerge clip 10 (77 ticks); agro at w + 78; set-up w + 79; first choice w + 80. The Treasure Vault pair (emerge 1, group 1) is a pure ambush: each rises when the hero stands within 0.7 units or when the other has entered agro (15 to 45 ticks later).
- **Wander, patrol, guard (5, 6, 7).** As the shared rules. A Slorg walks at 0.048 per tick, so a wander leg of 20 units takes 7 seconds; its rest fidget 3 plays for 224 ticks; the idle fidgets 115 and 116 add occasional clips.
- **Search (state 2).** A notice pass plays 16 (look, 124 ticks); a pass while 18 is not playing requests 18 (10 keys, 37 ticks) and agro follows when its gate passes. With no pass the search countdown int[30, 119] runs; at zero the table has no id 17, so the setter with 0 runs at once and the Slorg returns to its default state. The Slorg's pass chance at distance d (standing hero, in the 90 degree front cone) is 1 inside 2.1 (3 x 0.7), roll A below 60 of 100, and chance(d) = 75 - 65 (d - 3) / 37 between 2.1 and 28, halved outside the cone. The Giant sees much farther: for a standing hero 3.5 to 49 units (spotted by a running hero at up to 122 units), chance 99 - 79 (d - 5) / 65, no cone (cosine 0.0).
- **Return to the default** after losing the hero: the shared exits (leash, lost-sight search, victory pose). A Slorg whose default is 8 walks home and hides again; patrols and wanderers resume.

#### Per-tick order (agro)

As SPEC-0038 Behavior 44: recovery countdown, missile defence, melee defence, lost-sight test, normal choice, counter choice, outer switch, agro tail. The distance-hide gate does nothing (flag 0). The outer switch starts at outer 0 (one tick) then outer 1 (rush).

#### State table

| State | Action id and clip | Keys | Speed | Gate and exits |
|---|---|---|---|---|
| Hidden (class 8) | 9 -> clip 18 | 2 | 1.0 | Wake rule |
| Emerge | 10 -> clip 4 | 20 | 1.0 | Gate s + 77, then agro |
| Search (state 2) | 16 -> clip 5, 18 -> clip 6 | 32 / 10 | 1.0 | 18: gate s + 37 then agro; no 17: default state at once |
| Set-up (outer 0) | agro idle pick 1 -> clip 7 (the table maps id 1 to its hold loop) | 18, loops | 1.0 | One tick, then outer 1 inner 1 |
| Rush hold (inner 1) | 22 -> clip 7 | 18, loops | 1.0 | d < 5.0 (Giant 6.0): back off; d > 20.0 (Giant 25.0): approach |
| Approach (inner 0) | 21 -> clip 0 | 21, loops | 2.90 units per second | Until d < desired |
| Back off (inner 2) | 23 -> clip 8 | 21, loops | 1.32 units per second backward | Toward hero - dir x 22.0 (Giant 27.0); until d > desired |
| Spit (slot 0) | 34 -> clip 9 | 23 | 1.0 | Bolt launched on s + 32; B gate s + 89 (no C), end path s + 90 |
| Bite (slot 1) | 39 -> clip 10 | 15 | wind-up 1.0 (request tick), strike 2.0 | Window opens s + 11. Slorg: first touch on a still hero s + 11 (any start from the chase stop 2.35 to the range 3.0); Giant: s + 14 (start 4.2 to 5.0; its chase stop is 4.175). Whiff: gate s + 29, end path s + 30; after a hit on s + 11: gate s + 46, end path s + 47 (Giant, hit on s + 14: gate s + 43, end path s + 44); blocked on the touch tick t: state 4 at t + 1, no D, end path t + 2 |
| Charge (slot 2) | A 43 -> clip 11, B 44 -> clip 12, E 47 -> clip 13 | 11 / 7 / 25 | 1.0 (Giant 1.3) | State 6: A to its gate; state 7: run; state 12: E to its gate, end path |
| Melee dodge (outer 5) | 55 -> clip 15 | 15 | 1.0 | Gate s + 57, then the defence exit |
| Missile dodge (outer 6) | 53 or 54 -> clip 23 / 14 | 15 | 1.0 | Gate s + 57, then the defence exit |
| Hit reaction kind 0 (outer 10) | 57 -> clip 16 | 10 | 1.0 | Gate s + 37, then outer 0 |
| Hit reaction kind 3 (blocked charge) | 62 -> clip 16, then 64 -> clip 20 | 10 / 22 | 1.0 | s + 37, then s + 85, then outer 0 (about 124 ticks) |
| Wait in line (outer 12) | 2 -> clip 0 (the walk clip; the same clip as the rush id 21) | 21 | 2.90 units per second | Stand-off point in a 10 x 10 square around the hero; 73 on arrival |
| Victory pose | 73 | | 1.0 | Slorg gate s + 124; Giant s + 224 |
| Dead | 58 -> clip 17 | 40 | 1.0 | Plays once; disappear 300 from progress 0.95 |

Defence exit: outer 0, base state 1, defence hold int[60, 120].

#### Attack chooser

Slots tested 2, 1, 0 each tick under the usual gates (hero attackable, recovery below 1, outer 1, 3, 4 or 11). Slot 2: cooldown below 1, band, facing cosine above 0.9239, roll int[1, 100] below 50, and a route to the hero (a charge slot needs one). Slot 1: band 0 to 6, roll below 75 (74 of 100), cosine above 0.9239, route or d < 10. Slot 0: band, cosine above 0.0, roll below 75; a missile slot needs no route. The first to pass wins and the three distances are redrawn.

| d | Slorg: start chance | Giant: start chance |
|---|---|---|
| 0 < d <= 5 | 74 % (slot 1) | 74 % (slot 1) |
| 5 < d < 6 | 93 % (slot 1 74, slot 0 19) | 87 % (slot 2 49, slot 1 38) |
| 6 <= d <= 7 | 87 % (slot 2 49, slot 0 38) | 49 % (slot 2) |
| 7 < d < 20 | 87 % (slot 2 49, slot 0 38) | 87 % (slot 2 49, slot 0 38) |
| 20 <= d < 30 | 49 % (slot 2) | 87 % (slot 2 49, slot 0 38) |
| 30 <= d < 100 | 0 | 74 % (slot 0) |
| d >= 100 (Giant), d >= 30 (Slorg) | 0 | 0 |

The slot cooldowns thin these out: slot 2 and slot 0 each wait 30 to 180 and 60 to 180 ticks after use (the Giant's bite waits 300 to 600). The enemy stands between choices at its desired distance, so most choices are slot 0 and slot 2 at 8 to 21 units.

#### Spit (slot 0), the Slorg's main weapon

Choice tick c: for a missile slot sequencer state 0 goes to state 1 at once (no approach); s = c + 1. State 1 requests B (clip 9; the wind-up id 33 is missing) and state 2 begins. The spawn event is the event word 0x24 on key 8 of the clip, active on s + 32 to s + 35 (four ticks); the first tick of the window launches one bolt (the spawn pending flag is then set and clears when the event ends; the count of 1 is reached at once, so one bolt per swing). The launch point is emit point 2 (the lowest set point bit is 4). The bolt flies straight from that point toward the hero's position plus (0, 2.5, 0) at the moment of launch, at 1.0 unit per tick (60 per second; Giant 1.2), with an aim scatter of 0 (scatter 5.0 in a random horizontal direction when the shooter is outside the camera cone: dot of the camera forward with the direction from the camera to the shooter below 0.85); it lives 90 ticks (75 for the Giant) and hits a hero whose body it touches within a sphere of radius 0.5 (hit rules and shield of SPEC-0029 Behavior 9; damage 15 or 25). Sequencer state 3 waits for B's gate at s + 89 (no C), state 5 on that tick, end path s + 90 (no E): recovery int[60, 120], slot 0 cooldown int[60, 180]. A Slorg's spit is a missile slot, so the strike state turns it toward the hero every tick at the row's aim turn weight even without the orient flag; the bolt direction is computed at launch from the launch point to the hero, so the shot is accurate whatever the Slorg's facing is then; the Giant (orient flag) also turns to the hero during the wind-up.

#### Charge (slot 2; sequencer states 6, 7, 8, 12)

1. Choice tick c: sequencer state 6. Each tick of state 6 the enemy aims at the hero with weight 1.0 (turn value) and stores its own position as the run's start point. A (id 43, clip 11) is requested on c and plays to its gate at c + 41 (Giant c + 32). During A the enemy turns to the hero every tick, so it tracks a hero who circles.
2. Tick c + 41 (Giant c + 32): A passes its gate, B (id 44, clip 12) is requested and state 7 is set with the tick counter 0. The enemy's heading is fixed from this tick on; the run never steers.
3. State 7 from the next tick: the counter goes up by one, t = the distance run from the start point. Each tick the strike test runs against the weapon-1 volume (radius 2.0, height 4.0; the clip carries bit 0x1 on every key so the volume is live for the whole run). A hit goes to state 8; a blocked strike to state 9; t at least the drawn charge distance goes to state 12; more than 100 ticks with speed at most 0.001 also goes to 12. Per-tick travel 0.459 (Giant 0.597); the charge distance is a float[12, 25) drawn at the choice (the draw uses the selected slot's own range words, so it is the slot-2 range only when slot 2 was the slot just chosen), so the run lasts 27 to 55 ticks (Giant 21 to 42).
4. State 8 (after a hit): kind 0 plays C (id 45), which is missing, so nothing changes; the run continues until t reaches the charge distance or the navmesh blocked flag is set, then state 12. The strike test is not run again, so a charge hits at most once. A hit deals 6 damage and knockback 3 (the hero goes to his knockdown state 48, SPEC-0036), then the Slorg runs on through him.
5. State 9 (blocked): the slot's blocked-charge kind 4 is not 0, 1 or 2, so hit-reaction kind 3 starts (ids 62 then 64: 37 + 85 ticks of staggering, the Slorg has stopped its run). A hero holding L1 facing the Slorg (dot at most -0.5) blocks the charge. The shield absorbs the hit.
6. State 12: E (id 47, clip 13) plays to its gate (96 ticks; Giant 74), then the end path (recovery int[60, 120], slot 2 cooldown int[30, 180]). Total: 41 + (27 to 55) + 96 = 164 to 192 ticks (Giant 32 + (21 to 42) + 74 = 127 to 148).

A hero standing at distance D in front of the Slorg (measured when the run starts) is struck on the first run tick on which his centre is within about 3.8 ahead of the Slorg's centre (the "at the tick" reach of weapon 1, Weapon reach table), so the charge connects when the drawn distance exceeds about D - 3.8: for D = 12 always, for D = 20 with probability (25 - 16.2) / 13 = 68 percent, for D = 28 about 6 percent, from about 28.8 never (same rule for the Giant). Medium (level hero, straight line).

#### Bite (slot 1)

For d above 3.0 (Giant 5.0) the sequencer starts in state 0 and walks (id 21, 0.048 per tick) until within the range, then state 1 on the next tick; the chase stop discards a step within the sum of the radii (1.175 + hero radius; Giant 3.0 + hero radius). Because the band is 0 to 6, a Slorg at 5 units spends about 40 ticks walking in before it swings. The bite's window opens at s + 11 and weapon 3 (radius 5.0, height 3.0) touches a still hero from any distance the Slorg can start the swing from: on s + 11 for the Slorg (s + 12 for a hero at exactly 2.5 units), on s + 14 for the Giant (which starts 4.2 to 5.0 away); after a Giant's hit on s + 14 the gate is s + 43 and the end path s + 44. Damage 20 (Giant 25 and a stun of 6 to 8 seconds on 100 of 101 rolls). Whiff: gate s + 29, end path s + 30; after a hit: the clip finishes at rate 1.0, gate s + 46, end path s + 47 (gate s + 45 and end path s + 46 for the hit on s + 12). A blocked bite goes to state 4 at s + 12; D is missing, so state 5 follows and the end path is s + 13.

#### Movement

Movement table (movement pass; both rows, the Giant's values in brackets where they differ; the Giant's size does not scale any of these; speed byte 2):

| Mode | Source | u/t | u/s | Turn | Clip (id: clip, keys, loop) | Exits |
|---|---|---|---|---|---|---|
| Wander, patrol, walk home (state 8), wait in line, rush approach (inner 0), bite approach (state 0) | root | 0.0483 average; per key 0.003 to 0.090 (a lurching crawl) | 2.90 | steering 0.3 / 0.1 | 2, 4, 21: clip 0, 21, loop (the Slorg table has no id 4: clip 0 plays, the same clip) | goal reached; d below desired (5 + 15f, 8 to 17; Giant 6 + 19f, 9.8 to 21.2) |
| Rush hold (inner 1) | none | 0 | 0 | aim 1.0 | 22: clip 7, 18, loop | d below 5.0 (6.0) or above 20.0 (25.0) |
| Back off (inner 2) | root, backward | 0.0220 | 1.32 | faces the hero | 23: clip 8, 21, loop | d above desired; point 22 (27) from the hero |
| Emerge (state 8 wake) | root | 2.02 units forward over the clip | - | none | 10: clip 4, 20, one-shot | gate |
| Charge wind-up (state 6) | root | 0.249 units over the clip at the wind-up word | - | aim 1.0 each tick | 43: clip 11 (Giant at word 1.3) | gate c + 41 (c + 32) |
| Charge run (states 7, 8) | root x strike word | 0.459 (0.597) average | 27.56 (35.83) | none: the heading is fixed | 44: clip 12, 7, loop | charge distance float[12, 25); non-solid only on state-7 ticks (kind 0) |
| Dodges | root | 53: 3.24 sideways, 54: 3.08 to the other side, 55: 5.60 backward over the clip | - | none | clips 23, 14, 15 (15 keys) | gate |
| Hit reaction kind 0, kind 3 | none (64: 0.57 forward) | 0 | 0 | snapped to the attacker | 57, 62: clip 16; 64: clip 20 | gate |
| Death | none | 0 | 0 | none | 58: clip 17 | removal |

Worked example (chase from 30). A Slorg in agro, hero standing 30 away on flat ground, desired distance drawn 12.5, no attack starting (the spit's band is 5 to 20, so it does start in practice once d is below 20). Tick 1 outer 0; tick 2 rush entry (hold); d above 20 gives inner 0 on tick 3 and the walk clip is requested; first step tick 4. At 0.0483 u/t it needs 17.5 / 0.0483 = 362 ticks (6.0 seconds), so it holds at about tick 366; within one clip cycle (84 ticks, 4.06 units) the step varies from 0.003 to 0.090 per tick, so the stop tick can move by up to about 20 ticks. A walking hero (13.5 u/s) is 4.7 times as fast.

What a port most likely gets wrong (movement): a smooth glide (the crawl has near-stops inside each cycle); a fast or scaled Giant (the Giant walks exactly as slowly as the Slorg; only its charge word 1.3 makes its run faster); stopping the charge at the hero (kind 0 runs through him on the run ticks).

Rules in words. Rush archetype at 2.90 units per second (0.048 per tick) while d is above the desired distance; hold facing the hero (turn weight 1.0, clip 7); back off at 1.32 units per second when d is below 5.0 (Giant 6.0) toward the point 22 (Giant 27) units from the hero. The only fast movement is the charge (27.56 units per second; Giant 35.83) and the dodges. The chase stop applies in sequencer states 0 to 3 only, so the run passes through the hero. With no route the waypoint walk and leash apply (SPEC-0038 Behavior 23).

#### Hit and hurt windows

| Attack | Window | Hits | Damage per hit before armour |
|---|---|---|---|
| Spit | launch tick s + 32 (bolt arrives distance/1.0 ticks later) | one bolt | 15 (Giant 25) |
| Bite | s + 11 to s + 16 | one, on the first touching tick: s + 11 (Slorg), s + 14 (Giant) | 20 (Giant 25 and stun) |
| Charge | from the first run tick until contact | one | 6, knockback 3 |

The Slorg can be struck in every state except a dodge (outer 5 and 6), while hidden with cue word 0, and while dead. A hit cancels a spit, a bite or a charge in progress (hit reaction), apart from a blocked bite.

#### Weapon reach

Conventions are those of "Weapon reach tables" in SPEC-0064 (shared field behaviour). Rows 229 and 230, Medium. The windows above are authoritative for the hit ticks.

| Kind, slot (phase, clip) | Weapon | Window ticks (level contact) | Centre forward / up / side | Tor ahead, at the tick | Tor ahead, from the start | Tor feet vs origin |
|---|---|---|---|---|---|---|
| Slorg 1 bite (B, clip 10) | w3 (5.0 x 3.0) | s+11 to s+16 (s+11 to s+16) | -1.1 to 0.8 / 2.1 to 2.4 / 0.1 to 0.5 | -6.2 to 6.4 | -6.4 to 7.1 | -4.0 to 6.0 |
| Slorg 2 charge (B, clip 12) | w1 (2.0 x 4.0) | s+1 to s+27 (s+1 to s+27) | -0.7 to 0.7 / -0.2 to 2.3 / 0.0 | -3.4 to 3.8 | -2.2 to 15.3 | -5.5 to 4.5 |
| Giant Slorg 1 bite (B, clip 10) | w3 (5.0 x 3.0) | s+11 to s+16 (s+11 to s+16) | -1.2 to 1.4 / 2.7 to 3.5 / 0.1 to 0.7 | -6.4 to 7.2 | -6.6 to 7.9 | -3.0 to 7.0 |
| Giant Slorg 2 charge (B, clip 12) | w1 (2.0 x 4.0) | s+1 to s+21 (s+1 to s+21) | -0.7 to 0.7 / 0.1 to 3.3 / 0.0 | -3.4 to 3.8 | -2.2 to 15.6 | -5.0 to 5.5 |

The bite reaches 6.4 ahead (Giant 7.2) and about the same behind; the charge volume is narrow (3.4 behind to 3.8 ahead at the tick) and its far figure (15.3, Giant 15.6) is the run's own travel, since the charge window opens on the first tick of the run.

#### Defence and counters

- Melee dodge (once per hero swing, hold below 1, not in a reaction, not mid-sequence except sequencer states 0, 1 and 5, hero within 8.0 and in front): the block roll never passes; the dodge roll passes below 50 for every class on the Slorg (49 of 100) and below 100 on the Giant (99 of 100). A dodge sets the hold to 60 and plays 55 (57 ticks, about 5.6 units backward); at the exit the hold is int[60, 120]. Tor's power step is never dodged.
- Missile dodge: a hero shot whose path within 60 ticks meets the body: roll below 75 (Slorg, 74 of 100) or below 100 (Giant, 99 of 100) plays 53 or 54 (57 ticks, about 3 units to one side).
- Hit reaction kind 0: 37 ticks (id 57, clip 16); restarts on each hit.
- Counters. Slorg slot 1, trigger 1 (the active hero's recover window is open; Tor's swing clips carry it, Roberta's do not): a Slorg within 6 units and in front of Tor starts the 20-damage bite on any tick of the window with probability 50/101 (roll int[0, 100] below 50) in any outer sub-state (the counter choice has no sub-state gate, so it can replace a spit in progress with the bite), and then waits the bite's cooldown of 30 to 60 ticks; this makes close combat with Tor costly. The Giant's slot 1, trigger 6: on every tick of a hit reaction it rolls 20/101 for the bite, so a hit on a Giant inside 6 units in front of it is answered within a few ticks (probability about 99.97 percent over the 37-tick reaction; mean 4 ticks) unless the 300 to 600 tick cooldown is running: a 25-damage bite with a 6 to 8 second stun. Slot 2's trigger 5 needs a blocked bite and L1 held and a distance above 6, which a blocked bite cannot have, so it never fires.

#### Hero actions against these enemies

| Hero action | What the Slorg does | What the Giant does |
|---|---|---|
| Stands still in the cone beyond 28 (49) units | Nothing | Nothing |
| Walks into the notice band | Look (124 ticks), alert (37), agro | same; much larger band |
| Stands at 8 to 20 units | Spits (38 %) or charges (49 %) per tick | same |
| Stands beyond 20 units | Charges at 49 % a tick up to 30 units | Spits (and charges up to 30) |
| Sidesteps the charge after A | The run keeps its heading; it misses | same |
| Guards (L1) facing it | Blocks the charge (Slorg staggers 122 ticks), the bite and the bolt | same |
| Quick or normal swing in front within 8 | Dodged 49 of 100 | Dodged 99 of 100 |
| Tor swings inside 6 units in front | Bite counter 50 of 101 a tick during the recover window | Not triggered by the swing |
| Lands a hit | Damage by SPEC-0036, doubled if not in agro; 37-tick reaction | Damage minus 5; 37-tick reaction; bite counter about 99.97 percent unless on cooldown |
| Roberta's shots | Dodged 74 of 100 | 99 of 100 |
| Hero hurt, knocked down or dead | No new choice; cooldowns stop; a running swing or charge continues | same |

#### Edge cases for implementers

- Sentries and rests. The Slorg table has no ids 6, 7 or 8, so a Slorg guard (two placements) and a patrol fidget play clip 0, the walk clip, in place of the stand or look clip: its root travel (2.9 units per second) carries it off its post; when it is farther than 0.5 from home it walks home with the same clip. The result is a slow shuffle around the post. The Giant has all three ids.
- The charge distance. Every normal attack choice redraws the stored distance of every charge slot from the range words of the slot just chosen. After choosing slot 0 or slot 1 the stored distance for slot 2 is a draw from float[0, 0) = 0, which would end a counter charge at once; because slot 2's counter cannot fire, only a normal choice of slot 2 (which redraws from slot 2's own range) ever runs a charge.
- A charge needs a route to the hero even at point-blank range; the spit and the bite do not (the bite needs d < 10 or a route).
- Packs: the manager weight is 2, but with the starting gear (capacity 0) only the first listed attacker may start an attack; the other Slorgs of a pack wait in line (outer 12) in a 10 x 10 square about the hero and attack as the first one leaves agro or the capacity grows.
- A killing blow on the hero sends the attacker to the victory pose (Slorg clip 5, 124 ticks; Giant clip 3, 224 ticks), then the default state.
- The Slorg's alarm group (the two Treasure Vault ambushers) works as the shared rule: the second wakes within 15 to 45 ticks of the first entering agro.
- The Giant's damage reduction makes a weak sword hit worth 0: a product of 5 or less becomes 0 damage (the hit lands as absorbed).
- A Giant's bite stun is a hero status: for 6 to 8 seconds the hero cannot act while the Giant keeps spitting and charging.

#### What a port most likely gets wrong

1. Making the Slorg a melee enemy. Its normal attacks are the bolt (15) and a charge that does 6; it bites only inside 6 units.
2. Steering the charge. The run keeps the heading it had when the wind-up ended and runs 12 to 25 units through the hero.
3. The charge wind-up: 41 ticks (32 for the Giant) during which the enemy keeps facing the hero; the run lasts 27 to 55 ticks; the recovery clip is 96 ticks. The whole charge is 164 to 192 ticks.
4. The spit timing: the bolt leaves on s + 32 from emit point 2, straight at the hero's position at that moment, not along the facing.
5. Giving the Slorg melee speed. It walks at 2.9 units per second; the hero outruns it easily.
6. Skipping the Giant's damage reduction, its 99 percent dodges and the stun bite (a 25-damage bite with a 6 to 8 second stun on 100 of 101 rolls).
7. The Slorg's counter against Tor's swings and the Giant's counter against any hit (both close-range).
8. Forgetting the 5.0 scatter of off-camera shots.
9. Blocked charges: the Slorg staggers for about 122 ticks; a port that plays a blocked clip or skips the stagger misses the hero's best counter.

#### Timeline test vector

Slorg, default 8, emerge distance 10, cue word 0, no group; the hero stands still facing it 6.5 units away from tick 0 (6.5 < 10 x 0.7 = 7.0), guard off.

| Tick | Event |
|---|---|
| 0 | Wake test passes; 1 emerge clip 10 requested (gate 77); 78 agro; 79 set-up; 80 first choice |
| 80 | d = 6.5, in front: slot 2 (charge) 49 %, else slot 0 (spit) 38 %, nothing 13 %. Slot 1 is out of its band (it ends at 6) |
| spit chosen at c | s = c + 1; bolt launched c + 33 and reaches the hero on c + 37 (its fifth position is within 1.675 of the hero's axis); 15 damage; B gate c + 90; end path c + 91; recovery int[60, 120]; cooldown int[60, 180] |
| charge chosen at c | A requested c (gate c + 41); B at c + 41; run from c + 42; contact once the hero is within about 3.8 ahead, after about 2.7 units at 0.459 per tick (6 ticks): hit at about c + 48 for 6 damage and knockback 3; the run goes on to 12 to 25 units, 27 to 55 ticks in all; E requested at about c + 68 to c + 98; end path about c + 165 to c + 195; recovery int[60, 120]; cooldown int[30, 180] |
| 600 | A motionless hero with no armour and no guard has taken 2 to 3 attacks between ticks 80 and 600: about 12 to 36 damage (mean 27) |

Giant, default 5 (wander), hero standing 15 units away in front: noticed on the first cycle tick with chance 99 - 79 x 10 / 65 = 87 percent (roll B below 86.8): state 2, look 16 begins, the second check on that tick passes with the same chance, then 18 (37 ticks) and agro 37 ticks after that pass. First choice two ticks after agro: slot 2 (charge, 49 percent a tick) or slot 0 (the 25-damage bolt, 38 percent). A motionless hero takes 37 to 56 damage (before armour) in the 520 ticks after the Giant's first choice, from about three attacks.

#### Open questions

1. Weapon contact geometry. The Slorg's bite uses weapon 3 (radius 5.0, height 3.0) and the charge weapon 1 (radius 2.0, height 4.0); the sequencer only starts a bite at d at most 3.0 (Giant 5.0). Resolved by the general rule (SPEC-0064, Shared field behaviour: Weapon contact and the hero's hurt shape); the distances are in the Weapon reach table; stand-in only for a port without attachment frames: the bite hits a level hero from 6.2 behind to 6.4 ahead (Giant 6.4 behind to 7.2 ahead) of the enemy's centre; the charge hits one from 3.4 behind to 3.8 ahead on a tick of the run.
2. Whether a bolt that leaves the shooter outside the camera cone scatters by exactly 5.0. Resolved (High; proactive (open close-out)): yes; the aim offset is a draw from 0 to the row's inaccuracy (0 here) plus 5.0, in a random horizontal direction (SPEC-0038 Behavior 43), so it is exactly 5.0.
3. The sentry shuffle of the Slorg (ids 6, 7, 8 missing). Resolved (High; proactive (open close-out)): a missing id plays clip 0 of the main set (the lookup-miss rule, SPEC-0040 Behavior 3D), which is the walk clip, so the Slorg's stand and look play the walk clip with its root travel.
4. Resolved: the charge strike test is the ordinary weapon test (weapon 1 against the hero's capsules, SPEC-0038 Behavior 13); a level hero is touched from 3.4 behind to 3.8 ahead of the Slorg's centre on a run tick (Weapon reach table), which gives the connection rule under Charge.
5. First-tick offset of the missile spawn window. Resolved (Medium-High; proactive (open close-out)): s + 32. An event on key k of a clip with speed byte 2 at play rate 1.0 is first seen on tick s + 4k, because the clip starts in the animation pass of the request tick s; key 8 gives s + 32.



## Flyers

Five kinds make up the batch: the Biting Spellbook (monster type 60, variant 0, row 134), the Shooting Spellbook (type 60, variant 1, row 135), the Crokabek (type 61, row 136), the Mountain Crokabek (type 62, row 137) and the Swamp Crokabek (type 63, row 138). All are class-17 common enemies and follow SPEC-0038 (think loop, pre-step, attack choosers, sequencer, attack manager). All five have Can Fly 1, approach routine 4 (strafe) and Display in default 1. The batch is about flight: how a flyer holds its height, which animations and speeds it uses, how it attacks from the air, when it comes down, and how the hero reaches it. Confidence: High for the row values, the clip tables, the height rule and its gates, the grounding by a hit and by death, the charge arithmetic and the navmesh kind; Medium-High for the start-state behaviours (they reuse the shared field rules); Medium for weapon contact geometry and for the exact tick offsets of the charge attacks; Low for the two placements that have no flying navmesh. All times are ticks (60 per second). "s" is the tick on which the sequencer first requests a clip of an attack, "c" is the tick of an attack choice, and a one-shot clip of F keys at speed byte 2 and play rate 1.0 passes its 0.99 gate 4(F - 1) + 1 ticks after its request, one tick earlier from F = 25 keys (speed byte 1: 2(F - 1) + 1, one tick earlier from F = 50 keys; SPEC-0005).

### Shared rules for the five flyers

#### What they have in common

- **Visible kinds.** Every row has Display in default 1, so none of the hidden-minion rules apply: no state 21, no decloak clip, no state 23 on the way home. A flyer is drawn and hittable in every state except the dodge sub-states (SPEC-0036, Hittable predicate) and, for one Crokabek placement, the hide pose of state 8 (see Start states). State 2 is the ordinary alerted search of SPEC-0038 Behavior 5.
- **Element and status.** Target type T is 5 on all five rows. No melee slot of the batch carries a hero status. The only status in the batch is the curse of the Swamp Crokabek's missile slot, which can never fire (see that kind).
- **Notice values (identical on all five).** View cosine 0.707 (45 degrees half-angle), awareness 50 (roll A: int[1, 100] at most 50), awareness timer 30 (a check can pass only on one tick in every 32), near-edge chance 80 at the near range and far-edge chance 10 at the far range, no always-has-line-of-sight. Near range 10 on all rows; far range 40 for the Spellbooks and 60 for the Crokabeks. The chance between the edges is 80 - 70 (d - 10) / (far - 10) over the unscaled band; both band edges are scaled by F1 of the hero's action state (0.7 standing, 0.75 slow walk, 1.5 walking, 1.75 running, 1.0 otherwise). Inside the scaled near edge the check passes after roll A alone (50 percent per check). The chance is halved when the hero is outside the view cone. Examples for a standing hero: Crokabek at 20 units chance 66, at 30 units 52, at 40 units 38 (far edge 42); Spellbook at 20 units chance 57, at 28 units 38 (far edge 28). Roll B must be strictly below the chance, so a chance of 66 passes on 65 of 100 draws.
- **Search state.** Crokabek rows search for 120 ticks (search timer 120 to 120), Spellbook rows for 0 ticks.
- **Weights in the attack manager** (SPEC-0038 Behavior 45): Crokabek 0, Swamp Crokabek 0, Mountain Crokabek 5, both Spellbooks 5. All five are approach 4, so each can be held back into the wait in line (outer 12) when the manager says wait; the stand-off radius (row +0xB0) is 10 for the Crokabeks and 15 for the Spellbooks. At starting gear (capacity 0) the first listed attacker of any group never waits and every later entry waits when the weights of the entries before it sum to more than 0: weight-0 Crokabeks never make the ones behind them wait, one weight-5 flyer does.
- **Socialize.** Row +0x40 is 0 on all five rows, so none of them chats (SPEC-0038 Behavior 10).
- **Drops and death.** The death hook pays the placement's drop ordinal (record offset 0xB0; -1 on the placements that carry none) with the rules of SPEC-0038 Behavior 34. Death disappear timer: 1 tick on all three Crokabek rows, 10 on both Spellbook rows. None of the rows steals Neopoints; the plain Crokabek steals an item and then carries its price (see that kind).
- **Hurt shapes** (bone capsules of the model, SPEC-0040 Behavior 12A): Spellbooks one capsule on bone 0 (length 1.83, radius 0.73); Crokabek a body capsule (length 1.34, radius 0.48) and two wing capsules (length 1.31, radius 0.18); Mountain Crokabek bone 0 (length 2.39, radius 0.86), body (1.36, 0.56) and wings (2.33, 0.33); Swamp Crokabek bone 0 (1.83, 0.66), body (1.04, 0.43) and wings (1.79, 0.25). Body size (row +0x28, +0x2C): Spellbooks radius 2.0, height 1.2; Crokabek 1.0 by 1.5; Mountain 2.0 by 2.0; Swamp 1.0 by 2.0. The row radius is used only for the chase stop and spacing; the row cylinder (feet at the position, up by the row height) is what Mote power objects and enemy projectiles test (Medium).

| Row item | 134 Biting Spellbook | 135 Shooting Spellbook | 136 Crokabek | 137 Mountain Crokabek | 138 Swamp Crokabek |
|---|---|---|---|---|---|
| Hit points | 35 | 35 | 25 | 55 | 35 |
| Flying idle / agro velocity (+0x8C, +0x90) | 1.0 / 3.0 | 1.0 / 3.0 | 3.0 / 8.0 | 3.0 / 6.0 | 3.0 / 8.0 |
| Max fly height (+0xB4) | 20 | 20 | 30 | 100 | 20 |
| Flying hit value (+0xAC) | 0 (stagger) | 0 (stagger) | 1 (knock down) | 1 (knock down) | 1 (knock down) |
| Ground timer, grounded for (+0x9C to +0x98) | 600 to 900 (never grounded alive) | 600 to 900 (never) | 300 to 900 | 600 to 900 | 300 to 900 |
| Turn weights ground / flying (+0x84, +0x88) | 1.0 / 1.0 | 1.0 / 1.0 | 0.5 / 0.5 | 0.5 / 0.5 | 0.5 / 0.5 |
| Strafe band (+0x4F4 to +0x4F8) | 5 to 16 | 3 to 15 | 3 to 15 | 3 to 15 | 10 to 30 |
| Desired strafe distance (redrawn after each normal choice) | 5 + 11 f: 7.2 to 13.8 | 5.4 to 12.6 | 5.4 to 12.6 | 5.4 to 12.6 | 10 + 20 f: 14 to 26 |
| Backbiter scale (+0x500) | 0.0 | 1.0 | 1.0 | 1.0 | 1.0 |
| Recovery after an attack | int[30, 60] | int[30, 60] | int[30, 60] | int[30, 60] | int[30, 60] |
| Missile defend time / dodge / block (+0xE0, +0xE4, +0xE8) | 180 / 30 / 0 | 180 / 30 / 0 | 0 / 50 / 0 | 0 / 100 / 0 | 180 / 75 / 0 |
| Melee block quick, normal, heavy | 0, 0, 0 | 0, 0, 0 | 0, 0, 0 | 0, 0, 0 | 0, 0, 0 |
| Melee dodge quick, normal, heavy | 30, 30, 30 | 30, 30, 30 | 50, 30, 20 | 75, 50, 30 | 100, 30, 25 |
| Defence timer after a defence (+0x104 to +0x108) | 30 to 60 | 30 to 60 | 60 to 180 | 60 to 180 | 60 to 180 |
| Attack-manager weight | 5 | 5 | 0 | 5 | 0 |
| Death disappear timer | 10 | 10 | 1 | 1 | 1 |
| Blocking angle, damage reduction | 0, 0 | 0, 0 | 0, 0 | 0, 0 | 0, 0 |

(The percent chances are rolls strictly below the value, so 30 passes on 29 of 100 draws and 100 on 99 of 100 for the block and dodge rolls; SPEC-0038 Behavior 44.)

#### Where they stand: default states and counts

Counts are placements the player can meet (confirmed or plausibly loaded), without the CUT test block of pack 1259; a Biting Spellbook block of 18 placements (region 1331 of the Library) has no loader in any module and is listed apart.

| Kind | State 0 | State 5 (wander) | State 6 (patrol) | State 7 (sentry) | State 8 (hover, wake) | State 9 (follow a named character) |
|---|---|---|---|---|---|---|
| Biting Spellbook | 0 | 5 | 0 | 0 | 8 (+ 18 in the unloaded block) | 0 |
| Shooting Spellbook | 1 | 0 | 0 | 0 | 0 | 0 |
| Crokabek | 0 | 6 | 15 | 9 | 18 | 0 |
| Mountain Crokabek | 0 | 1 | 16 | 0 | 0 | 3 |
| Swamp Crokabek | 0 | 2 | 0 | 0 | 12 | 0 |

Main places: Biting Spellbook: Fyora's Library (5 wanderers in region 1330), Rathbone Family Crypts (8 hover-and-wake books, embedded in the pack). Shooting Spellbook: one, in Fyora's Library (region 1328). Crokabek: Meridell Plains (3 wander, 8 patrol, 13 wake), Meridell Castle Courtyard (9 sentries, 1 wake), Drackon Ridge (2 wander, 4 patrol), Brightvale Coast (1 patrol, 2 wake), Farm Road (1 wander, 2 wake), Meridell Outskirts (2 patrol). Mountain Crokabek: Cogham Steppes (15 patrols), Brightvale Outskirts (2 follow), Steppe Plateau (1 wander, 1 patrol), Crossroads (1 follow). Swamp Crokabek: Bogshot Swamp (2 wander, 7 wake), Werelupe Woods (5 wake). Alarm group (record byte 0xB7): 255 (none) everywhere except two Brightvale Coast Crokabeks (group 9), one Crossroads Mountain Crokabek (group 10) and four Library-block Spellbooks (group 1). Drop ordinals (record offset 0xB0 not -1): Biting Spellbook 31 of 31, Crokabek 30 of 48, Mountain 18 of 20, Swamp 14 of 14, Shooting Spellbook 0 of 1.

Height of the placed Crokabeks above the ground under them (the spawn point against the ground probe, in units; the hero-relative height rule lifts a flyer to 0.2 x its distance from the hero, up to its max fly height, even while it is idle): Meridell Castle Courtyard sentries 0.1 to 3.3; Farm Road 1.9 to 3.2; Brightvale Coast 3.0 to 41.5; Brightvale Outskirts 10.9 to 11.7; Meridell Outskirts 12.0 to 18.1; Werelupe Woods 11.8 to 25.0; Drackon Ridge 15.7 to 60.8; Steppe Plateau 20.9 to 28.9; Meridell Plains 26.9 to 43.6; Bogshot Swamp -0.1 to 35.2; Cogham Steppes 47.6 to 56.2; Crossroads 126. No area name on the disc contains the word "view"; the fields with the most unusual terrain for a Crokabek are Steppe Plateau (no flying mesh), the Castle Courtyard (three sentries outside every mesh), Bogshot Swamp (flying mesh far larger than the ground mesh, over water) and Cogham Steppes (patrols 48 to 56 units above the ground; 37 flying triangles lie below the ground).

Spawn record values that drive the field states (SPEC-0006 kind 3):

| State | Record values | On disc for this batch |
|---|---|---|
| 5 | float +0xBC (side R of the wander square, 0 = the whole navmesh); ints +0xC0, +0xC4 (rest range); int +0xC8 (rest percent) | Biting Spellbook R 50, no rest. Crokabek R 50 (3), 100 (2), 0 (1), no rest. Mountain R 0. Swamp R 85, rest int[0, 20] at 25 percent (2) |
| 6 | s16 +0xB8 (path group); int +0xBC (0 loop, else ping-pong); ints +0xC4, +0xC8 (rest range at each point); int +0xCC (mid-path rest percent) | All loops (+0xBC = 0), no rest at the points, no mid-path rest. Crokabek groups 0, 1, 2, 3, 4, 10, 11, 14, 15; Mountain groups 0, 1, 5, 6, 8, 9, 19, 34, 35, 37, 38, 42 |
| 7 | int +0xC4 (0 fidget at home, else keep the placed facing) | 0 on all nine Crokabek sentries |
| 8 | float +0xC0 (wake distance); int +0xC4 (1: no alarm stinger, and visible hover); int +0xC8 (hide pick selector) | Biting: 12 in the unloaded Library block (selector 0 on 15, 1 on 3), 8 with selector 2 at Rathbone (8). Crokabek: 20 (13), 5 (3), 50 (2); selector 1 on 3 records; one Courtyard Crokabek has +0xC4 = 0. Swamp: 10 (7), 12 (5) |
| 9 | int +0xBC (the named character's list id); float +0xC0 (follow distance); int +0xC4 (1) | Mountain: id 0 at 10.0 (2), id 8 at 30.0 (1) |

#### Start states

The think loop starts a record whose default word is 8 in state 8, any other word through the setter with 0 (SPEC-0038 Behavior 1). With Display in default 1 the setter's default case gives state 0 and the pre-step of state 0 runs the setter with the default word on the next tick, so a flyer reaches its default state one tick after any exit from agro: there is no state 23 and no clip 81. The first navmesh pick at creation takes the first flying (kind 2) mesh that holds the spawn point and sets the goal point to the spawn position, height included (see Flight).

- **State 0 (one Shooting Spellbook).** The setter runs the default word 0 every tick, so the book stays in state 0 for ever: it plays the idle pick (97 on the Spellbook table, clip 0, a 9-key hover) and hovers at its spawn height. Notice is tested only in states 3, 5, 6, 7 and 9, and the record has no alarm group, so the book never wakes by itself. It fights only after a hit (the take-hit hook sends it to agro with the damage doubled), and no level script hands it to agro because the Library's module makes no such call (High).
- **State 5 (wander).** The goal rule of the shared field model: with R below 0.1 a random triangle centroid of the current (flying) navmesh, otherwise a point in the square of side R about the home point, retried up to 10 times halfway toward the enemy, then home. It flies toward the goal with the wander gait (id 98) at the idle row speed while the rest timer is below 1, and every rest roll is skipped when the percent is 0. Arrival is within 0.5 horizontal, a blocked counter of 120, or no route. The rest fidget is animation 3, which is not a flyer id: the Crokabek tables have it (clip 15, 71 keys, speed byte 1, gate 140), the Spellbook tables do not, and the fidget counter only counts down when the table has the id. Only the two Swamp wanderers rest at all (int[0, 20] ticks at 25 percent), so the fidget is a rarity.
- **State 6 (patrol).** The locator group named by +0xB8 is flown in loop order with gait id 99; there is no rest at the points (+0xC4 = +0xC8 = 0) and no mid-path rest (+0xCC = 0), so the flyer circuits at the idle speed without stopping. Spellbooks have no patrol records.
- **State 7 (sentry).** Fly home when farther than 0.5 from the home point, otherwise stand: animation 144 (clip 16, a 10-key hover) with fidget 145 (also clip 16) every int[60, 600] ticks, which is visually the same hover. Because no walk target is set while standing, a sentry keeps the height it had when it last set one: its spawn height, or the hero-relative height of its last homing flight (see Flight).
- **State 8 (hover, wake).** The flyer hovers at home playing the hide pick (id 9 on every table; selector 1 gives 128, selector 2 gives 129; Crokabek: clip 5 for all of them, a 10-key hover; Biting Spellbook: id 9 is clip 5, a 2-key pose, id 128 is clip 17 and id 129 is clip 18, one-key poses). While the emerge sub-state is 0 the height goal is the home point's height (so a flyer returning to state 8 climbs or sinks back to its spawn height). The wake rule is SPEC-0038 Behavior 40: sub-state 0 tests the horizontal distance against the record's wake distance times F1 of the hero's action state (0.7 standing, so 20.0 wakes at 14.0; 1.5 walking: 30.0; 1.75 running: 35.0); on the tick after that test sub-state 1 requests the flyer emerge animation 165. **None of the five tables has a clip for id 165, so sub-state 1 enters agro in that same tick** (the emerge is skipped; there is no emerge clip and no vulnerable window beyond the doubled first hit). The wake happens one tick after the distance test; the agro machine first runs on the following tick (outer 0 sets the strafe archetype), and the first attack choice is possible the tick after. Except for one Courtyard Crokabek the flyer is not hidden while it hovers: the visibility rule for a row with Display in default 1 hides it only when the record's +0xC4 is 0 and the hide pose is playing, and +0xC4 is 1 on every other record. The visible hovering flyer is hittable; a hit in state 8 sends it to agro with the damage doubled (SPEC-0038 Behavior 37). A distance of 0 never wakes; a group alarm wakes a state-8 member into agro whatever its distance.
- **State 9 (three Mountain Crokabeks).** Each tick the record's named character (list id in +0xBC) is looked up. When it is missing the flyer runs the wander state (state 5 handler). Otherwise it takes over that character's locomotion mode and flies toward it while it is at least the follow distance away (10.0 for the two in Brightvale Outskirts, 30.0 for the Crossroads one). Inside the follow distance, with +0xC4 = 1 (all three records), it keeps flying toward the character until it is closer than 6.0, and there plays animation 34 (no clip on this table, so clip 0 plays) while a 60-tick timer runs. The row speed applies as in any flight, so the flyer trails its character at the idle speed (6 units per second). The notice test of states 3 and 9 applies every tick (SPEC-0038 Behavior 2), so the usual state-2 chain starts when the hero is noticed. The ids are looked up by comparing with the id of each bound character of the world, the embedded block first and then the slot regions in order, and the first match wins; ids are record ordinals within a region (the Cogham Village followers show this). Read that way, the two Brightvale Outskirts flyers name record 0 of their own region, an unnamed Whinny, and the Crossroads flyer names record 8 of its region, an unnamed Mortog; the hero is not what they follow (Medium-High; needs a runtime check only for whether an earlier slot region supplies a character with the same id first).

**Returning to the default state** (leash, lost-sight search, victory pose, flyer re-launch): setter 0, then the default word one tick later. A state-8 flyer therefore flies home at the idle speed and hovers there again, hittable, with the wake rule live; a wanderer or patroller resumes its goals.

#### Noticing the hero (states 5, 6, 7, 9, 3 and 0 excluded)

The pre-step notice check passes on one tick per 32; a pass calls the setter with 2. With Display in default 1 this is the plain alerted search: the search countdown (Crokabek 120, Spellbook 0) is stored, the hero's position becomes the last-seen point, and the state-2 handler runs in the same tick:

1. If neither the search clip nor the lose-interest clip plays, request the look animation 147 (flyer form of 16). Crokabeks: clip 16, a 10-key hover loop. Spellbooks: no clip for 147 or 16, so the idle pick is requested.
2. Aim at the last-seen point with the row's flying turn weight.
3. Run the notice check again (new rolls, same tick; the awareness countdown is still 0). A pass requests the search animation 149 (flyer form of 18): Crokabek clip 22 (18 keys, one-shot, gate 69), and when it passes its gate the enemy enters agro. The Spellbook tables have no clip for 149, so a pass enters agro at once in that tick.
4. A failed repeat counts the search countdown down by 1. At or below 0 (at once for a Spellbook) the enemy requests the lose-interest animation 148 (Crokabek clip 16, gate 39; Spellbook: no clip, so it leaves at once) and, when that finishes, runs the setter with 0. The check repeats on every tick on which the awareness countdown is 0, so a Crokabek that fails its second roll has three more chances (ticks 32, 64 and 96 later) before the 120-tick countdown ends, and a pass at any of them starts clip 22.

So a Crokabek that is noticed by the first roll enters agro with probability p on the same tick, where p is the chance of one more successful check: p = 0.5 x (chance - 1) / 100 for a hero beyond the near edge (0.5 inside it). Where the second roll passes, the enemy fights 69 ticks later (Spellbook: at once). The group alarm (SPEC-0038 Behavior 3) sends non-state-8 members to state 2 and plays 18 / 149; a Crokabek of group 9 or 10 does this to its group.

#### Flight

This is the part a port gets most wrong. All five rows have Can Fly 1: the flyer rules apply while the retarget countdown (+0xBF4) is 0 or below; a positive countdown means the enemy is on the ground and moves as an ordinary ground enemy. A flyer starts airborne (the creation value is -1) and comes down only when it is hit (Crokabek rows) or dies.

**Navmesh kind.** A row with Can Fly not 0 picks only flying (kind 2) navmeshes: the first kind-2 mesh that holds the enemy's position becomes its current mesh (SPEC-0018 Mesh kind by query mode; SPEC-0038 Behavior 6 should read "kind 2 for a flyer, kind 1 for a ground enemy", not "either"). Flying meshes are separate authored data and differ from the ground mesh of the same area: for example Bogshot Swamp has 4.5 times more flying than ground area and only 13 percent of its flying triangles lie over the ground mesh; Meridell Plains has 1.9 times, Cogham Steppes 2.2 times, Drackon Ridge 1.3 times; Fyora's Library has the same area for both; Rathbone Family Crypts and the Meridell Castle Courtyard have a much smaller flying mesh than ground mesh (0.3 and 0.1 times). A flyer is confined to its flying mesh (the border edges of the current triangle act as walls, SPEC-0040 Behavior 14A), not to the ground mesh. So a flyer flies over swamp water, ledges and gaps where the flying mesh extends, and cannot leave the flying mesh even where the ground goes on. Route finding for the route flag and for the wait-in-line stand-off point uses the same flying mesh; a hero standing outside it gives no route (see Leaving agro).

**Height keeper (every tick, before the motion step; SPEC-0040 Behavior 14D item 7).** Each tick: dy = target height - y. If |dy| > 0.1, y moves 0.1 toward the target (6 units per second up or down); otherwise y is set to the target exactly. The keeper writes the position directly and applies only while the enemy is alive and the flying test holds; the record's no-gravity flag is set every tick while the flying test holds, so there is no fall and no ground projection of the step.

**Target height** (the goal height, computed when a walk target is set, which is every steering tick of a moving sub-state, not every tick): goal y = hero y + min(max fly height, 0.2 x the horizontal distance from the enemy to the hero), raised to at least (ground height under the goal point + 2.0), where the ground probe is the ordinary one started 8.0 above the goal point. While the emerge sub-state is 0 (state 8 waiting) the goal y is the home point's height instead. A hold sub-state sets no walk target, so the goal keeps its last value: a flyer that holds in strafe keeps the height it last steered to. Two exceptions: (a) in the hit-reaction sub-state of kind 6 (the Spellbooks) the goal is not recomputed and is only capped at the max fly height; (b) in sequencer states 7, 8, 9 and 12 (charge run, charge hit, blocked charge and charge end) the target is the hero's y + 2.0 whatever the goal is. The goal is initialised to the enemy's own position whenever its navmesh index first becomes valid, so a sentry or a waiting flyer starts at its spawn height.

Worked heights. Hero standing at y = 0 on flat ground: a Crokabek 10 units away hovers at y = 2.0, at 12 units y = 2.4, at 30 units y = 6.0 (cap 30 is reached at 150 units); a Spellbook at 12 units y = 2.4 and never higher than 20; a Swamp Crokabek in its band (14 to 26) hovers 2.8 to 5.2 above the hero's feet. In a charge it moves to hero y + 2.0 at 6 units per second. The keeper is not tied to the terrain under the enemy, only to the hero's height and the floor at the goal point, so over a pit or water it keeps the hero-relative height; over higher ground than the hero's (a ledge above the hero) the floor term lifts it to ledge height + 2.0.

**Row speed.** On top of the root travel of the clip it plays, a flyer moves along its movement axis at v / 30 per tick (2 v units per second), where v is the flying idle velocity (+0x8C) outside agro and the flying agro velocity (+0x90) in agro. The row speed is added only while the animation being played is the slot animation of the current locomotion mode (modes 7 forward, 8 back, 9 and 10 sideways, with ids from the flyer set) or, in sequencer states 7, 8, 9 and 12, whatever the animation is, always straight along the facing. It is not added in holds, dodges, hit reactions, the search or the attack states 0 to 5. Speeds: Crokabek and Swamp Crokabek 6 units per second idle and 16 in agro; Mountain Crokabek 6 and 12; Spellbooks 2 and 6.

**Gait modes and clips.** The flyer set is written when the archetype is entered (strafe: ids 103 to 106) and when the idle setter runs (wander 98, patrol 99, named-target 100, idle state 154). In the strafe archetype: gait 0 (fly toward the hero) plays 103, gait 3 (back off) plays 104, the two side gaits play 105 and 106. Crokabeks: 103 clip 1 (16 keys, loop), 104 clip 18 (8 keys), 105 clip 25 (10 keys), 106 clip 17 (10 keys), all with no root travel; hold 132 clip 23 (14 keys). Spellbooks: 103 clip 0 (9-key hover), 104 clip 21, 105 clip 9, 106 clip 8, hold 132 clip 0; the wander and patrol ids 98, 99, 100 are missing on the Spellbook tables, so a wandering book plays clip 0 (the 9-key hover; the request is still the gait id, so the row speed still applies). Because the clips carry no travel, the flight speed of a flyer in the strafe archetype is exactly the row speed.

**Turning.** Flyers turn by the steering blend (SPEC-0038 Behavior 48). In the hold sub-states they aim at the hero with the row's flying turn weight: 0.5 for the Crokabeks, 1.0 for the Spellbooks (an instant turn).

**Landing.** A can-fly-1 flyer lands in exactly two ways.
1. **Hit reaction kind 7 (flying hit value 1: the three Crokabek rows).** On the tick that the reaction requests animation 76 the flying test grounds the enemy: the retarget countdown becomes int(float[ground timer min, ground timer max)), the navmesh is picked again (same rule) and the keeper stops. Gravity (128 units per second squared) pulls it down. The reaction waits until the motion record's ground gap (height above the ground under it, not the speed) is at most 2.0, then plays 74 to its end when the table has it, then the outer sub-state returns to 0. Every Crokabek table has both 76 and 74 and both name the same one-shot clip 13 (56 keys, speed byte 1, gate s + 110, root travel backward 3.64 units per second, about 6.8 units over the clip); the change from id 76 to id 74 does not restart the clip (a request for a clip that the main layer already holds starts nothing), so the slide lasts about 110 ticks counted from the tick that requested 76, and longer only while the ground gap stays above 2.0. The character is not hittable for as long as animation 76, 74 or 75 is the current animation (see Knock-down phases below). A flyer more than 2 units above the ground falls at 0.0178 n squared units after n ticks (4 units in 15 ticks, 10 in 24).
2. **Death.** The death hook draws the same countdown and the corpse falls and (Spellbooks, timer 10) sinks 1/180 unit per tick once the clip passes 0.95.

**Grounded fight.** With the countdown positive, the flying test fails: the keeper and the row speed stop, the ground gait sets apply, the slots with trigger special 1 (airborne only) are blocked and the slots with special 2 (grounded only) open. For the Crokabeks that is the whole point of the knock-down: slot 1 is their ground attack. Further hits while grounded give hit reaction kind 0 (animation 57, Crokabek clip 33: 16 keys, gate 61, root 6.66 units per second backward, about 7.1 units) or kind 5 (animation 127, absent: clip 0) and do not extend the countdown. The countdown falls by 1 per tick in the pre-step. When it reaches 0 the flying test passes again (the keeper restarts that tick) and, unless the agro machine is in a block (outer 2 with sequencer state 0 to 3, or the attack-active latch), the pre-step picks the navmesh again and writes state 22 directly: animation 75 (Crokabek clip 24, 5 keys, gate 17 ticks, forward root 1.04 units per second) then the setter with 0. **A re-launching flyer leaves agro**: after the clip it is in its default state and must notice the hero again (state 22 does not go back to state 19). If the countdown reaches 0 during the blocked window it stays 0 (the flyer is airborne again) and the pick and the state 22 write happen on the first later tick that passes the gate.

**Over drops, water and ground the flyer cannot use.**
- The flyer's horizontal range is its flying mesh, not the ground mesh. Where the flying mesh covers a pit, a river or swamp water the flyer crosses it freely at its hover height, which does not depend on what is under it except through the floor term (ground height under the goal point + 2.0; when the probe finds no ground the floor does not apply and the hero-relative height rules). A hero who crosses the same gap on a bridge or by a jump is followed along the hero-relative height.
- Where the flying mesh ends, its border edges are walls: a flyer pursuing a hero on a ledge, an island or across a gap outside its mesh flies to the border and stops there (the blocked flag ends a circling step and sets the strafe lock to 120).
- A hero outside the flying mesh gives no route. The route flag is then 0: the archetype steers straight at the hero, the leash counter runs while the walk waypoint is at least 15 units from the hero and the flyer leaves agro on the 181st tick; the charge slots (Crokabek slots 0 and 2, Biting slot 0) are not offered (they need a route), the non-charge melee slots need the hero within 10 units, and the Shooting Spellbook's missile slot is unaffected.
- A flyer that is knocked down (Crokabek rows) over a gap falls under gravity and the ground probe decides where it comes to rest; the grounded countdown runs meanwhile and the knock-down reaction waits for the 2.0 ground gap. A killed flyer's drop spawns at its position at the moment of death, in the air. No water-specific or void-specific rule exists for the flyers; the shared ground and hazard rules (SPEC-0018, SPEC-0057) apply to whatever it lands on (Medium-Low).

#### Knock-down phases, hittability and the ways out (Crokabek rows)

R is the tick on which the hit-reaction handler first runs after the hit (the same tick as the hook, or the next). c is the grounded countdown drawn on R: a whole number from [300, 900) for the Crokabek and the Swamp Crokabek, [600, 900) for the Mountain Crokabek. Confidence: High for the order, the gates and the hittability; Medium for the exact first tick of the handler relative to the hit.

| Phase | Ticks | Current animation | Hittable | Movement | Ends when |
|---|---|---|---|---|---|
| Fall | R until the ground gap is at most 2.0 | 76 (clip 13) | No | Gravity only; the keeper and the row speed are off; the root step of the clip is reversed while the gap is above 2.0 (edge guard, below) | The ground gap (height above the ground under the body, 0 while its floor flag is set) is at most 2.0. A Crokabek hovering at the floor height (2.0 above the ground, the usual height within 10 units of a hero on flat ground) passes on the next tick |
| Slide | until clip 13 passes 0.99 of its length, 110 ticks after R | 74 (same clip, not restarted) | No | Clip 13 root travel backward at 3.64 units per second (about 6.8 units in all); gravity | Clip 13 finished. If it already is (the fall took 110 ticks or more) the next tick. Then outer sub-state 0 and the ground gait set |
| Grounded fight | R + 111 to R + c | ground hold and gaits; 57 after a further hit | Yes, except in the dodge sub-states | Ground gaits, no row speed, slot 1 (peck) only | The countdown reaches 0 (value c - j on tick R + j) |
| Re-launch | state 22 is written on R + c + 1; its handler runs from R + c + 2 | 75 (clip 24, 17 ticks) | No | The keeper restarts and lifts the body at 6 units per second toward the last goal height | Clip finished; setter 0; the default state follows one tick later |

- **The way out is independent of the reaction.** The countdown is stepped in the pre-step on every tick of every state. At 0 the pre-step picks the flying mesh again and writes state 22 directly, unless the Action-19 gate blocks (class state 19 with outer sub-state 2 and sequencer state 0 to 3; the knock-down is outer 10 and never blocks). While blocked the countdown stays at 0, the flying test passes (the body is airborne again) and the write happens on the first later tick that passes. So the longest a knock-down can hold a Crokabek is about c + 20 ticks (5 to 15 seconds), after which it re-launches into its default state.
- **A further hit while grounded** gives hit-reaction kind 0 (animation 57, clip 33, 61 ticks), never kind 7, because the kind-7 choice needs a countdown below 1; the countdown is not redrawn. A hit during the fall, the slide or the re-launch cannot happen (not hittable). A hit on a Crokabek that is airborne again but still in agro (countdown 0 held by the gate) is kind 7 again and draws a new countdown.
- **Ground that never gives a gap of 2.0 or less.** Over a pit the body falls until the ground probe's height meets it (a fall of n ticks covers 0.0178 n (n + 1) units: 50 units take about 53 ticks, 126 units about 84). Where the probe finds no ground cell at all it returns height 0.0 and the body lands on that plane. Where a cell has entries but none is sampled at the point, the probe returns its miss value of minus 1.0e9, the gap is about 1.0e9, the condition never holds, the body falls with growing speed for the rest of the countdown and, after the re-launch, climbs back at 0.1 per tick (Medium). On the flying meshes of the Crokabek fields such points are few: sampled at the centre of every flying triangle, 16 of 221 in Bogshot Swamp, 5 of 32 at Brightvale Coast, 7 of 215 in Brightvale Outskirts (plus 8 empty cells), 3 of 236 on Farm Road, 3 of 256 at Crossroads, 3 of 290 in Cogham Steppes, 1 of 293 in Werelupe Woods, none in Drackon Ridge, the Castle Courtyard, Meridell Outskirts or Meridell Plains. Elsewhere the flying mesh lies 3 to 60 units above real ground (median: Bogshot 6.8, Brightvale Coast 4.9, Meridell Plains 3.7, Courtyard 7.6, Crossroads 7.4, Cogham 10.6, Meridell Outskirts 11.2, Drackon Ridge 20.2, Farm Road 31.3, Brightvale Outskirts 38.3, Werelupe Woods 64.1), so a knock-down falls 3 to 60 units before the slide.
- **Edge guard.** For a character that is not airborne in the flyer sense, a ground gap above 2.0 in size reverses that tick's root-motion step (the position becomes the tick's start position minus the step). It applies to a knocked-down Crokabek and not to an airborne one (the guard is skipped while either flying test holds), so hovering charges are not affected, while a grounded Crokabek cannot walk off an edge higher than 2.0 and is pushed back each tick instead.
- **What can wait, and what cannot.** A missing action id never waits: the recovery ids of slots 0 and 2 (37, 47), the wind-up of slot 1 (38) and the emerge (165) are skipped in the same tick. A present id waits for the finished flag of its own clip, which is false while that clip is not on one of the four animation layers. The attack sequencer states that wait on distance are 7 (stall exit: tick counter above 100 with speed at most 0.001), 8 of the swoop (exits on distance travelled at least the charge distance, or on the wall flag that a bump into a mesh border raises for 120 ticks) and 9 of the swoop (distance only; no stall and no wall exit, so a flyer pressed against its mesh border after a blocked swoop waits there until it is hit). Every one of these states is hittable. No other state of the Crokabek waits without a bound.
- **Frozen and unhittable in the original.** Only the fall wait: the body is not hittable (animation 76) and does not steer, for up to c ticks, then it re-launches. After a miss-value point it also falls out of reach. It cannot stay frozen for ever.

#### Agro in the air

The agro machine of SPEC-0038 Behavior 44 runs with the strafe archetype (approach 4) and the flyer words.

- **Entry.** Outer 0 sets the archetype: outer 4, inner 10, hold 0. The distances are drawn at creation and after every normal choice: desired = strafe lo + (hi - lo) f with f = int[20, 80] / 100.
- **Strafe (outer 4).** The route flag is tested each tick. With no route: inner 10 near the waypoint, otherwise inner 7. Inner 7 (approach): gait 0, steer to the hero, until d < desired (then inner 10). Inner 10 (hold): hold animation 132, aim at the hero; while the hold timer is positive the distance tests apply; when it ends r = int[1, 3]: 1 stays in hold, 2 goes to inner 8 (stay in hold if the strafe lock is positive), 3 to inner 9 (same lock rule), and the hold timer is int[60, 180]. Inners 8 and 9 circle: the side gait steers to own position +/- 2.0 along the sideways direction, the hold counts down and ends the circle below 1; a blocked flag ends it at once and sets the strafe lock to 120. If d is above the band's hi value the flyer approaches (inner 7); if d is below the band's lo value and the lock is below 1 it backs off (inner 11, gait 3, toward the point hero - dir x hi) until d is above the desired distance. So the flyer hovers 3 to 16 units (Swamp Crokabek 10 to 30) from the hero, mostly facing it, drifting sideways for 60 to 180 ticks at a time at the agro row speed, and backing off when the hero walks up to it.
- **Wait in line.** When the manager refuses an attack start and the approach is not 3, the enemy enters outer 12: gait set 2 (flyer id 98), stand-off point hero + (u - R/2, 0, v - R/2) with R = 10 (Crokabeks) or 15 (Spellbooks), idle pick while the wait counter is positive, victory-pose animation 163 once on arrival, then a new point (SPEC-0038 Behavior 45). The ids 98 and 163 are absent on the Spellbook tables (clip 0 plays; a missing 163 ends the pose at once).
- **Normal choice.** Slots are tested from the highest down; the first that passes its tests and its roll starts. For approach 4 the roll threshold is int(int(chance) x backbiter) when dot(enemy facing, hero facing) is at most 0, i.e. when the hero is not looking away from the enemy. The backbiter is 1.0 on four rows and 0.0 on the Biting Spellbook (see that kind). Trigger special 1 (airborne only) passes only while the enemy is a can-fly-1 flyer in the air; special 2 (grounded only) passes only while its retarget countdown is positive. A can-fly-2 row (Minion Wizard, Magician, Restless Spirit, Death Knight) never passes special 1 and always passes special 2. A charge slot (charge type not -1) also needs a route to the hero; a missile slot never needs one; a non-charge melee slot needs a route or d below 10.
- **Counters** (SPEC-0038 Behavior 14) apply only to the Spellbooks (see them).

#### Charge attacks of a flyer

Every Crokabek slot and the Biting Spellbook's first slot is a charge: the sequencer starts in state 6 (wind-up and aim), then state 7 (run) with the strike speed word as the play rate, state 8 (hit) with rate 1.0, state 12 (end). The flyer moves straight along its facing in states 7, 8, 9 and 12 at the agro row speed (v / 30 per tick) plus the root travel of the clip it plays; nothing steers during the run or the hit. Consequently:

- The run speed per tick is v / 30 + the clip's average root travel x the strike word. Crokabek slot 0: 0.267 + 0.292 = 0.559; slot 2: 0.267 + 0.365 = 0.632; Mountain slot 0: 0.200 + 0.365 = 0.565; Swamp slot 0: 0.267 + 0.438 = 0.705; Biting slot 0: 0.100 + 0.111 = 0.211. In state 8 the clip is a still hover or a pose, so the speed falls to the row speed alone (0.267, 0.200, 0.267, 0.100).
- Kind 2 (slot 0 of the Crokabeks, slot 0 of the Biting Spellbook): no strike test in the run. State 7 goes to state 8 when the horizontal distance to the hero is within the slot's melee range (10.0 for the Crokabeks, 4.25 for the Biting Spellbook) and to state 12 when the distance travelled from the start point reaches the charge distance (6.5 for the Crokabeks, float[20, 30) for the Biting Spellbook), and also to state 12 if a tick counter above 100 meets a speed of at most 0.001. State 8 requests the C clip on its first tick and tests the strike every tick after that until C finishes. The flyer keeps flying forward during C.
- Kind 1 (slot 2 of the plain Crokabek): the strike test runs every tick of the run (the B clip carries the weapon bit on every key). A hit goes to state 8 (play C while the distance travelled is below the charge distance float[6, 12), then state 12); a blocked strike goes to state 9 (blocked-charge kind 0: play D until the distance is reached); otherwise the run ends in state 12 when the distance travelled reaches the charge distance.
- The tick counter, the start point and the charge distance are set in state 6; the charge distance is drawn on the choice tick. The start point is rewritten on every tick of state 6, so it is the body's position on the tick the wind-up clip passes its gate and the run is requested; what the wind-up clip moved the body (root travel of clip 22 is 0.62 units per second at rate 1, 0.81 at the wind-up word 1.3, about 0.7 units over the 53-tick wind-up) is not part of the distance travelled, but it does bring the body that much closer to the hero before the run starts.
- During states 7 to 12 the keeper's height target is the hero's y + 2.0, so a swooping flyer comes down to the hero's chest height at 6 units per second.

#### Hit reactions and defence

- **Flying hit value 1 (Crokabeks):** kind 7 as above, grounding the flyer. **Flying hit value 0 (Spellbooks):** kind 6: the enemy stops (its walk target is set to its own position), animation 152 plays to its end (Spellbook clip 3: 10 keys, gate s + 37, root travel backward 10.41 units per second, about 6.4 units of recoil) and the outer sub-state returns to 0. The Spellbooks are never grounded by a hit; the goal height is frozen during the reaction.
- **Melee defence** is rolled once per hero swing under the SPEC-0038 Behavior 44 gates (hold below 1, hero within its reach and in front of the swing arc): block never passes (all block chances 0); the dodge roll passes below the row value for the swing's weight (table above). A dodge plays 140 to its end (Crokabek clip 18, an 8-key loop, gate s + 31, no root travel, so the flyer stays in place; Spellbook clip 7, 20 keys, gate s + 77, root travel backward 8.51 units per second, about 11 units). The enemy is not hittable during the dodge. After a defence the hold is int[defence timer min, max].
- **Missile defence:** for each live hero shot whose path from its position to its position + velocity x (missile defend time) meets the body capsule, one roll below the missile dodge percent starts a dodge: 138 or 139 (Crokabek clip 17, a 10-key loop, gate s + 39, no root; Spellbook clips 10 and 11, 16 keys, gate s + 61, sideways 5.46 units per second, about 5.5 units). With a missile defend time of 0 (Crokabek and Mountain Crokabek) the look-ahead path is a single point, so the dodge can only start when a shot is already inside the capsule at the enemy's check, which the shot's own hit test normally resolves first: treat those two kinds as never dodging shots (Medium).

#### Death

Death hook (SPEC-0038 Behavior 34): base state 16, the flyer is grounded (countdown drawn), the drop and payout run, all status timers clear. The death clip is 58 (the ground form, because the flying test now fails): Crokabeks clip 12 (2 keys, a still pose, 0.95 at s + 5), Spellbooks clip 4 (10 keys, 0.95 at s + 36). The corpse leaves after the death disappear countdown: Crokabeks about 7 ticks after death (timer 1), Spellbooks about 47 ticks (36 + 11). The body falls under gravity from its hover height (no keeper once dead) and sinks 1/180 unit per tick when its countdown is below the timer minus 30; the drop spawns at the enemy's position at the moment of death (in the air).

#### Hero actions against the flyers

| Hero action | Result |
|---|---|
| Melee swing (Tor, Roberta's swats) | Tested against the flyer's bone capsules with the ordinary swing cylinder (radius 1.0, length weapon reach + 2.0, SPEC-0036 Hit detection); height is not special-cased. A hovering flyer is reachable while it is within about 4 to 5 units horizontally and within about 2 units above the hero's hand level. Strafing flyers hover 2.0 to 2.5 units above the hero's feet at 5 to 12 units on flat ground (the 2.0 floor holds up to 10 units, 0.2 d beyond), so they are in reach mainly when they swoop or back off close; they are not hittable during a dodge |
| Roberta's shots | Straight shots that hit a bone capsule. A shot that crosses the flyer's capsule hits it; Spellbooks and the Swamp Crokabek may dodge (75 for the Swamp Crokabek, 30 for the Spellbooks, rolled again on every tick a shot is on course while the hold is below 1 and no hit reaction runs, so nearly every first shot is dodged); the other two do not in practice |
| Jump attacks | A hero in the air meets a flyer at its hover height without the ground-reach penalty |
| Mote power objects and cylinders | Test the row cylinder from the feet up by the row height (1.2 to 2.0), so a flyer hovering well above the hero's feet can be missed (Medium) |
| First hit on a Crokabek in the air | Reaction kind 7: it falls, plays clip 13 for about 110 ticks (not hittable the whole time, nor during the 17-tick re-launch), then fights on the ground for int[300, 900) ticks (Mountain 600 to 899, Swamp 300 to 899) and takes off |
| First hit on a Spellbook | Reaction kind 6: it staggers back about 6.4 units for 37 ticks and stays in the air |
| Hit on a flyer in state 8 or state 0 | Wakes it into agro, damage doubled, same reaction as above |
| Hero stands behind the Biting Spellbook (looking away) | It starts its normal attacks; facing it prevents them (see that kind) |
| Hero guards with L1 facing a flyer's melee swing | Blocks Crokabek and Spellbook swings (their slots are blockable) when the hero faces them (dot at most -0.5); nothing blocks a missile |

#### Edge cases shared by the five

- **No flying mesh.** Steppe Plateau (pack 1238) holds no kind-2 mesh. Its two Mountain Crokabeks have navmesh index -1: no goal point is initialised (the goal word is only filled by the pick), the wander goal rule has no mesh, and the leash cannot find a route. The navigation step does nothing for a character with no mesh (SPEC-0056), so such a flyer hovers at its placement height and never leaves state 5 or 6 (High); if noticed it fights from where it hovers (Default if unresolved (rewrite choice): direct steering toward the hero without a route).
- **Placements outside every mesh.** Three Courtyard Crokabeks and nine placements of the Library block (region 1331, which is never loaded, so those nine do not exist in play) lie in neither a ground nor a flying mesh. Resolved (High; proactive (open close-out)): the navmesh pick accepts only a mesh with a triangle that contains the position (SPEC-0029 Behavior 2), so such a placement keeps navigation index -1 exactly like the Steppe Plateau pair; it does not snap to the nearest triangle.
- **Hit-reaction kind 7 with the table check.** Both 76 and 74 exist on all three Crokabek tables; a table without 74 would end the reaction at the 76 tick.
- **State 2 with a missing clip.** A missing look animation (Spellbooks) requests the idle pick instead; a missing lose-interest animation ends the state at once.
- **The unused steal data.** Rows 137 and 138 carry a steal selector on their slot 2, but slot 2 is unused on both rows; only the plain Crokabek's slot 2 steals.
- **Per-tick order in agro** (SPEC-0038 Behavior 44): recovery countdown, block-state fix, missile defence, melee defence, lost-sight test, normal choice, counter choice, distance-hide gate, outer switch, agro tail (route flag, leash), then the class handler (hold timer, idle pick or hold animation, pursuit step). The pre-step runs the awareness countdown, the alert flag, the retarget countdown (grounded timer and re-launch), the alarm check and the map-search step before all of that.

#### Leaving agro

The leash: with no route to the hero and the walk waypoint at least 15 units from the hero, a counter runs and the enemy leaves agro on the 181st tick (SPEC-0038 Behavior 23). The route test uses the flyer's current flying mesh, so a hero who stands where the flying mesh does not reach (a ledge or an island outside it) is out of the flyer's reach. The lost-sight search (outer 9) walks to the last-seen point and gives up after 120 ticks. The victory pose (animation 163; Crokabek clip 28, 10 keys, gate 37; the Spellbooks have none and go to state 0) plays after a melee strike leaves the hero at 0 HP. The re-launch after a grounding also leaves agro.

#### What a port most likely gets wrong, shared by the five

1. Giving the flyers a ground navmesh. They take the first flying (kind 2) mesh; where the flying mesh extends over water and gaps they fly there, and where it stops they stop.
2. Letting a flyer fall or sit on the ground in normal play. It hovers at hero y + min(maxfly, 0.2 d), at least ground + 2.0, moves 6 units per second vertically and never lands unless it is hit (Crokabek rows) or dies.
3. Forgetting that the first hit grounds a Crokabek for 300 to 900 ticks, and that the grounded Crokabek has different attacks (slot 1 only). A re-launch sends it back to its default state, not to agro.
4. Giving the flyers an emerge clip. There is none (id 165 absent); a state-8 flyer enters agro on the tick after its wake test.
5. Hiding them. Display in default is 1: the hover is visible and hittable.
6. Treating the charge as a chase. The run and the C clip fly straight ahead at the row speed with no steering; most lunges end behind the hero.
7. Applying the minion hide rules (state 21, clip 82, state 23, clip 81) to these rows.

Causes that make a Crokabek "freeze when hit or when it attacks, and then stop taking damage and moving", most likely first:
8. A knock-down that never ends its fall. The reaction waits for the ground gap to be at most 2.0, and animation 76 (then 74) keeps the character unhittable and still while it waits. If the port keeps the hover keeper or the no-gravity flag after the countdown is drawn, measures the gap against the wrong surface (the ground navmesh, the flying mesh, the goal point's floor, or a speed), or turns a missing ground sample into a huge value, the gap never closes. Gravity must act as soon as the countdown is positive, and the gap is the height above the ground probe's result at the body's own position.
9. A countdown that does not run or a re-launch that does not fire. The countdown is stepped in the pre-step in every state (agro sub-state 10 included), the gate blocks only outer sub-state 2 with sequencer state 0 to 3, and the countdown becomes -1 on the tick state 22 is written (the flying test passes from then on). A port that steps the countdown only in idle states, or lets the knock-down block the gate, leaves the Crokabek grounded or in 74, 75 or 76 for ever.
10. An unhittable test that stays on. Hittability depends on the current animation id (74, 75, 76 are the only ones among these) and the dodge sub-states. If the next state requests no animation, or requests one the table lacks and the port keeps the old id as current, the Crokabek stays unhittable after the reaction. Only those three ids are special; a hit on a grounded Crokabek must produce kind 0 (animation 57), not kind 7 again.
11. Waiting on ids the table lacks. Recovery 37 (slot 0), recovery 47 (slot 2), wind-up 38 (slot 1) and emerge 165 are skipped in the same tick. Playing clip 0 (a looping walk on these tables) and waiting for it to finish stalls the end of the lunge and the swoop.
12. Charge bookkeeping. The start point is rewritten on every wind-up tick; the row speed (agro velocity / 30 per tick) is added in sequencer states 7, 8, 9 and 12 whatever clip plays; the swoop's state 8 and state 9 end only when the distance travelled from the start point reaches the charge distance (state 8 also on the wall flag). A port that stops the body when the strike lands, or that holds the keeper at the hit height without the forward row speed, never reaches the distance and hangs in state 8 or 9.
13. The edge guard. It must skip airborne flyers (a hovering charge keeps the clip's root travel) and act on a grounded Crokabek whose gap exceeds 2.0 (the step is reversed). Applying it to flyers makes the run clip's travel cancel and the lunge stall; omitting it lets a grounded Crokabek walk off ledges.
14. Mesh and route. A grounded Crokabek still uses the flying (kind 2) mesh, so its walk range is the flying mesh and a ledge higher than 2.0 stops it. The Steppe Plateau Crokabeks have no flying mesh at all and the three Courtyard sentries stand outside it; a port that needs a mesh to move or to pick a goal will freeze these (use the defaults under Edge cases).

### Crokabek

The Crokabek is the plains bird: a 25-hit-point flyer of Meridell, Brightvale and Drackon Ridge that wanders, patrols, stands sentry or hovers at a wake point. In agro it circles 5.4 to 12.6 units from the hero at 16 units per second and attacks with two straight dashes: a swoop that pecks for 2 and steals a potion, and a lunge for 10 that lands only when its solid body carries a hero who stands in its line. The first hit knocks it out of the air for 5 to 15 seconds, during which it walks and pecks; then it flies off and must notice the hero again. To beat it: sidestep during its 53-tick wind-up, then hit it once to ground it and finish it on the ground.

#### Identity and tunables

| Item | Value |
|---|---|
| Record | Monster type 61, variant 0, row 136, name "Crokabek"; one clip table for all 48 placements (a second, test-block-only copy in the CUT pack); 49 records in all, 48 playable: 6 wander, 15 patrol, 9 sentry, 18 wake |
| Class and approach | Class 17; approach 4 (strafe); Can Fly 1; hostile; Display in default 1 |
| Hit points, body | 25; radius 1.0, height 1.5, weight 1.0; attack-manager weight 0 |
| Flight | Speeds idle 3.0 / agro 8.0 (6 and 16 units per second); max fly height 30; grounded 300 to 900 ticks after a hit |
| Strafe, backbiter | Band 3 to 15, desired 5.4 to 12.6; backbiter 1.0; stand-off radius 10 |
| Turn | Hold aim weights 0.5 ground and flying |
| Defence | Missile: dodge 50, defend time 0 (never in practice); melee dodge 50, 30, 20 (quick, normal, heavy), block 0; defence timer int[60, 180] |
| Item steal | Slot 2: flag 0x04, selector 1 (items 150 and 149), chance 100 (100 of 101 draws), amount range 0 to 0 |
| Weapons | Weapon 3 (bit 0x4, the one every Crokabek clip uses): radius 4.0, height 4.0 (full), attachment 2; weapons 1, 2, 4, 5 radius 2.0, height 1.0 |

Attack slots (slot 3 unused; slots are tested 2, then 1, then 0):

| Slot | Kind | Distance band (strict) | Chance (passes on) | Facing arc | Melee range | Special | Charge | Delay | Damage | Counter |
|---|---|---|---|---|---|---|---|---|---|---|
| 2 | swoop and steal, charge kind 1 | 0 to 15 | 25 (24 of 100) | cos above 0.0 | 12.0 | airborne only | distance float[6, 12) | int[180, 420] | 2, knockback 0 | none |
| 1 | ground peck | 0 to 10 | 25 (24 of 100) | cos above 0.9239 | 3.5 | grounded only | none | 60 | 2, knockback 0 | code 0 (none) |
| 0 | lunge, charge kind 2 | 7 to 20 | 25 (24 of 100) | cos above 0.0 | 10.0 | airborne only | distance 6.5 | int[120, 300] | 10, knockback 0 | none |

Speed words (wind-up / strike / recovery): slot 0 1.3 / 4.0 / 2.0; slot 1 1.0 / 3.0 / 0.7; slot 2 1.3 / 5.0 / 2.0. No orient flag. Slots 0 and 2 are blockable (flags 0 and 0x04), slot 1 blockable. The strike of slot 1 sets no status.

#### Animation ids and clips

| Id | Use | Clip |
|---|---|---|
| 0, 1, 7 | ground idle pick | clip 21 (10 keys, loop) |
| 97, 98, 99, 100, 154 | flyer idle, wander, patrol, named target, idle state | clip 16 (10 keys, loop, no root travel) |
| 142 to 148, 150, 151, 155, 156, 158, 159, 162, 164 | flyer fidgets, look (147), lose interest (148), sentry stand and fidget (144, 145), block, hold forms | clip 16 |
| 152, 153, 157, 160 | flyer hit reaction kind 6, and the other ground reactions | clip 13 (56 keys) |
| 3, 5, 6, 8, 12 | ground fidgets (3 is the wander rest fidget) | clips 15, 14, 20 (71 to 76 keys, speed byte 1) |
| 149 | search before agro | clip 22 (18 keys, one-shot, root 0.62 per second at rate 1, gate 69) |
| 103, 104, 105, 106 | strafe forward, back, sides (flying) | clips 1 (16 keys), 18 (8), 25 (10), 17 (10), all loops with no root travel |
| 132 | strafe hold (flying) | clip 23 (14 keys, loop) |
| 140 | melee dodge (flying) | clip 18 (8-key loop, gate s + 31) |
| 138, 139 | missile dodge (flying) | clip 17 (10-key loop, gate s + 39) |
| 76, 74 | hit reaction kind 7 | clip 13 (56 keys, speed byte 1, one-shot, gate s + 110, backward root 3.64 per second) |
| 75 | flyer re-launch | clip 24 (5 keys, gate s + 17, forward root 1.04 per second) |
| 57 | ground hit reaction kind 0 | clip 33 (16 keys, gate s + 61, backward root 6.66 per second) |
| 58 | death | clip 12 (2 keys, 0.95 at s + 5) |
| 163 | victory pose | clip 28 (10 keys, one-shot, gate s + 37) |
| 9, 128 to 131 | state 8 hide pick | clip 5 (10-key hover loop) |
| 2, 4, 11 | ground walk | clip 0 (12 keys, loop, root 1.25 per second) |
| 28 | ground strafe forward | clip 3 (10 keys, loop, root 6.63 per second) |
| 32, 55, 63 | ground strafe back | clip 2 (8 keys, loop, root -7.05 per second) |
| 29, 30, 53, 54 | ground sidesteps | clip 4 (8 keys, loop, root 11.39 per second sideways) |
| 31, 59 to 62, 64 | ground strafe hold | clip 29 (20 keys, loop) |
| 33 / 43 | slot 0 / slot 2 wind-up (A) | clip 22 (18 keys, rate 1.3, gate s + 53) |
| 34 / 44 | slot 0 / slot 2 run (B) | clip 11 (10 keys, loop; root 4.38 per second at rate 1; strike word 4.0 / 5.0) |
| 35 | slot 0 hit (C) | clip 9 (18 keys, one-shot, bit 0x4 on keys 13 to 17, gate s + 69) |
| 45 | slot 2 hit (C) | clip 26 (15 keys, one-shot, bit 0x4 on keys 13 and 14, gate s + 57) |
| 36 / 46 | blocked (D) | clip 13 (56 keys, gate s + 110) / clip 27 (14 keys, gate s + 53) |
| 39 | slot 1 strike (B) | clip 6 (7 keys, rate 3.0, bit 0x4 on keys 5 and 6) |
| 40 | slot 1 follow-through (C) | clip 7 (1 key, bit 0x4) |
| 41 | slot 1 blocked (D) | clip 33 (16 keys, gate s + 61) |
| 42 | slot 1 recovery (E) | clip 8 (4 keys, rate 0.7, gate s + 18, forward root 5.77 per second) |
| 37, 38, 47 | slot 0 / 1 / 2 recovery and wind-up | none: skipped in the same tick |
| 165 | flyer emerge | none (state 8 enters agro at once) |

#### Per-tick order, start and wake

The shared flyer rules give the start (default states 5, 6, 7, 8) and the notice chain: the noticed Crokabek plays 147 (clip 16) while it rolls again; a pass plays 149 (clip 22, 69 ticks) and then enters agro, a failure ends after 120 ticks of searching plus the 39-tick lose-interest clip. In agro each tick runs: recovery countdown, defences, lost-sight test, normal choice, counter choice, distance-hide gate (the row has no hide flag), outer switch, leash, class handler.

#### State table (agro)

| State | Action id and clip | Keys | Gate / exits |
|---|---|---|---|
| Set-up (outer 0) | agro idle pick: 141 while flying (not in this table: clip 0), 1 on the ground (clip 21) | 10, loop | one tick, then outer 4 inner 10 |
| Hold (inner 10) | 132 (clip 23) | 14, loop | hold int[60, 180], then the side-step draw; d above 15 gives approach, d below 3 gives back-off |
| Approach (inner 7) | 103 (clip 1) | 16, loop | 16 units per second toward the hero; ends when d is below the desired distance |
| Back-off (inner 11) | 104 (clip 18) | 8, loop | 16 units per second away; ends when d is above the desired distance |
| Circle (inners 8, 9) | 105 or 106 (clip 25 or 17) | 10, loop | 16 units per second sideways while facing the hero; hold int[60, 180] |
| Lunge, slot 0 | 33 (clip 22, rate 1.3), 34 (clip 11, rate 4.0), 35 (clip 9) | 18, 10, 18 | A gate c + 53; run from c + 54; hit window 52 to 69 ticks after the C request; end path after the C gate |
| Swoop and steal, slot 2 | 43 (clip 22), 44 (clip 11, rate 5.0), 45 (clip 26) | 18, 10, 15 | A gate c + 53; run from c + 54 until contact or the charge distance |
| Ground peck, slot 1 (grounded only) | 39 (clip 6, rate 3.0), 40 (clip 7), 42 (clip 8, rate 0.7) | 7, 1, 4 | hit at s + 8; follow-through C requested s + 9 (gate s + 10); recovery E requested s + 11, gate and end path s + 29. A miss: B gate s + 10, then E as above |
| Melee dodge (outer 5) | 140 (clip 18) | 8, loop | gate s + 31, then the defence exit |
| Missile dodge (outer 6) | 138 or 139 (clip 17) | 10, loop | gate s + 39 |
| Knock-down (outer 10 kind 7) | 76 / 74 (clip 13) | 56 | grounds the flyer; waits for gap at most 2.0; about 110 ticks |
| Grounded hit reaction (kind 0) | 57 (clip 33) | 16 | gate s + 61 |
| Re-launch (state 22) | 75 (clip 24) | 5 | gate s + 17, then the setter with 0 |
| Wait in line (outer 12) | 98 (clip 16) | 10, loop | flies to the stand-off point |
| Victory pose (state 10) | 163 (clip 28) | 10 | gate s + 37, then the default state |
| Dead | 58 (clip 12) | 2 | corpse leaves about 7 ticks after death |

#### Attack chooser

Each agro tick, with the hero attackable, the recovery countdown below 1 and the outer sub-state 1, 3, 4 or 11: slot 2 first (cooldown spent, 0 < d < 15, facing cosine above 0.0, a route exists, airborne, roll int[1, 100] below 25), then slot 1 (only grounded), then slot 0 (7 < d < 20, route, airborne, roll below 25). Airborne with d between 7 and 15 both charge slots are live and the chance of a start in one tick is 1 - 0.76 x 0.76 = 42 percent (24 percent a swoop, 18 percent a lunge); from 15 to 20 only the lunge is live (24 percent); inside 7 only the swoop (24 percent). Grounded: slot 1 only, 0 < d < 10, cosine above 0.9239, roll below 25 (24 percent per tick), cooldown 60 ticks. After a choice the slot cooldown is drawn immediately and again at the end path, and the desired and charge distances are redrawn.

#### Movement

Strafe archetype with the flyer gaits (shared rules): hover 5.4 to 12.6 units from the hero, circling at 16 units per second, approaching when d exceeds 15 and backing off when d is under 3. Height: hero y + 0.2 d, at least ground + 2.0, 6 units per second vertical; in the charge states the hero's y + 2.0. Wander and patrol speed 6 units per second; a grounded Crokabek uses the ground gaits (walk 1.25 units per second, strafe forward 6.63, back 7.05, sidesteps 11.39) with no row speed.

Movement table (movement pass; the Mountain and Swamp Crokabeks use the same clips; their row speeds are in their sections; speed byte 2 unless noted):

| Mode | Source | u/t | u/s | Turn | Clip (id: clip, keys, loop) | Exits |
|---|---|---|---|---|---|---|
| Wander, patrol, follow, wait in line (airborne) | row speed only (idle 3.0) | 0.100 | 6.0 | steering 0.3 / 0.1 | 98, 99, 100: clip 16, 10, loop, no root | goal reached |
| Strafe approach (inner 7), back off (inner 11), circles (inners 8, 9) | row speed only (agro 8.0) | 0.267 | 16.0 | approach steers; back and circles face the hero | 103: clip 1; 104: clip 18; 105, 106: clips 25, 17; all loops, no root | d below desired; d above desired; hold count or blocked flag |
| Strafe hold (inner 10) | none | 0 | 0 | aim 0.5 (flying weight) | 132: clip 23, 14, loop | hold count, distance tests |
| Vertical | height keeper | 0.1 up or down, or snap within 0.1 | 6.0 | - | - | target = hero y + min(30, 0.2 d), at least ground + 2.0; hero y + 2.0 in states 7 to 12 |
| Lunge wind-up (state 6) | root x 1.3 | 0.0135 average | 0.81 | aim 0.5 each tick | 33: clip 22, 18, one-shot (gate c + 53) | gate |
| Lunge run (state 7) | row speed + root x 4.0 | 0.267 + 0.292 = 0.559 | 33.5 | none (heading fixed); solid | 34: clip 11, 10, loop | horizontal d at most 10.0, or travelled 6.5 |
| Lunge hit clip (state 8) | row speed (clip root 0) | 0.267 | 16.0 | none; solid | 35: clip 9, 18, one-shot (gate +69) | clip finished |
| Swoop run (slot 2, state 7) | row speed + root x 5.0 | 0.632 | 37.9 | none; not solid on run ticks | 44: clip 11 | contact or travelled float[6, 12) |
| Melee dodge (outer 5), missile dodge (outer 6) | none | 0 | 0 | none | 140: clip 18; 138, 139: clip 17 | gate (31, 39 ticks) |
| Knock-down fall and slide (kind 7) | gravity, then root backward (speed byte 1) | 3.64 u/s average over the clip's 112 ticks (gate 110), 6.80 units in all | 3.64 | none | 76 then 74: clip 13, 56 keys | clip finished |
| Grounded: walk / strafe forward / back / sides | root (no row speed) | 0.0208 / 0.1105 / 0.1175 / 0.1898 | 1.25 / 6.63 / 7.05 / 11.39 | ground gaits; aim 0.5 in holds | 2, 4, 11: clip 0; 28: clip 3; 32, 55: clip 2; 29, 30: clip 4 (29 to +x, 30 to -x) | grounded countdown |
| Grounded hit reaction kind 0 | root backward | 7.11 units over 61 ticks | 6.66 | snapped to the attacker | 57: clip 33, 16 | gate |
| Re-launch (state 22) | root + keeper | 0.35 units forward over 17 ticks | 1.04 | none | 75: clip 24, 5 | gate |
| Death | none (falls under gravity) | 0 | 0 | none | 58: clip 12 | removal |

The grounded Crokabek is subject to the edge guard and its navmesh is the flying mesh (shared flyer rules); the airborne one is not guarded.

What a port most likely gets wrong (movement): giving its strafe clips root travel (the whole airborne speed is the row speed); a run that steers; a body that is not solid during the lunge (the hero must be pushed ahead of a lunging Crokabek; see Hit and hurt windows); a lunge run at the row speed only (the clip's root times the strike word 4.0 adds 0.292 per tick).

#### Hit and hurt windows

- **Lunge (slot 0).** The wind-up A takes 53 ticks (rate 1.3) and state 6 holds the aim; the body drifts about 0.7 units forward during it. At c + 53 the run is requested (B, rate 4.0) and state 7 runs from c + 54 at 0.559 units per tick. If the horizontal distance is already at most 10.0 on the first state-7 tick (a hero closer than about 10.7 units at the choice), state 8 follows at once; otherwise the run goes on until the distance is 10.0 (a few ticks) or the travelled distance reaches 6.5 (a hero farther than about 17.0 units at the choice: the lunge ends in state 12 without a hit). State 8 requests C on its first tick (tick c8); weapon 3 (radius 4.0, height 4.0) is live from tick c8 + 52 to c8 + 69 (keys 13 to 17; the last key stays until the gate); one hit of 10 damage with no knockback; the gate passes at c8 + 69, state 12 follows, the recovery clip is missing, and the end path runs in the next tick. While C plays the body flies on at 0.267 units per tick, so at the first window tick it is 13.6 to 13.9 units ahead of where it was when C was requested and at the last about 18.1. **Resolved (movement pass): whether the lunge connects (Medium).** The earlier estimate here (a 5.2-unit sphere around the body; hits for a hero 9.4 to 17.2 units away at the choice) is withdrawn; the Weapon reach table below is right about the weapon, but neither decides the outcome alone, because the lunging body is solid (charge kind 2; shared Movement rules, item 10) and pushes the hero.
  - Without the push, nothing is hit. For a hero standing still at D = 4 to 30 units, C is requested with the hero at most about 9.5 units ahead (the run stops at the 10.0 test, one more step follows), and at the first window tick the body is already 4.3 to 5.3 units past him; the weapon then reaches at most about 2.9 units behind the body, and later window ticks are further away. So a hero who is not in the body's path is never struck by the lunge.
  - With the push, a hero who stands in the line is carried. The aim of state 6 (weight 0.5 per tick for 53 ticks) puts the line through the hero's position, so when the body arrives (row radius 1.0 + 1.175 = 2.175 between the centres, horizontally; the flyer's body at hero y + 2.0 overlaps the hero's height) the hero is pushed ahead of it by the full overlap every tick, stays 2.18 ahead, and is struck on the first window tick (c8 + 52, 10 damage). Example D = 12: c8 = c + 58, the body reaches the hero at about c + 83 and carries him about 7.4 units; hit on c + 110.
  - A hero off the line by more than a little is pushed aside instead and the body passes him: the sideways part of each push grows every tick. Model results for a hero standing still with a sideways offset e at the choice (D 10 to 16): e up to about 0.05 is still carried and hit; e of 0.2 or more slides off, ends about 2.2 beside the path and is not hit. So a hero who sidesteps by a fraction of a unit during the 53-tick wind-up is missed, and a hero who does nothing is hit.
  - A hero who flees straight away at walking speed after the wind-up is hit for D about 4 to 17; at running speed only from about 4 units; one who walks toward it is not hit (passed before the window).
Default if unresolved (rewrite choice): implement the body push and the kind-2 solid rule exactly; the outcome then follows. Medium for the outcome (the hero pose is taken as the standing pose; the slide-off threshold depends on that, which only a play check can settle: needs a runtime check).
  - **Answer for a hero standing still (Medium; High for the code facts: kind 2 never makes the body non-solid, and a standing hero is the body that gets moved).** The original does land the lunge, so a build whose lunge never connects on a hero who stands in the line is wrong. A hero standing still is never hit by the weapon volume alone (the volume opens with the body 4.3 to 5.3 units past him; it reaches at most 4.0 behind the body). The hit comes only from the carry: from the tick the body's circle (radius 1.0) touches the hero's (1.175) he is moved ahead every tick by the whole overlap and stays 2.18 ahead of the body's centre, so the first window tick finds him inside the weapon. The state-6 aim (0.5 per tick for 53 ticks) puts the line through his position, so the sideways offset is negligible for a hero who does not move. Hit tick by choice distance D (horizontal distance at the choice tick c; the wind-up starts on c; c8 is the tick the hit clip is requested): D from 7 to 10: c8 = c + 55, hit on c + 107; D 11: c8 = c + 56, hit c + 108; D 12: c + 58 and c + 110; D 13: c + 59 and c + 111; D 14: c + 61 and c + 113; D 15: c + 63 and c + 115; D 16: c + 65 and c + 117; D 17: c + 66 and c + 118. The hit is always on c8 + 52, 10 damage, no knockback, and the hero has been carried about 11 (D 7) down to 6 (D 17) units from where he stood, ending about 2.2 in front of the body (the body flies on at 0.267 per tick). For D above about 17.0 the 6.5-unit run ends with the hero still more than 10.0 away: state 12 on c + 66, no hit clip, no window, no damage. With the band 7 to 20 about 77 percent of the choices on a standing hero connect and the rest cannot. A hero off the line by 0.2 or more at the choice is pushed aside and missed; a hero walking toward the Crokabek is passed before the window. Test vector: hero standing 14.0 away, exactly in the line, level ground, Crokabek airborne in agro: choice on c = 0, wind-up requested 0, run requested on 53 (state 7 from 54), the 10.0 test passes on 60 so the hit clip is requested on c8 = 61, the body reaches the hero's circle at about tick 86 and carries him 2.18 ahead (7.4 units in all), the window opens on 113: 10 damage; the hit clip's gate passes on 130, state 12 on 131, end path on 132. Default if unresolved (rewrite choice): as above, implement the carry; if the carry cannot be built, count a standing hero inside the choice distances 7 to 17 as hit on c8 + 52, which gives the same outcomes for a hero who stands still.
- **Swoop (slot 2).** The wind-up A takes 53 ticks and holds the aim; B (rate 5.0) runs from c + 54 at 0.632 units per tick with weapon 3 live on every tick. The first tick on which the hero's capsule meets the weapon volume ends the run; the run weapon sits 1.2 ahead of the body with a constant pose, so a hero level with the flyer or up to about 2 units below it (the swoop flies at the hero's height + 2.0) is touched once his centre is within about 6.2 units ahead (Weapon reach table; 6.0 when the flyer is 2.4 above his feet). On that tick: damage 2, knockback 0, the steal roll (below), then state 8 (C requested on the following tick) and the flyer keeps flying forward at 0.267 units per tick until the distance travelled reaches the charge distance float[6, 12); then state 12 and the end path. A swoop that never touches the hero ends in state 12 when the distance is reached (about 10 to 19 ticks after c + 54). The swoop connects when the drawn distance exceeds about D - 6.9, where D is the hero's distance at the choice (D less 0.7 of wind-up drift, less the 6.2 reach): always for D up to about 12.9, and at the band's far end (D = 15) on about 65 percent of the draws. The swoop is a kind-1 charge, so the body is not solid on its run ticks and passes through a hero it misses. (Replaces an earlier contact distance of 5.2, which was a stand-in.)
- **Steal.** On the hit tick a roll int[0, 100] strictly below 100 passes on 100 of the 101 draws; it takes one of the items 150 and 149 that the hero holds (a uniform choice among those he holds; nothing if he holds neither). The enemy carries the item for 180 ticks (drawn at 0.45 scale), its carried Neopoints grow by the item's catalogue price, and that amount is paid out as coins when it dies. The item is not returned.
- **Ground peck (slot 1).** B (rate 3.0) hits on its fifth key at s + 8 (single hit, damage 2); a miss ends B at s + 10. Recovery clip E runs 18 ticks after being requested at s + 11. Blocked: state 4 (clip 33 as D, 61 ticks).
- **Hurt.** The Crokabek can be hit in every agro sub-state except the dodges (outer 5, 6), while dead, in state 8 for the hidden placement, and while animation 76, 74 or 75 is the current animation (the knock-down fall and slide, and the re-launch). Every attack phase of every slot is hittable; a hit during an attack abandons it (outer 10).

#### Weapon reach

Conventions are those of "Weapon reach tables" in SPEC-0064 (shared field behaviour). Row 136, Medium. The ticks of the lunge and swoop text above are authoritative; "sC" is the tick the clip is requested (c8 for the lunge). Weapon 3 has radius 4.0 and height 4.0 in every row.

| Slot (phase, clip) | Window ticks (level contact) | Centre forward / up / side | Tor ahead, at the tick | Tor ahead, from the start | Tor feet vs origin |
|---|---|---|---|---|---|
| 0 lunge (C, clip 9) | sC+52 to sC+69 (sC+52 to sC+69) | 0.5 to 1.9 / 0.6 to 3.0 / -0.6 to 0.0 | -4.0 to 6.6 | 10.7 to 23.7 | -5.0 to 6.5 |
| 1 peck (B, clip 6) | s+8 to s+10 (s+8 to s+10) | 0.4 to 0.5 / 1.4 to 2.5 / -0.5 to -0.2 | -3.8 to 5.2 | -3.8 to 5.9 | -3.5 to 5.5 |
| 1 peck (C, clip 7) | sC+1 (sC+1) | 0.4 / 3.6 / -0.8 | -3.8 to 4.0 | -3.8 to 4.0 | -3.5 to 7.5 |
| 2 swoop (B, clip 11) | s+1 to s+21 (s+1 to s+21) | 1.2 / 1.4 / 0.0 | -3.2 to 6.2 | -3.2 to 18.6 | -5.0 to 4.5 |

The weapon sits close to the body (0.5 to 1.9 ahead), so at a given tick the lunge touches a hero from 4.0 behind to 6.6 ahead of the body, a vertical band from 5.5 below to 6.5 above its origin; the large "from the start" figure (10.7 to 23.7) is the body's flight during the 17-tick window. Whether a lunge connects is decided by the body push of the run, not by this table alone (Hit and hurt windows, Resolved).

#### Defence and counters

Melee dodge: once per hero swing, hold below 1, hero within its reach and in front; roll below 50 (quick, 49 of 100), 30 (normal), 20 (heavy). It plays 140 in place for 31 ticks and is unhittable meanwhile; after the exit the hold is int[60, 180]. The missile defence is effectively off (defend time 0). No counter slot is armed (codes -1, 0, -1).

#### Hero actions against this enemy

See the shared table. In addition: the first hit drops the Crokabek and opens its short grounded form (300 to 900 ticks) in which it bites for 2 damage on 24 percent of the ticks inside 10 units; it takes off afterward and must notice the hero again. A swoop costs the hero an item 150 or 149 almost every time one is held, so a hero who holds neither loses nothing.

#### Edge cases for implementers

- The attack bands overlap (swoop 0 to 15, lunge 7 to 20); the swoop is tested first.
- A Crokabek of alarm group 9 (two at Brightvale Coast) is woken by its group's broadcast; it does not need to notice the hero.
- A hit during the wind-up or the run abandons the attack (outer 10); the cooldown already drawn stays.
- The wind-up phase aims; the run and the hit phase do not, so a hero who walks sideways during the 53-tick wind-up is missed completely.
- The drop ordinal is set on 30 of the 48 placements.
- Grounded and slot 1: the peck is the only slot usable while grounded. It needs 0 < d < 10, a facing cosine above 0.9239 and a roll below 25 (24 percent per tick); after a use the cooldown is 60 ticks (drawn at the choice and again at the end path) and the recovery is int[30, 60].

#### What a port most likely gets wrong

1. Letting the lunge track the hero. It aims for 53 ticks and then flies straight for 12 to 70 ticks.
2. Making the swoop a ground bite: it is an airborne dash that steals item 150 or 149.
3. Keeping it airborne after a hit; it falls and fights on the ground.
4. Using the plain walking gait while hovering.
5. Giving it a missile or a block.

#### Test vector

Plain Crokabek (row 136), already in agro (t = 0 the setter tick), hero standing 12.0 units away at ground level in the Crokabek's view, holding items 150 and 149, hero not attacking, no other enemy. t = 1: set-up. From t = 2 the choice is 42 percent per tick (a swoop 24, a lunge 18). Swoop at c: wind-up requested c, gate c + 53, B requested at c + 53, run from c + 54 (distance 11.3 after the 0.7 wind-up drift); contact on c + 62, when the distance has fallen to about 6.2 (5.1 units travelled); damage 2 and a steal of item 150 or 149 (100 of 101); state 8 from that tick; the flyer flies on until it has travelled between 6 and 12 units from its start point, so the end path comes between c + 65 and c + 87; recovery int[30, 60]; slot 2's cooldown is redrawn at the end path (180 to 420 ticks). Lunge at c: A until c + 53, run from c + 54 at 0.559 per tick; from 12 units the distance is 11.3 on the first run tick and 9.6 after three ticks, so state 8 is set at about c + 57 and C is requested at c8 = c + 58; the window is c8 + 52 to c8 + 69; the solid body reaches the standing hero at about c + 83 and carries him ahead (2.18 between the centres), so the hit (10 damage) lands on the first window tick c8 + 52 = c + 110 with the hero about 7.4 units from where he stood; had the hero sidestepped by 0.2 or more during the wind-up, he would be pushed aside and the window would open with the body about 4.7 units past him: no hit; state 12 at c8 + 70, end path c8 + 71 (about c + 129). Over 600 ticks the Crokabek makes 3 to 4 attack starts (about 1 to 2 swoops, 1 to 2 lunges), each followed by recovery 30 to 60 ticks and the long slot cooldowns; the hero loses at most the two potions and about 2 to 10 HP.

Hit sequence (Tor hits a Crokabek that is already in agro, hovering at the floor height 2.0 above flat ground, hero 4.5 units away, Crokabek at 25 hit points, hit damage below 25; ticks count from R, the tick on which the reaction handler first runs; the draw is c = 600 as an example):
- Hit tick (R or the tick before): damage applies with no doubling (already in agro), outer sub-state 10 with kind 7, the force-start word is set, the alert flag is set, the Crokabek turns toward Tor. The animation is still the hold (132); it is hittable.
- R: handler, kind 7, animation 76 is not current: request 76 (clip 13 starts at key 0 on this tick's animation step), the flying test passes, so the countdown is drawn (c = 600), the flying mesh is picked again (same mesh, no change). From R the Crokabek is not hittable.
- R + 1: the pre-step counts 600 down to 599. The update hook no longer sets the no-gravity flag; the keeper and row speed are off. The handler sees 76 current and reads the ground gap: 2.0, not above 2.0, and the table holds 74, so the reaction goes to kind 8. Gravity has started: the body is still 2.0 above the ground and falls 0.036 units in the first tick, 0.107 in the second (0.0178 n (n + 1) in all); the ground constraint puts it on the ground about 11 ticks later (R + 12), where the gap becomes 0.
- R + 2: kind 8 requests 74. Clip 13 keeps playing (no restart).
- R + 2 to R + 109: the progress of clip 13 is below 0.99, nothing changes; the body falls to the ground and slides backward 3.64 units per second (about 6.8 units over the clip; the edge guard does not act here because the gap never exceeds 2.0). It is not hittable and does not steer.
- R + 110: progress reaches 0.99; the flag is read on this tick or the next and the handler runs the outer entry (outer 0). R + 111 at the latest: outer 0 sets the strafe archetype with the ground gait set (hold clip 29, walk clip 3, side clip 4). The Crokabek is hittable again as soon as the current animation is no longer 74.
- R + 112 to R + 600: grounded fight. Only slot 1 (peck: 0 < d < 10, facing cosine above 0.9239, 24 percent per tick, cooldown 60) can start. A hit now gives kind 0: animation 57 (clip 33, 61 ticks), no new countdown, hittable throughout. The countdown has value 600 - j on tick R + j.
- R + 600: the pre-step leaves the value 0. R + 601: gate passes (outer is not 2 with sequencer 0 to 3), the mesh is picked again, state 22 is written and the value becomes -1. The flying test passes: the keeper lifts the body toward the last goal height at 6 units per second (0.1 per tick; 20 ticks for 2.0 units).
- R + 602: state 22 handler requests 75 (clip 24); not hittable from here. R + 619: clip finished (17 ticks), setter 0, class state 0. R + 620: the default-state setter runs (wander, patrol, sentry or hover-and-wake according to the placement); the Crokabek is out of agro and must notice the hero again.
Total time not hittable: R to R + 111 (about 112 ticks) and R + 602 to R + 620 (about 19 ticks).
Same hit with the hero 20 units away and the hero standing on level ground: hover height 4.0 above the ground, gap 4.0; the handler waits until the fall has taken the gap down to 2.0: 0.0178 n (n + 1) at least 2.0 first holds at n = 11 ticks, so 74 is requested about 11 ticks later than in the first case (R + 13 instead of R + 2); the clip does not restart, so its end stays at about R + 110.

Attack cycle with its exits (swoop, slot 2; hero standing still 8.0 units away on flat ground, holding item 150; the Crokabek hovers 2.0 above the ground; c is the choice tick; the charge distance drawn is 9.0 from [6, 12) as an example):
- c: the roll below 25 passes (the swoop is tested before the lunge). The slot cooldown int[180, 420] and the desired strafe distance are drawn at once, the charge distance is drawn, outer sub-state 2, sequencer state 6. Animation 43 (clip 22, rate 1.3) is requested.
- c to c + 52: wind-up. Every tick the Crokabek aims at the hero and rewrites the start point. It drifts about 0.7 units toward the hero (horizontal distance 7.3 at the end). It is hittable throughout.
- c + 53: animation 43 passes its gate. The start point is the position on this tick. Animation 44 (clip 11, rate 5.0) is requested, state 7, tick counter 0.
- c + 54 and c + 55: state 7 each tick: counter up, strike test (weapon 3 is live on every key of the run clip), straight run at 0.267 + 0.365 = 0.632 units per tick with no steering. The horizontal distance seen by the strike test runs 7.3, 6.67.
- c + 56: the distance 6.04 is inside the swoop's reach of about 6.2: hit. Damage 2, no knockback; the steal roll int[0, 100] below 100 passes and item 150 is taken. State 8. Distance travelled t = 1.26; the motion of this tick still runs at the strike word (t = 1.90 after it).
- c + 57: state 8 requests animation 45 (clip 26, 15 keys) and compares t with 9.0 on this and every later tick. From now on the play rate is 1.0: this tick's step is the run clip's root at rate 1.0 plus the row speed (0.34, t = 2.24); from c + 58 the body flies on at 0.267 units per tick (row speed only; clip 26 carries no travel).
- c + 84: t has reached 9.0 (26 steps of 0.267 after c + 57): state 12. c + 85: recovery id 47 is missing, so the end path runs on this tick at once: outer 0, recovery int[30, 60] (say 45), slot 2 cooldown redrawn. The body ends about 1.8 units past the hero's position. The next attack choice is possible from about c + 130 (the swoop itself not before the cooldown ends). Medium for the exact ticks (per-key root variation of the run clip).
- A hit by Tor on any tick from c to c + 84 abandons the attack (outer sub-state 10, kind 7 as above); the sequencer state is left as it was until the next choice rewrites it.
- If instead Tor holds guard facing it (facing dot at most -0.5) on c + 56, the strike is blocked: state 9 with animation 46 (clip 27, 14 keys, backward root 10.56 units per second, so the net movement is slightly forward, about 0.09 units per tick while the clip plays, then 0.267). State 9 waits until t reaches 9.0 (about c + 120); there is no stall exit and no wall exit in this state (Medium), then state 12 and the end path as above.
- The lunge (slot 0) differs in having no strike test in state 7: from distance 12 the run goes to state 8 at about c + 57 (horizontal distance 9.62 at most 10.0), animation 35 (clip 9, 18 keys) is requested at c + 58 and the weapon window is c + 110 to c + 127; its hit is tested in state 8 on every tick until the clip finishes (c + 127), then state 12, recovery id 37 missing, end path on c + 128.

#### Open questions

- Resolved (movement pass; Medium): the lunge range conflict between the earlier sphere estimate and the Weapon reach table. The table is right about the weapon; the outcome is decided by the solid body carrying the hero (Hit and hurt windows, Resolved). A still hero in the line is struck at the first window tick; a hero off the line by about 0.2 or more is pushed aside and missed.
- Resolved (Medium; R-0199): whether the Crokabek lunge should connect on a hero standing still, and when. It does, whenever the choice distance is 17.0 or less, on the hit clip's first window tick (c8 + 52, which is c + 107 to c + 118), through the carry of the solid body; the figures are under Hit and hurt windows. A build that tests only the weapon volume never connects.
- Weapon contact geometry: the anchor of attachment 2 relative to the body and the hero's capsule pose decide whether the lunge's late window can touch a hero it has flown past. Resolved by the general rule (SPEC-0064, Shared field behaviour: Weapon contact and the hero's hurt shape); the distances are in the Weapon reach table; stand-in only for a port without attachment frames: count a window as hitting when a level hero is inside the "at the tick" range of the slot's row (4.0 behind to 6.6 ahead for the lunge) and his feet are within the row's feet band.
- Resolved: the id change from 76 to 74 does not restart clip 13 (the same clip is already on the main layer), so the knock-down slide lasts about 110 ticks from the tick that requests 76 (High).
- Whether kind 5 (animation 127, absent) can occur on a grounded Crokabek. Resolved (High; proactive (open close-out)): no; kind 5 needs a stored flag that no routine of these kinds sets (SPEC-0038 Behavior 37, step 7), so only kind 0 occurs.
- Which items 150 and 149 are, and whether the steal has a visible effect on the hero. Resolved (High for the names; Medium for the lack of an effect; proactive (open close-out)): item 150 is Red Juppie and item 149 is Purple Juppie, the hero's basic restoratives (he starts a new game with three of each). The steal removes one unit from the inventory and calls nothing on the hero; the Crokabek carries the item for 180 ticks drawn at 0.45 scale, and its carried Neopoints grow by the item's catalogue price (SPEC-0029 Behavior 11b).

### Mountain Crokabek

The Mountain Crokabek is the Crokabek's heavier form. It uses the same clip table layout (a table of its own with the same slot ids, plus the differences below), the same shared flight rules and the same notice values.

#### Differences from the Crokabek

| Item | Mountain Crokabek (row 137) |
|---|---|
| Hit points, body | 55; radius 2.0, height 2.0, weight 1.0; attack-manager weight 5 |
| Flight | Idle 3.0 / agro 6.0 (6 and 12 units per second); max fly height 100; grounded 600 to 900 ticks |
| Defence | Missile dodge 100 with defend time 0 (never in practice); melee dodge 75, 50, 30 (74, 49 and 29 of 100); defence timer int[60, 180] |
| Slots | Slot 0 (lunge, charge kind 2, distance 6.5): band 7 to 20, chance 50 (49 of 100), damage 25, delay int[120, 300], speed words 1.3 / 5.0 / 1.5. Slot 1 (ground peck): band 0 to 10, chance 50, arc 0.9239, range 3.5, special grounded only, damage 45, knockback 2.0, delay 60, speed words 1.0 / 3.0 / 0.7. Slot 2 and 3 unused (the steal data of slot 2 is ignored) |
| Weapons | Weapon 3 radius 5.0, height 5.0; the others radius 3.0, height 2.0 (weapon 5 height 1.0) |
| Table differences | The slot ids, the flyer gait ids, the dodge ids and the knock-down ids (76, 74, 75, 57, 58) are the Crokabek's. Missing: 59 to 61, 128 to 131 (so the hide pick of selector 1 is absent), 144, 145, 156, 164. Different: id 31 (ground strafe hold) is clip 15 (71 keys, one-shot), 64 and 66 are clip 14, 65 is clip 29, 81 is clip 5, 152 is clip 27 (14 keys, gate s + 53, backward root 10.56 per second; unused, kind 6 does not occur). Added: id 56 is clip 32 (18 keys) |
| Start states | 16 patrols (loop, no rest), 3 follow a named character (state 9), 1 wander (R 0) |
| Item steal | none (unused slot) |

Run speeds: lunge 0.200 + 0.365 = 0.565 units per tick; the body flies on at 0.200 per tick in state 8, so it is 10.4 units ahead of its entry point at the first window tick and 13.8 at the last. Resolved (movement pass; Medium): the weapon (radius 5.0) and the slower run make this lunge connect where the plain Crokabek's misses. A hero standing still at 7.5 to 17 units at the choice is struck even when the body does not touch him; with the body push (radius 2.0 + 1.175 = 3.175 between the centres) a hero standing up to 3 units to the side of the line is also struck, within about 4 ticks of the first window tick, after being shoved about 6 units. A hero beyond about 17.2 is not reached (the run ends at the 6.5 travel limit). The earlier figure (contact radius 6.2, hero 4.9 to 17.2 at the choice) was a sphere estimate; the model above replaces it. Its first-hit reaction is kind 7 (knock down) as for the Crokabek; the grounded form bites for 45 damage with knockback 2.0 (the hero is thrown into state 48) at 49 percent per tick inside 10 units, delay 60.

Start, wake, state table, chooser, movement and defence follow the Crokabek's with these values. Attack chooser: slot 1 first (grounded only), then slot 0 (7 to 20, 49 percent per tick, route needed, airborne). The slot ids and clips of the lunge and of the ground peck are the Crokabek's (wind-up A clip 22 at rate 1.3, run B clip 11 at rate 5.0, hit C clip 9; ground peck B clip 6, C clip 7, E clip 8).

#### Weapon reach

Conventions are those of "Weapon reach tables" in SPEC-0064 (shared field behaviour). Row 137, Medium. The lunge text above is authoritative for its ticks; "sC" is the tick the clip is requested (c8 for the lunge). Weapon 3 has radius 5.0 and height 5.0.

| Slot (phase, clip) | Window ticks (level contact) | Centre forward / up / side | Tor ahead, at the tick | Tor ahead, from the start | Tor feet vs origin |
|---|---|---|---|---|---|
| 0 lunge (C, clip 9) | sC+52 to sC+69 (sC+52 to sC+69) | -0.4 to 0.0 / 0.8 to 2.8 / 0.0 | -5.8 to 6.0 | 5.4 to 18.4 | -7.5 to 8.0 |
| 1 peck (B, clip 6) | s+8 to s+10 (s+8 to s+10) | 0.0 / 0.3 to 1.8 / 0.0 | -3.4 to 5.6 | -3.4 to 5.6 | -8.0 to 6.5 |
| 1 peck (C, clip 7) | sC+1 (sC+1) | 0.0 / 3.6 / 0.0 | -2.6 to 4.8 | -2.6 to 4.8 | -5.0 to 9.0 |

The weapon is centred on the body, so the lunge touches a hero from 5.8 behind to 6.0 ahead of the body at the tick, a wider band than the plain Crokabek's, and a hero 5.4 to 18.4 ahead of where state 8 began (the body flies about 10.4 to 13.8 during the window). It is the lunge that reaches a close hero, as stated above.

#### What a port most likely gets wrong

1. Giving it the steal (unused data).
2. Missing its grounded bite: 45 damage and a knockdown, the hardest hit of the batch.
3. Letting it dodge shots (defend time 0), or forgetting its high melee dodge (74 percent for a quick swing).
4. Treating its patrols as wander.

#### Test vector

Mountain Crokabek in agro, hero standing 12 units away: t = 1 set-up; from t = 2 a lunge starts with 49 percent per tick (slot 1 is closed while airborne), so the first choice c is t = 2 with 49 percent, t = 3 with 25 percent, later otherwise. A gate c + 53, run from c + 54 at 0.565 per tick (distance 11.3 on the first run tick, inside 10.0 after 3 ticks), so c8 = c + 58; window c8 + 52 to c8 + 69; the hit of 25 lands at or within about 4 ticks of the first window tick (the solid body reaches the standing hero at about c + 86 and pushes him ahead, about 3.2 between the centres, about 6 units in all; without the push the weapon still reaches him, the body being about 1.8 past him at c8 + 52); end path c8 + 71, recovery 30 to 60, cooldown int[120, 300] redrawn at the end path. Over 600 ticks: 2 to 3 attack starts. The first hit on it drops it for 600 to 899 ticks, during which it bites for 45.

#### Open questions

- The two Steppe Plateau placements have no flying mesh. Resolved (High; proactive (open close-out)): they keep navigation index -1 and cannot walk or wander (see the shared edge cases).
- The named-character ids 0 and 8 of the state-9 records. Resolved (Medium-High): the ids are record ordinals of the flyer's own region, and on the disc that region is always the pack's own embedded region. The lookup compares the id with each character's world-wide id, which is the id base of its region block plus its record ordinal; the embedded block's base is 0 (the word that would set a base is never written), and each created slot region starts at 50 times (its slot number plus 1), that is 50 and up for the first slot, so a stored id of 0 or 8 can only match a record of the embedded block and never a slot region's record. The two Brightvale Outskirts flyers (pack 756, embedded records 1 and 3) follow embedded record 0, an unnamed Whinny, and hover 10 units from it until they notice the hero; the Crossroads flyer (pack 882, embedded record 10) follows embedded record 8, an unnamed Mortog, at 30 units; if the lookup fails they wander. Nothing else is needed: the three Cogham Village Spearmen (pack 843) are embedded too.

### Swamp Crokabek

The Swamp Crokabek is the Crokabek of Bogshot Swamp and Werelupe Woods: 35 hit points, a wider strafe band (10 to 30), a faster lunge for 15, a 25-damage ground peck, and a curse missile in its data that never fires (the slot has no clips, so it only wastes a few ticks and a recovery). It is grounded by its first hit like the Crokabek.

#### Differences from the Crokabek

| Item | Swamp Crokabek (row 138) |
|---|---|
| Hit points, body | 35; radius 1.0, height 2.0; attack-manager weight 0; orient flag on slot 1 |
| Flight | Idle 3.0 / agro 8.0 (6 and 16 units per second); max fly height 20; grounded 300 to 900 ticks |
| Strafe | Band 10 to 30, desired 14 to 26; stand-off radius 10; backbiter 1.0 |
| Defence | Missile dodge 75 with defend time 180; melee dodge 100, 30, 25 (the quick-swing dodge passes on 99 of 100); defence timer int[60, 180] |
| Slot 0 | Lunge, charge kind 2, distance 6.5, band 7 to 20, chance 25, damage 15, delay int[120, 300], speed words 1.3 / 6.0 / 2.0 (run 0.705 units per tick) |
| Slot 1 | Ground peck: band 0 to 10, chance 25, arc 0.9239, range 3.5, special grounded only, damage 25, delay 60, orient on |
| Slot 3 | **A missile slot with no animation at all**: band 0 to 40, chance 75, special airborne only, delay int[300, 600], speed 1.0, range 70, radius 2.0, damage 0, flags 0x21, status curse (id 1) at chance 100 for 30 to 60 seconds, visual kind 6, spawn count 1 |
| Table | Ids 48 to 52 (all five phases of slot 3) are missing; ids 61 to 65 differ: 61 is clip 31 (17 keys, loop), 62 and 63 are clip 13 (56 keys), 64 is clip 14 (71 keys), 65 is clip 8; 81 is clip 5; 152 is clip 27 (14 keys, gate s + 53); no ids 128 to 131 and no 144, 145, 156, 164 |
| Start states | 12 hover-and-wake (wake distance 10 for 7, 12 for 5), 2 wander (R 85, rest int[0, 20] at 25 percent, the only rests of the batch) |

#### The slot with no clip

Slot 3 is tested first each tick (the highest slot first). While its cooldown is spent and d is under 40 the Swamp Crokabek starts it with probability 74 percent per tick (roll below 75). The sequencer starts in state 0 (missile, charge -1): when the hero and the steer point are on the walk mesh it goes to state 1 at once, otherwise it turns toward the hero first. State 1 finds the wind-up missing and the strike missing, so it goes to state 5; state 5 finds the recovery missing and runs the end path: outer 0, recovery int[30, 60], slot 3 cooldown drawn again from 300 to 600. No projectile is ever created (the spawn needs state 2), so the curse never reaches the hero. The attack takes 3 ticks and its only effects are the two cooldowns: an attack-free recovery of 30 to 60 ticks about once every 300 to 600 ticks, and a first real attack that is delayed by that dud at the start of every fight (74 percent chance on the first eligible tick). The ground form's slot 1 and the lunge are otherwise as the Crokabek's.

#### Hit and hurt windows

Lunge: strike speed 6.0, run 0.705 per tick, C hit window c8 + 52 to c8 + 69, 15 damage; the body flies on at 0.267 per tick, so it is 13.9 units ahead of its entry point at the first window tick. Resolved (movement pass; Medium): as for the Crokabek, a hero who is not pushed by the solid body is never struck (the window opens with the body past him); a hero standing still in the line is carried ahead of it and struck on the first window tick. The Swamp Crokabek's faster run (0.705 per tick) makes the push stronger sideways as well: a hero up to about 0.5 off the line is still carried and struck, one about 1.0 or more off it is pushed aside and missed (model, D 10 to 16). Ground peck: B (rate 3.0) hits at s + 8 for 25 damage; end path s + 29. The rest of the table (ids, dodge and re-launch clips, wind-up A clip 22 at rate 1.3, run B clip 11 at rate 6.0, hit C clip 9) is the Crokabek's.

**Weapon reach.** Conventions are those of "Weapon reach tables" in SPEC-0064 (shared field behaviour). Row 138, Medium. The lunge text above is authoritative for its ticks; "sC" is the tick the clip is requested (c8 for the lunge). Weapon 3 has radius 4.0 and height 4.0.

| Slot (phase, clip) | Window ticks (level contact) | Centre forward / up / side | Tor ahead, at the tick | Tor ahead, from the start | Tor feet vs origin |
|---|---|---|---|---|---|
| 0 lunge (C, clip 9) | sC+52 to sC+69 (sC+52 to sC+69) | -0.3 to 0.0 / 0.6 to 2.3 / 0.0 | -4.8 to 5.0 | 9.8 to 21.7 | -6.5 to 6.5 |
| 1 peck (B, clip 6) | s+8 to s+10 (s+8 to s+10) | 0.0 / 0.2 to 1.5 / 0.0 | -2.8 to 4.4 | -2.8 to 4.4 | -7.0 to 5.0 |
| 1 peck (C, clip 7) | sC+1 (sC+1) | 0.0 / 2.9 / 0.0 | -2.4 to 4.0 | -2.4 to 4.0 | -4.5 to 7.0 |

The lunge touches a hero from 4.6 behind to 5.0 ahead of the body at the tick; measured from where state 8 began the band is 9.8 to 21.7 ahead. Whether a lunge connects is decided by the body push of the run (text above), not by this table alone.

#### What a port most likely gets wrong

1. Firing the curse missile. The slot has no clips; it is a 3-tick dud.
2. Forgetting the dud's recovery: after each attempt the Swamp Crokabek is idle for 30 to 60 ticks.
3. Giving it the steal.
4. Keeping it near the hero: its strafe band is 10 to 30.

#### Test vector

Swamp Crokabek in agro, hero standing 15 units away: t = 1 set-up; t = 2 slot 3 starts at 74 percent (else slot 0 at 24 percent in band, d 15 is inside 7 to 20): dud c, state 1 at c + 1, state 5 at c + 2, end path c + 3, recovery 30 to 60 to c + 63 at the latest, slot 3 closed until c + 303 to c + 603; then the lunge at 24 percent per tick: A until c' + 53, run 0.705 per tick from c' + 54, state 8 within 2 ticks, window c8 + 52 to c8 + 69 (a hit only on a hero carried ahead by the solid body: a still hero in the line, or one up to about 0.5 off it; Hit and hurt windows), end path c8 + 71. Over 600 ticks: 3 to 4 attack starts, one or two of them the dud and one or two a lunge.

#### Open questions

- Whether a state-0 missile wait loops when the hero is off the walk mesh. Resolved (High; proactive (open close-out)): it waits only until the facing test passes (cosine at least 0.95), turning toward the hero at blend 0.1 each tick, and then goes to state 1; it cannot loop for ever (SPEC-0038 Behavior 13, state 0).

### Biting Spellbook

The Biting Spellbook is a 35-hit-point flying book of Fyora's Library and the Rathbone Family Crypts. It hovers 7.2 to 13.8 units from the hero and never starts an attack while the hero looks at it (its backbiter scale is 0); when the hero turns away it bites for 30 or lunges for 20 (both with a knock-down), and after it dodges a Tor swing it may counter. A hit makes it recoil in the air; it is never grounded alive. To beat it: keep facing it and strike between its dodges.

#### Identity and tunables

| Item | Value |
|---|---|
| Record | Monster type 60, variant 0, row 134, name "Biting Spellbook"; one clip table (25 clips) in all 31 playable records (a second copy in the CUT pack); 13 conf placements (5 Library wanderers in region 1330 and 8 Rathbone hover-and-wake books), 18 more in a Library block that no loader names |
| Class and approach | Class 17; approach 4; Can Fly 1; hostile; Display in default 1 |
| Hit points, body | 35; radius 2.0, height 1.2, weight 2.0; attack-manager weight 5 |
| Flight | Idle 1.0 / agro 3.0 (2 and 6 units per second); max fly height 20; flying hit 0 (stagger, never grounded alive) |
| Strafe | Band 5 to 16, desired 7.2 to 13.8; stand-off radius 15; **backbiter 0.0** |
| Turn | Hold aim weights 1.0 ground and flying (instant turn) |
| Defence | Missile dodge 30, defend time 180; melee dodge 30, 30, 30 (29 of 100); block 0; defence timer int[30, 60] |
| Counters | Both slots counter after a dodge (code 2) with chance 50 (slot 0) and 25 (slot 1) |
| Weapons | Weapon 3 (the bit every clip uses): radius 3.0, height 2.0 (full); weapons 1, 2, 4, 5 radius 2.0, heights 1.0, 1.0, 2.0, 2.0 |

Attack slots (slots 2 and 3 unused; testing order: slot 1 first, then slot 0):

| Slot | Kind | Distance band (strict) | Chance | Arc | Melee range | Special | Charge | Delay | Damage, knockback | Counter |
|---|---|---|---|---|---|---|---|---|---|---|
| 1 | bite | 0 to 10 | 75 (74 of 100) | cos above 0.0 | 4.25 | airborne only | none | int[10, 45] | 30, 3.0 | code 2, chance 25 |
| 0 | lunge, charge kind 2 | 8 to 40 | 75 (74 of 100) | cos above 0.0 | 4.25 | airborne only | float[20, 30) | int[10, 45] | 20, 3.0 | code 2, chance 50 |

Speed words (wind-up / strike / recovery): slot 0 3.0 / 2.0 / 1.0; slot 1 2.0 / 1.0 / 1.0. Blockable. No orient flag, no status. The knockback is the hero's velocity 0.6 K forward and 0.8 K up, K = 3.0: the hero goes into the long-fall state 48.

#### The backbiter

The normal choice scales the roll threshold by the backbiter when dot(enemy facing, hero facing) is at most 0. The Biting Spellbook turns to face the hero (weight 1.0 in the hold), so that dot is positive only when the hero faces the same way, that is, looks away from the book. With the hero facing the book (or sideways) the threshold is int(75 x 0.0) = 0 and the roll can never pass: the book never starts a normal attack. It attacks only (a) when the hero turns his back, (b) as a counter after it dodges a swing, or (c) from the loop it is already in. This is the "backbiter" of the row field name.

#### Animation ids and clips

| Id | Use | Clip |
|---|---|---|
| 0 | idle pick (ground) | clip 13 (10 keys, loop) |
| 1, 97, 101, 103, 110, 132, 141 | flyer idle, strafe forward (103) and hold (132) | clip 0 (9-key hover loop) |
| 98, 99, 100, 142 to 148, 154 | wander, patrol, named target, fidgets, look and lose interest | missing: clip 0 plays |
| 149 | search before agro | missing: agro at once |
| 104 | strafe back | clip 21 (9 keys, loop) |
| 105, 106 | strafe sides | clip 9 / clip 8 (9 keys, loops) |
| 9, 128, 129, 130 | state 8 hide pick | clip 5 (2-key loop) / 17 / 18 / 19 (one-key poses) |
| 140 | melee dodge (flying) | clip 7 (20 keys, one-shot, gate s + 77, backward root 8.51 per second) |
| 138, 139 | missile dodge | clips 10 / 11 (16 keys, one-shot, gate s + 61, root 5.46 per second sideways) |
| 152 | hit reaction kind 6 | clip 3 (10 keys, gate s + 37, backward root 10.41 per second) |
| 58, 153 | death (58 is played; 153 is the same clip) | clip 4 (10 keys, 0.95 at s + 36) |
| 163 | victory pose | missing (state 0) |
| 34 | slot 0 run (B) | clip 20 (5 keys, loop, root 3.34 per second at rate 1; rate 2.0 in the run) |
| 35 | slot 0 hit (C) | clip 1 (7 keys, one-shot, bit 0x4 on keys 2 to 4, gate s + 25) |
| 36, 41 | blocked (D) of slot 0 (unused) and slot 1 | clip 12 (18 keys, gate s + 69, backward root 3.77 per second) |
| 39 | slot 1 strike (B) | clip 1 (7 keys, one-shot, bit 0x4 on keys 2 to 4) |
| 33, 37, 38, 40, 42 | the other phases | none: skipped in the same tick |
| 165 | flyer emerge | none |

#### Per-tick order, start and wake

Default states: 5 wanderers in the Library (region 1330) and eight hover books in Rathbone Family Crypts (wake distance 8.0, hide pose 129, a one-key pose; the unloaded Library block would use wake distance 12 with poses 9 or 128). Wander: gait 98 is absent, so clip 0 plays while the body flies at 2 units per second over a square of side 50 with no rests. Noticing: chance 80 at 10 units to 10 at 40 units (both scaled by F1 as in the shared rules); a pass in state 2 goes straight to agro because 149 is missing; a failed repeat ends the state at once (search 0), and the next awareness cycle (32 ticks later) starts the same two-roll chain again. A hovering book at Rathbone wakes on the distance test and enters agro on the tick after. The agro machine then runs as the shared strafe archetype with hold weight 1.0.

#### State table (agro)

| State | Action id and clip | Keys | Gate / exits |
|---|---|---|---|
| Set-up (outer 0) | agro idle pick 141 while flying, 1 on the ground (both clip 0) | 9, loop | one tick, then outer 4 inner 10 |
| Hold (inner 10) | 132 (clip 0) | 9, loop | hold int[60, 180]; faces the hero at once |
| Approach / circle / back-off | 103 (clip 0), 105 or 106 (clip 9 or 8), 104 (clip 21) | 9, loops | 6 units per second; desired 7.2 to 13.8; approach above 16, back off under 5 |
| Lunge, slot 0 | state 6: A missing, B 34 (clip 20, rate 2.0) at the choice tick; state 7 from c + 1; state 8: C 35 (clip 1) | 5 (loop), 7 | no wind-up; run at 0.211 per tick to within 4.25 or the drawn distance (20 to 29.99); window ticks 8 to 19 after the C request; C gate 25, then state 12 and the end path |
| Bite, slot 1 | state 0 approach, then B 39 (clip 1, rate 2.0 on the first tick, then 1.0) | 7 | B starts when d is at most 4.25; hit at s + 7; B gate s + 24; end path s + 25 |
| Blocked bite | D 41 (clip 12) | 18 | state 4: gate s + 69 |
| Melee dodge (outer 5) | 140 (clip 7) | 20 | gate s + 77; backs the book away about 11 units |
| Missile dodge (outer 6) | 138 or 139 (clips 10, 11) | 16 | gate s + 61; 5.5 units sideways |
| Stagger (outer 10 kind 6) | 152 (clip 3) | 10 | gate s + 37; recoil about 6.4 units |
| Dead | 58 (clip 4) | 10 | corpse about 47 ticks after death |

#### Attack chooser

With the hero facing the book: no normal choice ever passes (threshold 0). With the hero facing away (dot above 0): slot 1 for 0 < d < 10 at 74 percent per tick, then slot 0 for 8 < d < 40 at 74 percent per tick (so from 8 to 10 units either, 93 percent combined; beyond 10 only the lunge, 74 percent; under 8 only the bite). Delay int[10, 45] at the choice and again at the end path, recovery int[30, 60]. Counter choice (every tick of the reaction or the dodge): slot 0 (code 2, chance 50) and slot 1 (code 2, chance 25) after a dodge while the hero's recover window is open (Tor's swings only; Roberta's clips never carry it), arc cos above 0.0 and the slot's own band; the last eligible slot wins.

#### Movement

Hover and circle at 7.2 to 13.8 units from the hero (approach above 16, back off under 5), 6 units per second in every direction, height hero y + 0.2 d (cap 20). The bite approaches with gait 0 in state 0 at 6 units per second and is held off by the chase stop within 3.175 units of the hero (radius 2.0 + 1.175), still inside the 4.25 range. The lunge needs no approach: the run starts at the choice and flies at 0.211 per tick.

Movement table (movement pass; Can Fly 1, never grounded alive; speed byte 2):

| Mode | Source | u/t | u/s | Turn | Clip (id: clip, keys, loop) | Exits |
|---|---|---|---|---|---|---|
| Wander (state 5), wait in line | row speed only (idle 1.0) | 0.0333 | 2.0 | steering 0.3 / 0.1 | 98 missing: clip 0, 9-key hover, no root (the gait id still counts as current, so the row speed applies) | goal reached |
| Hover in state 8 | none | 0 | 0 | none | 9 / 129: one-key poses | wake |
| Strafe approach, back off, circles | row speed only (agro 3.0) | 0.100 | 6.0 | approach steers; back and circles face the hero | 103: clip 0; 104: clip 21 (no travel to speak of); 105: clip 9; 106: clip 8 | as the shared strafe rules |
| Strafe hold | none | 0 | 0 | aim 1.0 (snaps) | 132: clip 0 | hold count |
| Vertical | height keeper | 0.1 | 6.0 | - | - | hero y + min(20, 0.2 d), at least ground + 2.0; hero y + 2.0 in states 7 to 12 |
| Lunge run and hit clip (states 7, 8) | row speed + root x 2.0 (run); row speed (hit) | 0.211; then 0.100 | 12.7; 6.0 | none; solid throughout | 34: clip 20, 5, loop | horizontal d at most 4.25, or travelled float[20, 30) |
| Kind-6 hit reaction | root backward | 6.94 units over 37 ticks | 10.41 | snapped to the attacker; goal height frozen | 152: clip 3, 10 | gate |
| Melee dodge 140 | root backward | 11.35 units over 77 ticks | 8.51 | none | clip 7, 20, one-shot | gate |
| Missile dodges 138, 139 | root sideways | 5.83 units over 61 ticks | 5.46 | none | clips 10, 11, 16 | gate |
| Death | gravity only | 0 | 0 | none | 58: clip 4 | removal 10 ticks after progress 0.95 |

Worked example (bite approach). The hero stands 6.0 in front of a hovering book (slot 1: d below 10, roll passes). Sequencer state 0: d above the range 4.25, gait 0 (id 103, no root) at the agro row speed 0.100 per tick; the chase stop discards the step while the 3D distance is below 3.175. From 6.0 it needs 1.75 units (18 ticks) to reach 4.25; state 1 follows on the next tick. In the lunge the book is solid: once the hero is within 4.25 the hit clip runs and the book keeps flying forward at 0.100 per tick, so a hero standing in its line is shoved ahead of it at that speed (shared Movement rules, item 10).

What a port most likely gets wrong (movement): giving the hover gaits travel (only the row speed moves it); moving it at the agro speed while it wanders (2.0 u/s outside agro); grounding it on a hit (kind 6: it recoils 6.9 units in the air).

#### Hit and hurt windows

- **Lunge.** B requested at c (no wind-up clip), state 7 from c + 1 at 0.211 units per tick (rate 2.0: clip 20 root plus the 0.1 row speed); state 8 begins on the first tick with the horizontal distance at most 4.25 (about 37 ticks for a hero 12 units away, up to 125 ticks for 30); C requested the tick after (tick c8), weapon 3 (radius 3.0, height 2.0) live from c8 + 8 to c8 + 19 (keys 2 to 4 at rate 1.0); one hit of 20 damage with knockback 3.0; the gate passes at c8 + 25, the end path runs next tick. The book flies 0.1 per tick throughout C (2.5 units). A hero beyond cdist + 4.25 (24.25 to 34.25 units) is missed.
- **Bite.** From state 0 the book flies at the hero; at d at most 4.25 state 1 runs: no wind-up, B requested at s (the first tick runs at the wind-up word 2.0, then 1.0); hit at s + 7 (key 2) for 30 damage, knockback 3.0; the swing ends s + 24 (C is absent, so state 3 waits for B's gate), end path s + 25. A guarding hero facing the book blocks the bite (state 4, 69 ticks).
- **Hurt.** Hittable in every agro sub-state except the dodges (outer 5, 6), while dead, and in its wake state when the hide pose is on.

#### Weapon reach

Conventions are those of "Weapon reach tables" in SPEC-0064 (shared field behaviour). Row 134, Medium. The tick text above is authoritative. Weapon 3 has radius 3.0 and height 2.0.

| Slot (phase, clip) | Window ticks (level contact) | Centre forward / up / side | Tor ahead, at the tick | Tor ahead, from the start | Tor feet vs origin |
|---|---|---|---|---|---|
| 0 lunge (C, clip 1) | sC+8 to sC+19 (sC+8 to sC+19) | 0.7 / 0.0 to 0.1 / 0.0 | -2.6 to 4.2 | -1.9 to 6.0 | -4.0 to 1.0 |
| 1 bite (B, clip 1) | s+7 to s+18 (s+7 to s+18) | 0.7 / 0.0 to 0.1 / 0.0 | -2.6 to 4.2 | -2.6 to 4.2 | -4.0 to 1.0 |

The weapon centre stays 0.7 ahead of the book at its own height, so a hero is touched only from 2.6 behind to 4.2 ahead, and only if his feet are no more than 1 above and 4 below the book's origin (a flyer that hovers above the hero's head misses).

#### Defence and counters

Melee dodge 29 of 100 for any swing weight, once per swing: clip 7 (77 ticks, backward about 11 units), unhittable meanwhile; hold then int[30, 60]. Missile dodge: a 29-in-100 roll on every agro tick for each hero shot whose path meets the body within the 180-tick look-ahead (so it starts when the shot's path meets the body within 3 seconds, and a shot that is on course for n ticks is dodged with probability 1 - 0.71^n: 0.94 for n = 8, 0.50 for n = 2), plays clip 10 or 11 sideways for 61 ticks; the exit then holds defence off for int[30, 60] ticks, and a landed hit keeps it in its 37-tick reaction with no defence. A dodge opens the counters when it was against a Tor swing: lunge 49 percent or bite 24 percent on the first eligible tick. Hit reaction kind 6: the book stops, plays clip 3 for 37 ticks recoiling 6.4 units, outer 0 afterwards.

#### Hero actions against this enemy

Face it and it never starts an attack (unless it counters); show it your back and it attacks at once. A swing that it dodges (29 percent) may be answered by a lunge or a bite. A hero guarding with L1 blocks the bite when facing it. Roberta's shots face a 29-in-100 dodge roll on every tick they are on course (about 94 of 100 first shots from 10 units); a shot that finds it unaware (wandering) lands doubled, and her quick chain then keeps it in its 37-tick reaction for the next shots.

#### Edge cases for implementers

- Facing: the test is the dot of the two facing vectors; the book turns to the hero at once in its hold (weight 1.0), so for a hero walking away from it the book attacks from behind. A hero strafing while still looking at the book keeps the dot negative.
- The lunge's two hit chances: only the C window can hit; the run has no strike test, so a book that reaches the hero in state 7 stops at once (state 8) and swings.
- The hide poses of the Rathbone books (id 129) and of the unloaded block (ids 9 and 128) are poses of the closed book; the book is hittable in them.
- The Library wanderers and the Rathbone books have no alarm group (255).
- The pursuit of the bite uses the chase stop within 3.175 units (radius 2.0 plus the hero's 1.175), but the range is 4.25, so the swing starts before the stop matters.

#### What a port most likely gets wrong

1. Letting it attack a hero who faces it. Its threshold is 0; it is a backbiter.
2. Using a wind-up: there is none; the lunge starts running at the choice tick.
3. Running the lunge fast: 0.211 units per tick, a lumbering 12 units per second at most (6 u/s agro row speed plus 6.7 of clip travel at rate 2.0).
4. Grounding it after a hit: it only staggers (kind 6).
5. Skipping the counters after a dodge.

#### Test vector

Biting Spellbook in agro, hero standing 12 units away facing the book: t = 1 set-up; the normal choice threshold is 0, so no attack starts in 600 ticks; the book hovers 7.2 to 13.8 units away, circling at 6 units per second in 60 to 180-tick legs; a hero swing (quick, normal or heavy) is dodged on 29 of 100: dodge 77 ticks, then the counter on the following tick is a lunge (49 percent) or a bite (24 percent) if the swing was Tor's and the hero is in the slot's band. Same book, hero turned away from it: slot 0 starts with 74 percent on the first eligible tick (t = 2): c = 2, B requested, state 7 from c + 1 at 0.211 per tick, state 8 at about c + 38, C requested c + 39, window c + 47 to c + 58, hit of 20 damage and the hero knocked into state 48 at c + 47; C gate c + 64, end path c + 66; recovery 30 to 60; cooldown int[10, 45] redrawn; the next choice about c + 100 to c + 130. Over 600 ticks with the hero's back turned: 4 to 6 lunges.

#### Open questions

- Whether the Library block of 18 placements (region 1331) is ever loaded. Resolved (High; proactive (open close-out)): never; no level module loads region 1331, so the 18 placements do not exist in play and a port omits them (SPEC-0066). If a port keeps them anyway, give them the behaviour of the Rathbone books (hover, wake distance 12).
- The anchor of attachment 2 for the lunge window. Resolved by the Weapon reach table (the weapon centre is 0.7 ahead of the book at its own height). Default if unresolved (rewrite choice): a hero from 2.6 behind to 4.2 ahead of the book, feet from 4 below to 1 above its origin, is hit.
- Whether the counter choice can start the lunge while the book is mid-stagger. Resolved (Medium-High; proactive (open close-out)): yes, on any tick of the reaction, per the shared counter rule (SPEC-0038 Behavior 14).

### Shooting Spellbook

The Shooting Spellbook is the Biting Spellbook's rig with one missile slot. It shares everything in the shared rules and the Biting Spellbook's ids (the same clip table layout; the strike clip is different).

#### Differences from the Biting Spellbook

| Item | Shooting Spellbook (row 135) |
|---|---|
| Record | Type 60 variant 1; the one playable record (Fyora's Library, region 1328) has default state 0 (idle in state 0 for ever until hit or scripted); a second copy lives in the CUT pack. No drop ordinal, no alarm group |
| Strafe, backbiter | Band 3 to 15, desired 5.4 to 12.6; backbiter 1.0 |
| Slot 0 | Missile, band 10 to 100, chance 50 (49 of 100), facing cosine above 0.0, special airborne only, no charge, delay int[60, 180], damage 30, knockback 0, flight mode 0 (straight), speed 2.0, range 100 (lifetime 50 ticks), radius 0.5, spawn count 1, aim inaccuracy 0, no status, counter code 2 at chance 50, speed words 3.0 / 2.0 / 1.0, T 5 |
| Slots 1 to 3 | Unused |
| Strike clip | B 34 is clip 2 (15 keys, one-shot, spawn event bit 0x20 on key 6; gate s + 43, because after the release state 3 plays the rest at rate 1.0; the gate would be s + 28 if the strike word held to the end); A, C, D and E missing |
| Combat timer | none (0 to 0) |
| Movement (movement pass) | As the Biting Spellbook (row speeds 2.0 and 6.0 u/s, hover gaits with no root), except the strafe back-off: id 104 is clip 7 (20 keys, one-shot, root backward), so a back-off moves 6.0 (row) + 8.51 (clip) = 14.5 u/s for its first 77 ticks; the finished clip is then held, its last key has no root, and the back-off continues at the row speed 6.0 alone (a later back-off does not restart the clip while id 104 is still the current id). Its hold weight is 1.0. It never moves in its default state 0 (idle pick, hover in place) |

#### Per-tick order, start and wake

The only record is in state 0, which keeps the setter on 0 each tick (no notice, no wander). The book fights only when hit (damage doubled, hit reaction kind 6 then outer 0) or when a script hands it to agro. In agro it runs the strafe archetype (hold 132, clip 0; circle ids 105, 106; back 104; speeds 6 units per second) around 5.4 to 12.6 units.

#### Attack chooser and the shot

Each tick with the hero attackable, recovery below 1 and the outer sub-state 1, 3, 4 or 11: slot 0 (cooldown spent, 10 < d < 100, facing above 0.0, airborne, roll below 50 so 49 per cent per tick, no route needed for a missile). The choice sequencer starts in state 0: a missile with the hero and the steer point on the walk mesh goes to state 1 on the next tick (otherwise it turns until its facing cosine is at least 0.95). State 1: A missing, B (id 34, clip 2) requested at s; the first tick runs at the wind-up word 3.0, afterward the strike word 2.0. The spawn event (bit 0x20 on key 6) is seen on tick s + 12 (the window lasts two ticks, s + 12 and s + 13, but only one shot is created because the shot counter reaches the spawn count 1 on the first). The projectile is created at the emit point 0 (the event word 0x20 selects point 0), toward the hero's position + (0, 2.5, 0) plus a random horizontal offset of length U[0, 0], plus 5.0 when the book is outside the camera cone and 3.0 while it waits in line. Speed 2.0 per tick, lifetime 50 ticks (range 100), radius 0.5, damage 30, element T 5, no status, no area. It reaches a hero 30 units away after 15 ticks. State 3 waits for the B gate; state 3 stores no speed word, so the clip plays its remaining keys at rate 1.0 (cursor 6.75 after the release tick, 0.25 key per tick after it) and the gate passes on s + 43; state 5 has no recovery clip, and the end path runs on s + 44: recovery int[30, 60], cooldown int[60, 180] drawn again.

#### Hit and hurt windows

The shot is the only damage source: one 30-damage projectile per attack. It is an enemy projectile (SPEC-0029 Behavior 9): it hurts the hero on contact with the hero's body capsule, and the swing-block rule of melee strikes does not apply to it. The book is hittable as the Biting Spellbook is; the first hit in state 0 sends it to agro with the damage doubled.

#### Defence and counters

As the Biting Spellbook: melee dodge 29 of 100, missile dodge a 29-in-100 roll on every tick a shot is on course within the 180-tick look-ahead. In the lesson the book is in state 0 until the first hit (no defence runs there), so the first shot always lands (doubled); after that it is as the Biting Spellbook: about 5 landed hits and 30 magic for its 35 health with a fast press rhythm (3.5 s), more when the rhythm slips. Counter code 2 at chance 50: after a dodge of a Tor swing the book fires a counter shot if the hero is farther than 10 units (the band), which is rare because the dodge needs the hero to be in melee reach.

#### Hero actions against this enemy

Leave it alone and it never wakes; hit it once and it fights, firing a 30-damage shot every 100 to 250 ticks while the hero is more than 10 units away. The shot travels at 2.0 units per tick for 50 ticks, so the hero can step out of its line.

#### Edge cases for implementers

- The one playable book is inert until woken; never start it in agro.
- Its attack band starts at 10 units: with the hero closer than 10 units it cannot shoot. Its strafe band is 3 to 15 with a desired distance of 5.4 to 12.6, so it shoots mainly while it closes in from beyond 15 units or while it hovers in the 10 to 15 part of its band.
- The projectile pool is shared with every enemy projectile (ten slots, first free); a full pool creates nothing but still counts the shot.

#### What a port most likely gets wrong

1. Waking it by distance or by a notice roll: in state 0 it does neither.
2. Firing from under 10 units.
3. Firing two shots per attack (one window, one shot).
4. Giving it the Biting Spellbook's lunge or backbiter: this book has one missile slot and a backbiter of 1.0.

#### Test vector

The hero lands a hit at tick h: damage doubled, agro through the setter, hit reaction kind 6 (recoil 6.4 units, 37 ticks, then outer 0). The strafe archetype starts after the reaction: approach from 20 units until the distance is under the desired value. For the hero standing 20 units away: the first choice can come on the first tick after the reaction at 49 percent per tick; with the choice at c, the sequencer starts in state 0 and goes to state 1 on c + 1, B is requested at c + 1 (s), the shot is created on s + 12 (c + 13), flies 2.0 units per tick and hits a stationary hero 20 units away on s + 21 (c + 22; its tenth move, counting the move of the release tick; Distances table) for 30 damage; the B gate passes on s + 43 (c + 44), the end path runs on c + 45, recovery 30 to 60, cooldown int[60, 180] drawn again at the end path. Over 600 ticks: 3 to 5 shots (about one per 160 ticks).

#### Open questions

- Whether any level script puts the Shooting Spellbook into agro. Resolved (High; proactive (open close-out)): none does; the only module call that hands an enemy to the alert state is made by other packs than the Library's (SPEC-0038 Behavior 40), so the book stays in state 0 until hit.
- The lone placement's purpose. Unknowable (developer intent is not encoded anywhere); behave as above.


## Crypt Spirits and Knights

Four kinds: the Restless Spirit (monster type 86, row 222), the Skull Spirit (type 89, row 228), the Spirit Sword (type 64, variants 1 and 2, rows 140 and 141) and the Death Knight (type 65, row 143). All are class-17 common enemies and follow SPEC-0038 (think loop, pre-step, notice check, attack chooser, sequencer, counter choice, take-hit hook, death hook). They live in the crypts and ruins of the late acts: Ancient Castle Spires, Dungeon Ruins, Ruined Caves, Knights Crypts, Crypt of the Sentries, Crypt of the Fallen and Black Keep Dungeon. Confidence: High for row values, clips, gate ticks, attack arithmetic, start states and the ghost rule; Medium-High for the loop drain, the charge and the flyer movement; Medium for weapon contact geometry, the lookup-miss movement of flyers and the hero's reaction to a drain hit.

Five warnings before the details.
1. All four rows have Display in default 1. None of them uses the hidden minion chain (state 21, decloak clip 82, state 23): they are drawn and hittable in every state except the two cases below, and a noticed enemy goes through the ordinary state 2 search (look clip, notice repeated, alert clip). The exceptions are the defence states (outer 5 and 6) and a waiting state-8 enemy whose cue word is 0 (see "Waiting in state 8").
2. The Restless Spirit and the Death Knight are ghosts (SPEC-0029 Behavior 33). A Restless Spirit is in its ghost phase for 560 of every 621 to 650 ticks, a Death Knight for 120 to 240 of every 480 to 720. A hit in the ghost phase deals nothing and only reveals the enemy for 10 to 30 (Restless) or 60 to 180 (Death Knight) ticks.
3. The Restless Spirit's main attack (slot 0) is not a swing. It is a drain loop: it hovers within 5 units and deals 10 damage every 61 to 121 ticks for as long as the hero stays near, ignoring the hero's shield and without staggering the hero.
4. The Death Knight has no clips for its dodge ids (138, 139, 140). Every dodge plays clip 0 of its table (a 20-key hover loop, 79 ticks), during which the knight cannot be hit.
5. The Spirit Swords use the armoured-knight rig of the Dark Knight (Minion Corps B) but their own table: they have recovery clips the Dark Knight lacks, no slot 3 (row 140) or a slot 3 that can never be chosen (row 141), and row 141 blocks.

All times are ticks (60 per second). "c" is the tick on which an attack is chosen, "s" the tick on which the sequencer requests its first clip (s = c + 1 for a slot whose first phase needs no approach), and "s + j" means j ticks after a clip request, the tick on which a gate or an event window is seen by the behaviour step (SPEC-0038 Behavior 13). "Gate" is the tick j after the request at which clip progress reaches 0.99 (SPEC-0005).

### Placements and start states (in play, all four kinds)

Counts are placements the player can meet (confirmed or plausibly loaded); cut placements in the unreached test block are left out (Spirit Sword 2, Death Knight 1). Default-state words 0 and 9 do not occur on these kinds.

| Kind | Placements | State 5 (wander) | State 6 (patrol) | State 7 (sentry) | State 8 (hidden) | Main places |
|---|---|---|---|---|---|---|
| Restless Spirit | 59 (4 reserved for a breakable) | 3 | 8 | 14 | 34 | Ancient Castle Spires 28, Dungeon Ruins 7, an unnamed pack 5, Black Keep Dungeon 4, Ruined Caves 4, Crypt of the Sentries 4, Knights Crypts 3, Crypt of the Fallen 2, Black Keep Grounds 1, Werelupe Graveyard 1 |
| Skull Spirit | 12 | 4 | 1 | 0 | 7 | Ruined Caves 6, Ancient Castle Spires 4, Crypt of the Sentries 2 |
| Spirit Sword | 16 (row 140: 12, row 141: 4) | 0 | 0 | 5 | 11 | Knights Crypts 9, Crypt of the Sentries 4, Ancient Castle Spires 2, Dungeon Ruins 1 |
| Death Knight | 6 | 0 | 0 | 2 | 4 | Knights Crypts 3, Ancient Castle Spires 2, Crypt of the Fallen 1 |

Fourteen Restless Spirit records (13 sentries in three Ancient Castle Spires packs and one patrol in Black Keep Grounds) sit in packs that are brought in by the area loader rather than by a confirmed slot load (Medium). The four reserved Restless Spirit records (summon byte 1, spawn key 1816; three of them in state 8 and one a sentry) are never streamed in. They are released by breakables with spawn key 1816: one in Ruined Caves (one reserved record), one in Crypt of the Fallen (one) and one in Crypt of the Sentries (two reserved records, one breakable, so only the first can be released by it). A released spirit starts in agro (state 19) with animation 82 requested once, with the ghost state reset (SPEC-0029 Behavior 31, source 4).

Alarm groups (spawn byte +0xB7, 255 = none): two Restless Spirits in Black Keep Dungeon share group 10 (both hidden, emerge distance 10); three Death Knights in Knights Crypts share group 1 (all hidden, emerge distances 1.0, 10.0 and 1.0). Every other record has no group. A group number only means something inside its area. Drop ordinal (+0xB0) is -1 (no placement drop) on 41 of 59 Restless Spirits, 4 of 12 Skull Spirits, 3 of 16 Spirit Swords and 2 of 6 Death Knights. None of the four rows steals.

Spawn record values that drive the start states (record offsets, SPEC-0006; meanings in the field rules of the Road and Ridge Brigands and Minion Corps A sections):
- State 5 wander: side R at +0xBC, pause range +0xC0 to +0xC4, pause percent +0xC8. Restless Spirit: R 60 with pause range 0 to 30 at chance 0 (never pauses), R 40 (no pause), R 32 with pause 30 to 180 at 20 percent. Skull Spirit: all four R 1000 with pause range 10 to 10 at 10 percent (a very wide roaming square: a random point in a 1000-unit square around home, pulled halfway toward the current position and retried up to 10 times while off the mesh).
- State 6 patrol: path index +0xB8, loop mode +0xBC (0 loop, other back and forth), point pause +0xC4 to +0xC8, random-stop chance +0xCC and stop range +0xD0 to +0xD4. Restless Spirit: Dungeon Ruins paths 0, 1, 2, 3 back and forth with point pauses 60 to 180, 0 to 180, 60 to 180, 0 to 120 and random-stop chances 0, 50, 0, 20 (stop range 0 to 0, so a random stop lasts one tick); Ruined Caves path 2 back and forth, pause 120 to 240, stop chance 100; Ancient Castle Spires paths 0 and 2 (loop) and Black Keep Grounds path 8 (loop), no pauses. Skull Spirit: Ruined Caves path 3 back and forth, no pauses.
- State 7 sentry: +0xC4 zero on 13 Restless Spirits and all 5 Spirit Swords (they play the look and stand clips and never turn); nonzero on one Restless Spirit and both Death Knights (they hold the placed facing).
- State 8 hidden: emerge distance +0xC0, cue word +0xC4, selector +0xC8 (0 on every record, so a ground emerge uses id 10). Restless Spirit (31 streamed records, all cue word 0): 15 (12), 20 (3), 10 (4), 9 (2), 7 (3), 2 (2), 4 (1), 40 (1), 0 (3, never wakes by distance). Skull Spirit: 20 (4, cue 1), 10 (1, cue 1), 20 (1, cue 0), 16 (1, cue 0). Spirit Sword row 140: 20 (2, cue 0), 5, 5, 8, 8 (cue 1), 0 (2, cue 0); row 141: 4 (cue 1), 10 (cue 1), 0 (cue 0). Death Knight: 1.0 (2), 10.0 and 4.0, all cue word 0.

### Waiting in state 8 and the cue word

A waiting state-8 enemy (class state 8, emerge sub-state 0) stands at home playing the hide pick (id 9 for all four kinds: Restless clip 25, Skull clip 5, Spirit Sword clip 19, Death Knight clip 24). The hidden predicate for a Display-1 row says it is hidden (not drawn, not hittable, not auto-targeted) while its cue word is 0 and the hide pick is current. So:
- Restless Spirit (31 streamed records, cue 0), Death Knight (4, cue 0), Skull Spirit (2, cue 0) and Spirit Sword (5 of 11, cue 0) are invisible and untouchable until they wake. They wake when the hero comes closer than the emerge distance times F1 (0.7 standing, 0.75 slow walk, 1.5 walk, 1.75 run) or when an alarm-group member enters agro (up to 45 ticks later). A distance of 0 never wakes.
- Skull Spirit (5 of 7, cue 1) and Spirit Sword (6 of 11, cue 1) are drawn and hittable while they wait; a hit wakes them with doubled damage.
- The emerge clip then runs: Death Knight id 165 (clip 27, 25 keys, gate 96) and Skull Spirit id 10 (clip 4, 30 keys, gate 116) play to their gate and go to agro. The Restless Spirit has no id 165 and the Spirit Sword no id 10: a missing emerge clip goes to agro at once, so these two appear in the middle of the room already fighting (the first tick of agro is the tick after the wake test).
- The alarm-emerge cue is the shared stinger 4 only (SPEC-0009, Stinger and emote-clip playback), and it starts only when the group alarm wakes the enemy and the cue word is 0 (SPEC-0038 Behavior 3); it starts no camera, and a wake by distance never starts it.

### Shared field rules (start, wake, return)

The start, wake and return rules are the settled rules of the Road and Ridge Brigands and Minion Corps A sections, with this batch's numbers:
1. Start. Default word 8 enters state 8; default 5, 6 and 7 are entered on the first tick through state 0 (the setter's default case gives state 0 for a Display-1 row, and the pre-step runs the default state).
2. States 5, 6, 7. The notice check runs on one tick of each awareness cycle (timer + 2 ticks). A pass moves the enemy to state 2: it stops, turns to the last-seen point, requests the look clip (flyer id 147, ground id 16), repeats the check on the same tick and then once per cycle. A pass while the alert clip is not playing requests the alert clip (flyer 149, ground 18) and the enemy goes to agro when it passes its 0.99 gate; a table without the alert id goes to agro on the pass. With no pass the search countdown float[search min, search max) runs (only while the alert clip is not playing); at zero the lose-interest clip (flyer 148, ground 17) plays to its gate and the enemy runs the setter with 0 (state 0, then the default state); a table without the lose-interest id runs the setter with 0 at once.
3. Group alarm. A group member entering agro broadcasts; members in states 0, 3, 5, 6, 7 and 14 go to state 2 within int[15, 45] ticks and request the alert id directly (no second notice needed); members in state 8 emerge. Where the alert id has no clip (Spirit Sword), the request plays clip 0 of the table and the handler waits for that clip's gate (below).
4. A hit at any time is a normal hit (doubled damage once, agro, a kind-0 or kind-6 reaction). A hit on a Skull Spirit or Spirit Sword in state 8 with cue word 1 wakes it the same way.
5. Return. The three exits from agro (leash: no route and the walk waypoint 15.0 or more from the hero for more than 180 ticks; lost-sight search above 120; victory pose) run the setter with 0 and the enemy resumes its default state: 5 wanders on from where it stands, 6 resumes its patrol, 7 walks back to its post, 8 walks home and hides again (and the whole wake sequence repeats).
6. Socialize applies only to the Skull Spirit (row socialize flag 1; see that section).
7. The victory pose (a strike that leaves the hero at 0 HP): only the Spirit Sword has the clip (id 73, clip 24, 38 keys, gate 148); for the other three the state becomes 0 at once.

Notice numbers (the chance is interpolated linearly between the near and far chance over the unscaled range, halved outside the view cone, and roll B must be below it; roll A is the awareness value):

| Item | Restless Spirit | Skull Spirit | Spirit Sword (140, 141) | Death Knight |
|---|---|---|---|---|
| Awareness (roll A at most) | 100 | 75 | 100 | 90 |
| Awareness timer, cycle length | 60, 62 ticks | 30, 32 | 10, 12 | 60, 62 |
| View cosine | 0.0 (the front half) | 0.92388 | 0.92388 | 0.707 |
| Range near, far (unscaled) | 3.5, 50 | 3.5, 50 | 10, 50 | 1.0, 10 |
| Chance at near, at far | 70, 50 | 70, 10 | 50, 40 | 1, 90 |
| Sees through walls | yes | no | no | yes |
| Standing hero (F1 0.7): always noticed inside, never beyond | 2.45, 35 | 2.45, 35 | 7, 35 | 0.7, 7 |
| Walking hero (F1 1.5) | 5.25, 75 | 5.25, 75 | 15, 75 | 1.5, 15 |
| Running hero (F1 1.75) | 6.1, 87.5 | 6.1, 87.5 | 17.5, 87.5 | 1.75, 17.5 |
| Search countdown | float[30, 120) | float[60, 240) | float[180, 300) | float[60, 240) |
| Look clip | 147 = clip 33 (45 keys), gate 176 | 16 = clip 10 (68 keys), gate 268 | 16 = clip 21 (40 keys), gate 156 | 147 = clip 12 (45 keys), gate 176 |
| Lose-interest clip | 148 absent: setter at once | 17 absent: setter at once | 17 = clip 29 (23 keys), gate 89 | 148 = clip 16 (33 keys), gate 128 |
| Alert clip | 149 = clip 37 (40 keys), gate 156 | 18 = clip 0 (30 keys), gate 116 | 18 absent: agro on the pass | 149 = clip 7 (30 keys), gate 116 |

One notice check passes with probability P(roll A) x P(roll B below the chance). Per awareness cycle, for a still hero in the cone: Restless Spirit 1.00 inside 2.45, then 0.70 at 3, 0.67 at 10, 0.62 at 20, 0.56 at 35 and 0 beyond 35; Skull Spirit 0.75 inside 2.45, 0.46 at 10, 0.36 at 20, 0.22 at 35; Spirit Sword 1.00 inside 7, 0.49 at 10 to 12, 0.47 at 20, 0.43 at 35; Death Knight 0.90 inside 0.7, 0.18 at 3, 0.36 at 5, 0.54 at 7 and 0 beyond 7. A walking or running hero is noticed much farther away; a Death Knight's chance interpolation passes 100 beyond 10 units, so a hero who walks or runs to within the scaled far range (15 or 17.5) is noticed whenever roll A passes (0.9).

Flyer rules (Restless Spirit and Death Knight: Can fly 2, always airborne). They use the flyer ids and height rule of the Minion Corps B section. A flyer moves by the root travel of its clip plus the row speed of 2 x the flying idle velocity (outside agro) or 2 x the flying agro velocity (in agro) along its forward axis (SPEC-0040 Behavior 14D item 7): Restless Spirit 2.0 and 4.0 units per second, Death Knight 4.0 and 14.0. The goal height is set whenever a walk target is set. Corrected (movement pass; High for the rule, Medium for how it looks): these are Can Fly 2 rows, so the goal height is the hero's own height (the home point's height while the enemy waits in state 8), lowered to at most the row's maximum fly height above the ground under the goal (Restless Spirit 5.0, Death Knight 100.0), with no lower bound, no 0.2 x distance term and no hero-height-plus-2.0 rule in the charge states (those three belong to Can Fly 1 rows only). The keeper moves the body at most 0.1 per tick toward it. Gravity is not switched off for Can Fly 2 rows, so in the air the fall soon outweighs the keeper's 0.1 per tick and the body settles within a few tenths of a unit of the ground, bobbing, unless the ground under it is above the goal (SPEC-0040 Behavior 14D item 7). On flat ground with the hero these two kinds therefore move at the hero's foot level, not above him. They are not held back by the 2.0 edge guard (it skips Can Fly 2 rows); the flying mesh border still confines them, and a body that crosses a drop within that mesh falls under gravity. The flyer hit-reaction kind is 6 for both (flying hit word 0): animation 152, then outer 0. Death plays 153 and the corpse does not sink. Every attack slot of these two rows has trigger special 0 (no airborne limit), so the airborne test of the attack choice (which is the Can-fly-1 test only: special 1 never passes and special 2 always passes for a Can-fly-2 row) does not matter here. Flyers pick only flying-kind navmeshes for their current mesh, and kind 7 (which grounds Can-fly-1 flyers) is not used by these rows.

A lookup miss (a requested id the table lacks, outside the attack phases and fidgets that are checked first) plays clip 0 of the table's main set, not the idle: Restless Spirit and Death Knight clip 0 is a 20-key loop with no root travel (gate 79); Skull Spirit clip 0 is a 30-key one-shot (gate 116); Spirit Sword clip 0 is a 15-key loop that carries the knight forward at 10.0 units per second (gate 59).

Ghost rule on this batch (SPEC-0029 Behavior 33 has the full per-tick rule). The "attacking" test that makes a ghost solid and targetable is true while the class state is agro, the outer sub-state is 2 (an attack) and the sequencer state is 0, 1, 2 or 3, and, as a separate test, while the sequencer state is 14 (the loop strike). It is false in the sequencer states 4 to 13 and 15, so a charging Death Knight (states 6 to 9, 12) or a Restless Spirit that is approaching for its drain (state 13) or ending it (state 15) is not attacking and keeps cycling through its ghost phase. A hit is absorbed only while the solid flag is 0 (the ghost phase, including its fade-out; the solid fade-in already has the flag set) and no reveal runs.

### Restless Spirit (row 222)

Monster type 86 variant 0, name "Restless Spirit"; one table of animation ids and clips (the 43-clip ghost rig, shared with the Death Knight) for all 59 records. A small flying ghost with 75 hit points that hovers beside the hero and drains him. It has two usable attack slots, a swipe and a drain loop; the other two slots are dead. Confidence: High for the row, the table, the start states and the ghost cycle; Medium-High for the drain loop; Medium for the movement of its lookup-miss gaits and for the hero's reaction to a drain hit.

#### Identity and tunables

| Item | Value |
|---|---|
| Class, approach | Class 17; approach 1 (rush); Can fly 2; flying idle and agro velocity 1.0 and 2.0 (2.0 and 4.0 units per second); maximum fly height 5.0 |
| Hit points, body | 75; radius 1.5, height 3.0; defence value 0.0; attack-manager weight 5 (it can wait in line, stand-off radius 10.0); hostile; Display in default 1; element T 6 |
| Notice | Awareness 100, timer 60 (cycle 62), view cosine 0.0, range 3.5 to 50, chance 70 at 3.5 and 50 at 50, sees through walls; search countdown float[30, 120) (table in the shared rules) |
| Turn | Flyer aim weight 0.7 (ground weight 1.0) |
| Rush band | Rush minimum 3.0, maximum 5.0: desired distance 3 + 2 f (3.4 to 4.6, f from 0.2 to 0.8), redrawn after every normal attack choice. Hold between 3.0 and 5.0; below 3.0 back off toward the point 7.0 from the hero; above 5.0 approach |
| Recovery after an attack | int[15, 60] ticks |
| Missile defence | Hold 120, dodge 100 (99 of 100 per shot), block 0 |
| Melee defence | Dodge 100 for quick, normal and heavy swings (99 of 100), block 0; defence timer int[30, 60] (the row stores 60 first and 30 second; the draw takes either order); Blocking Angle 0 (never guards) |
| Hit reaction | Flyer kind 6: animation 152 = clip 35 (12 keys, gate 45, pushed back at 16.6 units per second), then outer 0 |
| Death | Animation 153 = clip 30 (44 keys, gate 172); the corpse is gone 10 ticks after the clip reaches progress 0.95 (about 175 ticks after death); it never sinks (flyer) |
| Ghost (row words) | Solid phase int[1, 30], ghost phase int[500, 500], opacity 0.0 to 0.5 (alpha at most 64), reveal int[10, 30] |
| Anger, steal, socialize, map search | None (anger 0, no steal flag, socialize flag 0, map search flag 0) |
| Weapon volumes | Weapon 1 radius 7.0, height 5.0; weapons 2 to 5 none |
| Other | No footstep shake or rumble. The combat timer float[600, 900) is drawn on entering agro; no rule in this section reads it |

Attack slots. Slots are tested from the highest slot down, so slot 3, then 2, 1, 0.

| Slot | Role | Band (strict), chance, arc | Range, cooldown, orient | Damage and effect | Counter | Clips (phase ids 33 + 5 k + p) |
|---|---|---|---|---|---|---|
| 3 | Dead slot (type melee, band 0 to 0, chance 0) | never passes | | 0 | none | all five phases absent |
| 2 | Unused (type 0xFFFF) | | | | | |
| 1 | Swipe | 0 to 15, 50 (49 of 100), arc 0.966 (15 degrees) | melee range 6.0, cooldown int[60, 180], orient 1 | 60, unblockable (flag 0x1), knockback 3.0, slow (status 3) at 10 (10 of 101 draws) for 1 to 3 s | trigger 2 (after a dodge), chance 25 | A absent; B 39 = clip 29 (30 keys, played at rate 3.0, gate 39 at rate 3.0, weapon 1 live s + 26 to s + 29); C, D, E absent |
| 0 | Drain (loop slot, loopable word 1) | 0 to 15, 50 (49 of 100), arc 0.707 | melee range 4.0, loop exit distance 5.0, cooldown int[0, 0], orient 1 | 10 per drain hit (direct damage, below); flags 0x21 | none | A 33 = clip 28 (26 keys, loops; never requested by the loop); B 34 = clip 40 (24 keys, loops); E 37 = clip 34 (40 keys, gate 156); C, D absent |

Speed words: slot 1 wind-up 3.0, strike 3.0, recovery 1.0; slot 0 all 1.0. Knockback on the hero from slot 1 is 3.0 (per-step velocity, so 1.8 horizontal and 2.4 vertical; the hero goes to the long-fall state). A swipe's rate is 3.0 only in sequencer state 2; the follow-through state 3 plays at 1.0.

#### Animation ids and clips (flyer ids)

| Id | Use | Clip |
|---|---|---|
| 141 | agro idle pick (set-up, stands in agro) | clip 25 (30 keys, loops) |
| 0 | idle pick outside agro | clip 36 (45 keys, loops); 97 absent |
| 101 | rush approach, loop-attack pursuit | clip 28 (26 keys, loops, mode-0 clip, 2.56 per second forward root travel; with the row speed 6.56 per second) |
| 102 | back off | clip 26 (15 keys, loops, -12.57 per second; with the row speed -16.57 per second) |
| 155 | rush hold | clip 25 (30 keys, loops) |
| 98, 99, 100, 154 | wander, patrol, named-target walk, timed approach | absent: clip 0 (20 keys, loops, no root travel); the row speed alone moves it (2.0 per second outside agro) |
| 144, 145 | sentry stand, sentry fidget | absent: clip 0 |
| 3, 6 | wander pause fidget, patrol pause fidget | clip 34 (40 keys, gate 156) |
| 142 | patrol hold fidget | clip 12 (45 keys, gate 176) |
| 138, 139 | missile dodge (right, left) | clip 31 and clip 32 (12 keys, gate 45, sideways 8.9 per second) |
| 140 | melee dodge | clip 38 (13 keys, gate 49) |
| 152 | hit reaction kind 6 | clip 35 (12 keys, gate 45) |
| 153 | death | clip 30 (44 keys, gate 172) |
| 147, 148, 149 | look, lose interest, alert | clip 33 (gate 176), absent, clip 37 (gate 156, net +5.4 per second forward over the clip) |
| 165, 163, 73 | emerge, victory pose | absent (emerge goes to agro at once; the victory state becomes 0) |
| 82, 10 | breakable appear | clip 27 (25 keys, gate 96), requested once |
| 69 | confused | clip 41 (40 keys, loops, 4.5 per second) |
| 81 | state 23 clip | clip 38 (not used: Display is 1) |

#### Start, wake, return

Start states and the wake sequence are the shared rules above with the Restless column of the notice table.
1. Default 5 (3 records), 6 (8) and 7 (14) enter their idle state on the first tick. The wander, patrol and sentry gaits use ids that this table lacks (98, 99, 144, 145), so the spirit drifts at the row speed of 2.0 units per second while the hover loop (clip 0) plays; pauses and fidgets use id 3 or 6 (clip 34, a 156-tick clip). A sentry plays clip 0 and never turns unless its +0xC4 word is nonzero.
2. A pass of the notice check enters state 2 (search countdown float[30, 120)): look clip 147 (176 ticks), second check on the same tick and then once per 62-tick cycle. A second pass requests the alert clip 149 (156 ticks) and the spirit enters agro when it passes its gate. Because the search countdown (30 to 119) is usually shorter than two cycles, the second check is mostly the same-tick repeat and one cycle tick: for a still hero 10 units away the chance of reaching agro after a first pass is about 0.67 + 0.33 x 0.63 x 0.67 = 0.81. If the countdown ends with no second pass, the lose-interest clip is absent, so the setter runs with 0 at once (state 0, then the default state).
3. Waiting in state 8 (31 streamed records, all cue word 0): invisible and untouchable until the hero is closer than the emerge distance x F1 (distances 15, 20, 10, 9, 7, 2, 4, 40; 0 never). It then enters agro on the next tick with no appear animation (id 165 is absent). The two Black Keep Dungeon records of group 10 wake together within 45 ticks.
4. A released spirit (breakable key 1816) starts in agro with animation 82 requested once (clip 27, 25 keys); the agro idle pick replaces it at the next set-up tick.
5. A hit at any time wakes it into agro with doubled damage and a kind-6 reaction, unless the ghost rule absorbs it (a ghost-phase hit only starts a reveal).
6. Return after losing the hero: the shared exits; the lost-sight search and the leash apply as usual (the spirit sees through walls for noticing, but the agro tail's route test still applies).

#### Per-tick order (agro) and state table

Standard agro order (SPEC-0038 Behavior 44): recovery countdown, missile defence, melee defence, lost-sight test, normal attack choice, counter choice, distance-hide gate (off), outer switch, leash. The ghost update runs before the character's AI step each tick (SPEC-0029 Behavior 33).

| State | Action id and clip | Keys | Gate and exits |
|---|---|---|---|
| Set-up (outer 0) | 141 | 30 | one tick, then outer 1 inner 1 (hold) |
| Approach (inner 0) | 101 -> clip 28 | 26, loops | 6.56 per second (0.109 per tick) toward the hero; d < desired (3.4 to 4.6) gives hold |
| Hold (inner 1) | 155 -> clip 25 | 30, loops | faces the hero at weight 0.7; d < 3.0 gives back off; d > 5.0 gives approach |
| Back off (inner 2) | 102 -> clip 26 | 15, loops | 16.57 per second backward toward the point 7.0 from the hero; ends when d > desired |
| Swipe (slot 1) | 39 -> clip 29 | 30 | s = c + 1; hit at s + 26; with a hit: B gate s + 62, end path s + 63; with no contact: B gate s + 39, end path s + 40 |
| Drain approach (state 13) | 101 -> clip 28 | 26, loops | pursues while d > 4.0 (horizontal); at d <= 4.0 goes to state 14 with the countdown 60 |
| Drain loop (state 14) | 34 -> clip 40 | 24, loops | first damage on tick c' + 61 after state 14 was entered at c' (below); leaves to state 15 when d > 5.0 |
| Drain end (state 15) | 37 -> clip 34 | 40 | gate 156, then the end path |
| Melee dodge (outer 5) | 140 -> clip 38 | 13 | gate 49, then the defence exit (outer 0, hold int[30, 60]) |
| Missile dodge (outer 6) | 138 or 139 -> clip 31 / 32 | 12 | gate 45, then the defence exit |
| Hit reaction kind 6 (outer 10) | 152 -> clip 35 | 12 | gate 45 after the last hit, then outer 0 |
| Wait in line (outer 12) | 98 absent: clip 0 | 20, loops | stand-off point 10.0 from the hero (SPEC-0038 Behavior 45) |
| Victory pose | 163 absent | | state 0 |
| Dead | 153 -> clip 30 | 44 | plays once |

#### Attack chooser

Each tick while the hero is attackable, the recovery countdown is below 1 and the outer sub-state is 1, 3, 4 or 11: slot 1 (cooldown below 1, 0 < d < 15, facing cosine above 0.966, roll below 50, route to the hero or d < 10), then slot 0 (0 < d < 15, facing above 0.707, roll below 50, same route rule). Slot 3 and slot 2 never pass. With the spirit facing the hero within 15 degrees and slot 1 off cooldown, the per-tick choice is slot 1 49 percent, slot 0 25.0 percent (0.51 x 0.49), nothing 26 percent; with slot 1 on cooldown, slot 0 49 percent. After a choice the slot cooldown is drawn (slot 1: int[60, 180], redrawn at the end path) and the desired distance is redrawn. A choice cannot be made before the end path has run, the recovery countdown (int[15, 60]) has run out and one tick of outer 0 has passed.

The drain loop (slot 0 chosen at tick c): the sequencer starts in state 13 (loopable slot). Tick c + 1: d at most 4.0 sends it to state 14 with the countdown 60 (otherwise it pursues at 6.56 per second until it is within 4.0). From tick c + 2 each tick of state 14 plays clip 40; when the countdown is 0 the hero takes the drain hit and the countdown is redrawn int[60, 120]; otherwise it falls by 1. So the first drain lands on tick c + 62 and each later one 61 to 121 ticks after the previous one. When the horizontal distance exceeds 5.0, state 15 plays clip 34 (156 ticks) and the end path follows. Nothing else ends the loop: it does not time out, the hero's guard does not stop it, and the spirit keeps hovering through the hero's retreat as long as the hero stays within 5.0 (it does not chase in state 14).

Drain hit rule (Medium-High). The damage is the enemy's attack damage to a hero (SPEC-0036: the slot damage 10, the hero's armour, the enemy's own curse and strength scale, no shield block, truncated) and it goes to the hero's take-hit with the blocked flag set (slot flag 0x20), so the hero shows the block reaction instead of the hurt state, keeps control unless he is in a busy state, takes the full damage, and receives no knockback (slot 0 knockback is 0). The hit list, the guard and the direction of the hero are not tested. Slot flag 0x01 (unblockable) has no effect on the loop.

#### Movement

Rush archetype with the flyer gaits: approach 6.56 per second, back off 16.57 per second, hold facing the hero at weight 0.7. Corrected (movement pass): the spirit is a Can Fly 2 row, so its goal height is the hero's own height, at most 5.0 above the ground, and with gravity acting it stays near the ground (the flyer paragraph above); it does not hover 0.2 times the distance above the hero. It does not chase in state 14.

Movement table (movement pass; speed byte 2; Can Fly 2 throughout, so only flyer ids are requested):

| Mode | Source | u/t | u/s | Turn | Clip (id: clip, keys, loop) | Exits |
|---|---|---|---|---|---|---|
| Wander, patrol, walk home (states 5, 6, 7, 8) | row speed only (idle 1.0) | 0.0333 | 2.0 | steering 0.3 / 0.1 | 98, 99, 100 missing: clip 0, 20-key loop, no root (the requested id counts as current, so the row speed applies) | goal or point reached |
| Rush approach (inner 0) and loop approach (sequencer 13) | root + row speed (agro 2.0) | 0.0427 + 0.0667 = 0.110 | 6.56 | steering | 101: clip 28, 26, loop (frame-stored root; shared Movement rules, item 3) | d below desired (3.4 to 4.6) or 4.0 for the loop |
| Rush hold (inner 1) | none | 0 | 0 | aim 0.7 (flying weight) | 155: clip 25 | d below 3.0 or above 5.0 |
| Back off (inner 2) | root + row speed | 0.2095 + 0.0667 = 0.276 | 16.57 | faces the hero | 102: clip 26, 15, loop | d above desired; point 7.0 from the hero |
| Vertical | keeper 0.1 per tick and gravity | - | - | - | - | goal = hero y, at most ground + 5.0 |
| Kind-6 hit reaction | root backward | 13.24 units over 45 ticks | 16.6 | snapped to the attacker | 152: clip 35, 12 | gate |
| Missile dodges 138, 139 | root sideways | 7.1 units over the clip's 48 ticks (gate 45) | 8.9 | none | clips 31, 32, 12 | gate |
| Melee dodge 140 | none | 0 | 0 | none | clip 38 | gate |
| Death | none (gravity) | 0 | 0 | none | 153: clip 30 | removal |

Worked example (follow). A hero walking straight away at 13.51 u/s from a spirit that is 4.0 away: the spirit approaches at 6.56 u/s, so the gap grows by 6.95 u/s; after 2 seconds the hero is about 17.9 away. With a route the leash never counts; the fight ends only through the lost-sight search, or never on open ground.

What a port most likely gets wrong (movement): decoding its approach clip from the wrong root table (it is a frame-stored clip; read wrongly it drifts sideways and upward); floating it above the hero (Can Fly 2 keeps gravity and aims at the hero's own height); moving it at 4.0 u/s while it patrols (2.0 u/s, the idle row speed with a clip that has no travel). It out-runs nothing: the hero (13.5 walking) leaves its drain range in a few ticks, and the approach (6.56) is slower than a walking hero, so a hero who keeps walking away is followed at half his speed until the leash (181 ticks without a route while the walk waypoint is 15.0 or more from the hero) or the lost-sight search ends the fight.

#### Hit and hurt windows

- Swipe: weapon 1 (radius 7.0, height 5.0) is live from s + 26 to s + 29 at rate 3.0, one hit of 60 per window; the clip's forward lunge (net about 2.5 units in its second half) carries the spirit toward the hero. Status 3 (slow) on a hit at 10 (a draw int[0, 100] below 10) for 1 to 3 seconds; knockback 3.0.
- Drain: no volume; the hero must be within 5.0 horizontally of the spirit when the countdown reaches 0 (the loop exit test runs after the damage in the same tick).
- Hurt shapes (Medium): body capsule of length 3.55 radius 0.96 and limb capsules of radius 0.3 to 0.7. The spirit can be hit in every state except its two dodges (outer 5 and 6), while dead, while hidden in state 8 with cue word 0, and in the ghost phase (the hit is absorbed, below).

#### Weapon reach

Conventions are those of "Weapon reach tables" in SPEC-0064 (shared field behaviour). Row 222, Medium. The tick text above is authoritative. Slot 0 (the drain loop) has no weapon test.

| Slot (phase, clip) | Weapon | Window ticks (level contact) | Centre forward / up / side | Tor ahead, at the tick | Tor ahead, from the start | Tor feet vs origin |
|---|---|---|---|---|---|---|
| 1 swipe (B, clip 29) | w1 (7.0 x 5.0) | s+26 to s+29 (s+26 to s+29) | -0.4 to 4.1 / 2.1 to 5.9 / -3.1 to 2.8 | -6.2 to 11.4 | -4.8 to 14.3 | -5.5 to 9.0 |

A level hero is touched from 6.2 behind to 11.4 ahead of the spirit at the tick (14.3 from where it started, after its own 2.5 of forward travel). The hero's feet may be from 5.5 below to 9 above the spirit's origin.

#### Ghost cycle

Per tick (SPEC-0029 Behavior 33): reset gives solid, phase countdown -1, fade -1. Tick 1 draws the solid length int[1, 30]. At its end the ghost phase starts: a 60-tick fade-out (opacity 0.5 to 0.0, hits absorbed from the first tick), then the ghost length int[500, 500] is drawn, so the ghost phase lasts 560 ticks. Then the solid phase starts: a 60-tick fade-in (hits are real from the first tick, auto-targeting only after the fade), and then a solid length int[1, 30] again. One cycle is 621 to 650 ticks, of which 61 to 90 ticks are damageable by a swing and the rest absorb.
- A hit in the ghost phase deals no damage, starts no reaction and sets the reveal countdown int[10, 30]. While the reveal runs the spirit takes normal hits and is auto-targetable; at its end the spirit is solid again with its phase countdown reset (the next solid length is drawn on the following tick). A swing that touches a ghost still counts as a hit on the hero's side (SPEC-0036 hit list).
- While attacking (sequencer state 0 to 3 for the swipe, state 14 for the drain) the spirit is solid and targetable and the phase countdown is not touched; after the attack the remaining ghost countdown (up to 500) keeps counting as a solid countdown, so a spirit that has just attacked stays damageable for the rest of that countdown (up to 500 ticks) before its next fade-out. Approach (state 13) and drain end (state 15) are not attacking.
- Opacity: alpha = trunc(128 x opacity), at most 64; the spirit always draws translucent.

#### Defence and counters

Melee: once per hero swing (hold below 1, not in a reaction, not mid-attack except sequencer states 0, 1 and 5, hero within 8.0 and in front within 60 degrees), a dodge on 99 of 100 swings of any class (clip 38, 49 ticks, no root travel), then a hold of 30 to 60 ticks with no further defence. A dodging spirit cannot be hit, so in the ghost phase the first swing of a combo is dodged and does not even reveal; the next swing after the hold is absorbed and reveals. Missile: for each live hero shot whose next 120 ticks of path meet the capsule, 99 of 100 give a dodge (clips 31 or 32, 45 ticks, sidestep about 6.7 units), then the same hold. Counters: slot 1 only, trigger 2 (after a dodge): eligible during outer 5 or 6 while Tor's recover window (table-B event bit 0x800 on any of his clips) is open, with slot 1 off cooldown, 0 < d < 15 and the facing above 0.966; one roll int[0, 100] below 25 per tick (25 of 101). A hit reaction triggers no counter. The drain loop is never a counter.

#### Hero actions against this enemy

| Hero action | Result |
|---|---|
| Stands still within 35 units in front | Noticed within one or two 62-tick cycles (0.56 to 0.70 per check); another 156 ticks of alert clip before the first attack |
| Walks or runs | Noticed up to 75 or 87.5 units, with a chance 0.34 to 0.65 per check |
| Within 5 units of a drain | 10 damage every 61 to 121 ticks, unblockable by shield, no stagger; steps out of range (more than 5.0) end the loop |
| Tor swings while the spirit is in the ghost phase | 99 percent dodged (no effect); the next swing after the 30 to 60 tick hold is absorbed and opens a 10 to 30 tick reveal in which the third swing lands |
| Tor swings while it attacks or is revealed | Normal damage; HP 75, defence value 0, doubled once if the hit wakes it; reaction 45 ticks, cancels the swipe or drain |
| Roberta's shots | Dodged on 99 of 100 shots that meet the capsule inside 120 ticks, absorbed (and reveal) in the ghost phase |
| Guards (L1) | Does not stop the drain (flag 0x20 forces the block reaction but not a block of the damage) or the swipe (unblockable) |
| Alarm group (Black Keep Dungeon) | Group members emerge within 45 ticks |

#### Edge cases for implementers

- The wander, patrol and sentry gaits are lookup misses: the clip is the 20-key hover loop and the speed is the row speed alone. The spirit slides without a walk cycle.
- An emerging spirit has no appear animation; it is simply there, already in agro.
- Slot 3 is a used slot that can never be chosen (band 0 to 0), and slot 2 is unused: a port that reads these as live attacks adds a phantom attack.
- Slot 0 cooldown is int[0, 0]: only the choice roll limits it. Slot 1's cooldown is drawn at the choice and again at the end path (the second draw wins).
- Slot 0 flags 0x21: bit 0x01 has no reader in the loop, bit 0x20 selects the blocked flag of the direct damage. A port that treats bit 0x01 as "damage ignores armour" is wrong.
- State 14 draws its next interval before it applies the damage, and tests the exit distance after: the tick on which the hero has just stepped out still receives the damage if the countdown is 0.
- The sequencer does not aim in state 14, and the chase-stop (no step within the sum of the radii) applies only in sequencer states 0 to 3, so a spirit in state 13 or 14 may drift into the hero's position.
- A hit in outer 10 abandons the loop (outer sub-state leaves 2); the next choice restarts in state 13 and the first drain is again 62 ticks after the choice.
- The reserved records (breakable key 1816) are bound by the breakable, not by the chunk streaming; a second reserved record in the same block stays unbound.

#### What a port most likely gets wrong

1. Treating slot 0 as a normal swing with a strike clip: it is a loop with no hit window; the damage is a timer.
2. Making the drain blockable or interruptible by the hero's guard, or staggering the hero with the hurt state.
3. Letting a ghost-phase hit damage the spirit, or forgetting that the attack makes the spirit solid.
4. Giving the wake a walk-in: the spirit has no emerge clip.
5. Treating waiting spirits as visible and hittable (cue word 0).
6. Giving the missing gait ids a walk speed from the table instead of the row speed.
7. Forgetting the 99 percent dodge, which makes the spirit nearly untouchable by melee while its hold is below 1.

#### Test vector

Restless Spirit in agro (t = 0 the tick the setter ran), hero standing still 4.0 units in front within 15 degrees, guard off, hero not attacking. t = 1 set-up. From t = 2 the per-tick choice is slot 1 49 percent, slot 0 25 percent, else nothing.
- Slot 1 chosen at c: s = c + 1; hit at c + 27 (60 damage, slow at 10 percent for 1 to 3 s, hero knocked into the long-fall state); B gate c + 63; end path c + 64; recovery int[15, 60]; slot 1 cooldown int[60, 180]. If the hero has left the volume: gate c + 40, end path c + 41.
- Slot 0 chosen at c: state 13 on c + 1 (d = 4.0 is within range), state 14 from c + 2; drain hits at c + 62 and then every 61 to 121 ticks (c + 123 to c + 183, ...), each 10 damage.
- Over 600 ticks (20000 simulated runs, hero never moving or guarding, every window landing): mean 0.66 swipes and one drain loop, mean first damage at tick 41, about 5 to 7 drain hits; total damage about 100 (range 50 to 160). The hero's Tor swings are dodged 99 percent of the time while the hold is below 1.

#### Open questions

- Whether the block reaction shown on a drain hit takes the hero's control for any ticks. Resolved (High; proactive (open close-out)): no; a take-hit with the blocked flag set only requests the block animation (SPEC-0036, Taking a hit and Block reaction), writes no hurt or knockback state and no stun, and in the idle state the next idle request cuts the pose on the same tick.
- Whether the drain's exit distance is a horizontal or a 3D distance. Resolved (High; proactive (open close-out)): horizontal; the sequencer's exit test uses the shared distance helper, which drops the height difference.
- The contact geometry of weapon 1 (radius 7.0) for the swipe. Resolved by the general rule (SPEC-0064, Shared field behaviour: Weapon contact and the hero's hurt shape); the distances are in the Weapon reach table; stand-in only for a port without attachment frames: a window hits a level hero from 6.2 behind to 11.4 ahead of the spirit.
- Whether a flyer's row speed is added while a lookup-miss clip plays as its gait. Resolved (High; proactive (open close-out)): yes (2.0 units per second); the flyer step compares the requested animation id with the gait id of its class, not the clip that actually plays, and a lookup miss keeps the requested id.
- Whether the second reserved record of Crypt of the Sentries can be released. Resolved (High; proactive (open close-out)): no; the breakable releases only the first reserved record, as the section above says.
- Whether a distance wake with the missing emerge clip starts the camera cue. Resolved (High; proactive (open close-out)): no; only the group alarm starts the cue, and it is the stinger sound alone (Open question 5 of the Swamp Tentacle).

### Skull Spirit (row 228)

Monster type 89 variant 0, name "Skull Spirit"; one table of animation ids and clips (a small 20-clip ground rig) for all 12 records. A 300-hit-point floating skull that hangs 10 to 30 units from the hero and shoots at him, closing to bite whenever he is within 20 units. It is a ground enemy (Can fly 0). Confidence: High for the row, the table, the projectile numbers and the timings; Medium-High for the order of the cast; Medium for the shot launch point, the bite's contact geometry and the roaming wanderers.

#### Identity and tunables

| Item | Value |
|---|---|
| Class, approach | Class 17; approach 1 (rush); Can fly 0 |
| Hit points, body | 300; radius 2.0, height 3.0; defence value 2.0; attack-manager weight 8 (it can wait in line, stand-off radius 10.0); hostile; Display in default 1; element T 5 |
| Notice | Awareness 75, timer 30 (cycle 32), view cosine 0.92388, range 3.5 to 50, chance 70 at 3.5 and 10 at 50, no line-of-sight flag; search countdown float[60, 240) (table in the shared rules) |
| Turn | Hold aim weight 0.5 |
| Rush band | Rush minimum 10.0, maximum 30.0: desired distance 10 + 20 f (14 to 26), redrawn after every normal attack choice. Hold between 10 and 30; below 10 back off toward the point 32 from the hero; above 30 approach (gait 2 beyond 60, the same id) |
| Recovery after an attack | int[15, 30] ticks |
| Missile defence | Hold 60, dodge 100 (99 of 100 per shot), block 0 |
| Melee defence | Dodge 100, 100, 90 for quick, normal and heavy swings (99, 99 and 89 of 100), block 0; defence timer int[0, 30]; Blocking Angle 0.707 (never guards: block chance 0) |
| Hit reaction | Ground kind 0: animation 57 = clip 14 (18 keys, gate 69, pushed back 14.2 per second, about 16 units), then outer 0 |
| Death | Animation 58 = clip 17 (65 keys, gate 256); the corpse is gone 60 ticks after progress 0.95; it sinks after the first 30 of them |
| Socialize | Flag 1, pair radius 20.0 (separation at most 40), stop within 6.0 (fixed; the row's 5.0 is not read, SPEC-0038 Behavior 10), talk time float[120, 180), quiet countdown float[420, 480); chat ids 14 and 111 to 114 are absent and play the idle pick |
| Other | No steal, no map search, no ghost, no anger. Weapon volumes: all five radius 2.0, height 2.0. Footstep shake class 0, rumble class 0 |

Attack slots (slots tested 3, 2, 1, 0; slot 3 is unused):

| Slot | Role | Band (strict), chance, arc | Range, cooldown, orient | Damage and effect | Clips |
|---|---|---|---|---|---|
| 2 | Bite (melee) | 0 to 20, 60 (59 of 100), arc 0.9239 (22.5 degrees) | melee range 4.0; cooldown int[0, 0]; orient 0 | 30, blockable, knockback 0, no status | A, C, D, E absent; B 44 = clip 11 (25 keys, gate 96; all five weapon volumes live s + 20 to s + 83) |
| 1 | Five-shot volley (missile, straight) | 0 to 50, 40 (39 of 100), arc 0.9239 | cooldown int[30, 120]; orient 0; spawn count 5 | 40 per shot; speed 2.0 per tick, range 50 (lifetime 25), radius 1.0, scatter 0 | B 39 = clip 18 (65 keys, gate 256; five release windows at s + 124, 148, 176, 192, 216); other phases absent |
| 0 | Single shot (missile, straight) | 7 to 40, 100 (99 of 100), arc 0.0 | cooldown int[0, 0]; orient 0; spawn count 1 | 30; speed 1.0 per tick, range 30 (lifetime 30), radius 1.0, scatter 0 | B 34 = clip 12 (25 keys, gate 96; one release window at s + 36); other phases absent |

Speed words are 1.0 on every slot. No counters, no stun, no status, no projectile effect ids (all six are -1), visual kind 0, element 5. Phase ids not listed have no clip and are skipped in the same tick.

#### Animation ids and clips (ground ids)

| Id | Use | Clip |
|---|---|---|
| 1 | agro idle pick, rush hold (22, 25) | clip 1 (25 keys, loops) |
| 0, 7 | idle pick outside agro | clip 8 (40 keys, loops) |
| 21, 24 | rush forward (approach, charge-less pursuit) | clip 3 (25 keys, loops, 6.12 per second) |
| 23, 26, 27 | back off | clip 2 (25 keys, loops, -9.03 per second) |
| 2, 4, 11 | wander, patrol and named-target walk | clip 13 (25 keys, loops, 5.10 per second) |
| 3, 5, 6, 8, 12 | pause fidgets | clip 9 (58 keys, gate 228) |
| 9 | hide pick (state 8) | clip 5 (1 key) |
| 10 | emerge | clip 4 (30 keys, gate 116) |
| 16 | look | clip 10 (68 keys, gate 268) |
| 18 | alert | clip 0 (30 keys, gate 116) |
| 17, 73, 14 | lose interest, victory pose, chat | absent |
| 53, 54 | missile dodge (right, left) | clip 6 and clip 7 (15 keys, gate 57, sideways 10.3 per second) |
| 55 | melee dodge | clip 15 (15 keys, gate 57, backward 10.4 per second) |
| 57 | hit reaction kind 0 | clip 14 (18 keys, gate 69) |
| 58 | death | clip 17 (65 keys, gate 256) |
| 68 | spare (not requested by this row's states) | clip 16 (40 keys) |
| 69 | confused | clip 19 (35 keys, loops, 4.0 per second) |

#### Start, wake, return

Start states and the wake sequence are the shared rules with the Skull column of the notice table.
1. Default 5 (4 records, Ancient Castle Spires), 6 (1, Ruined Caves) and 8 (7). The wanderers pick points in a 1000-unit square around home, so they roam the whole room; they walk at 5.1 per second and pause 10 ticks on 10 percent of arrivals. They socialize (below).
2. A pass of the notice check enters state 2 (search float[60, 240)): look clip 16 (268 ticks, the longest in the batch), second check on the same tick and then once per 32-tick cycle. A second pass requests the alert clip 18 (clip 0, 116 ticks) and the Skull Spirit enters agro when it passes its gate. If the countdown ends with no second pass, the lose-interest clip is absent: the setter runs with 0 at once.
3. Chance of reaching agro: a still hero 15 units away in the cone is noticed with 0.41 per cycle (0.75 x 0.55); at 30 units 0.26; beyond 35 never. A walking hero is noticed to 75 units (0.07 at 50). Sight must be clear: a blocked sight line replaces F1 by F2 (0.0 for a standing hero, 0.25 for a slow walker).
4. Waiting in state 8: 5 of 7 records have cue word 1 and are visible and hittable while they wait (a hit wakes them with doubled damage); 2 (Crypt of the Sentries, emerge distances 20 and 16) have cue word 0 and are invisible until the hero is within 20 or 16 times F1. They play the emerge clip 10 (116 ticks) and enter agro at its gate.
5. Socialize (flag 1): while idle in state 5, 6 or 7 the quiet countdown float[420, 480) runs; at zero the enemy looks for a peer (flag set, not in states 1, 2, 3, 19 or 21, within the sum of the pair radii 40 and with its own quiet countdown below 1; a peer hidden in state 8 qualifies and is pulled out, SPEC-0038 Behavior 40), walks toward it, stops within 6.0, turns and idles for float[120, 180) ticks drawn at the stop, then returns to its default state. The notice ranges are halved while the talk countdown is positive.
6. Return after losing the hero: the shared exits and default-state rules. A wanderer resumes roaming from where it stands.

#### Per-tick order (agro) and state table

Standard agro order (SPEC-0038 Behavior 44). The outer switch starts at outer 0 and moves to outer 1 (rush).

| State | Action id and clip | Keys | Gate and exits |
|---|---|---|---|
| Set-up (outer 0) | 1 | 25 | one tick, then outer 1 inner 1 (hold) |
| Hold (inner 1) | 22 -> clip 1 | 25, loops | faces the hero at weight 0.5; d < 10 gives back off; d > 30 gives approach |
| Approach (inner 0) | 21 -> clip 3 | 25, loops | 6.12 per second (0.102 per tick); ends when d < desired (14 to 26) |
| Back off (inner 2) | 23 -> clip 2 | 25, loops | 9.03 per second backward toward the point 32 from the hero; ends when d > desired |
| Bite (slot 2) | pursuit 21 -> clip 3, then 44 -> clip 11 | 25 | state 0 walks to d <= 4.0 (chase stop within 3.18); then s = request tick; window from s + 20; first touch on a still hero s + 27 when the walk-in ends about 3.9 away (s + 25 from 3.6, s + 20 from 3.2, s + 28 at 4.5); B gate s + 96; end path s + 97 (hit or miss) |
| Single shot (slot 0) | 34 -> clip 12 | 25 | s = c + 1; shot leaves at s + 36; B gate s + 96; end path s + 97 |
| Volley (slot 1) | 39 -> clip 18 | 65 | s = c + 1; shots leave at s + 124, 148, 176, 192, 216; B gate s + 256; end path s + 257 |
| Blocked bite (state 4) | D 46 absent | | the blocked state is skipped in the same tick; state 5; end path next tick |
| Melee dodge (outer 5) | 55 -> clip 15 | 15 | gate 57, then the defence exit (outer 0, hold int[0, 30]) |
| Missile dodge (outer 6) | 53 or 54 -> clip 6 / 7 | 15 | gate 57, then the defence exit |
| Hit reaction kind 0 (outer 10) | 57 -> clip 14 | 18 | gate 69 after the last hit, then outer 0 |
| Wait in line (outer 12) | walk 2 -> clip 13 | 25, loops | stand-off point 10.0 from the hero |
| Victory pose | 73 absent | | state 0 |
| Dead | 58 -> clip 17 | 65 | plays once |

#### Attack chooser

Each tick while the hero is attackable, the recovery countdown is below 1 and the outer sub-state is 1, 3, 4 or 11: slot 2 (0 < d < 20, facing within 22.5 degrees, roll below 60, a route to the hero or d < 10), then slot 1 (0 < d < 50, within 22.5 degrees, roll below 40, cooldown below 1), then slot 0 (7 < d < 40, in front, roll below 100, no route needed for a missile slot). With the Skull Spirit facing the hero and no cooldown: d < 7: bite 59 percent, volley 16 percent (0.41 x 0.39), nothing 25 percent; 7 to 20: bite 59, volley 16, single shot 25 (0.41 x 0.61 x 0.99); 20 to 40: volley 39, shot 60, nothing 1; 40 to 50: volley 39 percent. After a choice the cooldown is drawn (volley int[30, 120], redrawn at the end path) and the desired distance is redrawn. Because the bite has no cooldown, a Skull Spirit within 20 units bites on 59 percent of its ready ticks.

A bite chosen at d above 4.0 first walks toward the hero (state 0, id 21, 6.12 per second) until d <= 4.0 or within the sum of the radii (2.0 + hero 1.175); at 15 units that takes about 108 ticks, so the bite lands about 135 ticks after the choice (108 ticks of walking, then the first touch 27 ticks after the strike request). A shot or volley chosen at any distance starts at once (no approach).

#### Movement

Rush archetype on the ground: approach 6.12 per second beyond 30, back off 9.03 per second below 10, hold in between. The hero out-walks it (13.5) and out-runs it. After a bite it is 4 units from the hero, so it backs off at 9.03 per second to the point 32 away until d exceeds its desired distance (14 to 26), taking 100 to 150 ticks; during that time slot 2 stays eligible (d < 20) and slot 1 stays eligible. A hit reaction pushes it back 14.2 per second for 69 ticks (about 16 units).

Movement table (movement pass; a ground kind, Can Fly 0, so its ids 98 to 108 are ground clips; speed byte 2):

| Mode | Source | u/t | u/s | Turn | Clip (id: clip, keys, loop) | Exits |
|---|---|---|---|---|---|---|
| Wander (R 1000), patrol, walk home, wait in line | root | 0.085 on every key | 5.10 | steering 0.3 / 0.1 | 2, 4, 11: clip 13, 25, loop | goal or point reached |
| Rush approach (inner 0) and sequencer approach | root | 0.102 average; per key 0.085 to 0.108 | 6.12 | steering | 21: clip 3, 25, loop | d below desired (10 + 20f, 14 to 26) |
| Rush hold (inner 1) | none | 0 | 0 | aim 0.5 (halves the angle each tick) | 22: clip 1, 25, loop | d below 10 or above 30 |
| Back off (inner 2) | root backward | 0.1505 average; per key -0.297 to +0.069 (one key steps forward) | 9.03 | faces the hero | 23: clip 2, 25, loop | d above desired; point 32 from the hero |
| Hit reaction kind 0 | root backward | 17.08 units over 69 ticks | 14.24 | snapped to the attacker | 57: clip 14, 18 | gate |
| Dodges 53 / 54, 55 | root | 10.26 sideways; 10.41 backward, over the clips' 60 ticks (gate 57) | 10.3; 10.4 | none | clips 6, 7, 15 | gate |
| Emerge, death | none | 0 | 0 | none | 10: clip 4; 58: clip 17 | gate; removal |

Worked example (chase from 40). Hero standing 40 away, desired distance 20: inner 1 sees d above 30 and switches to inner 0; the rush clip is requested and its first step lands one tick later; 20 units at 0.102 per tick take about 196 ticks (3.3 s), then it holds. Its missile slots (bands 7 to 40 and 0 to 50) normally start long before that.

What a port most likely gets wrong (movement): flying it (its flyer ids are ground clips: it walks); a smooth back-off (its back clip has one forward key per cycle).

#### Hit and hurt windows

- Bite: all five weapon volumes (radius 2.0, height 2.0 each) are live from s + 20 to s + 83; the volumes sweep around the skull, so a still hero 3.2 away is first touched on s + 20, one 3.6 away on s + 25 and one 3.9 away (the usual end of the walk-in) on s + 27; the strike test hits once (30 damage) and the sequencer goes to state 3 and then 5. The clip has no second window.
- Single shot: leaves at s + 36 from emit point 0, aimed at the hero's position + (0, 2.5, 0) at that tick (scatter 0; +5.0 outside the camera cone; +3.0 while waiting in line), speed 1.0, at most 31 moves (range 30), radius 1.0; hit test as for the enemy projectile pool (SPEC-0029 Behavior 9): damage 30 on an unblocked hit, a quarter (min 1) on a blocked one; destroyed by the ground and by static scenery after 2.0 units.
- Volley: five shots at 2.0 per tick (26 moves, 52 units); each shot is aimed at the hero's position at its own release tick, so a hero who keeps moving dodges most of them; a standing hero at 30 units is hit 14, 17, 16, 9 and 11 ticks after the five releases (the emit point swings from behind the skull to in front of it), five times for 40.
- Hurt shapes (Medium): body capsule of length 1.93 radius 0.97 and limb capsules of radius 0.2 to 1.1. The Skull Spirit can be hit in every state except its dodges (outer 5 and 6), while dead, and while hidden in state 8 with cue word 0.

#### Weapon reach

Conventions are those of "Weapon reach tables" in SPEC-0064 (shared field behaviour). Row 228, Medium. The tick text above is authoritative. The five volumes are live together, so the row gives their union.

| Slot (phase, clip) | Weapons | Window ticks (level contact) | Centre forward / up / side | Tor ahead, at the tick | Tor ahead, from the start | Tor feet vs origin |
|---|---|---|---|---|---|---|
| 2 bite (B, clip 11) | w1 to w5 (each 2.0 x 2.0) | s+20 to s+83 (s+20 to s+83) | -10.3 to 11.4 / -0.4 to 8.6 / -7.7 to 9.2 | -8.0 to 9.4 | -8.0 to 9.4 | -5.0 to 6.5 |

The five volumes sweep widely (about 10 either way from the body over the window), so over the whole window a level hero is touched from 8.0 behind to 9.4 ahead; at any single tick the band is much narrower. The skull does not travel.

#### Defence and counters

Melee: once per hero swing (hold below 1, not in a reaction, not mid-attack except sequencer states 0, 1 and 5, hero within 8.0 and in front): never a block (block chance 0); a dodge when int[1, 100] is below 100 (quick, normal) or 90 (heavy): 99, 99 and 89 of 100 (clip 15, 57 ticks, retreating about 10 units), then a hold of int[0, 30] ticks with no further defence. So a combat swing lands only in the 0 to 30 tick gap after a dodge or on a heavy swing in 1 of 9 cases. Missile: per live hero shot meeting the capsule inside 60 ticks, 99 of 100 give a dodge (clips 6 or 7, 57 ticks, sidestep about 9.7 units). No counters. A hit reaction triggers nothing.

#### Hero actions against this enemy

| Hero action | Result |
|---|---|
| Stands still 30 units away in its cone | 26 percent per 32-tick cycle to be noticed; 268-tick look clip first, then agro 116 ticks after the second pass |
| Within 20 | The Skull Spirit walks in and bites for 30 (59 percent per ready tick), then backs off to 14 to 26 units |
| 20 to 50 away, standing | Shoots: single 30 damage shots about every 100 ticks (they reach a standing hero only within 28.75 units: the shot leaves 3.9 behind the skull and lives 31 units) and a five-shot 200-damage volley (standing target) every 270 ticks or so |
| Quick or normal swing in reach | Dodged 99 of 100 (hold then 0 to 30 ticks); heavy swing 89 of 100 |
| Roberta's shots | Dodged 99 of 100 per shot meeting its capsule |
| Guards (L1) | The bite is blockable (hero facing the Skull within 60 degrees: the usual shield block); shots are blocked for a quarter |
| A landed hit | Damage by SPEC-0036 (300 hit points, defence value 2.0, doubled once out of agro); reaction 69 ticks, pushes it back, cancels the bite or volley |
| Alarm group | None |

#### Edge cases for implementers

- The bite chooser sits above the shots in the slot order and has no cooldown: inside 20 units the Skull Spirit almost always bites, even though its design is a ranged attacker.
- The volley is the only attack that is long (257 ticks); it is cancelled by any hit (outer 10), after which the cooldown int[30, 120] still holds.
- A lookup miss on the lose-interest and victory ids is skipped by the table checks (the setter runs at once), not played as clip 0.
- Dodge chances apply once per swing; a combo's later swings are evaluated against the new hold.
- A wandering Skull Spirit in the 1000-unit square leaves its room; a port that clamps the pick to a small radius changes where it meets the hero.
- The look clip is 268 ticks, so a first pass does not mean an attack soon: the alert clip adds 116 more.

#### What a port most likely gets wrong

1. Making it a pure ranged enemy: the bite is its most common attack inside 20 units.
2. Dropping the bite's approach walk (the bite starts at distance up to 20 and lands about 135 ticks later from 15 units).
3. Skipping the dodge: 99 percent against quick and normal melee.
4. Aiming volley shots at one point: each shot re-aims at the hero's position at its own tick.
5. Forgetting that waiting records with cue word 1 are visible.
6. Using the wrong shot ranges: the single shot reaches 31 units, the volley shot 52.

#### Test vector

Skull Spirit in agro (t = 0 the tick the setter ran), hero standing still 30 units in front, in the cone, guard off. t = 1 set-up. From t = 2: volley 39 percent, single shot 60 percent (0.61 x 0.99), nothing 1 percent per tick.
- Single shot chosen at c: s = c + 1; shot leaves c + 37 from high above and behind the skull and expires on c + 67 without reaching a hero 30 units away (it reaches a standing hero only within 28.75; Distances table); from 25 units it would hit on c + 64 for 30 damage. B gate c + 97; end path c + 98; recovery int[15, 30].
- Volley chosen at c: shots leave at c + 125, 149, 177, 193, 217 and reach the hero (2.0 per tick; 14, 17, 16, 9 and 11 ticks after each release, because the emit point swings from behind the skull to in front of it) at c + 139, 166, 193, 202, 228 for 40 each; B gate c + 257; end path c + 258; cooldown int[30, 120].
- The hero walks in to 15 units: the bite (59 percent per ready tick) walks in at 0.102 per tick: the bite is requested about 108 ticks after the choice and lands about 27 ticks later (first touch from about 3.9 units), 30 damage.
- Over 600 ticks from 30 units (20000 simulated runs, hero standing): mean 4.0 attacks (1.3 volleys, 2.7 single shots); the single shots cannot reach a hero at 30 units, so only the volleys hurt: up to 200 per volley, about 260 on average if every volley shot lands; from 15 units: mean 3.9 attacks (2.8 bites, 0.7 volleys, 0.5 shots), mean 186.

#### Open questions

- Resolved: the shot launch points are the posed emit points of the strike clips, given in the Distances and ranged reach table (single shot 0.05 to the side, 7.88 up and 3.88 behind the skull's position; the volley's five points swing from behind to in front). Medium-High.
- The bite's contact geometry (five 2.0 volumes). Resolved by the general rule (SPEC-0064, Shared field behaviour: Weapon contact and the hero's hurt shape); the distances are in the Weapon reach table; stand-in only for a port without attachment frames: hit when a level hero is from 8.0 behind to 9.4 ahead of the Skull Spirit at some tick of the window.
- Whether static scenery stops the shots. Resolved (Medium-High; proactive (open close-out)): yes; the ground and the world collision stop a shot once it is more than 2.0 units from its start, and other characters never do (SPEC-0029 Behavior 9).
- The reach of a 1000-unit roaming square on the real navmesh. Resolved (High for the rule; proactive (open close-out)): the pick follows the wander rule of SPEC-0038 (Behavior 5): a point in the square is accepted when it lies on the navmesh, otherwise it is moved halfway toward the enemy, up to 10 tries, and after 10 failures the goal is the home point (not the current position). How much of the square falls on the mesh is a property of each room's mesh.

### Spirit Sword (rows 140 and 141)

Monster type 64, variant 1 (row 140, 50 hit points, 12 records) and variant 2 (row 141, 65 hit points, 4 records), name "Spirit Sword". An armoured ghost knight on the Dark Knight's rig: a ground strafer with three melee swings, found in the Knights Crypts, Crypt of the Sentries, Ancient Castle Spires and Dungeon Ruins. One shared guide with a table of the differences; the tables of the two rows are the same except one animation id. Confidence: High for the rows, clips, gate ticks and attack arithmetic; Medium-High for the guard rules of row 141; Medium for weapon contact geometry and the alarm-wait slide.

#### Identity and tunables (both rows unless a row is named)

| Item | Value |
|---|---|
| Class, approach | Class 17; approach 4 (strafe); Can fly 0 |
| Hit points, body | Row 140: 50; row 141: 65; radius 2.0, height 2.5; defence value 0.0; attack-manager weight 5 (approach 4: it can wait in line, stand-off radius 15.0); hostile; Display in default 1; element T 3 |
| Notice | Awareness 100, timer 10 (cycle 12), view cosine 0.92388, range 10 to 50, chance 50 at 10 and 40 at 50, no line-of-sight flag; search countdown float[180, 300) (table in the shared rules) |
| Turn | Hold aim weight 1.0 |
| Strafe band | 6.0 to 16.0 (hold between); desired distance 6 + 10 f (8.0 to 14.0) redrawn after every normal attack choice; backbiter factor 1.0 (no effect) |
| Recovery after an attack | int[60, 180] ticks |
| Slot cooldowns | Slots 0, 1 and 2 draw int[0, 0]: never on cooldown |
| Missile defence | Hold 60; dodge 75 (74 of 100); block 0 (row 140) or 75 (row 141) |
| Melee defence | Dodge 75 for every swing class (74 of 100 after the block roll); block 0 (row 140) or 75 for every swing class (row 141); defence timer int[120, 300]; Blocking Angle 0.0 |
| Hit reaction | Ground kind 0: animation 57 = clip 26 (13 keys, gate 49, pushed back 5.5 per second) |
| Death | Animation 58 = clip 25 (56 keys, gate 220); corpse gone 510 ticks after progress 0.95 (about 720 ticks after death); it sinks after the first 30 of them |
| Other | No socialize (flag 0), steal, ghost, anger or map search. Weapon volumes: weapons 1 and 2 radius 6.0, height 2.0; weapons 3 to 5 radius 2.0, height 2.0 |

Differences between the rows: hit points 50 and 65; row 141 guards (melee and missile block 75, so it blocks 75 of 100 swings and then dodges 74 of the remaining 25); row 141 has a fourth slot that can never be chosen (below); the table of variant 2 has one extra id (64, a one-shot of 8 keys with gate 29) that no state requests. The placement mix is row 140 (Knights Crypts 6 records, Crypt of the Sentries 4, Ancient Castle Spires 2) and row 141 (Knights Crypts 3, Dungeon Ruins 1).

Attack slots (slots tested 3, 2, 1, 0; the bands overlap):

| Slot | Band (strict) | Chance (rolled below) | Arc | Range | Damage and effect | Orient | Counter | Clips (phase ids 33 + 5 k + p) |
|---|---|---|---|---|---|---|---|---|
| 3 | Row 140: unused. Row 141: 0 to 8 | Row 141: 0 (never) | 0.9239 | 7.0 | row 141: 10, knockback 3.0 | 0 | none | row 140 none; row 141 B 49 = clip 20 (8 keys, loops, 33.8 per second forward), never played |
| 2 | 0 to 10 | 60 (59 of 100) | 0.707 | 15.0 | 30, blockable | 0 | none | B 44 = clip 10 (10 keys, gate 37, weapon 1 live from s + 16 to the end of the clip); D 46 = clip 11 (10 keys, gate 37); E 47 = clip 12 (10 keys, gate 37); A, C absent |
| 1 | 7 to 15 | 15 (14 of 100) | 0.707 | 12.0 | 30, unblockable (flag 0x1), curse (status 1) at 75 (75 of 101 draws), 30 to 60 s | 0 | trigger 5 (sequencer blocked with L1 held), chance 50 | B 39 = clip 13 (20 keys, gate 77, weapon 1 live from s + 48); D 41 = clip 14 (10 keys, gate 37); E 42 = clip 15 (5 keys, gate 17); A, C absent |
| 0 | 0 to 16 | 50 (49 of 100) | 0.707 | 13.0 | 15, blockable | 1 | none | A 33 = clip 17 (10 keys, gate 37); B 34 = clip 18 (10 keys, rate 1.5 then 1.0, gate 25 at rate 1.5; weapon 1 live ticks 6 to 10 of B at rate 1.5); D 36 = clip 9 (12 keys, gate 45, backward 12.9 per second); C, E absent |

Speed words: slot 0 wind-up 1.0, strike 1.5, recovery 1.0; all other slots 1.0. Knockback on the hero is 0 on every live slot. Phase ids not listed have no clip and are skipped in the same tick; slots 1 and 2 have no wind-up, so their strike clip is requested on the first sequencer tick.

#### Animation ids and clips (ground ids)

| Id | Use | Clip |
|---|---|---|
| 1 | agro idle pick | clip 23 (20 keys, loops) |
| 0, 13 to 15 | idle pick outside agro | clip 28 (40 keys, loops); fidgets 115, 116 = clip 31 (63 keys, gate 248), clip 32 (60 keys, gate 236) |
| 2, 4, 5 | wander and patrol walk | clip 5 (30 keys, loops, 3.0 per second) |
| 3 | wander pause fidget | clip 31 (63 keys, gate 248) |
| 6 | patrol pause fidget | clip 21 (40 keys, one-shot, gate 156) |
| 7, 8 | sentry stand, fidget | clip 30 (40 keys, loops), clip 32 (60 keys, gate 236) |
| 28 | strafe forward (approach, pursuit before a swing) | clip 0 (15 keys, loops, 10.0 per second) |
| 29, 30 | circle (side gaits) | clip 1 (12 keys, +4.2 per second sideways), clip 2 (12 keys, -4.5 per second) |
| 31 | strafe hold | clip 23 (20 keys, loops) |
| 32 | strafe back off | clip 7 (12 keys, loops, -6.0 per second) |
| 53, 54, 55 | missile dodge, melee dodge | clip 3 and clip 4 (13 keys, gate 49, sideways 16.3 per second), clip 27 (10 keys, gate 37, backward 7.7) |
| 56, 126 | block pose, block flash (row 141) | clip 33 (1 key), clip 39 (8 keys, gate 29) |
| 57 | hit reaction kind 0 | clip 26 (13 keys, gate 49) |
| 58 | death | clip 25 (56 keys, gate 220) |
| 9 | hide pick (state 8) | clip 19 (1 key) |
| 16, 17 | look, lose interest | clip 21 (40 keys, gate 156), clip 29 (23 keys, gate 89) |
| 73 | victory pose | clip 24 (38 keys, gate 148) |
| 69 | confused | clip 8 (20 keys, loops, 4.4 per second) |
| 10, 18 | emerge, alert | absent |

#### Start, wake, return

Start states and the wake sequence are the shared rules with the Spirit Sword column of the notice table.
1. Default 7 (5 records, Knights Crypts): +0xC4 is 0 on all five, so the sword plays the look clip (id 8, clip 32, 236 ticks) once and then the stand clip (id 7, clip 30) for int[60, 600] ticks, and repeats, never turning. Default 8 (11 records): 6 with cue word 1 (visible and hittable while waiting), 5 with cue word 0 (invisible).
2. A pass of the notice check enters state 2 (search float[180, 300)): look clip 16 (156 ticks), second check on the same tick and then once per 12-tick cycle. Alert id 18 is absent, so a second pass goes to agro on that tick. If the countdown ends with no second pass, the lose-interest clip 17 (89 ticks) plays and the setter runs with 0 (state 0, then the default state).
3. Waiting in state 8: wake by distance (x F1); emerge id 10 is absent, so the sword enters agro on the next tick with no appear animation. Emerge distance 0 (three records) never wakes by distance.
4. Alarm group: none of the 16 records has a group.
5. A hit at any time wakes it into agro (doubled damage, kind-0 reaction). A sword in state 8 with cue word 0 cannot be hit.
6. Return after losing the hero: the shared exits; a sentry walks home and stands, a hidden sword walks home and hides again.

#### Per-tick order (agro) and state table

Standard agro order (SPEC-0038 Behavior 44). The outer switch starts at outer 0 and moves to outer 4 (strafe, inner 10).

| State | Action id and clip | Keys | Gate and exits |
|---|---|---|---|
| Set-up (outer 0) | 1 | 20 | one tick, then outer 4 inner 10 |
| Strafe hold (inner 10) | 31 -> clip 23 | 20, loops | hold counts down from int[60, 180]; at 0 a roll int[1, 3]: 1 stay, 2 circle left (inner 8), 3 circle right (inner 9) (a circle is refused to a stay while the strafe lock is above 0) |
| Circle (inner 8 / 9) | 30 / 29 -> clip 2 / 1 | 12, loops | sideways about 4.5 / 4.2 per second; hold int[60, 180]; ends at hold 0, blocked (lock 120), d > 16 (approach) or d < 6 with the lock below 1 (back off) |
| Approach (inner 7) | 28 -> clip 0 | 15, loops | 10.0 per second (0.167 per tick); ends when d < desired (8 to 14) |
| Back off (inner 11) | 32 -> clip 7 | 12, loops | 6.0 per second backward toward the point 16 from the hero; ends when d > desired |
| Slot 2 swing | 44 -> clip 10, then 47 -> clip 12 | 10, 10 | s = c + 1; window from s + 16; first touch on a still hero s + 18 (3.2 to 6.0 away) or s + 20 (9 to 10 away); B gate s + 37; E requested s + 38; E gate s + 75 = end path (hit or miss) |
| Slot 1 swing | 39 -> clip 13, then 42 -> clip 15 | 20, 5 | s = c + 1; window from s + 48; first touch s + 48 (up to 2.5 away), s + 49 (3.0 to 9.0 away) or s + 50 (9.5 to 12.0 away); B gate s + 77; E requested s + 78; E gate s + 95 = end path |
| Slot 0 swing | 33 -> clip 17 then 34 -> clip 18 | 10, 10 | A requested at s = c + 1 (after any approach to 13.0), gate s + 37; B requested then (rate 1.5); window s + 43 to s + 47; first touch on a still hero s + 44 (5.5 to 10.5 units away), s + 45 (3.5 to 5.0) and s + 47 (at 3.0 and at 11.0), none from 11.5 or more (the swing then whiffs although the range is 13); with a hit on s + 44 or s + 45 the end path is s + 71, after a hit on s + 47 s + 70, with none s + 63 (E absent) |
| Blocked swing (state 4) | D 36 -> clip 9 (slot 0), D 46 -> clip 11 (slot 2); slot 1 is never blocked | 12, 10 | blocked on the hit tick; state 4 on the next tick; gates 45 and 37; then state 5 and the end path (slot 2 then plays E first) |
| Melee dodge (outer 5) | 55 -> clip 27 | 10 | gate 37, then the defence exit (hold int[120, 300]) |
| Missile dodge (outer 6) | 53 or 54 -> clip 3 / 4 | 13 | gate 49, then the defence exit |
| Melee block (outer 7, row 141) | 56 -> clip 33, base state 20; flash 126 -> clip 39 on a blocked hit | 1, 8 | while the hero swings; then the defence exit |
| Missile block (outer 8, row 141) | 56 -> clip 33 | 1 | while a hero shot is live |
| Hit reaction kind 0 (outer 10) | 57 -> clip 26 | 13 | gate 49 after the last hit, then outer 0 |
| Wait in line (outer 12) | walk 2 -> clip 5 | 30, loops | stand-off point 15.0 from the hero (SPEC-0038 Behavior 45) |
| Victory pose (state 10) | 73 -> clip 24 | 38 | gate 148, then the default state |
| Dead | 58 -> clip 25 | 56 | plays once |

#### Attack chooser

Each tick while the hero is attackable, recovery is below 1 and the outer sub-state is 1, 3, 4 or 11: slot 3 (never passes), slot 2 (0 < d < 10, within 45 degrees, roll below 60), slot 1 (7 < d < 15, within 45 degrees, roll below 15), slot 0 (0 < d < 16, within 45 degrees, roll below 50). A melee slot with no charge passes when a route exists or d < 10. Per-tick chance of starting some attack with the sword facing the hero and no cooldowns: d < 7: 59 percent slot 2, 20 percent slot 0, nothing 21 percent; 7 to 10: 82 percent (slot 2 59, slot 1 6, slot 0 17); 10 to 15: 56 percent (slot 1 14, slot 0 42); 15 to 16: slot 0 49 percent. Because there are no cooldowns, the next attack starts as soon as recovery (int[60, 180]) ends and a ready tick passes the rolls. A melee slot with d above its range first walks toward the hero at 10.0 per second (gait 0, id 28; the step is dropped within the sum of the radii 2.0 + 1.175): slot 1 to 12.0, slot 0 to 13.0; slot 2 (range 15) never walks. After a choice the desired distance is redrawn.

#### Movement

Strafe archetype with the settled rules (hold 60 to 180 ticks then a draw of stay, left, right; circle with the strafe lock; approach beyond 16; back off below 6). The sword is slow: approach 10.0 per second, circle 4.2 to 4.5, back off 6.0, wander and patrol 3.0; the hero (13.5 walking) out-walks every gait. An approach is steered; a swing's own root travel carries it forward during the strike clip (slot 2 11.3 per second: up to 7.6 units over its 37 ticks). Corrected (movement pass): the strike runs in sequencer state 2, inside the chase-stop states 0 to 3, so every step that would start within 3.175 (radius 2.0 + 1.175, 3D distance) of the hero is discarded; a sword that chose slot 2 at 9 units ends about 3.2 from a standing hero, not inside him.

Movement table (movement pass; both rows; speed byte 2):

| Mode | Source | u/t | u/s | Turn | Clip (id: clip, keys, loop) | Exits |
|---|---|---|---|---|---|---|
| Walk home (state 7 sentries, state 8), wait in line | root | 0.0501 on every key | 3.00 | steering 0.3 / 0.1 | 2, 4: clip 5, 30, loop | home reached |
| Strafe approach (inner 7) and sequencer approach | root | 0.1668 on every key | 10.01 | steering | 28: clip 0, 15, loop | d below desired (6 + 10f, 8 to 14) or within the slot range |
| Circle (inners 8, 9) | root sideways | 29: 0.0703 (+x), 30: 0.0753 (-x) | 4.22, 4.52 | faces the hero (side gait rule) | 29: clip 1; 30: clip 2; 12, loop | hold count, blocked flag, distance tests |
| Back off (inner 11) | root backward | 0.0997 | 5.98 | faces the hero | 32: clip 7, 12, loop | d above desired; point 16 from the hero |
| Strafe hold | none | 0 | 0 | aim 1.0 | 31: clip 23 | hold count |
| Slot 2 strike (state 2) | root x 1.0 | 7.56 units over the clip | 11.33 | none; chase stop at 3.175 | 44: clip 10, 10 | gate |
| Alarm alert with no clip | root | 0.1668 | 10.01 | none | 18 missing: clip 0 (the strafe-forward clip), gate 59 | gate (about 9.8 units forward) |
| Dodges 53 / 54, 55 | root | 14.16 sideways; 5.15 backward | 16.3; 7.7 | none | clips 3, 4 (13 keys); 27 | gate |
| Hit reaction kind 0 | root backward | 4.72 units over 49 ticks | 5.45 | snapped to the attacker | 57: clip 26, 13 | gate |
| Death | root backward | 1.33 units over the clip | - | none | 58: clip 25 | removal |

Worked example (approach from 20). Hero standing 20 away, desired distance 11: the hold sees d above 16 and switches to inner 7; 9 units at 0.1668 per tick take 54 ticks, then it holds and the slot-0 band (below 16) is already live. From 9 units a slot-2 choice carries it to 3.2 from the hero in about 35 ticks of the strike clip.

What a port most likely gets wrong (movement): letting a strike's root travel carry it through the hero (the chase stop applies to the strike); giving its alarm alert no travel (the missing clip 18 plays the strafe-forward clip and walks it about 10 units).

#### Hit and hurt windows

- Slot 2: weapon 1 (radius 6.0, height 2.0) live from s + 16 until the clip ends; one hit of 30 per window (the window is one continuous stretch, so one hit). Slot 1: weapon 1 from s + 48, one hit of 30 plus the curse roll. Slot 0: weapon 1 from s + 43 to s + 47 (B ticks 6 to 10 at rate 1.5), one hit of 15 on a hero no more than about 11 units ahead when B starts. The first touch on a still hero is one to four ticks after each window opens (state table); slot 0's range (13) and band (up to 16) lie beyond its reach, so a slot-0 swing started from more than about 11 units whiffs (slot 2's band ends at 10, inside its reach). The recovery clips E carry weapon bits but the strike test does not run in state 5.
- Hit damage before armour and shield: 30, 30, 15 (and 10 for the unreachable slot 3 of row 141). A guarding hero (L1, facing the sword) blocks slots 0 and 2 (state 4, no damage) and not slot 1; blocking slot 0 or 2 at 7 < d < 15 opens slot 1 as a counter (chance 50 per tick of the blocked state, 45 or 37 ticks long, so almost certain while the hero holds guard).
- Hurt shapes (Medium): body capsule of length 2.55 radius 1.45 and limb capsules of radius 0.6 to 1.0 (variant 1 carries a 6.1-long blade capsule of radius 0.62). The sword can be hit in every state except its dodges (outer 5 and 6), while dead, while hidden with cue word 0, and while it blocks (row 141, below).

#### Weapon reach

Conventions are those of "Weapon reach tables" in SPEC-0064 (shared field behaviour). Rows 140 and 141, Medium. The tick text above is authoritative. The figures are the Dark Knight's (SPEC-0064): the Spirit Sword uses the same rig and clips, so the table is the same. Ticks count from the tick the slot's clip is requested.

| Slot (phase, clip) | Weapon | Window ticks (level contact) | Centre forward / up / side | Tor ahead, at the tick | Tor ahead, from the start | Tor feet vs origin |
|---|---|---|---|---|---|---|
| 0 (B, clip 18) | w1 (6.0 x 2.0) | s+6 to s+10 (s+7 to s+10) | 3.3 to 4.9 / 1.2 to 4.0 / -3.1 to 2.9 | 3.0 to 9.4 | 4.8 to 11.1 | -7.0 to 9.0 |
| 1 (B, clip 13) | w1 (6.0 x 2.0) | s+48 to s+77 (s+48 to s+77) | 1.4 to 4.4 / 0.1 to 5.2 / -2.4 to -0.6 | -4.0 to 10.8 | -0.8 to 15.1 | -8.5 to 10.0 |
| 2 (B, clip 10) | w1 (6.0 x 2.0) | s+16 to s+37 (s+16 to s+37) | 0.3 to 4.6 / 1.1 to 2.5 / -2.6 to -0.6 | -0.4 to 6.4 | 2.6 to 13.7 | -8.0 to 8.0 |

Slot 0 never touches a hero closer than 3.0 ahead; slot 1 reaches 10.8 ahead at the tick and 15.1 from where the sword started. Row 141's unreachable slot 3 is not tabulated.

#### Defence and counters

Row 140: once per hero swing (hold below 1, not in a reaction, not mid-attack except sequencer states 0, 1 and 5, hero within 8.0 and in front): a dodge on 74 of 100 swings of any class (clip 27, 37 ticks, then a hold of 120 to 300 ticks with no further defence). Missile: 74 of 100 per live shot meeting the capsule inside 60 ticks (clips 3 and 4, 49 ticks).
Row 141: first r1 at most 75 gives a block (75 of 100 swings): the sword faces the hero and guards (base state 20) while the hero swings; a hero non-power swing at a guarding sword is blocked, with no damage, when dot(sword facing, hero facing) is below 0.0 (hero in front of the sword), and passes from behind; then r2 below 75 gives a dodge (74 of 100 of the remaining 25), so 93.5 percent of swings are answered. A missile: dodge 74 of 100, then block 74 of 100 overriding it. Hold int[120, 300] after either. A Tor power swing (sub-state 10 with a Mote) ignores the guard.
Counters: slot 1 only, trigger 5 (above). The hit reaction triggers no counter.

#### Hero actions against this enemy

| Hero action | Result |
|---|---|
| Stands still 12 units in front of a sentry | 49 percent per 12-tick cycle; look clip 156 ticks, then agro on the next pass |
| Walks within 15 | Noticed on the next awareness tick (every cycle is 12 ticks) |
| Closes inside 10 | Slot 2 (30, 59 percent per ready tick, hit 19 to 21 ticks after the choice) and slot 0 (15) |
| Quick, normal or heavy swing inside 8 (row 140) | Dodged on 74 of 100 swings, then no dodge for 120 to 300 ticks |
| Same (row 141) | Blocked 75 percent, dodged 18.5 percent, hit 6.5 percent; a swing from behind is not blocked |
| Roberta's shots | Row 140: dodged 74 of 100; row 141: dodged or blocked 93 of 100 |
| Guards (L1) | Slots 0 and 2 are blocked and may be answered by slot 1; slot 1 hits through the guard |
| A landed hit | Damage (50 or 65 hit points, defence value 0, doubled once out of agro), reaction 49 ticks that cancels the swing, no counter |

#### Edge cases for implementers

- Spirit Swords are never hidden by Display and never use state 21. The alert id 18 is absent, so a noticed sword is in agro on the tick of its second pass. A group alarm would request id 18 directly and play clip 0, a 15-key strafe loop with 10 units per second of forward travel, for 59 ticks before agro (no sword has a group on disc).
- Rows 140 and 141 have no cooldowns, so recovery int[60, 180] and the choice rolls set the pace; slot 2 at 59 percent dominates inside 10 units.
- Row 141 slot 3 is a used slot with chance 0: the threshold int(0) = 0 means the roll never passes. A port that reads its damage 10 and knockback 3 as a live attack is wrong.
- Row 141's guard needs the hero in front (dot below 0.0), so circling behind a blocking row-141 sword avoids the guard.
- A swing's root travel is not a lunge the chase-stop can prevent: after the choice the sword may end up on top of the hero.
- The recovery clips E of slots 1 and 2 (17 and 37 ticks) lengthen those swings relative to the Dark Knight (Minion Corps B), which has none.

#### What a port most likely gets wrong

1. Treating row 141 as row 140: it guards and blocks 75 percent of swings.
2. Giving the sword an alert clip or a hidden state.
3. Forgetting the 74 percent dodge and its 120 to 300 tick quiet period.
4. Using the slot order as a priority from slot 0: it starts at the highest slot (2 first), so the common close swing is the 30-damage slot 2.
5. Missing the unblockable curse of slot 1 and its counter use after a blocked swing.
6. Giving it a fast gait: approach 10.0, circle 4.4, back off 6.0 per second.
7. Dropping the E clips (they add 38 and 18 ticks to slots 2 and 1).

#### Test vector

Spirit Sword (row 140) in agro (t = 0 the tick the setter ran), hero standing still 12 units in front, guard off. t = 1 set-up. From t = 2 the per-tick chance of starting an attack is 56 percent (slot 1 14 percent, slot 0 42 percent), so the first choice is at t = 2 with 56 percent, t = 3 with 25 percent, later otherwise (mean 2.8).
- Slot 0 at c: d = 12 is inside its range 13, so s = c + 1, A to s + 37 (the wind-up clip has no root travel), B requested at c + 38; the blade reaches at most about 11 units from where B starts (Weapon reach table: 9.6 at the tick plus B's own 2.2 of travel), so a still hero 12 away is never touched: whiff, end path c + 64. From 5.5 to 10.5 units the first touch is B + 7 (c + 45, 15 damage, no status) and the end path c + 72; at exactly 11 units the first touch is B + 10 (c + 48) and the end path c + 71.
- Slot 1 at c: hit at c + 51 (first touch s + 50 at 12 units; 30 damage, and curse on 75 of 101 draws for 30 to 60 s), end path c + 96 (B gate c + 78, E requested, E gate c + 96).
- Recovery int[60, 180] follows the end path, then the next choice at the per-tick chances above (slot 2 added once the sword has closed inside 10), so one attack every 130 to 270 ticks.
- Over 600 ticks (20000 simulated runs, hero standing at 12 units): mean 3.6 attacks (0.9 slot 1, 2.7 slot 0); only the slot-1 swings can touch a hero 12 units away, so the damage is about 27 on average (30 per slot-1 hit, 0 to 90); with the hero at 6 units: mean 4.1 attacks (3.1 slot 2, 1.0 slot 0), mean 104. Over the same window a Tor swing is dodged on 74 of 100 swings (row 141: 93.5 percent answered).

#### Open questions

- Weapon contact geometry (volume anchor and the hero's capsule pose). Resolved by the general rule (SPEC-0064, Shared field behaviour: Weapon contact and the hero's hurt shape); the distances are in the Weapon reach table; stand-in only for a port without attachment frames: a window hits a level hero inside the "at the tick" range of the slot's row (3.0 to 9.6, 4.0 behind to 10.8 and 0.4 behind to 6.4 ahead for slots 0, 1 and 2).
- Whether the forward slide of the alert wait (clip 0 as id 18) exists in play. Resolved (High; proactive (open close-out)): yes; a missing id plays clip 0 of the main set (the lookup-miss rule, SPEC-0040 Behavior 3D) and the root step of whatever clip plays is applied, so the clip plays for its 59 ticks including its root travel.
- Which wait clip a sentry with a nonzero +0xC4 plays. Resolved (High; proactive (open close-out)): it does not occur on this kind: all five Spirit Sword sentry records have +0xC4 = 0. (For a record that had it, the sentry turns toward its stored look point and holds animation 7, SPEC-0038 Behavior 5, state 7.)

### Death Knight (row 143)

Monster type 65 variant 0, name "Death Knight"; one table of animation ids and clips (the 43-clip ghost rig shared with the Restless Spirit) for all 6 records in play. A tall armoured ghost that flies, guards, dodges and, rarely, charges 30 to 50 units in a straight line. It waits invisible in the Knights Crypts (a group of three), the Crypt of the Fallen and, as sentries, the Ancient Castle Spires. Confidence: High for the row, the table, the ghost cycle, the defence arithmetic and the slot numbers; Medium-High for the charge flow and the lookup-miss dodge; Medium for weapon contact geometry and for the charge against scenery.

#### Identity and tunables

| Item | Value |
|---|---|
| Class, approach | Class 17; approach 1 (rush); Can fly 2; flying idle and agro velocity 2.0 and 7.0 (4.0 and 14.0 units per second); maximum fly height 100.0 |
| Hit points, body | 200; radius 1.5, height 5.0; defence value 0.0; attack-manager weight 12 (it can wait in line, stand-off radius 20.0); hostile; Display in default 1; element T 6 |
| Notice | Awareness 90, timer 60 (cycle 62), view cosine 0.707, range 1 to 10, chance 1 at 1.0 and 90 at 10 (it rises with distance), sees through walls; search countdown float[60, 240) (table in the shared rules) |
| Turn | Flyer aim weight 1.0 (ground weight 1.0) |
| Rush band | Rush minimum 5.0, maximum 15.0: desired distance 5 + 10 f (7 to 13), redrawn after every normal attack choice. Hold between 5 and 15; below 5 back off toward the point 17 from the hero; above 15 approach |
| Recovery after an attack | int[15, 60] ticks |
| Slot cooldowns | Slot 0 and slot 1 int[0, 30]; slot 2 int[30, 60] |
| Missile defence | Hold 15, dodge 100 (99 of 100 per shot), block 0 |
| Melee defence | Block 10, 50, 50 and dodge 90, 50, 50 for quick, normal and heavy swings; defence timer int[15, 30]; Blocking Angle 0.707 |
| Hit reaction | Flyer kind 6: animation 152 = clip 35 (12 keys, gate 45, pushed back 16.6 per second), then outer 0 |
| Death | Animation 153 = clip 30 (44 keys, gate 172); corpse gone 10 ticks after progress 0.95 (about 175 ticks after death); never sinks |
| Ghost (row words) | Solid phase int[300, 420], ghost phase int[60, 180], opacity 0.1 to 0.5 (alpha 12 to 64), reveal int[60, 180] |
| Anger level | 1: each hit raises the anger counter; no slot has the anger flag (0x40), so the counter feeds nothing |
| Other | No steal, socialize or map search. Weapon volumes: weapon 1 radius 7.0, height 1.0; weapons 2 and 3 radius 5.0, height 3.0; weapons 4 and 5 radius 2.0, height 2.0 |

Attack slots (slots tested 2, 1, 0; slot 3 is unused):

| Slot | Role | Band (strict), chance, arc | Range, cooldown, orient | Damage and effect | Counter | Clips (phase ids 33 + 5 k + p) |
|---|---|---|---|---|---|---|
| 2 | Charge (kind 0) | 10 to 20, 15 (14 of 100), arc 0.707; needs a route | melee range 8.0; charge distance float[30, 50); cooldown int[30, 60]; orient 0 | 10, blockable, knockback 5.0, slow (status 3) at 50 (50 of 101) for 3 to 5 s | trigger 2 (after a dodge), chance 75 | A absent; B 44 = clip 5 (28 keys, loops, 5.45 per second root travel at rate 1.0; weapon 1 live in two 8-tick windows every 56 ticks at rate 2.0); C absent; D 46 = clip 4 (15 keys, gate 57); E absent |
| 1 | Heavy cut | 4 to 10, 50 (49 of 100), arc 0.707 | melee range 4.0; cooldown int[0, 30]; orient 1 | 60, unblockable (flag 0x1) | trigger 5 (sequencer blocked with L1 held), chance 100 | A absent; B 39 = clip 23 (21 keys, rate 2.0, gate 41, weapon 1 live s + 16 to s + 21); D 41 = clip 4; C, E absent |
| 0 | Quick cut | 0 to 15, 75 (74 of 100), arc 0.707 | melee range 4.0; cooldown int[0, 30]; orient 1 | 30, blockable, slow (status 3) at 10 (10 of 101) for 1 to 3 s | trigger 6 (hit reaction), chance 50 | A absent; B 34 = clip 21 (29 keys, rate 3.0, weapon 1 live s + 14 to s + 19); D 36 = clip 22 (15 keys, gate 57); C, E absent |

Speed words: slot 0 wind-up 1.0, strike 3.0, recovery 1.0; slot 1 wind-up 2.0, strike 2.0, recovery 1.2; slot 2 wind-up 1.0, strike 2.0, recovery 1.0. Phase ids not listed have no clip and are skipped in the same tick; no slot has a wind-up or recovery clip, so every strike clip is requested on the first sequencer tick.

#### Animation ids and clips (flyer ids)

| Id | Use | Clip |
|---|---|---|
| 141 | agro idle pick | clip 1 (20 keys, loops) |
| 0, 97 | idle pick outside agro | clip 15 (40 keys, loops); fidget 142 = clip 12 (45 keys, gate 176) |
| 101 | rush approach, pursuit before a cut | clip 3 (15 keys, loops, no root travel; the row speed 14.0 per second moves it) |
| 102 | back off | clip 2 (15 keys, loops, -12.25 per second; with the row speed -26.25 per second) |
| 98, 99, 100 | wander, patrol, named-target walk | clip 0 (20 keys, loops, no root travel; row speed 4.0 per second) |
| 144, 145 | sentry stand, sentry fidget | clip 13 (40 keys, loops), clip 12 (45 keys, gate 176) |
| 3, 6 | wander pause fidget, patrol pause fidget | clip 12 (gate 176), clip 11 (40 keys, gate 156) |
| 138, 139, 140, 155 | missile dodge, melee dodge, rush hold | absent: clip 0 (20 keys, loops, gate 79) |
| 150, 151 | block pose, block flash | clip 8 (1 key), clip 9 (8 keys, gate 29) |
| 152 | hit reaction kind 6 | clip 35 (12 keys, gate 45) |
| 153 | death | clip 30 (44 keys, gate 172) |
| 147, 148, 149 | look, lose interest, alert | clip 12 (gate 176), clip 16 (33 keys, gate 128), clip 7 (30 keys, gate 116) |
| 9 | hide pick (state 8) | clip 24 (1 key) |
| 165 | emerge | clip 27 (25 keys, gate 96) |
| 163, 73 | victory pose | clip 11 (40 keys, gate 156) |
| 69 | confused | clip 41 (40 keys, loops, 4.5 per second) |

#### Start, wake, return

Start states and the wake sequence are the shared rules with the Death Knight column of the notice table.
1. Default 7 (2 records, Ancient Castle Spires): +0xC4 is 1, so the knight turns to the placed facing every tick and plays the stand clip (id 144, clip 13). Default 8 (4 records, all cue word 0): invisible and untouchable until the hero comes closer than the emerge distance x F1. Distances: 1.0 (two Knights Crypts records of group 1: 0.7 for a standing hero, 1.75 for a running one), 10.0 (group 1: 7 standing, 17.5 running) and 4.0 (Crypt of the Fallen: 2.8 and 7.0).
2. Wake from state 8: id 165 (clip 27, 25 keys, 96 ticks) plays and the knight enters agro when it passes its gate; the knight is visible and hittable during the emerge clip (a hit during it is a normal hit: doubled damage, agro, reaction). The three group-1 knights emerge together: any member entering agro makes the others emerge within int[15, 45] ticks, at any distance.
3. Wake from state 7: a pass of the notice check enters state 2 (search float[60, 240)): look clip 147 (176 ticks), second check on the same tick and then once per 62-tick cycle; a second pass requests the alert clip 149 (116 ticks) and the knight enters agro at its gate; a countdown that ends with no second pass plays the lose-interest clip 148 (128 ticks) and runs the setter with 0 (state 0, then the default state, which turns the sentry back to its post).
4. Chance of a pass per cycle for a still hero in the cone: 0.18 at 3 units, 0.36 at 5, 0.54 at 7, none beyond 7; 0.09 at 2; a walking or running hero from 15 or 17.5 units (every pass is certain beyond 10 once roll A passes, 0.9).
5. A hit at any time wakes it into agro with doubled damage, unless the ghost rule absorbs it.
6. Return after losing the hero: the shared exits; a sentry walks back to its post and faces it, a hidden knight walks home and hides again (the whole wake sequence repeats).

#### Per-tick order (agro) and state table

Standard agro order (SPEC-0038 Behavior 44). The ghost update runs before the character's AI step each tick (SPEC-0029 Behavior 33).

| State | Action id and clip | Keys | Gate and exits |
|---|---|---|---|
| Set-up (outer 0) | 141 | 20 | one tick, then outer 1 inner 1 (hold) |
| Approach (inner 0) | 101 -> clip 3 | 15, loops | 14.0 per second (0.233 per tick); ends when d < desired (7 to 13) |
| Hold (inner 1) | 155 absent: clip 0 | 20, loops | faces the hero at weight 1.0; d < 5.0 gives back off; d > 15 gives approach |
| Back off (inner 2) | 102 -> clip 2 | 15, loops | 26.25 per second backward toward the point 17 from the hero; ends when d > desired |
| Quick cut (slot 0) | 34 -> clip 21 | 29 | pursuit to d <= 4.0 first (state 0); then s = request tick; hit at s + 14; with a hit: B gate s + 84, end path s + 85; with no contact: B gate s + 39, end path s + 40 |
| Heavy cut (slot 1) | 39 -> clip 23 | 21 | s = c + 1 (pursuit if d > 4.0); window from s + 16; first touch on a still hero s + 20 (2.5 to 4.5 away; s + 19 from 5.0 or more; s + 18 at 1.0 to 1.5; no touch at 2.0); with a hit on s + 20: B gate s + 60, end path s + 61 (hit on s + 19: gate s + 61, end path s + 62; hit on s + 18: gate s + 62, end path s + 63); no contact: gate s + 41, end path s + 42 |
| Charge (slot 2, states 6, 7, 8, 12) | 44 -> clip 5 | 28, loops | B requested c + 1 (state 6 aims at the hero once); state 7 from c + 2 at 24.9 per second (0.415 per tick); strike windows at B ticks 18 to 25, 38 to 45, then every 56 ticks; a hit goes to state 8 (runs on at 19.5 per second); the run ends when the travelled distance reaches the drawn 30 to 50 (state 12), end path next tick |
| Blocked cut (state 4) | D 36 -> clip 22 (slot 0); slot 1 is never blocked | 15 | blocked on the hit tick; state 4 on the next tick; gate 57; then state 5 and the end path |
| Blocked charge (state 9) | D 46 -> clip 4 | 15 | the blocked-charge kind is 0: plays D (backward 10 per second against the 14 per second row speed) until the distance is covered, then state 12 |
| Melee dodge (outer 5) | 140 absent: clip 0 | 20, loops | gate 79, then the defence exit (outer 0, hold int[15, 30]); not hittable |
| Missile dodge (outer 6) | 138 or 139 absent: clip 0 | 20, loops | gate 79, then the defence exit; not hittable |
| Melee block (outer 7) | 150 -> clip 8, base state 20; flash 151 -> clip 9 on a blocked hit | 1, 8 | while the hero swings; then the defence exit |
| Missile block (outer 8) | 150 -> clip 8 | 1 | while a hero shot is live |
| Hit reaction kind 6 (outer 10) | 152 -> clip 35 | 12 | gate 45 after the last hit, then outer 0 |
| Wait in line (outer 12) | 98 -> clip 0 | 20, loops | stand-off point 20.0 from the hero |
| Victory pose (state 10) | 163 -> clip 11 | 40 | gate 156, then the default state |
| Dead | 153 -> clip 30 | 44 | plays once |

#### Attack chooser

Each tick while the hero is attackable, the recovery countdown is below 1 and the outer sub-state is 1, 3, 4 or 11: slot 2 (cooldown below 1, 10 < d < 20, within 45 degrees, a route to the hero, roll below 15), then slot 1 (4 < d < 10, within 45 degrees, roll below 50), then slot 0 (0 < d < 15, within 45 degrees, roll below 75). With the knight facing the hero and no cooldowns: d < 4: quick cut 74 percent; 4 to 10: heavy cut 49 percent, quick cut 38 (0.51 x 0.74), nothing 13; 10 to 15: charge 14, quick cut 64 (0.86 x 0.74), nothing 22; 15 to 20: charge 14 percent. A cut chosen at d above 4.0 first flies toward the hero at 14.0 per second (the step is dropped within the sum of the radii 1.5 + 1.175) and starts when d <= 4.0, so a cut chosen at 12 units lands about 50 ticks after the choice. After a choice the cooldown is drawn (slot 0 and 1: int[0, 30]; slot 2: int[30, 60]; redrawn at the end path) and the desired distance is redrawn.

#### Movement

Rush archetype with the flyer gaits: approach 14.0 per second, back off 26.25 per second, hold at 5 to 15 units. Corrected (movement pass): the knight is a Can Fly 2 row, so its goal height is the hero's own height (capped at 100 above the ground), gravity acts, and it moves at about the hero's foot level; the 0.2 x distance height, the 2.0 floor and the hero-height-plus-2.0 charge height of the Can Fly 1 rows do not apply to it. A hold distance of 7 to 13 sits inside the cut bands, so a knight waiting for its rolls is usually inside slot 0's band (and often slot 1's). A hit reaction pushes it back 16.6 per second for 45 ticks (about 12 units).

The charge. Slot 2 is a kind-0 charge: at the choice tick the knight turns toward the hero once (state 6 aims), then runs in that fixed heading without steering. Its flying row speed adds 14.0 per second in the charge states, so the run covers 0.415 per tick (24.9 per second) while state 7 lasts and 0.324 per tick (19.5 per second, clip rate back to 1.0) in state 8. The run ends when the distance travelled from the start point reaches the drawn charge distance (30 to 50 units), or 100 ticks after it stopped moving; a hit does not end it. The knight finishes 20 to 40 units beyond a hero who stood still at 12 units and then turns back with the ordinary rush rules. On the run ticks (state 7) a kind-0 charger is not solid, so it passes through a hero it misses; in state 8 (after a hit) it is solid again (shared Movement rules, item 10).

Movement table (movement pass; Can Fly 2; speed byte 2):

| Mode | Source | u/t | u/s | Turn | Clip (id: clip, keys, loop) | Exits |
|---|---|---|---|---|---|---|
| Walk home (states 7, 8), wait in line | row speed only (idle 2.0) | 0.0667 | 4.0 | steering 0.3 / 0.1 | 98, 99: clip 0, no root | home reached |
| Rush approach (inner 0) and sequencer approach | row speed only (agro 7.0); the clip has no travel | 0.233 | 14.0 | steering | 101: clip 3, 15, loop | d below desired (5 + 10f, 7 to 13) or the slot range |
| Rush hold | none | 0 | 0 | aim 1.0 | 155 | d below 5 or above 15 |
| Back off (inner 2) | root + row speed | 0.204 + 0.233 = 0.437 | 26.25 | faces the hero | 102: clip 2, 15, loop | d above desired; point 17 from the hero |
| Charge run (state 7) | row speed + root x 2.0 | 0.415 | 24.9 | none; not solid | 44: clip 5, 28, loop | travelled 30 to 50 |
| Dodges 138, 139, 140 | none (missing ids: clip 0, no root; no row speed, a dodge is not a gait clip) | 0 | 0 | none | clip 0, 79 ticks | gate |
| Kind-6 hit reaction | root backward | 13.24 units over 45 ticks | 16.6 | snapped to the attacker | 152: clip 35, 12 | gate |
| Death | none | 0 | 0 | none | 153: clip 30 | removal |

What a port most likely gets wrong (movement): hovering it above the hero (Can Fly 2); giving its approach clip travel (the 14.0 is all row speed); making the charge solid on the run ticks.

#### Hit and hurt windows

- Quick cut: weapon 1 (radius 7.0, height 1.0) live s + 14 to s + 19 at rate 3.0, one hit of 30 plus a slow roll; heavy cut: weapon 1 live s + 16 to s + 21, one hit of 60 (unblockable) on the first touching tick, s + 20 for a still hero 2.5 to 4.5 away and s + 19 from 5.0 (the knight stays at about the hero's foot level, so the level figures apply); charge: weapon 1 live in two windows per 56 ticks (B ticks 18 to 25 and 38 to 45, then 74 to 81 and 94 to 101, ...), tested each tick of state 7 only; the first contact ends the test (state 8) with one hit of 10 and knockback 5.0 (hero velocity 3.0 sideways-forward and 4.0 up, long-fall state), slow at 50 percent for 3 to 5 s.
- Hit damage before armour and shield: 30, 60, 10. A guarding hero (L1, facing the knight) blocks slot 0 (state 4, no damage), blocks the charge (state 9, no damage) and not slot 1; blocking slot 0 at 4 < d < 10 opens slot 1 as a counter (chance 100 per tick of the blocked state, 57 ticks).
- Hurt shapes (Medium): body capsule of length 3.55 radius 0.96 and limb capsules of radius 0.3 to 0.7. The knight can be hit in every state except its dodges (outer 5 and 6), while dead, while hidden in state 8 with cue word 0, and in the ghost phase (the hit is absorbed).

#### Weapon reach

Conventions are those of "Weapon reach tables" in SPEC-0064 (shared field behaviour). Row 143, Medium. The tick text above is authoritative (the charge windows in the table are counted from the tick the charge clip is requested, one tick after the clip-tick numbers above). Weapon 1 has radius 7.0 and height 1.0 in every row.

| Slot (phase, clip) | Window ticks (level contact) | Centre forward / up / side | Tor ahead, at the tick | Tor ahead, from the start | Tor feet vs origin |
|---|---|---|---|---|---|
| 0 quick cut (B, clip 21) | s+14 to s+20 (s+14 to s+20) | -2.6 to 2.7 / 1.9 to 5.7 / -2.3 to 1.4 | -5.6 to 5.6 | -2.9 to 8.1 | -7.5 to 11.5 |
| 1 heavy cut (B, clip 23) | s+16 to s+21 (s+18 to s+21) | -1.2 to 4.3 / 2.8 to 8.3 / -4.3 to 2.5 | 0.4 to 11.0 | 1.4 to 12.6 | -6.0 to 14.0 |
| 2 charge, pass 1 (B, clip 5) | s+19 to s+26 (s+19 to s+26) | -2.9 to 2.3 / 1.9 to 3.7 / -3.4 to 1.1 | -3.6 to 8.0 | 3.8 to 18.3 | -8.0 to 10.0 |
| 2 charge, pass 2 (B, clip 5) | s+39 to s+46 (s+39 to s+46) | -1.4 to 2.9 / 1.9 to 4.9 / -3.3 to 0.9 | -7.6 to 5.8 | 10.2 to 23.2 | -7.0 to 12.0 |
| 2 charge, pass 3 (B, clip 5) | s+75 to s+82 (s+75 to s+82) | -2.9 to 2.3 / 1.9 to 3.7 / -3.4 to 1.1 | -3.6 to 8.0 | 27.0 to 41.5 | -8.0 to 10.0 |
| 2 charge, pass 4 (B, clip 5) | s+95 to s+100 (s+95 to s+100) | -1.3 to 2.9 / 1.9 to 4.9 / -3.3 to 0.9 | -7.6 to 5.8 | 33.4 to 46.4 | -7.0 to 12.0 |

The quick cut touches from 5.6 behind to 6.0 ahead at the tick; the heavy cut never touches a level hero closer than 0.4 ahead and reaches 11.6. The charge's four passes cover up to 46 units from where it started, because the knight flies on between passes (flyer travel included); the contact test ends at the first contact. The vertical band is large (7.5 below to 13.5 above the knight's origin).

#### Ghost cycle

The rules of SPEC-0029 Behavior 33 with this row's words. Reset: solid, phase countdown -1; tick 1 draws the solid length int[300, 420]. At its end the ghost phase starts: a 60-tick fade-out (opacity 0.5 to 0.1, hits absorbed from the first tick), then the ghost length int[60, 180] is drawn: the ghost phase lasts 120 to 240 ticks. Then the solid phase: a 60-tick fade-in (hits are real from the first tick, auto-target after the fade) and a solid length int[300, 420]: a cycle is 480 to 720 ticks.
- A hit in the ghost phase deals no damage and starts a reveal int[60, 180] in which normal hits land and the knight is auto-targetable; the hit also does not wake a sentry, because the absorb returns before the agro transition (it stays in its idle state until a hit lands or it notices the hero).
- While attacking (sequencer state 0 to 3: the two cuts) the knight is solid and targetable. The charge states 6 to 9 and 12, the blocked-cut state 4 and the recovery state 5 are not attacking: a charging knight that is in its ghost phase absorbs hits.
- The knight's defence still runs in the ghost phase: a Tor swing is blocked or dodged at the rates below, so the first swing of a combo may never reach the absorb test.
- Alpha = trunc(128 x opacity): 64 solid, 12 at the deepest fade.

#### Defence and counters

Melee, once per hero swing (hold below 1, not in a reaction, not mid-attack except sequencer states 0, 1 and 5, hero within 8.0 and in front within 60 degrees): the block roll comes first (r1 at most 10 for quick, 50 for normal and heavy), then the dodge roll (r2 below 90 for quick, 50 for the others). Quick swing: block 10 percent, dodge 80.1 percent (0.90 x 0.89), hit 9.9 percent. Normal or heavy swing: block 50 percent, dodge 24.5 percent (0.5 x 0.49), hit 25.5 percent. A block faces the hero in base state 20 for the length of the swing (Blocking Angle 0.707: a non-power swing is blocked, with no damage, when dot(knight facing, hero facing) is below 0.707, so swings from behind or from the same heading land); a dodge plays clip 0 (id 140 is absent) for 79 ticks and the knight cannot be hit meanwhile. After either, the hold int[15, 30] means a second defence can follow almost at once. Tor's power swing ignores a block. Missile: for each live hero shot meeting the capsule inside 15 ticks (hold 15), 99 of 100 give a dodge (79 ticks, clip 0); no missile block.
Counters: slot 0, trigger 6: evaluated on every tick of a hit reaction, with 0 < d < 15, the knight facing the hero (cosine above 0.707) and slot 0 off cooldown (int[0, 30]): the roll int[0, 100] is below 50 on 50 of 101 draws per tick, so a hit on the knight is answered by a quick cut with near certainty within a few ticks of the reaction; slot 1, trigger 5: sequencer state 4 with L1 held (above); slot 2, trigger 2 (after a dodge, chance 75): needs 10 < d < 20 while a dodge runs, but the melee defence needs the hero within 8.0, so it can only fire after a missile dodge (outer 6) with Tor's recover window open, which no hero shot provides: effectively dead.

#### Hero actions against this enemy

| Hero action | Result |
|---|---|
| Walks up to a sentry or a waiting knight | A sentry notices a still hero only within 7 (0.2 to 0.5 per cycle); a walker within 15 is noticed on every pass; a waiting knight wakes at 0.7, 7 or 2.8 units (standing) |
| Tor swings at a knight (not in the ghost phase) | Quick swing: 90 percent answered (10 block, 80 dodge); normal or heavy: 75 percent (50 block, 25 dodge); a dodge makes it untouchable for 79 ticks, then a hold of 15 to 30 ticks |
| Tor swings at a ghost-phase knight | Mostly dodged or blocked first; a swing that gets through is absorbed and opens a reveal of 60 to 180 ticks |
| Roberta's shots | Dodged on 99 of 100 shots that meet the capsule inside 15 ticks; absorbed in the ghost phase |
| Hero hits the knight | 200 hit points, defence value 0, doubled once out of agro; reaction 45 ticks, pushed back 12 units; answered by a quick cut (30 and 10 percent slow) |
| Guards (L1) | Quick and heavy cuts: the quick cut is blocked (and then answered by the unblockable heavy cut at 4 to 10 units); the heavy cut and the unblockable parts ignore the guard; the charge is blocked |
| Alarm group (Knights Crypts) | All three knights emerge within 45 ticks of the first one entering agro |

#### Edge cases for implementers

- The knight is invisible and untouchable while it waits (cue word 0), then plays a 96-tick emerge clip during which it can be hit.
- Every dodge uses a lookup-miss clip (clip 0, a hover loop): the knight is not hittable for 79 ticks and the hold after it is only 15 to 30. A port that gives it a dodge clip from the ground table (ids 53 to 55 exist in the table but are not requested by flyers) is wrong.
- The charge is aimed once and not steered; the hit ends only the strike test, not the run. The knight flies through and beyond the hero.
- Slot 2's counter (trigger 2) cannot fire in play; slot 1's counter and slot 0's hit-reaction counter can.
- The anger counter rises with every hit but nothing reads it.
- The ghost phase is short (120 to 240 of 480 to 720 ticks), but the absorbed hit costs the hero a swing and the reveal is long (60 to 180 ticks).
- A knight whose defence hold is below 1 evaluates a swing on the first tick of the swing, before the swing's active window: the dodge starts before the blow can land.

#### What a port most likely gets wrong

1. Treating the dodge as short: it is a 79-tick untouchable state.
2. Letting the charge steer or stop on the first hit.
3. Showing waiting knights and letting them be hit before they wake.
4. Making the ghost phase absorb the knight's own attacks: cuts make it solid; the charge does not.
5. Using the Dark Knight's strafe numbers: this knight is a rush flyer (approach 14.0, back off 26.25 per second).
6. Forgetting that the quick-swing block and dodge rates (10 and 80 percent) are far higher than the normal-swing rates (50 and 25).
7. Giving the notice check a range that shrinks with distance: the chance rises from 1 percent at 1 unit to 90 percent at 10, and the near/far ranges are tiny for a still hero.

#### Test vector

Death Knight in agro (t = 0 the tick the setter ran), hero standing still 12 units in front within 45 degrees, guard off, hero not attacking. t = 1 set-up. From t = 2 the per-tick choice is charge 14 percent, quick cut 64 percent (0.86 x 0.74), nothing 22 percent.
- Quick cut chosen at c: the knight flies 8 units to d <= 4.0 (35 ticks at 0.233), so s is about c + 36; B at rate 3.0; hit at c + 50 (30 damage, slow at 10 percent); B gate c + 120; end path c + 121 (c + 76 if the hero has left); recovery int[15, 60]; cooldown int[0, 30].
- Charge chosen at c: B requested c + 1 at rate 2.0; state 7 from c + 2 at 0.415 per tick; the knight has covered about 7.4 units when the first strike window opens at c + 20 (s + 19, Weapon reach table), but the hero 12 ahead is not touched yet: the swinging weapon volume only reaches him on s + 21 = c + 22, when the knight has covered about 8.2 units and he is about 3.8 ahead of it (the touch is the same first-touch rule as the other slots, with the knight's own run added to the clip travel; a still hero 4.0 to 7.0 ahead is touched on the opening tick, s + 19, 9.5 to 10.0 ahead on s + 22, 10.5 to 13.0 ahead on s + 21, while a start 7.5 to 9.0 ahead has no touch in pass 1 at all, a gap in the swing geometry). He takes 10 damage, knockback 5.0, slow at 50 percent for 3 to 5 s; state 8 at 0.324 per tick; the run ends after 30 to 50 units, at about c + 88 to c + 150; end path next tick; recovery int[15, 60]; cooldown int[30, 60].
- The hero swings at the knight (quick swing) in front of it: block 10 percent, dodge 80 percent (79 ticks, hold int[15, 30]), hit 10 percent.
- Over 600 ticks (20000 simulated runs, hero standing 12 units in front): mean 4.3 attacks (mostly quick cuts, 0.2 charges), 10 to 180 damage if every swing lands (mean 124). A knight that hovers at 7 to 10 units also throws heavy cuts (60) at 49 percent of its ready ticks.

#### Open questions

- Contact geometry of weapon 1 (radius 7.0, height 1.0) and of the charge. Resolved by the general rule (SPEC-0064, Shared field behaviour: Weapon contact and the hero's hurt shape); the distances are in the Weapon reach table; stand-in only for a port without attachment frames: a window hits a level hero inside the "at the tick" range of the slot's row (5.6 behind to 6.0 ahead for the quick cut, 0.4 to 11.6 ahead for the heavy cut) when his feet are inside the row's feet band.
- Whether a charge through scenery is stopped. Resolved (High; proactive (open close-out)): a region character is confined to its navigation mesh (a border edge is a wall that sets the blocked flag, SPEC-0056, Navmesh confinement step), and its other world walls are the cell wall lists, so the charge obeys them; a knight that has not moved for the 100-tick limit goes to state 12.
- Whether the lookup-miss dodge really plays for its 79 ticks. Resolved (High; proactive (open close-out)): yes; the request stores the missing id and clip 0 of the main set plays (the lookup-miss rule, SPEC-0040 Behavior 3D), whose gate is 79 ticks.
- Whether a sentry knight (cue word irrelevant) turns at the blend 1.0 toward its post each tick. Resolved (Medium-High; proactive (open close-out)): the two Death Knight sentry records have +0xC4 = 1, so they turn toward their stored look point each tick with the row's ground aim turn weight (SPEC-0038 Behavior 5, state 7), and that weight is 1.0 on the Death Knight row.


## Cogham Ixi Force

Four kinds make up the Ixi soldiers of the Cogham region: the Ixi Spearman (monster type 76, variant 0, row 159), the Dark Ixi Spearman (type 76, variant 2, row 161), the Ixi Archer (type 75, variant 0, row 156) and the Dark Ixi Archer (type 75, variant 2, row 158). The Ixi Henchman (type 75 variant 1, row 157, and type 76 variant 1, row 160) is the middle variant of the same two rigs; its guides are SPEC-0014 and SPEC-0063 Round 1, and this section only adds what changes when a Henchman stands in the field. All are class-17 common enemies and follow SPEC-0038 (think loop, pre-step, notice check, alarm groups, attack chooser, sequencer, counter choice, take-hit hook, death hook). A Spearman is a melee strafer (approach routine 4); an Archer is a keep-distance shooter (approach routine 3). The Dark variants have three to three and a half times the hit points and different defence, damage and notice values, but the same clips. Confidence: High for row values, clip data, attack arithmetic, the start states, the two-stage notice and the alarm path; Medium-High for the idle states (they are the shared states of the other batches); Medium for weapon contact geometry, the follow state of the Cogham Village escorts and the Coliseum hero start position. All times are ticks (60 per second); "s" is the tick on which the sequencer requests the first clip of an attack and "c" is the tick of the attack choice; every clip of these kinds has speed byte 2 (0.25 keys per tick at play rate 1.0), so a one-shot clip of F keys passes its 0.99 gate 4(F - 1) + 1 ticks after the request, one tick earlier from F = 25 keys (SPEC-0005).

### Shared rules for the Ixi force

#### What they have in common

- **Visible kinds.** All six rows (156 to 161) have Display in default 1. They are drawn and hittable in every state, they use the real state 2 (not state 21) when they notice the hero, and the setter's default case gives state 0, which the pre-step turns into the spawn record's default state on the same tick. There is no state 23, no clip 81 and no decloak clip 82 for these kinds. The only hidden case is a state-8 record with cue word 0 (one Dark Ixi Archer in the Abandoned Mines).
- **Weights in the attack manager** (SPEC-0038 Behavior 45): 8 on all six rows. The Spearmen (approach 4) wait in line when the manager refuses them; the Archers (approach 3) never wait. Stand-off radius for the wait (row +0xB0): Ixi Archer and Henchman archer 30, Dark Archer 50 (unused: the Archers never wait), Spearmen 10.
- **Rows differ in values, not in clips.** The Archer rows share one animation table and clip set (26 clips), and the three Spearman rows share one table (40 clips) except two ids (below). Variant 1 is the Henchman. Within each rig the Dark variant is a stronger row, not a new rig.
- **Target type (alignment) T.** 0 on the plain and Henchman rows; 6 on the Dark rows (158 and 161). It is the element of every projectile the enemy fires (SPEC-0029 Behavior 9, SPEC-0036 shield reduction).
- **Socialize.** All six rows have the socialize flag (row +0x40) set to 1, so Ixi in states 5, 6 and 7 chat with a nearby peer exactly as the Thief and Bandit do (road-brigands rules, SPEC-0038 Behavior 10): talk time float[120, 180) ticks (row +0x50 and +0x4C), quiet time float[row +0x58, row +0x54) = [360, 480) for the Archers and [420, 480) for the Spearmen, pair radius (row +0x44) 10.0 on rows 156 and 157, 12.0 on row 158, 20.0 on the Spearman rows. A pair needs the separation to be at most the sum of both radii, the same navmesh, the peer's quiet countdown below 1 and the peer not in states 1, 2, 3, 19 or 21 (a peer hidden in state 8 qualifies and is pulled out, SPEC-0038 Behavior 40). The quiet countdown is 0 at creation, so the first scan finds a peer on the first idle tick: two sentries 8 units apart start talking at once. The chat clip is id 14 on a draw of 0 and ids 111 to 114 on draws 1 to 4; the Ixi tables have only id 14 (Spearman clip 36, 18 keys, loops; Archer clip 21, 25 keys, loops), so four draws in five play the idle pick instead. While the talk countdown is positive the notice band is halved.
- **Map search.** The Spearman rows have the flag at 0. The three Archer-rig rows (156, 157, 158) have the flag at 1 with search type 0 (look for a world item of kind 145, a Bagguss), but their chance (row +0x124) and greedy range (row +0x168) are 0.0, so the roll int[1, 100] below 0 never passes and the Archers never leave their post for an item. Treat the Archer map search as absent.
- **Hurt shapes** (body capsules of the model, SPEC-0040 Behavior 12A; Medium): Spearman: bone 0 length 1.54 radius 0.46, bone 5 length 0.82 radius 0.38, bone 12 length 0.90 radius 0.46, four limb capsules length 0.62 radius 0.17. Archer: bone 0 length 0.79 radius 0.38, bone 3 length 0.53 radius 0.23, bones 4, 7 and 10 lengths 0.79, 0.68, 0.93 with radius 0.38 to 0.39, four limb capsules (0.57 to 0.63, radius 0.15) and four foot capsules (radius 0.30). Row radius and height only feed the chase stop and spacing (Spearman radius 2.0 height 4.0; Archer radius 1.5 height 3.5).
- **Death.** Death hook, base state 16, animation 58 once; the corpse is removed 360 ticks (row +0x80) after the clip reaches progress 0.95, and sinks 1/180 unit per tick once the countdown is 30 ticks old. Spearman death clip 10 (28 keys, gate 108, 0.95 at 104): corpse gone about 464 ticks after death. Archer death clip 24 (14 keys, gate 53, 0.95 at 52): about 412 ticks. Killed Ixi stay dead (SPEC-0038 Behavior 47). Drops: the spawn record's drop ordinal (offset 0xB0) names a placement of the region (SPEC-0038 Behavior 34); -1 (nothing) on 6 of 15 Archers, 7 of 26 Dark Archers, 11 of 21 Spearmen and 4 of 22 Dark Spearmen in play. None of the rows steals, so carried Neopoints are 0.
- **Hit reaction.** Can fly is 0 on all rows and the flying-hit word is 0, so every hit that is not blocked starts hit-reaction kind 0 (animation 57, outer 10; kind 5 never occurs because its trigger flag is a ghost-only word that stays 0): Spearman clip 11 (13 keys, gate 49, travels backwards about 3.4 units per second, so about 2.7 units in all), Archer clip 25 (13 keys, gate 49, 3.6 per second backwards). The reaction ends at the clip gate (outer 0, then the approach routine again). A hit while not in agro doubles the damage, enters agro at once through the setter and skips whatever state the enemy was in (SPEC-0038 Behavior 37).
- **Blocking Angle** (row +0x10C) 0.0 and row defence word (+0x114) 0.0 on all six rows.

#### Where they stand: default states and counts

Placements in play (reached packs and confirmed or plausible slot loads; the unreached test block 1259, which holds two Archers and two Spearmen, is excluded), by the default-state word of the spawn record. The state meanings are those of the earlier batches: 0 idle until hit or alarmed, 5 wander, 6 patrol, 7 sentry, 8 hide and emerge, 9 follow a named character.

| Kind (row) | Total | 0 | 5 wander | 6 patrol | 7 sentry | 8 hidden | 9 follow | Where |
|---|---|---|---|---|---|---|---|---|
| Ixi Archer (156) | 15 | 3 | 2 | 3 | 7 | 0 | 0 | Cogham Steppes 10 (5 sentries, 3 patrols, 2 wanderers), Cogham Village 2 (sentries), Abandoned Mines 3 (idle) |
| Dark Ixi Archer (158) | 26 | 3 | 2 | 8 | 9 | 4 | 0 | Abandoned Mines 13 (3 idle, 5 patrols, 4 sentries, 1 hidden), Cogham Steppes 10 (2 wanderers, 3 patrols, 5 sentries), Coliseum 3 (hidden) |
| Ixi Spearman (159) | 21 | 0 | 2 | 11 | 5 | 0 | 3 | Cogham Steppes 14 (11 patrols, 3 sentries), Cogham Village 7 (2 wanderers, 2 sentries, 3 escorts) |
| Dark Ixi Spearman (161) | 22 | 2 | 2 | 12 | 3 | 3 | 0 | Cogham Steppes 16 (2 idle, 2 wanderers, 9 patrols, 3 sentries), Abandoned Mines 3 (patrols), Coliseum 3 (hidden) |
| Ixi Henchman, archer rig (157) | 6 | 0 | 0 | 1 | 5 | 0 | 0 | Cogham Steppes: one ambient patroller (region 801) and the five sentries of the Chieftain fight's wave (region 808, SPEC-0014) |
| Ixi Henchman, spear rig (160) | 5 | 0 | 1 | 1 | 2 | 1 | 0 | Cogham Steppes 4 (ambient wanderer and patroller of region 801; the gate pair of region 807, two sentries in group 1), Meridell arena round 1 (hidden; SPEC-0063) |

Cogham Steppes is pack 822 (regions 823 to 836; the ordinary field) and pack 800 (regions 801 to 808; the Chieftain's yard, SPEC-0014). The Abandoned Mines Archers are slot-loaded character sets of the transition pack 874 (regions 875 and 876) plus the ordinary pack 866. Counts of alarm group bytes (255 = none): Ixi Archer 6 of 15 have no group (groups 1 to 5, 7, 8 on the rest); Dark Archer 17 of 26 (groups 1, 2, 3, 4, 6, 8); Ixi Spearman 8 of 21 (groups 0, 1, 2, 4, 5, 7, 8); Dark Spearman 14 of 22 (groups 2, 4, 5, 8). Squads (members that share a group byte inside one area): Cogham Steppes group 2 is one Ixi Archer, three Ixi Spearmen (patrols), two Dark Archers and three Dark Spearmen (patrols) of regions 831 and 834; group 5 is two Ixi Spearmen, one Ixi Archer (wanderer) and three Dark Spearmen of regions 827 and 830; groups 1, 3, 4, 7, 8 are two to five members each; Cogham Village group 1 is three escort Spearmen and one Archer; group 0 (a real group) is two Spearman sentries.

#### Spawn record values that drive the field states

Same record words as the other batches (SPEC-0006 kind 3, SPEC-0038 Layout): default word at +0x84; for state 5 the float at +0xBC (square side R), integers +0xC0 and +0xC4 (pause range) and +0xC8 (pause percent); for state 6 the s16 at +0xB8 (path), +0xBC (0 loop, other ping-pong), +0xC4/+0xC8 (pause at each point), +0xCC (percent) and +0xD0/+0xD4 (random stop range); for state 7 the integer +0xC4 (0 or 1); for state 8 the float +0xC0 (emerge distance), +0xC4 (cue word) and +0xC8 (selector, 0 everywhere here); for state 9 the integer +0xBC (id of the character to follow), the float +0xC0 (stay distance) and +0xC4 (0 or 1).

Values on disc:

| Kind | State 5 (R; pause range; percent) | State 6 (loop or ping-pong; pause at points; random stop) | State 7 (+0xC4 = 0 / 1) | State 8 (emerge; cue) |
|---|---|---|---|---|
| Ixi Archer | 30; 60 to 240; 25. 50; none | ping-pong 120 to 240; ping-pong 60 to 240; loop 60 to 240 (no random stops) | 4 / 3 | none |
| Dark Ixi Archer | 32; 0 to 240; 2. 30; 60 to 240; 25 | ping-pong 240 to 480 at the points (4; two of them also carry a 25 percent stop of length 0, which does nothing); ping-pong 120 to 240 (1); loop 60 to 480 (1, with a 50 percent stop of length 0); ping-pong without point pause but 25 percent stops of 240 to 600 (1) or 240 to 480 (1) | 4 / 5 | Coliseum 40.0, cue 1 (3); Abandoned Mines 80.0, cue 0 (1) |
| Ixi Spearman | 60; 0 to 60; 25. 85; 0 to 60; 25 | ping-pong without any pause (7); ping-pong 60 to 240 (2); ping-pong with 25 percent stops of 60 to 240 (2) | 1 / 4 | none |
| Dark Ixi Spearman | 15; 120 to 400; 15. 32; 0 to 240; 2 | loop 120 to 600 (2); loop without pause (1); ping-pong without pause (2); ping-pong 60 to 240 (2); ping-pong 240 to 480 (1); ping-pong 60 to 480 with a 25 percent stop of length 0 (1); ping-pong with 25 percent stops of 60 to 240, 120 to 320 or 240 to 480 (3) | 2 / 1 | Coliseum 40.0, cue 1 (3) |
| Henchman 157 / 160 | 160: 15; 120 to 400; 15 | 157: ping-pong, 25 percent stops of 240 to 600. 160: loop 120 to 600 | 157: 5 / 0. 160: 2 / 0 | 160: Meridell arena 40.0, cue 1 |
| State 9 (Ixi Spearman only) | | | | follow id 3, 5 and 7, stay distance 5.0, +0xC4 = 0 |

The three Cogham Village escorts (record ordinals 4, 6 and 8 of region 843) follow the characters with ids 3, 5 and 7 of the same region: Rose, Gershwin and Giovanni the Explorer, three townsfolk that patrol in the same alarm group 1 (Medium-High: the id word equals the record ordinal of the followed character on all three).

#### The idle states, per start state

These use the shared definitions of the other batches (wander, patrol and sentry rules are in the road-brigand and minion rules; the numbers that matter for the Ixi are repeated here). In every idle state the pre-step runs first each tick: awareness countdown, notice check (below), socialize scan, and for state 8 the wake test; the state body follows.

- **Default 0 (idle).** The think loop sets state 0, which stays there: it plays the idle pick (base animation 0: Spearman clip 30, 50 keys, loops; Archer clip 13, 20 keys, loops; a fidget variant from ids 115 to 119 after int[120, 600] ticks when the table has it: Spearman 115 to 117 (clips 34, 33, 32), Archer 115 only (clip 8)). No notice check, no socialize. It leaves only by a hit (agro with doubled damage) or by its alarm group (its group byte is 255 on every default-0 Ixi in play, so only a hit). These are the six slot-loaded Archers of the Abandoned Mines transition (three Ixi Archers, three Dark Archers) and two plausible Dark Spearmen of the Cogham Steppes.
- **Default 5 (wander).** Goal square of side R about the home point (R below 0.1: random triangle of the navmesh), walk id 2 (clip 20 Spearman, clip 23 Archer, 12 keys, loops, 5.0 units per second = 0.0833 per tick), pause rolls per the record, fidget id 3 once when the pause fidget timer expires (Spearman clip 32, Archer clip 8, 21 keys, gate 81). A pause plays the idle pick.
- **Default 6 (patrol).** Locator route from the s16 at +0xB8, walk id 4 (same clip as id 2), pause at each point int[+0xC4, +0xC8] (fidget id 6: Spearman clip 34, Archer clip 14, 50 keys, gate 196) and a random stop of int[+0xD0, +0xD4] ticks on a roll below +0xCC on the one tick per awareness cycle when the countdown is 0.
- **Default 7 (sentry).** Walks home when more than 0.5 away; at home with +0xC4 = 0 it stands id 7 for int[60, 600] ticks then plays the look clip id 8 once (the first thing a fresh sentry does is the look, because its look-in-progress word is -1 at creation) and repeats: Spearman ids 7 and 8 are both clip 32 (21 keys, one-shot, so the stand is a held last pose), Archer id 7 clip 11 (18 keys, loops) and id 8 clip 8 (21 keys, gate 81). With +0xC4 = 1 it turns to the stored look point every tick (blend = the row's aim weight: 0.7 on rows 156 and 157, 1.0 on the others) and plays id 7.
- **Default 8 (hidden).** Stands at home on the hide pick id 9 (Spearman clip 31, 14 keys; Archer clip 0, 20 keys) and wakes when the hero is closer than the emerge distance times F1 of the hero's action state (0.7 standing, 0.75 slow walk, 1.5 walking, 1.75 running, 1.0 other). The next tick requests the emerge id 10 (Spearman clip 25; Archer clip 10; both 20 keys, gate 77, one-shot) and the enemy enters agro on the gate tick (request + 77); the agro machine's first run is the next tick and the first attack choice the tick after. The emerge is skipped by a hit (damage doubled, agro at once). The ambush cue (camera and sound) runs only when the cue word is 0 (the Abandoned Mines Dark Archer). A hidden Ixi with cue word 1 is hittable and drawn; with cue word 0 (the hide pick is its current animation) it is neither drawn nor hittable until the emerge request. An emerge distance of 0 never wakes it. Emerge 80.0 on that Dark Archer means 56 for a standing hero, 60 slow walking, 120 walking, 140 running.
- **Default 9 (follow, Cogham Village Spearmen).** Each tick: look up the character whose id is the record's +0xBC; if it is gone, the wander body of state 5 runs in place on every tick (the class state stays 9; the goal square is the whole navmesh because the word that state 5 reads as the square side is the small integer id, and there are no pauses because the pause percent is 0), and following resumes if the character reappears. Otherwise adopt that character's gait number as its own gait, steer toward it and walk (walk id 11 = clip 20, 5.0 units per second for gait 0) while the horizontal distance is at least the stay distance (5.0). Closer than 5.0 with +0xC4 = 0 (all three): plays the idle pick and stops. So an escort shadows its villager along the villager's patrol, always ending 5 units behind it. The notice check runs in this state (a pass goes to state 2); the alarm-group check does not include state 9, so an escort never joins an alarm by itself, and socialize does not run in it.
- **Returning to the default state.** Every exit from agro (leash, lost-sight search ending, victory pose) runs the setter with 0, giving state 0, which becomes the default state on the next tick: 5 resumes wandering from where it stands, 6 resumes the patrol, 7 walks home and stands, 8 walks home and hides (the whole wake and emerge repeats), 9 resumes following, 0 stands. The same applies after the search of state 2 gives up.

#### Noticing the hero (two stages) and the alarm groups

States 3, 5, 6, 7, 9 and 2 run the notice check (SPEC-0038 Behavior 3). The awareness countdown reloads with the row's timer and counts down once per tick, so a cycle is the timer plus 2 ticks (32 ticks for the Spearman rows, 62 for the Archer rows) and the check can pass only on the one tick per cycle on which the countdown is 0. On that tick roll A int[1, 100] must be at most Awareness; then the band (range 5 to the row maximum, both scaled by F1 of the hero's action state: 0.7 standing, 0.75 slow walk, 1.5 walking, 1.75 running) decides: nearer than the scaled near edge is noticed with no further roll, farther than the scaled far edge is never noticed, in between roll B int[1, 100] must be below the chance interpolated from the near chance at range 5 to the far chance at the row maximum (unscaled), halved when the hero is outside the view cone and halved again by the Tattered Cloak. A blocked sight line replaces F1 by F2 unless the row has "always has line of sight" (row 156 only). Values:

| Row | Awareness | Timer (cycle) | View cosine | Range | Chance near / far | Always LOS |
|---|---|---|---|---|---|---|
| 156 Ixi Archer | 100 | 60 (62) | 0.9239 | 5 to 60 | 80 / 30 | yes |
| 157 Henchman archer | 100 | 60 (62) | 0.9239 | 5 to 100 | 90 / 60 | no |
| 158 Dark Ixi Archer | 75 | 60 (62) | 0.9239 | 5 to 70 | 80 / 30 | no |
| 159 Ixi Spearman | 75 | 30 (32) | 0.707 | 5 to 30 | 50 / 10 | no |
| 160 Henchman spear | 85 | 30 (32) | 0.707 | 5 to 40 | 70 / 10 | no |
| 161 Dark Ixi Spearman | 100 | 30 (32) | 0.707 | 5 to 70 | 90 / 10 | no |

Probability that one cycle passes, hero standing (F1 0.7), sight line clear, hero inside the view cone / outside it:

| Row | 6 units | 10 | 15 | 20 | 30 | 45 | 60 |
|---|---|---|---|---|---|---|---|
| 156 | 0.79 / 0.39 | 0.75 / 0.37 | 0.70 / 0.35 | 0.66 / 0.33 | 0.57 / 0.28 | 0 | 0 |
| 158 | 0.59 / 0.29 | 0.57 / 0.29 | 0.54 / 0.27 | 0.51 / 0.26 | 0.45 / 0.23 | 0.37 / 0.18 | 0 |
| 159 | 0.36 / 0.18 | 0.31 / 0.15 | 0.25 / 0.12 | 0.19 / 0.09 | 0 | 0 | 0 |
| 161 | 0.88 / 0.44 | 0.83 / 0.41 | 0.77 / 0.38 | 0.71 / 0.35 | 0.59 / 0.29 | 0.40 / 0.20 | 0 |

A hero closer than 3.5 (standing) is noticed whenever roll A passes. A walking hero (F1 1.5) is noticed out to 1.5 times the range (45 for a Spearman row 159, 90 for row 156, 105 for row 161), a running hero out to 1.75 times; the chance values do not scale. Example: a walking hero at 30 units is noticed by an Ixi Spearman with probability 0.07 per 32 ticks and by a Dark Ixi Spearman with 0.59 per 32 ticks.

**Stage 1: state 2 (alerted search).** A pass in states 5, 6, 7 or 9 (or 3) calls the setter with 2. The setter draws the search countdown trunc(float[60, 240)), stores the hero's present position as the last-seen point and sets state 2 (Display in default is 1). Per tick in state 2, in this order:

1. If the alert clip (id 18) and the search-end clip (id 17) are both not current, request the search idle id 16 (Spearman clip 37, Archer clip 16: 30 keys, one-shot, gate 116).
2. Aim at the last-seen point with the row's aim weight; the enemy does not walk.
3. Run the notice check again on the same tick (a new pair of rolls; the awareness countdown is still 0). A pass sends the enemy to agro at once, because **no Ixi table has an alert clip (id 18)**: with the clip missing the handler skips the 18 stage entirely.
4. No pass: the search countdown goes down by 1 (every tick, not only on cycle ticks). At 0 or below the enemy requests id 17 (Spearman clip 38, Archer clip 17: 15 keys, gate 57) and, when it has finished, runs the setter with 0 (default state). The notice check keeps running during clip 17, and a pass then still goes to agro.

So an Ixi that notices the hero on cycle tick t0 goes to agro on that same tick with probability p (the second roll on the same tick), and otherwise stands, plays the long search clip and gets one more chance every 32 or 62 ticks for the countdown (60 to 239 ticks) plus the 57 ticks of clip 17; if none of those chances passes it returns to its default state. The hero can therefore step out of sight after being noticed and the Ixi gives up after 117 to 296 ticks. The number of chances is the number of cycle ticks inside that window, about 4 to 9 for a Spearman row and 2 to 5 for an Archer row; a hero standing 10 units from a row 159 Spearman is noticed in the end with probability about 0.9 and a hero 25 units from it only through the walking-range bonus.

**Alarm groups** (SPEC-0038 Behavior 3). The first group member to enter agro broadcasts. Every other member in state 0, 3, 5, 6, 7 or 14 goes, at its next alarm check (int[15, 45] ticks apart), to state 2 and requests id 18 directly. A group member in state 8 plays its emerge clip and enters agro on its gate. A member in state 9 and members in states 2, 19 and others do nothing. Because the Ixi tables lack id 18, the request plays clip 0 of the main set (the table-miss rule) while the current-id word still reads 18, and the state-2 handler waits for the gate of that clip before going to agro, with no notice roll: **Spearman: clip 0 is its jab swing (15 keys, one-shot), gate 57 ticks; Archer: clip 0 is its 20-key looping hold, gate 79 ticks.** An alarmed member therefore stands in its swing or hold pose for 57 or 79 ticks and then fights, wherever the hero is (the enemy may then walk across the area, or sit at the leash limit and drop back at once). The alarm check ignores an enemy already in state 2, 19 or any other state not listed.

**Notice gate between groups.** The gate of SPEC-0038 Behavior 3 (step 1) applies only to Display-in-default 0 enemies, which none of these are; Ixi squads of different groups can all fight at once.

#### Losing the hero

Agro exits to the default state by the leash (no route to the hero for 181 consecutive ticks while the walk waypoint is 15.0 or more from the hero), the lost-sight search timing out (outer 9, counter above 120) and the victory pose after a killing blow (animation 73: Spearman clip 39, Archer clip 4; 37 keys, gate 144). There is no distance give-up: an Ixi with a route follows the hero indefinitely. The "always has line of sight" flag of row 156 affects only noticing; in agro the real sight-line test still starts the lost-sight search, so an Archer behind a wall walks to the last-seen point.

#### Hero actions against the shared model

| Hero action | Result |
|---|---|
| Stands still out of range of a wanderer, patroller or sentry | Nothing; the check needs the scaled range (a Spearman row 159 ignores a standing hero beyond 21 units) |
| Walks or runs through the area | Notice ranges grow by 1.5 or 1.75; a squad member that notices triggers the whole group's alarm path (clip 0 pose, then agro) |
| Hits a sentry, wanderer, patroller, idle or hidden Ixi | Damage doubled, agro at once, hit reaction kind 0, no search stage |
| Hits an Ixi during its search (state 2) | The same: doubled damage, agro |
| Wears the Tattered Cloak | Noticed at half chance and never through a blocked sight line (the Ixi stay visible anyway) |
| Fights one member of a group | Other members in states 0, 3, 5, 6, 7, 8 and 14 of that group commit within 15 to 45 ticks plus their pose time (57 or 79 ticks); escorts (state 9) stay out of it |
| Kills a squad | Nothing special; corpses sink and vanish (about 410 to 460 ticks) |

#### What a port most likely gets wrong, shared by the four kinds

1. Hiding or decloaking them. They are always drawn and hittable (Display in default 1); there is no clip 82 and no state 21.
2. Sending a noticed Ixi straight to agro. The first pass puts it in the search state (the second roll can still lose); only a second pass, or an alarm, starts the fight.
3. Playing a missing alert clip as nothing. For an alarm the missing clip 18 plays clip 0 (a jab swing or a hold pose) for 57 or 79 ticks first.
4. Treating state 7 as a statue. Sentries look around every 60 to 600 ticks, chat with neighbours (socialize) and walk back home.
5. Waking state-8 Ixi on a timer. They wake only by emerge distance times F1 or by the alarm group; a peer's socialize pull can also bring one out into a chat without waking it (SPEC-0038 Behavior 40).
6. Giving the Archers the Thief's item detour. Their map search flag is on but its chance is 0.
7. Forgetting the escort state: the three Cogham Village Spearmen walk beside villagers, not around a fixed post.

### Ixi Spearman and Dark Ixi Spearman (rows 159 and 161; Henchman row 160 for comparison)

The three Spearman rows are one rig, one animation table (40 clips) and one set of three melee slots. They differ in hit points, damage, the lunge reach, defence, notice values and (row 159 only) in the clip behind the forward strafe gait. The Henchman of row 160 is specified in full by SPEC-0063 Round 1 and SPEC-0014; its numbers are in the table so a port can use one routine for all three.

#### Identity and tunables

| Item | Ixi Spearman (159) | Dark Ixi Spearman (161) | Henchman (160) |
|---|---|---|---|
| Record, name on the bar | type 76 variant 0, "Ixi Spearman" | type 76 variant 2, "Dark Ixi Spearman" | type 76 variant 1, "Ixi Henchman" |
| Hit points | 50 | 170 | 65 |
| Body | radius 2.0, height 4.0 | same | same |
| Target type T | 0 | 6 | 0 |
| Approach routine | 4 (strafe): desired distance 10 + 10 f (f = int[20, 80] / 100, so 12 to 18), hold between 10.0 and 20.0; backbiter factor 1.0 (no effect); rush band 20 to 50 and desired band 10 to 15 unused | same | same |
| Aim weight in hold states (row +0x84) | 1.0 | 1.0 | 1.0 |
| Recovery after an attack | int[30, 240] | int[30, 180] | int[30, 180] |
| Notice, awareness timer, view cone | shared table (awareness 75, cycle 32, cone 0.707, range 5 to 30) | (100, 32, 0.707, 5 to 70) | (85, 32, 0.707, 5 to 40) |
| Missile defence: look-ahead and hold / dodge / block | 60 / 75 / 25 | 60 / 100 / 75 | 60 / 80 / 50 |
| Melee block chance quick, normal, heavy | 25, 25, 50 | 90, 90, 90 | 80, 80, 80 |
| Melee dodge chance quick, normal, heavy | 50, 50, 50 | 100, 90, 90 | 80, 80, 80 |
| Defence hold after any defence | int[60, 240] | int[60, 240] | int[60, 240] |
| Weapon volumes | weapon 1 radius 4.0, weapons 2 to 5 radius 2.0, all height 2.0 | same | same |
| Counter triggers | none | none | none |
| Status, steal, drain | none (status id 0 with chance 0 on every slot: never applies) | none | none |
| Wait-in-line stand-off radius | 10 | 10 | 10 |

Attack slots (k = table slot; "per tick" is the chance that one roll int[1, 100] is below int(chance); slot 3 is unused on all three rows):

| Row, slot | Type | Band (strict) | Chance, per tick | Facing cosine above | Range | Cooldown draw | Charge | Damage, knockback | Speeds | Orient |
|---|---|---|---|---|---|---|---|---|---|---|
| 159, 0 | melee jab | 0 to 10 | 50, 49 % | 0.9239 | 4.0 | int[30, 60] | none | 10, 0 | 1.0, 1.0, 1.0 | no |
| 159, 1 | melee | 7 to 12 | 0 (never) | 0.707 | 10.0 | int[30, 60] | none | 14, 0 | 1.0 | no |
| 159, 2 | melee lunge | 0 to 30 | 25, 24 % | 0.966 | 5.0 | int[60, 120] | kind 2, distance float[15, 30) | 25, 1.0 | 1.0, 1.0, 1.0 | no |
| 161, 0 | melee jab | 0 to 10 | 25, 24 % | 0.9239 | 4.0 | int[30, 60] | none | 25, 0 | 1.0 | yes |
| 161, 1 | melee | 7 to 12 | 0 (never) | 0.707 | 10.0 | int[30, 60] | none | 35, 0 | 1.0 | no |
| 161, 2 | melee lunge | 0 to 50 | 25, 24 % | 0.966 | 5.0 | int[60, 120] | kind 2, distance float[10, 50) | 50, 1.0 | 1.0 | no |
| 160, 0 / 2 | jab / lunge | 0 to 10 / 0 to 50 | 24 % / 24 % | 0.9239 / 0.966 | 4.0 / 5.0 | 30 to 60 / 60 to 120 | none / kind 2, 10 to 50 | 15, 0 / 35, 1.0 | 1.0 | no |

Slot 1 is dead data on all three rows (chance 0). The charge distance is drawn at each normal choice, between the two charge words in either order.

#### Animation ids and clips (rows 159, 160, 161; speed byte 2 throughout)

| Role | Id | Clip, keys | Notes |
|---|---|---|---|
| Idle base | 0 | clip 30, 50 keys, loops | a frame-stored clip (mode 0; SPEC-0005): its root records are the last eight bytes of each frame record and are zero at every key, so no travel |
| Hide / hold (strafe hold, rush hold) | 9, 31, 22 | clip 31, 14 keys, loops | |
| Emerge | 10 | clip 25, 20 keys, one-shot, gate 77 | |
| Walk, patrol, follow | 2, 4, 11 | clip 20, 12 keys, loops, 5.0 units per second | |
| Rest fidget, sentry stand and look | 3, 7, 8 | clip 32, 21 keys, one-shot, gate 81 | |
| Patrol fidget | 6 | clip 34, 50 keys, gate 196 | |
| Chat | 14 | clip 36, 18 keys, loops | ids 111 to 114 absent |
| Search idle / search end | 16 / 17 | clips 37 (30 keys, gate 116) / 38 (15 keys, gate 57) | id 18 absent |
| Forward strafe gait | 28 | row 159: clip 26, 7 keys, loops, 20.0 units per second (0.3334 per tick). Rows 160 and 161: clip 22, 11 keys, loops, 5.67 units per second (0.0945 per tick) | the one table difference between row 159 and rows 160 and 161 (row 159 also maps id 19 to clip 25, unused) |
| Sidesteps left / right | 30 / 29 | clips 28 / 27, 11 keys, loop, 5.67 per second sideways | |
| Back off | 32 | clip 29, 11 keys, loops, 9.56 per second backward (0.1593 per tick) | |
| Hit reaction | 57 | clip 11, 13 keys, gate 49 | travels back 3.4 per second |
| Melee dodge | 55 | clip 12, 16 keys, gate 61 | travels back 6.9 per second |
| Missile dodge | 53 or 54 | clips 23 / 24, 18 keys, gate 69 | about 3.9 per second sideways |
| Block / block-hit | 56 / 126 | clip 8 (15 keys, loops) / clip 5 (7 keys, gate 25) | |
| Death | 58 | clip 10, 28 keys, gate 108 (0.95 at 104) | |
| Victory pose | 73 | clip 39, 37 keys, gate 144 | |
| Slot 0 jab | B 34, D 36 | clip 0 (15 keys, gate 57), clip 2 (3 keys, gate 9) | A, C, E skipped |
| Slot 1 | B 39 | clip 13, 20 keys | never chosen |
| Slot 2 lunge | B 44, C 45, D 46 | clip 6 (7 keys, loops, 20.0 per second), clip 4 (22 keys, gate 85), clip 2 | A and E skipped |

Fidget ids 115 to 117 are present (clips 34, 33, 32), 118 and 119 are absent. The idle pick plays base id 0.

#### Wake and start

Field starts, notice, search and alarm behaviour are in the shared rules; the Spearman rows add nothing except: a hit or an agro entry puts the Spearman in agro with outer sub-state 0 (set-up) on the first agro tick, outer 4 inner 10 (hold, hold timer 0) on the second, and the first attack choice on the third (A + 2 when A is the tick on which the setter ran). The Spearman faces the hero with aim weight 1.0 in every hold tick, so its facing is correct for the chooser from the first choice on. From an alarm (state 2 with the clip-0 pose) agro starts after the jab pose clip passes its gate (57 ticks). A Spearman in state 9 that notices the hero stops following its villager and fights.

#### State table (agro)

| State | Id and clip | Gate / exits |
|---|---|---|
| Hold (outer 4, inner 10) | 31, clip 31 | no gate. Hold timer down by 1 per tick; while it is positive the distance tests apply (d > 20.0 gives approach, d < 10.0 with the lock below 1 gives back off); at 0 one roll r = int[1, 3] (1 stay, 2 sidestep left, 3 sidestep right; 2 and 3 become stay while the lock is up), the hold timer is redrawn int[60, 180] and then the distance tests run and win |
| Approach (inner 7) | 28 | until d is below the drawn desired distance (12 to 18): hold timer 0, inner 10 |
| Sidestep (inner 8, 9) | 30, 29 | hold timer down by 1 per tick, at 0 inner 10; a wall block gives inner 10 with a 120-tick lock; then the distance tests |
| Back off (inner 11) | 32 | until d exceeds the desired distance, or a wall block (lock 120) |
| Slot 0 jab | B 34 (clip 0) | s + 20 to s + 39 weapon 1 open; gate s + 57; end path s + 58 |
| Slot 2 lunge | B 44, C 45 | run until d is at most 5.0 or the drawn charge distance is used; C window c8 + 24 to c8 + 47; gate c8 + 85; end path c8 + 86 |
| Melee dodge (outer 5) | 55 | 61 ticks, unhittable |
| Melee block (outer 7) | 56 loops, 126 once | until the hero's swing ends |
| Missile dodge / block (outer 6 / 8) | 53 or 54 (69 ticks) / 56 | |
| Hit reaction (outer 10) | 57 | 49 ticks, then outer 0 |
| Victory pose | 73 | 144 ticks, then default state |
| Dead (base 16) | 58 | 108 ticks, corpse removed 360 ticks after the 0.95 point |

#### Attack chooser

Per SPEC-0038 Behavior 7: runs every agro tick in outer 1, 3, 4 and 11 while the recovery countdown is below 1 and the hero is attackable; cooldowns count down only while the hero is attackable. Slots are tested 2, then 1, then 0 (slot 3 is unused); the first that passes wins. Slot 2 needs a route to the hero (a charge slot always does); slot 0 passes with a route or with d below 10.0 (which always holds when it can pass). The roll thresholds are 24 (slot 2) and 49 on row 159 or 24 on rows 160 and 161 (slot 0). Resulting per-tick behaviour with every cooldown spent:

- d in 10 to the lunge band maximum (30 on row 159, 50 on rows 160 and 161): lunge at 24 % per tick.
- d below 10: lunge at 24 % first, else jab at 49 % (row 159) or 24 % (rows 160, 161): row 159 jabs on 37 % of ticks, rows 160 and 161 on 18 %.
- d above the band maximum: no attack; the strafe approach closes the distance.

On a choice the slot's cooldown is drawn int[30, 60] (jab) or int[60, 120] (lunge), the desired distance is redrawn, and the lunge's charge distance is redrawn. The cooldown is drawn again at the end path. After the end path the recovery countdown (30 to 240 on row 159, 30 to 180 on the others) must run out before the next choice; a hit reaction abandons an attack with no end path, so only the cooldown drawn at the choice limits the next choice.

**Jab (slot 0).** Choice tick c, sequencer state 0 on c. With d above 4.0 the Spearman walks forward with gait 0 (id 28: 0.3334 per tick on row 159, 0.0945 on rows 160 and 161) until d is at most 4.0 (and the hero and the steer point are on the walk mesh), then state 1 on the next tick (s). On s: A is missing, B 34 is requested and state 2 begins on the same tick. Weapon 1 (radius 4.0, height 2.0) is open on ticks s + 20 to s + 39 (frames 5 to 9); one hit (damage 10 on row 159, 25 on row 161, 15 on row 160), knockback 0, on the first window tick the weapon touches the hero. For a still level hero straight ahead, the exact weapon-versus-body test gives a maximum reach of 2.8 units ahead of the Spearman's centre while the chase stop keeps the centres 3.175 apart, so the jab misses at the chase stop. A hero 2.5 units away when the jab starts is touched on s + 24. Resolved: the still-hero-at-chase-stop question is closed for a level hero straight ahead; oblique, moving, or non-level targets can differ under SPEC-0018's asymmetric test. The clip passes its gate on s + 57 and the end path runs on s + 58, hit or miss. A blocked strike plays D (3 keys, gate 9). The clip carries 3.4 units per second of forward travel (about 3.3 units over the swing), and the step is dropped while the sequencer is in states 0 to 3 and the Spearman is within the sum of the body radii of the hero. A Dark Ixi Spearman (orient flag set) turns to the hero at weight 1.0 in states 1 to 3, so it tracks a hero who circles it during the 20-tick wind-up; rows 159 and 160 keep their facing from the choice tick.

**Lunge (slot 2, charge kind 2).** On the choice tick c: state 6 aims at the hero, finds A missing, requests B 44 (the 20.0 units per second run clip) and moves to state 7 in the same tick. From c + 1 each tick: when d is at most 5.0, state 8 on the next tick; otherwise when the distance covered since tick c has reached the drawn charge distance L, state 12. The run is a straight line along the facing set on c (no steering, no strike test). In state 8: C 45 is requested on its first tick (c8); the strike test runs every tick; weapon 1 is open on c8 + 24 to c8 + 47; a damaging contact would deal 25 on row 159, 50 on row 161 or 35 on row 160 (knockback 1.0, hero into state 48). All three rows share the same exact fixed-pose, level, straight-ahead no-stop samples: c8+24 at d=1.0–3.0, c8+27 at d=3.5, c8+28 at d=4.0–7.0. Those are geometric samples only. The charger is solid, and its run/strike root motion can push Tor; the actual first-hit tick, connection probability, and hit-dependent end path are runtime-only until body movement and exact weapon contact are played together (needs a runtime check; default if unresolved (rewrite choice): the stationary reference below). On a run that reaches the 5.0 distance gate, state 8 follows on the next tick; if L runs out first, state 12 and the end path follow and the strike never starts. The old reference hit-time formula and connection probabilities based solely on d0 and L are withdrawn as combat outcomes; d0-L describes only the unpushed run-distance threshold. The no-contact strike clip gate is c8+85 and its end path c8+86.

#### Movement

Strafe archetype (SPEC-0038 Behavior 44, outer 4): after every swing the Spearman stands 4 to 5 units from the hero, which is below 10, so it backs off (inner 11, 0.1593 per tick) until d exceeds its desired distance (12 to 18): 48 to 85 ticks, usually longer than the shortest recovery, so a jab at d below 10 is possible only when the recovery draw is short. With the hero further than 20 it approaches (row 159 at 20.0 units per second, so 15 units in 45 ticks; rows 160 and 161 at 5.67, so 15 units in 159 ticks), holds at 12 to 18 and sidesteps left or right for 60 to 180 ticks at a time (5.67 per second). The wall blocked flag ends a sidestep and sets the lock for 120 ticks.

The lunge makes the Dark Ixi Spearman the long-range threat: it can start a lunge from 50 units away, crossing the distance at 20.0 per second in a straight line; a hero who steps sideways out of the line during the run takes nothing, because the run does not steer and the strike test only starts in state 8.

Movement table (movement pass; rows 159, 160, 161; speed byte 2):

| Mode | Source | u/t | u/s | Turn | Clip (id: clip, keys, loop) | Exits |
|---|---|---|---|---|---|---|
| Wander, patrol, walk home, wait in line | root | 0.0833 on every key | 5.00 | steering 0.3 / 0.1 | 2, 4, 11: clip 20, 12, loop | goal or point reached |
| Idle pick | none | 0 | 0 | none | 0: clip 30, 50, loop (a frame-stored clip with no root travel) | - |
| Strafe approach (inner 7) and sequencer approach | root | row 159: 0.3334; rows 160, 161: 0.0945 | 20.0; 5.67 | steering | 28: row 159 clip 26 (7 keys, the run); rows 160, 161 clip 22 (11 keys) | d below desired (12 to 18) or the slot range |
| Circle (inners 8, 9) | root sideways | 0.0945 | 5.67 | faces the hero | 29: clip 27 (+x); 30: clip 28 (-x); 11, loop | hold count, blocked flag |
| Back off (inner 11) | root backward | 0.1593 | 9.56 | faces the hero | 32: clip 29, 11, loop | d above desired; point 20 from the hero |
| Strafe hold | none | 0 | 0 | aim 1.0 | 31 | hold count |
| Lunge run (slot 2, state 7) | root | 0.3334 | 20.0 | none; solid | 44: clip 6, 7, loop | horizontal d at most 5.0 (stops before the bodies touch at 3.175) or the charge distance |
| Melee dodge 55 | root backward | 7.39 units over the clip's 64 ticks (gate 61) | 6.93 | none | clip 12, 16 | gate |
| Missile dodges 53, 54 | root sideways | 4.78, 4.59 units over the clips' 72 ticks (gate 69) | 3.98, 3.83 | none | clips 23, 24, 18 | gate |
| Hit reaction kind 0 | root backward | 2.93 units over the clip's 52 ticks (gate 49) | 3.38 | snapped to the attacker | 57: clip 11, 13 | gate |
| Death | root backward | 1.68 units | - | none | 58: clip 10 | removal |

Worked example (back-off after a jab). Row 160 at 4.5 units after a swing, desired 15: inner 11 backs off at 0.1593 per tick; 10.5 units take 66 ticks (plus one more step after the stop decision). Row 159 approaching from 30 to 15 at 0.3334 per tick takes 45 ticks; rows 160 and 161 take 159 ticks.

What a port most likely gets wrong (movement): one strafe speed for all three rows (row 159's strafe-forward clip is the 20.0 run); reading the idle clip's root from the wrong table (it would drift); a lunge that steers.

#### Hit and hurt windows

Weapon windows are given above (one hit per open window; the hit list is cleared at the attack start and at each separate open window). The Spearman is hittable by the hero except in outer 5 and 6 (dodges), in base state 16 and when script-held. It is hittable while blocking (outer 7), but the hero's swing is judged by the guard rule. Damage to the hero: row value x 1.0, then shield and armour; a blocked strike (hero guarding with the hero's facing dot the Spearman's facing at most -0.5) still deals the reduced damage, plays D and ends the swing; knockback 1.0 puts a living hero in state 48.

#### Weapon reach

Conventions are those of "Weapon reach tables" in SPEC-0064 (shared field behaviour). Rows 159, 160 and 161, Medium. The tick text of the guide is authoritative for the hit ticks (for a hero 4 to 6 units ahead the lunge hit lands a few ticks after its window opens); "sC" is the tick the lunge clip is requested. The three rows share the jab and lunge clips; the Spearman (159) alone has the second slot's weapons 4 and 5.

| Slot (phase, clip) | Weapon | Window ticks (level contact) | Centre forward / up / side | Tor ahead, at the tick | Tor ahead, from the start | Tor feet vs origin |
|---|---|---|---|---|---|---|
| 0 jab (B, clip 0) | w1 (4.0 x 2.0) | s+20 to s+39 (s+20 to s+39) | 0.6 to 1.2 / 2.6 to 3.6 / -1.1 to -0.7 | -1.6 to 2.8 | 1.3 to 6.3 | -4.5 to 7.5 |
| 1 (B, clip 13; row 159 only) | w4 (2.0 x 2.0) | s+48 to s+63 (s+48 to s+63) | -0.7 to 1.1 / 1.7 to 3.2 / 0.3 to 0.8 | -3.0 to 3.6 | -2.3 to 5.0 | -3.0 to 4.5 |
| 1 (B, clip 13; row 159 only) | w5 (2.0 x 2.0) | s+48 to s+63 (s+48 to s+63) | -0.6 to 1.4 / 1.7 to 2.8 / -0.6 to 0.5 | -2.6 to 3.8 | -1.5 to 5.2 | -2.5 to 4.5 |
| 2 lunge (C, clip 4) | w1 (4.0 x 2.0) | sC+24 to sC+47 (sC+24 to sC+39) | 0.0 to 2.7 / 2.2 to 2.6 / -1.7 to 3.4 | -2.4 to 5.6 | -1.9 to 6.1 | -4.5 to 6.5 |

The jab is short: it touches a level hero only from 1.6 behind to 2.8 ahead of the Ixi at the tick, and from 1.3 to 6.3 ahead of where the Ixi stood when the clip began (the clip steps it forward). The lunge reaches 5.8 ahead at the tick. Rows 160 and 161 have slot 1 as a charge with a drawn distance of 0 to 0, so their state 7 ends on its first tick and the slot 1 windows are not reached.

#### Defence and counter rules

Melee defence, once per hero swing, on the first Spearman tick that sees the swing: not in outer 10; not in outer 2 unless the sequencer is in state 0, 1 or 5; defence hold below 1; within 8.0 of the hero and inside the hero's 60-degree facing arc. r1, r2 = int[1, 100]; r1 at most the block chance gives outer 7, else r2 below the dodge chance gives outer 5; on either the hold is set to 60. Probabilities per swing (block / dodge only / none):

| Swing class | Row 159 | Row 161 | Row 160 |
|---|---|---|---|
| Quick (Tor 5, 6; Roberta 5, 6, 7, 10) | 25 % / 36.75 % / 38.25 % | 90 % / 9.9 % / 0.1 % | 80 % / 15.8 % / 4.2 % |
| Normal (Tor 2, 3, 9; Roberta 2, 3, 8, 11, 12) | 25 % / 36.75 % / 38.25 % | 90 % / 8.9 % / 1.1 % | 80 % / 15.8 % / 4.2 % |
| Heavy (Tor 4, 7, 8, 12; Roberta 4) | 50 % / 24.5 % / 25.5 % | 90 % / 8.9 % / 1.1 % | 80 % / 15.8 % / 4.2 % |

After the defence ends (the swing is over, or the 61-tick dodge) the hold is redrawn int[60, 240]: no defence for that long. Tor's power step has no swing class and is never defended. A swing while the Spearman is winding up, running or striking (sequencer states 2 to 4, 6 to 8) is not defended. Missile defence (hero shots whose 60-tick look-ahead path meets the body): dodge if int[1, 100] is below the dodge value, then block if a second roll is below the block value (block wins): row 159: block 24 %, dodge only 56.2 %, none 19.8 %; row 161: block 74 %, dodge only 25.7 %, none 0.3 %; row 160: block 49 %, dodge only 40.3 %. A blocked hero shot is judged by the guard rule: it does nothing if the dot of the two facings is at or above the Blocking Angle 0.0. No Spearman row has a counter attack.

#### Hero actions against these enemies

| Hero action | Row 159 Ixi Spearman | Row 161 Dark Ixi Spearman |
|---|---|---|
| Swing at it in the open, in front, within 8 units | defended 62 % of quick and normal swings (25 % block, 37 % dodge), 75 % of heavy ones | defended 99 % of the time (block 90 %) |
| Same swing while it winds up or lunges | lands | lands |
| Swings during the 60 to 240 ticks after a defence ended | every swing lands: the defence hold blocks new defences | same |
| Stands still 15 units away | lunge at 24 % per tick (hits for 25), approach at 20.0 per second | lunge from 50 units (hits for 50 with the probability above), approach at 5.67 |
| Runs in circles | The lunge run does not steer: a sidestep wastes it | same |
| Roberta's shots | 80 % avoided | 99.7 % avoided |
| Guards (L1) facing it | blocks the jab and the lunge, reduced damage, knockback 1.0 still applies on a lunge | same |
| Kills it | 50 hit points: 13 undefended starting-sword hits of 4 | 170 hit points: 43 undefended hits of 4, so defence decides the fight |

#### Edge cases for implementers

- **Row 159 runs.** Its forward strafe gait is the 20.0 units per second run clip; the other two rows use the 5.67 clip. The same id gives different travel, so the speed must come from the clip, not from the id.
- **Slot 1 is dead data** on all rows (chance 0): the dormant 14, 35 and 21 damage values never apply. Slot 1's counter and trigger are absent.
- **Recovery after the end path only.** A hit reaction abandons an attack; the next choice then waits only for the cooldown drawn at the choice (30 to 120 ticks), so a Spearman that is hit during its lunge run or jab can start another attack sooner than one that completed its swing.
- **Wait in line.** The Spearman is a strafer: when the attack manager refuses it (not first in the list with starting gear) it plays the wait-in-line state (outer 12, walks to a stand-off point of radius 10 around the hero, faces it, re-picks every 600 to 800 ticks) and starts its swing only when released. A wandering group of Spearmen therefore fights one at a time at capacity 0, while Archers (approach 3) are never held back but still count their weight 8 in the budget.
- **Lunge reach versus walls.** The run stops at the first wall block (blocked flag) as any gait does; a lunge into a wall ends in state 12.
- **Alarm pose.** An alarmed Spearman plays its 15-key jab clip once (57 ticks, with the weapon bit set on frames 5 to 9) before agro; no strike test runs in that state, so it deals no damage.

#### What a port most likely gets wrong

1. Making row 159 slow like the Henchman. It closes distance at 20.0 units per second.
2. Giving the Dark Ixi Spearman a short lunge. Its charge distance is 10 to 50 and its band reaches 50; it hits for 50.
3. Per-tick chances: 24 % and 49 %, not 25 % and 50 %.
4. Applying the Henchman's 80 % block / 80 % dodge to the other rows. Row 159 blocks 25 % (heavy 50 %), row 161 blocks 90 %.
5. Letting the lunge steer or hit during the run, or ending it at the first contact. The strike is the 24-tick-delayed C window.
6. Running the end path after a hit reaction.
7. Forgetting that a hero who does nothing except stand still beyond the notice range is ignored.

#### Test vectors

**S1: Ixi Spearman (row 159), hero standing still 18.0 units in front, Spearman already in agro (t = 0 setter tick), cooldowns 0.** t = 1 set-up (outer 4, inner 10, hold 0); t = 2 first choice: slot 2 at 24 % per tick (d = 18 is inside 0 to 30; the jab needs d below 10). Take c = 2. Charge distance L from 15 to 30 (at least 13 needed to reach the d=5 run threshold absent body push). Tick 2: B 44 requested, state 7. The run uses 39 steps (ticks 3 to 41); tick 42 sees d at most 5.0; tick 43 state 8 requests C (c8=43). The weapon window is c8+24 to c8+47. If Tor remains stationary and the strike begins at d=5.0, the exact no-push geometric sample first touches at c8+28; this is not a confirmed live hit because the solid charge can push Tor during the strike. The actual hit tick, hit-dependent gate/end path, knockback, future choices and 600-tick damage total are runtime-only (needs a runtime check). This vector retains the attack start and charge threshold only; do not use the former c8+28 hit and downstream chain as a confirmed fight timeline.

**S2: Dark Ixi Spearman (row 161), hero standing 40.0 units in front, in agro.** The first choice is slot 2 at 24 % per tick (0 to 50 band); L = float[10, 50) is drawn then. With L at least 35 (probability 0.375): run 105 steps, state 8 at c + 107, hit (50 damage, knockback 1.0) at c + 135 (c8 + 28; 137 for c = 2; the window opens on c + 131), C gate at c + 192, end path at c + 193. With L below 35 (probability 0.625) the run uses up L (30 to 105 steps), state 12 follows, the end path comes about 3 ticks later and the Spearman stands 5 to 30 units short of the hero; from d above 20 it approaches at 5.67 per second (0.0945 per tick), and as soon as the recovery (30 to 180) and the cooldown (60 to 120) are spent it lunges again at 24 % per tick from whatever distance is below 50. In 600 ticks: 3 to 5 lunges, 1 to 3 connecting (50 each; a hero of 100 hit points at starting armour is at risk), jabs only after a connecting lunge when the recovery is short.

**S3: defence arithmetic.** Tor ordinary swing (quick class) at a standing row-159 Spearman 6 units ahead and in front: r1 at most 25 blocks (25 %), else r2 below 50 dodges (36.75 %), else (38.25 %) the swing lands: starting sword damage 4 on an enemy in agro, with the Spearman's hit reaction (clip 11, 49 ticks) interrupting any attack with no recovery countdown. Against row 161 the same swing lands 0.1 % of the time.

#### Open questions

1. Weapon contact geometry: the jab window and C window assume the hero is inside weapon 1's cylinder (radius 4.0, height 2.0 about an attachment frame). Resolved by the general rule (SPEC-0064, Shared field behaviour: Weapon contact and the hero's hurt shape); the distances are in the Weapon reach table; stand-in only for a port without attachment frames: the strike touches a level hero inside the "at the tick" range of the slot's row (1.6 behind to 2.8 ahead for the jab, 2.4 behind to 5.8 ahead for the lunge).
2. Resolved: the idle base clip (id 0, clip 30, 50 keys) is a frame-stored clip (mode 0; SPEC-0005) whose root record, the last eight bytes of each frame record, is zero at every key, so the idle pick has no travel (High). The same holds for the rig's id 13 (clip 35, 56 keys, the arena intro).
3. Whether the Spearman's facing at the lunge choice is exact. Resolved (High; proactive (open close-out)): yes; on tick c state 6 aims at the hero with the row's aim turn weight, which is 1.0 on rows 159 and 161, before it requests the run clip, so the run starts along the horizontal direction to the hero as he stands on tick c.
4. Distance metric of the hold and band tests. Resolved (High; proactive (open close-out)): horizontal, with the height difference dropped (the shared distance helper).

### Ixi Archer and Dark Ixi Archer (rows 156 and 158; Henchman archer row 157 for comparison)

**Player's answer (R-0149).** Swings and shots timed to its dodge: it dodges quick, normal and heavy swings 79, 69 and 49 times in 100 and a shot 79 in 100, and each dodge opens a 60 to 120 tick window; it holds 10 to 40 units away and never closes in or gives up (SPEC-0038 Behavior 44).

The three Archer rows are one rig (26 clips, one table) and keep-distance fighters. The Henchman archer (row 157) is specified by SPEC-0014; row 156 is the same fighter without the lobbed shot and with different defence and notice values; row 158 is a different weapon: it never shoots straight and never clubs, it lobs a burning shell.

#### Identity and tunables

| Item | Ixi Archer (156) | Dark Ixi Archer (158) | Henchman archer (157) |
|---|---|---|---|
| Record, name on the bar | type 75 variant 0, "Ixi Archer" | type 75 variant 2, "Dark Ixi Archer" | type 75 variant 1, "Ixi Henchman" |
| Hit points | 65 | 200 | 65 |
| Body | radius 1.5, height 3.5 | same | same |
| Target type T | 0 | 6 | 0 |
| Approach routine | 3 (keep distance): hold while the hero is 10.0 to 40.0 away; desired distance 10 + 30 f (16 to 34); retreat band 0 to 0 (unused) | same | same |
| Aim weight in hold (row +0x84) | 0.7 | 1.0 | 0.7 |
| Recovery after an attack | int[30, 120] | int[30, 120] | int[30, 120] |
| Missile defence: look-ahead and hold / dodge / block | 180 / 80 / 0 | 180 / 100 / 0 | 180 / 90 / 0 |
| Melee block (quick, normal, heavy) | 0, 0, 0 | 0, 0, 0 | 0, 0, 0 |
| Melee dodge (quick, normal, heavy) | 80, 70, 50 | 90, 75, 60 | 90, 75, 60 |
| Defence hold after any defence | int[60, 120] | int[60, 120] | int[60, 120] |
| Weapon volumes | weapon 1 radius 1.0 height 1.0, weapon 3 radius 1.0 height 0.2, weapons 4 and 5 radius 2.5 height 2.0 | weapon 1 radius 1.0 height 1.0, weapon 3 radius 1.0 height 0.2, weapons 4 and 5 radius 3.0 height 3.0 | as 156 |
| Waits in line | never (approach 3) | never | never |
| Map search | flag on, type 0, chance 0 (inert) | same | same |

Attack slots (slots 0 and 2 are missiles, slot 1 is melee; slot 3 unused; "per tick" as in the Spearman table):

| Row, slot | Type | Band (strict) | Chance, per tick | Arc | Range | Cooldown draw | Damage, knockback | Speeds | Missile or counter |
|---|---|---|---|---|---|---|---|---|---|
| 156, 0 | missile, flight 0 (straight) | 10 to 40 | 85, 84 % | 0.9239 | n/a | int[60, 120] | 5, 0 | 1.0 | speed 2.0 per tick, range 40 (20 ticks), radius 0.1, aim scatter up to 1.0, count 1, status id 0 at chance 0 (inert) |
| 156, 1 | melee swipe | 2 to 12 | 100, 99 % | 0.707 | 10.0 | int[60, 180] | 10, 5.0 | 1.3, 1.3, 1.0 | counter trigger 6 (hit reaction), chance 50 |
| 157, 0 / 1 | as row 156 (scatter 0; counter chance 75) | | | | | | | | |
| 157, 2 | missile, flight 1 (lob) | 15 to 80 | 25, 24 % | 0.0 | n/a | int[60, 240] | 10, 0 | 1.0 | speed 1.0 ramping to 0.3, range 100, radius 0.1, no area |
| 158, 0 | unused | | | | | | | | |
| 158, 1 | melee | 0 to 12 | 0 (never) | 0.707 | 10.0 | int[30, 60] | 7, 0 | 1.0 | none |
| 158, 2 | missile, flight 1 (burning lob) | 0 to 80 | 50, 49 % | 0.0 | n/a | int[60, 240] | 4, 1.0 | 1.0 | speed 0.6 ramping to 0.001, range 100, radius 0.5, count 1, fire (status 7) at 100 %, 8 to 12 s, area radius 5.0 for 300 ticks, counter trigger 6 at 25 |

#### Animation ids and clips (all three rows; speed byte 2)

| Role | Id | Clip, keys | Notes |
|---|---|---|---|
| Idle base | 0 | clip 13, 20 keys, loops | |
| Hold (keep distance) | 25 | clip 0, 20 keys, loops | also clip 0 = hide pick id 9 and the table-miss clip |
| Approach forward | 24 | clip 22, 11 keys, loops, 5.0 units per second (0.0833 per tick) | |
| Back off | 26 and 32 | clip 12, 16 keys, loops, 6.93 per second backward (0.1155 per tick) | |
| Walk, patrol | 2, 4 | clip 23, 12 keys, loops, 5.0 per second | |
| Rest fidget, look | 3, 8 | clip 8, 21 keys, gate 81 | |
| Patrol fidget | 6 | clip 14, 50 keys, gate 196 | |
| Sentry stand | 7 | clip 11, 18 keys, loops | |
| Chat | 14 | clip 21, 25 keys, loops | |
| Emerge | 10 | clip 10, 20 keys, gate 77 | same clip as the wind-up |
| Search idle / end | 16 / 17 | clips 16 (30 keys, gate 116) / 17 (15 keys, gate 57) | id 18 absent |
| Melee dodge | 55 | clip 12, 16 keys, loops, gate 63 | |
| Missile dodge | 53 / 54 | clips 6 / 7, 18 keys, gate 69, 3.9 per second sideways | |
| Hit reaction | 57 | clip 25, 13 keys, gate 49 | back 3.6 per second |
| Death | 58 | clip 24, 14 keys, gate 53 (0.95 at 52) | |
| Victory pose | 73 | clip 4, 37 keys, gate 144 | |
| Slot 0 | A 33, B 34 | clip 10 (20 keys, gate 77), clip 1 (12 keys, gate 45, release bit on frames 0 to 2) | C, D, E skipped |
| Slot 1 | B 39 | clip 3, 21 keys; window frames 12 to 15 | at rate 1.3 gate 62 |
| Slot 2 (157, 158) | A 43, B 44 | clip 10 (20 keys, gate 77), clip 2 (9 keys, gate 33, release bit on frame 2) | C, D, E skipped |

No block ids (56, 126) exist: the Archers cannot block. Missing as for the Spearmen: 18.

#### Wake and start

As the shared rules. In agro: set-up on the first agro tick (outer 3, inner 4, hold), first choice on the second. The Archer faces the hero in the hold at the row weight after a first blend of 0.25 in the same tick (effective blend about 0.78 on rows 156 and 157, 1.0 on row 158).

#### State table (agro)

| State | Id and clip | Gate / exits |
|---|---|---|
| Hold (outer 3, inner 4) | 25, clip 0 | d above 40.0 gives approach (inner 3); d below 10.0 gives back off (inner 5); otherwise stay |
| Approach (inner 3) | 24 | until d is below the drawn desired distance (16 to 34), then hold |
| Back off (inner 5) | 26 | steer to the point 40.0 units from the hero along the line away; until d exceeds the desired distance |
| Slot 0 shot (156, 157) | A 33, B 34 | A gate s + 77, B requested the same tick; arrow released on B + 1; B gate B + 45; end path B + 46 (= s + 123) |
| Slot 2 lob (157, 158) | A 43, B 44 | A gate s + 77; B requested the same tick; shell released on B + 8; B gate B + 33; end path B + 34 (= s + 111) |
| Slot 1 swipe (156, 157) | B 39 | window s + 37 to s + 49 at rate 1.3; gate s + 62 without contact (end path s + 63), s + 69 with a hit (end path s + 70) |
| Melee dodge (outer 5), missile dodge (outer 6) | 55, 53 / 54 | 63 and 69 ticks, unhittable |
| Hit reaction (outer 10) | 57 | 49 ticks |
| Victory pose, dead | 73, 58 | as the shared rules |

#### Attack chooser

Slots are tested 2, 1, 0 (3 unused). Row 156: slot 1 first (2 < d < 12, facing above 0.707, 99 % per tick), then slot 0 (10 < d < 40, facing above 0.9239, 84 % per tick). So with every cooldown spent: d from 10 to 12 swipes (99 %), d from 2 to 10 swipes, d from 12 to 40 shoots at 84 % per tick, beyond 40 or inside 2 nothing. Row 157 adds the lob first (15 < d < 80, 24 % per tick, any facing in front), so at 15 to 40 units a Henchman archer picks the lob on 24 % of ticks and the straight shot on 64 % of ticks. Row 158 has only the lob: 0 < d < 80 at 49 % per tick, facing above 0.0 (anywhere in the front half plane), so a Dark Ixi Archer lobs from point-blank range and from 80 units alike.

A missile slot needs no route. State 0 passes to state 1 as soon as the hero and the steer point are on the walk mesh, otherwise a facing cosine of 0.95 or more is needed, otherwise it turns (blend 0.1). Neither missile slot sets the orient flag, but a missile slot turns the Archer toward the hero every tick of the strike state at the row's aim turn weight (SPEC-0038 Behavior 13, state 2); the shot is aimed from the emit point at the hero's position at the release tick.

**Straight shot (rows 156, 157 slot 0).** Choice tick c; state 0 on c; state 1 on s = c + 1: A (33, clip 10) is requested and takes 77 ticks; on tick s + 77 B (34, clip 1) is requested and state 2 begins; the release bit is active on frames 0 to 2, so the arrow is created on tick s + 78 (B + 1) and no second arrow follows (spawn count 1, one release window). The arrow leaves emit point 0 toward the hero's position plus 2.5 up (plus the scatter: up to 1.0 on row 156, none on row 157, plus 5.0 when the Archer is outside the camera cone: dot of camera forward and the unit vector from the camera to the Archer below 0.85), flies 2.0 units per tick for 20 ticks (40 units), damage 5, radius 0.1; a hero 30 units away is reached about 15 ticks after the release. B passes its gate on s + 122 and the end path runs on s + 123; recovery int[30, 120], and the slot cooldown (int[60, 120]) is drawn at the choice and again at the end path.

**Lob (rows 157 and 158 slot 2).** Same A (77 ticks), then B 44 (9 keys): the shell is created on B + 8 (frame 2). It is a flight-mode-1 shot: aim point = the hero's position (no height offset), horizontal distance d, speed s = v + u squared x (v_end - v) with u = 1 - (d - bmin) / (bmax - bmin) (bmin and bmax are the slot band, 0 and 80 on row 158, 15 and 80 on row 157), flight time T = d / s ticks, launch height chosen so that the shell reaches the aim height after T ticks (gravity 32/3600 per tick squared). Row 158 flight times (v 0.6, v_end 0.001): d = 10: 71; 20: 76; 30: 82; 40: 89; 60: 107; 80: 133 ticks. Row 157 (v 1.0, v_end 0.3): d = 15: 50; 30: 51; 50: 59; 80: 80 ticks. The shell lands about T ticks after the release (a few ticks earlier, since the contact test lifts it by its radius). It is aimed at where the hero stands at the release tick, not at where the hero will be.

**What a lob does on the hero.** If the shell's sphere (radius 0.5 on row 158) meets the hero's capsule on its way down, it is a direct hit: the hero takes 4 (row 158) or 10 (row 157), the status roll runs (row 158: fire at 100 %, 8 to 12 seconds, so always), and the shell is released (no patch). If it reaches the ground first (the hero moved away), row 158's shell enters area mode (radius 5.0 for 300 ticks), and the area pulses: the timer is loaded once at landing (tick L), the first pulse is on tick L + 61 and further pulses follow every 60 ticks while the hero is inside the radius and within 1.0 of the ground (SPEC-0029 Behavior 9, Area mode). Each pulse deals 4 with no block test and rolls the fire status (100 %); a 300-tick area allows at most 4 pulses (L + 61, L + 121, L + 181, L + 241). Row 157's shell has no area. The patch keeps one of the ten projectile slots for 300 ticks.

**Club swipe (rows 156 and 157 slot 1).** Chosen on 99 % of ticks inside 12 units when the cooldown is spent. With d above 10.0 the Archer first walks forward (id 24) until d is at most 10.0, then B 39 is requested on s (A is missing). The swipe plays at rate 1.3 (a key every 3.08 ticks): window ticks s + 37 to s + 49 (frames 12 to 15, weapons 4 and 5, radius 2.5), one hit of 10 with knockback 5.0 at s + 37; after a hit the rest of the clip runs at 1.0, gate s + 69 and end path s + 70; without contact gate s + 62 and end path s + 63. The clip has no forward travel and the swipe's volume is only 2.5 units across, so a hero more than about 3 to 4 units away is never touched, however close the chooser thinks it is (Medium; see the open questions).

**Counter.** Trigger 6 means: on every tick of the Archer's hit reaction, if the slot is eligible (cooldown below 1, d inside the slot's band, facing above the slot's arc) one roll int[0, 100] is drawn and a roll below the counter chance starts the slot at once, replacing the reaction. Row 156: the swipe, chance 50 (about 50 % per tick, so within 3 ticks on average) if the hero is 2 to 12 units away; row 157: chance 75; row 158: the lob, chance 25 per tick at any distance from 0 to 80, so a Dark Ixi Archer that is hit while its lob is ready answers with a lob in a few ticks (its 77-tick wind-up begins at once).

#### Movement

Keep-distance archetype: the hold is the normal state; beyond 40.0 the Archer walks forward at 5.0 units per second until closer than the drawn desired distance (16 to 34); inside 10.0 it backs off at 6.93 per second toward a point 40.0 from the hero until beyond the desired distance. Both gaits are slower than a walking hero (13.5 per second) and a running hero (19.3 per second), so the Archer cannot keep its distance against a hero who closes. During an attack (outer 2) the Archer stands still for 112 to 124 ticks; it moves only in the approach and back-off states between attacks, and it never strafes.

Movement table (movement pass; rows 156, 157, 158; speed byte 2):

| Mode | Source | u/t | u/s | Turn | Clip (id: clip, keys, loop) | Exits |
|---|---|---|---|---|---|---|
| Wander, patrol, walk home, wait in line | root | 0.0833 | 5.00 | steering 0.3 / 0.1 | 2, 4: clip 23, 12, loop | goal or point reached |
| Keep-distance walk-in (inner 3) | root | 0.0833 | 5.00 | steering | 24: clip 22, 11, loop | d below desired (10 + 30f, 16 to 34) |
| Keep-distance hold (inner 4) | none | 0 | 0 | 0.25 blend then the row weight: Ixi Archer 0.7 (0.78 effective), Dark Ixi Archer 1.0 | 25 | d above 40, below 10 |
| Back off (inner 5) | root backward | 0.1155 average; per key -0.386 to +0.023 | 6.93 | faces the hero | 26: clip 12, 16, loop | d above desired; point 40 from the hero |
| Retreat (inner 6) | never entered: the retreat band is 0 to 0 | - | - | - | - | - |
| Missile dodges 53, 54 | root sideways | 4.78, 4.58 units over the clips' 72 ticks (gate 69) | 3.98, 3.82 | none | clips 6, 7 | gate |
| Hit reaction kind 0 | root backward, a little sideways | 3.11 units over the clip's 52 ticks (gate 49) | 3.59 | snapped to the attacker | 57: clip 25 | gate |
| Death | none | 0 | 0 | none | 58: clip 24 | removal |

Worked example (back-off). A hero walks up to 8 units from an Archer whose desired distance is 25: inner 5 backs off at an average 0.1155 per tick toward the point 40 from the hero; with the hero standing still at 8 the Archer needs 17 units, about 147 ticks (its clip lurches: one of every 16 keys steps forward). A walking hero closes at 13.5 - 6.93 = 6.6 u/s.

What a port most likely gets wrong (movement): a retreat that turns its back (the retreat state never runs for these rows; the back-off walks backward facing the hero); a constant back-off speed.

#### Hit and hurt windows

The Archers are hittable in every state except outer 5 and 6, death, script hold and a hidden state-8 record with cue word 0. Damage to the hero: arrow 5, lob 4 or 10, swipe 10 with knockback 5.0 (hero into state 48), before shield and armour. A guarding hero facing the shot blocks it by the projectile rule (SPEC-0029 Behavior 9: the guard query and the dot of the shot's direction with the hero's facing below 0); a blocked arrow still deals max(1, 0.25 D) and no status.

#### Weapon reach

Conventions are those of "Weapon reach tables" in SPEC-0064 (shared field behaviour). Rows 156 and 158, Medium. The swipe is the only melee attack; it uses weapons 4 and 5 together, which share one place, and the archer does not move during it. The windows are counted from the tick the swipe clip is requested; the guide's tick text is authoritative.

| Row, slot (phase, clip) | Weapons | Window ticks (level contact) | Centre forward / up / side | Tor ahead, at the tick | Tor ahead, from the start | Tor feet vs origin |
|---|---|---|---|---|---|---|
| 156 Ixi Archer 1 (B, clip 3) | w4 and w5 (2.5 x 2.0) | s+37 to s+49 (s+37 to s+49) | 0.3 to 0.9 / 2.1 to 2.8 / 0.0 | -2.4 to 3.4 | -2.4 to 3.4 | -3.5 to 5.0 |
| 158 Dark Ixi Archer 1 (B, clip 3) | w4 and w5 (3.0 x 3.0) | s+48 to s+63 (s+48 to s+63) | 0.3 to 0.9 / 2.1 to 2.8 / 0.0 | -3.0 to 4.2 | -3.0 to 4.2 | -4.5 to 6.0 |

The swipe is short-ranged: 3.4 ahead (Dark 4.2) at most, though the sequencer starts it from farther away. The Henchman archer (row 157) has the same clip and weapon figures as the Ixi Archer.

#### Defence and counter rules

Melee defence (once per swing; within 8.0 of the hero, inside its 60-degree arc; not in outer 10; not in outer 2 unless the sequencer is in state 0, 1 or 5; hold below 1): no block (block values 0), dodge if int[1, 100] is below the dodge value: row 156: 79 % of quick swings, 69 % of normal, 49 % of heavy; rows 157 and 158: 89 %, 74 %, 59 %. Missile defence (look-ahead 180 ticks): dodge with probability 79 % (row 156), 89 % (row 157), 99 % (row 158); no block. A defence sets the hold to 180; at the defence exit it is redrawn int[60, 120], and no new defence starts while it is positive. A Dark Ixi Archer therefore avoids 99 % of the Roberta shots that reach it, but is open for 60 to 120 ticks after each dodge. Counters as above.

#### Hero actions against these enemies

| Hero action | Ixi Archer (156) | Dark Ixi Archer (158) |
|---|---|---|
| Stands 30 units away | arrow (5) every 184 to 244 ticks plus about 1 tick of chooser wait; first arrow 79 ticks after the choice | lob (4 and fire on a direct hit; the patch it leaves never hurts) every 172 to 352 ticks; first shell 86 ticks after the choice, landing about 78 ticks later (flight time 82 less the radius lift) |
| Keeps moving | arrows miss (aimed at the position at the release tick, speed 2.0, 20 ticks), lobs land behind the hero | same; patches stay 300 ticks |
| Closes inside 12 units | swipes at 99 % per tick (10, knockback 5.0) but only touches a hero inside about 3 units; backs off at 6.93 per second | still only lobs (49 % per tick, point-blank allowed) |
| Hits it in melee | 79 / 69 / 49 % dodged; a hit starts reaction 49 ticks and a swipe counter at about 50 % per tick if 2 to 12 away | 89 / 74 / 59 % dodged; a hit answered by a lob counter at 25 % per tick |
| Roberta's shots | 79 % avoided | 99 % avoided |
| Guard (L1) | blocks arrows and the swipe when the hero faces them (reduced damage, no status) | a lob that meets a guarding hero is a blocked hit: max(1, a quarter of 4), no fire, no patch |
| Kills it | 65 hit points: 17 undefended starting-sword hits of 4 | 200 hit points: 50 hits |

#### Edge cases for implementers

- **One arrow per attack.** Spawn count 1 and a single release window; no second arrow on any row.
- **No instant shots.** The wind-up is the 77-tick draw (clip 10), the same clip as the emerge.
- **Archers turn during an attack.** The orient flag is off on every slot, but the strike state of a missile slot aims at the hero every tick at the row's aim turn weight. A hero who walks behind the Archer after the choice is still hit by the arrow, which is aimed at the hero's current position.
- **The lob's point-blank case.** With bmin = 0 on row 158 the speed formula never leaves the tabled range: at d = 1 the shell crawls at 0.016 per tick and lands about 60 ticks later, almost vertically.
- **Row 158 slot 1 is dead data** (chance 0, no trigger); row 158 has no slot 0.
- **Shells share the ten-slot projectile pool.** A landed patch keeps its slot for 300 ticks; with a full pool a new lob creates nothing but the shot counter still advances (the lob is consumed).
- **Alarmed Archer.** An alarm puts it in state 2 with the 20-key looping hold clip 0 as the stand-in alert pose; the handler waits 79 ticks for that clip's gate before agro, with no notice roll.
- **Row 156 notices through walls** (always has line of sight) at 5 to 60 units; the others need a clear sight line unless the hero is within the near edge.
- **Item search.** None (chance 0).

#### What a port most likely gets wrong

1. Treating the Dark Ixi Archer as a straight shooter or as a club fighter. It has one weapon: the burning lob, at any distance from 0 to 80.
2. Instant fire. 77 ticks of wind-up come first, and the Archer is committed to the shot until its end path (it keeps turning toward the hero at the row's aim turn weight).
3. Aiming the arrow along the Archer's facing. It is aimed at the hero's position at the release tick.
4. Letting the Archer walk or back off during the sequencer. It stands still for 112 to 124 ticks.
5. A hit reaction that does not counter. Rows 156 and 157 counter at 50 and 75 per cent per tick, row 158 at 25 per cent.
6. Giving the Archers a block animation: none exists, block values are 0.
7. Making the lob's area hurt at the wrong times: row 158's 300-tick area of radius 5.0 pulses first on the 61st tick after landing, then every 60 ticks (at most 4), only for a hero inside it and on the ground (SPEC-0029 Behavior 9, Area mode).

#### Test vectors

**A1: Ixi Archer (row 156), hero standing still 30.0 units in front inside the camera cone, Archer already in agro (t = 0 setter tick), cooldowns 0.** t = 1 set-up; t = 2 first choice at 84 % per tick (slot 1 does not apply at 30, slot 0 does). Take c = 2: s = 3; A requested on 3, gate on 80; B requested on 80; arrow created on 81 and reaches the hero on tick 94 (5 damage; status roll 1 in 101 applies stun with 0 seconds: no effect); B gate on 125; end path on 126; recovery 30 to 120 and the slot cooldown 60 to 120 start on 126; the next choice comes on 126 + max(recovery, cooldown) + G, that is between 186 and 246 plus G; the second arrow on that tick + 79, the third a further 124 + 60 to 120 + 79 later. In 600 ticks: 3 arrows (2 to 3), 10 to 15 damage on a motionless hero. G is geometric with probability 0.84 per tick (mean 1.2).

**A2: Dark Ixi Archer (row 158), hero standing still 30.0 units away, Archer in agro.** t = 2 first choice at 49 % per tick (mean wait 2 ticks). Take c = 2: s = 3; A gate on 80; B requested on 80; the shell is created on 88; B gate on 113; end path on 114. The shell travels T = 82 ticks (d = 30) and reaches the standing hero on tick 166 (its 79th position, from the emit point 1.71 up and 1.38 behind the Archer): direct hit, 4 damage and fire for 8 to 12 seconds. A hero who ran off lands the shell on the ground at the aim point instead; its 300-tick area pulses from tick L + 61 (4 and fire each, every 60 ticks, at most 4) for a hero standing inside its radius 5.0. The next choice comes on 114 + max(recovery 30 to 120, cooldown 60 to 240) + G, between 174 and 354. In 600 ticks: 2 to 3 shells.

**A3: Ixi Archer (row 156), hero standing 6.0 units in front.** t = 2 first choice: slot 1 at 99 % per tick (2 < 6 < 12). Take c = 2: d is already below 10.0, state 0 passes at once, s = 3, B 39 is requested on 3; the weapon window is ticks 40 to 52; the swipe touches the hero only if the hero is inside the 2.5-unit volume at the club (at 6 units it whiffs): gate on 65, end path on 66 (no contact; with contact gate 72 and end path 73). Afterwards the Archer backs off (d below 10) at 0.1155 per tick: it needs 87 to 242 ticks to reach 16 to 34 units; recovery 30 to 120 and cooldown 60 to 180 block the next choice meanwhile.

**A4: counter.** A hero hits a Dark Ixi Archer 8 units away whose lob cooldown is spent: reaction (animation 57, 49 ticks) starts; on each reaction tick the counter roll succeeds with 25 of 101; the lob sequence starts on the first success (expected after 4 ticks), so the shell is released about 86 ticks after the hit.

#### Open questions

1. Whether the swipe of rows 156 and 157 can reach a hero further than its 2.5-unit volume allows (contact geometry of weapons 4 and 5). Resolved by the general rule (SPEC-0064, Shared field behaviour: Weapon contact and the hero's hurt shape); the distances are in the Weapon reach table (3.4 ahead at most, Dark Ixi Archer 4.2); stand-in only for a port without attachment frames: the swipe touches a level hero from 2.4 behind to 3.4 ahead of the archer (Dark: 3.0 behind to 4.2 ahead).
2. Whether a lob that reaches the hero's capsule releases without a patch (direct hit rule) for a hero standing exactly at the aim point. Resolved (Medium-High; proactive (open close-out)): yes; the character hit test of the projectile step runs before the world collision and a release there skips it (SPEC-0029 Behavior 9), so a shell that touches the hero's capsule is a direct hit and leaves no patch.
3. Camera-cone test input for the 5.0 scatter on enemies. Resolved (High; proactive (open close-out)): the active camera's forward vector and position; the shooter is outside the cone when the dot product of the camera forward with the unit direction from the camera to the shooter is below 0.85 (SPEC-0038 Behavior 43).
4. The movement predicate branch of the idle pick (base id 97 is absent from both tables). Resolved (High; proactive (open close-out)): a missing id plays clip 0 of the main set (the lookup-miss rule, SPEC-0040 Behavior 3D), so that branch plays clip 0.

### Ixi Henchman in the field

The Henchman rows have full guides (SPEC-0014 and SPEC-0063 Round 1). In the field they are ordinary Ixi: the same notice values (shared table), the same two-stage notice with the search clips (ids 16 and 17 exist in both tables, id 18 does not), the same alarm path (57 ticks on the spear rig, 79 on the archer rig), the same socialize rule and the same field states. Placements in play: the ambient trio of region 801 (row 160 wanderer R 15 with pauses of 120 to 400 ticks at 15 percent, row 160 patroller with a loop and point pauses of 120 to 600 ticks, row 157 patroller with a 25 percent stop of 240 to 600 ticks), the gate pair of region 807 (row 160, sentries, alarm group 1) and the five wave Henchmen of region 808 (row 157, sentries, no group, no drop). The Chieftain fight (SPEC-0014) hands the gate pair and the wave to the alerted search by script, after which the missing alert clip sends them to agro on the first notice pass. A Henchman can be met outside the boss fight only as the ambient trio.

### Altador Coliseum round 2: the Dark Ixi group

The coliseum module (pack 1371) loads region 1372 + t for arena round t + 1 (SPEC-0041): round 1 is the brigand ring (region 1372), **round 2 is the Dark Ixi ring (region 1373)**, round 3 the Drackonack ring, round 4 the skeleton ring, round 5 the Werelupe ring. Round 2 is selected when story flag 596 is set and flag 597 is clear. The round is won when all six characters of region 1373 are dead (the watcher thread tests the character range every tick) and lost when both heroes have died. Each of the six records has default state 8, emerge distance 40.0, selector 0, cue word 1 (hittable and drawn while hiding; no ambush cue), no alarm group and no drop (ordinal -1).

| Ring position (bearing from the centre point (0, 7.21, 0), radius about 26.4) | Fighter | Hit points |
|---|---|---|
| 0 degrees (25.8, 0) | Dark Ixi Spearman | 170 |
| 45 degrees (18.5, 19.3) | Dark Ixi Spearman | 170 |
| 90 degrees (0, 26.3) | Dark Ixi Archer | 200 |
| 180 degrees (-26.4, 0) | Dark Ixi Archer | 200 |
| 225 degrees (-18.3, -18.7) | Dark Ixi Spearman | 170 |
| 270 degrees (-0.1, -26.5) | Dark Ixi Archer | 200 |

Records are in the order 0 to 5 as listed; they face the centre.

**Start (the arena-start method of SPEC-0063).** The intro scene takes control of the six fighters (base state 33, so their think loops are not stepped), moves them to their places and releases them with the hero returned to stance (action state 1, F1 = 0.7), so the wake threshold is 40.0 x 0.7 = 28.0 for a standing hero, 30.0 slow walking, 60.0 walking, 70.0 running. Every fighter plays the hide pick (Spearman clip 31, Archer clip 0) until its own distance test passes. A hero at the centre of the ring is 26.4 from each fighter, below 28.0: all six pass on the same tick (call it tick a), request the emerge clip on a + 1 (77 ticks for both rigs), enter agro on a + 78, run the set-up on a + 79 and may choose their first attack on a + 80. A hero standing nearer one edge wakes only the fighters within 28.0 (the others wake when it moves). A hit on a hiding fighter wakes it at once with doubled damage.

**The fight at capacity 0 (starting gear).** All six fighters have weight 8 and enter the attack manager in agro in record order; the list is sorted by distance with the 5.0-unit hysteresis rule, and only the first entry (plus any entry whose predecessors weigh at most the capacity) may start a sequencer. The three Spearmen (approach 4) that are refused wait in line (walk to a stand-off point 10 units around the hero, face it); the three Archers (approach 3) are never refused but their weight counts, so while an Archer is nearest the hero's first-listed Spearmen cannot attack at all. In practice the Archers shoot throughout from 16 to 34 units (a lob every 112 plus 60 to 240 ticks each, so one shell is released roughly every 60 to 120 ticks), and the Spearmen lunge when they become entry 0, which happens as they close to 12 to 18 units and the Archers sit further out. With better gear (capacity 15 allows two weight-8 attackers, 39 allows five) more of the Spearmen attack together. Because a lunge from the ring (26.4 units) connects with probability (50 - 21.4) / 40 = 0.72, a hero who does not move after the wake takes 50 damage from the first Spearman to reach entry 0.

**Round 2 timeline (hero standing at the centre of the ring, capacity 0).** Tick a: all six wake. a + 78: agro. a + 80: first choices; each Archer lobs at 49 % per tick (mean wait 2 ticks), the first Spearman in the list lunges at 24 % per tick; the others queue. The first shells are released about a + 166 (a + 80 + 86, plus the chooser wait) and land about 75 ticks later (a + 241; Distances table), hitting the hero if it still stands there (4 damage and fire for 8 to 12 s each). A lunge that reaches the hero (charge distance 21.4 or more, 65 run steps) hits on its choice tick + 95 (c + 30 + 65; the C window opens on + 91). In 600 ticks of a motionless hero: 6 to 9 shells, 2 to 4 lunges; a moving hero takes mostly fire patches.

#### Open questions (round 2)

1. The hero's position when the intro releases it. Resolved (High for the place, Medium-High for the stance; proactive (open close-out)): the last hero move of the intro scene is to locator 14, the centre of the ring (0, 7.21, 0), and nothing moves him afterwards; the hero is released in stance, which is 26.4 units from every fighter, inside the stance wake distance of 28, so all six wake on the first think tick.
2. Whether the intro scene plays any animation on the fighters that changes their release pose. Resolved (Medium-High; proactive (open close-out)): the scene grabs the six fighters, moves them to ring points and loops an intro animation on each, then releases them; from the first think tick after the release the hide pick is requested every tick, so they stand in the hide pick as before.
3. The exact order in which the six enter the attack manager list on the same tick. Runtime-only (needs a runtime check): all six are 26.4 units from the hero, so the bind order of equal distances decides it. Default if unresolved (rewrite choice): record order.

### Batch open questions

1. State 9 gait copying: the follower adopts the followed character's gait number and plays the state-9 walk (id 11). Resolved (High): the follower stores the followed character's locomotion mode each tick (a ground follower always accepts it), so the three Cogham Village Spearmen use the villagers' fast-run mode while they are at least the stay distance away, not a walk of 5.0. They play the mode-2 run clip (animation 21), whose root travel is 0.3334 units per tick, 20.00 units per second; the villagers themselves run at 19.30 (SPEC-0038 Behavior 40). Copy the mode and move at the speed of that mode's clip; the state-9 walk id 11 applies only to mode 0.
2. Which character a state-9 follower tracks if two characters share an id. Resolved (High; proactive (open close-out)): the first match in the lookup order, which is the world's embedded block and then its slot regions in order (not necessarily the follower's own region).
3. Resolved: the look target for sentries with +0xC4 = 1 is a direction, not a point. It is the raw placed-facing vector (kept at the character's home-facing field, y lane included; SPEC-0038, the placed-heading rule), and the turn step blends the current facing toward that direction with the row's aim weight, so a Death Knight sentry (weight 1.0) faces along its placed facing at once and keeps it (High: the turn step takes a direction, as for every other aim call).
4. The ambush camera and sound cue of the Abandoned Mines Dark Ixi Archer (cue word 0). Resolved (High; proactive (open close-out)): the cue is the shared stinger 4 only, played when the group alarm wakes it (see the Swamp Tentacle, Open question 5); no camera is started.
5. Hurt-capsule geometry for the Archer. Resolved (Medium; proactive (open close-out)): thirteen capsules on bones 0, 3, 4, 7, 10, 13, 16, 20, 23, 28, 31, 37 and 41, with lengths 0.79, 0.53, 0.79, 0.68, 0.93, 0.63, 0.57, 0.63, 0.57, 0.29, 0.29, 0.32 and 0.32 and radii 0.38, 0.23, 0.39, 0.38, 0.38, 0.15, 0.15, 0.15, 0.15, 0.30, 0.30, 0.30 and 0.30 (model units). The Dark Ixi Archer and the Henchman row 157 use the same rig.

## Werelupe Burrows and Library Oddities

Four kinds are covered here. The Werelupe Basher (monster type 99 variant 0, row 240), the Werelupe Bone Thrower (type 99 variant 1, row 241) and the Werelupe Elite (type 99 variant 2, row 242) are the three variants of one rig. The Elite already has a full guide as the Round 4 arena fighter (SPEC-0063); this section only adds what changes when an Elite stands in the field. Evil Harris (type 70 variant 0, row 150) is a small melee kind of its own: three in Fyora's Library, six or seven in the Park Shrine. All four are class-17 common enemies and follow SPEC-0038 (think loop, pre-step, notice check, alarm groups, attack chooser, sequencer, counter choice, take-hit hook, death hook). The section ends with an overview of the Altador Coliseum (pack 1371) and the full guide to its fifth round, the Werelupe ring. Confidence: High for row values, clip data, attack arithmetic, the start states, the two-stage notice and the alarm path; Medium-High for the idle states (shared with the other batches); Medium for weapon contact geometry, the Park Shrine scripts and the Coliseum hero start position; Low for one Elite patrol record with no route. All times are ticks (60 per second); "s" is the tick on which the sequencer requests the first clip of an attack and "c" is the tick of the attack choice. Every clip of these kinds has speed byte 2 (0.25 keys per tick at play rate 1.0), so a one-shot clip of F keys passes its 0.99 gate 4(F - 1) + 1 ticks after the request, one tick earlier from F = 25 keys (SPEC-0005).

### Shared rules for the Werelupe rig (Basher, Bone Thrower, Elite)

#### What they have in common

- **Visible kinds.** Display in default is 1 on all three rows (and on Evil Harris). They are drawn and hittable in every state, they use the real state 2 (not state 21) when they notice the hero, and the setter's default case gives state 0, which the pre-step turns into the spawn record's default state on the same tick. There is no state 23 and no decloak clip. A hidden Werelupe (default 8) is drawn and can be hit while it hides, because every state-8 record on disc has cue word 1.
- **Rows differ in values, and in three table entries.** The Basher and Elite tables are identical except that the Basher also has the chat ids 111 to 114 and the selector-1 hide and emerge ids (128, 129, 133, 134). The Bone Thrower table differs in the fidgets (ids 3 and 6 both play clip 16), the circle id 5 (clip 0), the id 32 back-off (clip 14) and its slot 2 (a throw clip instead of the three-hit swing); it has no charge clips. Clip ids and roles are in the table of each kind.
- **Weights in the attack manager** (SPEC-0038 Behavior 45): Basher 8, Bone Thrower 8, Elite 12, Evil Harris 5. Refused enemies wait in line unless their approach routine is 3, so the Basher (rush, approach 1) and the Elite (strafe, approach 4) wait, and the Bone Thrower (keep distance, approach 3) never does. The wait-in-line stand-off radius (row +0xB0): Basher 10, Elite 10, Harris 10, Bone Thrower 50 (unused).
- **Target type T (alignment).** 1 on all three Werelupe rows; 3 on Evil Harris. It is the element of every projectile the enemy fires (only the Bone Thrower fires one).
- **Socialize.** All three Werelupe rows have the socialize flag set (row +0x40 = 1) with pair radius 20.0 (row +0x44). A pair needs a separation of at most the sum of both radii (40.0), the same navmesh, the peer's quiet countdown below 1 and the peer not in states 1, 2, 3, 19 or 21. In states 5, 6 and 7 the quiet countdown runs: float[row +0x58, row +0x54) = [420, 480) for the Basher and Elite and [360, 480) for the Bone Thrower; it is 0 at creation, so the first scan runs on the first idle tick. Both enemies then go to state 3 (walk toward each other, stop within 6.0, turn, chat for float[120, 180) ticks) and return to the default state. While the talk countdown is positive the notice band is halved. The chat clip is id 14 on a draw of 0 and ids 111 to 114 on draws 1 to 4. The Basher has all five (id 14 = clip 15, 30 keys, loops; 111 = clip 16 and 112 = clip 17, 30 keys, one-shot, gate 116; 113 = clip 22, 16 keys, gate 61; 114 = clip 40, 40 keys, gate 156); the Bone Thrower and Elite have only id 14, so four draws in five play the idle pick instead. Evil Harris does not socialize (flag 0).
- **Map search.** Flag 0 on all four rows. No steal, no drain, no status on any slot.
- **Hurt shapes** (body capsules of the model, SPEC-0040 Behavior 12A; Medium): Werelupe bone 0 length about 1.91 to 2.02 with radius 0.69 to 0.71, bone 7 length 0.76 to 0.87 with radius 0.65 to 0.68. Evil Harris: bone 0 length 0.57 radius 0.38, bone 5 length 0.69 radius 0.43, limb capsules of length 0.18 to 0.39 and radius 0.13 to 0.26. Row radius and height (Werelupe 1.175 and 3.5; Harris 1.0 and 2.0) only feed the chase stop and spacing.
- **Death.** Death hook, base state 16, animation 58 once. Werelupe: clip 3 (20 keys, gate 77, progress 0.95 at tick 75); the corpse is removed 300 ticks (row +0x80) after the clip reaches progress 0.95 and sinks 1/180 unit per tick once the countdown is 30 ticks old, so it is gone about 375 ticks after death. Harris: clip 17 (12 keys, gate 45, progress 0.95 at tick 44) and a removal delay of 4 ticks, so about 48 ticks. Killed enemies stay dead (SPEC-0038 Behavior 47). Drops: the spawn record's drop ordinal names a placement of the region (SPEC-0038 Behavior 34); -1 (nothing) on 10 of 48 Bashers (the 8 Lower Burrows sentries and the 2 Coliseum records), 2 of 20 Bone Throwers (Coliseum), 3 of 14 Elites (Coliseum 2, the Meridell arena record) and all 10 Harris.
- **Hit reaction.** Can fly 0 and the flying-hit word 0 on all four rows, so every hit that is not blocked starts hit-reaction kind 0 (animation 57, outer 10). Werelupe: clip 4 (9 keys, gate 33, travels back 5.78 units per second, about 3.5 units in all). Harris: clip 23 (30 keys, gate 116, travels back 7.54 units per second, about 15 units over the clip). The reaction ends at the clip gate (outer 0, then the approach routine again). A hit while not in agro doubles the damage, enters agro at once through the setter and skips whatever state the enemy was in (SPEC-0038 Behavior 37). A hit during an attack abandons the attack with no end path.
- **Blocking Angle** (row +0x10C): 0.0 on the Basher and Bone Thrower, 0.9239 on the Elite, 0.0 on Harris (Harris never blocks). Row defence word (+0x114) 0.0 on all.

#### Where they stand: default states and counts

Placements in play (reached packs and confirmed or plausible slot loads; the unreached test block 1259 and the copy of the Glade script's region in pack 970 are excluded), by the default-state word of the spawn record. 0 idle until hit or alarmed, 5 wander, 6 patrol, 7 sentry, 8 hide and emerge.

| Kind (row) | Total | 0 | 5 wander | 6 patrol | 7 sentry | 8 hidden | Where |
|---|---|---|---|---|---|---|---|
| Werelupe Basher (240) | 48 | 0 | 18 | 7 | 12 | 11 | Illusen's Glade 15 (9 wanderers, 2 patrols, 4 hidden), Burrows Middle 10 (5 wanderers, 1 patrol, 1 sentry, 3 hidden), Burrows Lower 8 (all sentries), Werelupe Woods 7 (2 wanderers, 3 patrols, 1 sentry, 1 hidden), Burrows Upper 6 (2 wanderers, 1 patrol, 2 sentries, 1 hidden), Coliseum 2 (hidden) |
| Werelupe Bone Thrower (241) | 20 | 0 | 6 | 1 | 9 | 4 | Illusen's Glade 5 (4 wanderers, 1 patrol), Burrows Upper 5 (4 sentries, 1 hidden), Burrows Middle 4 (2 wanderers, 2 sentries), Burrows Lower 2 (sentries), Werelupe Woods 2 (1 sentry, 1 hidden), Coliseum 2 (hidden) |
| Werelupe Elite (242) | 14 | 0 | 0 | 1 | 0 | 13 | Burrows Lower 11 (10 hidden, 1 patrol), Coliseum 2 (hidden), Meridell arena 1 (hidden; SPEC-0063 Round 4) |
| Evil Harris (150) | 10 | 1 | 9 | 0 | 0 | 0 | Park Shrine 7 (6 wanderers over the whole mesh, 1 idle), Fyora's Library 3 (wanderers) |

The Glade records are the character set of the Glade before the Gnarfas fight (set 7 of the Glade script, loaded when story flags 248 and 64 are both clear; SPEC-0043 module 956). The Burrows are packs 1244 (Upper), 1245 (Middle) and 1246 (Lower; live-enemy cap 12 instead of 6, SPEC-0006) and Werelupe Woods is pack 1250; all four hold their characters as embedded blocks, streamed by proximity. Alarm groups (spawn byte +0xB7, 255 = none): no group in the Glade, the Lower Burrows Bashers, the Woods and the Coliseum. Burrows Upper group 1: two Basher sentries. Burrows Middle group 1: one Basher patrol and one hidden Basher; group 2: two hidden Bashers, one sentry Basher, one wandering Basher and one sentry Bone Thrower. Burrows Lower group 1: three hidden Elites; group 2: two hidden Elites; group 3: one hidden Elite and the patrolling Elite.

#### Spawn record values that drive the field states

Same words as the other batches (SPEC-0006 kind 3, SPEC-0038 Layout): default word at +0x84; state 5: float +0xBC (square side R; 0 means a random triangle of the whole navmesh), integers +0xC0 and +0xC4 (pause range) and +0xC8 (pause percent); state 6: s16 +0xB8 (path index), +0xBC (0 loop, other ping-pong), +0xC4 and +0xC8 (pause range at each point), +0xCC (percent), +0xD0 and +0xD4 (random-stop range); state 7: integer +0xC4 (0 or 1); state 8: float +0xC0 (emerge distance), +0xC4 (cue word), +0xC8 (selector).

| Kind | State 5 (R; pause range; percent) | State 6 (loop or ping-pong; pause at points; random stop) | State 7 (+0xC4 = 0 / 1) | State 8 (emerge distance; cue; selector) |
|---|---|---|---|---|
| Basher | Glade R 50 (8) and 20 (1), no pause; Burrows Upper R 50, no pause (2); Burrows Middle R 200, pause 0 to 60 at 5 percent (2) or 1 percent (3); Woods R 500, pause 0 to 120 at 1 percent (2) | Glade: path 1 loop, path 2 ping-pong, no pause; Upper: path 1 ping-pong; Middle: path 0 ping-pong, pause 0 to 120, 1 percent stop of length 0 (does nothing); Woods: paths 0, 1 and 3 ping-pong, pause 30 to 120, 25 percent stop of length 0 | Upper 2 / 0, Middle 0 / 1, Lower 0 / 8, Woods 0 / 1 | Glade 20.0, cue 1, selector 1 (4); Upper 6.0 (1); Middle 5.0 (3); Woods 12.0 (1); Coliseum 40.0 (2); all cue 1 |
| Bone Thrower | Glade R 50 (4), Middle R 80 (2), no pause | Glade: path 0 ping-pong, no pause | Upper 4 / 0, Middle 0 / 2, Lower 2 / 0, Woods 1 / 0 | Upper 6.0 (1); Woods 18.0 (1); Coliseum 40.0 (2); all cue 1, selector 0 |
| Elite | none | Burrows Lower: path -1 (no route; see Elite section), ping-pong, pause 240 to 480 at points | none | Lower 12.0 (10); Meridell arena 40.0; Coliseum 40.0 (2); all cue 1, selector 0 |
| Harris | Library R 100, Shrine R 0; no pause on any | none | none | none |

#### The idle states, per start state

In every idle state the pre-step runs first each tick: awareness countdown, notice check, socialize scan, and for state 8 the wake test; the state body follows (SPEC-0038 Behavior 2 and 5).

- **Default 0 (idle).** Stays in state 0 and plays the idle pick (base animation 0: Werelupe clip 5, 22 keys, loops; Harris clip 34, 60 keys, loops). No fidget variants exist in these tables (ids 115 to 119 are absent on the Werelupes; Harris has all five but is idle only once). No notice check. It leaves only by a hit (agro, damage doubled) or by its alarm group (a Harris of the Park Shrine has none).
- **Default 5 (wander).** Goal square of side R about the home point (R below 0.1: centroid of a random triangle of the navmesh), walk id 2 (Werelupe clip 0, 11 keys, loops, 7.56 units per second = 0.126 per tick; Harris clip 5, 10 keys, loops, 7.50 per second = 0.125 per tick), pause rolls per the record (a pause plays the idle pick; fidget id 3 once when the pause fidget timer expires: Basher and Elite clip 17, Bone Thrower clip 16, 30 keys, gate 116). With the pause percent 0 (all Glade, Upper, Library and Shrine wanderers) the enemy never pauses and picks a new goal the tick it arrives.
- **Default 6 (patrol).** Locator route from the s16 at +0xB8, walk id 4 (same clip as id 2), pause at each point int[+0xC4, +0xC8] (fidget id 6: Basher and Elite clip 20, 38 keys, gate 148; Bone Thrower clip 16, 30 keys, gate 116), random stop on a roll below +0xCC on the one tick per awareness cycle when the countdown is 0. A route index of -1 installs no route (one Elite; see below).
- **Default 7 (sentry).** Walks home when more than 0.5 away; at home with +0xC4 = 0 it stands id 7 (clip 19, 22 keys, loops) for int[60, 600] ticks then plays the look clip id 8 once (clip 20, 38 keys, gate 148) and repeats; the first thing a fresh sentry does is the look, because its look-in-progress word is -1 at creation. With +0xC4 = 1 it turns to the home facing every tick (blend = the row's aim weight: 0.9 on the Basher and Bone Thrower, 1.0 on the Elite) and plays id 7.
- **Default 8 (hidden).** Stands at home on the hide pick (selector 0: id 9, clip 32, 22 keys, loops; selector 1: id 128, clip 19, the same standing clip as id 7) and wakes when the hero is closer than the emerge distance times F1 of the hero's action state (0.7 standing, 0.75 slow walk, 1.5 walking, 1.75 running, 1.0 other). The next tick requests the emerge clip and the enemy enters agro on the gate tick. Selector 0: id 10, clip 33 (27 keys), gate 104. Selector 1 (the four Glade Bashers): id 133, clip 23 (the 13-key alert clip), gate 49. If the table lacks the clip the enemy enters agro at once (not the case here). Timeline from the wake test tick a: request a + 1, agro entry a + 1 + gate (a + 105 or a + 50), set-up the tick after, first attack choice the tick after that. The emerge is skipped by a hit (damage doubled, agro at once). Cue word 1 on every record: no camera or sound cue, and the enemy is drawn and hittable while it hides. Emerge distances: 40.0 means 28 for a standing hero, 30 slow walking, 60 walking, 70 running; 20.0 means 14, 15, 30, 35; 12.0 means 8.4, 9, 18, 21; 6.0 means 4.2, 4.5, 9, 10.5; 5.0 means 3.5, 3.75, 7.5, 8.75.
- **Returning to the default state.** Every exit from agro (leash, lost-sight search ending, victory pose) runs the setter with 0, giving state 0, which becomes the default state on the next tick: 5 resumes wandering from where it stands, 6 resumes the patrol, 7 walks home and stands, 8 walks home and hides (the whole wake and emerge repeats), 0 stands. The same applies after the search of state 2 gives up.

#### Noticing the hero and the alarm groups

States 3, 5, 6, 7 and 2 run the notice check (SPEC-0038 Behavior 3). The awareness countdown reloads with the row's timer and counts down once per tick, so a cycle is the timer plus 2 ticks and the check can pass only on the one tick per cycle on which the countdown is 0. On that tick roll A int[1, 100] must be at most Awareness; then the band (range 1 to the row maximum, both scaled by F1) decides: nearer than the scaled near edge is noticed with no further roll, farther than the scaled far edge is never noticed, in between roll B int[1, 100] must be below the chance interpolated from the near chance at range 1 to the far chance at the row maximum (unscaled), halved when the hero is outside the view cone and halved again by the Tattered Cloak. A blocked sight line replaces F1 by F2. Values:

| Row | Awareness | Timer (cycle) | View cosine | Range | Chance near / far | Search countdown | After a pass |
|---|---|---|---|---|---|---|---|
| 240 Basher | 75 | 30 (32) | 0.9239 | 1 to 100 | 70 / 10 | int[60, 240) | alert clip 49 ticks |
| 241 Bone Thrower | 80 | 60 (62) | 0.9239 | 1 to 100 | 80 / 30 | int[60, 240) | alert clip 49 ticks |
| 242 Elite | 100 | 30 (32) | 0.707 | 1 to 100 | 90 / 30 | int[120, 240) | alert clip 49 ticks |
| 150 Evil Harris | 100 | 30 (32) | 0.9239 | 1 to 30 | 75 / 5 | int[30, 60) | alert clip 49 ticks |

Probability that one cycle passes, hero standing (F1 0.7), sight line clear, hero inside the view cone / outside it:

| Row | 3 units | 6 | 10 | 20 | 30 | 45 | 60 | 70 |
|---|---|---|---|---|---|---|---|---|
| 240 | 0.51 / 0.26 | 0.49 / 0.25 | 0.48 / 0.24 | 0.43 / 0.22 | 0.39 / 0.20 | 0.32 / 0.16 | 0.26 / 0.13 | 0.21 / 0.10 |
| 241 | 0.62 / 0.31 | 0.62 / 0.30 | 0.60 / 0.30 | 0.56 / 0.28 | 0.52 / 0.26 | 0.46 / 0.22 | 0.40 / 0.20 | 0.36 / 0.18 |
| 242 | 0.88 / 0.44 | 0.86 / 0.43 | 0.84 / 0.42 | 0.78 / 0.39 | 0.72 / 0.36 | 0.63 / 0.31 | 0.54 / 0.27 | 0.48 / 0.24 |
| 150 | 0.70 / 0.35 | 0.62 / 0.31 | 0.53 / 0.26 | 0.29 / 0.14 | 0 | 0 | 0 | 0 |

A standing hero closer than 0.7 units is noticed whenever roll A passes. A walking hero (F1 1.5) is noticed out to 150 units by the Werelupe rows (the far edge is 100 x F1) and a running one to 175; the chance values do not scale, so at 100 units a walking hero is noticed with probability 0.07 (Basher), 0.23 (Bone Thrower), 0.29 (Elite) per cycle in the cone. Evil Harris notices a standing hero out to 21 units, a walking one to 45 and a running one to 52.5 (range 30 x F1), and a walking hero at 30 units with probability 0.04 per cycle.

**Stage 1: state 2 (alerted search).** A pass in states 3, 5, 6 or 7 calls the setter with 2. The setter draws the search countdown trunc(float[60, 240)) (Elite [120, 240), Harris [30, 60)), stores the hero's present position as the last-seen point and sets state 2. Per tick in state 2, in this order:

1. If neither the alert clip (id 18) nor the search-end clip (id 17) is current, request the search idle id 16 (Werelupe clip 21, 40 keys, one-shot, gate 156; Harris clip 31, 45 keys, gate 176).
2. Aim at the last-seen point with the row's aim weight; the enemy does not walk.
3. Run the notice check again on the same tick (a new pair of rolls; the awareness countdown is still 0 on the entry tick). A pass while id 18 is not current requests id 18 (clip 23, 13 keys, gate 49 on all four rows) and the enemy enters agro when that clip passes its gate; the alert clip then plays to its end even if later checks fail.
4. With no pass and id 18 not current, the search countdown goes down by 1 every tick. At 0 or below the enemy requests id 17 (Werelupe clip 22, 16 keys, gate 61; Harris clip 26, 30 keys, gate 116) and, when it has finished, runs the setter with 0 (default state). The notice check keeps running during clip 17 and a pass then still goes to alert and agro.

So an enemy that notices the hero on cycle tick t0 has a second roll on the same tick and, if that fails, one chance every cycle (32 or 62 ticks) for the search countdown plus the end clip; if none passes it gives up after 121 to 300 ticks (Basher, Bone Thrower), 181 to 300 (Elite) or 91 to 120 (Harris). A pass is followed by 49 ticks of the alert pose before the first agro tick.

**Alarm groups** (SPEC-0038 Behavior 3). The first group member to enter agro broadcasts. Every other member in state 0, 3, 5, 6, 7 or 14 goes, at its next alarm check (int[15, 45] ticks apart), to state 2 and requests id 18 directly; it stands in the 49-tick alert pose and then enters agro with no notice roll, wherever the hero is. A member in state 8 plays emerge id 10 (clip 33, 104 ticks) and enters agro on the gate; the alarm check always requests id 10, even for a selector-1 record (none of the selector-1 Bashers has a group), and no ambush cue plays (cue word 1). Members in any other state (agro, 2, 19) do nothing.

#### Losing the hero

Agro exits to the default state by the leash (no route to the hero for 181 consecutive ticks while the walk waypoint is 15.0 or more from the hero), the lost-sight search timing out (outer 9, counter above 120) and the victory pose after a killing blow (animation 73, Werelupe clip 40, 40 keys, gate 156; Harris has no clip 73, so the setter drops to state 0 at once). There is no distance give-up: an enemy with a route follows the hero indefinitely.

#### Hero actions against the shared model

| Hero action | Result |
|---|---|
| Stands still out of range of a wanderer, patroller or sentry | Nothing; the check needs the scaled range (standing hero: 70 units for the Werelupes, 21 for Harris) |
| Walks or runs through the area | Notice ranges grow by 1.5 or 1.75; a member that notices triggers the alert pose of the whole group (49 ticks) |
| Hits a sentry, wanderer, patroller, idle or hidden enemy | Damage doubled, agro at once, hit reaction kind 0, no search stage |
| Hits an enemy during its search (state 2) | The same |
| Wears the Tattered Cloak | Noticed at half chance and never through a blocked sight line |
| Fights one member of a group | Other members in states 0, 3, 5, 6, 7, 8 and 14 of that group commit within 15 to 45 ticks plus 49 (alert) or 104 (emerge) ticks |
| Kills a squad | Nothing special; Werelupe corpses sink and vanish about 375 ticks after death |

### Werelupe Basher (row 240)

The Basher is the rush brawler of the Werelupe rig: a fast melee fighter that closes to 3.5 to 5 units of the hero at 23.6 units per second, jabs for 25, and from 20 to 50 units away runs a wound-up charge for 15 with a knock-down. It has a third, three-hit unblockable swing in its data that can never be chosen.

#### Identity and tunables

| Item | Value |
|---|---|
| Record, name on the bar | monster type 99 variant 0, "Werelupe Basher", character key 2320 |
| Hit points | 75 |
| Body | radius 1.175, height 3.5 |
| Class, approach routine | class 17, approach 1 (rush) |
| Attack-manager weight, target type T, hostile | 8, 1, yes |
| Rush band (row +0x4DC, +0x4E0) | 3.5 to 5.0. Desired distance 3.5 + 1.5 f, f = int[20, 80] / 100: 3.8 to 4.7, redrawn after every normal choice. Hold between 3.5 and 5.0 |
| Aim weight in hold states (row +0x84) | 0.9 |
| Recovery after an attack | int[15, 30] |
| Notice | awareness 75, cycle 32, cone 0.9239, range 1 to 100, chance 70 near / 10 far (shared table) |
| Combat timer drawn at agro entry | float[240, 480) (no rule reads it) |
| Missile defence: look-ahead and hold / dodge / block | 60 / 50 / 0 |
| Melee block chance quick, normal, heavy | 50, 15, 5 |
| Melee dodge chance quick, normal, heavy | 0, 0, 0 (never) |
| Defence hold after a defence | int[120, 240] |
| Blocking Angle | 0.0 |
| Weapon volumes | all five: radius 2.0, height 2.0 |
| Counter triggers | slot 0: code 2 (after a dodge), chance 25; slot 2: code 2, chance 25; slot 1 none |
| Status, steal, drain | none |
| Wait-in-line stand-off radius | 10 |
| Death | 300-tick corpse timer; see shared rules |

Attack slots (k = table slot; "per tick" is the chance that one roll int[1, 100] is below int(chance)):

| Slot | Type | Band (strict, min < d < max) | Chance, per tick | Facing cosine above | Range | Cooldown draw | Charge | Damage, knockback | Speeds (wind-up, strike, recovery) | Flags |
|---|---|---|---|---|---|---|---|---|---|---|
| 0 | melee jab | 0 to 12 | 50, 49 % | 0.9239 | 5.0 | int[30, 60] | none | 25, 0 | 1.0, 1.0, 1.0 | blockable |
| 1 | melee charge | 20 to 50 | 50, 49 % | 0.966 | 5.0 | int[30, 120] | kind 2, distance float[10, 50) | 15, 2.0 | 1.0, 1.0, 1.0 | blockable |
| 2 | melee, three windows | 20 to 5 (empty: min above max) | 50 | 0.9239 | 5.0 | int[30, 120] | none | 25, 0 | 1.0, 1.0, 1.0 | unblockable (flag 0x1) |

Slot 2 can never be chosen: its trigger band has the minimum (20) above the maximum (5), so no distance satisfies min < d < max, and its counter code (after a dodge) needs a dodge the Basher cannot make in melee. The other two slots are tested in the order 2, 1, 0.

#### Animation ids and clips (speed byte 2 throughout)

| Role | Id | Clip, keys | Notes |
|---|---|---|---|
| Idle base | 0 | clip 5, 22 keys, loops | |
| Walk, patrol, wait-in-line walk | 2, 4, 11 | clip 0, 11 keys, loops | 7.56 units per second (0.126 per tick) |
| Rush approach (gaits 0, 1, 2) | 21 | clip 9, 8 keys, loops | 23.58 units per second (0.393 per tick) |
| Rush hold | 22 | clip 24, 18 keys, loops | |
| Back off (gait 3) | 23 | clip 11, 8 keys, loops | 23.58 units per second backward |
| Sentry stand / look | 7 / 8 | clip 19 (22 keys, loops) / clip 20 (38 keys, gate 148) | |
| Rest fidget / patrol fidget | 3 / 6 | clip 17 (30 keys, gate 116) / clip 20 | |
| Chat | 14, 111 to 114 | clips 15, 16, 17, 22, 40 | see shared rules |
| Search idle / search end / alert | 16 / 17 / 18 | clip 21 (40 keys, gate 156) / clip 22 (16 keys, gate 61) / clip 23 (13 keys, gate 49) | |
| Hide pick / emerge | 9 / 10; selector 1: 128 / 133 | clip 32 (22 keys, loops) / clip 33 (27 keys, gate 104); clip 19 (22 keys, loops) / clip 23 (gate 49) | |
| Missile dodge | 53 or 54 (int[0, 1]) | clip 29, 13 keys, gate 49 | moves about 6.0 units sideways (6.91 per second) |
| Block / block-hit flash | 56 / 126 | clip 10 (15 keys, loops) / clip 42 (9 keys, gate 33) | |
| Hit reaction | 57 | clip 4, 9 keys, gate 33 | back 5.78 per second |
| Death / victory pose | 58 / 73 | clip 3 (20 keys, gate 77) / clip 40 (40 keys, gate 156) | |
| Slot 0 | B 34, D 36 | clip 8 (15 keys, gate 57), clip 26 (17 keys, gate 65) | A, C, E skipped |
| Slot 1 | A 38, B 39, C 40, D 41 | clip 27 (10 keys, gate 37), clip 41 (8 keys, loops, 28.14 per second), clip 28 (7 keys, gate 25), clip 26 | E skipped |
| Slot 2 | B 44, D 46 | clip 7 (28 keys, gate 108), clip 26 | never played |

The melee dodge id 55 (clip 11, the back-off clip) exists but is never requested, because the dodge chance is 0.

#### Wake and start

Field starts, notice, search, alarm and the hidden wake are in the shared rules. The Basher's own numbers: a Basher in state 5, 6 or 7 notices a standing hero at 6 units with probability 0.49 per 32-tick cycle in the cone; a hit or an agro entry puts it in agro with outer sub-state 0 on the first agro tick (idle pick, outer = 1, route flag 1), outer 1 inner 1 (hold) from the second tick, and the first attack choice on the second tick as well (the choice runs before the outer switch, with outer 1 already stored). The Glade ambushers (selector 1) stand in a plain standing pose until the hero comes within 14 units (standing) and then play the 13-key alert clip as their emerge (49 ticks); the Burrows and Woods ambushers use the 27-key emerge (104 ticks).

#### State table (agro)

| State | Id and clip | Gate / exits |
|---|---|---|
| Set-up (outer 0) | idle pick id 0 (clip 5) | one tick, then outer 1 inner 1 |
| Rush hold (inner 1) | 22, clip 24 | no gate. Faces the hero (weight 0.9). d < 3.5 gives back off; d > 5.0 gives approach |
| Approach (inner 0) | 21, clip 9 | gait 2 (same clip) when d > 10.0, else gait 0; ends when d < the drawn desired distance (3.8 to 4.7): inner 1. With no route: straight steer to the hero |
| Back off (inner 2) | 23, clip 11 | steers to the point hero - dir x 7.0; ends when d > desired |
| Slot 0 jab (outer 2) | B 34, clip 8 | weapon window s + 20 to s + 31; one hit at s + 20; gate s + 57; end path s + 58 |
| Slot 1 charge (outer 2) | A 38 (clip 27), B 39 (clip 41), C 40 (clip 28) | wind-up gate c + 37; run from c + 38; C window 4 to 15 ticks after its request; C gate 25 |
| Melee block (outer 7) | 56 loops, 126 once | until the hero's swing ends |
| Missile dodge (outer 6) | 53 or 54 | 49 ticks, unhittable, then the defence exit |
| Hit reaction (outer 10) | 57 | 33 ticks, then outer 0 |
| Wait in line (outer 12) | walk id 2, then 73 once at the stand-off point, then a countdown int[600, 800] | until the manager releases it |
| Victory pose | 73 | 156 ticks, then the default state |
| Dead (base 16) | 58 | 77 ticks, corpse removed 300 ticks after the 0.95 point |

#### Attack chooser

Per SPEC-0038 Behavior 7: runs every agro tick in outer 1, 3, 4 and 11 while the recovery countdown is below 1 and the hero is attackable; cooldowns count down only while the hero is attackable. Slots are tested 2 (dead), 1, 0. Slot 1 needs a route to the hero (a charge slot always does) and the facing cosine above 0.966; slot 0 passes with a route or with d below 10.0. Resulting behaviour with every cooldown spent:

- d above 50: no attack; the rush approach closes the distance.
- d from 20 to 50: charge at 49 % per tick.
- d from 12 to 20: no attack; the rush approach closes the distance.
- d below 12: jab at 49 % per tick.

On a choice the slot's cooldown is drawn (int[30, 60] jab, int[30, 120] charge), the desired distance is redrawn and, after a charge choice, the charge distance L = float[10, 50). The cooldown is drawn again at the end path. After the end path the recovery countdown (15 to 30) must run out before the next choice; a hit reaction abandons an attack with no end path, so only the cooldown drawn at the choice limits the next choice. A hero who stands still more than 20 units away is charged within about two ticks of the Basher having line and facing.

**Jab (slot 0).** Choice tick c, sequencer state 0 on c. With d above 5.0 the Basher walks forward with gait 0 (id 21, 0.393 per tick) until d is at most 5.0, then state 1 on the next tick (s). On s: A is missing, B 34 is requested and state 2 begins on the same tick. Weapon 1 (radius 2.0, height 2.0) is open on ticks s + 20 to s + 31 (frames 5 to 7); one hit, damage 25, knockback 0, at s + 20 (first contact tick). The clip passes its gate on s + 57 and the end path runs on s + 58, hit or miss. The clip carries 4.33 units per second of forward travel (about 4.3 units over the swing). A blocked strike (the hero guarding in front of it) plays D (17 keys, gate 65). Since the Basher holds at 3.8 to 4.7, which is inside the 5.0 range, a jab usually starts at s = c + 1.

**Charge (slot 1, charge kind 2).** Choice tick c: state 6 aims at the hero, requests A 38 (10 keys, forward travel 1.38 per second, about 0.9 units in all) and the wind-up passes its gate on c + 37; in that tick B 39 (the 28.14-units-per-second run clip, 0.469 per tick) is requested and state 7 runs from c + 38. Each tick of state 7: the tick counter goes up, t is the distance travelled from the start point; when the hero is within the slot range 5.0, state 8 follows on the next tick; otherwise when t has reached the drawn charge distance L, state 12. The run is a straight line along the facing set at c (no steering and no strike test). State 8 requests C 40 on its first tick (c8); the strike test runs every tick; weapon window 4 to 15 ticks after the C request (frames 1 to 3 of the 7-key clip); one hit, damage 15, knockback 2.0 (the hero goes into the long-fall state 48), at c8 + 4; the C clip carries 6.54 units per second of forward travel; the gate passes on c8 + 25, state 12 follows, E is missing, and the end path runs in the tick after the gate. If the run uses up L before reaching 5.0 the Basher stops where it is, state 12 and the end path follow at once and the charge whiffs: the whole cycle (37 + L / 0.469 ticks) is paid for nothing. Reaching a standing hero at distance d0 needs L at least d0 - 5, so the charge connects with probability (55 - d0) / 40 for d0 from 20 to 50 (d0 = 25: 0.75; 30: 0.625; 40: 0.375; 50: 0.125). The reference hit time for a standing hero is c + 43 + k with k = ceil((d0 - 5 - 0.85) / 0.469) run steps: d0 = 26.4: k = 44, hit at c + 87.

#### Movement

Rush archetype (SPEC-0038 Behavior 44, outer 1): the Basher approaches at 23.58 units per second (0.393 per tick, three times the walking pace of its own wander), stops at 3.8 to 4.7 units, holds facing the hero at weight 0.9 and backs off at the same speed only when the hero comes within 3.5. It never circles or strafes. With a hero walking away it re-enters the approach the moment d passes 5.0. With no route to the hero (blocked mesh) it steers straight at it and the leash counter of the shared rules counts. A chasing Basher passes the notice stage only once; in agro it follows until the leash or a lost-sight search.

Movement table (movement pass; the Werelupe rig, rows 240, 241 and 242 share these clips; speed byte 2):

| Mode | Source | u/t | u/s | Turn | Clip (id: clip, keys, loop) | Exits |
|---|---|---|---|---|---|---|
| Wander, patrol, walk home, wait in line | root | 0.1259 average; per key 0.105 to 0.146 | 7.56 | steering 0.3 / 0.1 | 2, 4: clip 0, 11, loop | goal or point reached |
| Rush approach (inner 0, gaits 0 and 2) and sequencer approach | root | 0.393 on every key | 23.58 | steering | 21: clip 9, 8, loop | d below desired (3.5 + 1.5f, 3.8 to 4.7) or the slot range |
| Rush hold (inner 1) | none | 0 | 0 | aim 0.9 | 22 | d below 3.5 or above 5.0 |
| Back off (inner 2) and melee dodge 55 | root backward | 0.393 | 23.58 | faces the hero | 23, 55: clip 11, 8, loop | d above desired; point 7.0 from the hero (the dodge: the looping clip's 0.99 gate, about 31 ticks, about 12 units) |
| Charge run (slot 1, state 7) | root | 0.469 | 28.14 | none; solid | 39: clip 41, 8, loop | horizontal d at most 5.0, or travelled 10 to 50 |
| Missile dodges 53, 54 | root sideways | 5.99 units over the clip's 52 ticks (gate 49) | 6.91 | none | clip 29 (54 is its mirror) | gate |
| Hit reaction kind 0 | root backward | 3.47 units over the clip's 36 ticks (gate 33) | 5.78 | snapped to the attacker | 57: clip 4, 9 | gate |
| Death | none | 0 | 0 | none | 58: clip 3 | removal |

Worked example (chase from 30, no attack passing). Agro tick 1: outer 0. Tick 2: rush entry, hold. Tick 3: d above 5.0 gives inner 0 and the rush clip is requested; the first step lands on tick 4. With desired 4.25 it needs 25.75 units: 66 steps of 0.393, so its think sees d = 4.06 (below desired) on tick 70, holds, and takes one more step on that tick to about 3.67. In play the charge (slot 1, band 20 to 50) and the club (slot 0, below 12) usually start first.

What a port most likely gets wrong (movement): a walk-speed chase (the rush is 23.58 u/s, faster than a running hero at 19.29); a slow back-off (the back gait is as fast as the rush); a melee dodge that only steps (the dodge plays the back-run clip, about 12 units; only a row with a melee dodge chance above 0 uses it, and the Basher's and Bone Thrower's are 0).

#### Hit and hurt windows

Weapon windows are given above (one hit per open window; the hit list is cleared at the attack start and at each separate open window). The Basher is hittable by the hero except in outer 5 and 6 (dodges; only outer 6 occurs), in base state 16 and when script-held. It is hittable while blocking (outer 7), but the hero's swing is judged by the guard rule: a guarding target never takes damage from a non-power swing (SPEC-0036 Hit detection, step 4). Damage to the hero: row value x 1.0, then shield and armour; a blocked strike (the hero guarding with the hero's facing dot the Basher's facing at most -0.5) still deals the reduced damage, plays D and ends the swing; knockback above 0 puts a living hero in state 48 (charge hit only).

#### Weapon reach

Conventions are those of "Weapon reach tables" in SPEC-0064 (shared field behaviour). Row 240, Medium. The tick text of the guide is authoritative for the hit ticks; "sC" is the tick the charge clip is requested. The Werelupe Elite (row 242) has exactly these rows (its own table is in SPEC-0063 Round 4); the Bone Thrower (row 241) has the first row only. Weapon radius and height are 2.0 by 2.0 in every row.

| Slot (phase, clip) | Weapon | Window ticks (level contact) | Centre forward / up / side | Tor ahead, at the tick | Tor ahead, from the start | Tor feet vs origin |
|---|---|---|---|---|---|---|
| 0 jab (B, clip 8) | w1 | s+20 to s+31 (s+20 to s+31) | 0.4 to 2.5 / 1.6 to 3.0 / -1.6 to 1.9 | -1.4 to 5.0 | 1.9 to 8.1 | -2.0 to 3.5 |
| 1 charge hit (C, clip 28) | w3 | sC+4 to sC+15 (sC+4 to sC+15) | 2.0 to 3.0 / 1.7 to 2.7 / -0.2 to 0.0 | 0.8 to 4.8 | 1.9 to 8.2 | -3.5 to 4.5 |
| 2 three-hit swing, first (B, clip 7) | w1 | s+28 to s+43 (s+28 to s+43) | 0.2 to 2.9 / 1.2 to 3.0 / -2.1 to 3.0 | -1.0 to 5.4 | 1.7 to 9.0 | -2.0 to 4.0 |
| 2 three-hit swing, second (B, clip 7) | w2 | s+52 to s+59 (s+52 to s+59) | 0.9 to 2.4 / 2.2 to 2.6 / -2.2 to 2.5 | 0.2 to 5.0 | 7.0 to 11.9 | -2.5 to 4.0 |
| 2 three-hit swing, third (B, clip 7) | w1 | s+76 to s+83 (s+76 to s+83) | 0.3 to 2.1 / 0.8 to 4.8 / -1.6 to 0.0 | -0.8 to 4.8 | 9.1 to 16.3 | -4.0 to 6.5 |

Every blow reaches 4.8 to 5.4 ahead of the Werelupe at the tick, which is close to the 5.0 start range, and 1.4 behind or less; the three-hit swing carries the Werelupe forward during its 84 ticks (the second and third blows land 7 to 16 units ahead of where it started).

#### Defence and counter rules

Melee defence, once per hero swing, on the first Basher tick that sees the swing: not in outer 10; not in outer 2 unless the sequencer is in state 0, 1 or 5; defence hold below 1; within 8.0 of the hero and inside the hero's 60-degree facing arc. r1, r2 = int[1, 100]; r1 at most the block chance gives outer 7 (block), else r2 below the dodge chance (0: never) would give outer 5. On a block the hold is set to 60 and, when the swing ends, redrawn int[120, 240].

| Swing class | Block | No reaction |
|---|---|---|
| Quick (Tor 5, 6; Roberta 5, 6, 7, 10) | 50 % | 50 % |
| Normal (Tor 2, 3, 9; Roberta 2, 3, 8, 11, 12) | 15 % | 85 % |
| Heavy (Tor 4, 7, 8, 12; Roberta 4) | 5 % | 95 % |

Tor's power step has no swing class and is never defended. While a Basher blocks, a non-power swing of the hero does nothing at all (no damage, no hit reaction); that includes swings from the side and behind, because the Blocking Angle is 0.0 and the swing is only "blocked" (block flash, no damage) when the dot of the two facings is below it. A swing while the Basher is winding up, running or striking (sequencer states 2 to 4, 6 to 8) is not defended. Missile defence (hero shots whose 60-tick look-ahead path meets the body): dodge if int[1, 100] is below 50 (49 %), block never; the dodge plays 53 or 54 for 49 ticks. Counters: slot 0 and slot 2 have counter code 2 (after a dodge, outer 5 or 6) with chance 25, and that code also needs the hero's recover window to be open (Tor's swing frames only). The only dodge the Basher can make is the missile dodge, which needs a live hero shot while the active hero is Tor in a swing; in practice the counter never fires. Slot 2's counter would not pass its band anyway.

#### Hero actions against this enemy

| Hero action | Result |
|---|---|
| Swing at it in the open, in front, within 8 units | blocked 50 % of quick swings, 15 % of normal swings, 5 % of heavy ones; otherwise lands |
| Same swing while it jabs, charges or winds up | lands |
| Swings during the 120 to 240 ticks after a block ended | every swing lands: the defence hold blocks new defences |
| Stands still 30 units away | charge within two ticks (49 % per tick), connecting with probability 0.625 and dealing 15 plus a knock-down; then jabs for 25 about every 90 to 120 ticks |
| Runs in circles | the charge run does not steer, so a sidestep during the 100-tick run wastes it |
| Roberta's shots | 49 % dodged |
| Guards (L1) facing it | blocks the jab and the charge, reduced damage; the charge's knockback still applies |
| Kills it | 75 hit points: 19 undefended starting-sword hits of 4 |

#### Edge cases for implementers

- **Slot 2 is dead data.** Do not give the Basher the Elite's three-hit swing. Its 25-damage unblockable flurry exists in the row but its band is empty.
- **Fast approach, stationary strikes.** The same rush speed (23.58 per second) applies to the sequencer's forward walk (gait 0) for a melee choice made from afar, so a jab chosen at 11 units walks the Basher in at 0.393 per tick before it swings.
- **Block timing.** The hold is 60 ticks at the start of a block and is redrawn to 120 to 240 only when the block ends (the defence exit), so the quiet period starts at the end of the hero's swing.
- **Recovery after the end path only.** A hit reaction abandons an attack; the next choice then waits only for the cooldown drawn at the choice.
- **Wait in line.** When the attack manager refuses a Basher (not first in the list with starting gear) it plays the wait-in-line state (outer 12, walks to a stand-off point within 5 units of the hero, faces it, plays the victory clip once, re-picks every 600 to 800 ticks) and starts its attack only when released. With capacity 0 a pack of Bashers fights one at a time.
- **Emerge by selector.** The four Glade Bashers read selector 1 and use ids 128 and 133 when they wake by distance; a port that always plays id 9 and 10 gives them the 104-tick emerge. (The alarm-group path always plays id 10; these four have no group.)
- **Alert pose.** An alarmed Basher plays the 13-key alert clip for 49 ticks with no weapon bit set.

#### What a port most likely gets wrong

1. Making it a strafer like the Elite. It rushes at 23.6 units per second and holds 3.5 to 5 units away; it has no circling and no band of 10 to 20.
2. Giving it the Elite's numbers: jab 25 (not 35), charge 15 with knockback 2.0 (not 35 and 4.0), recovery 15 to 30, block 50/15/5 with no dodge.
3. Letting the empty slot 2 fire.
4. Per-tick chances: 49 % (not 50 %) for both live slots.
5. Charging without the 37-tick wind-up, or letting the charge steer or hit during the run.
6. Treating hidden Bashers as invisible: all of them are drawn and hittable while they wait.
7. Forgetting the 49-tick alert pose before an alarmed or noticed Basher starts fighting.

#### Test vectors

**B1: Basher in agro, hero standing 30.0 units ahead (t = 0 setter tick), cooldowns 0.** t = 1 set-up; t = 2 first choice: slot 1 at 49 % per tick (30 is inside 20 to 50, facing above 0.966 assumed). Take c = 2. L = float[10, 50) is drawn at the choice. Tick 2: A 38 requested, state 6; wind-up gate on tick 39; B 39 requested on 39, state 7 from tick 40. The Basher has moved about 0.9 units in the wind-up, so it needs 24.1 more: k = 52 run steps (ticks 40 to 91) provided L is at least 25 (probability 0.625). Tick 92 sees d at most 5.0; tick 93 state 8 requests C (c8 = 93); the hit lands on tick 97 (15 damage, knockback 2.0, hero into state 48); the C gate on tick 118; state 12 and the end path on tick 119 (recovery 15 to 30; slot 1 cooldown int[30, 120] drawn again). With L below 25 the run stops after L / 0.469 ticks (21 to 53 steps), the end path follows and the Basher stands 5 to 15 units short; it then approaches at 0.393 per tick to 3.8 to 4.7 and, once d is below 12, jabs at 49 % per tick. After a connecting charge the Basher is within 5.0: the first jab choice is at tick 119 + max(recovery, 0) + geometric wait (mean 1 tick); B at s = c' + 1; hit at s + 20, end path s + 58, next jab choice after int[30, 60] (cooldown) and the recovery 15 to 30. Over 600 ticks with a motionless, unguarded hero: one charge (15) and 4 to 5 jabs (25 each) if every jab connects: 115 to 140 damage before armour.

**B2: hero standing 3.0 units from a Basher in agro.** t = 1 set-up; t = 2 first choice: slot 1 not eligible (d below 20); slot 0 at 49 %: c = 2 + G (G geometric, p = 0.49). d = 3.0 is below 5.0, so state 1 on c + 1: B requested s = c + 1; hit s + 20 = c + 21 (25); gate c + 58; end path c + 59. The rush hold sees d below 3.5 at t = 2 and starts the back off (inner 2, 0.393 per tick) in the same tick; the back off lasts until d passes the drawn desired distance, about 2 to 5 ticks, so the Basher stays within 5.0 of a motionless hero. Start-to-start spacing of jabs: 59 + int[30, 60] cooldown + geometric wait: 90 to 120 ticks.

**B3: Glade ambusher (state 8, emerge 20.0, selector 1), hero standing 10.0 units away.** Wake test tick a: 10 < 20 x 0.7 = 14, sub-state 1. a + 1: request id 133 (clip 23, 13 keys). a + 50: gate passes, setter 19 (agro entry; group broadcast: none). a + 51: set-up. a + 52: first choice (d = 10 is below 12: jab at 49 % per tick). The Burrows Middle ambusher (emerge 5.0) with the same hero stays hidden until the hero is within 3.5.

**B4: defence arithmetic.** Tor's ordinary swing (normal class) at a Basher 6 units ahead and in front, hold 0: r1 at most 15 gives outer 7 (15 %): the swing deals 0. Otherwise (85 %) the swing lands: starting sword damage 4 on an enemy in agro. After a block, the hold is int[120, 240].

#### Open questions

1. Weapon contact geometry: the jab window and the C window assume the hero is inside weapon 1's cylinder (radius 2.0, height 2.0 about an attachment frame). Resolved by the general rule (SPEC-0064, Shared field behaviour: Weapon contact and the hero's hurt shape); the distances are in the Weapon reach table; stand-in only for a port without attachment frames: the strike touches a level hero inside the "at the tick" range of the slot's row (1.4 behind to 5.0 ahead for the jab).
2. Whether the facing at the charge choice is exact. Resolved (High; proactive (open close-out)): effectively yes; state 6 aims at the hero every tick of the 37-tick wind-up with the row's aim turn weight (0.9 on row 240), so the remaining angle error is 0.1 to the power 37 of the starting error, and the run starts along the horizontal direction to the hero as he stands on the last wind-up tick.
3. Distance metric of the hold and band tests. Resolved (High; proactive (open close-out)): horizontal, with the height difference dropped (the shared distance helper).

### Werelupe Bone Thrower (row 241)

**Player's answer (R-0149).** Close in: it has a club slot out to 12 units, never dodges a swing, only blocks quick, normal and heavy swings on 50, 15 and 5 percent, and dodges a shot 49 times in 100; it keeps 10 to 50 units away but never gives up (SPEC-0038 Behavior 44).

The Bone Thrower is the ranged Werelupe: it keeps 10 to 50 units from the hero, hurls a bone for 25 from 10 to 75 units away, and clubs anyone who comes within 12 units for 25. It has no charge and no second melee slot.

#### Identity and tunables

| Item | Value |
|---|---|
| Record, name on the bar | monster type 99 variant 1, "Werelupe Bone Thrower", character key 2321 |
| Hit points | 85 |
| Body | radius 1.175, height 3.5 |
| Class, approach routine | class 17, approach 3 (keep distance) |
| Attack-manager weight, target type T, hostile | 8, 1, yes |
| Keep band (row +0x4E4, +0x4E8; retreat words 0) | 10.0 to 50.0. Desired distance 10 + 40 f = 18 to 42 (f = int[20, 80] / 100), redrawn after every normal choice. The retreat state (inner 6) never starts: its minimum is 0 |
| Aim weight in hold (row +0x84) | 0.9. In the hold the agro machine first aims at 0.25 and the class handler then aims at 0.9 in the same tick, so the effective blend is 0.925 |
| Recovery after an attack | int[30, 120] |
| Notice | awareness 80, cycle 62, cone 0.9239, range 1 to 100, chance 80 near / 30 far (shared table) |
| Combat timer | float[240, 480) (no rule reads it) |
| Missile defence: look-ahead and hold / dodge / block | 60 / 50 / 0 |
| Melee block chance quick, normal, heavy | 50, 15, 5 |
| Melee dodge chance | 0, 0, 0 |
| Defence hold after a defence | int[60, 120] |
| Blocking Angle | 0.0 |
| Weapon volumes | five of radius 2.0, height 2.0 (the club uses weapon 1) |
| Counter triggers | slot 0: code 2, chance 25 (see Defence); slot 2: none |
| Status, steal, drain | none |
| Wait-in-line | never (approach 3); shot scatter +3.0 only while the manager would hold it |

Attack slots (slot 1 and 3 unused):

| Slot | Type | Band (strict) | Chance, per tick | Facing cosine above | Range | Cooldown draw | Damage, knockback | Speeds | Flags |
|---|---|---|---|---|---|---|---|---|---|
| 0 | melee club | 0 to 12 | 85, 84 % | 0.9239 | 5.0 | int[30, 60] | 25, 0 | 1.0, 1.0, 1.0 | blockable |
| 2 | missile (thrown bone) | 10 to 75 | 50, 49 % | 0.0 | none | int[60, 240] | 25, 0 | 1.0, 1.0, 1.0 | blockable by the hero's guard |

Missile words of slot 2: flight mode 0 (straight shot, kind 0), launch speed 2.0 per tick, range 100 (lifetime trunc(100 / 2) = 50 ticks), radius 0.5, aim inaccuracy 0.0, shot count 1, no area, no status, six effect ids (-1, -1, 160, -1, 160, 160): the same effect plays at a blocked hit, a world impact and an unblocked hit. Element = T = 1.

#### Animation ids and clips (speed byte 2 throughout)

| Role | Id | Clip, keys | Notes |
|---|---|---|---|
| Idle base | 0 | clip 5, 22 keys, loops | |
| Walk, patrol, wait-in-line walk | 2, 4, 11 | clip 0 | 7.56 units per second |
| Keep approach (gaits 0 to 2) | 24 | clip 9, 8 keys, loops | 23.58 per second |
| Keep hold | 25 | clip 24, 18 keys, loops | |
| Back off (gait 3) | 26 | clip 11, 8 keys, loops | 23.58 per second backward |
| Sentry stand / look | 7 / 8 | clip 19 / clip 20 | as the Basher |
| Rest fidget / patrol fidget | 3 / 6 | clip 16 (30 keys, gate 116) | both |
| Chat | 14 | clip 15 (30 keys, loops) | no 111 to 114 |
| Search idle / end / alert | 16 / 17 / 18 | clips 21 / 22 / 23 | as the Basher |
| Hide pick / emerge | 9 / 10 | clip 32 / clip 33 (gate 104) | selector 0 on every record |
| Missile dodge | 53 or 54 | clip 29 (13 keys, gate 49) | 6.0 units sideways |
| Block / block-hit flash | 56 / 126 | clip 10 / clip 42 | |
| Hit reaction | 57 | clip 4 (9 keys, gate 33) | |
| Death / victory | 58 / 73 | clip 3 / clip 40 | |
| Slot 0 club | B 34, D 36 | clip 8 (15 keys, gate 57), clip 26 (17 keys, gate 65) | A, C, E skipped |
| Slot 2 throw | B 44 | clip 2 (17 keys, one-shot, gate 65) | A, C, D, E skipped. Spawn event on frame 10, ticks 40 to 43 after the request |

The Bone Thrower has no circle clips in use (strafe is not its archetype) and no melee dodge.

#### Wake and start

As the shared rules, with the Bone Thrower's numbers: notice cycle 62 ticks, awareness 80, standing-hero reach 70 units, no always-line-of-sight flag. Entering agro: outer 0 on the first tick (outer = 3, inner 4, route flag 1), the hold from the next tick, and the first attack choice on that second tick.

#### State table (agro)

| State | Id and clip | Gate / exits |
|---|---|---|
| Set-up (outer 0) | idle pick id 0 | one tick, then outer 3 inner 4 |
| Keep hold (inner 4) | 25, clip 24 | no gate. d > 50 gives approach (inner 3); d < 10 gives back off (inner 5) |
| Approach (inner 3) | 24, clip 9 | steers to the hero; ends when d < desired (18 to 42): inner 4 |
| Back off (inner 5) | 26, clip 11 | steers to the point hero - dir x 50; ends when d > desired |
| Slot 2 throw (outer 2) | B 44, clip 2 | spawn at s + 40; gate s + 65; end path s + 66 |
| Slot 0 club (outer 2) | B 34, clip 8 | window s + 20 to s + 31; hit at s + 20; gate s + 57; end path s + 58 |
| Missile dodge (outer 6) | 53 or 54 | 49 ticks |
| Block (outer 7) | 56, 126 | until the swing ends |
| Hit reaction | 57 | 33 ticks |
| Death | 58 | 77 ticks |

#### Attack chooser

Slot 2 is tested first: cooldown below 1, 10 < d < 75, facing cosine above 0.0 (within 90 degrees of its facing), roll int[1, 100] below 50 (49 %); a missile slot needs no route. Then slot 0: cooldown below 1, 0 < d < 12, facing above 0.9239, roll below 85 (84 %), a route to the hero or d below 10. Per-tick outcome with both ready: d from 12 to 75: throw at 49 %; d from 10 to 12: throw 49 %, else club 43 % (0.51 x 0.84), none 8 %; d below 10: club 84 %. The cooldowns are drawn at the choice (throw int[60, 240], club int[30, 60]) and again at the end path; the recovery int[30, 120] starts at the end path, so after a throw the next throw comes at least 60 to 240 ticks after the end path and usually about 165 (the later of the recovery and the cooldown).

**Throw.** Choice tick c, sequencer state 0 (missile slot: no walking), state 1 on c + 1 (s = c + 1; the hero and the steer point must be on the walk mesh, otherwise the Bone Thrower first turns toward the hero at blend 0.1 until the facing cosine is at least 0.95). B 44 is requested on s and state 2 begins on the same tick; the spawn-event frame (bit 0x20) is active on ticks s + 40 to s + 43, so one bone is released at s + 40 (the shot count is 1, so the sequencer then goes to state 3; no follow-through clip, so the state ends when B passes its gate, s + 65; the end path runs on s + 66). The bone's direction is computed from the launch point and the aim point at the release tick, not from the facing; the orient flag is clear, but a missile slot still turns the Thrower toward the hero every tick of the strike state at the row's aim turn weight.

**The bone** (SPEC-0029 Behavior 9, kind 0): launch point = emit point 0 (the event bit 0x20 selects point 0); aim point = hero position + (0, 2.5, 0) plus a horizontal offset of length U[0, 0.0] + 5.0 when the Bone Thrower is outside the camera cone (dot of camera forward and unit(Thrower - camera) below 0.85), in a random horizontal direction; velocity = unit(aim - launch) x 2.0 per tick; lifetime 50 ticks, so the bone flies up to 100 units. It hits the hero when it touches the hero's capsule: damage 25 (times the enemy damage scale 1.0), no knockback, no status; the hero's guard blocks it when the hero faces the bone. Standing 30 units away the hero is reached on the 14th move, 13 ticks after the release (s + 53; Distances table).

**Club (slot 0).** As the Basher's jab: window s + 20 to s + 31, one hit at s + 20, 25 damage, no knockback; with d above 5.0 the Thrower first walks in at 23.58 units per second (gait 0) until d is at most 5.0. Because the keep hold sends it backward when d is below 10, a Thrower clubs only when its choice passes before the back off has moved it away: the choice runs before the outer switch in every tick, so a hero standing next to a Thrower is clubbed on the first tick after recovery and cooldown allow (84 % per tick).

#### Movement

Keep-distance archetype (SPEC-0038 Behavior 44, outer 3): it holds between 10 and 50 units, approaches at 23.58 per second when the hero is beyond 50 until d is below the drawn desired distance (18 to 42), and backs off at 23.58 per second toward a point 50 units from the hero when the hero is closer than 10, until d is beyond the desired distance. It never wanders from this band by itself: a hero standing 30 units away is met with a motionless Thrower that throws every 150 to 300 ticks.

Movement (movement pass): the Basher's table applies (same rig and clips), with the keep-distance ids: walk-in id 24 is the run clip (clip 9, 0.393 per tick, 23.58 u/s), back-off id 26 is the back-run clip (clip 11, 0.393 per tick), the keep hold blends 0.25 and then its row weight 0.9 (0.925 effective); the retreat state never runs (retreat band 0 to 0). Worked example: a hero walks up to 6 units from a Thrower whose desired distance is 30: it backs off at 0.393 per tick toward the point 50 from the hero; with the hero standing still it needs 24 units, 62 ticks (about 1 second), faster than the hero can follow on foot (13.5) or running (19.29). Mistake to avoid: a slow, cautious back-off; it sprints backward.

#### Hit and hurt windows

Club: as the Basher's jab. Bone: one projectile per attack from the pool of ten shared projectile slots (SPEC-0029 Behavior 9); a full pool means no bone. The Thrower is hittable by the hero except in outer 6 (missile dodge), base state 16 and when script-held; the guard rule of the Basher applies in outer 7.

#### Weapon reach

Conventions are those of "Weapon reach tables" in SPEC-0064 (shared field behaviour). Row 241, Medium. The club is the Basher's jab.

| Slot (phase, clip) | Weapon | Window ticks (level contact) | Centre forward / up / side | Tor ahead, at the tick | Tor ahead, from the start | Tor feet vs origin |
|---|---|---|---|---|---|---|
| 0 club (B, clip 8) | w1 (2.0 x 2.0) | s+20 to s+31 (s+20 to s+31) | 0.4 to 2.5 / 1.6 to 3.0 / -1.6 to 1.9 | -1.4 to 5.0 | 1.9 to 8.1 | -2.0 to 3.5 |

#### Defence and counter rules

Melee defence is the Basher's: block 50 / 15 / 5 percent by swing class, dodge never, hold int[60, 120] after a defence (shorter than the Basher's 120 to 240). Missile defence: 49 % dodge, no block. The slot-0 counter (code 2: after a dodge with the hero's recover window open, roll int[0, 100] below 25) can start a club swing after a missile dodge; it needs Tor swinging at the time, so it essentially never fires.

#### Hero actions against this enemy

| Hero action | Result |
|---|---|
| Stands still 25 to 50 units away | the Thrower holds and throws at 49 % per tick once off cooldown: a bone every 150 to 300 ticks, 25 damage |
| Runs at it | the Thrower backs off at 23.58 per second when the hero comes within 10; a hero that reaches 12 units gets clubbed at 84 % per tick |
| Swing at it in front within 8 | blocked 50 % / 15 % / 5 % (quick / normal / heavy) |
| Guards facing it | blocks the club and the bone |
| Roberta's shots | 49 % dodged |
| Kills it | 85 hit points: 22 undefended starting-sword hits of 4 |

#### Edge cases for implementers

- **No wait in line.** The Thrower always fights; its weight (8) still counts in the manager's budget, so while it is nearest the hero it keeps a Basher or Elite behind it from attacking at capacity 0.
- **Thrown, not lobbed.** The bone is a straight shot (mode 0), unlike the Ixi Archer's lob; it ignores gravity and is gone after 50 ticks.
- **Camera scatter.** Outside the camera cone the aim point is moved 5.0 units in a random horizontal direction, so most off-screen bones miss a standing hero (the capsule radius is about 1.2).
- **Club range.** The club reaches 5.0 and the Thrower walks in to use it; a Thrower is not harmless at close range.
- **Chat.** The Thrower has only chat id 14.

#### What a port most likely gets wrong

1. Giving it a lob or an area. It throws a straight bone: speed 2.0 per tick, range 100.
2. Making it retreat to 50 units. It holds anywhere between 10 and 50 and backs off only below 10.
3. Forgetting the club at 84 % per tick below 12 units.
4. Throwing on the choice tick: the bone leaves 40 ticks after the throw clip is requested.
5. Using the Basher's defence hold (120 to 240) or the Elite's block numbers.
6. Making it wait in line.

#### Test vectors

**T1: Bone Thrower in agro, hero standing 30.0 units ahead and on screen (t = 0 setter tick), cooldowns 0.** t = 1 set-up; t = 2 first choice: slot 2 at 49 % per tick (c = 2 + G, G geometric p = 0.49, mean 1). s = c + 1; the bone is released on c + 41, reaches the hero on c + 54 (25 damage); the gate passes on c + 66 and the end path runs on c + 67 (recovery int[30, 120]; throw cooldown int[60, 240] drawn again). The next choice is at c + 67 + max(recovery, cooldown) + G'. Over 600 ticks: 3 throws (starting near ticks 3, 235 and 470), 75 damage before armour on a motionless hero.

**T2: hero standing 8.0 units from a Bone Thrower in agro.** t = 2: slot 2 not eligible (d below 10); slot 0 at 84 %: c = 2 + G (G mostly 0). The hold at t = 2 sees d below 10 and starts the back off (0.393 per tick); the sequencer (outer 2) takes over on the choice tick, so the Thrower walks in (gait 0) from 8.0 to 5.0 (8 steps), s about c + 10, hit at s + 20 (25), end path s + 58, then backs off to beyond 18 to 42 units during the recovery 30 to 120.

**T3: defence.** A Roberta shot that crosses the Thrower's body within 60 ticks: r < 50 (49 %) gives outer 6 and clip 53 or 54 for 49 ticks, unhittable, then hold int[60, 120].

#### Open questions

1. Whether the bone's first contact with a hero standing in front of the camera is a direct hit or needs the aim offset. Resolved (High; proactive (open close-out)): a direct hit; the aim inaccuracy is 0, so inside the camera cone the bone is aimed straight at the hero's feet + (0, 2.5, 0) with no offset, and the projectile step tests the 0.5 sphere against the hero's capsule each tick (SPEC-0029 Behavior 9); the 5.0 offset applies only outside the cone.
2. Where emit point 0 lies on the model. Resolved (Medium-High; proactive (open close-out)): at the release tick s + 40 it is at (-2.04, 2.80, 0.70) from the Thrower's feet (side, up, forward; Distances and ranged reach table).
3. Resolved (proactive (open question sweep, round 5)): weapon contact geometry for the club is the general rule, as for the Basher (question 1); the distances are in this kind's Weapon reach table.

### Werelupe Elite in the field (row 242)

The Elite's full fight guide is SPEC-0063 Round 4 (identity, slots, clips, charge, three-hit swing, defence, counters, test vectors). Every combat number there holds in the field. This part lists what is different when an Elite stands in the Werelupe Burrows or the Coliseum instead of the Meridell arena.

| Item | Arena Elite (SPEC-0063) | Field Elite |
|---|---|---|
| Where | Meridell arena, region 1183, one record | Werelupe Burrows Lower: 11 records (10 hidden, 1 patrol); Coliseum round 5: 2 records; the arena record is counted in the 14 in play |
| Start | scripted: intro scene, base state 33 until released, hero 30 units away | none: created in its default state by streaming (proximity, cap 12 live enemies in pack 1246) |
| Default word, emerge distance, cue, selector | 8, 40.0, 1, 0 (the arena record; 0xBC = 20.0 unused) | 8, 12.0, 1, 0 in the Burrows (wake at 8.4 for a standing hero, 9.0 slow walk, 18 walking, 21 running); 8, 40.0, 1, 0 in the Coliseum |
| Wake | hero idle inside 28 units at release | wake test of the shared rules; a group member's agro entry wakes the rest of its group 15 to 45 ticks later (check interval) with the same 104-tick emerge |
| Notice before agro | not used (starts hidden) | awareness 100, cycle 32, cone 0.707, near chance 90 far 30 over 1 to 100 units, search int[120, 240); a walking hero is noticed out to 150 units (probability 0.29 per cycle in the cone at 100 units) |
| Alarm groups | none | Burrows Lower group 1: three hidden Elites; group 2: two hidden Elites; group 3: one hidden Elite and the patrolling Elite; the other four Elites and the eight Bashers have no group |
| Drop | none | drop ordinals 21 to 32 and 37 on the Burrows records (a placement of the region), -1 in the Coliseum and the arena |
| Socialize | n/a | a patrolling Elite is the only one that can chat (flag 1, radius 20.0, quiet time [420, 480)); hidden Elites cannot |
| Attack-manager list | alone | shared by every enemy in agro of every area (up to 20 entries, weight 12 for the Elite); stand-off radius 10 when it waits |
| Victory pose | 73, then it walks home and hides again (default 8) | the same |

The Burrows Lower Elites share the room with the eight sentry Bashers, two Bone Throwers and four tentacles. With starting gear (capacity 0) exactly one of all of them may start an attack at a time; every refused Elite or Basher plays the wait-in-line walk to a stand-off point within 5 units of the hero, so a woken group closes in on the hero and attacks in turns. The Bone Throwers are never held back. In pack 1246 each of the 8 Bashers stands on its own isolated square walk mesh beside the King's pit and cannot leave it.

**The patrolling Elite.** One Elite in the Lower Burrows (record ordinal 25 of region 1246, alarm group 3) is a patrol (default 6) with route index -1, ping-pong flag set, a point pause of 240 to 480 ticks and no random stop. A route index below 0 installs no route (the set-up step requires an index of 0 or more below the number of paths in the area). The patrol handler then walks to the point of the area's locator list with index 0 + the route counter (both 0), pauses 240 to 480 ticks on arrival and repeats, because no route advance happens without a route. Resolved (movement pass; Medium-High, by SPEC-0038 Behavior 49, Invalid index): the target is entry 0 of the pack's entry list, about 406 units from the placement, so this Elite walks there with its walk clip (id 4, clip 0: 0.1259 per tick, 7.56 u/s; about 3,220 ticks, nearly 54 seconds, on a straight line over the mesh), pauses 240 to 480 ticks, and then keeps pausing there; when that point is outside its mesh it stops at the mesh border (no stuck handling). It does not stand at home.

**Movement (movement pass).** The field Elite moves with the Basher's clips (Werelupe Basher, Movement table) and the strafe ids of SPEC-0063 Round 4: strafe forward id 28 is the walk clip (7.56 u/s), the back-off id 32 is the back-run clip (23.58 u/s), the circles 4.04 u/s, the charge run 28.14 u/s; hold weight 1.0. A refused Elite walks to its stand-off point with id 2 (7.56 u/s). Each of the eight Burrows Bashers is confined to its own square mesh (navigation border), so it cannot follow the hero off it; a hero outside that square gives no route, and the leash ends the fight after 181 ticks while the hero is 15.0 or more from its walk waypoint.

**Weapon reach.** Conventions are those of "Weapon reach tables" in SPEC-0064 (shared field behaviour). The Elite's jab, charge hit and three-hit swing have the Basher's clips and weapon volumes; the rows (jab 1.4 behind to 5.0 ahead at the tick, charge hit 0.8 to 4.8, three-hit swing 1.0 behind to 5.4 ahead, 0.2 to 5.0 and 0.8 behind to 4.8 ahead) are in the Werelupe Basher's Weapon reach table. Medium.

**Open questions (Elite in the field).**
1. Resolved (movement pass): the behaviour of the Elite with route index -1 (above: it walks to entry 0 of the entry list and pauses there).
2. Whether the Burrows Lower cap of 12 live enemies and the 20-entry manager list ever bind: eleven Elites, eight Bashers, two Bone Throwers and four tentacles are 25 records. Resolved (High; proactive (open close-out)): the cap of 12 binds and the 20-entry list never does. The 25 records lie in a block about 500 by 300 units, so most of them are within the 280-unit streaming range of a hero standing in the room, and the streaming pass binds the nearest up to the cap (SPEC-0006); at most 12 enemies of this pack are bound, and so at most 12 can be in agro and in the manager list, which never reaches 20.

### Evil Harris (row 150)

Evil Harris is a small striker that behaves like the Biting Spellbook: it circles at 8 to 13 units and attacks only when the hero has turned away from it. Face it and it never swings. It is weak (75 hit points, two 10-damage swings) and has no recovery between attacks. Three Harris wander the lap of the Endless Staircase (pack 1299, the library tower; the roster lists them under the Library) and six more wander the Park Shrine, where they are scripted props that cannot be killed.

#### Identity and tunables

| Item | Value |
|---|---|
| Record, name on the bar | monster type 70 variant 0, "Evil Harris", character key 807 |
| Hit points | 75 |
| Body | radius 1.0, height 2.0 |
| Class, approach routine | class 17, approach 4 (strafe) |
| Attack-manager weight, target type T, hostile | 5, 3, yes |
| Strafe band (row +0x4F4, +0x4F8) | 7.0 to 14.0. Desired distance 7 + 7 f, f = int[20, 80] / 100: 8.4 to 12.6, redrawn after every normal choice. Back off below 7, approach above 14 |
| Backbiter scale (row +0x500) | 0.0 (see the chooser) |
| Aim weight in hold states (row +0x84) | 1.0 |
| Recovery after an attack | 0 (int[0, 0]) |
| Notice | awareness 100, cycle 32, cone 0.9239, range 1 to 30, chance 75 near / 5 far (shared table), search int[30, 60) |
| Missile defence: look-ahead and hold / dodge / block | 180 / 100 / 0 |
| Melee block chance | 0, 0, 0 (never) |
| Melee dodge chance quick, normal, heavy | 25, 15, 15 |
| Defence hold after a defence | int[60, 120] |
| Blocking Angle | 0.0 (unused) |
| Weapon volumes | weapons 1 to 3: radius 2.0, height 1.0 |
| Counter triggers | none (-1) on both slots |
| Status, steal, drain, socialize, map search | none; flags 0 |
| Wait-in-line stand-off radius | 10 |
| Death | corpse removed 4 ticks after the death clip reaches 0.95 (about 48 ticks after death) |

Attack slots (slots 2 and 3 unused):

| Slot | Type | Band (strict) | Chance, per tick | Facing cosine above | Range | Cooldown draw | Damage, knockback | Speeds | Flags |
|---|---|---|---|---|---|---|---|---|---|
| 0 | melee | 0 to 15 | 50, 49 % | 0.9239 | 5.0 | int[60, 300] | 10, 0 | 1.0, 1.0, 1.0 | blockable |
| 1 | melee | 0 to 15 | 50, 49 % | 0.9239 | 5.0 | int[30, 120] | 10, 1.0 | 1.0, 1.0, 1.0 | blockable |

Both roll thresholds are multiplied by the backbiter scale 0.0 whenever the dot product of the Harris's facing and the hero's facing is at most 0, which makes them 0 (a roll never falls below 0).

#### Animation ids and clips (speed byte 2 throughout)

| Role | Id | Clip, keys | Notes |
|---|---|---|---|
| Idle base | 0 | clip 34, 60 keys, loops | fidget ids 115 to 119 present (clips 21, 22, 31, 34, 21) |
| Walk, patrol, wait-in-line walk | 2, 4, 11 | clip 5, 10 keys, loops | 7.50 units per second (0.125 per tick) |
| Strafe forward (gaits 0 to 2) | 28 | clip 9, 10 keys, loops | 12.72 units per second (0.212 per tick) |
| Strafe hold | 31 | clip 8, 15 keys, loops | |
| Circle left / right | 30 / 29 | clip 7, 9 keys, loops (the same clip for both) | 8.42 units per second (0.140 per tick) |
| Back off (gait 3) | 32 | clip 6, 10 keys, loops | 15.90 units per second backward (0.265 per tick) |
| Stand / look | 7 / 8 | clip 8 / clip 21 (55 keys, gate 216) | |
| Rest fidget / patrol fidget | 3 / 6 | clip 21 / clip 31 (45 keys, gate 176) | |
| Search idle / end / alert | 16 / 17 / 18 | clip 31 (gate 176) / clip 26 (30 keys, gate 116) / clip 33 (13 keys, gate 49) | |
| Hide pick / emerge | 9 / 10 | clip 25 (25 keys, loops) / clip 28 (11 keys, gate 41) | unused: no Harris starts in state 8 |
| Melee dodge | 55, 126 | clip 18, 12 keys, gate 45 | back 5.14 units per second (about 4 units) |
| Missile dodge | 53 or 54 | clip 19, 15 keys, gate 57 | sideways 4.87 per second (about 4.9 units) |
| Block | 56 | clip 32 (23 keys) | never used: block 0 |
| Hit reaction | 57 | clip 23, 30 keys, one-shot, gate 116 | back 7.54 per second (about 15 units in all) |
| Death | 58 | clip 17, 12 keys, gate 45 | back 14.65 per second (about 11.7 units) |
| Slot 0 | B 34, D 36 | clip 10 (20 keys, gate 77), clip 14 (24 keys, gate 93) | A, C, E skipped |
| Slot 1 | B 39, D 41 | clip 11 (18 keys, gate 69), clip 14 | A, C, E skipped |

Harris has no clip 73, so after a killing blow on the hero the setter drops straight to state 0.

#### Where it is met and how it starts

- **Endless Staircase, pack 1299 (3).** Embedded records, default 5, R 100, no pause, no alarm group, no drop: at (-114.9, 27.1, -73.3), (-125.4, 29.3, -59.6) and (-141.5, 34.4, -74.8) on the pad-puzzle lap. They take the pack's only (ground) mesh at start and wander over a 100-unit square about their homes, never pausing; the library hall's Biting Spellbooks (pack 1327) are a different pack. Killable.
- **Park Shrine (7).** Six Harris (region 1403, script entities 16 to 21) are loaded only after the stage 4 arm trigger and wander the whole mesh (R 0, no pause). Each update the Shrine script sets all six undamageable (hit-point loss is skipped; hit reactions and the switch to agro still happen), so they can never be killed there. While stage 4 is armed the script also restores all six (entities 16 to 21) to full health. The stage-4 trigger is a 2.2 by 2.2 column at (0.405, 1043.506) that only a Harris can fire (SPEC-0043): the Harris that enters it is removed for good and counts as one of the six passes, so each of the six must be driven into the column by ordinary movement (wandering, backing away from a closing hero, or the sword knock-back of about 14.6 units straight away from the hero). On each pass the script sets the behaviour code of all six to 0 and moves them to random points of a 12-point list round the arena, then sets it back to 5 (wander) for the survivors. One more Harris (region 1404, default 0, beside a 5-hit-point type-102 creature, both placed at the column's centre) is used by the same script and the script moves it onto the firer's spot for the pass scene. The mesh the Shrine Harris use is ground mesh 0 (SPEC-0043).
- Because no Harris has a group, a hit or a notice involves only that Harris.

The idle and notice rules are the shared ones (wander only; no sentries or patrols). A Harris that notices the hero at up to 21 units (standing) stops, plays the search idle, passes the second check and plays the 13-key alert for 49 ticks before it fights.

#### State table (agro)

| State | Id and clip | Gate / exits |
|---|---|---|
| Set-up (outer 0) | idle pick id 0 | one tick, then outer 4 inner 10 |
| Strafe hold (inner 10) | 31, clip 8 | hold timer down by 1 per tick; at 0 one roll r = int[1, 3] (1 stay, 2 circle left, 3 circle right; 2 and 3 become stay while the lock is up), the timer is redrawn int[60, 180], and the distance tests run and win: d > 14 gives approach; d < 7 with the lock below 1 gives back off |
| Approach (inner 7) | 28, clip 9 | until d < desired (8.4 to 12.6): hold 0, inner 10 |
| Circle left / right (inner 8, 9) | 30 / 29, clip 7 | hold down by 1 per tick, at 0 inner 10; a wall block gives inner 10 with a 120-tick lock |
| Back off (inner 11) | 32, clip 6 | steers to the point hero - dir x 14; until d exceeds desired, or a wall block (lock 120) |
| Slot 1 swing (outer 2) | B 39, clip 11 | window s + 32 to s + 43; one hit, on an idle level hero in front at s + 35 (10, knockback 1.0); gate s + 69; end path s + 70 |
| Slot 0 swing (outer 2) | B 34, clip 10 | window s + 28 to s + 51; one hit, on an idle level hero in front at s + 28 (s + 30 when the swing starts 5.0 away) (10); gate s + 77; end path s + 78 |
| Melee dodge (outer 5) | 55 | 45 ticks, unhittable |
| Missile dodge (outer 6) | 53 or 54 | 57 ticks, unhittable |
| Hit reaction (outer 10) | 57 | 116 ticks, then outer 0 |
| Wait in line (outer 12) | walk id 2; no clip 73, so the 600 to 800-tick countdown is set at once | until released |
| Dead (base 16) | 58 | 45 ticks, corpse gone 4 ticks after the 0.95 point |

#### Attack chooser

Per SPEC-0038 Behavior 7: runs every agro tick in outer 1, 3, 4 and 11 while the recovery countdown (0 for Harris) is below 1 and the hero is attackable. Slots are tested 1, then 0. For each: cooldown below 1, 0 < d < 15, facing cosine above 0.9239 (the Harris faces the hero in the hold and while backing off, and its move direction while approaching), a route to the hero or d below 10 (melee), and a roll int[1, 100] below the threshold. **The threshold is int(50 x backbiter) = 0 whenever dot(Harris facing, hero facing) is at most 0.** The Harris turns to face the hero, so with the hero looking at it (the facings are opposite) the dot is negative and the threshold is 0: no roll can pass, and no normal attack ever starts. When the hero faces away from the Harris, so that both look the same way (dot above 0), the threshold is 50 and each slot passes at 49 % per tick: slot 1 first, then slot 0. A hero strafing around the Harris while still looking at it keeps the dot negative. There is no counter attack (no counter codes), so a Harris never reacts to a hit with an attack. On a choice the cooldown is drawn (slot 1 int[30, 120], slot 0 int[60, 300]) and drawn again at the end path; the recovery is 0, so the next slot can be chosen on the tick after the end path. The sequence after a back swing is therefore slot 1, then slot 0 straight after, then slot 1 again once its 30 to 120 tick cooldown ends, as long as the hero stays turned away.

**Swing (either slot).** Choice tick c, sequencer state 0. With d above 5.0 the Harris walks forward with gait 0 (id 28, 0.212 per tick) until d is at most 5.0, then state 1 on the next tick (s). On s: A is missing, B is requested and state 2 begins on the same tick. Slot 0: weapon 2 on frames 7 to 9 and weapon 1 on frames 10 to 12 form one continuous window s + 28 to s + 51, one hit on the first window tick the weapon touches the hero (s + 28 for an idle level hero within 4 units, s + 29 at 4.5, s + 30 at 5.0; damage 10, knockback 0), gate s + 77, end path s + 78. Slot 1: weapons 1 and 2 on frames 8 to 10, window s + 32 to s + 43, one hit on the first touching tick, s + 35 for an idle level hero 2.0 or more in front (s + 34 at 1.5 units; damage 10, knockback 1.0, the hero goes into the long-fall state 48), gate s + 69, end path s + 70. The clips carry 4.29 (slot 0) and 6.26 (slot 1) units per second of forward travel. A hero blocking (L1, in front) takes the reduced damage and plays the blocked clip D (24 keys, gate 93).

#### Movement

Strafe archetype (SPEC-0038 Behavior 44, outer 4): it stays 7 to 14 units from the hero (desired 8.4 to 12.6), approaches at 0.212 per tick when farther than 14, backs off at 0.265 per tick when nearer than 7 and circles left or right at 0.140 per tick in 60 to 180-tick legs. With no attack ever starting while the hero faces it, the Harris just orbits.

Movement table (movement pass; speed byte 2; the size selector of its records does not change these):

| Mode | Source | u/t | u/s | Turn | Clip (id: clip, keys, loop) | Exits |
|---|---|---|---|---|---|---|
| Wander, patrol, walk home, wait in line | root | 0.1251 | 7.50 | steering 0.3 / 0.1 | 2, 4, 11: clip 5, 10, loop | goal or point reached |
| Search look (state 2) and idle fidget 6 | root forward | 2.97 units over the clip (177 ticks) | 1.0 | aims at the last-seen point (weight 1.0) | 16 and 6: clip 31, 45, one-shot | clip end |
| Strafe approach (inner 7) and sequencer approach | root | 0.212 average; per key 0.074 to 0.266 | 12.72 | steering | 28: clip 9, 10, loop | d below desired or the slot range 5.0 |
| Circle (inners 8, 9) | root sideways | 0.1403 | 8.42 | faces the hero | 29: clip 7 mirrored (+x); 30: clip 7 (-x); 9, loop | hold count, blocked flag |
| Back off (inner 11) | root backward | 0.265 average | 15.90 | faces the hero | 32: clip 6, 10, loop | d above desired; point 14 from the hero |
| Strafe hold | none | 0 | 0 | aim 1.0 | 31 | hold count |
| Dodges 53, 54; 55; block 56 | root | 4.87 sideways; 4.11, 5.51 backward | - | none | clip 19 (54 its mirror); 18; 32 | gate |
| Hit reaction kind 0 | root backward | 15.08 units over 117 ticks | 7.54 | snapped to the attacker | 57: clip 23, 30 | gate |
| Death | root backward | 11.72 units over 45 ticks; the corpse is removed 4 ticks after progress 0.95 | - | none | 58: clip 17, 12 | removal |

Worked example (orbit). Hero standing 10 away facing the Harris: hold for int[60, 180] ticks, then a draw; a right circle (inner 9, id 29) moves it 0.1403 per tick sideways while it faces the hero, so over a 120-tick leg it travels 16.8 units around the hero and d drifts outward (it steers toward a point 2.0 to the side every tick, not along a circle); when d passes 14 it walks back in at 0.212 per tick.

What a port most likely gets wrong (movement): a stationary search (its look clip walks it 3 units forward); a short hit reaction (it slides 15 units over 117 ticks); orbiting at constant radius.

#### Hit and hurt windows

Weapon windows are given above (weapon volumes radius 2.0, height 1.0). The Harris is hittable by the hero except in outer 5 and 6 (dodges), base state 16 and when script-held. It never blocks, so the guard rule never applies to it. Damage to the hero: 10 before armour and shield.

#### Weapon reach

Conventions are those of "Weapon reach tables" in SPEC-0064 (shared field behaviour). Row 150, Medium. The tick text of the guide is authoritative for the hit ticks. Weapons are radius 2.0, height 1.0.

| Slot (phase, clip) | Weapon | Window ticks (level contact) | Centre forward / up / side | Tor ahead, at the tick | Tor ahead, from the start | Tor feet vs origin |
|---|---|---|---|---|---|---|
| 0 (B, clip 10) | w2 | s+28 to s+39 (s+28 to s+39) | 0.1 to 1.4 / 1.6 to 3.6 / -0.6 to 0.9 | -1.8 to 3.8 | 1.0 to 6.6 | -3.0 to 5.0 |
| 0 (B, clip 10) | w1 | s+40 to s+51 (s+40 to s+51) | -0.3 to 1.1 / 1.5 to 2.9 / -0.8 to 0.8 | -2.0 to 3.4 | 2.0 to 8.6 | -2.5 to 4.5 |
| 1 (B, clip 11) | w1 | s+32 to s+43 (s+32 to s+43) | -0.2 to 1.2 / 0.1 to 5.0 / -0.5 to -0.2 | -2.0 to 3.8 | 3.4 to 9.5 | -5.0 to 6.5 |
| 1 (B, clip 11) | w2 | s+32 to s+43 (s+34 to s+43) | -0.3 to 1.2 / 0.3 to 4.9 / 0.2 to 0.4 | -2.2 to 3.8 | 3.0 to 9.4 | -5.0 to 6.5 |

The Harris reaches 3.4 to 3.8 ahead at the tick, less than its 5.0 start range, and carries itself forward 2 to 6 units during the swing (up to 9.4 from where it started).

#### Defence and counter rules

Melee defence, once per hero swing (conditions as the other rows): r1 at most the block chance (0) never passes; r2 below the dodge chance gives outer 5: quick swings 24 %, normal and heavy 14 %. A dodge sets the hold to 180 and plays 55 (45 ticks, a 4-unit hop back); the hold is then redrawn int[60, 120]. Missile defence: a roll below 100 (99 in 100) on every tick a hero shot's 180-tick look-ahead path meets the body, so practically every shot that arrives while the Harris is in agro and outside a hit reaction is dodged; the dodge plays 53 or 54 for 57 ticks and the exit holds defence off for int[60, 120] ticks, the only open window. A shot that finds it unaware (wandering, not yet in agro) lands doubled, and a chain of shots then keeps it in its 116-tick reaction: 11 landed hits (70 magic, about 6 s) kill it when the first shot finds it unaware, 15 shots and 12 hits (80 magic, 8.3 s) when it is already in agro. There is no block and no counter.

#### Hero actions against this enemy

| Hero action | Result |
|---|---|
| Faces the Harris and fights it | it never attacks; swings land 76 % (quick) or 86 % (normal, heavy); each hit keeps it staggered for 116 ticks and knocks it back about 15 units |
| Turns away within 15 units | the Harris closes to 5 units and swings: 10 damage, then the second swing, about every 70 to 80 ticks in all |
| Roberta's shots | A 99-in-100 roll on every tick a shot is on course: dodged unless it finds the Harris unaware or inside a hit reaction (a landed hit locks it for 116 ticks, so a chain keeps landing) |
| Hits it in the Park Shrine | hit reaction and agro, but no hit-point loss |
| Walks past a wandering Harris at 30 units | not noticed standing (reach 21); noticed with probability 0.04 per cycle when walking |
| Kills it (Library) | 75 hit points: 19 undefended starting-sword hits of 4; the corpse is gone in about 48 ticks |

#### Edge cases for implementers

- **The backbiter is the whole AI.** The roll threshold is zero while the facings are opposed; do not reduce it to a chance, and do not use the dot of the direction to the hero.
- **No recovery.** Slot 1 and slot 0 can follow each other on consecutive ticks after the end path; only the cooldowns (counting down while the hero is attackable) limit them.
- **Hit reaction.** The 116-tick clip is restarted by every hit and moves the Harris about 15 units back in all (7.54 units per second), so a chain of hits keeps it away from the hero.
- **Death.** Clip 17 throws the body back 11.7 units over 45 ticks and the corpse vanishes 4 ticks after progress 0.95.
- **Park Shrine.** The six Harris cannot die there (undamageable); a port that applies hit points to them breaks the stage scripts, which expect them alive.
- **Wait in line.** A Harris refused by the manager plays the wait-in-line walk to a stand-off point within 5 units of the hero (it has no victory clip, so the 600 to 800-tick countdown is set at once).

#### What a port most likely gets wrong

1. Letting it attack a hero who faces it. It attacks only when both look the same way.
2. Giving it a recovery or a block. Recovery is 0; block is 0; its only defence is the 14 to 24 % melee dodge and the 99 % missile dodge.
3. Using a wind-up or a longer corpse. There is none; the corpse is gone in 48 ticks.
4. Killing the Shrine Harris.
5. Applying the Werelupe notice values: Harris reach is 30 units (21 for a standing hero), awareness 100, cycle 32.
6. Forgetting the 116-tick hit reaction and its 15-unit push.

#### Test vectors

**H1: hero standing 20.0 units away facing the Harris, Harris in agro (t = 0 setter tick).** t = 1 set-up (outer 4, inner 10, hold 0). t = 2: first choice: threshold 0, nothing; the hold rolls r, the hold is redrawn 60 to 180, and d = 20 above 14 gives approach (inner 7, 0.212 per tick). The Harris reaches d < desired (8.4 to 12.6) after (20 - 10.5) / 0.212 = 45 ticks (36 to 55), holds, circles left or right in 60 to 180-tick legs. No attack starts in 600 ticks.

**H2: hero 10.0 units away facing away (the Harris is behind him).** Dot above 0, threshold 50. t = 2: slot 1 at 49 %: c = 2 + G. d = 10 above 5.0, so the Harris walks 5 units in 24 steps; s = c + 26; window from s + 32, hit on the first touching tick s + 35 = c + 61 (10, knockback 1.0, hero into state 48); gate s + 69; end path s + 70. The next tick slot 1 is on cooldown (int[30, 120] drawn at the end path); slot 0 (cooldown 0 unless used) is chosen at 49 % per tick: B at s' = end + 2 + G; hit s' + 28; end path s' + 78. A hero who stays turned takes 6 to 8 hits (60 to 80 before armour) in 600 ticks. If the hero turns around, the threshold falls to 0 on that tick and no further swing starts, but one already started completes.

**H3: defence.** A normal Tor swing at a Harris 6 units ahead: r2 below 15 (14 % of the time) dodges: outer 5 for 45 ticks, a 4-unit hop back. Otherwise it lands (damage 4 against an enemy in agro, which then stands in a 116-tick hit reaction).

#### Open questions

1. Which navigation mesh the six Shrine Harris use at run time and whether their wander goals stay on the walkable ground (SPEC-0043 open question). Resolved (High; proactive (open close-out)): all six placements lie inside triangles of mesh 0 of the Shrine pack, the first ground mesh, so the navmesh pick gives them mesh 0; with R 0 each wander goal is the centroid of a random triangle of that mesh, so the goals always lie on walkable ground.
2. What the script call that sets the behaviour code 0 does to a class-17 enemy that is already wandering. Resolved (High; proactive (open close-out)): the set-behaviour command copies the parameter block and stores the code as the default-state word, and when the default state changes the class state is set to 0 at once (SPEC-0038 Layout, open question 20); state 0 then enters the default state on the next think tick. A second call that sets 5 again repeats this.
3. Whether the facing used by the backbiter test is the full three-dimensional forward vector or the horizontal one. Resolved (Medium-High; proactive (open close-out)): the test is a dot product of the two stored forward vectors with no flattening step; both vectors are horizontal for a grounded Harris and a hero on the ground, so it equals the horizontal dot, and differs only while the hero is pitched.
4. Weapon contact geometry (weapon radius 2.0, height 1.0). Resolved by the general rule (SPEC-0064, Shared field behaviour: Weapon contact and the hero's hurt shape); the distances are in the Weapon reach table; stand-in only for a port without attachment frames: contact when a level hero is from 2.4 behind to 3.8 ahead of the Harris at the tick.

### Altador Coliseum (pack 1371): rounds, casts and where each is described

The Altador Coliseum is the five-round arena that the master of the Coliseum (pack 1350) opens by posting the blackboard value 20 for world 157 and loading pack 1371 (SPEC-0041 module 1371, SPEC-0042 module 1350). It is a different arena from the Meridell arena of SPEC-0063. Each round loads one character region of six fighters; every region has the same ring geometry and the same record settings.

**Round selection (SPEC-0041).** The round index t is 4 if story flag 599 is set, else 3 if 598 is set, else 2 if 597, else 1 if 596, else 0; the round loads region 1372 + t. Flags 596 to 599 only ever become set in order, one per win (SPEC-0042: every win sets 596, and a win with 596 set sets 597, and so on up to 600), so the highest set flag names the next unfinished round. A lost round sets flag 595 and does not advance. After the fifth win (flag 600 set) the challenge is no longer offered. Every won round gives two of item 147. Region 1377, a single non-fighting host character (monster type 45 variant 5, "Sevaric", default 0), is loaded in every round.

| Round | Selected when | Region | Ring, in record order (bearings 0, 45, 90, 180, 225, 270 degrees) | Described in |
|---|---|---|---|---|
| 1 | none of 596 to 599 set | 1372 | Bandit, Rogue, Thug, Bandit, Thug, Thief (brigands) | the Road and Ridge Brigands section |
| 2 | 596 set, 597 clear | 1373 | Dark Ixi Spearman, Dark Ixi Spearman, Dark Ixi Archer, Dark Ixi Archer, Dark Ixi Spearman, Dark Ixi Archer | the Cogham Ixi Force section |
| 3 | 597 set, 598 clear | 1374 | Giant Drackonack, Drackonack, Drackonack, Giant Drackonack, Drackonack, Drackonack | the Road and Ridge Brigands section |
| 4 | 598 set, 599 clear | 1375 | Skeleton Fighter, Skeleton Archer, Skeleton Wizard, Skeleton Fighter, Skeleton Archer, Skeleton Wizard | the Skeleton Legion section |
| 5 | 599 set, 600 clear | 1376 | Werelupe Elite, Werelupe Basher, Werelupe Bone Thrower, Werelupe Bone Thrower, Werelupe Elite, Werelupe Basher | below |

**Settings common to all 30 records.** Default state 8 (hidden at home), emerge distance 40.0, cue word 1 (the fighter is drawn and hittable while it hides; no ambush cue), emerge selector 0, no alarm group (255), no drop ordinal (-1), no summon reservation. Positions: radius about 26.4 around the point (0, 7.21, 0) at bearings 0 degrees (25.81, 0), 45 degrees (18.45, 19.27), 90 degrees (0, 26.28), 180 degrees (-26.40, 0), 225 degrees (-18.34, -18.69) and 270 degrees (-0.12, -26.51), each facing the centre. The round is won when all six characters of the region are dead (the watcher thread tests the region's character range every tick) and lost when both heroes have died (the first death swaps to the other hero).

**Start (the arena-start method of SPEC-0063).** The intro scene takes control of the six fighters (base state 33, so their think loops are not stepped), moves them to their places and releases them with the hero returned to stance (action state 1, F1 = 0.7). The wake threshold is therefore 40.0 x 0.7 = 28.0 for a standing hero, 30.0 slow walking, 60.0 walking and 70.0 running; a hero at the centre of the ring is 26.2 to 26.7 from each fighter and wakes all six on the same tick. A hit on a hiding fighter wakes it at once with doubled damage. Capacity of the attack manager is 0 at starting gear: one attacker at a time, plus every fighter that approach routine 3 never makes wait.

### Altador Coliseum round 5: the Werelupe ring

Round 5 (region 1376, selected when flag 599 is set) puts two of each Werelupe variant on the ring.

| Ring position | Fighter | Hit points | Attack-manager weight | Approach | Distance to the ring centre |
|---|---|---|---|---|---|
| 0 degrees (25.81, 0) | Werelupe Elite | 110 | 12 | strafe | 25.8 |
| 45 degrees (18.45, 19.27) | Werelupe Basher | 75 | 8 | rush | 26.7 |
| 90 degrees (0, 26.28) | Werelupe Bone Thrower | 85 | 8 | keep distance | 26.3 |
| 180 degrees (-26.40, 0) | Werelupe Bone Thrower | 85 | 8 | keep distance | 26.4 |
| 225 degrees (-18.34, -18.69) | Werelupe Elite | 110 | 12 | strafe | 26.2 |
| 270 degrees (-0.12, -26.51) | Werelupe Basher | 75 | 8 | rush | 26.5 |

Records are in the order 0 to 5 as listed. Total 540 hit points. All six have the Burrows' table (emerge id 10, clip 33, 104 ticks) and their field values; the Basher, Bone Thrower and Elite sections above and SPEC-0063 Round 4 give every number used below.

**Wake.** A hero at the centre wakes all six on tick a (26.7 is below 28.0). Each requests its emerge clip on a + 1 (104 ticks), enters agro on a + 105 (the setter runs; each is added to the attack manager in the tick of its own think step, record order), runs the set-up on a + 106 and may choose its first attack on a + 107. A hero standing nearer one edge wakes only the fighters within 28.0; the others wake when the hero moves, because the test repeats every tick. Fighters keep their hide pose (clip 32) until their own test passes.

**The fight at capacity 0 (starting gear).** The list has six entries; the Elites weigh 12, the others 8. Only entry 0 (and nobody else, since every weight is above 0) may start an attack; approach 3 never waits, so the two Bone Throwers fire regardless, while Bashers and Elites that are not entry 0 wait in line: they walk (the 7.56 units per second walk) to a stand-off point within 5 units of the hero and stand facing it, which takes about 170 ticks from the ring. The order step of the manager moves entries every frame among fighters whose distances to the hero are within 5.0 of each other (all six at the start), so the front place alternates between fighters from frame to frame and the fighter that actually starts an attack on the first free tick is effectively arbitrary; once a refused melee fighter has come within 5 units of the hero it is the nearest and takes the front place for good. With capacity 15 (for example sword and armour items totalling 5) two fighters of weight 8 may attack together, with 39 up to five.

**What each fighter does at the start.**
- Bone Thrower (two): from a + 107 each throws at 49 % per tick (distance 26.3 is inside 10 to 75). A bone is released 41 ticks after the choice and reaches a hero at the centre 26.3 units away 11 ticks later (c + 52), for 25 damage; off-screen throwers add 5.0 units of aim scatter. Each Thrower then waits the later of its recovery (30 to 120) and its throw cooldown (60 to 240), about 165 ticks on average, so it throws 2 to 3 times in 600 ticks. If the hero walks to within 12 units of a Thrower it clubs (84 % per tick).
- Basher (two): when it is released from the line it charges from the ring at 49 % per tick (distance in 20 to 50, facing above 0.966): a 37-tick wind-up, a run at 0.469 per tick, and a hit of 15 with knockback 2.0 about 87 ticks after the choice on a hero at the centre. The charge connects with probability (55 - 26.5) / 40 = 0.71 against a hero who does not move. If it waits in line it instead arrives next to the hero and jabs (25) at 49 % per tick when released.
- Elite (two): charges at 24 % per tick (distance in 10 to 50), a hit of 35 with knockback 4.0 about 90 ticks after the choice (probability (50 - 22.7) / 40 = 0.68 against a standing hero at 26.2); at 10 units or less it uses slot 2 (three unblockable hits of 40) or slot 0 (35). The three Elite strikes of SPEC-0063 apply unchanged.

**Round 5 timeline (hero standing at the centre of the ring, capacity 0, no guard, 600 ticks after tick a).** a: all six wake. a + 105: agro. a + 107: first choices; the first bones are chosen at about a + 108 and reach the hero at about a + 160 (two bones within a few ticks of each other, 50 damage together before armour). The first charge of a front-of-the-list Basher or Elite starts at about a + 108 to a + 112 and lands about 87 to 90 ticks later (a + 197 to a + 202): 15 or 35 damage and a knock-down. From a + 280 the refused melee fighters arrive at the hero and queue; each in turn becomes entry 0 and starts a jab, a charge from close range is impossible (the band starts at 20 for the Basher and 10 for the Elite), so at close range they jab (Basher 25, Elite 35) and the Elite uses its three-hit swing (40 each). Over 600 ticks a motionless, unguarded hero takes 4 to 6 bones (100 to 150), 2 to 4 melee strikes of 15 to 40 each, and is expected to be down well before the end; a moving hero outruns the charges (their run does not steer) and mostly takes bones.

**Hero actions against this ring.**

| Hero action | Result |
|---|---|
| Waits at the centre | all six wake together; bones from about 160 ticks, charges from about 200 |
| Runs to one edge | wakes only the fighters within 28 units (the nearest three or four); the rest stay hidden but can be hit (cue 1) |
| Hits a hidden fighter | wakes it at once, damage doubled, no emerge |
| Kills the Bone Throwers first | removes the only ranged damage; the melee fighters then arrive one at a time |
| Wears heavy gear (capacity 15 or more) | two melee fighters attack together |

#### Open questions (round 5)

1. The hero's position when the intro releases it. Resolved (High for the place, Medium-High for the stance; proactive (open close-out)): the centre of the ring, locator 14 at (0, 7.21, 0), in stance, so all six wake together (round 2, Open question 1).
2. The order in which the six enter the attack manager list on the wake tick and the effect of the manager's order step when all distances are within 5.0. Runtime-only for the bind order (needs a runtime check). Default if unresolved (rewrite choice): record order, then apply the order step exactly as the manager is described (SPEC-0038 Behavior 45).
3. Whether the intro scene plays any animation that changes the fighters' release pose. Resolved (Medium-High; proactive (open close-out)): the scene loops an intro animation on the grabbed fighters, but from the first think tick after the release they request the hide pick every tick, so they stand in the hide pick (round 2, Open question 2).

### Batch open questions

1. Resolved: the Elite patrol record with route index -1 walks to entry 0 of the pack's entry list and pauses there (Werelupe Elite in the field, The patrolling Elite).
2. Whether the Shrine's scripted Harris keep a class state of agro after the stage scripts reset their behaviour code. Resolved (High; proactive (open close-out)): no; each change of the default-state word sets the class state to 0 (SPEC-0038 Layout, open question 20), so a reset from 5 to 0 and back to 5 drops the agro state, and they return to wandering.
3. The contact geometry of every melee weapon of these kinds (radius 2.0, height 1.0 or 2.0 about an attachment frame). Resolved by the general rule (SPEC-0064, Shared field behaviour: Weapon contact and the hero's hurt shape); the distances are in each kind's Weapon reach table; stand-in only for a port without attachment frames: count a contact when a level hero is inside the "at the tick" range of the slot's row.

### Timing table for this batch (timing pass)

Confidence: High for the units and the clip arithmetic; Medium for the first-touch ticks (weapon-versus-body collision uses SPEC-0018; Tor idle and level, straight in front). Units, phases, what counts down and the common worked exchange are in SPEC-0064 (Shared field behaviour: Timing table and units); this table adds only what is particular to the kinds in this spec.

| Kind and attack | Window opens | Hit tick on a still hero in front | Notes on units |
| --- | --- | --- | --- |
| Werelupe Basher and Bone Thrower jab or club (slot 0) | s + 20 | s + 20 | Gate s + 57, end path s + 58, rate 1.0 throughout |
| Werelupe Elite three-hit swing (slot 2) | s + 28, 52, 76 | s + 29, 53, 77 | Each window hits once; a window that reopens re-enters the strike state one tick after it opens |
| Evil Harris slot 0 / slot 1 | s + 28 / s + 32 | s + 28 (to 4.0; s + 29 at 4.5, s + 30 from 5.0) / s + 35 (2.0 and more ahead; s + 34 at 1.5, s + 32 at 1.0) | One continuous window for slot 0 (weapons 2 then 1) |
| Ixi Spearman, Dark Ixi Spearman jab | s + 20 | no touch on a still level hero straight ahead at the 3.175-unit chase stop | Exact reach at the tick is at most 2.8 units ahead; see the jab section |
| Ixi Spearman family lunge (charge strike C) | c8 + 24 | rows 159, 160 and 161 exact fixed-pose, level, straight-ahead samples with no chase stop: c8+24 at 1.0–3.0, c8+27 at 3.5, c8+28 at 4.0–7.0 for each row; this is geometric contact only, not the live solid-body charge result | No chase stop in the charge states |
| Death Knight overhead (slot 1) | s + 16 | s + 20 from 2.5 to 4.5 (s + 19 from 5.0 or more, s + 18 at 1.0 to 1.5, no touch at 2.0) | Can Fly 2 row: it stays at about the hero's foot level, so the level figures apply; after a hit on s + 20 the gate is s + 60 (end path s + 61), after a hit on s + 19 gate s + 61 (end path s + 62), on s + 18 gate s + 62 (end path s + 63) |
| Death Knight quick cut (slot 0) | s + 14 | s + 14 (2.7 to 4.0) | Gate s + 84 after the hit, s + 39 without |
| Swamp Tentacle lash / smash (slots 0 / 1) | s + 11 / s + 25 | s + 13 (3.2 to 4.0), s + 14 (5.0) / s + 26 (5.0) to s + 28 (7.0) | Strike word 1.5 until the touch, 1.0 after |
| Tentacle slots 2 / 1 / 0 | s + 8 / s + 12 / s + 18 | s + 10 to 12 (3.2 to 6.0) / s + 12 to 13 (3.2 to 4.0) / s + 20 to 21 (3.2 to 5.0) | Strike word 2.0 until the touch |
| Slorg / Giant Slorg bite | s + 11 | s + 11 (s + 12 at exactly 2.5 units) / s + 14 (the Giant starts 4.2 to 5.0 away) | |
| Skull Spirit bite | s + 20 | s + 27 from the usual 3.9 (s + 25 from 3.6, s + 20 from 3.2; s + 28 at 4.5, s + 29 at 5.0) | Five volumes sweep around the skull |
| Spirit Sword slots 2 / 1 / 0 | s + 16 / s + 48 / B + 6 (s + 43) | s + 18 (3.0 to 7.5), s + 19 (8.0 to 8.5), s + 20 (9 to 10) / s + 48 (to 2.5), s + 49 (3.0 to 9.0), s + 50 (9.5 to 12.0) / B + 7 (5.5 to 10.5), B + 8 (3.5 to 5.0), B + 10 (3.0 and 11.0), none from 11.5 | Slot 0 started 11.5 to 13 away whiffs |
| Restless Spirit swipe (slot 1) | s + 26 | s + 26 (2.7 to 6.0) | Rate 3.0 |
| Crokabek-family ground peck (slot 1) | s + 8 | s + 8 | Grounded only |
| Biting Spellbook bite / lunge hit | s + 7 / c8 + 8 | s + 7 / c8 + 8 (up to 4.25) | |
| Werelupe charge hit (C) | c8 + 4 | c8 + 4 (3.5 to 4.5), c8 + 5 (5.0) | No chase stop in the charge states |
| Restless Spirit loop strike | none (direct damage) | first damage on the 61st tick of loop state 14, then every drawn interval + 1 ticks | The loop countdown starts at 60 and reloads from the row range; it is not a weapon window |

Ranged attacks of this batch use the shared pool (SPEC-0029 Behavior 9): a straight shot of launch speed v (units per tick) and range r lives trunc(r / v) ticks and moves that many times plus one; a homing shot lives twice that and slows to at most 1 unit per tick; a gravity shot's lifetime is trunc(60 T) + 1 with T already in ticks (the original's unit slip; keep it). The release tick is the first tick of the spawn event window (bit 0x20) on which the pending flag is clear; a full pool drops that shot but still counts it.

Flyers (Crokabeks, Spellbooks, Grunts) add their row's flying speed along the facing in sequencer states 7, 8, 9 and 12 on top of the clip's root travel scaled by the strike word, and aim their height at the hero's height plus 2.0 in those states; the height moves at most 0.1 per tick. A grounded Can Fly 1 flyer counts its ground countdown (300 to 900 ticks on the rows that use reaction kind 7) in the pre-step on every tick, whatever its outer sub-state.

**What a port most likely gets wrong (timing, this batch).**
1. Hitting on the window start (table above), or letting the Spearman's jab land on a hero it cannot reach.
2. Treating the Restless Spirit's loop strike as a weapon window: it is a countdown of 60 ticks, then the drawn interval plus 1.
3. Moving shots in units per second: launch speeds and ranges are per tick and units, and the lifetime is trunc(range / speed).
4. Stopping a grounded flyer's ground countdown while it is in a reaction; it runs every tick.

## Movement rules for every kind of this spec (movement pass)

Confidence: High for the clip data (root records, key counts, speed bytes, loop flags of each kind's own main table and clip set) and for the rules cited; Medium-High for the worked tick counts (flat ground, straight lines, facing already on the line; the steering blend and the per-key variation of the root step move them by a few ticks); Medium where a kind's section says so. The shared movement rules are in SPEC-0038 Behaviors 48 and 51 and SPEC-0040 Behaviors 14D and 14E; this section only lists what this spec's kinds need from them and the facts their clips add. All counts are ticks at 60 per second; "u/t" is units per tick and "u/s" units per second (u/t x 60). Speeds are at play rate 1.0 (no status effect, no attack speed word) and are the average over one clip cycle unless a per-key range is given.

1. **Where the speed comes from.** A ground gait moves the enemy only by the root travel of the clip it plays: each tick the step is the root record of the current key times 0.5 / S times the play rate (S is the clip's speed byte). With S = 2 one key's record is used for 4 ticks and a clip of F keys lasts 4F ticks per cycle; with S = 1, 2 ticks per key. A port that applies one key's record once per key instead of once per tick moves the enemy 4 times too slowly at S = 2; a port that applies the per-key value as a per-second value is off by 15 times. There is no acceleration: the first step lands one tick after the request, and the enemy takes one more full step on the tick after it decides to stop. A flyer adds its row speed (item 7). In the kind tables, the u/s of a one-shot clip (dodge, reaction, death) is its total travel over the clip's 4F ticks; the clip's gate (4(F - 1) + 1 ticks, 4(F - 1) from F = 25) ends it slightly earlier.
2. **Size does not scale travel.** The per-bone size selector of a placement (Giant Slorg, Giant Spyder, both Tentacles, the Ixi rows, the Mountain and Swamp Crokabeks, Evil Harris) scales the bone translations of the pose only; the root step is the clip's root record times the clip speed, never scaled. A giant kind is as fast as its own clips: the Giant Slorg's charge is 27.56 x its strike word 1.3 = 35.83 u/s, not 27.56 x 1.625.
3. **Clips with frame-stored roots.** In a clip stored in the older frame layout (mode 0; SPEC-0005) the root record of a key is the last eight bytes of that key's frame record, not the separate root table. The Restless Spirit's approach gait (id 101, clip 28) is such a clip; its root is 0.0427 u/t forward. Reading the root table of a mode-0 clip gives values that look like a strong sideways and upward drift (about 6.7 u/s sideways and 8.8 u/s up for that clip); a port must not do that.
4. **Held one-shot clips.** When a one-shot clip has finished and the same id is requested again, it is not restarted, and while it is held with a non-zero speed its root step is twice the last key's root step every tick (SPEC-0005, Layer step). In this spec: the Shooting Spellbook's back gait (id 104, clip 7, 20 keys) moves 8.51 u/s for 77 ticks and then adds nothing (its last key has no root); the Tentacle and Swamp Tentacle corpses keep drifting sideways at 0.057 u/t (3.4 u/s) after their death clip until they are removed (about 180 to 200 ticks; Medium for how it looks; reproduce it); an Evil Harris corpse is removed 4 ticks after its death clip and hardly moves. The arena Rogue's back gait is the other case (SPEC-0063).
5. **Turning.** While a gait moves the enemy, its facing turns by the steering blend only (0.3 per tick when it is settled in the goal triangle, 0.1 on a route edge or off the mesh; SPEC-0038 Behavior 51); back gaits face away from the move direction; side gaits face the hero. While it holds, it aims at the hero each tick with the row's hold weight (ground row word +0x84, flyers +0x88), a blend fraction per tick, not an angle: 1.0 snaps, 0.5 halves the remaining angle. The hold weights of this spec: Tentacles, Slorgs, Spellbooks, Spirit Sword, Death Knight (flying), Ixi spear rows, Dark Ixi Archer, Elite, Evil Harris 1.0; Werelupe Basher and Bone Thrower 0.9; Restless Spirit 0.7 (flying); Ixi Archer 0.7 (keep-distance hold: 0.78 effective); Crokabek rows and Skull Spirit 0.5. A hit snaps the facing to the attacker.
6. **Sideways gaits.** On every kind of this spec that has them, id 29 moves along the side axis in the clip's +x direction and id 30 along -x (Crokabek 11.39 u/s, Spirit Sword 4.22 and 4.52, Ixi rows 5.67, Werelupe rig 4.04, Evil Harris 8.42, Spyders 11.26). As SPEC-0038 Behavior 51 states, the circle therefore travels to the side opposite to its steering point.
7. **Flyers.** Can Fly 1 rows (Crokabeks and Spellbooks) and Can Fly 2 rows (Restless Spirit and Death Knight) add the row speed v / 30 per tick (2v u/s; v = flying idle velocity outside agro, flying agro velocity in agro) along the class axis while the playing clip is the gait clip of the current class, and in charge states 7, 8, 9 and 12 always along the facing. The height rules differ: Can Fly 1 aims at the hero's y + min(max fly height, 0.2 x horizontal distance), at least 2.0 above the ground under the goal, with the hero's y + 2.0 in the charge states, and has no gravity while airborne; Can Fly 2 aims at the hero's own y (the home height while waiting in state 8), at most the max fly height above the ground, has no lower bound, no charge override, and keeps gravity, so it stays within a few tenths of a unit of the ground in practice (SPEC-0040 Behavior 14D item 7).
8. **Knockback received.** None. Hero swings, shots and Mote objects give an enemy no velocity. A hit-reaction "slide" is the root travel of the reaction clip, straight away from the attacker because the hit snaps the facing first. The same holds for the dodge clips (53 and 54 sideways, 55 backward) and the death clip.
9. **Ledges and walls.** A ground enemy (and a knocked-down Crokabek) never leaves its current navigation mesh: the mesh border stops it, and a step that would leave it more than 2.0 above the ground under it is reversed (the edge guard), so it does not walk off ledges. An airborne Crokabek or Spellbook and every Can Fly 2 row skip the edge guard; the flying mesh border still confines them. Ground enemies push each other apart (full overlap, the moving one is pushed) and do not test world wall segments.
10. **Charges and the hero's body.** The hero is pushed out of every solid enemy body it overlaps (full overlap, horizontally, along the line between the centres; SPEC-0033 Behavior 53). A charging enemy is not solid on the run ticks (sequencer state 7) of charge kinds 0 and 1 (Slorg and Giant Slorg slot 2, Death Knight slot 2, the Crokabek's swoop) and is solid on every other tick, so a kind-2 or kind-3 charge (the Crokabek-family lunge, the Biting Spellbook's lunge, the Ixi spear lunges, the Werelupe charges) stays solid through the whole run and its hit clip. A solid charger whose line passes within its radius + 1.175 of a standing hero shoves the hero; a hero on the line is carried ahead of the body until the bodies part, and a hero off the line slides off sideways faster the further off he is. This is what decides whether the Crokabek lunge connects (Crokabek, Hit and hurt windows).

## Distances and ranged reach (every kind of this spec)

Confidence: High for the row values and the rules (SPEC-0038 Behaviors 3, 4, 7, 13, 23, 44 and 45; SPEC-0029 Behaviors 9, 13 and 13a); Medium-High for the emit points (posed strike clip of the most common record group, root travel included; the Ixi bows from the parent joint of the bow part at rest); Medium for the hit ticks of arcs (flat ground, hero standing still, feet level with the enemy's ground). The melee weapon reach is in each kind's own section and is not repeated here. The Ixi Henchman row 160, the Sludgy and the Werelupe Elite are in SPEC-0063.

**Reading the tables.** As in SPEC-0064 (Distances and ranged reach): d is the distance to the selected hero; the notice band is scaled by 0.7 for a hero standing still and 1.75 for a running hero, and by F2 through a wall unless the row sees through walls; the sight cone only halves the notice chance; "walk-in target" is the drawn desired distance; "back-off point" is the point that far from the hero; "last seen + x" is the lost-sight search point moved x further along the enemy-to-hero line. Shared by every kind: holds and circles face the hero every tick with the row's turn weight; the strafe circle sidesteps toward a point 2.0 to the side every tick and so keeps roughly the current d; the only leash is 181 ticks with no route while the walk waypoint is 15.0 or more from the hero; the lost-sight search gives up after 121 ticks at its point; streaming releases an enemy beyond 280; the chase stop is the enemy's radius + 1.175; a confused enemy may attack at radius + 1.425. For the rush kinds with the distance-hide flag (the two Tentacles) the hide gate is a real give-up distance: above it, each tick's roll sends the enemy home (state 14), where it rises again when the hero comes within 5.0 and returns to agro if the hero is within 20.0 when it has risen.

**Ranged reach.** s is the tick the strike clip B is requested (for the Ixi archers, B is requested on the wind-up gate, 77 ticks after the wind-up request). Emit points are side, up and forward from the enemy's feet on the release tick (side is the model's x axis). Every shot aims once, at release, at the hero's feet + (0, 2.5, 0) (arcs: the feet), plus the row's scatter; there is no lead. The hit volume is the shot's sphere against the hero's body cylinder (radius 1.175, height 3.5, Tor and Roberta alike), tested after each move only. Stops: the hero, a facing guard, the ground and the world once 2.0 from the start, the lifetime; never other enemies. The area words of the arcs never deal damage (SPEC-0029 Behavior 9, Area mode). Worked examples: hero standing still at distance D, scatter 0, enemy facing him.

| Kind, row, slot | Fires when | Release tick; emit point (side, up, forward) | Flight | Lifetime; travel; reach | Radius | Hero at D: tick of the hit |
|---|---|---|---|---|---|---|
| Slorg 229, slot 0 | 5 < d < 20, in front | s + 32; emit point 2 (-0.04, 2.28, -0.17) | straight 1.0 | 90; 91; 92.5 | 0.5 | 6: s + 36; 6.5: s + 36; 12: s + 42; 19: s + 49 |
| Giant Slorg 230, slot 0 | 7 < d < 100, in front (aims at weight 1.0) | s + 32; emit point 2 (-0.07, 3.03, 0.18) | straight 1.2 | 75; 91.2; 93 | 0.5 | 8: s + 37; 30: s + 55; 60: s + 80; 99: expires on s + 107 |
| Shooting Spellbook 135, slot 0 | 10 < d < 100, in front, airborne | s + 12; (0, 0.21, 0), the book hovering min(20, 0.2 d) above the hero | straight 2.0 | 50; 102; 102 | 0.5 | 11: s + 16; 30: s + 26; 60: s + 41; 99: s + 61 |
| Skull Spirit 228, slot 0 (single shot) | 7 < d < 40, in front | s + 36; (-0.05, 7.88, -3.88): high above and behind the skull | straight 1.0 | 30; 31; 28.75 | 1.0 | 8: s + 46; 20: s + 58; 25: s + 63; 29 to 40: never hit (it expires on s + 66) |
| Skull Spirit 228, slot 1 (five-shot volley) | 0 < d < 50, within 22.5 deg | s + 124, 148, 176, 192, 216; emit point swings from (-3.01, 4.98, -1.59) through (0.09, 3.77, -6.33) and (2.24, 3.02, -5.79) to (0.53, 2.69, 8.09) and (-3.65, 2.51, 4.78) | straight 2.0 | 25; 52; 52.25 (first shot; the shots from behind reach about 45) | 1.0 | at 30: + 14, + 17, + 16, + 9, + 11 ticks after each release; first shot: 5: s + 126; 20: s + 133; 49: s + 148 |
| Ixi Archer 156, slot 0 | 10 < d < 40, within 22.5 deg | s + 1; (-1.51, 2.22, 1.49) | straight 2.0, scatter U[0, 1] | 20; 42; 44.5 | 0.1 | 11: s + 5; 25: s + 12; 30: s + 14; 39: s + 19 |
| Ixi Henchman 157, slot 0 | 10 < d < 40, within 22.5 deg | s + 1; as the Archer | straight 2.0, no scatter | 20; 42; 44.5 | 0.1 | as the Archer |
| Ixi Henchman 157, slot 2 | 15 < d < 80, in front | s + 8; (-1.44, 2.92, 1.04) | arc, 0.3 at d 15 ramping to 1.0 at d 80 | lands | 0.1 | 16: s + 53; 40: s + 60; 79: s + 84 |
| Dark Ixi Archer 158, slot 2 | 0 < d < 80, in front | s + 8; (-0.57, 1.71, -1.38) | arc, 0.001 at d 0 ramping to 0.6 at d 80 | lands | 0.5 | 5: s + 60; 20: s + 78; 40: s + 94; 79: s + 139 |
| Werelupe Bone Thrower 241, slot 2 | 10 < d < 75, in front | s + 40; (-2.04, 2.80, 0.70) | straight 2.0 | 50; 102; 104 | 0.5 | 11: s + 44; 30: s + 53; 60: s + 68; 74: s + 75 |

Never fires: the Swamp Crokabek's slot 3 (no strike clip, so no spawn event). The Ixi Archer's scatter of up to 1.0 never makes it miss a standing hero (the body takes 1.275 sideways).

**Moves with no weapon test.**
- **Swamp Tentacle slot 2 (grab).** One test on the first tick of the strike state (s + 1): horizontal distance below 6.0 and the hero within 60 degrees of the Tentacle's facing (dot above 0.5), touched flag clear, path clear. That is the whole catch volume (a 60-degree sector of a flat disc of radius 6.0, no height limit); a failure ends the grab for this attack. The hero is carried at the enemy's attach point during the hold (SPEC-0040 Behavior 8c) and thrown with speed 5.0 when the release bit is seen (s + 76).
- **Restless Spirit slot 0 (loop strike).** No volume at all: the sequencer walks in until d is at most 4.0, then deals the slot's damage directly on the 61st tick of the loop and then every int[60, 120] + 1 ticks while d stays at most 5.0; d above 5.0 ends the loop. The hold distance is measured the same way as every other agro distance, so height and facing do not matter (the spirit hovers at the hero's height). Worked example: loop entered at d = 3.8 on tick L; the hero stands still: damage on L + 60 (the 61st loop tick), then L + 121 to L + 181; a hero who steps to 5.1 before L + 60 takes nothing.
- **Ixi Spearman row 159 slot 1 and Dark Ixi Spearman row 161 slot 1.** Never chosen: trigger chance 0 and counter code -1. Row 161's slot is also a charge with distance 0 to 0, which would end on its first run tick even if started. They have no reach to port.

#### Swamp Tentacle

| Distance (units) | Value |
|---|---|
| Notice band (x0.7 standing, x1.75 running) | 10 to 40 (7 to 28; 17.5 to 70) |
| Sight cone half-angle | 90 deg |
| Rush: hold band; walk-in target; back-off point | 3 to 12; 4.8 to 10.2; 14 |
| Slot 0 | melee, 0 < d < 17, in front, closes to 5 |
| Slot 1 | melee, 7 < d < 17, in front, closes to 7 |
| Slot 2 | grab, 5 < d < 17, in front, closes to 6 (catch below 6, within 60 deg) |
| Slot 3 | never chosen (band 0 to 0, chance 0) |
| Lost-sight search point | last seen |
| Distance hide (give-up) | d > 10, 75 percent a tick |
| Wait-in-line square | 50 |
| Chase stop; confused strike | 3.17; 3.42 |

#### Tentacle

| Distance (units) | Value |
|---|---|
| Notice band (x0.7 standing, x1.75 running) | 5 to 40 (3.5 to 28; 8.75 to 70) |
| Sight cone half-angle | 90 deg |
| Rush: hold band; walk-in target; back-off point | 3 to 15; 5.4 to 12.6; 17 |
| Slot 0 | melee, 3 < d < 17, within 22.5 deg, closes to 5 |
| Slot 1 | melee, 3 < d < 10, in front, closes to 4 |
| Slot 2 | melee, 3 < d < 13, in front, closes to 6 |
| Slot 3 | row 237: none; row 239: never chosen (band 0 to 0, chance 0) |
| Lost-sight search point | last seen |
| Distance hide (give-up) | d > 8, 100 percent a tick |
| Wait-in-line square | 50 |
| Chase stop; confused strike | 3.17; 3.42 |

#### Slorg

| Distance (units) | Value |
|---|---|
| Notice band (x0.7 standing, x1.75 running) | 3 to 40 (2.1 to 28; 5.25 to 70) |
| Sight cone half-angle | 45 deg |
| Rush: hold band; walk-in target; back-off point | 5 to 20; 8 to 17; 22 |
| Slot 0 | missile, 5 < d < 20, in front |
| Slot 1 | melee, 0 < d < 6, within 22.5 deg, closes to 3 |
| Slot 2 | melee, 6 < d < 30, within 22.5 deg, charge run 12 to 25 |
| Lost-sight search point | last seen |
| Wait-in-line square | 10 |
| Chase stop; confused strike | 2.35; 2.6 |

#### Giant Slorg

| Distance (units) | Value |
|---|---|
| Notice band (x0.7 standing, x1.75 running) | 5 to 70 (3.5 to 49; 8.75 to 122.5) |
| Sight cone half-angle | 90 deg |
| Rush: hold band; walk-in target; back-off point | 6 to 25; 9.8 to 21.2; 27 |
| Slot 0 | missile, 7 < d < 100, in front |
| Slot 1 | melee, 0 < d < 6, within 22.5 deg, closes to 5 |
| Slot 2 | melee, 5 < d < 30, within 22.5 deg, charge run 12 to 25 |
| Lost-sight search point | last seen |
| Wait-in-line square | 10 |
| Chase stop; confused strike | 4.17; 4.42 |

#### Crokabek

| Distance (units) | Value |
|---|---|
| Notice band (x0.7 standing, x1.75 running) | 10 to 60 (7 to 42; 17.5 to 105) |
| Sight cone half-angle | 45 deg |
| Strafe: circle band; walk-in target; back-off point | 3 to 15; 5.4 to 12.6; 15 |
| Slot 0 | melee, 7 < d < 20, in front, charge run 6.5 (ends early when d <= 10), only while airborne |
| Slot 1 | melee, 0 < d < 10, within 22.5 deg, closes to 3.5, only while not airborne |
| Slot 2 | melee, 0 < d < 15, in front, charge run 6 to 12, only while airborne |
| Lost-sight search point | last seen + 3 |
| Wait-in-line square | 10 |
| Hover height above the hero | min(30, 0.2 d), floor ground + 2 |
| Chase stop; confused strike | 2.17; 2.42 |

#### Mountain Crokabek

| Distance (units) | Value |
|---|---|
| Notice band (x0.7 standing, x1.75 running) | 10 to 60 (7 to 42; 17.5 to 105) |
| Sight cone half-angle | 45 deg |
| Strafe: circle band; walk-in target; back-off point | 3 to 15; 5.4 to 12.6; 15 |
| Slot 0 | melee, 7 < d < 20, in front, charge run 6.5 (ends early when d <= 10), only while airborne |
| Slot 1 | melee, 0 < d < 10, within 22.5 deg, closes to 3.5, only while not airborne |
| Lost-sight search point | last seen + 3 |
| Wait-in-line square | 10 |
| Hover height above the hero | min(100, 0.2 d), floor ground + 2 |
| Chase stop; confused strike | 3.17; 3.42 |

#### Swamp Crokabek

| Distance (units) | Value |
|---|---|
| Notice band (x0.7 standing, x1.75 running) | 10 to 60 (7 to 42; 17.5 to 105) |
| Sight cone half-angle | 45 deg |
| Strafe: circle band; walk-in target; back-off point | 10 to 30; 14 to 26; 30 |
| Slot 0 | melee, 7 < d < 20, in front, charge run 6.5 (ends early when d <= 10), only while airborne |
| Slot 1 | melee, 0 < d < 10, within 22.5 deg, closes to 3.5, only while not airborne |
| Slot 3 | missile, 0 < d < 40, within 22.5 deg, only while airborne (fires nothing) |
| Lost-sight search point | last seen + 10 |
| Wait-in-line square | 10 |
| Hover height above the hero | min(20, 0.2 d), floor ground + 2 |
| Chase stop; confused strike | 2.17; 2.42 |

#### Biting Spellbook

| Distance (units) | Value |
|---|---|
| Notice band (x0.7 standing, x1.75 running) | 10 to 40 (7 to 28; 17.5 to 70) |
| Sight cone half-angle | 45 deg |
| Strafe: circle band; walk-in target; back-off point | 5 to 16; 7.2 to 13.8; 16 (backbiter 0: no normal choice while the hero faces it) |
| Slot 0 | melee, 8 < d < 40, in front, charge run 20 to 30 (ends early when d <= 4.25), only while airborne |
| Slot 1 | melee, 0 < d < 10, in front, closes to 4.25, only while airborne |
| Lost-sight search point | last seen + 5 |
| Wait-in-line square | 15 |
| Hover height above the hero | min(20, 0.2 d), floor ground + 2 |
| Chase stop; confused strike | 3.17; 3.42 |

#### Shooting Spellbook

| Distance (units) | Value |
|---|---|
| Notice band (x0.7 standing, x1.75 running) | 10 to 40 (7 to 28; 17.5 to 70) |
| Sight cone half-angle | 45 deg |
| Strafe: circle band; walk-in target; back-off point | 3 to 15; 5.4 to 12.6; 15 |
| Slot 0 | missile, 10 < d < 100, in front, only while airborne |
| Lost-sight search point | last seen + 3 |
| Wait-in-line square | 15 |
| Hover height above the hero | min(20, 0.2 d), floor ground + 2 |
| Chase stop; confused strike | 3.17; 3.42 |

#### Restless Spirit

| Distance (units) | Value |
|---|---|
| Notice band (x0.7 standing, x1.75 running) | 3.5 to 50 (2.45 to 35; 6.12 to 87.5); sees through walls |
| Sight cone half-angle | 90 deg |
| Rush: hold band; walk-in target; back-off point | 3 to 5; 3.4 to 4.6; 7 |
| Slot 0 | loop strike, 0 < d < 15, within 45 deg, closes to 4, ends when d > 5 |
| Slot 1 | melee, 0 < d < 15, within 15 deg, closes to 6 |
| Slot 3 | never chosen (band 0 to 0, chance 0) |
| Lost-sight search point | last seen |
| Wait-in-line square | 10 |
| Hover height above the hero | 0 (cap ground + 5) |
| Chase stop; confused strike | 2.67; 2.92 |

#### Skull Spirit

| Distance (units) | Value |
|---|---|
| Notice band (x0.7 standing, x1.75 running) | 3.5 to 50 (2.45 to 35; 6.12 to 87.5) |
| Sight cone half-angle | 22.5 deg |
| Rush: hold band; walk-in target; back-off point | 10 to 30; 14 to 26; 32 |
| Slot 0 | missile, 7 < d < 40, in front (hits only to 28.75) |
| Slot 1 | missile, 0 < d < 50, within 22.5 deg |
| Slot 2 | melee, 0 < d < 20, within 22.5 deg, closes to 4 |
| Lost-sight search point | last seen |
| Wait-in-line square | 10 |
| Chase stop; confused strike | 3.17; 3.42 |

#### Spirit Sword

| Distance (units) | Value |
|---|---|
| Notice band (x0.7 standing, x1.75 running) | 10 to 50 (7 to 35; 17.5 to 87.5) |
| Sight cone half-angle | 22.5 deg |
| Strafe: circle band; walk-in target; back-off point | 6 to 16; 8 to 14; 16 |
| Slot 0 | melee, 0 < d < 16, within 45 deg, closes to 13 |
| Slot 1 | melee, 7 < d < 15, within 45 deg, closes to 12 |
| Slot 2 | melee, 0 < d < 10, within 45 deg, closes to 15 |
| Slot 3 | row 140: none; row 141: never chosen (chance 0) |
| Lost-sight search point | last seen + 6 |
| Wait-in-line square | 15 |
| Chase stop; confused strike | 3.17; 3.42 |

#### Death Knight

| Distance (units) | Value |
|---|---|
| Notice band (x0.7 standing, x1.75 running) | 1 to 10 (0.7 to 7; 1.75 to 17.5); sees through walls |
| Sight cone half-angle | 45 deg |
| Rush: hold band; walk-in target; back-off point | 5 to 15; 7 to 13; 17 |
| Slot 0 | melee, 0 < d < 15, within 45 deg, closes to 4 |
| Slot 1 | melee, 4 < d < 10, within 45 deg, closes to 4 |
| Slot 2 | melee, 10 < d < 20, within 45 deg, charge run 30 to 50 |
| Lost-sight search point | last seen |
| Wait-in-line square | 20 |
| Hover height above the hero | 0 (cap ground + 100) |
| Chase stop; confused strike | 2.67; 2.92 |

#### Ixi Spearman

| Distance (units) | Value |
|---|---|
| Notice band (x0.7 standing, x1.75 running) | 5 to 30 (3.5 to 21; 8.75 to 52.5) |
| Sight cone half-angle | 45 deg |
| Strafe: circle band; walk-in target; back-off point | 10 to 20; 12 to 18; 20 |
| Slot 0 | melee, 0 < d < 10, within 22.5 deg, closes to 4 |
| Slot 1 | never chosen (chance 0, no counter); band 7 < d < 12 |
| Slot 2 | melee, 0 < d < 30, within 15 deg, charge run 15 to 30 (ends early when d <= 5) |
| Lost-sight search point | last seen + 10 |
| Wait-in-line square | 10 |
| Chase stop; confused strike | 3.17; 3.42 |

#### Dark Ixi Spearman

| Distance (units) | Value |
|---|---|
| Notice band (x0.7 standing, x1.75 running) | 5 to 70 (3.5 to 49; 8.75 to 122.5) |
| Sight cone half-angle | 45 deg |
| Strafe: circle band; walk-in target; back-off point | 10 to 20; 12 to 18; 20 |
| Slot 0 | melee, 0 < d < 10, within 22.5 deg, closes to 4 |
| Slot 1 | never chosen (chance 0, no counter); band 7 < d < 12, charge run 0 to 0 |
| Slot 2 | melee, 0 < d < 50, within 15 deg, charge run 10 to 50 (ends early when d <= 5) |
| Lost-sight search point | last seen + 10 |
| Wait-in-line square | 10 |
| Chase stop; confused strike | 3.17; 3.42 |

#### Ixi Archer

| Distance (units) | Value |
|---|---|
| Notice band (x0.7 standing, x1.75 running) | 5 to 60 (3.5 to 42; 8.75 to 105); sees through walls |
| Sight cone half-angle | 22.5 deg |
| Keep distance: hold band; walk-in target; back-off point | 10 to 40; 16 to 34; 40 |
| Retreat: below; to point; stops at | none (0 to 0) |
| Slot 0 | missile, 10 < d < 40, within 22.5 deg |
| Slot 1 | melee, 2 < d < 12, within 45 deg, closes to 10 |
| Lost-sight search point | last seen + 10 |
| Map search | flag set, distance 0 and chance 0: inert |
| Chase stop; confused strike | 2.67; 2.92 |

#### Ixi Henchman (row 157)

| Distance (units) | Value |
|---|---|
| Notice band (x0.7 standing, x1.75 running) | 5 to 100 (3.5 to 70; 8.75 to 175) |
| Sight cone half-angle | 22.5 deg |
| Keep distance: hold band; walk-in target; back-off point | 10 to 40; 16 to 34; 40 |
| Retreat: below; to point; stops at | none (0 to 0) |
| Slot 0 | missile, 10 < d < 40, within 22.5 deg |
| Slot 1 | melee, 2 < d < 12, within 45 deg, closes to 10 |
| Slot 2 | missile, 15 < d < 80, in front |
| Lost-sight search point | last seen + 10 |
| Map search | flag set, distance 0 and chance 0: inert |
| Chase stop; confused strike | 2.67; 2.92 |

#### Dark Ixi Archer

| Distance (units) | Value |
|---|---|
| Notice band (x0.7 standing, x1.75 running) | 5 to 70 (3.5 to 49; 8.75 to 122.5) |
| Sight cone half-angle | 22.5 deg |
| Keep distance: hold band; walk-in target; back-off point | 10 to 40; 16 to 34; 40 |
| Retreat: below; to point; stops at | none (0 to 0) |
| Slot 1 | never chosen (chance 0); band 0 < d < 12 |
| Slot 2 | missile, 0 < d < 80, in front |
| Lost-sight search point | last seen + 10 |
| Map search | flag set, distance 0 and chance 0: inert |
| Chase stop; confused strike | 2.67; 2.92 |

#### Werelupe Basher

| Distance (units) | Value |
|---|---|
| Notice band (x0.7 standing, x1.75 running) | 1 to 100 (0.7 to 70; 1.75 to 175) |
| Sight cone half-angle | 22.5 deg |
| Rush: hold band; walk-in target; back-off point | 3.5 to 5; 3.8 to 4.7; 7 |
| Slot 0 | melee, 0 < d < 12, within 22.5 deg, closes to 5 |
| Slot 1 | melee, 20 < d < 50, within 15 deg, charge run 10 to 50 (ends early when d <= 5) |
| Slot 2 | never chosen (band minimum 20 above maximum 5) |
| Lost-sight search point | last seen |
| Wait-in-line square | 10 |
| Chase stop; confused strike | 2.35; 2.6 |

#### Werelupe Bone Thrower

| Distance (units) | Value |
|---|---|
| Notice band (x0.7 standing, x1.75 running) | 1 to 100 (0.7 to 70; 1.75 to 175) |
| Sight cone half-angle | 22.5 deg |
| Keep distance: hold band; walk-in target; back-off point | 10 to 50; 18 to 42; 50 |
| Retreat: below; to point; stops at | none (0 to 0) |
| Slot 0 | melee, 0 < d < 12, within 22.5 deg, closes to 5 |
| Slot 2 | missile, 10 < d < 75, in front |
| Lost-sight search point | last seen + 10 |
| Chase stop; confused strike | 2.35; 2.6 |

#### Evil Harris

| Distance (units) | Value |
|---|---|
| Notice band (x0.7 standing, x1.75 running) | 1 to 30 (0.7 to 21; 1.75 to 52.5) |
| Sight cone half-angle | 22.5 deg |
| Strafe: circle band; walk-in target; back-off point | 7 to 14; 8.4 to 12.6; 14 (backbiter 0: no normal choice while the hero faces it) |
| Slot 0 | melee, 0 < d < 15, within 22.5 deg, closes to 5 |
| Slot 1 | melee, 0 < d < 15, within 22.5 deg, closes to 5 |
| Lost-sight search point | last seen + 7 |
| Wait-in-line square | 10 |
| Chase stop; confused strike | 2.17; 2.42 |

**What a port most likely gets wrong (distances).**
1. Firing the Skull Spirit's single shot from the skull's centre. It leaves 7.9 up and 3.9 behind the skull and lives 31 units, so a standing hero beyond 28.75 is never hit although the slot is chosen out to 40.
2. Mistiming the Ixi lobs' fire patch. The Dark Ixi Archer's 300-tick area pulses first on the 61st tick after landing and then every 60 ticks (at most 4) while the hero is inside (SPEC-0029 Behavior 9, Area mode).
3. Testing shots against bone capsules, or sweeping them (body cylinder at the post-move positions only).
4. Treating the grab or the drain loop as weapon windows. The grab is a one-tick 60-degree disc test; the drain is a distance hold (at most 5.0) with direct damage.
5. Giving any enemy here a leash radius other than the Tentacles' distance hide.

## Test vectors

Each kind has its own test vectors inside its section. The cross-kind vectors are in SPEC-0064.

## Open questions

Each kind has its own open questions inside its section. The shared defaults are in SPEC-0064.
