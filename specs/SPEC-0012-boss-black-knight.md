# SPEC-0012: Boss behavior: Black Knight

- Answers: proactive, none requested (boss AI); proactive (fight AI build guide); R-0067 (the locomotion slot values, clips and root travel of modes 0, 5 and 6), R-0097
- Status: Draft
- Disc build: SCUS-97367 v1.00 (same fingerprint as SPEC-0001)
- Confidence: per section. Fight AI build guide: action-to-clip mapping, clip lengths, gate ticks, step order and hit-window ticks High (they follow from the clip data and the 0.25 key-per-tick speed); weak-point gates High for the predicates and thresholds, Medium for the side pairing of the under-arms; orbit side rule High; deflect stance flow High; flying-sword timeline High from the end of the launch on, Medium for the launch point and Low for the launch length; approach tick count and start placement Medium. Tuning values, the state list and the animation numbers: High. Weak-point maxima (50, 50, 50, 50, 50, 100) and the boss total (350): High; per-fight handler record and its initial state, phase, speed multiplier, counter flag, weak-point values, charge references, accumulators and latches: High. The health display (maximum 350, current HP from the quirk sum, every tick): High. Tick order, with the defeat test before the attack selectors and the state handler: High. Stun windows of 600 ticks: High. Ticks per second (60): High. Unused tunables: High. Phase rules: High that the phase changes happen as written, Medium that the phase reset is intended. Enraged sub-step machines: High for transitions, Medium for the wording of their purpose. The charge run exit (edge-contact flag gives the knee stun, body contact alone gives the impact): High for the rule; Medium-High that the walk-mesh boundary is the arena wall; Medium for the edge-wall geometry. Charge direction test: High for the arithmetic and the x/z axis mapping. Flying sword: High for its gate, sub-state list, held sub-state 5 and reset at the next spin-up, and return destination record and step rule; High for the attachment represented by the return point (attachment point 0); Medium for target-height point attribution; High for clearing the sword-hit latch at count-incrementing full turns; Medium-High for the shared values and orbit-to-homing rule; High for the target-height probe formula; Low for the resulting world-space path; High for the side scalar scaling a normalized pose-derived orbit offset. Parry and deflect: High for hero-variant dispatch, predicates, slot path and roll arithmetic; Medium for the stance/animation interpretation. Idle movement: Medium. Weak-point tests of the enraged attacks: High for the selector-to-pose-frame lookup, sizes, threshold, reaction, hit effect (effect 305), latch, health change, and hero body-cylinder length query (weapon reach plus 2.0); High for target-cylinder construction; Medium for the part-to-selector pairing; Low for anatomical labels. Flying sword world-volume pass: High for the entry filter, the class dispatch and the breakable cues (143 to 146); High for the class-3 cylinder radius field location. Sounds (animation cues, the special-victim cue 34): High. Arena flow: High for the post-fight block and one-time result; Medium for earlier order and gating. Melee weapons, windows, damage (unreduced by hero defences), the swing block and the knockback: High; the flying sword's frame, its bone table, attachment points and model axis: High (cylinder centre and half-length rule High). Several flags: Low or unknown (see Open questions). Hero-side mechanics (the knight's per-tick unhittable and guarding writes, the Blocking Angle write, the block path, the base state 1 write on a passing stun gate, the forced lock): High for the writes and the control flow, Medium-High for the list of Tor swing steps that recoil and for the reading that Tor's under-arm and neck hits need the plunge of a standing jump. High-swing distance rule (the knight within 10.0 of its start point) and the kick's phase reset: High. Level state 10 during the high-swing stun: High for the writes, Medium for what the arena shows.

## Summary

The Black Knight is a melee boss with six weak points (left knee, right knee, left under-arm, right under-arm, back, neck). It runs a per-fight controller with 20 states. Its controller record is created on the active stack frame and initialized before the first tick: state 0, phase 0, speed multiplier 1.0, counter flag 1, weak-point HP 50/50/50/50/50/100, and charge references +20/−20/+60/−60. Each tick it updates its weak-point blend, checks whether it should react (laugh, change phase) and whether it is defeated, checks parry and deflect, may start an attack, chooses an idle move when it is idle, and then runs the handler for its current state. It moves up through three phases as its swings miss, but a landed hit or the end of an enraged attack sets it back to phase 0, so phase 2 must be earned again by missing another angry swing. It becomes vulnerable at a weak point only during the stun window of the attack that matches that point. During its sword spin a flying sword leaves its hand, orbits, homes in on the hero and returns, and the spin goes on only after the sword is back. When all six reach zero or below, it enters the defeated state for good and sets a level state that lets the arena script play the post-fight sequence. Its voices and attack sounds come only from its animation table, and a weak-point hit shows a spark effect (see Sounds and Effects). The clean side can build the full controller from the rules below. Times given in seconds become ticks at 60 ticks per second (SPEC-0007), so a value of 10.0 s is 600 ticks.

## Fight AI build guide

This is the ordered recipe for the whole shipped fight. Each item points to the detailed section that has the rule, its confidence and its edge cases. A tick is 1/60 s (SPEC-0007). "T" is the tick on which a state is entered; the handler of the new state runs on that same tick (item G3, step 7), so the first animation request of a state is on tick T. "Gate m" means the tick T + m on which an animation that was requested on tick T has reached its 0.99 progress gate (item G4).

### G1. Scope of the shipped fight

- States 0 to 19 all occur. The parry (state 6) happens only when the hero is Tor, and the deflect stance (state 7) only when the hero is Roberta. The enraged group (states 11 to 14) is reached only through a missed angry swing (item G11). The sword spin also starts from the balcony check in any phase.
- Not part of the fight: the three unused tunables, the speed multiplier written by en garde, state values 20 and above, action id 25 (a clip no routine requests) and action ids 48 and above (no clip, never requested).

### G2. Initial values (first controller tick)

- State 0 (idle), phase 0, speed multiplier 1.0, counter flag 1, laugh, angry-start, enraged-start, loosen-neck and en-garde flags 0, parry and deflect accumulators 0, shared parry/deflect latch 0, hit latches 0, stance timer 0. The orbit side, orbit timer and stun timer have no start value; each is written before it is read.
- Weak-point HP: left knee 50, right knee 50, left under-arm 50, right under-arm 50, back 50, neck 100. Charge references +20, -20, +60, -60. Every tunable comes from the boot program's data (Tuning values); the clean side reads it from the disc.
- Actor HP: replaced on the first tick by 350 / 350 (health display, G3 step 8). Row-0 monster data supplies model scale, weapon cylinders and radius (Hit windows and damage).
- Placement: the character record in the arena places the knight at (0.0, 0.0, 4.141) with facing direction (-0.003, 0.0, -1.0), that is, facing toward -z (Medium). The hero's start comes from the world. If the knight has no walk mesh at start, it takes the one that contains its position.
- Locomotion slots are filled with (3, 3, 3, -1, -1, 5, 4) before the first tick (Locomotion clips). Play rate is 1.0 and every knight clip has speed byte 2, so a clip advances 0.25 key per tick (4 ticks per key).
- Writes made once before the first tick (High): the Blocking Angle word (+0x10C) of the knight's monster row is set to 1.0 (the disc value is 0.0), which makes every ordinary hero swing that touches the knight a blocked swing (G15). The knight's start point is its spawn position, the character record position (0.0, 0.0, 4.141); it is stored with the character for the whole fight and the high-swing selector measures against it (G6, item 6).
- Timing of the first tick: the level script plays movie 46 and refills the heroes (stage 0 of the boss-fight setup) before it sets its fight-ready level states; the controller thread is created with the area and the fight clock starts when play resumes after the movie (Medium; the module does not hold the controller on a level state).

### G3. Per-tick order (one pass of the controller per tick)

1. Four writes, in this order, on every tick whatever the state (High): store the knight's id as the hero's forced lock target (hero +0x15E0, SPEC-0040 Behavior 20); set the character time scale to 1.0; set the knight's unhittable word (+0x7F0) to 1; set the character base state to 20 (guarding). The unhittable word is never cleared and base state 20 is changed only by the stun gates, which write base state 1 on the ticks where they pass (G9). The knight is therefore unhittable to every generic hero walk for the whole fight, and a guarding target to the hero's swing and shots except on those ticks (G15).
2. Update the six weak-point glow colours (Blend).
3. Flag decision, then the defeat test (Flags, Defeat). Both run every tick; the flag chain only in ready states.
4. Parry (Tor) or deflect (Roberta) check, in ready states only (Parry, Deflect).
5. Attack selectors, in ready states only: counter, kick, slow swing, angry swing, phase-2 group (charge, high swing, floor strike, sword spin), balcony. The first that matches changes the state; later ones then see a busy state (Attack selection).
6. Idle movement choice, in states 0 and 15 to 18 (Idle movement).
7. Handler of the current state (States, Enraged attacks). In state 14 the flying sword runs right after the spin handler, in the same tick.
8. Health display (350 / sum with the quirk), then the swing-ribbon effect, then wait one tick.

Ready states are 0, 15, 16, 17 and 18 (and 20 and above, never used). Busy states are 1 to 14 and 19. So a selector can interrupt a turn, an approach or an orbit, but never an attack, a laugh, a parry or a stance.

### G4. Animation requests and real durations

- A request of the id that is already current does nothing. A new id restarts at key 0 on that tick's animation pass. A request changes the character's current id at once, on the same tick, but the progress gate of a clip that has not started yet reads 0 (not finished); so "request X, then test X" in one tick never passes, and a reaction clip requested by a part hit is still current (and unfinished) when the same tick's stun-clip logic looks at it.
- "Play X to its end" in every handler is: if the current id is not X, request X and wait; otherwise wait until the progress gate (progress at least 0.99, SPEC-0005 Clip progress) passes, then take the exit in that same tick. The gate is read by the controller before the tick's animation pass, so it sees the cursor and fraction left by the previous tick. At 0.25 key per tick the gate passes on tick T + m, with m = 4 x (keys - 1) + 1 for clips of 24 keys or fewer and m = 4 x (keys - 1) for clips of 26 keys or more. A state that plays one clip therefore occupies m + 1 ticks (T to T + m), and the next ready tick is T + m + 1. A step that follows another step starts (and requests its clip) on the tick after the previous step's gate tick.
- Every action id the controller requests (0 to 47) has a clip in the knight's own action table (125 entries; entries 48 to 124 have no clip and are never requested). No requested id is skipped and none falls back to clip 0, so the gates below are the real waits.

| Keys | Action ids | Gate m |
| --- | --- | --- |
| 4 | 22 | 13 |
| 7 | 35, 41 | 25 |
| 8 | 16 | 29 |
| 9 | 26, 33, 45 | 33 |
| 10 | 20, 39 | 37 |
| 11 | 10, 11, 21, 28, 34, 44 | 41 |
| 14 | 24, 38 | 53 |
| 15 | 31 | 57 |
| 16 | 9, 46 | 61 |
| 17 | 43, 47 | 65 |
| 18 | 18, 19, 37 | 69 |
| 19 | 32 | 73 |
| 20 | 29 | 77 |
| 20, looping | 8 (parry; the loop never sets "finished", the gate still passes) | 79 |
| 21 | 17 | 81 |
| 28 | 15 | 108 |
| 30 | 2, 13, 14 | 116 |
| 31 | 1 | 120 |
| 36 | 12 | 140 |

Looping clips with no gate: 0 idle (20 keys), 3 to 5 locomotion (12, 10, 10 keys), 6 and 7 turns (10), 23 floor-strike stun (18), 27 charge run (8), 30 charge stun (10), 36 high-swing stun (30), 40 sword wait (30), 42 neck stun (20).

Root travel of the clips (root motion moves the knight; forward is +z of the knight's frame; units over one full play, from the clip data): slow swing 15 +4.59; kick 17 +1.99; angry swing 18 +1.96, recovery 19 +1.13; floor-strike windup 20 +1.13, follow-through 21 -0.58; charge run 27 +0.5625 per tick, impact 28 +6.88; charge stun start 29 -16.99; high swing, spin and turn clips 0; recoveries 24 -6.52, 31 +1.90, 32 -2.28, 37 +1.65, 38 -7.16, 43 -17.63. A one-shot clip that stays current after it ends moves the knight by twice its last key's root step on every tick it is held (SPEC-0005 Layer step 8): only clip 43 has a large last key (-0.555 per tick; see item G13); the others stay below 0.04 per tick.

### G5. State table with durations (ticks counted from T)

| State | Entry and per-tick work | Ends |
| --- | --- | --- |
| 0 Idle | request 0 every tick | a selector, flag or movement choice |
| 1 Laugh | 12 | T + 140 (141 ticks) |
| 2 Angry start | flag step sets phase 1 on T; 13 | T + 116 (117) |
| 3 Enraged start | flag step sets phase 2 on T; 14 | T + 116 (117) |
| 4 Loosen neck | 1 | T + 120 (121) |
| 5 En garde | 2 | T + 116 (117) |
| 6 Parry | 8; sets the shared "defended" word (+0xE5C) to 1 every tick | T + 79 (80); sets the loosen-neck flag |
| 7 Deflect stance | 9 held; strike 10 (or 11); see Deflect | timer expiry, no armed shot, or strike 10 finished |
| 8 Kick | 17; hit window T + 16 to T + 39 | T + 81 (82); a landed kick sets phase 0 on the tick it lands (any phase) |
| 9 Slow swing | 15, turning 0.25 per tick; hit window T + 40 to T + 51 | T + 108 (109); blocked at tick T + b (b from 40 to 51): 16 from T + b + 1, ends T + b + 30 |
| 10 Angry swing | 18; hit window T + 24 to T + 39 | T + 69 (70); blocked at T + b (b from 24 to 39): 19 from T + b + 1, ends T + b + 70 |
| 11 Charge | run-in 26 steering at 0.25 per tick (T to T + 33); run 27 from T + 34 (contact flags tested from T + 35); see Enraged attacks | see the charge row of G9 |
| 12 High swing | windup 33; hit tick T + 32 only | see G9 |
| 13 Floor strike | windup 20; hit window T + 28 to T + 36 | see G9 |
| 14 Sword spin | 39, wait 40, spin 41, neck stun, 43 | see G12 |
| 15 Turn toward / 16 Turn away | 7 / 6 every tick; turn weight 0.05 per tick | forward-to-hero dot above 0.99; 52 to 82 ticks from 91 to 180 degrees |
| 17 Approach | locomotion mode 0 | horizontal distance to hero below 10.0 (strict) |
| 18 Orbit | locomotion mode 5 or 6 | horizontal distance above 18.0, or hero not on the knight's mesh |
| 19 Defeated | 43 every tick | never |

After any state ends it returns to state 0 on the gate tick and the flag step and selectors of the next tick see state 0.

### G6. Attack choice in priority order (ready states only, one tick, first match wins)

1. Flag chain (before the selectors; at most one flag per tick): laugh flag (clear it; 25 of 101 rolls enter state 1; the tick's flag step ends either way), else angry-start flag (phase 1, state 2), else enraged-start flag (phase 2, state 3), else loosen-neck flag (clear it; 25 of 101 enter state 4), else en-garde flag (state 5).
2. Parry or deflect (item G10).
3. Counter kick: once per fight, on the first ready tick the counter flag is found set; it is cleared and 40 of 101 rolls enter state 8 from any distance. The flag is never set again.
4. Kick: only when the hero is guarding (SPEC-0036 guard predicate), horizontal distance below 5, and a roll 0 to 100 below 70. This roll is made every ready tick (70 of 101 per tick), so a guarding hero within 5 units is kicked almost at once. The roll is drawn only when the guard test and the distance test have already passed. The kick selector has no phase condition, and a kick that lands sets phase 0 (item G11).
5. Slow swing (phase 0, distance below 10, no roll). Angry swing (phase 1, distance below 11, no roll).
6. Phase 2 only: charge (a knee above 0 and the direction test), then high swing (hero distance below 11, the knight's own horizontal distance from its start point below 10.0, an under-arm above 0), then floor strike (back above 0, no distance test), then sword spin (all five other parts at or below 0). In practice the charge test is true whenever the hero's direction is closer to the world x axis than to the z axis (reference +20 or -20 against a value of at most 1 in size), and false otherwise (reference +60 or -60 can never pass a limit of 30); which world axis the first lane is (x) is Medium, and the test is a comparison of the two lane sizes only. Consequence of the start-point rule (High): the high swing is available only while the knight stands within 10.0 of (0.0, 4.141) on the ground plane. After a long approach (the usual case when the hero starts 20 or more units away) or after a charge recoil it is out of range, and a hero lined up with the z axis then gets the floor strike (while the back is above 0) instead; with the back dead and no knee line, no group member matches and the knight falls through to the balcony roll and the idle movement (approach or orbit) until it is back near its start point or the hero moves onto the x line.
7. Balcony (any phase): every ready tick a roll 0 to 100 below 30, with the hero not located on the knight's mesh, enters state 14.
8. Nothing chosen: idle movement (item G7).

### G7. Movement

- Turn: in state 0, when the angle between the facing and the direction to the hero is more than 90 degrees, state 15 (hero on the right axis side) or 16 (left). Each tick of a turn state blends the facing toward a target with weight 0.05. While the hero is behind (forward dot below 0) the target is the knight's own right axis (state 15) or left axis (state 16); once the hero is in the front half the target is the hero direction. The turn ends when the dot is above 0.99. From 91 degrees it takes 52 ticks, from 120 degrees 62, from 150 degrees 72, from 180 degrees 82 (hero standing still). No turning happens inside the 90 degree cone.
- Approach (state 17): chosen when the hero is on the knight's mesh and the horizontal distance is above 10; the knight walks by clip 40's root motion, 6.61 units per 48-tick cycle (0.1377 per tick on average), toward the hero's current position, and stops when the distance is below 10. In phases 0 and 1 the swing selector fires at the same distance (below 10, or below 11), so the approach hands over directly to the swing.
- Orbit (state 18): chosen only when the distance is 10 or less and no selector matched (late fight, when the phase-2 attacks are used up, or exactly 10.0). It starts on a random side with the orbit timer 0. Each tick: the timer counts down if above 0; the side flips (and the timer is set to 30) only on a tick where the timer is 0 or less and the body contact flag is set; otherwise the side is kept. Side 0 is mode 5 (0.1751 per tick to the knight's left), side 1 is mode 6 (0.1649 per tick to its right). It ends when the distance exceeds 18.0 or the target locate fails.
- Hero off the knight's mesh: state 0 does nothing (no approach, no orbit) after the turn test; the balcony roll (item G6, 7) can then start the sword spin.
- Attacks do not steer except the slow swing (0.25 per tick toward the hero) and the charge run-in (0.25 per tick); the knight is moved by the clip's root motion (item G4) and kept on the walk mesh by the shared edge step.

### G8. Hit windows in ticks

A weapon is tested on ticks T + m where the clip's current key (cursor whole part, key f lasts ticks 4f to 4f + 3) has the weapon's event bit set and the clip has not yet passed its gate. One hit per attack (latch). Frames come from the clip data and match the table in Hit windows and damage.

| Attack | Weapon | Keys with bit | Ticks T + m |
| --- | --- | --- | --- |
| Kick (17) | 5 | 4 to 9 | 16 to 39 |
| Slow swing (15) | 1 | 10 to 12 | 40 to 51 |
| Angry swing (18) | 1 | 6 to 9 | 24 to 39 |
| Charge impact (28), counted from the tick 28 is requested | any | 3 to 5 | 12 to 23 |
| High swing windup (33) | 2 | 8 | 32 |
| Floor strike windup (20) | 1 | 7 to 9 | 28 to 36 |

### G9. Weak-point loop and stun windows

- Exposure: the stun timer is set to 600 on the gate tick g of the stun-start clip (22, 29, 35, or the spin) with no test on that tick. Ticks g + 1 to g + 600 are tested. On g + 601 the timer is 0, the step ends with no test, and on g + 602 the recovery clip is requested (31 for the charge, 37 for the high swing, 24 for the floor strike, 43 for the spin), ending on g + 602 + m (charge 31: g + 659; high swing 37: g + 671; floor strike 24: g + 655), then idle. The spin has no timer recovery: when its timer runs out the state returns to idle, phase 0, on g + 601. If the last required part reaches 0 on tick h (both knees, both under-arms, the back, or the neck), the step moves to the part-loss recovery at once on that tick: the clip is requested on h + 1 and the state ends on h + 1 + m (knees 32: h + 74; under-arms 38 and back 38: h + 54), and these three set the en-garde flag. For the knees and the under-arms the second part's death is the trigger: with one of the pair still above 0 the stun goes on until the countdown ends (High: the end test is made after the second part's test and needs both values at 0 or below); the neck goes to 43 and sets nothing (the defeat test takes over on h + 1).
- Per tick during the stun (all four attacks): (1) timer test and decrement; (2) request the stun clip unless the part reaction (44 to 47) is still playing; (3) set the glow flags of parts with HP above 0; (4) build the part cylinders; (5) gate. The gate for the melee part test is: the hero is attacking and positioned relative to the knight as follows: knees need the attack-state predicate (SPEC-0040 Behavior 18) and the horizontal direction from the knight to the hero within the front half-space (dot with the knight's forward axis above 0.25); the back needs the attack-state predicate and the hero behind (dot below -0.25); under-arms need the strike-active predicate (Tor: action state 13 with sub-state 11 or 12, or action state 7 with air phase 8 or 9; Roberta: action state 13 with sub-state 5 or 12; for Tor only the second form can occur, so Tor needs the plunge of a standing jump, G15) and the hero on a side (dot of the direction to the hero with the knight's side axis above 0.25 for the first under-arm slot, below -0.25 for the second, and that slot's HP above 0; the side axis is the knight's +X when it faces +Z, so the first slot, the left under-arm, is the one a hero on the knight's left attacks; Medium-High for the arm-to-side pairing). A hero on the side of a dead arm fails the gate and is blocked; the neck needs the strike-active predicate and the hero behind (dot below -0.25). (6) If the gate fails and no hero shot slot is armed, the part latch is cleared and the tick's test ends. (7) Otherwise, when the part latch is clear and the hero's weapon attack window is open, or when any hero shot slot is armed, the shared weak-point hit routine runs on each exposed part in the order of the attack (right knee then left; right under-arm then left; back; neck). A hit sets the latch, so a single swing hurts once; the latch clears on the first tick whose gate fails (the swing is over or the hero moved). The routine's cylinders, order and damage are in Weak-point test in the enraged attacks. Because the hero must be swinging, standing in a part's cylinder without attacking does nothing.
- Outside these windows no hero hit lowers any part: the actor's own HP is rewritten from the part values every tick, so ordinary hits on the body do not matter. The hero's generic hit routines never reach the knight's take-hit at all (it is unhittable and, for a swing, blocked; G15), so the knight never flinches, flashes or plays a hit sound from a hero hit; the only sound is the special-victim impact cue (High).
- Base state 1 on a passing gate (High): on every stun tick where the gate of step 5 passes, the knight writes base state 1 (not guarding) after the per-tick write of 20. The gate is evaluated before the part test and before the hero's update of the same tick (the knight's think runs first, SPEC-0007), so the hero's swing and shots of that tick find a target that is neither hittable nor guarding and pass through it; the damage then comes only from the weak-point routine. On a tick where the gate fails the base state stays 20 and the hero's swing is blocked (Tor recoils; G15). This is the whole reason a swing from the wrong side bounces off while the same swing from the right side hurts.
- Reaction lengths (High, from the clip data): the part-hit reaction clip plays to its gate before the stun clip is requested again: 44 (floor strike) 42 ticks, 45 (charge) 34 ticks, 46 (high swing) 62 ticks, 47 (sword spin) 66 ticks counted from the hit tick. The charge step requests its stun clip at the start of its tick (before the part test) and the other three at the end of their tick (after it); the result is the same, the reaction requested on the hit tick stays current. Hits that land during a reaction count fully and do not restart it.
- Lock marker (Medium): every stun tick the knight overrides its own target point with the frame of the exposed part (for the two-part attacks, the live part on the hero's side of the knight). This moves the hero's lock marker and lock aim point (SPEC-0040 Behavior 20, item 6) onto the weak point; nothing else in the fight reads it.
- Level state 10 (High for the writes): the enraged high swing sets level state 10 on the arm-open gate tick (the tick the 600 countdown starts) and clears it when the stun ends, whether by the countdown or by both under-arms reaching 0. No other attack sets it. The arena script uses it to loop and stop an animation on arena object 2 (Arena flow).
- Floor-strike stun (Medium-Low): on every tick the knight also adds its weapon 1 disc (radius 8.0, length 0.75, on the hand-grip frame) to the hero's pending contact shape list (SPEC-0040 Behavior 14A, tag 0); the shared mover decides what that does to the hero's movement.
- Charge row: after the run-in (T + 33) the run clip starts at T + 34 and its contact flags are read from T + 35. Let c be the first tick on which a flag is found set. If the edge-contact flag is set, clip 29 is requested on c + 1 and the stun-start gate is g = c + 78. Otherwise, if only the body contact flag is set, clip 28 is requested on c + 1, the hit window is c + 13 to c + 24, and the state ends on c + 42. Travel before contact is 0.5625 units per tick along the heading fixed at the end of the run-in; clip 29 then moves the knight back 16.99 units over its 78 ticks.
- High swing row: windup 33 requested on T; if the weapon touches on tick T + 32 the hit lands and follow-through 34 is requested on T + 33 (ends T + 33 + 41, no stun, phase 0); with no hit the gate passes on T + 33 and arm-open 35 is requested on T + 34 (gate g = T + 34 + 25 = T + 59).
- Floor strike row: windup 20 requested on T; a hit on tick h in T + 28 to T + 36 leads to follow-through 21 (requested on h + 1, ends on h + 42, no stun); with no hit the gate passes on T + 37 and stun start 22 is requested on T + 38 (g = T + 38 + 13 = T + 51).

### G10. Parry and deflect

- Both checks run only in a ready state, every tick, and are skipped entirely (the latch is not cleared either) while the knight is busy.
- Tor: when the attack-state predicate is false the shared latch clears. When it is true and the latch is clear, the latch is set and one roll 0 to 100 is made against the parry accumulator: below it enters state 6 and resets the accumulator; otherwise the accumulator grows by 50. One roll per hero attack, so the chances on successive attacks are 0, 50 of 101, 100 of 101 until a parry resets it. The latch stays set while the hero keeps attacking or while the knight stays busy.
- Roberta: the six hero shot slots are scanned; on the first armed slot with the latch clear the latch is set and, if that slot overlaps the knight, a roll against the deflect accumulator (growing by 20 per failure) can enter state 7 with a stance timer of 240 to 360 ticks. With no armed slot the latch clears. "Overlaps" means: a segment 200.0 units long, starting at the slot's position and running along the slot's direction, crosses the knight's body cylinder (radius and height from its motion record, 1.5 and 5.5; centred half the height above its position) with the crossing point ahead of the slot and inside the segment (Medium-High). So the stance can start while the shot is still far away, as long as it is aimed at the knight. The roll is float-compared: the integer roll 0 to 100 is below the accumulator (0, 20, 40, ...).
- The branch is chosen every tick from the hero that is currently selected (hero index 0 or 1). The latch and both accumulators are shared and persist across a hero swap; nothing resets them except a successful roll (the accumulator of the branch that succeeded).
- Deflect stance: every tick requests 9. It ends at once if no shot slot is armed on that tick. While a slot is armed and its distance (3D) from the knight is below 10.0, each tick picks strike 10 or 11 with equal chance; strike 10 plays to its gate (41 ticks) and ends the state; strike 11 is replaced by the stance request on the next tick because the stance code only recognises 10 as a strike (so it is a one-tick flicker and the pick repeats next tick). The state also ends when the timer is below 1 and strike 10 is not playing. The timer drops by 1 every tick of the state.
- Sticky "defended" word (+0xE5C): set to 1 on every tick the parry plays and during strike 10 and never cleared during the fight. After the first parry or first strike 10, Roberta's quick shots bounce off the knight for the rest of the fight (SPEC-0036 Quick shot meets a guarding target).
- Parry's end sets the loosen-neck flag: the next tick, 25 of 101 rolls enter state 4.

### G11. Phases

- Phase 0: slow swing. A swing that does not land (miss or blocked) sets the angry-start flag: on the next tick phase 1 and state 2 (117 ticks), then the angry swing is chosen at once if the hero is within 11. A swing that lands sets the laugh flag (phase 0 stays).
- A landed kick sets phase 0 from any phase, on the tick it lands (the kick has no phase condition). A kick in phase 1 or 2 that lands therefore throws the fight back to the slow swing; a kick that misses changes nothing.
- Phase 1: angry swing. Missed or blocked sets the enraged-start flag: next tick phase 2 and state 3 (117 ticks). Landed sets phase 0.
- Phase 2: exactly one of the group (G6, 6). Every enraged attack, however it ends, sets phase 0. The charge impact that lands sets phase 0 too.
- Slow swing to the first enraged decision with the hero standing in range and every swing missing: 109 + 117 + 70 + 117 = 413 ticks plus the approach.
- Phase and the weak-point HP are never reset by a phase change.

### G12. Flying sword timeline (state 14 entered on tick T)

- T: request 39 (spin-up). T + 37: gate passes, the step becomes 1, the sword-returned flag clears, the sword sub-state becomes 0, and the sword runs on this same tick (attach: sub-state 0 to 1).
- T + 38: the spin requests 40 (wait). The sword's launch sub-state (1) runs each tick: it moves 0.4 units toward a launch point (the knight's position + 3.0 along its forward axis + 5.0 up, Medium) and ends when it is within 0.4 of it. With the hand grip D0 units from that point, launch lasts about D0 / 0.4 + 1 ticks (roughly 10 to 21 for D0 between 4 and 8; Low). Every launch tick resets the shared orbit values.
- Orbit tick n (n = 1, 2, ...) adds 0.05 + 0.00075 n to the angle. Full turns end on orbit ticks 79, 128, 167, 201, 230, 257, 282, 306, 327 (count reaches 9), 348 (10th). On those ticks the hit latch clears; the sword can hit the hero (15 damage and knockback) once between them. Homing starts on tick 348 if the sword is within 0.05 of its target height (usual), else on the first later full turn that is (ticks 368, 387, 405, 422, 439 = latest).
- Homing: 1.25 units per tick toward the hero (H = distance / 1.25 ticks, about 24 for 30 units) until it touches the hero (15 damage, no knockback) or scene geometry; then return at 1.25 per tick to the hand (R about the same); on the tick the remaining distance is below 1.25 the sword-returned flag is set and the sword is held at the hand (sub-state 5).
- The next spin tick sees the flag and moves to the spin step; the tick after requests 41 (gate 25). 41 finishes: if any of the other five parts is above 0 (the balcony case) the attack ends, idle, phase 0. Otherwise the neck stun starts: 42 requested the next tick, timer 600 on that tick, exposure as in G9, recovery 43.
- Whole spin with launch L, homing H and return R ticks: about 38 + L + 348 + H + R + 2 + 26 ticks (about 480 to 560 ticks with the usual homing start), plus the neck stun and recovery when the other parts are dead.

### G13. Arena hand-offs and death

- Level state 0 is set once by the defeat test, on the first tick all six parts are at or below 0. The arena script reacts with level state 6, the fade-out, movie 47 and the return to world 78 (Arena flow). The knight has one other hand-off: level state 10, set while the enraged high swing's under-arm stun runs (G9) and read by the arena script's object-2 animation loop. It neither waits for nor reads any level state.
- After the defeat the knight's loop keeps running in state 19: it still writes unhittable and base state 20 every tick, so hero swings at the dying knight are still blocked (Tor still recoils) until the fade.
- Death: state 19 requests 43 every tick: 17 keys, 66 ticks to the last key, then the knight stays on the last key and (root motion rule above) slides back by 0.555 units per tick until the walk-mesh edge stops it (Medium). Nothing leaves state 19.
- Defeat ordering: a part that reaches 0 on tick t is seen by the defeat test on tick t + 1, which enters state 19 before that tick's selectors and handler.

### G14. Mistakes a port is likely to make

- Treating the orbit side flip as a 30-tick timer; it needs body contact.
- Letting the kick fire at any distance; only the counter kick (first ready tick) does, otherwise a guarding hero is required.
- Playing clip 0 for a missing action id; none is missing, every wait is the clip's real gate (G4).
- Counting 4 ticks per key at the wrong speed byte or play rate (G4), or counting seconds where keys are meant.
- Treating the stun as damage-on-contact; the hero must be swinging, in the right half-space, with the part latch clear (G9).
- Using the speed multiplier or the unused tunables (nothing reads them).
- Resetting the parry latch while the knight is busy (it is not touched then).
- Ending the swing at the first hit (the swing plays to its gate; the hit only sets the latch and, for the high swing and floor strike, switches to the no-stun follow-through).
- Horizontal versus 3D distance: every range in the tuning table is horizontal; the deflect strike distance (10.0) is 3D.
- Treating the knight as an ordinary hittable enemy: hero swings are blocked and shots bounce; the only damage path is the weak-point routine in the four stun windows (G9, G15).
- Leaving the high swing available at any distance: it needs the knight within 10.0 of its start point (G6, item 6).
- Forgetting that a landed kick sets phase 0 (G11).

### G15. Hero actions against this boss

Rule (High): from the hero's side the knight is unhittable and guarding on every tick, except on the stun ticks where the gate of G9 step 5 passes (base state 1 for that tick). No hero action lowers a part except through the weak-point routine of an open stun window (G9); everything else is blocked, bounced or ignored. The knight's own hits on the hero are unreduced by armour, shield or Motes; only the two swings can be cancelled by a block (Hit windows and damage).

| Hero action | Outside a window, or the gate fails (knight guarding) | In a window with the gate passing |
| --- | --- | --- |
| Tor ordinary swing (combo steps) | Blocked at the first overlapping tick of the swing (the Blocking Angle is 1.0, so facing does not matter): no damage, no knight reaction, impact cue 34, the knight joins the swing's hit list (one block per swing). Tor's sword recoils (attack sub-state 13, the sword bounce) for the combo steps and the swing ends. A blocked swing still pays the hero's magic and Mote charge like a landed hit (SPEC-0036 Damage) | The swing passes through the knight. The weak-point routine runs when the hero is in an attack state (the gate rules of G9: knees need the hero in front, back needs the hero behind) and its weapon window is open; one hit per swing |
| Tor power step (Mote attack, sub-state 10) | Not blocked and not a hit: it passes through with no effect (Medium-High: a power step with a Mote skips the block path) | Same as an ordinary swing for the knees and the back; the damage uses power 1.0, not the power table, so the Mote adds nothing but its element factor |
| Tor plunge (jump attack) | Blocked and bounced like a swing (Medium) | The only Tor action that can hurt the under-arms (high-swing stun) and the neck (sword-spin stun): their gate needs the strike-active test, which for Tor is true only in jump state 7 (a standing jump) in air phase 8 or 9, with the hero at the side (under-arms) or behind (neck). Also valid for the knees and the back. Plunge hits do double damage (step factor 2.0 in sub-state 12) |
| Roberta melee (attack sub-states 5 and 12) | Blocked, no recoil, no damage | Same gates as Tor, with the strike-active test true in sub-state 5 or 12. Her step factor is 0.5, so each hit is weak |
| Roberta quick shot | While the shot's arming counter is above 0 it passes through. Otherwise it bounces off (scatter, SPEC-0036 Quick shot meets a guarding target). After the knight has once parried or struck from the stance, the sticky defended word is set for the rest of the fight and the shot is sent back at Roberta as a hostile shot | An armed shot slot is tested whatever the hero is doing, even when the melee gate fails: it hits when its position is at least 0.5 ahead along the routine's axis and inside the exposed part's sphere (radius 5.0 plus the shot radius), is disarmed and hurts the part (G9, Weak-point test). The sphere is large, so the shot usually registers before it reaches the body |
| Roberta full release | Not blocked; disarmed on contact with no damage | Same as a quick shot; damage uses power 1.0, not the table power |
| Motes (spell objects, beams) | Skipped by the generic character walks (the knight is unhittable): no effect (Medium-High) | Same |
| Shield or guard against the knight | Kick, charge impact, high swing, floor strike, sword: unblockable, damage unchanged. Slow and angry swing: cancelled into the recovery clip with no damage when the hero guards (state 20, Roberta spinning, or a running guard timer), the weapon touches and the facings are opposite (dot below -0.5); a swing that already landed can still be cut short this way | - |
| Dodge, roll, invulnerability | None exists for either hero (SPEC-0036); a knight hit lands whenever its weapon volume overlaps a hero bone capsule while its window is open; the kick, charge impact and sword hit knock the hero back | - |
| Lock and facing | The hero's lock target is forced to the knight every tick, so the lock marker and the hero's attack turn assist always point at it; during a stun the marker sits on the exposed part | - |
| Items and petpets | Nothing in the fight acts on the knight; healing items change only the hero (petpets not examined) | - |
| Hero dies | The knight takes no notice. Its code never reads hero health; the swap or game-over rules of SPEC-0055 apply and the fight continues against whichever hero is selected | - |

Damage per weak-point hit (High): d = truncate(A x B x C x D x E), the melee formula of SPEC-0036 with attack power 1.0, defence 0 and target type 0 (the knight's row has defence 0.0 and type 0, so the Mote type factor C applies only to Nova Motes). A is the equipped weapon's damage stat (an unarmed hero deals 0), D is the step factor of the hero's current attack sub-state (Tor 2.0 in sub-states 4 and 12, 0.75 in 5 and 6; Roberta 0.5 in 5 and 12), E is 1.25 with certain petpets. Example: a Tor weapon with A = 4 and no Mote deals 4, or 8 in sub-state 4 or in the plunge; a 50-point part then takes 13 hits at 4, or 7 hits at 8.

### G16. Edge cases for implementers

- First tick: the controller thread is created with the area, so a few ticks of thinking (the opening counter roll, the first approach step) may happen before movie 46 starts; the rewrite default is to start the controller after the movie (Open questions).
- The kick sets phase 0 on landing, in any phase (G11).
- The high swing needs the knight within 10.0 of its start point (0.0, 4.141); otherwise the floor strike is chosen for a hero on the z line (G6, item 6). Charge needs the hero closer to the x axis than to the z axis (G6, item 6).
- Under-arms and the neck can be hurt by Tor only with the plunge of a standing jump (state 7); a running or walking jump (states 8 to 11) counts for the knees and the back only. Roberta uses the same swats for all four windows.
- A swing from the wrong half-space during a stun bounces off (the gate fails, base state stays 20). A swing that hits the knight outside any window bounces too.
- The block test of the two swings does not look at the landed latch: a swing that has already landed turns into its recovery clip if the hero then guards, the weapon still touches and the facings are opposite. The recovery then sets the missed-swing flag at its end, even though the hit landed (phase was already 0).
- The slow swing keeps turning toward the hero at 0.25 per tick on every tick of its state, including its blocked recovery; the angry swing never turns.
- "Request X, then test X in the same tick" never passes (G4): a state that requests its clip on tick T tests it from T + 1.
- A selector can interrupt a turn, approach or orbit at any tick, and also the sword spin from the balcony roll while moving; the balcony roll (30 of 101) is drawn on every ready tick, so a hero off the mesh gets the sword spin within about 3 ticks on average.
- Selectors use horizontal distance only, never the walk mesh. A hero standing off the mesh within range is still swung at, kicked and charged; only the balcony roll, the idle movement and the orbit look at the mesh.
- Same-tick order: the flag step comes before the defeat test, so a flag-step state (laugh, angry start, enraged start, loosen neck, en garde) on the same tick as the last part reaching 0 is overwritten by the defeated state.
- The parry state writes base state 1 on its last tick; the next tick writes 20 again. The defended word (+0xE5C) is set every tick of the parry and of strike 10 and never cleared.
- The deflect stance ends on the first tick with no armed shot slot, so it normally lasts only as long as the shot is in flight; the 240 to 360 tick timer matters only while shots keep coming.
- Stun gates, part latch and reaction: one hit per swing; reaction 44 to 47 does not restart on a second hit; the stun clip returns after the reaction's gate (G9).
- The level state 10 write of the high swing is set on the arm-open gate tick and cleared on the exit tick, not at the hit.
- The knight stays unhittable and guarding after it is defeated (G13).
- A hero swap keeps the shared parry/deflect latch and both accumulators (G10).
- Random draws: counter once, kick only when guarding and in range, balcony on every ready tick, flag rolls once per flag; the draw order does not need to match the original, only the distributions.

## Where it lives on the disc

- The tuning values are 32-bit words in the boot program's data, read at runtime by virtual address. Almost all of them lie between 0x2DA3E8 and 0x2DA483, and the flying-sword words lie in the same block. Each value below gives its address. "int" is a signed 32-bit integer and "float" an IEEE single.
- The values given are the stored values at those addresses. The stored value is the one the behavior uses (no host override is assumed; see Open questions).
- The clean side must not invent these values; it reads them from the disc, or from a documented default table if the read fails validation.
- The generic spawn lookup reads type and variant from the character data block and points into a shared monster-table row; it does not copy the row. In world pack 989, the character record named Black Knight has data-relative type 0 and variant 0, so this lookup selects row 0 and initially sets actor HP from that row's +0x30 field. The boss controller's state and combat fields live in a separate per-fight record on its active stack frame. The row pointer stored on the actor supplies other character data such as model scale; it does not back the handler fields described below.

## Layout

### Tuning values

| Name | Type | Address | Default | Meaning |
| --- | --- | --- | --- | --- |
| LaughChance | int % | 0x2DA3E8 | 25 | Chance of a laugh after a landed slow swing |
| LoosenNeckChance | int % | 0x2DA3EC | 25 | Chance of a neck-loosening move after a parry |
| EnGardeIncrease | int % | 0x2DA3F0 | 25 | Speed multiplier increase when the en garde stance starts (the multiplier is never read, so this has no effect in play; Open question 6) |
| ParryChanceIncrease | int % points | 0x2DA3F4 | 50 | Added to the parry accumulator per failed parry check |
| DeflectChanceIncrease | int % points | 0x2DA3F8 | 20 | Added to the deflect accumulator per failed deflect check |
| DeflectStanceMin | int s | 0x2DA3FC | 4 | Lower bound of the deflect stance |
| DeflectStanceMax | int s | 0x2DA400 | 6 | Upper bound of the deflect stance |
| CounterAttackChance | int % | 0x2DA404 | 40 | Chance of the opening kick (the counter flag check, see Attack selection, item 1) |
| KickAttackRange | int units | 0x2DA408 | 5 | Kick range (horizontal distance from the boss to the hero) |
| KickAttackChance | int % | 0x2DA40C | 70 | Chance of a kick when the hero is in kick range |
| KickAttackDamage | int | 0x2DA410 | 20 | Damage to the hero |
| SlowSwingRange | int units | 0x2DA414 | 10 | Slow-swing range (horizontal distance from the boss to the hero) |
| SlowSwingDamage | int | 0x2DA418 | 20 | Damage to the hero |
| AngrySwingRange | int units | 0x2DA41C | 11 | Angry-swing range (horizontal distance from the boss to the hero) |
| AngrySwingDamage | int | 0x2DA420 | 20 | Damage to the hero |
| EnragedChargeRange | int | 0x2DA424 | 30 | Limit of the charge's direction test (see Attack selection, item 5a) |
| EnragedChargeSpeed | int | 0x2DA428 | 80 | Unused in the original (no load or store in the boot program's data; keep the value, do not apply) |
| EnragedChargeDamage | int | 0x2DA42C | 20 | Damage to the hero on charge impact |
| EnragedChargeStunTime | float s | 0x2DA430 | 10.0 | Vulnerable window after the charge |
| EnragedHighSwingRange | int units | 0x2DA434 | 11 | High-swing range (horizontal distance from the boss to the hero, first measure) |
| EnragedHighSwingDamage | int | 0x2DA438 | 20 | Damage to the hero |
| EnragedHighSwingStunTime | float s | 0x2DA43C | 10.0 | Vulnerable window after the high swing |
| EnragedFloorStrikeDamage | int | 0x2DA440 | 20 | Damage to the hero |
| EnragedFloorStrikeStunTime | float s | 0x2DA444 | 10.0 | Vulnerable window after the floor strike |
| EnragedSwordSpinStunTime | float s | 0x2DA448 | 10.0 | Vulnerable window after the sword spin |
| Turn threshold (view half-angle, degrees; stored name FieldOfView) | int degrees | 0x2DA44C | 90 | Turn threshold (see Edge cases) |
| CloseDistance | int units | 0x2DA450 | 10 | Approach stops inside this horizontal distance from the boss to the hero |
| CloseSpeed | int | 0x2DA454 | 5 | Unused in the original (no load or store in the boot program's data; keep the value, do not apply) |
| OrbitDistance | int units | 0x2DA458 | 10 | Orbit starts inside this horizontal distance from the boss to the hero |
| FlyingSwordDamageA | int | 0x2DA45C | 15 | Damage to the hero from a flying sword. At most once per revolution of the orbit, and once on a homing hit (see Flying swords) |
| FlyingSwordSpeed | int units/s | 0x2DA460 | 75 | Homing and return speed of a flying sword. Applied as value / 60 units per tick (1.25 per tick) |
| FlyingSwordDamageB | int | 0x2DA464 | 20 | Unused in the original (no load or store in the boot program's data; keep the value, do not apply) |
| FlyingSwordLaunchThreshold | float | 0x2DA488 | 0.4 | A sword leaves its launch when its distance to the launch point falls below this; it is also the distance the sword moves toward that point on each launch tick |
| FlyingSwordSpinStep | float rad/tick | 0x2DA48C | 0.05 | Orbit angle step set at each launch (a shared value; see Flying swords) |
| FlyingSwordSpinAccel | float rad/tick per tick | 0x2DA494 | 0.00075 | Growth of the shared orbit angle step each orbit tick |
| FlyingSwordPathRate | float per tick | 0x2DA4A4 | 0.002 | Growth of the shared path rate each orbit tick. The path angle grows by the rate each tick |
| FlyingSwordBounceSpeed | float per tick | 0x2DA4BC | 0.04 | Speed of the up-and-down bounce during the orbit |
| FlyingSwordBounceLimit | float | 0x2DA4C0 | 0.5 | The bounce runs between minus and plus this value |
| FlyingSwordRevolutions | int | 0x2DA4B8 | 9 | Full turns of the orbit before the sword may start homing |
| LeftKneeHP | int | 0x2DA468 | 50 | Left knee maximum HP |
| RightKneeHP | int | 0x2DA46C | 50 | Right knee maximum HP |
| LeftUnderArmHP | int | 0x2DA470 | 50 | Left under-arm maximum HP |
| RightUnderArmHP | int | 0x2DA474 | 50 | Right under-arm maximum HP |
| BackHP | int | 0x2DA478 | 50 | Back maximum HP |
| NeckHP | int | 0x2DA47C | 100 | Neck maximum HP |
| BalconyAttackChance | int % | 0x2DA480 | 30 | Chance of a sword spin from the balcony check |

Hard-coded constant (not a tunable): a float 20.0 stored at 0x2DA484, used as the knock-back scale on the hero for kick and charge hits.

Tick rate: the per-second rate that the speeds and timers use (seconds x rate for timers, value / rate per tick for speeds, for example 75 / 60 for the homing speed) is the float 60.0 stored at 0x30C598, the first entry of the 60 Hz row of the frame-rate table, chosen at boot (SPEC-0007). So the boss runs at 60 ticks per second (High for the value; Medium that the table index chosen at boot is always the first one).

Shared runtime words in the same block (not tuning; they start at zero or at a fixed value and the boss changes them while the sword runs): the shared spin step (0x2DA490, set to 0.05 on each launch and grown each orbit tick), the shared path rate (0x2DA498, reset to 0 on each launch and grown each orbit tick), the reset value of the path rate and path angle (0x2DA49C, 0.0), the shared path angle (0x2DA4A0, starts at 0), the side scalar (0x2DA4A8, set to 3.0 on each launch from the reset value at 0x2DA4AC; during orbit it scales a normalized pose-derived offset before that offset is added to the boss position), the per-tick increment of the side scalar (0x2DA4B0, zero on disc, so the side scalar does not change during the orbit), the shared revolution count (0x2DA4B4, reset to 0 on each launch), and the lower bound of the sword's target height (0x2DA4C4, a float 4.0).

Only the values above are used by the behavior. Other tunables listed under this boss's name in the editor range are not used by the boot program; do not apply them. The three rows marked unused (EnragedChargeSpeed, CloseSpeed, FlyingSwordDamageB) are read by no code in the boot program, neither directly nor through a base address plus offset (High; Open question 5).

### Constants used but not on the disc

- Finished-animation threshold: an animation counts as finished when its playback progress reaches 0.99.
- Turn-to-face threshold: a turn is finished when the forward direction's dot product with the direction to the hero is above 0.99 (about 8 degrees).
- Orbit exit distance: 18.0 units, a horizontal distance from the boss to the hero (hard-coded).
- Orbit side flip period: 30 ticks.
- Turn steering step: 0.05 per tick along the forward axis (Medium).
- Orbit target offset: 3.0 units along the perpendicular direction.
- Charge run-in steering: 0.25 per tick toward the hero while the run-in animation plays (hard-coded, Medium).
- High-swing second measure limit: 10.0 units, applied to the horizontal distance between the knight and its start point (hard-coded, High; Attack selection item 5b).
- Random numbers: uniform integers in an inclusive range. The original uses the shared Mersenne Twister of SPEC-0014 (state at data address 0x39BE50, seeded on first use from the console's real-time clock); the clean side need not match its sequence, only the distribution.

### Weak points

| Part | Maximum HP | Vulnerable only during |
| --- | --- | --- |
| Left knee | 50 | Enraged charge stun |
| Right knee | 50 | Enraged charge stun |
| Left under-arm | 50 | Enraged high-swing stun |
| Right under-arm | 50 | Enraged high-swing stun |
| Back | 50 | Enraged floor-strike stun |
| Neck | 100 | Sword-spin stun |

Total maximum HP across the six parts is 350 (50, 50, 50, 50, 50, 100). At controller setup, each part's current HP is initialized from its matching maximum. The health display sets the boss's maximum HP to 350 on every tick and its current HP to the sum of the current part values with the quirk in Edge cases (High). Generic spawn initially sets actor HP from the selected row's +0x30 field; the health display replaces it with the weak-point sum on the first controller update.

The defeat test runs every tick (see Defeat). If all six part values are at or below zero, the boss goes to the defeat state. Since all six start at their positive maxima, the encounter does not begin defeated.

### Boss state storage and fields

The boss handlers use a per-fight record in the active stack frame. It is initialized before the first controller tick. Its field meanings and use are described below:

- Current state (0 to 19): word 0. It starts at 0 (idle). The current sub-step of the active state is set when the state is entered.
- Phase (0 slow, 1 angry, 2 enraged): word 1. It starts at 0 (slow).
- Speed multiplier: word 4, a float. It starts at 1.0.
- Counter flag: word 15. It starts at 1 and is cleared when the counter selector checks it.
- Six weak-point HP values: words 19 to 24 start at their matching maxima (50, 50, 50, 50, 50, 100). Each is floored at 0 by damage.
- Four charge reference values: words 25 to 28 start at +20, −20, +60, −60, respectively. The charge selector uses them as described below.
- Flags: laugh, angry start, enraged start, loosen neck and en garde, the sword-returned flag for the sword spin, and the per-attack hit latches change under the rules below. All of them start at 0 (Fight AI build guide, G2); the sword-returned flag is cleared at spin-up and the hit latches are cleared when an attack is selected.
- A stun timer, a deflect stance timer, an orbit side (0 or 1) and an orbit timer (ticks): not given a start value; each is set before it is used (Medium).
- Parry accumulator, deflect accumulator and one shared parry/deflect latch: they start at 0.
- The edge-contact flag with its hold counter, and the body contact flag, read by the charge run step. Both are generic AI-character and body words set by the body movement step, not part of the boss's own state (see Edge contact).
- A walk mesh. At controller start, if the knight has no walk mesh, it looks up the walk mesh that contains its position and assigns it; if none is found it fights without one (High).
- One flying-sword record per boss, with its sub-state (0 to 5), orbit angle, bounce value and a hit latch. The spin step, path rate, path angle, revolution count and side scalar are the shared values listed under Tuning values.

## Behavior

### Per-tick order

Each tick, after the four per-tick writes of the build guide (G3 step 1: forced hero lock, time scale 1.0, unhittable word 1, base state 20), in this order (order matters because later steps see earlier flags) (High):

1. Update the six weak-point glow colours (visual; see Blend).
2. Flag decision (see Flags below), then the defeat test (see Defeat below). The defeat test runs in every tick and every state, before the attack selectors and the state handler.
3. Parry and deflect checks.
4. Attack selection (only if the boss is in a ready state).
5. Idle movement selection (only in idle or movement states).
6. Run the handler of the current state. In the sword-spin state (14) the flying sword (see Flying sword) runs right after the spin step, in the same tick.
7. Update the health display. This sets the boss's maximum HP to the sum of the six maxima (350) and its current HP to the sum of the current part values (with the quirk in Edge cases). It runs every tick, so the boss's HP values are rewritten each tick (High).
8. Update the effect (see Effects).

Any state change made in steps 2 to 4 is seen by the later selectors in the same tick, so they do not fire.

### States

The boss is busy in states 1 to 14 and 19. It is ready (may start a decision) in states 0, 15, 16, 17, 18, and 20 and above. Each state ends by returning to idle (state 0) unless noted. Animation numbers are entries of the boss's main animation table (SPEC-0005).

| # | State | Animations | What it does | Leaves when |
| --- | --- | --- | --- | --- |
| 0 | Idle | 0 | Hover in place. Played every tick. | Chosen by a decision or movement check |
| 1 | Laugh | 12 | Plays the laugh animation | Animation finished |
| 2 | Angry start | 13 | Sets phase to 1 | Animation finished |
| 3 | Enraged start | 14 | Sets phase to 2 | Animation finished |
| 4 | Loosen neck | 1 | Plays the neck animation | Animation finished |
| 5 | En garde | 2 | Multiplies the speed multiplier by (EnGardeIncrease + 100) / 100 (the word is never read; the knight does not speed up) | Animation finished |
| 6 | Parry | 8 | Plays the parry. Its end sets the loosen-neck flag (see Parry). | Animation finished |
| 7 | Deflect stance | 9 stance, then 10 or 11 strike | Holds the stance (9 requested every tick) while a hero shot slot is armed. With an armed shot slot within 10.0 units (3D) of the knight it picks strike 10 or 11 with equal chance each tick; only strike 10 is recognised as a strike (strike 11 is overwritten by the stance request on the next tick). The strike deals no damage (High). See Deflect | Immediately when no shot slot is armed; when the stance timer is below 1 and strike 10 is not playing; or when strike 10 reaches its gate (41 ticks) |
| 8 | Kick | 17 | Short-range kick. Deals KickAttackDamage on a hit and knocks the hero back. A landed kick sets phase to 0 (any phase). | Animation finished |
| 9 | Slow swing (phase 0) | 15 swing, 16 recovery | Swing, turning toward the hero by 0.25 per tick. If the hero blocks it (see Hit windows and damage), the swing turns into recovery 16 with no damage. A landed hit sets phase to 0 (no change). | Animation finished. Sets laugh flag if it landed, otherwise the angry-start flag. |
| 10 | Angry swing (phase 1) | 18 swing, 19 recovery | Swing. Same block rule (recovery 19). A landed hit sets phase to 0. | Animation finished. Sets the enraged-start flag if it did not land. |
| 11 | Enraged charge (phase 2) | 26, 27, 28, 29, 30 (45 on a hit), 31 or 32 | Charges at the hero when the charge direction test passes (see Attack selection, item 5a). Knees are vulnerable during the stun window, which is reached when the run touches the edge of the walk mesh (see Enraged attacks). | Recovery animation finished |
| 12 | Enraged high swing (phase 2) | 33, 34, 35, 36 (46 on a hit), 37 or 38 | High swing. Under-arms are vulnerable during the stun window. | Recovery animation finished |
| 13 | Enraged floor strike (phase 2) | 20, 21, 22, 23 (44 on a hit), 24 or 38 | Floor strike. Back is vulnerable during the stun window. | Exit animation finished |
| 14 | Sword spin / flying sword (phase 2, or balcony check) | 39, 40 (wait), 41, 42 (47 on a hit), 43 | Spin with the flying sword. Neck is vulnerable during the stun window. | Recovery animation finished, or the spin ends early (see Enraged attacks) |
| 15 | Turn toward hero | 7 | Turns to face the hero | Forward within about 8 degrees of the hero |
| 16 | Turn away | 6 | Turns away from the hero (the hero is on the left, or dead ahead on the right-hand test) | Forward within about 8 degrees |
| 17 | Approach | locomotion mode 0 | Walks toward the hero | Horizontal distance to the hero is within CloseDistance |
| 18 | Orbit | locomotion modes 5 or 6 | Circles the hero at a side | Horizontal distance to the hero is farther than 18.0 units, or the hero is not found on the knight's mesh. Side flips only on body contact (see Idle movement) |
| 19 | Defeated | 43 | Plays the defeat animation every tick. The defeat test sets level state 0 once (see Defeat). | Never (terminal) |

The defeat animation (43) is the same table entry as the sword-spin recovery (state 14), and the floor-strike and high-swing recovery animation 38 is shared between the two attacks as listed above.

Phases: phase 0 uses the slow swing (state 9). A missed slow swing (or a blocked one) moves to phase 1 through state 2, where the angry swing (state 10) is used. A missed angry swing (or a blocked one) moves to phase 2 through state 3, where the enraged attacks (states 11 to 14) are used. A landed hit or the end of an enraged attack returns the phase to 0 (see Phases below). No state value of 20 or more is ever set by this controller, so those values are not reached from it (High).

### Phases

- From phase 0 the boss moves up when a swing misses: a missed slow swing (or a blocked one) sets phase 1 through the angry-start state, and a missed angry swing (or a blocked one) sets phase 2 through the enraged-start state. The controller initializes phase to 0.
- **A landed hit resets the phase to 0.** A landed angry swing sets phase to 0 (and so does not set the enraged-start flag). A landed enraged charge impact sets phase to 0. A landed slow swing leaves phase 0 unchanged. A landed kick sets phase to 0 from any phase (the kick selector has no phase condition). The high swing and floor strike hits do not change the phase on landing; they end with phase 0 anyway.
- **Every enraged attack ends with phase 0.** The charge, high swing, floor strike and sword spin each set phase to 0 when they finish. So after an enraged attack the boss is back in the slow-swing phase, and it reaches phase 2 again only after a missed angry swing. Confidence High that the writes exist, Medium that this is intended. Reproduce as stated (see Edge cases).

### Flags (every tick)

1. If the boss is ready and the laugh flag is set: clear it. With probability LaughChance / 101, go to state 1. (The roll is an integer 0 to 100 and succeeds when it is strictly less than the chance.) Whether or not the roll succeeds, the check for this tick ends, so the flags in steps 2 to 5 are not checked in that tick (High).
2. Else if the angry-start flag is set: clear it, phase = 1, go to state 2.
3. Else if the enraged-start flag is set: clear it, phase = 2, go to state 3.
4. Else if the loosen-neck flag is set: clear it. Roll against LoosenNeckChance, success goes to state 4.
5. Else if the en-garde flag is set: clear it, go to state 5, and multiply the speed multiplier by (EnGardeIncrease + 100) / 100.
The en-garde flag is set only at the end of three enraged attacks: the enraged charge (after both knees die), the enraged high swing (after both under-arms die) and the enraged floor strike (after the back dies). The sword spin does not set it. The loosen-neck flag is set only at the end of the parry state.

### Defeat (every tick)

The defeat test runs every tick right after the flag decision, before the parry and deflect checks, the attack selectors and the state handler, in every state, including the flying-sword and enraged states. It runs whether or not the boss is ready, and also on a tick where the laugh check ended the flag decision early. It does not depend on the state for its test (High).

- If all six weak-point HP values are at or below zero, the boss goes to state 19. This applies in any state. Because state 19 is busy, the attack selectors do not run in that tick, and the state handler plays the defeat animation in that same tick.
- Then, if level state 0 (bit 0 of the level's state bits) is not already set, the boss sets it. So it is set once, and the arena script reacts to it (see Arena flow) (High for the test and the set; Medium-High that this is the level state 0 that the arena script reads).
- Damage to parts is dealt in the state handler, after the defeat test. A part that reaches zero in a tick is therefore seen by the defeat test at the start of the next tick (see Edge cases).

### Parry

Checked when the boss is ready, by hero variant. The word used here is the hero index (0 = Tor, 1 = Roberta), not an attack-state value (High; SPEC-0040 Behavior 18).

- Tor (hero index 0): call the hero's attack-state predicate (SPEC-0040 Behavior 18). A false result clears the shared parry/deflect latch. A true result with the latch clear sets it and rolls against the parry accumulator. Success (roll strictly less than the accumulator): reset the accumulator to 0 and go to state 6. Failure: accumulator += ParryChanceIncrease. A true result with the latch already set does nothing.
- With the accumulator at 0, a roll can never succeed (no integer 0 to 100 is below 0). After one failure it is 50, so the chance is 50/101. After a second failure it is 100, so the chance is 100/101.
- The whole check, including the latch clear, sits inside the ready test (High): while the knight is in states 1 to 14 or 19 it does nothing. If the hero's attack starts while the knight is busy and is still running when the knight becomes ready, the latch has kept its old value; if that value is clear, the roll is made then, and if it is set (the previous attack was never seen to end), no roll is made until a ready tick sees the hero not attacking. An attack that began and ended while the knight was busy is never seen. The roll needs no distance and no facing test: any hero attack anywhere in the arena counts.
- The parry state (action 8, 20 keys, a looping clip) ends on tick T + 79 and so lasts 80 ticks; the loosen-neck flag is set on that tick, and the flag step of the next tick rolls 25/101 for the neck move (state 4, 121 ticks). The parry gives the knight no damage reduction: the stun-window tests are the only way a part loses HP anyway (Weak points).
- The parry state, when its animation finishes, sets the loosen-neck flag (Medium).

### Deflect

Roberta only (hero index 1): scan the hero's six active shot slots (SPEC-0036); the Tor attack-state predicate is not called on this branch. The same shared latch used by the Tor parry path gates the overlap test:

- On the first active slot while the shared latch is clear: set the latch. If that slot overlaps the boss and a roll is strictly less than the deflect accumulator, reset the accumulator to 0 and go to state 7 with a stance timer of a random whole number of ticks in [DeflectStanceMin x 60, DeflectStanceMax x 60] (240 to 360 ticks). If it overlaps but the roll fails, accumulator += DeflectChanceIncrease. If there is no overlap, no roll occurs. The latch remains set while any shot slot is active, so later active slots and ticks in the same uninterrupted active period do not run another overlap test.
- When no slot is active, clear the shared latch. Other hero-index values take neither branch.
- The deflect stance strike (animation 10 or 11, a two-way random pick) deals no damage to the hero (High).
- **Stance tick by tick (High).** The check at the top of the state handler is: if the stance timer is below 1 and strike 10 is not the current animation, go to idle. While strike 10 is not current (this includes the case where strike 11 was just chosen): request 9, then scan the six shot slots and remember the last armed one; if none is armed, go to idle; if one is armed and its 3D distance from the knight's position is below 10.0, pick 10 (random value 0) or 11 (value 1) and request it. While strike 10 is current: set the sticky defended word (+0xE5C) to 1 on each tick before its gate, and go to idle when its progress passes 0.99 (41 ticks after the request). The timer drops by 1 on every tick of the state except the first-line exit. Strike 11 is never recognised as a strike, so the tick after it is chosen the handler treats the state as the stance again (request 9) and picks again if a shot is still armed within 10.0; the practical result is that the strike that plays to the end is always strike 10 (cue 2 once; cue 3 for each flicker of 11).
- The sticky defended word (+0xE5C) is also set on every tick of the parry (state 6). Nothing clears it during the fight, so once the knight has parried or struck from the stance once, Roberta's quick shots bounce off it for the rest of the fight (SPEC-0036, Quick shot meets a guarding target).

### Attack selection

Only in ready states. The first selector that matches wins, and the later ones do not run because the state has changed. Every selector that chooses an attack clears the per-attack hit-landed latch and the second latch (charge, high swing, floor strike, sword spin, slow swing, angry swing, kick, counter and balcony).

1. Counter (opening kick): if the counter flag is set, clear it. Roll an integer 0 to 100; if it is strictly less than CounterAttackChance (40), go to state 8 (kick). There is no range or hittable check, so a kick can start at any distance. If the flag is set on the first ready tick, this gives a 40% chance (40 of 101 roll values) of an opening kick; the flag starts set, giving a 40% chance (40 of 101 roll values) of an opening kick. If the flag is clear, the later selectors run in the same tick as usual. The controller itself does not set the flag to 1.
2. Kick: if the hero passes the guard predicate (SPEC-0036, Guard timer and per-attacker block), the horizontal distance from the boss to the hero is less than KickAttackRange, and a roll against KickAttackChance succeeds, go to state 8.
3. Slow swing: in phase 0, if the horizontal distance from the boss to the hero is less than SlowSwingRange, go to state 9. No roll.
4. Angry swing: in phase 1, if the horizontal distance from the boss to the hero is less than AngrySwingRange, go to state 10. No roll.
5. Phase 2 group, checked in this order:
   a. Charge: requires at least one knee above zero. Let A be the x component and B the z component of the horizontal unit direction from the boss to the hero. The shared boss-to-hero helper returns this direction in x/z, forces y to zero, and puts horizontal distance in w (High; see Open question 3). If |B| is less than |A|, select controller word 25 when A > 0 and word 26 otherwise. If |B| is at least |A|, select word 27 when B > 0 and word 28 otherwise. The charge starts (state 11) only if |selected reference - A| is less than EnragedChargeRange (30). No roll. The controller initializes these references to +20, −20, +60 and −60, respectively.
   b. High swing: if the horizontal distance from the boss to the hero is less than EnragedHighSwingRange, the knight's own horizontal distance from its start point (the spawn position, (0.0, 0.0, 4.141); the vertical part is ignored) is below 10.0, and at least one under-arm is above zero, go to state 12. (High: the second length is measured between the knight's current position and the start pose stored with the character at spawn; Medium-High for the start value.) The start point does not move during the fight.
   c. Floor strike: if the back is above zero, go to state 13.
   d. Sword spin: if both knees, both under-arms and the back are all at or below zero, go to state 14.
6. Balcony: in any phase, if a roll from 0 to 100 inclusive is strictly less than BalconyAttackChance (30 of 101) and the hero's position is not found on the knight's own ground mesh, go to state 14. The position is located with the full-scan form of the AI locate (SPEC-0018, Incremental locate), using the knight's cached triangle and scan cursor; a hero standing on any triangle of that mesh is found, so the spin starts only when the hero is off the knight's walkable floor, for example on a ledge or balcony the knight cannot reach. (High for the gate and the test; Medium for the design meaning of "balcony". See Edge cases.)

### Enraged attacks

Each enraged attack is a short chain of steps. A step with an animation plays that animation; "finished" means the animation has reached its end. The stun window of each attack is a countdown of its StunTime in ticks (10.0 x 60 = 600 ticks for all four; High), and the countdown runs only while the part is exposed. Boundary (High): the countdown is set to 600 on the tick the step before the stun (stun start, arm-open, or the like) ends, and that tick has no part test. Each later stun tick first checks the countdown: if it is below 1, the stun ends on that tick with no part test; otherwise the tick runs its part tests and lowers the countdown by 1 (the charge lowers it before its part tests and the high swing after; the number of tested ticks is the same either way). So the parts are exposed on exactly 600 ticks, and the 601st stun tick is the exit tick. Damage to a part is described in Weak points.

**Enraged charge (state 11).**

| Step | Anim | What happens | Next |
| --- | --- | --- | --- |
| Run-in | 26 | Steers toward the hero at 0.25 per tick while the animation plays | Finished: Run |
| Run | 27 | Repeats each tick while both the body contact flag and the edge-contact flag are clear. The run sets no velocity: the knight moves by the clip's root motion. Animation 27 is a looping 8-key clip whose root moves 2.25 units straight forward on every key, about 33.8 units per second (High for the clip data, Medium-High for the speed) | Edge-contact flag set: Stun start (no impact). Otherwise, contact flag set: Impact |
| Impact | 28 | While any of the event bits 0x1 to 0x10 is on (frames 3 to 5 of the 11-frame clip) and the hero is not yet hit, deal EnragedChargeDamage to the hero and knock it back (see Hit windows and damage). There is no shape or distance test. Sets phase to 0 and marks the attack hit | Finished: idle |
| Stun start | 29 | Plays once; starts the 600-tick countdown | Finished: Stun |
| Stun | 30 (45 on a hit) | Knees exposed. Each tick: knee hit test, right knee first, then left (Weak points), gated as in the weak-point loop of the build guide (G9): the hero must be attacking and in front of the knight (direction dot above 0.25). While reaction 45 plays, the stun clip is not requested; hits during the reaction still count but do not restart it. The countdown falls by one each tick | Both knees at 0 or below: Knee-loss recovery. Countdown ends: Timer recovery |
| Timer recovery | 31 | Plays once. Sets phase to 0. No en-garde flag | Finished: idle |
| Knee-loss recovery | 32 | Plays once. Sets phase to 0 and sets the en-garde flag | Finished: idle |

In play, a charge that misses the hero and reaches the edge of the walkable arena stuns the knight (knees exposed); a charge that hits the hero or another body first ends in the impact and returns the boss to idle without exposing the knees. The knee-loss recovery and its en-garde flag are reached only through the edge stun (High for the rule; Medium-High that the arena's walk-mesh boundary is the arena wall).

**Edge contact** (generic to every AI-controlled character; the knight is one). Every AI-controlled character on a walk mesh runs an edge step each tick as part of its body movement, in this order:

1. If the edge-contact flag is set, its hold counter decreases by 1, and the flag clears when the counter is below 1.
2. For each edge of the character's current walk-mesh triangle that has no neighbouring triangle (a boundary edge), a vertical wall 20 units high is built on the edge. If the character's body overlaps it, the character is pushed back inside. A push sets the edge-contact flag and sets the hold counter to 120 ticks (2 s).
3. After the pushes, if the character's position is no longer on the mesh (and its body is not in its exempt state), it is moved back to its last valid position, and the flag and the 120-tick hold are set. Otherwise the current position is stored as the last valid position.

A push also sets the body contact flag for that tick. Body contact is also set by other collision pushes, for example when the body is pushed by another character (the hero). The body contact flag is cleared at the start of each body movement step. The boss controller only reads the two flags. Confidence: High for the flag, the hold, the order and the run exit; Medium for the wall geometry; Medium-High for the contact flag's other setters.

**Enraged high swing (state 12).**

| Step | Anim | What happens | Next |
| --- | --- | --- | --- |
| Windup | 33 | If weapon 2 touches the hero (event bit 0x2, frame 8 of 9) and the hero is not yet hit: deal EnragedHighSwingDamage, mark the attack hit | Hit: Follow-through. Finished without a hit: Arm-open |
| Follow-through | 34 | No stun. Plays once | Finished: idle, phase 0 |
| Arm-open | 35 | Plays once; starts the 600-tick countdown and sets level state 10 on the gate tick | Finished: Stun |
| Stun | 36 (46 as the hit reaction) | Under-arms exposed. Each tick: right-arm hit test, then left-arm hit test (an arm already at 0 or below is skipped), gated as in the weak-point loop of the build guide (G9): the hero must be in a strike-active state and on the matching side of the knight. Destroying one arm does not end the stun while the other is above 0. Level state 10 stays set during the whole stun | Countdown ends: clear level state 10, Timer recovery. Both arms at 0 or below: clear level state 10, Under-arm recovery |
| Timer recovery | 37 | Plays once. Sets phase to 0. No en-garde flag | Finished: idle |
| Under-arm recovery | 38 | Plays once. Sets phase to 0 and sets the en-garde flag | Finished: idle |

**Enraged floor strike (state 13).**

| Step | Anim | What happens | Next |
| --- | --- | --- | --- |
| Windup | 20 | If weapon 1 touches the hero (event bit 0x1, frames 7 to 9 of 10) and the hero is not yet hit: deal EnragedFloorStrikeDamage, mark the attack hit | Hit: Follow-through. Finished without a hit: Stun start |
| Follow-through | 21 | Plays once | Finished: idle, phase 0 |
| Stun start | 22 | Plays once; starts the 600-tick countdown | Finished: Stun |
| Stun | 23 (44 as the hit reaction) | Back exposed. Each tick the weak-point loop of the build guide (G9) runs: the hero must be attacking (attack-state predicate) and behind the knight (direction dot below -0.25) with the part latch clear and its weapon window open; on a hit the back loses the hero's damage value | Back at 0 or below: Back recovery. Countdown ends: Timer exit |
| Timer exit | 24 | Plays once. Sets phase to 0. No en-garde flag | Finished: idle |
| Back recovery | 38 | Plays once. Sets phase to 0 and sets the en-garde flag | Finished: idle |

**Enraged sword spin (state 14).**

| Step | Anim | What happens | Next |
| --- | --- | --- | --- |
| Spin-up | 39 | Plays once. When it ends, the sword's sub-state is set to 0 and the sword-returned flag is cleared, so each spin launches the sword once | Finished: Wait |
| Wait | 40 | The flying sword starts its cycle (attach, launch, orbit, homing, return, reset) and the spin waits until the sword-returned flag is set, which happens when the sword has come back to the hand | Flag set: Spin |
| Spin | 41 | Plays once. If any of the other five weak points (both knees, both under-arms, back) is above 0, the attack ends now: idle, phase 0. Otherwise start the neck stun | Finished: Neck stun, or the attack ends |
| Neck stun | 42 (47 as the hit reaction) | Neck exposed for the 600-tick countdown. Each tick the weak-point loop of the build guide (G9) runs: the hero must be in a strike-active state and behind the knight (direction dot below -0.25) with the part latch clear and its weapon window open; on a hit the neck loses the hero's damage value | Neck at 0 or below: Neck recovery. Countdown ends: idle, phase 0 (on the exit tick, with no recovery clip) |
| Neck recovery | 43 | Plays once. Sets phase to 0. No en-garde flag | Finished: idle |

The flying sword (below) runs alongside the sword spin, in the same tick, after the sword-spin step, from the end of spin-up to the end of the spin. Its whole flight happens during the wait (animation 40); the spin (animation 41) starts only after the sword has returned (High for the gate; Medium for this reading of the wait's end).

### Idle movement

Runs only in states 0, 15, 16, 17 and 18. The rules apply in state 0; states 15 to 18 keep their own rules:

- If the angle between the boss's forward direction and the direction to the hero is greater than the turn threshold (90 degrees): if the hero is on the boss's right (dot with right axis > 0), go to state 15 (turn toward). Otherwise go to state 16 (turn away). Turning stops when the forward direction is within about 8 degrees of the hero.
- Otherwise, if the hero's position is not found on the knight's own ground mesh on this tick, do nothing. This test uses the batched form of the AI locate (SPEC-0018, Incremental locate: the cached triangle, then up to 50 triangles per call from the knight's own cursor), so a hero on the mesh but away from the cached triangle can read as not found for a few ticks while the scan advances (High).
- Otherwise, if the horizontal distance from the boss to the hero is more than CloseDistance, go to state 17 (approach). Else if it is at most OrbitDistance, go to state 18 (orbit), choosing a random side (0 or 1) and setting the orbit timer to 0 (see Orbit side below). A distance of exactly 10.0 is "not more than" CloseDistance and "at most" OrbitDistance, so it enters the orbit.
- Approach ends when the horizontal distance from the boss to the hero is less than CloseDistance.
- Orbit side (High): entering the orbit sets the side to 0 or 1 with equal chance and the orbit timer to 0 (not 30). Every orbit tick the side may flip: if the knight's body contact flag is set and the timer is 0 or less, the side flips and the timer is set to 30; then, whenever the timer is above 0, it drops by 1 (so after a flip it reads 29 on that tick). Without body contact the side never changes, so the knight keeps circling the same way until it bumps into the hero, another body or a walk-mesh edge, and a flip can then happen at most once every 30 ticks. It ends when the horizontal distance from the boss to the hero is more than 18.0 units or the hero is not found on the knight's ground mesh (the same batched test). The orbit is entered only when no attack selector matched and the distance is 10.0 or less; in phases 0 and 1 the swing selectors match first at any distance below 10 or 11, so the orbit appears mainly late in the fight, when the phase-2 attacks are used up.
- Locomotion: approach selects locomotion mode 0. Orbit selects mode 5 on side 0 and mode 6 on side 1. The knight's monster row has Can Fly 0, so the mode is stored unchanged (the flyer remap to modes 7 to 10 of SPEC-0029, Behavior 2, never applies). The movement system sets the actual walking speed from the clip root travel (Locomotion clips, below). The boss's own code does not use CloseSpeed, so the approach speed is not set by this controller (Medium-High). Modes 1 and 2 play the same clip as mode 0. Modes 3 and 4 have no clip, and this controller never requests them (High).

**Locomotion clips.** At the start of its fight the controller fills the knight's seven locomotion slots with the shared slot setter (SPEC-0029, Behavior 2) using the values (3, 3, 3, -1, -1, 5, 4). In the setter's order these are the slots of modes 0, 1, 2, 3 and 4, then mode 6, then mode 5. Each value is an entry number of the knight's main animation table, and the entry names a clip of its model clip bank (49 clips, counted from 0). The slots are not changed afterwards. All three clips have speed byte 2, so at play rate 1.0 a clip advances a quarter of a key per tick (4 ticks per key; High). Travel per tick is the root value x 0.000977 / 4 (Medium-High, resting on the shared root-travel rule).

| Mode | Used for | Table entry | Clip | Keys (ticks) | Root value per key | Travel |
| --- | --- | --- | --- | --- | --- | --- |
| 0 (also 1, 2) | Approach (state 17) | 3 | 40 | 12 (48), loops | forward: 514, 681, 724, 676, 509, 357, 314, 602, 668, 638, 543, 543 | 6.61 units per cycle, 8.27 units/s on average, 0.0767 to 0.1768 per tick |
| 5 | Orbit, side 0 (state 18) | 4 | 38 | 10 (40), loops | sideways +717 at every key | 0.1751 per tick, 10.51 units/s |
| 6 | Orbit, side 1 (state 18) | 5 | 39 | 10 (40), loops | sideways -675 at every key | 0.1649 per tick the other way, 9.89 units/s |

The two orbit speeds differ because the two clips are separate clips with different root values. The sign of the sideways value is relative to the knight's own frame; the steering side for modes 5 and 6 is given in SPEC-0029, Behavior 2.
- Approach refreshes its navigation target to the hero's position each tick. Orbit sends a target three units to one side of the boss along the horizontal perpendicular to the boss-to-hero direction, with side selecting movement mode 5 or 6. Both then run the shared navigation step and route advancement (SPEC-0056). When the orbit target-locate query returns -1, the boss resets its movement mode and changes the target to its current position before that step. Here -1 means that this locate call produced no containing triangle: it can indicate an out-of-mesh target, a missing mesh, or an incremental batch that has not reached a matching triangle yet. It does not by itself prove that the target lies outside the mesh. [High for return meaning and fallback; Medium for exact per-tick caller cadence]

### Hit windows and damage

Confidence: High for the weapons, windows, damage and block rule; High for the flying-sword frame and the knight's attachment points (Open question 20).

- **Weapons.** The knight's melee shapes are the monster weapon cylinders of its monster row (row 0, as stored in its fight region, resource 1260; SPEC-0029 Behavior 10.2): weapon 1 radius 8.0, length 0.75; weapons 2 to 5 radius 5.0, length 1.0. Weapon N is tested only while event bit 2^(N-1) is on in the current frame of the attack clip, against each hero bone capsule (static test).
- A swing or strike deals its damage once per attack: the hit-landed latch prevents a second hit in the same attack, and it is cleared when the attack is selected.
- **Damage is unreduced.** Each value below goes straight to the hero's take-hit and is subtracted from HP unchanged: the hero's shield, armour, armour Mote and petpet factor do not reduce it (SPEC-0029 Behavior 10.5).

| Attack (animation) | Shape | Window (frames with the event bit on) | Damage | Block | Other |
| --- | --- | --- | --- | --- | --- |
| Kick (17) | weapon 5 | bit 0x10, frames 4 to 9 of 21 | KickAttackDamage (20) | no | knockback |
| Slow swing (15) | weapon 1 | bit 0x1, frames 10 to 12 of 28 | SlowSwingDamage (20) | yes | turns 0.25 per tick toward the hero |
| Angry swing (18) | weapon 1 | bit 0x1, frames 6 to 9 of 18 | AngrySwingDamage (20) | yes | |
| Charge impact (28) | none | any of bits 0x1 to 0x10, frames 3 to 5 of 11 | EnragedChargeDamage (20) | no | knockback along the boss-to-hero direction |
| High swing windup (33) | weapon 2 | bit 0x2, frame 8 of 9 | EnragedHighSwingDamage (20) | no | |
| Floor strike windup (20) | weapon 1 | bit 0x1, frames 7 to 9 of 10 | EnragedFloorStrikeDamage (20) | no | |
| Flying sword | cylinder radius 0.75, length 8.0 on the sword's own frame | orbit (once per revolution) and homing | FlyingSwordDamageA (15) | no | an orbit hit also gives the kick knockback |

- **Block (the two swings only).** A swing is blocked when the hero guards, weapon 1 touches the hero, and dot(hero facing, boss facing) < -0.5 (the two face each other). A blocked swing deals no damage (the take-hit is not called) and goes straight to its recovery animation (16 or 19). It counts as a missed swing for the phase rules. Every other attack of the knight is unblockable. The block test is made on every tick of the swing's hit window before the landed latch is looked at, so a swing that has already landed is still cut into its recovery if the hero then guards with the weapon still touching and the facings opposite; the damage already dealt stays, and the recovery's end sets the missed-swing flag (angry-start after the slow swing, enraged-start after the angry swing).
- **Knockback (kick, charge impact, flying-sword orbit hit).** The hero gets the velocity unit(9 x d + 12 x up) x 20.0, where d is the boss's facing (kick, flying sword) or the horizontal boss-to-hero direction (charge impact), and the 20.0 is the knock-back scale at 0x2DA484. The hero enters its knocked-back state (state 0x30, SPEC-0036).
- **Charge impact has no shape test.** It hits whenever its window is on, wherever the hero is.
- **Flying-sword cylinder.** The sword reuses weapon 1 of row 0 with the two words swapped: radius 0.75 (the row's length word) and length 8.0 (the row's radius word), placed on the sword's frame and tested against the hero bone capsules.
- **Bone table and attachment points (High).** The knight's fight region holds two character records and two model sets, each with its own bone table and nine attachment entries. The knight is the first record (its special-victim kind is 240). It uses the first main skeleton and the first model set, whose bone table has 44 bones; the animation clips of that skeleton also have 44 bones. The second record (the Undead Draik) uses the second model set, a 61-bone table. In the knight's bind pose bone 0 is the root at (0, 2.662, 0) and the arms run along the X axis: bone 11 is the forearm and bone 12 the wrist on the +X side, with bones 26 and 27 as their mirrors on the -X side. The entries that the weapons use are 0 to 4 (weapon N uses entry N - 1, SPEC-0005 pose entries):
  - Entry 0: bone 12, no rotation, offset (0.4928, -0.2450, 0.0392) in the bone's frame. The offset equals the local position of the hand-grip bone below the wrist, so the point is the grip: (4.626, 4.358, 0.000) in the bind pose. It is the frame of weapon 1, of the sword held in the hand, and the attach and return point of the flying sword.
  - Entry 1: bone 27, no rotation, offset (-0.25, -0.15, 0.0), the opposite hand (weapon 2).
  - Entries 2, 3 and 4: bone 0, no rotation, zero offset, the body centre (weapons 3, 4 and 5).
- **Rendered sword and axis mapping (High).** The sword is not part of the knight's body mesh. It is the slot 0 attachment model of the knight's model set (the second child model of the set, role 10), drawn in the hand and, while the sword flies, at the flying sword's frame (SPEC-0046, Second walk and attachment slots). In the model's own coordinates its long axis is +X: the pommel end is at x = -1.24, the cross-guard lies between x = 0.5 and 1.7 (2.5 across along Z and about 0.3 thick along Y), and the blade runs to the tip at x = 7.21 (up to 1.2 wide along Z and 0.2 thick along Y, narrowing at the tip). The model origin is therefore near the grip, not at the middle of the sword. The hand matrix has the axes (Z_0, Y_0, -X_0) and the origin T_0, where X_0, Y_0, Z_0 and T_0 are the rows of attachment frame 0. So model +X (pommel to tip) is Z_0, model +Y is Y_0 and model +Z is -X_0. The flying sword stores exactly these four rows as its frame: the cylinder axis (the first row) is Z_0, which is the blade direction, and the frame origin T_0 is both the cylinder centre and the model origin. The drawn sword and the hit cylinder therefore always share one frame.
- **Cylinder reach on the sword (High).** The cylinder is centred on the frame origin, with half length 4.0 and radius 0.75, so it covers model x from -4.0 to 4.0. The rendered sword spans x from -1.24 to 7.21: the hit shape covers the pommel, the grip, the cross-guard (whose tips, 1.25 from the axis, lie outside the radius) and the blade up to x = 4.0, and leaves the outer 3.21 units of the blade, including the tip, outside the hit shape. Reproduce this; do not recentre the cylinder on the visible sword.
- The weak-point tests of the four enraged attacks, which use finite cylinders, are described under Weak-point test in the enraged attacks.

### Shape tests called by the controller

Each item names its owner.

- **Weak-point test (four enraged attacks).** Owned by the four enraged attack routines. See Weak-point test in the enraged attacks.
- **Flying sword volume pass.** Owned by the flying sword. See Flying sword, World-volume pass.
- **Monster weapon test (kick, swings, high swing, floor strike).** Owned by the shared enemy framework (SPEC-0029 Behavior 10.2). See Hit windows and damage.
- Damage values: kick 20, slow swing 20, angry swing 20, charge 20, high swing 20, floor strike 20 (all from the tuning table), flying sword 15 (FlyingSwordDamageA). All unreduced by the hero's defences.
- On a charge impact, on a kick hit and on an orbiting flying-sword hit, the hero is knocked back using the 20.0 knock-back scale.
- Swing block: if the hero guards while weapon 1 touches it and the two face each other (dot of the facings < -0.5) during a slow or angry swing, the swing is cancelled into its recovery animation with no damage.

### Weak-point test in the enraged attacks (Confidence: High for the sizes, threshold, reaction animation, hit effect, latch, health change and target-cylinder construction; Medium for the part-to-frame pairing)

During the stun window of each enraged attack, the controller tests its exposed parts once per tick. The four attacks share one hit routine, and each attack passes its own cylinder size and parts:

| Attack (state) | Cylinder radius | Cylinder length | Threshold | Part (pose-frame selector) | Reaction animation |
| --- | --- | --- | --- | --- | --- |
| Enraged charge (11) | 1.5 | 2.0 | 0.5 | Right knee (selector 0x25), then left knee (selector 0x22) | 45 |
| Enraged high swing (12) | 2.0 | 2.5 | 0.5 | Right under-arm (selector 0x18), then left under-arm (selector 0x09) | 46 |
| Enraged floor strike (13) | 1.5 | 2.0 | 0.5 | Back (selector 0x04) | 44 |
| Sword spin (14) | 2.5 | 2.0 | 0.5 | Neck (selector 0x07) | 47 |

Every hit starts effect 305 (not a sound; see Effects). The health values are the boss's own six part values (see Boss variables).

Each selector in the table chooses a composed joint frame from the knight's 44-joint skeleton and the actor frame transforms its vectors. The target cylinder uses the frame's second basis vector as its axis. Its centre is the frame translation plus half the cylinder's full length along that axis, so the joint translation is the near endpoint. This establishes the cylinder construction but does not establish an anatomical label for the second basis vector or independently verify the part names. Part names remain associated by attack and health-slot order. The shared routine runs in this order, and the first success ends the test with a hit:

1. Hero body. The hero's body cylinder has radius 1.0. Its length is queried each time through the active hero's reach class query with argument zero: weapon reach W plus 2.0. A valid equipped weapon selects the hero-specific reach value; if no valid weapon is equipped, W is 1.5 and the cylinder length is 3.5. For example, Tor with item 98 has length 3.7, Tor with item 101 has 4.5, Roberta with item 111 has 3.2, and Roberta with item 114 has 3.0. Equipment changes can therefore change the cylinder during the fight. This is a dynamic query result, not the common-character motion height. The cylinder is tested against the part cylinder with the static finite-cylinder overlap of SPEC-0018.
2. Sweep. If step 1 misses and the part's two stored endpoints differ, a swept test of the part cylinder against the hero body centre runs (SPEC-0018 sweep).
3. Hero shot slots. Each of the hero's six shot slots that is armed (SPEC-0036) is tested. When the moved word is zero, the slot's point is tested directly. When it is not zero, a slot is tested only if the offset from the routine's start vector to the slot point, projected on the end vector, is at least the threshold (0.5). A slot tested this way is a point-against-sphere test with radius 5.0 plus the slot's radius. A slot that touches the part is disarmed, and the routine reports a hit.

The moved word is 1 for all four weak-point attacks: charge, high swing, floor strike and sword spin. Thus an armed hero shot slot must also pass the 0.5 projection gate in each window. The attachment points that the weapon cylinders use are listed under Hit windows and damage (Bone table and attachment points; Open question 20).

On a hit, in this order:

- The knight plays its reaction animation from the table (replacing its stun animation).
- The hero's damage query for the boss is called with argument 1.0. Its float result is rounded down to a whole number d.
- The part's health value becomes 0 if it is less than d, otherwise it becomes health minus d. A part therefore never goes below 0.
- The attack's hit-landed latch is set to 1. The latch is cleared when the attack is selected.
- Effect 305 starts once at the world position of the hit part's frame (frame kind 0x25 or 0x22 for the knees, 0x18 or 0x09 for the under-arms, 0x04 for the back, 0x07 for the neck). It is a one-shot spark burst (see Effects), restarted on each hit. No sound plays from this routine, and the reaction animations 44 to 47 carry no sound word, so the knight adds no sound of its own to a weak-point hit (see Sounds).

Confidence: High that the selectors choose composed joint frames, for the attack sizes, order, hit effects, and target-cylinder centre/axis convention. Medium for the part-to-selector pairing, which follows the attack and health-slot order. Anatomical labels for the selected joints and the semantic name of the second basis vector remain unverified.

### Weak points

- A weak point loses HP only during the stun window of the matching attack: knees (charge), under-arms (high swing), back (floor strike), neck (sword spin). Each stun window lasts the matching StunTime: 10.0 x 60 = 600 ticks for all four (High). The window is 600 ticks of exposure; the 601st tick is the exit tick (Medium).
- Each hit subtracts an amount from the part's HP. The amount is the value returned by the hero's damage query for the boss (query argument 1.0), a float rounded down to a whole number. The amount is not a tunable. The result is floored at zero, so a part never goes below 0.
- A part at or below zero is skipped by its window test and no longer takes damage. It also no longer gates its attack in step 5 of attack selection.
- The six HP values are all at or below zero: the boss is defeated.

### Flying sword

The boss has one flying-sword record (Medium-High). The sword runs only in the sword-spin state (14), once per tick right after the spin step, and only while the spin's step is not the first one (spin-up), that is from the end of spin-up until the spin ends (High for this gate). The spin handler sets the step to 1 on the tick the spin-up clip passes its gate (T + 37 for a spin entered on tick T) and the sword call follows the handler in the same tick, so the attach sub-state runs on that gate tick itself; the wait clip (40) is requested on the next tick, and the launch sub-state runs from that tick (see the timeline in the build guide, G12). It does nothing in any other state. The sword keeps a sub-state from 0 to 5 and runs one sub-state per tick. Its return (sub-state 4) sets the sword-returned flag that ends the spin's wait (High). There is no separate release flag: the flag that the spin-up clears and the wait reads only reports that the sword is back.

**Shared values.** The spin step, path rate, path angle, revolution count and side scalar are fixed shared words (see Tuning values), not per-sword values. Each launch tick resets them: revolution count 0, spin step 0.05, path rate 0, path angle 0, side scalar 3.0 (Medium-High). The per-sword values are the orbit angle, the bounce value, the sub-state and the hit latch.

Its sub-states are:

0. **Attach**: the sword copies the boss's attachment frame 0 (the hand grip point; see Bone table and attachment points) into its record and moves to launch.
1. **Launch**: the sword starts at the hand grip. Each tick it takes the vector from its position to a launch point (the knight's position plus 3.0 along the knight's forward axis plus 5.0 up; Medium for the 5.0 being a height) and its length. If the length is below 0.4 (FlyingSwordLaunchThreshold), the sword is placed on the launch point and enters orbit with the orbit angle at 0; otherwise it moves 0.4 along the unit vector (the same word is the step length) (Medium-High: the 0.4 test is on the length, not on a direction component, as an earlier reading had it). With the grip D0 units from the launch point the launch takes about D0 / 0.4 + 1 ticks. Each launch tick does the shared resets above.
2. **Orbit**: each tick, in this order:
   a. Path rate += 0.002, and spin step += 0.00075 (both shared). On the n-th orbit tick the spin step is 0.05 + 0.00075 n, so the orbit speeds up.
   b. The bounce value moves by 0.04 in its current direction; the direction reverses when its magnitude passes 0.5.
   c. Orbit angle += spin step.
   d. Path angle += path rate (shared). Because the rate grows each tick, the path angle grows quadratically with time (High). Its sine and cosine are used as coefficients on two vectors in a basis built from the sword/boss pose, then the resulting orbit offset is normalized; the angle is not a fixed world x/z heading (Medium-High for the coefficient mapping, Low for the exact world-space path). Separately, the orbit angle weights two pose-derived vectors; the vertical bounce is added, the resulting xyz direction is normalized, scaled by the side scalar (3.0), and added to the boss position as an intermediate point for the following frame construction (High for this dataflow and scalar role).
   e. If the revolution count is above 0, the sword's height moves 0.05 per tick toward its target. The target is 2.0 units above the ground height returned by a probe at the knight's current x/z, with the probe ceiling set to 8.0 units above the knight's current y; the final target is at least 4.0. "On target" means within 0.05 of it. The probe formula is High confidence; attribution of its point to the knight's current position is Medium-High.
   f. **Wrap test** (the orbit angle is at least a full turn, 2 pi):
      - If the count is below 9, or the sword is not on target, and the count is below 14: the count goes up by one, the orbit angle goes down by one full turn, and the tick ends. No homing-start test runs on this tick.
      - Otherwise (the count is at least 9 and the sword is on target, or the count is 14): the homing-start test runs.
   g. **Homing-start test** (only on the full-turn ticks that reach it): the sword leaves the orbit and starts homing when the count is at least 9. The direction test (dot product above 0.99) is OR-ed in, but it never changes the result, since the count is always at least 9 when the test runs (High).
   h. **Hit test**: if the hit latch is clear and the sword's cylinder (radius 0.75, length 8.0, on the sword's frame) touches a hero bone capsule (static overlap of SPEC-0018, no sweep), deal FlyingSwordDamageA (15), apply the kick knockback and set the latch. The latch clears on each full turn that increments the count, before the hit test on that tick, so the sword can hit once per revolution (High). High for the sizes. The cylinder is centred on the sword frame's origin and its axis follows the rendered blade's long axis (model +X from pommel to tip; Bone table and attachment points; Open question 20).
3. **Homing**: each tick the sword moves toward the hero at FlyingSwordSpeed / 60 (1.25 units per tick at the default 75), while the path angle keeps growing and its sine/cosine continue to weight a basis rebuilt from the current sword frame. The hero-directed motion and the evolving frame offset are both active; their exact combined world trajectory depends on the live pose and frame vectors. A hit test on the hero (the same cylinder) follows the latch rule above: a hit deals FlyingSwordDamageA and moves the sword to return. Contact with the scene also moves it to return. The scene query checks active collision records using a static overlap of two finite oriented cylinders; its spatial calculations use x, y and z, while the fourth lane is carried but not used for spatial length, dot or cross products. For the sword cylinder, radius is 0.75 and full axial length is 8.0 from the Black Knight's selected generic monster-table row; these are collision-shape parameters, not measured sword-mesh dimensions. The centre and axis come from the current sword pose record; the axis follows the rendered blade's long axis (model +X from pommel to tip; Open question 20). Each scene record supplies its own radius and axial extent, and a record whose extent/activity value is not positive is skipped. The authoring meaning of eligible scene records remains open. In the homing state, a miss from this cylinder query is followed by a separate segment-versus-frame-rectangle query. The segment starts at the sword pose centre and ends at centre + axis × 6.0 (0.75 times the monster row's first collision-shape dimension). The query uses candidates from the current grid cell; it accepts a plane-crossing parameter from −0.02 through 1.02 and in-plane coordinates within the rectangle bounds widened by 0.25 on each side. A hit also sends the sword to return. This query receives no separate sphere radius and tests no rounded ends; the intent of the 0.25 margins is unknown.
4. **Return**: the sword moves toward a stored point on the boss's render object at 1.25 units per tick (FlyingSwordSpeed / 60). When the remaining distance is less than 1.25 units, it sets the sword-returned flag, which ends the spin's wait, and moves to reset. The return destination and step rule are High confidence; the stored point is the position of attachment point 0, the knight's hand grip point (High).
5. **Held** (High): on every tick it runs, the sword's frame is copied from the knight's hand attachment (as in sub-state 0, but without moving on). It does not fly, test hits or touch the world volumes. It lasts for the rest of the spin, including the neck stun. When the next spin's spin-up clip (39) ends, the spin sets the sword's sub-state to 0 and clears the sword-returned flag.

Synthetic coefficient vectors (ideal trig values): at angle 0, (cos, sin) = (1, 0), so the first pose-derived basis vector supplies the offset; at π/2, (cos, sin) = (0, 1), so the second supplies it. These vectors specify the coefficient mapping only, not the basis orientation or world-space sword position.

**Homing-start timing** (single sword; orbit ticks counted from the first orbit tick, default constants): the 9th full turn completes on orbit tick 327 (5.45 s) with the count going to 9 and no homing start. The 10th completes on tick 348 (5.8 s) and starts homing if the sword is on its target height. Otherwise the count climbs by one on each later full turn, up to 14, and homing starts at the 15th full turn (tick 439, 7.3 s) at the latest. Typically the sword reaches its target height soon after the first turn, so homing starts at tick 348.

Confidence: High for the gate, the sub-state list, the return destination record and step rule, and the homing segment query inputs and bounds. Medium-High for the shared values, the homing-start rule and the sine/cosine coefficient mapping into pose-derived frames. High for the attachment represented by the return point (attachment point 0); Medium for the target-height point attribution and the hit-latch reset. High for the side scalar's role in scaling the normalized orbit offset; Low for the resulting world-space path.

### World-volume pass (flying sword)

While the flying sword is in flight, its position is also tested against the level's world volumes. This check is run by the flying sword and is the only caller of its routine. Its timing within the sword's sub-states is not established here.

- The pass reads the sword's 64-byte query record (words 0 to 15) and finds the volumes in the cell of the level grid that the record is in.
- Each volume entry whose kind word (+0x40) is 1 or 3 is tested. The test depends on the volume's class byte (at +0x10 of the volume record):
  - Class 3 is a finite cylinder (SPEC-0018). Its radius is the float at +0x3C of the volume record and its length the float at +0x24 (High for these field locations).
  - Class 1 uses a finite point-to-segment distance test against the sword query point and a segment built from the volume record (Medium; exact lane mapping and numeric threshold remain uncertain). Class 2 always reports no hit.
  - Classes 4 and 7 use one of two tests, chosen by whether the float at +0x2C is 0 or less. At 0 or below, a bounded segment is intersected with a rectangular plane: the segment parameter must be within [0, 1], and the intersection's projections must lie within two nonnegative extents. Above 0, five transformed probes are tested in sequence and stop at the first hit. The exact endpoint lane mapping and five-probe placement remain uncertain (High for branch structure; Medium for the plane-and-rectangle interpretation).
- A volume that hits fires once: a generic once-latch test on the entry's id (u16 at +0x58) must return 0. Then a sound cue of the main sound slot plays at the entry's position (+0x30). The volumes hit here are breakables, and the cue is chosen by the breakable's material, the entry's type word (short at +0x5C): 2 gives cue 143, 3 gives cue 145, 4 gives cue 146, and any other value gives cue 144. These are the breakable contact cues of SPEC-0031 (Breakable contact cues). This is the only cue the knight's own code plays; all its other sounds come from its animation table (see Sounds).
- After a volume hit, the break activation is called with strength value 0.25 and a normalized direction from the sword toward the volume, then the material-specific cue plays. The value is forwarded into the shared break-effect path; its downstream effect on spawned pieces is not established (Medium).

Confidence: High for the entry filter, the class dispatch, the class-4/7 branch structure and the cue choice. High for the class-3 cylinder radius field location; Medium for the class-1 lane mapping and the class-4/7 plane interpretation. Their numeric threshold, exact lane mapping and generated probe placements remain open (Open question 22).

### Blend (visual)

The blend is a pulsing glow on an exposed weak point (High). It is a mesh colour, not a pose.

- **Channels.** Six channels, one per weak point, in the order left knee, right knee, left under-arm, right under-arm, back, neck (High). Channel k drives the colour of model mesh slot k + 1 of the knight (meshes 1 to 6; High for the slots, Medium that each slot is the mesh on that body part).
- **Lit flag.** Each channel has a lit flag. A flag is set by the enraged attack that exposes the part, on every tick of its exposed step while that part's HP is above 0: the knees in the charge stun, the under-arms in the high-swing stun, the back in the floor-strike stun and the neck in the sword-spin stun (High). The blend runs first in the tick, reads the flags set on the previous tick, and then clears all six, so the glow follows the exposed window with a one-tick lag and stops on the first tick after it (High).
- **Pulse.** Each channel holds a colour (r, g, b, a) and a direction. While a channel is unlit, its colour is reset to (0, 0, 0, 1) and its direction to "up" every tick. While lit: going up, if r is at least 1.0 the direction turns to "down" and nothing else changes that tick; otherwise r and b each gain 0.01. Going down, if r is 0 or less the direction turns to "up" and nothing else changes; otherwise r and b each lose 0.01. g stays 0 and a stays 1 (High). So the part ramps from black to magenta (r = b) and back, about 100 ticks each way.
- **Output.** Each tick, for each channel, the mesh colour of its slot is set to the pulse colour when the channel is lit, and to (0, 0, 0, 1) when it is unlit (High). The colour is written into the slot's per-frame draw block for the frame being built (the game keeps two and alternates them each frame). A black colour leaves the knight looking normal, so the colour acts as an added glow (Medium).
- Until the stuns begin (phase 2), no channel is ever lit.

### Effects

- While the hero is in front of the boss or passes the attack-state predicate (SPEC-0040 Behavior 18), an effect-request flag is set; otherwise the trail latch is cleared.
- When the request flag is set and no trail handle exists (or the latch is clear), one swing ribbon is started from two points on the boss and the returned trail handle is retained; while the request continues with the latch set, later updates append another pair of points. When the request is absent, the latch is cleared; that branch does not explicitly clear the trail handle. The ordinary points are based on the model-frame translation and actor direction; during sword-spin state 14 they come from two stored pose vectors. The trail uses the held-weapon trail record and the material of common-object 297 (SPEC-0034, Swing trail). The exact texture artwork and live appearance are not established. Confidence High for the trail-manager path and record; Medium for the point interpretation. (Open question 14, Resolved for operational identity.)
- **Weak-point hit effect** (High for the id, the start point and the one-shot rule; Medium for the visual description). A weak-point hit (see Weak-point test) starts effect 305 of the shared effect group 3 (resource 9) once, at the world position of the hit part's frame. Effect 305 has four one-shot emitters: 3 pink model particles and 1 sprite (0.7 s), 15 white sparks (1 s), and 15 dark sprites (2 s). Every emitter is a single burst, so the effect ends by itself and the next hit starts it again. It is an effect, not a sound.
- The knight's animations start no effects: its fight region has no clip effect list entries for it (SPEC-0034).

### Sounds (Confidence: High)

**Rule.** The knight plays its voices and attack sounds only through its main animation table (SPEC-0005, SPEC-0031). Each sound entry names a cue of creature block 365 (sound group 1). When the knight's main animation id changes to k, the cue of entry k plays once, at the knight's position on block 365's slot, on the first tick of the animation: every sound entry of this table has start frame 0 and no keep-alive bit. The cue plays only while the block's slot is loaded, at the default volume and with no do-not-restart (SPEC-0031). A request of the animation that is already current starts nothing, so a held or replayed-every-tick animation plays its cue once; a new start after a different animation plays it again. The animation-sound step runs every tick except while the character's base state is 42, where it is silent. The controller itself plays no cue except the flying sword's breakable cues (see World-volume pass).

**Animation to cue** (fight region 1260, block 365; animation -> cue, with the cue's subject):

- 1 -> 0 (loosen neck); 2 -> 1 (en garde); 10 -> 2 and 11 -> 3 (deflect strikes); 12 -> 4 (laugh); 13 -> 5 (angry); 14 -> 6 (enraged start).
- 15 -> 7 (slow swing); 16 -> 8 (slow-swing block recovery); 17 -> 9 (kick); 18 -> 10 (angry swing); 19 -> 11 (angry-swing block recovery).
- Floor strike: 21 -> 12 (hit), 22 -> 13 (missed), 23 -> 14 (stunned), 24 -> 15 (recover); 25 -> 16 (weak spot destroyed).
- Enraged charge: 26 to 32 -> 17 to 23 in order (start, charge, hit hero, hit wall, stun, recover, destroyed).
- High swing: 33 to 38 -> 24 to 29 in order (attack, hit, missed, stun, recover, destroyed).
- Sword spin: 40 to 43 -> 30 to 33 in order (stand, catch, stun, destroyed).

Silent: 0 (idle), 6, 7 (turns), 8 (parry), 9 (deflect stance), 20 (floor strike start), 39 (spin-up) and 44 to 47 (the part-hit reactions). Any animation not listed is silent.

- The defeated state requests animation 43 every tick, but the id does not change, so cue 33 plays once when the defeat starts.
- **Special victim.** The knight's character record has special-victim kind 240 (SPEC-0031). A hero sword hit on the knight plays cue 34 of block 365 (its impact-special cue) in place of the sword's normal impact cue 14. No animation names cue 34.

### Arena flow (level script of world pack 989; owned by SPEC-0041)

The arena of this fight is run by the level script of world pack 989 (SPEC-0041). The boss's only link to it is level state 0, which the boss's defeat test sets. In outline (Medium-High for the steps, Medium for their order and gating):

- When the world activates, the script sets level state 2.
- Pre-fight: when level state 2 is set and level state 3 is clear, the script plays movie 46 (31.13 s) and waits for it to end, applies the hero's boss-fight setting for instance 7, stage 0, and sets level states 3 and 4.
- After level states 3 and 4 are set, it sets level states 5 and 1.
- It sets level state 6 when level state 1 is set, level state 6 is clear and level state 0 (the boss's defeat) is set.
- Post-fight: when level states 5 and 6 are set and level state 7 is clear, it applies the hero's boss-fight setting for instance 7, stage 1, fades the screen out over 0.1 s, posts a blackboard message (type 2, value 2, world 78), deactivates entity 0, sets game states 51, 298 and 86, sets world 78 as the world to return to after the movie, and plays movie 47 (32.57 s). After the movie the game goes to world 78.
- Level state 10 is set by the knight itself (not by this script) while the enraged high swing's under-arm stun runs. Thread A of the module loops an animation on arena object 2, the object whose eight locators effects 0 to 7 follow on every update, while the state is set, and stops it when the state clears (High for the writes and the script gate; the visible result is not established).
- Level state 7 is never set by this module, but the post-fight block runs once: movie 47 returns to world 78 and reloads the area (SPEC-0041, module 989; High).

## Edge cases and error handling

- **Phase reset** (reproduce it as written; Medium on intent): a landed angry swing sets phase 0, so the boss goes back to the slow swing instead of the enraged group. A landed enraged charge impact sets phase 0. Every enraged attack ends with phase 0, and phase 2 is re-entered only through a missed angry swing. After an enraged attack the next attack selection in phase 0 is the slow swing (state 9), if the hero is in its range.
- **Charge path to stun** (High for the branch): the knee stun is entered from the charge's run step only when the edge-contact flag is set. If the knight touched a walk-mesh edge in the 120 ticks before the run step, the run ends in the stun on its first checked tick. If the knight has no walk mesh, the edge flag is never set and the charge can only end in the impact, so the knees are never exposed through the charge and the en-garde flag from the knee loss is not set.
- **Opening kick:** the controller initializes the counter flag to 1. On the first ready tick, it clears the flag and rolls 0 to 100; values below 40 start a kick at any distance. The flag is not set again by this controller.
- **Per-fight handler state:** the controller record is allocated in its active stack frame and initialized before the first tick. Its weak-point HP, counter flag and charge references are per-fight values, separate from the actor's shared monster-table row.
- **Health bar quirk** (reproduce as written): the current-HP number used by the health display is LK + RK + LA + RK + B + N. The right knee's value is counted twice and the right under-arm's value is never counted. The defeat test uses all six values, so the bar can read zero while the right under-arm still has HP and the boss is not yet defeated. The bar differs from the true sum (LK + RK + LA + RA + B + N) by RK minus RA. Test case below. Confidence High that the expression is as stated; Medium that it is unintended.
- **Defeat ordering** (High): the defeat test runs before the attack selectors and the state handler in every tick. Parts lose HP in the state handler, so the last part reaching zero in tick t is seen by the defeat test in tick t + 1. In tick t the stun step still runs to its end (for example it may move to its recovery step), and in tick t + 1 the boss enters state 19, no attack is selected, and the defeat animation starts.
- **Turn threshold reversed** (Medium): the boss turns to face the hero only when the hero is more than the turn threshold, in degrees, off its forward direction. Inside that cone it approaches or orbits without turning. Whether this is intended is unconfirmed.
- **Balcony sword spin** (High for the rule): the balcony check can start the sword spin in any phase, not only phase 2, whenever the hero stands off the knight's ground mesh. After enough consecutive misses the full-scan cursor passes the end of the mesh, and the next call reports not found without testing (SPEC-0018), so one roll can pass even with the hero back on the mesh, unless the hero is on the knight's cached triangle. The sword spin's own rule (all knees, under-arms and back dead) is not checked for this entry. Reproduce the entry as stated, flag as unconfirmed.
- **Block into phase change**: a blocked slow swing counts as a missed swing and moves the boss to phase 1; a blocked angry swing moves it to phase 2.
- **Charge impact hits at any distance** (High): the impact has no shape or distance test, so a hero anywhere in the arena takes the 20 damage and the knockback when the impact clip's window (frames 3 to 5) is on and the hero has not been hit in this attack. Reproduce as stated.
- **Weapon 1 is a flat disc** (reproduce as stored): weapon 1 of row 0 is radius 8.0 and length 0.75, a wide flat disc on its attachment frame rather than a long blade, while the flying sword uses the same two words the other way round (radius 0.75, length 8.0). The stored order is what the game uses for the swings and the floor strike.
- **Boss melee ignores defences**: the knight's hits are not reduced by the hero's armour or shield; only the two swings can be blocked, and a blocked swing deals nothing.
- **Shared recovery animation**: animation 38 ends both the high swing (loss of both under-arms) and the floor strike (back loss), and animation 43 is both the sword-spin recovery and the defeat animation.
- **Revolution count and homing-start timing** (Medium-High): the count goes up on each full turn that does not start homing, so a sword whose target height is not reached keeps counting up to 14. The homing start is tested only on a full-turn tick, so a sword that is not on target at the ninth full turn does not start homing there. It starts homing at the first full turn with the count at 9 or more and the sword on target, or at the 15th full turn at the latest.
- **Sword hit once per revolution** (High): the sword's hit latch clears on each full turn that increments the count, so the sword can hit the hero once per revolution, not once per sword.
- **Accumulators do not reset between attacks** except on success, so the parry and deflect chances grow across the fight.
- **Defeat is terminal**: once in state 19, nothing returns the boss to another state, even if an HP value is restored.
- **Unexpected state values**: the rules above cover states 0 to 19 only. This controller never sets a state value of 20 or more, and behavior for other values is not established.
- **Speed multiplier** (High): it is only written by the en-garde state and nothing reads it, so en garde changes neither the knight's walking speed, its orbit speed nor its animation rate (Open question 6).

## Test vectors

Synthetic checks, hand-computed from the rules above, except the last item, which is read from the disc.

- Health display: LK 50, RK 10, LA 50, RA 0, B 50, N 100. Correct current sum = 260. As written: 50 + 10 + 50 + 10 + 50 + 100 = 270. Maximum sum = 350.
- Defeat: LK 0, RK -5, LA 0, RA 0, B 0, N 0 gives defeated. Change N to 1 and the boss is not defeated.
- Defeat timing: in tick t a neck hit takes the neck from 3 to 0 (the other five already 0). Tick t + 1: the defeat test sets state 19 and level state 0, no attack selector runs, and the defeat animation (43) plays. Tick t + 2: level state 0 is already set and is not set again.
- Parry (Tor attack-state predicate true): accumulator 0, any roll: no parry; accumulator becomes 50. On the next check, roll 49 succeeds and resets the accumulator to 0. On the following check, roll 50 fails and sets the accumulator to 50.
- Deflect: accumulator 20, one active slot overlaps, latch clear, roll 19: deflect, stance timer in [240, 360]. Roll 20: no deflect, accumulator 40. If the first active slot does not overlap, the latch still sets and later active slots are skipped until all slots are inactive and the latch clears.
- Initial controller vector: state 0, phase 0; speed multiplier 1.0; counter flag 1; part values 50/50/50/50/50/100; charge references +20/−20/+60/−60; parry and deflect accumulators and latch 0. The health display sets maximum and current HP to 350 on the first update.
- Opening counter vector: first ready tick, counter flag 1, hero 40 units away, roll 39: kick (state 8), flag cleared. Roll 40: no kick, flag cleared, later selectors run in the same tick.
- Kick range: horizontal distance 4.9 units and roll 69 with KickAttackChance 70: kick. Horizontal distance 5.0 units: no kick.
- Slow swing: phase 0, horizontal distance 9.9 units: state 9. Horizontal distance 10.0 units: not in range.
- Charge-reference vector: phase 2, a knee above 0, initialized controller references +20/−20/+60/−60. A = 0.8, B = 0.3 selects +20, and |20 - 0.8| = 19.2 < 30, so charge (state 11). A = 0.2, B = 0.9 selects +60, and |60 - 0.2| = 59.8, so no charge. A = −0.6, B = 0.1 selects −20, and |−20 + 0.6| = 19.4 < 30, so charge.
- Phase 2 order: knee above zero, |B| < |A|, and an arm above zero: charge wins (state 11). Knee above zero but |B| >= |A|, hero inside EnragedHighSwingRange, the knight within 10.0 of its start point, an arm above zero: high swing (state 12). The same case with the knight 12.0 from its start point: not a high swing; the floor strike is chosen when the back is above zero.
- Sword spin: knees and under-arms and back all at 0, neck 50: state 14 from attack selection.
- Phase reset: phase 2, the boss finishes a floor strike. Phase is now 0. With the hero 5 units in front and the boss idle, the next attack is the slow swing (state 9), not an enraged state.
- Phase re-entry: phase 0, slow swing misses: phase 1. Angry swing misses: phase 2 through state 3. Then a charge impact lands: phase 0.
- Angry swing lands: phase 0 and no enraged-start flag. Next selection in range is the slow swing.
- Weak-point damage: back HP 50, hero damage query returns 12.7: back becomes 38. Back HP 5, damage 20: back becomes 0 (not -15).
- Turn: forward-to-hero angle 100 degrees, hero on the right: state 15. Angle 80 degrees, horizontal distance 12: state 17 (approach). Angle 80 degrees, horizontal distance 9: state 18 (orbit).
- Orbit side: entering the orbit gives timer 0. With no body contact the side never flips, however long the orbit lasts. On the first tick with body contact the side flips and the timer reads 29 after that tick; contact on the next 29 ticks changes nothing; the first contact tick at which the timer is 0 flips it again (30 ticks after the first flip).
- Orbit target locate: a call returning -1 triggers the boss's fallback target (its current position) and resets the movement mode. A -1 after a partial 50-triangle batch means no hit in that batch yet; it does not establish that the requested orbit point lies outside the mesh.
- Block: during a slow swing the hero guards, weapon 1 touches it and dot(hero facing, boss facing) = -0.6: the swing is cancelled into recovery 16 and the hero's HP is unchanged. The same with dot -0.4: the hero takes 20.
- Charge impact (synthetic): the hero is 30 units away while impact frames 3 to 5 play: the hero still takes 20 and the knockback.
- Flying sword (synthetic): a hero capsule whose axis is 0.7 from the blade axis and inside its 8.0 length: hit (0.7 is below 0.75 plus the capsule radius).
- Unreduced damage (synthetic): a hero with 100 HP and full armour is kicked: HP 80.
- Charge run: contact 1, edge 0: impact. Contact 1, edge 1: stun start, no impact. Contact 0, edge 1: stun start. Both 0: keep running.
- Edge hold: edge push at tick t: flag 1, hold 120. With no further contact the flag is still 1 at tick t + 119 (hold 1) and clear at tick t + 120 (hold 0).
- Balcony: ready state, roll 29 (below 30), and the hero on no triangle of the knight's ground mesh: the sword spin starts. Roll 30: no spin. Roll 0 with the hero standing on the arena mesh: no spin.
- Flying sword gate: in the sword-spin state on the spin-up step (ticks T to T + 36) the sword does nothing. On tick T + 37 (spin-up gate, step becomes 1) it runs its attach sub-state; from T + 38 (wait step) it runs launch. In any state other than 14 it does nothing. When the return finishes, the sword-returned flag is set and the next spin tick moves from the wait to the spin (animation 41).
- Navmesh edge wall: with the Knight’s radius 1.5, edge x = 100, inward normal +x, sufficient opposite-vertex depth and edge parameter in range, the boundary reach is 1.5; a centre at x = 101 is pushed to x = 101.5. The 20-unit wall height does not gate the test. If off-mesh with lock 0, restore the saved on-mesh position; with a nonzero one-tick lock, retain the current position.\
- Tick rate: 600 ticks for a 10.0 s stun at 60.0 per second; homing 75 / 60 = 1.25 units per tick.
- Stun boundary: the knee stun-start clip ends on tick T (countdown 600, no knee test). Knee tests run on ticks T + 1 to T + 600 (600 ticks). On tick T + 601 the countdown is 0, the stun ends and no knee test runs, so a hero hit landing on that tick does no knee damage.
- Arena: level state 2 set and 3 clear: movie 46 plays, then level states 3 and 4 are set. Level states 1 and 0 set, 6 clear: level state 6 is set. Level states 5 and 6 set and 7 clear: the post-fight block runs and movie 47 plays, returning to world 78.
- Flying sword homing: horizontal distance 30 units, sword at 1.25 units per tick: about 24 ticks to close the distance (ignoring movement of the hero).
- Flying sword target height: if the ground probe returns 0, the target is 4.0; if it returns 5.0, the target is 7.0. The probe's ceiling is the knight's current y plus 8.0.
- Flying sword homing-start timing: orbit tick 327 is the 9th full turn: the count becomes 9 and homing does not start on that tick. Orbit tick 348 is the 10th full turn: homing starts if the sword is within 0.05 of its target height. Otherwise the count becomes 10, the test repeats on each later full turn, and homing starts at the 15th full turn (tick 439) at the latest.
- Flying sword hit once per revolution: a hit at orbit tick 200 deals 15 and sets the latch. A second hit before the next full turn deals nothing. The first full turn after tick 200 clears the latch.
- Flying sword second spin: the sword is still in sub-state 5 from the first spin. At the end of the second spin-up (animation 39) it is set to 0 and the sword-returned flag is cleared; on the wait step it attaches and launches again. While in sub-state 5 during the first spin's neck stun it follows the hand and deals no hit.
- Flying sword sub-states: a sword in sub-state 2 with a count below 9 whose orbit angle passes 2 pi on a tick has its count go up by one and runs no homing-start test that tick. A sword in sub-state 3 that hits the hero sets sub-state 4 and takes no second hit.
- Flying sword return: at 1.24 units from the stored boss point, it resets and sets the returned flag; at exactly 1.25 units it takes a 1.25-unit movement step instead (the reset check is strict).
- Weak-point glow: the left knee is exposed (lit) from tick 1. Tick 1's blend still sees the flag clear: colour (0, 0, 0, 1). Tick 2: (0.01, 0, 0.01, 1). Tick 101: r = b = about 0.999999 (100 steps of 0.01 in single precision stay just below 1.0, so one more step is taken). Tick 102: about 1.01. Tick 103: no change, direction turns down. Tick 104: about 1.00. The first tick after the exposed window ends: (0, 0, 0, 1).
- Weak-point glow: right under-arm at HP 0 during the high-swing stun: its channel is never lit and stays (0, 0, 0, 1) while the left under-arm pulses.
- Locomotion clips: the slot values are (3, 3, 3, -1, -1, 5, 4), so mode 0 plays table entry 3 (clip 40), mode 5 entry 4 (clip 38) and mode 6 entry 5 (clip 39). Orbit on side 0 for 40 ticks (one cycle) moves 40 x 0.1751 = 7.01 units sideways; on side 1 it moves 6.59 units the other way. Approach at the first key: 514 x 0.000977 / 4 = 0.1255 per tick.
- Charge run: 60 ticks of animation 27 with no contact move the knight about 33.8 units forward.
- Flying-sword frame in the bind pose (no animation rotation, bone 12 at (4.133, 4.603, -0.039)): attachment frame 0 has origin T_0 = (4.626, 4.358, 0.000) and axes X_0 = (1, 0, 0), Y_0 = (0, 1, 0), Z_0 = (0, 0, 1). The sword frame is (Z_0, Y_0, -X_0, T_0). The sword tip at model (7.21, 0, 0) is at world (4.626, 4.358, 7.21); a cross-guard tip at model (1.0, 0, 1.25) is at world (3.376, 4.358, 1.0). The hit cylinder is centred on (4.626, 4.358, 0.000) with axis (0, 0, 1), radius 0.75 and half length 4.0: a point of a hero bone capsule at (4.6, 4.4, 3.5) lies inside it; a point at (4.6, 4.4, 5.5) lies outside, although it is on the drawn blade (the blade reaches z = 7.21).
- Disc: entry 17 (kick) of the knight's main animation table in region 1260 has the sound word 0x0000490F: sound group 1, block 365, cue 9, start frame 0, no keep-alive. So cue 9 of block 365 plays once at the knight on the first tick of the kick. A weak-point hit looks up effect id 305 in effect group 3 (resource 9) and finds definition 444.
- Kick phase reset (synthetic): phase 2, a kick (counter or guard kick) lands on tick h: phase 0 on tick h. The same kick missing: phase stays 2.
- High-swing range (synthetic): phase 2, a knee above 0 but |B| >= |A|, hero 9 units away, an under-arm above 0, back above 0. The knight is 8.0 from (0.0, 4.141): state 12. The knight is 10.0 or more from it: state 13 (floor strike). The knight is 8.0 from it and the hero 11.0 away: state 13.
- Hero swing against the guarding knight (synthetic): Tor swings outside any stun window and the weapon overlaps a knight bone capsule: no damage, knight state unchanged, Tor goes to the sword bounce (sub-state 13), the knight joins the swing's hit list. The same swing during the floor-strike stun with Tor behind the knight (dot below -0.25) and attacking: base state 1 for the tick, no bounce, the back loses d. The same swing with Tor in front of the knight: gate fails, base state 20, bounce, no damage.
- Under-arm gate for Tor (synthetic): high-swing stun, Tor in a ground combo swing at the knight's side: gate fails (not strike-active), bounce. Tor in the plunge of a standing jump (state 7, air phase 8), at the side: gate passes (side-axis dot above 0.25 with the first under-arm slot above 0, or below -0.25 with the second above 0).
- Level state 10 (synthetic): high swing, arm-open gate on tick g: level state 10 set on g. Right under-arm already 0, left under-arm reaches 0 on tick h: level state 10 cleared on h and recovery 38 requested on h + 1. Left under-arm still above 0 when the right one reaches 0: nothing changes, the stun continues (level state 10 still set).
- Landed-swing block (synthetic): slow swing lands on tick 149 (latch set). On tick 152 the hero is guarding, the weapon still touches and dot = -0.7: recovery 16 requested on tick 153, damage stays, the angry-start flag is set at the end of the recovery.
- Hero index per tick (synthetic): Tor selected, parry accumulator 50, latch set by an attack; the hero is swapped to Roberta: the next ready tick takes the deflect branch with the same latch (an armed slot with the latch set does not roll) and the same deflect accumulator.

### Timeline test vectors (fight clock: tick 0 is the first controller tick)

Vector A, hero standing still, not attacking and not guarding, on the arena mesh, 25.0 units from the knight and inside its 90 degree forward cone; the tuning values of the disc. Random draws: R1 is the opening counter roll (tick 0), R2 the laugh roll after a landed slow swing. Numbers marked (~) depend on the approach path and carry a tolerance of a few ticks.

| Ticks | What happens |
| --- | --- |
| 0 | Counter flag cleared. R1 = 0 to 39 (40 of 101): kick, state 8, T = 0 (case A2 below). R1 = 40 to 100 (61 of 101): no kick. The kick selector does not fire (the hero is not guarding). Idle movement: distance 25 is above 10, so approach (state 17), locomotion mode 0 |
| 1 to 108 (~) | Approach: 6.61 units per 48 ticks (0.1377 per tick on average). Distance 18.5 at tick 48, 11.9 at tick 96, 10.007 at tick 108 |
| 109 (~) | Distance 9.84, below 10: the slow swing selector fires in step 5 of the tick, state 9, T = 109, clip 15 requested. The knight turns toward the hero by 0.25 per tick during the swing |
| 149 to 160 | Slow-swing hit window (T + 40 to T + 51). If weapon 1 touches the hero on tick h in this range and the hero is not guarding facing the knight, 20 damage lands on tick h, phase stays 0 and the latch is set; nothing else changes until the clip ends |
| 217 | Gate (T + 108). Landed: laugh flag set, state 0. Missed: angry-start flag set, state 0 |
| 218 | Landed: R2 = 0 to 24 (25 of 101) enters laugh (state 1, ticks 218 to 358); otherwise (the flag step ends without a state change) the selectors of the same tick run and the slow swing starts again at T = 218 if the hero is still below 10. Missed: phase 1, state 2 (angry start, clip 13), ticks 218 to 334 |
| 335 | Missed branch: ready; angry swing selector fires (distance below 11), state 10, T = 335, clip 18 |
| 359 to 374 | Angry-swing hit window (T + 24 to T + 39); a hit sets phase 0 |
| 404 | Gate (T + 69). Missed: enraged-start flag, state 0 |
| 405 to 521 | Phase 2 and state 3 (enraged start, clip 14), 117 ticks |
| 522 | First phase-2 decision. Hero direction along the world x axis (|B| < |A|): charge, T = 522, run-in 522 to 555, run clip from 556, contact flags read from 557. Hero direction along z (|B| >= |A|): in this vector the knight has walked about 15 units from its start point (25 units to the hero, stopping near 10), which is more than 10.0, so the high swing is not available; floor strike (back above 0): T = 522, hit window 550 to 558, gate 559, stun start (22) requested 560, g = 573, exposure 574 to 1173, exit 1174, recovery 24 requested 1175 and ending 1228 (next ready tick 1229). Variant with the hero starting within about 20 units, so that the knight is still within 10.0 of its start point, hero within 11 and an under-arm above 0: high swing, T = 522, hit tick 554 only, gate 555, arm-open requested 556, stun-start gate g = 581 (level state 10 set), exposure 582 to 1181, exit tick 1182 (level state 10 cleared) |

Case A2 (R1 below 40): kick state T = 0, hit window ticks 16 to 39 (weapon 5, radius 5.0 at the knight's body centre; out of reach at 25 units), gate tick 81, idle at tick 82; the approach then starts at tick 82 and every later tick of the table above is 82 later (the 109 becomes about 191).

Count check: from the slow-swing start to the first enraged decision with every swing missing is 109 + 117 + 70 + 117 = 413 ticks (109 to 521, the decision on 522).

Vector B, hero attacking (Tor), knight ready (idle) on all listed ticks, parry accumulator 0, latch clear: the hero's attack-state predicate is true on ticks 300 to 330, false on 331 to 400, true on 401 to 430. Tick 300: latch set, roll against 0 fails, accumulator 50. Ticks 301 to 330: nothing. Tick 331: latch cleared. Tick 401: latch set, roll r = 0 to 100: r < 50 (50 of 101) enters parry (state 6, T = 401, clip 8, gate 480, loosen-neck flag set on tick 480, accumulator 0); otherwise accumulator 100. Tick 481: loosen-neck roll below 25 (25 of 101) enters state 4 (clip 1, ticks 481 to 601). If the knight had been busy (for example in an angry swing) on ticks 400 and 401, nothing would have been evaluated on them.

Vector C, floor-strike weak-point window from vector A (g = 573, back 50, Tor with damage query 12.7 per hit): ticks 574 to 1173 are tested. On each tick the hero is attacking (attack-state predicate), behind the knight (direction dot below -0.25), its weapon window is open and the part latch is clear and its body cylinder overlaps the back cylinder: the back loses 12 (50 to 38), reaction 44 is requested (it plays 42 ticks, then the stun clip 23 is requested again), the latch is set. The latch clears on the first tick the gate fails, so a Tor swing that stays behind the knight for its whole window costs the back once. After the fifth hit (five separate swings; the back goes 50, 38, 26, 14, 2, 0) on tick h, back recovery 38 is requested on h + 1 and the state ends on h + 54 with the en-garde flag set. If the back survives, tick 1174 is the exit tick (no test) and recovery 24 is requested on 1175.

Vector D, flying sword (spin entered on tick T = 1000 from a balcony roll or after all five parts died; launch distance D0 = 6.0 units so launch takes 15 ticks; hero 20.0 units away): 1000 request 39; 1037 gate, attach; 1038 to 1052 launch; orbit tick n is tick 1052 + n; the 9th full turn is tick 1379 (count 9, no homing), the 10th is tick 1400 (homing starts if within 0.05 of the target height); homing 16 ticks (1401 to 1416); return about 16 ticks (1417 to 1432), flag set on 1432; 1433 step 2; 1434 request 41; gate 1459; others alive: idle at 1459 (next ready tick 1460); others dead: neck stun timer 600 set on 1459, exposure 1460 to 2059, timer exit 2060 (idle, phase 0) or recovery 43 if the neck dies.

Vector E, orbit side: the knight enters the orbit on side 0 with timer 0 at tick 500 and the hero stays 9 to 12 units away without touching the knight: the side stays 0 for the whole orbit (ends when the distance exceeds 18.0). If the knight touches a walk-mesh edge on tick 530, the side flips to 1 on 530 (timer reads 29 after the tick); a new contact on tick 540 does nothing; a contact on tick 560 (timer 0) flips it back to 0.

## Open questions

- Default if unresolved (rewrite choice): the first controller tick is the first tick after movie 46 ends; the knight is placed at (0.0, 0.0, 4.141) facing -z; the sword launch uses a point 3.0 forward and 5.0 above the knight's position and takes D0 / 0.4 + 1 ticks (clamped to 10 to 25 if the pose is not modelled); the locomotion clip starts on tick 0 and its first root step is applied on tick 1; the defeated knight slides back 0.555 units per tick until the edge step stops it.
- Default if unresolved (rewrite choice): the hero starts the fight on the arena mesh about 20 to 25 units from the knight inside its 90 degree forward cone (the timeline vectors assume 25); if the controller thread in the original runs a few ticks before movie 46, a rewrite that starts it after the movie differs by at most those ticks of approach.
- Open (Low): the exact hero start position in this arena, the first-tick pose of the locomotion clip, the launch distance D0 from the spin-up end pose, and whether the heading of the charge after the run-in is changed by anything but the run-in steering.

- Resolved (R-0067, side of root x): in modes 5 and 6 the shared steering turns the knight's move direction toward the hero (SPEC-0029, Behavior 2: with d the direction to the side target, mode 5 steers to up x d and mode 6 to d x up, which is the direction to the hero), and the root step of a clip is applied along the record's side axis, up x forward (SPEC-0040, Behavior 14C), a positive root x going along it. The table entries of these clips carry no mirror flag. With the knight facing +Z that axis is +X, which is the knight's left (SPEC-0004: the right side is -X). So the mode 5 clip, whose sideways root is positive, moves the knight to its left of the line to the hero, and the mode 6 clip, whose sideways root is negative, to its right. This is the opposite of the side point that the target names; that point only feeds the steering (High for the rule; Medium-High for what shows on screen). Test vector: boss at (0, 0, 0) and hero at (0, 0, 10): mode 5 moves toward +X and mode 6 toward -X, while the mode 5 target point is (-3, 0, 0) and the mode 6 target point is (3, 0, 0). Black Knight: orbit side 0 (mode 5, +717 per key) circles to its left, side 1 (mode 6, -675) to its right.

1. Resolved: the controller uses a per-fight stack record and initializes the six weak-point values to their maxima, the counter flag to 1, and the charge references to +20/−20/+60/−60. The character record independently selects shared monster-table row 0 for generic actor data.
2. Resolved for the flags: the parry state's completion sets the loosen-neck flag to 1; the charge, high-swing and floor-strike recovery branches set the en-garde flag to 1 when their matching weak point has been destroyed (see Flags; High). Still open: whether the walk-mesh boundary coincides with the arena wall (Medium-High). Answered: the charge run reads the walk-mesh edge-contact flag and the body contact flag (see Edge contact; High).
3. Resolved: the shared boss-to-hero helper returns horizontal unit direction in x/z, forces y to zero, and returns horizontal distance in w. The charge test calls that helper, uses x for A and z for B (Attack selection, item 5a; High). The four per-fight charge references are initialized to +20, −20, +60 and −60 (Open question 19). The second high-swing measure is the knight's horizontal distance from its start point (High; Attack selection item 5b).
4. Resolved: the sword targets 2.0 units above the ground height probed at the knight's current x/z, using a ceiling 8.0 units above its current y, with a minimum target height of 4.0. The formula is High confidence; attribution of the probe point to the knight's position is Medium-High. Return targets attachment entry 0 on bone 12, the sword grip point, and moves at 1.25 units per tick, resetting when the remaining distance is strictly less than one step (High). The exact world-space homing/orbit path remains Low. The side scalar's role is resolved: it scales the normalized pose-derived orbit offset added to the boss position (High); its disc value is 3.0. The run gate, held sub-state 5, homing start and hit reset are also answered in Flying sword.
5. Resolved (High): FlyingSwordDamageB, EnragedChargeSpeed and CloseSpeed are never read in play. Beyond the absence of any direct load or store, no read through a base address plus offset reaches them: the knight's routines that load the start of the knight tuning block read only other offsets, no routine adds a register index to that start, no word of the boot program's data points into the block, and no loader fills it from the disc. The only other way to change a word of the block is the tuning link to an external PC, which does nothing unless a PC is connected, and none is in the shipped game. The three words therefore keep their disc values and affect nothing: every flying sword hit and every homing hit deals FlyingSwordDamageA, the charge distance comes from the clip's root travel, and the approach speed is not set by this controller. Test vector: with FlyingSwordDamageB 20 or 99, a flying sword hit deals 15; with EnragedChargeSpeed 80 or 0, 60 ticks of the charge run still cover about 33.8 units; with CloseSpeed 5 or 50, the approach is unchanged.
6. Resolved (High for the controller and its helpers; Medium-High for the absence of any other reader): the speed multiplier has no consumer. The controller record is a local of the knight's per-tick routine; its address goes only to the knight's own state handlers and helpers, and none of them reads the multiplier's word. The animation play rate is set to 1.0 on every tick, and the locomotion requests carry no speed, so the multiplier reaches neither. After en garde the knight walks, orbits and swings exactly as before. Test vector: after three en garde stances the word holds 1.0 x 1.25 x 1.25 x 1.25 = 1.953125 (EnGardeIncrease 25), and the travel per tick of approach and orbit is still the clip's root travel (Locomotion clips).
7. The exact boundary of the stun window. Resolved: 600 exposed ticks after the start tick, and the 601st stun tick is the exit tick with no part test (see Enraged attacks; High).
8. The six blend values: which part each drives. Resolved: they are the glow colours of the six weak points, in the order left knee, right knee, left under-arm, right under-arm, back, neck, written to mesh slots 1 to 6 and lit only while that part is exposed with HP above 0 (see Blend; High). Whether the colour is added or multiplied in drawing is Medium.
9. Resolved (High): the value 0 or 1 is the hero index (Tor or Roberta), not an attack-state value. The Tor parry path and the Black Knight's front/effect tests call the hero attack-state predicate (SPEC-0040 Behavior 18); the Roberta deflect path scans shot slots instead. The kick selector uses the separate guard predicate, which is state 20, Roberta's spin sub-states 6/7, or a positive guard timer (SPEC-0036, Guard timer and per-attacker block). Values other than 0/1 take neither hero-specific branch and the attack-state dispatcher returns false. Still open (Low): whether any such value occurs at runtime; the shipped heroes use 0/1.
10. Resolved (High): the approach speed is the root travel of the approach clip, applied by the movement system; no word of this controller (CloseSpeed, EnragedChargeSpeed or the speed multiplier) changes it (open questions 5 and 6).
11. Whether the balcony check is intended to run in phases 0 and 1: the original has no phase check, so it runs in all phases. Whether "balcony" is the right label for the position test is Medium.
12. Whether the phase reset (see Edge cases) is intended. The spec reproduces it as written.
13. Resolved (Medium-High): no. The only writer of a tuning word at run time is the registry's link to an external PC, which has no caller in the shipped game; no disc file or level module names these words, and no host call reads or writes a tuning word. The values at the stated data addresses are the values used in play.
14. Resolved: the persistent effect update is a two-point swing ribbon from the shared trail manager, using the held-weapon trail record and common-object 297 for its material (see Effects; SPEC-0034, Swing trail). It is not an effect-definition id. The texture artwork and live appearance remain unverified. The weak-point hit separately starts particle effect 305, and the knight's sounds are the cues of its animation table (see Effects and Sounds).
15. Partly resolved: the balcony test and the idle-movement block test are both the hero's position located on the knight's own ground mesh (full-scan form for the balcony, batched form for idle movement and the orbit end; SPEC-0018, Incremental locate; High; see Attack selection item 6 and Idle movement). The shared boss-to-hero helper's direction and distance contract is resolved in Open question 3. Approach refreshes the hero-position target, orbit supplies its side-offset target, and both use shared route advancement (SPEC-0056; High; see Idle movement). Resolved: the orbit target-locate query's -1 means no containing triangle was returned by that call. A batched miss can be temporary; it does not establish that the target is outside the mesh. The boss responds by resetting its movement mode and targeting its current position before route advancement (High). Resolved: the scene query uses static finite oriented cylinder overlap and performs spatial calculations in x, y and z (High). The angle helpers return cosine and sine of radian angles and weight vectors in pose-derived orbit/path frames (High for helper identity and coefficient use). Resolved: in homing, a miss from the scene-cylinder test is followed by a segment-versus-frame-rectangle query using the sword centre to centre + axis × 6.0, inclusive parameter window −0.02 to 1.02 and 0.25 bounds margins (High for calls/limits; Medium-High for record interpretation). The flying-sword axis follows the rendered blade's long axis (Open question 20; High). The side scalar scales the normalized pose-derived orbit offset before it is added to the boss position (3.0 at launch; High; see Flying sword). Still open: exact world-space basis trajectory; scene-record authoring meaning; intent of the 0.25 margins; and separate sphere-helper contract. The return destination is attachment entry 0 on bone 12, the hand-grip point (High; see Bone table and attachment points).
16. The per-second rate. Resolved: the float 60.0 at 0x30C598, so the boss runs at 60 ticks per second (see Tuning values; High; Medium that the table index chosen at boot is always the first one).
17. Whether the knee stun is reachable. Resolved: the knight runs the edge step every tick through its body movement, so the knee stun, the knee-loss recovery and its en-garde flag are reachable (see Edge contact).
18. Resolved: the per-fight counter flag starts at 1. The first ready check clears it and makes one opening-kick roll; this controller does not set it again.
19. Resolved: the per-fight charge-reference words 25 to 28 start at +20, −20, +60 and −60, respectively. They are initialized by the controller before its first tick.
20. Resolved (High): the knight's bone table is the 44-bone table of the first model set of its fight region (the 61-bone table belongs to the Undead Draik); its weapon attachment entries 0 to 4 are bone 12 with the grip offset, bone 27, and bone 0 three times; and the flying-sword cylinder axis is the rendered blade direction (model +X, from pommel to tip), with the cylinder centred on the model origin near the grip (Hit windows and damage, Bone table and attachment points; test vector in Test vectors). The flying-sword scene cylinder uses the current centre and axis vectors from its runtime record. Its radius 0.75 and full axial length 8.0 come from the Black Knight's selected generic monster-table row; these are collision-shape parameters and do not measure the rendered sword. The related homing segment extent is 6.0. The runtime axis is the cylinder axis and equals the rendered sword's long axis. Resolved: the moved word is 1 for all four weak-point attacks, so the hero shot-slot projection gate is active for charge, high swing, floor strike and sword spin (High for the call-site value). Answered: limb volumes and sword cylinder size are in Hit windows and damage.
21. Part frames. Resolved (High): selector values 37, 34, 24, 9, 4 and 7 index the knight's composed skeleton joint frames in its 44-joint rig; the selected frame's four output vectors are transformed by the actor frame. The target cylinder's axis is the frame's second basis vector and its centre is the frame translation plus half its full length along that vector; the joint translation is the near endpoint. The association of those indices with the left knee, right knee, left under-arm, right under-arm, back and neck follows attack and health-slot order, with Medium confidence and no independent anatomical labels. The weak-point test first compares the hero's body cylinder (radius 1.0 and dynamic length from equipped weapon reach plus 2.0) against the selected target cylinder; after a miss it may sweep a changed target-frame endpoint, then checks armed projectile slots. The attack-specific target radii/lengths are 1.5/2.0 for knees, 2.0/2.5 for under-arms, 1.5/2.0 for back and 2.5/2.0 for neck. The semantic name of the target axis and the exact world-space axis through animation remain unknown. This uncertainty applies to the six weak-point tests only. The flying-sword cylinder axis is established separately in Open question 20.
22. Partly resolved: class 1 uses a finite point-to-segment distance test against the sword query point and a segment built from the volume record; exact lane mapping and numeric threshold remain uncertain (Medium). Class 2 always reports no hit. For classes 4 and 7, a nonpositive +0x2C value selects a bounded segment-to-plane test with two extent checks; a positive value selects five transformed probes tested in sequence, stopping at the first hit. The segment endpoint lanes, plane orientation and extent mapping, and generated probe placements remain uncertain (Medium). After a hit, break activation receives strength 0.25 and the normalized sword-to-volume direction; the downstream effect on spawned pieces remains unknown (Medium). Answered: the volume-pass codes are main-slot cues 143 to 146, chosen by the breakable's material (see World-volume pass; High).
23. Resolved: the post-fight block runs once. Although level state 7 is not set in the module, movie 47 returns to world 78 and reloads the area (SPEC-0041, module 989; High). The order and gating of earlier arena steps remain Medium.
24. Resolved for navmesh confinement: the knight uses the region-character boundary rule in SPEC-0056, with its motion-record radius (1.5 units). Boundary reach, the current and predicted position tests, and push/clip behavior are specified there. A boundary-edge correction or off-mesh restore sets the shared blocked flag and holds it for 120 ticks; because AI runs before confinement, its read sees the previous motion update. The knee-stun branch also requires the separate kick-allowed word; body contact alone follows the ordinary impact path. The sole ground mesh has 24 flat triangles and 18 boundary edges, spanning x=-31.333 to 31.333 and z=-55.953 to 51.796. Its collision data includes 42 vertical cylinders in perimeter-like rows and one rectangle north of the mesh; some east/west cylinders extend about 16.7 beyond the mesh x boundary. This spatial arrangement does not prove one-to-one correspondence between every edge and visible wall. The wall’s 20-unit height is not tested, so this flag does not prove overlap with the visible arena wall. An off-mesh restore is skipped only while a generic one-tick motion lock is set; no Black Knight-specific exempt state or writer is established. High for the shared rule and row radius; Medium for the arena correspondence and reachability of a correction during an ordinary charge. Also resolved for animation 27: the run moves the knight by root motion, 2.25 units forward per key, about 33.8 units per second (see Enraged attacks; High for the clip data).
25. Sound step gates: the knight is taken to be a world character like every character spawned from a level's character records, so the world-character sound rule applies (Medium-High). Two further gates of the animation-sound step, a motion-block gate and a per-character mute word, are not named.


25. Resolved (High): the weak-point test queries the active hero reach class method with argument zero. Its result is the equipped weapon reach W plus 2.0; W is selected from the active hero and weapon, with fallback 1.5 when no valid weapon is equipped. Thus the cylinder length is 3.5 unarmed, but varies with equipment (for example, Tor item 98: 3.7; Tor item 101: 4.5; Roberta item 111: 3.2; Roberta item 114: 3.0). The test queries it dynamically, so preserve the equipment-dependent value rather than substituting a fixed height. Radius remains 1.0. The query is shared with weapon reach, so the engine behavior is established while its intended character-height meaning is uncertain.
26. Resolved (High for the writes and the control flow): the knight's hero-side behaviour. It sets the unhittable word and base state 20 on every tick, writes 1.0 into its row's Blocking Angle at fight start, writes base state 1 on the ticks where a stun gate passes, and forces the hero's lock target. Hero swings are blocked and bounced except on passing-gate ticks; the build guide item G15 lists every hero action. Medium-High: the exact list of Tor swing steps that recoil (combo steps), and the reading that Tor's under-arm and neck hits need the plunge of a standing jump.
27. Resolved (High): the high swing's second distance test is the knight's horizontal distance from its start point, below 10.0 (Attack selection item 5b). The high swing's early stun end needs both under-arms at 0 or below, not the right one alone (Enraged high swing).
28. Resolved (High): a landed kick sets phase 0; the enraged high swing's under-arm stun sets level state 10 for arena object 2's animation loop (Enraged high swing; Arena flow).
29. Open (Low): whether the controller thread runs a few ticks before movie 46 (it is created with the area; the movie is started by the level script after activation), and the hero's start position. The default is in the first lines of this section.
