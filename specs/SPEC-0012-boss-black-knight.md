# SPEC-0012: Boss behavior: Black Knight

- Answers: proactive, none requested (boss AI); R-0067 (the locomotion slot values, clips and root travel of modes 0, 5 and 6)
- Status: Draft
- Disc build: SCUS-97367 v1.00 (same fingerprint as SPEC-0001)
- Confidence: per section. Tuning values, the state list and the animation numbers: High. Weak-point maxima (50, 50, 50, 50, 50, 100) and the boss total (350): High; per-fight handler record and its initial state, phase, speed multiplier, counter flag, weak-point values, charge references, accumulators and latches: High. The health display (maximum 350, current HP from the quirk sum, every tick): High. Tick order, with the defeat test before the attack selectors and the state handler: High. Stun windows of 600 ticks: High. Ticks per second (60): High. Unused tunables: High. Phase rules: High that the phase changes happen as written, Medium that the phase reset is intended. Enraged sub-step machines: High for transitions, Medium for the wording of their purpose. The charge run exit (edge-contact flag gives the knee stun, body contact alone gives the impact): High for the rule; Medium-High that the walk-mesh boundary is the arena wall; Medium for the edge-wall geometry. Charge direction test: High for the arithmetic and the x/z axis mapping. Flying sword: High for its gate, sub-state list, held sub-state 5 and reset at the next spin-up, and return destination record and step rule; Medium for the exact attachment represented by the return point, target-height point attribution and hit-latch reset; Medium-High for the shared values and orbit-to-homing rule; High for the target-height probe formula; Low for the resulting world-space path; High for the side scalar scaling a normalized pose-derived orbit offset. Parry and deflect: High for hero-variant dispatch, predicates, slot path and roll arithmetic; Medium for the stance/animation interpretation. Idle movement: Medium. Weak-point tests of the enraged attacks: High for the sizes, threshold, reaction, hit effect (effect 305), latch and health change; Medium for the part-to-frame pairing. Flying sword world-volume pass: High for the entry filter, the class dispatch and the breakable cues (143 to 146); Medium for the cylinder leaf's radius offset. Sounds (animation cues, the special-victim cue 34): High. Arena flow: High for the post-fight block and one-time result; Medium for earlier order and gating. Melee weapons, windows, damage (unreduced by hero defences), the swing block and the knockback: High; the flying sword's frame: Medium-Low. Several flags: Low or unknown (see Open questions).

## Summary

The Black Knight is a melee boss with six weak points (left knee, right knee, left under-arm, right under-arm, back, neck). It runs a per-fight controller with 20 states. Its controller record is created on the active stack frame and initialized before the first tick: state 0, phase 0, speed multiplier 1.0, counter flag 1, weak-point HP 50/50/50/50/50/100, and charge references +20/−20/+60/−60. Each tick it updates its weak-point blend, checks whether it should react (laugh, change phase) and whether it is defeated, checks parry and deflect, may start an attack, chooses an idle move when it is idle, and then runs the handler for its current state. It moves up through three phases as its swings miss, but a landed hit or the end of an enraged attack sets it back to phase 0, so phase 2 must be earned again by missing another angry swing. It becomes vulnerable at a weak point only during the stun window of the attack that matches that point. During its sword spin a flying sword leaves its hand, orbits, homes in on the hero and returns, and the spin goes on only after the sword is back. When all six reach zero or below, it enters the defeated state for good and sets a level state that lets the arena script play the post-fight sequence. Its voices and attack sounds come only from its animation table, and a weak-point hit shows a spark effect (see Sounds and Effects). The clean side can build the full controller from the rules below. Times given in seconds become ticks at 60 ticks per second (SPEC-0007), so a value of 10.0 s is 600 ticks.

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
| FlyingSwordLaunchThreshold | float | 0x2DA488 | 0.4 | A sword leaves its launch when a component of its direction falls below this |
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
- High-swing second measure limit: 10.0 units, applied to a second horizontal length computed from a different pair of points, not the hero direction (hard-coded, Medium).
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
- Flags: laugh, angry start, enraged start, loosen neck and en garde, the sword-returned flag for the sword spin, and the per-attack hit latches change under the rules below. Their initial values are not all established; the sword-returned flag is cleared at spin-up and the hit latches are cleared when an attack is selected.
- A stun timer, a deflect stance timer, an orbit side (0 or 1) and an orbit timer (ticks): not given a start value; each is set before it is used (Medium).
- Parry accumulator, deflect accumulator and one shared parry/deflect latch: they start at 0.
- The edge-contact flag with its hold counter, and the body contact flag, read by the charge run step. Both are generic AI-character and body words set by the body movement step, not part of the boss's own state (see Edge contact).
- A walk mesh. At controller start, if the knight has no walk mesh, it looks up the walk mesh that contains its position and assigns it; if none is found it fights without one (High).
- One flying-sword record per boss, with its sub-state (0 to 5), orbit angle, bounce value and a hit latch. The spin step, path rate, path angle, revolution count and side scalar are the shared values listed under Tuning values.

## Behavior

### Per-tick order

Each tick, in this order (order matters because later steps see earlier flags) (High):

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
| 7 | Deflect stance | 9 stance, then 10 or 11 strike | Holds a stance for a random time, then may strike. The strike deals no damage (High) | Stance timer ended and no strike playing |
| 8 | Kick | 17 | Short-range kick. Deals KickAttackDamage on a hit and knocks the hero back. | Animation finished |
| 9 | Slow swing (phase 0) | 15 swing, 16 recovery | Swing, turning toward the hero by 0.25 per tick. If the hero blocks it (see Hit windows and damage), the swing turns into recovery 16 with no damage. A landed hit sets phase to 0 (no change). | Animation finished. Sets laugh flag if it landed, otherwise the angry-start flag. |
| 10 | Angry swing (phase 1) | 18 swing, 19 recovery | Swing. Same block rule (recovery 19). A landed hit sets phase to 0. | Animation finished. Sets the enraged-start flag if it did not land. |
| 11 | Enraged charge (phase 2) | 26, 27, 28, 29, 30 (45 on a hit), 31 or 32 | Charges at the hero when the charge direction test passes (see Attack selection, item 5a). Knees are vulnerable during the stun window, which is reached when the run touches the edge of the walk mesh (see Enraged attacks). | Recovery animation finished |
| 12 | Enraged high swing (phase 2) | 33, 34, 35, 36 (46 on a hit), 37 or 38 | High swing. Under-arms are vulnerable during the stun window. | Recovery animation finished |
| 13 | Enraged floor strike (phase 2) | 20, 21, 22, 23 (44 on a hit), 24 or 38 | Floor strike. Back is vulnerable during the stun window. | Exit animation finished |
| 14 | Sword spin / flying sword (phase 2, or balcony check) | 39, 40 (wait), 41, 42 (47 on a hit), 43 | Spin with the flying sword. Neck is vulnerable during the stun window. | Recovery animation finished, or the spin ends early (see Enraged attacks) |
| 15 | Turn toward hero | 7 | Turns to face the hero | Forward within about 8 degrees of the hero |
| 16 | Turn away | 6 | Turns away from the hero (the hero is on the left, or dead ahead on the right-hand test) | Forward within about 8 degrees |
| 17 | Approach | locomotion mode 0 | Walks toward the hero | Horizontal distance to the hero is within CloseDistance |
| 18 | Orbit | locomotion modes 5 or 6 | Circles the hero at a side | Horizontal distance to the hero is farther than 18.0 units, or blocked |
| 19 | Defeated | 43 | Plays the defeat animation every tick. The defeat test sets level state 0 once (see Defeat). | Never (terminal) |

The defeat animation (43) is the same table entry as the sword-spin recovery (state 14), and the floor-strike and high-swing recovery animation 38 is shared between the two attacks as listed above.

Phases: phase 0 uses the slow swing (state 9). A missed slow swing (or a blocked one) moves to phase 1 through state 2, where the angry swing (state 10) is used. A missed angry swing (or a blocked one) moves to phase 2 through state 3, where the enraged attacks (states 11 to 14) are used. A landed hit or the end of an enraged attack returns the phase to 0 (see Phases below). No state value of 20 or more is ever set by this controller, so those values are not reached from it (High).

### Phases

- From phase 0 the boss moves up when a swing misses: a missed slow swing (or a blocked one) sets phase 1 through the angry-start state, and a missed angry swing (or a blocked one) sets phase 2 through the enraged-start state. The controller initializes phase to 0.
- **A landed hit resets the phase to 0.** A landed angry swing sets phase to 0 (and so does not set the enraged-start flag). A landed enraged charge impact sets phase to 0. A landed slow swing leaves phase 0 unchanged.
- **Every enraged attack ends with phase 0.** The charge, high swing, floor strike and sword spin each set phase to 0 when they finish. So after an enraged attack the boss is back in the slow-swing phase, and it reaches phase 2 again only after a missed angry swing. Confidence High that the writes exist, Medium that this is intended. Reproduce as stated (see Edge cases).

### Flags (every tick)

1. If the boss is ready and the laugh flag is set: clear it. With probability LaughChance / 101, go to state 1. (The roll is an integer 0 to 100 and succeeds when it is strictly less than the chance.) Whether or not the roll succeeds, the check for this tick ends, so the flags in steps 2 to 5 are not checked in that tick (High).
2. Else if the angry-start flag is set: clear it, phase = 1, go to state 2.
3. Else if the enraged-start flag is set: clear it, phase = 2, go to state 3.
4. Else if the loosen-neck flag is set: clear it. Roll against LoosenNeckChance, success goes to state 4.
5. Else if the en-garde flag is set: clear it, go to state 5, and multiply the speed multiplier by (EnGardeIncrease + 100) / 100.
The en-garde flag is set only at the end of three enraged attacks: the enraged charge (after both knees die), the enraged high swing (after the right under-arm dies) and the enraged floor strike (after the back dies). The sword spin does not set it. The loosen-neck flag is set only at the end of the parry state.

### Defeat (every tick)

The defeat test runs every tick right after the flag decision, before the parry and deflect checks, the attack selectors and the state handler, in every state, including the flying-sword and enraged states. It runs whether or not the boss is ready, and also on a tick where the laugh check ended the flag decision early. It does not depend on the state for its test (High).

- If all six weak-point HP values are at or below zero, the boss goes to state 19. This applies in any state. Because state 19 is busy, the attack selectors do not run in that tick, and the state handler plays the defeat animation in that same tick.
- Then, if level state 0 (bit 0 of the level's state bits) is not already set, the boss sets it. So it is set once, and the arena script reacts to it (see Arena flow) (High for the test and the set; Medium-High that this is the level state 0 that the arena script reads).
- Damage to parts is dealt in the state handler, after the defeat test. A part that reaches zero in a tick is therefore seen by the defeat test at the start of the next tick (see Edge cases).

### Parry

Checked when the boss is ready, by hero variant. The word used here is the hero index (0 = Tor, 1 = Roberta), not an attack-state value (High; SPEC-0040 Behavior 18).

- Tor (hero index 0): call the hero's attack-state predicate (SPEC-0040 Behavior 18). A false result clears the shared parry/deflect latch. A true result with the latch clear sets it and rolls against the parry accumulator. Success (roll strictly less than the accumulator): reset the accumulator to 0 and go to state 6. Failure: accumulator += ParryChanceIncrease. A true result with the latch already set does nothing.
- With the accumulator at 0, a roll can never succeed (no integer 0 to 100 is below 0). After one failure it is 50, so the chance is 50/101.
- The parry state, when its animation finishes, sets the loosen-neck flag (Medium).

### Deflect

Roberta only (hero index 1): scan the hero's six active shot slots (SPEC-0036); the Tor attack-state predicate is not called on this branch. The same shared latch used by the Tor parry path gates the overlap test:

- On the first active slot while the shared latch is clear: set the latch. If that slot overlaps the boss and a roll is strictly less than the deflect accumulator, reset the accumulator to 0 and go to state 7 with a stance timer of a random whole number of ticks in [DeflectStanceMin x 60, DeflectStanceMax x 60] (240 to 360 ticks). If it overlaps but the roll fails, accumulator += DeflectChanceIncrease. If there is no overlap, no roll occurs. The latch remains set while any shot slot is active, so later active slots and ticks in the same uninterrupted active period do not run another overlap test.
- When no slot is active, clear the shared latch. Other hero-index values take neither branch.
- The deflect stance strike (animation 10 or 11, a two-way random pick) deals no damage to the hero (High).

### Attack selection

Only in ready states. The first selector that matches wins, and the later ones do not run because the state has changed. Every selector that chooses an attack clears the per-attack hit-landed latch and the second latch (charge, high swing, floor strike, sword spin, slow swing, angry swing, kick, counter and balcony).

1. Counter (opening kick): if the counter flag is set, clear it. Roll an integer 0 to 100; if it is strictly less than CounterAttackChance (40), go to state 8 (kick). There is no range or hittable check, so a kick can start at any distance. If the flag is set on the first ready tick, this gives a 40% chance (40 of 101 roll values) of an opening kick; the flag starts set, giving a 40% chance (40 of 101 roll values) of an opening kick. If the flag is clear, the later selectors run in the same tick as usual. The controller itself does not set the flag to 1.
2. Kick: if the hero passes the guard predicate (SPEC-0036, Guard timer and per-attacker block), the horizontal distance from the boss to the hero is less than KickAttackRange, and a roll against KickAttackChance succeeds, go to state 8.
3. Slow swing: in phase 0, if the horizontal distance from the boss to the hero is less than SlowSwingRange, go to state 9. No roll.
4. Angry swing: in phase 1, if the horizontal distance from the boss to the hero is less than AngrySwingRange, go to state 10. No roll.
5. Phase 2 group, checked in this order:
   a. Charge: requires at least one knee above zero. Let A be the x component and B the z component of the horizontal unit direction from the boss to the hero. The shared boss-to-hero helper returns this direction in x/z, forces y to zero, and puts horizontal distance in w (High; see Open question 3). If |B| is less than |A|, select controller word 25 when A > 0 and word 26 otherwise. If |B| is at least |A|, select word 27 when B > 0 and word 28 otherwise. The charge starts (state 11) only if |selected reference - A| is less than EnragedChargeRange (30). No roll. The controller initializes these references to +20, −20, +60 and −60, respectively.
   b. High swing: if the horizontal distance from the boss to the hero is less than EnragedHighSwingRange, a second horizontal length, computed from a different pair of points and not from the hero direction, is below 10.0, and at least one under-arm is above zero, go to state 12. (Medium.)
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
| Stun | 30 (45 on a hit) | Knees exposed. Each tick: knee hit test, right knee first, then left (Weak points). The countdown falls by one each tick | Both knees at 0 or below: Knee-loss recovery. Countdown ends: Timer recovery |
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
| Arm-open | 35 | Plays once; starts the 600-tick countdown | Finished: Stun |
| Stun | 36 (46 as the hit reaction) | Under-arms exposed. Each tick: right-arm hit test, then left-arm hit test (an arm already at 0 or below is skipped). The left arm does not end the stun | Countdown ends: Timer recovery. Right arm at 0 or below: Right-arm recovery |
| Timer recovery | 37 | Plays once. Sets phase to 0. No en-garde flag | Finished: idle |
| Right-arm recovery | 38 | Plays once. Sets phase to 0 and sets the en-garde flag | Finished: idle |

**Enraged floor strike (state 13).**

| Step | Anim | What happens | Next |
| --- | --- | --- | --- |
| Windup | 20 | If weapon 1 touches the hero (event bit 0x1, frames 7 to 9 of 10) and the hero is not yet hit: deal EnragedFloorStrikeDamage, mark the attack hit | Hit: Follow-through. Finished without a hit: Stun start |
| Follow-through | 21 | Plays once | Finished: idle, phase 0 |
| Stun start | 22 | Plays once; starts the 600-tick countdown | Finished: Stun |
| Stun | 23 (44 as the hit reaction) | Back exposed. Each tick, if the hero passes the in-front test and the band test, the back loses the hero's damage value | Back at 0 or below: Back recovery. Countdown ends: Timer exit |
| Timer exit | 24 | Plays once. Sets phase to 0. No en-garde flag | Finished: idle |
| Back recovery | 38 | Plays once. Sets phase to 0 and sets the en-garde flag | Finished: idle |

**Enraged sword spin (state 14).**

| Step | Anim | What happens | Next |
| --- | --- | --- | --- |
| Spin-up | 39 | Plays once. When it ends, the sword's sub-state is set to 0 and the sword-returned flag is cleared, so each spin launches the sword once | Finished: Wait |
| Wait | 40 | The flying sword starts its cycle (attach, launch, orbit, homing, return, reset) and the spin waits until the sword-returned flag is set, which happens when the sword has come back to the hand | Flag set: Spin |
| Spin | 41 | Plays once. If any of the other five weak points (both knees, both under-arms, back) is above 0, the attack ends now: idle, phase 0. Otherwise start the neck stun | Finished: Neck stun, or the attack ends |
| Neck stun | 42 (47 as the hit reaction) | Neck exposed for the 600-tick countdown. Each tick: if the hero passes the neck test and the band test, the neck loses the hero's damage value | Neck at 0 or below: Neck recovery. Countdown ends: idle, phase 0 |
| Neck recovery | 43 | Plays once. Sets phase to 0. No en-garde flag | Finished: idle |

The flying sword (below) runs alongside the sword spin, in the same tick, after the sword-spin step, from the end of spin-up to the end of the spin. Its whole flight happens during the wait (animation 40); the spin (animation 41) starts only after the sword has returned (High for the gate; Medium for this reading of the wait's end).

### Idle movement

Runs only in states 0, 15, 16, 17 and 18. The rules apply in state 0; states 15 to 18 keep their own rules:

- If the angle between the boss's forward direction and the direction to the hero is greater than the turn threshold (90 degrees): if the hero is on the boss's right (dot with right axis > 0), go to state 15 (turn toward). Otherwise go to state 16 (turn away). Turning stops when the forward direction is within about 8 degrees of the hero.
- Otherwise, if the hero's position is not found on the knight's own ground mesh on this tick, do nothing. This test uses the batched form of the AI locate (SPEC-0018, Incremental locate: the cached triangle, then up to 50 triangles per call from the knight's own cursor), so a hero on the mesh but away from the cached triangle can read as not found for a few ticks while the scan advances (High).
- Otherwise, if the horizontal distance from the boss to the hero is more than CloseDistance, go to state 17 (approach). Else if it is at most OrbitDistance, go to state 18 (orbit), choosing a random side (0 or 1) and resetting the orbit timer to 30 ticks.
- Approach ends when the horizontal distance from the boss to the hero is less than CloseDistance.
- Orbit toggles its side each time the orbit timer reaches 0 and resets the timer to 30 ticks. It ends when the horizontal distance from the boss to the hero is more than 18.0 units or the hero is not found on the knight's ground mesh (the same batched test).
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

Confidence: High for the weapons, windows, damage and block rule; Medium-Low for the exact flying-sword frame (Open question 20).

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

- **Block (the two swings only).** A swing is blocked when the hero guards, weapon 1 touches the hero, and dot(hero facing, boss facing) < -0.5 (the two face each other). A blocked swing deals no damage (the take-hit is not called) and goes straight to its recovery animation (16 or 19). It counts as a missed swing for the phase rules. Every other attack of the knight is unblockable.
- **Knockback (kick, charge impact, flying-sword orbit hit).** The hero gets the velocity unit(9 x d + 12 x up) x 20.0, where d is the boss's facing (kick, flying sword) or the horizontal boss-to-hero direction (charge impact), and the 20.0 is the knock-back scale at 0x2DA484. The hero enters its knocked-back state (state 0x30, SPEC-0036).
- **Charge impact has no shape test.** It hits whenever its window is on, wherever the hero is.
- **Flying-sword cylinder.** The sword reuses weapon 1 of row 0 with the two words swapped: radius 0.75 (the row's length word) and length 8.0 (the row's radius word), placed on the sword's frame and tested against the hero bone capsules.
- The weak-point tests of the four enraged attacks, which use finite cylinders, are described under Weak-point test in the enraged attacks.

### Shape tests called by the controller

Each item names its owner.

- **Weak-point test (four enraged attacks).** Owned by the four enraged attack routines. See Weak-point test in the enraged attacks.
- **Flying sword volume pass.** Owned by the flying sword. See Flying sword, World-volume pass.
- **Monster weapon test (kick, swings, high swing, floor strike).** Owned by the shared enemy framework (SPEC-0029 Behavior 10.2). See Hit windows and damage.
- Damage values: kick 20, slow swing 20, angry swing 20, charge 20, high swing 20, floor strike 20 (all from the tuning table), flying sword 15 (FlyingSwordDamageA). All unreduced by the hero's defences.
- On a charge impact, on a kick hit and on an orbiting flying-sword hit, the hero is knocked back using the 20.0 knock-back scale.
- Swing block: if the hero guards while weapon 1 touches it and the two face each other (dot of the facings < -0.5) during a slow or angry swing, the swing is cancelled into its recovery animation with no damage.

### Weak-point test in the enraged attacks (Confidence: High for the sizes, the threshold, the reaction animation, the hit effect, the latch and the health change; Medium for the part-to-frame pairing; Medium-Low for the centre and axis of each part cylinder)

During the stun window of each enraged attack, the controller tests its exposed parts once per tick. The four attacks share one hit routine, and each attack passes its own cylinder size and parts:

| Attack (state) | Cylinder radius | Cylinder length | Threshold | Part (frame kind) | Reaction animation |
| --- | --- | --- | --- | --- | --- |
| Enraged charge (11) | 1.5 | 2.0 | 0.5 | Right knee (kind 0x25), then left knee (kind 0x22) | 45 |
| Enraged high swing (12) | 2.0 | 2.5 | 0.5 | Right under-arm (kind 0x18), then left under-arm (kind 0x09) | 46 |
| Enraged floor strike (13) | 1.5 | 2.0 | 0.5 | Back (kind 0x04) | 44 |
| Sword spin (14) | 2.5 | 2.0 | 0.5 | Neck (kind 0x07) | 47 |

Every hit starts effect 305 (not a sound; see Effects). The health values are the boss's own six part values (see Boss variables).

Each part's cylinder is built from a frame the controller makes from its own posed body, using the frame kind in the table. The part's cylinder takes the table's radius and length. The shared routine runs in this order, and the first success ends the test with a hit:

1. Hero body. The hero's body cylinder (radius 1.0, a fixed value; length from the hero's class query, whose value is not established) is tested against the part cylinder with the static finite-cylinder overlap of SPEC-0018.
2. Sweep. If step 1 misses and the part's two stored endpoints differ, a swept test of the part cylinder against the hero body centre runs (SPEC-0018 sweep).
3. Hero shot slots. Each of the hero's six shot slots that is armed (SPEC-0036) is tested. When the moved word is zero, the slot's point is tested directly. When it is not zero, a slot is tested only if the offset from the routine's start vector to the slot point, projected on the end vector, is at least the threshold (0.5). A slot tested this way is a point-against-sphere test with radius 5.0 plus the slot's radius. A slot that touches the part is disarmed, and the routine reports a hit.

The moved word is 1 for all four weak-point attacks: charge, high swing, floor strike and sword spin. Thus an armed hero shot slot must also pass the 0.5 projection gate in each window. The exact pose-to-mesh mapping and attack attachment identities remain open (Open question 20).

On a hit, in this order:

- The knight plays its reaction animation from the table (replacing its stun animation).
- The hero's damage query for the boss is called with argument 1.0. Its float result is rounded down to a whole number d.
- The part's health value becomes 0 if it is less than d, otherwise it becomes health minus d. A part therefore never goes below 0.
- The attack's hit-landed latch is set to 1. The latch is cleared when the attack is selected.
- Effect 305 starts once at the world position of the hit part's frame (frame kind 0x25 or 0x22 for the knees, 0x18 or 0x09 for the under-arms, 0x04 for the back, 0x07 for the neck). It is a one-shot spark burst (see Effects), restarted on each hit. No sound plays from this routine, and the reaction animations 44 to 47 carry no sound word, so the knight adds no sound of its own to a weak-point hit (see Sounds).

Confidence: High for the table, the order and the hit effects. Medium for the part-to-frame pairing (inferred from the order of the frame builds and sweeps; no label). Medium-Low for the frame's centre and axis vectors (Open question 21).

### Weak points

- A weak point loses HP only during the stun window of the matching attack: knees (charge), under-arms (high swing), back (floor strike), neck (sword spin). Each stun window lasts the matching StunTime: 10.0 x 60 = 600 ticks for all four (High). The window is 600 ticks of exposure; the 601st tick is the exit tick (Medium).
- Each hit subtracts an amount from the part's HP. The amount is the value returned by the hero's damage query for the boss (query argument 1.0), a float rounded down to a whole number. The amount is not a tunable. The result is floored at zero, so a part never goes below 0.
- A part at or below zero is skipped by its window test and no longer takes damage. It also no longer gates its attack in step 5 of attack selection.
- The six HP values are all at or below zero: the boss is defeated.

### Flying sword

The boss has one flying-sword record (Medium-High). The sword runs only in the sword-spin state (14), once per tick right after the spin step, and only while the spin's step is not the first one (spin-up), that is from the end of spin-up until the spin ends (High for this gate). It does nothing in any other state. The sword keeps a sub-state from 0 to 5 and runs one sub-state per tick. Its return (sub-state 4) sets the sword-returned flag that ends the spin's wait (High). There is no separate release flag: the flag that the spin-up clears and the wait reads only reports that the sword is back.

**Shared values.** The spin step, path rate, path angle, revolution count and side scalar are fixed shared words (see Tuning values), not per-sword values. Each launch tick resets them: revolution count 0, spin step 0.05, path rate 0, path angle 0, side scalar 3.0 (Medium-High). The per-sword values are the orbit angle, the bounce value, the sub-state and the hit latch.

Its sub-states are:

0. **Attach**: the sword copies the boss's hand pose into its record and moves to launch.
1. **Launch**: the sword starts at the hand. It turns upward until its vertical direction component falls below 0.4 (FlyingSwordLaunchThreshold). Each launch tick does the shared resets above. When the threshold is passed, it enters orbit with the orbit angle at 0.
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
   h. **Hit test**: if the hit latch is clear and the sword's cylinder (radius 0.75, length 8.0, on the sword's frame) touches a hero bone capsule (static overlap of SPEC-0018, no sweep), deal FlyingSwordDamageA (15), apply the kick knockback and set the latch. The latch clears on each full turn that increments the count, so the sword can hit once per revolution (Medium-High). High for the sizes; Medium-Low for the exact centre and axis of the sword's frame (Open question 20).
3. **Homing**: each tick the sword moves toward the hero at FlyingSwordSpeed / 60 (1.25 units per tick at the default 75), while the path angle keeps growing and its sine/cosine continue to weight a basis rebuilt from the current sword frame. The hero-directed motion and the evolving frame offset are both active; their exact combined world trajectory depends on the live pose and frame vectors. A hit test on the hero (the same cylinder) follows the latch rule above: a hit deals FlyingSwordDamageA and moves the sword to return. Contact with the scene also moves it to return. The scene query checks active collision records using a static overlap of two finite oriented cylinders; its spatial calculations use x, y and z, while the fourth lane is carried but not used for spatial length, dot or cross products. For the sword cylinder, radius is 0.75 and full axial length is 8.0 from the Black Knight's selected generic monster-table row; these are collision-shape parameters, not measured sword-mesh dimensions. The centre and axis come from the current sword pose record. The axis is the value passed as the cylinder axis; whether it follows the model’s authored long axis is unknown. Each scene record supplies its own radius and axial extent, and a record whose extent/activity value is not positive is skipped. The authoring meaning of eligible scene records remains open. In the homing state, a miss from this cylinder query is followed by a separate segment-versus-frame-rectangle query. The segment starts at the sword pose centre and ends at centre + axis × 6.0 (0.75 times the monster row's first collision-shape dimension). The query uses candidates from the current grid cell; it accepts a plane-crossing parameter from −0.02 through 1.02 and in-plane coordinates within the rectangle bounds widened by 0.25 on each side. A hit also sends the sword to return. This query receives no separate sphere radius and tests no rounded ends; the intent of the 0.25 margins is unknown.
4. **Return**: the sword moves toward a stored point on the boss's render object at 1.25 units per tick (FlyingSwordSpeed / 60). When the remaining distance is less than 1.25 units, it sets the sword-returned flag, which ends the spin's wait, and moves to reset. The return destination and step rule are High confidence; the exact attachment represented by the stored point is Medium.
5. **Held** (High): on every tick it runs, the sword's frame is copied from the knight's hand attachment (as in sub-state 0, but without moving on). It does not fly, test hits or touch the world volumes. It lasts for the rest of the spin, including the neck stun. When the next spin's spin-up clip (39) ends, the spin sets the sword's sub-state to 0 and clears the sword-returned flag.

Synthetic coefficient vectors (ideal trig values): at angle 0, (cos, sin) = (1, 0), so the first pose-derived basis vector supplies the offset; at π/2, (cos, sin) = (0, 1), so the second supplies it. These vectors specify the coefficient mapping only, not the basis orientation or world-space sword position.

**Homing-start timing** (single sword; orbit ticks counted from the first orbit tick, default constants): the 9th full turn completes on orbit tick 327 (5.45 s) with the count going to 9 and no homing start. The 10th completes on tick 348 (5.8 s) and starts homing if the sword is on its target height. Otherwise the count climbs by one on each later full turn, up to 14, and homing starts at the 15th full turn (tick 439, 7.3 s) at the latest. Typically the sword reaches its target height soon after the first turn, so homing starts at tick 348.

Confidence: High for the gate, the sub-state list, the return destination record and step rule, and the homing segment query inputs and bounds. Medium-High for the shared values, the homing-start rule and the sine/cosine coefficient mapping into pose-derived frames. Medium for the exact attachment represented by the return point, the target-height point attribution and the hit-latch reset. High for the side scalar's role in scaling the normalized orbit offset; Low for the resulting world-space path.

### World-volume pass (flying sword)

While the flying sword is in flight, its position is also tested against the level's world volumes. This check is run by the flying sword and is the only caller of its routine. Its timing within the sword's sub-states is not established here.

- The pass reads the sword's 64-byte query record (words 0 to 15) and finds the volumes in the cell of the level grid that the record is in.
- Each volume entry whose kind word (+0x40) is 1 or 3 is tested. The test depends on the volume's class byte (at +0x10 of the volume record):
  - Class 3 is a finite cylinder (SPEC-0018). Its radius is the float at +0x3C of the volume record and its length the float at +0x24 (Medium).
  - Class 1 uses a finite point-to-segment distance test against the sword query point and a segment built from the volume record (Medium; exact lane mapping and numeric threshold remain uncertain). Class 2 always reports no hit.
  - Classes 4 and 7 use one of two tests, chosen by whether the float at +0x2C is 0 or less. At 0 or below, a bounded segment is intersected with a rectangular plane: the segment parameter must be within [0, 1], and the intersection's projections must lie within two nonnegative extents. Above 0, five transformed probes are tested in sequence and stop at the first hit. The exact endpoint lane mapping and five-probe placement remain uncertain (High for branch structure; Medium for the plane-and-rectangle interpretation).
- A volume that hits fires once: a generic once-latch test on the entry's id (u16 at +0x58) must return 0. Then a sound cue of the main sound slot plays at the entry's position (+0x30). The volumes hit here are breakables, and the cue is chosen by the breakable's material, the entry's type word (short at +0x5C): 2 gives cue 143, 3 gives cue 145, 4 gives cue 146, and any other value gives cue 144. These are the breakable contact cues of SPEC-0031 (Breakable contact cues). This is the only cue the knight's own code plays; all its other sounds come from its animation table (see Sounds).
- After a volume hit, the break activation is called with strength value 0.25 and a normalized direction from the sword toward the volume, then the material-specific cue plays. The value is forwarded into the shared break-effect path; its downstream effect on spawned pieces is not established (Medium).

Confidence: High for the entry filter, the class dispatch, the class-4/7 branch structure and the cue choice. Medium for the cylinder radius offset, the class-1 lane mapping and the class-4/7 plane interpretation. Their numeric threshold, exact lane mapping and generated probe placements remain open (Open question 22).

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
- **Shared recovery animation**: animation 38 ends both the high swing (right-arm loss) and the floor strike (back loss), and animation 43 is both the sword-spin recovery and the defeat animation.
- **Revolution count and homing-start timing** (Medium-High): the count goes up on each full turn that does not start homing, so a sword whose target height is not reached keeps counting up to 14. The homing start is tested only on a full-turn tick, so a sword that is not on target at the ninth full turn does not start homing there. It starts homing at the first full turn with the count at 9 or more and the sword on target, or at the 15th full turn at the latest.
- **Sword hit once per revolution** (Medium-High): the sword's hit latch clears on each full turn that increments the count, so the sword can hit the hero once per revolution, not once per sword.
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
- Phase 2 order: knee above zero, |B| < |A|, and an arm above zero: charge wins (state 11). Knee above zero but |B| >= |A|, hero inside EnragedHighSwingRange, second measure below 10.0, an arm above zero: high swing (state 12).
- Sword spin: knees and under-arms and back all at 0, neck 50: state 14 from attack selection.
- Phase reset: phase 2, the boss finishes a floor strike. Phase is now 0. With the hero 5 units in front and the boss idle, the next attack is the slow swing (state 9), not an enraged state.
- Phase re-entry: phase 0, slow swing misses: phase 1. Angry swing misses: phase 2 through state 3. Then a charge impact lands: phase 0.
- Angry swing lands: phase 0 and no enraged-start flag. Next selection in range is the slow swing.
- Weak-point damage: back HP 50, hero damage query returns 12.7: back becomes 38. Back HP 5, damage 20: back becomes 0 (not -15).
- Turn: forward-to-hero angle 100 degrees, hero on the right: state 15. Angle 80 degrees, horizontal distance 12: state 17 (approach). Angle 80 degrees, horizontal distance 9: state 18 (orbit).
- Orbit side: timer starts at 30. After 30 ticks the side flips and the timer resets to 30.
- Orbit target locate: a call returning -1 triggers the boss's fallback target (its current position) and resets the movement mode. A -1 after a partial 50-triangle batch means no hit in that batch yet; it does not establish that the requested orbit point lies outside the mesh.
- Block: during a slow swing the hero guards, weapon 1 touches it and dot(hero facing, boss facing) = -0.6: the swing is cancelled into recovery 16 and the hero's HP is unchanged. The same with dot -0.4: the hero takes 20.
- Charge impact (synthetic): the hero is 30 units away while impact frames 3 to 5 play: the hero still takes 20 and the knockback.
- Flying sword (synthetic): a hero capsule whose axis is 0.7 from the blade axis and inside its 8.0 length: hit (0.7 is below 0.75 plus the capsule radius).
- Unreduced damage (synthetic): a hero with 100 HP and full armour is kicked: HP 80.
- Charge run: contact 1, edge 0: impact. Contact 1, edge 1: stun start, no impact. Contact 0, edge 1: stun start. Both 0: keep running.
- Edge hold: edge push at tick t: flag 1, hold 120. With no further contact the flag is still 1 at tick t + 119 (hold 1) and clear at tick t + 120 (hold 0).
- Balcony: ready state, roll 29 (below 30), and the hero on no triangle of the knight's ground mesh: the sword spin starts. Roll 30: no spin. Roll 0 with the hero standing on the arena mesh: no spin.
- Flying sword gate: in the sword-spin state on the spin-up step the sword does nothing. On the wait step it runs from attach. In any state other than 14 it does nothing. When the return finishes, the sword-returned flag is set and the next spin tick moves from the wait to the spin (animation 41).
- Navmesh edge wall: with the Knight’s radius 1.5, edge x = 100, inward normal +x, sufficient opposite-vertex depth and edge parameter in range, the boundary reach is 1.5; a centre at x = 101 is pushed to x = 101.5. The 20-unit wall height does not gate the test. If off-mesh with lock 0, restore the saved on-mesh position; with a nonzero one-tick lock, retain the current position.\\n- Tick rate: 600 ticks for a 10.0 s stun at 60.0 per second; homing 75 / 60 = 1.25 units per tick.
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
- Disc: entry 17 (kick) of the knight's main animation table in region 1260 has the sound word 0x0000490F: sound group 1, block 365, cue 9, start frame 0, no keep-alive. So cue 9 of block 365 plays once at the knight on the first tick of the kick. A weak-point hit looks up effect id 305 in effect group 3 (resource 9) and finds definition 444.

## Open questions

- Resolved (R-0067, side of root x): in modes 5 and 6 the shared steering turns the knight's move direction toward the hero (SPEC-0029, Behavior 2: with d the direction to the side target, mode 5 steers to up x d and mode 6 to d x up, which is the direction to the hero), and the root step of a clip is applied along the record's side axis, up x forward (SPEC-0040, Behavior 14C), a positive root x going along it. The table entries of these clips carry no mirror flag. With the knight facing +Z that axis is +X, which is the knight's left (SPEC-0004: the right side is -X). So the mode 5 clip, whose sideways root is positive, moves the knight to its left of the line to the hero, and the mode 6 clip, whose sideways root is negative, to its right. This is the opposite of the side point that the target names; that point only feeds the steering (High for the rule; Medium-High for what shows on screen). Test vector: boss at (0, 0, 0) and hero at (0, 0, 10): mode 5 moves toward +X and mode 6 toward -X, while the mode 5 target point is (-3, 0, 0) and the mode 6 target point is (3, 0, 0). Black Knight: orbit side 0 (mode 5, +717 per key) circles to its left, side 1 (mode 6, -675) to its right.

1. Resolved: the controller uses a per-fight stack record and initializes the six weak-point values to their maxima, the counter flag to 1, and the charge references to +20/−20/+60/−60. The character record independently selects shared monster-table row 0 for generic actor data.
2. Resolved for the flags: the parry state's completion sets the loosen-neck flag to 1; the charge, high-swing and floor-strike recovery branches set the en-garde flag to 1 when their matching weak point has been destroyed (see Flags; High). Still open: whether the walk-mesh boundary coincides with the arena wall (Medium-High). Answered: the charge run reads the walk-mesh edge-contact flag and the body contact flag (see Edge contact; High).
3. Resolved: the shared boss-to-hero helper returns horizontal unit direction in x/z, forces y to zero, and returns horizontal distance in w. The charge test calls that helper, uses x for A and z for B (Attack selection, item 5a; High). The four per-fight charge references are initialized to +20, −20, +60 and −60 (Open question 19). The second high-swing measure is Medium (its sign convention partly described).
4. Resolved: the sword targets 2.0 units above the ground height probed at the knight's current x/z, using a ceiling 8.0 units above its current y, with a minimum target height of 4.0. The formula is High confidence; attribution of the probe point to the knight's position is Medium-High. Return targets a stored point on the boss's render object and moves at 1.25 units per tick, resetting when the remaining distance is strictly less than one step (High); the exact attachment represented by that point is Medium. The exact world-space homing/orbit path remains Low. The side scalar's role is resolved: it scales the normalized pose-derived orbit offset added to the boss position (High); its disc value is 3.0. The run gate, held sub-state 5, homing start and hit reset are also answered in Flying sword.
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
15. Partly resolved: the balcony test and the idle-movement block test are both the hero's position located on the knight's own ground mesh (full-scan form for the balcony, batched form for idle movement and the orbit end; SPEC-0018, Incremental locate; High; see Attack selection item 6 and Idle movement). The shared boss-to-hero helper's direction and distance contract is resolved in Open question 3. Approach refreshes the hero-position target, orbit supplies its side-offset target, and both use shared route advancement (SPEC-0056; High; see Idle movement). Resolved: the orbit target-locate query's -1 means no containing triangle was returned by that call. A batched miss can be temporary; it does not establish that the target is outside the mesh. The boss responds by resetting its movement mode and targeting its current position before route advancement (High). Resolved: the scene query uses static finite oriented cylinder overlap and performs spatial calculations in x, y and z (High). The angle helpers return cosine and sine of radian angles and weight vectors in pose-derived orbit/path frames (High for helper identity and coefficient use). Resolved: in homing, a miss from the scene-cylinder test is followed by a segment-versus-frame-rectangle query using the sword centre to centre + axis × 6.0, inclusive parameter window −0.02 to 1.02 and 0.25 bounds margins (High for calls/limits; Medium-High for record interpretation). Resolved: the side scalar scales the normalized pose-derived orbit offset before it is added to the boss position (3.0 at launch; High; see Flying sword). Still open: exact world-space basis trajectory; axis-to-rendered-mesh-axis identity and scene-record authoring meaning; intent of the 0.25 margins; and separate sphere-helper contract. The return destination is a stored point on the boss render object; its named attachment or bone identity remains unverified (Medium; see Flying sword).
16. The per-second rate. Resolved: the float 60.0 at 0x30C598, so the boss runs at 60 ticks per second (see Tuning values; High; Medium that the table index chosen at boot is always the first one).
17. Whether the knee stun is reachable. Resolved: the knight runs the edge step every tick through its body movement, so the knee stun, the knee-loss recovery and its en-garde flag are reachable (see Edge contact).
18. Resolved: the per-fight counter flag starts at 1. The first ready check clears it and makes one opening-kick roll; this controller does not set it again.
19. Resolved: the per-fight charge-reference words 25 to 28 start at +20, −20, +60 and −60, respectively. They are initialized by the controller before its first tick.
20. Partly resolved: the flying-sword scene cylinder uses the current centre and axis vectors from its runtime record. Its radius 0.75 and full axial length 8.0 come from the Black Knight's selected generic monster-table row; these are collision-shape parameters and do not measure the rendered sword. The related homing segment extent is 6.0. The runtime axis is confirmed as the cylinder axis, but its correspondence to the rendered sword's authored long axis remains unknown. Resolved: the moved word is 1 for all four weak-point attacks, so the hero shot-slot projection gate is active for charge, high swing, floor strike and sword spin (High for the call-site value). The attachment bones used by the knight's weapons (pose entries 0 to 4) and the axis-to-rendered-sword mapping remain open. Answered: limb volumes and sword cylinder size are in Hit windows and damage.
21. Part frames: the pairing of the frame kinds (0x25, 0x22, 0x18, 0x09, 0x04, 0x07) with the parts follows the order of the builds and sweeps, not a label (Medium), and the meaning of the frame's centre and axis vectors is Medium-Low, because the vector component order is not settled. Answered: the frame kinds are the part ids passed to the frame builder.
22. Partly resolved: class 1 uses a finite point-to-segment distance test against the sword query point and a segment built from the volume record; exact lane mapping and numeric threshold remain uncertain (Medium). Class 2 always reports no hit. For classes 4 and 7, a nonpositive +0x2C value selects a bounded segment-to-plane test with two extent checks; a positive value selects five transformed probes tested in sequence, stopping at the first hit. The segment endpoint lanes, plane orientation and extent mapping, and generated probe placements remain uncertain (Medium). After a hit, break activation receives strength 0.25 and the normalized sword-to-volume direction; the downstream effect on spawned pieces remains unknown (Medium). Answered: the volume-pass codes are main-slot cues 143 to 146, chosen by the breakable's material (see World-volume pass; High).
23. Resolved: the post-fight block runs once. Although level state 7 is not set in the module, movie 47 returns to world 78 and reloads the area (SPEC-0041, module 989; High). The order and gating of earlier arena steps remain Medium.
24. Resolved for navmesh confinement: the knight uses the region-character boundary rule in SPEC-0056, with its motion-record radius (1.5 units). Boundary reach, the current and predicted position tests, and push/clip behavior are specified there. The wall’s 20-unit height is not tested, so it is not a vertical overlap. An off-mesh restore is skipped only while a generic one-tick motion lock is set; no Black Knight-specific exempt state or writer is evidenced. High for the shared rule and row radius; Medium for the absence of a Knight-specific writer. Also resolved for animation 27: the run moves the knight by root motion, 2.25 units forward per key, about 33.8 units per second (see Enraged attacks; High for the clip data).
25. Sound step gates: the knight is taken to be a world character like every character spawned from a level's character records, so the world-character sound rule applies (Medium-High). Two further gates of the animation-sound step, a motion-block gate and a per-character mute word, are not named.

