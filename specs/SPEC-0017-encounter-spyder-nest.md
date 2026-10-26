# SPEC-0017: Encounter behavior: Spyder Nest

- Answers: proactive, none requested (boss AI and encounter behavior)
- Status: Draft
- Disc build: SCUS-97367 v1.00 (same fingerprint as SPEC-0001)
- Confidence: per section. Tunable defaults and addresses: High (taken from the boot program image). Level object fields: High. Setup, timing and the valve state sweep: High. Nest state: High (the nest starts active; idle is unreachable). Valve state machine: High for transitions and animations. Hit rules: High for the window, the two branches and their HP costs; High for the valve cylinder's centre and axis (the valve's root bone) and for the absence of a direction gate at the attacking call. Spawn placement: High for the gate and the offsets, Medium-High for the placement rule. Animation speed-up: High. Spider release and death: High for the state steps; Medium for the spider chooser (see SPEC-0038). No nest or valve attack, and the spiders' damage path and row values: High. Valve hit flash 222 and the spider sound cues: High. Reachability (the encounter never runs on this disc; no record places the nest or a valve, and no valve or nest model, animation or sound exists): High. Start-up world binding: Medium. Presentation (a debug text only): High. Spider mode after release (agro, mode 19): High.

## Summary

The Spyder Nest is an encounter controller with three valves, each of which holds a group of spiders. A valve is closed while its spiders are alive. When every spider of a valve is dead, the valve waits 10 s, opens, and releases its spiders again. The hero damages a valve by hitting it while its hit window is open. The window stays open until a hit lands, so a valve can be hit at any point of its cycle, not only while it is open. Each valve has 3 hit points, and by default each open cycle allows one hit, so a valve needs three cycles to die. Each destroyed valve speeds the nest's animation up by a factor of 1.2. When a valve reaches 0 hit points it is destroyed and its living spiders take 1000 damage. When all three valves are destroyed, the nest enters its defeat state. A clean implementation can build the setup, the valve cycle, the hit rule and the defeat from this spec. Spider behavior is not covered here; SPEC-0038 (Behavior 12) gives the spider states the nest uses.

**The encounter is not reachable on this disc (High).** The nest controller is the think routine of behaviour class 3, and each valve is a character of class 4, which yields every tick and does nothing (SPEC-0029, Layout 3). Class 3 occurs only on monster row 12 and class 4 only on rows 13 to 15 (types 10 to 13). No placement record of types 10 to 13 exists in any region or world pack, and no record selects rows 12 to 15 at load, including by a variant that overflows from another type. The only run-time route in code is the variant re-pick of SPEC-0029 (Run-time variant re-pick): it sets a placed character's variant to 0 to 5 and reads first row of its type plus that variant, so a re-picked type 6 character (variant 5, row 12) or type 8 character (variants 2 to 5, rows 12 to 15) could land on a nest or valve row, and type 7 and 9 characters could too, but those two types are placed only in the unreached pack 1259. The re-pick needs a Minion Wizard to hit the target, changes only the row and palette, and only a later default-class request would read the new class. The reachable type 6 and 8 characters are the Juppie monsters of the Juppie stage and the Plague Serpent; no shipped placement puts a Minion Wizard in the same resident set with either (Medium), so this is a code possibility, not an encounter. No class request in the boot program passes class 3 or 4 (the only explicit classes requested are 23 to 26, 34 and 35; SPEC-0029). The level-module API inventory has no class-request call: the generic SetAI_Behavior API changes an enemy AI mode and its parameters, not its behavior class (SPEC-0030). No level module names the nest, its valves or its states, and no story row, flag or movie belongs to it. So no pack holds the nest, no level module drives it, and it has no trigger, gate, win or lose rule, flag, movie or exit. Spiders do appear in play, but only as ordinary class-17 common enemies (SPEC-0038). A clean implementation may omit the encounter, or implement it from this spec as unused content; the per-tick rules below describe the code as it would run if a level placed the nest.

## Where it lives on the disc

- The tunables are 4-byte words in the boot program image at the addresses in the Layout tables. Read them through the ELF program headers of the boot program (SCUS_973.67). Integers are signed 32-bit. "float" means IEEE single.
- The nest's tunables are six words from 0x2DB0A0 to 0x2DB0B4. Four further words at 0x2DB0B8 to 0x2DB0C4 are read by the spawn step (see Layout).
- Valves and spiders are bound characters of the current world, found by the class field of their monster row and the behaviour code of their placement record (see Layout).
- The nest would be a character whose monster row has class 3 (row 12, type 10, variant 0), and the valves characters of rows 13 to 15 (types 11, 12 and 13). Resource 530 holds those rows; no placement record (kind-3 entry of any type-36 chunk, SPEC-0006) selects them, so no world holds the nest or a valve (High).
- Spider and valve animation numbers are entries of each character's own animation tables (SPEC-0005). No region holds a model, animation table or sound word for types 10 to 13, so the valve and nest animations (0 to 4) have no data on the disc (High).

## Layout

### Tunable values

| Tunable (descriptive name) | Type | Address | Default | Used |
| --- | --- | --- | --- | --- |
| Spiders per cycle, valve 0 (normal spiders) | int | 0x2DB0A0 | 5 | yes |
| Spiders per cycle, valve 1 (poisonous spiders) | int | 0x2DB0A4 | 3 | yes |
| Spiders per cycle, valve 2 (giant spiders) | int | 0x2DB0A8 | 2 | yes |
| Valve hit points | int | 0x2DB0AC | 3 | yes |
| Hit points removed per hit (melee hit only) | int | 0x2DB0B0 | 1 | yes |
| Pulsation animation speed increase | int | 0x2DB0B4 | 20 | no (unused in the original; no load or store in the boot program, High; keep the value) |

Internal constants read by the spawn step:

| Constant | Address | Value | Meaning |
| --- | --- | --- | --- |
| Spawn launch speed, forward | 0x2DB0B8 | 10.0 | Velocity added along the valve's forward axis |
| Spawn launch speed, up | 0x2DB0BC | 10.0 | Velocity added to the world vertical (y) part |
| Spawn offset, forward | 0x2DB0C0 | 5.0 | Distance from the valve point along its forward axis |
| Spawn offset, up | 0x2DB0C4 | 0.0 | Distance from the valve point along its up axis |

Fixed values of the nest (not tunable words): the animation speed base 1.2, the melee cylinder (radius 2.0, length 4.0), the shot-slot distance 3.0 plus the slot's radius (not attacking), the shot-slot fallback 5.0 plus the slot's radius (attacking), and the closed countdown 600 ticks.

### Valve and spider objects in the level

Each bound character points to its monster row (resource 530 row, SPEC-0029 Layout 4) and to its placement record (kind-3 data, SPEC-0006). The fields below are offsets inside those records (High).

| Field | Record | Offset | Size | Meaning |
| --- | --- | --- | --- | --- |
| Class | monster row | +0x36 | u16 | The row's default behaviour class; 4 for a valve; the valve is then tested for its index |
| Valve index | monster row | +0x38 | u16 | The row's approach-routine field, read here as the valve index 0, 1 or 2; selects the valve record |
| Spider kind | placement record | +0x84 | s16 | The placement's behaviour code (SPEC-0056): 5 = valve 0 (normal spiders), 7 = valve 1 (poisonous), 6 = valve 2 (giant) |

- On disc: row 14 (type 12) has class 4 and index 0, row 13 (type 11) class 4 and index 1, row 15 (type 13) class 4 and index 2. Row 12 (type 10) has class 3 and index field -1 (High).
- A character is a valve when its class is 4. Every other character is tested for the spider kind, whatever its type: any character of the world whose placement behaviour code is 5, 6 or 7 joins a valve's list. Codes 5, 6 and 7 are ordinary behaviour codes (meander, patrol and guard, Medium for the names; 628, 294 and 308 of the placement records on disc), so in a world with a nest, townsfolk with these codes would be adopted as spiders too (High for the rule).
- Each valve may hold up to 10 spiders.
- A spider entry stores the spider's handle (u32 at +0x14C of the level object), not the object itself. The handle resolves to the object at runtime; a handle that does not resolve is treated as dead (see Living spiders).

### Per-fight state (not on disc)

- Nest state: active (1) or defeated (2). The nest is set to active when the encounter starts. Idle (0) is never assigned in the shipped game, so there is no step from idle to active, and no idle branch is run (see Open questions).
- Per valve: state code (0 to 6), HP (starts at the valve HP tunable), spider count (copied from the tunable), spider list (10 slots), countdown timer, spawn-once flag, hit window flag.
- Per spider: state, HP, action code, despawn flag, navigation area, position and velocity (the level object's own fields).

## Behavior

### 1. Setup (once, when the encounter starts)

1. Set the nest state to active (1). This happens before the scan.
1a. **Bind the whole world (Medium).** In the nest's world slot (SPEC-0030, world slot fields): set the character cap to 14, then turn the keep (streaming) flag off, which sets the cap to 15. Then, for each character record index i from 1 to the record count minus 1, release the character at position i (no kill bit); afterwards, for each i in the same range, bind record i into position i. Record 0 is left alone (it would be the nest itself). So every record of the world is bound at once and streaming stops, and the scan sees all valves and spiders. With the default counts the fight needs 1 nest + 3 valves + 10 spiders = 14 characters, within the cap of 15.
2. Clear the three spider lists and mark every slot empty.
3. Scan the world's active characters in position order (0 to the active count minus 1). Each valve is attached to the valve record for its index. Each spider (kinds 5, 6, 7) is added to its valve's list in level order, and is hidden and given state 16 (the inactive or death state of SPEC-0038, which the nest reads as dead). Hidden spiders count as dead until they are revealed (section 4). This is what keeps closed valves closed.
4. Wait 180 ticks (3 s).
5. Start the per-tick loop.

The scan runs before the wait. Nothing in the valves changes during the wait.

### 2. Per-tick loop

Each tick, in this order:

1. **Tunable copy.**
   - Spider count: every tick, each valve's count is set to its spider-count tunable.
   - Valve HP: each valve keeps a copy of the HP tunable it last saw. When the tunable differs from that copy, the valve HP is set to the tunable and the copy is updated. So at the start the HP is set to 3, and a runtime change to the tunable resets every valve's HP to the new value.
2. **Valve sweep**, for valves 0, 1 and 2 in that order. Let N be the valve's spider count and L the number of living spiders among the first N entries of its list. An entry is living when its handle resolves to an object whose state is not 16. An unresolved handle is not living. Entries after the first N are ignored.
   - If the valve is not destroyed and L > 0, its state becomes 4 (alive), whatever its state was.
   - If the valve is not destroyed and L = 0 and its state is not 1, 2 or 3, its state becomes 1 (closed), and the countdown is set to 600 ticks.
   - Consequence: a fresh valve (state 0) goes to closed at once, when it has no living spiders.
3. **Nest update.** Let d be the number of destroyed valves.
   - If d = 3, the nest becomes defeated (state 2).
   - Otherwise the nest's animation playback rate is set to 1.2 to the power d (section 6), applied after the sweep.
4. **Nest state.**
   - Active (1): update each valve's state machine (section 3), valves 0, 1 and 2 in order.
   - Defeated (2): play the defeat animation (4) on the nest, if it is not already playing. The valve machines do not run after defeat.
   - Idle (0): not reached in the shipped game (see Nest state). No idle branch is run.
5. **Presentation (High).** The nest writes a debug text into its own character's debug text buffer: the nest state name (none, fighting, destroyed), then three lines, sack A, B and C, which are valves 1, 0 and 2 in that order, each with its state name (none, pre-open, open, launch, idle, damaged, dead for states 0 to 6) and its HP, and a range line taken from the horizontal distance between the nest and the hero (its number format is Medium). It changes no game state (High); that no retail screen shows the buffer is Medium. A clean implementation may skip it.
6. Wait one tick.

### 3. Valve state machine

States and what they do:

| State | Behavior | Next |
| --- | --- | --- |
| 0 initial | Plays idle animation (0). | The sweep moves it to closed when no spiders are alive. |
| 1 closed | If the countdown is below 1, go to 2. Otherwise subtract 1. No animation. | Opening (2) after the countdown. |
| 2 opening | Plays animation 1. When it is playing and finished: open the hit window and go to 3. | 3 |
| 3 spawn | Plays animation 2. On the first tick animation 2 is playing and unfinished, and the spawn-once flag is clear, set the flag and reveal the spiders (section 4). When animation 2 is playing and finished: close the hit window and go to 4 (rare: see the hit window). | 4 |
| 4 alive | Plays idle animation (0). Waits. | Set by the sweep when spiders die. |
| 5 hit reaction | Plays animation 3 and the shared hit flash, one-shot effect 222, at the valve point (SPEC-0034). | Never reached in normal play: the sweep moves it out before its step runs (Edge cases). |
| 6 destroyed | Requests idle animation (0) and then destruction animation (4) each tick. A request only stores the main id, and the clip starts later in the tick's animation pass (SPEC-0040 Behavior 3D), so the second request wins: animation 4 starts on the destruction tick and is not restarted while it plays, because the already-playing test passes (High). | Final. |

After the state step, in the same tick:

- If the hit window is open and the valve is not destroyed, run the hit test (section 5).
- If the valve's HP is exactly 0 and the valve is not destroyed, the valve is destroyed: its state becomes 6, it plays destruction animation 4, and every living spider in its list takes 1000 damage.
- For each spider in the valve's list whose state is 16, its death animation (58) is playing and finished: set its despawn flag (section 9).

### 4. Spawn and reveal

- Reveal happens once per cycle, controlled by the spawn-once flag. The flag clears when the valve enters the closed state.
- The spawn first searches the current world's navmesh list for the first ground mesh (kind 1; the search runs in mode 0, SPEC-0018 navmesh queries) with a triangle that contains the hero's position. Flying meshes (kind 2) are never chosen (High). If none is found, the search returns no area (-1), nothing is revealed that cycle, and the flag is still set, so the cycle does not retry. (Medium-High.)
- If an area is found, each of the first N entries of the valve's list (N = the valve's current spider count) whose handle resolves to an object is revealed. Other entries are skipped.
- Each revealed spider:
  - base state := 1 (active), AI mode := 19 (agro), HP := its maximum, despawn flag := 0. Setting mode 19 is the shared agro entry of the common enemies (SPEC-0038, SPEC-0029): it posts the spider's group alert when its placement has a group, joins the attack manager, and draws a new attack timer from its row. So every released spider hunts the hero at once, with no idle, patrol or notice phase (High);
  - navigation area := the hero's area;
  - position := the valve point + 5.0 along the valve's forward axis + 0.0 along its up axis;
  - velocity gains 10.0 along the forward axis and 10.0 on the world vertical axis. The velocity is the body's motion-record velocity, which the shared integrator applies as position += velocity x (1/30) each tick (SPEC-0040 Behavior 14A), so 10.0 moves the spider 1/3 unit per tick (20 units per second at 60 ticks per second) before gravity and friction act (High for the unit; Medium-High that the vertical part is world y, not the valve's own up axis);
  - plays its idle-variant animation, chosen by the spider's own chooser (SPEC-0038).
- Revived spiders reuse the same objects; no new spiders are created.

### 5. Hit window and hit rules

**Hit window.** The window opens when the valve's opening animation (1) finishes. It closes only when a hit lands, or in the spawn state when animation 2 finishes while no spider is alive. In a normal cycle the spawn state never sees animation 2 finish, because the sweep moves the valve to alive on the tick after the reveal. The window therefore stays open through the alive phase, through closed phases, and through the next cycle until a hit lands. Confidence: High.

**Hit tests** run each tick while the window is open and the valve is not destroyed, in any valve state.

The hit test has two branches, chosen by whether the hero is attacking (the hero's attacking query at its class table slot +0xE4 reads 0). Both run only while the window is open.

- **Not attacking.** Each of the hero's six shot slots that is armed and whose point lies within 3.0 plus the slot's radius of the valve point hits. This is a point-against-sphere test (SPEC-0036), not the cylinder test. A hit sets the valve to state 5 and removes exactly 1 HP (a fixed value). This branch does not disarm the slot (High).
- **Attacking.** The valve's cylinder (radius 2.0, length 4.0; fixed values, not disc words) is the second shape of the static finite-cylinder overlap of SPEC-0018. The first shape is the hero's body cylinder (radius 1.0; SPEC-0036). The valve cylinder is placed from the world frame of the valve's root bone (bone 0 of its posed skeleton): its axis is that bone's own up axis (the second row of the bone's world frame), and its centre is the bone's world position plus 2.0 (half the length) along that axis, so the cylinder runs from the bone position to 4.0 along its up axis (High). If that misses and the segment's two stored endpoints differ, a swept test of the valve cylinder runs (SPEC-0018 sweep). If both miss, the six shot slots are tested as in SPEC-0036: each armed slot is tested with a point-against-sphere test of radius 5.0 plus the slot's radius, centred on the root bone's world position. The call passes no direction gate (its gate switch is 0), so the 0.0 threshold is never applied and every armed slot within range counts (High). A slot that touches the valve is disarmed and the test hits. A hit in this branch removes the "hit points removed per hit" tunable (default 1).
- **On any hit:** the window closes and the valve's state becomes 5. The HP reduction depends on the branch, as above.

### 6. Animation speed

- Each tick, after the sweep, the nest's animation playback rate is 1.2 to the power d, where d is the number of destroyed valves: 1.0 at the start, 1.2 with one valve destroyed, 1.44 with two.
- When d = 3 the nest goes to defeat and the rate stays at the value set on the previous tick (1.44, since that tick had d = 2).
- The "pulsation animation speed increase" tunable (0x2DB0B4) is unused in the original (High).

### 7. Respawn cycle

Typical sequence, from the first closed state:

1. Closed (state 1) for 600 ticks (10 s).
2. Opening (state 2) for the length of animation 1. The window opens when it finishes.
3. Spawn (state 3). The spiders are revealed on the first tick animation 2 is playing and unfinished.
4. Alive (state 4) on the next tick, while spiders are alive, until every spider in the valve's list is dead.
5. When the last spider dies, the sweep moves the valve to closed (state 1), and the cycle repeats.

Consequences:

- The hit window stays open until a hit lands. Since a hit closes the window, at most one hit lands per cycle.
- A valve with 3 HP needs three hits, one per cycle, so it can take three cycles. Each cycle requires the player to kill all of that valve's spiders first.
- Destruction on the third hit happens in the same tick as the hit.

### 8. Defeat

- When all three valves are destroyed, the nest becomes defeated (state 2). Each tick, the nest plays animation 4 if it is not already playing.
- The defeat has no exit (High). The controller keeps looping: it still copies the tunables, sweeps, and writes the debug text, but sets no story flag, starts no movie, gives no reward and moves the hero nowhere. Any outcome of the fight would have had to come from a level module, and none refers to it.

### 9. Spider death

- A spider in state 16 whose death animation (58) is playing and has finished gets its despawn flag set (High).
- The state-16 behavior and the spider's chooser are in SPEC-0038 (Behavior 12).
- Spider behavior otherwise is not covered.

### 9a. Damage to the hero (High)

- The nest and its valves never attack the hero. Neither has a hit routine of its own; their only effects on characters are the 1000 damage a destroyed valve deals to its own spiders.
- All damage to the hero comes from the spiders. They are class-17 common enemies (SPEC-0038), so their hits use the shared enemy chain (SPEC-0029 Behaviors 10.1 and 10.5): the hero's shield block, armour, armour Mote and petpet factor apply, unlike a boss's melee hits.
- Attack damage words of the spider monster rows (attack slots 0, 1 and 2; "none" = no value in that slot): Spyder (row 234) 3, none, 7, with a blocking angle of 0.966; Poison Spyder (row 235) 5, none, 15; Giant Spyder (row 236) 5, 20, 15.

### 9b. Sounds and effects

- **Nest code.** The only effect the nest controller starts is the shared hit flash, one-shot effect 222 (group 3 of the shared effect resource 9), at the valve point when the valve step runs in state 5, together with animation 3. That step is not reached in normal play (Edge cases), so the flash does not show (High).
- **Spiders.** The spiders' sounds come from their own animation tables, through the world-character sound rule (SPEC-0031): the cue of a sound entry plays at the spider on its creature block's slot when that animation starts, and a keep-alive entry is refreshed while the animation plays. All spiders use creature block 351 (sound group 1). Spyder and Poison Spyder: skitter cues 8 and 7, ambush 3, go aggressive 1, roll loop 4 (keep-alive), attack 0, take damage 5, dodge 2, die 6, poison bite 9. Giant Spyder: the large-spider cues 10 to 18, with the roll loop 14 (keep-alive). The step does not run while a character's base state is 42 (High).
- **Valves and nest.** No world record holds a valve or the nest (types 10 to 13 have no placement record and no character region), so they have no animation tables and no sound words on the disc. Their only sound or effect would be the hit flash above (High).

### 9c. Spiders of the fight (High unless marked)

- Per cycle, with the defaults: valve 0 releases 5 normal Spyders, valve 1 releases 3 Poison Spyders, valve 2 releases 2 Giant Spyders, 10 in all. Which monster row a spider uses comes from its own placement; the nest only checks the behaviour code (Layout).
- Respawn: the same characters come back. A valve releases again only when every spider among its first N entries is dead (or hidden), and then after 600 ticks plus the opening animation. Each release restores full HP, so no spider stays dead while its valve stands.
- When a valve is destroyed, its spiders take 1000 damage on every tick, so its group is gone for good. The other valves keep cycling.
- Spiders are released straight into AI mode 19 (agro) on the hero's ground mesh, launched out of the valve (Behavior 4). From there they run the class-17 agro machine of SPEC-0038 with the rows' values: all three spider rows use approach routine 4 (strafe). HP: Spyder 25, Poison Spyder 16, Giant Spyder 150. Attacks as in Behavior 9a.
- Death of a spider: the shared enemy death (state 16, death animation 58, then despawn flag), after which the valve counts it dead.

### 9d. Intended play sequence (as the code would run; not reachable on this disc)

1. The hero enters the nest's world. The nest binds every character of the world and hides all spiders; nothing moves for 3 s.
2. Each valve finds no living spider, closes, and waits 10 s. The three valves cycle in step, because they share the same timings.
3. Each valve plays its opening animation; when it ends, the valve becomes hittable and its spiders burst out toward the hero and attack at once.
4. The player can strike a valve now, or while fighting, or later: one hit per valve per cycle counts (a melee hit, or a shot while not attacking, Behavior 5). The hit valve loses 1 HP.
5. To get the next hit on a valve, the player kills all of that valve's spiders; 10 s later the valve opens again and releases them with full HP.
6. After three counted hits a valve is destroyed and its spiders die. Each destroyed valve makes the nest pulse 1.2 times faster.
7. When the third valve is destroyed the nest plays its defeat animation. Nothing else happens; the player leaves the world by its ordinary exits.

### 10. Timings

| Item | Value |
| --- | --- |
| Setup to first sweep | 180 ticks (3 s) |
| Closed countdown | 600 ticks (10 s), plus one tick before the valve opens |
| Opening animation (1) | length of the animation |
| Spawn (animation 2) | spiders revealed on the first tick the animation is playing and unfinished |
| Destruction | same tick as the lethal hit |
| Destruction animation (4) | no length on the disc (no valve animation data); no rule waits for it to finish |

## Edge cases and error handling

- Changing the HP tunable at run time resets every valve's HP to the new value. Changing the spider counts takes effect at the next spawn.
- Exact-zero test: a valve is destroyed only when its HP is exactly 0. If the amount removed is greater than the remaining HP (for example hit points per hit of 2 against a valve with 1 HP), the HP goes negative and the valve is never destroyed. Reproduce this as the original behaves.
- **Hit-reaction state (5) never shows** (High). The sweep runs before the valve step on every tick and moves a state-5 valve to alive, or to closed when no spiders are alive, before its step runs. Animation 3 is therefore not requested and the valve's hit flash 222 does not show in normal play.
- **List registration has no bounds check** (reproduce it). Each valve's list has 10 slots, but registration does not stop at 10. Spiders beyond the tenth write into the valve's following state fields: the 11th overwrites the countdown, the 12th the spawn-once flag, and the 13th the hit-window flag. Setup is the only place that registers spiders.
- **Living count uses the first N entries.** Entries after the first N (N = the spider count) are ignored by the sweep, and an unresolved handle is not living. A spider count above the number of registered spiders reveals only the registered ones.
- In the opening (2) and spawn (3) states the sweep moves a valve to alive only when at least one spider is alive, and otherwise leaves it alone.
- Destroyed valves request the idle animation (0) and then the destruction animation (4) on each tick. Living spiders in a destroyed valve's list take 1000 damage each tick, so they are dead after the first tick.
- Placement failure: if no navigation area contains the hero at the spawn tick, nothing is revealed that cycle, and the spawn-once flag is already set, so the cycle does not retry. The valve still reaches closed when no spiders are alive.
- Spider kind to valve mapping follows the kinds above (High for the numbers; Medium for the kind names).
- No ground mesh: in a world with only flying meshes, or with the hero off every ground mesh, every reveal fails and the valves stay closed and empty after each countdown.
- Two animation requests on one tick: only the last request of the tick counts (Behavior 3, state 6).
- On disc no placement set fits the mapping fully. The Abandoned Mines region with spiders holds Poison Spyders with code 7 and Giant Spyders with code 6 (the codes of valves 1 and 2) but no Spyder with code 5, and it holds no valve; the Root Cellar region holds code-5 Spyders but its Giant Spyder has code 7. Low that either was meant as the nest's arena.

## Test vectors

Synthetic values. Defaults are the tunables above.

- Encounter starts at tick 0. The nest state is 1 (active) from tick 0 and no tick sets it to 0. The first sweep runs at tick 180.
- Valve in state 0, no living spiders: after the sweep, state 1 with countdown 600.
- Valve in state 4, no living spiders: after the sweep, state 1 with countdown 600.
- Valve in state 1, countdown 600, no living spiders: stays in state 1, countdown decreases by 1 each tick.
- Valve in state 1, countdown 0: the next tick goes to state 2 (no decrement on that tick).
- Valve in state 5 with 2 living spiders: after the sweep, state 4.
- Valve in state 5 with no living spiders: after the sweep, state 1.
- Valve in state 3 with 1 living spider: after the sweep, state 4.
- Valve with HP 1, the hero in melee, a melee hit that removes 1: HP 0, valve destroyed in that tick, each living spider of the valve takes 1000 damage.
- Valve with HP 1 and a melee hit that removes 2 (hit points per hit set to 2): HP -1, not destroyed.
- Hero not attacking, one armed shot slot with its point at 2.9 from the valve point (slot radius 0): a hit, state 5, HP -1. At 3.1: no hit.
- Valve cylinder: root bone at (0, 0, 0) with its up axis (0, 1, 0): the cylinder is centred at (0, 2, 0), radius 2.0, spanning y = 0 to 4. A valve tilted so that the bone's up axis is (1, 0, 0) gives the centre (2, 0, 0) and a horizontal cylinder from x = 0 to 4.
- Attacking fallback: an armed shot slot of radius 0 at 4.9 from the root bone, behind the valve, hits (no direction gate); at 5.1 it misses.
- Hit window: opened at the end of animation 1, no hit yet. After many cycles with no hit the window is still open; the first hit closes it.
- Two destroyed valves: animation rate 1.44. Three destroyed valves: nest defeated.
- Spider count tunable 12 with 8 registered spiders: 8 spiders revealed.
- Spider count tunable 2 with 10 registered spiders: the first 2 revealed.
- Living count with spider count 2 and a list of [dead, alive, alive]: living = 1 (only the first two entries count).
- A valve with index 1 receives spiders of kind 7 only.
- Spawn placement: valve point (0, 0, 0) with forward (1, 0, 0) and up (0, 1, 0): a spider starts at (5, 0, 0) with velocity (10, 10, 0) added. Without gravity, friction or contacts this would move it 1/3 unit per tick along x and along y; the integrator of SPEC-0040 Behavior 14A applies those forces first.
- A hero standing next to a closed valve for any length of time takes no damage from the valve or the nest.
- Reachability: over all 3,036 placement records on disc, the count with type 10, 11, 12 or 13 is 0; monster rows with class 3: only row 12; with class 4: rows 13, 14, 15.
- Valve index: a character of row 14 attaches to valve 0, row 13 to valve 1, row 15 to valve 2.
- Adoption: a townsfolk character with behaviour code 6 in the nest's world joins valve 2's list and is hidden at setup.
- Placement search mode: the hero stands inside a triangle of a flying mesh (kind 2) and of no ground mesh: no area, nothing revealed. Inside a triangle of ground mesh index 3 (the first ground mesh that contains the point): the revealed spiders get navigation area 3.
- Same-tick requests: a destroyed valve requests 0 then 4 on every tick. On the destruction tick animation 4 starts; on each later tick 4 is already playing, so it keeps its cursor and plays through.
- Release mode: a revealed spider has base state 1 and AI mode 19, and its previous mode is kept as the prior mode.
- Debug text with nest state 1, valve 1 in state 4 with 2 HP, valve 0 in state 1 with 3 HP, valve 2 in state 6 with 0 HP: nest "fighting"; sack A "idle" 2; sack B "pre-open" 3; sack C "dead" 0.

## Open questions

1. Starting nest state. Resolved: the nest starts active (1) before the setup scan, and the idle branch is unreachable (Per-fight state; High).
2. Spider behavior beyond the nest's own release and state-16 checks is covered in SPEC-0038: Behavior 12 describes spider release, inactive/death handling and chooser; Behavior 44 describes the agro machine entered at release; Behavior 48 describes common-enemy movement. The nest and valves themselves do not attack (Behavior 9a; High).
3. Answered: see SPEC-0038 for the spider chooser's details (hold counter and the kind-dependent variant).
4. Units of the spawn launch velocities. Resolved: motion-record velocity, applied as velocity x (1/30) per tick, so 10.0 is 20 units per second; the vertical part is added on world y (see Behavior; High for the unit, Medium-High for the axis).
5. The placement helper's area-type filter argument. Resolved: the search runs in mode 0 on the current world, so only ground meshes (kind 1) are searched, at the hero's position (Spawn and reveal; High).
6. Length of the destruction animation (4) and the outcome of two animation requests on the same tick. Resolved: the last request of the tick wins, so animation 4 starts once and plays through; it has no length on the disc, because no valve animation data exists, and no rule waits for it (Valve state machine, Timings; High).
7. Effects and sound ids. Answered: the nest's effect and the spiders' cues are in Behavior 9b (High). Resolved: no world record holds a valve or the nest, so they have no animation tables or sound words (Behavior 9b; High).
8. The presentation routine. Resolved: a debug text of the nest and valve states, HP and range; no game effect (Per-tick loop, step 5; High).
9. Whether the threshold gate of the shot-slot step, used at the attacking call, matches the hero's intended rule. Resolved: the call turns the gate off, so the threshold never applies and any armed slot in range counts; reproduce that (Hit tests; High).
10. The attacking branch's valve cylinder. Resolved: axis = the valve root bone's up axis, centre = the root bone's world position + 2.0 along it; the call passes no direction gate, so the fallback shot-slot test has none (see Hit tests; High).
11. The centre of the valve cylinder. Resolved: 2.0 (half of the length 4.0) along the root bone's own up axis from the root bone's world position (High).
12. Which pack and level module hold the fight, and its triggers, flags, movies and exit. Resolved: none; the encounter is not placed on this disc (Summary; High).
