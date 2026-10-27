# SPEC-0015: Boss behavior: Juppie boss

- Answers: proactive, none requested (boss AI)
- Status: Draft
- Disc build: SCUS-97367 v1.00 (same fingerprint as SPEC-0001)
- Confidence: per section. Tuning values and addresses: High (every listed word is a word of the boot program data and holds its default). Attack-choice rules and cooldown tables: High. Stage flow and level flags: High for the logic, Medium for the story meaning. Stage-complete marker and death sequence: High. Locator test and locator use counter: High (both thresholds included). Head-slam sequence: High for order and for the stun re-entry rule (one draw; the counter keeps its remaining time), Medium for the meaning of the stun-loop checks. Dazed loop: High for its exit rules; the per-tick field write and its reader (the main loop's skip of the deflect pass) are High. Spit sequence: High for order and timing (the between-cycle animation reload is Medium). Spit projectile arc: High for the horizontal speed formula and the vertical-speed formula, Medium for their per-tick application, High for the unit of the life argument (ticks); its effect ids and their roles: High (the visual descriptions: Medium). Sounds (animation cues, the cues that never play): High. Head-slam hit flash 222: High for the id, Medium for the point. Stun self-damage: Medium. Head-slam window and raw damage, spit pool kind and its hero-hit rule: High.

## Summary

The Juppie boss uses two attacks: a head slam onto a point near the hero, and a spit, in which it throws a set of Juppie projectiles at the hero. It picks an attack on a timer whose length depends on its stage. Its HP fraction, checked at 5/6, 1/2 and 0, moves the fight through stages, and the level's scripts react to the same moments through level flags. A slam that lands on a stage locator can stun the boss, and the stage events mark placed locators for the level's scripts. The level's stage spawns place Tentacle characters, not Juppies: one at the fight's start, two after the stage-1 threshold and three after the stage-2 threshold (SPEC-0045, module 1217). A boss that gains HP above a stage threshold is set back to replay that stage. Its sounds come only from its animation table, and the spit glob is silent (see Sounds and effects). Times are seconds on the 60 Hz game tick (SPEC-0007): ticks = trunc(seconds x 60).

## Where it lives on the disc

- The tuning values are 32-bit words in the boot program's data. The clean side reads them at runtime from the boot executable by virtual address (SPEC-0001 gives the fingerprint). File offset = virtual address - 0xFFF80.
- All addresses below are in the 0x2DAF50 to 0x2DAFF0 range of the boot program's data.
- Locator objects with ids 101 to 106 (0x65 to 0x6A) are placed by the level (SPEC-0006). Stage events post their script events at these locators.
- "Animation n" means entry n of the boss's own animation table (SPEC-0005).
- "Distance" is the horizontal distance between the boss and the hero, in world units (see Cross-boss facts in SPEC-0014).
- "Post script event n" means set level flag n (SPEC-0014, cross-boss facts).
- Action-frame (attack-active) queries. Every non-hero boss is an AI-controlled character (only three character method tables exist: base, hero and AI-controlled character). When a boss asks the action-frame query of itself, the answer is true when any of animation event bits 0x1 to 0x10 is set on its current frame. When it asks the same query of the hero, the answer is the hero's attack-active word (event bit 0x1 of the hero's current frame, SPEC-0036). High. The boss's can-act query in the slam (sub-step 1) is this boss-side query (Medium for the identification).
- "Top-level state n" is the boss's main state number. The states used here are 1 (free) and 6 (finale).

## Layout

### Tuning values

Type: "f32" = IEEE single, "i32" = signed 32-bit integer.

| Parameter | Type | Address | Default | Meaning |
| --- | --- | --- | --- | --- |
| Head slam: min distance | f32 | 0x2DAF50 | 10.0 | Head slam only when distance is greater than this |
| Head slam: max distance | f32 | 0x2DAF54 | 50.0 | Head slam only when distance is less than this |
| Head slam: chance | i32 % | 0x2DAF68 | 50 | |
| Head slam: damage | i32 | 0x2DAF6C | 25 | Raw: not scaled, and not reduced by the hero's defences |
| Head slam: stun min (`headSlamStunMin`, registered twice) | f32 s | 0x2DAF60 | 2.0 | Minimum stun wait after a landed slam |
| Head slam: stun max | f32 s | 0x2DAF64 | 5.0 | Maximum stun wait (not in the tuning list; see Edge cases) |
| Head slam: count per stage (registered under the name `spitJuppieDamage`, which is wrong for these words) | i32 | 0x2DAFA0, 0x2DAFA4, 0x2DAFA8 | 3, 3, 3 | Stage 1, 2 and 3 values: counted hits allowed in the stun loop before recovery |
| Spit: trigger distance | f32 | 0x2DAF70 | 100.0 | Spit only when distance is less than this |
| Spit: chance | i32 % | 0x2DAF78 | 50 | |
| Spit: projectile damage (`spitJuppieDamage`) | i32 | 0x2DAF84 | 100 | Not scaled by the boss; the pool's hero-hit rule then applies the hero's shield and armour (see Damage) |
| Spit: pre-wait | f32 s | 0x2DAFB4 | 2.0 | Wait before the first spit cycle (see Edge cases) |
| Cooldown after a decision, stage 1: min | f32 s | 0x2DAF88 | 1.5 | Stages 0 and 1 use this range |
| Cooldown after a decision, stage 1: max | f32 s | 0x2DAF8C | 3.5 | |
| Cooldown after a decision, stage 2: min | f32 s | 0x2DAF90 | 0.75 | |
| Cooldown after a decision, stage 2: max | f32 s | 0x2DAF94 | 2.0 | |
| Cooldown after a decision, stage 3: min | f32 s | 0x2DAF98 | 0.25 | |
| Cooldown after a decision, stage 3: max | f32 s | 0x2DAF9C | 1.0 | |
| Dazed time after a slam | f32 s | 0x2DAFB0 | 5.0 | Length of the dazed loop (spit flag set) |

Fixed values read at runtime, not in the tuning list:

| Constant | Type | Address | Value | Meaning |
| --- | --- | --- | --- | --- |
| Slam impact radius | f32 | 0x2DAFC8 | 7.0 | A slam hits the hero only within this horizontal distance of the impact point |
| Slam push speed | f32 | 0x2DAF5C | 5.0 | Speed of the head-slam push on the hero (used) |
| Locator front cone | f32 | 0x2DAFBC | 0.9 | Minimum dot product for a stage locator to be usable (see Stage events) |
| Locator distance | f32 | 0x2DAFB8 | 10.0 | Maximum distance from the head bone to a usable stage locator |
| Dazed front cone | f32 | 0x2DAFD8 | 0.9 | Minimum dot product for a counted hit in the dazed loop |
| Dazed shape values | f32 | 0x2DAFD0, 0x2DAFD4 | 4.0, 6.0 | Size values of the mouth-area hit shape in the dazed loop |
| Stun blocker box | f32 | 0x2DAFDC, 0x2DAFE0, 0x2DAFE4 | 4.0, 1.0, 30.0 | Full sizes of the box that blocks the hero during the stun (the box reaches half of each size from its centre) |
| Stun self-damage table | i32 | 0x2DAFE8, 0x2DAFEC, 0x2DAFF0 | 4, 2, 0 | Multiplier of the hero's melee damage, indexed by the spit flag |
| Stage HP threshold, stage 1 | f32 | (fixed in the behavior) | 5/6 (0.8333334 in single precision) | See Stages |
| Stage HP threshold, stage 2 | f32 | (fixed in the behavior) | 0.5 | |
| Stage HP threshold, stage 3 | f32 | (fixed in the behavior) | 0.0 | |
| Spit count range | i32 | (fixed in the behavior) | 3 to 5, inclusive | Chosen per spit decision |
| Slam and spit turn rates | f32 | (fixed in the behavior) | 1.0 for the slam approach, 0.25 for the spit wait | Per tick |
| Spit effect ids | i32 | (fixed in the behavior) | 1400 (0x578), 1400, 1399 (0x577), -1, 1399, 1399 | The six effect ids given to the spit projectile, in pool order: in flight (two), blocked hit, lifetime expiry (none), world impact, unblocked hit (see Spit projectile); display only (High) |

Registered tuning values that the behavior does not use are listed under Edge cases. Do not apply them.

## Behavior

### Random numbers

- A percent roll is an integer drawn uniformly from 0 to 100 **inclusive** (101 values). The event with chance P happens when the roll is less than P, so the probability is P/101 (a 50 % chance is 50/101, about 49.5 %).
- A tick cooldown with range (min, max) in seconds is an integer drawn uniformly, inclusive, from trunc(min x 60) to trunc(max x 60).
- A count range (3 to 5) is an integer drawn uniformly, inclusive.
- The random generator is the shared one seeded once from the console clock (SPEC-0014).

### Variant: the spit flag

At fight start the spit flag is set when story progress flag 0x53 is set (the Act 3 marker), or when the third word of the stun self-damage table (0x2DAFF0, value 0 on disc, never written) is nonzero. The table is indexed by the flag (0 or 1), so its third entry is otherwise never read as a multiplier. No later writer is known, so the flag is fixed for the fight. With the flag: spit attacks are possible, and a landed slam leads to the dazed loop. Without it: no spit, and a landed slam leads to the stun loop.

### Stages

The boss has a stage value (1, 2, 3, or 0 for dormant). Stage 0 is dormant: the boss does not choose attacks.

**Stage start**: the level raises an event flag that moves the boss forward. The boss must be in its hub state (waiting for the script) for these flags to be read:

- Dormant to stage 1 when level flag 1 is set.
- Stage 1 to stage 2 when level flag 5 is set.
- Stage 2 to stage 3 when level flag 10 is set.

When a flag moves it to a stage, the boss enters a short stage-start state (it turns to face the hero at rate 1.0 and plays animation 5), then returns to free. The stage-complete marker is cleared at the same time.

**Stage-complete marker**: while set, the marker holds the stage number (1, 2 or 3), not just a yes or no. Cleared means no stage is marked.

**Stage complete**: the stage is marked complete when, during the stun loop or the dazed loop, the HP fraction f (current HP / max HP) is at or below the stage threshold. The attack chooser runs only while the stage is not complete. Once the boss is free with the stage complete, the stage rules run:

- **Stage 1**: if f <= 5/6, set level flag 3 and return to the hub. Otherwise (HP rose above the threshold): the stage goes back to 0, level flag 0xF is set, level flag 1 is cleared, and the boss returns to the hub.
- **Stage 2**: if f <= 0.5, set level flag 8 and return to the hub. Otherwise (HP rose above the threshold): the stage goes back to 1, level flag 0x10 is set, level flag 5 is cleared, and the boss returns to the hub to wait for the stage-start flag again.
- **Stage 3**: if f <= 0, run the death sequence (below). Otherwise (HP rose above the threshold): the stage goes back to 2, level flag 0x11 is set, level flag 10 is cleared, and the boss returns to the hub.

So a set-back means: replay the stage. The flag that started it is cleared, the flag of the set-back (0xF, 0x10 or 0x11) tells the level to re-arm that stage's locators, and the boss waits in the hub until the stage-start flag is raised again. A set-back is reachable only when HP rises above the threshold after the stage-complete marker was set. Nothing in the shipped game raises the Juppie's HP: the marker is set with the same threshold that the stage rule tests (5/6, 1/2, 0), the boss's own code only subtracts, the Juppie's level module writes no hit points, and the heal spell of the common enemies needs a Minion Wizard that this arena never holds. So the three set-back branches are unreachable in play (Medium-High; Open question "Stage numbering semantics").

### Death sequence

Used by the stun loop, the dazed loop and the stage-3 rule when the HP fraction is 0 or less. In this order:

1. Set level flag 0xE.
2. Every other living character in the world's character table whose townsfolk flag is clear takes 10000 damage. Townsfolk (characters whose default behaviour class is 6, with the townsfolk flag set; SPEC-0056) are spared. The boss itself is not hit by this rule, and characters that are already dead are skipped (High).
3. Set the boss's HP to 1.
4. Set the boss's top-level state to 6 (finale). The finale hold then plays animation 7 (see States).

### Stage events (locators)

Each stage has locators, each with its own level flag. Stage 1: locator 101 (flag 2). Stage 2: locator 102 (flag 6) and locator 103 (flag 7). Stage 3: locators 104, 105 and 106 (flags 0xB, 0xC and 0xD).

- During the slam's animation (before impact), the stage events run once per slam. For each locator of the current stage whose flag is clear, the locator is usable when both tests pass:
  - the horizontal unit direction from the boss to the locator, dotted with the boss's forward direction, is above 0.9;
  - the distance from the boss's head point (skeleton bone 15) to the locator is below 10.0.
- A usable locator is used once: its flag is set, its script event is posted there (the level's scripts decide what spawns at that locator), and the boss's use counter goes up by one. A use is reported to the slam, which sets the once-per-slam latch. The stage's locators are tried in order and each is skipped while its flag is set. Stage 1 tries locator 101. Stage 2 tries locator 102, then locator 103. Stage 3 tries 104, then 105, then 106. At most one locator is used per call.

A locator is used only when its flag is clear. The test is called only from the slam's stage events. The use counter is not checked by the slam itself; the slam only uses the reported use (Medium on the spawn meaning).

### Attack decision (cadence)

The decision runs only while the boss's top-level state is 0 or 1 (the free states), and only while the stage is active (stage 1 to 3; stage 0 never starts a decision). The cadence:

1. The chooser runs once on every pass of the boss's loop, whatever the top-level state is, and every pass on which the decision cooldown is above 0 and no attack is entered lowers it by one tick. The cooldown therefore keeps running during a head slam, a spit, the stun and dazed loops and the hub, and it can already be 0 when the boss becomes free again. Only the decision itself (steps 2 to 5) needs the free state and an active stage. When the boss is not free, or the stage is not active, or the cooldown is above 0, the pass only lowers the cooldown (not below 0) and stops (High).
2. On a free pass with an active stage and the cooldown at 0, draw a new cooldown from the stage's range: stage 1 (or stage 0) 1.5 to 3.5 s, stage 2 0.75 to 2.0 s, stage 3 0.25 to 1.0 s.
3. In the same tick, check the head slam:
   - Let D be the horizontal distance to the hero. If D is greater than 10.0 and less than 50.0, roll the head-slam chance (50). If the roll is below 50, enter the head slam (sub-step 0).
4. If no head slam was chosen, check the spit (only if the spit flag is set):
   - If D is less than 100.0, roll the spit chance (50). If the roll is below 50, enter the spit state and draw the spit count (3 to 5, inclusive).
5. Otherwise nothing happens, except that the same pass then lowers the new cooldown by one tick (a draw of 100 ticks leaves 99); the next decision comes when it runs out. When an attack is entered, the drawn value is kept unchanged on that pass and then runs down during the attack.

The cooldown is drawn at the start of each decision cycle only. The head-slam roll only happens if the distance check passes; the spit roll only happens if the spit flag is set and the distance is under 100.

### Start of the fight

When the boss's controller starts (the moment the Juppie's character is activated in its region), it sets the boss's top-level state to the hub (2) and the stage to 0, and requests animation 7 once before its first tick (High; the request is the same animation request that every later animation change uses). The hub's animation rule is: if the current animation is 7, do nothing; otherwise, if it is not 6, request 6; if it is 6 and 6 has finished (progress at least 0.99), request 7. So at the start of a fight the boss is already in animation 7 and the hub's animation 6 never plays; animation 6 plays only when the boss returns to the hub from another animation (after an attack ends, for example). No other animation request before the controller's own is known: the class switch only starts the controller, and the spawn record carries no animation field (Medium); the level script's own animation calls during the intro (SPEC-0045) end with 6 and then 7 before the boss is released, which agrees with this rule (Medium-High for the sequence with the intro, High for the controller's own request).

### States

| State | Animations | Behavior | Ends |
| --- | --- | --- | --- |
| Free (decision) | 7 | Runs the cadence above | Attack chosen |
| Free, facing | 2 | Turns toward the hero at rate 1.0 | Choice |
| Hub | 6 once, then 7 (only when the current animation is neither; see Start of the fight) | Waits for stage flags | Stage flag sets the next stage |
| Head slam | sub-steps below | Slam onto a point near the hero | See sub-steps |
| Spit | sub-steps below | Throws the spit projectiles | See sub-steps |
| Stage start | 5 | Turns toward the hero at rate 1.0 | Animation ends: free |
| Finale hold | 7 | Holds (top-level state 6) | Level flag 0x12 sets the boss to the idle hold |
| Idle hold | 0 | Holds | Never |

### Head slam (sub-steps)

Sub-step 0, approach:

- Play animation 0x15 and turn toward the hero at rate 1.0 per tick.
- When it ends: clear the slam latch, the once-per-slam latch, and the counted-hit counter of the stun loop (so each slam starts with zero counted stun hits), then go to sub-step 1.

Sub-step 1, slam:

- Play animation 0x12. While it runs (not yet ended), and only while the slam latch is clear and the boss's can-act query is true (event bit 0x1, on in frames 10 to 19 of the 39-frame clip; at clip speed byte 1 that is ticks 20 to 39 after the clip starts, SPEC-0005; Medium for the exact first tick):
  - Run the stage events once per slam (see Stage events). If they report a spawn, set the once-per-slam latch.
  - Find the impact point: the position of the boss's head bone (bone 15) in world space. If its horizontal distance to the hero is less than 7.0 (word 0x2DAFC8), set the slam latch, deal head-slam damage 25 (raw, unblockable) to the hero, push the hero (see Knockback), and play the shared hit flash, one-shot effect 222, detached, at the slam's stored strike point (taken to be this impact point; SPEC-0034; High for the id, Medium for the point). There is no weapon volume: the test is this distance only.
- When it ends:
  - If the once-per-slam latch is set: draw the stun wait from 2.0 to 5.0 s (ticks 120 to 300 inclusive). This is the only draw of the stun wait for this slam. If the spit flag is set, go to sub-step 6 (dazed). Otherwise go to sub-step 2 (stun).
  - Otherwise go to sub-step 4 (recover).

Sub-step 2, stun:

- Play animation 0x10 while the stun counter runs. The counter counts down by one tick, and it counts down only in this sub-step. When it reaches 0: go to sub-step 4 (recover).
- While stunned, two temporary blockers are placed on the hero's movement each tick: a box and a vertical cylinder. Let B be the boss's position, P the world position of the head bone (bone 15), and u the horizontal unit direction from B to P (the vertical part of P − B set to 0, then normalized).
  - Box: centre c = B + 0.5 x |P − B|h x u, where |P − B|h is the horizontal distance from B to P, so c is the horizontal midpoint between the boss's position and its head bone, at the height of B. The values are full sizes, halved about c: 4.0 along the world up axis (2.0 above and below c), 30.0 along u (15.0 either side of c, so along the line from the boss's body to its head), and 1.0 along w = normalize(up x u) = (u.z, 0, −u.x) (0.5 either side). The box is a thin wall 4 high, 30 long and 1 thick lying along the stunned boss (High).
  - Cylinder: centre P, axis the world up axis, radius 4.0 and length 10.0 (5.0 above and below P). It is tested with the finite-cylinder overlap of SPEC-0018 against the hero's body cylinder (its radius and length from the hero's motion record), and on overlap the hero is pushed out by the overlap depth (High for the shape and sizes; Medium-High for the push).
  - The shapes go into the hero's pending contact list (SPEC-0040, pending contact shapes; the box is tag 3, the cylinder tag 0). The list holds up to ten shapes; a shape is not added when ten are already held. The hero's mover resolves every held shape once and then empties the list, so the blockers act only on the ticks on which the boss adds them (High).
- Each tick, a hit on the boss counts when the hero is attacking, the hero's attack frames are active, and the hero's weapon touches the boss's hit shape (the hero's single-weapon reach test, SPEC-0029 Behavior 10.2). A counted hit: the counted-hit counter increases by 1, and the boss loses trunc(4 x trunc(hero melee damage)) HP (the table entry for the spit flag being clear; the stun loop runs only then). Then, if the HP fraction is above the stage threshold: go to sub-step 3. Otherwise: set the stage-complete marker and go to sub-step 4. If the HP fraction is 0 or less: run the death sequence.

Sub-step 3, stun count check:

- Play animation 9 once. When it ends: if the counted-hit counter is below the stage's count (3 in every stage, see Layout), return to sub-step 2 without drawing a new stun wait. The stun counter keeps the time it had left when sub-step 3 started, and it does not count down during sub-step 3, so the total stun time is the one drawn wait, however many counted hits there are. Otherwise go to sub-step 4.

Sub-step 4, recover:

- Play animation 0x11. When it ends: the boss returns to free.

Sub-step 6, dazed entry (only when the spit flag is set):

- Play animation 0x11 until it ends. Then set the dazed timer to 5.0 s (300 ticks) and go to sub-step 5.

Sub-step 5, dazed loop:

- While the dazed timer is above 0: count it down by one tick and play animation 3. The hit helper runs in its counting mode (below). Its damage is subtracted from the boss's HP. Each tick the loop also sets the per-fight "helper already ran" flag to 1. The main loop reads this flag after the state step: when it is set, the deflect-every-contact run of the hit helper is skipped for that tick, and the flag is cleared to 0 after the tick's wait (High). If the HP fraction is at or below the stage threshold, the boss sets the stage-complete marker to the current stage number and leaves the loop for its top-level free state. If the HP fraction is 0 or less, run the death sequence.
- Each tick of the dazed loop also adds the head-bone cylinder blocker of the stun (radius 4.0, length 10.0, centred on bone 15) to the hero's pending contact list; the box is not added in the dazed loop (High).
- When the timer is 0: the boss returns to free.

Hit helper (dazed loop and every other tick):

- Each of the hero's six attack slots is tested against a shape at the mouth point (bone 25; shape values 4.0 and 6.0).
- In the dazed loop, a contact counts only when the horizontal unit direction from bone 25 to the slot's position, dotted with bone 25's forward direction, is above 0.9. For each counted contact, the returned single-precision damage is converted to an integer by truncation toward zero before it is added to the tick total; the resulting integer total is subtracted from boss HP. Other contacts are deflected (High for conversion and summation order).
- Outside the dazed loop the helper runs each tick in a mode that deflects every contact. (On ticks the dazed loop runs the helper, the deflect pass is skipped.)

### Spit (sub-steps)

Sub-step 0, pre-wait:

- Play animation 0x13. Turn toward the hero at rate 0.25 per tick. A wait counter of 2.0 s (120 ticks) starts when the animation starts and counts down while it plays. When it reaches 0: go to sub-step 1.

Sub-step 1, shoot:

- Play animation 0x14. Once per cycle, when the animation's release event fires (event bit 0x20, frame 8 of the clip), spawn one spit projectile toward the hero with damage 100.
- When the animation ends: count the cycle. If the count reaches the spit count (3 to 5), go to sub-step 2. Otherwise play animation 0x16, reset the once-per-cycle latch, and repeat sub-step 1 for the next cycle. On the next tick the cycle's animation is 0x14 again, so the 0x16 reload is replaced at once; the reload does not change the timing (Medium).

Sub-step 2, recover:

- Play animation 0x16. When it ends: the boss returns to free.

Spit projectile (formulas, pool kind and life argument unit High):

- It is a projectile of the shared enemy projectile pool (SPEC-0029 Behavior 9): behaviour kind 1 (straight with gravity), visual kind 23, element 0, damage 100 (word 0x2DAF84), area radius 0 and area duration 0. Because the area duration is 0, the landing does not leave a damaging ground area: the slot ends on the tick after landing.
- It is launched from the mouth point (bone 25) toward the hero's position. Let d be the horizontal distance from the launch point to the hero and dy the height difference.
- Horizontal speed s = 0.5 - 0.4 x (1 - (d - 5) / 95)^2 units per tick. Flight time T = d / s ticks.
- Vertical speed = dy / T + 0.5 x (g / 3600) x T, where g is the game's gravity in units per second squared (SPEC-0007) and 3600 is 60 squared, matching 60 ticks per second. The projectile moves at these speeds per tick; the pool's kind-1 motion applies the gravity each tick (vertical speed minus g / 3600, then the move; SPEC-0029 Behavior 9).
- Damage 100 on contact with the hero, through the pool's hero-hit rule (SPEC-0029 Behavior 9): the hero's shield reduction and armour reduce it, and a guarding hero facing the spit takes max(1, 0.25 D) with the blocked flag. Kind 1 is not reflectable, so a shield never sends the spit back.
- Life argument: trunc(T x 60) + 1, with T in ticks (High). The pool lifetime counts ticks (SPEC-0029 Behavior 9), so the spit's life is about 60 times its flight, and the world and ground test ends the spit first (SPEC-0029 edge case "ballistic lifetime unit slip"). Reproduce the value as written (High).
- Effects (High for the ids and roles; Medium for the visual descriptions). The spit sets the pool projectile's six effect ids (SPEC-0029 Behavior 9), all group 3 definitions of the shared effect resource 9. They are display only and do not affect the hit:
  - Ids 0 and 1, the flight effects, placed on the projectile frame every drawn frame: 1400 and 1400, the in-flight glob (two copies of five looping green sprites, 2 s).
  - Id 2, played at a blocked or reflected hero hit: 1399, the splat burst (sixteen green sprites in four groups and a ten-sprite olive puff, once, 2 to 3 s).
  - Id 3, played at lifetime expiry: -1, so an expired spit shows nothing.
  - Id 4, played at a world impact (ground kinds 2 to 8 with the ignore-world flag clear, and the area kinds): 1399.
  - Id 5, played at an unblocked hero hit: 1399.
  - Visual-kind replacements of the pool: visual kinds 5 and 7 to 14 replace ids 0 and 4 with 410 and 412, and visual kind 6 replaces them with 900 and 412. The spit is visual kind 23, so its ids are used unchanged.
  - The world impact also sends cue 190 to the hero's weapon sound slot, which plays nothing (SPEC-0031). With the spit cues that never play (see Sounds), the glob is silent in flight and on impact.

### Knockback

- Head slam: the hero is pushed with K = 5.0 (0x2DAF5C) opposite the boss's forward direction: the push vector is K x normalize(−9 x f + (0, 12, 0)), where f is the boss's horizontal forward unit vector. The hero enters its knocked state. This slam-specific direction is read from the boss's facing; it does not use the hero's facing or a boss-to-hero direction.

### Damage

- Head slam: 25 raw, not scaled and unblockable. It goes straight to the hero's take-hit and is not reduced by the hero's shield, armour, armour Mote or petpet factor (SPEC-0029 Behavior 10.5).
- Spit projectile: 100, not scaled by the boss, then the pool's hero-hit rule (shield and armour apply; blocked: max(1, 0.25 D)).
- Death sequence (stage-3 rule, stun loop, dazed loop): every other living character in the world's character table whose townsfolk flag is clear takes 10000 damage. Townsfolk and the boss itself take no damage from this rule.
- Stun self-damage: a counted hit in the stun loop costs the boss 4 times the hero's melee damage (the spit flag is clear in that loop). In the dazed loop the hit helper's summed damage is subtracted directly.

### Sounds and effects (Confidence: High unless marked)

**Sound rule.** The boss plays its voices and attack sounds only through its main animation table (SPEC-0005, SPEC-0031); its controller plays no cue. Each sound entry names a cue of creature block 389 (sound group 1). When the boss's main animation id changes to k, the cue of entry k plays once, at the boss's position on block 389's slot, on the first tick of the animation: every sound entry has start frame 0 and no keep-alive bit. The cue plays only while the block's slot is loaded, at the default volume and with no do-not-restart (SPEC-0031). A request of the animation that is already current starts nothing; a new start after a different animation plays the cue again. The animation-sound step runs every tick except while the character's base state is 42, where it is silent.

**Animation to cue** (regions 1218 and 1219, identical tables; animation -> cue, with the cue's subject):

- 0 -> 8 and 1 -> 9 (deaths); 4 -> 6 (laugh); 5 -> 0 (emerge); 6 -> 1 (submerge).
- 8, 9, 13, 14 and 15 -> 7 (take damage).
- 16 (0x10, stun) -> 2 (head slam dazed); 17 (0x11, recover and dazed entry) -> 3 (head slam return); 18 (0x12, slam) and 21 (0x15, approach) -> 5 (head slam start).
- 19 (0x13, spit pre-wait) -> 10 (prepare spit); 20 (0x14, shoot) -> 11 (spit).

Silent: 3 (dazed loop), 7 (free and finale hold), 22 (0x16, spit recover), and every animation not listed (for example 2, the facing turn). Cues 4 (head slam), 12 (poison flight) and 13 (poison splat) are named by no animation and never play, so the spit glob is silent in flight and on impact.

Consequences in play:

- A head slam plays cue 5 twice: at the start of the approach (21) and again at the start of the slam (18).
- In the stun loop each return from the count check (animation 9, cue 7) to the stun (animation 16) plays cue 2 again.
- Each shoot cycle starts animation 20 again after the one-tick reload of 22, so cue 11 plays once per cycle (Medium; follows the reload rule in Spit).
- The stage start plays cue 0 and the hub plays cue 1. The finale hold (7) is silent; when level flag 0x12 moves the boss to the idle hold, animation 0 plays cue 8 (a death cue).

**Effects started by code.** The head slam's hit flash 222 (Head slam, sub-step 1) and the spit's six effect ids (Spit projectile). Both are group 3 definitions of resource 9.

**Clip effect lists** (animation-driven effects, SPEC-0034; Medium-High, because the list index follows the order of the region's effect records):

- 4, 5 and 6: effect 480 twice (30 grey model particles, once, 1 s) on attach slot 7, in positional mode 0.
- 0 and 9: effect 474 (16 brown sprites, once, 3 s; a mud splash) from 0.3 of the clip; 17: effect 474 on 0 to 0.5 of the clip; 18: effect 474 from 0.3.

## Edge cases and error handling

- Start of the fight with the intro skipped: the boss is released with animation 7 already current, so the hub requests nothing and animation 7 plays from the first tick.

- **Spit flag**: the spit is only chosen when the spit flag is set. A boss with the flag clear never spits, and a landed slam goes to the stun loop instead of the dazed loop.
- **Stun re-entry**: returning from the count check to the stun loop does not draw a new stun wait. The stun counter resumes with the time it had left, so a stun interrupted by counted hits lasts one drawn wait in total (High).
- **Counted-hit counter**: it is cleared at the start of each slam (sub-step 0), not at the stun end. It counts hits within one slam only.
- **Stun max value**: the head-slam stun maximum (5.0 s, at 0x2DAF64) is taken from an address that is not in the tuning list. The registration lists the stun-min address (0x2DAF60) twice under the same name (`headSlamStunMin`, both rows with the value 2.0). The second row is a duplicate, not the maximum, and no registered name covers 0x2DAF64. Use the addresses above.
- **Spit pre-wait**: the registered spit linger time (0x2DAF74, 2.0) is not used. The spit pre-wait uses the constant at 0x2DAFB4 (2.0 s). Use 0x2DAFB4.
- **Spit damage and head-slam counts**: the name `spitJuppieDamage` is registered for four words. Only 0x2DAF84 (100) is the spit projectile damage. The three entries at 0x2DAFA0, 0x2DAFA4 and 0x2DAFA8 (each 3) are the per-stage counts used by the stun loop, not damage; they need their own names.
- **Juppie count**: the tuning values for the minimum and maximum number of Juppies (0x2DAF7C = 1 and 0x2DAF80 = 5 on disc) are not used by the behavior. The spit count is always the fixed range 3 to 5.
- **Unused tuning values**: dazed time for act 1 (0x2DAFAC, 5.0), head-slam damage range (0x2DAF58, 20.0), and spit linger time (0x2DAF74, 2.0). Do not apply them.
- **Stage 1 above 5/6**: the stage-1 branch with f > 5/6 sets the stage back to 0 (see Stages). Reproduce it.
- **Stage 2 above 0.5**: the boss sets the stage back to 1 and waits in the hub. It does not start a spit (the spit happens only through the decision).
- **Dormant boss**: a boss in stage 0 never starts a decision.
- **Timer rounding**: all tick counts use truncation (trunc(seconds x 60)).
- **Juppie list**: the death sequence hits every living non-townsfolk character in the world's character table, not only Juppies. A townsfolk character (townsfolk flag set) is never hit. A character counts as dead when its dead flag is set.
- **Cues that never play** (High; reproduce): the head-slam cue 4 and the poison flight and splat cues 12 and 13 of block 389 are named by no animation, so the slam has no impact sound of its own and the spit glob is silent. The spit's world impact sends cue 190 to the hero weapon slot, which plays nothing. Do not add these sounds.
- **Dazed exit**: the dazed loop's HP exit goes to the top-level free state, not to the recover animation. The stun loop's HP exit goes to recover. Both are High (see Open questions).

## Test vectors

Synthetic checks on the rules, except the last item, which is read from the disc.

- Head slam with horizontal distance 10.0: not eligible (must be greater than 10.0). Distance 10.5 and roll 49: slam. Roll 50: no slam (49 < 50 only).
- Head slam with distance 50.0: not eligible. Distance 49.9 with roll 0: slam.
- Spit with distance 99.9 and the spit flag set, roll 49: spit. Distance 100.0: no spit. Spit flag clear, distance 50: no spit.
- Cooldown ticks: stage 1 -> 90 to 210 inclusive. Stage 2 -> 45 to 120. Stage 3 -> 15 to 60.
- Cooldown timing: free boss, stage 1, cooldown 0, distance 5 and the spit flag clear (no slam or spit possible): the draw is 100, and the pass ends with 99; after 99 more passes it is 0 and the next pass draws again. Same start with distance 20 and roll 10: the slam starts with the cooldown at 100; the slam lasts N passes, and the cooldown is max(0, 100 - N) when the boss is free again (0 for a slam longer than 100 ticks, so the first free pass decides at once).
- Stun wait ticks: 120 to 300 inclusive.
- **Stun re-entry timing**: the slam ends with the once-per-slam latch set and the stun wait drawn as 150 ticks. The stun counter runs 40 ticks in sub-step 2 (110 ticks left). One counted hit, with the HP fraction still above the stage threshold, moves the boss to sub-step 3 (counted-hit counter 1). Sub-step 3 plays animation 9 and does not count the stun down. The check returns to sub-step 2 (1 < 3) with 110 ticks still left, and no new wait is drawn. The stun then runs exactly 110 more ticks in sub-step 2 and the boss goes to recover. Total time counted down in sub-step 2: 150 ticks (2.5 s), not 150 plus a second draw.
- **Stun re-entry, two returns**: from the vector above, a second counted hit (counter 2, still below 3) returns to sub-step 2 again with the remaining time unchanged. A third counted hit (counter 3, not below 3) goes to sub-step 4 (recover) instead.
- **Counted-hit reset**: a new slam starts with the counted-hit counter at 0 even if the previous slam ended with counter 3.
- Spit pre-wait: 120 ticks. Spit count: each of 3, 4, 5 possible; never 2 or 6.
- Slam impact at horizontal distance 6.9 from the hero: hit for 25. At 7.0: no hit (must be less than 7.0).
- Juppie slam push: boss forward f = (1, 0, 0), K = 5.0 gives hero velocity (−3, 4, 0) before the knocked state; reversing the boss direction reverses only the horizontal component.
- Locator: a locator at 5 units in front with dot 0.95 and the head bone 9.0 from it, flag clear: used (flag set, event posted, use counter goes up by one, one use reported to the slam). Dot 0.85: not used. Head bone 10.0 from it: not used (must be below 10.0). A locator whose flag is already set is never used again.
- Stage 2 with locator 102 unusable and locator 103 usable and flag clear: 103 is used, and the call reports one use.
- Stage 2 with HP fraction 0.5: flag 8 and back to the hub. HP fraction 0.51: stage goes back to 1, flag 0x10 set, flag 5 cleared, boss waits in the hub.
- Stage 3 with HP fraction 0: death sequence (flag 0xE, other living non-townsfolk characters take 10000, boss HP set to 1, top-level state 6). HP fraction 0.01: stage goes back to 2, flag 0x11 set, flag 10 cleared.
- Stage-complete marker in stage 2 after a dazed-loop HP exit at or below the threshold: the marker holds 2.
- Stun loop with the spit flag clear and a hero melee hit of 12 damage: the boss loses 48.
- Dazed loop: starts at 300 ticks and ends after 300 ticks. In the loop a contact with dot 0.95 counts; dot 0.85 is deflected. The loop exits to the top-level free state when its HP exit fires.
- Dazed loop, one tick: the counting run of the hit helper runs once and the deflect run does not run on that tick; on the next tick outside the loop the deflect run runs again.
- Stun blocker box with centre c: it spans c.y − 2.0 to c.y + 2.0 vertically, 15.0 either side of c along its long direction and 0.5 either side across it.
- Stun blocker placement: boss at B = (0, 0, 0), head bone at P = (8, 1, 6). u = (0.8, 0, 0.6); c = (4, 0, 3); the long axis is u, the thin axis w = (0.6, 0, −0.8). The box spans y −2 to 2, from c − 15u = (−8, 0, −6) to c + 15u = (16, 0, 12) along its length, and 0.5 either side along w. The cylinder is centred on (8, 1, 6), vertical, radius 4.0, from y = −4 to y = 6.
- Cylinder blocker: hero body cylinder radius 1.0 centred 4.9 units horizontally from P and within the height span: overlap (4.9 < 4.0 + 1.0), so the hero is pushed out. At 5.1 units: no overlap.
- Spit cycle count 3: three shoot cycles, each with one projectile on its release event, then recover.
- Dazed hit-helper quantization: two counted contacts returning 0.9 and 1.9 contribute trunc(0.9) + trunc(1.9) = 1 HP total; truncating after summing would give 2, which is not the routine behavior.
- Spit life: d = 50 gives s = 0.389197 units per tick, T = 128.47 ticks and a life argument of trunc(7708.2) + 1 = 7709 ticks; the spit lands and ends long before that.
- Spit blocked by a guarding hero with no shield reduction and no armour: max(1, 0.25 x 100) = 25. Unblocked on the same hero: 100.
- Head slam on a hero with armour: still 25 (no reduction).
- Disc: an unblocked spit hit on the hero plays the pool's id 5, effect 1399, which resolves to definition 392 of effect group 3 (resource 9). A spit that reaches its lifetime plays id 3, which is -1, so nothing is shown.

## Open questions

- **Starting animation (R-0036 4)**: Resolved: animation 7, requested once by the controller when it starts, with the hub's animation 6 skipped (see Start of the fight; High). Test vector: a new fight enters the hub with the current animation 7; the hub requests nothing; when the hub is entered after an attack with another animation current (17, for example) it requests 6, and when 6 reaches 0.99 it requests 7.
- **Stage events and the spawn**: Resolved. Pack 1217 turns stage-locator flags into Tentacle hit and close scenes; the boss thresholds start two additional groups, bringing the Tentacle count from one to two and then three. See SPEC-0045, pack 1217, Stage spawns and Fight monitor (High).
- **Spit flag meaning**: the spit flag comes from the Act 3 story flag. Its meaning as "spit allowed" is the observed behavior; no other writer is known.
- **Stage numbering semantics**: Resolved (Medium-High). The set-back branches for stage 1, 2 and 3 re-stage the fight (stage back to 0, 1 or 2, flag 0xF, 0x10 or 0x11 set, the start flag 1, 5 or 10 cleared), but they cannot run: the stage-complete marker is only set when the HP fraction is at or below the same threshold that the stage rule then tests, and nothing raises the Juppie's HP between the two (see Stages). They are defensive code. The clean side should implement them as written and may treat them as never taken. Test vector: Act 1 Juppie (190 HP), stage 1: at 158 HP the fraction is 0.8316, at or below 5/6, so the marker is set and flag 3 is raised; there is no HP value that sets the marker and later fails the test.
- **Dazed loop per-fight field**: Resolved: it is the "helper already ran" flag. The main loop skips the deflect pass of the hit helper while it is set and clears it after each tick, so a dazed-loop tick runs the helper once, in counting mode (see Dazed loop; High).
- **Stun loop and dazed loop state after a stage-complete marker**: Resolved: the dazed loop's HP exit goes to the top-level free state, and the stun loop's HP exit goes to recover (sub-step 4). This is High, and the rule is written under Behavior and Edge cases. Test vectors (synthetic, stage 1, threshold 5/6 = 0.8333): in the stun loop (spit flag clear) a counted hit leaves f = 0.80: the marker becomes 1 and the boss goes to sub-step 4 (animation 0x11), then to free; once free with the marker set, f <= 5/6 sets level flag 3 and returns the boss to the hub. The same hit leaving f = 0.90 plays animation 9 and returns to the stun loop (counted hits below 3) or to recover (3 counted hits). In the dazed loop (spit flag set) a counted hit leaving f = 0.80 sets the marker to 1 and leaves the loop for the top-level free state at once, with the dazed timer abandoned; a hit leaving f = 0.90 keeps the loop going until the timer ends.
- **Projectile life unit and zero-distance boundary**: Resolved: the life argument trunc(T x 60) + 1 uses T in ticks, and the pool lifetime counts ticks (SPEC-0029 Behavior 9), so the spit lives about 60 times its flight time; the world and ground test ends it first (SPEC-0029 edge case "ballistic lifetime unit slip") (High). At release, the selector requires only horizontal distance D < 100; it has no positive-distance lower bound. The launch routine has no d = 0 guard: it normalizes the horizontal aim vector by d, computes T = d/s, then divides the vertical term by T. The exact values delivered to the projectile builder at d = 0, and their downstream pool behavior, remain open because the static trace does not settle the PS2 floating-point zero-division results.
- **Spit projectile gravity**: Resolved: the spit is a kind-1 projectile of the enemy pool, whose motion subtracts g / 3600 from the vertical speed each tick before the move (SPEC-0029 Behavior 9; High).
- **Spit damage path**: Resolved: the spit goes through the pool's hero-hit rule, so the hero's shield and armour reduce it and a block gives max(1, 0.25 D); it is not raw damage (High).
- **Stun blockers on the hero side**: Resolved: both blockers go into the hero's pending contact list, which the hero's mover resolves and empties each tick; the cylinder is tested against the hero's body cylinder with the SPEC-0018 overlap (High). The box's own resolution step is the tag-3 rule of SPEC-0040 (its internals are open there).
- **Spit display effects**: Resolved: 1400 (0x578) is the in-flight glob, set as both flight effects, and 1399 (0x577) is the splat burst, used for a blocked hit, a world impact and an unblocked hit; lifetime expiry has no effect (-1). The roles of the pool's six ids and the visual-kind replacements are given under Spit projectile (High for the ids and roles; Medium for the visual descriptions).
- **Strike point and sound step gates**: the point where the head slam's hit flash plays is taken to be the slam's impact point (Medium). The boss is taken to be a world character like every character spawned from a level's character records, so the world-character sound rule applies (Medium-High); two further gates of the animation-sound step, a motion-block gate and a per-character mute word, are not named.
- **Blocker shapes**: Resolved for the box: its three values are full sizes, halved about the centre, with 4.0 on the up axis (High). Resolved: the 30.0 runs along the horizontal direction from the boss's position to its head bone, about the midpoint of the two, and the second shape is a vertical cylinder of radius 4.0 and length 10.0 centred on the head bone (see Sub-step 2; High).
- **Finale state**: Resolved. In Act 1, pack 1217 sends level flag 18 after entity 7 plays its death animation, closes the eleven fight records, then runs the post-fight exchange and sets result flags 82 and 147. Act 3 follows the same broad order with entity 8 and result flags 278, 84, 5 and 6. Full scene order: SPEC-0045, pack 1217, Outro A and Outro B (High for the calls and order).
- **Random generator**: Resolved: the boss uses the shared Mersenne Twister (state at data address 0x39BE50), seeded once on its first draw from the BCD date and time bytes of the console's real-time clock, so the sequence differs from boot to boot (SPEC-0014; High).
- **Dazed-loop helper value**: Resolved. For each counted contact the helper asks the hero's shot routine for the damage of that shot slot against the boss; this is the hero shot rule of SPEC-0036 (Roberta shot projectile): the larger of 0 and S x C x T minus the boss's defence, where S is the damage scale stored in the shot slot. The returned single-precision value is truncated toward zero to a whole number for each contact, and the whole numbers are added into the tick total that is subtracted from the boss's HP (see Hit helper; High). The truncation is per contact, not on the sum. Test vector: two counted contacts in one tick that return 17.9 and 12.6 reduce the boss's HP by 29 (17 plus 12), not by 30; a contact that returns 0.7 reduces it by 0 and still disarms the shot.
