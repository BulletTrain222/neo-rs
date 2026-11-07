# SPEC-0020: Boss behavior: Gnarfas

- Answers: proactive (open close-out); proactive (weapon box); proactive, none requested (boss AI); proactive (fight AI build guide); R-0064 (in part: how Gnarfas becomes active in pack 985; the full answer is in SPEC-0019); proactive (boss consistency pass 3); proactive (arena pass 4); proactive (timing pass); proactive (reach pass 3); R-0135 (the King's hold in this fight); R-0137; R-0145; R-0172 (the mode setter's handler path, a shot that bounces in the guard state); proactive (boss win check); proactive (open question sweep, round 12)
- Status: Draft
- Disc build: SCUS-97367 v1.00 (same fingerprint as SPEC-0001)
- Confidence: per section. Tuning values, the phase hub, the attack order and the cooldown rules: High. Damage taking (own hit check, flinch and exhaustion): High for the control flow, Medium-High for the hit test. Missile reaction: High for the rolls and timer; Medium-High for the missile contact shapes (bone cylinders for the reaction, body cylinder for the hit). Wait state never entered: Medium-High. Melee reaction: High. Orbit, approach and jump-to-tree sequence: High for states and exits; High for the landing height rule; Medium for the flight meanings. Body slam and head butt: High (damage and reach). Knockback: High. Chain whip and rapid strike: High for timing, the windows, the bone-chain test and its hero cylinder, the once-only rapid strike and the full-damage block. Raw boss damage (no hero defences): High. Block-counter branch: unreachable (High). Facing by locomotion mode: High for the shared steering rule (the 0.1 turn blend, the 0.9 switch, the snap weight 1.0, the caller blends and the mode lists, see Facing); Medium for which modes Gnarfas uses beyond orbit (Open question 14). Run or walk at 5.0: High. Travel speeds and distances at R = 1 (clip root motion): High. Per-tick animation-speed tunable and same-tick clip-rate effect: High; shipped default 1.0. Animation sounds (block 393, the table of the pack 985 record) and the chain sparks (effect 710): High; clip effect lists: Medium-High. Fight AI build guide: High for the order, states, clips and key counts; the tick counts follow the shared gate arithmetic (High for the formula, Medium-High for plus or minus one tick at the start of an attack); the opening approach time: Medium. The hero-is-down test (hold, knockback, death gate: action state 16): High. Hero-side interactions (what each hero action does to him): High for his own checks, Medium for the guard path in the block state; Medium-High for the zero damage and zero cost of Roberta's shots. Root motion of each clip: High.

## Summary

Gnarfas is a melee boss that closes on the hero, orbits when the hero is too close, and attacks with a chain whip, a rapid chain strike and a head butt. A chain whip and a rapid strike run on a cooldown that is drawn at random each time it is armed. A rapid strike can leave the boss exhausted for a while, and a jump sequence follows: the boss jumps between four tree points and sometimes jumps at the hero, landing next to the hero for a body slam. Phase 2 is switched on by a world flag, not by health. A one-shot rage flag arms when health falls below 25 percent. The boss reacts to the hero's missiles (block, dodge or jump) and melee attacks (dodge), and it takes damage only from hits it does not react to. All timings run on the 60 Hz game tick (SPEC-0007): a time in seconds becomes ticks = trunc(seconds x 60), and a random cooldown is drawn as a whole number of ticks between the two truncated bounds.

## Fight AI build guide

This is the ordered recipe for the shipped fight (pack 985, game state 281 clear, boss-fight instance 1). The detailed sections below give the reasons and the edge cases. Everything here runs on the 60 Hz tick. "s" is the tick in which a state first requests its animation; "s + n" is the tick in which the state's end test passes. All of this boss's clips have speed byte 2, and the controller keeps the clip rate at 1.0, so every clip advances 0.25 key per tick (4 ticks per key). A one-shot clip of F keys (F below 25) passes its 0.99 gate in tick s + 4(F - 1) + 1; the gate is read before that tick's animation pass (Clip table and durations).

### Initial values

- Character: embedded record 7 of pack 985 (type 5, variant 0). Monster row: 225 HP, body cylinder radius 2.5 and height 7.0, defence 0, behaviour class 14 (the Gnarfas controller). Position (-617.357, 126.221, -558.802), facing (0, 0, 1). The hero arrives at locator 9, 21.41 units away. The four tree points are the locators with ids 101 to 104.
- State idle, phase 1, rage flag 0, strike count 0, tree index none, reaction timer 0, hit latch 0, orbit side timer 0, wait flag clear. Jump timer 30 ticks. Cooldowns, drawn once at spawn as whole ticks, both ends inclusive: head butt 240 to 420, chain whip 120 to 240, chain power 120 to 360, rapid strike 180 to 360.
- Disc tunables are the words listed under Tuning values; read them at run time.

### Per-tick order

1. Write the animation-speed word (1.0) to the clip rate scale, set the character's base state to 1 and clear the motion record's per-tick flag (the jump state sets it again).
2. Count the seven timers down by 1 each, not below 0: reaction, head-butt cooldown, whip cooldown, chain-power cooldown, rapid cooldown, jump timer, exhaustion.
3. Damage check (Taking damage). It is skipped in flinch, death, chain whip, rapid strike, chain power, body slam and head butt.
4. Phase hub (Phase hub): wait flag, world queries, phase 2, then in free states the hold (hero down, action state 16), rage and death tests.
5. Reactions, only in a free state at the start of this step: missile reaction, then melee reaction. The melee reaction does not test the state again, so when both fire in one tick the melee dodge replaces the missile reaction's state.
6. Attack choosers (Attack selection), only if the state is still a free state.
7. Movement chooser (Movement), only if the state is still a free state (an attack just chosen is not).
8. The current state's handler (below). A choice made in steps 3 to 7 runs its handler in the same tick, so an attack's request tick is the tick of the choice.
9. Chain spark updates; then write the boss's character id into the selected hero's forced lock-on target field (every tick, so the hero's lock-on goes to Gnarfas while his controller runs; the hero's lock choice uses a forced target once and resets it, SPEC-0040 Behavior 20); then wait one tick.

Free states are idle, approach and orbit.

### State table

| State | Action (clip, keys) | Ticks | Exit |
| --- | --- | --- | --- |
| Idle | 3 (clip 0, 44, loop) | open | a hub, reaction or chooser decision |
| Approach | 6 run (clip 13, 13, loop) | open | orbit when the hero is closer than 6 (see Movement); the walk clip is never used here |
| Orbit | 13 or 14 (clip 22, 20, loop) | open | hero farther than 8 (approach), or an attack or reaction |
| Flinch | 9 (clip 8, 13) | n = 49 | damage applied; death, back to exhaustion, or idle |
| Triumph | 1 (clip 23, 41) | until the hero is no longer down (action state 16) | idle; the boss does not turn |
| Rage | none | 1 tick | idle (flag set) |
| Death | 2 (clip 5, 11) | n = 41, then event 2 every tick | none |
| Maniacal laugh | 0 (clip 16, 51) | n = 200 | idle |
| Chain whip | 19 (clip 4, 19) | n = 73 | idle, whip cooldown drawn |
| Rapid strike | 20 (clip 20, 41) | n = 160 | rapid cooldown drawn; roll below 75 exhaustion, else jump |
| Chain power | 21 (clip 17, 32) | n = 124 | never started |
| Exhaustion | 4 (clip 12, 32, loop) | phase 1: 1 tick; phase 2: 120 to 180 | jump sequence, step 0 |
| Body slam | 22 (clip 3, 15) | n = 57 | idle, or laugh |
| Head butt | 24 (clip 9, 20) | n = 77 | idle, head-butt cooldown drawn |
| Dodge missile | 16 (clip 7, 16, mirrored) | n = 61 | idle |
| Dodge melee | 16, 17 (clip 7) or 15 (clip 6, 14) | n = 61, 61 or 53 | idle |
| Block missile | no request; guard base state each tick | until no missile is due | idle |
| Jump, step 0 | 7 (clip 18, 7) | n = 25 | step 1 |
| Jump, flight | 12 once (clip 14, 10), then 11 (clip 11, 20, loop) | about 60 ticks (Jump to tree) | perch or landing |
| Jump, perch | 10 (clip 10, 20, loop) | wait 0 to 180 | step 2 or step 5 |

Every requested action id exists in the boss's own table, so the "missing id is skipped" rule never applies to this boss (the table has no clip only for ids 23 and 25, which are never requested). Ids 8, 18 and 26 have clips but are never requested.

### Attack chooser, in priority order (free states only)

1. Block counter: never starts (flag never set).
2. Chain whip: cooldown 0 and the hero horizontally closer than 9. No roll.
3. Rapid strike: cooldown 0 and the hero closer than 9. No roll.
4. Chain power: cooldown 0 and the hero closer than 9: roll 0 to 100 inclusive against a chance of 0, so it never starts; a failed roll redraws its cooldown (120 to 360).
5. Head butt: cooldown 0 and the hero closer than 15: roll 0 to 100 inclusive below 40 (phase 1) or 25 (phase 2) starts it; otherwise redraw its cooldown (240 to 420).

A roll is a whole number from 0 to 100 inclusive, so "chance c" passes with probability c / 101, not c / 100. Cooldowns count only while they are above 0; an attack is checked in the first free tick after its cooldown reaches 0, and after any attack the boss returns to idle for at least one tick, so the next free tick (s + n + 1) already runs the choosers.

### Movement (free states, after the choosers)

Hero farther than 8: approach (run). Otherwise, if the hero is at least 6 away or the boss is orbiting: nothing (idle stays idle; an approach continues). Otherwise (closer than 6, not orbiting): orbit with a coin for the side. In practice: the approach always runs, because it only starts above 8 and continues down to 6; the orbit takes over below 6 before the approach's own exit at 5 can be tested. The run clip moves 15.36 units per second along the facing; orbit slides 4.63 units per second sideways while facing across the motion (Travel, Orbit).

### Hit and hurt windows in ticks (frame f is live in handler ticks s + 4f to s + 4f + 3; tick s only requests)

| Attack | Live ticks | Result |
| --- | --- | --- |
| Chain whip | frames 7 to 13: s + 28 to s + 55 | one hit of 15 (bone-chain test), full damage even if blocked |
| Rapid strike | bit 0x4 frames 6 to 10: s + 24 to s + 43; bit 0x2 frames 10 to 13: s + 40 to s + 55; bit 0x1 frames 18 to 23: s + 72 to s + 95; bit 0x8 frames 25 to 29: s + 100 to s + 119 | one hit of 15 per attack, taken from the first window that touches the hero |
| Head butt | frames 6 to 13: s + 24 to s + 55 | knockback 2.0, no damage; hero in front (dot above 0.5) and closer than 7 |
| Body slam | any tick s + 1 to s + 56 | one strike of 30 if the hero is closer than 5 |

While in chain whip, rapid strike, chain power, body slam, head butt, or flinch, the boss cannot be hit. In every other state a hero melee hit or missile overlap starts a flinch (49 ticks), including during a dodge, the exhaustion, a jump flight or a perch. Damage is applied at the end of the flinch, once. A shot starts the flinch too, but its damage is 0. In the block-missile state only missiles are ignored.

### Phases, hand-offs and death

- Phase 1 to phase 2 needs two things: level bit 0 (the boss sends it when the hero is below half health or has been struck 3 or more times) and level bit 1 (set by the arena script after the mid-fight movie). Phase 2 changes the missile block chance to 25, the head-butt chance to 25, the exhaustion time to 120 to 180 ticks, and heals the hero to full once. Nothing else is reset: cooldowns, timers and the current state carry on.
- Rage arms once below 25 percent; it only sets a flag.
- Death: the end of a flinch that leaves health below 1 percent, or the hub in a free state with health below 1 percent, enters death. The death clip lasts 41 ticks; from then on event 2 (level bit 2) is sent every tick and the arena script runs the defeat sequence. The boss stays in its last pose until the script deactivates it.

### What a port gets wrong easily

- Rolls are 0 to 100 inclusive (101 values). A phase 1 missile block (100) fails on a roll of 100; a melee dodge (100) fails on a roll of 100.
- The animation for the maniacal laugh is action 0, not 18.
- Approach never uses the walk clip, and the approach's exit at 5 never runs. Walk appears only as the orbit fallback.
- The choosers run before the movement chooser, and both before the state handler. The laugh, rage and hold are the only extra states.
- Cooldowns are drawn in ticks (seconds times 60, truncated), and the whip and rapid cooldowns are drawn at the end of the attack, not at its start.
- Attack and dodge ends are clip gates (about 4 ticks per key), not timers: a chain whip occupies 74 ticks, a rapid strike 161, a head butt 78.
- Distances are horizontal. The hit shapes are bone cylinders, not boxes.
- The shipped tables carry sound cues per action (not the whip cue everywhere); see Sounds and effects.
- "The hero is down" in the hold, the knockback and the death gate means the hero's action state is 16. It does not mean the hero's busy list (jumping, swinging, being hit). Reading it as the busy list makes the boss freeze whenever the hero attacks and removes both the dodge and the knockback.

### Edge cases for implementers

These are the settled details that are easiest to miss. Each is explained in the section named in brackets.

1. **Hit before choice.** The damage check runs before the reactions and choosers in the same tick, so a hit on a tick on which an attack was due flinches the boss and the attack waits (Taking damage). A free boss that is hit does not roll the melee dodge in that tick.
2. **Melee dodge needs a free boss and a fresh swing.** The dodge roll (100 of 101) is made on the first free tick on which the hero is attacking and the latch is clear. A swing that began during an attack, a flinch or a jump is rolled for late, if it is still going on when the boss is free (Melee reaction). Only Tor's swings, power step and plunge and Roberta's swats count as attacking.
3. **The hero is "down" at state 16 only.** The boss does not freeze while the hero is swinging, jumping or in the hit reaction (Phase hub, step 5).
4. **Rolls.** Every "roll" is a whole number from 0 to 100 inclusive; a range draw whose two ends are equal returns that value and uses no random number (the phase 1 exhaustion of 0 to 0 ticks); the random side, the dodge variant and the tree pick are plain draws of 0 to 1, 0 to 2 and 0 to 2 (Jump to tree).
5. **No root motion in the attacks.** The chain whip, rapid strike, laugh and the whole jump sequence do not move the boss except by the flight velocity. The head butt lunges (14 units at its peak, 9.5 net) and the flinch slides back 5.3; the dodges move 9.0 sideways or 6.1 back. The edge guard stops root steps at ledges higher than 2.0 in every state except the jump sequence (Travel).
6. **Timers run in every state.** All seven timers count down on every tick whatever the state, so after a long jump sequence or exhaustion the whip, rapid and head butt cooldowns are all 0 and the next free tick starts the first of them that is in range, in the fixed order whip, rapid, chain power roll, head butt (Attack selection).
7. **Attack choices need range at that tick only.** A chain attack starts when the hero is closer than 9 on a free tick; it does not check the hero's facing or state, and the hero who then leaves is simply missed. The head butt starts from 15 and its range test is the only one that sees the hero beyond 9.
8. **The block holds still and keeps moving.** Block-missile writes no animation and never turns or steers the boss. A block from approach continues to run forward on the run clip's root motion (15.36 units per second) until the state ends; from orbit it keeps strafing (Missile reaction).
9. **Missile timer is not a damage window.** After a reaction the boss makes no further reaction to missiles for 60 to 180 ticks; a shot that arrives in that window strikes the body, is removed and flinches him for 49 ticks, but deals 0 damage. No shot ever lowers his health, in any state or phase; only hero melee does (Missile reaction, Taking damage). In phase 1 almost every first shot is blocked (100 of 101).
10. **Flinch ends in different places.** From exhaustion back to exhaustion (timer kept), from any other state to idle, never back into the jump sequence, and at once into death if health is below 1 percent. A hit in the first tick after a flinch is applied at once (Taking damage).
11. **Death percent.** Health below 1 percent is dead: with 225 health, 1 or 2 points left is dead and 3 is not (Phase hub, Edge cases).
12. **Strikes count everything.** The count behind event 0 (more than 2) rises on a blocked whip, a head butt, a skipped slam and a hit on a downed hero; it never falls. The hero's health below half also sends event 0. The movie therefore usually comes after the third strike (Phase hub, step 2).
13. **Phase 2 changes four things only:** block chance 25, head butt chance 25, exhaustion 120 to 180 ticks, and a one-time heal of the hero to full. The script moves the boss to locator 12 and the hero to locator 10 during the movie; the boss's state, timers and cooldowns are unchanged (the movie stops the world, so no controller tick runs during it; Arena flow).
14. **Hold and death order.** In a free state the hub tests: wait flag, phase, hold (hero down), rage, death, in that order; hold beats death, death beats rage. A boss that is not in a free state cannot hold, rage or die until it is (exception: the flinch end tests death itself).
15. **Landing slam is a distance test only.** The slam strikes on the first tick of its clip in which the hero is within 5.0, deals 30 and a knockback of 1.0 (unblockable), and misses a hero who is airborne on that tick for the whole clip (Body slam).
16. **Orbit never ends by itself.** Once orbiting, the boss keeps orbiting while the hero is closer than 8; the chain attacks interrupt it and approach replaces it only above 8. An orbit whose side point is off the mesh walks at the hero instead (Orbit).
17. **Chain whip and rapid strike follow the hero by turning only.** The turn weight is (1 - p) x 0.5 on each tick after the request tick; the hit test reads the poses of the previous tick (Chain whip and rapid strike).
18. **Spawn.** The four cooldowns are drawn once at the start of the fight; the jump timer is armed and unused. The boss starts in idle facing +z, 21.41 units from the hero, and approaches at once.

### Hero actions against this boss

"Free" means idle, approach or orbit. Damage numbers are from SPEC-0036 (the hero damage rules with defence 0; the target type of this boss is 0, so only a Nova Mote adds the element bonus). Chances are per roll out of 101.

| Hero action | Against Gnarfas |
| --- | --- |
| Tor normal swing (state 13, sub-states 2 to 12) | Starts: if the boss is free, the melee dodge rolls 100 / 101 on the first tick of the swing (sidestep 4.6 units by tick 8 and 8.3 by tick 24, or a step back 1.4 and 5.3); if the boss is busy the roll waits until it is free. Hits: when the weapon cylinder touches a bone of the boss in a tick in which the boss is in any state except flinch, death, chain whip, rapid strike, chain power, body slam or head butt. A hit starts the 49-tick flinch and its damage is applied at the end of the flinch. A dodging boss can still be hit. |
| Tor power attack (sub-state 10, with or without a Mote) | Counts as an attack for the dodge roll like a normal swing. Its melee hit is asked at power scale 1.0 like every melee hit he takes, so the attack power table is not used and the Mote adds nothing but its element bonus (a Nova Mote only); the status request that the shared swing code sends with it is not sent (the boss is not selected by the shared code), and the Mote power-attack objects (Rock, Leaf, Fog and the beams) never hit the boss. |
| Tor plunge attack (jump states, air phase 8 or 9) | Counts as an attack for the dodge roll. Hits as a melee swing. |
| Roberta swat (state 13, sub-states 5 and 12) | As Tor's swing, with her damage step factors. Her shots, spins and leap are not attacks for the dodge roll. |
| Roberta quick shot or full-release shot | A shot is a hero shot slot. When its next 10 ticks of travel touch a bone of the boss and the reaction timer is 0: phase 1 block 100 / 101 (the shot is removed harmlessly), else dodge 25 / 101, else jump sequence 25 / 101, else nothing; phase 2 block 25 / 101. The reaction timer then runs 60 to 180 ticks. A shot that overlaps his body cylinder is removed and, unless he is in block (the shot is only removed) or in one of the seven states in which the check is off, flinches him for 49 ticks. The flinch stores damage 0, so a shot never lowers his health, whether it arrives while the timer runs, after the three rolls fail, or in any other free or busy state: his check clears the shot slot first and only then asks for the shot damage, which is 0 for a cleared slot. The same clearing means the shot pays no MP and no Mote charge and plays no hit sound (Medium-High; a quick shot or a full-release shot behaves the same). Only Roberta fires shot slots; while Tor is the selected hero no shot is tested. In block a quick shot touching a bone bounces (guard path, Missile reaction). |
| Roberta Rock (skipping rock), beams, Leaf, Fog, Tor's Rock sub-shots | No damage and no reaction (Taking damage). The skipping rocks are Medium-High. |
| Shield guard (L1, state 20, or the 40-tick guard timer) | Chain whip and rapid strike: the hero still loses the full 15 but plays the block reaction and is not knocked back, if he faces the boss within 60 degrees. Head butt: no effect (knockback of 2.0 happens anyway). Body slam: no effect (30 damage and knockback 1.0). Nothing can be blocked into zero damage. |
| Jump or air state | A hero in action states 7 to 11 or 43 to 46 on the first in-range tick of a body slam takes nothing from that slam. Chain whip, rapid strike and head butt have no such rule (the chain cylinders are tested at the hero's present height; the head butt tests only the horizontal distance and facing). |
| Being down (state 16) | The boss leaves free states for the triumph hold after the attack it is in has ended; its strikes still count. It resumes when the hero is no longer down. |
| Items and potions, Motes as statuses | No effect on the boss. The status effects of a hero attack are not applied to the boss. |
| Taking the phase-2 trigger | Phase 2 starts only when the movie has run (level bits 0 and 1); the hero's health is then set to maximum once. |

When the hero can actually hurt him (only melee lowers his health; Roberta's shots flinch him for 49 ticks but deal 0 damage): the openings are the jump sequence (step 0 of 26 ticks, the flights, the perch), the exhaustion of a rapid strike in phase 2 (120 to 180 ticks, no reaction at all) and the laugh (200 ticks). A shot that lands inside the 60 to 180 tick missile timer, or after a failed reaction roll, still interrupts him with a flinch, which also keeps him from attacking for those ticks. Against a free boss almost every melee swing is dodged, and the dodge itself carries it out of reach in about 8 ticks. During a chain whip, rapid strike, head butt and body slam (74, 161, 78 and 58 ticks) and the flinch (50) he cannot be hurt.

### Timing table (timing pass)

Confidence: High for the windows and the counters; Medium for the hit tick inside a window. Units: ticks of 1/60 s; the seven timers (reaction, head-butt, whip, chain-power and rapid cooldowns, jump timer, exhaustion) are whole ticks lowered at the start of every controller pass, not below 0; cooldowns are drawn in ticks (seconds x 60, truncated) at the end of the attack.

| Attack (s = request tick = choice tick) | Live ticks | Result |
| --- | --- | --- |
| Chain whip | s + 28 to s + 55 | one hit of 15, full damage even if blocked |
| Rapid strike | s + 24 to s + 43, s + 40 to s + 55, s + 72 to s + 95, s + 100 to s + 119 | one hit of 15 per attack, from the first window that touches the hero |
| Head butt | s + 24 to s + 55 | knockback 2.0, no damage; hero in front and closer than 7 |
| Body slam | any tick s + 1 to s + 56 | one strike of 30 if the hero is closer than 5 |
| Flinch | 49 ticks | the hero's damage is applied at the end of the flinch, once |
| Death clip | 41 ticks | then level event 2 every tick |

Hero side: the whip and the rapid strike hit inside the boss's think, before the hero's update; there is no invulnerability, so a hero in the hurt state can take the next attack.

Worked exchange (Tor with a weapon 6 units in front): s chain whip; first touch h in s + 28 to s + 55: 15 damage, Tor flinches from h and is back in state 1 on h + 37; the whip cooldown is drawn when the whip ends. A Tor hit on a free Gnarfas starts its 49-tick flinch, and the damage reaches the boss only on the flinch's last tick; further hits in those 49 ticks are ignored.

What a port most likely gets wrong (timing): applying the hero's damage at the hit instead of at the end of the flinch; drawing cooldowns at the start of an attack; hitting on the window start.

## Arena build guide

The arena is Illusen's Glade (world pack 985), the same floor as the Gelert Assassin fight (SPEC-0013, Arena build guide, which holds the floor outline, passages, ring points and numbers shared by both fights). This section gives the Gnarfas side. Confidence is High unless marked.

**1. Floor and mesh.** The floor is the flat patch at height 121.436 with the nine-corner outline of SPEC-0013 (about 89 by 88, area 5,579). Gnarfas is on mesh 1 (x -663.0 to -586.0, z -579.1 to -503.7, area 4,143, flat at 122.53), the larger of the two ground meshes; the script assigns it to him every tick while flag 214 is clear. Tree points 101, 102 and 103 lie only in mesh 1 (mesh 0 does not reach them); tree 104 is in both. The orbit side point and the jump landing are therefore tested against mesh 1. No deadly ground, pit, water or damage volume exists.

**2. Start positions.** Arrival is a replacement load; the selected hero is first placed at the pack's entry 0 far away, then the activation callback moves him to locator 9, (-637.03, 122.67, -567.24), facing (0.546, 0, 0.838), and the normal camera is set (3 s). Gnarfas is a streamed character with record position (-617.357, 126.221, -558.802), facing (0, 0, 1); on the first script tick after arrival the script moves him to locator 11, (-617.357, 123.335, -558.802), the same x, z, 2.9 lower, 21.4 from the hero (the hero looks 34 degrees off him). The Werelupe King and Illusen are placed at (-580.80, 121.88, -496.08) and (-582.57, 121.48, -493.09) in the north-east passage beyond the sealing wall, 62 and 66 from the hero; the King is grabbed and held on the arrival tick, playing his animation 0. There is no intro scene and no camera track: control is the hero's from the arrival, and the boss approaches at once.

**3. Walls, objects and tree points.** The module's mask word has bits 1 and 2 set at arrival (when flag 214 is clear), turning on the north-east wall (gate 1; from (-590.91, -527.83) to (-612.3, -510.1), 17.1 high) and the south-west wall (gate 2; from (-656.19, -568.09) to (-622.97, -583.40), 23.2 high); the defeat clears bit 1, opening the north-east passage. The placed glow effects 7 to 12 (north-east mouth) are stopped by the defeat; 1 to 6 (south-west) keep running. There is no pickup, breakable or save point. The four tree points (locators 101 to 104) are at (-589.21, 127.0, -534.68), (-635.92, 128.0, -507.60), (-658.73, 127.0, -552.39) and (-613.24, 128.0, -575.53): 34 to 37 from the floor's centre and 4 to 9 inside the outline, 5.6 or 6.6 above the floor (4.3 or 5.3 above the hero's start height). From Gnarfas's spawn the nearest is 104 at 17.2; the others are 37.1, 41.9 and 54.5 away. Run through the step-1 flight rule from the spawn to tree 104, the horizontal exit test (below 3) fires on the 50th move with the body about 16.7 above the floor (about 10 above the locator): the perch hangs high, not at the locator's height (Medium; it follows the flight rule as written).
**4. Script timeline.** (1) Arrival: hero to locator 9, normal camera, flag 212 set; with flag 214 clear: level states 0, 1, 2 cleared, both wall bits set, refill of the first hero's health and second meter (instance 1, stage 0). (2) Next tick of the control thread (arrival bit set): sleep 1, Gnarfas to locator 11, King grabbed. Mesh 1 is assigned each tick. (3) The boss fights; its event 0 (hero below half health or struck three times) sets level state 0. (4) Phase hand-over: movie 12 (waited), hero to locator 10, (-625.73, 122.67, -548.23) facing (-0.505, 0, 0.863); Gnarfas to locator 12, (-647.11, 122.75, -526.51), 30.5 away; normal camera 3 s; flag 213; level state 1; 30 ticks; dialogue line 14; phase 2 starts on his next tick. (5) Death (event 2 every tick): the fight-end routine runs once: effects 7 to 12 stopped, Gnarfas and the King deactivated, flag 214 set, wall NE off, instance 1 stage 1 (no effect). (6) Next ticks (flag 214 set): movie 13 (waited), fade out, hero to locator 26, (-621.28, 122.69, -542.68), Illusen's first record deactivated and her second record (same spot, facing (0.707, 0, -0.707)) activated and looped in animation 3, normal camera 10 s, fade in 1.0 s. (7) The hero walks north-east: the slab of trigger 4 (67 long, 7 thick, 5.8 beyond the open wall) fires on contact; the thread sets flag 64, sets the movie return world and plays movie 14, which returns to the village. What holds the boss: nothing; he is free from the arrival tick, and during the movie of step 4 no tick runs.

**5. Camera and music.** Follow camera on the floor's profile throughout; no boss camera, no fixed track (unlike the Gelert fight, this module plays no camera track in the Gnarfas branch). Music: plain loop (bank resource 31, stream resource 163): the selector-3 alternate needs flag 64 set and progress bit 216 set, and flag 64 is set only after the fight. One ambient loop (radius 45 at the floor's centre) covers the floor. No sound call in the module.

**6. Death and retry.** Act 1 swap mode 0 (locked to Tor): Tor's death is refused for a take-over, so the game over page follows the death clip (about 63 ticks) after 91 more timer additions. Continue re-applies the last save or load (no save point in the pack, so from before the fight): boss HP 225, phase 1, strikes, level states and mask word reset; flags 212 and 214 as saved. If flag 214 was clear the fight is due again via the village trigger (movie 11 plays first, then the load). Leaving mid-fight and re-entering also restarts the fight at full boss HP; the hero's health is refilled by the arrival call.

**7. Victory.** Rewards: flags 214 (set at the defeat) and 64 (set at the exit step); no item. The hero ends at locator 26 (-621.28, 122.68, -542.68), facing (0.628, 0, 0.778), with control and the normal camera; Illusen's second record stands in the open passage straight ahead (62.9 away, within one degree of the hero's facing). The way out is the slab of trigger 4: its near face is 36.5 in front of locator 26 along that facing (the north-east wall opened by the defeat is crossed after 30.9), 67.26 wide and 7.0 thick, so a hero who simply walks forward meets it near its middle. The game shows no line, marker or talk after the fade-in; Illusen's looping animation and the hero's facing are the only guide. Trigger 4 is the only exit: the pack has no door or teleport plane, the module registers no other trigger, and the south-west wall stays closed. Touching the slab sets flag 64 and plays movie 14, which returns to the village. The pack is not re-entered afterwards in the Act 1 route (the village trigger needs flag 214 clear for this branch).

**8. What a port most likely gets wrong here.** (1) Using mesh 0 for Gnarfas: he uses mesh 1. (2) Treating locator 11 as Gnarfas's phase-2 point: the move after the movie is locator 12; locator 11 is the arrival point. (3) Forgetting the sealing walls and the King and Illusen behind them. (4) Believing the tree locator is the perch height. (5) Playing the alternate music, or a boss camera.

**9. End-to-end test vector (Act 1, flags 212 set, 214 and 64 clear, Tor selected).** Load; Tor at (-637.03, -567.24) facing (0.546, 0, 0.838); Gnarfas at (-617.357, -558.802) 21.4 away on mesh 1 (triangle shared with mesh 0), both walls on, plain music. Gnarfas approaches; at the third strike or Tor below half health event 0 sets level state 0; movie 12 plays, Tor is at (-625.73, -548.23), Gnarfas at (-647.11, -526.51), line 14 shows, phase 2 starts and heals Tor. At death the walls and effects 7 to 12 change as above, movie 13 plays, Tor is at (-621.28, -542.68) and Illusen is in the passage. Tor walks about 45 units north-east through the open diamond, crosses the slab near (-598, -512), flag 64 is set and movie 14 returns to the village.

## Reach and distances (boss reach pass)

How the figures are made (Medium-High for the shapes, Medium for the edges): Gnarfas's skeleton is posed from each attack clip at the cursor of the previous animation pass; on each window tick the bone runs selected by the frame's bits (Chain whip and rapid strike) are placed as cylinders of radius 0.5 and length 1.25 from each bone's origin along its Y axis and tested with the exact rule of SPEC-0018 (bone cylinder as the first shape, an assumed role: Medium; the figures did not move apart from one tick at one distance) against the hero's upright cylinder (radius 1.0, length 3.0 on the hero's position) at horizontal distance d from Gnarfas's position, level with him. His attack clips have no root travel. Both attacks turn toward the hero (weight (1 - p) x 0.5), so a hero who stays put is taken on the facing line; the side figures show what a sidestep changes.

| Attack | Window | Hero touched (d from Gnarfas) | First touching tick |
| --- | --- | --- | --- |
| Chain whip (0x13, bones 29 to 33) | s + 28 to s + 55 | on the facing line 3.0 to 6.0; 2 units to the side 3.0 to 5.5; 4 to the side 0.5 to 5.0; never beyond about 6.3 | s + 31 on the line; s + 29 to s + 32 at 2 to the side; s + 28 to s + 29 or s + 33 to s + 37 at 4 to the side (by side) |
| Rapid strike (0x14) | four windows (bits 0x4, 0x2, 0x1, 0x8) | on the line 1.5 to 6.0 in the first window, 6.5 to 7.5 only in the last; behind him 2.0 to 0.5 in the second; up to 8.0 on the side; never beyond about 8.0 | s + 29 to s + 30 for 1.5 to 6.0; s + 101 to s + 102 for 6.5 to 7.5; s + 41 to s + 52 behind |

- The hero cannot stand closer than about 3.5 to his centre (body radius 2.5 plus the hero's radius), so the whip's inner edge is never reached in play.
- Distance-only hits (no shape): head butt, the hero in front (dot above 0.5) and below 7.0 on ticks s + 24 to s + 55, knockback strength 2.0 (velocity normalize(9 d + 12 up) x 16, that is 9.6 out and 12.8 up in motion-record units); body slam, below 5.0 on any tick s + 1 to s + 56 (skipped, but used up, while the hero is airborne). The whip and rapid strike knock back with strength 1.0 (4.8 out, 6.4 up).
- Distances the AI uses (horizontal): chain whip, rapid strike and chain power below 9; head butt below 15 (40 or 25 in 101); approach (run, 15.36 units per second) above 8 until the orbit takes over below 6; orbit below 6 (4.63 units per second sideways); jump step 1 to the nearest of the four trees; perch when the tree is within 3 horizontally (step 1) or 3 in 3D (step 2); after a landing, body slam when the hero is within 5; missile reaction from a hero shot whose 10-tick path touches a bone; block missile while a shot's 120-tick path still touches; melee dodge on every swing start (100 in 101), turning 0.1 per tick.
- Safe spots (Medium): 6.5 to 9 from him when the whip starts (chosen below 9, reaches about 6.3) and 8 to 9 for the rapid strike; beyond 7.0 or behind his shoulder line for the head butt; airborne on the first slam tick within 5.0.
- Worked example: phase 1, whip cooldown 0, hero standing still 5.0 in front of Gnarfas: the whip is chosen on tick s; on s + 31 the chain bones overlap the hero's cylinder: 15 raw damage and a knockback of strength 1.0 (the full 15 with the block reaction and no knockback if he guards facing Gnarfas). The same hero standing 7.0 away when the whip starts is never touched.

## Player summary

- Only melee hurts him: Tor's sword (a power step with a Mote counts as an ordinary swing) and Roberta's staff swat. Shots, Motes, beams and spells never lower his health; a shot only makes him flinch for nothing.
- A swing counts in every state except while he is flinching or in the middle of his chain whip, rapid strike, head butt or body slam. So hit him after an attack ends, while he waits, while he is exhausted or laughing, and while he is in the jump sequence; being in the air does not protect him.
- He sidesteps almost every swing (100 of 101), but a sidestepping Gnarfas can still be hit. A hit makes him flinch for about 49 ticks and the damage lands when the flinch ends; he cannot be hit again during it.
- Avoid standing in front of the chain whip and rapid strike (15 each), the head butt and the body slam (30). The shield removes none of that damage; it only cancels the knockback of the chain attacks. Phase 2 starts after the mid-fight movie and restores the hero's health once.

## Where it lives on the disc

- The boss's tuning values are 32-bit words in the boot program's initialized data. The game reads them at runtime by virtual address, so a clean build should read the shipped defaults from the disc. The values sit at data addresses 0x2DAD98 to 0x2DAE74. Shared constants sit at 0x2DAE78, 0x2DAE7C, 0x2DAE80 and 0x2DAE8C (see Layout).
- "int" is a signed 32-bit integer and "float" an IEEE single. Percent values are whole numbers from 0 to 100 unless stated.
- The hero is the player's character. Distances are measured horizontally from the boss to the hero, in world units (see Cross-boss facts in SPEC-0014). The unit scale is not confirmed (see Open questions).
- Tree points: four locator objects with ids 101 to 104 in this boss's level (ids are per level).
- Animation numbers are entries of the boss's animation table (SPEC-0005). Locomotion modes are separate: a mode selects which table animation the movement plays (see Movement).
- "Post event n" means set level flag n (SPEC-0014, cross-boss facts). The level script's set-level-state and query-level-state calls use the same level bitset as the boss events, so a boss event n and the script's level state n are one bit (bit n of byte n / 8) (High).

## Layout

### Tuning values

Rows marked "unused" are stored and registered, but no code in the boot program reads them (see Edge cases). Rows marked "unreached" are read by code that the behavior described here does not reach.

| Parameter | Type | Data address | Shipped value | Meaning |
| --- | --- | --- | --- | --- |
| Maniacal laugh chance | int % | 0x2DAD98 | 30 | Chance to laugh after a body slam that hit (the chain power is unreachable) |
| Exhausted chance 1 | int % | 0x2DAD9C | 25 | Unused |
| Exhausted chance 2 | int % | 0x2DADA0 | 75 | Unused |
| Rage trigger percent | int % | 0x2DADA4 | 75 | Rage arms when health falls below 100 minus this value (25 percent) |
| Near-boss distance | float units | 0x2DADA8 | 0 | Unused |
| Missile block chance, phase 1 | int % | 0x2DADAC | 100 | Block chance against a missile in phase 1 |
| Missile block chance, phase 2 | int % | 0x2DADB0 | 25 | Block chance against a missile in phase 2 |
| Missile dodge chance | int % | 0x2DADB4 | 25 | Dodge chance against a missile, if the block roll failed |
| Missile dodge timer min | float s | 0x2DADB8 | 1 | Shortest wait after a missile reaction (the reaction timer) |
| Missile dodge timer max | float s | 0x2DADBC | 3 | Longest wait after a missile reaction |
| Missile jump chance | int % | 0x2DADC0 | 25 | Jump-sequence chance against a missile, if the dodge roll failed |
| Melee dodge chance | int % | 0x2DADC4 | 100 | Chance to dodge when the hero starts a melee attack |
| Melee jump chance | int % | 0x2DADC8 | 100 | Unused |
| Block counter range | float units | 0x2DADCC | 15 | Unreached (the block-counter step never starts an attack, see Attack selection) |
| Block counter chance | int % | 0x2DADD0 | 35 | Unreached (see Attack selection) |
| Chain whip chance | int % | 0x2DADD4 | 40 | Unused (the chain whip does not roll; see Attack selection) |
| Chain whip cooldown min | float s | 0x2DADD8 | 2 | Shortest chain whip cooldown |
| Chain whip cooldown max | float s | 0x2DADDC | 4 | Longest chain whip cooldown |
| Chain whip trigger range | float units | 0x2DADE0 | 9 | Hero must be closer than this to start a chain whip |
| Chain whip damage | int | 0x2DADE4 | 15 | Damage per chain whip hit |
| Chain whip head-butt chance | int % | 0x2DADE8 | 25 | Unused |
| Rapid strike chance | int % | 0x2DADEC | 10 | Unused (the rapid strike does not roll; see Attack selection) |
| Rapid strike cooldown min | float s | 0x2DADF0 | 3 | Shortest rapid-strike cooldown |
| Rapid strike cooldown max | float s | 0x2DADF4 | 6 | Longest rapid-strike cooldown |
| Rapid strike trigger range | float units | 0x2DADF8 | 9 | Hero must be closer than this to start a rapid strike |
| Rapid strike damage | int | 0x2DADFC | 15 | Damage per rapid-strike hit |
| Rapid strike exhaust chance | int % | 0x2DAE00 | 75 | Chance that a rapid strike ends in exhaustion (otherwise the jump sequence) |
| Chain power trigger range | float units | 0x2DAE04 | 9 | Hero must be closer than this to start a chain power |
| Chain power cooldown min | float s | 0x2DAE08 | 2 | Shortest chain-power cooldown |
| Chain power cooldown max | float s | 0x2DAE0C | 6 | Longest chain-power cooldown |
| Chain power chance | int % | 0x2DAE10, 0x2DAE14 | 0 / 0 | Phase 1 / phase 2 chance (both zero: unreachable) |
| Chain power damage | int | 0x2DAE18 | 13 | Damage per chain-power hit |
| Jump-to-hero chance | int % | 0x2DAE20 | 50 | Chance that the jump sequence jumps at the hero rather than to another tree |
| Body slam damage | int | 0x2DAE2C | 30 | Damage of the body slam |
| Head-butt trigger range | float units | 0x2DAE30 | 15 | Hero must be closer than this to start a head butt |
| Head-butt cooldown min | float s | 0x2DAE34 | 4 | Shortest head-butt cooldown |
| Head-butt cooldown max | float s | 0x2DAE38 | 7 | Longest head-butt cooldown |
| Head-butt chance | int % | 0x2DAE3C, 0x2DAE40 | 40 / 25 | Phase 1 / phase 2 chance |
| Approach range | float units | 0x2DAE48 | 8 | Approach when the hero is farther than this |
| Time between jumps | float s | 0x2DAE54 | 0.5 | Jump timer length (30 ticks) |
| Exhausted time min, phase 1 | float s | 0x2DAE64 | 0 | Shortest exhaustion in phase 1 |
| Exhausted time max, phase 1 | float s | 0x2DAE68 | 0 | Longest exhaustion in phase 1 |
| Exhausted time min, phase 2 | float s | 0x2DAE6C | 2 | Shortest exhaustion in phase 2 |
| Exhausted time max, phase 2 | float s | 0x2DAE70 | 3 | Longest exhaustion in phase 2 |
| Animation speed | float | 0x2DAE74 | 1.0 | Written directly to per-tick speed scale P by the class routine each tick, before clip re-time. Shipped value 1.0; a changed value scales same-tick clip playback and the resulting root step (see Edge cases). |
| Switch-to-walk percent | int % | 0x2DAE4C | 25 | Unused |
| Orbit range | float units | 0x2DAE50 | 7 | Unused (the orbit uses fixed thresholds) |
| Chance min | int % | 0x2DAE58 | 5 | Unused |
| Chance max | int % | 0x2DAE5C | 80 | Unused |
| Injured stance percent | int % | 0x2DAE60 | 25 | Unused |
| Body slam cooldown | float s | 0x2DAE28 | 10 | Unused |
| Jump land distance | float units | 0x2DAE1C | 10 | Unused (the landing uses a height rule) |
| Jump attack trigger range | float units | 0x2DAE24 | 5 | Unused |
| Head-butt prepare chance | int % | 0x2DAE44 | 75 | Unused |

Shared constants used by the boss:

| Constant | Type | Data address | Value | Meaning |
| --- | --- | --- | --- | --- |
| Tick rate | float | 0x2DAE7C | 60.0 | Ticks per second |
| Step constant | float | 0x2DAE80 | 0.05 | Per-tick gravity of the jump arcs (the jump's vertical speed falls by this each tick) |
| Knockback push speed | float | 0x2DAE78 | 8.0 | Speed multiplier of the knockback push (8 x strength) |
| Run-or-walk distance | float units | 0x2DAE8C | 5.0 | Approach uses the run locomotion when farther than this, walk when not |

Fixed values used by the facing rule (not data words): the turn blend 0.1 per tick, the alignment test 0.9, and the snap weight 1.0. The caller's default blend is 0.1 for the strafe call (the data word at 0x2DD86C) and 0.3 otherwise (a fixed value).

## Behavior

### Setup at spawn (Confidence: High)

- The state starts at idle. The jump timer starts at 30 ticks.
- Each cooldown is drawn once at spawn: the head-butt cooldown from its min and max, the chain whip cooldown from its min and max, the chain power cooldown from its min and max, and the rapid-strike cooldown from its min and max. Each draw is a whole number of ticks, uniform from trunc(min x 60) to trunc(max x 60), both inclusive.
- The four tree points are found in the level (ids 101 to 104).

### Each tick, in order

1. Write the animation-speed tunable to the boss character's per-tick speed scale P. This class coroutine runs after the NPC update resets P and applies status factors, but before clip re-time. The direct write replaces P for that tick, so a changed tunable affects the current clip rate; the shipped value is 1.0 and therefore leaves it at normal rate (see Edge cases).
2. Count down the timers: the reaction timer, the cooldowns, the jump timer, and the exhaustion timer, each by one tick, not going below zero.
3. Run the damage check (Taking damage).
4. Run the phase hub (below).
5. Run the missile reaction and then the melee reaction (free states only).
6. Run the attack choosers, then the movement chooser (free states only; the attack order is under Attack selection).
7. Run the current state's behavior.
8. Run the post updates (the chain spark checks), then write the boss's character id into the selected hero's forced lock-on target field (Medium-High for the effect on the hero's lock choice; High that the write happens on every tick).
9. Wait one tick.

The same order, with the per-state clips and tick counts, is in the Fight AI build guide.

### Phase hub (Confidence: High for the order and the world queries; Medium for the phase 2 effect)

Run each tick, in this order:

1. **Wait flag.** If the wait flag is set, the boss goes to the wait state (see States) and the rest of the hub is skipped. The wait flag is a word of the game's control state, not a level-script value. The game stores 0 to it at start-up and on each world load, and nothing ever sets it. So the wait state is never entered: implement the test and leave the flag always clear (High).
2. **World queries.** The world queries are bits 0 and 1 of the level's flag array. While bit 0 is clear: if the hero's health is below half its maximum, or Gnarfas has struck the hero more than twice (3 or more strikes), Gnarfas sends event 0 (sets level bit 0). The send sets the bit, so this happens once.
3. **Phase 2 check.** If bit 0 and bit 1 are both set and Gnarfas is not yet in phase 2, Gnarfas enters phase 2 once: the one-time phase-2 effect runs and the hero's health is restored to its maximum. Phase 2 is set by the world flags, not by health. Bit 1 is set by the arena script in answer to bit 0, after the mid-fight movie (see Arena flow).
4. **Free-state gate.** The rest of the hub runs only if the boss is in a free state: idle, approach or orbit.
5. **Hold.** If the hero is down, the boss enters the triumph (hold) state and nothing else in the hub applies. "Down" is exactly the hero's dead query: the hero's action state is 16 (the death state; SPEC-0036, Health and death). It is not the wide busy list of the hero (jumping, attacking, being hit, scripted states) and not the script query IsHeroOccupied: a hero who is swinging, jumping, in the hit reaction or knocked back is not down, and the boss keeps fighting (High). The same test guards the knockback (Knockback) and, in the hold, the exit.
6. **Rage.** Otherwise, let p = trunc(100 x health / maximum health). If p < 100 - 75 (75 is the rage trigger word 0x2DADA4), so p < 25, and the one-shot rage flag is 0, the boss enters the rage state.
7. **Death.** If p <= 0 (health below 1 percent, since p is truncated), the boss enters the death state. This overrides the rage state in the same tick. If p is above 0, the boss keeps its state.

Details of the hub:

- Death is checked only in free states and only when the hero is not down. A boss at 0 health that is holding over a downed hero (triumph state) does not die until the hold ends. A boss whose flinch ends at 0 health dies at once, whatever the hero's state (Taking damage).
- The rage flag is set in the rage state and never cleared, so rage arms at most once per boss.

### Taking damage (Confidence: Medium-High)

Gnarfas checks hits against itself each tick, before the hub. The check runs in every state except flinch, dead, chain whip, rapid strike, chain power, body slam and head butt (in those states hits do nothing).

- **Melee hit**: the hero is attacking, the hero's attack is active and the hero's weapon touches Gnarfas (the hero's single-weapon reach test, SPEC-0029 Behavior 10.2). This starts the flinch.
- **Missile hit**: each of the hero's six missile slots is tested as a sphere (the slot's point and radius) against Gnarfas's vertical body cylinder (the body record's radius and height, standing on its position, axis up). The first slot that overlaps is removed. If Gnarfas is not in the block-missile state, the flinch value comes from the hero shot damage query for that same slot index against Gnarfas, asked after the slot was cleared, so it is 0 (see below); the result is truncated toward zero when the flinch starts, and the tick's test ends. In the block-missile state the slot is only removed and the test goes on to the next slot without querying damage.

Order matters (Medium-High): the slot is cleared before the damage query is made, and the query returns 0 at once for a cleared slot (it also skips the MP and Mote charge payment that the shared shot code makes for a live slot). The stored flinch damage of a shot hit is therefore always 0, the flinch still starts, and no health is lost.
- The flinch plays animation 9. The flinch records "was exhausted" when it starts in the exhausted state.
- The damage (the hero's damage for that hit) is applied once, when the flinch animation ends. Then: if trunc(100 x health / maximum) < 1, death; else, if the hit came while exhausted, back to exhaustion (its timer keeps running); else idle.
- So Gnarfas takes no damage during its own attacks, a running flinch, or in the block-missile state (missiles only; a melee hit still flinches him there).
- **Melee comes first.** The shot loop runs only when the melee test did not hit in this tick. A tick with a melee hit leaves every shot slot untouched, and the shots are tested again on the next tick the check runs.
- **What a melee hit also does (Medium-High).** Besides the flinch, a melee hit marks Gnarfas as the target shown by the boss health display for 600 ticks, starts a hit spark (entry 222 of the combat effect bank, at the weapon contact point) and plays the hero's hit sound for the weapon. A shot hit does none of these. They are cosmetic and do not change the fight.
- **Self-guard of the hero's shared hit code.** Gnarfas sets the word that makes him unselectable by the shared hero hit routines when his controller starts (the hero swing resolver, the shot walk, the Mote power-attack objects and the random target finder never choose him) and never clears it. All damage he takes therefore comes through this check, and only the hero's melee lowers his health: the six shot slots start a flinch with damage 0. Roberta's Shadow, Sun and Bubble beams, the Leaf and Fog objects, Tor's Rock sub-shots (the Mote power-attack objects of SPEC-0036) and Roberta's skipping rocks never hurt him (High for the objects that use the shared walk; Medium-High for the rocks). The one visible exception is the guard path described under Block missile.
- **One swing, one hit.** This check keeps no hit list. The 49-tick flinch is the only limit on repeated hits from one swing; the longest hero attack window (Tor's clip 39, 48 ticks) is shorter than the flinch, so one swing hurts him at most once.
- **Back-to-back flinch (Medium-High).** The flinch handler asks for its clip only when the clip is not already the current one, and tests the clip end with the clip as it is. In the first free tick after a flinch (the tick after the gate tick) the current clip is still the finished flinch clip, because the idle request has not been made yet. A hit in exactly that tick starts a new flinch whose gate is already open: its damage is applied in that same tick and the boss is back in idle (or dead) at once, with no 49 ticks of invulnerability. Rare: it needs a hit in that single tick.
- **Damage application.** At the end of the flinch the stored whole-number damage is subtracted; if it is not below the current health, health becomes 0 (no negative). The death test then reads the truncated percent of health over maximum. Health 0 in the middle of a flinch cannot happen, because damage is stored, not applied, until the gate.
- **Exhaustion and flinch.** A hit while exhausted flinches the boss; when the flinch ends the boss returns to exhaustion with its timer still counting (the timer ran down during the flinch like every other timer, and the exhaustion ends on its first tick if the timer has reached 0). A phase 2 exhaustion of 120 to 180 ticks therefore holds one flinch, and a second only if the hero lands it within what is left of the timer.
- **A hit beats a new attack.** The damage check runs before the choosers in the same tick, so a hit that lands in the tick on which an attack would have been chosen flinches the boss and the attack is not chosen (the choosers run only in free states).
- These two are the only hit kinds (High). Nothing else starts the flinch: no hazard, no other character and no script hit is tested. The stored damage is a whole number: the hero's melee damage (asked at scale 1.0) or the missile-slot-indexed hero damage query, which is always 0 here because the slot is cleared first, each truncated toward zero when the flinch starts (High for the routing and truncation; Medium-High for the zero).
- **Shapes of the two contact tests.** The melee contact is the hero's single-weapon reach test against Gnarfas (Open question 17, resolved). The missile hit is the sphere against body cylinder test above; the missile reaction uses a different shape (bone cylinders, see Missile reaction). The hero-side shape records that some of the controller's tests read belong to the Werelupe King and are described in SPEC-0019, not here.

### States (Confidence: High for transitions; Medium for entry effects)

| State | Animation (table index) | Behavior | Ends |
| --- | --- | --- | --- |
| Idle | 3 | Waits. A choice moves the boss out of idle. | A choice starts. |
| Flinch | 9 | Plays the flinch animation; damage is applied at its end (see Taking damage). | Death, exhaustion or idle, as stated. |
| Triumph (hold) | 1 | Plays animation 1 and holds its last pose while the hero is down (action state 16). It does not turn toward the hero, and the hold request is repeated every tick until the clip is current. | When the hero is no longer down (revived, respawned or Continued): idle. |
| Rage arm | none | Sets the one-shot rage flag. | Immediately: idle. |
| Death | 2 | Plays the death animation. On finishing, sends event 2 (sets level bit 2), which starts the arena's defeat sequence (see Arena flow). | Terminal. |
| Maniacal laugh | 0 (the state's own number is 0x12; its animation request is action 0) | Laughs. | Animation finished (200 ticks): idle. |
| Chain whip | 0x13 | Hits once with the chain whip damage. The facing test picks the outcome (see Chain whip and rapid strike). | Animation finished: chain-whip cooldown drawn; idle. |
| Rapid strike | 0x14 | Hits once (no re-arm frame) with the rapid-strike damage (see Chain whip and rapid strike). | Animation finished: rapid-strike cooldown drawn. Then a roll less than 75 starts exhaustion; otherwise the jump sequence starts. |
| Exhaustion | 4 | Counts the exhaustion timer down. | Timer below 1 tick: the jump sequence starts, with its first step. |
| Chain power | 0x15 | Hits once with the chain-power damage (unreachable with shipped chances). | Animation finished: exhaustion with a drawn timer. If it hit and a roll is less than 30, maniacal laugh instead. |
| Body slam | 0x16 | Strikes once if the hero is closer than 5 units (see Body slam). | Animation finished: idle. If it hit and a roll is less than 30, maniacal laugh instead. |
| Head butt | 0x18 | Strikes when a hit test passes, once per attack. Adds one to the strike count and sends the hero a knockback with strength 2.0. No damage. | Animation finished: head-butt cooldown drawn; idle. |
| Approach | 6 when the hero is farther than 5 units (locomotion mode 1), otherwise 5 (locomotion mode 0) | Moves toward the hero: run 15.36 units/s, walk 6.34 units/s on average (root motion; see Travel). The state starts only when the hero is farther than 8 and the movement chooser keeps it going down to 6, so the run clip is the only one it plays. | Hero closer than 5 units: idle. This exit cannot run: the movement chooser switches to orbit at 6, before the state's own test (see Movement). |
| Orbit | 0x0D or 0x0E (locomotion mode 5 or 6; a coin picks the side) | Moves sideways around the hero at 4.63 units/s on average (see Orbit and Travel). | Approach or attack ends it. |
| Dodge missile | 0x10 | Dodges a missile (the reaction chose it). | Animation finished: idle. |
| Block missile | no animation request; the character's guard base state (state 20) is written every tick | Keeps playing whatever clip was current (idle, run or strafe). Holds the guard state while a hero missile would reach Gnarfas. | No missile due: idle. |
| Dodge melee | variant 0: 0x10, variant 1: 0x11, variant 2: 0x0F | Dodges a melee attack (see Melee reaction). | Animation finished: idle. |
| Jump to tree | sub-steps below (7, 0x0B, 0x0C, 0x0A) | The jump sequence. | Sub-steps finished: idle. |
| Wait (live pause) | 3 | Waits while the wait flag is set. Never entered, since the flag is never set (see Phase hub). | Flag clear: idle. |

The jump-to-centre state is never entered.

### Clip table and durations (Confidence: High for the clip data; Medium-High for plus or minus one tick)

The boss's action table has 27 entries (ids 0 to 26). Ids 23 and 25 have no clip and are never requested. Every clip has speed byte 2, so at the shipped rate 1.0 each key lasts 4 ticks. A request in tick s starts the clip in that tick. A state that "plays an action to its end" requests it when the current action differs, then, in each later tick s + j, reads the clip progress before that tick's animation pass: progress = (cursor + stored fraction) / keys, where the cursor is 0.25 j and the stored fraction is the fractional part of the previous tick's cursor. The gate is progress of 0.99 or more, or a finished clip. For a one-shot clip of F keys with F below 25 this passes in tick s + 4(F - 1) + 1; for F from 25 to 74 (here 32, 41 and 51 keys) it passes one tick earlier, in tick s + 4(F - 1), because the progress alone reaches 0.99 on the last key (the common gate rule). The state's exit is applied in that tick. The end-of-attack draws (cooldowns, exhaustion timer, rolls) are made in that same tick, and the next tick starts the choosers.

| Action | Clip | Keys | Loop | Gate tick n | Used by |
| --- | --- | --- | --- | --- | --- |
| 0 | 16 | 51 | no | 200 | maniacal laugh |
| 1 | 23 | 41 | no | not gated | triumph (hold) |
| 2 | 5 | 11 | no | 41 | death |
| 3 | 0 | 44 | yes | none | idle |
| 4 | 12 | 32 | yes | none | exhaustion |
| 5 | 24 | 19 | yes | none | walk (orbit fallback only) |
| 6 | 13 | 13 | yes | none | run (approach) |
| 7 | 18 | 7 | no | 25 | jump step 0 |
| 9 | 8 | 13 | no | 49 | flinch |
| 10 | 10 | 20 | yes | none | perch |
| 11 | 11 | 20 | yes | none | flight (loop) |
| 12 | 14 | 10 | no | 37 | take-off of a flight |
| 13 | 22 (mirrored) | 20 | yes | none | orbit, locomotion mode 5 |
| 14 | 22 | 20 | yes | none | orbit, locomotion mode 6 |
| 15 | 6 | 14 | no | 53 | melee dodge variant 2 |
| 16 | 7 (mirrored) | 16 | no | 61 | missile dodge, melee dodge variant 0 |
| 17 | 7 | 16 | no | 61 | melee dodge variant 1 |
| 19 | 4 | 19 | no | 73 | chain whip |
| 20 | 20 | 41 | no | 160 | rapid strike |
| 21 | 17 | 32 | no | 124 | chain power (never started) |
| 22 | 3 | 15 | no | 57 | body slam |
| 24 | 9 | 20 | no | 77 | head butt |

Ids 8, 18 and 26 have clips and are never requested. The state that occupies a gated clip lasts n + 1 ticks (s to s + n).

Frame bits are read from the clip's cursor in the handler's tick: tick s + j (j at least 1) sees key floor(j / 4). So key f is live in ticks s + 4f to s + 4f + 3, and tick s itself never tests a hit.

### Missile reaction (Confidence: High for the rolls and timer; Medium-High for the contact shape)

In a free state, when the reaction timer is 0: for each active hero missile, its path is the segment from its point to point + 10 x velocity. The path is tested against each bone of Gnarfas's skeleton whose bone-record length is above 0. Each such bone is a finite cylinder: bone-record +0x40 is its full length and +0x44 its radius. The posed base point is offset by half the length along the posed bone's local Y axis, centering the cylinder on the bone (SPEC-0029 Behavior 26; High). If any such path touches a bone:

1. Roll less than the missile block chance (100 in phase 1, 25 in phase 2): enter the block-missile state.
2. Otherwise roll less than the missile dodge chance (25): enter the dodge-missile state (animation 0x10, then idle).
3. Otherwise roll less than the missile jump chance (25): start the jump sequence from its first step.

In all three cases the reaction timer is then set to a uniform whole number of ticks in [60, 180] (the dodge timer values x 60), whatever the outcome. If no missile path touches Gnarfas, nothing happens. Each roll is a whole number from 0 to 100 inclusive (101 values), so the phase 1 block (chance 100) fails on a roll of exactly 100 and then falls to the dodge roll; the chances are c / 101 per roll. The three rolls are made in turn only as far as needed. A dodge-missile state lasts 62 ticks (s to s + 61); a block-missile state lasts as long as a hero missile, extended 120 ticks, still touches Gnarfas. The reaction runs once per timer expiry: while the timer is above zero a touching missile does nothing (the damage check still removes it and flinches, Taking damage).

Block missile: Gnarfas holds its block pose while any hero missile, extended 120 ticks ahead, would touch it; otherwise it returns to idle. Missiles that touch Gnarfas in this state are removed with no effect.

Details of the reaction and the block (High unless marked):

- **Timer.** The reaction is tried only when the timer is below 1. The timer is set only on a tick on which a path touched a bone, and then in all three outcomes, including the case in which all three rolls failed and the state did not change. The timer counts down on every tick of every state, so a block that lasts 40 ticks has already used up 40 of the 60 to 180. A missile that arrives while the timer is still above 0 gets no reaction and strikes the body: that is how shots reach his body in phase 1; they flinch him for 0 damage.
- **Path length.** The reaction path is 10 ticks of the missile's travel from its present point. The block state's hold test uses 120 ticks of travel from the present point. The two tests use the same bone cylinders (bones with a positive length). A missile removed by the damage check in an earlier tick no longer counts.
- **The block holds still.** The state makes no animation request, never turns the boss and never calls the walk step. Whatever clip was current stays current, so a block entered from idle holds the idle loop in place; a block entered from approach keeps playing the run clip, and the run clip's root motion keeps carrying the boss forward along its facing at 15.36 units per second until the state ends; a block entered from orbit keeps strafing the same way at 4.63 units per second. The guard base state (20) is written on every tick of the state; the loop's first step writes base state 1 again on the next tick (Medium-High for the carried-over motion, High for the missing request).
- **Exit.** The state ends on the first tick on which no armed slot's 120-tick path touches a bone cylinder, and the exit is to idle at once (a block lasts one tick if the missile is already gone). Its handler tests all six slots every tick.
- **Guard path (Medium-High).** In base state 20 a non-hero character counts as guarding for the hero's shared shot code (SPEC-0029 and SPEC-0036: a guarding target is visited even when it is not hittable). Gnarfas is never hittable for that code (see Taking damage), so the only effect is that, during the block, a Roberta quick shot with an arming counter of 0 that touches his bone cylinders bounces away (SPEC-0036, Quick shot meets a guarding target) and a full-release shot is disarmed with no damage. A bounced quick shot is not spent: it stays in flight as a friendly shot with its direction reversed and perturbed, its arming counter set to 15 (so for 15 ticks it ignores guards and does not steer), and it can touch him or another target again afterwards; his deflect flag is never set, so it is a bounce and not a deflect that turns the shot against the hero (Medium-High; High for the bounce rule, the open point is the order against his own removal below). Gnarfas's own check removes any slot that overlaps his body cylinder, which ends a shot that is still inside his body cylinder when that check runs. Neither path damages him. A hero swing meets the same guard rule once: on the first tick of its attack window the shared swing code sees a guarding target with Blocking Angle 0.0 (monster row 6), so a swing made while the two face each other (dot of the two facings below 0) is blocked and the hero recoils as against the Werelupe King's guard (SPEC-0019, Hurt side), and for Tor the swing is then over before Gnarfas's own check can read it on the next tick (High: the block throws Tor into his recoil sub-state, in which the hero's attack-state test that the own check requires is false); a swing from the side or behind has no effect in the shared code, and his own check takes it on the next tick and flinches him (High: the own check has no facing exemption, it needs only the attack-state test, the open attack window and the weapon reaching a bone cylinder). Roberta does not recoil, so a frontal swat that the shared code blocks on its first tick stays in its attack state and window, and his own check counts it on the next tick and flinches him if the weapon still reaches a bone cylinder (Medium-High); the block lasts only while a missile is about to arrive, so this matters mostly for Roberta's swats.

### Melee reaction (Confidence: High)

On the tick the hero starts attacking (rising edge), roll less than 100 (melee dodge chance) -> dodge melee. The roll is a whole number from 0 to 100 inclusive, so the dodge fails on a roll of exactly 100 (1 in 101 swings). The latch that detects the edge clears as soon as the hero is no longer attacking and is set whether or not the roll passed, so one swing gets one roll. The variant is 0, 1 or 2 (each 1/3), playing animation 0x10 (61 ticks), 0x11 (61 ticks) or 0x0F (53 ticks) and turning toward the hero with weight 0.1 every tick. The state ends when the animation ends (gate tick s + 61 or s + 53), then idle. The boss can be hit during the dodge (the damage check is active in this state).

"The hero is attacking" is the hero attack-state test (SPEC-0040, Behavior 18): Tor in action state 13 with attack sub-state 2 to 12, or Tor in the plunge attack of the jump states (air phase 8 or 9); Roberta only in action state 13 with sub-state 5 or 12 (her swats). Her shots, spins, leap and charge do not count, and no spell or Mote cast counts for her. The hero's busy list plays no part: the hub's hold test is "hero is down" and never fires on a swinging hero, so the dodge roll is reached (High).

The latch is a word of the controller. It is read and written only by this reaction, which runs only in a free state. It is cleared on every free-state tick on which the hero is not attacking, and set on every free-state tick on which the hero is attacking and the latch was clear, whether or not the roll passed. Consequences that are easy to miss (High): (1) a swing that starts while the boss is in a non-free state (any attack, flinch, dodge, block, exhaustion, jump, laugh, rage or hold) never rolls at its start; if the swing is still going on when the boss returns to idle, the roll happens on the first free tick (a late dodge in mid-swing); (2) if a swing begins and ends entirely while the boss is not free, the latch keeps its old value, so a latch left at 1 by an earlier swing suppresses the roll of the next swing until the boss has seen one free tick with the hero not attacking; (3) the roll is made after the hub, so a swing that begins in the same tick as a damage-check hit, which already made the boss flinch, is not rolled for.

The dodge steers toward the hero with weight 0.1 on every tick of the state, including its request tick, and it uses the clip's own root motion for the sidestep or the step back (see Travel for the cumulative distances). The variant is stored when the dodge is chosen, and is drawn with the shared random helper, so a draw of 0, 1 or 2 is uniform.

### Attack selection (Confidence: High for the order and tests; High for the cooldown rules)

Attack choices run only in idle, approach or orbit. They run in this order, and the first one that starts ends the choices for the tick.

1. **Block counter (unreachable, High).** The step has its own flag, which lives in the boss's own controller record. The record starts at 0, and only the chooser writes the flag, always with 0 (it reads and clears it). When the flag is set the step would start a block-counter branch, which uses the same top-level state value as the chain whip. Since the flag is never set, the step never starts an attack. The range 15.0 (0x2DADCC) and chance 35 (0x2DADD0) are stored only.
2. **Chain whip.** If the chain-whip cooldown is zero and the hero is closer than 9 units, start the chain whip. No roll is made. (The chain whip's own chance, 40, is unused.)
3. **Rapid strike.** If the rapid-strike cooldown is zero and the hero is closer than 9 units, start the rapid strike. No roll is made. (The rapid strike's own chance, 10, is unused.)
4. **Chain power.** If the chain-power cooldown is zero and the hero is closer than 9 units: roll; if the roll is less than the phase's chance (0 in both phases), start the chain power. Otherwise draw a new chain-power cooldown and continue.
5. **Head butt.** If the head-butt cooldown is zero and the hero is closer than 15 units: roll; if the roll is less than the phase's chance (40 in phase 1, 25 in phase 2), start the head butt. Otherwise draw a new head-butt cooldown.

Cooldowns are drawn fresh each time they are armed (see Setup). The chain whip and the rapid strike arm their cooldowns when they finish, and the head butt arms its cooldown when it finishes or after a failed roll.

### Movement (Confidence: High for thresholds; Medium for meaning)

Movement is chosen each tick when the state is a free state:

- If the hero is farther than 8 units, the boss goes to approach.
- Otherwise, if the hero is at least 6 units away, or the boss is already orbiting, the jump timer is armed at 30 ticks if it is at zero. Nothing else changes.
- Otherwise (the hero is closer than 6 units and the boss is not orbiting), the boss starts orbiting and a coin picks the side.

Approach (run or walk, High): the boss uses locomotion mode 1 (run, animation 6) when the horizontal distance to the hero is greater than 5.0, the run-or-walk tunable, and mode 0 (walk, animation 5) otherwise. It exits to idle once the hero is closer than 5 units. In play only the run branch is reachable, and the idle exit never runs: the movement chooser runs before the state's behavior in the same tick, and it switches an approaching boss to orbit as soon as the hero is closer than 6 (the boss is not orbiting), so the state's own tests only ever see a hero at 6 or more (High). Walk (animation 5) is used only by the orbit fallback toward the hero when the side point is off the walk mesh. The hero standing in the band 6 to 8 leaves an idle boss idle. No speed tunable exists for approach or orbit; the boss travels by the root motion of the locomotion animation (see Travel).

Approach in full (High unless marked): each tick the handler picks the mode (1 when the horizontal distance is above 5.0, else 0), sets the goal to the hero's present position and runs the shared walk step of SPEC-0056. The walk step steers with weight 0.3 only in the settled case, when the boss's current triangle equals both its stored triangle and the goal's triangle and it has not arrived (the goal is in the triangle it stands in), and with weight 0.1 in every other case: off the mesh, in the first tick in a new triangle or toward a new goal triangle, following a route over triangles, and the first tick after entering the goal's triangle (SPEC-0056, Navigation step; High, the weights are the 0.3 constant of the step and the 0.1 word of the shared data, and the case is fixed by the triangle bookkeeping of that tick). The walk step requests the mode's clip every tick. The handler's own exit (hero closer than 5, that is 8 minus 3) is tested after the walk step and cannot be reached. The boss never stops to attack from the approach: the choosers run first in the tick and take over (Attack selection), so an approach in progress is cut off by a head butt (hero closer than 15) or by a chain attack (hero closer than 9) as soon as a cooldown is at 0; the run clip's root motion then stops at once, apart from the chosen action's own motion. An approach also resumes at once after each attack: the next free tick sees a hero farther than 8 and requests the run clip (the idle request of the tick after an attack is replaced by it).

Locomotion modes are the numbers the movement uses to pick an animation. Mode 0 and 1 are walk and run for approach. Modes 5 and 6 are the strafe modes of the orbit (animations 0x0D and 0x0E).

### Travel (Confidence: High for the values at R = 1)

Gnarfas travels only by the root motion of the clip it plays, as every character does (SPEC-0040 Behavior 14C; SPEC-0056, Travel by root motion): each tick the root record of the current key times the clip speed c = R x 0.5 / S is turned by the facing basis (B1 side, with B1 = up x facing; B3 facing) and added to its position, one tick after the animation pass that produced it, with ground following as for the heroes. R = 1 (see Edge cases).

| Movement or action | Animation | Travel |
| --- | --- | --- |
| Approach walk (mode 0) | 5 | Forward root 393 to 554 per key (mean 0.4229 units per key), S = 2: 0.0960 to 0.1353 per tick, mean 6.34 units/s. |
| Approach run (mode 1) | 6 | Forward root 1048 per key, S = 2: 0.25597 per tick, 15.36 units/s. |
| Orbit (mode 5) | 0x0D | Along +B1 at 4.63 units/s on average (the root varies by key). |
| Orbit (mode 6) | 0x0E | Along -B1 at 4.63 units/s on average. |
| Dodge | 0x10 / 0x11 | 8.98 units along +B1 / 8.98 units along -B1. |
| Dodge back | 0x0F | 6.09 units backward. |
| Head butt | 0x18 | 9.57 units forward. |
| Flinch | 9 | 5.33 units backward. |
| Body slam | 0x16 | 1.24 units backward. |

Here "forward" is +B3 and "backward" -B3. One-shot distances are totals over the whole animation at R = 1. Animation entries 0x0D and 0x0E name the same clip; entry 0x0D carries the animation table's mirror flag (SPEC-0005), which plays the clip mirrored and negates the side part of its root motion. So mode 5 goes to one side and mode 6 to the other. The dodge pair 0x10 and 0x11 is built the same way.

**Clips with no root motion (High).** The maniacal laugh (0), triumph (1), death (2), idle (3), exhaustion (4), jump start (7), perch (10), flight (11), take-off (12), chain whip (19), rapid strike (20) and chain power (21) carry no root motion at all: while they play the boss stays where it is, apart from the jump flight's own velocity. So a chain whip or a rapid strike never closes the distance and never follows a moving hero; only the turn toward the hero (weight (1 - p) x 0.5 per tick) tracks. Only the actions in the table above move the boss.

**When the motion happens (cumulative root distance along the facing or side axis, units, from the request tick s; High for the clip data, Medium-High for plus or minus one tick of lag).**

| Action | s+8 | s+16 | s+24 | s+32 | s+40 | s+48 | end |
| --- | --- | --- | --- | --- | --- | --- | --- |
| Dodge side (0x10 / 0x11) | 4.6 | 7.5 | 8.3 | 8.7 | 9.1 | 9.0 | 9.0 at s+56 |
| Dodge back (0x0F) | 1.4 | 4.0 | 5.3 | 5.8 | 6.0 | 6.1 | 6.1 |
| Flinch (9), backward | 2.4 | 3.8 | 4.9 | 5.3 | 5.3 | 5.3 | 5.3 |
| Head butt (0x18), forward (negative first) | -1.2 | -1.3 | 1.0 | 13.6 | 14.2 | 12.9 | 9.5 at s+77 (14.2 near s+40; 11.9 at s+55) |
| Body slam (0x16), forward | 1.0 | 1.0 | 1.0 | 1.0 | 0.3 | -1.4 | -1.2 at s+57 |

Reading: the sidestep dodge is almost finished by tick 16 (the hero's weapon window for most Tor swings starts at about tick 8 and runs to tick 40), so a dodge started at the swing's first tick puts the boss 4.6 units to one side at the first possible hit tick. The head butt first rocks back 1.3, then lunges about 13 units forward between ticks 22 and 34 (7.8 at tick 28), and ends 9.5 units ahead of the start; its hit window (ticks 24 to 55) therefore covers a boss that is already 1 to 14 units forward, and the hero's distance is tested each tick against the boss's present position.

**Edge guard (High; SPEC-0040, Behavior 14E).** The shared motion step reverses a tick's root step when the boss's height above the ground afterwards exceeds 2.0 in magnitude and the motion lock word is 0. The lock word is cleared at the top of each controller tick and set only by the jump state, so in every state except the jump sequence the boss cannot be walked or knocked off a ledge more than 2.0 high (or into a hole deeper than 2.0): its root motion simply stops there. During a jump sequence the guard is off. A flinch taken while perched on a tree or in flight (lock word cleared, height above ground over 2.0) therefore loses the flinch's backward slide, and the boss then drops under normal gravity.

### Facing by locomotion mode (Confidence: Medium)

This rule is shared by all characters that use the generic movement routines; SPEC-0040 Behavior 8a is the authoritative description (including the stored-facing blend and the caller weights 0.1 and 0.3). This section summarises it for Gnarfas.

Each movement step turns the boss toward its move direction through a shared steering helper. The helper's rule depends on the locomotion mode number m that the movement has set:

- **Modes 3, 4 and 8** (backward walk): the target direction is the negated move direction. The blend weight is the one the caller passed.
- **Modes 5, 6, 9 and 10** (strafe): the target direction is the horizontal direction perpendicular to the move direction. Modes 5 and 9 take the perpendicular on one side; modes 6 and 10 take it on the opposite side. The blend weight is 0.1 per tick while the dot of the boss's forward direction with the target is at most 0.9; when that dot is above 0.9, the weight is 1.0 (snap).
- **Any other mode**: the target is the move direction, with the caller's blend weight.
- **Caller blends.** The default blend is 0.1 for the strafe call and 0.3 otherwise (the 0.1 is the data word at 0x2DD86C; the 0.3 is a fixed value). The blend passed by the caller applies in the first and third bullets.

The new heading is the normalized blend of the target and the current heading: normalize(w × target + (1 − w) × current). The current heading is the boss's stored facing vector, the same one the other steering calls use.

In effect, while orbiting in mode 5 or 6, the boss keeps facing across its motion (turning 0.1 per tick toward the perpendicular, then snapping to it), and while a move uses mode 3, 4 or 8, the boss faces away from its move direction. The perpendicular is taken against the constant up vector (0, 1, 0, 0) at 0x3932E0. For move direction m, modes 5 and 9 face up x m = (m.z, 0, -m.x), and modes 6 and 10 face m x up = (-m.z, 0, m.x) (High).

### Orbit (there is no separate back-off state; Confidence: High)

Entered when the hero is closer than 6 units and Gnarfas is not already orbiting. A coin picks the side. The move point is 3 units from Gnarfas toward that side (perpendicular to the hero direction), and Gnarfas strafes toward it in locomotion mode 5 (animation 0x0D, the mirrored clip, moving along +B1) or 6 (animation 0x0E, moving along -B1), at 4.63 units/s on average. When the movement reports it is blocked, the side flips, at most once every 30 ticks. If the side point is not on the walk mesh, Gnarfas walks toward the hero instead. Orbit has no exit of its own: the approach choice (hero farther than 8) or an attack ends it.

Orbit in full (High unless marked):

- **Side and point.** The coin is a draw from 0 to 1 made when the movement chooser enters orbit, and stored as the side (0 is mode 5, 1 is mode 6). Each tick the orbit handler recomputes the point: take the direction from the boss to the hero, cross it with the up vector, normalise, and move 3.0 along it from the boss's position (side 0) or minus 3.0 (side 1). The hero's height does not matter (the cross product drops it). The point is always 3 units away, so the walk step's arrival distance of 0.25 is never reached.
- **Side flip.** The orbit handler keeps a flip timer that it owns. Each tick: if the motion record's touched flag (the contact flag the previous tick's motion pass left; High: the motion pass clears it at its start and sets it whenever any contact response runs, that is a push out of a wall or solid volume, a push back at a navigation-mesh border, or a push from another character; the think runs before the motion pass of the same tick, so the orbit handler always reads the previous tick's result) is set and the timer is below 1, the timer becomes 30 and the side flips; then, whatever happened, a timer above 0 counts down by 1. After a flip the timer reads 29 at the end of the tick, so the next flip is possible 30 ticks later. The timer is not reset when the boss leaves orbit, so a later orbit starts with whatever is left of it (0 in practice, since orbit stints are long; it can still make the first tick of a new orbit flip at once if the touched flag is set from an approach step).
- **Off-mesh fallback.** After setting the mode and point, the handler tests whether the point is a valid position on the walk mesh. If it is not, the mode becomes 0 (walk) and the goal becomes the hero's position, so for that tick the boss walks toward the hero at 6.34 units per second on average, turning at the route weight of SPEC-0056. This can carry it inside 6 without leaving orbit (orbit is never left by distance below 8). It is the only way the walk clip (action 5) is played.
- **Walk step.** The shared walk step of SPEC-0056 then runs (steering by the mode rule, the locomotion clip request, route following). It requests the mode's clip on every tick. A fresh navmesh block counts as arrival in that step and puts the boss in the walk step's idle picker for the tick (Medium; the orbit handler repeats its own work on the next tick).
- **Entry.** The movement chooser enters orbit on the tick the hero is closer than 6 and the boss is not already orbiting, from idle or approach. The entering tick runs the orbit handler in the same tick (step 8), so the first strafe request and the first step happen at once.

### Jump to tree (Confidence: High for steps and exits; Medium for the flight meaning)

The jump sequence starts from the rapid strike (when the exhaust roll fails), from the end of exhaustion, or from a missile reaction. Its steps are numbered 0, 1, 2, 3 and 5 (step 4 is unused). Four tree points are used.

- **Step 0**: animation 7. Then step 1.
- **Step 1**: pick the tree nearest to Gnarfas in the horizontal plane and jump to it. Velocity per tick: v = (dx / 60, dy / 60 + 1.5, dz / 60), where (dx, dy, dz) is the offset to the tree point. Each tick v.y decreases by 0.05 and Gnarfas moves by v, playing animation 0x0B and turning toward its motion (weight 0.2). When the horizontal distance to the tree is below 3, it perches: step 3 with wait 0.
- **Step 3**: animation 10, no motion; count the wait down. When it is 0 or less: roll less than the jump-to-hero chance (50) -> step 5, aimed at the hero's position at that moment. Otherwise step 2, aimed at one of the other three trees, chosen uniformly.
- **Step 2**: the same flight (animation 0x0C, then 0x0B). When the three-dimensional distance to the tree is below 3: step 3 with a wait drawn uniformly in [0, 180] ticks.
- **Step 5**: the same flight toward the stored hero point. It lands when its height above the ground is 1.0 or less (High; this is the landing rule for every jump in the sequence). Then, if the hero is closer than 5 units, body slam; otherwise idle.

Flight timing (Confidence: High for the rules; the counts depend on the positions). Step 0 lasts 26 ticks (s to s + 25, animation 7). The boss is hittable in every step (a hit gives a flinch, which ends in idle and drops the boss under normal gravity). In steps 1, 2 and 5 the boss's no-gravity flag is set every tick and it is moved only by the velocity v. Order inside one flight tick: lower v.y by 0.05, then add v to the position (the turn toward the motion, weight 0.2, runs when the facing is not yet within 0.99 of the motion direction). The first tick of a flight computes v; step 1 does not move on that tick, steps 2 and 5 do. Because v.x and v.z are the offset divided by 60, a flight spans about 60 ticks if it were allowed to complete; the perch test ends step 1 and step 2 earlier: the horizontal remaining distance D(1 - k / 60) falls below 3 after about 60(1 - 3 / D) moves (step 1 from the spawn point toward tree 104: D = 17.2, about 50 moves). Step 5 aimed at the hero on flat ground lands after 59 moves (the height above the ground first reaches 1.0 or less on the way down: 1.475k - 0.025k squared, with k = 59 giving 0). Flight animation: step 1 requests 11 at once; steps 2 and 5 request 12 when the sub-step is entered from the perch and switch to 11 when action 12's clip gate passes (37 ticks); the perch plays animation 10. The perch wait of step 3 is 0 after a first flight, a uniform whole number in [0, 180] after a flight to another tree, and is checked from the tick after arrival.

Jump sequence in full (High unless marked):

- **Motion lock.** Every tick of the jump state sets the motion lock word (edge guard off, Travel) and, in steps 1, 2, 3 and 5, the no-gravity flag. Both are per tick, so they lapse as soon as another state runs.
- **Tree choice.** The nearest tree is chosen once, on the first tick of step 1, by horizontal distance from the boss's position at that moment; on a tie the lower tree number wins. The tree index is kept between sequences but is chosen again at every new sequence. After a perch, "another tree" is one of the three trees other than the present one, each with probability 1/3 (a random whole number modulo 3 picks among them).
- **Perch wait.** The perch decreases its wait by 1 first, then tests it. Wait 0 therefore ends on the first perch tick; a wait w drawn after a tree-to-tree flight ends on the w-th perch tick (the first if w is 0 or 1). The perch clears the velocity and requests animation 10 on each of its ticks.
- **Perch exit.** At the end of the wait a roll from 0 to 100 inclusive below 50 sends the boss at the hero (50 / 101), else to another tree. The hero's position is stored at that moment and the flight is not retargeted: a moving hero is simply missed. Either way the take-off clip (action 12) is requested on that tick and the flight begins to move on the next tick.
- **Landing height.** The height above the ground used by the landing test is the ground gap word the motion pass left on the previous tick; it reads 0 while the floor-contact flag is set. The test runs before the tick's move: if the gap is above 1.0 the boss moves; otherwise it lands in that tick and does not move. A leap from a perch that is not above 1.0 over the ground lands on its first tick without moving. The four tree points sit 4 to 5 units above the hero's start height (122.7), so this is not expected in the shipped arena (Medium; the ground height under each tree was not measured).
- **After the landing.** The no-gravity flag stops, the lock word is cleared by the next tick's top step, and the boss is in body slam if the hero is closer than 5.0 horizontally (the slam's strike latch cleared), else idle. In idle the chooser runs on the next tick, so a whip or rapid strike can follow a landing at once.
- **Typical length.** Step 0 lasts 26 ticks and the first flight about 50, then at each perch the leap at the hero has probability 50 / 101 about one half, so there are two perches on average; each extra tree-to-tree flight costs about 55 ticks plus a wait averaging 90. A sequence from the end of the rapid strike to the landing therefore takes about 280 ticks on average (140 at the shortest, with no upper bound; Medium), then 57 ticks of slam when the hero is near. The whole time the boss is hittable and does not react to anything (the reactions, hub and choosers do not run in the jump state), except that a flinch cancels the sequence and returns to idle.
- **Flinch inside the sequence.** A hit during any step starts a flinch; at its end the state is idle and the sequence is lost (no step is resumed). The jump variables are cleared again by the next sequence's starter. The boss drops under normal gravity if it was in the air (the edge guard may cancel the flinch's backward slide, Travel).

### Body slam (Confidence: High)

Started only by the landing of step 5: no other state write in Gnarfas's code sets the body-slam state (High). It has no shape test and reads no event bits: on any tick of its clip (0x16) it strikes once when the hero's horizontal distance is below 5.0. The strike deals 30 damage (the body-slam damage, word 0x2DAE2C) with a knockback of strength 1.0, and adds one to the strike count. A laugh (maniacal laugh) follows after a hit with a roll below 30.

Airborne skip (High): when the strike would land while the hero is in a jump or air state (action states 7 to 10, 11 or 43 to 46), it deals no damage and no knockback, but the strike latch is still set and the strike count still rises, so the slam cannot land later in the same clip. A hero who is airborne on the first tick in which he is within 5.0 is therefore safe from the whole slam, even if he lands a tick later. The body slam is unblockable (the guard is never tested). The strike's own tick is the first tick of the clip after the request tick in which the horizontal distance is below 5.0, so a hero already in range at the landing is struck on the first slam tick.

### Head butt (Confidence: High)

No damage. When its hit test passes (event bit 0x1, which is on in frames 6 to 13 of the 20-frame clip; the hero in front of Gnarfas, dot(boss facing, boss-to-hero direction) above 0.5; and the hero's horizontal distance below 7.0), the hero is knocked back with strength 2.0. There is no shape test beyond the facing and the distance (High).

### Knockback (Confidence: High)

A knockback of strength s:

- It runs only when the hero is not down (action state not 16; the same test as the hub's hold). Otherwise nothing changes. A killing hit therefore pushes nobody: the take-hit has already written state 16 by the time the push is requested (SPEC-0036, Health and death; the default death runs because module 985 registers no death callback, Medium).
- The hero's velocity is cleared, then 12.0 is added to its vertical part and 9 d to its horizontal part, where d is the horizontal unit direction from Gnarfas to the hero.
- The result is normalized and scaled by 8.0 x s (8.0 is the push speed word 0x2DAE78): velocity = normalize(9 d + 12 up) x 8 s. The hero then enters reaction state 0x30.
- So the push is 4.8 s horizontally and 6.4 s vertically, in the hero body's velocity units (SPEC-0040).

### Chain whip and rapid strike (Confidence: High)

- Both turn toward the hero with weight (1 - p) x 0.5, where p is the animation progress from 0 to 1.
- **Damage is raw.** Each hit sends its damage (15, words 0x2DADE4 and 0x2DADFC) straight to the hero's take-hit; the hero's shield, armour, armour Mote and petpet factor do not reduce it (SPEC-0029 Behavior 10.5).
- **Block still deals full damage.** If the hero is guarding and faces Gnarfas (dot of the hero's forward direction with the direction from the hero to Gnarfas above 0.5), the take-hit runs with the full 15 and the blocked flag set: the hero loses 15 HP and plays its block reaction instead of its hit reaction, and there is no knockback. Otherwise the hit is unblocked: 15 with the hit reaction and a knockback of strength 1.0. The "hero not down" check before the knockback is the hero's busy condition, the same as the hub's hold test (High).
- **Once per attack.** The chain whip hits once per attack. The rapid strike's hit latch would be re-armed only on a frame carrying event bit 0x20, and no frame of its clip carries that bit, so the rapid strike also hits at most once per attack, although its clip has four windows.
- **Order inside one handler tick (High).** Request tick: the clip is requested and nothing else happens (no turn, no hit test). Every later tick before the gate: turn toward the hero (weight (1 - p) x 0.5, p the clip progress at that moment), then, only if the attack's hit latch is clear, the bone-chain test and, on a hit, the take-hit. Gate tick: no turn and no hit test; the exit and the end draws. The hit latch is cleared when the attack is chosen, and for the rapid strike also on any tick that shows frame bit 0x20 (none does).
- **What the chain test reads (Medium-High).** The frame bits come from the clip's current key. The boss's bone poses and the hero's cylinder are those left by the previous tick's animation pass (SPEC-0036, Tick order of the tests), so a window opens one pose behind the clip. The tests use the hero's present position for the cylinder, whatever its state; a hero in the air is hit if the cylinders overlap, and the hero's hit reaction (state 17) does not prevent a hit (the hero has no invulnerability window, SPEC-0036).
- **The strike count (High).** It rises by 1 on each whip hit (blocked or not), rapid strike hit, head butt hit and body slam strike (including a skipped one), at the moment the latch is set. It is never reduced. A hit on a downed hero still counts, because the latch and the count do not look at the hero's state; the take-hit then returns at once (SPEC-0036, Taking a hit, step 4).
- **Where the hero is when the whip lands.** The hero is struck whenever any bone of the selected run overlaps his cylinder, with no distance or facing condition on the boss: the hero can be behind him. The chain whip's range condition (hero closer than 9) is only the start condition; a hero who walks out of the reach during the attack is simply missed. The turn weight (1 - p) x 0.5 is 0.5 at the start and falls linearly toward 0 as the clip ends (0.25 at the half way point, about 0.1 in the last fifth), so the boss follows a circling hero well early in the attack and poorly late in it; it never moves from its spot.
- **Blocked and unblocked (High).** The hero guards by SPEC-0036's guard predicate (shield up through the guard stance, Roberta's spin, or the 40-tick guard timer raised by a nearby attacker), and the facing test is that the hero's forward vector dotted with the unit vector from the hero to the boss is above 0.5 (the hero faces him within 60 degrees). The facing test is made only when the hero guards. An unblocked hit sends the hero into the hit reaction (state 17) and then the knockback of strength 1.0 (Knockback) replaces it with state 0x30, unless the hero is dead; a blocked hit plays the block reaction (animation 0x40) and no knockback.
- **Windows** (frames with the event bit on, from the boss's own clips in the pack 985 record; monster row 6, whose five weapon volumes (radius 3.0, length 0.5 each) are never tested by his attacks):

| Attack (animation) | Windows | Damage |
| --- | --- | --- |
| Chain whip (0x13) | bit 0x1: frames 7 to 13 of 19 | 15, once |
| Rapid strike (0x14) | bit 0x4: frames 6 to 10; bit 0x2: frames 10 to 13; bit 0x1: frames 18 to 23; bit 0x8: frames 25 to 29 (of 41) | 15, once |
| Chain power (0x15) | bits 0x1, 0x2, 0x4 and 0x8: frames 9 to 17 of 32 | 13 (unreachable) |

- **Bone-chain test.** On each tick, the event bits on in the current frame select runs of the boss's bones, tested in this bit order: bit 0x1 bones 29 to 33, bit 0x2 bones 15 to 19, bit 0x4 bones 46 to 49, bit 0x8 bones 38 to 41. Bones are tested in order within a run, and the first bone that overlaps ends the test with a hit. Each bone is a finite cylinder of radius 0.5 and length 1.25 that starts at the bone's posed origin and runs along the bone's Y axis (its centre is origin + 0.625 x Y). The hero is a vertical cylinder of radius 1.0 and length 3.0 standing on the hero's position (its centre is the position + 1.5 up). The overlap is the static finite-cylinder test of SPEC-0018. On a hit, the chain spark (effect 710) of the hitting run's tip bone plays: bit 0x1 bone 33, 0x2 bone 19, 0x4 bone 49, 0x8 bone 41 (see Sounds and effects).

### Chain power (Confidence: High for its gate; Medium for its hit)

The chain power has no guard test in its hit. It uses the bone-chain test with all four runs (bits 0x1 to 0x8, frames 9 to 17). When it hits, it deals 13 damage (word 0x2DAE18, raw) and a knockback of strength 1.0. It can only start when its roll passes, which the shipped chances (0 and 0) never allow.

### Arena flow (level script module 985; owned by SPEC-0041; Confidence: High for the calls and order unless noted)

The fight runs in the exterior world whose level script is module 985, in the branch taken when game state 281 is clear (the branch with 281 set is the Gelert Assassin fight, SPEC-0013). Gnarfas is placed in that world pack as record 7 of the embedded block (entity 7). No placement flag or host call activates him: the module's activation filter admits ordinal 7 when flag 281 is clear, the character streaming binds him on the first pass after the hero is placed at locator 9 (21 units away), and his controller runs from the bind. The pack's Werelupe King (entity 8) is bound the same way but is grabbed by the script on the arrival tick and stays held (SPEC-0019, Activation in play).

- **On world activation** (game state 281 clear): the hero is moved to locator 9, the normal camera is set (3 s), and game state 212 is set. If game state 214 (Gnarfas beaten) is clear, level states 0, 1 and 2 are cleared and the boss-fight refill runs (boss-fight instance 1, stage 0).
- **Phase 2 hand-over.** Each tick the script's control thread checks: if level state 0 is set and level state 1 is clear, it plays movie 12 (25.63 s) and waits for it, moves the hero to locator 10, moves entity 7 (Gnarfas; Medium-High) to position 12, sets the normal camera (3 s), sets game state 213, sets level state 1, sleeps 30 ticks and shows dialogue line 14 (and waits for it). Gnarfas then sees level states 0 and 1 set and enters phase 2 once, healing the hero to full (Phase hub, step 3).
- **Defeat.** When the thread sees level state 2 (Gnarfas's death event) and has not yet run the defeat, it runs the defeat sequence: effects 7 to 12 are stopped, entities 7 and 8 are deactivated, game state 214 is set, a world collision mask is set and, behind further tests, the boss-fight call runs for instance 1, stage 1 (Medium for the gating of this last call).

### Sounds and effects (Confidence: High unless marked)

**Sounds come only from the animation table.** The controller starts no sound of its own. Gnarfas's main animation table (the record's own table in the pack 985 embedded block) names cues of creature block 393 (SPEC-0031). Every sound entry has start frame 0 and no keep-alive bit, so when the main animation changes to an entry with a sound, that cue plays once, at the boss, on the first tick of the animation. A request of the animation that is already playing starts nothing. No sound plays while the boss is in base state 42 (the animation-sound step is skipped there; SPEC-0031).

- Cues of block 393 by action id (decimal), from the table of the pack 985 record: 0 (maniacal laugh) cue 4; 1 (triumph) cue 4; 2 (death) cue 6; 9 (flinch) cue 3; 16 and 17 (dodges), 22 (body slam) and 24 (head butt) cue 5; 19 (chain whip) cue 0; 20 (rapid strike) cue 2; 21 (chain power, never started) cue 1. Ids 18 and 26 also carry cues (5 and 4) but are never requested.
- Silent: 3 (idle), 4 (exhaustion), 5 and 6 (walk, run), 7 (jump start), 8, 10, 11 (perch, flight), 12 (take-off), 13 and 14 (orbit), 15 (melee dodge variant 2), 23 and 25.
- Cues 0 to 6 of block 393 are all used. The earlier reading that every entry names cue 0 comes from the copy of this table in region 1264, which a pack 985 fight does not use; do not apply it.

**Effects started by the code.**

- **Chain sparks.** At spawn Gnarfas creates four effect objects of effect 710, one per chain tip bone: 19, 33, 41 and 49. Effect 710 is 35 white sparks, once, 1 s.
- A chain hit on the hero (chain whip, rapid strike or chain power) plays the object of the hitting run's tip bone: bit 0x1 bone 33, 0x2 bone 19, 0x4 bone 49, 0x8 bone 41.
- Each tick, a tip bone whose height is below the ground under it (the ground query at the tip's horizontal position with ceiling = tip height + 10.0, SPEC-0018; the tip is below ground when its height minus the found ground height is negative) plays its object once. A latch per bone holds until that bone is above the ground again, so a tip that stays in the ground sparks only once.

**Clip effect lists** (SPEC-0034; Medium-High for which list belongs to which clip): animations 2, 7, 8 and 22 show effect 1609 over the whole clip on attach slot 6 (13 brown sprites, once); 10 and 12 show 1609 with positional mode 0; 24 shows 1609 from 0.1 of the clip; 21 shows effect 936 at 0.85 (blue and grey models).

### Winning sequence (derived from the rules and the disc data; not played in the original; High unless marked)

1. Start. Pack 985 with flag 281 clear (Act 1, Tor only, swap mode 0). Arrival moves the hero to locator 9, refills the first hero and clears level states 0, 1 and 2; on the next script tick Gnarfas is moved to locator 11 (21.4 from the hero). There is no intro scene: he is free from the arrival tick and approaches at once. Nothing waits on the hero.
2. Fight (225 HP, phase 1). Only Tor's sword lowers his health; shots and Motes flinch him for nothing. Strike after one of his attacks ends, while he is exhausted or laughing, or in the jump sequence; one swing hurts him at most once because the flinch lasts 49 ticks.
3. Phase 2 (optional for the win). When the hero is below half health or Gnarfas has struck three times, event 0 sets level state 0; the script plays movie 12, moves the hero to locator 10 and Gnarfas to locator 12, sets flag 213 and level state 1, waits 30 ticks and shows line 14 (the player closes it). Gnarfas enters phase 2 once, the hero is healed to full, and the missile-block and head-butt chances and the exhaustion time change. A hero who is never struck three times and stays above half health never sees phase 2, and the fight can still be won, because the death check does not depend on it.
4. End. Health below 1 percent after a flinch, or in a free state: the death clip (41 ticks), then event 2 every tick; the defeat routine runs once (effects 7 to 12 stopped, Gnarfas and the King deactivated, flag 214, north-east wall off). Next: movie 13, a fade, the hero placed at locator 26 with control, Illusen standing in the open passage 62.9 ahead. The hero walks about 45 units north-east to the slab (trigger 4): flag 64 and movie 14, back to the village. The game shows no marker; Illusen's looping animation is the only guide.
5. Waits with no timeout. None for the boss; the line-14 box waits for the player. If the hero is down (action state 16) the boss holds in its triumph state and does not die until the hold ends (a flinch that ends at 0 health still kills it at once). In Act 1 a downed Tor is game over.
6. Cannot-finish check: none found. Every required hit is melee; Tor's swing reaches him on mesh 1.

## Edge cases and error handling

- **Mid-fight movie at once.** If the hero is already below half health when the fight starts, event 0 fires on the first tick, so the phase-2 movie plays at once.
- **Chain power is unreachable with shipped data.** Both phase chances are 0, so the roll can never be less than them. Its exhaustion and maniacal-laugh branches therefore never run from it. Reproduce this.
- **Chain whip and rapid strike never roll.** Their chance tunables (40 and 10) are stored but unused. Each fires whenever its cooldown is zero and the hero is in range.
- **Head butt deals no damage.** Its only effect is the knockback of strength 2.0. There is no head-butt damage tunable.
- **Rapid strike hits once** (High; reproduce): its clip has four hit windows, but the re-arm needs event bit 0x20, which no frame of the clip carries, so only the first hit of the attack lands.
- **A block does not save HP against the chains** (High; reproduce): a guarding hero facing Gnarfas still loses the full 15 to a chain whip or rapid strike; the block only swaps the hit reaction for the block reaction and skips the knockback.
- **Boss hits ignore defences**: none of Gnarfas's hits is reduced by the hero's armour or shield.
- **Block counter is unreachable (High).** Its flag is in the boss's own controller record, starts at 0 and is only ever written with 0, so the block-counter step never starts an attack. Its tunables (range 15, chance 35) are stored only.
- **Wait state is unreachable (High).** The wait flag is cleared at start-up and on every world load and never set, so the wait state never runs; the other tests on the same flag (for example the hero's take-hit lock test) always see it clear.
- **Jump timer.** It is armed and counted but never used to start anything.
- **Rage is one shot.** It is armed once, and a later drop below 25 percent does not arm it again. If health is also at or below 1 percent in the same tick, death replaces rage.
- **Rage only sets the flag.** The rage state does nothing else.
- **Death at below 1 percent.** Death uses the truncated percent, so a boss at 0.5 percent of its maximum dies, and a boss at 1 percent does not. With 225 health this means 1 or 2 health points left is dead and 3 is alive (3 / 225 = 1.33 percent). Death is only checked in free states, and not while the hero is down; the end of a flinch tests it without those conditions.
- **Hold can outlast zero health.** A boss at 0 percent that is in a free state while the hero is down holds instead of dying, until the hero is up again.
- **Event 0 is sent once.** The test runs only while level bit 0 is clear, and the send sets it, so event 0 is sent on the first tick its condition holds and not again (unless the script clears level state 0, which happens only on world activation).
- **Phase 1 exhaustion is zero ticks.** Both phase-1 exhaustion values are 0, so the exhaustion state ends on its first check and the boss goes straight to the jump sequence. Phase 2 uses 2 to 3 seconds (120 to 180 ticks, inclusive, drawn uniformly).
- **Mode 5 or 6 facing with a blocked move.** The perpendicular target flips with the blocked side flip (Orbit), and the facing turns at 0.1 per tick again from the current heading, so the boss turns for several ticks after each flip before it snaps (Medium).
- **Sounds per action.** Each sound entry names its own cue of block 393 (Sounds and effects); idle, walk, run, orbit and flight are silent. There is no whip-on-everything quirk in the pack 985 table.
- **Animation-speed tunable (High).** Each class-coroutine tick directly writes the tunable to P after the NPC update's reset/status processing and before clip re-time. A non-1.0 value changes that tick's clip rate and replaces any status multiplier already accumulated in P. The shipped value is 1.0, so the shipped build plays at the normal rate and this parameter has no visible effect with its current data.
- **Unused tunables.** These are stored and registered, but no code in the boot program reads them: exhausted chance 1 and 2, near-boss distance, melee jump chance, chain whip chance, chain whip head-butt chance, rapid-strike chance, jump land distance, jump attack trigger range, body slam cooldown, head-butt prepare chance, switch-to-walk percent, orbit range, chance min and max, and injured stance percent (16 values). The block counter's range and chance are also not used. No load or store to any of the 16 listed words exists in the boot program (High).

## Test vectors

- With the boss in idle and the hero not down: health 25 of 100 (p = 25) gives no rage. Health 24 of 100 (p = 24) with the rage flag clear arms rage once.
- Same conditions: health 0.5 of 100 (p = 0) gives death. Health 1 of 100 (p = 1) gives no death.
- Chain whip cooldown drawn with min 2 and max 4: a uniform whole number from 120 to 240 ticks, inclusive.
- Head butt, phase 1, hero 14 units away, cooldown zero, roll 39: head butt starts (39 < 40). Roll 40: no head butt, and a new head-butt cooldown is drawn.
- Head butt, phase 2, same case: roll 24 starts it; roll 25 does not.
- Chain power, any phase, hero 8 units away, cooldown zero: no roll value starts it (roll less than 0 never succeeds).
- Chain whip, hero 8.9 units away, cooldown zero: starts without a roll.
- Chain whip, hero 9.1 units away: does not start from the chain-whip step.
- Rapid strike exhaust: roll 74 leads to exhaustion (74 < 75); roll 75 leads to the jump sequence.
- Phase 2 exhaustion timer: uniform whole number from 120 to 180 ticks, inclusive. Phase 1: exactly 0 ticks.
- Missile, phase 1: roll 99 with block chance 100 -> block missile. Phase 2: roll 24 with block chance 25 -> block missile (24 is less than 25). Roll 25 -> not a block; then the dodge roll 24 with dodge chance 25 -> dodge missile.
- Missile reaction timer: a draw of 60 to 180 ticks, inclusive.
- Landing: a jump whose height above the ground is 1.0 lands; a height of 1.1 does not.
- Body slam: hero 4.9 units away when the landing happens: one strike of 30 and a knockback of 1.0; hero 5.0 units away: no strike.
- Melee: hero starts attacking, roll 99 -> dodge melee; variant 2 plays animation 0x0F.
- Movement: hero 8.5 units away starts approach. Hero 7 units away arms the jump timer at 30 ticks if it is zero. Hero 5.5 units away starts orbit.
- Approach exit: an approaching boss with the hero 5.9 units away switches to orbit in that tick (the movement chooser runs before the approach's own exit test); a hero 4.9 units away gives orbit as well, never idle.
- Approach travel: 60 ticks of run (animation 6) on flat ground cover 15.36 units; 60 ticks of walk (the orbit fallback only) cover about 6.34 units.
- Orbit side: mode 5 moves the boss along +B1 (with facing (0, 0, 1), B1 = (1, 0, 0), so toward +x); mode 6 along -B1.
- Approach animation: an approach that starts with the hero 8.5 units away and continues to 6.2 plays animation 6 throughout. An approach never plays animation 5.
- Facing, mode 5 (synthetic): the move direction is along +z and the target perpendicular is along +x. The boss faces -z (dot with +x is 0). One tick at weight 0.1 gives heading normalize(0.1 × (1,0,0) + 0.9 × (0,0,-1)). Once the forward direction is within dot 0.9 of +x, the weight is 1.0 and the heading snaps to +x.
- Facing, mode 3 (synthetic): move direction +z, caller weight 0.5. The target is -z; the new heading is normalize(0.5 × (0,0,-1) + 0.5 × current).
- Probability: roll less than 75 has probability 75 / 101, about 0.74.
- Knockback strength 1.0, d = (1, 0, 0): the hero's velocity becomes (4.8, 6.4, 0) (magnitude 8), then hero state 0x30.
- Head butt knockback, strength 2.0, d = (1, 0, 0): (9.6, 12.8, 0).
- Knockback while the hero is down (state 16): no change. Knockback while the hero is in the hit reaction (state 17, the state a normal unblocked whip hit has just written) or swinging: applies, and the hero ends in state 0x30.
- Facing, mode 6 with m = (0, 0, 1): target m x up = (-1, 0, 0).
- Missile hit: a slot sphere of radius 0.5 whose centre is 0.4 outside the body cylinder's side wall overlaps; the slot is removed and Gnarfas flinches with stored damage 0 (his health is unchanged when the flinch ends; Roberta's magic and charge are unchanged). The same in the block-missile state: the slot is removed and nothing else happens.
- Chain whip, hero guarding with dot(hero facing, hero-to-boss) 0.6: take-hit 15 with the blocked flag; HP 100 becomes 85, block animation 0x40, no knockback. Dot 0.4: HP 85 with the hit reaction and a knockback of 1.0.
- Rapid strike (disc clip): a hero standing inside the chains through all four windows takes one hit of 15 per attack.
- Body slam with the hero in state 8 and 4.0 units away on the landing: no damage, no knockback, and no later strike in that clip.
- Event 0: hero 49 of 100, strikes 0, level bit 0 clear: bit 0 set this tick. Hero 50 of 100, strikes 3: set. Hero 50 of 100, strikes 2: not set.
- Sounds (disc table of the pack 985 record): entry 0 is block 393 cue 4. The main animation changes to 9 (flinch): cue 3 plays once at the boss. Changes to 19 (chain whip): cue 0. Changes to 20 (rapid strike): cue 2. Changes to 3 (idle) or 13 (orbit): no sound.
- Chain-spark ceiling: a tip at height 2.0 over a floor at 0.0 with a ledge at 11.5 above the same spot: the ceiling is 12.0, the ground found is the ledge (11.5), so the tip counts as below ground and sparks. With the ledge at 12.5 the found ground is the floor and no spark plays.
- Flinch damage: a hero melee hit whose damage query returns 7.9 stores 7; the flinch ends and health drops by 7.
- Chain spark: a chain whip hit on the hero in a bit 0x1 frame plays the effect-710 object of bone 33. Bone 41 dips below the ground on tick t and stays below until tick t + 30: its object plays once, on tick t. It rises and dips again: it plays again.
- Phase 2 hand-over: level states 0 set and 1 clear: movie 12, hero to locator 10, game state 213, level state 1 set, 30-tick sleep, dialogue line 14; the next Gnarfas tick enters phase 2 and sets the hero's health to its maximum.

- Hold: the boss in idle, the hero in state 13 (swinging), 17 (hit reaction), 7 (jump) or 0x30 (knocked back): no hold, the choosers run. The hero in state 16: triumph from the next free tick.
- Melee latch: the boss in a chain whip when the hero begins a swing (latch 0). The whip ends in tick E and the swing is still going on: in tick E + 1 the boss is idle, the hero is attacking and the latch is clear, so one roll is made (99 or less: dodge) and the latch is set. A second swing that begins and ends inside a flinch leaves a latch of 1 untouched.
- Reaction timer: a phase 1 shot touches a bone in tick 100 with the timer at 0: roll 40 gives block, timer drawn 60 to 180 (say 100). A second shot arrives in tick 130: the timer is 70, no reaction, the shot overlaps the body cylinder, is removed and the boss flinches for 49 ticks with his health unchanged (damage 0). A shot in tick 210: the timer is 0, a new reaction.
- Reaction odds: phase 1 per touching shot with the timer at 0: block 100 / 101 = 0.990, dodge 0.0025, jump sequence 0.0018, nothing 0.0056. Phase 2: block 25 / 101 = 0.248, dodge 0.186, jump sequence 0.140, nothing 0.426.
- Block from approach: the boss runs, a shot's path touches a bone: block. For every tick the state lasts the boss still advances 0.256 units along its facing on the run clip's root motion, with no turn; at the first tick with no touching path it is idle and the run request stops.
- Back-to-back flinch: the flinch gate passes in tick E (state idle, damage applied). A melee hit in tick E + 1: flinch state again, the clip is still the finished flinch clip, the gate is open, the second damage is applied in tick E + 1 and the state is idle (or death) in the same tick.
- Death percent: 225 maximum health, flinch damage leaves 2: trunc(100 x 2 / 225) = 0, death. Leaves 3: trunc(1.33) = 1, idle.
- Phase 2 exhaustion: the rapid strike ends in tick E with a roll of 10: exhaustion with a timer of 150. The hero hits in tick E + 60: flinch to tick E + 109, damage applied, the timer reads 41 in that tick, the boss returns to exhaustion, and the exhaustion handler sees the timer below 1 in tick E + 150 and starts the jump sequence (the handler requests the exhaustion clip on each tick it runs).
- Perch: a tree-to-tree arrival in tick T with a drawn wait of 3: perch ticks T + 1, T + 2, T + 3; the roll is made in tick T + 3 (the wait reads 0 after its decrement). A drawn wait of 0: the roll is made in tick T + 1.
- Chain whip turn: tick s + 10, cursor 2.5 plus the stored fraction 0.25, over 19 keys: progress 0.1447, turn weight (1 - 0.1447) x 0.5 = 0.428. Tick s has no turn.
- Body slam airborne: the landing leaves the hero 4.0 away in action state 8 on slam tick 1: latch set, count + 1, no damage; the hero lands in tick 10: no strike.
- Head butt reach: the hero 14.5 away and in front when the head butt starts; the lunge puts the boss 7.8 units forward in tick s + 28 (the hero is then 6.7 away and the hit test passes, 6.7 < 7.0, facing dot above 0.5): knockback 2.0, no damage.

### Timeline vector A: the first 600 ticks, hero standing still at locator 9, never attacking, no missiles, phase 1

Ticks count from the first controller tick (tick 0). Example draws: chain whip cooldown 150, rapid-strike cooldown 200, head-butt cooldown 300, exhaustion roll 50 (below 75). Random items and their ranges are marked "R".

- Tick 0: idle, hero 21.41 units away (above 8): approach starts, run clip (action 6). The boss starts facing +z while the hero lies to the south-west, so it curves toward the hero at the movement turn weight; this costs about 2 ticks.
- Ticks 0 to about 61: run, 0.256 units per tick (15.36 per second). The movement chooser switches to orbit at the first tick the horizontal distance is below 6, about tick 62 (Medium, plus or minus 4). The side of the orbit is a coin (R).
- Ticks 62 to 148: orbit (action 13 or 14), sliding sideways at 4.63 units per second. Chain whip is checked every tick; its cooldown is still above 0.
- Tick 149 (R: cooldown value minus 1, so 119 to 239 for the drawn range 120 to 240, when the hero is within 9 by then): the whip cooldown has reached 0 and the hero is closer than 9, so the chain whip starts (s = 149, action 19). Live ticks 177 to 204. The hit of 15 lands in the first live tick in which a chain bone of the posed bone run touches the hero cylinder; with the hero outside the chain's reach no hit lands. The whip ends in tick 222 (s + 73): state idle, whip cooldown redrawn (R: 120 to 240 ticks, so the next whip is possible from tick 222 + the drawn value).
- Tick 223: the first free tick. The rapid-strike cooldown has been 0 since tick 199, and the chooser runs before the movement chooser, so the rapid strike starts at once (s = 223, action 20). Live ticks: 247 to 266 (bit 0x4), 263 to 278 (bit 0x2), 295 to 318 (bit 0x1), 323 to 342 (bit 0x8); at most one hit of 15 in total (the first window that touches the hero). The state ends in tick 383 (s + 160): rapid cooldown redrawn (R: 180 to 360), then a roll from 0 to 100: 50 is below 75, so exhaustion (R: below 75 has probability 75 / 101; otherwise the jump sequence starts at once, one tick earlier).
- Tick 384: exhaustion (phase 1 timer 0): the handler requests action 4 and passes to the jump sequence in the same tick. In phase 2 this state would last 120 to 180 ticks (R) and be hittable.
- Tick 385: jump step 0, action 7; the gate passes in tick 410 (s + 25).
- Tick 410: jump step 1 picks the horizontally nearest of the four trees from the boss's position, computes v and does not move. Ticks 411 onward: flight with action 11, about 50 to 60 moves, then the perch.
- The head-butt cooldown (300) reached 0 in tick 299, during the rapid strike, so it is first checked in the first free tick after the jump sequence.
- Earliest and latest first attacks over the random draws: the first chain whip starts between ticks 119 and 239; the rapid strike can only start in a free tick at or after tick 179 (R: 180 to 360 minus 1); the head butt only at or after tick 239 and then only when its 40 percent roll passes.

### Timeline vector B: hurt side, hero in melee range

Boss orbiting at tick 300, hero 5 units away, cooldowns not yet zero.

- Tick 300: the hero starts a swing (the swing's latch is clear, so one roll): roll 0 to 99 starts the melee dodge (probability 100 / 101); variant 0, 1 or 2 (R, one third each). Variants 0 and 1 request action 16 or 17 and end in tick 361; variant 2 requests action 15 and ends in tick 353. The boss moves 8.98 units sideways (variants 0, 1) or 6.09 units backward (variant 2). A hero hit while the dodge plays still flinches the boss.
- If instead a hero hit lands in tick 400 while the boss is orbiting (the swing's weapon touches it): flinch from tick 400 (action 9), invulnerable through tick 449; in tick 449 the stored damage (the hero's damage, truncated) is subtracted: with 225 health and a damage of 20, health becomes 205 and the boss returns to idle (no exhaustion, since the flinch began outside exhaustion). In tick 450 the choosers run.
- If that damage brings health below 1 percent: tick 449 sets death; the death handler requests action 2 in tick 450, the clip passes its gate in tick 491, and event 2 is sent in tick 491 and in every later tick.
- Rage: a hit that leaves health below 25 percent (below 56.25 of 225, i.e. 56 or less) arms rage in the next free tick: the state is rage for one tick and then idle; the flag is never cleared.

## Open questions

Default if unresolved (rewrite choice): where the chain-bone reach of a posed Gnarfas cannot be computed, test the bone cylinders from the animated skeleton as described in Chain whip and rapid strike, using the previous tick's pose. For the orbit side flip, the touched flag is (Resolved, High) "a wall or solid push, a navigation-mesh border push or a push from another character ran in the previous tick's motion pass"; a wall or border contact with a penetration of 0.05 or less that the boss is not moving into does not count. The walk-step turn weight is (Resolved, High) the case rule of SPEC-0056, Navigation step, stated in Approach in full: 0.3 only when settled in the goal's triangle, 0.1 otherwise. When the movie moves him to locator 12, keep his state, timers and cooldowns. A shot that overlaps him outside the block state always flinches him for 0 damage and costs Roberta nothing; do not run the shot damage rule of SPEC-0036 for it.


1. Resolved: level bit 0 is Gnarfas's event 0 (hero below half health, or 3 or more strikes); bit 1 is set by the arena script of module 985 after the phase-2 movie when bit 0 is set and bit 1 clear; bit 2 is Gnarfas's death event, which runs the defeat sequence (see Arena flow). The level script's level-state calls and the boss events share one level bitset (High). With flag 281 clear, the pack's embedded records identify ordinal 7 as Gnarfas and ordinal 8 as the Werelupe King. The arrival script moves entity 7 and grabs entity 8 for its idle presentation; the King stays held and is deactivated with Gnarfas at the end of the encounter. Confidence: High for record identity and script actions; Medium-High for runtime entity-to-ordinal binding under the documented streaming order (SPEC-0019, Activation in play).
2. Resolved: the knockback velocity is normalize(9 d + 12 up) x 8 s, then hero reaction state 0x30, and it is skipped while the hero is down, action state 16 (see Knockback; High).
3. The stagger's entry condition. Resolved: only a hero melee hit (attacking, attack active, weapon touch) and a hero missile slot overlap start the flinch; no other hit kind is tested (see Taking damage; High).
4. Resolved: approach and orbit travel by clip root motion. Walk 6.34 units/s on average, run 15.36 units/s, orbit 4.63 units/s sideways (mode 5 along +B1 with the mirrored clip, mode 6 along -B1); see Travel. The jump flight speeds are as given in the Jump section.
5. Resolved: the jump-sequence landing rule is a height test. Landing happens when the height above the ground is 1.0 or less (High). The jump-land tunable has no reader (see item 9), so its unit does not matter.
6. Resolved: only the landing of jump step 5 writes the body-slam state; every state write in Gnarfas's code is accounted for (High).
7. Resolved: the distance tunables that code reads (for example the run-or-walk distance 5.0) are world units, compared directly with the horizontal distance between the boss's and the hero's positions, with no scale factor. The other distance tunables have no reader (see item 9) (High for the read ones).
8. Resolved: the chain-whip, rapid-strike and chain-power volumes are bone chains selected by event bit (bit 0x1 bones 29 to 33, 0x2 bones 15 to 19, 0x4 bones 46 to 49, 0x8 bones 38 to 41), each bone a cylinder of radius 0.5 and length 1.25 along its Y axis, against a vertical hero cylinder of radius 1.0 and length 3.0 (see Chain whip and rapid strike; High).
9. Resolved: no code outside the boot program reads the tunables. The only other program code is the 173 level modules; none of them contains an absolute data reference into the boot program's data area (no pair of high-half and low-half immediates forms an address in the data area, which starts right after the code and ends at 0x5EFFFF, so none touches the tunable words at 0x2DAD98 to 0x2DAE8C), and they reach the game only through host calls, none of which returns a boss tunable. So the 16 unused tunables and the block counter's range and chance are stored and never read (High for the modules; the boot program itself has no read, as stated under Unused tunables).
10. Resolved: the wait flag is a word of the game's control state, not a level-script value. It is stored as 0 at start-up and on each world load and never set, so the wait state is never entered (High). Every caller of the control state's accessor uses it only for field loads and zero stores, and the block is never the target of a bulk copy, so no copy can set the flag either.
11. Resolved: the block-counter flag lives in the boss's controller record, starts at 0 and is written only with 0 by the chooser, so the block-counter step is unreachable (High).
12. Resolved: the head-butt hit test is the event bit 0x1 window plus the facing (dot above 0.5) and distance (below 7.0) tests, and the body slam's airborne skip covers the hero states 7 to 10, 0x0B and 0x2B to 0x2E (High). The "hero not down" check of the chain whip and the rapid strike, before their knockback, is the hero-is-down test (action state 16), the same as the hub's hold test (High).
13. Resolved: the missile reaction tests each hero missile's 10-tick path against Gnarfas's bone cylinders for records whose full length is positive. The bone record's +0x40 is full cylinder length and +0x44 is radius; the cylinder is centered halfway along the posed bone's local Y axis from its posed base point (see Missile reaction; SPEC-0029 Behavior 26). The separate missile-hit test uses each slot sphere against the vertical body cylinder (Taking damage; High).
14. **Shared facing helper.** The facing rule above is a shared character-runtime rule. Two shared movement routines call the helper, so it is not specific to Gnarfas; the clean side may place it in the shared character runtime instead (SPEC-0040). Resolved: Gnarfas asks the locomotion mode setter for only four modes: 0 (walk and the orbit fallback toward the hero), 1 (run), 5 (orbit one side) and 6 (orbit the other side). All five call sites are in the approach and orbit routines; no Gnarfas routine requests 2, 3, 4 or 7 to 10 (High for the call sites). The turning weights are the caller's, as the helper rule states: modes 0 and 1 use the weight the movement step passes, and 5 and 6 use the perpendicular rule above. Test vector: the approach with the hero 7 units away requests mode 1; mode 0 occurs only in the orbit fallback (the side point off the walk mesh), never in the approach.
15. **Mode setter and handlers.** The routine that stores the locomotion mode calls a per-mode handler instead of storing the mode only when the monster row's Can Fly value is 1 or 2, the character's ground countdown is below 1 and the requested mode is below 7. Resolved (R-0172, High): Gnarfas's monster row (row 6) has Can Fly 0, so none of his four modes (0, 1, 5, 6) ever takes the handler path; each is stored, and the facing rule covers every mode he uses. Also resolved: the reference direction for the perpendicular modes is the constant up vector (0, 1, 0, 0) at 0x3932E0; modes 5 and 9 face up x m, modes 6 and 10 face m x up (High).
16. The 0.1 turn blend, the 0.9 alignment test and the snap weight 1.0 are the constants of the one shared steering routine used by every actor that moves by locomotion mode (High). The chain whip's own weight, (1 − p) × 0.5, is a separate rule.
17. Resolved: Gnarfas's melee hits use the bone-chain test (whip, rapid strike, chain power) or plain distance tests (body slam below 5.0, head butt below 7.0 with the facing test); it uses no monster weapon volumes. The single-weapon reach test found in its code is the hero's weapon striking Gnarfas (SPEC-0029 Behavior 10.2), not a boss attack. The 2.5 by 7.0 and 2.5 by 0.25 tests belong to the Werelupe King (SPEC-0019) (High).
18. Resolved: the hero side of the bone-chain test is a vertical cylinder standing on the hero's position (High).
19. Resolved: class 14 runs the Gnarfas coroutine from the per-character class arbiter after the NPC update reset/status work and before clip re-time. The loop writes the animation-speed tunable directly to P at its per-tick start, so non-1.0 values affect that tick's clip rate; the shipped value 1.0 leaves playback at normal rate (controller call path; High).
20. Chain-spark ground test. Resolved: 10.0 is added to the tip's height to form the query's ceiling, so the ground found is the highest surface at or below 10 units above the tip (see Sounds and effects; High).
21. Resolved: the shipped fight's animation data is the embedded record 7 of pack 985 with its own tables (main table 6, main clip set 2 of 25 clips). The second copy of the Gnarfas table in region 1264 has the same clips but different sound words and effect-list indices, and is not used by this fight (High). Gnarfas has 225 health, a body cylinder of radius 2.5 and height 7.0, and defence 0.
22. Resolved: the maniacal laugh requests action 0 (clip of 51 keys, 200 ticks); 0x12 is only the state's own number (High).
23. Resolved: approach never plays the walk clip and never reaches its idle exit, because the movement chooser switches to orbit at a distance below 6 before the approach handler runs (High).
24. Resolved: all requested action ids exist in the boss's table (ids 23 and 25 have no clip and are never requested), so the "missing id is skipped" rule has no effect on this boss. The durations in the Clip table and durations section are the real ones (High for the arithmetic).
25. Resolved (Medium-High): a frontal Tor swing is judged by the shared guard/block test on its first active tick before Gnarfas's own character check can read the refreshed attack-active predicate. The enemy think runs before the selected hero update, so Gnarfas sees that predicate on the following tick; his own damage check cannot pre-empt the shared block. While Gnarfas is in guard base state 20, the frontal swing is consumed without damage; a swing from a side or behind is not blocked by the facing test and may then be processed by his own check on the next tick, subject to its reach conditions. Test vector: Gnarfas facing Tor, guard state 20, and Tor's attack-active predicate becomes true during hero update tick T: the shared resolver blocks the frontal hit in T; Gnarfas's own check first sees the refreshed predicate during his think on T+1.
26. Resolved (High): "the hero is busy" in the earlier wording of the hold, the knockback and the death gate is the hero-is-down test (action state 16). The hero's wide busy list and the script query IsHeroOccupied play no part (see Phase hub, step 5).
27. Resolved (Medium-High): a melee hit on him sets the boss health display target for 600 ticks, plays a hit spark and the hero's hit sound; shots do not. These do not affect play.
28. Resolved (High): the chain whip, rapid strike, laugh, exhaustion, jump clips and perch carry no root motion; the head butt, flinch, dodges and body slam have the cumulative distances under Travel.
