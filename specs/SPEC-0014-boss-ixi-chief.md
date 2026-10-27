# SPEC-0014: Boss behavior: Ixi Chief

- Answers: proactive, none requested (boss AI); R-0067 (the locomotion slot values, clips and root travel of modes 0, 1, 3, 5 and 6, and the charge walk clip); R-0088 (navigation steering, the charge start wait and charge-point validity)
- Status: Draft
- Disc build: SCUS-97367 v1.00 (same fingerprint as SPEC-0001)
- Confidence: per section. Tuning values and addresses: High (the defaults are words of the boot program data). Fight mode and stage/HP rules: High. Mode-2 flag source (game state 358, set by the final fight's script pack): High; whether mode 2 is reached in play: Medium. Stage and fight mode as separate values: High. Lava quirk (mechanism): High. Level-flag interaction: High for the flags, Medium for which level script reacts to them. Attack choice and cooldowns: High. Charge sub-steps, exits and navigation steering: High; charge-point containment ignores height (High); the pack-800 stuck scenario follows from static rules (Medium-High). Push: High for the formula, Medium for which hits use it. Incoming-hit rules: High for the control flow. Missile contact: Medium-High (per-bone spheres with radius above zero; the sphere geometry is Medium; the projectile path length of 120 ticks is Medium). Orbit: High for the mode numbers and the 30-tick flip; the 3.0-unit offset and its side sign are High. Buck kick hit: High (monster weapons 4 and 5). Movement-blocking flag (lock word): High. Knockback lanes: High (the knocked state's own effect belongs to the hero, SPEC-0036). Attack shapes, windows, damage path and block rule (Hit shapes and windows): High. Sounds (animation cues, the unused counter-attack cue) and the incoming-hit flash 222: High; the flash point: Medium.

## Summary

The Ixi Chief is a melee boss that answers incoming missiles and melee attacks with a block or a dodge, counters some attacks, and chooses among a charge, two swipes, a buck kick and (in fight mode 1) a whirl. Movement keeps it inside a distance band from the hero. Its HP decides three stages. In fight mode 1 it also sets off two lava events at HP fractions 0.66 and 0.33 (the second one can never fire, see Behavior). The fight mode is chosen once when the fight starts from two story progress flags. Its sounds come only from its animation table, and each counted hit on it shows a hit flash (see Sounds and effects). Times are seconds on the 60 Hz game tick (SPEC-0007): a stored time T seconds becomes ticks = trunc(T x 60).

## Where it lives on the disc

- The tuning values are 32-bit words in the boot program's data. The clean side reads them at runtime from the boot executable by their virtual address (SPEC-0001 gives the fingerprint). File offset = virtual address - 0xFFF80 (the loadable segment has file offset 0x80). The values below are the defaults that the boot program ships with.
- All addresses below are in the 0x2DAE90 to 0x2DAF48 range of the boot program's data.
- "Animation n" means entry n of the boss's own animation table (SPEC-0005).
- The hero is the player's character object. "Distance" in this spec is the horizontal distance between the boss and the hero (see Cross-boss facts).
- Level flags: the boss talks to the level through numbered level flags (the level scripts read and write the same flags, SPEC-0030). "Post script event n" in this spec means set level flag n.

## Cross-boss facts (apply to SPEC-0014, 0015, 0016, 0019, 0020)

- Distances are horizontal. Every boss-to-hero or boss-to-home distance is sqrt(dx^2 + dz^2); height is ignored. Directions used for facing and aiming are horizontal unit vectors. High.
- Script events are level flags. Posting event n sets level flag n; the level's scripts read the same flags. High.
- Action-frame (attack-active) queries. Every non-hero boss is an AI-controlled character (only three character method tables exist: base, hero and AI-controlled character). When a boss asks the action-frame query of itself, the answer is true when any of animation event bits 0x1 to 0x10 is set on its current frame. When it asks the same query of the hero, the answer is the hero's attack-active word (event bit 0x1 of the hero's current frame, SPEC-0036). High.
- The random generator is one shared Mersenne Twister (the standard 624-entry generator with standard tempering). Its state object is at ELF data address 0x39BE50, in run-time memory outside the file image: 624 entries held in 8-byte slots, with an index word at 0x39D1D0. A start-up initialiser sets the index word to 625, which means not seeded. The first draw then seeds the generator from the BCD date and time bytes of the console's real-time clock. The range helpers (integer range, integer modulo, real range) all pass the state object on to the same core draw. It is not re-seeded per fight. High. Draws already occur during start-up, so the seed is taken before play begins (Medium).
- Boss sounds come from animation tables only. No boss controller plays its own voice or attack cues. Each boss's main animation table carries sound words that name cues of its creature block (sound group 1; block = 350 + block index): Ixi Chieftain 392 (with 373, the Ixi warrior block), Juppie 389, Plague Serpent 366, Werelupe King 367, Gnarfas 393. Every boss sound entry has start frame 0 and no keep-alive bit, so the cue plays once at the boss, on the first tick of the animation, when the main animation id changes to that entry. The step does not run while the character's base state is 42 (SPEC-0031). High.

## Layout

### Tuning values

Type: "f32" = IEEE single, "i32" = signed 32-bit integer. A percent value is an i32 P used in the percent rule under Behavior.

| Parameter | Type | Address | Default | Meaning |
| --- | --- | --- | --- | --- |
| Lava trigger 1 | f32 | 0x2DAE90 | 0.66 | HP fraction for the first lava event (fight mode 1) |
| Lava trigger 2 | f32 | 0x2DAE94 | 0.33 | HP fraction for the second lava event (fight mode 1; never fires, see Behavior) |
| Missile-defense cooldown min | f32 | 0x2DAE98 | 0.5 s | |
| Missile-defense cooldown max | f32 | 0x2DAE9C | 1.0 s | |
| Missile block chance | i32 % | 0x2DAEA0 | 25 | First roll after a missile contact |
| Missile dodge chance | i32 % | 0x2DAEA4 | 25 | Second roll, if the block roll failed |
| Melee block chance | i32 % | 0x2DAEAC | 25 | Melee reaction, second roll |
| Melee dodge chance | i32 % | 0x2DAEB4 | 25 | Melee reaction, first roll |
| Counter-attack damage | i32 | 0x2DAEBC | 25 | Before scale |
| Charge retreat distance min | f32 | 0x2DAEC0 | 10.0 | Distance of the charge's candidate points from the hero |
| Charge retreat distance max | f32 | 0x2DAEC4 | 20.0 | |
| Charge cooldown min | f32 | 0x2DAEC8 | 1.0 s | |
| Charge cooldown max | f32 | 0x2DAECC | 3.0 s | |
| Charge slam chance | i32 % | 0x2DAED0 | 40 | Chance that a charge ends in the stop-and-slam |
| Stop-and-slam damage | i32 | 0x2DAED8 | 100 | Before scale |
| Forward swipe range | f32 | 0x2DAEE4 | 7.0 | |
| Forward swipe cooldown min | f32 | 0x2DAEE8 | 1.0 s | |
| Forward swipe cooldown max | f32 | 0x2DAEEC | 3.0 s | |
| Forward swipe chance | i32 % | 0x2DAEF0 | 50 | |
| Forward swipe damage | i32 | 0x2DAEF4 | 50 | Before scale |
| Wide swipe cooldown min | f32 | 0x2DAEF8 | 1.0 s | |
| Wide swipe cooldown max | f32 | 0x2DAEFC | 3.0 s | |
| Wide swipe chance, fight mode 0 | i32 % | 0x2DAF00 | 25 | |
| Wide swipe chance, fight modes 1 and 2 | i32 % | 0x2DAF04 | 50 | |
| Wide swipe damage | i32 | 0x2DAF08 | 35 | Before scale |
| Buck kick range | f32 | 0x2DAF0C | 4.0 | |
| Buck kick damage | i32 | 0x2DAF14 | 25 | Before scale |
| Whirl trigger range | f32 | 0x2DAF18 | 15.0 | Whirl starts only at distance below this |
| Whirl hit range | f32 | 0x2DAF1C | 12.0 | The whirl hits only within this distance |
| Whirl cooldown min | f32 | 0x2DAF24 | 2.0 s | |
| Whirl cooldown max | f32 | 0x2DAF28 | 4.0 s | |
| Whirl chance | i32 % | 0x2DAF2C | 40 | |
| Whirl damage | i32 | 0x2DAF30 | 100 | Before scale |
| Whirl push speed | f32 | 0x2DAF20 | 5.0 | Total push speed K of the hero when the whirl hits: 3.0 away plus 4.0 up (used) |
| Rush-forward distance | f32 | 0x2DAF38 | 15.0 | Movement band (see Behavior) |
| Strafe-close distance | f32 | 0x2DAF3C | 10.0 | Movement band |
| Strafe-back distance | f32 | 0x2DAF40 | 5.0 | Movement band |

Constant not in the tuning list, also read at runtime:

| Constant | Type | Address | Value | Meaning |
| --- | --- | --- | --- | --- |
| Base damage scale | f32 | 0x2DAF48 | 0.25 | Damage scale in fight mode 0 (see Damage) |

Tuning values that the boot program registers but the behavior does not read are listed under Edge cases. Do not apply them.

### Per-fight state (not on disc)

| Field | Meaning |
| --- | --- |
| Current state | One of the states in the table under Behavior |
| Charge sub-step | 0 to 6 |
| Stage | 0 to 2, see Behavior, "Stages and modes". Stage and fight mode are separate values |
| Fight mode | 0, 1 or 2, chosen once at fight start (see Behavior) |
| Lava latches | two flags, one per lava trigger |
| Cooldowns | missile defense, charge, forward swipe, wide swipe, whirl (each in ticks) |
| Hit latch | one hit per attack |
| Melee-reaction latch | set when a melee reaction was evaluated, cleared when the hero stops attacking |
| Dodge direction | 0 or 1, picked at random |
| Orbit direction and timer | 0 or 1; 30-tick timer |
| Retreat distance | float, the charge's candidate distance |
| Charge target | a point in world space |
| Damage scale | 0.25 or 1.0 (see Damage) |

## Behavior

### Random numbers

- A percent roll is an integer drawn uniformly from 0 to 100 **inclusive** (101 values). The event with chance P happens when the roll is less than P. So the probability is P/101, not P/100. A 25 % chance is therefore 25/101 (about 24.8 %).
- A tick cooldown with range (min, max) in seconds is an integer drawn uniformly, inclusive, from trunc(min x 60) to trunc(max x 60).
- A real-valued range (min, max) is drawn uniformly from the closed interval.
- A coin flip is a uniform integer from 0 to 1 inclusive.

### Fight mode (chosen once)

At fight start the boss reads two story progress flags and picks a fight mode. It never changes afterwards.

- Progress flag 0x166 set: mode 2.
- Else progress flag 0x3C set: mode 1.
- Else: mode 0.

The Ixi fight's level script sets flag 0x3C when the level loads, so the normal fight is mode 1. No known level runs the fight without it (Open questions). The damage scale is 0.25 in mode 0 and 1.0 in modes 1 and 2.

Mode 2 (High for the flag, Medium for its use): progress flag 0x166 is game state 358, which the fight reads through the game-state query (SPEC-0030). The only writer on the disc is the init of script pack 1432 (the final fight), which sets it. So mode 2 applies to an Ixi Chief whose controller starts after the final fight has begun. Whether that world places an Ixi Chief is not established (Open questions).

### Each tick, in order

1. Count down each cooldown that is above 0 by one tick (missile defense, charge, forward swipe, wide swipe, whirl). A cooldown stops at 0.
2. Stage and mode rules (see Stages and modes).
3. If the boss is free (see Busy and free), run, in order: missile reaction, melee reaction, the contact check, the attack chooser, and the movement band.
4. Run the handler of the current state.

### Busy and free

- A state is **free** when the boss is idle or in one of the movement states (rush forward, strafe close, strafe back, orbit). Every other state is **busy**, including the attack states, the reactions, get-hit, lava, the stage-transition state, the hold states and death.
- The missile and melee reactions are evaluated when the boss is free, and also in the charge (except its run-past and start sub-steps), counter-attack, forward-swipe, wide-swipe, whirl and buck-kick states. In all other busy states they are not evaluated.
- Attack choice runs only while the boss is free.

### Stages and modes

Stage 0 is the dormant stage. The boss is held in its hold state (animation 7, no attack choice) until level flag 0 is set. Then it moves to stage 1: it resets to idle and rolls three cooldowns: charge (1.0 to 3.0 s), forward swipe (1.0 to 3.0 s) and wide swipe (1.0 to 3.0 s), each as a tick count. So the boss does not attack in stage 0.

The stage value and the fight mode are separate values. The stage rules below read the stage value only in fight mode 0, and the fight mode decides which rules run at all. Fight modes 1 and 2 do not read the stage value.

**Fight mode 0:**

- **Stage 1**: while the HP ratio (current HP / max HP) is below **0.5**, the boss enters the stage-transition state, sets level flag 1, and holds (idle pose, animation 7). When level flag 2 is set, it returns to idle and moves to **stage 2**.
- **Stage 2**: when the HP ratio is 0 or less, the boss enters the death state (animation 7, never ends) and sets level flag 3.

**Fight mode 1:**

- Lava triggers (below).
- Death: when the HP ratio is 0 or less (and the boss is not already dead), the death state is entered and level flag 0 is set.
- The hold state applies (below).

**Fight mode 2:**

- Death at HP ratio 0 or less, which sets no flag.
- No lava triggers.

**Hold (modes 1 and 2):** while level flag 3 is set, the boss is in its hold state (animation 7). When flag 3 clears, the hold ends and the boss returns to idle. The level sets flag 3 when it loads, clears it when the intro cutscene ends, and sets it again when the end cutscene starts.

**Height check (High):** at the start of every stage-rule pass, in every fight mode, the boss compares its own height (the y of its position) with the height of the controlled hero. When the two differ by more than 1.0 (strictly), the boss's lock word is set to 1. The lock word is cleared to 0 at the start of every controller pass, and the charge handler also sets it on every tick it runs (Medium-High for the charge). It is the lock word of the motion record (SPEC-0040, edge guard): while it is 1, the edge guard that reverses a root step over a drop of more than 2.0 does not fire, and the navigation-mesh recovery that puts a character back at its last valid position after a failed triangle locate is skipped (SPEC-0029). So a boss on a different level from the hero can walk over ledges that would normally hold it back. Test vector: boss height 12.0 and hero height 10.5 set the lock word; heights 12.0 and 11.2 do not; heights that differ by exactly 1.0 do not.

**Lava (mode 1 only):**

- **Lava trigger 1**: the first tick the HP ratio is strictly below 0.66, the boss enters the lava state and sets the first lava latch.
- **Lava trigger 2**: the first tick the HP ratio is strictly below 0.33, the same thing would happen with the second latch. The lava state sets the second latch on each of its ticks, including the first. The second check runs only while the first latch is set, so once trigger 1 has entered the lava state, the second latch is already set when trigger 2 is checked, and trigger 2 can never fire. Reproduce this.
- The lava state plays animation 1. While it plays, every tick the boss's own action-frame query is true it sets level flag 2. When animation 1 ends, the boss returns to idle.

The stage-transition state (stage 1, mode 0) is never entered in modes 1 and 2.

### Encounter flow (pack 864; High for state and call order)

Init creates the character slot stores, raises progress flag 287, raises progress flag 60 if it is clear, and sets level state 3. The Chieftain is region set 4; its instance-trigger worker loads that set into store A through main trigger 10. Script trigger 1 separately marks the fight-start stage bit. When the stage thread sees that bit, it starts boss-fight instance 8 at stage 0, plays camera 8 and dialogue line 6, then cameras 10 and 11. Once camera 11 ends, it plays camera 7 and clears level state 3. That clear releases the boss's held state. Progress flag 60 selects normal fight mode 1, with full damage scale and the mode-1 lava and death rules.

At zero HP, the instance-8 boss posts level state 0 in this mode. The stage thread sees that event, marks the fight-end bit and raises flag 293. On its next pass the end stage raises level state 3 to hold the boss, plays camera 10 and resets the camera shake, deactivates entity 44, clears level state 3, plays camera 15, and ends boss-fight instance 8 at stage 1. It also raises flag 668, starts the follow-up camera/object thread, and completes the configured 32-object sequence. The individual camera art, dialogue text and several progress-flag/locator meanings are specified with the level script data, not guessed from the boss controller.

Test path: after trigger 1 marks fight start, instance 8 starts at stage 0; level state 3 clears only after camera 11 finishes. A lethal hit in mode 1 sets level state 0; the next stage-thread pass marks the end, and the following pass runs the end sequence and ends instance 8 at stage 1.
### Melee and missile reactions

**Missile reaction** (runs only when the missile-defense cooldown is 0, and only when a missile contact is found among up to six active projectile slots):

1. Roll the block chance. If the roll is below the missile block chance: enter the block-missile state.
2. Otherwise roll the missile dodge chance. If the roll is below it: enter the dodge-missile state with a dodge direction drawn from a coin flip.
3. Set the missile-defense cooldown to a random tick count from 0.5 to 1.0 s. This is set only when a contact was found.

The missile contact is the same test as the one described under Incoming hits: the projectile's path is tested against the boss's per-bone collision spheres (see there).

**Melee reaction** (runs only when the hero is attacking, the melee-reaction latch is clear, the hero is within 7.0 units, and the hero is in front of the boss):

- "In front" means the hero faces the boss: the dot product of the hero's forward direction and the unit horizontal direction from the hero to the boss is above 0.5.
- Set the melee-reaction latch.
- Roll the melee dodge chance. If the roll is below it: enter the dodge-melee state with a coin-flip direction.
- Otherwise roll the melee block chance. If the roll is below it: enter the block-melee state.
- Otherwise the boss does nothing (the latch stays set until the hero stops attacking).

The melee latch clears as soon as the hero is not attacking.

### Incoming hits (contact)

- In the hold state the boss takes no hits. In the lava state all hits are ignored.
- A **hero melee hit** counts when the boss is not in the get-hit or stage-transition state, the hero is attacking with its attack frames active, and the hero's weapon touches the boss (the hero's single-weapon reach test, SPEC-0029 Behavior 10.2; this is the only use of that test in this boss).
- A **hero projectile** (up to six projectile slots) counts when its path touches one of the boss's per-bone collision spheres, and only spheres with a radius above zero count (Medium-High). The projectile is used up. The path tested for each active projectile slot runs from the projectile's current position to its position plus 120 times its velocity (120 ticks of travel; the 120 is a fixed value, and its meaning as ticks is Medium).
- During dodge-melee, block-melee, forward-swipe and get-hit, a melee hit is ignored when the hero is in front of the boss (the dot product of the boss's forward direction with the direction from the boss to the hero is above 0.5).
- A counted hit (melee or projectile) puts the boss in the get-hit state and reduces its HP by the hero's damage (melee) or by the summed projectile damage.
- A counted hit first plays the shared hit flash at the boss's position: one-shot effect 222, detached (SPEC-0034; the helper's flag is clear, so not 220). High for the id; Medium for the point.

The sphere geometry itself is Medium confidence.

### Attack chooser (runs while free)

Let D be the horizontal distance to the hero. The chooser checks, in order, and the first rule that fires sets the state:

1. **Counter attack**: if the hero is currently in an attacking query and D < 6.0, enter the counter-attack state. (The 6.0 is written in the behavior, not a tuning value.)
2. **Charge**: if the charge cooldown is 0, enter the charge state, set the retreat distance to a uniform real from 10.0 to 20.0, and set the charge sub-step to 0. There is **no distance check** on this rule.
3. **Forward swipe**: if the forward-swipe cooldown is 0 and D < 7.0: roll the forward swipe chance. If the roll is below it, enter the forward-swipe state. Otherwise set the forward-swipe cooldown to a random tick count from the forward-swipe range. (Nothing happens when D >= 7.0.)
4. **Wide swipe**: if the wide-swipe cooldown is 0, D < 7.0, and the hero is in walk-and-turn/strafe (state 32) or a strafe jump (states 43 to 46; SPEC-0033): roll the wide-swipe chance (fight mode 0: 25; fight modes 1 and 2: 50). If below, enter the wide-swipe state. Otherwise set the wide-swipe cooldown.
5. **Buck kick**: if the hero is behind the boss (the dot product of the boss's forward direction with the direction to the hero is below -0.5) and D < 4.0, enter the buck-kick state.
6. **Whirl** (fight mode 1 only): if the whirl cooldown is 0: roll the whirl chance (40). If the roll is 40 or more, set the whirl cooldown to a random tick count from 2.0 to 4.0 s and stop. If the roll is below 40 and D < 15.0, enter the whirl state. Otherwise nothing.
7. Otherwise stay idle.

When the chooser picks an attack, it also clears the hit latch.

### Movement band (runs while free; only changes the state when the boss is in a movement-capable state or idle)

Using D (horizontal distance to the hero), checked in this order. The movement states use the shared locomotion mode numbers given in the States table:

- D > 15.0: rush forward (locomotion mode 1).
- 10.0 < D <= 15.0: strafe close (locomotion mode 0) if D < 13.0. For 13.0 <= D <= 15.0 the current movement is kept.
- D < 5.0: strafe back (locomotion mode 3).
- 5.0 <= D <= 10.0: orbit (locomotion modes 5 and 6), unless the boss is already orbiting, or it is strafing close and D is above 8.0. Entering orbit draws the orbit direction from a coin flip and resets the orbit timer (the 30-tick timer in Per-fight state).

The 8.0 and 13.0 limits are fixed in the behavior, not tuning values. The gaps (8 to 10 while strafing close, 13 to 15) give the band some hysteresis.

**Strafe back-up target.** The back-up (strafe back) target is the boss's position minus 2.0 times the horizontal unit direction from the boss to the hero. The 2.0 is a fixed constant in the behavior, not a tuning word; the distance D is not used for this target. Confidence: Medium-High.

### Orbit

Orbit moves the boss toward a point 3.0 units to one side of its own position. Let d be the horizontal unit direction from the boss to the hero and s = normalize(d x up) = (−d.z, 0, d.x), with up = (0, 1, 0). Direction 0 targets the boss's position + 3.0 x s and uses locomotion mode 5; direction 1 targets the boss's position − 3.0 x s and uses locomotion mode 6. With the handedness of SPEC-0004 (facing +Z, the right side is −X), s is the boss's right-hand side while it faces the hero, so direction 0's target point lies on its right and direction 1's on its left (the steps themselves follow the clip root travel, Locomotion clips, which goes the other way). If the target is not a valid place to stand (the navigation check), the target becomes the boss's own position. The 30-tick timer restarts when it runs out, and at that moment the direction flips (High).

### Locomotion clips

The boss moves only by the root travel of the clip that its current locomotion mode plays (SPEC-0005, Playback; SPEC-0040). Its controller fills the boss's seven locomotion slots with the shared slot setter (SPEC-0029, Behavior 2) at the start of the fight and again before every state handler, with the values (9, 10, 30, 14, 15, 17, 16). In the setter's order these are the slots of modes 0, 1, 2, 3 and 4, then mode 6, then mode 5. Each value is an entry number of the boss's main animation table (SPEC-0005), and the entry names a clip of the boss's model clip bank (40 clips, counted from 0). The Ixi Chief's fight regions (806, 869 and 1275) carry identical entries for these numbers (High).

All of these clips have speed byte 2, so at play rate 1.0 a clip advances a quarter of a key per tick: 4 ticks per key and a cycle of 4 x keys ticks. Every clip below has a constant root record, so travel per tick is the root value x 0.000977 / 4 (High for the values; Medium-High for the travel, which rests on the shared root-travel rule).

| Mode | Used for | Table entry | Clip | Keys (ticks) | Root value per key | Travel |
| --- | --- | --- | --- | --- | --- | --- |
| 0 | Rush forward | 9 | 21 | 14 (56) | forward 387 | 0.0945 per tick, 5.67 units/s forward |
| 1 | Strafe close | 10 | 26 | 7 (28) | forward 1365 | 0.3334 per tick, 20.0 units/s forward |
| 3 | Strafe back | 14 | 29 | 11 (44) | forward -652 | -0.1592 per tick, 9.56 units/s backward, with a small vertical bob (at most 48 per key) |
| 5 | Orbit, direction 0 | 16 | 27 | 11 (44) | sideways +387 | 0.0945 per tick, 5.67 units/s sideways |
| 6 | Orbit, direction 1 | 17 | 28 | 11 (44) | sideways -387 | 0.0945 per tick the other way, 5.67 units/s |

Mode 2 uses entry 30 (clip 9) and mode 4 uses entry 15 (clip 12), but no controller state requests either mode. All five clips loop. The two orbit clips are separate clips, not one clip mirrored. The sign of the sideways value is relative to the boss's own frame; the steering side for modes 5 and 6 is given in SPEC-0029, Behavior 2.

Charge clips (the charge's own slot writes, High). Charge sub-step 1 ("walk toward the target") writes only slot 0 with entry 30 (clip 9: 7 keys, 28 ticks, forward 1365 per key, 20.0 units/s forward) and selects mode 0, so the walk to the charge point runs at 20.0 units/s. Sub-steps 4 and 5 write only slot 0 with entry 22 (clip 15: 15 keys, 60 ticks, forward 1365 per key, 20.0 units/s) and select mode 0. The next pass of the controller loop restores the seven standard values.

### Turning

A turn toward a target direction with blend weight r (per tick) sets the facing to normalize(r x target + (1 - r) x old facing). If the old facing and the target are almost opposite (their dot product below -0.99), 0.1 times the side vector is added before normalizing. r = 1.0 faces the target at once. The weights this boss uses are 0.1, 0.25 and 1.0 (named in the states below). High.

### States

Animation numbers are entries of the boss's table. "Hit" means the boss applies damage once to the hero (see Damage).

| State | Animations | Behavior | Ends |
| --- | --- | --- | --- |
| Idle | 7 | Hovers; runs the choosers | Not applicable |
| Stage-transition (mode 0, HP below 0.5) | 7 | Holds; sets level flag 1 on entry | Level flag 2 sets stage 2 and returns to idle |
| Death | 7 | Holds | Never |
| Hold (modes 1 and 2) | 7 | Holds | Level flag 3 clears |
| Block missile | block pose (5) with the block animation (0x14) | Holds while any missile contact is present | When no missile contact remains: idle |
| Dodge missile | 0x13 (dodge, direction 0) or 0x14 (direction 1) | Plays the dodge chosen by the direction | Animation ends: idle |
| Dodge melee | same as dodge missile | Same | Same |
| Block melee | block pose (5) with 0x14 | Holds while the hero is attacking | When the hero stops attacking: idle |
| Counter attack | 0x19 | Would hit once with counter-attack damage x scale through the weapon test, but its clip has no event bits, so it never hits (see Hit shapes and windows) | Animation ends: idle |
| Charge | sub-steps below | Charges at a target point | See sub-steps |
| Forward swipe | 0x18 | One hit at most | Animation ends: idle; forward-swipe cooldown is set |
| Wide swipe | 0x1A | Turns toward the hero at rate 0.25 per tick while playing; one hit at most | Animation ends: idle; wide-swipe cooldown is set |
| Whirl | 0x1B | One hit if the hero is within the whirl hit range (no guard test); pushes the hero (see Push) | Animation ends: idle; whirl cooldown is set (2 to 4 s) |
| Buck kick | 0x1D | One hit at the first contact of weapon 4 or 5 (see Hit shapes and windows) | Animation ends: idle (no cooldown set) |
| Rush forward | locomotion mode 1 | Moves toward the hero | Until the chooser changes state |
| Strafe close | locomotion mode 0 | Moves toward the hero | Until the chooser changes state |
| Strafe back | locomotion mode 3 | Moves away from the hero | Until the chooser changes state |
| Orbit | locomotion mode 5 (direction 0) or 6 (direction 1) | Circles the hero; the direction flips every 30 ticks | Until the chooser changes state |
| Get hit | 6 | No damage | Animation ends: 25 % (roll below 25 of 0 to 100) to forward swipe, otherwise idle |
| Lava (mode 1) | 1 | Sets level flag 2 while playing (see Lava) | Animation ends: idle |

Animation 0x30 is not a boss animation. It is the hero's knocked state, which the push (see Push) puts the hero into.

### Get hit step (state 6)

Entering the get-hit state plays animation 6. While the state runs, each tick does the following:

1. If the boss's current animation is not 6, play animation 6 and stop there.
2. Otherwise, if the current clip's progress has reached 0.99 (the shared clip-end test of SPEC-0040 Behavior 22B, asked of the boss's current animation, which step 1 has just found to be 6), draw a percent roll from 0 to 100 inclusive (101 values). If the roll is below 25, clear the boss's once-per-attack hit latch and enter the forward-swipe state (state code 7). Otherwise go to idle (state code 0). This is the 25 of 101 chance in the table above.
3. If the clip has not ended, nothing changes.

The cleared word is the shared once-per-attack hit latch of the boss: the forward swipe, the counter attack and the whirl deal their hit only while it is 0 and set it to 1 when they hit, the charge uses it the same way, and the attack chooser clears it whenever it starts an attack. The get-hit branch clears it itself because the forward swipe it starts does not pass through the chooser; without that clear, a swipe started after a landed hit would never hit.

Confidence: High for the branches, the roll range, the clip-end test and the latch. Medium for the state code numbers (the state table above lists names only).

### Charge (sub-steps)

Sub-step 0 is entered when the charge starts.

0. **Find a position**: try three candidate points around the hero, in order: the hero's position plus the retreat distance along the hero's forward direction, then along the hero's side axis, then along the opposite side. The side axis is the first row of the hero's orientation frame, normalize(up x f) = (f.z, 0, −f.x) for the hero's forward direction f; with the handedness of SPEC-0004 this is the hero's left-hand side, so the second candidate is to the hero's left and the third to its right (High). The first candidate that is a valid place to stand (a navigation check) becomes the charge target. If none is valid, set the charge cooldown to a random tick count (1.0 to 3.0 s) and go idle.
1. **Start**: walk toward the target, playing table entry 30 (clip 9, 20.0 units/s; see Locomotion clips). Move on to sub-step 2 when the boss is within 1.0 horizontal units of the target.
2. **Turn and face**: play animation 7 if it is not already playing. Turn toward the hero at rate 0.1 per tick. When the animation ends: sub-step 3.
3. **Pre-twirl**: play animation 0x15. Turn toward the hero at rate 1.0 per tick. When the animation ends: sub-step 4.
4. **Charge to**: the target is the hero's current position, re-read every tick. If that point is not valid for navigation, set the charge cooldown and go idle. Otherwise move toward it, playing animation 0x16. When the horizontal distance to the hero is below 6.0, roll the charge-slam chance (40): if the roll is below 40, go to sub-step 6 (stop-and-slam). Otherwise go to sub-step 5 (run past) and clear the hit latch.
5. **Run past**: the move target is the boss's position plus 1.0 times its forward direction. Leave (idle, and set the charge cooldown) when the hero is more than 7.5 units away, or when the boss is blocked. While the hero is within 7.5 units and the boss is not blocked, a body contact pushes the hero (see Push) with total speed K = 5.0, which is 3.0 horizontally away from the boss plus 4.0 upward, unless the hero is already in its knocked state.
6. **Stop and slam**: play animation 0x17. One hit at most, applied while the animation plays when the hit test passes (stop-and-slam damage x scale). When the animation ends, the boss always goes idle and the charge cooldown is drawn (whether or not it hit).

The charge cooldown is set at the end of the stop-and-slam (hit or miss), on the run-past exit, and on navigation failure in sub-steps 0 and 4.

**Navigation for the Ixi (R-0088).** Its charge walk and movement states call the shared navigation step of SPEC-0056. Sub-step 1 sets movement mode 0 and follows the stored charge point; sub-step 4 sets mode 0 and follows the hero's current position after checking that point on the assigned mesh. Rush-forward uses mode 1, strafe-close mode 0, and strafe-back mode 3. The shared step follows the mesh route table toward a clamped edge crossing while the goal is in another triangle, and steers toward the goal once both are in the same triangle. Its steering weight depends on that step's branch (0.1 for route crossings; 0.3 for the settled same-triangle approach, with the other 0.1 cases described in SPEC-0056). Modes 0 and 1 steer along the chosen direction; mode 3 steers against it (SPEC-0029, Behavior 2).

The charge sub-step 1 advances only when the horizontal distance to its stored target is strictly below 1.0. It does not check the navigation blocked flag, navigation arrival flag or a timer. A fresh-block arrival in the shared navigation step can mark navigation complete and turn the character toward the hero, but does not change the Ixi's charge sub-step. If blocked farther than 1.0 from the target, the charge can remain in sub-step 1 indefinitely unless another event changes the Ixi state. At 3.3 horizontal units from the target, the exit gate fails.

Charge candidates are accepted when their x-z projection lies inside a triangle of the Ixi's assigned mesh. The containment test does not compare candidate height with the triangle plane, so a point can pass below a raised surface at the same horizontal coordinates. At controller start, the Ixi keeps its assigned mesh if it has one; otherwise it searches the world's kind-1 ground meshes in list order and assigns the first that contains the boss. Fight mode 0 rechecks its mesh each tick; modes 1 and 2 do not.

### Push

A push sets the hero's velocity to K x normalize(9 x d + (0, 12, 0)), where d is the horizontal unit direction from the boss to the hero. K is the total speed (the length of the velocity), not a horizontal speed. Equivalently the velocity is 0.6 K along d and 0.8 K upward, so K = 5.0 gives 3.0 horizontally away from the boss and 4.0 up. In order: the hero's motion-record velocity is set to zero, 12.0 is added to its vertical (y) lane only, 9 x d is added (d has no vertical part), the result is normalized and scaled by K, and the velocity is zeroed again and set to that value (High for the lanes). The hero is then put into its knocked state (hero action state 0x30). Uses:

- Whirl: K = 5.0 (the whirl push speed tuning value), applied when the whirl hits.
- Charge run past: K = 5.0 (a fixed value in the behavior).
- Counter-attack and buck-kick knockback values are not used (no push in those handlers).

### Hit shapes and windows (Confidence: High)

The boss's melee shapes are the monster weapon cylinders of its monster row (row 155; SPEC-0029 Behavior 10.2): weapons 1 to 5 each radius 5.0, length 3.0. Weapon N is tested only while event bit 2^(N-1) is on in the current frame of the attack clip, against each hero bone capsule (static test). The fight regions 806, 869 and 1275 carry the same clips, so the windows below hold for all three. Each attack hits at most once (the hit latch).

| Attack (animation) | Shape | Window (frames with the event bit on) | Damage before scale | Block |
| --- | --- | --- | --- | --- |
| Counter attack (0x19) | weapon test | none: the clip carries no event bits, so it never hits | 25 | not applicable |
| Stop and slam (0x17) | weapon 1 | bit 0x1, frame 6 of 13 | 100 | no |
| Forward swipe (0x18) | weapon 1 | bit 0x1, frames 6 to 11 of 22 | 50 | yes (see Damage) |
| Wide swipe (0x1A) | weapon 1 | bit 0x1, frames 6 to 11 (the same window as the forward swipe) | 35 | no |
| Whirl (0x1B, looping) | horizontal distance below 12.0 (whirl hit range) | bit 0x1 is on in every frame | 100, once; push K = 5.0 | no |
| Buck kick (0x1D) | weapons 4 and 5 | bits 0x8 and 0x10, frames 12 to 15 of 20 | 25 | no |

### Damage

- Damage applied to the hero is the integer tuned value times the damage scale, truncated toward zero: damage = trunc(tuned x scale). It goes straight to the hero's take-hit and is subtracted from HP unchanged: the hero's shield, armour, armour Mote and petpet factor do not reduce it (SPEC-0029 Behavior 10.5).
- **Damage scale rule**: the scale is 0.25 in fight mode 0 and 1.0 in fight modes 1 and 2 (see Fight mode).
- Tuned values before scale: counter-attack 25, forward swipe 50, wide swipe 35, whirl 100, buck kick 25, stop-and-slam 100.
- Incoming hits remove the hero's damage from the boss's HP, computed by the hero's damage query with multiplier 1.0 (see Incoming hits). The boss's damage scale applies only to the attacks above.
- A forward swipe that lands on a hero who is guarding and faces the boss (the hero's forward direction and the direction from the hero to the boss have a dot product above 0.5) deals 0 damage and is flagged as blocked: the hero's take-hit runs with 0 and the blocked flag, so the hero plays its block reaction. Only the forward swipe has this block; every other attack is unblockable.

### Sounds and effects (Confidence: High)

**Sound rule.** The boss plays its voices and attack sounds only through its main animation table (SPEC-0005, SPEC-0031); its controller plays no cue. Each sound entry names a cue of a creature block (sound group 1): block 392 (the Ixi Chieftain) or block 373 (the Ixi warrior block). When the boss's main animation id changes to k, the cue of entry k plays once, at the boss's position on that block's slot, on the first tick of the animation: every sound entry has start frame 0 and no keep-alive bit. The cue plays only while the block's slot is loaded, at the default volume and with no do-not-restart (SPEC-0031). Block 373 is loaded only when a registered table names it; both chieftain tables do. A request of the animation that is already current starts nothing; a new start after a different animation plays the cue again. The animation-sound step runs every tick except while the character's base state is 42, where it is silent.

**Animation to cue, regions 806 and 869** (the two tables are identical; block 392 unless marked; animation -> cue, with the cue's subject):

- 0, 1 and 2 -> 10 (death); 3 -> 14 (triumph); 4 -> 0 (war cry).
- 5 (block pose), 19 (0x13, dodge) and 20 (0x14, dodge and block) -> cue 8 of block 373.
- 6 (get hit) -> 8; 11 -> 7 (block); 12 -> 9 (defend flinch).
- 21 (0x15, pre-twirl) -> 1 (raise and twirl); 22 (0x16, charge to) -> 2 (twirl and charge); 23 (0x17, stop and slam) and 30 -> 12 (charge).
- 24 (0x18, forward swipe) -> 4 (swing); 25 (0x19, counter attack) -> 3 (stab down); 26 (0x1A, wide swipe) -> 5 (wide swipe); 27 (0x1B, whirl) -> 6 (whirl); 28 -> 11 (attack stab).
- 29 (0x1D, buck kick) -> cue 6 of block 373 (kick).

Silent: 7 (idle, hold, stage-transition and death), 9, 10, and 14 to 18. Cue 13 of block 392 (counter attack) is named by no animation, so it never plays; the counter-attack state sounds cue 3 instead.

Consequences in play: the lava state's animation 1 plays cue 10 (the death cue); the death state plays the silent animation 7, so the boss dies without a death sound.

**Animation to cue, region 1275** (a different set): 0 -> cue 0 of block 392; 4 -> cue 2 of block 373; 5, 11, 19 and 20 -> cue 8 of block 373; 6 -> cue 10 of block 373; 26 -> cue 15 of block 373; 28 -> cue 5 of block 373; 29 -> cue 6 of block 373. Every other animation of this table is silent.

**Effects.** Each counted incoming hit plays the shared hit flash, one-shot effect 222, at the boss (see Incoming hits; Medium for the point).

**Clip effect lists** (animation-driven effects, SPEC-0034; Medium-High, because the list index follows the order of the region's effect records): 0, 1 and 23: effect 1411 (brown dust sprites) at 0.07 to 0.17 and 0.55 to 0.65 of the clip, plus effect 415 (4 white models, once) at 0.55 to 0.65, on attach slot 7. 2: effect 1201 at 0.9 on slots 7 and 4. 3, 4, 6, 9, 12, 14, 16, 17, 18, 28 and 29: effect 1410 (dust) for the whole clip. 10, 15, 19, 20 and 30: effect 1411. 21, 24, 25 and 26: effect 528 twice on slot 0 (red streaks, a weapon trail). 22 and 27: effect 528 on slot 0 (second half) and slot 1 (first half), plus 1411.

### Pair compatibility (two type codes)

A predicate takes a first code a and a second code b and decides whether the pair is compatible.

1. If a is not in the range 1 to 10 and not in the range 15 to 19, the pair is compatible. This covers a of zero or less, 11 to 14, and 20 and above.
2. If a is 6, the pair is compatible only when b is 0, 2, 3, 4 or 6.
3. If a is 5, or a is in 7 to 10, the pair is compatible for any b.
4. Otherwise (a in 1 to 4, or 15 to 19), the pair is not compatible.

Examples: (6, 2) compatible; (6, 1) not; (5, 99) compatible; (3, 0) not; (12, 0) compatible.

A caller runs its first two sub-steps only when the pair is compatible, and always runs its third sub-step (Medium). The meaning of the two codes is not established (Open questions).

Confidence: High for the rule as written. Medium for the caller's use.

## Edge cases and error handling

- **Probability quirk**: every percent roll is inclusive of 100. A 0 % chance can never fire, and a 100 % chance fires 100 out of 101 times.
- **Lava quirk**: after lava trigger 1, lava trigger 2 can never fire. The mechanism: the lava state sets the second latch on each of its ticks, including the first, and the second check runs only while the first latch is set, so it always finds the second latch already set. Reproduce.
- **Charge without a distance test**: the charge can begin at any distance once its cooldown is 0, including far away. Keep this.
- **Forward swipe has no cooldown when out of range**: no roll and no cooldown happen when D >= 7.0. The cooldown is only set after a failed roll inside range.
- **Missile cooldown**: the missile-defense cooldown is set when a missile contact is found, whether or not a block or dodge is chosen.
- **Charge cooldown on success**: the charge cooldown is also drawn after a successful stop-and-slam, at the end of the slam animation.
- **Wide swipe hero state**: the wide swipe needs the hero's current action state to be one of 0x20, 0x2B, 0x2C, 0x2D or 0x2E. Other states never get a wide swipe.
- **Height check**: the stage rule's height comparison (boss against controlled hero) sets the boss's lock word for the tick when the heights differ by more than 1.0 (High; see Stages and modes).
- **Unused tuning values** (registered with the tuner, not used by the behavior): missile-dodge chance 2 (0x2DAEA8, 50), melee-block chance 2 (0x2DAEB0, 50), counter-attack knockback (0x2DAEB8, 3.0), charge run-past chance 2 (0x2DAED4, 60), stop-and-slam kick chances (0x2DAEDC 25, 0x2DAEE0 50), buck-kick knockback (0x2DAF10, 3.0), angry-stance percentage (0x2DAF34, 50), orbit range (0x2DAF44, 10.0). The orbit uses fixed thresholds in the behavior instead. No load or store to any of these words exists in the boot program, so they are unused in the original (High). Do not apply these values.
- **Duplicate names**: two tuning entries share names with different data. Use the addresses above, not the names.
- **Lava events**: the latches fire once per fight. A boss that was healed above the trigger keeps its latch set.
- **Stage 0**: the boss does not attack before level flag 0 (held in the hold state).
- **Counter attack never hits** (High; reproduce): the counter-attack clip carries no event bits, so its weapon test never runs and the counter deals no damage, although its damage word (25) is read.
- **Boss melee ignores defences**: the boss's hits are not reduced by the hero's armour or shield.
- **Unused counter-attack cue** (High; reproduce): cue 13 of block 392 is named by no animation, so it never plays. The counter attack (animation 25) plays cue 3.
- **Death cue on lava, silent death** (High; reproduce): the lava animation 1 carries cue 10 (the death cue), and the death state's animation 7 is silent.

## Test vectors

Synthetic checks on the rules, not disc data.

- Fight start: flag 0x166 set gives mode 2 (even if 0x3C is set). Flag 0x166 clear and 0x3C set gives mode 1. Neither gives mode 0 (damage scale 0.25).
- Mode 0, stage 1, HP ratio 0.50: no transition (the test is strict below 0.5). HP ratio 0.49: transition, flag 1 set.
- Mode 1, HP ratio 0.66: no lava (strictly below 0.66 is needed). Ratio 0.65: lava 1; the first latch and the second latch are both set. Ratio 0.33 or 0.32 afterwards: no lava 2.
- Mode 2, HP ratio 0.10: no lava.
- Game state 358 set (the final fight's script pack has run its init) when the controller starts: mode 2. Game state 358 clear, 0x3C set: mode 1.
- Missile roll 24 with block chance 25: block. Roll 25: not a block; dodge roll 24 with dodge chance 25: dodge.
- Missile contact: a projectile whose path (from its position to its position plus 120 times its velocity) meets a sphere of radius 0 does not count; one that meets a sphere of radius above 0 does (the reaction then rolls as above). Velocity (0, 0, 0.1) from the origin gives a path ending at z = 12. A sphere of radius 0.5 centred at z = 11 is hit; a sphere of radius 0.5 centred at z = 13 is not.
- Melee: hero attacking, distance 6.9, dot 0.6 with the latch clear: a roll of 0 for melee dodge (below 25) gives dodge. A roll of 30 gives no dodge; a block roll of 10 gives block.
- Counter attack at scale 0.25: trunc(25 x 0.25) = 6. Scale 1.0: 25.
- Forward swipe at scale 0.25: trunc(50 x 0.25) = 12. Wide swipe at 0.25: trunc(35 x 0.25) = 8. Whirl at 0.25: 25. Stop-and-slam at 0.25: 25. Buck kick at 0.25: 6.
- Cooldown ticks: charge 1.0 to 3.0 s gives an inclusive range of 60 to 180 ticks. Whirl 2.0 to 4.0 s gives 120 to 240. Missile defense 0.5 to 1.0 s gives 30 to 60.
- Movement: D = 16 -> rush forward (locomotion mode 1). D = 12 -> strafe close (mode 0). D = 4 -> strafe back (mode 3). D = 14 -> no change. D = 9 with the boss idle -> orbit (mode 5 or 6, by the drawn direction).
- Locomotion clips: the slot values are (9, 10, 30, 14, 15, 17, 16), so mode 5 plays table entry 16 (clip 27) and mode 6 plays entry 17 (clip 28). Mode 1 over one 28-tick cycle moves 28 x 0.3334 = 9.34 units forward; mode 0 over its 56-tick cycle moves 5.29 units. Charge sub-step 1 over 28 ticks moves 9.34 units.
- Orbit timer: a boss in direction 0 is still in direction 0 after 29 ticks, and direction 1 from tick 30 on (the timer restarts at 30).
- Charge sub-step 4 with the horizontal distance at 5.9: the roll of 39 (below 40) goes to stop-and-slam; a roll of 40 goes to run past.
- Run past: hero at 7.6 horizontal units -> the run past ends. At 7.5 it does not.
- Push: K = 5.0 with a hero directly in front (d = (1, 0, 0)): velocity = (3.0, 4.0, 0). Check: 0.6 x 5 = 3, 0.8 x 5 = 4. The run past uses the same K = 5.0, so its push gives the same velocity. With d = (0, 0, -1) the velocity is (0, 4.0, -3.0); its length is 5.0 in every case.
- Orbit side: boss at (0, 0, 0), hero at (0, 0, 10), so d = (0, 0, 1) and s = (−1, 0, 0). Direction 0 targets (−3, 0, 0); direction 1 targets (3, 0, 0).
- Charge candidates: hero at (0, 0, 0) facing f = (0, 0, 1), retreat distance R: first (0, 0, R), then (R, 0, 0), then (−R, 0, 0).
- Wide swipe: hero action state 0x2B with D = 6: eligible. Hero action state 0x10 with the same D: not eligible.
- Counter attack (disc clip): the hero stays inside weapon 1 for the whole counter-attack clip: no damage.
- Whirl: the hero 11.9 units away: 100 x scale once (25 in mode 0, 100 in mode 1). 12.0 units away: nothing.
- Forward swipe block: the hero guards with dot(hero facing, hero-to-boss) = 0.6: take-hit 0 with the blocked flag, HP unchanged. Dot 0.4: trunc(50 x scale).
- Get hit to forward swipe: the boss's last whirl hit set the hit latch to 1; it is then hit and plays animation 6. At progress 0.99 the roll is 24: the latch becomes 0 and the forward swipe starts, so its hit can land. Roll 25: idle, latch unchanged (the next attack start clears it). At progress 0.98 no roll is made.

## Open questions

- Resolved (R-0067, side of root x): in modes 5 and 6 the shared steering turns the boss's move direction toward the hero (SPEC-0029, Behavior 2: with d the direction to the side target, mode 5 steers to up x d and mode 6 to d x up, which is the direction to the hero), and the root step of a clip is applied along the record's side axis, up x forward (SPEC-0040, Behavior 14C), a positive root x going along it. The table entries of these clips carry no mirror flag. With the boss facing +Z that axis is +X, which is the boss's left (SPEC-0004: the right side is -X). So the mode 5 clip, whose sideways root is positive, moves the boss to its left of the line to the hero, and the mode 6 clip, whose sideways root is negative, to its right. This is the opposite of the side point that the target names; that point only feeds the steering (High for the rule; Medium-High for what shows on screen). Test vector: boss at (0, 0, 0) and hero at (0, 0, 10): mode 5 moves toward +X and mode 6 toward -X, while the mode 5 target point is (-3, 0, 0) and the mode 6 target point is (3, 0, 0). Ixi Chief: direction 0 (mode 5, +387 per key, 0.0945 per tick) circles to its left, direction 1 (mode 6, -387) to its right; the orbit timer is as given.

- **Resolved: mode-2 whirl.** The attack chooser gates whirl selection on fight mode exactly 1. Mode 2 can use its 50 % wide-swipe chance, but cannot select the whirl. The wide-swipe row covers modes 1 and 2 (High).
- **Mode 2 start**: whether any Ixi Chief starts its controller in the final-fight world, so whether mode 2 is reached in play (Medium). The clean side must not guess. Answered: no known level script runs the fight in mode 0, and mode 2's flag 0x166 (game state 358) is set only by the init of script pack 1432, the final fight (High).
- **Ixi action-frame and attack queries**: Resolved: the boss-side query is the AI-character action-frame query and the hero-side query is the hero's attack-active word (see Cross-boss facts; High).
- **Hit test of the counter attack and the wide swipe**: Resolved: both use the monster weapon test, and the counter attack never hits (see Hit shapes and windows; High).
- **Shared volume test**: Resolved: the buck kick and the other melee attacks use the monster weapon test of SPEC-0029 Behavior 10.2 (row 155) (High).
- **Weapon attachment bones**: Resolved (High for the table entries; Medium for the body-part names). The five weapon frames are attach points 0 to 4 of the boss model's attach-point table (SPEC-0004, SPEC-0005); the table is identical in the model of regions 806 and 869 and in the Chieftain model of region 1275. Frame 0 follows bone 9 with offset (0, 0.055, -0.116) and a quarter turn about y. Bone 9 is the end of a two-bone chain that grows straight forward from the front of the torso, with a model-space rest position of (0, 3.24, 2.69), the most forward point of the model. Frame 1 follows bone 17, the end of the single arm on the +x side (rest position (2.25, 3.41, 0.39), offset (0.126, -0.108, 0)). Frame 2 follows the root bone with a zero offset. Frames 3 and 4 follow bones 51 and 47, the two rear hooves (rest positions (-0.36, 0.67, -0.76) and (0.36, 0.67, -0.76), zero offset, a half turn about x). So the stop-and-slam, forward swipe and wide swipe volumes (weapon 1) sit at the forward bone 9, the buck kick volumes (weapons 4 and 5) sit at the two rear hooves, and weapons 2 and 3 are used by no shipped attack window. The clean side reads the bone numbers and offsets from the model on the disc. Test vector: with the boss unrotated, standing at the origin in its rest pose, frame 0's origin is (0, 3.294, 2.576) and frame 3's origin is (-0.363, 0.666, -0.760).
- **Resolved: wide-swipe hero action states.** State 0x20 (32) is walk-and-turn/strafe, enabled by holding L2; states 0x2B to 0x2E (43 to 46) are strafe jumps entered with Cross while in strafe mode and the stick pushed. State 43 is the jump right, 44 the jump left, and 45/46 use the two other launch clips in the same family (SPEC-0033, Behaviors 4 and 50; High). The wide-swipe selector conditions are unchanged.
- **Knockback lanes**: Resolved: the 12.0 goes to the world vertical (y) lane of the hero's velocity and 9 x d to the horizontal lanes (see Push; High). What the hero's knocked state then does is the hero's rule (SPEC-0036).
- **Run-past stop and body contact**: Resolved (High for the two flags and their writers; Medium-High for which contacts raise the body flag). Both tests belong to the boss, not the hero. The stop test is the boss's navmesh blocked flag (SPEC-0056): it is set on a tick in which a navmesh boundary edge pushed the boss back or the boss was restored onto the mesh, and it stays set for the next 120 ticks, so one wall touch ends the run past on the next step. The body-contact test is the touched flag of the boss's motion record (SPEC-0040): it is cleared at the start of each motion step and set when a contact resolved against a surface in that step. The push in sub-step 5 therefore needs all of: blocked flag clear, hero within 7.5 horizontal units, touched flag set, and the hero not in its knocked state (state 48). If the touched flag is clear the step does nothing and the run past continues. Test vector: sub-step 5, hero 7.0 units away, blocked flag clear, touched flag clear: no push, still sub-step 5. Touched flag set and the hero standing: the hero's velocity becomes 3.0 along the horizontal direction away from the boss plus 4.0 up, and it enters state 48. The same with the hero already in state 48: no push. Blocked flag set, any other value: the run past ends (idle, charge cooldown set) and no push happens.
- **Side (right) direction**: Resolved: the charge's candidates use the hero's frame side axis (up x forward, the hero's left), and the orbit uses normalize(d x up) from the boss toward the hero (the boss's right) for direction 0 (see Orbit and Charge; High).
- **Charge arrival and run-past thresholds** (1.0, 6.0 and 7.5 units): Resolved (High). All three are horizontal distances: the vertical component of the offset is zeroed before the length is taken. The 1.0 test of sub-step 1 measures the boss's position against the stored charge target; the 6.0 test of sub-step 4 measures the boss against the hero's position of that tick; the 7.5 test of sub-step 5 measures the boss against the hero. Heights never enter. Test vector: in sub-step 5 a hero standing 30.0 units higher than the boss at a horizontal offset of (5.0, 5.0), a horizontal distance of 7.07, is within 7.5, so the run past continues; at a horizontal offset of (6.0, 5.0), 7.81, it ends.
- **Random generator clock source**: Resolved: the first-use seed comes from the console's real-time clock, so the sequence differs from boot to boot (High).
- **Intro and end cutscenes.** Resolved: pack 864 script trigger 1 starts boss-fight instance 8 at stage 0 and the camera/dialogue sequence; clearing level state 3 releases the boss. In normal mode, its death event sets level state 0; the stage thread then raises flag 293 and, on the following pass, holds and deactivates the boss, plays the closing cameras, and ends instance 8 at stage 1. See Encounter flow (High for the level-state sequence; camera art and some flag labels remain unspecified).
- **Movement-blocking flag**: Resolved (High). It is the motion record's lock word of SPEC-0040, set when the boss's height and the controlled hero's height differ by more than 1.0 (see Stages and modes, Height check). While set it switches off the edge guard and the navigation-mesh position restore.
- **Hero path length 120**: Answered: a fixed value; whether its unit is one tick of travel is Medium (the spit's velocities are per tick, SPEC-0015).
- Resolved: **Get-hit roll range.** This boss uses the inclusive 0 to 100 range, so a threshold of 25 passes for 25 of 101 possible draws. The 0 to 99 draws in SPEC-0021 are Index(100) drop and item bands; they are a separate helper and do not conflict (High).
- **Get-hit clip-end test**: Resolved: it is the shared test "current clip progress at least 0.99" (SPEC-0040 Behavior 22B); the animation id is checked separately just before it (see Get hit step; High).
- **Get-hit counter**: Resolved: the word reset on the forward-swipe branch is the boss's once-per-attack hit latch, so the forward swipe that follows a get-hit can hit (see Get hit step; High).
- **Sound step gates and the flash point**: the boss is taken to be a world character like every character spawned from a level's character records, so the world-character sound rule applies (Medium-High). Two further gates of the animation-sound step, a motion-block gate and a per-character mute word, are not named. The exact point of the incoming-hit flash is Medium.
- **Pair codes (Low-Medium)**: what the two codes of the pair predicate are (kinds, states or group members), which caller routine uses the predicate, and what the three sub-steps do. The first-code range test takes its value through a helper whose input is not known, so the first code being the tested value is Medium.
- Resolved (R-0088): the Ixi uses the shared route-table navigation step and the mode-specific steering in SPEC-0029 and SPEC-0056. Its charge sub-step 1 exits only below 1.0 horizontal distance from its stored target; a navigation arrival or fresh block does not advance the boss sub-step. Charge-point validity uses x-z triangle containment and ignores height. The pack-800 stuck case can persist without an external state change. High for the static rules; Medium-High for the stated pack scenario.


