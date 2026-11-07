# SPEC-0057: Environmental hazards and damage zones

- Answers: proactive (open close-out) (environmental damage and hazard rules; extends SPEC-0036, SPEC-0044, SPEC-0055); proactive (open question sweep, world details)
- Status: Draft
- Disc build: SCUS-97367 v1.00 (same fingerprint as SPEC-0001)
- Confidence: per section. Confirmed on disc: the damage values (2, 3 and 10), the waits (0.5 s, 1.0 s, 0.3 s and 0.02 s), the 1.5 s cooldown, the poison and fire defaults, the status-apply defaults, and the object-to-slot map. No engine hazard volume type: High. Ground rules of the hero motion tick (deadly ground, hurting surface, deep overlay kill): High for the rules and the disc counts, Medium for the meaning of the surfaces (the deep overlay is probably water). Host-call counts per module: High for the reached counts and the import-list totals (see Behavior 1). Trigger callback rate (one callback thread per trigger record): High. Animated hazards in module 1091 (helper, mode map, slot flags, pool): High; High for the kill-branch outcome (the death clip plays at its normal rate; Behavior 2, Kill outcome). Script trigger routine and sweep in module 1091: High for the calls, the id ranges, the wait for the hero to leave and the one run per visit. Zone with cooldown (module 1174): High for the code, the clock unit (seconds), the object-word gate and the stamp reset. Zone helper (module 864): High for the calls, the constants, the caller, the zone lists and the 3-D distance; the exact tick of each hit is stated to within one tick. Status apply (one method per class, called by every source on the target; SPEC-0036): High. Per-tick poison: High for the period, the amount and the 1 HP floor; the hero's poison words are always -1 (High). No per-tick fire or confusion effect on a hero: High (SPEC-0036). No fall damage: High. Hits on a dead hero: High (SPEC-0055). Plague serpent cloud: High. Test vectors: synthetic.

## Summary

The engine has no hazard volume type. The hero takes environmental damage in four ways:

1. Level scripts call the damage, status, kill and fall host operations when a trigger or a zone fires. These are the level-script hazards of SPEC-0044 (modules 1091, 864 and 1174 in SPEC-0044, 1090 in SPEC-0041, 1089 and 1381 in SPEC-0042; Behavior 5A lists the last three).
2. The per-tick poison status deals periodic damage through the normal take-hit path, limited so that the hero keeps at least 1 HP (SPEC-0036). Fire has no per-tick effect on a hero: a fire status on a hero shows the fire effect and counts down, but deals no damage. Fire damage over time exists only in the enemy per-tick routine (SPEC-0029).
3. Enemy attacks and projectiles apply statuses on a hit (SPEC-0036; the plague serpent's poison cloud in SPEC-0016). They call the hero's own status-apply method, so the hero's immunities block them.
4. The hero's motion tick has three ground rules (Behavior 7A): deadly ground (SPEC-0055), a hurting surface that deals 10 HP once every 2.5 s of game clock, and a deep overlay surface that kills both heroes.

The clean side can build the script hazards by making triggers that call the same host operations with the values below, and the ground hazards from the ground entry fields. It does not need a hazard volume type.

## Where it lives on the disc

- The hazard triggers, zone lists and cooldown records are in the level-script modules described in SPEC-0044. Each module is the script chunk of the world pack with the same number (SPEC-0030). Their tables are in module data and world records and are not copied here.
- Hazard host calls are the host API of SPEC-0030: DamageHero, SetHeroStatus, KillCurrentHero, StartHeroLongFall, PlayAnim, MoveHero, Wait and GetSystemTime.
- The hero's status timers and the poison and fire words are on the hero object (SPEC-0036).
- The ground rules read the ground model-set entry under the hero (SPEC-0018, SPEC-0055): its kind word, its surface word and its linked overlay entry (Layout). The overlay height limit 2.3 is the data word 0x2DD43C. The hurting-surface values (10 HP, 2.5 s, the 0.2 height limit) are code constants.
- The game clock read by GetSystemTime is in seconds; it is the same clock as the hurting-surface rule.

## Layout

No new disc data. The runtime structures a hazard uses are:

- Hazard flags (module 1091): one flag word per hazard slot. Slots 1 to 8 are used; slots 0 and 9 are not used by any object. All flags start at 0 in the disc image and the flags of objects 4 to 10 stay 0 in play (Behavior 2). Flag 8 is also a one-time guard: a one-time routine sets it to 1 the first time it runs. Object 3 ignores its flag, so the guard has no hazard effect.
- Pending hazard records (module 1091): a pool of ten records (index 0 to 9), shared by the animated hazards and the script sweep routine. Each hazard run takes one and releases it at the end. Record 0 is tried first, then records 1 to 9. When all ten are busy the run gets index 10 and nothing is written (Edge cases).
- Zone list (module 864): a count (at record +0x28) and up to 10 animated-object ids (from record +0x00). The zone test uses locator 0 of each object. Three lists are used: zone A, zone B and zone C (Behavior 5).
- Zone cooldown stamp (module 1174): one word shared by every zone that uses this callback, initially 0.0. It holds the game clock value, in seconds, at the last zone hit.
- Status timers and extra words (SPEC-0036): fourteen timers in ticks; the poison amount and period words read by the per-tick poison effect; and the fire amount and period words, which no hero effect reads. All four hero words are -1 from the hero's creation and stay -1, because every source passes -1 for a hero.

Ground model-set entry fields used by the ground rules (the rest of the entry is in SPEC-0018; the deadly-ground fields are in SPEC-0055):

| Offset | Size | Type | Meaning |
| --- | --- | --- | --- |
| 0x0E | 2 | s16 | Linked overlay entry (another ground entry of the same pack), or none. 365 links on the disc, almost all to overlays with surface word 8. |
| 0x38 | 4 | s32 | Ground kind. 2 = deadly ground. |
| 0x40 | 2 | u16 | Surface word. 2 on a non-deadly entry = hurting surface. Surface 2 is a hot (burning) surface and surface 8 is water: on deadly ground the death clip and effect follow the surface word, a hot burst for 2 and a water splash for 8 (SPEC-0055, section 8A; Medium-High). On the disc: 23 hurting-surface entries (19 of kind 0, 4 of kind 3) in packs 696, 702, 714, 753, 864, 879 and 1080 (13 of them in 1080). Their zero-based ground-entry ordinals are 696: 121–123; 702: 62; 714: 95; 753: 17, 135; 864: 45, 46; 879: 4; 1080: 25, 42–44, 46, 70, 72, 77, 93, 94, 113, 117, 119. These are entry-list positions, not named sub-areas. |

## Behavior

### 1. Host-call inventory

Reached counts: call sites reachable from each module's entry points, deduplicated by module and call site. They are counts only.

| Host call | Reached sites | Reached modules | Hazard use |
| --- | --- | --- | --- |
| DamageHero | 8 | 6 | 1091: 2 (animated-hazard helper and sweep). 864: 1 (zone helper). 1174: 1 (zone with cooldown). 1089: 1 (hazard thread, 1 HP). 1090: 2 (hurt-zone routines for the two banks, 20 HP). 1381: 1 (falling-hazard routine, 5 HP; Behavior 5A). |
| KillCurrentHero | 3 | 3 | 1091: 1 (animated-hazard helper, kill branch). 735: 1 (hero-death callback; SPEC-0055). 1089: 1 (timeout kill of the crusher sequence; Behavior 5A). |
| SetHeroStatus | 1 | 1 | 1091: 1 (sweep; poison, status index 2, default duration). |
| StartHeroLongFall | 14 | 12 | Falls, not damage. No fall deals damage (Behavior 6). |
| SetHitPointsPercentage | 5 | 3 | Not a hazard call (boss and scripted health; SPEC-0036). |
| RegisterScriptAnimatedTrigger | 13 | 3 | 1091: 8. 956: 1. 1089: 4 animated triggers to a shared handler that dispatches to its button/animation routine. |
| RegisterScriptTrigger | 580 | 84 | All trigger kinds. Hazard callbacks among them: 1091, 1174, 1098, 735 and others not classified. |

Import-list totals. The module import lists also hold call sites that are not reached from any entry point. Totals over all listed sites:
- DamageHero: 11 sites in 6 modules. Three sites are dead: two in module 864 (no routine that is registered, started or called reaches them) and one in module 1089 (a short body under the hazard word that nothing references). The other 8 are reached, 4 of them only behind tests of module data words that start at 0 and are set later in play (1089, 1090 and 1381; Behavior 5A). Counting only the call sites that a plain walk from the entry points lists would miss those four.
- KillCurrentHero: 3 sites in 3 modules, all reached. The 1089 site is the timeout kill, behind the same kind of test.
- SetHeroStatus: 1 site, the only one listed.
- StartHeroLongFall: 14 sites in 12 modules.
- SetHitPointsPercentage: 7 sites. The 2 unreached sites are in module 1350.
- RegisterScriptAnimatedTrigger: 21 sites in 5 modules.
- RegisterScriptTrigger: 704 sites in 95 modules.

Caveats. Module 1089 registers four animated triggers from its initializer; their handler reaches a button/animation routine. One of its two DamageHero(1) calls is in a control thread that the initializer starts and the other is in the dead body named above; its KillCurrentHero call is in the per-tick update. Both live calls are gated by module words that are 0 at load, so a listing that decides those branches from the load-time value drops them; they run in play (Behavior 5A). No level module applies fire, slow, curse, confusion, strength or speed to the hero. The only status that level scripts apply is poison.

Not hazards, although they hurt or affect the hero:
- Enemy and boss contact damage and attacks (SPEC-0016, SPEC-0029, SPEC-0038).
- The plague serpent's poison cloud: an enemy projectile that lasts 300 ticks, descends 0.1 units per tick while more than 2.0 units above the ground, and applies poison for 15 s to the current hero within 6.0 units horizontally (SPEC-0016). Once it is at its hover height it drifts toward the hero in the horizontal plane by 0.1 per tick. It calls the hero's status-apply method with poison, its slot value as the duration in seconds, and -1 for both words, so the hero takes the default 2 HP every 4 s. It is skipped while the hero is already poisoned, and status 9 (immunity to poison) blocks it (effect 0xC3 plays). High.
- The actor trigger volumes of SPEC-0028. They push the actor and spawn an effect; they do not damage.
- The Lava Mote and the Fire Mote. These are hero items (SPEC-0021).
- Spike and lava volumes: none exists. The terrain hazards are the ground rules of the hero motion tick (Behavior 7A): deadly ground, the hurting surface and the deep overlay kill (probably deep water; Medium). Deaths on them are covered by SPEC-0055.

### 1A. Trigger callback rate

Confidence: High. This sets how often every hazard callback below runs.

- Every trigger record (each script trigger, and each entry of an animated object's trigger list) has one callback thread handle. The fire step runs on every tick of overlap and always updates the record (actor, frame stamp, object word, state word). It starts the callback as a thread only when the record's handle is empty.
- In step 2 of each tick's script step (SPEC-0030, section 3) every live trigger thread is resumed. When it has ended (returned), it is freed and its handle is emptied in that same step. A fire on a later tick then starts a new run.
- So a callback never runs twice at once for one trigger. A fire while the callback is still running only updates the record.
- A callback with no wait runs once per tick of overlap.
- A callback that waits w ticks and then returns runs at most once every w + 1 ticks while the hero stays inside: a fire on tick t starts it in step 2 of tick t, it resumes and ends on tick t + w, and the next fire (tick t + w + 1) starts the next run.
- A callback that holds itself until the hero leaves the trigger runs once per visit; a new visit after it has ended starts a new run.

Test vectors:
- TV-T1: callback = DamageHero(2), then wait 0.5 s (30 ticks), then return. The hero stands in the volume from tick 100. Damage on ticks 100, 131, 162, ... (2 HP every 31 ticks).
- TV-T2: a callback with no wait (the module 1174 zone, Behavior 4) runs on every tick of overlap; its 1.5 s stamp test limits the damage.
- TV-T3: the sweep routine (module 1091 trigger 43, Behavior 3). The hero enters on tick 100 and stays: one sweep, one DamageHero(2), one poison. The hero leaves on tick 400: the object scrubs back over 13 ticks and the run ends. The hero re-enters on tick 500: a new run (state word 1 again).

### 2. Animated hazard objects (module 1091, pattern A)

Eight animated triggers are reached, one for each object from 3 to 10. Each trigger runs one shared hazard helper with the slot for its object and a fixed mode. The object map is confirmed on disc.

Module 1091 has fourteen animated registrations in total. The other six are not reached from the module's entry points. Each of their callbacks calls a routine that calls neither the damage operation nor the kill operation, so they cannot damage or kill the hero.

| Object | Slot | Mode | Result |
| --- | --- | --- | --- |
| 3 | 8 | 2 | Always damage |
| 4 | 4 | 0 | Always damage in play (its slot flag stays 0) |
| 5 | 3 | 0 | As object 4 |
| 6 | 1 | 2 | Always damage |
| 7 | 7 | 1 | Always kill |
| 8 | 6 | 1 | Always kill |
| 9 | 5 | 0 | As object 4 |
| 10 | 2 | 0 | As object 4 |

Mode meanings: mode 2 always damages; mode 1 always kills; mode 0 damages while its slot flag is clear and kills while the flag is set. In play the flags of the mode-0 objects are never set (Slot flags below), so only objects 7 and 8 kill.

Helper steps for one run:
1. Take a free pending record. If none is free, the run gets index 10 and writes nothing (see Edge cases); it still damages or kills.
2. If the mode is 2, or the mode is 0 and the slot flag is clear: DamageHero(2), then wait 0.5 s.
3. Otherwise (kill branch): KillCurrentHero, wait 1.0 s (60 ticks), then MoveHero to locator 6 with the ground probe on, control option 1 and the facing on. The move places the hero at once at locator 6 on probed ground, with the locator's facing, in state 1 (hero control restored). Health is not changed by the kill or the move.
4. Release the pending record.

Slot flags (High). The move wrapper for an animated object o sets o's own slot flag to 1 and then at once to 0 (both writes use o's slot), then plays o forward with the wait on. A reverse move sets o's slot flag to 0 and plays o backward. So the net write of every move is 0. No other code sets the slot flag of objects 4 to 10 to 1 (the one-time guard writes only slot 8, and object 3 ignores its flag), and all flags are 0 on disc. In play the mode-0 objects (4, 5, 9 and 10) therefore always take the damage branch, and only objects 7 and 8 (mode 1) kill.

Damage rate. Each run deals 2 HP and then waits 0.5 s, so (Behavior 1A) a hero who keeps touching a damage object takes 2 HP every 31 ticks.

Kill outcome (High for the default-death path). On the tick of the kill the hero enters death state 16 with health unchanged and requests death action 34 (Tor clip 47; Roberta clip 61). With the hero base scale at 1.0, the hero update resets its per-tick animation factor to 1.0, and default death clears all status timers; the state-16 handler does not change the factor. The module-1091 callback does not change either hero playback scale. Thus for this path R = 1.0. Both selected main clips have F = 17 and S = 2, so their speed is 0.25 frame/tick (SPEC-0005, Playback 1a and Hero action ids). Progress is (c + t) / F; at c = 15.5, t = 0.5 it is 16/17, below 0.95, and at the next advance c = 15.75, t = 0.75 it is 16.5/17, above 0.95. The gate therefore first passes after 63 cursor advances. The first recurring damage callback retry at 31 ticks and the kill-branch move at 60 ticks both occur before that gate. A repeated DamageHero call on a hero already in state 16 has no effect (SPEC-0036, Damage); in the kill branch the move sets state 1 before the death gate can pass. Clip fade-in changes pose weight, not progress.

Test vectors:
- TV-A1: the hero touches object 6 with any flag state. DamageHero(2), then wait 0.5 s. Health falls by 2. If health is 2 or less, the hero dies (Behavior 7).
- TV-A2: mode rule only (does not happen with disc data): with the slot flag of object 4 forced to 1, a touch takes the kill branch: KillCurrentHero, then wait 1.0 s, then the hero moves to locator 6.
- TV-A3: the hero touches object 4 while its slot flag is clear. DamageHero(2), then wait 0.5 s.
- TV-A4: object 4 touched after any number of moves of any objects: DamageHero(2), wait 0.5 s; never the kill branch.
- TV-A5: object 7 touched by Tor at 40 HP, Roberta alive: state 16 and death voice 21 on tick t. At t + 60 the script callback attempts its move after the selected hero's update and animation pass. The clip has not reached 0.95 progress, so Tor moves to locator 6 in state 1 with 40 HP before the death gate passes.
- TV-A6: eleven runs at once: the eleventh gets index 10, still damages, and changes no flag.
- TV-A7: the hero stays on object 6 from tick 100: 2 HP on ticks 100, 131, 162, ...

### 3. Sweep and time scrub (module 1091, script triggers 41 to 48, pattern B)

Triggers 41 and 42 start two animation sequencers and also scrub the time of objects 2 and 17. Trigger 41 scrubs object 2 and starts sequencer 2. Trigger 42 scrubs object 17 and starts sequencer 1. Neither deals damage.

Each sequencer loops on one shared module word, and the trigger's pending record is held until its loop ends. Sequencer 1 loops while the word is nonzero, and sets the word on entry. Sequencer 2 loops while the word is zero, and clears the word on entry.

Triggers 43 to 48 run one shared routine with a sweep argument and a time-scrub object. The table is confirmed on disc for the ranges and objects:

| Trigger | Sweep argument | Sweep id range | Time-scrub object |
| --- | --- | --- | --- |
| 43 | 6 | 36 to 40 | 16 |
| 44 | 5 | 33 to 35 | 15 |
| 45 | 4 | 28 to 32 | 14 |
| 46 | 3 | 23 to 27 | 13 |
| 47 | 2 | 21 to 22 | 12 |
| 48 | 1 | 18 to 20 | 11 |

Routine for one run of triggers 43 to 48:
1. Take a pending record.
2. If the trigger's state word is 1 (the hero is inside): scrub the time-scrub object's animation through the times 0, 1/30, 2/30, ..., 10/30 s, waiting one tick after each step.
3. Sweep: for each id in the range, in order, call PlayAnim(id, 0, 5.0), then wait 0.02 s. This runs whatever the state word is. The id is an animated-object index of pack 1091: PlayAnim's only integer is the object, the 0 is the wait mode (no wait) and 5.0 is the speed factor, so each call plays that object's own clip once at five times normal speed (SPEC-0030 row 2). The sweep objects (18 to 40) are distinct from the time-scrub objects (11 to 16) (Medium-High).
4. DamageHero(2), then SetHeroStatus(2, -1). This is poison at the default 120 s with the default period and amount (Behavior 7).
5. Wait until the hero leaves the trigger.
6. Scrub the object through the times 13/30, 14/30, ..., 25/30 s, waiting one tick after each step. This runs whatever the state word is.
7. Release the pending record.

The routine has no edge test of its own. Step 5 holds the run until the hero leaves, so within one visit the sweep, the damage and the poison each run once. A new visit after the run has ended starts a new run (Behavior 1A, TV-T3).

Test vector TV-B1: trigger 43 runs once. The sweep calls PlayAnim for ids 36, 37, 38, 39 and 40 at speed 5.0, with 0.02 s waits (0.1 s in total). Then the hero takes 2 damage and is poisoned for 120 s. The routine then waits for the hero to leave, and no second sweep, damage or poison runs during that visit.

### 4. Damage zone with cooldown (module 1174, pattern C)

A script trigger callback for a zone. If the zone trigger has an active object, and the system clock now minus the stored clock is more than 1.5, the callback stores the clock now and calls DamageHero(3). The comparison is strict. The clock is the game clock in seconds (GetSystemTime), so the cooldown is 1.5 s.

Object-word gate (High). "Active object" means that the trigger record's object word is non-zero, which is the case only when the hero's own trigger-hit path fired the trigger (SPEC-0030, Layout 3). Fires from the hero's per-tick volume pass carry object 0, and the callback does nothing on them.

Call rate (High). The callback has no wait, so (Behavior 1A) it runs on every tick of overlap, and each run reads the game clock. The 1.5 s stamp test alone limits the damage.

The stamp is one word shared by every zone that uses this callback, not one per zone. A hit on one zone therefore blocks hits on the other zones for 1.5 s.

Switching zones (High). A stage routine of the module switches a zone group on: it starts the group's effects and registers the damage callback on the group's trigger index. A zone group is switched off by registering an empty callback (one that does nothing) on the same trigger index. The first stage that raises zones (the first fight stage, groups of three records) also sets the shared stamp to the game clock at that moment, so no zone hit happens in the first 1.5 s after that stage; later stages do not reset the stamp.

Test vectors (game clock in seconds):
- TV-C1: zone active, stored clock 0. At clock 10.0: damage 3, stored clock becomes 10.0.
- TV-C2: at clock 11.2 (difference 1.2): nothing.
- TV-C3: at clock 11.6 (difference 1.6): damage 3, stored clock becomes 11.6.
- TV-C4 (synthetic float32 clock): the first stage raises zones at an exactly represented clock value of 20.0 and stores that value. After 90 passes of adding the 32-bit tick length 1/60, the clock is 21.499977 and the difference is 1.4999771, so no hit occurs. After 91 passes the clock is 21.516644 and the difference is 1.5166435, so the strict > 1.5 test hits and stores the new clock. For other absolute clock values, float32 subtraction can shift the interval; do not assume every repeat is exactly 91 passes.
- TV-C5: a fire from the hero's per-tick volume pass (object word 0) at any clock: nothing.
- TV-C6: the zone group switched off (empty callback registered): the hero inside takes no damage.

### 5. Zone helper with a locator list (module 864, pattern D)

Confidence: High; hit timing is stated relative to the first eligible control-thread pass (Open questions).

Zone lists. At area entry one of the module's control threads fills three zone lists (Layout): zone A = animated objects 0, 1, 2 and 3; zone B = objects 4 to 8; zone C = objects 9 to 14. The same thread also starts each object's loop, with staggered start times. The module's exit hook empties the lists.

Caller. Module 864's Init allocates eleven control threads and starts all of them unconditionally. One of them loops forever: run the helper over zone A, then zone B, then zone C, then sleep 1 tick. The thread runs only while pack 864 is current (control threads of a world that is not current are not stepped; SPEC-0030, section 1).

Helper. For each object id in the list, in order: d = the full 3-D distance from the selected hero's position to locator 0 of that animated object (SPEC-0030; the distance is 0.0 when the object is missing, which does not happen here: pack 864 has 164 animated objects). If d < 2.0 (strict): DamageHero(10), then wait 0.3 s (18 ticks). The wait is inside the loop, so the thread stalls on each hit before it tests the next object. The helper always calls a debug print for each in-range object, with no gate on it.

First-hit phase. Init starts the list-building control thread in an earlier slot than the damage thread. On the first eligible per-tick script pass after pack 864 becomes current, the list builder runs before the damage thread, so its lists are populated before the hazard scan. The damage thread starts with zone A on that pass; the first object in A/B/C order whose locator 0 is within 2.0 units takes effect immediately, with no initial one-tick sleep. This is conditional on that script pass running (the world is current and script stepping is not frozen).

Rate. Within 2.0 of one object only: 10 HP on the first eligible scan and then every 19 eligible control-thread passes (the 18-tick wait plus the 1-tick sleep). Within 2.0 of k objects: k hits per loop, 18 ticks apart, then the 1-tick sleep.

Sibling threads. Two other control threads of the module use the same three lists. One starts a camera shake when the hero is within 35 units of locator 0 of any listed object (only while a module word is set; the strength grows with 35 / d), then waits 0.05 s. The enable word is 1 in the module file, so the shake is on in normal play. Two scripted scenes clear it while they run and set it back to 1 at their end: the scene of script trigger 22 (it runs once, guarded by its own once-word; its end sets flags 288 and 289 and posts a blackboard message) and a scene run by another of the module's control threads when its level-state and game-state tests pass (its end starts effects and sets flag 293). So the shake is off only during those two scenes (Medium-High). The other moves effects 38 + i, 46 + i and 51 + i to locator 1 of each listed object every tick. Neither deals damage.

Test vectors:
- TV-D1 (synthetic list): a zone with 8 object ids and the hero within 2.0 units of all of them. One pass deals 80 damage over 2.4 s (8 waits of 18 ticks).
- TV-D2: the hero 1.9 units from object 2's locator 0, no other object near, from tick 100: hits on ticks 100, 119, 138, ... (10 HP each).
- TV-D2a: on the first eligible control-thread pass after module 864 becomes current, zone A contains object 2 and the hero is 1.9 units from its locator 0, with no other object near: the first hit is on that pass; subsequent hits are 19 eligible control-thread passes apart.
- TV-D3: the hero exactly 2.0 units away: no hit.
- TV-D4: the hero within 2.0 of objects 0 and 5: hit (object 0), 18 ticks, hit (object 5), 18 ticks, 1 tick, repeat: 2 hits per 37 ticks.
- TV-D5 (shake gate): the hero 20 units from object 0's locator 0 and more than 2.0 units from every listed object, in normal play: shakes start, about one every 0.05 s, with no damage. The same spot while the trigger 22 scene runs: no shake until that scene ends.

### 5A. Other module hazards (modules 1090, 1089 and 1381)

Confidence: High for the calls, the gates and the constants; Medium for how often a player meets each one. The full module scripts are in SPEC-0041 (1090) and SPEC-0042 (1089, 1381).

- **Module 1090, hurt zones.** Two banks of seven hazards each have a trigger. Threads started by the initializer fire the hazards in repeating four-step patterns (0.7 s per firing in bank A, 0.5 s in bank B; the steps follow each other with no rest, and a hazard that the next step arms again stays active without a break). While the hero stands in the trigger of a hazard that is firing, the zone routine calls DamageHero(20) and then Wait(1.0 s), and tests again one tick later: the hero loses 20 HP once every 61 ticks. A zone whose hazard is not firing does nothing.
- **Module 1089, crusher hazard thread.** The first trigger of the module (index 0) starts the sequence and, after its setup scene and a 30-tick sleep, sets a module word that is 0 at load. While that word is set, a control thread started by the initializer checks 12 animated locators in turn and, for each locator whose straight-line (3-D) distance to the hero is below 6.0 (SPEC-0042, module 1089) and whose height is less than 2.0 above the hero's, calls DamageHero(1) and Wait(0.5 s). The word is cleared when the first object's animation time passes 18.4 s. If fewer than 4 button presses were counted, the update then calls KillCurrentHero when that time passes 19.9 s and moves the hero to locator 0 (the sequence can run again); with 4 or more presses the objects slow down and stop and nobody is killed. Outside death callbacks and the module 1091 kill objects, this is the only level-script path that kills a hero.
- **Module 1381, falling hazards.** Six falling hazards bind their trigger to a damage routine for part of each cycle. The routine calls DamageHero(5) and then Wait(2.0 s), and only while a module word is set. That word is 0 at load and is set when the hero uses the warp pad with trigger index 3.
- None of these calls checks the hero's health: damage equal to the health kills (Behavior 7).

### 6. Kills and falls

- KillCurrentHero kills the active hero. The death callbacks and the rules after a death are in SPEC-0055. In pattern A the kill branch then waits 1.0 s and moves the hero to locator 6 in state 1 with its health unchanged (Behavior 2, Kill outcome).
- StartHeroLongFall starts a long fall (state 48). It is a movement, not a damage call. Fourteen sites in twelve modules call it (Behavior 1). It is the area-entry arrival animation: the script moves the hero to a locator high up, starts the long fall, runs a tracking camera and a fade in, and waits (SPEC-0033, Behavior 9B).
- No fall damage exists (High). The jump and fall air states, the hard landing and the long fall never call the hit or the deduct routines, so no fall of any height changes health. The clean side must not add fall damage.
- Hard landing: the fall height is the take-off height (the start point stored when the jump or fall began) minus the hero's height at the landing decision, not the height of the apex. The decision is taken up to 1.0 above the ground, so the real drop can be up to 1.0 more than the fall height: a walk-off from 3.75 is soft and from 4.0 hard (SPEC-0033, Behaviors 9A and 50). A fall height above 3.0 (the data word 0x2DD828) gives the hard landing (animation 118), which deals no damage (SPEC-0033, Behavior 50).
- The soft-landing camera shake for a fall height above 4.0 (the data word 0x2DD814) can never run: any fall above 4.0 is already above 3.0 and so takes the hard landing first. Reproduce: a landing never shakes the camera.
- Falls kill only through ground data: deadly ground and the deep overlay rule (Behavior 7A).

Test vectors (synthetic):
- TV-F1: a jump from y = 5.0 whose landing decision is taken at y = 1.5 (fall height 3.5): hard landing, animation 118, health unchanged.
- TV-F2: a jump from y = 5.0 whose landing decision is taken at y = 2.5 (fall height 2.5) with the stick centred: soft landing 68, no camera shake.
- TV-F3: fall height 4.5: hard landing, no camera shake, health unchanged.
- TV-F4: StartHeroLongFall from 40 units up: state 48 with animation 136; the hero reaches the ground on tick 82 and clip 137 starts on tick 83 (SPEC-0033, Behavior 9B); health unchanged.
- TV-F5: walk-off from a ledge 3.75 high: soft landing; from 4.0: hard landing (animation 118); health unchanged both times.

### 7. Status damage: the status-apply method and the per-tick effects

A status is applied by calling the target's own status-apply method, one per class (SPEC-0036, Status apply). Every source uses the method of the target: SetHeroStatus, items, enemy bites and projectiles, the plague serpent's cloud and the save restore all reach the hero's method; the hero's Mote attacks reach the enemy's method. There is no separate script route and hit route. The two methods differ as follows (High):

| Item | Hero method | Enemy method (SPEC-0029) |
| --- | --- | --- |
| Death gate | None | Nothing is applied to a dead enemy |
| Element gate | None | The enemy ignores the status of its own element: stun when its target type is 4, curse when 6, poison when 2, slow when 5, confusion when 3, fire when 1. Strength, speed and poison attack have no gate. |
| Immunity check | Curse is blocked while status 8 is active, poison while status 9, slow while status 10, from every source; the block plays effect 0xC2, 0xC3 or 0xC4 | None (enemies have no immunity statuses) |
| Default duration (s) | Stun 5, curse 60, poison 120, slow 60, strength 60, speed 60, fire 5, status 12 and 13 60, statuses 8, 9 and 10 30 | Stun 5, curse 60, poison 120, slow 60, strength 60, speed 60, confusion 30, fire 5, poison attack 30 |
| Confusion (6) and poison attack (11) | Ignored | Applied with the defaults above |
| Statuses 8, 9 and 10 (immunity) | Clear the matching status (1, 2 or 3) if active, then set the immunity timer | Ignored |
| Display start | Runs when the tick count is 1 or more | Runs when the tick count is 0 or more |
| Poison and fire extra values | Stored in the hero's poison and fire words | Stored in the enemy's poison and fire words |

Duration unit: whole seconds, converted to ticks by multiplying by 60. A duration of -1 means the default. On the hero, a count of 0 sets the timer to 0 without the display start, and a negative count changes nothing.

Per-tick effects on a hero (each tick, after the timers are reduced; SPEC-0036, Status effects):
- Poison: while the poison timer is active, when the post-decrement timer is a multiple of (period in seconds x 60), the hero takes poison damage through the take-hit with attacker id -2. The hero's poison words are -1 from its creation, and every source passes -1 for a hero, so hero poison is always 2 HP every 4 s (High). The damage is the amount when health minus the amount is at least 2, and health minus 1 otherwise, that is, the smaller of the amount and health minus 1: poison never kills a hero, and at 1 HP it deals 0.
- Fire: no per-tick effect on a hero. A fire status on a hero, from SetHeroStatus(7, ...) or from an enemy attack, sets the fire timer, shows the fire effect and tint, counts down and expires; it deals no damage. The enemy per-tick routine deals fire damage (period 2 s and 2 HP by default, attacker id -1; SPEC-0029).
- Confusion: the hero's method ignores it, so it has no effect on a hero from any source.
- Other statuses that act each tick (SPEC-0036): stun puts the hero in state 1 and freezes its animation every tick; status 12 gives +2 HP and status 13 +2 MP on ticks where the status-12 timer is a multiple of 60 (status 13 reads the status-12 timer). None of these is a hazard effect.

Damage deduction (applies to every hit). If the damage is less than the hero's health, the damage is subtracted. Otherwise health becomes 0 and the death handling runs. So damage equal to health kills the hero. Poison on a hero is limited first by the 1 HP floor above, so it never reaches this kill case; script damage (DamageHero), enemy hits and the hurting surface have no floor.

Periodic counts:
- A fresh poison of 120 s (7200 ticks) hits on post-decrement timer readings that are multiples of 240 above 0: 6960, 6720, ..., 240. That is 29 hits, at 4 s, 8 s, ..., 116 s after the apply, for at most 58 HP in total; the floor stops the hero at 1 HP.
- A fresh fire of 5 s (300 ticks) on a hero deals nothing: the timer counts down from 300 and expires. (An enemy in the same case takes 2 HP at readings 240 and 120, 4 HP in total; SPEC-0029.)

### 7A. Ground rules of the hero motion tick

Confidence: High for the rules, the constants, the disc counts and the height value; Medium-High for the meaning of the surfaces (2 hot, 8 water).

For a character with the hero flag set (both heroes) that has a ground entry under it, each motion tick applies the first rule that matches:

1. Ground kind 2 (deadly ground): when the height value is 0.085 or less, the hero dies the deadly-ground death of SPEC-0055, section 8A, if the entry carries a respawn entry index (byte at +0x0D). If that byte is 0xFF (none), both heroes are killed instead: the partner first, then the hero on it, both with the death gate allowed. The kill runs again on every tick the condition holds (SPEC-0055). Deadly ground itself deals no per-tick damage and no knockback.
2. Otherwise, a ground entry with surface word 2 (a hurting surface): while the height value is below 0.2, the hero is hit for 10 HP on each tick where clock - trunc(clock x 0.4) x 2.5 < dt, with clock the game clock in seconds and dt the tick length (1/60 s). That is once every 2.5 s of game clock, at the clock times 2.5, 5.0, 7.5, ...; the phase is set by the clock, not by the moment of contact. The hit is a normal hit: attacker id -1 and reaction flag 0, so the hit cue and the state-17 flinch play (SPEC-0036, Taking a hit), with no knockback impulse. It has no floor: a hero at 10 HP or less dies.
3. Otherwise, when the ground entry links to an overlay entry (its field at +0x0E), the overlay passes a point test at the hero's x and z, and the overlay is not below the ground: if the overlay height is more than 2.3 units above the hero's position y (strict), both heroes are killed: the partner first, then the hero on it, both with the death gate allowed (SPEC-0055, section 8A). Otherwise, when the overlay's surface word is 2, the hurting tick of rule 2 applies, with no height-value test at all (the 0.2 limit belongs only to a hurting ground entry under the hero).

The height value is the motion record's ground gap: the hero's position y minus the ground height below it (High).

The motion tick has one early exit that skips these rules. It depends on three class queries, and the hero class answers all three with 0, so the early exit never applies to a hero (High).

Test vectors (synthetic):
- TV-G1: the hero stands on a surface-2 entry of kind 0 with height value 0.1. With the clock rising from 2.49 s, the hero is hit for 10 on the tick where the clock passes 2.5 s, and next at 5.0 s.
- TV-G2: the same entry with height value 0.25: no hit.
- TV-G3: a hero who steps onto the surface at clock 2.51 s takes the first hit at 5.0 s (2.49 s of grace).
- TV-G4: the hero at y = 0 over a linked overlay of height 2.4: both heroes are killed. Overlay height 2.3: no kill (the test is strict).
- TV-G5: a deadly-ground entry (kind 2) with surface word 2: rule 1 applies, so the hurting tick does not run on it.
- TV-G6 (surface meaning): a hero killed on a deadly-ground entry with surface word 2 plays death clip 173 with the hot burst 0x81; with surface word 8, clip 174 with the water splash 0xBC; with surface word 0, clip 136 and no effect (SPEC-0055, section 8A).
- TV-G7 (disc-backed entry): pack 696 entry 121 is a kind-0 surface-2 ground entry. With that entry underfoot, ground gap 0.1 and the clock just before 2.5 s, there is no hit before the phase boundary; on the tick crossing 2.5 s, the hero takes 10 HP.

### 8. Edge cases

- Pool overflow (needs eleven concurrent runs; the shared pool holds ten): the eleventh run gets index 10 and the allocator writes nothing. The run still damages or kills as usual. Its release clears flag words 0 and 9, which no object uses. Overflow has no effect on the hazards (TV-A6).
- Kill-branch timing: with the default-death state-16 path and no external status or playback-scale change, R = 1 and the 17-frame S=2 death clip reaches 0.95 after 63 cursor advances. The 1091 callback's 60-tick move runs first; fade-in does not change progress (Behavior 2, Kill outcome; SPEC-0005; High).
- Mode-0 kill: with disc data no mode-0 object ever kills; the kill rule of mode 0 is unreachable in play (Behavior 2).
- Zero period word: no source passes a zero period, and the hero's words stay -1, so the periodic test never divides by zero (Resolved, High).
- Duration 0 on a hero: SetHeroStatus(2, 0) sets the poison timer to 0. No hit, no display start.
- Immunity: with status 9 active (timer 3600), SetHeroStatus(2, -1), an enemy bite with poison and the serpent cloud all write no poison timer and no extra words; effect 0xC3 plays.
- Confusion and poison attack on a hero do nothing from any source: SetHeroStatus(6, -1), SetHeroStatus(11, -1) and an enemy bite with confusion change nothing.
- Fire on a hero: any fire apply sets the timer and runs the display start under the hero's timer rule; no fire damage reaches a hero from the status (Behavior 7).
- Element gate on enemies: a poison hit on an enemy of target type 2 applies nothing and writes no extra words (SPEC-0029). Heroes have no such gate.
- Hurting surface phase: entering the surface just after a 2.5 s mark of the game clock gives up to 2.5 s of grace; entering just before gives a hit at once.
- Dead hero: the hazard triggers do not check whether the hero is dead (SPEC-0044, module 1091; Low). A hit on a hero in the death state (DamageHero, an enemy hit, the hurting surface) plays the hit cue and the hit effect and then stops: no stun clearing, no reaction and no damage (High; SPEC-0055 section 1, SPEC-0036 Taking a hit). While a dialogue box or a list picker is up, every hit on a hero is discarded, cue included (SPEC-0055). The hero's status-apply method has no death gate, so a status can still be set on a dead hero. A kill (KillCurrentHero) of a dead hero runs the default death again (SPEC-0055).

### 9. Test vectors (summary)

- TV-S1: SetHeroStatus(2, -1) on a hero with no statuses. Poison timer 7200. First hit at reading 6960 (4 s after the apply); last at 240 (116 s). 29 hits, 58 HP.
- TV-S2: SetHeroStatus(7, -1) on a hero at 10 HP. Fire timer 300. No damage at any reading; health stays 10 and the timer expires after 5 s.
- TV-S3: status 9 active with timer 3600, then SetHeroStatus(2, -1). The poison timer is not changed. No extra words written. Effect 0xC3 plays.
- TV-S4: SetHeroStatus(2, 0). Poison timer 0. No display start. No poison.
- TV-S5: poison amount 2 on a hero at 2 HP deals 1 (health 1, no death). At 1 HP it deals 0. A hero at 3 HP takes 2 and has 1 left. DamageHero(2) on a hero at 2 HP has no floor: health becomes 0 and the hero dies.
- TV-S6: a hero at 10 HP with the poison timer at 240 after a decrement takes 2 HP (health 8). At 239 or 241 nothing happens. At 480 the hero takes 2 more.
- TV-S7: hero poison from 10 HP with the default words: health 8, 6, 4, 2, then 1 (damage 1), then hits of 0. The hero never dies of poison.
- TV-S8: an enemy with status 11 active bites the hero (row status -1): the hero is poisoned for 120 s.
- TV-S9: the plague serpent cloud reaches a hero who is already poisoned: nothing happens. A hero with status 9 active: nothing is applied and effect 0xC3 plays. Otherwise: poison for 15 s (900 ticks) at 2 HP every 4 s, hits at the readings 720, 480 and 240 (6 HP).
- TV-S10: hurting surface and overlay: see TV-G1 to TV-G5.

## Open questions

Numbers of removed questions are not reused.

2. Script-trigger state word. Resolved: the state word is 1 on the first fire and 2 on later fires while the hero stays inside; the call rate is in Behavior 1A. **[High]**
3. The clock read by GetSystemTime. Resolved: the game clock in seconds (High).
4. Sweep id. Resolved: in PlayAnim(id, 0, 5.0) the loop id (such as 36 to 40 for trigger 43) is an animated-object index, 0 is the wait mode and 5.0 the speed factor; each call plays that object's clip once at 5x speed without waiting (Behavior 3). **[Medium-High]**
6. The hero's poison and fire words. Resolved: -1 from the hero's creation, and every source passes -1 for a hero, so hero poison is always 2 HP every 4 s and no source passes a zero period (High).
9. Module 1089. Resolved in part: its initializer registers four animated triggers to a handler that dispatches to a button/animation routine. The resource's pack is present in the static stream graph via pack 1096, though this does not establish that a particular trigger instance fires in a playthrough. One `DamageHero(1)` call (the hazard thread) and the `KillCurrentHero` call (the timeout of the crusher sequence, in the update) are on paths that the initializer starts; the second `DamageHero(1)` call is in a body nothing references. The two live calls sit behind module words that are 0 at load and are set by the first trigger, so they are not dead (Behavior 5A). **[High for the thread, the update path and the gates; which trigger instance the player first touches depends on the player's path and is not decided by static data; Default if unresolved (rewrite choice): arm the module's gates when its first trigger fires, as described above (needs a runtime check)]**
15. Fire status on a hero. Answered: it shows the fire effect (effect 0xCA on three emitters) and the fire tint and deals no damage, and confusion is ignored from every source (SPEC-0036, Status display and cues). Fire reaches a hero through the enemy status rolls of bites and projectiles and through SetHeroStatus(7, ...), which no level module uses; which enemy rows and projectiles carry fire is in SPEC-0029.
16. Surface meanings. Resolved (High for the rules, Medium-High for the labels): surface word 2 is a hot surface, not acid, and surface word 8 is water (the deep overlay); the game ties them to the hot-burst and water-splash deaths (Layout). The exact pack-local ordinals of all 23 non-deadly surface-2 entries are listed in Layout. Pack 879's entry 4 is in a pack with no module and no incoming transition edge in the disc route graph, so that record is disc-present and treated as not player-reachable (SPEC-0006; Default if unresolved (rewrite choice): keep the rule on that record, it is harmless if never loaded). Naming the other entries by sub-area or map location is not needed: the rules read each ground entry's own surface word, so no place name is used by play. **[High for the entry inventory; Medium-High for the surface meanings and pack 879 reachability boundary]**
18. Repeated overlay kill. Resolved: the deep overlay and deadly-ground rules run on every tick the condition holds, and each kill of a hero already in the death state re-runs the default death (SPEC-0055, repeated death and section 8A). **[Medium-High]**
19. Partly answered (Behavior 4, TV-C4; Behavior 5, first-hit phase): from a synthetic 20.0-second stamp, float32 accumulation of the fixed 1/60-second step first satisfies the strict 1.5-second cooldown after 91 passes. Other absolute clock values can shift the interval through rounding. In module 864, the earlier list-building control thread runs before the damage thread in slot order; the first scan therefore uses populated lists and can hit immediately on the first eligible pass after the pack becomes current. With one qualifying object, later hits are 19 eligible control-thread passes apart. Exact wall-clock delay still depends on when the pack becomes current and whether script stepping is frozen. Default if unresolved (rewrite choice): use the 91-pass interval from a 20.0-second stamp and count passes, not seconds (needs a runtime check for the real stamp values). **[High for scheduler and module order]**
20. Resolved for the module-1091 default-death path: state 16 runs at R = 1 because the hero base scale is 1.0, the hero update resets the per-tick factor to 1.0, default death clears statuses, the state handler does not alter playback scale, and the module's callbacks do not set one. Action 34 selects a 17-frame one-shot with S = 2, advancing 0.25 frame/tick. With progress (c + t)/F, the 0.95 gate first passes at c = 15.75, t = 0.75, after 63 advances. The first 31-tick damage retry and the 60-tick kill-branch move both precede it; a repeated damage call on the already-dead hero has no effect, while the kill-branch move resets the hero to state 1 before the gate. External status application or world-pause changes are outside this module-specific result. **[High]**
21. Camera-shake gate in 864. Resolved: the enable word is 1 in the module file, so the 35-unit shake is on in normal play; only the scene of script trigger 22 and the second scripted scene clear it, and each sets it back to 1 at its end (Behavior 5). **[Medium-High]**

