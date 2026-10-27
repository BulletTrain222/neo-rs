# SPEC-0003: Texture chunks, palettes, and material texture state

- Answers: R-0003, R-0008
- Status: Draft
- Disc build: SCUS-97367 v1.00 (same fingerprint as SPEC-0001)
- Confidence: per section. Texture chunk layout and decoding: High. Material (shader list) texture fields and the texture-chunk selector: High for layout and lookup, Medium for the meaning of filter and blend settings in a modern renderer. Material record (passes, vertex programs, attribute masks, UV modes): High for layout, counts and what each vertex program does. Billboard program 3: High for the arithmetic, Medium for the corner UV orientation. UV modes: High for the arithmetic; environment-map space High for level geometry, Medium-Low for placed objects. Load-time handling of shader lists and texture chunks: High, including the two shader-list fix-ups. Sub-texture frame addressing: High for the cell arithmetic, the output and the frame source of both variants. Texture residency: High for the rules, Medium for the queueing detail. Run-time upload packet builder: Medium for the order; High for the chunking and the per-format sizes. Transfer-word helper: Low (its meaning conflicts with another reading). Material slot check: High. All are labelled in their sections.

## Summary

Textures live in chunk type 4 inside IDM chunk files (SPEC-0002). A texture chunk carries one block of pixel data together with a table of sub-textures. The game never decodes pixels on the CPU. It copies the block, unchanged, into the PS2 Graphics Synthesizer's local memory as 32-bit pixels, then samples the same memory as 8-bit or 4-bit indexed textures with 32-bit palettes. The GS memory layout itself does the "unswizzling". So the clean side decodes by emulating GS local-memory addressing (public PS2 hardware documentation: the GS User's Manual's PSMCT32, PSMT8 and PSMT4 page, block and column arrangements). Models do not refer to sub-textures by index. Their materials (the shader list, chunk type 2) carry ready-made GS texture register values, and those name a texture, its palette and its mip levels by GS memory address. Each material also carries a small record for the vector unit: one or two render passes, a vertex program and attribute mask per pass, and a UV mode per pass (none, environment map, scroll, or flipbook cell). With this spec the clean side can turn any texture chunk into RGBA images and resolve a material's texture, palette, mip chain, wrap, filter, blend and alpha-test settings, its passes and its UV animation.

## Where it lives on the disc

- Any IDM chunk file (SPEC-0002) can hold type-4 chunks. 686 top-level type-4 chunks exist: 585 inside ordinary chunk files and 101 as the single chunk of the texture packs (resources 577–677, the front-end pictures; see "Texture contents of fixed resources"). None are nested inside other chunks.
- Shader lists (type 2) appear 1,372 times: 215 as top-level chunks and 1,157 inside model-set wrappers (SPEC-0004). Each is followed in the same chain by the texture chunks it uses. Counts below marked "top-level" cover only the 215 top-level lists (15,592 groups); the nested lists hold 17,217 more groups (32,809 in all, counted per list occurrence).

## Layout

All fields are little-endian. "Payload" means chunk start + 16 + E, where E is the chunk's extra-header length (SPEC-0002).

### Texture chunk header bytes

- Chunk header bytes 12–15 are 01 01 00 00 in all 686 texture chunks. Nothing reads them; ignore them.
- The extra-header area (E bytes) is all zero except in the texture packs (577–677), where E = 112 and the bytes hold leftover build-tool values. Nothing reads them; skip them. E is not 0 in most texture chunks: it pads the payload to a 16-byte boundary of the file, so it runs from 8 to 124 in the chunks of model sets and outfits (for example 12, 28 and 108). Always locate the payload as chunk start + 16 + E; a reader that assumes the payload starts right after the 16-byte header sees a wrong texture table and finds no texture.

### Texture payload header (16 bytes)

| Offset | Size | Type | Meaning |
| --- | --- | --- | --- |
| 0 | 4 | u32 | Offset of the texture table, from payload start |
| 4 | 4 | u32 | Offset of the pixel block, from payload start |
| 8 | 4 | u32 | Offset of upload packet A, from payload start |
| 12 | 4 | u32 | Offset of upload packet B (always A + 0x100) |

The two upload packets are identical prebuilt GS transfer commands. At load the original points packet A at upload area 0 and packet B at upload area 1 (see "When textures are resident"). The clean side needs only two values from packet A, and both can also be derived from the texture table: see "Upload".

### Texture table

| Offset | Size | Type | Meaning | Seen on disc |
| --- | --- | --- | --- | --- |
| 0x00 | 4 | u32 | Upload segment count *n* | 1 in all 686 |
| 0x04 | 4 | u32 | Additional 48-byte record count included before the pixel data in the packet-builder descriptor; value 1 on all 686 tables | 1 in all 686 |
| 0x08 | 4 | u32 | Sub-texture entry count *m* | 2 … 291 |
| 0x0C | 4 | — | Unused | |
| 0x10 | 0x30 × *n* | | Upload segments, 48 bytes each | |
| 0x40 | 0x30 × *m* | | Sub-texture entries, 48 bytes each | |

Note that the sub-texture entries start at table + 0x40 regardless of *n*. This is consistent only because *n* = 1 everywhere on this disc. If *n* > 1 is ever met, treat the chunk as unsupported.

### Upload segment (48 bytes)

| Offset | Size | Meaning |
| --- | --- | --- |
| 0 | 4 | Bits 0–13: destination base block address (the segment TBP). Bits 16–21: buffer width in units of 64 pixels (= segment width / 64). Bits 14–15 and 22–31 are 0. The upload packet's BITBLTBUF carries the same width in its DBW field. |
| 4 | 4 | Bits 0–11: width in pixels. Bits 12–23: height in pixels. Bits 24–31: GS pixel format (PSM) of the upload. |
| 8 | 40 | Unused (0) |

On this disc every segment is PSM 0 (PSMCT32, 4 bytes per pixel), 128 or 256 pixels wide and 64 to 512 high. The pixel block is width × height × 4 bytes, row-major, starting at the pixel-block offset. It always fits inside the chunk.

The segment TBP is 0 in 584 chunks (every chunk that follows a shader list) and 0x2A00 in the 102 front-end chunks (resource 532 and the texture packs 577 to 677). In those 102 chunks every sub-entry TBP and every palette address is absolute (0x2A00 to 0x31F5), and so are the sprite or material register values that refer to them.

### Sub-texture entry (48 bytes)

| Offset | Size | Type | Meaning |
| --- | --- | --- | --- |
| 0x00 | 2 | u16 | Bits 0–13: base block address (TBP) in GS memory, in the same address space as the segment TBP (relative to the chunk's upload base where the segment TBP is 0; absolute in the front-end chunks) |
| 0x02 | 2 | u16 | Buffer width (TBW) in units of 64 pixels |
| 0x04 | 2 | u16 | Bits 0–11: real width in pixels. Bits 12–15: real height modulo 16. |
| 0x06 | 1 | u8 | Real height divided by 16, rounded down. Real height = 16 × this + (bits 12–15 of the previous field). |
| 0x07 | 1 | u8 | GS pixel format: 0x13 = PSMT8 (8-bit indexed), 0x14 = PSMT4 (4-bit indexed), 0x00 = PSMCT32 (a palette) |
| 0x08 | 8 | u64 | A GS TEX0 register value for this sub-texture: TBP, TBW and PSM equal the fields above. TW and TH (log2 of the sampling size) are the real width and height rounded up to a power of two. TCC = 1. TFX, CBP, CPSM, CSM, CSA and CLD are all 0. |
| 0x10 | 32 | — | Unused (0) |

Verified on all 22,606 entries on the disc.

Entry kinds:

- **Indexed textures**: PSMT8 or PSMT4.
- **Palettes**: PSMCT32 entries. They are always 16 × 16 (256 colors, for PSMT8 textures) or 8 × 2 (16 colors, for PSMT4 textures).
- **Mip levels** are ordinary entries too: each smaller level is its own entry with its own TBP.

Ordering: every table lists all texture entries first and all palette entries last (686 of 686 tables). The table itself does not say which palette goes with which texture. The material does (see below). One palette is often shared by several textures.

## Behavior

### Upload (how to build GS memory for one texture chunk)

1. Take a 4 MiB GS local-memory image, all zero. Use one image per texture chunk: sub-texture addresses are relative to the chunk's own upload base, and different chunks overlap in address.
2. Write the pixel block into it as a PSMCT32 image transfer: destination base block = the segment TBP, destination buffer width = segment width / 64, rectangle = segment width × height. Pixel (x, y) of the block is the 4 bytes at block offset (y × width + x) × 4, and it goes to the PSMCT32 address of (x, y). Equivalently, write at base 0 and subtract the segment TBP from every sub-entry address and every material or sprite register address before decoding. With base 0 and no subtraction, the 102 front-end chunks (segment TBP 0x2A00) decode from the wrong place. Upload packet A states the same destination width: its BITBLTBUF holds a DBW equal to width / 64, and its DBP is 0 on disc in the 584 ordinary chunks and 0x2A00 in the 102 front-end chunks.
3. Sub-textures and palettes are now read from that memory using their own TBP, TBW and PSM.

### Sub-texture frame addressing (cells of a grid)

Confidence: High for the cell arithmetic, the output and the frame sources. Both variants below use the same record shape and write the same output.

A texture can be cut into a grid of equal cells, for example the frames of one animation sheet. The hardware then shows one cell at a time (a flipbook). Each grid record is 16 bytes, read as four 32-bit floats, each converted to an integer by truncation toward zero:

| Offset | Size | Meaning | Notes |
| --- | --- | --- | --- |
| 0 | 4 | Frame count | Float. A count of zero or less changes nothing. |
| 4 | 4 | Grid rows | Float |
| 8 | 4 | Grid columns | Float |
| 12 | 4 | Frame divisor (variant 1) or table key (variant 2) | Float |

Disc examples: (16.0, 4.0, 4.0, 2.0) and (32.0, 4.0, 8.0, 1.0).

**Where the record and the output live.** The grid record for pass *p* (*p* = 0 or 1, see "Material record") is at group offset 0x00 + 0x10 *p*. The texture's own size is taken from the group's TEX0 for pass *p* (the first register set of the state block for pass 0, the second set for pass 1): bits 26–29 give log2 of the width (TW) and bits 30–33 give log2 of the height (TH). So texture width = 2^TW and texture height = 2^TH.

- Cell width = floor(texture width / columns). Cell height = floor(texture height / rows).
- A cell index *i* (from 0) gives column = *i* mod columns and row = floor(*i* / columns). Cell x = column × cell width; cell y = row × cell height.

**Which variant runs.** Group byte 0x2B (pass 0) and 0x2C (pass 1) hold the texture animator kind: 0 = none, 1 = variant 1, 2 = variant 2; any other value = none.

Output: a 64-bit GS CLAMP register value is written into the pass's UV-mode data of the material record for the current frame parity (group offset 0x60 + 0x10 *p* for parity 0, 0xB0 + 0x10 *p* for parity 1; see "Material record"):

| Bits | Field | Value |
| --- | --- | --- |
| 0–3 | WMS = 3, WMT = 3 | 0xF: region-repeat mode on both axes |
| 4–13 | MINU = cell width − 1 | U mask |
| 14–23 | MAXU = cell x | U fixed part |
| 24–33 | MINV = cell height − 1 | V mask |
| 34–43 | MAXV = cell y | V fixed part |
| 44–63 | Kept | Bits of the previous word, unchanged |

Each 10-bit value is taken modulo 1024. The following 64-bit word is set to the GS register number of the pass's CLAMP: 9 (CLAMP of the first register set) for pass 0, 8 (CLAMP of the second set) for pass 1. Region repeat samples texel ((u_t AND (cell width − 1)) OR cell x, (v_t AND (cell height − 1)) OR cell y), where u_t and v_t are the integer texel coordinates. So the mesh's UVs span one cell and the hardware moves them to the chosen cell. The word reaches the GS through UV mode 3 (see "UV modes").

**When it runs.** Once per main-loop pass, during the game update and before the draw, over every loaded shader list (both world slots, the sky dome's list, every character's list and two other global lists), for each group and each pass whose kind byte is nonzero, and only while the game-state code of SPEC-0026 (run-time word 0x35C9DC, recomputed each pass) is 0 to 8 (High). The draw therefore uses this pass's cell.
- Under code 9 (pause and the other freezing screens, and the streaming-load freeze) the flipbook cells hold.
- The render frame counter is still incremented on every main-loop pass (Medium-High). So UV scroll (mode 2), which reads the counter in the vector unit, keeps moving during a pause, and on resume a variant-1 flipbook jumps to cell floor(N / D) mod L.

**Variant 1 (frame passed in, with a divisor).**
1. If the count is zero or less, stop.
2. Divisor D = the record's fourth float, raised to at least 1.
3. Limit L = the smaller of the count and columns × rows.
4. Cell index = floor(N / D) mod L, where N is the render frame counter: an unsigned 16-bit counter (run-time word 0x3A4BB0, zero at boot) that increases by 1 at the end of every main-loop pass and wraps at 65536. At 60 passes per second a divisor D shows 60 / D cells per second.
5. Compute the column, row, cell x and cell y as above, and write the output.

**Variant 2 (frame index from a run-time table).**
1. If the count is zero or less, stop.
2. Store L − 1 (L as in variant 1) in the limit table under the key K (the record's fourth float).
3. Read the frame index from the frame table under K.
4. Column = index mod columns, row = floor(index / columns). There is no divisor and no modulo by the count.
5. Write the output as above.

The two tables are run-time integer arrays of the boot program, 4 bytes per key: the frame table at 0x36D020 and the limit table at 0x36D038. The frame table has room for 6 keys (0 to 5), so a key of 6 or more reads into the limit table: the frame read under key K ≥ 6 is the limit stored under key K − 6 (original quirk). Both tables are zero at boot. The only writers of the frame table are a routine that nothing calls (it would set a five-band damage stage of a character from its hit points, bands at 0.8, 0.6, 0.4, 0.2 and 0.0 of the maximum) and the Draik Skeleton's set-up, which sets keys 0 to 4 to 0. So in the original every variant-2 user always shows cell 0. Clean side: show cell 0. The intended design was probably that keys 0 to 4 follow the five hit-point bands (0.8, 0.6, 0.4, 0.2 and 0.0 of the maximum).

**Disc users of variant 2 (High for the groups).** 5 groups of a model-set wrapper in resource 1260 (the Black Knight fight region, SPEC-0012) select variant 2 for pass 0. Their keys are 0, 1, 3, 2 and 4 (groups 0, 1, 2, 3 and 5). Each grid record is 8 cells in 1 row by 8 columns, and the groups use vertex program 2 and UV mode 3. No top-level group selects variant 2.

**Disc users of variant 1 (High).** 2 groups, both pass 0 with UV mode 3:
- Resource 9 (item data) group 102: count 16, 4 × 4 grid, divisor 2, texture 128 × 128, so 32 × 32 cells; 30 cells per second, a 16-cell loop of 32 passes.
- Resource 1248 (a world pack) group 1: count 32, 4 rows × 8 columns, divisor 1, texture 256 × 128, so 32 × 32 cells; one cell per pass, a 32-pass loop.
The wrap of the 16-bit counter is seamless for both (65536 is a multiple of 32).

Test vectors (synthetic):
- Variant 1: count 6, rows 2, columns 4, divisor 1, texture 128 × 64 (TW 7, TH 6), N = 5. Limit = min(6, 8) = 6; index = 5; column 1, row 1; cell 32 × 32; cell x 32, cell y 32. Output word: 0x0000_0080_1F08_01FF (fields: width − 1 = 31 at bit 4, x = 32 at bit 14, height − 1 = 31 at bit 24, y = 32 at bit 34, low nibble F).
- Variant 1, divisor: same record, divisor 2, N = 7: index = floor(7 / 2) = 3; column 3, row 0; cell x 96, cell y 0.
- Variant 1, clamp: count 20, 4 × 2 grid, divisor 1, N = 9: limit = 8; index = 9 mod 8 = 1; column 1, row 0; cell x 32, cell y 0.
- Variant 2: key 2, count 6, 2 × 4 grid: the limit table gets 5 under key 2; frame table key 2 = 0, so cell 0. Key 7, with limit 3 stored earlier under key 1: the frame read is 3, so column 3, row 0.
- Region repeat: cell width 32, cell x 64. Texel u_t = 70: (70 AND 31) OR 64 = 6 OR 64 = 70. u_t = 100: (100 AND 31) OR 64 = 4 OR 64 = 68.

Test vectors (disc records, High):
- Resource 9 group 102. N = 0 and 1: cell 0, output 0x0000_0000_1F00_01FF. N = 2: cell 1 (x 32, y 0), 0x0000_0000_1F08_01FF. N = 8: cell 4 (x 0, y 32), 0x0000_0080_1F00_01FF. N = 31: cell 15 (x 96, y 96), 0x0000_0180_1F18_01FF. N = 32: cell 0. Register word: 9.
- Resource 1248 group 1. N = 7: cell 7 (x 224, y 0), 0x0000_0000_1F38_01FF. N = 31: cell 31 (x 224, y 96), 0x0000_0180_1F38_01FF. N = 33: cell 1.

### Runtime upload packet builder (per upload segment)

Confidence: Medium for the order of the parts and the chunking rule; Low-Medium for the per-format sizes (not checked against the disc); Medium for the link to the prebuilt packets above.

The prebuilt packets in each texture chunk are one fixed transfer. A separate builder creates the same kinds of transfer packets from upload-segment records. Its only identified caller runs during graphics startup, before the main frame loop: it expands two images embedded in the boot executable and submits each transfer. For that caller the destination base is the segment record DBP field, used directly without adding either ordinary texture upload-area base. The two embedded records each describe one 512 x 448 PSMCT32 image; their DBPs are 0xE00 and 0, respectively. This startup path is not an in-game texture residency path (High for the observed inputs, destinations and call order).

For each upload segment, the builder emits transfer packets whose parts appear in this order:

1. Destination buffer: base block address, buffer width (the segment width divided by 64) and the segment's pixel format.
2. Transfer position: destination x and y, and the base block.
3. Transfer size: width and height in the segment's units.
4. Transfer direction: from the host into local memory.
5. Image data tag, followed by the pixel data for the chunk.
6. Texture-cache flush.

The prebuilt packets use the same parts in the same order, wrapped in the DMA transfer records (Upload, above).

- **Chunking (High).** The unit is the 16-byte quadword of the image data. Let Q be the segment's size in quadwords and q its row size in quadwords (W / 16 for 8-bit indexed, W / 32 for 4-bit indexed, W / 4 otherwise; a row size of 0 counts as 1). The builder emits floor(Q / 0x7FFF) + 1 chunks. Each chunk is whole rows: floor(0x7FFF / q) rows, or the rows still left when fewer remain, so its transfer height is that row count, its image data tag counts q × rows quadwords, and the next chunk starts that many rows lower. The load-time handling of the prebuilt packets counts chunks with the same Q (one pair of area 0 and area 1 records per chunk). Test vectors: 256 × 256 32-bit: q = 64, Q = 16384, 1 chunk of 256 rows; 512 × 512 32-bit: q = 128, Q = 65536, 3 chunks of 255, 255 and 2 rows; 128 × 128 8-bit: q = 8, Q = 1024, 1 chunk; 64 × 64 4-bit: q = 2, Q = 128, 1 chunk.
- **Per-format size factor (High).** The data size is the segment's width times its height, times a factor from the pixel format byte of the segment: 1 byte per texel for the 8-bit indexed formats (0x13 and 0x1B); half a byte per texel for the three 4-bit indexed formats (0x14, and the high and low nibble formats 0x24 and 0x2C, which share the 4-bit size); 4 bytes per texel for every other format, including the 16-bit and 24-bit formats (quirk: they would be over-counted; the disc has only 32-bit segments, so this never matters). The source pointer advances by the same byte count per chunk and per segment. The graphics library's own transfer set-up, used elsewhere, sizes 16-bit formats at 2 bytes and 24-bit at 3 bytes per texel and refuses a transfer of 0x8000 quadwords or more; that path is not used for disc textures.
- **Sending.** When the builder has a target, it flushes the cache, then sends the packet by DMA. When there is no target, it only builds the packet.
- **Callers.** One caller was found. Further callers are not established.

### Transfer-word helper (Low; conflicts with another reading)

A small helper edits one 64-bit word in place, with one argument that selects between two forms. Reading its bit operations gives the fields of a GIF tag word, not the fields of an image-transfer register:

- It sets bit 15 (EOP) and bit 46 (PRE), and clears bits 0–14 (NLOOP) and bits 58–59 (FLG).
- It sets bits 60–63 (NREG) to 9.
- It writes one of two fixed patterns into the PRIM bits (47–57), chosen by whether the argument equals 1.
- It clears bits 16–31 and writes a second word holding nine 4-bit register codes, in order 1, 2, 5, 2, 5, 2, 5, 2, 5. In the public register codes these are RGBAQ, ST, XYZ2 repeated, which is a draw-vertex list.

The image-transfer packets in Upload use the A+D register list (BITBLTBUF, TRXPOS, TRXREG, TRXDIR) and contain no such list. So this helper looks like a builder for a draw tag and is not part of the texture upload. Another reading called it a transfer-register setter; the bit operations above do not support that. Its callers are not established.

### Material slot check (camera-block resend test)

Confidence: High.

The check answers "does this draw need a new camera block for the vector unit" (the eye point and matrix that UV modes 1 to 3 use). Input: a group index and the frame parity *b*.
1. If no shader list is bound, the result is false.
2. Read the 16-bit UV-mode words of the group's material record for parity *b*: record offsets 0x1C and 0x2C (pass 0 and pass 1; group offsets 0x4C and 0x5C for parity 0, 0x9C and 0xAC for parity 1). If both are zero, the result is false.
3. Otherwise compare with the last camera block sent this frame for parity *b*: the result is true if there is none, or if the eye point differs (the sum of the absolute component differences is not zero).
4. With an equal eye point, the original returns true only when **all four** rows of the 4 × 4 matrix differ from the stored ones; if any one row is equal, it returns false (original quirk: a rotation that keeps one row, such as a turn about the up axis, keeps the old block).

When the result is true, the caller takes a new camera block (at most 32 per parity per frame; a 33rd request is reported as an error and gets no block) holding the eye point and the matrix, and sends it before the batch. Clean side: recompute the environment UVs from the current eye point and matrix on every draw; reproduce the quirk only for strict parity.

### GS local-memory addressing (PSMCT32, PSMT8, PSMT4)

Confidence: High. The rules below are the standard PS2 Graphics Synthesizer layout. Every format the disc uses is covered: PSMCT32 (upload images and palettes), PSMT8 and PSMT4 (textures). No other pixel format appears in a texture chunk. A renderer that keeps its own copy of the decoded pixels does not need this section at run time, but it needs it once to decode the stored block.

**Units.** Local memory is 4 MiB. A block address (TBP, CBP, DBP) counts 256-byte blocks, so the range is 0 to 16,383. A page is 32 consecutive blocks (8,192 bytes). A buffer width (TBW, DBW) counts 64-pixel units, for every format.

| Format | Block size (pixels) | Page size (pixels) | Pages per buffer row | Blocks per page across × down |
| --- | --- | --- | --- | --- |
| PSMCT32 | 8 × 8 | 64 × 32 | width value (at least 1) | 8 × 4 |
| PSMT8 | 16 × 16 | 128 × 64 | width value / 2, rounded down (at least 1) | 8 × 4 |
| PSMT4 | 32 × 16 | 128 × 128 | width value / 2, rounded down (at least 1) | 4 × 8 |

**Address of texel (x, y)** for a buffer with base block B and width value W:
1. Page number = (y div page height) × pages per row + (x div page width).
2. Block-in-page number from the block column bx = (x div block width) mod blocks-across and the block row by = (y div block height) mod blocks-down:
   - PSMCT32 and PSMT8: (bx AND 1) + 4 × ((bx div 2) AND 1) + 16 × (bx div 4) + 2 × (by AND 1) + 8 × (by div 2).
   - PSMT4: 2 × (bx AND 1) + 8 × (bx div 2) + (by AND 1) + 4 × ((by div 2) AND 1) + 16 × (by div 4).
3. Block address = B + 32 × page number + block-in-page number. The addition is plain; B need not be a multiple of 32 (disc values such as 592 are not).
4. Offset inside the block, from the in-block coordinates u = x mod block width and v = y mod block height:
   - PSMCT32 (a word is 4 bytes): word = (u AND 1) + 4 × (u div 2) + 2 × (v AND 1) + 16 × (v div 2). Byte offset = 4 × word.
   - PSMT8: let r = v AND 3 and c = v div 4 (0 to 3). Start from t = 4 × (u AND 1) + 16 × ((u div 2) AND 3) + 2 × (u div 8) + 8 × (r AND 1). If (c AND 1) differs from (r div 2), flip bit 5 of t (add or remove 32). Byte offset = 64 × c + t + (r div 2).
   - PSMT4: let r = v AND 3 and c = v div 4 (0 to 3). Start from t = 8 × (u AND 1) + 32 × ((u div 2) AND 3) + 2 × (u div 8) + 16 × (r AND 1). If (c AND 1) differs from (r div 2), flip bit 6 of t (add or remove 64). Nibble offset = 128 × c + t + (r div 2); a block holds 512 nibbles.
5. Byte address = 256 × block address + byte offset. For PSMT4, nibble address = 512 × block address + nibble offset; the byte is the nibble address div 2, and the index is the low four bits of that byte when the nibble address is even and the high four bits when it is odd.

**Segment seen as indexed texture.** A segment is uploaded as PSMCT32 with buffer width equal to its pixel width / 64 (4 for a 256-pixel segment). The sub-textures read the same memory as PSMT8 or PSMT4, and their TBW is twice that value (8 for a 256-pixel segment), because a PSMT8 or PSMT4 page is twice as wide in pixels as a PSMCT32 page. The pixels of the stored block are therefore not the texture image in raster order; only the memory-address mapping above recovers it.

**Palettes (CSM1).** A palette is a PSMCT32 image read with buffer width 1 from its CBP:
- 256 entries (for PSMT8) form a 16 × 16 image. Linear entry j is the pixel (j mod 16, j div 16). The colour for index i is linear entry e, where e is i with bits 3 and 4 exchanged (the swap in "Decode one indexed texture").
- 16 entries (for PSMT4) form an 8 × 2 image. Index i is the pixel (i mod 8, i div 8). There is no swap. CSA is 0, so the palette starts at its first entry.
- Each palette entry is the 4 bytes R, G, B, A (alpha 0 to 128).
- A PSMT8 texture uses a 16 × 16 palette and a PSMT4 texture an 8 × 2 one. The palette is the one the material's TEX0 names by CBP, not the first palette in the chunk.

**Register placement.** Each material group's state block carries its TEX0 as a register pair (see "Load-time handling of shader lists and texture chunks"). In a mode-1 group the pair numbered 7 is TEX0 (of the second GS drawing context). In a mode-2 group, pass 0 uses the pair numbered 7 and pass 1 the pair numbered 6 (TEX0 of the first drawing context). Find the pairs by register number. TBP, TBW, PSM, TW, TH and CBP come from that value and nowhere else.

**Warning: all-white or wrong-colour pictures.** Two shortcuts produce all-white or badly coloured textures. (1) Ignoring the segment base: in the 102 front-end chunks the segment TBP is 0x2A00 and every sub-entry and palette address is absolute, so the texture must be read relative to the segment, not at address 0. (2) Using a fixed or "first" palette instead of the group's own: the palette is the one whose address equals the CBP in the group's TEX0, and different groups of one chunk use different palettes (some share a texture with a different palette). Skipping the CSM1 swap gives correct shapes with scrambled colours.

### Decode one indexed texture to RGBA

1. For each texel (x, y), 0 ≤ x < real width, 0 ≤ y < real height: read the index at the PSMT8 (or PSMT4) address for (x, y) with base TBP and buffer width TBW, using the rules of "GS local-memory addressing".
2. Look up the palette (see below for which palette). Palettes are stored with the GS "CSM1" layout:
   - A 16 × 16 PSMCT32 palette: read its 256 colors in row-major order from the PSMCT32 image at (TBP, buffer width 1). Then swap the CSM1 order: index *i* uses linear entry *i* with bits 3 and 4 exchanged. In other words, entries 8–15 and 16–23 of every group of 32 swap places.
   - An 8 × 2 palette for PSMT4: read its 16 colors in row-major order from (TBP, buffer width 1). No swap.
3. Each palette color is 4 bytes: R, G, B, A. **Alpha uses the PS2 range 0–128, where 128 means fully opaque.** On the disc no palette alpha exceeds 128 (807,376 palette entries checked); 128 and 0 are by far the most common. Convert as A8 = min(255, A × 2), or keep A / 128 as a float.
4. Texels beyond the real size up to the power-of-two TW/TH sampling size exist in GS memory but carry no meaningful image. Repeat-wrapping samples the power-of-two size (see CLAMP).

### Materials: the shader list (chunk type 2)

A shader list holds material "groups". Each group carries complete GS texture register values that name its texture, palette and mip levels by GS memory address. Offsets are from the shader-list payload (E = 0 for all 215 on disc).

Shader list header:

| Offset | Size | Type | Meaning | Seen on disc |
| --- | --- | --- | --- | --- |
| 0x00 | 16 | — | Unused (zero on disc) | |
| 0x10 | 4 | u32 | Offset of the group table | 0x20 in all 215 |
| 0x14 | 4 | u32 | Group count *N* | |
| 0x18 | 4 | u32 | Offset of the texture-slot array = 0x20 + N × 0x270 | all 215 |
| 0x1C | 4 | u32 | Count *B* of texture chunks this list uses | 1 … 6 |

The texture-slot array is *B* u32 values. The engine fills it at load time; on disc it is just space. The payload ends right after it.

The *B* texture chunks are the next *B* type-4 chunks in the same chain, in order (all 215 lists match). The original reports an error if a texture chunk appears before any shader list, if more than *B* appear, or if fewer than *B* appear by the end of the chain.

### Selecting a material group for a draw

Confidence: High.

Inputs: a group index and the bound shader list.
1. If no shader list is bound, nothing happens.
2. The index is taken as an unsigned 16-bit value. An index of *N* or more is reported as an out-of-range error and group 0 is used instead.
3. The group address = group table + index × 0x270. If that group is already the selected one, nothing more happens.
4. If the group has texture state (mode byte 0x26 nonzero), the residency rule below runs for its texture chunk *k*.
5. The group is sent: first its material record for the current frame parity to the vector unit, then its GS state block for the current upload area (block 0 when the group has no texture state). A group that is the last one sent is not sent again. The selection writes nothing into the group.

### Binding a billboard node to its material group (load time)

Confidence: High for the binding and the writes; the users of these nodes are open.

Three loaders of effect-node and world-node groups force program 3 on the groups they bind. One of them works as below; the other two make the same two writes.

Inputs: a draw context (which gives the index) and the shader list (whose group table pointer is the list's header word at offset 0x10, after load fix-up).
1. If the context or the list is absent, the binding fails (result 0) and nothing is written.
2. The group index is a signed 16-bit value read from the context at offset 0x5C.
3. The bound group address = table pointer + index × 0x270.
4. The 16-bit field at offset 0x44 and the 16-bit field at offset 0x94 of the bound group are set to 3: the pass-0 vertex program of both parity copies of the material record becomes program 3, the camera-facing billboard program (see "Material record"). The write goes to the shared group entry itself, not to a copy.
5. The binding returns success (1) and the bound address is passed back to the caller.

### Billboard program 3

Confidence: High for the arithmetic; Medium for the corner UV orientation.

The batch tag is followed directly by n elements, where n is the tag's NLOOP. Each element is three 16-byte vectors:
- q0: four RGBAQ integers, copied unchanged to every corner;
- q1: the centre (x, y, z) in the draw's model frame, and a in lane 4;
- q2: the extents (sx, sy, sz), and b in lane 4.

Let C = (VP · M)(x, y, z, 1) be the centre in clip space, with clip w = C.w, and let F = 512 (the camera focal length, SPEC-0046). With u = (a, b) and v = (−b, a), a = cos θ and b = sin θ rotate the sprite in screen space. Corner offsets, in clip units:
- c1 = sz F v − sx F u
- c2 = sz F v + sy F u
- c3 = −sz F v − sx F u
- c4 = −sz F v + sy F u

Each corner's screen position is (C.xy + offset) / C.w, and its depth is C.z / C.w. Texture coordinates per corner, with Q = 1: c1 (0, 0), c2 (0, 1), c3 (1, 0), c4 (1, 1).

Each element is output as a 4-vertex strip; c1 and c2 always carry the no-triangle bit. The whole sprite is dropped (the no-triangle bit is also set on c3 and c4) when any of these holds:
- C.w ≤ 1;
- any corner has a GS x or y of 4094 or more;
- any corner has a GS x or y of 0 or less.

Size quirk: the sprite size uses F = 512 on both axes, while the 3D projection uses 409.6 horizontally and 477.87 vertically (SPEC-0046, Camera projection). So a billboard is 1.25 times as wide and 1.071 times as tall on screen as a 3D quad of the same world size.

### Load-time handling of shader lists and texture chunks

The loader's chunk dispatcher handles these two chunk types as it walks the chain (SPEC-0006, "How the loader handles each chunk"). The same dispatcher handles type 8 and types 2000 and 2001; those are in SPEC-0004, "Load-time handling". A chunk type that the dispatcher does not list is reported as an error; whether the walk continues after that report is not stated. High confidence for the handling; Medium for the registration detail.

- **Shader list (type 2)**: it becomes the current list for the chunks that follow. Its handler stores the list pointer and runs the offset fix-up when the list is loaded. The fix-up adds upload area 0's base to every address in state block 0 and upload area 1's base to state block 1, and stores the two bases at group offsets 0x22 and 0x24. The handling is one test with two actions (High):
  1. The list header's word at +0x0C is set to −1.
  2. When the header's group-array offset (+0x10) is nonzero, it is turned into an address (offset plus the list's own address) and the area-base fix-up above runs over every group (group mode byte +0x26 of 1 or 2). That fix-up skips every texture word whose pixel format is 0x1B.
  3. When the header's texture-slot-array offset (+0x18) is nonzero, it is turned into an address the same way. The character and world-pack walkers write the fixed-up type-4 payload value into the next slot while the observed texture count is below the capacity at +0x1C.
  4. Back in the loader, when the group array is present, a second fix-up runs over the same groups and touches only the texture words of format 0x1B: it adds the texture manager's current allocation position (its running block counter, in 256-byte blocks, read when the chunk chain starts) to the texture base and 0 to the palette base.
  Test vector: a format-0x1B texture word with base 0x40 in a chain that starts with the allocation position at 0x1200 ends with base 0x1240 and an unchanged palette base; a format-0x00 word in parity record 1 gets area 1's base instead.
- **Texture chunk (type 4)**: its payload offsets are fixed up. For an ordinary chunk the destination of packet A is set to upload area 0 and that of packet B to upload area 1. The front-end chunks (segment TBP 0x2A00) are fixed up without changing their packets, so they upload at 0x2A00 as stored. The chunk is stored in the next free texture slot of the current list, in walk order. A texture chunk beyond the list's count *B* is dropped and logged, and loading goes on.
- A shader list must come before the texture chunks it uses. This holds in every pack that has both (SPEC-0006 census).

Group (0x270 bytes; group *g* at group table + g × 0x270):

| Offset | Size | Type | Meaning |
| --- | --- | --- | --- |
| 0x00 | 16 | 4 × f32 | Sub-texture grid record for pass 0 (see "Sub-texture frame addressing"); nonzero only where byte 0x2B is nonzero |
| 0x10 | 16 | 4 × f32 | Sub-texture grid record for pass 1 |
| 0x20 | 1 | u8 | Group index, equal to *g* |
| 0x22 | 2 | u16 | Written by the engine at load (upload area 0 base); ignore on disc |
| 0x24 | 2 | u16 | Written by the engine at load (upload area 1 base); ignore on disc |
| 0x26 | 1 | u8 | Mode: 0 = no texture state (670 top-level groups), 1 = one texture context (14,422), 2 = two texture contexts (500) |
| 0x27 | 1 | u8 | Character colour slot, 0 to 14. Used only when a character is drawn: the part that uses this group is drawn with the colour of that slot (colour word *j* of the character's record; SPEC-0040, Behavior 26, item 9). It applies to textured and untextured groups alike. Over the 1,157 character model sets (17,217 groups) 4,733 groups have a slot above 0; over the 16 hero outfits only 9 of 331 do, so a hero looks right even if the slot colour is ignored and an NPC does not. Level and prop groups leave it 0 |
| 0x28 | 1 | u8 | Texture chunk index *k*, 0 … B − 1: the group's addresses refer to the *k*-th texture chunk after this shader list. Always 0 when B = 1. |
| 0x29 | 1 | u8 | CPU vertex animator id: 0 none, 1 environment map, 2 UV scroll, 3 water (see "UV modes"). 3 in 2 groups on disc (resource 9 groups 20 and 21), otherwise 0. |
| 0x2B | 1 | u8 | Texture animator kind, pass 0: 0 none, 1 variant 1, 2 variant 2. 1 in 2 groups; 2 in 5 nested groups of resource 1260. |
| 0x2C | 1 | u8 | Texture animator kind, pass 1. 0 in all groups. |
| 0x30 | 0x50 | | Material record for frame parity 0 (see "Material record") |
| 0x80 | 0x50 | | Material record for frame parity 1 (identical to parity 0 in 15,587 of the 15,592 top-level groups) |
| 0xD0 | 0xD0 | | State block 0 |
| 0x1A0 | 0xD0 | | State block 1, a copy of block 0 (identical in 15,589 of the 15,592 top-level groups) |

Other group bytes are zero or not used. A group with mode 0 (no texture state) is not a missing material: it is a flat-colour part whose colour comes from the vertex colours and the character colour slot (see 0x27 above). The 17,217 groups of the character model sets split into 9,415 textured and 7,802 untextured (2,614 of them with a slot above 0); an NPC's skin, hair and clothes are mostly such untextured groups.

A state block is a GS register list. It starts with a 16-byte GIF tag. NLOOP is bits 0–14 of its first u16, and the tag uses packed A+D mode. After the tag come NLOOP pairs of 16 bytes: an 8-byte register value followed by an 8-byte register number (only the low byte matters). The clean side should read the pairs by register number, not by position.

- Mode 1 (NLOOP 6): TEX0 (register 0x07), TEX1 (0x15), MIPTBP1 (0x35), CLAMP (0x09), ALPHA (0x43), TEST (0x48).
- Mode 2 (NLOOP 12): those six, then a second set for the other GS drawing context: TEX0 (0x06), TEX1 (0x14), MIPTBP1 (0x34), CLAMP (0x08), ALPHA (0x42), TEST (0x47). The second set is the GS state of render pass 1 (see "Material record").
- One resource (528) has 5 groups whose block lacks the GIF tag and starts directly with register pairs. Treat a block as malformed if its first u64 does not look like a GIF tag with NLOOP 6 or 12, and skip those groups' texture state.

Blend, alpha test, wrap and filter come only from the state blocks. No field of the material record or of the group controls back-face culling or two-sided drawing, and no vertex program culls back faces (High): the only per-triangle rejection is the clip test of SPEC-0046 ("Per-triangle clipping in the vertex program"), which runs only while the model part's clip bit is set. Draw every material double-sided.

Register fields follow the public GS documentation. What they mean for this game's assets:

- **TEX0**: TBP (bits 0–13), TBW (14–19), PSM (20–25: 0x13 or 0x14 on disc), TW (26–29), TH (30–33), TCC (34, always 1: use texture alpha), TFX (35–36: 0 = modulate in 15,406 records, 3 = "highlight 2" in 18), CBP (37–50: the palette's base block), CPSM (51–54, always 0: 32-bit palette), CSM (55, always 0: CSM1 layout), CSA (56–60, always 0), CLD (61–63: palette-load control for the hardware, irrelevant to decoding).
  - Take texture chunk *k* (group offset 0x28). In its table, find the texture as the sub-entry whose TBP and PSM equal the record's TBP and PSM, and the palette as the PSMCT32 sub-entry whose TBP equals CBP. Use the record's TW/TH as the sampling size. Verified: in all 14,419 well-formed mode-1 groups (13,300 of them in lists with B > 1), both the texture and the palette are found in chunk *k*. Searching by address alone would be ambiguous: in multi-chunk lists the same TBP/PSM appears in several chunks in about 6,000 groups.
- **MIPTBP1**: TBP1 (bits 0–13), TBW1 (14–19), TBP2 (20–33), TBW2 (34–39), TBP3 (40–53), TBW3 (54–59). These are the base addresses of mip levels 1–3. Each matches a sub-entry of the same PSM with half the previous level's size (5,767 of 5,768 levels on disc). All levels share the material's palette.
- **TEX1**: MXL (bits 2–4) is the number of mip levels beyond the base. It is 0 in about 12,400 materials and 1–3 in about 3,000. MMAG (bit 5) is 1 (bilinear magnification) in all records. MMIN (bits 6–8) is 4 (bilinear within the nearest mip level) in almost all; 5 (bilinear within a level, linear between levels) in a few. LCM (bit 0) and K (bits 32–43, signed fixed point with 4 fractional bits) set the level-of-detail rule:
  - LCM = 1 with K = −1/16: a fixed level, effectively always the base level. This is the most common (about 12,400).
  - LCM = 0 with K between −6 and −2: the GS picks the level as LOD = log2(1/Q) + K, where Q is the per-vertex perspective term. A modern renderer can approximate this with its normal mip selection plus a bias. Exact parity is for the model/render specs.
- **CLAMP**: WMS (bits 0–1, U axis) and WMT (bits 2–3, V axis): 0 = repeat, 1 = clamp. On disc: repeat/repeat 28,835 records, repeat/clamp 450, clamp/clamp 422, clamp/repeat 134. A flipbook pass replaces this value with a region-repeat value for each batch (UV mode 3).
- **ALPHA** (blend Cv = ((A − B) × C >> 7) + D, using the public field meanings): 0x44 = (Cs − Cd) × As + Cd, ordinary alpha blending (29,067 records). 0x48 = Cs × As + Cd, additive (632). 0x58 = Cs × Ad + Cd (138). 0x64 with FIX 64 = (Cs − Cd) × 0.5 + Cd (4). 0x68 = Cs × FIX / 128 + Cd, additive with a fixed factor (54 second-set records in state block 0, FIX 86, 38 or 80; all on environment-map passes). In every case alpha 128 means 1.0.
- **TEST**: alpha test enabled (ATE = 1) in 15,341 of 15,429 records, method GEQUAL with reference 120 (of 128). AFAIL = 1 ("framebuffer only"): a pixel that fails the alpha test is still drawn and blended but does not write depth. Depth test is always on with method GEQUAL (the PS2 convention that a larger Z is nearer). In effect, texels with alpha ≥ 120/128 write depth, and softer texels blend without writing depth.

**Translucency in the draw order is not material alpha (High).** The render pass that orders object draws (SPEC-0046, Sort order) tests a flag and a value below 128. Both belong to the character's off-screen composite record, not to any material: the flag means "fade active" and the value is the character's fade level, 0 to 128 (128 = fully opaque). A faded character is drawn into an off-screen buffer and then put on screen as one textured, screen-aligned rectangle with the ordinary alpha blend and source alpha = level / 128, so it fades as a whole silhouette. Materials are never sorted by alpha; a material's palette alpha and its ALPHA and TEST registers act only inside its own draw. The objects that fade this way are: the current hero's camera-near fade (SPEC-0027, zoom ramp), ghost monsters (their Ghost Opacity Min/Max monster data, SPEC-0029), the Gelert Assassin's fade-in (level rises by 1 per tick up to 128), and the Cloud Beast (level 129: drawn through the composite path but sorted opaque). The full rule is in SPEC-0046.

### Material record (group bytes 0x30 to 0xCF)

Confidence: High for the layout, the counts and the vertex programs.

Group bytes 0x30 to 0x7F are a 5-qword record for the vector unit; bytes 0x80 to 0xCF are a second copy. The copy for the current frame parity (0 or 1) is the one sent on every selection of the group, and the one the texture animator writes. Offsets below are record offsets; group offset = 0x30 + record offset for parity 0 and 0x80 + record offset for parity 1.

| Rec. off. | Group off. (parity 0) | Size | Meaning |
| --- | --- | --- | --- |
| 0x00 | 0x30 | 16 | Record header: u16 at +2 = 5 (qword count), byte +4 = 5, byte +5 = 3 (record key 0x300) |
| 0x10 | 0x40 | u32 | Number of passes, 1 or 2 (equals the context count: 2 exactly for mode-2 groups) |
| 0x14 | 0x44 | u32 | Pass 0 vertex program (below) |
| 0x18 | 0x48 | u32 | Pass 0 vertex attribute mask: bit 0 position, bit 1 colour, bit 2 normal, bit 3 skin weights, bit 4 bone palette, bit 5 UV |
| 0x1C | 0x4C | u32 | Pass 0 UV mode: 0 none, 1 environment map, 2 scroll, 3 flipbook region |
| 0x20 | 0x50 | u32 | Never read (Medium-High): the vector unit stores it but never reads that lane, and no CPU accessor of a group reads it. 7 in the 279 two-pass groups with programs (1, 1) and a pass-1 environment map; 3 in the other 307 two-pass groups; 0 in one-pass groups. Ignore it. |
| 0x24 | 0x54 | u32 | Pass 1 vertex program; 0xFFFF when there is no second pass |
| 0x28 | 0x58 | u32 | Pass 1 attribute mask |
| 0x2C | 0x5C | u32 | Pass 1 UV mode |
| 0x30 | 0x60 | 16 | Pass 0 UV-mode data: scroll speeds (su, sv) as two floats for mode 2, or the flipbook CLAMP value and its register number for mode 3. In program-6 groups (UV mode 0) lanes 3 and 4 (group offsets 0x68 and 0x6C) hold the specular offset z and power p of SPEC-0046 ("Specular alpha term of program 6"). |
| 0x40 | 0x70 | 16 | Pass 1 UV-mode data |

Vertex programs (value of the program word; High). The program word of each pass chooses the vertex program; the mesh batch carries no program choice (SPEC-0004, "Vertex program").

| Program | What it does |
| --- | --- |
| 0 | Unlit. Fixed slots: position, colour, UV (mask 0x23). Output = trunc(byte × s), no limit. |
| 1 | Lit. Fixed slots: position, colour, normal, UV (mask 0x27). |
| 2 | Skinned and lit. Fixed six slots: position, colour, normal, weights, bone slots, UV (layout A; mask 0x3B or 0x3F). Three-bone blend in the near state, first bone only in the far state; the same lighting as program 1. |
| 3 | Camera-facing billboard (see "Billboard program 3"). Never stored on disc; written at load by the billboard binding. |
| 4 | Unlit. Slot positions follow the pass's attribute mask, so a normal slot can be skipped (mask 0x27). |
| 5 | Lit. Slot positions follow the attribute mask. Unused on disc. |
| 6 | Program 2 plus the specular alpha term of SPEC-0046, near state only (mask 0x3F). |

- Any other value: the batch is not drawn.
- The attribute mask is read only by programs 4 and 5 and by the UV-mode handlers. Programs 0, 1, 2 and 6 ignore it.
- Every program-2 or program-6 group with a UV mode has mask 0x3F, so the UV-mode handlers find the UV slot where program 2 reads it.

Counts (High): 1,372 shader lists (215 top-level and 1,157 inside model-set wrappers) hold 32,809 group records counted per list occurrence (15,592 top-level and 17,217 nested). Pass-0 program counts:
- Groups with texture state: program 0: 13,696; 1: 957; 2: 9,024; 4: 515; 6: 145.
- Groups without texture state: 0: 491; 1: 29; 2: 7,951; 6: 1.

How a pass is drawn: for each mesh batch the pass count is read; pass 0 uses the pass-0 words and the first GS register set; then, if the count is 2, pass 1 draws the same positions, colours and normals again with the pass-1 words and the second GS register set (mode 2). Each pass first applies its UV mode to the batch's UVs, then runs its vertex program.

Pass-1 UVs (High). Pass 1 reads its UVs from the second half of the batch's UV array (SPEC-0004, "Two-pass batches"). The UV mode of each pass works on that pass's own half: an environment map writes its generated UVs into it, a scroll adds its offset to it in place, and a flipbook only sends the CLAMP value. So the two passes never share UVs.

Counts by UV mode, over all 32,809 group records:
- Pass 0: environment map 202; scroll 917 (482 top-level, and 435 nested program-2 groups with mask 0x3F); flipbook region 7 (the 2 variant-1 and 5 variant-2 users); none in every other group.
- Pass 1 (the 586 two-pass groups): environment map 438; scroll 139; none 9.

### UV modes

Confidence: High for the arithmetic; environment-map inputs High for level geometry and for placed objects; Medium for the visible effect on placed objects.

Let N be the render frame counter (see "Sub-texture frame addressing", variant 1) as a float, 0 to 65535.

- **Mode 2, scroll (in the vector unit).** For every vertex of the batch: u' = u + (su × N − trunc(su × N)) and v' = v + (sv × N − trunc(sv × N)), with su and sv the pass's speeds. trunc rounds toward zero, so a negative speed gives an offset in (−1, 0]. UVs are in units of the texture's sampling size (1.0 = one texture width), and repeat wrapping does the rest. Speeds are per main-loop pass. Original quirk: the counter wraps at 65536, so the offset jumps once every 65536 passes (18.2 minutes at 60 passes per second) unless s × 65536 is a whole number. Users: 917 pass-0 passes (482 in top-level lists in 118 resources, and 435 program-2 groups with mask 0x3F in nested lists) and 139 pass-1 passes (70 resources). They include every sky dome (resources 0 to 3): group 1 scrolls one layer at (0.0000625, 0); group 2 is two-pass with two counter-scrolling cloud layers, pass 0 at (0.000125, 0.0000625) and pass 1 at (−0.0000625, −0.000125).
- **Mode 1, environment map.** The camera block holds the eye point E (the view point given to the tree draw) and a 4 × 4 matrix M (the matrix given to the tree draw; only its first three vectors are used, as columns, with no translation). For each vertex with position P and normal Nv: D = P − E; R = D − 2 (D · Nv) Nv; R' = M3 R; R^ = R' / |R'|; u = 0.5 (1 − R^z); v = 0.5 (1 + R^y). The UVs stored in the mesh are replaced.
  - Level geometry (High): M is the identity and E is the camera position in world space (the translation of the camera matrix). So the reflection is taken in **world space**, and the map does not turn with the camera.
  - Placed objects (High for the inputs; Medium for the visible effect): M is the object's current block, its model-to-world matrix (three scaled rotation rows). E is the object's own view-point vector, recomputed by each animation update as the object's position plus its mesh corner minus the camera's world position, so it has the opposite sign to the true offset from the object to the camera; it is the origin for an object that was never updated. The full rule, the quirk and a test vector are in SPEC-0046 (Placed-object draw frames).
  - Users: 202 pass-0 and 438 pass-1 passes; the top-level ones all have attribute mask 0x27 and program 1 or 4. Most are the second pass of a two-pass world material; the top-level pass-1 environment passes blend additively or by destination alpha (ALPHA 0x48: 292, 0x68 with FIX: 54, 0x58: 6).
- **Mode 3, flipbook region.** The pass's UV-mode data (the CLAMP value and register number written by the texture animator) is sent to the GS before the batch, overriding the state block's CLAMP for that batch. The mesh UVs are not changed. Users: the 7 groups of "Sub-texture frame addressing" (2 variant 1, 5 variant 2).

**CPU vertex animators (group byte 0x29).** A separate path rewrites the vertex data on the main processor, only for mesh leaves whose node byte +0x5E is 1. That byte is the packet kind of SPEC-0004: kind 1 is a CPU-animated, double-buffered leaf, and the animator rewrites the draw packet of the current frame parity (SPEC-0004, "Multi-packet leaves"). On disc exactly 13 leaves have kind 1: resource 9, type-12 children 16 to 28, which use groups 20 and 21 (animator 3).
- Id 1: the same environment-map formula as mode 1. No group selects it.
- Id 2, scroll: offset = fmod(N2 × s × K / 100, 1) per axis, with N2 the main-loop pass counter (run-time word 0x368F98), K the boot data word at 0x2DCB24 (0.025, so the offset is N2 × s / 4000), and the speeds s from group offsets 0x60 and 0x64. The offset is added to the base UVs. No group selects it.
- Id 3, water (resource 9 groups 20 and 21): a global phase u0 = fmod(u0 + 0.06, 1) is added to every vertex U; a second global phase p += 0.2 (never wrapped) gives an alpha a = 64 + (250 − 64) × (1 + sin p) / 2, rounded half up, written into the alpha byte of a 16-byte colour that is copied to every vertex colour. The colour comes from one of two run-time colour slots, chosen by whether the float at group offset 0x60 is nonzero. Data words of the boot program: 0x2DCA98 (U phase, 0.0 at boot), 0x2DCA9C (U step, 0.06), 0x2DCAA0 (alpha phase step, 0.2), 0x2DCAA4 (64.0), 0x2DCAA8 (250.0), 0x2DCAAC (alpha phase, 0.0 at boot). Both phases advance once per call, that is per mesh draw, not per pass (quirk: two meshes with this animator in one frame advance them twice).

### When textures are resident (for parity, not needed for decoding)

Confidence: High for the rules; Medium for the queueing detail.

- GS memory has two texture upload areas: area 0 starts at the run-time VRAM texture base, and area 1 starts 0x400 blocks later (256 KiB, one 256 × 256 PSMCT32 segment).
- At load, packet A of each ordinary texture chunk is set to upload to area 0 and packet B to area 1, and the group fix-up rebases state block 0 to area 0 and state block 1 to area 1. So block *b* is the state to use while the chunk sits in area *b*. This is why block 1 is a copy of block 0 on disc.
- The engine keeps a "free area" index *f* and the most recently uploaded chunk. When a selected group with texture state names a texture chunk (slot *k*, group offset 0x28) other than the most recently uploaded one, that chunk's upload into area *f* is scheduled and *f* flips. The state block sent with a group is block *f* XOR 1, the area most recently filled. Only the most recently uploaded chunk counts as resident: going back to the previous chunk uploads it again, even though it may still be in the other area. The first chunk change of a frame and later ones are scheduled through different queues (Medium).
- Front-end chunks (segment TBP 0x2A00: resource 532 and the picture packs 577 to 677) upload at 0x2A00 as stored, and their users (front-end sprite asset records, picture sprites) carry absolute TEX0 values. Resource 532's chunk covers blocks 0x2A00 to 0x31FF and a picture pack 0x2A00 to 0x2DFF, so they overlap; the front end uploads again before drawing (how often is open).
- A modern renderer can keep every decoded texture resident and ignore areas.

### Texture contents of fixed resources (front-end pictures and sky domes)

Confidence: High for the layout (all 101 packs and the four dome-format resources checked on disc); Medium for the buffer sizes.

- **Front-end picture packs (resources 577 to 677).** Which pack a screen shows is chosen by the rule in SPEC-0002 (picture = resource 575 + v). Each pack is one type-4 chunk whose table has two entries: one PSMT8 texture and one 16 by 16 PSMCT32 palette. The upload segment is 256 by 256 PSMCT32 (the pixel block of a 512 by 512 8-bit texture). The TEX0 sampling size is 512 by 512. The stored real size (sub-texture entry, offsets 4 to 6) is 512 by 448 in 97 packs and 512 by 440 in resources 586 to 588. Resource 594 is the exception: its segment is 256 by 128, and both its sampling size and real size are 512 by 128.
  - In 586 to 588 the u16 at entry offset 4 holds 33,280 (512 with bit 15 set). Under the entry layout above, bits 12 to 15 are the height modulo 16 (here 8), and with the byte at offset 6 they give a real height of 440. This bit is not a separate flag.
  - Resources 590 and 592 are byte-identical. Pack length on disc: 264,192 bytes (594: 133,120 bytes).
  - The front end reads a picture into one of two alternating buffers of 267,776 bytes each, and the read length it requests is 267,264 bytes, longer than a pack. The clean side should read the pack's own length. Only one picture read runs at a time (SPEC-0035).
  - These packs have segment TBP 0x2A00 and absolute sub-entry addresses (see Upload).
- **Sky domes (resources 1 to 3, and copy 0).** One texture chunk each, after the dome's shader list. Domes 1 and 2 (and resource 0): upload segment 128 by 192; three PSMT8 textures of 32 by 32, 256 by 256 and 128 by 128, and three 16 by 16 palettes. Dome 3: upload segment 256 by 128; the same three texture sizes with one 16 by 16 palette. The game reads a dome into a whole-resource buffer of 460,800 bytes, set up at boot; dome resources are 118,784 bytes (0 to 2) and 151,552 bytes (3). Their cloud materials scroll (UV mode 2, see "UV modes").
- **Resource 528 (dome layout, never loaded; SPEC-0002).** Upload segment 256 by 128; PSMT8 textures of 32 by 32, 256 by 256 and 32 by 32, one PSMT4 texture of 256 by 256, three 16 by 16 palettes and one 8 by 2 palette.

## Edge cases and error handling

- Upload segment count other than 1, a segment PSM other than PSMCT32, or a pixel block running past the chunk end: treat the chunk as malformed (none occur on this disc).
- A material record whose TBP/PSM matches no sub-entry in its texture chunks: 3 records on this disc, all in resource 528's malformed groups. Draw untextured.
- For the primary GS context of well-formed groups, if the palette address does not match a PSMCT32 sub-entry in selected chunk *k*, or if *k* is >= B, decode with a gray ramp and log it. The mode-2 second context has the exceptions below.
- Mode-2 groups, second GS context (TEX0 register 0x06): five groups have a texture address, palette address, or both that do not match chunk *k*. They are in resources 1148 (groups 86, 87, 89), 1247 (group 0), and 1327 (group 71); the records are duplicated in both frame-parity blocks. Each unmatched address does match a texture or palette entry in another type-4 chunk following the same shader list. However, group byte 0x28 selects one texture-chunk slot, and load-time rebasing applies that slot base to both GS contexts; no separate second-context chunk selector is established. The second context is active state: mode-2 mesh packets include a pass-B tag that draws with GS context 1, and group 71 in resource 1327 is referenced by the resource's level tree and a type-13 mesh child. Thus the address words are consumed and cannot all be dismissed as dormant format residue or unused groups. The static path still does not establish which texture contents occupy an unmatched address at draw time, so the original's resolution remains unknown. Do not search other chunks as a runtime fallback without further evidence.
- TFX 3 and ALPHA values 0x48/0x58 are rare but real; support them or fall back to ordinary blending with a warning.
- Front-end chunks: decoding them with base 0 and their absolute addresses reads the wrong memory; use the segment TBP as the base (Upload).

- Run-time upload builder, chunk count: when the segment total is an exact multiple of 0x7FFF, the formula still adds one chunk, so the last chunk may carry no units (Medium; from the formula). A clean upload must not write an empty final chunk unless a test shows the original does.
- Run-time upload builder, per-format factor: the rule gives 4 bytes per texel for every format other than the 8-bit and 4-bit indexed formats, so a 16-bit format would be sized as 4 bytes per texel. The disc uses only 32-bit segments, so this does not show on disc (Low-Medium).
- Sub-texture frame addressing, zero grid: a record with zero columns or rows would divide by zero. None occurs on disc, and the original's behavior is not checked. The clean side should treat such a record as invalid and leave the entry unchanged.
- Sub-texture frame addressing, rounding: when a texture size is not a multiple of the grid count, the cell size is rounded down and the remainder is unused.
- Sub-texture frame addressing, region-repeat bleed: the mask arithmetic is exact only when the cell size is a power of two and the cell origin is a multiple of it. A grid that does not divide the power-of-two size into power-of-two cells gives masked coordinates that bleed into neighbouring cells. Neither disc user hits this.
- Sub-texture frame addressing, frames: the variant 1 frame is the unsigned 16-bit counter, so it is never negative; it wraps from 65535 to 0.
- Variant 2, key of 6 or more: the frame is read from the limit table under key − 6 (quirk; the disc users have keys 0 to 4, so it does not show on disc).
- Variant 2 on disc: the 5 groups of resource 1260 always show cell 0, because nothing in play writes keys 0 to 4 except the set-up that zeroes them.
- Flipbooks during a pause: under game-state code 9 the cells hold while UV scroll keeps moving; on resume a variant-1 flipbook jumps to the current counter's cell.
- Skinned group with mask 0x3B and a UV mode: the UV-mode handlers place the UV slot by the mask, while program 2 reads fixed slots, so the handler would rewrite the bone-slot array. No such group exists on disc.
- Vertex program value above 6: the batch is not drawn.
- Billboard size (quirk): billboards are 1.25 times too wide and 1.071 times too tall compared with the 3D projection (see "Billboard program 3").
- UV scroll: the offset jumps when the 16-bit counter wraps (see "UV modes").
- Water animator: its phases advance per mesh draw, not per pass.
- Material slot check: with an unchanged eye point, a matrix change that keeps any one row keeps the old camera block (quirk).

### Edge cases: material group selection and binding

- Draw selection: an index of *N* or more is reported and replaced by 0; index bits above 16 are ignored.
- Billboard binding: a negative index addresses memory before the group table. The clean side should treat it as invalid.
- Billboard binding: a null context or list writes nothing and returns 0.

## Test vectors

Synthetic values:

- A sub-entry with TBP 0x3C, TBW 4, width 32, height 17 and PSM 0x13 (PSMT8). The sampling size is 32 × 32.
- A sub-entry with width 8, height 2 and PSM 0 (an 8 × 2 palette).
- CSM1 swap, for a 256-color palette: index 8 reads linear entry 16; index 16 reads linear entry 8; index 7 reads 7; index 40 reads 48; index 255 reads 255.
- Alpha: palette alpha 0x80 → 255 (opaque); 0x40 → 128; 0x00 → 0.
- A TEX0 value with TBP 0x2C, TBW 4, PSM 0x13, TW 5, TH 5, TCC 1 and CBP 0x38 is 0x0000_0705_5531_002C. Decoding that value must give back the same fields.
- TEX1 K field 0xFC0 (12-bit) = −64 / 16 = −4.0; 0xFFF = −1 / 16.
- Upload segment word 0 = 0x0004_0000: TBP 0, TBW 4 (a 256-wide segment: bits 16–21 hold 4). Word 0 = 0x0004_2A00: TBP 0x2A00, TBW 4. Word 0 = 0x0001_2A00 would be TBP 0x2A00, TBW 1 (a 64-wide segment). In such a chunk a sub-entry with TBP 0x2A10 lies 0x10 blocks into the upload.

Taken from the disc (end to end, upload to colour):

- **PSMT8 texel from a world texture.** World pack resource 1327. The shader list's group 1 has mode 1 and a TEX0 value of 0x8000_6405_9932_0250: TBP 592, TBW 8, PSM 0x13, TW 6, TH 6, CBP 800. The group's texture-chunk slot is 0, the first type-4 chunk (starts at resource offset 4,721,200). Its segment is 256 × 256 PSMCT32 with TBP 0 and DBW 4; its pixel block starts at resource offset 4,725,312; the matching sub-entry is 64 × 64 PSMT8. Texel (4, 0): page 0, block column 0, block row 0, so block 592; in-block u = 4, v = 0 gives byte offset 32; GS byte address 151,584. That byte sits in segment pixel (164, 128), so the index is stored at resource offset 4,857,040 and equals 137. Palette lookup: 137 is 10001001 in binary; exchanging bits 3 and 4 gives 10010001 = 145, which is pixel (1, 9) of the 16 × 16 palette at block 800: block 802, byte offset 12, GS byte address 205,324. That entry is segment pixel (65, 201), resource offset 4,931,396, bytes 177, 122, 142, 128. Final colour RGBA = (177, 122, 142, 255). Texel (0, 0) of the same texture has index 36 (bits 3 and 4 clear, no swap effect) and palette entry (179, 125, 145, 128), RGBA (179, 125, 145, 255).
- **PSMT4 texel from a font sheet.** Resource 532 (front-end database), the type-4 chunk starting at resource offset 156,348. Segment TBP 0x2A00 (10,752), 256 × 512 PSMCT32, DBW 4; pixel block at resource offset 171,200. Font 0's texture value is 0x8006_3C26_2142_2A00: TBP 10,752, TBW 8, PSM 0x14, TW 8, TH 8, CBP 0x31E1 (an 8 × 2 palette). Texel (26, 7): page 0, block address 10,752; in-block u = 26, v = 7 gives r = 3, c = 1, t = 38 + 16 = 54, no flip (c odd and r div 2 = 1 agree), nibble offset 128 + 54 + 1 = 183, which is byte 91 of the block, high nibble. GS byte address 2,752,603, which is segment pixel (2, 3) byte 3, at resource offset 174,283. The stored byte is 0x2B, so the index (high nibble) is 2. Palette entry 2 is pixel (2, 0) of the 8 × 2 image at block 0x31E1, at resource offset 663,528, bytes 15, 14, 14, 128. Final colour RGBA = (15, 14, 14, 255). Texel (8, 7) of the same sheet has index 0 and palette entry (26, 25, 25, 0): fully transparent.

Census (check, on disc): in all 191 game world packs, the texture count *B* stored in each shader list is consistent with the number of texture chunks in its pack (SPEC-0006 census). The front-end pack has no shader list and one texture chunk.

### Test vectors: UV modes (synthetic N, disc speeds where stated)

- Scroll, dome group 2 (resources 0 to 3): N = 1000: pass 0 offset (0.125, 0.0625), pass 1 (−0.0625, −0.125). N = 20000: pass 0 (0.5, 0.25). Dome group 1 (0.0000625, 0): one full texture width every 16,000 passes (266.7 s).
- Scroll wrap quirk: N = 65535 gives su × N = 8.191875 for su = 0.000125 (offset 0.191875); the next pass has N = 0 and offset 0.
- Environment map, M = identity: D = (0, 0, −1), Nv = (0, 0, 1): R = (0, 0, 1), u = 0, v = 0.5. D = (0, −1, 0), Nv = (0, 1, 0): R = (0, 1, 0), u = 0.5, v = 1.0. D = (1, 0, 0), Nv = (0, 1, 0): R = (1, 0, 0), u = 0.5, v = 0.5.
- Environment map, level geometry (world space): vertex at (0, 0, 0) with Nv = (0, 1, 0), camera at (0, 10, −10). D = (0, −10, 10), D · Nv = −10, R = (0, 10, 10), R^ = (0, 0.7071, 0.7071), so u = 0.1464 and v = 0.8536.
- Pause: with variant 1, divisor 2 and limit 16, a pause from N = 10 (cell 5) to N = 100 holds cell 5; the first update after it shows floor(100 / 2) mod 16 = 2.

### Test vectors: billboard program 3 (synthetic)

Centre on the view axis at w = 10, so the screen centre is (2048, 2048); extents (1, 1, 0.5), a = 1, b = 0:
- c1 = (1996.8, 2073.6)
- c2 = (2099.2, 2073.6)
- c3 = (1996.8, 2022.4)
- c4 = (2099.2, 2022.4)

With a = 0, b = 1: c1 = (2048 − 25.6, 2048 − 51.2) = (2022.4, 1996.8).

A centre with C.w = 1 (or less): the whole sprite is dropped.
- Water animator (disc data words): call 1: U offset 0.06, phase 0.2, alpha round(64 + 186 × 0.59933) = 175. Call 2: 0.12, phase 0.4, alpha 193. Call 17: U offset fmod(1.02, 1) = 0.02.
- CPU scroll (unused on disc): s = 40, N2 = 50: 50 × 40 / 4000 = 0.5.

### Test vectors: material slot check (synthetic)

- UV modes (0, 0): false.
- UV modes (1, 0), eye point moved: true.
- Eye point equal, matrix rows 0 to 3 all changed: true.
- Eye point equal, row 1 unchanged, the others changed: false (quirk).
- Eye point equal, no earlier camera block this frame for this parity: true.

### Test vectors: residency (synthetic)

Area base V. Ordinary chunk: packet A destination V, packet B destination V + 0x400; a state-block TBP of 0x40 on disc becomes V + 0x40 in block 0 and V + 0x440 in block 1.

Start with free area f = 0 and no chunk uploaded. Select a group of chunk X: X uploads into area 0, f = 1, block 0 is sent. Select another group of chunk X: no upload, block 0. Select a group of chunk Y: Y uploads into area 1, f = 0, block 1. Select a group of chunk X again: X uploads again, into area 0, f = 1, block 0. A group with no texture state: no upload, block 0.

### Test vectors: material group selection and binding (synthetic)

- Group table at base B: index 0 selects B; index 2 selects B + 1248 bytes (2 × 624).
- Draw selection with N = 4 groups: index 7 is reported and selects B (group 0).
- After the billboard binding of index 2, the 16-bit field at +68 and the 16-bit field at +148 of that group read 3 (pass-0 program 3 in both parity copies).

### Test vectors: sub-texture entries and a mip chain (disc)

- Entry size packing. Resource 504, the type-4 chunk at resource offset 9,412 (E 44, payload at 9,472): the table is at payload offset 512, the pixel block at 960 and the segment is 128 x 64 PSMCT32 with TBP 0 and buffer width 2 (32,768 bytes of pixels; 960 + 32,768 = 33,728 = the payload length). It has 8 sub-entries:
  - Entries 0, 1 and 2: the u16 at +4 is 0x0020 and the byte at +6 is 2, so width 32 and height 16 x 2 + 0 = 32; PSM 0x13. TBP 60, 44 and 36; TBW 4.
  - Entry 3: u16 0x0020, byte 4, so 32 x 64 (height 16 x 4); TBP 32, TW 5, TH 6.
  - Entries 4 and 5: u16 0x0040, byte 4: 64 x 64; TBP 16 and 0.
  - Entry 6: u16 0x0040, byte 2: 64 x 32; TBP 48, TW 6, TH 5.
  - Entry 7 (the palette): u16 0x0010, byte 1: 16 x 16, PSM 0, TBP 56, TBW 1.
- Height in the high nibble. Resource 678, the type-4 chunk at resource offset 1,978,432: entry 3 has u16 0x8008 and byte 0, so width 8 (low 12 bits) and height 16 x 0 + 8 = 8 (the nibble 8 is the height; it is not part of the width). PSM 0x13, TBP 135, TBW 8; its TEX0 value gives TW 3 and TH 3. Entries 40 to 46 of the same table have u16 0x2008 and byte 0: 8 x 2 palettes with PSM 0, TBW 2 and TH 1 (TW 3); entry 40 has TBP 613.
- Mip chain. Resource 1327, first shader list (127 groups, B = 3), group 4: mode 1, texture chunk index 0. TEX0: TBP 0, TBW 8, PSM 0x13, TW 7, TH 8, CBP 784. TEX1: LCM 0, MXL 3, MMAG 1, MMIN 4, K raw 4,016 = -80 / 16 = -5.0. MIPTBP1: TBP1 96, TBW1 8, TBP2 704, TBW2 8, TBP3 880, TBW3 8. The first texture chunk holds sub-entries at exactly those addresses: TBP 0 is 128 x 256, TBP 96 is 64 x 128, TBP 704 is 32 x 64 and TBP 880 is 16 x 32 (each half the previous size, all PSMT8). The palette at TBP 784 is 16 x 16 PSMCT32.
- Group 1 of the same list (the one in the PSMT8 texel vector above): MXL 1, so one mip level only. TEX1 K raw 4,032 = -64 / 16 = -4.0; MIPTBP1 holds TBP1 716 and TBW1 8, with TBP2 and TBP3 zero (unused). Sub-entry TBP 592 is 64 x 64 and TBP 716 is 32 x 32.

## Open questions

- Answered: material group choice by group index is specified in SPEC-0004.
- Resolved: group bytes 0x00 to 0xCF. They are the two grid records, the animator bytes and the two parity copies of the material record; 0x44 and 0x48 are the pass-0 vertex program and attribute mask, not culling or blend flags (see "Material record").
- Resolved boundary: the five mode-2 second-context groups listed in Edge cases match texture and palette entries in other chunks of their shader list, but the group selects one chunk slot for both GS contexts. No separate context-1 selection rule is established; the original runtime resolution remains open.
- Resolved: the texture-table word at offset 4 contributes the additional 48-byte record count in the packet-builder input layout; value 1 includes one record in the source-data prefix. Every on-disc texture table stores 1. The only identified caller handles decompressed startup-image buffers; no alternate value occurs on disc.
- How often the front end uploads the front-end pack (532) again over a picture (SPEC-0035). Answered: pack 532 uploads at 0x2A00 with absolute addresses and overlaps the picture area; the picture packs (577 to 677) are in "Texture contents of fixed resources".
- Resource 576 is 88 consecutive 76-sector book records, each with its own six-chunk chain and type-0 terminator. The earlier apparent extra TEX0 copies came from scanning past record 0 into the later records; no exact texture-entry TEX0 copies were found in the bytes after any record terminator. Those 432-to-944 trailing bytes remain unexplained.
- Resolved: **draw-order alpha**. The value compared with 128 is the character's fade level, not material alpha; no material sorts as translucent ("Translucency in the draw order"; SPEC-0046).
  - Resolved: **Shader-list texture-slot array and count**. The header owns the expected type-4 count at +0x1C and the texture-slot-array offset at +0x18. Both character and world-pack walkers relocate the array pointer, store each fixed-up type-4 value while below capacity, and log/drop overflow while continuing. The character walk also logs a final expected-versus-observed mismatch; this mismatch alone does not fail the character asset loader when its required records and capacity checks pass. The second register fix-up is the separate format-0x1B group pass gated by the group-array pointer at +0x10 (see Load-time handling, Shader list). **[High for the static loader paths]**
- Resolved: **texture-chunk destination**. Packet A uploads to area 0 and packet B to area 1 (see "When textures are resident").
- Resolved: **Shader-list registration**. There is no separate registration: the helper is the area-base fix-up itself, run when the group-array offset is nonzero, and the loader then runs the format-0x1B pass on the same condition. **[High]**
- **Chunk types not listed**: the dispatcher reports an unlisted chunk type as an error. Whether the walk continues after it is not established (SPEC-0006).
- Resolved: **Run-time upload builder destination and reachability**. The one identified caller submits two embedded startup images before the main frame loop and uses the DBP stored in each upload segment directly (0xE00 and 0). It does not use either ordinary texture-manager upload-area base. No gameplay caller was found. The prebuilt texture-chunk packets serve the game asset path; this caller is a separate startup-image path. **[High for the identified caller]**
- Resolved: **Run-time upload builder, unit of the chunk size**. Quadwords; each chunk is whole rows (see Chunking). **[High]**
- Resolved: **Run-time upload builder, per-format factor**. The 4-bit high and low nibble formats share the 4-bit size; the 16-bit and 24-bit formats are counted at 4 bytes per texel (quirk, never reached on disc). **[High]**
- **Run-time upload builder, base and placement**: the destination base used by the run-time builder (the prebuilt packets use the two upload areas, or 0x2A00 in the front-end chunks; see "When textures are resident") is not established.
- Resolved: **material slot check, record base**. The record is the group's material record at 0x30 (parity 0) and 0x80 (parity 1); record offsets 0x1C and 0x2C are the UV modes of passes 0 and 1.
- Resolved: **material slot check, match rule**. It compares the eye point and the four matrix rows with the last camera block of that parity (see "Material slot check").
- Resolved: **frame source (variant 2)**. The frame table at 0x36D020 is zero at boot and only an uncalled routine and the Draik Skeleton set-up (to 0) write it, so the 5 variant-2 groups of resource 1260 show cell 0; keys of 6 or more read the limit table (quirk).
- Resolved: **Resource 1260 variant-2 parts**. No routine reached in play writes frame-table keys 0 to 4. The only code that forms the frame table's address is the texture-animation reader, the five-band damage-stage writer (no jump, call or data word refers to it) and the Draik Skeleton set-up, which zeroes keys 0 to 4. So the five variant-2 groups of resource 1260 show cell 0 on every tick (High). Test vector: any tick of the Black Knight fight region: the frame read for key 0, 1, 2, 3 or 4 is 0.
- **Transfer-word helper**: whether it builds a draw tag (as the bit operations suggest) or a transfer (as another reading said), and which code calls it. Settle by reading its callers before any use.

- Answered: the draw context that gives the billboard group index belongs to the model spec; see SPEC-0004 (mesh leaf field 0x5C, material group index).
- Resolved: **written fields 0x44 and 0x94**. They are the pass-0 vertex program words of the two parity copies; 3 selects the billboard program.
- Resolved: **material record word at group offset 0x50** (0, 3 or 7). It has no reader; the clean side ignores it (see "Material record"). **[Medium-High]**
- **Billboard program**: which effect and world-node meshes reach the three loaders that force program 3. Answered: the element layout, corner arithmetic and reject rules are in "Billboard program 3". **[High for the arithmetic; Low for the users]**
- Resolved: **Environment-map space**: level geometry reflects in world space (M the identity, E the camera position). For placed objects M is the object's current block and E is the object's view-point vector (SPEC-0046, Placed-object draw frames). **[High for both inputs; Medium for the visible effect on placed objects]**
- Resolved: **culling**. No material field or vertex-program path culls back faces; the only rejection is the clip test of SPEC-0046 ("Per-triangle clipping in the vertex program"), and there is no far clip. **[High]**
- Answered: the world-state gate is the game-state code; see SPEC-0026, Game-state code (effect under code 9 in "When it runs"). **[High]**
- Resolved: **CPU vertex-animator enable**. Node byte +0x5E = 1 is the multi-packet leaf kind of SPEC-0004 (the 13 leaves of resource 9 that use animator 3). **[High]**
- Resolved: **program 2** is skinned and lit, not unlit. **[High]**
- **Counter off-by-one**: the render frame counter is sent to the vector unit at the end of a pass, so the batches of pass *k* probably see N = k; the exact offset is Medium.
- Answered: the faded-character composite (buffer size, placement, depth) is specified in SPEC-0046.
