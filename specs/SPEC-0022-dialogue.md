# SPEC-0022: Dialogue, text and progress flags

- Answers: proactive
- Status: Draft
- Disc build: SCUS-97367 v1.00 (same fingerprint as SPEC-0001). Line tables are read from world-pack resources in the archive; tuning words are read from the boot program's data by virtual address.
- Confidence: per section. Line use: High for the counts, Medium for their meaning. Line table layout and control-code layout: High (true of every line table on the disc). Shared text engine (glyph records, break search, reveal arithmetic, rounding): High. Fonts (descriptors, glyph records, metrics, composite letters): High. Justification rule: High. Dialogue box layout: High for the formulas, Medium-High for the canvas units. Speaker and choice rules: High (partner block Medium). Memory-card and controller messages: High. Reveal rate, paging and release timing: High. Dialogue command kinds (flags, items, stage): High. Progress flag semantics: High for storage and set/clear/test, High for the stage table and the initial state, Medium for the event-to-flag list. Quest rows (journal entries, Quests below): Medium-High for the fields, Medium for their use, Low-Medium for the listing gate. Which script line is chosen: Medium (the boundary described in Open questions). Text window draw layout: High for the structure, Medium for the cue meanings. Interaction prompts (Square caption, prompt ids, gates, chooser order) and the talk candidate and talk entry: High; outcomes on disc of one press acting in several places: Medium-High. The leap branch tests no button: High. The Action button is Square (logical bit 0x0080, press edge): High. Conversation calls and level lines (LevelMsg, LevelPrompt): High; the hero state between two script lines: Medium. Dialogue control records (kinds, flag rule, counts): High for the layout, the kinds, the run-pass mask and the counts; kind 0x16 High (its label Medium-High); who each emote kind acts on High; the stale-emote quirk Medium; the item-notice setting of kind 0x13 Medium. No spoken dialogue and no run-time lip sync: High (the meaning of face id 0 Medium). Line alignment, the case-insensitive compare, the page stop search and substitution codes: High; the same-text reopen of the box: Medium.

## Summary

The game shows dialogue through one shared text box. A line is a record in a per-world table: a speaker name, a text, and a colour. Text is single-byte with embedded control codes, the most important of which is a page stop. All dialogue, label and HUD text is drawn by one shared text engine: it lays out lines against a wrap width, breaks them at spaces and underscores, and draws each glyph from one of three fonts held in the front-end pack (archive resource 532). The box reveals text at 20 characters per second (five times faster while Cross is held), pages at stops, and takes a Square press to continue. It can also show two labelled choices, answered by Circle (result 1) or Triangle (result 0). Memory-card messages and the controller message reuse the same box. The engine only displays a line when a level script asks for it by index or by a raw string. The script, not the engine, decides which line to show, using the progress flags. Progress flags are one bitmap of 743 bits, set and cleared by scripts, read by scripts and the engine, and watched by registered callbacks. The bitmap starts with every flag off. Talking to a character is started by a Square press while the talk candidate (the leader of the hero's talker list: within 20 units and in front of the hero) is within 5.8 units; the press starts the level script's talk callback. One Square caption is shown at a time, chosen each tick by a fixed-order interaction chooser. Lines may also carry inline control records that change the text colour, request emotes, switch the speaker to the hero and back, set or clear progress flags, give items or apply a stage when the draw or the reveal reaches them (see Dialogue control records). All timings run on the 60 Hz game tick (SPEC-0007).

## Where it lives on the disc

- Dialogue lines: an IDMX table (chunk type 1000) at the top level of a world-pack resource. 131 of the 192 world-pack resources contain one. Table sizes run from 1 to 1,043 records (counts vary by table; none is a fixed size).
- Layout constants of the dialogue box: six 32-bit words in the boot program's data, at virtual addresses 0x2DCC18, 0x2DCC1C, 0x2DCC20, 0x2DCC24, 0x2DCC44 and 0x2DCC48 (file image; read as little-endian 32-bit integers). Values are in the Layout section.
- Glyph constants of the text engine: 32-bit words in the boot program's data at virtual addresses 0x2EC8F0, 0x2EC90C and 0x2EC910, and a 0.5 rounding constant at 0x2DD7D8 (see Layout, Box geometry constants).
- Character-class table used by the case-insensitive compare: 256 bytes of the C library's class table in the boot program's data, indexed from 0x329FC1 (see Text comparison and page stop search).
- Progress-flag default table: 743 32-bit entries at virtual address 0x0032C2A0. This table lies after the end of the file-backed part of the boot program's data, so it is zero-initialised memory, not file data. There are no default flag values on the disc (see Initial state).
- Fonts: the front-end pack, archive resource 532. Its type-38 chunk payload (the front-end database) holds the font count and the font table: three font descriptors of 0x110 bytes, followed by their glyph records. The glyph sheets are 4-bit sub-textures of the same resource's texture chunk (see Fonts).
- Canvas and box sprite words: 32-bit words in the boot program's data at 0x2EC900 (float 1.0, UI y scale), 0x2EC904 (448), 0x2EC908 (512), 0x2EC91C, 0x2EC920, 0x2EC924, 0x2EC928 (box border sizes 15, 28, 14, 26), 0x2EC92C and 0x2EC930 (box sprite size 64 and 64), and 0x2DCC40 (two 16-bit halves, 128 and 384: the prompt label x positions). Text colours: four bytes each at 0x3159F0 (speaker name), 0x3159F8 (body base colour) and 0x315A38 (prompt labels). See Dialogue box layout.
- Accent tables: the 40 accented Latin-1 codes at data address 0x321528, the four accent codes at 0x321550 and the ten vowels at 0x321558 (see Fonts and Accent rule).
- Message, label, hero colour and stage tables: design tables of archive resource 533 (SPEC-0024): table 9 is the system message table (memory-card and controller messages), table 10 the hero colours, tables 11 and 12 the stage table pairs and triples, table 13 the UI label table.
- Progress-flag bitmap and dialogue state: runtime memory only, not on the disc. The clean side keeps its own state.
- The 5.8-unit talk range, the talker-list range 20.0, its facing cosine 0.5 and its 90-tick leader hold are immediate constants in the code, not data words. The lock-on list words are at 0x2DD6A8 to 0x2DD6B8 (SPEC-0036). The Square caption buffer is run-time memory at 0x37C060, its shown count is the word at 0x2DCD40, and the six HUD prompt slots' caption and count pointer tables are at 0x315A98 and 0x315AB0 (see Interaction prompts).
- Quest rows (journal entries): table index 1 of the item-table file (archive resource 533, an IDMX table of 238 rows of 36 bytes). The layout is in Quests below. The journal and map rules that use them are in SPEC-0035 section 14.

## Layout

### Dialogue line table (chunk type 1000)

The table is a child of the chunk, starting at chunk start + 16 + the chunk's extra field. Call this position the table base. All offsets below are relative to the table base.

| offset | size | type | meaning |
| --- | --- | --- | --- |
| +0x00 | 4 | ASCII | magic "IDMX" (true on all 131 tables) |
| +0x04 | 4 | u32 | 0x4C on all 131 tables on disc; meaning not confirmed |
| +0x0C | 2 | u16 LE | record count N (1 to 1,043) |
| +0x0E | 2 | u16 LE | record stride, 12 on all 131 tables |
| +0x10 | 12 × N | records | dialogue line records |

Record (12 bytes, little-endian):

| offset | size | type | meaning |
| --- | --- | --- | --- |
| +0 | 4 | u32 | speaker-name offset, relative to the table base. Non-zero in all 7,547 records on disc |
| +4 | 4 | u32 | text offset, relative to the table base. Non-zero in all 7,547 records on disc |
| +8 | 4 | u32 | colour word. Bytes 0, 1, 2 are red, green and blue (0 to 255). Byte 3 is not used by the text box (see Behavior) |

Strings are NUL-terminated. Both offsets point at strings inside the same chunk (in the sample, after the record array). On disc, all 7,547 records across the 131 tables resolve to terminated strings. Checked on disc: 7,547 records, stride 12 on every table, and no record with a zero speaker or text offset. The record has no emotion, expression or voice field: speaker, text and colour are its only fields. Emote requests are inline control codes in the text (see Dialogue control records), not fields of the record.

### Box geometry constants

Read from the boot program's data (32-bit integers). Units are pixels of the 512 by 448 UI canvas (Medium-High; see Dialogue box layout).

| virtual address | value | use |
| --- | --- | --- |
| 0x2DCC18 | 20 | frame margin: frame x = anchor − 20; frame y = base line − 20 − line height; also a term of the frame width and of the name baseline |
| 0x2DCC1C | 16 | frame width term (width = wrap + 20 + 16) |
| 0x2DCC20 | 20 | frame height term |
| 0x2DCC24 | 4 | frame height term |
| 0x2DCC40 | 128 and 384 (16-bit halves) | x positions of the left (Triangle) and right (Circle or continue) prompt labels |
| 0x2DCC44 | 410 | text wrap width given to the line counter; also a term in the box width; also the x position of the price caption (SPEC-0035 section 15) |
| 0x2DCC48 | 50 | box x anchor; the body text is drawn at this x; the speaker name at this x plus 20 |

Text engine constants (read from the boot program's data):

| virtual address | value | use |
| --- | --- | --- |
| 0x2EC8F0 | 1 (32-bit integer) | one-shot palette-load flag. The first front-end textured draw after boot (text or sprite) sets up its texture with the palette (CLUT) load forced to happen at once, then sets the word to 0. Nothing sets it again. Otherwise the font templates load the palette only when it changes. It has no visible effect, and a clean renderer may ignore it (High) |
| 0x2EC90C | 0.0 (float) | per-space extra spacing factor for justification (see Text layout and glyph drawing, rule 6) |
| 0x2EC910 | 1.5 (float) | per-glyph spacing cap for justification, in units |
| 0x2DD7D8 | 0.5 (float) | rounding offset added before truncation when advances are rounded (see Text layout and glyph drawing, rule 2) |

Speaker names on disc: 120 records carry the placeholder name "Hero" exactly; no other casing of it occurs, and "[Hero]" does not occur. 65 records carry a speaker name that is a single control code (type 32, no arguments). The rule for these is in Behavior.

Colour bytes on disc: byte 3 is 0x80 in 528 records and 0 in the rest. The text box does not depend on byte 3 (Medium).

### Text encoding

Text is single-byte. Outside control codes, all bytes are printable ASCII. On disc no byte of 0x80 or above and no escape byte appears outside a control code (all 7,547 texts on disc).

A control code is a run of bytes that starts with the marker 0x01:

| offset from marker | size | meaning |
| --- | --- | --- |
| +0 | 1 | marker 0x01 |
| +1 | 1 | n. The code carries n − 2 argument groups |
| +2 | 1 | L. Total length of the code in bytes, marker included. The next byte after the code is at +L |
| +3 | 1 | type T |
| +4 | 1 | value V. V − 2 is a run-pass mask: bit 0 = the code runs each time the drawing reaches it; bit 1 = it runs when the reveal reaches it. Type 4 is handled as the stop whatever its mask |
| +5 | L − 5 | argument bytes, encoded (see escape rule) |

Escape rule for argument bytes: a byte 0xFD followed by a byte b in the range 0x40 to 0x60 stands for the single value b − 0x40 (so 0 to 0x20). 0xFD followed by 0x02 stands for the byte value 0xFD itself. 0xFD followed by any other byte gives 0 and is reported as an error. Any other argument byte stands for itself. No 0xFD 0x02 pair occurs in codes on disc. A code whose length runs past the end of its string ends the text. Decoding the argument bytes must give exactly 4 × (n − 2) bytes. This held for all 18,252 codes on disc.

Observed codes on disc (counts over all line tables):

| type T | codes | shape | notes |
| --- | --- | --- | --- |
| 4 | 2,538 | n = 2, L = 5, V = 6 | page stop (no arguments); V is 6 on all of them |
| 2 | 4,687 | n = 3, V = 3; L = 9 (3,532), 10 (759) or 11 (396) | text colour: the argument group is R, G, B, A; draw pass (mask 1) |
| 16 | 8,998 | n = 3, V = 4; L = 12 (6,917) or 13 (2,081) | kind 0x10, character body emote; reveal pass; most common |
| 23 | 923 | n = 3, V = 4; L = 9 (6), 12 (907) or 13 (10) | kind 0x17, hero body emote; reveal pass; the L = 9 codes are also present (see below) |
| 24 | 293 | n = 3, V = 4; L = 9 (3) or 12 (290) | kind 0x18, hero face emote; reveal pass; the L = 9 codes are also present (see below) |
| 32 | 434 | n = 2, L = 5, V = 4 | speaker override to the active hero; reveal pass; also used as the speaker-name case |
| 34 | 191 | n = 2, L = 5, V = 4 | speaker override clear; reveal pass |
| 19 | 166 | n = 4, V = 20; L = 19 (63), 20 (88) or 21 (15) | kind 0x13, give items (item, count); reveal pass |
| 21 | 7 | n = 3, L = 13, V = 4 | kind 0x15, no effect (a cut dialogue-camera command); all seven in one line of pack 772 |
| 22 | 10 | n = 3, L = 12, V = 4 | kind 0x16, partner face emote with no clip; reveal pass |
| 33 | 4 | n = 2, L = 5, V = 6 | mask 4: never run |
| 17 | 1 | n = 4, L = 16, V = 12 | kind 0x11, set progress flags; reveal pass; one code on disc (pack 772, flag 416) |

Type 4 is the page stop. The meanings of the other types are in the table above and in Dialogue control records. Every registered handler has width 0: it adds nothing to a line's measured width. A type with no handler does nothing.

Types 23 and 24 also occur with L = 9 (six and three codes on disc); a reading that lists only L = 12 or 13 for them is incomplete.

Type 5 (n = 3, L = 13, V = 34) occurs in no line table. It is the substitution code of the system messages, design table 9 of SPEC-0024 (19 codes; see Substitution codes).

### Fonts

Confidence: High unless marked.

Fonts live in the front-end pack, archive resource 532. Its type-38 chunk payload (the front-end database) holds a font count (u32 at payload +0x44; 3 on disc) and the font table (u32 payload-relative offset at payload +0x90). The table is three font descriptors of 0x110 bytes each; their glyph records follow the three descriptors. The book layout chunk (type 2001, SPEC-0024) uses the same descriptor format.

Font descriptor (0x110 bytes, little-endian):

| offset | size | type | meaning |
| --- | --- | --- | --- |
| +0x00 | 4 | u32 | on disc: index of the font's texture record in the database (records of 0x88 bytes in the texture-record section named by the database header word at payload +0x8C). At load it is replaced by the record's address. The record's first 8 bytes are a GS texture template for the glyph sheet. Disc values 4, 31 and 146 |
| +0x04 | 1 | u8 | number of glyph records N (102, 89 and 91 on disc) |
| +0x05 | 1 | u8 | mode: 0 = codes from the first to the last code; 1 = as 0, with a to z folded to A to Z first; 2 = codes from the first code for N codes; 4 = code map. All three fonts on disc use mode 4 |
| +0x06 | 1 | u8 | first code (32 on disc) |
| +0x07 | 1 | u8 | last code (241 on disc) |
| +0x08 | 4 | u32 | 0 on disc; not read by the text paths |
| +0x0C | 256 | u8[256] | mode 4: glyph number for each code; 0 = no glyph; a value v gives glyph index v − 1 |
| +0x10C | 4 | u32 | on disc: offset of the glyph records from the database start; at load, their address |

Glyph record (18 bytes, little-endian), at the glyph array + 18 × index:

| offset | size | type | meaning |
| --- | --- | --- | --- |
| +0x00 | 2 | s16 | left texel bound of the glyph in its sheet, in 1/16 texel |
| +0x02 | 2 | s16 | top texel bound, in 1/16 texel |
| +0x04 | 2 | s16 | right texel bound, in 1/16 texel |
| +0x06 | 2 | s16 | bottom texel bound, in 1/16 texel |
| +0x08 | 2 | s16 | quad width in canvas pixels (on disc: the texel width rounded down, plus 1) |
| +0x0A | 2 | s16 | quad height in canvas pixels (on disc: the texel height) |
| +0x0C | 2 | s16 | advance in pixels |
| +0x0E | 1 | s8 | x bearing: quad left edge relative to the pen (−1 to 2 on disc) |
| +0x0F | 1 | s8 | descent: quad bottom edge relative to the baseline, positive = below (−3 to 7 on disc) |
| +0x10 | 1 | s8 | accent anchor x for composite letters (0 on all records) |
| +0x11 | 1 | s8 | accent anchor y (0 on all records) |

On disc, left < right and top < bottom hold on all 282 font glyph records and on the 91 glyph records of the book layout chunk.

Font metrics: the line height of a font is the height field (+0x0A) of its glyph record 1. The space advance is the advance field (+0x0C) of its glyph record 0. There is no kerning: the pen moves by each glyph's own advance.

| font id | glyphs | glyph sheet (sub-texture of the resource 532 texture chunk) | line height | space advance | use |
| --- | --- | --- | --- | --- | --- |
| 0 | 102 | sub-texture 110: 4-bit indexed, 256 × 256, 16-colour palette | 21 | 10 | dialogue body text and speaker name |
| 1 | 89 | sub-texture 87: 4-bit, 256 × 128 | 17 | 6 | dialogue prompt labels |
| 2 | 91 | sub-texture 116: 4-bit, 256 × 128 | 14 | 5 | in-play journal quest-objective text; the book uses a separate type-2001 descriptor and glyph records with the same code map (SPEC-0024) |

The sheet of a font is the sub-texture whose buffer base equals the base in the font's texture template (10,752, 11,136 and 11,072 for fonts 0, 1 and 2). The glyph art carries its own drop shadow, and the palette holds greys with alpha (Medium for the look).

Runtime use of font 2 in the in-play journal follows the objective label binding's font id through the journal row drawer to the shared text renderer. That renderer selects the resource-532 font-2 descriptor and its glyph sheet (sub-texture 116). The book reader uses its own type-2001 glyph records and book texture template; its descriptor shares font 2's code map, but it does not use the resource-532 font-2 glyph records or sheet (SPEC-0024, Book records). The complete census of every front-end screen that binds font 1 or font 2 remains open.

Button and currency glyphs of font 0: "@" is the Triangle icon, "#" Circle, "^" Cross, "~" Square, "_" SELECT, "=" START and "$" the NeoPoints logo. Fonts 1 and 2 have no glyph for these codes. Dialogue uses them as button prompts and as the currency mark (on disc: "$" 98 times, "#" 6, "_" 3, "~" 1, "^" 1).

Composite (accented) letters, mode 4: a map value v with v − 1 ≥ N names composite k = v − 1 − N (0 to 39). Its base glyph is the glyph of the vowel at position k mod 10 of the sequence A E I O U a e i o u (data address 0x321558). Its accent is the glyph of accent code number k div 10 of the four accent codes at data address 0x321550 (0x88 circumflex, 0xB4 acute, 0x60 grave, 0xA8 diaeresis). The 40 accented Latin-1 codes are at data address 0x321528, in the same order (accent-major, vowel-minor). At load, for each k whose accented code has map value 0 while its vowel and its accent both have glyphs, the map value is set to N + k + 1. On disc every font already carries all 40 values (highest map values 142, 129 and 131, which are N + 40). Drawing a composite draws the base glyph, then the accent glyph at the same pen, moved by (base anchor x minus half the accent's advance, base anchor y). The pen advances by the base glyph's advance. Measuring uses the base glyph only. Because all anchors are zero, an accent is drawn centred on the pen's left position at baseline height (Medium; no dialogue text on disc uses a code of 0x80 or above, so this is never seen in dialogue).

Lookup rule (all modes): a code is drawable only when it has a glyph. Mode 4 uses the code map. Modes 0 and 2 use the code minus the first code, inside their range. Mode 1 folds a to z to A to Z first. In mode 2 a space also adds the space advance on top of its own glyph advance. A code that is not drawable has no width and draws nothing.

## Behavior

### Text layout and glyph drawing

Confidence: High unless marked.

1. Glyph lookup: as in Fonts (Lookup rule). A code that is not drawable has zero width and draws nothing.
2. Advances are rounded to whole units: add 0.5 with the sign of the advance, then truncate. (So 2.4 becomes 2, 2.5 becomes 3, and −2.5 becomes −3.)
3. Break search (the only one in use; it serves the line counter, the reveal draw, labels and books alike): from the line start, each drawable glyph adds its advance times the transform's x scale, rounded as in rule 2, to a 16-bit running width. A space (0x20) or an underscore (0x5F) is a break candidate; at a candidate the width so far is remembered. A newline (0x0A) ends the line; the newline is consumed and the line is not stretched. A type-4 stop, the end of the text or a truncated control code ends the text. Other control codes add their handler's width, which is 0 for every handler on this build.
4. When the running width exceeds the wrap width (strictly greater): if a candidate was seen, the line ends at the last candidate; the candidate byte is consumed with the line, and the line width is the remembered width. If no candidate was seen, the line ends just after the glyph that overflowed, so the line is wider than the wrap width by up to one glyph.
5. The line text that is drawn includes the consumed candidate byte. A space draws nothing. An underscore at a break draws the SELECT glyph of font 0 at the end of the line, outside the measured width (Medium-High; on disc the 3 underscores sit inside leftover markup, see Edge cases).
6. Stretch (justification): only a line that ended at a candidate and has at least one more candidate before it is stretched. Leftover L = wrap width − line width; n = the number of bytes before the break byte, control-code bytes included. Each glyph's advance is raised by min(L / n, 1.5) pixels (the cap 1.5 is the data word at 0x2EC910). The remainder would go to spaces, capped at the space advance times the factor at 0x2EC90C, which is 0.0, so spaces get nothing extra. Last lines, lines ended by a newline and lines with only one candidate are not stretched. The body text is not centred.
7. Drawing places glyphs at unrounded float pen positions: pen += (advance + spacing × s) × s, where s is the current x scale, so the letter spacing is multiplied by the scale twice (Medium-High). With s = 1, which applies to all dialogue and to 110 of the 112 front-end labels, the step is advance + spacing. After a space, the per-space extra (0 on this build) is added unscaled. Canvas positions are converted to 1/16-pixel GS coordinates and rounded half away from zero.
8. Glyph quad: left = pen + x bearing; right = left + quad width × s; bottom = baseline + descent; top = bottom − quad height × s. The texture coordinates are the four texel bounds (in 1/16 texel). The quad size comes from the width and height fields, not from the bounds. s = 1 except for the reveal frontier glyph (Reveal and paging, item 4).
9. Measuring (used by aligned lines and labels) indexes the mode-4 map with the code read as a signed byte, so a code of 0x80 or above would read 128 bytes before the map (quirk; Medium-High; no dialogue code is in that range). The draw and the break search read the map with the code unsigned, so such a code measures as a wrong glyph but draws correctly. Font 0 has glyphs for 58 codes of 0x80 or above.
10. Label and HUD text use the same engine, with their own font ids. Front-end label text (node kind 2) gets its colour, letter spacing and font from its label binding (SPEC-0035, Section G).
11. Newline in drawing: a newline (0x0A) sets the pen x back to the line start and moves the baseline down by the font's line height.
12. Colour bytes: the colour that starts a draw, and each type-2 colour, is four unsigned bytes R, G, B, A, written as 0 to 255 into the GS packed colour (128 = neutral). A value computed above 255 has already wrapped to its low 8 bits when it is stored (SPEC-0035, Section G).

### Line alignment

Confidence: High unless marked.

Line draw:

1. **Empty line.** A line with no text draws nothing and returns 0.
2. **Metrics.** For alignment modes 1, 2 and 3 the line is measured first, with scale 1 and the caller's letter spacing. The measure gives a width W and a height Ht, both whole numbers cut toward zero to 16 bits:
   - W is the largest pen x reached. Each drawable glyph adds advance × s + trunc(spacing × s) × s, where s is the scale (1 here). A newline returns the pen to 0. A control code adds its handler width (always 0). A type-4 stop or a truncated code ends the measure.
   - Ht = (number of newlines × the font's line height) + the quad height (+0x0A) of the last glyph measured.
   - Other modes measure nothing (W = Ht = 0).
3. **Alignment.** The mode is set by the caller.
   - **Mode 1, centre:** x = x − W/2, with W/2 rounded toward zero (width 33 gives 16; width 34 gives 17).
   - **Mode 2, right:** x = x − W.
   - **Mode 3, raised:** x is unchanged and y = y − Ht, so the text is lifted by its own measured height. It does not use the advance of the last glyph.
   - Other modes: no shift.
4. **Floor.** After step 3, a start x below 16 becomes 16, in every mode, including mode 0. It is a left-margin clamp on the start x, not a width rule.
5. The line is then drawn glyph by glyph from the adjusted start (Text layout and glyph drawing).

Users (Medium-High for the full list): mode 0 is used by the speaker name. Modes 0, 1 and 2 are used by labels and value text (the mode taken from the node's alignment byte). Mode 1 is used by prompt and price captions. Mode 3 is used only by the on-screen world message, the host call DisplayScreenMessage (SPEC-0030), which draws its label above its projected point. That call has 183 call sites in 42 level modules, and all of them lie in script code that is never reached, so mode 3 is never seen in play (High).

Front-end text nodes: the text-node draw of the front end (node kind 2 of resource 532) is owned by SPEC-0035 (Section G and the kind-2 draw). It draws through the same line-run routine; its colour, letter spacing and font come from the label binding.

Code constants (instruction operands, not disc data): the floor 16 and the mode numbers 1, 2 and 3.

### Showing text

The engine offers four operations. Scripts call them through the level script interface (the names are the host calls; the line index comes from the script):

1. Show a raw string: no speaker, colour (128, 128, 128), the box does not block the caller.
2. Show table line i: the line's speaker and colour are applied, no blocking.
3. Show table line i and wait until the box closes.
4. Show table line i (or, if the index is negative, no line), with two choice labels, and wait. The result is 1 for Circle and 0 for Triangle, and LevelPrompt returns true for 1. Labels when the index is not negative: the name field of table line idx is the Circle label (right) and its text field the Triangle label (left). When the index is negative the labels are the table-13 labels 0x21 "Yes" (Circle) and 0x22 "No" (Triangle). See Choices, input and release.

An out-of-range table index returns no record. LevelMsg then shows nothing, changes no state and returns at once, whatever its wait value; LevelPrompt returns false at once (Showing a level line).

### Substitution codes

Confidence: High unless marked.

1. A code whose run-pass mask (V − 2) has bit 5 (value 32) set is a substitution code. No type has a handler for it, and the draw and reveal passes never run it. On disc these are the type-5 codes of design table 9 (SPEC-0024): n = 3, L = 13, V = 34.
2. When the dialogue box opens a text, it first scans the text byte by byte. Each substitution code is replaced by the text of substitution slot k, where k is the code's first argument:
   - the result (the text before the code, then the slot text, then the text after the code) is built in a 512-byte work area;
   - the scan continues one byte after the code's position;
   - codes without bit 5 are stepped over whole.
3. There are four slots (k = 0 to 3), each holding up to 63 characters. k is not range-checked in the original (only k = 0 and 1 occur on disc).
4. Slot 0 holds the memory-card slot number, set to 1. Slot 1 holds the number of KB a save needs. Both are set by the card-state screen and by the load screens before a card message opens. A number is written in decimal (a negative value would give wrong characters; quirk, not met on disc). Where the KB value comes from is open (Open question 64).
5. Only the box performs substitution. Elsewhere a substitution code draws nothing and has width 0.

### Dialogue box layout

Confidence: High for the formulas; Medium-High for the canvas units.

UI positions are in a 512 × 448 canvas: the UI-to-GS conversion centres a 512-wide, 448-tall area (data words 0x2EC908 = 512 and 0x2EC904 = 448; UI y scale 0x2EC900 = 1.0). Let L = the number of wrapped lines of the current page at wrap width 410 with font 0 (at least 1), h = 21 (the font 0 line height) and Y0 = the box's y offset word (0 on every path described here).

- Base line y: yb = 345 − 10 L + Y0. The 345 and the 10 are code constants.
- Frame: x = 50 − 20 = 30; y = yb − 20 − h; width = 410 + 20 + 16 = 446; height = L h + 4 + h + 20 (data words in Box geometry constants).
- The frame is a nine-slice of the 64 × 64 box sprite (front-end logical sprite 0x19). Borders: left 15, right 28, top 14, bottom 26 pixels (data words 0x2EC91C, 0x2EC920, 0x2EC924, 0x2EC928; sprite size words 0x2EC92C and 0x2EC930 = 64). The texture splits are at u 15 and 36 and at v 14 and 38. The frame colour is the speaker colour (R, G, B, each 0 to 255, used as a GS modulate colour where 128 = unchanged) with alpha 45 (of 128, about 35 % opacity).
- Speaker name: font 0, left aligned at x = 70, baseline y = yb − 20 − floor(h / 2) = yb − 30, spacing −1 pixel per glyph, colour (128, 190, 128, 128) from data address 0x3159F0. It is drawn only when a name is set.
- Body text: font 0 at x = 50. The first baseline is yb and line i is at yb + 21 i; all L lines are drawn. Base colour (128, 128, 128, 128) from data address 0x3159F8. Wrap width 410.
- Prompt row (drawn only after the page is fully revealed): baseline y = yb + h L + 1. The continue label is at x = 384 with the Square icon (logical sprite 0x1B): label 0x10 "Next" when a stop is pending, label 0x11 "Finish" on the last page. In choice mode on the last page there is no continue label: the second label is at x = 128 with the Triangle icon (logical sprite 0x78) and the first label at x = 384 with the Circle icon (logical sprite 0x79). The x values are the two 16-bit halves of data word 0x2DCC40.
- A prompt label is font 1, spacing −1, colour (160, 160, 90, 127) from data address 0x315A38. With the label's text width tw (font 1, spacing −1) and the icon extent E = 32: group left gx = x − (tw + E/2) / 2 (halved toward zero); the icon is at (gx, y); the text is at x = gx + E/2, y = y + E/2 − th/2, where th is the measured label height (the height of its last glyph). Medium-High; the icon's anchor convention is Medium.
- The box grows upward by 10 pixels per line and downward by 11 pixels per line.

Lines per page on disc (font 0, wrap 410): 1 line on 2,707 pages, 2 on 4,767, 3 on 2,088, 4 on 462, 5 on 48, 6 on 10, and 7, 10 and 11 lines on one page each.

### Reveal and paging

Confidence: High unless marked.

1. A page is the text from the page start to the next type-4 stop or to the end. The page is copied into a 512-byte buffer (with no bound check; the longest page on disc is 345 bytes). The page length B counts bytes, control codes included.
2. Each box update (one per front-end pass; see Pacing below) draws first, then adds the time step to the progress p. The time step is 1/30 per pass (the fixed step of SPEC-0007), times 5 while Cross is held (held bit 0x0040). The reveal rate r is 10.0 characters per progress unit, set when the box is created. So normally one character appears every 3 passes (20 characters per second), and 100 per second with Cross held.
3. The visible index is n = floor(p × r + 0.001). Any control codes at or before the frontier are run (reveal pass) and skipped at once, and p is set to n / r after the skip, so control codes take no time. When p is 0, n = 0 and the frontier fraction is taken as 0.1.
3a. Code runs per update (High). Let m be the visible index of the previous update (taken from the progress the previous update ended its skip step with, before the time step was added) and n the new one. First, every control record that starts in m ≤ index < n is run once, in order, stepping over each record's length. Then records that start exactly at n are run and stepped over as in item 3. The next update's m is at or past every record already run, so no record runs twice in one reveal. A skip (item 5) moves n to B and runs every record not yet run, once, in that update. Test vector: page "ab" + record R + "cd" (R at index 2). Update with m = 1, n = 2: nothing runs in the range, R runs at the frontier and n becomes 2 + L. Later updates start at m ≥ 2 + L, so R does not run again.
4. Bytes before n are drawn in full. The glyph at byte n is drawn scaled by f = p × r − n (0 to 1) about its bottom-left corner, so each glyph grows in over its 3 passes.
5. Skip: on a Square press (press edge, logical bit 0x0080) with p above 0, p is set to B / r and the whole page shows in that update. The p > 0 guard keeps the press that turned the page from also skipping the new page. Holding Square does not skip; only the press edge does.
6. The page is complete when n ≥ B. Input for the prompt row is read only in an update that starts with the page already complete (set by the previous update). The box never advances on its own.
7. A box opened with the instant flag (the controller message) is complete on its first update.

Pacing (High): while a box is up, the box host state runs once per main-loop pass and updates the shared box once in that pass. Main-loop passes are paced to one per vertical blank and never catch up (SPEC-0007), and the step added each update is a fixed word, not measured time. So the reveal counts passes: 60 passes per second when no pass overruns, fewer under slow-down, and the characters per pass never change. While a front-end load is pending the pass ends before any state runs, so the box does not advance.

Timing: a page of 5 plain characters is complete in update 15 (the 16th update, 0.25 s). With Cross held from the start it is complete in update 3. A Square press in update 4 completes it in update 4.

### Choices, input and release

Confidence: High unless marked.

The box reads the OR of all pads' held words and the OR of their press-edge words from the input block (SPEC-0033 button bits). These come from the menu snapshot, which is pad slot 0.

1. Last page, not choice mode, prompt allowed: label "Finish". A Square press closes the box: the text is cleared. The box plays no sound and runs no callback. On the next pass the box host state (SPEC-0035 state 23) sees the box idle, plays cue 0x97 (text box close) and returns to play. A script waiting in DisplayDialogText (wait form) or LevelPrompt resumes.
2. Stop pending: label "Next". A Square press moves to the page after the stop (finding the next stop), plays cue 0x98 (advance) and, if the box has an owner, runs the owner's action. During a talk the owner's action clears the speaker override (Speaker name and colour, rule 5).
3. Choice mode, last page: a Circle press sets the result to 1 and plays cue 0x98. A Triangle press sets the result to 0 and plays cue 0x99 (back). Either closes the box as in item 1. Circle wins when both are pressed in the same pass. On the earlier pages of a prompt the Next rule applies and no choice is shown.
4. LevelPrompt returns true when the result is 1 (the Circle answer). With no line index the labels are table-13 labels 0x21 "Yes" (Circle, right) and 0x22 "No" (Triangle, left). With a line index the record's name field is the Circle label and its text field is the Triangle label.
5. The result is −2 after start-up, −1 after a new text or a choice setup (no choice yet), and 0 or 1 after a choice. It stays readable until the next text or prompt.
6. The talk is not ended by the box. It ends when the script calls EndConversation (Conversations, item 5).

### Memory-card and controller messages

Confidence: High.

1. A memory-card operation that needs a message opens the shared dialogue box. The box shows the message text, two choice labels and a colour taken from the system message table: design table 9 of archive resource 533, 27 records of 16 bytes (SPEC-0024).
2. The routine then waits until the box returns a choice. The choice is read the same way as in Choices, input and release.
3. When the caller sets its wait flag, the card status is also checked on each pass. Two card-status fields are read; the wait returns early when either one is zero.
4. The save and load overwrite prompts are these card messages. The save spec describes the card operations that call them.
5. Controller message: a second box exists for it. While pad 0 is not ready, every front-end state runs this box with table 9 record 25 (its text and colour), instant reveal, alpha 128 and no prompt row. The box goes away when the pad is ready again.

### Speaker name and colour

Confidence: High; the partner block Medium.

When a table line is shown:

1. If the record's speaker name equals "Hero" or "[Hero]" ignoring the case of A to Z (so "HERO" also matches; Text comparison and page stop search), or starts with a control-code marker, the name is the active hero's: "Tor" when the hero flag of the active hero record is 0, else "Roberta". The colour is design table 10 entry 0 or 1 (SPEC-0024): Tor (74, 100, 255), Roberta (0, 222, 0) (bytes R, G, B; byte 3 unused).
2. Otherwise, if the hero has a talk partner, the partner character's own name and colour are used: the name and the colour bytes of its placement data (name at placement data +0x20, colour bytes at +0x40). On disc 580 of 634 placed characters have white (255, 255, 255); named characters have their own colours. The record's name and colour are not used during a talk.
3. Otherwise the record's name and its colour bytes 0 to 2 are used. On disc 7,019 records have colour (0, 0, 0), which gives a black translucent frame; 347 have Tor's colour and 40 Roberta's.
4. A raw string (DisplayDialogText with a string) has no name and the colour (128, 128, 128), with the frame alpha 45.
5. Override: a type-32 code (reveal pass) sets an override name and colour to the active hero's (rule 1). A type-34 code clears the override. While an override is set it replaces the name and the frame colour. During a talk, every page advance clears it (the talk owner's action). Opening a new text clears it. On disc, a type-32 code is at the start of its page in 382 of 434 codes, and a type-34 code in 189 of 191. Types 32 and 34 play no sound.

### Control codes in the text

1. Type 4 is a page stop (see Reveal and paging).
2. Other types draw nothing and add no width. They take no reveal time (Reveal and paging, item 3). A type whose run-pass mask has bit 0 set runs each time the drawing reaches it; one with bit 1 set runs when the reveal reaches it (Text encoding, byte V).
3. Type 2 is a text colour code. Its one argument group is R, G, B, A, and the glyphs drawn after it use that colour. Each update starts the page with the base colour (128, 128, 128, 128); the colour then carries over line ends to the end of the page. On disc there are 16 distinct colours: the most common is the reset (128, 128, 128, 128) in 2,340 codes, then (139, 35, 35) in 711, Roberta's green in 396 and Tor's blue in 333.
4. Handlers: types 2 and 3 are registered by the UI setup; types 16, 22, 23, 24, 32 and 34 by the front-end post-load; types 17 to 21 and 25 by a separate start-up registration. Types without a handler do nothing.

### Dialogue control records (emote, speaker and command records)

Confidence: High for the record layout, the kinds, the run rule, the counts and who each emote kind acts on. High for kind 0x16 (Medium-High for its label). Medium for the item-notice setting of kind 0x13.

A dialogue control record is a control code of the kind described under Control codes in the text. Its fields:

| offset from marker | size | meaning |
| --- | --- | --- |
| +0 | 1 | marker 0x01 |
| +1 | 1 | n: the group count plus 2. Every emote request on disc has n = 3 (one argument group); every kind 0x11 and 0x13 record has n = 4 (two groups) |
| +2 | 1 | encoded length of the record in bytes |
| +3 | 1 | kind (see the table below) |
| +4 | 1 | flag plus 2 (byte V of Text encoding). Every emote request on disc has the byte 4 (flag 2); the kind 0x11 and 0x13 records have flags 10 and 18 |
| +5 | 4 per group | argument groups, decoded with the escape rule (0xFD is the escape byte); each group is a signed 32-bit little-endian word |

For an emote request the first argument group is the emote id. For the command kinds, arg0 is the first group and arg1 the second. Every disc record of a two-argument kind has two groups; what a missing group reads as is not established.

Run rule. The draw pass tests bit 0 of the flag and the reveal pass tests bit 1. Every record of these kinds on disc has bit 1 set and bit 0 clear, so none runs on the draw pass. Each runs on the reveal pass, once, when the typewriter reveal reaches its position in the text.

| kind | meaning | records in line tables on disc |
| --- | --- | --- |
| 0x10 | Partner body emote. Acts on the selected hero's talk partner; with no partner it queues nothing. The emote id is a main animation id of the partner and a row of table 7 (SPEC-0024). The clip is offset by 74 times the partner's voice set (SPEC-0009, Emote clip lookup). The request goes to the partner's body queue (SPEC-0040). | 8,998 per line record (8,947 distinct strings) |
| 0x17 | Hero body emote. Acts on the selected hero. The emote id is a main animation id of the hero and a row of table 8. The clip is offset by 183 when the selected hero is Roberta (the hero directory's entry count shifted right by one). The request goes to the hero's body queue. | 923 (919) |
| 0x18 | Hero face emote. Acts on the selected hero. The emote id is a face animation id and a row of table 8. No hero shift is applied for either hero. The request goes to the hero's face queue. | 293 (290) |
| 0x16 | Partner face emote, no clip. Acts on the selected hero's talk partner; with no partner it queues nothing. It queues (no directory, position -1, emote id) on the partner's face queue, so the face animation plays with no voice. | 10: ids 126 (2) and 127 (8), all with flag 2 |
| 0x20 | Speaker override: the speaker becomes the active hero (Speaker name and colour, rule 5). No arguments. | 434 |
| 0x22 | Speaker override clear. No arguments. | 191 |
| 0x11 | Set progress flags. If arg1 < 1 or arg1 ≤ arg0, set flag arg0. Otherwise set every flag from arg0 to arg1 inclusive. This is the plain set: no watcher is notified. | 1 (pack 772, line 51: sets flag 416 alone, arg1 = −1) |
| 0x12 | Clear progress flags. The same range rule; plain clear, no watcher is notified. | 0 |
| 0x13 | Give items. Give arg1 copies of item arg0 to the party inventory, by the same add as the level-script item grant. A count of exactly 18 is replaced by 1 before the give (quirk). Item 135 with a count is a Neopoint reward (the count is the amount). An "item received" notice is shown first when a global setting is on (the setting is not identified; Medium). | 166 (counts 1 to 2,500; item 135 in 67 of them) |
| 0x14 | No effect: the handler calls an empty routine. | 0 |
| 0x15 | No effect: the handler only prints a debug message (a cut dialogue-camera command whose argument is a camera number). | 7 (all in one line of pack 772) |
| 0x19 | Apply stage arg0 from the stage table, with notification (the same as the stage change of Progress flags). | 0 |

Command binding (High). The text engine keeps a 48-slot command table (run-time data 0x39B480), indexed by command kind. The six command handlers are bound into it once, at boot: the main boot function stores one handler object each into slots 0x11, 0x12, 0x13, 0x14, 0x15 and 0x19, after the game-globals setup and before the start-up loaders (SPEC-0026, boot order). The handler objects and the table are both built by start-up table entries (SPEC-0026, Start-up table). A store to a slot index of 48 or more is ignored.

Counts. Counted per line record, the emote request kinds give 8,998 / 923 / 293 / 10 records (kinds 0x10 / 0x17 / 0x18 / 0x16). 58 of these codes sit in text strings that two line records share; counting each string once gives 8,947 / 919 / 290 (and the same 10 for kind 0x16). A raw byte scan finds 29 more kind 0x10 matches outside the line tables (11 of them inside texture chunks of packs 776 to 789); none lies in a line table, so the text box never reads them. Kinds 0x10, 0x17 and 0x18 occur in 99, 66 and 53 resources.

Line census (per line record). In lines named by a live call in reached packs (Line use) there are 2,861 kind 0x10, 358 kind 0x17, 131 kind 0x18 and 7 kind 0x16 requests. By the row they name: kind 0x10, 8,235 a row that always plays a clip, 689 a row with no clip and 74 a row whose clip never plays; kind 0x17, 667 always, 250 no clip and 6 with id -2; kind 0x18, 288 always, 2 no clip and 3 with id -2.

Who emotes (High):

- Kinds 0x17 and 0x18 act on the selected (controlled) hero. The Roberta shift and the "Hero" speaker name both come from the selected hero, so the voice half always matches the name shown.
- Kinds 0x10 and 0x16 act on the selected hero's talk partner. The hero has a partner only after the level script calls SetConversationTarget, or SetConversationTargetWithOrient with t = 1 (Conversations). With no partner, these kinds queue nothing. In a proximity talk the partner is the talked entity, except for a few cutscene lines with a fixed partner and prompt lines with none (Proximity talk frame and line rotation, item 6).
- The emote id is also the animation id: a main-table id for kinds 0x10 and 0x17, a face-table id for kinds 0x16 and 0x18.

No spoken lines (High). No line record field and no control code selects a spoken line or a line recording. The only audio a line can carry is the emote clips of kinds 0x10, 0x17 and 0x18: short gesture vocalisations (SPEC-0009, Emote clip playback).

No run-time lip sync (High; Medium for the meaning of face id 0). Nothing drives a face from audio or from the text reveal, and no level-script call touches faces. While an actor is in the talking state, each tick its face override is held at the queued face emote id, or at face id 0 when its face queue is empty (SPEC-0040, Behavior 22). Any mouth motion is part of those face animations. Matches of kinds 0x11 to 0x15 and 0x19 outside the line tables are false matches in texture and mesh data. Among the kind 0x13 rewards is item 23, given by Guildmaster Marl in pack 1017 (the Trader's Guild reward). No dialogue record sets or clears a main-story flag.

Emote row lookup, the roll and the directory position are in SPEC-0009 (Emote clip lookup). Queueing and playback are in SPEC-0040 (Behavior 22).

### Text comparison and page stop search

**Case-insensitive compare (High).**
- Two strings are compared one character at a time after folding. Folding adds 0x20 to the codes 0x41 to 0x5A (A to Z) and leaves every other code unchanged. It uses the C library's character-class table in the boot program's data at 0x329FC1 + code, where bit 0 marks an upper-case letter. Among codes 0 to 255 the table marks only A to Z.
- The result is the difference of the first pair of folded characters that differ, so equal strings give 0.
- Its only user is the speaker test (Speaker name and colour, rule 1), which therefore ignores the case of A to Z. On disc only the exact name "Hero" occurs (120 records).
- A byte of 0x80 or above can never give a false match, because the two names compared against hold only ASCII.

**Page stop search (High).** This is the box's search for the next page stop, made against the control-code handler registry. Paging (Reveal and paging) and box open find the next page stop by testing every byte position from the page start; they do not step over codes. A position is a stop when its byte is the marker 0x01 and the type byte 3 positions later parses as 4. The pending stop is recorded as its position and its end (position + L), and the next page starts at that end. When no stop is found, both are cleared. A raw 0x01 byte inside a code's arguments could in theory fake a stop. On disc no code argument holds a raw byte 0x01 in any of the 18,252 line-table codes or the 274 design-table codes, because values 0 to 0x20 are always written as escape pairs.

### Conversations (talk start and end)

Confidence: High unless marked.

1. A talk starts when a Square press reaches the talk entry of the talk candidate (Interaction range and target choice). The press starts the level script's talk callback as a thread. The talking state and the partners are set by the script calls below, not by the press itself.
2. SetConversationTarget(id, b): the hero and the entity id enter the talking state (hero state 25). Each saves its previous state, unless it is already in state 25. The hero's talk partner becomes the entity. The entity's partner becomes the hero only when b is 1. Both characters' emote queues (body and face) are cleared. The dialogue box's owner is set to the talk; the owner's action, run on each page advance, clears the speaker override (Choices, input and release, item 2).
3. SetConversationTargetWithOrient(id, b, t): the same as SetConversationTarget, except that the hero's partner is set only when t is 1.
4. Turning: in state 25 the hero stops moving and turns its facing toward its partner by 0.1 of the remaining angle each tick, so the remaining angle shrinks by the factor 0.9 per tick. An NPC in base state 25 with a partner and no pose turns by the steering rule with weight 0.3, a vector blend v = 0.3 t + 0.7 c, then normalised (SPEC-0056, Talk facing). A character with no partner, or an NPC in a pose, does not turn.
5. EndConversation(id): the hero, if it is still in state 25, returns to its saved state, and its partner is cleared. The entity likewise returns to its saved state if it is still in state 25, and its partner is cleared. The box owner is cleared. The box itself does not release its owner and does not end the talk.
6. The hero's partner is the speaker source of Speaker name and colour, rule 2. A line shown while the hero has no partner uses the record's own name and colour.
7. Box closed in state 25: when the box is not open while the hero is in state 25, the chooser's special-state branch returns a hero that is not busy to state 1 and clears the box (Interaction prompts, chooser step 2). If that happens between two lines of a script, the next LevelMsg puts the hero back into state 25 (without saving its state), and EndConversation later finds the hero not in state 25 and leaves its state unchanged. The chooser does run between two lines: in each pass the hero update runs before the script step, so it returns the hero to state 1 before the next LevelMsg puts it back into state 25 (SPEC-0030, Per-tick script step). (Medium-High for the between-lines outcome.)
8. Queries: IsHeroTalking is true when the hero's state is 25. IsDialogActive is true while the box has text or a pending stop. IsHeroOccupied is the special-state test of SPEC-0033 (the set used for Z in Interaction prompts).
9. The dialogue does not move the camera. The camera changes only through script commands (Medium).

### Proximity talk frame and line rotation

Confidence: High unless marked. This spec owns the frame; the level specs give each callback's branches and lines and refer to it.

1. **Frame.** A talk callback (Interaction range and target choice, Talk entry) is a first-match chain of tests, or a small state machine kept in module data words, that chooses at most one line index. It then runs SetConversationTarget(the talked entity, 1), LevelMsg(line, world, b, wait 1) and EndConversation(the talked entity). The talked entity is the entity whose talk entry fired; several entities may share one callback. When no line is chosen, the callback does nothing: no conversation starts, no state changes and no box opens. Some callbacks first return at once when the hero is already talking. A prompt line (LevelPrompt) inside a callback is shown with no conversation target, unless a line was shown just before it without EndConversation; the prompt's true answer is "yes".
2. **Branches.** The chain tests story flags in a fixed order, highest story stage first, and the first set flag picks the branch. Some callbacks also test the active hero (one line for Tor, another for Roberta), held items, the petpet type, Neopoints, an earlier prompt answer or a random draw.
3. **Rotation ("lines in turn").** A callback keeps two module data words, a branch marker M and a counter C, both 0 in the file. A rotating branch k first compares M with k; when they differ it sets M = k and C = 0, so the sequence restarts. Then C picks the line and steps. Four step patterns occur (cycle, first-once, two-line and stop); their rules are in Line rotation patterns below.
4. **Shared counter.** Some callbacks keep C and no M, so a change of branch does not restart the count (Line rotation patterns).
5. **Ownership and lifetime.** The words belong to the callback, not to an entity, so all entities that share a callback continue one sequence. They are ordinary module data words: they keep their values while the module's pack stays in its world buffer and return to their file values when the pack is loaded again (SPEC-0030, Behavior section 1.1). So an exterior keeps its rotations across interior visits, and an interior restarts them on every entry.
6. **Talk partner of the lines (High for the rule; Medium-High for the counts).** A proximity line is shown between SetConversationTarget and EndConversation, so the hero's talk partner during the line is the talked entity, and kind 0x10 emotes use that entity's voice set (SPEC-0009). For a shared callback the same line is voiced by whichever of its entities was talked to. Over the 14 level modules of the main path: 2,444 lines (3,038 kind 0x10 requests) have the talked entity as partner; 34 lines (46 requests), in cutscenes run from a proximity callback, have a fixed partner id; 17 lines (23 requests), the arena and map-seller prompts and one race-start line, are shown with no partner, so their kind 0x10 records queue nothing (Who emotes). The complete crosswalk of active proximity registrations in the 14 modules appears below, with shared callback groups shown together. Fixed partner IDs in proximity-run cutscenes and lines with no partner are listed separately.



#### Entity id to placement and voice set

Confidence: High for the entity-table, placement, row lookup and voice set. Each row groups the entity IDs registered to one callback; singleton registrations are shown separately. IDs are local to the pack. The voice set is the signed word at resource 531 row +0x14, selected by the placement's type and variant. A value of −1 has no character emote voice set.


| Pack | Callback group entity ids | Placement name | Voice set |
| ---: | --- | --- | ---: |
| 909 | 31, 32, 33, 34, 35, 36 | Lucy | 11 |
| 909 | 37, 38, 39, 40, 41, 42 | Patricia | 10 |
| 909 | 43, 44, 45, 46, 47, 48, 49, 50 | Hubert | 9 |
| 1023 | 59, 96 | Wymond | 8 |
| 1023 | 60, 61 | Bermund | 5 |
| 1023 | 62, 95 | Grizwuld | 13 |
| 1023 | 72, 75, 94 | Rory the Shady Guy | 5 |
| 1023 | 76, 78 | Estrid | 6 |
| 1023 | 84, 88, 91 | Tristan | 3 |
| 1023 | 85, 92 | Jervis | 5 |
| 1023 | 89, 93 | Corthos | 14 |
| 1023 | 2 | Guardsman Darol | 2 |
| 1023 | 3 | Guardsman Conely | 2 |
| 1023 | 4 | Guardsman Ulfric | 2 |
| 1023 | 5 | Guardsman Hector | 2 |
| 1023 | 6 | Guardsman Henkel | 2 |
| 1023 | 7 | Guardsman Wade | 2 |
| 1023 | 57 | Sir Aldwin | 15 |
| 1023 | 58 | Sir Luthor | 14 |
| 1023 | 63 | Alexandra | 0 |
| 1023 | 64 | Sally | 1 |
| 1023 | 65 | Hawker Gamzan | 2 |
| 1023 | 66 | Shanra | 0 |
| 1023 | 67 | Miriam | 6 |
| 1023 | 68 | Master Gryth | 13 |
| 1023 | 69 | Ella | 0 |
| 1023 | 70 | Gram | 8 |
| 1023 | 71 | Sammy | 15 |
| 1023 | 73 | Vonkar | 8 |
| 1023 | 74 | Old Knight | 8 |
| 1023 | 77 | Guntur | 15 |
| 1023 | 79 | Otho | 4 |
| 1023 | 80 | Guff | 4 |
| 1023 | 81 | Paunch | 13 |
| 1023 | 82 | Sir Ignatius | 13 |
| 1023 | 83 | Sir Ebard | 8 |
| 1023 | 86 | Kendel | 3 |
| 1023 | 87 | Isolde | 0 |
| 1023 | 90 | Sowter | 8 |
| 1300 | 7, 8, 9, 10, 55 | Seradar | 3 |
| 1300 | 11, 12, 13 | Council Herald Kekou | 5 |
| 1300 | 14, 15, 16, 17, 74 | Old Lady | 1 |
| 1300 | 61, 76 | Queen Fyora | -1 |
| 1300 | 18 | Painter Howell | 5 |
| 1300 | 19 | Potionmaker Fizlar | 13 |
| 1300 | 20 | Shopkeep Visal | 12 |
| 1300 | 21 | Guardsman Digby | 2 |
| 1300 | 22 | Guardsman Renold | 2 |
| 1300 | 23 | Guardsman Gillard | 2 |
| 1300 | 24 | Guardsman Danvers | 2 |
| 1300 | 56 | King Hagen the Wise | 14 |
| 1300 | 57 | King Skarl | 14 |
| 1300 | 58 | Lucy | 11 |
| 1300 | 59 | Patricia | 10 |
| 1300 | 60 | Hubert | 9 |
| 1300 | 62 | Prince Tourin | 3 |
| 1300 | 63 | Princess Dona | 0 |
| 1300 | 64 | Chelsie | 6 |
| 1300 | 65 | Lexa | 6 |
| 1300 | 66 | Whitney | 6 |
| 1300 | 67 | Primrose | 6 |
| 1300 | 68 | Mistress Marion | 6 |
| 1300 | 69 | Illusen the Earth Faerie | -1 |
| 1300 | 70 | Old Knight | 8 |
| 1300 | 71 | Master Torak | 4 |
| 1300 | 72 | Servant Faun | 0 |
| 1300 | 73 | Gord the Merchant | 14 |
| 1300 | 75 | Kell the Beggar | 15 |
| 758 | 26, 28 | Syd | 4 |
| 758 | 23, 25 | Lady Deborah | 0 |
| 758 | 10, 12 | Harry | 15 |
| 758 | 11, 13 | Lady Margo | 1 |
| 758 | 27, 30 | Cinthia | 6 |
| 758 | 14 | Lilly | 6 |
| 758 | 15 | Corwill | 5 |
| 758 | 16 | Minerva | 1 |
| 758 | 17 | Flaebert | 3 |
| 758 | 18 | Lord Cedrick | 3 |
| 758 | 19 | Sir Caradoc | 4 |
| 758 | 20 | Gillian | 0 |
| 758 | 21 | Toby | 8 |
| 758 | 22 | Velhar | 2 |
| 758 | 31 | Sir Drew | 8 |
| 758 | 32 | Guardsman Wallace | 2 |
| 758 | 33 | Guardsman Kris | 2 |
| 758 | 34 | Guardsman Lisle | 2 |
| 758 | 35 | Guardsman Piero | 2 |
| 758 | 36 | Guardsman Jack | 2 |
| 777 | 0, 4 | Guardsman Burhold | 2 |
| 777 | 1, 3 | Herald Johfrit | 15 |
| 777 | 2, 5 | Guardsman Chester | 2 |
| 777 | 11 | Sir Kaherdin | 14 |
| 777 | 12 | King Hagen the Wise | 14 |
| 777 | 13 | Sir Drysdan | 4 |
| 777 | 14 | Lore the Librarian | 13 |
| 777 | 15 | Lady Bovonova | 6 |
| 777 | 16 | Lord Sydney | 5 |
| 842 | 3, 17, 22, 44 | Rose | 6 |
| 842 | 5, 16, 34, 43 | Gershwin | 15 |
| 842 | 7, 20, 23, 29, 37, 47 | Giovanni the Explorer | 14 |
| 842 | 10, 18, 33, 45 | Stanley | 8 |
| 842 | 11, 14, 27, 41, 48 | Tadd | 15 |
| 842 | 15, 26, 28, 42 | Ruffie | 4 |
| 842 | 21, 32, 35, 46 | Mayor Jurgin | 5 |
| 842 | 24, 39 | Ingmar Anvilmaster | 4 |
| 842 | 25, 40 | Serene | 1 |
| 934 | 0, 40 | Farmer Tessa | 1 |
| 934 | 1, 43 | Luke | 3 |
| 934 | 3, 44 | Peter | 13 |
| 934 | 4, 5 | Old Man Boggs | 4 |
| 934 | 9, 42 | Claire | 1 |
| 934 | 2 | Farmer Addison | 13 |
| 1098 | 35, 36 | Master Torak | 4 |
| 1098 | 38, 43 | Guardsman Henry | 2 |
| 1098 | 40, 42 | Lady Anabel | 1 |
| 1098 | 19, 22, 31 | King Skarl | 14 |
| 1098 | 23, 33 | Sir Harlag | 4 |
| 1098 | 15 | Lord Adelheit | 8 |
| 1098 | 16 | Servant Eggbert | 13 |
| 1098 | 17 | Squire Grayson | 8 |
| 1098 | 18 | Squire Grayson | 8 |
| 1098 | 20 | Sir Lawrence | 15 |
| 1098 | 21 | Sir Cadmere | 13 |
| 1098 | 24 | Sir Rufus | 4 |
| 1098 | 25 | Sir Ebard | 8 |
| 1098 | 32 | Lord Cormac | 8 |
| 1098 | 34 | Chef Henry | 4 |
| 1098 | 37 | Servant Gottard | 5 |
| 1098 | 39 | Old Knight | 8 |
| 1098 | 41 | Lord Darius | 13 |
| 1098 | 44 | Guardsman Korbin | 2 |
| 1098 | 45 | Guardsman Oliver | 2 |
| 1098 | 46 | Guardsman Laertes | 2 |
| 1098 | 47 | Guardsman Eogan | 2 |
| 1098 | 48 | Lady Leonarda | 6 |
| 1098 | 49 | Chamberlain Lazlo | 3 |
| 1098 | 50 | Servant Quintin | 5 |
| 1350 | 24 | Sevaric | 14 |
| 1350 | 25 | Bremner | 15 |
| 1350 | 26 | Cestius | 4 |
| 1350 | 27 | Gyrtias | 3 |
| 1350 | 28 | Talos | 8 |
| 1350 | 29 | Tharius the Sage | 13 |
| 1350 | 30 | Nicodromos | 2 |
| 1350 | 31 | Tullus | 15 |
| 991 | 9, 12, 40 | Sir Jaceus | 15 |
| 991 | 10, 13 | Ironscale | 13 |
| 991 | 15, 42 | Steen | 15 |
| 991 | 16, 41 | Lady Marissa | 1 |
| 991 | 1 | Vernan | 2 |
| 991 | 2 | Luger the Swift | 3 |
| 991 | 3 | Zippy | 12 |
| 991 | 4 | Guardsman Melyn | 2 |
| 991 | 5 | Guardsman Kaethor | 2 |
| 991 | 6 | Guardsman Tonkin | 2 |
| 991 | 7 | Guardsman Palk | 2 |
| 991 | 8 | Guardsman Alfred | 2 |
| 991 | 11 | Lord Ulaf | 8 |
| 991 | 14 | Prince Tourin | 3 |
| 991 | 17 | Monty | 15 |
| 991 | 18 | Warren | 12 |
| 991 | 19 | Hiergard | 2 |
| 991 | 20 | Lord Sorguld the Humble | 3 |
| 722 | 10, 13, 6 | Finn | 3 |
| 722 | 8, 15, 4 | Calyn | 5 |
| 722 | 9, 16, 5 | Olga | 1 |
| 722 | 7, 11, 3 | Oscar | 4 |
| 722 | 12 | Arken | 15 |
| 722 | 14 | Chelsie | 6 |
| 722 | 17 | Haplo | 8 |
| 722 | 18 | Hank | 8 |
| 1327 | 1 | Guardsman Flit | 2 |
| 1327 | 2 | Henrik | 8 |
| 1327 | 3 | Klement | 15 |
| 1327 | 4 | Jensine | 1 |
| 1327 | 5 | Librarian Amaris | 6 |
| 1327 | 43 | Seradar | 3 |
| 1327 | 44 | Council Herald Kekou | 5 |
| 859 | 0 | Motara the Mote Merchant | 0 |
| 1005 | 0, 1 | Danthar | 3 |


#### Fixed partners and lines without a partner

The fixed partner IDs in proximity-run cutscenes resolve to: pack 1098 entity 39 = Old Knight (voice set 8), entities 35 and 33 = Master Torak (voice set 4); pack 1300 entity 18 = Painter Howell (voice set 5); pack 934 entity 9 = Claire (voice set 1). The arena/map prompt and race-start lines have no partner, so their kind-0x10 requests queue nothing.

#### Line rotation patterns

Confidence: High. This is the shared notation for the rotating branches that the level specs list. A rotating branch has the number k and two or three line indices a, b, c. Its state is the callback's counter C (0, 1 or 2) and, unless the callback is marked "shared counter", the callback's branch marker M. Each talk that reaches the branch does three steps:

1. **Reset check.** If the callback has M and M is not k, set M = k and C = 0. A callback with no M skips this step.
2. **Show.** Show the line that C selects in the table below. If C selects no line, the talk is silent: no conversation starts and nothing changes (Frame, item 1).
3. **Step.** Set C to the next value in the table.

C and M also return to 0 when the module's pack is loaded again (item 5).

| Notation in the level specs | Pattern | C = 0 | C = 1 | C = 2 | Lines shown after a reset |
|---|---|---|---|---|---|
| "a, b, c" (also written as the range "a-c") | cycle | a; C = 1 | b; C = 2 | c; C = 0 | a, b, c, a, b, c, ... |
| "a; then b, c" | first-once | a; C = 1 | b; C = 2 | c; C = 1 | a, b, c, b, c, ... (a once per reset) |
| "a, b" | two-line | a; C = 1 | b; C = 0 | not reached | a, b, a, b, ... |
| "a, b, then silent" | stop | a; C = 1 | b; C = 2 | no line; C stays 2 | a, b, then silent until the branch changes |

**Shared counter.** "Shared counter" marks a whole callback, not one branch. The callback keeps C but has no M, so step 1 never runs. When the branch changes, the count does not restart: the next talk shows the new branch's line at the current C. A later reload of the module still sets C back to 0.

**Who shares the state.** M and C belong to the callback, so every entity that uses the callback advances one sequence (item 5). The branch tests (item 2) choose the branch first, and the rotation then picks the line within it. When the callback has M, a talk in a different rotating branch of the same callback restarts the count at C = 0 (step 1).

### Showing a level line (LevelMsg and LevelPrompt)

Confidence: High.

1. LevelMsg(i, world, b, wait) looks up line i of the world's line table. When i is out of range, nothing happens: nothing is shown, no state changes and the call returns at once, whatever the wait value.
2. Otherwise the speaker and colour rule applies (Speaker name and colour). Then, if the hero is not busy (states 33, 48, 50 to 53 and 55), it is put into state 25 directly; its previous state is not saved, unlike SetConversationTarget. It does not clear the emote queues and does not set a partner. The shared box opens with the line.
3. With wait = 1 the call yields once per tick while the box is open and returns when it closes. With any other wait value it returns at once. On disc every LevelMsg that runs in play passes 0 or 1: 831 of the 841 live sites wait and 10 do not.
4. The argument b is stored and never read.
5. LevelMsg always uses the shared dialogue box. There is no other display path: no banner form, no auto-dismiss and no timer. The box closes only on the Square "Finish" press (Choices, input and release, item 1).
6. LevelPrompt(i, world, idx) shows line i the same way, sets the two choice labels (Choices, input and release, item 4), yields while the box is open and returns the answer. When line i is out of range, nothing is shown and it returns false at once.

### Interaction range and target choice

Confidence: High unless marked.

The Action button is Square (logical bit 0x0080); S below is its press edge. The hero's button chain that routes it is in SPEC-0033 Controls.

**Candidate lists.** The hero keeps two candidate lists of up to 15 characters each. They are rebuilt from the character list every tick after the hero's button step, so the button step uses the lists of the previous tick.

1. Talker list: the characters whose talker flag is set. The talker flag is the townsfolk flag (character word +0xC54; SPEC-0040 Behavior 3C and SPEC-0056), which is 1 exactly when the character's default behaviour class is 6, so the talker list is the townsfolk-flagged characters. An entry whose character's flag is clear is removed on the next update (High). An entry is eligible when its 3-D distance to the hero is at most 20.0 units and the cosine between the hero's facing and the direction to it is at least 0.5 (both code constants).
2. Lock-on list: the targetable characters of SPEC-0036 (range 20.0 for Tor and 100.0 for Roberta, cosine 0.5; data words 0x2DD6A8, 0x2DD6AC, 0x2DD6B0 and 0x2DD6B4).
3. Leader: entry 0 of each list. A leader that stays eligible is kept for up to 90 ticks (code constant 90 for the talker list; data word 0x2DD6B8 for the lock-on list), even when a nearer entry appears. The count restarts when another entry becomes leader. When the leader is not kept, the list is reordered: eligible entries first; then, for each position i and each later position j, entries i and j swap when distance(j) < distance(i) + 1.0. This is not a strict nearest-first order (Edge cases).
4. Talk candidate: the lock-on leader when it is eligible, unless the talker leader is eligible and strictly nearer. When only one leader is eligible, that one. When neither is, there is no candidate. So a nearer lock-on leader takes the candidate from a talker (the lock-on steal); at equal distance the lock-on leader wins.

**Talk entry.** Each world keeps one talk entry per character descriptor, written by RegisterEntityProximityCallback (SPEC-0030) with a callback, the descriptor index and a range flag. The talk is tested only when the hero is not busy (states 33, 48, 50 to 53 and 55), is not in state 25, no buyable item is chosen (SPEC-0050) and a talk candidate exists. With d the 3-D distance between the hero's and the candidate's positions, the entry of the candidate's descriptor gives:

1. No callback registered: no talk, and the candidate blocks nothing.
2. Callback registered and its thread still running: no caption and no new start, but the talk claims the press (below).
3. Range flag set: when d < 5.8, label 8 ("Talk") is shown, and on S in the same tick the callback starts as a thread; it receives a record with the descriptor index, d and the flag. When d is 5.8 or more, nothing is shown or started, but the talk still claims the press.
4. Range flag clear: the callback starts with no press and no range test. Every registration in the level modules on disc sets the flag, so this case is unused.

The thread ends when the callback returns. Its handle is cleared in the world's script step after the hero update, so the next press can start the callback again.

**Claim.** In normal play (hero not in a special state, game-state code 0), when the candidate has a talk entry with a callback (cases 2 and 3) or a buyable item is chosen, no later idle step of the chooser runs that tick: no Grab, no door confirm, no book, no Feed petpet and no Square-with-no-target step. This holds even when the candidate is beyond 5.8 units. (High for the rule; Medium-High for its effect in play.)

### Text window (draw and init)

Confidence: structure High; cue meanings Medium.

1. The text window is drawn only while its open check passes. The screens that open it are the save and load screens, the menu base, and the in-play dialogue path. The save and load card messages use this same box (see Memory-card and controller messages, and Edge cases).
2. The text is laid out with the line counter, using the wrap width of 410 from the layout constants. All L wrapped lines of the page are drawn, one below the other at a step of 21 pixels, the font 0 line height (see Dialogue box layout).
3. There is no choice cursor. The prompt row draws the button icons: logical sprite 0x78 is the Triangle icon, 0x79 the Circle icon and 0x1B the Square icon (32 × 32 each). Cue 0x98 plays on an advance and on a Circle answer, cue 0x99 on a Triangle answer (see Choices, input and release).
4. Creating the window object (its init) writes the HUD tint start colour, neutral 128 per channel (four floats of 128.0), and the neutral colour words of the two HUD elements that flash on timers (SPEC-0023 Layout and section 13). The init also writes other colour words of the window. The text-window init is a start-up table entry (SPEC-0026), so it runs once at boot, before any HUD pulse: the tint start colour (data address 0x37BF20) is 128.0 per channel from boot. Confidence: High (the init writes these, and runs at boot), Medium (the effect).

### Interaction prompts (world action prompt)

Confidence: High for the caption rule, the prompt ids, the gates and the chooser order; Medium-High for the outcomes on disc of one press acting in several places. The chooser is one routine with a fixed order. It is separate from the character idle-animation picker, which chooses idle (fidget) animations on a random timer and shows no prompt.

**Square caption.** The HUD has six prompt slots (their caption and shown-count pointer tables are at data addresses 0x315A98 and 0x315AB0). Only the Square slot (SPEC-0023 section 33) has a live writer. A writer sets the caption text (run-time buffer at 0x37C060) and a shown count of 1 (word 0x2DCD40). The in-play screen draws the six slots once per pass, after the world update, and lowers each count by 1 after drawing it. So a caption is visible only on passes in which something set it: every writer re-asserts its caption each tick, and when several writers set it in one pass, the last one wins. A writer that hides the caption (count 0) removes a caption set earlier in the same pass. While a quick-item panel is open the slots are not drawn and their counts are not lowered. Caption texts are table-13 labels (SPEC-0024) or the purchase text.

**Prompt ids.** Every prompt id is a label index of design table 13 (SPEC-0024); the id picks the caption text.

- Shown by the chooser and the talk: 8 (Talk), 9 (What's this?, the item ask-about of SPEC-0030), 10 (Grab block), 14 (Feed petpet), 18 (Stop Climbing), 19 (Stop Sidling), 20 (Let go), 22 (Grab Door), 23 (Grab Statue), 27 (Catch Miamouse), 28 (Drop Miamouse), 31 (Read Book), the door prompt (13 Enter or 26 Exit, or a script door's own context; SPEC-0006 Doors) and the purchase text.
- Dead in this build: 11 (Sidle), 12 (Fill water bucket) and 24 (Use Rope Gun) sit in the chooser behind tests that always fail, so they are never shown.
- Other writers: 21 (Grab Wall) from the walk states, as the hero enters state 21; 25 (Save Game) from the save points (SPEC-0025); level scripts through DisplaySquareButtonContext.
- Labels 15, 29 and 30 have no known writer.
- The value 0x8D (141) used by the idle-animation picker is an animation number, not a prompt id.

**DisplaySquareButtonContext(n, ...).** Shows label n of table 13 as the Square caption, formatted with the call's extra arguments, and sets the hero's script-prompt flag, which is cleared at the end of every hero input step. n = 0 hides the caption. The call is ignored entirely (no caption, flag unchanged) while the run-time game word at 0x35C9A0 is nonzero (flag D of SPEC-0026, the flag behind game-state code 4). Contexts used on disc: 1 to 6, 13, 26 and 32. While the script-prompt flag is set, a Square press with no target does not enter state 22 that tick (chooser step 3).

**Gates.**

1. The chooser runs each tick from the hero's button step, first in the chain, only while hero input is enabled: bit 0 of the run-time word 0x35C984 is set and the game-state code (SPEC-0026, run-time word 0x35C9DC) is below 1, or is 3 or 4 (SPEC-0033 Behavior 1A, rule 2). The latch words that choose the chain's mask are the two quick-panel flags.
2. Let Z be true when the hero is in a special state (the IsHeroOccupied set of SPEC-0033) or the game-state code is above 0. With code 3 or 4 the chooser therefore runs only its special-state branch.
3. Button bits: the chooser reads the held word and the press-edge word of the input record it is given (+0x150 and +0x154). Square held and Square edge are bit 7 (0x0080) of each. The attack edge is bit 1 or bit 5 (R2 or Circle) of the edge word, and counts only when the hero's mode word at +0xF80 is 0 or 1.
4. The chooser ends the button chain for the tick whenever Square or the attack edge was pressed (SPEC-0033 rule 5).

**Chooser order.**

1. Buy, then talk. When a buyable item is chosen (SPEC-0050), without a press the caption is "Buy" followed by the item name, with the balance and price captions; with S the purchase runs. The purchase text comes from a template with one item-name slot at data address 0x319528. Talk (Interaction range and target choice) is tested only when there is no buyable item.
2. If Z, the first matching test below acts (its caption, and its action on S), and then the chooser ends:
   - state 57 carrying a Miamouse: "Drop Miamouse" (label 28); S drops it;
   - state 25 or a busy state: when the dialogue box is not open, a hero that is not busy returns to state 1 and the box is cleared; nothing else runs;
   - state 21 (unless its sub-mode is 7): "Stop Climbing" (label 18); S returns the hero to state 1 (22 when the stance flag is set), then requests state 7 and animation 67;
   - state 41: "Stop Climbing"; S returns the hero to state 1 (22 with the stance flag);
   - state 23: "Stop Sidling" (label 19); S leaves the sidle;
   - states 18 (stick centred and mode 40), 19 (mode 73), 26, and 27 (sub-mode 4): "Let go" (label 20); S leaves the state (to 1, or 22 with the stance flag);
   - stance group (states 7 to 10 and 43 to 46): the climb entry (state 21) or the second climb entry (state 41), with no button;
   - state 28: "Let go"; S leaves it; while Square is not pressed, cue 0x8D is kept playing;
   - stance group: the leap entry (state 28; SPEC-0033 Behavior 9), with no button;
   - states 29 (mode 40) and 30 (sub-modes other than 3, 5 and 6): "Let go"; S leaves the state;
   - none of these: when no talk claims and no buyable item is chosen, the caption is hidden (count 0), so a caption set earlier in the pass (for example "Save Game") is not shown.
3. If not Z: S with the weapon out (equipped state on) and no attack in progress puts the weapon away (equipped state off, with its cue on the hero bank), and the press continues. Then, if a talk claims or a buyable item is chosen, the chooser stops. Otherwise the first match below wins:
   - Catch Miamouse (label 27): a class-21 character within 4.0 units with facing cosine above 0.5 while the Miamouse game runs and the Miamouse's catchable word is set (it is idle or wandering, not fleeing, landing or caged); distance and facing are horizontal; S enters state 57;
   - climb entry (state 21), with no button;
   - second climb entry (state 41), with no button;
   - leap entry (state 28; SPEC-0033 Behavior 9), with no button;
   - Grab block (label 10) while a block is in front; S enters state 18; after pushing for the tick count in data word 0x2DD574 (15), the hero enters state 58;
   - Sidle, Fill water bucket and Use Rope Gun: their tests always fail (dead);
   - Grab Statue (label 23); S enters state 26;
   - Grab Door (label 22); S enters state 29;
   - door prompt: when the hero's door prompt id is 0 or more, that label is shown, and S sets the door-confirm flag to 1 (SPEC-0006, Door prompt and confirm);
   - then, if a caption other than "Feed petpet" has already been set this pass (for example the door prompt, "Save Game" or "Grab Wall"), the chooser stops;
   - Feed petpet (label 14); S enters state 49 when the pet has food, else plays cue 0xA2;
   - Read Book (label 31) while a book is pending (SPEC-0028 book stations); S takes the book (cue 0xAF) and the pending index is cleared. The prompt draws as the Square caption: slot 4 at (327, 395), the default caption colour, with the Square icon drawn in the same pass. It is not drawn while a quick-item panel is open (SPEC-0035 section 6.1);
   - then, if the weapon was put away this tick, the chooser stops;
   - Square with no target (SPEC-0033): outside the stance states 22, 39 and 40, S enters state 22 unless the hero is in state 20, 32 or 43 to 46, or the script-prompt flag is set this tick. In states 22, 39 and 40, nothing changes while Square is held, and the hero returns to state 1 when it is not held (hold to stay).

**Precedence (one Square press edge on one tick, game-state code 0, hero input enabled).** The steps that read the same edge, in tick order:

| order | step | outcome when it acts | relation to the others |
| --- | --- | --- | --- |
| 1 | save point (region tick of the current world's own region, before the hero update; SPEC-0025) | "Save Game" caption; S opens the save screen when the hero is not in a special state | independent. A later caption overwrites "Save Game"; the special-state hide removes it; it stops the idle list before Feed petpet, Read Book and Square-with-no-target |
| 2 | level script threads and the chest press thread (IsButtonPressed) | script actions | after the hero update, before the front end (SPEC-0030, Per-tick script step; SPEC-0026, main loop step 8.1.12); the chest press skips its prompt when a caption is already set this pass (SPEC-0048) |
| 3a | chooser: Buy | purchase | beats talk and every idle step |
| 3b | chooser: talk | callback thread | beats every idle step (the claim, even beyond 5.8 units); loses to Buy; a nearer lock-on leader takes the candidate |
| 3c | chooser: special-state branch (Z) | state exit | runs after 3a and 3b, so both can act on the same press; no idle step runs |
| 3d | chooser: idle list | first match | loses to 3a and 3b; a caption set earlier in the pass stops Feed petpet, Read Book and Square-with-no-target |
| 4 | door step (the hero's motion-record trigger pass) | travel or door callback | needs the confirm flag set by 3d on the same press (SPEC-0006) |
| 5 | item ask-about (the same trigger pass) | callback thread | independent (SPEC-0030) |
| 6 | dialogue box (front end) | page advance or finish | runs after the world steps, which have already used the edge |

One press can therefore act in more than one place: save screen and talk; save screen and door confirm; special-state exit and talk; door confirm and item ask-about; weapon away and any idle step.

### Line use

Confidence: High for the counts; Medium for their meaning. A line is shown only through the shared box when a call names it (Showing a level line), but a call whose index is computed at run time can name lines that a fixed-index count misses (Open question 59).

The 7,547 dialogue lines are held in 131 tables, one table per world pack. A line counts as used when a level-script call names its record number. A pack is reached when a route from the start of a new game loads it. The lines split as follows:

- 2,131 lines are named by a live call in reached packs.
- 223 lines are named only by calls in packs that no such route loads.
- 32 lines are named only by routines that are not called or registered from start-up, the per-tick update, exit, a hook or a registered callback.
- 5,161 lines are named by no call. Of these, 4,724 sit in reached packs and 437 sit in packs that no route loads.

The four groups add up to 7,547.

Twelve tables have no call at all, and they hold 22 lines between them: tables 1077, 1129, 1159, 1238, 1245, 1247, 1253, 1254, 1255, 1256, 1397 and 1398. Tables 1077 and 1129 hold the same lines.

Most unnamed lines sit in a few large tables. Twelve tables each have more than 100 unnamed lines. The largest has about 1,000 unnamed lines out of 1,019, and another has 720 unnamed out of 1,043. Some of the large unnamed groups repeat text held in other tables.

The 5,161 count is an upper bound on unused lines, for the reason given in Open questions (item 59).

### Progress flags

The progress flags are one bitmap of 743 bits, numbered 0 to 742.

1. Test(i): returns on or off. For an out-of-range index, returns off.
2. Set(i): for an out-of-range index, does nothing. Otherwise, if a watcher list exists, it posts a notification with flag i, the new value on and the bit's value before the set. Then, only when the bit was off and a world is loaded, it runs the per-flag cue of design table 2 (SPEC-0024, Progress-flag cues): row i's stinger value v plays stinger v - 1 at volume 1.0 through the game's music and stinger player (SPEC-0009) when v is 1 or more, and a non-zero journal kind sets the journal-notification flag with its phase at 0 (SPEC-0023, section 8). Then the bit is set (always, even with no world). Script SetGameState(i, true) uses this path. **[High]**
3. Clear(i): for an out-of-range index, does nothing. Otherwise it posts a notification with flag i, the new value off and the old bit, then clears the bit. Clear has no table 2 cue. Script SetGameState(i, false) uses this path. Clear posts on every call; unlike Set it does not first test that a world is loaded, so the clean side should skip the post when none is. **[High]**
4. Watcher callbacks: each flag i has its own watcher slot i (slot records are 24 bytes: callback, busy word, new value, old value, and two registration words). A notification is delivered only when i is below the list's slot count, the slot has a callback and its busy word is 0. Delivery stores the new and old values in the slot, starts the callback on its own thread and stores the thread in the busy word. Otherwise the notification is dropped: it is not queued and not retried. Every slot is reset (no callback, busy 0) when the world's script manager is set up. Whether the busy word returns to 0 when a callback finishes is not established (Low), so the clean side should free the slot when the callback ends. **[High for the drop rule]**
5. Level-scoped state is separate from the global bitmap. SetLevelState(i, value, world) changes one of 64 bits of that world's own state, with i below 64. QueryLevelState is not described.

#### Initial state

Confidence: High for the disc side.

The default progress-flag table is 743 32-bit entries at the virtual address given under Where it lives. On disc, that table lies after the end of the file-backed part of the boot program's data, so it is zero in memory. At boot, all 743 entries are zero. A confirmed new-game start resets the world and clears all progress flags, then applies the chosen start index: index 0 applies no stage rows, and indices 1 to 8 apply their stage rows (SPEC-0026; SPEC-0049, New game and start indices). Every new-game start then sets flag 707. Thus index 0 begins world play with flag 707 on and the other flags off; the other start indices begin with their table flags plus 707. A loaded save restores its saved bitmap (SPEC-0026; SPEC-0025). On the new-game Act 1 path, pack 909's opening update then sets flag 1 and clears flag 706 before queuing movies 88 and 1 (SPEC-0026, Movie play sites). **[High for these initialization and opening writes]**

#### Stage table

Confidence: High for behavior; Medium for the content of the table (run-time memory).

Changing to a stage id applies a fixed list of progress flags, in two groups. The lists are held in run-time memory, not on disc.

1. Pair entries (stage id, flag index): each flag is set when the flag index is zero or more.
2. Triple entries (stage id, flag index, value): when the value is positive and the flag index is zero or more, the flag is set through the world's notify path, but only when the caller asks for notification. The debug stage command and the New game path both use the stage table; the debug command passes the notify argument, and New game passes it too.

The entries for stages 2, 3 and 4 in the Flags used by the engine table below are the flags that the stage change sets.

#### Flags used by the engine

| flag(s) | meaning in the engine |
| --- | --- |
| 220 (0xDC) | set by stage change to stage 2; progress tier test |
| 248 (0xF8) | set by stage change to stage 3; progress tier test |
| 332 (0x14C) | set by stage change to stage 4 |
| 216 (0xD8) | gates the variant key lookup below |
| 273 (0x111) | set when a boss world starts (Medium) |
| 107 to 110 | set by a counter-driven event, one of four depending on a count (Medium) |

Stage change: stage 2 sets flag 220, stage 3 sets flag 248, stage 4 sets flag 332. Stage 2 also switches the active hero (Medium).

Progress tier (passed to a world setting whose use is not identified): tier 0 if flag 220 is off; tier 1 if flag 220 is on and flag 248 is off; tier 2 otherwise.

#### Variant key check

When flag 216 is on and the world state holds a variant key k (1 to 13), the variant is chosen by the key check below. Otherwise the variant key check is not used.

| key | result is 1 when |
| --- | --- |
| 1 | flag 83 is on and flag 5 is off |
| 2 | flag 10 is on and flag 322 is off |
| 3 | flag 64 is on and flag 283 is off |
| 4 | flag 57 is on and flag 58 is off |
| 5 | flag 74 is on and flag 263 is off |
| 6 | flag 85 is on and flag 298 is off |
| 7 | flag 92 is on and flag 94 is off |
| 8 | flag 248 is on |
| 9 to 12 | flag 332 is on and flags 100, 102, 104 and 106 are off, one for each key in order |
| 13 | always 1 |
| other | 0 |

For character rendering, a true result of the variant key check enables the per-character colour override at bind: the two skin-colour channels are set to RGB (155, 123, 193) and (101, 81, 126), while the other colour channels keep their placement values (SPEC-0040, Behavior 26). This check is not an alternate dialogue-line selector. Which runtime code writes the world's variant key remains open.

#### Quests

A quest row is one journal entry. The rows are table index 1 of the item-table file (archive resource 533): 238 rows of 36 bytes, read by row index. The owner of the quest rows is SPEC-0049 (Quest rows; Quest steps and the journal); this section is a summary. Confidence: High for the fields and the listing rule.

Checked on disc: 238 rows. Row kind 0 on the 158 rows 0 to 157 (main story) and 1 on the 80 rows 158 to 237 (side quests). Start and done flag ids run from 0 to 675; no row has −1 in either field. Stage 0 on 48 rows, 1 on 115 and 2 on 75. Scene id −1 on 40 rows. Every title and every objective is readable.

| Offset | Size | Type | Meaning |
| --- | --- | --- | --- |
| +0x00 | u32 | u32 | Row kind: 0 = main story (158 rows), 1 = side quest (80 rows) (SPEC-0049). |
| +0x04 | u32 | u32 | Start flag: a progress-flag id (0 to 742; values 0 to 675 on disc). −1 means none in the code; no disc row uses it. |
| +0x08 | u32 | u32 | Done flag: a progress-flag id, same rule. |
| +0x0C | pointer | text | Title (a short task name), a NUL-terminated string in the same resource. |
| +0x10 | pointer | text | Objective text (longer), a NUL-terminated string. All 238 readable. |
| +0x14 | u32 | u32 | Stage, 0 to 2 on disc (48, 115 and 75 rows): 0 = no map marker, otherwise 1 + row kind (1 on main rows, 2 on side rows). Fixed disc data; nothing writes it at run time (SPEC-0049; SPEC-0035 14.4). |
| +0x18 | f32 | float | Map position, first coordinate (for example −1840.0). |
| +0x1C | f32 | float | Map position, second coordinate (for example 1115.0). |
| +0x20 | s32 | s32 | Scene id, −1 on 40 rows. The map marker shows only when this equals the current scene. |

Listing rule for a quest row (High; SPEC-0049, Quest steps and the journal): the row is ignored when either flag id is −1. Otherwise it is listed when its start flag is set and its done flag is clear. The show-all setting is a debug override whose word is zero on disc, so the flag test always applies. Rows of every stage are listed. A level script completes a step by setting its done flag; nothing else changes the row.

Test vector: a row with start flag 5 and done flag 6, on an all-off bitmap: not listed. Set 5: listed. Set 6: not listed.

A quest has no reward or objective table of its own. A quest's state is held by progress flags, which scripts set and clear (the flag rules above).

### Record search by run (owner: SPEC-0006 and SPEC-0030)

Owner: SPEC-0006 (tagged tables) and SPEC-0030 (host call GetAssetGroupCharacterRange).

This is not a text routine. It finds the run of entries with a given slot byte in the entity-id table (tag 0x12) and gives the first and last index of that run; it is the body of GetAssetGroupCharacterRange. A second form of the search, on the placement-id table (tag 0x13), has no caller. A clean dialogue implementation needs neither (High).

### Grouped record lookup (owner: SPEC-0006 and SPEC-0030)

Owner: SPEC-0006 (region pool and entity-id table) and SPEC-0030 (host call CharacterIndex).

This is not a text routine. It turns a character's spawn counter back into an entity id: group g = counter / 50 − 1 and ordinal o = counter mod 50; g = −1 gives o itself, and otherwise the result is the entity-id table entry under region pool key g with ordinal o, or −1. It is the body of CharacterIndex (Medium-High).

### Number formatting by category (owner: SPEC-0029)

Owner: SPEC-0029 (monster-parameter schema).

This is not a text routine. It is the value printer of the monster-parameter schema, part of a debug tuning editor that has no caller, so it never runs in retail. A clean implementation needs none of it (High).

### Text message ids for the shared box (owner: SPEC-0034 3.16)

Owner: SPEC-0034 3.16 (the one-shot effect at a point).

This is not a text routine and the ids are not message ids. It is a one-shot effect spawn: it takes shared effect group 3, picks effect definition 222 when its flag argument is 0 or 220 when it is 1, places the effect at a given position, starts it and detaches it (mode 2), so the effect plays once and frees itself. Nothing is shown in the dialogue box (High). SPEC-0034 3.16 holds its rules and the look of the two definitions.

### Accent rule (Latin-1 vowels)

Confidence: High. Input is a base character and one accent code.
- Four accent codes exist: 0x88, 0xB4, 0x60 and 0xA8 (game font codes; data address 0x321550). The fonts draw them as circumflex, acute, grave and diaeresis. The accent index is the position of the code in that list, or 4 when the code is none of them.
- The vowel index is the position of the base character in the ten-letter sequence A E I O U a e i o u (data address 0x321558), or 10 when it is not a vowel.
- When both indices are valid, the result is the accented letter at position accent index x 10 + vowel index in the 40-entry Latin-1 table at data address 0x321528 (four accents, ten vowels each). The clean side reads the 40 codes from that address.
- Otherwise the input character is returned unchanged.
- Reverse lookup (strip accent): if the character is one of the 40 accented entries, the result is the plain vowel at its position in the ten-letter sequence (index mod 10); otherwise the input is returned unchanged.

### Line break search

Confidence: High.

The program also holds a second break routine that treats only NUL and space as candidates and returns the limit when none is found. It has no caller and no stored address, so it is never used. The clean side need not implement it. The break rule in use is the one of Text layout and glyph drawing (rules 3 and 4): spaces and underscores are break points everywhere.

## Edge cases and error handling

- Out-of-range flag index: test returns off; set and clear do nothing, and no notification is posted.
- Out-of-range table index: no record is returned; the caller shows nothing from the table.
- Watcher slot busy: the notification is dropped. It is not queued and not retried (Progress flags, item 4; High).
- Quest row with a start or done flag id of −1: never listed in the journal, whatever the gate or the show-all setting (both ids must be non-negative for listing). This branch never applies to the disc data, since no row has −1.
- Quest row with stage 0: no map marker, even when its scene matches.
- Control types with no handler (and type 33, whose mask 4 never runs): skipped during reveal and draw nothing. A clean implementation must still advance past them using their length byte L. Types 23 and 24 occur with L = 9 as well as the longer lengths, so the length byte must be read for every code.
- Word wrap with no break opportunity: a word wider than the wrap width breaks just after its first overflowing glyph, so that line is wider than the wrap width by up to one glyph.
- Reveal back-scan: a back-scan for a space before the reveal frontier is computed, but its result is discarded, so it has no effect. The reveal uses the same line breaks as the line counter.
- Justification: a line is stretched only when it ended at a candidate and has another candidate before it, so a line with no gaps is never stretched. The divisor counts control-code bytes, so codes in a line reduce its stretch.
- Text ending with a stop (6 records on disc) leaves an empty last page: the box shows no text with "Finish" and needs one more Square press. 8 records have a last page holding only codes, and one record has a code-only middle page.
- Leftover authoring markup in a few lines (emote names ending in "]", with an underscore): "]" has no glyph in font 0 (8 uses) and draws nothing, and the underscore draws the SELECT glyph.
- The page buffer is 512 bytes with no bound check; a page longer than 511 bytes overruns it in the original (none on disc).
- A page whose line count reaches 1,000 logs an error (unreachable on disc).
- Kind 0x13 with a count of exactly 18 gives one item (quirk; Dialogue control records).
- Kinds 0x11 and 0x12 set and clear flags without notifying the watchers, unlike Set(i) and Clear(i) of Progress flags.
- Reproducible quirk: the speaker rule treats any name that begins with a control code as the hero, so a type-32 code alone is shown as the hero name.
- Memory-card messages use the same box as dialogue but are not table lines. Their text comes from design table 9 (see Memory-card and controller messages).
- The debug commands that set and clear flag ranges and the debug command that sets the stage exist in the binary. Whether they are reachable in the retail build is not established.
- Interaction prompt chooser with nothing to do: there is no default prompt. On the special-state branch the caption is hidden when no talk claims and no buyable item is chosen, so a caption set earlier in the pass (for example "Save Game") is not shown. On the idle path a Square press with no target goes to the stance rule (state 22).
- **Talker reorder tolerance (quirk)**: the reorder swaps entries i and j (j later) when distance(j) < distance(i) + 1.0, so a talker up to 1.0 unit farther can become leader. Reproduce it.
- **Sticky leader**: a leader that stays eligible is kept for 90 ticks even when a nearer talker is eligible.
- **Lock-on steal**: a lock-on leader that is nearer than the talker leader, or at the same distance, becomes the candidate; with no talk callback it starts nothing and the idle list runs.
- **Out-of-range claim (quirk)**: a candidate with a registered talk callback claims the press even beyond 5.8 units (up to the 20-unit list range), so no door prompt, Grab, Feed petpet, Read Book or Square-with-no-target step runs while it is the candidate. While its callback thread runs, the same holds.
- **Caption overwrite**: the last caption writer in a pass wins; a hide by the special-state branch removes a caption set earlier in the pass.
- **Dead prompts**: Sidle (11), Fill water bucket (12) and Use Rope Gun (24) are in the chooser, but their tests always fail in this build.
- **LevelMsg on a busy hero**: a busy hero is not put into state 25; the line is still shown. LevelMsg never saves the hero's previous state, so a later EndConversation can only restore a state saved by SetConversationTarget.
- **LevelMsg out of range**: nothing is shown and the call does not wait, even with wait = 1.
- **Stale emotes (quirk, Medium)**: the emote queues are cleared only at spawn and by SetConversationTarget (and its WithOrient form). Hero emotes requested while the hero is busy (not in state 25, so the consumer does not run) stay queued, up to the 4 newest (a push onto a full queue drops the oldest). They play at the next level message that puts the hero into state 25. Reproduce.
- **Partner emotes with no partner**: a line shown by LevelMsg with no talk partner runs its kind 0x10 and 0x16 records but queues nothing; the hero kinds 0x17 and 0x18 still queue on the selected hero.
- **One press, several actions**: one Square edge can open the save screen and start a talk, confirm a door and fire an item ask-about, or put the weapon away and act in the idle list (Interaction prompts, Precedence).
- **Leap with no pad input**: a hero in states 7 to 10 or 43 to 46 with a kind-3 candidate that passes the probes, and with no climb target (neither climb entry finds one), goes to the leap state even when no button is held (Behavior 9 of SPEC-0033; reproducible, Medium-High).
- **R2 or Circle edge with the mode word outside 0 and 1**: the edge test fails even when the bit is set (Medium).
- **Square hold in the stance states**: in states 22, 39 and 40 the hero stays while Square is held and returns to state 1 on the first tick it is not held; the test runs every tick. A level prompt (script-prompt flag) blocks the state 22 entry for that tick.
- Purchase prompt for an item with no name: behaviour not established.
- Text window: its lines and the prompt row are drawn only while the open check passes.
- Chooser first: the chooser runs before the rest of the hero's button chain and ends the chain on any Square or attack edge, so the later chain steps do not run on such a tick (SPEC-0033 Controls, rule 5).
- Input disabled: when the hero ignores the pad (input-enable setting off, or the game mode outside 0, 3 and 4), no Action press reaches the target rule (SPEC-0033 Controls, rule 2).
- Line with no text: nothing is drawn and the result is 0 (Line alignment, step 1).
- Centre mode with an odd width: the half width rounds toward zero, so the shift is one unit less than the exact half (step 3).
- Start x below 16 in any mode, including mode 0: the start x is raised to 16 before drawing (step 4). A right-aligned line wider than its start position therefore starts at 16, not at a negative x.
- Mode 3 lifts the text by its measured height (newlines × line height + the last glyph's quad height), not by a glyph advance (step 3). It is used only by DisplayScreenMessage, whose calls are never reached.
- Speaker test and case: "HERO", "hero" and "[HERO]" are also treated as the hero placeholder, because the compare ignores the case of A to Z (none occurs on disc).
- **Same-text reopen (Medium):** every substituted text lives at the same work-area address. When the box opens a text at the address it already shows (the same line again, or a second substituted message while one is open), it keeps the stop, choice, result and speaker-override state that a new text would clear. It still restarts at the first page and resets the reveal. On disc the table-9 messages hold no page stops, so the visible effect is limited to the kept choice and override state. No disc path shows a kept choice: every reopen that follows a pending choice is itself a choice message, which sets the choice again (Open question 65).
- Substitution slot index: k is not range-checked; only k = 0 and 1 occur on disc (Substitution codes).

### Edge cases: accents and breaks

- Accent: a non-vowel with a valid accent code returns the input. A vowel with an unknown code returns the input.

## Test vectors

Synthetic inputs, not copied from the disc.

- Control code: n = 3 (one argument group), L = 11, type 2. The argument bytes hold two escape pairs and two plain bytes and decode to four values (1, 0, 2 and 3). The next code starts 11 bytes on.
- Stop code: n = 2, L = 5, type 4, value 6. It is a page stop with no arguments.
- Advance rounding: an advance of 2.4 gives 2; 2.5 gives 3; −2.5 gives −3.
- Line break, synthetic font: letters with advance 8, space and underscore with advance 4, wrap width 30. Text "aaa aaa aaa" (spaces at indices 3 and 7): running widths reach 24, 28 (after the space), then 36 at the next letter, so the break falls after the space at index 3; the second line breaks after the space at index 7; the third line is "aaa". Text "aaa_aaa_aa" (underscores at indices 3 and 7): the line-break search gives the same breaks as the space version, after indices 3 and 7, and the last line is "aa".
- Glyph quad (synthetic): pen x 100, baseline 200, x bearing 1, quad width 17, quad height 21, descent 2, s = 1: the quad runs from x 101 to 118 and from y 181 (top) to 202 (bottom). Texel bounds 2,600 and 2,856 (sixteenths) give texture u from 162.5 to 178.5 texels; the bounds set the texture coordinates only, not the quad size.
- Font 0, code 0x41 (from the disc): map value 22, glyph index 21: advance 17, quad 17 × 21, x bearing 0, descent 2; texture bounds (162.5, 36) to (178.5, 57) texels.
- Composite, font 0, code 0xE9 (from the disc): map value 119, so k = 119 − 1 − 102 = 16: base "e", accent 0xB4 (acute); the advance is that of "e" (15).
- Break with stretch: letters advance 8, space advance 4, wrap width 30, text "aa a aa aaa". Line 1 is "aa a" plus the consumed space, width 28, stretch min(2 / 4, 1.5) = 0.5 per glyph. Line 2 is "aa" plus the space, width 16, not stretched (one candidate). Line 3 is "aaa".
- No candidate: "aaaaaa" with letters of advance 8 and wrap width 30: line 1 is 4 letters (width 32), line 2 is "aa".
- Box: L = 1 gives yb = 335, frame (30, 294, 446, 66) and prompt row y 357. L = 2 gives yb = 325, frame (30, 284, 446, 87), name baseline 295, body baselines 325 and 346, prompt row 368.
- Reveal at rate 10 with step 1/30: one character every 3 updates. A page of 5 plain characters is complete in update 15; with Cross held, in update 3.
- Speaker: during a talk, a record whose speaker is "Brigson" shows the partner's name. A record whose speaker is "Hero" with hero flag 1 shows "Roberta" with frame colour (0, 222, 0).
- Choice: a Circle press gives result 1 and LevelPrompt true; a Triangle press gives result 0 and false.
- Command record, set one flag (synthetic): marker, n = 4, length 16, kind 0x11, V = 12 (flag 10, reveal pass); first group the byte 0xA0 then escape pairs for 1, 0 and 0, decoding to 416; second group four bytes 0xFF, decoding to −1. Result: flag 416 on; no watcher is told.
- Set a range: kind 0x11 with arg0 = 100 and arg1 = 103 sets flags 100, 101, 102 and 103. With arg0 = 103 and arg1 = 100 only flag 103 is set.
- Money reward (synthetic): kind 0x13, length 19, V = 20 (flag 18); arg0 = 135, arg1 = 500: the party gets 500 Neopoints.
- Count quirk: kind 0x13 with arg0 = 50 and arg1 = 18 gives one item 50, not 18.
- Reveal count: progress 2.0 with rate 4 gives floor(8 + 0.001) = 8. Progress 0.2 with rate 5 gives floor(1.0 + 0.001) = 1.
- Progress tier: flag 220 off, flag 248 on → tier 0. Flag 220 on, 248 off → tier 1. Flags 220 and 248 on → tier 2.
- Variant: flag 216 on, key 2, flag 10 on, flag 322 off → 1. Set flag 322 on → 0.
- Range: distance 5.8 with Action pressed → does not fire. Distance 5.79 without Action → hint 8 shown, does not fire. Distance 5.79 with Action → fires.
- Out of range: test(1000) → off. Set(1000) → nothing changes, no notification.
- Set then clear: set(248) posts on, bit becomes on; clear(248) posts off, bit becomes off.
- Quest row: start flag 5 and done flag 7, flag 5 set and flag 7 clear: listed; with flag 7 set: not listed. A flag id of −1 in either position: not listed, even with the show-all setting on.
- Quest row: stage 0 with a matching scene: no map marker. Stage 2 with a matching scene: marker shown.
- Button bits (synthetic): held word with bit 7 set and edge word with bit 7 set: Square held and Square edge are both true. Held word with bit 5 only: R2 or Circle held is true and R2 or Circle edge is false. Edge word with bit 5 set and mode word 2: R2 or Circle edge is false.
- Leap without a button (synthetic): hero in state 8, pad word zero, the climb entry (state 21) and the second climb entry (state 41) find nothing, leap predicate passes. The hero goes to state 28.
- Stance group order (synthetic): hero in state 43 and the climb entry finds a target. The hero goes to state 21 and the second climb entry (state 41) does not run.
- Action edge (synthetic; Medium): Square edge with a range-gated target 4.0 units away and no busy state: the target fires once. The same tick's Cross edge does not also run, because the first acting step ends the chain.

- Line alignment, centre: start x 100, width 33, mode 1. The start x becomes 84 (100 minus 16).
- Line alignment, right: start x 100, width 50, mode 2. The start x becomes 50.
- Floor: start x 10 in mode 0 gives 16. Start x 10 in mode 2 with width 50 gives −40, which the floor raises to 16.
- Mode 3: start (50, 200), a one-line text whose last glyph has quad height 17, gives start (50, 183) before the floor. A two-line text in a font of line height 14 whose last glyph has height 12 gives (50, 174). The x value is unchanged.
- Measure: letters of advance 8, spacing −2, scale 1: "aaa" gives W = 18. With spacing 0, W = 24.
- Compare: "hero" against "Hero" gives 0 (match). "Heroes" against "Hero" is not 0. "[HERO]" against "[Hero]" gives 0.
- Substitution: slot 0 = "1". The text "slot ", then a substitution code (marker, n = 3, L = 13, type 5, V = 34, then four escape pairs for the values 0, 0, 0 and 0), then "." gives "slot 1.". The page length after substitution is 7 bytes.
- Substitution: the same code with first argument 1 (the first escape pair for 1) and slot 1 = "96" gives "96" in place of the code. Two substitution codes in one text are both replaced, left to right.
- Substitution slot number: setting a slot to the number 0 gives "0"; setting it to 1234 gives "1234".

### Test vectors: talk, prompts and level lines (synthetic)

1. Caption lifetime: "Talk" is set on pass n only. It is drawn on pass n and the count is 0 after the draw; pass n+1 shows nothing.
2. Talk: hero idle, talker leader at 4.0 units, entry flag 1, no buyable item: "Talk" is shown; S starts the callback once. A second S while the callback still runs shows nothing and starts nothing.
3. Range edge: the same at 5.8 units: nothing is shown and nothing starts. A door prompt 13 posted on the previous tick is not shown, and S does not set the confirm flag (the claim).
4. Buy first: a buyable item and a talker leader at 3.0 units with a callback: "Buy" with the item name is shown; S buys; the callback does not start.
5. Lock-on steal: a lock-on leader with no talk callback at 3.0 units and a talker leader at 4.0 units: no talk, and the idle list runs. Talker at 2.9 units: talk. Both at 3.0 units: the lock-on leader wins.
6. Talker order quirk: eligible talkers at 5.0 units (entry 0) and 5.5 units (entry 1), on a reorder: 5.5 < 5.0 + 1.0, so they swap and the leader is the one at 5.5.
7. Sticky leader: the leader stays eligible and is kept for 90 ticks while a nearer talker is present; on the 91st tick the list is reordered.
8. Default door, exterior current: on tick t the door step posts 13. On tick t+1 "Enter" is shown; S on tick t+1 sets the confirm flag, and the door step on tick t+1 clears it and travels.
9. Interior current: the door prompt is 26 ("Exit").
10. Script door with context 13: S on the press tick starts the callback, and the confirm flag stays 1. When the hero later touches a default door, it travels on the first contact tick, with no prompt.
11. Script door with context 0: the callback starts on the first contact tick, with no press.
12. Save point and talk, both 3 units away: S opens the save screen and starts the talk callback; the caption shown that pass is "Talk".
13. Save point and book station: "Save Game" is shown and "Read Book" is not; S opens the save screen only, and the pending book index stays set.
14. LevelMsg(500) on a 100-line table with wait 1: returns at once; the hero's state is unchanged and nothing is shown.
15. LevelPrompt(500, W, −1): returns false at once. LevelPrompt(3, W, −1) answered with Circle: true.
16. SetConversationTargetWithOrient(id, 1, 0): the entity's partner is the hero and the hero has no partner. A line whose speaker is "Brigson" shows "Brigson", and the hero does not turn.
17. Hero in state 25, the box closes, and the chooser runs before the next line: the hero goes to state 1.
18. Idle, no target, S: state 22. Next tick, Square still held: state 22. Square released: state 1. The same press with a script prompt up that tick: no state change.
19. Weapon out, S, door prompt 13 pending: the weapon is put away and the confirm flag is set in the same tick.
20. DisplaySquareButtonContext(4) while the word at 0x35C9A0 is 1: nothing is shown and the script-prompt flag is unchanged.
21. Emote target: SetConversationTarget(id, 0), then a line with a kind 0x10 record (id 51) and a kind 0x17 record: the 0x10 entry goes to the entity's body queue and the 0x17 entry to the selected hero's body queue. The same line shown by LevelMsg with no partner: only the 0x17 entry is queued.
22. Kind 0x16 with id 127 during a talk: the partner's face queue gets (no directory, position -1, 127); the face animation 127 plays and no clip starts.
23. Stale emotes: the hero is busy (state 48) when LevelMsg reveals five kind 0x17 records; the hero stays out of state 25 and the four newest stay queued. A later LevelMsg that puts the hero into state 25 plays them in order.
24. Rotation (Proximity talk frame and line rotation): one callback shared by entities 1 and 2, with branch 1 (flag F set) a first-once rotation of lines 10, 11, 12 and branch 2 (F clear) a cycle of lines 20, 21. With F clear: talks to entity 1, entity 2, entity 1 show 20, 21, 20. Set F: the next talks show 10, 11, 12, 11, 12. Clear F again: the marker differs, so the next talk shows 20. With a shared counter (no marker) instead, the first talk after setting F, made when C is 1, shows 11. A branch that chooses no line: no conversation starts and the hero's state is unchanged.
25. LevelMsg wait (synthetic): LevelMsg(3, W, 0, 0) on a valid line returns in the same tick with the box open; LevelMsg(3, W, 0, 1) returns in the tick after the Finish press.
26. Slow-down (synthetic): if every main-loop pass takes two vertical blanks, a page of 5 plain characters is still complete in update 15, which is now 0.5 s.
27. Card removed during a monitored choice message: the routine returns -1 with the box still open. The next message, a choice message, shows its own two labels, and its result is -1 until Circle or Triangle is pressed.

### Test vectors: accents (synthetic)

- Accent: base E with the second accent code gives the accented E of accent index 1 (position 11 of the 40 entries). Base X with any accent code returns X. Reverse lookup of an accented entry returns its plain vowel.

## Open questions

Numbers of removed questions are not reused.

1. What the type-3 handler does (no type-3 code occurs on disc).
2. Resolved: **Front-end pass rate while a box is up.** One box update per main-loop pass, at one pass per vertical blank with no catch-up and a fixed step, so 60 per second unless a pass overruns; the reveal then slows with the game (Reveal and paging, Pacing). **[High]**
3. After the final page. Resolved: a Square press clears the text, and the box host state then plays cue 0x97 and returns to play (Choices, input and release).
4. Partner block match (Medium): the partner block is matched to the placement record's name (+0x20) and colour (+0x40) by the disc layout, not by the routine that writes the block.
5. Resolved: a raw string has no name and the colour (128, 128, 128), with frame alpha 45.
6. Box buttons. Resolved: Square advances, Circle answers 1 (cue 0x98), Triangle answers 0 (cue 0x99) and holding Cross reveals five times faster, all read from pad slot 0 (Reveal and paging; Choices, input and release).
7. UI label table. Resolved: design table 13 of archive resource 533 (SPEC-0024); the box uses labels 0x10, 0x11, 0x21 and 0x22 (Dialogue box layout).
8. Resolved in part: one side-quest conversation crosswalk is established in SPEC-0049, Flags 592 and 593, and OQ20. In proximity talks, flag 593 selects between the two line sets for eight callbacks in resource 777, four in 1017 and one in 1300; the resolved quest step that sets the flag and the identified aftermath lines are documented there. This is script-specific selection, not a general dialogue-engine rule. The remaining per-module line-index and flag mapping stays with the level-script specs.
9. Resolved: the boot bitmap is zero; a confirmed new-game start clears all flags, applies the selected start-index stage rows (none for index 0), then sets flag 707. The Act 1 opening in pack 909 next sets flag 1 and clears flag 706. A loaded game instead restores its saved bitmap. The full start-index and stage-row behavior is in SPEC-0049, New game and start indices (SPEC-0026; High).
10. Resolved: the per-flag table is design table 2 of archive resource 533 (SPEC-0024, Progress-flag cues): per flag a journal kind and a stinger value. The weight-1.0 call plays stinger v - 1 at volume 1.0 through the game's music and stinger player (SPEC-0009). It runs only when the flag rises with a world loaded (Progress flags, item 2). **[High]**
11. Resolved: the distance for the 5.8 test is the 3-D length between the hero's and the candidate's positions (Interaction range and target choice).
12. Resolved: a notification for a busy slot is dropped, not delayed; Clear posts on every call (Progress flags, items 3 and 4). The world manager polls callback-task completion during its update and clears the slot’s busy handle after the callback task exits, so the slot can accept a later notification. **[High]**
13. Resolved in part: for character rendering, a true result of this check enables a bind-time override of the two skin-colour channels to RGB (155, 123, 193) and (101, 81, 126); the other colour channels retain their placement values (SPEC-0040, Behavior 26). This does not select alternate dialogue text. Still open: which runtime code writes the variant key in world state.
14. Resolved: the writer classes and dialogue-control record census are in SPEC-0049 (Who writes the flags). The only kind-0x11 dialogue control record sets side-quest start flag 416; no kind-0x12 record occurs, and dialogue control records set or clear no main-story flag. A separate active level-script talk callback also sets 416: in Market Town (module 991), Prince Tourin (entity 14) sets it on the first eligible talk when flag 417 is clear and 416 is clear. Quest row 173, The Gelert Family Crypts, uses 416/417 (SPEC-0049, quest table). Thus, from a clean-zero flag bitmap, if play reaches that module and invokes this callback, the row passes its start-on/done-off listing gate after the talk. The callback later sets 417 and removes item 12 when 416 is already set and item 12 is held. This establishes a script-local clean-start path, not that a particular travel order or encounter is guaranteed. The separate dialogue control record in pack 772 is another writer of 416. [High for branch and row mapping; route conditional]
15. The purchase rule itself, which belongs to SPEC-0050 (SPEC-0035 open question 1).
16. Resolved: the show-all setting is a debug override whose word is zero on disc, and the test is "start flag set and done flag clear" (Quests; SPEC-0049, Quest steps and the journal; High). Concrete reachability boundary: Angry Mozitos row 204 (flags 521/522) stays unlisted from a clean new game because the recovered boot, stage-table and level-script census finds no setter for either flag; see SPEC-0049, Open question 9. This does not cover externally modified saves.
17. Resolved: the two linked ids are the row's start flag and done flag, progress-flag ids in the 743-bit bitmap (Quests; SPEC-0049; High). Applied to main-story row 3, Talk to Mum, the gate is flag 110 on and flag 111 off. The static farm path supports this clean-start sequence: the Whinny group routine sets flags 107 through 110 as the count of penned Whinnies rises to three; then Patricia's first task talk with 110 on sets 111. Thus row 3 becomes listed after all three Whinnies are penned and before that talk, and is hidden after the talk sets 111. Pack 909 resets an incomplete roundup on activation but preserves the all-four-flags-complete state. This describes the recovered gates and callbacks; it does not claim a live-play run or prescribe a penning order (SPEC-0049, Who writes the flags; SPEC-0045 pack 909; SPEC-0058 Act 1 progression).
18. Resolved: +0x00 is the row kind (0 main story, 1 side quest). The stage is 0 (no map marker) or 1 + row kind; it is fixed disc data and no event moves it. A step is completed only by a script setting its done flag (Quests; SPEC-0049; High).
19. Line step. Resolved: 21 pixels (the font 0 line height), and all lines of the page are drawn (Dialogue box layout).
20. Resolved: there is no choice cursor. Cue 0x98 plays on an advance and on a Circle answer; cue 0x99 on a Triangle answer.
21. Chooser order. Resolved: the full order is in Interaction prompts; every prompt id is a table-13 label, and 0x8D is an animation number, not a line request.
22. The purchase rule that follows an accepted prompt (SPEC-0050; SPEC-0021 shop rule).
23. Resolved: the hint is table-13 label 8 ("Talk"), the caption of the talk entry, which is the chooser's second step (Interaction range and target choice).
24. Resolved in part: font 2 is used by the in-play journal's quest-objective rows (the row drawer reads font id 2; SPEC-0035, section 14.3 and OQ129). The book reader uses a separate type-2001 font descriptor with the same code map as font 2, but its own glyph metrics; it is not the resource-532 font-2 descriptor. Font 1 is confirmed for dialogue-box prompt labels. Front-end text nodes select a font through their label binding, but a complete screen-to-binding census of font-1/2 users is not established here.
25. Resolved: the field at the array pointer plus 0x1C is the height field of glyph record 1, which is the font's line height. The array pointer points at the glyph records, with no header (Fonts).
26. Glyph bounds. Resolved: the four s16 values are the glyph's texel bounds in its sheet, in 1/16 texel (Fonts).
27. Whether any text uses composite letters: all accent anchors are zero on disc, and no dialogue on disc uses them.
28. Resolved: the stretch applies to every wrapped line that ended at a candidate and has another candidate before it; the leftover width is in canvas pixels (Text layout and glyph drawing, rule 6).
29. Resolved: the stage table pairs and triples are design tables 11 and 12, and the message table is design table 9, of archive resource 533 (SPEC-0024).
31. Prompt ids. Resolved: every prompt id is a table-13 label index (Interaction prompts, Prompt ids, Chooser order).
32. Chooser Square-hold rule. Resolved: it is the stance hold of states 22, 39 and 40 (Interaction prompts, chooser step 3); the R2 or Circle hold belongs to SPEC-0033 Controls.
34. Button-read gate. Resolved: the bit-0 global word gates the button reads, together with a game mode of 0, 3 or 4, or below 0 (Interaction prompts, chooser step 1; High).
35. Partly resolved: the chooser runs the grab-point leap predicate in two places (the stance-group step of the special-state branch and the idle list). The predicate requires the hero’s +0xA04 grab cooldown to be below one tick; this is the 60-tick grab-point cooldown specified in SPEC-0033, Behaviors 3A and 9. Still open: whether the two calls pass different meaningful arguments and which state the second path selects after the predicate succeeds. **[High for the cooldown gate; Low for the remaining call/state detail]**
36. Fold table. Resolved: the compare folds only A to Z to lower case, so the speaker test ignores case (Text comparison and page stop search; High).
37. Resolved: **Keyword scan.** It is the box's next-page stop search over the control-code handler registry: a position is a stop when it holds the marker and its type parses as 4 (Text comparison and page stop search). **[High]**
38. Floor: start x or width. Resolved: it is a left-margin clamp of the start x to at least 16, in every mode (Line alignment, step 4; High).
39. Mode-3 axis. Resolved: mode 3 lifts the screen y by the measured text height (Line alignment; High).
40. Offset source. Resolved: the four offset floats are the colour tint factors of the front-end text-node draw (SPEC-0035, Section G; High).
41. Resolved: **Byte storage of offsets.** The four values are the text's start colour R, G, B, A, stored and drawn as unsigned bytes; a value above 255 wraps to its low 8 bits (Text layout and glyph drawing, rule 12; SPEC-0035). **[High]**
42. Answered: see SPEC-0035, Section G (a front-end label binding record, not a glyph record).
43. Run search callers. Resolved: the search runs on the entity-id table for GetAssetGroupCharacterRange (Record search by run; SPEC-0006 and SPEC-0030; High).
44. Answered: see SPEC-0006 and SPEC-0030, CharacterIndex (region pool slot keys).
45. Grouped lookup tables. Resolved: the records are the entity-id table and the group keys are the world's region pool keys (Grouped record lookup; SPEC-0006 and SPEC-0030; Medium-High).
46. Answered: see SPEC-0029 (the debug value printer of the monster-parameter schema).
47. Answered: see SPEC-0029 (a schema field of a monster-parameter record).
48. Catalogue slot 3. Resolved: the routine is a one-shot effect spawn, not a message (Text message ids for the shared box; SPEC-0034 3.16; High).
49. Line-break conflict. Resolved: the routine that counts only NUL and space is never used (Line break search; Text layout and glyph drawing, rule 3).
50. **Accent table bytes.** Resolved: the 40 Latin-1 codes are at data address 0x321528, the four accent codes at 0x321550 and the ten vowels at 0x321558 (Fonts and Accent rule).
51. **Colour byte 3.** Resolved: byte 3 of the colour word is not used by the text box; the frame alpha is 45 whatever its value.
52. Kinds 0x16, 0x20 and 0x22 against the observed-codes table. Resolved: kinds 0x20 and 0x22 are the speaker override set and clear records, which have no arguments, and kind 0x16 is the partner face emote with no clip (Dialogue control records; High; Medium-High for the label).
54. Kinds 0x11 to 0x15 and 0x19. Resolved: they are the command kinds of Dialogue control records, none is a sound request, and the line tables hold 174 of them (166 of kind 0x13, 7 of kind 0x15 and 1 of kind 0x11).
55. Emote triggers. Resolved: emote records occur only in dialogue line text and run when the reveal of a shown line reaches them; there are no idle, hit or pickup lines and no barks or greetings (Dialogue control records, Who emotes; High).
56. **Hero record for kind 0x17.** Resolved: kinds 0x17 and 0x18 act on the selected (controlled) hero, and the Roberta shift of 183 is read from that hero, the same hero whose name a "Hero" line shows. **[High]**
57. **Kind 0x10 records outside line tables.** Resolved: the 29 extra kind 0x10 matches are not in any line table (11 lie inside texture chunks of packs 776 to 789), so the text box never reads them. **[High]**
58. Resolved: no. Within one reveal each control record runs exactly once (Reveal and paging, item 3a). A record runs again only when its page is revealed afresh, that is when the reveal of that page restarts from progress 0. **[High]**
59. Level-message dispatch. Resolved: LevelMsg always shows the named line in the shared box (Showing a level line); the 5,161 unnamed count is an upper bound because a call whose index is computed at run time can name lines that a fixed-index count misses (High).
60. Script thread order. Resolved: level-script threads and the chest press thread step after the selected hero's update and before the front end in each pass (SPEC-0030, Per-tick script step; SPEC-0026, main loop step 8.1.12; High).
61. Talker and lock-on flags. Resolved: the sets do not overlap in practice; the townsfolk routine clears its lock-on eligibility word +0x800 on its first tick (SPEC-0036), and a class-17 enemy is never a talker (Interaction range and target choice; Medium-High).
62. Resolved: **LevelMsg wait values.** Every LevelMsg that runs in play passes wait 0 or 1 (831 sites pass 1, 10 pass 0); no other value occurs. **[High]**
63. **Face id 0 during talks.** What face id 0 shows for townsfolk and heroes (a neutral or a talking face), so the face held at 0 while talking can be described visually. **[Medium]**
64. **Substitution slot 1 value.** The memory-card word that fills substitution slot 1 (the KB a save needs) has no known writer; the card manager set-up does not write it. No store in the boot program writes that word by its offset in the card manager or by its fixed address, and it lies in zero-filled memory. If no block copy fills it, the number shown is 0. The number shown on screen is therefore not established (SPEC-0025). **[Open]**
65. Resolved: **Same-text reopen with a pending choice.** A choice is left pending with the box open only when the card is removed during a card-monitored choice message, which then returns -1 at once. In every such path the next message is again a choice message, and the choice opener sets choice mode and clears the result (-1) after the open, so the player sees that message's own two labels and no kept answer. Non-choice card messages follow only an answered choice, which has already closed the box and cleared its text, so they open as new texts. The kept stop and speaker-override state stays without visible effect, since table-9 messages have no page stops. **[Medium-High]**
66. Resolved: **Entity id to placement and voice set.** The table above maps every active proximity registration in the 14 main-path modules to its disc placement name and resource-531 voice set, including the callback sharing groups. The fixed-partner cutscene IDs are also mapped; the lines without a partner queue no voice emotes. **[High]**
67. Resolved: **Farm row 4 clean-start gate.** SPEC-0049 row 4, Drawing Water, uses flags 111/112. From a clean bitmap, after flag 110 is reached, Patricia's task talk can set 111; while 112 is clear this makes row 4 pass the journal's start-on/done-off gate. Pack 909's pump interaction then succeeds only with 111 on, 112 off and item 6 (Empty Bucket) held; it consumes item 6, grants item 7 and sets 112, hiding row 4. The bucket has a placed pickup point in the farm and uses the normal pickup distance (SPEC-0058, vectors 14/18/43; SPEC-0045 pack 909; SPEC-0049). This is a conditional static sequence, not a live-play or route-order claim.
68. Resolved: **Farm row 6 clean-start gate.** SPEC-0049 row 6 is one of the Fixing the Water Pump entries and uses start flag 113 and done flag 114. The recovered pack 909 chain reaches 113 when, after the pump exchange has provided item 7 and set 112, Patricia's return talk consumes item 7 and sets 113; row 6 is listed while 114 is clear. Hubert's next talk sets 114, hiding row 6. The later repair step has a separate item-34 gate and sets 116/117. This is conditional on the listed flags and item/talk callbacks; it does not establish a live route or encounter order (SPEC-0049; SPEC-0045 pack 909; SPEC-0058 farm vector 19).
69. Resolved: **Farm row 8 item-dependent journal window.** SPEC-0049 row 8 uses start flag 116 and done flag 117. In pack 909's follow-up Hubert conversation with flag 114 on and 117 off, the no-item-34 branch sets 116 alone, so row 8 becomes listed while 117 stays clear. If item 34 is held instead, the branch sets 117 and 116 in the same interaction (and consumes the pliers), so row 8 does not pass the start-on/done-off gate after that interaction. Thus the row can appear when the player reaches Hubert before obtaining the pliers, but can be skipped when they already hold them. This is the recovered callback behavior, not a claim about which inventory state the normal route produces (SPEC-0049; SPEC-0045 pack 909; SPEC-0058 farm vector 19).
70. Resolved: **Farm row 9 cutscene flag window.** SPEC-0049 row 9, A Job from Dad?, uses start flag 118 and done flag 119. In pack 909, the Update path reached after flag 117 is set waits for any active dialogue to end, sets 118, then runs the house-job scene; a later part of that scene sets 119. During that interval, row 9's journal gate is true (118 on, 119 off), then the done write hides it. This is a static flag-state window; it does not establish that the journal can be opened or rendered during the scene, or that every clean-start route reaches the flag-117 prerequisite (SPEC-0049; SPEC-0045 pack 909; SPEC-0058 farm vector 37).
71. Resolved: **Farm-to-house row 10 gate.** SPEC-0049 row 10, A Job from Dad?, uses start flag 119 and done flag 120. Pack 909's flag-117 house-job sequence sets 119 near its end. On a first entry to the house in pack 951, the story thread sets 120 when it is clear, then plays FMV 2 and grants item 15. Therefore row 10 passes the journal gate in any state interval where 119 is on and 120 remains clear, and the pack-951 first-entry write hides it. This maps the two script transitions but does not establish that the normal clean-start route reaches the house in that order or that the journal is accessible during the scene (SPEC-0049; SPEC-0045 pack 909; SPEC-0058 farm/house vector 23).
72. Resolved: **House row 11 chest-and-parcel gate.** SPEC-0049 row 11, A Job from Dad?, uses start flag 120 and done flag 121. Pack 951's first-entry thread sets 120, so while 121 remains clear the row meets the journal gate. The upstairs chest consumes item 15 (Hubert's Chest Key) and drops item 98 (Wooden Sword); Dad gives item 32 (the parcel) only when the sword is held, and that handoff sets both 121 and 122. The parcel handoff therefore hides row 11. Because 121 and 122 are set together, the next row's 121/122 gate is not true after the handoff. These are script and item conditions, not proof of the player's route or journal access during the scenes (SPEC-0049; SPEC-0041 pack 951; SPEC-0058 step 4, vectors 23–24).
73. Resolved: **House row 12 simultaneous-start/done write.** SPEC-0049 row 12 uses flags 121/122. In pack 951, Dad's parcel callback requires the sword (item 98), then grants the parcel and sets 121 and 122 in the same callback. The resulting bitmap does not satisfy row 12's start-on/done-off journal gate; the handoff instead makes flag 122 available as the start of the next Deliver Package row. This conclusion is limited to the recovered pack-951 handoff and does not rule out a separate writer or modified save bitmap (SPEC-0049; SPEC-0041 pack 951; SPEC-0058 step 4).
