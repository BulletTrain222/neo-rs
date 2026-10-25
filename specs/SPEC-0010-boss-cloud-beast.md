# SPEC-0010: Boss behavior: Cloud Beast

- Answers: proactive, none requested (boss AI)
- Status: Draft
- Disc build: SCUS-97367 v1.00 (same fingerprint as SPEC-0001)
- Confidence: per section. Placement and reachability (the fight is not placed on the disc): High. Tuning values, stage rule, hesitation and shock: High (the values are words of the boot program's data). Attack choice procedure (the band rule with cooldowns): High for the procedure, Medium that the original design intends the overlap. Air-flow and rapid air-flow sub-steps: High for order and exits, High for the sub-step 2 animation speed. Rapid count (k + 1 gust cycles, check before decrement): High. Gust next-point update: High. Gust connect test (once per decision): High for the once-per-decision flag, Medium-High for the test timing. Gust connect shape (hero body cylinder against the gust cylinder through the shared finite-cylinder overlap of SPEC-0018): High for both shapes, their fields and the general-rule path; High for the zero-length case. Gust origin, end point, travel and knock-back: High. Cooldown draw: High for the shared real-valued draw and for truncation. Random draws and their end values: High. Lightning: High for the sub-state order, the countdown latch, the wind-up table, the order of the bolt tests, the aim lift lane, the guard aim shift, the beam cylinder and the meaning of the 500.0 value (beam growth per update). Second beam (the reflected beam): High for both triggers, their gates and the geometry. Recorded hero position: High (overwritten before use; no other reader). Special reply (state 7): High for the mechanism (guard, facing, Fog Mote 166 in the shield word, reflected beam, 20 damage through the shared HP subtraction, roll); High for the shared NPC hit route, with original hurt-capsule presence unknown. Companion (row 2, class 10): High for its attachment, facing target, and roles as the lightning source and reaction-sphere centre. Arena movement: High (position snapped to the current point, facing steered). Bouncing balls: High for the no-restitution rule, the spawn blend, the bounce test and the travel measure; High that requested effect definitions 447–451 and 454 are absent from the disc effect library. Shock effect handle 454: High for its purpose and lifecycle; its definition is absent. Distance gates (shock, reaction, aim): horizontal, High. Reaction trigger: High (hero variant, Tor's in-attack test, Roberta's shot segments and attack-interruption rules). Unused words 0x2DA58C, 0x2DA5F0 and 0x2DA614: Medium-High. Boss-fight instance 4: Low. Animation numbers: High except where marked.

## Summary

The Cloud Beast is a boss that hovers between six arena points and alternates between idling and one of six attacks: a close-range shock, a combat fidget, a single air-flow gust, a rapid series of gusts, a lightning barrage and bouncing bolt balls. Its HP decides one of three stages, and each stage changes the attack weights, cooldowns and speeds. When a band's attack is cooling down, its share of the roll goes to the lightning band instead of being dropped. A second body, the companion (monster row 2, class 10), rides on the boss: the lightning comes from the companion, and the reaction trigger watches it. Ordinary hero weapon hits use the shared NPC character hit route when the target passes the normal hit-scan gates; the route can reduce the Cloud Beast's HP, though its missing rig leaves the needed hurt capsules unknown. The controller also damages itself after a reflected lightning beam: a guarding hero with the Fog Mote (item 166) in the shield word, facing the boss from more than about 14 units, sends the beam back, and the boss plays a special-reply state that costs it 20 HP. All timings run on the 60 Hz game tick (SPEC-0007). Times given in seconds become ticks = trunc(seconds x 60). The fight is not placed anywhere on the disc and does not occur in normal play (see Placement and reachability); this spec describes the behaviour as it exists in the boot program, for a reimplementation that may still include it.

## Where it lives on the disc

- The boss's tuning values are 32-bit words in the boot program's data, read at runtime by virtual address (SPEC-0001 gives the fingerprint). They occupy 0x2DA4D0 to 0x2DA617. Each value below gives its address. "int" is a signed 32-bit integer and "float" an IEEE single.
- The six arena points are the level locators with user ids 101 to 106 (the 16-bit id field of the level's locator records, SPEC-0006). If any is missing, the boss cannot move between points. No level on the disc is a Cloud Beast arena (see Placement and reachability).
- The boss's monster row is row 1 of the monster table (monster type 1, variant 0) and its companion's is row 2 (type 2); see SPEC-0029 Layout section 4.
- "Animation n" means entry *n* of the boss's main animation table (SPEC-0005). Every animation request is sent to the companion too when it exists.
- Three effect objects of the boot program's run-time data serve the boss: the gust stream at 0x36B4D0, lightning beam 1 at 0x36B560 and beam 2 (the bounced or reflected beam) at 0x36B5C0. A beam object keeps its drawn length, a target length and a growth limit; see Lightning.

## Layout

Tuning values (stage 1 / 2 / 3 where three are listed):

| Parameter | Type | Address(es) | Values |
| --- | --- | --- | --- |
| Fidget weight (`combatFidgetChance1`–`3`) | int % | 0x2DA4E8, +4, +8 | 20 / 10 / 0 |
| Air-flow weight (`airFlowAttackChance1`–`3`) | int % | 0x2DA4F4, +4, +8 | 40 / 30 / 25 |
| Rapid air-flow weight (`rapidAirFlowAttackChance1`–`3`) | int % | 0x2DA500, +4, +8 | 0 / 5 / 15 |
| Lightning weight (`lightningAttackChance1`–`3`) | int % | 0x2DA50C, +4, +8 | 40 / 35 / 25 |
| Bouncing-bolt weight (`bouncingBoltAttackChance1`–`3`) | int % | 0x2DA518, +4, +8 | 0 / 20 / 35 |
| Hesitation, stage 1 (min, max) (`decisionHesitationMin1`, `decisionHesitationMax1`) | float s | 0x2DA4D0, 0x2DA4D4 | 1.0, 2.0 |
| Hesitation, stages 2 and 3 (min, max) (`decisionHesitationMin2`, `decisionHesitationMax2`) | float s | 0x2DA4D8, 0x2DA4DC | 0.5, 1.0 |
| Air-flow cooldown (min, max per stage) (`airFlowTimerMin1`–`3`, `airFlowTimerMax1`–`3`) | float s | 0x2DA53C … 0x2DA550 | 5–10 / 2–5 / 1–2 |
| Lightning cooldown (min, max per stage) (`lightningTimerMin1`–`3`, `lightningTimerMax1`, `lightningTimerMax2`; 0x2DA538 is registered as `lightningTimerMin3` in error, see Developer names below) | float s | 0x2DA524 … 0x2DA538 | 5–10 / 2–5 / 1–2 |
| Bouncing cooldown (min, max per stage) (`bouncingBoltTimerMin1`–`3`, `bouncingBoltTimerMax1`–`3`) | float s | 0x2DA554 … 0x2DA568 | 5–10 / 4–8 / 2–5 |
| Lightning fire time (`fireBoltTime1`–`3`) | float s | 0x2DA5A8, +4, +8 | 3 / 2 / 1 |
| Lightning wind-up, stages 1 and 2 (`lightingBoltWindupTime2`, `lightingBoltWindupTime3`; see Developer names below) | float s | 0x2DA5A0, 0x2DA5A4 | 1.0, 0.5 (stage 3: see Lightning) |
| Gust animation speed (`airFlowAnimSpeedMultStage1`–`3`) | float × | 0x2DA580, +4, +8 | 1.25 / 2.0 / 3.0 |
| Balls per cast (`numBallsCreatedStage1`–`3`) | int | 0x2DA5C4, +4, +8 | 2 / 3 / 4 |
| Ball speed (`ballSpeedStage1`–`3`) | float units/s | 0x2DA5D0, +4, +8 | 10 / 10 / 10 |
| Shock trigger range (`shockAttackRange`) | float units | 0x2DA56C | 15 |
| Shock radius (`shockAttackRadius`) | float units | 0x2DA5B8 | 8 |
| Shock damage (`shockAttackDamage`) | int | 0x2DA5BC | 10 |
| Gust damage (`airFlowDamage`) | int | 0x2DA598 | 20 |
| Lightning bolt damage (`fireBoltDamage`) | float | 0x2DA5B4 | 10 |
| Ball damage (`bouncingAttackDamage`) | float | 0x2DA5DC | 50 |
| Ball range (`bouncingBoltRange`) | float units | 0x2DA5C0 | 50 |
| Rapid gust count (min, max) (`minNumRapidAirFlowAttacks`, `maxNumRapidAirFlowAttacks`) | int | 0x2DA590, 0x2DA594 | 3, 5 |
| Gust travel per tick | float units | 0x2DA5E8 | 0.5 |
| Gust hit-volume radius (radius of the gust cylinder in the gust connect test, see Gust connect shape test) | float units | 0x2DA5EC | 6.0 |
| Gust knock-back speed | float units | 0x2DA5E0 | 7.0 |
| Lightning guard aim shift (along the hero-to-boss direction) | float units | 0x2DA5FC | 1.65 |
| Beam growth limit (largest length added per beam update; used for both beams) | float units | 0x2DA5F4 | 500.0 |
| Lightning hit latch period | float s | 0x2DA5F8 | 1.0 |
| Companion pose point index (lightning source, reaction sphere centre, boss look-at point) | int | 0x2DA5E4 | 1 |
| Special-reply damage to the boss | int | 0x2DA608 | 20 |
| Ball touch radius (sphere radius) | float units | 0x2DA610 | 2.0 |
| Ball starting and bounce vertical speed | float units/tick | 0x2DA600 | 1.0 |
| Ball gravity per tick | float units/tick | 0x2DA60C | −0.1 |
| Reaction hold | float s | 0x2DA604 | 1.5 |

Words the behavior does not use:

| Word | Type | Address | Value | Status |
| --- | --- | --- | --- | --- |
| decisionHesitationMin3 (stage-3 minimum by name) | float s | 0x2DA4E0 | 0.25 | Stored; no reader is known. Stage 3 uses the stage-2 hesitation pair (see Edge cases), not this word. |
| decisionHesitationMax4 (the name has no fourth stage) | float s | 0x2DA4E4 | 0.5 | Stored; no reader is known. |
| stunTimeMin | float s | 0x2DA570 | 5 | Stored; no reader is known. |
| stunTimeMax | float s | 0x2DA574 | 8 | Stored; no reader is known. |
| threshold1 (the name gives no unit) | int | 0x2DA578 | 33 | Stored; no reader is known. |
| threshold2 (the name gives no unit) | int | 0x2DA57C | 66 | Stored; no reader is known. |
| Gust shape axis (start-up value) | 16-byte vector | 0x3932E0 | (0, 1, 0, 0) | The constant up vector, a unit vector. It is one of five constant vectors (0x3932C0 to 0x39330F) written once at boot by a start-up initialiser (SPEC-0026, Start-up table walk) and never changed; it is not in the file image, so the clean side sets it itself. The gust connect callers pass the whole vector as the axis of both shapes in the gust connect shape test (see that section). High. |
| Lightning wind-up, stage 0 | float s | 0x2DA59C | 2.0 | Stage 0 is unreachable (see Lightning). Keep the value. |
| Unknown factor | float | 0x2DA5F0 | 0.1 | Unused: no instruction of the boot program loads this word. The 0.1 steer weight in Arena movement and the companion's 0.1 steer weight are fixed values, not this word. Medium-High. |
| Unknown count | int | 0x2DA614 | 128 | Unused: no instruction loads this word. The gust fade scale 128 (Air-flow sub-steps) is a fixed value, not this word. Medium-High. |
| Rapid animation multiplier (`rapidAirFlowAnimSpeedMult`) | int | 0x2DA58C | 6 | Unused: registered as a tuning word, but no instruction loads it. Rapid air-flow uses the stage gust speeds. Medium-High. |

The "no reader is known" and "Unused" rows above are not used by the behavior. Don't apply them.

Developer names (identifiers, as registered): 0x2DA538 is the stage-3 maximum for lightning cooldown. Its registered name repeats `lightningTimerMin3` (the stage-3 minimum at 0x2DA534); the maximum's name should be `lightningTimerMax3`. The wind-up names are one index ahead of the stage: stage 1 uses `lightingBoltWindupTime2` (1.0 s), stage 2 uses `lightingBoltWindupTime3` (0.5 s), and `lightingBoltWindupTime1` (2.0 s) is the stage-0 word, which is unreachable (see Lightning). The hesitation words 0x2DA4E0 and 0x2DA4E4 are registered as `decisionHesitationMin3` and `decisionHesitationMax4`; they are not the stage-3 pair, and the name "Max4" has no fourth stage.

Fixed values of the boss, not tuning words (High): the reaction sphere radius 5 and the shot look-ahead factor 10; the reaction distance 10 and facing gate 0.5; the ground-probe ceiling 8 (boss and balls; the probe of SPEC-0018 searches for the highest surface below the point raised by 8); the ball spawn distance 2.0 and the spawn and bounce blend ranges 0.5 and 0.9; the beam sweep step 3.0; the beam cylinder radius 1.0; the reflected-beam reach 5.0 and lift 4.0; the aim lift 1.5; the facing gates 0.75, −0.75 and 0.99; the steer weights 0.1 and 1.0 and the air-flow facing reach 5.0; the gust charge step 0.25 and cap 10; the gust fade scale 128; the knock-back pair 9 and 12; the stage thresholds 33 and 66.

## Behavior

### Placement and reachability (High)

- **Not placed.** The Cloud Beast is not placed anywhere on the disc, so the fight does not occur in normal play. No character record in any region or world pack has its monster type (type 1) or its companion's type (type 2), so no model or animation set is linked to it. No level script spawns it or gives an enemy its class, and no quest entry, movie or fight name refers to it. A child NPC in Brightvale Castle has a pretend-play line about a large cloud creature flying high in the sky; it does not identify the class-9 boss or say where a fight would happen. The Wind Mote's pickup text separately compares the item to a tiny thundercloud; this is item flavor text, not a character or boss reference (SPEC-0024, Table 0; SPEC-0021). Confidence: High for the negative placement findings; neither incidental text establishes a connection to this boss.
- **No encounter-ending logic.** The controller has no hero-death check or explicit victory/defeat transition, cutscene, reward, story update or arena exit. It deals hero damage through shared character methods; when the reflected-beam reply reduces the boss to zero HP, shared death and refresh methods run, but the controller adds no boss-specific aftermath. The death hook places a class-9 actor in base state 16, and the shared character update skips AI stepping on later updates while that state remains set; ordinary corpse timing and streaming cleanup then apply (SPEC-0029, Behavior 34; High). This does not establish whether the same update finishes its current controller work or whether an external scene script exists. Confidence: High for the controller and shared-state boundary; no placed instance or linked scene was found.
- **No model on disc.** No model, skeleton or animation set for the Cloud Beast or its companion was found on the disc. Every model set, skeleton and clip bank in the character data is used by a character record of another type, and the boss code needs a bank of at least 24 clips. The monster rows supply behavior and combat parameters but do not identify a model, skeleton or animation table; those are selected by the instantiated character record. No record of type 1 or 2 exists, and no unassigned character-region model, skeleton or clip bank supplies a direct asset binding. The conditional type-0 row-overflow path described above changes the selected row and class but keeps the donor actor's existing rig and animation table; it does not identify the intended Cloud Beast asset. Confidence: High that no unused character-region skeleton or clip bank exists and no type-1/type-2 binding is shipped; Medium-High that no dedicated Cloud Beast model is identifiable in the indexed disc resources. A private packet-space visual check found no recognizable boss mesh among resources 544–575, while resource 528 remains inconclusive because its transforms and material-to-texture bindings are unresolved; these checks cannot rule out an environmental asset or a removed/development-only model.
- **Missing effect definitions.** The controller requests group-3 definitions 447 and 448 when setting up each lightning beam, 449 for the gust stream, 450 for the five ball visuals, 451 for ball expiry/contact bursts, and 454 for the shock effect handle. All six definitions are absent from the disc's effect library. The beam objects still have separate geometric state that the controller updates, so the missing definitions do not by themselves establish whether a bare beam line is rendered. The disc preserves the attack logic but does not identify these requested effect appearances. Confidence: High for the requests and missing definitions; unknown for what the missing assets would have looked like.
- **Sound and clip limits.** The controller makes no direct sound-cue call. Although it requests animation indices on the boss and companion, there is no linked type-1/type-2 model, clip bank or animation sound table from which to identify the intended clips or cues. The controller reveals state transitions and event gates but not clip frame counts or playback lengths; the exact frame of ball-spawn event 32 and total durations of animation-gated attacks cannot be recovered. Confidence: High for the absence of a linked sound or animation asset in the shipped disc data; the missing clip timing is unrecoverable from this build.
- **Behavioral rig contract (not recovered art).** A compatible boss rig needs a main skeleton whose bone 6 supplies the gust origin and whose bone 11 supplies the companion attachment point. Its animation bank must provide indexed entries through 23; the controller requests entries 4, 5, 6, 7, 8, 10, 11, 12, 13, 14, 15, 16, 22 and 23. The companion must be a separately addressable animated entity with pose point 1, used as the lightning source, reaction-sphere centre and boss look-at point. The companion is attached and receives the boss's animation requests and animation speed. For those forwarded IDs to select companion-specific actions rather than fall back, the companion's main action table also needs entries through the highest requested ID, 23; a missing ID resolves to clip 0 under the shared animation lookup (SPEC-0005, Lookup miss). This describes a compatible reconstruction, not evidence that the missing original companion table used those mappings. These are functional attachment points, not evidence of original bone names or clip meanings. The disc does not establish either model's silhouette, proportions, topology, rest pose, skin weights, bone naming, or original animation poses; the monster-row collision radius and height are gameplay dimensions, not mesh measurements. A newly authored rig can satisfy these constraints, but its visual design and animation interpretation would be reconstruction choices.
- **What remains.** The behaviour (class 9), the tuning words above and two monster rows remain. Row 1 (the boss) has default class 9, maximum HP 240 (row field 0x30), radius 3.0 and height 3.5, and no attack slots, so all of its damage comes from the tuning words above. Row 2 (the companion) has default class 10 and HP 50.
- **Class reachability.** No recorded request directly names class 9, and no type-1/row-1 Cloud Beast placement is on disc. A conditional indirect route exists in code: a Minion Wizard can select an injured type-0 character without a class/faction match, and its kind-6 spell can re-pick that character to variant 1/row 1 on one random outcome. The spell changes the character's row and palette but not its active class; a later row-default request could then select class 9. The only same-pack type-0/Minion-Wizard pair in the census is pack 1259, an unreachable cut variant of 909. The other type-0 record is the Black Knight in pack 989; its recorded neighboring packs and its module-loaded character region contain no Minion Wizard. Accordingly, the recovered normal placement, module-load and transition data provide no Minion Wizard hit on a type-0 target and no shipped class-9 route. A later default-class request remains mechanically possible after a hypothetical mutation; this does not establish a planned Cloud Beast fight (SPEC-0029, Run-time variant re-pick; SPEC-0038 Behavior 43).
- **Test spawn.** To run it (for testing), spawn monster type 1, variant 0: this selects row 1, whose default class is 9. The boss looks for its companion as the entity whose row default class is 10 (row 2, type 2). Row 2 is not placed either, so in a test spawn of the boss alone the companion lookup finds nothing. The companion is needed: it is the lightning source and the centre of the reaction sphere, and without it those two steps have no defined result (see Companion and Test spawn walkthrough). Spawn type 2, variant 0 as well.
- **Lethal-hit aftermath of the row-1 test spawn (High, conditional).** If an accepted hit reduces this ordinary class-9 test spawn to zero HP, the shared death hook puts it in base state 16. Row 1 does not reanimate and has a 510-tick (8.5 s) death-disappear timer; the common corpse update and proximity-streaming release then apply (SPEC-0029, Behavior 34). This is generic character cleanup, not a boss victory, reward, story update or arena exit. It establishes no behavior for a missing original placed encounter or an external scene script.
- **Companion.** Its default behavior class (10) has no independent actions; it waits one game tick and repeats. The boss drives the companion. Each tick the boss looks for the companion if it has none yet: the last entity in the character list whose row default class is 10 (High). Until one is found the attachment step does nothing. Once found, every tick: the companion is placed at the boss's skeleton bone 11, its facing is steered with weight 0.1 toward the active hero using the horizontal direction from companion to hero, its animation speed is set to the boss's, and the boss's look-at point is set to the companion's pose point 1. Every animation the boss plays is also requested on the companion (High).
- **Arena.** The six arena points are the level locators with user ids 101 to 106. Only two levels on the disc hold all six, and both are other bosses' arenas: the Plague Serpent arena, which uses ids 101 to 124 as its anchor points (SPEC-0016), and the Juppie arena, which uses ids 101 to 106 as stage locators (SPEC-0015). In any other level the boss finds no points and stays in place (see Edge cases).
- **Rendering residue.** The boss sets its character fade level to 129. This sends an active character through the composite rendering path but sorts it as opaque; it is not material opacity (SPEC-0003, SPEC-0046). It provides no information about the missing model's appearance. Confidence: High.
- **Story.** No story slot exists for the fight; each Act 3 area has its own boss (SPEC-0049).

### Distances

Every distance test in this spec is the horizontal distance from the boss to the hero (x and z only; height is ignored). The direction used by the shock, reaction and aim tests, and the direction in the reaction dot-product test, is the horizontal unit direction from the boss to the hero. Confidence: High.

### Each tick, in order

1. Count down each active timer by one tick (a value above 0 drops by 1): hesitation, the three attack cooldowns, and the current attack's sub-timer.
2. Clear the gust fade flag and reset the animation speed to 1.0.
3. **Stage**: hp% = trunc(100 × HP / max HP) (the quotient is taken in single precision). If hp% < 33, the stage becomes 3. Otherwise, if hp% < 66, the stage becomes 2. Otherwise the stage is unchanged. The stage starts at 1. Stage 2 never returns to stage 1. Stage 3 drops to stage 2 only when healing leaves it between 33 % and 66 %; at 66 % or above it stays stage 3.
4. **Reaction check**: the trigger is evaluated in every state except air-flow and rapid air-flow, including lightning, the special reply and the reaction itself. If the trigger (below) is true, the boss switches to the reaction state and stops both lightning beams.
5. **Decide**, if idle (below). The decision step does nothing in the special-reply and reaction states.
6. Run the current state.
7. Arena movement (see Arena movement), then draw the gust stream, then update the balls, then attach the companion (see Companion).

### Deciding (idle only)

When the hesitation timer reaches 0, the boss makes one decision:

1. Roll r = random integer 0–100 (inclusive). Let F, A, R, L and B be the stage's fidget, air-flow, rapid air-flow, lightning and bouncing weights. Let cA, cL and cB say whether the air-flow cooldown (which also covers rapid air-flow), the lightning cooldown and the bouncing cooldown are running.
2. **Fidget.** If r < F, the boss fidgets. The fidget has no cooldown.
3. **Air-flow.** If r < F + A: if cA is clear, choose air-flow. If cA is running, set base = F. If r ≥ F + A, set base = F + A. (So while cA is running, the air-flow share is removed from the rolls that were inside the air-flow band.)
4. **Rapid air-flow.** If cA is clear and r < base + R, choose rapid air-flow. (With cA clear this is the band F + A to F + A + R.)
5. **Lightning.** Let U = base + R + L. If r < U: if cL is clear, choose lightning. If cL is running, lower U by L and continue.
6. **Bouncing.** If r < U + B and cB is clear, choose bouncing.
7. Otherwise the boss stays idle for this decision. Roll 100 is outside every band (the five weights of each stage sum to 100), so it always leaves the boss idle (1 roll value in 101).

On every choice the boss clears the attack sub-state word and the hit latch, and it sets the hesitation timer to a random integer number of ticks from trunc(60 × minimum) to trunc(60 × maximum) inclusive, from the stage's range (see Edge cases for the stage-3 range). A lightning choice also sets the wind-up sub-timer (see Lightning).

**Random draws** (High). One shared random stream (the game-wide generator of SPEC-0007). An integer draw from a to b returns a + (n mod (b − a + 1)) for an unsigned 32-bit stream value n, so both end values occur. A real draw from a to b returns a + n × ((b − a) × 2^−32) in single precision; a is reached, and b only through rounding of the largest values of n. This covers the decision roll (0 to 100), the 50 / 25 / 25 rolls (0 to 100), the ring step (1 to 3), the rapid count (3 to 5), the reply coin (0 to 1), the hesitation ticks, the cooldown seconds and the ball blend draws.

Worked values for stage 2 (F 10, A 30, R 5, L 35, B 20):

- Air-flow cooling, lightning ready: rolls 10 to 79 choose lightning; rolls 80 to 99 choose bouncing. Rolls 0 to 9 fidget.
- Lightning cooling, air-flow ready: rolls 0 to 9 fidget, 10 to 39 air-flow, 40 to 44 rapid air-flow, 45 to 64 bouncing, 65 to 79 idle, 80 to 99 bouncing.
- Both air-flow and lightning cooling: rolls 0 to 9 fidget, 10 to 34 bouncing, 35 to 39 idle, 40 to 64 bouncing, 65 to 79 idle, 80 to 99 bouncing. (Medium: the lowered lightning band falls through to the bouncing check; check against a clean build when available.)

If the decision picks nothing, the boss stays idle, and the shock check below applies.

**Shock**: whenever the boss is still idle after deciding (including ticks while hesitation is counting), if the horizontal distance from the boss to the hero is less than 15 units, it starts the shock.

### Reaction trigger

The trigger branches on the hero variant word (0 Tor, 1 Roberta; SPEC-0036), not on an action state. Confidence: High.

- **Tor (variant 0).** The trigger is true when Tor's in-attack test passes, the horizontal distance from the boss to the hero is less than 10 units, and the dot product of the boss's forward direction with the horizontal unit direction from the boss to the hero is greater than 0.5. The in-attack test is the hero's own query (shared with SPEC-0012 and other bosses): it passes while Tor is in action state 13 with an attack sub-state from 2 to 12, or in action states 7 to 11 (the jump states) while his hero word at +0x1508 is 8 or 9 (the meaning of that word is Medium-Low).
- **Roberta (variant 1).** For each of her six shot slots that is armed (SPEC-0036 Hero shot slots), take the segment from the shot's position S to S + 10 × V, where V is the shot's direction times its speed times the run-time frame step (the distance the shot covers in one update). The trigger is true when any such segment touches a sphere of radius 5 centred on the companion's pose point 1. So this mode reacts to Roberta's shots heading at the companion, not to melee volumes, and her own melee does not trigger it.
- **Any other variant value.** The trigger is false.

While the trigger stays true, the reaction state restarts its 1.5 s hold (see Reaction state).

### Arena movement

The boss does not travel between points; it is placed on its current point every tick and only its facing is blended (High).

1. **Position.** The boss's position becomes the current point's position (locator position, SPEC-0006), and its height is then replaced by the ground probe at that spot with ceiling 8 (SPEC-0018 Ground probe: the highest surface below the point raised by 8; 0.0 when the level has no grid there). When the current point changes (end of the gust, air-flow sub-step 1), the boss is on the new point on the next tick.
2. **Facing.** The boss's steer method is called with a weight w and a horizontal unit direction (the facing moves toward the direction by the blend w; w = 1.0 turns it fully). Default: w = 0.1 and the current point's facing vector (the locator facing, SPEC-0006).
3. **Air-flow state only** (state 3; the rapid air-flow state uses the default): the direction is the horizontal unit direction from the boss to a target T, with:
   - sub-step 0: T = the next point, w = 0.1;
   - sub-step 1: T = the next point, w = 1.0;
   - sub-step 2: T = the previous point (the point just left), w = 1.0;
   - sub-step 3: T = the boss's position plus 5 times the current point's facing, w = 0.1 (the same direction as the default);
   - sub-step 4 (one tick): T = 5 times the boss's forward vector taken as a point, w = 0.1 (an artefact; it lasts one tick).

### States

| State | Animations | Behavior | Ends |
| --- | --- | --- | --- |
| Idle | 4 | Hover | — |
| Shock | 11, 12, 13 (sub-steps below) | Starts a radius-8 effect for up to 60 ticks (1 s). During the active interval, once per shock, if the horizontal distance to the hero is 8 units or less, deal 10 damage. Leaving the radius prevents further damage but does not end the shock. | Timer expires, then animation 13 finishes: idle |
| Fidget | 5 | No damage | Animation done: idle |
| Air-flow | 6, 7, 22 (sub-steps below) | One gust cycle: wind-up, gust, second gust, run-out. | Sub-step 4: idle; air-flow cooldown drawn |
| Rapid air-flow | 6, 7, 22 (sub-steps below) | The gust cycle repeated. The count k (3 to 5) is drawn on entry and the attack runs k + 1 cycles. | After the last cycle: idle; air-flow cooldown drawn |
| Lightning | 8 (wind-up), 8 (fire phase), 10 (recovery) | Wind-up for the stage's wind-up time, then the fire phase (see Lightning). | A reflected beam that points at the boss sends it to the special-reply state (below). Otherwise, after recovery: lightning cooldown drawn, then the reply roll (below; the rapid choice draws its count, 3 to 5) |
| Bouncing bolt | 14 | On the animation's event 32, once per decision, spawn 2 / 3 / 4 balls into the 5-slot pool; a ball that finds no free slot is not spawned and is not retried | Animation done: idle; bouncing cooldown drawn |
| Special reply (state 7) | 15 or 16 (chosen by a fair coin when the state is entered) | Both lightning beams were stopped on entry. Until the chosen animation is current, the boss requests it every tick (with a hit effect). When it is current and has finished, the boss takes 20 damage. The reaction trigger can still cut the state short, before the damage. | After the damage: the reply roll |
| Reaction | 23 | See Reaction state | Hold over: the reply roll |

**Reply roll** (lightning end, special reply, reaction): draw r1 from 0 to 100. If r1 >= 50 the boss goes idle (51 of 101). Otherwise draw r2 from 0 to 100: r2 < 50 gives air-flow (chance 2500/10201, about 24.5 %), otherwise rapid air-flow (2550/10201, about 25.0 %). Both attack choices clear the hit latch, reset the sub-state and pick the next point; the rapid choice also draws its count. The roll does not check the air-flow cooldown (High).

Shock sub-steps (High):

- **Sub-step 0**: wait for animation 11 to start, then set a 60-tick timer.
- **Sub-step 1**: play animation 12 while the timer is above 0. Start the radius-8 shock effect at the boss; while the timer runs, if the hero is within 8 horizontal units and the hit flag is clear, deal 10 damage and set the flag. Leaving the radius prevents a later hit but does not end the timed effect.
- **Sub-step 2**: when the timer expires, stop the effect and play animation 13. Return to idle when animation 13 finishes.

When entering air-flow or rapid, the boss also picks its next arena point: with the points as a ring 0–5, next = (current + random integer 1–3 + 1) mod 6. That is 2, 3 or 4 steps ahead, never the same point or a neighbour, each with chance 1/3 (the draw includes both ends). Confidence: High.

### Air-flow and rapid air-flow sub-steps

Both states share sub-steps 0 to 4. The gust is a straight column with two ends, the origin O and the end E, kept in the boss record; it is not aimed at the hero (High). The controller updates the gust-stream handle from O to E on every tick of sub-steps 1 to 3, but its requested particle definition 449 is absent (Placement and reachability). Let B6 be the position of the boss's skeleton bone 6 this tick, f the boss's forward vector and c the charge value.

- **Sub-step 0, wind-up**: animation 6. When it ends, c = 0.
- **Sub-step 1, gust**: animation 7 at the stage's gust speed (1.25, 2.0 or 3.0), set every tick. c rises by 0.25 per tick up to 10 (before use, so the first gust tick has c = 0.25). Then O = B6 and E = O + c × f: the column grows out of bone 6 along the boss's facing, which turns fully toward the next point in this sub-step (Arena movement). A fade value trunc((1 − p) × 128) is kept, where p is the animation's progress from 0 to 1 (Low: the effect's use of it). The gust connect test runs each tick while the attack's hit flag is clear (see Gust connect test). When animation 7 finishes, the boss's arena points are updated: the current point takes the next point, the previous point takes the old current point, and the next point is cleared (High). From the next tick the boss stands on the new point.
- **Sub-step 2, travel**: animation 22 with animation speed 0 (High), fade 0. Let w = B6 − E. If |w| < 0.5, go to sub-step 3. Otherwise both ends move by 0.5 × w / |w|: the column keeps its length and direction and slides at 0.5 units per tick toward the boss's bone 6, which is now on the new point. So the gust crosses the arena from the old point to the new one. The gust connect test runs here as well while the flag is clear.
- **Sub-step 3, run-out**: animation 22 at the stage's gust speed. The fade value is trunc(p × 128). c falls by 0.25 per tick down to 0. Then E = B6 and O = E + c × f (the column now points from the boss along its facing, with length c). The gust connect test runs each tick while the flag is clear. When animation 22 finishes, go to sub-step 4.
- **Knock-back.** A gust connect deals 20 damage (not blocked, kind 2) and then knocks the hero back: the hero's velocity is set to 7.0 × normalize(−9 f_h + (0, 12, 0)) = (−4.2 f_h.x, 5.6, −4.2 f_h.z), where f_h is the hero's own horizontal forward vector (so the push is away from the hero's facing, not away from the boss), and the hero is sent to action state 48 (the knock-back of SPEC-0036 with K = 7.0, the form SPEC-0015 uses with the reversed hero facing).
- **Sub-step 4, end** (air-flow): the boss returns to idle and draws the air-flow cooldown.

**Rapid air-flow**: with a drawn count k (3 to 5), the attack runs k + 1 gust cycles. After each cycle the boss checks the count before decrementing it. If the count is zero, the attack ends: the boss returns to idle and draws the air-flow cooldown. Otherwise the count drops by one, the next arena point is chosen, and the boss goes back to sub-step 1 (the gust), with no new wind-up. Example: k = 2 gives cycles with counts 2, 1 and 0 at the check, so three cycles. Confidence: High.

**Gust connect test (once per decision)**: the gust connect test runs on each tick of sub-steps 1, 2 and 3 while the attack's hit flag is clear. Its first connect deals 20 damage, knocks the hero back and sets the flag. The flag is cleared only when a new decision or reply-roll choice is made, so at most one gust hit lands per decision, however many cycles the rapid attack runs. Confidence: High. The test is the overlap test described in Gust connect shape test. The gust routine exists as two copies (air-flow and rapid air-flow); they are one behavior, except that only the air-flow state steers the boss's facing toward the points (Arena movement).

**Cooldown draw**: all three cooldowns (air-flow, lightning, bouncing) draw a real number u from the stage's minimum to maximum seconds (Random draws), and use ticks = trunc(60 × u). Confidence: High.

### Gust connect shape test

The gust connect test is one call of the shared finite-cylinder overlap of SPEC-0018 (Finite-cylinder overlap, kind 3 routine), which gives every branch, the depth sign and the degenerate cases. Shape A is the hero's body and shape B the gust; the boss's own body takes no part (High).

- **Shape A (hero).** Radius = the hero's body radius (word +0x08 of the hero's motion record); full length H_A = the hero's body height (word +0x0C of the same record); centre = the hero's position plus (H_A / 2) along the up vector; axis = the up vector (0, 1, 0).
- **Shape B (gust).** Radius 6.0 (0x2DA5EC); full length L = |E − O| (equal to the charge value c in sub-steps 1 and 3, and kept unchanged while the column slides in sub-step 2); centre = (O + E) / 2; axis = (E − O) / L, the boss's facing in sub-step 1 and its reverse in sub-step 3.
- **Branch.** The boss's facing is horizontal, so the axes are perpendicular and the test takes the general rule of SPEC-0018, not the parallel rule. Its quirks apply: for example, when the hero's centre lies on the gust's axis line the routine misses the overlap (SPEC-0018 Edge cases).
- **Zero length.** When c has fallen to 0 in sub-step 3, L = 0 and the axis is the zero vector. The hero has positive body height, so neither shape can contain the other and the shared overlap routine finds no connect (High).

The gust connect test uses only the overlap result (true or false); the depth and push direction are not used there. It runs once per decision (see Gust connect test).

### Lightning

- Wind-up is animation 8. The fire phase also plays animation 8. Recovery is animation 10. The source of every bolt is the companion's pose point 1 (0x2DA5E4), not a point of the boss (High).
- **Wind-up time** by stage: stage 1 uses 1.0 s; stage 2 uses 0.5 s; stage 3 sets no new value and keeps the sub-timer as it was (usually 0, so stage-3 lightning fires almost at once). Stage 0 would use 2.0 s, but the stage starts at 1 and only 2 or 3 is ever written, so stage 0 is unreachable. Confidence: High.
- **Beam objects.** A beam is reset by a start call: its drawn length becomes 0, its target length a very large value, and its growth limit 500.0 (0x2DA5F4). Each later set call with a start point and an end point makes the target |end − start|; if the drawn length is below the target by more than 500.0 it grows by 500.0, otherwise it becomes the target (it also shrinks at once). The beam is drawn from the start along the unit direction for the drawn length. "Fully extended" means the target is at most the drawn length. So 500.0 is a growth rate per update and, at arena sizes, beams reach their end in one update (High).
- **Start of the fire phase**: when the wind-up ends, beam 1 is reset, the fire timer is set to 3, 2 or 1 s (times 60 ticks) by stage, and the hero's position is written to the boss record (overwritten before use, see Recorded hero position).
- **Aim (each fire tick).** Let P be the hero's position, f_h the hero's horizontal forward vector, u the horizontal unit direction from the hero to the boss and D the horizontal distance between them. The aim point is A = P + (0, 1.5, 0) (the lift is on the vertical lane). If the hero's guard query is true (SPEC-0036; the query other bosses use for blocking) and dot(u, f_h) > 0.75 (the hero faces the boss), A moves 1.65 units along u, toward the boss: the beam then ends at the guarding hero's shield. Beam 1 is set from the source S to A, animation 8 plays, and len is the beam's drawn length; d = (A − S) / |A − S|. Confidence: High.
- **Hits.** The fire timer counts down in ticks. After the wall sweep (Second beam (a)), if the latch is set or fewer than six ticks of the fire phase have elapsed, the tick ends. Otherwise:
  1. **Connect.** The beam is a cylinder of radius 1.0 and full length len, centred at S + 0.5 × len × d (the middle of the drawn beam), with axis d. It is tested against the hero's body cylinder (radius, height and centre as in Gust connect shape test) with the shared overlap of SPEC-0018. Call the result "overlap".
  2. **Guard.** If overlap is true or beam 1 is fully extended, and the hero's guard query is true, and dot(d, f_h) < −0.75 (the hero faces into the beam):
     - with the Fog Mote (item 166, SPEC-0021) in the hero's shield word (word 1 of the hero's Mote words, SPEC-0036), the tick is a reflect candidate (Second beam (b));
     - otherwise the hit is blocked: the hero's take-hit runs with amount 0 and the blocked flag set (so the hero plays the block reaction, SPEC-0036), the latch is set, and no damage follows on this tick.
  3. **Damage.** If the tick is not a candidate, beam 2 is stopped. Then, if overlap is true and the hit was not blocked, the hero takes 10 damage (blocked flag clear, kind 2) and the latch is set.
  So an unguarded hero takes damage only on overlap; a guarding hero who faces the beam is never damaged by it. Confidence: High.

### Second beam (two triggers)

Confidence: High.

Stopping: the second beam stops on a bolt tick that is not a reflect candidate. Ticks that end early (the six-tick gate, a set latch, or a wall bounce) leave it running.

The second beam is reset by either trigger below when it is not active; it is placed (set) on the later ticks of the same trigger.

- **(a) Wall bounce.** On any fire tick, before the six-tick gate, the beam is swept against the level's collision triangles: probe points every 3.0 units along d from S up to len, each point's grid cell, every triangle of that cell tested against the segment from S to A, nearest fraction kept. If a triangle is hit, beam 1 is set from S to S + d × len × fraction. If the second beam is not active it is reset; otherwise it is set from that hit point to the hit point plus v' = v − 2 (v · n) n, where v is the vector from the boss to the hero and n is the unit normal of the hit triangle. The tick ends there, so a wall-hit tick cannot damage the hero.
- **(b) Reflect candidate.** Let K = S + len × d (the end of beam 1, the aim point when the beam has reached it). If the second beam is not active it is reset and the tick ends (no damage, no latch change). Otherwise the second beam is set from K to R = P + (D + 5.0) × f_h + (0, 4.0, 0): out of the shield along the hero's facing, 5 units past the boss's distance and 4 units up. If dot((R − K) / |R − K|, u) > 0.99, the boss enters the special-reply state: the reply animation is drawn (15 or 16 by a fair coin) and both beams stop. There is no offset along the boss's facing.
- **Reflect geometry.** With the hero facing the boss exactly, on level ground and with the beam at full length, the vertical gap between K and R is 2.5 and the horizontal run is D + 3.35, so the dot is (D + 3.35) / sqrt((D + 3.35)² + 6.25). It exceeds 0.99 only for D greater than about 14.2. Closer than that the reflection never counts; farther away, the hero must face the boss within about 5 degrees (Medium for the angle, which depends on the heights).

### Recorded hero position

At fire start the hero's position is written as a four-word vector into the boss record (word indices 0x13C to 0x13F). On every fire tick the aim point is computed and written to the same words before they are read, and only the lightning fire phase reads them. So the value written at fire start has no effect. Confidence: High.

### Special-reply state (state 7)

- Entered only from a reflect candidate as above. Both beams are stopped on entry.
- Until the chosen animation (15 or 16) is current, the boss requests it every tick and plays a hit effect. When it is current and has finished, the boss takes 20 damage (special-reply damage tunable) through the shared HP subtraction: if 20 is below the boss's HP (HP field at offset 0x154 of the entity record) it is subtracted; otherwise HP becomes 0 and the entity's death method runs. Then the entity's refresh method runs. Then the reply roll.
- The same subtraction is used by the shared NPC take-hit and by the hero's take-hit, so a reply hit counts like any other HP loss for the stage rule. The class-9 behavior coroutine does not replace the character object's shared NPC hit method: after a normal hero hit scan accepts the boss and dispatches a hit, that method can reduce the boss's HP (SPEC-0029 Behavior 34; High, with the missing hurt-capsule condition above). When the shared NPC combat-state word is not agro, the take-hit route doubles the incoming damage before subtraction. A real scan still needs positive-length hurt capsules, which cannot be confirmed because the disc has no Cloud Beast rig.
- The reaction trigger is still evaluated in this state, so a reaction can replace the reply before the damage lands (High).

### Bouncing balls

- **Spawn.** Each ball takes a free slot of the 5-slot pool; with no free slot it is not spawned. Its slot requests particle definition 450, which is absent (Placement and reachability). Position (and previous position) = the boss's position + 2.0 × f, where f is the boss's forward vector. Horizontal direction = normalize((1 − r) × f + r × p), where r is a real draw from −0.5 to 0.5 and p = normalize(f × up), the horizontal perpendicular. The blend is lopsided: r = 0.5 turns the ball 45 degrees to one side, r = −0.5 about 18.4 degrees to the other. Horizontal speed = ball speed / 60 per tick (10 units per second). Vertical velocity 1.0, travel 0.
- **Each tick, in order:** position += direction × horizontal speed; position.y += the current vertical velocity; vertical velocity −= 0.1 (gravity per tick).
- **Falling ticks only** (vertical velocity now below 0): g = the ground probe at the ball with ceiling 8 (SPEC-0018). If position.y ≤ g − 1.0, the ball bounces: direction = normalize((1 − |r|) × direction + r × perpendicular), r a real draw from −0.9 to 0.9, and vertical velocity = 1.0. Then travel += the horizontal distance from the previous position to the current one, and the previous position becomes the current one. If travel > 50 (0x2DA5C0), the ball despawns with a burst effect. Because the previous position is updated only on falling ticks, travel sums to the horizontal path length; at 10 units per second a ball lasts about 5 s.
- **No restitution.** Horizontal speed never changes and every bounce launches at 1.0 per tick. From a bounce at 1.0 with gravity 0.1 per tick the apex is 5.5 units above the bounce point (sum of 1.0 − 0.1k for k = 0 to 9). Confidence: High.
- **Touch** (every tick, once per ball): the hero's body cylinder (radius, height, base at the hero's position, up axis) is tested against a sphere of radius 2.0 at the ball. On a touch the ball despawns and the hero takes 50 damage (no attacker id, kind 2). The ball code requests group-3 effect 451 on touch and range expiry, but that definition is absent from the disc's effect library, so no particle burst is created by the normal lookup path; its intended appearance is unknown.

### Lightning hit latch

- During the fire phase the latch is cleared on each tick where the remaining count is a multiple of 60. The countdown runs before the state, so the first fire tick already shows the fire time minus 1: in stage 1 (180 ticks) the clears seen are at 120 and 60, in stage 2 (120 ticks) at 60, and in stage 3 (60 ticks) none. The first window relies on the latch cleared by the decision. With the six-tick gate, at most 3 hits land in stage 1, 2 in stage 2 and 1 in stage 3. Confidence: High.
- The clear is tied to the counter, not to the time of the last hit. A hit just before a boundary can be followed by another a tick later.

### Reaction state

- When the trigger is true on a tick of this state, animation 23 is requested and the hold is set to trunc(60 × 1.5) = 90 ticks. On a tick with the trigger false, a hold above 0 drops by 1; at 0 the reply roll runs. Both beams were stopped on entry.

### Test spawn walkthrough

A rebuild spawns monster type 1, variant 0 (row 1, class 9) in the Juppie arena, whose locators 101 to 106 all exist (SPEC-0015), and the hero stands still about 20 units away, not guarding. Tick 0 is the controller's first tick.

- **Tick 0, start-up.** The record starts in idle, stage 1, with the hesitation timer at a random 60 to 120 ticks, the air-flow cooldown at 180 ticks (3 s, so the first attack is never an air-flow or rapid gust), the lightning and bouncing cooldowns at 0, the current point = point 0 (locator 101) and no next point. All six points are found. The three effect objects and the ball effects are loaded.
- **Every tick from then on:** timers count down; the stage stays 1 (HP 240 of 240); the reaction trigger is checked; the boss is placed on locator 101 at the ground height under it and turns 10 % per tick toward that locator's facing; idle animation 4 plays; the companion lookup runs.
- **Companion.** With type 1 alone the lookup never succeeds, so nothing rides on the boss. Spawn type 2, variant 0 too: from the first tick after it exists it is found, pinned to the boss's bone 11 and given the boss's animations. Without it, the first lightning fire tick and Roberta's reaction check read the companion's pose through an empty reference, which has no defined result; a rebuild should skip those steps (or not run the test without the companion).
- **Shock.** If the hero comes within 15 units horizontally while the boss is idle, the shock starts at once (it does not wait for the hesitation).
- **Tick H (60 to 120), first decision.** Roll r from 0 to 100 with the air-flow cooldown still running: r 0 to 19 fidget (20 of 101), r 20 to 99 lightning (80 of 101; the cooling air-flow band falls through to lightning and the rapid band is empty in stage 1), r 100 idle (1 of 101). The hesitation is redrawn (60 to 120 ticks). A fidget plays animation 5 and returns to idle; the next decision comes after the new hesitation.
- **Lightning, ticks H to H + 60.** The wind-up sub-timer is 60: animation 8 plays while it counts down.
- **Tick H + 60.** The sub-timer reads 0: beam 1 is reset, the fire timer set to 180, the hero position recorded.
- **Ticks H + 61 to H + 239 (fire, remaining 179 down to 1).** Each tick: the aim point is the hero's position raised 1.5; beam 1 runs from the companion's pose point 1 to it and reaches it at once; the wall sweep runs. From remaining 174 (six ticks elapsed) the beam cylinder is tested against the hero; an unguarded hero inside it takes 10 damage, then the latch holds until remaining 120, again until 60: at most 30 damage in total.
- **Tick H + 240.** Both beams stop, recovery animation 10 plays; when it finishes the lightning cooldown is drawn (5 to 10 s) and the reply roll picks idle (51 of 101), air-flow or rapid air-flow. By then the air-flow cooldown has expired, though the reply roll does not check it in any case.
- **First gust (if chosen).** The next point is 2, 3 or 4 steps around the ring. Wind-up animation 6, then the column grows from bone 6 toward that point (10 units after 40 ticks), the boss jumps to the point when animation 7 ends, and the column slides across the arena to it at 0.5 units per tick; one connect costs the hero 20 HP and a knock-back.
- **Getting the boss to stage 2** needs the reflect: give the hero the Fog Mote in the shield word, guard facing the boss from more than about 14.2 units during the fire phase, and each accepted reflection costs the boss 20 HP: stage 2 after 5 reflections (HP 140), stage 3 after 9 (HP 60), death after 12.

## Edge cases and error handling

- **Band rule with cooldowns** (reproduce it): a cooling-down air-flow band does not drop out of the roll. Its rolls fall through to lightning, and a cooling lightning band lowers the lightning share and lets the rest fall to bouncing (see Deciding). Do not treat a blocked band as a weight of 0.
- **Latch tied to the counter** (High): the once-per-window limit is a fixed clock, so two hits can land within a few ticks of each other across a multiple of 60. Reproduce the counter rule, not a rolling one-second window.
- **Wind-up quirk** (reproduce it): stage 1 uses the 1.0 s wind-up stored in the table, stage 2 uses 0.5 s, and stage 3 sets no new wind-up; it keeps whatever the sub-timer held, which is usually 0, so stage-3 lightning fires almost at once. Confirmed (High).
- Stage-3 hesitation uses the stage-2 range (0.5–1.0 s).
- **Rapid cycles restart at the gust**: the second and later rapid cycles begin at sub-step 1, with no new wind-up.
- **Gust hit once per decision**: later cycles and sub-steps deal no new gust damage until a new decision is made (see Gust connect test).
- **Missing arena points** (High for the original's behaviour): for each missing locator the original prints two debug messages (the second says the behaviour will probably fail) and stores an empty point. Arena movement then reads the current point's position through that empty reference, which has no defined result. A rebuild should leave the boss where it was spawned, with its facing unchanged, and still run the attacks.
- **Missing companion** (High): see Test spawn walkthrough. A rebuild should skip the lightning fire phase and Roberta's reaction mode while no companion exists.
- **Reaction interrupts attacks** (High): the reaction check can replace any state except air-flow and rapid air-flow. It stops both lightning beams and skips the interrupted handler that tick. If lightning or bouncing bolts are interrupted, their cooldown is not assigned because each cooldown is drawn only on the normal recovery or animation-complete exit. Bouncing balls spawned before the interruption keep updating and can still hit during reaction; if interruption occurs before animation event 32, no balls spawn. If reaction interrupts shock, the normal stop call is skipped; whether any visible effect persists cannot be established because its definition is absent. This is controller behavior; the intended balance effect is unknown.
- **Shock fallback does not reset hesitation** (High): entering shock from an idle decision leaves the hesitation timer at zero. When shock ends, the next tick decides immediately; if no weighted attack is chosen and the hero is still within range, shock can repeat without a fresh wait. Choosing a weighted attack follows its usual timer reset.
- **Stage sampling during attacks** (High): the stage is recalculated every tick, but attack parameters are read at different points. Hesitation and lightning wind-up are chosen on attack selection; lightning fire duration is chosen when wind-up ends. Gust speed is read during gust/run-out ticks, and the air-flow cooldown is drawn on exit using the then-current stage. Ball count is read at the animation event, ball speed at each ball spawn, and bouncing cooldown at normal animation completion using the then-current stage. Lightning cooldown is likewise drawn on normal recovery completion using the current stage. Reaction interruption can skip those cooldown draws (above).
- **Interrupted reply** (High): the reaction trigger can end the special reply before its 20 damage.
- **Debug display**: the original keeps a debug text line with the state name, HP, rounded distance and stage; only states 0 to 6 have names, which suggests the reply and reaction states were added later. Not needed for play.

## Test vectors

- Test spawn of monster type 1, variant 0: row 1, default class 9, maximum HP 240, radius 3.0, height 3.5. Type 2, variant 0: row 2, default class 10, HP 50.
- Animation-table compatibility: a companion main table with 24 entries can represent requested IDs 0–23; with only 14 entries, forwarded IDs 14 and 23 have no entry and resolve to clip 0 (SPEC-0005, Lookup miss).
- Spawn of monster type 0, variant 0: row 0, class 8 (Black Knight), never class 9.
- A level whose locators hold ids 101 to 104 only: two points are missing; the rebuild keeps the boss in place (Edge cases).
- HP 50 of 100 → hp% 50 → stage 2. HP 32 → stage 3. HP 70 after being stage 2 → stays stage 2. HP 70 after being stage 3 → stays stage 3; HP 50 after being stage 3 → stage 2.
- Stage 1, no cooldowns: r = 15 gives fidget (band 0–19). r = 30 gives air-flow (band 20–59). r = 70 gives lightning (band 60–99). r = 100 gives idle.
- Stage 1, start-up (air-flow cooldown 180 running): r = 30 gives lightning; r = 100 gives idle.
- Stage 2, air-flow cooldown running, lightning ready: roll 20 gives lightning; roll 42 gives lightning; roll 85 gives bouncing.
- Stage 2, air-flow cooldown and lightning cooldown both running: roll 50 gives bouncing; roll 70 gives idle; roll 85 gives bouncing.
- Integer draw 1 to 3 with stream value n = 5: 1 + (5 mod 3) = 3. Draw 0 to 100 with n = 100: 100. Hesitation stage 1 with n = 0: 60 ticks; n = 60: 120 ticks.
- Reply roll: r1 = 50 → idle; r1 = 49, r2 = 49 → air-flow; r1 = 49, r2 = 50 → rapid air-flow.
- Ring: current point 4, random 3 → next = (4 + 3 + 1) mod 6 = 2.
- Gust next-point update: current 4, next 2, previous 1 at the end of sub-step 1 → current 2, previous 4, next none.
- Rapid air-flow with count 2 (k = 2): cycle 1 checks count 2 (not zero), the count drops to 1; cycle 2 checks 1, the count drops to 0; cycle 3 checks 0 and the boss goes idle with the air-flow cooldown drawn. Three cycles (k + 1).
- Gust column, sub-step 1: B6 = (0, 4, 0), f = (0, 0, 1), third gust tick (c = 0.75): O = (0, 4, 0), E = (0, 4, 0.75). Sub-step 2 with E = (0, 4, 10) and B6 = (0, 4, 30): both ends move by (0, 0, 0.5) per tick; the step to sub-step 3 comes when E is within 0.5 of B6 (after 40 moves, E = (0, 4, 30) is at distance 0).
- Gust shape (hero radius 1.0, height 3.5, centre at the origin, axis up; gust radius 6.0, length 10, axis (1, 0, 0)): gust centre (0, −6.5, 0): the gust's near side line at height −0.5 crosses the hero at parameters 4 and 6 → connect. Gust centre (0, −8, 0): no line crosses and no containment → no connect. Centres 9.9 apart: bounding radii 2.016 + 7.810 = 9.826 < 9.9 → no connect.
- Gust hit: the first connect in a decision deals 20 damage; later connects before the next decision deal nothing. Knock-back with hero facing (0, 0, 1): velocity (0, 5.6, −4.2).
- Lightning aim: hero at (0, 0, 20), boss at the origin. Not guarding: A = (0, 1.5, 20). Guarding and facing (0, 0, −1): A = (0, 1.5, 18.35). Guarding but facing (1, 0, 0) (dot 0): A = (0, 1.5, 20).
- Lightning fire phase of stage 2 (120 ticks): the remaining counts seen run from 119 to 1; the latch clears at 60; bolts are tested from remaining 114. At most 2 × 10 = 20 damage. Stage 1: clears at 120 and 60, at most 30. Stage 3 (60 ticks): no clear, at most 10.
- Lightning latch: a hit lands at remaining count 61. The latch clears at remaining count 60, so a second hit can land one tick later.
- Guard: hero guarding, dot(d, f_h) = −0.9, shield word empty: amount 0 with the blocked flag, latch set, no 10 damage. Same with dot(d, f_h) = −0.5 and overlap: 10 damage. Hero not guarding and overlap: 10 damage. No overlap, beam fully extended, not guarding: nothing.
- Reflect (hero facing the boss exactly, level ground, Fog Mote 166 in the shield word, second beam already active): D = 20: dot = 23.35 / 23.483 = 0.9943 → state 7. D = 10: dot = 13.35 / 13.582 = 0.9829 → no reply, no damage, no latch change. First candidate tick: the second beam is only reset; no reply test.
- Special reply: boss HP 30 → 10 after the reply animation; HP 20 → 0 and the death method runs.
- Ball spawn: f = (0, 0, 1), r = 0.5 → direction normalize(0.5, 0, 0.5) or its mirror, depending on the handedness of f × up (45 degrees); r = −0.5 → 18.4 degrees to the other side.
- Ball bounce: from vertical velocity 1.0 with gravity 0.1 per tick, the height rises 5.5 units before the ball falls. Ground height 0: a falling ball at y = −0.9 does not bounce; at y = −1.0 it bounces.
- Ball travel: at 10/60 units per tick the travel passes 50 after about 300 ticks; it is added only on falling ticks, so the despawn tick is the first falling tick with travel above 50.
- Air-flow cooldown draw: a real u = 5.0 seconds in the stage-1 range gives trunc(300.0) = 300 ticks.
- Shock: horizontal distance 14.9 starts the shock; 15.0 does not (strict). A hero at horizontal distance 8.0 during the shock takes the 10 damage (inclusive).
- Reaction trigger, Tor: in an attack (state 13, attack sub-state 2), horizontal distance 9.9, dot 0.6: true. Distance 10.0: false. Sub-state 1: false.
- Reaction trigger, Roberta: an armed shot 12 units from the companion's pose point, moving straight at it at 1 unit per update: segment length 10 ends 2 units away, inside the radius 5 → true. The same shot 16 units away: the segment ends 6 units away → false.
- Shared routine, parallel rule (radii 1.0, full lengths 2.0, axes both (0, 1, 0), c_A at the origin): c_B = (3.5, 0, 0): bounding sum 2.83 is below 3.5, no overlap. c_B = (0, 2.5, 0): along-axis separation 2.5 is above 2.0, no overlap. c_B = (2.5, 0, 0): perpendicular distance 2.5 is above 2.0, no overlap. c_B = (1.5, 0, 0): overlap, depth −0.5, direction (−1, 0, 0). c_B = (0, 1.5, 0) (centres on the common axis): overlap, direction (0, 0, 0), depth −2.0 (SPEC-0018 degenerate case; the cap case does not apply because s = −1.5 is not below −1.6).
- Line test: cylinder radius 1.0, full length 2.0, axis (0, 1, 0), centre at the origin. Line X = (0, −5, 0), D = (0, 1, 0): two roots at t = 4 and t = 6. Line X = (2, −5, 0), D = (0, 1, 0): no roots. Line X = (0, 0, 0), D = (1, 0, 0): two roots at t = −1 and t = +1.

## Open questions

- Resolved: The lightning aim lift of 1.5 is on the vertical lane (High).
- Resolved: The 500.0 value is the beam growth limit per update of both beams; the effect objects are the gust stream, beam 1 and beam 2 (see Where it lives and Lightning) (High).
- Resolved: The reflected beam has no offset along the boss's facing: it starts at the end of beam 1 and runs along the hero's facing (Second beam (b)). No other state reads the recorded hero-position words (High).
- Resolved: 166 is the Fog Mote, read from the hero's shield word (word 1 of the hero's Mote words); the test that returns it runs only for a guarding hero facing into the beam (High).
- Resolved: A class-9 test-spawn created as an ordinary region character keeps the shared region-character method table; selecting its Cloud Beast behavior coroutine does not replace the shared take-hit method. Therefore, when the active, visible test-spawn passes the normal hit-scan gates and a positive-length hurt capsule overlaps, Tor melee or a Roberta shot dispatches to the shared method, which can subtract HP and call the death method at zero. If the shared NPC combat-state word is not agro, incoming damage is doubled before subtraction. These dispatch and damage rules are High confidence. Still uncertain: the missing rig prevents confirming that a Cloud Beast test-spawn has any positive-length hurt capsules, so an attack may not produce an eligible overlap; no runtime hit check was made.
- Resolved: The zero-damage hit is a blocked hit (guarding hero facing into the beam, no Fog Mote): the hero plays the block reaction and the latch is set (High).
- Resolved: The gust's origin is bone 6, its end is c units along the boss's facing, and the column slides toward the boss at 0.5 units per tick in sub-step 2 (High).
- Resolved: The 6.0 radius is the gust cylinder's radius; it has no other use (High).
- Resolved: The gust connect test uses the hero's body (radius word +0x08 and height word +0x0C of the hero's motion record) and a gust cylinder of length |E − O| centred between the ends; no other radius is passed by the boss (High).
- Resolved: The vector at 0x3932E0 used by the gust connect callers is the constant up vector (0, 1, 0, 0), a unit vector written once at boot and never changed. It is the axis of the hero's cylinder only; the gust's axis is its own direction, so the gust test takes the general rule (High).
- Resolved: Shape conflict: the shared overlap test is a static test between two finite solid cylinders with flat ends, not oriented boxes and not a sweep (High). SPEC-0018 (Finite-cylinder overlap) gives the full rule; it is the reference for this section.
- Resolved: The general rule, the cap case of the parallel rule (0.95 and 0.4), the depth sign and the degenerate case are given in SPEC-0018 (High).
- Resolved: The 0.99 axis threshold is a fixed value of the shared routine (High; SPEC-0018 Constants).
- Resolved: The words at 0x2DA5F0 (0.1), 0x2DA614 (128) and 0x2DA58C (6) are never loaded (Medium-High).
- Resolved: The ball's ground probe returns the ground height under the ball (ceiling 8; SPEC-0018); the ball bounces at 1.0 below it. Despawn uses the summed horizontal path length, not the distance from the boss (High).
- Resolved: Every integer draw includes both end values; real draws include the lower end and reach the upper only by rounding (High).
- Resolved: The reaction trigger branches on the hero variant (Tor, Roberta); Tor's mode uses his in-attack test; Roberta's mode uses her shot slots (High). This answers the hittable-query part of SPEC-0012 open question 10 for this boss.
- Resolved: The beam length is the beam's drawn length; 0.5 × len places the beam cylinder's centre at the middle of the beam (High).
- Resolved: There is no movement target outside the air-flow state: the boss is placed on its current point and its facing is steered toward the point's facing with the fixed weight 0.1 (High).
- Whether the unused boss-fight instance value 4 was intended for this boss remains unknown (Low). All 12 level modules that import the refill host pass values 0 to 3 or 5 to 14; none passes 4. In the host, the instance only selects the hero-refill branch, with no boss or encounter lookup, so the gap cannot identify a Cloud Beast fight. The disc has no placed monster type 1 or 2 records and no module setup for them. A conditional row-overflow path to class 9 exists in code, but shipped play does not establish the required target, hit and later class re-selection (SPEC-0029, Run-time variant re-pick; SPEC-0038 Behavior 43). This does not establish whether an earlier development build planned the fight.




