# SPEC-0029: Enemy and creature behavior framework

- Answers: proactive, none requested (enemy and creature AI); R-0013 (in part); R-0016; R-0034 (items 2 to 4, in Behavior 9 and the reflection charge); R-0039 (in part; items 1 and 2 resolved, Open questions); R-0066; R-0067 (boss slot values, Behavior 2); R-0073 (Behavior 5, flee sub-steps A and B); R-0082 (Behavior 13); R-0088 (Ixi Chief movement modes and steering)
- Status: Draft
- Disc build: SCUS-97367 v1.00 (same fingerprint as SPEC-0001). The tuning words below are in the boot program's data; read them by virtual address through the program headers.
- Confidence: per section. Miamouse (Behavior 28): High for the states, the dispatch, the catch, carry, release and delivery rules, the counts and the cage meshes. Mortog zone count and launch summary: High for the counter arithmetic; High for the launch value (a distance) and the hold end. No fishing, racing, scoring or gambling mechanics: Medium-High. Target Dummy (Behavior 29): High for the structure, the interval and the hit reset; Medium for the animation indices. Queen Fyora controller (Behavior 30): High for the wait, the states, the chooser, the slots and the cooldown range; Medium for the deflect side. Darkest Faerie volley and timed state machine (Behavior 18): High for the four-shot count, the four spawns by key, the state machine and the owner; Medium-Low for the start and reset conditions. Controller slot handlers of classes 37 and 38 (Behavior 19): High for the owners and the constants; High for the shared class-38 array; Medium-Low for the class 37 steps. Animation steps and level-state posts (Behavior 2c): High. Serpent phase check (Behavior 2d): High. Class 37 loops A and B (Behavior 7): High; Medium for the push-out and the pack of loop B. Enemy decision tick (Behavior 20): Medium-Low (words High). Engagement probe and contact test (Behavior 21): Medium-Low. Timed flame-wave attack (Behavior 22): a cross-reference to SPEC-0019, which holds the full description. Behavior 16 is a pointer: the character light slots are in SPEC-0046. Actor-list dispatcher (Behavior 17): Medium-Low. Framework (one thinker per tick, behavior ids, priority rule): High. Class id map (Layout section 3): High for ids, routines and the owners of the routines that SPEC-0038 keeps for other classes; Medium for names not yet confirmed. Class 6 (townsfolk, Layout section 3): High for the identity, the default-class rule, the census and the 14-value dispatch; the mode bodies are specified in SPEC-0056. Navmesh pick (Behavior 2): High; movement helpers: Medium. Mortog: High for states, zone rules and constants. Whinny: High for modes and pen rules, Medium for the group sound. Draik Skeleton: High for stages, timers, attack choice and hit registration and the attack dispatch; High for the fixed bone-index mapping and level-setter values; Medium for which live records consume the level configuration, the meaning of hit shapes, the hero-side calls, the attack handler bodies (Section 6.8; attack-end state, spit and orbit numbers now High) and the breath beam ranges and damage words (Section 6.5; High). Draik flying height keeper (Section 6.9): Medium-High for the rule; High for the stage gate and for the writers of the flying offset; Low for the offset it uses after death. Draik base state and killing hit (Section 6.10): High. Ghosts (Behavior 33): High for the rule and the rows; Medium for the hidden and attacking tests. Death, despawn, drops and SetAI_Agro for every class (Behavior 34): High. Class inventory and reach (Layout section 3a): High. The 50-second timer (Behavior 18; Open question 58): High. It is the Darkest Faerie class-37 absorb-window timer: the final-fight controller starts the four-entity burst, initializes this window, then waits in the absorb state until three absorbs cause the fall or the 3000-tick timer expires. Steering look-ahead (Behavior 2a): Medium; its sideways rule: Medium-Low. Nearest triangle and goal latch (Behavior 2b): High; the vertical part of the distance is open. Boss encounter latch (Behavior 8): Medium; the boss is not named. Faerie Sisters controller (classes 13, 15, 16; Behavior 4): High for class ids and tuning words; Medium for state roles and alternate-loop selection. Controllers for classes 37 and 38 (Behavior 7): Medium; the boss is not named. Monster parameter schema record layout (Layout section 2): High for count and layout, Medium-High for field order. Enemy projectile pool and per-tick update (Behavior 9): High for the single ten-slot first-free pool, the slot layout, the lifetime rule, the units, the motion, the gravity and the homing blend, and the behaviour kinds; Medium-High for the hit on the hero and on the owner, the world collision and the capsule shape; Low for the capsule end cap below the base; High for the extra effects of kinds 14 and 15, Medium-High for the mote-eater group duration in ticks (the kind 9 rule is in SPEC-0016). Zone chooser (Behavior 12.13): Medium. Spawn-slot binder (Behavior 14): Medium. Grab-and-throw machine, alarm-group check, pursuit and part capsule sweep of the common enemies (Behaviors 23 to 26): Medium for the control flow (High for the alarm-group gate), Low for the names of the tests they call; the part capsule sweep (Behavior 26) is High for the walk over the target's bone capsules and the shape. Enemy status apply and tick (Behavior 27): High for the apply, the element immunity, the per-tick values and status 11; Medium for the meanings of the two class tests and for confused sub-state 11; Medium-Low for the stun flinch rule. Monster footstep shake and rumble (Behavior 32) and the grab-and-throw rumble (Behavior 23): High. Tuning words read from disc (Where it lives): High for values, Medium for meanings. Draik missile dodge (Behavior 10.4): High; it never runs in play. Monster table (per-type parameter values, resources 530/531): High for the lookup and the core fields, Medium for field meanings. Common-enemy class 17 behaviour: see SPEC-0038. Contact, reach and hit reaction (Behavior 10): Medium-High for the weapon volumes, the reach test and the shared enemy damage (10.2, 10.5; High for the weapon row fields and event-bit gates); High that the push routine of 10.1 is never reached (do not implement); High for the knockback on the hero (10.1a) and for the deflected hero shots (10.6); High for the owners and the flow of 10.3 and 10.4. Attack release handlers and their recovery draws (Behavior 11): Medium for the animation indices and timing ranges; the spawn kinds and the classes that use them are not known. Behavior 12 owner map: High except for marked Medium items; 12.10 is the class-38 slot type-2 handler of Behavior 19, and class 17 uses SPEC-0038. Behavior 12.8 is the live petpet timer state, not a common-enemy cooldown (SPEC-0038 Behavior 27). Projectile spawners (Behavior 13): High for the four spawners, their kinds, aim, lifetimes and the ballistic formula; the per-tick test vectors follow from them. Message handling (Behavior 2c): Medium-Low; its two patterns are High and the code meanings are open. Health bands (Behavior 2d): Medium. Approach, homing and glide steps (Behavior 2e): Medium-Low. Six-entry value tables (Behavior 2): High for the values, Low for their meaning. Enemy record pool (Layout section 1a): Medium. Kind search for classes 37 and 38 (Behavior 7): High for the search, Medium for its link to those classes. Words read from disc for this spec are listed in "Where it lives on the disc"; fixed constants are marked as such and were not taken from the disc. Monster target type T and attack damage alignment (Layout 4): High for offsets, values and counts; Medium-Low for the labels of T. Enemy defence (row offset 0x114): High for the field and its reader; Medium-High for its use in the damage formulas. Enemy drop and coin fields (Layout section 1: spawn record, direct drop item, carried Neopoints): High for the offsets and the creation values; Medium-Low for the carried-Neopoints growth in the item branch (Behavior 11b). Steal fields (Layout section 4, Behavior 11a and 11b): High for offsets, table values, the chance and range rule and the Neopoint branch; Medium-Low for the item branch. Display name (Layout section 4c): High. Spawn by key and summon-reserved records (Behavior 31): High for the search, the reserved-record rule, the row keys, the four sources and the start in agro; Medium for the spawn direction of the member summons. Element by variant and no per-area scaling (Scaling section): High.

## Summary

Every enemy and creature runs as one coroutine that performs one "think" step per game tick (1/60 s, SPEC-0007) and then yields one tick. Each enemy carries a behavior class id from 0 to 39 (0 = none) that selects which think routine it runs. Layout section 3 maps the ids to their routines and to the specs that describe them. A new class replaces the current one only when its priority is at least as high. No framework routine chooses a target: class-17 enemies always use the selected hero (SPEC-0038 Behavior 6), and a navmesh pick keeps the first ground or flying navmesh with a triangle that contains the character's position (Behavior 2).

This spec describes the shared framework and four behaviours in full: Mortog (a creature that flees the hero, holds in a target zone, and wanders while avoiding it), the Faerie Sisters controller (classes 13, 15 and 16: three sisters, each with its own state block, sharing one alive count; the sisters' player-facing behaviour is in SPEC-0011), Whinny (a group that wanders, flees the hero, and is herded into pens) and Draik Skeleton (a timer-driven creature with three stages, attack cooldowns, an attack set dispatched by attack number, breakable parts and a breath beam). Two controller families are routed here: class 37 is the Darkest Faerie family's three-way entry dispatcher, and class 38 is the Queen Fyora controller, which waits for a class-37 object and aims at it (Behaviors 7 and 30). The Darkest Faerie fight itself is specified separately. Two non-boss routines are also specified here: the miamouse (class 21, Behavior 28), which deals no damage, and the Target Dummy (class 36, Behavior 29). Per-monster parameter values live in a monster table (resource 530, with a companion table in resource 531), selected by fields of the spawn record. The common enemies (skeletons, spyders, slorgs and others) all use one shared behaviour class (17), whose think routine is specified in SPEC-0038. Enemy movement uses a navmesh pick (Behavior 2), a steering look-ahead (Behavior 2a) and a nearest-triangle search with a goal latch (Behavior 2b); no framework routine chooses a target. A boss encounter latch (Behavior 8) raises an encounter event once per enemy while a progress flag is clear. Enemy projectiles live in one global pool of ten slots, shared by all enemies and bosses: a new projectile takes the first free slot, and a full pool spawns nothing (Behavior 9). The pool is updated once per tick from the main loop: each live slot counts down its lifetime in ticks, moves by its velocity in units per tick (with gravity or homing by kind), tests one character against its body capsule, and tests the world. Missiles are set up by three flight-mode spawners and a lobbed summon (Behavior 13), and spells by the common enemies' spell sub-types (SPEC-0038). An enemy group can also bind a new member into a slot, refused when the group is full or the kind is already present (Behavior 14), and a three-zone distance chooser selects states from the distance to the target (Behavior 12.13). Many state routines play an animation and act when it finishes, and set numbered level states (Behavior 2c); the Plague Serpent checks health bands that set state 11 (Behavior 2d); and descent, homing and glide steps are listed by owner (Behavior 2e). The enemy framework also has a contact test and a reach test against the hero, the Ixi Chief's incoming-hit check (Behavior 10.3) and the Draik's missile dodge, which never runs in play (Behavior 10.4); the release handlers that play an attack animation and spawn a projectile or leap on its release event (Behavior 11); a set of generic AI controllers whose classes are not yet named, with their timers and random choices (Behavior 12; the timers once listed as ambient cue timers are the hero idle picker of SPEC-0033); and four spawners that set up enemy missiles for the pool (Behavior 13). Boss-specific routines of these families (the Gelert, Plague Serpent, Werelupe King and Gnarfas attacks, and the Faerie Sisters energy ball) are described in their boss specs. This spec also carries three routines that appear in later sections: the Darkest Faerie's four-shot volley with a timed hit-counting state machine (Behavior 18), the slot handlers of the class 37 and class 38 controllers with their homing and flight steps (Behavior 19), the enemy decision tick with the melee-dodge and missile-dodge rolls of the Faerie Sisters (Behavior 20), the engagement probe and contact test (Behavior 21), and the Werelupe King's timed flame-wave attack (Behavior 22, described in full in SPEC-0019). It also covers the common enemies' grab-and-throw machine (Behavior 23), their alarm-group check (Behavior 24), pursuit (Behavior 25), the part capsule sweep (Behavior 26) and the enemy status rules: the apply with its element immunity, and the tick that applies stun, poison, slow, speed, confusion and fire (Behavior 27). Monster footsteps shake the camera and rumble the controller by row class and distance (Behavior 32). Creature animation sounds, the Werelupe and Undead Draik cue sets and the shared hit flash are in Behavior 35. The Miamouse also has a catch-and-cage setup with a delivery count (Behavior 28), and Mortog's zone count rules are stated in one place (Behavior 3). The game has no fishing, racing, scoring or gambling mechanics. This is Medium-High, based on the absence of any such routine and host call in the shipped code.

An enemy shows the name text of its own spawn record, not a name from its monster row (Layout section 4c). Some spawn records are reserved for summons: they are never brought in by the proximity streaming of SPEC-0006 and appear only when a summoning attack or object spawns them by key at a point it supplies (Behavior 31). Enemies do not scale with the area: a placement's variant chooses the row, and for the minion families the variant chooses the element, not the strength (Scaling section). Some rules belong to every region character, whatever its class: the ghost cycle of rows with the Ghost Flag (Behavior 33) and the shared take-hit, death hook, corpse step and drops (Behavior 34).

All times in this spec are seconds unless marked ticks. Ticks = trunc(seconds x 60). Distances are world units. Distances used by the creatures in this spec are horizontal (x and z only). "Squared distance" comparisons avoid a square root and are stated as the squared threshold.

## Where it lives on the disc

- Behavior class table: 40 consecutive 32-bit words in the boot program's data at virtual address 0x312C10 (word index = class id). Entry 0 means no behavior. The words hold program addresses of the class routines. The clean side only needs to know which ids exist and what they run (Layout section 3); it must not copy the entries. Ids 0, 5 and 19 are empty; every other id from 1 to 39 has a routine.
- Enemy tuning constants are 32-bit words in the boot program's data (see the tables under Behavior). Each entry gives its virtual address. "int" is a signed 32-bit integer and "float" an IEEE single.
- Some constants are fixed constants rather than data words. They are marked "fixed constant" where they appear. The clean side may use the stated value but cannot read it from the disc.
- Per-monster parameter values: the monster table in GAMEFILE.DAT resource 530 (and resource 531), Layout section 4. The schema of Layout section 2 names its fields. The schema itself is not on the disc (Layout section 2).
- Animation indices below are entries of the enemy's own animation table (SPEC-0005).
- Spawn record fields read by this spec (kind-3 records, SPEC-0006; offsets from the record's data): the name text at +0x20 (display name, Layout section 4c), the s16 character kind key at +0xAC and the summon byte at +0xB6 (Behavior 31). The summon keys of a monster row are the u16 words at row offset 0x3CC + 2k (Layout section 4).
- Tuning words read from disc of the projectile, zone, timer and breath rules: listed in the table "Tuning words read from disc" below.
- Tuning words (boot program data; each taken from the disc image and read as the stated type): 0x2DB1D8 = 0.5 (f32); 0x2DAAB8 = 10.0 (f32); 0x2DD57C = 10.0, 0x2DD580 = 12.0, 0x2DD584 = 6.0, 0x2DD588 = 10.0 (f32; the two timers of the hero idle picker, SPEC-0033; see Behavior 12.9); 0x2DAA74 = 0.1 and 0x2DAA78 = 0.1 (f32; Draik flying approach, Behavior 12.12); 0x2DAAA8 = 0.5 (f32; per-tick step factor of the step homing, glide and homing hit routines, Behaviors 2e and 19); 0x2DAD54 = 40, 0x2DAD58 = 8, 0x2DAD90 = 15 (s32; the eight-slot shuffle, Behavior 12.5, roles open except that 8 is the slot count); 0x2DA628 = 50.0 (f32; the volley timer of Behavior 18 reads it, Open question 58). The word at 0x2DD54C (boot value -1, s32) is not a tuning word and not a target index: it is one run-time countdown of the common enemies' grab-and-throw machine (Behavior 23; SPEC-0038 Behavior 15), shared by all enemies, so the clean side must not take its value from the disc as a constant.
- Run-time words of the projectile pool (above the disc image, not read from the disc): the pool base pointer at 0x35C8D8, the gravity of the physics block at 0x3A1A7C (32.0, written at start-up) and the tick-rate word at 0x368FA0 (60). The clean side uses g = 32.0 and 60 ticks per second (Behavior 9). The controller slot arrays of Behavior 19 (0x36B640 and 0x36D3F0) are also run-time memory.
- Tick-rate words held at run time (above the disc image) are not read from the disc. Where a range is written as seconds, the clean side uses 60 ticks per second (SPEC-0007). The factors 0.5, 0.75, 2, 4, 1.5, 180, 600, 65, 40 and the others marked "fixed constant" are fixed constants, not data words. They are stated here as the program uses them; the clean side may use them but cannot read them from the disc, and they are not confirmed.

### Tuning words (read from disc)

Each word below is taken from the disc image at its virtual address (data words; file-backed) and read as the stated type. "s32" is a signed 32-bit integer and "f32" an IEEE single. Uses are as described in the behaviour sections named.

| Words (virtual addresses) | Type | Values | Used for |
| --- | --- | --- | --- |
| 0x2DB240, 0x2DB244, 0x2DB248, 0x2DB24C | f32 | 3.0, 1.5, 7.0, 5.0 | Live petpet follow gait thresholds: gait 0 to walk, gait 1 to idle, gait 1 to run, gait 2 back to walk (Behavior 12.13) |
| 0x2DB200, 0x2DB204 | s32 | -1, -1 | No-target sentinels, read by a controller that picks a point at distance 3.0 |
| 0x2DB250, 0x2DB268 | s32 | 1, -1 | 0x2DB250: mode flag of a close-test controller and the Whinny pen count (Behavior 5; Open question 69). 0x2DB268: Whinny entrance id (Behavior 5) |
| 0x2DA628, 0x2DA62C, 0x2DA640, 0x2DA644, 0x2DA648, 0x2DA64C | f32 | 50.0, 3.0, 15.0, 10.0, 5.0, 0.05 | Volley timer of 50 seconds; timer multiplier (3.0, times the tick rate gives ticks) and sub-state timer in seconds; hover offsets for sub-states 0, 1 and 2; maximum height change per tick, also the approach gap of Behavior 2e (Behavior 18; Open question 58) |
| 0x2DA9C8, 0x2DA9CC, 0x2DA9D8, 0x2DA9DC | f32 | 2.0, 5.0, 2.0, 5.0 | Fire breath range, low and high; ice breath range, low and high (Behavior 6.5; Open question 14) |
| 0x2DA9D4, 0x2DA9E4 | s32 | 10, 10 | Beam damage word and its second copy (Behavior 6.5; Open question 34) |
| 0x2DA9E8 | f32 | 1.0 | Breath beam length growth per tick (Behavior 6.5) |
| 0x2DA9A8 | s32 | 15 | Percent threshold of the roll in the Draik missile dodge (Behavior 10.4) |
| 0x2DA638, 0x2DA63C, 0x2DAAA8, 0x2DA634, 0x2DAA6C | f32, f32, f32, s32, f32 | 0.75, 3.0, 0.5, 10, 0.5 | Per-tick step factor of the controller slot homing steps and the damped homing step (0.75); damped homing timer in seconds (3.0); per-tick step factor of the step homing, glide and slot homing hit routines (0.5); orb damage of a slot homing detonation (10; not an effect id, since no effect definition 10 exists; SPEC-0054); slot sphere radius against the hero in the flight step (0.5) (Behaviors 2e and 19) |
| 0x2DD448, 0x2DD440 | f32 | 0.25, 1.0 | Homing blend weight of pool kind 17; area-mode pulse interval in seconds (times the tick rate gives 60 ticks) (Behavior 9) |
| 0x2DAAC8, 0x2DAACC, 0x2DAAD0 | s32 | 50, 60, 75 | Missile-dodge chance for one, two and three living sisters (Behavior 20) |
| 0x2DAAD4, 0x2DAAD8, 0x2DAADC | s32 | 40, 50, 60 | Melee-dodge chance for one, two and three living sisters (Behavior 20; label confidence Medium) |
| 0x2DAAE0, 0x2DAAE4 | f32 | 2.5, 5.0 | Energy-ball cooldown minimum and maximum, label 1 (SPEC-0011); the alternate loop overwrites them with 1.5 and 3.0 when it starts |
| 0x2DB124, 0x2DB128 | f32 | 10.0, 40.0 | Flame-wave distance band, lower and upper bound (Behavior 22) |
| 0x2DB12C | s32 | 10 | Flame-wave cooldown in seconds; the counter limit is this times 60 ticks (Behavior 22) |
| 0x2DB130, 0x2DB134, 0x2DB138 | s32 | 0, 50, 75 | Flame-wave chance for tiers 0, 1 and 2 (Behavior 22) |
| 0x2DCB20, 0x2DCAC4 | f32 | 1.0, 0.05 | Full-progress value for a timed blend; a threshold word whose owner is not established |
| 0x2DCAC8, 0x2DCADC, 0x2DCAE0 | f32, s32, s32 | 30.0, 1, 20 | Mote-eater timed group: length factor (steps = trunc(30.0 x tick rate) + 1 = 1801); drain enable; charge drained per entry (Behavior 9, kind 14) |

## Layout

### 1. Enemy runtime record (conceptual)

Each enemy instance has these fields (names are descriptive, not original):

| Field | Type | Meaning |
| --- | --- | --- |
| Behavior class id | int 0..39 | Which think routine runs. |
| Behavior priority | byte | Priority of the current class (see Behavior 1). |
| Think coroutine handle | handle | The running coroutine. |
| Target index | int, -1 = none | Index of the target in its region's character list. |
| Target triangle | int, -1 = none | Cached walk-mesh triangle of the target point. |
| Move target point | vec3 | Current point the enemy walks toward. |
| Arrival flag | bool | Set when the enemy is within 0.5 units of its move point. |
| Locomotion mode | int 0..10 (character word +0xA3C) | Selects the locomotion animation and the steering (Behavior 2). Mode 0 = walk and mode 2 = run for Mortog and Whinny; 7 to 10 are the flyer forms. |
| Hit points / max hit points | int | Current and maximum HP. |
| Catchable word | int | Miamouse only: 1 while it is idle or wandering and can be caught, 0 while it flees, lands or is caged (Behavior 28). |
| Goal latch | latch flag, goal point, current and last-valid triangle, last-valid point | The goal latch and its fallback (Behavior 2b). The rule that clears the latch is open. |
| Encounter latch | flag (0 or 1) | Set when a boss encounter starts for this enemy; see Behavior 8. |
| Spawn record (offset 0xA44) | pointer | The enemy's kind-3 spawn record (SPEC-0006), set when the enemy is set up from that record. Its data word at offset 0xB0 (signed 16-bit) is the drop placement ordinal, read at the first death tick (SPEC-0038, Behavior 34). -1 = no ordinal. |
| Monster rows (offsets 0xA48 and 0xA4C) | pointers | The character's resource-530 row (0xA48) and its resource-531 companion row (0xA4C), Layout section 4. When a region character is bound to its spawn record, the bind stores the record pointer (0xA44), the row-530 pointer, the hit points and the row-531 pointer (High). |
| Direct drop item (offset 0xA50) | integer, -1 = none | Item id that spawns directly when the enemy dies, or -1 to use the placement ordinal instead. Set to -1 at creation. Only the drop-item argument of a spawn by key sets it (Behavior 31). Two spawn-by-key routes pass item 172 (Nova Mote): the class-37 member summon and the Darkest Faerie volley, so every Minion Warrior and Minion Grunt she summons drops a Nova Mote at its first death tick (SPEC-0038 Behavior 34, part A). The other routes pass -1. After a direct spawn it is set back to -1. |
| Carried Neopoints (offset 0xC28) | integer, 0 at creation | Neopoints the enemy carries. Grows only in the item steal branch of an attack (Behavior 11b), by a per-item value. The Neopoint steal (Behavior 11a) does not change it. Split into coins at the first death tick (SPEC-0038, Behavior 34), then set to 0. |

The world holds a navmesh list: a count and an array of navmesh references. Each navmesh has a kind (1 ground, 2 flying), a count of triangles and the triangle list (three corners per triangle).

### 1a. Enemy thorn-projectile records (High for count, stride, purpose, and sister-group use; Medium-High for the static ownership boundary)

The program uses five records of 0xE0 bytes each in runtime memory (not on the disc). First use initializes all five records, and the time-change reset and first-free claim scans cover the same five-record range with the same starting offset. These are projectile records: the observed thorn-spawn path claims the first free record, places it, and initializes its damage to the Thorn Attack Damage value (10) and its maximum hit count to the Thorn Attack Max Hits value (3). The shared initialization, per-frame refresh, and deferred spawn helper are called by the three Faerie Sisters controller loops (classes 13, 15 and 16: Malice, Spite, and Vanity). The deferred helper is the only direct caller found for the first-free allocator, and no other enemy-controller call or address-construction path was found in the scanned boot program. The pool is therefore evidenced as shared by the sister group; the static census cannot exclude dynamically computed calls or runtime-loaded module code. See the ownership limits in Open question 98.

### 2. Per-class parameter schema (what the schema contains)

The monster parameter block is one record per monster type. The schema describes its fields as: a section label, a value kind, a key name, a byte offset in the block, and two clamp bounds (upper and lower). Value kinds:

- kind 0: 32-bit float
- kind 1: 32-bit signed integer
- kind 3: 16-bit signed integer

Clamp bounds are applied to a stored value; the sentinels 50000.0 and -50000.0 mean unbounded. A zero-initialized field with kind 0 and lower bound 0 needs no explicit store.

The full schema has 349 entries (block offsets 0x00 to 0x500) in one table and 27 entries (offsets 0x00 to 0x5C) in a second table that repeats a subset of keys with its own offsets. Keys by section (counts in the full table):

| Section | Count | Examples of keys, kind, clamp |
| --- | --- | --- |
| Basic | 15 | Hit Points (u16, offset 0x30, clamp 0..25000); Radius (float, 0x28, 0..50000); Height (float, 0x2C, 0..50000); Draw Distance (u16, 0x32, 0..2500) and Low Res Distance (u16, 0x34, 0..2500), both never read by any code (disc values in SPEC-0046, Designer properties); Weapon 1..5 Collision Length (float, offsets 0x00..0x10) and Radius (float, 0x14..0x24), upper clamp 40.0 (not a default). The names are swapped against their use: the "Length" word is the weapon radius and the "Radius" word its full height (Behavior 10.2) |
| Behavior | 3 | Default Behavior (u16, 0x36, -1..2500); Approach Routine (u16, 0x38, -1..2500); Difficulty Value (u16, 0x3A, -1..2500; the enemy's weight in the common-enemy attack-manager budget, SPEC-0038 Behavior 45; it does not scale the enemy's own numbers) |
| Monster Type | 21 | Alignment (u16, 0x3C, -1..20; the target type T, Layout 4); Hostile Flag (u16, 0x3E, 0..1); Socialize Flag (int, 0x40, 0..1); Spawn Number (u16, 0x7C, -1..1000); Ambush Invisible (u16, 0x7E, 0..1); Death Disappear Timer (u16, 0x80, up to 32767); Display in Default (u16, 0x82) |
| Movement | 17 | Ground Turn Speed (float, 0x84, 0..1); Flying Turn Speed (float, 0x88, 0..1); Can Fly (int, 0x94, up to 10); Ground Timer Min/Max; Distance Hide Flag, Distance and Chance; Max Fly Height; Footstep effects |
| Sense | 8 | View Angle; Awareness (int, 0xC4, 0..100); Awareness Timer (float, 0xC8, -1..50000); Sense Min/Max Chance and Range (floats, 0xCC..0xDC); Always Has Line of Sight |
| Defense | 14 | Missile Defend Time (int, 0xE0, up to 1000); melee block and dodge chances for Quick, Normal and Heavy hits (floats, 0xEC..0x100, up to 100.0); Defense Timer Min/Max (floats, 0x104 and 0x108, default 1000.0; the defence hold drawn when a common enemy's defence reaction ends, SPEC-0038 Behavior 44); Blocking Angle; Tough Hide; Damage Reduction (float, 0x114; the enemy defence, Layout 4) |
| Monster data | 21 | Weight (float, 0x118); Map Search Timer Min/Max (int, 0x128, 0x12C, up to 50000); Ghost Solid and Ghost timers (int, up to 500); Ghost Opacity Min/Max (float, 0..1); Reanimation; Anger Level; Greedy Hostile Range |
| Attack 1..4 | 240 | Each attack has the same 60 fields, stored field-major: each field has four slots, one per attack. Fields include Type (u16), Trigger Range Max/Min, Trigger Chance, Windup/Attack/Recovery Speed, Melee Arc and Range, Attack Delay, Charge type and distances, Missile flight and velocity fields, Damage Value, Knockback, Effect Chance, Spawn Entity Type and Count, Steal Neopoints fields, Projectile Type and Effects, Loop Timer Min/Max, Loop Stop Range |
| Agro Rush | 2 | Agro Rush Melee Distance Min; Agro Rush Distance Max |
| Agro Distance | 4 | Desired Distance Min/Max; Retreat Distance Min/Max |
| Agro Strafe | 4 | Desired Distance Min/Max; Strafe Erratic; Backbiter |

The second table repeats: Hit Points (u16), Radius, Height, Draw Distance, Low Res Distance, Default Behavior, Approach Routine, Emote index (u16, default 128), Socialize Flag (u16), Socialize distance and timer fields, Search Timer Min/Max, Ground Turn Speed, Ground Timer Min/Max, View Angle, Awareness (int), Awareness Timer, and Sense Min/Max Chance and Range. Which object owns this second block is not known.

Field meanings and units are given in Layout section 4 (the table after the row fields). Chance values are percents; the timer values are ticks, not seconds, in every routine that reads them (Layout section 4, Units).

#### 2a. Schema record layout (in the program's memory, built at start-up)

The schema is not stored on disc. At each start the program writes its records from fixed definitions into memory. There are 376 records of 24 bytes each: 349 in the main table and 27 in the second table. Each table is announced by a two-word descriptor (entry count, then the base of the record array). Each record is zero-initialized before its fields are set.

| Offset in record | Size | Field | Meaning |
| --- | --- | --- | --- |
| 0x00 | 4 | Section label | Text of the section name (Section column of the table above). Stored as a pointer to program text. |
| 0x04 | 4 | Kind | 0 = float, 1 = signed 32-bit integer, 3 = signed 16-bit integer. |
| 0x08 | 4 | Key name | Text of the key name. Stored as a pointer to program text. |
| 0x0C | 4 | Byte offset | Offset of the value in the monster block. |
| 0x10 | 4 | Upper clamp | Upper bound; 50000 (50000.0 as float) means unbounded. |
| 0x14 | 4 | Lower clamp | Lower bound; -50000.0 means unbounded. |

The first record is Hit Points: kind 3, byte offset 0x30, upper clamp 25000, lower clamp 0. Confidence: High for the record count, 24-byte size and field order (the builder writes each field at its stated offset); Medium-High for the exact clamp interpretation on integer kinds. A clean implementation takes the key names, kinds, offsets and clamps from the tables in this section; it does not need the in-memory record form.

Schema readers (High). Only two routines read the schema, and neither has a caller or a stored address: they are a debug tuning editor that never runs in retail. A clean implementation needs neither.
- An editor-row builder makes one row per schema record: kind 0 a float slider with step 0.1, kinds 1 and 2 integer sliders with step 1 (kind 2 read unsigned), kind 4 a widget with no range, and any other kind a 16-bit signed slider, each with the record's two clamp bounds.
- A value printer writes a field's value as text: kind 0 as the float with a leading minus sign when negative, the whole part, a dot and 6 fraction digits, each truncated (no rounding); kind 1 as a signed decimal; kinds 2, 4 and 5 as unsigned decimals; any other kind as the 16-bit field in decimal. Example: kind 0 with value -2.75 prints -2.750000.

### 3. Behavior class ids

Each id names the think routine the enemy runs. "Spec" is the spec that describes the routine; "not specified" means the routine exists but is not described in this spec or any other spec yet.

| Id | Role | Spec | Confidence |
| --- | --- | --- | --- |
| 1 | Gelert Assassin | SPEC-0013 | Medium-High |
| 2 | Plague Serpent nest controller (main loop) | SPEC-0016, SPEC-0017 | High |
| 3 | Spyder Nest controller (the nest main loop; on monster row 12 only) | SPEC-0017 | High |
| 4 | Spyder Nest valve: yields every tick and does nothing; on rows 13 to 15 only (field 0x38 is the valve index: row 14 valve 0, row 13 valve 1, row 15 valve 2), which no record places | SPEC-0017; this section | High |
| 6 | townsfolk (the default class of every placement whose monster row has no class; 14-value mode controller; see the text below the table) | SPEC-0056 (modes); Layout 3 text below; its group leader handover is SPEC-0038 Behavior 10b | High |
| 7 | Whinny | Behavior 5; also SPEC-0038 Behaviors 16, 29 and 30 (chase-to-point controller, approach and attack bands, wait-and-approach step) | High |
| 8 | Black Knight | SPEC-0012 | Medium-High |
| 9 | Cloud Beast (not placed on the disc: no request gives class 9; see SPEC-0010, Placement and reachability) | SPEC-0010 | High |
| 10 | Cloud Beast companion (the partner entity it links to; not placed: monster row 2 has no spawn record) | SPEC-0010 | Medium (role); High (not placed) |
| 11 | Ixi Chief | SPEC-0014 | High |
| 12 | Draik Skeleton | Behavior 6 | High |
| 13 | Faerie Sisters controller (main routine) | Behavior 4; SPEC-0011 | High (id) |
| 14 | Gnarfas | SPEC-0020 | High |
| 15 | Faerie Sisters controller (second routine) | Behavior 4; SPEC-0011 | High (id) |
| 16 | Faerie Sisters controller (third routine) | Behavior 4; SPEC-0011 | High (id) |
| 17 | common enemies | SPEC-0038 | High |
| 18 | Juppie boss controller | SPEC-0015 | Medium |
| 20 | second petpet behaviour (hero reaction, approach and wander) | SPEC-0047 | Medium |
| 21 | miamouse (deals no damage) | Behavior 28; its chase-and-retreat state is also written out as SPEC-0038 Behavior 17 | Medium-High |
| 22 | plays its idle pick every tick; no requester | this section | High |
| 27 | ends at once; no requester | this section | High |
| 28 | plays the animation of its script-animation record, waits for progress 0.99, plays its idle pick, ends; no requester | this section | High |
| 23, 24 | level-script walk to a locator | SPEC-0056; SPEC-0030 | High |
| 25, 26 | level-script walk to a locator, then turn to the locator's facing | SPEC-0056; SPEC-0030 | High |
| 29 | patrol list walk; no requester and no monster row has this class, so it never runs on the disc | SPEC-0038 Behavior 32 | High (owner and no requester); Medium-Low (steps) |
| 30 | stub: writes a debug line, plays its idle pick, ends; no requester | this section | High |
| 31 | moves to the point and facing of its script record, plays its idle pick, ends; no requester | this section | High |
| 32 | Werelupe King | SPEC-0019 | High |
| 33 | placed petpet characters (the default class of monster rows 244, 245, 247, 250 and 251) | SPEC-0047 | High (default); Medium (states) |
| 34 | live (summoned) petpet controller: main loop with a state selector, states 0 to 6 | SPEC-0047; SPEC-0038 Behaviors 27 and 33; Behaviors 12.8 and 12.13 | High (role, owner and steps) |
| 35 | level-script path walk (0.5 units per point) | SPEC-0056; also written out as SPEC-0038 Behavior 31 | High (role and owner); Medium-Low (steps) |
| 36 | Target Dummy | Behavior 29; the same idle routine is SPEC-0038 Behavior 28 | High (control); Medium (animation meanings) |
| 37 | Darkest Faerie family, three-way entry dispatcher (four-member controller and two alternate loops) | Behavior 7 (the fight is in its own spec) | Medium |
| 38 | Queen Fyora controller | Behavior 30 | Medium |
| 39 | Mortog | Behavior 3 | High |
| 0, 5, 19 | empty (no behavior) | - | High |

Classes 3 and 4 (Spyder Nest, SPEC-0017; High): class 3 is the nest controller, and the only row that names it is row 12. Class 4 is the valve, a routine that yields every tick and changes nothing; the valve behaviour itself (spider reveal, valve HP, destroy) is run by the controller (SPEC-0017). Neither routine is reached by any other path, and no record on the disc places rows 12 to 15, so in play neither class runs. Test vector: a character on row 14 (class 4, valve index 0) gets no change from its class routine over any number of class ticks.

Class 6 is the townsfolk class (High). A region character runs class 6 when its default class is 6: the monster row's default class (resource 530 +0x36) is none (negative), and the companion row (resource 531 +0x10) gives 6. The default-class request also sets the character's townsfolk flag, which is 1 exactly when the default class is 6. On disc 666 of the 3,036 kind-3 placements resolve to class 6, under 280 distinct names (Layout section 4a). Its controller has 14 mode values, 0 to 13; the full rules are in SPEC-0056:

- 0: the mode becomes the placement's behaviour code (kind-3 record data +0x84, replaced by SetAI_Behavior) at the start of the next class tick; a code-0 character stays in mode 0 and idles.
- 3: villager pair (the roles alternate, then both return to the code with a 10 to 30 second cooldown). The pair search needs a horizontal distance below the row's word +0x44, which is 0.0 on every class-6 row, so no pair forms on disc.
- 5 and 6: wander around the home point and patrol a world path, with pauses.
- 10, 11 and 12: hold fixed poses.
- 13: greeting pause, entered when the hero is closer than 6.0 horizontally and in the front half; it ends when the hero is more than 12.0 away or behind (dot below -0.1).
- 1, 2, 4, 7, 8 and 9: nothing happens; the character keeps its playing animation.

Routines of other classes in SPEC-0038: SPEC-0038 keeps, under its heading Routines of other classes, the routines of classes 6, 7, 21, 29, 34, 35 and 36 listed in the table. The owners come from the class routine table (High). A class-17 enemy does not run them.

Ids 40 and above are errors (Behavior 1).

Petpets (ids 34, 33 and 20): the petpet the player summons runs as class 34 (requested at priority 2 when it is created); class 33 is the default class of the placed pet rows; a second petpet type runs as class 20. Their state rules, hit values and score are in SPEC-0047 (Petpets). A petpet strikes objects whose type code is 146 to 157 and never the anchor type 145. The placed pets use monster rows 244, 245, 247, 250 and 251 of Layout 4; whether the summoned petpet reads a monster row is open (SPEC-0047 open question 14).

### 3a. Class inventory on disc (High)

The classes reached on the disc are the default classes of placed rows (1, 2, 6, 7, 8, 11 to 18, 20, 21, 32, 33, 36 to 39) and the six explicit classes 23 to 26, 34 and 35 (Behavior 1, step 4). Classes 3, 4, 9 and 10 sit only on rows that no record places. Classes 22 and 27 to 31 are named by no row, no companion row and no request, so they never run. A routine that ends keeps its class id and priority; nothing on the disc depends on that.

Counts over the kind-3 records on disc. "Region" counts records in character region resources, "embedded" counts records inside world packs; "reserved" counts summon-reserved records (Behavior 31).

| Class | Rows | Placed (region / embedded) | Reserved | Owning spec |
| --- | --- | --- | --- | --- |
| 1 | 1 | 7 (2 / 5) | 0 | SPEC-0013 |
| 2 | 1 | 1 (1 / 0) | 0 | SPEC-0016 |
| 3 | 1 | 0 | 0 | SPEC-0017 |
| 4 | 3 | 0 | 0 | this spec (Layout section 3) |
| 6 | 119 | 666 (617 / 49), all through the companion default | 0 | SPEC-0056 |
| 7 | 1 | 23 (23 / 0) | 0 | Behavior 5 |
| 8 | 1 | 2 (1 / 1) | 0 | SPEC-0012 |
| 9, 10 | 1 each | 0 | 0 | SPEC-0010 |
| 11 | 1 | 4 (3 / 1) | 0 | SPEC-0014 |
| 12 | 1 | 2 (1 / 1) | 0 | Behavior 6 |
| 13, 15, 16 | 1 each | 3 each (3 / 0) | 0 | SPEC-0011 |
| 14 | 1 | 2 (1 / 1) | 0 | SPEC-0020 |
| 17 | 109 | 2,197 (1,654 / 543) | 208 | SPEC-0038 |
| 18 | 2 | 3 (3 / 0) | 0 | SPEC-0015 |
| 20 | 1 | 7 (2 / 5) | 0 | SPEC-0047 |
| 21 | 1 | 21 (21 / 0) | 0 | Behavior 28 |
| 23 to 26, 35 | 0 | explicit only: 76, 67 and 34 distinct call sites of WalkEntity, WalkEntityAndOrient and WalkEntityAlongPath, in 38, 34 and 30 modules | - | SPEC-0056 |
| 32 | 1 | 3 (1 / 2) | 0 | SPEC-0019 |
| 33 | 5 | 49 (34 / 15) | 0 | SPEC-0047 |
| 34 | 0 | explicit only (petpet creation) | - | SPEC-0047 |
| 36 | 1 | 7 (7 / 0) | 0 | Behavior 29 |
| 37 | 2 | 6 (4 / 2) | 0 | Behavior 7, SPEC-0054 |
| 38 | 1 | 4 (3 / 1) | 0 | Behavior 30 |
| 39 | 1 | 23 (15 / 8) | 0 | Behavior 3 |

### 4. Monster table (resources 530 and 531)

Both resources share one header: u32 *N* = 112 (monster types) at offset 0, then *N* u32 values *first[t]* at offset 4 (the first row of type *t*). Rows start at offset 0x1D0 (4 + 4N rounded up to 16). Resource 530 rows are 0x504 bytes each (259 whole rows); resource 531 rows are 0x60 bytes.

**Lookup**: a spawn record (kind 3, SPEC-0006) carries the monster type *t* as a u16 at data + 0xB4 and a variant *v* as a u16 at data + 0xB2. Row = first[t] + v, valid only when t < 112. The same row index selects the companion row in resource 531. Read from the disc over all 3,036 spawn records on the disc (including those inside embedded regions): every type is in range, and the type is constant per character name for 350 of 353 names.

**Placement coverage**: 222 of the 259 monster rows are the selected row of at least one spawn record on disc. Of those, 79 are placed only by records embedded in world packs that load in play, 129 only by records in character regions that a level module loads with a SlotCreate or SlotLoad call, 13 only in world packs that do not load in play, and one (row 179) only in a region that no level module loads. The remaining 37 rows are selected by no record. Confidence: High for the 37; Medium for the split between packs that load in play and packs that do not.

Core fields of a resource-530 row (offsets in the row; names as the schema of Layout section 2 calls them):

| Offset | Type | Field |
| --- | --- | --- |
| 0x00 + 4(N-1) | f32 | Weapon N radius, N = 1 to 5 (schema name "Weapon N Collision Length"; Behavior 10.2) |
| 0x14 + 4(N-1) | f32 | Weapon N full height, N = 1 to 5 (schema name "Weapon N Collision Radius"; Behavior 10.2). On disc 138 of the 259 rows have all ten weapon words 0 |
| 0x28 | f32 | Radius: the body radius of a region character. Placement copies it into the motion record's radius (+0x08), which the character contacts of SPEC-0018 use (section 4d) |
| 0x2C | f32 | Height: the body height, copied into the motion record's height (+0x0C) at placement (section 4d) |
| 0x30 | u16 | Hit points (starting and maximum health); positive in all 259 rows |
| 0x36 | u16 | Default behaviour class (Layout section 3); 0xFFFF = none (negative when read as signed). When none, the companion row's field at +0x10 (resource 531, read as signed) gives the default instead: it is 6 on 258 of the 272 companion rows, 0 on 12 and -1 (none) on 2, and never 9 (High). Class 6 is the townsfolk class (Layout section 3; High). Row 1 is the only row with class 9 (240 hit points), and no spawn record on the disc selects row 1 (High) |
| 0x38 | u16 | Approach routine (1, 3 or 4 on disc: rush, keep distance and strafe of the common enemies' agro machine, SPEC-0038 Behavior 44) |
| 0x3C | u16 | Target type T. Values 0 to 7 on disc: 0 = none (non-monster characters also report 0); 1 to 6 = the Mote element classes, paired 1 with 4, 2 with 5 and 3 with 6 by the match rule of SPEC-0036; 7 = matched only by Nova. Schema range -1 to 20. Counts in section 4b. |
| 0x3E | u16 | Hostile flag |
| 0x5C, 0x60 | f32 | Search timer max, min |
| 0x64, 0x68 | f32 | Attack timer max, min |
| 0x6C, 0x70 | f32 | Flee timer max, min |
| 0x74, 0x78 | f32 | Combat timer max, min |
| 0x80 | u16 | Death disappear timer |
| 0x84 | f32 | Ground turn speed |
| 0x94 | s32 | Can fly: 1 = flying while the ground countdown is at or below 0; 2 = always flying; 0 = never. All 42 rows with a nonzero value (26 with value 1 and 16 with value 2) are class 17; 36 of them are placed, by 642 records, so no character of another class ever flies (High; Behavior 34) |
| 0xB8 | s16 | Footstep camera shake class: 1, 2, 3 give strength 0.5, 1.0, 1.5; 0 = none (Behavior 32) |
| 0xBA | s16 | Footstep controller rumble class: 1, 2, 3 give magnitude 0.5, 0.75, 1.0; other values 0 (Behavior 32; fades to 0 at 30 units) |
| 0xC0 | f32 | View angle |
| 0xC4, 0xC8 | s32, f32 | Awareness, awareness timer |
| 0xCC, 0xD0 | f32 | Sense chance max, min |
| 0xD4, 0xD8 | f32 | Sense range max, min |
| 0x114 | f32 | Enemy defence (schema name Damage Reduction). Nonzero in four rows: 10.0, 10.0, 2.0 and 5.0. Read by the enemy class getter. The melee and shot damage formulas (SPEC-0036) subtract it from the product of the attack factors, before the status scale. |
| 0x118 | f32 | Weight (schema name Weight): copied into the motion record's weight (+0x04) at placement, the divisor of the velocity in the motion step (SPEC-0040; section 4d) |
| 0x138 to 0x158 | s32, f32 | Ghost words: Ghost Flag (+0x138), solid phase min/max (+0x13C/+0x140), ghost phase min/max (+0x144/+0x148), opacity min/max (+0x14C/+0x150, f32), reveal min/max (+0x154/+0x158) (Behavior 33) |
| 0x16C + 2k | u16 | Attack k type (k = 0 to 3; 0 = melee, 1 = missile) |
| 0x17C / 0x18C + 4k | f32 | Attack k trigger range max / min |
| 0x19C + 4k | f32 | Attack k trigger chance |
| 0x1BC / 0x1CC / 0x1DC + 4k | f32 | Attack k wind-up / attack / recovery speed |
| 0x1EC / 0x1FC + 4k | f32 | Attack k melee arc / melee range |
| 0x20C / 0x21C + 4k | f32 | Attack k delay max / min |
| 0x2BC + 4k | s32 | Attack k flight mode: 0 straight aim (kind 0), 1 speed ramp with a ballistic arc (kind 1), 2 homing (kind 17) (Behavior 13). On disc: 0 in 844 used attack slots, 1 in 9, 2 in 1 |
| 0x2CC / 0x2FC + 4k | f32 | Attack k missile launch speed (units per tick) / range (units) |
| 0x2DC + 4k | f32 | Attack k end speed of the speed ramp (flight mode 1, units per tick); values 0 to 1 on disc |
| 0x2EC + 4k | f32 | Attack k projectile radius (Behavior 13) |
| 0x30C + 4k | f32 | Attack k aim inaccuracy, units (Behavior 13) |
| 0x31C / 0x32C + 4k | f32 | Attack k projectile spin rate (radians per tick) / roll (Behavior 13) |
| 0x36C / 0x37C / 0x38C / 0x39C + 4k | s32, f32, f32, f32 | Attack k projectile status id, status chance (percent), status value range min / max in ticks (the stored value is the drawn ticks / 60) (Behavior 13) |
| 0x3AC + 4k | u32 | Attack k flags: 0x02 marks a summoning slot: on a missile slot it selects the lobbed summon (Behavior 13), and in the class-17 attack choice it limits the slot by the living summon count (SPEC-0038); both readings hold. 0x10 sets the projectile's ignore-world flag. In the class-17 melee strike 0x01 makes the strike unblockable, 0x20 forces the hit to count as blocked, and 0x08 (grab-and-throw) has no weapon-volume path (Behavior 10.2; SPEC-0038 Behaviors 13 and 15). 0x40 is read by the attack choice (SPEC-0038). Bit 0x02 is set on 3 of 854 used slots (row 128 slot 0, row 211 slot 0, row 221 slot 3) |
| 0x3BC + 4(k-1) | s32 | Damage alignment of attack k, k = 1 to 4. Values 0 to 7 on disc (slot 3 never reaches 7; schema range -1 to 20). In slot 1 it equals row T in 249 of 259 rows. |
| 0x33C + 4k | s32 | Attack k missile count (spawn count of the sequencer, SPEC-0038 Behavior 13); values 0 to 10 on disc, 1 on 813 of 854 used slots |
| 0x34C / 0x35C + 4k | f32 | Attack k damage / knockback K. K is the speed of the knockback a class-17 strike gives the hero (Behavior 10.1a). On disc K is non-zero in 130 slots of 75 rows, with values 1 to 17 (1, 2, 3, 4, 5, 6, 7, 10, 12, 14, 15, 17) |
| 0x43C + 4k | s32 | Attack k projectile type: the visual kind of a missile slot's projectile (Behavior 9) |
| 0x44C / 0x45C + 4k | f32 | Attack k projectile area duration (ticks) / area radius (units) (Behavior 9, area mode) |
| 0x46C + 2k + 8j | u16 | Attack k projectile effect id j (j = 0 to 5) (Behavior 13) |
| 0x22C + 4k | s32 | Counter Attack Trigger code of the class-17 counter-attack choice (SPEC-0038 Behavior 14): -1 on 644 of the 854 used attack slots on disc, 0 on 98, and 1 to 6 on 112 (6 on 50 slots, 5 on 32, 2 on 20, 3 on 3, 4 on 1, 1 on 6). Codes 0 and -1 both never trigger. |
| 0x23C + 4k | f32 | Counter Attack Chance, a percent from 0 to 100 (the same choice). Zero on 726 of the 854 used slots; nonzero on 128, of which 112 also have a trigger code of 1 to 6. |
| 0x4DC to 0x4FC | 9 x f32 | Agro distances: rush melee min, rush max, desired min/max, retreat min/max, strafe desired min/max, erratic (0x4FC; never read by class 17) |
| 0x3AC + 4k, bit 0x04 | u32 bit | Attack k enables the steal branch (Behavior 11a). Set on the Thief and Thug rows and on the Crokabek rows. |
| 0x3DC + 4k | s32 | Attack k steal amount, minimum (Neopoints) |
| 0x3EC + 4k | s32 | Attack k steal amount, maximum (Neopoints) |
| 0x3FC + 4k | u32 | Attack k steal selector: 0 = Neopoint steal (Behavior 11a); 1 to 3 = item steal branch (Behavior 11b); other values pick no item and have no effect |
| 0x40C + 4k | f32 | Attack k steal chance, in percent |
| 0x3CC + 2k | u16 | Attack k spawn key (k = 0 to 3): the character kind key that a summoning attack of slot k spawns by key (Behavior 31). 0xFFFF (and 0) = none. On disc only five slots hold a key: row 128 (k = 0: 2864, Minion Warrior; k = 1: 2844, Minion Grunt), row 211 (k = 0: 2844), row 215 (k = 0: 2807, Minion Archer) and row 221 (k = 3: 619, Flower Monster) |
| 0x3D4 + 2k | u16 | Attack k spawn entity count: the most live summons the enemy may have from slot k (SPEC-0038 Layout 2). 4 on the summoning slots of rows 128 and 211; 2 on row 215's slot 0; 5 on row 221's slot 3 |

**Units (High for every field that code reads).** The row's timer fields are in ticks (60 per second), not seconds. The float ranges (talk and quiet countdowns, search, attack recovery, timed approach, combat, ground retarget, defence hold, slot cooldown and the stun range) are drawn between their two ends and cut to whole ticks; the awareness timer is a cycle length in ticks; the integer fields (missile defend time, map search timers, ghost phases and reveals, reanimation range, death disappear timer) are whole ticks. The attack wind-up, attack and recovery speed words are the dimensionless clip-rate factor P of SPEC-0040 (1.0 is normal speed, 0.5 half speed). Missile velocity, end speed and the throw speed are units per tick; ranges, radii and distances are world units; chances are percents; the two turn speeds are fractions from 0 to 1 used as per-tick blend weights. The reading of each field is in SPEC-0038, Layout section 2. Read every value from the disc.

**Resource 530 after the last row (Medium).** The resource is 333,824 bytes: the header of 0x1D0 bytes, 259 whole rows and 804 further bytes. The 804 bytes are a partial row's worth of high-entropy data that no code reads: the largest row index any spawn record reaches is 258, and the lookup rule of this section never goes beyond the 259th row. A clean build ignores them.

**Resource 531, the companion rows (High for the fields code reads; the rest are zero).** The 272 rows of 0x60 bytes follow the same header; the same row index selects the row. After the 272 rows there are 48 further bytes (three zero words and nine words equal to 1.0) that no code reads. Fields: +0x00 u16 hit points (50 on 260 rows), +0x04 f32 radius (1.175), +0x08 f32 height (3.5), +0x0C u16 draw distance (300) and +0x0E u16 low-resolution distance (200), both never read; +0x10 s16 default behaviour class (6 on 258 rows, 0 on 12, none on 2; read by the spawn when the 530 row has none); +0x12 u16 approach routine (none on 260 rows); +0x14 s16 emote group (none, -1, on 156 rows; 0 to 15 on the other 116; the group index into the emote table of 74 entries per group that a talking character uses; Medium); +0x16 u16 socialize flag (1 on 259 rows, read by the pair test of SPEC-0040, Behavior 3B); +0x18 f32 socialize distance (0 on 268 rows and 20.0 on rows 255 to 258; read by the same test); the other bytes up to +0x5F are zero on the first 255 rows and are not read, apart from a float 120.0 at +0x4C on 259 rows (never read). Rows 259 to 271 are 13 extra rows with other values: no spawn record can select them (the highest row any type and variant reach is 258), so a clean build may drop them.
Test vector: spawn type 109 selects row 253 of resource 530 and row 253 of resource 531; a townsfolk spawn whose 530 row has default class 0xFFFF reads +0x10 of its 531 row and starts as class 6 (townsfolk) when that is 6.

Class usage on this disc: 109 rows use class 17 and 119 rows have no class. Classes 3, 4, 9 and 10 occur only on unused rows (rows 12, 13 to 15, 1 and 2). Rows 1 and 2 (monster types 1 and 2) are the Cloud Beast (class 9, 240 hit points) and its companion (class 10, 50 hit points); no spawn record on the disc has type 1 or 2 (High; SPEC-0010). Classes 1, 8, 12, 14, 18, 20, 21, 33, 36, 37, 38 and 39 each occur on at least one row that is placed in play. All common enemies named in spawn records (skeleton fighters, archers and wizards, spyders, slorgs, werelupe bashers, spirits, tentacles, sludgies, minions, spirit swords) use class 17. They differ only by row values: attack types and ranges, timers, flying, approach routine. Bosses and special creatures use their own classes, matching Layout section 3.

### 4a. Placed-record census by class (data facts)

Counts of the 3,036 spawn records of kind 3 (SPEC-0006) on the disc, grouped by the behaviour class of the monster row each record uses (row field 0x36). A record counts under "none" when its row has no class (value 0xFFFF).

| Class | Placed records | Notes |
| --- | --- | --- |
| 17 (common enemies) | 2,197 | SPEC-0038 |
| none (resolves to 6, townsfolk) | 666 | 543 with a name, 123 unnamed (280 distinct names). Every one resolves to class 6 (townsfolk) through its companion row (resource 531 +0x10) |
| 33 (PetPet) | 49 | SPEC-0047 |
| 39 (Mortog) | 23 | Behavior 3 |
| 7 (Whinny) | 23 | Behavior 5 |
| 21 (miamouse) | 21 | Behavior 28; all unnamed |
| 36 (Target Dummy) | 7 | Behavior 29 |
| 20 (second petpet) | 7 | SPEC-0047 |
| 1 (Gelert Assassin) | 7 | SPEC-0013 |
| 37 (Darkest Faerie family) | 6 | Behavior 7 |
| 11 (Ixi Chief) | 4 | SPEC-0014 |
| 38 (Queen Fyora controller) | 4 | Behavior 30 |
| 32 (Werelupe King) | 3 | SPEC-0019 |
| 16, 15, 13 (Faerie Sisters) | 3 each | SPEC-0011 |
| 18 (Juppie boss) | 3 | SPEC-0015 |
| 14 (Gnarfas) | 2 | SPEC-0020 |
| 8 (Black Knight) | 2 | SPEC-0012 |
| 12 (Draik Skeleton) | 2 | Behavior 6 |
| 2 (Plague Serpent) | 1 | SPEC-0016 |

The rows sum to 3,036. Sixteen of the 112 monster types have no placed record.

Further counts over the same records (High):

- 2,467 records use a hostile or classed row (enemies and creatures). Grouped by display name (Layout section 4c), the packs reached in play (SPEC-0006) hold 70 enemy kinds, besides the non-damaging creatures (Whinny, Mortog, Miamouse, wild PetPets).
- Drop ordinal outcome (SPEC-0038 Behavior 34, step 6) over the hostile and classed records: no ordinal on 1,100; a drop group on 913; a direct item on 190; fixed coins on 67; an empty placement on 64.
- 208 records are summon-reserved (summon byte nonzero, Behavior 31). By key: Flower Monster 90, Minion Grunt 64 (60 named, 4 with an empty name), Minion Warrior 16, Sludgy 7, Spyder 7, Dumb Skeleton 6, Minion Archer 6, Restless Spirit 4, Skeleton Fighter 2, Poison Spyder 1, Giant Spyder 1, and 4 unnamed records of one more key.
- 281 records have an empty name.

### 4b. Target type T: counts and likely labels (counts High; labels Medium-Low)

Counts of the target type T (row offset 0x3C) over the 259 monster rows, and over the 3,036 placed enemy records (each placed record counts under the T of its row):

| T | Monster rows | Placed enemies | Likely label (Medium-Low) | Example enemies |
| --- | --- | --- | --- | --- |
| 0 | 163 | 1,201 | no type | (many types, including the T = 0 variant rows of some families) |
| 1 | 13 | 444 | fire-type | Drackonack, Giant Drackonack |
| 2 | 20 | 534 | nature | Poison Spyder, Flower Monster |
| 3 | 10 | 104 | light | Evil Harris |
| 4 | 14 | 230 | water | Slorg, Swamp Tentacle |
| 5 | 13 | 215 | air or fog | Crokabek, Skull Spirit |
| 6 | 25 | 307 | dark | Death Knight, Restless Spirit |
| 7 | 1 | 1 | matched only by Nova | Minion Warrior (one variant) |

- T is a property of the monster row. Within one family the variant rows can carry different values; the Minion Grunt family, for example, has rows with T = 1 to 6. The element of an enemy is therefore a row property, not a placement property.
- The labels are inferred from the family names. The data does not state whether a class is weak to or resistant to its opposite Mote class.
- The match rule that uses T, with the Mote kinds, is in SPEC-0036.

### 4c. Display name of an enemy (High)

- The name an enemy shows is the name text of its own spawn record (kind-3 record data +0x20, ASCII, NUL-terminated). The longest name on disc is 29 characters. When the enemy is hit, the monster health bar copies the name (up to 31 characters) together with the health values (SPEC-0023 section 10).
- Monster rows carry no name. One row can therefore appear under several names: for example row 152 is placed as Thief, Drub, Gil the Fence and Drool, row 153 as Rogue, Darrick the Blade and Red Jack, row 151 as Bandit, Sid the Vicious and Lump, and row 154 as Thug, Thud and Korvin the Krusher.
- A record with an empty name (281 on disc) shows no name panel when hit.
- Names carry no digits or counters; two enemies placed with the same name show the same text.

### 4d. Body sizes of region characters (High)

A region character (townsperson, named character, enemy or boss) gets its body size from its monster row (row = first[t] + v, section 4) when it is placed: weight = row +0x118, radius = row +0x28, height = row +0x2C. The placement writes them once into the character's motion record (+0x04 weight, +0x08 radius, +0x0C height; SPEC-0040), and later re-placements keep them. The heroes do not use a row: both have weight 5.5, radius 1.175 and height 3.5 (SPEC-0040). The radius and height are the reach and body height of the character contacts in SPEC-0018 (the box push, the class-0 cylinder test, the navmesh confinement of SPEC-0056).

On disc the 259 rows hold 42 distinct (radius, height, weight) triples; 143 rows hold (1.175, 3.5, 5.5), the hero values. Examples (radius, height, weight), by the names the rows are placed under:

| Character | Radius | Height | Weight |
| --- | --- | --- | --- |
| Townsfolk and named NPCs; Werelupe Basher, Bone Thrower and Elite; Slorg; Plague Serpent, Gelert Assassin, Darkest Faerie | 1.175 | 3.5 | 5.5 |
| Dumb Skeleton, Skeleton Fighter | 2.0 | 3.5 | 5.5 |
| Skeleton Archer, Skeleton Wizard | 2.0 | 2.5 | 5.5 |
| Spyder, Poison Spyder | 1.0 | 1.0 | 2.0 |
| Giant Spyder | 1.5 | 2.0 | 5.5 |
| Minion Grunt | 2.0 | 2.0 | 5.5 |
| Minion Archer | 2.0 | 3.5 | 5.5 |
| Minion Warrior | 1.25 | 3.0 | 5.5 |
| Minion Wizard | 1.75 | 6.0 | 5.5 |
| Minion Brute | 4.0 | 7.0 | 5.5 |
| Minion Magician | 2.5 | 5.0 | 2.0 |
| Minion Artillery | 2.0 | 4.0 | 5.5 |
| Giant Slorg | 3.0 | 5.0 | 5.5 |
| Drackonack | 1.5 | 2.0 | 5.5 |
| Giant Drackonack | 3.5 | 4.5 | 5.5 |
| Crokabek | 1.0 | 1.5 | 1.0 |
| Dark Knight | 3.0 | 5.0 | 3.0 |
| Death Knight | 1.5 | 5.0 | 5.5 |
| Swamp Tentacle | 2.0 | 4.0 | 5.5 |
| Flower Monster | 1.0 | 5.0 | 2.0 |
| Restless Spirit | 1.5 | 3.0 | 2.0 |
| Sludgy | 2.0 | 4.0 | 3.0 |
| Ixi Spearman | 2.0 | 4.0 | 5.5 |
| Ixi Archer | 1.5 | 3.5 | 5.5 |
| Ixi Chieftan | 2.0 | 5.0 | 5.5 |
| Black Knight | 1.5 | 5.5 | 5.5 |
| Werelupe King | 3.0 | 6.0 | 5.5 |
| Gnarfas | 2.5 | 7.0 | 5.5 |
| Juppie Monster | 3.0 | 10.0 | 20.0 |

Test vector: a Minion Brute placed from its row has motion-record weight 5.5, radius 4.0 and height 7.0, so its contacts use a reach of 4.0 and a body height of 7.0; a Crokabek has 1.0, 1.0 and 1.5; a townsperson on a (1.175, 3.5, 5.5) row gets exactly the hero values.

## Behavior

### Scaling of enemy numbers (High for the search; Medium-High for coverage)

The game has no player-selected difficulty setting: no option, save field or game rule selects enemy tuning from a player choice. Enemy row values do not scale with the act or chapter, the area, the hero's level (there is none; SPEC-0036), a death count (none is kept), or the number of heroes in the party. No enemy routine reads party size. The monster-row Difficulty Value (0x3A) does not scale an enemy's own stats; the shared attack manager uses it as that enemy's weight in the per-tick attacker budget (SPEC-0038, Behavior 45).

The scaling that does exist is local to one placement, one enemy family or one fight:

1. Row variant: a spawn record's variant selects a different monster row of its type (SPEC-0024). This is the only per-placement choice. For the minion families (Minion Grunt, Archer, Warrior, Brute, Wizard, Magician and Artillery; Grunt and Archer share monster type 80, the Grunt on rows 0 to 6 of the type and the Archer on rows 7 to 13) the variant rows of one family share hit points and damage values and differ in element (target type T and the damage alignments), in projectile and effect fields and, for the Minion Wizard, in the summon key (Behavior 31). A minion placement's variant therefore chooses its element, not its strength. For example the Minion Warrior rows 204 to 210 differ only in T, the four damage alignments and one further word (row offset 0xBC).
2. Faerie Sisters: the dodge chances depend on the living sister count (Behavior 20), and so do the movement and launch speeds (Behavior 4).
3. Health fraction: the sisters' animation speed rises as their health falls (Behavior 2); the health bands of Behavior 2d change the state; the Draik's stage thresholds are 0.66 and 0.33 of its health (Behavior 6); the Juppie's phase tests are in SPEC-0015.
4. Story flags: a story progress flag switches a single boss behaviour on or off. The Juppie spit is one example (SPEC-0015).
5. No per-area or per-act multiplier exists. An enemy met in a later area is harder only because a different row (another type) is placed there, and because more enemies are placed.

### 1. Think loop and class selection (High)

1. Each enemy starts one coroutine when it is given a class. The coroutine runs its class's start routine once, then loops forever.
2. Each loop iteration runs one think step of the current class, then yields exactly one tick (1/60 s). So each enemy thinks once per tick.
3. Class assignment rule: a request to set class id k with priority p is applied only if p is greater than or equal to the stored priority.
   - If k equals the current class id, nothing changes.
   - If k is 0 or names an empty table entry, the stored priority is cleared (set to 0).
   - If k is 40 or higher, the request is an error (the behavior count is exceeded).
   - When the request is applied, the old coroutine is torn down and the new class's coroutine starts.
   - Example: current class 12 with priority 5. A request for class 7 at priority 4 is ignored. A request for class 7 at priority 5 replaces it.
4. Class requests come from four sources (High):
   - Spawn default: when an enemy is spawned or a character is activated, it requests its default class at priority 1. The default class is the monster row's field 0x36; when that is none (negative when read as signed; 0xFFFF on disc), the companion row's field 0x10 is used instead (Layout section 4). The same request sets the character's townsfolk flag to 1 when the default class is 6 and to 0 otherwise (Layout section 3).
   - Explicit: a given class id is requested at priority 2. The explicit classes on the disc are 23, 24, 25, 26, 34 and 35. Classes 23 and 24 (walk to a locator), 25 and 26 (the same, then turn to the locator's facing) and 35 (walk along a path, 0.5 units per point) are the level-script walks (SPEC-0030 rows 106, 108 and 111; SPEC-0056). Class 34 is the controller of the live (summoned) petpet, requested at priority 2 when the pet is created (SPEC-0047). Confidence: High. A request for class -1 (or any negative class) through the same call is a spawn-default request at priority 1; when the default it finds is also negative, nothing is requested. Class 9 is never requested explicitly.
   - Pending apply: a request (class and priority stored on the enemy) made while the enemy's think coroutine is running is not applied at once. It is held and applied on a later tick by the enemy's per-tick think path, under the rule of step 3. A request made while no coroutine is running is applied at once. Where in the tick the held request is applied is Medium.
   - Group alert board: each enemy has a shared alert board. When an enemy enters AI mode 19 and its spawn record has a group byte (not -1), it posts an entry keyed by that group with class 0 at priority 0 that lives 100 ticks. The board keeps at most 40 entries, ordered by remaining life; the head entry counts down one per pass and is removed when its count reaches 1. The countdown is run by the main loop on every pass, also while the game is paused or another freezing screen is up (game-state code 9) and while the world is disabled, so board entries age while paused (High; SPEC-0007). The held-request step reads the best entry for the enemy's key when the board has changed since the enemy last looked, but an entry is used only if its priority is higher than the held or current priority, and priority 0 never is. So the board never wins and never changes a class. Confidence: High.

   These four sources are the complete set of inputs to the class-assignment rule. Class 9 (Cloud Beast) and class 10 (its companion) are never requested on the disc: no explicit request names them, no placed record selects monster row 1 or 2, no companion row default is 9 or 10, and the alert board posts only class 0. Confidence: High. The classes reached on the disc are the default classes of placed rows (1, 2, 6, 7, 8, 11 to 18, 20, 21, 32, 33, 36 to 39) and the explicit classes 23 to 26, 34 and 35; classes 3, 4, 9 and 10 sit only on rows no record places, and classes 22 and 27 to 31 have no requester at all, so they never run (Layout section 3a; High). (The run-time variant re-pick of Edge cases changes a row without requesting a class; Open question 159.)

### 2. Shared helpers and conventions (Medium)

- Navmesh pick (High; not a target search): walk the world's navmesh list in order. Accept a navmesh of kind 1 (ground; query mode 0) or kind 2 (flying; query mode 1) when one of its triangles contains the character's own position (a point-in-triangle test on the horizontal plane, by the signs of 2D cross products). Store the index of the first accepted navmesh as the character's current navmesh, or -1 when none is accepted. When the stored index changes, the character is located again on the new navmesh (its current and last-valid triangles) and its last-valid point is stored again; a failed locate raises a warning (text not specified). The query mode is 1 for a character whose Can Fly value is not 0, and 0 otherwise. The common-enemy pre-step (SPEC-0038) picks again this way when its ground countdown reaches 0. The Draik picks both: a ground mesh (mode 0, assigned) and a flying mesh (mode 1, kept). Its own status messages call them the ground mesh and the flying mesh, which confirms the two labels (High).
- No framework routine chooses a target among characters. Class-17 enemies always use the selected hero as their target (SPEC-0038 Behavior 6).
- Random helpers: an integer draw in [lo, hi], both ends included (bounds in either order; returns lo when lo equals hi), and a float draw uniform in [lo, hi) (the upper end is reached only by float rounding). The integer draw takes the generator output as an unsigned 32-bit number and its unsigned remainder by (hi - lo + 1), so the result is never below lo or above hi (High). These draw from the game's shared random generator, which is seeded once from the console clock at its first use and is not re-seeded per fight (SPEC-0014). The generator itself is described in SPEC-0008.
- Distances used by Mortog and Whinny are horizontal (x and z only). This is the same convention as the bosses (SPEC-0014).
- Movement (High): each tick the enemy runs the navigation step of SPEC-0056. That step uses this Behavior's goal latch (2b), which runs the look-ahead (2a) on the move point. It reads the mesh's route table, and the route step steers toward a clamped crossing point on the edge the table names. The arrival flag is set when the horizontal distance (not its square) to the goal point is below 0.25, or on a fresh block once the enemy is settled in the goal's triangle. Setting a new move point through the plain setter restarts the goal's triangle search when the new point is outside the stored target triangle and more than 1.0 from the previous move point (SPEC-0056, Walk target setters).
- Locomotion: a creature has a table of animation slots, one per locomotion mode (0 to 10; slot words below). The slot values are set at spawn and the movement step plays the animation of the current mode. For Mortog and Whinny the slots are: mode 0 (walk) = animation 2, mode 1 = animation 2, mode 2 (run) = animation 3. Their other slots are unset. The walk and run modes are the ones confirmed by use. A creature has no speed constant: it travels by the root motion of the clip it plays (SPEC-0040, Behavior 14D). The Whinny's speeds are in Behavior 5; the Mortog's are not established.
- Locomotion modes and slot words (High for the mapping). The mode word is character +0xA3C. The movement step plays the animation held in the slot word of the current mode m:

  | Mode | Slot word | Steering |
  | --- | --- | --- |
  | 0 to 4 | +0xB58 + 4m | 0 to 2 forward gaits (walk, run, fast run); 3 and 4 backward (toward minus the target direction) |
  | 5 | +0xB70 | strafe to one side |
  | 6 | +0xB6C | strafe to the other side |
  | 7 to 10 | +0xB74, +0xB78, +0xB7C, +0xB80 | flyer forms (forward, backward, side, side) |
  | any other value | +0xB58 | as mode 0 |

  Flyer redirect (High; R-0039): when the row's Can Fly word is 1 or 2, the character's grounded countdown (+0xBF4) is not positive, and the mode is below 7, the mode is first mapped: modes 0, 1 and 2 use the flyer slot +0xB74, modes 3 and 4 use +0xB78, mode 5 uses +0xB7C and mode 6 uses +0xB80. The movement step then reads the slot as above. For every other row, and for a flyer while its countdown is positive, the table above applies unchanged. The four flyer slots are written by a second setter that takes four values in the order +0xB74, +0xB78, +0xB7C and +0xB80 (not used by the Miamouse; its entry leaves them at -1).

  The shared slot setter takes seven values and writes them in order to +0xB58, +0xB5C, +0xB60, +0xB64, +0xB68, +0xB6C and +0xB70. Its 6th value is therefore mode 6's clip and its 7th is mode 5's clip. Example: the townsfolk set is (1, 6, 5, 3, 3, -1, -1). Each value is an entry number of the creature's main animation table (SPEC-0005), not a clip index; the entry names the clip. The five bosses set their slots once per pass of their own controller loop (the Ixi Chief before every state handler) with these values, in the setter's order: Ixi Chief (9, 10, 30, 14, 15, 17, 16); Black Knight (3, 3, 3, -1, -1, 5, 4); Gelert Assassin (12, 12, 12, -1, -1, 15, 14); Werelupe King (2, 2, 2, -1, -1, 4, 3); Gnarfas (5, 6, 6, -1, -1, 14, 13). So the Ixi Chief's mode 5 plays entry 16 and mode 6 entry 17, the Black Knight's mode 5 plays entry 4 and mode 6 entry 5, and the Gelert's mode 5 plays entry 14 and mode 6 entry 15. The clips, lengths and root travel of the first three are in SPEC-0014, SPEC-0012 and SPEC-0013 (High). A boss's non-flyer rows store the requested mode unchanged.

  Steering by mode (High). The steering call takes a unit horizontal direction d and a weight w, and blends the move direction toward a direction v: new = unit(w v + (1 - w) current), where current is the motion record's move direction. Here up is (0, 1, 0).
  - Modes 0 to 2, 7 and any other value: v = d, with the caller's weight.
  - Modes 3, 4 and 8: v = -d, with the caller's weight.
  - Modes 5 and 9: v = up x d = (d.z, 0, -d.x). Modes 6 and 10: v = d x up = (-d.z, 0, d.x). In these four modes v is normalised and the caller's weight is replaced by 0.1, or by 1.0 (a snap) when the dot product of v and the character's facing is above 0.9.
  Ixi Chief application (R-0088): charge sub-steps 1 and 4 use modes 0 and follow the stored charge point and current hero position respectively. Rush-forward uses mode 1, strafe-close mode 0, and strafe-back mode 3. Modes 0 and 1 steer along the navigation step's chosen direction; mode 3 steers opposite it. The shared navigation step supplies the weight: route-crossing steering uses 0.1 and the settled same-triangle approach uses 0.3; other branches are specified in SPEC-0056.
  The mode setter remaps a requested mode 0 to 6 through a fixed table to a flyer mode only while Can Fly is 1 or 2, the ground countdown is below 1 and the request is below 7. Otherwise it stores the mode unchanged. The flyer modes 7 to 10 are entered only through the mode setter, which remaps modes 0 to 6 to the flyer forms while Can Fly is 1 or 2 and the ground countdown is at or below 0 (Layout section 4). The only fixed writers of modes 3 to 6 are boss routines: mode 3 by an Ixi Chief routine, and modes 5 and 6 by the Black Knight orbit, the Gelert strafes, the Gnarfas back-off, the Werelupe King orbit and an Ixi Chief routine. Class 17 sets its modes through its gait logic (SPEC-0038).
- Animation helpers: play an animation by table index (with blend and two unused arguments), test whether an animation has finished, test whether it is currently playing, and read its progress as 0..1. A one-shot animation returns the enemy to its idle state when the finish test is true (or at 50 % progress for classes that use it).
- Facing and aim: aim at a direction with a blend weight of 0.1, 0.25 or 1.0 per tick (1.0 snaps; the weight is a per-tick turn-toward weight, not a fixed angle step).
- Sister-controller animation rate (see Behavior 4): before each state dispatch, the class routine writes P = 2 − HP fraction after the NPC update's reset and status processing, but before clip re-time. The direct write replaces the accumulated status factor in P and sets the same tick's clip-rate factor from 1.0 at full HP to 2.0 at zero HP; root motion follows from the later animation and motion passes.
- Dead test (High): a townsfolk or enemy character is dead when its base state is 16, or when its AI mode word (+0xA34) is 16 and the counter at +0xD14 is above 0. When an AI-controlled character's hit points reach 0, the shared death hook sets its base state to 16, except for behavior classes 1, 11 and 32 (High; the hook is shared by every class, Behavior 34). So a boss whose controller never rewrites its base state, such as the Darkest Faerie (class 37, SPEC-0054), meets the dead test. The proximity streaming of SPEC-0006 sets the character's kill bit on every tick it is dead, and releases it once its death disappear countdown is below 0 (SPEC-0038 Behavior 36).
- Kill memory (High): kill bits are stored back to the per-slot table only when the per-pass load step releases a world for a request that is not a game load (replacement loads, the after-movie reload, the front-end entry), and when the game is saved (SPEC-0006, Kill memory). Kills made in a world that is then released by a streaming load or by the enter-interior or leave-interior job are forgotten: those enemies come back on the next visit. Reproduce this.

- Six-entry attack-band tables (High for the values and access rule). A family of getters returns entry k (k from 1 to 6), the word at offset 4 x (k - 1) of a table. Values, given as (entries 1 to 5, entry 6): 0x2DA678 (10.0, 20.0); 0x2DA68C (20.0, 10.0); 0x2DA704 (5.0, 15.0); 0x2DA718 (15.0, 1.0); 0x2DA754 (0.0, 10.0); 0x2DA768 (10.0, 4.0); 0x2DA7CC (5.0, 10.0); 0x2DA7E0 (10.0, 5.0); 0x2DA8BC (0.0, 5.0); 0x2DA8D0 (5.0, 4.0). The stage-choice dispatcher selects one of five routines for current stage 1 to 5 and passes that stage value unchanged to the getters. In shipped play the active stages are 1, 3 and 5; stages 2 and 4 are never entered, and death stage 6 has no attack-choice case. Thus these getters receive only indices 1, 3 and 5 in play; none of the differing sixth entries is read by the attack-band choice path. These tables are the band limits of Behavior 6.3. **[High for dispatch, indices and reachability; meaning of the unused sixth values unknown]**

### 2a. Steering look-ahead (High; the second-side gate Medium)

The look-ahead adjusts the character's goal point before it moves. It does not find a target. Input: the walk target W. P is the character's position and up is (0, 1, 0).

1. Lanes. Let d = unit horizontal (W - P) and s = unit(d x up), which is 1.0 long. Two test segments run parallel to the straight line: from P + s to W + s, and from P - s to W - s.
2. Obstacles. Each is a vertical finite cylinder, tested against the first segment and then, if that misses, the second. There are four sources, in this order:
   - the kind-3 cylinders of cell list A of the character's grid cell (SPEC-0018), with their own length and radius;
   - each active movable block, at its point;
   - the entries of the grid cell at P whose type value is 1 or 3, at their point;
   - each other character in the region's character list that is not hidden and not dead, at its position.
   The last three use a cylinder 20.0 long (centred on the point) with radius 5.0.
3. Choice. Of all hits, the obstacle O kept is the one nearest P (horizontal distance). Let R be its radius (5.0 for the last three sources). With no hit, W is returned unchanged.
4. Shift. Let u = unit horizontal (O - the current hero's position) and n = unit(u x up) = (-u.z, 0, u.x) after normalising. The first candidate is O + (R + 5.0) n. If it lies on the character's navigation mesh (the locate of SPEC-0018), it is returned. Otherwise, if the locate's scan cursor is still set, W is returned for this tick. If the cursor is clear, the second candidate O - (R + 5.0) n is tried: it is returned if it is on the mesh, and W otherwise.
5. Result. The goal latch of Behavior 2b locates the returned point and stores it as the character's goal point.

The side is fixed by the hero's position relative to the obstacle, not by the walker's.

### 2b. Nearest triangle and goal latch (High; three-dimensional centroid distance)

Nearest triangle, for a query point Q:

1. When the character's current navmesh index is -1 or not below the navmesh count, the result is none (-1).
2. Walk the triangles of the current navmesh in list order, p = 0, 1, 2 and so on. A triangle's reference point is its centroid: the sum of its three corners divided by 3.0. Its distance is the three-dimensional Euclidean length of the difference between the centroid and Q, including vertical separation.
3. The best distance starts at 1e9. A triangle is considered only when its distance is strictly smaller than the best so far, so a tie goes to the earlier triangle.
4. A considered triangle is kept only when the occupancy test (below) is false. Keeping it sets both the character's current triangle and its last-valid triangle to p, makes its distance the best distance, and makes its centroid the result point.
5. The result is the centroid of the last kept triangle. When no triangle is kept, the character's triangle indices do not change and the result point is not defined (treat it as none).

Occupancy test (High; an original quirk to reproduce): walk the level's enemy list by list position i. Skip a character X when X is in base state 16 or 42, when the shared character-visibility predicate returns true, when X is the character itself, or when X is on another navmesh. This predicate is also a negative gate in hero auto-target eligibility and enemy-projectile contact, so it is a shared interaction-visibility test rather than a navmesh-only flag. The predicate's full rule follows this paragraph (Open question 27). The occupancy test is true as soon as X's last-valid triangle index equals i, X's own list position. It never reads the triangle p being tested, so within one search either every triangle passes or none does.

**Hidden test, full rule (High).** The shared character-visibility predicate returns true when a character X is excluded from interaction. It is evaluated in this order:
1. If X's invisibility-off word is nonzero (set by the DisableInvisibility script command; 0 at creation), the result is false.
2. If the monster row's Display in Default field (the u16 at row offset 0x82) is 0: the result is true when X's AI mode equals the default mode of its spawn record (the u16 at record offset 0x84), or when X's AI mode is 21 and animation 82 is not playing; it is then false instead when the active hero's body-outfit entry holds item 126 (the Tattered Cloak). In every other case the result is false. So a kind that is invisible in its default state is excluded from hits, auto-targeting and navmesh occupancy until it leaves that state, unless the active hero wears the cloak, which makes it interactable.
3. If Display in Default is nonzero: the result is true when X's nest-held word is nonzero (only the Spyder Nest controller sets it, so it is 0 in shipped play). Otherwise, with the emerge sub-state word of state 8 equal to 0: false when the spawn record's cue word (offset 0xC4) is nonzero; else true exactly when the hide animation (9, or 128 to 131 for emerge selector 1 to 4; SPEC-0038 Behaviors 20 and 40) is the current animation. With the emerge sub-state word nonzero: true only when X's AI mode is 14 and the state-14 phase word is 1 (walking home to hide); otherwise false.
Test vectors: Display in Default 0, AI mode equal to the default mode, no cloak: true (a hero projectile passes through, auto-target skips it, navmesh occupancy skips it). The same with the cloak worn: false. The same with the invisibility-off word set: false. Display in Default 0 and AI mode agro (not the default mode, not 21): false. Display in Default 1, cue word 0, emerge sub-state 0, hide animation playing: true; with cue word 1: false.

Goal latch (High):

1. Every tick the look-ahead of Behavior 2a gives the goal point, which is then located on the current navmesh.
2. While the latch is clear: when the locate succeeds, the latch is set and the located triangle is returned. When it fails, the goal point is offset by j = (id mod 20) x 0.2 - 2.0, where id is the character's id word; j is added to x for an even id and to z for an odd id. The nearest-triangle search is run from that point, its centroid becomes the goal point, and the character's current triangle is returned. The latch stays clear.
3. While the latch is set: when the locate fails and the last-valid triangle is not -1, the goal point becomes the stored last-valid point and the last-valid triangle is returned. Otherwise the locate result is returned.
4. Clearing (High). The latch is set to 1 by the character's second initialisation and by a successful locate in step 2. The goal locate is the batched form of SPEC-0018: the cached target triangle first, then up to 50 triangles per tick from its cursor. It clears the latch when a batch that ends at the mesh's last triangle misses. So the latch reads "the goal point was found on the mesh". It drops only after the whole mesh has been searched without a hit, and it rises again on the next hit.
5. Use. The returned triangle and goal point are the target triangle and goal point of the navigation step (SPEC-0056).

### 2c. Animation steps and level-state posts (High for the two patterns and the owners; Medium where marked)

The small integer codes that many state routines use are not messages. They are indices into the object's own animation table (Behavior 2), and the "relayed" codes are level states. Two patterns cover most of these routines:

1. Play, then wait for the end. If animation n is not the object's current animation, the routine starts it (no blend, default loop flags) and returns. If it is current, the routine tests whether it has finished (the clip has reached 0.99 of its length). Only when it has finished does the routine act: it sets a new state, clears a flag, or counts.
2. Level-state post. The action can set a numbered level state on the level's event manager. Level scripts and other controllers test, set and clear the same numbered states (SPEC-0030).

Owners and uses (High for the owners; the per-row rules are Medium unless marked):

| Number | Kind | Owner and rule |
| --- | --- | --- |
| 0, 1, 2, 3, 4, 15, 16, 17 | Animation | Gnarfas states (SPEC-0020). Animation 2 is its death: when it finishes, level state 2 is set. Animation 4: played each tick of one state; when the count at state-block offset 0x2C is below 1, the state becomes 5 and the words at 0x44 and 0x08 are cleared. One routine picks animation 16, 17 or 15 from a field value of 0, 1 or 2. |
| 3, 4 and 6 | Animation | Plague Serpent (SPEC-0016). Animation 3 ends a waypoint leg. With the flag at state-block offset 0x38 clear: animation 4; when it finishes, state 6. With the flag set: animation 6; when it finishes, the flag is cleared. |
| 5 | Animation | Plague Serpent phase change (Behavior 2d). A second serpent routine plays animation 5 and, when it finishes, sets state 0 and the word at state-block offset 0x1C to 1. |
| 6, 13, 14, 20 | Animation | Class 37 four-member controller (the Darkest Faerie's final fight, SPEC-0054): 6 is the idle animation, 13 and 14 the stun animations, 20 the hover. |
| 10, 12, 13, 15, 16 | Level state | Class 37 four-member controller (SPEC-0054): set as listed under message 13 below and in Behavior 2e. While its code is 3 (orb volley) the controller holds its cast glow on her; in any other code the glow is released. |
| 13 | Animation | Miamouse (Behavior 28): while animation 13 is current, its state is forced to 3. |
| 26, 27 | State | Draik stun hit registration (below). These are state numbers, not animations. |

**Message 13 (class 37 code 6, the stun count; High).** Play animation 13. Each time it finishes, add 1 to the stun counter. Below 3, go to code 0. At 3 or more, set level state 12, then level state 10, and set the controller's done flag. The code stays 6. While the animation is held finished, the counter keeps rising and both level states are set again each tick, until the gate moves the controller to code 1 (SPEC-0054). The done flag is read only by the controller's choice step, which is skipped while the flag is set. Code 1 (burst) clears the flag, together with the stun counter, when it sees level state 15.

**Message 27 (Draik stage-3 stun hit registration; High).** This runs while the Draik's state is 27 (stunned in stage 3) and its current animation is not 46 (the stage-3 hit reaction). Let M be "the hero's melee attack is active" and S be "any of the hero's six projectile slots is active" (Medium for the two labels).
- When neither M nor S holds, the hit latch (state-block offset 0x5C) is cleared and nothing else happens.
- Otherwise, when the latch is clear or S holds, the Draik tests its five hit groups (Section 6.4). If any passes, the state becomes 3 (the reaction) and the latch is set.
- So a melee swing counts once, and a projectile counts each time (Section 6.4). The stage-1 form is the same with state 26, reaction animation 23 and the leg-inclusive groups; a pass sets state 1.

**Message 4 (High).** No shared message 4 exists. The three readings are the Gnarfas animation-4 state and the Serpent animation-4/6 pair (both in the table), and a Draik routine that tests the stage (value 4) and runs the stage-4 chooser (Section 6).

### 2d. Health bands and state 11: the Plague Serpent's phase check (High)

The health-band check belongs to the Plague Serpent alone (SPEC-0016 holds the full rule); no other class runs it. Each tick the serpent's loop runs it, except in states 1 (move), 5 (relocate), 6 (arrive), 7 (bite) and 11 (phase change); states 2 and 3 are never entered, so in play it runs only while idle (0) or belching (4) (SPEC-0016). It takes the ratio r of current HP to maximum HP and picks a wanted phase. The thresholds 0.25, 0.5 and 0.75 are fixed constants, not data words.

- r at most 0.25: wanted phase 4.
- r above 0.25 and at most 0.5: wanted phase 3.
- r above 0.5 and at most 0.75: wanted phase 2.
- r above 0.75: no check.

The check compares the wanted phase with the serpent's phase counter (state-block offset 0x1F8, starting at 1); no band is stored. When they differ, the serpent:
1. sets state 11 (phase change);
2. records the two phase-change anchor slots from its current anchor (SPEC-0016);
3. plays its animation 5 (the turn).

The phase change adds 1 to the counter when it completes. So a large drop chains one change per phase, and healing into a higher band repeats changes (SPEC-0016, Edge cases).

### 2e. Descent, homing and glide steps (High for the owners and the words; Medium for the steps)

Descent (step 0.05, word 0x2DA64C). The "approach" routines are codes 8, 9 and 10 of the class 37 four-member controller (SPEC-0054).

1. Code 8 (timeout descent): each tick the ground height under her is probed from 8.0 units above her. While her height over the ground is above 0.05 she moves down 0.05. Otherwise she sets level states 13 and 10 and goes to code 1.
2. Code 9 (fall) plays animation 20 and descends the same way; at the ground it sets level state 16 and goes to code 10. Code 10 plays animation 20 while level state 13 is clear; once it is set, she sets level state 10 and goes to code 1.

Homing and glide steps:

3. Step homing (Queen Fyora's slot types 0 and 1; Behavior 19). Each tick the slot moves 0.5 units (word 0x2DAAA8) toward its target's target point, horizontally; the slot keeps its height. Its effect is moved with it.
4. Damped homing (class 37 four-member controller). Each call moves toward the target with a damping factor of 0.75 (word 0x2DA638). When the gap is under 1.5 (fixed constant), it sets a 3-second timer (word 0x2DA63C; 180 ticks) and sets state 5.
5. Glide (Queen Fyora's slot type 3; Behavior 19). Each tick the slot adds 0.5 times its direction to its position and counts its timer down. When the timer ends, the slot is freed. When the hero is within 1.5 units (horizontal), the hero takes a hit of 5 and the slot is freed.

The target point is the target object's own aim point.

### 2f. State 13 entry gate (High)

This is the class-6 townsfolk greeting entry check described in SPEC-0056. It applies when the character is not excluded by the state-exclusion test (SPEC-0033, State exclusion) and is not already in state 13. Let H be the hero's position relative to the character. The gate passes when both of these hold:

1. The horizontal distance to the hero is below 6.0 (boot data word at 0x2DD8CC, f32); vertical separation does not affect this distance test.
2. The hero is in the character's front half: the horizontal facing dot the horizontal direction to the hero is greater than 0. A zero dot is outside the front half.

When the gate passes, the townsfolk enters mode 13. Three counters (+0xAA4, +0xAA8 and +0xAAC) are set to 0. The field at +0xAB0 is set from a character-class test for the character's id (the id list is in the character runtime spec). In mode 13, a zero value enables the timed animation branch: the counter at +0xAAC advances up to 3 seconds, then animation 30 is requested and its completion advances the handler. A nonzero value skips that animation branch. The separate distance and facing exit test still runs for either value, returning the character to mode 0 when the hero is farther than twice the entry threshold or the horizontal facing dot is below -0.1.

### 2g. Launch record setup (Medium)

An oriented entry in a twelve-slot transient sprite array is initialized from two orientation vectors, a third value, and an activation flag. The owning feature or visual effect is not identified (Medium).

- The orientation block at +0x10 to +0x4C is computed from the two vectors by a helper.
- The third input is stored at +0x0C; both known callers pass 0.
- The activation flag is stored at +0x00; the known callers set it for every fifth entry and clear it for the others. Offset +0x04 is cleared.
- The value 40 (boot data word at 0x2DD418, u32) is stored at +0x08.

The initializer then computes derived transform and bound data. A per-entry updater handles the active and falling states and refreshes those fields. The entries form a three-by-four array; one caller initializes it, while another refreshes it within a 50-unit range and hides it outside that range. The meaning of offset +0x0C and the owning visual effect remain unknown.

### 3. Mortog (class 39) (High for states, zone rules and constants)

Mortog is a creature that flees the hero, holds still when it is inside a target zone, and wanders while keeping out of the zone. Its states are idle (0), wander (1), flee (2) and hold (3).

Start: target index -1; enter the idle state; play animation 0; set the locomotion slots (Behavior 2): mode 0 = animation 2, mode 1 = animation 2, mode 2 = animation 3.

Configuration (set by the level script, defaults in the data):

- Zone centre A: the home point, a point placed by the level.
- Zone radius, squared (L): the horizontal squared distance between two placed points, set by the script's setter (InitMortogBehavior): L = (xi - xj)^2 + (zi - zj)^2 for locators i and j; the height is not used (High). Default 0. With L = 0 the zone never fires, and the wander avoidance rule is also off.
- Zone count: the number of Mortogs currently holding in the zone (starts at 0; the data word is at 0x2DB218).
- Hold time range: 10.0 to 20.0 seconds (data words). The setter also writes the range, and a second script call changes it later (ChangeMortogTimes). The hold maximum (the upper word) must be non-zero for the hold to end (see state 3; High).
- Idle timer range: integers 2 and 8, so an idle timer is a random whole number of seconds in [2, 8].
- Flee distance: 8.0 units (the distance the flee point is placed from Mortog).
- Target default: -1.

Tuning constants (data words):

| Parameter | Address | Value | Unit |
| --- | --- | --- | --- |
| Zone radius (squared), default | 0x2DB214 | 0.0 | units^2 (set by script) |
| Hold time minimum | 0x2DB21C | 10.0 | seconds |
| Hold time maximum | 0x2DB220 | 20.0 | seconds |
| Idle minimum | 0x2DB228 | 2 | seconds (int) |
| Idle maximum | 0x2DB22C | 8 | seconds (int) |
| Arrival radius (squared) | 0x2DB230 | 4.0 | units^2 (2 units) |
| Notice radius (squared) | 0x2DB234 | 25.0 | units^2 (5 units) |
| Flee radius (squared) | 0x2DB238 | 225.0 | units^2 (15 units) |
| Flee distance | 0x2DB23C | 8.0 | units |
| Zone count | 0x2DB218 | 0 | count |
| Target default | 0x2DB224 | -1 | index |

Run rule: while fleeing, the boss uses mode 0 (walk, animation 2) when the hero's squared horizontal distance is above 112.5 and mode 2 (run, animation 3) otherwise.

States:

| State | Entry and per-tick behavior | Transitions |
| --- | --- | --- |
| 0 Idle | Play animation 0. Count the idle timer down by one tick. When it is below 1, pick a random point for wandering. | To Flee (2) when the squared horizontal distance to the hero is below 25. To Wander (1) when the idle timer ends. |
| 1 Wander | Each tick, in this order: the hero test; the arrival test against the current move goal; otherwise the zone rule below, then walk (mode 0) toward the move goal. | To Flee (2) when the squared horizontal distance to the hero is below 25. To Idle (0), setting a new idle timer, when the squared horizontal distance to the move goal is below 4. |
| 2 Flee | First, the zone test: if the squared horizontal distance from Mortog to the zone centre A is below L (strict), enter the hold state (3) with a timer of round(60 x uniform real in [min, max]) ticks and increase the zone count by 1. Then, in the same tick, if the squared horizontal distance to the hero is below 225 and the path to the point 8 units directly away from the hero is clear, move toward that point (run rule above). Otherwise go to Idle. | Hold (3) when the zone test passes (the same tick continues, see Edge cases). Idle (0) when the hero is 15 units or farther or the path is blocked. |
| 3 Hold | Play animation 0. Count the hold timer down by one tick. | When the timer is below 1 and the hold maximum is non-zero: pick a whole number k in [4, 8], set the move goal to k x unit(position - A), go to Wander (1), and decrease the zone count by 1. The goal has no position term (see Hold end below). |

Distances (High). The zone test, the hero tests and the arrival test use the horizontal squared distance (x and z only). The unit vectors of the hold end and of the zone rule use all three coordinates.

Hold end (High). At the end of a hold the new move goal is k x u, where u = (P - A) / |P - A| for Mortog's position P, and k is a whole number drawn uniformly from 4 to 8. The position is not added, so the goal is a point within 8 units of the world origin, not a point beside Mortog. k is a distance (the length of that offset), not a speed. On the first wander tick the zone rule usually replaces this goal: Mortog has just left the zone, so its look-ahead point is near A (see the rule). If the rule does not fire, Mortog walks toward the point near the world origin until the hero comes near, it arrives, or the zone rule fires on a later tick. Reproduce this.

Wander zone rule (High). On every wander tick that passes the hero and arrival tests, look 6 units ahead of Mortog along its facing. If the squared horizontal distance from that look-ahead point to A is below 2L, the move goal becomes P + 3L x u, with u as above (L is the squared value; reproduce as stated, see Edge cases). Otherwise the move goal is left as it is. The random wander point is chosen only on leaving idle.

Zone count and launch (summary of the rules above):

- The zone count goes up by 1 on hold entry (state 2 to state 3) and down by 1 on launch (state 3 to state 1).
- Hold ticks = round(uniform(min, max) x 60), with min and max the hold time range in seconds. Example: a draw of 12.5 seconds gives 750 ticks.
- At the end of a hold, when the hold maximum is not 0, Mortog launches with a whole number from 4 to 8, drawn uniformly, and returns to wander (state 1). The launch value is the length of the move-goal offset (hold end above).
- When the hold maximum is 0, the hold timer keeps counting below 0 and the hold never ends. Mortog never launches, and the zone count never falls.

Use in play (High). One level (SPEC-0041, module 1363) uses Mortogs as a puzzle: four must hold in the zone at the same time. It starts the hold range at 25 to 30 seconds and widens it by 5 seconds at each odd-numbered rise of the zone count. When the count reaches 4, it sets the range to 0 and 0, so the four holds never end and the count stays at 4.

Test vectors:
- Zone count (synthetic): a hold entered with the zone count at 0 raises it to 1. A hold drawn at 750 ticks ends after 750 ticks with a launch, and the count returns to 0. With hold maximum 0, the count stays at 1 indefinitely.
- Hold end: P = (100, 0, 50), A = (96, 0, 47), k = 4. Then P - A = (4, 0, 3) with length 5, u = (0.8, 0, 0.6) and the move goal is (3.2, 0, 2.4). It is not (103.2, 0, 52.4).
- Hold end with height: P = (3, 4, 0), A = (0, 0, 0), k = 5. Then u = (0.6, 0.8, 0) (three-dimensional) and the goal is (3, 4, 0).
- Zone rule: L = 25, A = (0, 0, 0), P = (4, 0, 0), facing +x. The look-ahead is (10, 0, 0) with squared horizontal distance 100, which is not below 50, so the goal is unchanged. Facing -x, the look-ahead is (-2, 0, 0) with 4 below 50, so the goal becomes (4, 0, 0) + 75 x (1, 0, 0) = (79, 0, 0).
- Zone test: L = 25 and Mortog at horizontal squared distance 25.0 from A: no hold (strict). At 24.99: hold.

Edge cases:

- **Flee continues into idle.** When the zone test enters the hold state, the flee step still runs in the same tick. If that step then goes idle (the hero is too far or the path is blocked), the state becomes idle but the zone count stays increased. Reproduce this.
- **Zero zone radius.** With L = 0 the zone test is "below 0" and never passes; the wander avoidance also never applies.
- **Units of L.** L is a squared distance, but the wander rule uses 2L and 3L as if they were distances. Reproduce as stated.
- **Chase name.** Flee is state 2. There is no chase state for Mortog.

### 4. Faerie Sisters controller (classes 13, 15, 16) (High for class ids, state ids and tuning words; Medium for state roles and the alternate loops)

Three sisters fight together. Each sister runs one of three controller routines, under class ids 13, 15 and 16. The three routines share one state set and the same tuning words. Each sister has its own state block, its own cooldowns and an order index. The sisters' player-facing behaviour (idle, orbit, dodge, ranged attacks, the group dome attack, flinch and death) is specified in SPEC-0011; this section gives the controller's state set, class ids and tuning words, and SPEC-0011 gives the behaviour they serve. The state ids below are the controller's own numbering and are used in the state table.

**Alive count.** The alive count is the number of sisters whose state is not dead; it is recomputed once per frame. When it reaches 0, the level's region event fires once (Medium). It is set to 3 when a sister controller starts and to -1 at cleanup (Medium; Open question 111). It is not a difficulty value. The movement speed depends on it (see Speed rule).

**Routine differences.** Classes 13 and 15 share alternate loop B, which runs instead of their main loop when level flag 0 is set (Medium). It does not write the shared alive count. Class 16 uses alternate loop A under the same flag and sets the alive count to 3 at entry (Medium). The class-13 startup diagnostic names Malice, consistent with the placement mapping in Open question 138; class 13 shares the loop with class 15 rather than having a unique alternate loop. The routines differ in some handler subsets; the state set is the same.

Tuning constants (data words):

| Parameter | Address | Value | Use |
| --- | --- | --- | --- |
| Orbit band, minimum distance | 0x2DAB48 | 15.0 | the orbit radius is drawn in [15, 30] (SPEC-0011) |
| Orbit band, maximum distance | 0x2DAB4C | 30.0 | same |
| Sister movement speed, 3 alive | 0x2DAB50 | 7.5 | units per second (the movement speed of the path-following step, SPEC-0011; not the orbit speed) |
| Energy-ball launch speed, stage 3 (3 alive) | 0x2DAB94 | 0.5 | launch speed: the launch velocity is the unit direction toward the aim point times this value (SPEC-0011) |
| Energy-ball launch speed, stage 2 (2 alive) | 0x2DAB98 | 0.75 | same |
| Energy-ball launch speed, stage 1 (1 alive) | 0x2DAB9C | 0.9 | same |
| State 4 parameter (energy-ball damage) | 0x2DAB58 | 25 (int) | damage per energy ball (SPEC-0011) |
| State 5 status pairs (damage, then effect chance) | 0x2DAB5C and 0x2DAB60; 0x2DAB64 and 0x2DAB68; 0x2DAB6C and 0x2DAB70 | 10 and 100 for each pair | damage and chance of the three status missiles, in the kind order of SPEC-0011 (slow, poison, curse) |
| State 6 special-missile parameters | 0x2DAB74, 0x2DAB7C, 0x2DAB80 | 20, 50, 10 (int) | mote-eater damage, leach drain damage, thorn damage (SPEC-0011) |
| Thorn maximum hits | 0x2DAB84 | 3 (int) | SPEC-0011 |
| Dome duration | 0x2DAB88 | 40.0 | seconds (SPEC-0011) |
| Dome shot interval | 0x2DAB90 | 3.0 | seconds (SPEC-0011) |

The three launch-speed words are the words that SPEC-0011 lists as projectile speed by alive count (stage n is used with n sisters alive: 0.9 with 1 alive, 0.75 with 2, 0.5 with 3). Zero alive applies the 3-alive value, since no sister remains to fire.

Constants in the behavior that are not data words (fixed constants): the melee-dodge trigger distance (7 units), the "close enough" tolerance (0.5), the sister's movement speed for 2 and 1 alive (10.0 and 12.5 units per second; see Speed rule), and the dome radius (0.8 x 25 = 20 units).

Speed rule (High): the path-following move step uses 12.5 units per second with 1 alive and 10.0 with 2 alive (fixed constants of the move routine, not data words), and otherwise the word 0x2DAB50 (7.5 on disc). Per-tick step = speed / 60.

Animation rate: before the state switch each tick, the sister's class routine writes P = 2 - HP fraction. This follows the NPC update's reset/status work and precedes clip re-time, so it changes that tick's clip rate and replaces the status factor accumulated in P. At full HP the factor is 1.0; at zero HP it is 2.0 (High; SPEC-0011).

HP-based state chooser (per tick in idle or move, from hp fraction r = HP / max HP of the sister; High). The first rule that holds applies:

- If r is below 0.33 and the third counter (the special cooldown) is 0 or less: choose state 6 (special missile).
- Otherwise, if r is below 0.66 and the second counter (the status cooldown) is below 1: choose state 5 (status missile).
- Otherwise, if the first counter (the energy cooldown) is below 1: choose state 4 (energy ball).
- Otherwise choose state 1 (approach or move).
- So below 0.33 a sister whose special cooldown has not expired falls through to the middle band. All three controller routines handle state 6.
- When a new state is chosen, the sister's fired flag is cleared.
- A "start special" argument starts state 7 (group attack) at sub-state 0.
- If the level region bit is set, the sister plays its idle-like animation and goes to state 0.

Counters: the four action/group cooldown counters (energy, status missile, special missile and group attack) count down by one per tick unless the region bit is set. The special-missile cooldown is drawn uniformly from 1.25 to 3.0 s (75 to 180 ticks; SPEC-0011); the other ranges are given by SPEC-0011. A separate per-sister timer is initialized to 0, decremented by one per tick while positive when the region bit is clear, and set to 60, 120 or 180 ticks (a random 1 to 3 multiplied by the 60-tick-per-second frame rate) when a hero projectile passes the dodge proximity test (Behavior 20); later proximity hits in the same scan can set it again if an earlier dodge roll failed. This is the missile-dodge scan cooldown: projectile scanning runs when it is below 1, and a successful dodge roll changes the sister to state 2. Its observed behavior does not establish a developer-facing “fifth counter” label; it is not read by the HP state chooser.

States (roles marked Medium are inferred from the animation ids that SPEC-0011 gives for the same behaviours):

| State | Animation | Behavior |
| --- | --- | --- |
| 0 Idle | 1 | Idle / walk loop (SPEC-0011 idle). |
| 1 Approach (move) | 1 | Orbit or chase the target at the speed rule (SPEC-0011 move). If a target index is valid, aim toward it. Branch on the vertical offset to the target against a random band [15, 30] (details Low). |
| 2 Dodge missile | 13, or 14 when the variant flag is 1 | One-shot; back to state 0 when the animation finishes (Medium). |
| 3 Dodge melee | 15, or 16 when the variant flag is 1 | One-shot; back to state 0 when the animation finishes (Medium). |
| 4 Energy ball | 17 | When the animation finishes, or the release event fires, or progress passes 0.5 (once per animation), spawn a projectile of effect 460 from the sister's position, launched at the launch speed for the alive count (stage n for n sisters alive). Then back to the chooser. Damage 25 (0x2DAB58). |
| 5 Status missile | 18 (all kinds) | The sister requests main action 18 for every status kind. On disc, her main table maps both action 17 (state-4 energy ball) and action 18 to clip 19 for all three sisters in region 1123. The kind selector at block +0x08 chooses poison, curse or slow. The corresponding pool behavior/visual-kind pairs are (12, 18), (13, 19) and (11, 17), with effect 462, 463 or 461, respectively. Each projectile does 10 damage and has a 100% status chance; the applied statuses are poison, curse and slow, respectively (High). |
| 6 Special missile | 19 | The same kind selector at block +0x08 chooses kind 0 thorn, kind 1 leach or kind 2 Mote-eater. On the trigger it spawns effect 466, 465 or 464, respectively. Projectile animation pairs are (16, 22), (15, 21) or (14, 20), respectively. Damage is 10, 50 or 20, respectively (High; the exact kind mapping is confirmed by the shared state handler). |
| 7 Group (dome) attack | phase-specific; sub-state 3 requests action 23 | Sub-states 0 to 4, with -1 after the end. Sub 0 moves each sister toward her group-indexed ring point using the movement-speed word divided by 60 per tick (7.5 units/s at baseline) and advances when the remaining distance is no greater than that per-tick step. Sub 1 waits until every living sister has left sub-state 0; then sub 2 begins, the elapsed timer resets, the dome forms, and the first shot delay is drawn from 0 to 180 ticks. Sub 2 fires when its timer expires, then resets that timer to 180 ticks; it runs while elapsed time is at most 2,400 ticks and the dome exists. A hit during sub-state 2 enters sub-state 3, which requests action 23 and moves toward the ring point recomputed by the same position helper with the same sister group index, using 5 units/s. In region 1123, action 23 resolves to main clip 15 for Malice, Spite, and Vanity; the main table also labels its sound cue as the sister's take-damage cue. When the action finishes, sub-state 3 returns to sub-state 2 if the dome/time gates still hold, otherwise it enters sub-state 4. Sub-state 4 ends the group attack and returns the sister to idle. The waypoint is a live ring-position result, not a fixed route; the navigation path and total travel distance depend on the current position. The intended attack never starts in shipped play; its trigger and ring/dome details are in SPEC-0011. |
| 8 Flinch | Action 23, or action 24 when the variant flag is 1 | One-shot; back to state 0 when the animation finishes (Medium). In region 1123, action 23 maps to main clip 15 for all three sisters, the same clip requested by state 7 sub-state 3; action 24 maps to clip 16. |
| 9 Death | 26 | On completion: clear the hidden flag, stop the effect, mark the sister dead. The sister is then removed from the alive count. |

Effect ids (decimal): 460 for state 4; state 5 uses 462 for poison, 463 for curse and 461 for slow; state 6 uses 466 for thorn, 465 for leach and 464 for Mote-eater. The values are program constants, not data words.

Effect cleanup: when the sister's effect flag is clear, the effect it holds is released (Medium).

Alternate loops (classes 13, 15 and 16, Medium): when level flag 0 is set, each routine runs its alternate loop instead of the main loop. Classes 13 and 15 share loop B; class 16 uses loop A. Both use idle, move and energy-ball behaviour only (SPEC-0011). In both loops the move step uses the movement speed word doubled while a sister is avoiding the hero (SPEC-0011). Loop A writes the 3-alive movement speed (0x2DAB50) to 10.0 units per second, the stage 3 launch speed (0x2DAB94) to 0.5, and two other constants (1.5 and 3.0, used as the energy-ball cooldown range in seconds, SPEC-0011). Nothing restores these words, so they keep the written values for the rest of the session (High). Loop B makes no such writes. In either loop's idle state, it plays animation 1 and aims toward the target at a blend weight of 0.25.

Edge cases:

- The data word for the 3-alive movement speed is 7.5, but alternate loop A writes 10.0 into it at its start and nothing restores it. A main loop that runs after the alternate loop in the same session therefore also uses 10.0. The alternate loops run in the step-21 escape (pack 1174), which comes before the normal Sisters fight (pack 1122, step 43), so a continuous session fights with 10.0; a session restarted from a save made after step 21 fights with 7.5 (Medium-High).
- The stated variant pairs apply only when the variant flag is set; otherwise the default animation index is used.
- Alive count of 0 fires the region event once.
- State 9 sisters are dead and are not counted; the fight ends when the count reaches 0 (SPEC-0011).

### 5. Whinny (class 7) (High for modes and pen rules; Medium for the group sound)

Whinny is a group creature that wanders, flees the hero when it comes close, and is herded into pens. Each Whinny runs one of four modes, chosen in priority order: penned, lead, flee, wander.

Start: the three shared group counters are set to 0. The Whinny also takes its navmesh once, at start: the first ground navmesh (kind 1) of the world that holds its position, by the same rule as Behavior 2. No search is made later. The current navmesh is set directly at two moments: on entering lead, to the pen's entrance mesh (below), and on becoming penned, to the pen's inner mesh.

Mode priority each tick: penned, then lead, then flee, then wander. Only the first one that applies runs.

Speeds (High). The Whinny has no speed constant: it travels by the root motion of the clip of its current movement mode (SPEC-0040, Behavior 14D), at playback rate 1 (its coroutines never change the speed scale; only a slow status halves it and a stun stops it). Run (mode 2, animation 3) plays a looping 7-key clip whose root step is 922 × 0.000977 = 0.9008 per key straight ahead (no sideways part) with clip speed 0.25 keys per tick, so a Whinny runs 0.2252 units per tick, 13.51 units per second at 60 ticks per second; one 28-tick cycle covers 6.31 units. This is the hero's walking speed (13.51); the hero's run (19.29) is 1.43 times faster. Walk (modes 0 and 1, animation 2) plays a 9-key clip with 284 × 0.000977 = 0.2775 per key, so 0.0694 units per tick, 4.16 units per second. All Whinny records on the disc (the packs that place them) use the same two clips. The step is along the steered move direction, with no acceleration: a clip requested on tick n first moves the Whinny on tick n + 1, and every later tick moves a full step; a character that finishes a sub-step still moves in that same tick toward its old point, so the run is continuous across re-plans. The body radius is 1.175 (monster row, as the heroes).

Wander:

- If the hero's horizontal distance is below 5, switch to flee.
- Otherwise walk to a random point on its mesh. Movement mode 2 (run, animation 3) when the point is more than 10 units away, mode 0 (walk, animation 2) when it is 3 to 10 units away. Within 3 units it has arrived: it plays idle (animation 0) and a timer counts down.
- A new point and a new timer are drawn (timer uniform whole ticks in [120, 400]) when the timer has run out after arriving, or when its movement is blocked.
- If there is one pen and Whinny is inside that pen's inner area, it becomes penned.

Flee:

- First, if Whinny is inside the entrance area of an accepting pen, it switches to lead for that pen: the pen is marked occupied, the lead timer is set to 30 ticks, the pen point becomes its target, and its current navmesh is set directly to the pen's entrance mesh (no search, and the pen point lies on that mesh). The entrance-area test is a horizontal point-in-mesh test on the entrance mesh's triangles, whatever mesh the Whinny is on (High).
- Disc (resource 909): the start mesh of the three Whinnies (mesh 1, 9 triangles) overlaps the entrance mesh (5) horizontally, so a Whinny that walks its start mesh can enter the entrance area; the start mesh does not overlap the inner mesh (0). Test vector: a Whinny on mesh 1 that stands inside the entrance area while fleeing gets current navmesh 5 at once; its target is the pen point, which lies on mesh 5, so the lead walk can reach it.
- Otherwise run (mode 2). Flee ends (back to wander) when the hero's horizontal distance is above 8.
- Sub-step A: the candidate point is Whinny's position plus 4 units along the unit direction away from the hero (horizontal). "The path is clear" is a single test: the candidate must lie inside one triangle of the Whinny's current navmesh (the horizontal inside test of SPEC-0018, edges inclusive, every triangle of that one mesh tested, no meshes other than the current one). It is not a segment test against the boundary edges, not a world collision test and not a test of the straight line to the candidate; only the end point is tested. A candidate on no triangle of the current mesh (including a Whinny with no current mesh) fails the test. When the candidate passes and the navmesh-blocked latch is clear, run to it for at most 15 ticks or until within 2.5 units (from a target 4.0 away the run stops after 7 ticks, 1.576 units of travel, when 2.42 remain, so the step is re-planned every 7 to 8 ticks; the longest run is 15 ticks, 3.38 units). Otherwise (candidate off the mesh, or latch set) take sub-step B. The latch is the shared character latch: navmesh confinement sets it when it pushes a character off a boundary edge, or restores the last valid position after a failed triangle locate, and it also loads a 120-tick timer; the confinement step counts that timer down every tick the latch is set and clears the latch when it reaches 0. Besides the timeout, the flee step clears it whenever a flee sub-step ends (sub-step A ending by timer, by arrival, or by the latch being seen set during the walk, which also abandons the target; sub-step B ending by timer or arrival). The Whinny never changes mesh during flee except by the entrance rule above.
- Sub-step B: otherwise ask the mesh for a safe point and run to it for at most 25 ticks or until within 2 units. The mesh query is the random far-centroid pick of SPEC-0056, run on the Whinny's current mesh with the hero's position as the reference and distance 10: with T triangles it makes up to T draws, each draw a uniform whole number in [0, T-1] (draws are independent, so a triangle can repeat and some may never be drawn); a draw's centroid (mean of the three vertices) is accepted when its horizontal distance from the hero is strictly more than 10, and the first accepted one is the safe point. If no draw is accepted, the last drawn centroid is the safe point, however near the hero it is. With no current mesh the safe point is the hero's own position. So the point is a random far triangle centroid, not the first one in triangle order; a build that takes the first centroid in triangle order beyond 10 is deterministic where the original is not.
- Disc check, resource 909 (High for the speeds, the clip data and the edge rule; Medium for the herding rates, which depend on how the hero moves and are from a model of both). The three Whinnies start at (-1845.754, 1143.807), (-1865.531, 1135.022) and (-1853.762, 1143.985) (x, z); each lies on meshes 1 and 6, and mesh 1 is chosen because it comes first. Mesh 1 (9 triangles) and entrance mesh 5 (2 triangles, pen point (-1840.40, 1125.23)) overlap in a strip of about 31 square units along mesh 1's south-east edge, whose width across the edge is about 2.2 at the west end and 2.7 at the east end. The entrance test runs at the top of every flee tick, so a Whinny that stops inside the strip at any tick enters lead. Edge wall at these meshes: the Whinny's radius is 1.175 and the confinement rule of SPEC-0056 applies to the boundary edges of the triangle it stands in. On mesh 1 the south-east, south and east edges (the ones on the entrance side, with the world origin far outside them) have reach 1.175, and the north, west and north-west edges have reach 0. So a Whinny cannot come closer than 1.175 to mesh 1's south-east edge (a contact leaves it up to one step, 0.225, farther in), and the usable part of the overlap strip is 1.2 to about 2.2 (west end) or 2.7 (east end) from that edge, 1.0 to 1.5 wide. The strip is normally entered from farther out, while the Whinny is still 2.2 to 2.7 from the edge, so the edge wall rarely matters on the way in.

Worked example with a hero who keeps 4 units behind it on the line to the pen and a Whinny that is already running at full step (the hero's speed does not matter while he keeps the gap): Whinny at (-1845.754, 1143.807), hero at (-1846.86, 1147.65). Sub-step A: candidate (-1844.65, 1139.96) is on mesh 1, so it runs 7 ticks (1.576 units) and plans again at tick 8 from (-1845.26, 1142.08), candidate (-1844.15, 1138.23); at tick 16 it plans from (-1844.76, 1140.34), candidate (-1843.65, 1136.50), still on mesh 1, and at tick 23 it stands at (-1844.32, 1138.83), inside mesh 5, and enters lead. No sub-step B is used. A start at (-1853.762, 1143.985) makes five sub-step A steps (planned at ticks 0, 8, 16, 24 and 32) and then, at tick 40 with the candidate beyond mesh 1's south-east edge, sub-step B (a draw of triangle 6, centroid (-1869.01, 1133.16)); the Whinny is already inside mesh 5 at tick 43, (-1848.34, 1136.01), and enters lead. The start at (-1865.531, 1135.022) needs about three sub-step B detours (median 131 ticks, 90th percentile 229 ticks): its direct-away step fails within 4 units of mesh 1's south-east edge, the fallback goes to a random far centroid (usually away from the pen), and the next sub-step A steps bring it back toward the edge until one of them ends inside the strip. Ticks shift by a few if the Whinny starts from a walk or from rest (the first run step comes one tick after the clip request).

Herding rate. A hero who keeps 4 units behind the Whinny on the line to the pen (the hero walks at 13.51 or runs at 19.29) brings every one of the three Whinnies into lead in 300 of 300 trials per start (random sub-step B draws), walking or running. With a start angle up to 90 degrees off the pen line, a gap of 2.5 to 4.9 and a start delay of up to 4 ticks, 292 to 300 of 300 still reach lead; the few that do not are lost (a walking hero cannot close the gap after a long detour, the Whinny is more than 8 away, flee ends and it goes back to wander). A hero who only runs straight at the Whinny (stopping 3 units away) brings 300 of 300 into lead from the straight-line start for the first and third Whinny, and about 210 to 220 of 300 for the second within 3000 ticks (50 seconds); in the varied starts 225 to 252 of 300 do so, and the rest keep circling mesh 1 without being lost. Dropping the edge wall, or applying a 1.175 reach to every edge, changes these counts by less than 5 percent.
Conclusion: herding works reliably (Medium). A Whinny cannot be driven in a straight line, because the direct-away step fails within 4 units of mesh 1's edge and sub-step B sends it to a random far centroid. But it moves at the hero's walking pace and every re-plan of sub-step A carries it 1.6 units toward the pen, so a player who keeps behind it and slightly pen-side gets it into the entrance area in under 1 second (first and third Whinny) to a few seconds (the second); only a hero who stays more than 8 units behind after a detour loses it.

Lead:

- Run (mode 2) to the pen point on the current navmesh (the entrance mesh). Within 3 units (squared 9) it stops and checks where it is: if it is inside the pen's inner area (horizontal point-in-mesh test on the inner mesh, whatever its current navmesh) it becomes penned; otherwise it aims at the pen point again with a timer of 30 ticks and runs again.

Penned:

- Walk (mode 0) to a random point on the pen mesh. When it is within 1.5 units or blocked, it plays idle (animation 0) for a uniform whole number of ticks in [120, 400], then picks another point.
- A penned Whinny counts as speaking for the group sound.

Pens: up to four, set by the level script.

- Pen count: at 0x2DB250 (the data holds 1).
- Per pen i (i < 4): inner-area id at 0x2DB258 + 4i, entrance id at 0x2DB268 + 4i (the data holds -1 here), occupied flag at 0x2DB278 + 4i (4 bytes each). The pen point is set at runtime by the script setters below, from a locator.
- Two script setters exist (High):
  - SetWhinnyPenmesh(world, inner, entrance, locator k): one pen. Pen count = 1; pen 0 gets the inner-area id, the entrance id, pen point = locator k's point (all four lanes) and occupied = 0. Locator k is not range-checked. Disc: resource 909 (inner 0, entrance 5, locator 8).
  - SetWhinnySwitches(world, inner list, entrance list, locator list, n): a pen list, not a switch setter. Pen count = n; for each i < n: pen i gets inner[i], entrance[i], pen point = the point of locator list[i], and occupied = 0. The pen arrays hold 4, so n above 4 writes past them. Disc: resource 1398 passes n = 4.
- With count 1 the single pen accepts every Whinny. With count above 1 each pen takes one Whinny (the occupied flag is set when it takes one).

Group sound (High for ids, Medium for meaning):

- Each tick the group counts its members (reset when the world frame changes) and counts members with a "speaking" flag. Penned Whinnies are speaking.
- When the group count is 3, a sound is played: flagged count 0 plays sound 107, 1 plays 108, 2 plays 109, 3 plays 110. The sound is started through the sound player with a second argument of 1 (meaning not confirmed), unless that sound is already playing.

Turn weights: the values 0.1, 0.25 and 1.0 passed with a direction are per-tick turn-toward weights (1.0 snaps; Behavior 2).

### 6. Draik Skeleton (class 12) (High for stages, timers, attack choice, hit registration and bone mapping; Medium for hit-shape meaning and which live records use the pack 1096 breakable-object indices)

The Draik Skeleton is a timer-driven creature with three active stages (1, 3 and 5). Its death is stage 6. It has seven attack cooldowns, a breath beam (fire and ice), claw, spit, wind, stomp and bite attacks, the tail and orbit handlers of its attack set (Section 6.8), a transition at 66 % and 33 % HP, and breakable parts that fall off in the transitions and at death. Its flying height keeper (6.9) acts only in the unreachable stages 2 and 4 and after death, so in reachable play the Draik never flies.

Start: navmesh pick in mode 0 (ground mesh, kind 1) and mode 1 (flying mesh, kind 2) (Behavior 2). If no suitable mesh is found, a warning is raised (text not specified). The stage starts at 1 and every cooldown starts at the stage-1 range.

#### 6.1 Stages

- Stage 1 at start.
- Stage 3 after the first transition (the legs and tail break).
- Stage 5 after the second transition (the chest, arms and wings break).
- Stage 6 after death (the head breaks).
- Stages 2 and 4 are never entered in this build (High): the stage is set only to 1 at start and to 3, 5 and 6, never to 2 or 4. Their choosers and their stage-table entries are present in the program but unreachable (Open question 16).
- The level can force stage 5 with all nine parts broken when level flag 0x11 is set.
- A global pause flag puts the Draik in a paused state (28) until the flag clears.

#### 6.2 Cooldowns

Seven attack cooldowns, in ticks, each decreased by 1 per tick while above 0. Each is drawn as a uniform whole number of ticks in [60 x min, 60 x max] whenever it is set (at start and when its attack ends). The min and max come from 5-entry stage tables (index = stage - 1; the entries read on the disc are equal within each table; each address below is the first entry of its table). The drawn value is stored into its cooldown and counts down from there; it is not discarded (High for the storage, Medium-High for the use):

| Attack | Gates | Min and max (seconds, data addresses) |
| --- | --- | --- |
| Breath | fire and ice beams | 4 to 8 (0x2DA650, 0x2DA664) |
| Spit | fire and ice spit | 1 to 2 (0x2DA6DC, 0x2DA6F0) |
| Claw | claw | 1 to 3 (0x2DA72C, 0x2DA740) |
| Wind | wind hazard | 4 to 8 (0x2DA7A4, 0x2DA7B8) |
| Tail | tail (unreachable) | 2 to 5 (0x2DA81C, 0x2DA830) |
| Stomp | stomp | 2 to 5 (0x2DA894, 0x2DA8A8) |
| Bite | bite | 2 to 5 (0x2DA90C, 0x2DA920) |

The breath cooldown is also set after a reflected breath stun (see 6.5).

#### 6.3 Attack choice and movement

Attack choice runs only when the Draik is in a free state: idle, approach, facing or orbiting. Let d be the horizontal distance from the Draik to the hero. Breath, spit, claw, wind, stomp and bite use these distance bands (a band is open when min < d < max, with the stage-table values, all equal on disc):

- Breath: 10 to 20 (0x2DA678, 0x2DA68C). The breath keeps a standoff of 10 to 15 (0x2DA6A0, 0x2DA6B4) and aborts at 25 (0x2DA6C8).
- Spit: 5 to 15 (0x2DA704, 0x2DA718).
- Claw: 0 to 10 (0x2DA754, 0x2DA768); claw standoff 4 to 7 (0x2DA77C, 0x2DA790).
- Wind: 5 to 10 (0x2DA7CC, 0x2DA7E0); wind approach to 10 (0x2DA808).
- Stomp: 0 to 5 (0x2DA8BC, 0x2DA8D0); stomp approach to 7 (0x2DA8F8).
- Bite: 0 to 10 (0x2DA934, 0x2DA948); bite approach threshold is a fixed code value of 6.0, not a tuning word.

Choice by stage, first match wins unless noted, and only when the attack's cooldown is 0:

- **Stage 1**: breath (fire or ice, by coin) if 10 < d < 20; else spit (fire or ice, by coin) if 5 < d < 15; else claw if 0 < d < 10.
- **Stage 3**: breath; spit; wind if 5 < d < 10 (no early return); stomp if 0 < d < 5.
- **Stage 5**: spit; stomp if 0 < d < 5 (no early return); bite if 0 < d < 10, tested last, so bite replaces stomp when both are ready.

Movement when no attack starts (facing cosine c = dot of the horizontal unit vector from the Draik to the hero with the Draik's forward direction):

- Stage 1: approach when d >= 5 (0x2DA988 = 5.0). Otherwise, if c <= 0.5, face the hero; a facing state ends only when c > 0.95. Otherwise idle.
- Stages 3 and 5: approach when d >= 10 (0x2DA9A4 and 0x2DA9C4 = 10.0). Otherwise, if c <= 0.5, face the hero, until c > 0.95 as above.
- Approach: nav-mesh move with the stage's locomotion animation (2, 27 or 50 for stages 1, 3 and 5). Facing turns with blend 0.1 and plays the turn animation (10, 24, 34, 47 or 55 by stage).

Stage tables for the unreachable stages 2 and 4: the stage-2 band pair is 5.0 (0x2DA844, the minimum) and 10.0 (0x2DA858, the maximum). The stage-2 chooser reads these, but stage 2 is never entered (Open questions).

#### 6.4 Vulnerability and hit registration

- Each registered hit removes floor(0.025 x max HP), the whole part of max HP x 0.025, from the Draik's HP. For a max HP of 1000 that is 25 per hit.
- A melee swing counts once. A hero projectile counts each time it hits, and is removed.
- Stage 1: hits count only while the Draik is stunned (state 26), and not during its hit animation, on any body part.
- Stage 3: hits count only while stunned (state 27), on the wings, arms, body or head (groups without the legs).
- Stage 5: hits count at any time, on the head only.
- Body groups and their bones (High): left leg 25, 27, 28; right leg 26, 32, 33; wings 37, 47, 48, 49, 50, 51; left arm 9, 10, 11; right arm 17, 18, 19; body 1, 2, 3, 4, 7, 54, 57; head 54, 57. A group counts when any of its bones passes. Stage 1 tests all seven groups, stage 3 every group except the two legs, and stage 5 only the head group, so bones 54 and 57 belong to both the body and the head group. A bone test counts when the hero's melee attack is active and the hero's weapon volume overlaps a cylinder of radius 1.0 and length 3.0. The cylinder starts at the bone's origin and runs along the bone's own second axis (its local Y axis in world space), so its centre is 1.5 units from the bone origin along that axis (axis choice Medium-High). Bone 7 is the one bone whose cylinder radius is 2.0 (High); it is in the body group, and it is also the bone where the head part is placed when it breaks. A hero projectile counts when it lies within 5.0 plus its own radius of the bone (and is removed). These radii and lengths are fixed constants.
- Hit reaction: stage 1 plays animation 23; stage 3 plays animation 46; stage 5 plays animation 54. When the reaction ends, the stun resumes (the stun countdown keeps running during the reaction).
- After the stage-1 reaction: if the HP fraction is at most 0.66, the Draik posts level event 0 and enters transition 21 (animation 15): it breaks the left leg, right leg and tail, and goes to stage 3.
- After the stage-3 reaction: if the HP fraction is at most 0.33, the Draik posts level event 0 and enters transition 23 (animation 39): it breaks the chest, left arm, right arm, left wing and right wing, and goes to stage 5.
- A stage-5 hit that would reduce the HP to 0 or below kills it: dead state 20 (animation 56). When the death animation finishes, the head breaks, level event 12 is posted, and the stage becomes 6. The event repeats each tick while the finished animation is held (Open questions).

**Breakable parts (High for the record layout and bone mapping; Medium for which live actor receives pack 1096's object-index configuration).** Each of the nine records stores a broken flag, a part object index read from the matching Draik index query, and a fixed bone index used to position the broken part. Setup does not provide fixed defaults for the object indices. The pack 1096 level module sets the nine indices during initialization in setter order (left leg, right leg, tail, chest, left arm, right arm, left wing, right wing, head): 17, 20, 22, 15, 16, 19, 18, 21 and 23. These indices are held as one shared current configuration, not in a per-actor table; each Draik setup reads the values current at its nine queries. The module's Init runs in the pack's load-completion step, before any region character of that world is bound (SPEC-0030, Init order), so the Draik of pack 1096 (entity ordinal 0) starts its controller after the nine values are set and reads 17, 20, 22, 15, 16, 19, 18, 21 and 23 (Medium-High). Pack 1096's Treasure Vault is the only place a Draik is reached in play; the second Draik record on disc is not reached by any route of the game.

| Part | Bone index |
| --- | --- |
| Left leg | 25 |
| Right leg | 26 |
| Tail | 4 |
| Chest | 1 |
| Left arm | 9 |
| Right arm | 17 |
| Left wing | 38 |
| Right wing | 5 |
| Head | 7 |

Breaking a part places its configured object at the corresponding bone, spawns effect 0xFC0 and releases it.
#### 6.5 Breath and reflection

- The breath keeps a distance of 10 to 15 from the hero (approaching if farther than 15, backing off if closer than 10: each tick it walks toward the point 5 units behind its own position, along the horizontal direction from the hero to the Draik and at its own height, and the target is recomputed every tick, so it keeps backing away until it is 10 or more from the hero; the 5 is a fixed constant, High). It then plays its windup animation (3, 16, 28 or 41 by stage 1 to 4), and fires a beam from the mouth for a duration drawn as a random whole number of ticks from 2 x 60 to 5 x 60 (inclusive). The fire and ice ranges are the pairs at 0x2DA9C8 and 0x2DA9CC (fire) and at 0x2DA9D8 and 0x2DA9DC (ice); each pair holds 2.0 and 5.0 seconds. The pairs are the beam duration in seconds (High; Open question 14). The beam's end moves toward the hero, 1.5 units above the feet, at 0.25 units per tick (fixed constants). The beam stops when d reaches 25 or its time runs out. It then plays its recovery animation (5, 18, 30 or 43 by stage).
- Beam length (Medium-High): each beam keeps a length l, 0 when it starts, and a target t, the 3-D distance from the beam start (the mouth) to the beam end, recomputed every tick. When l < t, l grows by the rate 1.0 (0x2DA9E8) but not past t. When l >= t, l becomes t at once. The beam is drawn along the normalised start-to-end direction, with the world up vector as its reference.
- Beam hit: 10 damage (fire 0x2DA9D4, ice 0x2DA9E4; both words hold the integer 10, read from disc), then 60 ticks before the next hit (at most 10 per second). The two words are the damage amount passed to the hero's damage routine, with the Draik as the attacker; the same damage call is used by other enemies (High for the call shape; Medium-High for the meaning). They are not a sound or animation id (Open question 34).
- Beam hit test (Medium-High): each tick of the beam, the segment from the mouth to the beam's end point (the moving end point, not the grown length l) is tested against a vertical cylinder centred on the hero's position raised 1.5, with its axis along world up, height 1.75 (0.875 each way) and radius 5.0. The segment crossing the cylinder is a beam contact; the reflection test and the hit below follow.
- Reflection: the hero must be blocking, face the beam (dot above 0), and have an eligible Mote in equipment word 1 (the shield-Mote slot). The Draik maps the equipped Mote id to an element kind: the fire beam reflects for Mote ids 160–161 (kind 1) or 172–173 (kind 7), and the ice beam for ids 162–163 (kind 4) or 172–173 (kind 7). These numeric values are Mote element selectors, not shield-family item categories. The shield-family gear (item config types 2 and 5; ids 106–110 and 176–180) does not satisfy this Mote-element test by itself. On reflection, a ball travels from the beam end back to the Draik chest at 0.25 per tick. On arrival the Draik is stunned for 6 s (360 ticks) in state 26 (stage 1) or 27 (stage 3), and its breath cooldown is set to a random value (stage 1) or twice a random value (stage 3). The stun animations are 12 and 36 (High; Open question 11).

#### 6.6 Other attacks

- Spit (attack 8 for fire, 9 for ice; one handler each, Open question 33): the stage-1 chooser picks one by coin. At the spit animation's event, one projectile is fired from the mouth at the hero, aimed 2 units above the hero. It travels 1.0 unit per tick (fixed constant), deals 10 damage on hit (radius 0.5), and is removed on touching the ground. The spit is not a projectile of the enemy pool (Behavior 9): the Draik keeps two spit records of its own, one for fire and one for ice, so it has no pool behaviour kind, visual kind, element or lifetime counter. It is released on the first tick on which animation event bit 0x20 is active, once per attack, at the mouth point (the Draik's bone 52), in the direction of the aim point, 1.0 per tick (words 0x2DA9E8 and 0x2DA9F8), damage 10 (words 0x2DA9F4 and 0x2DAA04) with no attacker id. Each tick a sphere of radius 0.5 (word 0x2DAA6C) is tested against the hero's body capsule; a hit damages the hero and ends the spit. It also ends when its height reaches the ground height under it, found with the ground probe started 8 units above it. No lifetime limit exists (High for the record; Medium for the lack of a lifetime). At the end of the attack the spit cooldown is a random 1 to 2 seconds (0x2DA6DC = 1.0 and 0x2DA6F0 = 2.0, read from disc), and the state returns to 0. Spit animations by stage: 6, 31 and 52 (stages 1, 3, 5; stages 2 and 4, which are unreachable, use 19 and 44).
- Spit appearance (High for the effect ids and the moments they start and stop; Medium for the look, because the sprite pictures themselves are not described). The spit is drawn only by particle effects of the shared effect group (SPEC-0034); no mesh and no light belong to it. Each of the two spit records owns two effects bound at level start: a flight effect and an impact effect. Fire uses flight id 528 and impact id 991. Ice uses flight id 527 and impact id 992. All four have three emitters, a near/far threshold of 100 units, no cull distance, and no emitter that is gated to the far state.
  - Timing. At the release tick both effects are moved to the mouth point and only the flight effect starts. On every tick while the spit flies, both effects are moved to the spit's position. When the spit hits the hero or reaches the ground, the flight effect is stopped (no new particles; the living ones finish their life) and the impact effect starts at the spit's final position as a single burst. The same tick that ends the spit does this once.
  - Fire flight (528), all three emitters ride the spit's position; the particles of the glow and the sparks also follow its motion, while the core sprites stay where they were born: a large red additive glow sprite (size 5.0 shrinking to 0 over its 0.5 s life, one every 0.2 s, spinning 30 rad/s, green channel flickering between 0 and 1 so the colour runs red to yellow and back); a white additive core sprite (size 2.5 shrinking, life 0.2 s, one every 0.01 s); white alpha-blended sparks (2 every 0.1 s, size 0.4 growing, life 0.5 s, thrown at 5 units/s in every direction and pulled down at 2 units/s^2).
  - Ice flight (527): the same three-part layout in blue. An additive glow sprite (size 2.0 shrinking, 2 every 0.2 s, life 1.0 s, start colour (0, 0.5, 0.75)); small white alpha sprites (size 0.6 shrinking, one every 0.02 s, life 0.5 s, speed 1.5, drifting up at 0.4); white alpha sparks (size 0.4 growing, one every 0.1 s, life 0.5 s, speed 5, pulled down at 5).
  - Fire impact (991), one burst of red additive sprites: 3 sprites of size 0.5 growing (life 0.9 s, speed about 1, upper hemisphere); 2 large sprites of size 5.19 shrinking (life 0.7 s, almost still); 10 small sprites of size 0.5 growing (life 0.8 s, speed 2 in every direction).
  - Ice impact (992), one burst: 5 blue alpha sprites of size 0.5 growing (life 0.9 s, speed 1, upper hemisphere); 5 white alpha sprites of size 0.37 (life 0.5 s, speed 10 with drag 3, fading out); 7 blue additive sprites of size 1.48 growing (life 1.0 s, still).
  - Test vector: a fire spit released at the mouth point (0, 3, 0) starts effect 528 there and nothing else; on the tick it touches the hero (or its height reaches the ground height found from 8 units above) effect 528 stops and effect 991 starts at the spit's position. An ice spit does the same with 527 and 992. The two differ only in these effects; damage 10, speed 1.0 and radius 0.5 are the same.
- Claw (stage 1, animation 7): one hit test per attack, 20 damage (fixed constant).
- Wind (stage 3, animation 31): approaches until d <= 10, holds 5 to 10 s after a 1 to 2 s delay, and at the animation's event (bit 0x20) spawns a ground hazard living 7 to 10 s. The hazard starts at the Draik's own position (x and z), with its height set to the ground height under it from the ground probe started 8 units above; its first target is set at once as described below (High). Data: hold 5 and 10 at 0x2DAA20 and 0x2DAA24; delay 1 and 2 at 0x2DAA34 and 0x2DAA38; hazard life 7 and 10 at 0x2DAA3C and 0x2DAA40. The pairing by word order is Medium. The Draik then idles for 10 to 12 ticks (float words at 0x2DAA2C and 0x2DAA30, Medium; the idle time is not multiplied by 60; see Edge cases).
- Hazard: 4 slots. The hazard slides toward a target at 0.1 per tick (0x2DAA70). The target is the hero's position plus random offsets in [-5, 5] on x and z, renewed every 60 ticks (range endpoints and renewal interval are fixed code values). Within 4.0 units of the hero (fixed code threshold), it deals 10 damage (0x2DAA48, int) and throws the hero. The throw replaces the hero's velocity with 10.0 (word 0x2DAA44) times the unit vector of (0, 12, 0) + 9 u, where u is the horizontal unit vector from the hazard to the hero; the 12 and the 9 are fixed code constants. The throw therefore has a vertical speed of 8.0 and a horizontal speed of 6.0 away from the hazard, whatever the hero's distance (High; u horizontal: Medium-High). The hero enters its launched action state (48) once.
- Tail (state 12; dormant stage-2 branch): the stage-2 chooser checks it after breath, spit and claw, when its cooldown is ready and horizontal distance is strictly between 5 and 10. Its first sub-step uses the stage-2 flying-approach rule with 5 and 10 as the target-distance bounds. After arrival, it requests action 21 until the action is current and finished, then returns to idle and draws the tail cooldown from 2 to 5 seconds. This handler has no separate hero-hit or damage operation; whether action 21 has an externally handled hit event is unknown. Stage 2 is never entered in the shipped fight.
- Stomp (stage 3 animation 32, stage 5 animation 53): approaches until d <= 7 (0x2DA8F8); no damage is applied (only a latch at the event).
- Bite (stage 5, animation 51): approaches until d <= 6.0 (fixed code value). One hit: 10 damage (0x2DAA68, int), plus a call to the hero with 0.0 (its meaning is open).

#### 6.7 Animation ids by stage (1 / 3 / 5)

- Idle: 1 / 26 / 49.
- Breath windup, beam, recovery: 3, 4, 5 / 28, 29, 30 (stage 1 and stage 3 values; the beam and recovery sets for stage 5 are not listed).
- Spit: 6 / 31 / 52. Claw: 7 (stage 1 only). Wind: 31 (stage 3). Stomp: 32 / 53. Bite: 51.
- Turn: 10 / 34 / 55. Stun: 12 (stage 1), 36 (stage 3). Hit: 23 / 46 / 54. Dead: 56.
- Transition animations: 15 (to stage 3), 39 (to stage 5).

#### 6.8 Attack set and dispatch (dispatch High; handler behaviour Medium)

The Draik's attacks are chosen by the attack choice of 6.3 and run by handlers. The Draik's main loop dispatches on an attack number:

| Attack number | Attack | Described in |
| --- | --- | --- |
| 6 | Fire breath | 6.5 |
| 7 | Ice breath | 6.5 |
| 10 | Claw | 6.6 |
| 11 | Wind | 6.6 |
| 12 | Tail (not reached in the disc flow) | 6.2 and 6.3 |
| 13 | Stomp | 6.6 |
| 14 | Bite | 6.6 |
| 8, 9 | Spit, fire and ice | 6.6 |
| 18 | Orbit, two entries (stage-2 and stage-4 values; unreachable) | this section |

Spit (8 and 9) is chosen by the attack choice of 6.3 (Open question 33).

Each handler runs from an attack record that holds a sub-state, a counter and an attack index. When a handler ends, it sets the Draik's top-level state to 0 (idle). The stun states 26 and 27 are entered only by reflection in the breath handler, and they also return to 0 when they end (High; Open question 32).

Orbit (attack number 18). The stage-2 entry draws an orbit radius from 7.0 to 14.0 and a flying-height offset from 5.0 to 15.0. The stage-4 entry uses the same ranges. Both entries set state 18 and choose an initial side. During orbit, the controller steers radially until its horizontal distance from the hero is within 0.5 of the drawn radius; then its tangential target advances 0.1 units per tick along one of two normalized perpendicular directions. It turns toward the hero with a 0.1 blend, or a 1.0 blend when the facing dot is above 0.9. The stage-2 branch requests actions 12/13 and the stage-4 branch actions 36/37, depending on whether a navigation target is available. The flying-height offset feeds the height keeper (6.9). These details describe dormant code: orbit is unreachable because only the stage-2 and stage-4 choosers select it, and no code path writes either stage. The orbit handler has no explicit top-level state exit, and its outside exit condition is unknown (High for the observed motion inputs and action ids; Medium for the navigation-target geometry). The per-stage chooser ownership and the meaning of both drawn values are established in questions 89 and 90.

#### 6.9 Flying height keeper (Medium-High for the rule; High for the stage gate and the offset writers)

The Draik's main loop calls this keeper every tick. It changes only the height of the Draik, not its horizontal position.

1. Stage gate. In stages 1, 3 and 5 the keeper does nothing; it acts only in stages 2, 4 and 6 (High). Stages 2 and 4 are never entered (6.1), so in reachable play it acts only after death (stage 6).
2. Target height. The ground height under the Draik is found with the ground probe of SPEC-0018, started 8 units above the Draik. The target height is that ground height plus the Draik's flying offset, a 32-bit float of the Draik's runtime record. Only the stage-2 and stage-4 choosers (a draw from 5.0 to 15.0, 6.8) and the stage-2 and stage-4 flying approach (2.0, Behavior 12.12) write it, so no reachable path writes it before the keeper acts in stage 6 (Low for the value it then holds; Open question 59).
3. Current height. The Draik's current height is the vertical component of its position.
4. Rule. Let g be the absolute difference between the current height and the target height.
   - If g is less than 0.1, the height is set exactly to the target (a snap).
   - Otherwise, if the target is below the current height, the height changes by -0.1 this tick. If the target is above it, the height changes by +0.1 this tick.
   The 8-unit probe offset, the 0.1 threshold and the 0.1 step are fixed constants, not data words. A step of 0.1 units per tick is 6 units per second at 60 ticks per second.
5. Side effect. Each call also sets a flag on the Draik's movement record to 1. Its meaning is not known.

Confidence: High for the direct position-Y snap and +/-0.1 step; the non-snap helper adds the step to the motion record's position vector, not its velocity. Medium-High for the remaining target-height rule. High for step 1 (the stage gate) and for the writers of the flying offset; Low for the offset value after death (Open question 59).

#### 6.10 Base state, killing hit and row write (High)

- Each tick the Draik sets its base state to 20, then to 1 while its state is 1, 3, 26 or 27 or its stage is 5. The shared hit filter does not exclude base state 20, so this write alone does not make the Draik unhittable; what base state 20 does is make it a guarding target. With the Blocking Angle of 2.0 written at setup (below), hero swings against a guarding Draik are always blocked, and in base state 1 the guard is off (Open question 171; Medium-High).
- Its killing hit runs the shared hit-point subtract, so the shared death hook runs and sets base state 16 (Behavior 34). The Draik then puts its base state back to 1 and its own state to 20 (dead, 6.4). So the shared corpse step (Behavior 34) never applies to the Draik.
- At setup it writes 2.0 into its monster row's Blocking Angle word (+0x10C).

#### 6.11 Shipped Undead Draik encounter progression (High)

The active fight uses monster-table row 3 (type 3, variant 0), which starts at 200 HP. The per-hit subtraction is floor(0.025 x maximum HP), so each accepted Draik hit removes 5 HP. This gives a concrete progression for the fight in pack 1096:

| Phase | Starting HP | Vulnerability | First HP value that advances the phase | Accepted hits in this phase |
|---|---:|---|---:|---:|
| Stage 1 | 200 | During state 26 stun, any registered body group | 130 (65% of maximum; threshold is at most 66%) | 14 |
| Stage 3 | 130 | During state 27 stun, excluding both legs | 65 (32.5% of maximum; threshold is at most 33%) | 13 |
| Stage 5 | 65 | Any time, head group only | 0 or less | 13 |

The stage-1 and stage-3 hit reactions finish before their HP thresholds are checked. A crossing starts the corresponding transition animation and breaks the listed parts: both legs and tail for the first transition, then chest, both arms and both wings for the second. The lethal stage-5 hit starts the death animation; when it finishes, the head breaks and level state 12 is posted. At full starting HP, no healing, and with every accepted hit landing, the fight takes 40 registered hits in total. Stage-1 and stage-3 hits require a reflected breath to enter their 360-tick stun windows; the hit reaction runs within that window and the stun timer continues to count down. Stage 5 needs no reflected-beam stun, but its hit test still accepts only the head group.

The pack 1096 activation path runs only while game flag 45 is clear. On world activation it plays movie 51, moves the hero to locator 10, activates entity 0, and sets boss-fight instance 6 to stage 0. The level thread consumes the Draik's threshold/death events; state 12 starts the finale, which ends instance 6 at stage 1 and sets flags 45 and 316. The exact scene actions and item dispensers are specified with the level module. Stages 2 and 4, including orbit and missile-dodge behavior, are not part of this shipped encounter.

Test vector: row 3 starts at 200 HP; after 13 accepted stage-1 hits it has 135 HP and does not meet the transition threshold; after hit 14 it has 130 HP and the transition check passes once reaction 23 ends. From 130 HP, 12 stage-3 hits leave 70 HP; hit 13 leaves 65 HP and passes its threshold once reaction 46 ends. From 65 HP, 12 head hits leave 5 HP; the thirteenth reaches 0 and starts the death sequence.

### 7. Class 37 (Darkest Faerie family, three-way entry dispatcher) and class 38 (Queen Fyora controller) (High for the routing and the loops; Medium where marked)

Class 37 is the entry dispatcher of the Darkest Faerie family. It routes each object to one of three loops. The Darkest Faerie fight, with its phases and attacks, is described in its own boss spec (SPEC-0054); this section gives the routing, the two alternate loops and their link to class 38. The class 38 controller is described in Behavior 30.

**Placed records.** Class 37 uses monster type 56 (rows 128 and 129, hit points 5 each): six placed records, five named "Darkest Faerie" or "The Darkest Faerie" (resources 1108, 1175, 1349, 1428 and 1432) and one unnamed (resource 1263). The unnamed record selects variant 0 / row 128, the same type and variant used by the named Darkest Faerie records; its game-defined character type is therefore supported as Darkest Faerie. The placement data does not establish a distinct encounter role for resource 1263.

**Class 37 routing.** The class 37 routine selects one of three paths, once, when it starts, in this order:

1. If game state 364 (progress flag 0x16C) is set: run the four-member controller (below).
2. Otherwise, if level state 18 (0x12) is set: run alternate loop A.
3. Otherwise: run alternate loop B.

Game state 364 is set by the init of script pack 1432 (the final fight) and level state 18 by the init of script pack 1174 (SPEC-0054, Routing). So the final fight runs the four-member controller, the earlier encounter runs loop A, and the Queen Fyora encounter (script pack 1428) runs loop B (Medium for the pack of loop B: it is the only class-37 placement that has a class-38 partner in its story step).

**Four-member controller.** Its eleven codes (0 to 10), its chooser and its four projectile slots (types 0, 1 and 2: elemental orb, aimed bolt, reflected orb) are specified in SPEC-0054 (Member codes and timings, Projectiles). The slot records are four 0x210-byte records at 0x36B640 (run-time memory), advanced by the slot handlers of Behavior 19. The code numbers used by its routines are animations and level states (Behavior 2c). Its descent codes 8 to 10 are in Behavior 2e.

**Alternate loop A** (level state 18 set, game state 364 clear). It plays animation 0 at start. Each tick a chooser runs first:
1. A countdown above 0 is lowered by 1.
2. If the state is not 1: when the horizontal distance to the hero is below 10.0 (fixed constant) and the countdown is below 1, the state becomes 1; at 10.0 or more the state becomes 0.

State 0 plays animation 0. State 1 is a knockback of the hero, done in one tick:
1. The countdown is set to 180 ticks (3 s).
2. The hero's velocity is reset and its upward speed is set to 12.0 (fixed constant).
3. Let u be the horizontal unit direction from the Faerie to the hero, and g = (horizontal distance) - (her radius + hero radius + 0.1). When g is below 0, the hero is moved by -5.0 x g x u (pushed out by five times the overlap; Medium).
4. The hero's horizontal velocity is set to 9.0 x u (fixed constant). The whole velocity is then scaled to length 15.0 (1.5 x the word 0x2DA630 = 10.0).
5. The hero enters hero state 48 (SPEC-0033), one more helper runs on her (an effect or a sound; Medium-Low), and the state returns to 0.

**Alternate loop B** (both clear): the duel with Queen Fyora.
1. Wait, one tick at a time, until the character list holds a character whose monster row's default class is 38 (the kind-38 search). Then set base state 20.
2. Each tick: run the loop B chooser; turn toward that class-38 object with blend 1.0 (except in state 5); run the state; update her effect group.

Loop B states (animation numbers are her own table's):

| State | Animation | Rule | End |
| --- | --- | --- | --- |
| 0 | 6 | Idle. | Chooser. |
| 2 | 10 | Deflect: on the first tick the animation is current and not finished, relaunch the first active type-1 slot (Fyora's orb) as type 3 with effect 4008 (Behavior 11, rule 4). | State 0 when the animation ends. |
| 3 | 7 | Cast at Fyora: at the animation's event, once, open a type-0 slot at her sample point (Behavior 19). | State 0 and a cooldown of a whole number of ticks in [30, 45] when the animation ends. |
| 4 | 9 | Knockback: on the first tick the animation is current, the knockback of loop A steps 2, 4 and 5 (no push-out), scaled to length 10.0 (word 0x2DAAB8), and a burst effect 0.5 units above her. | State 0 when the animation ends. |
| 5 | 7 | Summon cast: at the animation's event, once, open a type-2 slot at her sample point. Each tick she turns with blend 0.25 toward the hero (Medium). | State 0 and a cooldown in [120, 240] ticks when the animation ends. |

Loop B chooser, in order:
1. A cooldown above 0 is lowered by 1.
2. If a type-1 slot is within 1.5 units of her (horizontal): state 2, and the deflect flag is cleared.
3. Otherwise, if the hero is under 15.0 units away (horizontal; word 0x2DAAB4): state 4, and the knockback flag is cleared.
4. Otherwise, if the state is 0 or 2, no type-0 slot is active and the cooldown is below 1: state 5 if the summon test passes, else state 3; the event flag is cleared.

Summon test (Medium-High): it passes when level state 2 is clear, a random whole number modulo 4 is 0 (one chance in four), and no character in the list that she summoned is still alive. State 1 is never entered in loop B.

**Class 38: Queen Fyora controller.** Behavior 30. It waits for a class-37 object, faces it, and shares its four typed slots with loop B: the two throw orbs at each other, deflect them, and the deflected orbs can hit the hero (Behavior 19).

**Kind search (High).** The two searches walk the level's character list and return the first character whose monster row's default class (row offset 0x36) is 37 or 38; they do not test whether it is alive. The class 38 controller calls the class-37 search; loop B of class 37 calls the class-38 search, which finds Queen Fyora. Neither is the boss of Behavior 8.
### 8. Faerie Sisters encounter latch (High for owner, gates, order and event kinds; Medium for the story role of progress flag 220)

The routine is the encounter-start gate shared by the Faerie Sisters controllers (classes 13, 15 and 16). Its local kind is 0, 1 or 2 for the three variants. It is called once per pass by the three alternate loops only (SPEC-0011, Controller routines), never by the main loops, so it runs only in a world whose level state 0 is set (in the shipped game, pack 1174). It runs while progress flag 220 is clear; the flag is an Act 1 stage-2 progress flag, but its narrative reason for suppressing this encounter is unknown.

1. The first hero-side query runs before checking the encounter latch. If it returns null, the latch is cleared, even if it had been set.
2. Only when the first query is non-null and the latch is clear does the routine run the second hero-side query and the encounter reach test. The second query must also return a value.
3. The first query is the hero's cached attack-active predicate: it reflects event bit 0x1 on the current animation frame and is refreshed at the start of the hero update (SPEC-0036). Region characters run before the selected hero in the world pass (SPEC-0026, step 8.1), so a sister reads the cached value left by the prior pass; the selected hero's refresh later in the current pass becomes visible to sisters on the next pass. The separate weapon-reach test reads the hero's attachment-0 pose as the hero's own swing does: the axis D1 from the animation pass of the previous tick and D2 from the pass before it (SPEC-0036, Hit detection), because a sister's think runs before the hero's update and animation pass of the same tick (SPEC-0007, item 6) (Medium-High).
4. The reach test checks whether the hero's active weapon volume reaches the sister: the shared single-weapon test applies a 65-unit distance gate and tests the hero's attachment-0 cylinder against the target's bone capsules, with a swept test when the weapon tip moved (SPEC-0036).
5. When all conditions pass, the latch is set and an encounter event is raised. Kind 0 posts event kind 1, kind 1 posts event kind 3, and kind 2 posts event kind 2. Kinds outside 0 to 2 set the latch but post no event.

Edge cases: when the latch is set, successful tests do not post again; a later null result from the first hero-side query clears it and rearms the gate. With progress flag 220 set, none of the queries, latch updates or event posts run. An unexpected kind outside 0 to 2 leaves the latch set without posting an event, so it will not retry until a later null first-query result clears it.

**Synthetic check.** With progress flag 220 clear, a non-null first query, clear latch, non-null second query and a successful reach test, kind 1 sets the latch and posts event kind 3. If the first query later returns null, the latch clears without a post. With kind 3 and all tests successful, the latch sets but no event is posted.

### 9. Enemy projectile pool, per-tick update and hit tests (High for the pool, the lifetime, the motion and the kinds; Medium-High for the hit and world tests)

**Projectile pool (High).** The game keeps one pool of ten enemy-projectile slots, shared by all enemies and bosses. The slots are 0x15D0 bytes each, in run-time memory (the pool's base pointer is the run-time word at 0x35C8D8, not on the disc). A new projectile takes the first free slot in index order, 0 to 9 (a slot is free when its active word is zero). If no slot is free, the launch does nothing: no slot changes and no projectile is created. All ten slots are reset when a world starts. The enemy record holds no projectile slots of its own.

The pool is updated once per tick, slots 0 to 9 in order, inside the world branch of the main loop (High): after the region passes of the current and second worlds and the placed-effect lists, and before the common-enemy attack manager (SPEC-0038), the world item pool and the selected hero's update (SPEC-0007). Like the rest of the world branch it does not run while the game-state code is 9 (pause and the other freezing screens) or the world is disabled, so projectiles hold still under pause. A second, draw-side step runs in the draw pass (Visual update, below); it only places the visuals and has no lifetime, motion or hit logic.

**Slot layout (High for the offsets; meanings as listed).** 0x15D0 bytes per slot. Unlisted bytes hold the effect objects and their state.

| Offset | Type | Meaning |
| --- | --- | --- |
| +0x04 | s32 | Trail model-part pointer for visual kinds below 5; zero means no part |
| +0x0C | s32 | Behaviour kind (-1 when free) |
| +0x10 | s32 | Lifetime counter, ticks |
| +0x14 | s32 | Starting lifetime, ticks |
| +0x18 | s32 | Active word (1 live, 0 free) |
| +0x1C | f32 | Damage |
| +0x20 | f32 | Radius |
| +0x30 | vec4 | Velocity, units per tick |
| +0x40, +0x50, +0x60 | vec4 | Orientation axes (+0x60 = forward) |
| +0x70 | vec4 | Position |
| +0x80 | vec4 | Start position |
| +0x90 | f32 | Spin rate, radians per tick (row 0x31C + 4k) |
| +0x1590 | s32 | Area mode flag |
| +0x1594 | f32 | Area radius (row 0x45C + 4k) |
| +0x1598 | s32 | Area duration, ticks (row 0x44C + 4k, truncated) |
| +0x159C | s32 | Area pulse timer, ticks |
| +0x15A0 | s32 | Target character id (kinds 2 to 7) or spawn key (kind 8) |
| +0x15A4 | s32 | Ignore-world flag (row flag 0x10) |
| +0x15A8 | s32 | Visual kind (row projectile type 0x43C + 4k; 23 for spells and lobbed summons) |
| +0x15AC, +0x15B0, +0x15B4 | s32 | Status id (-1 none), status chance (percent), status value |
| +0x15B8 | s32 | Element (row 0x3C) |
| +0x15BC | s32 | Hit selector: 1 = the hero (default), 0 = the owner |
| +0x15C0 | s32 | Owner id (-1 none) |
| +0x15C4 | s32 | Hold word, -1. Nothing is known to set it; a value other than -1 freezes the slot |
| +0x15C8 | s32 | Breaks-objects flag |

**Setup record and submit (High).** A spawner fills one setup record of 0x98 bytes and submits it; the submit copies it into the slot it claims. Record fields: +0x00 behaviour kind, +0x04 visual kind, +0x08 element, +0x0C owner id, +0x10 start point, +0x20 direction, +0x30 velocity, +0x40 lifetime (ticks), +0x44 damage, +0x48 radius, +0x4C trail asset, +0x50 trail model-part pointer, +0x54 roll, +0x58 spin rate, +0x5C to +0x70 six effect ids, +0x74 area radius, +0x78 area duration, +0x7C target id or spawn key, +0x80 ignore-world flag, +0x84, +0x88 and +0x8C status id, chance and value, +0x90 breaks-objects flag, +0x94 hit selector. Defaults: hit selector 1, ids -1. On submit, visual kinds 5 and 7 to 14 replace effect ids 0 and 4 with 410 and 412; visual kind 6 uses 900 and 412 (program constants). The launch frame is built from the direction and the roll (Medium).

**The six effect ids (High).** Each is an effect of the shared effect group (resource 9; SPEC-0034), or -1 for none. Id 0 and id 1 are the flight effects: both are placed on the projectile's frame every drawn frame (Visual update). Id 2 plays at a blocked or reflected hit on the hero. Id 3 plays at lifetime expiry. Id 4 plays at a world impact (the kind-1 landing that starts area mode, the ground hit of kinds 2 to 8, and the ground hit of the other kinds when the ignore-world flag is clear). Id 5 plays at an unblocked hit on the hero. An id of -1 plays nothing. Examples: the Faerie Sisters' energy ball has only id 0 = 460 (SPEC-0011); Juppie's spit has 1400, 1400, 1399, -1, 1399, 1399, so its expiry shows nothing (SPEC-0015).

**Behaviour kinds (High).**

| Kind | Meaning |
| --- | --- |
| 0, 10 to 16 | Straight shot. Kinds 14 (mote eater) and 15 (leach) also run an extra effect after an unblocked hero hit (below) |
| 1 | Straight with gravity; enters area mode when it lands |
| 2 to 7 | Spell (straight; touches only its target character; deals no damage) |
| 8 | Lobbed summon (gravity; touches no character; spawns a summon where it lands) |
| 9 | The Plague Serpent's poison cloud; its rule is in SPEC-0016 section 10 (High) |
| 17 | Homing |

**Per-tick update of a live slot (High).**

1. If the hold word is not -1, skip to the effect upkeep.
2. If the owner id is not -1 and the owner no longer exists, release the slot (step 7) and stop.
3. Run area mode if the area flag is set (below). This step does not gate the later steps: a landed kind-1 shot still runs its lifetime test and its kind step, so it keeps its motion and gravity rather than holding its landing point (High for the order; Medium-High for the motion).
4. If the lifetime counter is negative: stop the flight effects, play effect id 3 (lifetime expiry) at the position and reset the slot. No hit, and nothing else happens.
5. Otherwise run the kind step: (a) motion; (b) character hit test; (c) world collision. A release in (b) skips (c).
6. Subtract 1 from the lifetime counter.
7. Release: a kind-8 slot first spawns its summon (below). Then the slot is reset (kind and lifetime -1, active word 0, ids -1).

So a projectile with lifetime L moves L + 1 times and is reset at the start of its (L + 2)-th update. A lifetime of 0 still moves once.

**Motion (High).** Velocity is in world units per tick and is added to the position once per tick.

- Facing: if the spin rate is zero, kinds 1 and 17 point their forward axis along the velocity. Otherwise the side and forward axes turn by the spin rate each tick.
- Gravity (kinds 1 and 8): first vy becomes vy - g', then the move. g' = g / R^2 with g = 32.0 (the gravity of the physics block, run-time word 0x3A1A7C) and R = 60 (the tick-rate run-time word 0x368FA0). Both are written at start-up and are not on the disc, so g' = 32 / 3600 = 0.0088889 units per tick squared. After n ticks, y_n = y_0 + n vy_0 - g' n(n + 1)/2.
- Homing (kind 17): before the move, u = unit(velocity), w = unit(hero position + (0, 2, 0) - position), and the velocity becomes 0.75 u + 0.25 w (the 0.25 is the data word 0x2DD448, f32). The result is not rescaled to the launch speed, so the speed is |0.75 u + 0.25 w|, at most 1 unit per tick, whatever the launch speed was.

**Character hit test (High for the flow; Medium-High for the capsule shape).** A projectile tests one character per tick. The test is a sphere (centre = the projectile position, radius = the slot radius r) against the character's vertical body capsule: base point P (the character position), radius Rb and height Hb (the character's motion-record radius +0x08 and height +0x0C; High). It hits when the centre is within Hb/2 of the capsule's mid-height and its horizontal distance from the axis is below Rb + r, or when it is inside the end-cap region; on the axis the top reach is Hb + r. Below the base the end-cap result is not regular (Low; Test vectors).

- Kinds 2 to 7 test the character in the slot's target field. On contact: kind 2 sets the target's word +0xD14 to -1 (the reanimation countdown of SPEC-0038 Layout section 1; Medium for that link); kind 3 applies status 4 (strength); kind 4 applies status 5 (speed) (both at the default 60 s); kind 5 sets the target's HP to its maximum; kind 6 sets the target's variant to p - 1 for a random integer p from 1 to 6, re-selects its monster row and loads colour preset p (Edge cases; SPEC-0040 Behavior 26); kind 7 applies status 11 (poison attack, default 30 s; Behavior 27.1). The statuses go through the target's own apply method (Behavior 27.1). Then the slot is released. No damage.
- Kind 8 has no character test.
- Other kinds: when the hit selector is 1 the hero is tested; when it is 0 the owner is tested.

**Hit on the hero (Medium-High).** Skipped entirely while Roberta's attack sub-state word is 6 or 7 (the spins), 8 (the leap) or 13 (its landing) (SPEC-0036; High). On contact:

1. The shot is blocked when the hero's guard query (SPEC-0036, Hit detection) is true and dot(projectile direction, hero forward) < 0.
2. Blocked and reflectable (High; R-0034): the kind is 0 or 10; the visual kind is in {0 to 5, 7 to 14, 16, 23}; word 1 of the equipment holds an item (the shield Mote); and the element is nonzero and either equals that Mote's element kind or that kind is 7. A shot of element 0 is never reflected (it takes rule 3). The projectile is re-aimed from its position at the owner's position + (0, 2, 0) at unchanged speed. Kind 10 loses its damage. The hit selector becomes 0. The shield's charge is updated (reflection charge, below). Effect id 2 plays. The slot stays live.
3. Blocked, not reflectable: effect id 2 plays and the hero takes max(1, 0.25 D) with the blocked flag, where D is the defence-adjusted damage below. No status.
4. Not blocked: effect id 5 plays and the hero takes trunc(D). Then the status roll: when the status id is not -1 and a random integer from 0 to 100 (both ends included) is at most the status chance, the status is applied with the status value. Status 0 (stun) is not applied while the hero's stun is already active (High).
5. Cases 3 and 4 release the slot.

D = max(0, F(damage - S)), where S is the hero's shield reduction for the projectile's element and F applies the hero's armour and global damage factor (SPEC-0036). With no shield, no armour and a global factor of 1.0, D = damage.

**Reflection charge (High; R-0034).** Applied only when the hero's shield-family entry exists and word 1 holds an item. Let Q = 1 (the same power for both heroes and both Mote parities) and s be the shield entry's stat B for type 5, stat D for type 3, else 0 (SPEC-0036). The amount is round half away from zero of Q x s x 0.01 (add 0.5 and truncate, or subtract 0.5 when the product is negative). It is taken from the word-1 Mote's charge by the charge use of SPEC-0021, which removes the Mote when the amount reaches its remaining count. There is no minimum of 1 and no magic payment, and an amount of 0 still goes through the charge use. Test vectors (synthetic, type-5 shield in word 1): stat B 40 gives 0; stat B 49 gives 0; stat B 50 gives 1. A type-2 book (Roberta) has s = 0, so its reflection takes 0.

**Extra effects of kinds 14 and 15 (High).** These are the Faerie Sisters' mote eater and leach (SPEC-0011). After an unblocked hit on the hero and its damage (case 4):

- Kind 14 (mote eater; High, timing Medium-High): if the timed attack-family group (SPEC-0036) is not running, it is filled with Mote ids and then started. There is one such group for the whole game, a single global object; it is not owned by the hero.
  - Draws. The candidate table is the twelve values 160 to 171, at indices 0 to 11. Draw 1 takes a uniform index v from 0 to 11. Each of the seven later draws takes a uniform index from 0 to 11 and, while that index is already used, replaces it with (v mod 12) + 1 until it is new. Each index gives the candidate at that index. The eight is a fixed constant; the data word at 0x2DAB78 (s32, 8) is not read by this rule.
  - Index 12. The probing can reach index 12, which is accepted the first time it comes up. Index 12 reads the word just past the candidate table, and that word holds the first drawn index (0 to 11), not a Mote id. The group add refuses any id outside 160 to 173, so that draw adds nothing and the group holds 7 entries. Exactly 116,239,111 of the 12^8 = 429,981,696 equally likely draw sequences reach index 12, so 27.03 % of groups hold 7 entries. Index 0 (Fire, 160) can come only from a direct draw, never from probing.
  - Steps. The group runs T = trunc(30.0 x R) + 1 = 1801 group steps (30.0 at 0x2DCAC8; R the tick rate, 60). Its spawn interval is I = trunc(T / n) with a minimum of 1, where n is the number of entries (8 gives 225; 7 gives 257). Each step lowers the remaining count by 1; at 0 or below the group ends and resets. Otherwise, when the remaining count mod I is 0 and fewer than 8 entries have been spawned, the next entry spawns.
  - Drain. Spawning an entry plays that Mote's effect and, while the word at 0x2DCADC is nonzero (1 on disc), drains d = the word at 0x2DCAE0 (s32, 20) charge from that Mote in the party inventory (SPEC-0021): if the party holds the Mote with a charge above d, the charge drops by d; otherwise the slot is cleared. When the Mote is then not held (or never was), every equipment word of the active hero that holds it is cleared to -1, with cue 0xA4 for a word that held it. So a mote-eater hit eats up to 8 random Motes' charge, 20 each.
  - Who steps it. Each Faerie Sister thread calls the group update once after its state handler, including during state 9's death animation. The alive-sister count excludes state 9, but the timed-group update does not read that count. When the death animation finishes and the shared death path places the sister in base state 16, generic character processing stops stepping that AI; she contributes no further group updates. Thus the drain rate follows the number of sister threads still being stepped, including sisters in their death animation, and decreases only after each reaches base state 16. Exact elapsed duration depends on the death-animation lengths, which are not available from the linked assets (Medium-High; Open question 173).
- Kind 15 (leach): the hero loses magic by the data word 0x2DD458 (s32, 50), never below 0 (at 0 the magic-empty notification runs), and effect 4039 plays at the hero.

**Hit on the owner, after a reflection (Medium-High).** The same contact test is run on the owner, with the owner's block query and the same dot < 0 rule. Not blocked: the owner takes trunc(slot damage) (kind 10: 0) and the status roll applies. The slot is released either way.

**World collision (Medium-High).** It runs only once the projectile is more than 2.0 units from its start point.

1. Ground: let h be the ground height under the projectile. If position.y - r < h: kind 1 enters area mode (area flag 1, pulse timer 60 ticks = the data word 0x2DD440 (1.0, f32) times R; effect id 4 at (x, h, z); cue 190) and stays live. Kinds 2 to 8 play effect id 4 at the ground point and cue 191, then release (kind 8 spawns its summon). Kind 9 ignores the ground. Other kinds: nothing happens when the ignore-world flag is set; otherwise effect id 4 plays at the ground point and the slot is released. **The cues 190 and 191 play nothing (High):** they are sent to the hero's weapon slot, which does not play them (SPEC-0031 Edge cases), so a landing or ground hit is silent. They are not sound ids of the projectile.
2. Otherwise, unless the ignore-world flag is set, the lateral segment through the shot's position is tested against the static meshes of the world cell. Its ends are the position minus and plus 0.5 times a unit vector r, where r is the normalised cross product of the normalised record vector at +0x30 and the record vector at +0x50 (Open question 176; High for the routine). Test vector (synthetic): a shot at (10, 2, 5) with +0x30 = (3, 0, 0) and +0x50 = (0, 1, 0) tests the segment from (10, 2, 4.5) to (10, 2, 5.5). A hit releases the slot. Then the world's collision primitives of that cell are tested (SPEC-0018), then the world object list, then every moving object whose squared 3D distance from the projectile is below 22500 (150 units); the moving-object test runs in the object's local frame. A primitive hit releases the slot. When the breaks-objects flag is set and the primitive belongs to a breakable, that breakable is triggered.

**Area mode (kind 1 after landing; High).** Each tick: if the pulse timer is 0, the hero's horizontal distance to the slot is below the area radius and the hero's height above ground is below 1.0, the pulse timer is set to 60 ticks, the hero takes trunc(D) (no block test) and the status roll runs. Then the area duration drops by 1, and a positive pulse timer drops by 1. The slot is reset when the area duration is below 1. The first pulse can come 61 ticks after landing. A zero area duration ends the slot on the tick after landing. The area flag does not stop the shot's own motion: the kind step (motion with gravity, hit test, world collision) still runs each live tick after the area step, so a landed kind-1 shot keeps moving and is held down by the ground test (Medium-High).

**Lobbed summon release (kind 8; High).** When a kind-8 slot is released (it lands, it touches the world, or its owner disappears), it first makes a world-level spawn-by-key request (Behavior 31) for the key in its target field, at its position, facing its forward axis, with its owner as the summoner. An expiry in the air (lifetime) releases through step 4 and spawns nothing.

**Visual update (draw pass; High).** Each drawn frame, every live slot rebuilds its frame from its forward axis and the world up vector and places its effect object at the position. For visual kinds below 5, a nonzero trail model-part pointer is drawn at the projectile position. The pointer comes from the character's selected model-set table (Behavior 13). The six independent effect ids are described above.

**Hero-side scan (not a hit test).** The hero also scans the ten pool slots each tick for a projectile in front of it, as part of its guard logic. That scan belongs to the hero (SPEC-0036); it does not move, hit or release projectiles.

**Not part of this pool.** The six-slot shot step reached from the hero's own update is the hero's shot slots (SPEC-0036). The four-slot homing, detonation and flight steps are the slot handlers of the class 37 and class 38 controllers (Behavior 19).

**Spawners and other submitters.** The spawners that fill the setup record for enemy missiles are in Behavior 13 (and SPEC-0038 Behaviors 13 and 43). Class 17 creates its missiles through the attack sequencer of SPEC-0038 Behavior 13, and its spell sub-types (SPEC-0038 Behavior 43) submit kinds 2 to 7. Boss and object routines also submit to the pool; the kinds seen there are 10 with visual kind 16 and lifetime 1000, 9 with visual kind 15, and 0 and 1 with visual kind 23 (Open question 163). The values 4007 and 4008 of the release handlers F and G (Behavior 11) are effect ids, not pool kinds; nothing from those routines reaches this pool (Open question 43).

### 10. Contact, reach and hit reaction (generic enemy framework) (High for 10.1, 10.1a and 10.6; Medium-High for 10.2 and 10.5; High for the owners of 10.3 and 10.4)

These tests are shared by enemy and boss routines. The weapon volumes (10.2) are used by the common enemies and the bosses listed there; inside boss routines the single-weapon reach test is only the hero's weapon striking the boss (10.2). Behavior 10.3 is the Ixi Chief's incoming-hit check and 10.4 the Draik Skeleton's stage-4 missile dodge (Open questions 52 and 66).

**10.1 Melee hit with a block push: unreferenced, do not implement (High).** The boot program holds a melee-hit routine with a block test, a push from the attacker's knockback value capped at a length of 40, and a second facing test of 0.95. Nothing reaches it: there is no call to it, no stored pointer to it and no load of its address anywhere in the boot program. A clean implementation leaves it out. The block rule that does run for the common enemies is the class-17 strike of 10.2 (last item), and the knockback that does run is 10.1a.

**10.1a Knockback on the hero from class-17 strikes (High).** After the take-hit of a class-17 melee strike on the hero (SPEC-0038 Behavior 13), when the damage was above 0 (blocked or not), the hero's HP is still above 0, and the attack slot's knockback K (monster row +0x35C + 4k, Layout section 4) is above 0:
1. The hero's velocity is replaced (not added to) by K x unit(9 x f + (0, 12, 0)), where f is the attacker's facing. For a level facing this is 0.6 K along f and 0.8 K upward.
2. The hero's grounded flag is cleared.
3. The hero enters action state 48 (the long-fall and landing state of SPEC-0033 Behavior 9B).
A strike with damage 0, or one that kills the hero, gives no knockback. The grab-and-throw path (SPEC-0038 Behavior 15) enters state 48 without a velocity. Bosses follow their own rules (each boss spec), and pool projectiles give no knockback (Behavior 9). Heroes push no enemies: hero swings, hero shots and Mote objects apply no impulse to an enemy, which reacts only with its hit-reaction animation (SPEC-0038 Behaviors 37 and 44). On disc 130 attack slots in 75 rows have a non-zero knockback, with values 1, 2, 3, 4, 5, 6, 7, 10, 12, 14, 15 and 17.

**10.2 Weapon volumes and reach tests (High for the row fields, the event-bit gates and the target shape; Medium for the anchor and axis reading of the attachment frames).** Hurt shapes, attack windows and the hit list are in SPEC-0040 Behavior 12A; the overlap test is the finite-cylinder test of SPEC-0018.

- Attack-active test of an enemy-kind character: true when any of the animation event bits 0x1 to 0x10 is set on its current frame (SPEC-0005 table B). Bit 2^(N-1) is the window of weapon N. This is also the "action-frame" query that the boss specs call on a boss.
- Weapon N (N = 1 to 5): a finite cylinder with radius = monster row word +4(N-1) and full length = row word +0x14 + 4(N-1) (Layout section 4; the schema names of the two words are swapped and 40.0 is their upper clamp), centred on the anchor of attachment frame N-1 (the translation row of the posed pose entry, SPEC-0005), so it extends half its length to each side of the anchor, with axis = that frame's Z row. A weapon of size 0 is a point at the anchor (Medium-Low). The target's hittable query is not consulted by this test (High).
- Weapon test: for N = 1 to 5 in order, and only when event bit 2^(N-1) is set now, weapon N is tested against each bone capsule of the target (Behavior 26) with the static overlap test (no sweep); the first hit returns true.
- Disc examples: row 9 weapon 1 radius 3.0, height 3.0 (weapon 4 radius 1.5, height 3.0); row 11 weapon 1 radius 5.0, height 5.0; row 169 weapon 3 radius 20.0, height 20.0; row 204 weapons 1 to 3 radius 3.0, height 2.0; row 0 (Black Knight) weapon 1 radius 8.0, length 0.75 and weapons 2 to 5 radius 5.0, length 1.0; row 4 (Gelert Assassin fight) weapon 1 radius 6.0, length 1.75; row 10 (Plague Serpent) weapon 3 radius 5.0, length 5.0; row 155 (Ixi Chief) weapons 1 to 5 radius 5.0, length 3.0.
- Users of the weapon test: the class-17 melee strike (SPEC-0038 Behavior 13) and the bosses' weapon attacks: Black Knight (kick, the two sword swings, high swing, floor strike), Gelert Assassin (sword swing, thrust lunge, jump thrust), Ixi Chief (counter attack, slam, both swipes, buck kick), Plague Serpent (lunge) and Draik Skeleton (High). The other boss melee hits use their own shapes: bone-chain cylinders (Gnarfas), plain distance tests, the Werelupe King's own reach test (SPEC-0019), or no shape at all (the Black Knight's charge impact); see each boss spec.
- Single-weapon reach test (the hero-style test). Every call of it in boss code is the hero's weapon striking the boss (damage taken by the boss); no boss attack uses it (High). A hit is possible only when the full 3D distance between the two character positions is at most 65.0 (fixed constant). The weapon is a finite cylinder of radius 1.0 and full length L along the Z row of attachment frame 0, centred at its anchor + 0.25 x axis; for enemy-kind characters L is fixed at 1.5 (the hero's L is in SPEC-0036). It is tested statically against each target bone capsule; when that misses and the tip moved since the previous tick, the swept test of SPEC-0036 (Combat reach test) runs with the triangle (P0, P0 + L x axis now, P0 + L x axis on the previous tick). The first hit returns true.
- Class-17 strike flags (row 0x3AC + 4k, Layout section 4): 0x01 unblockable (the weapon test runs even while the hero guards); 0x20 forces the blocked flag to 1; 0x08 (grab-and-throw) has no weapon-volume path (SPEC-0038 Behavior 15). Otherwise a hit on a guarding hero with dot(hero facing, enemy facing) at most -0.5 is a blocked hit.

**10.3 Incoming hits of the Ixi Chief (High).** This routine is the Ixi Chief's incoming-hit check (class 11; SPEC-0014, Incoming hits). The boss's loop runs it every tick with the boss's own state word; state 0x12 is the boss's get-hit state, so the state change falls on the struck boss. Nothing happens in state 15 (hold). Melee: when the state is neither 0x12 nor 0x10, and the hero is in an attack with its attack window active, the hero's single-weapon reach test (10.2) is run against the boss. Shots: each live slot of the hero's six shot slots is tested against each bone capsule of the boss whose hurt length is above 0; a contact disarms the shot. In state 0x13 both results are cleared. In states 3, 4, 7 and 0x12, a melee contact is ignored when the hero is in front of the boss (the boss's forward direction dotted with the unit direction to the hero is above 0.5). On any remaining contact the hit flash plays, the state becomes 0x12 and the boss's health word is reduced: by the hero's damage against the boss (multiplier 1.0) for a melee contact, or by the shot damage (SPEC-0014). Health reaching 0 runs the death hook. Because state 0x12 skips the melee test, one swing gives at most one melee hit through this check. Test vectors: the boss in state 4 with the hero swinging in front of it (dot 0.8): no hit; the same swing from behind (dot -0.2): hit, state 0x12.

**10.4 Draik missile dodge (High; never runs in play).** This routine belongs to the Draik Skeleton (class 12) and runs only in stage 4, which is never entered (Section 6.1). When the Draik's state is 0 (idle) or 18 (orbit), or animation 38 is current, and at least one of the hero's six shot slots is live: each live slot's path, from its position to its position plus 300 times its velocity, is swept against a test volume of radius 10.0 and height 5.0 placed 5.0 above the Draik's position (Medium for the volume shape). When any path passes, one roll int[0, 100] is drawn, and only then; a roll below 15 (word 0x2DA9A8) sets the Draik's state to 19 (dodge missile). Otherwise the state is unchanged. State 19 here is the Draik's dodge state, not a hit reaction, and not the Ixi Chief's state 0x12 of 10.3.

**10.5 Shared enemy outgoing damage to a hero (High for the shared enemy class and for the boss rule).** The damage query of the shared enemy class, for a hit on a hero with attack slot k:

1. d0 = the attack damage value, monster row +0x34C + 4k (Layout section 4).
2. Shield block: when the hit is blocked (10.2: the hero guards and the facing test passes), the hero's shield block is subtracted (SPEC-0036).
3. Armour: the hero's armour Mote and body armour reduce the result, and it is multiplied by 0.9 when the active petpet's kind is 1 or 7 (SPEC-0036, SPEC-0047).
4. Multiply by G, the attacker's status scale, clamp at 0, and truncate toward zero (no minimum).

The full formula, its factors and its test vectors are in SPEC-0036 (Enemy attack damage to a hero). Routines that pass a blocked flag of 0 are never blocked.

Who uses this chain (High). Only the class-17 common enemies (including their minions, such as the Darkest Faerie's summoned Minion Warriors and Grunts) and the Spyder Nest's spiders (SPEC-0038). No boss melee routine calls it.

**Boss melee damage (High).** Every boss melee hit sends its value, either a fixed constant or a boss tuning word in the boot program's data, straight to the hero's take-hit. The take-hit subtracts that value from HP unchanged: it is not reduced by the hero's shield block, body armour, armour Mote, the petpet 0.9 factor or the attacker's status scale. The values and windows are in each boss spec.

**Boss projectiles (High).** Boss projectiles that go through the enemy projectile pool (the Gelert Assassin's daggers, Juppie's spit, the Faerie Sisters' balls and missiles) follow the pool's hero-hit rule (Behavior 9), which does apply the shield and the armour, with max(1, 0.25 D) for a blocked shot that cannot be reflected.

**Blocked flag in the take-hit (High for the subtraction; Medium-High for the reaction).** When a routine calls the hero's take-hit with the blocked flag set, the full value passed is still subtracted from HP; only the reaction changes. The hero does not enter its hit reaction; it plays its block reaction instead (the block sound cue and hero animation 0x40) unless it is busy. The block reaction ignores the float argument it is given. A boss that wants a block to cost nothing either skips the take-hit or passes 0 (see each boss spec).

**10.6 Deflected hero shots (Black Knight and Gelert Assassin; High).** Each character has a deflect flag (+0xE5C). The Black Knight's and the Gelert Assassin's deflect and guard stance routines set it to 1 (SPEC-0012, SPEC-0013). Only character creation clears it, so once the enemy has used that stance the flag stays set for the rest of the fight. Reproduce this. When one of Roberta's quick shots meets a guarding target while the shot's arming counter is 0 (SPEC-0036, block, bounce and deflect of shots):
- flag 0: the shot bounces off (reversed and scattered; arming counter 15 ticks);
- flag 1: the shot is sent back at the hero. Its direction becomes unit(the current hero's aim point - the shot position), the arming counter is set to 15 ticks, and the shot turns hostile: once the counter is 0 it steers toward the current hero with the 0.25 blend of the hero's shots and tests the hero's body cylinder instead of enemies. A guarding hero bounces it again (it stays hostile). Otherwise the hero takes the shot's damage, any status of the shot is requested, and the shot is disarmed. A deflected shot that hits the hero also pays its MP cost (quirk).
Full-release shots are never blocked or deflected (SPEC-0036).

### 11. Attack release handlers (Medium for animation indices and timing ranges; Low for spawn details)

Eight routines share one shape. Each plays one animation from the enemy's own animation table (Behavior 2); it waits for that animation's release event; at the event it performs one action (a projectile, or a knockback); and then it ends in a recovery that returns to the enemy's default state. Routines C, D, G and H are states 3, 5, 2 and 4 of class 37 loop B, and E and F are states 3 and 2 of class 38 (Behaviors 7 and 30; High). Routines A and B are codes 3 and 4 of the class-37 Darkest Faerie controller. The indices and ranges below are fixed constants, not data words on the disc.

| Routine | Release animation | Action at the release event | Recovery after the action |
| --- | --- | --- | --- |
| A | 7 | Class 37, code 3 (Darkest Faerie): one type-0 elemental orb with kind 1–6, or one type-1 aimed bolt with kind 0, fired at the animation event; aim blend 0.25 | On animation completion, draw cooldown [60, 240] ticks (1–4 seconds) |
| B | 8 | Class 37, code 4 (Darkest Faerie): one type-1 aimed bolt with kind 0 at the animation event | On animation completion, draw cooldown [60, 240] ticks (1–4 seconds) |
| C | 7 | Class 37 loop B, state 3: opens a type-0 slot (an orb that homes on Queen Fyora) at her sample point | 30 to 45 ticks |
| D | 7 | Class 37 loop B, state 5: opens a type-2 slot (an orb that homes on the hero and summons); turn blend 0.25 (fixed constant) | 120 to 240 ticks |
| E | 22 | Class 38, state 3: opens a type-1 slot (an orb that homes on the class-37 object) at her sample point | 30 to 45 ticks |
| F | 23 | Class 38, state 2: relaunches the first active type-0 slot as type 3 and plays effect 4007 on it (rule 4) | Back to state 0 when the animation ends (Behavior 30) |
| G | 10 | Class 37 loop B, state 2: relaunches the first active type-1 slot as type 3 and plays effect 4008 on it (rule 4) | Back to state 0 when the animation ends (Behavior 7) |
| H | 9 | Class 37 loop B, state 4: knocks the hero back (upward 12.0, along 9.0, scaled to 10.0, word 0x2DAAB8) and puts the hero in hero state 48 (Behavior 7) | Back to state 0 when the animation ends |

Rules that apply to the table:

1. Recovery draws. A recovery is a random whole tick count. "0.5 to 0.75 seconds" is drawn as a whole number of ticks in [30, 45]; "2 to 4 seconds" is drawn in [120, 240]. The multipliers 0.5, 0.75, 2 and 4 are fixed constants, and the tick rate is 60 per second (SPEC-0007). Do not multiply these values by 60 again.
2. Spawn. Routines A and B use the Darkest Faerie four-slot array of Behavior 19: routine A submits either a type-0 elemental orb (kind 1–6) or a type-1 aimed bolt (kind 0); routine B submits a type-1 aimed bolt (kind 0). They do not submit to the ten-slot enemy pool of Behavior 9. Routines C to G also use the four-slot array.
3. A release event that never fires produces no action here. What the routine does after a missed event is not established.
4. Relaunch (routines F and G; High, axis roles Medium). Routine F takes the first active type-0 slot and routine G the first active type-1 slot of the four-slot array at 0x36D3F0 (Behavior 19); with none, it does nothing. It makes that slot type 3 with a 60-tick timer, launches it along unit(|f| x forward + (1 - |f|) x side), with the side negated when f is 0 or less and f = U[0, 1) - 0.5, and plays effect 4007 (F) or 4008 (G) on it. 4007 and 4008 are effect ids. The forward and side vectors are the deflecting character's own, so the orb leaves at 45 to 90 degrees from her facing. Example: f = 0.25 gives unit(0.25 x forward + 0.75 x side), about 71.6 degrees from forward.

### 11a. Neopoint steal of an attack (selector 0) (High; wallet identity Medium-High)

An attack slot whose steal bit is set (Layout section 4, bit 0x04) and whose selector is 0 can take Neopoints from the hero after the attack-impact gate passes. Before the steal chance roll, the impact step returns without continuing when the normal strike result is nonpositive and the special impact-success marker is clear. For the normal slot path, a positive strike result is required. For the slot path with bit 0x08 set, the marker is set only when the sequence state is 4 and the shared gate query returns 0 (the hero is not yet in the swing's ten-slot struck list). The tests behind a positive strike result are given in Open question 157. Steps after that gate, in order:

1. Chance roll: draw a whole number r from 0 to 100, both included (101 values). The steal succeeds if r is below the slot's steal chance (a percentage, compared as numbers). For a whole-number chance c the success probability is c out of 101: a chance of 50 gives 50/101 (about 49.5 percent), a chance of 100 gives 100/101 (about 99.0 percent). If the steal fails, nothing else happens: no amount is drawn, the wallet is unchanged and no coins spawn.
2. Amount: if the slot's steal minimum equals its steal maximum, the amount is that value and no draw is made. Otherwise, if the minimum is above the maximum, the two are swapped. A whole number is then drawn uniformly from the lower value to the upper value, both included (Open question 156).
3. Wallet: the amount taken is the drawn amount, or the whole wallet when the wallet holds less. The wallet is reduced by the amount taken and never goes below zero. The wallet is the party's Neopoint wallet (SPEC-0021).
4. Payout: the amount taken is spawned at once as coins at the hero's position, split with the coin rule of SPEC-0021 (Stolen Neopoints split into coins). All coins of one steal sit at one point. The enemy's carried Neopoints (offset 0xC28) are not changed by this branch (Open question 155).

Table values on disc: the Thief has a range of 10 to 100 Neopoints at chance 50; the Thug has 20 to 200 at chance 50; both have selector 0. The Mountain Crokabek has selector 0 with a range of 0 to 0 at chance 100, so its steal takes 0 Neopoints and spawns no coins. The other two Crokabek rows have selectors 1 to 3 and use the item branch (Behavior 11b).

The steal itself has no separate hit test: it follows the shared attack-impact gate above, whose tests are given in Open question 157.

### 11b. Item steal of an attack (selectors 1 to 3) (Medium-Low)

An attack slot whose steal bit is set and whose selector is 1, 2 or 3 takes an item from the hero instead of Neopoints. Selectors below 0 or above 3 pick item id -1, which no inventory slot holds, so the steal ends with no effect (selector 0 is the Neopoint branch of Behavior 11a). Steps, in order:

1. Candidates: each selector offers two items. Selector 1 offers items 150 and 149; selector 2 offers 146 and 157; selector 3 offers 156 and 147. The pairs are stored as data in the program at 0x2DD550 (Medium).
2. The two offered item ids are checked in stored order with the same hero-inventory eligibility test. Each offered item held by the hero is retained as a candidate. If neither is held, the steal ends and nothing else happens.
3. Otherwise one retained candidate is chosen uniformly at random (Medium).
4. One unit of the chosen item is removed from the hero's inventory. The enemy then carries it: its carried flag is set, the item id is recorded, and the carry timer is set to 180 ticks (3 seconds at 60 ticks per second; SPEC-0007). While the flag is set and the timer is positive, the enemy's item is drawn at 0.45 scale. Each timed-emitter update subtracts one tick; when the timer is below 1, the carried flag is cleared. The item-id field is not cleared by this expiry path, but the drawing stops because the flag is clear. The death payout of carried Neopoints is specified in SPEC-0038, Behavior 34.
5. The enemy's carried Neopoints (offset 0xC28) grow by the stolen item's catalogue price (the item text-table value at row +0x0C). On death, carried Neopoints are paid out as coins and the field is set to zero (SPEC-0038, Behavior 34). The Neopoint steal (Behavior 11a) does not change this field.

Confidence: High for the selector routing, candidate-test order, use of the same inventory eligibility test, inventory removal, carry flag/item id, 180-tick timer, expiry flag clear, catalogue-price increase, and death payout. The candidate pair and uniform choice remain Medium.

Test vectors: selector 1 with only item 150 held retains (150) and chooses 150; only 149 held retains (149) and chooses 149; both held retain (150, 149) in that order and choose either uniformly; neither held ends the steal without removal or carry.

### 12. Routines once listed as generic AI controllers: owners (High for the owners; Medium where marked)

None of the routines below is a shared controller. Each one belongs to one behaviour class or to the hero, as the class routine table (Layout section 3) and the callers show. The common-enemy class (17) runs none of them; its own AI is SPEC-0038.

| Part | Owner | Monster rows on disc | Where it is specified |
| --- | --- | --- | --- |
| 12.1 | hero (Roberta charge state) | none | SPEC-0036 |
| 12.2 | hero state 30 (hang from a ledge) | none | SPEC-0033 |
| 12.3 | hero state 23 (sidle; unreachable in play) | none | SPEC-0033 |
| 12.4 | hero state 21, sub-mode 10 (climb panel step) | none | SPEC-0033 |
| 12.5 | class 1, Gelert Assassin: the rainbow dash | row 4 (type 4) | SPEC-0013, 4.8 |
| 12.6 | class 21, Miamouse: the dispatch | row 248 (type 104) | Behavior 28 |
| 12.7 | class 2, Plague Serpent: the anchor chooser | row 10 (type 8) | SPEC-0016 |
| 12.8 | class 34, live (summoned) petpet: state 1 | no row (the class is set when the petpet is created) | this section; SPEC-0038 Behavior 27 |
| 12.10 | class 38, Queen Fyora: shared-array slot type 2 handler | shared array type 2 | Behavior 19 |
| 12.11 | class 12, Draik Skeleton: state 11 (Medium) | row 3 (type 3) | Section 6 |
| 12.12 | class 12, Draik Skeleton | row 3 (type 3) | Section 6 |
| 12.13 | class 34, live petpet: state 2 | no row | this section; SPEC-0038 Behavior 27 |

**12.1 See SPEC-0036.** A 14-state routine whose state word, animations 27, 147 and 148, cues 162, 169 and 171, and state-13 facing gate (a strict test against 0.98) match the hero's charge routine, the Roberta variant of SPEC-0036 State 13, is not an enemy routine. It is not part of this spec and must not be used as an enemy controller. (High for the match of the constants; Medium for the state list, which is not confirmed here.)

**12.2 Hero state 30 (High for the owner).** The seven-state routine with animations 0x3B, 0x3C, 0x3D and 0x47 to 0x49 and the fixed values 0.25, 0.7 and 1.5 is run by the hero's state dispatcher for hero state 30 (hang from a ledge, SPEC-0033). It is not an enemy routine and must not be used as one.

**12.3 Hero state 23 (High for the owner).** The routine with animations 79, 80 and 81, the one-shots 42, 59 and 61 and the fixed values 0.25, 0.1 and 1.5 is the body of hero state 23 (sidle, SPEC-0033), which play never offers. It is not an enemy routine.

**12.4 Hero state 21, sub-mode 10 (High for the owner).** The step that plays animation 58, then 49, and ends at progress 0.95 with a 100-tick timer before sub-mode 2 is the climb-panel step of hero state 21 (SPEC-0033). It is not an enemy routine.

**12.5 Gelert Assassin rainbow dash (High).** The eight-slot shuffle is the Gelert Assassin's rainbow dash (class 1), specified in SPEC-0013, 4.8. Its three words are the boss's tuning words: 0x2DAD54 = 40 is the dash speed in units per second (the step per tick is 40 divided by the tick rate, 0.667 at 60); 0x2DAD58 = 8 is the hit radius (horizontal distance to the hero); 0x2DAD90 = 15 is the damage of one dash hit. The order of the eight points is drawn once per dash, at the dash start, as a permutation of 0 to 7. Animations 44 (the points) and 45 (the end) are the dash animations.

**12.6 Miamouse dispatch (High).** The dispatch with the one-shot of animation 13 and the value 10.0 is the class-21 dispatch of Behavior 28: while clip 13 (the release) is current the state becomes 3; 10.0 is the hero's horizontal notice distance; states 0 to 4 are the Miamouse main states (0 idle, 1 wander and pause, 2 flee, 3 released and landing, 4 delivered).

**12.7 Plague Serpent anchor chooser (High).** The nearest-of-eight routine is the Plague Serpent's anchor chooser (class 2, SPEC-0016). The eight candidates are the anchor slots; the distance is the horizontal distance from the hero to point B of each slot; on a tie the lower slot wins. On a change of slot the serpent goes to state 5 (relocate), and the draw of 0 or 1 is the new anchor flag. State 1 is the idle state that the arrive step returns to.

**12.8 Live petpet, state 1: wander near the hero (High; animation meanings Medium).** This routine is state 1 of the class-34 main loop (SPEC-0038 Behavior 27), which is the controller of the summoned petpet (SPEC-0047). It is not a common-enemy timer. Its outer gate is the class-34 selector of SPEC-0038 Behavior 27: the selector enters state 1 when the petpet still has time left, the hero is not in a jump state, and a second hero query returns zero. The numbers 1, 6 and 7 of the choice are animation codes of the petpet, not states.

Let L be the petpet's remaining-time word (+0xE34 of the character, in ticks; the main loop lowers it by 1 per tick). Let r = L / (0.5 x tick-rate word x 60), with the 0.5 the word 0x2DB1D8; at 60 ticks per second r = L / 1800, so r = 1 when 30 seconds are left. The state record holds a point, a timer (-1 when unset), a hold count, and the id of the last character it reacted to. Each tick:

1. Notice. The routine looks for the first listed character that passes the character-list test and is within 10.0 units (horizontal) of the petpet, with an id different from the stored one. When it finds one, or while animation 8 is current: if animation 8 is not current, the id is stored and animation 8 plays; when animation 8 has finished, animation 0 plays. Nothing else runs that tick.
2. Walk. When the horizontal distance from the petpet to its point is 1.0 or more, the petpet turns toward the point (weight 0.1), plays animation 2 (walk), and the timer and hold are reset (timer -1, hold 0).
3. Arrive. Otherwise: an unset timer draws a whole number in [180, 600] ticks and animation 0 plays. A set timer goes down by 1; when it reaches 0 or less, a new point is drawn (the hero's position plus a random offset on x and on z, each of size 3.0 to 10.0 with a random sign) and the timer and hold are reset.
4. Idle choice, on every other arrived tick: when r is below 1, animation 7. Otherwise a whole number roll in [0, 100] is drawn; animation 1 when the roll is greater than 200 x |1 - r|, otherwise animation 6. The choice is made again on every such tick; a request for the current animation does not restart it.
5. The hold step (set the hold to 1.5 x the tick-rate word, 90 ticks, after a finished animation, then count it down while animation 0 plays) is present, but the hold is 0 on entry and after every walk and is set only inside that step, so it never runs on the disc.

**12.9 Not an enemy routine: the hero idle picker (High).** The routine once listed here as "ambient cue timers" is the hero's idle picker, specified in SPEC-0033. Its two countdowns (words 0x2DD57C to 0x2DD588) and its values (0x6B to 0x6F, 0x9E to 0xA1, 0x9D, 0x92, 1 and 0x23) are hero animation ids, not sounds or effects. Hero action tables carry no sound word, so the picker plays no sound. It must not be used as an enemy controller (Open question 46, resolved).

**12.10 Class-38 shared-array slot type 2 handler (High; see Behavior 19).** The routine formerly described as a range/region check is the slot type 2 handler of the class-38 four-slot dispatcher. For an active type-2 slot, it advances the slot toward the destination value returned by the hero's method at +0x294 by 0.5 units per tick. At strict horizontal distance below 2.0 it deactivates the slot and clears its orientation. If current-world level-state id 2 is clear, it requests a spawn by key using the class-37 opponent's row slot-0 key at the hero's position and facing, with the opponent as owner. The test is current-world level-state id 2: in Fyora's Tower, the switch-trigger path pulses it for one tick after progress flags 329 and not 330; while set, this handler suppresses its arrival spawn (High for this local effect). The hero method returns the 8-byte position value at attach point 7 (the status display anchor documented in SPEC-0004), which this handler uses as its destination point. Why this slot targets that anchor is not established. Behavior 19 describes the shared array and the class-37 state that opens a type-2 slot.

**12.11 Draik Skeleton state 11 (Medium).** The helper with animation 31 (0x1F) matches the Draik Skeleton's state-11 handler (class 12, Section 6): in its sub-state 1 it turns toward the hero (weight 0.25), plays animation 26 and then 31, and counts down a timer drawn from two Draik tuning words. The "state 1" it checks is its own sub-state, not the hero's state.

**12.12 Draik-side helpers (Low).** These are the helpers of the Draik family outside Section 6.

- Flying approach. Two copies (the first and its sibling) turn the enemy toward its target and play one of three animations: 0x0C (12), 0x0E (14) and 0x0F (15) in the first copy; 0x24 (36), 0x26 (38) and 0x28 (40) in the sibling. A range gate uses a parameter of the call. A cosine threshold of 0.95 (fixed constant) is used. Two words of value 0.1 (0x2DAA74 and 0x2DAA78, both read from disc) are used as blend weights (role Low). The two copies are the stage-2 and stage-4 approach of the breath, claw and tail handlers, so they are unreachable (High); they write 2.0 into the Draik's flying offset (6.9).
- Fire and ice spit (states 8 and 9; High for handler identity and record separation). Both select the same stage-based clip map (6, 19, 31, 44 and 52 for stages 1–5) and react to its event child. State 8 fills the first spit projectile record; state 9 fills the second. Both aim from the shared emit frame toward the hero position plus 2 units of height, then use the common projectile update with speed 1.0, hit radius 0.5 and damage 10. Their motion and collision-callback values match, while their attached effects use separate configurations. Their appearance is described under "Spit appearance" in Section 6: fire is a red sprite effect and ice a blue one, with no model; neither a distinct element nor status effect exists (Open question 53).

**12.13 Live petpet, state 2: follow the hero (High).** The three-zone chooser is state 2 of the class-34 main loop (SPEC-0038 Behavior 27). Its "states" 0, 2 and 3 are the petpet's animation codes 0 (idle), 2 (walk) and 3 (run), and its zone flag is the gait. The follow point is the hero's position plus 4.0 times the hero's first basis vector plus 1.5 times its second basis vector (which body axes these are is Medium). The distance is the horizontal distance from the petpet to that point, compared with the four thresholds listed under Where it lives (3.0, 1.5, 7.0 and 5.0). Each tick:

- Gait 0: if the distance is 3.0 or less, play animation 0. Otherwise play animation 2 and set the gait to 1.
- Gait 1: if the distance is above 7.0, play animation 3 and set the gait to 2. Otherwise, if the distance is below 1.5, play animation 0 and set the gait to 0. Otherwise play animation 2.
- Gait 2: if the distance is below 5.0, play animation 2 and set the gait to 1. Otherwise play animation 3.
- In gaits 1 and 2 (the gait at the start of the tick), the petpet also turns toward the point with a weight of 0.1 (a fixed constant). In gait 0 it does not turn.

Boundary values: a distance exactly equal to a threshold is not above or below it, so 3.0 in gait 0 gives animation 0, 7.0 in gait 1 gives animation 2, 1.5 in gait 1 gives animation 2 and 5.0 in gait 2 gives animation 3.

### 13. Projectile spawners for enemy missiles (extends Behavior 9) (High)

Four spawners each build one setup record (Behavior 9) and submit it once to the pool. They are called by the attack sequencer of the common enemies (SPEC-0038 Behavior 13, state 2): row flag 0x02 of the slot selects the lobbed summon; otherwise the slot's flight mode (row 0x2BC + 4k) selects mode 0, 1 or 2, and any other value spawns nothing. k is the attack slot.

**Common to all four spawners.**

- Launch point: the enemy's emit point for the spawn-event bit that fired. The bits are tested in the order 1, 2, 4, 8 and 16; the first one set selects emit point 0, 1, 2, 3 or 4 (bit 1 gives point 0, bit 2 point 1, and so on). Bit 0x20 and any other word select point 0 (High). Test vector: event word 0x26 selects point 1; event word 0x20 selects point 0. An emit point is one of the enemy's pose-slot positions: point k is the position stored in the enemy's pose block at a fixed step of 0x40 bytes (point 0 first, then 1 to 4 at the next 0x40 steps), so it is an attach-slot position of the animated pose, not a bone or a fixed offset (Medium-High: the selection is complete; the slot-to-attach-point identity is inferred from the layout).
- Aim point = target point + e w. w is a random horizontal unit vector, from x and z each drawn uniformly in [-1, 1] and normalized (the draws come in the order e, then x, then z, all from the shared random stream; High). e = U[0, row inaccuracy 0x30C + 4k], plus 5.0 when the enemy is outside the camera cone (dot(camera forward, unit(enemy - camera)) < 0.85), plus 3.0 when the common-enemy attack manager has the enemy waiting in line (SPEC-0038 Behavior 45).
- Owner = the enemy id; element = row 0x3C; damage = row 0x34C + 4k times the enemy damage scale (1.0; x 0.5 under curse, status 1; x 1.5 under strength, status 4), floored at 0.
- Radius = row 0x2EC + 4k. Roll = row 0x32C + 4k. Spin rate = row 0x31C + 4k. Visual kind = row 0x43C + 4k. Six effect ids = the row u16 words at 0x46C + 2k + 8j (j = 0 to 5).
- Area: radius row 0x45C + 4k, duration trunc(row 0x44C + 4k) ticks. Status: id row 0x36C + 4k, chance trunc(row 0x37C + 4k), value trunc(U[row 0x38C + 4k, row 0x39C + 4k] / 60).
- Trail model part (visual kinds below 5): the selected character model set supplies an 11-entry table of 32-bit model-part pointers. Each region model-set record occupies 0x2C bytes in the model-set array beginning at region offset 0x7C8; the character's region character record selects one record, and the character keeps its pointer to that table at offset 0x940. The model-set loader places each part in slot (stored part index − 10), for stored indices 10 through 20. The trail lookup uses this table with zero-based indexing: requested index 1 is the second pointer (stored part index 11). For shot counters 0, 1, 2, 3 and 4, modes 0 and 2 request indices 1, 7, 8, 9 and 10 respectively; other shot counts request index 1. Mode 1 always requests index 1. A null selected pointer contributes no trail part. The shot counter is the sequencer's count of shots in the current attack. This table is model data, not the monster row, companion row, an effect-id table or a boot-program table.
- Ignore-world flag = row flag 0x10 (modes 0 and 2).

| Spawner | Kind | Target point | Velocity | Lifetime (ticks) |
| --- | --- | --- | --- | --- |
| Mode 0 | 0 | hero position + (0, 2.5, 0) | unit(aim - launch) x v | trunc(range / v) |
| Mode 2 | 17 (homing) | hero position + (0, 2.5, 0) | unit(aim - launch) x v | 2 x trunc(range / v) |
| Mode 1 | 1 | hero position | ballistic, ramp speed s | trunc(60 T) + 1 |
| Row flag 0x02 | 8 (lobbed summon) | hero position | ballistic, speed v | trunc(60 T) + 1 |

v = the launch speed (row 0x2CC + 4k) and range = row 0x2FC + 4k, in units per tick and units. A straight shot therefore travels about its range before it expires.

**Ballistic launch (High).** d = the horizontal distance from the launch point to the aim point; dy = aim.y - launch.y.

- Mode 1 speed ramp: u = 1 - (d - bmin)/(bmax - bmin), with bmin = row 0x18C + 4k and bmax = row 0x17C + 4k (the slot's trigger band), and s = v + u^2 (v_end - v), with v_end = row 0x2DC + 4k. So s = v_end at d = bmin and s = v at d = bmax. u is not clamped: outside the band u^2 grows again. The lobbed summon uses s = v.
- Flight time T = d / s, in ticks. Horizontal velocity = s x unit(horizontal offset). Vertical velocity vy = (0.5 g' T^2 + dy) / T, with g' = 32 / 60^2 (Behavior 9, Motion).
- Because the motion applies gravity before the move, the shot is 0.5 g' T units below the aim height after T ticks (it lands slightly short or low).
- Lifetime trunc(60 T) + 1: the flight time T is already in ticks, so this is 60 times the flight time (a unit slip in the original; reproduce it). The ground test of Behavior 9 normally ends the flight long before.

**Lobbed summon (row flag 0x02; High).** Kind 8, visual kind 23, damage 3.0 and radius 1.0 (fixed; kind 8 never touches a character). The target field holds the slot's spawn key, row u16 0x3CC + 2k. When the shot lands (or its owner disappears) the keyed record spawns at the landing point (Behavior 9, Lobbed summon release; Behavior 31). The same flag also limits the slot by the count of the enemy's living summons (SPEC-0038 Behavior 7, step 2e). On disc the flag is set on three used attack slots: row 211 slot 0 and row 221 slot 3 (missile slots, so lobbed summons) and row 128 slot 0 (a melee slot of the class 37 Darkest Faerie row, so no lobbed summon is fired from it).

**Spell builders.** The seven spell sub-types of the common enemies (SPEC-0038 Behavior 43) build kinds 2 to 7 with a fixed speed of 1.0 per tick, aimed at the target character + (0, 1.5, 0), with lifetime trunc(distance) + 5 ticks.

Test vectors: with a synthetic model-part table whose indices 1 and 7 contain non-null pointers and index 8 is null, shot counters 0 and 1 request the first two pointers, counter 2 requests null, and mode 1 requests index 1 regardless of counter. For common-enemy row 190, the six stored effect-id words are [1603, 0, 991, -1, 991, 991]; therefore the six setup effect ids are not all -1, and the trail pointer is not the only visual/effect input. Row 149 has [1400, -1, 1399, -1, 1399, 1399]. These row fields remain separate from the model-part pointer lookup. [High]

Edge cases in this family are listed under the Edge cases section.

### 14. Spawn into a member slot group (Medium)

The owner keeps a group of member slots. The group has a capacity, a count of occupied slots, and for each slot a flag, a member reference and the kind of its member. These fields are part of the owner's runtime record; they are not disc layout.

A spawn request for kind k is handled in this order:

1. If the count is at the capacity, the request is refused.
2. If a slot already holds a member of kind k, the request is refused. A kind already present blocks the spawn.
3. Otherwise a member of kind k is created. If the member cannot be created, the request is refused .
4. On success the slot's flag is set, the member's transform is copied from the request (two four-float transforms), the member's animation is set, and the member is added to the active list.

Despawn: the projectile pool of Behavior 9 releases its own slots; that is not a member removal. Removal of a member from the group list is not described (Open question 67).

### 15. Entity group update (Medium-Low)

This pass updates five groups of world-zone entities during world submission; it is not an enemy-AI update. It runs only when the world-active word is nonzero and receives a camera-derived query vector. The pass clears a per-world reference list, then walks the five groups in a fixed order. Zone-instance callbacks repopulate that list during their own submission, up to 15 objects; hero collision and interaction scans consume the refreshed entries and their part/contact records.

1. The count-prefixed array of entity references is cleared (its count is set to zero).
2. Five groups are walked in a fixed order. Each group is a count word and a pointer to an array of entity references, held in the world record: count and pointer at offsets +0x2D30 and +0x2D34, then +0x2D28 and +0x2D2C, then +0x2D20 and +0x2D24, then +0x2D10 and +0x2D14, then +0x2D18 and +0x2D1C.
3. For each entry, the entity's method is called. The entity holds a class table pointer at +0xAC; the class table gives the method's function slot at +0x1C and the object offset (a u16 at +0x18), and the method's object is the entity plus that offset. The method receives a time value from the game's frame timing (SPEC-0007) and the caller's argument.
4. The results of all the calls are summed and returned to the caller.

A group with a zero count adds nothing. The callback results are summed and returned to the world submission wrapper, which discards that return; no gameplay use for the sum is established. The authoring names and exact roles of all five groups remain open (Open question 77). The camera-derived query value and its source are specified in SPEC-0046 (Open question 78). [Medium-Low]

### 16. Attack count and attack slots (pointer)

The routine once described here as an attack-slot chooser fills a character's four light slots; it is specified in SPEC-0046 (Character light slots). The number of enemies allowed to attack the hero at once is set by the attack manager of SPEC-0038 Behavior 45. The attack slots of a monster row and the choice among them come from SPEC-0038 Behaviors 7 and 14.

### 17. Active character list and the hero shot's character walk (High for the list; Medium-High for the walk)

The list of active characters is walked once per tick by every live hero shot slot (SPEC-0036, Hit on characters), not by an enemy framework. The list count and the list pointer are held in two runtime words (data addresses 0x35C8F8 and 0x35C8FC). They are outside the file image and need no boot value, because the main loop rebuilds both before any use on every pass (below).

Actor-list rebuild (High). The main loop rebuilds this list from scratch on every pass, also while the game is paused (game-state code 9) and while the world is disabled (SPEC-0007). The list holds up to 45 actors. It takes, in order, the own region of each of the three world buffers that is loaded, then each slot region of the current world, and from each region every present character in slot order; the count word is then set to the number taken. So the list always names the characters of the loaded regions as of this pass, and a character removed from its region drops out of the list on the next pass. The walk of one shot slot (the record layout is in SPEC-0036, Hero shot slots) visits the listed characters in list order. For each character:

1. Eligibility: the character's hittable-target method (class slot +0xCC; SPEC-0036, Hittable predicate). When it is false, the character is still visited if its guard method (class slot +0x164) is true and the shot's arming counter (slot +0x7C) is 0; otherwise it is skipped. A visited character must also have at least one bone hurt record of positive length (item 2) and must not be hidden by the invisible-kind predicate.
2. Hit list: the target's bone hurt records, 0x50 bytes each, with their count at +0x38 of its skeleton block. A record counts only when its length (+0x40) is above 0. The shot is a sphere (centre slot +0x30, radius slot +0x48) tested against the bone's capsule (radius +0x44, length +0x40, centred on the bone's posed anchor and extended half the length, the factor 0.5, along the bone's axis; SPEC-0004 and SPEC-0040, Behavior 12A). The first record that overlaps ends the loop. The 0.5 is therefore the half-length factor of the capsule, not a scale of the sphere.
3. On a hit against a guarding target (guard method true) whose shot word at +0x84 is 0: with the arming counter above 0 the shot passes through with no effect, and with the counter at 0 it bounces or deflects (SPEC-0036, Quick shot meets a guarding target) and stays live. In every other case (a target that is not guarding, or a shot whose +0x84 word is nonzero) and when the hittable method is true, the target's take-hit method (class slot +0xBC) receives the damage from the shared damage helper, and the armed word (slot +0x58) is set to 0 whether or not damage was dealt. The take-hit also receives two shot words as arguments, one of which is true when the homing mode (+0x5C) is 1 and the other the +0x84 word; their use inside the target is not described here (Low). The attacker id and the damage come from SPEC-0036.
4. The four words at slot +0x68 to +0x74 are a stored push vector. When the first word is not -1, the vector is also handed to the target's push method (class slot +0x124) after the take-hit; when it is -1, only the take-hit runs (Medium).
5. The 0x18C0 is an offset, not a size: before the capsule loop the walk sets the 16 bytes at +0x18C0 of the target's animation object to zero. Fourteen hit routines of the game do the same, and no routine reads that block, so it is scratch space that a clean build may omit (High for the writes; High that nothing reads it).

Test vectors (synthetic): a bone record of length 0 is skipped even when the sphere centre is inside it. Two listed characters, the first hidden and the second hittable, with the shot sphere inside both: only the second is hit and the shot's armed word becomes 0. A guarding target with the shot's arming counter at 3 and the +0x84 word 0: nothing happens and the shot stays armed; the same target with the counter at 0 bounces the shot.

Confidence: High for the list rebuild; Medium-High for the order of the walk; Medium for the push vector.

### 18. Darkest Faerie volley and its timed state machine (owner High; control flow Medium-Low; words High)

A volley controller fires four projectiles in one volley, then runs a timed state machine that counts hits. It belongs to the Darkest Faerie (class 37 controller, Behavior 7): the volley and its 50-second hover window are her burst and absorb window, described as a fight in SPEC-0054.

**Volley start (Medium-Low).** The start routine runs only when a lookup of event id 14 on the enemy's sub-object succeeds (meaning open). It then:

1. Plays animation 6 through the enemy's animation controller and sets the enemy state to 7 (the volley state).
2. Resets the hit count to 0 and sets the volley timer to 50.0 seconds: the word at 0x2DA628 (f32 50.0) times the tick rate, which is 3000 ticks. In this controller, event 14 is the only path that resets these two fields.
3. Makes four spawns by key (Behavior 31) of the row's slot-1 spawn key (row offset 0x3CE), shot index 0 to 3 (High). Each spawn is at a point computed from its shot index (Medium-Low) and passes direct drop item 172 (Nova Mote). For row 128 the key is 2844, so each volley summons up to four Minion Grunts. The spawns do not use the member slot group of Behavior 14: a spawn is refused only when no unbound reserved record with the key, no free character object or the active cap remains, so four reserved records give four Grunts (High; Open questions 102 and 160).

A second test, on event id 15, sets the state to 0 and clears a global word and two record words (at offsets 0x28 and 0x04). It does not clear the hit count or the volley timer (Open question 119).

**State machine while in state 7 (Medium-Low).** Each tick:

1. Sets the motion record's no-gravity flag (+0x144) to 1 each tick. This suppresses the vertical gravity force and skips ground projection for that tick (SPEC-0040, Behaviors 14A and 14C).
2. Counts the volley timer down by one. When it is below 1 the state becomes 8 and the routine stops.
3. Plays animation 14 (Open question 103).
4. Sets a hover target from the highest qualifying ground height under the boss, then adds the offset for the current sub-state. The ground probe uses the boss motion record's current position and an 8.0-unit ceiling above its current height (SPEC-0018, Ground probe). Sub-state 0 adds +15.0 (word 0x2DA640), sub-state 1 adds +10.0 (0x2DA644) and sub-state 2 adds +5.0 (0x2DA648). The height moves toward the target by at most 0.05 units per tick (0x2DA64C). Sub-states 1 and 2 each have a 3-second timer (0x2DA62C = 3.0 times the tick rate, so 180 ticks). When that timer runs out in sub-state 2 the sub-state becomes 1, and in sub-state 1 it becomes 0. Sub-state 0 has no timer.
5. Tests each valid target entry (up to six) against the boss's hit point with the target's radius. On the first hit of a target the target is removed from the list, the hit count goes up by one, and the timer is reset to 180 ticks. If the count reaches 3 the state becomes 9.

The hit count and the sub-state are the same stored value. After each hit the sub-state equals the new count, so the offset drops 15.0, 10.0, 5.0 in step with the hits. Transitions out of state 7: to 8 when the volley timer runs out, to 9 at three hits, and to 0 through the second test.

**Descent and recovery (states 8 to 10; High).** State 8 is the timer-expiry route: it keeps no-gravity enabled, probes the ground beneath the boss from 8.0 units above its current height, and lowers the boss by at most 0.05 units per tick. When the remaining gap is at most 0.05, it posts events 13 and 10 and returns to state 1. State 9 is the three-hit route: it keeps no-gravity enabled, requests animation 20, probes the same ground reference, and lowers the boss by 0.1 units per tick while the gap exceeds 0.1. At a gap of 0.1 or less it runs the position-height setter, posts event 16 and enters state 10. State 10 requests animation 20 while event 13 is clear; once event 13 is set, it posts event 10 and returns to state 1. The handler does not set no-gravity, so normal motion-record gravity and ground projection resume there.

### 19. Controller slot handlers of classes 37 and 38: homing, detonation and flight steps (owners High; constants High; step behavior Medium-Low)

**Owners (High).** The four-slot homing, detonation and flight steps below are not enemy-pool projectiles (Behavior 9) and the enemy record holds no projectile slots. They are the per-tick slot handlers of two controllers' own slot arrays:

- Class 37, the Darkest Faerie four-member controller (Behavior 7; SPEC-0054): four slot records of 0x210 bytes at 0x36B640 (run-time memory, not on the disc), slot types 0, 1 and 2.
- Class 38, the Queen Fyora controller (Behavior 30): four slot records of 0x210 bytes at 0x36D3F0 (run-time memory), slot types 0 to 3, shared with class 37 loop B (below).

Each controller advances its four slots once per tick in ascending slot order through its own slot dispatcher, which calls the handler for each active slot's type. Class-37 allocation scans in that same order, claims the first inactive slot and returns; if all four are active, the allocation is dropped.

**Class-37 slot allocation (High for slot order and supplied point; Medium for random offsets).** A slot is free when its active word is zero. Allocation scans slots 0 through 3, claims the first free slot and returns; when all four are active, it does nothing. It marks the slot active, copies the supplied character sample point into the slot, and sets its start point using two offsets drawn uniformly from -5.0 to 5.0 (fixed bounds; float draws in [-5.0, 5.0)). These class-37 allocation paths do not call the 8.0 ground-probe helper or derive a speed factor from it. The source of the earlier third-coordinate/speed-factor claim remains unidentified (Open question 109).

**Homing step, detonation (Medium-High for the steps; High for the range timing).** Each tick the slot computes the straight-line separation to its target, then moves 0.75 units along the unit direction toward that target (word 0x2DA638 = 0.75, read from disc; one step per tick). The detonation gate uses the separation measured before movement and passes only when it is strictly below 1.5 (fixed constant). A kind test (the blocker is the slot kind or kind 7) and a dot test of the target facing against the slot direction, with a threshold of 0.5, can block the detonation; a blocked detonation sets the state to 2. Otherwise it applies the orb damage (word 0x2DA634 = 10; SPEC-0054; the 10 is not an effect id, since no effect definition 10 exists) and its slot is released.

**Shared class-38 array (High).** The array at 0x36D3F0 is shared by Queen Fyora (class 38) and class 37 loop B, and only the Queen Fyora controller advances it. A new slot starts exactly at the caster's sample point. Each slot has a type word; the handler by type is:

| Type | Opened by | Each tick | Ends |
| --- | --- | --- | --- |
| 0 | Loop B state 3 | Moves 0.5 units (word 0x2DAAA8) toward Queen Fyora's aim point, horizontally; the height is kept. | When Queen Fyora deflects it (it becomes type 3). |
| 1 | Queen Fyora state 3 | The same step toward the class-37 object's aim point. | When loop B deflects it (it becomes type 3). |
| 2 | Loop B state 5 | The same step toward the position value at the hero's attach point 7 (the status display anchor documented in SPEC-0004), returned by the hero's method at +0x294. Why this slot targets that anchor is not established. | When the horizontal distance to that point, measured before the step, is below 2.0 (fixed constant): the slot is freed and, when current-world level-state id 2 is clear, a spawn by key (Behavior 31) is made of the class-37 object's row slot-0 key at the hero's position and facing, with the class-37 object as owner. |
| 3 | A deflect (Behavior 11, rule 4) | Moves 0.5 times its launch direction and counts its 60-tick timer down. | When the timer falls below 1: freed. When the hero is within 1.5 units (horizontal): the hero takes a hit of 5 (word 0x2DAAAC) with no attacker, then freed. |

Types 0 and 1 have no end of their own: an orb whose target never deflects it follows the target for ever. On disc row 128 carries key 2864 (Minion Warrior) and row 129 no key. So a type-2 orb summons a Minion Warrior when the class-37 object uses row 128 and level state 2 is clear (High for the keys; Medium for which row the duel uses).

Test vectors:
- Type 0 at (0, 7, 0), Queen Fyora's aim point at (3, 0, 4): one tick moves it to (0.3, 7, 0.4).
- Type 3 launched at (0, 2, 0) with direction (1, 0, 0): after 60 ticks it has moved 30 units and is freed, unless the hero came within 1.5 units first.
- Homing detonation boundary (Behavior 19): pre-movement separation 1.49 passes the strict 1.5 gate; exactly 1.50 fails that tick. With step 0.75 the remaining separation becomes 0.75, so the next tick passes. For the second form, pre-movement separation 2.00 fails the strict 2.0 gate, moves to 1.25, then passes on the next tick.

**Homing step, second form (class 37, type-1 member; High).** A second handler computes the straight-line separation to the hero before movement, then uses the same 0.75 step each tick toward the hero. It ends the member only when the pre-movement separation is strictly below 2.0. Its release action is the aimed-bolt summon of SPEC-0054: a spawn by key of the row slot-0 key with direct drop item 172.

**Flight step (High).** A flying slot moves by its velocity scaled by its speed factor each tick. A sphere test of radius 0.5 (word 0x2DAA6C) against the hero's position decides the hit. On a hit the slot plays its impact animation and changes state. Otherwise, the updated slot position is ground-probed at that position with an 8.0-unit vertical ceiling lift. When the slot's Y coordinate is at or below the returned ground height, the state becomes 2 and the slot ends. The value 8.0 is the probe's vertical lift, not a distance cap.

### 20. Enemy AI decision tick (Medium-Low; words High)

Each tick the enemy reads its state, its six target entries, the living sister count and two chance tables. The chance words are the Faerie Sisters' dodge chances (SPEC-0011): the melee-dodge chance and the missile-dodge chance, each indexed by the living sister count. The living count is held in a run-time word (0x36DC64, not in the file image; Open question 111).

| Living sisters | Melee-dodge chance (percent) | Missile-dodge chance (percent) |
| --- | --- | --- |
| 1 | 40 (0x2DAAD4) | 50 (0x2DAAC8) |
| 2 | 50 (0x2DAAD8) | 60 (0x2DAACC) |
| 3 | 60 (0x2DAADC) | 75 (0x2DAAD0) |
| 0, -1 or any other value | 0 (no chance) | 0 (no chance) |

A chance roll draws a whole number from 0 to 100, both included (101 values), and passes when the draw is strictly below the chance. Label confidence for the melee-dodge words is Medium.

The boot program's tunable registry names these words dodgeMissileChance1 to dodgeMissileChance3 (the missile column, one to three living sisters) and dodgeMeleeChance1 to dodgeMeleeChance3 (the melee column). A living count other than 1 to 3 gives 0 for both chances.

Per tick:

1. If the enemy's state is 9, nothing happens.
2. If no target is present, the engaged flag and the acquired flag are cleared.
3. If the acquired flag is clear, a target is present and the enemy main state is 0, 1 or at least 10, the acquired flag is set. States 2 through 8 fail this gate. State 9 returns at the start of the tick, before this gate or any other Behavior 20 work. The routine takes the unit direction from the sister toward the active hero and dots it with the sister's facing. When this dot is at most 0.5, the melee-dodge test is skipped and processing continues to the projectile-dodge check. When the dot is above 0.5 and the horizontal distance is below 7.0 (fixed constant), a melee-dodge roll is made with the chance for the living-sister count; on a pass the state becomes 3 (melee dodge, with no damage or movement) and the animation variant is drawn from 0 or 1. If the roll fails or distance is at least 7.0, processing continues to the projectile-dodge check.
4. Dodge cooldown: when no cooldown is running (value below 1), each hero projectile is tested for proximity with a capsule of radius 6.0 and length 2.0 (fixed constants). Each projectile that passes the proximity test sets the cooldown to a whole number drawn from 1 to 3, multiplied by the runtime frame rate of 60 ticks per second. Then a missile-dodge roll is made with the missile-dodge chance for the living sister count; on a pass the state becomes 2 (missile dodge) and the animation variant is a draw of 0 or 1. On a failed roll the check continues with the next projectile, so a later proximity hit can set the timer again.
5. Engage: when the engaged flag is clear, a target is present, and a sensor test with values 3.0 and 6.0 passes, the enemy starts its attack action with argument 1.0, runs its state step, and sets the engaged flag.
6. Contact scan: each valid target is tested with a capsule of radius 1.5 and length 6.0. On a hit an event is raised with the target's index, the target is removed from the list, and the enemy's state step runs.

Test vectors for this section are under "Test vectors".

### 21. Engagement probe and contact test (constants High; construction Medium-Low)

**Engagement check (Medium-Low).** For each of the six active projectile slots, the routine takes the slot's saved point and forms a probe 120.0 units long (fixed constant) along a direction derived from the slot's stored orientation vector and scale fields. For each entry of the enemy's target list (16-bit count in the list header; entries of 0x50 bytes) with a positive length (at entry offset 0x40), a capsule test runs between the probe and the entry, using the entry's length (offset 0x40) and radius (offset 0x44). The entries are skeleton bone records (SPEC-0040 Behavior 12A). If any test hits, the check passes. If none hits, the enemy's state is set to 0. On a hit, a sub-object callback is called with value 20 before block animation 5 is requested; the callback's operational effect remains unknown (Open question 115).

**Contact check (Medium).** First, if a gate on the sub-object passes and the enemy overlaps the query capsule (radius 6.0, segment length 10.0) the check returns a contact. The segment extends 5.0 units in each direction along its supplied axis; this is a capsule test, not a radius-6 sphere test. Otherwise each valid target is tested with a sphere of radius 5.0 against the target's point. On the first hit that target is removed from the list and the check returns a contact. If nothing hits, there is no contact.

### 22. Timed special attack: the Werelupe King's flame wave (cross-reference to SPEC-0019)

The routine that starts this attack is a timed trigger in the enemy framework. Its rules, the distance band, the phase chances, the states, the animation and the projectile waves are described once, in SPEC-0019 (Attack selection, Flame wave, and Flame-wave volley). This section keeps only the words the trigger reads: the distance band from 0x2DB124 (10.0) to 0x2DB128 (40.0), the cooldown word 0x2DB12C (10 seconds, times the tick rate) and the three chance words 0x2DB130, 0x2DB134 and 0x2DB138 (0, 50 and 75). Confidence: High for the words; Medium-High for the trigger rules.

### 23. Grab-and-throw machine of the common enemies (states 0 to 4)

Confidence: Medium for the state flow; Low for the names of the tests it calls. The machine is reached from the common-enemy class update (SPEC-0038): it is the grab-and-throw strike of an attack slot whose flags have bit 0x08 (SPEC-0038 Behavior 15), and "the target" below is the hero. It keeps a state word (0 to 4) and a shared countdown word, which is -1 at rest. The countdown word is the run-time word described under Where it lives. Its unit is the tick (60 per second, SPEC-0007). Each tick:

- State 0 (arm). Set the countdown to -1. While the busy test is clear, play the enemy's default cue (a helper gives its id). When the engaged test is clear and the enemy's readiness test passes, move to state 1 and load the countdown with 60 ticks.
- State 1 (choose). If the facing dot product toward the target is at most 0.5, or the distance to the target is at least the melee range of the enemy's selected attack (Layout section 4), move to state 3 and stop for this tick. Otherwise choose the grab point index from the animation event bits of the enemy's current clip frame, testing in this order: bit 0x02 gives index 1, bit 0x04 gives 2, bit 0x08 gives 3, bit 0x10 gives 4, and none of them gives 0 (High). The index selects one of the enemy's attach transforms (0 to 4), and that transform's position is the aim point. Then, if the hero's stun word (word 0x238 of the hero's sub-object) is zero and a path test toward the aim point is clear, put the hero in base state 53 (held; this step stores the hero's position as the last good position on every tick it runs), start the attack's effect (a call with argument 3, meaning open), request a controller rumble of preset 0 (constant) for 1.0 s at magnitude 0.7 (SPEC-0033 Behavior 1C), and move to state 2. If the stun word or the path test fails, move to state 3 and play animation 1.
- State 2 (strike). While the busy test is clear, the attack's cue plays each tick. When the hero is in base state 53 and the release test passes, the hero is thrown: an upward value of 8.0 and 15.0 times the enemy's facing are combined, normalized and scaled by the slot's throw speed (row 0x41C + 4k), so the throw velocity is normalize(15.0 x facing + 8.0 x up) x throw speed (Medium), and the state becomes 4. If the hero is still in base state 53, each state-2 tick also tests the hero's touched flag and the straight path from the stored last good position to the hero's present position. When the flag is clear and the path is open, the present position becomes the new last good position and the hero's no-gravity flag is set for the tick, so a held hero is not pulled down. When the flag is set or the path is blocked, the state becomes 3 and the hold ends: the hero is put back in base state 1, its velocity is cleared and its position is set to the stored last good position (Medium-High). The countdown runs in this state: on each state-2 tick, if the countdown is 0 it is reloaded with 60 and the same rumble (preset 0, 1.0 s, magnitude 0.7) is requested again; then the countdown falls by 1. Since state 1 loaded it with 60, the rumble repeats every 61 ticks while the hero is held (High for the rumble values and the period). Test vector: the grab on tick 0 (state 1) requests the rumble and leaves the countdown at 60; state 2 runs from tick 1, where the countdown reads 60 and falls to 59; it reads 0 at the start of tick 61, so the rumble is requested again on tick 61, and the next time on tick 122.
- State 3 (recover). Set the countdown to -1 each tick. While the busy test is clear, play the enemy's idle cue (a helper gives its id).
- State 4 (done). Set the countdown to -1. Nothing else happens.

States 3 and 4 never leave by themselves inside this routine. Two outside writes reset the state word (High): entering agro outer sub-state 2 (the attack sequencer, SPEC-0038) sets it to 1, so a new grab starts at the choose step; and the attack choice that selects the enemy's next attack slot (and draws that slot's cooldown, SPEC-0038) sets it to 0.

### 24. Alarm-group check of the common enemies

Confidence: High for the gate and the branches; Low for the helper meanings. This is the alarm-group check of SPEC-0038 Behavior 3, run every 15 to 45 ticks.

1. The routine runs only when the enemy's alarm group byte (byte 0xB7 of its spawn record) is not 0xFF and the group has a new broadcast for it (a member entered agro).
2. Let C be the enemy's current class state. C is compared with the set 0, 3, 5, 6, 7, 14 and 8.
   - If C is 0, 3, 5, 6, 7 or 14: set the enemy's state to 2, then play animation 18 (0x12). If either of two class tests passes, play animation 149 (0x95) instead.
   - If C is 8 (the special case): a helper gives the emerge animation (SPEC-0038 Behavior 20). If it returns -1, set the enemy's state to 19 (0x13). Otherwise play animation 10 and set the enemy's word at offset 0xBC0 to 2. If the enemy's word at offset 0xC4 is zero, call a blend with weight 1.0 and argument 4 (meaning open).
   - Any other C: no effect.

Behavior 10.3 is the Ixi Chief's incoming-hit check, a different routine from this check.

### 25. Pursuit

Confidence: Medium for the branches; the speed and turn weights are as given here. Each tick, with the chase target T (none = -1) and a second index U (from a helper):

1. No target: walk toward the enemy's route point (from a helper) at a speed of 0.1 per tick (the word at 0x2DD86C, read from disc as 0.1), playing the walk animation through the animation slots.
2. A target, but U is none: play the hold animation (from a helper) and stop.
3. Otherwise, keep the previous chase point for the change test, then take the distance d from the enemy to its chase point.
   a. If d is below 0.25, or the enemy's counter at offset 0xBD0 equals 120 while the flag at offset 0xCC4 is nonzero: set the arrival flag (offset 0x9B8 = 1), play animation 1, turn toward the target point (the hero's position, from a helper) with blend weight 0.1 per tick, and stop for this tick.
   b. Else, if T or U changed since the last tick: re-plan the route to the chase point with a route search that uses the speed word. If no route is found, stop for this tick; otherwise follow the new route.
   c. Else: turn toward the chase point with blend weight 0.3 per tick and play the chase animation (from a helper).

### 26. Part capsule hit sweep

Confidence: High for the walk, the record fields and the shape; Medium for the lane reading of the posed frame. This is the weapon-against-target test of the weapon volumes (Behavior 10.2). It receives one weapon cylinder (radius, full height, centre and axis from the attacker's attachment frame) and walks the bone capsules of the target, not of the attacker. It first clears a 16-byte block in the target's controller state (offset 0x18C0). It then walks the target's skeleton bone table in index order (SPEC-0040 Behavior 12A). A bone is skipped unless its length (bone record +0x40) is above 0. For each such bone, its base point (+0x30) is posed by the bone's current world frame (the four pose vectors at offsets 0x290 to 0x2C0 of the frame): P = posed base point, Y = the frame's local Y axis. The capsule is a flat-ended finite cylinder of radius +0x44 and full height +0x40 along Y, centred at P + 0.5 x length x Y. The weapon cylinder and the capsule are tested with the static finite-cylinder overlap test of SPEC-0018 (no sweep). The routine writes its working vectors into an output record (offsets 0x390 to 0x3AC) for each tested bone; their meaning is open. It returns 1 at the first hit and 0 if no bone is hit.

### 27. Enemy status: apply, element immunity and per-tick rules (High for the apply, the element gate, the per-tick values and status 11; Medium for the meanings of the two tests A and B and for sub-state 11; Medium-Low for the flinch rule)

The enemy record holds 14 status timers, one per status id 0 to 13, at offsets 0x808 to 0x83C (SPEC-0040), and four per-enemy words: poison amount (0xCB4), poison period in seconds (0xCB8), fire amount (0xCBC) and fire period in seconds (0xCC0).

#### 27.1 Applying a status to an enemy

A status is applied by calling the target character's status-apply method with a record of four words: status code, duration in whole seconds (-1 = the default), and two extra values (the poison or fire amount and period; -1 = the default). Heroes and enemies each have their own method; other characters ignore the call. Every source calls the method of the target, so the rules below hold for every status that reaches an enemy: the heroes' Mote attacks and charms, enemy spells, pickups and anything else. The hero's method is in SPEC-0036.

Enemy apply. Nothing happens if the enemy is dead (class state 16, or dying). Let T be the enemy's target type (monster row 0x3C, Layout section 4). By code:
- 0 stun (default 5 s): nothing when T = 4. Otherwise the enemy is put in class state 4 at once and the timer is set.
- 1 curse (default 60 s): nothing when T = 6.
- 2 poison (default 120 s): nothing when T = 2. Otherwise the two extra values become the enemy's poison amount and period words.
- 3 slow (default 60 s): nothing when T = 5.
- 6 confusion (default 30 s): nothing when T = 3.
- 7 fire (default 5 s): nothing when T = 1. Otherwise the two extra values become the enemy's fire amount and period words.
- 4 strength (default 60 s), 5 speed (default 60 s) and 11 poison attack (default 30 s): no element gate.
- 8, 9, 10, 12, 13 and any other code: nothing. Enemies have no immunity statuses and no regeneration.
- Timer: ticks = seconds x 60 (the tick rate). When ticks >= 0 the status timer is set and the status display start runs (also for a duration of 0). A negative duration other than -1 changes nothing.

Element immunity (High). An enemy is immune to the status of its own element: T 1 blocks fire (7), T 2 blocks poison (2), T 3 blocks confusion (6), T 4 blocks stun (0), T 5 blocks slow (3) and T 6 blocks curse (1). T 0 and T 7 block nothing. By the likely labels of Layout section 4b (Medium-Low for the labels): fire against fire, leaf and rock against poison, light against confusion, water against stun, fog and wind against slow, smoke and shadow against curse. The blocked apply shows no effect.

Stacking (High). A new apply overwrites the timer and the extra words. Durations never add up.

Sources on enemies (from the attacker's side; confidence per line):
- Spell kinds 3, 4 and 7 (Behavior 9): status 4 (strength), 5 (speed) and 11 (poison attack) at the default duration. High.
- Hero power attack with a Mote in slot 2, and the Water Mote stun object of either hero: the Mote's status (below). Medium-High (Water stun object: Medium).
- Tor's Lava charm hit: fire with Tor's Lava values. Roberta's Lava hit: fire with Roberta's Lava values, on half of the hits (a random 0 or 1 equals 1). Roberta's Leaf effect hit: poison with Roberta's Leaf values (Medium). Tor's Shadow charm and Sun charm: curse and confusion with Tor's duration. Medium-High (SPEC-0036).

Mote statuses (High for the table and the lookup). The status code comes from the Mote's family (SPEC-0021): Fire and Lava give fire, Water gives stun, Leaf and Rock give poison, Fog and Wind give slow, Smoke and Shadow give curse, Light and Sun give confusion; Bubble and the remaining Motes give none. The duration, amount and period come from design table 16 of resource 533 (60 f32 words; SPEC-0024): words 0 to 29 hold Tor's values and words 30 to 59 Roberta's, and a word of -1 means the default. Examples from the disc: Tor's Leaf Mote poisons for 34 s at 5 HP every 4 s, Roberta's for 34 s at 7 HP every 4 s; the Fire and Lava Motes burn for 12 s at 5 HP every 2 s; the Water Mote stuns for 8 s. The Rock Mote passes -1 for all three, so it uses the enemy defaults (120 s at 2 HP every 4 s).

Status 11, poison attack (High). Status 11 is an enemy buff with no per-tick effect. While it is active, every melee hit that the enemy lands also applies poison (status 2) to the target at the default (120 s; default amount and period), through the target's own apply method, so a hero with status 9 active is not poisoned. It is given by spell kind 7.

#### 27.2 Per-tick rules

The status tick runs every tick for a common enemy (class 17): it writes that class's state and agro outer sub-state (SPEC-0038 Behaviors 5 and 44). Earlier in the same enemy update the speed scale is set to 1.0. Then, in this order:

1. Timers: each timer of 1 or more counts down by one. A timer below 1 runs the status display's expire check for its status every tick (SPEC-0034: when the shown status is no longer active, the next active status is shown); the timer is not reset.
2. Stun (status 0) active: the speed scale is set to 0 and the class state to 4 (stunned), every tick while the stun lasts. State 4 returns the enemy to agro (class state 0x13) once the stun is over (SPEC-0038 Behavior 2). While the enemy is stunned, a hit on it does not play its flinch (Medium-Low).
3. Poison (status 2) active: P = the poison period word (4 when negative), A = the poison amount word (2 when negative). On the ticks where the status-2 timer (offset 0x810) is a multiple of P x 60, the enemy takes a hit of A hit points from attacker id -2 with the reaction flag 0, so the enemy flinches (unless stunned). There is no health floor: poison can kill an enemy. Only the hero's own status tick stops poison at 1 HP (SPEC-0036). So the default poison deals 2 HP every 4 seconds.
4. Slow (status 3) active: the speed scale is multiplied by 0.5 when test A (moving) or test B (attacking) passes.
5. Strength (status 4): no action here (it scales the enemy's outgoing damage, below).
6. Speed (status 5) active: the speed scale is multiplied by 2.0 when test A (moving) passes.
7. Confusion (status 6) active and test B (attacking) fails: agro outer sub-state 11 (confused) is requested, unless the current sub-state is 2 (the attack sequencer). In sub-state 11 the enemy plays animation 69, walks in a random horizontal direction drawn each tick at speed 0.05 per tick (word 0x2DD540), and runs its attack choice when it is within the sum of the two radii plus 0.25 of the current hero (SPEC-0038 Behavior 44; Medium).
8. Fire (status 7) active: as poison, with the fire words (period 2 s and amount 2 HP when negative), counted on the status-7 timer (offset 0x824), from attacker id -1, reaction flag 0, no floor. So the default fire deals 2 HP every 2 seconds.
9. The status display tick runs last.

Tests A and B are two class tests; by their use they mean "moving" (A) and "attacking" (B) (Medium).

Outgoing damage scale (High): an enemy's damage is multiplied by 0.5 under curse (status 1) and by 1.5 under strength (status 4) (Behavior 13). The hero's and the base character's factors are 0.75 under curse and 1.5 under strength (SPEC-0036).

Test vectors:
- Tor's Leaf Mote on an enemy with T not 2: 34 s = 2040 ticks, period 240 ticks. Hits at timer readings 1920, 1680, ..., 240: 8 hits of 5 = 40 HP. Roberta's Leaf Mote: 8 hits of 7 = 56 HP. An enemy with T = 2: nothing is applied.
- Fire Mote: 12 s = 720 ticks, period 120: hits at readings 600, 480, 360, 240 and 120: 25 HP.
- An enemy at 1 HP takes the full default poison hit of 2 (a hero at 1 HP would take 0).
- Stun on an enemy with T = 4: nothing (no timer, no state change). On any other living enemy: class state 4 at once, timer 300.
- Poison attack (status 11): an enemy with status 11 active bites the hero (its row status -1): the hero is poisoned for 120 s.
- Duration 0 on an enemy: the timer is set to 0 and the display start runs, but the status is not active.

### 28. Miamouse (class 21) (High)

**Who.** Monster type 104 (row 248): 21 placed records, all unnamed, in resources 538, 759 (10 records), 1164, 1281 and 1400. Row values used here: radius 1.175, height 3.5, hit points 5, no attack slots, no sense box, no timers, no flying. The row sets no damage and no state of this routine reads a damage value, so the miamouse deals no damage (Medium).

**Start (High).**
- The Miamouse sets its catchable word to 1.
- It then finds its own navmesh: the first navmesh of the current world whose triangles contain its position (the inside test of SPEC-0018). When none does, it logs an error saying that no mesh was found.
- It sets six AI parameters to 2, 2, 3, -1, -1 and -1, and a seventh to -1.
- Its target is always the current hero. It makes no target search.
- Each tick it clears the motion-record word at +0x284, runs the dispatcher, runs the current state (0: play clip 0; 1 to 4 below) and waits one tick.

**Main state (the state word).** 0 idle; 1 wander and pause; 2 flee; 3 released and landing; 4 delivered (caged).

**Dispatch (each tick, before the state runs; High).** Let u and d be the horizontal unit direction and the horizontal distance from the Miamouse to the hero (y ignored). d is a plain distance, not a squared one.
1. If clip 13 is the current clip, the state becomes 3.
2. If the state is 2, 3 or 4, stop.
3. If the hero's action state is 1 (standing), make no reaction (go to 5).
4. If d < 10.0:
   - Hero not in action state 22, 39 or 40 (the sneak states of SPEC-0033): run the flee entry.
   - Hero sneaking: run the flee entry only when dot(F, u) > cos(1.0 rad) = 0.5403, where F is the Miamouse's forward. So a sneaking hero is noticed only inside a cone of 1 radian round the Miamouse's facing.
   - Otherwise go to 5.
5. If the state is 0, the state becomes 1 with sub-state 0.

Flee entry: state 2, sub-state 0, timer 0, flee point = far centroid pick (radius 30.0) of the Miamouse's own navmesh measured from the hero's position. The flee is always away from the hero.

Far centroid pick (radius r, mesh m, point p):
- Up to N draws, with N the triangle count of m.
- Each draw picks a triangle uniformly and takes its centroid c. It stops at the first c with horizontal distance |c - p| > r.
- When none qualifies, the last drawn centroid is used. With no mesh m, the result is p.

**State 1 (wander and pause).** Sub-state and a countdown timer in ticks. On every tick of state 1 the catchable word is set to 1.
- Sub-state 0: set the timer to 0 and go to sub-state 1.
- Countdown (High): at the start of every tick of state 1, in every sub-state, the timer goes down by 1 when it is above 0. The tests below see the value after this step.
- Sub-state 1, timer above 0: play clip 0 (idle). When the timer is a multiple of 90 (90 or 180), go to sub-state 2 (emote).
- Sub-state 1, timer at 0: the destination is the centroid of a uniformly random triangle of the Miamouse's own navmesh (when the world has no navmesh list, its home point). Go to sub-state 3.
- Sub-state 2 (emote): request clip 20 until it is the current clip, then wait until it has played to 0.99 and return to sub-state 1 (no timer test on that tick). The countdown keeps running during the emote, so a multiple of 90 that falls inside the emote is skipped: a second emote happens only when the timer is still above 90 on the tick the emote ends (High).
- Sub-state 3 (move): move toward the destination in walking mode (mode 0; clip 2 for the Miamouse, from the values its entry sets for modes 0 to 6: 2, 2, 3, -1, -1, -1, -1 for modes 0, 1, 2, 3, 4, 6, 5; High). When the distance to it is below 1.0 unit, set the timer to a random whole number of ticks in [120, 210] (2.0 to 3.5 s) and go to sub-state 1. The first test of a new pause sees the drawn value minus 1, so a draw of 181 emotes on the first pause tick and a draw of 180 does not emote until the timer reaches 90 (High).

**State 2 (flee).** On every tick of state 2 the catchable word is set to 0, and the Miamouse runs (movement mode 2; clip 3, the third of its entry's slot values, High). Distances to the hero are horizontal.
- Sub-state 0:
  - If the motion record's touched flag (+0x238) is set: go to sub-state 1 with a far centroid pick of radius 10.0 from the hero, and play clip 0.
  - Then, if the hero is at most 20 units away, aim at P + 4 x v, where P is the Miamouse's position and v the horizontal unit direction from the hero to the Miamouse. This is a step away from the hero. When that point is not walkable, go to sub-state 1 with a radius-10 pick instead.
  - If the hero is more than 20 away: go to sub-state 2 with a radius-30 pick.
- Sub-state 1: run to the point. On arrival within 0.5, or when the touched flag is set: return to sub-state 0 if the hero is within 20 units, else go to sub-state 2 with a new radius-30 pick.
- Sub-state 2: run to the point. On arrival within 0.5, or when touched: main state 0.

**State 3 (released and landing; High).** Each tick: catchable 0; navmesh none.
- While clip 13 is current:
  - If the Miamouse's position is in the cage zone (below): navmesh = cage mesh. Let c = far centroid pick (radius 3.0) of the cage mesh from its position. Its move target becomes c, and its position becomes (c.x, its own y, c.z) with velocity and displacement cleared.
  - Else, if its position is not on the field mesh either: its position becomes its home point (its spawn position).
  - Then, if its ground gap is below 0.25 (the gap is 0 while the floor flag is set; SPEC-0040), play clip 19 (landing).
- While clip 13 is not current: when the current clip's progress has passed 0.99:
  - In the cage zone: navmesh = cage mesh, the same centroid placement as above, main state 4, then the delivery step.
  - Otherwise: navmesh = field mesh, then the flee entry.
- The landing is decided on the tick on which the landing clip ends. There is no trigger volume.

**State 4 (delivered; High).** The Miamouse runs the state-1 routine (so it wanders between centroids of the cage mesh), then sets its catchable word to 0. A delivered Miamouse never leaves state 4 and cannot be caught again.

**Catch and cage (level-script interface; High).** These rules are global to the boot program, not per pack: one catching switch, one pair of mesh ids and one count.

- Setup (InitMiamouseBehavior(m1, m2)): field mesh = m1, cage mesh = m2, catching on, count = 0. Disable (MiamiceCatchDisable): catching off; the count is kept. Count query (NumMiamiceInCage): the count. At boot both mesh ids are -1, the count is 0 and catching is off. Disc users: 1163 passes (0, 1), 1398 passes (15, 16).
- Cage zone: a position is in the cage zone when its x and z lie in a triangle of the current world's navmesh with the cage id. This is the inside test of SPEC-0018: points on edges count, height is ignored. With the cage id -1 or out of range, nothing is in the zone. The field test is the same with the field id.
- **Catch** (hero side; SPEC-0022 chooser, not-busy branch; SPEC-0040 state 57). The prompt "Catch Miamouse" (label 27) shows when:
  - catching is on;
  - there is a character of class 21 whose catchable word is set (it is idle or wandering; never while fleeing, landing or caged);
  - its horizontal distance from the hero is below 4.0;
  - the horizontal dot product of the hero's forward with the unit direction from the hero to it is above 0.5.
  The first such character in the world's character list is taken. A Square press edge enters hero state 57 and stores that character as the carried Miamouse.
- **Carry (hero state 57; sub-states 0 pick up, 1 stand, 2 walk, 3 put down, as in SPEC-0033 Behavior 18).** The catch enters sub-state 0, and the hero plays pick-up clip 178. The hold runs on every tick of sub-state 0 once clip 178 is the current clip (from its second tick), on every tick of sub-states 1 and 2, and on the first tick of sub-state 3 (before the release). On each such tick the hero:
  - holds the Miamouse at its attach point (SPEC-0005, Attach points and carrying, scale 1.0);
  - puts it in base state 33 (not hittable);
  - requests its clip 12;
  - sets its navmesh to none.
  The Miamouse's own routine keeps running. The hold overrides its position, clip and navmesh. The catch does not change the Miamouse's main state word. The hold is skipped only on the first tick of sub-state 0, while clip 178 is being requested and is not yet current; from the second tick on the Miamouse is held during the pick-up clip, so it can run for at most one tick after the catch (High). In sub-states 1 and 2 the hold runs every tick; the hero stands with clip 179 and walks with clip 180.
- **Drop.** In sub-states 1 and 2 the prompt "Drop Miamouse" (label 28) shows, and Square sets sub-state 3. The hero plays clip 181. The release happens on the tick after clip 181 starts; when 181 ends the hero returns to state 1. Leaving state 57 by any other path while holding a Miamouse releases it at once.
- **Release.** The released Miamouse:
  - leaves the hold (state 33 to 1);
  - plays clip 13;
  - gets the cage mesh when its own held position (the hero's attach point) is in the cage zone, else the field mesh;
  - is placed at its held position + 1.0 x the hero's forward, with velocity cleared.
  The dispatcher then puts it in state 3.
- **Landing outcome.** Inside the cage navmesh: delivered (state 4 and the delivery step). Elsewhere on the field: it flees, and can be caught again once its flight ends. Off both navmeshes: it goes back to its spawn point.
- **Delivery step.** Count = count + 1. Then, on the current world's level states: take the lowest k of 0, 1, 2, 3, 4 whose state is on; set state k + 1 on and state k off. When none of 0 to 4 is on, nothing else changes. These are the same level-state set and clear operations that SetLevelState uses (SPEC-0030 row 152). So with only state 0 on at the start, delivery n turns state n on and state n - 1 off.
- **Count needed and cages (High).**
  - Pack 1163 needs 5 deliveries and places exactly 5 Miamice (region 1164), so every Miamouse must be caged. Field navmesh 0: 42 triangles in x -108.0 to -57.4, z 60.5 to 109.3. Cage navmesh 1: two triangles forming the axis-aligned rectangle x -70.91 to -65.49, z 55.30 to 60.45 (5.42 by 5.15 units, area 27.9), at floor height -126.4, on the south edge of the field. The fifth delivery turns level state 5 on, and the pack's Update then sets flags 158 and 23 (SPEC-0045).
  - Pack 1398 needs 4 (its stage check tests the count for exactly 4) and places exactly 4 Miamice (region 1400). Cage navmesh 16: a near-rectangle x -2.54 to 2.15, z 915.85 to 919.57 (4.7 by 3.7 units, area 17.2), inside field navmesh 15. Pack 1398 does not set level state 0, so there the delivery step only adds to the count.
  - There is no timer, fail state or penalty.

**Constants (fixed program constants unless marked).** Catch range 4.0 (horizontal, strict). Catch facing cosine 0.5 (strict). Notice range 10.0 (horizontal distance, strict). Sneak notice cone: cos(1.0 rad) = 0.5403 (strict). Release distance 1.0 (a data word; the only one). Wander arrival 1.0; point reached 0.5. Cage landing pick radius 3.0. Landing gap 0.25 (strict). Clip-end progress 0.99. Flee radii 10, 20 and 30; flee step 4. Timers in ticks: wander pause 120 to 210 (closed integer draw); emote at multiples of 90. Clips: Miamouse 0 idle, 2 walk (mode 0), 3 run and flight (mode 2), 12 held, 13 released (flail), 19 landing, 20 emote; hero 178 pick up, 179 and 180 carry, 181 drop.

**Test vectors (synthetic unless a pack is named).**
- Wander pause. A pause draws 120. The countdown reaches 90 after 30 ticks, so the emote starts on tick 30 of the pause (assumes the check follows the decrement; Medium).
- Notice. The Miamouse is at (0, 0, 0), facing (0, 0, 1), in state 1.
  - Hero walking (state 3) at (0, 0, 9.9): flee. At (0, 0, 10.0): no reaction.
  - Hero standing (state 1) at (0, 0, 1.0): no reaction.
  - Hero sneaking (state 40) at (0, 0, 5): dot 1.0, flee. At (4, 0, 3): dot 0.600, flee. At (4.2, 0, 2.6): dot 0.526, no reaction. At (5, 0, 0): dot 0, no reaction.
- Catch prompt. Hero at (0, 0, 0), facing (0, 0, 1), catching on.
  - Miamouse in state 1 at (0, 0, 3.9): shown. At (0, 0, 4.0): not shown.
  - At (2, 0, 3): distance 3.606, dot 0.832, shown. At (3, 0, 1.5): distance 3.354, dot 0.447, not shown.
  - In state 2 at (0, 0, 2): not shown.
  - Catching off (before the keeper talk in 1163): never shown.
- Delivery step. Level states {0} on, count 0. Five deliveries give {1}, {2}, {3}, {4}, {5} with counts 1 to 5. A sixth gives count 6 with {5} unchanged. With no state of 0 to 4 on (pack 1398), each delivery only adds 1 to the count.
- Cage zone, pack 1163 (cage mesh 1).
  - In: (-68.20, y, 57.88), and the corners (-65.49, y, 55.30) and (-70.91, y, 60.45).
  - Out: (-65.40, y, 57.88) and (-68.20, y, 60.50).
- Cage landing pick, pack 1163. The triangle centroids are (-67.30, 57.02) and (-69.10, 58.73).
  - From (-68.0, 57.0) the distances are 0.70 and 2.06. Neither is above 3.0, so after two draws the second drawn centroid is used.
  - From (-70.5, 55.5) the distances are 3.54 and 3.52, so the first draw is used.
- Release. Held point (-68.0, -125.0, 56.0), hero forward (0, 0, 1). The mesh is the cage (the held point is in the zone). The Miamouse is placed at (-68.0, -125.0, 57.0) and plays clip 13. When its landing clip ends inside the zone, it reaches state 4 and the count goes up by 1.
- Hold timing. On the first tick of state 57 (sub-state 0) clip 178 is requested and the Miamouse is not held, so its own routine still moves it on that tick. From the second tick on clip 178 is current, and the Miamouse is held at the attach point in base state 33 with clip 12. When 178 reaches 0.99 the sub-state becomes 1 and the hold continues with clip 179. Drop (as SPEC-0033 test vector 137): Square on tick n sets sub-state 3, requests clip 181 and holds once more on tick n; the release comes on tick n + 1.
- Pause and emote. A pause draws 150 on the arrival tick. The timer is 149 on the next tick and 90 sixty ticks after the arrival, which starts the emote. If the emote lasts 40 ticks, the timer is 50 when it ends, no second emote follows, and the next destination is picked when the timer reaches 0, 150 ticks after the arrival. With a draw of 200 the first emote starts at timer 180 (20 ticks after the arrival); with an emote of 40 ticks it ends at timer 140, so a second emote starts at timer 90 (110 ticks after the arrival).

**Edge cases.**
- While a Miamouse is held, its dispatcher still sees the hero (state 57) within 10 units, so its main state becomes 2. This has no visible effect: the hold overrides position and clip, and the release forces state 3 through clip 13 (Medium).
- During clip 13 inside the cage zone the Miamouse is moved to a cage centroid on every tick, with velocity cleared each time, so it hops between the centroids of the 1163 cage while it falls and the fall is slowed. Reproduce this, or accept it as cosmetic (Medium for the visual effect, High for the rule).
- The gap between the 1163 field (z from 60.5) and the cage (z up to 60.45) is 0.05 units. A Miamouse landing exactly there is on neither mesh and is put back at its spawn point.
- After the 1163 win catching stays on until 1163 next becomes current. A pack holding class-21 characters entered while it is on would allow catches against its own navmeshes 0 and 1; on the story path this does not happen (Medium).

**Confidence.** High for the states, the dispatch, the catch, carry, release and delivery rules, the counts and the cage meshes. Medium where marked. No damage is specified.

### 29. Target Dummy (class 36) (High for structure, interval and hit reset; Medium for animation indices)

**Who.** Monster type 110 (row 254): seven placed records, all named "Target Dummy", in resources 1057 (1), 1058 (3), 1059 (1) and 1060 (2). Row values used here: radius 1.175, height 3.5, hit points 50, no attack slots, no sense box, death-disappear field 510 (unit not confirmed). It never attacks.

**Behavior.**
- A pending hit flag (+0xABC), set by the shared take-hit on every hit that is not ignored or absorbed (Behavior 34; Open question 132), is consumed on a hit. Each hit:
  - plays animation 3 (hit reaction);
  - sets current hit points to the maximum hit points;
  - sets the twitch counter to 1;
  - draws a new twitch interval.
- The twitch interval is a random whole number of ticks in [400, 800] (6.67 s to 13.33 s). It is drawn at start and after each hit.
- The counter increases by exactly 1 on each tick that is not a hit tick, except while animation 3 is playing and not finished (High). On such a tick nothing else happens. The twitch, idle and twitch-still-playing branches each add the 1; none adds a second count.
- On any other tick, if the counter is a multiple of the interval, play animation 1 (twitch). Otherwise, if animation 1 is not current or has finished, play animation 0 (idle). If animation 1 is still playing, do nothing else.
- The idle call repeats every tick. A request for the id that is already requested does not restart the clip: the clip request stores a new id, and resets the clip's play state, only when the id differs from the current request (High). So the idle clip plays on as it would with a single request.

**Edge cases.** A hit on the same tick as a twitch: the hit branch runs first, so the twitch is skipped. The first twitch comes when the counter equals the interval, so it falls one full interval after start, not at a random moment.

**Test vectors (synthetic).** Interval 400 at start. A hit at tick 10 sets the counter to 1 and draws 650. The first twitch then comes when the counter reaches 650, 649 ticks after the hit (Medium). With interval 400 and no hit, the twitch is requested on the tick the counter is 400; the counter is 401 on the next tick, and twitches recur at 800, 1200 and so on while no hit comes (High). An idle clip 0 that is current on tick t and requested again on tick t + 1 keeps its progress.

**Confidence.** High for the structure, the interval and the reset of the counter on a hit. Medium for the animation indices (0 idle, 1 twitch, 3 hit). Current hit points are set to the maximum on every hit: the write is certain; that this field is the current hit point value is a convention (Medium).

### 30. Queen Fyora controller (class 38) (High for the wait, the states, the slots, the chooser and the cooldown range; Medium for the deflect direction's side)

**Who.** Monster type 58 (row 132): four placed records, two named "Queen Fyora" (resources 1318 and 1322) and two unnamed. Row values used here: hit points 50 (template), radius 1.175, height 3.5, no attack slots. The link between this controller and the Queen Fyora name is Medium.

**Main loop.**
1. Wait, one tick at a time, until the character list holds a class-37 object (the kind search of Behavior 7).
2. Each tick, in this order:
   - advance the four shared slots (Behavior 19);
   - run the state chooser;
   - turn this object's facing, with blend 1.0, along the horizontal unit vector from Queen Fyora toward the class-37 object (High);
   - run the state.

**States.** Only states 0, 2 and 3 are ever entered; the state starts at 0.
- State 0 (idle): play animation 26.
- State 2 (deflect): play animation 23. On the first tick it is current and not finished, relaunch the first active type-0 slot (the Darkest Faerie's orb) as type 3 and play effect 4007 on it (Behavior 11, rule 4). With no active type-0 slot nothing is launched. When the animation finishes, return to state 0.
- State 3 (cast): play animation 22. At the animation's event, once, open a type-1 slot at her sample point (her hand or staff point; Medium); it homes on the class-37 object (Behavior 19). When the animation finishes, return to state 0 and set the cooldown to a random whole number of ticks in [30, 45] (0.5 s to 0.75 s).

**State chooser.**
- If a type-0 slot is active and lies within 1.5 units of Queen Fyora (horizontal): state 2, and the deflect flag is cleared. This test runs every tick, so it can restart state 2.
- Otherwise, if the state is 0 and no type-1 slot is active: if the cooldown is below 1, go to state 3 and clear the event flag; otherwise count the cooldown down by 1.

**Test vectors.**
- Cooldown (synthetic): cooldown set to 32 on leaving state 3. Ticks 1 to 32 count it down to 0 (the chooser counts before the state runs). On tick 33 the cooldown is below 1 and the state becomes 3.
- Recovery bounds (synthetic): at 60 ticks per second, the state-3 recovery draw uses [trunc(60 x 0.5), trunc(60 x 0.75)] = [30, 45] ticks, inclusive. The bounds are scaled from fixed second values, not read from a tuning word.
- Aim: Queen Fyora at (0, 5, 0), the Darkest Faerie at (3, 9, 4). The aim vector is (0.6, 0, 0.8).
- Chooser: a type-0 slot at horizontal distance 1.49 from her gives state 2; at 1.5 it does not. While her own type-1 orb is in flight she does not cast again.

### 31. Spawn by key (summons) (High for the search, the reserved-record rule, the start in agro and the four sources; Medium for the spawn direction of the member summons)

Some spawn records are reserved for summons. A kind-3 record whose summon byte (data +0xB6) is nonzero is never brought in by the proximity streaming of SPEC-0006 (character activation and streaming): its record exists in the block but is skipped by every streaming bind. 208 records on disc are reserved this way. All 180 kind-3 records whose whole position (x, y, z, w) is zero are among them: their spawn point is supplied by the summoner, not by the record.

**Spawn-by-key request.** A request names a key, a point, a facing and two values passed on to the spawned character. It searches one character block for the first record, in record order, that (a) has a key (s16 at data +0xAC) equal to the requested key, (b) has a nonzero summon byte and (c) is not bound to a character object. The search runs only while the block has a free character object and its active count is below its cap (SPEC-0006). The found record is bound at the given point with the given facing, starting from its monster row at full health, and the request reports success. It requests its default class at priority 1, then takes class-17 state 19 (agro). For the minion keys 2801 to 2807, 2811 to 2817, 2821 to 2827, 2831 to 2837, 2841 to 2847, 2851 to 2857, 2861 to 2867 and 2871 to 2877 it instead plays animation 82 and takes state 21. A request with the arrival flag set (the breakable source) also plays animation 82 on the agro path. The request's drop item is stored as the spawned character's direct drop item (Layout section 1), and its summoner id is stored on it (High). With no such record, no free object or the cap reached, nothing spawns and the request reports failure.

- A request made by a character searches that character's own block (the pack's embedded region, or the script slot it was loaded into).
- A world-level request searches the embedded block first and then each created slot in slot order, and stops at the first success.
- A summoned enemy is a normal character of its block from then on. Whether its kill is remembered follows the kill memory rules of SPEC-0006 (Behavior 2, Kill memory: a kill in a world later released by streaming or by an interior job is forgotten); IsAI_Dead tests a summon-reserved record by the live state of its bound character (SPEC-0030).

**Sources of requests.**

1. Member summon (High): the class-37 Darkest Faerie's type-1 member, on reaching the hero, spawns the row's slot-0 spawn key (row offset 0x3CC) once, at the hero's position + 4 u facing the hero, with direct drop item 172 (Behavior 7). The second form is the type-2 orb of the shared class-38 array (Behavior 19): it spawns the slot-0 key of the class-37 object's row at the hero's position, facing opposite the hero's forward direction, with the class-37 object as summoner, only while level state 2 is clear. Row 128 carries key 2864 and row 129 none, so it summons only when the duelling Darkest Faerie uses row 128 (Medium).
2. Volley (High): the Darkest Faerie's volley start spawns the row's slot-1 spawn key (row offset 0x3CE) four times, each with direct drop item 172 (Behavior 18).
3. Lobbed summon (High): a missile slot with row flag 0x02 fires a kind-8 projectile into the enemy projectile pool (Behaviors 9 and 13). The projectile carries the slot's spawn key (row 0x3CC + 2k) and, when it lands, touches the world or loses its owner, makes a world-level request (embedded block first, then each created slot) at its position, facing its forward axis, with its owner as the summoner. An expiry in the air spawns nothing.
4. Breakable (High): a breakable whose spawn key word (+0x54, SPEC-0048) is 0 or more makes a world-level request for that key at its break point and facing, with the arrival flag set. On disc 20 breakables carry a key; for example key 1960 in pack 732 matches the two reserved Sludgy records of that pack.

The spawn entity count of the slot (row offset 0x3D4 + 2k, SPEC-0038 Layout 2) limits how many live summons one enemy keeps from that slot; for a class-17 enemy this is the spawn gate of the attack choice (SPEC-0038 Behavior 7, step 2e), read through the same flag 0x02.

**Summoning rows on disc (High).** Only these rows hold a spawn key (Layout section 4): row 128 (Darkest Faerie family: slot 0 Minion Warrior, key 2864; slot 1 Minion Grunt, key 2844; four each), row 211 (Minion Wizard, fire variant: slot 0 Minion Grunt, four), row 215 (Minion Wizard variant: slot 0 Minion Archer, key 2807, two) and row 221 (Spore Monster: slot 3 Flower Monster, key 619, five). Row 211 slot 0 (flight mode 0) and row 221 slot 3 (flight mode 1) are missile slots with flag 0x02, so they summon through the lobbed summon (source 3): the Minion Wizard lobs a Minion Grunt and the Spore Monster lobs a Flower Monster, each landing where the shot comes down, up to the slot's live count. Row 128 slot 0 (a melee slot with flag 0x02) and row 215 slot 0 (an unused slot type) are not missile slots. Row 128's slot-0 key is read by the member summon (source 1) and its slot-1 key by the volley (source 2), both run by the class-37 Darkest Faerie (High). Row 215's slot-0 key is read by none of the four sources, so it never spawns (Medium).

**Matching records.** A summon succeeds only when the summoner's block holds an unbound reserved record with the key. For example the reserved Minion Grunt and Minion Warrior records sit in the blocks of the fights that summon them (Root Cellar, Fyora's Tower, the final fight), and 90 reserved Flower Monster records sit in the Shadowglen Woods blocks.

### 32. Monster footstep camera shake and rumble (High)

Each pass, a monster checks its footsteps. The check acts only when the monster's current animation frame number differs from the last one it handled and the animation frames passed since the last pass carry event flag 0x80 or 0x100. Let d2 be the squared distance from the camera position to the monster.

- Camera shake (row 0xB8). Class 1, 2, 3 give strength 0.5, 1.0, 1.5; class 0 gives none (other nonzero values give 1.0). The strength is kept as it is when d2 <= 100 (10 units), multiplied by (1 - (d2 - 100) / 2600) when 100 < d2 <= 2500, and 0 beyond 2500 (50 units). A positive strength starts camera shake preset 0 with duration 1.0 s and that strength (SPEC-0027).
- Rumble (row 0xBA). Class 1, 2, 3 give magnitude 0.5, 0.75, 1.0; any other value gives 0. The magnitude is kept as it is when d2 <= 1, multiplied by (1 - (d2 - 1) / 901) when 1 < d2 <= 900, and 0 beyond 900 (30 units). A positive magnitude requests a rumble of preset 0 (constant) with duration 1.0 s and that magnitude (SPEC-0033 Behavior 1C).
- The frame number is remembered whenever the flag test passes, so one footstep frame gives one shake and one rumble.

On disc (resource 530, 259 rows): shake 0 and rumble 0 in 238 rows; shake 0 and rumble 1 in 4 rows (139 Dark Knight; 142; 154 Thug, Thud, Korvin the Krusher; 155 Ixi Chieftan); 1 and 1 in 7 rows (162 to 168, Minion Artillery); 2 and 2 in 9 rows (3 Undead Draik, 145 Giant Drackonack, 169 to 175 Minion Brute); 3 and 2 in 1 row (181, which no spawn record places). Rows 142, 168, 175 and 181 have footstep values but no spawn record places them.

Test vectors:
- Rumble class 2 at 10 units (d2 = 100): magnitude 0.75 x (1 - 99 / 901) = 0.6676, motor level 170. At 1 unit or less: level 191. At 30 units (d2 = 900): magnitude 0.0017, level 0 (a request is made, the motor runs at 0). At 31 units: no request.
- Shake class 2 at 30 units (d2 = 900): strength 1 - 800 / 2600 = 0.692. At 51 units: no shake.

### 33. Ghosts (High for the rule and the rows; Medium for the hidden and attacking tests)

The rule applies to every region character whose monster row has the Ghost Flag (+0x138) nonzero, whatever its behaviour class. Row words (s32 ticks unless noted): solid phase min/max +0x13C/+0x140, ghost phase min/max +0x144/+0x148, opacity min/max +0x14C/+0x150 (f32, 0 to 1), reveal min/max +0x154/+0x158. Per character: solid flag S (1 at reset), phase countdown P (-1 at reset), fade counter c (-1 at reset), fade length L, reveal flag V (0 at reset) and reveal countdown R (0 at reset). Draws are closed integer draws (Behavior 2). Let lo = opacity min and hi = opacity max.

Each tick, before the character's status and AI step:

1. The character is made not targetable.
2. Revealed (V = 1): the character is targetable. If R < 1: S = 1, P = -1, c = 0, L = 0, V = 0 and the opacity is hi. Otherwise R falls by 1, c falls by 1 when above 0, and the opacity is hi - (c / L)(hi - lo).
3. Not revealed, and attacking (its agro attack sequence is running) or its sequencer state is 14: targetable; c falls by 1 when above 0; the opacity is hi - (c / L)(hi - lo); S = 1.
4. Not revealed and hidden (retreated in state 14, or emerging; SPEC-0038): if c < 1, then c = 0 and L = 60, else c falls by 1; the opacity is lo + (c / L)(hi - lo); S = 0.
5. Otherwise c falls by 1 when above 0 and P falls by 1 when above 0. Then:
   - S = 0 (ghost phase): if c < 1 and P < 0, P = draw[ghost min, ghost max]. Else if P = 0: S = 1, P = -1, L = 60, c = 60.
   - S = 1 (solid phase): if c < 1 and P < 0, P = draw[solid min, solid max]. Else if P = 0: S = 0, P = -1, L = 60, c = 60.
   - Opacity: with S = 0, lo + (c / L)(hi - lo) while c > 0, else lo. With S = 1, hi - (c / L)(hi - lo) while c > 0, else hi, and the character becomes targetable.
6. Alpha = trunc(128 x opacity). An alpha of 128 or more draws opaque; below 128 the character draws with that alpha.

So each phase change fades over 60 ticks, and a ghost is targetable only once its solid fade-in has ended, while it attacks, or while it is revealed.

Hit rule. While S = 0 and V = 0 (ghost phase, not revealed), a hit through the shared take-hit (Behavior 34) is absorbed: no damage, no hit reaction, no doubling and no pending-hit flag. Instead it sets V = 1 and R = draw[reveal min, reveal max], and the character can be hit normally while revealed.

A reveal that starts during an active ghost-phase fade keeps the existing fade counter and length. On the next tick, the reveal countdown and positive fade counter each fall by 1; opacity then uses the revealed fade-in expression, hi - (c / L)(hi - lo), while the hidden fade used lo + (c / L)(hi - lo). A reveal can therefore lower opacity sharply when it begins early in the ghost phase, even though the character is targetable. The fade-in expression raises opacity by (hi - lo) / L on each later tick while c remains positive. [High for the counter and opacity rule; rendered appearance not observed]

Not targetable means that the hero's auto-target and lock-on skip the character and the hero's shots do not steer onto it (SPEC-0036). A swing or shot that touches it still reaches its take-hit.

Rows on disc: 143 Death Knight (solid 300 to 420, ghost 60 to 180, opacity 0.1 to 0.5, reveal 60 to 180; 7 placed), 215 Minion Wizard variant (120 to 180, 120 to 180, 0.2 to 0.7, 30 to 180; 3 placed) and 222 Restless Spirit (1 to 30, 500, 0.0 to 0.5, 10 to 30; 59 placed), all class 17; and 223 Vorsh (solid 500, ghost 1 to 30, opacity 0.25 to 0.5, reveal 0 to 120; 4 placed, in packs 1429, 1430, 1431 and 1433), a class-6 townsfolk and the only ghost outside class 17.

### 34. Death, despawn and drops for every class (High)

The take-hit hook, the hit-point subtract, the death hook and the dead state belong to the region-character class, so they run for every behaviour class. SPEC-0038 Behaviors 34, 36 and 37 describe them; they are not class-17-only rules. This section states what holds for the other classes.

- Shared take-hit, any class. A hit is ignored while the character is dead (base state 16), while a ghost absorbs it (Behavior 33), or when the character's townsfolk flag is set (class 6; SPEC-0056). Otherwise the damage is doubled when the class-17 state word is not 19, and the word becomes 19. Only class 17 changes that word, so on a character of any other class only the first damaging hit through the shared take-hit is doubled. The hit subtracts HP unless the character is undamageable (+0x7F4; SetEntityUndamageable, SPEC-0030 row 125). The hook also turns the character to face the attacker (weight 1.0), posts an alert for its spawn record's group byte, adds it to the attack-manager list (SPEC-0038 Behavior 45) and sets the pending-hit flag +0xABC, which the Target Dummy consumes (Behavior 29). The hero swing reaches every hittable region character through this hook, the bosses of classes 1, 11 and 32 included; the Ixi Chief also has its own incoming-hit check (Behavior 10.3; Open question 170).
- Hit filter, any class (High): before a hero hit reaches the take-hit, the hit filter refuses the enemy when its AI mode word (+0xA34) is 16, when its script-hold word (+0xC54) is set, or when its invulnerable word (+0x7F0) is set, among other tests. The same hittable test is also false in base states 16, 33, 36, 53 and 54. The invulnerable word 0x7F0 has two setters: a hit-count threshold that is never reached in play (the threshold is -1), and a level-script command that sets and clears it. The script-hold word 0xC54 is set to 1 when the script AI hands the character back with class id 6, and is cleared when the character is created.
- Death hook, any class: base state 16 unless the row's class is 1, 11 or 32 (Behavior 2, dead test); then the drop (direct item, else placement ordinal), the disappear countdown and the carried Neopoints, and all statuses are cleared (SPEC-0038 Behavior 34). All 150 non-class-17 rows have reanimation flag 0 and death disappear timer 510 ticks (8.5 s), so no character outside class 17 reanimates.
- Dead state, any class in base state 16: the corpse step of SPEC-0038 Behavior 36 (death clip, countdown from 95 % clip progress, sink, release by the proximity streaming of SPEC-0006 once the countdown is below 0). The character's AI is not stepped while its base state is 16.
- Drops outside class 17: one record has a drop ordinal (Sir Jaceus, class 6, pack 1001, ordinal 0) and none has a direct item on disc, so in practice the only drops are the Nova Motes of the summoned minions (Behavior 31). Classes 1, 11 and 32 never enter base state 16 through the hook; their own specs own their end.
- Flyers: the two fly tests (Can Fly 1 with the ground countdown at or below 0, or Can Fly 2; Layout section 4) are region-character tests used by every class for the death clip, the agro animation and the take-hit reaction. All 42 rows with Can Fly nonzero are class 17, so the tests are always false for other classes.
- SetAI_Agro (SPEC-0030 row 115) on a living entity of any other class gives base state 1 and an animation request 18 (149 for flyers) on its own animation table; a dead entity is left alone. It also writes the class-17 state (2, or 21), a search countdown of trunc(U[row +0x60, row +0x5C)) ticks and a last-seen point, but no other class reads them. Because the class-17 state word becomes 2 or 21, the next damaging hit is doubled again. The non-class-17 rows hold search timers 0/0 (147 rows) or 60/120 (3 rows). Which classes the 53 call sites in 16 modules name is open (Open question 172).

### 35. Creature sounds and the shared hit flash (High)

**Animation sounds.** Creatures and bosses play their voices and attack sounds through their animation tables: when a character's main animation changes to an entry with a sound word, its cue plays on the creature block the word names, at the character, once its start frame is reached; nothing plays in base state 42. The rule and the block list are in SPEC-0031; each boss spec lists its own table.

- **Werelupe Basher, Elite and Bone Thrower** share creature block 350: idle cue 8 (keep-alive), patrol 7, sleep 9, ambush 6, go agro 2, claw 0, lunge bite 1, toss bone 10, dodge 3, take damage 4, die 5. Block 350 uses all its cues.
- **Undead Draik** (block 391): its table names only cue 0 of block 391 (die, on animation 0), so its attacks are silent. Cue 1 plays only as its special-victim impact cue (SPEC-0031). Cues 2 to 11 never play.

**Shared hit flash (High).** One shared routine plays a detached one-shot hit flash at a given point: effect 222 (a red flash, two models, 15 orange sparks and a red light, 0.2 to 0.4 s) when its flag is 0, or effect 220 when its flag is 1. Its users include the Gelert Assassin's flinch (SPEC-0013), the Ixi Chieftain (SPEC-0014), Juppie's head slam at its strike point (SPEC-0015), the Plague Serpent's landed bite (SPEC-0016), a damaging hit on a Faerie Sister, 1.5 units above her (SPEC-0011), and the Spyder Nest valve state 5, which is unreachable (SPEC-0017). The effect itself is in SPEC-0034.

## Edge cases and error handling

- Class request with a priority lower than the current one is ignored silently.
- Class ids 40 and up are errors.
- Navmesh pick returns -1 when no navmesh of the requested kind has a triangle that contains the character's position; the character then has no current navmesh.
- Look-ahead: when no obstacle lies inside either rectangle, the goal point is not shifted (Behavior 2a).
- Look-ahead: an inactive actor and the character itself are never taken from the level's actor list as obstacles (Behavior 2a).
- Mortog flee and zone: the zone test can enter the hold state and the flee step still runs in the same tick (Behavior 3, edge cases).
- Mortog with a zero zone radius: the zone never fires, and the wander avoidance rule is off.
- Sister alternate loops: classes 13 and 15 share loop B, and class 16 uses loop A, when level flag 0 is set. Only loop A overwrites the 3-alive movement speed word (Behavior 4).
- Sister group count of 0 fires the region event once.
- Whinny with one pen: every Whinny may use it. With several pens: one Whinny per pen.
- Draik stages 2 and 4 are never entered; their choosers are unreachable.
- Draik wind idle: the 10 to 12 idle ticks are not multiplied by 60 (the value is used as ticks). Reproduce as stated.
- Draik death event: the level event for the death repeats each tick while the death animation's last frame is held.
- Draik breakable parts: the setup stores object indices from the current nine query values and fixed bone indices. Pack 1096 sets the nine object indices during module initialization, before the Draik of that pack is set up (Behavior 6).
- Draik height keeper: a gap of exactly 0.1 is not less than 0.1, so the keeper takes a step rather than snapping (Behavior 6.9). Because of float rounding at that boundary, the step may or may not land on the target exactly; the next tick then snaps.
- Draik height keeper: in stages 1, 3 and 5 it does nothing, so in reachable play the Draik never flies; it acts only after death (stage 6), with a flying offset that no reachable path has written (Behavior 6.9).
- Ghost reveal during a ghost-phase fade (Behavior 33): for lo 0, hi 0.5, L 60 and c 45 after the last hidden update, hidden opacity is 0.375 (alpha 48). If a hit sets V=1, the next tick reduces c to 44 and opacity becomes 0.5 - (44/60) x 0.5 = 0.133333… (alpha 17), while the ghost is targetable. Later revealed ticks increase opacity by 0.5 / 60 as c falls.
- Classes 37 and 38 wait on a helper before their main part starts (Behavior 7); the wait's exit condition is not described.
- Nearest-triangle tie: the triangle that comes first in the navmesh's list wins (Behavior 2b).
- Goal latch: while the latch is set, a failed locate with a last-valid triangle of -1 returns the failed result, -1 (Behavior 2b).
- Boss encounter latch: a kind outside 0 to 2 sets the latch but raises no event; a null first hero-side query clears the latch even when previously set (Behavior 8; High).
- Projectile pool full (Behavior 9): with all ten slots live, a launch does nothing; no slot changes. A released slot is reused by the next launch when it is the lowest free index (first free slot; no compaction).
- Projectile lifetime (Behavior 9): the slot is reset only when its counter is already negative at the start of an update, so a lifetime of L gives L + 1 moves, and a lifetime of 0 still gives one move. A lifetime expiry plays no hit and posts no event.
- Ballistic lifetime unit slip (Behavior 13, reproduce it): the lifetime trunc(60 T) + 1 multiplies a flight time that is already in ticks by 60, so ballistic shots live about 60 times their flight; the ground test ends them first.
- Ballistic shots land short (Behavior 13): with gravity applied before the move, the shot is 0.5 g' T below the aim height after T ticks.
- Speed ramp outside the band (Behavior 13): u is not clamped, so a target nearer than the band minimum or farther than the band maximum gives u^2 above the band values and a speed outside [v_end, v] (reproduce it).
- Homing speed cap (Behavior 9, kind 17): after the first tick the speed is at most 1 unit per tick whatever the launch speed, because the blended direction is not rescaled (reproduce it). A homing shot launched at 3.0 per tick straight at its target moves at 1.0 per tick from the second tick on.
- Homing lifetime (Behavior 13): mode 2 doubles the straight lifetime, 2 x trunc(range / v), while its speed is capped near 1 per tick, so its reach is not its range.
- Reflected shot (Behavior 9): after a reflection the slot tests only its owner (hit selector 0); it can no longer hit the hero. A reflected kind-10 shot does no damage.
- Blocked shot damage (Behavior 9): a blocked, not reflectable hit always takes at least 1 HP, even when 0.25 D is below 1.
- Status roll (Behavior 9): the roll is an integer from 0 to 100 compared with "at most the chance", so a chance of c applies on c + 1 of 101 values, and a chance of 0 still applies on a roll of 0.
- A status on a dead enemy (class state 16 or dying): ignored. A status on an enemy of the matching element (Behavior 27.1): ignored, with no effect shown.
- Stunned enemy: a hit (including a poison or fire tick) does not play its flinch (Medium-Low).
- Forgotten kills (Behavior 2, Kill memory; SPEC-0006): an enemy killed in a world that is then released by a streaming load or by the enter-interior or leave-interior job is bound again, at full health, on the next visit. A save while that world is current or second keeps the kill.
- Dead test, second arm (Behavior 2): AI mode 16 counts as dead only while the counter at +0xD14 is above 0.
- Rumble while Vibration is off: the footstep and grab rumbles are still requested and run down unseen (SPEC-0033 Behavior 1C).
- Area mode with zero duration (Behavior 9): a kind-1 shot with an area duration of 0 ends on the tick after landing and never pulses.
- World collision near the launch (Behavior 9): no ground or world test runs until the projectile is more than 2.0 units from its start point.
- Moving-object gate (Behavior 9): an object whose squared distance is exactly 22500 is not tested (the test is strict).
- Lobbed summon in the air (Behavior 9): a kind-8 shot whose lifetime expires before it lands spawns nothing; one that loses its owner spawns its summon at once where it is.
- Occupancy test (Behavior 2b): the test does not depend on the triangle being tested, so one nearest-triangle search either keeps every strictly nearer triangle or keeps none.
- Spawn into a member group (Behavior 14): a spawn is refused when the group is full or when a member of that kind is already present.
- Petpet follow gait (Behavior 12.13): a distance exactly equal to a threshold does not satisfy a strict comparison; the strict or non-strict form of each comparison is as stated there.
- Draik missile dodge (Behavior 10.4): a roll of exactly 15 is not below 15, so the state is unchanged.
- Ballistic shot with a launch speed of 0 or a horizontal distance of 0: T = d / s cannot be formed or is 0, and vy divides by T; this spec does not define the result.
- Draik beam words: the two integer-10 words are read as damage to the hero (Open question 34).
- Petpet idle choice (Behavior 12.8): when the ratio r is exactly 1, the threshold is 0, so animation 1 is chosen on 100 of the 101 rolls (any roll from 1 to 100) and animation 6 on roll 0. As r grows above 1 the threshold grows, and from r = 1.5 on (threshold 100) animation 6 is always chosen.
- Petpet idle choice (Behavior 12.8): when r is below 1 the routine never draws a roll; animation 7 is chosen at once. The choice is remade every waiting tick, so with r at or above 1 the idle animation can switch between 1 and 6 from one tick to the next.
- Release handlers (Behavior 11): recovery draws are whole tick counts. Do not multiply them by 60 again.
- Block push routine (Behavior 10.1): never reached in the shipped program; do not implement its push or its cap of 40.
- Knockback (Behavior 10.1a): a blocked class-17 strike that still deals damage above 0 knocks the hero back; a strike of damage 0 or a killing strike does not.
- Deflect flag (Behavior 10.6): the flag set by a Black Knight's or Gelert Assassin's stance is never cleared during the fight, so every later quick shot that meets that enemy guarding comes back at the hero.
- Reach test (Behavior 10.2): in the single-weapon reach test a hero beyond 65 units (full 3D distance) is never a hit, whatever the shapes would give. The weapon-volume test has no such gate.
- Ixi Chief incoming hits (Behavior 10.3): while the boss is in its get-hit state 0x12, melee contacts are not tested, so a swing that is still active after the first contact adds no second melee hit through this check.
- Gelert rainbow dash (Behavior 12.5; SPEC-0013, 4.8): the permutation is drawn once per dash, so the eight points are each visited exactly once per dash.
- Class 6 value 3 (villager pair): if there is no partner, or the partner is not in value 3, or the two are on different nav meshes, the townsperson returns to its code with a cooldown of 10 to 30 seconds (High). On disc no pair ever forms, because the pair radius (row +0x44) is 0.0 on every class-6 row.
- Class 6 value 13: a hero more than 12 units away ends the state even when the hero is in front of the townsperson (High).

- **Volley hit count (Behavior 18)**: the hit count doubles as the hover sub-state. A gap of 3 seconds (180 ticks) with no hit lowers the count by one, so three hits must land without such a gap to reach state 9. Two hits, then a 3-second gap, leave the count at 1.
- **Volley reset (Behavior 18)**: the event-15 reset does not clear the hit count or the volley timer (Open question 119). A new volley start sets both again.
- **Grab-and-throw states 3 and 4 (Behavior 23)**: nothing in the routine leaves them; another routine must set the state word back (Open question 121).
- **Grab-and-throw sub-attacks (Behavior 23)**: with a pending-action test that gives the same answer on every call, only sub-attack 0 and sub-attack 1 can be chosen.
- **Living count of 0, -1 or above 3 (Behavior 20)**: both chance tables give 0, so no melee-dodge roll or missile-dodge roll ever passes.
- **Exactly 6.0**: the distance test is strict, so a value of 6.0 does not pass the gate.
- **Facing product of zero**: the gate needs a positive product, so zero fails.
- **Unnamed enemy (Layout section 4c)**: a spawn record with an empty name shows no name panel when it is hit; the health values still update.
- **Summon with no reserved record (Behavior 31)**: a summoning attack whose block holds no unbound reserved record with the key spawns nothing. The same holds when the block has no free character object or is at its cap.
- **Zero-position records (Behavior 31)**: a reserved record with an all-zero position is never placed at the origin; it appears only at a summoner's point.
- **Run-time variant re-pick** (reproduce it): a kind-6 spell projectile (the Minion Wizard, row 213, map search type 8; SPEC-0038 Behavior 43) acts on its target, the caster's current target id copied into the slot at launch (the map search picks a nearby injured character). On contact it draws a random integer p from 1 to 6 (both ends included), sets the target's variant to p - 1 (so 0 to 5) and selects its monster row again (the row-530 pointer, offset 0xA48) as first[type] + variant, with the same type-only bound as the spawn lookup (Layout section 4). A variant past the type's row count therefore selects a row of the next type (type 0 with variant 1 gives row 1). The step requests no class and does not change HP; only a later spawn-default request (for example on character activation) reads the new row's class. It then loads colour preset p into the target's colour block and pushes it to both colour buffers (buffer argument -1); the presets, the push and the effect on colour word 0 are in SPEC-0040 Behavior 26. Confidence: High for the step and the preset push, for the kind-6 source (sub-type 6 of the common-enemy spells builds kind 6) and for the target (the projectile's target, not its owner).
- **Group alert board under pause (Behavior 1)**: the board keeps counting down while the game is paused, so an entry posted just before a pause can expire before the world runs again. Reproduce this.
- **Projectile pool under pause (Behavior 9)**: the pool is part of the world branch and does not step while paused.
- **Ghost fade before the first phase change (Behavior 33)**: in the attacking branch before the first phase change, c = -1 and L = 0, so the fade fraction c / L is formed with L = 0. The result is a huge opacity that draws opaque; reproduce it as opaque.
- **Ghost fade after a reveal ends (Behavior 33)**: when a reveal ends the fade counter c and the fade length L are both set to 0, and the tick that ends the reveal draws opacity hi. If the ghost is attacking (or in sequencer state 14) on a later tick, the attacking branch forms c / L as 0 / 0. The console's divide returns the largest positive finite number for 0 / 0 (no NaN), so opacity = hi - (largest float) x (hi - lo), a huge negative value whenever hi is above lo (true for all four ghost rows); the alpha conversion saturates to the most negative integer and the draw clamps alpha to 0. The ghost is therefore fully transparent, yet targetable and attacking, on every attacking tick after a reveal until the next phase change (which sets L to 60 again); on ticks when it is not attacking the solid-phase rule gives opacity hi. It is not drawn opaque. With hi equal to lo the product is 0 and opacity stays hi. Medium-High (the divide rule is a hardware behaviour that the code relies on).
- **Translucent solid ghosts (Behavior 33)**: a ghost's maximum opacity below 1.0 means it stays translucent even in its solid phase (0.5 for the Restless Spirit, alpha 64).
- **Absorbed ghost hit (Behavior 33)**: a hit in the ghost phase deals no damage and starts no reaction; it only reveals the ghost. The ghost is not targetable then, but a swing or shot that touches it still counts as a hit.
- **First-hit doubling outside class 17 (Behavior 34; reproduce it)**: the first damaging hit through the shared take-hit on a character of any class other than 17 is doubled; later hits are not, until SetAI_Agro writes the class-17 state word again.
- **Mote eater with seven Motes (Behavior 9)**: the repeat probing can reach index 12, which reads the first drawn index (0 to 11) instead of a Mote id; the group add refuses it, so the group holds seven Motes instead of eight in 27.03 % of groups (116,239,111 / 429,981,696).
- **Draik killing hit (Behavior 6.10)**: the death hook runs, but the Draik sets its base state back to 1, so the shared corpse step never removes it.
- **Breakable spawn with no reserved record (Behavior 31)**: a break whose key has no unbound reserved record in the world's blocks spawns nothing.

## Test vectors

Synthetic, hand-checkable:

- Class priority: current class 12 at priority 5. Request class 7 at priority 5 -> replaced. Request class 7 at priority 4 -> ignored. Request class 12 again -> nothing happens.
- Mortog: squared distance to the hero 24.9 -> flee (below 25). Exactly 25.0 -> not below, stays idle.
- Mortog flee end: squared distance to the hero 224 -> still fleeing. 225 -> idle.
- Mortog run rule: squared distance to the hero 112 -> run (mode 2). 113 -> walk (mode 0).
- Mortog zone: with L = 4, Mortog at squared distance 3.9 from A -> hold (timer round(60 x a draw of 10 to 20)). With L = 0 nothing happens.
- Mortog zone setter (synthetic): locator i at (0, 0, 0) and locator j at (3, 10, 4) give L = 9 + 16 = 25; the height difference of 10 is ignored.
- Mortog hold end: a draw of 6 gives a move point 6 units from Mortog directly away from A; the zone count decreases by 1.
- Mortog wander: squared distance to the wander point 3.9 -> back to idle. 4.0 -> still walking.
- Mortog wander zone: L = 4, look-ahead point at squared distance 7.9 from A (below 2L = 8) -> the wander point is Mortog plus 12 units along the unit vector from A to Mortog.
- Whinny: hero at 4.9 units -> flee. At 5.0 -> stays in wander.
- Whinny flee end: hero at 8.1 units -> back to wander. Hero at 8.0 -> flee continues.
- Whinny group: three members, two speaking -> sound 109. Three members, one penned and one speaking -> sound 109 (the penned one counts as speaking).
- Whinny pen with count 1: a Whinny inside the inner area becomes penned.
- Whinny flee with a clear path to the 4-unit away point: a clear navmesh-blocked latch selects the direct-away step; a set latch selects the safe-point step. If the latch becomes set during the direct-away step, the target is abandoned, the sub-step resets and the latch is cleared.
- Whinny flee path test (resource 909, current mesh 1): Whinny at (-1845.17, 1141.79), hero at (-1846.28, 1145.63): candidate (-1844.06, 1137.95) is on mesh 1, latch clear -> sub-step A. Whinny at (-1844.59, 1139.77), hero at (-1845.70, 1143.61): candidate (-1843.48, 1135.93) is on no triangle of mesh 1 -> sub-step B, even with the latch clear. Same first case with the latch set -> sub-step B.
- Whinny safe point (mesh 1, hero at (-1845.70, 1143.61)): draw 2 (centroid (-1853.71, 1143.73), distance 8.0) is rejected; draw 1 (centroid (-1865.53, 1151.33), distance 21.0) is accepted. With all 9 draws rejected the last draw is the result.
- Whinny entrance test (resource 909): position (-1849.04, 1135.42) is inside mesh 5 -> lead for the pen (mesh 5 current, target (-1840.40, 1125.23), timer 30). Position (-1844.88, 1139.33) is on mesh 1 but outside mesh 5 -> stays in flee.
- Whinny speed: running (animation 3) moves 0.225198 units per tick (13.512 per second); walking (animation 2) moves 0.069367 per tick (4.162 per second). A Whinny running in a straight line for 60 ticks covers 13.51 units; sub-step A toward a point 4.0 away ends after 7 ticks (1.576 units, 2.42 left), and sub-step A is capped at 15 ticks (3.38 units); sub-step B is capped at 25 ticks (5.63 units).
- Whinny herding (resource 909, hero 4.0 behind the Whinny on the line to the pen, Whinny already running): start (-1845.754, 1143.807), hero (-1846.86, 1147.65): sub-step A at ticks 0, 8 and 16, enters lead at tick 23 at about (-1844.32, 1138.83). Start (-1853.762, 1143.985): sub-step A at ticks 0, 8, 16, 24 and 32, sub-step B at tick 40, enters lead at tick 43 at about (-1848.34, 1136.01). Start (-1865.531, 1135.022): enters lead after about 130 ticks (up to 270). Ticks are accurate to a few.
- Whinny edge wall (mesh 1, radius 1.175): the south-east edges have reach 1.175 and the north-west edges 0. A Whinny pushed by the south-east edge ends 1.175 from it (plus up to 0.225 when its step pointed into the edge); one pushed by a north-west edge ends on the edge line.
- Draik cooldown: breath at stage 1, a draw of 4 to 8 gives 240 to 480 ticks inclusive.
- Draik hit with max HP 1000: 25 HP per hit. After a stage-1 hit reaction with HP 660 or less: transition 21 (stage 3).
- Draik stage 1 choice: d = 7 with spit ready and claw ready: spit (the spit band is tested before the claw band).
- Draik stage 5 choice: d = 6 with stomp and bite both ready: bite.
- Draik turn: facing cosine 0.5 with d = 3 starts facing. While facing, cosine 0.9 keeps facing; cosine 0.96 ends it.
- Draik height keeper (synthetic): ground height 3.0 and flying offset 2.0 give a target of 5.0. Current height 4.0: the gap is 1.0 and the target is above, so the height becomes 4.1 this tick. Current height 4.95: the gap is 0.05 (less than 0.1), so the height snaps to 5.0. Current height 5.3: the gap is 0.3 and the target is below, so the height becomes 5.2. In stage 3: nothing changes.
- Draik height approach (synthetic): from height 4.0 toward a fixed target of 5.0, the height rises by 0.1 per tick while the gap is 0.1 or more. The gap reaches the snap range after about 10 ticks, so the height is at 5.0 after about 10 to 11 ticks, the exact tick depending on float rounding at the 0.1 boundary.
- Navmesh pick: three navmeshes in the world list; navmeshes 0 and 2 have kind 1, navmesh 1 has kind 2. The character's position lies inside a triangle of navmesh 2 only. Mode 0 returns 2. Mode 1 returns 1 only if the position lies inside a triangle of navmesh 1, else -1.
- Sister chooser: r = 0.33 exactly -> not below 0.33, so the chooser proceeds to the cooldown checks. r = 0.25 with the third counter at 0 -> state unchanged.
- Sister animation rate at 25 % HP: P = 2 − 0.25 = 1.75 for that tick's clip re-time; any earlier status multiplier in P is replaced by this direct store.
- Sister energy-ball launch speed with 2 alive: 0.75. With 1 alive: 0.9. With 3 alive: 0.5.
- Look-ahead (synthetic, Behavior 2a): no obstacle inside either 20 x 5 rectangle -> the goal point is unchanged. One obstacle inside a rectangle -> the goal point moves 5.0 units sideways.
- Nearest tie (synthetic): candidate A (earlier in the list) and candidate B both at distance 6.0 -> A. If B is at 5.9 -> B.
- Goal latch (synthetic): latch clear, id 7, the look-ahead goal (10, 0, 10) fails to locate. Then j = 7 x 0.2 - 2.0 = -0.6 on z, the search point is (10, 0, 9.4), the goal becomes the centroid of the nearest triangle, and the latch stays clear. Id 30 gives j = 0.0 on x; id 19 gives j = 1.8 on z. Latch set, the goal fails to locate and the last-valid triangle is 3: triangle 3 is returned and the goal becomes the last-valid point.
- Encounter latch (synthetic): progress flag 220 clear, latch clear, both hero-side tests return values, third test passes, kind byte 1 -> latch set, encounter event of kind 3. Kind byte 4 -> latch set, no event. First hero-side test returns nothing -> latch cleared, no event.
- Ballistic lifetime (Behavior 13): horizontal distance 10.0 and launch speed 4.0 give a flight time T = 2.5 ticks and a lifetime of trunc(150.0) + 1 = 151 ticks (the unit slip). Distance 1.0 and speed 7.0: T = 0.142857 ticks and a lifetime of trunc(8.5714) + 1 = 9 ticks.
- Darkest Faerie descent (Behavior 18): with state 8 and boss 0.04 above the probe height, the next tick posts events 13 and 10 and returns to state 1; with state 9 and boss 0.15 above ground, it lowers by 0.1 and remains in state 9, then at 0.08 above ground it runs the height setter, posts event 16 and enters state 10. In state 10, event 13 clear keeps animation 20; event 13 set posts event 10 and returns to state 1.
- Projectile pool: slots 0 to 3 live and slot 4 free: the next spawn takes slot 4. With all ten slots live, the spawn is refused.
- Projectile placement (synthetic, Behavior 9): slots 0 and 2 live, slot 1 released, slots 3 to 9 free: the next launch takes slot 1 (the first free slot), and the one after it takes slot 3.
- Entity group update (synthetic): five groups with counts 2, 0, 1, 0 and 3 (in the listed order). Each entry's method is called once, the group with a zero count adds nothing, and the results are summed.
- Reflect permission (Behavior 9, synthetic; kind 0, visual kind 23, blocked, word 1 holds Mote 161 (element kind 1)): a shot of element 1 is reflected; a shot of element 0 is not; a shot of element 4 is not (4 is neither 1 nor 7). With Mote 173 (element kind 7) in word 1, a shot of element 4 is reflected.
- Capsule contact (Behavior 9, synthetic): Rb 1.0, Hb 4.0, r 0.5, base (0, 0, 0). Hits: (1.4, 2, 0), (1.4, 0.1, 0), (1.4, 3.9, 0), (0, 4.49, 0), (1.2, 4.3, 0). Misses: (1.6, 2, 0), (0, 4.51, 0), (1.0, 4.6, 0), (0.5, 5.4, 0). Below the base on the axis the results alternate (hits at y = -0.4, -0.8 and -1.4; misses at -1.0 and -1.6). That end-cap case is degenerate (Low; do not rely on it).
- Straight motion and lifetime (Behavior 9): kind 0, position (0, 1, 0), velocity (0, 0, 2) per tick, lifetime 2. After updates 1, 2 and 3 the positions are (0, 1, 2), (0, 1, 4) and (0, 1, 6), and the counter reads 1, 0 and -1. At the start of update 4 the counter is negative and the slot is reset, with no hit and no event.
- Lifetime 0 (Behavior 9): the slot moves once on its first update (the counter becomes -1) and is reset at the start of its second update.
- Pool effect ids (disc; Behavior 9): Juppie's spit hitting an unguarded hero plays id 5 = effect 1399 (it resolves to definition 392 of resource 9); the same spit expiring plays id 3 = -1, nothing. A Faerie Sisters energy ball blocked by the hero plays id 2 = -1, nothing. A kind-1 projectile that lands plays id 4 at the ground point, and its cue 190 plays nothing.
- Gravity motion (Behavior 9): kind 1, position (0, 1, 0), velocity (0, 0.2, 1) per tick. After ticks 1 to 4, y = 1.19111, 1.37333, 1.54667, 1.71111 and vy = 0.191111, 0.182222, 0.173333, 0.164444. After tick 45, y = 0.8 and vy = -0.2 (closed form y_n = y_0 + n vy_0 - g' n(n + 1)/2 with g' = 32/3600).
- Moving-object gate (synthetic, Behavior 9): a moving object at squared distance 22499 from the projectile is tested; at 22500 it is not.
- Nearest triangle (synthetic, Behavior 2b): query point (0, 0, 0); triangle 0 has centroid (6, 0, 0), triangle 1 centroid (3, 0, 4) and triangle 2 centroid (0, 0, 5), so the distances are 6.0, 5.0 and 5.0. No other enemy is on the navmesh. Result: triangle 1 (the tie with triangle 2 goes to the earlier one); the current and last-valid triangles become 1.
- Nearest triangle with vertical separation (synthetic, Behavior 2b): query point (0, 0, 0); triangle 0 centroid (0, 3, 0), triangle 1 centroid (2, 0, 0). Their full distances are 3.0 and 2.0, so triangle 1 wins; a horizontal-only metric would select triangle 0.
- Occupancy quirk (synthetic, Behavior 2b): the same triangles, and one other live enemy on the same navmesh at enemy-list position 2 whose last-valid triangle is 2. The test is true for every triangle, so nothing is kept and the character's triangle indices do not change. With that enemy's last-valid triangle 1 instead, the test is false for every triangle and the result is triangle 1.
- Petpet follow gait (synthetic, Behavior 12.13): gait 0 with distance 4.0 gives animation 2 and gait 1. Gait 1 with distance 8.0 gives animation 3 and gait 2. Gait 1 with distance 1.2 gives animation 0 and gait 0. Gait 2 with distance 4.9 gives animation 2 and gait 1; gait 2 with distance 5.0 gives animation 3 and stays in gait 2. Gait 1 with distance exactly 7.0 gives animation 2 and stays in gait 1.
- Spawn into a member group (synthetic, Behavior 14): capacity 3, count 2, kinds present 5 and 7. A spawn of kind 5 is refused (kind present). A spawn of kind 9 is accepted and the count becomes 3. A further spawn of kind 4 is refused (full).
- World-zone group pass (synthetic, Behavior 15): group callbacks return -2 and 3, so the pass returns 1; the submission wrapper discards that result. With the per-world reference-list count reset to 0, 15 zone-instance registrations append 15 entries; a 16th registration leaves the count at 15.
- Sensor hit reaction (synthetic, Behavior 10.4): a sensor reports a hit and the roll is 14: the state becomes 0x13. The same hit with roll 15: the state is unchanged. No sensor hit: the state is unchanged whatever the roll.
- Draik breath range: a fire breath draw of 2.5 s gives 150 ticks; the range is 2.0 to 5.0 s (120 to 300 ticks).
- Draik beam length (Behavior 6.5): rate 1.0, target 3.4, length 0 at the start: lengths 1.0, 2.0, 3.0, 3.4 on ticks 1 to 4. If the target then drops to 1.2, the length is 1.2 on the next tick.
- Locomotion slots (Behavior 2): slot setter values (28, 28, 28, 32, 55, 29, 30): mode 3 plays 32, mode 4 plays 55, mode 5 plays 30 and mode 6 plays 29.
- Release recovery (Behavior 11): a draw of 30 ticks is 0.5 s and 45 ticks is 0.75 s (routines C and E). A draw of 120 ticks is 2 s and 240 ticks is 4 s (routine D).
- Leap (Behavior 11, routine H): the launch speed is the word 0x2DAAB8 (10.0; its unit is not confirmed). If the speed were in units per second, a leap of 0.5 s would cover 5 units (synthetic; the leap's own timer and end condition are not described).
- Steal (Behavior 11a, synthetic): a slot with minimum 10, maximum 100 and chance 50. A chance draw of 30 is below 50, so the steal succeeds. An amount draw of 47 takes 47 from a wallet of 500, which becomes 453. The coins are 25, 10, 10, 1, 1, 1 and 5 (53 Neopoints in total) at the hero's position. The carried Neopoints do not change. A chance draw of 50 is not below 50, so the wallet and the coins are unchanged.
- Steal impact gate (synthetic, Behavior 11a): normal strike result 0 and special marker clear -> return before steal chance; positive normal strike result -> continue to steal eligibility/chance. For slot flag 0x08, state 3 or a nonzero shared gate query leaves the special marker clear; state 4 with a zero query sets it and allows processing to continue.
- Steal, whole-number amounts (Behavior 11a; coins are listed by value; the amount is taken from a wallet of 500 unless stated):
  - Amount 10: one coin of 10 (total 10, wallet 490).
  - Amount 7: coins 5, 1, 1, 1, 5 (total 13, wallet 493).
  - Amount 11: coins 10, 1, 1, 1, 1, 5 (total 19, wallet 489).
  - Amount 1: coins 1, 1, 1, 1, 5 (total 9, wallet 499).
  - Amount 4: coins 1, 5 (total 6, wallet 496).
  - Amounts 5, 10, 25, 30, 100, 150 and 200: exact. Amount 5 is one coin of 5; 25 is one coin of 25; 30 is 25 and 5; 100 is one coin of 100; 150 is 100, 25 and 25; 200 is 100 and 100.
  - Amount 99: coins 25, 25, 25, 10, 10, 1, 5 (total 101, overpay 2).
  - Wallet 3 and drawn amount 50: taken 3, wallet 0, coins 1, 1, 5 (total 7). The steal is limited by the wallet, not by the draw.
  - Wallet 0 and drawn amount 10: taken 0, nothing spawns, wallet stays 0.
  - Over all amounts from 1 to 200, the coin total is exact for multiples of 5 and overpays by 2 to 8 Neopoints for other amounts.
- Steal success (Behavior 11a): chance 50 succeeds for 50 of the 101 chance draws (about 49.5 percent). Chance 100 succeeds for 100 of 101 (about 99.0 percent). A chance draw of 50 against chance 50 fails.
- Steal range (Behavior 11a): minimum 50 and maximum 50 make no draw and take 50. Minimum 150 and maximum 50 are swapped, so the draw is from 50 to 150, both included.
- Petpet wait (Behavior 12.8): on the first arrived tick the unset timer draws a value in [180, 600]; a draw of 180 is 3 s. With the hero at (0, 0, 0), a new point lies in x and z between 3.0 and 10.0 or between -10.0 and -3.0 on each axis, for example (5.2, y, -7.9).
- Petpet idle choice (Behavior 12.8): remaining time 900 ticks: r = 0.5, animation 7 with no roll. 1800 ticks: r = 1.0, animation 1 on rolls 1 to 100 and animation 6 on roll 0. 2160 ticks: r = 1.2, threshold about 40, so roll 39 gives animation 6 and roll 41 gives animation 1. 2700 ticks: r = 1.5, threshold 100, animation 6 on every roll.
- Knockback (Behavior 10.1a, synthetic): K = 10, attacker facing (0, 0, 1): unit(0, 12, 9) = (0, 0.8, 0.6), so the hero's velocity becomes (0, 8, 6), the grounded flag is cleared and the hero enters state 48. K = 5, facing (1, 0, 0): velocity (3, 4, 0). Damage 0, or a hit that leaves the hero at 0 HP: no knockback.
- Reach (Behavior 10.2): in the single-weapon reach test a hero at 64 units can be hit when the shapes overlap; a hero at 66 units never is.
- Trail entry (Behavior 13): shot counter 3 gives effect-table entry 9; counter 0 gives entry 1; counter 4 gives entry 10; counter 5 gives entry 1. A flight-mode-1 shot always uses entry 1.
- Flower Monster seed (row 149, attack slot 0; Behaviors 9 and 13). Flight mode 1 (kind 1): v 0.5, v_end 0.1, band 3 to 60, radius 1.0, status 3 (slow) at chance 25, area duration 0. Launch (0, 1, 0); aim at the hero position (20, 0, 0) with no scatter; d = 20, dy = -1. u = 1 - (20 - 3)/(60 - 3) = 0.70175; s = 0.5 + u^2 (0.1 - 0.5) = 0.30302; T = 66.003 ticks; vy = 0.27820; lifetime trunc(60 T) + 1 = 3961 ticks. Positions (x, y): tick 1 (0.303, 1.2693); tick 2 (0.606, 1.5297); tick 33 (9.9995, 5.1938). With the hero elsewhere and flat ground at height 0, the ground test (y - 1.0 < 0) fires on tick 62 at x = 18.787: area mode starts (effect id 4 at the landing point; cue 190 goes to the hero weapon slot and plays nothing), and the zero area duration resets the slot on the next tick.
- Minion Archer (row 190, attack slot 3; met in Ellis Family Farm), launch to hit (Behaviors 9 and 13). Row values: missile slot, flight mode 0, launch speed 2.0, range 70, radius 0.05, damage 10, spawn count 2, element 1, status 7 (fire) at chance 100 with value trunc(U[600, 1800] / 60) = 10 to 30, visual kind 23. Launch point (0, 2, 0); hero at (0, 0, 20); the archer inside the camera cone and its inaccuracy 0, so no scatter. Aim (0, 2.5, 20); unit direction (0, 0.02499, 0.99969); velocity (0, 0.04998, 1.99938) per tick; lifetime trunc(70 / 2) = 35. Per-tick z: 1.999, 3.999, ..., 17.994 after tick 9 and 19.994 after tick 10. With a synthetic hero capsule Rb = 0.5, Hb = 3, the first contact is on tick 10. Damage 10 x 1.0 = 10: with no shield or armour the hero loses 10 HP, fire (status 7) is applied for the drawn value, and the slot is released. One attack fires two arrows, one per spawn event.
- Minion Archer expiry (Behavior 9): the same arrow with nothing in its way makes 36 moves (z = 71.977) and is reset at the start of update 37, with no event.
- Homing (Behavior 9, kind 17; Minion Wizard row 213, attack slot 1; lifetime 2 x trunc(100 / 1.0) = 200). Start (0, 2, 0), velocity (0, 0, 1); hero at (10, 0, 0), so the homing target point is (10, 2, 0). Velocity after ticks 1 to 5: (0.25, 0, 0.75), (0.4864, 0, 0.6923), (0.6782, 0, 0.5752), (0.8153, 0, 0.4279), (0.9026, 0, 0.2735); speeds 0.791, 0.846, 0.889, 0.921, 0.943. A launch velocity of (3, 0, 0) aimed straight at the target becomes (1, 0, 0) after one tick.
- Spell lifetime (Behavior 13; SPEC-0038 Behavior 43): a launch-to-aim distance of 12.7 gives a lifetime of 12 + 5 = 17 ticks at 1.0 unit per tick, so the spell makes 18 moves.
- Status roll (Behavior 9): chance 25 applies the status for rolls 0 to 25, that is 26 of the 101 equally likely values.
- Class 6 value 3 (synthetic): the townsperson is in value 3 with no partner found. It goes to value 0 with a cooldown between 600 and 1800 ticks (10 to 30 seconds), and the next tick back to its code.
- Class 6 value 13 (synthetic): a hero 12.5 units away returns the townsperson to value 0. A hero 12.0 units away and in front keeps value 13. A hero 8.0 units away and behind the townsperson (the behind test: dot(unit horizontal direction to the hero, facing) below -0.1) returns it to value 0.
- Class 6 default (disc rule): a placement whose row +0x36 is 0xFFFF and whose companion row +0x10 is 6 runs class 6 and has its townsfolk flag set; a placement whose row +0x36 is 17 runs class 17 with the flag clear.
- Serpent phase check (synthetic): current 30 of maximum 100 (ratio 0.3) wants phase 3; 25 of 100 wants phase 4; 80 of 100 runs no check. With phase 3 at 25 of 100, the serpent sets state 11 and plays its animation 5 when it is in state 0 or 4. The same health in state 1, 5, 6, 7 or 11 starts nothing on that tick.
- Approach gap (synthetic): a distance of 0.04 to the target is inside the gap, so the approach routine sends 13 and 10 and sets state 1 without moving. A distance of 0.06 is outside the gap, and the object moves.
- Damped homing (synthetic): a gap under 1.5 starts the 3-second timer, which is 180 ticks.

- Volley timer (synthetic, Behavior 18): the start sets 50.0 seconds times 60 = 3000 ticks; after the first tick the timer reads 2999.
- Hover offset (synthetic, Behavior 18): after one hit the sub-state is 1, so the offset is 10.0. After another 180 ticks with no hit the offset returns to 15.0.
- Melee-dodge chance (synthetic, Behavior 20): two living sisters, horizontal distance 6.5, facing test passes: a draw of 49 gives state 3 (49 is below 50); a draw of 50 does not. Distance 7.0 is not below 7.0, so the roll is not made.
- Missile-dodge chance (synthetic, Behavior 20): one living sister, a hero projectile passes the proximity test, and a draw of 49 (chance 50) gives state 2; a draw of 50 does not.
- Contact sphere (synthetic, Behavior 21): a target point 4.9 units from the enemy's point is inside the 5.0 sphere and removed; 5.1 is outside.
- Contact capsule shape (Behavior 21): radius 6.0 and segment length 10.0 means a 5.0-unit half-segment on the supplied axis, with radius 6.0 around it; it is not equivalent to a radius-16 sphere.
- Homing detonation (synthetic, Behavior 19): a controller slot 1.2 units from its target with no blocking test detonates; at 2.5 units it moves 0.75 units along the direction.
1. State 13 gate (synthetic): an eligible townsfolk in a mode other than 13, with the hero 5.9 units away horizontally and in its front half, enters mode 13 regardless of vertical separation. A horizontal distance of 6.0 does not enter. A hero directly to the side (horizontal facing dot 0) does not enter.
2. State-13 flag gate (synthetic): with mode 13, +0xAA4 = +0xAA8 = +0xAAC = 0, and distance/facing still inside the mode, +0xAB0 = 0 advances the counter to 1 and requests its selected animation; +0xAB0 = 1 leaves the counter at 0 and skips that branch. If distance exceeds twice the entry threshold, either value returns to mode 0.
2. Faerie Sisters kind selector (synthetic): in state 5, kind values 0, 1 and 2 select poison, curse and slow; in state 6, they select thorn, leach and Mote-eater. The associated effect ids are 462/463/461 and 466/465/464, respectively.
2. Oriented entry (synthetic): for a flagged entry the first word is 1; +0x04 is 0; +0x08 is 40; +0x0C is 0 in the two known callers. For an unflagged entry the first word is 0, with the same other initialized values.
3. Spawn by key (synthetic, Behavior 31): a block holds records A (key 50, summon byte 1, bound), B (key 50, summon byte 0), C (key 50, summon byte 1, unbound) and D (key 60, summon byte 1, unbound), with a free object and the active count below the cap. A request for key 50 at point P binds C at P. A second request for key 50 fails (A is bound, B is not reserved). A request for key 70 fails.
4. Display name (synthetic, Layout section 4c): two records of the same row named "Thief" and "Drub" show "Thief" and "Drub" when hit; a record of that row with an empty name shows no name panel.
5. Weapon windows (synthetic, Behavior 10.2): an enemy whose current frame has table B value 0x6 tests weapons 2 and 3 only; value 0x40 tests none and is not attack-active; value 0x11 tests weapons 1 and 5.
6. Weapon size (disc, Behavior 10.2): row 9 weapon 1 is radius 3.0, full height 3.0; row 9 weapon 4 is radius 1.5, full height 3.0; row 11 weapon 1 is radius 5.0, full height 5.0.
7. Enemy single-weapon reach (synthetic, Behavior 10.2): L = 1.5, attachment-0 anchor (0, 1, 0), axis (0, 0, 1): the cylinder is centred at (0, 1, 0.25), radius 1.0, and spans z = -0.5 to 1.0. Characters 65.0 apart: tested; 65.1 apart: no hit whatever the shapes.
8. Block test (synthetic, class-17 strike flags of Behavior 10.2): target guarding, facings dot -0.5: blocked (flag 1); dot -0.4: not blocked. Strike flag 0x01: never blocked; flag 0x20: the hit counts as blocked.
9. Integer draw (Behavior 2): a draw from 1 to 6 gives only 1 to 6, both ends possible, never below 1 or above 6.
10. Boss melee is unreduced (synthetic, Behavior 10.5): a hero with 100 HP, body armour stat 50 and an armour Mote is struck by a boss melee hit of 25: HP 75. The same hero struck by a pool projectile of damage 10 with 50 % armour and no Mote: trunc(10 x 0.5) = 5, HP 95.
11. Blocked flag (synthetic, Behavior 10.5): a take-hit of 15 with the blocked flag set on a hero with 100 HP leaves HP 85; the hero plays the block reaction (animation 0x40), not the hit reaction.

Alert board and pause (synthetic): an entry posted with a life of 100 on pass 0 is the only entry. The player pauses on pass 10 and resumes on pass 300. The entry counts down on every pass, paused or not, reaches 1 on pass 99 and is removed then, so it is gone when the world runs again although the world ran only 10 passes with it.

- Ghost timeline (Behavior 33; row 222, synthetic draws): after reset, tick 1 draws a solid length of 20: opacity 0.5 (alpha 64), targetable. Tick 21: P reaches 0 and the ghost phase starts with c = 60: opacity 0.5, not targetable, hits absorbed. Ticks 22 to 81: opacity (c / 60) x 0.5, reaching 0.0 (alpha 0) on tick 81, where the ghost length 500 is drawn. Tick 581: the solid phase starts with c = 60, opacity 0.0. Tick 641: opacity 0.5, targetable again, and a new solid length is drawn. A hit on tick 300 with a reveal draw of 15: no damage; ticks 301 to 315 are revealed at opacity 0.5 and damageable; tick 316 ends the reveal (solid), and tick 317 draws a solid length.
- Ghost after a reveal (Behavior 33; row 222, lo 0, hi 0.5): the reveal ends on tick t (R below 1): opacity 0.5, alpha 64, c = 0, L = 0, S = 1. If the ghost is attacking on tick t + 1: c / L is 0 / 0, opacity is 0.5 - (largest float) x 0.5, alpha 0 (invisible), still targetable. If it is not attacking on tick t + 1 and not hidden: opacity 0.5, alpha 64.
- Summon start (Behavior 31): a spawn by key 2844 plays animation 82 and takes state 21. A spawn by key 619 (Flower Monster, lobbed) takes state 19 with no animation 82.
- Member summon (Behavior 7, synthetic): hero at (0, 0, 0), boss at (10, 0, 0): the Minion Warrior spawns at (4, 0, 0) facing -x, and it drops a Nova Mote when it dies.
- Leach (Behavior 9, kind 15): hero magic 80, a leach hit leaves 30. Magic 30: 0.
- Mote eater (Behavior 9, kind 14): picks 3, 3, 5 give Motes 163, 164 and 165. Picks 3, 3, 5, 0, 1, 2, 4, 6 give 163, 164, 165, 160, 161, 162, 166, 167 (8 entries).
- Mote eater, index 12: picks 11, 11, 11, 0, 1, 2, 3, 4 give 171, then index 12 (refused), then 161, 160, 162, 163, 164, 165: 7 entries.
- Mote eater timing: with 8 entries the drains fall on group steps 1, 226, 451, 676, 901, 1126, 1351 and 1576; with 7 entries on steps 2, 259, 516, 773, 1030, 1287 and 1544. The group ends on step 1801.
- Mote eater drain: a Fire Mote with charge 15 is removed by one drain and unequipped (cue 0xA4 if it was equipped); a charge of 100 becomes 80.
- Shared death (Behavior 34): a Mortog (row 249, HP 20) whose first hit is 10 takes 20 and dies; it plays animation 58 and is released 510 ticks after its clip reaches 95 %. A miamouse (row 248, HP 5) dies to a first hit of 3 (doubled to 6). A Target Dummy (HP 50) that is not undamageable dies the same way to a first hit of 25 or more.
- Animation step (Behavior 2c): an object in a state whose routine uses animation 4, with animation 2 current. Tick 1: animation 4 starts, nothing else happens. Ticks while animation 4 plays: nothing. First tick after animation 4 reaches 0.99 of its length: the action runs (for the Plague Serpent with its flag clear, state 6).
- Stun count (Behavior 2c, message 13): counter 0. First finish of animation 13: counter 1, code 0. Second: counter 2, code 0. Third: counter 3, level states 12 and 10 set, done flag set, code stays 6. Next tick, still held: counter 4, both states set again.
- Draik stage-3 hit (Behavior 2c, message 27): state 27, animation 9 current, latch clear, hero melee active, a body group passes: state 3, latch set. Next swing tick with the latch set and no projectile: nothing. A tick with no melee and no projectile: latch cleared. With the latch set and a projectile active and a group passing: state 3 again.
- Serpent phase (Behavior 2d): HP 70 of 100 with phase 1: wanted 2, so state 11. HP 70 with phase 2: nothing. HP 80 with phase 4: nothing (above 0.75 there is no check). HP 40 in state 5: nothing (the check does not run in state 5).
- Loop B chooser (Behavior 7): hero at 14.9 units, no type-1 slot near her: state 4. Hero at 20 units, state 0, cooldown 0, no type-0 slot, level state 2 set: state 3 (the summon test fails).
- Loop A knockback (Behavior 7): radii 1.0 and 0.5, hero 1.2 units away along +x. g = 1.2 - 1.6 = -0.4, so the hero is moved by +2.0 along x; the velocity (9, 12, 0) is scaled to length 15, giving (9, 12, 0).
- Steering by mode (Behavior 2): d = (0, 0, 1). Mode 0: v = (0, 0, 1). Modes 3 and 4: v = (0, 0, -1). Mode 5: v = (1, 0, 0). Mode 6: v = (-1, 0, 0). Mode 5 with facing (1, 0, 0) (dot 1.0 > 0.9): weight 1.0, so the move direction becomes (1, 0, 0) at once. With facing (0, 0, 1) (dot 0): weight 0.1.
- Look-ahead (Behavior 2a): P (0, 0, 0), W (0, 0, 20). s = (-1, 0, 0), so the lanes are x = -1 and x = +1 from z = 0 to z = 20. Another character stands at (0.5, 0, 10) and the hero at (0.5, 0, 0). The lane x = -1 passes 1.5 from its axis, inside radius 5.0: hit, R = 5.0. u = (0, 0, 1) and n = (-1, 0, 0). The first candidate is (-9.5, 0, 10). If it is off the mesh and the scan cursor is clear, the second is (10.5, 0, 10). If that is also off the mesh, W is kept.
- Goal latch clearing (Behavior 2b): a mesh of 120 triangles and a goal point on none of them. Batches test triangles 0 to 49, 50 to 99 and 100 to 119 on three ticks. The third batch ends at the last triangle and misses, so the latch clears on that tick.

- Live petpet timer choice (synthetic, Behavior 12.8): at r = 1.0 the threshold is 0, so every roll above 0 selects animation 1; at r = 1.25 the threshold is 50, so roll 51 selects animation 1 and roll 50 selects animation 6; at r = 1.5 the threshold is 100, so no possible roll selects animation 1.
- Draik wind throw (synthetic): hazard at (0, 0, 0), hero at (3, 0, 4). u = (0.6, 0, 0.8); (0, 12, 0) + 9u = (5.4, 12, 7.2), length 15; the hero's velocity becomes (3.6, 8.0, 4.8), speed 10.
- Draik breath back-off (synthetic): Draik at (0, 0, 0), hero at (6, 0, 0) (distance 6, below 10). The target this tick is (-5, 0, 0); next tick, from its new position, the target is again 5 units behind it, still on the x axis.
- Draik breath beam (synthetic): mouth (0, 3, 0), beam end (0, 1.5, 20). Hero at (4, 0, 10): cylinder centre (4, 1.5, 10), horizontal offset 4 is below 5 and the segment height there is 2.25, within 0.875 of 1.5: contact. Hero at (6, 0, 10): offset 6, no contact.
- Draik hit groups (synthetic): in stage 3 a swing that overlaps only bone 27 (left leg) registers nothing; one that overlaps bone 11 (left arm) registers. In stage 5 a swing that overlaps only bone 7 (body group) registers nothing; one that overlaps bone 54 registers (head group).
- Draik spit (synthetic): mouth (0, 3, 0), hero at (10, 0, 0): aim point (10, 2, 0), velocity (0.99504, -0.09950, 0) per tick.

### Test vectors: a real monster row and the hero damage it deals (disc)

- Monster table (resource 530): N = 112, first[0..3] = 0, 1, 2, 3; first[80] = 183 and first[81] = 197, so monster type 80 owns rows 183 to 196. Row offset = 0x1D0 + row x 0x504.
- A spawn record with type 80 and variant 3 selects row 183 + 3 = 186, at byte offset 239,288: hit points 50, target type T = 4, radius 2.0, height 2.0, behaviour class 17. Variants 0 to 6 (rows 183 to 189) all have 50 hit points, radius 2.0 and height 2.0 and differ in T (1, 2, 3, 4, 5, 6, 6). Variant 7 (row 190, the archer rows) has 40 hit points, height 3.5 and T = 1.
- Row 183, attack damage values at +0x34C + 4k for k = 0 to 4: 25.0, 25.0, 50.0, 0.0 and 10.0 (the stored words are single-precision floats). The knockback word K at +0x35C is 10.0 for k = 0 and 0 for k = 1 to 3. Row 190: 25.0, 25.0, 0.0, 10.0, 10.0.
- Damage to a hero from this row (no curse, no strength, not blocked; factors as in SPEC-0036, Enemy attack damage to a hero; the columns are no body item, body item 122, body item 125, body item 125 with petpet kind 1): attack 0 (25) gives 25, 15, 7, 6; attack 2 (50) gives 50, 30, 15, 13; attack 4 (10) gives 10, 6, 3, 2.

## Open questions

- **Default if unresolved (rewrite choice).** If the hero stands exactly above or below a wind hazard (horizontal distance 0, so u has no direction), use u = (0, 0, 0): the throw is straight up at speed 10. A Draik spit has no lifetime limit; a rewrite may remove one that has travelled 300 ticks as a safety bound.
- **Resolved (R-0066).** The bones of the seven hit groups, the radius-2.0 bone (7) and the length axis (the bone's local Y axis), the breath beam test (segment to the end point against a vertical cylinder around the hero), the spit (own records, event bit 0x20, no pool fields), the wind hazard (the Draik's own position, throw velocity 10 x unit((0, 12, 0) + 9u)) and the breath back-off (5 units behind the Draik, away from the hero) are in Behavior 6.4 to 6.6 (High; axis and beam test Medium-High).
- Resolved (R-0039, item 1): the Miamouse's walk mode (mode 0) plays clip 2 and its flight and run mode (mode 2) plays clip 3. The class-21 entry sets the mode slots to 2, 2, 3 and -1, and the slot lookup is in Behavior 2 (High).
- Resolved (R-0039, in part): (1) resolved above. (2) Resolved (R-0039, item 2): the damage-bit choice of attach points 1 to 4 applies to the hero's carry as well, since the carry step reads the carrier's frame bits (SPEC-0005, Attach points and carrying). The offset is stored in the carrier's record, not the Miamouse's (scale 1.0; vector (0, 0, 0, 1)). The hero's actions 178 to 181 carry no damage bit 0x2 to 0x10 on any frame (actions 179 and 180 carry no table-B bit at all: Tor clips 67 and 68, Roberta clips 106 and 105; actions 178 and 181 carry only bit 0x20, a release event, on frame 5 of 10 and frame 1 of 8), so the carried Miamouse is always placed at attach point 0 (High). Test vector: Tor action 179 has 18 frames whose table-B values are all 0, so on every frame the choice gives attach point 0. (4) Resolved: the other proximity routines of pack 1163 are entities 8, 6, 14, 13 and 9, each a template W routine with its own base line and bracket rule (SPEC-0045, module 1163). (5) Resolved: the serving scene's sets 3 and 4 are regions 1167 and 1168 (SPEC-0045, module 1163).

1. Resolved: a spawned enemy takes its class from data, not from its spawn name. The spawn record's type and variant select a monster row, and the row's default class (field 0x36, or the companion row's field 0x10 when none) is requested at priority 1 (Behavior 1, step 4). The spawn-name value (for example 53) is not a class id.
2. Monster table: Resolved: the units of the timer and speed fields (ticks, clip-rate factors and units per tick; Layout section 4, Units), the trailing 804 bytes of resource 530 (unread data after the 259th row) and the fields of resource 531's rows (Layout section 4, Resource 531) are settled. Also resolved: the default behaviour class is applied by the spawn itself at priority 1 (Behavior 1, step 4), not by the level script.
3. Resolved: the explicit requests of Behavior 1 (step 4) are named. Classes 23 and 24 (walk to a locator), 25 and 26 (the same, then turn to the locator's facing) and 35 (walk along a path, 0.5 units per point) are the level-script walks (SPEC-0030 rows 106, 108 and 111; SPEC-0056); class 34 is the live (summoned) petpet controller (SPEC-0047) (High). Classes 4, 22, 27, 28, 30 and 31 are described in Layout section 3: class 4 sits only on unplaced rows and classes 22 and 27 to 31 have no requester, so none of them runs on the disc (Layout section 3a; High). The routines of classes 29, 34 and 35 are written out in SPEC-0038 (Behaviors 32, 27 and 33, and 31). Class 6 is the townsfolk class (Layout section 3; open question 71 is resolved). Class 20 is described in SPEC-0047. Class 21 (Behavior 28) and class 36 (Behavior 29) are specified here. Class 33 is in SPEC-0047. Class 37 is the Darkest Faerie family (Behavior 7); class 38 is the Queen Fyora controller (Behavior 30).
4. Resolved: the Mortog zone test is the horizontal squared distance to A, strictly below L; the wander rule uses 2L and 3L with L squared (reproduce as stated); the hold-end and zone-rule unit vectors are three-dimensional (Behavior 3; High).
5. Resolved: the hold maximum is the non-zero gate of the hold end. With it at 0 the hold never ends; script module 1363 uses this to keep its four Mortogs in the zone once the puzzle is done (Behavior 3; High).
6. Resolved: the move speed is 12.5 units per second with 1 alive and 10.0 with 2 alive (fixed constants), otherwise the word 0x2DAB50. That word is 7.5 on disc; alternate loop A stores 10.0 into it (and 0.5 into 0x2DAB94) at its start and nothing restores them, so they keep these values for the rest of the session. Alternate loop A runs only where the world's level state 0 is set, which for the sisters is the Act 1 escape (pack 1174, main-story step 21; SPEC-0044, module 1174). The normal Sisters fight (pack 1122, step 43) comes later. The word is program memory and is not saved, so the three-alive speed of the normal fight is 10.0 in a session that has already played the escape and 7.5 in a session that was started from a save made after step 21 (High for the writes and the readers; Medium-High for the step order). Test vector: three sisters alive in the normal fight, no alternate loop started since the program started: speed 7.5 units per second, 0.125 per tick; after the escape has started alternate loop A in the same session: 10.0 units per second, about 0.1667 per tick. The low-HP band of the chooser picks state 6 when the special cooldown has expired (High). Resolved: the separate fifth per-sister timer is the missile-dodge scan cooldown described under Counters and Behavior 20. No authored label for a “fifth counter” is established.
7. Resolved: the state-5 kind selector maps 0 to poison (status id 2), 1 to curse (status id 1) and 2 to slow (status id 3); the state-6 selector maps 0 to thorn, 1 to leach and 2 to Mote-eater. Controller classes 13, 15 and 16 initialize the selector to 0, 1 and 2, respectively (SPEC-0011; High). State 7 sub-state 3 recomputes the group-indexed ring point through the same helper used in sub-state 0; its route and travel distance depend on current position and navmesh routing. In region 1123, action 23 maps to main clip 15 for Malice, Spite and Vanity in both state 7 sub-state 3 and state 8's default flinch. State 8's variant action 24 maps to clip 16. The action-23 sound cue is the table's take-damage cue. The data states no design meaning for the shared action beyond these clips.
8. Resolved: the state-4 handler requests sister main action 17; the state-5 handler requests main action 18 for every missile kind. In region 1123, the main animation tables of Malice, Spite and Vanity map both action IDs 17 and 18 to clip 19, so these attacks reuse one sister clip. “17” for the slow missile is its pool visual kind, a separate projectile setting. The attacks still use different projectile behavior/visual kinds and effects (High; SPEC-0011).
9. Resolved: class 13 is Malice (also established by Open question 138 and the placement data). When level flag 0 is set, it takes alternate loop B, the same loop used by class 15; it has no unique alternate loop. This is separate from the shared state set and does not change state-13 behavior elsewhere.
10. Resolved: the pen point is set by the two script setters, SetWhinnyPenmesh and SetWhinnySwitches, from a locator's point. The flee push flag is the shared navmesh-blocked latch: edge confinement or last-valid-position restore sets it, with a 120-tick timeout. If set before flee target selection, Whinny takes the safe-point branch instead of the direct-away branch; if set during the direct-away walk, Whinny abandons that target, resets its flee sub-step and clears the latch. On the single-pen Farm configuration, the occupied bit does not reserve the pen from other Whinnies; each can enter its entrance and follow the same pen point. The path-is-clear test of the direct-away step is only that the 4-unit candidate lies on the Whinny's current navmesh (Behavior 5, sub-step A). [High]
11. Resolved: the reflection gate checks the equipped shield Mote's element kind, not the shield-family equipment category. The fire beam accepts Mote ids 160–161 (element 1) or 172–173 (element 7); the ice beam accepts ids 162–163 (element 4) or 172–173 (element 7). The shield-family items (config types 2 and 5) do not satisfy the element lookup; the Mote in equipment word 1 is what the check reads (Behavior 6.5; High). The bite-following hero call remains as described: it produces the blocked-hit reaction regardless of guard state because its take-hit passes blocked flag 0 (Behavior 10.5; Medium-High).
12. Resolved for the Undead Draik route in pack 1096: after the stage-1 hit reaction, HP at or below 66% sets level state 0 and starts transition 21; after the stage-3 hit reaction, HP at or below 33% sets level state 0 and starts transition 23. When the stage-5 death animation finishes, the head breaks and level state 12 is set before the Draik enters stage 6. Pack 1096's main state thread consumes state 0 through its first eligible state-pair helper and state 12 through the finale (SPEC-0041 module 1096; SPEC-0058 step 42). The route starts only when game flag 45 is clear: world activation starts the fight setup (movie 51, entity 0 activation and boss-fight instance 6 stage 0); the state-12 finale sets level states 13 and 14, game flags 45 and 316, and boss-fight instance 6 stage 1. With flag 45 set, the main state thread is not started. The separate references to game flags 4, 5, 10, 11, 13 and 17 are not consumers in this pack-1096 route and remain outside this answer.
13. Resolved in part: the hazard-slide scalar is applied directly as a 0.1-unit position step, once per Draik tick; it is units per tick, not units per second. The spit projectile advances by its 1.0-unit speed value once per projectile update; the Draik calls that update once per tick, so it is also units per tick, not units per second (Behavior 6.6; High). The full state set of the tail and orbit handlers remains open.
14. Answered: the breath pairs at 0x2DA9C8 and 0x2DA9CC (fire) and 0x2DA9D8 and 0x2DA9DC (ice) are the beam duration range in seconds (2.0 to 5.0), drawn as a whole number of ticks times 60, inclusive (High).
15. Partially resolved: the pack 1096 initializer sets indices (17, 20, 22, 15, 16, 19, 18, 21, 23; part order as in Section 6) into nine shared current values, and each Draik setup reads those values when it initializes its breakable records. The configuration is not keyed by actor or entity ordinal. Pack 1096 has one placed Undead Draik at entity ordinal 0, but static data does not establish whether that actor is initialized after the setter runs. Resolved (Medium-High): the module's Init runs before the world's characters are bound, so the pack 1096 Draik, the only one reached in play, reads the values (Behavior 6, Breakable parts). Test vector: on a fresh entry to the Treasure Vault, the Draik's nine breakable records are set up with part object indices 17, 20, 22, 15, 16, 19, 18, 21 and 23, in the order left leg, right leg, tail, chest, left arm, right arm, left wing, right wing, head.
16. Resolved: no path sets the Draik's stage to 2 or 4; the stage is set only to 1 at start and to 3, 5 and 6 (Section 6.1; High).
17. Resolved: Draik hazard target offsets [-5, 5], 60-tick renewal, 4.0 hit threshold and 12.0 upward launch component are fixed code values, not tuning words. The bite approach threshold of 6.0 is also a fixed code value. The nearby 12.0 wind-idle tuning word does not supply the hazard launch component.
18. Partly answered: class 37 is the Darkest Faerie family and class 38 is Queen Fyora (Layout section 3; Behaviors 7 and 30). The class-37 dive uses fixed upward velocity 12.0, horizontal velocity 9.0 and offset -5.0 (Behavior 7, loop B). The class-37 route tests game state 364 first, then level state 18: these select the four-member controller, loop A and loop B respectively; packs 1432 and 1174 set the first and second values (Behavior 7). The narrative purpose of these flags beyond routing, and the counter and timer meanings, remain open.
19. Partly answered: the aim blend weights 0.1, 0.25 and 1.0 are per-tick turn-toward weights, with 1.0 snapping to the target direction (Behavior 2). The behavior-table pair previously described as aim slots is an adjusted call entry for the turn-toward method: its first word supplies a character-address adjustment (zero in the detected entries), and its second identifies the method. Callers supply a computed direction and blend weight, not an aim-slot index. The method blends into the character's shared motion-record direction vector, which generic locomotion steering also reads and updates. Still open: whether any behavior shares a separate target vector in addition to this common direction state, and which target-point fields individual behaviors use. [High for the detected call shape and shared vector; Medium for its full use across all derived classes.]
20. Resolved: the navmesh pick tests the character's own position (Behavior 2). The nearest-triangle search of Behavior 2b takes its query point from the goal latch (the offset goal point).
21. Resolved: the mode word is +0xA3C; modes 0 to 4 read slot words +0xB58 + 4m, mode 5 reads +0xB70 and mode 6 reads +0xB6C, so the slot setter's 6th value is mode 6's clip and its 7th is mode 5's. Modes 3 and 4 steer toward -d, mode 5 toward up x d and mode 6 toward d x up (the strafes use weight 0.1, or 1.0 within a dot product of 0.9 of the facing), and 7 to 10 are the flyer forms of the same rules. The fixed writers of modes 3 to 6 are all bosses: the Black Knight, Gelert, Gnarfas, the Werelupe King and the Ixi Chief (Behavior 2).
22. **Resolved in part: Monster schema record field order.** The boot-time builder writes section label at +0x00, kind at +0x04, key name at +0x08, value byte offset at +0x0C, upper clamp at +0x10 and lower clamp at +0x14. The interpretation and application of clamp fields for integer kinds remain open beyond the documented sentinel rule. **[High for field order; Medium-High for float-kind bounds; open for integer-kind clamp conversion]**
23. Resolved: the look-ahead tests two lanes 1.0 to either side of the straight line against vertical cylinders (radius 5.0 and length 20.0 for blocks, grid entries and characters), keeps the nearest hit, and shifts the goal by the obstacle radius + 5.0 to the side given by the hero's direction to the obstacle, then the other side (Behavior 2a). Still open: what the locate's scan cursor gate on the second side means in play (Medium). Nearest triangle (Behavior 2b): the centroid distance includes x, y and z; vertically separated floors therefore compete by full 3D distance (High).
24. Resolved: neither routine finds a target. Behavior 2 is the navmesh pick and Behavior 2a the steering look-ahead; class-17 enemies always target the selected hero (SPEC-0038 Behavior 6).
25. Resolved: navmesh kind 1 is the ground mesh (mode 0) and kind 2 the flying mesh (mode 1) (Behavior 2, High).
26. Resolved: enemies follow the precomputed route table of their mesh, through the navigation step and route step of SPEC-0056; there is no run-time graph search (High).
27. Resolved (High for the rule, Medium for the design reading; Behavior 2b, Hidden test): the shared character-visibility predicate gates navmesh occupancy, hero auto-target eligibility, enemy-projectile contact and the other interaction tests that skip a hidden character. The rule is a fixed decision list over the invisibility-off word, the row's Display in Default field, the AI mode against the record's default mode, the hide animation and the Tattered Cloak (item 126) check on the active hero. The design reading is that a kind with Display in Default 0 is an invisible ambusher that cannot be hit or targeted in its default state, and the cloak makes it interactable. The DisableInvisibility script command clears the exclusion (SPEC-0030, row 121).
28. Resolved: the nearest-triangle search (Behavior 2b) has no kind or group filter; it walks the triangles of the character's current navmesh.
29. Resolved: Behavior 8 is the Faerie Sisters' encounter-start gate (classes 13, 15 and 16); the third condition is hero weapon reach against the sister, and kinds outside 0 to 2 set the latch without posting. Progress flag 220 is an Act 1 stage-2 flag that suppresses the helper. The first hero test reads the cached attack-active predicate, which reflects the current animation frame's event bit 0x1. Because region characters run before the selected hero, the sisters read the cache from the preceding pass; the hero update refresh is visible to them on the next pass (Behavior 8; SPEC-0026, step 8.1; SPEC-0036). Resolved: the pose-sampling tick of the weapon-reach test is the previous tick's animation pass for the axis D1 and the pass before it for D2 (Behavior 8 item 3; Medium-High). Test vector: a hero whose attack-active predicate becomes true in its update on tick T is first seen by a sister on tick T + 1, and the sister's reach test then uses the hero's pose from tick T's animation pass (D1) and tick T - 1's (D2). Resolved: all three variants reach the gate in shipped play. Each alternate loop calls it once per pass (class 13 and class 15 loop B, class 16 loop A) and the main loops never call it, so it runs only where level state 0 is set, which is pack 1174 (its Init sets level state 0). The posted event kinds 1, 3 and 2 become level states 1 (Malice, class 13), 3 (Spite, class 15) and 2 (Vanity, class 16), and the stage thread of pack 1174 turns level states 1, 2 and 3 into stage bits 2, 3 and 4 (SPEC-0044, module 1174). In the normal fight in pack 1122 the gate never runs. Flag 220 is set at the end of the escape, together with flag 219, when the Act 2 start runs; the data states no narrative reason beyond that (High for the callers and the posts, Medium-High for the story reading). Test vector: in pack 1174 with flag 220 clear, Spite (kind 1) passes all tests: level state 3 is set, which the stage thread turns into stage bit 4.
30. Resolved: the first hero-side query runs before the latch check; a null result clears the latch even when set. The second query and reach test run only after a non-null first result and while the latch is clear.
31. Draik orbit (attack number 18): resolved in part. The stage-2/4 branches move radially to a drawn hero-relative radius, then advance a tangential target 0.1 units per tick and turn toward the hero. Their action-table requests differ by stage and by whether a navigation target is available (Section 6.8; High for the values and branches, Medium for the target geometry). These stages are never entered in the shipped fight. The handler does not write a top-level exit; the condition or timer that would end orbit remains unknown.
32. Answered: a Draik attack handler ends by setting the state to 0 (idle). The stun states 26 and 27 are entered by reflection and return to 0 at their ends (High).
33. Answered: spit is attack 8 (fire) and attack 9 (ice) (High).
34. Answered: the integer-10 words 0x2DA9D4 and 0x2DA9E4 are damage passed to the hero's damage routine, with the Draik as attacker (High for the call; Medium-High for the meaning). They are not a sound or animation id.
35. Resolved: the pool lifetime counter is in ticks and is written by the spawner (Behavior 13). Each update releases the slot when the counter is already negative (no hit, no event); otherwise the slot moves and the counter drops by 1, so a lifetime L gives L + 1 moves (Behavior 9). The six-slot step with a probe is the hero's shot update (SPEC-0036), not an enemy step.
36. Resolved: the pool has homing in its own update: kind 17 (flight mode 2) blends 0.75 of its old direction with 0.25 of the direction to the hero (Behavior 9). The four-slot homing, detonation and flight steps are the slot handlers of the class 37 and class 38 controllers, run by each controller's own slot dispatcher (Behavior 19).
37. Resolved: launch speeds are in units per tick, and the ballistic flight time T is in ticks. The ballistic lifetime trunc(60 T) + 1 is 60 times the flight time, a unit slip in the original (Behavior 13, Edge cases).
38. Resolved: the ballistic launch is vy = (0.5 g' T^2 + dy)/T with g' = 32/60^2 (g = 32.0 from the physics block written at start-up). The band progress runs from far to near: u = 1 - (d - bmin)/(bmax - bmin), so u = 1 at the near edge, and it is not clamped (Behavior 13).
39. Resolved: projectile hits are decided by the pool update itself, one character per tick (the hero, the owner after a reflection, or the spell target) against its body capsule (Behavior 9). The hero's projectile-in-front scan of its guard logic (SPEC-0036) is not a hit test, and the projectile test over nearby objects is part of the world collision, whose moving-object gate is a 3D squared distance below 22500.
40. Resolved: flag 0x02 marks a summoning slot. On a missile slot it selects the lobbed summon (kind 8), which spawns the row's key 0x3CC + 2k at the landing point, and in the class-17 attack choice it limits the slot by the live summon count; both readings hold (Behaviors 13 and 31; SPEC-0038 open question 19). The per-slot flag that chooses between two hit routines is the hit selector: 1 tests the hero (default), 0 tests the owner after a reflection (Behavior 9).
41. Resolved: the spawn-pending flag is cleared by the sequencer on the first tick the slot's spawn event is not active (SPEC-0038 Behavior 13, state 2).
42. Resolved: routines A and B are class-37 Darkest Faerie codes 3 and 4. Routine A submits a type-0 elemental orb with kind 1–6 or a type-1 aimed bolt with kind 0; routine B submits the type-1 aimed bolt with kind 0. Both draw a [60, 240] tick cooldown after the animation completes (Behavior 11).
43. Resolved: 4007 and 4008 are effect ids, not pool kinds. Routine F (the Queen Fyora controller's state 2, animation 23) and routine G (a class-37 loop, animation 10) relaunch the first active type-1 slot of the controller array at 0x36D3F0 as type 3 with a 60-tick timer and play effect 4007 or 4008 on it; nothing reaches the enemy pool (Behavior 11, rule 4; Medium-High).
44. Resolved: the trail lookup in Behavior 13 selects an entry of the character's 11-entry model-part pointer table, using zero-based indexing; it does not select an effect-id cue. Kind 17 is the homing kind of the pool (Behavior 9).
45. Resolved: one ballistic formula applies to flight mode 1 and to the lobbed summon (Behavior 13). The term 0.5 g t is the per-tick velocity term, and the term 0.5 g t^2 / v is 0.5 g' T^2 / T. The kind-0 spawner is flight mode 0 and has no speed ratio.
46. Resolved: the "ambient cue timers" of Behavior 12.9 are the hero idle picker (SPEC-0033). The values 0x6B to 0x6F, 0x9E to 0xA1, 0x9D, 0x92, 1 and 0x23 are hero animation ids, neither sounds nor effects, and since hero action tables carry no sound word the picker plays nothing (High).
47. Answered: owners of Behavior 12 routines are in the table above and sections 12.1–12.13; unresolved owners remain explicit there. Class 17 uses SPEC-0038.
48. Answered: the eight-slot shuffle is the Gelert Assassin rainbow dash; see Behavior 12.5 and SPEC-0013 section 4.8.
49. Answered: the outer gate and the meanings of 1, 6 and 7 are specified in Behavior 12.8; the timer ratio and roll outcome are given there.
50. Resolved: the routine of Behavior 10.1 (block push capped at 40, second facing test 0.95) has no call, no stored pointer and no address load anywhere in the boot program, so it never runs; do not implement it, and its open operands do not matter (High). The block test that runs is the class-17 strike's (Behavior 10.2), and the knockback that runs is Behavior 10.1a: K x unit(9 x attacker facing + (0, 12, 0)) on the hero, then state 48 with the grounded flag cleared; heroes push no enemies (High).
51. Resolved: the attack volumes are the five monster weapon cylinders (row +4(N-1) radius, row +0x14 + 4(N-1) height), each tested only while its event bit is set, against the target's bone capsules (Behaviors 10.2 and 26); the 65-unit reach test is the single-weapon test with the fixed length 1.5 for enemy-kind characters (Behavior 10.2).
52. Resolved (High): Behavior 10.3 is the Ixi Chief's incoming-hit check (class 11; SPEC-0014). The state change falls on the struck boss: state 0x12 is the Ixi Chief's own get-hit state, numbered in its own state table, and not state 18 of SPEC-0038.
53. Resolved (High for the clips, the handlers and the shared profile): there is no per-substate animation table. State 8 is fire spit and state 9 is ice spit. Both select the clip by Draik stage (stages 1–5: 6, 19, 31, 44, 52), wait for its event child and then fill a different projectile record. The motion, collision-callback argument and shared flight values match; their attached effects use separate configurations. Stages 2 and 4 are unreachable in play. The clip is requested from the stage every tick until it is the current clip; the launch happens on the first tick with event bit 0x20 on, once per attack; when the clip reaches 0.99 progress the spit cooldown is drawn (1 to 2 s, by stage) and the state returns to 0. The two states differ only in the record they fill and in the pair of effect definitions each record starts: fire spit uses the shared-group definitions 528 and 991, ice spit 527 and 992. Every number the spit uses is the same for both: speed 1.0 per tick, damage 10, hit radius 0.5. So no damage, status or timing difference exists between fire and ice spit, beyond how the two effects look (Section 6, "Spit appearance": sprite effects only, no model; red for fire, blue for ice). Test vector: stage 1 fire spit requests clip 6, stage 3 requests clip 31 and stage 5 clip 52, for state 8 and state 9 alike; at the first tick of event bit 0x20 one record is filled (the fire record for state 8, the ice record for state 9) and it flies 1.0 unit per tick toward the point 2 units above the hero. [High for the clips, records and effect ids; Medium for how the effect pairs look]
54. Resolved: 0x22C + 4k is the Counter Attack Trigger code and 0x23C + 4k the Counter Attack Chance of the class-17 counter-attack choice (SPEC-0038 Behavior 14); codes 0 and -1 both never trigger. The normal attack choice (SPEC-0038 Behavior 7) reads the trigger chance 0x19C instead; both choices run every agro tick, the normal choice first.
55. Resolved: the projectile test over objects within 150 units is part of the world collision of the pool update (Behavior 9); it tests world objects and moving objects in their local frame, not the enemy list as targets.
56. Resolved: the dispatcher of the spawners is state 2 of the attack sequencer (SPEC-0038 Behavior 13): row flag 0x02 selects the lobbed summon, otherwise flight mode 0, 1 or 2 selects the spawner.
57. Resolved: the dispatch helper of Behavior 12.6 is the Miamouse dispatch (class 21, Behavior 28). States 0 to 4 are its main states, 10.0 is the hero notice distance, and clip 13 is the release clip that forces state 3 (High).
58. Answered: the 50-second timer is the Darkest Faerie's absorb-window duration (SPEC-0054). Its volley-start routine also launches the four burst summons and sets the two timers (Behavior 18; High).
59. Resolved in part: the +/-0.1 step changes the Draik's position Y directly; it is not a vertical-velocity write (Behavior 6.9; High). The keeper acts after death in stage 6, but the value of its +0x50 runtime offset remains unknown because no reachable path writes it before then. Whether the controller frame starts with this offset at zero is unestablished, so the post-death target cannot be assumed to equal ground height (Low).
60. Resolved: there is one global pool of ten enemy-projectile slots of 0x15D0 bytes (Behavior 9). The six slots stepped from the hero's own update are the hero's shot slots (SPEC-0036).
61. Resolved: the lifetime counter starts at the lifetime in ticks of the setup record (Behavior 9): trunc(range / v) for straight shots, twice that for homing, trunc(60 T) + 1 for ballistic shots and trunc(distance) + 5 for spells (Behavior 13). A lifetime of 0 or 1 still moves once or twice.
62. Resolved: the probe and its event lookup belong to the hero's shot update (SPEC-0036), not to an enemy projectile. Enemy projectiles post no hit event; they release on contact or on world collision (Behavior 9).
63. Resolved: the six-slot routine with motion modes is the hero's shot update (SPEC-0036). The pool update selects its motion by behaviour kind (Behavior 9).
64. Resolved: in the pool update the lifetime test comes first and ends the slot only when the counter is already negative, so the last move happens on the update that takes the counter from 0 to -1 (Behavior 9).
65. Resolved: the 150-unit gate is a 3D squared distance below 22500 from the projectile to a moving object, inside the world collision (Behavior 9). It does not cover the hero; the hero is tested by the character hit test.
66. Resolved (High): the owner of Behavior 10.4 is the Draik Skeleton's stage-4 missile dodge, which never runs in play. The six entries are the hero's six shot slots, the roll is drawn only after a shot path passes, and state 0x13 is the Draik's dodge-missile state (19), a different state of a different routine from the Ixi Chief's get-hit state 0x12 (Behavior 10.3).
67. Member slot group (Behavior 14): the class that calls the binder; whether removal of a member from the group list exists and how; and the outcome when a member cannot be created (refused).
68. Resolved: the zone chooser is state 2 (follow the hero) of the live petpet, class 34 (Behavior 12.13). Its states 0, 2 and 3 are animation codes 0 (idle), 2 (walk) and 3 (run); the 0.1 is a fixed turn weight, applied in gaits 1 and 2 (High).
69. Shared words: 0x2DB250 is used both as the Whinny pen count (Behavior 5) and as a mode flag of a close-test controller; 0x2DB268 is used both as the Whinny entrance id and as a per-slot list of a chase routine. Which owner uses which meaning is not settled. The values (1 and -1) are read from disc. Partly resolved: the chase routine that reads 0x2DB268 as a per-slot list is the class-7 chase-to-point controller (SPEC-0038 Behavior 16), so both readings of 0x2DB268 belong to Whinny. The owner of the close-test reading of 0x2DB250 is still open.
70. Resolved (High): classes 23–26 are explicit level-script walk requests: WalkEntity selects 23/24 and WalkEntityAndOrient selects 25/26 (SPEC-0030 rows 106 and 108). Their arrival, navigation, facing-gated hold and orientation behavior are described in SPEC-0056. The 180-tick counter governs the hold animation and is not a navigation retry; a failed initial navmesh lookup logs and ends through the idle path. These classes have no single enemy owner. The class-7 Whinny chase-to-point controller in SPEC-0038 (Behavior 16) is separate. Walk displacement/speed remains under the existing SPEC-0040 movement questions.
71. Resolved: class 6 is the townsfolk class. It is the default class of every placement whose monster row has no class (row +0x36 none, companion row +0x10 = 6): 666 of the 3,036 kind-3 placements, under 280 names. The default-class request sets the townsfolk flag exactly when the class is 6. The mode bodies, including wander (5), patrol (6) and the greeting pause (13, with the behind test dot(unit horizontal direction to the hero, the townsfolk's facing) below -0.1), are specified in SPEC-0056 (High).
72. Resolved: a new projectile takes the first free slot of the single ten-slot pool; there is no placement at a count and no compaction (Behavior 9). The count-and-append list at owner +0x2F8 (entries of 0x70 bytes from +0x300) is a per-character list of pending contact shapes used by the character mover (SPEC-0040, SPEC-0018), not projectile storage.
73. Resolved: the tag word 3 belongs to the per-character contact-shape list (an oriented box shape; tags 0 to 3), not to projectiles (SPEC-0040, SPEC-0018).
74. Resolved: the three scalars and the unit direction belong to the contact-shape list (SPEC-0040, SPEC-0018), not to projectiles.
75. Resolved: the vector inputs belong to the contact-shape list (SPEC-0040, SPEC-0018), not to projectiles.
76. Resolved: the owner-held list at +0x2F8 and +0x300 is the contact-shape list (SPEC-0040, SPEC-0018) and the six slots are the hero's shot slots (SPEC-0036); neither is the enemy projectile pool.
77. Resolved in part: Behavior 15 is a world-zone submission pass, not enemy AI. It clears a per-world zone-object list that zone-instance callbacks refill, up to 15 entries; hero collision and interaction scans consume the refreshed entries. Its callback results are summed and returned to the wrapper, which discards the sum. The authoring names and exact roles of the five groups remain unidentified.
78. Answered in part: the sole caller is the world tick, and the time input is the frame-timing value of SPEC-0007 (Behavior 15). The caller argument and whether it varies remain open.
79. Resolved: the gravity is g = 32.0 from the physics block (run-time word 0x3A1A7C, written at start-up), applied per tick as g / 60^2; launch speeds are in units per tick (Behavior 9).
80. Resolved: this routine is the character light-slot chooser, not an enemy routine; the four slots are a character's light slots and the candidate types are light types 0 to 3; the radius gate runs for types 1 and 3 only (SPEC-0046, Character light slots).
81. Resolved: this routine is the character light-slot chooser, not an enemy routine; the score is the light weight (the strength, or the point-light falloff) and the per-slot threshold is the weight of the displaced light (SPEC-0046, Character light slots).
82. Resolved: this routine is the character light-slot chooser, not an enemy routine; the occupant's word is its light strength; the wait flag is the slot lock, which lasts for one walk of the light set (SPEC-0046, Character light slots).
83. Resolved: this routine is the character light-slot chooser, not an enemy routine; the shared word is the slot weight: the cross-fade writes the displaced light's weight there (SPEC-0046, Character light slots).
84. Resolved: this routine is the character light-slot chooser, not an enemy routine; the two words are the locks of slots 2 and 3 and the two floats are the weights of slots 2 and 3 (SPEC-0046, Character light slots).
85. Resolved: this routine is the character light-slot chooser, not an enemy routine; the stored value is the light's colour or direction transformed for the character (SPEC-0046, Character light slots).
86. Resolved: this routine is the character light-slot chooser, not an enemy routine; the precondition is the zone gate: the light's zone key against the character's zone value (SPEC-0046, Character light slots).
87. Resolved: the actor-list dispatcher (Behavior 17) is the character walk of the hero shot slots. The class predicate is the hittable-target method, its method slots are the guard method, the take-hit method and the push method, the hit list is the target's bone hurt records, the word at slot +0x68 is a stored push vector, the second branch is the push call after the take-hit, and the 0x18C0 is an offset of a 16-byte scratch block that nothing reads. The list words at 0x35C8F8 and 0x35C8FC have no boot value (outside the file image; rebuilt on every pass) (High for the roles; Medium for the push vector).
88. Resolved: the codes of Behavior 2c are animation indices, not messages. The two tests are "animation n is current" and "the current animation has finished (0.99 of its length)"; the relayed codes are numbered level states. Each owner is listed in Behavior 2c (High).
89. Resolved: the two routines that set state 18 are the Draik's own stage-2 and stage-4 choosers, run only by its per-stage chooser dispatch. No other class calls them, and orbit stays unreachable (Behavior 6.8; High).
90. Resolved: the 7.0 to 14.0 draw is the orbit radius (a distance that scales the direction to the hero) and the 5.0 to 15.0 draw is the flying height, stored as the Draik's flying offset (Behavior 6.8; High).
91. Resolved: the kind searches match the monster row's default class (37 or 38). Class 37 loop B calls the class-38 search, which finds Queen Fyora; the class 38 controller calls the class-37 search. Neither object is the boss of Behavior 8 (Behavior 7; High).
92. **Resolved: six-entry attack-band tables (Behavior 2).** The five stage-choice routines pass the current stage value directly as the getter's 1-based table index. Their dispatcher has cases only for stages 1 to 5; stages 2 and 4 are never entered, and death stage 6 has no attack-choice case. Shipped attack choices therefore read indices 1, 3 and 5 only. The different entry-6 values in these attack-band tables are not read in play; their authoring purpose is unknown (Behavior 6.3; High for static caller and stage scope).
93. Resolved: the hold end only sets a move goal (k x u with k in [4, 8], no position term), sets state 1 and decreases the zone count. Mortog's routines create no projectile and do not aim at the hero; the second description was wrong (Behavior 3, Hold end; High).
94. The word at 0x2DA638 (0.75) is the per-tick step factor of the controller slot homing steps (Behavior 19) and of the damped homing step (Behavior 2e); each routine applies it once per tick. Resolved: the slot homing steps are the per-tick handlers of the class 37 (Darkest Faerie) and class 38 (Queen Fyora) controllers. The type-1 member of Behavior 7 is the second-form homing step of Behavior 19 and moves 0.75 units per tick.
95. Resolved: the health-band check is the Plague Serpent's phase check. No band is stored; the wanted phase is compared with the phase counter at offset 0x1F8, and the helper records the two phase-change anchor slots. Healing repeats phase changes as SPEC-0016 states (Behavior 2d; High).
96. Resolved: there is no shared message 4. The three readings are the Gnarfas animation-4 state, the Plague Serpent animation-4/6 pair and a Draik stage-4 chooser call (Behavior 2c; High).
97. Resolved: message 13 is the class 37 stun count (animation 13). Its done flag stops the four-member controller's choice step and is cleared by code 1 on level state 15. Message 27 is the Draik's stage-3 stun hit registration: state 27, animation 46 not current, the hero's melee or projectile test, and a once-per-swing latch at offset 0x5C that clears when neither is active (Behavior 2c; High).
98. Resolved in part: first-use initialization, time-change reset and first-free claim all cover five records of 0xE0 bytes from the same starting offset. The records are used as enemy thorn projectiles; the observed setup uses 10 as damage and 3 as maximum hit count. Static callsites show the shared initialization, per-frame refresh, and deferred spawn helper under all three Faerie Sisters controllers; the deferred helper is the sole direct allocator caller. No other enemy-controller caller or address-construction path was found in the scanned boot program, so the evidenced owner is the sister group. Remaining limit: dynamically computed references or runtime-loaded module code could not be excluded (Medium-High).
99. Resolved: the "approach" routines are the class 37 descent codes 8 to 10. The 8.0 is the ground probe's start height above her and the 0.05 gap is her height over the ground. The step homing and glide steps are Queen Fyora's slot steps (Behavior 19). The damped homing step is the active type-2 handler in class 37's four-slot dispatcher: it advances 0.75 units per tick toward the target aim point and, strictly below 1.5 units, deactivates the slot and sets controller state 5 with a 180-tick timer. This is separate from the generic projectile pool's kind-17 homing rule in Behavior 9. The pool and controller-slot homing paths are distinguished in Open question 36. High.

100. Resolved: the volley and its state machine (Behavior 18) belong to the Darkest Faerie (class 37; SPEC-0054).
101. Resolved: the volley loop runs for four shots (indices 0 to 3), not three.
102. Resolved: the volley spawns are spawns by key, not members of the member slot group of Behavior 14, so a later shot is not refused because an earlier one is alive; four reserved records give four Grunts (Behavior 18; High).
103. Resolved: the volley-start value 6 and per-tick value 14 select entries in the enemy animation table. The state machine also sets the motion record's no-gravity flag (+0x144) each tick, suppressing gravity and ground projection (SPEC-0040, Behaviors 14A and 14C; High). The designer-facing names or intended visual roles of the two animations are not identified.
104. Resolved: the reference height is the ground-probe result under the boss motion record current position. The probe ceiling is 8.0 units above the boss current height; the hover offset is then added to that result (Behavior 18; SPEC-0018, Ground probe). If no candidate is sampled, the probe returns its miss value, as specified in SPEC-0018.
105. Resolved: the state machine's hit count and sub-state are one stored value (Behavior 18; High). The hover offset (+15.0, +10.0, +5.0 for 0, 1, 2), the 180-tick timer and the three-hit exit all read it. Whether the sharing was meant cannot be told from the program, and the play result is the same either way, so the rewrite keeps one value. Test vector: hit at tick 10 (count 1, offset +10.0, timer 180); no hit for 180 ticks (count 0, offset +15.0); two hits (count 2, offset +5.0, timer 180); 180 ticks with no hit (count 1); the next hit gives count 2, not 3, so the state stays 7; one more hit without a gap gives 3 and state 9. At count 0 there is no timer.
106. Resolved: state 8 performs the slow timer-expiry descent and posts events 13 and 10 when within 0.05 of the ground; state 9 performs the faster three-hit fall and hands off to state 10 within 0.1 of the ground. State 10 waits for event 13, then posts event 10 and returns to state 1 (Behavior 18; High).
107. Resolved: the homing and flight steps of Behavior 19 are called by the per-tick slot dispatchers of the class 37 and class 38 controllers.
108. Resolved (High): after movement and the optional hero-hit test, the flight step ground-probes at the updated slot position with an 8.0-unit vertical ceiling lift. It ends the slot when slot Y is less than or equal to the returned ground height. The 8.0 is the probe lift, not a distance limit (Behavior 19; SPEC-0018, Ground probe).
109. Resolved in part (High for the documented class-37/class-38 slot setup): the class-37 and class-38 allocators copy the supplied character sample point into the slot; those allocation and frame-setup paths do not call the 8.0 ground-probe helper or derive a speed factor from it. Thus the earlier third-coordinate/speed-factor claim is unsupported for Behavior 19. The subsystem that prompted that claim, if any, remains unidentified; do not infer that it describes a ground probe (Behavior 19).
110. Resolved: these are two different timers. Behavior 20 draws 1 to 3 times the per-enemy speed factor; Behavior 12.8 is the live petpet's wait of 180 to 600 ticks in its state 1 (class 34), not an enemy cooldown (High).
111. Answered: the run-time word at 0x36DC64 (not in the file image) is the count of living Faerie Sisters. It is set to 3 when a sister controller starts, recounted each frame as the number of the three sister state words that are not the death state (9), and set to -1 at cleanup. It is not a difficulty level. The count is shared by the three sisters (Behavior 4). The recovered direct caller of the routine that writes 3 is the class-16 controller's alternate-loop branch (Behavior 4, Routine differences; High for the caller and write). The level-state predicate that selects this branch is not identified here.
112. Resolved: every integer draw of the random helper is closed, [lo, hi] with both ends included, and uses an unsigned remainder, so it never falls outside the range (Behavior 2; SPEC-0008). Float draws are [lo, hi). This covers Behaviors 19 and 20 (and the hero idle picker of SPEC-0033, formerly Behavior 12.9).
113. Resolved in part: the word at 0x2DAAA8 (0.5) is the per-tick factor of the step homing and glide steps (Behavior 2e) and of the slot homing hit step (Behavior 19). Behavior 12.10 is the class-38 slot type-2 handler described in Behavior 19, so its owner is known. The gate is current-world level-state id 2; in Fyora's Tower, a script pulse after the switch puzzle suppresses this handler's arrival spawn for one tick. The hero method returns the position value at attach point 7, the status display anchor documented in SPEC-0004, and this handler uses it as its destination. Why this slot targets that anchor remains open (Behavior 12.10; High for the point mapping, Low for the design purpose).
114. Resolved: Behavior 20 step 3 dots the unit direction from the sister to the active hero with the sister's facing. The melee dodge requires a dot strictly above 0.5 and horizontal distance strictly below 7.0; otherwise the routine proceeds to the projectile-dodge check (High).
115. Resolved in part: the probe direction comes from each active projectile slot's stored orientation vector and scale fields. Its precise design label is unknown. The sub-object callback with value 20 precedes the block-animation request after a hit; its operational effect remains unidentified (Behavior 21).
116. Resolved: the contact query uses a capsule with radius 6.0 and segment length 10.0; its segment extends 5.0 units either side of its center along the supplied axis. The prior description as a sphere was inaccurate (Behavior 21; High for the geometry parameters).
117. Answered (moved): SPEC-0019 holds the single full description of the Werelupe King's flame wave and its open items; Behavior 22 here is a cross-reference.
118. Resolved (High): the Werelupe King flame-wave trigger uses the state gate listed in SPEC-0019 Attack selection: it may start from idle, hit flinch, walk, hop back, orbit right or orbit left, and is closed in the attack, block, jump, roll, catch, triumph, phase-entry and death states. Behavior 20 sets the acquired flag only in main states 0, 1 or 10 and above; states 2–8 fail the gate, and state 9 returns before the routine does any work. The accepted state ranges follow the shared predicate and its caller; no design label is inferred.
119. Resolved in part: event 15 clears its global word and record words at +0x28 and +0x04, but leaves the hit count and volley timer unchanged. In the reviewed controller, event 14 is the only reset for those two fields; state 7 changes/decrements them, and the state-8/9 handlers do not clear them. This is scoped to the controller call graph (Behavior 18; High).
120. Resolved: the 1.5 detonation gate and the second-form 2.0 end gate use straight-line separation measured before the tick movement; both comparisons are strict (Behavior 19; High).
121. Resolved: the four tests read different animation event bits (0x02, 0x04, 0x08, 0x10) of the enemy's current clip and pick grab point index 1 to 4 (0 when none is set). States 3 and 4 are reset to 1 when agro outer sub-state 2 is entered and to 0 when the next attack slot is chosen (Behavior 23; High).
122. Resolved: Behavior 27 is the enemy status tick of the common enemies; its eight tests are the status-active tests of statuses 0 to 7. The handler run for an expired timer is the status display's expire check (SPEC-0034). The two class tests A and B mean "moving" and "attacking" (Medium).
123. Resolved: Behavior 24 is the common enemies' alarm-group check (SPEC-0038 Behavior 3), gated by the spawn record's alarm group byte; it is not a contact reaction. Behavior 10.3 is a different routine, the Ixi Chief's incoming-hit check (Open question 52).
124. Resolved: the state-13 entry gate is the class-6 townsfolk greeting check. Its 6.0 threshold compares horizontal distance to the hero, not squared distance; vertical separation is ignored (SPEC-0056; High).
125. Resolved: the facing gate is a horizontal front-half check: the character's horizontal facing dotted with the horizontal direction to the hero must be strictly positive. Vertical components do not participate (SPEC-0056; High).
126. Resolved: the mode-13 handler reads +0xAB0 as a gate for its timed animation sequence. Zero runs the counter and animation-30 branch; nonzero skips it. The independent distance/facing exit still runs in either case. This describes the field's operational use, not the design meaning of the character-class predicate.
127. Resolved in part: the setup initializes entries in a twelve-slot transient sprite array; its two known callers mark every fifth entry, and the derived-transform helper and per-entry motion updater are identified. The third input is stored at +0x0C, but both callers pass 0 and no semantic consumer of that field is established. Neither the initializer nor the distance-gated refresh has a direct caller in the boot program call-reference map; indirect dispatch and runtime-loaded callers remain unverified. The owning visual effect and the field meaning remain unknown.
128. Resolved: Miamouse state 3 is the released mouse's landing state. The close-range test is the ground gap below 0.25 (0 while the floor flag is set), which switches clip 13 to the landing clip 19 (Behavior 28; High).
129. Resolved: the facing test is a dot product, dot(Miamouse forward, unit direction to the hero) > cos(1.0 rad) = 0.5403; it applies only to a sneaking hero, and a pass makes the Miamouse flee (Behavior 28; High).
130. Resolved: the countdown runs first on every state-1 tick, in every sub-state, including the emote. A multiple of 90 reached during the emote is skipped. A fresh pause is tested first at its drawn value minus 1, so no pause emotes at once unless it drew 181 (Behavior 28; High).
131. Resolved: the "group cue" is the delivery step. It adds 1 to the cage count and moves one level-state progress bit up (the lowest state k of 0 to 4 that is on goes off, and state k + 1 comes on). There is no cue; world flags 0 to 4 are the current world's level states 0 to 4 (Behavior 28; High).
132. Resolved: the pending hit flag (+0xABC) is set by the shared take-hit of every region character (Behavior 34; High).
133. Resolved: a repeated request for the current id does not restart the clip (Behavior 29; High).
134. Resolved: the four-member controller's eleven codes are those of SPEC-0054. Loop A's result-1 handler and loop B's states 0 to 5 are specified in Behavior 7 (High). The 12.0 and 9.0 velocities are the hero knockback of loops A and B, and the -5.0 is the push-out factor of loop A; none belongs to a member slot (High).
135. Resolved: game state 364 is set by the init of script pack 1432 and level state 18 by the init of script pack 1174 (SPEC-0054). The unnamed type-56 placement in resource 1263 selects the same variant-0 monster row as the named Darkest Faerie placements, supporting its identity as that game-defined character type. Its encounter role remains unknown.
136. Resolved: the slot handlers, the slot tests and the release helper of the Queen Fyora controller are specified in Behaviors 19 and 30 (types 0 to 3, the 1.5 and 2.0 tests, the sample-point cast). 4007 is the effect played on the slot that state 2 relaunches (Behavior 11, rule 4); what it shows is not known.
137. Resolved: class 18 is the Juppie boss controller, and its chooser is the attack decision of SPEC-0015 (Attack decision (cadence)): a free-state test (state 0 or 1) and an active-stage test, a stage-dependent cooldown (1.5 to 3.5 s, 0.75 to 2.0 s, 0.25 to 1.0 s), then the head slam at distance above 10 and below 50 with a 50 percent roll, then the spit (spit flag set, distance below 100, 50 percent roll, count 3 to 5). The cooldown runs down on every pass in every state. The class-18 rows are the Juppie Monster records (rows 7 and 8, hit points 190 and 400; resources 1218 and 1219); the Act 1 placement selects row 7 and the Act 3 variant selects row 8, so the identity is High.
138. Answered: class 13 is Malice (row 125), class 15 is Spite (row 126), and class 16 is Vanity (row 127), matching their names in the placement data (High).
139. Resolved: the named records whose monster row has no class (543 placements, census of Layout section 4a; for example Illusen the Earth Faerie, Seradar, Master Torak and Prince Tourin) take class 6 from their companion row and run as townsfolk (Layout section 3; SPEC-0056). Class 6 has no attack mode, so none of them fights as an enemy (High).
140. Resolved in part: all nine unnamed records of monster type 102 select variant 0 and monster-table row 246 (5 HP, no behavior class, and no configured attack type); all use character key 805 and have the summon field clear. Resource 536 is pet package kind 2, which is not in the activation set; resources 1281 and 1302 are in regions loaded only by unreached module 1259. The type-102 record in resource 1404 is at the same position as an Evil Harris record. The resource-1301 type-102 placement also has a conditional slot-load request from module 1300 when flags 94, 93 and 92 are clear and flag 397 is clear; this request occurs in both flag-222 branches. It establishes a resource-load route only, not activation, visibility or purpose. The roles of all nine records remain unknown. **[High for the row, key, and placement data; Medium-High for the resource-load boundary; role unknown]**
141. Resolved: the older count of 541 is a named-only class-set bucket: it skips empty names and groups each nonempty name by the set of classes used by its selected rows. The two Gelert Assassin records that select the no-class row are grouped with the name's other six class-1 records, so they are outside that bucket; record-level counting gives 543 named no-class records. Adding the 123 unnamed no-class records gives 666 total. The separate class-17 increase from 2,171 to 2,197 is the 26 unnamed class-17 records. Layout section 4a gives the record-level counts (High).
142. Resolved: state 14 of the common enemies is the retreat-home-and-hide state, entered by the distance-hide gate of the agro machine (SPEC-0038 Behaviors 24 and 38).
143. Resolved: death is not a class-17 state. The shared hit-point subtract runs the death hook of the region-character class at 0 HP for every class (base state 16 except classes 1, 11 and 32), and the shared dead state removes the character (Behavior 34; SPEC-0038 Behaviors 34 and 36; High).
144. Resolved: the attack sequencer of agro outer sub-state 2 and the counter-attack choice are specified in SPEC-0038 Behaviors 13 and 14; the map-search spell sub-types are SPEC-0038 Behavior 43.
145. Resolved (High): the class-17 hero-target routines are the grab-and-throw strike of attack slots with flag 0x08 (SPEC-0038 Behavior 15; Behavior 23 here). The hero side is the held state (base state 53), which stays on while any enemy is in its strike state with a grab slot, and the carry placement is SPEC-0005 (Attach points and carrying) (SPEC-0038 open question 31).
146. Resolved: the 120-to-210 and 400-to-800 ranges are closed integer draws in Behaviors 28 and 29; their bounds are fixed constants. In Behavior 28 the release distance 1.0 is the only data word, while the 10.0 notice range and other distances are fixed constants. The 30-to-45-tick recovery draw belongs to Queen Fyora's state 3 (Behavior 30), not Target Dummy (Behavior 29); its bounds are 0.5 and 0.75 seconds multiplied by the 60-tick rate, so they are fixed program constants, not a data word (High).
147. Resolved: the aim is the horizontal unit vector from Queen Fyora toward the class-37 object (Behavior 30; High).
148. Resolved: the flee entry sets state 2, sub-state 0 with a radius-30 far centroid pick from the hero; sub-state 0 then chooses the next point by the hero's distance (Behavior 28; High).
149. Resolved: the counter advances exactly once per counted tick; each branch adds its own single step and no branch adds two (Behavior 29; High).
150. Resolved: the Queen Fyora controller enters only states 0, 2 and 3 (Behavior 30; High).
151. Resolved: the Mortog launch value is the length of the move-goal offset, not a speed. The goal is that offset alone, k x unit(P - A), with no position term (Behavior 3; High).
152. Resolved: catching does not use the Miamouse's main state word. A held Miamouse is in base state 33; main states 3 (landing) and 4 (delivered) are entered only after release (Behavior 28; High).
153. Resolved: the cage zone is the cage navmesh's inside test on x and z. There are no cage positions and no fill animations; each delivery only adds to the count and shifts the level-state progress bit (Behavior 28; High).
154. Resolved in part: none of the four request sources of Behavior 1, step 4 directly names class 9 or 10, and the Cloud Beast has no type-1 placement on disc. The Minion Wizard kind-6 code can re-pick an injured type-0 character to row 1/variant 1; a later default-class request would then select class 9. The only same-pack type-0/Minion-Wizard pair in the census is pack 1259, an unreachable cut variant of 909. The other type-0 placement is in pack 989; its recorded neighbors and module-loaded character region contain no Minion Wizard. The recovered normal placement/module/transition data therefore provide no Wizard hit on a type-0 target, so this conditional code path does not establish a shipped class-9 route. The later default request remains mechanically possible after a hypothetical mutation (SPEC-0010, Placement and reachability; Run-time variant re-pick; SPEC-0038 Behavior 43).
155. Answered (Behavior 11a): the stolen amount is spawned at once as coins at the hero's position. The enemy's carried Neopoints do not change. The amount taken is limited by the wallet.
156. Answered (Behavior 11a): the amount draw is inclusive at both ends. When the minimum equals the maximum no draw is made, and when the minimum is above the maximum the two bounds are swapped.
157. Resolved: the pre-steal impact gate is specified in Behavior 11a. It returns before steal processing unless the normal strike result is positive or, for the slot path with bit 0x08, the special impact-success marker is set (sequence state 4 and shared gate query 0). For item selectors 1 to 3, Behavior 11b now specifies the candidate ids, stored check order, common inventory eligibility test, and uniform selection among retained candidates. The carried-Neopoint increase is the item's catalogue price; the carry lasts 180 ticks, then the carry flag clears and its display stops. The item-id field is not cleared on expiry. Carried Neopoints are paid out at death as specified in SPEC-0038, Behavior 34. Resolved: the normal strike result is the damage that the enemy's damage routine returns for the selected hero, and that routine runs only when (1) the slot's attack-active event bits are on (Behavior 10.2), (2) the hero is not yet in the swing's ten-slot struck list, and (3) the weapon test of Behavior 10.2 finds an enabled weapon volume overlapping a hero body capsule. A guarding hero hit by a blockable slot (slot flag 0x01 clear) with dot(hero facing, enemy facing) at most -0.5 takes the blocked call (blocked flag 1; the hero's block reaction plays and the struck list gains the hero); a guarding hero in guard sub-states 6 or 7 takes no damage call at all, so the result is 0 and the steal does not run (High for the test order; Medium for the names of the guard sub-states). A result of 0 or less ends the impact step before the steal and before the status roll. For the slot path with bit 0x08 the marker is set when the sequence state is 4 and the hero is not yet in the struck list, and the hero is added to the list. Selector values below 0 or above 3 pick item id -1 and the steal ends with no effect (High). Test vector: a Thief swing (selector 0, chance 50) whose weapon volume misses every hero capsule returns a result of 0, so no chance roll is made and no coins spawn; the same swing on a hero already in the struck list also makes no roll.
158. Resolved: the two routes that pass item 172 are the class-37 member summon and the Darkest Faerie volley (Behavior 31, sources 1 and 2), so every Minion Warrior and Minion Grunt she summons drops a Nova Mote at its first death tick. Each spawns only when the block holds an unbound reserved record of its key (High).
159. Resolved: a kind-6 projectile acts on its target (the caster's target id stored at launch), and the attack that builds it is the Minion Wizard's spell (row 213, map search type 8; SPEC-0038 Behavior 43). A re-pick on a type-0 character followed by a later spawn-default request would read row 1 (class 9). The only same-pack type-0/Wizard pair is in unreachable pack 1259; the reachable type-0 placement in pack 989 has no Wizard in its mapped resident packs or module-loaded character region. No normal reachable hit-and-reselection sequence is supplied by the recovered level data (High for these data boundaries; hypothetical re-selection behavior remains as described above).
160. Resolved: the four volley spawns are spawns by key of the row's slot-1 key, made by the volley start of Behavior 18 (High).
161. Resolved: row 221's slot-3 key (619, Flower Monster) is read by the lobbed summon: slot 3 is a missile slot with flag 0x02, so each lob spawns one Flower Monster where it lands, up to five live (Behavior 31). Row 211's slot 0 summons its Minion Grunt the same way.
162. Resolved: the type-2 orb's summon uses the class-37 object's row, not Queen Fyora's. It spawns that row's slot-0 key (2864 on row 128; row 129 has none) at the hero's position while level state 2 is clear (Behavior 19; High for the rule, Medium for the row the duel uses).
163. Resolved (High): the boss pool submitters are mapped. Kind 10 with visual kind 16 is the Faerie Sisters' energy ball; kinds 11 to 16 with visual kinds 17 to 22 are their slow, poison and curse missiles, Mote-eater, Leach and thorn (damage, status, lifetime and test vectors in SPEC-0011); kind 9 with visual kind 15 is the Plague Serpent's belch cloud (SPEC-0016); kind 0 is the Gelert Assassin's dagger and shadow dagger (SPEC-0013); kind 1 with visual kind 23 is Juppie's spit (SPEC-0015). Resolved for the rest: the kind 9 rule is in SPEC-0016 section 10, and the extra effects of kinds 14 and 15 are in Behavior 9 (High).
164. Resolved (High for the sizes and the base; the end-cap result below the base stays Low): the projectile hit test reads the character's motion-record radius (+0x08) and height (+0x0C), with the motion-record position (the feet) as the base point and the up vector as the axis. The sizes are 1.175 and 3.5 for the heroes and row 0x28 and 0x2C for region characters (Layout section 4d). Test vectors: hero feet at (0, 0, 0), projectile radius 0.5 at (1.6, 1.0, 0): hit (height offset -0.75 from the mid-height 1.75; side distance 1.6 below 1.675, depth 0.075); at (1.7, 1.0, 0): miss.
165. Resolved (High): the exempt states are Roberta's attack sub-states 6 and 7 (the spins), 8 (the leap) and 13 (its landing), read from her attack sub-state word; the predicate that keeps status 0 from being applied is the hero's own stun being active; the block query is the hero's guard query (SPEC-0036, Hit detection), with the dot rule of step 1.
166. Resolved: the status value of the projectile status roll (Behavior 13) is in whole seconds: the stored value is trunc(drawn row value / 60), and the target's apply method multiplies it by 60 to get the timer in ticks (Behavior 27.1). A drawn value below 60 therefore gives 0 s, which sets the timer to 0 and leaves the status inactive (High).
167. Resolved for class-17 target selection and conditional effect: on contact, the map-search type-2 spell sets its selected target's reanimation countdown to -1. The type-2 picker draws from the active region-character list, excludes candidates playing animation 4, and requires a candidate to have current HP below maximum, lie within the caster row's map-search distance, and pass the sight-line checks. It does not reject a candidate for class state 16 or a positive reanimation countdown (SPEC-0038 Behavior 43). A class-17 enemy already in state 16 with an active countdown and its death clip can therefore be selected if it remains in the list and passes those other tests; the next state step skips the remaining timed-death phase and enters the rise checks (SPEC-0038 Behavior 35). Old Mine (pack 906) and Ancient Castle Spires (pack 1091) have shipped placements of row-225 Skeleton Wizards, which use map-search type 2, alongside one or more of the reanimating skeleton rows 224, 226 and 227. This establishes shipped encounter co-placement, not that exact positions, sight lines, cooldowns and attack event align to produce a hit in ordinary play. The clear-area and rise-animation gates still apply; a row without a rise animation heals to full and returns to agro. Effects on heroes and other classes remain unknown (High for picker predicates and conditional class-17 effect; Medium for encounter co-placement; Low for actual in-play spell contact).
168. Resolved: ghost fade after a reveal (Behavior 33). When a reveal ends, c = 0 and L = 0, so the attacking branch forms 0 / 0, which the console's divide returns as the largest positive float; the resulting opacity is hugely negative (hi above lo) and the alpha clamps to 0. The ghost is invisible but targetable while it attacks, until the next phase change; it is not opaque. See the edge case "Ghost fade after a reveal ends" and the test vector. **[Medium-High]**
169. Resolved: a hit during the ghost phase keeps the existing fade counter and length. On the next ghost update, both a positive fade counter and the reveal countdown decrease by 1 before opacity is calculated with the fade-in expression, hi - (c / L)(hi - lo). For the Restless Spirit row (lo 0, hi 0.5), L 60 and c 45 after the last hidden update, opacity changes from 0.375 (alpha 48) to 0.133333… (alpha 17) on reveal; the ghost is targetable during this low-opacity reveal. Subsequent ticks raise opacity by 0.5 / 60 while c remains positive. This specifies the state and alpha values, not perceived rendering (Behavior 33; High).
170. Resolved (High for the route; Medium-High for the Ixi Chief's double path): one method table serves every region character and holds both the enemy-kind hittable query and the shared take-hit. The hero swing tests every hittable entry of the enemy list and sends each connecting hit to that shared take-hit, so it reaches the bosses of classes 1, 11 and 32 like any other region character (doubling on the first damaging hit, facing, alert; Behavior 34) whenever they pass the hittable query. The Ixi Chief's loop clears its invulnerable word (+0x7F0) every tick and sets it again only in its states 0, 15 and 16, so it is hittable in its other states; its own incoming-hit check (Behavior 10.3) subtracts from the same health word. A connecting swing on the Ixi Chief in another state therefore passes through both paths in the same pass.
171. Resolved (Medium-High): base state 20 is the shared "guarding" state of every non-hero character. The character's guard query is true in base state 20, and for a common enemy also while its agro outer sub-state is 7 or 8 (SPEC-0038 Behavior 44: base state 20 is written when a block starts, and written back to 1 when the defence ends or when the outer sub-state is neither 7 nor 8). The users of that query are the hero's swing and shot tests (SPEC-0036, Hit detection: a guarding target is tested even when it is not hittable, and a swing that meets it is blocked when the facing dot is below the row's Blocking Angle), the boss attack routines (they ask it of the hero, not of themselves) and the pool-shot hit routines. The common-character update treats base states 20 and 1 alike. For the Draik this decides everything: its Blocking Angle is set to 2.0 at setup, and no facing dot reaches 2.0, so while the Draik is in base state 20 every hero swing against it is blocked (no damage, block effect) whatever the facing, and quick shots meet it as a guarding target. In base state 1 the guard is off, so the swing is not blocked and goes on to the hit filter and the take-hit like a hit on any character: that is during its states 1, 3, 26 and 27 (hit reaction of stages 1 and 3, and the two stunned states) and during the whole of stage 5 (Section 6.10). So the Draik guards in every other state of stages 1 and 3 and drops the guard only when stunned or reacting. The Draik's own hit registration (Section 6.4) is a separate route that counts hits in its stunned states. Test vector: Draik at stage 3 in state 0 (base state 20): a Tor swing whose capsule test passes is blocked with any facing dot, from -1.0 to 1.0; the same swing while the Draik is in state 27 (base state 1): the guard rule does not block it.
172. Resolved: all 53 distinct listed SetAI_Agro instructions in the 16 modules target valid entities whose default behavior class is 17 (common enemies; SPEC-0038). This maps the targets, not whether every gated or debug call executes in normal play (High).
173. Resolved in part: each sister loop advances the active Mote-eater group through state 9 and once on the terminal state-9 pass; subsequent passes stop after the shared death path sets base state 16. The drain rate follows the loops still being stepped, not the living-sister counter. In the main arena, action 26 selects a 50-frame clip with S = 2; at baseline scales its duration from the first playback advance is ceil(196 / (2 − f)) ticks, where f is current HP divided by maximum HP (SPEC-0011, Mote-eater drain). The exact runtime duration depends on HP after the lethal hit and the playback-start tick order (Medium-High).
174. Pool trail field (setup record +0x50): Resolved (High). Both Gelert dagger setup paths supply model-part entry 1 of the boss's part table, and the setup submitter copies that word to live pool-slot +0x04. It is a model part, not an effect id: the pool's draw pass submits that part at the slot's position for each live slot whose visual kind is below 5 and whose trail word is non-zero (SPEC-0013, Open question 37). The Gelert fight region's set does hold a mesh child with code 11 (3,840 bytes), so the word is non-zero for these daggers. The part is shown only while a dagger slot is live; it is not a model that stays attached to the boss.
175. Resolved (R-0013): a Whinny takes the first ground mesh holding its position once, at start (High). Entering lead does not search: the Whinny's current navmesh is set to the pen's entrance mesh, and the lead walk runs on that mesh, which holds the pen point (High). So a lead run never needs a path outside the start mesh (Behavior 5).
176. Resolved (R-0016): the world-test segment of a kind step is lateral, not along the velocity: its ends are the position minus and plus 0.5 times the unit cross product of the normalised vector at +0x30 with the vector at +0x50 (Behavior 9, step 2; High for the routine, Medium-High for what the two vectors mean). The caller that fills them was not read; the endpoints do not depend on it.



















