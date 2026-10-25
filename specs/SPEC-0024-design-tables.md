# SPEC-0024: Game design tables (resource 533)

- Answers: proactive (none requested)
- Status: Draft
- Disc build: SCUS-97367 v1.00. Fingerprint to check: resource 533 of the game data archive is 112,640 bytes (55 sectors), its first chunk is a type-42 IDM chunk with format byte 0xCD, and it holds exactly 20 top-level chunks (17 of type 42, then 3 of type 43).
- Confidence: per section. Species and villager tables: header layout High (counts read from disc); load check Medium; row fields belong to the enemy specs. Container, table counts and record sizes: High. Pointer relocation and string rules: High. Field meanings: per table, marked in the tables below. Emote tables 7 and 8: record counts, row layout, index ranges, chance and variance bytes: High (from disc). Emote clip lookup (character stride 74, hero shift 183, uniform random step): High; emote id as animation id: High; emote trigger (dialogue control codes): High (SPEC-0022). Villager row voice set (+0x14) and its census: High. Table 2 progress-flag cues (rising edge, stinger, journal icon, stinger 4 cooldown): High. Table 13 labels: High for content and for the type-40 index field; Medium-High for use. Table 14 stinger identity for indices 0 to 18, and the dropped indices 19 and above: High. Table 4 field roles: High. Tables 11 and 12 (stage flag set and start-point item grant): High. Table 9 colour: Medium-High. Table 15 styles and scroll: High. Table 16 fields, index map and status request words: High; which objects send the requests: Medium-High. Type 43 layout and totals: High; unit use: Medium. Book records (resource 576) layout and values: High; book request (one-slot cache): High; layout chunk field roles: High (font descriptor format of SPEC-0022); text chunk field roles: Medium.

## Summary

Resource 533 is the game's bundle of design tables: 17 fixed-size record tables, most with a string pool behind them, plus 3 path records that name sound banks and give a directory of entries in each bank. The book records in resource 576, which the third directory indexes, are described under Book records. Records are looked up by id (directly or with a split at id 200), and most string fields are pointers into the pool. The clean side can build the item database, quest journal, location names, pickup points, emote and message selection, credits, and several tuning arrays from this resource. Some tables are only partly understood; those are marked, and their use is listed under Open questions. The per-table fields below are facts to implement from; do not guess the unknown ones.

## Where it lives on the disc

- The resource is number 533 in the game data archive (numbering per SPEC-0001). It is an IDM chunk file of 112,640 bytes.
- Nothing from this resource is stored in the boot program; everything is read from the archive at runtime. The credits colours and scroll constants used with table 15, and the Mote-to-status map used with table 16, are in the boot program, not in this resource (see those tables).
- Resource 527 is not part of this spec (it is a light setup list).

## Layout

### Container

The resource is a sequence of top-level chunks. Each chunk starts with a 16-byte IDM header (magic "IDM", a format byte, a u16 type at +4, a u16 extra at +6, a u32 size at +8, and a u32 at +12). Chunks are contiguous with no gaps; the next chunk starts at the previous chunk's offset plus its size.

Top-level chunks, in file order:

| # | Type | Offset | Size (bytes) | Content |
| --- | --- | --- | --- | --- |
| 0 | 42 | 0x00000 | 33,280 | Table 0 (181 x 52) + string pool |
| 1 | 42 | 0x08200 | 24,048 | Table 1 (238 x 36) + pool |
| 2 | 42 | 0x0DFF0 | 5,888 | Table 2 (732 x 8) |
| 3 | 42 | 0x0F6F0 | 1,424 | Table 3 (48 x 20) + pool |
| 4 | 42 | 0x0FC80 | 7,792 | Table 4 (194 x 40) |
| 5 | 42 | 0x11AF0 | 1,880 | Table 5 (77 x 8) + pool |
| 6 | 42 | 0x12248 | 2,016 | Table 6 (248 x 8) |
| 7 | 42 | 0x12A28 | 476 | Table 7 (148 x 3) |
| 8 | 42 | 0x12C04 | 612 | Table 8 (193 x 3) |
| 9 | 42 | 0x12E68 | 2,744 | Table 9 (27 x 16) + pool |
| 10 | 42 | 0x13920 | 40 | Table 10 (2 x 4) |
| 11 | 42 | 0x13948 | 2,728 | Table 11 (674 x 4) |
| 12 | 42 | 0x143F0 | 1,108 | Table 12 (179 x 6) |
| 13 | 42 | 0x14844 | 532 | Table 13 (36 x 4) |
| 14 | 42 | 0x14A58 | 216 | Table 14 (91 x 2) |
| 15 | 42 | 0x14B30 | 11,876 | Table 15 (623 x 8) + pool |
| 16 | 42 | 0x17994 | 272 | Table 16 (60 x 4) |
| 17 | 43 | 0x17AA4 | 9,756 | Path record (sound bank for emotes) |
| 18 | 43 | 0x1A0C0 | 3,212 | Path record (sound bank for hero emotes) |
| 19 | 43 | 0x1AD4C | 980 | Path record (books sound bank) |

Each type-42 chunk has a payload that starts at chunk offset + 16 with a table block (an IDMX structure, magic "IDMX"). The table block is:

| Offset in table block | Size | Meaning |
| --- | --- | --- |
| +0x00 | 4 | Magic "IDMX" |
| +0x04 | u16 | Sub-type, 76 on all 17 tables |
| +0x08 | u32 | Byte size of the table block = 16 + count x record size |
| +0x0C | u16 | Record count |
| +0x0E | u16 | Record size in bytes |
| +0x10 | count x size | Records, back to back |

For all 17 tables, 16 + count x record size equals the size field at +0x08 exactly. A string pool, if the table has one, starts immediately after the table block, inside the same chunk (chunk 0 pool starts at 0x24E4 in the file; the chunk payload continues to the chunk end).

Table counts and record sizes (from disc):

| Table | Records | Record size | Table block bytes |
| --- | --- | --- | --- |
| 0 | 181 | 52 | 9,428 |
| 1 | 238 | 36 | 8,584 |
| 2 | 732 | 8 | 5,872 |
| 3 | 48 | 20 | 976 |
| 4 | 194 | 40 | 7,776 |
| 5 | 77 | 8 | 632 |
| 6 | 248 | 8 | 2,000 |
| 7 | 148 | 3 | 460 |
| 8 | 193 | 3 | 595 |
| 9 | 27 | 16 | 448 |
| 10 | 2 | 4 | 24 |
| 11 | 674 | 4 | 2,712 |
| 12 | 179 | 6 | 1,090 |
| 13 | 36 | 4 | 160 |
| 14 | 91 | 2 | 198 |
| 15 | 623 | 8 | 5,000 |
| 16 | 60 | 4 | 256 |

Table identity comes from file order: the k-th type-42 chunk (k = 0 to 16) is table k. Any extra type-42 chunk, or a chunk of an unknown type, makes the resource invalid.

### Pointers and string relocation

- A pointer field is a u32 offset measured from the start of the table block (chunk offset + 16), not from the chunk or the file start. A value of 0 means "no string".
- To resolve a non-zero pointer, add the table block's file offset to it. The result is the start of a NUL-terminated string.
- Synthetic example: a table block at file offset 0x1000 with a pointer field of 0x0200 points to the string at file offset 0x1200.
- Pointer fields per table (offset within the record, 4 bytes each, unless noted):
  - Table 0: +0x00, +0x20, +0x24, +0x28, +0x2C, +0x30
  - Table 1: +0x0C, +0x10
  - Table 3: +0x00
  - Table 5: +0x00
  - Table 9: +0x00, +0x04, +0x08
  - Table 13: the whole 4-byte record (each of its 36 records is one pointer)
  - Table 15: +0x00 (in each 8-byte record; the second u32 is not a pointer)
- All other fields are plain values. Some fields are runtime state that is zero on disc (listed below); the clean side must start them at zero and not read them as saved progress.

### Strings

- Each string is a run of bytes ending at a NUL byte. One byte per character.
- Printable ASCII is the bulk. Strings use the control-code format of SPEC-0022 (Text encoding). Every byte of 0x01 to 0x06 and 0x09 to 0x0D, and every byte of 0x80 or above, in the design-table strings is part of a well-formed control code (the marker, n, L, type, V or an escaped argument byte), with one exception: the two 0xAE bytes (registered mark) in the text of table 9 record 25, which font 0 draws as a glyph (map value 92). No newline byte occurs outside a code. Keep the raw bytes; do not drop or trim them. Counts on disc (High):
  - Table 0 (452 strings): 255 codes, each decoding to exactly 4 × (n − 2) bytes. 213 are page stops (type 4, L 5, V 6). 42 are colour codes (type 2, V 3, L 9, 10 or 11) in 8 colours: neutral 128 (12 codes), (139, 35, 35) (9), Roberta's green (6), (255, 156, 0) (4), Tor's blue (4), (255, 240, 0) (3), (124, 28, 168) (3) and (107, 27, 44) (1), all with alpha 128. The strings with codes are field +0x24 (103), +0x30 (17), +0x2C (16), +0x28 (5) and +0x20 (2). The 8 distinct bytes of 0x80 or above are colour values and the escape byte 0xFD.
  - Table 9 (64 strings): 19 substitution codes (type 5, n 3, L 13, V 34), all in field +0x00 of 16 records; first argument 0 in 16 codes and 1 in 3. The dialogue box replaces each one with substitution slot 0 (the memory-card slot number) or slot 1 (the KB a save needs) when it opens the message (SPEC-0022, Substitution codes).
  - Tables 1, 3, 5, 13 and 15: no control bytes.
- Pointer checks on disc: every non-zero pointer in table 1 (476 of 476), table 3 (44 of 44), table 5 (77 of 77), table 13 (36 of 36) and table 15 (450 of 450) resolves to a non-empty string. Table 0 has 452 non-zero pointers across its six pointer fields, all resolving to non-empty strings.

### Path records (type 43, chunks 17 to 19)

Each of the three chunks is a path record. Its fields are payload-relative:

| Offset | Size | Type | Meaning | Confidence |
| --- | --- | --- | --- | --- |
| +0x000 | 256 | bytes | Name: a NUL-terminated string, zero-padded. The loader does not use it. | H |
| +0x100 | 4 | u32 | Entry count N. | H |
| +0x104 | 8 x N | entries | Entry i for i = 0 to N - 1: u32 start in units, then u32 length in units. | H |

Rules: the entries are contiguous. The start of entry i + 1 equals the start of entry i plus its length. Entry i covers units [start, start + length). Entries of length 0 are present and cover no units. The directory covers the whole resource: the lengths add up to the resource size. A unit is 2,048 bytes (M for the unit meaning).

Counts and totals (H):

| Chunk | Resource | Entries (N) | Entries of length 0 | Total units |
| --- | --- | --- | --- | --- |
| 17 | 144 | 1,185 | 189 | 10,056 |
| 18 | 145 | 367 | 9 | 3,327 |
| 19 | 576 | 88 (all of length 76) | 0 | 6,688 |

### Book records (resource 576)

Resource 576 holds 88 book records (indices 0 to 87), each 76 sectors long. The chunk 19 directory gives each record's start and length (see Path records). Each record is a chunk chain in this order: type 2, type 8, type 2000 (text), type 2001 (layout), type 4, type 4, then a type-0 terminator. Unparsed bytes may follow the terminator up to the end of the record. Book text is not copied into this spec.

Text chunk (type 2000), offsets within the chunk payload:

| Offset | Size | Type | Meaning | Confidence |
| --- | --- | --- | --- | --- |
| +0x00 | 4 | u32 | Title length, counting the terminating NUL. | H |
| +0x04 | 4 | u32 | Style index. It selects a style value in the program's data and is passed to the line layout. | H (use M) |
| +0x10 | title length | bytes | Title, NUL-terminated (exactly one NUL, at the stated length). At most 255 bytes are copied to the shared text buffer. On disc the title length is 7 to 39 bytes including the NUL (27 distinct values), so no title reaches the 255-byte limit. | H |
| after the title | to chunk end | bytes | Body paragraphs, each NUL-terminated. Their layout is not described here. | H (count) |

Layout chunk (type 2001): a font descriptor in the 0x110-byte format of SPEC-0022 (Fonts), followed by its glyph records. Offsets within the chunk payload:

| Offset | Size | Type | Meaning | Confidence |
| --- | --- | --- | --- | --- |
| +0x00 | 4 | u32 | Texture-template slot. Zero on disc; the loader points it at the book's 8-byte texture template. | H |
| +0x04 | 1 | u8 | Number of glyph records: 91. | H |
| +0x05 | 1 | u8 | Mode of the character table: 4 (code map). | H |
| +0x06 | 1 | u8 | First character code (32). | H |
| +0x07 | 1 | u8 | Last character code (241). | H |
| +0x0C + code | 1 per code | u8 | Code map: glyph number for character code `code` (v gives glyph index v - 1; 0 means none). | H |
| +0x10C | 4 | u32 | Payload-relative pointer to the glyph records, which start at +0x110. | H |
| +0x110 | 18 x 91 | records | Glyph records (1,638 bytes) in the 18-byte format of SPEC-0022 (Fonts). | H |

Bytes +0x04 to +0x10B are byte-identical to the descriptor of front-end font 2, so the book uses font 2's code map; the table is the same in all 88 records. The glyph records are the book's own, for its own glyph sheet. There is no separate metrics block or rectangle data: offset +0x0C of the glyph array (value 5) is glyph 0's advance, the space advance, and offset +0x1C (value 18) is glyph 1's height field, the line pitch in pixels at scale 1.0. The rest of the array is the other glyph records.

Glyph coverage (High for the counts). In the title and paragraph strings (from text payload +0x10 to the chunk end, split at NUL), the only bytes without a glyph are newline (314 on disc, a line end in the text engine), one "{" and one "}". Codes 1, 4, 7, 8 and 12 occur only in the header words of the text chunk (+0x00 to +0x0F), not in the strings (Medium-High). Codes 144 and 148 do not occur in the strings. The clean side must keep the strings as raw bytes and must not treat a byte without a glyph as an error; such a byte draws nothing.

## Tables

Confidence labels: H = high, M = medium, L = low. "Runtime" marks a field that is zero on disc and set by the game during play.

### Table 0: items (181 x 52) (purpose: item database, M-H)

| Offset | Size | Type | Meaning | Confidence |
| --- | --- | --- | --- | --- |
| +0x00 | 4 | pointer | Item name. 163 of 181 records have one. | H |
| +0x04 | 4 | u32 | Category code, values 0 to 9. Category 7 is the most common; 0 and 6 are next. Some game checks test for category 7. | M |
| +0x08 | 4 | u32 | Carry limit Q: the most of this item the player can hold (SPEC-0021). Small (0 to 5 seen). | H |
| +0x0C | 4 | u32 | Price in currency units. 110 records are 0; the maximum is 15,000. | M |
| +0x10 to +0x18 | 12 | 3 x u32 | Zero in all records. | H |
| +0x1C | 2 | s16 | Progress-flag id that selects between the texts at +0x2C and +0x30 (see Item text by flag); -1 in 163 records (no flag). | H |
| +0x1E | 1 | s8 | Read as a signed byte and handed to the caller by the item text getter; values 0 to 4. Meaning open. | L |
| +0x1F | 1 | u8 | Flag, 0 or 1. Meaning open. | L |
| +0x20 | 4 | pointer | Item description. Zero in 73 records; 106 of the 108 others resolve to text. | M |
| +0x24 | 4 | pointer | Text used when the player holds fewer than 1 of the item. | H |
| +0x28 | 4 | pointer | Text used when the player holds 1 or more of the item. | H |
| +0x2C | 4 | pointer | Text used when the progress flag at +0x1C is clear. | H |
| +0x30 | 4 | pointer | Text used when the progress flag at +0x1C is set. | H |

Lookup: the record for item id n is at index n (direct). Ids outside 0 to 180 have no record.

Item text by flag (Medium-High). When an item showcase starts, the game picks one text for the item. Let s be true when the held count of the item is below 1.
1. If s is true and +0x24 is non-zero, use +0x24.
2. Otherwise, if s is false and +0x28 is non-zero, use +0x28.
3. Otherwise, if +0x1C is 0 or more, use +0x2C when progress flag (+0x1C) is clear and +0x30 when it is set.
4. Otherwise there is no text.
The getter also hands +0x1E (signed) to the caller. The showcase passes the chosen text to the world script's item callback.

Use: item name and description appear in use and inventory messages; the category and price are read by shop and inventory code.

### Table 1: quest and journal entries (238 x 36) (purpose: quest journal with map markers, M)

| Offset | Size | Type | Meaning | Confidence |
| --- | --- | --- | --- | --- |
| +0x00 | 4 | u32 | Row kind: 0 = main story (158 records), 1 = side quest (80 records) (SPEC-0049, SPEC-0058). The journal loop goes through all records. | H |
| +0x04 | 4 | s32 | Quest start flag: a progress-flag id (0 to 742), or -1 for none (SPEC-0049). | H |
| +0x08 | 4 | s32 | Quest done flag: a progress-flag id, or -1 for none (SPEC-0049). | H |
| +0x0C | 4 | pointer | Short title. | H for text |
| +0x10 | 4 | pointer | Longer objective text. All 238 resolve. | H for text |
| +0x14 | 4 | u32 | State or stage, 0 to 2. | L |
| +0x18 | 4 | f32 | Map x position. | M |
| +0x1C | 4 | f32 | Map z position. | M |
| +0x20 | 4 | s32 | Scene id; -1 in 40 records (the entry shows in every scene). Compared with the current scene. | M |

### Table 2: progress-flag cues (732 x 8) (purpose: stinger and journal cue per progress flag, H)

Row i belongs to progress flag i (the flag bitmap of SPEC-0049).

| Offset | Size | Type | Meaning | Confidence |
| --- | --- | --- | --- | --- |
| +0x00 | 4 | s32 | Journal kind: 0 (518 records), 2 (212), 1 (1, record 110), -1 (1, record 0). A non-zero kind lights the journal-notification icon when the flag rises (see Behavior). | H |
| +0x04 | 4 | s32 | Stinger value v: 0 (332 records) or 1 to 19 (maximum 19). When the flag rises and v is 1 or more, stinger v - 1 plays. Record 0 holds -1 (see Edge cases). | H |

Indexed directly by flag id (0 to 731). Flag ids 732 to 742 exist in the bitmap but have no row (see Edge cases).

### Table 3: pickup or treasure points (48 x 20) (purpose: named pickup points with a taken flag, M)

| Offset | Size | Type | Meaning | Confidence |
| --- | --- | --- | --- | --- |
| +0x00 | 4 | pointer | Point name. 44 of 48 records have one; all resolve. | H for text |
| +0x04 | 4 | f32 | x position. | M |
| +0x08 | 4 | f32 | z position. | M |
| +0x0C | 4 | s32 | Gate: -1 in 4 records (skipped by scans); other small values. | M |
| +0x10 | 4 | u32 | Runtime taken flag. Zero in all 48 records on disc. | M |

Lookup by id (shared rule, see Behavior): index = id when id <= 200, else index = id - 200. Table 3 is scanned together with table 6 (see table 6).

### Table 4: music track parameters (194 x 40) (purpose: music track rows, H)

194 records of 40 bytes, one per music row. The field roles and their use are owned by SPEC-0009 (Track row and its fields); this table restates them. Getters accept ids 0 to 192. Ids 193 and above return the defaults. Record 193 (the last of the 194) is therefore never read.

| Offset | Size | Type | Meaning | Default | Confidence |
| --- | --- | --- | --- | --- | --- |
| +0x00 | 4 | s32 | Stream offset, 0 to 54 on disc (no -1 on disc; -1 keeps the current stream, SPEC-0009). | -1 | H |
| +0x04 | 4 | s32 | Bank offset for the track. 0 to 66 in normal records; -1 keeps the current music; -3600 (four records) and -6000 (one record) fade the music out over that many milliseconds and stop it (SPEC-0009). The bank started is this value plus 10, and plus 67 more when a run-time flag is set (SPEC-0009, "Music selection"). | 1 | H |
| +0x08 | 4 | s32 | Block offset, 0 to 59; -1 in 8 records (keep the current block); -3600 in 3 records and -6000 in 1 record (below -1: stop the block, SPEC-0009). | 1 | H |
| +0x0C | 4 | u32 | Cap: the stream volume ceiling in 1/1024 units (300 to 900 on disc). | 0 | H |
| +0x10 | 4 | u32 | Level: the bank target volume in 1/1024 units (600, 700 or 750 on disc). | 1024 | H |
| +0x14 | 4 | u32 | Scale: the sound-block slot volume in 1/1024 units (1024 = full), multiplied by the sound-effects gain; 1024 in all 194 records. | 1024 | H |
| +0x18, +0x1C | 4 each | u32 | First reverb pair (SPEC-0009). | 0 | H (role) |
| +0x20, +0x24 | 4 each | u32 | Second reverb pair (SPEC-0009). | 0 | H (role) |

None of these fields is a timing. The only times are the bank offsets -3600 and -6000, which are fade times in milliseconds; in the block offset the same values only stop the block (SPEC-0009).

### Table 5: location names (77 x 8) (purpose: area names, M-H)

| Offset | Size | Type | Meaning | Confidence |
| --- | --- | --- | --- | --- |
| +0x00 | 4 | pointer | Location name. All 77 resolve. | H |
| +0x04 | 4 | u32 | Runtime count of flagged table-6 entries for this location. Filled at load time; zero on disc. | M |

A save-slot list shows the location name; if the id is missing the game shows a generic "unknown location" label (text not copied here).

### Table 6: per-id collected or visited flag (248 x 8) (purpose: collected or visited marks grouped by location, M)

| Offset | Size | Type | Meaning | Confidence |
| --- | --- | --- | --- | --- |
| +0x00 | 4 | s32 | Location index into table 5, or -1 for none. Values -1 to 73. | H for range |
| +0x04 | 4 | u32 | Runtime flag: 0 to 1 when obtained or visited. Zero on disc in all records. | M |

Behavior: a per-location count sums the flags of the table-6 entries whose location index matches. Writers (High):

- The flag is set to 1 only by the three treasure-group callbacks: the Square-press chest, the element-break chest and the dig site (SPEC-0048, SPEC-0051). Each writes the row named by its group's collected-row key.
- The single-row write ignores row 0 and any row at or past the row count.
- A save load rewrites rows 1 to min(row count, 255) - 1 from the collected-treasure set of the slot (SPEC-0025).
- The new-game world reset clears the flag of every row, 0 to row count - 1 (all 248 on disc), in one pass, together with every table-3 held flag.

### Table 7: character emote clips (148 x 3) (purpose: emote clip choice, H)

Each row is 3 bytes. The row number is the emote id, which is also a townsfolk animation id: row i is the voice of animation id i, one row per entry of the 148-entry main and face animation tables of every townsfolk record (High). A body emote plays main animation i and a face emote face animation i; the row adds an optional clip.

| Offset | Size | Type | Meaning | Confidence |
| --- | --- | --- | --- | --- |
| +0x00 | 1 | u8 | Clip index in the character emote bank (resource 144); 255 = none. Disc: 77 rows are 255; 71 rows have an index from 0 to 74. | H (values); H (clip arithmetic) |
| +0x01 | 1 | u8 | Chance in percent: 100 in 59 rows (always play) and 0 in 89 rows (never play). No other value. | H (values); M (use) |
| +0x02 | 1 | u8 | Variance byte; zero in all 148 rows, so no random step applies. | H (values) |

- Always-play rows: 59 (chance 100), all with a clip index.
- Never-play rows: 89 (chance 0). Of these, 77 have no clip index and 12 have one.
- Character clip = the clip index read from the row, plus 74 times the character's voice set c (villager row +0x14; see Species and villager tables). The stride 74 is a fixed constant of the program, not a table value (High). The arithmetic follows SPEC-0009 (Emote clip lookup), which is the reference for lookup steps.
- Clip index 74 occurs on disc, so an index cap at 73 is not the rule. With the stride of 74, clip index 74 of one character is the same bank clip as clip index 0 of the next character. This overlap is real and must be reproduced; the stride is not 75.
- Chance test: a draw of 0 to 99 is compared with the chance byte, and the clip plays when the draw is below the chance. A row with clip 255 is not guarded by the picker, but every such row on disc has chance 0, so none plays (High).

### Table 8: hero emote clips (193 x 3) (purpose: hero emote clip choice, H)

Each row is 3 bytes, with the same layout as table 7. The row number is the emote id, which is also a hero animation id: row i is the voice of hero animation id i, one row per entry of the 193-entry hero action tables (resources 520 to 525; SPEC-0005) (High). Disc check: the body emote ids 182 to 191 have a main animation for both heroes and no face animation, and the face emote ids 122 to 132 have a face animation for both heroes and no main animation. Disc values:

- Clip index: 175 rows are 255 (none) and never play. The other 18 rows have clip indices from 4 to 178 (17 distinct values). Row 10 is a none row. The always-on row with clip index 10 is row 124; its random step can reach positions 9, 10 and 11, which have zero length and play nothing.
- Chance: 0 in the 175 none rows and 100 in the 18 live rows. Chance is only ever 0 or 100: 100 always plays and 0 never plays. Rows with clip index 255 never play.
- Variance byte: zero in the 175 none rows and nonzero in the 18 always-play rows. Only zero or nonzero is tested. A nonzero value gives a random step (see below).

The clip index is a position in the clip directory of the hero emote bank, which is resource 145. That directory has 367 positions, 358 of them with a nonzero length (see Path records). The random step and the hero shift of 183 are described under Emote choice below and in SPEC-0009 (Emote clip lookup). With the step, the hero positions reached by these rows are 3 to 179 for Tor and 186 to 362 for Roberta. The two ranges do not overlap.

Random step: when the variance byte is nonzero, the clip index is moved by a random step: a uniform whole number from {0, 1, 2}, minus 1, so -1, 0 and +1 are equally likely (High). When the byte is zero, there is no step. The step is applied per play, not stored in the table. The hero shift for Roberta is half the entry count of the hero emote directory (chunk 18), rounded down: 367 / 2 = 183 (High).

### Table 9: system message boxes (27 x 16) (purpose: memory-card and controller prompts, H content, M use)

| Offset | Size | Type | Meaning | Confidence |
| --- | --- | --- | --- | --- |
| +0x00 | 4 | pointer | Message text. All 27 resolve. 16 records hold substitution codes (19 in all; see Strings). | H |
| +0x04 | 4 | pointer | First button label, or zero (19 records have one). | H |
| +0x08 | 4 | pointer | Second button label, or zero (18 records have one). | H |
| +0x0C | 4 | u32 | Packed colour for the box's text line. Bytes 0, 1 and 2 are red, green and blue (the same order as table 10). Byte 3 is 0x80 in all records and is not read. Three distinct colours: (100, 0, 0) in 13 records (including record 25), (74, 100, 255) in 11 and (0, 222, 0) in 3; the last two equal the Tor and Roberta name colours of table 10. | M-H |

Content: memory-card messages (save space, card missing, save or load success, cancel and format prompts) and one message about an analog controller, at record 25. The record's colour fills the same colour slot as a dialogue line's colour (SPEC-0022); a null colour gives (128, 128, 128). The speaker name is empty for these boxes. Record 25 is shown while pad 0 is not ready: every front-end state then runs a second box with its text and colour, an instant reveal, alpha 128 and no prompt row (SPEC-0022, Memory-card and controller messages).

### Table 10: hero name colours (2 x 4) (purpose: colours for the two hero name variants, M)

Two packed colour words. The first word is for name variant 0 (Tor) and the second for variant 1 (Roberta). Name variant is chosen by the hero flag (0 or 1). Bytes 0, 1 and 2 are red, green and blue; byte 3 is not used by the dialogue box. Values: Tor (74, 100, 255), Roberta (0, 222, 0) (SPEC-0022, Speaker name and colour). Word 0 is read for hero flag 0 and the next word (4 bytes on) for hero flag 1, by the dialogue name colour and by the level message and shop prompts.

### Table 11: stage flags (674 x 4) (purpose: progress flags set by a stage, H)

| Offset | Size | Type | Meaning | Confidence |
| --- | --- | --- | --- | --- |
| +0x00 | 2 | u16 | Stage key, 0 to 7. Key 0 in 184 records. | H |
| +0x02 | 2 | s16 | Progress-flag id f to set, -1 to 707; -1 means no flag (SPEC-0049). | H |

### Table 12: start-point item grants (179 x 6) (purpose: inventory given by a stage, H)

| Offset | Size | Type | Meaning | Confidence |
| --- | --- | --- | --- | --- |
| +0x00 | 2 | u16 | Stage key, 0 to 7. | H |
| +0x02 | 2 | s16 | Item id a (a table 0 index), -1 to 176. One row (stage 7) has -1 and is skipped. | H |
| +0x04 | 2 | s16 | Count b, 1 to 10,000 (18 distinct). | H |

Behavior (applying stage k, from new game or the start-point chooser, SPEC-0049):
1. Every table 11 row with key k and f of 0 or more sets progress flag f with the plain set. The plain set does not read table 2: no stinger, no journal cue and no flag notification.
2. Every table 12 row with key k, a of 0 or more and b above 0 calls the normal add-item action with item a and count b. All known callers do this step.
3. Add item follows SPEC-0021: the carry limit Q (table 0 +0x08) applies, and a Mote is added with a charge count of 100. Item 135 adds b Neopoints to the wallet. Items 136 to 139 add 5, 10, 25 and 100 Neopoints whatever b is.

Table 12 content (disc, summary): stage 0 grants 1,000 Neopoints (item 135), equipment items 100, 107 and 121, 200 of each Mote 160 to 171, and consumables. Stages 2 and 3 each grant 10,000 Neopoints, 800 of each Mote 160 to 171 and 500 each of Motes 172 and 173. Stage 1 grants items 112, 128 and 176. Start index 0 applies no stages, so a plain new game grants nothing from this table.

### Table 13: action labels (36 x 4) (purpose: player-visible action labels, M)

This table is not a sorted id list. It holds 36 u32 offsets, one per label. Each offset is payload-relative (pointer rule above) and points to a NUL-terminated label string in the table's string pool. The offsets are strictly increasing, and the first is 0xA0.

| Offset | Size | Type | Meaning | Confidence |
| --- | --- | --- | --- | --- |
| +0x00 + 4 x i | 4 | pointer | Label i, for i from 0 to 35. | H for content |

Labels are player-visible text. Examples: 0 "No Message", 1 "Turn On", 2 "Turn Off", 3 "Examine", 4 "Use", 35 "Hero". The other labels are read from the string pool and are not copied here.

Index use (Medium-High):
- The hero keeps a list-label word (-1 = none) and an accepted word (0 or 1). The hero update clears the list-label word to -1 once per tick. Hero init and reset set the list-label word to -1 and the accepted word to 0, and the scene setup sets the accepted word to 0 on both heroes.
- Touching a type-40 interaction entry (SPEC-0028, section 11), with the active hero and no world event busy: the entry is marked touched. If the entry's kind is non-zero and it has no running action, let i be the entry's field at +0x0C (on disc 0, 1, 2, 10 or 12). If i is 1 to 35, i is stored in the hero's list-label word, and the action fires only on a touch in a tick where Square has a press edge and the hero's accepted word is 1. If i is 0 or out of range, the action fires at once on contact, with no button. Firing starts the entry's action thread once; it does not restart while it runs.
- The hero prompt chooser shows label (list-label word) of this table as the Square prompt when no earlier check took the prompt (SPEC-0022, Interaction prompts). On a Square press edge it sets the accepted word to 1.
- Door labels: the exit and door path sets the label to 13 ("Enter") or 26 ("Exit") directly, and clears the accepted word when it acts.
- Host-call labels: scripts show a label through the DisplaySquareButtonContext host call (SPEC-0030): label i of this table is used as a format string with the call's extra arguments, and i = 0 clears the prompt. On disc the scripts pass 1, 2, 3, 4, 5, 6, 13, 26 and 32 (3, 3, 43, 84, 9, 20, 70, 23 and 2 calls). Label 2 ("Turn Off") is shown by scripts.
- Fixed uses by the game: 8 (Talk), 9, 10, 11, 12, 14, 16 and 17 (dialogue next and finish), 18 to 25 (25: save point), 27, 28, 31 (book), 33 and 34 (yes and no). Labels 7, 15, 29, 30 and 35 have no known reader (Medium). Label 0's text is never shown.

Label 31 ("Read Book") is used by no type-40 entry on disc. The book prompt shows it directly, without a type-40 entry, while the hero stands in a book station (SPEC-0035, section 6.1).

### Table 14: weighted clip classes (91 x 2) (purpose: stinger clip choice; H for records and for indices 0 to 18)

91 records of 2 bytes each.

| Offset | Size | Type | Meaning | Confidence |
| --- | --- | --- | --- | --- |
| +0x00 | 1 | u8 | Weight in percent: 10, 15, 20, 30, 50 or 100. | H |
| +0x01 | 1 | u8 | Class index, 0 to 65. Records 0 to 18 are the one-entry classes 0 to 18, each with weight 100. | H |

Behavior: the records of a class are contiguous. A stinger effect index k (0 to 18) selects class k, which has one record of weight 100. That record is always chosen, so stinger k plays clip number 256 plus k (the clip numbering in SPEC-0009). The mapping is the identity. A count and a start record for each class are built at load time; the start is correct for classes 0 to 18 only (see Edge cases).

Roll: the chooser draws a uniform whole number from 0 to 100 inclusive (101 values). For a one-record class of weight 100 it always returns that record.

Dropped indices 19 and above (cut stingers): stinger requests of 19 or more are made and are dropped. The game requests indices 19, 20, 21, 25, 26, 27, 28, 30, 32, 33 and 37 on Tor events and the same events plus 24 (43, 44, 45, 49, 50, 51, 52, 54, 56, 57 and 61) on Roberta events, plus 42 (Tor) and 66 (Roberta) when an item showcase starts. The events are take-hit, attack actions, item add and item showcase. The stinger routine rejects every index of 19 or more (SPEC-0009, "Effect and stinger index"), so nothing plays. Classes 19 to 65 hold records 19 to 90, whose clip numbers would be 256 + 19 to 256 + 90 = 275 to 346. Archive resources 275 to 346 are exactly the 72 empty (size 0) archive entries (SPEC-0001). These are cut hero-voice and item-event stingers. Class 66 has no records. A clean build must play nothing for them.

Do not implement a weighted pick for classes 19 to 65 from this table. The chooser does not implement it (see Edge cases).

### Table 15: credits roll (623 x 8) (purpose: credits text, H content, M mechanism)

| Offset | Size | Type | Meaning | Confidence |
| --- | --- | --- | --- | --- |
| +0x00 | 4 | pointer | Credit line. 450 records have one; 173 are blank spacers (zero). All non-blank lines resolve. | H |
| +0x04 | 2 | u16 | Style index. Text rows: 3 (287), 2 (139), 1 (12), 7 (9), 0 (3: rows 0, 3 and 22). Blank rows: 0xFFFF (169), 5 (2: rows 214 and 524), 6 (2: rows 335 and 557). No text row has 0xFFFF. | H |
| +0x06 | 2 | u16 | Colour set, 0 (340) or 1 (283). | H |

Behavior (High). Units are pixels and passes; one pass per tick, 60 per second.
- Colour: the boot program's data holds a credits colour table at 0x2DCFA8: two sets of 8 packed colour words. The set is chosen by +0x06 and the word by the style index. The text uses bytes 0 to 2 (red, green, blue) of the chosen word, and the fade alpha replaces byte 3. Read the colours from that table.
- Scroll: the line pitch is 24.0 (f32 at 0x2DCF84). The first line starts at y = screen height + 1 (the whole number at 0x2DCF94). Each pass, y falls by the current speed. When the top line's y is below -20 (the whole number at 0x2DCF90), that line is dropped and y gains 24. The roll ends when every row has been dropped.
- Alpha: 128 x (y / 32) within 32 px of the top (the whole number at 0x2DCF98), 128 x ((H - y) / 32) within 32 px of the bottom (0x2DCF9C), where H is the screen height, else 128; clamped to 0 to 128.
- Speed: the current speed moves toward the target speed by 1/512 per pass (0x2DCFA0). The target starts at 1.0 (0x2DCF88). Each pass, every row from the top line down to the first row below the screen is walked. A style-5 row sets the target to 1.5 (0x2DCF8C), a style-6 row sets it to 1.0, and the last one walked wins. The target then stays until another such row is walked.
- Style 7 flips a word that nothing reads, so it has no visible effect beyond its colour.
- Blank rows (pointer 0) take up a line of space like text rows.

### Table 16: Mote parameters (60 x f32) (purpose: per-hero Mote power and status effect, H)

60 f32 values, one per 4 bytes from the payload + 0x10 position (the table block is 256 bytes). There are 13 distinct values; the most common are 1.0 and 2.0, then 30.0 and 5.0.

- Layout: two hero blocks of 30 floats. Indices 0 to 29 are hero row 0 (Tor); indices 30 to 59 are hero row 1 (Roberta). In each block the Motes come in id order 160 to 173. Each Mote has 1, 2 or 4 consecutive floats, in this order: P (power factor), then D (status duration, seconds), then A (status amount), then T (status period, seconds).
- Fields per Mote: Motes 160, 161 and 164 have P, D, A and T. Mote 163 and Motes 166 to 171 have P and D. Motes 162, 165, 172 and 173 have P only. That is 4 + 4 + 1 + 2 + 4 + 1 + 2 + 2 + 2 + 2 + 2 + 2 + 1 + 1 = 30 per hero.
- Index rule: within a hero block (base 0 or 30), the offsets are: 160: P 0, D 1, A 2, T 3. 161: P 4, D 5, A 6, T 7. 162: P 8. 163: P 9, D 10. 164: P 11, D 12, A 13, T 14. 165: P 15. 166: P 16, D 17. 167: P 18, D 19. 168: P 20, D 21. 169: P 22, D 23. 170: P 24, D 25. 171: P 26, D 27. 172: P 28. 173: P 29.
- Coverage: each of the 60 indices is read by exactly one field reader: 28 P, 20 D, 6 A and 6 T.
- P reader: returns the float. For a hero row other than 0 or 1, an id outside 160 to 173, or a missing table, P is 1.0 (SPEC-0036).
- D, A and T readers (integer readers): the float is converted toward zero to a whole number (all disc values are whole). A missing field (a Mote without D, A or T, an id outside 160 to 173, or a hero row other than 0 or 1) reads -1. With the table missing, the clean build returns -1 as well.
- Meaning of D, A and T: they are the second to fourth words of the status request (status index, duration, amount, period) that a Mote hit sends to its target (Behavior: Mote status requests). -1 means "use the receiver's default".
- Status index per Mote: a fixed map in the program, not in this table. Motes 160 and 161 give 7 (fire); 163 gives 0 (stun); 164 and 165 give 2 (poison); 166 and 167 give 3 (slow); 168 and 169 give 1 (curse); 170 and 171 give 6 (confusion); 162, 172 and 173 give -1 (no status). The status numbers are those of SPEC-0036 (Status timers).
- Rock quirk: Mote 165 (Rock, the greater Mote of the poison pair) maps to poison but has no duration field (and no amount or period), so its request is (2, -1, -1, -1) and the receiver applies the default poison. The lesser Mote 164 (Leaf) sends its own words instead. The greater Mote therefore does not give a stronger poison than the default. Reproduce this.

### Species and villager tables (resources 530 and 531)

Confidence: High for the header layout and the on-disc counts; Medium for the load check; Low for the row fields (read by other specs), except the villager voice set at +0x14 (High).

These two tables are separate resources, not part of resource 533. They are loaded at boot under the type codes 0x212 and 0x213. Those codes are the resource numbers 530 and 531 in hexadecimal (0x212 = 530, 0x213 = 531). The table of resource 530 is named a species table in a size error message of an unused load routine (Load check), and a monster table in the enemy specs. Row use belongs to the enemy specs (SPEC-0029 and SPEC-0038), not to this spec.

Layout (both tables, High; read from the disc):

| Offset | Size | Meaning |
| --- | --- | --- |
| +0x00 | u32 | row count N; 112 in both resources |
| +0x04 | N x u32 | first row index for each type t; on disc these begin 0, 1, 2, 3, 4, 6 |
| align16(4 + 4N), which is 0x1D0 when N is 112 | rows | row i starts at the row base plus i times the stride; the stride is 0x504 in resource 530 and 0x60 in resource 531 |

On disc, resource 530 is 333,824 bytes: the 0x1D0-byte header, then 259 whole rows (ending at byte 333,020) and 804 trailing bytes. Resource 531 is 26,624 bytes: the header, 272 whole rows (ending at byte 26,576) and 48 trailing bytes. Both file sizes are whole multiples of 2,048 (163 and 13 sectors), and each tail is exactly the fill from the end of the last whole row to the next 2,048-byte boundary. The tails hold no table: the 530 tail is irregular bytes with no row or header structure, and the 531 tail is 14 zero bytes followed by 1.0 floats, a fragment, not a row. No lookup reaches them, because rows are found only through the index array, whose largest first-row index is 255 in both files, far below the whole-row counts (High).

Load check (Medium-High). At boot each table is read whole into boot memory at a 16-byte aligned address, with the remaining boot memory as the size limit. The boot memory pointer then moves on by the number of bytes read, so the tail is loaded too. There is no row count, stride or byte-length check on this path: any length is accepted. The table block then stores the row count read from word 0 of the file (+0x08), a pointer to the index array (+0x0C) and the row base (+0x10), which is the start of the rows after the aligned header. The row stride is not stored; it is fixed in the row lookup. A second load routine for each table compares the bytes read with an expected byte length and reports that the table has changed size and the build must be remade; nothing calls it, so the message never appears.

Row lookup: the row for type t and sub-entry sub is the row whose index is the first row index of type t plus sub. The lookup first checks that t is below N. A check that fails returns no row.

### Villager row voice set (resource 531)

Confidence: High.

| Row offset | Size | Type | Meaning |
| --- | --- | --- | --- |
| +0x14 | 2 | s16 | Emote voice set c: -1 = none, 0 to 15 = a 74-position block of the character emote bank (resource 144); the character emote clip is the table 7 clip index + 74 x c (SPEC-0009, Emote clip lookup) |

- The row is chosen at spawn: type t is the placement's species word (placement data +0xB4) and the sub-entry is its variant word (placement data +0xB2), so the row is (first row index of t + variant), at the 0x60 stride.
- Rows by c, over the 272 whole rows of resource 531: -1 in 156; 0 in 22; 8 in 14; 4 and 14 in 11 each; 15 in 10; 13 in 8; 1 and 2 in 7 each; 3 and 5 in 6 each; 6 and 12 in 5 each; 7, 9, 10 and 11 in 1 each.
- Placement census: over the 3,036 character placements on disc, c is -1 in 2,396 (enemies and creatures, which have no voice set) and 0 to 15 in the other 640. All 592 townsfolk records (main animation family -7) are voiced. Species 16 to 50 map onto the 16 voice sets; for example the guards share set 2, Master Torak is set 4, Hubert set 9, Patricia set 10 and Lucy set 11.
- Set 7 has no clips in the bank (positions 518 to 591 are all empty). Its only user is one unnamed record in pack 1259, which no route loads.

### Monster row fields (resource 530)

The row fields that the enemy specs read are listed in SPEC-0029 (Layout). Two fields have no confirmed use, so no rule may be built on them yet:

- Difficulty Value: u16 at row offset 0x3A. Values 0, 1, 2, 5, 8, 12 and 15 occur, and 65535 on 23 rows. No value is known to be scaled by it.
- Anger Level: s32 at row offset 0x134. Zero on 257 of the 259 rows and nonzero on 2.

Damage Reduction is read on the enemy damage path. It is f32 at row offset 0x114, zero on 255 rows and nonzero on 4 rows. The enemy class getter reads it, and it is subtracted from the product of the damage factors before the status scale is applied. Its use and layout are in SPEC-0029 (Layout) and SPEC-0036 (Damage).

Row selection by spawn record: a spawn record (layout in SPEC-0006) has a monster type t (u16 at its offset 0xB4) and a variant v (u16 at its offset 0xB2). The row used is the row whose index is first[t] + v, where first[t] is the first row index of type t in the header above. The pair is valid when t is below 112 and v is below the number of rows of type t (the first index of type t+1 minus first[t]). All 3,036 spawn records on disc are valid.

Rows store no model id. The model and animation for a placed monster come from the character region that holds the placement; the row does not name them.

Unused rows: 37 of the 259 rows are the selected row of no spawn record on disc. Grouped:

- Default templates (no class; 50 hit points, radius about 1.17, height 3.5): rows 17, 18, 19, 35, 70, 89, 92, 94, 115, 117, 123, 124, 133 and 255 to 258 (17 rows).
- Templates with a class id (template values, with a behaviour class): rows 2, 12, 13, 14 and 15 (5 rows).
- Own stats: rows 1, 142, 146, 168, 175, 178, 180, 181, 182, 203, 217, 218, 232, 233 and 243 (15 rows).

Type 15 has no spawn record, so rows 17 to 19 are unselected.

Rows with no loaded placement (14 rows): rows 9, 11, 51, 69, 71, 72, 73, 81, 99, 111, 121, 131 and 176 are selected only from world packs that do not load in play. Row 179 is selected only from a region that no level module loads.

Copies: row 142 is a copy of row 139 (same radius, height, hit points, attack types, attack damage values and damage reduction). Row 178 is identical to row 177, and row 180 is identical to row 182.

Examples (read from the disc):

- Type 80: the row has HP 50 for variants 0 to 6 and HP 40 for variants 7 to 13.
- Type 99, variant 2: the row has HP 110 and attack 1 damage 35.

## Behavior

### Progress-flag cues (table 2)

Confidence: High.

The cue fires on the rising edge of a progress flag (SPEC-0049 bitmap). A flag is set "with notify" by the SetGameState host call with true (SPEC-0030), by the game's own flag setters (for example flag 707 at new game, and flags 109, 110 and 273), and by the default-flag pass. When flag n (0 to 742) is set this way:

1. If a world is loaded and it has a script manager, post the flag notification (flag n, its value before the set, "set").
2. If a world is loaded and the flag's bit was 0: let v be the table 2 stinger value of row n. If v - 1 is not -1, play stinger v - 1 at volume 1.0 (SPEC-0009; SPEC-0030 row 227 rules). If the row's kind is not 0, set the journal-notification flag and set its phase to 0 (SPEC-0023, section 8).
3. Set the bit. This happens always, even with no world loaded.

Rules:
- The "seen marks" are the progress-flag bits themselves; there is no separate set. They are saved and loaded with the slot's 93-byte bitmap (SPEC-0025, SPEC-0049), cleared by new game, and cleared one at a time by a flag clear (SetGameState with false), which also posts a notification. A flag that is cleared and set again plays its cue again.
- Setting a flag that is already set plays nothing and changes nothing; the notification is still posted, with old value 1.
- The plain set used by the stage table (table 11) does not read table 2: no stinger, no journal cue and no notification.
- Journal flag consumer: while the flag is set, the in-play HUD draws the pulsing journal icon (SPEC-0023, section 8). Building the quest journal list clears the flag. Nothing else clears it: not new game, not a load. A front-end handler also reports it when its save check passes (Medium for that handler).
- Stinger 4 cooldown: stinger 4 plays only when the game clock (seconds since boot, +1/60 per pass; SPEC-0007) is more than 40.0 s past the time stinger 4 last started. That last-start time is set to 0.0 when the music object is set up (and at its resets), so stinger 4 cannot play while the clock is at 40 s or less. A blocked stinger 4 is dropped, not queued.
- The stinger index is the value minus 1, so value 19 gives stinger 18.

### Mote status requests (table 16)

Confidence: High for the request words; Medium-High for which objects send them.

- A request is four whole numbers: status index s, duration d in seconds (-1 = default), amount a and period p in seconds. It is sent to the target's apply-status method after the hit's damage.
- Senders and their words (D, A and T are table 16 fields, the map is the Mote-to-status map of table 16):
  - Tor's power step, for the Mote in weapon slot 2 (hero row from the hero flag): (map(id), D, A, T) of that row and id.
  - Lava (161) power-attack object: fire (7) with D, A and T of the object's hero row. Tor's object always sends it. Roberta's object sends it only when a uniform whole-number draw from {0, 1} gives 1 (chance 1/2).
  - Roberta's Leaf (164) object: poison (2) with D, A and T of row 1.
  - Tor's Shadow (169) object: curse (1) with D(Tor, 169), and amount and period -1.
  - Tor's Sun (171) object: confusion (6) with D(Tor, 171).
  - Water bubble (163), both heroes: the bubble stores D(row, 163) when it starts, and its hit sends stun (0) with that duration.
  - Roberta's full-release shots (SPEC-0036, shot table B): the shot record carries (reaction field, D(1, id), A, T). The duration is read for Motes 163, 167, 168 and 170. Amount and period are read for Mote 160 only and are -1 otherwise. A shot hit sends the stored request and the shot ends.
- Receiver, the enemy class (SPEC-0029): the apply rules are those of the hit-side route of SPEC-0036 (Status apply routes): an immunity test on the target first, then a per-status gate on the monster row, then the defaults when d = -1 (stun 5 s; curse, slow, strength and speed 60 s; poison 120 s; fire 5 s; confusion and poison attack 30 s), and the timer is set to d x 60 ticks. Poison stores a and p as its per-tick amount and period, and fire stores them as its amount and period. The enemy status tick deals a HP every p seconds, with the defaults 2 HP every 4 s (poison) and 2 HP every 2 s (fire) when the stored word is negative.
- The base character class ignores a request (no effect). The hero class has its own apply method (SPEC-0036).

### Id lookup (shared)

- Table 0 and table 2 and table 4: index = id.
- Tables 3 and 6: id <= 200 maps to index = id; id >= 201 maps to index = id - 200. Table 3's entry at index i is paired with table 6's entry at i + 200.
- Out-of-range indices have no record; readers treat them as missing (use defaults or skip).

### Emote choice (tables 7 and 8)

Emotes are requested only by dialogue control codes (SPEC-0022); no other source requests them (High). The request's first argument is the emote id, which is the row number and also the animation id that the actor plays. A character body emote (on the selected hero's talk partner) reads table 7. A hero body emote and a hero face emote (on the selected hero) read table 8. A row with no clip, or one whose chance fails, still plays its animation, silently (SPEC-0009, Emote clip playback).

The lookup steps are defined in SPEC-0009 (Emote clip lookup). In short, for the emote id:

1. Read the row for the emote id from table 7 (character) or table 8 (hero).
2. If the clip index is 255, play no clip.
3. Chance is 0 or 100 in every row. Play the clip when the chance is 100, and play no clip when it is 0.
4. If the variance byte is nonzero, add a random step of -1, 0 or +1 to the clip index. If it is zero, add no step.
5. Character emote: clip = clip index + 74 times the partner's voice set c (villager row +0x14). With c = -1 the position is negative for every clip index below 74 and plays nothing.
6. Hero body emote: clip = clip index + step + 183 when the hero is Roberta (the hero flag is set), and clip index + step otherwise. Hero face emotes take no hero shift for either hero. The hero flag, the body and face split and the shift of 183 are as stated in SPEC-0009 and SPEC-0022.
7. A clip index that gives a directory entry of zero length plays nothing.

### Save state

Runtime fields (table 3 +0x10, table 5 +0x04, table 6 +0x04) are zero in the file. The clean side keeps them in its own save state and starts them at zero.

### Loading order

Chunks are read in file order. The first 17 type-42 chunks are tables 0 to 16. The three type-43 chunks follow; they are loaded into the sound slots listed below. Missing or extra chunks make the resource invalid.

### Path records (type 43)

The loader gives each path record an archive slot. The slot selects the resource; the name in the record is not used to find it.

| Chunk | Slot | Archive resource | Name stored in the record | Used by |
| --- | --- | --- | --- | --- |
| 17 | 0x90 | 144 | characterEmotes.bls | Table 7 character emote clips |
| 18 | 0x91 | 145 | heroEmotes.bls | Table 8 hero emote clips |
| 19 | 0x240 | 576 | BOOKS.bls | Book page sounds |

Coverage (H): each directory covers its whole resource. Chunk 17 covers resource 144 (1,185 entries, 10,056 units). Chunk 18 covers resource 145 (367 entries, 3,327 units). Chunk 19 covers resource 576 (88 entries, 6,688 units).

The stored name has a folder prefix before the file name; that prefix is not used.

### Book index (chunk 19)

A book index i selects directory entry i of chunk 19, which gives the record's start and length in resource 576. Book i starts at the start field of entry i, which is 76 x i units, and has 76 units (H).

- Valid book indices are 0 to 87. Index 87 is directory entry 87, so it is a valid index and selects record 87. On disc, record 87 is byte-identical to record 86.
- The book reader does no upper-bound check on the index. The range is set by the code that supplies the index, not by the directory. An index of 88 or more has no directory entry.
- One book at a time (one-slot cache, High). The reader asks for one book index. A request for the index that is already loaded does nothing: the book is not read again and its page timer is not reset. A request for another index discards the current book and starts a new load, and only a new load sets the page timer to its start value (SPEC-0035, section 6.1).
- Disc availability and modeled reach are separate. The disc has 134 qualifying book stations in 27 world packs (kind-3 cylinder volumes with flag word 0x0040; SPEC-0028, section 13). The 27 pack ids all occur in SPEC-0006’s 174-pack static new-game reach closure, so the graph admits a station for each of the 86 distinct book indices found on disc. This is an upper bound, not proof that every station is traversable in ordinary play: the area graph does not evaluate story gates or actual ground-level walkability. Station values are 1 to 77 and 79 to 87, inclusive; the reader uses value minus 1. Thus index 77 has no station, and index 87 has no station even though record 87 exists and duplicates record 86. [High for station data and static-closure intersection; Medium-High for actual play reach] Per pack there are 1 to 17 stations (pack 777: 17; pack 1414: 14; pack 1327: 13). Synthetic check: pack 786 has a station value of 87, which selects record index 86; value 78 would select index 77, but no such station value occurs.

## Edge cases and error handling

- Species or villager table with a count other than the expected one: the table is rejected, the build reports a changed table size, and no partial table is used (Medium).

- Pointer 0 means no text. Zero-length strings are not expected in the pool.
- Table 15: blank spacer rows have a zero pointer and must still take up their line space.
- Table 3 gate -1 rows are skipped by scans.
- Table 6 location -1 means "not in any location".
- Table 8 and table 7 index 255 means "no sound".
- Table 8 with a clip index of 178, a step of +1 and Roberta gives position 362. That position exists and has nonzero length. It is the largest position any table 8 row reaches.
- Strings with control bytes must be shown or stored as raw bytes; do not trim them. The two 0xAE bytes of table 9 record 25 are not control bytes: they are glyphs of font 0. Book strings also hold newlines and a few bytes with no glyph (Book records, Glyph coverage); keep them.
- Table 2 row 0 holds value -1. A rising edge of flag 0 would request stinger -2, which passes the stinger routine's bound test and reads before the chooser arrays (quirk). Flag 0 has no setter, so this is unreachable. A clean build should ignore negative stinger indices.
- Table 2 has rows for flags 0 to 731 only. The bitmap allows ids up to 742, and the cue step does not bound the table read, so ids 732 to 742 would read the bytes after table 2 (the next chunk's header; for example id 733 gives kind 1424 and v 1, which would play stinger 0 and light the journal icon). No script or game setter uses an id above 731. A clean build should treat ids 732 to 742 as kind 0, v 0, and may log them.
- Table 4 record 193 (the last of 194) is never read, because the getters accept ids 0 to 192 only. It holds an ordinary-looking track row (stream 27, bank 20, block -1). A clean build must also return the defaults for id 193.
- Table 13: the accepted word is not cleared when a list entry fires. A later Square edge in another list entry can fire that entry before the chooser runs in that tick (Medium-Low).
- Table 16 Rock quirk: Mote 165 has no status fields, so its poison is the receiver's default (Table 16).
- Table 14 classes 19 to 65 contain two defects. They are not reachable, because the stinger routine rejects indices of 19 or more. (a) The chooser reads only the first record's weight of a class and never moves to the later records; for class 63 (two records of weight 50) it returns the first record for every roll from 0 to 100. (b) The start record of each class is the running count from class 1, not from class 0. That is correct for classes 0 to 18 only.
- Zero-length directory entries play nothing. Chunk 17 has 189 of them; chunk 18 has 9, at positions 9 to 11, 41 to 43 and 103 to 105.
- Table 2 journal flag: set by a cue with a non-zero kind, cleared only by building the quest journal list (Progress-flag cues).

## Test vectors

- Header arithmetic (synthetic): N = 3 gives an index array of 12 bytes after the 4-byte count, so the rows start at the next multiple of 16, which is 16 (0x10). For N = 112 the index array is 448 bytes and the rows start at 464 (0x1D0), as on disc.

Synthetic, not copied from disc:

- Id lookup: id 5 -> table 3 index 5. Id 201 -> table 3 index 1. Id 200 -> index 200 (out of range for table 3, so no record).
- Pointer: table block at file offset 0x1000, pointer 0x0200 -> string at 0x1200. Pointer 0 -> no string.
- Stinger identity (table 14): stinger index 7 selects record 7, which has weight 100, and plays clip number 263 (256 + 7).
- Flag cue (table 2, synthetic): a flag with value 6 and kind 0 rising plays stinger 5 once and does not light the journal icon. A flag with value 0 and kind 2 rising plays nothing and lights the icon.

From the disc:

- Flag cues (table 2): flag 110 (kind 1, v 18) rising plays stinger 17 and lights the journal icon. Flag 273 (kind 0, v 2): stinger 1, no icon. Flag 707 (kind 0, v 0): only the bit is set (new game sets it). Flag 107 (kind 2, v 17) set twice: the cue plays once.
- Stinger chooser (table 14): request 42 (Tor item showcase) plays nothing. Request 18 (table 2 value 19) plays clip 274.
- Table 16 fields: P(Tor, 160) = index 0 = 1.0; P(Roberta, 160) = index 30 = 1.25. D(Tor, 161) = index 5 = 12; A(Tor, 161) = index 6 = 5; T(Tor, 161) = index 7 = 2. D(Roberta, 164) = index 42 = 34; A(Roberta, 164) = index 43 = 7; T(Roberta, 164) = index 44 = 4; for Tor the same Mote gives 34, 5 and 4. D(Tor, 163) = D(Roberta, 163) = 8. D(Tor, 167) = index 19 = -1 (default); D(Roberta, 167) = index 49 = 30; D(Tor, 166) = 30; D(Roberta, 166) = index 47 = -1. D(Roberta, 169) = -1 and D(Roberta, 171) = -1; D(Tor, 169) = D(Tor, 171) = 30. D(any, 165) = -1 (no field).
- Mote status request: Tor's power step with the Lava Mote (161) on an enemy sends (7, 12, 5, 2). The enemy burns for 720 ticks and takes 5 HP each time the fire timer is a multiple of 120 ticks.
- Rock quirk: Tor's power step with the Rock Mote (165) sends (2, -1, -1, -1); the enemy is poisoned for the default 120 s with the default 2 HP every 4 s. The Leaf Mote (164) sends (2, 34, 5, 4).
- Start-point grant (tables 11 and 12): start index 2 (stages 0 and 1) adds 1,000 NP to the wallet and items 100, 107, 121, 112, 128 and 176, plus stage 0's Motes and consumables (carry limits apply), and its stage flags play no stinger.
- Path directory (synthetic): N = 3 at +0x100, then entries (start 0, length 4), (start 4, length 2) and (start 6, length 0) from +0x104. Entry 0 covers units [0, 4), entry 1 covers units [4, 6), and entry 2 covers no units. The total is 6 units, which is 12,288 bytes at 2,048 bytes per unit.
- Emote chance: chance 100 always plays; chance 0 never plays. Chance is only ever 0 or 100 on disc.
- Emote index 255: nothing plays, whatever the chance.
- Character emote: row clip index 5 with character index 2 gives clip 153 (5 + 74 x 2).
- Hero body emote: row clip index 10 with step -1 and Roberta gives position 192 (10 - 1 + 183). The same row with Tor and no step gives position 10.
- Hero face emote: the same row with step -1 and Roberta gives position 9 (no shift).
- Voice set (from the disc): a placement of species 33 (Hubert) reads a villager row with c = 9, so a kind 0x10 request with emote id 51 (clip index 36) gives position 36 + 74 x 9 = 702. An enemy placement has c = -1 and gives -38, which plays nothing.
- Table 6 count: a location with 3 entries of which 2 have the flag set -> count 2.
- Table 6 writes (synthetic): a write of 1 to row 0 or to row 248 changes nothing; a write to row 213 sets it. A new game then clears rows 0 to 247, so the location count of row 213's location falls by 1.
- Resource tails (from the disc): 0x1D0 + 259 x 0x504 = 333,020 and 333,824 - 333,020 = 804; 0x1D0 + 272 x 0x60 = 26,576 and 26,624 - 26,576 = 48. Both file sizes divide by 2,048.
- Boot load (synthetic): a resource 530 edited to 300 whole rows loads without a message; the block count stays word 0 of the file (112).

## Open questions

Numbers of removed questions are not reused.

2. **Table 0 +0x1E/+0x1F.** Resolved in part: +0x1F is the important-item flag (SPEC-0021). The flag-gated item-text getter returns +0x1E as a signed byte (disc values 0 to 4), but the item-showcase caller does not use that output; its showcase and world-item callback decisions use the returned text, item id, quiet flag and hero state. The byte's semantic meaning and any other readers remain unknown. Answered: +0x08, +0x1C, +0x24, +0x28, +0x2C and +0x30 are under Table 0 and Item text by flag. **[High for caller non-use; Low for field meaning]**
3. Resolved: table 1 +0x04 and +0x08 are the quest start and done flags, progress-flag ids (SPEC-0049).
4. Resolved: every design-table control byte belongs to a well-formed SPEC-0022 control code, except the two 0xAE glyph bytes of table 9 record 25 (Strings). **[High]**
6. Resolved: table 16 holds four fields per hero and Mote: power factor P and the status duration D, amount A and period T (Table 16; Mote status requests).
9. Resolved: table 11 sets progress flags with the plain set (no stinger, no journal cue, no notification); table 12 grants items through the add-item action, with item 135 adding Neopoints (Tables 11 and 12).
11. Resolved: table 0 +0x0C is the catalogue price read by the enemy item-steal path; the shop does not read it. A shop item's price is its placement's fixed count (+0x2A), copied to the live world-item price field (+0x8A) and optionally changed by script price setters/multipliers. The purchase compares this single-item price against the wallet and subtracts it only after item addition succeeds; there is no separate quantity total. See SPEC-0021, Shop purchase rule. **[High]**
12. **Resource 533 language scope.** Resolved: its 17 design-table chunks expose fixed record fields and table-local string pointers, with no language-index field or alternate-language chunk among the 20 top-level chunks. This establishes only the resource-533 format; whether another resource or system supplies a second language remains open. **[High for resource scope; Low for game-wide absence]**
13. Resolved: the flag is set only by the three treasure-group callbacks, rewritten from the save set on load, and cleared for all 248 rows at once by the new-game world reset (Table 6). **[High]**
14. Resolved for the type-43 path-record payloads: resources 144 and 145 are the character and hero emote banks, and both use the shared emote clip playback path, which refuses a directory position whose length is zero (SPEC-0009, Emote clip playback). Resource 576 is book-record data, not an audio bank. This does not generalize to unrelated directory consumers elsewhere in the program.
16. Resolved: both tails are fill up to the next 2,048-byte boundary (804 bytes in 530, 48 in 531). They hold no second table and no lookup reaches them (Species and villager tables). **[High]**
17. Resolved: the boot load of resources 530 and 531 has no count, stride or length check. The only size-checked load routine is never called (Load check). **[Medium-High]**
18. **Species name.** The table of resource 530 is called a species table in the loader's size message and a monster table in the enemy specs. Which name to use is not decided here. **[Low]**
19. Resolved: the stinger 4 cooldown is 40 s of game clock time (Progress-flag cues).
23. Resolved for the type-43 path-record consumers: one unit is 2,048 bytes (one sector). Emote playback uses the entry length as a sector count and passes it to the stream path; the book reader converts the sector length to bytes by multiplying by 2,048. Both select directory entries by position, but their read paths consume the shared unit in different forms (SPEC-0009, Emote clip playback; Book index; High). This rule is limited to the known type-43 consumers.
25. The text chunk words at +0x08 and +0x0C, the paragraph layout, and the exact style value behind the style index. Answered: the layout chunk is a font descriptor followed by its glyph record array (Book records).
26. Whether the book reader gives the single "{" and "}" any meaning (in the text engine they draw nothing). Answered: the rest of the glyph coverage is in Book records, Glyph coverage.
27. The prompt chooser runs during the hero update, before the end-of-update reset. On the known frame-end contact path, the type-40 callback writes the label after both; the chooser can therefore read and submit it on the next eligible update, but not on the contact tick. The shared contact helper is also called by motion-record, teleport/relocation, pose-change and scripted-transition paths; their order relative to the chooser is not established, so a universal same-tick visibility rule remains unresolved. **[High for the frame-end path; Medium for the remaining boundary]**
28. Resolved: status timing is path-specific. Tor's power-step sends its status request before the subsequent damage operation. The Tor and Roberta Lava objects, Roberta Leaf, Tor Shadow and Tor Sun call the target's take-hit method before sending their status request. Roberta's full-release shot path also deals damage before sending its stored request. Tor Water (163) uses a type-4 family-object callback that sends stun but makes no damage call, so that callback has no damage/status ordering. Its target walk invokes the callback after collision and hit-list checks. The enemy status receiver returns for a dead or dying target before element immunity (SPEC-0029, Behavior 27.1). See SPEC-0036, Sources of statuses on an enemy from the heroes; Status-hit order. **[High for the listed call orders and absence of damage in the Water callback; Medium-High for mapping that callback to Water 163]**
29. Resolved for the registered argument-record caller: it is the dialogue control command kind 0x19 (“Apply stage”), which passes its first argument as the stage id and requests notification. It is bound in the dialogue control table; no kind-0x19 record occurs in the shipped dialogue line tables, so the handler is present but no invocation from those records is established (SPEC-0022; High for the binding and disc-record census). The concrete runtime object class name remains unidentified and is not needed to implement the table behavior.
30. Labels 29 and 30 are used by the sealed-chest context prompt: label 29 on the element-mismatch branch and 30 on the matching/zero-element branch. The disc contains 39 such chest groups. No path was established for labels 7, 15 or 35; because computed and indirect values are not fully excluded, they remain unresolved rather than unused. **[High for the two chest-label branches; Medium for the unresolved absence boundary]**
31. Non-townsfolk partners are mechanically allowed: the partner setter accepts an entity without a townsfolk/class predicate, and an eligible lock-on leader can supply the talk candidate when no strictly nearer townsfolk candidate wins (SPEC-0022). Voice lookup still uses the spawned character's villager row, selected by placement species and variant; a row voice index of -1 gives no character-bank clip. Queen Fyora provides a shipped class-38 proximity example, but its registration is not yet tied to a partner-assignment call, so ordinary-play non-town assignment remains unproven. **[High for the generic assignment and voice lookup; Medium for the shipped example boundary]**
