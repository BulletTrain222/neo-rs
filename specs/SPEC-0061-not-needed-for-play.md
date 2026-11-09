# SPEC-0061: What the rewrite does not need (code and data with no effect in shipped play)

- Answers: proactive, none requested; proactive (verify pass)
- Status: Draft
- Disc build: SCUS-97367 v1.00
- Confidence: per entry (High or Medium-High only; lower-confidence claims in the source specs are left out on purpose)

## Summary

This is an index, not a new source of facts. Many specs say that some routine, state, data block or field exists in the game files but never has an effect in shipped play: it has no caller, nothing places it, a gate can never open, or a debug test is always false. This spec gathers those statements in one place so the rewrite team can plan what to leave out. Each entry names the thing, gives one line on why it never matters, and points to the spec and section that holds the detail and the confidence. Every entry here is High or Medium-High in the source spec. A final section lists things that look dormant but do run, so that nothing live is skipped by mistake.

How to read an entry: "leave out" means that no shipped play path reaches it, so leaving it out cannot change what a player sees or hears. It does not mean the data is absent from the files; the rewrite still reads its own copy of every disc table. Where a spec says a quirk must be reproduced (for example a branch that is reachable but has no visible result), that entry is in the last section, not in the groups above it.

## Where it lives on the disc

Not applicable. Nothing in this spec is read from the disc. Every pointer is to a published spec.

## Layout

Each group is a table with the columns: what it is, why it never matters, pointer (spec and section), confidence. Counts per group are in the Behavior section.

### 1. Cut content

| What | Why it never matters | Pointer | Conf. |
| --- | --- | --- | --- |
| Cloud Beast boss (monster class 9) and its companion (class 10) | No record places either monster type, no script spawns them, no model or animation set exists for them. The cloud wall on the story path is passed with a magic, not a fight. The boss's own attack code (air-flow, rapid, lightning and bouncing-bolt attacks, companion binding) is present in the program but runs only for a placed monster of that class, so it never runs. | SPEC-0010 (Placement and reachability); SPEC-0029 (Class inventory on disc); SPEC-0049 (cloud wall) | High |
| Spyder Nest encounter: nest controller (class 3) and its valves (class 4) | No placement record of the nest or valve types exists in any pack, and no run-time route selects them. | SPEC-0017 (Summary); SPEC-0029 (Class inventory on disc) | High |
| Monster classes that no row places (3, 4, 9, 10 on unplaced rows; 22 and 27 to 31 named by no row) | No spawn path names them, so their behavior code is unreachable. | SPEC-0029 (Class inventory on disc; Behavior 1) | High |
| World packs not enterable from a new game (17 of the 191 world packs, among them the unused copy of one pack that only shares its module) | No door, script load or cell route in the static graph leads to them (an upper bound on reach, so a route that depends on run-time data could in principle still exist). Resources that only they use never load. | SPEC-0006 (Reach); SPEC-0058 (open question 7, pack 1077); SPEC-0066 (contents, classes and load recipes of all 17) | Medium-High |
| Character regions that no level script loads (697, 698, 700, 701, 729, 731, 773, 894, 899, 1325, 1326, 1331 and 1349) and the item-point region 815 | A character region appears only when a level script loads it, and no level script names these; the reached packs that own them play complete without them. | SPEC-0066 (the region sections); SPEC-0006 (Region slots) | Medium-High |
| Attract-loop movie 87 | It is in the movie table but no queue caller ever asks for it. | SPEC-0026 (Movies) | High |
| Outfit entry of item 134 | The item can only go to the general list, so its body-slot outfit offset is never used. | SPEC-0021 (Outfit items) | High |
| Drop sub-table 7 | No drop group names it. | SPEC-0021 (Drop placements) | High |
| Grant codes of the shop event handler | The handler has no caller in the boot program or any level module, so no grant code can fire. | SPEC-0050 (section 7; Open question 10) | High |
| Treasure row 47 (Red Negg of the dig in pack 1230) | The row is a sentinel with no map value, so nothing can dig it up. | SPEC-0051 (section 10, Treasure pieces and the dig) | High |
| Hero emote clips that no request reaches (289 of the 358 positions with data in the hero emote bank) | Only 69 positions are reachable; the rest are never selected. | SPEC-0009 (hero emote reach) | Medium-High |

### 2. Hero states and moves

| What | Why it never matters | Pointer | Conf. |
| --- | --- | --- | --- |
| Sidle (hero state 23) | The availability test always fails, so the Square prompt never offers it. | SPEC-0033 (Behavior 51, State 23) | High |
| Statue turn (state 26) and the statue count | The built-in puzzle-object pool is never filled, so the prompt never appears. The statue puzzle of pack 1353 is scripted. | SPEC-0033 (Behavior 51); SPEC-0028 (section 15.1); SPEC-0047 (Statue puzzle objects) | High |
| Door hold (state 29) | The door index it needs is never set to a usable value, so the prompt never appears. | SPEC-0033 (Behavior 51); SPEC-0028 (grab-door records) | High |
| Scripted jump onto a box | Its only entry has no caller and no data reference. | SPEC-0033 (Behavior 16) | High |
| Left-stick turn and speed path with the reverse codes 10 and 11 ("path A") | The stick routine that holds it is never called. Only the walk-and-turn state's table and the standard speed tiers run. | SPEC-0033 (Behavior 4, Behavior 5.1) | High |
| Soft-landing camera shake (fall height above 4.0) | Any fall above 3.0 already takes the hard landing, so the soft case never gets that tall a fall. | SPEC-0033 (tunables); SPEC-0057 (Kills and falls) | High |
| Jump chain from a soft landing on a Cross press | The animation has no event frame for it for either hero. | SPEC-0033 (input table, Behavior 50) | High |
| Separate "guard" and "evade" moves | They do not exist as moves. The names belong to the hard and soft landings, which no button starts. | SPEC-0036 (Evade variants, Guard and evade triggers); SPEC-0033 (Behavior 50) | High |
| Tor's quick-tap attack branch | Its threshold is 0, so the branch cannot start. | SPEC-0036 (charge section) | High |
| Hero sub-states 5, 6, 7, 8 and 12 launch rows of attack table A | Their clips carry no launch bit, so those rows never launch a shot. | SPEC-0036 (Roberta shot projectile) | High |
| Hero action state 15 | Nothing sets it. It is an empty route. | SPEC-0046 (open question 88) | Medium-High |
| Edge guard for the heroes | The reversed-step guard applies only to generic characters and never fires for a hero. | SPEC-0033 (Behavior 38); SPEC-0040 (Behavior 14E) | High |
| Rumble to pads 1 to 3 | The rumble manager only ever sends to pad 0. | SPEC-0033 (Behavior 1C) | High |
| Aim-view marker object model (the marker is stepped and drawn, but no model exists for it on disc) | There is nothing to draw. Aim mode itself is live; only the marker's visible model is missing. | SPEC-0033 (Behavior 23); SPEC-0036 (Right-stick aim mode) | Medium-High |

### 3. Enemies and bosses

| What | Why it never matters | Pointer | Conf. |
| --- | --- | --- | --- |
| Draik Skeleton stages 2 and 4 and its missile dodge | Neither stage is ever set (the stage goes only to 1, 3, 5 and 6), and the missile dodge runs only in stage 4. | SPEC-0029 (Behavior 6.1, 10.4) | High |
| Block-test push of a melee hit (capped at a length of 40, second facing test) | Nothing calls or points to the routine. The block test that runs is the shield test of the common-enemy strike. | SPEC-0029 (Behavior 10.1); SPEC-0036 (Edge cases) | High |
| Werelupe King big-laugh state | Its request word starts at 0 and nothing ever sets it to 1. | SPEC-0019 (Edge cases) | High |
| Gnarfas block-counter branch | Its flag is only ever written with 0. | SPEC-0020 (Attack selection, Edge cases) | High |
| Gnarfas wait state and jump timer | The wait flag is never set; the jump timer is armed and counted but starts nothing. | SPEC-0020 (Edge cases, Open question 10) | Medium-High |
| Gnarfas chain power attack, with its exhaustion and maniacal-laugh endings | Both phase chances are 0 on disc, so the roll cannot succeed and the attack never starts. The maniacal laugh after a body slam that hits is a different route and is live (see the last section). | SPEC-0020 (Attack selection; Chain power; Edge cases) | High |
| Gelert Assassin shadow thrust-lunge state | No chooser or state writes it. Keep it unreachable. | SPEC-0013 (Edge cases) | Medium-High |
| Gelert Assassin settle test and tracked-point smoothing | The dispatcher that would run them has no caller. | SPEC-0013 (4.14); SPEC-0040 (Behavior 8b) | High |
| Shared tracked-point settle family (two-point settle, mode-0 angle, initialiser, four-point ease) | Same dispatcher, no caller, and no code pointer to it. | SPEC-0040 (Behavior 8b; Open question 119) | High |
| Gelert unused tunable rows (invisible bucket, clone backstab, just-appear mirror and other bucket words) | They are read only by getters with no caller, or not read by the chooser of their pattern. | SPEC-0013 (Open question 13) | High |
| Ixi Chief second lava trigger (HP fraction 0.33) | It never fires. | SPEC-0014 (tunables table, Behavior) | High |
| Faerie Sisters dome-attack damage | Only the dome set-up reads it, and that set-up never runs in the shipped game. | SPEC-0011 (tunables table) | High |
| Plague Serpent tunables that are registered but unused, and its tail-slap state | The covered states never set the tail-slap value, and the unused words are never read. | SPEC-0016 (Tunable values; Tail slap, state 3: not entered) | Medium-High |
| Common-enemy attack sequencer states 10 and 11 | They have no writer. | SPEC-0038 (Behavior 13, Edge cases) | Medium-High |
| Common-enemy leash steer-and-stop branch | The normal choice resets the selected slot before the branch's test, so its test never passes. | SPEC-0038 (Behavior 23, Edge cases) | High |
| Common-enemy hit-count threshold | It stays at -1, so the invulnerable lock never triggers from it. | SPEC-0038 (Behavior 37, step 12) | High |
| Distance-hide gate on approach-4 rows (the spyders) | The gate runs only in outer sub-state 1, which those rows never use. | SPEC-0038 (Behavior 44, Edge cases) | High |
| Monster-parameter schema readers (the debug tuning editor) | Neither reader has a caller or stored address. | SPEC-0029 (Layout, schema readers); SPEC-0022 (Number formatting by category) | High |

### 4. World and objects

| What | Why it never matters | Pointer | Conf. |
| --- | --- | --- | --- |
| Pressure-plate manager pool, its contact rule and completion step | The add routine has no caller. Plates in play are scripted trigger volumes. | SPEC-0028 (section 15.1); SPEC-0048 (Plate manager) | High |
| Built-in statue puzzle-object pool (30 records) and the aligned-statue count | Nothing creates a record, so update, draw and completion never run. | SPEC-0028 (section 15.1); SPEC-0047 (puzzle-object section) | High |
| World object pool with the radius separation step, the drift step and the four-count trigger | The pool is never filled. | SPEC-0028 (section 15.1); SPEC-0040 (Behavior 11A, 25A) | High |
| Tween-node pool step | It never runs. | SPEC-0028 (section 15.1) | High |
| Object crossing solver | Nothing calls it, jumps to it or stores its address. Do not implement it. | SPEC-0028 (crossing solver; Open question 47) | High |
| Grab-door records | No door record on disc has the bit that would build one. | SPEC-0028 (Grab-door records) | High |
| Volume kind 2 firing on the weapon-contact pass, and the 83 cone volumes | That pass never fires kind 2, and every cone has no fire bit. | SPEC-0028 (Open question 76; section 13) | High |
| Pack 1299 puzzle state 6 | At state 5 a crossing to pad B is not teleported, so state 6 cannot be entered. | SPEC-0028 (Pack 1299 looping-door puzzle; Edge cases) | High |
| Lock-remove routine for chests | It has no caller, so no lock entry is ever removed. | SPEC-0048 (Lock remove) | High |
| Kind-12 script group entries of pack 1369 | No callback is registered for kind 12 in that pack. | SPEC-0048 (Edge cases) | Medium-High |
| Inert foliage bush patches (76 patches whose main model is missing) | They have an inner distance of 0, so their logic never runs. | SPEC-0060 (Distances and fade bands; Edge cases) | High |
| Mode-0 kill rule of hazard objects | No mode-0 object on disc ever kills, because their slot flags are never set. | SPEC-0057 (Behavior 2, Slot flags; Edge cases) | High |

### 5. Level scripts and flags

| What | Why it never matters | Pointer | Conf. |
| --- | --- | --- | --- |
| Every branch under the debug-mode test (including the 27 item-grant call sites, which are copies of one debug menu routine with a Neopoint-grant twin; the other 57 grant sites in level scripts are live; and the record writes inside the 22 debug name registration calls in 1098 and inside the eight registrar calls in 986) | The debug query returns 0 on every path, so none of it runs. Only the code under the test is dead: the registration calls themselves still execute and simply store nothing, and code that merely follows a debug-print helper is live. | SPEC-0030 (IsDebugMode; Edge cases); SPEC-0021 (Script grants); SPEC-0045 (1098); SPEC-0042 (986) | High |
| Debug log mode setter | It does nothing in this build. | SPEC-0030 (host call table) | High |
| Per-tick debug entity loops and their button-check helpers | They sit behind the debug test or have no starter. | SPEC-0042, SPEC-0043, SPEC-0045 (module sections) | High |
| The 27 flags with no writer (for example 0, 427 to 429, 496, 521, 522, 647, 648) | They stay off from a clean start and a clean save, so the set side of every test on them never shows. The tests themselves still run and take the clear branch, so keep reading the flags. | SPEC-0049 (Never-true flags) | High |
| Flag 5 of the Darkest Faerie level | Nothing sets it, so flag 4 alone starts the finale. | SPEC-0054 (Finale gating); SPEC-0049 | High |
| Routines that no entry, callback, thread or table names (dormant routines in level modules, for example in 776, 783, 786, 787, 788, 789, 837, 851 and many others) | They never start. They are static facts about each module, so "dormant" always means nothing in the module or the engine reaches the routine. | SPEC-0037, SPEC-0039, SPEC-0041, SPEC-0042, SPEC-0043, SPEC-0044, SPEC-0045 (dormant routine lists) | Medium-High |
| Loads that go through a slot word whose file value is -1 and that no code ever stores | They load nothing. | SPEC-0032 (Slot rules); SPEC-0037 (837); SPEC-0043 (956) | High |
| Item ask-about callbacks | The record search only accepts item ids below 10, and every registration names an id from 57 to 174, so the callbacks never fire. | SPEC-0030 (Item ask-about quirk) | High |
| Interior modules 1429, 1430 and 1431 when streamed into the exterior buffer | They get no Init, Update or Exit there, so their scripts never run. | SPEC-0030 (Interior modules in the exterior buffer) | High |
| Switch handler of the 29 to 32 trigger group (the later registration replaces it) | The second registration replaces the first on all four triggers. | SPEC-0042 (Open question; section on the switch group); SPEC-0048 (Script groups, kind 12) | High |
| Villager group controller thread of 991 | Its three controller words are all 0, so the thread is never started. | SPEC-0045 (991 section) | High |
| Module 1098 enter-to-1170 routine; dead lines 196 to 201 and the flag-194 scene | Nothing references the routine. A talk-counter test the code never stores gates the lines. | SPEC-0045 (Open question 22; 1098 section) | High |
| Update button swaps in 1066 | The button test is constant false because of a true third argument. | SPEC-0043 (Open question 17) | High |
| Cutscene skip checks and the L2 branch in 1217 | They sit behind the debug test, or the button test is constant false. No scene can be skipped. | SPEC-0052 (Skipping) | High |
| Dead DamageHero sites (two in 864, one in 1089) | No registered, started or called routine reaches them. | SPEC-0057 (Host-call inventory) | High |
| Hero-switch request fifth argument (fade in from black, skipping guards) | No caller passes it. | SPEC-0053 (Edge cases) | High |
| Time-of-day tests, the hour and minute getters and the extra calendar-advance helper | They have no caller and are not in the script call table. The calendar itself still advances every pass (it feeds play-time totals), so keep that. | SPEC-0007 (In-game calendar and clock, Who reads the clock) | High |

### 6. Effects and rendering

| What | Why it never matters | Pointer | Conf. |
| --- | --- | --- | --- |
| Remote effect path (owner state packing, send routine) | Only a debug statistics routine with no caller reaches it, and it sends to a host link. | SPEC-0034 (Open question 42; remote path note) | High |
| Twelve-sprite grid effect | It has no caller, code pointer or built address. | SPEC-0034 (section 22) | High |
| Periodic effect counter and the hash range draw built on it | The counter has no caller or table entry. | SPEC-0008 (Random numbers); SPEC-0033 (Behavior 48) | Medium-High |
| Trail object of the trail section | Nothing builds or updates it. | SPEC-0034 (Open question 98) | High |
| Spark hit test for items 1 to 4; texture animator selected by no material; the jitter ramp | The tests never run; no material on disc selects the animator; the jitter never runs in play. | SPEC-0034 (Open questions 90, 106; section 20) | High (jitter: Medium-High) |
| Hero animation clip effect and trail lists | The hero never runs the clip walk. The swing trail comes from the held-weapon path, which is live. | SPEC-0034 (Open question 144, section 3.8) | High |
| Third size rate of one particle family | Its threshold of 2.0 is never reached. | SPEC-0034 (particle section) | High |
| Colour sheet (the debug colour display) | It is never drawn. | SPEC-0046 (Open question 24 to 29) | High |
| Sphere-first draw variant; the variant-B quantizer and darken table | No caller, so they never run. | SPEC-0046 (Open questions 56, 77) | High |
| Vector clip-status loop and transform helper | Neither is referenced anywhere. | SPEC-0046 (Open question 46) | High |
| Fade bit and the 257-entry fade table | The vertex program stores the bit but nothing reads it, and the selector has no caller. | SPEC-0046 (Open question 85) | High |
| Class B level-of-detail flag | Written by the draw and read by nothing. | SPEC-0046 (Open question 80) | High |
| GS fog stage | The game has no fog. Fog is never enabled. | SPEC-0046 (Fog section) | High |
| Widescreen 16:9 projection and 800 by 450 remap | The widescreen flag is never set; its setter has no caller. | SPEC-0027 (Projection) | High |
| Debug camera modes 1, 3, 7, 11 and 12, the smoothed follow (mode 9) and the mode-3 shoulder-height term | No caller passes these modes. The height term's enable field is never set. Implement mode 0, mode 8 and the scripted plays. | SPEC-0027 (Mode routing, Mode entry points) | High |
| Camera mode setter's fallback to mode 2, and modes 2, 4 and 5 | No caller passes them and their bodies are empty. | SPEC-0027 (Mode routing) | High |
| Bounds-checked LZO decoder | Nothing calls it and no address of it is stored. | SPEC-0005 (Decoder section) | High |
| Clip fade-out starts and the layer-1-only rule | No caller for the fade-out starts; layer 1 is never used on disc. (The starts that take an explicit weight or layer have no known caller either, but that is only Medium in the source, so they are not listed.) | SPEC-0005 (Clip start, Playback records and layers) | High |
| Joint pose sample routine | It has no caller and no stored address. | SPEC-0005 (Joint pose sample) | High |
| Animation event pair list (pair count 0 everywhere) | No entry has a pair, so no event is ever raised from it. | SPEC-0005 (Events source) | High |
| Whole-resource expansion step of the loader | No load sets the expand flag. | SPEC-0002 (Whole-resource expansion); SPEC-0008 (Asynchronous read queue) | High |
| 16-bit and 24-bit upload size factor | No such texture format is uploaded on disc. | SPEC-0003 (Open questions) | High |
| Two-slot combination of animation records | Layer 1 is never used, so only the main layer 0 writes the joint list. | SPEC-0005 (Open questions; Playback records and layers) | High |

### 7. UI and debug

| What | Why it never matters | Pointer | Conf. |
| --- | --- | --- | --- |
| Variable list routines (add, reset, save, restore, clear, edit hook) | No callers. | SPEC-0008 (Debug overlays) | High |
| Random-number statistics dump and vector-unit self-test | Developer leftovers with no play use. | SPEC-0008 (Developer and test leftovers) | High |
| Debug colour tuner and its run-time colour table | The tuner has no caller, so the table stays zero. | SPEC-0040 (Behavior 26, item 7) | High |
| Debug colour editor for kind-8 menu nodes | No page has such a node. | SPEC-0035 (Edge cases; Open question 71) | High |
| Start-point chooser screen (container 854, page 858, nine items) | It is marked debug-only and nothing opens it. The new-game and start-point confirm state that its items would target is a different matter and is live through the title screen. The chooser's confirm routine also hands a number per item to an empty routine, which plays no cue and gives no item; only the stage inventory rows of SPEC-0024 table 12 would be given. | SPEC-0035 (Behavior 2 survey) | High |
| Lead-in page stepper and front-end records 15, 16, 21 and 24 to 29 (states 27 and 28) | Nothing enters them. | SPEC-0035 (reach survey) | High |
| Spatial-search menu pages 858 and 896 | Neither page is entered. | SPEC-0035 (Page table; section 9d Focus) | High |
| Menu colour tracks (records 12 and 13) | No animation slot reaches them. | SPEC-0035 (section 17.5) | Medium-High |
| Percent gauge (arc or bar), its label font block and label words | The gauge update has no caller, so no gauge is ever drawn. | SPEC-0023 (section 16; Open questions 34, 35, 37) | High |
| Button prompt slots other than Square | Only the Square slot has a live writer. | SPEC-0023 (sections 30 and 33) | High |
| Option cycler handler (the unlabelled option word) | No screen, item or pointer reaches it. | SPEC-0023 (section 27) | High |
| UI canvas setter, and the 512-high PAL canvas | The setter is never called and the region query always reports NTSC. | SPEC-0023, SPEC-0035 (section 10), SPEC-0008 | High |
| The "show all" journal and map debug switch | The switch word is zero on disc and is not set in play. | SPEC-0035 (section 14.3; Open question 31); SPEC-0051 (Edge cases) | Medium-High |
| Second counter of the movie-skip button detector (threshold 3) | A newly pressed button is always also held, so the counter cannot advance. The first counter (five frames of Cross or Start held) is live. | SPEC-0026 (Movie skip) | High |
| Separate "Positions unlocked" ten-step routine | No caller, pointer or address build leads to it. | SPEC-0033 (Behavior 36; the live ten-step sequence is not this routine, see the last section) | Medium-High |
| Dormant card-transfer state machine, second save routine and unformatted-card paths other than the save scan | Nothing calls them. The reachable save scan offers a format; the load scan shows the no-data message. | SPEC-0025 (Card jobs, open questions 21, 25, 44) | High |
| Save "away" flag | Write-only. No code reads it. | SPEC-0025 (Save points; open question 82) | High |
| On-screen world message host call (DisplayScreenMessage) | Its calls are never reached, so mode 3 text layout never runs. | SPEC-0022 (Text layout) | High |

### 8. Audio

| What | Why it never matters | Pointer | Conf. |
| --- | --- | --- | --- |
| SetMusic host call | No level module imports it and the boot code has no call to it. | SPEC-0009 (SetMusic host call) | High |
| Track selector for music requests (tracks 163 to 166) | Nothing calls it or points to it. | SPEC-0009 (Track selector) | High |
| Sound-bank registers that are never set (seven cases in the soundtrack tree) | Children that depend on them never switch. | SPEC-0009 (Interactive segments) | High |
| Sound flag bits for dry-mix removal, allocator lock and the priority modes | They are not set on any disc entry. | SPEC-0031 (Layout, voice record flag word) | High |
| Sound commands after a stop-type command (for example the record after type 43, and the type-1 record at voice offset 96) | The earlier record ends the cue first. | SPEC-0031 (Test vectors; cue examples) | High |
| Silent fills in 1090 (cue 0 of block 419) | The entry has no commands and its start fraction never fires. | SPEC-0031 (Edge cases) | High |
| Generic C-library random generator and its helper | The helper has no callers and no pointers. The game's own generator is the one used. | SPEC-0008 (Random numbers: the game's generator; Open questions) | High |
| C++ exception unwinder tables and sorts (merge, compaction, interval index) | The unwinder's lookup never runs in play. | SPEC-0008 (Open question on the sorted interval index) | High |
| Static-constructor teardown (stage 0) | The boot function never returns, so no destructor pass runs. | SPEC-0026 (Start-up table) | High |

### 9. Data fields to ignore

| What | Why it never matters | Pointer | Conf. |
| --- | --- | --- | --- |
| Serial number at offset 0x4C of the model header | Not needed. | SPEC-0004 (header table) | High |
| Navmesh grid record upper-corner pair (x1, z1) | Not needed for sampling. | SPEC-0018 (layout) | High |
| Constant-channel animation bytes 2 to 3 and 6 to 11 | Never read by the walk; only the first flag word and the stored value are used. | SPEC-0005 (Animation set, constant channel; Open questions) | High |
| Animation entry bytes 0x25 (fade-out length), 0x26, 0x21 and 0x28 (when 0) | Dormant, always 0xFF, or 0 on disc. | SPEC-0005 (Entry flags) | High |
| Table B event pair count, unused animation event lists | Zero on every entry. | SPEC-0005 (Table B) | High |
| Control codes with no handler (including type 33) | The reveal and draw passes skip them. Keep reading their length byte so the text stream stays aligned. (The type-5 substitution codes are different: the dialogue box replaces them when it opens a text, so they are live; see the last section.) | SPEC-0022 (Control types) | High |
| Second line-break routine (NUL and space only) | No caller or stored address. | SPEC-0022 (Line break search) | High |
| Type-3 slot index values 1 to 4 | Every pack holds 0. | SPEC-0006 (Open questions) | High |
| Sound-track record internals (the 0x17C-byte records) | Library internals, not needed. | SPEC-0008 (Sound-track records) | Medium-High |
| Save flag bits of 193 and above | Never set, always clear; they pass through load and save unchanged. | SPEC-0025 (flag bitmap) | High |
| Duplicate-key rows 167 and 180 of the world key table | The world index always comes from the first match. | SPEC-0025 (key tables) | High |
| Unused design-table size check | The only size-checked load routine is never called. | SPEC-0024 (Load check) | Medium-High |

## Behavior

Entry counts per group: Cut content 11; Hero states and moves 14; Enemies and bosses 18; World and objects 12; Level scripts and flags 17; Effects and rendering 23; UI and debug 18; Audio 9; Data fields to ignore 12. The list of things that look dormant but run has 17 entries.

### Things that look dormant but do run (do not skip these)

Several audits found text or code that looked unused, but the shipped game does run it. Keep all of the following.

| What | Why it looks dormant | What actually happens | Pointer |
| --- | --- | --- | --- |
| Sphere collision pushing characters | The sweep rule blocks without pushing, so spheres seem inert. | A separate contact pass does push characters out of sphere records whose class bits include class 0 or class 1 and not class 2. That covers 5,861 of the 6,678 spheres. | SPEC-0018 (Sphere contact; Layout summary) |
| Camera shake | The soft-landing shake never runs, so shake seems unused. | The script calls start shakes in many modules, including the 35-unit proximity shake in module 864, which is on in normal play. Only the soft-landing case is dead. Each shake start clears the shake record first, so every shake plays in full. | SPEC-0027 (Camera shake presets); SPEC-0057 (section 5, zone helper of module 864) |
| Sneak entry | The sidle and similar traversal prompts never appear. | Square with no target and no other prompt enters the Square stance (state 22); pulling the stick back from there enters sneak (state 39), and releasing it returns to state 22. | SPEC-0033 (Behavior 1A Square rows, Behavior 4A.1, Behavior 5.1) |
| Live hazards in modules 1089, 1090 and 1381 | A DamageHero site in 1089 is dead and their gates start at 0. | Gates are set later in play. 1090 damages 20 HP once per 61 ticks in a firing zone, 1089's crusher thread damages 1 HP and its timeout kills the hero, and 1381 damages 5 HP once its module word is set by the warp pad. | SPEC-0057 (5A) |
| Road-fight AI code 7 | Code 7 is rare in scripts and mostly stands townsfolk still. | The road-fight intro sets the two road enemies to code 7 and then turns aggression on. Code 7 on townsfolk from the disc also binds mode 7 and keeps the current animation. | SPEC-0056 (SetAI_Behavior; on-disc codes) |
| Reed launched-object after-flight fall | SPEC-0028 called it dormant. | It runs for the 24 reed records in packs 909, 934 and 1056. | SPEC-0060 (reeds) |
| Post-fight block of module 989 | Level state 7 is never set by the module. | The block still runs once, because movie 47 returns to world 78 and the area reloads. | SPEC-0012 (Edge cases); SPEC-0041 (989) |
| Hero shot pass firing triggers | The object-0 fire of trigger 1 never sets flag 4. | The hero shot pass does fire script triggers, and the hero contact path sets flag 4. Keep both paths. | SPEC-0054 (Edge cases); SPEC-0028 (Hero shot pass) |
| Level-script modules named in the load-time constructor list | They have no ordinary caller. | The load hook runs them at every load, so they are not dormant. | SPEC-0030 (Module lifetime); SPEC-0041 |
| Modules registered through callback tables in module data (the only disc table of routine offsets is pack 1427's trigger table at module offset 0x408, SPEC-0037) | A routine with no direct reference looks dormant. | A table registration makes it live; a second registration on a trigger replaces the first. | SPEC-0030; SPEC-0042 (switch group) |
| Branches gated by module words that start at 0 | They look unreachable from a static count. | Level code sets those words later in play, so the branches run (for example the 1089, 1090 and 1381 hazards). | SPEC-0057 (5A); SPEC-0049 |
| Reachable branches that have no visible result | A "never" claim can look the same as a quirk. | Some branches run but have no visible result, for example the Ixi Chief counter attack, which reads its damage word but carries no damage window. Reproduce them. | SPEC-0014 (Edge cases) |
| Hidden ten-button sequence and the debug watch screen | The watch screen lists no variables and the sequence unlocks nothing playable. | The sequence check on the second pad runs every pass in every mode. Once it completes, holding L3 and R3 on the first pad opens the watch screen as a blocking loop that shows nothing, and some buttons then hang the game. The source says not to implement the screen or tuner as a feature; the sequence check itself is live, so do not describe the screen as unreachable. | SPEC-0033 (Behavior 36); SPEC-0026 (Button-combination detectors); SPEC-0008 (Debug overlays) |
| Type-5 substitution codes in text | They occur in no line table and no pass draws them. | The dialogue box scans a text when it opens and replaces each such code with the text of its substitution slot (slots 0 and 1 occur on disc). Implement the substitution; only the draw and reveal passes ignore the codes. | SPEC-0022 (Substitution codes) |
| Wild petpets (placed petpet characters) | The statue puzzle-object pool looks like a companion pool that nothing creates, so petpet code can look dormant. | Placed wild petpets run the class 33 loop; feeding one makes it the hero's live petpet. Only the built-in statue puzzle-object pool is empty, and it is not a petpet system. | SPEC-0047 (Acquisition (joining), Placed pet copies, Statue puzzle objects) |
| Gnarfas maniacal laugh after a body slam | The chain power attack that also ends in a laugh never starts. | A body slam that hits is followed by a maniacal laugh when a roll below 30 passes, so the laugh state is live. Only the chain power route to it is dead. | SPEC-0020 (States; Body slam) |
| Registrar calls under the debug test (module 986 and similar) | The registrar stores nothing in play. | Only the stores are skipped. The calls themselves run, and so do the control threads started around them. | SPEC-0042 (986, Init); SPEC-0030 (Edge cases) |

## Edge cases and error handling

- Confidence. This spec lists only High and Medium-High statements. Source entries that are Medium or lower (for example the camera eye-lag path, the Medium-rated parts of Gnarfas, and the break route gated on one breakable field) are not listed here and should be implemented as the source spec describes until a stronger claim exists.
- Static reach. Statements of the form "no routine reaches it" come from the full static call graph of the boot program and all 173 level modules, including table registrations and the load-time constructor lists. A reversal found later in a source spec replaces the entry here.
- Debug-gated code. Because the debug query returns 0 on every path, any branch under it is dead. A call that only follows a debug-print helper is not gated by it and is live.
- Do not delete data. The rewrite still reads every disc table the source specs describe. This index only says which behaviors need no code.

## Test vectors

Not applicable: there is no new behavior here. For the rules this index points to, the test vectors are in the cited specs (for example SPEC-0033 for the path A reference case, SPEC-0013 for the settle rule, and SPEC-0028 for the dormant pool update).

## Open questions

- Dormant level-module routines are counted from static reach. Each module spec lists the routines that remain open because an outside name lookup might start them. Until that is shown to happen, treat them as dormant.
- Each entry should be re-confirmed when its source spec changes. If a source spec reverses a claim, move the entry to the "look dormant but run" list.
- Entries at Medium-High depend on what a full play-through would show. Entries with only a static basis are the ones to re-test first.
