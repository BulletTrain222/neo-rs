# SPEC-0051: Map page (area pictures, area selection, icons, cursor, pulses, hero marker, return to the journal)

- Answers: proactive (UI pass: map page build guide, section 11; journal-to-map pending target correction in section 6), none requested; R-0047 (in part: the icon art source, Open question 12); R-0070 (icon and marker anchors, marker rotation, overlay root, area-name and label fonts: sections 4, 7 and 8, Open question 26); proactive (open close-out); R-0241 (answered from the code and disc data; not run in the original)
- Status: Draft
- Disc build: SCUS-97367 v1.00 (check: resource 533 tables, resources 601 to 677 each holding one texture chunk, and the boot program data words listed under Where it lives)
- Confidence: High for the map pictures, the area selection, the icon list, the cursor, the exits, the pulses, the overlay gate and the treasure table layouts; High for the button bit names and press edges (SPEC-0033 Controller map); High for the projection shape; High for the hero marker angle rule (facing minus yaw plus pi) and for the side on which the facing is negated (f.x above 0); High for the dig sites, the dig sequence and the treasure rewards (section 10), Medium for the exact tick on which a dig starts; High for the projection table's address, record count, stride, layout and values (the values are given in Layout by user decision)

## Summary

The map page shows one area: a full-screen picture of that area with icons for its active quests and held treasures, and a marker for the hero when the hero is in that area. There is one map page per area, 77 in all, each with its own picture resource. A single word, the selected map area, chooses which page opens; it is set from the hero's current area (View Map, D-pad Left in play, the L2/R2 page ring) or from a journal row (Cross). There is no world map, no reveal or unlock state, no scrolling and no zooming. Icon and marker positions are world x and z, mapped onto the picture by a per-area 2D affine projection. A cursor moves between the icons; Cross on an icon with items opens the quest journal with that icon pending, and the journal returns the cursor to it. The page has a "Treasures: n of m" line that shows on the 55 areas whose total is above zero; the loader sets each total to the number of collected-table rows naming the area (section 3). A treasure has three parts: a treasure record (its name, position, area and a held-map flag), a collected flag in the collected table, and a dig site in the world. Standing at a dig site while its map is held runs a short dig sequence that drops the site's reward (an item or Neopoints) and sets the collected flag; the map is not used up. The clean side can build the page from these tables, the pictures, the pulse constants and the rules below; the projection values, which the game builds at start-up, are given in Layout.

## Where it lives on the disc

- Map pictures: archive resource 601 + k for area k (k = 0 to 76), each a resource with a single texture chunk (SPEC-0003), drawn as a 512 x 448 picture on a 512 x 512 indexed texture. General front-end rule: a background with background index i is archive resource 577 + i - 2; the base 577 is a constant of the front-end loader, and the background index is the word at +0x0C of the sprite's asset record (SPEC-0035, asset records whose kind word is -1). Map page k's sprite uses asset record 268 + k, whose background index is 26 + k. The journal background (index 18) is resource 593.
- Map page screens: front-end screen record 31 + k is the map page of area k; its state code is 0x26C + 20 x k and its state is the map state (SPEC-0035).
- Hero map area: the 32-bit word +4 of the current world pack's type-1001 record (payload + 0x14 of the type-1001 chunk; SPEC-0006).
- Area and quest tables: the item-table file (archive resource 533, IDM table chunks). Table index 1 holds quest rows (238 rows of 36 bytes). Table index 3 holds treasure rows (48 rows of 20 bytes, of which 44 are real treasures across 24 areas; the other 4 are sentinel rows). Table index 5 holds one 8-byte row per area (77 rows, index = area id); its first word is the area name text and its second word is the area total, which is zero on disc and is recounted when the file loads (section 3). Table index 6 holds 248 rows of 8 bytes; its first word is an area id and its second word is a collected flag, zero on every row on disc. Entry 200 + i belongs to treasure row i (same area id) and holds treasure i's collected flag.
- Pulse constants (integer and float), read from the boot program's data at run time: integer scale start 4096 (virtual address 0x2DD328), step 50 (0x2DD314), upper bound 4300 (0x2DD310), lower bound 3900 (0x2DD312). Float scale start 1.0 (0x2DD32C), step 0.03 (0x2DD324), upper bound 1.5 (0x2DD320), lower bound 0.8 (0x2DD31C).
- Text colours: icon name label, word at 0x2DD318 (neutral); area name label, word at 0x2DD010 (bytes 127, 125, 81, 128).
- Treasures line numbers: front-end number slots 16 (collected count) and 17 (area total), through the number-source table at 0x2DD200 (SPEC-0035).
- Projection table (High): 77 records of 0x50 bytes (80) at the run-time address 0x382F90 to 0x38479F; record k (area k, k = 0 to 76) starts at 0x382F90 + 0x50 k. The address lies beyond the end of the file-backed image (VA 0x32B214), so the table is zero in the file. A start-up initialiser (a start-up table entry, SPEC-0026, called once with (1, 0xFFFF)) writes all 77 records. It reads no source table: every value comes from its own code (SPEC-0035 section 14.5).
- Values (user decision): because no data table in the file image holds the projection values, they are published in this spec (Layout, projection values table) as a user-approved exception to the rule against copied tables. The clean side uses them as constants and does not read this table from the boot executable.
- Icon list, cursor, selected map area, fallback map area and the pending target: runtime state. Not stored on disc.
- Treasure held-map flags (table index 3, +0x10) and collected flags (table index 6, entries 200 + i): runtime values, both zero on disc, so no map is held and no treasure is collected at the start of a game.
- Dig sites: entries of a world pack's type-40 script-group list (SPEC-0028 section 11) whose kind word is 6, one per site (47 entries in 28 world packs; section 10). Each site's reward is a kind-8 placement of the same pack, named by the site through the pack's placement id table (SPEC-0021, SPEC-0006).

## Layout

Quest row (36 bytes, table index 1), fields used by the map:

| Offset | Size | Type | Meaning |
| --- | --- | --- | --- |
| +0x00 | 4 | u32 | Kind: 0 = main story, 1 = side quest (SPEC-0049). Not read by the map. |
| +0x04 | 4 | u32 | Start progress flag id (SPEC-0049). The row is skipped when it is -1. |
| +0x08 | 4 | u32 | Done progress flag id (SPEC-0049). The row is skipped when it is -1. |
| +0x0C | 4 | u32 | Title text, copied to the icon as its label. |
| +0x10 | 4 | u32 | Objective text (journal only). |
| +0x14 | 4 | u32 | Stage. Stage 0 gives no icon. Otherwise the marker kind is stage minus 1. |
| +0x18 | 4 | f32 | World position x. |
| +0x1C | 4 | f32 | World position z. |
| +0x20 | 4 | i32 | Scene (area) id, or -1 for none. |

Treasure row (20 bytes, table index 3, 48 rows):

| Offset | Size | Type | Meaning |
| --- | --- | --- | --- |
| +0x00 | 4 | u32 | Name text of the treasure (for example "Bogshot Treasure"). Zero on the 4 sentinel rows. |
| +0x04 | 4 | f32 | World position x. |
| +0x08 | 4 | f32 | World position z. |
| +0x0C | 4 | i32 | Area id (0 to 73), or -1 on the sentinel rows. A row with area -1 is never listed as a treasure. |
| +0x10 | 4 | u32 | Held-map flag. Zero on disc. Set when the hero holds map i (section 10). |

Sentinel rows (table index 3): rows 0, 7, 15 and 47 (first word zero, area id -1). The other 44 rows are treasures, numbered 1 to 46 by row index.

Area row (table index 5, 8 bytes, index = area id): the first u32 is the area name text (shown on the map page); the second u32 is the area total. The total is zero on all 77 rows on disc, and the file loader overwrites it with the row count of the collected table for that area (section 3).

Map area names (High): a map area id m (0 to 76) is row m of table 5, and its name is the NUL-terminated text that the row's first word points to. One id space serves the type-1001 map-area word of a world pack (SPEC-0006), the RegisterTransitionMap argument (section 1) and the quest row's scene id (SPEC-0049). The 77 rows hold 74 distinct names: "Sewer Shrine" is the name of rows 26 and 28, and "Heroes Rest" of rows 34, 35 and 36. So a name does not identify an area; keep the id. Read the names at run time; do not copy them.

Collected rows (table index 6, 8 bytes): the first u32 is the area id, the second u32 is the collected flag (nonzero = collected). Entry 200 + i belongs to treasure row i. Entries 0 to 199 have the same layout and are counted by the collected count (section 3).

Icon record (runtime, 24 bytes): the icon list holds up to 32 icons, in order. Icon k starts at 24 times k from the list start.

| Offset in icon | Size | Type | Meaning |
| --- | --- | --- | --- |
| +0x00 | 4 | f32 | World x (quest row x, or treasure x). |
| +0x04 | 4 | f32 | World z (quest row z, or treasure z). |
| +0x08 | 4 | u32 | Marker kind. Quest icon: stage minus 1 (0 or 1). Treasure icon: 2. |
| +0x0C | 4 | u32 | Label text: the quest title (quest icon) or the treasure name (treasure icon). |
| +0x10 | 4 | runtime reference | Quest icon: the quest row reference (nonzero = the icon has items). Treasure icon: 0. |
| +0x14 | 4 | runtime reference | Treasure icon: the treasure record reference (nonzero = the icon has items). Quest icon: 0. |

Projection record (runtime, 80 bytes, one per area, 77 records; area k's record at 0x382F90 + 80 k). Every record is zero until start-up writes it.

| Offset | Size | Type | Role |
| --- | --- | --- | --- |
| +0x00 | 16 | 4 f32 | G0 = (a, 0, b, 1): column 0 of the transform; a and b are the world-x factors of screen x and screen y |
| +0x10 | 16 | 4 f32 | G1 = (0, 1, 0, 1): column 1 (height; never reaches the screen point) |
| +0x20 | 16 | 4 f32 | G2 = (c, 0, d, 1): column 2; c and d are the world-z factors of screen x and screen y |
| +0x30 | 16 | 4 f32 | G3 = (tx, 0, tz, 2): column 3; tx and tz are the screen offsets in pixels |
| +0x40 | 4 | f32 | yaw offset of the picture, radians (section 7) |
| +0x44 | 12 | zero | unused |

The fixed zeros, ones and twos shown are those of all 77 records written at start-up (High). The start-up write adds (0, 0, 0, 1) to each group after building it, so the fourth components are 1, 1, 1 and 2. The screen point uses only result components 0 and 2 (section 7), so the fourth components do not move it.

Projection values (High). These values are published here by the user's approval, as a limited exception to the rule against copied tables; the clean side uses them as constants. Row k is area k's record at 0x382F90 + 0x50 k. Each float is written with enough decimal digits to give back the exact 32-bit float. The 12 bytes at +0x44 to +0x4F are zero in every record. In the column names, a = G0.x, b = G0.z, c = G2.x, d = G2.z, tx = G3.x and tz = G3.z.

| Area k | G0.x | G0.y | G0.z | G0.w | G1.x | G1.y | G1.z | G1.w | G2.x | G2.y | G2.z | G2.w | G3.x | G3.y | G3.z | G3.w | Yaw (+0x40) |
| ---| ---| ---| ---| ---| ---| ---| ---| ---| ---| ---| ---| ---| ---| ---| ---| ---| --- |
| 0 | 1.04305 | 0.0 | 0.0 | 1.0 | 0.0 | 1.0 | 0.0 | 1.0 | 0.0 | 0.0 | 1.21474 | 1.0 | 192.63324 | 0.0 | 331.59692 | 2.0 | 0.0 |
| 1 | 0.73112 | 0.0 | -0.00029 | 1.0 | 0.0 | 1.0 | 0.0 | 1.0 | 0.00025 | 0.0 | 0.85362 | 1.0 | -957.2571 | 0.0 | 1710.0322 | 2.0 | 0.000345 |
| 2 | 0.73208 | 0.0 | -0.00029 | 1.0 | 0.0 | 1.0 | 0.0 | 1.0 | 0.00025 | 0.0 | 0.85412 | 1.0 | 253.54434 | 0.0 | 190.18715 | 2.0 | 0.000345 |
| 3 | 0.65231 | 0.0 | -0.00026 | 1.0 | 0.0 | 1.0 | 0.0 | 1.0 | 0.00023 | 0.0 | 0.75849 | 1.0 | 234.58755 | 0.0 | 172.76564 | 2.0 | 0.000345 |
| 4 | 0.44305 | 0.0 | 0.0 | 1.0 | 0.0 | 1.0 | 0.0 | 1.0 | 0.0 | 0.0 | 0.5143 | 1.0 | 165.83887 | 0.0 | 193.02713 | 2.0 | 0.0 |
| 5 | 0.57417 | 0.0 | 0.0 | 1.0 | 0.0 | 1.0 | 0.0 | 1.0 | 0.0 | 0.0 | 0.66975 | 1.0 | -171.63916 | 0.0 | 1615.2902 | 2.0 | 0.0 |
| 6 | 0.70419 | 0.0 | -0.0004 | 1.0 | 0.0 | 1.0 | 0.0 | 1.0 | 0.00034 | 0.0 | 0.82166 | 1.0 | -220.28339 | 0.0 | 2016.964 | 2.0 | 0.000488 |
| 7 | 0.617 | 0.0 | -0.00035 | 1.0 | 0.0 | 1.0 | 0.0 | 1.0 | 0.0003 | 0.0 | 0.72022 | 1.0 | -313.33118 | 0.0 | 1651.3778 | 2.0 | 0.000488 |
| 8 | 0.35059 | 0.0 | 0.0 | 1.0 | 0.0 | 1.0 | 0.0 | 1.0 | 0.0 | 0.0 | 0.40805 | 1.0 | 718.5642 | 0.0 | -256.51343 | 2.0 | 0.0 |
| 9 | 0.64429 | 0.0 | -0.00025 | 1.0 | 0.0 | 1.0 | 0.0 | 1.0 | 0.00022 | 0.0 | 0.73077 | 1.0 | 1505.665 | 0.0 | -597.9626 | 2.0 | 0.000345 |
| 10 | 1.18491 | 0.0 | 0.0 | 1.0 | 0.0 | 1.0 | 0.0 | 1.0 | 0.0 | 0.0 | 1.38002 | 1.0 | 283.44287 | 0.0 | 232.33742 | 2.0 | 0.0 |
| 11 | 0.45427 | 0.0 | 0.0 | 1.0 | 0.0 | 1.0 | 0.0 | 1.0 | 0.0 | 0.0 | 0.53013 | 1.0 | 215.2739 | 0.0 | 214.32294 | 2.0 | 0.0 |
| 12 | 0.9572 | 0.0 | -0.00041 | 1.0 | 0.0 | 1.0 | 0.0 | 1.0 | 0.00033 | 0.0 | 1.18978 | 1.0 | 286.25964 | 0.0 | 334.36316 | 2.0 | 0.000345 |
| 13 | 0.38711 | 0.0 | 0.0 | 1.0 | 0.0 | 1.0 | 0.0 | 1.0 | 0.0 | 0.0 | 0.45204 | 1.0 | -797.09705 | 0.0 | 430.53616 | 2.0 | 0.0 |
| 14 | 0.72885 | 0.0 | -0.0003 | 1.0 | 0.0 | 1.0 | 0.0 | 1.0 | 0.00025 | 0.0 | 0.85459 | 1.0 | -1365.1035 | 0.0 | 958.83716 | 2.0 | 0.000345 |
| 15 | 0.37604 | 0.0 | 0.0 | 1.0 | 0.0 | 1.0 | 0.0 | 1.0 | 0.0 | 0.0 | 0.43823 | 1.0 | -465.04755 | 0.0 | 101.71341 | 2.0 | 0.0 |
| 16 | 0.36839 | 0.0 | -0.00021 | 1.0 | 0.0 | 1.0 | 0.0 | 1.0 | 0.00018 | 0.0 | 0.43094 | 1.0 | -431.25006 | 0.0 | 389.7728 | 2.0 | 0.000488 |
| 17 | 0.38184 | 0.0 | -0.00015 | 1.0 | 0.0 | 1.0 | 0.0 | 1.0 | 0.00013 | 0.0 | 0.44318 | 1.0 | -737.1537 | 0.0 | 151.77046 | 2.0 | 0.000345 |
| 18 | 0.32714 | 0.0 | -0.00013 | 1.0 | 0.0 | 1.0 | 0.0 | 1.0 | 0.00011 | 0.0 | 0.38245 | 1.0 | -95.92188 | 0.0 | 521.6359 | 2.0 | 0.000345 |
| 19 | 0.38953 | 0.0 | 0.0 | 1.0 | 0.0 | 1.0 | 0.0 | 1.0 | 0.0 | 0.0 | 0.45489 | 1.0 | -387.0746 | 0.0 | 574.9614 | 2.0 | 0.0 |
| 20 | 1.30511 | 0.0 | 0.0 | 1.0 | 0.0 | 1.0 | 0.0 | 1.0 | 0.0 | 0.0 | 1.51934 | 1.0 | -131.36536 | 0.0 | -957.3627 | 2.0 | 0.0 |
| 21 | 0.22305 | 0.0 | -0.00009 | 1.0 | 0.0 | 1.0 | 0.0 | 1.0 | 0.00008 | 0.0 | 0.26017 | 1.0 | 150.11417 | 0.0 | 1.81174 | 2.0 | 0.000345 |
| 22 | 0.32771 | 0.0 | 0.0 | 1.0 | 0.0 | 1.0 | 0.0 | 1.0 | 0.0 | 0.0 | 0.37879 | 1.0 | -386.68848 | 0.0 | 581.7608 | 2.0 | 0.0 |
| 23 | -0.10139 | 0.0 | 1.18223 | 1.0 | 0.0 | 1.0 | 0.0 | 1.0 | -1.18223 | 0.0 | -0.10139 | 1.0 | 315.84497 | 0.0 | 229.77393 | 2.0 | -1.656349 |
| 24 | 0.68568 | 0.0 | -0.00048 | 1.0 | 0.0 | 1.0 | 0.0 | 1.0 | 0.00041 | 0.0 | 0.79675 | 1.0 | 308.82764 | 0.0 | 211.34525 | 2.0 | 0.000598 |
| 25 | 0.92241 | 0.0 | 0.0 | 1.0 | 0.0 | 1.0 | 0.0 | 1.0 | 0.0 | 0.0 | 1.07279 | 1.0 | 250.90172 | 0.0 | 238.49026 | 2.0 | 0.0 |
| 26 | 0.38036 | 0.0 | 0.0 | 1.0 | 0.0 | 1.0 | 0.0 | 1.0 | 0.0 | 0.0 | 0.44343 | 1.0 | 294.1 | 0.0 | 576.4526 | 2.0 | 0.0 |
| 27 | 0.44053 | 0.0 | 0.0 | 1.0 | 0.0 | 1.0 | 0.0 | 1.0 | 0.0 | 0.0 | 0.5157 | 1.0 | 120.11545 | 0.0 | 273.18677 | 2.0 | 0.0 |
| 28 | 0.56961 | 0.0 | 0.0 | 1.0 | 0.0 | 1.0 | 0.0 | 1.0 | 0.0 | 0.0 | 0.66719 | 1.0 | 248.41087 | 0.0 | 400.9008 | 2.0 | 0.0 |
| 29 | 0.33847 | 0.0 | -0.00019 | 1.0 | 0.0 | 1.0 | 0.0 | 1.0 | 0.00017 | 0.0 | 0.39741 | 1.0 | 244.25713 | 0.0 | 311.15204 | 2.0 | 0.000488 |
| 30 | 0.35651 | 0.0 | 0.0 | 1.0 | 0.0 | 1.0 | 0.0 | 1.0 | 0.0 | 0.0 | 0.41585 | 1.0 | 169.96088 | 0.0 | 225.18387 | 2.0 | 0.0 |
| 31 | 0.34931 | 0.0 | 0.0 | 1.0 | 0.0 | 1.0 | 0.0 | 1.0 | 0.0 | 0.0 | 0.4053 | 1.0 | 331.83493 | 0.0 | 226.06445 | 2.0 | 0.0 |
| 32 | 0.35237 | 0.0 | 0.0 | 1.0 | 0.0 | 1.0 | 0.0 | 1.0 | 0.0 | 0.0 | 0.40996 | 1.0 | 258.24988 | 0.0 | 137.00436 | 2.0 | 0.0 |
| 33 | 0.34904 | 0.0 | 0.0 | 1.0 | 0.0 | 1.0 | 0.0 | 1.0 | 0.0 | 0.0 | 0.40672 | 1.0 | 258.58884 | 0.0 | -105.53748 | 2.0 | 0.0 |
| 34 | 0.65156 | 0.0 | 0.0 | 1.0 | 0.0 | 1.0 | 0.0 | 1.0 | 0.0 | 0.0 | 0.76094 | 1.0 | 293.32556 | 0.0 | 269.6815 | 2.0 | 0.0 |
| 35 | 0.38399 | 0.0 | 0.0 | 1.0 | 0.0 | 1.0 | 0.0 | 1.0 | 0.0 | 0.0 | 0.44641 | 1.0 | 265.47998 | 0.0 | 201.6826 | 2.0 | 0.0 |
| 36 | 0.87737 | 0.0 | 0.0 | 1.0 | 0.0 | 1.0 | 0.0 | 1.0 | 0.0 | 0.0 | 1.02212 | 1.0 | 257.01053 | 0.0 | 239.55505 | 2.0 | 0.0 |
| 37 | 0.28616 | 0.0 | 0.0 | 1.0 | 0.0 | 1.0 | 0.0 | 1.0 | 0.0 | 0.0 | 0.33129 | 1.0 | 194.7616 | 0.0 | 276.60623 | 2.0 | 0.0 |
| 38 | 0.41515 | 0.0 | 0.0 | 1.0 | 0.0 | 1.0 | 0.0 | 1.0 | 0.0 | 0.0 | 0.48363 | 1.0 | 635.60187 | 0.0 | -6.03427 | 2.0 | 0.0 |
| 39 | 0.45944 | 0.0 | -0.00047 | 1.0 | 0.0 | 1.0 | 0.0 | 1.0 | 0.00022 | 0.0 | 0.95824 | 1.0 | 671.56085 | 0.0 | -236.12561 | 2.0 | 0.000488 |
| 40 | 0.63658 | 0.0 | 0.0 | 1.0 | 0.0 | 1.0 | 0.0 | 1.0 | 0.0 | 0.0 | 0.7439 | 1.0 | -111.51642 | 0.0 | 28.27921 | 2.0 | 0.0 |
| 41 | 0.60065 | 0.0 | -0.00024 | 1.0 | 0.0 | 1.0 | 0.0 | 1.0 | 0.00021 | 0.0 | 0.69987 | 1.0 | 357.0287 | 0.0 | 194.13304 | 2.0 | 0.000345 |
| 42 | -0.71084 | 0.0 | -0.0553 | 1.0 | 0.0 | 1.0 | 0.0 | 1.0 | 0.0553 | 0.0 | -0.71085 | 1.0 | -150.3432 | 0.0 | 229.58147 | 2.0 | 3.063948 |
| 43 | 0.35617 | 0.0 | 0.0 | 1.0 | 0.0 | 1.0 | 0.0 | 1.0 | 0.0 | 0.0 | 0.41576 | 1.0 | 328.5676 | 0.0 | 203.492 | 2.0 | 0.0 |
| 44 | 0.25196 | 0.0 | -0.00014 | 1.0 | 0.0 | 1.0 | 0.0 | 1.0 | 0.00012 | 0.0 | 0.29312 | 1.0 | 265.36176 | 0.0 | 400.99683 | 2.0 | 0.000488 |
| 45 | 0.67641 | 0.0 | 0.0 | 1.0 | 0.0 | 1.0 | 0.0 | 1.0 | 0.0 | 0.0 | 0.78877 | 1.0 | 5.35699 | 0.0 | 632.25604 | 2.0 | 0.0 |
| 46 | 1.09091 | 0.0 | 0.0 | 1.0 | 0.0 | 1.0 | 0.0 | 1.0 | 0.0 | 0.0 | 1.27516 | 1.0 | 861.72815 | 0.0 | -1127.2908 | 2.0 | 0.0 |
| 47 | 0.44157 | 0.0 | 0.0 | 1.0 | 0.0 | 1.0 | 0.0 | 1.0 | 0.0 | 0.0 | 0.51565 | 1.0 | 203.73636 | 0.0 | -87.88193 | 2.0 | 0.0 |
| 48 | 1.14567 | 0.0 | -0.00046 | 1.0 | 0.0 | 1.0 | 0.0 | 1.0 | 0.0004 | 0.0 | 1.33554 | 1.0 | 263.67035 | 0.0 | 222.63002 | 2.0 | 0.000345 |
| 49 | 1.14191 | 0.0 | 0.0 | 1.0 | 0.0 | 1.0 | 0.0 | 1.0 | 0.0 | 0.0 | 1.33554 | 1.0 | 263.21747 | 0.0 | 222.5679 | 2.0 | 0.0 |
| 50 | 2.46535 | 0.0 | 0.0 | 1.0 | 0.0 | 1.0 | 0.0 | 1.0 | 0.0 | 0.0 | 2.95745 | 1.0 | 238.80197 | 0.0 | 709.2766 | 2.0 | 0.0 |
| 51 | 2.53535 | 0.0 | 0.0 | 1.0 | 0.0 | 1.0 | 0.0 | 1.0 | 0.0 | 0.0 | 2.91765 | 1.0 | 240.48485 | 0.0 | 682.02356 | 2.0 | 0.0 |
| 52 | 2.55844 | 0.0 | -0.00099 | 1.0 | 0.0 | 1.0 | 0.0 | 1.0 | 0.00088 | 0.0 | 2.86813 | 1.0 | 252.73619 | 0.0 | 669.1474 | 2.0 | 0.000345 |
| 53 | 0.57457 | 0.0 | -0.00023 | 1.0 | 0.0 | 1.0 | 0.0 | 1.0 | 0.0002 | 0.0 | 0.66891 | 1.0 | 208.85158 | 0.0 | -131.62158 | 2.0 | 0.000345 |
| 54 | 0.64883 | 0.0 | -0.00026 | 1.0 | 0.0 | 1.0 | 0.0 | 1.0 | 0.00022 | 0.0 | 0.7538 | 1.0 | 533.3732 | 0.0 | 658.18097 | 2.0 | 0.000345 |
| 55 | 0.60477 | 0.0 | -0.00034 | 1.0 | 0.0 | 1.0 | 0.0 | 1.0 | 0.0003 | 0.0 | 0.70501 | 1.0 | -349.73358 | 0.0 | 142.95248 | 2.0 | 0.000488 |
| 56 | 0.78823 | 0.0 | 0.0 | 1.0 | 0.0 | 1.0 | 0.0 | 1.0 | 0.0 | 0.0 | 0.91798 | 1.0 | 255.75012 | 0.0 | 220.86972 | 2.0 | 0.0 |
| 57 | 0.42387 | 0.0 | 0.0 | 1.0 | 0.0 | 1.0 | 0.0 | 1.0 | 0.0 | 0.0 | 0.49466 | 1.0 | 255.50443 | 0.0 | 254.9581 | 2.0 | 0.0 |
| 58 | 1.72955 | 0.0 | -0.00086 | 1.0 | 0.0 | 1.0 | 0.0 | 1.0 | 0.0006 | 0.0 | 2.4998 | 1.0 | 278.14407 | 0.0 | 217.48892 | 2.0 | 0.000345 |
| 59 | 1.0397 | 0.0 | 0.0 | 1.0 | 0.0 | 1.0 | 0.0 | 1.0 | 0.0 | 0.0 | 1.21344 | 1.0 | 266.86786 | 0.0 | 208.42456 | 2.0 | 0.0 |
| 60 | 0.43539 | 0.0 | 0.0 | 1.0 | 0.0 | 1.0 | 0.0 | 1.0 | 0.0 | 0.0 | 0.50876 | 1.0 | 271.8101 | 0.0 | 203.07576 | 2.0 | 0.0 |
| 61 | 0.5098 | 0.0 | -0.00029 | 1.0 | 0.0 | 1.0 | 0.0 | 1.0 | 0.00025 | 0.0 | 0.59404 | 1.0 | -1092.0647 | 0.0 | 730.4917 | 2.0 | 0.000488 |
| 62 | 0.47727 | 0.0 | -0.00018 | 1.0 | 0.0 | 1.0 | 0.0 | 1.0 | 0.00016 | 0.0 | 0.525 | 1.0 | -1598.9265 | 0.0 | 166.53557 | 2.0 | 0.000345 |
| 63 | 0.29053 | 0.0 | 0.0 | 1.0 | 0.0 | 1.0 | 0.0 | 1.0 | 0.0 | 0.0 | 0.33881 | 1.0 | -694.8701 | 0.0 | 410.01135 | 2.0 | 0.0 |
| 64 | 0.32949 | 0.0 | 0.0 | 1.0 | 0.0 | 1.0 | 0.0 | 1.0 | 0.0 | 0.0 | 0.38438 | 1.0 | 602.2047 | 0.0 | 221.44446 | 2.0 | 0.0 |
| 65 | 0.64631 | 0.0 | -0.00026 | 1.0 | 0.0 | 1.0 | 0.0 | 1.0 | 0.00022 | 0.0 | 0.75147 | 1.0 | 907.9389 | 0.0 | 908.72943 | 2.0 | 0.000345 |
| 66 | 0.27838 | 0.0 | 0.0 | 1.0 | 0.0 | 1.0 | 0.0 | 1.0 | 0.0 | 0.0 | 0.32583 | 1.0 | 224.31708 | 0.0 | -324.46503 | 2.0 | 0.0 |
| 67 | 0.28334 | 0.0 | 0.0 | 1.0 | 0.0 | 1.0 | 0.0 | 1.0 | 0.0 | 0.0 | 0.33004 | 1.0 | 293.4334 | 0.0 | -501.8175 | 2.0 | 0.0 |
| 68 | 1.69042 | 0.0 | 0.0 | 1.0 | 0.0 | 1.0 | 0.0 | 1.0 | 0.0 | 0.0 | 1.9805 | 1.0 | 1378.4886 | 0.0 | -4444.102 | 2.0 | 0.0 |
| 69 | 0.29846 | 0.0 | 0.0 | 1.0 | 0.0 | 1.0 | 0.0 | 1.0 | 0.0 | 0.0 | 0.34723 | 1.0 | 461.61646 | 0.0 | -457.36328 | 2.0 | 0.0 |
| 70 | 0.67599 | 0.0 | 0.0 | 1.0 | 0.0 | 1.0 | 0.0 | 1.0 | 0.0 | 0.0 | 0.78785 | 1.0 | -585.1721 | 0.0 | 454.28915 | 2.0 | 0.0 |
| 71 | 0.33507 | 0.0 | 0.0 | 1.0 | 0.0 | 1.0 | 0.0 | 1.0 | 0.0 | 0.0 | 0.39156 | 1.0 | 354.892 | 0.0 | 458.7405 | 2.0 | 0.0 |
| 72 | 0.36866 | 0.0 | -0.00014 | 1.0 | 0.0 | 1.0 | 0.0 | 1.0 | 0.00013 | 0.0 | 0.4041 | 1.0 | -238.93832 | 0.0 | 894.6209 | 2.0 | 0.000345 |
| 73 | 0.42619 | 0.0 | 0.0 | 1.0 | 0.0 | 1.0 | 0.0 | 1.0 | 0.0 | 0.0 | 0.50336 | 1.0 | 539.8159 | 0.0 | -145.75848 | 2.0 | 0.0 |
| 74 | 0.57686 | 0.0 | 0.0 | 1.0 | 0.0 | 1.0 | 0.0 | 1.0 | 0.0 | 0.0 | 0.67002 | 1.0 | 273.63312 | 0.0 | 1180.992 | 2.0 | 0.0 |
| 75 | 1.5625 | 0.0 | 0.0 | 1.0 | 0.0 | 1.0 | 0.0 | 1.0 | 0.0 | 0.0 | 1.8125 | 1.0 | 261.0 | 0.0 | 802.0 | 2.0 | 0.0 |
| 76 | 0.84251 | 0.0 | 0.0 | 1.0 | 0.0 | 1.0 | 0.0 | 1.0 | 0.0 | 0.0 | 0.98111 | 1.0 | 254.48405 | 0.0 | 227.11746 | 2.0 | 0.0 |

Value summary (High): the yaw offset is 0 in 48 records, a small angle (0.000345, 0.000488 or 0.000598 radians) in 27, and large in two: area 23 (-1.656349) and area 42 (3.063948). Those two areas have rotated pictures: their factors a, b, c and d form a rotation by about 95 degrees (area 23) and about 176 degrees (area 42), the angle of (a, b) being the negative of the yaw offset to within 0.00001 radians.

The pending return target is a runtime reference to a quest row or to a treasure record (section 6).

## Behavior

### 1. Map pictures and the selected map area

- There is one map page per area, area ids 0 to 76. Map page k draws the full-canvas picture of resource 601 + k and, on top of it, the icons, the marker and the overlay below. Backgrounds are loaded when the screen changes, into one of two alternating buffers of up to 0x41400 bytes (High).
- There is no world-level map, no per-area reveal or unlock flag, no scrolling and no zoom. The page shows one area's picture and the icons on it (High for the page; Medium for the absence of any other reveal state).
- Hero map area: the current world pack's map area id (type-1001 record word +4). When that word is -1, the fallback map area is used instead: the last value passed to the script call RegisterTransitionMap (call 263) in this session. The fallback starts at -1 during map-manager initialization, is set by RegisterTransitionMap, and is not reset by world loads; a valid current-pack area always takes precedence (High; see Open question 18).
- The selected map area is one word. It is set:
  - to the hero map area by View Map on the pause page (Cross), by D-pad Left in play, by L2 on the journal and by R2 on the item screen;
  - to the row's scene id (quest row +0x20) when that is 0 or more, else to the treasure's area (+0x0C) when that is 0 or more, by Cross on a journal row.
- The map page opened is record 31 + the selected area, and its displayed area is the selected area. If the selected area is negative the map does not open: View Map and D-pad Left stay where they are, R2 on the item screen goes to the journal instead, and L2 on the journal goes to the item screen instead (High).
- Disc survey (type-1001 chunks of the 192 world packs): 124 packs carry an area id, 67 carry -1, and pack 532 has no such chunk. Areas 36, 39, 50, 51 and 52 belong to no pack; 50, 51 and 52 (Meridell Castle 3rd to 5th Floor) are reached only through the RegisterTransitionMap calls of pack 1122. Five level modules call RegisterTransitionMap from triggers, with areas 5, 7, 21, 37, 43, 48, 49, 50, 51, 52, 54, 64 and 73. Area 36 is reached by neither route and no quest row or treasure names it, so its page is never shown (Medium-High). Area 39 is shown only from the journal row whose scene is 39.
- During Act 2 the hero areas are: pack 1174 area 48, 1300 area 57, 1324 area 58, 1327 and 1299 area 59, 1335 and 1344 area 60; pack 1098 has none and uses the last registered transition area (48 or 49), and the Vorsh shop (1433) has none and uses 57. No treasure belongs to areas 48, 49 or 57 to 60, so Act 2 pages show only quest icons (rows of SPEC-0049, Journal rows during Act 2). Marker rows 73 to 79 all sit at world (1, 1) of area 59. Row 88's icon is on the page of area 60 although the hero stands in area 59 (pack 1299) while it is listed (High).
- Quirk: the map page of area 17 has, besides its picture, a second fixed text label with the area's name in its own node tree; the other 76 pages have only the picture.

### 2. Building the icon list

- Opening the map stores the icon art, the displayed area and the list head, clears the icon list and sets the cursor to icon 0.
- The list is built once, on the first pass of the map page, not every tick. Quest icons are added first, in quest table order, then treasure icons, in treasure table order (High).
- Quest icons: for each quest row whose start flag is set, whose done flag is clear, whose stage is nonzero and whose scene id equals the displayed area, add one icon at the row's world x and z. A row whose start or done id is -1 is skipped. The marker kind is stage minus 1: stage 1 gives kind 0, stage 2 gives kind 1, and stage 0 gives no icon (High).
- Treasure icons: for each treasure row i (rows 1 to 46), add one icon at the treasure's world x and z when all of the following hold: the held-map flag is nonzero; the row's area id equals the displayed area and is not -1; the collected flag (entry 200 + i) is zero. The icon kind is 2 (High).
- Journal rows (for reference): the journal lists, with no area filter, every accepted quest row (start flag set, done flag clear, neither id -1) in table order, then every treasure record that is held, has area not -1 and is uncollected, in table order. The stage and scene tests belong to the map icons only.
- Icon capacity is 32. An icon that would be the 33rd is not added, and an overflow report is written (Medium; Open question 11). Disc data never reaches the cap: the largest possible count is 16 (area 9: 15 marked quest rows and 1 treasure).
- After the build, a pending target moves the cursor (section 6), and the pending target is cleared.

### 3. Area counts and the Treasures line

- Total for an area: the second word of the area's row in table index 5, as it stands after the file loader has run. The stored value on disc is 0, and the loader sets it as follows: it zeroes the second word of every area row, then walks all 248 rows of the collected table (entries 0 to 199 and the 48 treasure entries) and adds 1 to the area row named by each row's area id, skipping ids that are negative or 77 or more (High). So the total of an area is the number of collected-table rows that name it, whatever their flags say. It is not counted from the treasure rows alone (High).
- Collected for an area: the number of rows in the collected table (all 248 rows, entries 0 to 199 included) whose area id equals the area and whose second word is nonzero (Medium-High for the counting rule; High for the disc values, all zero).
- The overlay has a line "Treasures: n of m", where n is the collected count (front-end number slot 16) and m is the area total (slot 17). The line is shown only while the area total of the selected area is nonzero.
- On disc data the totals after loading are nonzero for 55 of the 77 areas (221 rows in all: 177 from entries 0 to 199 and 44 from the treasure entries; 27 rows hold area id -1 and count for no area), so the line does appear on the map page of those 55 areas (High for the counting and the gate; Medium-High that nothing later overwrites it, Open question 19). The totals count entries 0 to 199 as well as the treasure entries, so the total can exceed the number of treasures of the area. Check values from the disc: area 3 totals 9, area 43 totals 19, area 44 totals 15 and area 15 totals 0 (no line). The rewrite recomputes every total from the user's own collected table at load.

### 4. Each tick on the map page

In this order:

1. Pulses: the float pulse and the integer pulse of section 5 each take one step. The steps are part of the icon pass, so they are skipped on the tick on which Select leaves the page; they still step while the selected area is negative (High).
2. Hero marker (section 7), only when the hero map area equals the displayed area.
3. Cursor: if the list is empty, the cursor is set to 0. Otherwise an Up (0x1000) or Right (0x2000) press moves to the next icon (wrapping from the last to the first); else a Down (0x4000) or Left (0x8000) press moves to the previous icon (wrapping from the first to the last). If both kinds are pressed in one tick, the next move wins. These read the menu newly-pressed2 word, which includes stick-derived directions, so one push gives one move and holding gives no repeat (High on the rule and the bit names; SPEC-0033 Controller map). Cursor moves play no cue.
4. Icons: each icon is drawn at its projected point (section 7) with the art for its kind: kind 0 primary-quest art (26 x 32), kind 1 secondary-quest art (26 x 29), kind 2 treasure-spot art (31 x 31). The cursor icon is drawn with the integer pulse as its scale on both axes; the others with scale 1.0 (4096). The cursor icon's label (quest title or treasure name) is drawn 14 pixels above its point in the colour at 0x2DD318. Icon anchor (High): the projected point is the centre of the icon on both axes. The icon's drawn width is its art width times its scale, and its top-left corner is the point minus half the drawn width and half the drawn height, so the pulse scale grows the cursor icon about its centre and the icon node's own position plays no part (the three art nodes sit at (0, 0)). A 26 x 32 icon at (105, 77) covers x 92 to 118 and y 61 to 93. Label (High): front-end font 2 (line height 14, the same font as the journal's objective line), letter spacing 0, no wrapping, centred on the point's x (the text width is halved toward zero and subtracted; the start x is raised to 16 if it falls below 16, SPEC-0022 Line alignment), baseline y = the point's y - 14, coordinates cut toward zero. A label of 100 pixels width at x 105 starts at x 55; a label of 60 pixels at x 30 starts at 16.
5. Exits, tested in this order:
   - Select press (0x0100 on the menu newly-pressed2 word): back to play (state code 0xB4), no cue. The exit fires on the press edge, on the tick Select goes down; holding Select does not keep firing it (High).
   - Cross (0x0040) press while the current icon has items, or R2 press: the pending target becomes the current icon's quest reference, or else its treasure reference (none if neither); the front-end select sound (cue 0x98) plays; the journal opens (state code 0x1B8). R2 with no icon or an icon without items opens the journal with no pending target.
   - L2 press: go to the item screen for the current tab and the active hero (SPEC-0035 section 14.4), no cue.
   - Otherwise the page stays: the area-name label is set to the area's name (area row first word) in the colour at 0x2DD010, and the overlay group (section 8) is drawn, and the shared front-end card message box is offered as on other pages (SPEC-0035).
6. Cross on an icon without items, or with an empty list, does nothing. The area name is drawn every tick regardless of the cursor.
7. Triangle is handled by the shared front-end back rule (SPEC-0035 section 9). The map record's back rule is 9, so opening a map page cuts the screen stack back to play and pushes the page; Triangle therefore always returns to play with cue 0x99, from any route, also when the map was opened from the pause page (High).

L2 and R2 are bits 0x0001 and 0x0002 of the menu newly-pressed word (buttons only, without stick-derived directions; SPEC-0033). The cursor directions, Select and Cross read the newly-pressed2 word (stick directions included). Button bit names follow SPEC-0033, Controller map: 0x0001 L2, 0x0002 R2, 0x0040 Cross, 0x0100 Select, 0x1000 Up, 0x2000 Right, 0x4000 Down, 0x8000 Left (High). The menu words are the combined snapshot, which in the shipped game equals pad slot 0.

### 5. Pulses

Integer pulse (scale of the cursor icon, 4.12 fixed point; High on the constants):
- Start at 4096. Each tick, add the step (starts at +50).
- If the value would exceed 4300, set it to 4300 and make the step -50.
- If the value would fall below 3900, set it to 3900 and make the step +50.

Float pulse (brightness of the hero marker; High on the constants):
- Start at 1.0. Each tick, add 0.03.
- If the value exceeds 1.5, set it to 1.5 and negate the step.
- If the value falls below 0.8, set it to 0.8 and negate the step.

Test vectors:
- Integer, value 4290, step +50: next value is 4300 (since 4340 exceeds 4300); step becomes -50. Next tick: 4250, step stays -50.
- Float, value 1.45, step +0.03: next value 1.48. Then 1.51 is above 1.5, so the value becomes 1.5 and the step becomes -0.03.

### 6. Return from the journal

- Cross or R2 on the map makes the current icon's reference pending before the journal opens (one setter: the quest row when the icon has one, else the treasure). Cross on a journal row opens the map, but the journal sets the row target and then the treasure target in one step, and the second setter clears the first. So only a treasure row hands the map a pending target; Cross on a quest row leaves nothing pending and the map cursor starts on icon 0 (quirk, reproduce; High).
- When the map list is built with a pending reference, the cursor is set to the index (0-based, its own position in the list) of the first icon whose quest reference, or treasure reference, is the same reference (compared by identity, not by id). There is no off-by-one. If no icon matches, or none is pending, the cursor stays at icon 0 (High).

### 7. Projection and hero marker

- Projection: for a world point (x, y, z), take (x, y, z, 1) and multiply by the area's record with the four groups as columns (result = G0 x + G1 y + G2 z + G3 w). Screen x is result component 0 and screen y is result component 2. With the start-up shape this is:
  - screen x = a x + c z + tx
  - screen y = b x + d z + tz
  The height y only reaches component 1, so it does not affect the screen point (High).
- Icons are projected from (x, 0, z, 1) using their world x and z. The hero marker uses the hero's world position with a fourth component of 1. Icons and the marker use the same record. Screen positions are rounded half away from zero to whole pixels.
- The yaw offset at +0x40 is the rotation of the picture in radians: 0 or a small angle under 0.001 in 75 areas, and -1.656349 (area 23) or 3.063948 (area 42) on the two rotated pictures (Layout, value summary).
- Hero marker: drawn only when the hero map area equals the displayed area. Its art is the locator sprite (32 x 32). Its colour RGB is the base colour (128, 128, 128) times the float pulse, each channel rounded half away from zero, so the marker brightens and dims between 102 and 192 while its size stays the same (High). The base colour (128, 128, 128, 128) is set at start-up.
- Marker angle = hero facing minus the area's yaw offset, plus pi (radians) (High for the sign and the pi term). The hero facing angle theta is taken from the hero's forward vector f (the flattened unit facing; heading 0 is +Z, 90 degrees is +X): theta = arccos(f.z), negated when f.x is strictly above 0 (High; from the code; not run in the original). For a horizontal unit f this is atan2(-f.x, f.z). The marker angle is A = theta - yaw + pi. The arrow art points up at A = 0 and turns clockwise on the screen as A grows, so before the 6/7 squash it points along the screen direction (sin A, -cos A). Subtracting the yaw turns the marker by the same rotation as the picture, whose (a, b) angle is the negative of the yaw offset (Layout, value summary). Do not add the yaw. In 75 of the 77 areas world +X is screen right and world +Z is screen down; in area 23 +X is down and +Z is left; in area 42 +X is left and +Z is up; no record is a mirror. The rule therefore makes the arrow follow the facing as it appears in the picture.
  Vectors (yaw 0, an ordinary area): facing +Z: theta 0, A = pi, the arrow points down (world +Z); facing +X: theta = -pi/2, A = pi/2, right; facing -X: theta = pi/2, A = 3 pi/2, left; facing -Z: theta = pi, A = 2 pi (the same as 0), up. Area 42 (yaw 3.063948), facing +Z: A = 0.077645, up (+Z is up in that picture). Area 23 (yaw -1.656349), facing +Z: A = 4.797942, left (+Z is left in that picture).
  Rewrite check: a rewrite that negates the facing angle when f.x is above 0 and then adds pi, so that +X gives pi/2, matches this rule. If its arrow still looks reversed, the cause lies elsewhere: the rotation sense of its sprite draw (the original turns clockwise for positive A on a y-down screen), the art's orientation (up at A = 0), a mirrored x mapping from world to screen, or a facing vector that is the reverse of the motion record's forward vector.
- Marker quad and rotation (High): the locator art is drawn as a quad whose corners are (-w/2, -h/2), (+w/2, -h/2), (-w/2, +h/2) and (+w/2, +h/2) around the marker's projected point (the point is the centre; w = h = 32; the node's scale is not applied). With A the marker angle, C = cos A and S = sin A, each corner offset (x, y) is turned into the screen offset ((6/7) (C x - S y), S x + C y), then added to the point and rounded as for icons. This is a rotation by A (clockwise on the screen, y downward) followed by a horizontal squash to 6/7 (0.857143) of the width; the icons have no such squash. The texture covers the whole 32 x 32 art, corner to corner. Order within a tick: the marker is drawn first, then the icons.

Test vectors (synthetic records; not game values):
- Record decoding: a record whose bytes at +0x00 are 00 00 00 40 00 00 00 00 00 00 00 3F 00 00 00 00 has G0 = (2.0, 0, 0.5, 0), so a = 2.0 and b = 0.5. With G2 = (0.25, 0, 3.0, 0) and G3 = (100.0, 0, 50.0, 1.0), the world point (x 10, z 20) gives screen x = 2.0 x 10 + 0.25 x 20 + 100 = 125 and screen y = 0.5 x 10 + 3.0 x 20 + 50 = 115.
- Record address: area 5's record starts at 0x382F90 + 5 x 0x50 = 0x383120, and its yaw offset is the f32 at 0x383160.
- Any height: with the same record, the hero at (10, 50, 20) gives the same point as (10, 0, 20).
- Marker angle: hero facing 0.2 radians and yaw offset 0.5 radians give 0.2 - 0.5 + pi = 2.8416 radians.
- Marker corners: angle pi (hero facing +Z, no yaw), point (200, 100): C = -1, S = 0, so the corner offset (-16, -16) becomes (+13.71, +16) and the corner (+16, -16) becomes (-13.71, +16); the quad spans x 186.29 to 213.71 and y 84 to 116. Angle pi/2: C = 0, S = 1, so (-16, -16) becomes (+13.71, -16) and (+16, -16) becomes (+13.71, +16).
- Marker colour: float pulse 0.8 -> (102, 102, 102); 1.0 -> (128, 128, 128); 1.5 -> (192, 192, 192).

Test vector (game values, from the projection values table):
- Area 75 (a = 1.5625, b = 0.0, c = 0.0, d = 1.8125, tx = 261.0, tz = 802.0): the world point (x -100, z -400) gives screen x = 1.5625 x -100 + 0.0 x -400 + 261 = 104.75 and screen y = 0.0 x -100 + 1.8125 x -400 + 802 = 77.0, rounded to (105, 77).

### 8. Overlay group

Drawn every tick the map stays:
- "Triangle Exit", always.
- "Cross View Quest", only while the current icon has items and no overlay is busy.
- The area name label: a text node in front-end font 0 (line height 21), letter spacing 0, at (256, 40) with centre alignment. Before each draw the node's width is replaced by the measured width W of the current text, so the text starts at x = trunc(256 - W / 2) and its baseline is y 40. The page writes the area name and the colour at 0x2DD010 into the node's label binding on every tick it stays (High).
- L2 and R2 tab hints.
- The "Treasures: n of m" line, only while the area total is nonzero (section 3; on disc data it shows for 55 of the 77 areas).

### 9. Page ring with L2 and R2

The item screen, the map and the journal form a ring. R2 goes Item screen -> Map -> Journal -> Item screen; L2 goes the other way.

| From | L2 | R2 |
| --- | --- | --- |
| Item screen | journal (cue 0x98) | map of the hero map area (cue 0x99); journal if the area is negative |
| Map | item screen (no cue) | journal, current icon pending (select sound, cue 0x98) |
| Journal | map of the hero map area (cue 0x99); item screen if the area is negative | item screen (cue 0x98) |

The map and the journal are the same for both heroes; only the marker follows the active hero (High).

### 10. Treasure pieces and the dig

**Treasure pieces (High).** Treasure i (rows 1 to 46) has three pieces: the treasure row (name text, map x and z, area id and held-map flag; table index 3), the collected flag (collected table entry 200 + i), and a dig site in a world pack. The held flag is set when the hero receives map i (item 175 with map number i; SPEC-0021).

**Dig-site record (High).** A dig site is an entry of the world pack's type-40 script-group list (SPEC-0028 section 11) whose kind word (+0x00) is 6 and whose key (+0x08) is 200 + i. The entry is 0x44 bytes. Each list below is a count followed by a payload-relative offset to that many s32 values.

| Offset | Field | On disc (all 47 kind-6 entries) |
| --- | --- | --- |
| +0x00 | Kind | 6 |
| +0x08 | Key: 200 + treasure number | 201 to 247, and one 165 |
| +0x0C / +0x10 | Locator list (type-11 entry records; SPEC-0030 locator table) | 1 locator |
| +0x14 / +0x18 | Script trigger index list | 1 index |
| +0x1C / +0x20 | Animated-object list (SPEC-0028 section 7.1) | 3 objects: [0] undug, [1] reveal, [2] dug |
| +0x24 / +0x28 | Camera track list | 2 tracks |
| +0x2C / +0x30 | Breakable list | empty, except the dead site of pack 1091 (2) |
| +0x34 / +0x38 | Placed-effect list (type-39 records) | 1 record, the site marker (effect 399; 976 in pack 1091) |
| +0x3C / +0x40 | Reward list: [0] placement id (placement id table, SPEC-0006), [1] character id | 2 values; [1] is -1 on every site |

Let P be the locator's position (+0x30) and D its facing (+0x40; a unit vector, horizontal on disc).

**Dig volume (High for the data).** The trigger index names a sphere of the pack's type-11 sphere list with flags 0x0008 (script trigger fire; SPEC-0028 section 13). On all 45 live sites the sphere has radius 4.0 and its centre is P + 1.31 D, at the height of P. The treasure row's map position lies within 6.7 units (horizontally) of that centre. Nothing else is tested: there is no distance test against the treasure row and no area test.

**Site set-up** (at world load, and again when the script call ReActivateScriptGroup names the group; High):
1. Register the dig callback on the trigger index, with the group as its owner.
2. If treasure i is not collected: object [0] is shown at the start of its animation (time 0). The marker effect is on exactly when map i is held.
3. If treasure i is collected: object [2] at the start of its animation and object [1] at the end of its animation (the dug look). The marker effect is off.
4. Objects not named in a branch keep their load state. Set-up needs at least 3 objects; all 47 entries have 3. The collected and held flags are read only when the reward list is not empty (all 47 entries have one); a site with an empty reward list would always show the undug look with the marker off (High; not on disc).

Setting a map held (a script grant, a pickup, a purchase or a save load) also turns that site's marker on at once when the site is in the current world, or else in the second loaded world. The site is found by its key 200 + i (High).

**When a dig starts (High for the tests; Medium-High for the exact tick).** There is no button, prompt or hero-state test. The callback receives the trigger block (SPEC-0030) and goes on only when all of these hold:
- The trigger state is 2, a repeat fire: the hero was already inside the sphere on an earlier tick. The sphere fires on every tick the hero's body overlaps it, the first tick gives state 1 and the second gives state 2, so the dig starts on the second tick of contact.
- The block's object value is nonzero. The main hero's own trigger-hit path gives a nonzero value (SPEC-0028); the per-tick pass gives 0, so it never starts a dig. So a dig needs both kinds of fire. Only the selected hero runs the trigger and contact step; the other hero is parked out of the world and is not updated (SPEC-0053, The parked hero), and there is no AI follower. So only the selected hero can start a dig (High).
- The group exists, has at least one object, and object [0] is active.
- Treasure i is not collected.
- Map i is held.

No level module registers a trigger on, deactivates or re-activates a dig group.

**Dig sequence (High for the order and the constants; times in seconds, 60 ticks per second):**
1. Place the active hero at P - 2.2 D, facing D.
2. Hero equipped state off (SetHeroEquippedState 0); HUD reset.
3. Play camera track [0] with no wait, end mode 0, speed 1.0.
4. Marker effect off.
5. Start effect 4036 of shared effect group 3 at P + 2.9 D, with no rotation.
6. Camera shake with the values (2.0, 0.3, 25.0, 0.5, 0.8, 1.5708, 0.2), in the argument order of StartCameraShake (SPEC-0030 row 41).
7. Play object [0] once at speed 1.0 and wait for it to end. Deactivate object [0]. Activate objects [2] and [1] at time 0. Play object [1] once at speed 1.0 and wait for it to end.
8. Wait 1 tick. Start effect 790 of shared effect group 3 at P + 2.9 D, turned to face D (the up axis is a run-time constant, not on disc).
9. Wait 1.0 s (60 ticks). Play camera track [1] as in step 3.
10. Hero one-shot animation 186 (PlayHeroAnim 186, no wait; it yields one tick). Id 186 is inside both heroes' action tables (ids 0 to 192, SPEC-0005, Hero animation data), so it plays a clip in every family that has one; a family whose entry has no clip takes its clip 0 (Lookup miss). Hero entries start no cue, so the animation plays no sound (High).
11. Reward: when reward [0] is 0 or more, run the drop action (SPEC-0021) on that placement id at P + 2.2 D with direction -D. When reward [1] is 0 or more, run the character activation route at the same point (no site on disc uses it).
12. Wait 2.0 s (120 ticks). Return to the normal camera with no preset and release the hero (hero state 1).
13. Set collected entry 200 + i to 1. Wait 1 tick.

The collected flag is written only at step 13, at the very end. The animation lengths are the same on every live site (High, measured on all 45 live sites and the sentinel site): object [0] has 61 keys, so it runs 4.0 s (240 ticks); object [1] has 23 keys, so it runs 1.4667 s (88 ticks); object [2] has a single key. Both camera tracks are static shots (no keys), so neither ends by itself and the camera returns only at step 12. Step 7 therefore lasts about 328 ticks, and the whole sequence about 511 ticks (8.5 s), give or take two ticks for yields (Medium for the tick count; High for the lengths).

**Reward (High).** A dig gives a reward. The reward placement is a kind-8 placement of the same pack with flag 4 (never spawned at region load; SPEC-0021), placed at the site. It goes through the drop action, not through the item add. Key 0 or more drops that item. Key -1 pays the placement's drop group money roll when the group is valid and the roll succeeds, otherwise its fixed count, as coins (100, 25, 10, 5, 1). Because a direction is given, each coin starts 0.4 s after the one before. The luck gate and the region-load vetoes do not apply. The reward items are dropped, not placed, so they expire: 11.2 s, or 15.2 s for a charm such as a Mote, unless picked up. Important items are range-collected after 0.4 L and given at expiry (SPEC-0021).

Reward survey (the 44 real treasures; High):
- 25 item rewards: Supernova Mote 10, Kauvara's Potion 4, Fyora's Potion 3, Starry Negg 3, Red Negg 2, Four-Leaf Clover 2, Codex of Protection 1. The Codex of Protection is the reward of treasure 8 (Meriload Mine) and has no other source on disc.
- 19 Neopoint rewards: 16 with drop group -1 and a fixed count from 653 to 4,118 NP; 3 (treasures 3, 37 and 41) with drop group 3 and a fixed fallback: a 30 % chance of a whole amount from 400 to 1,000 NP, otherwise the fixed count. The fixed counts total 33,499 NP.
- The two dead sites hold a Red Negg (key 247) and an empty placement (key 165: group -1, count 0).

**Sounds and effects (High).** The dig plays no sound cue of its own. It starts effects 4036 and 790 (both in shared effect group 3), a camera shake and hero animation 186. Any sound comes from that animation's own cue words (SPEC-0005, SPEC-0031) and from the later coin and item pickups. The site marker is effect 399.

**Without the map, or away from a site (High).** There is no dig action. Away from a site nothing can be dug. At a site whose map is not held, the sphere fires but the callback stops at the held test: no message, sound, animation or state change. The undug object is shown and the marker is off. A collected site shows the dug objects and never digs again.

**The map after a dig (High).** The map is not used up: item 175 is not kept as an inventory item, and the held flag stays set. Nothing clears it except the new-game reset. The map page icon and the journal row disappear because the treasure is now collected (section 2). Map sellers skip held maps, so a dug map is never offered again.

**Survey (High).** There are 47 kind-6 entries in 28 world packs. 45 are live sites for the 44 real treasures; treasure 16 has a site in each of two variant packs (956 and 970). Two sites can never be dug:
- Key 247 in pack 1230: treasure row 47 is a sentinel and no map 47 exists, so no map value makes it diggable. Its Red Negg is unreachable.
- Key 165 in pack 1091: the held test reads row 165 for this key, which is outside the 48 rows, so the site is never held. Its marker uses effect 976.

**Other script groups (High).** Kinds 1 and 2 of the same group list are chests (keys 0 to 199; SPEC-0048). They also end with the drop action and set their own collected entry, but they need no map. Only kind 6 is a treasure.

**Test vectors.**
- Treasure 11 (pack 880, key 211), map 11 held, entry 211 = 0. Locator P = (1856.320, 158.869, -1041.617), D = (0.5629, 0, -0.8265). Sphere centre (1857.057, 158.869, -1042.700), radius 4.0. The dig places the hero at (1855.082, 158.869, -1039.799) and starts effects 4036 and 790 at (1857.952, 158.869, -1044.014). Reward placement id 6 (ordinal 6, key 58): one Kauvara's Potion dropped at (1857.558, 158.869, -1043.435) with direction (-0.5629, 0, 0.8265). Afterwards entry 211 = 1, the marker is off and map 11 is still held.
- Same site with map 11 not held: the hero stands in the sphere for 10 s and nothing happens; the marker stays off.
- Treasure 9 (pack 886, key 209): placement key -1, group -1, count 4,118. It drops 46 coins (41 of 100, one 10, one 5, three 1) from (858.008, 69.568, -990.502). Coin k (0-based) has delay 0.4 k s, so the last starts at 18.0 s.
- Treasure 37 (pack 1350, key 237): group 3, count 2,256. A money roll below 30 pays a whole amount drawn from 400 to 1,000. Otherwise it pays 2,256 = 22 x 100 + 2 x 25 + 5 + 1 (26 coins).
- Treasure 8 (pack 816): the Codex of Protection. It is important, so it is collected from up to 25 units after 4.48 s, and given at expiry if left.
- Re-entering pack 880 after the dig: object [2] at its start, object [1] at its end, marker off. Entering the sphere does nothing.
- Pack 1230, key 247: no map value from 1 to 46 makes it diggable.

### 11. Build guide pointer

The element tables, draw order, exits and a worked example for the map page are in SPEC-0035, section G.16 (Screen build guide). Added here: the page draws no frame, fade or transition; the overlay group is drawn only on passes on which the page stays; on the pass that Select leaves the page nothing but the picture is drawn (and the four music overrides are cleared); on the pass that Cross (icon with items), R2 or L2 leaves it, the marker and icons are drawn but the area name and overlay are not. Default if unresolved (rewrite choice): draw the previous pass's frame until the next page appears.

## Edge cases and error handling

- Selected map area negative (hero pack area -1 and no transition map set): the map does not open; see section 1 for where each route goes instead.
- A map opened from the journal for an area other than the hero map area: no marker is drawn.
- Area id outside 0 to 76: total 0, collected 0.
- Icon list at capacity (32): the next add is refused (never reached by disc data).
- Empty icon list: cursor 0, no cursor icon, Cross does nothing, R2 opens the journal with no pending target, L2 goes to the item screen.
- Icon with both reference fields zero: Cross does nothing and the map stays open.
- Pending reference with no matching icon: the cursor stays at icon 0.
- Journal Cross on a quest row whose area has a matching quest icon: no pending target, so the cursor stays at icon 0; Cross on a treasure row: the cursor starts on that treasure's icon.
- Quest rows with a nonzero stage but scene -1 (39 rows on disc) never get an icon.
- A treasure row with area -1 is never listed, whether or not its map is held.
- Area 36's page exists but no route on disc data reaches it.
- A projection record never written (all zero) puts every icon and the marker at screen (0, 0) (Medium).
- A debug-only switch word, zero on disc, makes the treasure and quest flag tests ignore their flags. It is not set in play (Medium).
- Dead dig sites: key 247 (pack 1230) and key 165 (pack 1091) are never held, so they never dig (section 10).
- Dig without the map: the sphere fires but nothing happens; the map is never used up by a dig (section 10).
- A large Neopoint reward is many coins in the 50-slot world-item pool (SPEC-0021); 4,118 NP is 46 coins. Every coin of the split takes its slot when the drop runs; a delayed coin holds its slot while it waits (state 5). The split runs in order 100, 25, 10, 5, 1, and each coin spawns only while a slot is free. So with f free slots, only the first min(f, coin count) coins spawn, and the value of the rest is lost for good: there is no retry and no wallet credit. The treasure stays collected (Medium-High; composed from the SPEC-0021 pool and split rules).

## Test vectors

- Map pictures: area 0 (Werelupe Sinkholes) -> resource 601; area 17 (Brightvale Coast) -> resource 618; area 76 (Altador) -> resource 677. Journal background (index 18) -> 577 + 18 - 2 = resource 593.
- Map area names (disc check values; read table 5 to check a reader): 5 "Werelupe Burrows Upper", 7 "Werelupe Burrows Lower", 43 "Ruined Caves", 49 "Meridell Castle 2nd Floor", 54 "Oubliette". Rows 26 and 28 are both "Sewer Shrine"; rows 34, 35 and 36 are all "Heroes Rest".
- Map page state code: area 47 -> 0x26C + 20 x 47 = 0x600 (record 78).
- Selected area: pack 678 (area word 8) -> Farm Road page (area 8). A pack with -1 after a trigger called RegisterTransitionMap(50) -> area 50. A pack with -1 and no transition map ever set -> View Map does nothing.
- Quest row with start flag 107, done flag 108, stage 1 and scene 9. With flag 107 set, flag 108 clear and displayed area 9: one quest icon of kind 0. Setting flag 108: no icon. With stage 2 and the same flags: one icon of kind 1. With stage 0: no icon.
- Order: an area with one active quest row Q and one held treasure T gives icons [Q, T].
- Treasure record 13 held, area 3, collected flag (entry 213) zero: displayed area 3 gives one treasure icon. With entry 213 set to 1: no icon.
- Treasure record 1 held, area 63, collected flag zero: displayed area 63 gives one icon; displayed area 3 gives none.
- A held treasure record with area -1: never an icon.
- Synthetic collected rows: entry 203 (area 3, second word 1), entry 204 (area 3, second word 0), entry 1 (area 4, second word 1). Collected count for area 3 is 1; for area 4 it is 1.
- Treasures line, disc data after loading: area 3 has 9 collected-table rows, so with collected 1 the line reads "Treasures: 1 of 9"; with nothing collected, "Treasures: 0 of 9". Area 15 has no rows: total 0, no line. Synthetic collected table with rows naming areas 3, 3, 4 and -1: area 3 total 2, area 4 total 1, every other total 0; the -1 row counts for nothing.
- Area id 77: total 0 and collected 0.
- Dig coins and a full pool: treasure 9 (4,118 NP, 46 coins: 41 x 100, 10, 5, 1, 1, 1) with 50 free slots gives all 46 coins. With 42 free slots it gives 41 x 100 and the 10 coin (4,110 NP); 8 NP is lost. With 0 free slots no coin spawns, and the treasure is still marked collected.
- Dig timing, any live site, dig starts on tick 0: object [0] is deactivated on about tick 240, object [1] ends on about tick 328, effect 790 starts on about tick 329, camera track [1] and the hero animation begin about 60 ticks later (about tick 390), the camera returns and the hero is released on about tick 510, and the collected entry is set on about tick 511.
- Dig start, a hero walks into a live site's sphere (map held, treasure not collected): contact on tick 100 gives trigger state 1 and no dig; contact on tick 101 gives state 2 with object value 1, so the dig starts on tick 101 and the timings above count from it (object [0] deactivated about tick 341, collected entry set about tick 612).
- Hero animation 186 (from the disc action tables): Tor main clip 117 and sub-part clip 0, with no face clip (face clip 0 by the miss rule); Roberta main clip 47, sub-part 37 and face 30. No cue in any of them.
- Icons in cursor order A, B, C with cursor on A: Down wraps to C. Right from C wraps to A.
- Icons A, B, C; cursor on B; Up and Down both pressed in one tick: the next move wins, cursor on C.
- Icons A, B, C; cursor on A; analog pad, left stick Y byte goes from 128 to 20 on tick n and stays at 20: the cursor moves to B on tick n only (one newly-pressed2 Up edge), with no repeat while held.
- Select down on tick n (up on tick n-1): the map exits to play on tick n.
- Icons A (quest, has items) and B (treasure, has items), cursor on B, Cross: journal opens with B's treasure pending; when the map is rebuilt with that pending target, the cursor is 1.
- Empty icon list: Cross does nothing; R2 opens the journal with nothing pending; L2 goes to the item screen.
- Dig: map 13 held and treasure 13 not collected. The dig runs once; afterwards entry 213 is 1, the site shows its dug objects, and entering the sphere again does nothing. Map 13 stays held. Further dig vectors are in section 10.
- Pulse, projection and marker vectors as in sections 5 and 7 (the projection vectors use synthetic records).

## Open questions

Numbers of removed questions are not reused.

1. Resolved: values published in the spec by user decision; the table's address, layout and all 77 records are in Where it lives and Layout, and the clean side uses them as constants.
5. Resolved: quest icons first (quest table order), then treasure icons (treasure table order).
6. Resolved: with an empty list the cursor is 0, Cross does nothing, R2 opens the journal with nothing pending and L2 goes to the item screen.
7. Resolved: direction bit names are fixed by SPEC-0033, Controller map: 0x1000 Up, 0x2000 Right, 0x4000 Down, 0x8000 Left (High).
8. Resolved: the float pulse scales the hero marker's colour (brightness 102 to 192); the integer pulse scales the cursor icon.
9. Resolved: yaw sign and angle unit. The yaw offset is subtracted: marker angle = hero facing - yaw offset + pi, and the facing is an angle in radians (section 7). This matches the pictures: on the two rotated pictures (areas 23 and 42) the angle of (a, b) is the negative of the yaw offset, so subtracting the yaw turns the marker by the same rotation as the projection. **[High]**
11. Resolved as a default: the overflow report for a 33rd icon is a debug report with no gameplay effect, and disc data never reaches the cap. Default (rewrite choice): drop the 33rd icon silently (needs a runtime check; same as SPEC-0035 item 74).
12. Resolved (R-0047, part 3; High): the map's art comes from four nodes of the front-end database (SPEC-0035, sections C, E and J; the key is looked up in section C and gives a section J node). (a) The three icon arts are the first three children of the group node with key 210 (node 1176): kind 0 primary quest, 26 x 32, sprite record 711 (sheet asset 263); kind 1 secondary quest, 26 x 29, sprite record 712 (asset 264); kind 2 treasure spot, 31 x 31, sprite record 713 (asset 265). The map opens by storing these three sprite records in order and draws an icon by its kind number, so the child order is the kind order. (b) The hero locator is the sprite node with key 26 (node 938): 32 x 32, sprite record 505 (sheet asset 134). It is the sprite whose colour bytes the pulse rewrites and which the marker draw rotates; the earlier statement that key 26 holds the icon art was wrong. (c) The overlay group is the node with key 214 (node 1180), drawn every tick of the map page. Its children in order: the Triangle hint sprite (32 x 32 at (359, 400), sprite record 714, asset 33; always drawn) with its caption node (the string "Exit", at (393, 423)); a group gated by predicate slot 26 with mode 2 that holds the Cross hint sprite (32 x 32 at (65, 400), sprite record 715, asset 30) and its caption (the string "View Quest", at (100, 423)); the "Treasures: n of m" group at (322, 60), gated by predicate slot 73 (three value and text nodes: the word "Treasures:", the count, "of" and the total); a group holding the area name text node (key 215, centred at (256, 40), whose text pointer and colour the page overwrites each tick); and the L2 and R2 tab hint group at (0, 181) with two 42 x 60 sprites, the left one at (0, 0) (sprite record 716, asset 42) and the right one at (469, 0) (sprite record 717, asset 43). The Triangle and Cross sprites use the same button-glyph sheet assets (33 and 30) as the hints of the other front-end pages. Test vector: a kind-2 icon at the projected point (105, 77) draws sprite record 713 (31 x 31) centred there; the same page with the cursor icon of kind 0 draws record 711 at scale 4096 times the pulse.
14. Resolved: the cursor is set to the matched icon's own 0-based index; there is no off-by-one.
16. Answered: the front-end sound slots (select 0x98, back 0x99, focus moves 0x9A) are filled once at start-up; see SPEC-0035 Behavior 1b. The map's own cursor moves play no cue.
17. Resolved: the D-pad Left map shortcut runs only while both quick panels are closed, so a press that opens a panel turns it off in the same tick (SPEC-0035 section 6). **[High]**
18. Resolved from static data: the fallback map area is initialized to -1 during map-manager initialization and RegisterTransitionMap replaces it; world loads do not clear it. When the current pack map area is -1, the retained fallback supplies the page area; if both values are negative, the map does not open. No black-box confirmation is claimed. High.
19. Resolved (High): the writer is the file loader. After the item-table file's area table and collected table are bound, it zeroes the area totals and adds one per collected-table row to the row's area (section 3). The only later writers of that table's second words are the collected-flag setters, which touch the collected table, never the area table. So the "Treasures" line appears on 55 areas. Earlier wording that the total is zero and the line never appears was wrong.
20. Resolved: the sphere fires on every tick the selected hero's body overlaps it, with object value 1 (the hero's trigger-hit path; the shot pass is the only other hero path, and it fires with object 0 and needs a shot). The first overlapping tick writes trigger state 1 and the callback stops at its state test; the next tick writes state 2 and the dig starts. A hero that is already inside the sphere when the map becomes held, or when the world loads, digs on the next tick. **[Medium-High]**
21. Resolved: only the selected hero can start a dig. The other hero is parked out of the world and runs no trigger or contact step (section 10). **[High]**
22. Resolved: hero animation 186 is in range of both heroes' action tables. It plays a clip and starts no cue (dig step 10). Effect 790 uses the boot-initialized unit up vector (0, 1, 0, 0) when the dig facing direction is converted to its orientation (SPEC-0007). **[High]**
23. Resolved: the three site objects have 61, 23 and 1 keys (4.0 s, 1.4667 s and a single pose) on every site, so dig step 7 lasts about 240 + 88 ticks. Both camera tracks are static shots that never end by themselves, so the camera cannot return early before track [1] (High; see the note after the dig sequence).
24. Resolved: coins that find no free world-item slot are lost; the split is spawned in full at the drop, largest coins first (Edge cases). **[Medium-High]**
25. Resolved (Medium-High): the quirk cannot show. Loading held flags mirrors each flag onto the site object of an already loaded world with no collected test, but a load applies the slot and then reloads both resident packs (SPEC-0026, Load game), and every pack load runs the site set-up of section 10 again, which tests the collected flag: a collected site gets the dug look and its marker off. Test vector: load a save with map 11 held and treasure 11 collected while pack 880 is resident: after the reload, marker off, dug objects shown.
26. Resolved (High): icon and marker anchor = centre of the art on the projected point; the marker rotates about its centre with a 6/7 horizontal squash (section 7); the cursor label is front-end font 2, 14 pixels above the point, centred (section 4); the overlay root is the front-end node with key 214, drawn each tick with draw offset (0, 0), and the area name is font 0, centred at (256, 40) (section 8).
