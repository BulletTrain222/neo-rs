# SPEC-0038: Common enemy behavior (behavior class 17)

- Answers: proactive, none requested (enemy and creature AI), follow-up to SPEC-0029 section 7; R-0015 (turn speed row, notice to agro, play-to-end gate, sight points and back and side gait facing; Open question 87); R-0017 (in part: swing classes, reach and arc; point 3 answered in SPEC-0033); R-0100; R-0117 (code 5 reads L1 from the active hero's pad-slot-0 held word, not the control flag)
- Status: Draft
- Disc build: SCUS-97367 v1.00 (same fingerprint as SPEC-0001). Row values are read from the monster table (SPEC-0029 section 4).
- Confidence: per section. Think loop, pre-step, class states and setter (Behaviors 1, 2 and 5): High. Notice check and alarm groups (Behavior 3) and the band factors by hero action state (Behavior 18): High. Normal and counter-attack choice (Behaviors 7 and 14): High, including the hero recover window of trigger codes 1 to 3. Animation table (Layout section 6): High for the table identity, the sizes, the table check of the slot and fidget ids and the presence counts. Agro machine (Behavior 44): Medium-High overall; High for the writers of the outer sub-state and the gates of the defence reactions. Attack sequencer (Behavior 13): Medium-High for the sixteen states and their transitions; High for the spawn rule of state 2 and the four projectile spawners (aim, lifetimes, ballistic launch, lobbed summon). Strike resolution: Medium; knockback Medium-Low. Grab-and-throw strike (Behavior 15): Medium. Attack manager and wait in line (Behavior 45): Medium. Agro tail and leash (Behavior 23) and the distance-hide gate (Behavior 24): High. States 8 and 14 (Behaviors 38 and 40): High for the entry and the emerge steps; Medium-High for the state 14 phases. Death and reanimation (Behaviors 34 to 36): High for the branches, the countdowns, the rise steps, the flyer grounding and the corpse sink. The drop step (Behavior 34, step 6): High for the first-death-tick trigger, the direct-item branch and the ordinal rules; Medium for the coin source (Open question 79). The take-hit hook (Behavior 37): High for the order, the state changes and the hit-count threshold; Medium for the damage doubling. Map-search attack (Behavior 42) and its spell sub-types (Behavior 43): High for the fields, the flow, the spell speed, aim, lifetime and contact effects. Spider release (base actor state 1 with class state 19), death and the inactive state 16 from the nest path (Behavior 12): High for the state steps and the despawn rule; Medium for the idle-variant chooser. Counter-attack row fields (Layout section 5): High for the disc counts and their meaning. Behaviors 19, 21, 22, 25 and 26: Medium or Low as marked; the value tables are High for their lookups and have no known meaning. Respawn and kill memory (Behavior 47): High for the kill bit, the bind exclusion, the reanimation bypass and the release rule; Medium-High for the save link. Steal rows and the absence of an area multiplier (Behavior 46): High, confirmed on disc. Poison Spyder example (Test vectors): High for the row values and placements; Medium-High for the tick-by-tick vectors. Routines of other classes kept in this spec (Behaviors 10b, 16, 17 and 27 to 33): High for their owner classes; Medium to Medium-Low for their steps, as marked in each section. Movement (Behavior 48): High for the root-motion mechanism, animation ids, and same-update effect of the attack speed words on clip rate; Medium-High for which id each state sets and the following motion tick's use of the updated root step. Think-loop stepping inside the update hook (Behavior 1): High.

## Summary

Every common enemy (skeletons, spyders, slorgs, minions, spirits, tentacles, sludgies, werelupes and similar) runs one shared think routine, behavior class 17. The routine is generic: each enemy's row in the monster table (SPEC-0029 section 4) and its kind-3 spawn record (SPEC-0006) supply its numbers. In one tick the enemy runs a pre-step (pause, awareness, alert, retarget, alarm-group check, map search, then a switch on its class state; Behavior 2), then the handler of its class state (Behavior 5), and then waits one tick (1/60 s). In agro (class state 19) the pre-step runs the agro machine (Behavior 44), which owns all combat: two attack choosers, a sixteen-state attack sequencer, four defence reactions, hit reactions, a search after lost sight, confusion, a wait-in-line state under the attack manager, and the leash. The normal attack choice (Behavior 7) and the counter-attack choice (Behavior 14) both run every agro tick, the normal choice first, and both start the same attack sequencer (agro outer sub-state 2, Behavior 13). Class state 18 is used only by the map-search attack (Behaviors 42 and 43); no attack slot uses it.

The routine has these parts. (1) Notice: a hostile enemy rolls each awareness cycle whether it notices the hero, using a distance band scaled by the hero's action state (Behavior 18), a line-of-sight test, a view cone and a percent chance; a hero closer than the band's near edge is noticed without the chance roll (Behavior 3). Enemies of one alarm group alert each other (Behavior 3). (2) Movement archetype: the row's Approach Routine selects the rush (1), keep-distance (3) or strafe (4) outer sub-state of the agro machine, with distance bands drawn at set-up and after each normal attack choice (Behaviors 4 and 44); the enemy travels only by the root motion of its locomotion animations (Behavior 48). (3) Attacks: up to four attack slots. The normal choice tests slots 4, 3, 2, 1 and takes the first whose band, facing cone, chance, cooldown, spawn limit, airborne rule and charge rule pass. The counter-attack choice walks slots 1 to 4 and reacts to hero attacks, dodges, blocks, recoveries and hit reactions by the slot's counter code and chance. The chosen slot runs through the sequencer, which fires missiles, resolves melee strikes, runs charges and repeated strikes, or runs the grab-and-throw (Behavior 15). (4) Side states: stunned (state 4), emerge and hide (state 8), retreat home and hide (state 14), map search (states 17 and 18), death and reanimation, and the take-hit hook (Behaviors 34 to 43).

A common enemy that dies is set to base state 16 and removed when its death countdown ends, except the three reanimating rows (224, 226 and 227), which rise again at full health (Behavior 34 to 36). A hit on a non-agro enemy sends it to agro, doubles the damage and starts a hit reaction (Behavior 37). The nest's spiders use the inactive or death state 16 and are revived with base actor state 1 and class state 19, the agro state (Behavior 12). A killed common enemy does not come back: each world remembers its killed spawn records and never binds them again, except the three reanimating rows (Behavior 47).

Several routines described in this spec belong to other behavior classes, as the class routine table shows (SPEC-0029 Layout section 3): the group leader handover (Behavior 10b, class 6), the chase-to-point controller, the approach and attack bands and the wait-and-approach step (Behaviors 16, 29 and 30, class 7, Whinny), the approach controller (Behavior 17, class 21, Miamouse), the main loop with states 0 to 6 and its approach states 4 and 5 (Behaviors 27 and 33, class 34), the idle coroutine with its 400 to 800 tick period (Behavior 28, class 36, Target Dummy), the charge and wait coroutine (Behavior 31, class 35) and the patrol list walk (Behavior 32, class 29). They are kept at the end of the Behavior section, under Routines of other classes, for reference; a class-17 enemy does not run them. All times are seconds unless marked ticks. Ticks = 60 per second (SPEC-0007). Distances are world units. Percent values are integers from 1 to 100.

## Where it lives on the disc

- The class id for a common enemy is the row's Default Behavior field (0x36) and equals 17 for every common enemy. Row values are in GAMEFILE.DAT resource 530 (SPEC-0029 section 4); resource 531 is not used by this class (see Open questions).
- The think routine is not a data table; it is a fixed routine. Boot data words it reads: the band factors F1 at 0x2DD5A4 to 0x2DD5BC and F2 at 0x2DD5C0 to 0x2DD5D8 (f32, seven each; Behavior 18) and the confusion aim blend at 0x2DD540 (f32, 0.05; Behavior 44). The word at 0x2DD54C (boot value -1, s32) is not a tuning value: it is one run-time countdown of the grab-and-throw strike, shared by all enemies (Behavior 15). Run-time memory, not on disc: the attack-manager list at 0x35B320 (Behavior 45) and the frame-rate word (60, SPEC-0007). Other numbers of the routine are fixed constants, listed where they are used.
- The counter-attack choice (Behavior 14) and the grab-and-throw (Behavior 15) read the same monster row as the rest of the class (resource 530). They do not read resource 531.
- The class-7 chase-to-point controller (Behavior 16) and the class-21 approach controller (Behavior 17), kept in this spec under Routines of other classes, use these run-time words, all data addresses: a point pool with a 4-byte owner id per index at 0x2DB268 (boot value -1) and a 4-byte claimed flag per index at 0x2DB278; a point table at 0x36EC60 with 16-byte entries (not in the boot image, so its values are runtime data and are not given here); a frame-rate word at 0x368FA0 (not in the boot image; written once at start-up as 60 ticks per second from the boot rate table at data address 0x30C598, SPEC-0007, Rate selection); and the current-world word at 0x35C834, whose navmesh list the Miamouse flee uses for its far centroid picks. The word at 0x2DB268 is also read by SPEC-0029 as the Whinny entrance id (SPEC-0029 open question 69).

The routines of Behaviors 27 to 33 (classes 34, 36, 7, 35 and 29; not class 17) read two boot words: 0x2DB250 (s32, boot value 1; the approach controller tests it against 1) and 0x2DB1D8 (f32 0.5; the selector uses it as a factor). They also read two run-time words that are outside the file image and are not on disc: the tick-rate word at 0x368FA0 (written once at start-up as 60 from the boot rate table at 0x30C598; SPEC-0007, Rate selection) and the state of the shared random generator at 0x39BE50 (SPEC-0008). The hero record is reached through the run-time current-hero pointer at 0x35C7D4 (High; Open question 69), and the shared object pool is at 0x35C7B0. The run-time word at 0x35C7B4 is the camera pointer (SPEC-0033), not a hero pointer: the routines here that read it use the camera (the aim scatter of Behavior 13 and the class-34 leave test of Behavior 33).

The band factors (Behavior 18) are the data words above. The animation-code tables and the attack and emerge codes (Behaviors 19 and 20) and the spacing predicate (Behavior 22) use fixed constants. The value tables (Behavior 26) are read at data addresses 0x2DA6A0 and 0x2DA8F8 and their neighbours. The spacing predicate reads a shared object pool at 0x35C7B0, and the distance-hide gate draws from the shared random generator, whose state is at 0x39BE50 (Behavior 24).

## Layout

### 1. Per-enemy runtime fields (conceptual)

| Field | Type | Meaning |
| --- | --- | --- |
| State | int | Current state id (table in Behavior 5). |
| Previous state | int | State before the last change; restored when the pause state (20) ends. |
| Default state | int | Idle state chosen by the setter when no other state applies. |
| Target index | int, -1 = none | Index of the enemy's target in its region's character list. |
| Awareness countdown | tick count | Counts the awareness cycle (Behavior 2). |
| Retarget countdown | tick count, -1 = disabled | When it reaches 0 the enemy re-selects its target. |
| Alarm-group check countdown | tick count | Object offset 0xC88. At 0 it is reloaded with a random 15 to 45 ticks and the alarm-group check runs (Behavior 3). |
| Attack slot cooldowns | 4 tick counts | One per attack slot. Counted down by one per tick by the normal attack choice, only while the hero is attackable (Behavior 7). Set to 0 on entering agro. |
| Attack recovery countdown | tick count | Object offset 0xB18. Set by the sequencer's end path; counted down each agro tick; blocks the normal attack choice while positive (Behaviors 7 and 13). |
| Selected attack slot | int, -1 = none | Slot chosen by the attack selection (0..3). Object offset 0xA24. The per-slot row fields of Layout section 2 are read with this index (Behavior 43). |
| Desired distance | float, units | Object offset 0xE40. Drawn at creation and after each normal attack choice (Behavior 4). |
| Retreat distance | float, units | Object offset 0xE44. Drawn with the desired distance. |
| Charge distance per slot | 4 floats, units | Drawn with the desired distance, from the selected slot's charge range (Edge cases). |
| Search countdown | tick count | Time spent searching after losing the player. |
| Last seen point | vec3 | Where the player was last noticed (search). |
| Socialize countdowns | 2 tick counts | Talk and quiet timers for group alert (Behavior 10). |
| Alert flag | bool | One-shot "something alerted this enemy" flag, consumed on read. |
| Base actor state | s32 | Object offset 0x144. This is the base state word, not an enemy kind. The death hook sets it to 16 for non-reanimating enemies (Behavior 34 and 36). Creation writes 42, the default value. The agro machine sets 20 while the enemy blocks and 1 when a defence ends (Behavior 44). |
| Second data block | pointer | Object offset 0xA44. Points to the enemy's kind-3 spawn record (SPEC-0006), which holds the drop placement ordinal at data offset 0xB0 (Behavior 34, step 6). The 32-bit word at block offset +0xC8 (decimal 200) is a shared value, not the variant. Its main use is as the end of a random tick range whose start is the word at +0xC4 (Medium). Behavior 20(b), Behavior 40 and the emerge state also read it as a selector: values 0 to 4 pick a code or animation, and any other value takes the default (Open question 80). On disc it is 0 in 2,627 of 3,036 kind-3 records, 1 in 41, 2 in 10, 3 and 4 in none, and tick-scale values from 5 to 1,800 in 352. The monster row variant is the u16 at data offset 0xB2 (SPEC-0006), a different field. Other fields read by class 17: the u16 at 0x84 (default state, Behavior 5), the byte at 0xB7 (alarm group, 0xFF = none; Behavior 3), the f32 at 0xC0 (emerge distance, Behavior 40) and the word at 0xC4 (zero starts a camera and sound cue in the alarm-group check). |
| Leash counter | tick count | Object offset 0xB50. Ticks in agro with no route to the hero while the enemy's walk waypoint is at least 15.0 units from the hero (Behavior 23). |
| Agro machine words | s32 and float | Outer sub-state 0xAF8, inner sub-state 0xB30, sequencer state 0xB2C, hold timer 0xB34, strafe lock 0xCB0, route flag 0xAFC, defence hold 0xAB8, hit-reaction kind 0xB28, stun countdown 0xBF0, stun type 0xC60, search counter 0xB54, melee-swing latch 0xB88 (object offsets; Behavior 44). |
| Pre-step words | s32 | Awareness countdown 0xBA0, retarget countdown 0xBF4, alert flag 0xABC; step words 0xB00, 0xB04 and 0xB08, reset by the setter (Behavior 5). |
| Reanimation countdown | tick count, -1 = idle | Object offset 0xD14. Set by the death hook for reanimating rows and counted down by the reanimation state (Behavior 34 and 35). |
| Death disappear countdown | tick count | Object offset 0xD24. Set from the row's death disappear timer on death and counted down by the dead-state update (Behavior 34 and 36). |

### 2. Monster row fields used by this class (resource 530, row offsets)

Row offsets are in the row of SPEC-0029 section 4. Kinds: f32 float, u16, s32.

| Offset | Kind | Field | Use |
| --- | --- | --- | --- |
| 0x28, 0x2C | f32 | Radius, Height | Body size (not used by the think routine). |
| 0x30 | u16 | Hit points | Starting and maximum health. |
| 0x36 | u16 | Default behavior | Class id (17 for common enemies). |
| 0x38 | u16 | Approach routine | Movement archetype: 1 rush, 3 keep distance, 4 strafe (agro outer sub-states 1, 3 and 4, Behavior 44); other values use the default band and never reach an attack by the normal choice (Edge cases). Only 1, 3 and 4 occur on disc. |
| 0x3A | u16 | Difficulty value | Weight of the enemy in the attack-manager budget (Behavior 45). |
| 0x3C | u16 | Alignment | Owner alignment for enemy projectiles: the element of every projectile the enemy fires (Behaviors 13 and 43). |
| 0x3E | u16 | Hostile flag | 0 = never notices the player. |
| 0x4C, 0x50 | f32 | Socialize social max, min | Talk countdown range (ticks). |
| 0x54, 0x58 | f32 | Socialize quiet max, min | Quiet countdown range (ticks). |
| 0x5C, 0x60 | f32 | Search timer max, min | Search countdown range (ticks). |
| 0x64, 0x68 | f32 | Attack timer max, min | Recovery range after an attack (ticks). |
| 0x6C, 0x70 | f32 | Schema labels: Flee Timer Max, Flee Timer Min | Runtime: the timed-approach duration range (ticks), with maximum at +0x6C and minimum at +0x70. The chosen float is converted to an integer countdown. The row label is retained; no authoring rationale is established. (Open question 2 resolved.) |
| 0x74, 0x78 | f32 | Combat timer max, min | Combat timer range (ticks) drawn when entering the agro state. |
| 0x82 | u16 | Display in default | Non-zero: the setter's default case gives state 0 (Behavior 5). Zero (an invisible kind): the setter for state 2 gives state 21 instead unless invisibility is disabled for the enemy, the enemy takes part in the alarm-group gate of the notice check (Behavior 3), and it is excluded from hits in some states (Edge cases, Invisible kinds). |
| 0x84, 0x88 | f32 | Ground turn speed, flying turn speed | Aim turn speed of the agro hold sub-states: 0x84 on the ground, 0x88 for flyers (Behavior 44). The value is a fraction from 0 to 1 (the monster-row range) and is used as the per-tick blend weight of the aim turn, clamped to that range (Medium: the use as a weight, not a rate or an angle, is inferred from the range and the aim routine). Not read for movement: while an enemy travels, its turning is the steering blend only (Behavior 48). |
| 0xA0 | u16 | Distance hide flag | Non-zero enables the distance-hide gate (Behavior 24) and makes state 8 hide in place (Behavior 40). |
| 0xA4, 0xA8 | f32 | Distance hide distance, chance | Distance and percent of the distance-hide gate (Behavior 24). |
| 0x80 | u16 | Death disappear timer | Ticks a dead enemy stays before it is removed (Behavior 34 and 36). Values on the 109 class-17 rows are listed in Behavior 34. |
| 0x98, 0x9C | f32 | Ground timer max, min | Retarget countdown drawn after a death, in decimal ticks (Behavior 34, step 2). |
| 0xAC | int | Flying hit | Hit-reaction kind chosen on a hit to a flyer (Behavior 37). |
| 0xE0 | int | Missile defend time | Defence hold time in ticks, set when a defence reaction starts; also the look-ahead of the missile defence in ticks (Behavior 44). |
| 0xE4, 0xE8 | f32 | Missile dodge, missile block chance | Percent chances of the missile defence (Behavior 44). |
| 0xEC, 0xF0, 0xF4 | f32 | Melee block chance for quick, normal and heavy swings | Behavior 44, melee defence. |
| 0xF8, 0xFC, 0x100 | f32 | Melee dodge chance for quick, normal and heavy swings | Behavior 44, melee defence. |
| 0x104, 0x108 | f32 | Defence timer min (0x104) and max (0x108) | Defence hold drawn when any defence reaction ends (Behavior 44), in ticks. The draw is a whole number from low to high, both included; a reversed pair is handled by the draw (row 222 has 0x104 above 0x108). |
| 0x11C | int | Map search flag | Non-zero enables the map search (Behavior 9). Set on 15 rows. |
| 0x134 | int | Anger level | Non-zero: each hit raises the anger counter (Behavior 37). |
| 0x15C | int | Reanimation flag | Non-zero: a death starts a reanimation countdown (Behavior 34 and 35). Set on rows 224, 226 and 227. |
| 0x160, 0x164 | int | Reanimation min, max | Countdown range in ticks drawn at death for reanimating rows (Behavior 34). |
| 0x138 to 0x158 | int, f32 | Ghost flag (0x138), solid and ghost phase ranges, opacity range, reveal range | Nonzero ghost flag: the enemy alternates between a solid and a ghost phase, is not lock-on targetable in the ghost phase and absorbs hits there (SPEC-0029 Behavior 33, Ghosts; the rule is shared by every class). Set on three class-17 rows: 143 (Death Knight), 215 (Minion Wizard variant) and 222 (Restless Spirit). |
| 0x94 | s32 | Can fly | 1: a flyer while its retarget countdown is at most 0 (airborne); 2: always a flyer; 0: never. Chooses flying animations and the flying search mode. |
| 0xC0 | f32 | View angle | Cosine of the half-angle of the view cone (Behavior 3). |
| 0xC4 | s32 | Awareness | Percent chance per awareness cycle. |
| 0xC8 | f32 | Awareness timer | Awareness cycle length (ticks). |
| 0xCC, 0xD0 | f32 | Sense max chance, min chance | Percent chance at the far and near edge of the band. |
| 0xD4, 0xD8 | f32 | Sense max range, min range | Notice band (world units). |
| 0xDC | s32 | Always has line of sight | Non-zero skips the line-of-sight test. |
| 0x120..0x130 | | Map search distance, chance, timer min/max, type | Map search (Behavior 9). |
| 0x168 | f32 | Greedy hostile range | Map search distance test (Behavior 9). |
| 0x16C + 2k | u16 | Attack k type | 0 = melee, 1 = missile, 0xFFFF = slot unused (k = 0..3). |
| 0x17C + 4k | f32 | Attack k trigger range max | Upper bound of the attack band (units). |
| 0x18C + 4k | f32 | Attack k trigger range min | Lower bound of the attack band (units). The speed-ramp spawner (flight mode 1) reads the same pair as its distance band: bmin = 0x18C, bmax = 0x17C (Behavior 13). Both readings hold. |
| 0x19C + 4k | f32 | Attack k trigger chance | Percent chance to use the slot when it is otherwise eligible. |
| 0x1AC + 4k | f32 | Attack k trigger special | Truncated to an integer: 1 = slot usable only while airborne, 2 = only while not airborne, any other value = no limit (Behavior 7). |
| 0x174 + 2k | u16 | Attack k orient flag | Non-zero: the sequencer aims at the hero at weight 1.0 in states 1 to 3 (Behavior 13). |
| 0x1BC, 0x1CC, 0x1DC + 4k | f32 | Attack k windup, attack, recovery speed | Directly stored by the sequencer to P: wind-up in states 1 and 6, strike in state 2, recovery in state 5 (Behavior 13). The class coroutine runs after reset/status processing and before same-update clip re-time (Behavior 1), so the selected row value sets that update's clip rate and replaces P accumulated from earlier status factors. The animation pass evaluates at that rate; the updated root step is consumed by the following motion-record pass (Behavior 48). |
| 0x1EC + 4k | f32 | Attack k melee arc | Cosine threshold for facing the player when the slot is used. |
| 0x1FC + 4k | f32 | Attack k melee range | Melee reach (units). The sequencer uses the same value as the slot's attack range (Behavior 13, state 0). |
| 0x20C, 0x21C + 4k | f32 | Attack k delay max, min | Cooldown after the slot is chosen (ticks). |
| 0x24C + 4k | s32 | Attack k charge type | -1 = no charge (sequencer starts in state 0); 0 to 3 = charge kinds of sequencer states 7 and 8 (Behavior 13). |
| 0x25C, 0x26C + 4k | f32 | Attack k charge distance max, min | Charge distance range (units). |
| 0x27C, 0x28C + 4k | f32 | Attack k stun range max, min | Stun countdown range of a blocked charge of kind 2 (Behavior 13, state 9). |
| 0x29C + 4k | s32 | Attack k stun type | Stored with that stun (Behavior 44, hit reactions). |
| 0x2AC + 4k | s32 | Attack k blocked-charge kind | 0 to 2 select the blocked-charge rule of sequencer state 9; other values give hit-reaction kind 3 (Behavior 13). |
| 0x2BC + 4k | s32 | Attack k missile flight | Flight mode: 0 straight aim (kind 0), 1 speed ramp with a ballistic arc (kind 1), 2 homing (kind 17) (Behavior 13). On disc, over all 259 rows: 0 in 844 used slots, 1 in 9, 2 in 1 (confirmed on resource 530). |
| 0x2CC, 0x2FC + 4k | f32 | Attack k missile velocity, range | Missile launch speed in units per tick and reach in units. Straight lifetime trunc(range / speed) ticks; the speed is also the divisor of the ballistic flight time (Behavior 13). |
| 0x2EC + 4k | f32 | Attack k projectile radius | Sphere radius of the projectile in the hit test (Behavior 13). |
| 0x30C + 4k | f32 | Attack k aim inaccuracy | Upper bound of the random aim scatter, units (Behavior 13). |
| 0x31C, 0x32C + 4k | f32 | Attack k projectile spin rate, roll | Spin in radians per tick and launch roll (Behavior 13). |
| 0x2DC + 4k | f32 | Attack k end speed | End speed of the speed ramp (flight mode 1), units per tick; the speed at the near edge of the band. Values 0 to 1 on disc. |
| 0x33C + 4k | s32 | Attack k spawn count | Projectiles fired by one spawn loop of the sequencer (Behavior 13, state 2); a count of 0 still fires one. Values 0 to 10 on disc; 1 on 813 of the 854 used slots, counted over all 259 rows (confirmed). |
| 0x22C + 4k | s32 | Attack k counter attack trigger (Layout section 5). 1 to 6 are trigger codes; 0 and -1 never trigger. | Counter-attack choice (Behavior 14). Confirmed on disc. |
| 0x23C + 4k | f32 | Attack k counter attack chance, a percent 0 to 100 (Layout section 5). | Counter-attack choice (Behavior 14). Confirmed on disc. |
| 0x34C + 4k | f32 | Attack k damage | Damage value. |
| 0x35C + 4k | f32 | Attack k knockback | Knockback value. |
| 0x3AC + 4k | u32 | Attack k damage special (flags) | Bit 0x02 marks a summoning slot: it limits the slot by the live summon count (Behavior 7, step 5e) and, on a missile slot, selects the lobbed summon (Behavior 13); both readings hold. Bit 0x01 makes the melee strike unblockable. Bit 0x04 enables the steal (SPEC-0029 Behaviors 11a and 11b). Bit 0x08 replaces the strike with the grab-and-throw (Behavior 15). Bit 0x10 sets the projectile's ignore-world flag (flight modes 0 and 2). Bit 0x40 replaces the trigger chance with the anger rule (Behavior 7). Bit 0x80 adds a Mote drain to a melee hit. Bit 0x20 changes the victim-reaction route for repeated strikes in sequencer state 14 (Behavior 13): when the victim-reaction eligibility predicate does not claim the hit, a clear bit requests base state 17 with a selected animation, while a set bit calls the victim's additional attack-reaction method. That method can reset the hero attack sub-state to 13 when its own checks pass (SPEC-0036). The design-facing name of this flag remains unknown. Bit 0x02 is set on 3 of the 854 used slots, counted over all 259 rows (row 128 slot 0, row 211 slot 0, row 221 slot 3; confirmed). |
| 0x3CC + 2k | u16 | Attack k spawn key | Character kind key that the slot summons by key (SPEC-0029 Behavior 31); 0xFFFF or 0 = none. Set on five slots on disc (rows 128, 211, 215 and 221; SPEC-0029 Layout section 4). A lobbed summon carries it and spawns it where it lands (Behavior 13). |
| 0x3D4 + 2k | u16 | Attack k spawn entity count | Maximum number of live enemies this enemy may have spawned for the slot. |
| 0x43C + 4k | s32 | Attack k projectile type | Visual kind of the missile spawners' projectile (Behavior 13); not used by the spell sub-types (Behavior 43), which use visual kind 23. |
| 0x36C, 0x37C + 4k | s32, f32 | Attack k projectile status id, chance | Status applied on a hit and its percent chance (Behavior 13). |
| 0x38C, 0x39C + 4k | f32 | Attack k projectile status value range | Drawn uniformly, divided by 60 and truncated to give the status value (Behavior 13). |
| 0x46C + 2k + 8j | u16 | Attack k projectile effect id j (j = 0 to 5) | Effect ids of the projectile (Behavior 13). |
| 0x44C, 0x45C + 4k | f32 | Attack k projectile area duration, radius | Area mode of a flight-mode-1 projectile after landing: duration in ticks (truncated) and radius in units (SPEC-0029 Behavior 9). |
| 0x41C + 4k | f32 | Attack k throw speed | Speed of the grab-and-throw (Behavior 15). |
| 0x42C + 4k | f32 | Attack k Mote drain | Amount drained on a melee hit with flag 0x80 (Behavior 13). |
| 0x49C + 4k | s32 | Attack k loopable | Non-zero: the normal choice starts the sequencer in state 13, a repeated strike (Behavior 13). |
| 0x4AC, 0x4BC + 4k | s32 | Attack k loop interval range | Ticks between the direct damage ticks of sequencer state 14. |
| 0x4CC + 4k | f32 | Attack k loop hold distance | Sequencer state 14 ends when the hero is farther than this. |
| 0x4DC, 0x4E0 | f32 | Agro rush melee distance min, rush distance max | Rush band (approach 1). |
| 0x4E4, 0x4E8 | f32 | Agro distance desired min, max | Desired band (approach 3). |
| 0x4EC, 0x4F0 | f32 | Agro distance retreat min, max | Retreat band (approach 3). |
| 0x4F4, 0x4F8 | f32 | Agro strafe desired min, max | Strafe band (approach 4). |
| 0x4FC | f32 | Agro strafe erratic | Never read by class 17. |
| 0x500 | f32 | Agro strafe backbiter | Chance scale when the player is behind (approach 4). |

### 3. Notice state (per enemy)

| Field | Type | Meaning |
| --- | --- | --- |
| Awareness countdown | tick count | Starts at the row's awareness timer. |

### 4. Monster values used as fixed constants

The routine also uses these fixed constants (not stored in the row): 1 to 100 for percent rolls (0 to 100 for the counter-attack and distance-hide rolls); 0.2 to 0.8 for the band fraction (random integer 20 to 80, divided by 100); 0.1, 0.25 and 1.0 for aim blend weights; 5.0 units for the approach step point; 10.0 units for the melee-by-distance test in the attack choice; 1.0 unit for the waypoint arrival test; 2.0 units added to the back-off distances; 60 to 180 ticks for the strafe hold and 120 ticks for the strafe lock; 0.95 for the sequencer facing test; 120 to 600 ticks for idle variant holds, drawn as integers in that range (2 to 10 s); 15 to 45 ticks for the alarm-group check countdown (0.25 to 0.75 s); 15.0 units and 180 ticks for the leash; 120 ticks for the search give-up.

### 5. Counter-attack fields (used by the counter-attack choice, Behavior 14)

Each attack slot k (0 to 3) has two row fields, in resource 530 at the row offsets of Layout section 2. The schema names them Counter Attack Trigger and Counter Attack Chance. They are read only when the slot is in use (its row type at 0x16C is not 0xFFFF).

| Field | Type | Meaning | Disc check (used slots only; counted over all 259 rows, 854 used slots in all) |
| --- | --- | --- | --- |
| 0x22C + 4k | s32 | Counter attack trigger code. Codes 1 to 6 select a trigger rule of Behavior 14 step 1; 0, -1 and any other value never trigger. | -1 on 644 slots, 0 on 98, 6 on 50, 5 on 32, 2 on 20, 1 on 6, 3 on 3, 4 on 1. |
| 0x23C + 4k | f32 | Counter attack chance, a percent from 0 to 100. Compared with a roll in Behavior 14 step 3. | 0 on 726 slots. Nonzero values are 5, 10, 20, 25, 30, 40, 50, 60, 75, 80 and 100; 128 slots have a nonzero chance, and 112 of them have a code from 1 to 6. |

### 6. Animation table (High)

- Each enemy plays its animations through the main table of its own character record: the kind-2 table named by the record's main table index (kind-3 data + 0x9E; SPEC-0006 and SPEC-0005, Choosing an animation), bound to the character at creation. There is no shared class-17 table.
- Size: N = 166 on 1,653 of the 1,654 class-17 records, and 138 on one (resource 1273, monster type 9). Code 137 (map-search attack) is in range for all of them; an id at or above N (for example the flyer emerge code 165 on the 138-entry table) is a lookup miss.
- Table-checked picks: the slot attack ids 33 + 5k + p (slot k from 0 to 3; phase p = 0 wind-up, 1 strike, 2 follow-through, 3 blocked, 4 recovery) and the idle fidgets 115 to 119 (142 while flying) are tested against the table before they are played, and a missing one is skipped (Behaviors 8, 13 and 20). Every other request of an id with no clip (no entry, or word A clip 0xFFF) starts clip 0 of the main set (SPEC-0005, Lookup miss).
- Presence on disc, as the number of the 1,654 class-17 tables that have a clip for the id: 0: 1,535; 1: 1,536; 2: 1,461; 10: 1,141; 33: 233; 34: 1,603; 35: 566; 36: 1,109; 37: 67; 38: 175; 39: 1,419; 40: 548; 41: 1,040; 43: 448; 44: 1,052; 48: 71; 49: 557; 57: 1,572; 58: 1,572; 85: 44; 115: 984; 116: 742; 117: 578; 118: 365; 119: 365; 137: 336; 142: 533; 153: 559; 165: 416. So on most tables the slot-0 wind-up (33) and the slot-0 recovery (37) are skipped.

## Behavior

### 1. Think loop (High)

1. Start. The enemy searches its target (Behavior 6). If its class state is neither 19 nor 21, it enters state 8 when the spawn record's default-state word (u16 at data offset 0x84) is 8, and otherwise runs the setter with 0 (Behavior 5).
2. Each tick: if the target index is -1, a target is searched first; then the pre-step runs (Behavior 2), then the handler of the current class state (Behavior 5), and the loop yields one tick. The loop has handlers for states 0 to 23 except state 4 (stunned), which only the pre-step leaves.
3. Animation: in the idle states the idle picker (Behavior 8) chooses the idle or locomotion animation.

**When the think loop is stepped (High).** The think loop is the enemy's behaviour coroutine. There is no separate AI scheduler: the coroutine is stepped inside the enemy's own update hook, which the region character loop of the world update calls once per tick for each active character in an active zone (SPEC-0026, main loop step 8.1; SPEC-0040). The update hook works in this order:
1. It resets the per-tick speed scale P (+0x8B8) to 1.0.
2. It runs the status-effect tick, which applies the status factors.
3. It switches on the base actor state (0x144). In the default arm and in base states 17 and 31, the behaviour manager picks the request and steps the coroutine once. In base states 16, 25, 33, 36, 42, 53 and 55 the coroutine is not stepped. Base state 42 returns at once, so its clip is not re-timed either.
4. After the switch, the playing clip is re-timed from R = speed scale (+0x3D4) x P, and the remaining tail steps run.

So a write to P made by the coroutine lands between the reset and the re-time of the same tick, and a clip started in that tick's animation pass (which follows the update hook) also reads P after the write (Medium-High). The coroutine is not stepped at all while the world is frozen (game-state code 9; SPEC-0026, Pause and freezing screens) or while the enemy's zone is inactive.

Random draws (SPEC-0008): int[a, b] is the integer helper, both ends included, ends in either order; float[a, b) is the float helper. A float draw stored as a whole count is truncated toward zero. Row floats passed to the integer helper are truncated first. All counts are ticks (60 per second).

### 2. Pre-step, in this order (High)

1. Pause. If the global AI pause word (word +0x20 of the game-flow record) is nonzero and the state is neither 16 nor 8, the enemy enters state 20.
2. Awareness countdown (object offset 0xBA0). If it is negative, reload it with int(row 0xC8), the awareness timer in ticks. Otherwise subtract one. So a cycle lasts the timer value plus two ticks, and the countdown is zero for exactly one tick per cycle; the notice check (Behavior 3) can pass only on that tick.
3. Alert flag (0xABC). If it is set (it is consumed) and the state is not 19, 1, 4 or 16: a non-hostile enemy (row 0x3E is 0) moves to the timed approach (state 1); a hostile enemy moves to agro (state 19).
4. Retarget countdown (0xBF4). When it is 0, the pre-step consults the Action-19 gate (Behavior 21). Retargeting is blocked only while class state 19 is active and either the agro outer sub-state is 2 with sequencer state 0 to 3, or the object word at 0xE58 is nonzero. Otherwise the navmesh is picked again (Behavior 6) and the state word is set to 22 directly, without the setter entry work. Then the countdown is decremented. At -1 nothing happens. The object word 0xE58 is an attack-active latch set by the Black Knight slow- and angry-swing handlers and cleared by actor setup and the generic character tick. No class-17 writer was found, so the common-enemy sequencer does not set this fallback in the recovered path; why the shared predicate includes it remains open (Open question 54).
5. Alarm-group check countdown (0xC88). At 0 it is reloaded with int[15, 45] and the alarm-group check runs (Behavior 3). Otherwise subtract one.
6. Map search per tick (Behavior 9).
7. Switch on the class state:
   - 0: run the setter with the spawn record's default-state word (u16 at 0x84).
   - 3 and 9: if the notice check passes, move to state 2 (alerted search).
   - 4 (stunned): when status 0 (stun, SPEC-0040) is no longer active, move to agro (19).
   - 5, 6 and 7: if the notice check passes, move to state 2; otherwise, if the socialize alert (Behavior 10) fires, move to state 3.
   - 8: the emerge step (Behavior 40).
   - 10: when the current animation has finished, run the setter with 0.
   - 17: the map-search arrival check (Behavior 39).
   - 19: the agro machine (Behavior 44).
   - 20: when the pause word is clear, return to the previous state.

### 3. Notice check and alarm groups (High)

The notice check returns "noticed" or "not noticed". In order:

1. Gate. An enemy whose row Display in default (0x82) is 0 fails while another enemy, also with row 0x82 = 0 and in state 19 or 2, has a different alarm group byte (spawn record byte 0xB7), when neither byte is 0xFF. The enemy must be hostile (row 0x3E nonzero) and its awareness countdown must be 0.
2. Roll A: int[1, 100] must be at most the Awareness value (row 0xC4). Otherwise: not noticed.
3. Band: near = Sense min range (0xD8) x F1 and far = Sense max range (0xD4) x F1, with F1 chosen by the hero's action state (Behavior 18). If the socialize talk countdown is positive, both are halved.
4. Visibility check: test the sight line between the enemy's and active hero's sampled points. Each sampled point is the aim point that the character's class routine returns (the class table's method at slot 0x294, applied to the character's sub-object at +0x290; for heroes it is the position at attach point 7; Medium-High). The points are not a fixed fraction of the body height. This is a visibility test; the notice routine does not test for a navmesh route to the hero. If the sight line is blocked and Always has line of sight (0xDC) is 0, replace the band with near = Sense min range (0xD8) x F2 and far = Sense max range (0xD4) x F2 (Behavior 18). In this blocked-sight branch, a hero wearing the Tattered Cloak (item 126) is not noticed: the check returns before the distance and chance tests.
5. If d < near: noticed (no further roll). If d > far: not noticed.
6. Otherwise the chance is the linear interpolation from Sense min chance (0xD0) at Sense min range to Sense max chance (0xCC) at Sense max range: chance = 0xD0 + (0xCC - 0xD0) (d - 0xD8) / (0xD4 - 0xD8), over the unscaled band values (Edge cases). It is halved when the cosine between the enemy's facing and the direction to the hero is below View angle (0xC0), and halved again when the active hero wears the Tattered Cloak (item 126), including when the sight line is clear.
7. Roll B: int[1, 100]. Noticed when roll B is below the chance.

The notice gate's alarm-group suppression is separate from these checks: when this enemy's Display in Default field (0x82) is 0, a different alarm group on another Display in Default = 0 enemy in state 19 or 2 blocks notice, provided neither spawn record group byte is 0xFF. A passing notice result moves states 3 and 9 to state 2; states 5, 6 and 7 also move to state 2, before their fallback socialize-alert check can move them to state 3. No author-intent label for the visibility/range factors is established.

Test vector: Dumb Skeleton (Awareness 65, sense band 3 to 5, chance 50 at 3 and 1 at 5), hero in an action state not listed in Behavior 18 (F1 = 1.0), sight line clear. A hero at 2 units is noticed when roll A is at most 65 (closer than the near edge 3.0). A hero at 4 units in the cone has chance 25.5, so it is noticed when roll A passes and roll B < 25.5. A hero at 5.5 units is never noticed.

**Alarm-group check.** It runs when its countdown expires (every 15 to 45 ticks, Behavior 2, step 5). It acts only when the spawn record's alarm group byte (0xB7) is not 0xFF and a new broadcast for that group is pending; a group member broadcasts when it enters agro (setter for state 19, Behavior 5). Then, by the enemy's class state:
- 0, 3, 5, 6, 7 and 14: move to state 2 and play animation 18 (149 when either of two class tests passes; SPEC-0029 Behavior 24).
- 8: play the emerge animation (Behavior 40) and set the emerge sub-state to 2; when the emerge animation is missing, move to agro (19) instead. In this case a camera and sound cue also starts when the spawn record word at 0xC4 is 0.
- Any other state: no change.

### 4. Movement archetype and distance bands (High)

The desired distance is drawn as: desired = min + (max - min) x f, where f = (random integer in [20, 80]) / 100, with the band from the Approach routine:
- 1 (rush): rush melee min .. rush max.
- 3 (keep distance): desired min .. desired max.
- 4 (strafe): strafe desired min .. strafe desired max.
- other: the default band 5 to 15 units.

The retreat distance is a draw in [retreat min, retreat max]. For each slot a charge distance is drawn in float[charge distance min, charge distance max), using the selected slot's range for every slot (Edge cases).

The three draws are made at creation and again after every normal attack choice (Behavior 7). The counter-attack choice does not redraw them (Behavior 14).

Movement by archetype is the agro machine's outer sub-state 1 (rush), 3 (keep distance) or 4 (strafe), entered from outer sub-state 0 by the approach routine value (Behavior 44). An approach routine other than 1, 3 or 4 leaves the outer sub-state at that value, so the enemy has no archetype movement and the normal attack choice never runs for it (Edge cases).

### 5. States (table) and setter (High)

| Id | Name (working) | Behavior |
| --- | --- | --- |
| 0 | Default / idle | Idle animation pick (Behavior 8). The pre-step runs the setter with the spawn record's default-state word (Behavior 2, step 7). |
| 1 | Timed approach | Entered by a non-hostile enemy on alert. Stores the unit direction from the enemy toward the hero, then moves toward a point 5 units from its current position along that direction. The duration is an integer countdown made from a random float between row +0x70 (minimum) and +0x6C (maximum), then decremented once per tick; when it falls below 1, the enemy returns to its default state. The row schema labels the endpoints Flee Timer Min/Max, but this runtime behavior is an approach. |
| 2 | Alerted search | Setter entry: when row 0x82 (Display in default) is 0, the state becomes 21 instead, unless the enemy's invisibility-off word is set (the second condition). Only DisableInvisibility (SPEC-0030 row 121, 25 disc sites) sets that word; with it set, state 2 is entered normally. Otherwise the search countdown = trunc(float[row 0x60, row 0x5C)), the last-seen point = the hero's position, and the state is 2. Aims at the last-seen point; returns when the countdown ends. Each tick the notice test of Behavior 3 also runs here. The search countdown goes down only while the notice test fails and the search animation is not playing. When the notice test passes, the enemy requests its search animation (17 or 18, or 148 or 149 when its alternative test passes) and, once that animation reaches the 0.99 progress gate, goes to agro (state 19); with no such animation it goes to agro at once. When the countdown ends with the notice test failing, the enemy leaves the state through its exit routine (Medium; the exit's target state is not identified). |
| 3 | Group alert wait | Entered from the socialize alert (Behavior 10); wait then return. |
| 4 | Stunned | Written every tick by the enemy status tick (SPEC-0029 Behavior 27) while status 0 (stun) is active. A stun is not applied to an enemy whose row alignment (0x3C) is 4 (Medium). There is no state-4 handler; the pre-step moves the enemy to agro when the stun ends (Behavior 2, step 7). |
| 5, 6, 7 | Idle wander variants | Wander to random points and wait; notice (Behavior 3) moves them to state 2. |
| 8 | Emerge and hide | Hides at home until the hero comes near, then emerges into agro (Behavior 40). |
| 9 | Approach named target | Approaches a named object by its id, within a distance (Behavior 40). |
| 10 | Victory pose | Setter entry plays 73 (163 for flyers); if the animation is missing, the state becomes 0. Entered when a melee strike leaves the hero at 0 HP (Behavior 13). The pre-step runs the setter with 0 when the animation finishes. |
| 14 | Retreat home and hide | Entered by the distance-hide gate of the agro machine (Behavior 24); three phases, then agro, state 8 or the default state (Behavior 38). |
| 16 | Dead or reanimating (reanimating rows: Behavior 35; spiders: Behavior 12) | While its countdown is zero or more, plays the death animation (58), or 153 when a target query passes, and counts the countdown down. Otherwise waits for event 85, then plays the same animation again and runs its end step (not described). Medium. |
| 17 | Map search | Searches for an object of the map search type and steers to it (Behaviors 9 and 39). |
| 18 | Map-search attack | Plays animation 137 (Behavior 42); at the hit event, runs the spell sub-type (Behavior 43); then sets the recovery countdown and returns to agro. Entered only by the map search per tick (Behavior 9), on two paths; no agro routine writes state 18. |
| 19 | Agro | The agro machine (Behavior 44). Setter entry work: an alarm-group broadcast when the spawn record's group byte (0xB7) is not 0xFF; outer sub-state 0; defence hold 0; recovery countdown 0; combat timer = trunc(float[row 0x78, row 0x74)); all four slot cooldowns 0; the enemy is added to the attack manager (Behavior 45). |
| 20 | Pause | Plays the idle pick. Entered when the global AI pause word is set; returns to the previous state when it clears (Behavior 2). |
| 21 | Pursue recorded point | Moves to the last-seen point until the search countdown ends. |
| 22 | Flyer re-launch | Plays 75, then runs the setter with 0 (Behavior 41). Also written directly by the retarget step (Behavior 2, step 4). |
| 23 | Wait then default | Plays 81, then runs the setter with the spawn record's default-state word (Behavior 41). |
| 11, 12, 13, 15 | Unused by common enemies | Not used. |

Setter rule. Every state change through the setter saves the previous state when it differs, runs the entry work of the new state, stores the state, and resets three step words (0xB00 = 0, 0xB08 = 0, 0xB04 = 120 + int[-60, 60]). The entry work is given per state in the table and in the behaviours named there.

Default case of the setter (setter with 0): when row 0x82 (Display in default) is nonzero, the state becomes 0. Otherwise the state becomes 23 when the spawn record's default-state word (u16 at 0x84) is nonzero, and 0 when it is 0. The start of the routine uses a different rule (Behavior 1, step 1).

"Flyer" in this spec means: row can-fly (0x94) is 1 and the retarget countdown is at most 0 ("airborne"), or row can-fly is 2. A can-fly-1 enemy knocked out of the air is grounded for trunc(float[row 0x9C, row 0x98)) ticks, held in the retarget countdown (Behavior 34, step 2, and hit reaction kind 0 of Behavior 44).

### 6. Target search and hurt handling (Medium)

- Target (High). A class-17 enemy has one target: the selected hero (the hero pointer at 0x35C7D4). Every distance, facing, notice and attack test reads that hero. The other hero, a petpet, townsfolk and other enemies are never targets. When the player switches heroes, every enemy aims at the newly selected hero on the next tick.
- Navmesh pick (High). What the pre-step calls "picking the target again" is a navmesh choice, not a target choice (SPEC-0029 Behavior 2). The enemy takes the first navmesh of its world whose kind is 1 (ground) or 2 (flying, when the row's can-fly is non-zero) and one of whose triangles holds the enemy's own position. That index becomes the enemy's current navmesh.
- Hit reaction: every hit runs the take-hit hook (Behavior 37). It sends a non-agro enemy to agro, doubles the damage, and starts a hit reaction (agro outer sub-state 10 with a hit-reaction kind, Behavior 44). State 14 is entered by the distance-hide gate (Behavior 24), not by hits.
- Death: non-spider kinds use the death hook (Behavior 34). Reanimating rows use state 16 as a countdown and then a rise that heals to full health and returns to agro (Behavior 35; Open question 16). Spiders: Behavior 12. A stun reading of state 16 is not supported: the stunned state is 4 (Behavior 5).

### 7. Normal attack choice (High)

The normal choice runs every agro tick, from step 3 of the agro machine (Behavior 44), before the counter-attack choice (Behavior 14).

1. It runs only while the hero is attackable, that is, while the selected hero's hittable query passes (SPEC-0036, Hit detection, Hittable predicate): not dead, not in the hurt state 17 or the knockback state 48, not in any other busy or held state (16, 25, 33, 36, 50 to 55, 57, 59, 60), not in Roberta's leap or landing, and the script-hold word clear (High). Otherwise nothing happens, and the slot cooldowns do not count down. An attack picked earlier still runs and lands (SPEC-0007, Hit reaction and invulnerability).
2. Each of the four slot cooldowns goes down by one when positive.
3. If the agro outer sub-state is 2 (an attack is running), stop. Otherwise the selected slot (0xA24) is set to -1.
4. The choice continues only in outer sub-states 1, 3, 4 and 11 and only while the recovery countdown is below 1.
5. Slots are tested in the order 4, 3, 2, 1. Testing stops at the first slot that passes all of these:
   a. its type is not 0xFFFF, and its cooldown is below 1;
   b. the distance d to the hero satisfies trigger min (0x18C) < d < trigger max (0x17C), both strict;
   c. the facing cosine (forward dot direction to the hero) is above the melee arc (0x1EC);
   d. a roll int[1, 100] is below the threshold. The threshold is int(trigger chance 0x19C). For approach 4, when dot(enemy facing, hero facing) is at most 0, the threshold is int(int(chance) x backbiter 0x500). When the slot's flags have bit 0x40, the threshold is int(anger counter x 0.05) instead (the anger counter of Behavior 37, step 9);
   e. if bit 0x02 of the slot's flags is set, the number of enemies this one has spawned that are still alive must be below the slot's spawn entity count (u16 0x3D4 + 2k). The bit marks a summoning slot: on a missile slot the sequencer also reads it to fire the lobbed summon (Behavior 13), so this gate caps the live summons; both readings hold;
   f. trigger special, int(0x1AC + 4k): 1 = only while airborne, 2 = only while not airborne, other values = no limit;
   g. charge rule: when the charge type is -1, the slot passes only when a route to the hero exists, or the slot is melee and d < 10.0 units, or the slot is a missile slot; when the charge type is set, the slot passes only when a route exists.
6. On a choice: the slot becomes the selected slot; its cooldown = int[int(delay min 0x21C), int(delay max 0x20C)]; the grab word is cleared; with bit 0x40 the anger counter is cleared. The attack sequencer starts (outer sub-state 2, Behavior 13) in state 13 when the slot's loopable word (0x49C + 4k) is nonzero, otherwise in state 0 when the charge type is -1 and in state 6 when it is set. When the attack manager has the enemy waiting in line and its approach routine is not 3, outer sub-state 12 is entered instead (Behavior 45). Then the desired, retreat and charge distances are redrawn (Behavior 4).

The class state stays 19 throughout. Both melee and missile slots run through the sequencer; class state 18 is only the map-search attack (Behavior 42).

### 8. Idle animation picker (High)

Each tick the picker returns base animation 0 when the movement predicate is false and 97 when it is true. With no one-shot pending, it holds that base until the timer expires, then chooses uniformly among animations 115 to 119; when the movement predicate is true, it substitutes 142. It returns the selected variant only when that animation is present in the enemy table. If the selected variant is absent, it resets the timer and returns the base. When a selected variant is still playing, it is kept; after it finishes the timer is reset and the base is returned. Timer draws are whole ticks in the closed range [120, 600] (SPEC-0008).

### 9. Map search (Medium for control, Low for details)

When the row's map search flag is non-zero and the cooldown is zero, the enemy searches the scene for an object of the map search type (0 or 1 search object kinds; 2 to 8 search for a nearby enemy target). For object searches the object must lie within the map search distance (drawn with the map search chance) and beyond the greedy hostile range for type 0/1 steering. For enemy-target searches, a candidate character is chosen at random among up to 10 eligible characters (one with current HP below maximum, within the search distance, in view). The search type also selects the attack sub-type; the mapping from each search type to its sub-type is listed in Behavior 43. Object types 0 and 1 lead to state 17 instead (Behavior 39). The map search per tick is the only writer of states 17 and 18 (Behavior 42). After a search the cooldown is re-armed in [map search timer min, map search timer max] ticks.

### 10. Socialize and group alert (Medium)

Entering the socialize state draws a talk countdown in [social min, social max] ticks and a quiet countdown in [quiet min, quiet max] ticks. While the talk countdown is positive, notice distances are halved (Behavior 3). When the quiet countdown ends, the enemy looks for one other enemy with the same target and runs a compatibility test. The test rejects specified peer AI states, requires a nonzero eligibility value, and for a non-null peer compares their separation with the sum of the two actors’ distance contributions. It normally reads these values from monster-row offsets +0x40 and +0x44. When the candidate peer’s script-hold word is set, the class-17 path instead reads its eligibility value at companion-row offset +0x16 and the pair’s distance contributions at +0x18. A passing peer is sent to state 3. A zero quiet-countdown value does not disable this check: the counter is decremented first, so it becomes negative and the peer scan runs immediately, then continues on later eligible ticks if no peer passes. This establishes the conditional companion-row read in the class-17 code; how often it succeeds depends on the peers, target indices and selected row values.

### 11. Death (High for the branches; Medium for the tail)

Non-spider common enemies die through the death hook (Behavior 34). Normal death sets the base actor state to 16 and loads the death disappear countdown from row 0x80. Rows with the reanimation flag enter a reanimation countdown instead (Behavior 35). The dead-state update plays the death clip and removes the corpse when the countdown ends (Behavior 36). Spiders keep their own path (Behavior 12). Open question 4 is answered for non-spider kinds.

### 12. Spider states used by the nest (Medium overall; release and death High for the state steps)

Spyders, poison spyders and giant spyders are common enemies (class 17, Default Behavior 17 in resource 530). The Spyder Nest (SPEC-0017) uses two spider states, the inactive state and the active state, and an idle-variant chooser for the animation.

**Inactive state (16).** When the nest is set up, every spider it owns is hidden and set to state 16. The nest counts a spider in state 16 as dead, so a hidden spider does not count as alive until it is revealed (High for the nest side, SPEC-0017 Behavior 1). A spider that is down is also set to state 16 (Medium). While its countdown is zero or more, state 16 plays the death animation 58, or 153 when a target query passes, and counts the countdown down. When the countdown is negative, the handler checks whether animation 85 is present in the spider’s own animation table. If it is present and not current, a spacing test requires the spider to be at least 5.0 units from the hero and every other active actor. Passing that test starts animation 85. Failing it restarts the death animation, choosing 58 or 153 by the same target query used during the countdown. While animation 85 is current, the handler waits for its completion; then it restores the spider to full HP and enters class state 19 (agro). If animation 85 is absent, it reaches that same full-HP/agro end step immediately. This state therefore supports a spacing-gated return from inactive/death to agro; 85 is an animation code, not an event id. The nest counts living spiders as those whose state is not 16 (SPEC-0017). Medium-High for the state transitions; animation availability is per spider table.

**Release (revive) from the nest.** When a valve reveals its spiders (SPEC-0017, Behavior 4), each spider that the valve's list resolves, among the first N entries (N = the valve's spider count), is revived:

- base actor state set to 1 (active), and the class-state setter run with 19, the agro state (High for the state steps);
- HP set to the spider's maximum HP;
- despawn flag cleared;
- navigation area set to the hero's navigation area (the area the nest's placement helper found);
- position set to the valve point, moved 5.0 along the valve's forward axis and 0.0 along its up axis;
- velocity gains 10.0 units per second along the valve's forward axis and 10.0 units per second along world vertical (y). The shared character mover applies velocity times 1/30 each 60 Hz tick, so each launch component contributes 1/3 unit per tick before gravity, friction and contact modify the motion. The release adds these components to the motion record's existing velocity; it does not establish that the prior velocity is zero. This is the unused Spyder Nest release path: the disc places ordinary Spyders (row 234), Poison Spyders (row 235) and Giant Spyders (row 236) as class-17 common enemies, but places no nest or valve (SPEC-0017). (Open question 44, resolved; High for units and mover factor.)
- plays an idle-variant animation chosen by the chooser below (Medium for the animation values).

The two numbers live in different words (Open question 46, resolved; High). The 1 is the base actor state, not class state 1 (the timed approach). The 19 is the class state, entered through the setter with its full entry work (Behavior 5 table): an alarm-group broadcast when the spawn record's group byte is not 0xFF, outer sub-state 0, the combat timer drawn from the row, all slot cooldowns 0, the spider added to the attack manager, and the previous class state saved. So a revived spider runs the agro machine (Behavior 44) from its first tick.

Test vector: a spider in base state 16 with class state 0 is revealed by its valve. Afterwards its base state is 1, its class state is 19 and its previous class state is 0. It is in the attack manager, and its combat timer lies in the row's range (row 0x74 to 0x78).

**Idle-variant chooser (High for runtime mapping).** Spiders use the same generic class-17 picker as Behavior 8 during the common loop idle state. It returns base animation 97 when the movement predicate is true and 0 otherwise. When a variant hold expires it chooses uniformly from 115 to 119, or substitutes 142 when the movement predicate is true; the chosen animation must be present in that spider table. A missing choice resets the hold timer and returns the base. The hold draw is the closed whole-tick range [120, 600], so both endpoints are reachable (SPEC-0008). Animation-content labels and intended roles for 97 and 142 are not established.

- Variant values listed in the source are 97, 115, 116, 117, 118, 119 and 142. The values 115 to 119 are the five idle variants of Behavior 8.
- Each tick the chooser returns the current idle variant. When the hold counter is set, it is drawn as a whole number of ticks from 120 to 600. When it runs out, the chooser picks a new variant among 115 to 119.
- A spider-side test can select variant 142 instead of the 115 to 119 range. The test, and the rule for variant 97, are not described here (Open question 45).

**Death (High).** A spider in state 16 whose death animation (58) is playing and has finished gets its despawn flag set, once per tick. The nest sets this flag for a registered spider in that condition and clears it when the spider is released again (SPEC-0017, Behavior 9). The region update skips the spider’s update while this flag is set, and the shared drawable test suppresses its rendering (SPEC-0006; SPEC-0046). The spider remains registered in its active slot in the evidence examined; the flag is cleared when the nest releases it. No object-removal path is established. Animation 58’s length is supplied by the spider’s animation data; no fixed state-16 duration is used. The state-16 return path is described above; Open question 47 remains for the clip length and any actual removal path.

**Agro sub-states of a spider (High).** A spider in agro runs the common agro machine (Behavior 44). Its outer sub-states 1, 3 and 4, which turn toward the hero and play the hold animations 22 or 155, 25 or 156 and 31 or 132, are the rush, keep-distance and strafe archetypes of every class-17 enemy, not a spider-specific spawn rule; the spyder rows use approach routine 4 (strafe). A revived spider (base actor state 1, class state 19) enters agro through the setter, so it uses its row's approach routine like any other agro enemy (Open question 46, resolved).

### 13. Attack sequencer (agro outer sub-state 2) and projectile spawners (Medium-High for the states and transitions; High for the spawn rule of state 2 and the spawners)

While the agro outer sub-state is 2, one attack runs through a sequencer of sixteen states, numbered 0 to 15. States 10 and 11 have no writer and are never reached. The enemy keeps the selected slot k (Behaviors 7 and 14) and reads that slot's row values. The sequencer runs once per tick. It starts in state 0 (no charge), 6 (charge) or 13 (loopable slot).

Slot animations: A = 33 + 5k (wind-up), B = 34 + 5k (strike), C = 35 + 5k (follow-through), D = 36 + 5k (blocked), E = 37 + 5k (recovery). Each is checked against the enemy's own table before it is played, and a missing animation is skipped (Behavior 20; Layout section 6). With the slot's orient flag (row u16 0x174 + 2k) set, the enemy aims at the hero at weight 1.0 in states 1 to 3. Speed words: wind-up 0x1BC + 4k (states 1 and 6), strike 0x1CC + 4k (state 2), recovery 0x1DC + 4k (state 5); they are stored directly to P before the same-update clip re-time, so the selected row value sets the clip rate for that update (Layout section 2; Behavior 1; Edge cases).

- **State 0, approach and aim.** For a melee slot with d above the melee range (0x1FC + 4k): gait 0 and pursue the hero; when the hero's position or the steer target is off the walk mesh, go on to the facing test; otherwise stay. In all other cases (a missile slot, or within range): when the hero and the steer target are both on the walk mesh, go to state 1 at once; otherwise the facing test: a facing cosine of at least 0.95 (about 18 degrees) goes to state 1, a lower one turns toward the hero (blend 0.1) and stays.
- **State 1, wind-up.** Play A to its end; then start B and go to state 2 (no B: state 5).
- **State 2, strike (High for the spawn rule).** B plays. Melee slot: the strike test (below) runs each tick: a hit goes to 3, a blocked strike to 4; with no contact, B finishing goes to 5. Missile slot (type 1): while the spawn-pending flag is set, it is cleared on the first tick the slot's spawn event is not active. When the flag is clear and the spawn event is active, one spawner runs (below): row flag 0x02 selects the lobbed summon; otherwise the flight mode selects spawner 0, 1 or 2, and any other flight mode spawns nothing. The shot counter then goes up by one, even when the projectile pool was full and nothing was created. If the counter is below the slot's spawn count (0x33C + 4k), set the pending flag and stay in state 2; otherwise go to state 3. A spawn count of 0 still fires one projectile (the counter 1 is not below 0). If B finishes first, go to state 5.
- **State 3, follow-through.** Play C to its end, then state 5. With no C: while B still plays, a new contact re-enters state 2 once (the hit list is cleared); when B finishes, state 5.
- **State 4, blocked.** Play D to its end, then state 5.
- **State 5, recovery.** Play E to its end, then the end path. With flag 0x08 (grab-and-throw, Behavior 15) the recovery animation is skipped.
- **State 6, charge wind-up.** The start point = the enemy's position. Play A to its end; then start B and go to state 7 with the tick counter at 0 (no B: state 12).
- **State 7, charge run.** The tick counter goes up by one, and t = the distance travelled from the start point. Charge kinds 0, 1 and 3 run the strike test (hit: state 8; blocked: state 9); kind 2 runs no strike test. For every kind, a tick counter above 100 with the enemy's speed at most 0.001 goes to state 12. Kinds 0 and 1: t at least the charge distance goes to 12. Kind 2: the hero within the melee range goes to 8; otherwise t at least the charge distance goes to 12. Kind 3: t at least the charge distance, or the hero within the melee range, or a speed of at most 0.1 goes to 12.
- **State 8, charge hit.** Kinds 0 and 1: play C; t at least the charge distance, or the blocked flag (0xCC4), goes to 12. Kind 2: play C with the strike test each tick; C finished goes to 12. Kind 3: C finished goes to 12.
- **State 9, blocked charge.** By the slot's blocked-charge kind (0x2AC + 4k): 0 plays D and goes to 12 when t is at least the charge distance; 1 goes to 12 when D has finished; 2 starts hit reaction kind 2 (outer 10, Behavior 44) with the stun countdown (0xBF0) = trunc(float[row 0x28C + 4k, row 0x27C + 4k)) and the stun type (0xC60) = row 0x29C + 4k; any other value starts hit reaction kind 3. Resource 530 has 45 used class-17 charge slots: blocked-charge kind 0 on 38, kind 2 on 3, and kind 4 on 4; kind 1 does not occur. Thus the shipped charge slots use the kind-4 fallback to reaction kind 3, while the kind-1 timing branch has no recovered class-17 row instance.
- **States 10 and 11.** No writer; never reached.
- **State 12, charge end.** Play E, then the end path.
- **State 13, loop approach.** While d is above the melee range, pursue the hero; otherwise go to state 14 with the countdown at 60 ticks.
- **State 14, loop strike.** Play B. When the countdown is 0, the hero is damaged directly and the countdown is reloaded with int[row s32 0x4AC + 4k, row s32 0x4BC + 4k]; otherwise the countdown goes down by one. The slot flag 0x20 selects the fallback victim reaction: clear requests base state 17 with a selected animation; set calls the victim's additional attack-reaction method, which can reset the hero attack sub-state to 13 when its checks pass (Layout section 2; SPEC-0036). The flag's design-facing name remains unknown. When d is above row 0x4CC + 4k, go to state 15.
- **State 15, loop end.** Play E, then the end path.

**End path.** Outer sub-state 0; the recovery countdown = int[int(row 0x68), int(row 0x64)]; the slot cooldown = int[int(0x21C + 4k), int(0x20C + 4k)] again, so the slot cooldown is set twice: at the choice and here (Edge cases).

**Strike test (Medium; knockback Medium-Low).** The slot's attack volume is tested for contact with the hero. With flag 0x01 the strike cannot be blocked. A hit deals the slot's damage. Flag 0x04 performs the theft described in SPEC-0029 Behaviors 11a and 11b. When the status id (0x36C + 4k) is not -1, the handler draws an integer from 0 through 100 inclusive and applies the status only when the draw is strictly below the row chance (0x37C + 4k); chance 100 succeeds on 100 of 101 outcomes, and chance 0 never succeeds. The status duration is int[int(0x39C + 4k), int(0x38C + 4k)] / 60 whole seconds. This differs from the projectile-on-hero status roll in SPEC-0029 Behavior 9, whose comparison includes the threshold. The hit also knocks the hero back in the direction normalize(9.0 x facing + 12.0 x up), with the slot's knockback value (0x35C + 4k) as the velocity magnitude. Character-record velocity is expressed in units per 1/30-second step (SPEC-0040 Behavior 14A), so this value is per-step, not units per second; the shared integrator multiplies velocity by 1/30 each tick. With horizontal unit facing, the normalized direction has horizontal magnitude 0.6 and vertical component 0.8 (SPEC-0036 Knockback on the hero). Flag 0x80 drains Motes (amount 0x42C + 4k). A hit that leaves the hero at 0 HP sends the enemy to state 10 (victory pose). With flag 0x08 the grab-and-throw (Behavior 15) replaces the strike.

**Projectiles.** The projectiles go into the enemy projectile pool of SPEC-0029 Behavior 9 (one global pool of ten slots, first free slot, a full pool creates nothing), which moves them, tests hits and releases them. The missile count (0x33C) is the spawn count of state 2.

**Spawners of state 2 (High).** k is the attack slot. Common to all four spawners:

- Launch point: the enemy's emit point for the spawn-event bit that fired. The bits are tested in the order 1, 2, 4, 8 and 16; the first one set selects emit point 0, 1, 2, 3 or 4 (bit 1 gives point 0, bit 2 point 1, and so on). Bit 0x20 and any other word select point 0 (High). Test vector: event word 0x26 selects point 1; event word 0x20 selects point 0.
- Aim point = target point + e w. w is a random horizontal unit vector, from x and z each drawn uniformly in [-1, 1] and normalized, drawn in the order e, then x, then z (High). e = U[0, row inaccuracy 0x30C + 4k], plus 5.0 when the enemy is outside the camera cone (dot(camera forward, unit(enemy - camera)) < 0.85), plus 3.0 when the attack manager has the enemy waiting in line (Behavior 45).
- Owner = the enemy id; element = row 0x3C; damage = row 0x34C + 4k times the enemy damage scale (1.0; x 0.5 under curse, status 1; x 1.5 under strength, status 4), floored at 0.
- Radius = row 0x2EC + 4k. Roll = row 0x32C + 4k. Spin rate = row 0x31C + 4k. Visual kind = row 0x43C + 4k. Six effect ids = the row u16 words at 0x46C + 2k + 8j (j = 0 to 5).
- Area: radius row 0x45C + 4k, duration trunc(row 0x44C + 4k) ticks. Status: id row 0x36C + 4k, chance trunc(row 0x37C + 4k), value trunc(U[row 0x38C + 4k, row 0x39C + 4k] / 60).
- Trail effect (visual kinds below 5): entry n of the enemy's 11-entry model-part pointer table, with n = 1, 7, 8, 9 or 10 for a shot counter of 0, 1, 2, 3 or 4 (other counts give 1). These are zero-based pointer indices, not projectile effect IDs. Flight mode 1 always uses entry 1. In resource 530, 16 missile slots have visual kinds below 5; six have at least one of their six separate projectile effect IDs not equal to -1. Thus those IDs are not universally empty for this missile subset. This is a count of the data; reachability and whether each effect emits depend on the attack path.
- Ignore-world flag = row flag 0x10 (flight modes 0 and 2).

| Spawner | Projectile kind | Target point | Velocity | Lifetime (ticks) |
| --- | --- | --- | --- | --- |
| Flight mode 0 | 0 (straight) | hero position + (0, 2.5, 0) | unit(aim - launch) x v | trunc(range / v) |
| Flight mode 2 | 17 (homing) | hero position + (0, 2.5, 0) | unit(aim - launch) x v | 2 x trunc(range / v) |
| Flight mode 1 | 1 (gravity, area on landing) | hero position | ballistic, ramp speed s | trunc(60 T) + 1 |
| Row flag 0x02 | 8 (lobbed summon) | hero position | ballistic, speed v | trunc(60 T) + 1 |

v = the launch speed (0x2CC + 4k) in units per tick; range = 0x2FC + 4k in units.

**Ballistic launch (High).** d = the horizontal distance from the launch point to the aim point; dy = aim.y - launch.y. For flight mode 1, u = 1 - (d - bmin)/(bmax - bmin) with bmin = row 0x18C + 4k and bmax = row 0x17C + 4k (the slot's trigger band), and s = v + u^2 (v_end - v) with v_end = row 0x2DC + 4k; so s = v_end at bmin and s = v at bmax, and u is not clamped. The lobbed summon uses s = v. Then T = d / s ticks, the horizontal velocity is s x unit(horizontal offset), and vy = (0.5 g' T^2 + dy) / T with g' = 32 / 60^2 (SPEC-0029 Behavior 9). The lifetime trunc(60 T) + 1 multiplies a flight time already in ticks by 60 (a unit slip in the original; reproduce it).

**Lobbed summon (row flag 0x02; High).** Kind 8, visual kind 23, damage 3.0 and radius 1.0 (fixed; kind 8 never touches a character). The projectile carries the slot's spawn key (row u16 0x3CC + 2k); when it lands, or its owner disappears, the keyed summon-reserved record spawns at the landing point (SPEC-0029 Behaviors 9 and 31). An expiry in the air spawns nothing. The same flag also gates the slot in the attack choice by the count of the enemy's living summons (Behavior 7, step 5e); both readings hold. On disc the missile slots with this flag are row 211 slot 0 (Minion Wizard, Minion Grunt) and row 221 slot 3 (Spore Monster, Flower Monster).

### 14. Counter-attack choice (High)

The counter-attack choice runs every agro tick, right after the normal choice (Behavior 7), in any outer sub-state. It has no attackable test, no recovery test and no outer sub-state gate, and it does not count the cooldowns down. It reads the row's Counter Attack Trigger (0x22C + 4k) and Counter Attack Chance (0x23C + 4k) (Layout section 5).

1. Walk the slots from 1 to 4. A slot is eligible when all of these hold:
   a. its type is not 0xFFFF, and its cooldown is below 1;
   b. trigger min < d < trigger max (strict), as in Behavior 7 step 5b;
   c. the facing cosine is above the melee arc, as in Behavior 7 step 5c;
   d. its trigger code holds:
      - 1: the active hero's recover window is open this tick (below);
      - 2: after a dodge: outer sub-state 5 or 6, and the recover window;
      - 3: after a block: outer sub-state 7 or 8, and the recover window;
      - 4: sequencer recovery: outer sub-state 2 with the sequencer in state 5;
      - 5: the sequencer is in state 4 and L1 is held in pad slot 0 (mask 0x0004 in the active hero's current held-button word; SPEC-0033, Controller map). The word is rebuilt from the controller report each input pass, so this is a held-input condition, not an animation event or persistent hero flag. The hero's control-return flag at +0x3D0 is a separate boolean and is not read here. Thirty-two active class-17 row slots on disc use code 5;
      - 6: hit reaction: outer sub-state 10;
      - 0, -1 and any other value: never.
2. The last eligible slot of the walk wins: a later eligible slot replaces an earlier one.
3. If the winning slot is not already the selected slot, one roll int[0, 100] (101 values) is drawn for the tick. When the roll is below the chance, the slot becomes the selected slot, its cooldown = int[int(0x21C + 4k), int(0x20C + 4k)], and the sequencer starts in state 0 (charge type -1) or state 6 (charge type set). A chance of 0 never passes; a chance of 100 passes on every roll except 100.

Recover window (High): table-B event bit 0x800 on any of the active hero's playing clips this tick (SPEC-0005, Table B frame bits). Only the hero reports it; the same query on any other character reports none. Tor's swings (ids 14 to 20 and 27) carry it; Roberta's clips never do, so trigger codes 1 to 3 never fire while Roberta is the active hero.

The distances are not redrawn, so a counter charge runs to the charge distance of the last draw (Edge cases). The loopable start state 13 is not used by this path. The "states" named by the trigger codes are agro outer sub-states and sequencer states, not class states; the choice never enters class state 18.

### 15. Grab-and-throw strike (row flag 0x08) (Medium)

A slot whose flags (0x3AC + 4k) have bit 0x08 replaces the melee strike of the sequencer with a grab-and-throw. On disc 10 used class-17 slots have the bit. The machine is described step by step in SPEC-0029 Behavior 23 (its states 0 to 4); in outline:

1. Grab: when the hero is within the slot's melee range (0x1FC + 4k) and the facing cosine is above 0.5, and the sight path is clear, the hero is put in base state 53 (held).
2. Hold: while the hero is held, a periodic effect with the values 1.0 and 0.7 (the grab rumble, SPEC-0033) runs every 61 ticks. Its timer is the run-time word 0x2DD54C (boot value -1): one global countdown shared by all enemies, not a per-enemy field. The countdown is loaded with 60 (the frame-rate word as an integer); on each held tick, if the countdown reads 0 the effect fires and the countdown is reloaded with 60, and otherwise it falls by 1. It therefore counts 60 down to 0 and fires on every 61st tick (SPEC-0029 Behavior 23).
3. Throw: the hero's motion-record velocity becomes normalize(15.0 x facing + 8.0 x up) x the slot's throw speed (row 0x41C + 4k). This is an initial velocity in units per second; the shared motion step advances position by velocity x 1/30 each tick at 60 ticks per second (SPEC-0040 Behavior 14A). Later gravity, contact and movement constraints affect the resulting trajectory. (High for the velocity field and unit conversion.)

With bit 0x08 set, the sequencer skips the recovery animation of state 5, and the lost-sight test of the agro machine does not run while the attack runs (Behavior 44). Entering agro outer sub-state 2 writes the grab machine directly to state 1; state 0 is restored when the next attack slot is selected. The four grab-point event selectors read the enemy's current clip and choose its attach transform (SPEC-0029 Behavior 23). The hero's held-state handler requests animation 143 every tick and asks every actor in the level's actor list the same yes-or-no question; it remains in base state 53 while any actor answers yes, and returns to base state 1 on the first tick on which none does. The enemies answer yes only while they are in agro outer sub-state 2, in sequencer state 2 (the strike state) and the current attack slot has the grab flag 0x08 set. The question does not identify or compare against the enemy that initiated the grab, so any enemy that is striking with a grab slot holds the hero. The heroes and every other actor kind keep the base answer, which is always no (High; Medium that no non-class-17 actor reaches the sub-state words). Test vector: hero in state 53, one enemy with outer sub-state 2, sequencer state 2 and slot flag 0x08: the hero stays in state 53. On the tick the sequencer leaves state 2, with no other grabbing enemy, the hero is in base state 1. The same enemy in sequencer state 2 with a slot whose flag 0x08 is clear does not hold the hero. The row field 0xE0 is not a cooldown of this strike: it is the defence hold time of the defence reactions (Behavior 44).

### 18. Band factors by hero action state (High)

Two factors, F1 and F2, are looked up by the hero's current action state (SPEC-0036). F1 scales the notice band (Behavior 3) and the emerge distance of state 8 (Behavior 40); F2 replaces F1 in the notice band when the sight line is blocked (Behavior 3). The values are boot data words: F1 is the f32 at 0x2DD5A4 + 4i and F2 the f32 at 0x2DD5C0 + 4i, with i = 0 for action states 2 and 5, 1 for 3 and 9, 2 for 4, 6 and 10, 3 for 21 and 41, 4 for 23, 5 for 22 and 6 for 1. Any other action state uses the fixed constants F1 = 1.0 and F2 = 0.5.

| Hero action state | F1 | F2 |
| --- | --- | --- |
| 1 | 0.7 | 0.0 |
| 2, 5 | 0.75 | 0.25 |
| 3, 9 | 1.5 | 0.75 |
| 4, 6, 10 | 1.75 | 0.9 |
| 21, 41 | 0.5 | 0.3 |
| 22 | 0.5 | 0.0 |
| 23 | 0.3 | 0.0 |
| other | 1.0 | 0.5 |

An F2 of 0.0 makes the blocked-sight band empty, so in those hero action states an enemy without the always-has-line-of-sight flag cannot notice the hero through an obstacle.

### 19. Animation codes by stage, Draik Skeleton lookups (High)

These lookups belong to the Draik Skeleton (class 12; SPEC-0029 Behavior 6), not to the common enemies: the only routines that call them are the Draik's own attack handlers (reached through its attack dispatch), and no class-17 routine calls them. Each maps the Draik's stage value to an animation index. Stage values 1 to 5 are used by the five-entry lookups; the four-entry lookups cover stages 1 to 4, and stages 2 and 4 are never entered. Any other index returns -1 (no animation). Roles (High for the tables and the callers; Medium-High for the labels): group A is the start animation of the fire and ice breath, group B its held animation and group C its end animation, by stage; group D (index 1 gives 7) is the claw; group E (index 3 gives 0x20, index 5 gives 0x35) is the stomp; and two five-entry lookups not listed below give the turn animation (10, 0x18, 0x22, 0x2F, 0x37) and the fire and ice spit (6, 0x13, 0x1F, 0x2C, 0x34) by stage.

| Group | 1 | 2 | 3 | 4 | 5 |
| --- | --- | --- | --- | --- | --- |
| A | 3 | 0x10 | 0x1C | 0x29 | (none) |
| B | 4 | 0x11 | 0x1D | 0x2A | (none) |
| C | 5 | 0x12 | 0x1E | 0x2B | (none) |
| D | 7 | 0x14 | (none) | (none) | (none) |
| E | (none) | (none) | 0x20 | (none) | 0x35 |

Group B is group A with every code one higher, and group C is group A with every code two higher. The main-family table gives a concrete clip-index mapping for several shipped Werelupe kinds: IDs 3, 16, 28 and 41 resolve to clips 17, 21, 0 and 26 for Werelupe Bashers and Elites, and to clips 16, 21, 0 and 26 for Werelupe Bone Throwers. These IDs select entries in the character record's bound main clip bank. For Poison Spyders, IDs 3, 16, 28 and 41 resolve to clips 13, 29, 3 and 8 in all 28 checked placements. For all 86 scanned Drackonack records of type 66, variant 0 (monster row 144), the same IDs resolve to clips 11, 23, 21 and 18 in a 40-entry main clip bank. For Minion Archer (monster type 80), IDs 3, 16, 28 and 41 map to clip entries 21, 21, none and 9 in all 145 scanned character records across 34 resources and variants 7 to 13. Test vector: ID 3 -> clip 21; ID 16 -> clip 21; ID 28 -> no clip; ID 41 -> clip 9. For variant 7, the active fourth attack slot uses animation IDs 48 to 52; all 37 checked character records select clip entries none, 14, none, 9 and none respectively. For Minion Warrior (monster type 82), the 85 checked character records give IDs 3, 16, 28 and 41 as clips 1, 26, 3 and 9 in 83 records; the variant-6 record maps them to 1, 31, none and 9, and one variant-0 record maps them to 0, 31, none and 8. In these tables, 4095 means no clip. These are clip-entry mappings, not visual action labels; mappings for other codes and common-enemy kinds remain open (Open question 49).

### 20. Attack and emerge codes from state words (High)

(a) Attack code. The selected attack slot (Layout section 1, object offset 0xA24, value 0 to 3) selects a code. Group 1 gives 0x21, 0x26, 0x2B and 0x30 for values 0 to 3: these are the slot wind-up animations A (33 + 5k) of the sequencer. Group 2 gives 0x24, 0x29, 0x2E and 0x33: these are the slot blocked animations D (36 + 5k) (Behavior 13). The code is returned only when the animation is present in the enemy's animation table; otherwise the result is -1, and the sequencer skips that animation.

(b) Emerge code. The default code is 10. When either of two per-class tests passes (the flyer test), the code is 0xA5 (165). Otherwise the selector word (Layout section 1, +0xC8 of the spawn record) selects the code: 0 gives 10, 1 gives 0x85 (133), 2 gives 0x86 (134), 3 gives 0x87 (135) and 4 gives 0x88 (136). These are the emerge animations of state 8 and of the alarm-group check (Behaviors 3 and 40), not idle variants. The matching hide pick of state 8 is 9, or 128 to 131 by the same selector (Behavior 40).

### 21. Action-19 gate (Medium)

The gate is called by the common-enemy pre-step when the retarget countdown reaches 0 (Behavior 2, step 4). It returns 1 without a class test when the current class state is not 19. In class state 19, it calls the class-17 predicate: the predicate blocks when agro outer sub-state 2 and sequencer state 0 to 3 are both active; outside that range, a nonzero object word at 0xE58 also blocks. The gate inverts this result: a block skips target reacquisition; a pass performs the navmesh pick and sets state 22 directly. The meaning of 0xE58 is not identified (Open question 54).

### 22. Spacing predicate (Medium)

A candidate point passes only when its horizontal distance is at least 5.0 units from the hero and from every other non-null object in the shared pool. The test uses the horizontal-distance lane of the shared position helper, which ignores height. The enemy itself is not compared with itself.

### 23. Agro tail: route flag and leash (High)

The tail runs at the end of each agro tick (Behavior 44, step 5).

1. In outer sub-states 1, 3 and 4 only: the route flag (0xAFC) is set to whether a route to the hero exists.
2. With no route: if one slot's trigger band holds d and the selected slot index is 1, the enemy faces the hero and stops; this branch never runs, because the normal choice has set the selected slot to -1 earlier in the same tick (Edge cases). Otherwise the leash counter (0xB50) goes up by one when the enemy's current walk waypoint is at least 15.0 units from the hero, and is set to 0 when it is closer.
3. With a route, or in any other outer sub-state: the leash counter is set to 0.
4. When the leash counter is above 180 ticks (3 seconds), it is set to 0 and the setter runs with 0 (the default case, Behavior 5).

Test vector: outer sub-state 4, no route, the walk waypoint 15.0 units from the hero: the counter is 181 after 181 ticks and the enemy leaves agro on that tick. With the waypoint 14.9 units from the hero the counter stays 0. With a route the counter stays 0 at any distance.

### 24. Distance-hide gate (High)

The gate runs every agro tick, from step 3 of the agro machine (Behavior 44), only while the outer sub-state is 1 (rush) and the class state is not 14. When the row's distance hide flag (u16 0xA0) is nonzero, the hero's distance d is above the row's hide distance (0xA4), and a roll int[0, 100] is below the row's hide chance (0xA8), the enemy enters state 14 (retreat home and hide, Behavior 38). The roll is drawn again every tick the other tests pass. It is a closed integer draw (0 to 100, both included) on the shared random generator, whose state is at 0x39BE50 (SPEC-0008; Open question 53).

Rows with the flag: rows 9, 11, 237, 238 and 239 use approach routine 1 (for example the tentacles), so the gate is live for them. Rows 233 to 236 (spyders) use approach routine 4, so the gate never fires for them; the flag only makes state 8 hide in place (Behavior 40).

Test vector: flag 1, hide distance 5, chance 30, outer sub-state 1, d = 8: a roll from 0 to 29 (30 of the 101 values) enters state 14; a roll of 30 or more does not. At d = 4 the gate never fires. The same row in outer sub-state 4 never fires.

### 25. Proximity retarget (High for the gate, point and target update; Medium-High for the distance-threshold role)

In the state-5 side routine, when the timer has expired and the stored positive scalar r (object offset 0xC9C) is above 0, let P be the character’s position, F its forward basis, and Q its current offset point (object offset 0xCA0). If the horizontal XZ distance between P + 6F and Q is less than r, set A = P + 3r × unit3D(P − Q), store A at object offset 0xAD0, and pass A to the walk-target setter (SPEC-0056). The comparison is strict; equality does not retarget. The trigger ignores Y, while the normalized displacement uses X, Y and Z. The setter updates the navigation target and clears its arrival flag; the movement pass follows separately. The scalar and point are supplied by the CharacterSetAvoidPoint world-pack setting. A recovered packed-script caller applies setting index 33 to each of six character IDs from its character list; 33 selects the setting and is not the scalar r. The scalar acts as a distance threshold in the same coordinate units as P and Q: it is compared with a horizontal length and scales a 3D position displacement. Its exact design-facing label (for example, a radius versus a clearance) is not established. The factor 3.0 in A = P + 3r × unit3D(P − Q) is a spatial multiplier; it is separate from class 35’s wait of tick-rate × 3.0 (Open question 28).

Test vector: P=(0,0,0), F=(1,0,0), Q=(6,0,0), r=1 gives gate distance 0 and target A=(-3,0,0). With Q=(7,0,0), the gate distance is exactly 1, so the strict comparison does not retarget.

### 26. Value tables (lookups High; meaning unknown)

Seven tables of floats are read by 1-based lookups, where index 1 is the first entry. The lookups rooted at 0x2DA6A0 and 0x2DA8F8 are not common-enemy tuning: the first is read by the class-12 Draik breath standoff handlers as the 10.0 lower bound, paired with the adjacent 15.0 upper-bound table; the second is read by the class-12 Draik stomp handler as the 7.0 approach boundary (SPEC-0029, Draik behavior). The two lookup families use the current Draik stage as their index. The remaining tables of the family are also Draik attack-band and standoff tables, indexed by Draik stage: the breath abort distance (0x2DA6C8, 25.0), the claw standoff pair (0x2DA77C, 0x2DA790), the wind approach limit (0x2DA808) and the pairs of attack-band limits at 0x2DA678, 0x2DA68C, 0x2DA704, 0x2DA718, 0x2DA754, 0x2DA768, 0x2DA7CC, 0x2DA7E0, 0x2DA8BC and 0x2DA8D0. Every user is a Draik handler (SPEC-0029, Draik behavior; Open question 50).

### 34. Death hook (High for the branches, the flyer test and the coin split)

Numbers in Behavior 34 to 46 are decimal (state numbers, animation numbers and tick counts). Row and object offsets are written in hex, as elsewhere in this spec.

When an enemy's health falls to 0 or below from a hit, the damage routine sets health to 0 and calls the monster's death hook with the enemy and two flags set to 1. The hook reads the enemy's monster row.

1. Reanimation test. Read the row's reanimation flag (0x15C).
   - Flag 0 (106 of the 109 class-17 rows): if the row's class id is not 1, 32 or 11, the base actor state (0x144) is set to 16 (dead) through the character class's base-state setter, which first copies the old base state to the previous-base-state word (object offset 0x7FC) when the two differ (High). The class state is not changed. The meaning of the class id test is not confirmed.
   - Flag nonzero (rows 224, 226 and 227): if the reanimation countdown (object offset 0xD14) is at its idle value of -1, draw a whole number of ticks from the reanimation min (0x160) to the reanimation max (0x164), both included, store it as the countdown, and set the class state to 16 (Behavior 35). If the countdown is not -1, nothing changes. A second death while reanimating does not restart the countdown.
2. Grounding of a flyer. If the first flyer test passes (the row's can-fly value 0x94 is 1 and the retarget countdown, object offset 0xBF4, is 0 or less), draw a decimal number of ticks from the ground timer min (0x9C) up to, but not including, the ground timer max (0x98), cut down to a whole number, and store it as the retarget countdown. The maximum is therefore not reached. A positive countdown makes that test fail, so a can-fly 1 enemy counts as grounded from its death on. The think loop that would count the countdown down is not stepped for a dead enemy (Behavior 1), so it stays grounded as a corpse (High).
3. Death countdown: set once, in part C of step 6. Step 1 changes a state or sets a countdown only on a first death tick (a row with flag 0 keeps the reanimation countdown at its idle value -1, and a row with the flag sets it only from -1), so part C covers every such case with the same value.
4. There is no separate removal step. The helper that takes the enemy's position and the word at 0xC28 is the coin split of part D below, and 0xC28 is the carried Neopoints (Open questions 64 and 66, resolved).
5. All 14 status timers are cleared (SPEC-0040). This runs last, on every call.
6. Drop and payout. The payout runs only when the hook is called with its flag set, and only on the enemy's first death tick: the tick on which the base state was not 16 before the call, or the reanimation countdown was at its idle value -1. It reads the direct drop item (enemy object offset 0xA50; SPEC-0029, Layout section 1) and the drop placement ordinal (the signed 16-bit word at data offset 0xB0 of the enemy's spawn record, object offset 0xA44). The parts run in this order:
   - A. Direct item. If the direct drop item is 0 or more, spawn that item at the enemy's position and set the direct drop item to -1. Part B is then skipped.
   - B. Placement. Otherwise use the drop placement ordinal. It is valid only when it is in range of the enemy's region's placement list; an ordinal outside the list does nothing. For a valid ordinal, the placement's key decides:
     - Key 0 or more: spawn the item named by the key at the enemy's position. The fixed count is not used.
     - Key -1: if the placement's group is valid and its money roll succeeds (SPEC-0021), the money amount M is taken from the group's money range. If there is no money amount, M is the placement's fixed count, but only when that count is above 0. If M is above 0, split M into coins at the enemy's position (100, 25, 10, 5, 1) and stop. Otherwise roll the group's item (SPEC-0021) and spawn the item if one is given.
     - A group of -1 has no money roll and no item roll. Only the fixed count can pay coins.
   - C. Death countdown. Set the death disappear countdown (object offset 0xD24) to the row's death disappear timer (row offset 0x80), in ticks. This runs on every first tick, after A or B.
   - D. Carried Neopoints. Split the carried Neopoints (enemy object offset 0xC28) into coins at the enemy's position (100, 25, 10, 5, 1), then set the carried Neopoints to 0. This runs on every first tick, after A or B, whether or not A or B paid anything. If the carried value is 0, nothing spawns. The carried value grows only in the item steal branch of an attack (SPEC-0029, Behavior 11b). The Neopoint steal does not add to it.

The coins of part B and the coins of part D are separate payouts. A key -1 placement pays its own coins (or an item when it pays no coins), and the carried Neopoints are split on the same tick (Open question 79, answered).

The hook is also called with a flag of 0 on the reset path. Steps 1, 2 and 5 still run; parts A to D are replaced by setting the death countdown (0xD24) to 0 (High).

The coins of part D are all spawned at the enemy's position; the split adds no spacing between them (High).

Row values (resource 530, class 17, 109 rows). Death disappear timer values: 1 (17 rows), 4 (8), 5 (7), 6 (7), 10 (5), 60 (1), 180 (11), 200 (2), 240 (2), 300 (14), 320 (3), 360 (6), 420 (1), 510 (14), 600 (11). The reanimating rows are 224 (Skeleton Archer): countdown 3600 to 18000 ticks, disappear timer 320; 226 (Dumb Skeleton): countdown 8000 to 12000 ticks, disappear timer 240; 227 (Skeleton Fighter): countdown 12000 to 18000 ticks, disappear timer 240.

Confidence: High for the order of the payout parts A to D and their conditions, the flyer grounding and the base-state write; Medium-High for the money-roll details (SPEC-0021 rules). The reanimation ranges and the death disappear values are confirmed on disc.

### 35. Reanimating enemies: class state 16 (High for the steps; Medium-High for the rise rule)

State 16 is entered only through the death hook (Behavior 34) for rows with the reanimation flag. Its animations are 58 (lying), 153 (lying, while flying) and 85 (rise). In this state, "flying" is true when either flyer test passes: the row's can-fly value is 1 and the retarget countdown is 0 or less, or the can-fly value is 2. The death hook of Behavior 34 uses only the first.

While its class state is 16 the enemy is not a hit candidate: the hit filter of the character class refuses class state 16 (High). While the reanimation countdown is also above 0, the dead test of streaming counts it as dead, so its kill bit is set on every such tick (Behavior 47).

Each tick:
1. Countdown phase (reanimation countdown at object offset 0xD14, 0 or more): play 153 when the enemy is flying, otherwise 58. Subtract 1 from the countdown and stop.
2. Otherwise (countdown below 0): look up the rise animation 85 in the enemy's animation table.
   - If the enemy has no animation 85, it heals to full health at once and goes to agro (state 19).
   - If the rise is not the current animation: play 85 when the clear-area test passes (the hero is at least 5.0 units away, and no other live member of the shared character pool is closer than 5.0 units to the enemy; the enemy itself is skipped). Otherwise play 58 or 153 as in step 1 and wait.
   - If the rise is current and not finished: wait.
   - When the rise is finished (animation progress reaches 0.99): set health to its maximum (with the heal notification) and go to agro (state 19).

Confidence: High for the countdown, the missing-animation path and the clear-area rule (5.0 units to the hero and to every other pool member); Medium-High for the distance measure, which is the shared distance helper (Open question 40).

A reanimating enemy never enters base state 16, so the dead-state update below never runs for it and its death disappear countdown (set by part C of Behavior 34) never runs down. It is never released for being dead; only the distance and cell rules of streaming release it (SPEC-0006) (High).

### 36. Dead state and corpse removal, non-reanimating enemies (High)

When the base actor state is 16, the per-tick update of the enemy does this every tick:
1. Set the actor's sub-object flag to 1.
2. Play 153 when the enemy is flying (either flyer test of Behavior 35), otherwise 58. A can-fly 1 enemy was grounded by the death hook (Behavior 34, step 2), so only can-fly 2 rows play 153 as a corpse. The request is repeated every tick but does not restart the clip: the request store keeps the main id when it is unchanged, and the hookup starts a main clip only when its layer does not already hold it (SPEC-0040, Behavior 3D, step 4). The only path that forces a restart, the take-hit hook's force-start (Behavior 37, step 13), stops at step 2 for a dead enemy. So the death clip plays once from the tick of death (High; Open question 85, resolved).
3. Once the death clip reaches a progress of 0.95, the disappear countdown goes down by 1 each tick. The countdown (object offset 0xD24) started at the row's death disappear timer.
4. Sink. When the enemy is not flying and the countdown, after that tick's decrement, is below the death disappear timer minus int(half the tick rate) (30 ticks at 60 ticks per second), the corpse's world position moves down by 1/180 unit along world up (0, 1, 0), and the sink value at object offset 0xD28 grows by the same 1/180. So a corpse sinks 1 unit in 180 ticks (3 seconds) and keeps sinking until it is released. A flying corpse does not sink (Open question 63, resolved).
5. When the countdown is below 0, the target and pool searches treat the enemy as gone. It is not targetable and it drops out of the live count; streaming then releases it (SPEC-0006).

The think routine is not stepped for a dead enemy (Behavior 1), so its class state stays at its previous value.

Confidence: High for steps 1 to 5, including the single play of the death clip in step 2.

### 37. Take-hit hook (High for the order, state changes and first-hit damage multiplier)

The hook runs on every hit, after the damage is computed. In order:
1. Script hold. If the enemy's script-hold word (object offset 0xC54) is nonzero, stop. The word is set when a level script hands a character whose class id is 6 back to its own behaviour, and cleared when it hands back any other class; it is cleared at creation. The same word also keeps the character out of the hit filter. For a class-17 enemy it stays 0 (High).
2. If the base actor state is 16 (dead), stop.
3. Health bar. The enemy becomes the target of the single enemy name and health bar, shown for 10 seconds (SPEC-0023). Its name comes from the spawn record (up to 31 characters).
4. Grab release. If the enemy's grab phase word (0xC08) is 2 and the selected hero's base state is 53 (held), the hero's base state is set to 1 through its setter (High). The grab-and-throw machine (Behavior 15; SPEC-0029 Behavior 23) writes phase 2 at the moment it puts the hero in state 53, and phase 3 when it lets go normally. So any hit that reaches this step on the enemy that is holding the hero frees the hero at once. The hook does not change the phase word itself (Open question 84, resolved).
5. Ghost absorb. If the enemy's row is a ghost row (0x138 nonzero), the enemy is in its ghost phase (0xC38 is 0) and no reveal is running (0xC48 is 0), the hit is absorbed: no damage, no reaction, no doubling, no hit count and no alert flag. The reveal starts instead (0xC48 set to 1, reveal timer 0xC4C = a whole number of ticks from row 0x154 to row 0x158, both included) and the hook stops (SPEC-0029 Behavior 33; rows 143, 215 and 222; Open question 65, resolved).
6. The attacker id is stored at object offset 0x428. If the class-state word is not 19 (agro), the incoming damage value is doubled, the class state is set to 19, and the enemy is added to the attack-manager list (Behavior 45). This transition occurs before the status/extra-flag reaction gates and the hit-lock damage gate in step 8. If the hit-lock timer is nonzero, the doubled value is not applied, but the state transition still occurs; later hits then see state 19 and are not doubled. Thus the multiplier is tied to entering agro, not necessarily to the first damaging hit.
7. Hit reaction. This happens only if status timer 0 (stun, object offset 0x808) is zero or less and the caller's extra flag is zero. The agro outer sub-state becomes 10 with a hit-reaction kind (Behavior 44):
   - Flying enemy (the row's can-fly value is not 0) with the retarget countdown below 1: clear the stored flag (0x8C0). Then by the row's flying hit value (0xAC): 1 gives kind 7; 0 gives kind 6; 2 gives kind 7 if the caller's second argument is nonzero, otherwise 6; any other value gives no reaction.
   - Otherwise: if the stored flag is 0, kind 0; if it is set, the flag is cleared and the kind is 5.
   A new reaction is ignored while outer 10 runs kind 2 with stun type 1 (Behavior 44).
8. The damage is applied only when the hit-lock timer (+0x7F4) is zero. Example: from class state 0 with HP 20 and incoming hit amount 4, a clear hit lock causes entry to state 19 and subtracts 8 HP; with the hit lock set, the same first hit still enters state 19 but subtracts no HP, and a later hit amount 4 after the lock clears subtracts 4 because the enemy is already in state 19.
9. The hit-recovery timer is set to 90 ticks (1.5 seconds).
10. If the enemy's own base actor state (0x144) is 25, the base actor state is set to 1 (High).
11. Face the attacker. The enemy is turned toward the attacker with weight 1.0 through the character's aim method. The attacker is found from its id: 0 or 1 is the selected hero; 2 is the second hero when that slot holds a live character, otherwise the selected hero; 3 and up index the list of extra attackers in order (3 is its first entry), falling back to the selected hero when the index is past its end (Medium-High).
12. The hit counter (object offset 0xA2C) goes up by 1. When the hit-count threshold (object offset 0xA30) is not -1 and the count reaches it, the enemy's invulnerable word (0x7F0) is set to 1, and from then on the hit filter refuses it. Creation sets the threshold to -1 and nothing else in the boot code writes it, so in play the threshold is never reached (High; Open question 76, resolved). The same invulnerable word is set and cleared by a level-script command and by some bosses.
13. If status timer 0 (stun) is zero or less, the enemy's force-start word is set (SPEC-0040, Behavior 3D, step 3): on the next animation hookup the requested clips start again from their first key even when the same clips are already playing. So an unstunned enemy that is hit while its requested clip plays sees that clip begin again (High).
14. The alert flag (+0xABC) is set to 1. The pre-step consumes it (Behavior 2, step 3).
15. If the row's anger level (0x134) is nonzero, the anger counter (+0xC20) goes up by 1. The anger counter feeds the anger rule of Behavior 7, step 5d (threshold int(anger counter x 0.05)), and the normal choice clears it when it picks such a slot.

Confidence: High for the order and the state changes; Medium for the doubling reading; Medium-High for the attacker lookup of step 11.

### 38. State 14, retreat home and hide (High for the entry; Medium-High for the phases)

Entered only by the distance-hide gate (Behavior 24). The state uses a phase word at 0xC04.
- Phase 0: play 81 until it finishes, then go to phase 1.
- Phase 1: walk home with animation 125. When no route to the hero exists and the enemy is within 3.0 units of its home point, it enters state 8 hiding (Behavior 40), and on that tick its position is set to its home point (a jump of up to 3.0 units; Medium-High). When the hero is within 5.0 units (and a route to the enemy's own position exists), or animation 0 is current, it plays 82 and goes to phase 2.
- Phase 2: when 82 has finished: if the hero is within 20.0 units, go to agro (state 19); otherwise run the setter with 0.

### 39. State 17, map search (Medium; arrival tests Medium)

Entry: the map search of Behavior 9 runs each tick when the row's map search flag (0x11C) is nonzero and its cooldown is zero. The chosen target is an object index.

Each tick:
1. If the hero is closer than the row's greedy hostile range (0x168), go to the default state (step 6).
2. Type 1 target: look up the object by its index. If it is missing, reset (step 6). Otherwise let d be the distance from the enemy to the object.
   - If d is below 1.0 (arrival): if the object's type word is below 0xBF or its subtype is 0x11, consume it, setting its consume flag to 4 for a type 1 target (not confirmed). Clear the target index and play the idle pick (Behavior 8).
   - Otherwise steer toward the object and play the locomotion pick.
3. Type 0 target: the same as type 1, except that the consume flag is 0 (not confirmed).
4. Other types: do nothing.
5. After steering or arrival, if the target index is still set, stay in the state. Otherwise reset.
6. Reset: the default state, with the target index and the object index both set to -1.

Rows with a map search type: type 0 on rows 156, 157, 158, 231 and 232; type 1 on rows 152 and 153; types 2 to 8 on rows 225 (type 2), 197 (3), 198 (4), 201 (5), 199 (6), 216 and 217 (7), and 213 (8).

Confidence: High for the rows; Medium for the arrival and consume tests.

### 40. States 8 and 9, emerge and approach a named target (High for state 8; Medium for state 9)

State 8 pre-step (sub-state word at 0xBC0):
- Sub 0: if the hero's distance is below the spawn record's f32 at 0xC0 times F1 (Behavior 18), set sub 1.
- Sub 1: the emerge animation: 165 for flyers; otherwise by the spawn record word at 0xC8: 0 gives 10, 1 to 4 give 133 to 136, any other value gives 10 (Behavior 20). If the animation is missing, go to agro (19). Otherwise play it and set sub 2.
- Sub 2: when the animation has finished, go to agro (19).

State 8 handler (sub 0): if the enemy is within 0.35 units of its home point, or the row's distance hide flag (0xA0) is nonzero, it stands at home playing the hide pick (9, or 128 to 131 by the same selector word) and stops steering. Otherwise it walks home (gait 0).

State 9 handler: the target is the pool entry whose id is at +0xBC of the spawn record. If it is missing, go to state 5 (wander). Otherwise:
- Turn to the target's movement mode and steer toward it.
- If the distance is at least the spawn record's +0xC0 value, keep walking.
- Else, if +0xC4 is zero, play the idle pick and stop.
- Else, if the distance is below 6.0, play animation 34 and count the timer at +0xB10 down from 60 ticks.
- Otherwise walk toward the target.

Confidence: High for the state 8 pre-step, the animation choice and the handler; Medium for the state 9 timer.

### 41. States 20, 22 and 23 (High)

- State 20: play the idle pick (Behavior 8).
- State 22: query animation 75. If refused, go to the default state. If it is not current, play it. When it finishes, go to the default state.
- State 23: query animation 81. If refused, go to the default-state value of the second block. If it is not current, play it. When it finishes, go to that same default-state value.

### 42. Attack execution, state 18 (High for the flow; Medium for the hit event and the aim)

Entry: only from the map search per tick (Behavior 9), on two paths. The agro attack choices (Behaviors 7 and 14) never enter state 18. Each tick:
1. If animation 137 is not current, play it.
2. If it is current and not finished, aim at the target (the pool entry for the target id) with blend 0.1. When animation event 32 fires and the sub-type fired flag is 0, set the flag and run the attack sub-type (Behavior 43).
3. When the animation finishes, set the recovery countdown to a whole number of ticks drawn from the row's attack timer min (0x68) to its max (0x64), both included, and go to agro (state 19).

This path runs the map-search spells only; every attack slot, melee or missile, runs through the attack sequencer of Behavior 13. Confidence: High for the entry and steps 1 and 3; Medium for the blend and the event number.

### 43. Attack sub-types 0 to 6: spells (High)

The sub-type is chosen by the attacker's row map search type (the type that the search of Behavior 9 found). Each sub-type looks up its target: the enemy's current target id (the pool entry of the target id). Every sub-type first checks the target: if the target is missing, nothing is spawned and no damage is dealt. For map-search type 2, the picker scans other characters in the active region list, excludes candidates playing animation 4, and keeps candidates whose current HP is below maximum. It also requires them to be within the row's map-search distance and pass the sight-line checks. It does not filter candidates by class state or by the reanimation countdown. A class-17 enemy in state 16 with a death clip other than animation 4 is therefore eligible while it remains in that list and is damaged.

Each sub-type submits one spell projectile to the enemy projectile pool (SPEC-0029 Behavior 9):

- launch point: the enemy's sample point; aim point: the target's position + (0, 1.5, 0);
- velocity: unit(aim - launch) x 1.0, so a spell flies at 1.0 unit per tick;
- lifetime: trunc(distance from the launch point to the aim point) + 5 ticks, so the spell reaches its aim point before it expires;
- damage word 3.0 (unused: spell kinds deal no damage) and radius 1.0;
- visual kind 23 (the second constant of every builder);
- effect ids: the table's first word (0x848 to 0x84E), then -1, 412, -1, 412, 412;
- owner: the enemy; element: the row's 0x3C value; area duration and radius: the slot's values at 0x44C + 4k and 0x45C + 4k;
- the target id is stored in the slot's target field, and the spell touches only that character.

| Sub-type | Map search type that selects it (row) | Projectile kind | First effect id | Effect on contact with the target (SPEC-0029 Behavior 9) |
| --- | --- | --- | --- | --- |
| 0 | 2 (row 225) | 2 | 0x848 | Sets the target's word +0xD14 (the reanimation countdown, Layout section 1) to -1 |
| 1 | 3 (row 197) | 3 | 0x849 | Applies status 4 (strength) |
| 2 | 4 (row 198) | 7 | 0x84E | Applies status 11 |
| 3 | 5 (row 201) | 4 | 0x84A | Applies status 5 (speed) |
| 4 | 6 (row 199) | 5 | 0x84B | Sets the target's HP to its maximum |
| 5 | 7 (rows 216, 217) | 5 | 0x84C | On spell contact, sets the target's HP to its maximum. At build time, if a projectile slot is free, the attacker first deals its missing HP (maximum minus current) to the target and submits the spell. If the pool is full, neither damage nor projectile occurs. |
| 6 | 8 (row 213, Minion Wizard) | 6 | 0x84D | Re-picks the target's variant and monster row (SPEC-0029 Edge cases, run-time variant re-pick) |

Map search type 7 selects sub-type 5 directly (Behavior 39 and Behavior 9). The spell releases its slot on contact with its target, or on ground or world contact (SPEC-0029 Behavior 9). The slot's projectile type field (0x43C + 4k) is not an input of the spell builders; it is the visual kind of the missile spawners of Behavior 13.

Confidence: High for the table (confirmed on disc), the speed, the aim, the lifetime, the contact effects and the type-2 candidate predicates; Medium for the link of kind 2's word to the reanimation countdown.

### 44. Agro machine, class state 19 (Medium-High; High for the writers of the outer sub-state)

Words (object offsets): outer sub-state 0xAF8, inner sub-state 0xB30, sequencer state 0xB2C, hold timer 0xB34, strafe lock 0xCB0, route flag 0xAFC, desired distance 0xE40, retreat distance 0xE44, recovery countdown 0xB18, defence hold 0xAB8, hit-reaction kind 0xB28. d = distance to the hero; dir = unit vector from the enemy to the hero. "Gait n" plays word n of the current gait animation set (gaits 5 and 6 are swapped in the lookup: gait 5 plays word 6 and gait 6 plays word 5).

Each agro tick, in order:

1. The recovery countdown goes down by one when positive.
2. If the base actor state is 20 (blocking) and the outer sub-state is neither 7 nor 8, the base actor state is set to 1.
3. Reactions and choices, in this order: missile defence, melee defence, lost-sight test, normal attack choice (Behavior 7), counter-attack choice (Behavior 14), distance-hide gate (Behavior 24).
4. The outer sub-state switch (below).
5. The agro tail: route flag and leash (Behavior 23).

Then the class-19 handler runs: the defence hold goes down by one; outer 0 plays the idle pick; the hold inner sub-states of outers 1, 3 and 4 face the hero (turn speed row 0x84, flyers row 0x88) and play their hold animation; the moving inner sub-states, outers 9 and 11, and outer 12 while it walks run the pursuit step (SPEC-0029 Behavior 25), which plays the gait animation.

**Outer sub-state entry (High).** One entry routine takes (event, parameter), does the entry work and stores the outer sub-state the event names: event 1 rush (inner 1, gait set 21, 21, 21, 23, 55; flyer set 101, 102); event 3 keep distance (inner 4, set 24, 24, 24, 26, 55; flyers 107, 108); event 4 strafe (inner 10, hold 0, set 28, 28, 28, 32, 55, 30, 29; flyers 103 to 106); event 2 attack (sequencer state = parameter, hit list cleared), or the wait in line instead (outer 12, set 2, flyer 98) when the attack manager says wait and the approach routine is not 3 (Behavior 45); event 10 hit reaction (kind = parameter; Hit reactions below); event 11 confused (set 69, flyer 69), ignored in outer 2. The flyer-set words are written only when row Can fly (0x94) is non-zero. This is not a hero routine: the hero has no action-state event switch (SPEC-0036).

**Outer sub-states.**
- 0 (set up): outer = the row's approach routine (0x38: 1, 3 or 4 on disc; any other value is kept as the outer sub-state, which has no case, so the enemy never moves by archetype or attacks by the normal choice); route flag = 1; play the idle pick.
- 1 (rush; entered with inner 1 and gait set 21, 21, 21, 23, 55): if the route flag is 0: gait 0 and steer to the hero; inner = 1 when within 1.0 unit of the current walk waypoint, otherwise 0. With a route: inner 0: gait 2 when d > 2 x row 0x4E0, else gait 0; steer to the hero; d < desired gives inner 1. Inner 1 (hold, faces the hero, animation 22, flyers 155): d < row 0x4DC gives inner 2; d > row 0x4E0 gives inner 0. Inner 2 (back off): gait 3, steer to the point hero - dir x (row 0x4E0 + 2.0); d > desired gives inner 1.
- 3 (keep distance; entered with inner 4 and gait set 24, 24, 24, 26, 55): with no route, as outer 1 but with inner 4 near the waypoint and 3 otherwise. Inner 4 (hold, aim blend 0.25, animation 25, flyers 156): d > row 0x4E8 gives 3; else d < row 0x4E4 gives 5; else d < row 0x4EC gives 6. Inner 3: gait 0, steer to the hero; d < desired gives 4; then d < row 0x4EC gives 6. Inner 5: gait 3, steer to hero - dir x row 0x4E8; d > desired gives 4; else d < row 0x4EC gives 6. Inner 6 (retreat): gait 2, steer to hero - dir x (row 0x4F0 + 2.0); d at least the retreat distance gives 4.
- 4 (strafe; entered with inner 10, hold 0 and gait set 28, 28, 28, 32, 55, 30, 29 as words 0 to 6, so with the swap above gait 5 plays 29 and gait 6 plays 30): the strafe lock goes down by one when positive. With no route, inner 10 near the waypoint and 7 otherwise. Inner 7: gait 0, steer to the hero; d < desired gives hold 0 and inner 10. Inners 8 and 9 (circle): gait 6 or 5, steer to own position + p x (-2.0 for 8, +2.0 for 9), where p = normalize(dir x (0, 1, 0)), the enemy's right as it faces the hero, so inner 8 circles to its left and inner 9 to its right (Open question 82); the hold goes down by one and below 1 gives inner 10; the blocked flag (0xCC4) gives inner 10 and a strafe lock of 120; then d > row 0x4F8 gives 7, else d < row 0x4F4 with the lock below 1 gives 11. Inner 10 (hold, faces the hero, animation 31, flyers 132): the hold goes down by one; while it is still positive, the distance tests of inner 8 apply; otherwise r = int[1, 3]: 1 gives 10, 2 gives 8 (10 while locked), 3 gives 9 (10 while locked), the hold = int[60, 180], and then the distance tests run (they win). Inner 11 (back off): gait 3, steer to hero - dir x row 0x4F8; when d > desired or the enemy is blocked (blocked also sets the lock to 120): hold 0, inner 10.
- 2: the attack sequencer (Behavior 13).
- 5 and 7: melee dodge and melee block; 6 and 8: missile dodge and missile block (below).
- 9 (search after lost sight): while the sight line is blocked: the search point is the last-seen point (plus dir x row 0x4E4 for approach 3, plus dir x row 0x4F4 for approach 4). Within 0.5 units of the point the search counter (0xB54) goes up by one; otherwise gait 2, steer there, and the counter is set to 0 when a route exists, else it goes up by one. A counter above 120 runs the setter with 0. When the sight line is clear: outer 0.
- 10: hit reactions (below).
- 11 (confused; status 6 active, set by the enemy status tick, SPEC-0029 Behavior 27): the retarget countdown goes down by one; the enemy aims at a random horizontal direction (x and z each float[-1, 1), normalized) with blend 0.05 (data word 0x2DD540) and plays 69; when d is at most own radius + hero radius + 0.25, the normal attack choice runs. When the status has ended: outer 0.
- 12: wait in line (Behavior 45).

**Melee defence (outers 5 and 7).** Not in outer 10, and not in outer 2 while the sequencer is in a state other than 0, 1 and 5. It is latched on the edge of the hero starting a melee swing (latch word 0xB88), so it is evaluated once per swing. It acts when the defence hold is below 1, the enemy is within the hero's reach and inside the hero's facing arc. Two rolls r1, r2 = int[1, 100]. By the hero's swing class (quick, normal, heavy; SPEC-0036) the block chance is row 0xEC, 0xF0 or 0xF4 and the dodge chance row 0xF8, 0xFC or 0x100. r1 at most the block chance gives outer 7; otherwise r2 below the dodge chance gives outer 5. On either, the defence hold = row 0xE0. Outer 5 (dodge): animation 55 (flyers 140) to its end, then the defence exit. Outer 7 (block): while the hero swings, face the hero, play the block 56 (flyers 150) and set base state 20; a pending block-hit flag plays 126 (flyers 151) once; when the swing is over, the defence exit.

Swing classes and gates (R-0017, High for the mapping and the order; Medium for the reach and arc meanings). The hero's swing class comes from the hero's attack sub-state word (+0xF90) and its variant word (+0xF80: 0 Tor, 1 Roberta). For Tor, sub-states 5 and 6 are quick (class 0), 2, 3 and 9 normal (class 1), 4, 7, 8 and 12 heavy (class 2); sub-state 10 and every other value have no class and never block or dodge. For Roberta, sub-states 5, 6, 7 and 10 are quick, 2, 3, 8, 11 and 12 normal, 4 heavy, and 9 and 14 have no class. A class maps to its block and dodge rows: quick 0xEC and 0xF8, normal 0xF0 and 0xFC, heavy 0xF4 and 0x100. "Within the hero's reach" is the distance between the enemy and the hero (the offset routine of the hero query) being below the hero object's float at +0x14B8. The facing arc is the dot product of the hero's facing with the negated offset, which must exceed the hero object's float at +0x14B4. Both floats are set when the hero object is initialised (spawn and every respawn) and nothing else writes them in play: the reach is 8.0 units and the arc cosine is 0.5, a half-angle of 60 degrees, the same for Tor and Roberta and for every swing class (High for the values and the writer; Medium for the exact meaning of the offset in the dot product). Test vector: an enemy 7.9 units away whose negated offset has a dot of 0.51 with the hero's facing can block or dodge; at 8.0 units, or at a dot of 0.5, it cannot.

**Missile defence (outers 6 and 8).** Not in outer 10, not in outer 2 with the sequencer in state 5, and only when the defence hold is below 1. For each live hero shot (up to 6): the path runs from the shot's position to its position + velocity x row 0xE0 (the look-ahead in ticks). If the path meets the enemy's body capsule: int[1, 100] < row 0xE4 gives outer 6, then int[1, 100] < row 0xE8 gives outer 8 (the later one wins), and the defence hold = row 0xE0. Outer 6 (dodge): animation 53 or 54 by int[0, 1] (flyers 138 or 139) to its end, then the defence exit. Outer 8 (block): play the block 56 while a hero shot is live; when none is live, the defence exit.

Defence exit: outer 0, base state 1, defence hold = int[int(row 0x104), int(row 0x108)].

**Lost-sight test.** Not in outer 9, and not in outer 2 with a grab-and-throw slot (flag 0x08): when the sight line to the hero is blocked, outer 9 starts and the last-seen point is the hero's position.

**Hit reactions (outer 10).** The take-hit hook chooses the kind (Behavior 37, step 7); a blocked charge chooses kind 2 or 3 (Behavior 13, state 9).
- Kind 0: animation 57; for a flyer, the grounded countdown starts and the navmesh is picked again (Behavior 6); then outer 0.
- Kind 5: animation 127, then outer 0.
- Kind 6: the enemy stops; animation 152, then outer 0.
- Kind 7: animation 76 until the speed is at most 2.0, then 74 when present, then outer 0.
- Kind 2 (stun): animation 60 (flyers 158) until the stun countdown (0xBF0) is 0, then 61 (159), then outer 0. The outer-10 handler only reads this countdown; the only decrement instruction is in attack-sequencer state 11, which has no writer and is not stepped while outer 10 runs (Behavior 13). Thus this reaction’s countdown does not expire by itself (Edge cases).
- Kind 3: animation 62 (160), then 64 (161), then outer 0.
A new reaction is ignored while outer 10 runs kind 2 with stun type 1. Starting a reaction while the sequencer is in state 14 sets the recovery countdown and the slot cooldown and clears the selected slot.

### 45. Attack manager and wait in line (High for the list, the budget and the order; Medium-High for the wait-in-line steps)

There are no numbered attack slots. One shared list holds every class-17 enemy that is in agro. A weight budget decides each frame which of them may start an attack. A waiting enemy is held back from the sequencer start only.

- List: run-time memory at 0x35B320 (not on disc). A 16-byte header (count; an unused word; the weight used this frame; the capacity this frame), then up to 20 entries of 16 bytes (enemy id, a word 1, a distance float that starts at -1.0, the wait flag that starts at 0).
- Grant (add): on entering class state 19 (setter for state 19) and in the take-hit hook when it sends the enemy to agro. An id already on the list is refused, with a warning, and so is a 21st enemy. A refused enemy still fights, because a missing entry reads as "not waiting".
- Run: once per frame in the main frame loop, only while the list exists and a hero is selected. All enemies, of every area, share one list.
- Capacity C = 3.0 x (w + a), from the selected hero's gear, read again each frame.
  - w = weapon item id - 98 for Tor (Swords 98 to 105, so 0 to 7) or id - 111 for Roberta (wands 111 to 118, so 0 to 7).
  - a = body item id - 119 for Tor (armour 119 to 125, so 0 to 6) or id - 127 for Roberta (dresses 127 to 134, so 0 to 7). Item 126 gives 0 for either hero.
  - An empty slot gives -1. So C runs from -6 (nothing equipped) through 0 (starting gear) to 39 (Tor) or 42 (Roberta).
- Release (remove), in two passes each frame, with no timeout:
  - First pass: remove an entry whose enemy no longer exists, is not in class state 19, or meets the dead test. A kept entry stores its enemy's distance to the selected hero.
  - Third pass: remove an entry whose enemy no longer exists.
  - Removal copies the last entry into the freed place and shrinks the count. The pass then goes on with the next place, so the moved entry is skipped for that frame: its distance and wait flag keep the old values, and its weight is not counted.
  - Leaving agro releases the entry on the next frame. That covers retreat (state 14), death and reanimation (state 16). A released character in base state 42 (parked) keeps its entry while its class state is still 19. It is then measured from where it was parked, so it normally sorts last, but its weight still counts (Medium).
- Order (second pass): for i from 0 to n - 2, and for j from i to n - 1, swap entries i and j when both hold:
  - the enemy now in place i is eligible to move: not busy (sequencer states 0 to 3), not blocking, and its class-17 state-19 sub-state (+0xAF8) is not 10 (the hit-reaction entry sets this marker; it is distinct from outer-state word +0xB30);
  - distance j < distance i + 5.0 (strict).
  Entry i is read again after each swap. The result is not a plain sort. Within a group whose distances are less than 5.0 apart, the order turns over from frame to frame (Test vectors). A busy enemy is never moved by its own turn as i, so it keeps its place unless a free enemy before it swaps with it.
- Budget (third pass): used = 0. Walk the entries in order. Entry k waits when C < used and k > 0; otherwise it may attack. Then used += its row Difficulty value (0x3A: 0, 1, 2, 5, 8, 12 or 15 on disc). Its own weight is added after its own test, and the first entry never waits. So entry k attacks when the weights of the entries before it sum to at most C.
- Number of attackers: exactly one while C is negative. At C = 0 it is one, plus any entry that has only weight-0 enemies before it. With weight-8 enemies it is 1 at C = 0, 2 at C = 15 and 5 at C = 39. It is never more than 20. There is no per-class limit and no limit per attack type.
- Wait query: an enemy waits when its entry's flag is 1; an enemy not on the list never waits. Only three readers use it: the sequencer start (outer event 2), the wait-in-line state, and the aim scatter of spawned shots.
- Refused enemy (wait in line, outer sub-state 12): when outer event 2 would start the sequencer (Behaviors 7 and 14) while the enemy waits and its approach routine (0x38) is not 3, the event starts outer 12 instead. Approach-3 enemies are never held back; their shots get 3.0 more aim scatter while they wait (Behavior 13).
  - Entry: gait set 2 (flyers 98), and a stand-off point P is set as the move point. P = hero position + (u - R/2, 0, v - R/2), where u and v are drawn independently from the integers 0 to trunc(R) - 1 (0 when trunc(R) is 0), and R = row 0xB0 (on disc 0, 5, 10, 12, 15, 20, 25, 30, 40 or 50). If P does not locate on the enemy's navmesh, P moves halfway toward the enemy, up to 10 times; after the 10th failure P is the enemy's own ground point.
  - Each tick, while the countdown (object word 0xBF8, not reset on entry, normally 0) is 1 or more: face the hero, play the idle pick, and count down by one. The enemy keeps moving to P.
  - When the countdown is 0 or below and the enemy has arrived at P: play 73 once (163 for flyers). When it ends, or at once when the kind has no such animation, set the countdown to int[600, 800] and pick a new P in the same way with R = 50.0 (a fixed constant). Before arrival nothing more happens.
  - Every tick, after these steps: when the enemy no longer waits, outer 0 starts again (approach, then the normal attack choice).
- Hero switch: on the next frame the capacity comes from the new hero's gear and the distances are to the new hero. Entries are kept.
- The hero's own health is not tested. Victory poses come from the melee strike (setter 10), not from the manager.
- Animation pick (Behavior 8): idle variants 115 to 119 and walk 97. Flying variant 142.
- Animation sets: the six words written by the set helper are the gait animation set of the current outer sub-state (Behavior 44).

### 46. Row data for attack execution and map search (High for the counts; confirmed on disc)

- 109 class-17 rows in resource 530, out of 259 rows in the table. The counts in this section are for these 109 rows only.
- Attack slots per row, counting only the slots whose type is not 0xFFFF (31 of the 109 rows have gaps between their used slots, so the highest slot number is not the count): 0 on 2 rows, 1 on 15, 2 on 42, 3 on 35 and 4 on 15. Slot types: melee on 208 used slots, missile on 56 used slots.
- Projectile type field values over the 264 used slots of these 109 class-17 rows: 0 on 222, 23 on 38, 10 on 2, 6 on 1 and 1 on 1.
- The map search flag is set on 15 rows; their types are listed in Behavior 39.
- Reanimating rows: 224 (Skeleton Archer; approach routine 3; 50 hit points), 226 (Dumb Skeleton; approach routine 1; 50 hit points) and 227 (Skeleton Fighter; approach routine 1; 75 hit points).
- Steal rows (all 259 rows; steal fields of SPEC-0029 Layout section 4, rules in SPEC-0029 Behavior 11a and 11b). Neopoint steal (selector 0) on row 152 (Thief, slot 2: 10 to 100 Neopoints, 50 %), row 153 (Rogue, slot 2: 10 to 100, 50 %) and row 154 (Thug, slot 3: 20 to 200, 50 %). Item steal (selector 1, 100 %) on row 136 (Crokabek, slot 2) and row 138 (Swamp Crokabek, slot 2). Row 137 (Mountain Crokabek, slot 2) has the steal bit with selector 0 and an amount range of 0 to 0 at 100 %, so its steal takes nothing. No other row has the steal bit.
- No per-area or per-act multiplier: no routine scales a common enemy's row values by the area, the act or the story stage. Enemies met later are harder only because other rows (types) are placed there, in larger numbers (SPEC-0029, Scaling).

### 47. Respawn and kill memory (High for the kill bit, the bind exclusion, the reanimation bypass and the release rule; Medium-High for the save link)

The streaming of characters into a block and the kill bits are specified in SPEC-0006 (character activation, streaming and kill memory). For common enemies they give these respawn rules:

1. A killed common enemy never comes back. When a bound character is found dead, the kill bit of its spawn record is set. Each world keeps 256 kill bits. A record whose bit is set is never bound again by streaming. The bits are stored into the world's own saved record (the per-world record of SPEC-0025) when the world is left, loaded from it when the world is entered, and saved with the game. So an ordinary enemy does not respawn on re-entry, after a timer, or after save and load.
2. Exception, reanimating rows: rows 224 (Skeleton Archer), 226 (Dumb Skeleton) and 227 (Skeleton Fighter) ignore the kill bit when they are bound. They rise again in place after their reanimation countdown (Behavior 35), and they are bound again after being streamed out and back in. While such an enemy lies with its countdown above 0, the dead test counts it as dead and its kill bit is set each tick; the bind ignores it (Open question 81, resolved).
3. An enemy released while alive (the hero walked more than 280 units away, SPEC-0006) is not killed. Its spawn record takes the character's current position and facing, and a later bind brings it back at that point at full health (the bind starts fresh from the monster row).
4. A forced bind of a record clears its kill bit (ActivateEntity, SPEC-0030, and a few engine routines such as the spider nest). DeactivateEntity kills permanently: it runs the death hook and sets the kill bit.
5. A game load does not flush the bits of the world being left into that world's saved record. Kills made since the world was entered are lost when a game is loaded, which matches loading an older save.
6. A new game clears every world's bits (the per-world records are reset).

### 48. Movement: locomotion animations, travel speed and turning (High for the mechanism and the animation ids; Medium-High for which id each state sets)

1. **Travel by root motion.** A ground enemy moves only by the root motion of the locomotion animation it is playing (SPEC-0040 Behavior 14D). Class 17 has no speed value of its own: the speed of each gait is the root travel of that enemy's own clip (per tick = the clip's root step of the current key times the clip speed; SPEC-0005, SPEC-0040 Behavior 14C item 1), so it differs from enemy to enemy and is not a row value. "Steer toward a point" in this spec sets the move direction only. Flying enemies add a row speed (item 4).
2. **Locomotion animation ids.** Entering a class state or an agro outer sub-state loads a set of animation ids, one per movement class (the gait words of Behavior 44); the movement step plays the id of the current class.
   - Class states 5, 7 and 8: class 0 plays 2, class 2 plays 21. Class state 6: class 0 plays 4, class 2 plays 21. Class state 9: 11. Class state 1: 20.
   - Agro outer sub-states: rush, keep distance and strafe use the gait sets of Behavior 44 (rush 21, 23 back, 55; keep distance 24, 26 back, 55; strafe 28, 32 back, 55, sides 29 and 30); outer 11 (confused) plays 69; outer 2 (the sequencer) plays 2 while it walks the map-search path (Medium).
   - Flyers use their own ids: 98 (wander), 99 (patrol), 100 (class state 9), 154 (class state 1), 101 and 102 (rush), 107 and 108 (keep distance), 103 to 106 (strafe).
   - Missing ids: the attack phases and the fidgets are checked against the table and skipped when missing (Layout section 6). Any other request of an id that the enemy's table lacks starts clip 0 of its main set (SPEC-0005, Lookup miss), and the enemy then travels with that clip's root motion.
3. **Ground speeds (examples, play rate 1, units per second).** Werelupe Basher rush (21) 23.58; Skeleton Fighter rush 15.01; Spyder walk (2) 11.26 and rush 9.38; Ixi Archer rush 20.0; Minion Warrior strafe forward (28) 12.5. The back ids (23, 26, 32) move along the facing's reverse and the side ids (29, 30) along the side axis, each at the root speed of its own clip. The clean side takes every speed from the enemy's clips, not from a table.
4. **Flyers.** On top of the root step, a flying enemy moves along its movement axis at 2 x Flying Idle Velocity (row 0x8C) units per second outside agro and 2 x Flying Agro Velocity (row 0x90) in agro (SPEC-0040 Behavior 14D item 7). Row speeds, idle / agro: Minion Grunt 6 / 14, Minion Archer 8 / 16, Crokabek 6 / 16 (Crokabek's flying clips carry no root travel, so these are its whole speeds).
5. **Attack speed words.** The wind-up, attack and recovery speed words of a slot (Layout section 2) are direct stores to the per-tick speed scale P by the class coroutine, before the same-update clip re-time (Behavior 1). The current slot value therefore sets the clip rate that tick; the later animation pass evaluates at that rate, and its updated root step is used by the following motion-record pass. Because the row value is stored after status processing, it replaces the P value accumulated from earlier status factors for that update.
6. **Chase stop.** While the enemy is in agro outer sub-state 2 with the sequencer in states 0 to 3, its step is discarded on any tick that it is within the sum of the radii (its record radius plus the active hero's record radius, 3D distance) of the hero (SPEC-0040 Behavior 14D item 4). It does not walk into the hero while it approaches, winds up and strikes.
7. **Turning.** Each tick the enemy's facing is set from its steered move direction M by its locomotion mode (the facing helper of the movement step, SPEC-0040 Behavior 8). Modes 3, 4 and 8 (back gaits) face -M. Modes 5 and 9 face up x M, and modes 6 and 10 (side gaits) face M x up, where up = (0, 1, 0). Every other mode faces M. The blend is the value the steering step passes (0.3 in the navigation step) for back and other modes. For side modes it is 0.1, or 1.0 when the current facing is already above dot 0.9 with the target (High for the rule; Medium for the mode numbers of the gait sets). For ground enemies the gait number of a sub-state is the locomotion mode; flyer forms use their own clip ids (Behavior 44). Test vector: enemy at (0, 0, 0), hero at (0, 0, 10). Strafe left (inner 8, gait 6, move (1, 0, 0)): facing (0, 0, 1), toward the hero. Strafe right (inner 9, gait 5, move (-1, 0, 0)): facing (0, 0, 1). Back step away (gait 3, move (0, 0, -1)): facing (0, 0, 1). The steering blend weight of the step that steers is the only other turn limit (for example 0.1 in the sequencer's facing step, Behavior 13). Ground Turn Speed (row 0x84) is not read for ground movement; row 0x84 (and row 0x88 for flyers) is read only as the aim turn speed of the agro hold sub-states (Behavior 44).
8. **Timing.** The AI requests the animation during the NPC update. The animation pass later in that same character pass starts it and evaluates its first root step; the next motion-record pass consumes that step. Thus the first step comes one tick after the request, then every tick moves one full step, with no acceleration (SPEC-0040 Behavior 14D item 5; High for the order, Medium-High for exact clip-start boundary cases).

Test vectors (synthetic, computed from the rules above): a Werelupe Basher that rushes for 60 ticks after its first step covers 23.58 units; a Spyder that walks for 120 ticks after its first step covers 22.52 units. Chase stop: enemy radius 1.0, hero radius 0.5, sequencer state 0, distance 1.4: no step this tick; distance 1.6: the step is taken. The same enemy at distance 1.4 in outer sub-state 1 (rush, not the sequencer) takes its step.

### Routines of other classes (not class 17)

The routines of Behaviors 10b, 16, 17 and 27 to 33 are not run by class 17. The class routine table (SPEC-0029 Layout section 3) gives their owners: Behavior 10b is reached only from the class-6 controller; Behaviors 16, 29 and 30 belong to class 7 (Whinny); Behavior 17 to class 21 (Miamouse); Behaviors 27 and 33 to class 34 (the live petpet controller, SPEC-0047); Behavior 28 to class 36 (Target Dummy); Behavior 31 to class 35 (the level-script path walk, SPEC-0056); Behavior 32 to class 29. They keep their numbers here for reference. A class-17 enemy uses the idle picker of Behavior 8 (holds of 120 to 600 ticks), not the idle coroutine of Behavior 28. Confidence: High for the owners.

### 10b. Group leader handover, class 6 (Medium)

Owner: class 6 (the paired-partner creature, SPEC-0029 Layout section 3). The handover is reached only from the class-6 controller; it is not class-17 behaviour.

Each enemy record keeps two cooldown counts (+0xA94 and +0xA9C) and a leader id field (+0xAA0). Each call decrements both counts by one. The handover search runs only when the second count, after its decrement, is below 1.

The search runs over the global list of characters:

1. Skip the enemy itself.
2. The other character must pass the leader's candidate-eligibility test. It rejects a character when its social-disable word (+0xC98) is nonzero; otherwise it rejects modes 0, 1, 2, 8, 9, 10, 11, 0x70 to 0x74, 0x3B, 0x3C and 0x6B in the mode word (+0x3B0), and rejects when its AI state word (+0xA34) is 3. A pass continues to the group and range tests. The operational filter is known; the design meaning of the field-value exclusions is not (Open question 60).
3. The other character must share the same group value as the enemy (the group value comes from a helper for each character).
4. The other character must be within the enemy's record range: a value from a helper for the pair must be below the float at +0x44 of the enemy's record.

On the first match, the search stops and the handover happens:

- The enemy's leader field (+0xAA0) takes the other character's id (the value at its +0x14C).
- The shared AI-state setter sets the enemy's state to 3, then sets the other character's state to 3.
- The other character's leader field takes the enemy's own id.

The pairing reset runs when the linked partner is missing, is no longer in state 3, or has a different group value; it also runs when the current pairing sub-state is 3. It sets the pairing sub-state to 0, requests AI state 0, and resets the second cooldown (+0xA9C) with a closed integer draw from 10 times the frame-rate word to 30 times that word. At the normal frame rate of 60, this is 600 to 1800 ticks (10 to 30 seconds). Separately, when the shared sub-step setter is called while the pairing sub-state is 0 to enter sub-state 1 or 2, it resets the first cooldown (+0xA94) with a closed integer draw from 5 times the frame-rate word to 12 times that word (300 to 720 ticks at 60 Hz). It then stores the requested sub-state. Calls made while the current sub-state is nonzero do not redraw this timer. The setter is part of the class-6 pairing behavior; the recovered reset is runtime behavior, not a level-module write. The apparent targetless call on the enemy is the shared AI-state setter with state 3, like the following call on the matched character (Open question 61).

### 16. Chase-to-point controller, class 7 (High for the entry, the pick base, the distances and the mode logic)

Owner: class 7 (Whinny, SPEC-0029 Behavior 5). The shipped monster row is resource-530 row 253, selected by type 109, variant 0; its default behavior is class 7. This controller is the Whinny's flee mode together with the lead entry of SPEC-0029 Behavior 5; the pool is the list of pens, and the words at 0x2DB268 and 0x2DB278 are the pens' entrance ids and occupied flags.

The Whinny main loop keeps one block per Whinny and picks one routine per tick, in this order: the penned routine when the penned flag (+0x30) is set; the lead routine when the lead flag (+0x34) is set; otherwise the wander routine (Behavior 29) when the flee selector (+0x0C) is 0, and this controller when it is not 0. Behavior 29 sets the flee selector to 1 when the hero comes within 5.0. Offsets below are the block's: the flee selector (+0x0C), the lead timer (+0x08), the wander mode word (+0x10), the point (+0x20, four floats), the lead flag (+0x34), the pen index (+0x38), the pen entrance id (+0x04), the run timer (+0x40) and the run mode (+0x44; 0 pick, 1 run to the step point, 2 run to the safe point).

Let v and d be the result of the shared distance helper taken from the hero to the Whinny: v is the horizontal unit direction from the hero to the Whinny (away from the hero) and d is the horizontal distance (SPEC-0029 Behavior 2; the helper sets the height part of the direction to 0 and gives the horizontal length).

Each tick of the controller:

1. Pen entry (runs first, on every tick of the controller). The pens are searched in order 0 to count - 1 (count at 0x2DB250, at most 4; the shipped setup calls provide counts 1 or 4). A pen accepts when the pen count is 1 or its occupied flag is 0, and the Whinny's position lies inside that pen's entrance navmesh (the inside test of SPEC-0018: x and z in a triangle, height ignored). On the first accepting pen the controller writes: flee selector 0; lead timer 30 ticks (0.5 s); point = the pen point; wander mode 0; lead flag 1; pen index; entrance id; occupied flag 1. The setup setters clear all occupied flags when they load the list. For a one-pen list, selection ignores the occupied flag; for a multi-pen list, a selected flag stays set through the lead and penned flow. These routines do not clear it on arrival or exit, so multi-pen claims last until a setter reloads the list. It sets the Whinny's navmesh to the entrance id, sets its move target to the pen point, and ends the tick. From the next tick the lead routine runs; it reads the lead timer, the point and the pen index. The run mode (+0x44) is not written here, and nothing reads it again, because a Whinny that starts to lead stays in lead until it is penned and a penned Whinny never flees again.
2. Otherwise the Whinny runs (movement mode 2). If d is above 8.0 the flee selector is set to 0 (8.0 exactly does not), so the wander routine runs from the next tick; the run step below still runs this tick.
3. Run step by mode:
   - Mode 0 (pick). Candidate = the Whinny's position + 4.0 x v, a point 4.0 units from the Whinny directly away from the hero. If the candidate is walkable (the walk-mesh query returns nonzero) and the Whinny's blocked flag (character +0xCC4) is zero: mode 1, run timer = the whole part of (0.25 x the frame-rate word), point = candidate. Otherwise: mode 2, run timer 25 ticks (0.417 s), point = a far centroid pick of radius 10.0 from the hero's position over the Whinny's own navmesh (SPEC-0029 Behavior 28: random triangle centroids until one lies more than 10.0 from the hero horizontally). Then it steers to the point.
   - Mode 1 (run to the step point). The run timer goes down by 1. If it is then below 1, or the horizontal distance from the Whinny to the point is below 2.5, or the blocked flag is nonzero: mode 0 and the blocked flag is cleared. It steers to the point in every case.
   - Mode 2 (run to the safe point). The run timer goes down by 1. If the horizontal distance to the point is at least 2.0 and the timer is at least 1, it keeps running. Otherwise mode 0 and the blocked flag is cleared. It steers to the point in every case.

Distances to points are horizontal: the routine sets the height difference to 0 before it takes the length, the same rule as the shared helper.

Constants: 30 ticks (lead timer), 4.0 (step length), 8.0 (flee end), 0.25 (times the frame-rate word, for the mode 1 timer), 2.5 (mode 1 arrival), 10.0 (safe-point radius), 25 ticks (mode 2 timer), 2.0 (mode 2 arrival).

With the frame-rate word at 60 (its start-up value, SPEC-0007), the mode 1 timer is 15 ticks (0.25 s).

Confidence: High for the routine order, the pen entry, the pick base and the distance rule; High for the constants and the mode transitions.

### 17. Approach controller, class 21 (High)

Owner: class 21 (Miamouse). The shipped monster row is resource-530 row 248, selected by type 104, variant 0; its default behavior is class 21. This controller is the flee state, main state 2, of SPEC-0029 Behavior 28, which is the full rule; this section restates it with the words of this spec. The done word at +0x00 is the Miamouse main state word, and the mode word at +0x04 is the flee sub-state. The flee entry of Behavior 28 sets main state 2 and sub-state 0, so a new flee always starts in mode 0.

It keeps the point at +0x20. The touched flag is the word at +0x238 of the character's motion record (the sub-object at character +0x16C); it is set when the motion is blocked (SPEC-0029 Behavior 28). Every tick of the state the Miamouse runs (movement mode 2) and steers to its point.

Let v and d be the result of the shared distance helper taken from the hero to the Miamouse: v is the horizontal unit direction away from the hero and d the horizontal distance. Let p be the horizontal distance from the Miamouse to its point. A far centroid pick of radius R takes random triangle centroids of the Miamouse's navmesh until one lies more than R from the hero horizontally (SPEC-0029 Behavior 28).

Mode 0:

1. If the touched flag is set: mode 1 with a far centroid pick of radius 10.0, and play clip 0.
2. Then, if d is at most 20.0: candidate = the Miamouse's position + 4.0 x v (4.0 units directly away from the hero). If the candidate is not walkable: mode 1 with a new radius-10.0 pick. If it is walkable: steer to the candidate this tick and leave the record unchanged.
3. If d is above 20.0: mode 2 with a far centroid pick of radius 30.0.

Steps 1 and 3 can both run in one tick. With the flag set and d above 20.0, the last write wins, so the controller ends in mode 2 with the radius-30.0 point (Edge cases).

Mode 1:

1. Steer to the point.
2. If p is at least 0.5 and the touched flag is clear, keep running (no change).
3. Otherwise, if d is below 20.0: mode 0. If d is 20.0 or more: mode 2 with a new radius-30.0 pick.

Mode 2:

1. Steer to the point.
2. If p is below 0.5, or the touched flag is set: the main state becomes 0 (idle). The mode word stays 2; it is not read again until the next flee entry resets it to 0.
3. Otherwise keep running. No hero distance test is made in this mode.

Constants: 20.0 (flee band), 10.0 and 30.0 (pick radii), 4.0 (step length), 0.5 (arrival).

Confidence: High for the modes, the constants, the arrival test and the exits.

### 27. Class-34 main loop and state selector (Medium)

Owner: class 34 (SPEC-0029 Layout section 3), not class 17. Class 34 is the controller of the live (summoned) petpet (SPEC-0047).

The main loop runs once per enemy and takes one step per tick. Its state word starts at 0. Each tick:

1. The selector (below) runs and may change the state.
2. The handler of the current state runs. State 0 plays animation code 0. States 1, 2, 3 and 6 are described below; states 4 and 5 are Behavior 33.
3. When a global flag object is clear (read at a record offset; its meaning is open), the cooldown word at +0xE34 goes down by 1.
4. A per-tick update runs (not described here) and the loop yields one tick.

Selector words: cooldown A at +0xE30 (a request for state 4 when nonzero), cooldown B at +0xE34 (ticks; set by the state 4 handler), and the stun or steering word at +0x238 of the actor's sub-object (Open question 10). Let s be the current state.

- If s is 3, nothing changes.
- If the stun word is nonzero: when s is not 5, the state becomes 3. Otherwise nothing changes.
- If the stun word is zero and cooldown A is nonzero: the state becomes 4 and cooldown A is cleared.
- If the stun word is zero, cooldown A is zero, and s is neither 4 nor 6:
  - If a record flag at +0x2C is nonzero, or cooldown B divided by (0.5 x tick-rate word x 60) is at least 1.0:
    - If cooldown B is below 1, the state becomes 5.
    - Otherwise, if the hero is in a jump state (hero states 7 to 11 and 43 to 46) or a second hero query returns nonzero (its meaning is Medium-Low), the state becomes 2 (follow). Otherwise, unless the state is already 1, the state becomes 1 with a new point (the hero's position plus a random offset of 3.0 to 10.0, with a random sign, on each of x and z), its timer unset and its hold 0.
  - Otherwise the state becomes 6 and the record flag at +0x2C is set to 1.

Test vector (synthetic): cooldown B = 0, record flag 0, stun word 0, cooldown A 0, state 0: the selector moves to state 6 and sets the flag. With cooldown A = 1 in any state other than 3, the state becomes 4.

Cooldown B is the petpet's remaining time in ticks (High): the loop lowers it by 1 per tick, and the selector's ratio r = cooldown B / (0.5 x tick-rate word x 60) is 1 at 1800 ticks, 30 seconds at 60 ticks per second. So the petpet plays its state-6 warning once when 30 seconds are left (the record flag at +0x2C is never cleared again), and leaves (state 5) when the time runs out.

States 1, 2, 3 and 6 (High for the steps; Medium for the meaning of the animation codes):

- State 1, wander near the hero: SPEC-0029 Behavior 12.8. The petpet reacts once with animation 8 to each new character that comes within 10.0, walks (animation 2) to its point, waits there 180 to 600 ticks, then draws a new point near the hero. While it waits it plays animation 7 when r is below 1, otherwise animation 1 or 6 by the roll of that section.
- State 2, follow the hero: SPEC-0029 Behavior 12.13. The follow point is the hero's position plus 4.0 times the hero's first basis vector plus 1.5 times its second; the petpet idles (animation 0), walks (2) or runs (3) by the gait bands 3.0, 1.5, 7.0 and 5.0.
- State 3, recover when stuck (entered when the touched flag is set): the petpet plays animation 0. Let c = dot(camera view vector at +0x20, unit direction from the camera point at +0x30 to the petpet). While c is at least 0.5 (the petpet is inside the 60-degree view cone) nothing else happens. When c is below 0.5, a point P = the hero's position + 10.0 x the horizontal unit reverse of the camera view vector (10.0 units from the hero toward the camera) is tested: the ground under P (probe started 8.0 above P) must lie within 2.0 of the hero's height. If it does, the petpet is placed at P, facing the hero, and the state becomes 1. Otherwise it stays in state 3 and tries again next tick. The selector never moves the petpet out of state 3.
- State 6, time warning: the petpet plays animation 7 until it has finished, then the state becomes 0.

Test vectors (synthetic):

- Cooldown B = 2000, flag 0, hero standing: r = 1.11, so the selector enters state 1. Cooldown B = 1500, flag 0: r = 0.83, state 6 and the flag is set; after animation 7 ends, state 0, and the next selector tick with the flag set enters state 1 (hero standing) or 2 (hero jumping).
- State 3 with the camera view vector (0, 0, 1) and the hero at (0, 0, 0) on flat ground at height 0: P = (0, 0, -10). The ground under P is at height 0, within 2.0, so the petpet is placed at (0, 0, -10) facing +z and goes to state 1.

### 28. Idle coroutine, class 36 (Medium)

Owner: class 36 (Target Dummy). This is the routine of SPEC-0029 Behavior 29: code 3 is the hit reaction, code 1 the twitch and code 0 the idle. Both specs draw the period as a closed range: both ends are included.

A separate coroutine plays idle animation codes for an enemy. At start it draws a period P in [400, 800] ticks (closed integer draw, both ends included; 400 and 800 are fixed constants) and sets a counter c to 1. Each tick:

1. If the actor is busy (a state query on the actor returns nonzero), it plays animation code 3, sets c to 1, draws a new period P, and yields.
2. Otherwise, if the query for the idle state passes, then:
   - when c is a multiple of P, it plays animation code 1 and adds 1 to c;
   - otherwise, if the query for code 1 returns zero, it plays animation code 0 and adds 1 to c; if the query returns nonzero, it adds 1 to c without a call.
3. It yields one tick.

Result: animation code 0 is played on most ticks, code 1 on one tick in every P, and code 3 while busy. The animation codes here are the call codes of the actor's animation control. They are not the idle variant indices 0x73 to 0x77 of Behavior 8 (Open questions).

### 29. Approach and attack bands, class 7 (controller record; Medium)

Owner: class 7 (Whinny).

This controller is the approach branch of the second main loop. The loop chooses a routine by three loop-local words, tested in order: flag A (if nonzero, Behavior 30 runs); flag B (if nonzero, a routine not described here runs); and a selector word that is kept with the controller record at +0x0C (if zero, this controller runs; otherwise another routine not described here runs). The offsets of flags A and B within the loop's block are not identified (Open questions). The controller record holds: a state word at +0x00, a timer at +0x08 (ticks), the selector word at +0x0C, a mode word at +0x10 (0 approach, 1 hold), the point at +0x20 (four floats), and claim words at +0x3C, +0x40 and +0x44.

Let d be the distance from the actor to the target. Each tick:

1. Close band. If d is below 5.0, the state word becomes 2, the flag at +0x0C becomes 1, the claim words are reset (+0x3C to 0, +0x40 to -1, +0x44 to 0), and the controller returns.
2. Re-pick check. If the mode is 0 or the timer is above 0, and the steering flag at +0x238 of the actor's sub-object is clear, go to step 3 with the current mode. Otherwise the mode is cleared, and a new point is set from the target's position helper, and the timer is drawn in [120, 400] ticks (closed integer draw, both ends included). Then go to step 3 with mode 0.
3. Band or wait:
   - If the mode is 0 and the steering flag is clear: when the distance measure of this step is above 10.0, the movement class is set to 2 (fast approach); otherwise when it is above 3.0, the movement class is set to 0 (walk approach); otherwise the mode becomes 1 (hold). Then the target point is set and the actor steers toward it.
   - Otherwise (mode nonzero, or steering flag set): the actor plays animation code 0 and the timer goes down by 1.

The distance measure in step 3 is read from the point's vector, not from the target distance of step 1 (Open questions). The 10.0 and 3.0 bands are therefore Medium-Low. The 5.0 close band and the 120 to 400 tick timer are Medium.

### 30. Wait-and-approach step, class 7 (Medium for the rule; Medium-Low for the reach value)

Owner: class 7 (Whinny).

Runs when flag A of the second main loop is nonzero. It works on a record with a timer at +0x08, a done word at +0x10 and a point at +0x20, the same layout as Behavior 29. The routine is called with two arguments only, so the record it uses is not identified (Open questions). Each tick:

1. If done is nonzero:
   - If the timer is below 1: the point is set from the target's position helper, the timer is drawn in [120, 400] ticks (closed integer draw, both ends included), and done becomes 0. Continue to step 2.
   - Otherwise the actor plays animation code 0, the timer goes down by 1, and the step returns (the wait).
2. Approach (done is 0): let d be the distance from the actor to the point. If d is at most 1.5, or the steering flag at +0x238 of the actor's sub-object is set, done becomes 1. Otherwise the movement class is set to 0. Then the target point is set and the actor steers toward it.

Test vector (synthetic): with done 0 and d = 1.5, done becomes 1 on that tick. With d = 1.6 it stays 0 and the actor walks.

### 31. Charge and wait coroutine, class 35 (Medium-Low)

Owner: class 35, the level-script path walk (SPEC-0029 Behavior 1; SPEC-0056).

A coroutine with two phases. Its caller is not identified in this section (Open questions).

Phase A (approach). Each tick:

- If a readiness test passes, phase B starts with the phase flag set to 1 and the wait counter at 0.
- Otherwise, if the movement-idle test returns zero, the actor moves to the target point. When the distance from the actor to that point is below 0.5, an arrival helper runs. If the movement-idle test returns nonzero, a second movement test is used instead, and when it returns zero the actor moves to the point again. If neither applies, the coroutine ends.

Phase B (wait, then charge). Each tick:

- If the phase flag is set, the actor plays the animation chosen by the idle-animation picker of Behavior 8.
- Otherwise, while the wait counter is below 3.0 times the tick-rate word (3.0 seconds, 180 ticks at 60 ticks per second), the counter goes up by 1 and the actor plays the animation returned by the same idle-animation picker (Behavior 8).
- Otherwise, when the animation-state query for code 30 (0x1E) returns zero, the actor plays animation code 30. Then the phase flag is set from a follow-up query (Medium-Low).
- Then, let d be the distance from the actor to the target point. If d is above 12.0, the coroutine returns to phase A. If d is at most 12.0 and the direction test (a dot product) is below -0.1, it returns to phase A. Otherwise phase B continues.

### 32. Patrol list walk, class 29 (Medium-Low)

Owner: class 29. No monster row has class 29 and nothing requests it, so this walk never runs on the disc (High; see Open questions).

The patrol list is a list of points in the actor's record (a count, then each point and its id, read by index). Before walking the list, the controller checks for a current navmesh. If none is set, it selects a mesh using world/region context, then installs that mesh and its starting point. The shared selector chooses a mesh whose triangle contains the actor's current position; it does not select a patrol waypoint. The exact source of the selector's world/region argument, the point-list owner and the setup path that fills it remain open (Open questions).

1. The point is passed through a walkable-snap helper (not described).
2. If an index is available, the controller's target is set to the point, and each tick the actor steers toward it until its distance to the point is below 1.0. At that tick the movement object is notified with the point and its id, and the arrival ends. If no index is available, no movement happens for this point.
3. After arrival, if more points remain, the actor pauses for 60 ticks (one tick per count from 0 to 59) and then one more tick. No pause follows the last point.

### 33. Class-34 states 4 and 5, approach and leave routines (Medium-Low)

Owner: class 34, the states 4 and 5 of the Behavior 27 main loop. These are not class-17 states: class-17 state 4 is the stunned state (Behavior 5).

State 4 (nearest eligible object; animation codes 2 and 4; speed 0.3):

1. Search the shared object list for the nearest candidate in the requested category. If no candidate is found, or its index is negative, the state becomes 0. If the selected candidate has type code 0x91, this handler makes no further change that tick. The search chooses by squared distance and returns the selected object's position offset as a normalized direction vector.
2. The next branch tests whether the second component of that returned direction vector is below 2.0, or whether the shared object-claim timer is below 2.5 seconds. These are not object-distance or object-extent tests. For finite nonzero position differences, a normalized component is between -1 and 1, so the strict 2.0 test passes; the seek branch therefore cannot be reached for those inputs. The shared timer advances by elapsed frame time (or one third of it while its slow flag is set), is reset when an object is selected, and times out after 5.0 seconds. If the branch is taken, an animation-state query for code 4 returning zero causes the actor to play animation code 4. Otherwise, when the follow-up query passes, the object claim is cleared, the state becomes 0, and cooldown B is raised to the distance-based time (seconds x tick-rate word x 60) when it is smaller.
3. If neither branch condition passes, the target point is set to the object's position, the actor steers toward it at speed 0.3 (a fixed constant), and it plays animation code 2.

State 5 (leave in view of the camera; animation code 3; constant 0.1). The test uses the camera (the object behind the run-time word 0x35C7B4), not the hero:

1. A view test runs: c = dot(the camera's vector at +0x20, the unit direction from the camera's point at +0x30 to the actor) must be at least 0.5, that is, the actor is inside a cone of 60 degrees half-angle about the camera's view direction (the same camera vectors as the aim-scatter cone of Behavior 13). The steering flag at +0x238 of the actor's sub-object must also be clear.
2. If either test fails, a release helper runs on the shared object pool at 0x35C7B0 and the handler returns, leaving the state unchanged (the release ends the pet; Medium, SPEC-0047).
3. Otherwise the actor steers along the horizontal reverse of the camera's +0x20 vector (toward the camera) with the fixed constant 0.1 and plays animation code 3 (Medium-Low for the direction).

## Edge cases and error handling

Class 17:

- Notice band: a hero closer than the band's near edge is noticed as soon as roll A and the sight gate pass, with no chance roll (Behavior 3, step 5). A hero beyond the far edge is never noticed. With the sight line blocked, the near and far edges use F2, which is 0.0 in some hero action states (Behavior 18), so the blocked band is then empty.
- Notice interpolation uses the un-scaled band values while the range test uses the scaled ones (a reproducible quirk).
- Halving: when the line-of-sight test fails and the Always-has-line-of-sight flag is 0, the band is recomputed from the un-halved values, which discards any socialize halving (reproduce as written).
- Charge distance draws: the charge distances of all four slots are drawn from the selected slot's charge range (reproduce as written). At creation the selected slot is -1, so the draw reads the two row words just below slot 0's charge range: 0x258 (the last charge-type word, read as a float) and 0x268 (reproduce as written).
- Counter charge: the counter-attack choice does not redraw the distances, so a counter charge runs to the charge distance of the last draw, which used the range of the slot the normal choice picked last (or the creation-time words). A slot with a charge range of 0 to 0 picked last gives a counter charge of length 0.0, which ends at once unless the hero is already within the melee range (Test vectors, Poison Spyder 6).
- Trigger range is strict on both sides: a hero exactly at the trigger minimum or maximum does not trigger the slot.
- Normal attack trigger chance endpoints: the roll is an integer from 1 to 100 and must be strictly below the integer threshold. Thus threshold 100 passes for 99 outcomes and fails on 100; thresholds 0 and 1 never pass. For example, the Flower Monster row's 100-percent slot is not guaranteed by this roll. This differs from the counter-attack choice, whose 0-to-100 roll is also compared strictly below its chance (Behavior 14).
- Slot cooldowns: a slot's cooldown is set when the slot is chosen and set again at the sequencer's end path (Behavior 13). The cooldowns count down only inside the normal choice and only while the hero is attackable; while the hero is not attackable they do not move.
- Retarget with countdown -1 never retargets.
- Timer units: every countdown in this spec is in ticks. A row value of 120 is 2 seconds. Do not multiply row timer values by 60 again.
- Death disappear timer (row 0x80): a count of ticks (Behavior 34 and 36).
- Approach routine other than 1, 3 or 4: the agro machine keeps that value as its outer sub-state, which has no case, so the enemy has no archetype movement and the normal choice never runs (its outer sub-state gate fails). Only 1, 3 and 4 occur on disc.
- Display in default 0: the setter for state 2 gives state 21 unless the enemy's invisibility-off word is set. That word is 0 at creation and only DisableInvisibility (SPEC-0030 row 121) sets it, at 25 disc sites; with it set, the setter and SetAI_Agro enter state 2 normally (High).
- Invisible kinds and hits (Medium-High): an enemy with Display in default 0 whose invisibility-off word is clear is not a hit candidate while it is in its default state, or in state 21 while it is not playing animation 82. The exclusion does not apply while the selected hero's outfit slot holds the Tattered Cloak (item 126): the enemy is then hittable in every state the base hit rule allows. Once DisableInvisibility has run for the enemy, the exclusion never applies.
- Row 0x4FC (strafe erratic) is never read.
- Sequencer states 10 and 11 have no writer and are never reached.
- Attack speed words (High for same-update clip rate): the wind-up, attack and recovery speeds (rows 0x1BC, 0x1CC, 0x1DC + 4k) are stored directly to P by the behavior coroutine after reset/status processing and before the clip re-time (Behavior 1). The written value sets that update's clip rate, replacing earlier status multipliers. The animation pass then evaluates at that rate; its root step is consumed by a later motion-record pass (Behavior 48, item 5).
- Sequencer spawn with the projectile pool full: the spawner creates nothing, but the shot counter still goes up and the pending flag is set as usual, so a full pool costs the enemy that shot (Behavior 13).
- Shot counter and trail: the trail effect entry follows the shot counter, so a shot lost to a full pool still advances the trail entry of the next shot (Behavior 13).
- Sequencer spawn count 0: one projectile is still fired; the counter (1) is not below the count (0), so the loop then goes to state 3 (Behavior 13).
- Flight mode other than 0, 1 or 2 on a missile slot without flag 0x02: state 2 spawns nothing but still counts the shot (Behavior 13).
- Speed ramp outside the trigger band: u is not clamped, so the ramp speed leaves the range [v_end, v] when the target is outside the band (Behavior 13; reproduce it).
- Homing missiles (flight mode 2) live twice the straight lifetime but fly at no more than 1 unit per tick after the first tick (SPEC-0029 Behavior 9).
- Spells (Behavior 43) deal no damage on contact except for sub-type 5's later full-health effect; sub-type 5 also deals immediate missing-health damage when it builds the spell, and only if a projectile slot is free.
- Stun from a blocked charge never counts down (quirk): hit-reaction kind 2 waits for the stun countdown (0xBF0) to reach 0, and nothing counts that countdown down. The enemy stays in the stun until a new hit starts another reaction (allowed when the stun type is not 1) or a code-6 counter-attack starts the sequencer. The Drackonack rows 144 and 145 reach this when their charge is blocked. Reproduce.
- Leash steer-and-stop branch never runs (quirk): the agro tail's band test needs the selected slot index 1, but the normal choice sets the selected slot to -1 earlier in the same tick in outer sub-states 1, 3 and 4 (Behavior 23). Reproduce by leaving it out.
- Counter-attack choice: trigger codes 0 and -1 never pass, and a chance of 0 never passes. The roll is inclusive from 0 to 100, so a chance of 100 passes on 100 of the 101 rolls. One roll is drawn per tick, for the last eligible slot only; a slot that is already the selected slot is not rolled. Reproduce as written.
- Both choices in one tick: the counter-attack choice runs after the normal choice, so when both pick a slot in the same tick, the counter choice's slot and start state replace the normal choice's, and the distances keep the normal choice's redraw (Medium).
- Counter code 1 needs only the hero recover window (table-B bit 0x800 on the active hero's clips); it is eligible in any outer sub-state.
- Missile defence: the block roll is made after the dodge roll, so when both pass the block (outer 8) wins.
- Melee defence: the block test is "at most" and the dodge test "below": a block chance of 0 never blocks (r1 is at least 1), and a dodge chance of 100 always dodges when the block fails.
- Distance-hide gate on approach-4 rows (spyders, rows 233 to 236): never fires, because the gate runs only in outer sub-state 1; the row flag still makes state 8 hide in place.
- Stun on alignment 4: a stun status is not applied to an enemy whose row alignment (0x3C) is 4 (Medium).
- Animation lookups (Behavior 19): -1 means no animation. The caller must not play index -1.
- Action-19 gate (Behavior 21): a nonzero class test makes the gate return 0. What the caller does with a 0 result is open (Open question 54).
- Value tables (Behavior 26): the lookups are described as 1-based. The result for other indices is not described.
- Second death while reanimating: a second death while the reanimation countdown is active does not restart it.
- Refused rise: a rise refused by the animation query heals to full health and returns to agro at once.
- Flying hit values of 3 or more: a flying enemy with a flying hit value of 3 or more gets no hit reaction on a hit.
- Sub-type 5 damage: sub-type 5 applies its damage to the target before its projectile is built, but only when a projectile slot is free. If the projectile pool is full, there is no damage and no projectile.
- Class state after death: non-reanimating enemies keep their class state after death, because the think loop is not stepped in base state 16 (Behavior 1).
- Lying reanimating enemy: it cannot be hit while in class state 16, and it is never released by its death countdown (Behavior 35).
- Hit-count threshold: never reached in play, since the threshold stays -1 (Behavior 37, step 12).
- Released at full health (Behavior 47): an enemy damaged to 1 hit point and then streamed out comes back at full health where it was released.
- Record without an entity entry (Behavior 47; SPEC-0006): a script-slot record with no entry in the pack's entity table has no kill bit, so it is bound again after a kill whenever its slot streams.
- Footstep memory never reset (quirk): the monster's footstep step (shake and rumble on table-B bits 0x80 and 0x100; SPEC-0029 Behavior 32) remembers the last frame it handled, so a frame is handled once. The reset of that memory on an animation change never happens, because the request compares the new id with the stored id after storing it, so the two are always equal. A new clip whose first footstep frame has the same number as the last frame handled in the old clip skips that footstep.
- Missing animation ids: only the slot attack ids and the fidgets are skipped when missing; any other missing id starts clip 0 of the main set (Layout section 6). On the one 138-entry table, ids 138 to 165 (for example the flyer emerge code 165) all start clip 0.
- Counter-attack against Roberta: trigger codes 1 to 3 never pass while Roberta is the active hero, because her clips carry no recover bit (Behavior 14).

Routines of other classes (kept for reference; not class 17):

- Class 7, pen entry (Behavior 16, step 1): the entry does not write the run mode word. The 30-tick value is the lead timer, read by the lead routine from the next tick. A Whinny that leads never flees again, so the stale run mode is never used (High).
- Class 7, chase timers: the timer is reduced before the end test, so a timer of 1 at the start of a tick ends mode 1 or mode 2 in that tick (Medium).
- Class 7, chase and blocked flag: mode 1 and mode 2 end by clearing the actor's blocked flag as well, so a run that ends for any reason releases the flag (Medium).
- Class 21, flee with the touched flag set and a far hero (Behavior 17): with the flag set and d above 20.0, the controller enters mode 1 (with a radius-10.0 pick) and then mode 2 in the same tick, and the record ends in mode 2 with the radius-30.0 point (High).
- Class 21, flee mode 2 exit (Behavior 17): the exit sets the Miamouse main state to 0 and leaves the mode word at 2; the next flee entry resets it to 0 (High).
- Class 21, flee boundaries: mode 0 takes the away step when d is at most 20.0 (inclusive); mode 1 returns to mode 0 when d is below 20.0 (strict); a point distance of exactly 0.5 is not an arrival (High).
- Class 34, selector: cooldown B is compared with a threshold that uses the tick-rate word and a factor of 0.5; the unit of cooldown B is not settled (Open questions). Reproduce as written.
- Class 7, band controller: the 3.0 hold is reached only if the distance of step 3 of Behavior 29 is measured differently from the 5.0 test. Reproduce as written.
- Class 7, wait-and-approach: a target point within 1.5 units ends the approach on the same tick, before any steering in that tick.
- Class 29, patrol list: no monster row has this class, so the walk never runs on the disc. Dormant rule (Low): no movement when no index is found, but the pause after each point still runs.
- Class 34, state 4 with no object: the state returns to 0 on the same tick.
- Class 35, charge and wait: the wait length depends on the tick-rate word, so a different tick rate changes its length.
- Class 6, leader cooldown at the limit: a second count of 1 becomes 0 after the decrement, so the search runs. A count of 2 becomes 1, so no search runs.
- Class 6, no match: the cooldowns still decrement, and no leader field or state changes.

## Test vectors

Synthetic and disc-based, hand-checkable:

- Proximity retarget: P = (0,0,0), F = (1,0,0), Q = (6,0,0), r = 1 gives gate length 0 < 1 and A = (−3,0,0); Q = (7,0,0) gives gate length 1, so strict equality does not update the target.
- Notice band: Dumb Skeleton, Awareness 65, band 3 to 5 (chance 50 at 3, 1 at 5), hero action state with F1 = 1.0, sight line clear. A hero at 2 units: noticed when roll A is at most 65 (closer than the near edge). At 3 units: chance 50. At 4 units: chance 25.5 (interpolated; before the cone or hidden halvings). The same hero at 4 units in hero action state 1 (F1 = 0.7, band 2.1 to 3.5): not noticed (beyond the far edge).
- Cone: with View angle 0.707 and a hero at 45 degrees off facing (cosine 0.707, not below 0.707), no halving; at 60 degrees (cosine 0.5), the chance is halved.
- Approach band: approach 3, desired min 15, desired max 25, random integer 50 (f = 0.5): desired = 20 units. Retreat band 20 to 30: a draw of 25.
- Trigger: Dumb Skeleton has two melee slots with trigger band 0 to 10 units. Slot 2 is tested first (arc 0.71, chance 40, delay 60 to 120 ticks), then slot 1 (arc 0.92, chance 60, delay 30 to 240 ticks). A hero at 0 units: no slot (strict minimum 0). At 5 units, facing cosine 0.95, roll 30: slot 2 is chosen (30 is below 40 and 0.95 is above 0.71); its cooldown is drawn in [60, 120] ticks, and the sequencer starts in state 0 (no charge).
- Priority: an enemy with slots 1 and 4 both passing selects slot 4.
- Timer: a search timer draw of 90 ticks is 1.5 seconds; a recovery draw of 150 ticks is 2.5 seconds.
- Idle holds: an idle variant drawn for 300 ticks changes after 5 seconds.
- Strafe hold (synthetic): outer sub-state 4, inner 10 with the hold at 1. The hold becomes 0, r = 3 and the strafe lock is 0: inner 9 with a new hold from 60 to 180 ticks. With the lock at 50, r = 3 gives inner 10 again. Circling into an obstacle (blocked flag) gives inner 10 and a lock of 120 ticks.
- Sequencer approach and aim: melee slot range 10, distance 9.5, hero and steer target on the walk mesh: state 1 at once, whatever the facing. With the hero off the walk mesh and a facing cosine of 0.96 (at least 0.95): state 1. With a facing cosine of 0.9: stays in state 0 and turns with blend 0.1. At distance 12 with both on the mesh: the enemy pursues and stays in state 0.
- Sequencer spawn count: count 3. Spawn event active on ticks 1, 2, inactive on tick 3, active on ticks 4 and 5, inactive on tick 6, active on tick 7. Tick 1 fires shot 1 (counter 1, pending set). Tick 2: still pending, nothing. Tick 3: the pending flag clears. Tick 4 fires shot 2 (counter 2). Tick 6 clears the flag; tick 7 fires shot 3, the counter reaches 3 and the loop goes to state 3.
- Sequencer spawn count 0 (synthetic): one projectile is fired on the first active spawn event, then the loop goes to state 3.
- Minion Archer (row 190, slot 3; flight mode 0): launch speed 2.0 and range 70 give a lifetime of 35 ticks. Launch (0, 2, 0), hero at (0, 0, 20), no scatter: aim (0, 2.5, 20), velocity (0, 0.04998, 1.99938) per tick, damage 10 x 1.0 = 10, status 7 (fire) at chance 100 with value 10 to 30. Spawn count 2: two arrows per attack, one per spawn event (full flight in SPEC-0029 Test vectors).
- Flower Monster seed (row 149, slot 0; flight mode 1): v 0.5, v_end 0.1, band 3 to 60. Launch (0, 1, 0), hero at (20, 0, 0): d = 20, dy = -1, u = 0.70175, s = 0.30302, T = 66.003 ticks, vy = 0.27820, lifetime trunc(60 T) + 1 = 3961 ticks (full flight in SPEC-0029 Test vectors).
- Homing (row 213, slot 1; flight mode 2): launch speed 1.0 and range 100 give a lifetime of 2 x 100 = 200 ticks. Each tick the velocity becomes 0.75 of the old unit direction plus 0.25 of the unit direction to the hero + (0, 2, 0) (SPEC-0029 Test vectors).
- Spell lifetime (Behavior 43): a launch-to-aim distance of 12.7 gives 12 + 5 = 17 ticks at 1.0 unit per tick.
- Aim scatter (synthetic): row inaccuracy 0, enemy inside the camera cone, not waiting in line: e = 0 and the aim point is the target point. The same enemy outside the camera cone (dot 0.8): e = 5.0.
- Sequencer loop: state 13 sets a countdown of 60 ticks (one second) when the hero is within the melee range; state 14 counts it down and damages the hero on its 61st tick, when the countdown is 0, then reloads it from the loop interval range.
- Counter-attack, one slot only (synthetic): slot 2 is the only eligible slot. Its trigger code is 6 and its chance is 40. The enemy is in agro outer sub-state 10 (hit reaction), the hero is at distance 6 inside the bounds 0 to 10, and the facing cosine is 0.95 (above the arc 0.71). A roll of 39 chooses slot 2 and a roll of 40 does not.
- Counter-attack, chance 100 (synthetic): a roll of 99 chooses the slot; a roll of 100 does not.
- Counter-attack, recover window (disc): against Tor's swing id 14 (its frames OR to 0xDC1), a slot with code 1 is eligible on the frames that carry 0x800; against Roberta's id 14 (frames OR to 0x1A1, no 0x800) it is never eligible.
- Slot ids (synthetic): slot k = 2 gives wind-up 43, strike 44, follow-through 45, blocked 46, recovery 47. An enemy whose table has 44 but not 43 skips the wind-up and starts the strike; a walk request for id 2 on a table without it starts clip 0.
- Counter-attack, trigger tests (synthetic): with the hero recover window open, a slot with code 2 is eligible in outer sub-state 6 and not in 7; a slot with code 3 is eligible in outer sub-state 8 and not in 5. Without the window neither is eligible. A slot with code -1 is never eligible. With slots 1 and 3 both eligible, slot 3 is the one rolled for.
- Spacing (synthetic): a candidate 5.0 units from the hero and 6.0 from the nearest other pool object passes. At 4.9 units from one object it fails.
- Animation lookup: group B, index 3 gives 0x1D; group D, index 2 gives 0x14; group E, index 1 gives -1.
- Attack code (synthetic): selected attack slot 2 gives 0x2E in group 2 if the availability test passes, and -1 if it fails.
- Band factors: hero action state 4 gives F1 1.75 and F2 0.9. An action state not in the table (for example 7) gives F1 1.0 and F2 0.5.
- Leash and distance-hide gate: see Behavior 23 and Behavior 24.

Routines of other classes (classes 7, 21, 34, 35, 36, 29 and 6; synthetic; invented distances):

- Chase pool (synthetic): the pool search returns index 5 and the owner id at index 5 is -1. The controller takes point 5, sets the pick timer to 30, the counter to 0, the owner id to -1, and sets the claimed flag for index 5.
- Chase mode 1 timer: with the frame-rate word 60, the timer starts at 15 ticks. With the word 120, it starts at 30 ticks.
- Chase mode 1 arrival: a horizontal distance of 2.5 to the point keeps the run going (the test is strict, below 2.5). A distance of 2.4 ends it at once.
- Chase mode 1 timer end: a timer of 1 at the start of a tick is reduced to 0 and the run ends that tick.
- Chase mode 2: the timer starts at 25 ticks. A distance of 2.0 keeps the run going while the timer is at least 1; a distance of 1.9 ends it.
- Chase counter: a horizontal distance of 8.1 to the target sets the counter to 0. A distance of 8.0 does not.
- Chase pick: with the candidate walkable and the blocked flag clear, mode 1 is chosen. With the candidate not walkable, mode 2 is chosen with a 25-tick timer. With the candidate walkable but the blocked flag set, mode 2 is chosen.
- Approach mode 0 (synthetic): steering flag clear, d = 20.0, candidate walkable: the actor moves toward the candidate and the mode stays 0. With d = 20.1: mode 2 with the point 30.0 units out.
- Approach, flag set and d = 25 (synthetic): the record ends in mode 2 with the 30.0 point.
- Approach mode 1 (synthetic): direction test 0.6 and flag clear: keeps running. Direction test 0.6 with the flag set: not kept. With d = 15 then: mode 0.
- Approach mode 2 (synthetic): direction test 0.4: the done word becomes 0 and the mode word stays 2.
- Band controller, close band: d = 5.0 is not below 5.0, so it does not close. d = 4.9 sets the state to 2 and the flag to 1.
- Band controller, approach: with mode 0 and the steering flag clear, d = 10.0 selects movement class 0 (not above 10.0). d = 10.1 selects movement class 2.
- Re-pick timer: with mode 1 and the timer at 0, the mode is cleared and a new timer is drawn in [120, 400] (both ends included). With the timer at 1 the actor waits one tick (animation code 0) and the timer becomes 0.
- Wait-and-approach: done 0 and d = 1.5 gives done 1 on that tick. d = 1.6 keeps done 0 and the actor walks.
- Charge and wait: with the counter at 180 (at 60 ticks per second) the phase B wait ends. Distance 12.0 with a direction test of 0 keeps phase B. Distance 12.1 returns to phase A.
- Patrol list: with three points, the actor pauses 60 ticks after the first and second arrivals and does not pause after the third.
- Idle coroutine: with period P = 400 and counter 400, the actor plays animation code 1 on that tick.
- State 5: a direction test value of 0.49 runs the release helper. A value of 0.5 steers at speed 0.1.
1. Leader handover (synthetic): enemy id 5 and other id 9 in the same group and within range; the candidate has +0xC98 = 0, +0x3B0 = 3 and +0xA34 = 0, with the second cooldown at 1. After the call both AI states are 3, the enemy's leader field is 9, and the other's leader field is 5.
2. Leader candidate filter (synthetic): with +0xC98 = 1, candidate is rejected; with +0xC98 = 0 and +0x3B0 = 0x70, candidate is rejected; with +0xC98 = 0, +0x3B0 = 3 and +0xA34 = 3, candidate is rejected. Values +0xC98 = 0, +0x3B0 = 3 and +0xA34 = 0 pass this filter and proceed to the group and range tests.
3. Leader cooldown (synthetic): second cooldown at 2 becomes 1, and no search runs. The cooldown at 1 becomes 0, and the search runs.
4. Pairing reset (synthetic, at 60 Hz): a draw of 600 sets +0xA9C to 600 ticks (10 seconds); a draw of 1800 sets it to 1800 ticks (30 seconds). In either case, the pairing sub-state becomes 0 and the AI state becomes 0.

- Reanimation: a Dumb Skeleton (row 226) killed at tick 0 draws a countdown N in [8000, 12000]. It lies down with animation 58 for N + 1 ticks, then rises when clear. With N = 2: animation 58 on ticks 1, 2 and 3, and the rise starts on tick 4 if the area is clear.
- Death timer: row 183 has a death disappear timer of 1. If its death clip reaches progress 0.95 on tick T, the countdown is 0 at T and -1 at T + 1, so the enemy counts as gone from T + 1.
- Take-hit, agro: an enemy in agro (state 19) takes a hit of 6 with the hit-lock timer at zero. Damage 6 applies, and the agro outer sub-state becomes 10 with hit-reaction kind 0.
- Take-hit, non-agro: a hit of 6 on an enemy in state 17 with hit-lock 0 doubles to 12, sets state 19 and applies 12. With the same inputs but hit-lock nonzero, state 19 is still set and the internal value is doubled, but HP does not change; the next hit in state 19 is not doubled.
- Flying hit choice (synthetic): a flying enemy with a flying hit value of 1, a retarget countdown of 0 and no animation playing takes a hit with the caller's extra flag at zero. It clears the stored flag and starts hit-reaction kind 7 (outer sub-state 10). With a flying hit value of 0 it starts kind 6 instead.
- Sub-type 5 (synthetic): an attacker with maximum HP 50 and current HP 20 that reaches its event, with a valid target and a free projectile slot, deals 30 immediate damage and submits one spell. With a full pool it deals no damage and submits no spell. With no valid target it also produces no damage or spell. If the created spell later hits, the target is set to maximum HP.
- Map search type to sub-type (synthetic): search type 3 selects sub-type 1; search type 7 selects sub-type 5.
- Map-search type 2 target (synthetic): a visible class-17 candidate in state 16, playing its reanimation death clip rather than animation 4, with countdown 9000 and HP 0 of 50 passes the picker when within the caster's map-search distance. State 16 and the positive countdown do not disqualify it; setting its HP to 50 or selecting animation 4 does.
- Kill memory (synthetic, Behavior 47): kill the enemy of record 3 of a world, leave the world and enter it again. Record 3 is not bound again. Repeat with a Dumb Skeleton (row 226) on record 3: it is bound again.
- Release and return (synthetic, Behavior 47): an enemy with 50 maximum hit points at 10 hit points is released when the hero is 281 units away; when the hero comes back to 279 units, with a free object and the block below its cap, it is bound at the release point with 50 hit points.
- Game load (synthetic, Behavior 47): enter world A, kill record 3, then load a save made before entering A. Record 3 of A is alive again.
- Arrival (synthetic): a type 1 target at distance 0.8 whose type word is 0xBE and whose subtype is not 0x11 is consumed with a consume flag of 4. The target index is cleared and the idle pick plays.

Poison Spyder (row 235), a worked example with disc values (High for the row and placement values):

Row and placements: 70 placements in resources 697, 698, 796, 865, 903, 904, 948, 1080, 1244, 1245, 1250, 1257 and 1265; default-state word 8 on 47 of them, 7 on 14, 5 on 6, 0 on 2 and 9 on 1. Hit points 16, approach routine 4, difficulty value 8, display in default 1, distance hide flag 1 (distance 50.0, chance 90). Awareness 100 every 30 ticks, view angle 0.707, sense range 5 to 30, chance 75 at the near edge and 25 at the far edge. Missile defend time 15, missile dodge 100, missile block 0; melee dodge 25, 50 and 75 (quick, normal, heavy), melee block 0; defence timer 1 to 30. Strafe desired distance 4 to 15. Attack timer (recovery) 15 to 30. Slot 1: melee, band 0 to 8, chance 25, arc 0.9239, melee range 3, delay 30 to 60, no charge (charge range 0 to 0), poison chance 25 for 600 to 1800 ticks. Slot 3: melee with charge type 2, band 10 to 35, chance 50, arc 0.707, delay 0 to 0, charge distance 30 to 40, counter attack trigger 1 with chance 75, poison chance 50.

Vectors (synthetic positions; hero in action state 1, so F1 = 0.7):
1. Hidden in state 8 with the spawn record's emerge distance 10.0: a hero at 7.0 units leaves it hidden; at 6.99 units sub-state 1 starts; on the next tick animation 10 plays; when it has finished the enemy is in agro.
2. Wander state: band 3.5 to 21. A hero at 3.0 units (closer than 3.5) with roll A 40: noticed. At 10 units in the cone: chance 65; roll B 64 notices, 65 does not. At 21.5 units: never.
3. First agro ticks: on the entry tick the outer sub-state is 0; on tick 2 it is 4 with inner 10 and hold 0; on tick 3 the hold becomes -1, r = 2 gives inner 8 with a new hold (for example 100). At d = 9 no slot passes (9 is not below 8 for slot 1, and not above 10 for slot 3).
4. Choice: d = 12, facing cosine 0.8, roll 30: slot 3 passes (30 below 50); its cooldown is 0; the desired distance = 4 + 11 r / 100 (r = 50 gives 9.5); the charge distances come from float[30, 40) (for example 35.0); the sequencer starts in state 6.
5. Charge: A to its end, then B and state 7. With the hero reached at 2.9 units (within the melee range 3): state 8, C with strike tests, C finished: state 12, E, then outer 0 with the recovery drawn from 15 to 30. With the hero staying 5.0 units away: after 35.0 units travelled, state 12.
6. Counter: the hero recover window open with slot 3 eligible and not selected: roll 74 chooses it, 75 does not. If the last normal choice was slot 1, slot 3's charge distance is 0.0 (slot 1's range is 0 to 0), so the counter charge ends at once unless the hero is within 3.0 units.
7. Melee dodge: on a normal swing, r2 = 49 gives outer 5 with the defence hold 15; r2 = 50 gives no reaction. On a heavy swing r2 = 74 dodges.
8. Missile dodge: a hero shot whose 15-tick path meets the body: roll 99 gives outer 6; roll 100 gives no reaction.
9. Leash: no route, the walk waypoint 15.0 units from the hero for 181 ticks: the setter runs with 0 and gives state 0 (display in default 1); on the next tick the pre-step runs the setter with the default-state word 8; state 8 hides in place because the distance hide flag is 1.
10. Bite poison: roll 24 poisons, 25 does not; a duration draw of 1234 ticks gives 20 seconds (1234 / 60, truncated).

Attack manager (Behavior 45). Three enemies A, B and C on one hero, none busy, listed in the order A, B, C:
- Capacity: Tor with Sword 98 and armour 119 gives C = 3.0 x (0 + 0) = 0. Tor with sword 105 and armour 125 gives 39. Roberta with wand 118 and dress 134 gives 42. Roberta with nothing equipped gives -6. Item 126 in the body slot counts 0.
- Weights 8 (for example Minion Grunts) at 4, 9 and 20 units. C = 0: A attacks; B and C wait (used 8 and 16 are above 0). C = 15: A and B attack, C waits (used 16). C = 39: all three attack. The order stays A, B, C, because 9 is not below 4 + 5.0.
- Weights 12 (for example Minion Warriors) with C = 15: A and B attack (used 0 and 12), C waits (used 24). Five such enemies at 4, 12, 20, 28 and 36 units with C = 39: the first four attack and the fifth waits (used 48).
- Weights 0 (Crokabek) with C = 0: all attack. With C = -6: only the first attacks.
- Close group, C = 0, weights 8, at 4, 6 and 7 units: frame 1 gives the order C, B, A, so C (the farthest) attacks. Frame 2 gives A, B, C, so A attacks. The two orders then alternate each frame while the distances stay the same.
- Busy holder, C = 0, B busy at 6 units, with A at 4 and C at 7: frame 1 gives B, C, A and frame 2 gives B, A, C. B keeps the first place and the only attack.
- Refused enemy, approach routine 1: outer event 2 gives outer 12. With row 0xB0 = 10.0 the stand-off point lies within 5 units of the hero on each horizontal axis (integer draws 0 to 9, minus 5.0). Once there, it plays 73, then waits int[600, 800] ticks while it walks to a new point within 25 units of the hero on each axis. When its flag clears it returns to outer 0. With approach routine 3 the same event starts the sequencer instead.
- Corpse sink (Behavior 36), not flying, death disappear timer 600 at 60 ticks per second: the countdown starts to fall on the tick the death clip reaches 0.95. The first 30 decrements (599 down to 570) do not sink. Each tick from 569 down to -1 sinks 1/180 unit, 571 ticks in all, so the corpse is 571/180 = 3.172 units lower when the countdown reaches -1. Timer 60: 31 sinking ticks (29 down to -1), 0.172 units. Timer 1: the countdown goes 0, then -1, which is never below -29, so the corpse does not sink before it is released.
- Flyer at death (Behavior 34, step 2): a can-fly 1 enemy with ground timer min 60 and max 120 that dies while airborne gets a retarget countdown of a whole number from 60 to 119. As a corpse it plays 58 and sinks. A can-fly 2 enemy plays 153 and does not sink.
- Strafe circle (Behavior 44): enemy at (0, 0, 0), hero at (0, 0, 10): dir = (0, 0, 1), p = (-1, 0, 0). Inner 8 steers toward (2, 0, 0) and inner 9 toward (-2, 0, 0).
- Rise test (Behavior 35), countdown below 0: hero 6.0 units away and another pool member 4.9 units away: no rise, the enemy plays 58. With that member 5.0 units away: the enemy plays 85.
- Attacker lookup (Behavior 37, step 11), extra-attacker list of 3 entries: id 1 gives the selected hero, id 3 the first entry, id 4 the second entry, id 7 the selected hero.
- Hit-count threshold (Behavior 37, step 12): threshold -1; after 100 hits the hit counter is 100, the invulnerable word stays 0 and the enemy can still be hit.
- Reanimating enemy, lying (Behaviors 35 and 47): countdown 500 gives a kill bit on that tick and no hit candidate; countdown 0 gives no kill bit from the dead test, and the enemy is still not a hit candidate because its class state is 16.
- Grab release (Behavior 37, step 4): an enemy with grab phase 2 holds the hero (hero base state 53). A hit on that enemy that passes steps 1 and 2 sets the hero's base state to 1 in the same hook call; the phase stays 2. With phase 3, or with the hero in state 1, the hero's state is unchanged.
- Death clip (Behavior 36, step 2): a grounded enemy dies on tick T and its death clip 58 starts at its first key. On ticks T + 1, T + 2, ... the dead state requests 58 again and the clip keeps advancing; it is never restarted, so a clip whose progress first reaches 0.95 on tick T + k starts the disappear countdown on that tick, whatever the number of repeated requests.
- Force start (Behavior 37, step 13): an enemy with stun timer 0 playing its walk clip at key 7 is hit. On the next hookup the requested clips start again at key 0. With stun timer 30, the walk clip continues from key 7.

## Open questions

1. Resolved: the second per-enemy data block (object offset 0xA44) is the enemy's kind-3 spawn record (SPEC-0006). Class 17 reads its default-state word (u16 0x84), alarm group byte (0xB7), emerge distance (f32 0xC0), cue word (0xC4) and emerge selector (0xC8) (Layout section 1); state 9 reads the named target id at 0xBC.
2. Resolved: the row schema labels +0x6C/+0x70 “Flee Timer Max/Min,” while class state 1 uses the values as the upper/lower endpoints of an integer countdown for a timed approach toward the hero. The label is kept as serialized data; no authoring rationale for “Flee” is established (Behavior 5, state 1; High for runtime use).
3. Resolved: the agro outer sub-states 0 to 12 and their transitions are in Behavior 44. The attack sub-type actions 0 to 6 of the map-search attack are spells that submit one projectile of kind 2 to 7 to the enemy projectile pool, with the contact effects of Behavior 43.
4. Answered: non-spider common-enemy death and corpse behavior are specified in Behaviors 34 to 36, including the reanimating-row exception; the nest spiders use their separate state-16 path in Behavior 12.
5. Resolved: the attack-preparation setter is the agro outer sub-state setter. It starts the sequencer (outer sub-state 2) in the start state given by the choice, or the wait in line (outer sub-state 12) when the attack manager says so (Behaviors 7 and 45). The post-attack step is the sequencer's end path (Behavior 13).
6. Resolved for timing (High): in sequencer states 1 and 6, state 2, and state 5, the coroutine directly stores the selected row speed to P before the same-update clip re-time (Behavior 13; Behavior 1). A written P of 1.5 therefore sets that update's clip rate to 1.5. This direct store replaces P accumulated from status factors earlier in the update. The animation pass evaluates the clip at the written rate after the motion-record pass; its updated root step is used by the following motion-record pass (Behavior 48). The strafe erratic field (0x4FC) is never read.
7. Resolved: the charge draw uses the selected slot's range for every slot; at creation the selected slot is -1 and the draw reads the row words 0x258 and 0x268 (Edge cases). Reproduce.
8. Resolved: bit 0x40 replaces the trigger chance with int(anger counter x 0.05), and the choice clears the anger counter (Behavior 7, step 5d). Bit 0x02 marks a summoning slot, read both as the live-summon gate of the attack choice and, on a missile slot, as the lobbed-summon selector (Behaviors 7 and 13).
9. Resolved: the ground turn speed (0x84) and the flying turn speed (0x88) are the aim turn speeds of the agro hold sub-states (Behavior 44). Neither is read for movement; turning while travelling is the steering blend only (Behavior 48). The ground timers (0x98, 0x9C) give the grounded time of a can-fly-1 enemy, held in the retarget countdown (Behavior 5; Behavior 34, step 2).
10. Resolved: projectile launch speeds are in units per tick and ranges in units; a straight shot lives trunc(range / speed) ticks (Behavior 13). The missile count (0x33C) is the spawn count of sequencer state 2.
11. Resolved for operational behavior (Behavior 3): the alarm-group suppression scans other enemies in states 19 or 2; the notice routine itself performs a sight-line test, not a navmesh route test. When sight is blocked and Always Has Line of Sight is 0, a Tattered Cloak wearer is not noticed; wearing the cloak also halves the chance within the band. States 3 and 9, then 5 to 7, consume a passing result by entering state 2. The design meaning of the range factors remains unknown.
12. Resolved: the class-17 socialize compatibility test (Behavior 10) conditionally reads resource 531. When the candidate peer script-hold word is set, it checks companion-row +0x16 as an eligibility value and uses the pair +0x18 values as distance contributions; otherwise it reads the corresponding values from resource 530 (+0x40 and +0x44). The conditional read is in the class-17 peer scan; its successful use depends on matching targets and row values.
13. Resolved: the two factors are F1 and F2 of Behavior 18, looked up by the hero's action state. F1 scales the notice band and the emerge distance; F2 replaces it when the sight line is blocked.
14. Resolved for the spider state machine and its update/draw consumers: after its countdown, state 16 checks for animation 85 in the spider’s own animation table. If present, a spacing test gates that animation; failure restarts death animation 58 or 153, while completion of 85 restores full HP and enters agro state 19. If 85 is absent, the full-HP/agro end step runs immediately (Behavior 12). Animation 85 is a clip code, not an event id. The nest sets the despawn flag for a registered state-16 spider after death animation 58 completes and clears it on release (SPEC-0017, Behavior 9). While the flag is set, the region update skips the character update and the shared drawable test rejects it, but the object remains registered in its active slot. The evidence does not identify an object-destruction or slot-release path for this flag; removal timing remains unknown (SPEC-0006; SPEC-0046).
15. Resolved (High for the entry values and the hold animations; Medium-High for the transitions): sub-states 1, 3 and 4 with their hold animations are the rush, keep-distance and strafe outer sub-states of the agro machine (Behavior 44), not a spawn rule. Their entry sets the inner sub-state and the gait set (rush: inner 1 and set 21, 21, 21, 23, 55; keep distance: inner 4 and set 24, 24, 24, 26, 55; strafe: inner 10, hold 0 and set 28, 28, 28, 32, 55, 30, 29), with flyer sets 101 and 102, 107 and 108, and 103 to 106 written only when Can Fly is nonzero. The hold animations are played every tick of the hold inner sub-state while the enemy faces the hero: rush hold (inner 1) 22, keep-distance hold (inner 4) 25, strafe hold (inner 10) 31; a flyer plays 155, 156 and 132 instead (for the keep-distance hold, a flyer clip is used whenever either of the two fly tests holds). A revived spider enters agro through the setter (Open question 46, resolved). Test vector: a ground enemy whose approach routine is 4 enters outer 4 with inner 10 and plays 31 while holding; the same enemy with Can Fly 2 plays 132; approach routine 3 in inner 4 plays 25 (flyer 156); approach routine 1 in inner 1 plays 22 (flyer 155).
16. Answered: reanimating rows 224, 226 and 227 use class state 16 for the countdown and rise path in Behavior 35; spiders use a separate class-state-16 death path whose animation-85 return to full health and agro is described in Behavior 12. The nest despawn flag's downstream object-removal path remains unknown (Open questions 14 and 47).
17. Resolved: the spawn-pending flag is cleared on the first tick the slot's spawn event is not active. A full projectile pool creates nothing, but the shot counter still goes up and the flag is set as usual (Behavior 13, state 2). Also SPEC-0029 open question 41.
18. Resolved: every attack slot, melee or missile, runs through the attack sequencer (Behavior 13), started by the normal or the counter-attack choice. Class state 18 is entered only by the map search per tick and runs only the map-search spells (Behaviors 42 and 43).
19. Resolved: both readings hold. Bit 0x02 is the live-summon gate of the attack choice (Behavior 7, step 5e) and, on a missile slot, selects the lobbed summon, which spawns the slot's key where it lands (Behavior 13). Words 0x17C and 0x18C are the trigger band and also the speed-ramp band of flight mode 1 (bmin = 0x18C, bmax = 0x17C). Also SPEC-0029 open question 40.
20. Resolved: hold-field readings differ. The idle routine that drives the wander and hold states of common enemies reads the fields at 0xC4 and 0xC8 of the spawn record (Open question 1) as a hold range, 0xD0 and 0xD4 as a move range, and 0xCC as a chance roll. The row map of Layout section 2 reads the same offsets of the monster row as Awareness, Awareness timer and the sense values. Both are kept: they are offsets of two different records. Resolved: the spawn record's words 0xBC to 0xD4 are one seven-word parameter block of the default state. A level script's set-behaviour command copies all seven words together and sets the default-state word (0x84); when the default state changes, the class state is set to 0. Their meaning depends on the default state. The class-17 wander state (6) reads 0xC4 to 0xC8 as a hold range in ticks (a whole number, both ends included), 0xCC as a percent chance (a roll int[1, 100] below it starts a move) and 0xD0 to 0xD4 as a move range in ticks. State 8 reads 0xC0, 0xC4 and 0xC8 as the emerge distance, cue word and selector, and state 9 reads 0xBC as the named target. The monster row's 0xC4 and 0xC8 stay Awareness and Awareness timer. Test vector (synthetic): a spawn record with words 0xC4 = 60, 0xC8 = 120, 0xCC = 40, 0xD0 = 30 and 0xD4 = 90 in the wander state: a hold lasts a whole number of ticks from 60 to 120, both included; a percent roll of 39 starts a move (it is below 40) and a roll of 40 does not; a started move lasts a whole number of ticks from 30 to 90. The monster row's own fields at the same offsets are not read by this routine. [Resolved]
21. Resolved: all sixteen sequencer states and their next states are in Behavior 13. States 10 and 11 have no writer; state 9 kinds 0 and 1 end in state 12; state 12 plays E and runs the end path.
22. Resolved: in state 7 the 100-tick limit and the speed test (at most 0.001) are one combined exit, for every charge kind; kind 3 has its own exit on distance, hero range or a speed of at most 0.1 (Behavior 13). Kind 1 of state 9 goes to state 12 when D has finished.
23. Resolved: both readings are live and do different jobs. Behavior 7 is the normal attack choice and Behavior 14 the counter-attack choice; both run every agro tick, the normal choice first, and both start the attack sequencer. Neither enters class state 18. Also SPEC-0029 open question 54.
24. Resolved: the trigger codes test agro outer sub-states and sequencer states, not class states. Code 4 is the sequencer recovery (outer sub-state 2 with sequencer state 5). Code 5 requires sequencer state 4 and tests mask 0x0004 in pad slot 0's current held-button word: L1 is held. It does not read a Table-B animation event or the hero's separate control-return flag at +0x3D0. The held-button word is refreshed from the controller report on each input pass and clears when the reported L1 state clears, including an unavailable/disconnected pad (Behavior 14; SPEC-0033, Controller map). [High]
25. Resolved: the counter-attack walk runs from slot 1 to slot 4, and the last eligible slot wins (Behavior 14).
26. Resolved: the numbers 2, 5, 6, 7, 8 and 10 of the trigger codes are agro outer sub-states (2 attack, 5 and 6 dodges, 7 and 8 blocks, 10 hit reaction), not class states.
27. Resolved: row 0xE0 is the defence hold time of the defence reactions and the look-ahead of the missile defence (Behavior 44); it is not a cooldown of any strike.
28. Resolved: the state-5 side helper uses 3.0 as a multiplier of its spatial threshold r when placing the new target point. The class-35 charge-and-wait coroutine uses tick-rate × 3.0 as a wait duration and belongs to a separate class routine (Behaviors 25 and 31). The equal constant does not indicate shared timing behavior.
29. Resolved: one roll is drawn per tick, for the last eligible slot of the walk, and only when that slot is not already the selected slot (Behavior 14).
30. Resolved: codes 0 and -1, and any value outside 1 to 6, never trigger a counter-attack; class 17 makes no difference between them.
31. Resolved (High): entering agro outer sub-state 2 writes the grab machine directly to state 1; selecting the next attack slot resets it to state 0. The four grab-point event selectors read the enemy's current animation and choose its attach transform (SPEC-0029 Behavior 23). The hero's held handler requests animation 143 and stays in base state 53 while any actor in the actor list answers yes to the shared held-hero question. Only the common-enemy implementation can answer yes, and it does so exactly while the enemy is in agro outer sub-state 2 and sequencer state 2 with a grab slot (flag 0x08 set), without checking whether that enemy initiated the grab. The heroes and the other actor kinds share the base implementation, which always answers no (Behavior 15; High). Resolved: the earlier reading of a non-grab slot was inverted.
32. Resolved: the row fields 0xEC to 0x100 are the melee block and dodge chances of the melee defence (Behavior 44); the grab-and-throw does not read them.
33. Resolved: 0x22C is the Counter Attack Trigger code and 0x23C the Counter Attack Chance (Behavior 14). Also SPEC-0029 open question 54.
34. Resolved: the chase-to-point controller (Behavior 16) belongs to class 7 (Whinny) and the approach controller (Behavior 17) to class 21 (Miamouse), by the class routine table (SPEC-0029 Layout section 3). Their relation to the SPEC-0029 Behavior 12 controllers is not established (SPEC-0029 open question 47).
35. Resolved: the approach controller (Behavior 17) is the Miamouse chase-and-retreat state (class 21), not a class-17 movement, so it has no link to the class-17 bands of Behavior 4, the timed approach or the grab-and-throw.
36. Resolved: the frame-rate word at 0x368FA0 is outside the boot image and is written once at start-up from the boot rate table at data address 0x30C598, with the value 60 (SPEC-0007, Rate selection). Nothing changes it afterwards, so the mode 1 timer is 15 ticks (High).
37. Resolved for shipped setup and claim behavior (High): the point list contains one pen in the Farm setup and four in the four-pen setup; the four-entry arrays do not support a larger list (SPEC-0030 rows 65 and 89). A one-pen list ignores its occupied flag when selecting; a multi-pen list accepts only unoccupied entries, sets the selected flag, and does not clear that claim on arrival or in the lead/penned path. Setup reload clears the flags. Runtime point coordinates remain outside the boot image, and the wider meaning/linkage of the shared owner-id fields remains open (Behavior 16; SPEC-0029 OQ69).
38. Resolved: the controller is the Whinny flee mode, chosen by the main loop when the flee selector (+0x0C) is not 0; the pen entry runs first on every tick of the controller. The caller does not clear the run mode word. The lead timer at +0x08 and the lead flag at +0x34 are read by the lead routine, which runs from the next tick (Behavior 16; High).
39. Resolved: the candidate is the Whinny's position plus 4.0 times the horizontal unit direction from the hero to the Whinny, a point 4.0 units directly away from the hero. The reading with a point past the target is wrong (Behavior 16; High).
40. Resolved: the shared helper returns the horizontal unit direction from its first argument to its second (height part 0) and, in its fourth lane, the horizontal distance between them. Both controllers call it from the hero to the creature. The point distances (2.5, 2.0 and 0.5) are also horizontal: the routines zero the height difference before taking the length (Behaviors 16 and 17; High).
41. Resolved: the value compared with 0.5 is the horizontal distance from the Miamouse to its point (arrival when below 0.5), not a direction (Behavior 17; High).
42. Resolved: the done word is the Miamouse main state word; the mode 2 exit sets it to 0 (idle). The mode word is left at 2 and is reset to 0 by the next flee entry. Mode 2 has no other exit (Behavior 17; High).
43. Resolved: the point helper is the far centroid pick of SPEC-0029 Behavior 28 (random triangle centroids until one lies more than the radius from the hero horizontally). The Whinny passes its own world record (+0xBCC) and the radius 10.0; the Miamouse passes the current world (0x35C834) and the radius 10.0 or 30.0 (High).
44. Answered: the launch values are motion-record velocity in units per second; the shared mover applies velocity x 1/30 per tick, so 10.0 contributes 1/3 unit per tick. The vertical component is world y (SPEC-0017, Spawn; SPEC-0040 Behavior 14A).
45. Resolved (Behaviors 8 and 12): spiders use the generic class-17 idle picker. Its base is animation 0 when the movement predicate is false and 97 when true; when the hold expires it chooses 115 to 119, substituting 142 when the movement predicate is true, and only returns the selection if the spider table contains it. The hold range [120, 600] is a closed integer draw, so both endpoints are reachable (SPEC-0008). The visual/content labels and intended roles of 97 and 142 remain unknown.
46. Resolved (High): "state 1, action 19" is base actor state 1 (alive and active) plus class state 19 set through the class-state setter. The revived spider runs the agro machine (Behavior 44) with the full state-19 entry work (Behavior 5 table): alarm-group broadcast, combat timer drawn from the row, cooldowns cleared, added to the attack manager, previous class state saved. Class state 1 (timed approach) is not involved (Behavior 12).
47. Resolved in part: state 16’s post-countdown return path is documented in Behavior 12. Animation 58’s length comes from the spider’s animation data. The nest-set despawn flag suppresses the spider’s update and drawing while set; the active slot remains registered in the examined path, and the nest clears the flag on release. No object-removal path is established (Behavior 12; SPEC-0006; SPEC-0017 Behavior 9; SPEC-0046).
48. Resolved: F1 and F2 (Behavior 18) scale the notice band and the emerge distance (Behaviors 3 and 40), keyed by the hero's action state. [Resolved]
49. Resolved (High): the five variant lookups of Behavior 19 are Draik Skeleton lookups by stage, not common-enemy animation indices, and the attack codes of Behavior 20 are the slot wind-up and blocked animations described there. Which clip an id plays for a given common-enemy kind is data: the table is the enemy's own record main table (Layout section 6). Shipped Werelupe Bashers, Elites, Bone Throwers and Poison Spyders provide crosswalks for IDs 3, 16, 28 and 41 to bound clip entries (Behavior 19). The 157 shipped Minion Grunt records also map IDs 3, 16, 28 and 41 to clip entries 21, 21, none and 9 respectively (Behavior 19). All 86 scanned Drackonack type 66, variant 0 records map those IDs to clip entries 11, 23, 21 and 18 in their 40-entry main clip bank. Minion Warrior type 82 adds a crosswalk across 85 records: 83 map IDs 3, 16, 28 and 41 to clips 1, 26, 3 and 9; one variant-6 record maps to 1, 31, none and 9, and one variant-0 record maps to 0, 31, none and 8 (Behavior 19). For Minion Archer (monster type 80), group-A IDs 3, 16, 28 and 41 map to clip entries 21, 21, none and 9 across all 145 scanned character records in variants 7 to 13 (Behavior 19). For variant 7, row 190 has one active attack slot at slot 4; its sequencer codes 48–52 select none, clip 14, none, clip 9 and none across all 37 checked records (Behavior 13). Test vector: row 190, slot 4, code 49 -> clip-bank entry 14; code 51 -> entry 9; codes 48, 50 and 52 -> no clip. The crosswalks above show how the Draik-stage ids 3, 16, 28 and 41 resolve in other kinds' own clip banks; no common-enemy routine reaches these ids through the stage lookups, so they carry no common-enemy meaning (the ids that common enemies play are in Behaviors 8, 13, 20 and 48).
50. Resolved (High): the lookup at 0x2DA6A0 and adjacent upper-bound table serve class-12 Draik breath standoff (10.0 to 15.0); the lookup at 0x2DA8F8 serves its stomp approach boundary (7.0). Both are indexed by Draik stage and are not common-enemy tuning. The other tables of the family are Draik tables too: the breath abort distance (0x2DA6C8, 25.0), the claw standoff pair (0x2DA77C, 0x2DA790), the wind approach limit (0x2DA808) and the ten attack-band tables (0x2DA678 to 0x2DA8D0); each getter has only Draik handlers as callers, and their values are listed in SPEC-0029 (Shared helpers and conventions, six-entry attack-band tables). No common-enemy row reads any of them. Test vector: a class-17 enemy never reads the table at 0x2DA6A0; a Draik at stage 3 reads entry 3 of it (10.0).
51. Resolved: the steering lookup of the leash is the band test of the agro tail. It needs the selected slot index 1, which the normal choice has reset to -1 in the same tick, so it never runs (Behavior 23; Edge cases). [Resolved]
52. Resolved: the words at 0xA0, 0xA4 and 0xA8 are monster-row fields (distance hide flag, distance and chance), and the distance is the enemy's distance to the hero (Behavior 24). [Resolved]
53. Resolved: the word at 0x39BE50 is the state of the shared random generator (SPEC-0008), and the distance-hide gate's roll is a closed integer draw from 0 to 100 (Behavior 24). [Resolved]
54. Resolved (High): the class-17 pre-step calls the gate when its retarget countdown reaches 0. The gate reports "blocked" in two cases: the enemy is in class state 19, its agro outer sub-state is 2 and the sequencer state is 0, 1, 2 or 3; or the object word at +0xE58 is nonzero (in any class state). Otherwise it passes. A pass re-picks the target and sets state 22 (Behaviors 2 and 21). The word +0xE58 is an attack-active latch: the only stores of 1 are in the Black Knight's slow-swing and angry-swing handlers, and the only stores of 0 are the actor setup and the generic character tick. No class-17 routine writes it, so for every class-17 enemy it is always 0 and the gate reduces to the class-state-19 sub-state test. Test vector: class state 19, outer sub-state 2, sequencer state 3: blocked. Class state 19, outer sub-state 2, sequencer state 4 or higher: passes. Class state 19, outer sub-state 1: passes. Class state 5 (any sub-state): passes.
55. Resolved: the codes 10, 0xA5 and 0x85 to 0x88 are the emerge animations of state 8 and of the alarm-group check (Behaviors 3, 20 and 40), chosen by the flyer test and the spawn record's selector word at 0xC8 (Open question 80); they are not used in place of the idle picker. [Resolved]
56. Resolved: the 5.0-unit distances in the spacing predicate (Behavior 22) are horizontal. It calls the shared helper on the candidate and the hero or another pooled actor; the helper returns horizontal distance in its fourth lane, with the height difference ignored (High).
57. Resolved in part: CharacterSetAvoidPoint supplies the scalar and point used by the state-5 proximity retarget; a recovered caller uses setting index 33 for six character-list entries. The threshold compares horizontal XZ distance, while the displacement from the stored point to the character is normalized in 3D before scaling by the same scalar. The exact design-facing name (radius versus clearance or another threshold) remains open (Behavior 25).
58. Resolved: group 1 is the slot wind-up animation A and group 2 the slot blocked animation D of the sequencer; the availability test is the animation-present test (Behavior 20). [Resolved]
- Resolved, idle timing and codes: the idle coroutine with 400 to 800 ticks and codes 0, 1 and 3 (Behavior 28) belongs to class 36 (Target Dummy); class 17 uses the idle picker of Behavior 8 (holds of 120 to 600 ticks, variants 115 to 119). [Resolved]
- Resolved, states 4 and 5: the main loop of Behavior 27 and its states 4 and 5 (Behavior 33) belong to class 34. Class-17 state 4 is the stunned state and state 5 an idle wander variant (Behavior 5). [Resolved]
- Resolved, state 5: Behavior 15 is the grab-and-throw strike and has no state-5 side handler; the 3.0 value is Open question 28. The state 5 of Behavior 33 is a class-34 state. [Resolved]
- Resolved, touched word (classes 21 and 34): the word at +0x238 of the motion record is the touched (blocked) flag of SPEC-0029 Behavior 28. It ends a Miamouse flee run (Behavior 17) and sends a live petpet to state 3 (Behavior 27). [Resolved]
- Resolved, cooldown units (class 34): cooldown B counts ticks of remaining petpet time; the ratio is 1 at 1800 ticks, 30 seconds (Behavior 27). [Resolved]
- Resolved, main loop states (class 34): states 1, 2, 3 and 6 are described in Behavior 27 (wander near the hero, follow, recover when stuck, time warning). [Resolved]
- Second main loop (class 7): the offsets of flags A and B within the loop's block, the meaning of the selector word, and the two routines it dispatches to but this spec does not describe are open. The record that the wait-and-approach step works on is not identified (it is called with two arguments). [Open]
- Band distance (class 7): the 10.0 and 3.0 tests of Behavior 29 use a distance read from the point vector, while the 5.0 test uses the target distance. If they are the same distance, the 3.0 hold is never reached. [Medium-Low; Open]
- Resolved, random helper range rule: every integer draw is closed, lo to hi with both ends included, and never falls outside the range (unsigned remainder; SPEC-0008). The draws of Behaviors 28 and 30 are [400, 800] and [120, 400]. Float draws are [lo, hi). [Resolved]
- Resolved, idle codes (class 36): the animation codes 0, 1 and 3 of Behavior 28 are the idle, twitch and hit animations of the Target Dummy (SPEC-0029 Behavior 29; Medium). [Resolved]
- Charge helpers (class 35): partly resolved. The wait animation is selected by the idle-animation picker (Behavior 8), and the post-wait animation request uses code 30. The routine or script entry that starts this class-35 coroutine remains unidentified.
- Patrol list (class 29): Resolved (High): none of the 259 monster-table rows carries default class 29 (the row classes present are 1 to 4, 7 to 18 with 17 on 109 rows, 20, 21, 32, 33, 36 to 39, and none on 119 rows), and no script or code request sets it, so the patrol walk never runs on the disc. The list owner, the point source and the selector argument therefore have no shipped use. Default if unresolved (a rewrite choice, not a fact from the code): leave class 29 unimplemented. The controller's navmesh handling, for reference: the controller checks/sets its navmesh before route iteration, and the shared selector uses actor position to select a containing mesh; this selection does not choose a patrol waypoint (Behavior 32).
- Pool release in state 5 (class 34): the meaning of the release helper on the shared object pool at 0x35C7B0. [Open]
- State 4 arrival (class 34): why the normalized-direction component is compared with 2.0, and the animation/query meanings of code 4 and the follow-up query. The observed comparison passes for finite nonzero position differences; the 2.5 test is the shared object-claim timer, not an object extent. [Open]
- Movement classes (class 7): Resolved: the owner of class 7 is the Whinny, and its movement classes are the locomotion modes of SPEC-0029 (Locomotion modes and slot words). The class (mode) selects an animation slot, and the Whinny's slots are: class 0 plays animation 2 (a 9-key looping walk), class 1 plays animation 2 and class 2 plays animation 3 (a 7-key looping run). The mover has no speed value of its own: the speed is the root travel of the clip, so class 0 moves 0.0694 units per tick (4.16 units per second) and class 2 moves 0.2252 units per tick (13.51 units per second) at playback rate 1 (SPEC-0029, Behavior 5, Speeds). Class 0 is the walk and class 2 the run; both names are confirmed by the clips (High). Steering in both is straight toward the target point (the forward-gait rule of SPEC-0029). Test vectors: Behavior 29 with a distance measure of 12.0 sets class 2 and the Whinny moves 0.2252 units per tick from the second tick on; with 6.0 it sets class 0 and moves 0.0694 per tick; with exactly 10.0 it sets class 0, with 3.0 exactly it holds (mode 1). [Resolved]
- Tick rate: the tick-rate word at 0x368FA0 is outside the file image and is written once at start-up as 60 ticks per second from the boot rate table at 0x30C598 (SPEC-0007, Rate selection; Open question 36). [Resolved]
59. Resolved: the pairing reset writes the second cooldown (+0xA9C) as a closed integer draw from 10 to 30 times the frame-rate word (600 to 1800 ticks at 60 Hz). When the sub-step setter enters sub-state 1 or 2 from sub-state 0, it writes the first cooldown (+0xA94) as a closed integer draw from 5 to 12 times the frame-rate word (300 to 720 ticks at 60 Hz). No level-module write is needed for this timer (Behavior 10b).
60. Resolved for the operational test (High): Behavior 10b's candidate predicate requires +0xC98 = 0, a mode word (+0x3B0) outside {0, 1, 2, 8, 9, 10, 11, 0x70–0x74, 0x3B, 0x3C, 0x6B}, and an AI state word (+0xA34) other than 3. The semantic purpose of the excluded values is unknown.
61. Resolved: on a successful class-6 leader handover, the shared AI-state setter receives state 3 first for the enemy itself and then for the selected character. The first call's argument was not visible in the earlier function rendering; the MIPS instruction listing confirms both calls receive 3 (Behavior 10b).
62. Resolved: the think routine is not stepped for an enemy whose base actor state is 16 (dead). The update hook steps the behaviour coroutine only in its default arm and in base states 17 and 31; base states 16, 25, 33, 36, 42, 53 and 55 skip it (Behavior 1, When the think loop is stepped). [High]
63. Resolved: the sink moves the corpse's world position down along world up by 1/180 unit per tick, and only when the corpse is not flying (Behavior 36, step 4). [Resolved]
64. Resolved: the death hook has no removal helper. Its upper branch is the drop of parts A and B, and the word it clears (0xC28) is the carried Neopoints of part D (Behavior 34). [Resolved]
65. Resolved: the second take-hit test is the ghost absorb (Behavior 37, step 5), and the pending test is the per-enemy script-hold word 0xC54, which stays 0 for class 17 (step 1). [Resolved]
66. Resolved: the point argument is the enemy's position, where the carried Neopoints are split into coins (Behavior 34, part D). [Resolved]
67. Resolved: each frame the attack manager removes the entries whose enemy is gone, dead or not in class state 19 (Behavior 45), so a dead or reanimating enemy leaves the list until it enters agro again. [Resolved]
68. Resolved: Behavior 18 is keyed by the hero's action state, not by an enemy kind, and the distance-hide gate (Behavior 24) tests the enemy's class state (not 14). No kind key is needed. [Resolved]
69. Resolved: the word at 0x35C7D4 is the current-hero pointer, which the hero-facing routines of this spec read (Behaviors 37 to 39 among them; High). The word at 0x35C7B4 is the camera pointer (SPEC-0033), not a hero pointer. The routines of this spec that read it use the camera: the aim-scatter cone of Behavior 13 and the class-34 leave test of Behavior 33 (now written with the camera). [Resolved]
70. Resolved (High; direct builder path): sub-type 5 first requires a live target entry. It then requires a free projectile slot; only inside that branch does it deal the attacker's missing HP as immediate damage and build/submit the spell. A missing target or full pool produces neither damage nor a projectile. When the created spell later hits, its projectile kind sets the target's HP to maximum (Behavior 43; the follow-up effect follows from SPEC-0029 Behavior 9).
71. Resolved: the spell lifetime is trunc(distance from the launch point to the aim point) + 5 ticks, at a speed of 1.0 unit per tick (Behavior 43). [Resolved]
72. Resolved: state 14 is entered by the distance-hide gate of the agro machine (Behavior 24), on rows with the distance hide flag and approach routine 1. [Resolved]
73. Resolved: the trigger-special test has five entries; code 1 allows the slot only while the enemy is airborne, code 2 only while it is not, and the other codes always (Behavior 7, step 5f). [Resolved]
74. Resolved: the agro outer sub-states are in Behavior 44: 5 melee dodge, 6 missile dodge, 7 melee block, 8 missile block, 9 search after lost sight, 10 hit reactions, 11 confusion, 12 wait in line; outer sub-state 2 runs the sequencer of Behavior 13. [Resolved]
75. Resolved: the slot's projectile type field (0x43C + 4k) is the visual kind of the missile spawners (Behavior 13); the spell builders of Behavior 43 do not read it and use visual kind 23. [Resolved]
76. Resolved: reaching the hit-count threshold (object offset 0xA30) sets the invulnerable word 0x7F0, after which the hit filter refuses the enemy. The threshold is initialized to -1 and has no recovered runtime writer for common enemies, so this rule never fires in the shipped common-enemy path (Behavior 37, step 12). [Resolved]
77. Resolved: offset 0xA0 is one field, the monster row's distance hide flag. It enables the distance-hide gate (Behavior 24) and makes the state 8 handler hide in place (Behavior 40). [Resolved]
78. Resolved: the second constant 23 is the projectile's visual kind, the same visual kind the lobbed summon uses (Behaviors 13 and 43). [Resolved]
79. Answered (Behavior 34, step 6): the drop action of a key -1 placement pays its own coins (the money amount, or else the fixed count) and rolls an item only when it pays none. The carried Neopoints are split into coins separately on the same first tick. The two routes that assign direct drop item 172 are the Darkest Faerie’s class-37 member summon and volley. For row 128, they request keys 2864 (Minion Warrior) and 2844 (Minion Grunt), respectively. Each spawn requires an unbound summon-reserved record for its key in the summoner’s block, a free character object and active-count capacity; the successful summon drops item 172 on its first death tick (SPEC-0029, Behavior 31 and Open question 158). [Answered]
80. Answered (Layout section 1): the word at block offset +0xC8 (decimal 200) of the second data block is not the variant. It is a tick value used as the end of a random range whose start is the word at +0xC4, and it is also read as a selector for codes and animations (values 0 to 4). The monster row variant is the u16 at data offset 0xB2 (SPEC-0006), a different field. [Answered]
81. Resolved: the dead test of streaming is true in base state 16, or in class state 16 with the reanimation countdown above 0. A lying reanimating enemy therefore gets its kill bit on every tick of its countdown; its bind ignores the bit (Behavior 47). [Resolved]
82. Resolved: p = normalize(dir x u) with u = (0, 1, 0), so for dir = (dx, 0, dz) it is (-dz, 0, dx) / |dir|. Inner 8 steers to own position - 2.0 p and inner 9 to own position + 2.0 p. In the right-handed, y-up frame of SPEC-0004, dir x u points to the enemy's right as it faces the hero, so inner 8 circles to the enemy's left and inner 9 to its right (Medium-High for the side, which rests on the frame's handedness). [Resolved]
83. Resolved: the capacity indices are the weapon (Tor id - 98, Roberta id - 111) and the body item (Tor id - 119, Roberta id - 127, item 126 = 0), -1 when the slot is empty, so the capacity is -6 to 42 (Behavior 45). [Resolved]
84. Resolved: the value the take-hit hook writes to the held hero's base state is 1. The grab phase word 0xC08 is 2 while the enemy holds the hero, so a hit on that enemy frees the hero (Behavior 37, step 4). [Resolved]
85. Resolved: the dead state's repeated request does not restart the death clip; it plays once from the tick of death (Behavior 36, step 2). The take-hit hook's force-start is the restart path for live enemies and does not reach a dead one (Behavior 37, step 13). [Resolved]
86. Resolved: the class-17 hit-reaction entry sets both its outer-state word (+0xB30) to 10 and its state-19 sub-state word (+0xAF8) to 10. The attack manager reads the latter for its reorder exclusion; these are distinct fields (Behavior 45; High). [Resolved]
87. Resolved (R-0015). (1) Resolved (Medium-High): a passing notice test in alerted search (state 2) starts the fight. The notice test runs every tick in state 2, and a pass sends the enemy to agro (19) after its search animation reaches the 0.99 progress gate (Behavior 5, state 2). Wandering states 5 to 7 reach agro through state 2, not directly. A hit, the alert flag and the emerge are further ways in. (3) Resolved (Medium-High): "play to its end" is the progress gate of 0.99 on the current animation, the same test that the sequencer and the state waits use, not the finished flag. A looping clip passes the gate on the tick its progress reaches 0.99, which depends on how the animation system counts progress across a loop (SPEC-0005). (4) Resolved (Medium-High): the sight-line points are the aim points given by the class routine at slot 0x294 (Behavior 3, step 4). (2) Resolved (Medium-High for the rule, Medium for the mode numbers): the facing is not the raw steered direction for the back and side gaits. It follows the enemy's locomotion mode (Behavior 48, item 7). Back gaits (23, 26, 32 on modes 3, 4 and 8) face the reverse of the move direction, and side gaits (29 and 30 on modes 5, 6, 9 and 10) face perpendicular to it. On a strafe or keep-distance step the steer runs sideways or away, so the enemy ends up facing the hero. (5) Answered in Behavior 44 at Medium.





88. Resolved (R-0100): side gait modes 5 and 6 request ground clips 29 and 30; for Can Fly 1/2 characters off the ground, the mode setter maps those modes to flyer modes 9 and 10, whose ordered clips are 105 and 106 respectively. The class-17 row for the pack-1398 Evil Harris has Can Fly 0, so neither the remap nor flyer-specific reaction applies to these Harris. Kind 7 selects an animation and exits when speed reaches its threshold; it does not itself establish who sets velocity or direction. [High for the mode remap and Harris row; Medium-High for clip correspondence from the ordered mode tables]
