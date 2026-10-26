# SPEC-0020: Boss behavior: Gnarfas

- Answers: proactive, none requested (boss AI); R-0064 (in part: how Gnarfas becomes active in pack 985; the full answer is in SPEC-0019)
- Status: Draft
- Disc build: SCUS-97367 v1.00 (same fingerprint as SPEC-0001)
- Confidence: per section. Tuning values, the phase hub, the attack order and the cooldown rules: High. Damage taking (own hit check, flinch and exhaustion): High for the control flow, Medium-High for the hit test. Missile reaction: High for the rolls and timer; Medium-High for the missile contact shapes (bone cylinders for the reaction, body cylinder for the hit). Wait state never entered: Medium-High. Melee reaction: High. Orbit, approach and jump-to-tree sequence: High for states and exits; High for the landing height rule; Medium for the flight meanings. Body slam and head butt: High (damage and reach). Knockback: High. Chain whip and rapid strike: High for timing, the windows, the bone-chain test and its hero cylinder, the once-only rapid strike and the full-damage block. Raw boss damage (no hero defences): High. Block-counter branch: unreachable (High). Facing by locomotion mode: High for the shared steering rule (the 0.1 turn blend, the 0.9 switch, the snap weight 1.0, the caller blends and the mode lists, see Facing); Medium for which modes Gnarfas uses beyond orbit (Open question 14). Run or walk at 5.0: High. Travel speeds and distances at R = 1 (clip root motion): High. Per-tick animation-speed tunable and same-tick clip-rate effect: High; shipped default 1.0. Animation sounds (block 393) and the chain sparks (effect 710): High; clip effect lists: Medium-High.

## Summary

Gnarfas is a melee boss that closes on the hero, orbits when the hero is too close, and attacks with a chain whip, a rapid chain strike and a head butt. A chain whip and a rapid strike run on a cooldown that is drawn at random each time it is armed. A rapid strike can leave the boss exhausted for a while, and a jump sequence follows: the boss jumps between four tree points and sometimes jumps at the hero, landing next to the hero for a body slam. Phase 2 is switched on by a world flag, not by health. A one-shot rage flag arms when health falls below 25 percent. The boss reacts to the hero's missiles (block, dodge or jump) and melee attacks (dodge), and it takes damage only from hits it does not react to. All timings run on the 60 Hz game tick (SPEC-0007): a time in seconds becomes ticks = trunc(seconds x 60), and a random cooldown is drawn as a whole number of ticks between the two truncated bounds.

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
2. Count down the timers: the cooldowns, the jump timer, and the exhaustion timer, each by one tick, not going below zero.
3. Run the phase hub (below).
4. Run the movement and attack choices (approach, orbit, and the attack order).
5. Run the current state's behavior.
6. Run the post updates (effects, and the health-driven checks described in the phase hub).
7. Wait one tick.

### Phase hub (Confidence: High for the order and the world queries; Medium for the phase 2 effect)

Run each tick, in this order:

1. **Wait flag.** If the wait flag is set, the boss goes to the wait state (see States) and the rest of the hub is skipped. The wait flag is a word of the game's control state, not a level-script value. The game stores 0 to it at start-up and on each world load, and nothing ever sets it. So the wait state is never entered: implement the test and leave the flag always clear (Medium-High).
2. **World queries.** The world queries are bits 0 and 1 of the level's flag array. While bit 0 is clear: if the hero's health is below half its maximum, or Gnarfas has struck the hero more than twice (3 or more strikes), Gnarfas sends event 0 (sets level bit 0). The send sets the bit, so this happens once.
3. **Phase 2 check.** If bit 0 and bit 1 are both set and Gnarfas is not yet in phase 2, Gnarfas enters phase 2 once: the one-time phase-2 effect runs and the hero's health is restored to its maximum. Phase 2 is set by the world flags, not by health. Bit 1 is set by the arena script in answer to bit 0, after the mid-fight movie (see Arena flow).
4. **Free-state gate.** The rest of the hub runs only if the boss is in a free state: idle, approach or orbit.
5. **Hold.** If the hero's busy condition is true, the boss enters the triumph (hold) state and nothing else in the hub applies.
6. **Rage.** Otherwise, let p = trunc(100 x health / maximum health). If p < 100 - 75 (75 is the rage trigger word 0x2DADA4), so p < 25, and the one-shot rage flag is 0, the boss enters the rage state.
7. **Death.** If p <= 0 (health below 1 percent, since p is truncated), the boss enters the death state. This overrides the rage state in the same tick. If p is above 0, the boss keeps its state.

Details of the hub:

- Death is checked only in free states and only when the hero's busy condition is false. A boss at 0 health that is holding the hero (triumph state) does not die until the hold ends.
- The rage flag is set in the rage state and never cleared, so rage arms at most once per boss.

### Taking damage (Confidence: Medium-High)

Gnarfas checks hits against itself each tick, before the hub. The check runs in every state except flinch, dead, chain whip, rapid strike, chain power, body slam and head butt (in those states hits do nothing).

- **Melee hit**: the hero is attacking, the hero's attack is active and the hero's weapon touches Gnarfas (the hero's single-weapon reach test, SPEC-0029 Behavior 10.2). This starts the flinch.
- **Missile hit**: each of the hero's six missile slots is tested as a sphere (the slot's point and radius) against Gnarfas's vertical body cylinder (the body record's radius and height, standing on its position, axis up). The first slot that overlaps is removed. If Gnarfas is not in the block-missile state, the flinch value comes from the hero damage query for that same missile-slot index against Gnarfas (SPEC-0036); the query result is truncated toward zero when the flinch starts, and the tick's test ends. In the block-missile state the slot is only removed and the test goes on to the next slot without querying damage.
- The flinch plays animation 9. The flinch records "was exhausted" when it starts in the exhausted state.
- The damage (the hero's damage for that hit) is applied once, when the flinch animation ends. Then: if trunc(100 x health / maximum) < 1, death; else, if the hit came while exhausted, back to exhaustion (its timer keeps running); else idle.
- So Gnarfas takes no damage during its own attacks, a running flinch, or in the block-missile state.
- These two are the only hit kinds (High). Nothing else starts the flinch: no hazard, no other character and no script hit is tested. The stored damage is a whole number: the hero's melee damage (asked at scale 1.0) or the missile-slot-indexed hero damage query, each truncated toward zero when the flinch starts (High for the query routing and truncation; see SPEC-0036 for the query's damage inputs and target adjustment).
- **Shapes of the two contact tests.** The melee contact is the hero's single-weapon reach test against Gnarfas (Open question 17, resolved). The missile hit is the sphere against body cylinder test above; the missile reaction uses a different shape (bone cylinders, see Missile reaction). The hero-side shape records that some of the controller's tests read belong to the Werelupe King and are described in SPEC-0019, not here.

### States (Confidence: High for transitions; Medium for entry effects)

| State | Animation (table index) | Behavior | Ends |
| --- | --- | --- | --- |
| Idle | 3 | Waits. A choice moves the boss out of idle. | A choice starts. |
| Flinch | 9 | Plays the flinch animation; damage is applied at its end (see Taking damage). | Death, exhaustion or idle, as stated. |
| Triumph (hold) | 1 | Faces the hero while the hero is down. | When the hero's busy condition is false: idle. |
| Rage arm | none | Sets the one-shot rage flag. | Immediately: idle. |
| Death | 2 | Plays the death animation. On finishing, sends event 2 (sets level bit 2), which starts the arena's defeat sequence (see Arena flow). | Terminal. |
| Maniacal laugh | 0x12 | Laughs. | Animation finished: idle. |
| Chain whip | 0x13 | Hits once with the chain whip damage. The facing test picks the outcome (see Chain whip and rapid strike). | Animation finished: chain-whip cooldown drawn; idle. |
| Rapid strike | 0x14 | Hits once (no re-arm frame) with the rapid-strike damage (see Chain whip and rapid strike). | Animation finished: rapid-strike cooldown drawn. Then a roll less than 75 starts exhaustion; otherwise the jump sequence starts. |
| Exhaustion | 4 | Counts the exhaustion timer down. | Timer below 1 tick: the jump sequence starts, with its first step. |
| Chain power | 0x15 | Hits once with the chain-power damage (unreachable with shipped chances). | Animation finished: exhaustion with a drawn timer. If it hit and a roll is less than 30, maniacal laugh instead. |
| Body slam | 0x16 | Strikes once if the hero is closer than 5 units (see Body slam). | Animation finished: idle. If it hit and a roll is less than 30, maniacal laugh instead. |
| Head butt | 0x18 | Strikes when a hit test passes, once per attack. Adds one to the strike count and sends the hero a knockback with strength 2.0. No damage. | Animation finished: head-butt cooldown drawn; idle. |
| Approach | 6 when the hero is farther than 5 units (locomotion mode 1), otherwise 5 (locomotion mode 0) | Moves toward the hero: run 15.36 units/s, walk 6.34 units/s on average (root motion; see Travel). | Hero closer than 5 units: idle. |
| Orbit | 0x0D or 0x0E (locomotion mode 5 or 6; a coin picks the side) | Moves sideways around the hero at 4.63 units/s on average (see Orbit and Travel). | Approach or attack ends it. |
| Dodge missile | 0x10 | Dodges a missile (the reaction chose it). | Animation finished: idle. |
| Block missile | block pose (event 0x14) | Holds the block pose while a hero missile would reach Gnarfas. | No missile due: idle. |
| Dodge melee | variant 0: 0x10, variant 1: 0x11, variant 2: 0x0F | Dodges a melee attack (see Melee reaction). | Animation finished: idle. |
| Jump to tree | sub-steps below (7, 0x0B, 0x0C, 0x0A) | The jump sequence. | Sub-steps finished: idle. |
| Wait (live pause) | 3 | Waits while the wait flag is set. Never entered, since the flag is never set (see Phase hub). | Flag clear: idle. |

The jump-to-centre state is never entered.

### Missile reaction (Confidence: High for the rolls and timer; Medium-High for the contact shape)

In a free state, when the reaction timer is 0: for each active hero missile, its path is the segment from its point to point + 10 x velocity. The path is tested against each bone of Gnarfas's skeleton whose bone-record length is above 0. Each such bone is a finite cylinder: bone-record +0x40 is its full length and +0x44 its radius. The posed base point is offset by half the length along the posed bone's local Y axis, centering the cylinder on the bone (SPEC-0029 Behavior 26; High). If any such path touches a bone:

1. Roll less than the missile block chance (100 in phase 1, 25 in phase 2): enter the block-missile state.
2. Otherwise roll less than the missile dodge chance (25): enter the dodge-missile state (animation 0x10, then idle).
3. Otherwise roll less than the missile jump chance (25): start the jump sequence from its first step.

In all three cases the reaction timer is then set to a uniform whole number of ticks in [60, 180] (the dodge timer values x 60), whatever the outcome. If no missile path touches Gnarfas, nothing happens.

Block missile: Gnarfas holds its block pose while any hero missile, extended 120 ticks ahead, would touch it; otherwise it returns to idle. Missiles that touch Gnarfas in this state are removed with no effect.

### Melee reaction (Confidence: High)

On the tick the hero starts attacking (rising edge), roll less than 100 (melee dodge chance) -> dodge melee. The variant is 0, 1 or 2 (each 1/3), playing animation 0x10, 0x11 or 0x0F and turning toward the hero with weight 0.1. The state ends when the animation ends, then idle.

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

Approach (run or walk, High): the boss uses locomotion mode 1 (run, animation 6) when the horizontal distance to the hero is greater than 5.0, the run-or-walk tunable, and mode 0 (walk, animation 5) otherwise. It exits to idle once the hero is closer than 5 units. No speed tunable exists for approach or orbit; the boss travels by the root motion of the locomotion animation (see Travel).

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

### Jump to tree (Confidence: High for steps and exits; Medium for the flight meaning)

The jump sequence starts from the rapid strike (when the exhaust roll fails), from the end of exhaustion, or from a missile reaction. Its steps are numbered 0, 1, 2, 3 and 5 (step 4 is unused). Four tree points are used.

- **Step 0**: animation 7. Then step 1.
- **Step 1**: pick the tree nearest to Gnarfas in the horizontal plane and jump to it. Velocity per tick: v = (dx / 60, dy / 60 + 1.5, dz / 60), where (dx, dy, dz) is the offset to the tree point. Each tick v.y decreases by 0.05 and Gnarfas moves by v, playing animation 0x0B and turning toward its motion (weight 0.2). When the horizontal distance to the tree is below 3, it perches: step 3 with wait 0.
- **Step 3**: animation 10, no motion; count the wait down. When it is 0 or less: roll less than the jump-to-hero chance (50) -> step 5, aimed at the hero's position at that moment. Otherwise step 2, aimed at one of the other three trees, chosen uniformly.
- **Step 2**: the same flight (animation 0x0C, then 0x0B). When the three-dimensional distance to the tree is below 3: step 3 with a wait drawn uniformly in [0, 180] ticks.
- **Step 5**: the same flight toward the stored hero point. It lands when its height above the ground is 1.0 or less (High; this is the landing rule for every jump in the sequence). Then, if the hero is closer than 5 units, body slam; otherwise idle.

### Body slam (Confidence: High)

Started only by the landing of step 5: no other state write in Gnarfas's code sets the body-slam state (High). It has no shape test and reads no event bits: on any tick of its clip (0x16) it strikes once when the hero's horizontal distance is below 5.0. The strike deals 30 damage (the body-slam damage, word 0x2DAE2C) with a knockback of strength 1.0, and adds one to the strike count. A laugh (maniacal laugh) follows after a hit with a roll below 30.

Hero-down skip (High): when the strike would land while the hero is in one of the states 7 to 10, 0x0B or 0x2B to 0x2E, it deals no damage and no knockback, but the strike latch is still set, so the slam cannot land later in the same clip. The body slam is unblockable.

### Head butt (Confidence: High)

No damage. When its hit test passes (event bit 0x1, which is on in frames 6 to 13 of the 20-frame clip; the hero in front of Gnarfas, dot(boss facing, boss-to-hero direction) above 0.5; and the hero's horizontal distance below 7.0), the hero is knocked back with strength 2.0. There is no shape test beyond the facing and the distance (High).

### Knockback (Confidence: High)

A knockback of strength s:

- It runs only when the hero's busy condition is false (the same condition as the hub's hold test). Otherwise nothing changes.
- The hero's velocity is cleared, then 12.0 is added to its vertical part and 9 d to its horizontal part, where d is the horizontal unit direction from Gnarfas to the hero.
- The result is normalized and scaled by 8.0 x s (8.0 is the push speed word 0x2DAE78): velocity = normalize(9 d + 12 up) x 8 s. The hero then enters reaction state 0x30.
- So the push is 4.8 s horizontally and 6.4 s vertically, in the hero body's velocity units (SPEC-0040).

### Chain whip and rapid strike (Confidence: High)

- Both turn toward the hero with weight (1 - p) x 0.5, where p is the animation progress from 0 to 1.
- **Damage is raw.** Each hit sends its damage (15, words 0x2DADE4 and 0x2DADFC) straight to the hero's take-hit; the hero's shield, armour, armour Mote and petpet factor do not reduce it (SPEC-0029 Behavior 10.5).
- **Block still deals full damage.** If the hero is guarding and faces Gnarfas (dot of the hero's forward direction with the direction from the hero to Gnarfas above 0.5), the take-hit runs with the full 15 and the blocked flag set: the hero loses 15 HP and plays its block reaction instead of its hit reaction, and there is no knockback. Otherwise the hit is unblocked: 15 with the hit reaction and a knockback of strength 1.0. The "hero not down" check before the knockback is the hero's busy condition, the same as the hub's hold test (High).
- **Once per attack.** The chain whip hits once per attack. The rapid strike's hit latch would be re-armed only on a frame carrying event bit 0x20, and no frame of its clip carries that bit, so the rapid strike also hits at most once per attack, although its clip has four windows.
- **Windows** (frames with the event bit on, from the fight region 1264 clips; monster row 6, whose five weapon volumes (radius 3.0, length 0.5 each) are never tested by his attacks):

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

**Sounds come only from the animation table.** The controller starts no sound of its own. Gnarfas's main animation table (region 1264) names cues of creature block 393 (SPEC-0031). Every sound entry has start frame 0 and no keep-alive bit, so when the main animation changes to an entry with a sound, that cue plays once, at the boss, on the first tick of the animation. A request of the animation that is already playing starts nothing. No sound plays while the boss is in base state 42 (the animation-sound step is skipped there; SPEC-0031).

- Animations with a sound: 0 to 11, 16 to 18, 20 to 22, 24 and 26 (decimal). Every one of them names cue 0 of block 393 (the whip).
- Silent: 12 to 15, 19, 23 and 25. So the orbit (13, 14), dodge-melee variant 2 (15) and the chain whip's own animation (19) play no sound.
- **Quirk (reproduce).** The whip sound plays at the start of every animation that has a sound, including idle (3), triumph (1), death (2), flinch (9), exhaustion (4), approach (5, 6), the maniacal laugh (18) and the attacks other than the chain whip. Cues 1 to 6 of block 393 are named by no table and no code, so they never play.

**Effects started by the code.**

- **Chain sparks.** At spawn Gnarfas creates four effect objects of effect 710, one per chain tip bone: 19, 33, 41 and 49. Effect 710 is 35 white sparks, once, 1 s.
- A chain hit on the hero (chain whip, rapid strike or chain power) plays the object of the hitting run's tip bone: bit 0x1 bone 33, 0x2 bone 19, 0x4 bone 49, 0x8 bone 41.
- Each tick, a tip bone whose height is below the ground under it (the ground query at the tip's horizontal position with ceiling = tip height + 10.0, SPEC-0018; the tip is below ground when its height minus the found ground height is negative) plays its object once. A latch per bone holds until that bone is above the ground again, so a tip that stays in the ground sparks only once.

**Clip effect lists** (SPEC-0034; Medium-High for which list belongs to which clip): animations 2, 7, 8 and 22 show effect 1609 over the whole clip on attach slot 6 (13 brown sprites, once); 10 and 12 show 1609 with positional mode 0; 24 shows 1609 from 0.1 of the clip; 21 shows effect 936 at 0.85 (blue and grey models).

## Edge cases and error handling

- **Mid-fight movie at once.** If the hero is already below half health when the fight starts, event 0 fires on the first tick, so the phase-2 movie plays at once.
- **Chain power is unreachable with shipped data.** Both phase chances are 0, so the roll can never be less than them. Its exhaustion and maniacal-laugh branches therefore never run from it. Reproduce this.
- **Chain whip and rapid strike never roll.** Their chance tunables (40 and 10) are stored but unused. Each fires whenever its cooldown is zero and the hero is in range.
- **Head butt deals no damage.** Its only effect is the knockback of strength 2.0. There is no head-butt damage tunable.
- **Rapid strike hits once** (High; reproduce): its clip has four hit windows, but the re-arm needs event bit 0x20, which no frame of the clip carries, so only the first hit of the attack lands.
- **A block does not save HP against the chains** (High; reproduce): a guarding hero facing Gnarfas still loses the full 15 to a chain whip or rapid strike; the block only swaps the hit reaction for the block reaction and skips the knockback.
- **Boss hits ignore defences**: none of Gnarfas's hits is reduced by the hero's armour or shield.
- **Block counter is unreachable (High).** Its flag is in the boss's own controller record, starts at 0 and is only ever written with 0, so the block-counter step never starts an attack. Its tunables (range 15, chance 35) are stored only.
- **Wait state is unreachable (Medium-High).** The wait flag is cleared at start-up and on every world load and never set, so the wait state never runs; the other tests on the same flag (for example the hero's take-hit lock test) always see it clear.
- **Jump timer.** It is armed and counted but never used to start anything.
- **Rage is one shot.** It is armed once, and a later drop below 25 percent does not arm it again. If health is also at or below 1 percent in the same tick, death replaces rage.
- **Rage only sets the flag.** The rage state does nothing else.
- **Death at below 1 percent.** Death uses the truncated percent, so a boss at 0.5 percent of its maximum dies, and a boss at 1 percent does not. Death is only checked in free states, and not while the hero's busy condition holds.
- **Hold can outlast zero health.** A boss at 0 percent whose hero's busy condition is true holds instead of dying, until the condition clears.
- **Event 0 is sent once.** The test runs only while level bit 0 is clear, and the send sets it, so event 0 is sent on the first tick its condition holds and not again (unless the script clears level state 0, which happens only on world activation).
- **Phase 1 exhaustion is zero ticks.** Both phase-1 exhaustion values are 0, so the exhaustion state ends on its first check and the boss goes straight to the jump sequence. Phase 2 uses 2 to 3 seconds (120 to 180 ticks, inclusive, drawn uniformly).
- **Mode 5 or 6 facing with a blocked move.** The perpendicular target flips with the blocked side flip (Orbit), and the facing turns at 0.1 per tick again from the current heading, so the boss turns for several ticks after each flip before it snaps (Medium).
- **The whip sound on every animation (quirk, reproduce).** Every sound entry of the table names cue 0, so death, flinch and idle all start with the whip sound, and the chain whip itself (animation 19) is silent. Cues 1 to 6 never play.
- **Animation-speed tunable (High).** Each class-coroutine tick directly writes the tunable to P after the NPC update's reset/status processing and before clip re-time. A non-1.0 value changes that tick's clip rate and replaces any status multiplier already accumulated in P. The shipped value is 1.0, so the shipped build plays at the normal rate and this parameter has no visible effect with its current data.
- **Unused tunables.** These are stored and registered, but no code in the boot program reads them: exhausted chance 1 and 2, near-boss distance, melee jump chance, chain whip chance, chain whip head-butt chance, rapid-strike chance, jump land distance, jump attack trigger range, body slam cooldown, head-butt prepare chance, switch-to-walk percent, orbit range, chance min and max, and injured stance percent (16 values). The block counter's range and chance are also not used. No load or store to any of the 16 listed words exists in the boot program (High).

## Test vectors

- With the boss in idle and the hero's busy condition false: health 25 of 100 (p = 25) gives no rage. Health 24 of 100 (p = 24) with the rage flag clear arms rage once.
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
- Approach exit: hero 4.9 units away returns the boss to idle.
- Approach travel: 60 ticks of run (animation 6) on flat ground cover 15.36 units; 60 ticks of walk cover about 6.34 units.
- Orbit side: mode 5 moves the boss along +B1 (with facing (0, 0, 1), B1 = (1, 0, 0), so toward +x); mode 6 along -B1.
- Approach animation: hero 5.5 units away plays animation 6; hero 4.9 units away plays animation 5.
- Facing, mode 5 (synthetic): the move direction is along +z and the target perpendicular is along +x. The boss faces -z (dot with +x is 0). One tick at weight 0.1 gives heading normalize(0.1 × (1,0,0) + 0.9 × (0,0,-1)). Once the forward direction is within dot 0.9 of +x, the weight is 1.0 and the heading snaps to +x.
- Facing, mode 3 (synthetic): move direction +z, caller weight 0.5. The target is -z; the new heading is normalize(0.5 × (0,0,-1) + 0.5 × current).
- Probability: roll less than 75 has probability 75 / 101, about 0.74.
- Knockback strength 1.0, d = (1, 0, 0): the hero's velocity becomes (4.8, 6.4, 0) (magnitude 8), then hero state 0x30.
- Head butt knockback, strength 2.0, d = (1, 0, 0): (9.6, 12.8, 0).
- Knockback while the hero's busy condition is true: no change.
- Facing, mode 6 with m = (0, 0, 1): target m x up = (-1, 0, 0).
- Missile hit: a slot sphere of radius 0.5 whose centre is 0.4 outside the body cylinder's side wall overlaps; the slot is removed and Gnarfas flinches. The same in the block-missile state: the slot is removed and nothing else happens.
- Chain whip, hero guarding with dot(hero facing, hero-to-boss) 0.6: take-hit 15 with the blocked flag; HP 100 becomes 85, block animation 0x40, no knockback. Dot 0.4: HP 85 with the hit reaction and a knockback of 1.0.
- Rapid strike (disc clip): a hero standing inside the chains through all four windows takes one hit of 15 per attack.
- Body slam with the hero in state 8 and 4.0 units away on the landing: no damage, no knockback, and no later strike in that clip.
- Event 0: hero 49 of 100, strikes 0, level bit 0 clear: bit 0 set this tick. Hero 50 of 100, strikes 3: set. Hero 50 of 100, strikes 2: not set.
- Sounds (disc table): entry 0 is block 393 cue 0. The main animation changes to 9 (flinch): cue 0 (the whip) plays once at the boss. Changes to 19 (chain whip): no sound. Changes to 13 (orbit): no sound.
- Chain-spark ceiling: a tip at height 2.0 over a floor at 0.0 with a ledge at 11.5 above the same spot: the ceiling is 12.0, the ground found is the ledge (11.5), so the tip counts as below ground and sparks. With the ledge at 12.5 the found ground is the floor and no spark plays.
- Flinch damage: a hero melee hit whose damage query returns 7.9 stores 7; the flinch ends and health drops by 7.
- Chain spark: a chain whip hit on the hero in a bit 0x1 frame plays the effect-710 object of bone 33. Bone 41 dips below the ground on tick t and stays below until tick t + 30: its object plays once, on tick t. It rises and dips again: it plays again.
- Phase 2 hand-over: level states 0 set and 1 clear: movie 12, hero to locator 10, game state 213, level state 1 set, 30-tick sleep, dialogue line 14; the next Gnarfas tick enters phase 2 and sets the hero's health to its maximum.

## Open questions

1. Resolved: level bit 0 is Gnarfas's event 0 (hero below half health, or 3 or more strikes); bit 1 is set by the arena script of module 985 after the phase-2 movie when bit 0 is set and bit 1 clear; bit 2 is Gnarfas's death event, which runs the defeat sequence (see Arena flow). The level script's level-state calls and the boss events share one level bitset (High). Still open: entity 8 in that world pack is not identified, and entity 7 being Gnarfas is Medium-High.
2. Resolved: the knockback velocity is normalize(9 d + 12 up) x 8 s, then hero reaction state 0x30, and it is skipped while the hero's busy condition is true (see Knockback; High).
3. The stagger's entry condition. Resolved: only a hero melee hit (attacking, attack active, weapon touch) and a hero missile slot overlap start the flinch; no other hit kind is tested (see Taking damage; High).
4. Resolved: approach and orbit travel by clip root motion. Walk 6.34 units/s on average, run 15.36 units/s, orbit 4.63 units/s sideways (mode 5 along +B1 with the mirrored clip, mode 6 along -B1); see Travel. The jump flight speeds are as given in the Jump section.
5. Resolved: the jump-sequence landing rule is a height test. Landing happens when the height above the ground is 1.0 or less (High). The jump-land tunable has no reader (see item 9), so its unit does not matter.
6. Resolved: only the landing of jump step 5 writes the body-slam state; every state write in Gnarfas's code is accounted for (High).
7. Resolved: the distance tunables that code reads (for example the run-or-walk distance 5.0) are world units, compared directly with the horizontal distance between the boss's and the hero's positions, with no scale factor. The other distance tunables have no reader (see item 9) (High for the read ones).
8. Resolved: the chain-whip, rapid-strike and chain-power volumes are bone chains selected by event bit (bit 0x1 bones 29 to 33, 0x2 bones 15 to 19, 0x4 bones 46 to 49, 0x8 bones 38 to 41), each bone a cylinder of radius 0.5 and length 1.25 along its Y axis, against a vertical hero cylinder of radius 1.0 and length 3.0 (see Chain whip and rapid strike; High).
9. Resolved: no code outside the boot program reads the tunables. The only other program code is the 173 level modules; none of them contains an absolute data reference into the boot program's data area (no pair of high-half and low-half immediates forms an address in the data area, which starts right after the code and ends at 0x5EFFFF, so none touches the tunable words at 0x2DAD98 to 0x2DAE8C), and they reach the game only through host calls, none of which returns a boss tunable. So the 16 unused tunables and the block counter's range and chance are stored and never read (High for the modules; the boot program itself has no read, as stated under Unused tunables).
10. Resolved: the wait flag is a word of the game's control state, not a level-script value. It is stored as 0 at start-up and on each world load and never set, so the wait state is never entered (Medium-High). Still open: a whole-block copy over the control state is not ruled out as a setter.
11. Resolved: the block-counter flag lives in the boss's controller record, starts at 0 and is written only with 0 by the chooser, so the block-counter step is unreachable (High).
12. Resolved: the head-butt hit test is the event bit 0x1 window plus the facing (dot above 0.5) and distance (below 7.0) tests, and the body slam's hero-down skip covers the hero states 7 to 10, 0x0B and 0x2B to 0x2E (High). The "hero not down" check of the chain whip and the rapid strike, before their knockback, is the hero's busy condition, the same as the hub's hold test (High).
13. Resolved: the missile reaction tests each hero missile's 10-tick path against Gnarfas's bone cylinders for records whose full length is positive. The bone record's +0x40 is full cylinder length and +0x44 is radius; the cylinder is centered halfway along the posed bone's local Y axis from its posed base point (see Missile reaction; SPEC-0029 Behavior 26). The separate missile-hit test uses each slot sphere against the vertical body cylinder (Taking damage; High).
14. **Shared facing helper.** The facing rule above is a shared character-runtime rule. Two shared movement routines call the helper, so it is not specific to Gnarfas; the clean side may place it in the shared character runtime instead (SPEC-0040). Resolved: Gnarfas asks the locomotion mode setter for only four modes: 0 (walk and the orbit fallback toward the hero), 1 (run), 5 (orbit one side) and 6 (orbit the other side). All five call sites are in the approach and orbit routines; no Gnarfas routine requests 2, 3, 4 or 7 to 10 (High for the call sites). The turning weights are the caller's, as the helper rule states: modes 0 and 1 use the weight the movement step passes, and 5 and 6 use the perpendicular rule above. Test vector: the approach with the hero 6 units away requests mode 1; with the hero 4 units away it requests mode 0.
15. **Mode setter and handlers.** The routine that stores the locomotion mode sometimes calls a per-mode handler instead of storing the mode (when the character's class and busy flag allow it and the mode is below 7). The handlers are not described, so a mode that is dispatched rather than stored is not covered by the facing rule. Resolved: the reference direction for the perpendicular modes is the constant up vector (0, 1, 0, 0) at 0x3932E0; modes 5 and 9 face up x m, modes 6 and 10 face m x up (High).
16. The 0.1 turn blend, the 0.9 alignment test and the snap weight 1.0 are the constants of the one shared steering routine used by every actor that moves by locomotion mode (High). The chain whip's own weight, (1 − p) × 0.5, is a separate rule.
17. Resolved: Gnarfas's melee hits use the bone-chain test (whip, rapid strike, chain power) or plain distance tests (body slam below 5.0, head butt below 7.0 with the facing test); it uses no monster weapon volumes. The single-weapon reach test found in its code is the hero's weapon striking Gnarfas (SPEC-0029 Behavior 10.2), not a boss attack. The 2.5 by 7.0 and 2.5 by 0.25 tests belong to the Werelupe King (SPEC-0019) (High).
18. Resolved: the hero side of the bone-chain test is a vertical cylinder standing on the hero's position (High).
19. Resolved: class 14 runs the Gnarfas coroutine from the per-character class arbiter after the NPC update reset/status work and before clip re-time. The loop writes the animation-speed tunable directly to P at its per-tick start, so non-1.0 values affect that tick's clip rate; the shipped value 1.0 leaves playback at normal rate (controller call path; High).
20. Chain-spark ground test. Resolved: 10.0 is added to the tip's height to form the query's ceiling, so the ground found is the highest surface at or below 10 units above the tip (see Sounds and effects; High).
