# SPEC-0019: Boss behavior: Werelupe King

- Answers: proactive, none requested (boss AI)
- Status: Draft
- Disc build: SCUS-97367 v1.00 (same fingerprint as SPEC-0001)
- Confidence: per section. Tuning values, phase thresholds, attack gates and cooldown rules: High (the values are words of the boot program's data). Controller start, per-tick order and state transitions: High (every state of the dispatch is described). Free-state test used by triumph and big laugh: Medium (open question 20). Hit flinch, triumph and the catch: High. Flame wave (this is the single full description; SPEC-0029 Behavior 22 refers here): High for the trigger's counter, distance band, chances, start effects, the tier value (the phase field) and the gate word (the ready flag); High for the wave index, the wait between waves and the per-ring hit flags; High for the ring hit rule; Medium-High for the ring's growth (the size curve of ring effect 415, which is High as data). Boomerang path: High for the target order, steering and catch request; Medium for the frame details. Boomerang throw end: Medium-High. Boomerang hit: High for the gate, the cylinder sizes and the first-hit return; Medium-High for the once-per-throw rule. Shared once-per-attack hit (swing, overhead counter, lunge, combo): High for the damage routing, the gate order, the guard branch and the progress test; Medium-High for where the once flag persists between ticks. Reach test: High for the cylinder sizes, the record list and the first-record rule; Medium for the distance gate's compared quantity and the sweep test's internals. Combo: High for the sequence and the hit count; Medium-High for flag persistence. Boss action-frame predicate: High (the King is an AI-controlled character; the query is the AI attack-active query). Walk-mesh lookup parameter (the owning world): Medium-High. Spin and speed: Medium-High (per tick). Jump selector and flight: High (including the sign of the gravity term). Orbit and boomerang side vector: High. Walk, orbit and hop: High for the modes, thresholds and the orbit end rule; High for the travel speeds at R = 1 (root motion of the clips). Phase speed words 1.2 and 1.4 having no effect on playback or travel: Medium-High. Animation numbers: High except where marked. Big laugh: unreachable, since its request word is never set (High). Hit windows, raw damage and the blocked take-hit: High; the unread 10.0 block value: Medium-High. Animation sounds (block 367) and the absence of code-started effects: High.

## Summary

The Werelupe King is a large melee boss. Each game tick it chooses a movement (walk toward the hero, orbit around the hero, or hop back), and it may start one of several attacks: a block, a jump, a roll, a boomerang throw, a volley of flame waves, a swing, a lunge, or a three-hit combo. It also flinches when the hero hits it and triumphs while the hero is down. Block, jump and roll are answers to the hero's own attack (a rising reaction trigger); the others are picked on the boss's own timers. Its health sets three phases. Each attack has a cooldown, a distance window and a chance that depends on the phase. Phases 2 and 3 store nominal speed values of 1.2 and 1.4 in local coroutine frame data, but those values have no known reader. The per-tick speed scale P is stored as 1.0, so animations and travel remain at the normal rate in every phase. The boss moves only by the root motion of the clips it plays, the same way the heroes and every other character do (SPEC-0056, Travel by root motion). All timings run on the 60 Hz game tick (SPEC-0007): a time given in seconds becomes ticks = trunc(seconds x 60). The flame-wave wait is given in frames directly.

## Where it lives on the disc

- The boss's tuning values are 32-bit words in the boot program's initialized data. The game reads them at runtime by virtual address, so a clean build should read the shipped defaults from the disc and not hard-code them. The boss's own values sit at data addresses 0x2DB0C8 to 0x2DB1BC. Shared constants that the boss uses sit at 0x2DB1C0 to 0x2DB1D0 (see Layout).
- "int" is a signed 32-bit integer and "float" an IEEE single. Percent values are whole numbers from 0 to 100 unless stated.
- The hero is the player's character. Distances are measured horizontally from the boss to the hero (or to the boss's home point), in world units (see Distances), except the melee reach test, which is three-dimensional. The unit scale is not confirmed (see Open questions).
- Animation numbers are entries of the boss's animation table (SPEC-0005).

## Layout

### Tuning values used by the behavior

Phase-dependent values are given as phase 1 / phase 2 / phase 3.

| Parameter | Type | Data address | Shipped value | Meaning |
| --- | --- | --- | --- | --- |
| Block chance (`BlockChance1`, `BlockChance2`, `BlockChance3`; `BlockChance3` is registered twice at the same address and value) | int % | 0x2DB0E0, 0x2DB0E4, 0x2DB0E8 | 25 / 35 / 75 | Chance to start a block when the hero attacks |
| Jump trigger distance | float units | 0x2DB0EC | 5 | Hero must be closer than this to start a jump |
| Jump begin chance | int % | 0x2DB0F0 | 5 | Jump chance at 15 units from home (start of the ramp) |
| Jump end chance | int % | 0x2DB0F4 | 80 | Jump chance at 28 units from home (end of the ramp) |
| Roll chance | int % | 0x2DB0F8 | 50 | Chance to roll when the hero attacks |
| Roll counter-attack distance | float units | 0x2DB0FC | 5 | Hero must be closer than this after a roll for a counter-attack |
| Roll counter-attack chance | int % | 0x2DB100 | 100 | Chance for that counter-attack (tested with "less than or equal", see Edge cases) |
| Overhead counter distance (developer name `BlockCounterAttackDistance`; the name says block counter, this spec keeps the overhead-counter wording) | float units | 0x2DB104 | 5 | Hero must be closer than this for an overhead counter after a block |
| Overhead counter chance (developer name `BlockCounterAttackChance`) | int % | 0x2DB108 | 25 | Chance for that overhead counter |
| Boomerang distance min | float units | 0x2DB10C | 15 | Hero distance must be greater than this |
| Boomerang distance max | float units | 0x2DB110 | 40 | Hero distance must be less than this |
| Boomerang cooldown | int s | 0x2DB114 | 8 | Time between boomerang attacks |
| Boomerang chance | int % | 0x2DB118, 0x2DB11C, 0x2DB120 | 90 / 50 / 25 | Chance per phase |
| Boomerang speed | float | 0x2DB1B8 | 0.7 | Units per tick of the boomerang's flight |
| Boomerang spin rate | float | 0x2DB1BC | 0.6 | Radians per tick of the boomerang's spin |
| Boomerang damage | int | 0x2DB194 | 5 | Damage per boomerang hit |
| Flame-wave distance min | float units | 0x2DB124 | 10 | Hero distance must be greater than this |
| Flame-wave distance max | float units | 0x2DB128 | 40 | Hero distance must be less than this |
| Flame-wave cooldown | int s | 0x2DB12C | 10 | Time between flame-wave attacks |
| Flame-wave chance | int % | 0x2DB130, 0x2DB134, 0x2DB138 | 0 / 50 / 75 | Chance per phase |
| Flame-wave wait min | int frames | 0x2DB13C | 0 | Shortest wait between waves (frames, not scaled) |
| Flame-wave wait max | int frames | 0x2DB140 | 15 | Longest wait between waves (frames, not scaled) |
| Flame-wave damage | int | 0x2DB198 | 15 | Damage per flame projectile hit |
| Swing distance | float units | 0x2DB144 | 10 | Hero distance must be less than this |
| Swing cooldown | int s | 0x2DB148 | 10 | Time between swings |
| Swing chance | int % | 0x2DB14C, 0x2DB150, 0x2DB154 | 60 / 46 / 20 | Chance per phase |
| Swing damage | int | 0x2DB190 | 10 | Damage passed to the shared once-per-attack hit by the swing |
| Lunge distance | float units | 0x2DB158 | 10 | Hero distance must be less than this |
| Lunge cooldown | int s | 0x2DB15C | 8 | Time between lunges |
| Lunge chance | int % | 0x2DB160, 0x2DB164, 0x2DB168 | 30 / 40 / 50 | Chance per phase |
| Lunge damage | int | 0x2DB19C | 10 | Damage passed to the shared hit by the lunge |
| Combo distance | float units | 0x2DB16C | 10 | Hero distance must be less than this (three-hit combo) |
| Combo cooldown | int s | 0x2DB170 | 10 | Time between combos |
| Combo chance | int % | 0x2DB174, 0x2DB178, 0x2DB17C | 10 / 20 / 30 | Chance per phase |
| Combo hit damage 1, 2, 3 | int | 0x2DB1A0, 0x2DB1A4, 0x2DB1A8 | 10 / 10 / 10 | Damage passed to the shared hit by the first, second and third combo hits |
| Walk distance | float units | 0x2DB180 | 25 | Walk toward the hero when farther than this |
| Orbit distance | float units | 0x2DB184 | 10 | Orbit when closer than this |
| Hop-back distance | float units | 0x2DB188 | 5 | Hop back when closer than this (under the conditions in Movement) |
| Hop-back chance | int % | 0x2DB18C | 50 | Chance to hop back once the hop timer has run |
| Overhead counter damage | int | 0x2DB1AC | 15 | Damage passed to the shared hit by the overhead counter |
| Phase 2 threshold | int % | 0x2DB1B0 | 75 | Phase 2 starts below this health percent |
| Phase 3 threshold | int % | 0x2DB1B4 | 37 | Phase 3 starts below this health percent |
| Phase 2 animation speed (developer name `MouthWipeAnimSpeed`; the name suggests a mouth-wipe animation, but the word is the phase-2 speed) | float | 0x2DB0D8 | 1.2 | Stored in local coroutine frame data from phase 2 on; no known reader (see Edge cases) |
| Phase 3 animation speed | float | 0x2DB0DC | 1.4 | Stored in local coroutine frame data in phase 3; no known reader (see Edge cases) |
| Big laugh chance | int % | 0x2DB0C8 | 40 | Registered, never loaded (see Edge cases) |
| Exhausted pant values | int % | 0x2DB0CC, 0x2DB0D0, 0x2DB0D4 | 0 / 20 / 40 | Registered, never loaded (see Edge cases) |

Shared constants used by the boss:

| Constant | Type | Data address | Value | Meaning |
| --- | --- | --- | --- | --- |
| Tick rate | float | 0x2DB1C0 | 60.0 | Ticks per second |
| Jump step constant | float | 0x2DB1C4 | 0.05 | Jump only. Per-tick gravity term of the air phase (subtracted from the vertical speed each air tick) and the launch factor: the launch vertical speed is tick rate x 0.05 x 0.5 = 1.5 units per tick |
| Boomerang timeout | float s | 0x2DB1C8 | 10.0 | Boomerang flight limit (600 ticks) |
| Held-boomerang offset | float | 0x2DB1CC | 1.1 | Distance of the held boomerang along the hand axis |
| Far steering weight | float | 0x2DB1D0 | 0.1 | Boomerang steering weight when the target is far |

Fixed values written in the behavior itself (not tunables):

- Turn weight 0.05 per tick in idle, flame wave and block. This is a fixed value in the behavior, not the jump step constant above.
- Turn weight 0.2 per tick in the jump's launch and air phases. The landing phase does not turn.
- The jump ramp runs over home distances from 15 to 28 units, and continues above 28 (see Attack selection).
- Jump landing: within 3 units of home, measured in three dimensions.
- The orbit target is 5.0 units to the side (re-aimed every tick, so it is never reached).
- The hop timer must pass 60 ticks (see Movement).
- The boomerang's waypoints use the constants 0.5 and 0.4 (see Boomerang flight).
- The boomerang hit uses a finite cylinder of radius 2.5 units and length 0.25 units (see Boomerang flight).
- The melee reach test uses a distance gate of 65.0 units, and a finite cylinder of radius 2.5 units and length 7.0 units, starting 1.5 units behind the weapon anchor (see Hit test and blocking).
- The guard test blocks when the dot product of the facing directions is at most -0.5. A blocked hit calls the hero's take-hit with 0 damage and the blocked flag; the float 10.0 that goes with it is not read by the hero's block reaction.
- An attack's hit window ends when its animation reaches 0.99 progress (see Hit window).

## Distances

Every distance from the boss to the hero, or from the boss to its home point, is measured in the horizontal plane: sqrt(dx^2 + dz^2). Height is ignored. Unit directions used for facing and steering are horizontal (except the boomerang's own flight, which is three-dimensional). High. The one exception is the melee reach test of the shared hit: its distance gate uses the full three-dimensional length (see Hit test and blocking).

## Behavior

### Controller start (first run only) (Confidence: High for the order; Medium-High for the lookup parameter)

The boss's controller runs once per tick. Its first run does this, in order, before the first tick of behavior:

1. Navigation mesh (SPEC-0018 describes the navigation mesh). Read the boss's current mesh. If none is attached, print a diagnostic line. Then search the level's mesh records, in order, for the first record that passes the record's flag test and has an entry that contains the boss's current position. The search runs over the walk-mesh list of the boss's owning world (the world set on the character when it is placed; when none is set, the current world), and takes a mesh of the walking kind (Medium-High). If a record is found, attach to it: the boss's current mesh becomes that record, the boss's position becomes the attach point, and a success line is printed. If none is found, print a failure line and do not attach. The search is not repeated on later ticks.
2. Locomotion slots. Mode 0, 1 and 2 play animation 2; mode 3 and 4 play none; mode 5 (orbit right) plays animation 3; mode 6 (orbit left) plays animation 4.
3. Blackboard defaults: state idle, sub-animation 0, ready flag 1, big-laugh request word 0, catch request word 0, phase 1 (phase field 0), flame-wave hesitation counter -1, speed 1.0, home point = the boss's start position, jump ramp bounds 15 and 28 units, and an empty flame-projectile pool (five records, all inactive).

### Each tick, in order (Confidence: High for the order; Medium-High for the detail of the pre-check)

Each later tick does the following in order:

1. The class loop initializes its local speed value to 1.0 and stores it to the boss's per-tick speed scale P. Phase handlers separately store 1.2 or 1.4 in local frame data; those entries have no known reader and do not affect playback or travel (see Edge cases).
2. Pre-check. The steps below run in this order. A later step can replace the result of an earlier one in the same tick.
   a. Hit flinch. If the boss's reaction word is "hit" (value 10): clear the word (the shared AI-mode setter called with mode 0, which only writes 0 to the word and changes nothing else), go to hit flinch, and pick flinch animation 0x0B, 0x0C or 0x0D with probability 1/3 each. This step has no state gate.
   b. Catch. If the catch request word is 1: clear it and go to catch (animation 0x14).
   c. Big laugh. If the big-laugh request word is 1 and the state is free: go to big laugh. The word starts at 0 and is only ever cleared, so this never happens (see Edge cases).
   d. Triumph. If the hero is down and the state is free and the state is not already triumph: go to triumph (animation 0x12).
   e. Health. Let f = current health / maximum health. If f <= 0: go to death, every tick. Else if f < 0.37 and the phase is not 3 and the state is not already the phase 3 entry: go to the phase 3 entry. Else if f < 0.75 and the phase is not 2 and the state is not already the phase 2 entry: go to the phase 2 entry. See Phases for the phase-2 test.
3. Reaction (on the tick the hero's reaction trigger rises only; see Attack selection). If the state is free, run the block choice. Then, only if the state is still free, the jump choice. Then, only if the state is still free, the roll choice.
4. Attacks, in this order: boomerang, flame wave, swing, lunge, combo. Each runs only when its gate allows (see Attack selection).
5. Movement: walk, hop back, then orbit (see Movement).
6. Run the current state's behavior (see States).
7. Update the boomerang flight and the flame-wave projectiles (see Boomerang flight and Flame-wave volley). A debug text routine also runs each tick; it has no gameplay effect and is not specified here. Then wait one tick.

A later choice in the same tick sees the state set by an earlier one, so an earlier choice blocks the later ones.

### Hit flinch (any state)

When the hero's attack damages the boss, the shared monster hit handler applies the damage and sets the boss's generic "was hit" mode (the reaction word). On the next tick the boss enters the hit-flinch state, whatever it was doing (the flinch has no state gate). It plays one of the flinch animations 0x0B, 0x0C or 0x0D, each with probability 1/3, and returns to idle when the animation ends. The catch, big laugh, triumph and phase and death checks of the same tick run after the flinch and can replace it. A flinch can interrupt a jump, a volley or an orbit.

### Triumph

While the hero is down (knocked out), the boss, if its state is free, enters the triumph state and plays animation 0x12 once. It returns to idle when the animation ends. While the triumph is running it is not restarted. If the hero is still down on the next tick after the triumph has ended, the triumph starts again.

### Phases and health thresholds (Confidence: High)

Let f = current health / maximum health, as a floating-point number. The boss starts in phase 1. The phase is kept in one phase field of the controller record with the values 0, 1 and 2 for phases 1, 2 and 3: it is 0 at start, the phase-2 entry writes 1 and the phase-3 entry writes 2. The "phase is not 3" and "phase is not 2" tests below read this field (values 2 and 1), and the flame-wave chance index is the same field (High).

- If f <= 0: the boss enters the death state. This check runs every tick before the state is run, even while another state is active.
- Else if f < 0.37 and the boss is not in phase 3 and is not already entering phase 3: it enters the phase 3 entry.
- Else if f < 0.75 and the boss is not in phase 2 and is not already entering phase 2: it enters the phase 2 entry.

The intended rule is that phases only go up: 1, then 2, then 3. The shipped phase-2 test does not check for phase 3, though. A boss in phase 3 whose health is at least 0.37 and below 0.75 enters the phase 2 entry, and its phase becomes 2 again (phase-2 chances). It returns to phase 3 only when health falls below 0.37. This is a reproducible quirk (see Edge cases). Whether health can rise during play is not established.

Phase 2 entry: the boss sets its phase to 2, stores 1.2 in local coroutine frame data (no known reader; see Edge cases), and sends the phase-change event 4. It plays its idle animation and is idle by the end of the same tick, so the entry takes one tick and has no animation wait. Phase 3 entry: the boss sets its phase to 3, stores 1.4 in local coroutine frame data, sends the phase-change event 6, and is idle by the end of the same tick. A drop from above 75 percent straight below 37 percent enters phase 3 directly, so the phase 2 entry is skipped (High).

### States (Confidence: High for transitions and entry effects; Medium for the free-state test)

Each row is a state. The number is the state's index in the controller's dispatch, 0 to 20. "Free" means the attack gate is open (see Attack selection). Movement also requires that the boss is not flinching.

| State | Number | Animation (table index) | Behavior | Ends |
| --- | --- | --- | --- | --- |
| Idle | 0 | 0 | Turns with weight 0.05 per tick and plays animation 0. | A choice moves the boss out of idle. |
| Hit flinch | 1 | 0x0B, 0x0C or 0x0D (chosen on entry, 1/3 each) | Plays the flinch animation. | Animation finished: idle. |
| Catch | 2 | 0x14 | Catches the returning boomerang. | Animation finished: ready flag set to 1, boomerang counter reset to 0, then idle. |
| Big laugh | 3 | Its sub-animation | Turns 0.05 per tick and plays its sub-animation. | Animation finished: idle. Entered only when the big-laugh request word is 1, which is never set, so the state is unreachable (see Edge cases). |
| Triumph | 4 | 0x12 | Plays once while the hero is down. | Animation finished: idle. Re-entered on the next tick while the hero stays down. |
| Phase 2 entry | 5 | 0 | Sets phase 2 and stores 1.2 in local coroutine frame data (no known reader). | In the same tick: idle. |
| Phase 3 entry | 6 | 0 | Sets phase 3 and stores 1.4 in local coroutine frame data (no known reader). | In the same tick: idle. |
| Block | 7 | 0x19, then 0x1D | Turns 0.05 per tick. Each tick, in this order: (a) if the one-shot boss word is set (reading it clears it), play recovery 0x1D and then go idle; (b) else, while the hero's reaction trigger is true, hold 0x19; (c) else run the overhead decision (see Attack selection). If it does not start, go idle. | Recovery finished: idle. The overhead decision can start the overhead counter. |
| Jump | 8 | 6 (launch), 7 (air), 8 (landing) | Launch and air turn with weight 0.2 per tick. Moves by its velocity each air tick (see Jump flight). | Landing: idle. |
| Roll | 9 | 9 or 10 (chosen by a coin) | Plays the roll. | Roll finished: idle, or the roll counter-attack (state 10, animation 0x16) when the hero is within 5 units and the roll test passes. |
| Swing (and roll counter-attack) | 10 | 0x17 first, then 0x16 on the next swing (alternating through a toggle). A roll counter-attack always uses 0x16. | One hit window with the shared hit (damage 10). | Animation finished: idle. |
| Overhead counter | 11 | 0x18 | One hit window with the shared hit (damage 15). | Animation finished: idle. |
| Boomerang throw | 12 | 0x13 | On release event 0x20, if the ready flag is 1, launches the boomerang: ready flag 0, flight timer 600 ticks. | Animation finished: idle. The boomerang may still be flying. |
| Flame wave | 13 | 0x1A | Starts one flame ring at the boss's position per wave on fire event 0x400. Waves: 4 in phases 1 and 2, 5 in phase 3. Between waves: idle sub-animation, then a random wait of 0 to 15 frames (inclusive). Turns 0.05 per tick. | Last wave finished: idle. Flame-wave cooldown counter reset. |
| Lunge | 14 | 0x15 | One hit window with the shared hit (damage 10). | Animation finished: idle. |
| Combo | 15 | 0x16, then 0x17, then 0x18 | Up to three hits, one per animation, damage 10 each. Turns fully to face the hero between hits. | Third animation finished: animation 1, then idle. |
| Walk | 16 | 2 (locomotion mode 0) | Moves toward the hero's position at 6.37 units/s (root motion; see Travel). | Hero distance less than 25: idle. At exactly 25 it keeps walking. |
| Hop back | 17 | 5 | No movement target is set; the clip's root motion carries the boss 7.36 units backward over the 60 ticks of the animation. | Animation finished: idle. |
| Orbit right | 18 | 3 (locomotion mode 5) | Moves toward the point 5 units along the side vector (see Orbit side), sideways at 15.01 units/s. | Blocked: jump. Walk, hop back, a new attack or a flinch ends it. It does not end by itself (see Movement). |
| Orbit left | 19 | 4 (locomotion mode 6) | Moves toward the point 5 units against the side vector, sideways at 15.01 units/s. | As orbit right. |
| Death | 20 | 0 | Sets story flag 273 through event 0x111 and plays animation 0. When the flag is clear and a world is loaded, this first set requests progress-flag stinger 1 (VAG resource 257; SPEC-0009 and SPEC-0024). Its table kind is 0, so it does not raise the journal-notification icon. An already-set flag does not replay the stinger. | Terminal. |

Locomotion mode numbers and animation numbers are different things. The boss's movement picks a locomotion mode, and the mode plays a table animation. Modes 0, 1 and 2 play animation 2; mode 5 plays 3 and mode 6 plays 4. The six per-boss locomotion slots are set at the controller's start. Walking uses mode 0 and the orbit uses modes 5 and 6.

If an orbit is blocked by the level, the boss starts a jump instead (jump state with animation 6).

### Attack selection (Confidence: High for the order, gates, distances and chances)

Each attack choice has the same steps:

1. Cooldown. The attack has a counter. Each tick the counter adds one, whatever the state. While the counter is at or below the cooldown in ticks (cooldown seconds x 60, truncated), the choice does nothing else. Once the counter is above that value, the choice rolls each tick until it starts the attack. The counter resets when the attack starts (swing, lunge, combo), when the catch ends (boomerang) and when the flame volley ends (flame wave).
2. State gate. A new attack can start only while the state gate is open. The gate is open in idle, hit flinch, walk, hop back, orbit right and orbit left. It is closed in catch, big laugh, triumph, the two phase entries, block, jump, roll, every attack state (swing, overhead counter, boomerang throw, flame wave, lunge, combo), and death.
3. Ready flag. Boomerang, flame wave, swing, lunge and combo require the ready flag to be 1. Block and jump also require it. Roll does not.
4. Distance window. The hero's distance must satisfy the window for that attack.
5. Roll. Compare the roll with the chance for the current phase (see Roll and coin rules). The attack starts if the roll is less than the chance.

The rules for each attack:

- **Block.** Runs on the hero's reaction rising edge. Needs the ready flag and the state gate. Starts if the roll is less than the block chance for the phase (25 / 35 / 75).
- **Jump.** Runs on the same rising edge. Needs the ready flag, the state gate and a hero distance below 5. A second distance, measured from the boss's home point (the position it had at the controller's start) to the boss, must be above 15 units. The start chance then rises in a straight line: c = 5 + (d - 15) / 13 x 75 percent, where d is that home distance. There is no cap: from d = 31.47 units on, every roll succeeds. The jump starts if the roll is less than c.
- **Roll.** Runs on the same rising edge. Needs the state gate. Starts if the roll is less than 50. A coin then picks roll animation 9 (coin 0) or 10 (coin 1).
- **Boomerang.** Needs the ready flag, the state gate and a hero distance strictly between 15 and 40. Cooldown 8 s. Starts if the roll is less than 90 / 50 / 25.
- **Flame wave.** Needs the ready flag, the state gate and a hero distance strictly between 10 and 40. Cooldown 10 s. Starts if the roll is less than 0 / 50 / 75. In phase 1 the chance is 0, so flame waves never start there. The three chances are indexed by the phase field 0, 1 or 2 (phases 1, 2 and 3; High, see Phases); a value outside 0 to 2 never starts a wave. The trigger's counter adds one only while it is at most 600, so it stops at 601, and a failed check does not reset it; once the gates pass, a roll is made each tick. On a start the boss enters state 13 and plays animation 0x1A, the wave index is set to 0, the fired flag of the wave is cleared, and six words are cleared: the five per-ring hit flags (one per flame object) and one jump word that the jump sets again before it is used, so that clear has no effect (High). The main cooldown counter is not reset by the start. Both distance ends are strict: exactly 10 or exactly 40 units does not start a wave. The trigger's gate word is the ready flag (High), so no flame wave can start while a boomerang is out.
- **Swing.** Needs the ready flag, the state gate and a hero distance below 10. Cooldown 10 s. Starts if the roll is less than 60 / 46 / 20.
- **Lunge.** Needs the ready flag, the state gate and a hero distance below 10. Cooldown 8 s. Starts if the roll is less than 30 / 40 / 50.
- **Combo.** Needs the ready flag, the state gate and a hero distance below 10. Cooldown 10 s. Starts if the roll is less than 10 / 20 / 30.
- **Overhead counter.** Decided inside the block state (see the block row of the States table), not after the recovery animation. It runs on each block tick where the one-shot word is not set and the hero's reaction trigger is false. It starts only if the roll is less than 25 and the hero is closer than 5 units and in its attack action state (action state 13). The roll is tested first. A block is held while the reaction trigger stays true, so a counter cannot start during a held block.
- **Roll counter-attack.** Follows a roll. Starts if the roll has finished, the hero is closer than 5 units, and the roll is at most 100 (so it always starts with the shipped chance; see Edge cases). It enters the swing state with animation 0x16.

Reaction trigger: the block, jump and roll choices run only on the tick the hero's reaction trigger rises from false to true. The trigger is the hero being in an attack swing (the hero's attack predicate; Medium confidence on the name). It is cleared when it goes false again. The rising edge is recorded on the first tick the trigger is true, even if the boss is busy at that moment. A swing that starts while the boss is busy is therefore missed, and no block, jump or roll follows it.

Ready flag: it starts at 1. It is set to 0 only at the boomerang release. It is set back to 1 when the catch ends, or when the boomerang flight times out (600 ticks). No other path sets it.

### Movement (Confidence: High for thresholds, the orbit end rule and the speeds at R = 1)

Each tick, after the attacks, when the state is free and the boss is not flinching:

- If the hero is farther than 25 units: the boss walks toward the hero (locomotion mode 0, animation 2). The walk lasts until the hero is closer than 25 units.
- Otherwise, a hop timer adds one each tick while the hero is within 25 units. If the hero is closer than 5 units, the boss is not ready, and the timer has passed 60 ticks, the timer is cleared and a roll of less than 50 starts a hop back (animation 5). Hop-back quirks are listed under Edge cases.
- Otherwise, if the hero is closer than 10 units and the boss is not already orbiting, a fair coin picks the right or left orbit (locomotion modes 5 and 6). If the boss is already orbiting, the choice does nothing, so the orbit continues.

The speeds of these movements come from the root motion of their clips (see Travel).

Orbit end rule. The orbit has no timer and no exit of its own. While the hero is between 10 and 25 units away, an orbit that is running continues indefinitely. An orbit ends only when one of these happens: the orbit is blocked (jump), the hero moves beyond 25 units (walk starts), the hop-back test passes, an attack starts, the boss is hit (flinch), or a reaction edge starts block, jump or roll.

### Travel (Confidence: High for the values at R = 1)

The boss travels only by the root motion of the clip it plays, as every character does (SPEC-0040 Behavior 14C; SPEC-0056, Travel by root motion): each tick the root record of the current key times the clip speed c = R x 0.5 / S is turned by the facing basis (B1 side, with B1 = up x facing; B3 facing) and added to its position, one tick after the animation pass that produced it, with ground following as for the heroes. R = 1 in every phase (see Edge cases). Distances and speeds:

| Movement or action | Animation | Travel |
| --- | --- | --- |
| Walk (modes 0, 1, 2) | 2 | Forward root 435 per key, S = 2: 0.10625 per tick, 6.37 units/s. The forward part is the same on every key; the side and up parts vary. |
| Orbit right (mode 5) | 3 | Side root +1024 per key, S = 2: 0.25012 per tick along +B1, 15.01 units/s. |
| Orbit left (mode 6) | 4 | 0.25012 per tick along -B1, 15.01 units/s. |
| Hop back | 5 | One-shot, 60 ticks: 7.36 units backward in total, almost all of it in keys 2 to 7. |
| Roll | 9 / 10 | 10.72 units along +B1 over 48 ticks / 10.66 units along -B1 over 52 ticks. |
| Lunge | 0x15 | 14.26 units forward over 76 ticks. |
| Swing | 0x16 / 0x17 | 2.41 units forward / 4.05 units forward and 1.22 units along -B1. |
| Overhead counter | 0x18 | 5.86 units forward. |
| Boomerang throw | 0x13 | 2.67 units forward. |
| Flinch | 0x0D | 4.99 units backward. |
| Jump launch | 6 | No root motion; the flight is the jump rule (see Jump flight). |

Here "forward" is +B3 and "backward" -B3. One-shot distances are totals over the whole animation at R = 1.

### Orbit side (Confidence: High)

The side vector is the unit vector of the horizontal hero direction u crossed with the reference vector. The reference vector is the constant up vector (0, 1, 0, 0) at 0x3932E0, written at boot (SPEC-0007). So the side is s = normalize(u x up) = (-u.z, 0, u.x). The orbit target is the boss's position plus 5 units times the side vector (right orbit), or minus that (left orbit). The orbit is re-aimed every tick, and the boss turns toward its motion as it strafes.

### Boomerang flight (Confidence: High for the target order, steering and catch request; Medium for the frame details)

- Release: the boomerang starts at the hand and moves along the unit vector of (0.5 x forward - 0.5 x right) of the boss. The ready flag is set to 0. The flight timeout counter is set to 600 ticks.
- Let D be the horizontal hero distance and u the horizontal unit direction from the boss to the hero. Let s be the same side vector as the orbit, s = (-u.z, 0, u.x) (see Orbit side; High). Waypoint A = hand + 0.5 D u + 0.4 D s. Waypoint B = hand + 0.5 D u - 0.4 D s, computed when the hero point is reached.
- Targets in order: waypoint A; then the hero point (the hero's x and z, height = the ground under the hero + 2, re-aimed every tick); then waypoint B; then the hand (re-aimed every tick).
- Each tick: compute the three-dimensional offset to the current target, its length r and its unit direction t. If r < 0.5 the boomerang snaps to the target and that target counts as reached. Otherwise k = 1.0 if r < 5, 0.25 if r < 10, else 0.1 (the far steering weight). The direction becomes the normalized (1 - k) x direction + k x t. The position moves by 0.7 units times the direction, once per tick (boomerang speed).
- If the position is below ground + 1.0 it is raised to ground + 1.0 and the downward direction component is set to 0.
- Reaching the hand target sets the catch request. The boss enters the catch on the next tick, because the pre-check runs before dispatch (animation 0x14).
- **Throw end**: the boss's throw state ends when its own animation ends. It does not wait for the boomerang. So the boss can be idle, and even start other choices, while the boomerang is still in the air. The ready flag stays 0 until the catch or the timeout.
- **Hit (High for the gate, the shape and the first-hit return; Medium-High for the once-per-throw rule)**: each tick while it flies, if the boomerang's position is at most 65.0 units from the hero (the same gate as the melee reach test), the boomerang is tested against each record of the hero body list (see Hero body shape records) with length above 0. The boomerang's test shape is a finite cylinder of radius 2.5 units and length 0.25 units, centred on the boomerang's position, with the boomerang's stored direction as its axis. The test is static only; no sweep runs. On the first record it overlaps, the boomerang's hit flag is set to 1, the hero takes the boomerang's damage (5, raw: unblockable and not reduced by the hero's defences), and the test ends. The once-per-throw rule depends on the caller reading the hit flag, which is not described (Medium-High).
- While held, the boomerang sits 1.1 units along the hand axis.
- Its model spins by 0.6 radians per tick about an axis that eases toward a fixed axis with weights 0.95 / 0.05 per tick (cosmetic). Spin and speed are applied per tick, not per second (Medium-High).
- Timeout: the 600-tick flight limit stops waypoint updates. The boomerang is placed at its held-in-hand transform, without a catch animation. Its waypoint phase field is not changed to a timeout phase; it retains the phase active when the limit expires. The ready flag is restored, but the boomerang counter is not reset.

### Jump flight (Confidence: High)

- Launch (animation 6): it turns with weight 0.2 per tick. On the tick the launch clip ends, the velocity becomes v = (dx / 60, 60 x 0.05 x 0.5, dz / 60) = (dx / 60, 1.5, dz / 60), where (dx, dz) is the horizontal offset from the boss to home (60.0 is the tick-rate word 0x2DB1C0, 0.05 the jump step word 0x2DB1C4). Clip 7 starts.
- Air (animation 7): each later air tick, if the three-dimensional distance to home is at least 3.0: v.y = v.y - 0.05, then the position is moved by v directly (not through physics). It turns with weight 0.2 per tick. Otherwise the landing clip 8 starts.
- Landing (animation 8): the landing does not move the boss and it does not turn. The state ends when clip 8 ends (see the Jump row).
- Home is the boss's position when its controller started.
- Gravity during the jump (High for the flag writes, Medium-High for the effect): the jump state sets the body's no-gravity flag (the flag of SPEC-0040 Behavior 14A) at the start of each of its ticks. During launch and air it stays set, so the body's own gravity does not pull the boss down and only the jump's own v moves it. On every tick on which clip 8 is the current animation, the state clears the flag again, so for the whole landing clip the body's normal character gravity and ground contact act: a boss that ends its flight above the ground falls under normal gravity (vertical speed starting from 0) and is held on the ground once it reaches it. On the first landing tick (the tick that requests clip 8) the flag is still set. After the jump the flag stays clear.

### Combo (Confidence: High for sequence and hit count; Medium-High for flag persistence)

The combo has three attack animations, played in order: 0x16, then 0x17, then 0x18. Each of the three hits goes through the shared once-per-attack hit (see below), with its own damage value: the first hit uses the first combo damage (10), the second the second (10), and the third the third (10). Within each animation the hit runs only while that animation's progress is below 0.99 (see Hit window).

The once flag is cleared when the combo starts (as for every attack the selectors start), and at two transitions:

- When animation 0x16 reaches 0.99 progress, the flag is cleared and the combo moves on to animation 0x17.
- When animation 0x17 reaches 0.99 progress, the flag is cleared and the combo moves on to animation 0x18.

When animation 0x18 reaches 0.99 progress, the combo ends and the boss returns to idle without a flag clear. So the combo can hit up to three times, at most once in each animation. Between hits the boss turns fully to face the hero (not confirmed). After the third hit it plays animation 1, then returns to idle.

### Flame-wave volley (Confidence: High for counts, waits and the ring hit rule; Medium-High for the ring's growth)

The flame wave plays its animation. There are four waves in phases 1 and 2, and five waves in phase 3 (the boss holds five flame objects). The boss turns toward the hero with weight 0.05 per tick during the volley. The fire event is bit 0x400, on frame 12 of the flame-wave clip (0x1A). The volley uses the five flame objects held by the boss itself (not the shared enemy projectile pool).

Per-volley state: the wave index w (0 at the start), the wait counter, and a fired flag (cleared at the start of each wave).

Each tick of the flame-wave state:
1. If the current animation is not the requested sub-animation, request it and do nothing else.
2. If the current animation is 0x1A and its progress has reached 0.99: if w is below the last index (3 in phases 1 and 2, 4 in phase 3), add 1 to w, clear the fired flag, request the idle sub-animation (0) and set the wait counter to a uniform integer from the wait minimum to the wait maximum (0 to 15, both inclusive). Otherwise the volley ends: the state becomes idle, the sub-animation 0, and the flame-wave cooldown counter is reset to 0.
3. Otherwise, if the current animation is not 0x1A (the idle sub-animation between waves): if the wait counter is below 1, request animation 0x1A; else subtract 1 from it.
4. Otherwise (0x1A playing): on the fire event, if the fired flag is clear, start flame object w and set the fired flag.

So between waves the idle animation is current for wait + 2 ticks, and each wave restarts animation 0x1A from its start.

**Flame ring.** Starting flame object w places it at the boss's position at that moment and starts its ring effect. A wave is a ring of flame that spreads out from where the boss stood, not a shot aimed at the hero. The ring has a current radius r and a height h, both read from the running ring effect each tick. The ring stops when its effect ends (High for the start point).

**Ring growth (Medium-High).** Each flame object runs effect 415 of effect bank 3 (SPEC-0034). That effect has one emitter that makes one burst of four identical, motionless particles at the ring centre. The flame object reads its r and h from one of them: r is the particle's first size component and h is its second size component times the flame object's scale (1.0). The particle's size follows the effect system's piecewise size rule (SPEC-0034): it starts at (0.5, 6.2963, 0.5), and on every effect tick (dt = 1/60 s) each component adds rate x dt / L, where L is the particle's life and the rate is picked by the life fraction f = age / L reached on that tick:

- r (first component): rate 24.5 for every f up to 2.0, so r grows linearly and reaches 0.5 + 24.5 = 25.0 at the end of the life.
- h (second component): rate −64.0 while f ≤ 0.05208, then −2.52015 while f ≤ 2.0.

L is 0.8 s with the effect's life jitter of up to ±0.02 s (SPEC-0034), so the ring lives about 48 ticks. With L = 0.8 s, r grows by 0.5104 units per tick (about 30.6 units per second) from 0.5 to 25.0, and h falls from 6.30 to about 3.63 in the first two ticks and then by about 0.0525 per tick to about 1.21 at the end. The hit band (0.6 wide) is wider than the per-tick growth, so a ring always meets a still hero whose distance from the centre lies within the ring's range, on at least one tick.

**Ring hit test.** Every tick, in every state of the boss (it runs in the per-tick update after the state handler), each of the five flame objects that is running is tested against the hero. Let d be the horizontal distance from the ring's centre to the hero's position and dy the hero's height minus the centre's height. The ring touches the hero when r − 0.3 < d < r + 0.3 and dy < h (no lower bound on dy). A touch deals 15 (word 0x2DB198) once per flame object: each object has a hit flag, which is set by the touch and is cleared only when a new volley starts. That damage is raw: it goes straight to the hero's take-hit, is not reduced by the hero's defences, and cannot be blocked (High). So the hero avoids a ring by standing inside it once it has passed, by staying ahead of it, or by being more than h above its centre as it passes.

### Shared once-per-attack hit: swing, overhead counter, lunge and combo (Confidence: High for routing, gates, guard branch and progress test; Medium-High for flag persistence)

One shared routine delivers the damage of the swing, the overhead counter, the lunge and each combo hit. Each attack passes its own damage value (see the table in Layout): swing 10, overhead counter 15, lunge 10, and combo hits 10, 10 and 10. The routine does not take a common damage value.

Each tick the routine runs while its attack is active and its hit window is open. Its inputs are the boss, the hero, the attack's once flag, and the damage for this hit. The checks run in this order, and a check that fails ends this tick without a hit:

1. Once flag. If the once flag is set, nothing happens. A hit that has already landed in this attack therefore blocks all later hits of the same attack, even if the hero stays in reach.
2. Boss predicate. The boss's own action-frame query must be true. The King, like every non-hero character, is an AI-controlled character, so this is the AI-character attack-active query: true when any of animation event bits 0x1 to 0x10 (weapons 1 to 5) is set on the current frame of its playing animations (SPEC-0005; weapon volumes in SPEC-0029). The same query asked of the hero uses the hero's attack-active word instead (SPEC-0036). Confidence: High.
3. Reach test. The hero must pass the reach test (see Hit test and blocking).

When all three checks pass, the hit is resolved and the once flag is set. The flag is set on every hit that passes the reach test, whether or not the hero blocks it. A hit that fails any check does not set the flag, so a later tick in the window may still hit.

Where the flag lives: in the controller's shared state (a word at byte offset 0x60 of that state). The controller keeps that state from one tick to the next, so the flag survives between ticks (Medium-High; see Open questions).

Clearing: the flag is cleared at the start of each attack that the attack selectors start (swing, overhead counter, lunge, combo) and at the combo's two transitions. It is not cleared when a roll counter-attack starts (see Edge cases).

Hit resolution (guard branch): if the hero is guarding and the dot product of the hero's and the boss's facing directions is at most -0.5, the hit deals no damage: the hero's take-hit runs with 0 and the blocked flag, so the hero plays its block reaction (the block sound cue and hero animation 0x40, SPEC-0029 Behavior 10.5). A float of 10.0 is also passed to that reaction, which ignores it (Medium-High). Otherwise the hit deals the damage passed for this hit in full, straight to the hero's take-hit: it is not reduced by the hero's shield, armour, armour Mote or petpet factor (SPEC-0029 Behavior 10.5).

### Hit window

Each attack handler calls the shared routine only while the attack's animation progress is below 0.99. When progress reaches 0.99 the hit window closes. The progress test reads the animation's track data for that animation. If no track entry matches the animation, the animation counts as not finished, so the window stays open for the whole animation (High for the rule). The exact meaning of "progress" is Medium (see Open questions). The reach test itself has no phase check of its own.

Inside that window the boss predicate (event bits 0x1 to 0x10 on the current frame) decides when the shared hit can land. In the King's clips (fight region 1264, monster row 16, which has no weapon volumes) only bit 0x1 is used, on these frames (High):

| Animation | Used by | Frames with bit 0x1 on |
| --- | --- | --- |
| 0x15 | lunge | 5 to 10 of 20 |
| 0x16 | swing (alternating with 0x17), roll counter-attack, first combo hit | 5 to 8 of 18 |
| 0x17 | swing (alternating with 0x16; the first swing), second combo hit | 6 to 8 of 19 |
| 0x18 | overhead counter, third combo hit | 9 to 13 of 23 |

### Hero body shape records

The hero's body carries a list of shape records. The list is read through the hero's body block: the block's word at +0x04 points to a header, the header's word at +0x20 is the first record, and the 16-bit count at header +0x38 is the number of records. Each record is 0x50 bytes:

| Offset | Size | Meaning |
| --- | --- | --- |
| +0x40 | f32 | Length of the finite cylinder. A record with length 0 or less is skipped. |
| +0x44 | f32 | Radius of the finite cylinder. |
| +0x30 | 16 bytes | Centre vector in the record's own frame (Medium). |

Each record's cylinder is placed by the hero's current pose (Medium for the exact axis rule). The finite-cylinder overlap is SPEC-0018. Confidence: High for the offsets and the skip rule, Medium for the placement.

### Hit test and blocking (Confidence: High for the caller gates, dimensions, query order and first-record rule; Medium for the secondary sweep's exact geometry)

The reach test used by the shared hit runs in this order:

1. Distance gate. The full three-dimensional distance between the boss position and hero position must be at most 65.0 units. A greater distance fails before any body shape is tested.
2. Weapon points. Let A be the boss's weapon anchor and D1 and D2 the two pose-derived weapon directions. The start point is P0 = A − 1.5D1. The static weapon cylinder has radius 2.5, length 7.0, centre P0 and axis D1. Its second endpoint is E1 = P0 + 7.0D1; the alternative endpoint is E2 = P0 + 7.0D2.
3. Static test. For each positive-length hero body record in list order, first test the weapon cylinder at P0 against that record's finite cylinder (radius +0x44, length +0x40), using the static overlap of SPEC-0018.
4. Secondary sweep. Only after a static miss, compare E1 and E2. If their spatial components differ, call the shared secondary spatial helper using the hero record's radius and length and the prepared endpoint/frame inputs. If the endpoints are equal, skip this helper for that record. The helper has no time or frame-counter input, but its exact swept collision surface, mathematical branch result and endpoint-inclusion rule remain unknown; do not treat it as a fully specified swept cylinder.
5. Attack timing. The swing, overhead counter, lunge and combo handlers invoke this reach route on eligible attack-window ticks, before animation progress reaches 0.99. This is a spatial query within the existing attack window, not a separate timed sweep phase.

The test returns true on the first record that either test touches, and false otherwise.

The attack reach is therefore decided by the cylinder tests, not by the 65.0 gate alone: the gate only limits how far the hero can be at all.

### Test vectors for the reach gate

- Boss and hero 64.9 units apart: the distance gate passes; a hit depends on the shape tests.
- Boss and hero 65.0 units apart: the gate passes (the test is "at most"); a hit depends on the shape tests.
- Boss and hero 65.1 units apart: the gate fails; no hit, whatever the weapon and hero geometry, and the once flag is not set.
- The gate is three-dimensional: a hero 40 units to the side, 40 units high and 40 units ahead is about 69.3 units away and fails, though each axis alone is under 65.

### Sounds and effects (Confidence: High unless marked)

**Sounds come only from the animation table.** The King's controller starts no sound of its own. His main animation table (region 1264) names cues of creature block 367 (SPEC-0031). Every sound entry has start frame 0 and no keep-alive bit, so when the main animation changes to an entry with a sound, that cue plays once, at the boss, on the first tick of the animation. A request of the animation that is already playing starts nothing. No sound plays while the boss is in base state 42 (the animation-sound step is skipped there; SPEC-0031).

Animation to cue of block 367 (animation numbers in decimal; animations not listed have no sound):

- 1: cue 1 (growl).
- 7 (jump, air): cue 7 (jump whoosh).
- 9 and 10 (rolls): cue 0 (hit dirt).
- 11 and 12 (flinches): cues 5 and 6 (flinch hits). Flinch 13 is silent.
- 14: cue 2 (big laugh). 15: cue 10 (crowd please). 16: cue 8 (mouth wipe). 17: cue 9 (crazy rage).
- 18 (triumph): cue 11 (triumph).
- 19 (boomerang throw): cue 12 (throw). 20 (catch): cue 13 (catch).
- 21 (lunge) and 24 (overhead counter, third combo hit): cue 4 (lunge swipe).
- 22 and 23 (swings, first two combo hits): cue 3 (swipe).
- 25 (block): cue 15 (block).
- 26 (flame wave): cue 16 (flame wave), at the start of each wave: the idle animation is current between waves, so animation 26 restarts for every wave and the cue plays four times per volley in phases 1 and 2 and five times in phase 3 (High).
- 27: cue 17 (exhausted pant).
- 28: cue 18 (death).

Silent: idle 0, walk 2, orbit 3 and 4, hop back 5, jump launch 6, jump landing 8, flinch 13 and block recovery 29.

Never played: cues 2, 8, 9, 10, 17 and 18 of block 367, because animations 14 to 17, 27 and 28 are never requested (the boss dies silently). Also cues 14, 19 and 20 of block 367 (the boomerang whoosh and the two taunts) are named by no table and no code. The boomerang is silent in flight.

The Werelupe Basher, Elite and Bone Thrower use creature block 350, not 367 (SPEC-0029 Behavior 35).

**Effects.** The King's code starts no effect, and his clips carry no clip effect list entries.

## Roll and coin rules

- A roll is an integer from 0 to 100 inclusive, which is 101 values. "Roll less than c" succeeds with probability c / 101, not c / 100. A chance of 0 never succeeds. "Roll at most c" succeeds with probability (c + 1) / 101, so a chance of 100 always succeeds.
- A coin is an integer from 0 to 1 inclusive, so each value has probability 1/2.
- A uniform integer drawn between two bounds is inclusive on both ends. If the two bounds are equal, the result is that value.
- The random generator is the shared one, seeded once from the console clock (SPEC-0014).

## Edge cases and error handling

- **Phase regression (quirk).** The phase-2 test in the shipped game excludes only phase 2, not phase 3. A phase-3 boss whose health is at least 0.37 and below 0.75 re-enters phase 2 (phase-2 chances). Reproduce this. Whether health can rise during play is not established.
- **Flame waves are unreachable in phase 1.** The phase 1 flame-wave chance is 0, so no roll is less than it. The shipped values allow flame waves in phase 2 (50) and phase 3 (75) only.
- **Flame-wave counter stops at 601.** The counter adds one only while it is at most 600, so it stays at 601 once it has passed the limit. A failed check does not reset it, so while the gates pass a roll is made every tick.
- **Flame-wave tier outside 0 to 2.** The roll never passes, so no wave starts.
- **Flame-wave distance is strict.** A boss exactly 10 or exactly 40 units from the hero does not start a wave.
- **Roll counter-attack is certain.** The counter-attack test is "less than or equal" against a chance of 100, which every roll meets. Reproduce this: a hero who ends a roll within 5 units is always counter-attacked. The attack chances elsewhere use "less than".
- **Roll counter-attack leaves the once flag set.** The roll counter-attack does not clear the shared once flag. If an earlier attack's hit set the flag and no attack selector has cleared it since, the counter-attack animation plays and deals no damage. The next attack selector clears the flag. Reproduce this.
- **Hop back needs the boss to be not ready.** The ready flag is cleared only at the boomerang release, so in practice the hop back runs only while a boomerang is in flight. Reproduce this gate.
- **Hop timer quirk.** The hop timer adds one each tick the hero is within 25 units, whether or not the boss is close or ready. It is checked and cleared only when the hero is closer than 5 units and the boss is not ready. A boss that has spent a long time at mid-range therefore passes the 60-tick check at its first close, not-ready tick. Reproduce this.
- **Cooldowns run in any state.** The attack cooldown counters add one before the state gate is checked, so they run down while the boss is busy. The boomerang counter resets when the catch ends; the swing, lunge and combo counters reset when their attack starts; the flame counter resets when the volley ends. Block, overhead and roll have no counter.
- **Boomerang timeout keeps the counter.** A flight that times out does not reset the boomerang counter, so a new throw may follow at once when the counter is past its cooldown.
- **Boss idle while the boomerang flies.** Because the throw ends with its animation, the boss may start a new choice during the flight. The ready flag keeps the boomerang, flame wave, swing, lunge and combo choices closed until the catch or the timeout. Block and jump are closed the same way. Roll is not.
- **Big laugh is unreachable (High).** The big-laugh state runs only when its request word is 1. The word starts at 0 and is only ever cleared: no routine of the King (pre-check, choosers, the 21 state handlers, the reach-test body, boomerang flight and damage, flame waves, debug text) stores a non-zero value to it, and the controller record is passed to no other routine. The big-laugh chance is never loaded. Keep the state, but it never runs.
- **No flame wave while a boomerang is out.** The flame-wave gate is the ready flag, which is 0 from the boomerang release to the catch or timeout.
- **Unused tunables.** These values are stored and registered, but no code in the boot program reads them: the big laugh chance and the three exhausted-pant values. The block chance for phase 3 is registered twice; this is harmless.
- **Phase thresholds use whole percents.** The thresholds are stored as whole numbers and divided by 100 when compared with the health fraction.
- **Phase entries take one tick.** The phase entries set idle in the same tick, so there is no animation wait.
- **Flinch and phase checks.** A flinch can interrupt any state. The catch, triumph, phase and death checks of the same tick still run after the flinch.
- **Reaction trigger latch.** The trigger's rising edge is recorded even when the boss is busy, so a hero swing that starts while the boss is busy never triggers a block, jump or roll.
- **Shared hit happens once per attack.** After the first hit that passes the reach test, the same swing, overhead counter, lunge or combo animation deals nothing more, even if the hero stays in reach or the animation continues. A new attack starts with the flag clear (cleared at its start) and may hit again. A combo may hit once in each of its three animations.
- **Hits that miss do not use the flag.** A tick where the boss predicate or the reach test fails leaves the once flag clear, so a later tick in the same window can still hit.
- **Blocked hits use the flag.** A hero who is guarding and faces the boss takes no damage from the shared hit, for any attack. The hit still sets the once flag, so that attack cannot hit again. For example, a blocked overhead counter deals 0 (not 15) and sets the flag.
- **Feedback value has no effect.** On a blocked hit the hero's block reaction is given the float 10.0 along with the 0 damage, but the reaction does not read it (Medium-High). A clean build may drop it.
- **Phase speed values have no known reader (Medium-High).** The phase handlers store 1.2 and 1.4 in entries of the coroutine's local frame data; no reader of those entries is known. The per-tick class loop stores 1.0 in P each tick and has no phase-dependent P write, so clips and root travel remain at R = 1 in all three phases. Reproduce this; do not speed the boss up by phase.
- **Hop back moves without a target.** The hop-back state sets no movement, but its clip's root motion carries the boss 7.36 units backward.
- **Orbit never ends by itself.** An orbit at a hero distance of 10 to 25 units runs until one of the end conditions in Movement occurs.

## Test vectors

- Phase 1, health 70 of 100 (f = 0.70): the phase becomes 2 and 1.2 is stored in local coroutine frame data on this tick; the state is idle at the end of the tick.
- Phase 3 boss, health 60 of 100 (f = 0.60): the phase becomes 2 and 1.2 is stored in local coroutine frame data (the shipped phase-2 test). Health later below 37 of 100 returns it to phase 3.
- Health 80 of 100 (f = 0.80): phase 1, no entry. Health 74 of 100: phase 2 entry. Health 36 of 100: phase 3 entry, skipping phase 2 if the boss was in phase 1. Health 37 of 100 (f = 0.37): not below 0.37, so no phase 3 entry. Health 0: death.
- Boomerang, phase 1, hero 20 units away, counter above 480 ticks, ready: roll 89 starts a boomerang (89 < 90); roll 90 does not.
- Boomerang counter at exactly 480 ticks (8 x 60): not above, so no roll happens; on the next tick, with the counter above 480, rolls begin.
- Boomerang throw: the throw state ends when its animation ends, even if the boomerang has not been caught. A boss that finishes the throw animation at tick t is idle at t with the ready flag still 0.
- Boomerang hit: a sweep that first touches a hero shape deals 5 once; a second touch in the same throw deals nothing.
- Boomerang steering: a target 4 units away steers with weight 1.0; 7 units away with 0.25; 12 units away with 0.1.
- Flame wave, phase 2, hero 25 units away, counter above 600 ticks (10 x 60): roll 49 starts it (49 < 50); roll 50 does not. In phase 1 no roll value starts one.
- Flame wave, tier 1 (the chance of phase 2), counter 601, hero 20 units away, roll 49: state 13 and animation 26 start. Roll 50 does not. Hero exactly 10.0 units away does not start one. Tier 0 never starts one (synthetic).
- Flame volley wave count: phase 2 fires four waves, phase 3 fires five.
- Flame-wave wait between waves: an integer from 0 to 15 frames, inclusive, uniform. With a wait of 0 the idle animation is current for 2 ticks; with 15, for 17 ticks.
- Flame ring hit (synthetic): ring centre (0, 0, 0), current radius 10.0, height 2.0. Hero at (10.2, 0.5, 0): d = 10.2, inside 9.7 to 10.3, dy 0.5 < 2.0: 15 damage, and that ring's hit flag is set. Hero at (10.3, 0, 0): no hit (the band is strict). Hero at (0, 2.5, 9.8): no hit (dy 2.5 is not below 2.0). Hero at (0, -5.0, 10.0): hit (no lower bound).
- Flame ring growth, life 0.8 s: after 1 effect tick r = 1.0104 and h = 4.9630; after 2 ticks r = 1.5208 and h = 3.6296; after 3 ticks r = 2.0313 and h = 3.5771; after 48 ticks r = 25.000 and h = 1.2145. Life 0.78 s: 46 ticks, last r = 24.581. Life 0.82 s: 49 ticks, last r = 24.900.
- Flame ring reach: a still hero 12.0 units from the centre (life 0.8 s) is inside the band on effect ticks 22 and 23 (r = 11.729 and 12.240); the first of those ticks deals 15 if dy < h (h about 2.58), and the second deals nothing (hit flag). A hero 26.0 units away is never touched.
- Flame ring once per volley: ring 0 touches the hero at tick t (15 damage); at tick t + 1 it still touches: no damage. A ring 1 touch later in the same volley deals 15.
- Flame-wave cue: a phase 3 volley plays cue 16 five times.
- Jump landing (synthetic): the flight ends 1.45 above flat ground. On the tick that requests clip 8 the no-gravity flag is still set; from the next tick on it is clear while clip 8 plays, so the body falls under the normal character gravity of SPEC-0040 Behavior 14A from vertical speed 0 until it is grounded.
- Swing, phase 1, hero 9.9 units away, cooldown passed: roll 59 starts a swing; roll 60 does not.
- Shared hit, once per attack: swing attack, reach true at tick t and again at tick t + 1, boss predicate true, hero not guarding: damage 10 applied at t only; at t + 1 nothing.
- Shared hit, miss does not set the flag: swing attack, reach false at tick t (for example the hero 65.1 units away), reach true at tick t + 1: damage 10 at t + 1 only.
- Shared hit, damage routing: an overhead counter hit deals 15; a swing hit deals 10; a lunge hit deals 10; combo hits deal 10, 10 and 10 each from its own word. Each attack uses its own damage word, so the overhead counter's word is the one that tells the attacks apart from a shared value.
- Shared hit, guard threshold: hero guarding, facing dot -0.5 exactly, reach true: take-hit 0 with the blocked flag (the hero plays animation 0x40; the 10.0 has no effect), flag set. Hero guarding with dot -0.4: full damage 10 (a swing hit) and flag set.
- Shared hit, blocked overhead: hero guarding with dot -0.5, overhead counter with reach true: 0 damage (not 15), flag set; a second tick in reach deals nothing.
- Roll counter-attack flag: a swing lands a hit (flag 1). Later a roll ends with the hero 4 units away: the counter-attack animation 0x16 plays and deals 0 damage. A swing started after that clears the flag at its start and can hit.
- Reach gate vectors: see Test vectors for the reach gate (64.9 and 65.0 pass the gate; 65.1 fails).
- Combo, animation transitions: hit in animation 0x16 at progress 0.5 (10 damage, flag set). At progress 0.99 the flag clears and animation 0x17 begins; a hit lands in 0x17 (10 damage, flag set); at progress 0.99 the flag clears and 0x18 begins; a hit lands in 0x18 (10 damage). Total 30 damage over three hits.
- Combo, no transition clear: within animation 0x16, after its hit at progress 0.5, a second tick in reach deals nothing.
- Roll counter-attack: after a roll, hero 4.9 units away: the counter-attack starts for every roll value 0 to 100.
- Walk: hero 26 units away starts a walk. Hero 25 units away does not (the test is strictly greater than 25).
- Orbit: hero 15 units away and the boss orbiting; 600 ticks later the boss is still orbiting. If the hero moves to 26 units away, a walk starts.
- Hop back: hero 4 units away, boss not ready, hop timer past 60 ticks: roll 49 starts a hop back; roll 50 does not.
- Jump: home distance 21 units gives a start chance of 5 + (21 - 15) / 13 x 75, about 39.6. Rolls 0 to 39 start a jump (40 of 101 values). Home distance 32: chance above 100, every roll starts a jump.
- Jump flight (synthetic, home 30 units away along +x on level ground): v = (0.5, 1.5, 0). After 30 moves the height is +21.75. After 57 moves the distance to home is 3.22, so one more move follows. After 58 moves it is 1.76, so the landing starts on the next tick, 1.0 short of home and 1.45 above it.
- Orbit side: with u = (0, 0, 1) the right orbit target is pos + (-5, 0, 0); with u = (1, 0, 0) it is pos + (0, 0, 5).
- Flame wave with the boomerang in flight (ready flag 0), phase 3, all other gates passing: no wave starts for any roll.
- Hit flinch: a hit while the boss is casting a flame wave interrupts it and plays one of animations 0x0B to 0x0D.
- Lunge window (disc clip): with the hero in reach for the whole lunge, the hit can land only on frames 5 to 10 of animation 0x15; it lands once, for 10.
- Unreduced damage: a hero with 100 HP and armour, hit unblocked by the overhead counter: HP 85. Hit by one flame projectile: HP 85; a second touch by the same flame object deals nothing.
- Triumph: hero down, boss idle: triumph starts once; the next tick does not restart it while it runs. When it ends and the hero is still down, triumph starts again.
- Navigation lookup failure: boss not attached, no mesh contains its position: the failure line prints, no attach, the controller keeps running and never retries the lookup.
- Probability: roll less than 50 has probability 50 / 101, about 0.495.
- Walk travel: a walk on flat ground covers 0.10625 units per tick, 6.375 units in 60 ticks (first step one tick after the clip starts).
- Orbit travel: 60 ticks of right orbit move the boss 15.01 units along its side axis.
- Phase speed: in phase 3 (local value 1.4, with no known reader) the walk still covers 6.37 units/s and the hop back still takes 60 ticks.
- Sounds (disc table): the main animation changes to 19 (boomerang throw): block 367 cue 12 plays once at the boss on that tick; the boomerang's flight plays nothing. Changes to 25 (block): cue 15. Changes to 6 (jump launch): no sound; then 7 (air): cue 7.
- Death: on the death tick the boss sends event 0x111, setting flag 273. With a loaded world and flag 273 initially clear, stinger 1 (resource 257) is requested once and no journal icon is raised; a later set does not replay it. The boss plays animation 0. Animation 28 is never requested, so cue 18 of block 367 never plays.
- Flinch entry: reaction word 10 with a roll of 1 from {0, 1, 2}: the word becomes 0, the state becomes 1 and animation 0x0C is requested. Any other reaction value: no flinch.
- Distance unit: boss at (0, 0, 0), hero at (6, 30, 8): the distance used by the attack choice is 10.0 (the 30 of height is ignored), so a swing range of 10 does not allow the swing (it needs less than 10) and a boomerang minimum of 15 is not passed.

## Open questions

1. Travel speeds. Resolved: every action moves by clip root motion at R = 1 (walk 6.37 units/s, orbit 15.01 units/s sideways, hop back 7.36 units over 60 ticks; see Travel), and the jump's horizontal flight is a set per-tick velocity toward home (see Jump flight; High).
2. Resolved: there is no unit conversion. Every distance tunable of the attack choice (jump trigger, roll and overhead counter distances, boomerang, flame-wave, swing and lunge ranges) is compared directly with the horizontal distance between the boss's and the hero's positions (height ignored), in world position units (High).
3. The spin rate and the boomerang speed are applied per tick (Medium-High).
4. The 65-unit gate of the melee reach test is established for the shared hit (see Hit test and blocking). The boomerang hit uses the same gate, measured from the boomerang's position (High).
5. Whether the jump step constant (0.05) is also an animation blend time is not established. Answered: it is the jump's gravity term and launch factor, and the 0.05 turn weight in idle, flame and block is a separate fixed value (see Tuning values and Jump flight; High).
6. State 1 is the hit flinch, not an intro. Resolved: the reaction step is the only path into state 1, always with animations 0x0B, 0x0C or 0x0D (High).
7. A cap on the jump chance above 28 units. Resolved: there is no cap (see Attack selection).
8. The two unlabelled constants (1.1 and 0.1). Resolved: the held-boomerang offset and the far steering weight (see Tuning values).
9. The hero action that raises the reaction trigger. Resolved: the hero's attack swing (see Attack selection); the hero-side predicate name is Medium.
10. Whether any tunable above is read by code outside the boot program (for example an overlay). No use of the unused values is known. Whether the debug text routine that runs each tick is part of the shipped build is not established.
11. Boomerang model-frame details (cosmetic) and the hand bone used.
12. The boomerang hit: the cylinder radius 2.5 and length 0.25 are fixed values of the hit. Whether the boomerang's own shape data holds any other radius is not established (Medium-Low).
13. Reach test of the shared hit. Answered in part (High for the caller contract): the full-3D 65.0 gate, attack-window timing, point construction, per-record positive-length filter, static-first order, endpoint-difference gate and immediate first-hit return are specified in Hit test and blocking. Open (Medium): the shared secondary helper's exact collision surface, combined narrow-phase formula and endpoint-inclusion rule are not established; it is a shared spatial helper used by multiple callers.
14. Shared-hit damage routing: High. The shared routine serves the swing, overhead counter, lunge and combo, and each attack passes its own damage (swing 10, overhead counter 15, lunge 10, combo 10 / 10 / 10). The overhead counter does not deal 10 through a common value.
15. Once-per-attack flag and combo progress: the mechanism that keeps the controller's shared state across ticks is not described (Medium-High for persistence), and the exact meaning of "progress" as the animation's fraction played is Medium. Answered: the flag's location and the progress rule are in Shared once-per-attack hit and Hit window (High).
16. The boss action-frame query. Resolved: the King is an AI-controlled character, so the query is true when any of animation event bits 0x1 to 0x10 is set on the current frame; asked of the hero it uses the hero's attack-active word (see Shared once-per-attack hit; High).
17. The unused big-laugh and exhausted-pant values (see Edge cases) are established as unused only within the boot program. Whether an overlay or other module reads them is not established (see also 10).
18. High: the controller dispatches on the current state number (0 to 20), and each state in the States table has exactly one handler. The state number is set by the choices and by the handlers themselves. The 21 handlers are therefore the states listed above.
19. Order High, lookup parameter Medium-High: the first-run navigation lookup and its failure behavior are described in Controller start. After a failed lookup the controller keeps running without a mesh and never retries. The lookup parameter is the owning world (question 22).
20. Free-state test (Medium): the triumph and big-laugh choices, and the reaction edge, test "free" with the attack gate. This is assumed; the test they use is not described separately.
21. Whether the big-laugh request word is ever set. Resolved: it starts at 0 and is only cleared, so big laugh is unreachable (see Edge cases; High).
22. What the boss does while it has no walk mesh (movement calls on an unattached boss). Answered: the lookup parameter is the character's owning world, or the current world when none is set (see Controller start; Medium-High).
23. The boomerang's side vector. Resolved: the same as the orbit's, s = normalize(u x up) = (-u.z, 0, u.x) (see Boomerang flight; High).
24. Resolved in part: On timeout, the boomerang stops advancing its waypoint phase and is placed at the held-in-hand transform; the phase word is left at its current waypoint value. The hit routine is described in Boomerang flight: it checks eligible hero body records in list order and applies the damage on the first overlap. The shared overlap helper exact collision surface and endpoint rule remain open (see question 13). [High for timeout and call order; Medium-High for the helper boundary]
25. Flinch clear call. Resolved: it is the shared AI-mode setter with mode 0; for mode 0 it only stores 0 in the reaction word, with no animation, locomotion or timer change (High).
26. The orbit reference vector. Resolved: the constant up vector (0, 1, 0, 0) at 0x3932E0, written at boot (see Orbit side; High).
27. Roll counter-attack flag (Medium-High): confirm that no other path clears the once flag between the roll and the counter-attack. No such path is known.
28. The flame-wave gate word. Resolved: the ready flag, so a boomerang flight blocks flame waves (see Attack selection and Edge cases; High).
29. The tier value. Resolved: the phase field of the controller record, 0 at start (phase 1), 1 in phase 2 and 2 in phase 3 (see Phases and health thresholds; High).
30. Flame-wave second counter and six attack words. Resolved: the second counter is the wave index; the six words are the five per-ring hit flags and one jump word (see Attack selection and Flame-wave volley; High).
31. Resolved (Medium-High): the phase handlers store 1.2 and 1.4 in local coroutine frame entries, while the per-tick class loop stores 1.0 to P. The local entries have no known reader, so the phase values do not reach clip playback or root travel (Edge cases).
32. The float 10.0 passed with a blocked shared hit. Resolved: the hero's block reaction does not read it, so it has no effect (see Edge cases; Medium-High).
33. Flame projectile hit shape. Resolved: each wave is a ring centred at the boss's position, and the hero is touched within 0.3 of the ring's current radius (horizontal) and below the ring's height (see Flame-wave volley; High). Resolved: the radius and height follow the size curve of ring effect 415 (radius 0.5 to 25.0 linearly over a life of about 0.8 s; height 6.30 falling to about 1.21; see Flame-wave volley; Medium-High).
34. The shared-hit windows. Resolved: the frames with event bit 0x1 in the King's clips, and the flame wave fires on bit 0x400 at frame 12 (see Hit window; High).
35. Jump landing and ground following. Resolved: the flag is the body's no-gravity flag; it is clear on every tick of clip 8, so normal gravity brings the boss down onto the ground during the landing (see Jump flight; Medium-High for the effect).
36. Death cue. Resolved: no code of the boss requests animation 28, nor 14 to 17 or 27. The animation request word is only ever set to 0, 1, 5 to 13, 0x12 to 0x1A and 0x1D, the locomotion slots name 2, 3 and 4, and the jump plays 6 to 8. So the death cue (block 367 cue 18) and the cues of animations 14 to 17 and 27 are never heard; the boss dies silently (High for the boss's own code; Medium-High that no shared routine changes its animation set). The unreachable big laugh would replay whatever animation was last requested.
37. Flame-wave cue per wave. Resolved: the idle animation becomes current between waves, so animation 26 restarts and cue 16 plays at every wave (see Sounds; High).
38. Resolved: the death event sets flag 273. On its first clear-to-set transition with a world loaded, its table-2 cue requests stinger 1 (resource 257); kind 0 means no journal-notification icon. Repeated sets do not replay the cue. This is a one-shot stinger, not a music-track change (SPEC-0009, SPEC-0024, SPEC-0049; High for the static call and table chain).

