# SPEC-0035: Front-end menus and UI database

- Answers: proactive (none requested); R-0019 (points 1 (open question 136), 2 and 3 answered here; point 4, the w words of vectors A and B, answered: A has w = 1 and B has w = 0); R-0018 (point 1 settled in open question 137; points 2 to 4 answered in SPEC-0023 open question 74; point 5, the Square caption font, answered in SPEC-0023 section 30: shared font 1); R-0031 (points 1 to 6 answered; points 2 and 3 in Open question 138); R-0071 (book reader pages: section 6.1 rules 4, 5 and 8 to 14); R-0094; R-0130 (Quest-items tab cursor, offset, arrows and sounds: section 14.1); proactive (starting equipment: equipped-item strip and a fresh game, section 14.1); proactive (UI pass: screen build guide for the front end and main menu; item screen, journal, map page and book reader, G.11 to G.18); R-0136; R-0144 (no start point is a boss fight); R-0155 (coin model placement, scale, spin, light and depth, section 15.2); proactive (open close-out)
- Status: Draft
- Disc build: SCUS-97367 v1.00 (same fingerprint as SPEC-0001). The clean side should check the front-end resource (number 532) by its chunk chain and the type-38 header name before use.
- Confidence: High for the header, section counts, record sizes, tree links, the screen-stack rules, the save slot count (four; see Open question 9), the picker rows and the picker cursor rule (High), the load starting cursor (Medium-High), the save and load prompt records 18 and 17 (High; the save prompt is opened by a direct screen set at a save point, not by a return code), the section I item targets (High), the save screen flow and the pending-flag rule (High; High that a zero card state word means "saved or cancel confirmed"), the cue index rule (Medium-High; High on five confirmed cues), the option block arithmetic, the toggle rule, the item screen's tab wrap and the grid size (4 rows of 5). Medium-High for the item screen's cell and use rules, Medium for the charm and quest-item lists and the table B identities below. High for the second input word (section 4), the L2/R2 page ring, the pause item routing, the in-play map and journal shortcuts, the hero index that picks the item-screen pages, and the journal page (section 14.3, including the row-part positions and fonts). High for the UI canvas size (always 512 x 448; section 10). Medium-High for the draw-before-handler order and its one-pass lag (Behavior 1b). High for one front-end pass, the generic menu step, the handler return codes, the front-end sound slots, the back rule, the default focus, the list-page moves, the state return survey and the page graph (Behavior 1b, 2, 9, 9d and 13.2). The map page is specified in SPEC-0051. Medium for screen names, dispatch meaning and the start-point cues. Menu bit values and physical names: High (SPEC-0033 Controller map). Medium for the roles of Start and Select as menu openers. High for the option row mapping (the focus index picks the invert word; Open question 6, resolved). Node-kind draw dispatch: High for the dispatch by kind and for the draw condition modes (all confirmed); text node draw (kind 2) and the Section G binding fields: High (Medium-High for the double scaling of letter spacing, Medium for colour-track sampling); Medium for the condition of kind 1 and for kind 7 (partly described); High for the kind 4 focused flag. The condition mode byte is at +0x2C, not +0x0C (see Open question 43). Mode 1 of the draw rule draws only under the focused item (High; Open question 75, resolved). Node search and key mask rules (section 9b): High for the child-first order and the mode tests, Medium for kind 2 and the kind 1 data check. Low for the fields marked unknown below. The shop screen body and a separate equipment screen body are not known (Open question 1). Menu overlay cell rule: Medium-High. Price caption (layout, colours, writers, callers and clears): High. Map projection table (section 14.5): High for its record count, record size, zero-filled start and write-at-start-up; Medium for the marker transform and the yaw offset; Low for the index range; the address, layout and read rule are in SPEC-0051, and the values are not in this spec (Open question 49). Marker pulse constants: High (values), Medium-Low (their roles). Menu cursor step (section 4.1; Down moves to the next kind-4 item, Up to the previous, with wrap): High for the directions, Medium-High for the kind-4 rule. Menu driver, focus search, item position and page open (section 17): labelled there; their conflicts with Behavior 1, Behavior 4 and Behavior 12 are Open questions 60 to 72. The driver caps and the rectangle and quad sizes are not yet confirmed against the disc bytes. Byte tween, cursor rectangle (debug colour editor) and nine-slice panel (dialogue box frame), and sections M and N: High (section 17, Layout). Book reader (section 6.1): High for the cursor and index rules, the Cross exit, the play gate, the Square take (a rising edge), the book-station flag rule, the page timer endpoints and step, and the page layout values; High for the writers of the pending index and the cursor, the prompt, its timing and the page driver's call rate; Medium for which actors besides the hero set the pending index. Full-screen pictures (Behavior 1a): High for the picture records, the resource rule, the picture of each screen record and the buffer sizes; Medium for the map picture rule; High for the lead-in page mechanism of records 24 to 28 (Medium that they were cut); High that the credits picture is shown (Options, Credits; the finale gate) and that the lead-in, coming-soon and controls pictures are never shown (static survey); High that the logo and legal pictures are shown at boot and after every card-state visit. Controller check, logo page, attract and title input: High. Option set, apply timing, gains at start-up, the override-clear list and front-end sound volume: High; widescreen unreachable: Medium-High.

## Summary

The front end (title, options, save and load, start-point choice, in-game pause menu) is described by one archive resource that holds a self-contained UI database. The database has a header with a name and version, a string table, a tree of about 1,366 UI nodes (containers, text, sprites, clip groups and seven page roots), sprite records with frame counts, label bindings that give a text node a neutral or highlighted colour, and a list of source asset paths. Screens are pages of this tree. Navigation keeps a stack of up to 31 pushed screens above the root. All drawing is on a 512 x 448 UI canvas. Most screens also show one full-screen background picture, loaded from its own archive resource (577 to 677) when the screen is entered (Behavior 1a).

The screen logic is a set of front-end states (table B below) that read a 16-bit logical button word, change a 32-byte option block, drive the save and load pickers, and start a new game or a start point. The same states also run the in-game item screen (three tabs: items, charms and quest items), the quest journal, the map (section 14) and the book reader (section 6.1). The map draws the hero marker through a per-area projection table (section 14.5) that is written in memory at start-up and is not stored on the disc. The clean side can build the menus from this layout by reading the resource at runtime, and it can implement the states from the rules in Behavior. It needs no code from the original.

## Where it lives on the disc

- The front end is archive resource 532 of GAMEFILE.DAT. Resource 532 is 696,320 bytes (340 sectors of 2,048 bytes).
- Resource lookup: the boot program's data holds the archive's resource table at virtual address 0x2D7618. It has 1,434 entries of 8 bytes each: u32 first sector (relative to GAMEFILE.DAT), u32 sector count. Resource 532 is entry 532 (first sector 206,495, count 340).
- Resource 532 is a chunk chain (IDM-style chunks: 16-byte chunk header, payload after it). Its top-level chunks, in order:

| Chunk | Start in resource | Size (bytes) | Extra | Content |
| --- | --- | --- | --- | --- |
| type 31 | 0x000000 | 48 | 0 | Header: name "FrontendData" |
| type 38 | 0x000030 | 156,300 | 0 | Front-end UI database (payload 156,284 bytes, starting at +16) |
| type 4 | 0x0262BC | 539,140 | 52 | Texture sheet for the front end (layout not described here) |

- The loader walks the chain: the type 31 chunk is skipped, the type 38 payload is passed to the front-end setup, and the type 4 chunk goes to the normal texture upload path. Setup and the per-frame flow are in the Behavior section.
- All offsets in the next sections are relative to the start of the type-38 payload unless marked.
- Full-screen pictures are separate archive resources 577 to 677 (101 resources), not part of resource 532. Each holds one texture chunk with one 512 x 512 8-bit indexed texture and one 16 x 16 32-bit CLUT. Resource 594 (game over) is 512 x 128. Resources 586 to 588 are stored as 512 x 512, but their image fills 512 x 440. Resources 590 and 592 are byte-identical. Which screen shows which picture is in Behavior 1a. Confidence: High.
- The map projection table (section 14.5) is not stored as data on the disc. It is empty memory until a start-up hook writes it. Its run-time address, record layout and read rule are given in SPEC-0051 (Where it lives and Layout), which owns it (Open question 49).
- The front end's dispatch tables are boot-program data, not disc data. Table A has 74 node predicates (slots 0 to 73, indexed by a node's handler index) and starts at virtual address 0x2DD058. Table B has 31 screen states (slots 0 to 30; slot 31 is zero) and starts at 0x2DD180, directly after table A. Table A and table B together are 105 consecutive code pointers (74 + 31) from 0x2DD058. Table C starts at 0x2DD200 and is data, not code pointers: it is the variable table of the value-text nodes (Section K). Each slot is a 32-bit code address in the boot program, which the clean side does not use; the handler behaviour is given in words in the table of Behavior 2 and in section 9c.
- Cue ids: a sound cue id is the index of the cue's name in the boot program's cue-name table, which starts at data address 0x2DD8F0 and has 223 entries. Five checks agree with how the front end uses the cues: 0x94 is pause, 0x95 is unpause, 0x97 is text box close, 0xA2 is item use fail and 0xA9 is save complete. Cue 0xA8 is named for save-point activation. By the same rule 0x98 is the confirm (advance) button sound, 0x99 the back button sound, 0x9A the direction-pad press, 0xA1 the cloak use and 0xA3 the Mote activation, which agrees with every front-end use of them. Confidence: High for the index rule and for these names. Other specs may cite cue names by this rule. Read the names at runtime if they are needed; do not copy the table.

## Layout

### Header (payload +0x00 to +0x9F)

| Offset | Size | Type | Meaning | Value on disc |
| --- | --- | --- | --- | --- |
| +0x00 | 16 | bytes | Name, NUL padded | "Idol Minds FEF" |
| +0x10 | u16 | u16 | Version major | 1 |
| +0x12 | u16 | u16 | Version minor | 5 (version 1.05) |
| +0x14 | u32 | count | Records in section D | 108 |
| +0x18 | u32 | count | Records in section E (sprites) | 798 |
| +0x1C | u32 | count | Records in section F | 0 |
| +0x20 | u32 | count | Records in section G (label bindings) | 165 |
| +0x24 | u32 | count | Records in section H (pages) | 7 |
| +0x28 | u32 | count | Records in section I | 30 |
| +0x2C | u32 | count | Records in section J (UI nodes) | 1,366 |
| +0x30 | u32 | count | Records in section K | 52 |
| +0x34 | u32 | count | Records in section L | 0 |
| +0x38 | u32 | count | Strings in section A (string index) | 55 |
| +0x3C | u32 | count | Unknown; not the count of section N (Open question 13) | 585 |
| +0x40 | u32 | count | Records in section O (asset paths) | 345 |
| +0x44 | u32 | count | Fonts in the tail (font table) | 3 |
| +0x48 | u32 | count | Records in section M (112-byte records) | 14 |
| +0x4C | u32 | count | Records in section N (animation slots) | 20 |
| +0x50 | u32 | count | Records in section C (key pairs) | 94 |
| +0x54 to +0x90 | 16 x u32 | offset | Start offset of each section, in file order A, T, C, D, E, F, G, H, I, J, K, L, M, N, O, tail | see the section table |
| +0x94, +0x98, +0x9C | 3 x u32 | pointer | Filled in at load time with the addresses of tables A, B and C (A and B are handler tables; C is data). Zero on disc. | 0 |

The version check requires major 1 and minor 5. Any other version is refused, and the front end does not start.

Each section's record count is the header word for that section; the start words give the layout. Section size = next start minus this start.

### Section table

| Section | Start | Size | Record size | Records | Confirmed |
| --- | --- | --- | --- | --- | --- |
| A string index | 0x0A0 | 220 | 4 | 55 | size and count match |
| T text | 0x17C | 588 | - | 55 strings | all 55 resolve |
| C key pairs | 0x3C8 | 752 | 8 | 94 | size and count match |
| D entries | 0x6B8 | 2,160 | 20 | 108 | size and count match |
| E sprites | 0xF28 | 22,344 | 28 | 798 | size and count match |
| F (empty) | 0x6670 | 0 | - | 0 | - |
| G label bindings | 0x6670 | 2,640 | 16 | 165 | size and count match |
| H pages | 0x70C0 | 84 | 12 | 7 | size and count match |
| I pairs | 0x7114 | 240 | 8 | 30 | size and count match |
| J UI nodes | 0x7204 | 71,032 | 52 | 1,366 | size and count match |
| K records | 0x1877C | 1,040 | 20 | 52 | size and count match |
| L (no-op) | 0x18B8C | 4 | - | 0 | - |
| M animation tracks | 0x18B90 | 1,568 | 112 | 14 | size and count match |
| N animation slots | 0x191B0 | 640 | 32 | 20 | size and count (+0x4C) match |
| O asset paths | 0x19430 | 46,920 | 136 | 345 | size and count match |
| tail (fonts) | 0x24B78 | 5,892 | 272 per font descriptor, then 18 per glyph record | 3 fonts, 282 glyph records | size matches (3 x 272 + 282 x 18) |

Section start offsets are payload-relative.

### Load-time relocation

When the block is loaded at a base address, the loader adds that base to some stored offsets, and only those:
- The 16 section start fields at +0x54 to +0x90 always get the base added, with no test.
- In each section J node record (52 bytes), the five words at record offsets +0x00, +0x04, +0x08, +0x0C and +0x10 get the base added only when they are not zero.
- In each section M record (112 bytes), the four words at record offsets +0x58, +0x5C, +0x60 and +0x64 get the base added only when they are not zero.
- In each section K record (20 bytes), the word at record offset +0x00 gets the base added. No other word of the record is relocated. All 52 of these words are nonzero on disc.
- The handler pointer fields at +0x94, +0x98 and +0x9C are not in this list; they are filled at load time by the front end (Layout, header).

Section J relocation uses its header count at payload +0x2C (1,366 records on resource 532); section M uses its header count at +0x48 (14 records). Their section extents match 1,366 × 52 = 71,032 bytes and 14 × 112 = 1,568 bytes. Section K has its separate 52-record loop. Confidence: High for the fields, loop counts and record strides.

### Section A: string index and T: text

- A holds 55 u32 offsets. Each is an offset from the start of T. Every entry resolves to a NUL-terminated string.
- T holds those 55 strings. They are the UI labels (menu items, option names, prompts). The full list is not copied into this spec. Section 6 names the screens that use them.

### Section C: key-to-node map (94 x 8)

- Each record is two u32 values: a key and a J-node index. Keys lie in 1 to 223 (92 distinct keys). J-node indices lie in 42 to 1197, and each one is a node of section J (High for layout).
- At start-up the records are copied, in order, into a 512-entry table of 32-bit values. The table starts filled with -1, so a key that no record lists gives -1 (Medium-High). When a key occurs more than once, the last record wins. Key 0xD1 occurs three times, with J-node indices 1194, 970 and 186, so its effective value is 186 (High).
- Front-end routines look up a node by a constant key, with dozens of call sites. The node's record in section J is then used for drawing or for a sub-object. A key that is not in the table gives -1, and the caller treats the node as absent (Medium; the caller paths were checked on samples).
- The targets are 52 sprite nodes (type 1), 35 group nodes (type 5), 3 text nodes (type 2) and 4 nodes of type 6. Only 3 targets carry a label binding (Section G).
- The symbolic names behind the keys are not in the data, and the front-end code uses the keys only as numbers (Open questions).

### Section D: screen records (108 x 20)

Each record describes one front-end screen. Record N is at section D offset N x 20. A state code (below) is such an offset, so code N selects record N / 20.

| Offset | Size | Meaning | Value on disc |
| --- | --- | --- | --- |
| +0x00 | u32 | Payload offset of the screen's UI node (a J node). The node's first word points back to this record (108 of 108 on disc). | Differs for each record |
| +0x04 | s16 | Back and stack rule (High). -3, -2 and -1 are the stack codes of section 9; 0 or more names a record (section 9, parent-index rule). The Triangle back rule reads it (section 9, Back rule). Per record on disc: -2 (stack reset when entered; Triangle off) on records 0, 1, 9, 10, 19, 23, 24 to 28 and 30; -3 (no stack change; Triangle off) on records 3 to 8 and 18; -1 (pushed; Triangle on) on records 2, 11 to 17, 20, 21 and 29; 9 (play) on record 22 and the 77 map records 31 to 107. | -3, -2, -1 or 9 |
| +0x06 | s16 | Left neighbour record: used when a Left focus move finds no item on the page. The walk follows the neighbour's own +0x06 until it reaches a record whose node predicate passes (High). All values on disc are -1 or -3, so the walk never runs in the shipped game. | -1 or -3 |
| +0x08 | s16 | Right neighbour record: as +0x06, for a Right focus move (High). Unused on disc. | -1 or -3 |
| +0x0A | u16 | Screen state: the table B index (0 to 30) of the state that runs this screen. 0xFFFF means no state. | 0 to 30 on 101 records; 0xFFFF on 7 records |
| +0x0C | u16 | Run-time focus index: the index of the focused item among the page's kind-4 items (section 9d). Not stored on disc. | 0 |
| +0x0E | u16 | Run-time pass counter: the number of passes the screen has run. Not stored on disc. It is set to 0 when the screen is entered and grows by 1 per staying pass, up to 9,999 (Behavior 1b). | 0 on disc |
| +0x10 | 4 | Zero on disc. | 0 |

**State codes.** A state code is a byte offset from the start of section D, so code N selects record N / 20. Every code the front end returns is a multiple of 20. The codes below are the ones the front end uses. The state column is the table B index that runs the record, which is the record's +0x0A value.

| Code | Record | State | What the record is |
| --- | --- | --- | --- |
| 0x000 | 0 | 0 | Card state (the memory-card check). Set directly at start-up; reached on quit, on game over without a pending save, and at the end of the finale credits (Behavior 2, state 0). |
| 0x014 | 1 | 1 | Attract (node 10). Shows the legal screen; queues the opening movie and the title movie; leads to the title (record 10). |
| 0x028 | 2 | 7 | Pause page (node 12). |
| 0x0B4 | 9 | 11 | In-game play (page node 322). Its node tree is the in-play HUD (SPEC-0023): the HUD's art, layout and colours are this record's subtree in resource 532. |
| 0x0C8 | 10 | 12 | Title (node 757). |
| 0x0DC, 0x0F0, 0x104 | 11, 12, 13 | none | Options, Controls and Audio pages (pages 774, 794, 824). Entered by the section I targets of the title, pause and Options items, not by a return code. |
| 0x118 | 14 | 18 | Credits (node 852). Entered from the Options page's Credits item, and by the in-play story-bit gate (section 6). |
| 0x12C | 15 | none | Start-point chooser container (node 854; section 5). No state. Never entered (state return survey, Behavior 2). |
| 0x154 | 17 | 21 | Load prompt (node 911; section 7.2). |
| 0x168 | 18 | 22 | Save prompt (node 915; section 7.3). Not entered by a return code: the world's save-point step sets the current screen to this record directly when the player presses Square at a save point (section 7.3). No handler returns 0x168. |
| 0x190 | 20 | 24 | Book reader (node 930; section 6.1). |
| 0x1B8 | 22 | 25 | Quest journal (node 939). |
| 0x1CC | 23 | 26 | Game-over page (node 973). Caption "Continue" only. The return code is 460 (0x1CC), a byte offset into section D; table-B state 26 runs the page (Behavior 2). |
| 0x1E0, 0x208, 0x230 | 24, 26, 28 | 27 (all three) | Lead-in pages 2 to 4 (Attack Maneuvers, Quick Inventory, Mote screen), each with a Cross icon and "Next"; state 27 steps through them (section 5). Never entered: only state 27 returns these codes, and no route enters a record that runs state 27 (Behavior 2, survey). |
| 0x1F4 | 25 | 28 | Page with the "Coming Soon" 16:9 picture; it runs the fade step and has no exit. Never entered. |
| 0x21C | 27 | 27 | Lead-in page 1 (Controls), with a Cross icon and "Next"; it runs state 27. Never entered. |
| 0x258 | 30 | 29 | Neopets logo countdown (node 1199), 301 passes. |
| 0x03C, 0x050, 0x064 | 3, 4, 5 | 8, 9, 10 | Item screen pages Items, Charms and Quest items for Roberta (hero index 1; section 14.4). |
| 0x078, 0x08C, 0x0A0 | 6, 7, 8 | 8, 9, 10 | The same three pages for Tor (hero index not 1). |
| 0x26C | 31 | 30 | First map record (area 0). The map page of area k is record 31 + k, code 0x26C + 20 x k, for k = 0 to 76 (SPEC-0051). |

A pass is one front-end call, which runs once per game tick (60 per second; Behavior 1). The node numbers are section J node indices.

### Section E: sprite records (798 x 28)

| Offset | Size | Meaning | Checked on disc |
| --- | --- | --- | --- |
| +0x00 | u32 | Payload offset of the owning J node (back link) | 797 of 797 sprite J nodes point here. Record 797 is all zero and unlinked. |
| +0x04 | 4 bytes | Colour, bytes R, G, B, A. It modulates the sprite's texture, 128 = 1.0 per channel (Section 12). 0x80808080 (neutral) for 769 records | counted; meaning High |
| +0x08 | u32 | Asset index: the number of a section O record (0 to 344). At load it is replaced by a reference to that record (section O base + index x 136). | all 798 values lie in 0 to 344 (High) |
| +0x0C | u8 | Frame grid columns: 1 (778 records), 2 (18), 5 (1), 0 (unlinked record) | counted |
| +0x0D | u8 | Frame grid rows: 1 (778), 4 (18), 2 (1), 0 (unlinked record) | counted |
| +0x0E | u8 | Frame group A count | counted (see below) |
| +0x0F | u8 | Frame group B count | counted |
| +0x10 | u8 | Frame group C count | counted |
| +0x14 | u8 | Current frame index (0 for all 798 records on disc) | counted |
| +0x18 | u32 | 0x7FFFFFFF (528 records), 0 (258), 2 (12), 0 (unlinked) | counted; meaning Low |

Frame group counts on disc, for the 797 linked sprites:
- (0, 0, 0): 778 sprites. These have no frames.
- (1, 4, 8): 18 sprites, 13 frames each.
- (0, 10, 0): 1 sprite, 10 frames.
- The unlinked record has zero counts. So 779 records have no frames in total.

### Section G: label bindings (165 x 16)

| Offset | Size | Meaning |
| --- | --- | --- |
Confidence: High unless marked.

| Offset | Size | Meaning |
| --- | --- | --- |
| +0x00 | u32 | Payload offset of the owning J node (a text node, kind 2). At load it becomes the node's address |
| +0x04 | u32 | Index into the string index (A). At load it becomes the string's address (string index entry + text section start) |
| +0x08 | 4 bytes | Colour R, G, B, A (unsigned bytes, 128 = neutral). It is the start colour of the label text |
| +0x0C | s16 | Letter spacing in pixels, added to each glyph's advance (SPEC-0022, Text layout and glyph drawing, rule 7). On disc: 0 (124 records), -2 (30), -1 (8), -3 (2) and 9990 (1, node 954) |
| +0x0E | s16 | Font id (0, 1 or 2; SPEC-0022 Fonts). A negative id means the label draws nothing. On disc: 0 (99 records), 1 (63) and 2 (3) |

Disc survey: all 112 text nodes (kind 2) have a G record as their data, and each of those records' +0x00 points back to its node (112 of 112). The other 53 records are all zero and belong to no node.

Colour counts on disc:
- Neutral (128, 128, 128, 128): 82 bindings.
- Highlight (200, 115, 50, 128): 30 bindings.
- Zero: 53 bindings (the records that belong to no node).

Node 954 (under the quest journal, node 939; search key 0x33; font 2; a 9-character string) has spacing 9990. It is the journal row's objective template and is never drawn with that spacing: its parent group is gated by predicate slot 12 (always 0), so the page draw skips it, and the journal row drawer uses only its position, alignment and font id (section 14.3) (High; Open question 136).

The 52 strings with a neutral binding include all 28 strings that also have a highlight binding. So a label has a normal binding and, for 28 labels, a second binding with the highlight colour. The highlight colour is the selected or focused state (Medium).

### Section H: pages (7 x 12)

Confidence: High.

| Offset | Size | Meaning | Value on disc |
| --- | --- | --- | --- |
| +0x00 | u32 | Payload offset of a J node (a page root, type 3) | pages 14, 759, 774, 794, 824, 858 and 896, in that order |
| +0x04 | u16 | Zero on disc; no use known | 0 |
| +0x06 | u16 | Default focus index of the page (section 9d) | 0 on all seven pages |
| +0x08 | u16 | Zero on disc | 0 |
| +0x0A | u8 | Page mode: 1 = list page (Down and Up step through the items with wrap; Left and Right do nothing), 0 = spatial page (nearest item in the pressed direction, section 17.2) | 1 on pages 14, 759, 774, 794 and 824; 0 on pages 858 and 896 |
| +0x0B | u8 | Zero on disc | 0 |

All seven page nodes are type 3. The two spatial pages (the start-point chooser and the slot chooser) are never entered, so in the shipped game every reachable page is a list page.

### Section I: item targets (30 x 8)

One record per selectable menu item. Each record belongs to one item node (a J node of kind 4), and that node's data word points at the record. Confidence: High for the fields below.

| Offset | Size | Meaning |
| --- | --- | --- |
| +0x00 | u32 | Payload offset of the item node (relocated at load like the other node offsets) |
| +0x04 | s16 | Target when Cross is pressed on the item: -1, -2 or -3 are the stack codes of section 9 (parent-index rule); 0 or more is a section D record index |
| +0x06 | s16 | Table B state run while the item has focus; -1 = none |

Values on disc, per item (target, focus state; "none" for -1 as a state):
- Pause page: View Map (-3, state 2); View Quests (-3, state 3); Load Game (record 17, state 4); Resume Game (-3, state 5); Options (record 11, none); Quit (-3, state 6).
- Title page: New Game (-3, none); Load Game (record 17, none); Options (record 11, none).
- Options page: Controls (record 12), Audio (record 13), Credits (record 14); no focus state.
- Controls rows: target -3, states 13, 14 and 14. Audio rows: target -3, states 15, 16 and 17.
- Start-point items (nine): target -1, state 19. Save and load slot items (three): target -3, state 20.

No item targets record 18 (the save prompt). What each pause focus state does on Cross is in Behavior 2 (states 2, 3, 5 and 6).

### Section J: UI node tree (1,366 x 52)

| Offset | Size | Meaning | Checked on disc |
| --- | --- | --- | --- |
| +0x00 | u32 | Payload offset of this node's data. For sprite nodes (type 1) it is the E record. | 797 sprite nodes point at E records (Medium-High) |
| +0x04 | u32 | First child (payload offset; 0 if none) | 0 bad pointers |
| +0x08 | u32 | Next sibling (0 if none) | 0 bad pointers |
| +0x0C | u32 | Parent (0 for a top-level node) | 0 child-to-parent mismatches over all child lists |
| +0x10 | u32 | Zero for 1,346 nodes; nonzero for 20. Rebased at load time when nonzero. It is the node's animation list: the kind-2 draw reads its colour animation from it (Text node draw, section 12). None of the 112 text nodes has one | counted; Medium-High for the animation-list use |
| +0x14 | s16 | Position X, relative to the parent (pixels) | High; both position words are zero on 569 nodes |
| +0x16 | s16 | Position Y, relative to the parent (pixels) | High |
| +0x18 | s16 | Scale X, 4.12 fixed point (4096 = 1.0) | values confirmed |
| +0x1A | s16 | Scale Y, 4.12 fixed point | values confirmed |
| +0x1C | s16 | Authoring pivot X: half the node's size, or the centre of its children's box. Not used for placement. | High (not used) |
| +0x1E | s16 | Authoring pivot Y (as +0x1C). Not used for placement. | High (not used) |
| +0x20 | u16 | Height in pixels, before scale | sprite values confirmed (see below) |
| +0x22 | u16 | Width in pixels, before scale | sprite values confirmed (see below) |
| +0x24 | s16 | Handler index for the condition predicate (Section 9b); -1 for 893 nodes (no handler of its own); otherwise 0 to 73 | all 1,366 node records counted: 893 are -1 and 473 select indices 0–73; High |
| +0x26 | u16 | 0 for all 1,366 nodes | counted |
| +0x28 | u16 | 16-bit search key (Section 9b); 0xFFFF (-1 as signed) for 1,196 nodes, which have no key | counted; High on the search use |
| +0x2A | u8 | Row spacing in pixels, applied to this node's children (see Node transform below); 0 to 44 across the nodes, nonzero on 12 nodes, 0 for all 797 sprites | High |
| +0x2B | u8 | Node kind (see below) | counted |
| +0x2C | u8 | Condition mode (Section 9b; drawing rules in Section 12) | not counted; meaning High |
| +0x2D | u8 | Absolute flag: nonzero means the parent transform is discarded and the node is placed from the canvas origin (31 nodes) | High |
| +0x2E | u8 | Horizontal alignment: 0 left edge, 1 centre, 2 right edge (1,119, 207 and 40 nodes) | High |
| +0x2F | u8 | Argument passed to the node's predicate; its meaning depends on the selected predicate | all 1,366 node records counted: 40 distinct byte values; High for the pass-through, Medium-High for the disc survey |
| +0x30 | u8 | Hide flag: nonzero hides the node (0 on every node on disc; set at run time) | High |
| +0x31 | u8 | Animation active flag: set when a trigger starts the node's colour animations, cleared when the colour track ends (Text node draw, section 12) | Medium |
| +0x32 to +0x33 | 2 | Not confirmed | - |

Node types on disc:

| Type | Count | Use |
| --- | --- | --- |
| 0 | 117 | Container (all 117 are top-level roots) |
| 1 | 797 | Sprite (data is an E record) |
| 2 | 112 | Text (label, bound by G) |
| 3 | 7 | Page root (listed in H) |
| 4 | 30 | Clip or group with an extent. Not decoded. |
| 5 | 251 | Group |
| 6 | 52 | Value text: shows a game variable (data is a K record; see Section K) |

The tree: 117 top-level containers have no parent. Every one of the 1,366 nodes is reachable from them by following child and sibling links. Siblings form a chain under each parent.

Sprite sizes on disc: the most common width is 32 (328 sprites) and the most common height is 32 (341). Some sprites are canvas-sized: 102 sprites have width 512 and 101 have height 448. A few values read as negative when treated as signed (for example 65526 is -10), so the size may be used as a signed value in some cases (Low).

Position and scale of the page roots: pages 14, 759, 774, 794, 824, 858 and 896 are at (470, 80), (256, 310), (256, 178), (312, 188), (256, 203), (190, 93) and (0, 0), each with scale 4096 (1.0) and alignment 0. Their row-spacing bytes are 35, 31, 42, 44, 44, 24 and 0.

**Node transform** (High). A node is placed as follows, starting from its parent's transform:
1. Unless the node's absolute flag (+0x2D) is nonzero, start from the parent's transform; otherwise start from the canvas origin (the parent transform is discarded).
2. Translate by (x, y) = (+0x14, +0x16).
3. Row spacing: add s x n to y, where s is the parent's row-spacing byte (+0x2A) and n is 0 for the parent's first child, and otherwise the number of visible siblings from the second child up to and including this node.
4. Alignment (+0x2E): 0, x is the left edge; 1, x is the centre (subtract width x scaleX / 4096 / 2 from x); 2, x is the right edge.
5. After the translation, the node's scale (+0x18, +0x1A, 4.12) multiplies everything drawn under the node, about the node's own origin.
A node whose hide byte (+0x30) is nonzero is not drawn. The pivot words +0x1C and +0x1E play no part.

Scale values seen: 4096 (1.0) on 1,247 nodes, 16384 (4.0), 0, 2949, 4915, 5324, 5529, 3686, and others. Scale is always used as a 4.12 fixed-point value.

### Section K: records (52 x 20)

Each record is five u32 words. The records match the 52 type-6 nodes of section J, in order. A type-6 node is a value-text node: it draws the current value of a game variable as text (High).

- Word 0 (+0x00): payload offset of a type-6 node. Each such node's first word points back to its record (52 of 52; High). This is the only word of the record that is relocated at load (the base is added when it is not zero; see Load-time relocation).
- Word 1 (+0x04): an RGBA colour. The neutral value 0x80808080 is used in 49 records, 0x80669CA4 in 2 records and 0x80027070 in 1 (High for values). It is the text colour, bytes R, G, B and A at +0x04 to +0x07. Each byte is multiplied by the node's tint factor (1.0, or a colour-track byte / 128) and rounded half away from zero (High).
- +0x08 (u16): variable index, 0 to 17 on disc (3 in 28 records). It indexes the boot program's table at data address 0x2DD200 (table C), whose entries point at the game words shown (High).
- +0x0A (u16): font id: 0 (6 records), 1 (44) or 2 (2) (High).
- +0x0C (s16): -3 (34 records), -2 (5 records) or 0 (13 records). It is the letter spacing, passed to the layout and the line draw as the Section G spacing word is (High).
- +0x0E (u8): 2 to 10 on disc (3 in 34 records). It is a draw gate: 0 draws nothing; any other value draws. No other use of its size is known (High for the gate; Open question 133 resolved).
- +0x0F (u8): format: 0 = the variable is a string (9 records), 1 = a decimal integer (42 records), 3 = a time shown as M:SS:CC (minutes, seconds, hundredths; 1 record), or as HH:MM when byte +0x10 is 2 (High).
- +0x10: zero in all 52 records, so the HH:MM variant is not used on disc.

The colour word (+0x04) and +0x0C are described above. The value text is drawn only when the font id (+0x0A) is 0 or more and the font is loaded, and byte +0x0E is not 0. Format 2 (a number with the digit setting of +0x10) exists in the draw but no record uses it. Test vector: colour 0x80669CA4 (bytes A4, 9C, 66, 80) with tint 1.0 draws R 164, G 156, B 102, A 128; under a tint of 0.5 it draws 82, 78, 51, 64.

### Section M (14 x 112), N (20 x 32) and tail

Confidence: High for the layouts and counts; Medium-High for the reach statements.

Section M: 14 UI animation tracks of 112 bytes (1,568 bytes; count at header +0x48). Four of the words are relocated at load time (see Load-time relocation).

| Offset | Meaning |
| --- | --- |
| +0x00 | Kind: 1, 2 (scale) or 3 (colour) |
| +0x04 | Repeat count |
| +0x08 | Remaining repeats (run time) |
| +0x0C, +0x10 | t0 and t1, floats in seconds (the track's time window) |
| +0x14 | Start value: a float for scale; four bytes (+0x14 to +0x17) for colour |
| +0x18 | End value: a float for scale; four bytes (+0x18 to +0x1B) for colour |
| +0x58 | Next sibling track |
| +0x60 | Self |
| +0x64 | First child track (a child runs after its parent's window) |

- On disc: records 0 to 3 are one chain of scale steps, 1.0 -> 0.85 -> 0.95 -> 0.90 -> 1.0, 1 s per step. Records 4 to 7 are a chain of 0.25 s scale steps (1.0 -> 1.2, 1.1 -> 1.0, 1.0 -> 1.1, 1.1 -> 1.0). Records 8 to 11 are a chain of 1 s scale steps, 1.0 -> 1.15 -> 1.05 -> 1.10 -> 1.0. Records 12 and 13 are the only colour tracks (alpha 128 -> 0 over 10 s, and 0 -> 128 over 30 s). No kind-1 track is on disc.
- Colour tracks are the byte tween of section 17.5. No animation slot reaches records 12 or 13, so colour tracks never run in the shipped game.

Section N: 20 animation slots of 32 bytes (640 bytes). The count is the header word at +0x4C (20). The header word at +0x3C (585) is not N's count.

| Offset | Meaning |
| --- | --- |
| +0x00 | Trigger bit (the key-mask bit number, section 9b) |
| +0x04 | End handler: a table B index, -1 for none |
| +0x08 | Track (section M) |
| +0x0C | Repeat count (-1 = forever) |
| +0x10 | Remaining repeats (run time) |
| +0x14 | Next slot |
| +0x18 | Start time, float seconds (run time) |
| +0x1C | Active byte (run time) |

- A node's +0x10 word points at its first slot. 20 nodes have one: 18 sprites with trigger bit 0 and repeat count -1 (track 0 for HUD nodes 342, 352, 361 and 373; track 8 for nodes 428 to 454), and nodes 560 and 566 (keys 0x3C and 0x3E) with bit 4, repeat count 0 and track 0.
- When a slot's run ends: if remaining - 1 is below 1 and remaining is not -1, the slot stops; otherwise it restarts at the current clock (remaining is lowered by 1 when it is above 0). In both cases the end handler is called when it is 0 or more.

Tail: the font table, 3 font descriptors of 272 bytes followed by 282 glyph records of 18 bytes (5,892 bytes; count at header +0x44 = 3). The font format is in SPEC-0022 (Fonts).

### Section O: asset paths (345 x 136)

Each record: an 8-byte texture word (+0x00 to +0x07, below), a u32 kind word at +0x08, a u32 word at +0x0C, then a 120-byte NUL-padded name starting at +0x10. The names are developer paths for art files. The clean side does not need them and this spec does not copy them.

- Sheet assets (kind word 2), bytes +0x00 to +0x07 (High, all 244 records): a 64-bit GS texture register value (the standard TEX0 layout). Its texture base pointers lie in 0x2A00 to 0x31CA, its buffer width is 8 (in units of 64 pixels), its pixel format is 8-bit indexed on 207 records and 4-bit indexed on 37, and it holds the texture width and height (as powers of two) and the palette base pointer. The texture is the sub-entry of resource 532's texture chunk whose base pointer and pixel format equal the record's; the palette is the 32-bit sub-entry whose base pointer equals the record's palette pointer. All 244 sheet records resolve this way.
- Kind word +0x08: 2 on 244 records (sheet assets). -1 (0xFFFFFFFF) on 101 records: a full-screen picture that is not loaded yet. At run time, when the picture is loaded, the word is overwritten with a reference to the buffer that holds it (Behavior 1a).
- Word +0x0C on a picture record: the picture selector v, 2 to 102. The resource number is 575 + v, so the 101 picture records cover resources 577 to 677 one to one. The loader reads the selector as an index offset from resource 575 in the front-end data source.
- A picture record is used only as the asset of the first child sprite of a section D record's top node. Exactly one sprite uses each picture record, except the LoadGame picture (resource 590), which two sprites use (records 15 and 17).

Confidence: High (all 345 records checked).

### Texture chunk (type 4)

The 539,140-byte texture chunk holds the front-end sprite sheet. Its layout is the generic texture layout used by the other packs and is not described here. Its pixel block is uploaded to GS block 0x2A00 as one 256 x 511 image of 32-bit pixels; the section O texture words address the sub-textures and palettes inside that upload (High).

### Options menu and option block (32 bytes, part of each save)

The Options menu has three pages:
- Controls: Vibration, Invert Left-Right and Invert Up-Down. Each is a toggle shown as On or Off.
- Audio: Music, Sound effects and Voice. Each is a slider from 0 to 100.
- Credits.

These seven entries are the whole shipped option set. There is no camera option beyond the two invert toggles (they flip the right-stick bytes, which drive the camera's yaw and elevation; SPEC-0027), no subtitle option, no text-speed option, no sound-mode option, no screen-position or screen-adjust option and no widescreen option. None of the camera, subtitle, sound-mode or screen-adjust words appears in the front-end text, and no page or handler uses them (High). The widescreen flag (SPEC-0027) has a setter, but nothing calls that setter and no code-pointer table holds it, so 16:9 cannot be reached in the shipped game (Medium-High).

The options live in one 32-byte block. The block is copied into every save slot (Save and load), so its layout matters for save compatibility.

| Offset | Type | Use | Default | Confidence |
| --- | --- | --- | --- | --- |
| +0x00 | u32, 0 or 1 | Invert Left-Right | 0 | High (store and effect) |
| +0x04 | u32, 0 or 1 | Invert Up-Down | 0 | High (store and effect) |
| +0x08 | u32, 0 or 1 | Vibration | 1 | High (store) |
| +0x0C | u32, 0 to 2 | Unlabelled three-value cycler; no menu row | 1 | High (rule); Low (meaning) |
| +0x10 | u32, 0 or 1 | Unlabelled toggle; no menu row | 0 | High (rule); Low (meaning) |
| +0x14 | float, 0 to 100 | Music | 75.0 | High |
| +0x18 | float, 0 to 100 | Sound effects | 75.0 | High |
| +0x1C | float, 0 to 100 | Voice | 75.0 | High |

Bytes +0x0C and +0x10 are kept and saved with the block, but no menu shows them (Open question 91).

Defaults: the live option block is set to the values above once at boot, by the start-up table walk (entry 2, see SPEC-0026), before the first vibration sync and before the front end runs. The save slots' option blocks do not keep these defaults. Each slot's block is zero after start-up: the defaults are written into it first, and the empty-slot fill then zeroes the whole slot (Open question 87, resolved). Only the live block keeps the defaults.

Gains at start-up (High): writing the defaults does not run the apply rule (Behavior 3), and neither does a new game. The three gains (Music, Sound effects, Voice) are created at 1.0 when the audio settings are first set up, and they stay at 1.0 until the first apply. So from a fresh boot the game plays music, sound effects and voice at full gain, even though the sliders show 75. The first apply happens the first time the Audio page's Music row runs, the first time an invert row on the Controls page runs (while it has focus), or at a slot load, whichever comes first. With the default values it sets all three gains to 0.75. Opening the Audio page and leaving it without touching anything therefore lowers all three gains from 1.0 to 0.75. Reproduce this.

Vibration and the pad: the pad's rumble gate is written only by the display sync. The sync runs once at boot and again on every pass of the frame loop (SPEC-0033). A Vibration change in the menu, or a load from a save slot, is therefore seen by the gate within one pass (High; Open question 88, resolved). A load copies the slot's option block into the live block, so the loaded Vibration value reaches the gate on the next pass. Rumble works from the first boot with the default option. Only pad 0 can rumble (Medium-High). On a pass where the game status is 9, the gate is forced to 0 (SPEC-0033). The defaults are written before the first sync (Open question 92, resolved).

## Behavior

### 1. Per-frame flow (one call per game tick, 60 Hz)

1. Increment a frame counter. If no root screen is set, stop.
2. If a hold counter is above 0, decrement it and stop. The counter counts ticks. Nothing sets it in the shipped game, so this step never stops a pass (High).
3. If the texture upload for the front end is not ready, stop until it is.
4. Run one front-end pass (Behavior 1b): first draw the current page tree on the 512 x 448 canvas (section 10), and only then run the screen state, run the focused item's focus state, run the generic menu step, and resolve the result. Two handler tables are used: table A holds 74 node predicates (the count is Medium, inferred from the spacing to table B), indexed by a node's handler index (the signed 16-bit field at +0x24 of the J node; see section 9c), and table B holds 31 screen states, indexed by the +0x0A field of the screen's record (section D) or by the +0x06 field of the focused item's section I record. Table A sits directly before table B (see Where it lives on the disc).
5. Return values (High; one convention for every table B state, screen state or focus state): 0 means no decision, so the generic menu step of Behavior 1b runs; -4 means no decision, and the generic Cross and Triangle are not applied this pass (the generic focus moves still run); -1 means back (section 9, Back rule); a section D record (section D start plus a state code) means go to that record. The front end also understands -3 (stay, skip the generic step) and -2 (clear the current screen, so the front end stops running and drawing until a screen is set), but no state returns them; they come only out of the generic step. The list of section 17.1 (-3 no change, -2 close, -1 pop, other = go) is the same scheme after the generic step has run.
6. When the screen changes, run the enter and leave callbacks of the old and new screens. Entering a screen also starts the load of its full-screen picture, if it has one (Behavior 1a).

### 1b. One front-end pass and the generic menu step

Confidence: High.

Each pass runs these steps in order:
1. Input block. While the block flag is set (the pause page sets it while its quit box is open), the menu words given to focus states and to the generic step are zero for this pass. The screen state still gets the real words.
2. Draw the current record's page tree, with the focused flag of section 9d. The draw comes before the screen state of step 3 in the same pass (Medium-High).
3. Run the screen state (record +0x0A, table B) with the focus index (record +0x0C), the screen's pass count (record +0x0E), the record and the input.
4. Find the focused item: the focus index counts the page's kind-4 items in sibling order from 0. If that item's predicate fails, refocus to the first passing item.
5. If the screen state returned 0 or -4, run the focused item's focus state (section I +0x06, table B) with the same arguments. Its return value replaces the result.
6. If the focused item changed, run the leave key mask (bit 3) of the old item and the enter key mask (bit 3) of the new one (section 9b, Key masks).
7. Generic step. It runs only when the result is still 0 (or -4, which allows the focus moves only) and the screen's pass count is 2 or more, so it is off for the first two passes of every screen (about 33 ms). It reads the menu rising-edge word (section 4) and takes the first rule that applies:
   - Cross (0x0040): take the focused item's target (section I +0x04), if the item's own predicate passes. -3: nothing happens. -1: back. 0 or more: section D record N, if its node's predicate passes; if it fails, take the first item of that record's page and its target, and repeat. Any result other than -3 plays front-end sound slot 5.
   - Triangle (0x0010), or a Cross target of -1: the back rule (section 9). It plays sound slot 6 for Triangle; for the Cross case slot 5 has already played.
   - Otherwise one focus move (section 9d), tested in the order Down (0x4000), Up (0x1000), Right (0x2000), Left (0x8000). A move that succeeds plays sound slot 0, 1, 2 or 3 in that order.
   With a -4 result only the focus moves run.
8. Resolve the result. Stay (-3, or 0 after a generic step that did nothing): the pass count grows by 1, up to 9,999. -2: the current screen is cleared. -1: the stack is popped and the new top becomes current. A record: apply its stack rule (section 9), make it current, set its pass count to 0, start its picture load (Behavior 1a) and set its default focus (section 9d).

One-pass lag (Medium-High). Because the tree is drawn before the screen state, the focus state and the generic step run, every change a state makes to node fields (widths, positions, colours, alpha, or the words a node predicate reads) reaches the screen on the next pass. So the in-play HUD (state 11, SPEC-0023) shows its bar widths, fades and placements one pass after it computes them. Text that a state draws itself during its own run (the HUD prompt captions and the price caption of section 15) appears in the same pass. A new screen resolved in step 8 is first drawn on the next pass.

Front-end sound slots (High). Seven slots are filled once at start-up, on the menu sound bank: slots 0 to 4 play cue 0x9A (focus moves: Down, Up, Right, Left; slot 4 unused), slot 5 plays cue 0x98 (select) and slot 6 plays cue 0x99 (back). "Front-end menu sound N" in this spec means slot N. They play at the fixed volume of Behavior 3c.

### 1a. Full-screen pictures (resources 577 to 677)

Loading (High):
1. At front-end set-up, two picture buffers of 267,776 bytes each are reserved. The base number 577 is stored with them.
2. When a screen is entered, its node tree is searched children first. The search looks for the first sprite node (kind 1) whose asset record (section E +0x08, section O) has kind word -1, meaning a picture that is not loaded.
3. The picture resource is 575 + v, where v is the record's +0x0C word (2 to 102). Its first 267,264 bytes are read into the free buffer. The two buffers alternate: each new picture load goes into the buffer the previous load did not use.
4. The record's kind word is set to a reference to that buffer, so the picture counts as loaded and the sprite draws it. At the same moment, the asset record that last used that buffer (the picture from two loads ago) has its kind word set back to -1, so it counts as not loaded again and is read afresh the next time its screen is entered. A picture leaving the screen changes nothing. (High)
5. Only one picture load runs at a time. A load starts only when no load is pending and the found record's kind word is still -1; otherwise the screen entry starts nothing.
6. While a load is pending, each front-end pass ends before step 1 of Behavior 1b: nothing is drawn, and no screen state, focus state or generic step runs. When the read finishes, the next pass turns the buffer into a texture (when the read data is a texture block, header word +0x04 = 4), clears the pending state and runs normally. When no archive was given at front-end set-up, the read is skipped and the load counts as finished at once. (High)
7. The read is an ordinary archive read of the picture resource. It plays no sound. Its completion only marks the read as finished.

Picture of each screen record (section D record, resource, picture name in plain words). Confidence: High for the record-to-picture link (disc); the reach column is per row.

| Record | Resource | Picture | Reach |
| --- | --- | --- | --- |
| 1 | 577 | Legal screen, 4:3 | Attract (state 1); after the logo page: 301 passes on the first visit since boot, 2 passes on later visits (High). |
| 2 | 578 | Options background | Pause page; Select in play. |
| 3, 4, 5 | 579, 580, 581 | Item screen pages Items, Charms (motes) and Quest items, named for Roberta | Item screen entry codes of group B (hero index 1, Roberta; section 14.4). High. |
| 6, 7, 8 | 582, 583, 584 | The same three pages, named for Tormund | Item screen entry codes of group A (hero index not 1, Tor). High. |
| 10 | 585 | Title (start screen) | Title; reached. |
| 11, 12, 13 | 586, 587, 588 | Options, Vibration and Audio backgrounds | The Options, Controls and Audio pages themselves (records with no state); reached from the title and the pause page (High). |
| 14 | 589 | Credits background | Credits; reached from the Options page's Credits item and by the in-play story-bit gate after the finale (High). |
| 15 | 590 | Load game | Start-point chooser container; never entered (High, static survey). |
| 16 | 591 | Load or new game | Record with no state; never entered (High, static survey). |
| 17 | 590 | Load game (the same picture record as record 15) | Load prompt; reached. |
| 18 | 592 | Save game (a byte copy of the load-game picture) | Save prompt; reached by Square at a save point (section 7.3). |
| 22 | 593 | Quest states | Quest journal; View Quests, D-pad Right in play, and the L2/R2 page ring (section 14). |
| 23 | 594 | Game over, 512 x 128 | Game-over page; reached on game over. It is the picture of the page's sprite record 526. |
| 24, 26, 28 | 595, 597, 599 | Lead-in pages: Attack Maneuvers, Quick Inventory, Mote screen | Pages 2 to 4 of the lead-in sequence 27, 24, 26, 28 (state 27); never entered (High, static survey). |
| 25 | 596 | Coming Soon, 16:9 | Never entered (High, static survey). |
| 27 | 598 | Controls lead-in page | Page 1 of the lead-in sequence; never entered (High, static survey). |
| 30 | 600 | Neopets logo, 16:9 | Logo countdown, after every card-state visit and after the title's idle timeout. The logo is what the player sees there (High). |
| 31 to 107 | 601 to 677 | 77 area maps | Map page of area k = record 31 + k; picture = 601 + k (SPEC-0051 section 1). |

Records 0, 9 (in-game play), 19, 20 (book reader), 21 and 29 have no picture. Records 21 and 29 are never entered.

Handler return values follow Behavior 1, step 5: a state returns 0 when it makes no decision (the generic step of Behavior 1b then runs), -4 to block the generic Cross and Triangle, -1 for back, or the section D start plus a state code N to go to record N / 20. Each N is written as a code in the entries below.

### 2. Front-end state table (table B, 31 states)

Each state is a handler called once per tick while its screen is active. The index is the +0x0A field of the screen's record (Behavior 1, step 4; section D). Confidence is given per entry. Entries marked "partly described" were not confirmed in full.

| Index | Behaviour in words | Confidence |
| --- | --- | --- |
| 0 | Card state (record 0). Each pass it first runs the controller check (section 2b); while pad 0 is not usable it does nothing else, so the card job does not start. On the first visit in a session it sets text variable 0 to "1" (the MEMORY CARD slot) and text variable 1 to the free space the save needs, and starts the boot card check (SPEC-0025, Card jobs; no card: Retry / Continue; short of space: Retry / Continue). On every later pass it runs the message box and, when the card job is no longer busy, goes to code 0x258 (record 30, the logo countdown). There is no tick wait. The first-visit flag is written once and is never cleared, so a later visit lasts one pass. | High |
| 1 | Attract (record 1, legal screen picture). First pass: selects controller port 1 as the menu input. The wait is 300 passes on the first attract entry since boot, and 1 pass on every other entry, including the entry after a game over; it counts only on passes where the controller check reports ready. When it ends: queue movie 0, then movie 86 (the title movie), reset the lead-in page counter to 0, and go to code 0x0C8 (record 10, the title). The page reads no input and Triangle is off, so it cannot be skipped. The game-flow spec gives the timings (SPEC-0026, Attract mode and title). | High |
| 2 | View Map (focus state of the pause item View Map; section I). Runs the common overlay tick. On a Cross press (second input word, 0x0040) it sets the selected map area to the hero map area (SPEC-0051 section 1) and returns the code of that area's map page (base + 0x26C + 20 x area). If the hero map area is negative it stays. | High |
| 3 | View Quests (focus state of the pause item View Quests). On a Cross press (second input word) it sets the selected map area to the hero map area and returns the journal code (base + 0x1B8, record 22). | High |
| 4 | Load Game focus state (pause item Load Game). Each pass it resets the load picker stage, the slot cursor and the message box, and returns 0, so the generic Cross opens the item's target, record 17 (the load prompt), with cue 0x98. It does not end the screen. | High |
| 5 | Resume Game (focus state of the pause item Resume Game). Each pass it sets the pause music level and closes the stream gate (Behavior 3b). On a Cross press it plays cue 0x95 and returns code 0x0B4 (play). | High |
| 6 | Quit (focus state of the pause item Quit). On a Cross press it plays cue 0x9A and sets the pause-box flag to 1, which opens the quit box; the box is then run by state 7. It returns 0. Load Game, Resume Game and Options open no box. | High |
| 7 | Pause page (record 2, node 12, root 14). Each pass it sets the pause music level and closes the stream gate (Behavior 3b). Select, while its box is closed, plays cue 0x95 and returns code 0x0B4. The quit box is record 26 of the system message table (design table 9 of resource 533; SPEC-0022), a quit confirmation whose first label is "Quit" (the Circle answer) and whose second is "Resume" (the Triangle answer). Circle: cue 0x98, the world is torn down and the page returns code 0x000. Triangle: cue 0x99, the box closes and the pause page stays. While the box is open the page sets the input block (Behavior 1b, step 1), so Triangle answers the box and does not leave the page, and the focus states and the generic step see no input. Section 6 and section 13.1. | High |
| 8 | Item screen, Items tab (records 3 and 6; 20 cells in four rows of five). Draws the cells, handles cursor movement, Cross (use), Circle (quick-item marker), Square (Tattered Cloak on or off), L1/R1 (tab), L2/R2 (page ring) and Triangle or Start (close). Section 14. | High |
| 9 | Item screen, Charms tab (records 4 and 7). It runs the Mote picker of the in-play second quick panel (SPEC-0021, Second quick panel and Mote equipping): directions from the menu rising-edge word, buttons from the second input word. Cross equips the selection into equipment word 0, Square into word 1 (when the shield family is present) and Circle into word 2 (when the weapon family is present). It also runs the item-screen step (section 14.2). It stays only while the tab is 1; otherwise it returns the tab dispatch code (section 14.1). | High |
| 10 | Item screen, Quest-items tab (records 5 and 8): the owned category-7 records, scanning the 60 slots of the list and showing at most 20 cells. The scroll offset is always 0 and the cursor never passes cell 19 (section 14.1). Cross and Circle run the usability test (the entry exists, has an item-state record, and its category is not 7). Every entry of this tab is category 7, so both always play cue 0xA2 (section 14). | High |
| 11 | In-game play: draws the HUD (record 9's tree, SPEC-0023), runs the play gates, opens the menu, and shows the quick-slot item counts. It does not re-arm the HUD timers on each call (Behavior 6). | High |
| 12 | Title. Every pass it sets the game-state word +0x68 to 1 if it is 0. Pass 0: idle counter 0, stream gate closed. Pass 1 on: a pass whose rising-edge word (+0x89C) is 0 adds 1 to the idle counter, any other pass sets it to 0; when it exceeds 30 x 60 = 1,800 (data word 0x2DCE38 times the tick rate) it returns code 0x258 (the logo countdown), so with no input it leaves on pass 1,801. A held button does not reset the count; only new presses do. New game: focus 0 (New Game), page pass 5 or more and Cross (0x0040) newly pressed: reset the world (all flags cleared), cue 0x9A, clear both start-point overrides, select the start hero, set flag 707, load packs 691 and 909, clear the four music overrides and return code 0x0B4 (play). Every other pass: set the music bank override to 25 (bank resource 35), run the controller check and return 0, so the generic Cross runs Load Game and Options (records 17 and 11, cue 0x98) from pass 2. There is no "press start": Start does nothing and Triangle is off (back rule -2). | High |
| 13 | Vibration toggle: changes the Vibration word (+0x08) once per press and does not run the apply rule (Behavior 3). | High |
| 14 | Invert toggle: it receives the focus index. On a press it toggles +0x00 when the focus index is 1 (Invert Left-Right) and +0x04 for any other index (Invert Up-Down, index 2). It runs the apply rule on every pass, pressed or not (Behavior 3). | High |
| 15 | Music volume slider (Behavior 3). | High |
| 16 | Sound effects volume slider (Behavior 3). | High |
| 17 | Voice volume slider (Behavior 3). | High |
| 18 | Credits roll (record 14). On its first pass it turns the stream updates off (Behavior 3b). On every pass it sets the music bank override to 8, which is bank resource 18 (SPEC-0009, Overrides), then runs the controller check; while that reports not ready the roll is not drawn and does not move. The rows of design table 15 scroll up from y = 449 at a speed that eases between 1.0 and 1.5 pixels per pass; the full roll lasts 14,123 passes (235.4 s). The roll rule, its data words and its timing are in SPEC-0026 (Credits). From the Options page the credits are left by Triangle (generic back, cue 0x99) or, when the scroll ends, by -1 (back to Options, no cue). When entered by the in-play story-bit gate, any button press returns -4 (Cross and Triangle ignored), and the end of the scroll tears the world down (code 0x000). | High |
| 19 | New game and start-point confirm: sets stage ids, plays cue 0x9A, loads the world (Behavior 5). | High |
| 20 | Focus state of the three slot items of page 896 (never entered): Cross plays cue 0x9A and returns 0x154 (record 17, the load prompt). | High |
| 21 | Load game (record 17, code 0x154): stage 0 starts the card read and stage 1 runs the card box; both return -4, so Triangle does nothing. When the card job ends with a cancel answer in the box, or with no save data, it returns -1 (back to the screen below, no cue). Stage 2 shows the picker and returns 0, so Triangle leaves by the back rule to the pause page or the title (cue 0x99). Cross loads the chosen slot and returns 0x0B4 (Behavior 7.2). | High |
| 22 | Save game (record 18, entered by a direct screen set from a save point, not by a code): holds the hero idle, card scan, slot picker, Cross writes, Triangle in the picker plays sound slot 6 (cue 0x99), cancels and returns code 0x0B4; a nonzero card state word after the write goes back to the card scan and the picker (Behavior 7.3). | High |
| 23 | Box host (record 19, node 924). Entered with code 0x17C while the overlay box is busy. On its first pass it resets the screen-effect object (any running fade is cancelled, bar colour black, bar heights 32) and requests letterbox bars up over 10 frames, unless a script holds the bars. It runs the box each pass, and it draws the price caption every pass (section 15.2). When the box is idle it plays cue 0x97, requests the bars down over 10 frames and returns code 0x0B4. | Medium-High (High for the caption draw) |
| 24 | Book reader (record 20, node 930; section 6.1). Entered with code 0x190 while the book cursor or the saved book word is 0 or more. It shows the book at the cursor index minus 1. Cross plays cue 0x9A, sets the cursor to -1, unloads the book, and returns code 0x0B4. With no valid index it sets the cursor to -1, clears the four music overrides and returns code 0x0B4 with no cue. | High (flow and exit); Medium-High (node identity) |
| 25 | Quest journal (record 22): builds the journal rows (accepted quest rows, then held treasures), moves the cursor and the scroll, opens a row's map on Cross, and runs the L2/R2 page ring. Section 14.3. | High |
| 26 | Game-over page (record 23, node 973). It shows only the caption "Continue" and two sprites with no animation frames, and offers no choice. On its first pass it runs the same HUD and menu reset as the pause and item-screen openers (SPEC-0023 section 4.1): every HUD visibility timer, the quick-panel flags, the Square prompt count, the countdown counter and the monster bar timer go to 0, so the HUD is not drawn when play resumes after Continue until a re-arm. One sprite (record 526) shows the 512 x 128 game-over picture, resource 594 (Behavior 1a). The other (record 527) is the small Cross button icon (asset "x_sm") beside the "Continue" caption, shown once predicate slot 72 passes (page pass count above twice the tick rate, about 2 s). Only Cross is read. The page counts its passes whether or not a button is down, and the count does not restart when Cross is released. The count is set to 0 on the pass that hands over to the page, one pass before page pass 1. The page exits on the first page pass numbered 122 or later on which Cross is newly pressed (up on the pass before, down on the current pass). A press made earlier and still held does not exit. Page pass 122 is about 2.03 s at 60 passes per second. On exit, the game-over flag is cleared. If the pending flag is set (a slot was confirmed in the save or load picker earlier in the session, section 7.3), the game re-applies the session snapshot from memory with no card access, resets the heroes, clears the music overrides and goes to code 0x0B4 with no teardown, so Continue resumes from the last save or load. Otherwise the world is torn down and the front end goes to code 0x000. See SPEC-0055 for the timing and test vectors. | Medium-High (exit rule); High (sprite content) |
| 27 | Lead-in page stepper (records 27, 24, 26 and 28: Controls, Attack Maneuvers, Quick Inventory, Mote screen). Any newly pressed button from page pass 11 turns the page; the fourth press clears the music overrides and enters play (code 0x0B4). Section 5 gives the full rule. No route enters these records, so the pages are never shown (Behavior 2, survey). | High (mechanism and reach); Medium (that the pages were cut) |
| 28 | Fade step: each tick lowers the Music and Sound effects gains by 0.05, clamped to 0 to 1. It does not change the option block, so the next apply rule or a load restores the gains. | High |
| 29 | Logo countdown (record 30, node 1199). It shows the record's full-screen picture, the Neopets logo (resource 600). On its first pass (pass 0) it selects controller port 1 as the only menu input source (input mask = pad 0, then the merged menu words are rebuilt; this draws nothing) and loads the countdown with 300. Each later pass k sets the countdown to 300 - k, and the screen leaves to code 0x014 (attract) on pass 300, so the page lasts 301 passes (5.02 s at 60 passes per second). No input is read, the page has no items and Triangle is off, so the logo cannot be skipped. It writes no music override (Behavior 3a). | High |
| 30 | Map page (records 31 to 107): one area's picture (resource 570 + record, which is 601 + area), its quest and treasure icons, the cursor, the hero marker, and the exits (Select to play, Cross or R2 to the journal, L2 to the item screen). Specified in SPEC-0051; summary in section 14.4. | High |

State return survey (High). Every value each state can return; "0" means the generic step runs. Cues are in brackets.

| State | Returns |
| --- | --- |
| 0 card | 0; 0x258 when the card box is idle |
| 1 attract | 0; 0x0C8 after the wait |
| 2 View Map (focus) | 0; the map page code on Cross when the hero map area is 0 or more (no cue) |
| 3 View Quests (focus) | 0; 0x1B8 on Cross (no cue) |
| 4 Load Game (focus) | 0 |
| 5 Resume Game (focus) | 0; 0x0B4 on Cross (0x95) |
| 6 Quit (focus) | 0 (opens the box, 0x9A) |
| 7 pause page | 0; 0x0B4 on Select (0x95); 0x000 after Quit in the box |
| 8, 9, 10 item tabs | 0; 0x0B4 (Triangle or Start, 0x95); the tab codes; 0x1B8; a map code |
| 11 play | 0; 0x118, 0x190, 0x1CC (the gates); 0x028; a tab code; 0x1B8; a map code; 0x17C |
| 12 title | 0; 0x258 after 30 s with no press; 0x0B4 (new game) |
| 13 to 17 option rows (focus) | 0 |
| 18 credits | 0; -4; 0x000; -1 |
| 19 start point (focus) | 0; 0x0B4 |
| 20 slot item (focus) | 0; 0x154 |
| 21 load | -4; -1; 0; 0x0B4 |
| 22 save | 0; 0x0B4 |
| 23 box host | 0; 0x0B4 (0x97) |
| 24 book | 0; 0x0B4 (0x9A) |
| 25 journal | 0; a map code; a tab code |
| 26 game over | 0; 0x0B4; 0x000 |
| 27 stepper | 0; 0x1E0, 0x208, 0x230, 0x0B4 |
| 28 fade | 0 |
| 29 countdown | 0; 0x014 |
| 30 map | 0; 0x0B4; 0x1B8; a tab code |

Reach (High, static). Following these returns, the section I targets, the back rule and the two direct screen sets (record 0 at start-up and record 18 at a save point), records 15, 16, 21 and 24 to 29 are never entered. So states 27 and 28 never run, the start-point chooser (page 858) and the slot chooser (page 896) are never shown, and focus states 19 and 20 never run.

### 2a. Which screens freeze the world (Confidence: High unless marked)

The world update is gated by the play-active word, game-state word +0x04 (game globals at data address 0x35C980; SPEC-0026, Pause and freezing screens). The main loop clears it on every pass just before the front end runs. Only three table-B states set it back to 1 while they run:

- state 11 (in-game play), on every pass it gets past its credits, book-reader and game-over gates (section 6);
- state 23 (box host), which also sets the word +0x20, so the game-state code is 4 and the world, the AI and the level scripts keep running while a dialogue box is up; the level script's button query returns false meanwhile. On its first pass it resets the screen-effect object (any running fade is cancelled, bar colour black, bar heights 32) and requests letterbox bars up over 10 frames, unless a script holds the bars. When the box is idle it plays cue 0x97, requests the bars down over 10 frames and returns code 0x0B4 (SPEC-0052, Screen effect);
- state 27 (lead-in page stepper), on page passes 3 to 7 of each page (its record's pass counter +0x0E is 3 to 7); the stepper never runs in the shipped game (section 5).

Every other state leaves the word at 0, so the next pass's game-state code is 9 and the world is frozen from the pass after the screen first runs. These screens therefore freeze the world: the pause page and its focus states, the item screens (states 8 to 10), the map (state 30), the quest journal (state 25), the options rows and sliders, the load and save pickers (states 21 and 22), the quit box, the book reader (state 24, which sets only the word +0x20), the credits (state 18) and the game-over page (state 26). While frozen, the front end itself, the screen fade, the sound cues and the music (with the pause music level of Behavior 3b), the card step and the memory-card tasks keep running.

Timing (Medium-High; it rests on the page switch taking effect one pass after the return code). A screen opened by a return code on pass N runs from pass N+1, and the world is frozen from pass N+2. On the way back, the pass on which the screen returns the play code is followed by one more frozen pass while state 11 runs again. A screen set directly from the world update (the save screen of section 7.3) runs in the same pass as the Square press, so the world is frozen from the next pass.

Table A (74 predicate slots, indexed by a node's handler index) is described in section 9c. All 74 slots are described there; the few with Low confidence are listed in Open question 85.

### 2b. Controller check (Confidence: High)

One check is run each pass by the card state (0), the attract (1), the pause page (7), the title (12), the three Audio rows (15, 16 and 17), the credits (18), the start-point state (19), the quest journal (25) and the lead-in page stepper (27). The check plays no cue. If pad slot 0 is usable (the pad-usable test of SPEC-0033) it reports ready. Otherwise it opens a message box with design table 9 record 25 (the message that the analog controller is not detected; no answers; SPEC-0022), updates that box, and reports not ready. Effects of "not ready": the card state does nothing (the card job does not start); the attract wait does not count; the credits roll is not drawn and does not move. The title does not stop its idle count for it. In play the same test opens the pause page (the pad-not-usable auto-pause of section 6).

The logo countdown (29), the attract (1) and the page stepper (27) also select controller port 1 (pad 0) as the only menu input source on their first pass, and the merged menu words are rebuilt from it (SPEC-0033 Controller map).

### 3. Options: toggles, volume sliders and the apply rule

Confidence for the whole section: High on the block layout, on the toggle, cycler, slider and apply rules, and on which Controls row toggles which invert word.

Toggles (options 0 to 2, and the unlabelled toggle at +0x10):
- Each press of Left or Right on a toggle changes it once: new = (old != 1). So 1 becomes 0, and any other stored value becomes 1.
- On the Controls page, a toggle shows its first value node when the stored value is nonzero (mode 2), and its second when it is zero (mode 3) (predicate slots 66 to 68, section 9c; High). The Vibration row's value texts are On and Off; the two invert rows' value texts are Yes and No (High). Test vector: Invert Up-Down word 1 shows Yes; 0 shows No; Vibration word 0 shows Off.
- Vibration: the Vibration row reads the vibration word (option +0x08), and its toggle is table B state 13. The Vibration toggle does not run the apply rule. Its effect on the pad is given in SPEC-0033, and it reaches the pad on the next pass of the frame loop (see Options, Defaults).
- Invert: the invert toggle (table B state 14) receives the page's focus index, not the node argument. Controls row 1 (Invert Left-Right) toggles +0x00 and row 2 (Invert Up-Down) toggles +0x04. The Yes and No displays read +0x00 through predicate slot 67 and +0x04 through slot 68, which matches.

Cycler (option +0x0C, no menu row):
- A press of Left sets the value to 2 if it is below 1, and to value minus 1 otherwise.
- A press of Right sets the value to (value + 1) mod 3.
- The two steps run in that order when both are pressed on the same tick.

Volume sliders (Music, Sound effects, Voice):
- While the slider is focused, a held Right adds 1.0 per tick, and a held Left subtracts 1.0 per tick. If both are held, only the Right step applies. The value is clamped to 0 to 100. Sliders act on held input; toggles and the cycler act once per press.
- One-time knob set-up: a page flag (data word 0x2DCF80, 1 on disc) makes the first pass of the Music row also run the Sound-effects and Voice rows once with no input, so all three knobs are placed. The flag is then cleared and is not set again in the session. Until the first apply the knobs stay at the x positions stored in the page data (those positions are not given here; Open question 5a).
- Each slider row runs the apply rule below on every pass that it runs, whether or not a button is held.

When the apply rule runs (High): on every pass that the Music, Sound-effects or Voice slider row runs (held input or not), on every pass that an invert row runs (pressed or not), and after a slot load (SPEC-0025). Nothing else runs it: not start-up, not a new game, not the Vibration toggle, not the fade step (state 28). Until the first apply the three gains are 1.0 (see Options, Gains at start-up).

Timing of the effect (High): the gains change on the pass the slider changes; the sound output follows on the next music-controller tick (SPEC-0009). Sounds already playing keep the volume they started with; only sounds started afterwards use a new Sound-effects or Voice gain. The bank music moves toward its new target by at most 15 per tick. The stream ceiling drops at once when it is lowered and rises by 1 per tick when it is raised. The invert flags written by the apply rule are used by the next pad read. Vibration is not part of the apply rule (SPEC-0033: the gate follows the option within one pass).

Apply rule:
1. Gains: each of the three volume values is multiplied by 0.01 and clamped to 0.0 to 1.0. The results are the Music, Sound effects and Voice gains. The Music gain multiplies the music row's Level, so it scales the bank music's target volume; the pause page also reads it (Behavior 3b). The Sound-effects gain sets the default cue volume, trunc(row Scale x gain), which every world sound played at "default volume" uses; it also scales the stereo streams' fade-in ceiling and the stingers (SPEC-0009, Volume and ramps, Streams and Effect and stinger index; SPEC-0031). The Voice gain has one reader only: the emote-clip player, which plays a clip at trunc(1024 x Voice gain) (SPEC-0009, Emote clip lookup). Hero and enemy vocal cues played through the normal cue call follow the Sound-effects gain, not Voice. Confidence High.
2. Knobs: each slider's knob x-position = trunc(value x 1.29 + 2.0), written to the slider's knob element: 0xCE for Music, 0xCF for Sound effects and 0xD0 for Voice. Confidence High.
3. Pad inversion: Invert Left-Right sets a flag that complements the first pad's right-stick X byte (255 minus the value). Invert Up-Down sets a flag that complements the right-stick Y byte. The left-stick bytes are never changed, so inversion does not affect locomotion. The flags are written to the first pad only; no other writer is known. Confidence High for the bytes and axes; Medium-High for the first-pad-only rule.

Row draw order (High): the rows of each options page are drawn top to bottom in index order. Row n is at the page's y plus the row spacing times n:
- Options page (y 178, spacing 42): Controls 178, Audio 220, Credits 262.
- Controls page (y 188, spacing 44): Vibration 188, Invert Left-Right 232, Invert Up-Down 276.
- Audio page (y 203, spacing 44): Music 203, Sound effects 247, Voice 291.

Change feedback (High): option changes are silent. The helper that runs after a slider change is the controller check of section 2b, which plays no cue. Toggles and sliders play no cue, and Left or Right on a list page is not a focus move (section 4.1), so no front-end move sound plays either.

### 3a. Music overrides cleared on return to play

Confidence: High for the clears; Medium for the screen names.

The four music overrides set by level scripts (bank, level, reverb type and reverb volume; SPEC-0009, Overrides) are all set to -1 at these exits:
- the title start (state 12);
- the page stepper's fourth press (state 27, the lead-in pages; never reached);
- new game (state 19);
- the save screen, both when it completes and when it is cancelled (state 22);
- the book reader exit (state 24);
- the game-over page, on the branch that re-applies the pending snapshot (state 26);
- the map page exit to play (state 30);
- the item screens' shared close (Start or Triangle; it plays cue 0x95), used by states 8, 9, 10 and 21;
- the world (re)entry routine, the exit that restores the world at the last save area or at area 68;
- every world activation (SPEC-0006).

The pause page (state 7) and the Resume Game focus state (state 5) also clear a level-script bank override: on every pass they write all four overrides, bank -1, level trunc(400 x Music gain), reverb type -1 and reverb volume -1 (Behavior 3b). So a bank override set by a level script ends as soon as the pause page runs; after the pause the bank music follows the area's music row again.

They are not cleared by: the logo countdown (state 29; it writes no override and leaves to the attract, not to play) or the box host (state 23), which returns to play with the overrides kept. Quit does not clear them itself; they are cleared only by the world activation that follows. Two screens set the bank override on every pass instead of clearing it: the title (state 12) sets 25 (bank resource 35) on every pass that does not start a game, and the credits (state 18) set 8 (bank resource 18). The value 8 left by the credits stays in force on the Options page until the title or another writer replaces it. Because the music controller ticks on these front-end passes once the title has been shown (SPEC-0009, Per-tick controller, Pass gate), the title's bank 35 and the credits' bank 18 are heard (High).

A clean side should keep this list: a bank or level override set by a level script survives a dialogue box, and ends at a pause and at the exits listed above.

### 3b. Pause music level and the stream gate

Confidence: High for the values, the states and the writers; Medium-High for what the stream gate skips.

While the pause page is up, the bank music plays at a fixed lower level. On every pass of table B state 5 (Resume Game focus) and state 7 (the pause page), the front end writes all four music overrides (SPEC-0009, Overrides): the one-tick level override = trunc(400 x Music gain), and -1 for the bank and both reverb overrides. Writing -1 to the bank override clears any bank override a level script set (Behavior 3a). Because the level override lasts one tick, it holds only while one of these two states runs; the next tick of play goes back to the row Level. With it the bank's target volume is V = trunc(400 x Music gain) in place of row Level x Music gain (SPEC-0009, Volume and ramps). No other front-end state sets a level override; the only other setter is the level-script call OverRideMidiSettings.

The same two states also close the music controller's stream gate (the music object's stream-update flag) on every pass. While the gate is closed, the per-tick music controller skips its whole stereo-stream step (SPEC-0009, Streams): no stream fades in or out, and no stream is started or stopped. The bank music, the default cue volume and the reverb sends are still updated. The gate is opened on the first pass of in-game play (state 11), and closed at start-up (the flag starts at 0), on the first pass of the title (state 12), on the first pass of the credits scroll (state 18) and on every pass of states 5 and 7.

The credits scroll (state 18) and the title (state 12) also set the music bank override on every pass (Behavior 3a) (High).

Test vectors: Music gain 1.0 gives V = 400, sent to the driver as trunc(400 x 127 / 1024) = 49; Music gain 0.5 gives V = 200, sent as 24; Music gain 0 gives 0 (silent). Compare row Level 700 at gain 1.0, sent as 86. Music 75 (gain 0.75): level override trunc(0.75 x 400) = 300; while a stinger or an emote clip plays, the duck gives trunc(0.7 x 300) = 210.

### 3c. Front-end sound volume

Confidence: High.

Menu sounds played from the front-end menu sound table ("front-end menu sound N" in this spec) are always played at the fixed volume 1024 (full). They do not follow the Sound-effects slider and still play when it is 0. Front-end cues that a screen plays directly through the normal cue call (for example 0x95 on Resume and on the item screens' close, 0x9A, and 0xA9 on save complete) use the default cue volume, trunc(row Scale x Sound-effects gain), so they follow the Sound-effects slider (SPEC-0031); once that default is 0 they are not played at all. The music controller and the per-pass voice update run on title and menu passes once the title has been shown (no movie playing, the title-start word set and the game-state code 9; SPEC-0009, Per-tick controller, Pass gate), so on the title, the options, the credits and the pause page these cues get trunc(row Scale x Sound-effects gain). Before the first title pass (the card check, the logo, the legal screens and the attract movies) neither runs, and the default cue volume is 1,024 (Open question 5b, resolved).

### 4. Menu input (logical word)

Each tick the menus read the combined snapshot of the controlling set, which is always pad slot 0 (SPEC-0033 Controller map). Navigation, Cross, Triangle, Start, Select, L1, R1, Circle and Square on the item screen, and the pickers, use its newly-pressed2 word (+0x89C, the rising-edge word, stick directions included); sliders use its held2 word (+0x898). The screen states read the bits below. Direction names (High): 0x1000 Up, 0x4000 Down, 0x8000 Left, 0x2000 Right, from the stick mapping (stick up adds 0x1000) and from the focus search (0x1000 prefers items above, 0x4000 items below, on the y-down canvas). What each button does on each screen is listed in one table in SPEC-0026 (Controller map for menus and screens); this section gives the bit values, the words and the generic rules behind it.

| Bit | Role | Notes |
| --- | --- | --- |
| 0x1000 | Up | Save and load picker: previous slot. List menus: previous selectable item (section 4.1). Vertical analog bit (below threshold). |
| 0x4000 | Down | Save and load picker: next slot. List menus: next selectable item (section 4.1). Vertical analog bit (above threshold). |
| 0x8000 | Left | Slider decrease; toggles. Horizontal analog bit (below threshold). In play, D-pad Left (read from the second input word) opens the map of the hero's area (section 6). |
| 0x2000 | Right | Slider increase; toggles. Horizontal analog bit (above threshold). In play, D-pad Right (read from the second input word) opens the journal (section 6). |
| 0x0040 | Confirm (Cross) | Generic menus: opens the focused item's target (Behavior 1b, cue 0x98). Title start; confirm in save, load and new game; leaves the book reader (section 6.1). |
| 0x0010 | Back (Triangle) | Generic menus: the back rule (section 9, cue 0x99). Save picker: returns to play without writing (section 7.3). Item screen: closes it (section 14.2). |
| 0x0800 | Open in-game menu (Start) | In play. Plays cue 0x94 when the busy check is zero. |
| 0x0100 | Open in-game menu (Select) | In play. Plays cue 0x94 when the busy check is zero. |
| 0x0004 | Previous tab (L1) | Item screen only: moves to the previous tab, with wrap (Section 14). Plays cue 0x98. |
| 0x0008 | Next tab (R1) | Item screen only: moves to the next tab, with wrap. Plays cue 0x99. |
| 0x0020 | Quick-item marker (Circle) | Item screen, Items tab: sets or clears the active hero's quick-item selection marker (Section 14.2). Plays cue 0x9A. Charms tab: equips the selected Mote into equipment word 2. |
| 0x0080 | Cloak swap (Square); book take on a rising edge | Item screen, Items tab only (Section 14). Plays cue 0xA1 when it runs. In play, with a pending book index, it takes the book (section 6.1; cue 0xAF). |
| 0x0001 | L2 | Page ring: previous page (item screen -> journal -> map -> item screen; section 14). |
| 0x0002 | R2 | Page ring: next page (item screen -> map -> journal -> item screen; section 14). |

Second input word (High). Besides the main word above, the front end keeps a second input word (front-end input block +0x894). It is the snapshot's raw newly-pressed word (group 1): buttons only, without the stick-derived direction bits, and in the shipped game it is pad slot 0's word. Its bits are the same as the main word's (SPEC-0033): 0x0001 L2, 0x0002 R2, 0x0040 Cross, 0x0100 Select, 0x2000 D-pad Right, 0x8000 D-pad Left (and the other buttons at their main-word bits). So a stick push never sets a direction in the second word. Readers: the pause item states (Cross), the journal and the map (L2, R2), the in-play map and journal shortcuts (D-pad Left and Right), the page stepper (any bit) and Select on the pause page. The item screen reads L2 and R2 from the main word instead; because both words carry the same button bits, the result is the same.

- Analog-to-digital: a left-stick vertical byte below 0x20 sets the Up bit, and above 0xE0 sets the Down bit. A left-stick horizontal byte below 0x20 sets the Left bit, and above 0xE0 sets the Right bit. The thresholds are strict (below 32, above 224), and the bits go into the group-2 words only (held2 and newly pressed2), never into the second input word. Outside analog mode no stick bits are added. High (SPEC-0033 Controller map).
- Physical names: taken from the per-pad read of the raw packet (SPEC-0033, Layout). The bit positions are High; the names follow the standard controller layout (Medium-High). The roles of Start and Select as menu openers are Medium.
- The two menu openers take different paths. Select opens the pause page (state 7, record 2). Start opens the item screen on its current tab, and the state returns one of six item-screen codes (0x3C, 0x50, 0x64, 0x78, 0x8C, 0xA0), chosen by the tab and by the active hero's index (Section 14.4). These codes are entry and tab-change codes, not exit codes. High on the codes and on the screens (records 3 to 8).

#### 4.1 Menu cursor step (generic menus)

Confidence: High.

- The step takes a mode code and the menu's list of items. The mode codes are 0x4000 (Down) and 0x1000 (Up), the vertical direction bits of the rising-edge word (section 4). Down moves to the next selectable item and Up to the previous one. On a list page (section H byte +0x0A = 1) Left and Right give no move.
- The generic menu step (Behavior 1b) tests Down, then Up, then Right, then Left; only the first set bit moves. A move that succeeds plays front-end sound slot 0 (Down) or 1 (Up), both cue 0x9A.
- The selectable items are the kind-4 nodes (selectable page items, section 13.1) whose condition chain passes (section 9b). Items of other kinds, and items whose chain fails, are passed over.
- The move wraps. From the last selectable item, the next move goes to the first; from the first, the previous move goes to the last.
- The full focus rules (default focus, list and spatial pages, the focused draw flag) are in section 9d.

Mode meaning: the menu driver uses 0x4000 (Down) and 0x1000 (Up) as the vertical direction bits. The Cross bit is 0x0040 (section 4), so 0x4000 is not a Cross code.

### 5. New game and start points

Confidence: High for the reset and load steps; Medium on the flag and cue table. The label of each index is not confirmed.

When Cross is pressed on the new-game or start-point screen, with a start index from 0 to 8:
1. Play cue 0x9A.
2. Reset the world.
3. Set the start marker to 0x2C3.
4. Load the world from the chosen save slot (or, for index 0, with no slot).
5. Reset the screen.

Index 0 is the plain new game. It clears both chosen-slot values to none (-1) and sets no stage id.

Indices 1 to 8 set stage ids and play one cue:

| Index | Stage ids | Cue | Note |
| --- | --- | --- | --- |
| 1 | 0 | 0x97 | Also runs the one-time start routine with argument 1 |
| 2 | 0, 1 | 0x40 | |
| 3 | 0, 1, 2 | 0xB2 | |
| 4 | 4 | 0x5A | |
| 5 | 4, 5 | 0x5A | |
| 6 | 0, 1, 3 | 0x85 | |
| 7 | 0, 1, 3, 6 | 0x5A | |
| 8 | 0, 1, 2, 7 | 0xB2 | |

Start-point labels (Medium-High; the chooser page has these nine items, in the item order of its page):

| Index (argument of the item) | Label on the chooser |
| --- | --- |
| 0 | Act 1 Start |
| 1 | Act 2 Start |
| 2 | Act 3 Start |
| 3 | Act 4 Start |
| 4 | Act 1 Squire Done |
| 5 | Act 1 Chores Done |
| 6 | Act 3 Middle |
| 7 | Act 3 Entering Dungeons |
| 8 | Act 4 Districts Done |

The chooser is marked debug-only in the front-end text. Its container (page 858, under node 854) is not opened by any known screen push or root setting, and the handler of table B index 27 is attached only to four screens that nothing pushes (records 24, 26, 27 and 28). The container is the top node of section D record 15, which has no state and whose picture is the load-game picture (Behavior 1a). The chooser is never entered: the state return survey (Behavior 2) shows that no state returns record 15, and no section I target, back rule or direct screen set reaches it (High, static; Open question 86, resolved).

None of the nine start points is a boss fight: they set story stage ids and load the world from the chosen slot, and none names a boss pack. A developer start that puts the hero into a boss pack is therefore the rewrite's own addition; it should place the hero at the record the game's own script uses for that fight, listed per boss in SPEC-0062 (the final-fight pack 1432 has the hero at its entry record 0 in the alcove, and the pack's own fight-start step moves it to the hall; SPEC-0054). The title's New Game item does not use this chooser or the start-point state: the title itself runs the index-0 steps (reset the world, cue 0x9A, both chosen-slot values -1, start hero, flag 707, packs 691 and 909, music overrides cleared) and goes straight to play (Behavior 2, state 12).

Lead-in page stepper (table B state 27; High for the mechanism). Records 27, 24, 26 and 28 are four lead-in pages with the Controls, Attack Maneuvers, Quick Inventory and Mote screen pictures; each shows a Cross icon and the caption "Next". The stepper keeps a counter c, 0 at boot and reset to 0 by the attract when it leaves to the title. On a page's pass 0 it clears the movie cancel flag and selects controller port 1; on page passes 3 to 7 it sets the play-active word to 1 (section 2a); from page pass 11 on, any nonzero bit of the second input word (section 4) turns the page: c 0 -> 1 returns base plus 0x1E0 (record 24), 1 -> 2 returns 0x208 (record 26), 2 -> 3 returns 0x230 (record 28), and 3 -> 0 clears the four music overrides and returns 0xB4 (in-game play). On other passes it runs the controller check (section 2b). Entered at record 27 with c = 0, the player would see Controls, Attack Maneuvers, Quick Inventory, Mote screen, then play. No screen, item or code enters record 27 (or 24 to 28), so the pages are never shown; the counter reset in the attract and the exit to play fit a removed "New Game -> lead-in pages -> play" step (Medium). These pages are not start-point steps (Open question 126, resolved). Record 25 (Coming Soon, 16:9) runs the fade step (state 28), has no items and Triangle off, so it would be a dead end; it is not entered either.

### 6. In-game play (state 11)

Confidence: High on the path and the gate order; Medium-High on the identities of the screens.

- Record 9 (code 0xB4, state 11, page node 322) is the in-play HUD screen: its node tree is the HUD of SPEC-0023, drawn once per pass like any front-end page.
- On entry, it runs the element setup and the front-end HUD setup.
- State 11 does not re-arm the HUD timers on each call; the HUD elements fade out after their timers run down until an event re-arms them (SPEC-0023). Only the item-screen tab states 8, 9 and 10 re-arm them every tick (section 14.1).
- Play gates. On each pass in play, before any input is read, the state tests these in order (Open question 94, resolved):
  1. Story bit 368 (the progress-bit table, value 0x170). If it is set, the state sets a credits-pending flag and returns state code 0x118 (record 14, the credits screen). Bit 368 is set by pack 1300's epilogue (movie 83) after the finale: the finale's return sets 366, the first epilogue trigger plays movie 79 and sets 367, the second plays movie 83 and sets 368 (SPEC-0026, Credits; High for the setters, Medium for the player reaching the triggers). The credits end with a world teardown, which clears the bit.
  2. Busy latch. If the game-state word at +0x68 and the word at +0x60 are both nonzero, and the word at +0x6C is zero, set +0x6C to 1. This test does not return.
  3. Book reader gate. If the book cursor (the game-state word at +0x54) is 0 or more, or the saved book word (+0xA8 of the global settings block) is 0 or more, clear the first byte of the shared text buffer and return state code 0x190 (record 20, the book reader, section 6.1). The reader shows the book at index (+0x54 minus 1). This gate does not read the pending book index. In the shipped game the saved word is always -1, so only the cursor opens the reader.
  4. Game over. If flag E (game-state word +0x0C) is nonzero, return state code 0x1CC (record 23, the game-over page). On that pass the state does not draw its menus. It also clears the counter that the game-over page uses (Behavior 2, state 26).
  Step 1 is the credits gate and step 3 is the book reader gate; neither is a pause test or a busy test.
- Play-active word. After the four gates and before the openers, the state sets game-state word +0x04 to 1 on every pass. This keeps the world running on the next pass (section 2a). A pass that leaves through gate 1, 3 or 4 does not set it, so the world is frozen from the next pass.
- Pause and item-screen openers (after the gates; they read the rising-edge word +0x89C). Each opener first tests the busy check: when it is nonzero, the opener does nothing and the state continues. The busy check is true when any of ten world-slot fields is set (the game-state words +0x20 and +0x44, the word at +0x14 when it is above 0, and seven helper tests). Its shape is High; which slots are included is Medium.
  - Select: play cue 0x94, reset the menu and HUD flags, and return state code 0x028 (record 2, the pause page: node 12 with root 14, state 7). The same branch also runs, even without Select, when pad slot 0 is not usable: this is the pad-not-usable auto-pause of SPEC-0033 (Controller map).
  - Start: play cue 0x94, reset the menu and HUD flags, and return the item-screen entry code for the current tab (14.4).
- Box host gate (High). After the openers and the HUD reads, when the overlay box is busy (see below) the state returns code 0x17C (record 19, the box host, state 23) and does not read the map and journal shortcuts on that pass. Test vector: the box word +0x258 set, no gate fired, no button pressed: the pass returns 0x17C; with both box words 0 the pass goes on to the shortcuts.
- Map and journal shortcuts (High). They run only when all of these hold: the overlay box is idle; both quick-panel flags are clear (neither the quick-item panel nor the second quick panel of SPEC-0021 is open); and the in-play controls gate of SPEC-0053 section 3.13 passes (the selected hero's state is not 25, its second hero test is false, and the game-state words +0x20 and +0x5C are both zero). Both read the second input word (section 4). D-pad Left is tested first, so it wins when both are pressed.
  - The quick panels open on D-pad Up and Down (SPEC-0021). While a panel is open the shortcuts are off, so Left and Right go to the panel. The hero update runs before the front end in a tick, so a press that opens a panel turns the shortcuts off in the same tick: Up and Left pressed together open the quick-item panel and do not open the map.
  - D-pad Left (0x8000): play cue 0x9A, set the selected map area to the hero map area (the current world pack's map area id, else the last transition map; SPEC-0051 section 1, SPEC-0006), and return the code of that area's map page, 0x26C + 20 x area (record 31 + area, state 30, picture resource 601 + area). If the hero map area is negative, the cue still plays and the state stays in play.
  - D-pad Right (0x2000): play cue 0x9A and return state code 0x1B8 (record 22, the quest journal).
- Overlay box busy (High): the overlay box (the shared dialogue and message box of SPEC-0022) counts as busy when its current-page text word (+0x258) is set, or else when its next-page word (+0x25C) is set. It is idle only when both are zero. The interaction chooser of SPEC-0022 uses the same test. Test vector: +0x258 = 0 and +0x25C set gives busy; both 0 gives idle.
- Quick slots: the item counts of quick-item slots 0 to 2 are read each pass for the HUD (SPEC-0023 section 5).
- The HUD meter update (SPEC-0023 section 7) runs only while the HUD countdown is nonzero. A block of HUD elements fades with the countdown only while the HUD visibility flag is set. The menu reset clears that flag. The setter is not known.

#### 6.1 Book reader (table B state 24)

Confidence: High for the cursor and index rules, the Cross exit, the no-index exit, the play gate, the Square take (a rising edge), the flag rule of book stations, the writers and initial values of the pending index, the cursor and the saved word, the prompt draw and timing, the request cache, the page driver's call rate, the page timer endpoints and step, and the page layout values, the template rows and the body fill (rules 8 to 10, 13 and 14; Medium-High for the flat page image, the 3D structure and the placement of the page sheet in the camera frame in rule 12). Medium for which actors besides the hero set the pending index.

Terms used here:
- Pending book index: the game-state word at +0x50. -1 means none.
- Book cursor: the game-state word at +0x54. It is 1-based; -1 means none.
- Saved book word: the word at +0xA8 of the global settings block. The block is created on first use with this word set to -1.

Initial values and writers (High):
- The pending index and the cursor are set to -1 once, when the game-state object is created at start-up. Nothing else resets them: world teardown, new game and load leave them as they are.
- The pending index is set only by the book-station handler (Offering a book) and cleared only by the chooser's book step (Taking the book).
- The cursor is written only by the book step (a copy of the pending index on a Square edge, or -1 when there is none) and by the reader (-1). No other front-end handler reads or writes the pending index or the cursor.
- The saved word starts at -1, and its only other writer is the reader exit, which also writes -1. In the shipped game it is therefore always -1.

The book reader is table B state 24 (record 20, node 930). The 88 book entries (titles and pages) are described in SPEC-0024.

Offering a book:
- A book trigger is a kind-3 volume (a finite cylinder in a world pack; SPEC-0028, section 13) whose flag word has bit 6 (0x0040) set and bits 1, 2, 3 and 8 (0x0002, 0x0004, 0x0008 and 0x0100) clear. The flag test is made after the actor's cylinder test passes; on each tick that both pass, the pending book index is set to the volume's book value (a u16 in its volume header). Volumes that fail the flag test do not set it. On the disc every book station has flag word exactly 0x0040 (High).
- No script call and no host call writes the pending index (High).
- Actors (High): each tick, the selected hero runs its update (which includes the interaction chooser) and then its trigger step; then the live petpet runs its update and its trigger step; region characters run their update and their trigger step in the region pass. NPCs and enemies never reach a type-11 volume or a book station. The petpet's fire is refused by the fire step. So only the selected hero sets the pending book index (SPEC-0028, "Trigger pass").
- The disc has 134 book stations in 27 world packs (1 to 17 per pack; pack 777 has 17, pack 1414 has 14 and pack 1327 has 13). All have flag word 0x0040, mode 0, gate mask 0 and trigger index 0. Their values cover 1 to 77 and 79 to 87, giving 86 distinct station values; the reader selects record index value minus 1. All 27 station packs occur in SPEC-0006’s 174-pack static new-game reach closure. That closure is only an upper bound: it does not evaluate story gates or prove the hero can walk to each station on the ground, so call these 86 books station-available in the modeled area set, not confirmed readable in ordinary play. Index 77 has no station (value 78 is absent); index 87 has no station (record 87 exists and duplicates record 86). The values are read from each station’s own volume record on disc; they are not copied here. [High for station survey and graph intersection; Medium-High for actual play reach] Synthetic check: pack 786’s station value 87 opens record index 86; value 78 would select index 77, but is not present.
- A book value is the cursor value. The book index is the value minus 1, so the indices in play lie in 0 to 86.

Prompt (world action step, on each tick that its prompt branch runs):
- While the pending book index is 0 or more and the book cursor is negative, the label "Read Book" (label 31 of text table 13, a player-visible label) is shown on each tick. The label is not on any interaction-list entry; the step shows it directly (High).
- Draw (High): "Read Book" is the Square caption of SPEC-0022 (Interaction prompts): slot 4 at (327, 395), the default caption colour, with the Square icon drawn in the same pass. It is not drawn while a quick panel is open.
- The prompt makes no camera change (Medium).

Taking the book:
- The book step is the "Read Book" entry of the interaction chooser order (SPEC-0022). It runs only when all of these hold: hero input is enabled; the hero is not occupied; game-state word +0x5C is not above 0; no buyable item or talk target applies; no earlier chooser entry matches; and no caption other than "Feed petpet" was set earlier in the pass. So it does not run on every tick in play.
- Tick order (High): the book step runs inside the active hero's update, before that hero's trigger step in the same tick, so it reads the pending value set on the previous tick. The prompt first shows one tick after the hero enters a station, and shows one last time on the tick after the hero leaves.
- On every tick that the prompt branch runs, the pending book index is set to -1, whether or not Square is pressed. While the hero stays inside the station, the hero's trigger step sets it again later in the same tick, so the prompt keeps showing.
- Only on a rising edge of the Square bit (0x0080) in the pressed (rising-edge) word of the hero's input block (+0x154; the held word is at +0x158): the pending index is copied to the book cursor and cue 0xAF plays, before the clear above (High).
- The reader opens on the next play pass. The play gate (Behavior 6, step 3) returns code 0x190 while the book cursor is 0 or more, or the saved book word is 0 or more.

Reader rules:
1. Index. Index = book cursor minus 1. If the index is negative, index = (saved book word) minus 1. If that is still negative, leave the reader: set the book cursor to -1, clear the four music overrides and return code 0x0B4, with no cue. No book is unloaded and the saved word is not written.
2. Title. The book's title is copied, up to 255 bytes, into the shared front-end text buffer.
3. Request and waiting. The reader requests the book at the index. The book load keeps one slot (SPEC-0024, Book index): a request for the index that is already loaded and finished does nothing, so there is no reload and the page timer is not reset. The timer is set to its start value only when a new load starts. While the book is not loaded, the reader waits. The exit in rule 6 still applies.
4. Page driver. The page driver is called once per frame-loop pass while a book is loaded, whichever screen is current (also on the Triangle bounce pass of rule 7). Each call draws the flat page image and the book's 3D parts first (rules 11 and 12) and then lowers the page timer by a step. The timer is the page angle in radians. Its sine and cosine build one rotation about the model's x axis that is applied to both 3D page parts (rule 12). It is only a tilt animation of the book model: a book has a single spread and no page is ever turned (rule 8). The timer starts at 1.2 (the float at ELF data address 0x2DB2E0) and falls by the step on each call, clamped at a minimum of 0.12 (the float at 0x2DB2E4). The step is computed at book setup from these two words: step = (1.2 - 0.12) x 1/75 = 0.0144 per call (as a single-precision float, about 0.0144000014). The timer therefore reaches 0.12 after 75 passes (1.25 s at 60 passes per second). High.
5. Page text template. The body string of a book (SPEC-0024, Book records) is laid out into two text blocks, A and B, whose x, y and line limits come from one row of a 23-row template (rule 9). Text width is 192 px (the integer at ELF data address 0x2DB2D4). The line pitch is 18 px (the layout metrics pitch field, 18 in all 88 books) at scale 1.0 (the float at 0x2DB2D8). Block x values are 20 and 234 (the words at 0x2DB2B4 and 0x2DB2B8), block y values 54, 256 and 300 (0x2DB2BC, 0x2DB2C0 and 0x2DB2C4) and line limits 19, 7 and 5 (0x2DB2C8, 0x2DB2CC and 0x2DB2D0). Justification is rule 10. High.
6. Exit (Cross, bit 0x0040 in the front-end rising-edge word at +0x89C of the front-end object; SPEC-0033 calls it the menu rising-edge word). Plays cue 0x9A, sets the book cursor to -1, unloads the book, sets the saved book word to -1, and returns code 0x0B4 (in play, record 9).
7. Triangle (quirk, reproduce; Medium-High). The reader does not handle Triangle, so the generic back rule applies (record 20's back rule is -1): cue 0x99 plays and the stack pops to play. The book cursor is still 0 or more, so play's book gate opens the reader again on the next pass. The book stays loaded. The player sees one play pass and then the book again; only Cross leaves the reader. On the bounce pass play runs its pass-0 entry (HUD setup, stream gate opened), then its book gate returns 0x190 before it sets the play-active word, so the world stays frozen. The reader then requests the same index, which the one-slot cache ignores (rule 3): the book is not reloaded, the page angle carries on from its current value, and the title is copied again. Cross is read by the reader itself, so it works from the reader's first loaded pass; Triangle works only from the reader's pass 2, through the generic step (High).
8. Spreads and page turning (High). A book record holds one body string, so a book has exactly one spread: the left page is block A and the right page is block B. Nothing turns a page or loads a second spread: the reader handles only Cross (rule 6), and the page timer of rule 4 drives only the tilt of the 3D parts. The record's two textures, shader list and two 3D meshes are all the page data there is (SPEC-0024).
9. Template row (High for the values). The style index of the book (SPEC-0024, text chunk word +0x04; 0 to 21 on disc, 17 of the 23 rows used) is the row number. A row gives block A (x / y / line limit) and block B (x / y / line limit); a limit of 0 means the block is unused. Line j of a block (j from 0) is drawn at x = the block x and y = block y + 18 j, in the same convention as the other labels (the y is the baseline of the line; SPEC-0022). Rows, written A x/y/limit ; B x/y/limit: 0: 20/54/19 ; 234/54/19. 1: 20/256/7 ; 234/54/19. 2: 20/54/19 ; 234/256/7. 3: 20/54/7 ; 234/54/19. 4: 20/54/19 ; 234/54/7. 5: 20/300/5 ; 234/300/5. 6: 234/54/19 ; unused. 7: 20/54/19 ; unused. 8: 20/256/7 ; 234/256/7. 9: 20/54/7 ; 234/54/7. 10: 20/256/7 ; 234/54/7. 11: 20/54/7 ; 234/256/7. 12: 20/256/7 ; 234/54/19. 13: 20/54/19 ; 234/256/7. 14: 20/54/7 ; 234/54/19. 15: 20/54/19 ; 234/54/7. 16: 234/54/19 ; unused. 17: 20/54/19 ; unused. 18: 20/256/7 ; 234/256/7. 19: 20/54/7 ; 234/54/7. 20: 20/256/7 ; 234/54/7. 21: 20/54/7 ; 234/256/7. 22: 20/54/5 ; 234/54/5. Rows 6 and 16 put the whole text on the right page and rows 7 and 17 on the left page. The pairing of a y value with a limit is given per row; there is no general rule.
10. Filling the blocks and breaking lines (High for the rule; Medium-High for the glyph advance reading). The body is one string ended by a zero byte. A newline byte ends a line, and two newline bytes in a row leave one blank line (the paragraph gaps of the books are made this way). Block A is filled first: lines are taken from the start of the string until the block has drawn its limit or the text ends. Block B then continues with the next line of the same string (the lines drawn in A are skipped), up to its own limit. A blank line counts as a line. Lines that do not fit are not drawn. Line breaking: starting at the line's first byte, each byte adds the advance of its glyph (the advance field of the glyph record that the layout's code map gives for the byte; a byte with no glyph adds nothing and draws nothing; whole pixels, scale 1.0). A space or an underscore is a break candidate, remembered with the width before it. When the width exceeds 192 (strictly more), the line ends at the last candidate and the candidate byte is consumed. When the line has no candidate, it ends before the byte that overflowed, and that byte is consumed and lost. Justification: a line that ended at a candidate and has at least one candidate inside it is stretched. With L = 192 minus the line width and n = the number of bytes in the line before its break byte (spaces included), every glyph's advance gets e = min(L / n, 2.0) pixels added; then, if L - e n is not zero, each space gets a further min((L - e n) / k, 12.5) pixels, k being the number of candidates inside the line and 12.5 being 2.5 times the space advance (5). Lines that ended at a newline or at the end of the text, and lines with no candidate, are not stretched. The text colour is (100, 100, 100, 128). The text is never centred.
11. The flat page image (Medium-High). Each driver call first redraws the flat page into an off-screen 512 x 512 target in 24-bit colour, in canvas coordinates with the origin at the top left: one paper quad from (0, 0) to (448, 448) at the far depth, textured with the whole 128 x 128 paper picture (texture coordinates 0 to 128 texels) in colour (128, 128, 128, 128), then the two text blocks in the book's own glyph sheet (a 256 x 256 picture with 4 bits per pixel). The body therefore lies at x 20 to 212 (left page) and 234 to 426 (right page) of a 448-wide spread whose fold is at x 224. The rest of the 512 x 512 target is never shown.
12. The 3D parts and the transforms (Medium-High for the structure and the placement arithmetic; Medium-High that the camera's second row is up, so the sheet rises from below; Medium for the sideways sign). After the flat image the driver draws two meshes of the record's model chunk: the page sheet and the decoration mesh. The page sheet is one slightly curved sheet 1.70 wide (x from -0.85 to +0.85) and 1.12 high (y from 0.03 to 1.15) in model units, bent in depth by about 0.1 (z from 0.94 to 1.04). Its texture coordinates run from 0 to about 0.872 (448 / 512 with a small inset) in both directions, so it shows exactly the 448 x 448 flat image, and its vertex colours are (102, 102, 102, 128). The decoration mesh holds the book's 3D pictures as 20 switch parts: one with mask 0 (always drawn) and 19 with the masks 2, 4, 8, 16, 32, 64, 128, 256, 512, 1024, 2048, 4096, 8192, 16384, 32768, 65536, 131072 and 262144. A part with a nonzero mask is drawn only when its mask and the style mask (rule 14) share a bit. Both meshes are drawn with the same transform. With t the page angle of rule 4, C = cos t and S = sin t, a model point (x, y, z) goes to (x, C y - S z, S y + C z): a rotation about the model's x axis (at t = 1.2 the sheet is tilted far back, at t = 0.12 it stands almost upright facing the viewer). The result is scaled by 1.7 (a uniform scale of the three rotated axes, no other scale) and placed in the game camera's own frame, so the sheet stays fixed on screen whatever the camera does that frame. Call the camera's rows sideways, up and forward (SPEC-0027). The rotated point (x, y', z') with y' = C y - S z and z' = S y + C z has the camera-frame position X = 1.7 x, Y = 1.7 y' - 0.75 and Z = 1.7 z' + 1.0, meaning X along the sideways row, Y along the up row and Z along the forward row. The model origin therefore lies 1.0 ahead of the camera and 0.75 below the view axis, with no sideways offset. The four setup numbers 0.5, 1.3, 1.0 and 1.7 are the screen fraction of the origin (0.5 of the width, 1.3 of the height measured from the top), the depth 1.0 and the scale 1.7; the offsets come from them as (0.5 - 0.5) x 640 / 512 = 0 sideways and (1.3 - 0.5) x 480 / 512 = 0.75 downward, divided by the camera's current focal length over 512, so the origin always projects to the fraction (0.5, 1.3) of the viewport (pixel (256, 582.4) on the 512 x 448 screen, below its bottom edge) at any focal length. Screen mapping with the play projection (focal 512, 512 x 448 viewport, SPEC-0046): px = 256 + 409.6 X / Z and py = 224 - 477.87 Y / Z. The sheet's outer corners in model space are (-0.85, 0.030, 1.015) and (0.85, 0.030, 1.015) at the bottom and (-0.85, 1.152, 1.043) and (0.85, 1.152, 1.043) at the top; the middle of the bottom edge is (0, 0.030, 1.000) and the middle of the top edge (0, 1.152, 1.028), and a rim of vertices at x = +-0.771 lies about 0.07 behind the corners (z 0.943 at the bottom, 0.971 at the top). Worked example (corner pixels, x then y, and camera depth Z): at timer 1.2 (C = 0.3624, S = 0.9320) the bottom corners are (-98, 893) and (610, 893) at Z 1.67 and the top corners (85, 457) and (427, 457) at Z 3.47, so the whole sheet lies below the 448-line screen (the top edge 9 pixels under it) and nothing shows; at timer 0.6 (C = 0.8253, S = 0.5646) the bottom edge is at y 552 and the top edge at y 242; at timer 0.12 (C = 0.9928, S = 0.1197) the bottom corners are (38, 383) and (474, 383) at Z 2.72 and the top corners (58, 67) and (454, 67) at Z 3.00, a sheet 436 pixels wide at the bottom, 396 at the top and 316 tall, centred on x = 256. The timer falls 0.0144 per draw (rule 4): the top edge first reaches the screen on the third draw (timer 1.1712, top edge at y 443), and the sheet is at rest from draw 76 on. The sheet rises from below the screen to this rest pose. The screen x of the model's negative x edge (left or right of 256) is Medium (the set of corner positions is symmetric about 256, so it does not change the numbers). The paper and the text do not move with the angle. The driver also turns off the test setting (alpha or depth test) in both state blocks of every material of the book.13. The title (High). The title is the value-text node 936 of the reader's page tree (node 930). Its variable is the shared title buffer, its font is front-end font 0 (line height 21), its letter spacing is 0 and its colour is (112, 112, 2, 128). It sits at (256, 40) with centre alignment: the text starts at x = trunc(256 - W / 2) with W the measured width of the title, and its baseline is y 40. The page tree also holds the Cross hint "Close": a 32 x 32 Cross sprite at (391, 399) with its caption in font 1 at (421, 423). The paper and the body text use the book's own glyph descriptor and sheet; only the title uses a front-end font.
14. What the style index sets (High). It selects the template row of rule 9 and the style mask of rule 12, and nothing else (no font, no colour). The style mask is the word of the 23-word table at ELF data address 0x312D88 that the index picks: 0, 2, 4, 8, 16, 32, 64, 128, 258, 520, 514, 264, 1024, 2048, 4096, 8192, 16384, 32768, 66560, 135168, 132096, 69632, 262144 (rows 0 to 22).

Edge cases:
- Book cursor 0 gives index -1. With the saved word at -1 or 0, the fallback is still negative, so the reader leaves at once. Only a saved word of 1 or more lets the fallback open a book (next item).
- The saved word starts at -1 and is written only by the exit, and only to -1. So in the shipped game it is always -1: neither the fallback nor the saved-word branch of the play gate ever fires.
- Clean-side initialisation (quirk to avoid): the saved word must start at -1. With 0, play's gate would open the reader, the reader would find no index and return to play, and this would repeat every pass, forever, with the world frozen.
- The reader has no upper bound check on the index. Station values on disc give indices 0 to 86 only (High).
- Stale pending (quirk, reproduce). If the hero leaves a station on a tick when the book step does not run (for example while occupied by a jump), the pending index keeps the station's value. On the next tick the step runs (possibly in another area, since the pending index survives area changes), "Read Book" shows for that one tick, the pending index is cleared, and a Square edge on that tick opens the book.
- A volume whose flag word fails the rule (bit 6 set; bits 1, 2, 3 and 8 clear) does not set the pending index, even when the hero is inside it. A flag word of 0x0041 would pass and 0x0048 would not; on the disc the book stations all have exactly 0x0040.
- The pending index is cleared on each tick that the prompt branch runs. When the hero leaves the station, the trigger no longer sets it again, so the prompt stops one tick later.
- Overflowing text (Medium): by the wrap rule of rule 10, book 1 (style 14, capacity 7 + 19 = 26 lines) needs 32 lines and book 50 (style 7, capacity 19) needs 20, so their last lines are not drawn. Every other book fits.
- A line with no break candidate that is wider than 192 loses the byte at which it overflowed (rule 10); no disc book has such a line.

Test vectors (synthetic, not disc values):
- A volume with flag word 0x0040 and book value 82, with the hero inside it: pending index = 82, and "Read Book" is shown on each tick; the prompt branch sets pending to -1 and the trigger step sets it to 82 again later in the same tick. A rising Square edge sets the cursor to 82 and plays cue 0xAF, and pending is set to -1. The next play pass returns 0x190, and the reader shows index 81.
- The same volume without a Square press: the cursor stays at -1 and the reader does not open.
- A volume with flag word 0x0140 (bit 8 set) and book value 82, with the hero inside it: the pending index is not set.
- Page timer: 1.2 at setup; 1.1856 after 1 driver call; 0.12 after 75 calls; still 0.12 after 76 or more calls.
- Cursor 0 with the saved word at -1: index -1, fallback index -2, so the cursor is set to -1 and the reader leaves at once.
- In the reader, a Cross press: cue 0x9A, cursor -1, saved word -1, return code 0x0B4.
- Prompt timing: the hero enters a value-82 station on tick N: pending is 82 after tick N's trigger step, and "Read Book" first shows on tick N+1. The hero is last inside on tick M-1 and outside on tick M: the prompt still shows on tick M, and not on tick M+1.
- Petpet in a station: the live petpet stands inside a value-82 station with the hero 10 units away and free: "Read Book" shows, and a Square edge opens book index 81.
- Saved word 0 and cursor -1: play returns 0x190, the reader returns 0x0B4 with no cue, and this repeats every pass.
- Triangle re-entry: the reader is open on book index 81 for 100 passes (angle 0.12); Triangle on reader pass 2 or later: cue 0x99, one play pass, then the reader again with the angle still 0.12 (no reset, no reload).
- Worked example, disc book index 0 (the text is not copied here). Style index 12, so template row 12: block A x 20, y 256, limit 7; block B x 234, y 54, limit 19. The body is 451 bytes with 4 newline bytes and wraps at width 192 into 24 lines, widths 154, 188, 180, 159, 175, 188, 167, 177, 166, 146, 178, 146 (ends at a newline), 0 (blank), 134, 183, 188, 149, 149, 120 (ends at a newline), 0 (blank), 170, 158, 159, 166. Lines 0 to 6 are drawn in block A at x 20 and y 256, 274, 292, 310, 328, 346, 364. Lines 7 to 23 (17 lines) are drawn in block B at x 234 and y 54 + 18 j for j from 0 to 16 (54 to 342). All 24 lines fit in 7 + 19 = 26. The title has 29 bytes and is centred at x 256 on baseline 40 in font 0 (colour 112, 112, 2, 128). Style mask 1024 (row 12 of the mask table): the decoration part with mask 1024 and the part with mask 0 are drawn, and no other.
- Justification, the same book: line 0 has 17 bytes before its break byte, width 154 and 4 spaces inside: L = 38, e = min(38 / 17, 2.0) = 2.0, the remainder 38 - 2.0 x 17 = 4 gives each of the 4 spaces 1.0, so the line is exactly 192 wide. Line 1 has 24 bytes and width 188: e = 4 / 24 = 0.1667 and no space extra. Line 11 ends at a newline and is not stretched.
- Break rule: a 192-wide block where the running width passes 192 on a byte that follows a space at width 150: the line ends at that space (width 150) and the space is consumed; the next line starts at the byte after the space.
- Page timer and transform: t = 1.2 gives C = 0.3624, S = 0.9320, so the point (0, 1, 0) goes to (0, 0.3624, 0.9320); t = 0.12 gives C = 0.9928, S = 0.1197, so (0, 1, 0) goes to (0, 0.9928, 0.1197).

### 7. Save and load pickers

#### 7.1 Picker rows and cursor

- Slots. The save buffer holds four records of 0x3100 bytes each, one per slot, in order 0 to 3. A record is empty when its u32 key at +0x24 is exactly -1; the load starting-cursor search uses the same test (High).
- Rows. The picker draws four rows, one per slot. All four rows use the same drawing rule, so no label is needed for any row (High).
  - Empty slot: line 1 is the word "Empty" and line 2 is blank. The word is a fixed string in the program data, with a zero terminator. It is held at data addresses 0x315B00 and 0x3163B0; these are the only two copies of that string in the boot program (checked on disc). The length value used for the row is 5 + 2 = 7.
  - Filled slot: line 1 is the location name at +0x2F6F (59 bytes). Line 2 is the date text at +0x2FAB (16 bytes). The length value used for the row is the byte at +0x2FBA plus 2.
- Stacking. Rows stack vertically. Each row starts where the previous row ended, using the height the row returns. The row whose index equals the cursor is drawn highlighted. The rows are drawn by the journal's row drawer (section 14.3): row type = act + 2 (types 3 to 6 for Act 1 to Act 4; the act byte is read as a signed byte) for a filled slot and 7 for an empty slot, with bar colour (78, 119, 174) (High for the type rule; Medium-High for the colour).
- Labels. The front-end text has "Game 1", "Game 2" and "Game 3" on page 896. No code opens page 896, and the picker never draws these labels; they are a leftover, not slot names (High). The fourth row is slot index 3, the fourth 0x3100-byte record (High).
- Cursor. The cursor is an index from 0 to 3. The one exception is the load screen's starting value 4 when no slot is filled (section 7.2). Up (0x1000) moves to the previous slot and Down (0x4000) to the next; Down is tested first. Every Up press plays front-end menu sound 1 and every Down press sound 0 (both cue 0x9A), even when the cursor cannot move (High).
  - Load mode (the load screen): the cursor skips empty slots. A move that lands on an empty slot continues in the same direction to the next filled slot. If no filled slot lies in that direction, the cursor stays where it is. A move that would leave the range 0 to 3 does nothing (Medium).
  - Save mode (the save screen): a move changes the cursor by one, bounded to 0 to 3, with no skipping.
  - Cursor test vectors are in the Test vectors section.
- Confidence: High for the row text rules, the stacking, the cursor rule and the move sounds. Medium for the bound rule in load mode.
- Count: four slots (High). The fourth slot needs no label; see Open question 9.

#### 7.2 Load

- Load starting cursor (Medium-High): slot 0 if it is filled; otherwise the first filled slot among 1 to 3; if no slot is filled, the cursor is 4, which is outside the slot range. Cross with cursor 4 copies nothing (the save-game spec's rule for a slot index of 4 or more).
- If no saved game exists, the load screen returns without loading (High).
- States (High): 0 = start the load scan (SPEC-0025, Card jobs); 1 = wait for the scan, which shows its own card messages as needed (busy, no card, no save data, load failed, cancel confirm); when it ends cancelled, or without a loaded file, the screen is left with return code -1 and no load. 2 = show the picker with the load prompt; the card is checked every pass, and a removed or swapped card starts the load card-gone job (state 3): Retry, or No in the cancel confirm, scans again (state 0); Yes in the cancel confirm leaves with -1. In state 2, Cross loads the chosen slot (it starts the copy-out), copies the slot into the session snapshot and sets the pending flag (the same snapshot and flag as the save, section 7.3); the load then returns to play through the world load. The wait in state 1 has no fixed pass count; it lasts until the card job reports done (Medium).
- Leaving without a load (High). Stages 0 and 1 (card read and card box) return -4, so Triangle does nothing there. When the card job ends with a cancel answer in the card box, or with no save data, the state returns -1: back to the screen below (the pause page or the title), with no driver cue. In stage 2 (the picker) the state itself reads no Triangle and returns 0, so Triangle leaves by the generic back rule (record 17's back rule is -1) to the screen below, with cue 0x99, and the focus there is its default (section 9d). The card messages (Cancel load, then Yes) are the other way out.

#### 7.3 Save

- Opening (High). The save screen is opened only at a save point (SPEC-0006 kind 4; the per-tick rule is in SPEC-0025). When the player presses Square within 5 units of a save point in normal play with the hero not busy, the world's character-region update records the hero's restart point and then calls the front end's set-screen routine with record 18. No return code is involved, so no handler returns 0x168, and no menu item (section I), handler or level host call opens record 18. The opening also:
  - resets the save stage and the slot cursor to 0, so the picker starts on slot 0 (shown as the first row);
  - sets front-end text variable 0 to the text "1" (the card slot named in the card messages) and text variable 1 to the decimal text of the card layer's needed space, 106 (KB) (High). The card layer computes it once from the file sizes: 3, plus each data file rounded up to whole KB (the 50,176-byte save file gives 49 and the 100-byte support file gives 1), plus the icon file rounded up (49,858 bytes gives 49; the three icon entries name the same file, so it counts once), plus (data files + distinct icon files + 2) / 2 rounded down (here (2 + 1 + 2) / 2 = 2), plus 2: 3 + 49 + 1 + 49 + 2 + 2 = 106. Test vector: with three different icon files of 49,858 bytes each the same rule would give 3 + 49 + 1 + 49 + 49 + 49 + 3 + 2 = 205;
  - sets the screen's pass counter to 0 and runs the enter callbacks (Behavior 1).
  Because the front end runs after the world update in the frame loop, the screen's first pass runs in the same tick as the Square press (High).
- Each pass (High): the selected hero is set to state 1 (idle), so the hero is held idle while the screen is open.
- States: 0 = start the save scan (SPEC-0025, Card jobs; it may offer to format an unformatted card); 1 = run the card message box until it is idle (a cancel confirmed in the scan returns to play with no cue and nothing written); 2 = show the picker with the save prompt. While the picker is up the save screen checks the card every pass; a removed or swapped card starts the save card-gone job (SPEC-0025). In state 2, Cross rebuilds the chosen slot, copies it into the session snapshot, sets the pending flag and starts the write job, and the screen moves to state 3. The write job asks before writing: the overwrite question when the chosen slot held a game, the "save to MEMORY CARD slot 1?" question when it was empty, and none on a card formatted in this scan (SPEC-0025). Triangle (the back bit) in state 2 plays front-end menu sound 6 (cue 0x99), clears the music overrides and returns to play with code 0x0B4 without writing (High). The save cursor moves one slot at a time, bounded to 0 to 3 (section 7.1, High).
- In state 3 (or the error state 4) the screen runs the message box until it is idle, then reads the card state word:
  - 0 ("saved", or "cancel confirmed" inside the write job): in state 3, cue 0xA9 (save complete) plays; the music overrides are cleared; the screen returns code 0x0B4 (back to play).
  - nonzero ("not saved": No to the question, or Retry after a failure): the stage is set to 0 and the screen stays, so the next pass starts a new card scan and the picker shows again. Retry never rewrites at once; the player picks the slot again.
  - Quirk (reproduce or document): "Cancel save?" answered Yes inside the write job ends it with the "saved" word, so cue 0xA9 plays and the screen returns to play although nothing was written. No "Save Successful" message is ever shown.
  Confidence: High (Open question 122, resolved).
- Pending flag and snapshot (High). Confirming a slot in the save picker or in the load picker copies that slot (0x3100 bytes) into a session snapshot buffer and sets the pending flag. The save sets it when the write starts, before the card result is known, so a failed write still leaves the snapshot and the flag (Medium). Only the world teardown (quit, or game over without the flag) clears the flag. The game-over page uses it (Behavior 2, state 26; SPEC-0055).
- Each card operation waits for the card job to finish in its own state before showing the result (Medium). The save and load states have no frame counter: the length of each wait is the card driver's latency, not a fixed number of passes (Medium).
- Entry: the save prompt is record 18 (state 22), entered by the direct screen set above; its code value 0x168 (record 18 x 20) is not returned by any handler. The load prompt is record 17 (code 0x154, state 21).

### 8. Menu button blocking

A helper sets a block flag only when the current node's word at +0x0E is 0. Otherwise it does nothing (Medium). The block is meant for the first cycle of a menu.

The input block of Behavior 1b (step 1): the pause page sets it while its quit box is open; while it is set, focus states and the generic step see zero menu words (High). Independently of any flag, the generic step ignores input on the first two passes of every screen (Behavior 1b, step 7).

### 9. Screen stack (31 deep)

- Reset: the stack is set to hold only the root screen, and the depth is 0.
- Push: if the depth is below 31, the depth goes up by 1 and the new screen is stored above the previous top. A push at depth 31 is refused and the stack is unchanged.
- Pop: if the depth is 0, nothing is popped and the call returns no screen. Otherwise the depth goes down by 1 and the stored screen is returned.
- The stack therefore holds the base entry plus up to 31 entries above it (32 slots). The top entry is the current screen.
- Refused push: the refused push also writes a debug report that says the menu structure is too deep and gives the number 32 (the slot count, not the 31 pushes allowed). The stack is unchanged. The report's text is data in the boot program; only its meaning is described here (Medium).

Parent-index rule (High). When a page is opened, the page record's parent index (signed 16-bit at record offset +0x04, the back and stack rule of section D) decides how the stack changes. This is a stack update. It frees nothing.
- -1: the page is pushed, unless it is already the top entry. In that case nothing changes.
- -2: the stack is reset so that it holds only this page (depth 0). The page is not pushed on top of anything.
- -3: nothing changes.
- Any other negative value: a message is written to the log, and the current page block (the page the front end holds as current) is pushed. The page itself is not pushed.
- 0 or more: the page record for that index is looked up on the stack from the top. If it is found, the stack is cut back to that entry. If it is not found, the stack is reset to hold that entry alone. Either way the page is then pushed.
- Order: when a child page is opened, this update runs first, and then the page-open step of section 17.4 runs.
- A direct screen set (record 0 at front-end start-up, record 18 at a save point) resets the stack to that record and sets its default focus.

Back rule (High). Triangle in the generic step (Behavior 1b), or a Cross target of -1, backs out as follows (a handler's own -1 return pops the stack and takes the new top, Behavior 1b step 8):
1. Triangle backs out only when the current record's +0x04 is not -3 and not -2. On records with -3 or -2 the generic Triangle does nothing (a state may still handle Triangle itself, as the item screen and the save picker do).
2. The stack is popped and the new top becomes the target. If the stack held only its base entry, the target is the record named by +0x04, when that is 0 or more.
3. The target is entered by the normal rule, so its own +0x04 decides the stack change; a -1 record that is already the top is not pushed again.
4. Entering a record whose +0x04 is 0 or more first cuts the stack back to that record, or resets the stack to it when it is not on the stack, and then pushes the new record. So every record with +0x04 = 9 (the quest journal, record 22, and the 77 map pages) has play (record 9) directly below it. Triangle on the journal or a map page therefore always returns to play, also when the page was opened from the pause page (the pause page is cut off the stack when the journal or map opens).
5. Triangle on the pause page (record 2, +0x04 = -1, pushed over play) pops back to play with cue 0x99.
6. A generic Triangle plays sound slot 6 (cue 0x99). A handler's own -1 return plays no driver cue.

### 9a. Record stack with an overflow report

A second fixed stack, separate from the screen stack of section 9, holds state records. Its slots are 16 words (0x40 bytes) each, and each record starts four words into its slot. The stack object keeps a depth count and a pointer to the top record. The drawing path (Section 12) uses one such stack object: each drawn node pushes a frame onto it before its sprite is laid out, and the frame is popped after the node is drawn.

- Push: while the depth is below 32, the depth goes up by 1 and the new top record is a copy of the previous top record (its 16 words). So 32 pushes succeed from an empty stack.
- Push at depth 32: nothing changes, and an overflow report is written through the debug report routine (its fixed text is not copied here).
- The pop used by the drawing path is a separate call made after the node is drawn. Its body, and the pops of the menu code, are not described here (Open question 73).

Confidence: High for the limit of 32; Medium for the copy rule and the record offset; Medium for the report text, which is a boot-program string.

### 9b. Node conditions, search and key masks

Confidence: High for the mode table, the child-first order and the key masks. Medium for the kind rules of the search, which are partly described (Open questions 77 and 78).

Condition modes. Each node has a condition mode byte (+0x2C). Given the node's predicate value P and the caller's flag F, the node passes as follows:
- 0: passes.
- 1: passes with the value F (the caller's flag). The chain tests used by the front end pass F through unchanged.
- 2: passes when P is nonzero.
- 3: passes when P is zero.
- Any value above 3: passes.

Predicates. A node's predicate is a handler selected by the node's handler index (signed 16-bit at +0x24). The handler is called with the node's +0x2F byte. The two callers use the predicate in two ways:
- Condition chain (below): P comes from the nearest handler on the node's parent chain, starting with the node itself. A node with no handler of its own inherits the nearest ancestor handler; if no node on the way up has one, P = 1.
- Drawing, the condition chain, and the kind 1 scan use the same nearest-handler rule: start with the node itself and walk up its parents to the first nonnegative handler index. If no node on the way up has one, P = 1.

Condition chain. A node's condition is the AND of the mode tests along its parent chain, starting at the node and going up, and stopping at the first test that fails. The chain for a gate (kind 3) includes the gate itself. The chain for a selectable page item (kind 4) starts at the item's parent, so the item's own mode is not tested.

Search order. A search over the node tree visits a node, then its children (depth first), then its siblings. Node kinds in a search:
- Kind 1 (runnable entry): returned when its own mode test passes, using the predicate of the nearest handler as above. Mode 0, 1 and values above 3 pass; mode 2 and mode 3 test the predicate.
- Kind 3 (gate): ends the search of the sibling list that holds it. The gate's own children and the later siblings of the gate are not searched. The search then continues with the next sibling of the gate's parent.
- Kind 4 (selectable page item): returned when its chain (from its parent) passes. Its children are not searched.
- Kind 2: not returned as a runnable search result. The search walks through it to its children without testing its own condition; its condition gates drawing, separately.
- Other kinds: searched through their children.
A kind 1 node also has a data check in the runnable scan: through its sprite and asset links, the scan tests the signed word at asset-record +0x08. A negative value identifies an unloaded full-screen picture in the associated loading path; that path replaces it with a runtime picture-buffer slot before loading the resource selected by asset-record +0x0C. For picture records, +0x0C is a resource selector: the requested resource number is 575 plus the stored value. Resource 532 has 797 linked kind-1 nodes: 695 point to asset records with +0x08 = 2 and +0x0C < 0, while 102 point to records with +0x08 = -1 and +0x0C >= 0. A separate kind-1 scan tests +0x0C < 0 instead; do not treat the two tests as one rule.

Key search. A search for a 16-bit key compares it with the node's key at +0x28. The node is checked first. If it does not match, its children are searched, and then its siblings. The first match is returned, or none. A key of 0xFFFF matches every node whose key field is -1, which is the unkeyed nodes (Medium; Edge cases).

Key masks. A node may hold a list of keyed entries: its animation slots (section N, starting at the node's +0x10 word). Each entry has a 5-bit key (its trigger bit number). Given a 32-bit mask, each entry whose bit is set in the mask is changed as follows:
- The entry is marked active.
- Its remaining repeats are set to its repeat count.
- Its start time is set to the clock: the front-end pass counter (the word at 0x2EC8C4, plus 1 per front-end pass) divided by the tick rate (the word at 0x2EC8D4). The tick rate is 60 on disc and is overwritten with the run-time tick rate (60) at front-end start. The value is in seconds (High).
- The repeat counters of its track and of the track's child chain (section M) are reset.
Entries whose bits are clear are not changed. The node itself is marked active when the list is not empty.

Test vectors (synthetic):
- A chain with a child whose mode is 2 and whose handler gives P = 0, under a parent whose mode is 0: the child's test fails, so the chain fails.
- A node with no handler and mode 3: P = 1, so the mode 3 test fails. The chain fails.
- A key search for 7 over a parent with key 3 and a child with key 7: the parent does not match, its child is searched and matches, so the child is returned.
- A key mask with bits 1 and 3 set, and entries keyed 1, 2 and 3: entries 1 and 3 change, and entry 2 does not.

### 9c. Predicate table (table A, 74 slots)

The count of 74 is Medium: it is inferred from the spacing to table B, and the data does not contradict it. A predicate slot is chosen by a node's handler index (section 9b). Each slot returns a value that the node's condition mode tests (section 9b). Every slot is described below; confidence is High unless marked. Slots 13 to 20 serve the item screen; slots 21 to 65 serve the in-play HUD (SPEC-0023). "Marker" is the active hero's quick-item selection marker (SPEC-0021); "quick-panel entry" is an entry of the in-play quick-item panel (SPEC-0021).

| Slot | Result | Meaning | Confidence |
| --- | --- | --- | --- |
| 0 | always 1 | Load Game on the pause and title pages (never hidden). | High |
| 2, 56 | always 1 | No use identified. | High |
| 12 | always 0 | Library gate: keeps groups out of the automatic page draw (below). | High |
| 27, 39, 40 | always 0 | No use identified. | High |
| 1 | 1 or 0 | Items tab only: the marker equals the grid id of the argument cell (the "equipped" frame). | High |
| 3 | 1 or 0 | The argument equals the grid cursor cell (cursor frame). | High |
| 4 | count | Items tab: the owned count of the grid item of the argument cell (0 on the other tabs). The cell icon, the count number and the potency badges show only while it is above 0. | High |
| 5, 7 | 1 or 0 | The selected entry is usable: it exists, has an item-state record, and its category is not 7. | High |
| 6 | 1 or 0 | The marker equals the selected entry's id: picks Activate or Deactivate in the item-action box. | High |
| 8 | 1 or 0 | The hero's category-9 entry exists and is not empty (Items tab). | High |
| 9 | 1 or 0 | Argument 1: 1 when no category-9 entry exists. Other arguments: 1. | High |
| 10 | 1 or 0 | The argument is below the hero's count word +0x15EC (glyph sprites). | High |
| 11 | 1 or 0 | The count of the selected quick-panel entry is above 0 (in-play quick-item box, root 322). | High |
| 13 | 1 or 0 | Equipment icons of the item screen: the hero's category-1, 5 or 2 entry (argument 0, 1 or 2) is present and not empty, and, with focus index 0, the matching equipment word is set, or, with a focus index above 0, a charm is selected. | High |
| 14, 52 | 1 or 0 | The active hero index is 0 (Tor; slot 14) or 1 (Roberta; slot 52). | High |
| 15 | 1 or 0 | The Mote selection equals the argument (255 = none). An argument of 100 or more compares modulo 100, and only while the selection timer is above 0. | High |
| 16 | count | The owned count of Mote 160 + argument. | High |
| 17, 19, 20 | 1 or 0 | The count of the item in equipment word 2 (slot 17), word 0 (slot 19) or word 1 (slot 20) is above 0. | High |
| 18 | 1 or 0 | An icon tween is live for the equipment-slot icon of the argument (0 the weapon slot, equipment word 2; 1 the armour slot, word 0; 2 the shield slot, word 1). It asks the six-slot HUD icon tween pool (SPEC-0023 section 11.1) whether any live tween moves that slot's icon node. It sets nothing. Six sprite nodes use it (arguments 0, 1 and 2, twice each), each shown while the test is true. | Medium-High |
| 21 to 25 | 1 or 0 | Button-prompt slots 1 to 5 are shown: their text is set and no quick panel is open. Only prompt slot 5 has a setter that is called. | High |
| 26 | 1 or 0 | Cross hint on the journal and the map: the selected journal row has an area (or the current map icon has items) and no quick panel is open. | High |
| 28 | 1 or 0 | The quick-panel selection equals the argument. | High |
| 29 | 1 or 0 | The count of quick-panel entry 1, 0, 3 or 2 (for argument 0, 1, 2 or any other) is above 0. | High |
| 30, 33, 34, 36 | 1 or 0 | The count of quick-panel entry 0, 1, 2 or 3 is above 0. | High |
| 35 | 1 or 0 | Entry 2's count is above 0 and its item passes the item flag test with mode 5. | High |
| 31 | 1 or 0 | A quick-panel item passes the item flag test, with the argument as the mode. | High |
| 32, 37 | 1 or 0 | The first (32) or second (37) quick panel is open, and camera word +0x50 is zero. | High |
| 38 | 1 or 0 | Tests the active petpet kind. The current pet type id maps to kinds 0, 1, 3, 6, 7 or 8; no active petpet or an unmapped type returns -1. Argument 255 passes any recognized kind; other arguments pass only when equal to the mapped kind. The petpet portrait group in SPEC-0023 uses this predicate. | High |
| 39 | 0 | Always fails (its handler returns zero). Any node whose condition mode tests this slot is never shown by it. | High |
| 41 | 1 or 0 | The monster bar is visible: the first quick panel is closed, the busy test is false (no letterbox or screen fade active, and the two camera-effect words are not above 0) and the monster bar timer (SPEC-0023 section 10.2) is not 0. Only the monster bar group 0x24 uses this slot. | High |
| 42, 43, 45, 46 | 1 or 0 | HUD bar segments. The hero is Tor (42, 45) or Roberta (43, 46), and the argument is below (W - a) / b, where W is hero word +0x158 (42, 43) or +0x160 (45, 46). The offsets a for slots 42, 43, 45 and 46 are the words at 0x319BC0, 0x319BC4, 0x319BC8 and 0x319BCC (150, 120, 90, 150 on disc) and the divisors b are the words at 0x319BD0 to 0x319BDC (25, 20, 15, 25). | High (rule and values); Medium (names of the hero words) |
| 44, 47, 48, 51, 54, 60 | 1 or 0 | HUD words and flags (SPEC-0023): 44, a HUD word is set; 47, that word is set and hero word +0x160 is above 0; 48, 51 and 54, HUD flags; 60, a HUD word is set and game-state word +0x08 is 2. | High (tests); Medium (names of the HUD words) |
| 49 | 1 or 0 | The marker is 0 or more. | High |
| 50 | 1 or 0 | The marker's item passes the item flag test, with the argument as the mode. | High |
| 53, 55 | 1 or 0 | The hero's category-1 (53) or category-2 (55) entry is present, and a HUD flag is set. | High |
| 57, 59 | 1 or 0 | The "portrait available" word of Tor (slot 57) or Roberta (slot 59). Both are 1 in the file image. The hero-switch request sets them (SPEC-0023 section 12, Start): the current hero's word to 1, the target's word to 1 only when the request is accepted and the target is alive, otherwise 0. A node under these slots is the dimmed portrait (shown while the word is 0, mode 3 of the node). | High |
| 58 | 1 or 0 | The death test of the hero slot given by the argument (0 Tor, 1 Roberta): true when that hero's action state is 16 (SPEC-0053). The dead icon of the portrait uses it. | Medium-High |
| 61 | 1 or 0 | Loading spinner (SPEC-0023 section 35): 1 when the boundary-contact word and the load-busy word are both set, or a world or streaming load is busy while the screen fade alpha is above 100, or the exterior-replacement flag A is nonzero. | Medium-High |
| 62 | 1 or 0 | No quick panel is open, slot 41 is false and the busy test is false. | High |
| 63 | 1 or 0 | The script timer display is active. | High |
| 64 | 1 or 0 | The script counter display is active. | Medium |
| 65 | 1 or 0 | Slot 62 passes and a HUD word is set. | High |
| 66 | option word | Vibration option (word +0x08). | High |
| 67 | option word | Invert left-right option (word +0x00). | High |
| 68 | option word | Invert up-down option (word +0x04). | High |
| 69 | 1 or 0 | Journal up arrow: 1 when the top row index is not 0. | High |
| 70 | 1 or 0 | Journal down arrow: the more-below flag. | High |
| 71 | count | Journal row count (the list group shows when it is above 0). | High |
| 72 | 1 or 0 | The game-over page's pass count is above twice the run-time tick rate, so the Continue caption and its small Cross icon (sprite record 527) appear after about 2 s. | High |
| 73 | count | Area total of the selected map area (gates the map's Treasures line; stored as zero on disc, but the file loader recounts it as the number of collected-table rows naming the area, so it is nonzero for 55 areas, SPEC-0051 section 3). | High |

Use of the option slots: a node with mode 2 under slot 66 to 68 shows its On node when the stored value is nonzero, and its Off node (mode 3) when it is zero (High).

Slot 12 and the item screen (High). Slot 12 is a constant false gate used to keep groups out of the automatic page draw. Records 3 and 4 hold, under a mode-2 group gated by slot 12, the item-screen groups. The item-screen states draw those groups themselves on every pass, by section C key: 0xC2 (the cell grid, the cursor and equipped frames, the action box and the category-9 panel), 0xBF (the money and equipment panel), 0xC0 (the Mote panel, Charms tab only) and 0x31 (the HUD bars, at draw offset (60, 23)). A keyed group drawn this way starts at the keyed node, so its gated parent is not tested. Records 5 to 8 hold only the full-screen picture. The in-play HUD tree hides groups the same way.

### 9d. Focus

Confidence: High.

- Focus index. Each record's run-time word +0x0C is its focus index: the position of the focused item among the page's kind-4 items in sibling order, from 0.
- Default focus. Every time a record becomes current (push, pop or direct set), its focus index is set to the page's default (section H +0x06, which is 0 on every page), and, if that item's predicate fails, to the first child item whose predicate passes. No page remembers its focus between visits.
- Re-check. On every pass, if the focused item's predicate fails, the focus moves to the first passing item (Behavior 1b, step 4).
- List page (section H byte +0x0A = 1; pages 14, 759, 774, 794 and 824): Down moves to the next kind-4 item whose condition chain passes, wrapping from the last to the first; Up moves to the previous one, wrapping from the first to the last. Left and Right give no move.
- Spatial page (byte +0x0A = 0; pages 858 and 896): the nearest passing item in the pressed direction (section 17.2). When a Left or Right search finds nothing, the record's Left or Right neighbour (section D +0x06, +0x08) is used; on disc those are all -1 or -3. Only the start-point and slot pages are spatial, and neither is ever entered.
- Drawing. The tree draw passes a focused flag down: a kind-4 item whose running index equals the focus index sets the flag for its children. Mode 1 nodes are drawn only under that flag, or when a caller draws a group with the flag set (the load and save prompts do this for their hint group). Every list item has a mode-1 highlight sprite and a mode-1 highlight label; that is the selection highlight.
- Focus states. The focused item's section I +0x06 names its focus state (table B), which runs each pass with the focus index as an argument (Behavior 1b, step 5).

### 10. Canvas and drawing position

- The front-end canvas is 512 pixels wide and 448 pixels high. Its width and height are stored in the boot program's data as u32 values at 0x2EC908 (512) and 0x2EC904 (448). A global Y scale is stored as a float at 0x2EC900 (1.0), and a further 1.0 at 0x2EC8FC. These words have exactly one writer, a four-word setter that is never called (no call reaches it and no pointer to it exists in the program), so the canvas is always 512 x 448 with Y scale 1.0 (High; SPEC-0008, Display mode). The values 512 and 448 that appear together with 32, 32 or 256, 256 elsewhere are the screen-effect object's set-up (letterbox bar heights, SPEC-0052), not canvas set-ups.
- Screen coordinates have their origin at the top-left of the canvas. X runs 0 to 511 and Y runs 0 to 447.
- Drawing to the 12.4 fixed-point screen space:
  - X_screen = (x + 2048 - 256) x 16
  - Y_screen = ((y - 224) x S + 2048) x 16, where S is the global Y scale (1.0)
  - So the canvas centre (256, 224) maps to the screen centre (2048, 2048).
- The 640 x 480 values seen elsewhere in the code belong to the 3D view, not to the UI. The UI uses 512 x 448.
- Overlay cells are drawn only inside a 10-pixel margin around the canvas (section 15.1).

### 11. Sprite frames

- A sprite's frame count is the sum of its three group counts (+0x0E + +0x0F + +0x10).
- Setting a frame on a sprite (node type 1): the frame index is stored in +0x14 only if it is less than the frame count. Otherwise the request is ignored. The index is an unsigned byte. No wrap and no clamp.
- Setting a frame on a non-sprite node is refused and the node is unchanged.
- The three group counts contribute to one contiguous valid frame-index range; the setter accepts indices below their sum. The sprite renderer does not distinguish the groups: it maps the current frame byte to a grid cell using the column and row counts. The A/B/C boundaries have no separately established effect in the shipped front-end data (High for the summed range and grid mapping; Medium-High for no separate effect).
- Sprites with a count of 0 (779 records) ignore all frame requests. The 778 linked zero-count sprites have a 1-by-1 grid and current frame 0; if their node reaches the sprite draw path, they draw from the first grid cell. The zero count itself is not a draw gate. Parent visibility, node conditions and hide flags still apply (High for draw-path behavior; visibility is conditional).

### 12. Sprite drawing

- A sprite's on-screen size is its width and height multiplied by its scale, divided by 4096: displayed width = width x scaleX / 4096, displayed height = height x scaleY / 4096.
- Alignment is the node's byte +0x2E, applied in the node transform (Section J): 0 = x is the left edge; 1 = x is the centre (shift x by half the displayed width); 2 = x is the right edge (shift x by the full displayed width); any other value gives no shift (High). The focus-search position helper has a different rule (section 17.3). Alignment shifts x only; y is always the top edge (High).
- Integer coordinates are rounded away from zero (add or subtract 0.5 before truncating; Medium).
- The frame cell is chosen from the frame grid (columns and rows at +0x0C and +0x0D). Frame index f gives column f mod columns and row f div columns, each clamped to its maximum (Medium).
- Quad (High): after the node transform, the sprite is one quad from the node's local (0, 0) to (width, height), with width the signed 16-bit value at J +0x22 and height the one at J +0x20.
- Texture (High): the texture is sampled one texel per screen pixel (before the node scale), starting at the frame cell's top-left texel. The cell's top-left is (column x width, row x height) texels: u from the column index times J +0x22, v from the row index times J +0x20. The quad is not stretched to the texture: a wider sprite shows more texels.
- Colour (High): the E record colour (bytes R, G, B, A at E +0x04) modulates the texture, with 128 meaning 1.0 per channel.

Node-kind draw dispatch (the same path is used by the front end and the HUD; see SPEC-0023):
- Each UI node has a kind byte at +0x2B and a condition mode byte at +0x2C (Section 9b). The node's position is offset by two draw-offset values set by the caller, and by the object's transform.
- Draw condition (the mode byte at +0x2C, with the predicate of Section 9b): mode 0 draws; mode 2 draws when the predicate is nonzero; mode 3 draws when it is zero; mode 1 draws only when the focused flag passed down the tree is set (section 9d). Modes above 3 are not drawn by the draw routine.
- Focused flag (High; Open question 75, resolved): the draw passes a flag down to the children of the kind-4 item whose running index equals the record's focus index. So mode 1 nodes (the highlight sprite and highlight label of each list item) show only on the focused item.
- The kind selects the shape rule. Kind 1 draws a sprite quad with the object's scale and colour, but only when a stored value of the node's object is negative; otherwise it skips the quad. Kind 7 draws a strip of cells across the node width. Kind 8 draws a fixed panel of two quads on a 64-unit cell with neutral alpha. Kind 2 draws its label text (Text node draw, below). Kind 6 draws a value text (Section K). Kind 4 adds one to a counter that the caller passes in (the running item index), and sets the focused flag for its children when the counter equals the focus index.
- A node whose condition holds draws its own shape, then draws its first child subtree. Siblings are drawn only when the caller passes the sibling flag.
- Confidence: High for the dispatch by kind, the subtree order, the draw modes and the kind 4 focused flag. Medium for the exact condition of kind 1, and for kind 7 (partly described).

Text node draw (node kind 2; High unless marked). The node's data is its Section G binding.
1. If the binding's font id (+0x0E) is negative, nothing is drawn.
2. Tint T = (1, 1, 1, 1), unless the node's colour animation is running. It runs when the node has an animation list (J +0x10), its active flag (J +0x31) is set and the list's first entry is active. Then T is the four colour bytes of the animation's colour track at time t, each divided by 128. The time t is the front-end frame counter (data word 0x2EC8C4) divided by 60 (data word 0x2EC8D4), minus the entry's start time. When the track has ended, T returns to 1 and the active flags are cleared (Medium for the track sampling).
3. Drawn colour, per channel: c' = trunc(c × T + 0.5), where c is the binding colour byte (+0x08 to +0x0B). The result is stored as one unsigned byte (its low 8 bits) and drawn unsigned, so a value above 255 wraps.
4. The text is drawn by the shared line-run routine of SPEC-0022 with the binding's string, font and letter spacing, scale (1, 1) and the drawn colour as the start colour, at the node's transformed position. That position is the baseline of the first line; each glyph lies above it (High). Horizontal alignment comes from the node transform (alignment byte +0x2E, using the node's width). Before every draw of a text node the width and height words of the node (+0x22 and +0x20) are set to the measured size of the current text, so the width stored on disc is not used and a centred label is centred on its real text.
5. Letter spacing is scaled twice (Medium-High): each glyph moves the pen by (advance + spacing × s) × s, where s is the current x scale of the node transform. With s = 1 this is advance + spacing.

Disc facts (High):
- None of the 112 text nodes has an animation list, so T is always 1 and the drawn colour equals the binding colour.
- Two text nodes carry a scale other than 1 together with a nonzero spacing: scale 0.6 with spacing -1, and scale 1.3 with spacing -3. These are the only labels where the double scaling of rule 5 changes the result.

### 13. Menu stack and screen flow (what a player sees)

- The front end starts with the card state (record 0), then the Neopets logo page (record 30), the legal page (record 1, the attract), the opening and title movies, and the title (record 10); the game-flow spec gives the timings (SPEC-0026, Attract mode and title). The title screen shows its items through a page root.
- Navigation is by the stack above. Each page root is one screen. A page's child labels are drawn in its subtree.

#### 13.1 Page tree

Each screen is one page of the front-end UI tree. The table gives the container node that holds the page and the page root that the screen draws. Node numbers are record indices in the front-end database (section J). Item index is the position of the item among the page's selectable children. Confidence: High for the roots and the item order; Medium-High for the gates.

| Screen | Container / page root | Items in order (index) | Gate |
| --- | --- | --- | --- |
| Title | 757 / 759 | New Game (0), Load Game (1), Options (2) | none |
| Pause | 12 / 14 | View Map (0), View Quests (1), Load Game (2), Resume Game (3), Options (4), Quit (5) | Load Game: predicate slot 0, always 1 |
| Options | 772 / 774 | Controls (0), Audio (1), Credits (2) | none |
| Controls | 792 / 794 | Vibration (0), Invert Left-Right (1), Invert Up-Down (2) | On/Off on Vibration, Yes/No on the two invert rows (section 3) |
| Audio | 822 / 824 | Music (0), Sound effects (1), Voice (2) | none |
| Start-point chooser | 854 / 858 | nine items, index 0 to 8 (section 5) | marked debug-only; never entered (Behavior 2, survey) |
| Save and load slot chooser | 892 / 896 | three labelled items, index 0 to 2; no code opens this page, and the picker draws its own four rows (section 7.1) | none |
| Load prompt | 911 / 911 | Load Game (title only); record 17, code 0x154 | none |
| Save prompt | 915 / 915 | Save Game, with Select and Back hints; record 18, opened directly at a save point (section 7.3) | none |
| Item screen | 39 / 39 | grid group, action box, money panel | the groups sit under predicate slot 12 (always 0), so the page draw skips them; the tab states draw them by section C key (section 9c) |
| Journal | 939 (record 22) | row list, up and down arrows, empty-list text, "Triangle Exit", "Cross View Map", L2 and R2 hints (section 14.3) | list group: row count above 0; "Cross View Map": selected row has an area and no overlay is busy |
| Map overlay | 999 / 999 (likely) | "Triangle Exit", "Cross View Quest", area name, L2 and R2 hints, "Treasures: n of m" (SPEC-0051 section 8) | "Cross View Quest": current icon has items; Treasures line: area total nonzero (55 of 77 areas on disc data) |

- A selectable page item is a kind-4 node. It has a neutral label child and a mode-1 highlight label and highlight sprite, drawn only while the item has focus (section 9d). Its argument byte is 0 for the pause, title, option and controls items, and 0 to 8 for the start-point items.
- Select in play opens the pause page; Start opens the item screen on the current tab (Medium).
- Pause targets (High; section I and Behavior 2): View Map (state 2) opens the map page of the hero's area on Cross, or does nothing when that area is negative. View Quests (state 3) opens the journal (table B 25) on Cross, with the selected map area set to the hero's area. Load Game, on the pause page and on the title page, opens the load prompt (record 17, table B 21); on the pause page its focus state is 4. Resume Game (state 5) plays cue 0x95 and returns to play on Cross. Options opens record 11 (page 774). Quit (state 6) plays cue 0x9A and opens the quit box on Cross; the box's Circle answer (Quit) ends the game and its Triangle answer (Resume) closes the box (Behavior 2, state 7).
- The pause page has no Save Game item. Save is reached only through the save prompt (root 915, table B 22), which a save point in a level opens directly (section 7.3).

#### 13.2 Page graph

Confidence: High. Every reachable page with its input edges. Records are section D records (record = code / 20); the state is the table B state that runs the record. "Back" is Triangle by the back rule of section 9 (cue 0x99). Focus moves on list pages play cue 0x9A. Stack: "push", "pop" and "reset" follow section 9; "cut to 9, push" means the stack is cut back to play and the page is pushed over it; "none" means no stack change (+0x04 = -3).

| From (record, state) | Input or condition | To | Cue | Stack |
| --- | --- | --- | --- | --- |
| Start-up | front end set up | card state (record 0) | - | reset |
| Card (0, 0) | controller usable and card job not busy (first visit since boot); next pass on later visits | logo countdown (30) | - | reset |
| Logo countdown (30, 29) | pass 300 (301 passes); no input is read | attract (1) | - | reset |
| Attract (1, 1) | wait done (300 passes the first time since boot, else 1), counted only while the controller check is ready | movies 0 and 86 (each skippable on its own), then title (10) | - | reset |
| Title (10, 12) | Cross on New Game, page pass 5 or later | play (9) | 0x9A | reset |
| Title | Cross on Load Game / Options | load (17) / options (11) | 0x98 | push |
| Title | 30 s with no new press (pass 1,801) | logo countdown (30) | - | reset |
| Options (11, none) | Cross on Controls / Audio / Credits | 12 / 13 / 14 | 0x98 | push |
| Options, Controls, Audio, Credits | Triangle | the screen below | 0x99 | pop |
| Controls (12, none) | Left or Right on a row | the row toggles (states 13, 14); page stays | - | - |
| Audio (13, none) | Left or Right held on a row | the slider moves (states 15 to 17); page stays | - | - |
| Credits (14, 18), from Options | scroll ends | options (11) | - | pop |
| Credits, by the story bit | scroll ends (Cross and Triangle ignored) | card state (0), after world teardown; then logo, attract, movies and title | - | reset |
| Load (17, 21) | Cross in the picker | play (9) | load cues | reset |
| Load | Triangle in the picker | pause (2) or title (10) | 0x99 | pop |
| Load | cancel answer in the card box, or no save data; or card removed in the picker, then Cancel load and Yes | pause (2) or title (10) | - | pop |
| Play (9, 11) | story bit 368 set | credits (14) | - | push |
| Play | book cursor 0 or more | book (20) | - | push |
| Play | flag E set | game over (23) | - | reset |
| Play | Select, not busy | pause (2) | 0x94 | push |
| Play | Start, not busy | item tab record (3 to 8) | 0x94 | none |
| Play | D-pad Left, shortcut gate passes, hero map area 0 or more | map (31 + area) | 0x9A | cut to 9, push |
| Play | D-pad Right, shortcut gate passes | journal (22) | 0x9A | cut to 9, push |
| Play | overlay box busy | box host (19) | - | reset |
| Play | Square at a save point | save (18), by direct set | - | reset |
| Pause (2, 7) | Cross on View Map, hero map area 0 or more | map (31 + area) | - | cut to 9, push |
| Pause | Cross on View Quests | journal (22) | - | cut to 9, push |
| Pause | Cross on Load Game / Options | load (17) / options (11) | 0x98 | push |
| Pause | Cross on Resume Game | play (9) | 0x95 | reset |
| Pause | Cross on Quit | the quit box opens | 0x9A | - |
| Pause, quit box open | Circle (Quit) / Triangle (Resume) | card state (0) / the box closes, pause stays | 0x98 / 0x99 | reset / - |
| Pause, box closed | Select | play (9) | 0x95 | reset |
| Pause, box closed | Triangle | play (9) | 0x99 | pop |
| Item tab (3 to 8, states 8 to 10) | Triangle or Start | play (9) | 0x95 | reset |
| Item tab | L1 / R1 | previous / next tab record | 0x98 / 0x99 | none |
| Item tab | L2 | journal (22) | 0x98 | cut to 9, push |
| Item tab | R2 | map of the hero map area, else journal | 0x99 | cut to 9, push |
| Journal (22, 25) | Cross on a row with an area | map (31 + area) | 0x98 | cut to 9, push |
| Journal | L2 | map of the hero map area, else item tab | 0x99 | cut to 9, push / none |
| Journal | R2 | item tab | 0x98 | none |
| Journal, map | Triangle | play (9) | 0x99 | pop |
| Map (31 to 107, 30) | Select | play (9) | - | reset |
| Map | Cross on an icon with items, or R2 | journal (22) | 0x98 | cut to 9, push |
| Map | L2 | item tab | - | none |
| Box host (19, 23) | box idle | play (9) | 0x97 | reset |
| Book (20, 24) | Cross | play (9) | 0x9A | reset |
| Book | Triangle | play (9) for one pass, then the book again by play's book gate (section 6.1) | 0x99 | pop, then push |
| Save (18, 22) | Triangle in the picker | play (9) | 0x99 | reset |
| Save | Cross, write done with card state word 0 (also "Cancel save?" Yes inside the write job, nothing written) | play (9) | 0xA9 | reset |
| Save | "Cancel save?" Yes in the scan or in the card-gone job | play (9) | - | reset |
| Game over (23, 26) | Cross, page pass 122 or later | play (9) when the pending flag is set, else card state (0) | - | reset |

Default focus on entry (section 9d): list pages start on item 0 (pause: View Map; title: New Game; options: Controls; controls: Vibration; audio: Music). The item screen's Items tab keeps its last cell, or cell 0 if that cell is empty; the Quest-items tab starts at 0; the Charms tab shows the shared Mote selection (section 14.1). Journal: row 0, or the pending row (section 14.3). Map: icon 0, or the pending icon (SPEC-0051 section 6). Load picker: the first filled slot (section 7.2). Save picker: slot 0 (section 7.3).

### 14. Item screen, journal and map

Confidence: High on the tab and grid structure, the entry-code groups, the hero index, the table references, the L2/R2 page ring and the journal rules (including the row-part positions and fonts); Medium-High on the item cell and use rules; High on the Circle marker rule, the Charms-tab Mote picker and the start tab. The map page rules are in SPEC-0051. The shop screen body and a separate equipment screen body are not known.

#### 14.1 Item screen and its three tabs

- Cursor rule on the Items tab (Medium-High): the cursor is an index from 0 to 19, with row = index divided by 5 and column = index modulo 5. Up subtracts one row (row wraps 0 to 3), Down adds one row, Left subtracts one column (column wraps 0 to 4), and Right adds one column. Examples: from cell 19, Right gives 15 and Down gives 4; from cell 0, Up gives 15 and Left gives 4.
- Equipped frame (High): a 42 by 42 frame is drawn behind a cell whose id equals the active hero's quick-item selection marker (predicate slot 1). It is in the group with key 0xC2, which the tab states draw themselves every pass (section 9c).

- The item screen has three tabs: Items (tab 0), Charms (tab 1) and Quest items (tab 2). L1 moves to the previous tab and R1 to the next; the index wraps (0 to 2 and 2 to 0). Changing tab plays cue 0x98 (previous) or 0x99 (next).
- Start tab (High). The tab word is 0 (Items) at boot. Only L1 and R1 change it, so the item screen opens on the last tab viewed, for the rest of the session. It is not saved, and it is not reset by a new game or a load.
- Cursor on entry (High). Items tab: the cursor is kept between visits; on entry it is set to 0 only when its cell is empty. Quest-items tab: the cursor and the scroll are set to 0 on entry. Charms tab: the Mote selection is the one shared with the in-play second quick panel (boot value 255, none).
- Items tab: the grid always has 20 cells in four rows of five. Cell k (k from 0 to 19) holds the k-th item id of a list of 20 consecutive unsigned 32-bit words in the boot program's data, at virtual address 0x2DCF28. The list is read at run time from the disc; it is not copied here. High for the list (nothing writes it, so it is the same on every boot and in every save; Open question 36); Medium-High for the cursor and wrap rules. All 20 ids have item category 6 (Open question 37). Test vector: cell 0 holds id 150, cell 1 id 146 and cell 19 id 50.
- Charms tab: the cells are the owned records of item category 8 (charms, which are the config type-17 items of SPEC-0021), in list order. Its selection and buttons are those of the Mote picker of the in-play second quick panel (SPEC-0021, Second quick panel and Mote equipping): directions from the menu rising-edge word, buttons from the second input word; Cross equips the selection into equipment word 0, Square into word 1 (when the shield family is present) and Circle into word 2 (when the weapon family is present). The Mote panel (key 0xC0) is drawn on this tab only. Quest-items tab: the owned records of category 7, in list order. An owned record has a count above zero and a present item. Up to 20 cells show at a time. The Quest-items tab does not scroll: only the first 20 owned records can be shown or selected (Quest-items tab, below).
- Cursor: up and down move the cursor one row, and left and right one column. The cursor wraps within the grid. On the Items tab the grid always has 20 cells, so every direction press plays the move cue of its direction (Down, Up, Right, Left; vertical wins) even when the cursor does not move (High). On the Quest-items tab the exact rule is in the Quest-items bullet below. On the Charms tab the Mote picker moves the selection (SPEC-0021). Medium-High.
- Cell drawing (Items tab): a cell with an owned count of 0 is hidden. A count of 1 shows the item's icon with no number. A count above 1 shows the number in decimal, drawn in the neutral grey (the colour word 0x80808080 at data address 0x315AD8, the same neutral value as the menu labels). Medium-High.
- HUD lines (High): on every tick, each of the tab states 8, 9 and 10 sets the HUD timers G0 (bars group), G1 (item panel) and C (change detector) to 240 (SPEC-0023), so these stay fully visible. It draws the HUD bar group (element key 0x31) with a draw offset of (60, 23) on its top node, and clears that offset after the draw; the offset is used only here, not in play. It also draws element 0xBF (the "NeoPoints" label and the balance value text) and element 0xBE, whose children are the equipped-item icons of categories 1, 5 and 2, the category-2 icon for hero 0 (Tor) only; an empty icon is drawn with alpha 0 (the strip is described in the next bullet).
- Equipped-item strip (High for the order, the hide rule and the sources; Medium-High for the pixel offsets). This is the only place the item screen shows a weapon, a shield or a body item (armour or dress). No tab lists them: the Items tab grid holds only the 20 consumable ids, the Charms tab only charms and the Quest-items tab only quest items. The strip is element 0xBE, a group of three 64 by 64 icons stacked top to bottom, inside the NeoPoints group (0xBF); the group sits at (91, 113) in that group, and the icons at (0, 0), (2, 61) and (0, 122) inside it. All three tab states refresh and draw it on every pass.
  - Icon 1 (top) is the active hero's weapon (category-1 entry), icon 2 (middle) is the body item (category-5 entry) and icon 3 (bottom) is the shield (category-2 entry), shown for Tor only; for Roberta the third icon is always hidden.
  - The hero index is the active hero (0 Tor, 1 Roberta). For each entry that exists and is not empty (an item present and a held count above 0) the item's id picks the icon: the icon art of item id k is child number k of the item-icon group (key 0x9F, 160 children, so ids 0 to 159 have art). The icon is drawn at alpha 128. An empty or missing entry, or an id with no icon child, leaves the icon hidden (alpha 0) or with its previous art.
  - No name, description, count or "equipped" frame is drawn for the strip. The name and description lines, the count numbers and the equipped frame belong to the grid cells of the Items tab only, so the body item's name (for example "Farmer's Clothes") is never shown on the item screen.
  - The middle (body) icon is hidden on the Items tab while the active hero owns the Tattered Cloak (the cloak entry exists and is not empty); then two other small icons show the cloak and the current body item beside the Square (cloak swap) hint. On the Charms and Quest-items tabs the middle icon stays shown. Without the cloak the middle icon is shown on all three tabs.
  - The body icon changes at once when the body entry changes (new armour picked up, cloak swap).
- Fresh game, Tor active (disc data; the worked example for the strip and the tabs). The new game gives the party Farmer's Clothes (item 119, in Tor's body entry), Red Juppie x3 and Purple Juppie x3 and nothing else (SPEC-0021, Starting equipment). The item screen then shows:
  - Items tab (page record 6): of the 20 grid cells only cell 0 (Red Juppie, 150, count 3, number "3" drawn) and cell 5 (Purple Juppie, 149, count 3) are visible; the other 18 cells are hidden. The cursor is on cell 0: name line "Red Juppie (3)", description "Restores a small amount of health." Cursor on cell 5: "Purple Juppie (3)", "Replenishes a small amount of magic." Cross shows the Use action; Square does nothing (no cloak). Cell 1 (Chokato) and every other empty cell give a single space on both lines when selected.
  - Equipped-item strip (all three tabs): top icon (weapon) hidden, middle icon shows the art of item 119 (Farmer's Clothes) at alpha 128, bottom icon (shield) hidden. There is no text for it.
  - Charms tab (page record 7): no cells (no charm owned); the Mote panel is empty and every equipment word is -1. The strip is as above.
  - Quest-items tab (page record 8): no cells. The strip is as above.
  - NeoPoints balance 0.
  - Roberta cannot be selected in Act 1. At the Act 2 start she is selected and the party inventory is the Act 2 kit (SPEC-0021, Starting equipment): on her pages (records 3 to 5) the Items tab shows cell 0 (Red Juppie x3) and cell 5 (Purple Juppie x3) only, and the strip shows only the middle icon, the art of item 127 (Diplomat Dress); her weapon icon is hidden and her shield icon is always hidden. NeoPoints balance 1150. Tor's Act 1 holdings are in the stash and return at the Act 3 start.
- Quest-items tab cursor and scroll (High). Let n be the number of owned category-7 records (a record is owned when it has an item and a held count above 0; the list has 60 slots and empty slots may lie between owned ones). The cursor c is an index into the owned records in slot order (0 is the first owned record), not into the cells. With the 4-row by 5-column layout (row = c divided by 5, column = c modulo 5): nothing moves unless n is 2 or more. Up subtracts one row and Down adds one, wrapping between row 3 and row 0; Left subtracts one column and Right adds one, wrapping within the same row (Right at column 4 goes to column 0 of the same row, Left at column 0 goes to column 4); the row never changes on Left or Right. When the target index is n or more (an empty cell), the same step is applied again until the target is below n, so empty cells are skipped and the wrap continues through them. If Up and Down (or Left and Right) are both held, they cancel. Because the row is always taken modulo 4, c is always 0 to 19. The scroll offset is c's overflow above 19 divided by 5, which is always 0, so the offset never changes from its entry value 0: cell k shows the k-th owned record, the highlighted cell is c, and owned records from the 21st on are never shown and cannot be selected (a hero can hold up to 60). No page jump and no clamp exist. The scan reads slots 0 to 59 and fills cells with owned records until 20 are filled; unfilled cells are hidden. No more-above or more-below arrow is drawn on this tab (Medium-High: no code sets one; the only scroll arrows are the journal's). Sound: when n is 2 or more and a direction is pressed, front-end menu sound 0 (Down), 1 (Up), 2 (Right) or 3 (Left) plays (vertical wins over horizontal; cue 0x9A) every press, even when the cursor does not change (for example n = 2 and Down from cell 0); there is no distinct cue for an edge. With n of 0 or 1 there is no move and no cue. Description panel: it shows the name and description of the c-th owned record (section 14.1, item name lines). Cross and Circle run the usability test: the entry exists, has an item-state record, and its category is not 7 (quest items). Every entry of this tab is category 7, so Cross and Circle always fail and play cue 0xA2.
- Item name and description lines (High). On every pass that stays on the screen (no close or page-ring key this pass), the shared step of the three tab states writes the two item text lines for the selected entry. With no entry, or an entry whose held count is 0 or less, both lines become a single space. Otherwise line 1 is the item's name from its text row (copied up to 31 bytes) and line 2 its description (up to 255 bytes). Then, when the text row's carry limit (+0x08) is 2 or more and the item's times-obtained count (the per-item count table, SPEC-0025) is above 0, line 1 becomes the name, a space, and the two numbers in parentheses separated by a slash: obtained, then limit. When the carry limit is below 2 and the entry's held count is above 1, line 1 becomes the name, a space, and the held count in parentheses. Otherwise line 1 is the plain name. Test vectors (synthetic name Apple): limit 10, obtained 3 gives "Apple (3/10)"; limit 10, obtained 0 gives "Apple"; limit 1, held 4 gives "Apple (4)"; limit 1, held 1 gives "Apple"; no entry gives a single space on both lines.

#### 14.2 Buttons on the item screen

| Input | Role | Cue | Result |
| --- | --- | --- | --- |
| L1 (0x0004) | Previous tab | 0x98 | Index wraps. |
| R1 (0x0008) | Next tab | 0x99 | Index wraps. |
| L2 (0x0001) | Page ring: journal | 0x98 | Returns the journal code 0x1B8 (record 22). High. |
| R2 (0x0002) | Page ring: map | 0x99 | Sets the selected map area to the hero map area and returns that area's map page code (0x26C + 20 x area). If the area is negative, returns the journal code 0x1B8 instead. High. |
| Cross (0x0040) | Use the selected item | 0xA2 on denial | Denied with 0xA2 when the cell is empty, or when the active hero's check for the id fails. Otherwise the item's use action runs; that action plays its own effects (not established). Medium-High. |
| Circle (0x0020) | Items tab: set or clear the quick-item selection marker | 0x9A (and 0xA3 on a set) | When the active hero's quick-item selection marker (SPEC-0021) equals the cell's id, the marker is cleared to none (-1). Otherwise the marker is set to the cell's id; the marker set plays cue 0xA3 when it succeeds. Cue 0x9A plays in both cases. No equipment word is written. High (Medium-High that the id set is the cell's id). Charms tab: equips the selected Mote into word 2 (section 14.1). Quest-items tab: always refused with 0xA2. |
| Square (0x0080) | Put the Tattered Cloak on or take it off | 0xA1 | Items tab only. When the active hero's category-9 entry (its cloak entry; the hero index word of 14.4 picks the slot) exists and is not empty, the cloak swap of SPEC-0021 (Tattered Cloak) runs for the active hero: its body entry and cloak entry are exchanged, so the cloak goes on, or comes off when it was worn. Otherwise nothing happens. High. |
| Triangle (0x0010) or Start (0x0800) | Close the screen | 0x95 | Returns code 0x0B4 (record 9, in play). High, Medium (identity). |

- The "equipped" test (predicate slots 1 and 6) is the marker test: it compares the cell's id with the active hero's quick-item selection marker. It is applied on the Items tab only (High).
- Circle detail (High): Circle on the Items tab writes only the marker. Circle on a cell with no record or no item object does nothing. On the Charms tab the buttons are the Mote picker's (Cross word 0, Square word 1, Circle word 2; section 14.1). On the Quest-items tab Cross and Circle always fail the usability test and play 0xA2.
- Action box (High): when a cell is shown, the box lists the actions for its id. Cross-usable cells show "Use". Circle-usable cells show "Activate" when the cell's id is not the marker and "Deactivate" when it is. "Exit" closes the box. The box is in the key 0xC2 group, which the tab states draw themselves (section 9c). The same words also appear in the in-play quick-item box (SPEC-0023).

#### 14.3 Quest journal (record 22, table B 25)

Confidence: High unless marked. The journal and the map are the same for both heroes.

- Data: the journal reads two tables of the item-table file (resource 533, IDMX tables; SPEC-0022 Quests and SPEC-0021 for the file). Quest rows are table index 1 (238 rows of 36 bytes; layout in SPEC-0051 Layout). Treasure rows are table index 3 (48 rows of 20 bytes). Treasure row i is paired with table index 6 entry i + 200 (table 6 has 248 rows of 8 bytes; its first field is an area id, and its collected flag is a runtime field that is zero on disc).
- Entry: on the first pass the cursor, the top row index and the more-below flag are set to 0. If a pending target (a quest row or a treasure record, held as a reference; set by the map, SPEC-0051 section 6) matches a listed row, the cursor starts on that row instead. The pending target is cleared after the list is built.
- Rows, in order: every accepted quest row in table order, then every held, uncollected treasure that has an area, in treasure table order. An accepted quest row has neither flag id -1, its start flag set and its done flag clear (SPEC-0049). A listed treasure has its held-map flag (+0x10) nonzero, its area (+0x0C) not -1, and its collected flag (table 6 entry 200 + i) zero (SPEC-0051 section 2). The journal applies no area, stage or scene filter. A debug-only switch word, zero on disc, makes these flag tests pass for every row (SPEC-0051 Edge cases); it is not set in play. There is no list of completed quests.
- Row content: a quest row shows its title (+0x0C) and its objective (+0x10). A treasure row shows its name (+0x00) and no objective.
- Row type, icon and bar colour (RGB, alpha 128):

| Row | Type | Icon | Bar colour |
| --- | --- | --- | --- |
| Quest row, kind 0 (main story) | 0 | Primary | (222, 160, 1) |
| Quest row, kind 1 (side quest) | 1 | Secondary | (155, 155, 129) |
| Treasure | 2 | Treasure | (155, 155, 129) |

  The save and load picker (section 7.1) uses the same row drawer with types 3 to 6 (Act 1 to Act 4, row type = act + 2) and 7 (Empty), all with bar colour (78, 119, 174) (Medium-High).
- Selected row: its bar RGB is the base colour times the pulse, clamped to 0 to 255 and rounded half away from zero. The pulse moves by 0.02 per tick (data word 0x2DCDBC) between 1.2 (0x2DCDB4) and 1.8 (0x2DCDB8): each tick add the step; if the value is above 1.8, set it to 1.8 and negate the step; else if it is below 1.2, set it to 1.2 and negate the step. One full cycle takes about 60 ticks. The step keeps its sign between visits (it is one stored word). Every cursor move sets the pulse to 1.8. The other rows use their base colour.
- The selected-row vector is a colour, not a position offset. The drawer applies it only when the caller marks that row as selected, and writes the resulting RGBA bytes to the bar before drawing. The journal caller marks the cursor row after resolving any pending map target; the save/load picker marks its selected slot. Row element positions stay at the fixed positions above plus the cumulative row offset. The page's first pass draws no rows, then later passes draw them at those fixed offsets; there is no row slide-in.
- Fonts: the title is font 0 (line height 21) and the objective is font 2 (line height 14). The font ids come from the label bindings of the title and objective nodes.
- Text: the title colour is the word at 0x2DCDC8 (bytes 127, 125, 81, 128); the objective colour is the word at 0x2DCDD0 (neutral 128, 128, 128, 128). The title is one line. The objective wraps at 316 pixels (0x2DCDA8). Its line limit is floor((360 - (128 + p)) / 14) + 2, with the division truncated toward zero, where p is the row's offset (below). The objective is drawn only when the limit is not 0.
- Row parts. Every part is a canvas-absolute node of the journal tree, named by its node key and drawn at its node position plus (0, p), where p is the row's offset (0 for the first drawn row):

| Part | Node key | Position (x, y + p) | Notes |
| --- | --- | --- | --- |
| Left cap | 0xD3 | (95, 77) | 18 x 38; its y is the row-top reference |
| Right cap | 0x37 | (429, 77) | 18 x 38 |
| Bar | 0x36 | (104, 83) | Height 26; width set to 334; colour = the row colour (pulsed when selected) |
| Icon | child number "type" of the group with key 0xDD | (49, 84) for type 0; (51, 84) for types 1 to 7 | Types 0 to 2 journal, 3 to 7 save picker |
| Title | 0x34 | (118, 104) | Font 0, title colour, one line |
| Objective | 0x33 | (118, 128) | Font 2, wrap 316, neutral colour; treasure rows pass an empty text |

- Sizes: bar width 334 (0x2DCDC0), icon height 48 (0x2DCDD4), row padding 24 (0x2DCDAC), list height 360 (0x2DCDB0). Row height: the next row's offset is p + max(79, 75 + 14 L), where L is the number of objective lines drawn. A treasure row (L = 0) is 79 high; a quest row with 1, 2 or 3 lines is 89, 103 or 117 high.
- Visible rows: every row from the top index on is laid out down the list, each starting where the previous one ended. A row whose bottom offset (the next p) is 360 or more is not drawn. A row whose bottom offset is 360 or less counts as fully shown for scrolling. Quirk (reproduce): a row ending exactly at 360 is hidden but still counts as shown, so a cursor on it does not scroll it into view. On the journal's first pass no row is drawn. The more-below flag is set when a further row exists and the list ran past 360 or a text was cut.
- Selected bar alpha: the pulse changes only the bar's RGB; its alpha byte is not written and stays 128.
- Keys (main press-edge word): Down (0x4000) moves the cursor + 1, clamped to the last row; Up (0x1000) moves it - 1, clamped to 0. Each move sets the pulse to 1.8 and plays a front-end move sound. Down is tested first. With an empty list, Down sets the cursor to -1 (the row count minus 1) and Up clamps it back to 0; this has no visible effect. Scrolling: when the cursor is below the last fully shown row, the top index grows by 1 per tick; when it is above the top index, the top index shrinks by 1 per tick.
- The journal state also runs the controller check (section 2b) every pass.
- Cross (0x0040) on a row with an area (quest scene +0x20 is 0 or more, or treasure area +0x0C is 0 or more): the selected map area becomes the row's area; the front-end select sound plays; the map page of that area opens (code 0x26C + 20 x area). Quirk (reproduce; High): the journal sets the pending row target and then the pending treasure target in one step, and the second setter clears the first. So Cross on a quest row hands the map no pending target (the map cursor starts on icon 0), and Cross on a treasure row hands the map that treasure (the map cursor starts on its icon). Cross on a row without an area does nothing.
- L2 and R2 (second input word), as in the page ring (14.4): L2 plays cue 0x99 and opens the map of the hero map area, or the item screen when that area is negative; R2 plays cue 0x98 and returns the item-screen code for the current tab and hero.
- Triangle backs out through the shared back rule (section 9). Record 22's back rule is 9, so entering the journal cuts the stack back to play, and Triangle always returns to play (cue 0x99), also when the journal was opened from the pause page (High).
- Indicators: the list group is shown when the row count is above 0; otherwise a two-line message says that no quests are currently available. The up arrow shows when the top index is not 0; the down arrow (the same art, flipped) shows when the more-below flag is set. "Triangle Exit" is always shown; "Cross View Map" only when the selected row has an area and no overlay is busy. L2 and R2 tab hints are drawn.
- Background: picture resource 593 (Behavior 1a).
- Opening the quest journal clears the HUD's journal-notification flag, so the in-play journal icon stops showing (SPEC-0023).

#### 14.4 Map (table B 30), the page ring and the hero index

The map page (records 31 to 107, one per area) is specified in SPEC-0051: pictures, the selected map area, icons, cursor, pulses, projection, hero marker, overlay and exits. Summary of the parts that touch the other front-end screens (High):

- Selected map area: one word. View Map, D-pad Left in play, R2 on the item screen and L2 on the journal set it to the hero map area (the world pack's map area id, SPEC-0006, else the last transition map, SPEC-0051 section 1). Cross on a journal row sets it to that row's area. When it is negative the map does not open.
- Exits from the map: Select (main word, 0x0100) returns to play (code 0x0B4), no cue. Cross on an icon that has items, or R2, sets the pending target to the icon's quest row or treasure reference (none if the icon has no items, or with an empty list), plays the select sound and opens the journal (code 0x1B8). L2 returns the item-screen code for the current tab and hero, no cue. Cross on an icon without items, or with an empty icon list, does nothing; the area name is drawn every tick.
- Pending target: a reference to a quest row or a treasure record, compared by identity, not an id. The journal starts its cursor on the matching row; the map starts its cursor on the first icon with the same reference (SPEC-0051 section 6).

Page ring. Item screen, map and journal form a ring; R2 moves forward (item screen, map, journal), L2 moves back:

| From | L2 | R2 |
| --- | --- | --- |
| Item screen | journal (cue 0x98) | map of the hero map area (cue 0x99); journal if the area is negative |
| Map | item screen (no cue) | journal, current icon pending (select sound) |
| Journal | map of the hero map area (cue 0x99); item screen if the area is negative | item screen (cue 0x98) |

Both the map and the journal draw L2 and R2 tab hints.

Hero index. The value that picks the item-screen pages is the active hero's index (the global game object word +0x14C, SPEC-0053: 0 Tor, 1 Roberta). The journal and the map are the same for both heroes; only the map's hero marker follows the active hero.

- Entry and tab-change codes: the item-screen code for the current tab is returned when the item screen is entered (from Start), when its tab changes (L1 or R1), and when the map or journal hands back to it. A code N selects record N / 20 (section D).
  - Hero index not 1 (Tor; group A): Items tab 0x78 (record 6), Charms tab 0x8C (record 7), Quest items tab 0xA0 (record 8).
  - Hero index 1 (Roberta; group B): Items tab 0x3C (record 3), Charms tab 0x50 (record 4), Quest items tab 0x64 (record 5).
  - Each record shows its own picture (Behavior 1a): Roberta's pages for records 3 to 5, Tormund's for records 6 to 8.
  - Closing the item screen (Triangle or Start, Section 14.2) returns code 0x0B4 and is not one of these codes.

#### 14.5 Map projection table

Confidence: High for the record count, the record size, the zero start and the start-up write (the table is written only by the start-up init call; no other writer is known), and for the transform shape and its use (SPEC-0051 section 7). Medium for the yaw offset. The values are not in this spec; SPEC-0051 gives the address and the read rule (Open question 49).

What it is. The map needs one projection record per map area, so that the hero marker can be placed and oriented on the map for the area the hero is in. The table has 77 records, one for each area index from 0 to 76.

Storage and size:
- Each record is 0x50 bytes (80 bytes). The 77 records are contiguous, record k at byte offset 80 times k from the table start. The whole table is 6,160 bytes.
- The table is not in the disc image. It starts at the run-time address given in SPEC-0051 (Where it lives), beyond the file-backed part of the boot program, and its memory is zero until the start-up init call runs.

Record layout (per record):

| Offset | Size | Type | Meaning | Confidence |
| --- | --- | --- | --- | --- |
| +0x00 | 64 | 16 f32 | Four groups of four 32-bit floats G0 to G3, used as the columns of a 4 by 4 transform. The records written at start-up have the shape G0 = (a, 0, b, 0), G1 = (0, 1, 0, 0), G2 = (c, 0, d, 0), G3 = (tx, 0, tz, 1): a 2D affine map from world (x, z) to canvas pixels (SPEC-0051 section 7). | High |
| +0x40 | f32 | float | Yaw offset, read in radians (Open question 52). | Medium |
| +0x44 | 12 | zero | Unused padding that completes the 16-byte slot. | High |

The transform values differ from record to record. The float at +0x40 takes only a few small positive values, shared across records. Neither is copied here.

Start-up write. The table is filled by one of the game's subsystem init hooks. That hook does nothing unless it is called with a non-zero first argument and the init mode value; any other call leaves the table alone. When it runs, it writes all 77 records, each with its four groups of floats and its yaw value. Until it has run, every record reads as zero. Whether the game ever draws the map before start-up is not known.

Required values (plain statement). The clean side needs the values of all 77 records to draw the hero marker correctly on the map. By the user's decision they are read from the user's own boot executable, not copied into a spec; SPEC-0051 (Where it lives, Layout and Open question 1) gives the table address, the record layout and the read rule. The start-up code builds the records from values held in its own code, with no source table in the file image; how the clean side obtains them is SPEC-0051 Open question 1.

How the map marker uses a record (behaviour, in words):
1. Choose the record: the record of the displayed area (the selected map area, 14.4). A map page opens only for an area of 0 or more, so a negative index is not reached through the page routes (Open question 51).
2. Transform the marker point: (x, y, z, 1) from the hero's world position, times the transform with the groups as columns; screen x = a x + c z + tx and screen y = b x + d z + tz (result components 0 and 2). The height does not affect the point. Icons use the same record with their world x and z (SPEC-0051 section 7). The marker is drawn only when the hero map area equals the displayed area.
3. Orient the marker. Subtract the record's yaw offset from the hero's facing angle and add pi: marker angle = facing - yaw offset + pi, in radians (High; SPEC-0051 section 7).
4. Animate: the float pulse (step 0.03, bounds 0.8 and 1.5) scales the marker's colour (brightness 102 to 192); the integer pulse (step 50, bounds 3900 and 4300, 4.12 fixed point) is the cursor icon's scale, not an angle. Both reverse at their bounds (SPEC-0051 section 5).

The drawer reads a record only in the marker and icon drawing of the map (14.4). No other code reads the table.

Edge cases:
- A record read before start-up has run is all zero. A zero transform and a zero yaw would place the marker at the origin of map space; whether that can happen in play is not known.
- A current area index of 77 or more has no record in the table. The reader does not check for it (Open question 51).
- A negative index: the reader refuses it, and the marker result for that case is not established.
- The start-up call can be made with other mode values; those calls do nothing to this table.

Test vectors (synthetic, not disc values):
- Identity transform (a = d = 1, b = c = 0, tx = tz = 0): a hero at (x, y, z) gives the marker point (x, z); the height y is ignored.
- A synthetic record with a = d = 1, b = c = 0 and G3 = (10, 0, 5, 1): a hero at (0, 0, 0) gives a marker at (10, 5); a hero at (3, 40, 2) gives (13, 7).
- The marker angle with a yaw offset follows step 3 (facing - yaw offset + pi), matching SPEC-0051 section 7. Disc-value test vectors for areas 0 and 1 are in SPEC-0051 section 7.

Open questions for this section are 49 to 53.

### 15. Menu overlay cell and price caption

Confidence: overlay cell rule Medium-High; price caption High (coin model: High except Medium-High for the screen position). No shop screen that draws the caption is known (Open question 1).

15.1 Overlay cell

- An overlay cell is a cell of a menu overlay. Its projected position is converted to canvas pixels by subtracting the canvas origin values 1792 (horizontal) and 1824 (vertical). These are the same origin as Section 10 in the screen's pixel space (2048 minus 256, and 2048 minus 224), used here as code constants.
- The cell is drawn only if its canvas x satisfies -10 < x < 522 and its canvas y satisfies -10 < y < 458. A cell outside that range is not drawn.
- The margin is 10 pixels beyond each edge of the 512 by 448 canvas.
- Caller: the common overlay tick is the only routine that draws overlay cells, and it is run by these callers and no others (High; proactive (open close-out)): the pause item states 2 (View Map), 5 (Resume Game) and 6 (Quit); the in-play state (table B state 11); the new-game start-point confirm state (19); the never-entered slot-item focus state (20); the box host state (23); the map state (30); and the shared item-screen and journal input step, which states 8, 9 and 10 call. A further routine with the same body as the box host's tail has no caller. After every tick the tick empties the cell list, whether a cell was drawn or skipped.

15.2 Price caption

- Two lines, each with a value and a shown counter. Values and counters are run-time words (zero in the file image).
- Drawing: the caption is drawn by the in-play state (table B state 11; only while both quick panels are closed, in the same block as the prompt slots) and by the box host (state 23, every pass). A line is drawn while its counter is above 0, and each draw lowers its counter by 1. A pass that does not draw the line does not lower its counter.
- Both lines are font 0, right-aligned with the right edge at canvas x 410 (the text's width is subtracted from 410; the same boot-program word is the dialogue box's wrap width, SPEC-0022 Layout), at canvas y 115 (line 1) and y 140 (line 2).
- Line 1: if the value is exactly 32767 (0x7FFF), the text "Exorbitant!" (boot data, 0x315A48) is shown in colour (128, 60, 90, 127) (0x315A40). Otherwise the value is written into a nine-cell field (SPEC-0023 section 15: right-aligned, thousands separators, blank padding). If the value is no more than the balance (the party record's word at +0x14, read at draw time), it is drawn in the default caption colour (160, 160, 90, 127) (0x315A38). If it is above the balance, it is drawn in (128, 60, 90, 127). There is no separate cannot-pay label: the price is still shown, only in another colour.
- Line 2: its value in a nine-cell field, colour (128, 128, 128, 127) (0x315A58). There is no balance test.
- Writers:
  1. Buy prompt (SPEC-0050 section 9; SPEC-0022 chooser step 1): on each tick the prompt applies with no press, line 2 = the balance with counter 1, then line 1 = the item's price with counter 1 (replacing the old value). So the caption shows on exactly the ticks the prompt is up.
  2. Coin gain: every add of a coin item (ids 135 to 139; SPEC-0021, Coin pickups) with gain g: if line 1's counter is above 0, line 1 = old value + g; otherwise line 1 = g. In both cases the counter becomes 180. The wallet then grows by g and cue 0xA5 plays. So coins picked up in quick succession show one running total, which disappears 180 drawn passes (3 s of play with both panels closed) after the last coin. The coin add is a plain addition of the coin's value; a Tal id adds nothing.
- Clears: world teardown sets both values and both counters to 0. The menu and HUD reset (Select or Start opening a menu from play, and the game-over page) sets only line 1's counter to 0.
- Coin model (High for the gate, scale, spin, lighting and depth test; Medium-High for the screen position): while line 1's counter is nonzero and both quick panels are closed, the HUD's 3D pass draws the one-Neopoint coin model (item 135) spinning to the left of the caption number. It is a real 3D model in the main frame, drawn once per frame after the world and before the 2D HUD, and it is not a canvas sprite. The coin shows for exactly as long as line 1's counter is nonzero with both panels closed, and the spin does not advance on frames where it is not drawn. The counter only falls on passes that draw the caption line, so in a state that does not draw the caption the coin stays up while the counter is frozen.
  - Screen position. The coin's centre is anchored to a fraction of the screen, not to the caption: 0.65 of the screen width from the left and 0.24 of the screen height from the top. On the 512 by 448 canvas that is (332.8, 107.5). It is therefore 77 pixels left of the caption's right edge (x 410) and 7.5 pixels above the line-1 baseline (y 115). It does not move with the number's width. The anchor sits at a depth of 6.0 units straight ahead of the camera; the model's own origin is its centre (the model is symmetric about its origin), so the coin's centre is exactly the anchor point. Because the anchor is a screen fraction at a fixed depth, the coin follows the camera and is not a world object.
  - Projection check. The game's perspective maps a point at lateral offset X and depth Z to screen fraction 0.5 plus 0.8 X/Z across and 0.5 minus 1.0667 Y/Z down; the anchor is built from the same numbers, so the fractions above are the true screen fractions. (If the wide-screen option were ever on, the horizontal fraction would become (0.65 - 0.5) / 0.8 + 0.5 = 0.6875 and the vertical (0.24 - 0.5) / 1.0667 + 0.5 = 0.2563; no code that turns that option on is known.)
  - Scale. The model is scaled uniformly by 0.2625 divided by the length of the vector (low corner + size) / 2 of the model's bounding box, where the box is stored as a low corner and a size. For item 135 the low corner is (-0.14519, -0.62224, -0.62224) and the size is (0.29038, 1.24449, 1.24449), so that vector is (0.07259, 0.31112, 0.31112), its length is 0.44594 and the scale is 0.58864. The coin is a disc of radius 0.62224 x 0.58864 = 0.3663 units lying in the model's Y and Z axes, with a thickness of 0.29038 x 0.58864 = 0.1709 units along its X axis. At depth 6.0 the face-on disc is about 50 pixels wide and 58 pixels tall on the 512 by 448 canvas (half-width 25.0, half-height 29.2: the game's pixel aspect makes it 17 percent taller than wide), and edge-on it is about 11.7 pixels wide.
  - Spin. The coin turns about the screen's vertical axis (the camera's up axis; the model's Y axis) at 1.0 radian per second: the angle is the product of the rate 1.0 and a time value that starts at 0 and falls by one tick (1/60 s) on every drawn frame. After n drawn frames the angle is -n/60 radian, reduced modulo 2 pi; one turn takes 377 drawn frames (6.28 s). The time value is never reset, so the pose continues from where it stopped the last time the coin was drawn. The rotation maps the model's X axis to (cos a) times the camera's sideways axis minus (sin a) times the forward axis, the model's Z axis to (sin a) times sideways plus (cos a) times forward, and leaves Y alone. So the coin starts edge-on (its thin axis along the screen's horizontal) and is face-on when a = -pi/2 or +pi/2, i.e. about 94 drawn frames after the first one (pi/2 x 60) and every half turn after that. The direction of the turn is not visible on a symmetric disc (Low for the sign).
  - Lighting. It is lit, as a world item is: it is drawn with a scene light context made from the four scene default lights (SPEC-0046 Lights), and its material uses the lit vertex program (layout C, program 1), so its shading follows the model's rotation against those four fixed light directions. The area's own light set is not applied. World items are drawn the same way.
  - Depth. It is drawn with the depth test on (greater-or-equal, depth writes on, as for every material) against the depth the world just wrote in the same frame, and the Z buffer is not cleared between the world and this pass. So it is in the world, not over it: a wall, tree or character nearer than 6.0 units to the camera along that sight line hides it. Its alpha test is the usual one (reference 120, failing pixels written to the colour buffer only).
  - Test vector: first drawn frame, angle -0.01667 rad; centre (332.8, 107.5) on the 512 by 448 canvas; scale 0.58864; after 60 drawn frames the angle is exactly -1.0 rad and after 94 it is -1.567 rad (the first face-on pose).
  - Default if unresolved (rewrite choice): none needed for the placement, scale, spin and light, which are settled; use a depth-tested draw. If the camera is ever closer than 6.0 units to a wall the coin is hidden, as in the original.
- Edge cases: a coin total is drawn in (128, 60, 90, 127) when the balance is below the shown total within the 3 s. A buy prompt replaces a running coin total (and its counter becomes 1). A coin total of exactly 32767 would read "Exorbitant!" (not reached in practice).
- The caption is the buy-price and coin-gain display; no separate shop screen uses it.

### 16. Flag pairs, pools and layout walk (Confidence per item)

- **Complementary flags (Medium).** There are two front-end flags, A and B. Setting A stores whether B is zero, and setting B stores whether A is zero. So after either set, at most one of the two is 1. A query returns 1 when either flag is set. A third query returns B, unless a field of the global object is set, in which case it returns 0. Synthetic check: with A = 0 and B = 0, setting A makes A = 1, after which setting B makes B = 0.
- **Pool selection by mode (Medium).** A mode number selects one of several slot pools. Modes 1, 2, 5, 7, 8 and 9 each select a pool, and any other mode uses a default pool. A count query returns how many slots of the selected pool are free. A query for the nth free slot returns the nth free slot of the same selection.
- **Free list of 16 nodes (Medium).** A helper builds a list of 16 nodes. Each node stores its own index and a link to the next node. The helper sets the list's head and tail.
- **Layout walk (Medium).** A menu item's position is found by walking up its parent chain. At each node the walk adds the node's two signed 16-bit offsets (the position fields of section J). The walk ends when it reaches a node that has a flag byte set. Whether that flagged node's own offsets are added is not confirmed.

Which menu settings the flag pair represents, what the pools hold, and what the free list is for are open (Open questions 57 to 59).

### 17. Menu driver, focus search and menu helpers

Confidence: Medium for the handler codes. High for the focus search, the item position helper and the page open (the picture load). High for the identity of the driver's three values (a debug colour editor that never runs), the byte tween math, the cursor rectangle's identity and the nine-slice panel's identity and border words; Medium-High for the byte tween's reach. Where this section disagrees with Behavior 1, Behavior 4 or Behavior 12, the disagreement is listed in Open questions 60 to 72. The driver caps and the rectangle and quad sizes are not yet confirmed against the disc bytes.

#### 17.1 Menu driver (one call per menu tick)

- Each call adds 1 to a front-end frame counter, held in front-end data at 0x2EC8C4 (Medium).
- The driver keeps three adjustable values. While a raise bit is set in the held2 word (raw held for R1 and R2), its value goes up by 1 per call, and while a lower bit is set it goes down by 1 per call. So A rises on Down and falls on Up, B rises on Right and falls on Left, and C rises on R1 and falls on R2. Each value stops at 0 at the bottom and at its cap at the top (Medium):

| Value | Raise bit | Lower bit | Cap |
| --- | --- | --- | --- |
| A | 0x4000 | 0x1000 | 63 |
| B | 0x2000 | 0x8000 | 180 |
| C | 0x0008 | 0x0002 | 63 |

- The horizontal bits (0x2000, 0x8000) and the vertical bits (0x4000, 0x1000) also send a move to the focus search (17.2).
- **Pad masks and directions.** The bit values in this table are the same bits as the Behavior 4 table, with the Behavior 4 names (High): 0x4000 Down, 0x1000 Up, 0x8000 Left, 0x2000 Right. Value C's bits 0x0008 and 0x0002 act as R1 and R2 held.
- **Debug colour editor (High for reach).** Values A, B and C are the three channels of a debug colour editor. The editor runs only on a page that holds a kind-8 node, and no page on disc has one, so it never runs in the shipped game. When the editor opens, the values are loaded from the node's colour record with channels h, s and v: A = round((s - 1) / 4) - 1, B = round(h / 2), C = round(v / 4). When they are written back: h = 2B, s = 4A + 3, v = 4C.
- The current menu node's handler runs once per tick and returns a code:
  - -3: no change.
  - -2: close the menu.
  - -1: pop to the top of the stack.
  - any other value: push a child page (17.4).
  This is the same scheme as Behavior 1, step 5, seen after the generic step: a handler's 0 becomes -3 when the generic step does nothing, and the handler level's -4 only blocks the generic Cross and Triangle (Open question 61, resolved).

#### 17.2 Directional focus search

- Use: only spatial pages (section H byte +0x0A = 0; pages 858 and 896) use this search, and neither page is ever entered, so the shipped game never runs it (High; section 9d).
- Input: the focused item and a pressed direction: 0x8000 Left or 0x2000 Right (horizontal), 0x1000 Up or 0x4000 Down (vertical).
- Candidates (High): every kind-4 item among the page's items, in sibling order, except the focused item, whose node predicate passes. All items of the page take part; spatial pages have no scrolling.
- Reference point (High): the item position of 17.3 plus the item's pivot words (+0x1C, +0x1E). The candidate's y also gets (candidate index − focus index) × the parent's row-spacing byte (+0x2A). dx and dy are the candidate's point minus the focused item's point.
- Zero offset (High): a candidate whose offset along the search axis is exactly 0 is skipped.
- Wrap (High): a candidate behind the pressed direction uses the wrapped distance. For Left, dx > 0 becomes 512 − |dx|; for Right, dx < 0 becomes 512 − |dx|; for Up, dy > 0 becomes 448 − |dy|; for Down, dy < 0 becomes 448 − |dy|. A candidate ahead keeps its direct offset.
- Cross-axis weight (High): the offset across the search axis is multiplied by 4 before squaring when a nearness test passes. For a horizontal search the test uses each item's height (J +0x20, ha for the focused item and hb for the candidate): it passes when (ya ≤ yb or yb − hb ≤ ya + ha) and (yb ≤ ya or ya − ha ≤ yb + hb). A vertical search uses the widths (J +0x22) on x. With sizes of 0 or more the test always passes, so the cross term always weighs 16 times.
- Cost: horizontal = dx² + (4 dy)²; vertical = (4 dx)² + dy², with dx or dy along the axis already wrapped; the result is cut to a whole number toward zero.
- Result: the candidate with the lowest cost. A later candidate replaces the best only with a strictly lower cost, so ties keep the earlier item. The starting best is 1,234,567, so a cost of that or more is never chosen. The search returns the chosen item's index, or -1 when no candidate qualifies; then the caller uses the record's neighbour (section 9d).
- Test vectors (synthetic; focused item point (100, 100), sizes 16, row spacing 0):
  - Right, candidate at (140, 120): cost = 1,600 + 80² = 8,000.
  - Right, candidate at (60, 100): dx = −40 wraps to 472; cost = 222,784.
  - Right, candidate at (100, 150): dx = 0, skipped.
  - Up, candidates at (100, 60) and (130, 70): costs 1,600 and 120² + 900 = 15,300; the first is chosen.

#### 17.3 Item screen position

- The menu item record is the section J node. Its position for the focus search comes from a position helper: start from the item's own position (+0x14, +0x16), then add each ancestor's, walking up the parent chain, until a node with the absolute flag (+0x2D) is reached or the chain ends.
- Row spacing in this helper (High): at each ancestor with a non-zero row-spacing byte (+0x2A), the helper adds that byte times a sibling count. The count is the number of siblings before the child on the path, visible or not, and it is not reset between levels: the counts of lower levels carry into the higher ones. The draw's node transform (Section J) counts visible siblings instead, so the two can differ. This never matters on disc, because the search never runs.
- Horizontal alignment in this helper comes from the node byte +0x2E. Kinds 0, 3, 4 and 5 are never shifted. For the other kinds:
  - 1 (centre): left edge = x − trunc(width / 2), with width J +0x22 and no scale.
  - 2 (right): left edge = x − width − 1.
  - any other value: no shift.
- The sprite draw uses its own rule (Behavior 12): centre x − width × scaleX / 4096 / 2, right x − width × scaleX / 4096, other values no shift. The draw rule is the one that places every visible node; the helper's rule is reached only by the focus search, whose candidates are kind 4 and are never shifted (High).
- Test vectors: width 32, x = 100, scale 4096. Alignment 1: 84 in both rules. Alignment 2: 67 in the helper, 68 in the draw. Alignment 5: 100 in both.

#### 17.4 Menu page open

- This step is the full-screen picture load of Behavior 1a, not a page or sound step. It runs when a screen is entered (Behavior 1b, step 8) and when the root screen is set directly.
- It searches the new screen's tree for the first kind-1 sprite whose asset record is not loaded (kind word -1). It runs only when such a record is found, no load is pending, and that record is still not loaded. Otherwise it returns without change.
- It toggles the two-slot buffer index between 0 and 1. The previous user of the new slot gets kind word -1; the found record takes the slot (its kind word becomes the slot index) and the slot remembers the record and its resource word. The pending flag is set and the done flag is cleared.
- When an archive was given at front-end set-up, it reads 267,264 bytes of resource 575 + v into the slot's buffer. The read's completion callback only sets the done flag. No cue is played.
- Each pass, the driver polls the load first (Behavior 1a, item 6).

#### 17.5 Byte tween

- The byte tween is the colour kind (3) of the UI animation tracks of section M. It has four byte channels: start bytes at track +0x14 to +0x17 and end bytes at +0x18 to +0x1B. t0 and t1 are the floats at +0x0C and +0x10, and the duration is t1 − t0.
- t is the animation slot's local time in seconds, from a clock = front-end pass counter (section 17.1) / tick rate (60, written at front-end start; section 9b). t is not reduced by t0.
- Each channel = start + round_half_away((end − start) ÷ (t1 − t0) × t), stored as a byte (mod 256) (High).
- The four bytes are used as a tint multiplier (byte / 128) on the node's draw.
- Reach: the only colour tracks on disc are records 12 (alpha 128 -> 0 over 10 s) and 13 (alpha 0 -> 128 over 30 s). No animation slot reaches them, so the byte tween never runs in the shipped game (Medium-High).
- Test vectors: start 10, end 20, duration 4, t 1: 10 + round(2.5) = 13. Start 0, end 255, duration 10, t 5: round(127.5) = 128. Colour track 0 -> 128 with t0 = 0 and t1 = 30: t = 15 gives 64. Scale track (kind 2) 1.0 -> 0.85 over 1 s: t = 0.5 gives 0.925.

#### 17.6 Cursor rectangle (debug colour editor marker)

- It is not a list cursor. It is the value marker of the debug colour editor panel (section 17.1), drawn for kind-8 nodes only. No page on disc has a kind-8 node, so it is never drawn in the shipped game (High).
- It builds a rectangle element of 0x40 bytes, 16 units wide; its left edge is the item's x, and its top is the caller's base y + 0x40 − the value at 0x2EC8C8. That word is the editor's value channel, not a list scroll.
- Colour is white (0xFFFFFF). Alpha is 0x80, which on the PS2 scale is fully opaque (SPEC-0003, Decode one indexed texture to RGBA, step 3).
- Test vector: base y 100, value 30 gives a top edge of 134.

#### 17.7 Nine-slice panel (dialogue box frame)

- It is the dialogue and message box frame of SPEC-0022 (Dialogue box layout), drawn only by the box update (High).
- It builds nine quads in the standard nine-slice arrangement around the box. Each quad is one record of 0x70 bytes, and the call returns 0x3F0 bytes (nine records).
- The border words at 0x2EC91C to 0x2EC930 hold 15, 28, 14 and 26 (the borders) and 64, 64 (the sprite size, 64 x 64), as on disc (High). The quad placement is the one given in SPEC-0022. Coordinates are scaled by 16 into screen units (Section 10).

## Screen build guide (front end and main menu)

Confidence: High for every position, size, scale and alignment value (all come from the front-end database; the node transform of section J gives the placement), High for flow, inputs, timings and sounds (they restate the sections above), Medium where a line says so. This guide is for building the screens from nothing. It does not repeat the full rules; it names the section that owns each rule.

### G.0 Conventions used by every element table

- All positions are canvas pixels, 512 x 448, origin top-left, y down (section 10). A text position is the baseline of its first line and glyphs stand above it. A sprite position is its top-left corner after alignment.
- Placement of a node = parent origin + node offset; a child's offset is multiplied by the scale of every ancestor group (section J, Node transform). A node's own scale changes only what it draws, and its alignment uses its own scaled width.
- Alignment: "centre" means the text or sprite is centred on the stated x; "right" means its right edge is at the stated x; "left" means its left edge is there. Text is measured at draw time, so a centred or right-aligned label is placed by its real width (section 12, Text node draw).
- Label colours: the neutral label uses colour (128, 128, 128, 128) (the glyph sheet colour unchanged) and the highlight label uses (200, 115, 50, 128), an orange tint. A highlight sprite uses its sheet colour unchanged.
- Every list item of the main menus is the same four-part group, drawn in this order: highlight bar (a sprite, drawn only on the focused item), neutral label (always), highlight label (only on the focused item, same position as the neutral label, drawn after it so the orange text lands on top of the neutral text). Unfocused items show the neutral label only.
- The highlight bar is the first frame (cell 0) of the 2-column, 4-row selector sheet, 128 x 32 texels, drawn at its stated size. Nothing ever changes its frame (the frame setter has no caller), so it does not animate. It is placed 19 pixels above the item baseline (20 on the Audio page). Resolved: the focus marker is one static bar plus the orange label, with no blink, slide or pulse. (High)
- Instant swaps: no front-end page change fades, slides or wipes. The new page is drawn on the next pass and the old one is gone (Behavior 1b; the only fades are those of SPEC-0026, Loading screen and transition fades). (Medium-High)
- Inputs act on a new press (rising edge), never repeat while held, except the three volume sliders. Stick directions count as D-pad presses once per crossing of the 32 / 224 thresholds (section 4). Only pad 0 is read.
- The first two passes of every screen ignore generic Cross, Triangle and focus moves (Behavior 1b, step 7). A page that is entered by a Cross press therefore cannot be left or confirmed by the same press, and a held direction does not move the new page's focus.
- Sounds: focus move cue 0x9A (Down and Up), select cue 0x98, back cue 0x99, played by the generic step at fixed full volume (section 3c).

### G.1 Boot timeline, from power on to the first menu

Passes are front-end passes at 60 per second. Screens in order, with the picture each draws:

| Step | Screen (record, state) | Draws | Length | Leaves when |
| --- | --- | --- | --- | --- |
| 1 | Card check (0, state 0) | Nothing visible: the frame stays black, apart from the card message box when one is open (section G.1a) | First visit: until the boot card job ends (no fixed pass count; a quick job ends after the box's minimum hold) | Controller usable and card job not busy |
| 2 | Logo (30, state 29) | Full-screen Neopets logo picture (resource 600) | 301 passes (5.02 s) | Pass 300 |
| 3 | Legal (1, state 1) | Full-screen legal text picture (resource 577) | 301 passes on the first visit since boot, 2 passes on later visits | Wait ends; counts only while the controller check is ready |
| 4 | Movie 0, then movie 86 | Full-frame video, 512 x 448 | Until each ends or is skipped (Cross or Start held for 5 counted frames; SPEC-0026, Movie skip) | Both movies played |
| 5 | Title (10, state 12) | Full-screen title picture (resource 585) and the three-item menu | Until a choice, or 1,801 passes with no new press | New Game, Load Game, Options, or idle timeout (back to step 2) |

Worked example (cold boot, pad 0 plugged in, no card). Pass 0: the card check selects nothing yet and starts the boot card job; a message box with the "no card inserted" text (record 1, labels Retry and Continue) opens. The black screen shows only that box until the player presses Circle (Retry: the job restarts) or Triangle (Continue: the job ends with no card). Two passes after the answer the box returns, the card check leaves, and the logo shows for 301 passes with no input read. The legal picture shows for 301 passes (Cross is ignored). Movie 0 plays; the player holds Cross for 5 frames and it ends; movie 86 then plays at once (a skip ends only the current movie); then the title appears with New Game focused, music bank 35 playing. If a card with enough space is present, step 1 shows no box at all and lasts one pass beyond the job.

Later visits of the card check (after Quit, game over without a saved game, or the finale credits) last one pass, start no card job and show no box. Default if unresolved (rewrite choice): a rewrite with no memory card skips step 1 entirely and begins at the logo.

#### G.1a Card check page and the controller message

- Elements: three black squares (64 x 64) along the top edge at x 0, 200 and 400 (three more fall below the canvas), invisible on a black frame; the message box of SPEC-0022 (Dialogue box layout) is the only visible element. Resolved: the page has no art, no title and no text of its own. (High)
- Controller message: while pad 0 is not usable, the same box opens with record 25 ("Analog controller not detected", no labels) on every screen that runs the controller check (section 2b). The card check does nothing else until the pad is back, the legal page stops counting, and the credits roll stops drawing and scrolling. In play the loss pauses the game (section 6). The box closes by itself when the pad is usable again.
- Box placement for any message (SPEC-0022): for a text wrapped to L lines at width 410, the first baseline is at y = 345 - 10 L; the frame sits at x = 30, spans 446 pixels, starts 41 pixels above the first baseline and is L x 21 + 45 high, tinted with the record colour at alpha 45 of 128. The label row is drawn after the text is fully shown: second label (Triangle icon) centred near x = 128, first label (Circle icon) near x = 384, baseline = first baseline + 21 L + 1.

#### G.1b Logo and legal pages

- Both pages hold one 512 x 448 sprite at (0, 0) and nothing else: no text, no hints, no items, no input. They stay up for the stated passes whatever is pressed; Triangle is off.
- Neither page changes the music or sound state (the music controller does not tick before the first title pass), and there is no fade into or out of either.
- Both pages select pad 0 as the only menu source on pass 0. (High)

### G.2 Title screen (record 10, state 12)

Entry: from the second movie. Exit: New Game (to play), Load Game (record 17), Options (record 11), or idle timeout (record 30).

| Element | Art | Text | Position (canvas) | Notes |
| --- | --- | --- | --- | --- |
| Backdrop | Title picture, resource 585 | - | (0, 0), 512 x 448 | Drawn first. It carries the game logo; the screen has no "Press Start" text. |
| Item 0 highlight bar | Selector sheet cell 0 | - | x 179 to 333 (153.6 wide), y 291 to 317 (25.6 high) | Focused item only |
| Item 0 label | - | "New Game", font 0, spacing -2 | centre x 256, baseline 310 | Neutral, plus orange copy when focused |
| Item 1 highlight bar | same | - | x 179 to 333, y 322 to 348 | |
| Item 1 label | - | "Load Game", font 0, spacing -2 | centre x 256, baseline 341 | |
| Item 2 highlight bar | same | - | x 179 to 333, y 353 to 379 | |
| Item 2 label | - | "Options", font 0, spacing -2 | centre x 256, baseline 372 | |

Rows are 31 pixels apart. There is no hint line and no version text.

State machine and inputs (sections 2, 4.1, 9d):
- Entry: focus on New Game (index 0), pass counter 0, idle counter 0, music bank override 25 (bank resource 35) from pass 0 on; the stream gate is closed on pass 0.
- Down / Up (new press, D-pad or left stick): next / previous item, wrapping (New Game, Load Game, Options, then back to New Game); cue 0x9A. Left and Right do nothing.
- Cross on New Game from page pass 5: start the new game (section G.9). Before pass 5 nothing happens and no cue plays.
- Cross on Load Game or Options: cue 0x98, the page is pushed (stack: title below it). From pass 2 on.
- Start, Select, Triangle, Circle, Square, L1, R1, L2, R2: nothing.
- Idle: any pass on which no new button or stick crossing arrives adds 1 to the idle counter; any new press resets it to 0; at more than 1,800 the screen leaves to the logo page (pass 1,801). A held button does not hold the timeout off. The idle counter keeps running while the controller message is up.

Worked example. Pass 0 to 4: title drawn, New Game bar and orange label shown, music plays. Pass 3: the player presses Cross: nothing (not yet pass 5). Pass 6: Down: focus to Load Game, cue 0x9A, the bar and orange label move to y 322 and 341. Pass 7: Down again: Options. Pass 8: Down: wraps to New Game. Pass 9: Cross: cue 0x9A, the world resets and the first area loads (section G.9). Alternatively at pass 7, Cross on Options: cue 0x98, the Options page (G.4) appears on the next pass with the first row focused; Triangle there returns to the title with New Game focused, because no page remembers its focus.

### G.3 Pause page (record 2, state 7)

Entry: Select in play (cue 0x94, pushed over play). The world is frozen from the second pass on (section 2a). Music drops to the pause level (section 3b).

| Element | Art | Text | Position | Notes |
| --- | --- | --- | --- | --- |
| Backdrop | Options picture, resource 578 | - | (0, 0), 512 x 448 | The world is covered while this picture is drawn. Default if unresolved (rewrite choice): draw it opaque. |
| Item 0 | bar x 362 to 490, y 61 to 87 (128 x 25.6) | "View Map", font 0 | right edge x 470, baseline 80 | |
| Item 1 | bar x 324 to 490, y 96 to 122 (166.4 wide) | "View Quests" | right edge 470, baseline 115 | |
| Item 2 | bar x 336 to 490, y 131 to 157 (153.6 wide) | "Load Game" | right edge 470, baseline 150 | Always shown |
| Item 3 | bar x 317 to 490, y 166 to 192 (172.8 wide) | "Resume Game" | right edge 470, baseline 185 | |
| Item 4 | bar x 375 to 490, y 201 to 227 (115.2 wide) | "Options" | right edge 470, baseline 220 | |
| Item 5 | bar x 413 to 490, y 236 to 255 (76.8 x 19.2) | "Quit" | right edge 470, baseline 255 | |

Rows are 35 pixels apart; all labels use font 0 with spacing -2; there is no hint line. The bar and orange label sit on the focused row only. Item 5's bar is shorter because its y scale is 0.6.

Inputs: Up / Down move the focus with wrap (cue 0x9A, from pass 2). Cross acts per item: View Map and View Quests open the map or journal with no generic cue (the item states handle Cross themselves; View Map does nothing when the hero's area has no map), Load Game opens the load picker (0x98), Resume Game returns to play (0x95), Options opens the Options page (0x98), Quit opens the quit box (0x9A). Select (box closed) returns to play (0x95). Triangle (box closed) returns to play (0x99, generic back). Start, Circle and Square do nothing.

Quit box: the shared message box with system record 26 (SPEC-0025, Card message records), first label "Quit" (Circle), second label "Resume" (Triangle). While it is open all menu input is blocked except the box's own buttons. Circle: cue 0x98, the world is torn down, nothing is saved, and the front end goes to the card check page (G.1; a later visit, so one pass), then the logo, legal page, movies and title. Triangle: cue 0x99, the box closes, the pause page stays with the focus unchanged (Quit).

Worked example. Play, Select pressed: cue 0x94; pause page with View Map focused (bar at y 61, orange label at baseline 80). Down x5: the focus walks to Quit (cue 0x9A each; at the sixth Down it would wrap to View Map). Cross: cue 0x9A, the box opens over the page. Triangle: cue 0x99, the box closes. Up: Resume Game focused. Cross: cue 0x95, play resumes with the music back at the area level and no fade.

### G.4 Options pages (records 11, 12, 13; no state)

Entry: Cross on Options from the title or the pause page (cue 0x98). All three pages are list pages; the rows' own states (13 to 17) run only while their row has focus.

Options page (record 11), backdrop resource 586 at (0, 0):

| Element | Position | Notes |
| --- | --- | --- |
| "Controls" | centre x 256, baseline 178, bar 179 to 333 at y 159 | item 0 |
| "Audio" | centre x 256, baseline 220, bar at y 201 | item 1 |
| "Credits" | centre x 256, baseline 262, bar at y 243 | item 2 |
| Cross icon (32 x 32) and "Select" (font 1, spacing 0) | icon (65, 392); text left edge x 100, baseline 415 | Always shown |
| Triangle icon (32 x 32) and "Back" (font 1) | icon (359, 392); text left edge x 393, baseline 415 | Always shown |

Controls page (record 12), backdrop resource 587. Rows are 44 apart; each label is right-aligned at x 302 (font 0, spacing 0) and each value is left-aligned at x 322 (font 1, spacing 0):

| Row | Baseline | Label | Value shown | Highlight bar right edge, width |
| --- | --- | --- | --- | --- |
| 0 | 188 | "Vibration" | "On" when the Vibration word is nonzero, else "Off" | 306, 153.6 |
| 1 | 232 | "Invert Left-Right" | "Yes" when the word is nonzero, else "No" | 306, 230.4 |
| 2 | 276 | "Invert Up-Down" | "Yes" / "No" | 306, 217.6 |

Bars sit 19 pixels above each baseline, 25.6 high. A "Back" hint (Triangle icon, same place as on the Options page) is the only hint.

Audio page (record 13), backdrop resource 588. Rows are 44 apart; each label is right-aligned at x 246 (font 0, spacing -2); each row has a slider bar sprite (131 x 27) at x 256, y = baseline - 23, and a knob sprite (9 x 32) at x = 256 + k, y = baseline - 28, where k = trunc(value x 1.29 + 2):

| Row | Baseline | Label | Bar y | Knob y | Highlight bar right edge, width, top |
| --- | --- | --- | --- | --- | --- |
| 0 | 203 | "Music" | 180 | 175 | 251, 76.8, 183 |
| 1 | 247 | "Sound effects" | 224 | 219 | 251, 179.2, 227 |
| 2 | 291 | "Voice" | 268 | 263 | 251, 76.8, 271 |

Value 0 puts the knob at x 258, value 75 at 354 and value 100 at 387 (the bar's right end; the knob then overhangs it by 9). Bar and knob are always drawn on all three rows; the knob has no highlight copy. Only "Back" is hinted.

Inputs:
- Up / Down: next / previous row, wrapping, cue 0x9A. On the Options page Cross opens Controls, Audio or Credits (cue 0x98); Triangle goes back (cue 0x99) to whichever screen opened Options, with that screen's default focus (title: New Game; pause: View Map).
- Controls: Left or Right (new press, either) flips the focused row once; no sound. Vibration changes only its word. The invert rows also run the apply rule on every pass while focused (section 3), which is why they refresh the three audio gains and knobs.
- Audio: while Right is held the focused value rises 1.0 per pass (full travel 100 passes, 1.67 s); while Left is held it falls 1.0 per pass; both held, Right wins; clamp 0 to 100; no sound. Every pass of a focused slider row re-applies the gains.
- The knob draws one pass behind the value (one-pass lag, Behavior 1b). The first time the Audio page is ever shown, the knobs are drawn at their stored x of 2 (value-0 position) for one pass and then jump to the positions for the current values; later visits keep them in place.

Worked example. From the title: Cross on Options, Down, Cross: the Audio page; Music is focused with the knob at x 354 (value 75). Hold Right for 12 passes: value 87, knob k = trunc(87 x 1.29 + 2) = 114, x 370; the gains become 0.87 on the first pass of the hold and from then on; no sound plays for the change. Release, Down: Sound effects focused (cue 0x9A). Hold Left for 100 passes: value 75 falls to 0, knob at x 258. Triangle: cue 0x99, back to the Options page (focus Controls). The options are kept in the live block and copied into any slot that is saved.

### G.5 Credits (record 14, state 18)

Entry: Cross on Credits in Options (cue 0x98), or the finale gate. Backdrop: credits picture, resource 589, full screen. The rows are drawn over it by the state, not by the tree.

- Row style: font 0, spacing -2, centred on x 256, baseline = the row's y. Rows are 24 pixels apart; the first row's baseline starts at y 449 (just below the bottom edge) and rises 1.0 to 1.5 pixels per pass (SPEC-0026, Credits). A row fades in over its top 32 pixels (alpha 128 x y / 32 when y is below 32) and out over the bottom 32 (alpha 128 x (448 - y) / 32 when y is above 416).
- Row colours, by style and colour set from the table's third field (set 0 or 1), RGB (the fourth byte is the fade alpha):

| Style | Set 0 | Set 1 | Used for |
| --- | --- | --- | --- |
| 0, 1, 2, 4, 7 | (127, 125, 81) | (100, 100, 190) | Title line, company names, role headings, the asterisk divider rows |
| 3 | (190, 188, 122) | (120, 160, 200) | Person names (287 rows) |
| 5, 6 | - | - | Blank rows that change the scroll speed |

- Content: the first row is the title "Neopets: The Darkest Faerie" (style 0, set 1), then two blank rows, then the publisher line (set 0), the studio name (style 1) and role / name pairs (role style 2 above its names style 3). After the Sony and Santa Monica block comes a row of 31 asterisks, then "Idol Minds" in set 1. The table has 623 rows including 169 blank rows; the roll ends when the last row has scrolled off, 14,123 passes after the start. The text comes from the credits table of the design tables (SPEC-0024, Table 15); do not copy it into the rewrite.
- Hints: none. Music: bank 18 (override 8), replacing the title music.
- Inputs: from the Options page Triangle (after page pass 2) leaves early (cue 0x99) to Options; the end of the roll also returns to Options (no cue). From the finale gate all input is ignored; the end tears the world down and returns to the card check page.

Worked example (speed 1.0 at the start, so the first row's baseline is 449 minus the pass number). Pass 0: the credits picture and the title row at y 449, alpha 0 (invisible). Pass 10: the title row is at y 439, alpha 128 x 9 / 32 = 36. Pass 33: y 416, full alpha 128. Pass 417: y 32, still full; pass 433: y 16, alpha 64. Pass 469: the row has fallen below y -20 and is dropped, and the next row becomes the first. Each later row is 24 pixels below the one before it, so a new row starts to appear every 24 passes while the speed is 1.0.

### G.6 Load picker (record 17, state 21)

Entry: Cross on Load Game from the title or the pause page (cue 0x98). The picture is the load-game picture (resource 590). The only text in the tree is the caption "Load Game" (font 0, spacing -1, centre x 256, baseline 40). There is no hint line on this screen.

Stages (section 7.2 and SPEC-0025): stage 0 starts the load scan (busy box "Loading, do not remove the card" for at least 123 passes, then the file read); stage 1 shows any card box (no card, no save data, load failed, cancel confirm); stage 2 shows the four slot rows. While stages 0 and 1 run Triangle does nothing; in stage 2 Triangle goes back (cue 0x99).

Rows (section 7.1 and the journal row drawer, section 14.3): four rows stacked from the top, the first row's reference at y 77, each row 89 pixels high when it shows two lines and 79 when the second line is blank (an empty slot; Medium: a blank second line draws no objective line, as a treasure row does). The row parts use the fixed offsets of the row table in section 14.3: left cap (95, 77), right cap (429, 77), bar (104, 83) 334 x 26 in colour (78, 119, 174), act icon at (51, 84), line 1 at (118, 104) in font 0 and line 2 at (118, 128) in font 2. A filled slot shows the area name on line 1 and the date text (month.day.year hour:minute) on line 2, with the act icon for its progress tier; an empty slot shows "Empty" and no second line. The cursor row's bar pulses brighter (section 14.3).

Cursor and keys: start on the first filled slot (slot 0 if filled); with no filled slot the cursor is 4 and nothing is highlighted. Up / Down move to the previous / next filled slot, skipping empty ones, and cue 0x9A plays on every press even when the cursor cannot move. Cross loads the cursor slot (no menu cue; the world load follows with the white flash of SPEC-0026). A card removal while the picker is up opens the card-gone box.

Worked example. Slots 0 and 2 filled, 1 and 3 empty. Entering: busy box for 123 passes, then the rows: row 0 at p = 0, row 1 ("Empty") at p = 89, row 2 at p = 168, row 3 ("Empty") at p = 257, ending at 336. The cursor is on row 0 (bar bright). Down: the cursor skips row 1 and lands on row 2 (cue 0x9A). Down again: no filled slot below, the cursor stays (cue still plays). Cross: slot 2 loads and the game resumes.

### G.7 Save picker (record 18, state 22)

Entry: Square within 5 units of a save point (SPEC-0025, Save points). The picture is the save-game picture (resource 592, identical to the load picture). Elements: the caption "Save Game" (font 0, spacing -1, centre x 256, baseline 40), the same row table as the load picker, and the hint line drawn by the screen: Cross icon (65, 392) with "Select" (left edge 100, baseline 415) and Triangle icon (359, 392) with "Back" (left edge 393, baseline 415), font 1.

Stages (section 7.3): scan, card boxes, picker, write, result. Up / Down move one slot with no skipping, bounded 0 to 3, cue 0x9A on every press. Cross on a slot: question box (record 7 overwrite when the slot is filled, record 10 save to the card when empty, none on a card just formatted), busy box (record 5, at least 123 passes), write, then cue 0xA9 and return to play with the music overrides cleared. Triangle in the picker: cue 0x99, return to play with nothing written. A failed write opens record 6 or 3 with Retry (back to the scan and picker) and Cancel save (cancel-confirm record 9).

Worked example. Card present, slot 0 filled, others empty, the player at a save point. Square: the save screen opens in the same pass as the press and the hero stops; on a good card the scan shows no box, and the four rows appear when the scan job ends. Down, Down: cursor on slot 2 (cue 0x9A twice). Cross: the "Save to the card in slot 1?" box (record 10), Circle (Yes, cue 0x98): the busy box shows for at least 123 passes, the write finishes, cue 0xA9 plays and play resumes. Slot 0 is untouched. Repeating on slot 0 would show the overwrite question (record 7).

Clean-side note: with no memory card, the scan, format and card-gone boxes never appear; keep the picker, the overwrite or save question, the busy text and the failure box (SPEC-0025, Clean-side mapping of the card jobs).

### G.8 Game-over page (record 23, state 26)

Entry: the game-over flag in play (section 6). Elements: the game-over banner picture (resource 594, 512 x 128) at (0, 140), centred horizontally and drawn from pass 0; from pass 121 on, the Cross icon (32 x 32) at (345, 392) with the caption "Continue" (font 1) at left edge 377, baseline 416. There is no full-screen backdrop, so the frozen world and the letterbox bars of SPEC-0026 stay visible around the banner (Medium). No menu, no focus, no sound on entry.

Input: only Cross, as a new press, from page pass 122 (about 2.03 s); a Cross held since before does not count. Then: the game-over flag clears; with a pending save snapshot (a slot was saved or loaded earlier this session) play resumes from it with no card access; otherwise the world is torn down and the front end returns to the card check page, the logo, the legal page, movies and the title (SPEC-0055 for timing).

Worked example. Hero dies, banner appears (pass 0). Cross pressed on pass 30: nothing. Pass 121: the caption and Cross icon appear (the Cross press still has no effect before pass 122). Cross released and pressed on pass 125: if the player had saved at a save point earlier, play resumes from the snapshot; if not, the title sequence starts again from the card check.

### G.9 New game flow, up to gameplay

1. Title, Cross on New Game at page pass 5 or later: cue 0x9A.
2. The world resets (all flags cleared), both start-point overrides clear, the start hero is selected, flag 707 is set, and the two start packs load (691 into the transition buffer, 909 as the current exterior). The four music overrides clear.
3. The front end returns to play (record 9). Nothing is drawn by the front end during the load: no loading art, no text. The title picture is gone from the next pass and the frame is black until the world and its intro are ready (Medium; Default if unresolved (rewrite choice): clear to black).
4. The area's script plays the opening movie 88, then movie 1 (SPEC-0026, First play on a new game). The movies are skippable like the attract movies. Control starts after them. There are no lead-in screens, no hero choice and no name entry: both heroes exist and the start hero is fixed (SPEC-0053).
5. Load Game from the title follows G.6 and enters play with the white fade-in of SPEC-0026, in the saved area.

### G.10 What the screens never show

No save entry in the pause menu, no autosave, no "press Start" text, no continue item on the title (Load Game is the continue), no difficulty or language menu, no subtitles option, no screen-adjust page, and no confirmation when leaving the Options pages. The start-point chooser, the three-slot chooser, the lead-in pages and the "Coming Soon" page exist in the data but nothing enters them (section 2, reach).

## Screen build guide (item screen, journal, map page, book reader)

Confidence: High for positions, sizes, art, colours, inputs, sounds and order (they come from the front-end database and the state handlers); Medium where a line says so. Conventions are those of G.0 (canvas pixels, baselines for text, centred or left or right alignment by real width). For a centred sprite the position given here is its centre unless "left edge" is said. There are no fades or slides on any of these pages: a page change shows the new page on the next pass. The pages reachable from the pause page are View Map (G.16), View Quests (G.15), Load Game (G.6), Resume, Options (G.4) and Quit (G.3). There is no status, stats, equipment or map-list page: the item screen (reached with Start in play, not from the pause page) is the only inventory view, and the pause page has no item-screen entry.

### G.11 Item screen: shared frame (states 8, 9 and 10; records 3 to 8)

Entry. Start in play (code for the current tab and hero, section 14.4), or L1 or R1 on another tab, or L2 on the map, or R2 on the journal. Exit: Start or Triangle (cue 0x95, back to play, no fade; the four music overrides are cleared), L2 or R2 (page ring, G.11 inputs). The world is frozen from the second pass (section 2a). On entry the HUD visibility timers are held at 240 on every pass, so the HUD bars stay fully shown.

Pictures (full canvas, drawn first): Items tab Roberta record 3 (resource 579), Tormund record 6 (582); Charms tab Roberta 4 (580), Tormund 7 (583); Quest tab Roberta 5 (581), Tormund 8 (584). The hero is the active hero (section 14.4). All other elements are shared by both heroes and are drawn by the tab state, in this order: (1) the cell group (key 0xC2; Items and Quest tabs only), (2) the NeoPoints group (key 0xBF; all tabs), (3) the Mote group (key 0xC0; Charms tab only), (4) the HUD bar group (key 0x31) at offset (60, 23), (5) the two item text lines and the cell counts, (6) the controller message when pad 0 is not usable (section 2b). The screen picture is always under all of them.

Elements drawn on every tab:

| Element | Art | Text / value | Position (canvas) | Notes |
| --- | --- | --- | --- | --- |
| NeoPoints label | - | "NeoPoints", font 1, spacing 0 | centred on x 102, baseline 324 | neutral colour |
| NeoPoints value | - | the party balance, font 1 | centred on x 102, baseline 342 | neutral colour |
| Equipped strip, weapon | 64 x 64 icon | - | left edge (91, 113) | alpha 128 when present, 0 when empty (section 14.1) |
| Equipped strip, body | 64 x 64 icon | - | left edge (93, 174) | hidden on the Items tab while the cloak is owned |
| Equipped strip, shield | 64 x 64 icon | - | left edge (91, 235) | Tor only |
| Luck meter | 25 clover leaves, 23 x 29 (leaf 4 pieces: 26 x 28) | - | see below | leaf k shown when the hero's luck is above k |
| L2 tab | 42 x 60 | - | left edge (0, 181) | always |
| R2 tab | 42 x 60 | - | left edge (469, 181) | always |
| Name line | - | the selected entry's name, font 0 | left edge 63, baseline 388 | colour (164, 156, 102, 128), one line |
| Description line | - | the selected entry's description, font 1 | left edge 63, baseline 407 | same colour, one line, never wrapped |
| Exit hint | Triangle icon 32 x 32 | "Exit", font 1, spacing -1 | icon left edge (385, 330), text left edge 412, baseline 352 | always (Items and Quest tabs; the Charms tab draws it at the same place) |

Luck meter. The leaf order from the lowest (luck 1) to the highest, as left-edge positions: leaf 0 (61, 257) piece 4; 1 (58, 252) piece 3; 2 (51, 248) piece 2; 3 (46, 241) piece 1; 4 (54, 235) piece 2; 5 (60, 231) piece 3; 6 (61, 222) piece 4; 7 (58, 217); 8 (51, 213); 9 (46, 206); 10 (54, 197); 11 (60, 193); 12 (61, 179); 13 (58, 174); 14 (51, 170); 15 (46, 163); 16 (54, 152); 17 (60, 148); 18 (61, 136); 19 (58, 131); 20 (51, 127); 21 (46, 120); 22 (51, 113); 23 (54, 109); 24 (57, 110). Pieces repeat in the order 4, 3, 2, 1, 2, 3 (leaves 22 to 24 are 2, 3, 4). Luck is an integer from 0 to 24, so leaf 24 is never shown. A hero with luck 0 shows no leaf. (High)

Inputs (all tabs; rising edges of the menu word, one action per pass in this priority): Start or Triangle closes (cue 0x95); else L1 moves to the previous tab (cue 0x98); else R1 to the next (cue 0x99); else L2 (cue 0x98) opens the journal; else R2 (cue 0x99) opens the map of the hero map area, or the journal when that area is negative. So L1 held with L2 only changes the tab. After a tab change the state returns the tab's code and the new tab's record shows on the next pass. The first pass after an entry applies the cursor rules of section 14.1. Controller removed: the controller message of section 2b is drawn on top and the page keeps running.

### G.12 Items tab (state 8)

The cell group is a 4 x 5 grid. Cell k (0 to 19; row = k / 5, column = k mod 5) uses these positions (centres): icon centre x = 208 + 51 column; icon top y = 141, 188, 237, 286 for rows 0 to 3 (icon 32 x 32, left edge = centre - 16); "equipped" frame (42 x 42) top y = 136, 184, 233, 281; cursor frame (58 x 56) top y = 130, 178, 227, 275 and left edge 179 + 51 column. Draw order inside the group: equipped frames, then the icons, then the cursor frame, then the potency badges, then the Use / Activate / Exit boxes and the cloak group.

| Part | Art | Shown when | Notes |
| --- | --- | --- | --- |
| Equipped frame | red circle highlight, 42 x 42, centred on the icon centre | the cell id equals the hero's quick-item marker | alpha 128 |
| Icon | the cell's own consumable art (cell 0 Red Juppie 150, 1 Chokato 146, 2 Peach 156, 3 Kauvara 58, 4 Baggus 145, 5 Purple Juppie 149, 6 Starberry 157, 7 Ergyfruit 147, 8 Fyora's Potion 59, 9 Ointment of Curing 54, 10 Speckled Negg 153, 11 Striped Negg 155, 12 Silver Negg 152, 13 Power Potion 55, 14 Meerca Potion 56, 15 Spyder Potion 51, 16 Quickness Ointment 53, 17 Dispelling Potion 52, 18 Starlight Potion 57, 19 Revival Potion 50) | owned count above 0 (alpha 128, else alpha 0) | exact ids are the list of section 14.1 |
| Count | number text, font 1, colour (128, 128, 128, 128) | owned count above 1 | centred on the icon centre x (not left of 16), baseline = icon top + 40 (8 pixels under the icon); one decimal number, no "x" |
| Cursor frame | selection frame art, 58 x 56, centred on the icon centre | cell = cursor | always drawn, also on an empty cell |
| Health badge | health icon, 16 x 16 times the scale | owned count above 0; cells 0 to 3 (scale 0.7, 0.9, 1.0, 1.2) and, as a second badge, cells 10, 11, 12 (scale 1.0) | top-left (216, 160), (266, 160), (317, 160), (364, 160) for cells 0 to 3; (209, 255), (263, 255), (317, 255) for cells 10 to 12 |
| Magic badge | magic icon, 16 x 16 times the scale | owned count above 0; cells 5 to 8 (scale 0.7, 0.9, 1.0, 1.2) and cells 10, 11, 12 (0.7, 0.9, 1.0) | top-left (188, 207), (241, 207), (291, 207), (339, 207) for cells 5 to 8; (188, 255), (237, 255), (288, 255) for cells 10 to 12 |
| Use box | Cross icon 32 x 32 at left edge (181, 330); "Use" font 1 spacing -1, left edge 209, baseline 352 | the selected cell has an owned entry that is not a quest item | |
| Activate / Deactivate box | Circle icon at left edge (258, 330); "Activate" or "Deactivate", left edge 285, baseline 352 | same test; "Deactivate" when the cell id equals the marker | |
| Exit box | see G.11 | always | |
| Cloak group, no cloak | ring art 64 x 64, left edge (92, 177) | the hero owns no Tattered Cloak entry | behind the body icon |
| Cloak group, cloak owned | two rings 64 x 64 at left edges (105, 177) and (82, 177) holding the cloak's icon and the current body item's icon, and the Square icon (32 x 32 at 0.75, left edge (155, 199)) | the hero owns the cloak entry | the middle strip icon is hidden meanwhile |

A badge draws only for an owned cell: the owned count is the condition, not the id. Cells 4, 9 and 13 to 19 have no badge. (High)

Name and description lines show the selected cell (section 14.1 rules). Buttons: Cross use (cue 0xA2 when denied), Circle marker (cue 0x9A; the set also plays 0xA3), Square cloak swap (cue 0xA1, only with the cloak). Directions: every press plays the move cue of its direction (Down, Up, Right, Left; vertical wins) whether or not the cursor changes, because the grid has 20 cells.

Worked example (fresh game, Tor, Items tab, cursor 0). Draw: Tor's items picture; NeoPoints 0 at x 102; middle strip icon with the Farmer's Clothes art at alpha 128; luck leaves for Tor's luck; cell 0 shows the Red Juppie icon with a "3" centred at (208, 181) and a health badge, cell 5 the Purple Juppie icon with a "3" at (208, 228) and a magic badge, 18 cells empty with no icon, no count and no badge; the cursor frame is on cell 0 at (179, 130); the Use box and the Activate box show; the name line reads "Red Juppie (3)". Down: cue 0x9A, cursor frame to cell 5 at (179, 178), the name line "Purple Juppie (3)" and the description line "Replenishes a small amount of magic." Down: cell 10, empty: cursor frame still shown, the Use and Activate boxes vanish, both lines show a single space. Circle on cell 5: cue 0x9A then 0xA3, an equipped frame appears behind cell 5 and the box reads "Deactivate". L2: cue 0x98, the journal opens.

### G.13 Charms tab (state 9)

Draw list: the picture, the NeoPoints group, the Mote group, the HUD bars and the text lines. The cell group of G.12 is not drawn. The Mote panel (key 0xC0) shows 14 Motes in a ring around the centre (240, 144). Each Mote has: an element icon (32 x 32, left edge listed below), a selection ring (72 x 80 at scale 0.72, left edge listed) drawn only on the selected Mote, and a count text (font 1, spacing -3, centred, baseline listed) drawn only on the selected Mote and only while the selection timer is above 0. An icon is shown when the party owns at least one of that Mote (owned count above 0). With no selection (value 255) the single ring at left edge (285, 204) is drawn.

| Mote | Icon left edge | Ring left edge | Count centre x, baseline |
| --- | --- | --- | --- |
| 0 | (327, 233) | (316, 221) | (334, 272) |
| 1 | (363, 251) | (352, 238) | (368, 291) |
| 2 | (258, 194) | (248, 183) | (264, 231) |
| 3 | (223, 177) | (214, 165) | (230, 215) |
| 4 | (258, 232) | (248, 221) | (266, 273) |
| 5 | (223, 249) | (213, 238) | (230, 288) |
| 6 | (327, 194) | (318, 180) | (334, 232) |
| 7 | (362, 178) | (353, 166) | (370, 215) |
| 8 | (294, 251) | (282, 241) | (300, 289) |
| 9 | (293, 288) | (282, 277) | (298, 329) |
| 10 | (293, 176) | (281, 164) | (298, 216) |
| 11 | (293, 138) | (282, 126) | (298, 177) |
| 12 | (214, 213) | (204, 201) | (220, 250) |
| 13 | (372, 213) | (362, 201) | (378, 252) |

Slot overlay. Three equipment slots draw the equipped Mote over the strip: word 2 (weapon, top) at centre (132, 135) with its count at (140, 175); word 0 (body, middle) at centre (132, 187), count (140, 227); word 1 (shield, bottom) at centre (132, 242), count (140, 282) (icons 32 x 32, counts font 1 centred). A slot draws when the item in that equipment word has a count above 0. Button hints sit beside the strip (scale 0.75): Circle icon at left edge (85, 138) for the weapon slot, Cross icon (85, 199) for the body slot, Square icon (85, 258) for the shield slot (this Square hint is drawn for Tor only; the Circle and Cross hints follow whether the matching strip entry is present). A Triangle "Exit" hint is at the place of G.11.

The name and description lines show the selected Mote (section 14.1). Buttons are the Mote picker's (section 14.1 and SPEC-0021). The Mote picker's cursor sound follows SPEC-0021. L1 goes to the Items tab (cue 0x98), R1 to the Quest tab (cue 0x99). The tab returns to the dispatch code when the tab word is not 1.

### G.14 Quest-items tab (state 10)

Draw list: the picture, the cell group, the NeoPoints group, the HUD bars, the text lines. The 20 icon sprites and cell positions are those of G.12, but the icon art of cell k is picked by item id: the k-th owned category-7 record (slot order, first 20 only) picks child number "item id" of the icon group (ids 0 to 159 have art), alpha 128; cells beyond the owned count have alpha 0. No count number, no potency badge and no "equipped" frame is drawn on this tab; the cursor frame is drawn on the cell under the cursor, also when that cell is empty. Cross and Circle both play cue 0xA2 (no item is usable here), and only the Exit box shows (no Use and no Activate box). The cursor rule, wrap and sound are those of section 14.1. Name and description show the selected quest item; the longest on the disc is 68 characters. L1 goes to the Charms tab (cue 0x98), R1 to the Items tab (cue 0x99).

Worked example (Act 1, three quest items owned, ids 0, 1 and 2 in slots 0, 1 and 2). Cells 0 to 2 show their art; the cursor is on cell 0. Right: cue (right-move), the cursor frame to cell 1; Right: cell 2; Right: cell 3 is not owned so the step repeats and wraps to cell 0 (same row). Down: the target cell 5 is empty, the step repeats and wraps to cell 0 again (rows 1 to 3 are empty), so the cursor returns to cell 0 with the move cue played. Cross: cue 0xA2.

### G.15 Quest journal (record 22, state 25)

Entry. Select-page route: View Quests on the pause page (Cross), the item screen (L2), or the map (Cross on an icon with items, or R2). Exit: Triangle (cue 0x99, to play, because the record's back rule is 9), L2 (cue 0x99, map of the hero map area, or the item screen when it is negative), R2 (cue 0x98, the item screen), Cross (map of the row's area). Picture: resource 593 (full canvas).

Fixed elements: up arrow (32 x 16) left edge (237, 16) shown when the top row is not row 0; down arrow, the same art flipped vertically, with its top-left reference at (237, 432), shown when the more-below flag is set (Medium for the flipped extent: it covers y 416 to 432); L2 and R2 tabs as in G.11; "Exit" hint (Triangle icon left edge (359, 400); text left edge 393, baseline 423, font 1, spacing 0); the Cross icon (left edge (65, 400)) with the text "View Map" (left edge 100, baseline 423, font 1), drawn only while the selected row has an area; the empty message, shown when the row count is 0: the lines "You Do Not Currently Have" (centred on x 256, baseline 180) and "Any Quests Available" (centred on x 256, baseline 210), font 0, neutral. Left and right arrow sprites exist in the page data but are never drawn.

Row parts (positions in the table of section 14.3): left bracket (95, 77), bar (104, 83, width 334, height 26), right bracket (429, 77), icon (49 or 51, 84), title baseline 104, objective baseline 128 (font 2, wrap 316, line pitch 14). Rows are laid out down the page starting with the top row at the first row offset 0 and each next row after max(79, 75 + 14 L). Journal row geometry, colours, pulse and scroll are in section 14.3.

Inputs (main rising-edge word unless noted): Down moves the cursor down (clamped at the last row), cue is the Down move cue every press, even clamped; Up the same upward (clamped at 0, Up cue); each press sets the pulse to 1.8. The top index moves one row per tick toward the cursor (section 14.3). Cross on a row with an area: sets the selected map area to the row's area, plays cue 0x98 and opens the map. L2 and R2 (second word): see Entry. Triangle: back to play.

Quirk (reproduce, High): Cross on a quest row hands no pending target to the map, so the map cursor starts on icon 0. Cross on a treasure row does hand its treasure to the map, and the map cursor starts on that treasure's icon. (The journal sets the row target and then the treasure target in one step, and the second setter clears the first.) The map-to-journal direction is not affected: the journal cursor starts on the row of the icon that was current.

Worked example (Act 1 start; one accepted quest row with a 2-line objective and one held, uncollected treasure). Pass 0: the page shows no rows. Pass 1: row 0 at p = 0 (bar y 83, title baseline 104, 1 or 2 objective lines at 128 and 142), row 1 at p = 103 (if 2 lines: next row offset 75 + 28 = 103), the cursor on row 0 with the pulsing bar colour (222, 160, 1) x pulse; "Cross View Map" hint if row 0 has an area. Down: cue, cursor on row 1, the pulse restarts at 1.8, the Cross hint follows row 1. Cross on row 1 (treasure with area 3): cue 0x98, map of area 3 opens with the cursor on the treasure icon. Triangle on the map: back to play.

### G.16 Map page (records 31 to 107, state 30; SPEC-0051)

Entry: View Map, the D-pad Left shortcut, the page ring, or a journal row. Pictures: resource 601 + area, full canvas (area 17 also carries its own centred name label at (256, 40) under the overlay's copy of the same name). Tick order and exits: SPEC-0051 section 4. Draw order on a stay pass: picture, the hero marker (only when the hero is in the displayed area), the icons in list order with the cursor icon last-pulsed, the cursor icon's label, then the overlay group. On the pass that exits through Select nothing but the picture is drawn (no marker, no icons, no overlay) and the four music overrides are cleared (section 3a). On the pass that exits through Cross, R2 or L2 the marker and icons were already drawn that pass, and the area name and overlay are not.

Overlay elements (centre unless noted): Triangle icon (left edge (359, 400)) and "Exit" (left edge 393, baseline 423, font 1); Cross icon (left edge (65, 400)) and "View Quest" (left edge 100, baseline 423, font 1), only while the current icon has items; "Treasures:" (font 2, left edge 332, baseline 60), the collected count (font 2, right-aligned on x 402), the word "of" (left edge 424) and the area total (left edge 447), only while the area total is not 0; the area name (font 0, centred on x 256, baseline 40, colour (127, 125, 81, 128)); the L2 and R2 tabs of G.11. The icon arts are 26 x 32 (primary quest), 26 x 29 (secondary quest) and 31 x 31 (treasure spot), centred on the projected point.

Worked example (area 3 page, a hero in the area facing +Z, one quest icon and one held treasure). Pass 0: build the list (quest icon first, treasure second), cursor on icon 0. The marker brightens and dims (102 to 192) as the float pulse runs; the cursor icon swells 3900 to 4300 over the integer pulse; the label of the cursor icon is drawn 14 above it. Right: the cursor moves to the treasure icon (no cue). Cross: cue 0x98, the journal opens with the treasure pending and its row selected. L2 there: cue 0x99, back to this map (hero map area).

### G.17 Book reader (record 20, state 24)

Entry: the play gate when the book cursor is 0 or more (section 6.1). The world is frozen under the page; the reader has no picture of its own: the page driver draws the book's flat page and 3D page sheet over the frozen frame (rules 11 and 12 of 6.1). Elements: the title (centred on x 256, baseline 40, font 0, colour (112, 112, 2, 128)); the page text blocks (rule 9 of 6.1); the Cross icon (32 x 32, left edge (391, 399)) with "Close" (left edge 421, baseline 423, font 1). The Cross icon and "Close" are drawn from the first pass, even while the book loads. Inputs: Cross closes (cue 0x9A, section 6.1 rule 6); Triangle bounces (rule 7); nothing else. There is no page turn and no scroll.

Worked example (station value 82 opens index 81): Square edge in the station: cue 0xAF; next play pass returns to the reader; the title appears at baseline 40; the page sheet tilts over 75 passes from 1.2 to 0.12 radians; Cross: cue 0x9A, play resumes with the world unfrozen and no fade.

### G.18 Guess points left open on these pages

Default if unresolved (rewrite choice): a long description draws on one line and is cut by the canvas edge (the longest quest-item description is 68 characters; the longest Mote description is longer and uses colour codes, Medium); the flipped journal arrow draws as the same art mirrored vertically at y 416 to 432; the opaque picture under every page; a hero with no quick-item marker draws no frame.

## Menu screens and items as observed

Items are counted from the labels bound to each page's subtree (neutral-colour bindings only). "Nodes" counts all nodes in the page's subtree, including containers and sprites.

| Page root | Screen (name used here) | Nodes | Labelled items | Notes |
| --- | --- | --- | --- | --- |
| 14 | In-game pause menu | 25 | 6 | View Map (0), View Quests (1), Load Game (2), Resume Game (3), Options (4), Quit (5). No Save Game item. |
| 759 | Title menu | 13 | 3 | New Game, Load Game, Options |
| 774 | Options menu | 13 | 3 | Controls (0), Audio (1), Credits (2) |
| 794 | Controls menu | 25 | 7 distinct (9 bindings) | Vibration, Invert Left-Right and Invert Up-Down, Vibration with On and Off, and each invert row with Yes and No (section 3). Row order top to bottom: Vibration, Invert Left-Right, Invert Up-Down (High; Behavior 3, Row draw order). |
| 824 | Audio menu | 25 | 3 | Music, Sound effects, Voice |
| 858 | Start-point chooser | 28 | 9 | Four "Act N Start" items and five progress items (Act 1 Squire Done, Act 1 Chores Done, Act 3 Middle, Act 3 Entering Dungeons, Act 4 Districts Done). A string in the table marks this screen as debug-only. |
| 896 | Save/load slot chooser | 10 | 3 | Game 1, Game 2, Game 3. No code opens this page and the picker does not draw these labels; they are a leftover (High; Open question 45) |

Totals: 139 nodes across the seven page trees, and 32 distinct labels.

Labels outside these seven trees: 52 strings have a neutral binding; 32 of them appear under the pages above. The other 20 sit in node trees that are not reached from the seven pages. Their screens are not identified. They include the inventory and item actions (use, activate, deactivate, exit, close, select, back), a sword and money label, save and continue, next, treasure and location text, and quest text. The three remaining table strings have no neutral-colour binding.

Option values (music, sound effects, voice, vibration, inversion) exist as labels here. Their stored values are the option block in Layout and the rules in Behavior 3.

## Edge cases and error handling

- Version other than 1.05: refuse the table; the front end does not start.
- Menu stack push at depth 31: refused (no change). Pop at the root: nothing popped.
- Record stack overflow: a push at depth 32 leaves the stack unchanged and writes the overflow report; 32 pushes succeed from an empty stack (Medium).
- Parent index of any other negative value (not -1, -2 or -3): a log message is written and the current page block is pushed, not the page itself (Medium-High).
- Parent index 0 or more naming a page that is not on the screen stack: the stack is reset to that page record alone, and the page is pushed (Medium).
- Condition chain with a node that has no handler and mode 2: P = 1, so the test passes. The same node with mode 3 fails. The drawing path and the kind 1 scan do not do this: they use the nearest handler instead (Section 9b).
- Key 0xFFFF in a key search: matches the first unkeyed node (key field -1) it reaches, which may not be the node the caller wanted (Medium).
- Sprite frame request at or above the sum of counts: ignored, the previous frame stays.
- Sprite with all-zero counts (779 records): every frame request is ignored. Whether such sprites draw at all is open.
- Section N: read its count from header +0x4C (20 slots of 32 bytes). The header word +0x3C (585) is not N's count and does not divide the section size.
- Section M and N are not needed for drawing the menus shown above.
- The start-point screen is described in the strings as debug-only. It is never entered (High, static survey; Behavior 2).
- Generic input is ignored on the first two passes of every screen (Behavior 1b, step 7), so a Cross, Triangle or direction pressed on the pass that opens a screen, or on the next one, does nothing on the new screen.
- A Cross target of -1 (the start-point items) plays sound slot 5 twice in one pass: once for Cross and once in the back branch. Not reachable on disc.
- Quirk (reproduce; Medium-High): Triangle in the book reader is not handled by the reader, so the back rule returns to play, and play's book gate reopens the reader on the next pass. Cue 0x99 plays, the book stays loaded and the page angle is not reset (section 6.1).
- A handler's own -1 return (the credits at the end of their scroll from Options, the load screen after a card-box cancel) plays no driver cue; a generic Triangle plays cue 0x99.
- While the pause page's quit box is open, Triangle answers the box (Resume) and the pause page stays; the generic step and the focus states see no input.
- Toggle with a stored value other than 0 or 1: a change stores 1, because the rule is "1 if the old value was not 1" (Behavior 3).
- Volume: adjustments are clamped to 0 to 100. The apply rule clamps the gain to 0.0 to 1.0.
- Quirk (reproduce): from a fresh boot the gains are 1.0 while the sliders show 75; the first visit to the Audio page (or focus on an invert row, or a load) drops them to 0.75 with no input (Options, Gains at start-up).
- Sound effects 0: once the default cue volume has been refreshed to 0, every default-volume cue is refused (not started); front-end menu sounds still play at 1024 (Behavior 3c).
- Stick inversion: the complement maps 0 to 255 and 255 to 0, so a centred byte of 0x80 becomes 0x7F, not 0x80. It applies to the right-stick bytes only.
- Load with no filled slot: the starting cursor is 4, outside the slot range (Behavior 7.2). A move finds no filled slot in its direction, so the cursor stays at 4 (Behavior 7.1), and Cross copies nothing. Apart from that starting value, the movement rule never leaves the range 0 to 3.
- Picker row text follows the slot's key only: a slot with key -1 shows the empty text even if its name field holds other text. A filled slot whose name field is empty shows a blank first line (Medium).
- Start index above 8: the handler's parameter range is 0 to 8. The behaviour for other values is not established.
- Book reader with no valid index (book cursor 0 with a saved word of -1 or 0): the reader leaves at once (section 6.1).
- Volume flag word without bit 6, or with any of bits 1, 2, 3 or 8: does not set the pending book index (section 6.1). On the disc every book station has flag word exactly 0x0040.
- Table B has 31 valid states (indices 0 to 30). The field that selects a table B entry covers 32 values, of which the 32nd is not a valid state.
- Item screen, empty cell: Cross plays cue 0xA2 and changes nothing. Circle on a cell whose count is 0 is not established (Open questions).
- Item screen, Square with no category-9 item: nothing happens, with no cue.
- Item screen, Circle on the cell whose id is the quick-item marker: the marker is cleared to -1 (cue 0x9A only), not replaced. This is the same id test as the "equipped" frame and the Activate/Deactivate label. No equipment word changes.
- Item screen, Quest-items tab: Cross and Circle always play cue 0xA2, because every entry there is category 7.
- Item screen, count of 1: the cell shows the icon and no number. Only counts above 1 show text.
- Journal: a quest row whose start or done id is -1 is never listed. A row whose bottom offset is 360 or more is not drawn; a row ending exactly at 360 is hidden but still counts as fully shown, so the cursor on it does not scroll it into view (section 14.3). With an empty list, Down sets the cursor to -1 and Up sets it back to 0. Cross on a row without an area does nothing. With no rows the list group is hidden and the no-quests message is shown.
- Entry code group: the hero index is compared with 1 only. Any other value uses group A (Tor's pages).
- Map shortcut with a negative hero map area: D-pad Left in play still plays cue 0x9A and stays in play; R2 on the item screen opens the journal; L2 on the journal opens the item screen; View Map does nothing.
- Overlay cell exactly on a margin edge (canvas x = -10 or 522, or y = -10 or 458): not drawn, because the bounds are strict (Section 15.1).
- Price caption with its counter at 0: that line is not shown. A pass that does not draw a line (in play with a quick panel open) does not lower its counter.
- Price caption with a price needing more than nine characters: not established (SPEC-0023 Open question 31).
- Map projection table (14.5): a read before the start-up write sees all-zero records. A current area index of 77 or more is not confirmed by the reader; a negative index is checked, and its result is not established (Open question 51).
- Menu cursor with no selectable item (no item of kind 4): what the step returns is not established (section 4.1).

## Test vectors

- Frame groups (1, 4, 8): total 13. Set frame 12: accepted (12 < 13). Set frame 13: ignored.
- Audio slider node positions (disc data): Music, Sound Effects and Voice each have a stored knob-node x position of 2 before the first apply. The default value 75 applies x = trunc(75 x 1.29 + 2) = 98 to each knob. The knob's left edge on screen is 256 + that x, so 354 at the default value and 258 before the first apply (section G.4).
- Label colour (synthetic): binding colour (200, 115, 50, 128) with no animation draws (200, 115, 50, 128). With tint bytes (64, 128, 255, 128) the factors are 0.5, 1.0, 1.9921875 and 1.0, giving (100, 115, 100, 128): 50 × 1.9921875 = 99.6, which rounds to 100. Colour 255 with tint byte 255 gives 508.0, stored as byte 252.
- Label font (synthetic): a binding with font id -1 draws nothing.
- Label spacing (synthetic): letters of advance 8, spacing -2: at scale 1 each glyph moves the pen 6; at scale 0.5 it moves (8 - 1) × 0.5 = 3.5.
- Section C lookup (disc data, values only): key 0x9F maps to J node 1000, key 0x31 maps to J node 615, and key 0x19 maps to J node 929. A key of 0x1FF, which no record lists, gives -1.
- Section C duplicate: a table built from the records (0xD1, 1194), (0x31, 615), (0xD1, 970), (0xD1, 186) gives 186 for key 0xD1, and -1 for any other unlisted key.
- Section K record 0 (disc data, values only): word 0 is 0x00008F44, word 1 is 0x80808080, word 2 is 0x00010000 (low 0, high 1), word 3 is 0x00080000 (low 0, high 8) and word 4 is 0. Record 3: word 0 is 0x0000A910, word 2 is 0x00010003 (low 3, high 1), word 3 is 0x0103FFFD (low -3, high 259).
- Section K relocation (synthetic): with a load base of 0x1000, a record whose word 0 is 0x20 becomes 0x1020; words 1 to 4 are left as they are.
- Frame groups (0, 0, 0): any frame is ignored.
- Frame groups (0, 10, 0): total 10. Set frame 9: accepted. Set frame 10: ignored.
- Position at scale 1.0: node at x = 0, y = 0. X_screen = (0 + 2048 - 256) x 16 = 28672. Y_screen = ((0 - 224) x 1.0 + 2048) x 16 = 29184.
- Centre alignment, width 32, scale 4096, x = 100: left edge = 100 - 16 = 84.
- Right alignment, width 32, scale 4096, x = 100: left edge = 100 - 32 = 68.
- Width 32 at scale 8192 (2.0): displayed width = 32 x 8192 / 4096 = 64.
- Stack: 31 pushes succeed; the 32nd push is refused. Pops then return screens in reverse order, and a pop after the root returns nothing.

Record stack (Medium): from an empty record stack, 32 pushes succeed, the 33rd push leaves the depth at 32 and writes the overflow report, and each new top record is a copy of the one before it.
- Layout check: for each section, start of the next section minus start of this section equals the section size, and the size divided by the record size equals the header count. For N the count is header +0x4C (640 / 32 = 20); the tail is 3 x 272 + 282 x 18 = 5,892 bytes.
- Toggle: stored 0 becomes 1; stored 1 becomes 0; stored 2 becomes 1.
- Defaults after start-up: Music, Sound effects and Voice 75.0 each; gains 1.0 each until the first apply; after the Audio page's Music row first runs, gains 0.75 each and knob x 98 each; Invert Left-Right 0; Invert Up-Down 0; Vibration 1; cycler 1; unlabelled toggle 0.
- Fresh boot, row Level 700, Audio page never opened: bank target volume 700. After visiting the Audio page (no changes): 525.
- Fresh boot, row Scale 1024: default cue volume 1024 (also on every pass before the first title pass). After visiting the Audio page: 768.
- Volume: value 40.0 with Right held for one tick becomes 41.0. Value 100 with Right held stays 100. Value 0 with Left held stays 0. Value 0 with Right and Left both held becomes 1.
- Knob x for values 0, 1, 10, 25, 50, 75, 99.5 and 100: 2, 3, 14, 34, 66, 98, 130 and 131.
- Gain for values 0, 25, 75 and 100: 0.0, 0.25, 0.75 and 1.0.
- Gain use (SPEC-0009): Music 75 with a music row Level of 700 gives a bank target volume of 525 (1/1024 units). Sound effects 75 with a row Cap of 700 gives a stream fade-in ceiling of 525.
- Apply rule: volume 50 gives gain 0.5 and knob x = int(50 x 1.29 + 2) = int(66.5) = 66. Volume 100 gives gain 1.0 and knob x = int(131) = 131. Volume 0 gives gain 0.0 and knob x = 2.
- Toggle, one press: Vibration 1 with Right pressed gives 0; a following Left press gives 1. Invert toggle, one press with its argument 1: +0x00 changes from 0 to 1.
- Cycler, one press each: value 1 with Right gives 2; value 2 with Right gives 0; value 0 with Left gives 2; value 1 with Left gives 0; value 1 with Left and Right pressed together gives 1.
- Fade (state 28): Music gain 0.75 after one fade tick gives 0.70, and after another gives 0.65. Gain 0.02 after one fade tick gives 0.0.
- Load: live Music 75 and Vibration 1; the slot holds Music 20 and Vibration 0. After the load, live Music is 20, its knob x is 27 (int(20 x 1.29 + 2) = int(27.8)), its gain is 0.20, and the live Vibration word is 0. The load itself does not write the rumble gate; the display sync copies the live Vibration word 0 into the gate on the next pass, so rumble is off from then on (see Vibration and the pad).
- Vibration gate: with Vibration 0, the gate is 0 on every pass, so the motor bytes of every pad are cleared on each packet. A Vibration change during play reaches the gate on the next pass (Behavior 1 and Options).
- Black-box test BV-1 (Vibration toggle in play; to be run on the original disc on hardware with a rumble pad): start a fresh boot with Vibration On and trigger a rumble cue in play; the pad rumbles. Open the Controls menu and set Vibration to Off; return to play and trigger the same cue. Expected from this section: the pad stops rumbling on the next pass after the toggle (Medium-High). The gate is written after the pad read within each pass, so if the pad still rumbles on that pass, record it as a finding; a stop one pass later is also consistent with the rule. Record the pass-by-pass result, not only the final state.
- Stick inversion (right-stick bytes only): with Invert Left-Right on, right-stick X byte 0x10 becomes 0xEF (239). With Invert Up-Down on, right-stick Y byte 0x80 becomes 0x7F. With both on, right X 0x00 becomes 0xFF and right Y 0xFF becomes 0x00. A left-stick byte of 0x10 stays 0x10 under either option.
- Analog bits: vertical stick byte 0x1F sets Up; 0x20 sets nothing; 0xE0 sets nothing; 0xE1 sets Down.
- Load default: slot 0 empty, slot 2 non-empty: cursor starts at 2.
- Load starting cursor by key A of slots 0 to 3: keys [-1, 5, -1, -1] give cursor 1; keys [5, -1, -1, -1] give cursor 0; keys [-1, -1, -1, -1] give cursor 4, and Cross then copies nothing.
- Save success: after the write job reports done, cue 0xA9 (name index 169, save complete) plays and the save screen returns.
- Save cancel: Triangle in save state 2 writes nothing, plays front-end menu sound 6, clears the music overrides and returns code 0x0B4.
- Save open: the hero within 5 units of a save point presses Square while idle, cursor previously 2. The save screen (record 18) runs in the same tick with save stage 0 and cursor 0; the hero is set to state 1 on every pass while the screen is open.
- Save result: after the write, card state word 0 in state 3 gives cue 0xA9 and code 0x0B4; card state word 1 gives stage 0, no return, and a new card scan on the next pass followed by the picker.
- Pending flag: no save or load yet in the session, game-over Continue goes to code 0x000 (teardown). After one save (successful or not), game-over Continue re-applies the snapshot from memory and goes to code 0x0B4.
- Prompt codes: record 17 x 20 = 340 = 0x154 (load prompt, state 21); record 18 x 20 = 360 = 0x168 (save prompt, state 22).
- Cue index rule: cue 0x94 is name index 148 (pause), 0x95 is 149 (unpause), 0xA8 is 168 (save-point activation), 0xA9 is 169 (save complete).
- Load cursor, filled slots {0, 2}: from cursor 0, Down gives 2 (slot 1 is empty and is skipped). From cursor 2, Up gives 0.
- Load cursor, filled {2}: from cursor 0, Down gives 2. From cursor 2, Down stays at 2 (slot 3 is empty and no filled slot lies below it).
- Load cursor, filled {0}: from cursor 0, Down stays at 0 (slots 1, 2 and 3 are empty).
- Load cursor, filled {1}: from cursor 3, Up gives 1 (slot 2 is empty and skipped).
- Load cursor, filled {0, 3}: from cursor 0, Down gives 3 (slots 1 and 2 skipped). From cursor 3, Up gives 0.
- Load cursor, filled {0, 1, 2, 3}: from cursor 1, Down gives 2 (no skipping).
- Load cursor, filled {1, 2}: from cursor 0, Up stays at 0 (the move would leave the range).
- Load cursor, no filled slot: from cursor 2, Down stays at 2.
- Save cursor (no skipping, bounded): from cursor 0, Down gives 1 whatever is filled. From cursor 3, Down stays at 3. From cursor 1, Up gives 0. From cursor 2, Down gives 3 even when slot 3 is empty.
- Menu cursor (section 4.1), items of kinds 4, 1 and 4 in that order, cursor on the first: Down (0x4000) moves to the next selectable item, the third, and the middle item is skipped. From the third, Down wraps to the first. From the first, Up (0x1000) wraps to the third.
- Picker row, empty slot (key -1): line 1 reads Empty, line 2 is blank, and the length value is 7.
- Picker row, filled slot whose byte at +0x2FBA is 10: the length value is 12; line 1 is the location name and line 2 is the date text.
- Start index 2: stage ids 0 and 1 set, cue 0x40 played.
- Item tabs: on the Items tab, L1 gives tab 2 with cue 0x98; on the Quest items tab, R1 gives tab 0 with cue 0x99. On the Charms tab, R1 gives tab 2 with cue 0x99.
- Item grid: the Items tab has 20 cells, cells 0 to 19, in four rows of five. From cell 19 (row 3, column 4), Right wraps to cell 15 (row 3, column 0), and Down wraps to cell 4 (row 0, column 4). The wrap is within the grid. From cell 0, Up gives 15 and Left gives 4.
- Pause page: with the item index 0 to 5 in order, the page's item 2 is Load Game. A page item of index 1 is View Quests (focus state 3), which opens the journal on Cross. Item 0, View Map (state 2), with the hero map area 8: Cross returns 0x26C + 20 x 8 = 0x30C (record 39). Item 3, Resume Game (state 5): Cross plays cue 0x95 and returns 0x0B4. Item 5, Quit (state 6): Cross plays cue 0x9A and opens the quit box.
- Count text: a count of 0 hides the cell; a count of 1 shows no number; a count of 2 shows "2" in the neutral grey.
- Cross on an empty cell: cue 0xA2, no other change.
- Items tab, cell id 150 owned, marker -1, Circle: the marker becomes 150, cues 0xA3 and 0x9A play. Circle again: the marker becomes -1 and only cue 0x9A plays. No equipment word changes.
- Square with the category-9 index empty or out of range: nothing happens.
- Square on the Items tab, Tor active, Tor's body entry 121 and both cloak entries 126: cue 0xA1; Tor's body entry becomes 126 and his cloak entry 121 (the cloak is worn). A second Square restores 121 to the body entry.
- Entry codes: the Items tab with hero index 1 (Roberta) returns 0x3C (record 3); the Charms tab with hero index 0 (Tor) returns 0x8C (record 7).
- Journal rows: a quest row with start flag set and done flag clear is listed; with its done flag set, or with a flag id of -1, it is not. A held, uncollected treasure with area -1 is not listed.
- Journal bar colour: selected main-quest row at pulse 1.8 gives (255, 255, 2) (222 x 1.8 and 160 x 1.8 clamp to 255; 1 x 1.8 = 1.8 rounds to 2). Selected side-quest row at pulse 1.2 gives (186, 186, 155). Unselected rows keep their base colours.
- Journal pulse: value 1.79 with step +0.02: 1.81 is above 1.8, so the value becomes 1.8 and the step becomes -0.02. Value 1.21 with step -0.02: 1.19 is below 1.2, so the value becomes 1.2 and the step becomes +0.02. A cursor move at any value sets 1.8 and leaves the step as it is.
- Journal cursor: 3 rows, cursor 2, Down: the cursor stays 2. Cursor 0, Up: stays 0.
- Journal layout: rows with objectives of 1, 1 and 2 lines, then a treasure row, from top index 0: bottom offsets 89, 178, 281 and 360. The treasure row is not drawn, but with the cursor on it the top index stays 0. The first row's line limit is floor((360 - 128) / 14) + 2 = 18.
- Journal Cross: quest row 13 (scene 47) selected: the map page of area 47 opens (code 0x26C + 20 x 47 = 0x600, record 78), and the map cursor starts on that row's icon if it is listed. Quest row 27 (scene -1): Cross does nothing and the "Cross View Map" hint is hidden.
- Map (SPEC-0051): icons A (quest, has items) and B (treasure, has items), cursor on B, Cross: the journal opens with B's treasure pending. Empty icon list: Cross does nothing, R2 opens the journal with nothing pending, L2 goes to the item screen.
- Overlay cell: canvas x = -9 is drawn; x = -10 is not; x = 521 is drawn; x = 522 is not. Canvas y = 457 is drawn; y = 458 is not.
- Price caption: with line 1's counter above 0 and a value of 200000, the first line reads "  200,000" (two spaces, nine characters), right edge at x 410, y 115.
- Price caption colours: price 150, balance 200: line 1 "      150" in (160, 160, 90, 127), right edge x 410, y 115; line 2 "      200" in (128, 128, 128, 127), y 140. Price 300, balance 200: "      300" in (128, 60, 90, 127). Price 32767: "Exorbitant!" in (128, 60, 90, 127).
- Coin total: coins of 25 and then 10, picked up 60 passes apart: line 1 shows 25, then 35 with the counter at 180; the line is gone 180 drawn passes after the second coin.

- Play gates (synthetic): with story bit 368 and flag E both set, the pass returns 0x118. The gate order puts bit 368 first.
- Play gates (synthetic): flag E set, no story bit, book cursor -1, with Select or Start pressed, returns 0x1CC.
- Play gates (synthetic): book cursor 0 (the word at +0x54), no story bit, flag E clear: returns 0x190 before any input is read. The reader then computes index -1; with the saved word at -1 or 0 the fallback is still negative, so the reader leaves at once and returns 0x0B4 (section 6.1).
- Play gates (synthetic): no story bit, book cursor -1, flag E clear, Select pressed, busy check zero: returns 0x028 and plays cue 0x94.
- Item screen entry (synthetic): Start pressed, busy check zero. Tab 0 with hero index 1 returns 0x3C; tab 1 with hero index 0 returns 0x8C; tab 2 with hero index 0 returns 0xA0.
- Page ring (synthetic): item screen, hero map area 5, R2: cue 0x99, returns 0x26C + 100 = 0x2D0 (record 36). The same with hero map area -1: returns 0x1B8 (journal). Journal, L2, hero map area -1: cue 0x99, returns the item-screen code for the current tab and hero.
- Map entry (synthetic): D-pad Left pressed in play (second word 0x8000), no overlay busy, shortcut gate passing, no menu sub-state, hero map area 3: cue 0x9A, returns 0x26C + 60 = 0x2A8 (record 34), and entering that record loads picture resource 601 + 3 = 604. With hero map area -1: cue 0x9A and the state stays in play. A left push of the stick alone (main word only) does nothing.
- Journal entry (synthetic): D-pad Right pressed in play under the same gates: cue 0x9A, returns 0x1B8.
- Picture rule (synthetic): a section O record with kind word -1 and +0x0C = 10 names resource 585. A first picture load goes into one buffer, the next into the other, the third into the first again, and at that moment the first picture's record goes back to kind word -1, so re-entering its screen reads it again; each read is 267,264 bytes into a 267,776-byte buffer.
- Picture rule (synthetic): a section O record with kind word 2 is a sheet asset; entering its screen starts no picture load.
- Overlay box busy (synthetic): the box word at +0x258 is 1, no gate fires, no input. The pass returns 0x17C. On the next pass, with the box idle, the state plays cue 0x97 and returns 0x0B4.

Navigation (each press made at least two passes after its screen opened; stacks listed from the base):
- Play, Select: cue 0x94, pause page, stack (9, 2). Cross on Options: cue 0x98, stack (9, 2, 11). Cross on Controls: cue 0x98, stack (9, 2, 11, 12). Triangle: cue 0x99, the Options page with focus on Controls. Triangle: the pause page with focus on View Map. Triangle: play, stack (9).
- Pause page, focus 5 (Quit), Down: focus 0, cue 0x9A. Focus 0, Up: focus 5, cue 0x9A.
- Pause page, Cross on View Map with hero map area 8: map record 39, stack (9, 39), no cue. Triangle: play, cue 0x99.
- Title, Down: focus 1 (Load Game). Cross: the load screen, record 17, stack (10, 17), cue 0x98. After the card read, in the picker, Triangle: the title with focus 0 (New Game), cue 0x99.
- Tor, first Start since boot: record 6 (Items tab). R1: record 7 (Charms), cue 0x99. Triangle: play, cue 0x95. Start: record 7 again (the tab is kept).
- Controls page, focus 1 (Invert Left-Right), Right press: option word +0x00 toggles; focus 2, Right press: +0x04 toggles.
- Load screen during the card read (stage 0 or 1), Triangle: nothing happens (the state returns -4).
- Pause page with the quit box open, Triangle: cue 0x99, the box closes, the pause page stays, focus unchanged. Circle instead: cue 0x98, world teardown, record 0.
- A press of Cross on the first or second pass of a screen: ignored by the generic step.
- Logo countdown (state 29): pass 0 loads 300; pass 299 leaves the countdown at 1 and stays; pass 300 gives 0 and returns 0x014. Cross, Start or Triangle on any pass: nothing.
- Game-over reset: play with the bars group shown (timer G0 = 200) when flag E is set. On the game-over page's first pass G0 and every other HUD visibility timer become 0. After Continue with a pending snapshot, the first play pass draws no bars, panels or timer texts; a health change re-arms C, and a change seen by the bar update re-arms G0 and G1, after which the bars show again for 240 ticks.
- Game-over reach: with flag E clear, no menu choice, Triangle press or focus move in any screen reaches record 23.
- Controller check (section 2b): pad 0 unplugged on the attract's first visit: the controller box shows and the wait stops; plugged back after 120 passes: the wait resumes, so the movies are queued 120 passes later than the 301-pass case.
- Title (state 12): Cross on pass 4 with focus 0: nothing; on pass 5: new game, cue 0x9A, code 0x0B4. Start on any pass: nothing. No input: code 0x258 on pass 1,801.
- Music overrides: a level script sets the bank override to 64 (bank 74); the player opens the pause page: on its first pass the bank override is -1 again, and it stays -1 after Resume. The same override with a dialogue box open (state 23): kept.
- Lead-in stepper (synthetic, record 27 entered with c = 0): a press on page pass 10: nothing; on page pass 11: code 0x1E0 (record 24). Three more presses, each on pass 11 of its page: 0x208, 0x230, then 0x0B4 with the four music overrides cleared.
- Quest-items tab (synthetic holdings: 23 category-7 records, one of each in slot order: items 0 to 18, 21, 22, 23, 24 of the quest set, each held once; n = 23). On entry c = 0, offset 0, cells 0 to 19 filled with owned records 0 to 19 (cells 20 to 22 do not exist). Presses and resulting c: Down x4 gives 5, 10, 15, 0; Right x3 gives 1, 2, 3 (panel: Black Bogberries); Up x5 gives 18, 13, 8, 3, 18 (panel: Knight's Crypt Keys). The offset stays 0 on every press; records 20, 21 and 22 (Hammer of Thunder, Golden Trove of Riches, Kreludan Metal) are never visible or selectable. A small list, n = 7: from c = 6 (row 1, column 1) Right gives 5 (the same row wraps past empty cells), Left from 5 gives 6, Down from 6 gives 1, Up from 1 gives 6 (empty rows are skipped); each press plays sound 2, 3, 0 and 1 respectively.
- Fresh game item screen (disc): Tor active, Items tab: visible cells 0 (id 150, count 3) and 5 (id 149, count 3), 18 hidden cells; strip shows only the middle icon (item 119). At the Act 2 start with Roberta active: the same two cells, strip with only the middle icon (item 127). After Tor picks up item 120 (Regulation Chainmail), the middle icon shows item 120's art and item 119 is gone.

## Open questions

- Resolved (R-0019, part 4): the saved vectors' fourth words are w = 1.0 for A (position) and w = 0.0 for B (facing); see the R-0019 4 entry below. A load copies both vectors, with those words, into the restart point.

1. Resolved: there is no shop screen and no equipment screen.
   - Shop: the "Buy %s" text is the world interaction prompt, chosen by the interaction chooser (SPEC-0022, Interaction prompts), not a front-end screen. The purchase runs from that world prompt: its price check, the wallet change, the item add and the buy hook are in SPEC-0050 section 3 (High). The price caption of section 15 is drawn by the in-play state and the box host (state 23); its writers are the buy prompt and coin gains.
   - Equipment: no separate equipment screen is known. The equipment paths are the Mote picker of the second quick panel, which also runs on the item screen's Charms tab (Cross word 0, Square word 1, Circle word 2; SPEC-0021). Circle on the Items tab sets the quick-item marker, not an equipment word (section 14.2).
   - Confidence: High.
1a. Resolved: **Pause targets.** Section I binds each pause item to a focus state and a Cross target: View Map (state 2) opens the map page of the hero's area, View Quests (state 3) opens the journal (code 0x1B8), Load Game opens record 17 (state 4 while focused), Resume Game (state 5) returns to play, Options opens record 11 and Quit (state 6) opens the quit box (Behavior 2, section 13.1). **[High]**
2. Answered: the physical button map is in SPEC-0033 (Layout); no open item remains for the logical word.
3. Answered: the Vibration option gates pad rumble through the display sync, every pass (Options, Vibration and the pad; SPEC-0033).
4. Answered: the live option block gets the defaults once at boot and each save slot's block is zero after start-up (Options, Defaults).
5. Resolved: the Music gain multiplies the music row's Level (the bank music volume) and sets the pause level override; the Sound-effects gain sets the default cue volume (row Scale x gain) and scales the stream fade-in ceiling and the stingers; the Voice gain drives only the emote clips, at trunc(1024 x Voice gain) (Behavior 3; SPEC-0009) (High).
5a. Resolved: the Music, Sound Effects and Voice knob nodes each store x = 2 in the page data. Their key-to-node mapping is Music to node 832, Sound Effects to 840 and Voice to 848. At the default value 75, the first apply changes each x to 98 using the slider rule in Behavior 3. This is the stored node coordinate. Resolved (UI pass): the knob nodes sit directly under the slider's row group, which is at x 256, so the knob's left edge is at canvas x 256 + k, with k = trunc(value x 1.29 + 2); before the first apply the knobs are drawn at x 258 for one pass (section G.4). **[High for stored values, mapping and rendered position.]**
5b. Resolved: the music controller and the per-pass voice update run on title and menu passes once the title has been shown: no movie playing, the title-start word set and the game-state code 9. So the title's bank 35 and the credits' bank 18 are heard, and front-end cues played through the normal cue call get trunc(row Scale x Sound-effects gain). Before the first title pass (the card check, the logo, the legal screens and the attract movies) neither runs, and the default cue volume is 1,024 (Behavior 3a and 3c; SPEC-0009, Per-tick controller, Pass gate). **[High]**
6. Resolved: the invert toggle receives the page's focus index. Controls row 1 (Invert Left-Right) toggles +0x00 and row 2 (Invert Up-Down) toggles +0x04, which matches the display slots 67 and 68 (Behavior 3). The axis effect: +0x00 flips the right-stick X byte and +0x04 flips the right-stick Y byte. **[High]**
7. Resolved: table B states 2, 3, 5 and 6 are the focus states of the pause items View Map, View Quests, Resume Game and Quit (section I; Behavior 2). Table A (74 slots) is described in section 9c (Open question 85 for the Low slots). **[High]**
8. Partly resolved: every table B state, its record and its return values are in Behavior 2 (state table and state return survey), and the dispatch from a record's +0x0A and an item's section I +0x06 to table B is in Behavior 1b. **[High]** The colour-editor branch of the driver is a debug colour editor that never runs (section 17.1); its panel layout is not described.
9. Resolved: there are four save slots, and no fourth label ("Game 4") is needed. Each picker row is built from its slot's own text (the empty word, or the location name and date), not from a label (Behavior 7.1). The fourth row is slot index 3. The front-end labels "Game 1" to "Game 3" are not used by the row code; they sit on page 896, which no code opens (Open question 45). Confidence: High.
10. Resolved: Start and Select opening the in-game menu. Select opens the pause page (record 2, state 7) and Start opens the item screen on its current tab, whose entry code is one of the six codes (14.4). Each button also closes its own screen: Select leaves the pause page while its quit box is closed (cue 0x95, back to play), and Start or Triangle leaves the item screen (cue 0x95). Start does nothing on the pause page, and Select does nothing on the item screen. The two are separate entrances to one page group: the item screen reaches the journal and the map by L2 and R2, and the pause page reaches them by View Map and View Quests. Both openers run only after the credits, book-reader and game-over gates and only when the busy check is zero (section 6). Why the game uses two openers is a design choice that no data states; nothing more is needed to reproduce the behavior. Test vector: in play with the busy check zero, Select returns record 2 and Start returns the code of the current item tab; on the pause page Start is ignored, and on the item screen Select is ignored. High for the routing and the closing keys.
11. Answered: the start-point item indices are in section 5, and the chooser is never entered (Open question 86).
12. Section I is resolved (item targets, see Section I). Section K is resolved for the node link, colour modulation, variable index, font, spacing, draw gate and format; the unused HH:MM format variant is not present on disc (Section K). The tail is the font table (3 fonts) and the header word at +0x4C is section N's count (Section M, N and tail). The symbolic names behind the 92 keys of section C, and which screen each front-end key is used for, are open.
13. Partly resolved: section M is 14 UI animation tracks of 112 bytes (kinds 1, 2 scale, 3 colour), and section N is 20 animation slots of 32 bytes with its count at header +0x4C (Section M, N and tail). **[High]** The meaning of header word +0x3C (585), which is not N's count, is not encoded anywhere (unknowable; the rewrite ignores it). Kind-1 tracks: none is on disc, so the shipped game is not affected; the rewrite skips them.
14. Partly resolved: J +0x10 is a relocated animation-list pointer; when it points to a list, J +0x31 gates the node's colour-animation update and is set while a trigger starts that animation and cleared when the track chain ends (Section J; High). J +0x14 and +0x16 are the position, +0x1C and +0x1E an authoring pivot not used for placement, +0x2A the row-spacing byte, +0x2D the absolute flag, +0x2E the alignment byte and +0x30 the hide flag. J +0x32 to +0x33 and sprite field E +0x18 have no known reader; their purpose is not encoded (unknowable) and the rewrite ignores them (Section E). (E +0x08 is the asset index.) (The handler index at +0x24, the search key at +0x28 and the mode byte at +0x2C are described in section 9b.)
15. Resolved in part: the three frame counts form one valid index range and the renderer uses the frame byte and grid dimensions without distinguishing the group boundaries; the group names' authored intent remains unknown. A zero count does not suppress drawing: linked zero-count sprites use frame 0 and the first grid cell if their node reaches the sprite draw path. Whether every such node is visible under its page, parent and condition gates is not established. **[High for the range, grid mapping and lack of a zero-count draw gate; Medium-High that groups have no other effect.]**
16. Resolved (proactive (open close-out); the survey is complete, and visibility of each string follows its node's predicate slot and condition mode, section 9b and 9c): a section-D tree survey attributes several string groups to screen records. Record 3 (Roberta's Items tab) contains Use, Activate, Deactivate, Exit and NeoPoints; record 9 (the in-play HUD host) contains Use, Activate and Close; record 18 (the save prompt) contains Select, Back and Save Game; record 22 (the quest journal) contains You Do Not Currently Have, Any Quests Available and Exit; and record 23 (the game-over page) contains Continue. The four Next labels are in the lead-in page sequence, whose route is not entered in this build. Record 29 contains Exit, View Quest, Treasures:, of and Brightvale Coast, but record 29 is not entered; a separate map-page tree, record 48, also binds Brightvale Coast. Repeated Select, Back, Exit and Close labels occur in other records, including options and reader trees, so their string alone does not identify a unique screen. **[High for tree membership and screen-record association where the record is entered; medium for visibility.]**
   - Still open: the purpose and runtime consumer of every remaining off-page node copy, including the dormant record-29 tree, and the visibility gates for bound labels in context-dependent trees.
   - Default: preserve the authored node trees and show these labels only when a supported screen record or runtime consumer selects their tree and its draw gates pass. Do not invent a live screen for a node found only in a dormant or unassigned tree.
17. Resolved: **Neutral and highlight labels.** The renderer does not select between two bindings on one text node. The page tree has separate text nodes for the neutral label and its orange highlight counterpart, each with its own section G binding. The neutral node draws normally; the highlight node is under the selectable item's focused flag and uses draw mode 1, so it appears only while that item is focused (sections 9d and 12; Open question 75). The binding colour is each label's full start colour, multiplied only by the node tint, which is 1 on every text node on disc. **[High for the draw rule and node/binding structure; Medium-High for the presentation interpretation.]**
18. Resolved: the hold counter counts ticks, and nothing sets it in the shipped game, so it never holds a pass (Behavior 1, step 2). **[High]**
19. Resolved: the O record kind word is 2 for a sheet asset, whose first 8 bytes are a texture register value addressing the front-end texture chunk, and -1 for a full-screen picture that is not loaded yet (overwritten with the buffer reference once loaded; Section O, Behavior 1a). **[High]**
20. Resolved (proactive (open close-out)): the front-end texture chunk uses the generic texture layout and is uploaded at GS block 0x2A00 as 256 x 511 32-bit pixels; the section O texture words find each sub-texture and palette in it (Section O). The generic layout, including the absolute addresses of the 102 front-end chunks, is in SPEC-0003 (High).
21. Resolved: the canvas is always 512 x 448 with Y scale 1.0. The canvas words have one writer, a setter that is never called (section 10). The 512-high video-mode record belongs to the platform display set-up (SPEC-0008), and its PAL branch is unreachable. **[High]**
22. Resolved: normal play runs state 11 (record 9, the in-play HUD), which does not re-arm the HUD timers each call. States 8, 9 and 10 are the item-screen tabs, which re-arm them every tick (Behavior 6, section 14.1). **[High]**
23. Resolved: **Cutscene and menu hide rules for the HUD.** Menus and other pages are separate front-end states, so the HUD page is not drawn under them; a dialogue box hides the whole HUD and a cutscene without a box hides only the monster bar, the D-pad hint and the journal icon (SPEC-0023 section 4.1). The main-loop flag tied to game-state code 9 selects global tasks and does not hide anything (SPEC-0023 Open question 7). **[High]**
24. Resolved: the load cursor starts at 4, outside 0 to 3, when no slot is filled (Behavior 7.2); Cross then copies nothing. Apart from that starting value, the movement rule (Behavior 7.1) keeps the cursor at 0 to 3. **[Medium-High]**
25. Resolved: the item-screen tab states 8, 9 and 10 each keep HUD calls: they re-arm HUD timers G0, G1 and C every tick, draw the bar group 0x31 at a (60, 23) draw offset, and draw elements 0xBF and 0xBE (section 14.1). **[High]**
26. Resolved: the item screen's tab word is 0 (Items) at boot, and only L1 and R1 change it, so the screen opens on the last tab viewed for the whole session; it is not saved and not reset by a new game or a load (section 14.1). **[High]**
27. Resolved: Circle on the Items tab writes no equipment word. It sets the active hero's quick-item selection marker (SPEC-0021) to the cell's id, or clears it when it already holds that id; the "equipped" frame (slot 1) and the Activate/Deactivate labels (slot 6) show the marker (section 14.2). **[High; Medium-High that the id set is the cell's id, inferred from the comparison beside it]**
28. Resolved: on the Charms tab the buttons are those of the Mote picker of the second quick panel (Cross word 0, Square word 1, Circle word 2; SPEC-0021). On the Quest-items tab Cross and Circle always fail the usability test (every entry is category 7) and play 0xA2 (section 14.1). **[High]**
29. Resolved: **Party setting.** The word at +0x14C of the global game object is the active hero's index (SPEC-0053: 0 Tor, 1 Roberta); it selects the entry-code group, so Roberta's item pages are records 3 to 5 and Tor's records 6 to 8 (14.4). Square reads the same word on purpose: the category-9 list keeps one cloak entry per hero, and Square swaps the active hero's cloak entry with its body entry (SPEC-0021, Tattered Cloak). **[High]**
30. Resolved: **Entry destinations.** A code N selects section D record N / 20: 0x3C, 0x50 and 0x64 are Roberta's Items, Charms and Quest items pages (records 3 to 5), 0x78, 0x8C and 0xA0 are Tor's (records 6 to 8), 0xB4 is play (record 9) and 0x1B8 is the journal (record 22). **[High]**
31. Resolved: **Journal row gate.** A quest row is listed when neither flag id is -1, its start flag is set and its done flag is clear (SPEC-0049). The show-all setting is a debug-only switch word, zero on disc, that makes the flag tests pass for every row; it is not set in play (section 14.3; SPEC-0051 Edge cases). **[High]**
32. Resolved: **Journal rows and the map cursor.** The row-indexed vector in the shared row drawer is the row's RGBA bar colour, not a position offset. Its pulse branch runs only for the caller-marked selected row; the journal marks the cursor row after resolving any pending map target. The bar colour changes over time, but row geometry does not move: rows use their fixed positions plus cumulative row offsets, and the first page pass draws no rows. There is no row slide-in. The map cursor is specified in SPEC-0051 section 4. **[High]**
33. Resolved: **Second input word.** It is the raw newly-pressed word of the menu snapshot (+0x894): buttons only, without the stick-derived direction bits. The snapshot is the OR over the controlling set, which is always pad slot 0 (SPEC-0033 Controller map). Its bits are the main word's (0x0001 L2, 0x0002 R2, 0x0040 Cross, 0x0100 Select, 0x2000 D-pad Right, 0x8000 D-pad Left). The "select" and "back" bits of the journal and the map are L2 and R2, which move around the page ring (14.4). The item screen reads L2 and R2 from the rising-edge word (+0x89C) and the journal and map from the second word; both carry the same button bits, so there is no conflict. Select (rising-edge word) closes the map (section 4). **[High]**
34. Resolved: **Cue names.** By the index rule, 0x98 is the confirm (advance) button sound, 0x99 the back button sound, 0x9A the direction-pad press and 0xA1 the cloak use; 0xA3 is the Mote activation. These agree with every front-end use: Cross and tab or page moves play 0x98 or 0x99, focus and cursor moves play 0x9A, and the cloak swap plays 0xA1 (Where it lives on the disc, Cue ids). **[High]**
35. Resolved: the two quest-row linked ids are its start-flag and done-flag indices; all disc values are 0 to 675 and fit the 743-bit progress-flag bitmap (SPEC-0049; High). On the Quest-items tab, Cross and Circle run the shared item-usability test, which rejects category 7 for every entry and plays cue 0xA2, so neither action uses or consumes the quest item (section 14.1; High).
36. Resolved (High): the item grid list (14.1) is fixed boot data. Every routine that uses it (cell draw, cell test and the list walks of the item screen) only reads it; no store reaches it, and the save file has no field for it. It is the same on every boot and in every save.
37. Resolved (High): all 20 ids of the grid list have category 6 in table 0 of the item text database (SPEC-0021: consumables, which are the fruits, food, potions and eggs), 0 as carry limit, and a name. No id of another category appears, so the Items tab holds only category 6 items, and the Charms and Quest-items tabs hold the owned items of categories 8 and 7.
38. Resolved (High; proactive (open close-out)): an overlay cell outside the margin, or one whose projected depth value is not above 1, is skipped: nothing is drawn and it adds nothing to the draw counter. The tick empties the whole cell list after each pass, so no cell is tracked past the pass in which it was queued. The full caller set of the tick is in section 15.1.
39. Resolved: **Price caption colour test.** There is no cannot-pay label. Line 1 shows "Exorbitant!" in (128, 60, 90, 127) when its value is 32767; otherwise the number, in (160, 160, 90, 127) when it is no more than the balance (party +0x14) and in (128, 60, 90, 127) when it is above. Line 2 has no test (section 15.2). **[High]**
40. Resolved: **Price caption line 2.** It is the balance, set by the buy prompt, drawn as a number in (128, 128, 128, 127) (section 15.2). **[High]**
41. Resolved: **Second caller.** The other function that draws the caption is the box host (state 23), every pass. In play the caption is drawn only while both quick panels are closed (section 15.2). **[High]**
42. Resolved: **Caption writers and clears.** The buy prompt sets line 1 = price and line 2 = balance, each with counter 1, on every tick it applies; every coin add (items 135 to 139, cue 0xA5) sets line 1 to the gain, or adds the gain while the line is still shown, with counter 180 (a running total over 3 s). World teardown clears both lines; the menu and HUD reset clears line 1 (section 15.2). **[High]**
43. Resolved: **Condition-handler fields.** The signed index at +0x24 selects one of 74 predicate handlers; -1 means the node has no handler of its own. The byte at +0x2F is passed to that handler, whose interpretation of it depends on the selected predicate (section 9b and the predicate table). In all 1,366 nodes on disc, 893 indices are -1 and the other 473 are 0 to 73; the argument field has 40 distinct values. **[High for dispatch and index range; Medium-High for the disc counts]**
44. Resolved: the column offset uses the width at J +0x22 and the row offset the height at J +0x20; the texture is sampled one texel per pixel from the frame cell's top-left, with no stretch (Section 12). **[High]**
45. Resolved: the page 896 labels "Game 1" to "Game 3" are not drawn. No code opens page 896, and the picker row drawing does not use them; they are a leftover, not a name for any slot. **[High]**
46. Partly resolved: kind 6 is the value-text node (Section K). Kind 2 is resolved: it is the text node draw (Section G; Text node draw, section 12): tint factors 1.0 or colour-track bytes / 128, the binding's unsigned RGBA colour, its letter spacing (+0x0C) and its font id (+0x0E; negative draws nothing). Kind 1 and kind 7 nodes: default if unresolved (rewrite choice): draw them only as far as described here and treat the kind-1 object test as passing (needs a runtime check).
47. Resolved: **Text pattern with one string and two numbers.** It is not a picker line. It is the item screen's item-name line: the shared tab step writes the selected item's name, then appends the times-obtained count and the carry limit in parentheses, or the held count alone (section 14.1, Item name and description lines). The picker's line 2 is the 16-byte date text (section 7.1). **[High]**
48. Resolved: load starting cursor (Behavior 7.2). Slot 0 if it is filled; otherwise the first filled slot among 1 to 3; if no slot is filled, the cursor is 4. **[Medium-High]**
49. Resolved: map projection table values (section 14.5). By the user's decision the clean side reads them from the user's own boot executable; SPEC-0051 owns the table and gives its address, count, stride, layout and read rule. The remaining question (the records have no source table in the file image) is SPEC-0051 Open question 1.
50. Resolved: **Projection record matrix.** The four groups are columns; the input is (x, y, z, 1); screen x and y are result components 0 and 2, so the height is ignored (section 14.5; SPEC-0051 section 7). **[High]**
51. Partly resolved: **Map area index.** The record used is the displayed (selected) map area, 0 to 76 on disc data; a negative area never opens a map page (14.4; SPEC-0051 section 1). An index of 77 or more is never produced by disc data; default (rewrite choice): treat it like a negative area and do not open the page (needs a runtime check). **[High (range on disc data); Low (out-of-range behaviour)]**
52. Answered: the marker angle is facing - yaw offset + pi, in radians (section 14.5, step 3; SPEC-0051 section 7).
53. Resolved: **Marker pulses.** The float pulse (step 0.03, bounds 0.8 and 1.5) scales the hero marker's colour; the integer pulse (step 50, bounds 3900 and 4300) is the cursor icon's scale in 4.12 fixed point, not an angle. Both reverse at their bounds (SPEC-0051 section 5). **[High]**
54. Resolved: **Menu cursor mode codes (section 4.1).** The codes are 0x4000 (Down) and 0x1000 (Up). Down moves to the next selectable item and Up to the previous, with wrap; Left and Right give no list move. **[High]**
55. Resolved: **Direction per mode code (section 4.1).** Down (0x4000) selects the next move and Up (0x1000) the previous move. **[High]**
56. Menu item record (section 4.1): the layout of a menu item is not described here. Resolved in part: the selectable items are the kind-4 nodes; items of other kinds are passed over (Medium-High).
57. **Flag pair** (section 16): which setting flags A and B represent and which code reads the third query are not encoded in the data (unknowable); the rewrite stores the pair as documented and reads nothing from the third query.
58. **Pools by mode** (section 16): default if unresolved (rewrite choice): the default pool behaves as its own pool, separate from the named modes (needs a runtime check); the slot contents beyond the documented fields are not encoded (unknowable).
59. **Free list and layout walk** (section 16): the purpose of the 16-node free list is a developer intent that is not encoded (unknowable). Default if unresolved (rewrite choice): the flagged node's offsets are included in the layout walk (needs a runtime check).
60. Resolved: **Pad-mask directions in the menu driver and focus search (section 17.1).** The Behavior 4 names are right: 0x1000 Up, 0x4000 Down, 0x8000 Left, 0x2000 Right. The stick mapping (stick up adds 0x1000) and the focus search geometry (0x1000 prefers items above, 0x4000 below, on the y-down canvas) both confirm them. Value C's bits act as R1 and R2 held. **[High]** The three driver values A, B and C are the channels of a debug colour editor that runs only on a page with a kind-8 node; no such node is on disc, so it never runs (section 17.1). **[High]**
61. Resolved: **Handler return codes.** There is one convention. A state returns 0 (no decision; the generic step runs), -4 (no decision; generic Cross and Triangle blocked), -1 (back) or a section D record (go there). The front end also resolves -3 (stay) and -2 (clear the screen), which come only out of the generic step. The list of section 17.1 is the same scheme after the generic step (Behavior 1, step 5; Behavior 1b). The earlier reading of -4 as "pop and stop" was wrong. **[High]**
62. Resolved: **Focus search candidates and wrap (section 17.2).** Every kind-4 item of the page whose predicate passes, other than the focused one, is a candidate. Its point is the helper position plus its pivot, with a row-spacing term. Candidates behind the pressed direction use the wrapped distance. The cross-axis term is weighted 4 before squaring by a nearness test that always passes for sizes of 0 or more. The search never runs in the shipped game. **[High]**
63. Resolved: **Focus search result (section 17.2).** A zero offset along the search axis only skips that candidate. The search returns the lowest-cost item index, or -1 when no candidate qualifies; on -1 the caller uses the record's neighbour. **[High]**
64. Resolved: **Item record fields (section 17.3).** The menu item record is the section J node. +0x14 and +0x16 are the node's position relative to its parent, +0x1C and +0x1E an authoring pivot not used for placement, +0x2A the parent's row-spacing byte, +0x2D the absolute flag (it stops the parent walk) and +0x2E the alignment byte. Row spacing uses n = 0 for the first child and otherwise the count of visible siblings from the second child up to and including the node (Section J, Node transform). **[High]**
65. Resolved: **Right alignment (section 17.3).** Two rules exist. The draw (Behavior 12) shifts by the full scaled width for alignment 2 and half of it for 1. The position helper used only by the focus search gives x − width − 1 and x − trunc(width / 2), and never shifts kinds 0, 3, 4 and 5. Other alignment values give no shift in both. Visible nodes follow the draw rule. **[High]**
66. Resolved: **Row spacing (section 17.3).** The spacing is added to the vertical position only, with the count n of Section J (Node transform); no other condition on the parent is known. **[High]**
67. Resolved: **Byte tween (section 17.5).** It is the colour kind (3) of the section M animation tracks; t is in seconds (front-end pass counter / 60); start and end bytes are at track +0x14 and +0x18. No animation slot reaches a colour track, so it never runs in the shipped game. **[High (math); Medium-High (reach)]**
68. Resolved: **Nine-slice panel (section 17.7).** It is the dialogue and message box frame of SPEC-0022 (Dialogue box layout), drawn only by the box update; border words 15, 28, 14 and 26 and sprite size 64 x 64. **[High]**
69. Resolved: **Cursor rectangle (section 17.6).** It is the value marker of the debug colour editor panel, drawn for kind-8 nodes only, and never drawn in the shipped game; the word at 0x2EC8C8 is the editor's value channel, not a list scroll. **[High]**
70. Resolved: **Menu page open (section 17.4).** The step is the full-screen picture load. It plays no cue, and its completion callback only marks the read as finished. The two-slot index picks the picture buffer, and the record that last used it is marked not loaded (Behavior 1a). **[High]**
71. Resolved: **Menu driver values (section 17.1).** Values A, B and C belong only to the debug colour editor (loaded from and written back to a kind-8 node's colour record), which never runs in the shipped game; each steps by 1 per pass on its held bits. **[High]**
72. **Disc check of section 17 values.** The caps (63, 180, 63), the frame counter and scroll addresses, the nine-slice border addresses, and the rectangle and quad sizes come from the code. They are code constants, not disc data, so there are no disc bytes to compare; the rewrite uses them as given. The menu driver runs once per menu tick.
73. **Record stack owner and pops (section 9a).** The drawing path pushes a frame per drawn node and calls a separate pop after the draw; default if unresolved (rewrite choice): the pop restores the previous frame exactly, the draw path alone pushes and pops, and the 32-slot record stack and 31-entry screen stack are independent (needs a runtime check). **[Low]**
74. **Overflow report.** The overflow reports (the record stack's and the screen stack's) are debug-report calls. Default if unresolved (rewrite choice): when a push is refused, drop it, keep all state and show nothing (needs a runtime check); a debug report has no gameplay effect. **[Medium-Low]**
75. Resolved: **Draw condition mode 1.** The tree draw passes a focused flag to the children of the kind-4 item whose running index equals the focus index; mode 1 nodes draw only under that flag. Each list item's mode-1 highlight sprite and label are its selection highlight (section 9d, section 12). **[High]**
76. Resolved: **Key mask "rate" (section 9b).** It is the animation slot's start time: the clock = front-end pass counter (0x2EC8C4) / tick rate (0x2EC8D4, set to the run-time rate 60 at front-end start), in seconds. **[High]**
77. Resolved (High for the known search routines): kind 2 is not a runnable search result and its condition is not tested by those searches. General searches recurse through kind 2 to reach its children; only the draw path evaluates the text node's own condition before rendering. No other search path for these nodes is established.
78. Resolved: the runnable scan tests the signed +0x08 word in the linked asset record; −1 means the full-screen picture is unloaded, and its loading path replaces that value with a runtime buffer-slot index. On picture records, +0x0C selects resource 575 plus its stored value; values 2 through 102 therefore select resources 577 through 677. A separate scan uses the sign of +0x0C for its own eligibility test. **[High]**
79. Resolved: the 52-byte section J relocation loop uses the section count at payload +0x2C (1,366 records), and the 112-byte section M loop uses the count at +0x48 (14 records). Both counts match the exact section extents. Section K has its own 52-record loop. **[High]**
80. Resolved: **Item screen grid gate.** Slot 12 is a constant false library gate: the item-screen groups sit under it so the automatic page draw skips them, and the tab states draw them every pass by section C key (0xC2, 0xBF, 0xC0, 0x31; section 9c). **[High]**
81. Resolved: **Save prompt opener (section 7.3).** The save prompt (record 18, table B state 22) is opened by the world's character-region update when the player presses Square within 5 units of a save point (a kind-4 record of the world pack's character region, SPEC-0006) in normal play with the hero not busy. The update calls the front end's set-screen routine with record 18 directly; no return code is involved, so no handler returns 0x168. No menu item (section I), handler or level host call opens record 18. **[High]**
81a. Resolved: **Save screen and the world update.** The world is frozen: the save screen does not set the play-active word, so the game-state code is 9 from the pass after the Square press (section 2a). The save-point step is part of the frozen world update, so a second Square press cannot reopen the screen or reset the save stage. The card jobs run as global tasks that run in every game-state code. **[High]**
81b. Resolved: **Text variable 1 (section 7.3).** The value is the needed space computed by the card layer from the file sizes: 106 KB on every boot (section 7.3). **[High]**
82. Resolved: **Pause Resume Game and Quit targets.** Resume Game (state 5) plays cue 0x95 and returns code 0x0B4 on Cross. Quit (state 6) plays cue 0x9A and opens the quit box run by the pause state (state 7); its Circle answer tears the world down (code 0x000) and its Triangle answer closes the box (Behavior 2). **[High]**
83. Resolved: **Options page order.** Rows are drawn top to bottom in index order: Options at y 178, 220, 262; Controls at 188, 232, 276; Audio at 203, 247, 291 (Behavior 3, Row draw order). **[High]**
84. **Pure predicate and mode 1 reads (section 9c).** Slot 9 and slot 11 results depend on which quick-item slot and category the caller passes; default if unresolved (rewrite choice): evaluate them with the quick-item slot and category the calling screen holds (needs a runtime check). **[Medium]**
85. Resolved (proactive (open close-out)): **Predicate slots.** All 74 slots are described in section 9c. Slot 38 tests the active petpet kind and slot 39 always fails. Slots 57 and 59 are the Tor and Roberta portrait-available words, both 1 at boot and written by every hero-switch request (SPEC-0023 section 12), so the earlier statement that nothing writes them was wrong. Slot 58 is the hero-slot death test (action state 16). Slot 18 asks the HUD icon tween pool whether a tween is live on an equipment-slot icon (it sets nothing; the earlier "sets a Mote text" reading was wrong). Slot 61 is the loading spinner test. Only the developer names of the HUD words behind slots 44, 47, 48, 51, 53 to 55, 60, 63 and 64 are not encoded anywhere (unknowable; SPEC-0023 gives the behavior). **[High for most slots; Medium-High for 18 and 58]**
86. Resolved: **Start-point chooser and records 24 to 28.** The state return survey (Behavior 2) lists every value each state returns. Together with the section I targets, the back rule and the two direct screen sets, it shows that records 15, 16, 21 and 24 to 29 are never entered, so the start-point chooser and the page stepper never run in the shipped game. **[High, static]**
87. Answered: each save slot's option block is zero after start-up; only the live block keeps the defaults (Options, Defaults).
88. Answered: the Vibration display sync runs at boot and every pass, and status 9 forces the gate to 0 (Options, Vibration and the pad).
89. Resolved: **Change feedback sound.** The helper that runs after a slider change is the controller check (section 2b), which plays no cue. Option changes are silent (Behavior 3, Change feedback). **[High]**
90. Resolved: **Yes and No labels (Controls page).** They are the value texts of the two invert rows: each invert row shows Yes when its option word is nonzero (predicate slot 67 or 68, mode 2) and No when it is zero (mode 3). Only the Vibration row shows On and Off (section 3, Toggles). **[High]**
91. **Unlabelled option fields (+0x0C and +0x10).** Resolved: neither field is reachable and none reads them; the meaning is not encoded (unknowable; SPEC-0025 item 73). The clean side keeps and saves both words with their defaults, and nothing in the known front end changes them. **[Medium]**
92. Answered: the defaults are written before the first sync, so rumble works from the first boot (Options, Vibration and the pad).
93. Resolved: **Card state (table B state 0).** The job started on the first visit is the boot card check (SPEC-0025, Card jobs): no card shows the no-card message (Retry / Continue), a formatted card short of space with no save folder shows the free-space message (Retry / Continue), and an unformatted or good card shows nothing. The state waits for the job and then goes to code 0x258. The first-visit flag is never cleared, so later visits last one pass (Behavior 2, state 0). **[High]**
94. Resolved: **In-play gate codes (Behavior 6).** Every code the play state returns is the section D start plus the code, and each code is a multiple of 20 that names a screen record. Before any button is read the state tests, in this order: (1) story bit 368 (value 0x170 of the progress-bit table): sets the credits-pending flag and returns 0x118 (record 14, credits, state 18; the bit is set by pack 1300's epilogue after the finale, SPEC-0026 Credits); (2) the busy latch, which does not return; (3) the book cursor or the saved book word at 0 or more: clears the first byte of the shared text buffer and returns 0x190 (record 20, the book reader, state 24); (4) flag E: clears the game-over counter and returns 0x1CC (record 23, the game-over page, state 26). It then sets the play-active word and reads the openers: Select, or pad slot 0 not usable, while the busy check is zero, returns 0x028 (record 2, the pause page, state 7) with cue 0x94; Start returns the item-screen code of the current tab (0x3C, 0x50, 0x64, 0x78, 0x8C or 0xA0) with cue 0x94. After the HUD reads, an overlay box in use returns 0x17C (record 19, the box host, state 23). Otherwise D-pad Left returns the map code 0x26C + 20 x area (record 31 + area, state 30) and D-pad Right returns 0x1B8 (record 22, the quest journal, state 25), each with cue 0x9A. A pass that none of these ends returns 0. Each of the six screen records (2, 14, 19, 20, 22, 23) carries the stated table B state in its own record, so the screen identities are fixed by the disc data. **[High (order, codes and setter chain); Medium (that the player reaches the epilogue triggers)]** Test vectors: bit 368 set, cursor 5 and flag E set: 0x118. Bit 368 clear, cursor 5, flag E set: 0x190. Bit 368 clear, cursor -1, saved word -1, flag E set: 0x1CC. Nothing set, Select newly pressed, busy check zero: 0x028. Nothing set, overlay box busy, no button: 0x17C. Nothing set, box idle, D-pad Left, hero map area 8: 0x26C + 160 = 0x30C (record 39). The full button table is in SPEC-0026 (Controller map for menus and screens).
95. Resolved: **World updates while the game-over page is shown.** The world is frozen while the page is up. The page does not set the play-active word, so the game-state code is 9 from the pass after the in-play state hands over to it, although flag E stays set (section 2a; SPEC-0026). **[High]**
96. Resolved: **Game-over page sprites.** The page's two sprites (sprite section records 526 and 527) have no animation frames. Record 526 shows the game-over picture, resource 594 (512 x 128; Behavior 1a). Record 527 is the small Cross button icon (sheet asset "x_sm"), drawn beside the "Continue" caption once predicate slot 72 passes, after about 2 s (Behavior 2, state 26). **[High (asset); Medium-High (that it shows the Cross button)]**
97. Resolved: **Section D fields +0x04, +0x06 and +0x08.** +0x04 is the back and stack rule (-3, -2, -1 or record 9; per-record values in section D). +0x06 and +0x08 are the Left and Right neighbour records used when a Left or Right focus search finds nothing; all are -1 or -3 on disc, so they are unused (section 9d). **[High]**
98. Resolved: **Records with no state.** Exactly seven section D records carry 0xFFFF at +0x0A: 11, 12 and 13 (Options, Controls and Audio, entered by section I targets), 15 and 16 (never entered), and 21 and 29 (no picture, never entered). All seven have back rule -1 (pushed; Triangle returns). **[High]** Test vector: a scan of section D for +0x0A = 0xFFFF gives the record indices 11, 12, 13, 15, 16, 21 and 29.
99. Resolved: **Pending save or load at game over.** The pending flag is set by confirming a slot in the save picker or the load picker, which also copies the slot into a session snapshot (section 7.3). When the game-over page exits with the flag set, the job is a re-apply of that snapshot from memory (no card access), followed by a hero reset and code 0x0B4 with no teardown; so Continue resumes from the last save or load. A load picked earlier takes the same branch. Only the world teardown clears the flag. The quit path does not lead to the save prompt and never writes a save. **[High for the code; Medium-High for the player view]**
100. Resolved: **Countdown display (record 30).** The call made on the countdown's first pass selects controller port 1 as the only menu input source (the input mask is set to pad 0 and the merged menu words are rebuilt); it draws nothing. What the player sees for those 301 passes is record 30's own full-screen picture, the Neopets logo (resource 600). The attract and the page stepper make the same call on their first pass (Behavior 2, states 1, 27 and 29). **[High]**
101. Resolved: **Other routes to the game-over page.** None. The game-over branch of the in-play state is the only code that forms code 0x1CC, and the game-over handler is named only by its table-B entry. On disc no item target (section I) and no back, left or right field of a section D record names record 23 (item targets are -3, -1, 11, 12, 13, 14 and 17; the only non-negative back value is 9; left and right are -1 or -3). The page is reached only through flag E during play. **[High]**
102. Resolved: **In-game menu counters on game over.** The first-pass reset is the HUD and menu reset of the pause and item-screen openers (SPEC-0023 section 4.1). It sets every HUD visibility timer, the quick-panel flags, the Square prompt count, the countdown counter, the monster bar timer and the fade-enable flag to 0. What it changes on screen: after Continue on the snapshot branch (code 0x0B4, no teardown) the HUD starts hidden and returns on the next re-arm, as after closing the pause page. On the teardown branch the next screens (card check, logo, attract, title) draw no HUD, and play starts again with these values at 0. **[High for the writes; Medium-High for the on-screen result]**
103. Resolved: **Caller of the Cross latch (state 6).** State 6 is the focus state of the pause item Quit (section I); it is run while that item is focused, not through a record's +0x0A field. **[High]**
104. Resolved: **Pause box.** Only Quit opens a box: record 26 of the system message table, a quit confirmation with labels Quit (Circle) and Resume (Triangle). Circle: cue 0x98, world teardown, code 0x000. Triangle: cue 0x99, the box closes and the pause page stays. While it is open the input block hides the menu words from the focus states and the generic step. Load Game, Resume Game and Options open no box (Behavior 2, state 7). **[High]**
105. Resolved: **Writers of the pending book index (section 6.1).** The pending index (game-state word +0x50) and the cursor (+0x54) are set to -1 once, when the game-state object is created, and are never reset. The pending index is set only by the book-station handler (SPEC-0028) and cleared only by the chooser's book step; the cursor is written only by the book step and the reader. **[High]**
106. Resolved: **Saved book word (+0xA8).** It belongs to a global settings block, which is created on first use with the word at -1. Its only other writer is the reader exit (-1), so it is always -1 in the shipped game. The play gate also opens the reader when this word is 0 or more; a clean side that starts it at 0 would bounce between play and the reader every pass (section 6.1). **[High]**
107. Answered: SetGameOver is host call 154 of the level-script table (SPEC-0030; SPEC-0026, Game over).
108. Resolved: **Sub-menu flags.** The two exclusive flags are the open flags of the in-play quick-item panel and second quick panel (SPEC-0021), which open on D-pad Up and Down. The map and journal shortcuts need both clear (section 6). **[High]**
109. Resolved: **Journal Up and Down.** Down (0x4000) adds 1 to the cursor, clamped to the last row; Up (0x1000) subtracts 1, clamped to 0; each move sets the pulse to 1.8 and plays a front-end move sound; the top index then follows the cursor by 1 row per tick (section 14.3). **[High]**
110. Resolved: **Extra branch of the pause opener.** It is the pad-not-usable auto-pause: when pad slot 0 is not usable in play, the pause page opens as for Select (SPEC-0033 Controller map). **[High]**
111. Resolved: **Item-screen pass on the quest-items tab.** State 10 returns 0, 0x0B4 (Triangle or Start), the tab codes, 0x1B8 and a map code; Cross and Circle always play 0xA2 (Behavior 2, section 14.1). **[High]**
112. Resolved: **Overlay box busy test.** The box is busy when its current-page text word (+0x258) is set, or else when its next-page word (+0x25C) is set. The helper returns that test (Behavior 6, Overlay box busy). **[High]**
113. Resolved: **Input word for the openers and the pages.** The second word (+0x894) is the raw newly-pressed word, with no stick directions; the main rising-edge word is newly pressed2 (+0x89C), with stick directions, and SPEC-0033 calls it the rising-edge word (Controller map). Select and Start in play read +0x89C; Select on the pause page, the pause item states, D-pad Left and Right in play, and L2 and R2 on the journal and the map read +0x894. **[High]**
114. Resolved: **Credits item.** The Options page's Credits item targets record 14 (code 0x118, the credits; pushed, back rule -1), opened by the generic Cross with cue 0x98; the credits return to Options by Triangle (from page pass 2, cue 0x99) or at the end of the scroll (no cue, focus on Controls) (Behavior 2, state 18; section 13.2). **[High]**
115. Resolved: **Overlay box users.** The shared dialogue/message overlay is driven by the box host (state 23); neither the book reader (state 24) nor the journal (state 25) uses that overlay object as its page surface. The book reader draws the loaded book pages through the book page driver, and the journal draws its own page tree and row list. They may use shared text/font drawing helpers, which does not make them overlay-box users. (Behavior 2, sections 6.1 and 14.3; SPEC-0022; High.)
116. Resolved: **Book reader exit with no valid index (section 6.1).** The cursor is set to -1, the four music overrides are cleared and the reader returns 0x0B4 with no cue; nothing is unloaded and the saved word is not written. **[High]**
117. Resolved: **Book page timer (section 6.1).** The page driver runs once per frame-loop pass while a book is loaded, whichever screen is current. The timer is the page angle in radians, 1.2 down to 0.12 in steps of (1.2 - 0.12) / 75, so the fall takes 75 passes (1.25 s); each pass draws first and then applies the step. **[High]**
118. Resolved: **Book prompt draw (section 6.1).** "Read Book" is the Square caption of SPEC-0022 (slot 4 at (327, 395), default caption colour, Square icon in the same pass). **[High]**
119. Resolved: **Book cursor sharing (section 6.1).** No other handler reads or writes the cursor (+0x54). **[High]**
120. Resolved: **Actors that offer books (section 6.1).** NPCs and enemies never reach a type-11 volume or a book station, and the petpet's fire is refused, so only the selected hero sets the pending book index (SPEC-0028, "Trigger pass"). **[High]**
121. Resolved: **Book index bound (section 6.1).** The reader does not check the index against the size of the book directory; station values on disc give indices 0 to 86 only. **[High]**
122. Resolved: **Save success polarity (section 7.3).** The card state word is the write job's result: zero means "saved", or "Cancel save?" answered Yes inside the write job (a quirk: cue 0xA9 plays although nothing was written); nonzero means "not saved" (No to the question, or Retry after a failure), which goes back to a new card scan and the picker (SPEC-0025, Card jobs). **[High]**
123. Answered: Triangle in the load picker leaves by the generic back rule with cue 0x99; the card stages return -4 and a cancel returns -1 (section 7.2, Leaving without a load).
124. Resolved: **Book prompt gates and order (section 6.1).** The book step runs only when the chooser's generic chain runs (hero input enabled, hero not occupied, game-state word +0x5C not above 0, no buy or talk, no earlier entry). It runs in the hero's update, before the hero's trigger step, so "Read Book" shows one tick after entering a station and one tick after leaving; a pending value left while the hero is occupied stays stale until the step next runs. **[High]**
125. Resolved in most part: **Screen reach of the pictures (Behavior 1a).** The credits background (589, record 14) is shown, from the Options page's Credits item and after the finale. The Neopets logo (600, record 30) and the legal screen (577, record 1) are shown at boot and after every card-state visit (the logo also after the title's idle timeout). The lead-in pages (595, 597, 599; records 24, 26, 28), Coming Soon (596, record 25), Controls (598, record 27) and the chooser's copy of the load-game picture (record 15) are never shown, because their records are never entered (Behavior 2, survey). Still open: whether the player reaches the epilogue triggers that lead to the finale credits (SPEC-0026, Credits). **[High (reach); Medium (finale credits)]**
126. Resolved: **Records 24 to 28.** Reading (a): they are tutorial lead-in pages, not start-point steps. Records 27, 24, 26 and 28 carry the Controls, Attack Maneuvers, Quick Inventory and Mote screen pictures, each with a Cross icon and "Next"; state 27 turns the page on any new press from page pass 11 and enters play after the fourth (section 5). No screen, item or code enters record 27 (or 24 to 28), and the title's New Game goes straight to play, so the pages are not reached. **[High (mechanism and reach); Medium (that a New Game step to them was removed)]**
127. Resolved: **Map id per level.** The map id is the hero map area: word +4 of the current world pack's type-1001 record (SPEC-0006), or, when that is -1, the last value given to the script call RegisterTransitionMap in the session (SPEC-0051 section 1, with the survey of which packs carry which id). **[High]**
128. Resolved: **Picture buffer reuse.** When a third picture is loaded into the first buffer, the first picture's asset record is set back to "not loaded" (kind word -1) at that moment, not when its screen is left (Behavior 1a, item 4). **[High]**
129. Resolved: **Journal objective line height (section 14.3).** The objective is font 2 (line height 14) and the title font 0; row height = max(79, 75 + 14 L). **[High]**
130. Resolved: **Journal row-part pixel offsets (section 14.3).** The row parts are canvas-absolute nodes of the journal tree, drawn at their node position plus the row offset; their node keys and positions are in the row-part table of section 14.3. **[High]**
131. Resolved: **Triangle on the journal and the map.** Entering a record whose +0x04 is 9 cuts the stack back to play (record 9) and pushes the page, so Triangle pops straight to play (cue 0x99), whether the page was opened from play or from the pause page (section 9, Back rule). **[High]**
132. Resolved: **In-play shortcuts and the quick panels.** The shortcuts need the overlay box idle, both quick-panel flags clear and the in-play controls gate of SPEC-0053 section 3.13 (hero state not 25, the second hero test false, game-state words +0x20 and +0x5C zero). The panels open on D-pad Up and Down and own Left and Right while open; the hero update runs before the front end, so a same-tick Up and Left opens the panel and the map does not open (section 6). The meaning of the second hero test is left to SPEC-0053. **[High]**
133. Resolved: **Value-text byte K +0x0E.** It is a draw gate: the value text is drawn only when the byte is not 0 (and the font id is 0 or more). Its size has no other effect. All 52 records are non-zero, so every value text draws (Section K). **[High for the gate; Medium that nothing else reads it]**
134. Resolved: **Book reader Triangle re-entry (section 6.1).** The book request is a one-slot cache, so asking again for the loaded index does nothing: no reload and no page timer reset. On the bounce pass play runs its pass-0 entry while the world stays frozen. **[High]**
135. Answered: the Charms-tab equip plays cue 0xA3 and no slot has its own cue (SPEC-0021, Open question 98).
136. Resolved: **Node 954 (spacing 9990).** It is the journal row's objective template. The page draw skips it, because its parent group is gated by predicate slot 12 (always 0). The row drawer reads only its position, alignment and font id and draws the objective with the wrapped text routine, which never reads the node's letter spacing, so 9990 never shows (section 14.3). **[High]**
137. Resolved in part (R-0018 1; R-0019 2 to 4 resolved below): (R-0018 1a and 1b, High) the monster bar group 0x24 (node 599) has mode 2 with predicate slot 41, and slot 41 is the monster bar test: the first quick panel closed, the busy test false and the monster bar timer not 0. It is the same rule as the draw predicate of SPEC-0023 section 10.2, so there is one gate, not two: the page walk draws the group exactly while the timer is not 0, no quick panel is open and no letterbox or screen fade is active. (The earlier reading of the slot as a journal-notice test was wrong; the word it reads is the monster bar timer, which the arm sets to 600 and each in-play tick lowers.) Slot 62 (the gate of group 0xBB) also requires that the monster bar is not visible. Test vector: timer 30, panels closed, no screen effect: slot 41 is 1 and the bar draws at alpha 128; timer 0: 0; timer 30 with the first quick panel open: 0. Resolved (R-0019 2 and 3). (R-0019 2) The bar sprite clamps past the texture edge: the sprite draw sets no wrap mode, and the start-up clamp stays in force on the front end (SPEC-0023 Open question 67; High for the start-up value, Medium for the in-play state). (R-0019 3) The save and load prompts draw their hint group (group 0xDE and its Select and Back nodes) with the focused flag set by the caller, which the tree draw allows (section 9d). So the hints show on the prompt screens, not through a focused kind-4 item (Medium-High).
(R-0019 4) Resolved (High for A, Medium-High for B): the save writes the two vectors with all four words. Vector A is the hero's position quad with w = 1.0 (the motion step stores 1.0 in the fourth word of the position after every move, and a placement stores 1.0 too). Vector B is the hero's forward-axis row of its movement basis with w = 0.0 (the basis rebuild, which runs every tick and at every placement, zeroes the fourth word of all three rows). So a fresh save holds (x, y, z, 1) and (fx, fy, fz, 0); the end-of-pass apply then overwrites both stored vectors with (0, 0, 0, 1).

138. Resolved. (4) The Charms picker moves the selection through the direction table of SPEC-0021 (Movement, rules 6 to 8): Left, Right, Up and Down use columns 0, 1, 2 and 3, the two-direction diagonals use columns 4 to 7, and the row is the current selection plus one. The move does not check ownership, so the selection can rest on an unowned Mote or on "No Mote"; the picker does not step through owned Motes in id order (Medium-High). (6) The Charms tab lists the category-8 slot records in slot order, and the Quest-items tab scans category 7 in slot order from the scroll offset. Add fills the first empty slot and removal never moves slots, so the order is the order of acquisition with gaps refilled, not id order (High for the scan order; Medium-High for the effect of the add rule). (5) Resolved. The Triangle "Exit" hint is a 32 by 32 icon node and a text node inside the equipment group of the Mote panel's first equipment slot (predicate slot 13, argument 0). That group's own mode is 0, so its predicate is never tested, and every node on the chain from the panel root down is also mode 0. Nothing hides it and no offset moves it: the icon sits at canvas x 548 and the text at x 570, right of the 512-wide canvas, so it is outside the visible canvas (High for the placement; Medium-High for the drawing outcome). (2) An empty cell is hidden by the alpha byte of its icon element, not by a node predicate. On each pass, for each of the 20 grid cells in order, the item screen sets the cell's icon alpha to 0 when the owned count is below 1. When the count is 1 or more, it sets the alpha to 128 and the element's art word to the item's icon; if the item has no icon, alpha and art are left as they were. The icon nodes' predicate slot 2 always passes and plays no part (High for the rule; Medium for the no-icon case). (3) No node draws the count. The item screen's draw code draws it for each cell whose owned count is above 1: the number in decimal (format "%d"), centred horizontally, with the text point at the cell element's position plus (0, 40), in colour (128, 128, 128, 128) (boot word 0x80808080 at data address 0x315AD8), in the item screen's draw-context font (the same font word as the prompt caption, SPEC-0023 section 30). A count of 1 or 0 draws no number. Test vector: a cell whose element sits at (300, 200) and holds 5 items draws "5" centred at (300, 240); a cell holding 1 item draws its icon at alpha 128 with no number; a cell holding 0 items draws neither. Confidence: High for the draw path and the hide rule; Medium for reading the text point as the baseline.
139. Resolved (High unless marked): **Book reader pages (section 6.1 rules 8 to 14).** One body string per book; one spread; no page turning (the timer only tilts the 3D parts); the style index picks the template row and the part mask; the body fills block A then block B; the title is a centred font-0 value-text node at (256, 40). The 3D page sheet is placed in the game camera's frame (rule 12): origin 1.0 ahead and 0.75 below the view axis, scale 1.7, rest pose corners at pixels (38, 383), (474, 383), (58, 67) and (454, 67) on the 512 x 448 screen (Medium-High).

140. Resolved (R-0130): **Quest-items tab cursor and scroll (section 14.1).** The cursor is an index into the owned records (0 to 19), Left and Right wrap within the row, Up and Down wrap between rows 0 and 3 skipping empty cells, the scroll offset is always 0 (owned records from the 21st on are unreachable), no scroll arrow is drawn (Medium-High), and every directional press with 2 or more records plays front-end menu sound 0 to 3 (cue 0x9A). Default if unresolved (rewrite choice): none needed for the offset; if a scroll arrow is wanted, draw none. **[High; arrows Medium-High]**
141. Resolved (High; proactive, starting equipment): **Where armour shows on the item screen.** Armour, dresses, weapons and shields appear on no tab. They show only as the three-icon equipped-item strip (weapon, body, shield; shield for Tor only), with no text. On a fresh game Tor's strip shows only the Farmer's Clothes icon, and Roberta's strip at the start of Act 2 only the Diplomat Dress icon (section 14.1, equipped-item strip and the fresh-game example). A rewrite should not look for armour in a list.
142. Resolved (UI pass, High unless marked): **Main-menu look.** Every main-menu page lists its items as: static highlight bar, neutral label, orange label on top for the focused item; positions, bar sizes and alignment per page are in section G (G.2 to G.4). The bar never animates (the frame setter has no caller); no page change fades; the title has no "Press Start" text and no hint line; the Options page shows "Select" and "Back" hints, the Controls and Audio pages "Back" only, the pause page none.
143. Resolved (UI pass, Medium): **Rows of the save and load pickers.** Row height is 89 when the second line shows a date and 79 for an empty slot (blank second line); four rows always fit inside the 360-pixel list height (4 x 89 = 356). The load picker has no hint line; the save picker draws Select and Back hints (section G.6 and G.7). Default if unresolved (rewrite choice): give every row 89 pixels.
144. Open, visual (needs a runtime check) (UI pass, Medium): **Backdrop blending of the pause picture.** Whether the world shows through any transparent part of the pause backdrop picture is visual only. Default if unresolved (rewrite choice): draw the backdrop opaque, hiding the world.
145. Open, visual (needs a runtime check) (UI pass, Medium): **Black fill during the new-game load.** Between the title's Cross and the first world frame the front end draws nothing. Default if unresolved (rewrite choice): clear to black until the opening movie plays.
