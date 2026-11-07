# SPEC-0025: Save game

- Answers: R-0019 (point 5 answered here; point 4 answered in SPEC-0035: vector A has w = 1 and B has w = 0); proactive (starting equipment); proactive (UI pass: save and load screens as the player sees them); proactive (open close-out); proactive (open question sweep, menus and HUD); proactive (open question sweep, round 10)
- Status: Draft
- Disc build: SCUS-97367 v1.00 (same fingerprint as SPEC-0001)
- Confidence: per section. Slot size, file size, progress-flag placement, options copy, slot index rule and save-start rule: High to Medium-High. World key tables (values and stride): High (from the boot program's data). Slot field map: High for offsets, sizes and owners of every byte, and High for the meanings unless a row says otherwise; hero vectors A (position) and B (facing): High. Treasure sets, once-per-area sets, per-world tables, item lists and the cloak flags, luck, swap mode, equipment words and hero status words: High. World entered after a load, date text order and the stat-up reset of the new game: Medium-High. Count table meaning: Medium-High (Open question 88 resolved). Load order: High. Validity rules (no checksum, empty marker, lookup misses): High. Hero maxima and restore: High (Open question 66 resolved). Status timers: High for the saved form and the restore (whole seconds, truncated, through the hero's status-apply method). Options: toggles and volumes Medium-High, sound-group mapping Medium. Card jobs as the player sees them (boot check, save scan with the format offer, save write with the overwrite prompt, card gone, load scan, quit confirm), the message records each job shows and their Circle and Triangle answers: High. Busy-box minimum of 123 passes: Medium-High. Card result rules (directory check, block sizes): Medium-High; the unformatted card: High (the save scan offers a format, the load scan reports no save data; Open question 21 resolved). Card operation state machine (13 states, poll results and status codes; part of a dormant card path): Medium. Save job dispatcher (step records and the index rule): High; its kinds 0 to 8: Medium-High. Create-directory, write and delete jobs, their checks and the dispatcher end and failure rules: High. Delete job: High. Event codes: High for the one 24-code set, its names and its numbering; the -4 handling across the jobs: Medium (Open question 49). Save-slot writer, per-slot form (slot index k below 4, 0x3100 bytes): Medium-High. Present flag: Medium-High for the sign-bit rule; its polarity against the empty marker: Medium (Open question 57). Writer capacity groups: High (Open question 58 resolved). Request validator (codes 0 to 11, formerly called the save-slot planner): Medium-High. Save screen opener: High (a save point opens it by a direct screen set; Open question 65 resolved). Save screen stages, including the return to stage 0 on a nonzero card state word: High for the flow and High that 0 means "saved or cancel confirmed". Save and load timing (the write job asks before writing; a busy text stays up at least 123 passes; otherwise no fixed pass count): Medium-High. Restart point and the end-of-pass apply: High for the stores and the timing; Medium-Low for the camera placement. Save points (records, nearest-point selection, glow, sound, prompt, Square opening, restart point): High; what the glow looks like: Medium. Session snapshot and pending flag: High. Engine block as the movie played words, and the white screen and fade-in on load: High. Options apply after a load: High.

## Summary

The game keeps four save slots. Each slot is 0x3100 (12,544) bytes, and the whole save image is 0xC400 (50,176) bytes. Saving is a manual action from the front-end save screen only: the game has no autosave and no automatic checkpoint save. The save screen is opened at save points: within 5 units of a save point in normal play the "Save Game" prompt shows, and a Square press records the hero's restart point and opens the screen (Behavior, "Save points"). A save starts only when the player confirms a slot there; the game then asks whether to overwrite (a filled slot) or whether to save (an empty slot) before it writes, and it never shows a success message. Loading is a separate front-end action. Confirming a slot in the save or the load picker also keeps a copy of that slot in memory, which the game-over Continue re-applies (SPEC-0055).

Each slot holds a location name and date text, the saved world ids of the worlds in the transition, exterior and interior buffers, a progress-flag bitmap, two item containers with fixed kind capacities (the party inventory and the stash), the party and stash Neopoints, luck, a block for each of the two playable heroes (current and maximum health and magic, status timers, poison and fire words, equipment Mote words), a 191-byte per-item collected count table, a 32-byte options block, the 92 movie "played" words, the hero swap mode, the Tattered Cloak flags, five once-per-area item sets, the collected-treasure and held-map sets, and two 193-entry per-world tables. The first per-world table holds each world's broken-breakable bits and the second its 256-bit kill memory, so broken breakables and killed characters stay that way across a save and load. The original has no checksum. Its validity checks are the empty marker, the slot index, the file size and the table lookups.

The clean side writes its own save file of the same 0xC400-byte layout to the user data folder. It does not write to any memory-card location. The original memory-card names and card result rules are given in this spec only so that an optional import feature can read an original card's save, and so that card-based behaviour can match the original. The clean side's own file needs none of the card blocks.

Every byte of the slot has an owner and a meaning in the field map under Layout. Only 66 bytes are touched by neither the save nor the load (0x08D to 0x08F, 0x307A to 0x307B, 0x3083 and 0x30C4 to 0x30FF). A clean implementation stores and returns those unchanged.

## Where it lives on the disc

- No save data is on the disc. Saves are created at run time, so the layout cannot be confirmed against disc bytes. The clean side writes its own file (see Summary).
- Data the clean side must read from the disc at runtime, from the boot program's data (virtual addresses, SPEC-0001 fingerprint):
  - World key table, part 1: 193 entries of 32-bit signed integers at virtual address 0x31EFA8. Each is a world key (16-bit game key; values seen 678 to 1433).
  - World key table, part 2: 193 entries of 32-bit values at virtual address 0x31F2B0, spaced 4 bytes apart. The upper 16 bits of every entry are zero, so each entry is an unsigned saved world id (0 to 11070). Entry i of part 2 is the saved world id for the key in entry i of part 1.
  - Read both parts as parallel arrays in order. Do not copy the arrays into the clean code.
  - World index: the position of a world's key in part 1 (0 to 192), found as the reverse lookup of the forward lookup of the key. Each world gets its index from its key when it is entered. The two per-world tables and the five once-per-area sets are indexed by it (High).
  - In the boot program's data, the 193 saved ids are all distinct. Exactly one id is zero (key 678, entry 0). 97 ids are at most 1326; the other 96 run up to 11070. Two world keys appear twice in part 1: key 1391 (entries 166 and 167, ids 10024 and 10025) and key 1414 (entries 179 and 180, ids 10045 and 10046). The start keys 691 and 909 map to ids 11 and 602.
  - Glyph tables for name text (see Layout, "Name text and glyph codes"): a run of 33 four-byte entries at virtual address 0x2ECD58 followed by a zero entry, and three base entries followed by a zero entry at virtual address 0x2ECDE0. The clean side needs them only if it implements the card name fields (the import feature). Read them at runtime; do not copy them.
  - Growth words (read on load; see SPEC-0036, Hero growth): twelve signed 32-bit words at virtual addresses 0x319BC0 to 0x319BEC, in three rows of four. Row 1: start values 150 (Tor health), 120 (Roberta health), 90 (Tor magic) and 150 (Roberta magic). Row 2: per-band gains 25 (Tor health), 20 (Roberta health), 15 (Tor magic) and 25 (Roberta magic). Row 3: caps 500 (Tor health), 400 (Roberta health), 300 (Tor magic) and 500 (Roberta magic). The reference and step of Roberta's banded fallback (Hero maxima and restore) are not fixed constants: they are Tor's start and gain words, 0x319BC0 (150) and 0x319BD0 (25) for health, 0x319BC8 (90) and 0x319BD8 (15) for magic. These words are also the heal tables of SPEC-0021, so the two specs share them. Read them at runtime; do not copy them.
  - Card-operation state words (run-time state of the card layer, not save data; for reference only, see Open question 22): the stage word at virtual address 0x2DD3E4, the request words at 0x2DD3E8 to 0x2DD3FC, the active word at 0x2DD400 and the clear-only word at 0x2DD404 are all zero in the boot program's data. The mode word at 0x2DD408 is 1 in the boot program's data. The clean side must not read these; they are listed so the open questions can be confirmed against the disc.
- Original memory-card names (for the import feature only; the clean side never uses these for its own file):
  - Card directory: `BASCUS-97367-0`, the product code `BASCUS-97367` followed directly by the suffix `-0`, with nothing between them. The card side builds the name by joining the two strings (High; Open question 13 resolved).
  - Save file inside it: `savegame.01`, 0xC400 bytes. This is the only save file name the card layer uses. The card layer names no other save, bonus, link, online or tie-in file (Medium-High).
  - Icon file: `icon1.ico`. Card metadata file `icon.sys` carries the title "Neopets: The Darkest Faerie". The icon contents are not described.
  - These strings exist in the boot program's data at virtual addresses 0x3163A0 (product code), 0x316378 (suffix), 0x316390 (save file name), 0x316380 (icon name).
- Card message text: the player-facing memory-card messages are text records in GAMEFILE resource 533 (see Behavior, "Card message records"). The record a card box shows is taken from design table 9 (SPEC-0024; 27 records of 16 bytes, the system message boxes), which the text-database loader fills from resource 533 at start-up. Data address 0x43C468 holds a pointer to that table's descriptor (a header pointer and a record base). The pointer lies outside the file image, so the clean side builds the table from resource 533 itself (High for the source and for which job shows which record). An index i is valid when 0 ≤ i < the record count in the table header, and record i starts at the record base + 16 × i.
- Busy-box hold time: the float word at data address 0x2DD410 holds 2.0 (seconds). A busy card box holds for trunc(2.0 x 60) = 120 passes after its first 3 (Behavior, "Card message records"; High).
- Card-layer event names: a table of 24 text pointers at data address 0x2ECCF8 names the 24 event codes of the card layer, one per code in code order (Behavior, "Event codes"). The names are developer text and are never shown on screen (High).
- Save-point hint text: GAMEFILE resources 678, 686 and 691 (the Act 1 chores area) carry the hint text about save points (see Behavior, "Save points").
- Save points: kind-4 records of each world pack's own embedded character region (chunk type 32; record layout in SPEC-0006). On disc there are 88 of them in 60 world packs, 1 to 4 per pack, all with flag word 0 (High). The save-point constants (225.0, 25.0 and 2.5e9) are fixed constants of the boot program, not data words (High).

## Layout

### File and slots

| Item | Value | Confidence |
| --- | --- | --- |
| File size | 0xC400 (50,176) bytes | High |
| Slots | 4, stored one after another | High |
| Slot k starts at | k x 0x3100 (slot 0 at 0, slot 1 at 0x3100, slot 2 at 0x6200, slot 3 at 0x9300) | High |
| Slot size | 0x3100 (12,544) bytes | High |
| Valid slot index | 0 to 3. An index of 4 or more has no slot (see "Slot index"). | High |
| Byte order | little-endian for all multi-byte fields | Medium (assumed for this platform; not separately confirmed) |

### Slot field map (one slot, 0x3100 bytes)

Offsets are slot-relative. Sizes are in bytes. Hero index h is 0 for Tor and 1 for Roberta (SPEC-0036). "World index" is the key-table position of a world (Where it lives on the disc). Neither item container belongs to one hero: container A is the party inventory and container B the stash (Item lists). The spec named in a row owns the live state; this spec owns the slot layout of all of them.

| Offset | Size | Type | Meaning | Confidence |
| --- | ---: | --- | --- | --- |
| 0x000 | 16 | 4 x f32 | Hero vector A: the active hero's position (the position quad of its movement component, the sub-object at hero +0x16C, block +0x50). On load it becomes the hero's restart position at the end-of-pass apply (Behavior, "Restart point and the end-of-pass apply") | High |
| 0x010 | 16 | 4 x f32 | Hero vector B: the active hero's facing (the direction quad of the same movement component, block +0x2B0). On load it becomes the hero's restart facing | High |
| 0x020 | 4 | u32 | Active hero index (0 or 1), from the active hero's index word. On load, after the hero restore, the active hero is switched to it when it differs (SPEC-0053) | High |
| 0x024 | 4 | s32 | Key A: saved id of the world in the transition buffer (T). -1 (0xFFFFFFFF) = empty slot. On load it is posted into T, not as the current world (SPEC-0026) | High |
| 0x028 | 4 | s32 | Key B: saved id of the world in the exterior buffer (E). On load it is posted into E as the current world | High (Medium-High for the world entered; see World keys) |
| 0x02C | 4 | s32 | Key C: saved id of the world in the interior buffer (I). Written, never read | High |
| 0x030 | 93 | bytes | Progress flags. Flag n is bit (n AND 7) of byte (n shifted right by 3). Flags 0 to 742 are valid | High |
| 0x08D | 3 | - | Not written, not read | High |
| 0x090 | 368 | 92 x u32 | Engine block: the movie "played" words. Word i (at 0x090 + 4i) belongs to movie-table index i; 1 = that movie has played, 0 = not yet. Read by HasMoviePlayed (SPEC-0030). Copied from the movie object on save and back on load | High |
| 0x200 | 4 | u32 | Party Neopoints wallet (container A +0x14; SPEC-0021, SPEC-0050) | High |
| 0x204 | 4 | u32 | Stash Neopoints (container B +0x14): the wallet that the Act 2 start moves into the stash (Item lists) | High |
| 0x208 | 4 | u32 | Luck, 0 to 24 (hero +0x15EC; SPEC-0021). Saved from Tor; both heroes always hold the same value. On load each hero's luck is set to 0, then this value is added with the clamp 0 to 24 (Loading, step 12d) | High |
| 0x20C | 4 | u32 | Tor maximum health (+0x158). Restored by maxima rule 1 | High |
| 0x210 | 4 | u32 | Tor maximum magic (+0x160). Restored by maxima rule 2 | High |
| 0x214 | 4 | s32 | Pet type id (SPEC-0047). -1 (0xFFFFFFFF) = no pet: written when there is no live petpet or its remaining time is below 1, and also the empty-slot value. On load a type of -1 removes any petpet at scene start | High |
| 0x218 | 4 | s32 | Petpet remaining time T (pet +0xE34). Restored into the pending remaining time on load | High |
| 0x21C | 4 | s32 | Petpet full time F (pet +0xE38). Restored into the pending full time on load | High |
| 0x220 | 160 | 20 x pair | Container A (party inventory), kind 0: the item grid (capacity 20) | High |
| 0x2C0 | 480 | 60 x pair | Container A, kind 7: quest items (capacity 60) | High |
| 0x4A0 | 112 | 14 x pair | Container A, kind 8: charms (capacity 14) | High |
| 0x510 | 16 | 2 x pair | Container A, kind 1: weapon family (capacity 2) | High |
| 0x520 | 16 | 2 x pair | Container A tail: the kind-5 (body outfit) entry of hero 0, then of hero 1. While a hero wears the Tattered Cloak, the outfit the cloak replaced | High |
| 0x530 | 16 | 2 x pair | Container A, kind 2: shield family (capacity 2) | High |
| 0x540 | 160 | 20 x pair | Container B (stash), kind 0 | High |
| 0x5E0 | 480 | 60 x pair | Container B, kind 7 | High |
| 0x7C0 | 112 | 14 x pair | Container B, kind 8 | High |
| 0x830 | 16 | 2 x pair | Container B, kind 1 | High |
| 0x840 | 16 | 2 x pair | Container B tail: body outfit entry of hero 0, then hero 1 (same rule as 0x520) | High |
| 0x850 | 16 | 2 x pair | Container B, kind 2 | High |
| 0x860 | 8 | 2 x u32 | Current health, hero 0 then hero 1 (+0x154 of each hero). Restored after the maximum is set, clamped to it | High |
| 0x868 | 8 | 2 x u32 | Current magic, hero 0 then hero 1 (+0x15C of each hero). Restored like 0x860 | High |
| 0x870 | 56 | 14 x f32 | Hero 0 status timers, in seconds (live ticks x 1/60). Status index 0 to 13 (see Status timers) | High |
| 0x8A8 | 56 | 14 x f32 | Hero 1 status timers | High |
| 0x8E0 | 8 | 2 x s32 | Fire amount word, hero 0 then hero 1 (hero +0x14DC; SPEC-0036). Always -1 for heroes (Hero status words) | High |
| 0x8E8 | 8 | 2 x s32 | Fire period word (hero +0x14E0) | High |
| 0x8F0 | 8 | 2 x s32 | Poison amount word (hero +0x14D4) | High |
| 0x8F8 | 8 | 2 x s32 | Poison period word (hero +0x14D8) | High |
| 0x900 | 17 x 193 = 3,281 | per-world record | Table T1: the broken-breakable sets (SPEC-0048). Row = world index. Bytes 0 to 15: 128 bits, bit = breakable script index (below 128). Byte 16: unused | High |
| 0x15D1 | 33 x 193 = 6,369 | per-world record | Table T2: the per-world kill memory (SPEC-0006). Row = world index. Bytes 0 to 31: 256 kill bits of that world. Byte 32: copied both ways, no reader known | High |
| 0x2EB2 | 25 | 193-bit set | Once-per-area set of item 19 (Four-Leaf Clover): bit = world index (Once-per-area sets) | High |
| 0x2ECB | 25 | 193-bit set | Once-per-area set of item 20 (Twelve-Leaf Clover) | High |
| 0x2EE4 | 25 | 193-bit set | Once-per-area set of item 151 (Golden Negg) | High |
| 0x2EFD | 25 | 193-bit set | Once-per-area set of item 174 (Red Negg) | High |
| 0x2F16 | 25 | 193-bit set | Once-per-area set of item 154 (Starry Negg) | High |
| 0x2F2F | 32 | 256-bit set | Collected-treasure set: bit i = row i of the collected table (rows 1 to 247; SPEC-0051). Bit 0 always 0 (Treasure sets) | High |
| 0x2F4F | 32 | 256-bit set | Held-map set: bit i = the held flag of treasure row i (rows 0 to 47; SPEC-0051) (Treasure sets) | High |
| 0x2F6F | 59 | text | Location name, zero-terminated (area name; "Unknown Location" if the area has none). "Empty" in an empty slot. Read by the picker only | High |
| 0x2FAA | 1 | u8 | Zero on save, a terminator guard after a 59-character name. Never read | High |
| 0x2FAB | 15 | text | Date text "%2d.%02d.%02d %2d:%02d": month, day, year, hour, minute in local time, 14 characters and a zero terminator. Read by the picker only | High (format), Medium-High (field order) |
| 0x2FBA | 1 | u8 | Progress tier = the act number, 1 to 4 (see Progress tier). Rebuilt on each save; display only | High |
| 0x2FBB | 191 | u8 x 191 | Per-item collected count. Index = item id (0 to 190), one byte each: how many of that item have been collected. Copied out on save and into the item-count object on load (Count table, luck and swap mode) | High (size), Medium (meaning) |
| 0x307A | 2 | - | Not written, not read | High |
| 0x307C | 4 | u32 | Hero swap mode (SPEC-0053: 0 Tor only, 1 Roberta only, 2 free, 3 to 5 as there). Template value 2 (free) | High |
| 0x3080 | 1 | u8 | Tattered Cloak (item 126) owned: set on save when either hero's cloak entry (container A +0x50 record of that hero) is not empty. On load it gates the cloak step (Loading, step 9) | High |
| 0x3081 | 1 | u8 | Hero 0 wears the cloak: set on save when hero 0's body entry (container A +0x48 record) holds item 126. Acts on load only while 0x3080 is set | High |
| 0x3082 | 1 | u8 | Hero 1 wears the cloak (same rule as 0x3081) | High |
| 0x3083 | 1 | - | Not written, not read | High |
| 0x3084 | 32 | options | Copy of the live options block (see Options block) | High (copy), Medium (fields) |
| 0x30A4 | 24 | 3 x s32 per hero | Equipment Mote words: hero h, slot i at 0x30A4 + 12h + 4i; slot 0 armour Mote, 1 shield Mote, 2 weapon Mote; -1 = none (Equipment Mote words) | High |
| 0x30BC | 4 | u32 | Roberta maximum health (+0x158). Restored by maxima rule 3 | High |
| 0x30C0 | 4 | u32 | Roberta maximum magic (+0x160). Restored by maxima rule 4 | High |
| 0x30C4 | 60 | - | Not written, not read | High |

Written by the save and never read by the load: 0x02C (key C), 0x2F6F, 0x2FAB and 0x2FBA (picker display only), and 0x2FAA. Read by the load and not written by the save: none.

Never written and never read (keep unchanged; they belong to no apply step): 0x08D to 0x08F (3 bytes), 0x307A to 0x307B (2), 0x3083 (1), and 0x30C4 to 0x30FF (60). That is 66 bytes. The save writer does not zero the slot. On a save over an existing slot, these bytes keep their old values, and so do any other bytes the writer does not set.

- **Progress flags.** The bitmap is copied to the slot on save and back into the live progress state on load. The live state is the same 743-flag array the game reads for the progress tier (see Behavior, "Progress tier"). An empty slot has the bitmap zeroed. Example: flag 332 is byte 41, bit 4 (332 = 41 x 8 + 4). Flag 0x14C (332), flag 0xF8 (248, byte 31, bit 0) and flag 0xDC (220, byte 27, bit 4) are the three flags used for the progress tier.
- **Engine block (movie played words).** The 368 bytes at 0x090 are the 92 movie "played" words, one u32 per movie-table index (index i at 0x090 + 4i), where 1 means the movie has played. The game sets a movie's word to 1 just before that movie plays (SPEC-0026, Movies), and HasMoviePlayed reads the word (SPEC-0030). Saving copies the 92 words from the movie object into the slot; loading copies them back (Loading, step 3). Before the copy, the load resets the movie object: all 92 words 0, the movie queue emptied, the return world and return transition set to -1, and the no-skip word set to 0. The queue and the return words are not saved. Keep the block byte for byte when rewriting a slot (High).
- **Petpet fields (SPEC-0047).** The type id is at 0x214, the remaining time T at 0x218 and the full time F at 0x21C. The save writer stores the live petpet's type id, T and F; with no petpet, or T below 1, the type is written as -1 (0xFFFFFFFF), not 0xFFFF. On load the pet pending-load flag is set, and the three values are copied into the pending pet type, remaining time and full time. At the next scene start a type of -1 removes any petpet, and a valid type spawns it 7 units in front of the hero with T and F from the slot (SPEC-0047; High). Byte 0x10C is not a petpet field.
- **Present flag.** The save writer computes the present flag from the sign bit of key A before it rebuilds the slot, and returns it to the caller as an output: the slot is present when that bit is clear. Loading treats a negative key A as empty (Loading, step 4), and the empty-slot template writes -1. For the value -1 the sign-bit rule and the -1 marker agree. They differ for other negative values, which the template never writes (Open question 57).

### Item lists

Each list entry is 8 bytes: u32 item id (the item's 16-bit id, sign-extended), then u32 value. The value is the entry's count word (High).

**Capacities (High).** Each container holds a fixed number of entries per kind: kind 0 (the Items grid, 20 entries), kind 7 (quest items, 60), kind 8 (charms, 14), kind 1 (weapon family, 2) and kind 2 (shield family, 2). A list never holds more entries than its capacity. The five kind blocks of one container hold 98 entries (0x310 bytes). The save writer fills each container's list in six groups, in the same order as the load (Loading, step 8): kind 0 (20), kind 7 (60), kind 8 (14), kind 1 (2), the tail (one entry per hero, 2) and kind 2 (2), 100 entries in all, which is the whole 0x320-byte container (Open question 58 resolved; High).

**Two item containers (High for the layout and the copy and merge; Medium-High for the act link).** List A (0x220 to 0x52F) belongs to container A and list B (0x540 to 0x84F) to container B. Neither container belongs to one hero.
- Container A is the live party inventory, and 0x200 is its Neopoints wallet.
- Container B is a stash. The Act 2 start (TriggerActStart stage 2; SPEC-0053) copies the party inventory and wallet into B, then clears the party inventory and gives it a fresh kit: Red Juppie x3, Purple Juppie x3, Diplomat Dress (item 127) x1 as Roberta's body entry, and 1150 Neopoints (SPEC-0021, Starting equipment). The Act 3 start merges B back: an empty party entry takes B's entry, the wallets add, and B is cleared. 0x204 is the stash's wallet.
- Each container is 0x320 bytes: the five kind blocks and a 16-byte tail. The tail sits between the kind 1 block and the kind 2 block (0x520 for container A, 0x840 for container B). Each tail holds the kind-5 (body outfit) entry of hero 0 followed by that of hero 1, in the same 8-byte format. Do not read the tails as equipment ids.
- Kind 9 (the cloak entry) is not stored as a list. The flags 0x3080 to 0x3082 carry it (Tattered Cloak below).
- Stat-up items (19, 20, 151, 154 and 174) never appear in the lists, because the add consumes them at once (SPEC-0021, Stat-up items).

**Save rule (High).**
- Each entry writes its id and its count. An entry with no item record still gets its count written, but its id word is left as it was in the slot. A list index past the list's count writes nothing.
- Tails: for hero h, the writer stores hero h's body entry (the container +0x48 record). When hero h wears the Tattered Cloak (its body entry holds item 126), it stores hero h's cloak entry (the container +0x50 record) instead, which then holds the outfit the cloak replaced. The B tail follows the same choice.

**Load rule (High).** Both containers are cleared and their wallets set from 0x200 and 0x204. Every pair is then re-added through that container's normal add (SPEC-0021), in the order of Loading, step 8.
- A pair whose value is below 1 is skipped, so a stale id under a count of 0 is harmless.
- The add caps a count at the item's carry limit.
- Kinds 1, 2 and 5 go to the hero named by the item's own owner field, not by the slot position.

**Tattered Cloak (item 126; High).**
- 0x3080 is set on save when either hero's cloak entry is not empty (count above 0 and an item), which means the cloak is owned. 0x3081 and 0x3082 are set when hero 0 or hero 1 wears it.
- On load, for h = 0 then 1: re-add A tail h and B tail h. Then, if 0x3080 is set:
  - when h = 0, add item 126 with count 1. Item 126 is category 9, so both heroes' cloak entries receive it;
  - if 0x3081 + h is set, swap hero h's body and cloak entries, so the cloak is worn and the outfit is stashed again.

### Equipment Mote words

Three 32-bit words are held per hero (slot index 0 to 2), at 0x30A4 plus 12 times the hero index plus 4 times the slot index: slot 0 the armour Mote, slot 1 the shield Mote, slot 2 the weapon Mote (hero +0x1544 + 4i; SPEC-0021). -1 means none, and the empty template holds -1. They are not stored in the item tails (High).

On load, slot i of hero h goes through the hero's equip setter with the cue flag 0:
- if the word already equals the saved value, nothing happens;
- otherwise the value is stored, and that slot's equipment-panel timer is set to 240 (SPEC-0023);
- for Tor, slots 1 and 2 also run his Mote follow-up (Medium);
- no equip cue plays (cue 0xA3 plays only with the cue flag 1).

### Status timers

- Each of the 14 values per hero is a status-effect timer. The 14 statuses are the game's status set in index order: 0 stun, 1 curse, 2 poison, 3 slow, 4 strength, 5 speed, 6 confuse, 7 fire, 8 no-curse, 9 no-poison, 10 no-slow, 11 poison attack, 12 health regeneration, 13 magic regeneration (High: the restore passes the index as the status code of SPEC-0036).
- On save, each value is the hero's remaining time for that status, in seconds: the live tick count multiplied by 1/60 (one tick is 1/60 s, SPEC-0007), written as a float32. A status with 90 ticks left writes 1.5. A status with 0 ticks left writes 0.
- An inactive status is not 0 in the live state. After a reset each live timer is -1 (not 0), so an inactive status writes -1/60 (about -0.0167 s). The file therefore holds a negative float for it (Medium-High). The load truncates that value to 0, which sets the timer to 0, not -1 (inactive either way; see Edge cases for the one visible effect).
- The scale is the length of one tick. It is not a stat multiplier.
- On load, the original passes each status through the hero's status-apply method (SPEC-0036, Status apply) with a four-word record: the status index, the saved value truncated toward zero to whole seconds, then -1 and -1 (High). The method sets the timer to the whole seconds x 60 ticks under its usual rules, so the fraction of a saved value is dropped and a status with under 1 s left is lost. Codes 6 (confusion) and 11 (poison attack) are ignored by the hero's method and are not restored. Codes 8, 9 and 10 write nothing for a value of 0, so those timers stay -1; the other codes are set to 0 for a value of 0. A restored immunity (8, 9 or 10) of 1 s or more clears the matching status (1, 2 or 3) if it was restored active just before it (the codes are applied in index order).
- Further restore rules (High):
  - before the restore, each hero's 14 timers are set to -1, the status display is reset and the hero's action state is set to 1;
  - the timer is set when the ticks are 0 or more, but the status display starts only when they are 1 or more;
  - code 1, 2 or 3 is refused, with its immunity effect, while status 8, 9 or 10 is active. In index order, statuses 8 to 10 are still -1 when 1 to 3 are restored, so this never refuses a restore. Live play never has both of a pair active, so nothing is lost;
  - a restored poison (code 2) or fire (code 7) first sets its two hero status words to -1. The four words are then set from 0x8E0 to 0x8FF (Hero status words).
- A clean implementation reproduces this: truncate each saved value toward zero, then apply it as whole seconds through the hero's status-apply rules. A clean side that keeps the fraction must mark it as a deviation.

Confidence: High that the values are status timers stored in seconds, for the order, and for the restore rule; Medium-High for the save form.

### Hero status words (0x8E0 to 0x8FF)

These are each hero's four poison and fire words: fire amount (0x8E0 + 4h), fire period (0x8E8 + 4h), poison amount (0x8F0 + 4h) and poison period (0x8F8 + 4h), one s32 per hero each (SPEC-0036). The hero constructor sets them to -1, and every caller that applies poison or fire to a hero passes -1, so a saved slot holds -1 in all eight words. -1 means the defaults: poison 2 HP every 4 s, fire 2 HP every 2 s. They are copied out on save and set after the status restore on load (High).

### Options block

The options block is 32 bytes at 0x3084. Saving copies the live options block into the slot. Loading copies the same 32 bytes back into the live block and then runs the options apply rule (SPEC-0035), so a load overwrites the current options. The apply sets only these: the three gains (each volume x 0.01, clamped to 0 to 1), the three slider knob positions, and the first pad's two inversion flags. Vibration is not applied by this step; the loaded value reaches the pad's rumble gate through the per-pass sync (SPEC-0033), within one pass. The unlabelled words +0x0C and +0x10 are copied but not applied (High).

| Offset in block | Size | Meaning | Confidence |
| --- | ---: | --- | --- |
| +0x00 | u32 | Invert Left-Right (0 or 1), default 0. Set by the Controls page. Flips the right stick X byte of the first pad. Not an audio setting | High (store and effect); Medium-High (which row sets it) |
| +0x04 | u32 | Invert Up-Down (0 or 1), default 0. Flips the right stick Y byte of the first pad. Not an audio setting | High (store and effect) |
| +0x08 | u32 | Vibration (0 or 1), default 1. Turns controller rumble on or off through the pad input (SPEC-0033). Not an audio setting | High (store); Medium (what it gates) |
| +0x0C | u32 | Unlabelled three-value cycler (0 to 2), default 1. Its menu handler decrements on Left, wrapping 0 to 2, and increments on Right modulo 3. No shipped screen reaches that handler (Open question 73), and what the value means is unknown | High (range, transitions and reachability); Low (meaning) |
| +0x10 | u32 | Unlabelled toggle (0 or 1), default 0. Its menu handler toggles the value while Left or Right is held. No shipped screen reaches that handler (Open question 73), and what the value means is unknown | High (range, transition and reachability); Low (meaning) |
| +0x14 | f32 | Music volume, 0 to 100, default 75. Scaled by 0.01 and clamped to 0 to 1 for the Music gain. Its slider knob has UI element id 0xCE | High (values); Medium-High (label) |
| +0x18 | f32 | Sound effects volume, 0 to 100, default 75. Same rule for the Sound effects gain. Its slider knob has UI element id 0xCF | High (values); Medium-High (label) |
| +0x1C | f32 | Voice volume, 0 to 100, default 75. Same rule for the Voice gain. Its slider knob has UI element id 0xD0 | High (values); Medium-High (label) |

- The live block is a boot-set object. Start-up writes the defaults before the front end runs: Invert Left-Right 0, Invert Up-Down 0, Vibration 1, cycler 1, toggle 0, and the three volumes 75. A new game does not reset the block (see New game and reset).
- Gains before any apply: the three gains start at 1.0 at boot. Start-up writes the volumes 75 into the block but does not run the apply rule, so the gains stay 1.0 until the first apply (SPEC-0035). Outside the options menu, a load is the only event that changes the gains. A game started from a fresh boot without visiting the Audio page runs at gains 1.0; after any load it runs at the slot's values (0.75 each for a slot saved with the default volumes) (High).
- The volume fields and knob ids do not send sound cues. Element ids 0xCE, 0xCF and 0xD0 are the knob elements of the three sliders, not cues.
- The three volume fields are Music (+0x14), Sound effects (+0x18) and Voice (+0x1C) (Open question 9).
- Defaults for an empty slot have two open questions (Open question 76).

### Location, date and progress

- The location name (0x2F6F, 59 bytes) is the area name from the game data, zero-terminated. It is taken from the current world's area record (the area word of its type-1001 record), or from the last transition-map area when that word is -1. If the area has no name, the text is "Unknown Location". An empty slot holds "Empty". The zero byte at 0x2FAA ends a name of the full 59 characters.
- The date text (0x2FAB, 15 bytes) has the pattern "%2d.%02d.%02d %2d:%02d": month (2 characters, space-padded), a dot, day (2, zero-padded), a dot, year (2, zero-padded), a space, hour (2, space-padded), a colon, minute (2, zero-padded) (Medium-High). It is taken from the real-time clock, whose record is (status, second, minute, hour, pad, day, month, year), all BCD. The clock is converted to local time with an offset of (zone minutes + 60 x daylight-saving flag - 540) minutes, from the console settings. Example: 20 October 2026 at 22:13 local time gives "10.20.26 22:13".
- The progress tier (0x2FBA) is set on each save from three progress flags and equals the act number (see Behavior, "Progress tier"). It is not applied on load.
- The five once-per-area sets and the two treasure sets are described below. Every one of them is saved and restored.

### Once-per-area sets (0x2EB2 to 0x2F2E)

Five 25-byte sets, one per stat-up item, at the offsets in the field map: item 19 (Four-Leaf Clover), 20 (Twelve-Leaf Clover), 151 (Golden Negg), 174 (Red Negg) and 154 (Starry Negg) (High).
- Each is a 193-bit set indexed by world index. Bit i is bit (i AND 7) of byte (i >> 3). The add of one of these items tests and sets the bit of the current world's index (SPEC-0021, Stat-up items).
- An index of 193 or more is never set and always tests clear, so bits 193 to 199 pass through a load and a save unchanged.
- The sets are copied out on save and copied back on load. The world reset clears them.

### Treasure sets (0x2F2F and 0x2F4F)

Two 32-byte (256-bit) sets for the treasures of SPEC-0051. Bit i is bit (i AND 7) of byte (i >> 3) (High).

- **Collected set (0x2F2F).** Let n be the collected table's row count (248 on disc), capped at 255. On save, for each row i from 0 to n - 1, bit i is set when row i's flag is above 0 and i is at least 1, and cleared otherwise. Bit 0 is always cleared. Treasure k is row 200 + k. Bits n to 254 keep the live set's previous value, and bit 255 is never written. On load, the 32 bytes are copied into the live set, then for each row i from 1 to n - 1 the row's flag becomes its bit (0 or 1). Row 0 is not changed.
- **Held-map set (0x2F4F).** Let m be the treasure table's row count (48 on disc). On save, bit i is treasure row i's held flag (nonzero = 1), for i from 0 to m - 1. On load, each row's held flag becomes its bit.
- **Dig-site switch on load.** For each held-map row i, the load searches the resident current world, and failing that the other resident world, for the switch-group entry with id 200 + i (the dig site of treasure i). When it is found, the site is set active when the bit is 1 and inactive when it is 0. The load does not test the collected set when it switches a site (High).
- The live sets are zeroed at start-up and by the world reset, so bits beyond the table counts pass through a load and a save unchanged.
- Quirk: a debug switch word (zero on disc) makes every held test true, so a save would write bits 0 to 47 as 1. It is not set in play.

### Count table, luck and swap mode

- **Count table (0x2FBB; Medium-High).** One byte per item id 0 to 190: how many times that item has been obtained. The +1 is made by the item-obtained announcement (the pop-up that names an item as the player gets it), once per announcement, whether the item was picked up by touch or handed to the hero by one of the two give paths; it is not raised by first sight of the item. The count stops at 255, and an id of 191 or more is ignored. While the announcement is already showing its large first-find panel, a further announcement is skipped as a whole, so that obtain is not counted. A count of 0 before the +1 marks the obtain as a first find. The count is compared with the item's carry limit in the drop spawner and in the pickup. The item screen shows it after the item's name as the count over the limit, in brackets, for items whose limit is above 1 (SPEC-0035). It is copied out on save and into the item-count object on load (Open question 88 resolved).
- **Luck (0x208; High).** Luck is 0 to 24 and raises the pickup luck gate (SPEC-0021). Every luck add (the clover pickups) adds to both heroes, so the two values are always equal. The save stores Tor's value.
- **Swap mode (0x307C; High).** The hero swap mode of SPEC-0053 (template 2 = free). It is saved and restored, so a slot saved during Roberta's locked act loads locked to Roberta.

### Per-world tables

Two tables with one entry per world key (193 entries each). The record index is the world index, the position in the world key table (High).

| Offset | Entry size | Count | Ends at | Meaning | Confidence |
| --- | ---: | ---: | --- | --- | --- |
| 0x900 | 17 | 193 | 0x15D1 | Table T1: bytes 0 to 15 the world's broken-breakable set, bit = breakable script index (below 128); byte 16 unused | High |
| 0x15D1 | 33 | 193 | 0x2EB2 | Table T2: the world's 33-byte kill memory block; bytes 0 to 31 are 256 kill bits, byte 32 is copied both ways with no reader known | High |

The saved ids run from 0 to 11070 (see Where it lives on the disc), so they cannot be direct indexes into a 193-entry table. Row n of T1 starts at 0x900 + 17n and row n of T2 at 0x15D1 + 33n.

- **T1 (SPEC-0048).** A break sets bit (script index) in the current world's row, for script indexes below 128. At world load each breakable's start mode is applied only when its broken bit is set: mode 1 gives state 0 (disabled), mode 2 gives state 1 with the drop cleared (back, but drops nothing), and any other mode, including the default 0, gives state 1. A breakable whose bit is clear keeps its pack state (1 on disc). The table is copied out on save and back on load.
- **T2 (SPEC-0006).** Each resident world holds a live 33-byte kill block (world record +0x40): 256 kill bits of its character records, then one byte. A world load copies its own row of the run-time kill table (data address 0x386740, 33 bytes per area slot) into the live block. Before the copy, the save flushes the live kill blocks of the resident worlds (the current and the second world) into their rows, so those worlds' kills are current in the slot. Between saves the run-time table is written only when a driver load releases a world for a request that is not a game load. A world released by streaming (a stream load into the transition or exterior buffer) or by an enter-interior or leave-interior job does not write its row, so kills made there since it was loaded are not stored and are not in a later save (SPEC-0006, Kill memory; Medium-High until confirmed at run time). On load the table is copied back, and a world loads its row when it is entered.
- Quirks: rows 167 and 180, the second entries of the duplicate keys 1391 and 1414, are never used, because the world index comes from the first match. Pack 532 is the only world pack with no key; its world index would be 193, past both tables. It is not reachable in play (High).

### Empty-slot template

A slot is empty when key A (0x024) is -1. An empty slot is built as follows (the whole slot is first zeroed):

| Offset | Value |
| --- | --- |
| 0x000 and 0x010 | the four floats (0, 0, 0, 1.0) each, so the w float of vector A (0x00C) and of vector B (0x01C) holds 1.0 (0x3F800000), not 0 |
| 0x020 | 0 (u32) |
| 0x024, 0x028, 0x02C | -1 (s32 each) |
| 0x200, 0x204 | 0 (u32 each) |
| 0x208, 0x20C, 0x210 | 0 (u32 each) |
| 0x214 | -1 (s32) |
| 0x218, 0x21C | 0 (u32 each) |
| 0x2F6F | `Empty` followed by a zero byte |
| 0x2FAB | space (0x20), then a zero byte at 0x2FAC |
| 0x307C | 2 (u32) |
| 0x30A4 to 0x30BB | -1 (s32 each, all 6 values) |
| 0x30BC, 0x30C0 | 0 (u32 each) |

Everything else is zero (a rewrite that stores a zero w in the two vectors differs from the original only in an empty slot, whose vectors are never read), including the progress bitmap (0x030 to 0x08C), the options block (0x3084 to 0x30A3; the defaults question is Open question 76), the item containers, the 191-byte count table and both per-world tables. The save writer does not use this template: it rebuilds one slot from live state. The template is used to fill the slots at start-up and before each card scan.

### Validity rules (no checksum)

- There is no checksum, CRC or sum field in the slot or in the file. The save writer and the load path compute no sum over the slot or the file (High for the save writer and the load path).
- Empty: key A is negative. The save writer's present flag is the sign bit of key A before the rebuild. The load treats any negative key A as empty and posts no world load for it. The picker rows, and the load screen's search for its starting cursor, treat a slot as empty only when key A is exactly -1. The template writes -1.
- Slot index: 0 to 3 only. An index of 4 or more copies nothing and returns nothing.
- File read: a load needs a read that raises no general failure and returns exactly 0xC400 bytes; otherwise record 14 (Load failed) shows (High). The save scan does not check its own read (Edge cases).
- Saved ids: a forward lookup miss writes 0xFFFF. The value is stored as a 32-bit word with its high half zero, so the sign bit is clear and the slot reads as present. A reverse lookup miss on load gives 193. Load then posts no world load for that key, and the slot is not refused. A buffer with no world resident at save time has no forward id, so the writer stores this lookup-miss value 0xFFFF for that key (Medium: the no-world case is taken as a lookup miss; the whole-slot empty value -1 is a different case).
- Other fields are not range-checked on load except the luck clamp (0 to 24), the hero maxima rules, the carry-limit cap of the item add and the skip of item pairs with a value below 1. A damaged slot therefore loads without complaint as long as key A is not negative.

### Card save header (card side, not the slot)

The card save header is the work area from which the card side builds the console's standard save-description file, `icon.sys`. Bytes 0x040 to 0x403 of the header (964 bytes, 0x3C4) are that file, written into the save folder as `icon.sys` when the folder is created (Create directory job). The header is not part of the 0xC400-byte save file, and the clean side's own file does not contain it. It is described for the optional import feature. Offsets are from the start of the header block; subtract 0x40 for the offset inside `icon.sys`.

| Offset | Size | Type | Meaning | Confidence |
| --- | --- | --- | --- | --- |
| 0x040 | 4 | ASCII | Tag: the letters P, S, 2 and D, in that order. This is the fixed signature that opens every console save-description file | High |
| 0x044 | 2 | u16 | Zero | High |
| 0x048 | 4 | u32 | Zero | High |
| 0x204 to 0x403 | 512 | bytes | Cleared (set to zero) before the name fields are written | High |
| 0x100 | up to 68 | two-byte glyph codes | Title: a zero-terminated string of two-byte glyph codes made from the source text at 0x41C by the single-byte rule of Name text and glyph codes (offset 0xC0 of `icon.sys`, the title the console's browser shows) | Medium-High |
| 0x144 | up to 64 | ASCII | Icon file name for the list view: copied as a zero-terminated text from the name (at +4) of the icon object named by the save record's word at 0x34 | Medium-High |
| 0x184 | up to 64 | ASCII | Icon file name for the copy view: from the icon object named by the word at 0x38 when it is nonzero, otherwise the one at 0x34 | Medium-High |
| 0x1C4 | up to 64 | ASCII | Icon file name for the delete view: from the icon object named by the word at 0x3C when it is nonzero, otherwise the one at 0x34 | Medium-High |
| 0x408 | 20 | ASCII | Folder name, copied with a 20-byte bound when the header is set up: `BASCUS-97367-0` (the save folder). Outside the 964 bytes; not written to the card | High |
| 0x41C | up to 31 | ASCII | Source text for the title, copied with a 31-byte bound when the header is set up. The game always stores the fixed text `Neopets: The Darkest Faerie` (27 characters), followed by zero bytes. Outside the 964 bytes; not written to the card | High |

The fields are written in this order: tag, clearing of the 0x200-byte area, the title, then the three icon names. The 964 bytes from 0x040 to 0x403 are written as one file with the same write steps as the save file (Write job). The tag is written on every build of the header. The load scan reads only the save file, so the game itself has no use for the tag on load (Medium); an import may check it to recognise a genuine save folder. Confidence: High for the range and the file name.

### Name text and glyph codes

Name fields hold glyph codes, not plain text. Each character is one 16-bit code, stored byte-swapped: the first byte in the file is the high byte of the code and the second is the low byte (High). The converter reads two tables from the boot program's data at runtime:

- Run table (virtual address 0x2ECD58): 33 entries of 4 bytes each, then a zero entry. Each entry is a u16 two-byte code with first byte 0x81, followed by a u16 ASCII value. The 33 entries form four runs by ASCII value: 16 entries for 0x20 to 0x2F, 7 for 0x3A to 0x40, 6 for 0x5B to 0x60, and 4 for 0x7B to 0x7E (High for the layout).
- Base table (virtual address 0x2ECDE0): three entries of 4 bytes, each a two-byte code with first byte 0x82 followed by an ASCII base value, for the digits, the upper-case letters and the lower-case letters in that order; then a zero entry (High for the layout).

Digits and letters are not stored as 10- or 26-entry tables. Each group uses its base entry plus an offset (High). The values themselves are not copied into this spec.

## Behavior

### Creating the save file

The file has four slots, each built in the order given under Layout. At start-up all four slots are set to the empty template before any card read. The save image is a plain byte array of 0xC400 bytes. It is not a memory pool.

### Slot index

- Valid slot indices are 0, 1, 2 and 3. The save screen draws four rows, one per index, and the slot copy on load accepts the same indices.
- A load with an index of 4 or more copies nothing and returns no data. The load stops and the world is not changed (High).
- Rows: the picker draws four rows, one per index. A filled slot's row shows the slot's location name, its date text and an act icon: the row type is the progress tier byte (0x2FBA, read as a signed byte) plus 2. A row is empty when key A is exactly -1; an empty slot's row shows the text "Empty" (SPEC-0035, section 7.1, for the row rules). No row shows a play time, because the slot holds none (High). The fourth row is slot index 3, the fourth 0x3100-byte record (High). The front-end labels "Game 1" to "Game 3" sit on a front-end page that nothing opens, and the picker never draws them; they are a leftover, not slot names, so no fourth label is needed (High; Open question 2). The row text is a display matter; it does not change the slot.

### Saving

1. Saving starts only from the front-end save screen, when the player confirms a slot. The save screen (it shows "Save Game" with Select and Back hints) is front-end record 18 (front-end state 22; SPEC-0035 section 7.3). It is opened only at a save point: a Square press within 5 units of a save point, in normal play with the hero not busy, sets the front end's current screen directly to record 18 (see "Save points"). No return code is involved, so no front-end handler returns its code value 0x168. The pause menu has no "Save Game" entry: its six items are View Map, View Quests, Load Game, Resume Game, Options and Quit (SPEC-0035), and no level host call opens the save screen (SPEC-0030). The labels "Game 1" to "Game 3" are not slot names (see Slot index); the slot picker draws four rows from the slot records. The game has no autosave and no automatic checkpoint save: reaching a save point only offers the save screen (High). A second save routine writes a slot with the same card rules, but nothing calls it, so it is dormant in this build (High). A clean implementation must not start a save on its own.
2. The save screen works in stages 0 to 4. Opening the screen sets the stage to 0 and the slot cursor to 0, so the picker starts on the first row. On every pass, in every stage, the selected hero is set to state 1 (idle), so the hero is held idle while the screen is open.
   - stage 0: start the save scan (Card jobs);
   - stage 1: wait until the save scan ends (Card jobs, Save scan; it reads the save file into the four slots and may offer a format), then show the four rows (each row shows the location name, or "Empty", the date text and the act icon). If the player cancels in the scan (record 9, Yes), the screen returns to play with no cue and nothing written;
   - stage 2, slot picker: the player moves the cursor between the four rows. The card is checked on every pass while the picker is up. Cross on a row rebuilds that slot (step 3), starts the save write job (step 4) and moves to stage 3. Triangle leaves without writing: it plays front-end menu sound 6, clears the music overrides and returns to play (front-end code 0x0B4) (High);
   - stage 3, write: wait until the save write job ends, then read the card state word. If it is 0 ("saved"), the save-complete sound (cue 0xA9) plays, the music overrides are cleared and the screen returns to play (code 0x0B4). If it is nonzero ("not saved"), the stage is set to 0 and the screen stays: the next pass starts a new card scan and the picker shows again;
   - stage 4, card gone: entered when the per-pass card check in the picker finds the card removed or swapped. It runs the save card-gone job (record 4, then record 9; Card jobs). Retry and No scan again (stage 0); Cancel save then Yes returns to play with no cue (High).
   Confidence: High for the stages and transitions, and High that a zero card state word means "saved, or the cancel confirm answered Yes inside the write job" (see Edge cases for the cue quirk); Medium for which pad bits are confirm and cancel in the picker (see Open questions 12).
3. The chosen slot is rebuilt from live game state, and only that slot is changed: hero vectors A (position) and B (facing), header keys (saved world ids of the worlds in the transition, exterior and interior buffers, each found by the forward lookup under Lookups, 0xFFFF on a miss), location name, date text from the real-time clock (BCD converted to decimal), progress tier, the progress-flag bitmap, the two item containers, the petpet type, remaining time and full time (SPEC-0047), per-hero values, status timers (as seconds), the per-world tables (after the kill blocks of the current and the second world are flushed into their T2 rows; kills in worlds released by streaming or by an interior job were never stored, SPEC-0006), the once-per-area and treasure sets, the cloak flags, the swap mode, the equipment Mote words, and the options block. The other three slots are not touched by this step. The rebuilt slot is then copied into the session snapshot and the pending flag is set, before the write starts and so before the card result is known (see "Session snapshot and pending flag"; High). Before the rebuild the writer keeps the chosen slot's present flag (1 when key A was 0 or more); that flag selects record 7 or record 10 in the write job (Open question 39).
4. After Cross the write job asks before writing: record 7 (overwrite) when the chosen slot held a game, record 10 (save to the card) when it was empty, and no question on a card formatted during this scan (Card jobs, Save write). Yes writes; No goes back to a new card scan and the picker (High; Open question 26 resolved).
5. The whole 0xC400-byte array is written (Medium-High). Its four slots are, in order: the chosen slot (rebuilt in step 3) and the other three exactly as they stand in the array. The array is filled by the save scan that runs before the picker. Before each scan all four slots are reset to the empty template, and then, on a formatted card with the save folder, the save file is read into the array by a read-file request (dispatcher kind 2). The scan does not check the result of that read. So the other three slots are the bytes the scan read, or the empty template where the file was not read or could not be read (for example on a fresh card; see Edge cases). Whether the file is one card transfer or several is open (Open questions 24). The save writer also has a per-slot form: it takes a slot index k below 4 and writes that slot's 0x3100 bytes at offset k x 0x3100 of the file image (Medium). The per-slot form and the whole-array write use the same slot offsets, so both fit the layout; which one the card transfers use is open (Open questions 24).
6. The write job starts on the pass of the confirm; there is no frame delay before it (Medium-High). It first asks its question (step 4), then shows the busy record 5 for at least 123 passes before and while the card write runs (Card message records). The save screen waits pass by pass until the job ends. The save and load screens have no frame counter of their own; apart from the busy-box minimum, the wait lasts as long as the card driver takes (Medium-High). The clean side may write immediately and leave the write stage as soon as its own write has completed, or keep the busy text for 123 passes (Clean-side mapping of the card jobs).

### Card results (what the game does with them)

The save and load screens act on the results of the memory card. They never act on the card library's own codes.

1. No card. The save scan shows record 4 (Retry / Cancel save) and the load scan record 13 (Retry / Cancel load). Nothing is written and nothing changes (High).
2. Unformatted card. The save scan offers a format (records 2, 20, 21 and, on failure, 24; Card jobs, Save scan); after a good format the save goes on with no overwrite question. The load scan never formats: it shows record 11 (no save data). The two test card paths whose unformatted rules differed (a main card-state path that writes nothing, and a secondary save path that writes a different transfer block) are both dormant: nothing sets their request words and nothing calls the second save routine (High; Open question 21 resolved).
3. Directory check. Each save first checks whether the game's save directory exists on the card. This holds for the reachable save write job too: folder missing, it makes the folder and its icon files and then writes the save file; folder present, it writes the save file only.
   - Directory present: only the save file is written, using the smaller transfer block (0x88 bytes = 136 bytes).
   - Directory missing: the directory and its support files are created, then the save file is written, using the larger transfer block (0x110 bytes = 272 bytes) (confidence: Medium-High for the rule and the block sizes).
4. Query result. This rule belongs to the dormant transfer state machine only; no player-visible job reads it. The common job wait helper reports readiness separately from the card operation's result: in normal wait mode, helper result 0 means the operation is still pending; when it reports completion, it copies the operation result to a separate output. In the dormant transfer state machine, the card query's own result is then checked: zero selects the failure path and any non-zero value selects the completed path. A failed query stops the transfer silently, with no retry and no message (High for the separate values and branch; Medium for the external query-result meaning).
5. Card gone in the picker. The save and load pickers check the card on every pass. A card that is gone, or that is not the card seen before (swapped), counts as removed and starts the card-gone job (record 4 for save, record 13 for load; Card jobs). After a write or a card-gone job, a nonzero card state word sends the save screen back to stage 0: a new card scan, then the picker again (High).
6. Insufficient space. The card side reports it only when the save folder is missing and the free space is below the need; overwriting an existing save never needs free space. It shows record 3 in the save scan and the save write, and record 0 at the boot check (High).

These results apply to the save file only. The clean side writes its own file (Summary).

### Save and load entry (top layer)

Confidence: High for the states and transitions; Medium for the meaning of the return codes; Low for the request-code special case.

The save or load request goes through a small top layer with five states: 0 idle, 1 start, 2 running, 3 finished with success and 4 finished with error. A status word beside the state reads 2 while running, 1 after success, and 0 after failure or when idle.

Entry. A request carries a block of 0x80 bytes and a flag.

1. If the machine is busy (state 1, 2 or 3), the entry returns 1 and changes nothing. An idle machine (state 0) and a machine in the error state (state 4) accept a request.
2. The entry clears the completion callback slot and a per-request counter, sets the status word to 2, and starts the card operation. If the card operation cannot start, the state becomes 4, the status becomes 0, and the entry returns 2.
3. If the operation starts, the 0x80-byte block is copied into the request record and the state becomes 1. A nonzero flag means asynchronous: the entry returns 0 and later steps carry the work. A zero flag means synchronous: the machine is stepped until it is idle, and the entry returns 3. The outcome is then reported through the callback.

Step (run by the caller while the machine is not idle):

- State 1 moves to state 2 and starts the card operation for the request. For request codes other than 0 and 8, an extra flag is set and the card layer is reset before the start (Low).
- State 2 steps the card operation. The operation may move the machine to state 3 or state 4.
- State 3 calls the completion callback with 1 if one is set, sets the status to 1, and returns to idle.
- State 4 releases the card operation, calls the completion callback with 0 if one is set, sets the status to 0, and returns to idle.

The completion callback is optional. A machine in the error state accepts a new request.

### Transfer sub-machines

Confidence: Low. Two variants of a transfer sub-machine are described. Both use states 0 to 3. Each returns 1 while it is preparing or waiting, 2 when its transfer step has completed, and 3 in state 3. Return meanings are not confirmed beyond this.

- Count variant. In state 0, when no block is current, it takes a 20-byte block from the save record at offset 0x408 and sets a count of 100 (Open question 61 gives its meaning). It then starts the transfer and moves to state 1, which steps the transfer on each call and moves to state 2 when the step reports done.
- Header variant. In state 0 it writes the card save header (see Layout), sets a transfer of 964 bytes starting at header offset 0x040, copies a fixed 20-byte block from the boot program's data, and starts. It then steps as in the count variant.

### Card operation state machine

Confidence: Medium overall. The state flow, the poll rule and the mapping of poll results to status codes are described (High for the mapping).

The card transfer runs as a state machine of 13 states (0 to 12). State 0 starts the operation, the odd states from 1 to 9 call a card step, and the even states from 2 to 10 poll for the result of the step before them.

- **State 0 (start).** If the slot record holds no data, the operation writes status code 0x0C and stops. If the slot has data but its flag is zero, it writes status code 0x10 and stops. Otherwise it moves to state 1 when a check on the slot data passes; if that check fails it writes status code 0x10 and stops.
- **States 1, 3, 5, 7 and 9 (card steps).** Each calls one card step: open, verify, read, write or close, in the order the operation needs.
- **States 2, 4, 6, 8 and 10 (polls).** Each reads a result from the card layer. The results are read per state (High):
  - State 2 (after the first step): 0 moves on to state 3. Any other result from -9 up (including a positive one) writes code 0x0D (not the same card); a result below -9 writes code 0x16 (general failure). Either way the machine then ends (state 12).
  - State 4 (the path or file lookup): a result of 0 or more moves on to state 5. A negative result ends the machine (state 12) with code 0x11 for -4, code 0x10 for -2, and code 0x0C for any other negative value.
  - State 6 (the file open): a result of 0 or more is kept as the file handle and the machine moves to state 7. For a negative result, -4 writes code 0x10 and any other value writes no code; either way the machine ends (state 12).
  - State 8 (the read; the result is the byte count): a result of 0 or more is stored as a whole number and the machine moves to state 9. A negative result writes code 0x16, and -3 writes code 0x12 first, so a corrupted file raises both 0x12 and 0x16; the machine then ends (state 12).
  - State 10 (close): a result of 0 or more moves to state 11 (success); a negative result moves to state 12 and writes no code.
  - A poll that has no result yet repeats on the next pass. A card step call (states 1, 3, 5, 7 and 9) that reports failure ends the machine (state 12) with no code.
- **State 11 (read complete).** Sets the success status and writes a diagnostic line that names the port, the slot, the two file names and the byte count. The text of that line is not part of this spec.
- **State 12 (finish).** Ends the operation with result value 3. Any state may move here instead of its next state.

Status codes are written through a helper that takes a code and a slot index. The codes this machine uses are 0x0C, 0x0D, 0x10, 0x11, 0x12 and 0x16. They are codes of the one card event set (Event codes): 0x0C no card, 0x0D not the same card, 0x10 no file found, 0x11 path not found, 0x12 file corrupted, 0x16 general failure (High for the names). This machine belongs to the dormant card paths (Card results, items 2 and 4): nothing sets its request words in this build, so no player-visible job runs it.

Test vectors (synthetic): a poll in state 2 that returns 0 moves the machine on to state 3. State 0 with a slot that holds no data writes status code 0x0C and starts no card step.

### Save job dispatcher

Confidence: High for the index rule, the end and the failure rule; Medium-High for the kind table (Open questions 48 and 50 resolved).

- The save work is a list of step records. Each record is 0x10 bytes and holds a kind value; the kinds are 0 to 8. The list has a current index and a step count.
- The dispatcher is called once per pass and runs the step at the current index. Each step runs its own small state machine and answers busy, done or failed.
  - Busy: nothing changes; the same step runs again on the next pass.
  - Done: the index moves to the next record. If a record remains, its step starts (when the next record needs a fresh card query first, that query runs to done before the step). When the index reaches the step count, the job's state word becomes 3 (finished, success).
  - Failed: the job's state word becomes 4 (finished, failure) and the list stops; later records do not run. If the job raised no event code before failing, the dispatcher raises 0x16 (general failure).
- Kinds and the steps they run:
  - kind 0: card query. It counts the entries of the save folder; when the folder is missing and the free space is below the need, it raises 0x15 (insufficient space) (Medium);
  - kind 1: create folder (the create-directory job, below);
  - kind 2: read file. It can raise 0x0C, 0x0D, 0x10, 0x11, 0x12 and 0x16, and it stores the number of bytes read;
  - kinds 3 and 4: write file, one shared step: kind 3 the save file, kind 4 an icon file (the write job, below);
  - kind 5: delete (the delete job, below);
  - kind 6: not implemented; it raises 0x17 (High);
  - kind 7: format;
  - kind 8: new-card check: it reports whether a card is present and whether it is the same card as before.
- The game's card jobs build these lists: read; write; create folder then write; format; format then create folder then write; query; and card check (Medium-High).

### Shared rules for the card jobs

- **Path (High).** There is no joined path. Each file job first makes the save folder the card's current folder, using the absolute folder path: a slash, then the 20-byte (0x14) folder name held in the save record, which is `BASCUS-97367-0`. It then names the file by its bare name from the step record (for example the save file, `icon.sys` or an icon file), relative to that folder. Open question 55 resolved.
- **Waiting (High).** After each card call a job waits through one shared wait step. The wait helper's return describes readiness: 0 means "still running" (the job stays in its state and tries again next pass), a nonzero return means "finished" and the call's operation result is copied to a separate output, and "no call pending" makes the job fail at once with no event, so the dispatcher raises 0x16. The jobs then interpret the operation result according to the call; they do not use the card operation state machine's separate result rule (Open questions 41 and 53 resolved in part).
- **Results.** The value -4 is the card library's per-call result, and its meaning depends on the call: "already exists" for a folder create, "not found" for a delete, "path not found" for a change of folder. So each job's handling below is the natural one for its call and none is inverted (Medium; Open question 49 resolved).
- **Events.** A failed job raises a code of the one card event set (Event codes), or none, in which case the dispatcher adds 0x16.

| Job | Result -4 | Other negative results | Zero or above |
| --- | --- | --- | --- |
| Create directory (kind 1), create call | The folder already exists: event 0x13, job fails | Treated as success (the job goes on) | Success |
| Delete (kind 5), change of folder | Job fails, no event (0x16 added) | Job fails, no event (0x16 added) | Goes on |
| Delete (kind 5), delete call | Accepted: the file is already absent; job done | Job fails, no event (0x16 added) | Job done |
| Write (kinds 3 and 4), change of folder | Path not found: event 0x11, job fails | Job fails, no event (0x16 added) | Goes on |
| Write, open, write and close calls | Job fails, no event (0x16 added) | Job fails, no event (0x16 added) | Goes on; the write call's value is the byte count |

### Create directory job (kind 1)

Confidence: High for the checks and their codes; Medium-High for the rest.

1. Checks, in order, on the device information of the step (the card state from the last card query) and on the save record. The first that fails raises its code and the job fails with nothing created:
   - card type word (device information +0x10) is 0: event 0x0C (no card);
   - formatted word (device information +0x14) is 0: event 0x0E (unformatted);
   - free blocks (device information +0x18) below the blocks the save needs (save record +0x840): event 0x15 (insufficient space);
   - the save record's folder-present word (+0x404) is 1 or more: event 0x13 (the folder already exists).
   Open question 51 resolved.
2. Create the folder at the absolute folder path (Shared rules) and wait. A result of -4 raises event 0x13 and the job fails. Any other result counts as created; the job logs a diagnostic line.
3. Write `icon.sys`: the 964 bytes of the card save header (Card save header), built just before this step, through the write job.
4. Write the icon file through the write job. Its data and name come from the step record; when the step record gives no data, a built-in record is used, named after the save folder, with its data size word set to 100 (Medium).
5. Done. The save file itself is written by the next record of the list (create folder then write). So "create folder" makes the folder and its two support files, and is the step the directory branch of Card results, item 3, runs (Open question 52 resolved; Medium-High).

### Delete job (kind 5)

Confidence: High.

1. Make the save folder current (Shared rules) and wait. Any negative result fails the job.
2. Delete the file named in the step record and wait.
3. A result of zero or above, or -4 ("not found", the file is already absent), ends the job as done, with a diagnostic line. Any other result fails the job with no event of its own, so the dispatcher raises 0x16 (Open question 56 resolved).

### Write job

Confidence: High for the order and the tests.

1. Make the save folder current (Shared rules) and wait. -4 (path not found) raises event 0x11 and fails the job; any other negative result fails it with no event.
2. Open the file by its bare name with the open value 0x203 and wait. In the card library's open modes 0x200 means "create the file if it is missing" and 3 means "read and write", so 0x203 is create plus read-write (Open question 54 resolved). A non-negative result is the file handle.
3. Write the data block of the step record (its data pointer and byte size words) and wait. A non-negative result is the byte count written; it is stored as the step's result. The count is not compared with the size asked for, so a short write still counts as success.
4. Close the file and wait. A non-negative result ends the job as done, with a diagnostic line giving the page, the slot and the byte count (the log text is not part of this spec).
5. A negative result at the open, write or close fails the job with no event, so the dispatcher raises 0x16. A failed open or write does not close the file.

### Event codes

Confidence: High for the set, the numbering and the names; Medium-High for the validation-error meanings.

The card layer keeps one 24-bit event set. Each code 0 to 23 is one bit of it. A raise sets the code's bit and a test reads it; the values -1 and 24 are ignored by both. The status codes of the card operation state machine and the event codes of the jobs are the same family: they are all codes of this set. The names are developer text in a 24-entry pointer table at data address 0x2ECCF8 (Where it lives on the disc); no name is shown on screen.

- Codes 0 to 11: request validation errors, raised by the request validator before a request runs (Request validator below).
- Codes 12 to 23: run-time results.

| Code | Meaning |
| ---: | --- |
| 12 (0x0C) | No card |
| 13 (0x0D) | Not the same card as before |
| 14 (0x0E) | Unformatted |
| 15 (0x0F) | Already formatted |
| 16 (0x10) | No file found |
| 17 (0x11) | Path not found |
| 18 (0x12) | File corrupted |
| 19 (0x13) | Create folder: the folder already exists |
| 20 (0x14) | Save folder does not exist |
| 21 (0x15) | Insufficient space |
| 22 (0x16) | General failure |
| 23 (0x17) | Not yet implemented |

- A request that fails without raising any code raises 22 (general failure).
- The game's card jobs test only three things: whether any code is raised, whether 21 (insufficient space) is raised, and whether 22 (general failure) is raised. No other code changes what the player sees.

Codes raised by the jobs of this section:

| Event | Raised by | When |
| --- | --- | --- |
| 0x0C, 0x0E | Create directory | The device information shows no card (0x0C) or an unformatted card (0x0E) (job step 1) |
| 0x13 | Create directory | The save record says the folder is present, or the create call returns -4 (the folder already exists) |
| 0x15 | Create directory; card query | The free blocks are fewer than the blocks the save needs; the query finds the folder missing and too little free space |
| 0x11 | Write | The change to the save folder returns -4 (path not found) |
| 0x17 | Kind 6 | Always (not implemented) |
| 0x16 | Dispatcher | A step fails without having raised any code (for example a failed open, write or close) |

### Request validator

Confidence: Medium-High. It has no player-visible effect.

This is the routine earlier called the save-slot planner. Before the card layer runs a request, the validator walks its list of four-word step records (the number of records is the list's first word) and raises a code from 0 to 11 for each request-building error it finds. The errors, in plain words: a required field is missing (the card, the folder or the file); a name is bad; a folder is used before the request creates it; the icon count is wrong; the token name is wrong; a format is not the first step; a format is not followed by a create-folder step; a new-card check is not alone in its request; and an error about synchronous running. Some checks compare a step with the step before it over 0x14 bytes. The record kinds are the dispatcher kinds (Save job dispatcher). The validator reports success when it raises no code.

### Diagnostic lines

The card layer writes developer status lines to a diagnostic log. Some of them carry a test marker. That text is developer text with no format effect: it does not change the file, the slots or the card result. The lines cover six cases: the card is absent; the card is unformatted, so nothing is saved; the save folder is missing and is created with its support files; the save folder exists, so only the save file is written; a query completes; a query fails. No on-screen use of these lines is known (Medium; the log's destination is not described). A clean implementation has no reason to show them.

### Card message records

Confidence: High unless a line says otherwise.

**Records.** Every memory-card message is a record of design table 9 (resource 533, 27 records; SPEC-0024), shown in the shared dialogue box (SPEC-0022). Record r has a text, a first label, a second label and a colour. The first label is the Circle answer (result 1) and the second the Triangle answer (result 0); a record with only one label still accepts both buttons. If both buttons are pressed in the same pass, the answer is 1. Text variable 0 holds the card slot number shown in the texts, always 1, because only the first card slot is used. Text variable 1 holds the free space the game needs, in KB, computed by the card side from the file sizes (Medium that it reads 106). The clean side may use its own wording with the same meaning.

| Record | Meaning | Labels (Circle / Triangle) | Shown by |
| ---: | --- | --- | --- |
| 0 | Not enough free space; insert a card with at least the needed KB | Retry / Continue | boot check |
| 1 | No card inserted; to save, insert a card with at least the needed KB | Retry / Continue | boot check |
| 2 | Card unformatted; format it? | Yes / Cancel save | save scan |
| 3 | Not enough free space to save | Retry / Cancel save | save scan, save write |
| 4 | No memory card in slot 1 | Retry / Cancel save | save scan, save write, save card gone |
| 5 | Saving; do not remove the card (busy) | none | save write |
| 6 | Save failed | Retry / Cancel save | save scan, save write |
| 7 | Overwrite the previous save? | Yes / No | save write, chosen slot filled |
| 9 | Cancel save? | Yes / No | every save cancel |
| 10 | Save to the card in slot 1? | Yes / No | save write, chosen slot empty |
| 11 | No save data for this game | Retry / Cancel load | load scan |
| 13 | No memory card in slot 1 | Retry / Cancel load | load scan, load card gone |
| 14 | Load failed | Retry / Cancel load | load scan |
| 15 | Loading; do not remove the card (busy) | none | load scan |
| 17 | Cancel load? | Yes / No | every load cancel |
| 19 | Card error | OK / none | save scan; effectively unreachable |
| 20 | Confirm the format | Yes / No | save scan |
| 21 | Formatting; do not remove the card (busy) | none | save scan |
| 24 | Format failed | Retry / Cancel save | save scan |
| 25 | Analog controller not detected | none | controller box (SPEC-0022) |
| 26 | Quit the current game? Unsaved progress is lost | Quit / Resume | quit page (SPEC-0035) |

Records 8, 12, 16, 18, 22 and 23 are never shown: the three success texts (save, load, format), the load "unformatted" text, the "checking" text and the "cancel format" text. No success message exists: a good save plays cue 0xA9 and returns to play, and a good load goes straight into the world load.

**Question box.** It shows the record and waits, pass by pass, for an answer. After the answer it waits two more passes and returns 1 (Circle) or 0 (Triangle). A box opened with the card watch also checks the card on each waiting pass: if no card is present, or the card is not the card seen before (swapped), the box returns -1 at once. The text stays up until the next box replaces it.

**Busy box.** It shows a busy record (5, 15 or 21) with no labels, waits 3 passes, then holds for trunc(2.0 x 60) = 120 more passes (the seconds word at data address 0x2DD410; rate 60, SPEC-0007). Then the card operation runs; when it ends, the box is cleared and two more passes go by. A busy text therefore stays up at least 123 passes (about 2.05 s) even when the card is quick (Medium-High for one pass per wait step).

### Card jobs as the player sees them

Confidence: High.

Each card job runs alone as a global task (mask 3), and the screen that started it waits until it ends. "Cancel confirm" means: show record 9 (save) or record 17 (load); Yes ends the job cancelled, No starts the job again from its first step, unless a job below says otherwise. A watch cancel is the -1 answer of a question box with the card watch.

**Boot check** (once per session, from front-end state 0, before the start countdown; SPEC-0035):

1. Ask the card for its state.
2. No card: record 1. Retry: go to step 1. Continue: end.
3. Card formatted and the check reports a problem: if it is insufficient space (no save folder and less free space than needed), show record 0 with the card watch, otherwise record 1. Retry: go to step 1. Continue: end. A watch cancel shows record 1.
4. Unformatted card, or no problem: end with no message.

**Save scan** (start of the save screen):

1. Reset the four slots to the empty template and ask the card for its state.
2. No card: record 4. Retry: start again. Cancel save: cancel confirm.
3. Unformatted: record 2 with the watch. Cancel save: cancel confirm. Yes: record 20 with the watch; No there starts again. Yes on record 20: busy record 21, then format. Format ok: the card is new (state N below) and the scan ends. Format failed: record 24; Retry starts again, Cancel save goes to the cancel confirm. A watch cancel on record 2 or 20 shows record 4.
4. Formatted with a problem: insufficient space shows record 3, any other problem record 6. Retry: start again. Cancel save: cancel confirm.
5. Formatted, no problem, no save folder: state F (folder missing); the scan ends.
6. Formatted, no problem, save folder present: read the save file into the four slots; state P (present); the scan ends. The scan does not check this read (see Edge cases).
7. Cancel confirm Yes ends the scan cancelled: the save screen returns to play with no cue and nothing written.

**Save write** (after Cross on a slot; the slot is rebuilt first, and its present flag is kept):

1. State N: no question; go to step 3 with the "create folder" form.
2. State F: record 10 with the watch. State P: record 7 with the watch when the chosen slot was filled, record 10 when it was empty. Yes: go to step 3 (the "create folder" form for F, the "save file only" form for P). No: end with "not saved", so the save screen scans again and shows the picker. A watch cancel shows record 4.
3. Busy record 5, then write. Create-folder form: make the save folder and its icon files, then write the save file. Save-file-only form: write the save file.
4. Write ok: end with "saved".
5. Write failed: insufficient space shows record 3 with the watch, any other failure record 6. Retry: end with "not saved" (a new scan and the picker, not an immediate rewrite). Cancel save: cancel confirm. A watch cancel shows record 4.
6. Record 4 here: Retry ends with "not saved"; Cancel save goes to the cancel confirm.
7. Cancel confirm: No ends with "not saved". Yes ends with "saved" (quirk; see Edge cases).

**Save card gone** (the save picker saw the card removed or swapped): record 4. Retry: "not saved" (a new scan). Cancel save: cancel confirm; No gives "not saved", Yes leaves the save screen to play with no cue.

**Load scan** (start of the load screen; it never formats):

1. Reset the four slots to the empty template; busy record 15; read the save file.
2. No card: record 13. Retry: start again. Cancel load: cancel confirm.
3. Unformatted card, or no save folder: record 11. Retry: start again. Cancel load: cancel confirm.
4. The read raised no general failure and returned exactly 0xC400 bytes: loaded; the scan ends. Otherwise record 14. Retry: start again. Cancel load: cancel confirm.
5. Cancel confirm Yes: the load screen is left with no load (SPEC-0035 section 7.2).

**Load card gone** (the load picker saw the card removed or swapped): record 13. Retry: scan again. Cancel load: cancel confirm; No scans again, Yes leaves the load screen.

**Quit confirm** (SPEC-0035): record 26. Quit (Circle) quits the game; Resume (Triangle) stays on the quit page. The quit path never writes a save.

### Clean-side mapping of the card jobs

A clean save system has no memory card, so it maps the jobs onto its own file. "No card", "unformatted" and "format" never happen, and the boot check has nothing to report. It keeps: the four-slot picker; the overwrite question (the meaning of record 7 when the chosen slot is filled, record 10 when it is empty); the cancel confirms; a failure box whose Retry goes back to the picker; and a load failure box (the meaning of record 14) when its file is missing, unreadable or not exactly 0xC400 bytes. It may keep the busy text for at least 123 passes to match the original feel. It may drop the save-cue quirk of the cancel confirm (Edge cases), but must say so.

### Save and load screens as the player sees them (UI pass)

Confidence: High for the order of boxes and the minimum times; Medium for the row heights. Positions of the rows, captions and hints are in SPEC-0035 (sections G.6 and G.7); the box layout is in SPEC-0022 (Dialogue box layout).

- Picture and caption: the load screen shows the load-game picture with the caption "Load Game" and no hint line. The save screen shows the identical picture with "Save Game" and a hint line (Cross "Select" at the lower left, Triangle "Back" at the lower right). Both captions are centred at x 256, baseline 40.
- Order on the save screen with a good formatted card that holds the save file: no box during the scan, then the four rows. Cross on a row: question box (overwrite if the slot holds a game, save-to-card if empty), then the busy box for at least 123 passes, then cue 0xA9 and play. With no card: record 4 (Retry / Cancel save) before any row; with an unformatted card: record 2, then record 20, then busy record 21 (at least 123 passes), then the rows with a freshly formatted card (no question on the later Cross).
- Order on the load screen: busy box (record 15, at least 123 passes) first, always; then the rows on success; or record 13 (no card), 11 (no save data) or 14 (load failed), each with Retry and Cancel load.
- Boxes are drawn over the unchanged picture at the box position of SPEC-0022: first baseline y = 345 - 10 L for L wrapped lines; the rows are not drawn while a box is open (the picker is shown only in the last stage). Circle answers the first label, Triangle the second; the box returns two passes after the answer.
- Worked example (save, card removed while the picker is up): rows shown, cursor on slot 1; the card is pulled; on the next pass the picker disappears and record 4 appears (Retry / Cancel save). Circle (Retry): the screen scans again; with the card back the rows reappear with the cursor still on slot 1 (High: only the screen's opening, the load screen's opening and the cursor-move rule write the cursor; the retry path resets only the stage, so a rewrite must not reset the cursor to slot 0). Alternatively Triangle (Cancel save) opens record 9 (Cancel save? Yes / No): Circle (Yes) returns to play at once with no cue and nothing written; Triangle (No) rescans.
- Worked example (load, one filled slot): busy box for 123 passes; rows appear with the cursor on the filled row and empty rows below it; Cross loads it. Up and Down only play cue 0x9A because no other filled slot exists.

### Save points

This spec owns the save-point rule. SPEC-0006 (kind 4) gives the record layout, SPEC-0034 the glow effects, SPEC-0031 the cue, SPEC-0035 (section 7.3) the save screen and SPEC-0055 the Continue path. Hint text in the Act 1 chores area (GAMEFILE resources 678, 686 and 691) tells the player that progress can be saved at the save points that glow; each of those three packs has one save point, at the same position.

**Records (High).** A save point is a kind-4 record of the world pack's own character region (the type-32 region created at world load; SPEC-0006). Its data holds the position (vec4, w = 1) at +0x00, a facing (vec4) at +0x10 and a u32 flag at +0x20. The facing in the record is not used by the save-point step; the restart facing comes from the hero. A region keeps at most 8 save points; further kind-4 records are dropped with an error report. On disc there are 88 save points in 60 world packs, and the flag is 0 in all 88. Save points are not trigger volumes, placed objects or placed effects (SPEC-0028). Other loaded character regions of an area (sub-regions) are updated without the save-point step, so only the world pack's own save points work.

**Per-tick step (High).** The step runs once per tick for the current world's own region, inside the world update (so not while the world update is off; SPEC-0026). Let P be the selected hero's position. For each save point k in list order, d2(k) = dx² + dy² + dz², the squared 3D distance from P to the record position.

1. Selection. V is the nearest save point with a nonzero flag and U the nearest with a zero flag. Each starts as "none" with a best squared distance of 2.5e9. A point replaces the best only when its d2 is strictly smaller, so the lowest index wins a tie. A point at d2 of 2.5e9 or more is never chosen. On disc every flag is 0, so only U is ever set.
2. Glow. The region owns one glow made of two effect emitters, effect definitions 400 and 404 of shared effect group 3 (SPEC-0034). V and U each have a remembered index, both "none" at region reset. When the chosen index differs from the remembered index of its branch, the glow is stopped, moved to the chosen save point's position and started again, and the remembered index is updated. The glow is updated every tick. It is not distance-gated, so the glow always sits at the nearest save point. Region reset clears the save-point list, sets the remembered indices to "none" and stops the glow.
3. Sound. When d2 < 225 (15 units), cue 0xA8 is kept playing on the main bank slot (SPEC-0031): it is started when no voice with the same slot, cue and emitter is playing, otherwise that voice is refreshed. The cue is started with the "do not restart" flag and the default volume. This test is not under the gate of step 4.
4. Prompt gate. Steps 5 and 6 run only when no in-play menu sub-state is open and the game-state code is 0 (SPEC-0026). A running interior enter or leave sequence (code 6), a cut-scene, the game-over state or a timer state therefore hides the prompt and blocks the opening. Ordinary level script threads do not raise code 6, so they do not hide the prompt (Medium-High).
5. When d2 < 25 (5 units): the context prompt shows label 25 of text table 13 (SPEC-0035), the "Save Game" prompt, on this tick. If Square has a press edge on this tick (bit 0x0080 of the selected hero's pressed word) and the hero is not busy (the busy test of SPEC-0033), then, in the U (zero-flag) branch, the hero's restart point is set to (P, the hero's facing), and the save screen is opened (front-end section D record 18; SPEC-0035 section 7.3; Saving, steps 1 and 2). The prompt shows even when the hero is busy; only the opening is blocked.
6. When d2 >= 25: an away flag is set to 1. Opening the save screen sets it to 0. The flag has no reader: nothing reads the word directly, its getter has no caller and no stored pointer, and the only other writers (the world reset, the save-screen opener and one routine that nothing calls) all clear it. A rebuild may leave it out (Open question 82 resolved; High).

The V (nonzero-flag) branch runs the same steps 2 to 6, except that a Square press does not set the restart point. No disc record reaches it, and nothing in play writes a save-point flag (SPEC-0006, Open questions), so a clean build may leave this branch out.
**Order against the hero's Square handling (High).** The save-point step runs in the region tick, earlier in the world update than the hero update, and it reads the same Square press edge. Neither step consumes the edge. So one press within 5 units of a save point opens the save screen and, in the same tick, the hero's world interaction chooser (SPEC-0022) also acts on it: it can start a talk, run a purchase or confirm a door. The "Save Game" caption is set first in the pass, so any caption the chooser sets later in the pass replaces it (the last writer wins; SPEC-0023, section 33), and the chooser's hide step in its special-state branch hides it. A "Save Game" caption already shown in the pass also stops the chooser's idle list before Feed petpet, Read Book and the no-target Square stance entry, so those do not act on that press.

The three constants 225.0, 25.0 and 2.5e9 are fixed constants of the boot program (High). The save screen's first pass runs in the same tick as the Square press, because the front end runs after the world update in the frame loop (High). Saving does not move the hero: the restart point is the hero's own position and facing, not the save point's position.

### Restart point and the end-of-pass apply

- Each hero has a restart point: a position and a facing (16 bytes each). One hero method writes both (High). The placement routine puts the hero at the restart position. It puts the camera at (position − 10 × facing + 2.5 × W), where W is the constant up vector (0, 1, 0, 0) at data address 0x3932E0 (written once at boot, never changed), looking along the facing, with the camera's right axis = normalise(W × facing) (High for W and the eye position; Medium for the follow-up camera steps, SPEC-0027).
- Writers of the restart point: Square at a save point (U branch, above); the end-of-pass apply (below); the area entry and a game-state timer (SPEC-0006, SPEC-0055; Medium).
- Saved vectors. The save writes vector A (slot 0x000) = the active hero's position and vector B (slot 0x010) = the active hero's facing. These are the same values a save-point Square press records as the restart point (High).
- End-of-pass apply (High). A load (Loading, step 6) copies vectors A and B into a pending-placement block and sets the apply flag. At the end of every frame-loop pass, while the apply flag is set: the hero's restart point is set to (A, B), the hero is placed (and the camera with it), both stored vectors are set to (0, 0, 0, 1), and the flag is cleared. The pet restore of SPEC-0047 runs in the same step; where this spec or SPEC-0047 says the pet is restored "at the next scene start", this end-of-pass step is meant. It is not only a scene-start step: it runs at the end of whichever pass first sees the flag.

### Session snapshot and pending flag

- Confirming a slot in the save picker (Saving, step 3) or in the load picker (Loading, step 3) copies that slot (0x3100 bytes) into a session snapshot buffer in memory and sets the pending flag (High). The snapshot buffer is allocated by the card state on its first pass (SPEC-0035; Medium-High).
- The save sets the flag when the write starts, before the card result is known, so a failed write still leaves the snapshot and the flag (Medium for what the player sees).
- Only the world teardown (quit, or game over without the flag) clears the pending flag (High).
- Game-over Continue with the flag set re-applies the snapshot from memory, with no card access, exactly as a load applies a slot (Loading, steps 1 to 17), then resets the heroes and returns to play (SPEC-0055 section 4). The flag stays set, so a later game over continues from the same snapshot. Before any save or load in the session, Continue leads to the title flow instead (High for the steps; Medium-High for the player view).

### Progress tier (byte 0x2FBA)

The save writer sets the tier byte at slot offset 0x2FBA on every save. The value is 4, 3, 2 or 1, chosen by three progress-flag tests, tested in this order: tier 4 if flag 0x14C (332) is set; otherwise tier 3 if flag 0xF8 (248) is set; otherwise tier 2 if flag 0xDC (220) is set; otherwise tier 1. Each test reads one bit of the progress-flag bitmap. A flag index at or above 743 reads as clear. High for the rule. The tier is a progress level, not a count of save slots, so it does not set how many slots exist (the slot count is four, see Slot index). The tier byte is a display field; it is rebuilt on each save and not applied on load (High).

The tier is the act number. The scene setup routine (SPEC-0036, Respawn) is the act-start setup of SPEC-0053 (TriggerActStart, section 3.7). It sets flag 220 (0xDC) at the start of Act 2 (stage 2), flag 248 (0xF8) at the start of Act 3 (stage 3), and flag 332 (0x14C) at the start of Act 4 (stage 4). These are the three flags the tier reads, so tier 1 means Act 1 and tiers 2, 3 and 4 mean Acts 2, 3 and 4 (High).

### Loading

Card read and copy-out (steps 1 to 3 of the card side):

1. The load scan reads the card file (Card jobs, Load scan). No card shows record 13; an unformatted card or a missing save folder shows record 11. Otherwise the read must raise no general failure and return exactly 0xC400 bytes; if not, record 14 (Load failed) shows with Retry / Cancel load. In every one of these cases nothing changes.
2. The load screen shows the four slot rows from the scan (same as saving).
3. On confirm, the chosen slot is copied out of the read image into the working area: the whole 0x3100-byte slot. An index of 4 or more copies nothing and the load stops (see "Slot index"). The slot is also copied into the session snapshot and the pending flag is set (see "Session snapshot and pending flag").

Apply order. After the slot is copied out, the steps below run in this order (the numbers are the order of the original's load path):

1. Reset the engine state that the slot does not hold (the list is under "What is not saved, and what a load resets"). This is not the world reset of New game and reset, and it does not read slot data.
2. Copy the progress bitmap (0x030, 93 bytes) to the live progress state.
3. Copy the engine block (0x090, 368 bytes, the 92 movie played words) back into the movie object. At the start of the apply the screen effect is also reset, the screen turns white at once (full fade alpha), and the post-load fade-in is armed: after the load, 6 consecutive non-busy passes later, the screen fades in from white over 8 ticks (SPEC-0026, load game; High).
4. Key A (0x024). If it is 0 or more and the reverse lookup finds it, post a load of the world key at that index into the transition buffer (T), with the current flag 0. If it is negative, or the lookup finds nothing (193), no load is posted for it. No error is raised. Confidence High.
5. Key B (0x028). If it is 0 or more and found, post a load of that world key into the exterior buffer (E), with the current flag 1, so E becomes the current world. This is applied even when key A was empty or unknown. Key C (0x02C) is not read. Both posts use the plane words -1 and -1, which mean "no named entry point" (see World keys and the world after a load). Confidence High (Medium-High for the world entered; see "World keys and the world after a load").
6. Set the apply flag. Copy the hero vectors (0x000 position and 0x010 facing) to the pending-placement block. Set the pet pending-load flag, and copy the pet values 0x214, 0x218 and 0x21C into the pending pet type, remaining time and full time. The pet is restored at the next scene start (SPEC-0047).
7. Container A (party): clear it and set its wallet from 0x200. Container B (stash): clear it and set its wallet from 0x204.
8. Re-add the list entries through each container's normal add, in this order: kind 0 (container A and B interleaved, 20 entries each); kind 7 (60 each); kind 8 (14 each); kind 1 (2 each); the tails (2 each, with the cloak step of step 9 for each hero); kind 2 (2 each). Each pair is re-added with its value, and a pair whose value is below 1 is skipped. The kind is not stored separately: the container derives it from the item (Item lists, Load rule).
9. Cloak step, run in the tail pass after hero h's two tails (h = 0, then 1), only when 0x3080 is set: when h = 0, container A gets item 126 with count 1, which fills both heroes' cloak entries; then, if 0x3081 + h is set, hero h's body and cloak entries are swapped, so hero h wears the cloak again (Item lists, Tattered Cloak; High).
10. World tables: write back all 193 T2 records (33 bytes each) and all 193 T1 records (17 bytes each).
11. Sets: write back the five once-per-area sets (0x2EB2 to 0x2F16). Then copy the collected-treasure set (0x2F2F) and the held-map set (0x2F4F) into their live sets and unpack them into the collected table and the held-map flags, switching the dig sites in the resident worlds (Treasure sets).
12. For each hero h = 0, then 1:
    a. Reset the hero's status timers to -1 (all 14), reset its status display, and set its action state to 1.
    b. Status restore: for status index 0 to 13, read the saved float, truncate it toward zero to whole seconds, and pass the four-word record (index, seconds, -1, -1) to the hero's status-apply method, which sets the timer to seconds x 60 ticks (Status timers; SPEC-0036).
    c. Set the hero's four status words from 0x8E0 + 4h, 0x8E8 + 4h, 0x8F0 + 4h and 0x8F8 + 4h (Hero status words).
    d. Luck: set the hero's luck to 0, then add the value at 0x208 with the clamp 0 to 24. Both heroes therefore receive the same value (High).
    e. Maxima: set the hero's maximum health and maximum magic by the rules under Hero maxima and restore, using 0x20C and 0x210 for hero 0 and 0x30BC and 0x30C0 for hero 1. The hero's cached outfit index (+0x156C) is set to -1, so the outfit model is loaded again.
    f. Current values: set the current health from 0x860 + 4h, then the current magic from 0x868 + 4h. Each is clamped to the maximum set in step e: a value above the maximum becomes the maximum. There is no lower bound; a saved zero or negative value is stored as it is (High).
    g. Equipment: pass the words at 0x30A4 + 12h + 4i (i = 0 to 2) to the hero's equip setter, with the cue flag 0 (Equipment Mote words).
13. Active hero: if the value at 0x020 differs from the current active hero, switch the active hero to that index.
14. Count table: copy the 191 bytes at 0x2FBB into the item-count object.
15. Options: copy the 32 bytes at 0x3084 into the live options block, then run the options apply step. It sets only the three gains, the three knob positions and the first pad's inversion flags; Vibration reaches the pad through the per-pass sync, and +0x0C and +0x10 are not applied (see Options block).
16. Swap mode: set the hero swap mode (SPEC-0053) from 0x307C. A slot saved while the mode locks the party to Roberta therefore loads locked to Roberta.
17. Final world-change call. The end-of-pass apply later sets the restart point from the hero vectors (step 6), places the hero, and clears the flag.

Hero placement after a load (High). The hero is placed at the saved vectors by the end-of-pass apply, not at load time: at the end of the first frame-loop pass that sees the apply flag, the hero's restart point is set to vector A (slot bytes 0x000 to 0x00F, the position) and vector B (0x010 to 0x01F, the facing), the hero and the camera are placed there, both stored vectors are overwritten with (0, 0, 0, 1) and the flag is cleared (see "Restart point and the end-of-pass apply"; SPEC-0040 owns the movement component).

Load screen timing. The load screen starts the load scan, waits pass by pass for it (the busy record 15 stays up at least 123 passes), then shows the picker; confirm starts the copy-out. Apart from the busy-box minimum there is no fixed number of passes (Medium-High).

Refusals and checks: the read must raise no general failure and return exactly 0xC400 bytes, otherwise record 14 shows (High). A slot index of 4 or more copies nothing. There is no checksum.

Fields applied on load are the ones named in the steps above. The display-only fields (location name, date text, progress tier byte), key C and the guard byte 0x2FAA are not applied; they are rebuilt on the next save. The 66 untouched bytes listed under Layout are not applied.

### World keys and the world after a load

Confidence: Medium-High (Open question 81 resolved).

- The world objects of the three buffers hold their key in their first word. At save, key A is the world in the transition buffer (T), key B the world in the exterior buffer (E) and key C the world in the interior buffer (I), each as a saved id.
- On load:
  - key A, when found, posts a load into T with the current flag 0;
  - key B, when found, posts a load into E with the current flag 1, so E is current;
  - key C is never read, so a save made in an interior resumes in the saved exterior and transition pair;
  - both posts use the plane words -1 and -1. The first plane word is the entry-point index of the world being entered; -1 means "no named entry point" (High; Open question 89 resolved). The posted worlds are entered in the end-of-pass world-change step, T first and then E. On entry with no named entry point the hero (and the camera with it) is placed at the world's first start point: the position and facing of entry 0 of the world's start-point list, or the zero vector when that list is empty. In the same step, right after the entries, the apply flag is seen and the hero is placed again at vectors A and B (Restart point and the end-of-pass apply). No frame is drawn between the two placements, so the start-point placement is never visible. If the saved point is on transition ground, the normal ground search swaps to T on the next frame (SPEC-0026).
- The world index of each world is set from its key when the world is entered (Where it lives on the disc).

### What is not saved, and what a load resets

High unless noted.

- Never in the slot:
  - the partner hero's position;
  - the interior world (key C is unused);
  - the game clock and play time;
  - level-module data words, which a reloaded module restores to its disc values (SPEC-0030);
  - the lock-chest table and switch-group progress (per world, rebuilt at world load; SPEC-0048);
  - world items lying in the world and taken placements: they respawn at region load unless a veto applies (carry limit, once-per-area set, held map, or equipment that is not an upgrade; SPEC-0021);
  - the pet's live state (only the type, the remaining time and the full time are saved);
  - the camera, the music and the quick-item selection;
  - the journal's pending target and the selected map area;
  - the movie queue and the movie return words;
  - the hero action states (set to 1 on load);
  - the status display.
- The load also resets:
  - the screen effect, with a 0-frame white fade-out (SPEC-0026, flag G);
  - the camera play state and the camera focal word (512);
  - the ten projectile pools;
  - three item slots and four effect records;
  - the 17 boot-time global objects;
  - the camera, which is initialised again.
- The load does not run the world reset. Every saved live object is overwritten from the slot, and every other object is rebuilt when the posted T and E loads run.

### New game and reset

A new game (start index 0 to 8) writes no slot. It runs the world reset first, which:

- clears both item containers and their wallets (0x200 and 0x204), then adds the new-game kit to the party inventory through the normal add (SPEC-0021, Sources, New game);
- clears the 191-byte count table;
- clears the 93-byte progress bitmap;
- clears table T1 and table T2 (all 193 records each);
- clears the five once-per-area sets and the two live treasure sets;
- sets each hero's current and maximum pools to the start values: Tor 150 health and 90 magic, Roberta 120 health and 150 magic (current equals maximum);
- sets luck (+0x15EC) to 0 for both heroes;
- resets the status timers to -1 for both heroes;
- runs the engine's world reset.

The kit's Golden Negg is consumed by the add as a stat-up item, and the set and pool resets that follow undo its once-per-area bit and its stat-up, so a new game holds no Golden Negg and starts at the start pools (Medium-High; SPEC-0021).

The kit that stays is Farmer's Clothes (119) x1, Red Juppie (150) x3 and Purple Juppie (149) x3. Farmer's Clothes is a real entry in Tor's body slot of the category-5 list (the kind-5 entry of hero 0 in the container tail, below), not an empty default; every other equipment entry, both cloak entries and all Mote words start empty (SPEC-0021, Starting equipment; High). A save made right after a new game therefore writes the pair (119, 1) in the hero 0 pair of container A's tail, and a count of 0 in the hero 1 pair.

Then:

- Index 0 (plain new game): sets the two start-key globals to -1, so the start worlds fall back to world key 691 (saved id 11) in the transition buffer and world key 909 (saved id 602) in the exterior buffer, which is current. The start-key globals hold -1 in the boot program's data, and every store to them (new game index 0 and the title state) writes -1, so every start that reads them falls back to these two defaults; a rebuild can drop the globals (Open question 67 resolved; High). Sets flag 707 (0x2C3) and no other flags.
- Indices 1 to 8: sets flag 707 for every index. Runs the stage events listed for those indices in SPEC-0035 section 5. Each event sets its flag only when the runtime stage table has a matching entry; that table is not in the boot program. Indices 1, 4 and 5 call the world-change routine with 0 or 1. The cue calls in these indices reach a function that returns at once; it does nothing for any of its callers and no pointer to it is stored, so these calls play nothing and a rebuild plays no cue here (Open question 68 resolved; High). On every pass the new-game page also draws its queued text lines and adds their total height to the page's layout word (+0x20C of the front-end screen object); this is screen layout, not game state (Open question 72 resolved; Medium-High).

A new game does not reset the options block.

A fresh game has no pet. In play a petpet joins when the hero feeds a wild petpet (SPEC-0047, Acquisition); a pet in a loaded slot is restored at scene start. A new game writes no pet. It does not write the pet type id (0x214) or the two pet times (0x218 and 0x21C); those fields keep the empty-slot template values from start-up.

### Hero maxima and restore

The slot stores each hero's live maximum health and maximum magic, in the words named in the field map (0x20C and 0x210 for hero 0, 0x30BC and 0x30C0 for hero 1). It holds no experience and no level. Growth comes from stat-up items (SPEC-0036, Hero growth). On load, each maximum is taken from the first of its rules that applies, in the order below. The fallbacks use the caps and start values of SPEC-0036.

1. Tor health: if the word at 0x20C is from 1 to 500, the maximum is that word. Otherwise the maximum is 500.
2. Tor magic: if the word at 0x210 is from 1 to 300, the maximum is that word. Otherwise it is 300.
3. Roberta health: if the word at 0x30BC is from 1 to 400, the maximum is that word. Otherwise, let X be the word at 0x20C. If X is from 1 to 400, the maximum is 120 + 20 x T, where T = trunc((X - 150) / 25). Otherwise it is 400.
4. Roberta magic: if the word at 0x30C0 is from 1 to 500, the maximum is that word. Otherwise, let Y be the word at 0x210. If Y is from 1 to 500, the maximum is 150 + 25 x T, where T = trunc((Y - 90) / 15). Otherwise it is 500.

Current values. Each hero's current health and current magic are then restored from 0x860 + 4h and 0x868 + 4h, after the maxima are set, by the hero's current-value setters, which clamp only from above: a saved current value above the maximum is stored as the maximum, and any other value (including zero or a negative value) is stored unchanged. The setters' change notifications do nothing for a hero. This settles the question whether a slot word holds the current values (Open question 64). [High]

- The Roberta fallbacks (rules 3 and 4) read the hero 0 words (0x20C and 0x210), not Roberta's own words. This cross-hero reading is reproduced as written here (Medium-High).
- Truncation drops the fraction toward zero. For X from 1 to 149 the quotient is negative, so the fallback falls below 120: X = 100 gives 80.
- Synthetic checks: words 0x20C = 275, 0x210 = 210, 0x30BC = 0 and 0x30C0 = 0 give Tor health 275, Tor magic 210, Roberta health 220 (T = 5) and Roberta magic 350 (T = 8). Words 0x20C = 600 and 0x30BC = 0 give Tor health 500 and Roberta health 400. Words 0x20C = 300 and 0x30BC = 0 give Roberta health 240 (T = 6). Words 0x20C = 100 and 0x30BC = 0 give Roberta health 80 (T = -2).

Round trip (High; Open question 66 resolved). When a hero's own stored word is from 1 to its cap, it is passed unchanged to the maximum setter, which also sets the current value to it before step f restores the current value. A save and load therefore round-trips the maxima. The banded form is used only on Roberta's fallback (rules 3 and 4), when her own word is 0 or above her cap.

- All the numbers in the rules are growth words, read at runtime: the starts at 0x319BC0, 0x319BC4, 0x319BC8 and 0x319BCC, the gains at 0x319BD0, 0x319BD4, 0x319BD8 and 0x319BDC, and the caps at 0x319BE0, 0x319BE4, 0x319BE8 and 0x319BEC (Where it lives on the disc).
- The fallback's reference and step are not fixed constants. They are Tor's start word and Tor's gain word: reference 150 (0x319BC0) and step 25 (0x319BD0) for health, reference 90 (0x319BC8) and step 15 (0x319BD8) for magic. Roberta's start (0x319BC4, 0x319BCC) and gain (0x319BD4, 0x319BDC) give the 120 + 20 x T and 150 + 25 x T forms.
- A stat-up Negg raises every hero's maximum by that hero's gain with no cap check (SPEC-0021). A maximum above its cap therefore becomes the cap on the next load. A word of 0 (the empty template) also becomes the cap for Tor, and the fallback for Roberta.
- Setting the current values pokes the HUD bar timer when the value changes from the maximum (SPEC-0023).

### Lookups

- Forward lookup (world key to saved id): search the key table (part 1) for the world key, compared as a 16-bit signed value against each 32-bit entry, in index order. Return the saved id at the same position in part 2. If no entry matches, the result is 0xFFFF (High). The save writer uses this lookup for the three header keys, so a key with no entry is written as 0xFFFF (see Edge cases).
- Reverse lookup (saved id to world key): search the value table for the saved id. Return the position of the first match. If no entry matches, the result is 193 (not found).
- Keyed record search (Medium-High): a memory-card job requests a directory listing for the selected folder, with capacity for 16 entries in 0x40-byte records; the returned entry count is stored with that listing. The file-read job checks those entries against its 20-byte request key, comparing the first 20 bytes of each entry, and proceeds only when a match is found. In this context the key serves as the requested directory-entry name for the membership check; its exact encoding and padding are unknown. This is a runtime card-job result, not part of the save-file payload. (Open question 46 remains partly open.)

### Text conversion for name fields

Confidence: Medium-High (rules; the fallback text is not given). There are two helpers, and they return different kinds of code.

- Single byte (used for name fields). A text is converted one byte at a time. Each byte's 16-bit result is written byte-swapped, then a zero code ends the field. The single-byte rule:
  - bytes 0x20 to 0x2F, 0x3A to 0x40, 0x5B to 0x60 and 0x7B to 0x7E: the two-byte code paired with that ASCII value in the run table;
  - digits: the digit base code plus (byte minus 0x30); upper-case letters: the upper base code plus (byte minus 0x41); lower-case letters: the lower base code plus (byte minus 0x61);
  - any other byte (control codes, 0x7F, bytes 0x80 and above): the result is 0 and an error message is logged.
- Two-byte text (first byte, then second byte). The result is an ASCII value, not a glyph code.
  - First byte 0x82: a second byte from 0x4F to 0x58 gives the digit (second byte minus 0x1F); from 0x60 to 0x79 gives the upper-case letter (second byte minus 0x1F); from 0x81 to 0x9A gives the lower-case letter (second byte minus 0x20). Any other second byte gives 0.
  - First byte 0x81: the 33-entry run table is searched for the entry whose second byte matches. The result is that entry's ASCII value; no match gives 0.
  - Any other first byte gives 0 (unmapped).

### Import (optional feature)

An import reads an original card's `savegame.01` (0xC400 bytes) from the card directory `BASCUS-97367-0`. It checks the file size and then reads slots with the same layout. The saved world ids are converted with the reverse lookup. The source format of the card (raw card image or folder export) is not determined (Open question). The card result rules above (not found, unformatted, directory, query result) apply to the import as they do to a load.

## Edge cases and error handling

- **Wrong file size**: a card file whose read does not return exactly 0xC400 bytes, or whose read raises a general failure, is refused on load with record 14 (Load failed; Retry / Cancel load). Reproduce this check. Example: a file of 50,175 bytes shows record 14.
- **Load with a slot index of four or more**: nothing is copied and the world does not change (High).
- **Card not found**: the save scan shows record 4 and the load scan record 13, each with Retry and Cancel; nothing is written and nothing changes (Card results, 1).
- **Unformatted card**: the save scan offers a format (records 2, 20, 21; record 24 on failure), and after a good format the save goes on with no overwrite question. On load an unformatted card shows the "no save data" record 11, not the unused "unformatted" record 12 (High).
- **Directory missing on save**: the directory and its support files are created, then the larger transfer block is written (Card results, 3).
- **Query failure**: in the dormant transfer state machine only, the transfer ends without a message and without a retry (High). No player-visible job reads the query result.
- **Card gone in the picker**: the save and load pickers check the card every pass. A card that is removed, or swapped for another card, counts as removed and shows record 4 (save) or record 13 (load). Retry, or Cancel then No, scans again; Cancel then Yes leaves the screen with no cue and no write or load (High).
- **Write that fails**: record 6 (or record 3 for insufficient space, or record 4 when the card watch sees the card gone first). Retry never rewrites in place: every Retry and every No in the write job ends with "not saved", the screen rescans the card and shows the picker, and the player must pick the slot again. The snapshot and the pending flag were already set when the write started, so they stay (High for the flow; Medium for what the player sees of the snapshot).
- **No checksum**: a damaged slot loads without complaint as long as key A is not negative. A damaged world id can load the wrong world or none.
- **Save with a world key that has no entry**: the header key is written as 0xFFFF, stored as a 32-bit value with the high half zero. Its sign bit is clear, so the slot counts as present (see Present flag). On load, 0xFFFF matches no saved id (the largest saved id is 11070), so the reverse lookup gives 193 and the current world is kept (High).
- **Duplicate keys**: world keys 1391 and 1414 each appear twice in the key table, with different saved ids. The forward lookup returns the first match in index order: key 1391 gives id 10024, and key 1414 gives id 10045 (High for the table values; Medium-High for the first-match rule).
- **Saved id 0**: only key 678 (entry 0) has saved id 0, so the reverse lookup of id 0 has one match.
- **Not found on load**: a saved id that matches no entry gives the not-found index (193). No load is posted for that key, and the other key is still applied (Loading, steps 4 and 5). The original does not refuse the slot. A clean implementation may refuse the slot, but it must say so.
- **Empty slot choice**: Resolved. When all four slots have key A = -1, the load picker starts at cursor 4, outside the four rows; Up or Down cannot move it to a filled slot, and Cross copies nothing. This follows the load-picker rules in SPEC-0035, Behavior 7.1–7.2 and Open question 24. The empty template also has key B = -1, so no world load is posted (Loading, steps 4–5). (Open question 18; Medium-High.)
- **Picker row text follows the key only**: a slot with key A = -1 shows the "Empty" text in its row even if its name field holds other text. A filled slot whose name field is empty shows a blank first line (Medium).
- **Status timers on save**: an inactive status is saved as -1/60 (about -0.0167), because its live value is -1 and not 0 (see Status timers). Do not truncate on save. On load the original truncates the saved value toward zero to whole seconds before the hero's status-apply call, so -1/60 becomes 0 and the timer is set to 0 (not -1), except for statuses 8, 9 and 10, which stay -1. Reproduce the truncation: a status with under 1 s left is lost on load. One visible effect: a health-regeneration timer (12) restored as 0 makes a later magic regeneration (13) give +2 MP on every tick (SPEC-0036, Edge cases).
- **Current values above the maximum**: a saved current health or magic above the restored maximum is clamped to the maximum on load (Hero maxima and restore).
- **Luck (0x208) out of range**: a saved value above 24 is restored as 24, and a value below 0 as 0, on both heroes (Loading, step 12d).
- **Options**: the options block is written on save and copied back on load (Loading, step 15). A clean implementation applies the saved block when a slot is loaded and keeps the player's live settings otherwise.
- **Unmapped bytes**: keep 0x08D to 0x08F, 0x307A to 0x307B, 0x3083 and 0x30C4 to 0x30FF unchanged when rewriting a slot (66 bytes in all). A save must not zero bytes it does not understand. The engine block at 0x090 to 0x1FF (the movie played words) is copied from and to the movie object and must also be kept byte for byte.
- **Unread card or fresh card**: when the save scan did not read the file, the three slots other than the chosen one are the empty template, and the written file carries those bytes (Saving, step 5). The scan does not check its own read: on a formatted card with the save folder, a save file that cannot be read gives four empty rows, and the next save writes empty slots over the other three slots (High).
- **Petpet type with no kind**: the no-pet value -1 (0xFFFFFFFF) removes any petpet at scene start, and a saved type with no kind spawns no petpet on load (SPEC-0047).
- **Body-outfit tails**: the 16-byte tails at 0x520 and 0x840 hold the kind-5 (body outfit) item pairs of hero 0 and hero 1, not equipment Mote words. Do not write Mote words there.
- **Cloak flags (0x3080, 0x3081, 0x3082)**: on load these flags have no effect unless 0x3080 is set. A slot with 0x3081 set and 0x3080 clear loads the tails without adding item 126 and without the swap. Item 126 itself is never stored in a list (kind 9 is not saved); only the flags carry it.
- **-4 in the card jobs.** The value -4 is handled differently in each job because it is the card library's per-call result: for the folder create it means the folder already exists (event 0x13), for the delete it means not found (treated as already absent), and for the change to the save folder before a write it means path not found (event 0x11). None of the three is inverted (Shared rules; Medium-High).
- **Free-space shortfall on create.** A create when the card has fewer free blocks than the save needs raises event 0x15 and creates nothing (Create directory job, step 1, third check). A card with no card type or no format stops at the earlier checks (0x0C or 0x0E) first.
- **Short write.** A write call that returns any byte count of zero or more counts as success, even when the count is below the size asked for (Write job, step 3). The load scan's exact-size check (0xC400 bytes) is the only size test.
- **Dispatcher failure.** A step that answers busy keeps the index where it is and runs again on the next pass. A failed step ends the whole list with the job state word 4, and the dispatcher raises 0x16 when the step raised no code of its own; after the last step the state word is 3 (Save job dispatcher; High).
- **Two helpers, two kinds of code.** The single-byte rule returns a two-byte glyph code; the two-byte rule returns an ASCII value. Do not mix the two in one name field (High).
- **Unmapped name bytes.** A byte with no mapping is written as code 0. Code 0 is also the name terminator, so the name ends there (Medium).
- **Two-byte conversion failure.** A fixed fallback string is written in place of the converted text (Medium; the fallback text is not given).
- **Entry while busy.** The save or load entry returns 1 and changes nothing (High).
- **Save-point ties and overflow.** Of two save points at equal squared distance, the lower list index is chosen. A region with more than 8 save points keeps the first 8 (High).
- **Save points outside the world pack's own region.** Kind-4 records in sub-regions are never checked (none exist on disc) (High).
- **Save-point cue emitter.** For zero-flag save points (all on disc) the cue 0xA8 emitter comes from the other branch's unset index, so in practice the cue starts with no emitter position; SPEC-0031 gives the quirk and the clean-side choice (High for the rule; Low for the audible effect).
- **Restart point is the hero's, not the save point's.** Saving at a save point records where the hero stands and the way it faces; the save point's own position and facing are not used. Saving does not move the hero, so after the save screen closes the hero is still within 5 units and the prompt shows again (High).
- **No save point in an area.** 105 world packs with an embedded region have no save point. There the step finds none: no glow is placed, no cue plays and no prompt shows (Medium-High).
- **Cancel confirm after the write prompts plays the save cue.** "Cancel save?" (record 9) answered Yes inside the write job, after record 7, 10, 3, 6 or 4, ends the job with "saved", so the save screen plays cue 0xA9 and returns to play although nothing was written. The same Yes in the save scan or in the save card-gone job leaves with no cue. Reproduce it, or document the deviation (High).
- **Insufficient space only without the folder.** Insufficient space is reported only when the save folder is missing; overwriting an existing save never needs free space (High).
- **Swapped card.** A card that is not the card seen before counts as removed, in the picker check and in the card watch of a question box (High).
- **Single card slot.** Only card slot 1 is used: text variable 0 is always 1, and the game never looks at the second card slot (High).
- **Record 19 is effectively unreachable.** It needs a general failure after a check that reported no problem (High).
- **No success texts.** The game never shows a save, load or format success box (records 8, 16 and 23); a good save plays cue 0xA9 and a good load goes straight into the world load (High).
- **Picker move sound.** The picker plays its move sound on every Up or Down press, even when the cursor cannot move (SPEC-0035 section 7.1; High).
- **Busy text minimum.** A busy record stays up at least 123 passes (about 2.05 s), even when the card is quick (Card message records; Medium-High).

## Test vectors

- Slot offsets: slot 0 at 0x0000, slot 1 at 0x3100 (12,544), slot 2 at 0x6200 (25,088), slot 3 at 0x9300 (37,632). The file ends at 0xC400 (50,176).
- Slot index: index 3 is accepted (slot at 0x9300). Index 4 is refused: no copy, no world change.
- Card wait and query result: with a card request still pending, the wait helper returns 0 and the job stays in its state. Once complete, the helper returns nonzero and places the operation result in its output. For the dormant query operation only, output 0 selects failure (no retry, no message); output 1 or another nonzero value selects completion. The external meaning of nonzero remains unknown.
- Card rules, synthetic: no card gives record 4 in the save scan and record 13 in the load scan, and no write; an unformatted card gives the format offer in the save scan and record 11 in the load scan; directory present gives the 0x88-byte block (136 bytes); directory missing gives the 0x110-byte block (272 bytes) after the directory is created.
- Empty template, the floats at 0x00C and 0x01C are 1.0; the other bytes of 0x000 to 0x01F are 0. Bytes 0x024 to 0x02F: all 0xFF (three -1 keys). Bytes 0x2F6F to 0x2F74: `E m p t y` then 0x00. Byte 0x2FAB: 0x20.
- Hero 1 status timers start at 0x8A8. That is hero 0 status timers (0x870) plus one 56-byte block (0x38).
- Unlabelled option handlers (synthetic): with +0x0C = 0, Left gives 2; with +0x0C = 2, Right gives 0; with +0x0C = 1, Right gives 2. For +0x10, a held Left or Right changes 0 to 1 and 1 to 0.
- Progress flags: a slot with byte 41 set to 0x10 has flag 332 set and no other flag in that byte. Flag 332 tests as set; flag 333 tests as clear.
- Status timer on save: a status with 90 remaining ticks is written as 1.5 (90 x 1/60, float32). A status with 0 remaining ticks is written as 0. An inactive status (live value -1) is written as -1/60 (about -0.0167).
- Status restore record, synthetic: a saved value of 1.5 gives a record value of 1 second (truncated), followed by -1 and -1, so the timer becomes 60 ticks. A saved value of -1/60 gives a record value of 0.
- Status restore, synthetic: a saved poison of 35.9 s gives a poison timer of 35 x 60 = 2100 ticks. A saved -0.0167 gives 0 (inactive; the timer is 0, not -1). A saved 0.9 s gives 0 (the status is lost). A saved confusion of 20.0 s is not restored (the hero's method ignores code 6). A saved immunity to poison of 0.5 s writes nothing (the timer stays -1).
- Reverse lookup with a synthetic table: keys [700, 701], ids [0, 5]. Lookup of id 5 returns position 1. Lookup of id 9 returns 2 (not found). Lookup of id 0 returns position 0 (first match).
- Forward lookup with the same synthetic table: key 701 returns id 5. Key 702 (absent) gives the not-found result (0xFFFF).
- Forward lookup on the boot program's table: key 1391 returns id 10024 (first match, entry 166). Key 678 returns id 0 (entry 0).
- Reverse lookup of 0xFFFF on the boot program's table: 193 (not found).
- File size check: 50,176 bytes is accepted; 50,175 or 50,177 bytes is refused.
- Progress tier from scene setup: a slot whose progress bitmap has only flag 220 set (byte 27 = 0x10) has tier 2. Adding flag 248 (byte 31, bit 0 = 0x01) gives tier 3. Adding flag 332 (byte 41, bit 4 = 0x10) gives tier 4. Act-start stages 2, 3 and 4 set flags 220, 248 and 332 (SPEC-0053, section 3.7).
- Date text with synthetic fields (month 3, day 5, year 26, hour 7, minute 4): the pattern gives ` 3.05.26  7:04` (month, day, year). 20 October 2026 at 22:13 local time gives `10.20.26 22:13`.
- Item-list capacities (synthetic check): kind 0 has 20 entries, so entry 20 does not exist in that kind. The 98 entries of one container's kind blocks total 98 x 8 = 784 bytes (0x310). The next block starts 0x320 bytes after the first (0x220 to 0x540), which leaves the 16-byte tail at 0x520 to 0x52F. Kind 7 begins at 0x2C0, 0x2C0 - 0x220 = 0xA0 = 20 x 8 bytes after kind 0.
- Slot spacing: slots 0 to 3 start at 0x0000, 0x3100, 0x6200 and 0x9300. A save to slot 2 changes only its own slot, bytes 0x6200 to 0x92FF; slots 0, 1 and 3 keep their bytes. A slot index of 4 or more is outside the four-slot layout.
- Petpet fields: a save with pet type 703, remaining time 27000 and full time 27000 writes 703 at 0x214, 27000 at 0x218 and 27000 at 0x21C (little-endian). A save with no petpet writes -1 (0xFFFFFFFF), 0 and 0. A save with a live petpet whose remaining time is 0 writes -1, 0 and its full time. Nothing is written at 0x10C.
- Equipment slot word: hero 1, slot index 2 is the four bytes at 0x30B8 to 0x30BB (0x30A4 + 12 + 8).
- Container tails: the hero 1 pair of container A's tail starts at 0x528. The hero 0 pair of container B's tail starts at 0x840. They hold the body outfit entries (kind 5).
- Luck (0x208): a saved value of 30 (a damaged slot) is restored as 24 on both heroes. A saved value of 7 is restored as 7 on both heroes.
- Cloak flags: with 0x3080 = 1, item 126 is added once with count 1 on load (both cloak entries receive it). With 0x3080 = 0 and 0x3081 = 1, no item 126 is added and no swap runs.
- Current health restore, synthetic: saved maximum health 275 and saved current health 300 (0x860). The restored current health is 275.
- Current magic restore, synthetic: saved maximum magic 60 and saved current magic 90: restored as 60. A saved current health of 0 with maximum 275 is restored as 0 (no lower bound); a saved current health of 120 is restored as 120.
- Dispatcher, synthetic: a list of two step records, the first reports done. The index goes from 0 to 1. If the first step reports not done, the index stays at 0.
- Create directory, synthetic: the lookup returns -4. The job raises event 0x13 and creates nothing. The lookup returns 0 and the checks pass: the create step runs.
- Create directory, synthetic: block count 10 with free-block count 9 raises event 0x15 and creates nothing.
- Delete, synthetic: a lookup of -4 and a lookup of 0 both reach the delete call and log a diagnostic line (text not given). A lookup of -5 is not accepted.
- Write, synthetic: a write result of 0 counts as written. A write result of -4 raises event 0x11.
- Engine block round trip, synthetic: a save followed by a load returns the same 368 bytes at 0x090 to 0x1FF, whatever those bytes contain.
- Movie played words, synthetic: after movie-table index 5 has played, a save writes u32 1 at 0x0A4 (0x090 + 4 x 5) and 0 at the words of movies not yet played. After loading that slot, HasMoviePlayed(5) is true and HasMoviePlayed of an unplayed index is false.
- Gains and load: fresh boot, Audio page never opened: gains 1.0 each. Load a slot whose volumes are 75, 40 and 100: gains 0.75, 0.40 and 1.0, and the knobs move to match; the Vibration value reaches the pad gate on the next pass.
- Header transfer: the 964 bytes from header offset 0x040 to 0x403 inclusive. 0x403 minus 0x040 plus 1 is 0x3C4 (964).
- Name codes: the letter A in a name field is the code 0x8260, stored as bytes 0x82 then 0x60. The text "AB" stores 0x8260, then 0x8261, then a zero code. The letter a is 0x8281 (bytes 0x82, 0x81). The digit 0 is 0x824F (bytes 0x82, 0x4F).
- Two-byte text, first byte 0x82 and second byte 0x60: ASCII 0x41 (A). first byte 0x82 and second byte 0x81: ASCII 0x61 (a). First byte 0x83: 0 (unmapped).
- Single byte 0x0A (line feed): code 0 and an error is logged. The name "AB" followed by that byte stores two codes and then a zero code, so the name ends after B.
- Entry, synthetic: state 0 with a nonzero flag returns 0 and moves to state 1. State 2 with an entry returns 1 and does not change the state. An entry whose card operation cannot start returns 2, sets state 4 and status 0. A zero flag that completes inside the call returns 3.
- Name lengths, synthetic: the size word at 0x34 is 10 and the words at 0x38 and 0x3C are zero. Name fields 2, 3 and 4 then each take 14 bytes.
- Save success: after the write job reports done and the message box is idle, a card state word of 0 plays cue 0xA9 (index 169, save complete) and the save screen returns code 0x0B4. No pass count is involved.
- Save failure: the same point with a card state word of 2 sets the stage to 0; the next pass starts a new card scan, and the picker shows again. The pending flag stays set.
- Save cancel: Triangle in the slot picker writes nothing, plays front-end menu sound 6, clears the music overrides and returns front-end code 0x0B4 (play).
- End of a frame-loop pass with the apply flag set to 1: the hero's restart point becomes (vector A, vector B) (slot bytes 0x000 to 0x00F and 0x010 to 0x01F) and the hero is placed there once. Afterwards the apply flag is 0 and both stored vectors read (0, 0, 0, 1). With the apply flag 0, the restart point is not changed and no placement is made.
- Camera at placement (High): restart position (0, 0, 0), facing (0, 0, 1) and the up vector W = (0, 1, 0) put the camera at (0, 2.5, −10). Restart position (10, 0, 5), facing (0.6, 0, 0.8): camera at (4, 2.5, −3).
- Save points (synthetic; all flags 0, normal play means game-state code 0 and no in-play menu sub-state). Distances are given first, squared distances in brackets; the tests compare squared distances.
  - Three save points at distances 20, 4 and 4 (squared 400, 16 and 16), indices 0, 1 and 2: the zero-flag choice is index 1 (the tie with index 2 goes to the lower index), and the glow moves there. 16 < 225, so cue 0xA8 plays; 16 < 25, so the "Save Game" prompt shows. A Square press edge with the hero idle sets the restart point to the hero's position and facing and opens the save screen with stage 0 and cursor 0.
  - One save point at distance 10 (squared 100): cue 0xA8 plays (100 < 225); no prompt (100 is not below 25); the away flag is set to 1.
  - One save point at distance 15 exactly (squared 225): no cue and no prompt (both tests are strict); the glow still sits there; the away flag is set.
  - One save point at distance 5 exactly (squared 25): cue 0xA8 plays; no prompt (25 is not below 25); the away flag is set.
  - Distance 3 (squared 9), Square pressed while the hero is attacking (busy): cue plays, prompt shows, the save screen does not open and the restart point is unchanged.
  - Distance 3 (squared 9), Square pressed while an interior enter or leave sequence runs (game-state code 6): cue plays, no prompt, the screen does not open, and the away flag is not touched.
  - Save point and a talk candidate, both 3 units from the hero, hero idle, Square press edge: the save screen opens and the talk callback starts in the same tick; the caption drawn on that pass is "Talk".
  - Save point and a book station both in range, hero idle: "Save Game" is shown, "Read Book" is not; a Square press opens the save screen only, and the pending book stays set.
  - A save point with flag 1 at distance 3 and Square pressed with the hero idle: the save screen opens, but the restart point is not changed.
  - Square held down for several ticks within 5 units: only the tick with the press edge opens the screen.
- Load world, on the boot program's key table: a slot with key A = 10024 and key B = 0xFFFF. The reverse lookup of 10024 gives entry 166, so a load of key 1391 is posted into the transition buffer. The reverse lookup of 0xFFFF gives 193 (not found), so no load is posted into the exterior buffer.

Card jobs (synthetic; Circle answers 1, Triangle answers 0):

1. Save to an empty slot on a card that holds a save: Cross on slot 2 (key A -1) shows record 10. Circle (Yes): record 5 for at least 123 passes, the write succeeds, cue 0xA9 plays and the screen returns to play.
2. The same with slot 0 filled: Cross on slot 0 shows record 7 (overwrite). Triangle (No): nothing is written; the next pass rescans and the picker shows with the cursor where the player left it. The pending flag is set (it was set at Cross).
3. A fresh unformatted card, save: record 2; Circle; record 20; Circle; record 21 for at least 123 passes; the format succeeds; the picker shows four "Empty" rows. Cross on slot 0: no question, record 5, the folder and icon files are made, the file is written, cue 0xA9.
4. Card removed while the save picker is up: on the next pass record 4. Triangle (Cancel save): record 9. Triangle (No): rescan, and record 4 again while the card is out. Circle on record 9 (Yes): return to play, no cue.
5. Write fails because the card is pulled during record 5: record 6 (or record 4 if the card watch fires first). Triangle (Cancel save), then Circle on record 9 (Yes): cue 0xA9 plays and the screen returns to play, with nothing written (quirk).
6. Boot with no card: record 1; Triangle (Continue): the start countdown follows (front-end code 0x258). Boot with a formatted card that has less than the needed space and no save folder: record 0. Boot with an unformatted card: no message.
7. Load with an unformatted card: record 15 for at least 123 passes, then record 11. Triangle (Cancel load), then Circle on record 17 (Yes): the load screen is left with no load.
8. Load with a file of 50,175 bytes: record 14.
9. Box answer: a record labelled Retry / Continue. Circle gives 1 (Retry), Triangle gives 0 (Continue); both in the same pass gives 1.
10. Event set: raise 21, then test 21 gives 1 and test 22 gives 0. A failed request with an empty set ends with code 22 set. Raising -1 or 24 changes nothing.

Save and load round trips (a save then a load gives the same state; synthetic values at real table positions):

1. World indexes, from the boot program's key table: key 678 is 0, 691 is 5, 909 is 68, 1174 is 127, 1391 is 166, 1414 is 179 and 1433 is 192. T1 rows start at 0x900 + 17n: 909 at 0xD84, 1391 at 0x1406. T2 rows start at 0x15D1 + 33n: 909 at 0x1E95, 1391 at 0x2B37.
2. Kill memory: in world 909, the character records with kill bits 0 and 9 are killed. The save writes T2 bytes 0x1E95 = 0x01 and 0x1E96 = 0x02. After a load and a re-entry into 909, those two records stay unbound and every other record binds. Unstored kill (Medium-High until confirmed at run time): kill the record with kill bit 3 in transition 691 (world index 5), then stream a different transition into the transition buffer from the exterior, then save: 691's row (0x15D1 + 33 × 5 = 0x1676) keeps bit 3 clear (byte 0x1676 bit 3, mask 0x08), and record 3 binds again when 691 is next loaded (SPEC-0006).
3. Broken breakable: in world 678, the breakable with script index 5 is broken. The save writes byte 0x900 = 0x20. After a load, world 678's setup applies that entry's start mode: mode 1 gives state 0, mode 2 gives state 1 with no drop, mode 0 gives state 1. An entry with script index 130 is not recorded.
4. Once-per-area: a Golden Negg bought in world 909 (index 68). The save writes byte 0x2EE4 + 8 = 0x2EEC = 0x10 (bit 4). After a load, a second Golden Negg in 909 is refused, and one in 1174 (index 127: byte 0x2EE4 + 15, bit 7) is accepted.
5. Treasures: maps 3 and 13 held, treasure 13 dug (collected row 213). The save writes 0x2F4F = 0x08, 0x2F50 = 0x20 and 0x2F2F + 26 = 0x2F49 = 0x20; every other byte of both sets is 0. After a load, maps 3 and 13 are held, row 213 is 1, and the dig sites with ids 203 and 213, where resident, are both switched on by their held bits.
6. Movie words: movies 16 and 88 played. The save writes u32 1 at 0x0D0 and at 0x1F0. After a load, HasMoviePlayed(16) is true and the movie queue is empty.
7. Status: a poison with 2100 ticks left is written as 35.0 and restored as 2100 ticks; 2154 ticks is written as 35.9 and restored as 2100; 54 ticks is restored as 0; an inactive status is written as -1/60 and restored as 0, not -1; a confusion is never restored.
8. Maxima: Tor 225 and 135 with Roberta 180 and 225 all round-trip. Roberta words 0 with Tor 225 and 135 give Roberta 120 + 20 x 3 = 180 and 150 + 25 x 3 = 225. A Tor health word of 525 (above the cap) gives 500.
9. Luck 7 is written as 7 and restored as 7 on both heroes. Luck 30 (a damaged slot) is restored as 24 on both heroes.
10. Equipment: hero 0 words (-1, -1, 0x54). Hero 0's slot 2 word is at 0x30AC. After a load the word is 0x54. If it differed before the load, equipment-panel timer 2 is 240. No cue plays.
11. Cloak: hero 1 wears the cloak over outfit 130. The save writes 0x3080 = 1, 0x3081 = 0, 0x3082 = 1, and A tail hero 1 (0x528) = (130, 1). On load, outfit 130 is added, item 126 is added once, then hero 1 swaps: hero 1's body entry is 126 and its cloak entry is 130. With the cloak owned and not worn, 0x3080 = 1 and 0x3081 = 0x3082 = 0; after a load both cloak entries hold 126 and each hero wears its own outfit.
12. Stash: during Act 2, with the party wallet 40 and the stash wallet 500, the save writes 0x200 = 40 and 0x204 = 500. After a load and then the Act 3 start, the wallet is 540.
13. Swap mode 1: 0x307C = 1. After a load, a switch to Tor is refused (SPEC-0053).
14. Keys: T = 691 and E = 909 give 0x024 = 11 and 0x028 = 602. After a load, 691 is in T and 909 is in E, with E current. Key C (an interior id) is ignored.
15. Empty pair: a kind-0 entry with no item and count 0, over an old slot id 50, leaves 0x220 = 50 and writes 0x224 = 0. After a load, nothing is added.
16. Count table: item 40 has count 0. The hero touches one: the announcement shows the first-find panel and the count becomes 1. A second item 40 is given while that panel is still up: the announcement is skipped and the count stays 1. After the panel closes a third one is picked up: the count becomes 2. An item whose count is already 255 stays at 255.
17. Load placement: a slot with key B for a world whose start-point list holds entry 0 at (10, 0, 5) and vector A at (40, 2, -8). In the world-change step the hero is first set at (10, 0, 5), then at (40, 2, -8) with facing B in the same step. The first drawn frame shows the hero at (40, 2, -8).
18. Create checks: device information with card type 1, formatted 1, free blocks 50, and a save record that needs 60: event 0x15, nothing created, job state word 4. With free blocks 60 and the folder-present word 1: event 0x13. With free blocks 60, the folder-present word 0 and a create result of -4: event 0x13. With type 0 the first check stops at 0x0C, even when the card would also lack space.
19. Write job: the folder change returns -4: event 0x11, state word 4, no 0x16. The open returns -2: no event from the job, so the dispatcher raises 0x16. The write of 0xC400 bytes returns 0x8000: success, the step result is 0x8000.
20. Delete job: the delete returns -4: done, as if deleted. It returns -2: state word 4 with event 0x16.
21. Paths: a write of the save file makes "/BASCUS-97367-0" the current folder and then opens the file by its bare name; there is no "/BASCUS-97367-0/" plus name string.
22. Header: bytes 0x040 to 0x043 of the header are 0x50, 0x53, 0x32, 0x44 ("PS2D"); those bytes are the first four of the folder's `icon.sys`, which is 964 bytes long.
22a. Header source text: bytes 0x41C to 0x436 hold the 27 characters of `Neopets: The Darkest Faerie` and bytes 0x437 to 0x43A are zero. The title at 0x100 is 27 two-byte glyph codes followed by a two-byte zero (56 bytes, 0x100 to 0x137). Bytes 0x408 to 0x415 hold `BASCUS-97367-0` and 0x416 to 0x41B are zero.
23. Dispatcher: a list of create folder then write, where the create step fails with 0x15: the write record never runs, the job state word is 4, and only code 0x15 is set.
24. Save scan file-read sequence: a completed file-open result of -2 raises event 0x10, -4 raises 0x11, and another negative result raises 0x0C. A completed read result of -4 raises 0x10. On close, -3 raises 0x12 and another negative result raises 0x16.
25. Starting equipment (disc ids): a save written right after a new game holds, in container A's tail, hero 0's pair (id 119, count 1) and a count of 0 for hero 1, and the general list holds 150 x3 and 149 x3. Loading it adds 119 through the normal add: Tor's body entry holds 119 and the outfit is 0. A save written right after the Act 2 start holds, in container A's tail, a count of 0 for hero 0 and hero 1's pair (127, 1); container B's tail holds hero 0's pair (119, 1) if Tor had not replaced his outfit.

## Open questions

Numbers of removed questions are not reused.

1. **Status restore on load**. Resolved: each saved value, truncated toward zero, goes to the hero's status-apply method as whole seconds (Status timers; SPEC-0036, Status apply; High).
2. **Fourth slot label**: Resolved: there are four usable save slots and the fourth row is slot index 3, built from its own slot's text; "Game 1" to "Game 3" are a leftover of a front-end page that nothing opens, and no reserved slot is known (High; SPEC-0035 Open question 45).
3. **Lists A and B and the categories**: Resolved: container A is the party inventory and container B the stash; the value is the entry's count, and the kinds are as in Item lists (High, Medium-High for the act link).
5. **Per-world 256-bit state**: Resolved: bytes 0 to 31 of each T2 record are the 256 kill bits of one world (SPEC-0006, Kill memory), not a player-state bitmap (Per-world tables; High).
6. **Unlabelled fields**: Resolved: every field has an owner and a meaning in the field map (High).
8. **Duplicate and zero ids**: Resolved (High): the forward lookup scans the 193 key entries from index 0 upward and returns the saved id of the first entry whose key equals the sign-extended 16-bit input; it never looks past a match. Only key 678 has saved id 0, so the reverse lookup of id 0 has one match. Test vector (synthetic): if keys at indexes 5 and 90 were both K, the lookup of K returns the id stored at index 5; a key found nowhere returns 0xFFFF.
9. **Options: volume groups**: +0x14 is Music, +0x18 is Sound effects and +0x1C is Voice. Each is a float from 0 to 100, scaled by 0.01 and clamped to 0 to 1 for its gain. Each is shown on its slider knob, with UI element ids 0xCE, 0xCF and 0xD0 (High for the knob ids and the apply step; Medium-High for the labels). They do not send sound cues. A new game does not write the options block (see New game and reset). The live block is filled with the defaults at start-up (SPEC-0026).
10. **Audio apply after load**: Resolved: the apply sets only the three gains, the three knob positions and the first pad's inversion flags; +0x0C and +0x10 are copied but not applied (Options block; SPEC-0035; High).
12. **Save screen controls**: Answered: the slot picker uses the logical input mapping in SPEC-0035, section 4: 0x1000 Up, 0x4000 Down, 0x40 Cross confirms and 0x10 Triangle backs out. The card question boxes use bit 0x20 Circle (answer 1, cue 0x98) and bit 0x10 Triangle (answer 0, cue 0x99), as recorded above (High).
13. **Directory suffix**: Resolved: the product code and the `-0` string join with nothing between them, giving `BASCUS-97367-0` (High).
14. **Import source format**: a choice for the rewrite's import feature, not a game fact (unknowable from the disc). Default (rewrite choice): accept an extracted folder.
15. **Card icon and metadata**: the contents of `icon1.ico` and `icon.sys` are platform card metadata that the game does not define (unknowable); the import feature does not need them.
16. **Date field order**: Resolved: month, day, year, then hour and minute, in local time converted from the console clock (Location, date and progress; Medium-High).
17. **Unmapped bytes**: the exact unmapped bytes are 0x08D to 0x08F, 0x307A to 0x307B, 0x3083 and 0x30C4 to 0x30FF (66 bytes). The rest of the slot is mapped in the field map. No reader or writer touches them in this build; default (rewrite choice): copy them unchanged on load and save (unknowable beyond that).
18. **Empty slot choice**: Resolved: when all four slots are empty, the load cursor is 4, outside the row range, and Cross copies nothing (Edge cases; SPEC-0035, Behavior 7.1–7.2 and Open question 24; Medium-High).
19. **Scene modes 2, 3 and 4**: which callers pass each mode (SPEC-0053). Answered: they are the act-start setups of Acts 2, 3 and 4 (SPEC-0053 section 3.7).
20. **Progress tier meaning**: any use of the tier other than the save display field (none is known). Answered: the tier is the act number (Progress tier; High).
21. **Unformatted-card handling**: Resolved: the reachable save scan offers a format and the load scan shows record 11; the two other unformatted-card paths are dormant (Card jobs; High).
22. **Card-operation state words**: the stage, request, active and clear-only words (zero at boot) and the mode word (1 at boot). Resolved: nothing in this build sets the request words, so the card paths that use them are dormant (High).
23. **Card operation direction and mode meaning**: Resolved in part: the state-machine mode value 1 selects deferred handling. The operation start returns while work is pending, and later state-machine passes poll for completion; mode 0 waits and advances the operation to completion before returning. This is a wait policy, not the transfer direction: callers use operation records for both reads and writes. A nonzero mode also records event 0x0B during validation of operation kind 8. The underlying card-library mode mapping and the direction selector remain open.
24. **Save file transfer structure**: Answered in part: the card staging object points to the complete 0xC400-byte four-slot array and gives that full size to the write job. The boot-side job makes one data request to the card service per write operation, then polls for completion. The directory setup selects a 0x110-byte block when creating the directory and a 0x88-byte block when it already exists (see open question 38). Whether the external card service splits that request into multiple physical card transfers remains open.
25. **Trigger of the card load step**: it has no known direct caller. Answered: nothing calls the second save routine, so it is dormant in this build (High); it writes a slot with the same card rules, writes the whole array after refreshing the chosen slot, and passes no staging buffer (Medium).
26. **Overwrite prompt**: Resolved: yes; the save write job asks record 7 when the chosen slot held a game, record 10 when it was empty, and nothing on a card formatted in the same scan (Saving, step 4; High).
27. **Where card messages appear**: Resolved: every card job and the records it shows are under Card message records and Card jobs as the player sees them; there is no separate save-error screen (High).
29. **Diagnostic log destination**: Resolved (High): the log routine formats the text, collects it in a line buffer of 126 characters (the buffer restarts when full), and on each newline hands the line to the development-kit debug console channel of the system. Nothing is drawn on screen and nothing is written to the card, so no log line reaches the player. A rewrite may drop these lines.
30. **Petpet fields**: Resolved: the type is at 0x214, the remaining time T at 0x218 and the full time F at 0x21C; with no petpet, or T below 1, the type is written as -1 (0xFFFFFFFF) (Slot field map; High). No save or pet routine reads or writes 0x10C.
31. **Equipment slot ids**: Resolved: the three words per hero at 0x30A4 are the armour, shield and weapon Mote words, and the tails at 0x520 and 0x840 are body outfit entries; do not write Mote words to the tails (Equipment Mote words; Item lists; High).
32. **Record loop**: the save writer rebuilds only the chosen slot. The four-slot loop at 0x3100 steps is a reset to the empty template, run before each card scan. Saving steps 3 and 5 state the correct behaviour. Resolved (proactive (open close-out)): the other three slots are the bytes the pre-picker save scan read from the card file, or the empty template where the file was not read (Saving, step 5 and Edge cases; High).
33. **Hero order of the item containers**: Resolved: neither container belongs to one hero; only the tails and the kind 1, 2 and 5 entries are per hero, through the item's owner field (Item lists; High).
34. **Shared 368-byte block at 0x090 to 0x1FF**: Resolved: the 92 movie "played" words, one u32 per movie-table index (Engine block; High).
35. **Item container counters**: Resolved: 0x200 is the party Neopoints wallet and 0x204 the stash wallet (Item lists; High).
36. **0x2FBB count table**: Resolved: one byte per item id 0 to 190, raised by one by each item-obtained announcement, capped at 255 (Count table, luck and swap mode; Open question 88; Medium-High).
37. **Array fill during the scans**: Resolved: both scans reset the slots and fill the array with a read-file request; only the load scan checks the read (Card jobs; Medium-High).
38. **Card write block choice** (Medium): Answered in part: directory creation uses the 0x110-byte block; an existing directory uses the 0x88-byte block. The boot-side sequencer issues one data request for each file-write operation and then polls until completion. The external card service and its physical packet sequence are not on the disc's boot program (unknowable from our data; the rewrite needs only the two block sizes and the one-request-then-poll pattern; see open question 24).
39. **Overwrite flag**: Resolved: it is the chosen slot's present flag (key A 0 or more before the rebuild), which picks record 7 or record 10 (High).
40. **Item 126 grant and the cloak flags**: Resolved: item 126 is the Tattered Cloak; 0x3080 marks it owned and 0x3081 or 0x3082 worn by hero 0 or hero 1 (Item lists, Tattered Cloak; High).
41. **Card query result meaning**: Resolved in part: the wait helper's readiness result is separate from the operation result; helper result 0 means pending, while a completed query's operation result 0 selects failure and nonzero selects completion (Card results, item 4; Shared rules, Waiting). The external card-service meaning of nonzero, including whether it denotes a found entry or a service status, remains unknown. **[High for the value distinction and game branch; Medium for the service contract.]**
42. **Negative operation results**: Resolved for the file-read machine (see Card operation state machine, which gives the per-state mapping; High). In its path or file lookup poll -2 maps to event 0x10, -4 to 0x11 and other negative results to 0x0C. In its open poll -4 maps to 0x10 and other negative results write no code. In its read poll -3 maps to 0x12 (and 0x16 is raised as well) and other negative results map to 0x16. The close poll writes no code. These are job- and stage-specific branches; the external card-library meanings of -1 to -9 are defined by the console's card library, not by the game (unknowable from our data; the rewrite uses only the branches above and treats any other negative result of a job as that job's generic failure, a rewrite choice) (Card jobs; Medium-High for the branches).
43. **Planner codes**: Resolved: the planner is the request validator, and its codes 0 to 11 are request-building errors with no player-visible effect (Request validator; Medium-High).
44. **Callers**: Answered: the planner is the request validator, run on each request before the card layer (Open question 48), and the card operation state machine belongs to a dormant card path (Card operation state machine). Nothing player-visible remains open.
45. **Status code meanings**: Resolved: 0x0C no card, 0x0D not the same card, 0x10 no file found, 0x11 path not found, 0x12 file corrupted, 0x16 general failure (Event codes; High).
46. **Keyed record search**: Resolved in part: the 0x40-byte records are entries returned by a memory-card directory listing and the file-read job searches them using its 20-byte requested-entry key (Keyed record search; Medium-High). The key's exact encoding/padding and the appropriate owning spec remain open.
47. **Job event codes (0x0C, 0x0E, 0x11, 0x13, 0x15).** Resolved: they are bit numbers of the one 24-code event set (Event codes; High).
48. **Dispatcher kinds.** Resolved: 0 card query, 1 create folder, 2 read file, 3 and 4 write file, 5 delete, 6 not implemented, 7 format, 8 new-card check (Save job dispatcher; Medium-High).
49. **Meaning of -4.** Resolved: -4 is the card library's per-call result, so its meaning depends on the call and the three handlings are not inverted (Shared rules; Medium).
50. **Dispatcher end and failure.** Resolved: busy keeps the index; done advances it and the state word becomes 3 after the last step; a failure stops the list with state word 4 and raises 0x16 when no code was raised (Save job dispatcher; High).
51. **Create checks.** Resolved: no card (device information +0x10 is 0) gives 0x0C, unformatted (+0x14 is 0) gives 0x0E, free blocks (+0x18) below the save record's need (+0x840) give 0x15, and the save record's folder-present word (+0x404) of 1 or more gives 0x13, tested in that order (Create directory job; High).
52. **Create job and the directory branch.** Resolved: the create-directory job is that step; after the folder it writes `icon.sys` (the 964 header bytes) and the icon file (Create directory job; Medium-High).
53. **Job tests and the transfer poll rule.** Resolved: the jobs use one shared wait step (still running, finished, or no call pending, which fails the job) and then test the call's own result (Shared rules; High).
54. **Write open value.** Resolved: 0x203 is the card library's create flag (0x200) plus read-write (3) (Write job; High).
55. **Path joining.** Resolved: there is no joining; each job makes "/" plus the 20-byte folder name the current folder, then uses the bare file name (Shared rules; High).
56. **Other delete results.** Resolved: any result other than zero or above or -4, at the change of folder or at the delete, fails the job with no event of its own, so the dispatcher raises 0x16 (Delete job; High).
57. **Present-flag polarity.** Whether any save path writes a negative key A other than -1 (Medium). Answered: the writer's present flag is the sign bit of key A (0x024); the picker rows and the starting-cursor search treat only -1 as empty, while Loading treats any negative value as empty (High for the picker rule). Default (rewrite choice): treat any negative key A as empty and write -1 for an empty slot; a differing negative value cannot arise from the game's own saves.
58. **Writer capacity groups.** Resolved: per container the writer fills kind 0 (20), kind 7 (60), kind 8 (14), kind 1 (2), the tail (2) and kind 2 (2), the load order, 100 entries filling the 0x320 bytes (Item lists; High).
59. **Glyph use.** Which font or glyph set draws the two-byte codes, and whether the game reads the card name fields back; the import feature may need this. Answered: the glyph tables and conversion rules are in Name text and glyph codes. (Medium)
60. **Header tag.** Resolved: the tag is the fixed signature of the console's save-description file; header bytes 0x040 to 0x403 are written as `icon.sys`. The load does not read it (Card save header; High for the file, Medium for the load).
61. **Count of 100.** Whether the count of 100 stored with the 20-byte block in the count variant of the transfer sub-machine is a retry limit, a timeout (in units not known), or a number of blocks. This is console card-protocol detail with no encoded meaning (unknowable); the clean side does not need it. (Low)
62. **Header extent and the source text.** Resolved: the 964 bytes from 0x040 to 0x403 are the `icon.sys` file, written in one write job; the text at 0x41C is the source of the title at 0x100, which the console's browser shows (Card save header; Medium-High). The game puts the fixed game title `Neopets: The Darkest Faerie` at 0x41C, and the folder name `BASCUS-97367-0` at 0x408, both once when the card header is set up; no other value is ever stored there (High).
63. **Which helper the display uses.** Answered for the observed paths: the picker reads the saved location and date strings directly, and its row drawer renders the stored location glyph codes (SPEC-0035, section 7.1; SPEC-0022, Fonts). The card-browser title is written once from its fixed ASCII source through the single-byte name-field conversion; the game does not read that `icon.sys` title on load (Card save header, Open question 60). The two-byte text helper is not shown on either path. No mixed conversion is established. (Medium-High for the path boundaries; font identity is specified in SPEC-0022.)
64. Resolved: **Current values on load**: the current health and magic are saved at 0x860 and 0x868 (hero 0 then hero 1), and restored after the maxima, clamped from above only (a value above the maximum becomes the maximum; no lower bound) (High).
65. **Save prompt opener.** Resolved: the save screen (front-end record 18) opens only from a Square press near a save point (Behavior, Save points; SPEC-0035 section 7.3). High.
66. **Banded maxima.** Resolved: an own word from 1 to its cap is the maximum, so the maxima round-trip; the banded form is only Roberta's fallback, built from Tor's start and gain words (Hero maxima and restore; High).
67. **Start-key globals.** Resolved: every store writes -1 and the data value is -1, so every start that reads them uses the defaults 691 and 909 (New game and reset; High).
68. **New game cue calls.** Resolved: the target returns at once and is never reached through a pointer, so these calls play nothing (New game and reset; High).
69. **In-game clock and play time.** Resolved (High): the slot holds no clock and no play time. The calendar and the two play-time totals of SPEC-0007 are never saved and never restored. The save writer reads none of them, every byte of the slot that it writes is in the field map, and the 66 unmapped bytes are neither written nor read (question 17), so no second save path can carry a counter. The slot picker shows no play time for the same reason. Test vector: play 30 minutes, save, quit to the title and load: the calendar and totals continue from their values at the quit, not from the save.
70. Answered: the area-manager word (0x307C) is the hero swap mode; see SPEC-0053 (save and restore under Count table, luck and swap mode).
71. **Negative status timers.** Resolved (High): a live status timer is only ever -1 (after a reset or a clear) or 0 or more. The per-tick count-down touches a timer only while it is 1 or more, so it stops at 0, and the status apply sets a timer only when the new tick count is 0 or more (a negative duration other than the default -1 changes nothing; SPEC-0036, Status apply). So the only negative value in a save is -1/60 s from an inactive status. Test vector: a timer of 3 ticks reads 2, 1, 0 and then stays 0; its saved value is 0 s, while a cleared timer saves about -0.0167 s.
72. **Pending increment.** Resolved: +0x20C is the layout word of the front-end screen object; the routine adds the height of the text lines it draws. It is not game state (New game and reset; Medium-High).
73. **Options fields +0x08, +0x0C and +0x10.** +0x08 is Vibration (0 or 1, default 1). It gates rumble through the pad input and is not an audio setting (High for the store; Medium for the gate). For +0x0C, a menu handler decrements on Left (wrapping 0 to 2) and increments on Right modulo 3; for +0x10, a menu handler toggles the value while Left or Right is held. These state transitions are established. Resolved (High): neither field can be reached from any shipped screen. The front end's two dispatch tables (node predicates and screen states, SPEC-0035) have a getter and a handler for each of the three toggles and a handler for each of the three volumes, but no slot for the +0x0C cycler handler, the +0x10 toggle handler, or their getters, and no other table, call or address constant in the program names them. A UI node can only name a table slot, so no disc data selects them, and the level scripts import no option function. No code reads either value for any purpose. Their meaning is not encoded anywhere (unknowable). Default if unresolved (rewrite choice): keep both words at their defaults (1 and 0), copy them through load and save unchanged, show no row for them and never read them. (High for the transitions and for the reachability; Low for meaning.)
74. **Hero value at 0x208.** Resolved: it is luck, always equal for both heroes; the save stores Tor's value, and the load sets each hero to 0 and then adds it with the clamp (Count table, luck and swap mode; SPEC-0021; High).
75. **Save offsets of the treasure bitsets**: Resolved: the collected-treasure set is at 0x2F2F and the held-map set at 0x2F4F, 32 bytes each, LSB-first (Treasure sets; High).
76. **Default options in an empty slot.** Resolved (proactive (open close-out)): the empty-slot template is right: an empty slot's options block is zero. At start-up the default-writing routine is run for each slot, but the empty-slot fill then zeroes the whole 0x3100-byte slot, so the defaults do not survive (High). Only the live block holds the defaults.
77. **Save-point object.** Resolved: a save point is a kind-4 record of the world pack's own character region (SPEC-0006), handled by the region's per-tick step, with no level host call (Behavior, Save points). High.
78. **Format flow.** Resolved: only the save scan formats (records 2, 20, 21, and 24 on failure); the load scan never does (Card jobs, Save scan; High).
81. **World entered after a load.** Resolved: the exterior (key B) is current after a load, and key C (the interior) is written but never read (World keys and the world after a load; Medium-High).
82. **Away flag.** Resolved: write-only. No code reads the word, and its getter has no caller and no stored pointer (Save points; High).
83. Resolved: **World update while the save screen is open.** The world is frozen (game-state code 9 from the pass after the Square press; SPEC-0026), the card job advances as a mask-3 global task, and a second Square press cannot reopen the screen (High).
84. **Square conflict.** Resolved: one Square press both opens the save screen and acts in the hero's chooser in the same tick (Per-tick step, Order against the hero's Square handling; High).
85. **Camera basis vector.** Resolved: W is the constant up vector (0, 1, 0, 0) at data address 0x3932E0, set once at boot (Restart point and the end-of-pass apply; High).
86. **T2 byte 32.** The last byte of each 33-byte kill memory row is copied both ways, but no reader of it exists in the program; its purpose is not encoded (unknowable). Keep it unchanged. (Low)
87. Resolved: **Interior save points.** None of the 60 save-point packs is an interior: 50 are exteriors and 10 are transitions (678, 686, 691, 699, 706, 851, 861, 1079, 1395, 1396). The interior buffer is empty whenever a save point is used, so key C is never needed to restore the hero's world, and not reading it changes nothing in play. (High)
88. **Count-table increment.** Resolved: the item-obtained announcement adds 1 for touch pickups and for both give paths; it is skipped while the announcement shows its first-find panel (Count table, luck and swap mode; Medium-High).
89. **Plane words of the posted loads.** Resolved: the first plane word is the entry-point index; with -1 the entered world places the hero at its first start point, and the apply flag moves the hero to vectors A and B in the same step, before any frame is drawn (World keys and the world after a load; High).

Settled here: the progress-flag bitmap is saved at 0x030 and restored on load; the options block is restored on load; the status-timer scale is one tick (1/60 s), not a stat multiplier; the status restore applies whole seconds, truncated, through the hero's status-apply method (Open question 1); the slot count is four, with no known reserved slot, and the fourth row needs no label (Open question 2); the save path starts only from the save screen (no autosave and no automatic checkpoint save), and the save screen is opened at save points (open question 65); vectors A and B are the hero's position and facing (Restart point and the end-of-pass apply); a slot index of 4 or more is refused on load; an unknown world key on load posts no world load for that key and does not refuse the slot; the petpet fields are at 0x214, 0x218 and 0x21C (open question 30); the save writer rebuilds one slot, and the four-slot loop is a reset (open question 32); the value at 0x2FBA is the progress tier, not a slot count (High); the per-world kill memory (T2) and broken-breakable sets (T1) are saved (open question 5); the 191-byte count table at 0x2FBB (open question 36, size only); the current health and magic are saved and restored (open question 64); the save write starts at once and the save and load screens wait for the card job with no fixed pass count (Saving, step 6); the fourth picker row is slot index 3 and the "Game 1" to "Game 3" labels are a leftover (open question 2); the card message table is design table 9 from resource 533 (Where it lives on the disc); the camera basis vector W at placement is the up vector (0, 1, 0, 0) (open question 85); every byte of the slot has an owner, with 66 bytes untouched (open question 6); the treasure sets are at 0x2F2F and 0x2F4F (open question 75); the five 25-byte records are the once-per-area sets (open question 6); containers A and B are the party inventory and the stash, and 0x200 and 0x204 their wallets (open questions 3, 33 and 35); 0x3080 to 0x3082 are the Tattered Cloak flags (open question 40); the maxima round-trip (open question 66); 0x208 is luck (open question 74); 0x307C is the swap mode (open question 70); key B is the current world after a load and key C is never read (open question 81); the date text is month, day, year (open question 16); the progress tier is the act number (open question 20); the card messages, the overwrite prompt, the format flow and the quit confirm (open questions 21, 26, 27, 39 and 78); the card event set and the dispatcher kinds (open questions 43 to 45 and 47 to 49); the folder name `BASCUS-97367-0` (open question 13).

