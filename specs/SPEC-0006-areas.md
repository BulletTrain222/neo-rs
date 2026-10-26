# SPEC-0006: Areas, world packs and what they place

- Answers: R-0006, R-0009 (in part), R-0012, R-0032 (in part)
- Status: Draft
- Disc build: SCUS-97367 v1.00 (same fingerprint as SPEC-0001)
- Confidence: per section. Area object lifecycle (constructor, reset, unload; Behavior section): Medium. Entry (locator) records, the world spawn vector, the teleport plane list, the new-game spawn and the placement rule of each arrival (loading step 6, "Hero placement"): High. Area table, world-pack header, region adjacency, character, checkpoint and item records: High. Zone ids: High for the lookup, Medium for the meaning. Type-1001 pack record words (map area id, sound selector) and the default record: High. Region choice per area: Medium (decided by the level script; see Open questions). Grid header: Low. Binary grids (type 35, zone-visibility matrices): High for the layout, the zone manager, the query and the off-by-one quirk. Tagged tables (type 15): High for layout, lengths and walk; Medium-High for the entity-id and placement-id readers; Medium for the region-slot mapping. Box records (type 16): layout High (header-relative, counted on all 75 chunks); the field that selects the build routine Medium; purpose Medium-Low: keys 0 and 12 (the grass patches) also run the builder of the twinkling sprite set whose ring layout and size twinkle are in SPEC-0021 (Ring layout and item draw), and the other keys build foliage only. Effect lists (region record kind 16) and zone records (chunk types 40 and 41): layouts High; their run-time behaviour Medium-Low (range consumer Medium-High). Chunks marked Low below: layout only. Save-point (checkpoint) records and their per-tick step (glow, sound, "Save Game" prompt, save screen): High (code); Medium for what the glow looks like. Loader handling of each chunk type, the required chunk order and the finalize steps: High for dispatch and order; Medium for the pointer kinds in the grid finalize and the meaning of the pool step's arguments. Region slots: High for the slot path, Medium for the wait details; High for the computed-argument rule, the Random range and the asset-swap table rule. Interior entry steps (fade, loader wait, arrival, ground snap): High for the arrival (a teleport plane or an entry record) and the snap; Medium for the wait details. World kinds, world buffers, the buffer records, the pending step, the world swap, the neighbour-world test (both fixed sets and the pair list decoded), the driver loads and the door rule: High (code); Medium for the fade of an exterior driver request; Medium-High for reading the cell value as a neighbour tag; High for the door prompt and the confirm rule. Character activation, streaming and kill memory: High for the steps, the constants, the cell flag, the dead test and the activation filter store; High for the kill-memory store points; Medium-High for the loss of kills in worlds released without a store (until confirmed at run time). Area numbering, conversions and reach (174 of 191 packs, Medium-High): see "Area numbering" for its own confidence labels.

## Summary

An **area** is a **world pack**: one IDM resource whose first chunk is type 31. It contains everything static about the level: materials, textures, the level mesh, placed objects, instanced meshes, a spatial grid, collision/navigation data, lights, a level script module, and an embedded set of characters with their spawn points. The spatial grid's cells point to ground patches and object parts (SPEC-0018). A zone-visibility matrix (type 35) says which zones of this world and of its neighbours are active while the camera stands in a given zone; it gates character updates, streaming and object drawing (High; SPEC-0018, SPEC-0028). Characters that need more models and animations come from **character regions**: the resources stored directly after their world pack. Level scripts name an area by a slot number from 0 to 192. A table in the boot executable has one row per slot, giving the area's world-pack resource number and its world id (see "Area numbering"). The front end and saved games use world ids. Every pack has a fixed kind (transition, exterior or interior), and the game keeps one world buffer per kind. Overworld travel is streaming: the grid cell under the hero names a neighbour, which loads in the background into the other buffer, and the current world swaps when the hero walks onto the neighbour's ground. Doors and level-script calls make the other changes (interiors, replacement loads). Characters of a block are bound to character objects by proximity to the hero, and killed ones are remembered per world (except in worlds released by streaming or by an interior job, which forget their recent kills). With this spec the clean side can enumerate areas, load a level, move between areas, and place its characters, save points (checkpoints) and pickups.

## Where it lives on the disc

- **World packs**: 192 resources whose first chunk is type 31, numbers 532–1433. Resource 532 is the front-end pack ("FrontendData", version 2.0); the other 191 are game levels (version 2.1).
- **Area slot table**, in `SCUS_973.67` (read through the ELF program header; SPEC-0001 has the fingerprint):
  - Table A at virtual address 0x0031EFA8: 193 × u32, each a world-pack resource number. All 193 are world packs. They cover 191 distinct packs, because two packs occupy two slots each. The front-end pack 532 is not listed.
  - Table B at virtual address 0x0031F2B0: 193 × u32 world ids, parallel to table A. They are distinct and ascending, from 0 up to 11070.
  - Read both tables at runtime; don't copy them.
- **Character regions**: the 575 resources that contain a type-36 chunk, numbers 534–1426.

The area-pair table is a boot-program data table at ELF data address 0x31EEE0 (inside the file image; file offset = virtual address minus the image base plus 0x80). It holds 25 entries of two 32-bit pack numbers, eight bytes each, back to back (High). Entries 0 to 22 are pairs of non-zero pack numbers (23 pairs, all below 2,000), and entries 23 and 24 are zero pairs. The values are not copied into this spec. The test that reads the table is in the Behavior section ("Area-pair test"); the pairs link exteriors (see "Area-pair rule"). The current pack is a run-time value, not on disc.

Two more boot-program data tables feed the neighbour-world test (World buffers and transitions): the no-swap list at data address 0x31EEA0 (16 u32 pack numbers) and the block-pair list at 0x31EE80 (four pairs of u32 pack numbers, 8 bytes per pair). Read both at run time. The transition-pack test (51 packs) and the swap slot set (92 slots) are decision rules built into the code, not data tables; their members are stated in that section.

## Area numbering

Confidence: Medium-High for the numbering and conversions; High for the table values and for the slot reading of transition arguments; High for the area-name lookup, Medium for the fallback labels of packs with no map area; High for the new-game load, Medium for the other start readers; Medium-High for the reach count (see "Reach").

**Area numbering.** The game names an area by a slot number from 0 to 192. The area table has 193 rows, one per slot. Each row holds two values: the world-pack resource number (the pack that holds the area) and the world id (a number from 0 to 11070, ascending by row). Level scripts name areas only by slot number. The front end and saved game data name areas by world id.

**Table layout.** Both tables are arrays of 193 little-endian u32 values, 4 bytes per row, so each table is 772 bytes. Table A (pack) is at data address 0x0031EFA8 and table B (world id) at 0x0031F2B0. The row index is the slot. Three rows, for checking a reader: slot 0 holds pack 678 and world id 0; slot 5 holds pack 691 and world id 11; slot 68 holds pack 909 and world id 602. Slot 107 holds pack 1121 and world id 1362. The last row (slot 192) holds pack 1433 and world id 11070. The full table is not copied here; read it at runtime.

**Converting.**
- Slot to pack: read the pack value in that row.
- Pack to world id: find the pack's row and read the world id in the same row. Two packs occupy two rows each: pack 1391 (slots 166 and 167) and pack 1414 (slots 179 and 180). The two rows of each pack carry different world ids. The lookup returns the first matching row, so the lower slot (High). A pack with no row has no world id.
- World id to slot: find the world id value; its row number is the slot. A world id that is not in the table has no slot.
- Slot to kind: the kind (transition, exterior or interior) is that of the pack in the row (World kinds, under "World buffers and transitions").

**Area names (High for the lookup; Medium for the fallback labels).** The name of slot s is the name of its pack's map area. Read the s32 map area id m at payload + 0x14 of pack A[s]'s type-1001 chunk (pack record +0x04, "Zone id" below). When m is 0 to 76, the name is the text that the first word of row m of design table 5 in resource 533 points to (SPEC-0024; SPEC-0051 Layout). Of the 191 game packs, 124 have such an id and 67 have -1, so they have no name of their own. For those 67 the game shows the previous location, or a map area that a level script registered (SPEC-0051 section 1). A label for one of them, where a tool or a debug list needs one (the game never shows it), is chosen by the first rule that applies:
1. The map areas that the pack's own level script registers. Packs 685, 696, 1098, 1122 and 1255 do this, for example 1098 registers 48 and 49 (Meridell Castle 1st Floor and 2nd Floor).
2. For an interior: "inside X", where X is the name of the exterior from which it is entered.
3. "joins X", where X lists the names of the packs that its cells, doors and script transitions lead to.
4. Otherwise no label. This applies to a few packs, among them the module-less transitions 718, 720 and 721, and packs 1259 and 1323.
Names are disc data: read them at run time and do not copy them into code.

**Transitions take slots.** A script call that enters an interior, leaves an interior, starts an exterior replacement load (with or without fade), sets the return world or the return transition after a movie, or sets the pending slot takes a slot number as its world argument. The game loads the pack stored at that slot. Scripts do not name packs or world ids in these calls. The level modules contain 194 structural world-taking call sites in 66 modules. Every argument is a fixed slot number from 0 to 192, except in one helper of module 1427, whose callers pass 187, 188, 189, 190 and 191. A call is live only when the module receives the matching buffer-family lifecycle. In the general live-call census, the 44 enter calls run in exteriors and name interiors, the 21 leave calls run in interiors and name a transition, and the 98 exterior replacements and 12 return-world calls go from an exterior to an exterior. Exceptions include the door callback of transition 706, which the host refuses while the transition is current, and the Interior-only modules of 1429, 1430 and 1431, which receive no lifecycle callbacks when streamed into E (Edge cases). Confidence: High for the family rule and the stated cases; the full reach graph is not recalculated below.

The pending-area setter (SetWorldLoadOverRide) is also a transition call. It stores its argument as the current world's pending slot only when the argument is below 193; a larger argument is ignored. Its calls set pending slots 131 and 165 directly, and 187 to 191 through module 1427's helper, which that module's eight trigger callbacks call (triggers 0 to 7 give slots 187, 188, 188, 189, 189, 190, 190 and 191; the helper sets the slot at most once per tick). Those eight triggers (indices 0 to 7) are rectangle volumes with the moving test (flags 0x0020, switch value 71.02, above zero; SPEC-0028), stacked by height in a stairwell: their reference corners lie at y 2.6, 23.8, 55.8, 87.7, 158.8, 190.7, 214.3 and 246.2, alternating between two faces at z -390.9 and -330.9. Climbing past them requests packs 1428, 1429, 1429, 1430, 1430, 1431, 1431 and 1432 in turn. Their callbacks test no flag and no object value; contact alone fires them (High for the volumes and the calls). 1427 is entered only from 1432's cells, so these requests happen only after the final fight has been entered (Medium). One call with 861 has no effect. The slot acts only through the pending step (World buffers and transitions), so from an exterior it can only name a transition and from a transition only a non-transition (High).

**Default start.** A new game and the title state load pack 691 (slot 5) into the transition buffer and pack 909 (slot 68) into the exterior buffer, and make the exterior current (High). Both new-game loads are synchronous: each finishes before the next step. Both heroes then spawn at 909's world spawn vector, which is 909's entry record 0 (Hero placement, New game; High). The front-end load posts a replacement of the exterior with slot 68 unless a save is applied (Medium); a save load puts its two saved world ids back into the transition and exterior buffers (World buffers and transitions, Other routes). The two start-key words (the chosen-slot word at data address 0x2D7560 and the word beside it at 0x2D7564) hold -1 (none) in the boot program's data. The only stores to them in the whole boot program write -1 (on a new game and on the title state), and no level module refers to them, so both are always -1. The start-point chooser's indices only set story flags (SPEC-0049). So no start enters a pack other than 691 and 909 (High).

**Reach (Medium-High; static upper bound).** From the new-game start (pack 691 in the transition buffer, pack 909 in the exterior buffer, the exterior current), the static graph reaches 174 of the 191 game packs. It uses every mechanism in "World buffers and transitions (area exits)": cell streaming with world swaps, doors (the default rule and teleport callbacks), and structural level-script calls (enter and leave an interior, exterior replacement, the return world and return transition after a movie, and the pending setter). It applies the buffer, kind, area-pair and neighbour-world rules. It does not test story-flag gates or whether a streamed neighbour's ground can be walked onto from the current world, so this remains an upper bound for play. The graph excludes three leave-interior edges from 1429, 1430 and 1431 because those modules are resident in E after streaming and receive no lifecycle calls; their interior-buffer source states are unreachable from the new-game start. Removing those edges leaves reach at 174 packs and the script-call-only closure at 5 packs. The graph has 382 distinct pack pairs, including 18 leave-interior pairs. Streams and doors alone reach 61. Without the default door rule the count is 171: packs 784, 859 and 1251 are entered only by default doors. Applying the neighbour-world test changes no count (174 with and without it).

The transition graph has 382 distinct pack pairs after excluding the three inactive leave-interior edges: streaming 192, default doors 39, exterior replacement 82, enter interior 45, leave interior 18, return after a movie 13 (12 return-world and 1 return-transition) and pending setter 12 (7 with a fixed slot and 5 through module 1427's helper). Every pack with a movie-playing site is reached except the byte copies 678, 686 and 970.

The 17 packs that cannot be entered, each with its reason:
- **Duplicates.** 678 and 686 carry the same payload as the start transition 691, and their only in-edge is from 684. 970 is a byte copy of the module of 956, shares 956's zone id and is paired with it; no table names it.
- **Zone variants.** 1077 is an exterior with the zone id of 1129, paired with 1023 and 1080, not swap-enabled, and named by no table. It is an unused copy of 1129: the two share one level module, which is written for 1129's slot (109). That module reads the blackboard entries keyed by slot 109 and loads slot 90 (pack 1023). No script call names 1077's slot (95). The main-story scene that 1077 seemed to hold runs in 1129 (SPEC-0058). Its exclusion does not change the reach count (High). 1323 has no module and the zone id of 1427; its cells lead only to 1344.
- **Cut variants of the 909 surroundings.** 1259 (with a module) and 1298 (without one) are exteriors paired with 909 whose cells all lead to 909, but 909's lists never name them. 1298's default door asks for arrival door 2 of 909, which has only doors 0 and 1.
- **Cut interior.** 1170's only entry is an enter call in a routine of module 1098 that is never registered. Door 3 of 1098 is registered to a callback that only shows a level message (Medium-High that the door was replaced by the message).
- **Closed pair.** 683 (a transition with no module) and 1003 (an exterior with a 392-byte stub module) lead only to each other.
- **Named by nothing.** 684 and 720 (transitions with no module), 742 (a transition whose module is a text twin of 732's), 879 (no module), and the transitions 1004, 1055 and 1082 have no in-edge from any mechanism. 1055 registers a world-swap scene callback that nothing can start.

Pack 1121 has no level module and is reached (a transition streamed from 1098 and 1122).

**Movies.** The front-end attract sequence queues movies 0 and 86 only (SPEC-0026), and no other boot code queues a movie (Open questions).

## World buffers and transitions (area exits)

Confidence: High for the kinds, the buffers, the pending step, the world swap and the door rule (all from the code); Medium-High for reading the cell value as a neighbour tag; High for the door prompt and the confirm rule; High for the door placement and the arrival of the interior jobs; Medium-High for the rest of the interior jobs; Medium for the exterior replacement (High for its arrival rule). Reach is under "Area numbering".

**World kinds (High).** Every game pack has one of three kinds: transition, exterior or interior. A pack with a level module has the module's kind, which the module's exported entry points name (transition, exterior or interior entry points): 36 transition, 87 exterior and 50 interior modules. Eighteen packs have no module. Fifteen of them are transitions (683, 684, 703, 704, 705, 713, 717, 718, 719, 720, 721, 1121, 1370, 1395 and 1396); 879, 1298 and 1323 are not. The game's own transition test is a fixed set of 51 pack numbers built into the code (not a data table), and it holds exactly the 36 transition-module packs and these 15.

**World buffers (High).** The game keeps three world buffers: T (transition), E (exterior, called the "lair" in the engine) and I (interior). At most one pack is resident in each buffer, and exactly one buffer is current. A transition pack is only ever loaded into T, and an interior into I. Exterior packs are loaded into E, and so are the interior-kind packs 1429, 1430 and 1431 when they are streamed from 1427. Each buffer has an "other" buffer: making T or I current sets the other buffer to E, and making E current sets it to T. While I is current, E stays resident and T has been unloaded: the interior is loaded into T's memory.

**Buffer records (High).** Each buffer is a world record. The words a rewrite needs (offsets from the world record): +0x00 pack number (-1 when empty); +0x0C loaded (nonzero once the load has completed); +0x10 read in progress; +0x40 to +0x60 the kill block (256 kill bits in 32 bytes, then one spare byte that is copied and zeroed with the block but never set or tested, so it is always 0 in a new game; Kill memory); +0x58E4 in I, the pack T held when the interior was entered; +0x58E8 the embedded character block; +0x6C created-slot count, +0x70 slot capacity, +0x74 the slot records (2,960 bytes each); +0x57C4 pending slot; +0x58FC world id; +0x5900 area slot; +0x5930 to +0x594E the load request (Driver loads). The current and second ("other") world pointers belong to the game object: the second world is E when T or I is current, and T when E is current.

**Prior and active world slots (High).** The game object starts its stored slot values at -1. Its words +0xA4, +0xA8 and +0xAC hold the area-slot number last released from E, I and T, respectively; the corresponding level calls GetPriorExteriorFile, GetPriorInteriorFile and GetPriorTransitionFile return those values. A buffer release or a stream replacement stores that buffer's area slot, except when the release is part of a game load. Word +0xB0 is the previous active area slot and +0xB4 the current active area slot; making a world current shifts the former current slot into +0xB0 and writes the new world's +0x5900 slot into +0xB4. GetLastActiveWorld returns +0xB0. These are slot numbers, not pack ids.

**Load busy (High).** The load-busy word (game-globals record +0x28; SPEC-0026) is written on every pass: it is 1 when any of E, I and T has its read flag set, or when its embedded block or any of its allocated slots (every slot up to the capacity, not only the created ones) has its read flag set; otherwise 0.

**Make current (High for the steps; Medium for the roles).** When a world becomes current: if the world changes and the old current world has a script, the old script's leave step runs. The previous and current slot words are updated and the second pointer is set. The zone manager is re-set to the new world's type-35 chunk, with key = its world id and row = the zone of the hero's ground entry (Binary grids). The world's sound set and its fog colour row (by world id) are applied. When T becomes current, I gets its release request. Finally, when the world changed and the new world has a script, its activated step runs with the slot.

**Pending step (High for the steps; Medium-High for the reading).** Each tick the hero's update takes the cell value v of the grid cell under the hero: the low part of the word at cell record +0x18 (SPEC-0018, cell record table; 0 to about 11 on disc, most often 1 to 4), or -1 when the hero's ground-record slot holds no cell (SPEC-0018, "Hero ground-record slot"). It then runs the pending step on the current world:
1. If the current world's pending slot is 0 to 192, the target slot is that slot. Otherwise the target world id is entry v of the current pack's tag-0x0B list (-1 means stop), and the target slot is that world id's row in table B (no row means stop).
2. The target pack is table A at that slot. If the current pack and the target form a listed area pair, in either order, stop.
3. If the target is a transition and the current buffer is E, start an asynchronous load of the target into T. If the target is not a transition and the current buffer is T, start one into E. A load starts only when the destination buffer is not loading and does not already hold that pack. In every other case (I current, a transition target while T is current, a non-transition target while E is current) nothing is loaded.
4. In every case the pending slot is set back to -1 at the end.

So the cell value names the neighbour to keep resident while the hero stands there. It is not an exit trigger, and the step has no hero-side gate. A pack whose cells all hold 1 has one neighbour. Value 0 names no neighbour, because position 0 of the list is never used.

**Neighbour list (tag 0x0B).** Each pack has at most one tag-0x0B table (Layout, tagged tables). It holds N values, one per position. A value is a world id, or -1 for none. N is 191 in 144 tables and 192 in one; this is the table's fixed length, not a count of neighbours. The list is 1-based: position 0 is never used (0 of 145 tables), and in every table the used positions are exactly 1 to m, with no gaps. Position k is read with the tagged-table lookup rule, so it returns -1 for k at or above N. Of the 27,696 values on disc, 528 are world ids. Used positions by index: 145 at position 1, 68 at 2, 51 at 3, 46 at 4, 39 at 5, 33 at 6, 25 at 7, 19 at 8, 14 at 9, 8 at 10, 7 at 11 and 7 at 12. A world id can recur in many tables (world 1320 occurs 26 times), because many areas border the same hub. 107 of the 190 packs with cells hold one value on at least 95% of their cells (62 on value 1, 45 on value 0).

**World swap (High for the steps; Medium-High for the reading).** When the hero's ground search finds no ground in the current world, the current world is T or E, and the neighbour-world test passes, the search is repeated in the other buffer. A hit there makes that world current on the next frame, with the hit as the hero's ground record. This is how the hero walks between a transition and an exterior without a load screen.

**Neighbour-world test (High).** The same test decides whether the second world is drawn together with the current one and whether a world swap may happen. Input: a pack number q. Let c be the current pack and s the current world's area slot (world record +0x5900).
1. If c is one of the 16 packs of the no-swap list (data address 0x31EEA0; read them at run time), the result is 0.
2. If c is not a transition (the 51-pack set of World kinds) and s is not in the swap set, the result is 0. The swap set is a fixed set of 92 area slots built into the code (not a data table): 20, 22, 29 to 32, 34, 41, 43, 47, 50, 52, 60, 62 to 65, 68 to 70, 74, 75, 77 to 81, 85, 88, 90, 92 to 94, 96, 98, 99, 101, 103 to 106, 108, 109, 114, 119, 122, 124, 126 to 130, 133 to 141, 143, 147 to 151, 153 to 155, 157 to 161, 163 to 166, 168 to 170, 173, 174, 178 to 180 and 187 to 191.
3. If some pair (a, b) of the block-pair list (four pairs of u32 pack numbers at data address 0x31EE80, 8 bytes per pair; read them at run time) has (a, b) = (c, q) or (b, a) = (c, q), the result is 0.
4. Otherwise the result is 1.

In effect every exterior allows the swap except 19: the 15 exteriors of the no-swap list, and 906, 1003, 1077 and 1323, whose slots are outside the set. Those 19 exteriors can be left only by doors, script calls or a return after a movie. The only interior that passes step 2 is 1008 (slot 85); the interior-kind packs 1429 to 1431 also have swap-set slots, but they load into E.

Callers (High). The second world's character draw, its world draw and its placed-object draw pass q = the second world's pack, so the four block pairs do stop those pairs of worlds from being drawn together. The ground search of the world swap passes q = the pack of the world it searched, which is the current world for the hero, so for the swap the pair rule never matches: the block pairs never block a swap.

What runs for the second world (High):
- Its region pass (character updates, streaming, kill bits, its sound emitters) runs every tick whenever the second world is loaded. It uses no neighbour-world test and no interior check, so it also runs while I is current (the second world is then E).
- Its placed-object draw runs when the current world is not I, the second world is loaded and the test passes.
- Its world draw and its characters' draw run when it is loaded and the test passes.
So while interior 1008 is current, the exterior E's world and characters are drawn beside it, but not E's placed objects.

**Doors (High for the rule, the prompt and the confirm).** A door is a static rectangle collision volume (header kind 7; SPEC-0028) with flag bit 1 (0x0002) set in its 0x20-byte record header and a class (header +0x1E) other than 500 and 501; a bit-1 record of class 500 or 501 is a same-world pad, not a door (Teleport plane list). Its door number is the header u16 at +0x1C. At load the loader writes that word with the door's ordinal among the world's non-class bit-1 plane records, which is its index in the world's teleport plane list (Entry records and teleport planes), so a door number is a plane index; on the disc the written value equals the disc value for all 169 doors (High). Its gate mask (header +0x1A) is 0 on every door; a nonzero mask would have to share a bit with the world's gate word. On disc there are 169 doors in 102 packs (one in 77 packs, two in 12, three in 6, four, five and six in one pack each, eight in 3, eleven in 1), all with flags exactly 0x0002, mode 0, gate mask 0 and switch value 0.0 (High). When a character touches a door (inside the volume's proximity test, SPEC-0028), the touch test is the plane test of the primitive contact routine with margin 1.5 and the owner's radius times 3 (High for the limits): the character's feet point (its motion-record position) touches when its distance to the door plane is at most 3 times its radius (3.525 for a hero), its position along the plane lies within the door's width extended by that distance on each side, and its height lies between 1.5 below the door's base and the door's height. The body centre and the probe point are not tested. The door step then checks, in this order (High for the order):
1. the game-state code is 0 (no game over is running);
2. no screen fade is running;
3. the toucher is the selected hero;
4. no enter-interior or leave-interior job is pending on the current world;
5. if the current world has a teleport callback registered for this door number, the callback decides, and the step ends there (124 of the 169 doors, counting registration sites regardless of their conditions; Medium-High). Let c be the context registered with it (RegisterTeleportCallback, SPEC-0030). When c is above 0 and below the table-13 label count (36), the step posts c as the door prompt id and starts the callback as a script thread only on a tick that has a Square press edge and the door-confirm flag at 1 (both come from the same press; see Door prompt and confirm below). When c is 0 the callback starts on contact, with no prompt and no press. The callback path never clears the confirm flag (Edge cases, stale confirm). On disc nearly every registration uses context 13 ("Enter"); a few use 3 ("Examine"), one uses 26 ("Exit") and a few use 0. The callback receives the collision block with the door number and the hero flag set (SPEC-0030, RegisterTeleportCallback);
6. the hero is over a grid cell (its ground-record slot holds a cell);
7. no interior job is pending on the exterior world, when that world has a script;
8. the hero's door-confirm flag is 1. The step then clears the flag to 0 and the default rule below runs at once. Otherwise the step posts the door prompt id, 13 ("Enter") while E is current and 26 ("Exit") while T or I is current, and nothing else happens (High).

**Door prompt and confirm (High).** The hero has a door prompt id (−1 at start) and a door-confirm flag (0 at start). The prompt ids are table-13 labels (SPEC-0024; SPEC-0022, Interaction prompts). Within a tick the hero update runs first: its interaction chooser reads the prompt id, and the hero update sets the id back to −1 at its end. The hero's motion-record trigger pass runs after it and holds the door step, which posts the id again while the hero stays in the door. So the chooser sees the id posted by the door step of the previous tick. When the id is 0 or more, the chooser shows that label as the Square caption and, on a Square press edge, sets the confirm flag to 1. It does this only on its idle path (hero not in a special state, game-state code 0) and only when no talk candidate claims the press and no buyable item is chosen (SPEC-0022). The door step later in the same tick sees the flag, so the prompt appears one tick after first contact, and one press travels on the press tick.

Then the default rule applies (45 doors). Let D be the pack named by the cell value under the hero (through tag 0x0B, table B and table A, as in the pending step), and let e be the current pack's tag-0x0E entry at the door number. e is the arrival door number in the destination world, that is, a plane index of the destination's teleport plane list.
- E current, D a transition: the hero lands on teleport plane e of T (teleport landing), and T becomes current.
- E current, D not a transition and not paired with the current pack: D is entered as an interior (enter-interior job with no plane and no entry named, so the hero lands on entry record 0).
- E current, D paired with the current pack: replacement load of E with D, with plane index e, with "unload all" set.
- T current: the hero lands on teleport plane e of E, and E becomes current.
- I current: the interior is left (leave-interior job with plane e). T is reloaded with the pack that was in T when the interior was entered, and the hero lands on plane e of E. If I is not loaded, the hero only lands on plane e of E.

Landing on plane n of a world also makes that world current; the landing point, facing and ground rule are in "Hero placement" (High). The door step also computes a facing sign: 1 when the door's normal vector (+0x40 of the door record) has a non-negative dot product with the hero's forward direction, else 0. The sign is the reverse flag of the teleport landings above (T or E current): with sign 1 the hero arrives facing the reverse of the destination plane's normal (High). The interior jobs do not use it. On disc, every interior door's cell value names the exterior it is entered from, and its tag-0x0E entry lands on the matching door of that exterior (for example 1131 to door 5 of 1023, 951 to door 1 of 909, 857 to door 3 of 842).

**Interiors (Medium-High).** Entering (by a script call or a door, only while E is current) fades out, unloads T, loads the interior into I in T's memory, and remembers T's former pack. It lands the hero on the named teleport plane of the interior, or at the named entry record (entry 0 when none is named or the index is out of range), snapped to the ground from 6 units above. The script enter and leave calls name neither: all 28 enter and 19 leave call sites of the level modules (counted once per module image) pass plane -1 and entry -1, so a scripted entry or exit lands on entry record 0 (High). It fades in after five idle loader frames (see "Interior entry steps"). Leaving (only while I is current) makes E current, lands the hero on the named teleport plane of E (or at an entry record of E; see "Interior entry steps" for the bound quirk), unloads I and T, and reloads T with the named transition pack (none when the slot is -1). The leave call's slot names the transition to reload, not the destination. The interior is never saved: a game loaded later resumes in the resident exterior and transition.

**Exterior replacement (Medium).** A replacement request on E names a pack and a plane index p. On the next frame all buffers are unloaded (every script call sets the "unload all" option) and the pack is loaded into E. The hero is placed by the post-load rule of "Hero placement" (p = -1: entry record 0; p of 0 or more: teleport plane p), and the neighbours then stream in from E's cells. The script replacement calls work only while E is current.

**Driver loads (High unless marked).** Each buffer holds one load request in its world record: +0x5930 force, +0x5934 pack (-1 = none), +0x5938 plane index p, +0x593C spare (-1), +0x5940 unload-all (1 after a reset), +0x5944 make-current (1 after a reset), +0x5948 game-load flag, +0x594C a 3-byte fade colour. On each pass the driver step looks at T's request first, and at E's only when T has none. An E request while E is loaded starts a screen fade to the request's colour, configured to reach alpha 128 over six frames. The driver takes the request once its separate elapsed-time gate reaches 0.1 seconds; it does not wait for a fade-complete signal. A T request is taken at once (Medium for the E fade gate). Taking a request:
1. The front end and the sound are reset. When the game-load flag is set, the previous and current slot words become the new world's slot.
2. With unload-all (or for the matching buffer), E, then T, then I are released. Each released world first stores its kill bits (Kill memory), unless the game-load flag is set.
3. The pack is loaded into E (an E request) or into T (otherwise).
4. The step waits until no buffer is busy (the load-busy word; the frame blocks).
5. The post-load placement runs (Hero placement): the request is cleared; p ≥ 0 lands on teleport plane p (with fade); p = -1, or a refused plane, takes entry record 0; p ≤ -2 keeps the hero's position and forward. The hero's ground-record slot (hero record +0x174; SPEC-0018) is cleared, so the pending step sees v = -1 until the next cell lookup. The start pose is stored and the hero respawns at it without the heal. With make-current, the world becomes current.

**Stream loads (High).** The pending step's load into E starts only when E is not reading and the pack differs (or the caller forces it): the slot E held is recorded, the 15-entry actor list is cleared, E is released without storing its kill bits, and the asynchronous load starts. A load into T works the same way. The enter-interior job starts only when T is not reading: I's old slot is recorded, T's pack is remembered in I (+0x58E4), T and I are released without storing their kill bits, I takes T's memory pool, and the asynchronous load starts.

**Area-pair rule (Medium-High).** An area pair links two exteriors that are entered from each other by a door, through a full replacement load (for example 758 and 777). In the pending step a pair only stops a useless stream. At a door, a pair selects the replacement load of the default rule. The table and the test are in "Where it lives on the disc" and "Area-pair test".

**Other routes (Medium-High).**
- Return after a movie (High): when a movie ends and all buffers are idle, a driver request is posted, always with unload-all 0. If a return world is set, the request is on the current world, with that pack, p = -1 and make-current 1. If none is set and E is current, the return transition (if set) becomes the return world and the request is on T: with it, p = -1 and make-current 1; without it, T's own pack, p = -2 and make-current 0. If none is set and T or I is current, the request is on E with E's own pack, p = -2 and make-current 0. A buffer that is not loaded gives pack -1, and the request then does nothing. So without a return world the hero keeps its position. All 12 live return-world calls run from exteriors.
- Transition scene: StartTransitionScene(x) acts only when T has a registered world-swap scene callback (packs 797, 908, 1055 and 1254 register one). T becomes current, x streams into E, the scene runs, then E becomes current and E's activated callback runs. No live call of StartTransitionScene exists on the disc, so this path is unused in play.
- Save and load: the saved world id at save record +0x24 is reloaded into T and the one at +0x28 into E; an id with no row in table B is skipped (SPEC-0025). A save names only packs that were resident, so it adds no entry.
- No world change: the map page has no travel (SPEC-0051); hero switching loads nothing (SPEC-0053); game over leads to the front end (SPEC-0055), and only pack 1098 reloads its own world on the hero's death (packs 1176 and 1371 end their death sequences with a load of world 90 or 157; SPEC-0055); RegisterTransitionMap takes a map display id (5 to 73), not a load. Same-world warp pads (classes 500 and 501): one pair on disc, planes 2 and 3 of pack 1299, always built at load (the world word at +0x5814, the display node with id 401 of the type-8 chunk, which only pack 1299 has, gates only the looping-door puzzle set-up of SPEC-0028); they are not doors.

## Layout

### World-pack header (chunk type 31, first chunk of the pack)

| Offset in payload | Size | Meaning |
| --- | --- | --- |
| 0x00 | 4 | u32 major version: 2 |
| 0x04 | 4 | u32 minor version: 1 for game levels, 0 for the front end |
| 0x08 | 16 | Name, ASCII, NUL-padded (all game levels carry the same generic name) |
| 0x18 | 8 | Zero |

A version mismatch is logged and loading continues (SPEC-0002).

### Chunks of a world pack

A world pack is a top-level chunk chain (SPEC-0002). Its order varies (119 distinct sequences), so find chunks by type, not by position. Types 4 (textures) and 16 may appear more than once: one pack has two type-16 chunks. Every other type appears at most once on disc. The loader's order rules are in "How the loader handles each chunk".

### How the loader handles each chunk

The loader walks the pack's top-level chunk chain once, from the first chunk to the end. Each chunk header holds a type (u16 at +4), a payload offset (u16 at +6) and the chunk size (u32 at +8). The payload starts at the header plus 16 plus the payload offset. The next chunk starts at the header plus the size. The walk ends at a chunk of type 0. Each chunk is set up when the walk reaches it. Nothing is streamed: a pack is read once per area load.

Twenty-five types have a handler. Any other type is logged with its type and size and skipped (type 38 appears in one pack).

| Type | What the handler does | Spec | Confidence |
| --- | --- | --- | --- |
| 31 | Checks the header version (2.1 expected). A mismatch is logged, and loading continues. | this spec | High |
| 2 | Makes the chunk the current shader list and fixes its offsets up. A second fix-up runs when a sub-record of the chunk is nonzero (Open questions). | SPEC-0003 | High |
| 3 | Level display tree, world space. At this point in the chunk walk, the loader fixes the tree's payload-relative links against the payload base, supplies the current shader list for material handling, and stores the tree in a display-tree slot. The slot byte in the chunk header selects slots 0–4; a value above 4 falls back to slot 0. The slot is marked populated. The node formats and display behavior are in SPEC-0004. | SPEC-0004 | High |
| 4 | Fixes the texture chunk up and stores it in the next free texture slot of the current shader list, in walk order. A texture beyond the list's expected count is dropped and logged. | SPEC-0003 | High |
| 5 | Model set: a container of entries. At this point in the chunk walk, entry, cell and record offsets are made usable addresses and the frame records are linked to their owning set. Grid entries are ground height patches; part-only entries hold frame records; empty entries hold neither. Entry kinds, header and record layout are in SPEC-0018 (Model-set entries, chunk type 5). Registration in the grid's patch table still occurs during finalize step 1. | SPEC-0018 | High |
| 8 | Fixes a mesh container up against the current shader list. | SPEC-0004 | High |
| 10 | Ladders and climb panels: relocates a table of 32-byte groups and the 96-byte records they point to (Type 10, below). Type 29 reads this table, so type 10 comes earlier in the chain. | this spec | High |
| 11 | Relocates the object header. Then clears one flag byte in each of its 80-byte records. It holds the entry (locator) records and the teleport planes; after the walk the type-11 finalize builds the teleport plane list and the world spawn vector (Entry records and teleport planes). | SPEC-0028, this spec | High |
| 15 | Walks the tagged records to the end of the chunk. Each record's length depends on its tag. | this spec | High |
| 16 | Stores the chunk in the first free of ten slots. A chunk that finds no free slot is dropped without a message. | this spec | High |
| 19 | Relocates the grid. Patch resolution waits for the finalize step. | this spec, SPEC-0018 | High |
| 20 | Keeps at most two instance chunks. A third is logged and dropped. | SPEC-0004 | High |
| 21 | Navigation meshes. Rebases the mesh offset table, and for each mesh its two internal offsets (the triangle array and the bit block), so they become usable addresses. The table and meshes are read by the navigation queries and the player placement on the mesh. | SPEC-0018 | High |
| 28 | Relocates the record and stores it in its fixed slot. | SPEC-0028 | High (dispatch) |
| 29 | Placed objects. Relocates them and reads the type-10 table, so types 5 and 10 must come earlier in the chain. | SPEC-0028, SPEC-0004 | High (dispatch) |
| 30 | Walks N records; each record's size comes from the record. | SPEC-0028 | High (dispatch) |
| 32 | Allocates one character block of 2,960 bytes from the world pool and initialises it. Without a world pool, it is logged and skipped. The block is the same record type as a region slot (same constructor and method table, data address 0x30F720), with three differences: it shares the world pool instead of owning a buffer handle, its region number is −1 (so a record's entity id is its plain ordinal), and its data is the chunk payload already in memory, so no read is made. | SPEC-0005, SPEC-0040 | High |
| 34 | Creates one effect group of 56 bytes from the world pool. Without a pool, it is logged and skipped. | SPEC-0034 | Medium |
| 35 | Relocates the binary grids. After the walk, the first block key of the first grid is set to the world id. | this spec | High |
| 37 | Stores the chunk size and sets up the level script module. | SPEC-0030 | High |
| 39 | Placed effect instances. Each record (0x90 bytes) is bound to its effect definition, found by its effect id in the shared definition list. A record whose id is not found is reset and an error is logged. No objects are allocated at load; the record is the instance. Its active flag is clear on disc, so nothing runs until a script or a switch starts it. | SPEC-0034, SPEC-0028 | High (dispatch and binding); Medium (activation) |
| 40 | A fixed block of 21 records of 0x44 bytes, each with seven pointer fields that are rebased, followed by a counted list of 0x44-byte entries keyed by a switch key. Record 3 of the fixed block gets two runtime blocks. A setup step after the walk allocates a group and registers a script group type. Layout in "Switch-keyed entries" below. | this spec, SPEC-0028 | High (dispatch, layout); Medium (setup); Medium-Low (role of the entries) |
| 41 | A count followed by 0x50-byte zone records. When a record's link count is nonzero, its offset is rebased, and the link count gives the number of 16-byte entries stored after the records. Each frame, the game selects one zone record from the focus position and computes a blend factor (see "Zone records"). Not a trigger volume. | this spec, SPEC-0028 | High (layout); Medium (load steps); Medium-Low (per-frame selection) |
| 1000 | Dialogue line table: relocated and stored. | SPEC-0028 | Medium-High |
| 1001 | Reads the zone id (u32 at payload + 16), finds its row in the level table and stores it. The row's canonical pack number then replaces the first word of the requested pack number. A reference to the pack record (zone id, map area id, sound selector; see "Zone id") replaces the default record. | this spec, SPEC-0051, SPEC-0009 | Medium (lookup); High (pack record) |
| Any other type | Logged with its type and size, then skipped. | this spec | High |

Counts on disc for types 21, 39, 40 and 41 (all 192 packs): type 21 is in 135 packs, with 998 meshes and 52,225 triangles, matching SPEC-0018. Type 39 is in 158 packs with 5,139 records, and every chunk is exactly 0x10 plus its count times 0x90 bytes. Type 40 is in 191 packs, and its list always starts at the same offset (0x59C). Type 41 is in 138 packs with 185 records; 11 chunks also hold link entries after the records. High (counts).

Ordering rules (each holds in every pack that has both chunks; High, verified on disc):

- Type 2 comes before types 4 and 29.
- Type 19 comes before type 4 and before type 35.
- Types 5 and 10 come before type 29. Type 29 needs the type-10 table.
- Type 2 comes before every type-8 chunk. 191 of the 192 world packs have exactly one type-2 chunk followed later by one or more type-8 chunks; the remaining pack has neither. So every type-8 chunk is fixed up against its own pack's shader list.

Caps: the loader keeps at most two type-20 chunks per load (23 packs have one). The ten type-16 slots are filled in walk order (one pack has two type-16 chunks).

Finalize, after the walk, in this order:

1. **Grid finalize.** Each type-5 model set, and each placed object's inline model set, is registered under its patch id in the grid's patch table. Each cell list of the grid is relocated, and each cell-record entry gets its resolved entry pointer: patch id first, then model set, then entry (High for the order; Medium for the pointer kinds).
2. **Activation.** Allocates an activation block of 428 bytes and a block of 64 bytes from the world pool. Starts the sub-systems, and calls the object-table setup with the level values. Calls the type-40 setup when that chunk exists, and the object link when placed objects exist (Medium).
3. **Pools.** Makes two pools from the world pool, sized by the arguments 40 and 200, and one block of 500 units. The meaning of the arguments is open (Medium).
4. **Post-load helper.** Calls the character block's post-load function when the character block exists (High).
5. **Type-35 key.** The world id is written into the first block key of the first type-35 chunk (High).
6. **Statistics.** A statistics counter records the bytes used by step 3 (Medium).

The type-11 finalize runs inside step 2 (Activation), only when the pack has a type-11 chunk: after the activation blocks are allocated and the sub-systems are started, and before the object-table setup, the type-40 setup and the object link. It collects the teleport plane list and the same-world pad list, builds the dormant grab-door records, and sets the world spawn vector from entry record 0 (Entry records and teleport planes; High).

Region slots: a character region is loaded into a slot of the current world. A slot is a fixed-size record of 2,960 bytes in a per-world table. Loading into a slot first tears down the slot's previous content. The load then waits while its read is busy, waits once more, and only then spawns the region's characters (High for the slot path; Medium for the wait details). A slot is torn down by its next load or by a reset. A teardown never returns the slot's buffer to the world pool: it rewinds the slot's own buffer handle to the start of the buffer (cancelling a read still in progress), so the next region loaded into that slot reuses the same buffer from its start. The world pool is an arena with no per-allocation free; the world reset only drops the slot table (created count, capacity and table pointer set to 0) (High).

A level script reserves a buffer per slot (SlotCreate) and then loads a region into it (SlotLoad; SPEC-0030). The reservation reads only sizes from the archive directory; it does not load or queue the regions it names, and nothing loads them for it later. SlotLoad is the only call that reads a character region. The regions an area shows are exactly those its script loads through SlotLoad, so a region listed only in a reservation never appears. When counting a script's loads statically, follow every branch into a load call: in pack 909 one load call receives eight different region indices from eight branches (High).

Which region a slot load names (High):
- A region-slot load in a level script names region P + 1 + j, where P is the running world's pack (the script's own pack) and j is the script's argument (Behavior, Loading an area, step 5). When the pack number is negative the resource is j + 1. The load records j as the slot's region number, which the entity and placement tables match (Tagged tables).
- Most loads give j as a fixed number. Where j is computed at run time, it always comes from one of three sources: a small constant table in the script's own data, a chain of story-flag tests, or an argument of the script thread that makes the load, whose start sites all pass fixed values. No computed load is unbounded, so the full set of regions each script can load is known, provided every branch that reaches a load call is followed (see the paragraph above). The per-script region sets belong to the level-script specs.
- Random(a, b), the random-number call that level scripts use, returns a whole number from a to b, both ends included: a + (r mod (b - a + 1)) for a random value r. Confidence: High.

Asset-swap tables (High; used by several scripts):
- A script's data can hold an asset-swap table. It is made of sets, and each set is a group of zones. A story-flag chain in the script picks the active set; one script always uses set 0.
- The zone comes from the trigger that fires. Each script keeps a registration list of entries, 12 bytes each, made of a key, a zone index and a mode. Mode 0 means the key is a script trigger number; mode 1 means the key is the member id of an instance trigger in script group 0. When a trigger fires, the zone of the entry whose key matches is used; when none matches, zone 0 is used (High):
  - Module 758: seven entries at byte offset 32392. Trigger 11 gives zone 4 and trigger 12 gives zone 5. Member 0 gives zone 1, member 1 gives zone 0, member 2 gives zone 2, member 3 gives zone 3 (the cycle zone) and member 4 gives zone 6.
  - Module 991: three entries at byte offset 23752. Trigger 4 gives zone 0, trigger 5 gives zone 1 and member 0 gives zone 2.
  - Module 1023: eight entries at byte offset 71552. Triggers 27 to 34 give zones 0 to 7 in order.
- Firing rule (High): the zone trigger acts only when the trigger (or its member) is in state 1 and is active. Other states do nothing. Then the set is chosen: 758 always uses set 0; 1023 uses set 1 when flag 10 or flag 11 is set; 991 uses the flag ladder above.
- Zone load (High for the order; Medium for the cycle repetition). The trigger starts a zone thread with the zone index. A zone whose normal value is -1 loads nothing. A plain zone (cycle mark 0) loads at once in that thread. A cycle zone starts a second thread, which first waits, one tick at a time, while a world or character load is in progress.
  - Slot rule (High): slot 0 when its record is free; otherwise slot 1 when slot 1 is free and enabled; otherwise the thread waits one tick and tries again. Slot 1 is enabled in 758 always, in 1023 only in set 0, and never in 991 (991's set choice clears the enable word on every trigger, before the zone thread runs).
  - Plain load steps: the record takes the zone index, the normal value and the trigger key; the slot is reset (SPEC-0030 row 208), which releases the region it held; the region is the normal value, or the replacement when the replacement test passes (the first flag set or absent and the second clear or absent); the slot is loaded with keep 1 and activate 1 (SPEC-0030 row 205, which releases the slot's previous content again); the record is then cleared and the slot is marked free. So a zone load always unloads the previous region of the slot it picks, and never touches the other slot.
  - Cycle load (758; 1023 follows the same steps): one candidate is drawn with Random(0, 2) when the thread starts; a candidate out of range or empty becomes candidate 0. The slot is reset and loaded with that candidate, keep 0 and activate 1. A busy word stops two cycle zones from loading at once. The thread then stays while the trigger member keeps a nonzero state and stays active. Later passes draw nothing: each later reload uses the next candidate index after the drawn one (2 wraps to 0), and an index that is out of range or names an empty candidate is set to 0 first (Medium-High for 1023, Medium for 758). A later pass whose busy word is zero waits one tick and does not reload. On each pass that reloads, the thread reads the character range of the loaded set, activates each of its characters that is not present, and then waits one tick (Medium).
  - No fade is part of these zone loads. 991 has no cycle zone, so its cycle path is never reached from a trigger.
- A zone row holds a normal j, a replacement j, a third value (unused, -1, in plain zones), two story-flag ids and a cycle mark.
- Plain zone: the replacement is used when the first flag is set (or absent) and the second is clear (or absent); otherwise the normal j is used. An empty zone (normal j of -1) loads nothing.
- Cycle zone: the first three values are three candidate j values. The pick is drawn with Random(0, 2) when the cycle thread starts (Medium for later passes; see the zone-load bullets above), and a picked candidate that is empty uses the first candidate.
- The region loaded is P + 1 + j for the chosen j.
- Where the tables are (High for the layout, Medium-High for the set and zone counts). Each table is an array of 32-bit signed whole numbers inside the script module's image, at the byte offsets given here (decimal, counted from the first byte of the module image). A zone row is six numbers (24 bytes): normal j, replacement j, third value, flag id A, flag id B, cycle mark. A plain zone uses the first number only; a cycle zone uses the first three as candidates (an empty candidate is -1) and has cycle mark 1. An absent flag id is -1.
  - Module 758: one set of seven zones at byte offset 32224. The seven-entry trigger list follows at byte offset 32392.
  - Module 991: three sets of three zones at byte offset 23536. Set 0 holds plain zones with normal j 4, 6 and 7; set 1 holds 5, 6 and 7; set 2 holds 9 and 10 and one empty zone.
  - Module 1023: two sets of eight zones at byte offset 70144 (set 1 begins 192 bytes later). Set 0 holds plain zones 11, 20, 15 (replacement 16) and 7, then cycle zones (12, 13, 14), (21, 22, 23), (17, 18, 19) and (8, 9, 10). Set 1 holds five empty zones, cycle zones (25, 26, 27), plain zone 28 and cycle zone (29, 30).
  - The trigger-to-zone mapping, the slot rule and the load steps are given above .

| Type | Role | Spec | Confidence |
| --- | --- | --- | --- |
| 31 | Header | above | High |
| 2 | Shader list (materials) | SPEC-0003 | High |
| 4 | Texture chunks for that shader list | SPEC-0003 | High |
| 3 | Level display tree, world space | SPEC-0004 | High |
| 8 | Mesh containers for objects. Each child's role id is a global model registry id (the common-object registry: ranges 0 to 97, 98 to 190, 191 to 282 and 283 to 322, index = id minus the range start). Pickups, flowers and effect-model particles look their model up by that id. Role ids on disc run from 0 to 401; the world pack's own children register the ids its objects use | SPEC-0004 (layout); SPEC-0021, SPEC-0028 and SPEC-0034 (the registry and its users) | High |
| 29 | Placed objects with matrices and part records | SPEC-0028 (and SPEC-0004) | High |
| 20 | Instanced meshes (at most two type-20 chunks are accepted per load; see Edge cases) | SPEC-0004 | High |
| 19 | Spatial grid (below); cell records in SPEC-0018 | this spec, SPEC-0018 | High (layout) |
| 1001 | Zone id (below) | this spec | High (lookup) |
| 32 | Embedded character region: the level's own characters, spawns, save points (checkpoints) and items (below) | this spec, SPEC-0005 | High |
| 37 | Level script module ("SNR2" container with a code module); 173 packs have one | SPEC-0030 (container and host interface) | High (container) |
| 15 | Tagged tables (below): lookup and range tables, the entity-id table (0x12) and the placement-id table (0x13) | this spec | High (layout); Medium-High (use) |
| 16 | Box records (below): foliage patches | this spec; SPEC-0060 | High |
| 35 | Zone-visibility matrices (binary grids, below) | this spec, SPEC-0028 | High |
| 39 | Placed effect spawns (SPEC-0028) | SPEC-0028 | High (layout) |
| 28, 30 | Point records and light sets (SPEC-0028) | SPEC-0028 | High (layout); role Low |
| 10 | Ladders and climb panels (below; SPEC-0033 Behaviors 22 and 22A) | this spec, SPEC-0033 | High |
| 11 | Object header counts (SPEC-0028); entry (locator) records and teleport planes (below); the movable-block table (below; SPEC-0048) | SPEC-0028, this spec, SPEC-0048 | High |
| 34 | Effect definition group (the shared definition table lives in the type-34 chunk of resource 9; a few world packs hold small groups of their own) | SPEC-0034 | High (resource 9); Low (the small world-pack groups) |
| 38 | Present in 1 pack; the loader logs it and skips it | — | — |
| 40, 41 | Switch-keyed entries and zone records (below); SPEC-0028 section 11 | this spec, SPEC-0028 | High (layout); Medium (load steps); Medium-Low (run-time use) |
| 21 | Navigation meshes | SPEC-0018 | see that spec |
| 1000 | Dialogue line table (SPEC-0028) | SPEC-0028 | Medium-High |

### Spatial grid (chunk type 19)

| Offset | Size | Meaning |
| --- | --- | --- |
| 0x00 | 16 | 4 × f32: x min, x max, z min, z max of the level |
| 0x10 | 4 | f32 cell size (25 in 53 packs, 8 in 44, other values elsewhere) |
| 0x14 | 4 | s32 cells along x = ceil((x max − x min) / cell size) |
| 0x18 | 4 | s32 cells along z, same rule |
| 0x1C | 4 | u32 offset of the cell table: cells-x × cells-z u32 values, 0 = empty cell |
| 0x24 | 4 | u32 offset of a second table (meaning open) |

Each non-zero cell-table entry is an offset from the grid payload to a **cell record**. Cell records are decoded in SPEC-0018. On disc there are 1,562,362 cells over the 191 grids, and 108,573 are non-empty. Every cell table fits in its chunk. The table is x-major: the entry for cell (cx, cz) is at index cx + cz × cells-x (High; disc test in SPEC-0018, Cell-table order).

The grid bounds are the level's extent: placements fall inside them (see the checks below).

### Entry records and teleport planes (chunk type 11)

Confidence: High (layout from the loader and every reader; counts over all 192 packs).

The type-11 chunk (object header, SPEC-0028) holds two lists that place the hero. Header offsets are from the chunk payload; list addresses are payload-relative on disc and rebased at load, and both lists are used in place.

| Header offset | Size | Meaning |
| --- | --- | --- |
| +0x14 | s32 | Plane record count |
| +0x24 | s32 | Entry record count n |
| +0x3C | u32 | Address of the plane record list (records of 0x70 bytes) |
| +0x4C | u32 | Address of entry record 0 |

**Entry (locator) records.** Entry i is at the list address + 0x50 × i, for 0 ≤ i < n. This one list is the arrival list of loads and interior jobs, the source of the world spawn vector, the locator list of the script calls MoveHero and SetHeroStartPos and of the camera-locator call (SPEC-0030), and the respawn list of deadly ground (SPEC-0055). Each record starts with its own 0x20-byte record header, then a shape tag (the same header precedes the plane and box records; SPEC-0028).

| Offset | Size | Type | Meaning |
| --- | --- | --- | --- |
| +0x00 | 16 | — | Zero (all 4,964 records) |
| +0x10 | 2 | u8, u8 | Common primitive header: kind 1 and mode 0 in every record |
| +0x12 | 2 | u16 | Kind mask of the shared primitive header (the word that the load-time class tests read on plane and box records). Entry records are never tested against it, and no code reads this word on an entry record (Medium-High). 0x0020 in 4,924 records and 0x0021 in 40 (in 16 packs); bit 0 has no effect |
| +0x14 | 4 | u32 | Editor serial: unique within each pack. Not a locator id; not used by placement or lookup |
| +0x18 | 2 | u16 | Index; equals i in every record. Placement uses the list position i, not this word |
| +0x1A | 2 | u16 | 0, except 4 and 8 in two records of pack 699 (which is why the 32-bit word at +0x18 differs from i twice there) |
| +0x1C | 2 | u16 | 0 in every record |
| +0x1E | 2 | u16 | Locator tag: 0 in 4,912 records; a tag in 52 (below) |
| +0x20 | 4 | u32 | Shape tag 0xFFFFFFFF (all records) |
| +0x24 | 12 | 3 × f32 | Radius r, three equal values (0.625 in 3,718 records) |
| +0x30 | 16 | 4 × f32 | Position x, y, z; w = r |
| +0x40 | 16 | 4 × f32 | Facing x, y, z (unit length in every record; y nonzero in 128), w = 0 |

Disc: 4,964 entry records in 179 of the 192 packs. Thirteen packs have none: 532 (front end), 718 and 1082 (no type-11 chunk), and 702, 703, 713, 719, 720, 721, 1079, 1121, 1391 and 1427 (count 0).

**Locator lookup (High).** A lookup by tag t returns the first entry record of the list whose u16 at +0x1E equals t, or none. Boss AI (the Cloud Beast, the Gelert Assassin, the Gnarfas trees and Juppie) and one host call use it. The 52 tagged records: pack 735 (tags 101 to 124), 985 (101 to 104 and 200 to 207), 1056 (101, 102), 1122 (101), 1163 and 1174 (781 each) and 1217 (101 to 111).

This list is the 0x50-byte arrival record that the interior jobs read: an entry record, not the body of a door's rectangle volume.

**World spawn vector (at load).** After the chunk walk, when n > 0: the spawn position S is entry 0's position and the spawn facing is entry 0's facing. Then h = the ground height at (S.x, S.z) probed from S.y + 8.0 (SPEC-0018 ground probe). If h < -1e-7, S.y keeps the record's y; otherwise S.y = h. When n = 0 (or there is no type-11 chunk), the spawn stays at its reset value (0, 0, 0, 1) and there is no spawn facing, so the default facing (1, 0, 0, 1) of SPEC-0040 applies. The world reset also sets the spawn to (0, 0, 0, 1) and the list counts to 0.

**Teleport plane list (at load).** Plane record k is at the plane list address + 0x70 × k (the 0x70 covers the 0x20-byte record header and the plane body). In the record header, the u16 at +0x10 holds class bits, the u16 at +0x12 the kind mask, and the u16 at +0x1E the class. A plane joins the world's teleport plane list when bit 1 of its kind mask is set and its class is neither 500 nor 501 (class 500 and 501 planes, the same-world warp pads, go to a separate list: one pair on disc, planes 2 (class 500) and 3 (class 501) of pack 1299, which get class ordinals 0 and 1 in +0x1C; the loader builds the pad lists from the class-500 and class-501 planes of every pack, with no other condition; the world word at +0x5814 is the display node with id 401 of the type-8 chunk, only pack 1299 has such a node (none of the other 190 game packs does), and it gates only the looping-door puzzle set-up (SPEC-0028), High; class 500 also appears on sphere record 8 of pack 1246, but the class step reads only the plane list, so that word is never used). The list keeps disc order; a plane's position in it is its plane index, the number that loads, doors and the tag-0x0E arrival list use, and the loader writes it into the record header's u16 at +0x1C. The box records are a separate list (header +0x40, count +0x18; SPEC-0028). Disc: 675 plane records; 169 teleport planes in 102 packs (one in 77 packs, two in 12, at most 11). They are the same 169 records as the doors (Doors) (High). Example: pack 684 has 100 boxes and one plane.

Plane fields used for landing (offsets from the record start):

| Offset | Size | Meaning |
| --- | --- | --- |
| +0x24 | f32 | h, extent along Q |
| +0x28 | f32 | w, extent along D |
| +0x2C | f32 | 0 (thickness) |
| +0x30 | 4 × f32 | Reference point P |
| +0x40 | 4 × f32 | Axis F: the plane normal, the "through" direction |
| +0x50 | 4 × f32 | Axis D: the width direction |
| +0x60 | 4 × f32 | Axis Q: the height direction ((0, 1, 0) for upright doors) |

**Deadly ground and respawn entries.** A ground model-set entry (SPEC-0018 entry header) with kind word +0x38 = 2 is deadly ground. Its byte +0x0D is a respawn entry index into this pack's entry list (0xFF = none). Disc: 123 such entries in 50 packs; 79 carry an index, all below their pack's entry count, and 44 carry none. The kill, the death handler and the timed respawn are in SPEC-0055 (section 8A).

### Zone id (chunk type 1001)

The chunk's extra-header length E is 4. Its payload starts with a small "IDMX"-tagged child; the u32 at payload offset 0x10 is the pack's **zone id**. That id appears in table B in all 191 game levels. Looking up the zone id's row in table B and interpretation table A at that row gives the zone's canonical world pack. In 183 of 191 packs that is the pack itself. In the others, several packs share one zone id: variants of one place, such as before and after an event.

When the pack loads, the loader also writes the canonical resource into the first word of the requested world block. Whether any later code reads that word is not established (Open questions).

**Pack record.** The loader keeps a reference to the record that starts at payload + 0x10, the pack record (also called the zone record: the location-name and save code read the same record). Record offsets below are relative to payload + 0x10, so the zone id is record +0x00. Its known words (High):

| Offset in record | Payload offset | Size | Type | Meaning |
| --- | --- | --- | --- | --- |
| +0x00 | +0x10 | 4 | s32 | Zone id (above). |
| +0x04 | +0x14 | 4 | s32 | Map area id, which is also the location index: 0 to 76, or -1. It chooses the area's map page (the hero map area; SPEC-0051 section 1) and the location name, the row of design table 5 (SPEC-0024; the area table of SPEC-0051 Layout) that the save picker shows for a save slot (SPEC-0025). -1: the pack has no map page of its own, and the location keeps the previous value (the previous area's name, or a transition map a level script registered, SPEC-0051 section 1). |
| +0x08 | +0x18 | 4 | s32 | Sound selector, 0 to 11 on disc (SPEC-0009, Area to music row). |

Default record: at the start of every world load the reference is set to a default record in the boot program's data at address 0x2DA3D0, with zone id -1, map area id -1 and sound selector 0. The type-1001 chunk replaces it, so a pack without that chunk (only pack 532 on disc) keeps the default and has no map area (High).

Disc census of the map area id (192 packs): 124 packs carry an area id, 67 carry -1, and pack 532 has no chunk. Areas 36, 39, 50, 51 and 52 belong to no pack; a level script can name a map area for packs with -1 through the call RegisterTransitionMap (SPEC-0051 section 1). Example: pack 678 has map area id 8 (Farm Road).

### Embedded character region (chunk type 32)

The payload is a chunk chain shaped like a character region: one type-36 record table, plus model sets (18) and skeleton/animation chunks (22–27) as in SPEC-0005. 165 world packs have one.

The type-36 record table is a sequence of records. Each starts with a 16-byte header whose u32 at +0 is the **kind**; the record's data starts at record + 0x10. The record length depends on the kind (r16(x) means x rounded up to a multiple of 16):

| Kind | Record length | Meaning |
| --- | --- | --- |
| 3 | 0xF0 | Character placement (spawn) |
| 2 | 0x10 + r16(8n + 4), n = u32 at data + 0 | Animation table (SPEC-0005) |
| 4 | 0x40 | Save point (checkpoint) |
| 8 | 0x40 | Item pickup or drop point |
| 16 | 0x10 + r16(0x50n + 0x10), n = u32 at data + 4 | Effect list: a 16-byte list header, then n entries of 0x50 bytes (layout below) |
| 17 | 0x10 + (u32 at record + 8) | Zero-filled reservation whose bytes the loader hands out at run time (layout below) |

Any other kind ends the walk. All 575 region tables and all 165 embedded tables walk exactly to their end with these rules.

**Kind 3, character placement** (offsets from the record's data):

| Offset | Size | Meaning |
| --- | --- | --- |
| 0x00 | 16 | Position: x, y, z, then w. w is 1.0 in 2,856 of the 3,036 records. In the other 180 records the whole position (x, y, z, w) is zero. All 180 are summon-reserved records (byte 0xB6), whose position and facing are supplied by the summoner when they are spawned (Character activation, streaming and kill memory) |
| 0x10 | 16 | Facing: unit vector x, y, z, then w = 0 |
| 0x20 | 0x20 | Character name, ASCII, NUL-terminated (may be empty). On disc a name has at most 29 characters (30 bytes with the NUL), so every name ends before 0x3E. This is the name an enemy shows (SPEC-0023). Do not read the name as a 40-byte field: the bytes after it are the two words below |
| 0x40 | 4 | Talk colour word: bytes 0, 1 and 2 are red, green and blue; byte 3 is 0x80 and is not used. When the hero talks to this character the dialogue box shows the character's own name and this colour (SPEC-0022). 0x80FFFFFF (white) in 2,389 of 3,036 records; named characters have their own colour, for example 0x8040A0FF (255, 160, 64) for Bridge Builder Brendel |
| 0x44 | 4 | Draw mask. −1 (all bits) in all 3,036 records. A model part whose type-4 layer flags share no bit with the mask is left out (SPEC-0004, Drawing a node tree). If the word is neither 0 nor −1, the character is drawn with the region's mask word ORed with it; with −1 or 0 the region's mask word alone is used. The Draik enemy overwrites the word at run time with its damage stage v (1 to 5): the word is 0, then bit 2 (value v = 1 sets bits 2, 4, 8, 16 and 32; each higher v drops the lowest bit, so v = 5 sets bit 32 only) |
| 0x48 | 60 | 15 colour words w0 to w14, present in all 3,036 records. Each word packs byte 0 in bits 0 to 7, byte 1 in bits 8 to 15 and byte 2 in bits 16 to 23 (red, green, blue: High, because the push stores byte 0 in the first, red, lane of the colour scale); byte 3 is 0x80 or 0x00 and is not used (use alpha 1.0). w0 is 0x80FFFFFF in every record; 1,653 records have all 15 words at 0x80FFFFFF, and there are 275 distinct sets. w1 to w14 are the two skin, two shirt, two pants, hair, hat, two eye, belt, shoe and two spare colours (Medium). The character runtime pushes word j to colour slot j of the character's render object, and a model part takes the colour of the slot named by its material group (group byte 0x27, SPEC-0003). That is the only source of the hue of an NPC's skin, hair and clothes: 1,268 of the 2,402 region records use a slot above 0, and 1,148 of those 2,402 use only slots whose word is white. A client that ignores the words draws the NPC white (SPEC-0040, Behavior 26, item 9). High |
| 0x84 | 2 | u16 default AI mode word. The shared enemy and townsfolk behaviour copies it into the AI mode word (object +0xA34) when the character returns to its default behaviour (one class only tests it for the value 8). On disc: 0 in 628 records, 5 in 760, 6 in 347, 7 in 392, 8 in 883, 9 in 17, 10 in 7 and 12 in 2 (all 3,036). Mode 7 decides the release rule (Release and rebind) |
| 0x86 | 2 | s16 main-skeleton index (SPEC-0005); −1 = none; −5, −6 and −7 select global objects |
| 0x88 | 2 | s16 subskeleton index; −1 = none |
| 0x8A | 2 | s16 face-skeleton index; −1 = none |
| 0x8C | 2 | s16 main-set companion word: read once when the character is built and stored in the animation player as the main set's index word (the loader writes 0 there). No later reader is known (Medium). Disc: 0 in 2,005 records, 1 in 415, 2 in 154, 5 and 6 in 50 each, 25 in 46; 20 distinct values |
| 0x8E, 0x90 | 2 each | s16 companions of the sub and face words. Never read. −1 exactly when 0x88 (respectively 0x8A) is −1, otherwise 0, in all 3,036 records |
| 0x92, 0x94, 0x96 | 2 each | s16 second copies of the three skeleton indices 0x86, 0x88 and 0x8A: equal to them in 3,029 of 3,036 records. Never read |
| 0x98, 0x9A, 0x9C | 2 each | s16 main, sub and face animation words. Read once at character build and kept in the animation object; the only later test is whether the word is −1, meaning the part has no animation set (the main, sub and face clip-start helpers then play nothing). 0x98 is never −1 on disc (0 in 2,590 records, 7 in 135, 14 in 127, 6 in 66, 16 in 54, 2 in 49, 12 in 10, 9 in 3, 8 in 2); 0x9A is −1 exactly when 0x88 is −1 (1,889), and 0x9C exactly when 0x8A is −1 (1,746), otherwise 0 or a small value. The value itself is never used (High) |
| 0x9E | 6 | three s16 animation-table indices: main, sub, face (SPEC-0005) |
| 0xA4 | 2 | s16 model-set index, 0 to 7: the position of the model-set wrapper among the region's top-level type-18 chunks (SPEC-0004, Character regions keep their materials inside the model sets). The wrapper holds the model's shader list and textures |
| 0xAC | 2 | s16 key: a character kind key (for example 619 Flower Monster, 2844 Minion Grunt, 2864 Minion Warrior). A spawn-by-key request (SPEC-0029) looks for it, and a record whose key equals the active petpet's key is not streamed |
| 0xB0 | 2 | s16 drop placement ordinal. −1 = none. A value of 0 or more is the index, in walk order, of a kind-8 placement of the same region (SPEC-0021). On disc: −1 in 1,801 of 3,036 records, 0 in 139 and positive in 1,096. Every value of 0 or more is in range. Read only when the enemy dies |
| 0xB2 | 2 | u16 variant. The monster row used is first[type] + variant (SPEC-0029 section 4). All 3,036 placements on disc have a variant below the row count of their type |
| 0xB4 | 2 | u16 type: the monster type, a row group index (0 to 110 on disc). The type is the one used in the row formula above |
| 0xB6 | 1 | Summon byte. Nonzero (208 records on disc) marks a summon-reserved record: it is never streamed in, and it is bound only by a spawn-by-key request, at the point the request gives (Character activation, streaming and kill memory) |
| 0xC4 | 4 | s32 tick value (60 ticks per second, SPEC-0007): one end of a random duration range whose other end is the word at 0xC8. Enemy behaviour code draws a whole number of ticks uniformly from the range, both ends included, and uses it as a timer. The two ends may be stored in either order (the lower one is taken as the start); when they are equal, that value is used with no draw. On disc: 0 in 2,274 of 3,036 records, 1 in 409; the other values are tick-scale, such as 60, 120, 150, 240, 300 and 480. Confidence: High that the word is read as a range end; Medium for the timer role (inferred from the range draw and the values) |
| 0xC8 | 4 | s32, one word with three readings in the enemy code (SPEC-0038). (a) The other end of the tick range at 0xC4. (b) A selector: values 0 to 4 pick one of five idle or alert codes and animations; any other value (5 and above, or negative) takes the default. (c) A percent chance: a whole number is drawn from 1 to 100, both included, and the guarded step goes ahead only when the draw is below this word. On disc: 0 in 2,627 of 3,036 records, 1 in 41, 2 in 10, 3 and 4 in none, 5 to 1,800 in 352, and −1 in 6. So selector values 3 and 4 never occur on disc, and the main use is the range end: of the 331 records with this word above 4 and the word at 0xC4 above 0, the word at 0xC4 is the smaller in only 153, so the range is unordered. This word is not the monster-row variant (that is the u16 at 0xB2). Confidence: High that it is a separate 32-bit field and for the three readings; Medium for the range end as its main meaning |

The skeleton indices are in range of the record's own region in 1,833 of 2,402 records. Each character region has its own main, sub and face skeleton arrays, populated by that region's corresponding chunks. Character setup resolves a nonnegative index only against the corresponding array of the same region and only when the index is below that region's family count. An index at or above that count does not attach a skeleton through this path; it is not looked up in another loaded region. The 569 records with at least one such index therefore do not imply area-wide shared arrays. Negative main indices -5, -6 and -7 select global objects, and -1 means none (High for the lookup boundary). Kind-3 records cover both enemies and friendly characters. The name identifies the character type.

**Variant example.** Monster type 80 (Minion Grunt/Archer, 14 rows) has HP 50 in rows 0 to 6 of the type and HP 40 in rows 7 to 13. A placement with variant 0 to 6 therefore has HP 50, and one with variant 7 to 13 has HP 40. Here first[type] is the index of the type's first row in the monster table, and "row" counts from that first row. The variant is the only per-placement strength selector found (Confidence: High for the lookup; the other row fields are in SPEC-0029).

**Kind 4, save point (checkpoint)**: the save points where the player saves the game. Data (offsets from data): position (16 bytes, w = 1) at + 0x00, unit facing at + 0x10 (not read by the save-point step), u32 flag at + 0x20, 12 bytes of fill. A region keeps at most eight save points; further kind-4 records are dropped with an error report. On disc there are 88 save points in 60 world packs, 1 to 4 per pack, all in the world packs' own embedded regions (type 32), and the flag word is 0 in all 88. Packs 678, 686 and 691 each have one at the same position. Save points are not trigger volumes, placed objects (type 29) or placed effects (type 39). Confidence: High.

Save-point step (summary; the full rule and its test vectors are in SPEC-0025). Only the world pack's own region (created at world load) runs this step, once per tick inside the world update; the other loaded character regions of the area are updated without it, so only the world pack's own save points work. The reference position is the selected hero's position, and distances are squared 3D distances.
- The step keeps two nearest save points: the nearest with a nonzero flag and the nearest with a zero flag. A point replaces the best only when strictly nearer, so the lower list index wins a tie. Since every flag on disc is 0, only the zero-flag choice is ever set in play.
- A glow (two effect emitters, SPEC-0034) is moved to the chosen save point whenever the choice changes. The glow is not distance-gated: it always sits at the nearest save point.
- Within a squared distance below 225 (15 units), the save-point sound (cue 0xA8, SPEC-0031) is started or kept playing. The 225 test gates the sound, not the visual effect.
- Within a squared distance below 25 (5 units), in normal play (game-state code 0 and no in-play menu open), the context prompt "Save Game" is shown. A Square press while the hero is not busy then records the hero's restart point (its own position and facing, not the save point's) and opens the save screen (SPEC-0035, section 7.3). A busy hero still sees the prompt; only the opening is blocked.
- A save point with a nonzero flag behaves the same, except that Square does not record the restart point. No disc record has a nonzero flag.
Confidence: High (code); Medium (what the glow looks like).

**Kind 8, item point** (offsets from data):

| Offset | Size | Meaning |
| --- | --- | --- |
| 0x00 | 16 | Position: x, y, z, then w = 1.0 (all 3,421 records) |
| 0x10 | 16 | Unit facing: x, y, z, then w (all records unit length) |
| 0x20 | 1 | Flags, values 0 to 4 on disc. Two item-code tests read it: one is true for values above 1, the other only for 4 (what they gate is in SPEC-0021). Counts over all 3,421 records of all 284 resources that hold item points (every pack, reached or not; keyed and drop records together): flag 0: 512; flag 1: 201; flag 2: 1,241; flag 3: 1,241; flag 4: 226. The 1,061 keyed records alone (primary key 0 or more) split 452, 201, 145, 190 and 73, as SPEC-0021 counts them; the 2,360 drop records split 60, 0, 1,096, 1,051 and 153. Test vector: the five all-record counts add up to 3,421 and the five keyed counts to 1,061 |
| 0x21 | 1 | **Snap to ground** (High). Non-zero: when the placement is spawned, its height is replaced by G + 0.5, where G is the ground height (SPEC-0018 ground probe) at the placement's x and z in the world that owns the placement, probed with the ceiling set to the stored y + 4.0. Zero: the stored y is used unchanged. Both spawn routes use it: the region-load spawn (flag byte 0 or 1) and the scripted add by placement number (SPEC-0021, Spawn routes). When the probe finds no ground it returns 0.0, so y becomes 0.5. The editor default is 1. Disc: 1 in 3,029 records and 0 in 392. By flag byte: flag 0 has 336 ones and 176 zeros, flag 1 (shop stock) 4 ones and 197 zeros, flag 2 1,236 ones and 5 zeros, flag 3 1,233 ones and 8 zeros, flag 4 220 ones and 6 zeros. Test vector: placement at (10, 7, −3) with the byte set and a floor at y = 2.0 under (10, −3): the probe ceiling is 11.0 and the item spawns at (10, 2.5, −3); with the byte clear it spawns at (10, 7, −3) |
| 0x22 | 2 | Drop-group index. −1 in 1,423 records (no group); otherwise 0 to 6 on disc. It is not a second item id |
| 0x24 | 2 | Primary key. −1 (none) in 2,360 records. A key of 0 or more spawns that item id directly; negative values take a separate roll rule (SPEC-0021) |
| 0x26 | 2 | Second key. −1 exactly when the primary key is −1. Otherwise a per-item constant: 110 of 111 item ids map to one value (item 147 has two, 525 and 526); for example 56 → 1584, 150 → 992, and items 50 to 59 → 1580 to 1589. It is in the same editor id space as the kind-3 key (data +0xAC). Read only by a debug report printed when a block already holds 120 item placements and a 121st is refused (Blocks); no play reader (High). The copy kept in the world item record is never read |
| 0x28 | 2 | Luck gate. Nonzero enables the luck test (1 in 171 records; 0 in 3,250) |
| 0x2A | 2 | Fixed count (coins or amount when no money roll applies). 0 in 2,679 records |
| 0x2C | 1 | Treasure-map byte: for a treasure map (item 175) the map number is this byte plus 1 (SPEC-0021, Treasure maps; SPEC-0050). The byte is copied into the world-item record with the rest of the placement data |
| 0x2D | 3 | Fill on disc; no reader known |

Item ids and drop groups belong to the items spec (SPEC-0021).

### Kind 16, effect list (region record)

The data starts with a 16-byte list header, followed by n entries of 0x50 bytes each (n is the u32 at data + 4).

| Offset | Size | Meaning |
| --- | --- | --- |
| 0x00 | 4 | List id (105 distinct values, mostly 2022 to 2065). Never read: the clip walk picks a list by an index held in the character's clip table, not by this id (High for no reader) |
| 0x04 | 4 | Entry count n, 1 to 5 on disc (1 in 1,260 lists) |
| 0x08 | 4 | 1 in all 1,663 lists. Never read |
| 0x0C | 4 | 2 in all 1,663 lists. Never read |

Entry, at data + 0x10 + 0x50 i (offsets from the entry start):

| Offset | Size | Meaning |
| --- | --- | --- |
| 0x00 | 16 | Point 1: offset x, y, z in attach slot A's frame (w is taken as 1; the stored w is 0 except in 2 entries), the effect position and first trail edge. The first float is 0 in 2,097 entries and −0.2 in 57 (SPEC-0034, section 1.1) |
| 0x10 | 16 | Point 2: offset x, y, z in attach slot B's frame, the second trail edge (SPEC-0034, section 1.1) |
| 0x20 | u32 | Attach slot A: 0x40000000 plus a slot number 0 to 7 (this top nibble holds in all 2,154 entries) |
| 0x24 | u32 | Attach slot B, same encoding. 0x4FFFFFFF = none |
| 0x28 | f32 | Window start, as a fraction of the clip duration (0 to 0.97) |
| 0x2C | f32 | Window end, as a fraction of the clip duration (0.17 to 1.0) |
| 0x30 | s32 | Effect definition id; −1 = none (−1 in 386 of 2,154 entries; maximum 4040) |
| 0x34 | f32 | Rumble duration in seconds for positional mode 0 (0.4 is common); not passed to the sound setup |
| 0x38 | f32 | Maximum intensity (volume) for the positional path (1.0 typical) |
| 0x3C, 0x3D | u8, u8 | Ground-snap flags of point 1 (rule in SPEC-0034, section 1.1). 0x3C is 1 in 674 entries and 0x3D is 1 in 67 |
| 0x3E | u8 | Positional mode: 255 = none (1,973 entries), 0 (179), 1 (2); the positional path is SPEC-0034, section 5.3 |
| 0x3F | u8 | Rumble gate of the positional path: the value 0xFE (254) suppresses the rumble request, any other value lets it run. 0 in all 2,154 entries, so the rumble always runs for entries with a positional mode (High) |
| 0x40 | f32 | Inner radius (10.0 typical) |
| 0x44 | f32 | Outer radius (50.0 typical) |
| 0x48 | u8 | Trail flag: 0 starts a swing trail, any other value none (SPEC-0034, section 3.8) |
| 0x49 | u8 | Full-frame flag: non-zero passes slot A's whole matrix to the effect (SPEC-0034) |
| 0x4A, 0x4B | 2 | Zero in all entries; never read |
| 0x4C | u16 | 0xFFFF in 1,966 entries and 0 in 188; never read (High) |
| 0x4E, 0x4F | 2 | Zero in all entries; never read |

Behavior (Medium): while an object plays a clip, each entry whose window (its start and end, as fractions of the clip duration) contains the current clip time starts its effect definition, and the effect stops when the window ends. Start and stop use a pool of 10 effect slots. Effect behavior is in SPEC-0034. The list id, the header constants and the unread entry bytes above have no reader (High).

Span check: n = 1 gives a record of 0x70 bytes; n = 2 gives 0xC0 bytes.

Test vector (region 679, record at offset 0x2890 in the resource): list id 2302, n = 1, constants 1 and 2. Entry 0: point 2 offset (0, 0, 5.5), attach slots A and B both 0x40000000, window 0.0 to 1.0, effect id −1 (none), intensity 1.0, mode 255, radii 10.0 and 50.0.

### Kind 17, zero-filled reservation

| Offset | Size | Meaning |
| --- | --- | --- |
| 0x08 | 4 | Byte size of the reserved block (816 to 256,892 on disc; not a multiple of 16 in 207 of the 651 records) |
| 0x10 | size | Block content. All bytes are zero in all 651 records on disc |

There is one such record in each region that has one. The loader does not read the content. At region load, the skeleton-family warm-up uses this block as a run-time arena and writes pose data into it; it is not copied from another disc location. For each kind-3 character record, the arena cost is the sum of its main, sub and face skeleton allocations, using the corresponding family chunk's bone count and work-size word. A main skeleton with n bones uses a 4n-byte pointer list, then a 16-byte-aligned 32n-byte pose area (two identity vec4 values per bone) and 64n bytes of identity matrices, plus its family work area aligned to 4 bytes and 16 bytes. A sub or face skeleton uses 96n bytes plus 16 bytes and its paired family's work-size word. The total block size is the sum of these costs across the region's kind-3 records. This formula matches all 565 regions with both a type-36 chunk and a kind-17 record, including regions with and without global skeleton families (SPEC-0005).

Confidence: High (allocation layout, runtime initialization and census match). The identity fill is established for the pose and matrix areas; any later animation-time contents are outside this initialization description.

### Box records (chunk type 16)

Confidence for this section: High (layout counted on all 75 chunks; field roles from the build and the per-frame use).

Counts: 75 chunks in 74 packs (one pack has two). 4,205 records and 18,287 pool entries in all. Pack 678 has 71 records; pack 713 has 34.

Each record is a foliage patch (see "Purpose"). All offsets in this chunk, including every offset stored in it, are relative to the chunk header, which is 16 bytes before the payload (the chunk's extra length is 0 in all 75 chunks). At load the game turns each stored offset (record +0x14 and pool entry +0x10) into an address by adding the address of the chunk header.

Header (positions from the payload start):

| Offset | Size | Meaning |
| --- | --- | --- |
| payload +0x00 | s32 | Record count N (1 to 71 on disc) |
| payload +0x04 | u32 | Offset of the record table from the chunk header: 0x20 in all 75 chunks, so the records start at payload +0x10 |
| payload +0x08 | u32 | Zero in all 75 chunks |
| payload +0x10 | 0x40 x N | Records, back to back |

The pool entries follow the records directly: record 0's pool offset is 0x20 + 0x40 N from the header in all 75 chunks.

Record (0x40 bytes; record k starts at payload +0x10 + 0x40 k, which is header +0x20 + 0x40 k):

| Offset | Size | Meaning (on disc) |
| --- | --- | --- |
| +0x00 | 4 x f32 | Box x0, x1, z0, z1 (x min, x max, z min, z max). The patch centre is ((x0 + x1)/2, height, (z0 + z1)/2), and every row and plant lies inside the box. Record 0's box is what earlier versions of this spec called the chunk's union box. |
| +0x10 | u32 | Pool count: the number of pool entries of the record (earlier versions of this spec called this field the kind). Values 1 to 18 and 27 on disc. Counts: 1: 37 records; 2: 469; 3: 1,368; 4: 1,089; 5: 353; 6: 309; 7: 161; 8: 142; 9: 77; 10: 102; 11: 29; 12: 30; 13: 9; 14: 16; 15: 6; 16: 2; 17: 2; 18: 3; 27: 1. The values 0, 19 to 26 and 28 and up do not occur. The counts of all records add up to 18,287. |
| +0x14 | u32 | Pool offset: header-relative offset of the record's first pool entry. |
| +0x18 | u32 | Key, 0 to 39 (earlier called M); 0 in 811 records. The patch kind: it selects the build family and its models (SPEC-0060). |
| +0x1C | f32 | Height: the base y of the patch before ground snapping (not a radius). Pack 678, record 0: 74.1661. Pack 713, record 0: 118.5894. |
| +0x20 | u32 | Zone number of the patch (only the low byte is used): the column of the zone query (Binary grids). 0 to 17 and 48 to 57 on disc. In all 4,008 records of packs with a type-35 grid it is below block 0's width. |
| +0x24 | u32 | Jitter flag: 0 in 4,115 records, 1 in 90. When nonzero, the plants of some families are jittered (SPEC-0060). |
| +0x28 to +0x37 | 4 x f32 | Draw-distance overrides A, B, C, D (+0x28, +0x2C, +0x30, +0x34). All zero in 4,088 records. They are used when D or A is nonzero (117 records; the common sets are (200, 32, 50, 150) in 80 records and (50, 16, 10, 100) in 22); otherwise the patch's model gives the distances (SPEC-0060). |
| +0x38 | u32 | Fill word 0xCDCD0000 in 3,857 of 4,205 records. Its low half is 0 or 1 in packs 678 and 713. |
| +0x3C | u32 | 0xFFFFFFFF in every record of packs 678 and 713 (not counted over all records). |

Pool entry (0x14 bytes; one row of plants). A record's entries are pool count entries back to back from its pool offset:

| Offset | Size | Meaning (on disc) |
| --- | --- | --- |
| +0x00 | u32 | Item count of the entry (plants in the row) |
| +0x04 | f32 | Row z offset: the row lies at z = z0 + value (18,287 of 18,287 rows inside their box) |
| +0x08 | u32 | Fill (0xCDCDCDCD in most entries) |
| +0x0C | u32 | Fill (0xCDCDCDCD in most entries) |
| +0x10 | u32 | Item-block offset, header-relative. The block holds item count items of 16 bytes each. |

Item (16 bytes; one plant):

| Offset | Size | Meaning (on disc) |
| --- | --- | --- |
| +0x00 | f32 | Plant x offset: the plant lies at x = x0 + value (29,853 of 29,853 plants inside their box) |
| +0x04 | f32 | 3.0 in every sampled item of packs 678 and 713. Not read. |
| +0x08 | 4 bytes | RGBA colour of the plant instance: 0x80404040 in the sampled pack 678 items, 0x80334033 in the sampled pack 713 items |
| +0x0C | f32 | 173.4805 in all 29,853 items. Not read. |

Disc checks over all 75 chunks:
- Each record's pool starts where the previous record's pool ends (pool offset + 0x14 x pool count): 4,130 of 4,130 record pairs.
- All 18,287 item-block offsets lie inside their chunk.
- Within a record, each entry's item block starts exactly 16 x (previous entry's item count) bytes after the previous entry's block: 14,082 of 14,082 entry pairs.
- So no two pools overlap, no item block is shared, and nothing runs past the chunk end.

Purpose (High): each record is a foliage patch: grass, flowers, rustle beds, cuttable bushes, money and Negg bushes, clover patches, trees or bendable reeds. One patch object is built per record at world load, and the key selects its family and models. Plant k of row r sits at (x0 + plant offset, height, z0 + row offset), before the family's jitter and ground snap. Each frame a patch runs only while its zone is visible (zone query, column = +0x20) and the camera eye is within its far distance; within 100 units, and while the world state is below 9, its family logic also runs. The build families, the placement, the fade bands, the per-frame logic and the bush drops are in SPEC-0060 "Foliage patches (type 16)". Keys 0 and 12 (the grass patches, 933 records on disc) also run the builder of the twinkling sprite set (Medium-Low for this role): a set of sprite points grouped in rings of 12, whose size pulses with a 12-second period. Its ring layout, the angle formula and the test vector are in SPEC-0021 (Ring layout and item draw). The patch families of SPEC-0060 are unaffected.

### Tagged tables (chunk type 15)

Type 15 is a sequence of tagged records. Each record has a u32 tag at offset 0 and a payload from offset 0x10. A u32 at offset 4 holds K + 1, where K is the number of consecutive resources after the pack that hold a type-36 chunk (the region-slot count). The value is K + 1 for the tags 0x0A, 0x0B, 0x0E, 0x12 and 0x13 (High for 0x12 and 0x13, which match in all 291 tables). For tags 0x12 and 0x13 the u32 at offset 8 is the byte size of the entries and the u32 at offset 0x0C is zero. For tags 0x0A, 0x0B, 0x0D and 0x0E the count N is a u16 at offset 0x10. The length of each record follows from its tag:

| Tag | Record length | Payload |
| --- | --- | --- |
| 0x0B, 0x0D, 0x0E | 0x10 + the count N, plus 3, times 2, rounded up to 16 | N, then a u16 value table with at least N+3 entries; lookup of id k returns the value at index k + 3, or −1 when k ≥ N |
| 0x0A | 0x10 + 4N + 2, rounded up to 16 | N, then two u16 arrays A and B of N entries; A[k] and B[k] give an inclusive range |
| 0x12, 0x13 | 0x10 + (size at offset 8, low 4 bits cleared) | 16-byte entries; the count is size / 16. Byte 0 of an entry is the region number j (the j of a slot load: 0 to K-1), or 0xFF (−1) for the pack's embedded region. Byte 1 is the ordinal within that region. Bytes 2 to 15 are zero in all 6,449 entries on disc |

Checked: 182 of 182 chunks walk to their exact end. Tags on disc: 0x12 in 156 chunks (3,028 entries), 0x13 in 135 chunks (3,421 entries), 0x0A in 82 (645 range pairs, all with B + 1 ≥ A), 0x0B in 145, 0x0E in 71. Tag 0x0D is not used on disc. On load, each table is stored in the world record: tags 0x0A, 0x0B, 0x0D and 0x0E as range or lookup tables (0x0B at world +0x5904, 0x0D at +0x5908, 0x0E at +0x590C, 0x0A at +0x5910), and tags 0x12 and 0x13 as lists of 16-byte entries (count = size / 16). Tag 0x12 is stored at world-record offsets 0x38 (pointer) and 0x3C (count), and tag 0x13 at 0x64 and 0x68.

Value shape of tags 0x12 and 0x13 on disc: the slot byte takes 39 values (tag 0x12: slots 0 to 37 and 0xFF) or 31 values (tag 0x13: slots 0 to 29 and 0xFF), with 0xFF the most common slot for tag 0x13 (2,454 of 3,421 entries). Within a slot the ordinals run 0 to n-1 in table order in 154 of the 156 tag-0x12 tables. In tag-0x13 tables the ordinals are not always contiguous, because 0xFF entries are interleaved. Test vectors are in the Test vectors section.

Use. Range consumer (Medium-High for the role): the actor-setup routine reads N, and for actor id k below N it sets first = A[k] and count = B[k] − A[k] + 1 (High for the arithmetic). The 0x0B table is the neighbour list: entry v (a grid cell value) is the world id to stream or to pass through a door (World buffers and transitions; High). The 0x0E table is the arrival-door list: entry n (a door number of this pack) is the door number at which the hero arrives in the destination (High). Both are also read by an actor and event routine and by an item-class routine; one routine maps ids above 0xC0 through the 0x0B table (Medium-Low for those readers). Tag 0x0D is stored at world +0x5908 by the loader, and nothing reads that word; no chunk on disc uses the tag (High).

Tag 0x12 is the entity-id table (High). An entity id e is an index into this table. Script host calls that take an entity id, such as the entity-state queries IsAI_Active and IsAI_Dead, resolve it this way:
- Entry e gives the region byte g (signed) and the ordinal o. The ordinal of a kind-3 record is its index among the kind-3 records of its region's type-36 table.
- g = −1 (0xFF) names the world's embedded block (type 32).
- Otherwise g names the first created slot that is loaded and whose region number is g (the j its last slot load recorded; Region slots).
- The result is the character currently bound to record o of that block, or none. If no such slot is loaded, or e is at or above the entry count, the lookup fails and the host call does nothing or returns its default.

Reverse id (character to entity id; High). A character of the embedded block has id o, its plain ordinal, and the table is not consulted. A character of a slot has the index of the first entry whose region byte is that slot's region number and whose ordinal is o. The kill-bit index of a record follows the same reverse rule (Character activation, streaming and kill memory).

Embedded entries on disc (High). 86 of the 156 tag-0x12 tables hold 0xFF entries, and in all 86 their count equals the pack's embedded kind-3 count. In 71 they sit at index = ordinal, so the forward and reverse rules agree. In 15 they do not: packs 714, 732, 745, 758, 772, 893, 934, 947, 1066, 1074, 1091, 1148, 1257, 1324 and 1414. Quirk (reproduce): there, entity id e (the index of a 0xFF entry) reaches the embedded character, but that character's own id and kill bit are its ordinal, which is also the id of a slot entity. So host calls by the id from the table and by the character's own id name different characters, and a kill of the embedded character also marks the slot entity whose id equals its ordinal as killed (and the reverse). Example in the Test vectors.

Entity names (High). The character that entity id e of pack P names is fixed by the pack's data. Entry e gives a slot byte j and an ordinal. For j = 0xFF the record is in the pack's embedded region (type 32). Otherwise it is in character region resource P + 1 + j, the region that a slot load with that j names ("Which region a slot load names"). The record is the kind-3 record with that ordinal, and its name is the NUL-terminated text at kind-3 record data + 0x20 (Kind 3 table above). Villager rows (resource 531, SPEC-0024) hold no name text, so a character's name always comes from its placement. On disc: 3,028 entries in 156 packs. 2,755 name a record with a non-empty name. 271 name a record whose name field is empty (mostly monster types 100 to 109, and most of those in pack 1259). 2 name an ordinal past the end of the region's kind-3 list, so they find no record: pack 880 entry 18 (region 882, ordinal 11) and pack 1259 entry 186 (region 1293, ordinal 4). Names are disc data: read them at run time and do not copy them.

Tag 0x13 is the placement-id table (Medium-High; High for the slot rule). A placement id p is an index into this table. Entry p gives a region byte and an ordinal, resolved to a block exactly as for tag 0x12, and the placement is the kind-8 item record of that block with that ordinal. The breakable-drop routine and the scripted placement add use it. Its ordinal is the index among the kind-8 records of the region's type-36 table.

Slot range (High for the rule): for a region number g (−1 for the embedded block), the first and last entry index of the tag-0x12 table with region byte g give the run of entity ids for that region; this is the asset-group id range. If the region has no entry, the last index is the entry count minus one. The rule from an asset-group name to a region number is open (Open questions).

### Binary grids (chunk type 35)

Chunk: u32 count of blocks at 0x00 (1 to 8), u32 offset of the block table at 0x04. The table holds count u32 offsets, each to a block.

Block (324 blocks in 120 chunks on disc):

| Offset | Size | Field |
| --- | --- | --- |
| 0x00 | s32 | key. The first block (block 0) of each chunk holds −1 in the file and gets the pack's own world id when the pack loads; every further block holds the world id of a neighbour |
| 0x04 | s32 | width W (2 to 74) |
| 0x08 | s32 | height H (2 to 74) |
| 0x0C | s32 | W × H |
| 0x10 | u32 | offset of the cell bytes, relative to the block (0x14 in all blocks) |
| 0x14 | W × H bytes | one byte per cell, 0 or 1 |

**Zones (High).** Every ground model-set entry has a zone number z: the low byte of the entry header word at +0x0C (SPEC-0018, model-set entries; 0 to 23 on disc, where the upper three bytes are 0xFFFFFF). A type-35 chunk holds zone-visibility matrices. Block 0 is square: H = W = the pack's zone count (W is at least the highest zone + 1 in all 120 chunks, and equal to it in 80). Each further block is keyed by the world id of a neighbour, always one listed in the pack's tag-0x0B list (204 of 204 blocks). Its rows are this pack's zones (H = this pack's block-0 size, 204 of 204) and its columns are the neighbour's zones (W = the neighbour's block-0 size in 182 of 204; W = 2 for the 22 blocks whose neighbour has no type-35 chunk). Every neighbour that has a type-35 chunk has a block (419 of 419). Cell (r, c) = 1 means: while the camera stands in zone r of this pack, zone c of that world is active.

**Zone manager (High).** One global zone manager holds a grid G, a row R, a key K and an active block. On every pass the camera's mode update (SPEC-0027) re-sets it: G = the type-35 chunk of the camera's world (the world of the camera's ground entry, or the current world), K = that world's id, and R = the zone of the camera's own ground entry; in camera mode 0 with a follow target, the target's ground entry is used instead. R is stored only when the active block exists, z is below that block's H and z is below 255; otherwise R keeps its old value. The camera reset and make current (World buffers and transitions) also set R. The main loop sets the active block to block 0 for the current world's passes, and to the block keyed by the second world's id while it draws the second world's placed objects.

**Query (High).** A query gives a column c and a key k. It returns 1 when G is absent. If k equals K, block 0 is used. If k is -1 (the draw gates of placed objects, type-3 display-tree nodes and type-20 instances; the column is the object's stored zone column, SPEC-0028 and SPEC-0004), the active block is used (block 0 when there is none). Otherwise the block whose key is k is used, and the query returns 1 when there is no such block. With the block's width W and cell count N = W × H, the index is i = R × W + c. When i < 0 or i > N the query returns 1; otherwise the byte at index i decides (nonzero returns 1). Quirk (reproduce): i = N is not treated as outside, so it reads the byte just after the block's cells. A mode flag that would make the query "R equals c" is never set on this build. Fourteen routines make these queries, among them the character cell flag (Character activation, streaming and kill memory) and the object draw gate.

### Switch-keyed entries (chunk type 40)

One chunk per pack (191 packs). Offsets are from the chunk payload:

| Offset | Size | Meaning |
| --- | --- | --- |
| 0x000 + 0x44 k (k = 0 to 20) | 0x44 each | Fixed record k. There are 21 in every pack, and each record's first word is k. Seven pointer fields at record offsets 0x10 to 0x40 are rebased at load when nonzero |
| Fixed record 3, offsets 0x20 and 0x30 | 0x80 and 4 bytes | Run-time blocks allocated at load, not on disc |
| 0x534 to 0x593 | 0x60 | Nonzero in every pack; not decoded (Open questions) |
| 0x594 | u32 | Count M of list entries: 0 in 110 packs, 1 to 12 in the rest, 458 entries in all |
| 0x598 | u32 | List offset (0x59C in all 191 packs) |
| 0x59C + 0x44 i | 0x44 each | List entry i |

List entry (offsets from the entry start):

| Offset | Size | Meaning |
| --- | --- | --- |
| 0x00 | u32 | Kind, 0 to 12 on disc (meaning open) |
| 0x08 | u32 | Switch key: 0 in 227 entries (no key); otherwise 80 to 247. No key repeats within a pack |
| 0x34 | u32 | Count of effect indices: 0 in 332 entries, up to 49 |
| 0x38 | u32 | Offset of the effect-index list, payload-relative (rebased at load). Each index names a placed-effect record (chunk type 39) in the same pack |

Switch behavior (Medium-High for the key arithmetic; Medium for the callers):
- A switch id k is set by a switch-setting path. If k is below 201, the key is k + 200; otherwise the key is k. So ids 0 to 200 select keys 200 to 400, and ids 1 to 47 and their partners k + 200 select the same keys 201 to 247.
- The first list entry in list order with that key is applied. The switch value is stored. A nonzero value sets the active flag of each effect that the entry's index list names, which starts those effects. A zero value clears the flag and stops the effects.
- Key 0 is never sought by this path.
- Only 46 of the 458 list keys are 200 or above, so only those are reached through this path. The other 185 keys are below 200; other callers of the apply step exist and are not described, so these keys may still be used (Open questions).

Test vector (pack 732, list count 3): entry 0 has kind 6, key 203, and one effect index; switch id 3 selects it. Entry 1 has kind 2 and key 112, which is below 200 and has no path through this setter. Entry 2 has kind 0 and key 0 (empty).

### Zone records (chunk type 41)

The chunk holds a count N (u32 at offset 0). Records of 0x50 bytes start at offset 0x10. The chunk is in 138 packs (185 records). In 11 chunks (22 records) M 16-byte link entries follow the records, and each record's link offset points to its entries.

Record (offsets from the record start):

| Offset | Size | Meaning |
| --- | --- | --- |
| 0x00 | 4 x f32 | Vector A: RGBA colour for the screen post pass (no visible effect on disc); see SPEC-0028 section 11 |
| 0x10 | 4 x f32 | Vector B: screen overlay colour, RGB plus alpha (gated records get alpha 28 at load); see SPEC-0028 section 11 |
| 0x20 | f32 | Depth-range maximum (the depth-buffer scale of the view-screen mapping, not a draw distance; see SPEC-0028 section 11): 100000.0 in 92 records, 1048576.0 in 29, others elsewhere (Medium) |
| 0x24 | u32 | Gate key: 0 = ungated; 1 to 13 on disc (ids in the table below) |
| 0x30 | 4 x f32 | Bounding circle of a polygon zone: centre x, y, z (y unused); the fourth lane at +0x3C is its radius (SPEC-0028 section 11). Zero in some records |
| 0x3C | f32 | Bounding circle radius (1.0 in 163 records) |
| 0x40 | u32 | Link count M (0 in 163 records; 4, 5, 7 or 9 in the others) |
| 0x44 | u32 | Offset of the M link entries, payload-relative (rebased at load when M is nonzero). Each link entry is taken as a point |

Per-frame selection (Medium for the state and constants; Medium-Low for the rules):
1. Focus hysteresis: the routine remembers the last focus point. If the current focus point is more than 50.0 units from it, the remembered point is replaced and the blend is 0. Within 50.0 units, a fade factor of 1 − distance / 50.0 is kept, which is used to interpolate between the previous output and the new one.
2. Default: the first ungated record with no link entries is the fallback. A gated record with no link entries is taken when its gate test passes. If no record qualifies, a built-in default record is used.
3. A record with link entries whose gate test passes becomes a candidate when the distance from the focus point to its nearest link point is within 50.0 units.
4. Blend: for a gated candidate, the blend is 1.0 when the distance is above 25.0 (or when the chosen record is itself gated), and distance / 25.0 otherwise. An ungated candidate has blend 0. The largest blend over the candidates is the output blend.
5. Output: the chosen record's fields are copied to the zone output. The renderer receives the post-pass colour, screen overlay colour and depth-range maximum; the blend factor is kept in the camera record for the sky dome's role-10 opacity. The downstream display and draw behavior is described in SPEC-0028 section 11 and SPEC-0046, Sky domes. The camera only supplies the eye position (SPEC-0027).

Gate test: keys 1 to 7 and 9 to 12 pass when the first listed game flag is set and the second is clear; key 8 returns the raw value of its flag; key 13 always passes; other keys fail. Key 0 is handled as ungated by the caller. The ids are:

| Key | Required set flag | Required clear flag |
| --- | ---: | ---: |
| 1 | 83 | 5 |
| 2 | 10 | 322 |
| 3 | 64 | 283 |
| 4 | 57 | 58 |
| 5 | 74 | 263 |
| 6 | 85 | 298 |
| 7 | 92 | 94 |
| 9 | 332 | 100 |
| 10 | 332 | 102 |
| 11 | 332 | 104 |
| 12 | 332 | 106 |

Key 8 reads flag 248; key 13 has no flag test. The narrative meaning of these flags remains unknown. **[High for logic and ids]**

Constants (read from the boot program at data addresses): 50.0 at 0x2DB4CC and 25.0 at 0x2DB4D0. Both are used as described above.

Type 41 records are per-frame zone selectors. They are not trigger volumes: trigger volumes use other records (SPEC-0028). The focus point and the camera's use of zone output are described in SPEC-0027.

Test vector (pack 678, N = 1, record 0): gate key 1, bounding circle centre (0, 0, 0) and radius 1.0, depth-range maximum 100000.0, link count 0.

Confidence: layout High (counted on disc); the 50.0 and 25.0 thresholds High (values), Medium (use); per-frame selection Medium-Low.

### Tool data and transformed records (types 10, 11 blocks, 28, 30, 39)

- **Type 39** holds placed effect spawns, not transformed props. Its layout is in SPEC-0028 section 4.
- **Types 28 and 30** hold camera tracks and light sets (SPEC-0028 sections 1 and 2). Type 28 is the camera track table of SPEC-0052 and SPEC-0027 (High). Type 30 is the per-area light set that characters read (see SPEC-0046, Character lights).
- **Type 10: ladders and climb panels** (High). Payload: u32 group count G at +0x00 and u32 offset at +0x04 (chunk-relative, like every offset in this chunk) to G groups of 0x20 bytes: centre (3 x f32, +0x00), squared radius (+0x0C; every record's origin lies inside), u32 record count +0x10, u32 offset +0x14 to the group's 0x60-byte records. Record layout:

| Offset | Size | Meaning |
| --- | --- | --- |
| +0x00 | 4 x f32 | Origin (x, y, z, 1) |
| +0x10 | 4 x f32 | Normal n (unit); w = plane offset, so n . (p, 1) is the signed distance of a point p from the plane |
| +0x20 | 4 x f32 | Axis U (unit, the side axis); w = offset, so U . (p, 1) is the position along U from the record's start |
| +0x30 | 4 x f32 | Axis V (unit, the up axis); w = offset, as for U |
| +0x40 | f32 | Length LU along U |
| +0x44 | f32 | Length LV along V |
| +0x48 | 4 | Zero on disc |
| +0x4C | s8 | Type: -3 ladder, -2 climb panel |
| +0x50, +0x54, +0x58, +0x5C | 4 x u32 | Neighbour record links (offsets; 0 = none) |

  Disc census: 63 non-empty chunks (63 packs), 194 groups, 453 records. 218 are type -3: 109 ladder faces with a horizontal normal, each paired with a top record whose normal points up. 235 are type -2: 139 walls, 81 facing up and 15 facing down (the ceiling panels, in packs 1056, 1066, 1080, 1090, 1298 and 1392). Pack 1078's type-10 chunk is empty. The world loader keeps the table on the world object (world +0x5788). The hero's grabs walk it (SPEC-0033, Behaviors 22 and 22A), and the hero's object-wall contact tests its records (SPEC-0018).
- **Type 11 movable-block table** (High). The type-11 chunk (world +0x56B0) also holds a table of 0x24-byte entries: count at header +0x1C, offset at header +0x44 (SPEC-0028 section 6). At world load an entry becomes one 0xB0-byte movable-block record only when its box pointer (entry +0x20, a kind-4 rectangle record) is set and the box's flags have bit 1 (value 0x0002) set, as in SPEC-0048. Disc: 91 entries in 17 packs build 90 blocks (pack 1344's one entry has no box). The entry layout and the built block are in SPEC-0048.

## Behavior

### Loading an area

1. The game asks for an area by slot number: from a script transition, a door, the pending step, a return after a movie, or the default start. A saved game holds two world ids (transition and exterior buffers), each converted to a slot first. The slot's row gives the world-pack resource number to load (see "Area numbering"). The pack goes into the world buffer of its kind (World buffers and transitions).
2. It reads the pack, walks its chunk chain, and sets up each chunk by type (see "How the loader handles each chunk"). Then it runs the finalize steps, in order. The level mesh, objects and instances are drawn per SPEC-0004. The spatial grid and its cell records are set up per SPEC-0018.
3. It reads the zone id (type 1001) and looks up the world id in table B. The world id is handed to the character and sound managers. The first block key of each binary grid (type 35) is set to the world id (see the Binary grids section). The world's 256 kill bits are loaded from its saved per-world record (Character activation, streaming and kill memory).
4. The embedded character region (type 32) is set up as the world's character block. Its kind-3 records are not all spawned at once: they are bound to character objects by proximity to the hero, up to the block's active cap, using the region's model sets and skeletons (SPEC-0005; Character activation, streaming and kill memory).
5. The level script (type 37) runs. It loads extra **character regions** as needed: region slot *j* of a pack is resource **P + 1 + j**, where P is the pack's resource number. The script supplies *j* through SlotLoad; a slot-buffer reservation (SlotCreate) loads nothing (Region slots). Regions therefore sit directly after their pack on the disc. All 575 regions belong to exactly one pack's run. Each run is contiguous and starts at P + 1, except the front end's, which starts at P + 2 (resource 533 is the text resource between them). 102 game levels have a run, of up to 13 regions; 89 have none.
6. The hero is placed by the rules of "Hero placement" below: after a driver load by the request's plane index, after a door or an interior job on a teleport plane or an entry record of the new world, after a new game at the world spawn vector. After a world swap the hero simply keeps walking. The camera and fade steps are in SPEC-0026.

Step 3 above sets the first block key of each binary grid. In the loader that is finalize step 5, which runs after the walk.

### Hero placement

Confidence: High, except where marked. Notation: H(x, z, c) is the ground probe at (x, z) with ceiling c (SPEC-0018). normxz(v) is v with y set to 0, then normalised. Each hero keeps a start pose (a position and a facing, 16 bytes each); the respawn at the start pose (SPEC-0040) moves the hero there and puts the camera behind it. Entry records and plane records are those of "Entry records and teleport planes".

**New game.**
1. Load pack 691 into T and make it current. Then test mask 0x65 (bits 0, 2, 5 and 6) against word +8 of a global runtime object; this field is not a controller-button word, and the meanings of those mask bits are unknown. If any masked bit is set, the routine skips both the selected exterior load (resource 909 for the default start) into E and the step that makes E current; pack 691 remains current. Otherwise it synchronously loads the selected exterior (resource 909 for the default start) into E and makes it current. The object initializes word +8 to zero, and the static call-site scan found no later writer to that field, so the bypass is unreachable through normal controls or the debug input path in this build (High for the inspected static paths; external mutation is untested).
2. Set the act value from the story flags (SPEC-0049).
3. Spawn both heroes at the current world's spawn vector, with the spawn facing; this is also their start pose. In this build that is 909's entry record 0 (Test vectors).
4. Reset the camera, then respawn the focus hero at its start pose with the heal option (SPEC-0040).
Neither the level script's start step of 909 nor that of 691 (or its twin 678) moves the hero. 909's first-play step (story flag 707) plays movies 88 and 1 and starts act 1 without moving the hero.

**Load request and the post-load rule.** Each world buffer holds one load request: the world, a plane index p, a load mode, a make-current flag and a fade colour; a reset request has p = -1. The script calls StartExteriorReplacementLoad(world, p, colour) and StartExteriorReplacementLoadWithoutFade(world, p) pass p unchanged. The door path passes the tag-0x0E arrival entry of the touched door. The after-movie reload passes -1 when it reloads a return world or the return transition, and -2 otherwise (Other routes; High). The save apply and the front-end world entry pass -1. Of the 83 script exterior-replacement call sites (one count per module image), 74 pass -1, 8 pass 0 and 1 passes 2.

After a driver load into a buffer (World buffers and transitions, Driver loads), the request is cleared and then:
1. p ≥ 0: the focus hero lands on teleport plane p of the loaded world (Teleport landing). If the landing succeeds, stop here.
2. Otherwise the start pose becomes: for p ≤ -2, the hero's current position and forward; for p = -1, or a refused plane, entry record 0 of the loaded world (the zero vector for both position and facing when the world has no entry records). The hero's ground-record slot (hero record +0x174; SPEC-0018) is cleared, so the pending step sees v = -1 until the next cell lookup; the start pose is stored, and the hero respawns at the start pose without the heal.
3. With the make-current flag, the world becomes current.

**Teleport landing** on plane k of world W, for character C, with a fade flag and a reverse flag:
1. Refuse (log, return failure) when k < 0 or k is at or above W's teleport plane count. The plane is flat when |F.y| > 0.85.
2. If C is the focus hero, W becomes current; with the fade flag a 10-unit fade runs around the move. C's action state is reset.
3. Facing G = F, or -F when the reverse flag is set (the door step's facing sign, World buffers and transitions).
4. Position. Flat plane: L = P + h·Q + w·D, then L.y = L.y - 5.0, with no probe. Upright plane: L = P + 0.5·w·D + 5.0·normxz(F - Q), then L.y = H(L.x, L.z, L.y + 6.0).
5. If C is the focus hero, W is the interior buffer and W has entry records: L and G are W's entry record 0 instead.
6. C's motion moves to (L, G) at once. If W has entry records, C's start pose becomes W's entry record 0 (every teleport resets the respawn pose).
7. For the focus hero only: the partner gets the same world, the same start pose and the same (L, G); the camera eye is set to the hero position - 5 × hero forward.

**Interior jobs.** Their arrival rules are in "Interior entry steps".

**Load game.** Driver loads place the heroes at entry record 0 (p = -1). In the same pass, after both buffer drivers, the scene-start apply stores the saved position and facing as the start pose and respawns there without the heal, so the final pose is the saved one, not snapped (SPEC-0025).

**Script placement.** MoveHero(i, probe, control, face) moves the hero to entry record i of the current world at once and does not change the start pose. SetHeroStartPos(i) only stores entry record i as the start pose of both heroes and moves no one (SPEC-0030, SPEC-0055).

**Deadly-ground respawn.** After a death on deadly ground with a respawn entry index, timer F places the focus hero at that entry record of the current world after more than 1.5 s, in state 60 (SPEC-0055, section 8A).

### Character activation, streaming and kill memory

Confidence: High for the steps and constants (78,400, 20, 5, 8, 15, 6 and 40 are fixed values of the code), the cell flag, the dead test, the activation filter store and the kill-memory store points; Medium-High for the loss of kills in worlds released without a store (a reproducible quirk until confirmed at run time) and for the character world-id word.

**Blocks.** A character block is the embedded region of the pack (type 32, at world record +0x58E8) or one script slot (a loaded character region; the slot records start at world record +0x74). A block holds at most 40 kind-3 records (more are refused with an error), 8 save points and 120 item points. It owns min(records, 15) character objects. At world load a warm-up prepares the assets of record i on object i mod 15; that rule is only the warm-up, and a streamed bind takes the first free object. Each block has:
- an active cap: 6 after construction or reset, 15 after a slot load with keep false. SetMaxActiveAI sets the embedded block's cap, clamped to 15 (SPEC-0030; used only in packs 1176 and 1246, which set 12);
- a keep (streaming) flag: 1 for the embedded block; for a slot, 0 after a load with keep false and 1 after one with keep true;
- an optional activation filter at block +0x30: a callback that takes a record ordinal and returns true or false. RegisterAIPlayerActivateCallback(world, fn) (SPEC-0030) stores fn in the world record (+0x30) and forwards it to the embedded block and to every slot created so far (the created-slot count). A slot created after the registration has no filter; module 1096 registers before its SlotCreate, so only its embedded block is filtered. A slot reload keeps the filter (High). Pack 1217's active Act 1 fight filter admits embedded ordinal 3 and rejects ordinals 0–2 and 4–7; its explicit stage spawns force-bind the Tentacles (SPEC-0045).

**Activation-filter vector (pack 1217, Act 1):** embedded record ordinals 0 through 6 return false, false, false, true, false, false, false.

The region pass of a world runs the embedded block and then each created slot whose loaded word is set. It runs for the current world and, whenever the second world is loaded, for the second world (World buffers and transitions, Neighbour-world test).

**Eligibility.** Record i of a block may be bound when, in this order: the filter (if any) returns true; i is not bound to an object; and then either the record's monster row has the reanimation flag and its summon byte (data +0xB6) is 0, which makes it eligible at once, or its summon byte is 0, its kill bit is clear, and its key (s16 at data +0xAC) is not the active petpet's key. The petpet-key test is part of eligibility, so it applies to the swap step and to the fill step alike.

**Cell flag (High).** Each tick, for each active slot of every block of the current and the second world, the character's cell-enabled word (+0x18C) is recomputed: 1 when the character has no ground entry (+0x178), otherwise the zone query (Binary grids) with column = the zone of that ground entry and key = the character's world id (+0x7B4; set at bind and at make current, Medium-High). A character whose flag is 0 is neither updated nor drawn (SPEC-0046, Drawable test); in a streaming block it is also released by step 1 below. Records that are not bound are gated the same way at their own position: the record's ground entry is probed once in the block's own world (from the record's y + 5 in the swap step, or y + 8 in the fill step; whichever runs first fills a per-record cache, which a release clears), and then queried with that world's id. A probe that finds no ground counts as active, and so does a block with no world.

**Dead test (High).** Each character class has its own dead test. For townsfolk and enemies a character is dead when its base state is 16, or when its AI mode word (+0xA34) is 16 and the counter at +0xD14 is above 0. The death disappear countdown is the object word +0xD24, loaded from the monster row's death disappear timer (row +0x80) at death and counted down by the dead-state update (SPEC-0038, Behavior 34 to 36).

**Per tick, streaming block (keep 1).** H is the active hero's position, and d is the squared 3D distance from H to a character or record position.
1. For each active object slot: a character that is dead gets its kill bit set (on every tick while it stays bound). The character is marked for release when it is dead and its death disappear countdown is below 0, when its cell flag is 0, or when d > 78,400 (strict, so d = 78,400 exactly is kept; 280 units). A marked character is released. A kept character whose base state is 42 is set to base state 1. The farthest kept character (largest d; on a tie the first one found stays chosen) is noted.
2. If a kept character exists: among the eligible records whose cell gate passes and with d < 78,400 (strict), take the nearest (strictly smaller wins, so the earlier record wins a tie). If its d plus 20 is less than the d of the farthest kept character, release the farthest and bind the nearest. At most one such swap happens per tick. (Quirk: the 20 is added to a squared distance, so the margin shrinks with distance in units.)
3. While the active count is below min(cap, object count), bind the nearest eligible, gated record with d < 78,400 (strict) into the first free object slot; stop when none is found.

**Per tick, non-streaming block (keep 0).** Each active character that is dead gets its kill bit on every tick, and it is released once its death disappear countdown is below 0. There is no wake (a bound character in base state 42 is not set back to 1), no distance or cell release, and no bind.

**Base state on bind and release (High).** A character object is created and reset in base state 42, the parked state (SPEC-0040). Every bind (streamed, forced by ActivateEntity, and spawn by key) resets the object and then sets base state 1 before the character is first drawn, so a newly bound character is in state 1. Releasing a character that is not dead sets it to base state 42.

**Per tick, every block: slot update (High).**
- For each slot that is not active, an object that exists and is not dead is set to base state 42 if it is not already in it.
- For each active slot, the cell flag is recomputed (Cell flag above). The character's update runs only when the cell flag is 1 and its despawn flag (SPEC-0017) is 0. The same two words gate its draw (SPEC-0046, Drawable test).

**Release and rebind (High).** A release lowers the active count by 1, marks the slot inactive and sets the slot's bound-record byte to none. Unless the character's AI mode word is 7, its current position and facing are written back into its record and the record's cell cache is cleared. Mode 7 comes only from the record's default AI mode word (kind-3 data +0x84): the mode setter is never called with 7 as a fixed value, and the two default-mode paths copy the record word into the mode word. So the characters whose release keeps their record position are those whose record has default mode 7 while they are still in that mode: 392 of the 3,036 kind-3 records (84 in embedded blocks, 308 in region resources, in 134 resources and 54 monster types; for example Skeleton Archers, Poison Spyders, Ixi Spearmen, Bandits and several named guardsmen). Such a character is bound again at its disc point (Medium-High). If the character is not dead it is set to base state 42. Then the character is deactivated and its object is free. A later bind starts the character fresh from its monster row (full health) at that point. So an enemy the hero walked away from comes back at full health where it was left.

**Kill memory (High for the store points; Medium-High for the loss, until confirmed at run time).** Each world keeps 256 kill bits in the kill block at world record +0x40 (32 bytes; bit b is byte b >> 3 with mask 1 << (b & 7); one spare byte follows: the bit routines accept only indices 0 to 255, so the spare byte is never set and always stays 0, although it is copied with the block). The bit index of an embedded-block record is its kind-3 ordinal. For a slot record it is the index of the first entry (the slot's region number j, ordinal) in the pack's tag-0x12 entity table; a slot record with no entity entry has no bit and is never remembered as killed. In 15 packs an embedded record's bit is also a slot entity's bit (Edge cases, Embedded ids that collide). The largest entity table on disc has 224 entries, so every listed record fits. The kill memory proper is a per-slot table in run-time memory (data address 0x386740, 193 entries of 33 bytes, one per area slot, 6,369 bytes in all; table T2 of the save, SPEC-0025; a new game zeroes all of it). Each world load copies the 33 bytes of its area slot's entry into its kill block. The kill block is copied back to the table only:
- when the per-pass driver step releases a buffer for a request that is not a game load (replacement loads, the after-movie reload, the front-end world entry; Driver loads); and
- when the game is saved, for the current and the second world (SPEC-0025).

A world released by a stream load (the pending step loading into T or E), by the enter-interior job (T and I) or by the leave-interior job (I and T) does not store its bits. Kills made in that world since it was loaded are forgotten, and those enemies are bound again on the next visit (Edge cases). A record whose bit is set in the table is never bound again, so ordinary enemies stay dead on re-entry when their world's bits were stored. A bit is cleared only by a forced bind of its record (ActivateEntity, and three other code paths that use the same forced bind), and all bits are cleared by a new game. Rows with the reanimation flag ignore the bit and rise again (SPEC-0038).

**Summon-reserved records.** A record with a nonzero summon byte (data +0xB6; 208 records on disc) is never streamed in. It is bound only by a spawn-by-key request (SPEC-0029): the request names a key (data +0xAC), a point and a facing, and the first unbound summon-reserved record of the block with that key is bound at that point into a free object, while the active count is below the cap. All 180 kind-3 records whose position is all zero are summon-reserved; their position and facing come from the request.

**Location name.** The zone record (chunk 1001, payload + 0x10) holds the zone id at +0x00 and the location index at +0x04, the same field as the map area id (Zone id). The index is a row of design table 5 (SPEC-0024); -1 keeps the previous name.

### Area object lifecycle (Medium)

The area is held in one large state record of at least 0x5914 bytes (the reset's highest word is at 0x5910). Its method table has the reset at slot offset 296 and the unload at slot offset 304. The clean side needs only the slot names, not the table.

**Constructor.** Runs a base initialiser, sets the record's method table to the area table, clears 0x21 bytes at offset 0x40 (the kill block) and one word at 0x90, and initialises the record's sub-objects (a 0x2810-, 0x29F0-, 0x2D40-, 0x3850-, 0x4260- and 0x52F0-byte group, each with its own method table, plus sub-objects at 0xA0, 0x54D0 and 0x56C0). Four 0x30-byte records at offset 0x56C0 are each initialised by a helper. It then clears the words at 0x581C, 0x5818, 0x5848, 0x584C, 0x585C and 0x58E8, sets the word at 0x58E4 to -1, and calls the reset. Medium.

**Reset.** Resets the base state and the sub-objects, re-runs the four 0x30-byte record helpers, and sets four id words to -1 (at offsets 0x00, 0x2CCC, 0x57C4 and 0x58FC). It zeroes a run of about 60 state words and a short zero block at 0x40. It also sets a start vector from a zero default. Five buffer-release helpers run only when their pointer is nonzero, and each owner pointer is cleared afterward. Medium.

**What a released world loses (High for the steps; Medium for the roles).** Releasing a buffer runs the world reset. The zone manager is cleared when the world was current. Its script is torn down and its sound set is released. The four effect records, the effect group (+0x2CC8), the two placed-effect lists (+0x5848 and +0x584C) and the sub-objects (+0xA0, +0x2810, +0x29F0, +0x2D40, +0x3850, +0x4260 and the display node 401) are reset, and so is the record of the looping-door puzzle of pack 1299 (SPEC-0028). The embedded block is reset: every bound character is deactivated without a write-back, the per-record helpers are freed, keep becomes 1 and the cap 6. Every created slot is released (created count 0). The pack and the world id become -1 and the slot 0, the kill block is zeroed, and the load request is cleared. World items follow SPEC-0021 (pool reset at teardown). Since the pack is read again from disc on the next load, its characters start from their disc points at full health, except the killed records whose bits were stored (Kill memory).

**Unload.** Does not reset the state. It first sets each sub-object's method table to its base table, then walks five record lists from the last element back to the first (record sizes 0x160, 0x140, 0x160, 0xE0 and 0x150) and calls each element's destroy method. Each sub-object's method table is then set to an empty table. The record's own method table is set to the base table last. If bit 0 of the unload argument is set, the record itself is released. Medium.

Callers of the constructor, reset and unload are reached through the method table; the caller that unloads an area is not identified (Open questions).

### Pending area change (High)

The queued area change is the pending step of "World buffers and transitions (area exits)". It runs every tick on the current world: it resolves the target from the pending slot or from the cell value under the hero, compares the current buffer with the transition and exterior buffers, starts a load into T (from E) or into E (from T), and then clears the pending slot, so each request runs once.

**Pending slot writer.** SetWorldLoadOverRide is the writer of the pending slot. Its argument is a slot number (see "Transitions take slots"). The setter stores the argument in the current world's pending word only when the argument is below 193; an argument of 193 or more is not stored and does nothing. Its calls are listed under "Area numbering". A slot set by a trigger callback is also re-asserted each tick by the world pass while the game-state word +0x44 is nonzero (Medium). The pending step sets the pending word back to -1 at its end.

### Interior entry steps (High for the arrival; Medium for the wait details)

These are the steps of the two interior jobs (World buffers and transitions, Interiors). Each job carries a world slot, a plane index a, an entry index b and a fade colour. Notation as in "Hero placement". Both jobs use the 0.3 second fade length of SPEC-0026.

**Enter-interior job** (StartEnterInterior(world, a, b), and a door from E to an interior):
1. Fade: a screen fade out of 0.3 seconds starts, unless the screen is already faded.
2. Wait: the interior is requested, and the job yields one frame at a time while it loads (until the loader's list of pending loads is empty).
3. If the interior loaded: it becomes current and the hero's action state is reset. Defaults: L = (0, 0, 0), G = (1, 0, 0, 1).
4. Arrival: if 0 ≤ a < the interior's teleport plane count, L = P + 0.5·w·D + 5.0·normxz(F - Q) of plane a and G = F (the upright form of the teleport landing). Otherwise, if the interior has entry records, L and G are entry record b of the interior, with b replaced by 0 when b < 0 or b is at or above the interior's entry count.
5. Ground snap, always: L.y = H(L.x, L.z, L.y + 6.0).
6. The start pose becomes (L, G); the hero and the partner hero move there; the hero's state becomes 1; the camera eye is L - 5·G.
7. Fade-in of 0.3 seconds after five idle loader frames.

**Leave-interior job** (StartLeaveInterior(world, a, b), and a door from I):
1. The focus hero's state becomes 1; an instant fade to the job's colour; one frame passes. If E is not loaded, skip to step 5.
2. E becomes current and the hero's action state is reset. Defaults: L = (0, 0, 0, 0), G = (0, 0, 1, 0).
3. Arrival: if 0 ≤ a < E's teleport plane count, L is the upright form of the landing on plane a of E (always the upright form, and always probed: L.y = H(L.x, L.z, L.y + 6.0)), and G = F. Otherwise, if E has entry records: e = b, replaced by 0 when b < 0 or b is at or above the entry count of the interior buffer (quirk, below), and L and G are E's entry record e, with no probe.
4. The start pose becomes (L, G); the hero moves there at once and is attached to the ground; the partner hero is placed the same way; the hero's state becomes 1; the camera eye is the hero position - 5 × hero forward.
5. The other buffers are released as in "Interiors"; when the job's slot is 0 or more, its pack is loaded into T (asynchronously).
6. After five consecutive idle loader frames, a 0.3 second fade-in starts, and the camera returns to normal.

Quirk (reproduce): the leave-interior job bounds the entry index with the interior buffer's entry count but reads the entry from E's list. Every script call passes b = -1, so in play it always lands on E's entry record 0 (Edge cases).

The +6.0 of the ground snap is a fixed constant of the program, the probe start above the landing point; no run-time word holds it.

Relation to SPEC-0026: the arrival formulas of SPEC-0026's area entry steps (Entering the area) are these jobs' hero placement, not a camera pose. The camera follows the hero (eye = hero position - 5 × hero forward).

### Checks the clean side can run

- Every table-A entry is a world pack (193 of 193).
- Every facing vector is unit length: kind 3 in 3,036 of 3,036 records, kind 4 in 88 of 88, kind 8 in 3,421 of 3,421.
- Embedded placements (type-32 regions) lie within the type-19 bounds expanded by 50 units: kind 3 in 630 of the 634 embedded records, kind 4 in 88 of 88, kind 8 in 2,336 of the 2,454 embedded records.
- Every type-15 0x0A table has B + 1 ≥ A for all 645 range pairs.

### Buffer load requests (High)

The three load destinations are the world buffers E, T and I; they are not three queues of requests. E and T each take one load at a time. A load into either buffer is refused while that buffer is reading. Otherwise it starts when the requested pack differs from the resident pack, or when the caller requests an unconditional reload. The pending world-change step uses the skip-if-same form for E or T: if the target already occupies the destination buffer, it does nothing. The per-pass driver uses the unconditional form, so a driver request reloads the destination even when it already holds that pack.

An E or T load records the buffer's resident area slot, clears the 15-entry active-character list, and begins reading the requested pack into that buffer. The caller's reset argument controls extra teardown; the per-pass driver uses the form that resets render/effect state and stops sound, while the pending step uses the lighter form. A new game loads 691 into T and 909 into E directly.

The interior-entry job is a separate operation into I. It runs only while T is not reading; it records I's slot, remembers T's pack in I, releases T and I, gives I the transition buffer's pool, and begins reading the interior pack. Thus, a request made while T is busy is refused rather than queued. The loader's three-entry test belongs to an interior-load variant with no caller found in the boot program; the apparent three request queues came from conflating buffer paths and debug labels.

Confidence: High for the separate E, T and I paths, their busy and resident-pack guards, and the caller distinctions; the exact pass on which an asynchronous read completes remains open (area-change and world-lifetime evidence).

### Area-pair test (High for the test; Medium-High for the meaning)

The test takes a pack number and compares it with the current pack. It searches the 25 entries of the area-pair table in table order and returns 1 at the first entry where either:
- the first number of the entry is the current pack and the second is the argument; or
- the second number of the entry is the current pack and the first is the argument.

If no entry matches, it returns 0. Because entries 23 and 24 are zero pairs, the test also returns 1 when both the current pack and the argument are zero; no pack is numbered 0, so this never happens in play. The pairs link exteriors that are entered from each other by a door replacement load (Area-pair rule). The test is used in the pending step, where a 1 result stops the stream, and in the default door rule, where a 1 result selects the replacement load of E.

## Edge cases and error handling

- **Duplicate slots**: two packs each occupy two consecutive table slots. Treat a slot as an alias, not a different level.
- **Front-end pack (532)**: it has no level mesh or grid (chunks 4, 31 and 38 only) and isn't in the slot table. It is loaded by the front end (SPEC-0035).
- **Kind-3 records with an empty name** (246 of the 2,402 top-level records): still bind the character the indices describe; no name panel shows for it (SPEC-0023).
- **Kind-3 records with a zero position** (180 records): all are summon-reserved (nonzero summon byte). They are never streamed in; a spawn-by-key request binds them at the point and facing it supplies (Character activation, streaming and kill memory).
- **More than 40 kind-3 records in a block**: the extra records are refused with an error. On disc the largest block (pack 1080) has exactly 40. A block with more than 15 records still has only 15 objects (15 embedded and 20 region blocks on disc).
- **Swap margin quirk**: the streaming swap adds 20 to a squared distance (Character activation, step 2). Reproduce it as is; do not convert it to 20 units.
- **Kill memory forgets kills in worlds released without a store (quirk; Medium-High until confirmed at run time)**: only a driver-load release that is not a game load, and a save, copy a world's kill bits into the per-slot table (Kill memory). A world released by a stream load into T or E, by the enter-interior job or by the leave-interior job does not store them, and a load requested by a game load does not store the bits of the worlds it releases. Kills made in such a world since it was loaded are forgotten, and those enemies are bound again on the next visit. Reproduce it as is.
- **Released living enemies**: a character released by distance is rebound later at full health at the point where it was released.
- **Packs without a script module** (19 packs): there are no extra regions to load.
- **Save-point flag word**: the step tracks the nearest zero-flag and the nearest nonzero-flag save point separately. Every save point on disc has flag 0 and works fully (glow, sound, prompt, save screen, restart point). A nonzero flag only drops the restart-point write.
- **Save-point ties and overflow**: of two save points at equal distance, the lower list index is chosen. A region with more than eight kind-4 records keeps the first eight.
- **Save points outside the world pack's own region**: kind-4 records in other loaded character regions are not checked (none exist on disc).
- A world pack with no embedded character region has no spawns of its own.
- **Zone ids that resolve to another pack**: keep loading the pack that was requested. The id only groups variants. The loader does write the canonical resource into the first word of the requested world block (Open questions).
- **Instanced meshes (type 20)**: the loader accepts at most two type-20 chunks per load and reports a larger count. 23 packs have one each on disc.
- **Tagged tables**: an id at or above N returns −1. A zero N gives a record of 0x10 bytes only.
- **Entity and placement ids** (tags 0x12 and 0x13): an id at or above the entry count, or an entry whose region number has no created, loaded slot, gives no record; the calling host routine then uses its default. A slot byte of 0xFF always means the table's own pack's embedded region.
- **Embedded ids that collide (quirk)**: in 15 packs the 0xFF entries of the entity table are not at index = ordinal, so an embedded character's own id and kill bit equal the id of a slot entity (Tagged tables, Embedded entries on disc). Reproduce it as is.
- **Binary grids, off by one (quirk)**: the query treats only an index below 0 or above W × H as outside the block. An index equal to W × H reads the byte just past the block's cells (Binary grids, Query). Reproduce it as is. A query for a key with no block passes.
- **Type 38**: not decoded; the loader logs it and continues.
- **Entry records absent** (13 packs, Entry records and teleport planes): the world spawn vector stays (0, 0, 0, 1) with the default facing (1, 0, 0, 1); a load with p = -1 sets the start pose to the zero vector for both position and facing; the interior jobs keep their default L and G (then snapped, on entering); a teleport does not reset the start pose.
- **Leave-interior entry bound**: the entry index is bounded by the interior buffer's entry count but read from E's list (Interior entry steps). Example: b = 5 with 71 entries in E and 3 in the interior lands on E's entry 0. Reproduce it. No interior pack lacks entry records (Open questions), so the case of an interior without a type-11 chunk does not arise on disc.
- **Plane index out of range**: a teleport landing on a plane index at or above the target's teleport plane count is refused (logged); a driver load then falls back to entry record 0.
- **World spawn probe miss**: when the spawn probe gives a height below -1e-7, the spawn keeps the record's y (50.0 for 909's entry 0).
- **Deadly-ground respawn index**: an index at or above the current world's entry count would keep the previous start pose; all 79 indices on disc are in range (SPEC-0055).
- **Pack with no chunks**: the finalize steps would run with the shader-list and grid pointers left over from the previous area. Not reached on disc, because every pack starts with a type-31 chunk. A clean implementation should treat a pack with no chunks as an error.
- **Type-3 slot index**: the loader reads it as 0 to 4. It is 0 in every pack on disc, so slots 1 to 4 are never used. The loader's handling of other values is not described.
- **More than ten type-16 chunks**: the extra chunks are dropped without a message. Not reached on disc (75 chunks in total; one pack has two).
- **Slots with no row match**: a slot outside 0 to 192 has no row. No script on disc passes such a value (Area numbering).
- **Shared rows**: packs 1391 and 1414 each occupy two rows, so a slot of either row loads the same pack; a pack-to-world-id lookup returns the first row.
- **Shared level scripts**: several packs have a level module whose payload is identical to another pack's (for example 686 and 678, 970 and 956, and 1430, 1431 and 1433 with 1429). Their code is the same, but each pack is a separate area with its own world-pack data.
- **Pack with no level module**: 18 packs have no level module, including 1121, which is a transition destination (slot 107) and loads with no script. Fifteen of the 18 are transitions (World kinds).
- **Block pairs and the swap**: the block pairs never block a world swap (the swap caller passes the current pack as the argument, so it compares (current, current)), but they do stop the second world from being drawn: with current 797 and second 758 (a pair), 758 is updated but not drawn. Reproduce both.
- **Kind refusals**: StartEnterInterior and the exterior replacement calls do nothing but report an error unless E is current; StartLeaveInterior does nothing unless I is current. On disc, the door callback of transition 706 calls StartEnterInterior(984) while T is current, so that call is refused and loads nothing. The door step looks up the callback in the current world's teleport table, using the touched door's number, so 706's callback can only start while 706 (T) is current, and it is always refused (High). In play, door 0 of 706 shows "Enter" (prompt 13). On a Square press it shows level message 1 when game flag 67 is clear and game flag 64 is set; otherwise it makes the refused enter call and waits 3.0 seconds. The hero stays in 706. Interior 984 is still reached from exterior 956 (its door into 984).
- **Interior-kind packs in E**: 1429, 1430 and 1431 export only Interior-family module entry points, but 1427 streams them into the exterior buffer E. The host selects lifecycle calls by buffer family, so these module copies receive no Init, Update or Exit calls in E and their script callbacks are neither registered nor stepped. No leave-interior callback is refused: default door and world-cell behavior remains, and the hero can walk back into 1427. The three leave-interior edges from these modules are excluded from the new-game reach graph: their I-buffer source states are not reachable, and the packs occur only in E. Whether the family/buffer mismatch was intended remains unknown.
- **Exteriors without the swap**: 19 exteriors fail the neighbour-world test (World buffers and transitions), so the hero cannot walk out of them onto a streamed neighbour; they are left by doors, script calls or a return after a movie.
- **Interior 1008 draws E**: 1008 (slot 85) is the only interior in the swap set, so while it is current the exterior E's world and characters are drawn beside it (not E's placed objects, which need a current world other than I). Whether this is intended is open; reproduce it.
- **Second world without a pairing gate**: the second world's region pass (character updates, streaming, kill bits) runs whenever it is loaded, even when the pairing test fails and while I is current.
- **Arrival door out of range**: pack 1298's default door asks for arrival door 2 of 909, which has only doors 0 and 1. Pack 1298 is not reached, so this does not occur in play.
- **Stale door confirm (quirk, Medium-High)**: the script-door path never clears the door-confirm flag. After a script door with a context above 0 is confirmed, the flag stays 1 until a default door consumes it, so the next default door the hero touches runs the default rule on its first contact tick, with no prompt shown and no press. Reproduce it as is.
- **Door prompt suppressed**: while a talk candidate with a registered talk callback claims the press, or a buyable item is chosen (SPEC-0022, Interaction prompts), the chooser neither shows the door prompt nor sets the confirm flag, so the door cannot be confirmed. A hero in a special state cannot confirm a door either, because the chooser shows the door prompt only on its idle path.
- **Stale entry message between 1139 and 1129 (quirk, High)**: the level script of 1139 posts entry message 10 for slot 109 (1129) on every load, and the level script of 1129 posts entry message 10 for slot 114 (1139) on every load (blackboard, SPEC-0030). When 1139 is next entered after any visit to 1129, its script finds message 10 and places the hero at 1139's entry record (locator) 1 (the placement it uses for value 10) instead of the load's own arrival, even when the entry is through 1023's door 9, which posts no message. A 1129 entry reads and resets its own message, and the message that 1139 posts for 1129 is always overwritten before the next 1129 entry (1023's door 10 posts 20; 1139's door 1 posts 10). Reproduce it as is. Whether the blackboard is cleared at new game, load or world change is not established (SPEC-0030).

- Area-pair test with zero entries: a current pack of zero and an argument of zero match the zero entries, so the test returns 1. No pack is numbered 0, so this does not occur in play (High).

## Test vectors

Synthetic, for region-slot loads: Random(0, 2) with r = 7 gives 0 + (7 mod 3) = 1; Random(5, 5) always gives 5. A pack P = 1000 with a plain zone (normal j 4, replacement j 5, flags 10 and 11): flag 10 set and 11 clear loads region 1006; flag 10 clear loads 1005. A cycle zone with values (2, 3, -1) and a draw of 2 uses value 0, j = 2, and loads region 1003.

Synthetic, for the area-pair test (not the disc table): with pairs (10, 11) and (12, 13) in that order, the current id 10 and argument 11 give 1; the current id 11 and argument 10 give 1 (either order); the current id 10 and argument 12 give 0.

Transitions (disc check values; read the tables to check a reader, do not copy them into code):
- Pack 909 current in E, hero over a cell with value 1: entry 1 of 909's tag-0x0B list is world 11, which is slot 5, pack 691, a transition, so 691 is streamed into T (unless T already holds it). Cell value 3: world 615, pack 951, an interior, so nothing is streamed.
- Pack 1427 current in T, its trigger 3 sets pending slot 189: the next pending step loads pack 1430 into E.
- Pack 1139 after a 1129 visit: the hero visits 1129 (whose script posts message 10 for slot 114), returns to 1023 and enters 1139 through 1023's door 9. 1139's script reads message 10 and places the hero at its entry record (locator) 1, not at the load's arrival.
- Pack 722, door 3 (a teleport callback): enters slot 24, pack 745, as an interior. Leaving 745 by its default door: its cell value 1 names 722, and its tag-0x0E entry at door 0 is 4, so the hero arrives at door 4 of 722, and T is reloaded with the transition remembered at entry.
- Pack 1098, door 3: shows a level message only; nothing loads.
- Pack 706 current in T, door 0, game flag 67 set: "Enter" is shown; a Square press makes the enter call into 984, which is refused (T is current); no load is queued and the hero is still in 706 after the 3.0 second wait. With flag 67 clear and flag 64 set, the same press shows level message 1 instead.
- Interior entry records: every one of the 50 interior packs has at least one entry record, so an enter-interior job with no plane always lands on that interior's entry record 0.
- Default door, E current (synthetic): on tick t (first contact) the door step posts prompt 13. On tick t+1 "Enter" is shown; a Square press on tick t+1 sets the confirm flag, and the door step on tick t+1 clears it and runs the default rule.
- Default door, I current (synthetic): the door step posts prompt 26 ("Exit").
- Script door, context 13 (synthetic): a Square press on a tick when "Enter" is shown starts the callback on that tick, and the confirm flag stays 1. When the hero later touches a default door, the default rule runs on the first contact tick, with no prompt.
- Script door, context 0 (synthetic): the callback starts on the first contact tick, with no press.
- Synthetic swap: E current with an exterior whose slot is in the swap set and whose pack is not in the no-swap list; the hero steps off its ground onto ground that exists only in T: the next frame makes T current. The same step from pack 906 (slot outside the set) finds no ground and no swap.
- Neighbour-world test (disc check): current 909 (slot 68), second 691: 1. Current 797 (a transition), second 758: 0 (a block pair), so 758 is updated but not drawn. The swap test with current 797 passes q = 797: 1. Current 1299 (no-swap list): 0. Current 906 (slot 66, not in the set): 0. Current interior 951: 0; current interior 1008 (slot 85): 1.
- After-movie reload (disc check): E current, no return world or return transition set, T holds 691: T is reloaded with 691 (p = -2, make-current 0) and the hero keeps its position.

Area and entity names (disc check values; read the tables to check a reader, do not copy them into code):
- Slot 107: pack 1121, world id 1362, a transition, map area 49, "Meridell Castle 2nd Floor".
- Slot 5: pack 691, world id 11, a transition, map area 8, "Farm Road".
- Slot 192: pack 1433, world id 11070, an interior, map area -1. It is entered from 1300 (map area 57, "Fyora's Palace Gardens"), so its fallback label is "inside Fyora's Palace Gardens".
- Pack 1023, entity 66: slot byte 11, ordinal 1, so resource 1023 + 1 + 11 = 1035, second kind-3 record: "Shanra".
- Pack 1163, entity 15: slot byte 5, ordinal 0, resource 1169: "Sir Harlag".
- Pack 1217, entity 7: slot byte 0, ordinal 0, resource 1218: "Juppie Monster". Pack 1217's table has 9 entries, so entity ids 9 and above find nothing there.
- Pack 880, entity 18: region 882, ordinal 11, past the end of that region's kind-3 list: no record, so the lookup finds nothing.

Module payload count (disc check): 173 script-bearing packs minus 10 repeated payload copies across the five duplicate groups listed in the open-question answer gives 163 distinct full module payloads. Counting code sections alone gives 162.

Hero placement (disc check values; read the records to check a reader, do not copy them into code):
- New game: 909's entry record 0 is at (-1852.8021, 50.0, 1137.4628), facing (0.59588, 0, -0.80307). Spawn y = H(-1852.8021, 1137.4628, 58.0), or 50.0 when that probe is below -1e-7. Both heroes spawn there. The camera eye after the respawn is (-1858.761, y + 2.5, 1145.493).
- Teleport landing on plane index 0 of 909 (upright; P = (-1892.4012, 39.9597, 1173.0106), w = 2.81902, F = (0.95330, 0, 0.30201), D = (0.30201, 0, -0.95330), Q = (0, 1, 0)): L = (-1887.209, 39.960, 1173.177) before the probe; probe ceiling 45.960; facing F, or -F with the reverse flag.
- Plane index 1 of 909: L = (-2021.700, 44.600, 964.659) before the probe, ceiling 50.600, F = (-0.55806, 0, 0.82980).
- Plane index 2 of 909: refused (909 has two teleport planes).

Hero placement (synthetic):
- Flat plane: P = (0, 10, 0), F = (0, 1, 0), D = (1, 0, 0), Q = (0, 0, 1), h = 2, w = 4: L = (4, 5, 2), no probe, facing (0, 1, 0).
- Upright plane: P = (0, 0, 0), F = (0, 0, 1), D = (1, 0, 0), Q = (0, 1, 0), w = 2: L = (1, 0, 5) before the probe, ceiling 6.0.
- Load with p = -1 into a world whose entry record 0 is at (1, 2, 3) facing (0, 0, 1): start pose and position (1, 2, 3), facing (0, 0, 1), no probe.
- Load with p = -2: the hero stays at its current position and forward.
- Leave interior with b = 5, 71 entry records in E and 3 in the interior: E's entry record 0 is used.
- Entry list addressing: list address A and i = 3: the record is at A + 0xF0, its position at A + 0x120 and its facing at A + 0x130.
- World with no entry records: spawn (0, 0, 0, 1), facing (1, 0, 0, 1).
- Deadly ground: a death on a kind-2 ground entry whose byte +0x0D is 3 starts timer F; on the first pass at which it exceeds 1.5 s (pass 91 at 1/60 s per pass) the hero is at entry record 3 of the current world, in state 60. Byte 0xFF: no respawn; the game-over timer starts instead (SPEC-0055).

Character activation (synthetic):
- Embedded block, cap 6, nine eligible enemies all within 280 units at squared distances 100, 200, ..., 900: the six nearest (100 to 600) are bound. The nearest unbound record (700) replaces the farthest bound one (600) only if 700 + 20 < 600, which is false, so no swap. If instead the nearest unbound record is at 450 and the farthest bound one at 500, then 450 + 20 < 500 and they swap (one swap per tick); at 490 they do not, since 490 + 20 is not below 500.
- Kill the enemy of record 3 of pack P, leave P by a driver load (a door replacement load or the after-movie reload), enter P again: record 3's kill bit was stored, so it is not bound.
- Kill memory loss (disc check, Medium-High until confirmed at run time): kill record 3 in transition 691, then stream a different transition into T from the exterior: 691's bits are not stored, so when 691 is loaded again record 3 is bound again. The same kill followed by a save while 691 is current or second is kept.
- Distance limits: a bound character at d = 78,400.0 is kept, one at 78,400.1 is released. A record at d = 78,400.0 is never bound.
- Dead enemy with countdown 3: its kill bit is set on each of the following ticks, and it is released on the tick its countdown goes below 0.
- The same with a Dumb Skeleton (row 226, a reanimating row): the record is bound again.
- A record with summon byte 1 and a zero position stays unbound until a spawn-by-key request with its key binds it at the request's point.

Synthetic:

- Record walk: a table holding a kind-4 record (0x40 bytes), a kind-2 record with n = 3 (0x10 + r16(28) = 0x30 bytes) and a kind-3 record (0xF0 bytes) has records at offsets 0x00, 0x40 and 0x70, and ends at 0x160.
- Grid: x min −100, x max 130, z min 0, z max 50, cell 25 gives cells-x = ceil(230 / 25) = 10 and cells-z = 2.
- Region number: pack 900 with script slot j = 2 loads resource 903.
- Zone lookup: if the type-1001 zone id equals table B's row *k*, the canonical pack is table A's row *k*.
- Pack record: pack 678 has map area id 8, so its map page is area 8 (Farm Road, map picture resource 609). A synthetic pack record with words (zone 11, map area -1, selector 3) gives no map area of its own; a pack with no type-1001 chunk keeps the default record (zone -1, map area -1, selector 0).
- Type 15 range: a 0x0A table with N = 3, A = {1, 4, 7}, B = {3, 4, 9}: range 0 has count 3 (1 to 3), range 1 has count 1 (4 to 4), range 2 has count 3 (7 to 9).
- Type 15 length: a 0x0A table with N = 3 is 0x10 + round16(14) = 0x20 bytes.
- Type 15 tag 0x12 or 0x13 with size 0x30 (three entries) is 0x10 + 0x30 = 0x40 bytes; its entry count is 3.
- Type 15 entry bytes: the little-endian u32 0x00000100 is byte 0 = 0x00 (slot 0) and byte 1 = 0x01 (ordinal 1). The u32 0x00000001 is slot 0 with ordinal 0, and the u32 0x000000FF is region byte 0xFF (−1, the embedded region) with ordinal 0.
- Pack 678 (four type-36 regions, K = 4, so the header value at +4 is 5), tag 0x12 at payload offset 0x1D0 with size 0xB0: 11 entries with (slot, ordinal) pairs (0,0) (0,1) (0,2) (0,3) (1,0) (1,1) (1,2) (2,0) (2,1) (3,0) (3,1). Slot 0 has four entries, matching the four kind-3 records of region 679; slot 1 has three (region 680); slot 2 has two (region 681); slot 3 has two (region 682). The first four entries' u32 words are 0x0, 0x100, 0x200 and 0x300, and entry 4 is 0x1.
- Pack 678, tag 0x13 at payload offset 0x290 with size 0x50: 5 entries with (slot, ordinal) pairs (0,0) (0,1) (0,2) (3,0) (0xFF,0). The header value at +4 is 5.
- Pack 685 (no regions, K = 0, header value 1): tag 0x12 has 8 entries and tag 0x13 has 6 entries. Every slot byte is 0xFF, and the ordinals are 0 to 7 and 0 to 5.
- Synthetic entity lookup: a tag-0x12 table with entries (slot 0, ordinal 0), (slot 0, ordinal 1) and (slot 1, ordinal 0). Slot 0 is loaded with characters X and Y in table order, and slot 1 is not loaded. Id 1 gives Y. Id 2 gives nothing because slot 1 is not loaded. Id 3 gives nothing because it is at the entry count.
- Synthetic reverse id: the same table; character Y (region number 0, record 1) has id 1. An embedded character of record 2 has id 2 and kill bit 2, whatever the table holds.
- Collision quirk, pack 714: entity 13 is embedded character 0. Its own id and kill bit are 0. Killing it sets kill bit 0, and bit 0 also marks slot entity 0 as killed.
- Binary grid: a block with W = 3, H = 2 has N = 6, offset 0x14, and 6 bytes of 0/1 values.
- Binary grid query: with W = 3, row 1 and column 2 the index is 1 × 3 + 2 = 5, so the cell byte at index 5 decides the query. Row 1, column 0 is index 3. With H = 2 (N = 6), index 6 reads the byte just past the cells (quirk), and an index of 7 or more passes.
- Zone matrix (synthetic): block 0 with W = H = 3 and cells 1,1,0 / 1,1,1 / 0,1,1 (row by row). R = 0, c = 2: i = 2, inactive (0). R = 2, c = 0: i = 6, inactive (0). R = 1, c = 2: i = 5, active (1). A key with no block: 1.
- Zone matrix (disc check): pack 1184 has a block keyed 0 (world 0, pack 678) with W = 17 (678's block-0 size) and H = 19 (1184's block-0 size).
- Save-point choice: a region has save points A (index 0, squared distance 900 from the hero, flag 0), B (index 1, squared distance 400, flag 1) and C (index 2, squared distance 100, flag 0). The nearest nonzero-flag point is B (index 1), and the nearest zero-flag point is C (index 2). The glow sits at C. Cue 0xA8 plays (100 is below 225); no prompt is shown (100 is not below 25).
- Save-point sound and prompt: a save point at squared distance 200 plays cue 0xA8 (200 is below 225) but shows no prompt (200 is not below 25). At squared distance 225 exactly, neither (both tests strict). At squared distance 16, cue 0xA8 plays and "Save Game" is shown.

Disc check values for the type-36 record kinds (read the records to check a reader; do not copy them into code):

- Region 534, kind 3 at record offset 0x00: position (320.737, 0.0, 650.648), w 1.0, unit facing, empty name, main skeleton 0, subskeleton −1, face skeleton −1, model set 0, animation indices 0, −1, −1.
- Region 534, kind 2 at offset 0xF0: N = 110, record span 0x390.
- Region 534, kind 17 at offset 0x480: size 3,728 (0xE90), all bytes zero. Its record span is 0x10 + size, and it ends at the end of the chunk payload.
- Region 678, kind 4 at offset 0x00: position (−1706.435, 45.558, 1193.245), w 1.0, flag 0.
- Region 678, kind 8 at offset 0x40: position (−828.31, 58.2, 1202.8), flags 0, drop group −1, primary key 56, second key 1584, luck gate 0, count 0.
- Region 679, kind 16 at offset 0x2890: list id 2302, n = 1, constants 1 and 2 (see "Kind 16, effect list" for the entry values).
- Synthetic, kind 16 span: n = 1 gives 0x70 and n = 2 gives 0xC0.

Box records (type 16), disc check values for two packs (read the chunk to check a reader; do not copy them into code):

All offsets below are from the chunk header.

- Pack 678 (N = 71). Record 0: box (−1015.50, −1012.81, 1320.54, 1325.01); pool count 4; pool offset 0x11E0 (= 0x20 + 0x40 × 71); key 9; height 74.1661; zone 0; jitter flag 0; overrides zero; +0x38 0xCDCD0000; +0x3C 0xFFFFFFFF. Pool entries as (item count, row z offset, item offset): (5, 2.073, 0x2C0C), (2, 3.0792, 0x2C5C), (1, 4.4705, 0x2C7C), (1, 0.0, 0x2C8C). Its centre is (−1014.155, 74.166, 1322.775). Row 0 lies at z = 1320.54 + 2.073 = 1322.613, and its plant 1 at x = −1015.50 + 1.2959 = −1014.204 (no jitter).
- Pack 678, record 1: box (−1015.37, −1012.38, 1325.45, 1328.81); pool count 2; pool offset 0x1230 (record 0's pool offset + 4 × 0x14); entries (6, 0.6104, 0x2C9C), (3, 2.7393, 0x2CFC). Its first item block starts right after record 0's last one (0x2C8C + 16 × 1).
- Pack 678, items 1 to 4 of record 0's entry 0 (at 0x2C1C, 0x2C2C, 0x2C3C and 0x2C4C): (1.2959, 3.0, colour 0x80404040, 173.4805), (0.3235, 3.0, 0x80404040, 173.4805), (2.1971, 3.0, 0x80404040, 173.4805), (2.6904, 3.0, 0x80404040, 173.4805).
- Pack 713 (N = 34). Record 0: box (2758.25, 2765.45, −921.47, −909.19); pool count 4; pool offset 0x08A0 (= 0x20 + 0x40 × 34); key 5; height 118.5894; zone 1; jitter flag 0; overrides zero; +0x38 0xCDCD0001; +0x3C 0xFFFFFFFF. Its four pool entries each have item count 1, at item offsets 0x1138, 0x1148, 0x1158 and 0x1168.
- Synthetic: N = 2 gives records at header +0x20 and +0x60 and a record 0 pool offset of 0xA0. A record with pool count 3 at pool offset P has entries at P, P + 0x14 and P + 0x28, and the next record's pool starts at P + 0x3C. Two entries with item counts 2 and 3 whose first block is at B: the second block is at B + 0x20, and the next entry's block at B + 0x50.

Census on disc (counts of packs having a feature; checks, not behavior):

- Of the 192 packs whose first chunk is type 31, 191 have header version 2.1. The front-end pack has 2.0.
- Every chunk type except 4 and 16 occurs at most once per pack. Type 4 occurs 559 times in 192 packs. Type 16 occurs 75 times in 74 packs; one pack has two.
- Type 20 occurs once in each of 23 packs.
- Type 38 occurs once, in one pack, and is skipped.
- In all 191 game packs, the texture count stored in the shader list equals the number of type-4 chunks in the pack (SPEC-0003).
- The slot index field of every type-3 chunk is 0.
- Order: type 2 comes before types 4 and 29 in every pack that has them. Types 5 and 10 come before type 29 in every pack that has type 29. Type 19 comes before types 4 and 35 in every pack that has them. These hold in every pack on disc.
- Entry list size (synthetic): three entry records of 0x50 bytes occupy 240 bytes (0xF0).

Kill block, loading and slots (synthetic unless marked):
- Kill bit 255 sets byte +0x5F, mask 0x80. A request for bit 256 or above changes nothing and a test of it returns 0; the spare byte at +0x60 stays 0.
- Slot table: area slot 109 uses table bytes 109 × 33 = 3,597 to 3,629 (the table is 6,369 bytes for 193 slots).
- Release of a character whose AI mode word is 7, standing at (10, 0, 5) with its record at (0, 0, 0): the record keeps (0, 0, 0), so the next bind places it at (0, 0, 0). The same release with mode 8 writes (10, 0, 5) into the record.
- Disc: a Skeleton Archer record of embedded block 685 has default mode word 7; a Restless Spirit record of the same block has 8.
- Slot buffer: SlotLoad of region j into slot 2, then SlotLoad of region k into slot 2: the second region is read into the same buffer start as the first; no new buffer is taken from the world pool.
- Chunk order (disc): in every world pack the first type-8 chunk comes after the type-2 chunk.

### Test vectors: a real spatial grid (disc)

- Pack 909, the type-19 chunk at resource offset 2,398,304 (size 252,384): x from -2463.3989 to 3512.2166, z from -1079.0006 to 1848.0059, cell size 25. Cells along x = ceil(5975.6155 / 25) = 240 and along z = ceil(2927.0065 / 25) = 118; the cell table holds 28,320 entries, of which 1,524 are non-zero. The table offset in the header is 136.
- The new-game start position (-1852.8021, 1137.4628): cell x = floor((-1852.8021 + 2463.3989) / 25) = floor(24.42) = 24 and cell z = floor((1137.4628 + 1079.0006) / 25) = floor(88.66) = 88, so table index 24 + 88 x 240 = 21,144. Its entry is the offset 188,832, and the cell record there has list A count 10, entry count 10 and list B count 0.
- The point (-1849.04, 1135.42) falls in the same cell (24, 88).

## Open questions

- Resolved: the trigger-to-zone mapping for modules 758, 991 and 1023, the slot rule, the firing rule (state 1 only) and the plain load steps are given under Asset-swap tables. A zone load resets its slot first, so it unloads the region that slot held.
- Resolved (R-0032): the later passes of the cycle thread draw nothing and step the candidate index, as stated under Cycle load. The 1023 Meerca slot helper takes one argument: 0 frees a slot and loads region 1025 (a direct load of index 1, keep 1, activate 1; not an asset-swap zone), and 1 restores the slot's saved index or clears the slot (SPEC-0045, Open question 27). The free-slot finder of 1023 that the helper uses (slot 0 when its record word is nonzero; else slot 1 when its record word and the set-0 enable word are nonzero; else none) has no other caller in module 1023 (High).

Numbers of removed questions are not reused.

- **Which regions an area loads**: Resolved for the fresh New Game start in pack 909: its activation thread loads region resources 910, 914, 920 and 926 into the fourth, P, Q and R slots, respectively. Region 910 is selected while flag 110 is clear; the other three are the no-stage family placements (pack 909 in SPEC-0045, Activation thread). **[High]** The remaining per-area region sets belong to the respective level-script specs (Region slots); do not assume that every resource in a contiguous run is loaded.
- Resolved: **Player start position.** A new game spawns both heroes at 909's world spawn vector, which is 909's entry record 0 with its y probed from 8 units above (Hero placement, New game). The save-point (kind-4) records do not place the player: saving at one records the hero's own position and facing as the restart point (see Kind 4; SPEC-0025). 105 world packs with an embedded region have no save point. **[High]**
- Resolved: **Type-3 and type-5 walk-time setup.** Type 3 fixes the level tree's payload-relative links against the tree payload, supplies the current shader list for material handling, and stores the tree in a populated display-tree slot selected by the header slot byte. Values above 4 select slot 0; all type-3 chunks on disc use slot 0. Type 5 relocates its model-set entry, cell and record data and links frame records to their owning set; its patch-table registration remains in grid finalize step 1. The meaning of alternate display-tree slots and type-3 visibility-selector values remains open (How the loader handles each chunk; SPEC-0004). **[High]**
- **Type 2 sub-record**: the loader runs a second fix-up on the shader list when a sub-record is nonzero. The sub-record is not identified (SPEC-0003).
- Resolved: **Type 8 before type 2.** No. In every world pack that has type-8 chunks (191 of 192) the single type-2 chunk comes first (How the loader handles each chunk, Ordering rules). **[High]**
- Resolved: **Type 32 block and region slot.** They are one record type with one layout: the type-32 handler runs the same constructor as a slot. The block shares the world pool, has region number −1 and reads no data (type 32 row of the chunk table). **[High]**
- Resolved: **Region slot buffers.** They do not return. A teardown rewinds the slot's own buffer handle, and the next load into the slot reuses the buffer; the world reset only drops the slot table (Region slots). **[High]**
- **No-chunk pack path**: whether the finalize steps with stale pointers (Edge cases) are reached through any level script. Not seen.
- Resolved: **Checkpoint reader**: the nearest save-point index is used by the save-point step itself, to place the glow and to play the save-point sound; the save point opens the save screen (see Kind 4). No code places the hero at a save point's position. **[High]**
- Resolved: **Checkpoint reference position**: the distances are measured from the selected hero's position. **[High]**
- Resolved: **Skeleton indices beyond the region's own family count.** Each loaded character-region record has separate main, sub and face arrays and counts. A nonnegative index below the corresponding local count selects that region's array entry; an index at or above the count skips the attachment in this setup path. There is no cross-region fallback here. The 569-record census reflects local range mismatches, not shared area-wide indexing. The downstream effect of a skipped skeleton attachment is not established.
- **Cell order** of the spatial grid (x or z major) and the second grid table at 0x24. Cell records and their lists are in SPEC-0018.
- Purposes of the type-39 spawn fields beyond SPEC-0028, and of types 30 and 34, are not decoded here; the run-time use of types 40 and 41 is Medium-Low.
- **Type 15**: whether the tagged tables are collision or navigation data. Answered for tag 0x0D: the loader stores it at world +0x5908 and nothing reads it (High).
- Type 35. Resolved: the row is set on every pass by the camera's mode update (and by the camera reset and make current); the column is the zone number of a ground entry, or the object's zone key for the draw gate (Binary grids; High).
- Whether anything reads the first word of the requested world block after the zone chunk.
- Resolved: **Type-1001 words after the zone id.** Record word +4 (payload + 0x14) is the map area id (0 to 76, or -1 for no map), used by the map page and the save-slot location name; word +8 (payload + 0x18) is the sound selector (SPEC-0009). See "Zone id". **[High]**
- Pending area change. Resolved: the queued change is the pending step, which compares the current world with the transition and exterior buffers; SetWorldLoadOverRide is its slot writer (World buffers and transitions; High, Medium for the re-assertion while game-state word +0x44 is nonzero).
- Entry records and player start. Resolved: the 0x50-byte entry records are in the type-11 chunk; a driver load picks one by the request's plane index, the interior jobs by entry index, and a new game uses 909's entry 0 (Entry records and teleport planes; Hero placement; High).
- Resolved: **Transition spawn records.** The 0x50-byte record read by the interior entry steps is an entry record of the arrival world, selected by list position, not the body of a door's rectangle volume. A named plane uses the teleport plane list instead. **[High]**
- Resolved: **Transition order.** The fade, wait, place, snap order belongs to the interior jobs (Interior entry steps); the arrival formulas of SPEC-0026's area entry steps are the same jobs' hero placement. **[High]**
- Resolved: **Ground snap offset.** The +6.0 is a fixed constant of the program (probe start = landing y + 6.0), not a run-time word. **[High]**
- Resolved: **Attached actor.** The actor placed with the hero is the partner hero. **[High]**
- Resolved: **Script start step and post-load placement.** The start step (the module's Init) runs first, inside the load, before the post-load placement and before the world is made current (SPEC-0030). After a driver load, a start pose set by Init is therefore replaced by the placement at once. **[High]**
- Resolved: **Type-11 finalize order.** It runs inside finalize step 2 (Activation), before the object-table setup, the type-40 setup and the object link, so after the grid finalize and before the pools, the post-load helper and the type-35 key. **[High]**
- Resolved: **Door and teleport plane counts.** Doors and teleport planes are the same 169 records in 102 packs (plane records with bit 1 and a class other than 500 and 501); each door's +0x1C at load is its ordinal among them and equals the disc value. **[High]**
- Resolved: **Same-world pad enable word.** The pad lists are built from the class-500 and class-501 planes of every pack, with no other condition. The world word at +0x5814 is the display node with id 401 of the type-8 chunk; only pack 1299 has one, and it gates only the looping-door puzzle set-up (SPEC-0028) (Teleport plane list). **[High]**
- **Entry record words.** Resolved: +0x10 to +0x1F are the common primitive header; +0x14 is an editor serial, +0x1E the locator tag, +0x1A explains the two pack-699 index differences (Entry records and teleport planes; High). Still open: the meaning of flag bit 0 (0x0021 in 40 records), which has no known reader.
- Resolved: **New-game load mask.** The 0x65 mask tests bits 0, 2, 5 and 6 of a singleton runtime-object word at +8, not controller input bits; their meanings are unknown. The field is initialized to zero and no later writer was found in the inspected boot-program call sites, so normal controls and the debug input path cannot reach the branch in this build. If the branch were entered by external mutation, it would skip both loading 909 into E and making E current, leaving 691 current. **[High for the code and inspected paths; external mutation unknown]**
- Resolved: **Interior without type-11.** Not reached on disc: all 50 interior packs have a type-11 chunk with at least one entry record. The 12 world packs without entry records (718 and 1082 with no type-11 chunk; 702, 703, 713, 719, 720, 721, 1079, 1121, 1391 and 1427 with a count of 0) are all transitions. **[High]**
- Resolved: **Saved world ids.** A save holds two world ids: the one at save record +0x24 is reloaded into T and the one at +0x28 into E (ids with no row in table B are skipped). A save names only packs that were resident when it was written, so it adds no entry to the reach count. **[Medium-High]**
- Resolved: **Pack 1170 enter call.** The enter call to 1170 sits in a routine of module 1098 that is never registered. Door 3 of 1098 is registered to a separate callback that only shows a level message. So 1170 cannot be entered; the door was probably replaced by the message (cut). **[Medium-High]**
- **Module 991 switch arm (resolved).** Module 991 installs the same context-13 teleport callback for door indices 0 to 7 only when flag 85 is clear; its Init then sets flag 85. While registered, the callback selects its destination by the touched door's index, with no story-flag test in the callback: indices 0 to 3 enter interiors 84 to 87 (packs 1005, 1008, 1011 and 1014), index 4 loads exterior 88 (pack 1017), index 5 also loads exterior 88 (pack 1017), index 6 enters interior 89 (pack 1020), and index 7 loads exterior 63 (pack 880). The shared door rule invokes a context-13 callback after its normal door guards and a Square press on the Enter prompt. Thus story progress does not choose among the destinations; flag 85 gates whether this callback set is installed. See SPEC-0045 for the blackboard return values and reverse route. **[High]**
- Resolved: **Slot argument source.** Every world-taking call site in the level modules (194 sites in 66 modules) passes a fixed slot number, except the helper of module 1427, whose callers pass 187 to 191 (Transitions take slots). **[High]**
- Resolved: **Start-key words.** Both words hold -1 in the boot program's data, the only stores in the whole boot program write -1, and no level module refers to them, so they are always -1 (Default start). **[High]**
- Resolved: **Packs with no known script.** Pack 861's level module is an empty transition module of 392 bytes: its init, update and exit return at once and it makes no host calls, so it loads no region and makes no transition (High). The world-taking calls were counted over every module image, so the seven packs whose module equals another pack's (686, 691, 970, 1129, 1430, 1431 and 1433) are covered by that code. 1004 and 1082 are not reached and nothing names them, so their outgoing calls cannot change the reach. **[High]**
- Resolved: **Scene-start command.** StartTransitionScene has no live call on the disc (its two sites are in code that nothing reaches), so the scene path of "Other routes" is unused in play. **[Medium-High]**
- Resolved: **Distinct level-script modules.** Of the 173 packs with a module payload, 163 full payloads are unique by exact bytes. Five duplicate groups repeat 10 payloads in total: (678, 686, 691), (798, 861, 1004, 1082), (956, 970), (1077, 1129), and (1429, 1430, 1431, 1433). Comparing only code sections gives 162 distinct values, a different measure; use 163 for distinct full module payloads. [High]
- Packs 1121 and 1129. Resolved: 1121 (slot 107) is a transition with no level module, streamed between 1098 and 1122; 1129 (slot 109) is an exterior entered by replacement loads from 1023 and 1139 and streamed from 1130, and 1077 is an unused copy of it (Area numbering, Zone variants; Medium-High).
- **Movies.** Resolved: boot code queues only movies 0 and 86 (the attract routine). The queue add has two other callers: the script host call PlayFMV, which passes the fixed index written at each call, and a small wrapper that nothing calls and that would queue a data word holding -1, which the add refuses. Of the indices asked about, movie 4 is queued by pack 1023, movie 35 by pack 777, movie 67 by pack 1398 and movie 70 by pack 1382 (the six-pedestal puzzle); movie 68 repeats movie 67's file and 87 is the attract loop file, and neither is queued by any code. The world-transition code takes no movie index: SetReturnWorldForFMV and SetReturnTransitionForFMV take a slot number, and the return after a movie is posted when the queue empties (see "Return after a movie"). SPEC-0026, Movie play sites, lists every call. High.
- **Blackboard-only slots.** Six slots (packs 722, 732, 864, 1170, 1225 and 1353) are used as keys in the blackboard store as well; that use is not a load. All but 1170 are entered by other routes. Why a world is keyed this way is unknown.
- Resolved: **Pack 678.** Pack 678 (slot 0, world id 0) repeats the payload of the start transition 691 and cannot be entered; the default start loads 691 and 909 (Area numbering). **[High]**
- Resolved: **Duplicate-row lookup.** A pack-to-world-id lookup returns the first matching row, so packs 1391 and 1414 map to their lower slot. **[High]**
- **Area object lifecycle (added):** which code calls the constructor, the reset and the unload (all reached through the method table; the unloading caller is not identified). The base initialiser that the constructor runs first is not described. The 33-byte block at offset 0x40 is the kill block (Kill memory; its last byte is a spare that stays 0). The meaning of the sub-object groups is open. Which of the five release helpers frees which buffer is open (slot buffers do not return to the world pool; Region slots).
- Resolved: **Area-pair table stride and size.** 25 entries of 8 bytes (two u32 pack numbers each): 23 pairs and two zero entries. **[High]**
- Resolved: **Area-pair meaning.** A pair links two exteriors entered from each other by a door replacement load; the pending step and the default door rule read the test (Area-pair rule). **[Medium-High]**
- Resolved: **Buffer load request rules.** The three apparent queues are the E, T and I buffer paths. E/T loads each have one in-flight read, with either skip-if-same or unconditional-reload behavior chosen by the caller. Interior entry uses its own I-buffer operation and refuses while T is reading; it does not queue a later request. The three-entry test is in an interior-load variant with no boot-program caller. The exact asynchronous completion pass remains open (Buffer load requests; area-change and world-lifetime evidence). **[High]**
- Resolved: **Current area id word.** The pair test compares pack numbers: the current world's pack and the target pack. **[High]**
- Resolved: **Save-point flag writer.** Every save-point flag is zero on disc, and nothing in play writes it. The region keeps only pointers to the disc records (a list of up to eight, filled at load and cleared at reset), and the only code that reads the list is the per-tick save-point step, which tests the flag and never stores to it. The two other accessors of the list (an indexed getter and a count getter) have no caller, jump or table entry in the boot program or in any level module. A save does not store region data, so no flag survives a save either. A clean build keeps every flag at its disc value and may omit the nonzero-flag branch. **[Medium-High]**
- Resolved: **Kind 17 arena.** The block is a per-region run-time arena for character pose buffers. Its size is the sum of the main, sub and face skeleton allocation costs of the region's kind-3 records, and its pose and matrix areas are initialized to identity. It is written at run time, not loaded from another disc location. The formula matches all 565 regions with a type-36 chunk and kind-17 record (Kind 17, High).
- **Kind 3 fields.** Resolved: the fields at 0x8C to 0x9C (table above: 0x8C is stored only, 0x8E to 0x96 are never read, 0x98 to 0x9C are tested only against −1) and 0x84 (default AI mode word). Still open: the other values from 0xB7 to 0xDF; the role of the float word at 0xC0 (perhaps the start of a second hold range with 0xC4); which enemy kinds use the selector, the range and the percent readings of 0xC8. Answered: the word at 0xAE is -1 in every record on disc, so the skeleton set it selects is unused.
- **Kind 8 fields.** Resolved: the byte at 0x21 is the snap-to-ground flag (read when the item is spawned), and the second key at 0x26 has no play reader (table above). Still open: the bit meanings of the flags beyond the two item-code tests.
- Resolved: **Kind 16 fields.** The list id and the constants at list offsets 0x08 and 0x0C are never read; entry offsets 0x00 to 0x0F are point 1, 0x3C and 0x3D are the ground-snap flags, 0x3F is the rumble gate, and entry bytes 0x4A to 0x4F are never read (table above). **[High]** Kind 2: the 4 or 12 tail bytes after its entries are not always zero (minor).
- Type 16 base and field names. Resolved: every offset in the chunk is relative to the chunk header; the pools are packed, and none overlaps another or runs past the chunk end (Box records; High).
- Resolved: **Type 16 build selector and purpose.** The build switches on the key at +0x18, which picks one of six foliage families with fixed models; each record is one foliage patch (Box records; SPEC-0060 for the families). **[High]**
- Type 16 fields: the 0xCDCD fill at +0x38 and the 0xFFFFFFFF word at +0x3C have no known reader (Box records).
- Resolved: **Type 16 thresholds and query point.** The far distance comes from the overrides or from the patch's model, and the logic radius is 100 units; the query point is the camera eye (SPEC-0060). **[High for the thresholds; Medium-High for the eye]**
- **Type 15 slot rules (tags 0x12 and 0x13).** Still open: the rule from an asset-group name to a region number. Answered: the slot byte is the region number j of a slot load (0xFF is the embedded region), which resolves 3,026 of the 3,028 tag-0x12 entries on disc (Tagged tables; Entity names; High).
- **Type 40 keys and fields.** Which callers reach keys below 200 (185 keys have no known path through the switch setter). The list kinds 0, 7 and 12 have no built-in set-up; on disc their readers are level scripts (SPEC-0048, Script groups). The meaning of the other unused kinds, the 0x60-byte gap at payload offset 0x534, and the owner of the switch table passed to the setter are open.
- Resolved: **Area-exit cell pointer and ground slot.** Type-19 load rebases each nonzero cell-table entry against the grid payload, so runtime entries are absolute cell-record pointers. Each motion-record tick looks up the cell under the actor and writes its cell-record pointer to +0x174; the pending step reads the low exit value from that cell. The ground probe's selected entry is a separate +0x178 value consumed by ground attachment and is not copied into the +0x174 collision-cell slot. The apparent +0x178-to-+0x174 copy belongs to a separate store-interaction state record (SPEC-0018, Hero ground-record slot and Grid cells and cell records). **[High]**
- Resolved: **Area-pair test at doors.** At a door the pair test selects the replacement load of the default rule; it is not a hero-side condition (Doors). **[High]**
- Setter argument source. Resolved: module 1427's eight trigger callbacks pass slots 187 to 191 (packs 1428 to 1432) and test no flag (Transitions take slots; High).
- **Type 41 output.** The variant key that the check reads is the pack's sound selector (the third word of the type-1001 record), a disc value from 0 to 11 that no run-time code writes (High). Cross-spec story context narrows some keys but does not identify every flag: a passing variant check enables a character skin-colour override (SPEC-0022, Variant key check; SPEC-0040, Behavior 26). Key 1's flag 83 selects the Act 3 Juppie encounter, and its clear flag 5 is set at that encounter's Act 3 victory; this bounds key 1 to the Act 3 encounter before its completion. Key 2's clear flag 322 excludes the Act 4 trap/intro sequence in pack 1428, which only runs when 322 is set; Act 4 itself starts at flag 332. The meaning of key 2's required flag 10 remains unknown. Key 3's flag 64 marks the post-Gnarfas Glade stage; the meaning of clear flag 283 remains unknown. Key 8 reads flag 248, the Act 3 start flag. Keys 9 to 12 require flag 332, the Act 4 start flag, with respectively flags 100, 102, 104 or 106 clear; those four clear-flag roles remain unknown. No story meaning is established here for keys 4 to 7 or their flags, or for the remaining unresolved conditions above. The exact blend formula, the fields of the link entries, and whether vectors A and B are rewritten at load also remain open. Answered: the output endpoints are the renderer's post-pass colour, screen overlay colour and depth-range state, plus the sky dome's role-10 opacity through the camera record; the camera itself supplies only the eye position (SPEC-0028 section 11; SPEC-0027; SPEC-0046, Sky domes). [High for destinations and stated flag contexts; rendering details belong to SPEC-0046].
- Resolved: **Door confirm flag.** The interaction chooser sets the flag on a Square press edge while it shows the door prompt (idle path only); the door step consumes it in the same tick for a default door, and a script door leaves it set (Door prompt and confirm; Edge cases). Prompt 13 is "Enter" (posted while E is current), 26 is "Exit" (T or I current), and a script door posts its own context. Resolved: **Pack 1098 door 3 message.** On confirmation, its callback presents a hero-spoken locked-door response: “It seems to be locked.” It requests no area load, consistent with the cut 1170 route (Doors; Edge cases). **[High]**
- Block-pair list and the pairing sets. Resolved: the test is decoded in World buffers and transitions (Neighbour-world test); the pairs stop drawing but never block a swap (High).
- **Interior-kind packs in E.** Resolved in part: 1429, 1430 and 1431 export only Interior-family entry points but, when streamed into E, receive no module lifecycle calls; their script callbacks are not registered or stepped. The hero's return to 1427 uses default door/world-cell behavior, not a refused leave-interior call. The 174-pack reach and 5-pack script-only counts are unchanged; excluding the three inactive leave-interior edges changes the distinct pair census from 385 to 382 and leave-interior pairs from 21 to 18 (reach recount note dated 2026-10-23). Whether the family/buffer mismatch was intended remains open.
- Resolved: **Pack 706's door callback.** The callback is looked up in the current world's teleport table, so it runs only while 706 (T) is current, and its enter call into 984 is always refused (Edge cases, Kind refusals). The door never works; 984 is entered from 956. **[High]**
- **Reach and gates.** The 174-pack reach ignores story-flag gates and whether a streamed neighbour's ground is walkable from the current world. The gates are known for a few routes: the doors of 722 and 758 need only the door prompt; 1392 is entered from 1353 by a script trigger registered when flag 353 is set and flag 104 clear, on hero contact; 1382, 1430 and 1431 need hero contact with a trigger of 1391 or 1427. 1391's trigger 1 (pending slot 165, pack 1382) is a rectangle volume with the moving test (index 1, reference corner y -229.6, height 160) whose callback needs only the hero's contact; 1391's module holds no flag test at all. 1427's eight triggers test no flag either (Area numbering, pending setter) (High). The gates of the other routes are not mapped in this spec.
- Resolved: **Activation filter.** RegisterAIPlayerActivateCallback stores the filter at world +0x30 and forwards it to block +0x30 of the embedded block and of every slot created so far; a slot created later has no filter. Module 985's filter accepts or refuses record ordinals 5 to 7 by story flag 281; module 1217's active Act 1 fight filter admits only embedded ordinal 3 of that pack's records (SPEC-0045). Other pack filters remain open (Medium-Low). **[High for the store]**
- Resolved: **Dead test in streaming.** It is the class's dead test; for townsfolk and enemies: base state 16, or AI mode word (+0xA34) 16 with the counter +0xD14 above 0 (Character activation, Dead test). Reanimating rows probably never reach it (SPEC-0038). **[High]**
- Resolved: **Cell gate of bound characters.** The region update recomputes each active character's cell flag (+0x18C) every tick from the zone query, and the streaming pass releases a character whose flag is 0 (Character activation, Cell flag). **[High]**
- **Kill-bit loss on streaming.** Confirm by a run-time check (kill an enemy in a transition, force a different transition stream, return) before treating the forgotten kills as an original bug; the code path is unambiguous. **[Medium-High]**
- Resolved: **Kill block last byte.** It is a spare byte. The set, clear and test routines accept only bit indices 0 to 255, and no other code reads or writes the byte. It is only zeroed (constructor, reset, new game) and copied (world load, store, save and load of a game), so it is always 0 unless a save file supplies another value, and then it has no effect. **[High]**
- Resolved: **AI mode 7 release.** Mode 7 is set only from the record's default AI mode word (kind-3 data +0x84) when it is 7: 392 records in 134 resources (Release and rebind). **[Medium-High]**
- Resolved: **Fade gate for an exterior driver request.** While E is loaded, the request starts the screen fade at six frames toward alpha 128 and arms a separate elapsed-time gate. The loader takes the request when that gate reaches 0.1 seconds, independent of a fade-complete signal. **[Medium]**
- Open (R-0009): the clean side's pack 909 probe figures (18 percent of vertices matched with x-major; spawn height 40.5) were not reproduced here. The cell order itself is settled (High).
- Resolved (R-0012): the door step's own code (its state, fade, pending-job and cell tests, then the facing test that picks the side of entry) adds no width or height margin; the limits are those of the plane test in the Doors paragraph (High). Test vector (synthetic): for a hero (radius 1.175, so the distance limit is 3.525), a door whose plane is 2.67 from the feet point touches; a feet point 3.6 beyond the door's end along the plane does not touch, and one 3.5 beyond does. The height range is from 1.5 below the door's base to the door's height, inclusive at the top.



