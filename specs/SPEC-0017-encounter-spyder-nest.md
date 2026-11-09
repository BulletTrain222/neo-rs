# SPEC-0017: Encounter behavior: Spyder Nest

- Answers: proactive (weapon box); proactive, none requested (boss AI and encounter behavior); proactive (fight AI build guide); proactive (timing pass); proactive (reach pass 3); proactive (reach pass 2); proactive (movement pass); proactive (enemy sweep); proactive (open close-out)
- Status: Draft
- Disc build: SCUS-97367 v1.00 (same fingerprint as SPEC-0001)
- Confidence: per section. Tunable defaults and addresses: High (taken from the boot program image). Level object fields: High. Setup, timing and the valve state sweep: High. Nest state: High (the nest starts active; idle is unreachable). Valve state machine: High for transitions and animations. Hit rules: High for the window, the two branches and their HP costs; High for the valve cylinder's centre and axis (the valve's root bone) and for the absence of a direction gate at the attacking call. Spawn placement: High for the gate and the offsets, Medium-High for the placement rule. Animation speed-up: High. Spider release and death: High for the state steps; Medium for the spider chooser (see SPEC-0038). No nest or valve attack, and the spiders' damage path and row values: High. Valve hit flash 222 and the spider sound cues: High. Reachability (the encounter never runs on this disc; no record places the nest or a valve, and no valve or nest model, animation or sound exists): High. Start-up world binding: Medium. Presentation (a debug text only): High. Spider mode after release (agro, mode 19): High. Spider attack ids, clips and tick counts (Behavior 9e): High for the clip facts, Medium-High for tick offsets. Valve and nest animation lengths: no data (rewrite choice).

## Summary

The Spyder Nest is an encounter controller with three valves, each of which holds a group of spiders. A valve is closed while its spiders are alive. When every spider of a valve is dead, the valve waits 10 s, opens, and releases its spiders again. The hero damages a valve by hitting it while its hit window is open. The window stays open until a hit lands, so a valve can be hit at any point of its cycle, not only while it is open. Each valve has 3 hit points, and by default each open cycle allows one hit, so a valve needs three cycles to die. Each destroyed valve speeds the nest's animation up by a factor of 1.2. When a valve reaches 0 hit points it is destroyed and its living spiders take 1000 damage. When all three valves are destroyed, the nest enters its defeat state. A clean implementation can build the setup, the valve cycle, the hit rule and the defeat from this spec. Spider behavior is not covered here; SPEC-0038 (Behavior 12) gives the spider states the nest uses.

**The encounter is not reachable on this disc (High).** The nest controller is the think routine of behaviour class 3, and each valve is a character of class 4, which yields every tick and does nothing (SPEC-0029, Layout 3). Class 3 occurs only on monster row 12 and class 4 only on rows 13 to 15 (types 10 to 13). No placement record of types 10 to 13 exists in any region or world pack, and no record selects rows 12 to 15 at load, including by a variant that overflows from another type. The only run-time route in code is the variant re-pick of SPEC-0029 (Run-time variant re-pick): it sets a placed character's variant to 0 to 5 and reads first row of its type plus that variant, so a re-picked type 6 character (variant 5, row 12) or type 8 character (variants 2 to 5, rows 12 to 15) could land on a nest or valve row, and type 7 and 9 characters could too, but those two types are placed only in the unreached pack 1259. The re-pick needs a Minion Wizard to hit the target, changes only the row and palette, and only a later default-class request would read the new class. The reachable type 6 and 8 characters are the Juppie monsters of the Juppie stage and the Plague Serpent; no character region of the Juppie pack or of the Plague Serpent pack holds a Minion Wizard record, and the only type 6 and 7 characters outside the Juppie pack lie in the unreached pack 1259 (High), so this is a code possibility, not an encounter. No class request in the boot program passes class 3 or 4 (the only explicit classes requested are 23 to 26, 34 and 35; SPEC-0029). The level-module API inventory has no class-request call: the generic SetAI_Behavior API changes an enemy AI mode and its parameters, not its behavior class (SPEC-0030). No level module names the nest, its valves or its states, and no story row, flag or movie belongs to it. So no pack holds the nest, no level module drives it, and it has no trigger, gate, win or lose rule, flag, movie or exit. Spiders do appear in play, but only as ordinary class-17 common enemies (SPEC-0038). A clean implementation may omit the encounter, or implement it from this spec as unused content; the per-tick rules below describe the code as it would run if a level placed the nest.

## Fight AI build guide

Not part of the shipped game: no level places the nest or a valve (Summary). This guide is for a clean side that chooses to include the encounter; every rule below is the behavior the code would show if a level placed it. Tick numbers are 60 ticks per second. "Request tick" means the tick in which an animation id is asked for; the clip starts in that tick's animation pass. Section references point to the detailed rules below.

**A. Initial values.**
- Nest (the character of monster row 12, record 0 of its world): state 1 (active) from the first tick. It never attacks and has no hit points; it is not hittable.
- Valves (three, found by monster row 13, 14, 15; Layout): state 0, HP 3, spider count 5 / 3 / 2 for valves 0 / 1 / 2, countdown 0, spawn-once flag clear, hit window closed (all zero at start; High, confirmed against the code on 2026-11-08 together with the valve state table, the hit window, the spawn offsets 5.0, 10.0, 10.0 and 0.0, the despawn at clip 58, and the destroyed valve's 1000 damage with state 6 playing clip 0 and then 4 each tick). The spawn-once flag is set before the navigation lookup, so a failed lookup (-1) loses that cycle's spawn.
- Spiders: every placed character whose placement code is 5 (valve 0), 7 (valve 1) or 6 (valve 2) joins that valve's list in level order, is hidden and set to the dead state 16, so it counts as dead. HP when revealed: Spyder 25, Poison Spyder 16, Giant Spyder 150. Positions and facings come from the level.
- Disc tunables: spider counts, valve HP, HP per hit (Layout, 0x2DB0A0 to 0x2DB0B0) and the four launch constants (0x2DB0B8 to 0x2DB0C4). Everything else is a fixed value or a spider animation fact (section 9e).
- "Valve point" means the world position of the valve's root bone (bone 0). The valve forward axis is the third row of that bone's world frame and its up axis the second row (forward: Medium-High; up: High).

**B. Start-up.** Tick 0: bind the whole world, register spiders, hide them (Behavior 1). Wait 180 ticks. The per-tick loop starts at tick 180 (loop tick n = 0).

**C. Per-tick order of the nest (Behavior 2).** (1) copy tunables into the valves; (2) valve sweep 0, 1, 2 (living count, alive or closed); (3) nest update (destroyed count d; d = 3 means defeated, otherwise animation rate 1.2^d); (4) while active, run each valve's state step 0, 1, 2, each followed in the same tick by its hit test (window open), its destruction check (HP exactly 0) and its spider despawn check; (5) debug text (skippable). The sweep always runs before the state step, so a state set by the sweep is stepped in the same tick.

**D. Valve states (Behavior 3; animation ids 0 to 4 have no disc data, so the lengths are rewrite choices, Open question 13).**

| State | Animation id | Duration in ticks | Exit |
| --- | --- | --- | --- |
| 0 initial | 0 | 0 (sweep moves it at once) | closed in the first loop tick |
| 1 closed | none | countdown 600 set by the sweep; the same tick subtracts 1, so 599 remains after the first tick; the tick that finds 0 changes the state without a decrement | 600 ticks after the sweep tick: state 2 |
| 2 opening | 1 | L1 (default 60) ticks; the first tick only requests it | animation 1 finished: open the hit window, state 3 |
| 3 spawn | 2 | one tick to request, then the reveal on the next tick (animation 2 playing and unfinished) | the sweep moves it to 4 the tick after the reveal |
| 4 alive | 0 | until every spider among the first N is dead | sweep: closed (state 1, countdown 600, spawn-once flag cleared) |
| 5 hit reaction | 3 | not reached: the sweep moves the valve out first | none |
| 6 destroyed | 4 (request 0, then 4, each tick) | plays through; nothing waits for it | final |

Real cycle length (default L1 = 60): if the sweep that sets the closed state is in tick u, the countdown reaches 0 in tick u + 599, state 2 in u + 600, animation 1 is requested in u + 601, the window opens and state 3 begins in u + 601 + L1, animation 2 is requested in u + 602 + L1, the spiders are revealed in u + 603 + L1 and the valve is alive from u + 604 + L1.

**E. Hit rules (Behavior 5).** The window opens when animation 1 finishes and closes only on a hit. While open, the hit test runs in every valve state. Hero not attacking: an armed shot within 3.0 plus its radius of the valve point hits for 1 HP (slot stays armed). Hero attacking: the hero's body cylinder against the valve cylinder (radius 2.0, length 4.0 along the root bone's up axis from the bone position), then the swept test, then armed shots within 5.0 plus radius of the root bone; costs the HP-per-hit tunable (default 1). A hit sets state 5 and closes the window, so at most one hit lands per cycle. Destroyed exactly when HP is 0.

**F. Spiders in the fight (Behavior 4, section 9c, section 9e).** Revealed spiders are launched out of the valve straight into the agro machine of SPEC-0038 (Behavior 44), with approach routine 4 (strafe band 4 to 15). They use only two attacks, a melee swing and a charge bite; a Giant Spyder also has a second charge. The real attack ids, clips and tick counts are in section 9e. The summary:

| Attack | Used by | Chooser band (strict) and chance | Cooldown | Hit window after the request | Real total (before the recovery draw) |
| --- | --- | --- | --- | --- | --- |
| Swing (slot 0) | all three | distance below 6 (Spyder, Giant) or 8 (Poison), facing cosine above 0.9239, roll 1 to 100 below 25 | 30 to 60 at the choice and again at the end | ticks 17 to 33 (one hit) | 45 on a whiff, 50 after a hit (the rest of the clip then plays at 1.0) |
| Charge (slot 2) | all three | distance 10 to 35 exclusive, facing cosine above 0.707, roll below 50, a route to the hero | none | Spyder, Giant ticks 20 to 39 and Poison ticks 0 to 7 after the bite clip starts; one hit | run time plus 68 (Spyder, Giant) or 28 (Poison) |
| Charge (slot 1) | Giant only | distance 4 to 10 exclusive, facing cosine above 0.9239, roll below 25 | 30 to 120 | ticks 0 to 7 after the bite clip starts; one hit | 37-tick wind-up, then the run, then 39 |

After each attack the recovery countdown is a whole number from 15 to 30 for a Spyder or Poison Spyder and 30 to 60 for a Giant Spyder, during which no new attack is chosen.

**G. Movement (SPEC-0038 Behavior 48).** All spiders move by root motion: walk and sequencer approach 0.1876 units per tick (11.26 per second); strafe forward 0.1876 for a Spyder and 0.1563 (9.38 per second) for Poison and Giant Spyders; back steps and side steps 0.1876; charge run of slot 2 0.2657 per tick at play rate 1.7; the Giant Spyder's slot 1 lunge runs at play rate 1.0 and moves 0.262, 0.343, 0.392 and 0.392 units per tick in turn, each value held for 4 ticks and the cycle repeating. Turning is the blend of SPEC-0038 Behavior 48. A spider that cannot reach the hero is handled by the agro leash of SPEC-0038 (Behavior 23).

Movement table (movement pass; rows 234, 235, 236; all clips speed byte 2; 60 ticks per second; the Spyder's and the Giant Spyder's size selectors scale only the pose, never the travel, so the Giant moves exactly as fast as the others; shared rules in SPEC-0038 Behaviors 48 and 51 and SPEC-0065, Movement rules for every kind):

| Mode | Source | u/t | u/s | Turn | Clip (id: clip, keys, loop) | Exits |
|---|---|---|---|---|---|---|
| Field walk, patrol, wait in line | root | 0.1876 on every key | 11.26 | steering 0.3 / 0.1 | 2, 4, 11: clip 0, 10, loop | goal reached |
| Strafe approach (inner 7) and sequencer approach (state 0, gait 0 of the strafe set) | root | Spyder 0.1876; Poison and Giant 0.1563 | 11.26; 9.38 | steering | 28: Spyder clip 0; Poison, Giant clip 3 (7 keys) | d below desired (4 + 11f, 6.2 to 12.8) or the 3.0 range |
| Circles (inners 8, 9) | root sideways | 0.1876 | 11.26 | faces the hero; travel opposite to the steering point | 29: clip 18 (+x); 30: clip 16 (-x) | hold count, blocked flag |
| Back off (inner 11) | root backward | 0.1876 | 11.26 | faces the hero | 32: clip 17, 10, loop | d above desired; point 15 from the hero |
| Strafe hold | none | 0 | 0 | aim: Spyder 0.9, Poison 1.0, Giant 0.5 | 31 | hold count |
| Charge run, slot 2 (state 7) | root x strike word 1.7 | 0.2657 | 15.94 | none; solid (charge kind 2) | 44: clip 3, 7, loop | horizontal d at most 3.0 (before the bodies touch at radius + 1.175) or travelled 30 to 40 |
| Giant lunge, slot 1 (state 7) | root | 0.262, 0.343, 0.392, 0.392, each for 4 ticks | 20.85 average | none; solid | 39: clip 28, 4, loop | d at most 3.0 or travelled 5 to 10 |
| Missile dodges 53, 54 | root sideways | 4.85, 5.14 units over the clips' 36 ticks (gate 33) | 8.1, 8.6 | none | clips 20, 19 (9 keys) | gate |
| Hit reaction kind 0 | root backward | 7.92 units over 81 ticks | 5.65 | snapped to the attacker | 57: clip 14, 21 | gate |
| Death | root backward | 7.16 units over the clip | - | none | 58: clip 12 | removal (180, 180, 320 ticks after progress 0.95) |

Corrected (movement pass): the sequencer's approach plays gait 0 of the strafe set (id 28), so a Poison or Giant Spyder closes for its swing at 9.38 u/s, not 11.26. Worked example: a Poison Spyder chooses its swing at d = 5.0 with the hero standing still: state 0 walks 0.1563 per tick (first step the tick after the request) and sees d at most 3.0 after 13 steps (2.03 units), then state 1 next tick; the chase stop (radius 1.0 + 1.175 = 2.175) never acts because the range test stops it first. In a charge the 3.0 test (horizontal distance) is met before the bodies touch, so a hero standing in the line is reached, not shoved; a hero who has moved more than 3.0 off the line is passed by, and the run ends at its 30 to 40 travel limit. Because the range test (3.0) is larger than the body contact distance (2.175; Giant 2.675), the solid body never pushes the hero during a spider's run.

What a port most likely gets wrong (movement): scaling the Giant's speed by its size; one approach speed for the three rows; a charge that steers; side steps that keep a constant distance (they steer to a point 2.0 to the side each tick, so the distance drifts outward).

**H. Hurt side.** Spiders are hit like every common enemy (SPEC-0038 Behavior 37). Defence values: Spyder missile dodge 75, melee dodge 25 / 50 / 75 (quick / normal / heavy); Poison Spyder missile dodge 100, melee dodge 25 / 50 / 75; Giant Spyder missile dodge 100, melee dodge 95 / 50 / 50, defend hold 30 and defence timer 15 to 30. No spider blocks. A hurt spider keeps its valve membership; a dead spider leaves the living count at once.

**I. Phases.** There are no HP phases. Each destroyed valve multiplies the nest animation rate by 1.2. Destruction kills that valve's living spiders and ends its cycle for good.

**J. Hand-offs.** None. No flag, movie, camera or exit belongs to the encounter (Behavior 8). Default if unresolved (rewrite choice): a host level polls the nest state and treats state 2 as the win.

**K. Defeat.** All three valves destroyed: nest state 2, defeat animation 4 each tick if not already playing; the loop never ends.

## Reach and distances (boss reach pass)

The nest and the valves never reach the hero; all reach belongs to the spiders, whose weapon reach per window is in section 9e (Weapon reach, the same posing and capsule method as the common enemies) and whose AI distances are in section 9f; both stand as written (Medium for the edges, High for the bands). What remains is the valve's own reach for the hero (Behavior 5): a shot not during an attack counts within 3.0 plus its radius of the valve point; during an attack the hero's body cylinder (radius 1.0) must overlap the valve cylinder (radius 2.0, from the root bone 4.0 up its axis), or a shot must lie within 5.0 plus its radius of the root bone. Released spiders start on the hero's mesh beside their valve, so a hero who strikes a valve is within the 4 to 15 strafe band of its spiders. Worked example: a Spyder that has walked in to its 3.0 range (about 2.8 to 2.9 after its last step) in front of a still hero starts a swing on tick s and touches him on s + 17 (section 9e); a hero who backs off more than about 1.5 before s + 17 is missed.

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
3. Scan the world's active characters in position order (0 to the active count minus 1). Each valve is attached to the valve record for its index. Each spider (kinds 5, 6, 7) is added to its valve's list in level order, and is hidden and given base state 16 (the plain dead state of SPEC-0038 Behavior 36, which the nest reads as dead; no rise or self-revive exists for spiders). Hidden spiders count as dead until they are revealed (section 4). This is what keeps closed valves closed.
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
   - If d = 3, the nest becomes defeated (state 2) and its animation playback rate is set to 1.0 in the same pass.
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
- When d = 3 the nest goes to defeat and the rate is set back to 1.0 on that same pass (not left at 1.44), so the defeat animation plays at the normal rate (High; the rate store happens after both branches).
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
- Attack damage words of the spider monster rows, by slot number 0, 1 and 2 ("none" = slot unused): Spyder (row 234) 3, none, 7 (slot 2 unblockable), with a blocking angle of 0.966; Poison Spyder (row 235) 5 with a 25 % poison chance, none, 15 with a 50 % poison chance; Giant Spyder (row 236) 5, 20 (unblockable), 15 (unblockable, knockback 2, slow chance 25 %). Poison lasts a whole number of seconds from 600 to 1800 ticks divided by 60 (10 to 30 s); the Giant's slow lasts 60 to 600 ticks divided by 60 (1 to 10 s). The Giant's slots 0 and 1 carry status 0 (stun) with chance 25 and a value drawn from 1 to 10 ticks, which truncates to 0 seconds, so they have no visible effect (Medium). Slot 0 is the swing and slot 2 the charge bite for all three; slot 1 of the Giant Spyder is its second charge (section 9e). The status roll is a draw from 0 to 100 inclusive that must be strictly below the chance (SPEC-0038 Behavior 13).

### 9b. Sounds and effects

- **Nest code.** The only effect the nest controller starts is the shared hit flash, one-shot effect 222 (group 3 of the shared effect resource 9), at the valve point when the valve step runs in state 5, together with animation 3. That step is not reached in normal play (Edge cases), so the flash does not show (High).
- **Spiders.** The spiders' sounds come from their own animation tables, through the world-character sound rule (SPEC-0031): the cue of a sound entry plays at the spider on its creature block's slot when that animation starts, and a keep-alive entry is refreshed while the animation plays. All spiders use creature block 351 (sound group 1). Spyder and Poison Spyder: skitter cues 8 and 7, ambush 3, go aggressive 1, roll loop 4 (keep-alive), attack 0, take damage 5, dodge 2, die 6, poison bite 9. Giant Spyder: the large-spider cues 10 to 18, with the roll loop 14 (keep-alive). The step does not run while a character's base state is 42 (High).
- **Valves and nest.** No world record holds a valve or the nest (types 10 to 13 have no placement record and no character region), so they have no animation tables and no sound words on the disc. Their only sound or effect would be the hit flash above (High).

### 9c. Spiders of the fight (High unless marked)

- Per cycle, with the defaults: valve 0 releases 5 normal Spyders, valve 1 releases 3 Poison Spyders, valve 2 releases 2 Giant Spyders, 10 in all. Which monster row a spider uses comes from its own placement; the nest only checks the behaviour code (Layout).
- Respawn: the same characters come back. A valve releases again only when every spider among its first N entries is dead (or hidden), and then after 600 ticks plus the opening animation. Each release restores full HP, so no spider stays dead while its valve stands.
- When a valve is destroyed, its spiders take 1000 damage on every tick, so its group is gone for good. The other valves keep cycling.
- Spiders are released straight into AI mode 19 (agro) on the hero's ground mesh, launched out of the valve (Behavior 4). From there they run the class-17 agro machine of SPEC-0038 with the rows' values: all three spider rows use approach routine 4 (strafe). HP: Spyder 25, Poison Spyder 16, Giant Spyder 150. Damage as in Behavior 9a; attack clips, windows and real tick counts in Behavior 9e.
- Death of a spider: the shared enemy death (state 16, death animation 58, then despawn flag), after which the valve counts it dead.

### 9d. Intended play sequence (as the code would run; not reachable on this disc)

1. The hero enters the nest's world. The nest binds every character of the world and hides all spiders; nothing moves for 3 s.
2. Each valve finds no living spider, closes, and waits 10 s. The three valves cycle in step, because they share the same timings.
3. Each valve plays its opening animation; when it ends, the valve becomes hittable and its spiders burst out toward the hero and attack at once.
4. The player can strike a valve now, or while fighting, or later: one hit per valve per cycle counts (a melee hit, or a shot while not attacking, Behavior 5). The hit valve loses 1 HP.
5. To get the next hit on a valve, the player kills all of that valve's spiders; 10 s later the valve opens again and releases them with full HP.
6. After three counted hits a valve is destroyed and its spiders die. Each destroyed valve makes the nest pulse 1.2 times faster.
7. When the third valve is destroyed the nest plays its defeat animation. Nothing else happens; the player leaves the world by its ordinary exits.

### 9e. Spider animation ids, clips and real durations (High for the table facts; Medium-High for the tick offsets, which can move by one tick with the order of the behavior and animation steps)

All three spider kinds share one 34-clip family on disc, and each kind has its own 166-entry action table (ids 0 to 165). Every clip below has speed byte 2 unless noted, so at play rate R it advances R x 0.25 keys per tick (4 ticks per key at R = 1.0; SPEC-0005 Playback 1a). A one-shot clip of F keys reaches its last key after 4(F - 1) ticks at R = 1.0 and the next tick's progress gate (0.99) passes (from F = 25 keys the gate passes one tick earlier, SPEC-0005); the phase waits of SPEC-0038 Behavior 13 use that gate. An id with no clip in the table is skipped in the same tick with no wait.

**Attack ids per slot (k = 0, 1, 2; id = 33 + 5k + phase, phases A wind-up, B strike, C follow-through, D blocked, E recovery).**

| Slot | Used by | A | B | C | D | E |
| --- | --- | --- | --- | --- | --- | --- |
| 0 (swing) | all three | 33 missing | 34: clip 11, 14 keys, one-shot, weapon-3 window on keys 5 to 9 | 35 missing | 36: clip 8, 22 keys, one-shot | 37 missing |
| 1 (charge 2) | Giant only | 38: clip 25, 10 keys, one-shot | 39: clip 28, 4 keys, looping, weapon-3 window on all keys | 40: clip 27, 4 keys, one-shot, window on keys 0 and 1 | 41: clip 8 | 42: clip 26, 7 keys, one-shot |
| 2 (charge 2) | all three | 43 missing | 44: clip 3, 7 keys, looping (the run) | 45: Spyder and Giant clip 11 (window keys 5 to 9); Poison clip 27 (window keys 0 and 1) | 46: clip 8 | 47: clip 9, 4 keys, one-shot |

Slot 3 (ids 48 to 52) is unused by all three rows and has no clips. Slot 1 of the Spyder and Poison Spyder is unused. The weapon-3 volume is a finite cylinder centred on the third attachment frame, radius 3.0 and length 3.0 for the Spyder and Poison Spyder, radius 4.0 and length 4.0 for the Giant Spyder (SPEC-0029 Behavior 10.2). A window that is open on every key of a clip stays open for the whole clip, so one swing or bite gives one hit.

**Play rates.** Row speed words, wind-up / strike / recovery: slot 0 is 1.0 / 1.2 / 1.0; slot 2 is 2.0 / 1.7 / 1.2; the Giant's slot 1 is 1.0 / 1.0 / 1.0. The strike word applies in sequencer state 2 (swing) and in states 6 (from the tick the run clip starts) and 7 (the run); states 3, 4, 8, 9 and 12 run at 1.0. The swing's strike clip is requested in the last tick of state 1, whose rate is the wind-up word 1.0, so its first animation pass advances 0.25 keys and every later pass 0.3 keys (Medium).

**Real durations (ticks from the request tick of the clip named; High for the arithmetic).**
- Swing (slot 0): the wind-up is skipped, so the strike clip 34 is requested in the first sequencer tick after the approach (the approach walks with gait 0 of the strafe set, id 28, while the distance is above 3.0; section G). The strike test sees the window in ticks 17 to 33 after the request; the hit lands on the first of those ticks on which the weapon touches the hero (Weapon reach below: tick 17 for a hero 2.2 to 2.8 units ahead, tick 18 from 2.9, tick 19 at 3.5). The clip's gate passes in tick 44 on a whiff and the end path runs in tick 45; after a hit the rest of the clip plays at 1.0 (state 3 stores no speed word), so the gate passes in tick 49 and the end path runs in tick 50 (with no follow-through the sequencer goes to recovery, which is missing). A blocked swing plays clip 8 (id 36), whose gate passes 85 ticks after its request, then the end path runs the next tick. The end path draws the recovery countdown and the slot cooldown (30 to 60) again.
- Charge (slot 2): wind-up id 43 is missing, so the run clip 44 is requested in the first sequencer tick and the state becomes 7 at once. The spider runs 0.2657 units per tick (first step the tick after the request) until the hero is within the melee range 3.0 (state 8) or it has run 30 to 40 units (the charge distance range of this slot, drawn at the choice); from 12.0 units away it needs 34 steps. State 8 requests clip 45. Poison Spyder: the window (keys 0 and 1) is seen by the strike test in ticks 1 to 7 after the request (the request tick itself still sees the run clip), gate in tick 13, the recovery clip 47 is requested in tick 14 and its gate passes in tick 27, so the end path runs in tick 27 after the bite starts. Spyder and Giant Spyder: window passes in ticks 20 to 39, gate in tick 53, clip 47 requested in tick 54, end path in tick 67.
- Charge (Giant slot 1): clip 38 wind-up gate in tick 37, then the lunge clip 39 is requested (rate 1.0); the lunge ends when the hero is within 3.0 or after 5 to 10 units; state 8 requests clip 40 (window seen in ticks 1 to 7, gate in tick 13); clip 42 is requested in tick 14 and its gate passes in tick 25 after that, so the end path runs in tick 39 after the bite starts.
- After any end path the outer sub-state is 0 and the normal choice cannot run again until the recovery countdown (15 to 30, or 30 to 60 for the Giant Spyder) has run out; the choice also needs the hero attackable (SPEC-0038 Behavior 7).
- Death: clip 12 (id 58), 35 keys, one-shot. It starts in the tick the spider is set to the dead state; its finished flag is first seen 137 ticks after the request, and the valve step of that tick sets the despawn flag. The valve counts the spider dead from the tick it enters state 16, not from the despawn flag.
- Other present ids: walk 2 (clip 0, 10 keys, looping); rush 21 (clip 3); strafe forward 28 (Spyder clip 0; Poison and Giant clip 3); back step 32 (clip 17); side steps 29 and 30 (clips 18 and 16); strafe hold 31 (Spyder clip 1, 8 keys; Poison and Giant clip 21, 20 keys); melee dodge 55 (clip 24, 12 keys); missile dodges 53 and 54 (clips 20 and 19, 9 keys each); hit reaction kind 0 id 57 (clip 14, 21 keys); confused 69 (clip 30, 55 keys, looping). Absent: ids 85, 97, 116 to 119, 132, 142, 153, 155 and 156. Spiders therefore never use the rise (85) or the flyer ids, and the fidget picker has only id 115 (clip 13, 60 keys, looping) to choose.

**Weapon reach (Medium).** The weapon-versus-body test uses the asymmetric collision rule in SPEC-0018, including its facing-card case and near-parallel case. All attacks use weapon 3 on the third attachment frame, tested against Tor's body capsules (SPEC-0038 Behavior 13). Distances are along the spider's facing from its centre to Tor's origin, with Tor idle, facing the spider and level with it ("at the tick" = measured from where the spider is on that tick; "from the start" = from where it stood when the clip was requested, so it includes the clip's root travel).

| Kind | Attack (clip) | Window (ticks after the request) | Tor touched, at the tick | Tor touched, from the start | Usual hit tick |
| --- | --- | --- | --- | --- | --- |
| Spyder, Poison Spyder | Swing, slot 0 (clip 11) | 17 to 33 | -2.8 to 2.8 | -2.7 to 4.2 | 17 (hero 2.2 to 2.8 ahead); 18 from 2.9 up to 3.0 (the walk-in stops one step past the 3.0 range test, at about 2.8 to 2.9, so which of the two applies depends on where the last step lands; the chase stop 2.175 matters only for a hero who walks closer) |
| Spyder | Charge bite, slot 2 (clip 11) | 20 to 39 | -2.8 to 2.8 | -2.6 to 4.3 | 20 (hero within 2.5); 21 at 3.0 |
| Poison Spyder | Charge bite, slot 2 (clip 27) | 1 to 7 | -3.2 to 3.6 | -3.2 to 3.6 | 1 |
| Giant Spyder | Swing, slot 0 (clip 11) | 17 to 33 | -3.8 to 4.0 | -3.1 to 5.2 | 17 (hero 2.7 to 3.0 ahead) |
| Giant Spyder | Lunge bite, slot 1 (clip 27) | 1 to 7 | -2.8 to 4.6 | -2.8 to 4.6 | 1 |
| Giant Spyder | Charge bite, slot 2 (clip 11) | 20 to 39 | -3.8 to 4.0 | -3.0 to 5.3 | 20 |

Negative values mean the weapon also touches a hero standing behind the spider's centre (the weapon sweeps across the body). The swing is started at 3.0 units or less (the melee range) and the bites begin when the hero is within 3.0, so a still level hero is inside these sampled bands; first-touch timing can differ if the hero moves. The earlier feet-height bands came from the old round-cylinder approximation and are withdrawn; the exact pass establishes level-hero samples, not a vertical range.

**Spiders and the attack manager.** The spiders take part in the shared attack manager of SPEC-0038 Behavior 45 (budget weights 5 for a Spyder, 8 for a Poison Spyder, 12 for a Giant Spyder; capacity 3.0 x (weapon grade + armour grade) from the hero's gear, 0 with starting gear). With ten spiders in agro and starting gear only the first in the manager's order may attack and the rest wait in line (outer sub-state 12); the order is nearest first, with the 5.0-unit swap rule. A better-equipped hero is attacked by more spiders at once.

### 9f. Distances of the three spider rows, other than the weapon reach (High for the row values; Medium-High for the derived figures)

The spiders (rows 234 Spyder, 235 Poison Spyder, 236 Giant Spyder) run the common agro machine with approach 4 (strafe), so every distance below comes from the row and the rules of SPEC-0038 (Behaviors 3, 7, 23, 44 and 45). The weapon reach of the swings and bites is listed with the spider fight above and is not repeated here. d is the distance from the spider to the selected hero; "x0.7 standing" is the notice factor F1 of a hero standing still and "x1.75 running" that of a running hero (SPEC-0038 Behavior 18). Shared by all three: the sight line is tested (no always-sees flag), so through a wall the band is multiplied by F2 (0.5 in most hero states, 0 for a hero standing still); the view cone only halves the notice chance outside it, it is not a hard edge; a hold or circle sub-state faces the hero every tick with the row's ground turn weight; the leash fires after 181 ticks with no route to the hero while the spider's walk waypoint is 15.0 or more from the hero; the lost-sight search goes to the last-seen point moved 4.0 further along the spider-to-hero line and gives up after 121 ticks there; streaming releases a spider beyond 280. The distance-hide flag is set on all three rows, but the gate runs only for rush enemies, so it never fires (it only makes a waiting spider hide in place). The circle sub-states steer to a point 2.0 to the side every tick, so a circling spider keeps roughly its current d and drifts slowly outward until d passes 15 and it walks in again. The confused strike is a normal attack choice allowed at d at most radius + 1.175 + 0.25.

| Distance (units) | Spyder (234) | Poison Spyder (235) | Giant Spyder (236) |
|---|---|---|---|
| Notice band (x0.7 standing; x1.75 running) | 5 to 30 (3.5 to 21; 8.75 to 52.5) | same | same |
| Sight cone half-angle | 45 deg | same | same |
| Strafe circle band; walk-in target (drawn); back-off point | 4 to 15; 6.2 to 12.8; 15 from the hero | same | same |
| Slot 0 choice band, arc, close-in | 0 < d < 6, within 22.5 deg, closes to 3 | 0 < d < 8, within 22.5 deg, closes to 3 | 0 < d < 6, within 22.5 deg, closes to 3 |
| Slot 1 choice band, arc, run | none | none | 4 < d < 10, within 22.5 deg, charge run 5 to 10 (ends early at d <= 3) |
| Slot 2 choice band, arc, run | 10 < d < 35, within 45 deg, charge run 30 to 40 (ends early at d <= 3) | same | same |
| Wait-in-line stand-off square (first draw; later 50) | 5 | 5 | 15 |
| Chase stop (radius + 1.175); confused strike | 2.17; 2.42 | 2.17; 2.42 | 2.67; 2.92 |
| Revive from state 16 | by the valve only, with no spacing test and no rise animation (the 5.0 clear-area test and animation 85 belong to the reanimating skeleton rows, not to spiders; SPEC-0038 Behaviors 12 and 35) | same | same |

What a port most likely gets wrong here: giving the spiders a leash radius (there is none while a route exists); making the view cone a hard edge; letting the hide gate pull a spider home; circling at a fixed radius instead of the 2.0 sidestep that keeps the current distance.

### 10. Timings

| Item | Value |
| --- | --- |
| Setup to first sweep | 180 ticks (3 s) |
| Closed countdown | 600 ticks (10 s), plus one tick before the valve opens |
| Opening animation (1) | length of the animation |
| Spawn (animation 2) | spiders revealed on the first tick the animation is playing and unfinished |
| Destruction | same tick as the lethal hit |
| Destruction animation (4) | no length on the disc (no valve animation data); no rule waits for it to finish |

**Timing table (timing pass).** Ticks are 1/60 s; every spider clip has speed byte 2 (4 ticks per key at rate 1.0; slot rates as in Play rates above). The nest's own timers (setup 180, closed countdown 600) are whole ticks lowered once per nest tick.

| Item | Ticks |
| --- | --- |
| Setup to the first valve sweep | 180 |
| Valve closed countdown | 600, then the opening animation; spiders appear on the first tick the spawn animation plays unfinished |
| Spider swing (slot 0) | B requested on s; window seen s + 17 to s + 33; hit on the first touching tick (s + 17 for a hero 2.2 to 2.8 ahead, s + 18 from 2.9; the walk-in ends at about 2.8 to 2.9, so either can occur); gate s + 49 and end path s + 50 after a hit (s + 44 and s + 45 on a whiff) |
| Spider charge (slot 2) | run from the tick after the request at 0.2657 per tick until within 3.0 (34 steps from 12 units) or 30 to 40 units run; bite C requested on c8; Spyder and Giant window c8 + 20 to c8 + 39, hit c8 + 20 (c8 + 21 at 3.0), end path c8 + 67; Poison Spyder window c8 + 1 to c8 + 7, hit c8 + 1, end path c8 + 27 |
| Giant lunge (slot 1) | wind-up gate 37; bite window c8 + 1 to c8 + 7, hit c8 + 1; end path c8 + 39 |
| Recovery after any attack | int[15, 30] ticks (Giant int[30, 60]); slot cooldowns drawn at the choice and at the end path |
| Spider hit reaction (id 57, 21 keys) | 81 ticks from the last hit (restarted on every hit) |
| Spider death | clip 12 (35 keys) from the death tick; finished flag seen 137 ticks after the request; the valve counts the spider dead from the death tick |
| Valve respawn | when all of a valve's spiders are dead or hidden: 600 ticks plus the opening animation |

Worked exchange (one Spyder against Tor with a weapon, Tor idle 2.5 units in front, swing chosen on c): s = c + 1 (no approach, no wind-up); s + 17: the weapon touches, 3 damage before armour; Tor's flinch (36 ticks) starts on s + 17 and his done test passes on s + 54, so the spider (and every other common enemy) can choose nothing on s + 18 to s + 54 and its cooldowns stop; s + 49 gate (rate 1.0 after the hit); s + 50 end path, recovery int[15, 30]; from s + 55 the spider may choose again once its recovery has run out. A Tor swing pressed on h is seen by the spider's melee defence on h + 1.

What a port most likely gets wrong (timing): the bite of the Poison Spyder opens on the tick after the request, not on it; the swing's first touch depends on the distance (17 or 18); spider attack cooldowns stop while the hero is hurt; the death clip does not delay the valve count.

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
- Two destroyed valves: animation rate 1.44. Three destroyed valves: nest defeated, animation rate 1.0.
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

### Timeline vectors (defaults; valve animation 1 length L1 = 60 ticks as the rewrite choice of Open question 13)

**Nest timeline, hero standing still (ticks since the encounter starts; no random draws).**
- Tick 0: setup; all ten spiders hidden in state 16; the valves are in state 0. Ticks 1 to 179: nothing changes.
- Tick 180 (loop tick 0): sweep sets all three valves closed with countdown 600; the same tick's state step leaves 599.
- Tick 779: countdown reaches 0. Tick 780: state 2. Tick 781: animation 1 requested.
- Tick 841: animation 1 finished: the hit window opens on all three valves, state 3. Tick 842: animation 2 requested.
- Tick 843: spiders revealed (5 Spyders, 3 Poison Spyders, 2 Giant Spyders), each at the valve point plus 5.0 along the valve's forward axis, base state 1, agro, full HP, velocity gains (10 forward, 10 up). Tick 844: the sweep moves each valve to state 4.
- If the hero kills the last spider of a valve in tick K (the spider is in state 16 from tick K): tick K + 1 the sweep sets closed and countdown 600; the window (if closed by a hit) reopens in tick K + 662; the spiders are revealed again in tick K + 664 with full HP. A hit at any tick from 841 on (or from K + 662 in later cycles) takes 1 HP and closes the window; three hits on one valve need three windows. Fastest possible run, with every valve's spiders killed in the tick after each reveal: windows at ticks 841, 1506 (K = 844) and 2171 (K = 1509), so the first valve cannot be destroyed before tick 2171, and a real fight is longer because the ten spiders must be killed first.

**One Poison Spyder against a hero standing still 12.0 units away, flat open ground, hero attackable, no other spider in the manager list (reveal tick r = 843; random draws marked).**
- r: first agro tick, outer sub-state 0. r + 1: outer 4 (strafe), inner 10. From r + 1 on, each tick the charge (slot 2) is chosen with chance 50 % while the distance is between 10 and 35 and the spider faces the hero within 45 degrees (draw: geometric, tick r + 1 with probability 0.5). Call the choice tick s.
- s: bite-run clip requested; steps of 0.2657 units from s + 1; distance 3.0 reached after 34 steps; s + 35: state 8, bite clip 45 requested; weapon-3 window passes s + 35 to s + 42; first hit possible in s + 36 for 15 damage (poison 50 % for 10 to 30 s, draw 0 to 100 strictly below 50); s + 48: bite clip gate; s + 49: recovery clip 47 requested; s + 62: end path, recovery countdown N = a whole number from 15 to 30 (draw), slot 2 cooldown 0.
- The spider is now about 3.0 units from the hero. If the spider backs off (distance below 4 gives the back-off inner state, SPEC-0038 Behavior 44) it stops at a desired distance of 4 + 11 x f with f from 0.20 to 0.80, that is 6.2 to 12.8 units, and may charge again when that distance is above 10 (min cycle s + 62 + 15 = s + 77 plus the roll wait).
- Swing (slot 0) from within 8.0 units, facing within 22.5 degrees, 25 % per tick: strike clip requested in tick w (after walking in to 3.0 if farther); window passes w + 17 to w + 33; first hit w + 18 for 5 damage (poison 25 % for 10 to 30 s); gate w + 44; end path w + 45; slot 0 cooldown drawn from 30 to 60 at w and again at w + 45; recovery N from 15 to 30. Next swing starts no earlier than w + 45 + max(N, cooldown) and typically w + 80 to w + 115 including the roll wait. In 600 ticks a lone Poison Spyder in reach lands about 5 to 7 attacks, at most one hit each.
- Death of that spider at tick K: clip 58 starts in tick K, finished flag seen in tick K + 137, despawn flag set in that tick's valve step.

**Spyder vs Giant Spyder differences in the same setting.** Spyder (25 HP): charge bite clip 45 is the 14-key clip: window passes s + 55 to s + 74 (first hit one tick later), hit 7 unblockable, gate s + 88, end path s + 102. Giant Spyder (150 HP): the same slot-2 timing with a larger weapon cylinder (radius 4.0, length 4.0), damage 15, unblockable, knockback 2; its slot-1 lunge (distance 4 to 10, 25 % per tick) has the 37-tick wind-up in ticks s to s + 37.

## Open questions

**Default if unresolved (rewrite choice):** (a) valve animation lengths: animation 1 (opening) 60 ticks, animation 2 (spawn) 60 ticks, animation 4 (destruction) 90 ticks, none of which exist on disc; keep the opening length the only one that matters (it times the window and the reveal). (b) Hidden spiders stay registered and invisible until revealed and are never freed by streaming or corpse removal (their despawn flag stops their update); if a rewrite frees dead characters, keep one spawn pool per valve instead. (c) Valve destruction kills spiders through the normal death path of SPEC-0038 (loot and coins as for a hero kill; the death hook takes no killer, so this is the rule and not a choice, Open question 15). (d) The nest has no exit: the host level script polls nest state 2.

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
13. Lengths of the valve and nest animations 0 to 4. Unknowable: the encounter has no valve or nest model, animation or sound on the disc, so the lengths are developer-side values that are not encoded anywhere (proactive (open close-out)). Default (rewrite choice): item (a) above. Only the length of animation 1 changes the fight (it delays the window and the reveal by that many ticks, Behavior D of the guide).
14. What frees or hides a spider while it waits for its valve. Partly resolved (proactive (open close-out); High for the rules, runtime-only for the start value): hidden spiders are set to the dead state; the dead-state step counts the corpse countdown down only after the death clip passes 0.95 of its length, and the streaming pass of a slot with the keep flag off releases a dead character whose countdown is below 0. The nest sets the despawn flag when the death clip has finished, which stops that character's update. The character constructor does not write the countdown, so its value at setup is whatever the object slot held (0 in a slot never used, the left-over value of an earlier occupant otherwise), and the rules above then decide whether a hidden spider is released before its valve reveals it (needs a runtime check). Default if unresolved (rewrite choice): item (b) above.
15. Kill credit and loot for spiders killed by a destroyed valve. Resolved (High; proactive (open close-out)): the valve's 1000 damage goes through the same shared damage routine that every other source uses: it subtracts 1000 from the spider's HP, and when the HP is not above 1000 it sets HP to 0 and runs the spider's death hook with both flags set to 1. The hook takes no killer identity, so the drop, the placement payout, the carried-Neopoint coins and the death countdown follow SPEC-0038 Behavior 34 exactly as for a hero kill. A spider already in state 16 is skipped. Item (c) above is therefore the rule, not a default.
16. Whether the nest, valves and spiders update in the order nest, valve, spider within one tick, and whether "animation 2 playing" is true on the request tick or the next. Resolved (Medium-High; proactive (open close-out)): each active character runs its update, its motion and its own animation pass in turn, in list order, and the nest is record 0 with the valves and spiders bound after it (SPEC-0038 Behavior 50). A request the nest makes on tick t therefore reaches the valve's animation pass on tick t, and the playing test passes on tick t + 1, so the reveal is the tick after the request, as the timeline vectors assume. Only a different bind order of the valve records would shift every later number by one tick.
17. Resolved (High; proactive (open close-out)); corrections for other specs. SPEC-0038 Behavior 12 now agrees: a spider's state 16 is the base state 16 (the nest and the death hook both write it through the base-state setter); the spider rows 233 to 236 have no reanimation flag, so a killed or hidden spider uses the plain dead state of SPEC-0038 Behavior 36, no spider table has animation 85, and the only way out of state 16 is the valve's revive (base state 1, class state 19). The countdown, animation 85 and 5.0 spacing test are the class state 16 of the reanimating rows 224, 226 and 227 only. Still to be corrected in SPEC-0038 (not by this spec): the Poison Spyder worked example numbers the slots from 1 (its slots 1 and 3 are slots 0 and 2 here) and plays the charge wind-up, but the charge wind-up id 43 has no clip, so the charge goes straight to the run clip.
