# SPEC-0026: Game flow, attract mode and movies

- Answers: proactive (none requested); R-0023 (answered: font 0, spacing -2, baseline origin and vertex alpha, see Credits); R-0141 (what the movie hand-off's reload of W rebuilds and what the player sees; proactive (verify pass): the draw gap near a seam); R-0142 (what follows movie 85; the epilogue triggers and the save point); R-0190 (kills stored only on a release and on save; which sound loops the movie hand-off ends for good; the music restart when the current world is unchanged); R-0225 (the shop copies beside 1427); proactive (open question sweep, audio and flow); proactive (spec consistency pass); proactive (open question sweep, round 9)
- Status: Draft
- Disc build: SCUS-97367 v1.00 (same fingerprint as SPEC-0001). Movie table count and names read from the boot program's data. Start-up table count (60) and guard word read from the boot program's data.
- Confidence: per section. Start-up table (60 entries, the static-constructor list): High for its count, walk order, guard and stage meaning (stage 1 builds, stage 0 is a teardown that never runs); High that the walk runs before the start-up body (frame setup, game-globals setup, start-up loaders); High for the start-up values of the entries named under "What each entry does"; Medium for the owner of most entries, Low where marked. The unused 19-entry list: High that this build does not run it. Entering the area (enter-interior job: plane/entry landing selection, camera from the resulting hero pose, state 1, five consecutive idle loader passes and 0.3 s fade-in): High for the placement and entry-index rules and the fade/wait lengths. Loading screen and transition fades: High for the calls and frame counts; Medium for the one-tick float detail and for the missing fade after a movie return. Hero-swap task (formerly listed here as an area transition job; specified in SPEC-0053): High that it is the swap task started by the switch request and not part of an area change; its steps and confidence are those of SPEC-0053 section 3.4. Cut-scene start (two variants, which are the two hero-swap sequences of SPEC-0053): High for the fade colour, lengths and target; Medium for the order; Low for the meanings of the events. Main loop order and game-state rules: High. Tick rate and timer conversion: High. Status timer activity: High (names Medium). Boot order (the boot entry and the ten steps of the main boot function): High, including the status call of the spin (a field-sync wait) and the last initialiser (one early run of the atmosphere-zone pass). Boot initialiser (the game-globals builder of boot step 5): High for its writes and the subsystems it starts; Medium for the role of the draw-context record. End-of-pass apply (formerly listed as frame update sub-steps): High for the order, Medium for each step's role. Attract and title timers: High. Visible boot order (publisher page, developer page hold, card check, logo page, legal page, two movies, title): High for the order and the two boot pictures; Medium-High that nothing replaces the developer page during the hold. Controller check, title input and the lead-in page mechanism: High; Medium that the lead-in pages were cut. Credits roll: High for the rule and its timing; High for the setter chain from the finale to the credits; Medium that the player reaches the two epilogue triggers. Movie table and queue: High. Movie play sites: High for the 83 play calls, their fixed indices and the attract pair; Medium-High for which packs are entered. World buffers and travel: High for the three buffers, the pack kinds and the start loads; Medium-High for the travel rules and the reach count (174 of 191 packs). Movie playback and handoff: High (Medium for the frame steps inside the player). Movie skip rule: High that every movie has its skip check enabled; High for the buttons, the threshold of 5, the arming, the reset-at-start rule, the latch timing and the dead second counter; High for the once-per-presented-frame cadence and for the end of play after a skip. Dialogue command binding (the six start-up bindings once listed as event registrations): High. Movie groups, frame rates and audio rates per group: High for the values, Medium for the frame-time unit (SPEC-0008). Movie frame path: Medium-High. Area loading: High for the release and load rules (steps 3 and 4); Medium for the finish step order. World memory budget and level setup: Medium (sizes), Low (meaning of the slot capacities), High that the level setup runs once at boot inside the start-up routine. Teleport landing: Medium for the plane-index and height rules, Low for the rest. What persists across an area change: High for heroes, pet, world items, level scripts and world memory; Medium or Medium-High for the other rows; Medium for the meaning of hero word +0xF7C (the door-use latch). Game over flag and its effects: High (code); the front-end state it hands over to: High (the game-over page, front-end record 23). Respawn: no save-point respawn exists; the death and placement paths are in SPEC-0055 (High). Pause (code 9 through the play-active word) and the lists of what runs and what freezes: High; the one-pass lag: Medium-High. Act intros: High for the act 2 and act 3 intro gates (see Movie play sites). World tick order (main loop step 8.1): High for the order; none of the six world sub-tables is ever filled in play (High). Bush instance step (once listed here as the timed transition driver): High that it belongs to the type-16 foliage system; its rules are in SPEC-0060.

## Summary

The game runs in one fixed-rate loop (SPEC-0007). Each pass reads the controllers once, updates the game state, draws the frame, waits for the display, and advances the clock. The loop has no separate mode switch. What is active comes from three things: a game-state code (0 to 9) recomputed every pass, a world-enable flag with the loaded-world slots, and a front-end screen machine (title, menus, options, credits, load and save). At boot, a table of 60 start-up entries in the boot program's data is walked once, from the last entry down to the first. The table is the program's static-constructor list: each entry builds one program module's static objects at stage 1, and entry 2 writes the live option defaults. The main boot function then initialises the four save slots, sets the save-slot pointer, runs the frame setup and the game-globals setup, binds the six dialogue commands, and runs the start-up routine (sound, pools, start-up loaders, game object, vibration sync, world pool setup) before the frame loop (see "Boot order"). The owner of each entry is under "Start-up table walk". After the publisher page (shown while the start-up loaders run) and a 3-second developer page, the front end shows the memory-card check, the Neopets logo page and the legal page; the attract page then queues two movies (an act 1 opening, then the title movie) and passes to the title screen. A 30-second idle period on the title returns to the logo page and repeats that loop. The finale's epilogue ends in a credits roll that returns to the same boot loop. Movies are a 92-entry name table on the disc, played through a 5-entry queue, and each one blocks the loop until it ends. Only two things queue a movie: a level script's play command, always with a fixed index, and the attract screen. The files sit in six act folders (SPEC-0008). Most play at about 30 fps; the four act intros and one ACT2 movie play at 29.97 fps. Timed actions inside a world are counted in game ticks: a duration in seconds becomes a whole number of ticks by multiplying by 60 and dropping the fraction. Each character also carries 14 status-effect timers that are active while at least one tick remains and are saved with the game (see "Timers and ticks"). Areas live in three world buffers (transition, exterior and interior), one of them current; every pack has a fixed kind that decides its buffer, and travel is mostly seamless streaming of the neighbour into the other buffer, with doors and script loads for the rest (see "World buffers and travel"). A new game and every start point load packs 691 and 909. The clean side can rebuild this as a fixed-tick update with a state enum, a timer-driven attract sequence and a synchronous movie player that reads the same table.

## Where it lives on the disc

- Movie name table: 92 pointers, each to a NUL-terminated name string, stored in the boot program's data at ELF virtual address 0x2EB458 (4 bytes each, little-endian). Two more pointers give the path prefix (at 0x2EB450) and the file suffix (at 0x2EB454). Read the table at runtime; do not copy it.
- Movie files: `MOVIES/<group>/<name>` on the ISO (SPEC-0008). Each table name is relative to the MOVIES folder, for example `ACT1\100A.BIK`. The original builds the disc path from the table name with the prefix (the disc root's MOVIES folder) and the suffix `;1` (an ISO9660 version tag). The clean side drops the suffix and reads the file by its ISO path.
- Title idle timeout: a 32-bit integer in the boot program's data at 0x2DCE38, value 30 (seconds).
- Credits tuning (front end): 0x2DCF84 (row spacing, float 24.0), 0x2DCF88 (slow scroll, float 1.0), 0x2DCF8C (fast scroll, float 1.5), 0x2DCFA0 (speed ramp, float 1/512 per pass). Integers at 0x2DCF98 and 0x2DCF9C (fade distance, 32), 0x2DCF90 (reset threshold, 20, used as a negative value) and 0x2DCF94 (start offset, 1, added to the display height).
- Tick rate: a 2 by 2 table of four floats in the boot program's data at 0x30C598 (60, 30, 50, 25). Rows are video modes and columns are update divisors (Medium for this interpretation). This build always uses the first entry (60 updates per second). SPEC-0007 has the full rule. Clean code should use the constant 60 and does not need to read the table. The tick rate (60 per second) and dt = 1/60 s come from SPEC-0007.
- The other timing values below are constants inside the code, not data words, and are stated with their values.
- Options block (live): 32 bytes of runtime memory, not on disc. It is given its defaults during start-up (see Options). The option blocks inside the save slots are zero after start-up.
- Start-up table: 60 four-byte entries in the boot program's data at ELF virtual address 0x2FB828, with a count word of 0x3C at 0x2FB824, and a guard word at 0x393240 (see Layout).

## Layout

### Start-up table (60 entries)

| Item | Value |
| --- | --- |
| Entries | 60 (index 0 to 59), four-byte references to routines, in table order |
| Count | One word, value 0x3C, at ELF virtual address 0x2FB824 |
| Table | Starts at ELF virtual address 0x2FB828. Each entry refers to a routine; the entries are not copied here |
| Walk | From index 59 down to index 0. Each entry is called once. The walk uses the count word; only a count word of -1 would make it count the entries up to a zero entry, which is not the case on this disc |
| Guard | One-shot flag word in boot RAM at address 0x393240 (zero at boot). The walk sets it to 1 and runs only when it is zero |
| Arguments | Entries 0 to 58 call their module's static initialiser with stage 1 (build) and a second argument 0xFFFF. Index 59 is a tail call with its own arguments: the registration of the C++ exception-unwind tables, a library step |
| Entry 2 | Stage-1 setup routine. It runs its subsystem setups and then writes the live option defaults (see Options) |

The table and its guard are read at boot (High). No cursor is used.

### Unused 19-entry list

This build does not run this list (see Start-up table walk). It is not a boot step and the clean side does not read it. No movie is started from it. It holds 19 four-byte entries followed by one zero word, at ELF virtual address 0x2FB920 (the zero word is at 0x2FB96C). No reference to its walker was found in the boot program (Medium: a computed jump is not excluded).

### Movie name table

| Item | Value |
| --- | --- |
| Entries | 92 (index 0 to 91) |
| Entry type | 4-byte pointer to a NUL-terminated ASCII string |
| Distinct files | 75. Some names appear twice in the table (see Edge cases) |

Movie names (High). A movie has no display name. Its name is its table string, which is the file's folder and stem plus the suffix ".BIK" (for example `ACT1\103A.BIK`); with the suffix dropped this gives the name, written here with a forward slash: movie 4 is ACT1/103A, movie 78 is ACT4/416A2 and movie 86 is FRONTEND/TITLE. Two indices that hold the same string have the same name (for example 16 and 17, both ACT1/115A1). Read the strings at run time; do not copy them.

Index ranges (from the table):

| Range | Group | Entries | Distinct files |
| --- | --- | --- | --- |
| 0 to 17 | ACT1 | 18 | 17 |
| 18 to 29 | ACT2 | 12 | 11 |
| 30 to 56 | ACT3 | 27 | 24 |
| 57 to 85 | ACT4 | 29 | 17 |
| 86 | FRONTEND title | 1 | 1 |
| 87 | FRONTEND attract loop | 1 | 1 |
| 88 to 91 | ACTINTRO, acts 1 to 4 | 4 | 4 |

Movie files by group (frame and audio timing as stored in each file's header, SPEC-0008):

| Folder | Table indices | Files | Frame rate | Audio rate |
| --- | --- | --- | --- | --- |
| FRONTEND | 86 (title), 87 (attract loop) | 2 | 30 fps | Title 44,100 Hz; attract loop 48,000 Hz |
| ACTINTRO | 88 to 91 | 4 | 29.97 fps | 44,100 Hz |
| ACT1 | 0 to 17 | 17 | 30 fps | 44,100 Hz |
| ACT2 | 18 to 29 | 11 | 30 fps, except ACT2 210A at 29.97 fps | 44,100 Hz, except ACT2 210A at 48,000 Hz |
| ACT3 | 30 to 56 | 24 | 30 fps | 44,100 Hz |
| ACT4 | 57 to 85 | 17 | 30 fps | 44,100 Hz |

The table shows which act each movie belongs to. Which level pack plays each movie is in "Movie play sites" (Behavior).

Named indices used by the game flow:

| Index | Movie | Used by |
| --- | --- | --- |
| 0 | First ACT1 movie | Attract sequence, first movie. No level script plays it |
| 78 | ACT4, final fight entry | Final fight (pack 1432), entry sequence (SPEC-0052, SPEC-0054) |
| 80 | ACT4, timeout ending | Final fight (pack 1432), timeout ending, before game over (SPEC-0055) |
| 85 | ACT4, finale | Final fight (pack 1432), finale after the win (SPEC-0052) |
| 86 | Title movie (FRONTEND title) | Attract sequence, second movie. No level script plays it |
| 87 | Attract loop (FRONTEND attract) | Never played: no level call and not queued by attract (High) |
| 88 to 91 | Act intros, acts 1 to 4 | Level scripts: 88 in pack 909, 89 in pack 1174, 90 in pack 1327, 91 in pack 1428 (Medium; see Movie play sites) |

### Movie queue

| Field | Size | Meaning |
| --- | --- | --- |
| Ring slots | 5 | Movie indices waiting to play, played in the order queued |
| Played words | one u32 per movie index (92 words, 368 bytes) | Set to 1 when the movie starts to play and kept; read by HasMoviePlayed and by the skip-if-playing rule (Medium for the second reader); saved and restored as the save's engine block (SPEC-0025) |
| No-skip word | u32 | Cleared by every queue add and set by nothing, so the skip check of every movie is enabled |

### Bush instance record (moved)

The 0xD0-byte record once listed here as a timed transition record is a bush instance of the type-16 foliage system. Its layout is in SPEC-0060 (Foliage patches (type 16), Bush instance).

### Game-state code

One small integer, 0 to 9, held in the game-state object and recomputed every pass (see Behavior).

### Timers and counters

Each delayed-action timer is one float of seconds, with the value -1.0 meaning off. Six-pass counters are integers. Character status timers are integer tick counts (see "Timers and ticks").

## Behavior

### Boot order (once, before the first pass)

Confidence: High for the order; Medium for the boot entry's setup details.

The boot entry first clears the boot RAM range (data addresses 0x32B280 to 0x43C78C), so every static flag and pointer in that range starts at zero. It then runs its system calls, memory setup and cache flush, and calls the main boot function, which does not return in normal play. The main boot function, in order:

1. Walks the start-up table (see "Start-up table walk"). Entry 2 writes the live option defaults during this step.
2. Initialises the four save slots (SPEC-0025), once (a one-shot word guards it). Each slot's option block is first given the defaults, then each slot is zeroed, then the empty-slot fields are written (see Options). The live block keeps the defaults.
3. Sets the save-slot pointer (data address 0x35C8EC) to the save-slot array (data address 0x35CA20, four records of 0x3100 bytes). The save, load and slot-picker code reaches the slots through this pointer. It is not the live option pointer.
4. Runs the frame setup: the rate words, the pass counter and the game time (see Start-up body).
5. Builds the game globals (see Start-up body), including the clock pointer GO+0x08, the model registry GO+0x38 and the live option pointer (data address 0x35C8C8) to the live options block (data address 0x3396C0).
6. Binds the six dialogue commands (see Dialogue command binding).
7. Runs the start-up routine. In order: the sound set-up; the display and pad set-up; the memory pools; the start-up loaders (text database, items, menu sounds, music block, the clock init, the effect and event loaders, the 180-vblank hold, the front-end set-up and the global task system); the game object; the vibration sync (SPEC-0033; with the default Vibration value the rumble gate is on after it); the world pool setup with the value 6 (see World memory budget and level setup).
8. Spins until a status call returns nonzero. The status call is a display field sync: each call waits for the next vertical-blank start; then, when the display record at data address 0x2EF780 is in interlaced mode (its first half-word is 1), it returns the GS field bit (1 on an odd field), and in any other mode it returns 1. So the spin ends on the first vertical blank of an odd field, after one or two vertical blanks (High).
9. Clears the display object's two vertical-blank counters (the blank-start count that the pace step waits on, and the blank-end count), installs one callback and runs one more initialiser. The callback goes into slot +0x184 of the shared object at data address 0x43BCF0; when called with a nonzero argument it calls an empty routine, so it has no effect (High). The last initialiser is one early run of the atmosphere-zone pass of main loop step 11 (SPEC-0028 type 41) on the atmosphere state object at GO+0xE0 (data address 0x32D760). The boot initialiser has just zeroed that object's stored world word, so this run only stores the current world and computes the first zone blend with no world-change crossfade (High).
10. Enters the frame loop. The vibration sync runs again on every pass (SPEC-0033). No movie plays from boot code itself.

### Boot initialiser (once, during boot)

Confidence: High for what it sets and the subsystems it starts; Medium for the role of the draw-context record.

The boot initialiser is the game-globals builder: the main boot function calls it once, as boot step 5, after the frame setup and before the dialogue command binding. In turn, it:
1. Sets two global function slots (data addresses 0x424C00 and 0x424C04) to the two ground-height queries in the current world (SPEC-0018), and fills the game-object pointer words with fixed blocks of the boot data region: the camera (GO+0x04, data address 0x32D060), the current and other world slots (GO+0x84 and GO+0x8C at 0x339C20, GO+0x88 and GO+0x90 at 0x33F570, GO+0x94 at 0x344EC0), the hero table (GO+0x20) and the selected hero (GO+0x24), both 0x335270, with the petpet slot GO+0x28 cleared, the live option pointer (GO+0x118 to 0x3396C0) and the other subsystem blocks. These are set-up writes with no gameplay values.
2. Starts these subsystems, in order: the character-pose singleton (SPEC-0040); the atmosphere state object at GO+0xE0, with its stored world and blend-active words set to 0, then bound to the 512 by 448 canvas; the clock record (GO+0x08, SPEC-0007); character pool B (SPEC-0040); the model registry (GO+0x38); the game-globals record (GO+0x00, data address 0x35C980); a 75-entry table of 32-byte records allocated from the boot pool (0x35C470); an empty hook; an object at 0x35C6F0 (GO+0xF0; role Low); the screen-effect (fade) object 0x35B500 (GO+0x130), set to 512 by 448 with 32 by 32 cells (SPEC-0052); the draw-context record; and a 180-byte block from the boot pool, stored at GO+0x14C.
3. The draw-context record (36 bytes, data address 0x3855C0) holds a word from the frame object, the vector (0, 0, 0, 1), a zero word and a pointer to the block 0x339C08 (GO+0xE4). The main loop rewrites its vector on every pass, and the draw phase passes the record to the character, world-effect and HUD draws. No timer scale block is set (Medium for the record's role).

### Start-up table walk (once, during boot)

Walk:

1. Start at index 59, the last entry.
2. Call the routine that the entry refers to, once.
3. Move to the next lower index. Stop after index 0.

The walk runs only when the guard word is zero. The first call sets the guard to 1, so a later call of the walk calls nothing. Nothing found clears the guard (Medium). The walk is counted from the count word (60); it does not look for a zero word. The main boot function calls the walk first.

Call order (High):

- Entries 59 to 3 run first. Index 59 is a tail call with its own arguments (the exception-unwind table registration); entries 3 to 58 are stage-1 wrappers.
- Entry 2 runs next. It is a stage-1 setup routine. It runs its subsystem setups, then writes the live option defaults (see Options).
- Entries 1 and 0 run last. They are stage-1 wrappers.

What each entry does (High for the shape and the values listed; Medium for entries not listed):

- The table is the program's list of static constructors (High). Each entry is a small wrapper for one program module's static initialiser, called with (1, 0xFFFF). Stage 1 constructs that module's static objects once. The entries run once, at boot, before the start-up body, and nothing calls them again.
- Stage 0 (with 0xFFFF) is the matching static teardown: destructor calls and resets of class-table pointers to their base tables. It never runs, because the main boot function never returns. The four stage-0 data pointers of entry 2's routine are such resets, so the clean side ignores them (High).
- What each entry builds, grouped by the owning spec (Medium unless marked; addresses are run-time memory):
  - SPEC-0026 (this spec): 2, the game module statics (game object, camera, heroes, options); 43, the global task system 0x393030; 53, the object 0x3A32D0, whose three words it clears; its word 0x3A32E4 is the inert word of the main loop (Medium-High); 59, the exception-unwind table registration (High).
  - SPEC-0007: 5, the frame object 0x368F00; 44, the five constant vectors.
  - SPEC-0008: 51, the random generator; 55, the packet pages; 56, two stream records 0x42BD90 and 0x42BDC0 (id -1) used by the bone-strip draw (SPEC-0046); 57, six constant vectors from 0x42BE20 (colour 255, 255, 255, 255, then the corner coordinates (0, 0), (0, 1), (1, 1), (0, 0) and (1, 0)) read by the effect trail age step and the trail strip draw (SPEC-0034); 58, a segment record 0x42C750 read by the strip and trail packet builders (SPEC-0046) (all Medium).
  - SPEC-0010: 7, the Cloud Beast lightning objects (Medium: their readers are the lightning routines).
  - SPEC-0011: 11, the Faerie Sister statics (the 0x36DC60 block, living count -1).
  - SPEC-0013: 12, the Gelert Assassin statics.
  - SPEC-0018: 18, the move-test constant 0x370478.
  - SPEC-0019: 14, the Werelupe King statics.
  - SPEC-0020: 13, the Gnarfas statics.
  - SPEC-0022: 4, the six dialogue command handlers; 30, the text box object 0x37B910; 50, the dialogue command table 0x39B480 (High for 4 and 50).
  - SPEC-0023: 17, the HUD and page sprite constants; 31, the HUD statics with the tint start colour 0x37BF20.
  - SPEC-0024: 6, the tunable descriptor table (hit points, radius, height, draw distance and so on); 36, the design-table binder 0x385628.
  - SPEC-0027: 0, the camera shared parameter block 0x32C240, copied from 0x3710E0 (built first by entry 22); 22, the camera view defaults 0x3710C0.
  - SPEC-0029: 8 and 9, boss effect-instance tables (read by boss controller routines); 10, the class-38 slot objects and the vector (0, 3.9, 0) (read by the class-38 slot dispatcher) (all Medium).
  - SPEC-0033: 39, the grab-hold vector; 52, the physics block.
  - SPEC-0034: 1, the hero hit-flash colour vector constants 0x32CEB0 (not a target marker; the hero's ground marker is SPEC-0036, Target marker); 21, the eight static effect records 0x370DF0; 23, the 27 effect presets 0x3711B0; 24, the hit-pulse preset 0x374830 (80 bytes: start scale 0.5, 0.5, 0.5; peak scale 1.9, 1.7, 1.9; end scale 1.0, 0.8, 1.0; colour bytes 200 and 128; counts 10 and 4), the only preset passed to the interpolator set-up by the shared monster hit handler and the hero take-hit block path (Medium); 25, the attach-point offset vectors 0x374C50; 26, an effect constant 0x374D00; 33, the model and effect registry values 0x382170; 37, the effect endpoint statics.
  - SPEC-0035: 32, the front-end text tables; 48 and 49, the front-end menu objects.
  - SPEC-0036: 27, the hero attack-family effect objects; 29, the proximity radius 0x378B44.
  - SPEC-0038: 15, the Mortog zone vector.
  - SPEC-0040: 38, the reticle object 0x3859A0; 42, the sub-object drift constants.
  - SPEC-0046: 3, the view and projection record 0x35C928 (512 by 448); 20, the effect-pair preset 0x3704F0 (colours 255, 0, 0 with alphas 0, 78 and 0; count 9; floats 0.09, 0.02 and 0.3; two words of 2) read only by the slot routine that places two instances of effect 0x12A at two attachment points (SPEC-0040 owns that routine; Medium); 40, the world render identity matrix; 46, the light and GS packet tables; 47, the draw default 0x39A290; 54, the render state 0x3A4BB0.
  - SPEC-0051: 34 and 35, the map page statics.
  - Entries 16, 19, 28, 41 and 45 build nothing observable.
- Start-up values written by identified entries (all in run-time memory outside the file image, so the clean side sets them itself):
  - Entry 44 (counting from 0): five constant vectors at 0x3932C0 (0, 0, 0, 0), 0x3932D0 (1, 0, 0, 0), 0x3932E0 (0, 1, 0, 0), the up vector, 0x3932F0 (0, 0, 1, 0) and 0x393300 (0, 0, 0, 1). Nothing writes them later.
  - The random generator initialiser: the generator's index word at 0x39D1D0 = 625, meaning not seeded (state object at 0x39BE50; SPEC-0014). The first draw seeds it.
  - The physics-block initialiser: 0x3A1A70 to 0x3A1A84 = 30.0, 1/30, 0.0, 32.0, 80.0, 0.03, and 16 zero bytes at 0x3A1A60 (SPEC-0033, Physics block).
  - Entry 2: the live option defaults (see Options).
  - The text-window initialiser: the tint start colour at 0x37BF20 = 128 per channel (SPEC-0022, SPEC-0023).
  - The sister-count initialiser: the living Faerie Sister count at 0x36DC64 = -1 (SPEC-0011, SPEC-0029).
  - The effect initialisers: the eight static effect records at 0x370DF0 and the 27 effect presets at 0x3711B0 (SPEC-0034), and the fade table constructor (SPEC-0046).
- The progress-flag default table at 0x32C2A0 is written by no entry and stays zero (SPEC-0022).

The 19-entry list (see Layout) is not walked by this build, so its entries are not part of the boot order.

### Start-up body (once, after the walk, before the first pass)

Confidence: High for the values and for their place in the boot order (steps 4, 5 and 7 of Boot order).

After the start-up table walk, the main boot function runs its start-up body up to the head of the frame loop. Among its steps:

- **Frame setup.** The rate words are written: 0x368FA0 = 60.0 (ticks per second), 0x368FA4 = 0.5, 0x368FA8 = 1/60 (tick length; SPEC-0007). The pass counter at 0x368F98 is set to 0 and the game time at 0x368F94 is 0.0. The graphics set-up inside this step builds the fade tables once (SPEC-0046).
- **Game-globals setup.** Builds the game-globals block at 0x35C7B0 (run-time memory). The block is the game object itself (its first word, at 0x35C7B0, holds the address 0x35C980 of the game-globals record, the separate singleton that carries the game-state code at +0x5C; the game object reaches that record through this word). Its character list is the world characters, then the two heroes (each counted only while its active byte is set), then a second set, in that order. +0x04 (0x35C7B4) is the camera object pointer (0x32D060), written once. +0x20 (0x35C7D0) is the hero table base 0x335270 (record stride 0x1680), and +0x24 (0x35C7D4) is the current hero = 0x335270 + 0x1680 × hero index, rewritten by hero switch and load (SPEC-0053). 0x35C834 is the current world object (0x339C20 at start-up; it changes on a world switch). The effect manager pointer at 0x35C8A0 is set to the manager block at 0x35C6F0 and is never null afterwards (SPEC-0034). +0x08 (0x35C7B8) is the clock record pointer (0x3866D0; SPEC-0007), +0x38 the model registry, and +0x118 (0x35C8C8) the live option pointer, set to the live options block 0x3396C0. 0x35C8F8 and 0x35C8FC are the count of, and a pointer to, a 45-entry actor list taken from the start-up arena; the game object rebuilds that list on every pass (SPEC-0029).
- **Start-up loaders.** The text database is loaded from resource 533 (SPEC-0024); its table 9 is the memory-card message table (SPEC-0025). The effect and event loaders follow.

### Boot order, visible part (once, during and at the end of the start-up body)

Confidence: High for the order and the pictures; Medium-High that nothing replaces the developer page during the hold.

0. **Publisher page.** During the display and pad set-up (boot step 7), the boot code expands the first boot picture, the "Sony Computer Entertainment America Presents" page (SPEC-0002, Boot pictures in the executable), into the second display buffer. It then waits 7 display field syncs while showing the first buffer, whose content is not set yet (Low; expected black), and then shows the second buffer, the publisher page. It then expands the second boot picture, the Idol Minds developer page, into the first buffer, which is not shown yet. The publisher page stays on screen through the memory pools and the start-up loaders, so its time on screen is the loading time and depends on the disc.
1. **Developer page.** After the start-up loaders (text database, items, menu sounds, music controller block), the boot code waits one field sync, shows the first buffer (the developer page) as a plain 512 by 448 display, and waits 180 vertical blanks (3.0 s at 60 Hz). Nothing is drawn over it in this time (Medium-High). The first front-end frame then replaces it.
2. It then loads the front-end database (resource 532) and sets the current screen directly to front-end record 0 (the card check), with the screen stack reset. No movie plays from the boot code.
3. The front end then shows, in order: the card check (record 0), the Neopets logo page (record 30, 301 passes), the legal page (record 1, 301 passes on the first visit since boot), movie 0 (ACT1 100A, 990 frames, 33.0 s), movie 86 (FRONTEND title, 452 frames, 15.07 s) and the title menu (record 10). So the card check comes first, and the logo comes before the legal page. Each step is described under "Attract mode and title".

### Dialogue command binding (once, at boot; High)

After the game-globals setup and before the start-up loaders (Boot order, step 6), the main boot function stores six handler objects into slots 0x11, 0x12, 0x13, 0x14, 0x15 and 0x19 of the text engine's 48-slot command table. These are the dialogue control-record kinds of SPEC-0022 (Dialogue control records):
- 0x11 sets a flag or a flag range;
- 0x12 clears a flag or a flag range;
- 0x13 gives items;
- 0x14 does nothing;
- 0x15 does nothing (it only prints a debug line);
- 0x19 applies a story stage: every stage-table row of that stage id sets its flag with the plain set (no notification), and every item row of that stage id adds its item (SPEC-0049).

They are not events of a global event object. Start-up table entry 4 builds the handler objects and entry 50 builds the table. The store ignores a slot index of 48 or more.

### The main loop, one pass

One pass is one tick of 1/60 s (SPEC-0007). Confidence: High for the order and the gates unless marked. The loop has no other per-pass caller of game systems; this list is the only per-pass order.

Names used below. GG is the game-globals record at data address 0x35C980 (run-time memory); its words are named by offset (GG+0x04 and so on). GO is the game object at data address 0x35C7B0: +0x04 the camera, +0x20 the hero table, +0x24 the selected hero, +0x28 the petpet slot, +0x84 the current world, +0x88 the second world.

1. **Frame begin.** Graphics frame begin; the DMA page is reset.
2. **Input.** One controller read for every pad (SPEC-0033). Everything in the pass reads that snapshot. The motor gate applies here.
3. **Display set-up.** Display registers for this buffer and the page-flip set-up; per-frame draw counters and lists are cleared; the render state is set.
4. **Load-busy word.** GG+0x28 = 1 when any of the three world buffers, or the region of any of them, is loading; otherwise 0.
5. **Status machine.** Timer E (GG+0x10) and timer F (GG+0x14); the accumulators GG+0x30, GG+0x38, GG+0x40 and GG+0x4C (each += dt unless it is -1); the six-pass rules of flags B and G; then the game-state code GG+0x5C by the priority table (see Game-state code).
6. **Rumble.** Vibration sync, then the rumble timeline. In code 9 the motor gate is forced off and the timeline is skipped.
7. **Camera.** The camera's mode update runs on every pass. Its follow update runs only when GG+0x04 = 1, the value left by the previous pass's front end (SPEC-0027). Then the shake and track applier with dt, the view and projection set-up, and the effect-holder refresh (SPEC-0034).
8. **World branch.** It always exists after boot, because a current world is always set.
   - 8.1 Only when the code is below 9 **and** the world-enable word GG+0x6C is nonzero:
     1. World tick of the current world: six world sub-tables (by world offset, in this order: +0xA0, +0x29F0, +0x2810, +0x2D40, +0x3850, +0x4260; none is ever filled in play, see Open questions, World sub-table owners); then the placed-object animation list (world +0x5834, SPEC-0028); then the movable blocks (world +0x5810, count at +0x580C, SPEC-0048); then the region pass of item 3.
     2. Region pass of the second world, only when it is loaded. Its sub-tables, its object list and its movable blocks do **not** run.
     3. Region pass (both worlds): first the world's sound emitters; then the world's own region; then each slot region in slot order. Each region runs its character loop (SPEC-0040), then its region emitters and its ambient and save-point distance steps (SPEC-0025).
     4. Placed-effect lists of the current and the second world (two kinds; SPEC-0034).
     5. Projectile pool: 10 records (SPEC-0029).
     6. Common-enemy attack manager (SPEC-0038).
     7. Item showcase step, with dt (SPEC-0021, Item showcase). The call at this place steps the showcase object, not the world item pool (Medium-High; see Open questions).
     8. GG+0x24 (boundary contact) = 0.
     9. Selected hero: update hook (SPEC-0040), then motion-record tick, then animation pass, then the attack-family and effect driver. It runs only if the hero has a motion record, an animation object and that object's active word set. The other hero slot is not updated at all.
     10. Active petpet (GO+0x28, when present; SPEC-0047): update hook, then motion-record tick, then animation pass.
     11. Animation-effect pool: attachment points (SPEC-0034).
     12. Script step of the current world (SPEC-0030 steps 1 to 5), then the script step of the loaded second world, which runs step 1 only.
   - 8.2 Every pass, also in code 9 and with the world disabled: the actor-list rebuild (the 45-entry list, from the regions of the three buffers and their slots; SPEC-0029); the clock record pass (play-time totals and calendar; SPEC-0007); the group alert board countdown (SPEC-0029); two thread-pool sweeps (they step nothing on this build; Medium-High); and the step of the 5 x 5 grid object, which does nothing on this build (Open questions, Grid object).
9. **Global task system.** Two task slots and a pool. A task runs on this pass when its mask has bit 1 and the code is 0 to 8, or its mask has bit 2 and the code is 9. The hero-swap tasks and the memory-card tasks use mask 3, so they run on every pass.
10. **Camera volume blend** (SPEC-0027).
11. **Draw phase.** Object draw. The atmosphere-zone and spin update (SPEC-0028 type 41, after the world update). World draws: the second world, then the current world. The second world is drawn only when the current world is not the interior buffer, the second world is loaded and the pairing test passes (SPEC-0006). Character, item and projectile draw submission. The world-item pool advances each eligible item immediately before preparing its draw, in this draw phase (SPEC-0021); it is not the item showcase step at 8.1.7. The per-item update is skipped when the game-state code is 9 or higher. Two draw-side pool walks split the item pool by render bank, so each item is advanced at most once per presented frame (see World-item per-tick step in the open questions). Its first part is the zone pass, which runs the foliage logic and draw for the current world and then for the second world (SPEC-0060). That is a separate call from the world draws, so the two orders do not conflict. The grid draw, which also does nothing on this build. Then, only when the code is below 9: the global light and emitter step; the character "apply" hook of the selected hero, of the petpet and of every actor in the actor list; the draw-list flush; the shadow pass; the character pass; the effects pass. When the selected hero's composite record +0x40 is nonzero, the effects pass is drawn before the characters. The effects pass advances the effect manager only when the code is 8 or less (SPEC-0034). Last come the blob shadows. The item showcase (8.1.7), the hero attack-family and effect driver (8.1.9) and the animation-effect pool (8.1.11) step only inside branch 8.1, so only while the code is below 9 and the world-enable word is set. This phase draws them but does not step them, so they freeze under pause (High).
12. **Per-pass flag reset.** GG+0x04 = 0, GG+0x20 = 0, GG+0x00 = 0.
13. **Screen effect.** The fade overlay steps and draws on every pass (see Loading screen and transition fades).
14. **Front end** (SPEC-0035). The current table-B state handler runs, then the page is drawn. The in-play state draws the HUD (SPEC-0023), runs its element timers and the countdown timer, tests the openers and the hero-switch chord, and sets GG+0x04 = 1 (see Pause and freezing screens).
15. **End-of-pass apply** (see below).
16. **Submit.** Frame submit, wait until the DMA unit is idle, GS transfer.
17. **Pace.** Wait for one vertical blank since the previous pacing point. Extra blanks that passed during a long pass are discarded.
18. **DMA chain start.**
19. **Button-combination detectors.** The ten-step sequence check on controller port 2, then the L3 and R3 gate of the debug watch screen on port 1 (see Open questions, Button-combination detectors).
20. **Movies.** If the movie queue is not empty and no world buffer is loading, play the queue (see Movies). This blocks the whole loop until the movies end.
21. **Render frame counter** += 1.
22. **Transition driver.** Advance any pending area load (see Area loading).
23. **Flip and clock.** Swap display buffers. Game time (0x368F94, 0.0 at start-up) += dt, the tick-length word (1/60 s). Pass counter (0x368F98, 0 at start-up) += 1. This is the only place in the loop where either word changes, so both advance exactly once per pass (High).

So the button-combination detectors, the movie player and the transition driver come after the frame submit and the vertical blank, and see the state left by this pass's front end.

Input readers in one pass, in order: the pad read (step 2); the camera right stick (step 7); the hero button chain and the quick-item panel (step 8.1.9); the level-script IsButtonPressed (step 8.1.12); the front end with the openers, shortcuts, the hero-switch chord, the dialogue keys and the pages (step 14); the movie skip (step 20).

The world branch 8.1 depends on the game-state code and the world-enable word; the late part of the draw phase depends on the code; the movie step depends on the queue and on the load state. Every other step runs on every pass (see Pause and freezing screens).

### End-of-pass apply (step 15)

Confidence: High for the order; Medium for the role of each step.

One routine, called once per pass at step 15, after the front end. It is the only routine with this shape: there is no second per-pass order, and no order with music first. Its steps, in order:

1. Render-block clears.
2. The pack-1299 object integrator.
3. The petpet item timer (5 s; SPEC-0047). It has no game-state gate.
4. The pending world swap (set by the hero's ground search, SPEC-0006, World swap): when a pending world is set, it becomes current without the atmosphere-zone step, the selected hero takes its world id and refreshes its ground surface, the petpet takes the same world id and is placed at entry record 0 of that world when it has entries, and a pending ground entry is stored on the hero; both pending words are then cleared (Medium-High).
5. The sky-dome refresh.
6. The shader and object refresh, only when the code is below 9.
7. The memory-card transfer step (SPEC-0025).
8. Only when the movie queue is empty: the file I/O pump, then the sound-cue pool and the music mixer (SPEC-0009, SPEC-0031). The audio part runs when the world-enable word is set, or when the title-start word GG+0x68 is set and the code is 9. Both audio calls also receive an I/O-idle flag: 1 when the first word of the file I/O record (data address 0x39BA50) is 0, else 0 (High for the source; Medium that the word means no request in flight). GG+0x68 is set on the first title pass and is never cleared, so before the first title visit since boot (card check, logo page, legal page) neither the sound-cue pool nor the music mixer steps (High).

The hero is not ticked in this routine; its tick is step 8.1.9 of the main loop.

### Pause and freezing screens

Confidence: High unless marked.

There is no separate pause flag, and pause does not use the world-enable word GG+0x6C (that word is set by the in-play state and cleared only by the world reset; see Gating of the world update). **Pause is code 9 through the play-active word GG+0x04.**

- **Clear.** Step 12 of every pass sets GG+0x04 = 0, just before the front end runs.
- **Set.** At step 14, GG+0x04 is set to 1 by the in-play state (table-B state 11) on every pass it runs past its credits, book-reader and game-over gates, before it tests the openers; by the box host (table-B state 23); and by the page stepper (table-B state 27) when its record's field +0x0E is 3 to 7 (Medium for the page stepper).
- **Read.** The status machine of the next pass (step 5) gives code 9 when GG+0x04 is 0. The camera follow update (step 7) reads the same word.

So every front-end state that does not set the word freezes the world from the pass after it first runs. These screens freeze it: the pause page, the item screens, the map, the journal, the options, the load and save pickers, the quit box, the book reader (table-B state 24, which sets only GG+0x20), the credits and the game-over page. The box host and the page stepper keep the world running.

**One-pass simulation lag** (High; SPEC-0035 Behavior 1b, steps 2 and 8, establishes that a returned page becomes current at pass end; the main-loop order establishes when the play-active word is consumed). Select is newly pressed on pass N. The in-play state sets GG+0x04 and returns the pause code. Pass N+1 still simulates the world because its status code was computed from the word set on N. From pass N+2 the code is 9. The pause record is current from pass N+1 and is first eligible to draw then; its full-screen picture load can delay the first actual draw and handler pass. On resume, the pause page returns the play code on pass M. Pass M+1 remains frozen because the world branch has already run with code 9; the in-play state sets GG+0x04 again at the end of that pass. The world runs from pass M+2.

**Boundary wait (second cause of code 9).** The boundary-contact word GG+0x24 is set when the selected hero's contact pass touches a ground primitive whose attribute is 7 or 9. The load-busy word GG+0x28 is rewritten at step 4 of every pass from the load state of the three world buffers. The code is 9 while both are set. GG+0x24 is cleared only inside the world branch (step 8.1.8), so once this rule fires the world stays frozen until the load finishes; the next world branch then clears the word.

**Effects visible while frozen.** The world particle group and swing trails can still be drawn in their frozen state at code 9: the world group does not step, and swing-trail samples do not age. The hero attack-effect group is not driven while the code is 9 or higher (SPEC-0034, Open question 142; High for trails and the hero group, Medium-High for the world group).

| Situation | Code | World simulation, AI, script threads | What still runs |
| --- | --- | --- | --- |
| Normal play | 0 to 8 | all | all |
| Pause page, item screens, map, journal, options, load picker, quit box, book reader, credits | 9 | frozen (8.1 skipped; effect objects do not advance; no script step of any kind) | step 8.2, the mask-3 global tasks, the camera mode update without the follow update, the status machine and its GG timers, the draws, the fade step, the front end, the card step, the sound cues and the music (the world-enable word stays 1; the pause music level is in SPEC-0035), the transition driver, game time and the pass counter |
| Save screen opened at a save point | 9 from the pass after the Square press | frozen | as above; the card jobs run as mask-3 global tasks; the screen holds the hero idle (SPEC-0025) |
| Game-over page | 9 from the pass after the in-play state hands over | frozen | as above; flag E stays set |
| Dialogue box (box host, table-B state 23) | 4, unless a higher rule matches | runs: the world, the AI and every script step; IsButtonPressed returns false (GG+0x20 is set) | everything, except the in-play HUD element timers and the countdown, which do not step because the in-play state is not running |
| Hero swap | 5 | runs; script steps 2 to 5 are skipped, step 1 runs | the swap task (mask 3) |
| Area-boundary wait | 9 while GG+0x24 is set and a load is busy | frozen until the load finishes | as for pause |
| Movie | none (the loop is suspended) | stopped before the queue plays: the current world is stopped, characters are reset, sound is stopped | nothing; game time, the pass counter and the calendar do not advance |
| Title and front end before play | world-enable 0 | world branch skipped | step 8.2 (the calendar runs), the front end, the audio as in the end-of-pass apply |

**Keeps running in every frozen screen:** the pad read; the status machine and its GG timers (timer E, timer F and the four accumulators); the camera mode update (the follow update is skipped); the actor-list rebuild; the clock record (calendar and play time); the group alert board countdown, so board entries age while paused; the global task system for mask-3 tasks (hero swap, cut-scene and memory-card tasks); the screen-fade step; the front end; the memory-card transfer step; the sound-cue pool and the music mixer (while the world-enable word is set); the transition driver; game time and the pass counter.

**Frozen:** all world simulation; all script threads; the character status timers; the effects (drawn, but not advanced); the follow camera; the rumble timeline; the HUD element timers and the on-screen countdown (they run only inside the in-play state).

**Dialogue boxes do not freeze the world.** The box host sets GG+0x04 and GG+0x20, so the code is 4 (flag D). The world, the AI and every script step keep running, while the level-script IsButtonPressed returns false.

**Movies block the whole loop.** The movie player runs inside step 20 until the queue is done. Nothing else advances meanwhile, not even game time, the pass counter or the calendar.

### Timers and ticks

1. Every duration in the game's data is in seconds. When the game uses a duration as a countdown, it multiplies by the tick rate (60) and keeps the whole part (rounded toward zero). Example: 5.0 seconds is 300 ticks; 0.25 seconds is 15 ticks; 0.1 seconds is 6 ticks. Confidence High.
2. Speeds given per second are divided by the tick rate to get the distance per tick. Confidence High.
3. Each character has 14 status timers, one per status (index 0 to 13: stun, curse, poison, slow, strength, speed, confuse, fire, no-curse, no-poison, no-slow, poison attack, health regeneration, magic regeneration). A status is active when its timer is at least 1 tick. Each tick, an active timer counts down by one. Confidence High (activity test), Medium (the countdown rule per status is in the combat specs).
4. Status timers are saved as remaining seconds (see SPEC-0025). Restore on load is not confirmed. The restore rule in SPEC-0025 (round to the nearest whole tick) is an assumption and is deliberately not the truncating rule in item 1. The saved value is a float32 seconds value, so multiplying it back by 60 can land just below a whole tick, and truncation would drop a tick.

### What actor updates schedule

Each character's update runs once per pass in its own state machine. The following are the scheduling effects found in those state machines. The exact states and ids are in the combat and character specs.

- **Animations by id.** A state change plays one animation id from the character's animation table (the table is in SPEC-0040). Several states choose the id from a four-way selector on a small index (for example indices 0 to 3 map to four ids for one attack family). The clean side must use the same id table.
- **Attack events.** A projectile or missile attack runs an event list. Each event fires at a time computed from the attack's frame data, in ticks. If the number of events does not match the number of frames, the original reports an error. The clean side should treat the mismatch as invalid data.
- **Attack manager list.** Attackers are added to a per-class list of 20 entries (SPEC-0038 Behavior 45). A duplicate add is refused. An add to a full list is refused too; its error report does nothing, so the game does not halt (High).
- **Projectile and jump flight.** The flight time comes from a quadratic arc: the vertical position uses one half of the gravity-like term times the square of time, and the landing time uses a square root of the height difference. The constants are per-character data (see the character specs).

Confidence: High for the existence of these scheduling effects, Medium for the details of each.

### Memory pools

The original allocates its large working buffers from fixed-size pools created at boot. A reimplementation replaces this plumbing with its own allocation. The sizes below matter because they bound the original's asset and object working sets.

- A bump pool keeps a base, a current offset and an end. An aligned allocation returns the aligned old offset and advances it. An allocation that does not fit fails. An unaligned allocation advances the offset only when the result stays below the end.
- Boot creates the asset pool at 461,824 bytes (0x70800). When assets do not fit, the original prints a warning about too few pools.
- Sound pools: one of 300 entries and one of 800 entries.
- Object buffers: one VU1 object buffer of 0x15E units, and two pools of 700 and 1000 entries. The unit for the VU1 buffer and the two pools is not confirmed.
- The save image (SPEC-0025) is a plain byte array, not a pool.

Confidence: High for the bump-pool behavior. Medium for the sizes (they are constants in the boot code, not data words, and were not read from the boot data).

### World memory budget and level setup

Level setup creates the level heap, an object pool of 700 entries, and calls the world pool setup with the value 6. It runs once at boot, as the last step of the start-up routine (Boot order, step 7; High).

- The world memory total is 14,680,064 bytes. A fixed transition region of 4,501,504 bytes is reserved from it, and the rest serves the three world slots: 10,178,560 bytes. The current world's pool is locked at 5,095,328 bytes, the transition region plus 593,824 bytes.
- The transition report prints statistics over the 193 world slots and returns the fixed transition size.
- The three world slots get an object capacity of 200 (current), 1,000 (next) and 2,000 (other). Two more objects get 100 and 101 (Low for the meaning).
- Each area load allocates from the world pool: a character block of 2,960 bytes, the world's script manager (428 bytes), the world-item pool (a 64-byte header and 50 slots of 0x170 bytes; SPEC-0021, World items), an effect group of 56 bytes when the pack has a type-34 chunk, the level tables, and the pools and block of the loader's pool step (SPEC-0006, finalize step 3).
- World pool reset (High): each world buffer has its own bump pool (base, cursor, end). Releasing the buffer sets the cursor back to the base, reserves a 9-byte zeroed record and cancels a read in progress. Every allocation made for that world (the pack, the finalize blocks, the script manager, the item pool, the slot buffers and the region arenas) is reclaimed then and never earlier. The interior buffer uses the transition buffer's pool.

Confidence: Medium for the sizes (constants in the level-setup code; purposes inferred from the statistics labels); High for the roles of the 428-byte and 64-byte blocks and for the pool reset at release; Medium for the rest of the list of allocations.

### Game-state code (once per pass)

The code is the first matching rule in this order. Each pass also adds 1 to a count of passes spent outside code 9. The play-active word that rule 1 reads is cleared at main loop step 12 and set at step 14 of the previous pass, so a screen change shows in the code one pass later (see Pause and freezing screens).

| Priority | Code | Rule |
| --- | --- | --- |
| 1 | 9 | The play-active word GG+0x04 is 0 (the value left by the previous pass's front end; see Pause and freezing screens); or both the boundary-contact word GG+0x24 (flag P) and the load-busy word GG+0x28 (flag Q) are set. A missing focus character is not tested here: rule 7 simply reads false for it |
| 2 | 7 | Flag A set |
| 3 | 8 | Flag B set |
| 4 | 6 | The enter-interior or the leave-interior sequence thread of the current world or the second world is alive |
| 5 | 5 | Flag C set |
| 6 | 4 | Flag D set |
| 7 | 3 | The focus character's state word (+0x144, the word the set-state hook writes) is one of 25, 33, 50, 51, 52 or 55 |
| 8 | 2 | Flag E set, or timer E is counting (greater than 0) |
| 9 | 1 | Timer F is counting (greater than 0), or flag G is set |
| 10 | 0 | None of the above |

Codes 0 to 8 let the world update run (if the world-enable flag is set). Code 9 stops it.

Code 9 also selects which global tasks run (main loop step 9): a task runs when its mask has bit 1 and the code is 0 to 8, or bit 2 and the code is 9. The hero-swap tasks and the memory-card tasks use mask 3, so they run in every code (High).

### Focus-character state gate

A predicate tests the focus character's state word. It is true when a focus character exists and the state is one of six codes: 0x19 (25), 0x21 (33), 0x32 (50), 0x33 (51), 0x34 (52) or 0x37 (55). Otherwise it is false. Confidence: High for the set; Medium for its use. It is one of nine tests that a record-level predicate combines with OR (Medium).

Rule 7 of the game-state table (code 3) is this same predicate: both read the state word at +0x144, not an animation id (High).

### Gating of the world update

| Condition | Effect |
| --- | --- |
| No world slot object exists | World section skipped. Not the case after boot |
| Game-state code is 9 | World update skipped. Main loop step 8.2 and every other step still run (see Pause and freezing screens) |
| World-enable flag is 0 | World update skipped |
| Otherwise | World update runs |

The world-enable flag (GG+0x6C) is written by only two routines: the world reset (sets it to 0) and the in-play front-end state, table-B state 11 (sets it to 1 when a post-load flag and a title-start flag are both set, and the flag is still 0). Confidence High. Pause does not touch it. The title-start flag is GG+0x68, set on the first pass of the title page. The post-load flag is GG+0x60, set at the end of the new-game and start-point world load and at the end of the post-load step of every area load. Both are cleared only by the game-globals reset at boot, so after the first title visit and the first completed load they stay 1 (High). On a fresh game (and after any quit or game over) the world reset sets the world-enable flag to 0, the start load sets the post-load flag, and the first pass of the in-play state sets the world-enable flag to 1 at main loop step 14. The world update first runs at step 8 of the next pass (High).

### Flags, timers and six-pass rules

- **Timer E (1.5 s).** When its accumulated time exceeds 1.5 s, it sets flag E to 1, turns itself off and plays sound event 5 at volume 1.0 (a sound, not a screen effect). Confidence High for the rule, Medium for the role.
- **Timer F (1.5 s).** When its accumulated time exceeds 1.5 s: if the current world has an entry list and the held index is below its entry count, the two 8-word blocks of that entry (position and facing) go to the focus character's start-point handler, after the character's ground-cell word is cleared. Whether or not an entry was found, the active petpet is then placed at the focus hero's start pose, the index is set to 0xFF and the timer turns itself off. Confidence High (Medium-High for the petpet placement). Timer F (GG+0x14) and its entry index (the byte GG+0x1C) are started together by the death-state handler, when a hero dies on deadly ground that carries a respawn entry index and no game over is pending; the entry table is the current world entry list (count at +0x24, 0x50-byte records), and the two blocks are the entry position and facing (SPEC-0055, section 8A).
- **Flag G (six-pass rule, code 1).** While flag G is set, a counter adds 1 on each pass where the loading or transition-busy check is false, and resets to 0 on a busy pass. At 6 counted passes, flag G is cleared, an 8-frame fade-in from white starts (colour 255, 255, 255, the data bytes at 0x30BCF8; frames are game ticks), the selected hero is sent set-state request 59 (0x3B), and the counter resets. State 59 is the arrival state of SPEC-0055: animation 175 and a one-shot effect; at progress 0.99 the hero goes to state 1 with sound event 0 (High). Flag G is armed by applying a loaded save (New game, load and start point).
- **Shared counter (High).** The six-pass rules of flags B and G count in the same word, GG+0x18. The "busy" test is: any world buffer or its embedded character block or slot is loading, or an item-side transition is running, or the front-end singleton's count is nonzero. The flag-B rule runs first. When both flags are set, a non-busy pass adds 2. Arming flag B, clearing flag B and a firing flag-B rule each set the counter to 0; the flag-G rule on the same pass then adds its own 1 if the pass is not busy, so a firing flag-B rule restarts the count of flag G. Flag G's own set and clear leave the counter alone, but a firing flag-G rule sets it to 0.
- **Flag B (six-pass rule, code 8).** The same busy-based counter (GG+0x18). Once the counter is above 5 (6 or more) and a completion condition holds, flag B is cleared and, if the load request's fade-in word is nonzero, the screen fades in over 5 frames (game ticks) from the colour of the world's load request and the counter is set to 0 again. The rule is tried on every pass while the counter stays above 5 and the condition is false. The completion condition is: bit 0 of the flag's word is set, its own timer (0 when armed, then counting seconds) is at least 0.1 s, and none of the three world buffers has a read in progress. The driver arms flag B (word 1, timer 0, counter 0) every time an exterior replacement request on a loaded exterior buffer passes its fade-out gate; the fade-in word only decides whether the fade-in runs at the end. The fade-in word is 1 for the with-fade replacement call, door replacements, the front-end entry, game load and the movie return, and 0 for the without-fade call (Loading screen and transition fades).
- **Flags P and Q (code 9 inputs).** Flag P is the boundary-contact word GG+0x24: set when the selected hero's contact pass touches a ground primitive whose attribute is 7 or 9, and set to 0 at main loop step 8.1.8. Flag Q is the load-busy word GG+0x28, rewritten at step 4 of every pass. Both must be set for the boundary rule. Because flag P is cleared only inside the world branch, the world stays frozen until the load finishes (High; see Pause and freezing screens).

Flag C is the hero-swap in-progress word GG+0x44: the swap request sets it and the end of the swap task clears it (SPEC-0053). Flag D is GG+0x20: the box host (table-B state 23) and the book reader (state 24) set it to 1, and main loop step 12 clears it on every pass. Flag A is GG+0x2C, with its own accumulator GG+0x30. Its only writer is the world-load request handler of an exterior replacement load with a fade request: on the first pass it starts a 6-frame fade-out to the request colour and sets flag A (accumulator 0); on each later pass it waits until flag A is set and its accumulator is at least 0.1 s (the compared value is a hair above 0.1, so in practice the 7th accumulation; Medium for the exact pass); then it clears flag A, arms flag B and runs the load (High). So code 7 shows during the fade-out before an exterior replacement load, and code 8 during the load and fade-in after it. Their effects are the codes in the table. Flag E is set by timer E and by the game over call (see Game over flag below).

### Game over flag

Game over comes from a script call or from the death-state handler's timer E; a death alone does not end the game while the partner can take over. There are no lives and no death penalty, and a death takes no money, items or experience; the one continue is the game-over page's Continue, which needs a save or load made earlier in the session (SPEC-0036, Death; the whole flow is SPEC-0055, section 2A).

- The level script can call the game over function (SetGameOver). Its argument is ignored. The call sets the game-state flag E. Timer E also sets flag E, when it runs past 1.5 seconds.
- Flag E gives code 2 for at most one pass (the world keeps updating on that pass, as for codes 0 to 8). The in-play state then hands over to the game-over page, which does not set the play-active word GG+0x04, so from the next pass the code is 9 and the world is frozen while the page is up, although flag E stays set (High).
- On the pass after flag E is set, in normal play and after the pause check, the in-play state clears the quit-hold counter and returns the return code 0x1CC (decimal 460, a byte offset into section D; front-end record 23), the code of the game-over page. The page itself runs table-B state 26. The front end shows that page on the following pass. This is the only hand-over found from a hero death or a script game over. The front-end records and state codes are in SPEC-0035.
- **Game-over page content.** The page shows only the caption "Continue" and two sprites with no animation frames. It offers no choice. The only input is Cross. Flag E stays set while the page is shown.
- **Game-over page bars.** On entry the page resets the bar heights to 32 px and brings the letterbox bars up over 10 frames (game ticks); on leaving it plays sound 0x97 and takes the bars down over 10 frames (SPEC-0052, screen effect).
- **Quit hold on the page.** The counter is set to 0 on the in-play pass that hands over to the page, one pass before page pass 1. The page's own first pass does not reset it. The counter rises by one on every page pass, whether or not a button is down (capped at 2^31 - 1). It counts passes, not held time, and it is not reset when Cross is released. Page pass k sees a counter of k - 1 before its increment. The page exits on the first pass numbered 122 or later on which Cross is newly pressed: Cross was up on the pass before and is down on this pass. A Cross press made earlier and still held does not exit. The earliest possible exit is page pass 122, about 2.03 s after the page appears at 60 passes per second. The 120-pass threshold is twice the tick rate (SPEC-0007).
- **Exit when no save or load is pending (normal case).** Flag E is cleared, the world is torn down (see Quit teardown), and the front end goes to code 0x000 (front-end record 0, the card state). The card job runs on the first visit in a session only, so on this later visit the card state lasts one pass. It then goes to code 0x258, the Neopets logo page, which leaves on its pass 300 (301 passes, about 5 s at 60 passes per second), going to code 0x014, the attract (legal) page. On this later visit the attract waits 1 pass, queues movie 0 (the first ACT1 movie), then movie 86 (the title movie), and goes to code 0x0C8, the title screen. The player therefore sees the logo, the legal page for 2 passes, the opening movie, the title movie, then the title screen (see Attract mode and title).
- **Exit when a save or load is pending.** Flag E is cleared. The pending card job runs, the screen and sound are reset, and the front end goes to code 0x0B4 (in-game play). Play resumes with no world teardown. A save or load is pending when a save or load pick was confirmed and no world teardown has run since.
- The screen states, the card state and the countdown are described in SPEC-0035. The game-over rules, the death rules and the quit timing test vectors are in SPEC-0055.

Scripted game-over tests (test vectors below):
- Level script 1122 calls the game over function, from the arena entry until movie 53 has played, when hero 1 is dead, item 50 is not held and the active hero is not hero 1; after movie 53 the ordinary death rules apply (SPEC-0055). When the active hero is hero 1, the same test calls the hero swap instead, locking the swap to hero 1 (swap mode 1).
- Level script 1217 runs only during the Act 3 Juppie fight: its stage gate is active, game-state flag 83 is set and flag 278 is clear. It tests whether Roberta (hero 1) is in death state 16 and has no item 50. If Roberta is selected, it sets swap mode 1; otherwise it calls SetGameOver(2) immediately. The module has one game-over call site, not a second health/occupied/no-swap test. With Roberta selected, swap mode 1 refuses the later death-state switch request and timer E leads to game over (SPEC-0055, section 4 and Open question 15; High).
- Hero 1 is Roberta (SPEC-0030, row 167; confirmed by SPEC-0055, section 4). Item 50 is the Potion of Revival (SPEC-0021): its only effect is a partner revive to full health with a switch request to the revived hero, not a percent heal.

Level script 1432 calls the game over function once, inside a scripted ending (fade, movie 80, then game over). The final fight's control thread runs that ending when the fight clock runs out before the fight is won; it is a scripted ending, not a death (SPEC-0055). The finale (movie 85) is the other ending of the same thread; the two never both play in one fight (SPEC-0052).

The Potion of Revival gate (item 50) is in three level scripts: 1122, 1217 and 1428. Each game-over call in those scripts runs only when the hero holds no item 50. Script 1432 is the only one of the four scripts that call the game over function whose call has no such test (High for the calls and the three item 50 gates; Medium for the reading of the other conditions).

### Attract mode and title

Confidence: High unless marked. Attract, the logo page and the title are front-end screens (SPEC-0035). Passes are counted from the screen's pass 0 (the pass on which it is first run). The screens are reached in the loop card check, logo page, attract (legal page), movies 0 and 86, title; the title's idle timeout re-enters the loop at the logo page.

**Controller check.** Several front-end screens run one check each pass: if pad slot 0 is usable (SPEC-0033 pad-usable test) it reports ready; otherwise it opens a message box with design table 9 record 25 (the message that the analog controller is not detected; no answers), updates that box, and reports not ready. The card check, the attract, the title, the credits, the start-point state and the lead-in page stepper run it. While it reports not ready the card check does nothing (its card job does not start), the attract wait does not count and the credits scroll freezes. On the title it does not stop the idle count. This is the "ready check" of the attract.

**Card check (record 0; High for the flow, Medium for the card conditions).**
- Each pass it first runs the controller check; while pad 0 is not usable nothing else happens.
- First visit since boot: it writes the two text variables used by the card messages (variable 0 = "1", the MEMORY CARD slot number; variable 1 = the free space the save needs) and starts the boot card job. It does not leave on this pass.
- Every later pass, and every later visit (the first-visit flag is never cleared): it runs the message box, and when the card job is no longer busy it leaves to record 30. On a later visit (after quit, game over or the finale credits) the job is not restarted, so the card check lasts one pass.
- The boot card job and its messages (no card; insufficient free space; Retry or Continue) are in SPEC-0025 (Card jobs).

**Logo page (record 30, front-end state 29).** Pass 0: select controller port 1 (pad 0) as the only menu input source and set the countdown to 300. Pass k (k at least 1): countdown = 300 - k; leave to record 1 when it is below 1, that is on pass 300 (301 passes, 5.02 s). The screen shows record 30's full-screen picture, the Neopets logo (resource 600). No input is read, the page has no items and Triangle is off, so the page cannot be skipped. It clears no music override.

**Attract (record 1, front-end state 1).** Pass 0: select controller port 1; wait = 300 on the first attract visit since boot, else 1. Pass k (k at least 1): only when the controller check reports ready, wait -= 1; when the wait falls below 1: queue movie 0, then movie 86 (both "append"), reset the lead-in page counter to 0, and go to record 10 (the title). First visit with a controller attached throughout: it leaves on pass 300 (301 passes) while showing the legal screen picture (resource 577, 4:3). Later visits: it leaves on pass 1 (2 passes). The movies play at the movie step of the same main-loop pass (see Movies); the title's pass 0 runs on the next pass.
- Leaving early is not possible. The logo and legal pages read no input. During the two movies only the movie skip works (Cross or Start held for 5 counted frames; see Movie skip), and it ends only the current movie: the queue's "skip cancels the rest" flag is cleared by every queue add and never set. So skipping movie 0 still plays movie 86, which needs its own skip. After the second movie the title appears.
- Movie 87 (FRONTEND attract loop) is in the table but is never played. The queue has three callers: the script play command, the attract screen, and a routine that is never called. The attract screen queues only 0 and 86, and no level script call uses 87 (High).

**Title (record 10, front-end state 12).**
1. Every pass: set the game-state word GG+0x68 (the title-start word) to 1 if it is 0.
2. Pass 0: idle counter = 0; close the music stream gate (SPEC-0035, Behavior 3b).
3. Pass k (k at least 1): if the menu rising-edge word (newly pressed buttons and stick directions; SPEC-0035, section 4) is 0, idle += 1, and when idle > 30 x 60.0 = 1,800 (the 30 is the data word at 0x2DCE38, the 60 the tick-rate word at 0x368FA0) the title goes to record 30, the logo page. Otherwise idle = 0. Only new presses reset the count: a button held down does not hold the timeout off. With no input the title leaves on pass 1,801.
4. A new game starts when the focus is item 0 (New Game), the pass count is 5 or more, and Cross (0x0040) is newly pressed. On every other pass the title sets the music bank override to 25 (bank resource 35) with the other three overrides -1 (SPEC-0009, Overrides), runs the controller check and makes no decision, so the generic menu step handles Load Game and Options from pass 2 (SPEC-0035, Behavior 1b).
5. New game: reset the world (this clears all progress flags), play cue 0x9A, clear both start-point overrides, select the start hero, set flag 707, load packs 691 (transition) and 909 (exterior, current), clear the four music overrides, and go to record 9 (play). No stage table is applied (start index 0; SPEC-0049) and no lead-in page is shown.
- There is no "press start" step: the menu is live from pass 2 (generic Cross) and pass 5 (New Game). Start (0x0800) has no effect on the title, and Triangle is off.
- One idle loop with no input: title 1,802 passes, logo page 301 passes and attract 2 passes (2,105 front-end passes, 35.08 s) plus 48.07 s of movie, about 83.2 s in all.

**Lead-in pages (High for the mechanism; Medium that they were cut).** Records 27, 24, 26 and 28 show the Controls, Attack Maneuvers, Quick Inventory and Mote screen pictures, each with a Cross icon and the caption "Next". One page stepper (front-end state 27) runs them with a counter c (0 at boot; reset to 0 by the attract when it leaves). On a page's pass 0 it clears the movie cancel flag and selects controller port 1; on page passes 3 to 7 it sets the play-active word GG+0x04 to 1; from page pass 11 on, any newly pressed button (the second input word, buttons only) turns the page: c 0 -> 1, record 24; 1 -> 2, record 26; 2 -> 3, record 28; 3 -> 0, clear the four music overrides and go to play (record 9). On other passes it runs the controller check. Entered at record 27 with c = 0, the order is Controls, Attack Maneuvers, Quick Inventory, Mote screen, play. No screen, item or code enters record 27 (or records 24 to 28), and the title's New Game goes straight to play, so the pages are never shown in the shipped game. The counter reset in the attract and the exit to play fit a removed "New Game -> lead-in pages -> play" step (Medium). Record 25 (Coming Soon, 16:9) runs the audio fade (state 28) and has no exit (Triangle off, no items); it is not entered either.

**First play on a new game (Medium).** After New Game the world load runs and pack 909's script plays the act 1 intro (movie 88) and then movie 1, under flag 707 (set by New Game) and flag 1 clear (see Movie play sites). That is the only lead-in before the player has control.

### Front end

The front end has two consecutive dispatch tables of code pointers. Table A holds 74 node predicates. Table B follows directly after it and holds 31 screen states (slots 0 to 30; slot 31 is empty). Together they are 105 consecutive code pointers. The count "31 screen handlers" counts table B only. Each screen state returns the next state. The screens are: title, attract, start-point choice (Act 1 to Act 4 start), new game, load game, save game, options (audio, controls, vibration, invert left-right, invert up-down, music, sound effects, voice), credits, card reset and wait screens, the in-game menu (with its cursor), and the game-over page (its quit hold is described under Game over flag). The handler-to-screen map is Medium-Low for most entries.

### New game, load and start point

- **New game.** The title's New Game item (see Attract mode and title, Title item 5): reset the world (all progress flags cleared), clear both start-point overrides, select the start hero, set flag 707, clear the four music overrides, reset the screen, and load the start packs: 691 into the transition buffer and 909 into the exterior buffer, with 909 current (World buffers and travel). It goes straight to play; no lead-in page is shown.
- **Start point.** Every start-point index loads the same two packs, 691 and 909, with 909 current. The chosen index only sets story flags (SPEC-0049).
- **Load game.** Read the card, apply the slot to the save state (progress-flag bitmap and options block are restored; see SPEC-0025, Loading). Applying the slot resets the screen effect, turns the screen white at once (a 0-frame white fade-out, opaque after one update), restores the played-movie words and arms flag G; 6 non-busy passes after the load an 8-frame fade-in from white runs (Flags, timers and six-pass rules). Then load the two saved resident packs: the world at save record +0x24 into the transition buffer, not current, and the one at +0x28 into the exterior buffer, which is posted as the current world. No interior is restored (World buffers and travel).
- **Resume.** From the in-game menu, resume does not reset the world. The handler is not confirmed.

### Options

- Block: 32 bytes of runtime memory, copied into each save slot's record (the layout belongs to the save spec). On load, the saved block is copied back into the live block and then the audio settings are applied from it (SPEC-0025).
- Slot blocks after start-up: each save slot's option block is zero after start-up. The start-up step writes the defaults below to each slot's block first, and the empty-slot fill then zeroes the whole slot. Only the live block keeps the defaults (High for the order of these writes; Medium-High for the zero fill). This matches the empty template of SPEC-0025.
- Words 0 and 1 (4 bytes each) are on/off toggles. Each flips when the left or right input is pressed on its row (see Controller map for menus and screens). The menu rows and the words they change are given in SPEC-0035.
- Floats at byte offsets 0x14, 0x18 and 0x1C are volumes from 0 to 100: music, sound effects and voice, in that order (SPEC-0035).
- Slider rule: while the left or right input is held on a volume row, the volume changes by 1.0 per pass, clamped to 0 to 100.
- On leaving the audio options, two audio levels step down by 0.05 per pass (Medium).
- Defaults, written to the live block during the start-up table walk by entry 2 (stage 1). This write happens before the save slots are initialised and before the first vibration sync, which copies the Vibration value into the pad gate (SPEC-0033, Behavior 1B):
  - byte 0x00 (Invert Left-Right) = 0;
  - byte 0x04 (Invert Up-Down) = 0;
  - byte 0x08 (Vibration) = 1;
  - byte 0x0C (unlabelled three-value cycler) = 1;
  - byte 0x10 (unlabelled toggle) = 0;
  - floats at 0x14, 0x18 and 0x1C (music, sound effects, voice) = 75.0 each.
- The bytes at 0x0C and 0x10 are stored and saved with the block, but no menu shows them. The front-end menu, its rows and their effects are in SPEC-0035.

### Credits

Confidence: High for the roll rule, the timing and the two entries; High for the setter chain of the epilogue; Medium that the player reaches the epilogue triggers. The credits are front-end record 14 (state 18; picture BG_credits, resource 589; SPEC-0035). Units: pixels and passes. H is the display height, 448 on this disc (the NTSC display mode record).

Roll rule:
1. Pass 0: top-row index i0 = 0; top-row y = H + 1 = 449 (the 1 is the integer at 0x2DCF94); speed = target = 1.0 (0x2DCF88); close the music stream gate.
2. Every pass: set the music bank override to 8 (bank resource 18), other overrides -1. Then run the controller check (see Attract mode and title); if it reports not ready, stop here this pass (no draw, no scroll).
3. Draw: walk the rows from i0 at y, y + 24, y + 48 and so on (pitch 24.0, 0x2DCF84). For each row: alpha = 128 x y / 32 if y < 32; 128 x (H - y) / 32 if y > H - 32; else 128; clamped to 0 to 128 (the 32s are the integers at 0x2DCF98 and 0x2DCF9C). A row with text is drawn centred at x 256 with the colour of its style in SPEC-0024 table 15 and that alpha. A style-5 row sets target = 1.5 (0x2DCF8C); a style-6 row sets target = 1.0. The walk stops after the first row whose y > H (that row is still walked, so its style still sets the target).
4. The speed moves toward the target by 1/512 per pass (0x2DCFA0), not past it.
5. y -= speed. If y < -20 (the integer at 0x2DCF90): y += 24 and i0 += 1 (at most one row per pass).

Row text (R-0023; High). Each text row is drawn by the text helper with font 0 (the first font descriptor of the front-end font set), centred horizontally at x 256 (the helper's centring mode 1 subtracts half the text width). The helper applies no line-height offset in that mode, and the glyph placement takes the row's y as the baseline: each glyph's quad runs from the baseline plus its descent up to that bottom less the quad height (SPEC-0022, glyph quads). So a row's y is its baseline, and the glyphs stand above it. The letter spacing is minus 2 pixels per glyph: the credits call passes -2 as the spacing, added to every glyph advance (spaces included) both when the width is measured for the centring and when the glyphs are placed. The fade alpha is written as the fourth byte of the row's colour word (0 to 128, where 128 is opaque); the glyph quads take it as their vertex colour alpha (High). Whether the glyph sheet's own alpha is also combined is set by the font's texture function, not by the credits code (Medium).
6. While i0 < 623: if the credits-pending flag is set and any button is newly pressed (second input word), return "no decision, generic Cross and Triangle blocked"; else return "no decision". When i0 reaches 623: with the pending flag set, clear it, tear the world down and go to record 0; else return "back".

Timing (float32, no pad loss, H = 448): the first row drop is on pass 469 (y goes from -21 to 3). The target changes to 1.5 on pass 5,113 (row 214 walked), to 1.0 on pass 7,092 (row 335), to 1.5 on pass 11,564 (row 524) and to 1.0 on pass 12,135 (row 557). The last row is dropped on pass 14,122, so the roll lasts 14,123 passes (235.4 s). With H = 512 (the other display mode) it would be 14,187 passes.

Entries:
- From the Options page: the Credits item targets record 14 (pushed; record 14's back rule is -1). Triangle from page pass 2 leaves early to Options (cue 0x99); the end of the roll returns to Options with no cue, focus on Controls. The bank override 8 stays in force on the Options page until the title (25) or another writer replaces it.
- From play: the in-play gate on story bit 368 sets the credits-pending flag and goes to record 14 (pushed; SPEC-0035, section 6). Cross and Triangle are ignored. The end tears the world down (which clears all flags, so bit 368 does not fire again) and goes to record 0, then the logo page (301 passes), the attract (2 passes), movies 0 and 86 and the title.

The credits row colours by style and colour set are listed in SPEC-0035 section G.5.

Route from the finale to the credits (High for each setter; Medium for the player reaching the triggers):
1. Pack 1432's finale block, called by the fight control thread under its script-local flag-4 gate, sets the return world to world slot 151 (pack 1300), posts blackboard message type 2 with value 80 for world 151, and plays movie 85 (SPEC-0054, Finale). This is the matching message consumed by pack 1300's activate-world flag-366 setter; the static call path does not prove the state-dependent gate is reached in every playthrough.
2. After the movie the hand-off reloads the current exterior buffer with pack 1300 and makes it current (see Movies). When the world becomes current its activate-world callback runs: it reads the type-2 blackboard value for world 151, and value 80 sets flag 366 and moves the hero to locator 2 (SPEC-0045, pack 1300).
3. Pack 1300's script trigger 2 (flag 366 set, flag 367 clear) sets flag 367, fades out, plays movie 79 (ACT4 417A1), reloads a character slot and fades in.
4. Pack 1300's script trigger 3 (flag 367 set, flag 368 clear) fades out, plays movie 83 (ACT4 417B3) and sets flag 368.
5. On the next play pass the bit-368 gate opens the credits from play.
Both triggers are registered unconditionally at pack 1300's Init. The return locator is at z = -124.126, while trigger 2 spans z = 174.608–181.608 (x -17.87 to 17.96) and trigger 3 spans z = 87.217–105.769 (x -97.93 to 98.01); each record's reference point is its low corner, and both are 27.36 tall from y 1508.8 and 1509.3. The return placement overlaps neither volume. Walking south from the return locator the hero crosses trigger 3's band first, which does nothing yet (flag 367 is clear); trigger 2 then plays movie 79 and sets flag 367, and walking back north over trigger 3 plays movie 83 and sets flag 368. So the hero must move into trigger 2 to start movie 79, then back into trigger 3 to start movie 83; nothing in the flow starts either movie, a closing scene, a save prompt or the title by itself after movie 85. The pack's single save point (15.43, 1514.5, 101.27) lies inside trigger 3's band, so the ordinary save is available before the credits; the roll's end returns to the front end with no save step. Actual trigger cadence after returning from the movies has not been played through.

No movie is queued by or tied to the credits screen.

### Movies

**Queue.**

- Queueing a movie adds its index to the ring. The ring has 5 slots and plays in the order queued.
- Mode "append" always adds. Mode "skip if playing" does nothing when the movie's playing flag is set.
- Script `PlayFMV(index, flag)` uses append. Its second argument is not used by that path. The index is always a constant written at the call; no call computes it (see Movie play sites).

**Playback and handoff (once per pass, step 20 of the main loop; High).**

1. When the queue is not empty: wait until the sound side is idle, then keep pumping the disc read queue until no transition is busy, no world slot is loading and the read queue is empty (SPEC-0008, Asynchronous read queue). So every load queued before the movie completes, and its completion callback runs, before the movie starts.
2. Choose the reload world W. If a return world is set (SetReturnWorldForFMV), W is the current slot and its pack is that return world. Else, if the current world is the transition or the interior, W is the exterior slot with its own pack. Else (the exterior is current) W is the other slot (the transition): when a return transition is set (SetReturnTransitionForFMV) it is copied into the return world and W reloads that pack; otherwise W reloads its own pack.
3. Unload W (with a vertical-blank wait), stop and unload the sound, return the camera to normal (ReturnToNormalCamera(-1, true)), reset the screen effect (bar heights 32 px, bars and fade cleared; SPEC-0052), set the five sound slots of both heroes to -1 (the heroes are not otherwise reset) and stop the sound system. Only W is released; the other resident world keeps its pack and its module's data words.
4. Play every queued movie in order. Before each one plays, its played word is set to 1; then the file path is built and the movie is played with the Bink decoder. Each play blocks until the stream ends.
5. Advance the ring and clear the queue.
6. Post a load of W: entry record 0 (plane -1) when a return world, or a return transition copied as in step 2, is used; otherwise plane -2 (the hero keeps its position and facing); colour black. The request's seventh word is the make-current word (make W current after the placement), not a fade-in flag: it is 1 only in those return cases, so otherwise the hand-off does not make the reloaded buffer current. Then clear the return world and the return transition to -1. The activate-world and on-enter callbacks of a world start when it becomes current, so after the placement (SPEC-0006).
7. Because W was unloaded, the load takes the immediate path that does not arm the post-load fade-in (flag B), so the new area appears without an engine fade (Medium); the area's own script fade decides what the player sees.

**What the reload of W rebuilds and what the player sees (R-0141; High for the module and character rebuild, the kill loss, the callbacks and the sound reset; Medium-High for the visible list).**
- *W's script and contents.* The release in step 3 runs W's Exit and unlinks its module. The reload reads W's pack again from the disc into the rewound buffer and relinks the module, then runs the Init of the family that belongs to the buffer (exterior, interior or transition) at load completion, so Init sees the file values: every module data word of W is back to its disc value (a lever or chest latch kept in a module word is reset; story flags and the inventory are global and are kept). The embedded character region is built again from its records (all positions, health and states reset; patrol routes start over), the slots are empty until Init loads them again, and the objects, breakable and block states, world items and triggers are rebuilt from the disc. Kills: the hand-off releases W without storing its kill block (SPEC-0006, Kill memory), so enemies killed in W since W was loaded come back; kills stored by an earlier release or save stay. Nothing is carried over from the old W. Confirmed (R-0190, High): each world has a live kill block (256 bits) and each area slot has a stored entry (33 bytes) in the per-slot table. A kill sets a bit in the live block only. The live block is copied into the stored entry at exactly two kinds of moment: when the per-pass driver step releases a loaded buffer for a request that is not a game load (each buffer that request releases, exterior, transition or interior, is stored just before its release, and only if it is loaded), and when the game is saved (the current and the second world only). A world load copies the stored entry into the live block. Nothing else writes the table (a new game clears it and a game load overwrites it from the save). The hand-off's release of W and the stream and interior releases are not stored. So a rewrite needs two kill sets per world slot (live and stored) or a "stored" mark on each bit: bits set since the last store or load are lost when W is released by the hand-off. The release of W by the hand-off does not touch the other resident world's live block, so its un-stored kills survive the movie (and are stored by a later save or driver release). Test vector: area slot of W holds stored kills {3}; the hero kills record 5 (live {3, 5}); a movie plays and W is reloaded: live {3} (record 5 returns). The same with a save made after the kill and before the movie (the save stores W when W is the current or the second world): live {3, 5} after the reload. A door to another exterior before the movie stores {3, 5} at that release, and a later movie reload keeps both.
- *Callbacks.* The activate-world and on-enter callbacks start only when W becomes current. In the plane -2 case the make-current word is 0, so W is not made current and neither callback runs; the current world (the other buffer) keeps its module words and its running threads unchanged. When a return world is set, W is the current slot, the word is 1, and both callbacks run once the load completes.
- *The common case (the movie plays in the exterior E).* W is the transition T, reloaded with its own pack and plane -2. T is not drawn unless the hero stands near the seam to it, so away from the seam the reload of T is invisible; with the seam in view, T is missing from the frames until its load completes (see the R-0141 resolution in the Open questions list). What the player sees and hears after the movie is the same for every plane -2 reload: (a) the focus hero is respawned at his current position and facing without a heal (SPEC-0040, Respawn at the start pose): he is in state 1 (idle, animation 0) whatever he was doing, his velocity and ground cache are cleared, and the grab cooldown is 60; (b) the camera is placed behind him at the respawn offset (eye at his position minus 10 times his forward, plus 2.5 up), which discards a camera angle the player had chosen (the camera was already returned to normal in step 3); (c) a live petpet is moved to the hero's start pose; (d) the music restarts from the first child of its bank and the sound blocks reload (SPEC-0009, Transitions); (e) no fade. The exterior itself (its characters, objects, script words and the kill block) is untouched. When T is visible through the seam, its characters stand at their disc placements, at full health, with this visit's kills undone.
- *Sound slots.* Setting the five bank ids of each hero to -1 stops no sound by itself: the ids only force the hero's voice, weapon, shield and Mote banks to be selected again on the first hero update, because a bank is reloaded when its id changes. What stops the sounds is the sound-system stop in step 3, which silences everything, music and looping sounds included. The restart (SPEC-0009, world-load audio reset) restores the music and the main sound block, not the loops that scripts had started. Answer to R-0190 (which loops are "scripts' own"): the voices that are started once and afterwards only refreshed are dead after the movie and stay silent until their owner starts them again. They are (1) every voice a level script started with LoopSound, which the script keeps alive with VisitLoopSound (VisitLoopSound never starts a voice; 18 call sites in the modules of packs 1171, 1174, 1217, 1392 and 1397); (2) a looping cue that an animated-object sound slot (AddAnimObjSound) had started and not yet stopped (a looping object animation fires its start again when it next wraps, a play-once object does not); (3) a creature's animation keep-alive cue, which starts at the animation change and is only refreshed each tick afterwards (it restarts with the creature's next animation change; Medium). Everything that is started or refreshed by a per-tick routine that also starts a missing voice comes back by itself: the music, the stream channels, the placed ambient sound emitters (kind 8 after their short delay counter, kind 1 at once, once the area's sound block has reloaded), the save-point cue and the hero's own loops. In the common case the exterior's module is not reset, so a LoopSound loop of the exterior stays silent for the rest of the visit unless its script starts it again (Medium). The music restarts from the first child of its bank in every hand-off, whether or not the current world changes: the hand-off itself stops the sound system and resets the music object (no bank selected), and the next driver step, which sees the sound system off, restarts it, so the area's row bank starts again from child 0 (High).
Default if unresolved (rewrite choice): on the reload, rebuild W from its disc image with its Init and no make-current callbacks, discard W's un-stored kills, respawn the focus hero at his own position and facing without a heal, place the camera behind him, restart the music from the beginning (also when the current world is unchanged) and leave scripts' own sound loops stopped (the LoopSound voices, a mid-play looping animated-object cue and creature keep-alive cues; the placed ambient emitters and the music restart by themselves). Keep two kill sets per world slot (live and stored), store only at a driver release that is not a game load and at a save, and load the live set from the stored one.

Playback is synchronous. The game loop, input and vertical-blank pacing are suspended for the duration of each movie. While the handoff and the transition-busy spin run, the screen shows the last presented frame. Medium: the wait is inside the player, whose full body is not described.

**Movie-entry frame boundary (High for order; open for presentation).** The movie step runs after this pass's draw phase, frame submit, vertical-blank pace and DMA-chain start, but before the pass-final display-buffer flip (main loop steps 11 and 16 to 18, then 20, then 23). A movie queued during the pass therefore blocks the ordinary flip until playback returns. The prior presented image remains during the player's pre-play wait; whether the newly submitted frame is shown before movie frames begin depends on the graphics hardware's display-buffer timing and can be decided only on a console; the default is the one under Movie-entry presentation (show the last presented frame until the first movie frame).

### Movie skip (High for the rule and the cadence)

The check runs once per presented movie frame, and only when the movie's skip flag is set. The queue player sets that flag when its no-skip word (offset 0x394) is zero. Every queue add clears that word and nothing sets it, so every movie is skippable (High).

The check reads two words of the combined pad snapshot for the current frame. That snapshot is in the global input object, not in the movie player. The snapshot is cleared on each read and then filled from the selected pads.

- Word 1 (offset 0x890) is the buttons held, OR over the selected pads.
- Word 2 (offset 0x894) is the buttons newly pressed since the previous read (held now and not held before), also OR over the pads.

The skip buttons are Cross (bit 0x40) and Start (bit 0x800). The previous-read word is not used.

1. **Arming.** Until one frame is seen in which neither button is held in word 1, nothing is counted. Each movie starts unarmed.
2. **Counting, once armed.** On each frame, if word 1 has Cross or Start held, counter 1 goes up by one. When it reaches 5, the skip latches.
3. **The second counter is dead code.** A newly pressed button is always held, so word 2 is a subset of word 1. Word 2 is read only when word 1 has neither button, so counter 2 (threshold 3) can never increment and its latch never fires. Do not implement it as a live rule. The effective rule is five counted frames with Cross or Start held.
4. **Latch timing.** The frame that latches returns "no skip". After the latch the pad is not read again for that movie. The next check, after the next presented frame, returns "skip" and the movie ends, so exactly one more frame is shown after the latching frame.
5. **No reset on release.** The counter, the latch and the armed flag are reset only at the start of a movie. They are not cleared when a button is released, so the five counted frames need not be consecutive.
6. **After a skip (High).** The player closes the stream, stops the movie sound and waits until the sound side is idle, the same as at a normal end, and the last presented frame stays on screen. The queue player then treats the skip like a normal end: its "skip cancels the rest" word is zero (every add clears it and nothing sets it), so only the current ring slot is cleared, the cursor advances and the next queued movie plays at once, unarmed. A skipped movie keeps its played word (set before it started), so HasMoviePlayed reports it as played. After the last queued movie, the hand-off continues with step 5 of Playback and handoff.

While a movie plays (Medium-Low for the read path, Medium for the frame steps):

- The file is read by name from the disc, under the MOVIES folder, through the asynchronous read queue (SPEC-0008).
- Before each frame is handed to the display, the player waits until the graphics transfer channel is idle.
- Each described frame is converted to 32-bit pixels (512 by 448) and shown as a full-frame texture.

### Movie play sites (High for the calls and indices; Medium for pack reach)

- A movie is queued only by a level script's play command or by the attract screen (see Attract mode and title). The queue routine's third caller is never called.
- Every play command in the shipped level code passes a fixed movie index written at the call. No call computes its index. There are 83 such calls in 37 level modules. Module numbers below are the pack resource numbers (SPEC-0006).
- Every call sits in code that runs from the module's own entry routines (init, update, exit, load or unload, or a control thread one of them starts). No call sits only behind debug-only code. The calls for movies 3 and 52 have a debug-mode path and also a path with no debug test.
- 72 distinct indices have a call. With the attract pair (0 and 86), 74 of the 92 indices are referenced. The other 18 are never referenced: index 87, and 17 later entries that repeat a file whose other index has a call (17, 28, 31, 39, 45, 62, 64, 66, 68, 73, 74, 75, 76, 77, 81, 82 and 84).

Reach classes used below (pack reach is from SPEC-0006 and World buffers and travel):
- **Script:** an area transition in a level script enters the pack, including a door's teleport callback and a return after a movie (Medium-High).
- **Pending:** the pending-area setter enters the pack (SPEC-0006). High for pack 1382 (slot 165, set by pack 1391's trigger 1 with no flag test). Pack 1428's setter slot (187) is fixed in pack 1427's trigger table, but pack 1427 is entered only from pack 1432, so 1428 is entered by streaming in the story.
- **Streaming:** no script transition enters the pack; overworld streaming with a world swap, or a default door, reaches it (Medium-High).
- **None:** the pack is among the 17 packs that cannot be entered from a new game (World buffers and travel). Its calls are present in code but not reached in normal play (Medium-High).

| Pack | Movie indices called | Where the call sits and its gate | Reach |
| --- | --- | --- | --- |
| 909 | 88, 1, 5 | 88 (act 1 intro) and 1 are queued one after the other, act intro first, from the update routine. The opening block needs flag 707 on and flag 1 off (it does not test flag 706). In order it sets flag 1, clears flag 706, queues 88 then 1 and makes the act-start 1 call. A second block of the same update (707 on, 1 on, 706 off) repeats only the fade-out, camera and placement and queues no movie (High) | Script; start exterior pack (current at every start) |
| 691 | 3 | Update routine; one path has a debug-mode test, another has none | Start transition pack (loaded at every start) |
| 678, 686 | 3 | Same code as 691 | None |
| 951 | 2, 6 | | Script |
| 1023 | 4 | A control thread started at area load. The chase thread runs only while flag 136 is clear; the block sets flag 136, gives the hero item 32, then plays movie 4, so movie 4 plays once per game (High) | Script |
| 1098 | 7, 8, 9, 10, 15 | | Script |
| 956 | 11, 13, 14, 43, 44 | | Script |
| 970 | 11, 13, 14, 43, 44 | Same code as 956 | None (these indices also play from 956) |
| 985 | 12, 13, 14, 44 | | Script |
| 1174 | 16, 89, 18 | 89 is the act 2 intro. Trigger 1 (the end rectangle of the Faerie arena) plays 16 before the act 2 start, waits tick by tick until the act start is done, then queues 89 and 18 in the same tick, so they play back to back in one queue pass; no flag gates it (its module bit 15 is never set; SPEC-0044) | Script |
| 1327 | 19, 27, 29, 90, 30 | 90 is the act 3 intro | Script |
| 1335 | 20, 21, 25, 26 | | Script |
| 1344 | 22, 24 | | Script |
| 1324 | 23 | | Script |
| 886 | 32 | | Script |
| 735 | 38 | | Script |
| 1184 | 40 | | Script |
| 989 | 46, 47 | | Script |
| 1139 | 49 | | Script |
| 1096 | 51 | | Script |
| 1411 | 59, 60, 69, 71, 72 | 71 needs flag 337 set and flag 338 clear. 72 needs flag 338, which only the movie-71 branch sets, after 337. Flag 337 is set (with flag 98) in pack 1382 by the six-pedestal puzzle that plays movie 70, so 71 and 72 play in normal play (High) | Script |
| 1371 | 61 | | Script |
| 1397 | 63 | | Script |
| 1398 | 67 | | Script (Medium) |
| 1300 | 79, 83 | The epilogue. Script trigger 2 plays 79 under flag 366 set and 367 clear (and sets 367); script trigger 3 plays 83 under 367 set and 368 clear (and sets 368). Flag 366 is set by the activate-world callback when the pack is entered with the finale's blackboard value 80 (Credits; High for the setters, Medium for reaching the triggers) | Script (also the movie-85 return from pack 1432) |
| 1432 | 78, 80, 85 | Final fight, all under its control thread: 78 at fight entry, 80 in the timeout ending, 85 in the finale (SPEC-0052) | Script |
| 1428 | 55, 56, 57, 58, 91 | 91 is the act 4 intro. The pack's sequence runs only while flag 322 is on. 55 plays at the entrance trigger, which sets flag 326. After the sequence sets flag 331 (flags 326 to 331 in order), 56, 91, 57 and 58 are queued in that order, with the act-start 4 call and return world 178 (pack 1411). No once-per-game latch: re-entering with 322 on and touching the entrance trigger replays 55 (Medium) | Streaming (from pack 1344); also by pack 1427's trigger 0, which is reached only from pack 1432 |
| 1382 | 70 | Called from the update routine when the six pedestals are lit together (sets flags 337 and 98) | Pending: pack 1391's trigger 1 (hero contact, no flag test) sets slot 165 (High) |
| 753 | 33, 36, 37 | | Script (a door callback of pack 758); also Streaming |
| 1246 | 41, 42 | | Streaming |
| 696, 714 | 48 | | Streaming |
| 1130 | 50 | | Streaming |
| 1122 | 52, 53, 54 | 52 has a debug-mode path and a path with no debug test | Streaming |
| 777 | 34, 35 | Both in the same routine | Script (door 1 of pack 758, a replacement load) |
| 1394 | 65 | | Streaming (from pack 1396) |

Totals over the 92 indices: 58 are played from packs a script transition enters (71 and 72 included); 1 from a pack entered by the pending setter (70 in pack 1382); 13 only from packs entered by streaming (41, 42, 48, 50, 52, 53, 54, 55, 56, 57, 58, 65 and 91; pack 1428 is also named by a pending setter in pack 1427); 2 are queued only by attract (0, 86); 18 are never referenced. No index is played only from a pack that cannot be entered: the calls in 678, 686 and 970 repeat indices that 691 and 956 play.

### World buffers and travel

Confidence: High for the buffers, the pack kinds and the start loads; Medium-High for the travel rules; Medium-High for the reach count. The full rules for streaming, swaps, doors, interiors and replacement loads are in SPEC-0006; doors are also in SPEC-0028 (section 13).

**Three world buffers.** The game keeps three world buffers: transition (T), exterior (E) and interior (I). Each holds at most one world pack, and exactly one buffer is current. The "other" buffer is E when T or I is current and T when E is current. While I is current, E stays resident and T has been unloaded.

**Pack kinds.** Every one of the 191 game packs has a fixed kind. A pack with a level module has the module's kind: 36 transition modules, 87 exterior modules and 50 interior modules. Of the 18 packs without a module, 15 are transitions (683, 684, 703, 704, 705, 713, 717, 718, 719, 720, 721, 1121, 1370, 1395 and 1396) and 879, 1298 and 1323 are not. The engine's own transition test is a fixed list of the 51 transition packs. A transition pack is loaded only into T and an interior only into I. Exteriors are loaded into E, and so are the interior-kind packs 1429, 1430 and 1431 when they are streamed from pack 1427.

**Family entries follow the buffer (High).** The level-script entries that run are chosen by the buffer, not by the module: a module in E gets the Exterior Init, Update and Exit, a module in I the Interior entries, and a module in T the Transition entries (SPEC-0030, Module lifetime). A module whose own family differs from its buffer's gets no Init, Update or Exit. In play this happens only to 1429, 1430 and 1431, which export the Interior family and are streamed into E from 1427: their scripts never run, so none of their callbacks, triggers or threads exist. Their leave callback in particular is never registered (it is not registered and then refused). The hero cannot stand in these three packs: they are only resident in E beside 1427 (SPEC-0006, Edge cases, Interior-kind packs in E).

**Overworld travel (streaming and world swap).** Normal travel between areas is seamless streaming, not a script load. Each tick the exit value of the grid cell under the hero names the neighbour to keep resident. When E is current and that neighbour is a transition, it is loaded in the background into T; when T is current and the neighbour is not a transition, it is loaded into E; while I is current nothing is streamed. When the hero's ground search finds no ground in the current world but finds ground in the other buffer, the worlds swap: on the next frame the other buffer becomes current. Some packs never swap (SPEC-0006 gives the swap test).

**Doors.** A door volume (SPEC-0028, section 13) either runs the world script's teleport callback for that door, or applies the default rule: from E into T, into an interior, or into a paired exterior; from T into E; from I back to the exterior it was entered from. The arrival point is a door of the destination world.

**Script transitions.** A level script can enter an interior (only while E is current: T is unloaded and the interior is loaded into I), leave an interior (only while I is current: E becomes current and T is reloaded with a named transition), replace the exterior (all buffers are unloaded and the new pack is loaded into E), or set a pending slot that the next streaming step uses once (SPEC-0006, SPEC-0030). The transition-scene call (StartTransitionScene) has no live caller on the disc (Medium-High).

**Return after a movie (Medium-High).** After a movie, once all buffers are idle, a replacement load is posted. If the script set a return world (SetReturnWorldForFMV), the current world is reloaded with that pack. Otherwise, when E is current, T is reloaded with the return transition (SetReturnTransitionForFMV) or with its own pack; when T or I is current, E is reloaded with its own pack. Only a reload with a return world or a return transition is made current by the hand-off (Movies, step 6).

**Start, load and the front end.**
- New game, the title start and every start-point index load pack 691 into T and pack 909 into E, with E (909) current. The start-point choice only sets story flags (SPEC-0049); no start index names another pack.
- Loading a save restores only the two resident packs: the save's world id at record +0x24 is loaded into T and the one at +0x28 into E, and E is posted as the current world (an id with no slot in the world table is skipped). If the restored hero position is on transition ground, the normal ground search then swaps to T on the next frame. The interior is never saved, so a load resumes in the saved exterior and transition pair (SPEC-0025).
- Game over goes to the front end (see Game over flag). Play continues only through a new game or a loaded save. Among the death callbacks, pack 1098 reloads its own world on a death, and packs 1176 and 1371 end their death sequences with a load of world 90 or world 157 (SPEC-0055, section 5).
- Switching heroes never loads a world (SPEC-0053).
- The map page has no fast travel (SPEC-0051).

**Reach (Medium-High).** From the new-game start (T = 691, E = 909, E current), 174 of the 191 packs can be entered by streaming, doors, script transitions and movie returns. The 17 others are copies or variants of other packs (678, 686, 970, 1077, 1323, 1259 and 1298), the pair 683 and 1003 that reach only each other, packs that nothing names (684, 720, 742, 879, 1004, 1055 and 1082), and 1170, whose only entry is in a script routine that nothing registers. The count does not take story flags or the walkability of a streamed neighbour into account.

### Area loading

1. **Request.** A level script, a door or a movie return asks for a load. Request kinds: exterior replacement load (with or without a fade) and enter or leave interior. The request sets the target world and the load mode. Overworld streaming and world swaps are not requests: they load the neighbour into the other buffer in the background (World buffers and travel).
2. **Driver.** Every pass the transition driver checks pending loads, for the current and the next world slot (the old and new world keys and the load mode). It prints the load message.
3. **Unload.** The driver takes the T request first, then the E request (a request on a buffer with no pack waiting does nothing). A taken request resets the dialogue text box, clears the boss countdown word and stops a running scripted camera play; with the sound system on, it resets the music settings, the stream channels, the bank context table and both heroes' five sound-bank ids (to -1) and stops all sounds. The render and effect state (combat subsystems, effect lists) is reset only when the request is a game load or an unload-all request, so a script exterior replacement does not reset it. Unload-all is set by door replacements and the front-end world entry, not by the script replacement calls. The actor list is cleared by the buffer release. Only the buffers the request names are released (SPEC-0006, Buffer load requests): a request on E releases E only, a request on T or I releases T and I, and an unload-all request releases E, T and I. Releasing a buffer runs its script's Exit, unlinks its module, releases its character regions and counts (SPEC-0006, Region slots), rewinds its pool and resets the world. The two hero objects are kept (High).
4. **Load.** The pack is read from the disc into the rewound pool. Its chunks are set up in walk order, with the module linked at its chunk, and the finalize steps run (SPEC-0006). In the same load-completion step the family Init of the buffer runs (SPEC-0030, Module lifetime), before the post-load placement and before the world is made current. A request on E always loads into E (the lair path: the lair is E), and every other driver request loads into T; an interior is loaded into I only by the enter-interior job (Entering the area). Driver loads are forced, so a resident copy of the same pack is never reused, even when the old and new worlds are the same; the lair path always reloads E from the disc (High).
5. **Finish.** The request is cleared and the hero is placed (SPEC-0006, Hero placement: plane index 0 or more lands on that teleport plane; index -1 or a refused plane uses entry record 0; index -2 keeps the current pose); the petpet is placed with the hero. One vertical-blank flush follows. When the request's make-current word is set, the world becomes current (the world-activation steps of SPEC-0006, Make current). The camera volumes are then blended once and the post-load flag is set (see Gating). After both drivers, the pass-end load step applies a pending saved pose and petpet (load game) and restarts the sound system if it is off. High for the set of steps, Medium for the order of the blank flush and the activation.
6. **Complete.** Clear the pending-load flag and reset the transition thread.

Load pauses come from I/O completion. Script fades (`FadeScreenOut` and `FadeScreenIn`) take seconds, with WAIT options; the engine's own transitions use the fixed fades below. The fade overlay is the screen-effect object of SPEC-0052, drawn on every pass before the HUD with alpha 0 to 128 (128 fully opaque). Every frame count is a count of screen-effect updates, which is one per game tick; script seconds become max(1, trunc(seconds x 60.0)) frames, with the product formed in single precision and truncated toward zero by the console (SPEC-0008, Platform arithmetic): 0.3 s = 18, 0.5 s = 30, 0.7 s = 41 (0.7 x 60.0 gives 41.999996; round-to-nearest would give 42). A rising fade or bar ramp stops only when its value is strictly above the target (it is then clamped to the target and the step cleared), and the busy test is value ≠ target. So a ramp whose sum lands exactly on the target is done on update n, and one whose sum ends just below the target takes update n + 1. The per-fade update counts under the console's round-toward-zero float adds are in SPEC-0052 Test vectors (High for the stop rule; Medium-High for the counts, which rest on the console's float rounding).

### Loading screen and transition fades

Confidence: High for the calls, the colours and the frame counts; Medium where noted.

- **Loading screen.** There is no loading art or text. A 25 by 30 spinner sprite (SPEC-0023 section 35) is drawn over the fade during the boundary wait, during loads under a nearly opaque fade, and in the fade-out before an exterior replacement load. Loads are asynchronous: the main loop keeps running and drawing, and what the player sees during a load is the full-screen fade at alpha 128 over whatever is drawn. The fade is black for every door load and every level-script load whose colour is resolved.
- **Enter interior** (a door, or StartEnterInterior; the colour comes from the call, and every disc call passes none, which means black): the focus hero is set to state 1; a 0.3 s fade-out (18 frames) starts without waiting, unless the fade is already resting at a nonzero target; the transition buffer is torn down; 2 yields; the interior load starts; yield while it loads, re-posting the interior's slot as the pending slot on every yield so the streaming step cannot start a conflicting load (the interior's Init runs during this wait); the job only runs while the exterior is current, and if the interior never becomes loaded it does no placement and no fade-in (Medium-High); the interior becomes current and the hero and the pet are placed; the camera returns to normal; then the wait for 5 consecutive idle passes (a busy pass restarts it); then a 0.3 s fade-in (18 frames) without waiting. The fade-out runs during the load; the screen is fully covered 18 updates after the start (19 with round-toward-zero adds).
- **Leave interior** (StartLeaveInterior): the focus hero is set to state 1; a 0 s fade-out with wait (1 frame, opaque on the next update); 1 more yield; the hero and the pet (not the partner hero) are placed in the resident exterior, which is made current without a reload; the interior and transition buffers are torn down; the named transition is reloaded; the wait for 5 idle passes; a 0.3 s fade-in (18 frames); ReturnToNormalCamera(-1, true).
- **Exterior replacement load** (a script call, or a door to a world that is not resident): once the exterior slot is loaded, a fade-out over 6 frames to the request colour starts, and the load begins when the request's own timer reaches 0.1 s (7 passes). When that gate passes, flag A is cleared and flag B is armed (for every such request, with or without a fade-in); after the load, 6 consecutive non-busy passes, its own 0.1 s timer and no buffer reading give a 5-frame fade-in from the request colour only when the request's fade-in word is nonzero (StartExteriorReplacementLoad, door routes, front-end entry; Flags, timers and six-pass rules). StartExteriorReplacementLoadWithoutFade has the same 6-frame fade-out (to black), the same hold of code 8 and no engine fade-in; the area script fades in. When the exterior buffer is not loaded the request skips the fade-out, flag A and flag B and loads at once.
- **Load game:** instant white, then an 8-frame fade-in from white 6 non-busy passes after the load (New game, load and start point).
- **Game-over page:** 32 px letterbox bars in over 10 frames on entry and out over 10 on leaving (Game over flag).
- **Hero swap:** a white fade-out over 37 frames to full opacity and an 8-frame fade-in (Cut-scene start; SPEC-0053).
- **Movies:** the handoff resets the screen effect, and the reload after the movies has no engine fade (Movies).

### Entering the area (scene entry job)

Confidence: High for the enter-interior placement and entry-index rules, state request, five consecutive idle frames and fade timing.

This job is the enter-interior job, started by a door into an interior or by StartEnterInterior. It requests and waits for the interior load itself; the interior’s Init runs while the job waits. Once the load completes, the job makes the interior current and resets the hero’s action state. For landing, a valid plane argument selects that plane’s upright landing formula. Otherwise, if the interior has entry records, the entry argument selects one; a negative or out-of-range index selects entry 0. If the interior has no entries, the default landing is position (0, 0, 0) and facing (1, 0, 0, 1). The landing height is then probed from six units above it.

After choosing the landing, the job stores it as the hero’s start pose, moves the hero and partner, sets their state to 1, and sets the camera eye to the hero position minus five times the forward vector. It then waits for five consecutive loader-idle passes; a busy pass restarts the count. Finally it starts the 0.3-second fade-in. The previously described entry-record-derived camera pose belongs to a different placement path and is not part of this job.

On the shipped script call sites, all 28 enter-interior calls pass plane -1 and entry -1, so they select entry 0 when the destination has entries. This does not establish the physical route or encounter order for every call.
### Hero-swap task (not an area transition)

Confidence: High that this task is the hero-swap task and that no area change queues it; the steps are specified in SPEC-0053.

The cooperative task that saves the focus character's state, plays the one-shot state 0x33 with cues 0xB1 and 0xB0, waits for the screen fade, places the incoming character (with a separate placement in swap mode 4), restores the saved state and then holds for trunc(0.7 x F) = 41 ticks is the hero-swap task. It is started only by the switch request (SwitchHero and the in-play switch routes; SPEC-0053 sections 3.2 and 3.8), in cooperative task slot 1. The scene-change request of Area loading does not queue it, and an area load does not run it. Its two optional flags are SwitchHero's two flags (SPEC-0053 section 3.5), its "mode 4" is swap mode 4 (SPEC-0053 section 3.1), and the step once described as the start of an area load is the release of the outgoing hero's locomotion (SPEC-0053 section 3.4, step 5).

The full order of its steps, its waits and the 41-tick hold are in SPEC-0053 section 3.4 (the hold is step 15); this spec does not repeat them. The 0.7 x F hold appears at one site only in the boot program, the end of this task, so 41 ticks is not an area-transition delay. Variant A of Cut-scene start (below) is the start of this same task.

### Bush instance step (moved to SPEC-0060)

Confidence: High. The routine once described here as the timed transition driver (scene-keyed timer) is the per-plant step of the cuttable bushes of the type-16 foliage system. It has nothing to do with area transitions, movies or the transition driver of the frame order. Its records are bush instances: state 2 is grown, 1 regrowing and 0 cut; the counter is a 600-step regrow ramp; the "speed mode" is a 3,800-step cut countdown; and the output is the bush's draw scale (s, s, s, 0.0) as floats, with no integer truncation. The "scene ids" are bush model ids, and the "cue" is effect 139 of effect group 3, started at the bush while it is grown and re-armed every 18 to 42 steps. The step runs from the patch's logic callback inside the world draw submission (main loop step 11), only while the camera eye is within the patch's near distance and the game-state code is below 9. The rules, the record layout and the test vectors are in SPEC-0060 (Foliage patches (type 16), section 6, Bushes).

### Cut-scene start (two variants)

Confidence: High that these are the two hero-swap sequences and for the fade colour, lengths and target; Medium for the order of steps (variant B) and for the owner of the state word; Low for the meanings of the events. The two routines are the hero-swap sequences of SPEC-0053: one is registered by the SwitchHero request, the other by its second requester. They share the steps below. Variant A is the hero-swap task of SPEC-0053 section 3.4 (see Hero-swap task); where the two disagree, SPEC-0053 holds. Variant A's fade is conditional: SwitchHero's second flag starts a white fade-out over 37 ticks to alpha 128 and starts the 8-tick fade-in after hero selection; with that flag clear, no fade runs (SPEC-0053, section 3.5). Variant A plays sound 176 (0xB0). Variant B plays sound 161 and sends hero event 0xB1. Steps:

1. Effects: the routine makes three effect instances from effect ids in the range 0xFB8 to 0xFBB (SPEC-0053, section 3.5, gives 0xFB9, 0xFBA and 0xFBB of effect bank 3).
2. Sound: the routine plays its sound (176 in variant A, 161 in variant B).
3. Hero event: variant B sends hero event 0xB1. Variant A's hero event is not recorded (Open questions).
4. Wait: the routine sets the focus character's state word to 0x33 (51, the one-shot request that PlayHeroAnim also sets; SPEC-0030), then yields one frame at a time while the state word stays 0x33. The word leaves 0x33 when the one-shot ends (the action-end resolver of SPEC-0040, Behavior 15.4; Medium). This is not the game-state code of this spec (0 to 9).

Variant B order (Medium):

1. Play sound 161.
2. Start a white fade-out of the screen effect (the three colour bytes at the data address 0x320998 are 255, 255, 255) over 37 frames (game ticks) to alpha 128, which is fully opaque.
3. Send hero event 0xB1 to the focus character.
4. Set the focus state to 0x33 and wait while it stays 0x33 (step 4 above).
5. Yield two frames.
6. Yield while the fade alpha differs from its target (the white fade-out has not finished).
7. Run a teardown routine with the focus character's hero index (+0x14C: 0 Tor, 1 Roberta; SPEC-0053).
8. Yield two frames.
9. Fade in from white over 8 frames (game ticks), from alpha 128 to 0.
10. Set the focus state to 1 (free).
11. Yield one frame, then stop the sound cue.

In single precision the white fade-out finishes on the 38th update (37 with exact arithmetic), and the fade-in on the 8th (SPEC-0052 Test vectors; Medium).

Neither routine queues a movie. Movies are queued only by level script play calls and by the attract screen (see Movie play sites).

### Teleport landing

A teleport into another world places the hero at a landing point chosen by plane index from that world's plane list. For a door, the index is the arrival door number: entry (door number) of the departure pack's tag-0x0E list (SPEC-0028, section 13, Doors). The full rule is SPEC-0006, Hero placement, Teleport landing (High); in short:

1. If the index is negative or at or past the end of the list, the teleport is refused and a message is logged.
2. A plane is flat when its vertical normal component is above 0.85 in absolute value. Flat plane: the landing point is the plane point offset along its two in-plane axes, then lowered by 5.0, with no probe. Upright plane: the point is offset half a width along the plane and 5.0 units out from it, then snapped to the ground probed from 6.0 units above that point. The 6.0 is a fixed program constant, the start height of the ground probe, so a floor up to 6.0 units above the landing point is still found.
3. When the hero is the focus and the world is the interior buffer (the "other" world slot) and has entry records, entry record 0 of that world replaces the landing point and facing. Every teleport into a world with entry records also makes its entry record 0 the start pose.
4. The hero moves to the landing point at once. For the focus hero the partner gets the same world, start pose and landing point, and the camera eye is set to the hero position minus 5 times the hero's forward vector.

Bar side effect (Medium; reproduce): landing the focus hero on a teleport plane (door touches into a resident world, and plane landings after a load) sets both letterbox bar heights to 256 px and requests bars up and bars down over 10 frames in the same tick. The two requests cancel, so nothing is drawn, but the 256 px heights persist until the next movie handoff, the game-over page or a return to the front end, so a later LetterBoxOn covers the whole screen (SPEC-0052, Edge cases).

**Door-use latch (hero word +0xF7C; Medium for the setter's button path, High for the readers).** Each hero has a latch word at +0xF7C with two accessors, set and "is 1". It is set to 1 only by the hero's action chooser (SPEC-0033), when the action button is pressed while the hero's pending-action word (+0xF78) holds an offered action. It is set to 0 on both heroes at every change of the current world, and by the door code when a door uses it. The door contact handler acts only for the selected hero, only when the game-state code is 0, no screen fade is running and the level script of neither world is busy. It first offers the door number to the level script's door hook; a hook entry that needs a press fires only while the hero's button bit is set and the latch is 1. A hook entry that is offered writes its index into the pending-action word. For the crossing itself, the handler clears the latch and performs the door only while the latch is 1; with the latch at 0 it writes the door prompt into the pending-action word (13, "Enter", while the exterior is current; 26, "Exit", while the transition or the interior is current; SPEC-0006, Doors), so the press is asked for first and the crossing happens on a later contact pass.

### What persists across an area change

| Item | Rule | Confidence |
| --- | --- | --- |
| Heroes | Never torn down or rebuilt by an area change. When the current world changes, only the selected hero's per-world references are cleared (its three 15-entry reference lists and its interaction and target words), its two ground-record slots are cleared, its movement controller is reset and its world id is set to the new world's id; word +0xF7C of both heroes, the door-use latch (see Teleport landing), is set to 0. A driver load with the sound system running sets the five sound slots of both heroes to -1 (SPEC-0031). A full reset of both heroes happens only on quit and new game (see Quit and in-game menu). | High |
| Pet | No area change deactivates or re-spawns it. It is loaded by its own loader, outside the three world buffers, and keeps running. The respawn placement, the teleport landing and both interior jobs move it with the hero (world id and position). It is deactivated by each of the four act-start branches (TriggerActStart 1 to 4) and re-created only by the pet activation paths (SPEC-0047). | High |
| Progress flags and options | Not changed by an area change (SPEC-0025). | Medium |
| Sound | Stopped at the start of each driver load; a streaming step does not stop it (SPEC-0006, Buffer load requests). | Medium-High |
| Camera | A running scripted camera play is stopped when a driver load starts; the camera volumes are blended once at the finish of each area load; no other camera reset is part of an area load. | Medium-High |
| World inventory (world items) | Per world: the world-item pool is allocated from the world's memory at every load and reset when the buffer is released; items lying in E survive interior visits and swaps with T (SPEC-0021, World items). | High |
| Level scripts | A module's data words last until its buffer is released (SPEC-0030, Module lifetime). | High |
| World memory | Each buffer's pool is rewound at that buffer's release (World memory budget and level setup). | High |

### Quit and in-game menu

- The in-game menu is a page with View Map, View Quests, Load Game, Resume Game, Options and Quit. It is drawn by the front-end step (main loop step 14) each pass, and the world is frozen while it is up.
- **Quit hold.** This is the hold on the game-over page (see Game over flag). The counter counts page passes, not held time, and is not reset when Cross is released. The page exits on the first pass numbered 122 or later on which Cross is newly pressed (not on a press that began earlier and is still held). The exit clears flag E, then either returns to the front end (world teardown) or, when a save or load is pending, runs the pending card job with no teardown. Medium-High (exit pass 122). The button is Cross (see Controller map for menus and screens).
- Loading a slot later overwrites the whole progress bitmap from the slot (see SPEC-0025). Confidence Medium-High.
- **Quit teardown.** Quit to the front end sets the focus hero to state 1 and waits for the transition. It waits one vertical blank, clears the actor list and releases all three world buffers that are loaded (without storing their kill bits), clears both heroes' pending swap state, resets the audio channels, the music settings and the bank context table, stops all sounds, resets the dialogue text box, returns the camera to normal, begins the letterbox state, sets both heroes' sound-bank ids to -1 and runs the world reset. It then resets the draw records, waits one more vertical blank and waits for the transition again (High). The world reset: clears the world-enable word, flag E and timer E; resets both item containers, the stash and the movie queue; stops a scripted camera play and restores the draw distance; resets the 10 projectile slots; deactivates the petpet; clears all 93 bytes of progress flags (quietly, no notifications), the kill-memory table, the broken-object table, the once-per-area sets, the treasure and map collections and the collected-pickup marks; sets both heroes to state 1, re-initialises their hit points and mode from default words, resets their luck value and clears their status timers; releases the effect lists and combat subsystems; clears the item counts; and resets the screen-effect object (High for the list, Medium for the roles of the default words).
- There is no separate pause flag. Pause is code 9 through the play-active word GG+0x04, which every screen other than play, the box host and the page stepper leaves at 0 (High; see Pause and freezing screens).

### Controller map for menus and screens (Confidence: High unless marked)

This is the one table of what each button does on each menu and screen. These rules hold for every row:
- Source. Only controller port 1 (pad slot 0) is read by menus; controller port 2 plays no part. Menus act on a new press (a rising edge), so a held button does nothing more and nothing repeats, except the sliders, which read the held word. The left stick counts as the D-pad in the main word (a stick byte below 32 or above 224 gives one move per crossing); the second input word has buttons only. Bit values and words are in SPEC-0033 (Controller map); the generic menu step is in SPEC-0035 (Behavior 1b and section 4).
- The first two passes of every screen ignore the generic Cross, Triangle and focus moves (about 33 ms).
- Cues: 0x98 select (generic Cross), 0x99 back (generic Triangle), 0x9A focus move, 0x94 menu open, 0x95 menu close. "None" means the handler plays no cue.
- A button that is not listed for a screen does nothing there. Within one pass a screen takes at most one action, in the order given in its Remarks column.

| Screen | Button | Result | Cue | Remarks |
| --- | --- | --- | --- | --- |
| Legal, logo and card-check pages | any | Nothing; these pages cannot be skipped | none | |
| Title | Up, Down | Move the focus (New Game, Load Game, Options; wraps) | 0x9A | |
| Title | Cross | On New Game, from page pass 5: start a new game. On Load Game or Options: open that page | 0x9A (new game), 0x98 (pages) | Before page pass 5 a press on New Game starts nothing |
| Title | Start, Select, Triangle, Circle, Square, L1, R1, L2, R2 | Nothing; there is no press-start step | none | |
| Title | no new press for 1,800 passes (30 s) | Leaves to the logo page, then the legal page | none | Only new presses reset the count |
| Pause page | Up, Down | Move the focus (View Map, View Quests, Load Game, Resume Game, Options, Quit; wraps) | 0x9A | |
| Pause page, on View Map | Cross | Open the map of the hero's area; stays when the hero has no map area | none | Reads the second input word |
| Pause page, on View Quests | Cross | Open the quest journal | none | |
| Pause page, on Load Game | Cross | Open the load picker | 0x98 | |
| Pause page, on Resume Game | Cross | Back to play | 0x95 | |
| Pause page, on Options | Cross | Open the options page | 0x98 | |
| Pause page, on Quit | Cross | Open the quit box | 0x9A | |
| Pause page, quit box closed | Select | Back to play | 0x95 | |
| Pause page, quit box closed | Triangle | Back to play (generic back rule) | 0x99 | Start, Circle and Square do nothing |
| Quit box | Circle | Quit to the front end: the world is torn down and nothing is saved | 0x98 | The box blocks all other input |
| Quit box | Triangle | Close the box and stay on the pause page | 0x99 | |
| Options page and rows | Up, Down | Move the row | 0x9A | |
| Options page and rows | Left, Right (new press) | On Vibration, Invert Left-Right and Invert Up-Down: flip the setting once per press | none | |
| Options page and rows | Left, Right (held) | On Music, Sound effects and Voice: lower or raise the volume while held | none | |
| Options page and rows | Cross | On a row that names a page: open it (controls, audio, credits) | 0x98 | |
| Options page and rows | Triangle | Back to the screen below | 0x99 | |
| Credits, from Options | Triangle | Back to Options | 0x99 | The roll ends by itself and returns to Options |
| Credits, after the finale | any | Ignored: the roll cannot be skipped | none | The end of the roll tears the world down to the front end |
| Item screen (Items, Charms and Quest items tabs) | Up, Down, Left, Right | Move the cursor (Items tab: 4 rows of 5, wraps; the other tabs as in SPEC-0035 section 14) | 0x9A | |
| Item screen | L1, R1 | Previous or next tab (wraps) | 0x98, 0x99 | L1 wins over R1 |
| Item screen | L2, R2 | L2: journal. R2: map of the hero's area, or the journal when there is none | 0x98, 0x99 | |
| Item screen | Cross | Use the selected item; refused when the cell is empty, unusable or on the Quest-items tab. Charms tab: equip the Mote in word 0 | 0xA2 on a refusal | |
| Item screen | Circle | Items tab: set or clear the quick-item marker. Charms tab: equip in word 2. Quest-items tab: refused | 0x9A (0xA3 when set), 0xA2 on a refusal | |
| Item screen | Square | Items tab: put the Tattered Cloak on or take it off. Charms tab: equip in word 1 | 0xA1 | |
| Item screen | Triangle or Start | Close; back to play | 0x95 | Tested before L1, R1, L2 and R2; Cross, Circle and Square pressed in the same pass still act (Medium-High). Select does nothing |
| Quest journal | Down, Up | Next or previous row (clamped; no wrap) | 0x9A | Order: Down, Up, Cross, L2, R2 |
| Quest journal | Cross | On a row with an area: open that area's map and make the row the pending target | 0x98 | Nothing on a row without an area |
| Quest journal | L2 | Map of the hero's area, or the item screen when there is none | 0x99 | |
| Quest journal | R2 | Item screen on its current tab | 0x98 | |
| Quest journal | Triangle | Back to play, also when the journal was opened from the pause page | 0x99 | |
| Map | Up, Right | Next icon (wraps) | none | The cursor update runs first; then Select, then Cross or R2, then L2 |
| Map | Down, Left | Previous icon (wraps) | none | If both kinds are pressed, the next-icon move wins |
| Map | Select | Back to play | none | |
| Map | Cross on an icon that has items, or R2 | Open the journal with that icon pending | 0x98 | |
| Map | L2 | Item screen on its current tab | none | |
| Map | Triangle | Back to play | 0x99 | |
| Book reader | Cross | Close the book and return to play | 0x9A | |
| Book reader | Triangle | Does not close the book: one play pass, then the reader again | 0x99 | Quirk; only Cross leaves |
| Dialogue and message box | Square (new press) | While the page is still revealing: show the whole page. When it is complete: Next turns the page, Finish closes the box | 0x98 on Next; none on Finish | The box host plays 0x97 when it hands back to play |
| Dialogue and message box | Cross (held) | Reveal five times faster | none | |
| Dialogue and message box | Circle, Triangle | On the last page of a question: Circle answers yes, Triangle answers no | 0x98, 0x99 | Circle wins when both are pressed |
| Load picker | Up, Down | Previous or next filled slot (empty slots are skipped) | 0x9A | Up and Down work only while the picker is shown |
| Load picker | Cross | Load the chosen slot and enter play | none | |
| Load picker | Triangle | Back to the pause page or the title | 0x99 | Ignored during the card scan and card messages |
| Save picker | Up, Down | Slot 0 to 3 (no skipping) | 0x9A | |
| Save picker | Cross | Write the chosen slot; card questions follow | 0xA9 when the save completes | |
| Save picker | Triangle | Return to play without writing | 0x99 | |
| Card question boxes | Circle, Triangle | First label (Circle) or second label (Triangle) | 0x98, 0x99 | |
| Game-over page | Cross (new press) | Continue: the game-over flag is cleared and the front end or the pending save is re-applied | none | Only from page pass 122 (about 2 s); a press still held from before does not count |
| In play | Select | Open the pause page (also when pad slot 0 is not usable) | 0x94 | Does nothing while the busy check holds |
| In play | Start | Open the item screen on the last tab viewed | 0x94 | Same busy check |
| In play | D-pad Left | Open the map of the hero's area | 0x9A | Needs an idle overlay box, closed quick panels and the in-play controls gate (SPEC-0035 section 6); Left wins over Right |
| In play | D-pad Right | Open the quest journal | 0x9A | Same conditions |
| In play | Square with a book pending | Read the book | 0xAF | SPEC-0035 section 6.1 |
| In play | all other buttons | Hero controls (SPEC-0033), quick panels (SPEC-0021), dialogue (SPEC-0022) | | |

Test vectors (synthetic):
- Title, Start pressed on pass 10: nothing happens. Cross on New Game on pass 4: nothing; on pass 5: a new game starts with cue 0x9A.
- Title, no new press for 1,800 passes: the logo page opens on pass 1,801; a held button for the whole time still counts as no press.
- Pause page with the quit box open, Triangle: the box closes (cue 0x99) and the pause page stays; with the box closed, Triangle returns to play.
- Item screen, Triangle and Cross in the same pass on a usable item: the item is used and the screen closes with cue 0x95.
- Journal, Up and Down in the same pass: the cursor moves down one row.
- Map with an icon that has items, Cross and L2 in the same pass: the journal opens; Select and Cross in the same pass: play resumes.
- Book reader open for 100 passes, Triangle: cue 0x99, one play pass, the reader again; Cross: cue 0x9A, play.
- Game-over page, Cross held from its first pass to pass 130, then released and pressed on pass 131: it exits on pass 131, not before.

## Edge cases and error handling

- **Count, not terminator.** The start-up table walk uses the count word (60) and does not stop at a zero entry. The clean side walks all 60 entries from index 59 down to index 0.
- **Second walk does nothing.** A later call of the table walk finds the guard word set and calls nothing. Nothing found clears the guard (Medium).
- **No stage-0 pass at boot.** Every entry the boot walk calls passes stage 1. Stage 0 is the static teardown (destructors and class-table resets, including the four data pointers of entry 2's routine). It never runs, because the main boot function never returns, and the clean side does not need it (High).
- **Duplicate movie names.** The table has 92 entries but only 75 distinct files. Queueing a name that appears twice plays the same file twice. Keep the table as-is so queued indices match. No play call uses any of the 17 repeat entries listed in Movie play sites; each of those files plays, if at all, under its other index.
- **Ring overflow.** The ring has 5 slots, and the queue does not check for a full ring. A sixth movie queued while the ring is full is written at an invalid index (-1). Reproduce the cap of 5 by refusing the sixth request, or document the original's behavior if it matters.
- **Movie skip never arms.** A movie in which Cross or Start is held on every pass never arms the skip, so it cannot be skipped by the movie skip rule (Medium). Intermittent presses add up over the movie, because the counters are not reset on release (Movie skip).
- **Skip-if-playing.** The skip rule applies only when the mode says so. The attract and script paths use append, so the same movie can be queued twice in a row.
- **Movie failure.** If a movie fails to decode, skip it as if the player pressed skip (SPEC-0008).
- **Six-pass counters reset on busy.** A busy pass resets the count to 0, so a loading screen restarts the wait rather than pausing it.
- **Option slider speed.** At 1.0 per pass the slider goes from 0 to 100 in 100 passes (about 1.7 s).
- Resolved: **Zero option block from an empty slot.** An empty slot cannot be loaded: the load picker skips empty rows, and when all four slots are empty its cursor sits outside the rows so Cross copies nothing (SPEC-0025, Empty slot choice). The live option block is therefore never overwritten by the zero block of an empty slot, and the start-up defaults stay in force (High).
- **Attack event mismatch**: an attack whose event count does not match its frame count is reported as an error by the original. A clean implementation should refuse the attack data instead of guessing.
- **Duplicate attacker**: adding an attacker that is already on the attack list does nothing.
- Resolved: **Entry list empty or count zero** (Entering the area). If the enter-interior destination has no entries and no valid plane argument, the job uses position (0, 0, 0) and facing (1, 0, 0, 1), then applies its normal ground-height probe (Entering the area; High).
- Resolved: **Negative entry index** (Entering the area). For the enter-interior job, a negative or out-of-range entry index selects entry 0 when the destination has entries; with no entries it uses position (0, 0, 0) and facing (1, 0, 0, 1). All shipped enter-interior calls use entry -1, so they select entry 0 for destinations with entries. This rule is job-specific and does not generalize to every placement path (SPEC-0006, Interior entry steps; High).
- **Loader busy during scene entry**: the count restarts on every busy frame, so the job keeps waiting while the loader is busy. Only five consecutive idle frames release it (Medium).
- **Attack manager full.** An add to a full 20-entry attack list is refused, and the error report does nothing (no halt; SPEC-0038 Behavior 45).
- **Six-pass counter shared.** Flags B and G count in the same word GG+0x18. With both set, one non-busy pass adds 2, and a firing flag-B rule restarts the count of flag G (Medium-High).
- **Dialogue command slot range.** A command slot index of 48 or more is ignored by the binding store (High).

## Test vectors

- Tick conversion: 5.0 seconds gives 300 ticks; 0.25 seconds gives 15 ticks; 0.1 seconds gives 6 ticks (computed in float32 the product is about 6.0 and the whole part is 6; a clean implementation should round to the nearest tick only if it can show the same result).
- Start-up table walk, synthetic table [P, Q, R] with count 3: R is called, then Q, then P, each with stage 1. A second call calls nothing.
- Boot order: the save-slot pointer reads the slot array 0x35CA20 and the live option pointer reads 0x3396C0 before the first pass; the dialogue commands are bound before the text database is loaded.
- Status activity: a status timer of 0 is off; a timer of 1 is on for one more tick; after one tick it is 0 and off.
- Speed: a speed of 60 units per second moves 1 unit per tick.
- Movie hand-off kills: stored entry of W {3}; kill record 5 (live {3, 5}); movie, reload of W: live {3}. With a save between the kill and the movie: live {3, 5}. Another world's live block is untouched by the movie.
- Movie hand-off sound: music playing child 2 of the exterior's bank, a LoopSound voice of the exterior's script and a kind 8 ambient loop active; a movie plays (T reloaded, E current). After the movie: the music starts at child 0 again, the kind 8 loop starts again on the third tick inside its sphere, and the LoopSound voice stays silent until its script calls LoopSound again.
- Attract, first entry since boot, controller attached: the movies are queued on attract pass 300 (301 passes, about 5.0 s). Any later attract entry, including the one after a game over: on pass 1 (2 passes).
- Boot, controller and card present throughout, card job ending on its first pass: record 0 passes 0 to 1, record 30 passes 0 to 300, record 1 passes 0 to 300, movies 0 and 86, title pass 0. That is 2 + 301 + 301 = 604 front-end passes (10.07 s) before the movies, after the 3.0 s developer page hold.
- Boot with no controller until 2 s after record 0 opens: the controller box shows over record 0, and the card job starts on the first pass with pad 0 usable.
- Boot with no memory card: the no-card box (Retry / Continue) appears over record 0; Triangle (Continue) lets the boot go on to the logo page.
- Title idle: 1,800 passes with no button, then the idle counter exceeds 1,800 on pass 1,801 and the title goes to the logo page. A press on pass 1,000 resets the counter, so the timeout fires on pass 2,801 instead. L1 pressed on pass 1 and held: the count is reset on pass 1 only, and the timeout fires on pass 1,802.
- Title: Cross on pass 4 with focus 0 does nothing; Cross on pass 5 starts a new game (cue 0x9A). Cross on pass 2 with focus 1 opens the load prompt (generic step, cue 0x98). Start on any pass does nothing.
- Boot pictures (High for the order; the publisher page time depends on loading): field syncs 1 to 7 of the display set-up show the first buffer before its picture is in place; from field sync 7 the publisher page is shown until the start-up loaders end; then one field sync, the developer page, and 180 vertical blanks (3.0 s); then the first front-end pass (the card check).
- Movie skip end (synthetic): queue [0, 86]; skip latches on presented frame 100 of movie 0; frame 101 is presented, the check returns skip, movie 0 ends; its played word stays 1; movie 86 starts at its frame 0, unarmed, with counter 0. With the queue [0] only, the hand-off continues as after a normal end.
- Act 2 intro (synthetic order): hero enters pack 1174 trigger 1 at tick T: fade out 0.3 s (18 ticks), flags 219 and 220 set, movie 16 queued, act 2 start triggered; the routine waits until the act start reports done at tick T + n; at that tick movie 89 is queued, flag 221 is set, the movie return world is set to 151, and movie 18 is queued; the next queue pass plays 89 then 18, then reloads with pack 1300.
- Attract movie skip: Cross held through movie 0 ends it after 5 counted frames (once armed); movie 86 then starts unarmed and, with Cross still held, counts nothing until a frame with Cross and Start released, then needs 5 more counted frames.
- Timer E: starts at 0. It passes 1.5 s on its 91st addition of the single-precision 1/60 s: 90 additions sum to 1.4999993, which is not above 1.5, and 91 sum to 1.5166659 (SPEC-0055, Timers).
- Result codes: no focus character gives code 9, and the world update is skipped. Focus present, play-active word 1, no flags, no timers, no state word in the set, no interior sequence thread alive gives code 0.
- State rule: focus character state word 33 gives code 3 (unless a higher-priority rule matches).
- Six-pass rule: busy, busy, then 6 non-busy passes clears flag G on the 6th non-busy pass. A busy pass in between resets the count.
- Six-pass rule outcomes: flag G cleared on its 6th non-busy pass starts an 8-frame fade-in from white (alpha 112 after the first update, 0 after the 8th). Flag B with the fade-in flag starts a 5-frame fade-in instead.
- Enter interior (synthetic, black): the 18-frame fade-out starts with the request; alpha is 128 after update 18 (update 19 with round-toward-zero adds) while the load runs; after the load, 5 consecutive idle passes, then an 18-frame fade-in. One busy pass among the idle passes restarts the count of 5.
- Movie handoff (synthetic): exterior current, no return world, no return transition: the transition slot is unloaded and reloaded with its own pack, with plane -2 (the hero keeps its position). The hand-off does not make it current. Return world set: the current slot is reloaded with the return world at entry record 0 and made current. Exterior current, return transition set to slot 131: the transition buffer reloads pack 1220 at entry record 0 and becomes current. In all cases every queued movie's played word is 1 afterwards.
- Ring: queue 5 movies into an empty ring and all play in order. A 6th queued movie is an edge case (see Edge cases).
- Movie table: 92 entries; index 86 is the title movie; indices 16 and 17 both name the same ACT1 file.
- Movie names (disc check values): index 4 gives the string `ACT1\103A.BIK`, so the name ACT1/103A; index 86 gives FRONTEND/TITLE.
- Movie counts (group totals, consistent with the table): ACT1 18 entries for 17 files, ACT2 12 for 11, ACT3 27 for 24, ACT4 29 for 17, FRONTEND 2 for 2, ACTINTRO 4 for 4. Totals 92 entries and 75 files.
- Frame timing by index: index 0 (ACT1) plays at 30 fps; index 88 (ACTINTRO) plays at 29.97 fps.
- Play sites: 83 play calls in 37 level modules, each with a fixed index; 72 distinct indices. Attract queues 0, then 86. Index 87 is never queued in any play. Indices 17 and 28 (repeat entries) are never queued.
- Credits roll (float32, no pad loss, H = 448): the first row drop is on pass 469 (y from -21 to 3); the roll lasts 14,123 passes (235.4 s). Row alpha at y = 449, 440, 432, 416, 16 and 0: 0, 32, 64, 128, 64 and 0.
- Credits from Options, no input: 14,123 passes, then Options with focus 0 (no cue). Triangle on credits pass 2: back to Options, cue 0x99.
- Credits by the story-bit gate, Cross pressed on pass 100: ignored; after 14,123 passes the world is torn down and the front end goes to record 0.
- Finale route (synthetic, pack 1300 entered after movie 85 with blackboard value 80 for world 151): flag 366 is set and the hero is placed at locator 2; trigger 2 then plays movie 79 and sets 367; trigger 3 plays movie 83 and sets 368; the next play pass opens the credits.
- Lead-in stepper (synthetic, record 27 entered with c = 0): a press on page pass 10 does nothing; a press on page pass 11 goes to record 24; three more presses, each on pass 11 of its page, go to records 26 and 28 and then to play. Play is reached on the 48th page pass in all (4 x 12).
- Game over: a SetGameOver call (argument ignored) sets flag E. On the next pass the game-state code is 2, unless a higher-priority rule (codes 3 to 9) matches first. Codes 0 to 8 let the world gate run. From the pass after the in-play state hands over to the game-over page, the code is 9 and the world is frozen while the page is up. On the game-over page, Cross down on page passes 1 to 121 and released before pass 122 gives no exit. A Cross press first made on page pass 100 and held through pass 130 gives no exit on pass 122, because that press is not new on pass 122. The same press released on pass 131 and pressed again on pass 135 exits on pass 135, which clears flag E and goes to code 0x000. Cross tapped only on pass 121 gives no exit. Cross newly pressed on pass 122 exits.
- Movie skip (synthetic): Cross held on the first word for 4 passes before any pass with neither button held counts nothing (not armed). Then one pass with neither held arms the check. Cross held on the first word for 2 more passes, released, then held for 3 more passes: the counter reaches 5 on the fifth counted pass, and the skip latches. Start held on word 1 for five counted frames latches, and counter 2 stays at 0. A movie in which only newly pressed buttons appear in word 2 (never in word 1) does not latch.
- Entry list (synthetic): three entry records of 0x50 bytes occupy 0xF0 bytes. The rest of the list layout is open.
- Load wait: the scene entry job releases the actor reload after five consecutive frames with the loader idle. A busy frame restarts the count. Synthetic: idle, idle, busy, then idle frames: the release happens on the fifth consecutive idle frame after the busy frame.
- Budget arithmetic: 4,501,504 + 10,178,560 = 14,680,064 (transition region plus the world slots equals the world total). 4,501,504 + 593,824 = 5,095,328 (the locked pool of the current world).
- Teleport landing (synthetic): a plane with vertical normal component 0.9 lands at its offset point lowered by 5.0, with no probe. A plane with component 0.5 is probed from 6.0 units above its offset point. A plane index past the end of the list is refused. Hero forward (0, 0, 1) at (10, 2, 10) after the landing: camera eye (10, 2, 5).
- Dialogue command binding: a control record of kind 0x14 changes nothing; a record of kind 0x11 sets its flag (SPEC-0022).
- Flag G after a load (synthetic): a load whose apply arms only flag G, then 6 non-busy passes. On the 6th pass the fade starts at alpha 128 white, reads 112 after the first update and 0 after the 8th, and the selected hero is in state 59 with animation 175.
- Shared six-pass counter (synthetic): flags B and G both set and every pass non-busy: the counter adds 2 per pass. When the flag-B rule fires, the counter returns to 0, so flag G needs 6 more counted passes after that.
- Attack manager (synthetic): 20 attackers on the list; a 21st add is refused and the game goes on.
- Game-over test, level script 1122 (synthetic): hero 1 dead, item 50 not held, active hero 0. SetGameOver is called with argument 0 and the swap mode is unchanged.
- Game-over test, level script 1122 (synthetic): hero 1 dead, item 50 held, active hero 0. Nothing is called.
- Game-over test, level script 1122 (synthetic): hero 1 dead, item 50 not held, active hero 1. No game over; the swap mode is set to 1 (lock to hero 1).
- Game-over test, level script 1122 (synthetic): hero 1 alive, item 50 not held, active hero 0. Nothing is called.
- Game-over test, level script 1217 (synthetic): during the Act 3 fight stage with flag 83 set, flag 278 clear, Roberta in state 16 and no item 50, Tor selected calls SetGameOver(2) immediately. With Roberta selected, the thread sets swap mode 1; her later death-state switch request is refused and timer E leads to game over. With item 50 held or the flag gate closed, the test does not run.
- Game-over test, level script 1432 (synthetic): the game over call has no condition, so it runs whether or not the hero holds item 50.
- New game: the transition buffer holds 691, the exterior buffer holds 909, and 909 is current. While the hero stands on a cell of 909 whose exit value names 691 (a transition already resident in the transition buffer), nothing new is loaded. A cell whose exit value names pack 951 (an interior) streams nothing, because interiors are not streamed.
- Load: a save whose world ids at +0x24 and +0x28 name packs 1370 and 1350 resumes with 1370 in the transition buffer and 1350 in the exterior buffer; no interior is loaded (synthetic pair).
- Hero switch: switching heroes in any area leaves all three buffers unchanged.
- Hero across an interior: a hero at 250 of 400 HP enters an interior and still has 250 after the arrival (the placement does not heal).
- Pet across streaming: pet out, the hero walks from E into T: the pet is still active and has the hero's world id after the next respawn or landing. Pet out, Act 3 starts: the pet is gone.
- Interior-kind pack in E: 1429 streamed into E from 1427: no Init, Update or Exit runs, and no callback of its module exists.
- Lair reload: E holds 909 and a driver request on E names 909: E is released and 909 is read again from the disc, with fresh module data words.
- Timer E (synthetic): the timer passes 1.5 s, sets flag E, turns itself off and plays sound event 5 at volume 1.0. No screen effect is started.
- Pause lag (synthetic): Select newly pressed on pass 100: the world runs on passes 100 and 101 and is frozen from pass 102. The pause page returns the play code on pass 200: pass 201 is still frozen, and the world runs from pass 202.
- Dialogue box (synthetic): a box open for 60 passes, with an enemy walking at 0.0834 units per tick: the enemy moves 60 x 0.0834 = 5.0 units during the box, because code 4 does not freeze the world.
- Boundary wait (synthetic): GG+0x24 is set during the world branch of pass 10, and the load-busy word is 1 on passes 10 to 40. The code is 9 on passes 11 to 40. On pass 41 the load is no longer busy, the world runs, and its world branch clears GG+0x24.
- Movie (synthetic): a movie that plays for 600 presented frames from step 20 of pass 1,000 adds nothing to game time, the pass counter or the calendar while it plays. After pass 1,000 ends, the pass counter reads 1,001, as after any pass.
- Code 9 effects (synthetic): a paused world particle group is drawn at its held state without advancing; a held-weapon swing trail remains drawn without aging; the hero attack-effect group is not driven. On returning to code 8, their normal eligible updates resume.
- Boot field sync (synthetic): interlaced display, and the first vertical blank after the start-up routine begins an even field: the first status call returns 0, the second returns 1 on the next (odd) field, and the spin ends after 2 vertical blanks. A display in any other mode ends the spin after 1 vertical blank.
- Fresh game (synthetic): New Game is accepted on pass 500. The world reset sets the world-enable flag to 0; the start load finishes and sets the post-load flag (the title-start flag is already 1). The first pass in the in-play state is pass 520: its step 14 sets the world-enable flag to 1, and its step 8 did not run the world. Pass 521 runs main loop step 8.1 (code below 9, because the in-play state also set GG+0x04 on pass 520).
- Audio before the title (synthetic): during the logo page and the legal page after boot the title-start flag is 0, so the sound-cue pool and the music mixer do not step. On the first title pass the flag becomes 1; with the world-enable flag 0 and code 9 the audio part runs from that pass on.
- Flag A (synthetic): an exterior replacement load with a fade request is handled first on pass 300: the 6-frame fade-out starts, flag A is set and the code is 7 from the status machine of pass 301. The handler clears flag A, arms flag B and starts the load on the first pass where flag A's accumulator is at least 0.1 s (about pass 307); the code is then 8 until the flag-B six-pass rule completes.
- Door-use latch (synthetic): the selected hero touches a plain door on pass 50 with the latch at 0: the pending-action word becomes 13 and the hero does not cross. The action button is pressed on pass 51: the latch becomes 1. On the next contact pass with code 0 and no fade running, the latch is cleared and the door is used. A world change before that clears the latch, so the press must be repeated.
- Grid object: whatever the controller sticks do, the grid step and the grid draw change nothing on this build.
- Shared six-pass counter (synthetic): flag G is armed with the counter at 0 and no other flag set. Passes 1 to 5 are not busy (counter 1 to 5), pass 6 is busy (counter 0), passes 7 to 12 are not busy: the counter reaches 6 on pass 12, flag G clears on that pass, the 8-frame white fade-in starts and the hero gets request 59.
- Both flags (synthetic): flag B and flag G both set, counter 0, all passes not busy. The counter reads 2 after pass 1 and 4 after pass 2. On pass 3 the flag-B rule adds 1 (5, not above 5, no firing) and the flag-G rule adds 1 (6) and fires: flag G clears, the fade-in starts and the counter becomes 0. Flag B then needs 6 more counted passes (pass 9, if none is busy and its condition holds).
- Exterior replacement (synthetic): a with-fade request on a loaded exterior is handled on pass 300 (fade-out starts, flag A set). On pass 307 the gate passes: flag A clears, flag B arms and the load starts. A without-fade request does the same but its flag-B completion starts no fade-in. A request while the exterior buffer is empty loads on its first pass with neither flag.

## Open questions

Numbers of removed questions are not reused.

- Resolved (R-0141): the reload of W runs W's Init again with file values, rebuilds its characters, objects and items from the disc, drops this visit's un-stored kills, and runs the activate-world and on-enter callbacks only when W becomes current (return-world cases). In the common case (E current, T reloaded), the transition buffer is unloaded inside the movie step, after that pass's draw and vertical blank and before the first movie plays, and rebuilt while E stays current. The reload is posted when the movies end and the transition driver (the last step of that same pass) begins it; the file-I/O pump runs in the end-of-pass apply of the following passes (it is skipped on the pass that plays the movie, because the queue is not empty then). Every one of those later passes updates, and draws, its frame before the pump and the driver run, so a load that completes in a pass has already been drawn without T, and the first pass that can draw the rebuilt T is the one after the completing pass: it updates the loaded second world's regions, then draws that world before E if the world-pairing test passes. So T is absent from the frame in every pass from the one after the movie through the completing pass; nothing in the engine masks this (the hand-off cleared the screen effect, no fade is armed for this load, and no draw is held), so near a seam to T the exterior's edge shows without T for the load time (about as many passes as the read and set-up of T's pack take), and away from the seam nothing changes. On return its characters and other content are rebuilt from the pack (Playback and handoff, "What the reload of W rebuilds"). The player also sees the focus hero reset to idle at his own position, the camera behind him and the music restarted; script sound loops remain stopped. A level script's own fade is the only thing that can hide the gap. **High** for pass ordering and world availability; **Medium-High** for the absence of any mask.
- Resolved (R-0190): kills are stored only by a driver release that is not a game load and by a save (current and second world); the hand-off's release stores nothing, so the movie brings back W's kills made since W was loaded (High; see "Kills" in "What the reload of W rebuilds"). Scripts' own sound loops are the voices that are started once and only refreshed afterwards: LoopSound voices, a mid-play looping animated-object cue and creature keep-alive cues (Medium for the last); the music restarts from its first child even when the current world is unchanged (High). Default if unresolved (rewrite choice): as in the default at the end of "What the reload of W rebuilds".

  **Length of the gap (a formula; High for the pass counts, Low for the default drive rate).** T's pack is read as one request: all S sectors of the pack (2,048 bytes each; S is the pack's sector count in the archive directory, so the pack is 2,048 x S bytes), starting at its first sector, as a single drive command. The game does not cut a pack into pieces and sends no second request for it. The read queue moves its head request by one state per pass, because the main loop makes one pump call per pass, and makes none while a movie is queued or while a sound stream slot is reading (SPEC-0008). The request has four steps: queued, started, issued to the drive, done; then it completes. Count passes from the first pass after the movie as pass 1. Pass 1 starts the request. Pass 2 sends the drive command. From pass 3 on, each pass asks whether the drive has finished; the first pass that finds it finished marks the request done, and the next pass completes it (the completion callback runs, and T's Init runs). One pass is one video field, 1/60 s (SPEC-0007), so the passes after pass 2 that the drive needs are j = max(1, ceil(60 x D)), where D = 2,048 x S / R seconds and R is the drive read rate in bytes per second. The request is done in pass 2 + j and completes in pass 3 + j. T is missing from passes 1 to 3 + j, so the gap is G = 3 + max(1, ceil(60 x 2,048 x S / R)) frames, and T is drawn again from pass 4 + j. Test vector: an instant drive (R very large) gives j = 1 and G = 4, the shortest possible gap (start, send, one poll that finds the drive done, complete). A count of only three queue steps would leave out the pass that sends the drive command.

  **Input R (the one value the data cannot give).** R is the speed of the disc drive and has to be chosen by the rewrite. Default (rewrite choice, Low): R = 4,000,000 bytes per second, with seek time included; a faster drive only shortens the gap, down to G = 4, and the formula needs no other input. If a sound stream is reading during some of these passes, each such pass does not pump and adds one pass to G; the default assumes none (Medium for the rule, since how long the restarted music keeps its stream reading is not fixed by the data).

  **Worked examples (R = 4,000,000 bytes per second).** Pack 1300, the return world of movie 85 and of the act 2 movie chain (S = 3,839, 7,862,272 bytes): D = 1.966 s, j = 118, G = 121 frames (2.02 s). Pack 1411, the return world of the act 4 intro chain (S = 1,861, 3,811,328 bytes): D = 0.953 s, j = 58, G = 61 (1.02 s). Pack 1220, the return transition for slot 131 (S = 908, 1,859,584 bytes): D = 0.465 s, j = 28, G = 31 (0.52 s). Over all 191 world packs, the smallest (46 sectors) gives j = 2 and G = 5, the median (1,530 sectors) gives j = 48 and G = 51, and the largest (5,924 sectors) gives j = 182 and G = 185. The same count applies to the reload of any pack; it is a draw gap for T only when T is the pack reloaded and it would be in view (near a seam).

  Default if unresolved (rewrite choice): draw nothing for T in passes 1 to 3 + j, add no fade and do not keep drawing the old T.

- Resolved: **Other boot work placement.** The main boot function runs the table walk, the save slots, the save-slot pointer, the frame setup, the game-globals setup, the dialogue command binding, the start-up routine (sound, display and pad, memory pools, start-up loaders, game object, vibration sync, world pool setup), the status spin, the last set-up calls and the frame loop, in that order (Boot order; High).
- Resolved: **Stage meaning.** The table is the static-constructor list. Stage 1 builds each module's static objects; stage 0 is the static teardown, which never runs because the main boot function never returns (High).
- Resolved: **Subsystem names.** Entries 7 to 10, 20, 24, 53 and 56 to 58 are named by their readers: lightning objects, boss and class-38 tables, an effect-pair preset, the hit-pulse preset, the inert-word object, and the strip, trail and segment records of the render code (What each entry does; Medium, Medium-High for 53).
- Resolved: **Index 59.** It registers the C++ exception-unwind tables, a library step and not a game subsystem. It is the first entry the walk calls (High).
- Resolved: **Shutdown.** The main boot function does not return in normal play so the boot tail never runs in play; the rewrite just exits the process when the window closes (no game-visible behaviour; Medium-High). Answers: proactive (open close-out).
- Resolved: **Entries 59 to 3 before the defaults.** None of them reads the live option block. The block's address is formed in one place only, the call inside entry 2 that fills the block with its defaults. Every reader of the live option pointer word is front-end or pause-page code, which runs after the start-up body has set that pointer (Medium-High; calls made through stored function pointers are not followed).
- Resolved: **Stage-0 pointers.** The four data pointers set by the stage-0 part of entry 2's routine are teardown resets of class-table pointers. Stage 0 never runs, and the game does not need them (High).
- Resolved: **Unnamed boot calls.** The status call is a display field sync (one vertical blank per call; in interlaced mode it ends on an odd field). The last initialiser is one early run of the atmosphere-zone pass on GO+0xE0 (Boot order, steps 8 and 9; High).
- Game-over page sprites. Resolved: the game-over page (code 0x1CC, front-end record 23) has two sprites with no animation frames, the game-over picture (resource 594) and the Cross icon beside "Continue", shown after about 2 s (SPEC-0035, Behavior 2, state 26; High).
- **Respawn.** Resolved: there is no save-point or checkpoint respawn handler. After a death the player meets one of these paths, all in SPEC-0055: the default death and the switch to the partner; the game-over page and Continue, which re-applies the last save or load of the session (no card access); the deadly-ground respawn (timer F, state 60, 5 HP less); a level's death callback (sections 5 and 8 there); and world-load placement, which takes a dead selected hero out of the death state without healing it. The act-start setup heals by stage (stage 1 nobody, stage 2 Roberta only, stages 3 and 4 both; SPEC-0053 section 3.7) and a death never calls it. The whole flow is SPEC-0055, section 2A. The checkpoint choice tracked by the region code is not used by any routine (SPEC-0006). High.
- Resolved: **Hero index and level script 1217 test.** Hero 1 is Roberta. In 1217, the Act 3 fight stage runs its single test only when game-state flag 83 is set and flag 278 is clear; Roberta in state 16 without item 50 either causes an immediate SetGameOver(2) with Tor selected or a swap-mode-1 lock that reaches game over through timer E when Roberta is selected. There is no second game-over call or additional health/occupied/no-swap gate (Scripted game-over tests; SPEC-0055, section 4 and Open question 15; High).
- Scripted game over path. Answered: see SPEC-0055 (level script 1432 calls game over once, inside a scripted ending, not a death).
- Game-over effects on the world. Resolved: from the pass after the hand-over the code is 9 and the world is frozen while the page is up (Game over flag; High).
- Pause freeze. Resolved: the in-game menu forces code 9 through the play-active word GG+0x04 and does not touch the world-enable flag (Pause and freezing screens; High, Medium-High for the one-pass lag).
- Resolved: **Act intro gates.** 90 is queued by library door 0 of pack 1327 with flag 240 on and flag 245 off (High). 89 is queued by trigger 1 of pack 1174 once the act 2 start is done, with no flag gate (Movie play sites; High).
- Resolved: **Title idle target.** After the 30 s timeout the title goes to record 30, the Neopets logo page, which leads to the attract (Attract mode and title). **[High]**
- Resolved: **Attract exit target.** The attract leaves to record 10, the title, after queueing movies 0 and 86 (Attract mode and title). **[High]**
- Resolved: **Blank hold.** It is not blank: it shows the Idol Minds developer page, and the publisher page is shown before it while the start-up loaders run (Boot order, visible part; Medium-High that nothing replaces the page during the hold).
- Resolved (High for authored geometry): **Epilogue trigger reach.** The pack-1300 return placement at locator 2 is outside both epilogue trigger volumes. The hero must move into trigger 2 for movie 79, then trigger 3 for movie 83. Runtime trigger cadence after returning from the movies remains unconfirmed.
- Resolved: **Movie skip end.** After a latched skip the player ends as at a normal end and the queue plays the next movie (Movie skip, step 6; High).
- Resolved: **Start-up event registrations.** They are the dialogue command binding: the main boot function, after the game-globals setup and before the start-up loaders, stores six handlers into slots 0x11, 0x12, 0x13, 0x14, 0x15 and 0x19 of the text engine's 48-slot command table (SPEC-0022). Table entry 4 builds the handlers and entry 50 the table (Dialogue command binding; High).
- Resolved: **Movie skip and SPEC-0008.** SPEC-0008 states the player's presentation and end of play and refers to this spec for the skip rule; the two agree (High).
- **Controller map for menus.** Resolved: the full map for the title, pause page, quit box, options, credits, item screen, journal, map, book reader, dialogue box, load and save pickers, game-over page and in-play openers is the table in Controller map for menus and screens. The title starts a game on Cross (0x0040) on New Game from page pass 5; Start (0x0800) does nothing on the title (Attract mode and title). The option-row bits are in SPEC-0035 (section 4). **[High; Medium-High for same-pass combinations]**
- **Option row mapping.** Given in SPEC-0035 (which menu row changes which word).
- Resolved: **Start-point fallback update timing.** While 691 is resident in the transition buffer but another world is current, its level-script step does not advance the Update thread; the non-current world's step advances only the selected special threads (SPEC-0030, Per-tick script step). The Update thread can advance after 691 becomes current, subject to the ordinary world and script-step gates. This settles when the fallback routine can run, not the meaning of its fallback values. **[High]**
- **Button-combination detectors.** Resolved: the game has one live hidden combination and it unlocks nothing playable (High for the boot program). Every pass, after the frame submit and the display wait (step 19): (1) the sequence check compares the newly pressed buttons of controller port 2 with a ten-step table: Up, Up, Down, Down, Left, Right, Triangle, Cross, Square, Circle, one new press per step. A new press that is not the expected button resets the count to 0, a pass with no new press leaves it, and a press that holds the expected button plus others still counts. The tenth step sets a session latch that nothing clears. An L2 press after the tenth step moves the count to 11, where the next new press resets it; the latch stays set. (2) While the latch is set, the same step tests controller port 1: with L3 and R3 both down (held or new) the debug watch screen opens as a blocking loop (SPEC-0008, Debug overlays). It lists no variables in this build; leave it by releasing L3 and pressing R3 anew. Nothing else changes: no item, area, movie, hero, swap mode or story flag is granted. (3) A few calls earlier in the same pass, a new Cross press on port 2 calls an empty routine. A second ten-step detector with the same table shape exists in the boot program (it would print a debug text, set a mode word and then count new Cross presses from 0 to 3), but no code calls it, so it never runs. The in-level debug button combinations (tests of button code 7 and of IsDebugMode) are always false in play, and no code-entry, cheat or password screen exists. SPEC-0033 Behavior 36 has the table and test vectors. Test vector: port 2 presses Up, Up, Down, Down, Left, Right, Triangle, Cross, Square, Circle on ten successive passes: the count reaches 10 and the latch is set; a later pass with port 1 holding L3 and R3 opens the empty watch screen. Without a second controller the sequence cannot be entered.
- Resolved: **Timer F writer.** The death-state handler starts it for a death on deadly ground with a respawn entry index; the entry is that index of the current world entry list (Flags, timers and six-pass rules; SPEC-0055 section 8A; High).
- Resolved: **Flag A.** GG+0x2C, set by the world-load request handler at the start of the 6-frame fade-out before an exterior replacement load and cleared, with flag B armed, once its accumulator reaches 0.1 s (Flags, timers and six-pass rules; High).
- Resolved: **World-enable flag on a fresh game.** The title-start flag and the post-load flag only rise, so after the start load the first in-play pass sets the flag and the world runs from the next pass (Gating of the world update; High).
- Resolved: **Movie-return draw timing.** The world draw and frame submit happen before the movie player (main-loop steps 11 and 16 precede step 20). The movie blocks the loop; the hand-off has already unloaded the reload world W, and on return it posts the reload; the transition driver runs at step 22, after that pass’s draw and submit. Therefore that already-submitted frame cannot contain a draw of the reloaded world. Each later pass draws the current world as usual and adds the second world only when it is loaded, so W (the transition T in the common case) is missing from every frame until its load completes, and the first frame that can contain the rebuilt W is the one after the completing pass; no draw is held and no image is repeated (High; Playback and handoff; Main loop order). Only the load time, and so the visible size of the gap near a seam, is not given by the static data.
- **Movie-entry presentation.** The queue player starts after the pass's render submission and pacing but before the ordinary display-buffer flip. The screen during its pre-play wait remains the last presented image; whether the just-submitted frame becomes visible before the movie's first frame is only decidable on hardware (needs a runtime check). Default if unresolved (rewrite choice): show the last presented frame until the movie's first frame, with no extra fade (Playback and handoff; Main loop order).
- Resolved: **Attack manager limit.** 20 entries; a full list refuses the add and the error report does nothing (SPEC-0038 Behavior 45; High).
- Status timer restore. Answered: see SPEC-0025 Open question 1 and SPEC-0036, Status apply.
- Resolved: **Save-state flags for the quit path**: the quit hold's save-pending flag is the one described under Quit; its name is a label only, behaviour is as stated there. Answers: proactive (open close-out).
- Resolved: **Four-way animation selector**: the per-family selector sets belong to the character specs (SPEC-0036 for heroes, SPEC-0038 for common enemies); this spec states no rule that depends on them. Answers: proactive (open close-out).
- Resolved: **Memory pool sizes**: the pool sizes above are boot-code constants; which pool holds which asset does not affect play, because a rewrite allocates its own storage (Medium). Answers: proactive (open close-out).
- Resolved: **Entry list source (Entering the area).** The 0x50-byte entry records are the type-11 chunk's entry records of the world pack (SPEC-0006, Entry records and teleport planes). Entry record 0 gives the world spawn vector. [High]
- Resolved: **Entry choice.** A replacement (driver) load picks the entry record by the request's plane index, the enter-interior and start-point jobs pick it by entry index (list position in the arrival world), and a new game uses entry record 0 of its start world (SPEC-0006, Entry records and player start; High).
- Resolved: **Reach and the doors of packs without kind-4 rectangles.** The default doors of interior packs 855, 950, 1405 and 1423 return to the exterior already associated with that interior visit, so they add no destination pack. The two flagged kind-7 records in pack 1299 (classes 500 and 501) are same-world puzzle pads; the collision handler routes them to the pad behavior before the door behavior. None of these cases changes the static reach count of 174 packs (SPEC-0006, “Doors” and “Interiors”; SPEC-0028, sections 6 and 13; High).
- Resolved: **Entry routine caller.** The family entries are chosen once per load, at load completion, from the buffer that holds the pack (E: Exterior entries, I: Interior entries, any other: Transition entries); swapping the current buffer calls no family entry again (SPEC-0030, open question 4 and 15, Behavior 1.1; High). Answers: proactive (open close-out).
- Resolved: **Camera cut helpers.** The calls near the end of the scene entry job are the hero placement and camera eye steps already described in Teleport landing: for the focus hero and then for the partner, the job writes the landing pose, clears the hero's motion record and re-attaches it to the ground; then it writes one camera eye (hero position minus 5.0 times the hero forward vector, all three components) through the camera's eye setter. No other camera state changes (no mode, profile or shake change; High). Test vector: hero at (10, 2, 3) with forward (0, 0, 1) gives eye (10, 2, -2); with forward (1, 0, 0) the eye is (5, 2, 3). The later fade-in is 0.3 s (18 frames) in the fade colour requested by the call.
- Order against Area loading. Resolved: the scene entry job is the enter-interior job, not a step after Area loading; it requests the interior load itself, and the load completion runs during its wait (Entering the area; High).
- Resolved: **Scene-entry reload.** The leave-interior job optionally starts the requested pack load into transition buffer T after hero placement, then waits for five consecutive loader-idle passes before fade-in. This is the pack load described in SPEC-0006, “Interior entry steps,” not a separate hero-actor reload. [High]
- World tick order around the entity update. Resolved: the routine with six subsystem calls is the world draw submission, not the world tick; the world tick order is main loop step 8.1 (High).
- Resolved: **Level setup caller.** The level setup runs once at boot, as the last step of the start-up routine (Boot order, step 7; High).
- Resolved: **Two memory totals.** They are different pools: the 461,824-byte asset pool is made at boot by the memory-pool step and the 14,680,064-byte world total is made by the level setup (Memory pools; World memory budget; Medium-High). A rewrite replaces both with its own allocation. Answers: proactive (open close-out).
- Not needed for play (rewrite replaces allocation): **Pool-step arguments** (40, 200 and the 500-unit block) and **world-slot capacities** (200, 1,000, 2,000, 100, 101). They size the original's fixed buffers; no gameplay rule reads them, and the per-world limits that do matter (trigger count, 15 character objects per slot, 20 attacks) are given in their own sections. Rewrite choice: use growable storage, with those limits enforced where stated. Answers: proactive (open close-out).
- World pool reset. Resolved: each world buffer has its own bump pool, rewound to its base when the buffer is released (World memory budget and level setup; High).
- Resolved: **Hero reset.** The heroes are never torn down on an area change; only the selected hero's per-world references are cleared and word +0xF7C of both heroes is set to 0. A driver load resets both heroes' sound slots (What persists across an area change). **[High]**
- Resolved: **Pet respawn.** The pet is untouched by area changes. The former "mode branches" are the four act-start branches, which deactivate it (What persists across an area change). **[High]**
- World inventory. Answered: see SPEC-0021, World items (the world-item pool is per world).
- Resolved: **Lair load path.** The lair is E. A request on E is a forced load into E that always reloads the pack from the disc and never reuses the resident copy (Area loading, step 4). **[High]**
- Resolved: **Hero word +0xF7C.** It is the door-use latch, read and cleared by the door contact handler and by the level-script door hook test (Teleport landing). **[Medium]**
- Resolved: **Teleport landing.** The vector rules, the interior ("other" slot) entry-record rule, the camera eye step and the 6.0 probe start are in Teleport landing, after SPEC-0006 (Hero placement; High).
- Resolved: **Type-34 effect group.** The 56-byte group is filled by the same effect-table loader that builds the shared definition table of resource 9, so its contents have the layout of SPEC-0034 section 2 (six counts, then nodes, definitions and their references); the pack-local groups are small (1, 4, 18 and 6 definitions in packs 906, 1246, 1250 and 1254). The world keeps it beside the type-39 placed-effect list and steps it in the effects pass (Medium).
- Area transition job callers. Resolved: the routine is the hero-swap task, started only by the switch request (Hero-swap task; SPEC-0053 sections 3.2 and 3.4; High).
- Resolved: **Transition job flags.** The two optional flags are SwitchHero's two flags (swap cue and effects; white fade of 37 and 8 ticks), and the mode-4 placement is swap mode 4 (SPEC-0053 sections 3.1, 3.4 and 3.5). **[High]**
- Frame delay count. Resolved: 41 frames, the truncated single-precision product of 0.7 and the frame-rate word (41.999996), held at the end of the hero-swap task (SPEC-0053 section 3.4 step 15; Medium-High).
- Transition order. Answered: see SPEC-0053 section 3.4, steps 6 to 11 (the order of the hero-swap task).
- Resolved: **Boot initialiser subsystems.** They are listed in Boot initialiser, step 2. None of them is a start-up table entry; all belong to boot step 5 (High).
- Resolved: **Boot initialiser caller.** The main boot function, as boot step 5 (the game-globals setup; High).
- Resolved: **Timer scale block.** There is none. The last block the boot initialiser sets is the 36-byte draw-context record (Boot initialiser, step 3; Medium for its role).
- Resolved: **Frame update order and the one-pass steps.** The former sub-step routine is the end-of-pass apply, called once per pass at main loop step 15, after the front end. There is one per-pass order only (High).
- Resolved: **Hero tick and music position in the sub-steps.** There is no music-first variant. The hero is ticked at main loop step 8.1.9; the audio step is the last part of the end-of-pass apply (High).
- Resolved: **Audio flag input.** The I/O-idle flag from the first word of the file I/O record, beside the gate of world-enable, title-start and code 9 (End-of-pass apply, step 8; High).
- **Scene-to-movie mapping.** Resolved: every play call and the pack that holds it is in Movie play sites. Still open, per call:
  - Pack 909, movies 1 and 88: resolved. The opening block of the update needs flag 707 on and flag 1 off. In order it sets flag 1, clears flag 706, queues 88 then 1 and makes the act-start 1 call; the movies play at that pass's movie step in queue order, so the act start is already applied when 88 is shown. Flag 706 means "opening fade-in done". A second block (707 on, 1 on, 706 off, once per module load) repeats only the fade-out, camera and placement. No path tests flag 1 both set and clear (High; Medium for the tick order of the second block on a new game).
  - Pack 1023, movie 4: resolved. The chase thread runs only while flag 136 is off, and 136 is set before movie 4 plays, so movie 4 plays once per game. After a reload with flag 127 on, 136 off and flags 10 and 11 off, the thread is restarted and the chase resumes (High).
  - Packs 1428 and 1382: resolved. The setter slots are fixed: pack 1427's triggers set slots 187 to 191 (packs 1428 to 1432), and pack 1391 sets slot 165 (pack 1382). Pack 1427's eight triggers test no flag, and 1427 is entered only from pack 1432, so they do not gate the story; 1428 is entered by streaming from 1344. Movies 55 to 58 and 91 are gated inside 1428 by flag 322 and then flags 326 to 331 in order (Movie play sites). Movie 70 needs the six pedestals of 1382 lit together; 1391's trigger 1 tests no flag (High for the code).
  - Packs in the cell-exit and no-route groups: resolved. Cell exits are streaming neighbours (World buffers and travel), saves name only resident packs, and every pack of these groups except 678, 686 and 970 can be entered (see Movie play sites).
  - Flag 337 (movies 71 and 72): resolved. Pack 1382 sets flag 337 (with flag 98) with a fixed flag number when the six pedestals are lit, so movies 71 and 72 play in normal play (High).
  - Movie 85: resolved for the object field. The trigger fire step writes it: 1 when the main hero touched the volume, 0 for any other character (SPEC-0028, section 13; Medium-High). Which volume fires the finale trigger is in SPEC-0052.
- Resolved: Cut-scene starts, variant A. Variant A is the hero-swap task of SPEC-0053 section 3.4: it plays the outgoing hero's one-shot cue 0xB1 (step 3) and the incoming hero's cue 0xB0 (step 11), and with the first flag set it also plays sound 0xB0 at the start. Variant B sends hero event 0xB1 (Medium).
- Resolved: Cut-scene wait state. The word leaves 0x33 when the one-shot clip ends (the action-end resolver of SPEC-0040, Behavior 15.4). Variant A waits on the same word twice: step 4 waits while the outgoing hero's state stays 0x33, and step 12 waits until it leaves 0x33 or the fade alpha reaches its target (SPEC-0053 section 3.4; Medium-High).
- Resolved: **Cut-scene fade, variant A.** The SwitchHero task (variant A) runs a white fade-out to alpha 128 over 37 ticks and a fade-in over 8 ticks only when SwitchHero's second flag is set; with it clear, no fade runs (SPEC-0053 section 3.5; High).
- **Cut-scene wait direction.** The routine sets 0x33 and waits while the word stays 0x33 (High for the direction).
- Resolved: **Cut-scene teardown.** The +0x14C value is the hero index, 0 for Tor and 1 for Roberta (SPEC-0053; High).
- Resolved: **Game-state rule 3 and the state gate (conflict).** Both test the focus character's state word at +0x144 (the word the set-state hook writes); they are the same predicate (High).
- Resolved: **Timed transition driver (starting values, helper steps, output consumer, zero lane, cue repeat, cue arguments, scene tests, list walker).** The driver is the bush instance step of the type-16 foliage system; every one of these questions is answered in SPEC-0060 (Foliage patches (type 16), section 6, Bushes). The output is the bush draw scale (lane 3 is the zero lane), the scene tests are once-per-area item bits, the cue is effect 139 and the wait is re-armed after each start (High).
- Resolved for an active in-play exterior replacement load with flag B: after the new world finishes loading, the main-loop draw phase still submits the loaded current world during the six non-busy passes counted before fade-in. The screen-effect overlay is drawn afterward; when the request fade is black and opaque, it covers that world draw. The sixth counted pass starts the fade-in, so subsequent draws become visible through it. This does not generalize to fresh-game, save-load, no-fade, or movie-return paths (Main loop order; Flags, timers and six-pass rules; Loading screen and transition fades).
- Resolved: **State request 0x3B.** It is hero state 59, the arrival state of SPEC-0055 (animation 175 and a one-shot effect), sent after the 8-frame fade-in from white of flag G (Flags, timers and six-pass rules; High).
- Resolved: **Rising ramp timing.** A rising ramp stops only when its value is strictly above the target, and the busy test is value ≠ target, so a sum that lands exactly on the target is done on update n and one that ends just below it on update n + 1. The counts per fade are in SPEC-0052 Test vectors (Loading screen and transition fades; Medium-High for the console rounding).
- Resolved: **World-item per-tick step.** The world-item motion, age and expiry update runs from the draw-phase item submission, immediately before the item transform/draw preparation, and is skipped for game-state codes 9 and higher. It is separate from the item showcase step at 8.1.7. Two draw-side pool walks exist and they split the pool by the item's render-bank word (slot +0x90): the first walk handles items whose bank differs from the current alternate bank, the second (behind a render-path gate that is on in this build) handles items on the alternate bank, and items whose bank word is 0 are handled by neither. So every item is advanced at most once per presented frame, never twice, and the update count is 1 per frame per live item at 60 updates per second (Medium-High).
- Resolved: **World sub-table owners.** The six world sub-tables of main loop step 8.1.1, by world offset, in the order the world tick runs them: +0xA0 (the statue puzzle-object pool of SPEC-0028 and SPEC-0047), +0x29F0 (the drift, leash and radius-separation pool of SPEC-0028, section 15.1), +0x2810 (the secondary pool that the +0x29F0 add routine fills eight entries at a time; its own add routine is called only from that routine), +0x2D40 (plate manager, SPEC-0048), +0x3850 (tween nodes) and +0x4260 (a 0x160-byte model-instance pool of the same shape as +0x29F0, with an add routine that nothing calls). The world draw pass walks the model pools among them. None of the six is ever filled in play, so none has a run-time owner: no host call, script, table or data pointer reaches an add routine, and the only add routine called by another is the +0x2810 one, whose sole caller is the +0x29F0 add, which has no caller itself (SPEC-0028, section 15.1). The guess that +0x2810 holds chain bodies is not supported: chain bodies live in each character's animation player (SPEC-0040, Behavior 25). [High that none is filled; Medium for the +0x2810 and +0x4260 roles]
- Resolved: **Grid object.** The 5 x 5 grid object (GO+0x138, data address 0x35C480) is a debug grid. Its step reads the stick axes of a controller record and moves the grid origin, and its draw shows 25 cells, but both run only when its enable word (+0x00) is nonzero. The constructor sets that word to 0 and no routine writes it, so on this build the step and the draw do nothing and read no input. A rebuild can leave it out (High).
- Resolved: **Front-end page switch timing.** The front end applies returned page records at pass end, so the new record is current from the next pass; its first actual draw can be delayed by the full-screen picture-load gate. Combined with the main-loop status/world work preceding the front-end call, this gives the pause-entry and resume simulation timelines above (SPEC-0035 Behavior 1b, steps 2, 6 and 8; Main loop order; High).
- Resolved: **Other effect lists under code 9.** The world particle group does not step and is drawn frozen; swing trails draw without aging; the hero attack-effect group is not driven while the code is 9 or higher (SPEC-0034 Open question 142; High for trails and the hero group, Medium-High for the world group).
- **Inert words.** GG+0x00 has no writer found other than the step-12 clear. A data word at 0x3A32E4, cleared only by a constructor, would cancel GG+0x04 after the front end; no writer of it was found, so it is inert on this build (Medium). A routine with no caller also clears GG+0x04.

