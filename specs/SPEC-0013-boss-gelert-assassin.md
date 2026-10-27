# SPEC-0013: Boss behavior: Gelert Assassin

- Answers: proactive (none requested); R-0067 (the approach, strafe and back-up clips, their root travel, and the dagger's hand attachment point); R-0068 (the address of every word in the ten-word melee/strafe group and the 23 shadow-slip bucket words, the light arc's two end objects and the path of its balls, and the mirror attack's lateral axis); R-0098
- Status: Draft
- Disc build: SCUS-97367 v1.00 (same fingerprint as SPEC-0001)
- Confidence: per section. Tuning values and HP thresholds: High (taken from the boot program's data). Unused rows: High for the rows marked unused, including the invisible bucket, the just-appear-mirror word and the invisibility reappear pair. Ranges and chances in the decision rules: High for the numbers, Medium for their intent. Shadow-slip gate: High for the rules and for the source of its request (the navmesh blocked flag). Reappear and landing values: Medium-High (world units). Elapsed-time counter: High. Melee attacks (thrust lunge, sword swing, jump thrust, back-stab): High for the once-per-attack rule, the block test, the damage values (25, 20, 5, 5), the contact shape and the attack windows in frames; Medium for the exact first tick of each window and for the turn helper. Shadow thrust-lunge: unreachable (Medium-High). Rainbow dash (target points, visiting order, speed, hits per point) and light arc damage: High; light-arc hit shape: Medium. Heal arithmetic (including a negative heal): High. Missile and melee reactions: High for the rolls (the second roll is a fresh roll) and the contact shapes; Medium-High for the bone cylinder placement. Dagger throw and shadow dagger: High for the windup, the release event, the launch values (damage, radius, speed, lifetime) and the launch count. Settle and smoothing (4.14, the shared tracked-point rule defined in SPEC-0040): dormant in the shipped build. No state of this boss reaches it (High for the call graph; Medium overall). The rule's constants and steps: High for the 0.02 threshold (a fixed constant), the 0.15 blend, the settled flag and the second latch's set; Medium for the two-point count and the event 0x13 report. Other counters and the call-site details: Low (see Open questions). Sounds (animation cues, the silent clones, the unused die cue) and the code-started effects (heal aura 428, clone puff 559, flinch flash 222): High; the flinch flash point: Medium; clip effect lists: Medium-High.

## Summary

The Gelert Assassin is one boss that fights the hero with melee, thrown daggers, and "shadow" attacks. Every tick it updates its timers, checks its HP band, reacts to incoming missiles and melee swings, and, when it is free to act, chooses between approaching, circling, backing off, throwing, lunging, or slipping into shadow. Shadow slip makes it vanish and reappear as one of several shadow attacks, chosen by a weighted roll that depends on its HP band. The HP band (called a pattern here) only rises: pattern 1 at 75 % HP or more, then 2, 3 and 4 as HP falls. Each band changes which attacks are likely. Its melee hits, rainbow dash and light arc deal fixed damage that the hero's shield and armour do not reduce; only its thrown daggers go through the projectile rule. At patterns 3 and 4 the boss can heal in the shadows, and at those patterns it can summon up to four mirror images that attack from fixed lateral offsets. The shared tracked-point settle rule (4.14, defined in SPEC-0040) checks whether two tracked points have come to rest and smooths them. It is present in the game but dormant in the shipped build: no state of this boss reaches it. Its sounds come only from its animation table, and its code starts three effects: a heal aura, a puff on a hit clone and a hit flash on a flinch (4.15). All timings run on the 60 Hz game tick (SPEC-0007). Times given in seconds become ticks = trunc(seconds × 60). The clean side can build the boss from this spec and the 97 tuning words described below.

### Encounter flow (pack 985; High unless marked)

The active Gelert branch is selected by progress flag 281. On world activation, its callback moves the hero to locator 9 and clears the branch's local stage bits. The streaming filter admits the five Gelert records at ordinals 0 to 4: ordinal 0 is the primary boss in character slot 0, and ordinals 1 to 4 fill its four mirror-image slots (Medium-High). The opening scene again moves the hero to locator 9, starts boss-fight instance 5 at stage 0, blocks player control, presents the boss through its camera and dialogue sequence, then releases control. The exact scheduler phase of the first 280-unit streaming check relative to the arrival callback remains unknown.

The fight's script thread turns the boss's level-state events into four story stages. States 1, 3 and 5 start cutscenes; their completions set states 2, 4 and 6. State 7 starts the final scene, deactivates the primary boss and four clones, stops their effects, ends instance 5 at stage 1, and sets progress states 284 and 301. Afterward, entering trigger 4 runs the glade exit: the hero receives item 30, game states 67, 283 and 302 are set, and movie 44 returns the story to world 74. The boss's attacks and HP patterns remain independent of these camera/dialogue sequences; the level-state events are the bridge between them.

Test path: with flag 281 set, activate the world, place the hero at locator 9, stream records 0 to 4, and run the opening scene. During the fight, process state 1 then 2, state 3 then 4, state 5 then 6, and state 7. The expected result is four stage scenes followed by deactivation of all five Gelert records and instance 5 ending at stage 1. The initial stream-check ordering is not asserted.
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
| Dodge chance, melee (attack-pattern slots 1 to 4 and a mirror-image slot; their meaning is open) | 0x2DAC60–0x2DAC73 | 5 | Used (4.13). |
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
| Elapsed-time index initial value | 0x2DAD64 | float | 1.0 | Set at creation; no later read is known (4.6) |
| Thrust-lunge damage | 0x2DAD80 | int | 25 | HP taken from the hero by a thrust-lunge hit (High) |
| Sword-swing damage | 0x2DAD84 | int | 20 | HP taken from the hero by a sword-swing hit (High) |
| Light-arc loop length | 0x2DAD88 | int | 200 | Ticks (High) |
| Light-arc height offset | 0x2DAD8C | float | 1.5 | Units (Medium) |
| Rainbow-dash damage | 0x2DAD90 | int | 15 | HP taken from the hero by a rainbow-dash hit (High) |
| Dagger speed | 0x2DAD70 | float | 1.5 | Dagger speed in units per tick; also the divisor of the dagger's lifetime (4.4b) (High) |
| Dagger damage | 0x2DAD78 | int | 10 | The dagger's damage (10.0) given to the projectile; also the 10 added to the distance in the dagger's lifetime (High) |

Jump thrust, back-stab and shadow thrust-lunge deal 5 damage, a fixed constant, not a tuning word (High). Light-arc damage is the tuning word at 0x2DAD50 (20) and the rainbow-dash radius the word at 0x2DAD58 (8).

The projectile setup constants are fixed constants, not taken from the table above: 1.25 is the radius of both dagger projectiles (the light arc's setup also carries 1.25; its meaning there is not established); 3.0 is the shadow dagger's damage; 5.0 is the shadow dagger's speed in units per tick and the divisor of its lifetime (High).

The settle test and smoothing constants 0.02, 0.15 and event id 0x13 (4.14) are also fixed constants, not table entries.

## Behavior

### 4.1 Tick order

Every tick (1/60 s), the boss runs these steps in order:

1. Count down each of four timers by one tick (stopping at 0): the shadow-slip timer, the attack timer, the back-up cooldown, and one further counter (the healing countdown copy; Open question 3).
2. Reset the boss's animation speed to 1.0.
3. Script gate (4.3). Then the HP-band check (4.2), which can change the band and can start the death state.
4. Missile reactions and melee reactions (4.13), if the boss is in a state that allows reactions (4.5).
5. Attack choice (4.4), only if the attack timer is 0 and the boss is free to act.
6. Strafe choice and the shadow-slip gate (4.4), only if the boss is free to act.
7. Hit detection against the boss's six hit slots, and reactions to hits (4.13).
8. Run the handler for the boss's current state (4.4b, 4.7 to 4.12).
9. Advance the elapsed-time counter (4.6).

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
- Each band change resets the elapsed-time counter (4.6) to 0.

Confidence: High for thresholds and events. The starting pattern is 1 (4.11).

### 4.3 Script control

- The boss has a script-control flag and a hit counter. The script-control flag is set once, when the boss is created, from one bit of the game's flag table (flag number 258): it is 1 when that flag is set and 0 otherwise. The boss never changes it afterwards. The hit counter rises on each hit while the script-control flag is clear (4.13).
- When the flag is 0, the hit-count event rule below applies and the scripted wait never starts. When the flag is 1, the scripted wait rule applies and the hit-count event rule does not.
- While the script-control flag is clear (normal play), the boss raises script event 0 whenever the hit counter is above 2, or the hero's HP ratio is below 0.55. The check runs every tick, and the event is not latched (Edge cases).
- While the script-control flag is set (scripted), the boss is forced into the wait state whenever script bit 0 is clear, and the boss does nothing else. The wait state plays animation 24 once and holds its last frame. When bit 0 is set, a boss in the wait state returns to idle.
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

When a dagger throw, back-stab, thrust lunge or sword swing ends, T_a is set to trunc(rand(min, max) × 60) for the current pattern's attack-timer pair (the tuning table in Layout).

**Strafe and approach** (decision-ready states; re-checked every tick):

| Distance d | Result |
| --- | --- |
| d > 20 | Approach the hero (approach state; locomotion mode 0, speed from the clip root travel, see Locomotion clips). |
| 15 ≤ d ≤ 20 | If backing up, or already strafing left or right, keep the state. Otherwise pick left or right with a 50 % coin each. |
| d < 15 | If backing up, keep it. If T_b = 0: set T_b = 60 (1 s), then back up with chance 100/101 (any roll below 100). Otherwise, if T_b > 0 and the boss is idle or approaching, pick left or right with a 50 % coin; in other states keep the current state. |

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
- The shadow-slip request latch is set. (The latch is set in every tick in which the boss's navmesh blocked flag is set (SPEC-0056), and is cleared when a shadow slip's reappear begins. The timer, invisibility and clone mode are not checked for this path.)
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
- **Sword blocked** (animation 27): the boss's attack was blocked by the hero. Ends idle. A hit during this state gives a 50 % shadow slip, otherwise a flinch (4.13).
- **Jump thrust** (animation 29): a melee reaction (4.13). Once per attack, the same contact and block tests apply; otherwise a hit of 5 damage is sent. Ends idle.
- **Dagger throw** (animation 31 for the windup; daggers are released while event bit 0x20, event 32, is on). During the windup the boss turns toward the hero with steer weight 0.5 on every tick (Medium).
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

- An elapsed-time counter rises by 2 each tick while it is below 7200 ticks (2 minutes), and by 1 each tick after that. It is not advanced in the scripted wait state (High).
- It is set to 0 on each HP-band change and when a stance ends.
- A derived index is 1 + int(4 × counter / 7200) (High for the formula). Its use is not established (Open question 3).

### 4.7 Shadow slip

Shadow slip is a vanish, a choice, and a reappear. It leads into every shadow attack in 4.8.

1. **Vanish** (phase 0): play animation 24. Its clip effect list shows effect 427 at the boss (4.15); the reappear animation 25 shows the same effect. When it ends, go to phase 1.
2. **Choose** (phase 1): pick the shadow attack with the chooser (4.8), choose the landing point (below), show an effect, go to phase 2.
3. **Reappear** (phase 2): on the phase-1 handler pass, copy the selected landing point into the boss position and advance to phase 2. Then play animation 25. For shadow back-stab and mirror attack the reappear runs at animation speed 2.0. When it ends, the boss enters the chosen attack state.

Landing point: a random horizontal direction; a distance drawn uniformly between the chosen attack's reappear minimum and maximum, in world units from the hero's position. The candidate is accepted if it lies on any navigation mesh in the boss's current world region; this query scans the region's complete mesh list, not only the boss's assigned or current mesh (High). If none of 10 tries is accepted, the chosen attack becomes just appear at the hero's position. Back-stab does not use this search: it lands 3 units behind the hero and tests that point against the same full region mesh list; if it is not on any such mesh, the boss uses the shadow dagger instead. Reappear values are distances (world units), not times. Confidence: High for the all-mesh test and retries; Medium-High for the attack selection and point construction.

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
| Shadow dagger | Throws daggers while event bit 0x20 (event 32) of animation 31 is on, at frames 6, 12 and 18. There is no frame rule: a dagger is launched on every tick the bit is on, 4 ticks per release frame, so 12 launches are attempted per attack; at most 10 can be live at once (pool limit 10). Each is an enemy-pool projectile of kind 0, radius 1.25, damage 3.0, speed 5.0 units per tick (unit direction times 5.0), lifetime trunc((distance + 10) / 5.0) ticks; hits follow the pool rule (SPEC-0029 Behavior 9). Aim point: the hero's position raised 2.5 units (Medium). Landing distance uses the dagger-reappear minimum (Edge cases). (High) | 31 (windup), 32 (release) | Idle; shadow-slip timer re-armed |
| Light arc | Waits, then loops an arc for up to 200 ticks. The arc is two moving ball objects, A and B (the "end objects"; they are spawned effect objects, not level locators and not parts of the boss). At the tick the windup animation (37) ends, both are created at the boss's position raised by the height offset 1.5 (the float word at 0x2DAD8C), and the loop (animation 38) starts. Let d be the horizontal unit direction from the boss to the hero at that tick (4.4) and l = (−d.z, 0, d.x). The direction is fixed for the whole arc; the balls do not home. Ball A flies along (d + l) / √2 and ball B along (d − l) / √2, so their paths diverge at 90 degrees (45 degrees either side of d) at constant height. Each ball moves 20 / 60 units per tick (the speed word at 0x2DAD48 divided by the tick rate), starting on the first loop tick after the launch tick, so after k loop ticks each ball is 0.3333 k units from the start and the segment from A to B is perpendicular to d, centred on the point 0.2357 k units from the start along d, of length 0.4714 k units. The arc ends after 200 loop ticks (about 66.7 units of flight per ball) or at the first hit on the hero. Hit test: each arc ball is a sphere of radius 2.0 tested against the hero's body cylinder (the hero's motion-record radius and height). The arc deals the light-arc damage (20, word at 0x2DAD50) once per arc, unreduced, with attacker id 0. Missile wall (High for the rule and for the sign of n): on every loop tick the arc also cancels hero missiles. Let A and B be the positions of the two ball objects above, u the horizontal unit vector from A to B (u = −l, so u = (d.z, 0, −d.x)), and n = (−u.z, 0, u.x); this makes n equal d, the boss-to-hero direction at the launch, for the whole arc. The wall is therefore the line through A and B, perpendicular to d, advancing along d at 0.2357 units per tick. For each of the hero's six missile slots that is active, with point P and per-tick step s (its direction times its speed times the tick time step), let Q = P + 10 s. If n · (horizontal unit vector from A to P) > 0 and n · (horizontal unit vector from B to Q) < 0, the slot is made inactive at once (the missile is removed with no hit). So a hero missile due to cross the arc's vertical line from the n side within 10 ticks is stopped; the line has no end limits, and a missile crossing the other way passes. High for speed and damage; Medium for the hit shape and the height meaning. | 36 (wait), 37, 38 (loop), 39 | Idle; shadow-slip timer re-armed |
| Shadow healing | See 4.9. | 32, 33, 34, 35 | Idle; shadow-slip timer re-armed |
| Mirror attack | The boss plays 41 at speed 2.0, then 42. When animation 42 passes half-way, the clones waiting in base state 42 are woken (4.10). | 41 (speed 2.0), 42 | Idle; clone mode on (4.10) |
| Rainbow dash | Visits eight target points in a random order (see Rainbow dash below). A hit of 15 damage (word at 0x2DAD90), unreduced and unblockable, is sent on the first tick with the horizontal hero distance below 8.0 (word at 0x2DAD58), once per target point, so up to 8 hits per dash (High). After the eighth point the boss plays animation 45 until it ends, then is idle and its shadow-slip countdown is re-armed. | 44 (points), 45 (end) | Idle; shadow-slip timer re-armed; the pattern is recomputed |
| Start invisible | Plays animation 40, then the boss is invisible. | 40 | See 4.11 |

The block test used in 4.4b: dot(hero facing, boss facing) < −0.5, that is, the two characters face each other (High). The back-stab, rainbow dash and light arc have no block test.

**Rainbow dash** (High):

- Target points: at controller start the eight points are the positions of the level locators with ids 200 to 207. A missing locator leaves its slot unchanged.
- Order: at dash start (sub-step 0) the visiting order is built. For each of the 8 picks a draw r = (output of the shared Mersenne Twister generator) mod 7 (0 to 6) is made; the pick is the first unused index at or after r, cyclically over 0 to 7. So index 7 is reached only by moving on from a used index.
- Motion: each tick of sub-step 1 the boss plays animation 44 and turns toward the target (weight 1.0). It moves straight toward the target point, horizontally, by 40 / 60 units per tick (the rainbow-dash speed word 0x2DAD54, 40, divided by the tick rate 60). It snaps onto the point when the remaining distance is at most one step, then goes on to the next point.
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
- The boss and its clones are the characters of one block: block slot 0 is the boss, and slots 1 to 4 are the clones. The clone flag and the slot index of each character come from its block slot (High). In pack 985 the five Gelert records are consecutive ordinals 0 to 4; ordinal 0 aligns with the primary boss slot, which the level script controls as entity 0, while ordinals 1 to 4 align with the four clone slots (Medium-High for the record-to-slot alignment; see SPEC-0041, module 985).
- At set-up the boss's block is made non-streaming (keep 0; SPEC-0006), so the streaming pass never wakes the clones. Each clone is put into base state 42 (the parked state of SPEC-0040) and AI mode 32, with its clone flag set (High).
- An inactive clone waits in base state 42 in AI mode 32. It is not drawn (SPEC-0046, Drawable test), and its update returns at once (High).
- Wake: while the boss's mirror attack is in its second animation (42) and that animation has passed one half of its length, each clone whose base state is 42 is set to base state 1 and given a forced bind (SPEC-0006). This repeats every tick while the condition holds. The animation-42 test applies to the boss only; the clone test is on the base state (High).
- A woken clone's activation step does nothing while its current animation is 42. Once that animation is no longer active, it places the clone at its slot offset (+5, +10, −5 or −10 lateral), requests animation 42, and assigns controller state 0x16 (decimal 22), the mirror-attack handler (High).
- When animation 42 ends, the boss is idle and clone mode is on. (High.)
- The shadow-mode latch used by the shadow-slip chooser belongs to the current AI instance: the mirror handler sets that instance's latch, and its chooser reads the same latch. A clone running the handler would set its own latch; this does not set a shared flag on the boss or clone group. Whether a clone can enter the mirror-handler state in shipped behavior remains open.
- Clones use the mirror values of the chance tables: dodge 25, dodge-melee 15, jump-thrust 15, dagger bucket 25. The melee shadow bucket uses the same table for clones and bosses.
- A clone never starts a shadow slip itself (4.4).
- A hit on a clone sends it back to the inactive wait: base state 42 and AI mode 32. A one-shot effect 559 (a yellow flash with white sparks) plays at the clone, detached from it (High for the id). The clone is hidden again until the next wake (Medium-High).
- A clone in base state 42 is silent: the animation-sound step does not run in that base state (4.15).
- The clone controller state is 0x20 (decimal 32) while inactive and 0x16 (decimal 22) after activation; state 0x16 is the mirror-attack handler (High).
- While clone mode is on and at least one clone is alive, the boss does not re-arm its shadow-slip timer. The timer is re-armed once no clone remains.

Clone slots are held in the boss's own state. At set-up the boss (block slot 0) collects the characters of block slots 1 to 4 as its clones (High).

### 4.11 Timers and counters

| Timer or counter | Unit | Set by | Remarks |
| --- | --- | --- | --- |
| Shadow-slip timer | ticks | trunc(rand(min, max) × 60), pattern-indexed (Layout) | Re-armed after shadow attacks, healing and just-appear |
| Attack timer | ticks | trunc(rand(min, max) × 60), pattern-indexed (Layout) | Re-armed after dagger throw, back-stab, thrust lunge, sword swing, and shadow thrust-lunge |
| Back-up cooldown | ticks | 60 (1 s) when the close-range check runs with 0 | |
| Back-up cooldown at creation | ticks | 60 (1 s), counted down like the others | The first close-range decision in that second chooses a left or right strafe instead of a back-up (High). |
| Heal countdown | ticks | 5 (tuning word, raw ticks) | Heal every 6 ticks (4.9) |
| Invisible flag | flag | Set when start-invisible ends | Cleared at the end of a lunge or sword swing when no hit was sent (4.4b); cleared by back-stab when it was set and the back-stab's own check allows (Medium) |

Start-up values (High): at creation the pattern is 1. The shadow-slip countdown is set to 8 to 15 seconds (in ticks, times 60: 480 to 900 ticks), and the attack countdown to 2 to 4 seconds (120 to 240 ticks). Both use the pattern-1 ranges.

Invisibility: while invisible, the dagger-throw choice is not allowed, and the shadow-slip timer does not trigger a shadow slip. The start-invisible animation fades the boss with value int((1 − f) × 128), where f is the animation's progress (Medium). The described behavior reaches the invisible state only through back-stab and the end of a lunge or sword swing that sent a hit.

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

**Missile reaction.** Runs when the state allows (4.5). For each active hero missile, its path is the segment from its point to point + 180 × velocity. The path is tested against every bone entry in the boss's active pose whose collision radius is positive; there is no separate named-bone whitelist, and entries with radius 0 or less are skipped (High). Each eligible bone uses its record's full length and radius; the cylinder follows the posed bone's local Y axis and is centered halfway along its length from the posed base (SPEC-0029, Behavior 26; Medium-High for applying this placement to the swept-path caller). This is the same shape convention as the Gnarfas missile reaction (SPEC-0020), with 180 ticks instead of 10. The separate incoming-hit check later tests the missile slot sphere against the boss's vertical body cylinder from its monster row (High); the body cylinder is not the missile-reaction shape. A missile whose hit id is in the ring of the last 10 hit ids is skipped. For a missile whose path passes and whose hit id is new:
- a roll r1 in [0, 100]: if r1 < dodge chance (40 for patterns 1..4; 25 for a mirror image), the boss dodges: state 6, animation 18, or animation 19 when a 0/1 roll is 1 (Medium on the variant roll);
- otherwise a fresh roll r2 in [0, 100]: if r2 < deflect chance (80), the boss deflects: state 7, animation 21 while the missile is within 10 units, or idle animation 0 while no active hit is near (Medium);
- otherwise nothing.

Deflect detail: the 10 units is a fixed constant, not a tuning word. With no active hit slot near the boss, it is idle (animation 0). Confidence: High for the 10 units; Medium for the vector.

**Melee reaction.** Runs when the state allows (4.5), once per swing (a latch is set after one decision; it clears whenever the hero is not attacking). It applies when the hero is attacking and both tests pass:
- facing test: dot(hero facing, direction from the hero to the boss) > the facing threshold stored on the hero;
- reach test: horizontal distance < 1.5 × the radius stored on the hero.

Then:
- a roll r1: if r1 < dodge-melee chance (100 at patterns 1..4; 15 for a mirror image), the boss dodges melee: state 8, animation 20. When that ends, a roll below 50 starts a shadow slip (not for a mirror image), otherwise the boss is idle;
- otherwise a fresh roll r2: if r2 < jump-thrust chance (15, 15, 10, 10 for patterns 1..4; 15 for a mirror image), the boss jumps: state 9, animation 29 (4.4b). Starting the jump thrust also clears the attack's once-per-attack hit flag.

Confidence: High.

**Missile hit.** Each hit slot is tested as a sphere (the slot's point and radius) against the boss's vertical body cylinder (the body record's radius and height, standing on its position), as for Gnarfas (SPEC-0020) (High).

**Hittable states** (High). The boss's own hit check runs only in these states: idle, evil laugh, dagger throw, thrust lunge, sword blocked, shadow thrust-lunge, shadow dagger, mirror attack, just-appear, the five strafe states (idle strafe, right, left, back-up, approach), healing in phases 0, 1 and 3, light arc in its wait (phase 0), loop (phase 2) and end (phase 3), and rainbow dash in all its phases. In every other state a hit does nothing: flinch, the three stances, missile dodge, deflect, melee dodge, jump thrust, back-stab, sword swing, shadow slip, shadow back-stab, invisible attack, start-invisible, death, the two clone wait states, healing phase 2 and the light arc's windup (phase 1). A hit is either a hero melee hit (the hero is attacking, the attack is active and the hero's weapon touches the boss) or a hero missile slot whose sphere overlaps the body cylinder; each overlapping missile slot is removed.

**Hit reaction.** A hit on the boss in a hittable state when the boss is not a clone:
- if the boss is in the sword-blocked state, a roll below 50 starts a shadow slip (4.7), otherwise a flinch;
- during the light arc, the arc is removed (its arc balls are freed) and the boss flinches (High for the flinch, Medium for the removal);
- otherwise, a flinch (animation 16, or 17 when a 0/1 roll is 1). The flinch plays the shared hit flash at the boss: one-shot effect 222, detached (SPEC-0034; the same helper shows 220 instead when called with its flag set, which the flinch does not do) (High for the id; Medium for the point);
- during healing, a hit moves healing to phase 2 (4.9);
- while script control is clear, the hit counter rises by 1 for each hero missile slot that hit, and by 1 more when the flinch starts, except for a flinch out of the light arc (High). A melee hit therefore adds 1 and a single missile hit adds 2; a hit that moves healing to phase 2 or starts the shadow slip adds only its missile count (4.3);
- a hit on a clone sends it back to the inactive wait (base state 42, AI mode 32, with the one-shot puff effect 559 at the clone; 4.10).

Confidence: High for chances, the gate, the rolls, the facing and reach tests, and the positive-radius eligibility rule; Medium-High for the bone cylinder placement in the missile reaction (Open question 7).

### 4.14 Settle test and tracked-point smoothing (dormant in the shipped build)

The shared tracked-point settle rule is defined in SPEC-0040. It is not specific to Gelert. In the shipped build no state of this boss reaches it. The dispatcher that selects the two-point settle and smoothing has no caller. The four-point variant and the second move routine have no caller at all, and no code pointer in the boot program refers to any of these routines (High for the call graph; Medium overall, because a call made through a data pointer of an unrecognised form is not ruled out). A host export that could reach the rule has not been identified (Low). The rule is kept here as a reference only. Implement it as a reference, or leave it out. A reimplementation that wires it to the Gelert states would change behavior (Edge cases).

The values below are those of the two-point path.

Each tracked point has a saved copy. The rule uses the same "moving" test for every point.

**Moving test.** A tracked point is moving when any of its three axes differs from its saved copy by more than 0.02 units (a fixed constant, not a tuning word). A point that is not moving is stationary.

**Settle and smoothing (two points).** The rule runs only while the settled flag is clear. For each of the two tracked points, in order:
1. Apply the moving test.
2. If the point is moving, its saved copy moves 15 percent of the way toward the current position: saved = saved + 0.15 × (current − saved), per axis. The 0.15 is a fixed constant.
3. If the point is stationary, its saved copy is set exactly to the current position.
4. The updated saved copy is written to tracked-point slot 0x13 (the first point) or slot 0x14 (the second point), through the event 0x13 handler, which stores the four-float vector in that slot (Open question 27).
5. If neither point was moving on this run, set the settled flag.

If the settled flag is already set when the rule is reached, the rule does not run. A second latch is set to 1 instead. The second latch is also set to 1 when the object is created. Its meaning is open (Open question 23). Only the dormant move-start routine clears the settled flag and the second latch (Open question 24).

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

## Edge cases and error handling

- **Settle rule is dormant** (High for the call graph): the settle test and the smoothing (4.14) do not run in the shipped build. The drift, jump and snap cases below describe the rule only and do not occur in play.
- **Pattern cannot fall back**: the pattern only rises. A healed boss keeps its pattern, so a pattern-4 boss healed to 80 % still uses pattern-4 timers and chances.
- **Band changes wait for a decision**: the band check runs only while decision-ready. A boss that enters pattern 4 during a swing starts the injured stance after the swing.
- **Stance deadlock**: the three stances wait for script bits 2, 4 and 6. If the script does not set the bit, the boss stays in the stance indefinitely.
- **Event 0 is not latched**: while the hit counter is above 2, or the hero's HP ratio is below 0.55, event 0 is raised on every tick.
- **Scripted-mode override**: if script control is set and script bit 0 is clear, the boss is forced into the wait state on every tick and cannot start any action.
- **Shadow timer alone does not start a slip**: with script control clear, the shadow-slip countdown triggers nothing. A slip then starts only from the request latch (set while the boss's navmesh blocked flag is on, that is within 120 ticks (2 s) after an arena-edge push or an off-mesh restore of the boss; SPEC-0056) or from one of the two 50 % rolls (dodge-melee and sword-blocked hit).
- **Melee dodge is almost always chosen**: the dodge-melee chance is 100 at every pattern. Because the roll is 0–100 inclusive, the boss dodges an in-range swing with 100/101 chance. Only a roll of exactly 100 falls through to the jump-thrust check, which then needs a fresh roll below 15 (or 10 at patterns 3 and 4) (High). Jump-thrust is therefore rare in practice.
- **Lunge without contact**: a thrust lunge, sword swing or jump thrust whose weapon does not touch the hero on a tick sends nothing that tick and keeps its once-per-attack flag clear. The test is repeated on every tick of the window (event bit 0x1), so a hit can land on the last window tick; outside the window nothing can hit (High).
- **Shadow thrust-lunge is unreachable** (Medium-High): no shadow-attack chooser and no other state writes the shadow thrust-lunge state, so it never runs in play. Its 5-damage lunge exists only as unused behavior. Keep it unreachable; do not add it to a chooser.
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
- **Invisibility is mostly unreachable**: the described behavior reaches the invisible state only through back-stab and through a lunge or sword swing that ends after a hit was sent (4.4b, 4.11).
- **Dagger windup and the pending flag**: the dagger throw's windup sets the evil-laugh pending flag only while the hero is in hit-reaction animation 32 (4.3). The evil-laugh roll can therefore follow a dagger throw, but not a throw at a hero who is not reeling.
- **Drift past the threshold**: a point that drifts slowly is never moving (4.14), so a settle can complete while the point has moved a long way from where it started. The settle test does not measure total displacement.
- **A jump delays the settle**: a point that jumps more than 0.02 units holds the settle back until its saved copy has closed to within 0.02 units, which takes about 25 runs for a one-unit jump (Medium; follows from the rule in 4.14).
- **Silent death and evil laugh** (High): the die cue (cue 3 of block 386) is named by no animation, so the boss dies without a death sound, and the evil-laugh animation 23 has no sound word. Reproduce this; do not add the cues.
- **Hidden clones are silent** (High): a clone in base state 42 runs no animation-sound step; it plays cues only after it is woken.
- **Smoothing snaps when close**: a moving point whose saved copy comes within 0.02 units of its position counts as stationary on the next run, and its saved copy is then set exactly to the position (4.14). This follows from the same moving test (Medium).

## Test vectors

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
- Step: 40 / 60 = 0.6667 units per tick; a point 1.8 units away is reached in 3 ticks (two full steps, then the snap at 0.467 remaining).

Clones:

- Slot 1 at +5 units, slot 2 at +10, slot 3 at −5, slot 4 at −10 along the lateral direction.

- Missile reaction: a bone record with full length 2.0 and radius 0.5 produces a finite cylinder centered 1.0 unit along posed local Y from its base, with radius 0.5; radius 0 means that bone is skipped. Every positive-radius bone is considered. A path missing all positive-radius bones causes no missile dodge/deflect roll.
- Landing search: if a candidate lies only on mesh 2 while the boss is currently assigned to mesh 0, the candidate is accepted because the query scans all meshes in the region. Ten candidates outside every mesh fall back to the hero position and just-appear. A back-stab point outside every mesh falls back to shadow dagger.

Shadow-slip request latch:

- The boss, decision-ready and not a mirror image, is pushed back by a navmesh boundary edge in the motion step of tick n and not again. Its blocked flag is set on ticks n + 1 to n + 120, so the latch is set on each of those ticks, and the latch path of the shadow-slip gate (4.4) applies from tick n + 1. A boss that never touches a navmesh edge and is never restored onto the mesh never sets the latch, wherever the hero stands.

## Open questions

- Default if unresolved (rewrite choice): the words listed as unused in "Word order of the two groups" (including 0x2DACA0 and 0x2DACA4) may have a reader in code not reachable from the described states. Store them, and apply none of them.
- Resolved (R-0067, side of root x): in modes 5 and 6 the shared steering turns the boss's move direction toward the hero (SPEC-0029, Behavior 2: with d the direction to the side target, mode 5 steers to up x d and mode 6 to d x up, which is the direction to the hero), and the root step of a clip is applied along the record's side axis, up x forward (SPEC-0040, Behavior 14C), a positive root x going along it. The table entries of these clips carry no mirror flag. With the boss facing +Z that axis is +X, which is the boss's left (SPEC-0004: the right side is -X). So the mode 5 clip, whose sideways root is positive, moves the boss to its left of the line to the hero, and the mode 6 clip, whose sideways root is negative, to its right. This is the opposite of the side point that the target names; that point only feeds the steering (High for the rule; Medium-High for what shows on screen). Test vector: boss at (0, 0, 0) and hero at (0, 0, 10): mode 5 moves toward +X and mode 6 toward -X, while the mode 5 target point is (-3, 0, 0) and the mode 6 target point is (3, 0, 0). Gelert Assassin: the strafe right state (mode 5, positive sideways root) moves the boss to its left of the line to the hero and the strafe left state (mode 6, negative) to its right; the strafe-right target point, 3 units to the boss's right, only feeds the steering.

Numbers of removed questions are not reused.

1. **Runtime overrides.** Resolved (High that no disc file or level module overrides them; Medium-High for the PC link): no. The tuning words keep the values at their data addresses for the whole session. No disc file holds their names or values, no loader fills them, and no level module can name them. The only code that can write a registered word at run time is the registry's link to an external PC, and nothing in the shipped game starts that link. Test vector: the attack timer ranges of patterns 1 to 4 are 2 to 4, 2 to 3, 1 to 3 and 1 to 2 seconds in every fight of every session; there is no second set.
3. **Elapsed-time index and the healing countdown copy.** The derived index (1..5) has no known reader. The counter in the healing-countdown slot is set at creation and never read by boss logic.
4. **Damage values.** Resolved: each value is the damage itself (4.12; High).
5. **Healing length and factor.** Healing length and healing factor are not used by any described behavior. The healing timer pair is loaded only at creation (Layout).
6. **Negative heal.** Resolved: a heal caps HP at max HP with no lower clamp and no death call, so a negative amount lowers HP toward the target (4.9; High).
7. **Bone cylinder placement.** Resolved: every bone entry in the active pose with positive radius is eligible for the incoming missile path test; there is no fixed named-bone subset. A nonpositive radius skips that bone. The bone record's length is its full cylinder length and its radius is the cylinder radius; the cylinder follows posed local Y and is centered halfway along its length from the posed base (4.13, SPEC-0029 Behavior 26). Confidence is High for eligibility and field meanings, Medium-High for applying the shared placement to this swept-path caller. The separate incoming-hit check uses the boss body cylinder. The missile path and missile hit tests and the 10-unit deflect distance are also in 4.13 (High).
8. **Facing and block tests.** Resolved: the block test is dot(hero facing, boss facing) < −0.5 (4.4b), and the melee reaction's facing and reach tests are in 4.13 (High).
9. **Clone mirror attack.** Resolved: the mirror handler sets the current AI instance's shadow-mode latch, and its chooser reads that same instance-local latch. It does not set a shared boss/group flag. After wake activation, a clone is assigned controller state 0x16 (decimal 22), the mirror-attack handler (4.10). The shadow-mode latch remains instance-local.
10. **Rainbow target points.** Resolved: the level locators with ids 200 to 207, visited in a permutation built from draws mod 7 (4.8, Rainbow dash; High).
11. **Flinch conditions.** Resolved: the boss's hit check runs only in the hittable states listed in 4.13; a hit there flinches the boss except in healing (moves to phase 2) and the sword-blocked shadow-slip roll, and a hit during the light arc removes the arc and flinches (High; Medium for the arc removal).
12. **Invisibility reappear pair.** Resolved: the words 0x2DAD1C and 0x2DAD20 have no reader, so both are unused (High).
13. **Invisible bucket, clone backstab, just-appear mirror and other unused bucket words.** Resolved: 0x2DACD8 to 0x2DACE0, 0x2DACC0 and 0x2DAD08 are read only by getters that have no caller; the pattern 1 words, 0x2DACF4 and 0x2DAD00 are not read by the chooser of their pattern. All are unused (High). The full list is in "Word order of the two groups".
14. **Melee contact test.** Resolved: weapon 1 of monster row 4 (radius 6.0, full length 1.75), tested against the hero's bone capsules while event bit 0x1 is on (4.4b; High).
15. **Shadow thrust-lunge timers.** Resolved (High): the two re-arms sit in the branch taken on the tick the lunge animation (28) reaches 0.99 progress, the same branch that leaves the state. That branch sets idle (or, when the invisible flag is set and a hit was already sent, the start-invisible state; when the invisible flag is set and no hit was sent, it clears the flag), and then, in every one of those cases, draws the shadow-slip countdown first and the attack countdown second, each as trunc(rand(min, max) x 60) from the current pattern's ranges (4.11). A lunge that is blocked leaves through the sword-blocked state instead and does not reach this branch. Test vector: pattern 1, attack range 2 to 4 s: when the shadow thrust-lunge ends the attack countdown becomes a whole number from 120 to 240 (trunc(rand x 60) of a value in 2 to 4 s), and the shadow-slip countdown is drawn from the pattern's slip range on the same tick, before it.
16. **Thrust-lunge and sword-swing timers.** Resolved (High): both handlers re-arm only the attack countdown (never the shadow-slip countdown), in the branch taken on the tick their animation (28 for the lunge, 26 for the swing) reaches 0.99 progress. The branch first sets idle (or the invisible-flag cases of item 15), then draws the attack countdown from the current pattern's range. A tick on which the hit is blocked leaves through the sword-blocked state instead. Test vector: a sword swing whose animation reaches 0.99 on tick t leaves the attack countdown at its fresh draw on tick t (for pattern 1, 120 to 240), and the shadow-slip countdown is unchanged.
17. **Evil-laugh pending flag.** Resolved: the second test checks the hero, not the boss: the flag is set while the windup (31) plays unfinished and the hero's current animation is 32, a hero hit reaction (4.3; High for the test, Medium-High for the meaning of 32).
18. **Steer helper.** The turn helper in the lunge, sword swing and dagger throw is called with weights 1.0 (at the start of the thrust lunge), 0.5 (every tick of the sword swing, including before its animation, and every windup tick of the dagger throw). Its direction is the horizontal unit vector from the boss to the hero. The weights are fixed constants. Resolved (High for the identity of the helper, Medium-High for the call timing): the helper is the shared aim rule of SPEC-0040 (Behavior 8, Aim rule), the same character method that the Faerie Sisters' controllers and the class-37 recovery routine call to turn toward a target. With stored facing c, target direction t and weight w, the new facing is normalise(w t + (1 - w) c); a weight of 1.0 snaps the facing to t and 0.5 halves the remaining angle on each call. It changes no animation rate. Test vector: boss facing (1, 0, 0), hero to its side so that t = (0, 0, 1). Weight 0.5: facing (0.7071, 0, 0.7071) after one tick and (0.3827, 0, 0.9239) after two. Weight 1.0: facing (0, 0, 1) at once.
19. **Boss-to-hero helper.** The helper returns the horizontal unit direction from the boss to the hero (y forced to zero) and the horizontal distance. Every range test in this spec reads that distance (High).
20. **Release-event latch.** Resolved: bit 0x20 is on for 4 ticks at each of frames 6, 12 and 18, so a dagger throw launches 3 daggers and a shadow dagger attempts 12 launches (4.4b, 4.8; High).
21. **Launch velocity units.** Resolved: units per tick; 10 and 3.0 are the damage, 1.25 the radius, and the lifetime is trunc((distance + 10) / speed) ticks (4.4b; High).
22. **Which states use the settle test and the smoothing.** None in the shipped build. The dispatcher that selects the two-point settle has no caller, the four-point variant and the second move routine have no caller, and no code pointer in the boot program refers to them (High for the call graph; Medium overall). A host export that could reach the rule has not been identified (Low).
23. **Meaning of the second latch.** Its reader is not known (Medium). Answered: its setting and clearing are in 4.14.
24. **Clearing the settled flag.** Resolved: it is cleared only by the dormant move-start routine, so no clear is reached in the shipped build (Medium-High).
25. **Gate of the smoothing.** Resolved: the gate is the settled flag itself, and nothing in the shipped build reaches it (4.14; High).
26. **Blend direction.** When a point has moved more than 0.02 on some axis, its saved copy becomes 85 percent of itself plus 15 percent of the current position; otherwise the saved copy takes the current position (4.14; High for the constant and the direction; Medium-High for the order of the vector blend's weights).
27. **Event 0x13.** The handler stores a four-float vector in the object's point table. The table is indexed in 32-byte entries, and the event number selects the entry. Index 0x13 is the first tracked point and index 0x14 the second (High). The handler's argument list is not fully established.
28. **Which tracked points.** The two points of the default rule are tracked-point slots 0x13 and 0x14 of the object's point table (SPEC-0040). Which part of the boss's model each one follows is not identified. The "current" input that the rule reads is written outside the settle routine (Open).
29. **Float constants.** The 0.02 thresholds and the 0.15 blend are fixed constants of the two-point routine (Medium-High). The same float patterns also occur as aligned words in data at places not linked to these routines, so a data read is not ruled out (Low).
30. **Four-point variant.** The shared family has a four-point routine (tracked-point slots 8, 9, 0x10 and 0x11, SPEC-0040), but it has no 0.02 test and no caller in the boot program. No Gelert state reaches it (High).
31. **Rainbow-dash latch.** Resolved: cleared at the dash start and at every target point, so up to eight hits per dash (4.8; High).
32. **Light-arc third object.** Resolved: the third test is not a cylinder against the hero; it is the arc's missile wall, a side test of each active hero missile (now and 10 ticks ahead) against the vertical line through the arc's two end objects, which removes a crossing missile (4.8, Light arc; High for the rule, Medium-High for the side sign).
33. Resolved: **first tick of a window.** The clip starts on the request tick (tick 0) at cursor 0 and is advanced to 0.25 in that same tick; the fade-in does not move the cursor. The window test reads the frame before each tick's advance, so frame f is read on ticks 4f to 4f + 3, except frame 0, which is read on ticks 1 to 3 (4.4b; Medium-High).
34. **Weapon attachment.** (The dagger's hand point is attachment 1; see the dagger throw.) Resolved (High for the table entries; Medium-High for the body-part names): the Gelert Assassin's attach-point table (SPEC-0004, SPEC-0005) is the same in all three of its models (regions 741, 755 and 1258). Attachment 0, the anchor of weapon 1, follows bone 21 with offset (-0.02, -0.017, 0) and no rotation. Bone 21 is the end of the arm on the -x side (model-space rest position (-2.49, 3.45, 0.02)), the sword hand. Attachment 1, the dagger's hand point, follows bone 11 with offset (0.284, -0.14, 0): the end of the mirrored arm on the +x side (rest position (2.21, 3.59, 0.02)). Attachments 2 to 4 follow the root bone with a zero offset, so weapons 3 to 5 would sit at the model origin (the boss's weapon test uses weapon 1 only). The clean side reads the bone numbers and offsets from the model on the disc. Test vector: with the boss unrotated at the origin in its rest pose, attachment 0's origin is (-2.512, 3.437, 0.025) and attachment 1's origin is (2.492, 3.454, 0.025).
35. **Row 5.** Monster row 5 (weapon 1 radius 5.5, length 4.0, no class) is used by regions 741 and 1258. Whether either region holds a Gelert fight, and which weapon sizes such a fight would use, is not established. The Gelert records of both regions carry no sound words.
36. **Sound step gates and the flinch point.** The boss is taken to be a world character like every character spawned from a level's character records, so the world-character sound rule applies (Medium-High). Two further gates of the animation-sound step, a motion-block gate and a per-character mute word, are not named. The exact point of the flinch hit flash is Medium.
37. **Dagger trail field.** Resolved: the word is not an effect id. It is entry 1 of the boss's table of 11 extra model parts: while a model set loads, each child of its mesh container whose code is 10 to 20 sets entry (code − 10) of an 11-word table to that child's data, and every other entry stays 0. Both dagger throws copy entry 1 into the projectile's trail word. The projectile pool's draw pass, once per frame for each live slot whose visual kind is below 5 (the daggers use visual kind 0) and whose trail word is non-zero, submits that part at the slot's position with the slot's frame (forward axis and the world up vector), so a dagger is drawn as model part 1 of the boss's set (the mesh child with code 11) and no particle effect starts. When the set has no such child, entry 1 is 0 and nothing is drawn for the dagger (High for the identity of the word and the draw gate; Medium for the look of the drawn part). On the disc, the fight region of the boss (the one with the cue table above) holds one model set whose mesh container has three children: the main body mesh with code 1 (about 361,000 bytes), a part with code 10 (7,088 bytes) and a part with code 11 (3,840 bytes, a single small mesh tree). So entry 1 of the table is the code-11 part, and entry 0 is the code-10 part, which nothing in the boss's code requests. The part is shown only while a dagger slot is live: the slot's draw each frame submits it at the slot position, with the slot's forward axis and the world up vector, and it disappears when the slot ends, with nothing to hide separately (Medium-High); the boss's body is drawn from the main mesh, not from this part. Test vectors: a set with a child of code 11: the dagger slot's trail word is that part and it is drawn each frame until the slot ends. A set with no child of code 11: the word is 0 and the pool draws nothing for the slot, although the dagger still flies and hits.
38. **Empty clip effect entries.** Resolved: the entries of animations 18, 19, 26 and 27 (effect id -1, attach slots 0 and 0) start no particle effect, but each is a swing-trail entry: its trail flag byte is 0 (SPEC-0034, Clip effect list entry and Swing trail). While each of these animations plays, a ribbon is drawn between point 1 and point 2: point 1 is the origin of attach slot 0 (the sword hand), and point 2 is the point (0, 0, 5.5) of the same slot's frame, so the ribbon runs 5.5 units along the slot's forward axis (the blade). It runs over the whole clip (start 0.0, end 1.0). Its colour record is chosen by the attack slot's alignment while an attack slot is selected, otherwise by the monster row's alignment (SPEC-0034, Swing trail). The Gelert is a region character, so the clip walk that starts the trail does run for it (High for the entries, the flag and the geometry; Medium-High for the drawn result). Test vector: animation 26 (sword swing) while the boss is in its sword-swing state: one trail starts on the first pass with edge points (slot 0 origin) and (slot 0 frame point (0, 0, 5.5)) and takes one of the 12 trail slots; if all 12 are taken nothing starts.
- 39. **R-0098 streaming and encounter flow.** Resolved in part: with progress flag 281 set, pack 985 moves the hero to locator 9, admits Gelert records 0–4, maps ordinal 0 to the primary boss and 1–4 to the clone slots, then presents the opening scene and starts instance 5 at stage 0. The level thread translates boss states 1, 3 and 5 into the next three staged cutscenes, then state 7 into the finale, clone/boss deactivation and instance 5 stage 1. Trigger 4 completes the glade exit and return. The exact scheduler phase of the first 280-unit stream check relative to the arrival callback remains unknown (High except slot alignment Medium-High; see Encounter flow).

