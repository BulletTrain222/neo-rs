# SPEC-0013: Boss behavior: Gelert Assassin

- Answers: proactive (open question sweep, leftovers); proactive (open question sweep, boss starts); proactive (open close-out); proactive (verify pass); proactive (none requested); proactive (fight AI build guide); R-0067 (the approach, strafe and back-up clips, their root travel, and the dagger's hand attachment point); R-0068 (the address of every word in the ten-word melee/strafe group and the 23 shadow-slip bucket words, the light arc's two end objects and the path of its balls, and the mirror attack's lateral axis); R-0098; proactive (boss consistency pass 3); proactive (arena pass 4); proactive (Gelert release); proactive (timing pass); proactive (reach pass 3); R-0137; R-0180 (forcing the light arc, healing, mirror images and rainbow dash); R-0171 (first binding and clone wake order, the light-arc hit, the strafe coin); proactive (open question sweep, boss details); proactive (boss win check); proactive (open question sweep, round 9); proactive (reachability check)
- Status: Draft
- Disc build: SCUS-97367 v1.00 (same fingerprint as SPEC-0001)
- Confidence: per section. Tuning values and HP thresholds: High (taken from the boot program's data). Unused rows: High for the rows marked unused, including the invisible bucket, the just-appear-mirror word and the invisibility reappear pair. Ranges and chances in the decision rules: High for the numbers, Medium for their intent. Shadow-slip gate: High for the rules and for the source of its request (the navmesh blocked flag). Reappear and landing values: Medium-High (world units). Elapsed-time counter: High. Melee attacks (thrust lunge, sword swing, jump thrust, back-stab): High for the once-per-attack rule, the block test, the damage values (25, 20, 5, 5), the contact shape and the attack windows in frames; Medium for the exact first tick of each window and for the turn helper. Shadow thrust-lunge: unreachable (Medium-High). Rainbow dash (target points, visiting order, speed, hits per point) and light arc damage: High; light-arc hit shape: Medium. Heal arithmetic (including a negative heal): High. Missile and melee reactions: High for the rolls (the second roll is a fresh roll) and the contact shapes; Medium-High for the bone cylinder placement. Dagger throw and shadow dagger: High for the windup, the release event, the launch values (damage, radius, speed, lifetime) and the launch count. Settle and smoothing (4.14, the shared tracked-point rule defined in SPEC-0040): dormant in the shipped build. No state of this boss reaches it (High for the call graph; Medium overall). The rule's constants and steps: High for the 0.02 threshold (a fixed constant), the 0.15 blend, the settled flag and the second latch's set; Medium for the two-point count and the event 0x13 report. Other counters and the call-site details: Low (see Open questions). Sounds (animation cues, the silent clones, the unused die cue) and the code-started effects (heal aura 428, clone puff 559, flinch flash 222): High; the flinch flash point: Medium; clip effect lists: Medium-High. Fight AI second pass: root travel of every clip in every state, the home-point landing of the mirror attack and rainbow dash, the landing fallback, the melee-reaction reach and latch, the re-arm table, the clone fighters (pattern 1) and the hero-action table: High for the rules; Medium-High for the home point's value and the rainbow dash speed sum; Medium for the release timing (default 150 ticks), the Mote-object and status rows and the clone cursor copy.

## Summary

The Gelert Assassin is one boss that fights the hero with melee, thrown daggers, and "shadow" attacks. Every tick it updates its timers, checks its HP band, reacts to incoming missiles and melee swings, and, when it is free to act, chooses between approaching, circling, backing off, throwing, lunging, or slipping into shadow. Shadow slip makes it vanish and reappear as one of several shadow attacks, chosen by a weighted roll that depends on its HP band. The HP band (called a pattern here) only rises: pattern 1 at 75 % HP or more, then 2, 3 and 4 as HP falls. Each band changes which attacks are likely. Its melee hits, rainbow dash and light arc deal fixed damage that the hero's shield and armour do not reduce; only its thrown daggers go through the projectile rule. At patterns 3 and 4 the boss can heal in the shadows, and at those patterns it can summon up to four mirror images that attack from fixed lateral offsets. The shared tracked-point settle rule (4.14, defined in SPEC-0040) checks whether two tracked points have come to rest and smooths them. It is present in the game but dormant in the shipped build: no state of this boss reaches it. Its sounds come only from its animation table, and its code starts three effects: a heal aura, a puff on a hit clone and a hit flash on a flinch (4.15). All timings run on the 60 Hz game tick (SPEC-0007). Times given in seconds become ticks = trunc(seconds × 60). The clean side can build the boss from this spec and the 97 tuning words described below. The shipped fight runs in the scripted regime: the story has already set the script-control flag before this fight can be reached, so the boss waits until the opening scene releases it and its shadow slips are driven by the slip timer (4.3, 4.4). The boss has 800 HP, and every hero hit on it is multiplied by a damage factor that starts at 1.0 and climbs with the time since the last HP-band change (4.6, 4.13). The "Fight AI build guide" below is the ordered recipe.

### Encounter flow (pack 985; High unless marked)

The active Gelert branch is selected by progress flag 281. On world activation, its callback moves the hero to locator 9 and clears the branch's local stage bits. The streaming filter admits the five Gelert records at ordinals 0 to 4. Under the normal automatic stream path, with a fresh character block and no earlier bind, the first record is ordinal 0: all five records have identical stored positions, so the ascending scan keeps ordinal 0 on an exact distance tie. The destination scan then takes free slot 0, which the boss controller treats as the primary role. This first mapping is conditional on the block being fresh and slot 0 remaining empty; later record-to-slot bindings can differ (High for the static tie and slot rules, Medium-High for the normal encounter path). The opening scene again moves the hero to locator 9, starts boss-fight instance 5 at stage 0, blocks player control, presents the boss through its camera and dialogue sequence, then releases control. The scene's last call sets level state 0, which is what lets the boss out of its wait state (G1, G9). The first eligible range-stream pass after the transition runs before the newly started activate-world callback body is stepped, so it uses the pre-callback hero pose. Later passes see the callback move to locator 9 (High for scheduler order).

The fight's script thread turns the boss's level-state events into four story stages. States 1, 3 and 5 start cutscenes; their completions set states 2, 4 and 6. State 7 starts the final scene, deactivates the primary boss and four clones, stops their effects, ends instance 5 at stage 1, and sets progress states 284 and 301. Afterward, entering trigger 4 runs the glade exit: the hero receives item 30, game states 67, 283 and 302 are set, and movie 44 returns the story to world 74. The boss's attacks and HP patterns remain independent of these camera/dialogue sequences; the level-state events are the bridge between them.

Test path: with flag 281 set, activate the world, place the hero at locator 9, stream records 0 to 4, and run the opening scene. During the fight, process state 1 then 2, state 3 then 4, state 5 then 6, and state 7. The expected result is four stage scenes followed by deactivation of all five Gelert records and instance 5 ending at stage 1. The initial stream-check ordering is not asserted.
## Fight AI build guide

This is the ordered recipe for the shipped fight. It points to the detailed sections for the long rules. Confidence is High unless a line says otherwise. All times are 60 Hz ticks; the animation tick convention below is used for every duration.

### G1. Which regime the shipped fight runs in

- At creation the boss copies one bit of the story flag table (flag 258) into its script-control flag (4.3). The story sets flag 258 at the Brightvale library event (main story step 30), long before this glade fight (step 39), so in the shipped route the flag is 1 (Medium-High: it depends on the story order; nothing in the boot program or the glade module clears it). The scripted branch is therefore the shipped one. Who sets flag 258 and on which visits (Resolved; High for the writers, Medium for the reach of the other case): the only writer in the whole game is the end of the library event in the Brightvale library pack (it plays movie 37, then sets the flag); no code clears it, and none of the nine new-game start indices sets it (start indices 2, 3, 6, 7 and 8 begin Act 3 with it clear). The boss copies the bit again at every creation, that is, at every load of the glade pack in the flag-281 branch, a retry after defeat included. So the scripted branch runs on every visit made after the library event, and the flag-clear branch runs on a visit made before it. Nothing in the glade route tests the library event, so a glade fight done first is not excluded (Medium). Default if unresolved (rewrite choice): copy flag 258 into the script-control flag at each creation and run the branch it selects; do not hard-code the scripted branch.
- Scripted branch consequences: (a) every tick on which level state 0 is clear the boss is forced into the wait state (state 0x21: plays animation 24 once, shows its last frame, does nothing else); the opening scene sets level state 0 as its very last call, after all the dialogue and the closing camera shot, in the same script step that frees the hero and the boss's script control, and that releases the boss (High; G9 gives the timing); (b) the hit counter and script event 0 do not exist; (c) the shadow-slip timer itself starts slips (4.4); (d) the three stance states wait for level states 2, 4 and 6, which the stage cutscenes set.
- The branch with the flag clear (event 0 raised from the hit counter or from the hero's HP ratio, no wait state, slips only from the request latch or the two 50 % rolls) is kept for completeness. It is not what the shipped route plays.

### G2. Spawn and initial values

- Row: monster row 4 (class 1). Hit points 800 (current = maximum), body cylinder radius 1.175 and height 3.5, weight 5.5, ground turn speed 0, defence 0, no footstep shake. Weapon 1 radius 6.0 and length 1.75 (4.4b).
- Position: all five records (boss and four clones) are stored at (-623.68, 121.88, -542.24); the boss is the first record bound to slot 0. The arena ground mesh (mesh 0 of the pack; about 72 by 71 units, flat at height 122.5, containing the spawn point) is assigned to the boss by its own start-up code (the first ground mesh that contains its position) and again by the level script for entities 0 to 4. The boss therefore stands on a navigation mesh for the whole fight (Medium-High). Facing: not set by the controller; the opening scene turns it.
- State 0x21 (wait) is not left until level state 0 is set. Pose at start: animation 24 on its last frame.
- Pattern 1. Shadow-slip timer: whole seconds drawn from 8 to 15 inclusive, times 60 (480 to 900). Attack timer: whole seconds from 2 to 4 inclusive, times 60 (120, 180 or 240). Back-up cooldown 60. Damage factor 1.0 and its counter 0. All timers start counting down at creation, including while the boss waits; the counter and factor do not advance in the wait state. Invisible flag, clone mode, hit latches, request latch: all 0. Clones: state 0x20, clone flag 1 (4.10).
- Tuning words: 97 words (Layout); rolls are integers 0 to 100 inclusive and timer draws are integer seconds drawn uniformly from min to max inclusive.
- Release timing. The timers count down from the tick the boss is created, through the wait state, until the opening scene releases it. Level state 0 is set by the last call of the opening scene (G9), not after its first camera shot. Counting from the scene's first tick S, the release comes at S + 469 + A + D ticks (within about 3): 469 is the fixed waiting of the scene (2 ticks of set-up, camera shot 25 finished 118 ticks after it starts, two 30-tick pauses, a 20-tick and a 1-tick pause, the closing camera shot 29 of about 238 ticks, and a 0.5 s pause of 30 ticks), A is the length in ticks of the boss's animation 2 (the scene waits for it), and D is the total time the six dialogue boxes stay open (each closes only when the player presses Finish). So the timers have run at least 475 ticks at the release, and in practice far more, since the six boxes take seconds to read (High for the order and the fixed part, Medium for the exact offsets). The attack timer (120, 180 or 240) is therefore always 0 at the release. The slip timer (480 to 900) is 0 unless the player dismisses all six boxes within a few ticks, so the boss almost always slips into shadow on its first free tick (the slip gate fires at once with timer 0 in the scripted regime, 4.4). The boss is created on the scene's first tick, or one tick after it (item 43). Default if unresolved (rewrite choice): the boss is created on the scene's first tick; at the release set each timer to its drawn value minus the ticks elapsed since creation, not below 0 (the timers keep counting down through the wait state and stop at 0).
- The scene's boss-fight tweak (instance 5, stage 0) refills both heroes' health and magic before the fight starts.
- Home point. Every character keeps a home point, copied from its placement record when it is placed. For all five Gelert records it is (-623.68, 121.88, -542.24), the centre of the arena mesh. Nothing in the fight rewrites it, and the scene's moves of the boss to locators do not change it (Medium-High). The mirror attack and the rainbow dash land the boss there (G6).

### G3. Tick order and the animation convention

- Tick order is 4.1. State changes made by a chooser take effect in the same tick: the handler of the new state runs in that tick.
- Animation convention: "request tick R" is the tick on which a handler asks for a clip. The clip starts in that tick's animation pass at cursor 0 and moves a quarter key per tick (speed byte 2, play rate 1.0); at rate 2.0 it moves half a key per tick. A handler that waits for "done" reads the clip's progress at the start of each later tick and leaves on the first tick R + D for which progress is at least 0.99 (SPEC-0005, Clip progress). D is given per state below. It follows the common gate rule: at rate 1.0 and speed byte 2, D = 4(F - 1) + 1 for a clip of F keys up to 24 and D = 4(F - 1) for 25 to 74 keys (35 keys: 136); at rate 2.0 the clip advances half a key per tick and D = 2(F - 1) + 1 for F up to 49 (20 keys: 39; 30 keys: 59). The handler sets the next state on tick R + D; the choosers can pick a new state on tick R + D + 1.
- All animation ids the boss ever requests have a clip in its action table (ids 0 to 7, 12 to 45; the table holds 125 entries). Ids 8 to 11 and 46 to 124 have no clip, and the boss never requests them (locomotion slots 3 and 4 hold -1 and are never selected). The skip-without-wait rule for missing ids therefore never applies to this boss, and every duration below is the real clip duration.
- Root travel applies in every state, not only in the locomotion states. The boss moves by the root records of whatever clip is playing, as every non-hero character does (SPEC-0040, Behavior 14D), with the one-tick lag of that behavior: a clip requested on tick R first moves the boss on tick R + 1. The step is in the boss's own frame: z forward along its facing, x to the side (positive x is toward the boss's left when it faces the hero, the same way the strafe-right clip moves). The lunge, sword swing, jump thrust, back-stab, flinches, dodges and the end of the heal and the light arc all carry large root travel; G6b lists the net travel of each clip. The only other movement is the landing write of a shadow slip, the rainbow-dash step (G6) and the clones' placement (4.10).

### G4. State table (state number in hex; action id -> clip, frames; D = ticks from the request tick to the leaving tick)

| State | Action id (clip, frames) | D | Exit and notes |
| --- | --- | --- | --- |
| 0 idle | 0 (clip 3, 25, loops) | none | Requested every tick; choosers replace it. |
| 1 flinch | 16 (clip 10, 19) or 17 (clip 11, 15), coin | 73 or 57 | To 0. Damage is taken on entry (G7). The clip slides the boss back 3.95 (16) or 3.05 (17) units along its facing. |
| 2 evil laugh | 23 (clip 9, 35) | 136 | To 0. |
| 3, 4, 5 stances | none | until level state 2, 4, 6 is set | To 0 and the counter reset (G7). Not hittable. |
| 6 missile dodge | 18 (clip 7, 19) or 19 (clip 8, 20), coin | 73 or 77 | To 0. Not hittable. Slides the boss sideways 10.5 units (18, toward its left) or 10.9 units (19, toward its right) in about 40 ticks. |
| 7 missile deflect | 0 while waiting, then 21 (clip 26, 15) | 57 after 21 starts | If no hero missile slot is active: to 0 at once. Otherwise the last active slot (highest index of the six) is used: if its 3D distance from the boss's position is under 10, 21 starts; if 10 or more, stay in 7 playing idle until the slot leaves or comes within 10 (it can wait indefinitely). While 21 plays, on every tick except the last, the boss sets the shot-deflect flag (SPEC-0029 10.6) and reports itself guarding (base state 20, G13). Not hittable. No root travel. |
| 8 melee dodge | 20 (clip 6, 17) | 65 | Then a roll under 50 starts a shadow slip (not a clone; clone mode and the slip timer are not checked), else 0. Not hittable. Slides the boss back 10.2 units along its facing, 9.5 of it in the first 36 ticks. |
| 9 jump thrust | 29 (clip 19, 25) | 96 | Window ticks 48 to 75; one hit of 5; to 0, no attack-timer reset. No turning. Root travel: back 6.4 units by tick 28, then forward to a net +3.1 (about +2.4 at the end of the window). A blocked jump thrust goes to state E like the others. |
| A dagger throw | 31 (clip 4, 25) | 96 | Three daggers, launched on ticks 24, 48 and 72 after the request; attack timer re-armed at 96. |
| C thrust lunge | 28 (clip 40, 23) | 89 | Window ticks 24 to 63; 25 damage once; attack timer re-armed at 89. Turns to the hero once, on the request tick (weight 1.0), then keeps that heading. Root travel 11.05 forward in all: 0.9 by tick 24, 3.3 by 28, 6.1 by 32, 8.8 by 40, 9.8 by 64. Hittable, no reactions. |
| D sword swing | 26 (clip 39, 20) | 77 | Window ticks 24 to 39; 20 damage once; attack timer re-armed at 77. Turns toward the hero (weight 0.5) on every tick of the state, including during the swing. Root travel 4.2 forward: 2.3 by tick 24, 3.7 by 36. Not hittable. |
| E sword blocked | 27 (clip 38, 12) | 45 | To 0. The attack timer is not re-armed (it is still 0, so the next attack can be chosen on the first idle tick). Slides back 1.4 units. Hittable, no reactions; a hit gives a 50 % slip. |
| F shadow slip | 24 (clip 31, 13), then 25 (clip 32, 20) | vanish 49; chooser pass on the next tick; reappear request on the tick after that; reappear 77 (39 at rate 2.0) | Total from slip start S: attack state begins on S + 128 (S + 90 for back-stab and mirror attack). Not hittable. |
| 10 shadow back-stab | 30 (clip 1, 20) at rate 2.0 | 39 | One hit of 5 on the first tick with horizontal distance under 6.0 (tested again on every tick of the clip until a hit is sent); attack and slip timers re-armed. Turns to the hero (weight 1.0) on every tick after the request. Net root travel +2.1 forward (1.2 of it as a jump about 12 to 14 ticks in). Not hittable, no reactions. |
| 12 shadow dagger | 31 (clip 4, 25) | 96 | Launches on every tick event bit 0x20 is on (frames 6, 12, 18; 4 ticks each; ticks 24 to 27, 48 to 51, 72 to 75); slip timer re-armed (the attack timer is not). No turning at all during the state: it keeps the heading the reappear gave it. Hittable, no reactions. |
| 14 shadow heal | 32 (clip 13, 16), 33 (clip 14, 22 loops), 34 or 35 | 61 for 32; heal loop 6-tick period; 35 (clip 16, 14): 53; 34 (clip 15, 16): 61 | 4.9. Phases 0, 1 and 3 are hittable but a hit gives no damage (it moves the heal to phase 2); no reactions. The slip timer is re-armed at the end, the attack timer is not. A hit in phases 0, 1 or 3 starts clip 34, which slides the boss back 6.05 units. |
| 15 light arc | 36 (clip 21, 8), 37 (clip 20, 11), 38 (clip 22, 20), 39 (clip 23, 18) | 29, 41, loop up to 200 ticks, 69 | 4.8. Hittable in phases 0, 2 and 3 (not in the windup, phase 1); no reactions; the slip timer is re-armed at the end, the attack timer is not. The end clip (39) slides the boss back 4.85 units. |
| 16 mirror attack | 41 (clip 25, 30) at rate 2.0, then 42 (clip 24, 38) | 59, then 148 | Clones wake when 42 passes progress 0.5 (about 73 ticks after it starts); clone mode on at the end. The boss reappears at its home point, not near the hero (G6). Hittable throughout (about 207 ticks from the end of the reappear), no reactions; the attack timer is not re-armed. |
| 17 rainbow dash | 44 (clip 28, 12 loops) for eight points, then 45 (clip 30, 16) | 61 for 45 | 4.8. Starts from the home point (G6). Each tick moves 0.667 by its own step plus about 0.33 of clip root travel, about 1.0 per tick in all. Hittable in every phase, no reactions. The slip timer is re-armed at the end, the attack timer is not. The end clip carries the boss 6.75 units further forward. |
| 18 just appear | 22 (clip 41, 34) only when a roll above 74 | 132 | Otherwise to 0 on the first tick; slip timer re-armed at exit. Hittable, no reactions. |
| 19 start invisible | 40 (clip 18, 30) | 116 | Never entered in play: it is reached only from the end of a lunge, swing or back-stab while the invisible flag is already set, and only this state sets the flag (4.11). |
| 1A dead | none | none | G10. |
| 1B idle strafe | 0 | none | 4.4. |
| 1C strafe right | locomotion mode 5, entry 14 (clip 35, 11 loops) | none | Sideways root travel 4.72 per 44 ticks. |
| 1D strafe left | mode 6, entry 15 (clip 36, 11 loops) | none | Same, other side. |
| 1E back-up | 13 (clip 33, 10 loops) | 39 | Moves 0.2797 per tick backward; to 1B. |
| 1F approach | mode 0, entry 12 (clip 34, 15 loops) | none | 5.15 units per 60 ticks on average. |
| 20 clone parked, 21 wait | 42 (clone, held); 24 (boss, held) | none | 4.10 and G1. |

States 0xB (back-stab), 0x11 (shadow thrust-lunge), 0x13 (invisible attack) and 0x19 (start invisible) are not reachable in the shipped fight: no chooser selects the first three, and 0x19 is entered only while the invisible flag is set, which only 0x19 itself sets. Clip frame counts are keys; each key is 4 ticks at rate 1.0.

### G5. Decision order each tick (decision-ready states only: 0, 1B, 1C, 1D, 1E, 1F)

1. Timers decrement, then the script gate and the HP check (4.1 steps 1 to 3). The HP check and the pattern change run only in decision-ready states.
2. Reactions (4.13) in the reaction states; the melee reaction can start the melee dodge or jump thrust, the missile reaction the dodge or deflect.
3. Attack chooser, only when the attack timer is 0 (4.4): dagger throw first (20 < d < 40, not invisible, not a clone, roll under 30, a fresh roll every tick the conditions hold, so about 29.7 % per tick); otherwise if d < 15: d at most 10 rolls (60 or more lunge, else swing) and 10 < d < 15 always lunges.
4. Strafe chooser and the slip gate (4.4): the slip starts first when the slip timer is 0, the boss is not invisible, not in clone mode and not a clone (the request latch or the two 50 % rolls can also start it). Otherwise the distance bands: d > 20 approach (even from back-up); 15 <= d <= 20 keep back-up, right or left, else a coin; d < 15 keep back-up; if the back-up cooldown is 0 set it to 60 and back up on a roll under 100 (else idle strafe); otherwise plain idle (state 0) stays idle, while idle strafe or approach takes the coin.
   The coin: 0 is strafe right (state 1C, mode 5, clip 35), 1 is strafe left (state 1D, mode 6, clip 36) (High: the draw is the shared generator's output taken as an unsigned 32-bit number modulo 2, so it is the lowest bit of the tempered output, and nothing biases it). "Right" and "left" name the side of the move target (3 units to the boss's right for mode 5, to its left for mode 6, measured from the hero-to-boss line). The clip's own root travel is what moves the boss: clip 35 has a positive sideways root, which the shared root rule applies along up x facing, toward the boss's left when it faces the hero; clip 36 is the mirror and moves it toward its right (4.4; SPEC-0040, Behavior 14C). So coin 0 moves the boss toward its left (the hero's right when they face each other) and coin 1 toward its right; if a build wants the original's on-screen direction, keep the clip assignment above rather than the word "right". Nothing in the original favours a side: the two handlers differ only in the mode and in the sign of the 3-unit target offset, the draw happens only on entering a strafe (not while one runs), and the clips differ by 0.04 units per cycle (4.72 against 4.68). The only side-dependent effect is the mesh test, which examines the target point on the side opposite to the move, so an edge behind the boss's path is what triggers the fall-back to the approach clip for a tick.
   Forcing the shadow attacks that need a hurt boss (High; R-0180). Pattern 1 always picks just appear, so a boss kept at 75 percent or more never shows the light arc, healing, mirror images or rainbow dash. The slip is reached only when the attack chooser did not take the tick (the attack timer above 0, or the hero 15 to 20 away), because the slip gate needs a decision-ready state. With the slip timer 0, the scripted flag set, no clone awake, and slip start tick S: the first shadow-attack draw r falls about 50 ticks later; the attack state begins on S + 128 (S + 90 for the mirror attack). Draw ranges (0 to 100): pattern 2 light arc 66 to 98; pattern 3 light arc or healing 50 to 74, mirror 75 to 84; pattern 4 light arc or healing 8 to 39, mirror 40 to 49, rainbow dash 50 to 99. In patterns 3 and 4 a second draw is made: the light arc if health is at least 300 of 800 (pattern 3) or 100 of 800 (pattern 4), or the second draw is 50 or more; otherwise healing. Vectors: pattern 2, health 560, slip timer 0, attack timer 90, hero 18.0 away, r 70: light arc, state 15 (hex) from S + 128. Pattern 3, health 250, r 60, second draw 20: healing, state 14 (hex) from S + 128 (HP 250, 258, 266, then it stops at the 266.67 target). Pattern 3, health 350, r 80: mirror attack, state 16 (hex) from S + 90. Pattern 4, health 150, r 70: rainbow dash, state 17 (hex) from S + 128.
5. Hit test and hit reaction (4.13), then the current state's handler.
6. Same-tick rules. A later step sees the state an earlier step just wrote, and a later write replaces an earlier one. So: (a) a reaction that sets dodge, deflect, melee dodge or jump thrust makes the boss not hittable before that tick's hit test, so the swing or shot that caused the reaction cannot hurt it on that tick; (b) the HP-band step can set a stance and the reaction step still runs on the same tick (the stances allow reactions), so a missile or melee reaction on that tick replaces the stance state (the pattern, the event and the counter reset have already happened; the stance wait is skipped, and the cutscene still runs because it is driven by the event); (c) the evil-laugh roll runs before the HP check on the same tick and can be replaced by it; (d) within the reaction step the missile reaction runs first, and the melee reaction, if it fires on the same tick, replaces its result; (e) the missile reaction visits the hero's six shot slots in order and each qualifying slot writes the state, so the last one wins, and a dodge written by an earlier slot can be replaced by a deflect from a later one.

### G6. Movement

- Boss speed comes only from the root travel of the clip (4.4 Locomotion clips). The row's turn speed is 0 and is not used; steering uses the shared navigation step (SPEC-0029 Behavior 2, SPEC-0056): mode 0 steers along the route toward the hero position; modes 5 and 6 steer toward the side direction with the 0.1 blend (1.0 snap when already within about 26 degrees).
- Strafe target: a point 3 units to the side of the boss. If that point is not on the boss's mesh, the strafe tick uses mode 0 toward the hero's position instead (the approach clip plays that tick). With the hero standing still at 20 units the strafe slowly widens the distance (about 0.0003 per tick), so the boss alternates between strafe and approach around d = 20.
- Back-up has no steering: it plays clip 33 and the root moves the boss backward along its facing; it does not stop at a wall except through mesh confinement.
- Landing points after a slip are written directly (4.7). Landing distances from the hero (random direction, up to 10 tries on any mesh): just appear 15 to 40, light arc 25 to 40, healing 25 to 35, shadow dagger 15 to 15, back-stab 3 units behind the hero (single try). Mirror attack and rainbow dash do no search: the boss lands exactly on its home point (G2), wherever the hero is (High for the rule, Medium-High for the home point's value). If the 10 tries of a ranged landing all fail, the chosen attack becomes just appear at the boss's own current position (not the hero's). Each try draws x and z uniformly in [-1, 1), normalises the horizontal vector (so directions favour the diagonals) and draws the distance uniformly as a real number in [min, max); the dagger landing, whose min and max are both 15, always lands at exactly 15. The landing keeps the hero's height. The back-stab point is the hero's position minus 3.0 times the hero's facing vector, tested once; if it is on no mesh, the shadow dagger is used with its own 15-unit search.
- Unreachable hero: the boss has no path search; it moves toward the hero through the route step of the shared navigation, confined to the mesh. The edge push sets the blocked flag, which makes the request latch start a slip for the next 120 ticks (4.4).

### G6b. Root travel of the clips the boss plays (High)

Net travel along the boss's facing (z, positive forward) and to its side (x, positive toward its left) over the whole clip, from the clip's root records (each key is 4 ticks at rate 1.0; the travel of a key is spread over its 4 ticks; add the one-tick lag of G3). The locomotion clips are in 4.4.

| Action (state) | z | x | Shape |
| --- | --- | --- | --- |
| 16 flinch (1) | -3.95 | 0 | 1.9 in the first key, most of it in 32 ticks |
| 17 flinch (1) | -3.05 | 0 (swings +1.2 and back) | steady over 40 ticks |
| 18 missile dodge (6) | 0 | +10.46 | 10.6 by tick 40 |
| 19 missile dodge (6) | 0 | -10.94 | 10.5 by tick 40 |
| 20 melee dodge (8) | -10.22 | -0.28 | 9.5 by tick 36, 10.1 by tick 40 |
| 26 sword swing (D) | +4.23 | 0 | 2.3 by tick 24, 3.7 by 36 |
| 27 sword blocked (E) | -1.41 | 0 | 1.6 by tick 16, then drifts forward to -1.4 |
| 28 thrust lunge (C) | +11.05 | 0 | 0.9 by tick 24, 6.1 by 32, 8.8 by 40, 9.8 by 64 |
| 29 jump thrust (9) | +3.06 | 0 | -6.4 by tick 28, back to -0.5 by 48, +2.4 by 76, +3.1 at the end |
| 30 back-stab (10, rate 2.0) | +2.10 | 0 | 1.1 jump in key 6 (ticks 12 to 14), then 0.9 more |
| 34 heal interrupted | -6.05 | +0.8 and back | 6.0 by tick 40 |
| 39 light arc end | -4.85 | 0 | 3.9 by tick 36, 4.9 by 52 |
| 44 rainbow dash loop | +0.33 per tick | 0 | steady, on top of the dash step |
| 45 dash end | +6.75 | 0 | peaks at 7.5 by tick 28, ends at 6.75 |
| 0, 22, 23, 24, 25, 31 to 33, 35 to 38, 40 to 43 | under 0.5 | under 0.5 | none that matters |

The travel is not blocked by the hero or other characters; only the arena mesh confines it (SPEC-0056), and an edge push raises the blocked flag (4.4). The lunge therefore carries the boss through the hero's position when it starts within 11 units; the sword swing and the lunge keep their start heading (apart from the swing's half-weight turning) while moving.

### G7. Hurt side and damage the boss takes

- Hittable states and the hit test: 4.13. A hit is a hero melee swing with the weapon touching the boss, or a hero missile slot overlapping the body cylinder (radius 1.175, height 3.5). Melee and missiles are not both counted in one tick: a melee contact takes the tick; otherwise each overlapping missile slot is removed and counted, but only the last overlapping slot's damage is applied.
- Damage to the boss = truncate(hero hit damage x damage factor). The hero hit damage is the hero's melee damage at scale 1.0, or the hero damage query for that missile slot (SPEC-0036). The damage factor is 1 + 4 x counter / 7200 as a float (not truncated). The counter rises 2 per tick while it is below 7200 and 1 per tick after that, is 0 at the start, and is set to 0 again on every HP-band change and when a stance ends; it does not advance in the wait state. So the factor is 1 + t / 900 for t ticks since the last reset up to t = 3600 (2.0 at 15 s, 3.0 at 30 s, 5.0 at 60 s), and after that rises by 1 every 1800 ticks (6.0 at 90 s). It is unbounded.
- Damage is applied only when the hit makes the boss flinch. A hit in sword-blocked that rolls under 50 (slip), a hit during any healing phase (moves healing to phase 2), and a hit on a clone take no HP. A hit during the light arc removes the arc and flinches with damage.
- There is no armour or defence, no block chance and no invulnerability timer other than the non-hittable states; dodge, deflect and melee dodge happen only through the reactions of 4.13. Hit stun is the flinch duration (73 or 57 ticks); during it the boss cannot be hit again and chooses no attack or movement, but the reaction step still runs in the flinch (4.5), so a new hero swing (melee reaction) or a new hero shot (missile reaction) can replace the flinch at once with a dodge, deflect, melee dodge or jump thrust.
- Boss HP 800: with a factor of 1.0 and a hero hit of 15 the boss takes 15; the same hit at the 60 s mark takes 75.
- A melee hit needs all three of: the hero's attack-active word is set (event bit 0x1 of its current frame), the hero's second attack query holds, and the hero's weapon reaches one of the boss's bone capsules (SPEC-0036, Hero swing, step 2). If any fails, the missile slots are tested instead on that tick. There is no per-swing hit latch on the boss's side: after a hit the flinch makes it not hittable for 57 or 73 ticks, which outlasts any swing.
- Hero shots pass through the boss, and stay alive, whenever it is in a state that cannot be hit; a slot is removed only on an overlap while the boss is hittable (including the overlaps that give no damage: a clone, healing phases, the sword-blocked slip).
- The damage subtraction clamps at 0 (damage equal to or above HP leaves HP at 0) and runs the shared death hook, which changes nothing lasting for this boss because the loop restores base state 1 while HP is 0 (G10).
- Where the boss can actually be hurt. The melee reaction (4.13) fires on the first tick a hero swing is active, with the hero within 12 units and facing it, and sends the boss into the melee dodge with chance 100/101. It runs in every reaction-capable state (idle, strafing, backing up, the dagger throw, the flinch, the stances), so a swing started inside 12 units almost never connects while the boss is in those states; the dodge slides it back 10 units and makes it not hittable for 65 ticks. The boss is hittable but cannot react in: the thrust lunge, sword blocked, evil laugh, shadow dagger, mirror attack (about 207 ticks), just appear (and its 132-tick laugh), healing (phases 0, 1, 3, damage-free), light arc (phases 0, 2, 3), and rainbow dash (all phases). It is never hittable in the sword swing, the jump thrust, the shadow slip, the back-stab, the stances, the dodges, the deflect, the flinch and the wait.

### G8. Phase transitions

- Patterns by HP fraction (4.2): below 0.75, below 0.50, below 0.25, strict; events 1, 3, 5. At 800 HP the boundaries are 599, 399 and 199. The change is made only on a decision-ready tick, enters the stance (state 3, 4 or 5) and resets the counter, which returns the damage factor to 1.0.
- Timers and chances switch to the new pattern's values on the next draw; running timers keep their value.
- The stances wait for level state 2 (after event 1), 4 (after event 3) and 6 (after event 5). The stage cutscenes set them when they finish.

### G9. Arena hand-offs

- Opening: the scene holds the hero, plays the boss presentation, and ends with a run of calls in one script step with no pause between them: the normal camera returns and the hero is freed, the boss's script control ends, and level state 0 is set (the last call of the scene). The boss leaves the wait state on its next update (the script step runs after the world tick), so it is idle one tick after the scene's last step. Order and waits of the scene, from its first tick S: letterbox bars up at once, hero to locator 9, 1-tick pause, Illusen (entity 6) loops animation 3, boss-fight tweak (instance 5, stage 0), fade in 1.0 s (no wait), player control blocked (1 tick), camera shot 25 starts (about S + 2); a 30-tick pause; boss animation 25 starts (no wait); then the scene polls until camera shot 25 has finished (118 ticks after it started); 30-tick pause; camera shot 26; 30-tick pause; boss loops animation 1; conversation with the boss begins; line 0 (waits for the player); camera shot 27; line 1; camera shot 26; 20-tick pause; boss loops animation 3; lines 2 and 3; boss animation 4 (no wait); 1-tick pause; camera shot 30; boss animation 2 (the scene waits for it to finish); camera shot 33; line 4; camera shot 28; line 5; conversation ends; camera shot 29 (the scene waits for it, about 238 ticks); weapon-ready state on; 30-tick pause; bars down over 1.0 s (no wait); normal camera, hero freed; boss script control released; level state 0 set (High for the call order; the exact tick counts are Medium). Level state 0 is never cleared afterwards in this branch, so the scene runs once per load of the pack.
- Events 1, 3 and 5 start the stage cutscenes; each moves the boss to a locator and the hero to another and ends by setting the matching state. Event 7 (death) starts the finale (4.2).
- The level thread reassigns mesh 0 to entities 0 to 4 while the fight runs.

### G10. Death

- HP reaches 0 only through a flinching hit; the loop writes base state 1 on every tick on which the boss's HP is 0 (the first step of each tick, 4.1), so the dead base state of the shared death hook never takes effect and there is no death animation and no corpse. The HP check runs on the next decision-ready tick, so the boss first finishes the flinch (57 or 73 ticks) and any other non-decision state, then enters state 1A and raises event 7. The finale scene deactivates the boss and the clones.
- The boss cannot die in a state that takes no damage (healing, sword-blocked slip, stance).

### G11. What a port gets wrong easily

- Timers are whole seconds times 60, not fractional seconds.
- The dagger roll is per tick (about 29.7 %), not per choice; it needs d strictly between 20 and 40.
- Distances are horizontal.
- The slip is driven by the timer in the shipped regime; the pre-release timers have already counted down.
- Damage taken is multiplied by the time factor; do not clamp it.
- Boss melee ignores armour; daggers use the projectile rule.
- The boss stays on mesh 0; the strafe falls back to mode 0 when its side point is off the mesh.
- Durations: use the D column; do not replay clip 0 for any id.
- Movement is not only locomotion: every clip's root travel applies (G6b). A lunge that starts at 5 units carries the boss 6 units past the hero.
- The mirror attack and the rainbow dash start from the arena's home point, not from where the boss stands or where the hero is (G6).
- Clones are real fighters (G14), not scenery.

### G12. Edge cases for implementers

1. Timers run from creation, through the wait. At the release both are already partly spent (G2). The damage counter does not run in the wait.
2. Re-arming is uneven. Lunge, sword swing, dagger throw and back-stab re-arm the attack timer. Jump thrust, sword blocked, shadow dagger, light arc, healing, mirror attack, rainbow dash and just appear do not. Back-stab, shadow dagger, light arc, healing, rainbow dash and just appear re-arm the slip timer; the mirror attack re-arms it only when clone mode ends (item 13); the plain attacks do not. So after most shadow attacks the attack timer is already 0 and the boss attacks on its first idle tick if the range fits, and after a blocked attack it can attack again as soon as the 45-tick blocked state ends.
3. The request latch is raised again every tick the arena-edge flag lasts (up to 120 ticks after a push) and is cleared when a reappear starts and when it ends. A slip is started by the latch even in clone mode or with the slip timer running.
4. The melee-dodge exit rolls the 50 % slip with no check of clone mode, the slip timer or invisibility; only the boss itself (not a clone) does it.
5. Melee reaction numbers: hero within 12.0 units horizontally (1.5 times the hero's 8.0 reach float) and facing the boss within 60 degrees (dot above 0.5). The reaction runs once per hero swing, on the first tick the hero's attack-active word is set while the boss is in a reaction-capable state; the once-latch is set on that evaluation even when the distance or angle test fails, so a swing that fails the tests is never re-tested. The latch clears on any tick the hero is not attacking and on any tick the boss is in a state without reactions, so a swing that began while the boss was in one is tested when the boss leaves it, mid-swing.
6. Missile reaction: the path is 180 ticks long; each missile's hit id is looked up in a ring of the last 10 ids and, when new, is added whether or not either roll succeeds, so a missile gets one pair of rolls (dodge chance, then deflect chance with a fresh roll) however long it flies. A boss in a state without reactions does not look at missiles, so they are rolled later. Both rolls are 0 to 100 inclusive: dodge 40 of 101 for the boss, 25 of 101 for a clone; deflect 80 of 101.
7. The deflect state waits, playing idle, while the last active shot slot is 10 or more units away (3D distance from the boss's position); it ends at once when no slot is active.
8. A stance entered by a band change can be replaced on the same tick by a reaction (G5 item 6). The flinch can be replaced by a reaction on any later tick.
9. The back-stab check (horizontal distance under 6.0) starts two ticks after the reappear ends and repeats each tick of the 39-tick clip; a hero who stays within 6 units at any of those ticks is hit once. The reappear itself takes 39 ticks during which the boss stands behind the hero and turns to face it (weight 1.0 every tick).
10. Heal: the target is the maximum divided by the current pattern, so a pattern-4 heal tops out at 25 % and is only chosen below 12.5 %; a pattern-3 heal tops out at 33.3 % and is chosen below 37.5 % with a roll under 50, so from 33.3 to 37.5 % it lowers HP to the target. A heal gains 1 % of maximum every 6 ticks (8 HP at 800).
11. The light arc's balls are fixed to the launch direction; the arc's 20 damage hits once and ends the arc. A hit on the boss during the arc's loop removes the arc balls and flinches the boss with damage.
12. The wake of the clones repeats every tick during the second half of clip 42, but only parked clones respond; a clone hit earlier in the same attack is parked again and can be woken on a later tick of the same window.
13. Clone mode is turned on at the end of the mirror attack even if every clone was already hit; it turns off (and the slip timer is redrawn) on the next tick that finds no awake clone. With clone mode on the timer cannot start a slip; the latch and the melee-dodge roll still can.
14. The boss targets the currently selected hero (the active hero changes with hero switching), with no check of the hero's health or state; a dead or unreachable hero changes nothing in the boss's rules.
15. All range tests use horizontal distance, except the deflect test (3D) and the missile paths and shapes (3D).
16. The boss's facing is only changed by: the navigation steering of approach and the strafes, the turn calls of the lunge (once), the sword swing (every tick), the dagger throw (every tick after the request tick), the back-stab, the slip's reappear and the rainbow dash (every tick). Idle (state 0 and idle strafe) does not turn the boss, so it keeps the heading of the last action; this decides which way a dodge slide or a flinch knock-back goes.
17. Invisibility, the normal back-stab and the shadow thrust-lunge never occur (G4 note).

### G13. Hero actions against this boss

The boss is invulnerable to every generic hit (its invulnerable word is set when it is created and nothing clears it), so only the two paths below can take HP from it; its own hit step (4.13) is the only damage path. (High for the structure; Medium-High for the Mote-object rows, which follow from the hittable query.)

| Hero action | What it does to the boss |
| --- | --- |
| Sword swing and combo hits (either hero) | Damage trunc(hero hit x damage factor), only when the boss is in a hittable state and the swing's weapon reaches a bone capsule on a tick the attack-active word is set (G7). The first active tick of every swing within 12 units and 60 degrees triggers the melee reaction: dodge with chance 100/101; in the 1 case in 101 that it does not, a jump thrust with chance 15/101 (10/101 at patterns 3 and 4). A hit starts the flinch (57 or 73 ticks, knock-back 3 to 4 units), resets nothing else, and keeps the damage factor running. |
| Power attack (Tor's power step, Roberta's full release) | The weapon contact of Tor's power step counts as a swing. The Mote objects it spawns (Nova, Wind, Beam, Bubble, Sun and Shadow beams, and the like) only act on targets that pass the hittable query, which this boss never does: no damage, no status. |
| Block (hero guard) | Against the lunge, sword swing and jump thrust: if the weapon touches, the hero is guarding and dot(hero facing, boss facing) is below -0.5, the attack is blocked: no damage to the hero, the boss goes to sword blocked (45 ticks, hittable, 50 % slip on a hit, attack timer not re-armed). The back-stab, rainbow dash and light arc cannot be blocked. A shield hero raises the guard by itself when idle and the boss's attack-active is within 10 units in front (guard timer 40). Daggers follow the projectile rule (SPEC-0029 Behavior 9). |
| Dodge, roll, parry | None exists for either hero; there is no invulnerability after a hit. The hero avoids the boss's attacks only by moving. The boss's melee hits have no knock-back. |
| Roberta's quick shot | Each active shot slot is a sphere tested against the boss's body cylinder, only while the boss is hittable; a hit removes the shot and deals trunc(shot damage x damage factor) with the flinch (the last overlapping slot's damage if several). Before that, the missile reaction sees the shot's 180-tick path: dodge 40 of 101, then deflect 80 of 101 (a boss 'nothing' result is about 13 %), and the shot passes through a boss that is not hittable. While the boss plays the deflect clip it reports guarding and has its deflect flag set, so Roberta's quick shots that reach it are sent back at her (SPEC-0029 10.6). A shot that hurts the boss through its own hit step pays its quick-shot magic and Mote charge at the touch, as the shared shot-damage routine does for every own-hit-step boss that asks it (the boss is not hittable for the generic walks, but its own step asks); a shot that is deflected or passes through pays nothing, and a full-release shot pays nothing at a hit. Full-release shots are never blocked or deflected by that rule, but the boss's reactions and hit test treat every active shot slot alike. |
| Spells, Motes and charms | No effect (see the power attack row). |
| Items and statuses | Healing and recovery items act on the hero only. No status the heroes can apply works on this boss (Medium-High). |
| Swing against the deflect clip | While the deflect clip plays the boss counts as guarding for the hero's swing test (SPEC-0036, Hero swing, step 4): a swing that touches it with dot(boss facing, hero facing) below the row's Blocking Angle (0.0, so the two roughly face each other) is blocked with a spark; otherwise nothing happens. No damage either way, because the boss is not hittable then and its invulnerable word stops the generic path (Medium). A blocked swing counts as landed (Tor pays his swing magic) and Tor recoils into attack sub-state 13 when his sub-state is 2, 3, 4, 6, 11 or 12; Roberta never recoils; a swing that is not blocked has no effect at all (not landed, no hit-list entry). |
| Hitting a clone | A swing or shot that overlaps an awake clone in a hittable state removes the clone from play (parked, hidden, puff 559) with no damage and no flinch; the hit consumes the shot. See G14. |

### G14. Clone fighters

- Each clone is a full copy of the controller with the clone flag set. At the wake tick (4.10) it is placed 5 or 10 units to either side of the boss (measured from the boss's position that tick, which is the home point), copies the boss's facing, starts clip 42 at the boss's current clip cursor (so it appears part-way through the pose and finishes about when the boss's clip does), and its state becomes 0x16. At the same moment the clone draws its attack timer (pattern-1 range, 120, 180 or 240 ticks), its slip timer, sets its back-up cooldown to 60, and clears its invisible, evil-laugh and melee-latch values.
- A clone's pattern is always 1: its HP is never lowered (a hit parks it), so it never changes band. Its attack timer therefore always draws 2 to 4 s whatever the boss's band, and its reaction chances are the mirror values (dodge 25, melee dodge 15, jump thrust 15).
- When clip 42 ends the clone goes to state 0 and fights with the same choosers as the boss: approach, strafe, back up, lunge (25 damage) and sword swing (20) at the boss's ranges, jump thrust, dodges and deflects. It never throws daggers (the dagger test needs a non-clone), never slips (the dodge's slip roll, the request latch and the timer all exclude clones), never reaches the heal or the light arc, and has its own copy of the hit test, so it can also be put in the sword-blocked state and blocked by the hero's guard.
- A hit on an awake clone in any hittable state parks it at once (state 0x20, base state 42, puff 559); the missile or swing that hit it takes no damage effect on the boss. A parked clone can be woken again by the next mirror attack.
- The finale deactivates the clones together with the boss (SPEC-0041, stage 4).

### G15. Timing table (timing pass)

Confidence: High (from G4 and G5). Units: ticks of 1/60 s. The attack timer, the slip timer and the back-up cooldown are whole seconds times 60, lowered by one per controller tick from creation (also during the scripted wait). The D column of G4 is the number of ticks from the request tick to the leaving tick; every requested id has its own clip (speed byte 2, 4 ticks per key; rate 2.0 halves it).

| Attack | Window (ticks after the request) | Leaves | Timer re-armed |
| --- | --- | --- | --- |
| Thrust lunge (C) | 24 to 63, 25 damage once | 89 | attack timer |
| Sword swing (D) | 24 to 39, 20 damage once | 77 | attack timer |
| Jump thrust (9) | 48 to 75, 5 once | 96 | none |
| Dagger throw (A) | daggers launched on 24, 48 and 72 | 96 | attack timer |
| Shadow back-stab (10, rate 2.0) | first tick with horizontal distance under 6.0 | 39 | attack and slip timers |
| Shadow dagger (12) | launches on 24 to 27, 48 to 51, 72 to 75 (every tick of the event) | 96 | slip timer |
| Shadow slip (F) | vanish 49; attack state begins on S + 128 (S + 90 for back-stab and mirror attack) | | |
| Flinch (1) | | 73 or 57 | |
| Melee dodge (8), missile dodge (6) | | 65; 73 or 77 | |

Hit tick: one hit per attack, on the first window tick on which the weapon reaches the hero; the lunge carries the boss 11.05 units forward, 0.9 of it by tick 24 and 6.1 by tick 32, so a hero standing 5 to 8 units away is usually first touched well after tick 24. The dagger roll is about 29.7 percent per tick while 20 < d < 40 and the attack timer is 0.

Worked exchange (Tor with a weapon idle 8 units away, attack timer 0): on the first decision tick T the boss lunges (d at most 10, roll 60 or more) or swings; a lunge touches Tor on its first touching tick h (24 at the earliest), Tor's flinch starts on h and ends on h + 37; the lunge leaves on T + 89 and re-arms the attack timer; the boss can then strafe, back up or slip until the timer runs out. Tor's counter swing on a boss in state C is accepted (hittable, no reaction); in the dodges, the swing and the slip it is not.

What a port most likely gets wrong (timing): counting the timers in fractional seconds; re-arming the attack timer after every attack (only four attacks do); treating the dagger chance as per decision instead of per tick.

## Arena build guide

The arena is Illusen's Glade (world pack 985). The same pack and floor host two fights, chosen by progress flag 281: this fight (flag set, Act 3, boss-fight instance 5) and Gnarfas (flag clear, Act 1, instance 1; SPEC-0020 refers to this section for the shared floor). Coordinates are world x, z with height y where it matters; distances are horizontal. The pack's own data holds every point below, so the clean side reads positions from the pack and uses this section to check them. Confidence is High unless marked.

**1. Floor, walls, size.**
- The hero's floor is one flat height patch at height 121.436. Its outline is a closed ring of nine wall panels, 6.3 high, with corners (-627.74, -496.47), (-609.43, -505.25), (-587.92, -523.23), (-579.91, -552.36), (-602.32, -584.94), (-631.10, -582.96), (-654.06, -574.06), (-668.67, -549.80) and (-660.81, -515.68). It is a rounded blob about 89 wide (x -668.7 to -579.9) and 88 deep (z -584.9 to -496.5), area 5,579, centred near (-624.7, -543.1); corner distances from that centre run 40 to 48. Outside it lie tall cliff shells (type 3 panels, 120 high, whose tops are far below the floor or far outside) that no hero can reach.
- Two short side patches overlap the outline. The north-east passage is a diamond with corners (-608.2, -510.0), (-596.1, -494.7), (-579.6, -509.6) and (-591.4, -523.9), at height 119.72. The south-west passage is an oval about 27 by 27 around (-645.5, -586) at height 119.67, leading to a round platform (x -691 to -627, z -655 to -589, height 121.55). Both passages are sealed during the fights by gated solid walls (item 3). Whether the outline panels also stop the hero at the two passage mouths has no effect on play (the sealing walls block both passages during the fights). Default if unresolved (rewrite choice): let the sealing walls alone stop the hero at the two passage mouths.
- There is no deadly ground, no hurting surface, no pit, no water, no damage volume, no breakable and no pickup. The pack has no save point and no placed object.
- Navigation. Two flat ground meshes (38 triangles each, height 122.53, 1.09 above the floor patch): mesh 0, x -660.5 to -588.1 and z -578.1 to -507.3, area 3,831; mesh 1, x -663.0 to -586.0 and z -579.1 to -503.7, area 4,143. Mesh 0 lies wholly inside mesh 1 (mesh 1 adds a 312-unit rim). The Gelert is on mesh 0; Gnarfas is on mesh 1. The hero's floor reaches 5 to 12 units beyond mesh 0 all round; a hero standing in that outer ring is farther than the boss can route to. A slip landing is accepted on either mesh (4.7), so a landing can fall on the rim that belongs to mesh 1 only; the boss then stands off its assigned mesh until the script's mesh helper or its own steering pulls it back (Medium).

**2. Start positions.**
- Arrival from the Glade village pack is a replacement load with plane index -1, so both heroes are first placed at the pack's entry 0, (-703.10, 12.14, -813.36), far from the arena. The activation callback then moves the selected hero to locator 9, (-637.03, 122.67, -567.24), facing (0.546, 0, 0.838). The partner is not moved; it stays parked at entry 0 and is swapped in at the dead hero's position if needed (swap mode 2).
- The boss and its four clones are placed at (-623.68, 121.88, -542.24), facing (-0.487, 0, -0.874): 28.3 from locator 9 and 1.3 from the floor's centre. The boss is not moved by the opening scene (the scene has no boss move), so at release the hero is 28.3 away on a line within 6 degrees of his facing.
- The opening scene refills both heroes, holds control, plays camera track 25 (2.0 s) and a dialogue with tracks 26 to 33, then returns the normal camera and releases the boss; the hero is released at the scene's end. Illusen's record is looped in animation 3 during the scene and stands at (-582.57, 121.48, -493.09) in the north-east passage, beyond the sealing wall, for the whole fight.
- Stage scenes (bands 0.75, 0.5, 0.25) move the boss to locator 27 (-624.26, 122.03, -545.80), facing (-0.624, 0, -0.782), and the hero to locator 28 (-634.77, 122.03, -561.14), facing (0.488, 0, 0.873): 18.6 apart, facing each other. The boss's home point is unchanged by these moves.

**3. Hazards, objects and the sealing walls.**
- The only objects are two gated solid walls and their glow effects. Wall NE (gate 1): from (-590.91, -527.83) to (-612.3, -510.1), 17.1 high, 5.4 inside the outline, sealing the north-east diamond. Wall SW (gate 2): from (-656.19, -568.09) to (-622.97, -583.40), 23.2 high, sealing the south-west oval. The module keeps a two-bit mask word and passes it to the world on each change: the arrival callback sets both bits (value 3, both walls on); the finale clears bit 1 (value 2), so the north-east wall opens and the south-west wall stays. A visit that begins with the mask word untouched has no wall.
- Twelve placed glow effects of one kind run from load: records 1 to 6 along the south-west mouth, records 7 to 12 along the north-east mouth. The finale stops 7 to 12 only, matching the wall that opens. They deal no damage.
- Eight ring points for the rainbow dash are the locators tagged 200 to 207: (-612.60, -529.52), (-604.58, -542.57), (-608.14, -557.45), (-621.18, -565.47), (-636.07, -561.92), (-644.08, -548.87), (-640.53, -533.99) and (-627.49, -525.97). They lie 16.7 to 23.4 from the home point, 17 to 26 inside the outline, and all are inside mesh 0. The dash covers about 1.0 unit per tick, so one hop takes 15 to 20 ticks and a full dash of eight points about 130 to 170 ticks plus the first leg from home.
- Tree locators (tags 101 to 104) belong to Gnarfas and are not read by this boss.
- Trigger 4 is a slab 67 long, 7 thick and 31 tall, standing 5.8 beyond the north-east wall (near face through (-623.97, -492.79), running along (0.77, 0, -0.64)). Only the hero's body contact fires it, and it is reached only after the finale opens the wall.

**4. Script timeline** (SPEC-0041, pack 985 section, has the call list).
1. Load: Init registers the activation filter, the world-activation callback, trigger 4 and the control thread of this branch. Activation: hero to locator 9, normal camera (distance 3), both wall bits on, step word cleared, arrival bit set.
2. Stage 0 (level state 0 clear): the opening scene above. The boss is held in its wait state until level state 0 is set (G1). The scene's last call sets level state 0 (G9), after the dialogue, the closing camera shot 29 and the release of the hero and of the boss's script control; nothing else sets it in this fight.
3. Stages 1, 3, 5 (boss events at below 75 %, 50 %, 25 % HP): fade out, letterbox, boss to locator 27, hero to locator 28, fade in 0.7 s, camera 31, a boss animation (4, 5 or 6, after animation 25), dialogue line 8, 9 or 10 to 12, camera 29, release, then level state 2, 4 or 6. The boss waits in its stance until that state.
4. Stage 4 (event 7, HP 0): the same entry, animation 7, line 13, animation 24, fade out, normal camera, flags 284 and 301 set, wall NE off, entities 0 to 4 deactivated, effects 7 to 12 stopped, instance 5 stage 1, letterbox off, fade in, level state 8.
5. After the finale, while flag 284 is set, the mesh helper idles. Entering trigger 4 sets bit B1; on the next tick the thread sets B2 and B3; on the following tick it gives item 30 (no notice), sets flags 67, 283 and 302, sets the movie return world to pack 956's world and plays movie 44, which returns to the village.
6. What holds the boss: only the wait state before level state 0, the stances (waiting for states 2, 4, 6) and the scene's control of entity 0.

**5. Camera and music.**
- During play the camera is the ordinary follow camera on the floor's profile 0; the module makes no camera-mode call. Fixed tracks 25 to 33 play only in the scenes.
- Music: the pack's music row has selector 3 (passes while flag 64 is set and 283 clear). The alternate music (bank resource 98, stream resource 218) plays when progress bit 216 is also set, which is true throughout this fight, so this fight plays the alternate loop; after flag 283 is set at the exit the plain loop (bank 31, stream 163) returns. Neither reacts to combat level (SPEC-0009). One positioned ambient loop (radius 45, centre (-624.26, 122.03, -545.80)) covers the whole floor. The module makes no music or sound call.

**6. Death and retry.** The pack has no death callback and no game-over call. Flags 220 and 248 are set in Act 3, so the hero swap mode is 2: when the selected hero dies and the partner lives, the partner takes over at the dead hero's spot after the death clip (about 63 ticks) and the fight goes on with nothing reset (boss HP, pattern, clones, timers unchanged). When both are down the game-over page follows (91 more timer additions); Continue re-applies the last save or load of the session. The pack has no save point, so the snapshot comes from before the fight; every module starts fresh: boss HP 800, level states, step words and mask word reset, and the opening scene replays with the refill of both heroes. Flags 281 and 66 persist from the snapshot; 284 and 301 are set only at the finale and are not saved mid-fight. Leaving and re-entering mid-fight also resets the boss.

**7. Victory.** The finale sets flags 284 and 301; the exit step gives item 30 and sets 67, 283 and 302, then movie 44 returns to the village; the pack is not re-entered afterwards (the village's entry trigger needs flag 283 clear). The north-east passage is open after the finale.

**8. What a port most likely gets wrong in this arena.**
1. Treating the floor as a circle or square: it is the nine-corner outline above; the boss's route mesh (mesh 0) covers only its inner 69 percent.
2. Forgetting that the two sealing walls exist: the hero cannot reach Illusen or the village exit until the finale; the south-west wall never opens.
3. Placing the boss at locator 27 or the arena centre at the start: it starts at home (1.3 from centre) and the hero 28.3 away at locator 9.
4. Using locator 11 or 12 (Gnarfas's points) for this fight: this fight uses 9, 27, 28 and the ring 200 to 207.
5. Playing the plain music: the alternate loop plays here.
6. Leaving the boss in its wait state: level state 0 must be set by the last call of the opening scene (after the dialogue and camera shot 29), not when camera 25 finishes.

**9. End-to-end test vector.** Flags 281, 66, 220, 248, 64 and 216 set; 283 clear. The village loads the pack. The hero is at locator 9 facing (0.546, 0, 0.838); the boss and clones are at (-623.68, 121.88, -542.24) (28.3 away, mesh 0 triangle containing the home point), walls NE and SW on, effects 1 to 12 running, alternate music. The opening scene runs at least 469 ticks plus the boss's animation 2 and the time the six dialogue boxes stay open (for example, 60 ticks for the animation and 180 ticks per box: 469 + 60 + 1080 = 1609 ticks after the scene's first tick); level state 0 is set by the scene's last call and the boss is released with the attack timer at 0 and the slip timer at 0 as well (a draw of 480 to 900 ticks has run out), so it slips into shadow on its first free tick (G2, G9). At 599 HP the stance and the stage scene run: boss to (-624.26, -545.80), hero to (-634.77, -561.14), then control returns. The same at 399 and 199. At 0 HP the finale runs: flags 284 and 301 set, the north-east wall and effects 7 to 12 off, the five Gelert records deactivated, the hero back in control. The hero walks north-east from his spot to the wall line near (-600, -519), through the diamond and across the slab near (-598, -512) (about 45 units from the centre); the next tick gives item 30, sets 67, 283 and 302 and plays movie 44, which ends in the village.

## Reach and distances (boss reach pass)

How the figures are made (Medium): the boss's skeleton is posed from each attack clip at the cursor of the previous animation pass. The weapon volume on attachment frame 0 is tested against Tor's idle bone capsules with the exact asymmetric weapon-versus-bone rule in SPEC-0018. The rows below use Tor level with the boss, facing it, at horizontal distance d along the boss's facing at the request tick s. "From the start" includes the clip's root travel; boss turning is omitted, so Tor stays on the facing line. The method assumes the recorded attack clips, size rows, attachment frames, and a stationary idle hero; moving targets and other hero poses can change contact.

| Attack (row 4: weapon 1 radius 6.0, length 1.75) | Window | Level hero touched, from the start | First touching tick |
| --- | --- | --- | --- |
| Sword swing (26; travels 4.2) | s + 24 to s + 39 | 0 to 11.0 | s + 24 at 4.5 to 10.5; s + 25 at 4.0; s + 31 at 11.0; 0 to 3.5 at s + 32 or s + 33 |
| Thrust lunge (28; travels 11.0) | s + 24 to s + 63 | 0 to 13.0; none at 13.5 | s + 24 within 2.0; s + 27 at 4.0; s + 29 at 7.0; s + 33 at 10.0; s + 37 at 12.0; s + 45 at 13.0 |
| Jump thrust (29) | s + 48 to s + 75 | 0 to 5.5; none at 6.0 | s + 48 at 0 to 2.5; s + 49 at 3.0 to 3.5; s + 50 at 4.0; s + 51 at 4.5; s + 52 at 5.0; s + 53 at 5.5 |
| Clone swing (row 5: radius 5.5, length 4.0) | s + 24 to s + 39 | 0 to 11.0 | s + 24 for 1.5 to 10.0; s + 32 closer |
| Clone lunge (row 5) | s + 24 to s + 63 | 0 to 14.5 | s + 24 within 3.0; s + 31 at 10.0, s + 43 at 14.0, s + 51 at 14.5 |

- Vertical contact: the rows above are exact-query samples for a level hero only. The earlier feet-height bands came from the old round-cylinder approximation and are withdrawn; the exact pass does not establish a feet-height range.
- Distance-only hits (no shape): back-stab, horizontal distance below 6.0 on any tick of animation 30 (the boss lands 3.0 behind the hero); rainbow dash, below 8.0, once per target point.
- Projectiles. Dagger: enemy-pool kind 0 (straight, not homing), sphere radius 1.25 against the hero's body cylinder, 1.5 units per tick, lifetime trunc((D + 10) / 1.5) ticks for the hero distance D at the throw, so its range is about D + 10; aimed once at release. Shadow dagger: the same shape, 5.0 units per tick (5.0 is also its lifetime divisor), 3 damage, thrown from 15 units away. Light arc: two balls, spheres of radius 2.0 at 1.5 above the boss's feet, 45 degrees either side of the hero direction fixed at launch, 1/3 unit per tick for up to 200 ticks (66.7 units each); they spread apart (each is 0.2357 units per tick of flight off the launch line), so a hero standing on the launch line is clear of both once that offset exceeds 2.0 plus the hero's body radius, after about 14 ticks of flight, that is for a hero more than about 3.3 units from the launch point along the line.
- Body: radius 1.175, height 3.5 (row 4).
- Distances the AI uses (horizontal unless marked): dagger throw 20 < d < 40 (30 in 101 per tick); d at most 10: lunge on a roll of 60 or more, else swing; 10 < d < 15: lunge; d < 15: back up (cooldown 60); 15 to 20: strafe left or right (target 3 units to the side); above 20: approach. Melee reaction: hero within 12.0 and facing the boss within 60 degrees. Missile deflect: the last active shot within 10.0 (3D). Turning: the swing turns at weight 0.5 every tick, the lunge snaps once on its request tick, the back-stab turns at 1.0 every tick, the jump thrust and the shadow dagger do not turn. Landings after a slip: just appear 15 to 40, light arc 25 to 40, healing 25 to 35, shadow dagger exactly 15, back-stab 3 behind; mirror attack and rainbow dash at the home point.
- Safe spots (Medium): beyond about 13.4 from where a lunge starts (the chooser lunges at 10 < d < 15, so a hero standing 13.5 to 15 away is never hit by it); beyond about 11 for the swing; a step of more than about 2.3 units (dagger radius plus the hero's body radius) off the line of a dagger at release; between the two light-arc balls once they have spread.
- Worked example: attack timer 0, hero standing still 12.0 in front of the boss, so the lunge is chosen on tick s. The boss slides forward (0.9 by s + 24, 6.1 by s + 32) and the weapon touches the hero on s + 37: 25 damage, unless the hero guards facing it (block, sword-blocked state). At 14.0 the same lunge passes in front of the hero and ends on s + 89 without a hit.

## Player summary

- Only sword swings (either hero's combo hits, and the sword contact of Tor's power step) and Roberta's quick shots hurt it. Motes, Mote attacks, beams, spells and statuses do nothing.
- A hit counts only while it is in a hittable state, not while it is dodging, hidden or deflecting. It dodges nearly every swing that starts within 12 units in front of it (100 of 101), so most swings at close range miss; a swing or shot that lands while it is hittable flinches it for about one second.
- While its deflect pose plays, swings that face it are blocked and quick shots are sent back at Roberta; hold your shots until the pose ends.
- Its clones vanish when hit once (no damage), so hit the clones instead of waiting.
- Guard facing it stops the lunge, the sword swing and the jump thrust. The back-stab, the rainbow dash and the light arc cannot be blocked: move out of their reach instead.

## Where it lives on the disc

- All boss tuning values are 97 consecutive signed 32-bit integers in the boot program's data, at virtual addresses 0x2DABD8 through 0x2DAD58, 4-byte stride. Value i (i = 0..96) is at 0x2DABD8 + 4·i. The table below gives each group's address range. Values are loaded at runtime by virtual address (SPEC-0001 gives the fingerprint and the address-to-file mapping).
- A few more constants come from data (section "Other constants" below).
- The settle test and the tracked-point smoothing (4.14) use two float constants (0.02 and 0.15) and one event id (0x13). These are fixed constants, not tuning words, so they are not in the tables below.
- Distances are in world units, speeds in world units per second, times in seconds unless marked "ticks". "Unused" means no reader is known in the behavior described here. The clean side should store such values but need not act on them.
- Animation numbers are indices into the boss's main animation table, given in decimal (SPEC-0005).
- The boss's target is always the hero (the player).

## Layout

### Tuning values, grouped

The boss's tuning words are 97 consecutive signed 32-bit integers, stored in address order. The table gives each group's address range and use. Values are stated in the behavior sections where they are used (4.3 to 4.13). A pattern-indexed pair is two words, min first: the pattern p pair starts at the group address + 8 × (p − 1). A single value per pattern starts at the group address + 4 × (p − 1). A "clone" value is used when the acting entity is a mirror image.

| Group | Address range | Words | Use |
| --- | --- | --- | --- |
| Shadow-slip timer, min/max per pattern | 0x2DABD8–0x2DABF7 | 8 | Used (4.11). Seconds for patterns 1 to 4: 8 to 15, 4 to 8, 3 to 6, 2 to 5. |
| Healing timer, min/max per pattern | 0x2DABF8–0x2DAC17 | 8 | Loaded once at creation into a countdown that no boss logic reads; treat as stored only (High). |
| Attack timer, min/max per pattern | 0x2DAC18–0x2DAC37 | 8 | Used (4.4, 4.11). Seconds for patterns 1 to 4: 2 to 4, 2 to 3, 1 to 3, 1 to 2. |
| Evil-laugh chance | 0x2DAC38 | 1 | Used (4.3). |
| Missile dodge chance (four patterns and a mirror-image value) and missile deflect chance (four patterns) | 0x2DAC3C–0x2DAC5F | 9 | Used (4.13). |
| Dodge chance, melee (patterns 1 to 4, then the mirror-image value used by clones; stored 100, 100, 100, 100 and 15) | 0x2DAC60–0x2DAC73 | 5 | Used (4.13): the getter takes the pattern 1 to 4 and a clone flag, and a nonzero clone flag selects the fifth word (High). |
| Jump-thrust chance, melee (four patterns and a mirror-image value) | 0x2DAC74–0x2DAC87 | 5 | Used (4.13). |
| Dagger throw distances and chance, melee range, lunge chance, strafe distances and chances | 0x2DAC88–0x2DACAF | 10 | Used (4.4), except the strafe left/right range and taunt chance, which are stored but unused. Word order in "Word order of the two groups" below. |
| Shadow-slip buckets: backstab, dagger, invisible, light arc or heal, mirror images, rainbow dash, just-appear mirror | 0x2DACB0–0x2DAD0B | 23 | Used (4.8), except the words listed as unused in "Word order of the two groups" below (the pattern 1 words, the clone backstab word, the three invisible words, the pattern 2 mirror word, the pattern 3 rainbow word and the just-appear-mirror word; High; Open question 13). |
| Backstab range | 0x2DAD0C | 1 | Used (4.4b). |
| Thrust-lunge range | 0x2DAD10 | 1 | Unused. |
| Dagger reappear distances, min and max | 0x2DAD14–0x2DAD1B | 2 | Minimum used (Edge cases); maximum unused. |
| Invisibility reappear distances, min and max | 0x2DAD1C–0x2DAD23 | 2 | Unused: neither word has any reader (High; Open question 12). |
| Healing factor and healing landing range | 0x2DAD24–0x2DAD2F | 3 | Factor unused; landing range used (4.8, 4.9). |
| Healing tick period | 0x2DAD30 | 1 | Used (4.9). |
| Light-arc and just-appear reappear distances, min and max | 0x2DAD34–0x2DAD43 | 4 | Used (4.7, 4.8). |
| Healing length | 0x2DAD44 | 1 | Unused. |
| Light-ball speed, light-ball damage, light-arc damage | 0x2DAD48–0x2DAD53 | 3 | Speed and light-arc damage used (4.8, 4.12); light-ball damage unused. |
| Rainbow-dash speed and radius | 0x2DAD54–0x2DAD5B | 2 | Used (4.8). |

The invisibility reappear pair's second word (stored value 30) is its maximum. Neither word has any reader, so both are unused (High; Open question 12). The clean side should store both and not apply them.

### Word order of the two groups

All 33 words below are signed 32-bit integers. The distances are converted to floating point before they are compared with the horizontal distance d of 4.4; the chances and buckets are compared with an integer roll r in 0 to 100 inclusive. Confidence: High for every address, value and role.

**Ten melee and strafe words (0x2DAC88 to 0x2DACAF)**

| Address | Value | Role |
| --- | --- | --- |
| 0x2DAC88 | 40 | Dagger throw far limit: a throw needs d < 40. |
| 0x2DAC8C | 20 | Dagger throw near limit: a throw needs d > 20. |
| 0x2DAC90 | 30 | Dagger throw chance: a throw needs r < 30. |
| 0x2DAC94 | 15 | Melee range: a thrust lunge or sword swing is chosen only when d < 15. |
| 0x2DAC98 | 60 | Lunge threshold, used only for d ≤ 10 (the distance 10 is a fixed constant, not a word): r ≥ 60 gives the thrust lunge, r < 60 the sword swing. |
| 0x2DAC9C | 20 | Strafe outer range: d > 20 approaches; d ≤ 20 enters the strafe and back-up rules. |
| 0x2DACA0 | 20 | Strafe left/right range. Unused. |
| 0x2DACA4 | 10 | Taunt chance. Unused. |
| 0x2DACA8 | 15 | Back-up distance: the back-up rules apply when d < 15. |
| 0x2DACAC | 100 | Back-up chance: a back-up starts when r < 100. |

The dagger roll is made only when 20 < d < 40 and the boss is neither invisible nor a mirror image; the melee roll is made only when d ≤ 10. The back-up roll is made only on the tick that sets the back-up cooldown (4.4).

**Twenty-three shadow-slip bucket words (0x2DACB0 to 0x2DAD0B)**

| Address | Value | Bucket and pattern | Use |
| --- | --- | --- | --- |
| 0x2DACB0 | 0 | backstab, pattern 1 | Unused: pattern 1 always just-appears. |
| 0x2DACB4 | 33 | backstab, pattern 2 | Used. |
| 0x2DACB8 | 25 | backstab, pattern 3 | Used. |
| 0x2DACBC | 4 | backstab, pattern 4 | Used. |
| 0x2DACC0 | 25 | backstab, mirror image | Unused: the chooser uses the pattern value for a clone too. |
| 0x2DACC4 | 0 | dagger, pattern 1 | Unused (pattern 1). |
| 0x2DACC8 | 33 | dagger, pattern 2 | Used. |
| 0x2DACCC | 25 | dagger, pattern 3 | Used. |
| 0x2DACD0 | 4 | dagger, pattern 4 | Used. |
| 0x2DACD4 | 25 | dagger, mirror image | Used in place of the pattern value when the acting entity is a mirror image. |
| 0x2DACD8 | 10 | invisible, pattern 1 | Unused. |
| 0x2DACDC | 10 | invisible, pattern 2 | Unused. |
| 0x2DACE0 | 5 | invisible, pattern 3 | Unused. |
| 0x2DACE4 | 30 | light arc or heal, pattern 1 | Unused (pattern 1). |
| 0x2DACE8 | 33 | light arc or heal, pattern 2 | Used (always the light arc at pattern 2). |
| 0x2DACEC | 25 | light arc or heal, pattern 3 | Used. |
| 0x2DACF0 | 32 | light arc or heal, pattern 4 | Used. |
| 0x2DACF4 | 10 | mirror images, pattern 2 | Unused: the pattern 2 chooser has no mirror bucket. |
| 0x2DACF8 | 10 | mirror images, pattern 3 | Used. |
| 0x2DACFC | 10 | mirror images, pattern 4 | Used. |
| 0x2DAD00 | 20 | rainbow dash, pattern 3 | Unused: the pattern 3 chooser has no rainbow bucket. |
| 0x2DAD04 | 50 | rainbow dash, pattern 4 | Used. |
| 0x2DAD08 | 50 | just-appear mirror image | Unused. |

There are no mirror-image words for the light-arc-or-heal, mirror and rainbow buckets, and no pattern 1 mirror or rainbow word. A pattern lookup for a value outside 1 to 4 returns 0.

### Other constants read from data

| Constant | Address | Type | Value | Meaning |
| --- | --- | --- | --- | --- |
| Script-event hit-count limit | 0x2DAD5C | int | 2 | Used in 4.3 |
| Hero HP-ratio threshold | 0x2DAD60 | float | 0.55 | Used in 4.3 |
| Boss damage-taken factor, initial value | 0x2DAD64 | float | 1.0 | Starting value of the factor that multiplies each hero hit on the boss (4.6, 4.13) |
| Thrust-lunge damage | 0x2DAD80 | int | 25 | HP taken from the hero by a thrust-lunge hit (High) |
| Sword-swing damage | 0x2DAD84 | int | 20 | HP taken from the hero by a sword-swing hit (High) |
| Light-arc loop length | 0x2DAD88 | int | 200 | Ticks (High) |
| Light-arc height offset | 0x2DAD8C | float | 1.5 | Units (Medium) |
| Rainbow-dash damage | 0x2DAD90 | int | 15 | HP taken from the hero by a rainbow-dash hit (High) |
| Dagger speed | 0x2DAD70 | float | 1.5 | Dagger speed in units per tick; also the divisor of the dagger's lifetime (4.4b) (High) |
| Dagger damage | 0x2DAD78 | int | 10 | The dagger's damage (10.0) given to the projectile; also the 10 added to the distance in the dagger's lifetime (High) |

Jump thrust, back-stab and shadow thrust-lunge deal 5 damage, a fixed constant, not a tuning word (High). Light-arc damage is the tuning word at 0x2DAD50 (20) and the rainbow-dash radius the word at 0x2DAD58 (8).

The projectile setup constants are fixed constants, not taken from the table above: 1.25 is the radius of both dagger projectiles (the light arc's setup does not use 1.25: its only height constant is the 1.5 above the boss's feet, and its balls are tested with the 2.0 radius given under Projectiles); 3.0 is the shadow dagger's damage; 5.0 is the shadow dagger's speed in units per tick and the divisor of its lifetime (High).

The settle test and smoothing constants 0.02, 0.15 and event id 0x13 (4.14) are also fixed constants, not table entries.

## Behavior

### 4.1 Tick order

Every tick (1/60 s), the boss runs these steps in order:

1. Reset the boss's animation speed to 1.0. Then, if the boss's HP is 0, write base state 1 (this undoes the dead base state that the shared death hook may have set, G10). Then, if the navmesh blocked flag is set, raise the shadow-slip request latch (4.4).
2. Count down each of four timers by one tick (stopping at 0): the shadow-slip timer, the attack timer, the back-up cooldown, and one further counter (the healing countdown copy; Open question 3).
3. Script gate (4.3). Then the HP-band check (4.2), which can change the band and can start the death state.
4. Missile reactions and melee reactions (4.13), if the boss is in a state that allows reactions (4.5).
5. Attack choice (4.4), only if the attack timer is 0 and the boss is free to act.
6. Strafe choice and the shadow-slip gate (4.4), only if the boss is free to act.
7. Hit detection against the boss's six hit slots, and reactions to hits (4.13).
8. Run the handler for the boss's current state (4.4b, 4.7 to 4.12).
9. Advance the elapsed-time counter (4.6). Then, on every tick whatever the state, store the boss's character id as the hero's forced lock-on target (the same write the Black Knight and the Cloud Beast make; SPEC-0040 Behavior 20), step the boss's fade level (visual) and the debug text (High, from the main loop).

The boss is "free to act" (decision-ready) when it is idle or strafing (4.4). In every other state it is busy and does not choose a new action.

No state of this boss calls the settle test or the smoothing, so they are not part of this order (4.14, dormant).

### 4.2 HP bands (patterns)

Let f = current HP / maximum HP of the boss.

| Pattern | HP condition | Change of state | Script event raised | Remarks |
| --- | --- | --- | --- | --- |
| 1 | f ≥ 0.75 (starting value) | none | none | |
| 2 | 0.50 ≤ f < 0.75 | superior-sneer stance (4.3) | 1 | Only from pattern 1 |
| 3 | 0.25 ≤ f < 0.50 | angry stance (4.3) | 3 | Only from pattern 1 or 2 |
| 4 | 0 < f < 0.25 | injured stance (4.3) | 5 | Only from pattern 1, 2 or 3 |
| dead | f ≤ 0 | death state (no action) | 7 | No further checks |

- The pattern only rises; it never returns to a lower value, even after healing (4.9).
- A band can be skipped: if f falls from 0.8 to 0.2 at once, the boss goes straight to pattern 4 and raises only event 5.
- The band check runs only when the boss is decision-ready (4.1). A boss that drops into a lower band during a swing enters the new stance after its current action ends.
- Each band change resets the elapsed-time counter (4.6) to 0, which also returns the boss damage factor to 1.0.
- Death: the check at HP 0 runs only on a decision-ready tick. The killing hit is taken in a flinch (4.13), and the loop writes base state 1 on every tick on which the boss's HP is 0, so the dead base state of the shared death hook never takes effect and the boss plays no death animation; it finishes the flinch, then enters the death state and raises event 7.

Confidence: High for thresholds and events. The starting pattern is 1 (4.11).

### 4.3 Script control

- The boss has a script-control flag and a hit counter. The script-control flag is set once, when the boss is created, from one bit of the game's flag table (flag number 258): it is 1 when that flag is set and 0 otherwise. The boss never changes it afterwards. The hit counter rises on each hit while the script-control flag is clear (4.13).
- When the flag is 0, the hit-count event rule below applies and the scripted wait never starts. When the flag is 1, the scripted wait rule applies and the hit-count event rule does not. In the shipped story order the flag is 1: flag 258 is set at the Brightvale library event, which comes before the glade fight. The "flag clear" rules in this spec describe the other branch only.
- While the script-control flag is clear (not the shipped regime), the boss raises script event 0 whenever the hit counter is above 2, or the hero's HP ratio is below 0.55. The check runs every tick, and the event is not latched (Edge cases).
- While the script-control flag is set (scripted), the boss is forced into the wait state whenever script bit 0 is clear, and the boss does nothing else. The wait state plays animation 24 once and holds its last frame. When bit 0 is set, a boss in the wait state returns to idle. Script bit n is level state n of the arena script (bit 0 of the first byte of the world's 64-bit level-state array, the same array SetLevelState and QueryLevelState use; clear means wait); the opening scene sets level state 0 as its last call (G9), so the boss leaves the wait state on its next update after the scene ends. In the flag-clear regime the same bit is set by the boss's own event 0 (the rule above); the shipped regime has no other writer, because no other routine of the glade pack or of the fight sets level state 0. The check runs every tick, so the boss is also held if bit 0 were ever clear again.
- The three stances wait for a script bit and do not act until it is set. No animation is played while waiting.
  - Superior-sneer stance (pattern 2): waits for bit 2.
  - Angry stance (pattern 3): waits for bit 4.
  - Injured stance (pattern 4): waits for bit 6.
  When the bit is set, the boss returns to idle and resets the elapsed-time counter (4.6).
- Evil-laugh roll: a pending flag is checked in the HP step. The dagger throw sets it on any tick on which its windup animation (31) is playing and not yet finished and the hero's current animation is 32 (High). Animation 32 is one of the hero's two hit-reaction animations (SPEC-0055; Medium-High), so the boss laughs only when the hero is reeling from a hit during the windup. If the flag is set, it is cleared, and a roll of 0–100 that is below 75 (EvilLaughChance) starts the evil-laugh state (animation 23). The laugh ends when its animation ends, and the boss is idle.

Confidence: High for the structure and for the flag's source (a copy at creation); Medium for the meaning of the script bits. The two branches are High; the identity of flag 258 as a story flag is Medium-High.

### 4.4 Idle decisions: attack and strafe

Let d = the horizontal distance from the boss to the hero, in units (x and z only; height ignored). Every range test in this section reads this distance. Let r = a fresh roll, an integer 0–100 inclusive. Let T_a = the attack timer and T_b = the back-up cooldown, both in ticks.

**Attack choice** (only when decision-ready and T_a = 0):

1. Dagger throw if all of: 20 < d < 40, the boss is not invisible, the boss is not a mirror image, and r < 30.
2. Otherwise, if d < 15:
   - if d ≤ 10: r ≥ 60 → thrust lunge; r < 60 → sword swing.
   - if 10 < d < 15: thrust lunge.
3. Otherwise no attack is chosen this tick.

When a dagger throw, back-stab, thrust lunge or sword swing ends, T_a is set to 60 times a whole number of seconds drawn uniformly from min to max inclusive for the current pattern's attack-timer pair (the tuning table in Layout): pattern 1 gives 120, 180 or 240 ticks. A timer set to N on tick t reads 0 in the choosers of tick t + N. The jump thrust does not re-arm T_a.

**Strafe and approach** (decision-ready states; re-checked every tick):

| Distance d | Result |
| --- | --- |
| d > 20 | Approach the hero (approach state; locomotion mode 0, speed from the clip root travel, see Locomotion clips). |
| 15 ≤ d ≤ 20 | If backing up, or already strafing left or right, keep the state. Otherwise pick left or right with a 50 % coin each. |
| d < 15 | If backing up, keep it. If T_b = 0: set T_b = 60 (1 s), then back up with chance 100/101 (any roll below 100). Otherwise, if T_b > 0: the idle strafe state (0x1B) and the approach state (0x1F) pick left or right with a 50 % coin (coin 0 is right); plain idle (state 0) and every other state keep the current state. If T_b = 0 and the roll is 100, the state becomes idle strafe (0x1B). |

Strafing targets a point 3 units from the boss, perpendicular to the line from the boss to the hero: strafe right goes to one side and strafe left to the other. The 3 units are the offset of the move target, not a speed. The approach has no speed value of its own: every strafe and approach state moves only by the root travel of its clip.

**Locomotion clips** (High for the values and clips, Medium-High for the travel). The boss's creation routine fills its seven locomotion slots with the shared slot setter (SPEC-0029, Behavior 2) using the values (12, 12, 12, -1, -1, 15, 14). In the setter's order these are the slots of modes 0, 1, 2, 3 and 4, then mode 6, then mode 5. Each value is an entry number of the boss's main animation table (SPEC-0005), and the entry names a clip of its model clip bank (42 clips for fight region 755, counted from 0). All the clips below have speed byte 2, so at play rate 1.0 a clip advances a quarter of a key per tick (4 ticks per key). Travel per tick is the root value x 0.000977 / 4.

| State | How it plays | Table entry | Clip | Keys (ticks) | Root value per key | Travel |
| --- | --- | --- | --- | --- | --- | --- |
| Approach | mode 0 (modes 1 and 2 are the same) | 12 | 34 | 15 (60), loops | forward: 185, 275, 325, 373, 398, 398, 409, 396, 329, 254, 356, 386, 392, 399, 399; sideways sway of at most 43 | 5.15 units per cycle, 5.15 units/s on average |
| Strafe right | mode 5 | 14 | 35 | 11 (44), loops | sideways: 328, 521, 714, 584, 410, 206, 426, 464, 449, 364, 364 | 4.72 units per cycle, 6.43 units/s on average |
| Strafe left | mode 6 | 15 | 36 | 11 (44), loops | sideways: -364, -449, -464, -426, -206, -410, -584, -714, -521, -328, -328 | 4.68 units per cycle the other way, 6.39 units/s on average |
| Back-up | played directly as entry 13, not through a mode | 13 | 33 | 10 (40), loops | forward -1145 at every key | -0.2797 per tick, 16.78 units/s backward, 11.19 units per cycle |
| Idle strafe | played directly as entry 0 | 0 | 3 | 25 (100), loops | none | stands still |

The strafe right and strafe left clips are separate clips; the right one is the mode 5 clip and the left one the mode 6 clip. The sign of the sideways value is relative to the boss's own frame; the steering side for modes 5 and 6 is given in SPEC-0029, Behavior 2.

- A back-up lasts until its animation ends, then the boss is idle.
- The back-up cooldown is set whenever the close-range check runs with T_b = 0, even when the roll then does not back up.

**Shadow-slip gate** (4.7): the boss starts a shadow slip only if it is not a mirror image and one of these holds:
- The shadow-slip request latch is set. (The latch is set in every tick in which the boss's navmesh blocked flag is set (SPEC-0056), and is cleared when a shadow slip's reappear begins and again when it ends; because the blocked flag lasts 120 ticks, the latch is raised again on the next tick if the flag is still on. The timer, invisibility and clone mode are not checked for this path, but the gate runs only when the boss is decision-ready.)
- Or: the shadow-slip countdown is 0, the scripted-mode flag is 1, the boss is not invisible, and it is not in clone-shadow mode (4.10).
- Or: it has just finished a dodge-melee reaction and a roll below 50 passes (4.13).
- Or: it was hit while in the sword-blocked state and a roll below 50 passes (4.13).

Confidence: High for the latch, its source and the timer path. The blocked flag is set by the navmesh confinement step (SPEC-0056) and by nothing else: on a tick in which a navmesh boundary edge pushed the boss back, or the boss was restored onto the mesh, and it stays set for the next 120 ticks (2 seconds). So the latch is set on every tick within 120 ticks of such a push; it has nothing to do with detecting the hero. With the scripted-mode flag clear, the countdown alone never starts a slip.

### 4.4b Melee and dagger attacks

**Damage is not reduced.** Every melee hit of this boss (thrust lunge, sword swing, jump thrust, back-stab, shadow thrust-lunge), the rainbow-dash hit and the light-arc hit take their damage from the hero's HP unchanged: the hero's shield, armour, armour Mote, petpet factor and the attacker's status do not change it. These are unreduced boss melee hits as defined in SPEC-0029 Behavior 10.5; the shared enemy damage chain is not applied. Only the two dagger kinds are projectiles of the enemy pool and follow the pool's hero-hit rule (SPEC-0029 Behavior 9). (High)

**Melee contact.** The thrust lunge, sword swing, jump thrust and shadow thrust-lunge hit only on a tick when the boss's weapon touches the hero. The boss's weapons come from its monster row (SPEC-0029). The fight's region (resource 755) uses monster row 4. Weapon N is a finite cylinder with the row's weapon-N radius and full length, centred on the anchor of the boss's attachment N − 1 (it extends half its length on each side of the anchor), with its axis along that attachment frame's Z axis. Weapon N is tested only while animation event bit 2^(N − 1) is on, against each of the hero's bone capsules. In row 4 only weapon 1 has a size: radius 6.0, length 1.75, tested while event bit 0x1 is on; weapons 2 to 5 have radius and length 0. Row 5 (used by regions 741 and 1258) has weapon 1 radius 5.5, length 4.0 and no class (Open question 35). The same weapon test is the "reach test" named in 4.8. (High)

**Attack windows.** The windows are the frames of each clip that carry event bit 0x1. The Gelert's clips have speed byte S = 2, so at rate R = 1 a clip advances R × 0.5 / S = 0.25 frames per tick (SPEC-0005): frame f covers ticks 4f to 4f + 3 after the clip starts. Tick 0 is the tick on which the boss's update requests the clip. The request is made in the boss's update, and the clip starts in the animation pass of the same tick with its cursor at 0; that pass then advances the cursor to 0.25. The fade-in from the last pose changes only the clip's weight, not its cursor. The window test runs in the boss's update, before that tick's animation pass, so on tick k (k of 1 or more) it reads frame trunc(0.25 k). Frame 0 is therefore read on ticks 1 to 3 only (on tick 0 the test still sees the previous clip), and every later frame f on ticks 4f to 4f + 3. (High for the frames; Medium-High for the first tick.)

Test vector: a window on frames 5 to 7 of a clip requested on tick 0 is active on ticks 20 to 31; a window on frame 0 alone is active on ticks 1 to 3.

| Attack (animation) | Window frames (of clip length) | Window ticks | Damage |
| --- | --- | --- | --- |
| Sword swing (26) | 6 to 9 (of 20) | 24 to 39 | 20 (word at 0x2DAD84) |
| Thrust lunge (28) | 6 to 15 (of 23) | 24 to 63 | 25 (word at 0x2DAD80) |
| Jump thrust (29) | 12 to 18 (of 25) | 48 to 75 | 5 (fixed constant) |
| Shadow thrust-lunge (28) | as the thrust lunge | 24 to 63 | 5 (fixed constant); never entered (Edge cases) |

**Once-per-attack rule.** The lunges, the sword swing and the jump thrust run their contact and block tests only while the attack's once-per-attack flag is clear. The flag is set when a hit is sent, so each attack deals its damage at most once. A blocked attack leaves the state at once, so it needs no flag. If the weapon does not touch the hero on a tick, nothing is sent and the test is repeated on the next tick while the window lasts.

**Block test.** When the weapon touches the hero, the hero is guarding, and dot(hero facing, boss facing) < −0.5 (the two face each other), the attack is blocked: the boss goes to the sword-blocked state and the hero's take-hit is not called, so the hero loses no HP. Otherwise the hit of the attack's damage is sent. The test uses the two facings, not the direction between the characters. (High)

**Invisibility rule (end of a lunge or sword swing).** When the animation ends and the boss is invisible: if no hit was sent during this attack, the invisible flag is cleared and the state-changed marker is set; if a hit was sent, the boss starts the start-invisible state (4.11). This rule applies to the thrust lunge, the sword swing and the shadow thrust-lunge. (Medium-High; the back-stab rule is the same.)

- **Thrust lunge** (animation 28). On the tick the animation starts, the boss turns toward the hero at once (steer weight 1.0; Medium). During its window the once-per-attack rules apply: a blocked attack goes to sword-blocked; otherwise a hit of 25 damage is sent to the hero and the once-per-attack flag is set. When the animation ends: idle, the invisibility rule, and T_a is reset. Confidence: High for the states and the damage; Medium for the steer.
- **Sword swing** (animation 26). Every tick, including the ticks before the animation starts, the boss turns toward the hero with steer weight 0.5 (Medium). Once per attack, the same contact and block rules apply, and a hit of 20 damage is sent when not blocked. When the animation ends: idle, the invisibility rule, and T_a is reset.
- **Sword blocked** (animation 27): the boss's attack was blocked by the hero. Ends idle. A hit during this state gives a 50 % shadow slip, otherwise a flinch (4.13). Neither timer is re-armed when it ends (the attack timer was 0 when the blocked attack was chosen, and stays 0). The state plays clip 27 once, 45 ticks, and slides the boss back 1.4 units.
- **Jump thrust** (animation 29): a melee reaction (4.13). Once per attack, the same contact and block tests apply; otherwise a hit of 5 damage is sent. Ends idle with no timer re-armed. It does not turn toward the hero. Its clip first carries the boss 6.4 units backward, then forward to a net +3.1 (G6b); by the window (ticks 48 to 75) it is back near its start, so a level hero within 3.0 in front is touched on tick 48, one at 4.0 to 5.5 on ticks 50 to 53, and one beyond about 5.7 never (Reach and distances; corrects an earlier reading that it usually reaches the hero only in the second half of the window).
- **Dagger throw** (animation 31 for the windup; daggers are released while event bit 0x20, event 32, is on). During the windup the boss turns toward the hero with steer weight 0.5 on every tick after the request tick, including the release frames, so each dagger is aimed at the hero's position when it is launched (High for the identity of the call, Medium for the weight). The dagger throw has no root travel and is reaction-capable: a hero swing or shot can replace it (a dodge or a melee dodge ends it early, with no re-arm).
  - Release: event bit 0x20 is on at frames 6, 12 and 18 of the clip (ticks 24 to 27, 48 to 51 and 72 to 75). A dagger is launched only when the animation's integer frame differs from the frame recorded at the previous launch, so each release frame gives one launch: three daggers per throw. (High)
  - Launch: the origin is the boss's attachment point 1 (the second attachment frame, the one gated by event bit 2). The boss's origin query takes the first of event bits 1, 2, 4, 8 and 16 that is on in the current frame, and uses attachment 0 when none is on; the dagger throw clip has bit 2 on, together with bit 0x20, at frames 6, 12 and 18, so the origin is attachment 1 at each release (Medium-High, because the query is the shared one of the character class). The aim point is the hero's position raised 2.5 units (Medium). Each dagger is an enemy-pool projectile of kind 0, visual 0, radius 1.25, damage 10.0 (word at 0x2DAD78). Its velocity is the unit direction from origin to aim point times 1.5 units per tick (word at 0x2DAD70). (High)
  - Lifetime: trunc((distance + 10) / 1.5) ticks, so the dagger travels about distance + 10 units. (High)
  - Hits on the hero follow the pool rule of SPEC-0029 Behavior 9 (shield, armour, a block gives max(1, 0.25 D), reflection of kind 0 with a shield Mote).
  - Ends idle, and T_a is reset.
- **Back-stab** (animation 30, played at speed 2.0): entered only as the shadow back-stab (4.8); there is no normal back-stab. It has no weapon volume and no block test, so it cannot be blocked: on the first tick that the horizontal distance to the hero is below 6.0 while animation 30 plays, a hit of 5 damage is sent, once. Event bits are not read. Ends idle, and T_a is reset. (High)

### 4.5 States that allow reactions

Missile and melee reactions (4.13) run only in these states:
- idle, and the three strafe states (approach, left, right), and back-up;
- flinch;
- the three stances (superior sneer, angry, injured);
- the dagger-throw state.

They do not run during any other attack, shadow attack, healing, the wait state, mirror attack, or death. When reactions are gated off, the melee-reaction latch (4.13) is cleared.

### 4.6 Elapsed-time counter

- An elapsed-time counter rises by 2 each tick while it is below 7200, and by 1 each tick after that. It is not advanced in the scripted wait state (High).
- It is set to 0 on each HP-band change and when a stance ends.
- The boss damage-taken factor is the float 1 + 4 × counter / 7200 (not truncated), recomputed every tick the counter advances, starting at 1.0. It multiplies every hero hit on the boss (4.13). With the counter rising 2 per tick the factor is 1 + t / 900 until t = 3600 ticks (5.0), then rises 1 per 1800 ticks without limit. Every band change and every stance end sets the counter to 0, so the factor returns to 1.0 (High; Open question 3).

### 4.7 Shadow slip

Shadow slip is a vanish, a choice, and a reappear. It leads into every shadow attack in 4.8.

1. **Vanish** (phase 0): play animation 24. Its clip effect list shows effect 427 at the boss (4.15); the reappear animation 25 shows the same effect. When it ends, go to phase 1.
2. **Choose** (phase 1): pick the shadow attack with the chooser (4.8), choose the landing point (below), show an effect, go to phase 2.
3. **Reappear** (phase 2): on the phase-1 handler pass, copy the selected landing point into the boss position and advance to phase 2. Then play animation 25. For shadow back-stab and mirror attack the reappear runs at animation speed 2.0. When it ends, the boss enters the chosen attack state.

Landing point: a random horizontal direction; a distance drawn uniformly between the chosen attack's reappear minimum and maximum, in world units from the hero's position. The candidate keeps the hero's current vertical coordinate because the offset has no vertical component. The candidate is accepted if it lies on any navigation mesh in the boss's current world region; this query scans the region's complete mesh list, not only the boss's assigned or current mesh, and returns no owning-mesh identity to the landing picker (High). The accepted point is copied directly to the boss position. The picker does not project it to ground height or test path reachability, line of sight, headroom or collision clearance. During pack 985's active flag-281 fight the Gelert stands on the arena ground mesh (mesh 0): its own start-up code assigns the first ground mesh that contains its position when it has none, and the level script's mesh helper assigns mesh 0 to entities 0 to 4 on the ticks when no stage applies. The all-mesh hit of the landing picker does not change that assignment (Medium-High). A later shared ground-probe and physics update may settle the character against the local height field, but the result depends on the random candidate x/z, later movement and the tick of correction. If none of 10 tries is accepted, the chosen attack becomes just appear and the boss stays where it is (the landing is the boss's own current position, not the hero's). The mirror attack and the rainbow dash do not use this search at all: their landing is the boss's home point (the position of its placement record, G2), copied into the boss position on the same pass. Back-stab does not use this search: it lands 3 units behind the hero and tests that point against the same full region mesh list; if it is not on any such mesh, the boss uses the shadow dagger instead. Reappear values are distances (world units), not times. Locator 9 places the hero during arrival/intro; it is not the boss landing point. The exact post-grounding height and mesh patch remain candidate- and tick-dependent. Confidence: High for the all-mesh boolean test, direct candidate placement and lack of a mesh assignment during the active fight; Medium-High for the later grounding outcome.

Shadow slip sets the shadow-slip timer again when the shadow attack ends (4.11).

### 4.8 Shadow-attack chooser

Inputs for the boss's current pattern p (1..4):

- M = backstab bucket: pattern 1 gives 0. Otherwise the 4.3 table row above, with 0, 33, 25, 4 for patterns 1..4.
- D = dagger bucket: 0, 33, 25, 4 for patterns 1..4. A mirror image uses 25.
- H = light-arc-or-heal bucket: 30, 33, 25, 32 for patterns 1..4.
- MI = mirror-image bucket: 10 for patterns 2..4; 0 for pattern 1.
- RD = rainbow-dash bucket: 20 for pattern 3, 50 for pattern 4; 0 otherwise.
- r = fresh roll, integer 0–100 inclusive.

The buckets are cumulative, in the order M, D, H, MI, RD. The pattern selects which rule applies:

| Pattern | Rule |
| --- | --- |
| 1 | Always just-appear. |
| 2 | r < M → backstab. Otherwise M ≤ r < M+D → shadow dagger. M+D ≤ r < M+D+H → light arc. Otherwise just-appear. There is no mirror or rainbow bucket. |
| 3 | r < M → backstab. M ≤ r < M+D → shadow dagger. M+D ≤ r < M+D+H → light arc if HP is at least 37.5 % of max, or a second fresh roll is 50 or more; otherwise healing. M+D+H ≤ r < M+D+H+MI → mirror attack (target: the hero). Otherwise just-appear. There is no rainbow bucket. (High) |
| 4 | r < M → backstab. M ≤ r < M+D → shadow dagger. M+D ≤ r < M+D+H → light arc if HP is at least 12.5 % of max, or a second fresh roll is 50 or more; otherwise healing. Then mirror attack for MI, then rainbow dash for RD (target: the hero). Otherwise just-appear. Pattern 4 never starts a heal above its target. (High) |

- cur = the boss's current HP; max = its maximum HP. The thresholds 0.375 and 0.125 come from testing (cur − 0.25 max) / (0.25 max) ≥ 0.5 and cur / (0.25 max) ≥ 0.5 (Medium on this form).
- Just-appear (4.8 table below) is used when no bucket matches.

Shadow attacks:

| Attack | Behavior | Animations | Ends |
| --- | --- | --- | --- |
| Just appear | Reappears in place. If a roll above 74 is made, animation 22 plays first. | 22 (only when rolled) | Idle; shadow-slip timer re-armed |
| Shadow back-stab | Lands 3 units behind the hero. Animation speed is 2.0 during the whole attack. No weapon volume and no block test (unblockable): a hit of 5 damage is sent once, on the first tick that the horizontal distance to the hero is below 6.0 while animation 30 plays (High). If the landing point is not walkable, a shadow dagger is used instead. | 30 (reappear at speed 2.0) | Idle; attack timer and shadow-slip timer re-armed |
| Shadow thrust-lunge | Never entered: no chooser selects it (Edge cases). If entered it would act as the thrust lunge (4.4b) with 5 damage: weapon 1 contact, the block test, the once-per-attack rule and the invisibility rule. | 28 | Idle; shadow-slip timer and attack timer re-armed |
| Shadow dagger | Throws daggers while event bit 0x20 (event 32) of animation 31 is on, at frames 6, 12 and 18. There is no frame rule: a dagger is launched on every tick the bit is on, 4 ticks per release frame, so 12 launches are attempted per attack; at most 10 can be live at once (pool limit 10). Each is an enemy-pool projectile of kind 0, radius 1.25, damage 3.0, speed 5.0 units per tick (unit direction times 5.0), lifetime trunc((distance + 10) / 5.0) ticks; hits follow the pool rule (SPEC-0029 Behavior 9). Aim point: the hero's position raised 2.5 units (Medium). Landing distance uses the dagger-reappear minimum (Edge cases). (High) | 31 (windup and release; the release is event bit 0x20 of this clip, not animation 32) | Idle; shadow-slip timer re-armed |
| Light arc | Waits, then loops an arc for up to 200 ticks. The arc is two moving ball objects, A and B (the "end objects"; they are spawned effect objects, not level locators and not parts of the boss). At the tick the windup animation (37) ends, both are created at the boss's position raised by the height offset 1.5 (the float word at 0x2DAD8C), and the loop (animation 38) starts. Let d be the horizontal unit direction from the boss to the hero at that tick (4.4) and l = (−d.z, 0, d.x). The direction is fixed for the whole arc; the balls do not home. Ball A flies along (d + l) / √2 and ball B along (d − l) / √2, so their paths diverge at 90 degrees (45 degrees either side of d) at constant height. Each ball moves 20 / 60 units per tick (the speed word at 0x2DAD48 divided by the tick rate), starting on the first loop tick after the launch tick, so after k loop ticks each ball is 0.3333 k units from the start and the segment from A to B is perpendicular to d, centred on the point 0.2357 k units from the start along d, of length 0.4714 k units. The arc ends after 200 loop ticks (about 66.7 units of flight per ball) or at the first hit on the hero. Hit test: each arc ball is a sphere of radius 2.0 tested against the hero's body cylinder (the hero's motion-record radius and height). The arc deals the light-arc damage (20, word at 0x2DAD50) once per arc, unreduced, with attacker id 0. Missile wall (High for the rule and for the sign of n): on every loop tick the arc also cancels hero missiles. Let A and B be the positions of the two ball objects above, u the horizontal unit vector from A to B (u = −l, so u = (d.z, 0, −d.x)), and n = (−u.z, 0, u.x); this makes n equal d, the boss-to-hero direction at the launch, for the whole arc. The wall is therefore the line through A and B, perpendicular to d, advancing along d at 0.2357 units per tick. For each of the hero's six missile slots that is active, with point P and per-tick step s (its direction times its speed times the tick time step), let Q = P + 10 s. If n · (horizontal unit vector from A to P) > 0 and n · (horizontal unit vector from B to Q) < 0, the slot is made inactive at once (the missile is removed with no hit). So a hero missile due to cross the arc's vertical line from the n side within 10 ticks is stopped; the line has no end limits, and a missile crossing the other way passes. Confirmed in code 2026-11-09 (High): the test is made by one helper against the rig object that holds the two end objects (the anchor position is the vector at +0x30 of the first end object, the tip position that of the second; each defaults to (0, 0, 0, 1) when the object is absent); with u the horizontal unit vector from the anchor to the tip (the y part zeroed, then normalised) and n = unit(u × up), it returns "stop" only when dot(n, horizontal unit vector from the anchor to the missile point) is strictly above 0.0 and dot(n, horizontal unit vector from the tip to the look-ahead point) is strictly below 0.0; a dot of exactly 0.0 counts as the other side. It is called from the light-ball shot code. High for speed and damage; Medium for the hit shape and the height meaning. | 36 (wait), 37, 38 (loop), 39 | Idle; shadow-slip timer re-armed |
| Shadow healing | See 4.9. | 32, 33, 34, 35 | Idle; shadow-slip timer re-armed |
| Mirror attack | The boss reappears at its home point (4.7), plays 41 at speed 2.0, then 42. When animation 42 passes half-way, the clones waiting in base state 42 are woken (4.10). It is hittable throughout and cannot react. | 41 (speed 2.0), 42 | Idle; clone mode on (4.10) |
| Rainbow dash | Visits eight target points in a random order (see Rainbow dash below). A hit of 15 damage (word at 0x2DAD90), unreduced and unblockable, is sent on the first tick with the horizontal hero distance below 8.0 (word at 0x2DAD58), once per target point, so up to 8 hits per dash (High). After the eighth point the boss plays animation 45 until it ends, then is idle and its shadow-slip countdown is re-armed. | 44 (points), 45 (end) | Idle; shadow-slip timer re-armed (the attack timer is not); the pattern is not changed |
| Start invisible | Plays animation 40, then the boss is invisible. | 40 | See 4.11 |

The block test used in 4.4b: dot(hero facing, boss facing) < −0.5, that is, the two characters face each other (High). The back-stab, rainbow dash and light arc have no block test.

**Rainbow dash** (High):

- Target points: at controller start the eight points are the positions of the level locators with ids 200 to 207. A missing locator leaves its slot unchanged.
- Order: at dash start (sub-step 0) the visiting order is built. For each of the 8 picks a draw r = (output of the shared Mersenne Twister generator) mod 7 (0 to 6) is made; the pick is the first unused index at or after r, cyclically over 0 to 7. So index 7 is reached only by moving on from a used index.
- Motion: each tick of sub-step 1 the boss plays animation 44 and turns toward the target (weight 1.0). It moves straight toward the target point, horizontally, by 40 / 60 units per tick (the rainbow-dash speed word 0x2DAD54, 40, divided by the tick rate 60). It snaps onto the point when the remaining distance is at most one step, then goes on to the next point. The dash step is added to the position by the state itself; the clip's root travel (clip 28 carries 1.319 per key forward, 0.33 per tick along the facing, which is toward the target) comes on top through the usual motion step, so the boss really covers about 1.0 unit per tick (60 units per second) rather than 0.667. The tick on which the dash step reaches a point moves the boss onto it exactly; the next point is selected one tick later (that tick moves nothing by itself), and the first point is chosen on the tick the state starts, before the dash begins. The boss starts from its home point (4.7). Medium-High for the sum of the two movements; High for each.
- Hits: the hit latch is cleared at the start of the dash and at every point. While it is clear, the first tick with the horizontal hero distance below 8.0 deals 15 and sets it. So up to 8 hits per dash.
- After the 8th point animation 45 plays; then the boss is idle and the pattern is recomputed.

### 4.9 Healing

The healing state runs in phases:

- **Phase 0**: play animation 32 until it ends.
- **Phase 1** (heal loop): the first tick starts animation 33 and sets the countdown to 5 (the healing tick period) with no heal. Each following tick subtracts 1 from the countdown while it is above 0. On the tick when the countdown is 0, the boss heals once and resets the countdown to 5. So the first heal comes 6 ticks after the loop starts, then one heal every 6 ticks (0.1 s).
- **Each heal** adds trunc(0.01 × max HP) to the boss's HP (the step is 0.01 × max HP as a float, truncated when applied). The target is max HP divided by the pattern number: pattern 1 heals to full, pattern 2 to 50 %, pattern 3 to about 33 %, pattern 4 to 25 %. If the target is below the current HP plus that step, the boss heals trunc(target − HP) and goes to phase 3.
- **The heal operation** adds its amount to HP and caps the result at max HP. There is no lower clamp and no death call, and the HUD is refreshed. So a negative amount (the final step when HP is above the target) lowers HP by trunc(HP − target) toward the target (High).
- **Phase 3**: play animation 35 until it ends, then the boss is idle and its shadow-slip countdown is re-armed.
- **Phase 2** (entered by a hit during phase 0, 1 or 3; phase 2 itself cannot be hit): play animation 34 until it ends, then the boss is idle and its shadow-slip countdown is re-armed. No further heal happens in phase 2.

- **Heal aura** (High): when animation 32 starts (phase 0), effect 428 starts at the boss. It is stopped when phase 2 or phase 3 starts, so it shows through phases 0 and 1. Effect 428 is red sprites, a model and a white sprite, 2 s long and repeating every 0.4 s (Medium for the visual description).

Confidence: High for the phase 1 timing and the phase 2 ending; Medium for the hit entry.

Healing starts only from the chooser's light-arc-or-heal bucket (4.8) at patterns 3 and 4. The landing range for healing (25, 35) is used only by the landing search for this attack. Healing length and healing factor are not used.

### 4.10 Mirror images (clones)

- The boss has up to four mirror-image slots. Slot 1 is placed at +5 units from the boss along the lateral axis, slot 2 at +10, slot 3 at −5, slot 4 at −10 (High for the offsets and for the axis). The lateral axis is the boss's own side axis: with f the boss's horizontal facing (unit length) at the tick of the placement, it is the unit vector (0, 1, 0) × f = (f.z, 0, −f.x). It is the same side axis the character's heading basis keeps (SPEC-0040), so for a boss facing +Z it is +X. The clone's position is the boss's position plus the offset times this axis (same height as the boss), and the clone takes the boss's facing. Example: boss at (10, 2, 20) facing +Z: slots 1 to 4 land at (15, 2, 20), (20, 2, 20), (5, 2, 20) and (0, 2, 20).
- The boss and its clones are the characters of one block: block slot 0 is the boss, and slots 1 to 4 are the clones. The clone flag and the slot index of each character come from its block slot (High). In pack 985 the five Gelert records are consecutive ordinals 0 to 4, and the level script controls entity 0 as the primary boss. Under the automatic stream path with a fresh character block, ordinal 0 binds to block slot 0 and ordinals 1 to 4 to slots 1 to 4 in order, on the first streaming pass after the hero is placed at locator 9: that one pass also binds Illusen's ordinal 6, which this branch's filter admits, into slot 5 (she is farther from the hero than the Gelert point: squared distance 8,465 against 804), so she never takes a Gelert's slot. At the hero's arrival position on entry record 0 every record is out of range (squared distance above 78,400), so nothing binds earlier (High for the stream and slot rules; Medium-High that nothing else binds first; see Encounter flow, Open question 39 and SPEC-0041, module 985).
- At set-up the boss's block is made non-streaming (keep 0; SPEC-0006), so the streaming pass never wakes the clones. Each clone is put into base state 42 (the parked state of SPEC-0040) and AI mode 32, with its clone flag set (High).
- An inactive clone waits in base state 42 in AI mode 32. It is not drawn (SPEC-0046, Drawable test), and its update returns at once (High).
- Wake: while the boss's mirror attack is in its second animation (42) and that animation has passed one half of its length, the four clones are visited in one loop, in slot order 1, 2, 3, 4, on that one tick, and each clone whose base state is 42 is set to base state 1 and given a forced bind of its own slot and record (SPEC-0006). The forced bind does nothing when the slot is already occupied, and all four clones were bound on the first streaming pass, so the wake is the base-state write alone. All four clones therefore wake together on the first tick of the condition (the clip's progress passing one half), not one by one; on later ticks their base state is 1 and the loop skips them. The animation-42 test applies to the boss only; the clone test is on the base state (High).
- A clone's activation step does nothing while its base state is 42. Once the wake has set base state 1 (the clone's own update runs on the same tick when its slot comes after the boss's in update order, otherwise on the next), it places the clone at its slot offset (+5, +10, −5 or −10 lateral) from the boss's position of that tick (the home point, since the mirror attack lands the boss there), with the boss's facing, requests animation 42 and sets the clone's clip cursor to the boss's cursor (one tick behind), clears its invisible flag, evil-laugh flag, melee latch and fade value, sets its back-up cooldown to 60, draws its slip timer and its attack timer (pattern 1 ranges, 120, 180 or 240 ticks for the attack timer), and assigns controller state 0x16 (decimal 22), the mirror-attack handler (High; Medium for the cursor copy). In that state a clone only waits for animation 42 to finish; then it goes to state 0 with its clone mode on and fights as in G14.
- When animation 42 ends, the boss is idle and clone mode is on. (High.)
- The shadow-mode latch used by the shadow-slip chooser belongs to the current AI instance: the mirror handler sets that instance's latch, and its chooser reads the same latch. A clone running the handler would set its own latch; this does not set a shared flag on the boss or clone group. A clone does enter the mirror-handler state: the activation step assigns it at the wake, and it leaves it when its animation 42 ends (High); it never chooses the shadow-slip attacks itself.
- Clones use the mirror values of the chance tables: dodge 25, dodge-melee 15, jump-thrust 15, dagger bucket 25. The melee shadow bucket uses the same table for clones and bosses. A clone's pattern is always 1 (its HP is never lowered, so no band check ever changes it), so its attack and slip timers always use the pattern-1 ranges whatever the boss's band (G14). The dagger bucket and the back-stab bucket are never read for a clone, because a clone never slips.
- A clone never starts a shadow slip itself (4.4).
- A hit on a clone sends it back to the inactive wait: base state 42 and AI mode 32. A one-shot effect 559 (a yellow flash with white sparks) plays at the clone, detached from it (High for the id). The clone is hidden again until the next wake (Medium-High).
- A clone in base state 42 is silent: the animation-sound step does not run in that base state (4.15).
- The clone controller state is 0x20 (decimal 32) while inactive and 0x16 (decimal 22) after activation; state 0x16 is the mirror-attack handler (High).
- While clone mode is on and at least one clone is alive, the boss does not re-arm its shadow-slip timer. The timer is re-armed once no clone remains.

Clone slots are held in the boss's own state. At set-up the boss (block slot 0) collects the characters of block slots 1 to 4 as its clones (High).

### 4.11 Timers and counters

| Timer or counter | Unit | Set by | Remarks |
| --- | --- | --- | --- |
| Shadow-slip timer | ticks | 60 × a whole number of seconds drawn from min to max inclusive, pattern-indexed (Layout) | Re-armed after shadow attacks, healing and just-appear |
| Attack timer | ticks | 60 × a whole number of seconds drawn from min to max inclusive, pattern-indexed (Layout) | Re-armed after dagger throw, back-stab, thrust lunge, sword swing, and shadow thrust-lunge |
| Back-up cooldown | ticks | 60 (1 s) when the close-range check runs with 0 | |
| Back-up cooldown at creation | ticks | 60 (1 s), counted down like the others | The first close-range decision in that second chooses a left or right strafe instead of a back-up (High). |
| Heal countdown | ticks | 5 (tuning word, raw ticks) | Heal every 6 ticks (4.9) |
| Invisible flag | flag | Set when start-invisible ends | Cleared at the end of a lunge or sword swing when no hit was sent (4.4b); cleared by back-stab when it was set and the back-stab's own check allows (Medium) |

Start-up values (High): at creation the pattern is 1. The shadow-slip countdown is a whole number of seconds from 8 to 15 inclusive times 60 (480, 540, ..., 900 ticks), and the attack countdown 2, 3 or 4 seconds times 60 (120, 180 or 240 ticks). Both use the pattern-1 ranges. All timers count down from creation, including while the boss waits for its release (4.3).

Invisibility: while invisible, the dagger-throw choice is not allowed, and the shadow-slip timer does not trigger a shadow slip. The start-invisible animation fades the boss with value int((1 − f) × 128), where f is the animation's progress (Medium). The invisible flag starts at 0, only the start-invisible state sets it, and that state is entered only from the end of a lunge, sword swing or back-stab while the flag is already set. So the flag never becomes 1 in play, no chooser selects the invisible attack, and every rule here that tests the flag (the dagger gate, the slip gate, the end-of-attack branches) always sees it clear (High).

### 4.12 Damage

Each value below is the damage itself: a hit of N damage takes N HP from the hero.

| Attack | Damage | Source | Reduction | Block |
| --- | --- | --- | --- | --- |
| Thrust lunge | 25 | word at 0x2DAD80 | none | yes (4.4b) |
| Sword swing | 20 | word at 0x2DAD84 | none | yes |
| Jump thrust | 5 | fixed constant | none | yes |
| Back-stab | 5 | fixed constant | none | no |
| Shadow thrust-lunge | 5 | fixed constant | none | yes; never entered |
| Rainbow dash | 15 | word at 0x2DAD90 | none | no |
| Light arc | 20 | word at 0x2DAD50 | none | no |
| Dagger | 10.0 | word at 0x2DAD78 | pool rule (SPEC-0029 Behavior 9) | pool rule |
| Shadow dagger | 3.0 | fixed constant | pool rule | pool rule |

- "none": unreduced boss melee (SPEC-0029 Behavior 10.5); the hero's take-hit subtracts the value from HP unchanged. A blocked melee attack calls no take-hit at all, so the hero loses nothing (4.4b). (High)
- No attack of this boss applies a status to the hero.
- Light-ball damage (25) is stored, but no reader is known; no described behavior uses it for damage.
- Healing adds HP (4.9); it deals no damage.

### 4.13 Reactions

**Missile reaction.** Runs when the state allows (4.5). For each active hero missile, its path is the segment from its point to point + 180 × velocity. The path is tested against every bone entry in the boss's active pose whose collision radius is positive; there is no separate named-bone whitelist, and entries with radius 0 or less are skipped (High). Each eligible bone uses its record's full length and radius; the cylinder follows the posed bone's local Y axis and is centered halfway along its length from the posed base (SPEC-0029, Behavior 26; Medium-High for applying this placement to the swept-path caller). This is the same shape convention as the Gnarfas missile reaction (SPEC-0020), with 180 ticks instead of 10. The separate incoming-hit check later tests the missile slot sphere against the boss's vertical body cylinder from its monster row (High); the body cylinder is not the missile-reaction shape. A missile whose hit id is in the ring of the last 10 hit ids is skipped. A new id is added to the ring (replacing the oldest) as soon as its path passes, before either roll, so every missile gets exactly one pair of rolls. The six shot slots are visited in order and each qualifying slot writes the state, so the last one wins. For a missile whose path passes and whose hit id is new:
- a roll r1 in [0, 100]: if r1 < dodge chance (40 for patterns 1..4; 25 for a mirror image), the boss dodges: state 6, animation 18, or animation 19 when a 0/1 roll is 1 (Medium on the variant roll);
- otherwise a fresh roll r2 in [0, 100]: if r2 < deflect chance (80), the boss deflects: state 7, animation 21 while the missile is within 10 units, or idle animation 0 while no active hit is near (Medium);
- otherwise nothing.

Deflect detail: the 10 units is a fixed constant, not a tuning word. It is the full 3D distance from the boss's position (its feet point) to the point of the last active shot slot (highest slot index; the others are ignored), tested every tick. With no active hit slot near the boss, it is idle (animation 0); with no active slot at all, the state ends at once. Confidence: High for the 10 units; Medium for the vector.

**Melee reaction.** Runs when the state allows (4.5), once per swing (a latch is set after one decision; it clears whenever the hero is not attacking). It applies when the hero is attacking and both tests pass:
- facing test: dot(hero facing, direction from the hero to the boss) > the facing threshold stored on the hero (0.5, a 60-degree half-angle; SPEC-0038, hero swing classes);
- reach test: horizontal distance < 1.5 × the reach stored on the hero (8.0, so 12.0 units; the same hero word SPEC-0038 uses for the common enemies' block and dodge).

The latch is set when the reaction is evaluated, whatever the tests say: the first tick on which the hero's attack query is true and the boss is in a state that allows reactions makes the one decision of that swing. If the facing or reach test fails, nothing happens and the swing is not tested again. The latch is cleared on every tick the hero is not attacking and on every tick the boss is in a state without reactions (4.5). The hero's attack query is the attack-active word (event bit 0x1 of its current frame; SPEC-0036, Attack window), so the decision falls on the first active frame of the swing, not on the button press (High for the rule; Medium-High for the query being the attack-active word).

Then:
- a roll r1: if r1 < dodge-melee chance (100 at patterns 1..4; 15 for a mirror image), the boss dodges melee: state 8, animation 20. When that ends, a roll below 50 starts a shadow slip (not for a mirror image), otherwise the boss is idle;
- otherwise a fresh roll r2: if r2 < jump-thrust chance (15, 15, 10, 10 for patterns 1..4; 15 for a mirror image), the boss jumps: state 9, animation 29 (4.4b). Starting the jump thrust also clears the attack's once-per-attack hit flag.

Confidence: High.

**Missile hit.** Each hit slot is tested as a sphere (the slot's point and radius) against the boss's vertical body cylinder (the body record's radius and height, standing on its position), as for Gnarfas (SPEC-0020) (High).

**Hittable states** (High). The boss's own hit check runs only in these states: idle, evil laugh, dagger throw, thrust lunge, sword blocked, shadow thrust-lunge, shadow dagger, mirror attack, just-appear, the five strafe states (idle strafe, right, left, back-up, approach), healing in phases 0, 1 and 3, light arc in its wait (phase 0), loop (phase 2) and end (phase 3), and rainbow dash in all its phases. In every other state a hit does nothing: flinch, the three stances, missile dodge, deflect, melee dodge, jump thrust, back-stab, sword swing, shadow slip, shadow back-stab, invisible attack, start-invisible, death, the two clone wait states, healing phase 2 and the light arc's windup (phase 1). A hit is either a hero melee hit (the hero is attacking, the attack is active and the hero's weapon touches the boss) or a hero missile slot whose sphere overlaps the body cylinder; each overlapping missile slot is removed.

**Hit reaction.** A hit on the boss in a hittable state when the boss is not a clone:
- if the boss is in the sword-blocked state, a roll below 50 starts a shadow slip (4.7), otherwise a flinch;
- during the light arc loop, the arc is removed (the routine that frees the arc object's five ball and effect slots runs, so both balls and their followers vanish and the arc stops hurting) and the boss flinches (High for the flinch and for the removal);
- otherwise, a flinch (animation 16, or 17 when a 0/1 roll is 1). The flinch plays the shared hit flash at the boss: one-shot effect 222, detached (SPEC-0034; the same helper shows 220 instead when called with its flag set, which the flinch does not do) (High for the id; Medium for the point);
- during healing, a hit moves healing to phase 2 (4.9);
- while script control is clear, the hit counter rises by 1 for each hero missile slot that hit, and by 1 more when the flinch starts, except for a flinch out of the light arc (High). A melee hit therefore adds 1 and a single missile hit adds 2; a hit that moves healing to phase 2 or starts the shadow slip adds only its missile count (4.3);
- a hit on a clone sends it back to the inactive wait (base state 42, AI mode 32, with the one-shot puff effect 559 at the clone; 4.10).

**Damage to the boss.** On a hit that starts a flinch (the plain flinch and the flinch out of the light arc), the boss loses truncate(D × F) HP, where D is the hero's melee damage at scale 1.0 (melee contact) or the hero damage query for that missile slot against the boss (SPEC-0036), and F is the damage factor of 4.6 (1.0 at the start of the fight and after every band change or stance end). When several missile slots overlap on one tick, all are removed and counted, but only the last one's damage is applied. A melee contact takes the tick, so missile slots are not tested on that tick. The boss takes no HP from a hit that only moves healing to phase 2, from a hit in the sword-blocked state that rolls under 50, or when it is a clone. If the damage is at least the boss's HP, HP becomes 0 (no negative values) and the boss still flinches; the death state follows on the next decision-ready tick (4.2). The boss has no defence value, so there is no subtraction (High for the structure and the factor; Medium-High for the missile-slot query).

Confidence: High for chances, the gate, the rolls, the facing and reach tests, and the positive-radius eligibility rule; Medium-High for the bone cylinder placement in the missile reaction (Open question 7).

### 4.14 Settle test and tracked-point smoothing (dormant in the shipped build)

The shared tracked-point settle rule is defined in SPEC-0040. It is not specific to Gelert. In the shipped build no state of this boss reaches it. The dispatcher that selects the two-point settle and smoothing has no caller. The four-point variant and the second move routine have no caller at all, and no code pointer in the boot program refers to any of these routines (High for the call graph; Medium-High overall). The dispatcher is also named by no function-pointer table, method table, stored address, split address load or host export table, and no level module calls it, so no data route reaches it either (High for the dispatcher and the four-point variant). The rule is kept here as a reference only. Implement it as a reference, or leave it out. A reimplementation that wires it to the Gelert states would change behavior (Edge cases).

The values below are those of the two-point path.

Each tracked point has a saved copy. The rule uses the same "moving" test for every point.

**Moving test.** A tracked point is moving when any of its three axes differs from its saved copy by more than 0.02 units (a fixed constant, not a tuning word). A point that is not moving is stationary.

**Settle and smoothing (two points).** The rule runs only while the settled flag is clear. For each of the two tracked points, in order:
1. Apply the moving test.
2. If the point is moving, its saved copy moves 15 percent of the way toward the current position: saved = saved + 0.15 × (current − saved), per axis. The 0.15 is a fixed constant.
3. If the point is stationary, its saved copy is set exactly to the current position.
4. The updated saved copy is written to tracked-point slot 0x13 (the first point) or slot 0x14 (the second point), through the event 0x13 handler, which stores the four-float vector in that slot (Open question 27).
5. If neither point was moving on this run, set the settled flag.

If the settled flag is already set when the rule is reached, the rule does not run. A second latch is set to 1 instead. Confirmed 2026-11-09 (High) by the routine's readable form: on entry with the settled flag set it sets the second latch and returns at once with no comparison, copy or slot write; the first point is processed (and its slot written) before the second is examined; "moving" is any lane beyond ±0.02 strictly; a moving copy is a four-lane lerp 0.15 toward the current point. SPEC-0040, Behavior 8b rule 2, now says the same. The second latch is also set to 1 when the object is created. Its only reader is the dormant mode-0 settle routine of SPEC-0040 (rule 3), which, when it sees the latch set, clears the latch and the settled flags and restarts its saved copies; that routine has no caller either, so the latch has no effect in the shipped build (Medium; Open question 23). Only the dormant move-start routine clears the settled flag and the second latch (Open question 24).

Confidence: High for the 0.02 threshold, the 0.15 blend, the stationary copy, the settled-flag rule and the second latch's set. Medium for the two-point count and for the event report. The four-point variant has no caller (Open question 30).

**Drift quirk.** Because a stationary point's saved copy follows the point every run, a point that drifts by 0.02 units or less per run is never moving. Its saved copy keeps up with it, so the total drift is not bounded: the point can wander far from its starting place and still count as stationary. This is what the original does; it is not a designed dead-band (Medium-High).

A point that jumps by more than 0.02 units is moving. Its saved copy closes 15 percent of the gap on each run, and the point counts as stationary again once the gap is 0.02 units or less, when its copy is set exactly to the point.

**Gates.** The settle rule runs only while the settled flag is clear. The gate is the settled flag itself (High for the two-point routine; Open question 25). Nothing in the shipped build calls either part (Open question 22).

### 4.15 Sounds and effects

**Sound rule** (High). The boss plays its voices and attack sounds only through its main animation table (SPEC-0005, SPEC-0031); its controller plays no cue. Each sound entry names a cue of creature block 386 (sound group 1), the block shared by the boss and its clones. When a character's main animation id changes to k, the cue of entry k plays once, at that character's position on block 386's slot, on the first tick of the animation: every sound entry has start frame 0 and no keep-alive bit. The cue plays only while the block's slot is loaded, at the default volume and with no do-not-restart (SPEC-0031). A request of the animation that is already current starts nothing; a new start after a different animation plays the cue again. The animation-sound step runs every tick except while the character's base state is 42, so a hidden clone waiting in base state 42 is silent.

**Animation to cue** (fight region 755, block 386; animation -> cue, with the cue's subject):

- 2 -> 11 (taunt); 3 -> 2 (laugh); 16 and 17 -> 4 (take damage, the flinch); 18, 19 and 20 -> 10 (dodge).
- 24 -> 1 (disappear, the vanish); 25 -> 0 (reappear).
- 26 -> 6 (sword swing); 28 -> 7 (lunge; also the unreachable shadow thrust-lunge); 29 -> 8 (jump thrust); 30 -> 9 (back-stab).
- 31 -> 5 (dagger throw): once per throw, at the start of the windup. The three releases make no sound. The shadow dagger's windup (also 31) plays the same cue.

Silent: 27 (sword blocked), the healing animations 32 to 35, 37 and 38 (light arc loop), 44 (rainbow dash points), and every animation not listed, including 23 (evil laugh). The die cue 3 of block 386 is named by no animation, so it never plays. The Gelert records of regions 741 and 1258 carry no sound words at all.

**Effects started by code** (group 3 definitions of the shared effect resource 9; High for the ids):

- Heal aura 428 at the boss, from the start of animation 32 until healing phase 2 or 3 starts (4.9).
- Clone puff 559, one-shot and detached, at a clone that is hit (4.10).
- Hit flash 222, one-shot and detached, at the boss on a flinch (4.13; Medium for the point).
- The daggers and shadow daggers carry no effect id. The projectile's trail field holds entry 1 of the boss's table of model parts, which the pool's draw pass draws at the projectile's position; the part is shown only while a dagger slot is live (Medium-High; Open question 37).

**Clip effect lists** (animation-driven effects, SPEC-0034; Medium-High, because the list index follows the order of the region's effect records):

- 24 (vanish) and 25 (reappear): effect 427 (black sprites, 35 per second for 2 s) from 0.1 of the clip to its end, on attach slot 6.
- 18, 19, 26 and 27: an entry with id -1 and attach slots 0 and 0, which starts no particle effect but starts a swing trail along attach slot 0, from its origin to the point (0, 0, 5.5) (Open question 38).
- 37: effect 4009 (two model emitters of 17 each, every 0.6 s) at 0.9 of the clip; 38: effect 4009 from 0.5.

### Winning sequence (derived from the rules and the disc data; not played in the original; High unless marked)

1. Start. Pack 985 with flag 281 set: the activation moves the hero to locator 9 and the opening scene runs (letterbox, camera tracks 25 to 33, six dialogue boxes, about 1600 ticks with the boxes open). The boss, 28.3 from the hero, is held in its wait state until the scene's last call sets level state 0; a dialogue box stays open until the player closes it, so the scene has no timer of its own.
2. Fight. After release the boss slips into shadow on its first free tick, then attacks, slips and strafes on its own timers. The hero hurts it only with a hit that lands while it is in a hittable state (sword hits, Roberta's quick shots): a hit removes a clone and makes the primary flinch. Stay on the boss's route mesh side of the floor: a hero standing in the outer ring (5 to 12 beyond mesh 0) is farther than the boss can route to.
3. Phases. HP 800 falls below 599, 399 and 199: the boss enters a stance and waits for the matching level state (2, 4, 6) that the stage scene sets when it ends (fade, boss to locator 27, hero to locator 28, dialogue lines, release). Each scene ends by itself once its dialogue is closed.
4. End. HP reaches 0 only through a flinching hit; the check runs on the next decision-ready tick, so the flinch (57 or 73 ticks) finishes first, then event 7: the finale scene deactivates the boss and the four clones, sets flags 284 and 301, opens the north-east wall, ends instance 5 at stage 1 and sets level state 8. The hero walks north-east through the open diamond to the slab (trigger 4): item 30, flags 67, 283 and 302, movie 44, back to the village.
5. Waits with no timeout. (a) The release wait before level state 0, ended by the opening scene's last call. (b) Each stance, ended by its scene's level state. (c) Dialogue boxes, ended by the player. The boss cannot die in a state that takes no damage (healing, sword-blocked slip, stance); so the killing hit has to land in a state that takes damage. Nothing else holds it.
6. Cannot-finish check. None found: the finale exit is reachable (the south-west wall stays shut, the north-east wall opens), and the boss is released by a scripted call rather than a hero action. A developer start that skips the opening scene must set level state 0 itself.

## Edge cases and error handling

- **Settle rule is dormant** (High for the call graph): the settle test and the smoothing (4.14) do not run in the shipped build. The drift, jump and snap cases below describe the rule only and do not occur in play.
- **Pattern cannot fall back**: the pattern only rises. A healed boss keeps its pattern, so a pattern-4 boss healed to 80 % still uses pattern-4 timers and chances.
- **Band changes wait for a decision**: the band check runs only while decision-ready. A boss that enters pattern 4 during a swing starts the injured stance after the swing.
- **Stance deadlock**: the three stances wait for script bits 2, 4 and 6. If the script does not set the bit, the boss stays in the stance indefinitely.
- **Event 0 is not latched** (flag-clear branch only): while the hit counter is above 2, or the hero's HP ratio is below 0.55, event 0 is raised on every tick.
- **Scripted-mode override** (the shipped regime): if script control is set and script bit 0 is clear, the boss is forced into the wait state on every tick and cannot start any action.
- **Shadow timer alone does not start a slip** (flag-clear branch only; in the shipped scripted regime the timer starts the slip): with script control clear, the shadow-slip countdown triggers nothing. A slip then starts only from the request latch (set while the boss's navmesh blocked flag is on, that is within 120 ticks (2 s) after an arena-edge push or an off-mesh restore of the boss; SPEC-0056) or from one of the two 50 % rolls (dodge-melee and sword-blocked hit).
- **Melee dodge is almost always chosen**: the dodge-melee chance is 100 at every pattern. Because the roll is 0–100 inclusive, the boss dodges an in-range swing with 100/101 chance. Only a roll of exactly 100 falls through to the jump-thrust check, which then needs a fresh roll below 15 (or 10 at patterns 3 and 4) (High). Jump-thrust is therefore rare in practice.
- **Lunge without contact**: a thrust lunge, sword swing or jump thrust whose weapon does not touch the hero on a tick sends nothing that tick and keeps its once-per-attack flag clear. The test is repeated on every tick of the window (event bit 0x1), so a hit can land on the last window tick; outside the window nothing can hit (High).
- **Shadow thrust-lunge is unreachable** (High): no shadow-attack chooser and no other state writes the shadow thrust-lunge state (state 17), so it never runs in play. Every store to the controller's state word writes a fixed value other than 17, and the only variable store copies a saved return state, whose stored values are only -1, 16, 18, 20, 21, 22, 23 and 24. Its 5-damage lunge exists only as unused behavior. Keep it unreachable; do not add it to a chooser.
- **Back-stab only from the shadows**: the back-stab state is entered only as the shadow attack chosen after a shadow slip (patterns 2 to 4). There is no normal back-stab.
- **Back-stab cannot be blocked**: it has no weapon volume and no block test; a guarding hero within 6.0 still loses 5 HP. A hero at exactly 6.0 is not hit (the test is strict).
- **Boss melee ignores armour**: a hero with full armour and an armour Mote still loses the full 25, 20, 15 or 5. Only the daggers are reduced (pool rule).
- **Block cancels the whole hit**: a blocked lunge, swing or jump thrust calls no take-hit, so the hero loses no HP and gets no hit reaction; the boss plays its sword-blocked state instead.
- **Light arc hits once**: an arc deals its 20 at most once, even when several arc balls touch the hero.
- **Shadow dagger burst**: the shadow dagger attempts 12 launches (4 per release frame); with the pool limit of 10 live projectiles, launches beyond the limit are lost while the others fly.
- **Dagger reappear uses its minimum twice**: the dagger shadow attack passes the dagger-reappear minimum (15) as both the minimum and the maximum of its landing range. The maximum (30) is not used (Medium). Reproduce as written.
- **Pattern 2 mirror bucket and pattern 3 rainbow bucket are unused**: the pattern-2 chooser has no mirror bucket, and the pattern-3 chooser has no rainbow bucket, so their stored chances (10 and 20) are never read by the chooser (Medium).
- **Heal can start above its target**: pattern 3 can start a heal when HP is between 33.3 % and 37.5 % of max (the target is max/3, about 33.3 %). Pattern 4 never starts a heal above its target. At pattern 3 the heal amount trunc(target − HP) is then zero or negative. A negative heal simply lowers HP toward the target: the heal caps at max HP but has no lower clamp and makes no death call (4.9; High). Reproduce this.
- **Heal interval**: the countdown starts at 5, and one heal happens on the tick after the countdown reaches 0. The interval is 6 ticks, not 5 (High).
- **Back-up is nearly certain**: because the roll is 0–100 inclusive, the back-up happens with chance 100/101 (about 99.0 %).
- **Orbit band**: between distances 15 and 20 the boss never backs up; backing up needs d < 15.
- **Shadow timer for an out-of-range pattern**: the shadow-slip timer function gives a range of 0..1 s (0 or 60 ticks) for pattern values other than 1..4. Patterns 1..4 only are reached.
- **Taunt and left/right range are unused**: the strafe taunt chance and left/right range are stored but not used, so the boss never taunts (Medium).
- **Light-ball damage is unused**: light balls do not use the stored 25 damage in any described behavior.
- **Invisibility is unreachable**: the invisible flag is set only by the start-invisible state, which is entered only while the flag is already set (4.11). Do not implement the invisible attack or the start-invisible state as reachable.
- **Dagger windup and the pending flag**: the dagger throw's windup sets the evil-laugh pending flag only while the hero is in hit-reaction animation 32 (4.3). The evil-laugh roll can therefore follow a dagger throw, but not a throw at a hero who is not reeling.
- **Drift past the threshold**: a point that drifts slowly is never moving (4.14), so a settle can complete while the point has moved a long way from where it started. The settle test does not measure total displacement.
- **A jump delays the settle**: a point that jumps more than 0.02 units holds the settle back until its saved copy has closed to within 0.02 units, which takes about 25 runs for a one-unit jump (Medium; follows from the rule in 4.14).
- **Silent death and evil laugh** (High): the die cue (cue 3 of block 386) is named by no animation, so the boss dies without a death sound, and the evil-laugh animation 23 has no sound word. Reproduce this; do not add the cues.
- **Hidden clones are silent** (High): a clone in base state 42 runs no animation-sound step; it plays cues only after it is woken.
- **Smoothing snaps when close**: a moving point whose saved copy comes within 0.02 units of its position counts as stationary on the next run, and its saved copy is then set exactly to the position (4.14). This follows from the same moving test (Medium).

## Test vectors

### Fight timeline (shipped regime; boss released at tick 0)

Assumptions: scripted regime, pattern 1, hero standing still and never attacking, tick 0 is the first tick after the boss leaves the wait state with its timers at the creation draws (any ticks already spent in the wait state shorten them). Draws: attack timer N_a (120, 180 or 240), slip timer N_s (480 to 900), back-up cooldown 60. A timer set to N reads 0 on tick N - 1 when it is a creation value.

Vector A: hero 30 units in front.
- Ticks 0 to about 118: state 1F approach (clip 34); the boss closes about 10 units (5.15 units per second on average; slightly slower in the first cycle). No attack is possible before tick N_a - 1.
- About tick 118 to N_a - 1: the distance is about 20; the boss alternates between strafe (coin side) and approach as the distance crosses 20 (strafing widens the distance by about 0.0003 per tick). No attacks.
- From tick N_a - 1: on every tick with 20 < d < 40 a roll under 30 (30 of 101) throws. Expected first throw within 1 to 5 ticks of that. Throw at tick T0 (random): daggers launch on T0 + 24, T0 + 48, T0 + 72 at speed 1.5 per tick (about 14 ticks to cross 21 units); the boss is back to decision-ready at T0 + 97; the attack timer is re-armed on T0 + 96 to 120, 180 or 240 (random).
- Tick N_s - 1 (earliest 479, latest 899), or the first decision-ready tick after it: shadow slip starting at S: vanish S to S + 49, chooser pass S + 50, reappear request S + 51, pattern 1 always just appear at 15 to 40 units from the hero, reappear done S + 128, just-appear handler S + 129: with probability 75 of 101 it goes to idle on that tick; with 26 of 101 animation 22 plays and it goes idle on S + 261. The slip timer is re-armed (8 to 15 s) at the exit. The boss cannot be hit from S to the end of the reappear.

Vector B: hero 12 units in front, N_a = 120.
- Ticks 0 to 58: state 0 idle (d < 15 and the cooldown is not zero, so plain idle stays). Tick 59: cooldown 0, set to 60, back-up on a roll under 100 (100 of 101): state 1E from tick 59, ends tick 98 (39 ticks), moving about 0.2797 per tick, so d is about 22.9 at tick 98. Tick 99: d > 20, approach (1F).
- From tick 119 (N_a - 1) the attack timer is 0. While d is still above 20 (until about tick 133) every tick has a 30 of 101 dagger roll, so a dagger throw starts almost certainly before tick 133 (the chance of no throw in 14 ticks is below 1 %). If it throws at T0, the throw lasts T0 to T0 + 96.
- A hero staying inside 15 units after tick 119 with the attack timer at 0 is attacked on the first decision-ready tick: d <= 10: roll 60 or more lunge, else swing; 10 < d < 15 lunge. Swing: requested tick T, weapon test ticks T + 24 to T + 39, 20 damage once if it touches and is not blocked, boss free at T + 78. Lunge: window T + 24 to T + 63, 25 damage, free at T + 90.

Vector C: damage factor. Hero melee damage 15 each hit, no band change: hit at tick 0 of the fight: 15; at tick 900: 30; at tick 1800: 45; at tick 3600: 75; at tick 5400: 90. A band change at tick 2000 resets the factor to 1.0 from tick 2000, so a hit at tick 2900 (counter 1800) takes truncate(15 x 3.0) = 45. A hit that starts a flinch at HP 20 with damage 75 sets HP to 0; the death state follows when the flinch ends (57 or 73 ticks later).

Vector D: slip schedule by pattern. The slip timer draws are 8 to 15 s (pattern 1), 4 to 8, 3 to 6 and 2 to 5 s; the first slip of pattern 4 can come 120 ticks after the previous exit and the last 300 ticks after.

HP bands (max HP 800, boss decision-ready, pattern starts at 1):

- HP 600 (f = 0.75): stays pattern 1, because 0.75 is not below 0.75.
- HP 599 (f = 0.74875): pattern 2, superior-sneer stance, event 1.
- HP 399 (f = 0.49875), now at pattern 2: pattern 3, angry stance, event 3.
- HP 150 (f = 0.1875), now at pattern 1 (fresh start): pattern 4 directly, injured stance, event 5 only.
- HP 400 (f = 0.5), now at pattern 4: stays pattern 4.

Hits and flinches (script control clear, hit counter 0):

- One hero missile hits the boss while it is idle: the slot is removed, the counter goes to 2 (1 for the slot, 1 for the flinch) and the flinch starts. Event 0 is not raised (2 is not above 2). A second missile hit after the flinch: counter 4, event 0 is raised every tick from then on.
- Three melee hits in the idle state, each after the previous flinch has ended: counter 1, 2, 3; event 0 is raised from the third.
- A hit during the sword swing, the jump thrust or the back-stab: nothing happens (not hittable).
- A melee hit during healing phase 1: healing moves to phase 2 (animation 34), no flinch, counter unchanged. A missile hit there: phase 2, counter + 1.
- A melee hit during the light-arc loop: the arc ends, the boss flinches, counter unchanged.
- Dagger windup (animation 31 not finished) while the hero plays animation 32: the pending flag is set; on the next HP step a roll below 75 starts the evil laugh. The same windup while the hero plays any other animation: no flag.

Timers (seconds to ticks):

- Shadow-slip timer, pattern 1: random 8 → 480 ticks; random 15 → 900 ticks.
- Attack timer, pattern 2: random 3 → 180 ticks.

Attack choice (decision-ready, attack timer 0, not invisible, not a mirror image):

- d = 30, r = 29 → dagger throw. d = 30, r = 30 → no attack.
- d = 12, r = 99 → thrust lunge (10 < d < 15 always lunges).
- d = 8, r = 59 → sword swing. d = 8, r = 60 → thrust lunge.

Settle test (dormant rule; synthetic; two points, saved copies at the origin, settled flag clear):

- Both points at the origin on a run: both stationary → settled flag set on that run.
- Point 1 at (0.03, 0, 0), point 2 at the origin: point 1 is moving (0.03 > 0.02), so its saved copy moves to 0.15 × 0.03 = 0.0045, to (0.0045, 0, 0); point 2 is stationary; the settled flag is not set.
- Settled flag already set on entry, any positions: the test does not run and the second latch is set to 1.

Drift (dormant rule; synthetic; one point, saved copy at the origin on X, moving +0.015 units on X per run):

- Runs 1 to 10: each difference from the saved copy is 0.015, which is 0.02 or less, so the point is stationary and its saved copy follows it. After run 10 the point is at 0.15 and its saved copy is 0.15. The point has moved 0.15 units and never counted as moving.
- Run 11: the point jumps to 0.18. The difference from the saved copy (0.15) is 0.03, so it is moving; the saved copy moves to 0.15 + 0.15 × 0.03 = 0.1545.
- Run 12: the point is still at 0.18. The difference is 0.0255, so it is moving; the saved copy moves to 0.158325.
- Run 13: the difference is 0.021675, so it is moving; the saved copy moves to about 0.161576.
- Run 14: the difference is 0.018424, so the point is stationary; its saved copy is set exactly to 0.18.

Smoothing (dormant rule; synthetic; the gate flag is set; one point):

- Saved copy (1.0, 0, 0), current (2.0, 0, 0): difference 1.0 > 0.02, so moving. Saved copy becomes 1.0 + 0.15 × 1.0 = (1.15, 0, 0). The value reported under event 0x13 is (1.15, 0, 0).
- Next run, current still (2.0, 0, 0): difference from (1.15, 0, 0) is 0.85, so moving. Saved copy becomes 1.15 + 0.15 × 0.85 = (1.2775, 0, 0).
- Stationary case: saved copy (2.0, 0, 0) and current (2.01, 0, 0): difference 0.01, stationary. Saved copy is set exactly to (2.01, 0, 0).

Melee and dagger details (synthetic):

- Thrust lunge, weapon touching on a tick, hero guarding, dot(hero facing, boss facing) −0.6: sword-blocked state. Same tick with dot −0.4: a hit of 25 damage is sent and the once-per-attack flag is set; later ticks of the same animation send nothing.
- Sword swing, weapon touching, hero not guarding: a hit of 20 damage is sent once.
- Sword swing, weapon touching, hero guarding, dot(hero facing, boss facing) −0.6: the boss enters sword-blocked; no take-hit; hero HP unchanged. Same with dot −0.4: the hero loses 20.
- Thrust lunge lands on a hero with 100 HP, body armour and an armour Mote: HP 75 (no reduction). The same hero hit by a dagger with a pool armour factor of 50 % and no Mote: trunc(10 × 0.5) = 5 taken, HP 95.
- Back-stab at horizontal distance 5.9 while animation 30 plays: 5 damage, once. At 6.0: no hit.
- Windows at S = 2, R = 1: the sword swing's frames 6 to 9 are ticks 24 to 39 after the clip starts; a weapon touch on tick 23 or tick 40 sends nothing.
- Contact never passes during the lunge animation: nothing is sent; the boss returns to idle when the animation ends, and the attack timer is re-armed.
- Dagger throw, distance 30 units: lifetime trunc(40 / 1.5) = 26 ticks; the dagger makes 27 moves of 1.5, 40.5 units. Shadow dagger, distance 30 units: lifetime trunc(40 / 5.0) = 8 ticks; 9 moves of 5.0, 45 units.
- Dagger release: event bit 0x20 is on for the 4 ticks of frame 6 → one dagger throw launch, but four shadow dagger launches. A full dagger throw launches 3 daggers; a full shadow dagger attempts 12.
- Invisible boss ends a lunge with no hit sent: the invisible flag is cleared. Invisible boss ends a lunge after a hit: start-invisible state.

Strafe:

- Locomotion clips: the slot values are (12, 12, 12, -1, -1, 15, 14), so approach plays table entry 12 (clip 34), strafe right entry 14 (clip 35), strafe left entry 15 (clip 36). Back-up plays entry 13 (clip 33): 40 ticks x -0.2797 = -11.19 units. Strafe right over its 44-tick cycle moves 4.72 units sideways.
- Dagger origin: at frame 6 of the throw the event bits are 0x22, so the first set low bit is 2 and the origin is attachment 1; with only 0x20 on, the origin would be attachment 0.

- d = 25 → approach.
- d = 18, state idle → left or right by a 50 % coin.
- d = 12, back-up cooldown 0 → cooldown set to 60, back up on a roll of 0–99 (100 of 101 values); roll 100 → idle.
- d = 12, cooldown 30, state approach → left or right by a coin.

Shadow chooser:

- Pattern 2 (M = 33, D = 33, H = 33): r = 20 → backstab; r = 40 → dagger; r = 70 → light arc; r = 99 → just-appear.
- Pattern 3 (M = 25, D = 25, H = 25, MI = 10, max HP 800): r = 60 with HP 360 (0.45 max) → 0.45 ≥ 0.375 → light arc. r = 60 with HP 260 (0.325 max) and a second roll of 10 → healing. r = 80 → mirror attack (75 ≤ 80 < 85).
- Pattern 4 (M = 4, D = 4, H = 32, MI = 10, RD = 50): r = 3 → backstab; r = 6 → dagger; r = 20 → light arc or healing; r = 45 → mirror attack; r = 99 → rainbow dash; r = 100 → just-appear.
- Light-arc ball paths (synthetic): boss at (0, 10, 0), hero at (0, 10, 30), so d = (0, 0, 1) and l = (−1, 0, 0). Both balls start at (0, 11.5, 0). After 30 loop ticks (each ball moves 0.2357 per tick on each of x and z): A = (−7.071, 11.5, 7.071) and B = (7.071, 11.5, 7.071). The wall then runs along x at z = 7.071 and n = (0, 0, 1). After 200 loop ticks each ball is 66.67 units from the start, at (∓47.14, 11.5, 47.14), and the arc ends. Boss at (0, 10, 0), hero at (30, 10, 0): d = (1, 0, 0), l = (0, 0, 1): A flies toward (0.7071, 0, 0.7071) and B toward (0.7071, 0, −0.7071).
- Melee and strafe words (synthetic): d = 30, r = 29: dagger (20 < 30 < 40 and 29 < 30). d = 20 exactly or d = 40 exactly: no dagger. d = 10, r = 60: thrust lunge; r = 59: sword swing. d = 14.9 uses the lunge with no roll. d = 15: no melee attack. d = 20 enters the strafe rules; d = 20.1 approaches. d = 14.9 with the back-up cooldown at 0: cooldown set, back up on any roll 0 to 99.
- Bucket words (synthetic): pattern 3 clone (acting entity a mirror image): dagger bucket 25 (0x2DACD4), backstab bucket 25 (pattern value, 0x2DACB8), light-arc-or-heal 25, mirror 10; cumulative edges 25, 50, 75, 85. Pattern 2 uses 33, 33, 33: edges 33, 66, 99, so r = 100 gives just-appear.
- Mirror attack placement (synthetic): see the example in 4.10 (boss at (10, 2, 20) facing +Z). Boss facing +X, at the origin: f = (1, 0, 0), axis (0, 0, −1); slots 1 to 4 at (0, 0, −5), (0, 0, −10), (0, 0, 5), (0, 0, 10).
- Light-arc missile wall (synthetic): A = (0, 0, 0), B = (10, 0, 0), so u = (1, 0, 0) and n = (0, 0, 1). A hero missile at P = (5, 1, 2) with step s = (0, 0, −0.5): Q = (5, 1, −3); n · unit(P − A) > 0 and n · unit(Q − B) < 0, so it is removed. The same missile with s = (0, 0, −0.1): Q = (5, 1, 1), not removed this tick. A missile at (5, 1, −2) moving +z: not removed. A missile at (50, 1, 2) with s = (0, 0, −0.5): removed (the line has no ends).

Healing (max HP 900, pattern 3, target 300, HP 150 at the start):

- Each full heal adds 9 (trunc(0.01 × 900)). Sixteen full heals take HP from 150 to 294.
- The 17th heal: 300 < 294 + 9 holds, so it heals trunc(300 − 294) = 6 and ends at 300, then goes to phase 3.
- Total: 17 heal events.

Heal arithmetic (max HP 300, pattern 2, target 150, step 3):

- HP 152: 150 < 152 + 3, so the heal is trunc(150 − 152) = −2: HP 150, phase 3.
- HP 140: 150 < 143 does not hold, so the heal is +3: HP 143.

Rainbow dash:

- Draws 3, 1, 3, 0, 6, 6, 6, 5 give the order 3, 1, 4, 0, 6, 7, 2, 5. All draws 6 give 6, 7, 0, 1, 2, 3, 4, 5.
- Hero standing 5 units from every point (within 8.0 at each): 8 hits, 120 HP.
- Step: 40 / 60 = 0.6667 units per tick from the dash step alone; a point 1.8 units away is reached in 3 ticks by that step (two full steps, then the snap at 0.467 remaining). The clip's root travel adds about 0.33 per tick on top from the second dash tick (G6b; the first tick carries no root travel because of the one-tick lag).

Clones:

- Slot 1 at +5 units, slot 2 at +10, slot 3 at −5, slot 4 at −10 along the lateral direction.

- Missile reaction: a bone record with full length 2.0 and radius 0.5 produces a finite cylinder centered 1.0 unit along posed local Y from its base, with radius 0.5; radius 0 means that bone is skipped. Every positive-radius bone is considered. A path missing all positive-radius bones causes no missile dodge/deflect roll.
- Landing search: if a candidate lies only on mesh 2 while the boss is currently assigned to mesh 0, the candidate is accepted because the query scans all meshes in the region. Ten candidates outside every mesh leave the boss where it is and make the attack just-appear. A back-stab point outside every mesh falls back to shadow dagger.

Shadow-slip request latch:

- The boss, decision-ready and not a mirror image, is pushed back by a navmesh boundary edge in the motion step of tick n and not again. Its blocked flag is set on ticks n + 1 to n + 120, so the latch is set on each of those ticks, and the latch path of the shadow-slip gate (4.4) applies from tick n + 1. A boss that never touches a navmesh edge and is never restored onto the mesh never sets the latch, wherever the hero stands.

### Fight AI second-pass vectors (synthetic)

- Lunge travel. The boss faces a stationary hero 14 units away and requests the lunge on tick 0; the heading is set by the one turn on tick 0. Ignoring the one-tick lag, the distance is about 13.1 at tick 24 (window opens), 10.7 at 28, 7.9 at 32, 5.2 at 40 and 4.2 at 64 (window closes); the boss is free at tick 89 about 3 units from where the hero stood, having carried 11 units forward in all. The same lunge from 5 units ends about 6 units past the hero.
- Sword swing travel. From 8 units: 5.7 at tick 24, 4.3 at 36, 3.8 at the end; the swing keeps turning toward the hero by half the remaining angle each tick.
- Melee reaction. The hero's swing becomes active at tick 40 with the hero 9.0 units away and the boss inside its 60-degree arc, boss idle: a roll of 0 to 99 starts the melee dodge (state 8) on that tick, so the same tick's hit test finds the boss not hittable; it slides back to about 19.2 units by tick 106 (65 ticks) and then a roll under 50 starts a shadow slip. A roll of 100 falls through to a second roll: under 15 gives the jump thrust, else nothing. At 12.0 units, or outside the arc, nothing happens and the swing is never re-tested.
- Missile reaction. A Roberta shot whose 180-tick path crosses a bone capsule, boss at pattern 2, idle: first roll under 40 (40 of 101) dodges; else a fresh roll under 80 (80 of 101) deflects; else the shot continues and hits if it overlaps the body cylinder while the boss is hittable. A clone uses 25 for the first roll. A second shot with the same hit id is not rolled again.
- Mirror landing. Pattern 3, hero standing 20 units from the boss at (-600, 122.5, -520): a roll inside the mirror bucket (75 to 84) puts the boss at (-623.68, 121.88, -542.24) on the choose pass, not near the hero; the clones appear about 74 ticks into clip 42 at 5 and 10 units to each side of that point (perpendicular to the boss's heading then).
- Rainbow dash speed. A target point 20 units away is reached in about 20 ticks (0.667 from the dash step plus 0.33 from clip 28 per tick), then 1 tick for the point change; eight points of 20 units cost about 170 ticks, plus the end clip (61 ticks).
- Clone timers. A clone woken on tick W draws its attack timer at once, 120, 180 or 240 ticks, whatever the boss's band; its clip 42 ends about 74 ticks after W, so its first attack is on tick W + 120 at the earliest.
- Blocked lunge. The lunge's weapon touches a guarding, facing hero on tick 24: state E from tick 25, 45 ticks, free on tick 70; the attack timer is still 0, so a hero within 15 units is attacked again on the first idle tick.
- Same-tick stance. The boss is decision-ready at HP 599 (its flinch has ended) and a hero shot's path crosses it on the same tick: the band step sets pattern 2, state 3 and event 1, then the reaction step may replace state 3 with a dodge or deflect; the stance wait is skipped, and the level script's stage still runs because event 1 was raised.

## Open questions

- Arena release: resolved. The opening scene's last call sets level state 0 (G9); the rewrite sets it at that point, not when camera track 25 finishes. Moot: whether the outline panels stop the hero at the two passage mouths (the sealing walls stop the hero there during the fights). Default if unresolved (rewrite choice): the parked partner is not drawn (needs a runtime check).
- Default if unresolved (rewrite choice): copy story flag 258 at each creation and run the branch it selects (the scripted branch, flag set, on the normal route; see 4.3 and item 40); in the scripted branch draw the AI timers at creation and subtract the ticks elapsed since creation from each at the release, not below 0 (the release is the end of the whole opening scene, 469 ticks plus the boss's animation 2 plus the dialogue time after the scene's first tick; the boss is created on the scene's first tick or one tick after it; item 43).
- Default if unresolved (rewrite choice): the words listed as unused in "Word order of the two groups" (including 0x2DACA0 and 0x2DACA4) may have a reader in code not reachable from the described states. Store them, and apply none of them.
- Resolved (R-0067, side of root x): in modes 5 and 6 the shared steering turns the boss's move direction toward the hero (SPEC-0029, Behavior 2: with d the direction to the side target, mode 5 steers to up x d and mode 6 to d x up, which is the direction to the hero), and the root step of a clip is applied along the record's side axis, up x forward (SPEC-0040, Behavior 14C), a positive root x going along it. The table entries of these clips carry no mirror flag. With the boss facing +Z that axis is +X, which is the boss's left (SPEC-0004: the right side is -X). So the mode 5 clip, whose sideways root is positive, moves the boss to its left of the line to the hero, and the mode 6 clip, whose sideways root is negative, to its right. This is the opposite of the side point that the target names; that point only feeds the steering (High for the rule; Medium-High for what shows on screen). Test vector: boss at (0, 0, 0) and hero at (0, 0, 10): mode 5 moves toward +X and mode 6 toward -X, while the mode 5 target point is (-3, 0, 0) and the mode 6 target point is (3, 0, 0). Gelert Assassin: the strafe right state (mode 5, positive sideways root) moves the boss to its left of the line to the hero and the strafe left state (mode 6, negative) to its right; the strafe-right target point, 3 units to the boss's right, only feeds the steering.

Numbers of removed questions are not reused.

1. **Runtime overrides.** Resolved (High that no disc file or level module overrides them; Medium-High for the PC link): no. The tuning words keep the values at their data addresses for the whole session. No disc file holds their names or values, no loader fills them, and no level module can name them. The only code that can write a registered word at run time is the registry's link to an external PC, and nothing in the shipped game starts that link. Test vector: the attack timer ranges of patterns 1 to 4 are 2 to 4, 2 to 3, 1 to 3 and 1 to 2 seconds in every fight of every session; there is no second set.
3. **Elapsed-time index and the healing countdown copy.** Resolved: the derived value is the boss damage-taken factor, a float 1 + 4 x counter / 7200 that multiplies each hero hit (4.6, 4.13; High). The counter in the healing-countdown slot is set at creation and never read by boss logic.
4. **Damage values.** Resolved: each value is the damage itself (4.12; High).
5. **Healing length and factor.** Healing length and healing factor are not used by any described behavior. The healing timer pair is loaded only at creation (Layout).
6. **Negative heal.** Resolved: a heal caps HP at max HP with no lower clamp and no death call, so a negative amount lowers HP toward the target (4.9; High).
7. **Bone cylinder placement.** Resolved: every bone entry in the active pose with positive radius is eligible for the incoming missile path test; there is no fixed named-bone subset. A nonpositive radius skips that bone. The bone record's length is its full cylinder length and its radius is the cylinder radius; the cylinder follows posed local Y and is centered halfway along its length from the posed base (4.13, SPEC-0029 Behavior 26). Confidence is High for eligibility and field meanings, Medium-High for applying the shared placement to this swept-path caller. The separate incoming-hit check uses the boss body cylinder. The missile path and missile hit tests and the 10-unit deflect distance are also in 4.13 (High).
8. **Facing and block tests.** Resolved: the block test is dot(hero facing, boss facing) < −0.5 (4.4b), and the melee reaction's facing and reach tests are in 4.13 (High).
9. **Clone mirror attack.** Resolved: the mirror handler sets the current AI instance's shadow-mode latch, and its chooser reads that same instance-local latch. It does not set a shared boss/group flag. After wake activation, a clone is assigned controller state 0x16 (decimal 22), the mirror-attack handler (4.10). The shadow-mode latch remains instance-local.
10. **Rainbow target points.** Resolved: the level locators with ids 200 to 207, visited in a permutation built from draws mod 7 (4.8, Rainbow dash; High).
11. **Flinch conditions.** Resolved: the boss's hit check runs only in the hittable states listed in 4.13; a hit there flinches the boss except in healing (moves to phase 2) and the sword-blocked shadow-slip roll, and a hit during the light arc loop removes the arc and flinches (High; the removal is confirmed in R-0171: the hit reaction calls the arc object's free routine before it enters the flinch).
12. **Invisibility reappear pair.** Resolved: the words 0x2DAD1C and 0x2DAD20 have no reader, so both are unused (High).
13. **Invisible bucket, clone backstab, just-appear mirror and other unused bucket words.** Resolved: 0x2DACD8 to 0x2DACE0, 0x2DACC0 and 0x2DAD08 are read only by getters that have no caller; the pattern 1 words, 0x2DACF4 and 0x2DAD00 are not read by the chooser of their pattern. All are unused (High). The full list is in "Word order of the two groups".
14. **Melee contact test.** Resolved: weapon 1 of monster row 4 (radius 6.0, full length 1.75), tested against the hero's bone capsules while event bit 0x1 is on (4.4b; High).
15. **Shadow thrust-lunge timers.** Resolved (High): the two re-arms sit in the branch taken on the tick the lunge animation (28) reaches 0.99 progress, the same branch that leaves the state. That branch sets idle (or, when the invisible flag is set and a hit was already sent, the start-invisible state; when the invisible flag is set and no hit was sent, it clears the flag), and then, in every one of those cases, draws the shadow-slip countdown first and the attack countdown second, each as trunc(rand(min, max) x 60) from the current pattern's ranges (4.11). A lunge that is blocked leaves through the sword-blocked state instead and does not reach this branch. Test vector: pattern 1, attack range 2 to 4 s: when the shadow thrust-lunge ends the attack countdown becomes one of 120, 180 or 240 (a whole number of seconds from 2 to 4, times 60), and the shadow-slip countdown is drawn from the pattern's slip range on the same tick, before it.
16. **Thrust-lunge and sword-swing timers.** Resolved (High): both handlers re-arm only the attack countdown (never the shadow-slip countdown), in the branch taken on the tick their animation (28 for the lunge, 26 for the swing) reaches 0.99 progress. The branch first sets idle (or the invisible-flag cases of item 15), then draws the attack countdown from the current pattern's range. A tick on which the hit is blocked leaves through the sword-blocked state instead. Test vector: a sword swing whose animation reaches 0.99 on tick t leaves the attack countdown at its fresh draw on tick t (for pattern 1, 120 to 240), and the shadow-slip countdown is unchanged.
17. **Evil-laugh pending flag.** Resolved: the second test checks the hero, not the boss: the flag is set while the windup (31) plays unfinished and the hero's current animation is 32, a hero hit reaction (4.3; High for the test, Medium-High for the meaning of 32).
18. **Steer helper.** The turn helper in the lunge, sword swing and dagger throw is called with weights 1.0 (at the start of the thrust lunge), 0.5 (every tick of the sword swing, including before its animation, and every windup tick of the dagger throw). Its direction is the horizontal unit vector from the boss to the hero. The weights are fixed constants. Resolved (High for the identity of the helper, Medium-High for the call timing): the helper is the shared aim rule of SPEC-0040 (Behavior 8, Aim rule), the same character method that the Faerie Sisters' controllers and the class-37 recovery routine call to turn toward a target. With stored facing c, target direction t and weight w, the new facing is normalise(w t + (1 - w) c); a weight of 1.0 snaps the facing to t and 0.5 halves the remaining angle on each call. It changes no animation rate. Test vector: boss facing (1, 0, 0), hero to its side so that t = (0, 0, 1). Weight 0.5: facing (0.7071, 0, 0.7071) after one tick and (0.3827, 0, 0.9239) after two. Weight 1.0: facing (0, 0, 1) at once.
19. **Boss-to-hero helper.** The helper returns the horizontal unit direction from the boss to the hero (y forced to zero) and the horizontal distance. Every range test in this spec reads that distance (High).
20. **Release-event latch.** Resolved: bit 0x20 is on for 4 ticks at each of frames 6, 12 and 18, so a dagger throw launches 3 daggers and a shadow dagger attempts 12 launches (4.4b, 4.8; High).
21. **Launch velocity units.** Resolved: units per tick; 10 and 3.0 are the damage, 1.25 the radius, and the lifetime is trunc((distance + 10) / speed) ticks (4.4b; High).
22. **Which states use the settle test and the smoothing.** None in the shipped build. The dispatcher that selects the two-point settle has no caller, the four-point variant and the second move routine have no caller, and no code pointer in the boot program refers to them (High for the call graph; Medium overall). A host export that could reach the rule has not been identified (Low).
23. **Meaning of the second latch.** Resolved (Medium): its reader is the dormant restart step of the shared settle rule (SPEC-0040, rule 3), which nothing in the shipped build reaches, so the latch never changes play. Its setting and clearing are in 4.14.
24. **Clearing the settled flag.** Resolved: it is cleared only by the dormant move-start routine, so no clear is reached in the shipped build (Medium-High).
25. **Gate of the smoothing.** Resolved: the gate is the settled flag itself, and nothing in the shipped build reaches it (4.14; High).
26. **Blend direction.** When a point has moved more than 0.02 on some axis, its saved copy becomes 85 percent of itself plus 15 percent of the current position; otherwise the saved copy takes the current position (4.14; High for the constant and the direction; Medium-High for the order of the vector blend's weights).
27. **Event 0x13.** The handler stores a four-float vector in the object's point table. The table is indexed in 32-byte entries, and the event number selects the entry. Index 0x13 is the first tracked point and index 0x14 the second (High). The handler's argument list is not fully established.
28. **Which tracked points.** The two points of the default rule are tracked-point slots 0x13 and 0x14 of the object's point table (SPEC-0040). Which part of the boss's model each one follows is not identified. The "current" input that the rule reads is written outside the settle routine (Open). Because nothing in the shipped build reaches the rule (4.14), questions 26 to 28 (blend weight order, the event handler's argument list and the followed model part) cannot change anything a player sees. Default if unresolved (rewrite choice): do not implement the tracked-point rule or its event; if it is implemented, use saved = 0.85 x saved + 0.15 x current, which equals the 4.14 form, and leave both points at the boss root.
29. **Float constants.** The 0.02 thresholds and the 0.15 blend are fixed constants of the two-point routine (Medium-High). The same float patterns also occur as aligned words in data at places not linked to these routines, so a data read is not ruled out (Low).
30. **Four-point variant.** The shared family has a four-point routine (tracked-point slots 8, 9, 0x10 and 0x11, SPEC-0040), but it has no 0.02 test and no caller in the boot program. No Gelert state reaches it (High).
31. **Rainbow-dash latch.** Resolved: cleared at the dash start and at every target point, so up to eight hits per dash (4.8; High).
32. **Light-arc third object.** Resolved: the third test is not a cylinder against the hero; it is the arc's missile wall, a side test of each active hero missile (now and 10 ticks ahead) against the vertical line through the arc's two end objects, which removes a crossing missile (4.8, Light arc; High for the rule, Medium-High for the side sign).
33. Resolved: **first tick of a window.** The clip starts on the request tick (tick 0) at cursor 0 and is advanced to 0.25 in that same tick; the fade-in does not move the cursor. The window test reads the frame before each tick's advance, so frame f is read on ticks 4f to 4f + 3, except frame 0, which is read on ticks 1 to 3 (4.4b; Medium-High).
34. **Weapon attachment.** (The dagger's hand point is attachment 1; see the dagger throw.) Resolved (High for the table entries; Medium-High for the body-part names): the Gelert Assassin's attach-point table (SPEC-0004, SPEC-0005) is the same in all three of its models (regions 741, 755 and 1258). Attachment 0, the anchor of weapon 1, follows bone 21 with offset (-0.02, -0.017, 0) and no rotation. Bone 21 is the end of the arm on the -x side (model-space rest position (-2.49, 3.45, 0.02)), the sword hand. Attachment 1, the dagger's hand point, follows bone 11 with offset (0.284, -0.14, 0): the end of the mirrored arm on the +x side (rest position (2.21, 3.59, 0.02)). Attachments 2 to 4 follow the root bone with a zero offset, so weapons 3 to 5 would sit at the model origin (the boss's weapon test uses weapon 1 only). The clean side reads the bone numbers and offsets from the model on the disc. Test vector: with the boss unrotated at the origin in its rest pose, attachment 0's origin is (-2.512, 3.437, 0.025) and attachment 1's origin is (2.492, 3.454, 0.025).
35. **Row 5.** Monster row 5 (weapon 1 radius 5.5, length 4.0, no class) is used by regions 741 and 1258. Resolved: neither holds the boss fight. Row 5 is the Gelert Assassin's second variant (variant 1, 200 HP, no boss class, so no boss controller runs), placed as one ordinary enemy record in each of those two regions (levels loaded by modules 735 and 1257); the boss fight is variant 0 on row 4 in regions 755 and 985 only (High). The Gelert records of both regions carry no sound words.
36. **Sound step gates and the flinch point.** The boss is taken to be a world character like every character spawned from a level's character records, so the world-character sound rule applies (Medium-High). Resolved: the two further gates of the animation-sound step never block this boss. The "movement ready" word of its motion block is set at binding and cleared only by the hero's own movement states (Medium-High); the per-layer mute word of its animation controller is never set (High). Every sound in the animation list plays as listed. The exact point of the flinch hit flash is Medium.
37. **Dagger trail field.** Resolved: the word is not an effect id. It is entry 1 of the boss's table of 11 extra model parts: while a model set loads, each child of its mesh container whose code is 10 to 20 sets entry (code − 10) of an 11-word table to that child's data, and every other entry stays 0. Both dagger throws copy entry 1 into the projectile's trail word. The projectile pool's draw pass, once per frame for each live slot whose visual kind is below 5 (the daggers use visual kind 0) and whose trail word is non-zero, submits that part at the slot's position with the slot's frame (forward axis and the world up vector), so a dagger is drawn as model part 1 of the boss's set (the mesh child with code 11) and no particle effect starts. When the set has no such child, entry 1 is 0 and nothing is drawn for the dagger (High for the identity of the word and the draw gate; Medium for the look of the drawn part). On the disc, the fight region of the boss (the one with the cue table above) holds one model set whose mesh container has three children: the main body mesh with code 1 (about 361,000 bytes), a part with code 10 (7,088 bytes) and a part with code 11 (3,840 bytes, a single small mesh tree). So entry 1 of the table is the code-11 part, and entry 0 is the code-10 part, which nothing in the boss's code requests. The part is shown only while a dagger slot is live: the slot's draw each frame submits it at the slot position, with the slot's forward axis and the world up vector, and it disappears when the slot ends, with nothing to hide separately (Medium-High); the boss's body is drawn from the main mesh, not from this part. Test vectors: a set with a child of code 11: the dagger slot's trail word is that part and it is drawn each frame until the slot ends. A set with no child of code 11: the word is 0 and the pool draws nothing for the slot, although the dagger still flies and hits.
38. **Empty clip effect entries.** Resolved: the entries of animations 18, 19, 26 and 27 (effect id -1, attach slots 0 and 0) start no particle effect, but each is a swing-trail entry: its trail flag byte is 0 (SPEC-0034, Clip effect list entry and Swing trail). While each of these animations plays, a ribbon is drawn between point 1 and point 2: point 1 is the origin of attach slot 0 (the sword hand), and point 2 is the point (0, 0, 5.5) of the same slot's frame, so the ribbon runs 5.5 units along the slot's forward axis (the blade). It runs over the whole clip (start 0.0, end 1.0). Its colour record is chosen by the attack slot's alignment while an attack slot is selected, otherwise by the monster row's alignment (SPEC-0034, Swing trail). The Gelert is a region character, so the clip walk that starts the trail does run for it (High for the entries, the flag and the geometry; Medium-High for the drawn result). Test vector: animation 26 (sword swing) while the boss is in its sword-swing state: one trail starts on the first pass with edge points (slot 0 origin) and (slot 0 frame point (0, 0, 5.5)) and takes one of the 12 trail slots; if all 12 are taken nothing starts.
- 39. **R-0098 streaming and encounter flow.** Resolved (the chain and the order; the record-to-slot mapping stays conditional as stated): with progress flag 281 set, pack 985 moves the hero to locator 9 in its world-arrival callback, admits Gelert records 0–4, and moves the hero to locator 9 again in its stage-zero intro before starting instance 5 at stage 0. The boss's block slot is 0 and slots 1–4 are clone roles. Automatic character streaming selects the nearest eligible placement record, then binds it to the first free AI slot; placement ordinal and runtime slot are separate orderings. Under the normal automatic path, a fresh character block and no earlier bind produce ordinal 0 in slot 0: the five placements have identical positions, the ascending scan retains the first on a distance tie, and slot allocation starts at the first empty slot. The stream manager then repeatedly fills vacancies toward a six-character target until it reaches that target or no eligible record remains. With only the five Gelert records eligible and no competing bind, ordinals 0–4 consequently fill slots 0–4 in order, matching the primary and clone roles. A prior bind, a different active set, or another allocator caller can change this conditional mapping. The level thread translates boss states 1, 3 and 5 into the next three staged cutscenes, then state 7 into the finale, clone/boss deactivation and instance 5 stage 1. Trigger 4 completes the glade exit and return; the glade's exit latch is set by any contact with trigger 4, but the sealing walls stand until the finale's collision change, so the hero cannot touch it during the fight, and a contact before the finale would only bring the exit forward to the tick after the finale (Medium). Resolved for scheduler order: the first eligible range-stream pass after the transition precedes the activate-world callback body, so it uses the pre-callback hero pose; later passes see the move to locator 9. The reappear picker writes an ungrounded candidate at the hero's current y and accepts it by a boolean test against any region mesh; the Gelert stands on mesh 0 during the fight (4.7). Shared ground probing and physics may later settle it at a local height, but neither the accepted mesh identity nor a fixed final height is available from the picker. Exact later landing mesh and final position depend on the shared ground probe at run time. Default if unresolved (rewrite choice): drop the reappear point to the ground height at that x/z on mesh 0 (needs a runtime check). Confidence: High for the stream fill rule, equal positions, ordinal tie-break, first-free-slot rule and the split between candidate placement and later ground handling; Medium-High for applying the ordinary fresh-block path to the encounter and the later grounding outcome.
40. **Script-control flag in the shipped route.** Resolved (Medium-High): the flag copied at creation is story flag 258, set at the Brightvale library event before the glade fight, so the fight is the scripted branch; the opening scene releases the boss by setting level state 0 with its last call (G1, G9, 4.3). Release writer confirmed (High): the module has exactly one call that sets level state 0, at the end of the opening scene; the earlier statement that no such call exists was wrong.
41. **Damage the boss takes.** Resolved (High for the structure): damage is truncate(hero hit x damage factor), applied only on a flinching hit; the factor is the elapsed-time value of 4.6 (4.13, G7).
42. **Navigation mesh of the boss.** Resolved (Medium-High): the boss and clones are on mesh 0 (own start-up assignment plus the level script's helper), so mesh confinement, the request latch and the strafe-point test are live (4.7, G2, G6).
43. **Opening-scene timing.** Resolved (Medium-High): the boss timers count down from creation, including in the wait state. The release (level state 0) is set by the scene's last call, S + 469 + A + D ticks after the scene's first tick (G2, G9), so the attack timer is always spent and the slip timer almost always. Creation: the pass that binds the five Gelert records is the first pass after the one in which the arrival callback moved the hero (the first pass binds nothing, because the hero is still on the entry record, far from every record). That pass runs its character streaming before its script step, so the boss exists, and its timers start, on the scene's first tick, or one tick after it if the level thread already found the arrival bit set in the earlier pass. The elapsed ticks at the release are therefore 469 + A + D, or one fewer in that second case (a difference that matters only to a slip timer drawn within one tick of the total). With A the length of the boss's animation 2 in ticks and D the time the six dialogue boxes stay open, the attack timer (at most 240) is always spent, and the slip timer (at most 900) is unspent only when 469 + A + D falls short of the drawn value, that is, only when the player closes all six boxes within a few seconds. Default: the boss is created on the scene's first tick.
44. **Home point.** Resolved (Medium-High): the mirror attack and the rainbow dash copy the boss's home point (the start position stored with the placement record) into the landing slot, with no search; the value for the five Gelert records is the record position (G2). The scene's moves to locators change the live position but do not rewrite the stored start pose, so the home point remains (-623.68, 121.88, -542.24) for the boss and its four clones.
45. **Fight AI second pass.** Resolved: root travel of every clip applies in every state (G3, G6b); the rainbow dash covers about 1.0 unit per tick (4.8); the melee reaction fires within 12 units and 60 degrees (4.13); clones are pattern-1 fighters (G14); the landing fallback is the boss's own position (4.7); the sword-blocked state, the shadow dagger, the light arc, the heal, the mirror attack, the rainbow dash and just appear do not re-arm the attack timer (G12). Resolved (High): a melee contact queries hero damage with the attack-power argument fixed at 1.0, supplied by the boss's own hit step. The power-step multiplier of SPEC-0036 (the attack-power table value for Tor's equipped Mote, 0.5 to 2.0 for Motes 160 to 173) is that argument, so it is never applied to a hit on this boss, whichever Mote Tor has equipped; the step factor of the power-step sub-state (1.0) is a separate term and also stays in the product with the weapon, Mote and status terms. The Gelert's separate elapsed-time factor still scales the queried damage before truncation is applied.
46. Resolved (R-0171): **first binding in pack 985 and clone wake order.** On the first streaming pass after the hero is placed at locator 9 the filter's admitted records bind nearest first with the lower ordinal winning ties: Gelert 0, 1, 2, 3, 4 into slots 0 to 4, then Illusen's ordinal 6 into slot 5 (4.10; High; Medium-High that no earlier bind exists). The four clones wake together on one tick, visited in slot order 1 to 4 (4.10; High).
47. Resolved (R-0171): **a hit during the light arc.** It frees the arc's balls and flinches the boss (High).
48. Resolved (R-0171): **the strafe coin.** The draw is an unbiased low bit; 0 gives state 1C (mode 5, clip 35, positive sideways root), 1 gives state 1D (mode 6, clip 36). The state words name the move target's side; the clip root moves the boss to the opposite side. Nothing in the original favours one side (G5, item 4; High for the mapping, Medium for what shows on screen). Default if unresolved (rewrite choice): draw a fair coin on entering a strafe and keep clip 35 for coin 0.

