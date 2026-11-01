# SPEC-0063: Arena fighters

- Answers: proactive (weapon box); proactive (arena fighter AI); R-0124; proactive (enemy claim check pass 3); proactive (timing pass); proactive (reach pass 2); proactive (enemy reach pass); proactive (movement pass); proactive (enemy sweep); proactive (open close-out)
- Status: Draft
- Disc build: SCUS-97367 v1.00 (same fingerprint as SPEC-0001)
- Confidence: High for the Rogue's row and clip values, the wake rule, the attack chooser, the counter and defence rules, the phase skips and the clip gate ticks. Medium-High for exact tick offsets (they depend on the request-then-animation order of SPEC-0038 Behavior 1 and 48). Medium for weapon contact geometry, facing while circling and the order of the intro hand-off. Rounds 1 to 4 (Ixi Henchman, Sludgy, Skeleton Fighter, Werelupe Elite) give their own confidence per section; their tick timelines come from rule models of the original behaviour and are Medium-High.

## Summary

The Meridell arena (world pack 1176) is a five-round ladder against five class-17 common enemies. Round r (0 to 4) loads the opponent from character region 1179 + r and plays it as entity 12 + r. The ring, the script, the win and lose paths are in SPEC-0041 (module 1176) and SPEC-0045 (the arena master in pack 1023); this spec adds what a rewrite needs to make each opponent fight like the original: how the script hands control to the AI, what each fighter does tick by tick, and the places where a port is most likely to go wrong.

The Rogue (round 0) is a class-17 enemy (SPEC-0038) with an unusual start: its spawn record gives it the "emerge and hide" default state, so it does not notice the hero. It sleeps at its home point until the hero comes within 40 units scaled by the hero's action state, then goes straight to attack mode. In attack mode it is a hair-trigger striker: three of its four attack slots are tested every tick with fixed percent rolls, one of them is a three-hit unblockable combo, one is a charge that steals Neopoints, and every hit it takes is answered by a counter attack. Its numbers are all in monster row 153 and in the animation table of character record 0 of region 1179; both are read from the disc.

The other four rounds use the same start: every opponent's spawn record has default state 8 (emerge and hide) with emerge distance 40.0, so none fights a hero standing at the arrival point (30.19 units away; the stance threshold is 28.0). Round 1 is an Ixi Henchman (row 160, 65 HP), round 2 is Sludgy (row 231, 80 HP), round 3 is a Skeleton Fighter (row 227, 75 HP) and round 4 is a Werelupe Elite (row 242, 110 HP). Each round's section has its own build guide, test vectors and open questions.

## Arena rules shared by all rounds

### Round selection

- The encounter thread starts at module init and picks the round once, from the highest rank flag set: 384 gives round 4, else 383 gives 3, else 382 gives 2, else 381 gives 1, else round 0. (Flag 385 is only a result of the last win; it is not tested here.) The opponent is entity 12 + r, loaded from character region 1179 + r into the third slot group (group index 2 + r). Regions 1177 and 1178 hold the spectators and the host (entity 11, Shanra) and load first. Details and call arguments: SPEC-0041, module 1176.
- Intro animation of the opponent by round: 70, 13, 70, 73, 68. Victory animation (hero-death cutscene): 71, 73, 73, 22, 73. Intro and victory lines: SPEC-0041.

### Start sequence (what the script does, in order)

1. The opponent is grabbed (base state 33, script hold). From that moment its think loop is not stepped and it cannot be hit (SPEC-0030, Script control and the entity's AI).
2. The hero is grabbed and moved to locator 0 (177.66, 27.92, 311.07). The host moves to locator 1. Camera 6 plays with a fade cut.
3. The opponent is moved to locator 2 (205.60, 27.92, 322.52), camera 4 plays and the opponent plays its intro animation to its end (wait). Camera 5 and hero animation 182 follow, then camera 7 and the round line.
4. After the last fade cut the script makes three calls in one step, in this order: release the opponent (base state 1), return to the normal camera with the release flag (hero put in action state 1), return the hero's animation to idle. The script step is the last item of a world tick (SPEC-0026, main loop step 8.1.12), after every character has been stepped. So the opponent's think loop first runs on the next tick, with the hero already in action state 1 and standing at locator 0.
5. Nothing else tells the AI to fight. The script makes no aggression call. What the AI does next is the opponent's own default state (for the Rogue: hidden at home, below).

### Ring and bounds

- The ring is a 16-sided floor of radius about 30 around (190.6, 316) at height 27.54. An invisible two-sided wall loop encloses it (SPEC-0041, Edge cases); the hero cannot leave. Opponents are kept on navigation mesh 0 (32 triangles, x 163.2 to 217.9, z 288.7 to 343.4). The hero's body radius is 1.175 (SPEC-0018).
- Each opponent has a home point, which is the position of its spawn record and does not change when the script moves it (SPEC-0040). The Rogue's home equals locator 2.

### What ends a round

- There is no timer, no ring-out and no flag from leaving the ring. A round ends only when the opponent is dead or the hero's death marker is set (SPEC-0041, death watcher). The opponent does not heal.
- Win: the watcher sees the opponent dead (the dead test is true from the tick the death hook runs), waits 1 second, then the death sequence helper fades out, moves the hero to locator 0, plays camera 5 and hero animation 182, shows the win line and loads pack 1023 with result 1 (SPEC-0041).
- Lose: the hero-death callback runs the hero's death loop and sets the marker 41 ticks later; the watcher then plays the hero-death cutscene (the opponent is grabbed, moved to locator 2 and plays its victory animation, SPEC-0041) and loads pack 1023 with result -1. Health is not changed by the arena script (SPEC-0041, CB3).
- Rank flags and hit points: pack 1023 sets the next rank flag on a win (flag 380 on a loss) and raises the hero to 50 percent of maximum hit points when below that (SPEC-0045, activate-world routine). Nothing refills the hero between rounds except that rule; the hero starts round r + 1 with the health it had at the end of round r (at least 50 percent).
- While the hero is dead or in a held or hurt state, common enemies pick no new attacks (SPEC-0038 Behavior 7). A strike that leaves the hero at 0 hit points sends the striker to the victory pose (SPEC-0038 Behavior 13).

### Rules that apply to every opponent

- All five are class-17 enemies and follow SPEC-0038 exactly (think loop, pre-step, sequencer, attack manager). Only their rows and tables differ. The attack manager has a single member, so no opponent ever waits in line (the first entry never waits).
- Hero damage to an opponent, hit reaction and the damage doubling rule: SPEC-0036 (Damage) and SPEC-0038 Behavior 37. Enemy damage to the hero (armour, shield, status scale): SPEC-0036, Enemy attack damage to a hero.

### Timing table (all five rounds; timing pass)

Confidence: High for the units, the start chains and the clip gates; Medium for the first-touch ticks (weapon-versus-body collision uses SPEC-0018; Tor idle and level, straight in front). Units, the attack phases and the rules for what counts down are those of the common-enemy timing table (SPEC-0064, Shared field behaviour: Timing table and units). One tick is 1/60 s; every opponent clip in these rounds has speed byte 2 (4 ticks per key at rate 1.0) unless its row says otherwise.

| Round | From the wake tick (tick 0) to agro | First possible attack choice | Main attacks: window opens, hit tick on a still hero in front, end path (ticks after the request s, or c8 for a charge strike) |
| --- | --- | --- | --- |
| 0 Rogue | no emerge clip: agro on tick 1, strafe from tick 2 | tick 3 | slot 3: s + 16 / 40 / 60, hits s + 17 / 41 / 61, end s + 82; slot 0: s + 20, hit s + 26, end s + 60; slot 2 bite: c0 + 36, hit c0 + 38, end c0 + 82 |
| 1 Ixi Henchman | emerge requested tick 1, agro tick 78 | tick 80 | jab: s + 20, no touch on a still level hero straight ahead at the 3.175-unit chase stop (weapon reach table below), end s + 58; lunge: c8 + 24, hit c8 + 28, end c8 + 86 |
| 2 Sludgy | emerge requested tick 1, agro tick 42 | tick 44 | swipe: s + 32, hit s + 36, end s + 82; slam: s + 19, hit s + 21 (3 to 4 units) or s + 22 (5 to 9), end s + 91 after a hit on s + 21, s + 90 after s + 22, s + 68 on a whiff; lob: release s + 84 |
| 3 Skeleton Fighter | emerge requested tick 1, agro tick 181 | tick 183 | slot 0: s + 36, hit s + 40, end s + 54; slot 1: s + 28, hit s + 29, end s + 66; lunge: s + 29 window; first touch of a still level hero straight ahead with the chase stop in force: s + 34 (1.5 units), s + 33 (2.0–2.5), s + 32 (3.0), s + 33 (3.5–8.0), s + 34 (8.5), s + 35 (9.0–9.75), then s + 36–44 to 13.0 (no touch at 1.0). Without the chase stop the same samples miss at 1.0–6.0 and touch at s + 33 (6.5), s + 32 (7.0), s + 33 (7.5–8.0), s + 34 (8.5), s + 35 (9.0–9.75). These are fixed-pose results for a hero who does not move, not a live chase timeline. |
| 4 Werelupe Elite | emerge requested tick 1, agro tick 105 | tick 107 | slot 0: s + 20, hit s + 20, end s + 58; slot 2: s + 28 / 52 / 76, hits s + 29 / 53 / 77, end s + 109; charge strike: c8 + 4, hit c8 + 4, end c8 + 26 |

**Exchange timeline (round 3, Tor holding a weapon, idle 20 units in front of the hidden Fighter, never guarding).** Tick numbers as in Round 3 Vector A.

| Tick | What happens |
| --- | --- |
| 0 | Wake test passes (20 is below 28); sub-state 1 |
| 1 | Emerge clip requested |
| 181 | Emerge gate; agro entry (cooldowns 0, recovery 0) |
| 183 | First chooser tick; d = 20, no slot in band |
| 225 | d = 9.75: slot 2 (lunge) chosen with 99 of 100 |
| 226 | Lunge clip requested (this pass at 1.0, later passes of state 2 at 0.7) |
| 255 | Window opens; the weapon does not reach Tor yet |
| 261 | First touching tick: 50 damage before armour; Tor is hurt in the same tick and his flinch clip (36 ticks) starts on tick 261; the pass of 261 still runs at 0.7 |
| 262 to 298 | The Fighter's think (which runs before Tor's update) sees Tor hurt: its cooldowns stop counting and it can choose nothing |
| 289 | Lunge gate (the passes after the contact tick run at 1.0) |
| 290 | End path: recovery int[15, 30]; slot 2 cooldown 180 drawn again |
| 298 | Tor's flinch done test passes in his update: back in state 1 |
| 299 | The Fighter sees Tor attackable again; its cooldowns resume |
| 305 to 320 | The recovery has run out; the next choice is possible (slot 1 or 0) (Tor is attackable again from 299) |
| h | Tor presses a normal swing (sub-state 2) with the Fighter within 8 and in front |
| h + 1 | The Fighter sees the swing edge and rolls once: block 35 of 100 (it then guards until the swing ends) |
| h + 8 to h + 23 | Tor's swing window: unblocked, the first touching tick damages the Fighter, restarts its reaction clip 57 (65 ticks) and cancels any swing it was making |

**What a port most likely gets wrong in the arena (timing).**
1. Hitting on the window start. Every round has a later first-touch tick (table above); the Henchman's jab may not touch a still hero at all.
2. Starting the round on release. Each opponent waits for the 40 x F1 wake test and then plays its emerge clip (77, 41, 180 or 104 ticks; none for the Rogue).
3. Letting cooldowns run while Tor is hurt (36 ticks with a weapon, 40 unarmed) or knocked down: they stop; only the recovery countdown runs.
4. Keeping the strike word after contact: the lunge and the slam play at 1.0 from the tick after contact, so their end path moves with the hit tick.
5. Giving the hero invulnerability after a hit: a three-hit swing can land all three hits on a hero who is already hurt.

## Round 0: the Rogue

### Identity and tunables

| Item | Value |
|---|---|
| Fighter | The Rogue: character record 0 of region 1179 (the only record in that region), monster type 72 variant 1, monster row 153 |
| Class and approach | Class 17; approach routine 4 (strafe) |
| Hit points | 80 |
| Body | Row radius 2.0, height 2.0, weight 5.5; difficulty value 8; target type (alignment) 0, so among Motes only Nova (kind 7) gets the type-match bonus (SPEC-0036) |
| Hurt shape | One capsule on the root bone: radius 1.05, length 3.04, base 0.82 below the root joint (about 0.76 to 3.80 above his feet in the rest pose). He has no other hurt capsule, so every swing, shot and weapon test is against this one body |
| Spawn record | Default-state word 8 (emerge and hide); emerge distance 40.0; emerge selector 0; alarm group none (255); home (205.60, 27.92, 322.52); facing about (-0.93, -0.06, -0.36), which is within 3 degrees of the direction to the hero's start |
| Display in default | 1 (visible); not an invisible kind |
| Death | Death disappear timer 600; no drop placement (ordinal -1); carried Neopoints 0 |
| Notice values | View cosine 0.92388, awareness 75, timer 30, chance 75 at range 10 falling to 10 at range 60. Never used in the arena: the notice check runs only in states 2, 3, 5, 6, 7 and 9 and the Rogue is never in them (it is in state 8, 17 or 19) |
| Map search | Enabled: distance 100, chance 50, timers 15 to 120, type 1 (coins), greedy range 10.0 |
| Strafe band | Desired 6 to 20; backbiter 1.0 (no change to chances); turn speed 0.9 |
| Recovery after an attack | int[15, 30] ticks (row timer 30 max, 15 min) |
| Missile defence | Hold 60 ticks; dodge chance 75; block chance 0 |
| Melee defence | Block chances 0, 0, 0 (quick, normal, heavy); dodge chances 50, 25, 25; defence timer 0 to 0 |
| Other | Blocking Angle 0 (he never guards), Damage Reduction 0, steal enabled on slot 2 only, knockback 0 on every slot, no projectiles |

Attack slots (slot 1 is unused: type 0xFFFF). Ids are the sequencer's phase ids (SPEC-0038 Behavior 13); "none" means the main table has no clip for the id, so the sequencer skips the phase in the same tick.

| Slot | Kind | Distance band (strict) | Chance | Facing arc (dot above) | Melee range | Delay (cooldown) | Charge | Damage | Flags | Counter |
|---|---|---|---|---|---|---|---|---|---|---|
| 0 | melee | 0 to 10 | 50 | 0.92388 | 5.0 | int[30, 60] | none | 6 | none (blockable) | trigger 6 (own hit reaction), chance 25 |
| 2 | melee | 5 to 20 | 50 | 0.92388 | 5.0 | int[60, 120] | kind 2, distance float[15, 25) | 1 | steal (Neopoints, 10 to 100, chance 50) | none (trigger -1) |
| 3 | melee | 5 to 15 | 25 | 0.0 | 5.0 | 0 to 0 | none | 10 | unblockable | trigger 6, chance 25 |

All wind-up, strike and recovery speed words are 1.0, so every clip plays at its natural rate: speed byte 2 gives 0.25 keys per tick, 4 ticks per key.

| Slot | Phase ids and their clips |
|---|---|
| 0 | wind-up 33 none; strike 34 = clip 8 (15 keys, loops, strike bit on keys 5 to 12); follow-through 35 none; blocked 36 none; recovery 37 none |
| 2 | wind-up 43 none; run 44 = clip 40 (10 keys, loops, no strike bits); strike 45 = clip 39 (21 keys, strike bit on keys 9 and 10); blocked 46 = clip 11 (never played: kind 2 has no blocked state); recovery 47 none |
| 3 | wind-up 48 none; strike 49 = clip 38 (21 keys, strike bit on keys 4 to 7, 10 to 12 and 15 to 17); follow-through 50 none; blocked 51 none; recovery 52 none |

Weapon volumes (finite cylinders on attachment point 0, the right hand): weapon 1 radius 6.0, full height 1.0; weapon 2 radius 5.0, height 1.0; weapon 3 radius 5.0, height 3.0. Every strike bit used by the Rogue's clips is bit 0x1, so weapon 1 (radius 6.0, height 1.0) is the only volume that ever strikes.

Animation ids the AI uses and what the Rogue's table gives them:

| Id | Use | Clip |
|---|---|---|
| 0, 7 | idle pick base | 0 (25 keys, loops) |
| 9 | hide pick (state 8) | none: a direct request of a missing id starts clip 0 |
| 10 | emerge | none, so the emerge step is skipped |
| 28 | gait 0 (walk, run, charge) | clip 40 (10 keys, loops; root travel 7.27 units per 40 ticks, 10.9 units per second) |
| 32 | gait 3 (back step) | clip 4 (12 keys, one-shot; 4.6 units back over its first 44 ticks, then held: twice its last key every tick, 12.5 units per second back and 3.1 sideways, for as long as id 32 stays requested; Movement rules) |
| 29 | gait 6 slot word 6 (right circle) | clip 20 (8 keys, loops, 3.45 units sideways per 32 ticks) |
| 30 | gait 5 slot word 5 (left circle) | clip 21 (8 keys, loops, 3.51 units per 32 ticks) |
| 31 | strafe hold | clip 5 (23 keys, one-shot: plays once, then holds its last pose) |
| 53, 54 | missile dodge | clips 16 and 17 (13 and 12 keys, one-shot, 10.6 units sideways) |
| 55 | melee dodge | clip 15 (20 keys, one-shot, 10.04 units backward) |
| 56 | block | none (never used: block chances are 0) |
| 57 | hit reaction kind 0 | clip 3 (14 keys, one-shot) |
| 58 | death | clip 24 (19 keys, one-shot) |
| 70, 71, 73 | intro, victory, victory pose | clips 35 (38 keys), 12 (28 keys), 12 |
| 115 to 119 | idle fidgets | all present (clips 26, 34, 29, 13, 19) |

### Arena start and wake-up

The Rogue is released in class state 8, sub-state 0 (hidden). The rules, in order, each tick of its think loop:

1. State 8, sub-state 0: if the hero's distance to the Rogue is below 40.0 times the factor F1 of the hero's action state (SPEC-0038 Behavior 18), the sub-state becomes 1. The factor is 0.7 in action state 1 (standing, the state after the intro), 0.75 in states 2 and 5, 1.5 in states 3 and 9, 1.75 in states 4, 6 and 10, 0.5 in states 21, 22 and 41, 0.3 in state 23 and 1.0 in every other state. So the wake distances are 28 (standing), 30 (slow walk), 60 (walk), 70 (run) and 40 (any state not listed, for example while attacking or held).
2. Sub-state 1 (the next tick): the emerge animation is id 10 for selector 0. The Rogue's table has no clip for id 10, so the step does not play anything and sends the Rogue to agro (class state 19) at once. There is no emerge animation, no search animation and no notice roll.
3. While hidden (sub-state 0), the Rogue stands at its home point playing the hide pick (id 9, which has no clip, so clip 0, the idle loop). If it is more than 0.35 away from home it walks home first (gait 0). He is hittable while hidden (his spawn word +0xC4 is 1, which disables the hide-pick visibility rule of SPEC-0036).
4. The hero stands at locator 0, 30.19 units from home. A hero who stands still there does not wake him (28 < 30.19). Any walk or run wakes him on the next tick (60 and 70 exceed every distance inside the ring, whose farthest point from home is about 46). A hero who walks slowly (state 2) must come within 30.
5. A hit on a hidden or otherwise non-agro Rogue wakes him at once: the take-hit hook sets agro on the hit and doubles that hit's damage (SPEC-0038 Behavior 37 step 6).
6. Entering agro (setter for state 19): outer sub-state 0, defence hold 0, recovery countdown 0, melee-swing latch 0, all four slot cooldowns 0, attack manager entry. The combat timer drawn there (int[180, 300]) is stored and never read by the agro code.

### Per-tick order (agro)

Every think tick of class state 19 (SPEC-0038 Behavior 44), in this order:

1. The pre-step (Behavior 2): awareness countdown, alert flag, retarget countdown, alarm check, map search (see below), then the agro machine.
2. Agro machine: recovery countdown down by 1 when positive; guard-state reset; missile defence; melee defence; lost-sight test; normal attack choice; counter-attack choice; distance-hide gate (does nothing: flag 0); the outer sub-state switch; the agro tail (leash).
3. The class-19 handler: defence hold down by 1; in the hold inner state, face the hero (blend 0.9) and request id 31; in outer 0 request the idle pick.
4. The animation pass, then the motion pass (one-tick request-to-step latency, SPEC-0038 Behavior 48).

Because the choice runs before the outer switch in the same tick, a slot picked in tick c runs its first sequencer state in tick c.

### Attack chooser

Runs each tick before the outer switch (SPEC-0038 Behavior 7), only while the hero is attackable (not dead, hurt, held or knocked down; otherwise nothing happens, even the cooldowns do not count down).

1. Each slot cooldown goes down by 1 when positive.
2. If the outer sub-state is 2 (an attack running), stop. Otherwise the selected slot is cleared.
3. Continue only in outer sub-states 1, 3, 4 and 11 and only while the recovery countdown is below 1. The Rogue's movement state is outer 4, so this means: not while an attack, a dodge, a hit reaction or the recovery runs.
4. Test slots in the order 3, 2, 0 (slot 1 is unused). The first slot that passes all of these is chosen:
   - its cooldown is below 1;
   - the distance d to the hero satisfies the band strictly (slot 3: 5 < d < 15; slot 2: 5 < d < 20; slot 0: 0 < d < 10);
   - the dot product of the Rogue's facing and the unit direction to the hero is above the slot's arc (slot 3: above 0.0, the hero is anywhere in front; slots 0 and 2: above 0.92388, within 22.5 degrees of dead ahead);
   - a roll int[1, 100] is below the chance. Slot 3: below 25 (24 of 100); slots 2 and 0: below 50 (49 of 100). The backbiter factor is 1.0, so a hero behind him changes nothing;
   - a route to the hero exists (slot 2), or, for slots 0 and 3, a route exists or the distance is below 10.0.
   One roll is drawn for each slot whose cooldown, band and arc pass, in the order above, and the first success ends the test.
5. On a choice: the slot's cooldown is drawn (slot 0: int[30, 60]; slot 2: int[60, 120]; slot 3: 0), the sequencer starts in state 0 (slots 0 and 3) or state 6 (slot 2), and the desired distance (6 + 14 x int[20, 80] / 100) and charge distance are drawn again.
6. Probability that an attack starts in a tick when all cooldowns are 0, recovery is 0 and the hero is in front: d between 5 and 10: 1 - 0.76 x 0.51 x 0.51 = about 80 percent per tick (slot 3 first at 24 percent, slot 2 next, slot 0 last); d between 10 and 15: 1 - 0.76 x 0.51 = about 61 percent; d between 15 and 20: 49 percent (slot 2 only); d above 20 or inside 5 (slot 0 only): 0 or 49 percent. So the Rogue attacks within a few ticks of becoming eligible; the real limits are the cooldowns and the recovery countdown.

### State table

Ticks are relative to the tick s in which the state requests its clip; "gate" is the first tick in which the phase's play-to-end check passes (progress at least 0.99; SPEC-0005, Clip progress).

| State | Action id and clip | Keys | Speed | Gate and exits |
|---|---|---|---|---|
| Hidden (class 8, sub 0) | 9 -> clip 0 | 25, loops | 1.0 | Wake rule above; leaves on the next tick |
| Emerge (class 8, sub 1) | 10 -> skipped | none | none | Same tick: agro (class 19) |
| Agro set-up (outer 0) | idle pick: 0 -> clip 0, or a fidget 115 to 119 after int[120, 600] ticks | 25 | 1.0 | One tick, then outer 4 (strafe) with inner 10 and hold 0 |
| Strafe hold (outer 4, inner 10) | 31 -> clip 5 | 23, one-shot | 1.0 | Hold timer int[60, 180]; distance rules below |
| Strafe approach (inner 7) | 28 -> clip 40 | 10, loops | 1.0 | Ends when d is below the desired distance |
| Circle left / right (inner 8, 9) | 30 -> clip 21 / 29 -> clip 20 | 8, loops | 1.0 | Hold timer; d above 20 gives inner 7, d below 6 gives inner 11 |
| Back off (inner 11) | 32 -> clip 4 | 12, one-shot | 1.0 | Ends when d is above the desired distance (the one-shot clip finishes after 44 ticks and is then held at twice its last key, so he keeps backing away; Movement rules) |
| Slot 3 swing (outer 2) | 49 -> clip 38 | 21, one-shot | 1.0 | Windows open s+16, s+40, s+60; hits on a hero standing in front at s+17, s+41, s+61; gate s+81; end path s+82 |
| Slot 0 swing | 34 -> clip 8 | 15, loops | 1.0 | Window opens s+20; one hit, on a hero standing in front at s+26; gate s+59; end path s+60 |
| Slot 2 charge run (sequencer 7) | 44 -> clip 40 | 10, loops | 1.0 | Until the hero is within 5.0, or the charge distance is travelled |
| Slot 2 strike (sequencer 8) | 45 -> clip 39 | 21, one-shot | 1.0 | Window opens c0+36; hit on a hero 4.5 to 6.0 units ahead at c0+38 (c0+39 at 3.5 units or closer, c0+40 from 6.5); gate c0+81; end path c0+82 (c0 = tick the strike clip is requested) |
| Melee dodge (outer 5) | 55 -> clip 15 | 20, one-shot | 1.0 | Gate s+77, then the defence exit |
| Missile dodge (outer 6) | 53 -> clip 16 or 54 -> clip 17 (int[0, 1]) | 13 or 12 | 1.0 | Gate s+49 or s+45, then the defence exit |
| Hit reaction kind 0 (outer 10) | 57 -> clip 3 | 14, one-shot | 1.0 | Gate s+53, then outer 0 |
| Victory pose (class 10) | 73 -> clip 12 | 28 | 1.0 | Entered when his strike leaves the hero at 0 hit points; at the gate runs the default-state step (back to hidden) |
| Coin fetch (class 17) | 28 -> clip 40 | 10, loops | 1.0 | See the coin section |
| Dead (base state 16) | 58 -> clip 24 | 19, one-shot | 1.0 | Plays once; the 600-tick disappear countdown starts when it reaches progress 0.95 |

Defence exit: outer 0, base state 1, defence hold int[0, 0] = 0.

### Movement rules

- Wake and strafe entry: see above. From the third agro tick the Rogue is in outer 4, inner 10 (hold) with hold 0.
- Inner 10: the hold timer goes down by 1 each tick. While it is above 0 the Rogue stands still, faces the hero (blend 0.9) and plays id 31; if d > 20 it switches to inner 7, and if d < 6 and the strafe lock is 0 it switches to inner 11. When the timer is 0 or below: r = int[1, 3]: 1 stays in hold, 2 starts a left circle, 3 a right circle (a circle is refused while the lock is above 0), then the timer is redrawn as int[60, 180] and the distance tests run.
- Inner 8 and 9 (circle): gait 6 or 5, steering to a point 2 units to his left or right of his own position; the timer goes down by 1 and at 0 the state returns to inner 10; a wall contact ends it and sets the lock to 120 ticks; the same distance tests apply. Circling moves him sideways at about 6.5 units per second, so over a long circle the distance to a still hero grows.
- Inner 7 (approach): gait 0, steering at the hero, until d is below the desired distance (6 + 14 x int[20, 80] / 100, so 8.8 to 17.2, redrawn after every attack choice). He starts approaching only when d exceeds 20.
- Inner 11 (back off): gait 3, steering to a point behind him; ends when d exceeds the desired distance. Corrected (movement pass; High for the rule, Medium-High for how long a back-off lasts): id 32 is the one-shot clip 4 (12 keys), which is not restarted while id 32 stays requested. For its first 44 ticks it moves him 0.1043 per tick backward (4.6 units); from then on the clip is finished and held, and a held one-shot's root step is twice its last key's every tick (SPEC-0005, Layer step), so he keeps backing away at 0.2086 per tick (12.5 u/s) with 0.052 per tick (3.1 u/s) sideways for as long as the back-off lasts. He does not stand still. The clip restarts only after another id has played in between (the next back-off after a hold or a circle starts fresh).
- So inside the 6 to 20 band the Rogue does not close in by himself; he holds, circles, and attacks by the chooser. The sequencer does the closing.
- Sequencer state 0 (slots 0 and 3): while d is above the melee range 5.0, gait 0 toward the hero (clip 40, about 10.9 units per second, 0.18 per tick); the step is dropped on any tick where the distance is within the sum of the two body radii (2.0 + the hero's 1.175). When d is 5.0 or less the state becomes 1 on the next tick. Turning is the steering blend only.
- The hero outruns him: the hero walks at 13.5 and runs at 19.3 units per second (SPEC-0033); the Rogue's 10.9 is slower. His charge (clip 40 again) is no faster.
- Strike lunge: the swing clips carry forward root motion (clip 38 3.84 units, clip 8 2.59 per 60 ticks, clip 39 6.56 units) so he closes a little during a swing. The chase-stop rule applies only in sequencer states 0 to 3, not in the charge states 7 and 8.
- Knockback on the hero is 0 on every slot; a hit never pushes the hero.
- Body push (movement pass): the steal charge (slot 2) is charge kind 2, so the Rogue's body stays solid through its run and its hit clip (SPEC-0065, Movement rules for every kind, item 10). The run stops at the 5.0 range test, before the bodies touch (2.0 + 1.175 = 3.175), but the hit clip C (clip 39) carries him 6.56 units further forward over 84 ticks, so a hero who stands still in front of him is shoved ahead by the body during the swing.

Movement table (movement pass; row 153; speed byte 2; 60 ticks per second):

| Mode | Source | u/t | u/s | Turn | Clip (id: clip, keys, loop) | Exits |
|---|---|---|---|---|---|---|
| Hidden at home (state 8) | none | 0 | 0 | none | 9 | wake |
| Decloak (alarm or wake, id 82) | root forward | 11.10 units over the clip | 11.1 | none | 82: clip 36, 15, one-shot | gate |
| Alert (id 18) | root forward | 4.21 units over the clip | 3.2 | none | 18: clip 32, 20, one-shot | gate |
| Strafe approach (inner 7), sequencer approach, charge run | root | 0.1817 average; per key 0.142 to 0.242 | 10.90 | steering 0.3 / 0.1 (the run does not steer) | 28, 44: clip 40, 10, loop | d below desired (8.8 to 17.2), the 5.0 range, or the charge distance 15 to 25 |
| Circle left / right | root sideways | 30: 0.1096 (-x); 29: 0.1079 (+x) | 6.58; 6.47 | faces the hero | 30: clip 21; 29: clip 20; 8, loop | hold count, wall, distance tests |
| Back off (inner 11) | root backward, one-shot then held | 0.1043 for 44 ticks, then 0.2086 (and 0.052 sideways) | 6.26, then 12.5 | faces the hero | 32: clip 4, 12, one-shot | d above desired |
| Strafe hold (inner 10) | none | 0 | 0 | aim 0.9 | 31 | hold count, distance tests |
| Dodges 53, 54; 55 | root | 10.6 sideways (49 ticks); 10.04 backward (77 ticks) | - | none | clips 16, 17; 15 | gate |
| Hit reaction kind 0 | root backward | 2.84 units over the clip's 56 ticks (gate 53) | 3.04 | snapped to the attacker | 57: clip 3, 14 | gate |
| Death | none | 0 | 0 | none | 58: clip 24 | removal |

Worked example (back-off). Hero walks up to 4 units from a holding Rogue whose desired distance is 14: inner 11 starts; ticks 1 to 44 of the clip carry him 4.6 units back (d = 8.6 if the hero stops), and from then on 0.2086 per tick: the remaining 5.4 units take 26 ticks, so he reaches d 14 about 71 ticks after the back-off began, with about 1.4 units of sideways drift.

### Hit and hurt windows

Strike test (SPEC-0038 Behavior 13, Strike test): each tick in sequencer state 2 (and in state 8 for the charge) the Rogue tests weapon 1 against the hero's body capsules whenever the playing clip's current key has the strike bit. One hit per window is added to the hero's struck list; the list is cleared when a new window re-enters state 2.

| Attack | Strike windows (AI ticks after the request tick s) | Hits | Damage per hit (before the hero's armour and shield) |
|---|---|---|---|
| Slot 3 (clip 38) | s+16 to s+31, s+40 to s+51, s+60 to s+71 | on a hero standing 1.5 to 9.0 units in front: s+17, s+41, s+61 (from 9.5 to 10.5 units the first blow misses and the other two land on s+41 and s+61; at 11.0 to 11.5 units one blow lands, on s+62). The weapon already touches such a hero on s+40 and s+60, but after each hit the sequencer is in its follow-through state and re-enters the strike state only one tick after the next window opens, so the second and third blows are first tested on s+41 and s+61. A blow whose predecessor missed is tested while the strike state is still running and lands on its touching tick | 10 each, unblockable |
| Slot 0 (clip 8) | s+20 to s+51 | one hit, on a level hero standing straight ahead at s+26 from 3.0 to 4.5 units, s+27 at 5.0 | 6, blockable |
| Slot 2 (clip 39, c0 = request tick) | c0+36 to c0+43 | one hit, at c0+38 for a hero 4.5 to 6.0 units ahead (the run stops at 5.0 or less, so this is the usual case), c0+39 at 3.5 units or closer, c0+40 from 6.5 to 12.0; a still level hero at exactly 4.0 is not touched | 1, blockable, steal roll |

- The window start is not the hit tick: a strike lands on the first window tick when weapon 1 (radius 6.0, full height 1.0, on the right hand) passes SPEC-0018's asymmetric weapon-versus-body test against a hero body capsule (SPEC-0040 Behavior 12A). These hit ticks are for Tor idle and level, straight in front; with the chase stop, slot 0 first touches at s+26 from 3.0 to 4.5 units and s+27 at 5.0. The exact rule and its geometric limits are in SPEC-0064; Medium. A hero standing still in front of him is struck in every window.
- The hurt side: he can be struck while hidden, walking, circling, holding, attacking, recovering, in hit reaction, or fetching coins. He cannot be struck during a melee dodge or a missile dodge (outer sub-states 5 and 6), while dead, or while script-held. A hit he takes while attacking cancels the attack (see counters).
- A hero who is hurt, knocked down or dead is not attackable: the Rogue picks no new attack and slot cooldowns stop, but an attack already running keeps going and keeps landing (there is no hurt invulnerability on the hero, so a three-hit swing can land all three hits).
- Damage on the hero: SPEC-0036 (Enemy attack damage to a hero). With no armour, slot 3 deals 10 per hit and slot 0 deals 6. Slot 2's base damage of 1.0 becomes 0 after any body armour (the result is truncated, with no minimum), and a steal needs a strike result of at least 1 (SPEC-0029 Behavior 11a), so heroes wearing armour are never robbed by the Rogue. A strike whose damage comes to 0 does nothing at all: no take-hit, no hit effect and no hit-list entry.

### Weapon reach

Conventions are those of "Weapon reach tables" in SPEC-0064 (shared field behaviour). Medium. The hit ticks above are authoritative (they are for a hero straight in front at the start distance); the level-contact column is the first tick on which any position of an idle level hero is touched. Weapon 1 has radius 6.0 and height 1.0, on the right hand. "sC" is the tick the steal clip is requested.

| Slot (phase, clip) | Window ticks (level contact) | Centre forward / up / side | Tor ahead, at the tick | Tor ahead, from the start | Tor feet vs origin |
|---|---|---|---|---|---|
| 0 (B, clip 8) | s+20 to s+51 (s+20 to s+51) | -0.4 to 2.8 / 0.8 to 1.5 / -2.0 to -0.7 | -1.8 to 4.0 | -1.1 to 6.4 | -8.0 to 7.0 |
| 2 steal (C, clip 39) | sC+36 to sC+43 (sC+38 to sC+43) | -0.3 to 3.6 / 2.4 to 6.7 / -3.2 to 1.5 | -2.4 to 9.0 | 0.8 to 12.4 | -6.5 to 12.0 |
| 3 first blow (B, clip 38) | s+16 to s+31 (s+16 to s+31) | 0.2 to 2.1 / 2.3 to 2.9 / -1.5 to 1.8 | -4.2 to 8.6 | -3.5 to 9.4 | -6.0 to 8.5 |
| 3 second blow (B, clip 38) | s+40 to s+51 (s+40 to s+51) | -1.6 to 2.2 / 1.8 to 2.2 / -2.4 to -0.5 | -3.8 to 8.4 | -1.6 to 10.6 | -7.0 to 7.5 |
| 3 third blow (B, clip 38) | s+60 to s+71 (s+60 to s+71) | 0.9 to 2.0 / 1.9 to 2.0 / -1.7 to 1.2 | -4.4 to 8.2 | -1.0 to 11.7 | -7.0 to 8.0 |

For slot 3 the level-contact column starts at s+16, while the first touch of an idle hero straight in front at the start distance is s+17. In the second and third rows the level-contact start (s+40, s+60) is the tick on which the weapon first touches such a hero; the blow lands one tick later (s+41, s+61) whenever the previous blow hit, because the sequencer re-enters its strike state one tick after each window opens (SPEC-0038 Behavior 13, State 3). The three-hit swing touches a level hero up to 8.6 ahead at the tick and the steal swing up to 9.0; slot 0 is short (4.0 ahead at most). Slots without a row have no weapon test.

### Defence and counter rules

Melee defence (decided once per hero swing):
- Latch: the hero's swing query is true while Tor is in the attack state with attack sub-state 2 to 12, or in a jump state (7 to 11) in air-attack phase 8 or 9, and while Roberta is in the attack state with sub-state 5 or 12. The Rogue evaluates on the first of his ticks on which the query is true (hero update runs after his step, so that is the tick after the swing begins, before the window opens) and not again until the query has been false.
- Evaluation only when the defence hold is below 1, he is not in hit reaction, and he is not in the middle of an attack (sequencer states 0, 1 and 5 are open to it, states 2, 3, 4 and 6 to 15 are not), and when the hero is closer than 8.0 units and the hero's facing dotted with the negated offset is above 0.5 (inside a 60 degree half-angle).
- Two rolls int[1, 100]. The swing class is quick, normal or heavy. Tor: sub-states 5 and 6 quick; 2, 3 and 9 normal; 4, 7, 8 and 12 heavy; 10 (power step) and any other value have no class and never trigger a defence. Roberta: sub-states 5, 6, 7 and 10 quick, 2, 3, 8, 11 and 12 normal, 4 heavy. The block roll uses chance 0 and never passes. The dodge passes when the second roll is below 50 (quick: 49 of 100) or below 25 (normal and heavy: 24 of 100).
- A dodge sets outer sub-state 5 and the defence hold to 60 and plays id 55 the same tick. He is unhittable from then until the gate 77 ticks later; clip 15 carries him backward about 10 units, mostly between ticks 16 and 40. After the gate: outer 0, then the strafe entry on the next tick.
- The Rogue never blocks: block chances are 0 and he has no block clip. He is therefore never a guarding target, so the shared rule that a swing on a guarding target from behind does nothing (SPEC-0036, Hit detection step 4) never applies to him: every swing that reaches his capsule hits, from any side.
- Differences from the other arena fighters' shared pattern: he shares the hidden start, the distance 40.0 and the doubled first hit, but he has no emerge clip (id 10 is missing), so he goes to agro with no emerge animation and a hit does not skip anything. Missing slot phases are skipped, every hit restarts reaction 57 (and the counter roll can cancel it), and defence is rolled once per hero swing, as stated above.

Missile defence (Roberta's shots): not in outer 10, not during the recovery state 5 of the sequencer, hold below 1. For each live hero shot (up to six), the path from the shot's position to its position plus its velocity times 60 ticks is swept against his body capsule. On contact, a roll int[1, 100] below 75 (74 of 100) sets outer 6 and the hold to 60; the block roll (chance 0) never passes. The dodge plays id 53 or 54 (clip 16 or 17), is unhittable, and ends at the gate (s+49 or s+45).

Hit reactions: every hit that reaches him (the take-hit hook, SPEC-0038 Behavior 37) sets outer sub-state 10 with kind 0 (id 57) and restarts the playing clip from its first key on the next hookup. He is not knocked back. The reaction ends at the 57 gate, 53 ticks after it starts, then outer 0. A new hit during the reaction restarts it. If the Rogue was not in agro (hidden, coin fetch, victory pose) the hit's damage is doubled and he goes to agro (this applies once, on that hit).

Counter attack (trigger 6, chance 25; slots 0 and 3):
- Runs every agro tick, with no hero-attackable gate and no outer-state gate. While his outer sub-state is 10 it is eligible each tick: a slot is eligible when its cooldown is below 1, d is inside its band and the facing test passes. Slot 3 is eligible for 5 < d < 15 with the hero anywhere in front; slot 0 for d < 10 with the hero within 22.5 degrees of dead ahead and its cooldown at 0. The later eligible slot wins (slot 3 over slot 0).
- If a slot is eligible, one roll int[0, 100] per tick (101 values): below 25 starts that slot's attack at once (sequencer state 0, outer 2), replacing the reaction. So each tick of the 53-tick reaction has a 24.75 percent chance (25 of 101) to turn into a counter attack; the chance that no counter happens in a whole reaction is under one in a million when a slot is eligible. Expected delay about 4 ticks.
- Practical meaning: the Rogue is almost always countering a hit when the hero hits him from 5 to 15 units (slot 3: the full 10-damage three-hit combo) or from inside 5 when slot 0's cooldown has expired. He is only staggered for the full 53 ticks when no slot is eligible: the hero is inside 5 (slot 3 needs more than 5) and slot 0's cooldown (30 to 60 ticks after each use, set at the choice and again at the end of the swing) has not run out, or the hero is behind him.
- The counter does not reset the shared recovery or other slots' cooldowns; the chosen slot's cooldown is redrawn.

### Stealing and the coin detour

- Slot 2's strike has a steal chance of 50 (50 of 101: a roll int[0, 100] below 50) after a damaging strike result, an amount int[10, 100], limited to the party's Neopoints, and the coins (SPEC-0021 split) appear at the hero's position (SPEC-0029 Behavior 11a). The Rogue keeps nothing.
- Map search, type 1 (coins): from the pre-step, every tick, in any state except the confused outer 11, the attack state 18 and while the map-search cooldown is above 0 (always 0 for him), the Rogue looks for the nearest world item of each coin kind (ids 135 to 139, in that order, anywhere in the world pool, no range limit). For each kind that exists: a roll int[1, 100] below 50 and a route to it and the hero farther than 10.0 from him set his state to 17 with that coin as target (the last passing kind wins). There is no cooldown after a successful or failed try for this type.
- State 17: if the hero comes within 10.0 of him he gives up at once; otherwise he runs to the coin (clip 40) and removes it from the world on arrival within 1.0 unit. The coin is simply destroyed (the Rogue does not gain Neopoints). When no target remains he resets: class state 0, which the pre-step turns into the default state 8 - he walks back home (gait 0, 10.9 units per second) and hides again until the hero is within 40 x F1 of home.
- Result: after a successful steal, when the hero backs off beyond 10 units, the Rogue breaks off the fight, collects the coins, walks home and falls back to the wake rule. A hit on him while he fetches coins or walks home doubles that hit and returns him to agro. The hero can pick the coins up himself first (SPEC-0021, Pickup).

### Hero actions against this fighter

| Hero action | What the Rogue does |
|---|---|
| Stands still beyond the wake distance | Nothing: hidden at home, idle loop |
| Walks or runs (any state with F1 of 0.75 or more), or comes inside 28 | Wakes (sub-state 1 next tick), agro two ticks after the wake test, strafe entry on the third, first possible attack choice on the fourth |
| Tor quick swing (sub-states 5, 6) | One defence evaluation at the swing's start; dodge 49 of 100 if the hero is within 8.0 and inside 60 degrees; otherwise the swing can hit |
| Tor normal swing (2, 3, 9) or heavy swing (4, 7, 8, 12) | Same evaluation, dodge 24 of 100 |
| Tor power step (10) | Never dodged; damage uses the attack power table and the Mote's status request; a Mote only adds its type bonus when it is Nova (the Rogue's target type is 0) |
| Tor air slash (jump states) | Counted as a swing for the latch; the class comes from the same word, so it dodges only when that word is in a class |
| Roberta's near swat (sub-state 5) and sub-state 12 | Same evaluation as Tor's, quick class |
| Roberta's shots | Missile defence: 74 of 100 to dodge when the path will touch his body within 60 ticks; otherwise the shot hits and he reacts |
| Mote power-attack objects | Tested against his body cylinder (radius 2.0, height 2.0); a hit takes damage and reacts like any hit |
| A landed hit | Damage by SPEC-0036 (no defence value); doubled when not in agro; reaction kind 0; counter chance each tick if a slot is eligible |
| Hero guard (L1 shield) | Does not change his AI. It blocks slots 0 and 2 when the hero faces him (dot at most -0.5) and does nothing against slot 3 (unblockable) |
| Hero jump | He has no special case. Contact depends on the weapon volume and the hero's capsules; a jumping hero is still the target of every distance test (flat ring) |
| Hero roll | The hero has no roll action |
| Hero hurt, knocked down or dead | He chooses no new attack and the cooldowns stop; a swing already running continues |
| Hero swap (Tor and Roberta) | From the next tick every test uses the new hero (position, action state, swing query) |

### Edge cases for implementers

- The wake test and the emerge step use the hero's distance on the ground plane; the height difference is dropped, so an airborne hero counts as standing on the spot. Resolved (High): the shared distance helper that every common-enemy test calls (wake, attack choice, band, hold and exit tests) zeroes the height lane before it takes the length; only the chase stop and the weapon tests use 3D shapes.
- The Rogue's first agro tick after the wake is a set-up tick (outer 0): no attack can be chosen on it. The earliest attack choice is the fourth tick after the wake test (relative to the wake test at tick 0: sub 1 at 0, agro at 1, outer 4 at 2, first choice at 3, counting the wake test as tick 0).
- The sequencer reads the hero's position and the Rogue's own facing every tick. If the hero leaves the Rogue's band during state 0 (running away), the Rogue keeps chasing at 10.9 units per second until the swing starts; there is no abort. If the hero leaves while the Rogue is already swinging, the swing plays out and misses.
- Cooldowns count only while the hero is attackable and only down to 0. The end path redraws the chosen slot's cooldown a second time, so slot 0 is unavailable for 30 to 60 ticks after the swing ends, and slot 2 for 60 to 120.
- The recovery countdown (int[15, 30]) starts at the end path, so after a slot-3 swing (end path at s+82) the next choice can happen at s+83+N.. with N in [15, 30]; with slot 3's cooldown 0 the usual rhythm is a swing every 105 to 135 ticks. A slot-0 swing repeats at about the same rate, slot 2 at about 140 to 200.
- A dodge or a hit reaction aborts the sequencer without an end path, so the recovery countdown is not started and the chosen slot keeps the cooldown it received at the choice.
- Wake by being hit while hidden: damage is doubled and the Rogue goes to agro; he still has to run the set-up tick before he can choose.
- A strike that leaves the hero at 0 hit points puts the Rogue in the victory pose (id 73, 108 ticks); then he returns to the hidden state. The arena script grabs him within 41 ticks of that anyway (hero-death cutscene).
- Death: when the Rogue's health reaches 0 the death hook runs (base state 16, no drop, carried Neopoints 0) and the death clip plays once; the arena watcher takes over (SPEC-0041).
- Stun status (Mote effect) puts him in class state 4 until it ends, then back to agro; a stun is not refused for his alignment (only alignment 4 enemies are immune).

### What a port most likely gets wrong

1. Starting him awake. He is hidden until the hero is within 28 (standing), 30, 60 or 70 units; a hero who stands still at the start point (30.19 away) is never attacked. A port that wakes him at release, or that waits for a notice roll, behaves differently from the first second on.
2. Playing clip 0 for missing phases (SPEC-0038 Layout 6): the Rogue's slot 3 and slot 0 have only the strike phase, so a swing starts the tick it is chosen after the approach. A port that plays the idle loop for each missing phase makes every swing wait a full 100-tick cycle per missing phase.
3. Treating slot 3 as one hit. Its strike clip has three separate windows (ticks 16, 41 and 61 after the start), 10 damage each, unblockable. A shield does not help.
4. Using a timer for attacks. Attacks come from the per-tick percent rolls (24, 49 and 49 percent) while a slot's cooldown is 0, so the first swing follows the wake almost immediately when the hero is inside 15.
5. Walking him into melee by the strafe routine. The strafe routine only approaches beyond 20 and backs off inside 6; the sequencer does the closing in. Inside the band he holds and circles.
6. Not countering. Any hit he takes is answered by a 25-in-101 roll every tick of the reaction (slot 3 beyond 5 units, slot 0 inside 10 when off cooldown), so he almost always swings back within a few ticks, and a stunlock only works against him when no slot is eligible.
7. Giving him a block. He never blocks (chances 0, no clip). He dodges: one decision per hero swing at the swing's start (49 or 24 percent, only within 8 units and 60 degrees), then 77 unhittable ticks while he hops back 10 units. The dodge is decided before the swing's active window, not on contact.
8. Making him hittable during the dodge or letting him be hit while the dodge animation plays; the hittable test excludes outer sub-states 5 and 6.
9. Forgetting the doubling: the first hit that finds him not in agro (hidden, coin fetch) does double damage.
10. Missing the coin detour and the return to hiding. After he steals and the hero is more than 10 units away he runs to the coins, deletes them and goes home.
11. Treating the steal as always on. It needs a damaging strike (armoured heroes take 0 from slot 2, so no steal) and a 50 in 101 roll.
12. Recomputing speeds from a table. His run is the root travel of clip 40 (10.9 units per second), the back step is a one-shot that moves 4.6 units and then keeps moving him at twice its last key (12.5 units per second) while the back-off lasts, and the sideways circle is 6.5 units per second.
13. Hit points and drop: 80 hit points, no drop, no carried Neopoints; the corpse is irrelevant because the script ends the round one second after death.

### Default if unresolved (rewrite choices)

- Intro hand-off order (needs a runtime check): the slot's characters are set up when its read completes and the next streaming pass binds them, while the script grab is a script step at the end of a tick, so the fighter's think loop may run for at most the one pass in which it is bound, with the hero in whatever action state he holds then. Default if unresolved (rewrite choice): start him in the hidden state with the hero in action state 1 (wake distance 28). If the original instead lets his think loop run once before the script grabs him, the only difference is that a hero in an unlisted action state (factor 1.0, wake distance 40) would wake him during the intro. The original script gives the player no input during the intro, so the result is the same.
- Weapon contact: resolved by the general rule (SPEC-0064, Shared field behaviour: Weapon contact and the hero's hurt shape): weapon 1 (radius 6.0, height 1.0, right hand) against the hero's capsules, with the first touching ticks of the Hit and hurt windows table; stand-in only for a port without attachment frames: the "at the tick" range of the Weapon reach table (slot 0 1.8 behind to 4.0 ahead, slot 3 about 4.2 behind to 8.6 ahead, the steal swing 2.4 behind to 9.0 ahead). The sequencer starts a swing only when d is 5.0 or less, so a standing hero is always struck.
- Facing while circling and holding: resolved (Medium-High): the facing follows the turn blends of SPEC-0038 Behavior 48 (0.9 in the hold, 0.1 or 1.0 for the side gaits, 0.3 for the others), so it is close to but not exactly aimed at the hero; a port that does not model the blends may treat the facing as aimed (dot 1.0) for the arc tests.
- Hero distance in the wake and attack tests: resolved (High): horizontal distance, with the height difference dropped.

## Round 1: Ixi Henchman

Pack 1176, arena round r = 1 (flag 381 set, flag 382 clear). Opponent entity 13, loaded from region 1180, character record 0. Confidence: High for the row, clips, attack arithmetic and defence rules; Medium-High for the start sequence (one tick of offset either way); Medium for strike reach geometry and for anything that depends on attachment frames. All times are ticks (60 per second). Every clip of this fighter has speed byte 2, so at play rate 1.0 a key lasts 4 ticks and a one-shot clip of F keys passes its 0.99 gate 4(F - 1) + 1 ticks after the tick that requested it, one tick earlier from F = 25 keys (SPEC-0005). "s" is the tick on which a request is made.

### Fight AI build guide

**1. Identity and tunables**

| Item | Value |
| --- | --- |
| Name | Ixi Henchman (the name shown on the health bar) |
| Monster type / variant / row | 76 / 1 / row 160 of the monster table (the same row, clip set of 40 clips and 166-entry animation table as the close-combat Henchmen of the Chieftain fights, SPEC-0014) |
| Class | 17 (common enemy, SPEC-0038) |
| Spawn point | locator 2 (205.60, 27.92, 322.52); spawn facing (-0.932, -0.063, -0.358), which already points at locator 0 (dot 0.998) |
| Spawn record words | default state 8 (hide and emerge); emerge distance (f32 at +0xC0) 40.0; word +0xC4 = 1; selector +0xC8 = 0 (emerge animation 10, hide pick 9); alarm group 255 (none); drop placement ordinal -1 (no drop, no coins) |
| Hit points | 65 |
| Body | radius 2.0, height 4.0; attack-manager weight (difficulty) 8; target type (alignment) 0; hostile; Display in default 1 |
| Approach routine | 4 (strafe): strafe band 10.0 to 20.0, backbiter factor 1.0 (no effect); unused bands: rush 20 to 50, desired 10 to 15 |
| Ground turn weight (aim blend in hold states) | 1.0 (it faces the hero at once) |
| Recovery after an attack (row 0x64 / 0x68) | int[30, 180] ticks |
| Death disappear timer | 360 ticks |
| Combat timer 240 to 480 | stored on entering agro and never read; no effect |
| Notice values (only used outside this fight) | awareness 85, timer 30, view cosine 0.707, band 5 to 40 with chance 70 at 5 and 10 at 40 |
| Map search | off (flag 0) |
| Defence | missile look-ahead and hold 60; missile dodge 80, missile block 50; melee block 80 and melee dodge 80 for quick, normal and heavy swings; defence timer after any defence int[60, 240] |
| Blocking Angle | 0.0 |
| Weapon volumes | weapon 1 radius 4.0, weapons 2 to 5 radius 2.0, all full length 2.0 |

Attack slots (k = slot; band and chance are the row values; "per tick" is the chance of passing the roll on one tick):

| k | Type | Band (strict) | Chance, per tick | Facing cosine above | Range | Cooldown draw | Charge | Damage / knockback | Speeds (wind-up, strike, recovery) |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| 0 | melee jab | 0 to 10 | 25, 24 % | 0.9239 | 4.0 | int[30, 60] | none | 15 / 0 | 1.0, 1.0, 1.0 |
| 1 | melee | 7 to 12 | 0 (never chosen) | 0.707 | 10.0 | int[30, 60] | none | 21 / 0 | 1.0 |
| 2 | melee lunge | 0 to 50 | 25, 24 % | 0.966 | 5.0 | int[60, 120] | kind 2, distance drawn between 10 and 50 | 35 / 1.0 | 1.0, 1.0, 1.0 |
| 3 | unused | | | | | | | | |

No slot has a counter trigger, a status (the status chance is 0 on all slots), a steal, a drain or a summon.

**2. Arena start sequence**

1. Scene (SPEC-0041, encounter thread). The slot load creates the fighter. The script grabs its control (base state 33). A grabbed character's think loop is not stepped, so the fighter does nothing during the whole scene, whichever class state it holds (the loop's first step, which puts it in class state 8, either ran before the grab or runs at the first step after release; the result is the same). The script moves it to locator 2, plays camera 4 and plays intro animation 13 with wait: id 13 is clip 35 (56 keys, one-shot), a wait of 220 ticks. Later the hero plays animation 182, line 52 is shown, and the script releases the fighter (base state 1) and returns the camera to normal, which puts the hero in action state 1 (stance) and gives control back.
2. Release tick (call it R). From R the think loop runs. Class state is 8 (hide and emerge), sub-state 0. The fighter stands at its home point playing the hide pick, animation 9 (clip 31, 14 keys, looping). It is hittable in this state (the spawn word +0xC4 is 1, so the hidden-kind rule does not apply).
3. Trigger. Each tick in sub-state 0 the fighter compares its distance to the hero with 40.0 x F1, F1 being the band factor of the hero's current action state (SPEC-0038 Behavior 18): stance (1) 0.7, so 28.0; slow walk (2) and 5: 0.75, 30.0; walk and jump-walk (3, 9): 1.5, 60.0; run and others (4, 6, 10): 1.75, 70.0; sneak and the ladder states 22, 21, 41: 0.5, 20.0; any other state (attack 13, guard 20, jump 7, hurt 17, and so on): 1.0, 40.0. The test is strict (distance below the threshold). The hero starts at locator 0, 30.19 units away, so a hero that stands still, or only slow-walks, never starts the fight; the first normal walk step, an attack, a guard or a jump does. When the test passes, sub-state becomes 1 on that tick (tick R + a).
4. Emerge. On tick R + a + 1 sub-state 1 requests animation 10 (clip 25, 20 keys, one-shot) and sets sub-state 2. The gate passes 77 ticks after the request. On that tick the class state becomes 19 (agro), with the setter's entry work: outer sub-state 0, defence hold 0, recovery countdown 0, all four slot cooldowns 0, the fighter added to the attack manager.
5. First agro run. The next tick (request tick + 78) is the first run of the agro machine: outer sub-state 0 is a set-up step that sets the outer sub-state to the approach routine (4) with inner 10, hold 0, route flag 1. The normal attack choice does not run on that tick (it needs outer 1, 3, 4 or 11). The first choice is the tick after (request tick + 79). Medium: the whole chain can be one tick shorter or longer if the original runs the class handler differently; nothing else depends on it.
6. A hit during the hide or the emerge (sub-state 0, 1 or 2) skips the rest: the take-hit hook doubles that hit's damage (the class state is not 19), enters agro through the setter, and starts hit-reaction kind 0 at once (outer 10). No emerge animation plays (an emerge animation that was already playing is replaced).
7. If the hero dies to a strike, the fighter plays animation 73 (class state 10) and then returns through the default case to class state 0 and again to state 8, hiding. The death watcher of the script takes over long before that matters (SPEC-0041).

**3. Per-tick order**

Each tick, for this fighter: the speed scale P is reset to 1.0; the status tick runs; the base-state switch decides whether the think loop is stepped (it is for base states 1 and 17 and the default; not for 16, 25, 33, 36, 42, 53, 55); the playing clip is re-timed with P; the motion step moves the fighter by the root step the clip prepared on the previous tick; the animation pass starts a newly requested clip and prepares its first root step. So a gait requested on tick t moves the fighter for the first time on tick t + 1 and the think loop sees that step on tick t + 2.

Inside the think loop: target search, pre-step (pause, awareness countdown, alert flag, retarget countdown, alarm-group check, map search, class-state switch), then the class-state handler. In class state 19 the agro machine runs from the pre-step switch, in this order: (1) the recovery countdown goes down by one when positive; (2) base state 20 with outer other than 7 or 8 is reset to 1; (3) missile defence, melee defence, lost-sight test, normal attack choice, counter-attack choice (none for this row), distance-hide gate (off for this row); (4) the outer sub-state switch; (5) the route and leash tail. Then the class-19 handler (defence hold down by one, hold animations).

**4. State table**

| State | Action id -> clip | Keys, rate | Gate | Exits |
| --- | --- | --- | --- | --- |
| Hide (class 8, sub 0) | 9 -> clip 31, loop | 14 keys | none | hero closer than 40 x F1; any hit |
| Emerge (class 8, sub 1 and 2) | 10 -> clip 25 | 20 keys | 77 | agro |
| Strafe hold (outer 4, inner 10) | 31 -> clip 31, loop | 14 keys | none | see Movement |
| Strafe forward (inner 7) | 28 -> clip 22, loop | 11 keys, 5.67 units/s forward = 0.0945 per tick | none | d below desired |
| Sidestep left / right (inner 8 / 9) | 30 -> clip 28 / 29 -> clip 27, loop | 11 keys, 5.67 units/s sideways | none | hold ends |
| Back off (inner 11) | 32 -> clip 29, loop | 11 keys, 9.56 units/s backward = 0.1593 per tick | none | d above desired |
| Slot 0 jab | A 33 skipped; B 34 -> clip 0; C 35 skipped; D 36 -> clip 2; E 37 skipped | B 15 keys, D 3 keys | B 57, D 9 | see Attack chooser |
| Slot 2 lunge | A 43 skipped; B 44 -> clip 6 (loop, 7 keys, 20.0 units/s = 0.3334 per tick); C 45 -> clip 4; D 46 -> clip 2; E 47 skipped | C 22 keys | C 85 | see Attack chooser |
| Melee dodge (outer 5) | 55 -> clip 12 | 16 keys | 61 | defence exit |
| Melee block (outer 7) | 56 -> clip 8, loop (15 keys); block-hit 126 -> clip 5 (7 keys, gate 25) | | | defence exit |
| Missile dodge (outer 6) | 53 or 54 (int[0, 1]) -> clip 23 / 24 | 18 keys, about 3.9 units/s sideways | 69 | defence exit |
| Missile block (outer 8) | 56 -> clip 8 | | | defence exit when no hero shot is live |
| Hit reaction (outer 10, kind 0) | 57 -> clip 11 | 13 keys | 49 | outer 0 |
| Victory pose (class 10) | 73 -> clip 39 | 37 keys | 144 | class 0, then 8 |
| Dead (base 16) | 58 -> clip 10 | 28 keys | 108 (the 0.95 gate used by the corpse countdown comes at 104) | corpse removed 360 ticks after that gate |
| Intro (script only) | 13 -> clip 35 | 56 keys | 220 | script |

Slot 1 ids (38 to 42) exist only as B 39 (clip 13, 20 keys); the slot is never chosen. Ids 18, 23, 24, 25, 62, 64, 82 and 125 have no clip.

**5. Attack chooser**

Runs every agro tick in outer 1, 3, 4 or 11 while the recovery countdown is below 1 and the hero is attackable (not dead, not in hurt state 17 or knockback state 48, not in a held or script state). While the hero is not attackable the four cooldowns do not count down. Slots are tested k = 3, 2, 1, 0 and the first to pass wins: type not unused, cooldown below 1, band strict on both ends, facing cosine above the slot's arc, one roll int[1, 100] below int(chance), charge rule (a charge slot needs a route to the hero; the jab needs a route or d below 10.0, which always holds in the ring). In practice slot 2 is tried first at 24 % per tick whenever its cooldown is spent, then slot 0 at 24 % (so 18 % when slot 2 failed its roll) when d is under 10.

On a choice: the slot's cooldown is drawn; the attack sequencer starts (state 6 for the lunge, state 0 for the jab); the distances are redrawn in this order: the retreat distance first (only for an approach-3 enemy), then the desired distance, then the charge floats (the lunge's charge distance is therefore a draw from 10 to 50, uniform, made on the choice tick before the run starts). The cooldown is drawn again at the end path. After the sequencer's end path the fighter is in outer 0 with a recovery countdown of int[30, 180]; no new choice before it is spent. A hit reaction abandons an attack without running the end path (no recovery countdown), so after a reaction the next choice waits only for the cooldown drawn at the choice.

Jab (slot 0): tick c choice, state 0. With d above 4.0 the fighter walks forward with gait 28; at d of 4.0 or less (and both the hero and the steer point on the walk mesh) it goes to state 1 on the next tick (s). On s: A is missing, B 34 is requested and state 2 begins in the same tick. Strike window: ticks s + 20 to s + 39 (frames 5 to 9 carry weapon bit 1). One hit, 15 damage, on the first window tick the weapon touches the hero (not necessarily s + 20). For a still level hero straight ahead, the exact weapon-versus-body test gives a maximum reach of 2.8 units from the fighter's centre, while the chase stop keeps the centres 3.175 apart (radius 2.0 + 1.175); the jab therefore misses at the chase stop. A hero standing 2.5 units away when the jab starts is touched on s + 24. Resolved: the still-hero-at-chase-stop question is closed for a level hero straight ahead; oblique, moving, or non-level targets can differ under SPEC-0018's asymmetric test. The clip passes its gate on s + 57 and the end path runs on s + 58, with or without a hit (rate stays 1.0). A strike the hero blocks plays D (3 keys, gate 9) and then ends. The clip carries about 3.4 units/s of forward root travel (the step is dropped when it would bring the fighter within the sum of the two body radii of the hero while the sequencer is in states 0 to 3).

Lunge (slot 2, charge kind 2): on tick c state 6 aims at the hero, finds A missing, requests B 44 (the 20.0 units/s run clip) and moves to state 7 in the same tick. State 7 runs from c + 1: each tick it counts, and when d is 5.0 or less it goes to state 8 on the next tick; otherwise when the distance travelled since tick c has reached the drawn charge distance it goes to state 12. There is no steering and no strike test during the run: the fighter runs straight along its facing. State 8 requests C 45 on its first tick (c8): the strike test runs every tick; weapon 1 (radius 4.0) is open on ticks c8 + 24 to c8 + 47 (frames 6 to 11); one hit (35 damage, knockback 1.0) on the first touching tick: c8 + 28 for an idle level hero 4 to 7 units ahead when C starts (the usual case, since state 8 begins at d of 5.0 or less), c8 + 27 at 3.5 units, c8 + 24 at 3 units or closer (no chase stop in the charge states); the gate passes on c8 + 85, state 12 follows, E is missing and the end path runs on c8 + 86. If the drawn charge distance is shorter than the gap to the hero, the fighter stops where the distance runs out (state 12 at once) and the lunge whiffs; with the hero at distance D0 above 5, the run reaches the hero for charge distances of at least D0 - 5, so a hero 25 units away is reached with probability 75 %.

**6. Movement rules (approach routine 4)**

Desired distance = 10 + 10 f, f = int[20, 80] / 100 (12 to 18), drawn at creation and after every normal choice. Outer 0 -> outer 4, inner 10, hold 0, strafe lock 0.
- Inner 10 (hold, animation 31, faces the hero): hold goes down by one. While it is still positive the distance tests apply; when it is not, one roll r = int[1, 3] (1 stay, 2 inner 8, 3 inner 9; 2 and 3 are replaced by staying while the lock is up), then hold = int[60, 180], and the distance tests run and win. Distance tests: d above 20.0 -> inner 7; d below 10.0 and lock below 1 -> inner 11.
- Inner 7: walk toward the hero; d below desired -> hold 0, inner 10.
- Inners 8 and 9: sidestep (left for 8, right for 9, facing the hero), hold down by one, below 1 -> inner 10; a blocked flag (wall) -> inner 10 with lock 120; then the same distance tests.
- Inner 11: walk backward away from the hero; d above desired, or blocked (lock 120) -> hold 0, inner 10.
Consequence: after every swing the fighter, now 4 to 8 units from the hero, backs off until d exceeds its desired distance (12 to 18) unless it attacks first. With the hero standing still at 20 units or less it holds and attacks in place. The ring wall stops steering (blocked flag).

Movement table (movement pass; row 160 uses the Ixi spear rig of SPEC-0065, Ixi Spearman Movement table; speed byte 2): strafe approach (inner 7) and sequencer approach id 28 = clip 22, 0.0945 per tick (5.67 u/s); circles id 29 (+x) and 30 (-x), 0.0945 per tick sideways, facing the hero (the travel goes to the side opposite to the steering point, SPEC-0038 Behavior 51); back off id 32 = clip 29, 0.1593 per tick (9.56 u/s); hold id 31, aim weight 1.0; the slot-2 lunge run id 44 = clip 6, 0.3334 per tick (20.0 u/s), solid (charge kind 2), no steering; melee dodge 55 = 7.39 units backward over 61 ticks; hit reaction 57 = 2.93 units backward over 49 ticks. Turning while moving is the steering blend (0.3 / 0.1). The navigation mesh of the ring (32 triangles) confines it; it never meets a 2.0 drop.
Worked example (start). The hero stands 24.2 units ahead at the first agro tick: d above 20.0 gives inner 7, the walk clip is requested and its first step lands one tick later; with the desired distance drawn at 15.0 it covers 9.2 units in 98 ticks, sees d below 15 and holds after one more step, unless the slot-2 lunge (band 0 to 50) starts first, which it usually does.
The fighter never leaves agro in the arena (route flag is true; sight line is clear).

**7. Hit and hurt windows**

Attack windows are given above (jab s + 20 to s + 39, lunge c8 + 24 to c8 + 47, one hit per window). The hit list (one entry per target) is cleared at the attack start and at the start of each separate open window. It is hittable by the hero except when: in outer 5 or 6; base state 16; script-held. It is hittable while blocking (outer 7), but the hero's swing is judged by the guard rule below. Damage dealt to the hero: row value x 1.0, then shield and armour (SPEC-0036 Enemy attack damage to a hero); a blocked strike (hero guarding with the hero's facing dot the fighter's facing at most -0.5) still deals the reduced damage, plays D and ends the swing; damage above 0 on a living hero knocks it back by the slot's knockback (35-damage lunge: 1.0) into state 48.

**Weapon reach.** Conventions are those of "Weapon reach tables" in SPEC-0064 (shared field behaviour). Row 160, Medium. The tick text above and in the test vectors is authoritative for the hit ticks (the lunge window opens on c8 + 24 and the hit lands on the first touching tick); "sC" is the tick the lunge clip is requested. Weapon 1 has radius 4.0 and height 2.0.

| Slot (phase, clip) | Window ticks (level contact) | Centre forward / up / side | Tor ahead, at the tick | Tor ahead, from the start | Tor feet vs origin |
|---|---|---|---|---|---|
| 0 jab (B, clip 0) | s+20 to s+39 (s+20 to s+39) | 0.6 to 1.2 / 2.6 to 3.6 / -1.1 to -0.7 | -1.6 to 2.8 | 1.3 to 6.3 | -4.5 to 7.5 |
| 2 lunge (C, clip 4) | sC+24 to sC+47 (sC+24 to sC+39) | 0.0 to 2.7 / 2.2 to 2.6 / -1.7 to 3.4 | -2.4 to 5.6 | -1.9 to 6.1 | -4.5 to 6.5 |

The jab is short (2.8 ahead at most at the tick, 6.3 from where the fighter stood). The lunge reaches 5.8 ahead. Slot 1 (charge kind 0, drawn distance 0 to 0) ends its state 7 on the first tick, so its jab windows are not reached.

**8. Defence and counter rules**

Melee defence: evaluated once per hero swing, on the first fighter tick that sees the hero's swing predicate true (the hero is in a melee swing with a class: Tor sub-states 5 and 6 quick, 2, 3 and 9 normal, 4, 7, 8 and 12 heavy; Roberta 5, 6, 7 and 10 quick, 2, 3, 8, 11 and 12 normal, 4 heavy; Tor's power step has no class and is never defended). Conditions: not outer 10; not in outer 2 unless the sequencer is in state 0, 1 or 5; defence hold below 1; the fighter within 8.0 units of the hero and inside the hero's facing arc (dot of the hero's facing with the direction from the fighter to the hero above 0.5). Then r1 and r2 = int[1, 100]: r1 <= 80 -> block (outer 7); else r2 < 80 -> dodge (outer 5); else nothing (the latch stays set until the swing ends). Probabilities: block 80 %, dodge 15.8 %, nothing 4.2 %. Starting a defence sets the hold to 60; each tick the hold goes down by one; the defence exit (outer 0, base state 1) sets the hold to int[60, 240].
- Dodge: unhittable for the whole dodge (outer 5 is excluded from hero hits); 61 ticks.
- Block: base state 20 for as long as the hero's swing predicate holds, facing the hero. A hero melee swing that overlaps a guarding fighter is blocked when dot(fighter facing, hero facing) is below the Blocking Angle 0.0 (it always is when the two face each other): no damage, block effect, the fighter is added to the hero's hit list and block-hit animation 126 plays once. A swing with the dot at or above 0.0 (hitting a guarding fighter from behind) does nothing at all.
Missile defence (hero shots): not in outer 10, not in outer 2 with the sequencer in state 5, hold below 1. For each live hero shot (up to six) the path from the shot to its position plus velocity x 60 ticks is tested against the body capsule; on a meeting, r1 = int[1, 100] below 80 -> dodge (outer 6), then a second roll below 50 -> block (outer 8, the later wins). So per shot: block 49 %, dodge only 40.3 %, nothing 10.7 %. Hold 60; defence exit as above.
There are no counter-attacks.

**9. Hero actions against this fighter**

| Hero action | Effect |
| --- | --- |
| Any swing, fighter hidden or emerging (state 8) | Fighter hittable. Damage doubled, class state 19 at once, hit reaction kind 0. Not defended (class state is not 19). |
| Tor ordinary or finisher swing, fighter in agro, in front | Defended 96 % of swings (80 % block: nothing; 15.8 % dodge: nothing). Undefended swing: damage = weapon stat A x step factor D (2.0 for the finisher, 0.75 for the quick swings), minus 0 defence; starting sword 98 (stat A 4): 4, 4, 8. Take-hit: attacker faces the hero, hit reaction kind 0 (animation 57, 49 ticks) interrupts any attack in progress with no recovery countdown, alert flag set, hit-recovery timer 90 (no effect on damage). |
| Swing from outside the 60 degree arc or beyond 8.0 units | Not defended (the defence test fails). |
| Swing while the fighter is winding up in the sequencer (states 2, 3, 4, 6, 7, 8) | Not defended; hits normally. The lunge's run and strike therefore cannot be dodged or blocked. |
| Swing while the fighter is already defending (hold above 0) | Dodging: no effect (unhittable). Blocking: blocked again. |
| Tor power step with a Mote | Never defended; hits for P x A x B x C. Match bonus C only for a Nova Mote (target type 0). |
| Roberta shot | Missile defence above; an undefended shot damages per SPEC-0036; a blocking fighter judges it by the shot rules. |
| Hero guard / shield | Blocks the jab and the lunge when the hero faces the fighter (dot at most -0.5), reduced damage, knockback still applies. Does not stop the knockback. |
| Hero keeps 30.19 units away without attacking | The fight never starts (stance). |
| Killing hit | Death hook: base state 16, animation 58, no drop, no coins; the script's death watcher sees the dead entity next tick, waits 60 ticks, then fades out. |

**10. Edge cases for implementers**

- The creation-time charge distance is a draw between two row words that read as not-a-number and 0.0; it only matters for counter charges, which this row has none of. Redraw before use at every choice.
- Slot 1 has band 7 to 12 and chance 0: it is dead data.
- The reversed charge range (max 10 at the maximum field, 50 at the minimum field) is read as the range 10 to 50 whichever way round.
- Hit reaction during the lunge's run: outer 10 replaces everything; the body keeps no momentum.
- The attack manager has weight 8, but a lone fighter is entry 0 and never waits; with spectators (not class 17) nothing changes.
- The sidestep clips carry sideways root travel; the back-off clip carries about 9.56 units/s; the wall sets the blocked flag.
- Dead test for the script: base state 16.

**11. What a port most likely gets wrong**

1. Starting the fight at release. The fighter waits in the hide state until the hero is within 28.0 (stance), 30.0 (slow walk), 60.0 (walk) or 40.0 (attack and other) units, then plays a 77-tick emerge animation, then the agro machine starts one tick later and the first choice one more tick later (about 80 ticks after the trigger).
2. Treating missing attack phases as clip 0: only B (34), B/C/D (44, 45, 46) and D (36) exist; A, C and E of the jab and E of the lunge are skipped in the same tick.
3. Letting the lunge steer or hit during the run; it runs straight and the strike test starts only when state 8 plays C.
4. The 80 % block / 80 % dodge per swing and the 60 to 240 tick defence timer, and that a block holds until the hero's swing ends.
5. Not interrupting attacks on a hit (outer 10) and not skipping the end path.
6. Back-off after each attack and the strafe hold with random sidesteps.
7. Per-tick chances are 24 %, not 25 %.
8. Damage doubling on the first hit while still in state 8.

### Test vectors

Tick R is the release tick; tick 0 is the tick on which the hero first satisfies the trigger (stance F1 = 0.7).

1. Trigger thresholds: stance, hero at 27.9 units: passes; at 28.0: does not (strict). Slow walk at 29.9: passes; at 30.0 does not. Walk (state 3) at 59.9: passes. At the arena start (30.19), stance or slow walk: nothing happens for any number of ticks.
2. Partial start timeline, hero standing still 24.2 units ahead after the trigger, rolls passing on the first tick, timers at their minimum: tick 0 sub-state 1; tick 1 request animation 10; tick 78 class state 19; tick 79 outer 4, inner 10; tick 80 choice slot 2 (cooldown 60), B requested; tick 139 d is 4.86 (5.0 or less) -> state 8; tick 140 C requested. For a still, level Tor at d = 4.86 the exact weapon test first touches on c8 + 28 = tick 168 (35 damage, knockback 1.0); the gate is tick 225 and the end path tick 226. The charge has solid body collision and may push Tor before C starts, so a live fight with a displaced Tor can differ; the stationary figure is the reference. The choice on tick 80 happens on tick 80 + G where G is geometric with probability 0.24 per tick (mean 3.2); every later tick of this vector shifts by G.
3. Same start, slow draws (recovery 180, cooldowns 120, rolls failing about three ticks each): lunge hit on tick 168 (+G); end path on tick 226; back-off ends at tick 317 (d above 18); next choice on tick 408 (d about 18.4, charge run 42 ticks, hit on tick 479); end path tick 537. Hits land every 300 to 310 ticks. Range over all draws for the gap between two lunge hits: 30 + 146 (end path) to 180 + 146 ticks plus the run time.
4. Hero attacks first, hero 6 units ahead, fighter already in strafe hold: Tor presses a swing (sub-state 2, window ticks h + 8 to h + 23, recover bits h + 28 to h + 47) at tick h. On tick h + 1 the fighter sees the swing and rolls. Block (80 %): outer 7, hold 60, animation 56; the swing is blocked on its first window tick (no damage); the fighter stays in the block until the hero's swing ends (about h + 48), then defence exit with hold int[60, 240]: it cannot defend again before h + 109 to h + 289. Dodge (15.8 %): outer 5 from h + 1, unhittable until h + 62, the swing whiffs. No defence (4.2 %): hit on the first window tick, 4 damage with sword 98, fighter at 61 HP, hit reaction (animation 57, ends h + 59), attack choices resume after the cooldown and the recovery countdown.
5. Hit while hiding: Tor swing at the fighter in state 8 with sword 98: damage 8 (doubled), the fighter is in agro with outer 10 kind 0 at once and the emerge never plays.
6. Damage arithmetic on the hero: lunge 35, no shield, no armour: 35; with a body item of stat A 20: trunc(35 x 0.8) = 28.

### Open questions

1. Weapon reach geometry. The strike windows assume the hero is inside the weapon volume. The volumes are cylinders on attachment frames of the model; the distance at which a jab (range 4.0) or lunge (range 5.0) actually touches the hero depends on those frames. Resolved by the general rule (SPEC-0064, Shared field behaviour: Weapon contact and the hero's hurt shape); the distances are in the Weapon reach table of this round; stand-in only for a port without attachment frames: treat the strike as touching when a level hero is inside the "at the tick" range of the slot's row (1.6 behind to 2.8 ahead for the jab, 2.4 behind to 5.8 ahead for the lunge).
2. Hit count per window for the kind 2 lunge. Resolved (High; proactive (open close-out)): the swing's hit list is cleared at the start of each separate open window and holds one entry per target, so the lunge hits a given target at most once per window.
3. Root motion of the intro clip. Resolved (High; proactive (open close-out)): the intro clip (id 13, clip 35, 56 keys) is a frame-stored clip whose root record is zero at every key, so it carries no root travel; the fighter ends the intro at locator 2 (the script moves it there before the intro, and the first state-8 tick returns it to its home point if displaced more than 0.35).
4. Distance metric of the hide trigger. Resolved (High; proactive (open close-out)): horizontal, with the height difference dropped (SPEC-0038 Behavior 40).
5. Ring-edge behaviour of the back-off. Resolved (High; proactive (open close-out)): a region character is confined to its navigation mesh and a border edge acts as a wall that sets the blocked flag and a blocked timer of 120 ticks (SPEC-0056, Navmesh confinement step); the blocked flag ends the back-off and starts the 120-tick lock.
6. The exact one-tick offsets of the start chain. Resolved (proactive (open close-out)): they follow the per-tick order of SPEC-0038 Behavior 50 (each active enemy runs its update, motion and animation pass in list order, and the script step comes last); the only runtime-only part is the intro hand-off order given under the Rogue's defaults (needs a runtime check).
7. Jab against a still hero (timing pass). Resolved for a still level hero straight ahead at the chase stop: the exact weapon-versus-body result (SPEC-0018 Finite cylinder tests; High) reaches at most 2.85 units straight ahead (checked in 0.025 steps over every window tick) while the chase stop keeps the centres 3.175 apart, so the jab misses by 0.33. It touches on s + 26 at 2.7 units, s + 28 at 2.8 and s + 24 at 2.5. A hero standing 3 units to the weapon side of the facing line can be touched out to about 4.4 units of distance, but the fighter faces the hero when the jab starts. The same holds for the Henchman and Dark Ixi Spearman jabs (same weapon). Oblique, moving or non-level target outcomes follow the asymmetric collision rule in SPEC-0018 and are not represented by this straight-ahead example.

## Round 2: Sludgy

Pack 1176, arena round r = 2 (flags 381 and 382 set, 383 clear). Opponent entity 14, loaded from region 1181, character record 0. Confidence: High for the row, the clips, the attack arithmetic and the defence and counter rules; Medium-High for the start sequence; Medium for the lob's flight geometry and hit chance and for the strike reach geometry. Time units and the meaning of s are as in round 1. Every Sludgy clip has speed byte 2 (4 ticks per key at play rate 1.0).

### Fight AI build guide

**1. Identity and tunables**

| Item | Value |
| --- | --- |
| Name | Sludgy |
| Monster type / variant / row | 92 / 0 / row 231; clip set of 24 clips; animation table of 166 entries |
| Class | 17 |
| Spawn point | locator 2 (205.60, 27.92, 322.52), facing toward locator 0 as in round 1 |
| Spawn record words | default state 8; emerge distance 40.0; +0xC4 = 1; +0xC8 = 0; alarm group 255; drop ordinal -1 (no drop) |
| Hit points | 80 |
| Body | radius 2.0, height 4.0; attack-manager weight 2; target type (alignment) 2; hostile; Display in default 1 |
| Approach routine | 1 (rush): rush band 3.5 to 5.0 (hold while the distance lies in it) |
| Ground turn weight | 0.2 (it turns toward the hero by a fifth of the remaining angle per tick in hold states) |
| Recovery after an attack | int[15, 120] ticks |
| Death disappear timer | 300 ticks |
| Map search | flag on: type 0 (objects of kind 0x91), distance 100, chance 50, timer 30 to 60, hero must be farther than 10.0 (see Open questions) |
| Defence | missile look-ahead and hold 120; missile dodge 75, missile block 0; melee block 0; melee dodge 100 for quick, normal and heavy; defence timer after any defence int[0, 10] |
| Blocking Angle | 0.0 |
| Weapon volumes | weapons 1 and 2 radius 3.0 (full lengths 2.0 and 1.5), weapon 3 radius 2.5 (length 1.5), weapons 4 and 5 radius 2.0 (length 1.0) |

| k | Type | Band (strict) | Chance, per tick | Facing cosine above | Range | Cooldown draw | Speeds (wind-up, strike, recovery) | Damage / knockback | Counter trigger |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| 0 | missile, lobbed, flight mode 1 | 10 to 100 | 75, 74 % | 0.0 | n/a | int[120, 480] | 1.0, 1.0, 1.0 | 20 / 0 | none |
| 1 | melee swipe | 0 to 7 | 50, 49 % | 0.707 | 5.0 | 0 (0 to 0) | 1.0, 1.0, 1.0 | 25 / 0 | code 2 (after a dodge), chance 75 |
| 2 | melee slam | 0 to 15 | 65, 64 % | 0.9239 | 9.0 | int[15, 60] | 1.3, 1.5, 1.0 | 30 / 7.0, stun 50 % | code 6 (while in hit reaction), chance 80 |
| 3 | unused | | | | | | | | |

Slot 2 stun: status 0 (stun) with chance 50 (a draw int[0, 100] strictly below 50, so 50 of 101), duration int[60, 240] / 60 in whole seconds: 1, 2 or 3 seconds with probability 60/181 each and 4 seconds with 1/181. Lob (slot 0): projectile damage 20, radius 0.2, launch speed 1.0, speed at the near edge of the band 0.5, area duration 0 and radius 0, status 0 with chance 0 and value 0 (harmless), visual kind 0, effect ids: flight none, blocked-hit effect 1461, expiry none, world impact 1461, hero-hit effect 1461.

**2. Arena start sequence**

Identical to round 1 (scene, grab, intro, release, hide state 8 with the 40 x F1 trigger, hit during the hide doubles damage and skips the emerge), with these differences. Intro animation 70 is clip 17 (21 keys, wait 81 ticks); line 51 is shown before the release; the hide pick 9 is clip 4 (20 keys, looping); the emerge animation 10 is clip 5 (11 keys, gate 41). Trigger tick a; request tick a + 1; class state 19 on a + 42; first agro run a + 43 (outer 0 sets outer 1 with inner 1, gait set 21, 21, 21, 23, 55); first normal choice a + 44. The fighter starts in the rush hold (inner 1, animation 22).

**3. Per-tick order**

As in round 1 (think loop, pre-step, class-19 machine in the order recovery, base-state fix, missile defence, melee defence, lost sight, normal choice, counter choice, hide gate (off), outer switch, tail). The map search of the pre-step runs every tick.

**4. State table**

| State | Action id -> clip | Keys, rate | Gate | Exits |
| --- | --- | --- | --- | --- |
| Hide (class 8, sub 0) | 9 -> clip 4, loop | 20 keys | none | trigger; hit |
| Emerge | 10 -> clip 5 | 11 keys | 41 | agro |
| Rush hold (outer 1, inner 1) | 22 -> clip 9, loop | 16 keys | none | d below 3.5 -> inner 2; d above 5.0 -> inner 0 |
| Rush walk (inner 0) | 21 -> clip 8, loop (gait 0 and gait 2 both play it) | 10 keys, 9.0 units/s = 0.15 per tick | none | d below desired -> inner 1 |
| Back off (inner 2) | 23 -> clip 19, loop | 10 keys, 9.0 units/s backward | none | d above desired -> inner 1 |
| Slot 0 lob | A 33 skipped; B 34 -> clip 10; C, E skipped; D 36 -> clip 20 | B 30 keys | B 116 | end path |
| Slot 1 swipe | A skipped; B 39 -> clip 11; C skipped; D 41 -> clip 20; E skipped | B 21 keys | B 81, D 53 | end path |
| Slot 2 slam | A skipped; B 44 -> clip 12; C skipped; D 46 -> clip 20; E skipped | B 26 keys | B 67 at rate 1.5 with no contact; after a hit it slows to 1.0: 90 for a hit on s + 21, 89 for s + 22 | end path |
| Melee or missile dodge (outer 5, 6) | 55, 53, 54 -> clip 13 | 23 keys | 89 | defence exit |
| Hit reaction (outer 10 kind 0) | 57 -> clip 14 | 20 keys | 77 | outer 0 |
| Victory pose | 73 -> clip 3 | 44 keys | 172 | class 0 then 8 |
| Dead | 58 -> clip 15 | 25 keys | 96 (0.95 gate at 92) | removed after 300 ticks |
| Intro (script) | 70 -> clip 17 | 21 keys | 81 | script |

Ids 18, 24, 28, 126 and others not listed have no clip (id 13 and 117 as well). The block animation 56 shares clip 13 but the row never blocks.

**5. Attack chooser**

Same rules as round 1 (tested k = 3, 2, 1, 0; first pass wins; the choice is made only in outer 1, 3, 4 or 11 with the recovery countdown below 1 and the hero attackable). Practical order: slot 2 (64 % per tick when d is under 15.0, facing above 0.9239, cooldown spent), then slot 1 (49 % per tick when d is under 7.0, facing above 0.707; no cooldown), then slot 0 (74 % per tick when d is between 10.0 and 100.0, any facing in front; cooldown 120 to 480). A hero beyond 15 units therefore gets the lob on the first choice tick with 74 % per tick. A hero within 7 units gets the slam with 64 % and otherwise the swipe with 49 % of the rest. All cooldowns are zero on entering agro, and a missile slot passes the charge rule automatically.

Lob (slot 0): choice on tick c, state 0 (missile: goes on once the hero and the steer point are on the walk mesh), s = c + 1: A is missing, B 34 is requested and state 2 begins on s. The release bit 0x20 is carried only by frame 21: the projectile is created on tick s + 84 (the first tick on which the bit is seen; the bit stays for four ticks but the spawn count is 1). Emit point 0. State 3 waits for the B gate on s + 116, state 5 follows and the end path (recovery int[15, 120], slot 0 cooldown int[120, 480]) runs on s + 117. So the lob leaves 84 ticks after the swing starts and the whole action lasts 118 ticks. Aim point: the hero's position (feet) plus a random horizontal scatter e = U[0, 2.0], plus 5.0 when the fighter is outside the camera cone, plus 3.0 when waiting in line. Flight: ballistic, speed s_v = 1.0 - 0.5 u squared with u = 1 - (d - 10) / 90 (unclamped), flight time T = d / s_v ticks (d = horizontal distance to the aim point: 27 ticks at 15 units, 38 at 25, 51 at 40, 82 at 80, 100 at 100), lifetime trunc(60 T) + 1, so the shot always lands before it expires. On contact with the hero's body in flight: hero takes trunc of the reduced damage (20 minus shield or armour), the status roll (draw 0 to 100 at most 0) is a 1-in-101 stun of 0 seconds, the effect 1461 plays. A blocked hit (hero guarding with the shot heading against the hero's facing) deals max(1, a quarter of the reduced damage). On landing the shot enters area mode with duration 0 and ends on the next tick, so the ground does no damage: only a direct flight contact hurts.

Swipe (slot 1): s = first tick of state 1 (the tick after state 0 sees d at or under 5.0; above 5.0 the fighter first walks with gait 0). B 39 requested on s; weapon 1 open on ticks s + 32 to s + 43 (frames 8 to 10); one hit, 25 damage, on the first window tick the weapon touches the hero: the exact asymmetric contact samples confirm s + 36 for a stationary level hero straight ahead from 3.0 to 7.0 units (the window start s + 32 is not the hit tick); gate s + 81 (rate 1.0 throughout), end path s + 82. If blocked by the hero's guard: D 41 plays (gate 53), then the end path.

Slam (slot 2): the pursuit range is 9.0, so when the slam is chosen at d above 9.0 the fighter first walks (gait 0 plays the rush walk) until d is 9.0 or less; s is the following tick. On s state 1 stores the wind-up word 1.3 (A is missing, so it affects only the first pass), requests B 44 and begins state 2 at the strike rate 1.5. Weapons 1 and 2 are open on ticks s + 19 to s + 29 (frames 7 to 10); one hit, 30 damage, knockback 7.0, stun roll, on the first window tick a weapon touches the hero: s + 21 for a stationary level hero straight ahead from 3.0 to 4.0 units, s + 22 from 4.5 to 9.0 (the window begins at s + 19; these exact-helper samples miss at s + 19 and s + 20). After a hit the sequencer moves to state 3, where no row word replaces the play rate; the pass of the hit tick still runs at 1.5 and the rest of the clip at 1.0: gate s + 90 for a hit on s + 21, s + 89 for s + 22 (s + 91 for the earliest possible hit on s + 19), end path one tick later. With no contact: gate s + 67, end path s + 68. Blocked by the hero: D 46 (gate 53) then end path; the damage, knockback and stun roll still apply.

**6. Movement rules (approach routine 1)**

Desired distance = 3.5 + 1.5 f, f = int[20, 80] / 100 (3.8 to 4.7), drawn at creation and after every normal choice. Outer 1 begins in inner 1 (hold). Inner 1: d below 3.5 -> inner 2; d above 5.0 -> inner 0. Inner 0: walk toward the hero (gait 2 when d is over 10.0 and gait 0 otherwise: the same clip), d below desired -> inner 1. Inner 2: back off to the point 7.0 units from the hero along the line, d above desired -> inner 1. The fighter holds while the hero stays in 3.5 to 5.0, facing it with weight 0.2. During the sequencer states 0 to 3 its step is dropped on every tick on which it is within the sum of the radii of the hero (2.0 + 1.175 = 3.175, 3D distance). With a route it never runs the no-route branch.

Movement table (movement pass; row 231; speed byte 2): rush walk (inner 0, gaits 0 and 2) and sequencer approach id 21 = clip 8, 10 keys, 0.15 per tick on every key (9.0 u/s); back off (inner 2) id 23 = clip 19, 0.15 per tick backward (9.0 u/s), facing the hero; field walk ids 2 and 4 = clip 0, 0.0625 per tick (3.75 u/s; not used in the arena); hold id 22 with aim weight 0.2 (a fifth of the remaining angle per tick: from 90 degrees off it needs about 10 ticks to come within 10 degrees); confused id 69, 1.11 u/s; the attack clips carry no root travel worth counting, so the slam and the lob are thrown from where it stands. Turning while it walks is the steering blend (0.3 / 0.1); because the step goes along the current facing, a hero who circles it quickly at 4 units makes it walk in arcs. Worked example: hero standing 25 away at release, desired 4.25: inner 1 sees d above 5.0 and goes to inner 0; 20.75 units at 0.15 per tick take 139 ticks, then it holds, unless the lob (band 10 to 100, 74 percent per tick) starts first, which it does on its first chooser tick in almost every run.

**7. Hit and hurt windows**

Lob release: tick s + 84. Swipe: s + 32 to s + 43 (first touch of an idle level hero in front s + 36). Slam: s + 19 to s + 29 (rate 1.5; first touch s + 21 at 3 to 4 units, s + 22 at 5 to 9). All one hit per window, on the first touching tick, not on the window start. The fighter is not hittable in outer 5 and 6 (dodging), nor in base state 16, nor when script-held; it is hittable in every other state, including the slam's wind-up and the hit reaction.

**Weapon reach.** Conventions are those of "Weapon reach tables" in SPEC-0064 (shared field behaviour). Row 231, Medium. The first-touch ticks above are authoritative; the table adds where the hero must stand.

| Slot (phase, clip) | Weapon | Window ticks (level contact) | Centre forward / up / side | Tor ahead, at the tick | Tor ahead, from the start | Tor feet vs origin |
|---|---|---|---|---|---|---|
| 1 swipe (B, clip 11) | w1 (3.0 x 2.0) | s+32 to s+43 (s+32 to s+43) | -2.8 to 4.8 / 0.8 to 1.8 / -4.2 to 3.3 | -3.2 to 8.4 | -3.2 to 8.4 | -4.0 to 4.0 |
| 2 slam (B, clip 12) | w1 (3.0 x 2.0) | s+19 to s+29 (s+21 to s+29) | -2.0 to 5.8 / 0.4 to 6.4 / -1.6 to 0.0 | 2.2 to 9.4 | 2.2 to 9.4 | -4.0 to 8.5 |
| 2 slam (B, clip 12) | w2 (3.0 x 1.5) | s+19 to s+29 (s+22 to s+29) | -4.9 to 8.2 / 0.0 to 7.8 / -1.8 to 3.4 | 3.8 to 11.8 | 3.8 to 11.8 | -6.0 to 9.5 |

The swipe covers 3.2 behind to 8.4 ahead. The slam touches nothing closer than 2.2 ahead (weapon 1) or 4.0 ahead (weapon 2) and reaches 9.4 and 11.8; the trigger distance of 9.0 is therefore not the reach, and the weapon 2 volume reaches farther than that. Sludgy does not move during either blow.

**8. Defence and counter rules**

- Melee dodge: same trigger as round 1 (once per hero swing, first fighter tick that sees the hero swing; hold below 1; not in outer 10; in outer 2 only in sequencer states 0, 1 and 5; fighter within 8.0 units and 60 degrees of the hero's facing). r1 can never be at most 0, so it never blocks; r2 = int[1, 100] below 100 -> dodge: 99 % of swings. The hold is then 120; the dodge animation (clip 13) lasts 89 ticks; the defence exit (outer 0, base state 1) sets the hold to int[0, 10]. So a continuous Tor combo is dodged: swing 1 starts the dodge (the fighter is unhittable for about 90 ticks), swing 2 (started while the hold is still high) finds it unhittable, swing 3 usually starts after the exit with the hold at or under 1 and is dodged again. The hero's power step has no swing class and is never dodged; a swing outside the 60 degree arc, or one started while the fighter is in the sequencer's states 2 to 4 (strike), 6 to 15, or in a hit reaction, is not dodged.
- Missile dodge (hero shots): hold 120 look-ahead; first roll below 75 -> dodge (outer 6, animation 53 or 54, clip 13, 89 ticks): 74 %. The block roll never passes (chance 0).
- Counter code 2 (slot 1): while the fighter is in outer 5 or 6 and any hero clip shows the recover bit (Tor's swings carry it on their last frames: ordinary first swing frames 7 to 11, ticks about 28 to 47 after the press; second swing frames 8 to 15; finisher frames 9 to 12), slot 1 is eligible when d is between 0 and 7, the facing is above 0.707 and its cooldown is below 1 (always). One roll int[0, 100] per tick; below 75 (75 of 101) starts the swipe at once, which ends the dodge (outer 2). So a Tor swing at close range is dodged and punished: swipe request at about h + 29, hit (25 damage) on its first touching tick s + 36, about h + 65. Roberta never opens the recover bit, so against her the counter never fires.
- Counter code 6 (slot 2): while in hit reaction (outer 10), band 0 to 15, facing above 0.9239, cooldown below 1: roll int[0, 100] below 80 starts the slam at once (the same roll rule). The slot cooldown int[15, 60] is drawn then. In the other slots' order the later eligible slot wins (slot 2 over slot 1).
- The counters ignore the recovery countdown and the hero-attackable test and do not count cooldowns down.

**9. Hero actions against this fighter**

| Hero action | Effect |
| --- | --- |
| Any swing at a hidden or emerging fighter (state 8) | Hittable; damage doubled; agro and hit reaction kind 0 at once; no dodge (class state is not 19). Starting sword 98: 8. |
| Tor swing in front of a fighter in the rush hold, hold below 1 | Dodged 99 %, then likely punished by the swipe (see counter code 2). Undodged (1 %): 4 damage with sword 98, hit reaction animation 57 (77 ticks) which interrupts any action, then counter code 6 with 80 % (slam, 30 damage, stun 50 %). |
| Tor swing while the fighter attacks (sequencer 2 to 4) | Hits normally (no defence evaluated in those states): the way to damage this fighter. The hit interrupts the attack (outer 10, no end path), and the hero can chain follow-ups during the 77-tick reaction because outer 10 is not defended. |
| Swing from the side beyond 60 degrees of the hero's facing | Not dodged. |
| Tor power step with a Mote | Never dodged; matches only Motes of kind 5 for the type bonus (ids 166 and 167) and Nova (172, 173), because the fighter's target type is 2. A poison status from a Mote is ignored (own element). |
| Roberta shot | 74 % dodged per shot (when the look-ahead path meets the body and hold below 1), 26 % hit (hold 120 means one dodge covers the next 120 ticks of shots until the exit); a shot hit on an undefended fighter damages by the shot rules. |
| Hero guard | Blocks the swipe and slam when facing (dot at most -0.5): reduced damage, knockback 7.0 still applies for the slam, stun roll still applies. |
| Hero stunned | The stun forces action state 1 each tick but leaves the hero attackable: the fighter keeps choosing attacks (recovery 15 to 120, slam cooldown 15 to 60). |
| Killing hit | As round 1 (death animation 58, clip 15). |

**10. Edge cases for implementers**

- Slot 0's band is 10 to 100 and its facing arc is 0.0, so it fires from anywhere beyond 10 units with the fighter facing at all forward.
- Slot 1 has a zero cooldown, so after the end path it can chain as soon as the recovery countdown (15 to 120) is spent.
- The slam range of 9.0 is the trigger distance, not the reach; the volumes decide the contact.
- The slam drops from 1.5 to 1.0 after a hit: a landed slam lasts 92 ticks and a missed one 68.
- The lob's lifetime unit slip is irrelevant (the shot lands first).
- A hit reaction interrupts an attack without the end path, so the slot cooldown drawn at the choice is the only wait.
- The stun is applied after the take-hit (which clears an existing stun), so a fresh stun survives; the knockback follows and sets the hero to state 48; the stun forces state 1 each tick, before the state switch, so state 48 never runs while the stun lasts (Open question 4).
- The fighter's map search is on but never finds a target in the arena (Open question 3).

**11. What a port most likely gets wrong**

1. Starting at release (same as round 1): stance hero at 30.19 never wakes it; walking does; 41-tick emerge.
2. Letting the hero win by swinging: 99 % dodge, 89-tick unhittable animation, the hold int[0, 10] after, then the 25-damage swipe counter in the hero's recovery.
3. Using a straight shot for the lob: it is a slow ballistic lob (0.55 to 1.0 units per tick) released 84 ticks after the swing starts, and the ground impact does nothing.
4. The slam's 30 damage, knockback 7.0 and the 50 % stun for 1 to 3 seconds (4 seconds 1 time in 181).
5. Treating the missing phases as clip 0; every slot has only a B clip (plus D).
6. Per-tick chance 64 %, 49 % and 74 % (one less than the row percent).
7. Counter code 6 is hit reaction only, code 2 needs the hero's recover bit and a dodge.
8. The speed words: the slam strikes at 1.5 until it hits, then 1.0.

### Test vectors

1. Start (as round 1): trigger at tick 0 (hero 25.0 units ahead, stance): tick 1 request 10; tick 42 class state 19; tick 43 outer 1; tick 44 first choice.
2. Standing hero 25 units away, rolls passing at once, minimum timers (reference run, rounds of the chain shift by the geometric wait G with probability 0.74, 0.64 or 0.49 per tick): tick 44 choice slot 0 (cooldown 120), tick 45 B requested, tick 129 lob created (d 25: speed 0.653, flight about 38 ticks, lands about tick 167), tick 161 gate, tick 162 end path (recovery 15, cooldown 120), tick 163 outer 1, tick 164 d above 5.0 so it walks at 0.15 per tick; tick 233 choice slot 2 at d 14.95; walk on to d 9.0; tick 274 request B 44; tick 293 slam window opens; tick 296 the slam touches the stationary level hero 9 units away (s + 22; 30 damage, knockback 7.0, stun roll; exact first-touch sample with chase stop); tick 363 gate (s + 89); tick 364 end path (recovery 15, cooldown 15); tick 379 next slam choice. With a whiff the gate is tick 341 and the end path tick 342.
3. Same with slow draws (recovery 120, slam cooldown 60, lob cooldown 480): lob on 44 (+G), end path 162, recovery runs to 282, slam choice 282 at d 7.6, B requested 283, window 302, hit 305 (s + 22), gate 372, end path 373, next slam 493.
4. Hero 6 units ahead, stance, trigger at tick 0: tick 44 choice slot 2 (64 % per tick; slot 1 49 % otherwise), tick 45 request B (the fighter is within 9.0), window from tick 64, hit on tick 67 (s + 22, the first touching tick at 6 units; 30 damage, knockback 7.0, stun roll), tick 134 gate, tick 135 end path, next choice about tick 150 (recovery 15 to 120, cooldown 15 to 60). Landed slam cycle: 92 + N + wait; missed slam: 68 + N + wait.
5. Hero attacks: hero at 4 units, Tor presses a swing at tick h (window ticks h + 8 to h + 23, recover bit ticks h + 28 to h + 47). Tick h + 1: the fighter sees the swing; 99 %: dodge, outer 5, hold 120, animation 55, unhittable until h + 91; the swing whiffs. At h + 28 the first counter roll (75 of 101); with probability 0.743 per tick the swipe starts, so by h + 30 almost surely: outer 2, state 1 on h + 29 or h + 30, B requested, hit on the first touching tick s + 36, about h + 65 to h + 66 (25 damage minus armour). The 1 % no-dodge case: hit at h + 9, 4 damage (sword 98), hit reaction, slam counter with 80 % on the first tick (30 damage, stun 50 %).
6. Stun duration draw: draw 60 to 119 -> 1 s (60 ticks); 120 to 179 -> 2 s; 180 to 239 -> 3 s; 240 -> 4 s.
7. Lob speeds: d 15 -> 0.554 (T 27); d 25 -> 0.653 (T 38); d 40 -> 0.778 (T 51); d 80 -> 0.975 (T 82).

### Open questions

1. Weapon reach geometry (as round 1). Resolved by the general rule (SPEC-0064, Shared field behaviour: Weapon contact and the hero's hurt shape); the distances are in the Weapon reach table of this round; stand-in only for a port without attachment frames: touching when a level hero is inside the "at the tick" range of the slot's row (3.2 behind to 8.4 ahead for the swipe; 2.2 to 9.4 ahead for the slam, weapon 2 reaching 4.0 to 11.8).
2. Lob contact: the hero's body radius (about 1.0) and the scatter U[0, 2.0] (plus 5.0 outside the camera cone) decide how often a standing hero is hit; about 60 % inside the cone. Resolved (proactive (open close-out)): test the sphere against the hero's body cylinder each tick as specified (SPEC-0029 Behavior 9).
3. Map search: the row looks for an item of kind 145 on 50 % per search with a 30 to 60 tick timer and leaves agro for the map-search state when it finds one with the hero over 10.0 away. Resolved (High; proactive (open close-out)): the search never finds a target in the arena, because neither pack 1176 nor its five character regions has an item placement and every round loads the world afresh, so the world item pool is empty (the Sludgy has no drop, and the hero cannot leave an item there). Do not model the search.
4. Knockback state 48 versus the stun forcing state 1 on the same strike. Resolved (High for the order; proactive (open close-out)): the strike runs in the enemy loop before the hero's update of the same tick, and the hero's status step (stun forces state 1 and the speed scale 0 every tick) runs before the hero's state switch, so on every tick of the stun the hero is in state 1 and state 48 never runs; the knockback velocity written by the strike is kept and carries the hero as an idle body. How far the slide goes under state 1 is a physics result (needs a runtime check).
5. Hero's recover window sampling tick. Resolved (proactive (open close-out)): the fighter's think runs before the hero's update of the same tick (SPEC-0038 Behavior 50), so it reads the event bit left by the hero's previous animation pass; the offsets in the tables already use that order.
6. Intro and start offsets: as round 1 (the hand-off order is runtime-only, see the Rogue's defaults).

## Round 3: Skeleton Fighter

Pack 1176, round 3 (flag 383 set, flag 384 clear). Opponent entity 15, loaded from region 1182. Intro animation 73, victory animation 22. Shared rules are in the common-enemy spec (Behaviors 1 to 5, 7, 13, 14, 37, 44, 45 and 48); this section lists only what is specific to this fighter and the order in which the shared rules meet. Confidence: High for the identity, the row values, the clip data and the tick counts of each phase; Medium-High for the start sequence and the tick offsets (plus or minus one tick where the sequencer runs in the tick of the choice); Medium for the hit geometry of the melee volumes.

### Identity and tunables

| Item | Value |
| --- | --- |
| Character record | region 1182, record 0, name "Skeleton Fighter", entity 15 |
| Monster type, variant, row | type 88, variant 1, row 227 (resource 530) |
| Class, approach routine | class 17 (common enemy), approach 1 (rush) |
| Hit points, radius, height, weight | 75, 2.0, 3.5, 5.5 |
| Difficulty (attack-manager weight), alignment, hostile, display in default | 8, 0 (type 0: only a Nova Mote adds its element bonus), 1, 1 |
| Reanimation | flag set; countdown int[12000, 18000] ticks (200 to 300 s); the death-disappear timer 240 never runs |
| Damage reduction, anger, can fly, map search | 0, 0, 0, 0 |
| Spawn record | position (205.60, 27.92, 322.52), facing (-0.93, -0.06, -0.36); default-state word 8 (emerge and hide); emerge distance 40.0; word at 0xC4 = 1; selector word at 0xC8 = 0; alarm group 255 (none); drop ordinal -1 (no drop); no carried Neopoints |
| Weapon volumes (finite cylinders, radius x full length) | weapon 1: 2.0 x 2.0; weapon 2: 2.0 x 2.0; weapon 3: 3.0 x 2.0; weapons 4, 5: 2.0 x 2.0. Every strike clip of this fighter carries event bit 0x1 only, so only weapon 1 is ever active |
| Rush band (row 0x4DC, 0x4E0) | 3.5 to 5.0. Desired distance = 3.5 + 1.5 x f, f = int[20, 80] / 100, so 3.8 to 4.7, redrawn after every normal attack choice |
| Recovery after an attack (row 0x68, 0x64) | int[15, 30] ticks |
| Melee defence | block chance quick / normal / heavy swing 50 / 35 / 10; dodge chance 0 / 0 / 0 (never dodges); Missile defend time 60 ticks; missile dodge 0, missile block 90 (a roll below 90 blocks: 89 of 100) |
| Defence timer after a defence | int[45, 90] ticks |
| Blocking Angle (row 0x10C) | 0.707 |
| Notice and sense values | not used in the arena (the fighter goes from state 8 straight to agro); row awareness 80, band 3 to 5 |

Attack slots (slot 3 unused). Ids A to E are 33 + 5k to 37 + 5k; "none" means the table has no clip, so the phase is skipped in the same tick (shared rule, Layout section 6 of the common-enemy spec).

| Slot | Band (strict) | Chance (per tick) | Arc | Melee range | Cooldown draw | Speed words (wind-up / strike / recovery) | Damage | Knockback | Flags | Counter code / chance | Clips by phase |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| 0 | 0 < d < 10 | 60 (59 of 100 rolls pass) | 0.707 | 7.0 | int[30, 180] | 1.0 / 1.0 / 1.0 | 50 | 0 | none (blockable) | 3 (after its own block) / 50 | A none, B 34 = clip 4 (14 keys), C none, D 36 = clip 22 (15 keys), E none |
| 1 | 0 < d < 10 | 40 (39 of 100) | 0.707 | 7.0 | int[60, 180] | 1.0 / 1.0 / 1.0 | 60 | 0 | none | 5 (own strike blocked while the hero holds L1) / 50 | A none, B 39 = clip 3 (17 keys), C none, D 41 = clip 22, E none |
| 2 | 0 < d < 10 | 100 (99 of 100) | 0.0 | 50.0 | 180 (fixed) | 1.0 / 0.7 / 1.0 | 50 | 0 | none | none | A none, B 44 = clip 21 (14 keys), C none, D 46 = clip 22, E none |

All clips have speed byte 2 (0.25 keys per tick at play rate 1). Slot ranges: slot 2's melee range of 50 means the sequencer never walks toward the hero for it; its reach comes from the clip's own root travel (below).

### Animation ids used (all play rate 1.0 unless stated; ticks are the number of ticks after the request tick in which the 0.99 gate first passes)

| Use | Id | Clip, keys, loop | Gate (ticks) | Root travel |
| --- | --- | --- | --- | --- |
| Hide pick in state 8 | 9 | clip 11, 1 key, loop | none | none |
| Emerge | 10 | clip 10, 46 keys, one-shot | 180 | none |
| Rush walk (all three rush gaits) | 21 | clip 24, 11 keys, loop | 43 per cycle | +0.2502 per tick forward |
| Back off (gait 3) | 23 | clip 12, 10 keys, loop | 39 per cycle | -0.2908 per tick |
| Hold (faces the hero) | 22 | clip 19, 32 keys, one-shot (stays on its last key) | 124 | none |
| Hit reaction kind 0 | 57 | clip 1, 17 keys, one-shot | 65 | -5.72 in total (pushed back) |
| Melee dodge (never used, dodge chance 0) | 55 | clip 27, 17 keys | 65 | none |
| Block | 56 | clip 23, 16 keys, loop | 63 per cycle | none |
| Block hit flash | 126 | clip 28, 6 keys | 21 | -1.67 |
| Missile dodge (never used, dodge chance 0) | 53 / 54 | clips 7 / 8, 13 / 15 keys | 49 / 57 | 6 to the right / left |
| Victory pose (state 10) and arena intro | 73 | clip 20, 36 keys | 140 | none |
| Arena victory pose after a hero death | 22 | clip 19, 32 keys | 124 | none |
| Death (class state 16, countdown phase) | 58 | clip 13, 56 keys, speed byte 1 (0.5 keys per tick) | 110 | -0.82 (back) and 0.16 to the side over 112 ticks (frame-stored root, mode 0; SPEC-0005); last key 0, so nothing is held |
| Rise (not reachable in the arena) | 85 | clip 14, 39 keys, speed byte 1 | 77 | small |

### Arena start sequence

1. The encounter script loads region 1182 into the round slot. The fighter is created with the spawn-record default state 8 and is under script control from the first tick (control grabbed, base state 33), so its think loop does not step until it is released (shared rule: no stepping in base state 33). At most one think tick can run before the grab; in it the fighter only enters state 8 with sub-state 0, and the hero at the arrival point is 30.19 units away, outside the emerge distance, so the outcome is the same either way (Medium).
2. The script moves the fighter to locator 2 (205.60, 27.92, 322.52; the same point as its record), plays camera 4 and plays animation 73 with wait (140 ticks to the gate; the clip is a standing pose with no root travel). The entity then returns to its idle animation and stays in script hold (control option 0). The hero is on locator 0 (177.66, 27.92, 311.07), 30.19 units away. Camera 5, hero animation 182, camera 7 and message 50 follow (the message waits for the player to close the dialogue box).
3. The script releases the fighter (base state 1), returns the camera to normal and puts the hero in state 1 (idle). The hero's talk state ends first, so at the moment of release the hero is in action state 1.
4. First think tick T0 (the tick after release). State 8, sub-state 0: the fighter emerges when the hero's distance is below 40.0 x F1, where F1 comes from the hero's action state (common-enemy spec Behavior 18): 0.7 idle (28.0 units), 0.75 slow walk (30.0), 1.5 walk (60.0), 1.75 run (70.0), 1.0 in every other state (40.0, for example while attacking, hurt or talking). From the arrival point (30.19 units) an idle hero is therefore outside the range: the fighter stays at home playing its hide pick (animation 9) and does not fight until the hero moves nearer or walks (walking gives 60), or hits it. It is hittable in this state (the hidden-kind rule does not apply because the record word at 0xC4 is 1).
5. Emerge, one sub-state per tick: tick T0 sets sub-state 1 (distance test passed); tick T0 + 1 requests animation 10 and sets sub-state 2; the gate passes in tick T0 + 1 + 180 = T0 + 181 and the setter runs with 19 (agro; entry work: outer sub-state 0, defence hold 0, recovery 0, all cooldowns 0, attack-manager entry). While emerging the fighter is hittable. A hit during state 8 skips the emerge: the take-hit hook doubles that hit's damage, runs the agro setter (same entry work) and then starts hit reaction kind 0.
6. Agro tick 1 (T0 + 182): outer sub-state 0 stores outer 1 (rush) with inner 1 (hold). Agro tick 2 (T0 + 183): inner 1 sees d above 5.0, goes to inner 0 and the handler requests the walk animation 21 in the same tick. The first root step is applied in the motion pass of tick T0 + 184, so the distance the AI sees falls by 0.25 per tick from tick T0 + 185.
7. The normal attack choice first runs in agro tick 2 (T0 + 183), because it needs outer sub-state 1.

### Per-tick order (agro)

Pre-step (pause test, awareness, alert flag, retarget, alarm check, class switch) then the agro machine: recovery countdown down by one; melee defence and missile defence (hold below 1); lost-sight test; normal attack choice (hero must be attackable; cooldowns count down only then); counter-attack choice; distance-hide gate (flag 0 for this row, never fires); outer sub-state switch (rush states or the sequencer); route flag and leash. Then the class handler (defence hold down by one, hold animation or pursuit step), the animation pass (clip start and re-time at R = 1.0 x P) and the motion pass (root step). The AI reads the cursor left by the previous animation pass.

### State table

| State | Condition | Action id -> clip | Keys / speed | Gate ticks | Exits |
| --- | --- | --- | --- | --- | --- |
| Scripted intro | control grabbed, then 73 | 73 -> clip 20 | 36 keys, 0.25 per tick | 140 | released: class state 8 |
| 8 sub 0 (hide) | hero closer than 40 x F1 | 9 -> clip 11 | 1 key loop | none | sub 1 next tick |
| 8 sub 1 | next tick | 10 -> clip 10 | 46 keys | request | sub 2 |
| 8 sub 2 | wait | 10 | 46 keys | 180 | agro (19) |
| Agro outer 1 inner 1 (hold) | d between 3.5 and 5.0 | 22 -> clip 19 | one-shot, held | none | d < 3.5 inner 2; d > 5.0 inner 0 |
| inner 0 (approach) | d above the desired distance | 21 -> clip 24 | 11 keys loop, 0.2502 per tick | none | d < desired inner 1 |
| inner 2 (back off) | d < 3.5 | 23 -> clip 12 | 10 keys loop, -0.2908 per tick, aim point hero - dir x 7.0 | none | d > desired inner 1 |
| Outer 2, slot 0 swing | chosen, sequencer state 0 then 1 | A skipped; B 34 -> clip 4; D 36 -> clip 22; E skipped | 14 keys at 0.25 | 53 | end path at +54 |
| Outer 2, slot 1 swing | as above | B 39 -> clip 3; D 41 -> clip 22 | 17 keys | 65 | end path at +66 |
| Outer 2, slot 2 lunge | as above | B 44 -> clip 21; D 46 -> clip 22 | 14 keys, 0.175 per tick while striking, 0.25 after contact | 74 (miss) | end path at gate + 1 |
| Outer 2 state 4 (own strike blocked) | hero guards | D (clip 22) | 15 keys | 57 | state 5, end path |
| Outer 5 / 7 (melee defence) | hero swing edge, hold below 1 | 55 or 56 / 126 | see Defence | | defence exit |
| Outer 6 / 8 (missile defence) | Roberta shot | 53 or 54 / 56 | | | defence exit |
| Outer 10 kind 0 (hit reaction) | any damaging hit | 57 -> clip 1 | 17 keys | 65 | outer 0 |
| Class 10 (victory pose) | the fighter's strike leaves the hero at 0 HP | 73 | 36 keys | 140 | setter 0 -> class 0 -> setter 8 (home) |
| Class 16 (reanimation countdown) | death | 58 -> clip 13 | 56 keys, 0.5 per tick | 110, then holds the last key | rises after 12000 to 18000 ticks (never in the arena) |

### Attack chooser

Runs each agro tick in outer 1 (or 3, 4, 11) while the recovery countdown is below 1 and the hero is attackable (the hero's hurt state 17, knockback state 48 and other busy states pause both the choice and the cooldown countdown). Slots are tested 2, 1, 0. A slot passes when its cooldown is below 1, the hero distance satisfies 0 < d < 10, the facing cosine exceeds the slot arc, the chance roll int[1, 100] is strictly below the chance, and the charge rule holds (no charge: a route to the hero exists or the slot is melee with d < 10; both hold in the ring). Effective per-tick probabilities with all cooldowns clear: slot 2 99 %, then slot 1 39 % of the remainder, then slot 0 59 % of that remainder, so a choice happens with probability 1 - 0.01 x 0.61 x 0.41 = 99.75 % on the first eligible tick. Slot 2 is therefore chosen on the first eligible tick after an emerge or whenever it is off cooldown. On a choice the slot cooldown is drawn (table above), the sequencer starts in state 0, and the desired distance is redrawn. The end path draws the cooldown again, so the cooldown is always counted from the end of the swing. The recovery countdown (15 to 30 ticks) blocks every new normal choice after a swing.

Counter-attack choice (runs after the normal choice, all outer sub-states, no recovery test): slot 0 triggers after this fighter's own block while the hero's swing is in its recovery window (Tor's swing clips only; Roberta never) with roll int[0, 100] below 50; slot 1 triggers when the fighter's own strike was blocked (sequencer state 4) while the hero holds L1, roll below 50. The last eligible slot wins; the slot must be off cooldown, inside its band and arc.

### Phase timing for each slot (tick 0 = tick in which the sequencer requests the strike clip; the choice happened one tick earlier)

| Slot | Window (event bit 0x1, weapon 1) | Earliest tick a level hero can be touched | First touch of a level hero straight ahead (the usual hit tick) | Gate tick | End path tick | Damage |
| --- | --- | --- | --- | --- | --- | --- |
| 0 | keys 9 to 12 -> ticks 36 to 51 | 36 | 40 (start distance 3.0 to 8.5); 42 at 9.0 | 53 | 54 | 50 |
| 1 | keys 7 to 8 -> ticks 28 to 35 | 29 | 29 (start distance 3.5 to 7) | 65 | 66 | 60 |
| 2 | keys 5 to 13 -> from tick 29 (strike speed 0.7 until the first contact) | 32 | with the chase stop: 34 at 1.5; 33 at 2.0 to 2.5; 32 at 3.0; 33 at 3.5 to 8.0; 34 at 8.5; 35 at 9.0 to 9.75 (no touch at 1.0); without the stop: no touch at 1.0 to 6.0, 33 at 6.5, 32 at 7.0, 33 at 7.5 to 8.0, 34 at 8.5, 35 at 9.0 to 9.75 | 62 with contact at 32; 63 with contact at 33 to 35 (61 at 29); 74 without any contact | gate + 1 | 50 |

The window start is not the hit tick: a swing hits on the first window tick on which the weapon volume touches a hero capsule under the asymmetric rule of SPEC-0018. The "straight ahead" column counts the clip's root travel and the chase stop (the fighter's travel is dropped while it is within 2.0 + 1.175 = 3.175 of Tor), with Tor idle, level and stationary. Without the chase stop this lunge would miss a stationary target closer than 6.5 units (its weapon passes through a near-distance gap, then touches again farther ahead); with the stop the fighter cannot move into that gap and a hero from 1.5 units on is touched. A level hero is touched by slot 0 only on ticks 36 to 44.

Slot 2 with first contact at tick h: the pass of the contact tick still runs at 0.7, then the clip plays at rate 1.0 from the next tick. For the sampled exact contacts, the gate is tick 61 for h=29, 62 for h=32, and 63 for h=33 to 35; with no contact, the gate is tick 74. These examples use the event-progress gate rule in SPEC-0005. Each slot hits at most once per swing (one window; the hit list holds the hero until the next attack start).

Reach (Medium to Low): slot 0's clip moves the fighter forward 3.5 in total and slot 1's 5.5, most of it in the keys just before and inside the window. Slot 2's clip moves the fighter 9.6 forward: keys 0 to 4 add only 1.65, keys 5 to 9 add about 7.7 (ticks 29 to 57 of the lunge at 0.175 keys per tick). Exact stationary samples with the chase stop in force first touch at tick 34 (1.5), tick 33 (2.0–2.5), tick 32 (3.0), tick 33 (3.5–8.0), tick 34 (8.5), tick 35 (9.0–9.75) and ticks 36–44 across 10.0–13.0; without the stop they miss at 1.0–6.0 and first touch at tick 33 (6.5), tick 32 (7.0), tick 33 (7.5–8.0), tick 34 (8.5), and ticks 35–44 across 9.0–13.0. The non-monotone result invalidates the old broad 2–8 unit claim. The separate 4.86-unit fight example first touches on tick 33 with the chase stop (and misses without it). A hero who steps backward during the first 28 ticks also changes displacement and contact timing.

### Weapon reach

Conventions are those of "Weapon reach tables" in SPEC-0064 (shared field behaviour). Row 227, Medium. The phase table above is authoritative for the first-touch ticks; this table adds where the hero must stand. Weapon 1 has radius 2.0 and height 2.0 in every row.

| Slot (phase, clip) | Window ticks (level contact) | Centre forward / up / side | Tor ahead, at the tick | Tor ahead, from the start | Tor feet vs origin |
|---|---|---|---|---|---|
| 0 (B, clip 4) | s+36 to s+51 (s+36 to s+44) | -1.5 to 3.6 / 0.9 to 3.8 / -2.3 to 3.0 | -2.8 to 6.0 | -0.4 to 9.1 | -3.5 to 5.5 |
| 1 (B, clip 3) | s+28 to s+35 (s+29 to s+35) | 2.2 to 3.3 / 0.5 to 4.8 / -0.4 to 1.0 | 0.6 to 5.6 | 3.1 to 7.7 | -4.5 to 6.0 |
| 2 lunge (B, clip 21) | s+29 to s+74 (s+32 to s+74) | -0.1 to 3.7 / 1.8 to 4.5 / -2.4 to -0.6 | 1.6 to 5.6 | 6.7 to 14.7 | -3.0 to 6.0 |

The lunge figures assume strike speed 0.7 for the whole clip; the real clip drops to 1.0 after the first contact, a small effect on the far end. Measured from where the fighter began, the lunge touches a level hero 6.7 to 14.7 ahead, consistent with the reach paragraph above.

### Movement rules

Rush archetype (approach 1). With a route to the hero (always true on the ring): inner 0 walks toward the hero using animation 21; d below the desired distance gives inner 1. Inner 1 stands facing the hero (turn weight from the row, 1.0) in animation 22: d below 3.5 gives inner 2, d above 5.0 gives inner 0. Inner 2 walks backward (animation 23, aim point hero - dir x 7.0) until d exceeds the desired distance, then inner 1. The fighter therefore hovers between about 3.5 and 5 units from a standing hero, attacking from there; it chatters between approach and back-off when the hero stands still at the band edge. The ring wall keeps it on the arena mesh; a blocked move is not special for the rush outer sub-state. Leash: no effect while a route exists (always on the ring).

Movement table (movement pass; row 227; speed byte 2): rush approach (inner 0) and sequencer approach id 21 = clip 24, 11 keys, 0.2501 per tick on every key (15.01 u/s); back off (inner 2) id 23 = clip 12, 0.2909 per tick backward (17.45 u/s), facing the hero; field walk ids 2, 4, 11 = clip 2, 0.0518 per tick (3.11 u/s, per key 0.002 to 0.102; not used in the arena); hold id 22, aim 1.0; missile dodges 53 and 54 = 6.07 units sideways (never used: dodge chance 0) over 49 and 57 ticks; hit reaction 57 = 5.72 units backward over 65 ticks; the slot-2 lunge clip 21 carries it 9.6 forward (see Weapon reach); the death clip 58 (clip 13, a frame-stored clip, mode 0) moves 0.82 units backward over 112 ticks and is then held; its last key is 0, so the held clip adds nothing and the corpse lies still. Worked example: released 30 units from the hero with desired 4.25: inner 0 covers 25.75 units at 0.2501 per tick in 103 ticks and holds after one more step; a walking hero (13.5 u/s) is slower than this approach, a running hero (19.29) faster.

### Defence and counter rules

- Melee defence is evaluated once per hero melee swing (on the swing-start edge), only when the defence hold is below 1, the fighter is within 8.0 units of the hero and inside the hero's front arc (the dot of the hero's facing with the direction to the fighter above 0.5, a 60 degree half-angle), and not in the middle of its own swing (it may defend only while the sequencer is in state 0, 1 or 5, or not attacking). One roll int[1, 100] at or below the block chance blocks: quick swing 50 %, normal 35 %, heavy 10 %. The dodge roll never passes (chance 0). Tor's swing classes: quick = attack sub-state 6 (sub-state 5 is never entered), normal = 2, 3 and 9, heavy = 4, 7, 8 and 12; sub-state 10 (power step) and others never trigger a defence.
- Block (outer 7): the fighter faces the hero, plays 56 (loops), sets base state 20 (guarding) while the hero's swing lasts; a blocked hero swing (no damage) sets the block-hit flag and plays 126 once. The defence hold is then 60 ticks (the row missile-defend time) and, when the block ends, is redrawn as int[45, 90]. Hero swings that arrive while guarding are blocked when the dot of the two facings is below 0.707 (hero in front); a swing from behind (dot at least 0.707) does nothing at all.
- Missile defence (Roberta's shots only): if a shot's path of 60 ticks of travel meets the body, the fighter blocks with probability 89 % (outer 8 plays 56 while a shot is alive); it never dodges.
- Reanimation: a killed Skeleton Fighter is "dead" for every script test the moment the death hook runs (countdown above 0), is not a hit candidate, and cannot be hurt again; it stands up after 12000 to 18000 ticks only if the world is still loaded and the clear-area test passes. In the arena the win sequence deactivates it about 100 ticks after the kill and then replaces the world, so it never rises.

### Hero actions against this fighter

| Hero action | Result |
| --- | --- |
| Tor swing (sub-states 2, 3, 9) on a hittable, non-guarding fighter | damage = swing product (weapon stat A x Mote factors x step factor) minus 0 (damage reduction 0), truncated; the take-hit hook runs: hit counter, face the attacker, outer 10 kind 0 (animation 57 restarted from key 0, so every hit restarts it), cancels any swing in progress (no recovery countdown is set; the slot cooldown keeps running), alert flag |
| Tor finisher (sub-states 4, 12: step factor 2.0) | double damage; defence block chance 10 %, so it nearly always lands |
| Tor power step with a Mote (sub-state 10) | ignores the block test (but not the hittable test); an element bonus only for a Nova Mote (type 0) |
| First hit while the fighter is in class state 8 (before it is in agro) | damage doubled once; skips the emerge animation; starts the agro setter |
| Roberta quick shot or full release | missile defence 89 % block; otherwise the shot hits and runs the same hook; a full release ignores guard |
| Hero guard (L1, state 20, shield equipped) | blocks a strike when the facing dot of hero and fighter is at most -0.5 (face to face); the hero then takes the strike's damage reduced by the shield's stat A (armour applies too), no hurt state and no knockback beyond the row value (0); the fighter's sequencer goes to state 4 (animation 36, 41 or 46) and counter code 5 of slot 1 may fire (roll below 50) while L1 is held |
| Hero standing still | the fighter stays hidden beyond 28 units; inside 28 it emerges, then keeps within 3.5 to 5 units and uses slot 2, 1 and 0 as the chooser says |
| Hero jumping, climbing | state 25, 33 and the busy states pause the chooser; F1 changes the emerge test only |
| Hero roll or dodge | does not exist in this game |

### Death and the arena win path

Death sets class state 16 with the countdown int[12000, 18000], stops the think loop's attack machine and plays animation 58 (110 ticks to the gate; the last key holds). The death watcher tests the fighter every tick; the dead test is true at once, so the win path starts on that tick: 1 s wait, the hero is grabbed and unequipped, 0.7 s fade-out (waiting), the fighter is deactivated (its death hook finds the countdown already running and changes nothing), the hero moves to locator 0, 0.7 s fade-in, camera 5, hero animation 182, and message 54, 55 or 56. If the hero dies instead (the death callback sets the marker 41 ticks after the callback starts), the hero-death path takes over wherever the fight is: the fighter's own killing strike sends it to class state 10 (animation 73, 140 ticks), which then falls through to class state 0 and back to state 8 (it walks home to its start point and hides), and the cutscene grabs it, places it on locator 2, and plays animation 22 before message 61, 60, 59, 58 or 57 for the round (round 3 uses message 58).

### Edge cases for implementers

- The fighter does not fight on release. With the hero idle at the arrival point it stays in state 8 playing animation 9 until the hero moves (walk gives 60 units) or hits it.
- Emerge takes 182 ticks from the first think tick; there is no attack during it, and it can be damaged during it (the hit skips the rest of the emerge).
- Chooser strictness: "roll below chance". Chance 100 passes 99 times in 100; use int[1, 100] and a strict compare.
- Each slot has only its strike clip (and a blocked clip). Wind-up, follow-through and recovery ids are absent; the sequencer skips them in the same tick. Do not substitute clip 0.
- The speed word 0.7 applies only while the sequencer is in the strike state (slot 2). The tick the sequencer starts uses the wind-up word (1.0). After the first contact the sequencer leaves the strike state and the clip plays at 1.0.
- Hit list: one hit per swing, cleared when the next attack starts.
- Because only class state 16 (not base state 16) marks the dead fighter, it keeps its body and animation 58 held on the last key; the script removes it by deactivating it.
- Cooldown draws are made twice (at the choice and at the end path). Cooldowns do not count while the hero is hurt, being knocked back, talking, or otherwise busy.
- Hit reaction restarts on every hit, so a fast combo keeps the fighter locked in animation 57 (65 ticks each, 5.7 units of push-back per hit).

### What a port most likely gets wrong

1. Starting the fight on release (the fighter waits for the hero within 28 units).
2. Giving the fighter a wind-up (it has none) or substituting clip 0 for missing phases, which delays every attack by a full clip cycle.
3. Using "chance 100 = always" or an inclusive compare, and testing slots 0, 1, 2 instead of 2, 1, 0.
4. Playing slot 2 at a constant rate: it is 0.7 until the first contact (or the full 74 ticks) and 1.0 after.
5. Treating the lunge as a short reach: its range value is 50, so it begins from up to 10 units away and moves the fighter about 9.6 units.
6. Not restarting animation 57 on every hit, or letting a hit not cancel the swing.
7. Letting it rise again or never mark it dead: for the arena script it is dead immediately.
8. Forgetting that a swing from behind a guarding fighter does nothing at all (no damage, no reaction).

### Test vectors

Vector A (hero idle, 20 units from the fighter on its facing line, hero never acts). Tick numbers count from T0, the first think tick after release.

| Tick | Event |
| --- | --- |
| 0 | State 8 sub 0: distance 20 < 28, sub 1 |
| 1 | Request animation 10; sub 2 |
| 181 | Gate passes (180 ticks after the request); setter 19 |
| 182 | Outer 0 stores outer 1 (hold) |
| 183 | Inner 0; request 21; chooser first runs (d 20, no slot eligible) |
| 184 | AI sees d = 20.00; the first root step is applied after the AI of this tick |
| 185 | AI sees d = 19.75; each later tick -0.25 |
| 225 | AI sees d = 9.75 (below 10; at tick 224 it is exactly 10.00, which fails the strict test): slot 2 chosen with 99 % per tick (slot 1 39 %, slot 0 59 % if the slot 2 roll fails); the sequencer runs state 0 in this tick and state 1 in the next |
| 226 | Strike clip 44 requested (this tick runs at the wind-up word 1.0, later ticks of the strike at 0.7) |
| 255 | Window opens (tick 29 of the swing); the lunge has moved about 1.8 units, so a hero 9.75 units away is still out of reach |
| 261 | The weapon first touches that hero, 35 ticks after the request (Medium: SPEC-0018 weapon-versus-body test); the hit is 50 damage before armour; the pass of tick 261 still runs at 0.7, later passes at 1.0 |
| 289 | Gate passes (63 ticks after the request for contact at 35) |
| 290 | End path; recovery int[15, 30]; the slot 2 cooldown restarts at 180 |
| 305 to 320 | Next choice possible: slot 1 (39 %), else slot 0 (36 %), else none (25 %) per tick; the fighter stands at about 3 to 4 units after the lunge |

In the first 600 ticks after the agro entry, with every swing connecting and the hero always attackable: 7 to 8 swings, start-to-start spacing 70 to 111 ticks (mean 85), slot use about 39 % slot 2, 31 % slot 0, 30 % slot 1. The first normal choice happens at tick 225 to 226 for a hero 20 units away, and at tick 183 to 184 for a hero already inside 10 units.

Vector B (hero attacks). Tor, normal swing, lands a hit on the fighter at tick H while it is in the rush state: damage is applied; outer 10 kind 0 starts and animation 57 restarts at once; the fighter is pushed back about 5.7 units over the next 65 ticks; if H falls inside a swing, that swing ends (no recovery countdown), the slot's cooldown keeps its drawn value, and the fighter returns to outer 0 at tick H + 65 (gate) and to the rush state one tick later. A second hit at tick H + 30 restarts 57 again, so the fighter leaves the reaction at H + 95. If the hero's swing starts while the fighter is in outer 1 within 8 units and in front, the fighter rolls once: 50 %, 35 % or 10 % to block (by swing class), then it plays 56 until the swing ends and blocks every contact whose facing dot is below 0.707. A killing blow at tick K: class state 16 at K, the watcher sees it at K, and the win line runs 60 ticks later.

### Open questions

1. Hit geometry. The exact placement of the weapon-1 cylinder (attachment frame 0 anchor and axis) decides the first contact tick of each swing. Resolved by the general rule (SPEC-0064, Shared field behaviour: Weapon contact and the hero's hurt shape); the distances are in the Weapon reach table of this round; stand-in only for a port without attachment frames: count a contact on a window tick when a level hero is inside the "at the tick" range of the slot's row (2.8 behind to 6.0 ahead for slot 0, 0.6 to 5.6 ahead for slot 1, 1.6 to 5.6 ahead for the lunge).
2. Whether the sequencer's state 0 runs in the tick of the choice or the tick after. Resolved (High; proactive (open close-out)): the same tick; the normal choice starts the sequencer in state 0 and the outer switch of that tick runs it (SPEC-0038 Behavior 50), so the offsets in this section stand.
3. The facing, route and sight tests assume nothing on the ring blocks the line to the hero; if a scene object does, the lost-sight search (outer 9) would start. Default if unresolved (rewrite choice): never blocked, because both ends of the line lie inside the ring wall (needs a runtime check).
4. How many think ticks run between creation and the script's grab (zero or one). Runtime-only (needs a runtime check); no visible difference for the fighter, whichever it is. Default as for the Rogue's hand-off.
5. Whether the hero's talk state is cleared before the fighter's first think tick. Resolved (High; proactive (open close-out)): yes. The release, the return to the normal camera (which puts the hero in action state 1) and the idle animation are one script step at the end of a tick, and the fighter's first think runs on the next tick before the hero's update, so F1 = 0.7 (Start sequence, step 4).

## Round 4: Werelupe Elite

Pack 1176, round 4 (flag 384 set). Opponent entity 16, loaded from region 1183. Intro animation 68, victory animation 73. Confidence: High for the identity, row values, clip data and phase tick counts; Medium-High for the start sequence and the charge geometry; Medium for the melee volume geometry.

### Identity and tunables

| Item | Value |
| --- | --- |
| Character record | region 1183, record 0, name "Werelupe Elite", entity 16 |
| Monster type, variant, row | type 99, variant 2, row 242 (resource 530) |
| Class, approach routine | class 17, approach 4 (strafe) |
| Hit points, radius, height, weight | 110, 1.175, 3.5, 5.5 |
| Difficulty (attack-manager weight), alignment, hostile, display in default | 12, 1 (target type 1: Mote elements 4, that is items 162 and 163, add their element bonus; Nova adds it too), 1, 1 |
| Reanimation | none; death disappear timer 300 |
| Damage reduction, anger, can fly | 0, 0, 0 |
| Spawn record | position (205.60, 27.92, 322.52), facing (-0.93, -0.06, -0.36); default-state word 8; emerge distance 40.0; 0xC4 = 1; 0xC8 = 0; alarm group none; drop ordinal -1; no carried Neopoints |
| Weapon volumes | all five: radius 2.0 x full length 2.0 (weapon 1 = event bit 0x1, weapon 2 = 0x2, weapon 3 = 0x4) |
| Strafe band (row 0x4F4, 0x4F8) | 10.0 to 20.0; desired distance = 10 + 10 x f, f = int[20, 80] / 100: 12 to 18, redrawn after every normal choice |
| Recovery after an attack | int[15, 60] ticks |
| Melee defence | block chance quick / normal / heavy 100 / 75 / 50; dodge chance 75 / 40 / 30 (tested only when the block roll fails); missile defend time 60; missile dodge 25, missile block 75 |
| Defence timer after a defence | int[60, 240] |
| Blocking Angle | 0.924 |
| Backbiter scale | 1.0 (no effect) |

Attack slots (slot 3 unused). Only the listed phases exist; the rest are skipped.

| Slot | Band (strict) | Chance (per tick) | Arc | Melee range | Cooldown draw | Speed words | Damage | Knockback | Flags | Charge | Counter code / chance | Clips |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| 0 | 0 < d < 10 | 60 (59 %) | 0.9239 | 5.0 | int[30, 60] | 1.0 / 1.0 / 1.0 | 35 | 0 | none | none | 3 (after own block) / 25 | A none, B 34 = clip 8 (15 keys), C none, D 36 = clip 26 (17 keys), E none |
| 1 | 10 < d < 50 | 25 (24 %) | 0.966 | 3.5 | int[60, 120] | 1.0 / 1.0 / 1.0 | 35 | 4.0 | none | kind 2, distance float[10, 50) | none | A 38 = clip 27 (10 keys), B 39 = clip 41 (8 keys, loop), C 40 = clip 28 (7 keys), D 41 = clip 26, E none |
| 2 | 0 < d < 10 | 40 (39 %) | 0.707 | 5.0 | int[30, 120] | 1.0 / 1.0 / 1.0 | 40 | 0 | 0x1 (unblockable) | none | 2 (after own dodge) / 25 | A none, B 44 = clip 7 (28 keys), C none, D 46 = clip 26, E none |

All clips have speed byte 2 (0.25 keys per tick).

### Animation ids used

| Use | Id | Clip, keys, loop | Gate (ticks) | Root travel |
| --- | --- | --- | --- | --- |
| Hide pick | 9 | clip 32, 22 keys, loop | none | none |
| Emerge | 10 | clip 33, 27 keys | 104 | none |
| Strafe forward (gait 0, 1, 2) | 28 | clip 0, 11 keys, loop | 43 per cycle | +0.126 per tick (7.56 per second) |
| Back off (gait 3) | 32 | clip 11, 8 keys, loop | 31 per cycle | -0.393 per tick (23.58 per second) |
| Circle left (inner 8, gait 6) | 30 | clip 13, 10 keys, loop | 39 per cycle | -0.0673 per tick sideways |
| Circle right (inner 9, gait 5) | 29 | clip 12, 10 keys, loop | 39 per cycle | +0.0673 per tick sideways |
| Hold (inner 10) | 31 | clip 24, 18 keys, loop | | none |
| Hit reaction kind 0 | 57 | clip 4, 9 keys, one-shot | 33 | -3.47 in total |
| Melee dodge (outer 5) | 55 | clip 11, 8 keys, loop | 31 | -12.58 in total (a backward hop) |
| Block (outer 7) | 56 | clip 10, 15 keys, loop | 59 | none |
| Block hit flash | 126 | clip 42, 9 keys | 33 | -0.74 |
| Missile dodge (outer 6) | 53 or 54 (int[0, 1]) | clip 29, 13 keys (both ids) | 49 | +5.99 to the side |
| Victory pose (state 10 and arena after a hero death) | 73 | clip 40, 40 keys | 156 | none |
| Arena intro | 68 | clip 31, 43 keys | 168 | none |
| Death | 58 | clip 3, 20 keys | 77 | none |
| Slot 1 charge wind-up (A) | 38 | clip 27, 10 keys | 37 | +0.92 |
| Slot 1 charge run (B) | 39 | clip 41, 8 keys, loop | 31 per cycle | +0.469 per tick (28.14 per second) |
| Slot 1 charge hit (C) | 40 | clip 28, 7 keys | 25 | +3.05 |
| Slot 0 strike | 34 | clip 8, 15 keys | 57 | +4.33 |
| Slot 2 strike | 44 | clip 7, 28 keys | 108 | +12.36 |
| Blocked strike (D) | 36, 41, 46 | clip 26, 17 keys | 65 | none |

### Arena start sequence

1. The encounter script loads region 1183 into the round slot. The fighter (entity 16) is created with the spawn-record default state 8 and is under script control (base state 33) from the first tick, so its think loop does not step until it is released. At most one think tick can run before the grab; in it the fighter only enters state 8 sub-state 0, and the hero at the arrival point (30.19 units away) is outside the emerge range, so the outcome is the same (Medium).
2. The script moves the fighter to locator 2 (205.60, 27.92, 322.52), plays camera 4 and animation 68 with wait (168 ticks to the gate; a standing pose with no root travel), then the entity returns to its idle animation and stays in script hold. Camera 5, hero animation 182, camera 7 and message 49 follow (the message waits for the player to close the dialogue box). The hero is on locator 0 (177.66, 27.92, 311.07).
3. The script releases the fighter (base state 1), returns the camera to normal and puts the hero in state 1 (idle), so at release the hero's action state is 1.
4. First think tick T0 (the tick after release). State 8 sub 0: the fighter emerges when the hero is closer than 40.0 x F1 (28.0 for an idle hero, 30.0 slow walk, 60.0 walk, 70.0 run, 40.0 in any other hero state). The hero's arrival point is 30.19 units away, so an idle hero does not wake it. It stays at home on animation 9 and is hittable.
5. Emerge: tick T0 sub 1; tick T0 + 1 requests animation 10, sub 2; the gate passes in tick T0 + 1 + 104 = T0 + 105 and the setter runs with 19 (entry work as in round 3).
6. Agro tick 1 (T0 + 106): outer 0 stores outer 4 (strafe) with inner 10, hold 0. The normal choice first runs in tick T0 + 107.
7. Tick T0 + 107: inner 10 with hold 0 rolls r = int[1, 3] and sets the hold to int[60, 180], then the distance tests win: d above 20 gives inner 7 (approach, animation 28); d between 10 and 20 stays in the hold or circles (r = 2 gives inner 8, r = 3 gives inner 9, r = 1 keeps inner 10); d below 10 gives inner 11 (back off).

### Per-tick order, state table and movement

Per-tick order (agro): pre-step (pause test, awareness, alert flag, retarget, alarm check, class switch), then the agro machine: recovery countdown down by one; melee defence and missile defence (hold below 1); lost-sight test; normal attack choice (hero attackable; cooldowns count only then); counter-attack choice; distance-hide gate (flag 0, never fires); outer sub-state switch (strafe states or the sequencer); route flag and leash. Then the class handler (defence hold down by one, hold animation or pursuit step), the animation pass and the motion pass. The AI reads the cursor left by the previous animation pass.

State summary: scripted intro (68) -> class state 8 (sub 0 hide on animation 9, sub 1 request 10, sub 2 wait 104 ticks) -> agro outer 4 (strafe inner states below) -> outer 2 (slot swings or the charge) -> outer 5/7 (melee dodge 55 / block 56 and 126), outer 6/8 (missile dodge 53 or 54 / block 56), outer 10 (hit reaction 57, 33 ticks) -> class state 10 (animation 73, after the Elite's strike leaves the hero at 0 HP; then class 0 and back to state 8: it walks home and hides) -> base state 16 on death (animation 58, 77 ticks). The strafe outer sub-state 4 has these inner states:

| Inner | Meaning | Animation | Exits |
| --- | --- | --- | --- |
| 10 | hold, faces the hero | 31 | hold counts down; at 0 roll r: 1 stay (hold int[60, 180]), 2 circle left (inner 8, stay if locked), 3 circle right (inner 9, stay if locked); then the distance tests below apply every tick |
| 7 | approach (gait 0, steer to the hero) | 28 | d below the desired distance: hold 0, inner 10 |
| 8, 9 | circle: steer to own position - 2.0 x p (left) or + 2.0 x p (right), p the right axis | 30 / 29 | hold down by one; below 1 inner 10; blocked: inner 10 and strafe lock 120 |
| 11 | back off (gait 3, aim point hero - dir x 20.0) | 32 | d above the desired distance or blocked: hold 0, inner 10 (blocked also sets the lock to 120) |

Distance tests (applied from inner 8, 9 and 10): d above 20.0 gives inner 7; else d below 10.0 with the lock below 1 gives inner 11. The lock counts down one per tick. The fighter faces the hero in the hold and while circling; when approaching it faces its move direction; backing off faces the hero (facing = -move).

Attack chooser (shared rules; slot order 2, 1, 0; all cooldowns 0 on agro entry): with d between 10 and 50 only slot 1 is eligible (a route must exist, facing cosine above 0.966): 24 % per tick, so about 4 ticks after the first choice tick on average (tick T0 + 107 + G, G geometric with p = 0.24, P(G = 0) = 24 %, P(G <= 10) = 96 %). With d below 10 slot 2 (39 %) is tested first, then slot 0 (59 % of the remainder): 39 %, 36 % and 25 % none. For slots 0 and 2 (range 5.0) with d above 5.0 the sequencer walks forward on animation 28 (0.126 per tick) until d is 5.0 or less, then starts the strike; the Elite does not stop to back off while a melee slot is selected.

Charge (slot 1, kind 2). Sequencer state 6 starts in the choice tick c: A (id 38) requested at c; its gate passes in tick c + 37; B (id 39, loop) is requested in the same tick and state 7 runs from c + 38. The charge distance used is a fresh float from [10, 50) drawn in the choice tick itself (the redraw after the choice happens before the sequencer first reads it; only slots that have a charge type get a draw, and the draw uses the range of the slot just chosen). Each tick of state 7: travelled distance t is measured from the start point; if the hero is within 3.5 units, state 8; else if t is at least the charge distance, state 12. There is no strike test during the run (kind 2), no chase stop (it only applies in states 0 to 3), and the run is 0.469 units per tick. State 8: C (id 40) requested; the weapon-3 window is keys 1 to 3, ticks 4 to 15 after the request, one hit (35 damage, knockback 4.0 which puts the hero into the long-fall state 48); C finishes at tick 25 after the request; state 12 next tick; E absent: end path in that tick. A charge that ends by distance without reaching the hero costs the fighter the whole cycle (37 + travel ticks) and ends with the end path (recovery int[15, 60], slot cooldown int[60, 120] again).

Movement table (movement pass; row 242, the Werelupe rig of SPEC-0065; speed byte 2):

| Mode | Source | u/t | u/s | Turn | Clip (id: clip, keys, loop) | Exits |
|---|---|---|---|---|---|---|
| Walk home (after the victory pose) | root | 0.1259 average; per key 0.105 to 0.146 | 7.56 | steering 0.3 / 0.1 | 2: clip 0, 11, loop | home reached |
| Strafe approach (inner 7) and melee approach (sequencer state 0) | root | 0.1259 | 7.56 | steering | 28: clip 0 (the walk clip) | d below desired (12 to 18) or 5.0 |
| Circles (inners 8, 9) | root sideways | 0.0674 | 4.04 | faces the hero (the travel goes to the side opposite to the steering point, SPEC-0038 Behavior 51) | 30: clip 13 (-x); 29: clip 12 (+x); 10, loop | hold count, wall |
| Back off (inner 11) | root backward | 0.393 | 23.58 | faces the hero | 32: clip 11, 8, loop | d above desired; point 20 from the hero |
| Charge run (state 7) | root | 0.469 | 28.14 | none; solid | 39: clip 41, 8, loop | hero within 3.5 or travelled 10 to 50 |
| Charge hit (state 8) | root | 3.05 units over the clip's 28 ticks (gate 25) | 6.54 | none; solid (pushes a hero it overlaps) | 40: clip 28, 7 | clip end |
| Melee dodge 55 | root backward | about 12 units over about 31 ticks | 23.58 | none | clip 11 (the back-run loop) | gate |
| Missile dodges 53, 54 | root sideways | 5.99 units over the clip's 52 ticks (gate 49) | 6.91 | none | clip 29 and its mirror | gate |
| Hit reaction kind 0 | root backward | 3.47 units over the clip's 36 ticks (gate 33) | 5.78 | snapped to the attacker | 57: clip 4, 9 | gate |

Worked example (back-off). After a three-hit swing the Elite stands about 4 units from the hero; d below 10 sends it to inner 11 and it sprints backward at 0.393 per tick: with desired 15 it needs 11 units, 28 ticks, then holds (one more step after the decision, so about 15.4).

### Hit/hurt windows (tick 0 = the tick the sequencer requests the strike)

| Slot | Windows | Hits | Gate | End path | Damage |
| --- | --- | --- | --- | --- | --- |
| 0 (clip 8, weapon 1) | keys 5 to 7: ticks 20 to 31 | 1, at tick 20 or the first contact tick | 57 | 58 | 35 |
| 2 (clip 7) | weapon 1 keys 7 to 10: ticks 28 to 43; weapon 2 keys 13 to 14: ticks 52 to 59; weapon 1 keys 19 to 20: ticks 76 to 83 | up to 3: first contact in each window; for the first window, an idle level hero straight ahead at start distance 3.0–4.5 touches at tick 28 and at 5.0 touches at 29 (exact geometry, no chase stop). Tick 53 and 77 remain the 3–5-unit reference only when later windows are reached at their expected separations; moving-target/push effects can change them. | 108 | 109 | 40 each, unblockable, 120 in all |
| 1 charge hit (clip 28, weapon 3) | keys 1 to 3: ticks 4 to 15 after the C request | 1 | 25 | next tick | 35 (+ knockback 4.0) |

If the first contact of slot 2 comes late, later windows still hit independently (contact at tick 52: hits at 52 and 77; contact at 70: one hit at 76). The enemy strike test does not consult the hero's hittable state, so the second and third hits of slot 2 land on a hero already in the hurt state.

### Weapon reach

Conventions are those of "Weapon reach tables" in SPEC-0064 (shared field behaviour). Row 242, Medium. The hit ticks above are authoritative (they are for a hero straight in front at the start distance); "sC" is the tick the charge-hit clip is requested. Weapon radius and height are 2.0 by 2.0 in every row.

| Slot (phase, clip) | Weapon | Window ticks (level contact) | Centre forward / up / side | Tor ahead, at the tick | Tor ahead, from the start | Tor feet vs origin |
|---|---|---|---|---|---|---|
| 0 jab (B, clip 8) | w1 | s+20 to s+31 (s+20 to s+31) | 0.4 to 2.5 / 1.6 to 3.0 / -1.6 to 1.9 | -1.4 to 5.0 | 1.9 to 8.1 | -2.0 to 3.5 |
| 1 charge hit (C, clip 28) | w3 | sC+4 to sC+15 (sC+4 to sC+15) | 2.0 to 3.0 / 1.7 to 2.7 / -0.2 to 0.0 | 0.8 to 4.8 | 1.9 to 8.2 | -3.5 to 4.5 |
| 2 three-hit swing, first (B, clip 7) | w1 | s+28 to s+43 (s+28 to s+43) | 0.2 to 2.9 / 1.2 to 3.0 / -2.1 to 3.0 | -1.0 to 5.4 | 1.7 to 9.0 | -2.0 to 4.0 |
| 2 three-hit swing, second (B, clip 7) | w2 | s+52 to s+59 (s+52 to s+59) | 0.9 to 2.4 / 2.2 to 2.6 / -2.2 to 2.5 | 0.2 to 5.0 | 7.0 to 11.9 | -2.5 to 4.0 |
| 2 three-hit swing, third (B, clip 7) | w1 | s+76 to s+83 (s+76 to s+83) | 0.3 to 2.1 / 0.8 to 4.8 / -1.6 to 0.0 | -0.8 to 4.8 | 9.1 to 16.3 | -4.0 to 6.5 |

Every blow reaches 4.8 to 5.4 ahead at the tick; the three-hit swing carries the Elite forward, so the second and third blows land 7 to 16 units ahead of where it started. The same figures hold for the Werelupe Basher and Bone Thrower (SPEC-0065).

### Defence and counter rules

- Melee defence: evaluated once per hero swing at its start edge, when the defence hold is below 1, the hero is within 8.0 units and the hero faces the fighter within 60 degrees, and the fighter's sequencer is in state 0, 1 or 5 (or not attacking). Roll r1 = int[1, 100]: at or below the block chance blocks (outer 7: quick 100, normal 75, heavy 50). If the block fails, roll r2 = int[1, 100]; below the dodge chance dodges (outer 5, 75, 40, 30). With Tor's first two swings (normal class): block 75 %, dodge 25 % x 39 % = 9.8 %, no reaction 15.3 %; finisher (heavy): block 50 %, dodge 14.5 %, none 35.5 %; chained variant (quick): block 100 %.
- Dodge (outer 5): plays 55 (31-tick gate; a 12.6 backward hop over 8 keys); not hittable while outer 5 or 6; then the defence exit (outer 0, base state 1, hold int[60, 240]). Block (outer 7): plays 56 with base state 20 for the length of the swing; blocked swings need the facing dot below 0.924 (in front of the fighter); the block-hit flag plays 126 once.
- Missile defence (Roberta's shots): if a shot would meet the body within 60 ticks, block roll below 75 (74 %), dodge roll below 25 (24 %); block wins. Dodge plays 53 or 54 (49 ticks, moves 6 units sideways); block plays 56.
- Counters: slot 0 after its block (outer 7 or 8) while the hero's swing is in its recovery window, roll int[0, 100] below 25; slot 2 after its dodge (outer 5 or 6) with the same window, roll below 25. The last eligible slot of the walk wins; it needs its cooldown below 1, its band and its arc.

### Hero actions against this fighter

| Hero action | Result |
| --- | --- |
| Tor swing on a hittable, non-guarding Elite | damage product minus 0; take-hit hook: outer 10 kind 0, animation 57 restarted (33 ticks per hit, 3.5 units of push-back), swing cancelled, face the attacker |
| Tor swing while the Elite blocks (outer 7) from the front | blocked: no damage, block effect, landed; no reaction |
| Tor swing from behind a blocking Elite (dot at least 0.924) | nothing |
| Tor swing while the Elite dodges (outer 5, 6) | not hittable: passes through |
| Power step (sub-state 10 with a Mote) | skips the block test; element bonus when the Mote element is 4 (items 162, 163) or Nova |
| First hit before agro (state 8) | doubled damage once, skips the emerge |
| Roberta shots | missile defence rolls (74 % block, 6 % dodge only) |
| Hero guard | blocks slots 0 and 1 (not slot 2, which is unblockable) when face to face; damage minus shield stat |
| Hero standing still | the Elite hides beyond 28 units; inside it emerges, strafes at 12 to 18 units and charges within a few ticks (slot 1) |
| Hero approaching within 10 | the Elite backs off at 0.393 per tick while its chooser picks slot 2 or 0 at 75 % per tick; the chosen slot then walks it forward at 0.126 per tick to 5.0 units before the strike |

### Death and the arena win path

Death uses the ordinary death hook: base state 16, death clip 58 (77 ticks); the corpse lasts 300 ticks counted from clip progress 0.95, sinks 1 unit per 180 ticks after the first 30 of them, and is then released. The death watcher sees the kill at once (class dead test); the win path then runs exactly as in round 3 (message 54, 55 or 56). If the hero dies, the cutscene moves the Elite to locator 2 and plays animation 73 before message 57 for round 4 (the lose message for round 4 is 57).

### Edge cases for implementers

- No fight on release: the Elite waits until the hero is within 28 units (idle) or moves nearer or hits it; the emerge is 106 ticks long.
- The charge distance is drawn fresh at the choice from [10, 50): a charge stops short of a farther hero.
- Slot 2's three windows hit independently and the third window can hit while the hero is down.
- The Elite never has a wind-up for slots 0 and 2: the strike clip is requested one tick after the choice.
- The charge has no strike test during the run, so a hero who steps out of the line is missed; a hero who guards facing it blocks the hit (the strike test of state 8 takes its blocked path: reduced damage, block reaction), but the sequencer ignores the result for kind 2: no state 9, C plays to its end and state 12 follows.
- Cooldowns freeze while the hero is hurt or knocked back.
- Defence hold: after any defence the Elite cannot defend again for 60 to 240 ticks.

### What a port most likely gets wrong

1. Starting the fight on release, or skipping the 106-tick emerge.
2. Missing that the Elite is a strafer: it holds 12 to 18 units from a standing hero, backs off below 10, circles with 60 to 180 tick segments, and uses slot 1 only from 10 to 50 units.
3. Reading slot 1's charge distance as the trigger range; it is a separate draw from 10 to 50.
4. Making slot 2 one hit; it is up to three, each 40, unblockable, and the third can land after the hero is already hurt.
5. Playing clip 0 for missing phases (slots 0 and 2 have only B and D).
6. Block chance 100 for quick swings and the "from behind does nothing" rule; also blocks/dodges once per swing, not per tick.
7. Treating the dodge animation 55 as in place: it hops 12.6 units backward.

### Test vectors

Vector A (hero idle 25 units away on the Elite's facing line). T0 = first think tick after release.

| Tick | Event |
| --- | --- |
| 0 | State 8 sub 0, 25 < 28: sub 1 |
| 1 | Request animation 10 (104-tick gate) |
| 105 | Gate passes; setter 19 |
| 106 | Outer 0 stores outer 4, inner 10, hold 0 |
| 107 | First chooser tick: slot 1 eligible (d 25 in 10 to 50, facing ok): 24 % per tick; also inner 10: roll r, d above 20 gives inner 7, request 28 |
| 107 + G | Charge chosen (G geometric, p 0.24, mean 3.2): A requested; hero 25 - 0.126 G units away |
| 144 + G | A gate passes; B requested; state 7 begins at 145 + G |
| 145 + G to about 191 + G | Charge run at 0.469 per tick; reaches the 3.5-unit test after about 46 ticks if the drawn distance is at least 21.5 - 0.126 G (probability about 71 %); otherwise stops after t reaches the drawn distance (10 to 21 units: 21 to 45 ticks) |
| about 191 + G | State 8: C requested; window ticks 195 + G to 206 + G; hit 35 damage and knockback 4.0 |
| about 216 + G | C gate; state 12 next tick; end path at about 217 + G; recovery int[15, 60]; slot 1 cooldown int[60, 120] |

Vector B (hero standing toe to toe, d at most 5, always attackable, every swing connecting): slot 2 or 0 at the first chooser tick T0 + 107 (slot 2 39 %, slot 0 36 %, none 25 % per tick); slot 0 swing hits at +20 and ends at +58; slot 2 swing hits at +29, +53, +77 and ends at +109; over 600 ticks 4 to 6 swings (mean 5.4), start-to-start spacing 74 to 174 ticks (mean 120), slot 0 and slot 2 used about equally.

Vector C (hero attacks). A Tor hit at tick H: animation 57 restarts (33-tick gate), any swing or charge is cancelled, outer 0 at H + 33, strafe resumes. The block roll of a hero swing that starts at tick S with the Elite within 8 units, in front and with the hold at 0: normal swing 75 % block (outer 7 at S; the hero's hits during it deal 0), 9.8 % dodge (outer 5 at S, hop back 12.6 units over 31 ticks, not hittable), 15.3 % neither. After the defence ends the hold is int[60, 240] ticks.

### Open questions

1. Hit geometry (same as round 3). Resolved by the general rule (SPEC-0064, Shared field behaviour: Weapon contact and the hero's hurt shape); the distances are in the Weapon reach table of this round; stand-in only for a port without attachment frames: count a contact on a window tick when a level hero is inside the "at the tick" range of the slot's row (1.4 behind to 5.0 ahead for the jab, 0.8 to 4.8 ahead for the charge hit, up to 5.4 ahead for the three-hit swing).
2. Whether sequencer state 0 or 6 runs in the choice tick. Resolved (High; proactive (open close-out)): the same tick, as in Round 3 question 2; the offsets stand.
3. Resolved: in state 8 a kind-2 charge runs the ordinary strike test each tick, so a guarding hero facing it blocks the hit (reduced damage, block reaction, the hero enters the hit list), but the sequencer does not act on the result: it never goes to state 9, C plays to its end and state 12 follows (High).
4. How the circle segments interact with the ring wall. Resolved (High; proactive (open close-out)): a move into a mesh border edge sets the blocked flag and a 120-tick timer (SPEC-0056, Navmesh confinement step), which is the lock; the ring mesh edge is the block.
5. Whether a hero-state change between release and the first think tick changes the emerge test. Resolved (High; proactive (open close-out)): no; the hero's state is set in the release step and the first think runs before the hero's next update (Round 3 question 5).

## Distances and ranged reach (all five fighters)

Confidence: High for the row values and the rules; Medium-High for the emit point and the hit ticks of Sludgy's lob (posed clip, flat ground, hero standing still and level).

**Reading the tables.** d is the distance from the fighter to the selected hero, as the agro machine measures it. The notice band is the row's near and far range; a hero standing still scales both by 0.7 and a running hero by 1.75 (SPEC-0038 Behavior 18); through a wall it is scaled by F2 instead (0 for a standing hero) unless the row sees through walls. The sight cone only halves the notice chance outside it; it is not a hard edge. "Walk-in target" is the drawn desired distance (min + (max - min) x f, f from 0.2 to 0.8), redrawn after every normal attack choice. A slot is chosen only while its band holds strictly and the hero is inside its arc; a melee slot then walks in until d is at most its close-in distance. Shared by every fighter and not repeated in the tables: holds and circles face the hero every tick with the row's turn weight; the strafe circle steers to a point 2.0 to the side every tick, so it keeps roughly the current d and drifts outward; the only leash is 181 ticks with no route while the walk waypoint is 15.0 or more from the hero (in the closed arena ring a route always exists, so an arena fighter never gives up); the lost-sight search goes to the last-seen point ("last seen + x" means moved x further along the fighter-to-hero line) and gives up after 121 ticks there; the chase stop is the fighter's radius + 1.175 (the hero's body radius, both heroes); a confused fighter may attack at radius + 1.425.

**Sludgy's lob (slot 0, the only missile of the five).** Fires when 10 < d < 100, anywhere in front, at 74 percent per tick. One lob per attack, released on s + 84 (s = the tick the strike clip B is requested, c + 1), from emit point 0, which is 1.60 to one side, 2.54 up and 3.12 in front of Sludgy's feet on that tick. Flight mode 1: a gravity arc aimed at the hero's feet plus a horizontal scatter of up to 2.0 units, launch speed ramped from 0.5 per tick (at d = 10) to 1.0 (at d = 100), no lead, no homing. Radius 0.2 against the hero's body cylinder (radius 1.175, height 3.5, both heroes). It ends on the hero, on a guard (a quarter of 20), or on the ground; its area values are 0. Worked example, hero standing still and level, scatter 0: at 11 units it hits on s + 97, at 30 on s + 122, at 60 on s + 146, at 99 on s + 178. With the scatter the lob misses a standing hero whenever the sideways part of the scatter exceeds about 1.3 (11 units), 1.1 (30 units) or 0.9 (60 units), so about half of the far lobs land beside him.

**Henchman (row 160) slot 1 never connects.** Its trigger chance is 0 and its counter code is -1, so neither the normal choice nor the counter choice can select it; it is never started. If a port forced it, its charge distance range is 0 to 0, so sequencer state 7 would end on its first tick and the jab windows of the strike clip would still never be reached. A port must not give it a reach.

#### Rogue (row 153)

| Distance (units) | Value |
|---|---|
| Notice band (x0.7 standing, x1.75 running) | 10 to 60 (7 to 42; 17.5 to 105) |
| Sight cone half-angle | 22.5 deg |
| Strafe: circle band; walk-in target; back-off point | 6 to 20; 8.8 to 17.2; 20 |
| Slot 0 | melee, 0 < d < 10, within 22.5 deg, closes to 5 |
| Slot 2 | melee, 5 < d < 20, within 22.5 deg, charge run 15 to 25 (ends early when d <= 5) |
| Slot 3 | melee, 5 < d < 15, in front, closes to 5 |
| Lost-sight search point | last seen + 6 |
| Wait-in-line square | 10 |
| Map search distance; greedy range | 100; 10 |
| Chase stop; confused strike | 3.17; 3.42 |

#### Ixi Henchman (row 160)

| Distance (units) | Value |
|---|---|
| Notice band (x0.7 standing, x1.75 running) | 5 to 40 (3.5 to 28; 8.75 to 70) |
| Sight cone half-angle | 45 deg |
| Strafe: circle band; walk-in target; back-off point | 10 to 20; 12 to 18; 20 |
| Slot 0 | melee, 0 < d < 10, within 22.5 deg, closes to 4 |
| Slot 1 | never chosen (chance 0, no counter); band 7 < d < 12 within 45 deg, charge run 0 to 0 |
| Slot 2 | melee, 0 < d < 50, within 15 deg, charge run 10 to 50 (ends early when d <= 5) |
| Lost-sight search point | last seen + 10 |
| Wait-in-line square | 10 |
| Chase stop; confused strike | 3.17; 3.42 |

#### Sludgy (row 231)

| Distance (units) | Value |
|---|---|
| Notice band (x0.7 standing, x1.75 running) | 4 to 17 (2.8 to 11.9; 7 to 29.75) |
| Sight cone half-angle | 45 deg |
| Rush: hold band; walk-in target; back-off point | 3.5 to 5; 3.8 to 4.7; 7 |
| Slot 0 | missile, 10 < d < 100, in front (lob above) |
| Slot 1 | melee, 0 < d < 7, within 45 deg, closes to 5 |
| Slot 2 | melee, 0 < d < 15, within 22.5 deg, closes to 9 |
| Lost-sight search point | last seen |
| Wait-in-line square | 10 |
| Map search distance; greedy range | 100; 10 |
| Chase stop; confused strike | 3.17; 3.42 |

#### Skeleton Fighter (row 227)

| Distance (units) | Value |
|---|---|
| Notice band (x0.7 standing, x1.75 running) | 3 to 5 (2.1 to 3.5; 5.25 to 8.75) |
| Sight cone half-angle | 22.5 deg |
| Rush: hold band; walk-in target; back-off point | 3.5 to 5; 3.8 to 4.7; 7 |
| Slot 0 | melee, 0 < d < 10, within 45 deg, closes to 7 |
| Slot 1 | melee, 0 < d < 10, within 45 deg, closes to 7 |
| Slot 2 | melee, 0 < d < 10, in front, close-in distance 50 (starts at once) |
| Lost-sight search point | last seen |
| Wait-in-line square | 12 |
| Chase stop; confused strike | 3.17; 3.42 |

#### Werelupe Elite (row 242)

| Distance (units) | Value |
|---|---|
| Notice band (x0.7 standing, x1.75 running) | 1 to 100 (0.7 to 70; 1.75 to 175) |
| Sight cone half-angle | 45 deg |
| Strafe: circle band; walk-in target; back-off point | 10 to 20; 12 to 18; 20 |
| Slot 0 | melee, 0 < d < 10, within 22.5 deg, closes to 5 |
| Slot 1 | melee, 10 < d < 50, within 15 deg, charge run 10 to 50 (ends early when d <= 3.5) |
| Slot 2 | melee, 0 < d < 10, within 45 deg, closes to 5 |
| Lost-sight search point | last seen + 10 |
| Wait-in-line square | 10 |
| Chase stop; confused strike | 2.35; 2.6 |

**What a port most likely gets wrong (distances).** A leash radius (none exists while a route exists); a hard-edged view cone; aiming Sludgy's lob with lead or without the 2.0 scatter; testing the lob against the bone capsules instead of the body cylinder; giving the Henchman's slot 1 a reach.

## Test vectors

All tick counts are think-loop ticks (60 per second). "Wake tick" is the tick in which the Rogue's wake test first passes (tick 0). Random draws are written as ranges or as the outcome stated.

1. Hero stands still at the start (distance 30.19, action state 1). Wake distance 28. For 600 ticks: nothing. The Rogue stays hidden (class 8, sub 0), playing clip 0. No attack, no cooldown, no state change.
2. Wake thresholds (distance to the Rogue's home; strict "below"): hero standing, 27.9: wakes; 28.0: does not. Slow walk (state 2): 29.9 wakes, 30.0 does not. Walk (state 3): 59.9 wakes. Run (state 4): 69.9 wakes. Hero attacking (state 13, factor 1.0): 39.9 wakes. A hit on the hidden Rogue from any distance: wakes him, damage doubled.
3. Timeline from the wake, hero standing still 12 units in front, all slots off cooldown. Tick 0: wake test, sub-state 1. Tick 1: agro entry. Tick 2: outer 0 becomes outer 4 (strafe, hold). Tick 3: first possible choice. Example run (one random sequence): tick 3 no choice (hold roll, hold timer 92); tick 4 slot 3 chosen (cooldown 0), sequencer state 0, gait 0 starts; ticks 5 to 43 he runs 7 units to d = 5.0; tick 44 state 0 ends (d 4.97), state becomes 1; tick 45 the strike clip 38 is requested and state 2 starts (s = 45); tick 61 window opens (key 4); tick 62 hit 1 (first touching tick of an idle level hero in front); tick 85 window reopens, tick 86 hit 2; tick 105 reopens, tick 106 hit 3; tick 125 the strike clip's gate is almost reached; tick 126 (s+81) the gate passes, state 5; tick 127 (s+82) end path: recovery 27, slot 3 cooldown 0; tick 128 outer 0 becomes outer 4; tick 154 (recovery over) the next choice. Next attack starts 83 to 159 ticks after the previous one began in a run of 1,500 seeds, median 119.
4. Ranges over many random sequences, hero standing still in front of him (a model of the rules above that assumes each strike window hits when d is 6.0 or less; 600 ticks after the wake test): first attack chosen: tick 3 to 13 at d = 6, 12 or 18, tick 36 to 45 at d = 25 (he walks in first). First hit: tick 28 to 53 (d = 6), 60 to 89 (d = 12), 115 to 125 (d = 18, slot 2 charge), 155 (d = 25). Hits landed in 600 ticks: 5 to 15 (mean about 9) at d = 6, 5 to 15 (mean about 9) at 12, 5 to 13 at 18, 4 to 11 at 25. Attack starts per run by slot: about 1.4 to 1.7 for slot 0, about 2.1 for slot 2, 1.5 to 2.1 for slot 3.
5. Slot 2 charge, hero standing 12 units in front: chosen at tick c (cooldown int[60, 120]); the same tick the sequencer starts the run (clip 40, state 7) with the charge distance drawn float[15, 25); the Rogue runs until the hero is within 5.0 (about 7 units, 39 ticks); then state 8 starts clip 39 at c0; window from c0+36; hit (damage 1, steal roll) at c0+38 (the hero is about 5 units ahead; no chase stop in the charge states); the gate passes at c0+81; end path c0+82. Roughly 120 ticks from choice to end path.
6. Melee defence. The hero (Tor) begins a normal swing (sub-state 2) in the hero update of tick h; the Rogue is 6 units away and in front, hold 0, in outer 4. At tick h+1 the latch is set; rolls r1 = 70, r2 = 20: block no (chance 0); dodge yes (20 < 25). Outer 5, hold 60, clip 15 requested at tick h+1 (s = h+1); unhittable from h+1; gate h+78; outer 0 at h+78, outer 4 at h+79. With r2 = 25 or more: no dodge and the latch stays set until the swing ends. With the hero at 8.0 units or more, or at a dot of 0.5 or less, no rolls are drawn at all and the latch is still set.
7. Hit reaction and counter. The hero's swing hits the Rogue (hold, d = 4, slot 0 cooldown 0, slot 3 cooldown 0) in the hero update of tick H. At tick H+1 outer is 10. Slot 3 is not eligible (d must be above 5); slot 0 is: roll int[0, 100] below 25 (probability 25/101) at H+1 starts slot 0 at once (outer 2, state 0; d <= 5 so state 1 next tick; B requested at H+2; window from H+22; hit at H+28; gate H+61; end path H+62). If the roll fails, tick H+1 requests clip 3 (s = H+1), and every following tick draws another roll; if none succeeds the reaction ends at H+54, outer 0, then outer 4 at H+55. The same hit with the hero at d = 8: slot 3 and slot 0 are both eligible; slot 3 wins; the swing starts at the first success (windows from s+16, s+40, s+60; hits at s+17, s+41, s+61).
8. First hit on a hidden Rogue: hero hit of base damage 4 before doubling: damage 8 (80 hit points to 72); the next hit at 4 (72 to 68).
9. Steal. Slot 2 hit on a hero with no body armour and 300 Neopoints: one roll int[0, 100]; 0 to 49 succeeds (49.5 percent); amount int[10, 100]; the amount is spawned as coins at the hero; on the next tick the Rogue's map search rolls int[1, 100] below 50 for each coin kind that exists, but the hero is within 10.0 so nothing starts. When the hero retreats beyond 10.0 the Rogue fetches the coins with 49 percent per tick per kind, deletes them, and walks home.
10. Death: the Rogue at 3 hit points takes a hit of 5; hit points 0; base state 16; clip 24 starts in the same tick; the arena watcher sees the dead test true at its next pass and starts its 1-second wait.

## Open questions

Default if unresolved (rewrite choices): see "Default if unresolved" in the Rogue section above (hidden start with the hero in action state 1, which is runtime-only and needs a runtime check; the facing blends and the horizontal distance are settled rules there). Weapon contact is no longer a rewrite choice (Rogue section, Weapon reach).

1. Resolved: weapon contact geometry follows the general rule (SPEC-0064, Shared field behaviour: Weapon contact and the hero's hurt shape); the first touching ticks are in the Rogue's Hit and hurt windows table and the distances in its Weapon reach table (Medium).
2. Intro order. Whether the Rogue's think loop takes a step between its binding and the script's grab, and in which hero action state. Runtime-only (needs a runtime check); the default is the hidden start (Rogue defaults).
3. Facing during circling and holding. Resolved (Medium-High; proactive (open close-out)): the facing follows the turn blends 0.9 in the hold, 0.1 or 1.0 for the side gaits and 0.3 for the others (SPEC-0038 Behavior 48); aimed at the hero (dot 1.0) is only the fallback for a port without the blends.
4. Hero distance metric of the wake and attack tests. Resolved (High; proactive (open close-out)): horizontal, with the height difference dropped, whether or not the hero is airborne.
5. One-tick request-to-step latency of root motion (SPEC-0038 Behavior 48). Resolved (High; proactive (open close-out)): fixed by the per-tick order (the animation pass stores the next root step and the next tick's motion applies it), so the latency is exactly one tick, as the tables already count.
6. Whether a charge-kind-2 strike clip adds the hero to the struck list per window, as the other melee slots do. Resolved (High; proactive (open close-out)): yes; the list is cleared at the start of each separate open window and holds one entry per target (Round 4 question 3 gives the blocked case).
7. Rounds 1 to 4: see the Open questions inside each round's section.
