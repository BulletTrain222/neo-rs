# SPEC-0002: Identifying resource types in GAMEFILE.DAT

- Answers: R-0002
- Status: Draft
- Disc build: SCUS-97367 v1.00 (same fingerprint as SPEC-0001)
- Confidence: per section. Chunk container layout: High. Family classification: High. Load routes of fixed resources: Medium (per line). Meaning of each chunk type: Low to Medium. Per-type steps of the chunk-tree walk (shader list, texture and object-container handlers): Medium. Boot pictures in the executable and the unused whole-resource expansion: High.

## Summary

Resources have **no single type tag**. The game knows what it is loading from *which* resource number it asks for. Every loader is written for one kind of data. On the disc, though, the bytes fall into a few recognizable families: a tagged chunk container ("IDM" chunk files, which hold models, textures, levels, and so on), two Sony ADPCM audio formats, sound banks, and a few headerless binary families. No whole resource is compressed. Chunk files, sound banks and audio are stored plain. Compression appears in one place only: a block inside an animation clip entry (see SPEC-0005). Such entries occur as top-level clip chunks, as clip chunks nested inside a type-32 container chunk, and in some headerless clip banks. Test facts: a scan of every 4-byte position in every non-empty resource found 51,048 compressed clip blocks and no other LZO1X block. Two more compressed blocks sit in the executable's own data: the two boot pictures (see Boot pictures in the executable). With this spec the clean side can classify every resource, walk chunk files, and route each chunk to a decoder by chunk type.

## Where it lives on the disc

Every resource is a sector-aligned slice of `GAMEFILE.DAT`, located through SPEC-0001. The table below lists the families by resource-number range on this build. The ranges were measured from the disc. Classify by bytes, not by range; the ranges are only a cross-check.

| Family | How to recognize (first bytes) | Count | Resource numbers |
| --- | --- | --- | --- |
| Empty | index count = 0 | 76 | 4–6, 8, 275–346 |
| Chunk file, v1 | bytes 0–2 = ASCII `IDM`, byte 3 = 1 | 823 | 0–3, 7, 9, 504–519, 528, 532, 534–576, 678–1433 |
| Chunk file, texture pack | `IDM`, byte 3 = 4 | 101 | 577–677 |
| Chunk file, odd | `IDM`, byte 3 = 0xCD | 1 | 533 (design tables and path records; text is in their string pools, SPEC-0024) |
| Sound bank (v2) | first u32 LE = 1; ASCII `SBv2` at +0x20 | 134 | 10–143 |
| VAG mono ADPCM | ASCII `VAGp` | 21 | 144–145, 256–274 |
| Stereo ADPCM stream | ASCII ` KPV` (space K P V) | 110 | 146–255 |
| Sound block | first u32 LE = 3; ASCII `SBlk` at +0x18 | 144 | 347–490 |
| Headerless clip banks | none | 7 | 493, 495, 497, 499, 501, 503, 526 (843 clip entries; SPEC-0005) |
| Headerless skeleton headers | none | 6 | 492, 494, 496, 498, 500, 502 (SPEC-0005) |
| Plain ASCII text | none | 1 | 527 |
| Headerless hero action tables | none | 6 | 520–525 (role Medium) |
| Headerless blob, meaning unknown | none | 1 | 491 (51,200 bytes; its size equals the save array rounded up to whole sectors, but its contents are not a save image and no loader reads it; role unknown, Medium) |
| Headerless item/drop configuration | no magic; little-endian item-record count 208 at +0x00 | 1 | 529 (format: SPEC-0021, “Item config table”) |
| Headerless species/monster table | little-endian row-type count 112 at +0x00, followed by 112 first-row indexes | 1 | 530 (row schema: SPEC-0024 and SPEC-0029) |
| Headerless villager/companion table | same count-and-index header; 0x60-byte rows | 1 | 531 (row schema: SPEC-0024) |

## Layout

### Chunk file ("IDM" container), Confidence: High

A chunk file is a chain of sibling chunks. The first one starts at resource offset 0. Each chunk has a 16-byte header:

| Offset | Size | Type | Endian | Meaning |
| --- | --- | --- | --- | --- |
| 0 | 4 | bytes | — | Tag. In the first chunk, bytes 0–2 are always `IDM` and byte 3 is a format byte (1, 4, or 0xCD; see the family table). Most child chunks repeat `IDM` + 1; texture chunks inside texture packs use `IDM` + 4. The game checks only the 3 letters. |
| 4 | 2 | u16 | little | Chunk type (see below). 0 marks the end of the chain. |
| 6 | 2 | s16 | little | Extra header bytes *E*, a multiple of 4, seen 0…124 |
| 8 | 4 | u32 | little | Chunk size *S* in bytes, header included: the distance from this chunk's start to the next sibling. Always a multiple of 4. |
| 12 | 4 | u32 | little | Not used by any loader described here. Often a tool fill pattern (low 16 bits 0xCDCD) or 0/1. Ignore it. |

- Payload starts at `chunk + 16 + E` and ends at `chunk + S`.
- Next sibling = `chunk + S`.
- The chain ends when the next sibling's type field is 0 (the zero padding after the last chunk), or when the next header would run past the readable length (`count × 2048`).
- Some chunk payloads hold their own nested chunk lists in the same format. For example, a type-8 chunk in character files holds sub-chunks, one of which has type 12.

### Walk rules per container kind (from the real bytes)

- v1 chunk files: 791 of 823 end on a type-0 header. 32 (resources 544–575) end on their single type-13 chunk, followed by bytes that are not chunks. None ends exactly at the sector-rounded end.
- **Type-13 resources** are 33 resources holding 65 type-13 chunks in total. Resources 544–575 hold one chunk each, and whatever follows it is not chunk data. Resource 7 holds 33 type-13 chunks, after its type-2 chunk and before its type-4 chunk.
- **Texture packs** (`IDM` + 4) are a single type-4 chunk with E = 112. It is followed by a filler area of repeated 0xEE bytes, not a chunk. Stop after the first chunk. Each pack's chunk holds one 512 by 512 texture with 8-bit indexed pixels and one 16 by 16 palette; resource 594 is 512 by 128 (High; SPEC-0003). The packs are the front end's full-screen pictures (see Behavior).
- Rule of thumb that matches every resource on the disc: walk siblings. Stop on type 0, on running past the readable length, or on an implausible header (size 0, size not a multiple of 4, or size past the readable length). Texture packs and resources 544–575 are walked with a one-chunk limit. Resource 7 has no one-chunk limit and is walked to its end.

### Chunk types seen

Counts are over all top-level chunks on the disc. The meanings come from loader behavior and diagnostics; confidence is noted for each.

| Type | Seen in | Meaning | Confidence |
| --- | --- | --- | --- |
| 1 | character files | ignored by the loader | Medium |
| 2 | character files, world packs | shader/material list. It states how many texture bitmaps ("bits") it expects. | Medium |
| 4 | everywhere | texture bitmap (format: SPEC-0003) | Medium |
| 8 | character files, world packs | container whose children are chunk types 12 and 13. Each child header has a separate u16 role id at byte 14 (+0x0E); type 12 is a child chunk type, not a role id. The character outfit loader calls its first type-12 child the COREOBJ. At most one type-8 container per character file. | High for layout; Medium for loader role |
| 13 | 544–575 (one chunk each), 7 (33 chunks) | mesh container: one node tree per chunk, in the SPEC-0004 node format (root group, visibility group, LOD group, mesh leaves). The SPEC-0004 node walk covers every payload byte of all 65 chunks. Not float data. Resources 544–575 are held-item display models drawn with resource 7's material list (see Behavior). | High (layout); High that resource 7 loads at boot with the item data; High for the 544–575 load rules |
| 18 | character files, model sets | model geometry | Medium |
| 31 | world packs | world-pack header; carries an ASCII name such as "IdolWorld" or "FrontendData" | Medium |
| 32 | 165 resources; 86 of them with clip children | container. Its payload is a nested chunk file. Its children include clip chunks (types 25, 26, 27; SPEC-0005): 289 such children (111, 85, 93), with 290 linked entries. | Medium (container); High (clip children, by walk) |
| 33 | character files and model sets (nested under type-18 wrappers in 571 of 1,339; flat in outfit resources 504–519) | optional per-model block, layout in SPEC-0004 | Medium (chunk and placement); Low (use) |
| 38 | front-end pack (resource 532) | front-end database (SPEC-0035) | High (layout) |
| 15 | world packs | tagged lookup tables and the entity-id and placement-id tables (SPEC-0006) | High (layout) |
| 2000, 2001 | book records (resource 576) | book text (2000) and book glyph layout (2001); see SPEC-0024, Book records | High (layout) |
| 3 | world packs | level display tree, world space (SPEC-0004) | High |
| 5 | world packs, placed objects | model set: height patches and frame records (SPEC-0006, SPEC-0018) | High |
| 10 | world packs | ladders and climb panels (SPEC-0006) | High |
| 11 | world packs | object header: entry records, collision primitives and teleport planes (SPEC-0006, SPEC-0028) | High |
| 16 | world packs | box records, foliage patches (SPEC-0006, SPEC-0060) | High |
| 19 | world packs | spatial grid (SPEC-0006, SPEC-0018) | High |
| 20 | 23 world packs | instanced meshes (SPEC-0004) | High (layout) |
| 21 | 135 world packs | navigation meshes (SPEC-0018) | High |
| 22, 23, 24 | character regions | skeleton definitions: main, sub-parts, face (SPEC-0005) | High |
| 25, 26, 27 | character regions | animation clip entries paired with types 22, 23, 24 (SPEC-0005) | High |
| 28 | world packs | camera track records (SPEC-0052, SPEC-0028) | High |
| 29 | world packs | placed objects (SPEC-0028) | High |
| 30 | world packs | light sets (SPEC-0028) | High (layout); Medium (use) |
| 34 | resource 9 and 4 world packs | effect definitions; resource 9 holds the shared table (SPEC-0034) | High (resource 9); Low (the small world-pack groups) |
| 35 | world packs | binary zone-visibility grids (SPEC-0006) | High |
| 36 | 575 character regions | character region table: characters, animation tables, items, effects (SPEC-0004, SPEC-0005, SPEC-0006) | High |
| 37 | 173 world packs | level script module, "SNR2" container (SPEC-0030) | High (container) |
| 39 | world packs | placed effect spawns (SPEC-0034, SPEC-0028) | High |
| 40, 41 | world packs | switch-keyed entries; atmosphere zones (SPEC-0028, SPEC-0006) | High (layout); Medium (use) |
| 42, 43 | resource 533 (17 and 3 chunks) | design tables (42) and path records (43) (SPEC-0024) | High (layout) |
| 1000 | world packs | dialogue line table (SPEC-0022, SPEC-0028) | High |
| 1001 | world packs | zone id lookup (SPEC-0006) | High |

Typical chains (type sequences):

- Character outfit: 2, 4, 18, 8, 33.
- Model set: 26, 23, 27, 24, 18, 36, or 25, 22, 18, 36, with repeats.
- World pack: 31, 2, 3, 8, 19, 5, 10, 11, …, 1000, 1001, …, 15, 32, then several type-4 textures.

### VAG (`VAGp`), Confidence: High for numbers 256–274

Header fields are **big-endian**:

- +0x04 version (32 in resource 144; 0 in resource 145)
- +0x0C ADPCM data size in bytes
- +0x10 sample rate (22,050 Hz in all 21)
- +0x20 a 16-byte NUL-terminated name

The ADPCM data starts at +0x30. For 19 of 21, `count == ceil((size + 0x30) / 2048)`. Numbers 144 and 145 have a VAG header that doesn't describe the whole resource; number 144 is 10,056 sectors long. Resources 144 and 145 are the emote clip banks: one VAG header, then many clips addressed by directory and position (SPEC-0009, Emote voice banks; SPEC-0024, Path records). Resource 145 also has header version 0 where the others have 32.

### Stereo stream (` KPV`), Confidence: Medium

Header fields are little-endian u32:

| Offset | Value seen |
| --- | --- |
| +0x04 | a size *S* (bytes per channel, probably) |
| +0x08 | 2048 (interleave, probably) |
| +0x0C | 0xA000 |
| +0x10 | sample rate: 22,050 Hz (104) or 44,100 Hz (6) |
| +0x14 | channel count: 2 in all 110 |

+0x20 to +0x3F are zero. The resource is about 2 × S plus slack long. The exact data start and the size relation are not confirmed.

### Sound banks and sound blocks, Confidence: Medium

Both families begin with a small little-endian u32 header followed by a tagged block:

- **Sound bank (v2):** the first u32 is 1. The third u32 is 0x20, which is where the `SBv2` tag sits.
- **Sound block:** the first u32 is 3 and the second is 2. The third u32 is 0x18, which is where the `SBlk` tag sits. Three ascending offsets come before the tag.

In both, the first u32 looks like a section count and the third u32 looks like the offset of the first section. This is not yet verified for every section.

The sound system loads and streams these by resource number, and it treats them as banks. Format: open question.

### Misc. headerless resources, Confidence: Medium for the flat-chain layout; High for the entries

Seven headerless resources (493, 495, 497, 499, 501, 503, 526) are flat clip banks: a bank header at byte 0, then clip entries linked by next pointers, with no chunk header. They hold 843 entries, 148 of them LZO-compressed. Each compressed entry is one LZO1X block, and each decodes to exactly its stated size. Six headerless resources (492, 494, 496, 498, 500, 502) are skeleton headers with no wrapper (SPEC-0005). Resources 729, 1119, 1165 and 1166 are chunk files, not headerless. Resources 520–525 are hero action tables, and resource 491 is a save-size blob that holds no model (High for the structure; Medium for these roles). A separate count of 153 compressed entries includes five entries that could not be placed in these banks; that difference is open.

## Behavior

- The game does **not** probe a resource's bytes to decide its type. Each loader asks for a specific resource number (a constant, or one taken from game data) and expects one kind of data there. So the clean side should model resource numbers as typed slots once later specs map them. Byte-based classification is for tooling and validation.
- **Resources that hold textures, meshes or skeletons** (counted by a walk of every resource with nested chunk files followed to depth 6; High): 926 resources. 882 hold texture chunks (type 4), 812 hold mesh chunks (types 8 or 13), and 679 hold skeleton data (chunk types 18 or 22 to 24, or one of the twelve headerless hero resources 492 to 503). Audio, text, save data and the item and monster tables are not among them.
- **Fixed resources and the kind of routine that loads them** (Confidence per line):
  - 492 to 503: loaded in play by the hero set-up. They hold the bone headers and clip banks of the two heroes' three families each. Medium-High.
  - 526: a clip bank with no bones and no mesh, loaded together with header 492 by a second hero set-up. Medium.
  - 504 to 511 (first hero) and 512 to 519 (second hero): outfit chains, loaded by the outfit routine as a base number plus an offset taken from the equipped item. Medium-High (whether every outfit offset is reachable through an item is open).
  - 7 and 9: loaded at boot as part of the item data. Resource 7 holds 33 type-13 mesh containers; resource 9 holds five chunks in this order: a shader list, a type-8 mesh container, a type-34 effect-definition chunk of 837,856 bytes (SPEC-0034), and two textures (the effect sprite sheets). High for 7; High for 9.
  - 532: the front-end world pack, loaded by the front end (SPEC-0035). Medium.
  - 576: the book records, loaded by index by the book reader (SPEC-0024). Medium-High.
  - 534 to 543: pet packages, one per pet kind k at 534 + k, each holding one character's models, skeletons and clips. A package loads only when a saved pet of that kind is present. No known routine creates a first pet, so whether these load in a normal game is open. Medium for the route; Low for use in a normal game. The pet buffer is sized at boot from the largest of 534 to 543, excluding 536. High for the size rule; Medium for the reason (kind 2 is never activated, so package 536 never needs to fit).
  - 1, 2 and 3: the sky domes. Each holds a shader list, one mesh container with two mesh children, and one texture chunk. The game loads one per world, chosen by the world's slot index (its row in the 193-row world table): dome 2 for slots 151 to 156 and slot 186, dome 3 for slots 157 to 185, dome 1 for every other slot. When the chosen dome equals the one already loaded, nothing is read. Resources 1 and 2 are byte-identical, so dome 2 looks the same as dome 1. High.
  - 544 to 575: held-item display models, one standalone type-13 mesh container each, drawn with the material list of resource 7 (SPEC-0004). Four display slots refresh them (SPEC-0021, "Equipment display indexes and item display models"). High for the rules; Medium-High for the item names.
    - Weapon: index w = item id − 98 for ids up to 110, item id − 103 for ids 111 and above. For 0 ≤ w ≤ 15 the slot loads 544 + w (544 to 551 the eight swords, items 98 to 105; 552 to 559 the eight wands, items 111 to 118).
    - Shield: index s = item id − 106, used for 0 ≤ s ≤ 4; the slot loads 560 + s (items 106 to 110).
    - World item: when an item is spawned in the world and its index n inside its item-id range is 50 to 59, the slot loads 515 + n. The ten potions (items 50 to 59, n = id) give 565 to 574.
    - Resource 575 is the display model of item id 36. It is loaded once, on the hero's first update, whatever the inventory holds.
  - 577 to 677: the front end's full-screen pictures, one texture pack each (SPEC-0003). The front-end pack (resource 532) has an asset-path section of 136-byte records. A record whose field at +0x08 holds −1 is a picture not yet loaded, and its field at +0x0C holds the picture number v (2 to 102). The picture is resource 575 + v. 101 records carry such a v, each value once, so the 101 packs map one to one. 577 to 600 are menu and lead-in screens (options, inventory pages, title, load, save, game over, credits, controls, legal, logo); 601 to 677 are the 77 area-map backgrounds, picture 601 + map id (SPEC-0035). Resources 590 and 592 are byte-identical. High for the rule.
- **Resources with no known loader.** Only two fixed resources are requested by no load route, so on this disc they are taken as never loaded:
  - 0: a byte copy of domes 1 and 2. The dome rule never selects it (its smallest result is 1). High for the copy; Medium-High for "never loaded".
  - 528: a chunk file with the dome layout (a shader list, one mesh container with 13 mesh children, two 256 by 256 textures and two 32 by 32 textures with palettes). Five of its material groups lack the GIF tag (SPEC-0003). No loader names it. Medium-High.
  The clean side should load them only on request, like any other resource.
- The character and world loaders check that the first 3 bytes are `IDM`. If they aren't, the character loader reports an error (text not recorded) and loads nothing. Then they walk the sibling chain and dispatch on chunk type. The character loader reports and skips an unknown chunk type. A character file must contain a model (18), a core object (inside 8), and a shader list (2). The shader list must come before the textures, and the number of type-4 textures must equal the count the shader list states. Otherwise the loader reports an error.
- **Per-type steps of the chunk-tree walk** (Confidence: Medium). Each chunk is handled when the walk reaches it, in chain order. The walk checks the first three bytes, then for each sibling:
  - **Type 2 (shader list).** It becomes the current shader list. Its offsets are fixed up. The texture count it states is kept, and it is checked against the number of texture slots filled by the type-4 chunks that follow it (the check described above).
  - **Type 4 (texture).** The texture is fixed up and stored in the next free texture slot of the current shader list, in walk order. The type-2 shader-list payload owns the texture-slot capacity at offset 0x1C and the relocated slot-array pointer at +0x18. Both world-pack and character-region walks log and drop a type-4 texture after capacity while continuing; the character walk logs a final count mismatch, which alone does not fail loading when required records and capacity checks pass (SPEC-0003 and SPEC-0006).
  - **Type 8 (object container).** Its children are chunk types 12 or 13. The chunk type is the u16 at header byte 4; the distinct role id is the u16 at header byte 14 (+0x0E). In the world loader, each ordinary child is relocated and registered under that role id; role-id values 1, 2, 3 and 10 are among the keys seen. Role id 401 takes a special world path. In the character loader, the first child becomes the root and later children with role ids 10–20 fill extra-part slots. Thus 1, 2, 3 and 10 are keys/role ids, not child chunk types; type 12/13 and role id are separate fields on a child. Other role meanings remain open (SPEC-0004, Mesh container).
  - **Other types.** The character loader reports the type and skips it (see above).
  - Order: type 2 must come before types 4 and 29 (SPEC-0006 ordering rule).
  - **Relation to SPEC-0006.** SPEC-0006 ("How the loader handles each chunk") lists the world-pack handlers for types 2, 4 and 8 and the pack's order rules. This section gives the character walk's steps and the shared texture-slot overflow behavior; the character walk also logs the final expected-versus-observed count mismatch.
  - **Header fields.** The field at +6 is the extra-header length E in the layout table (payload = chunk + 16 + E). The sibling walk also treats it as the payload offset. Both describe the same two bytes; the layout table is authoritative.
- Two chunk types carry a data version. The original checks it, but a mismatch never stops a load: the warning goes to a print routine that does nothing in the retail build, and loading continues.
  - World-pack header (type 31): payload u32 at +0 is expected to be 2 and u32 at +4 to be 1. 191 of 192 type-31 chunks match. Resource 532 has 2 and 0 and loads anyway.
  - Front-end tables (type 38): payload u32 at +0x10 is expected to be 0x00050001 (shown as version 1.05). The only type-38 chunk (in resource 532) matches.
  - The clean side should log a mismatch and continue, the same as the original.
- Header bytes 12–15 are not a version field. They are build-tool fill (low half 0xCDCD) for most types, 1 or 0 for some, and 01 01 00 00 for every texture chunk. No loader reads them.
- A chunk with type 32 is a container. Its payload is a nested chunk file. In 86 of the 165 resources that hold one, its children include clip chunks (types 25, 26, 27). The container itself is not compressed.
### Boot pictures in the executable (High)

- The boot program's data holds two compressed blocks, reached through two pointers at data addresses 0x2DA304 and 0x2DA308 (blocks at 0x2E8F70 and 0x2DFB90; compressed sizes 8,544 and 37,847 bytes). Each starts with its decoded size and its compressed size, then one LZO1X block. Each decodes to 917,568 bytes.
- Decoded layout: a 64-byte upload header, then a 512 by 448 picture at 4 bytes per pixel (red, green, blue, alpha; alpha 0x7F throughout), 917,504 bytes. The header holds one upload segment: the video-memory destination (the second display buffer for the first block, the first display buffer for the second), the buffer width 512, the picture size 512 by 448, and a 16-byte source file name.
- Content: the first block is the "Sony Computer Entertainment America Presents" page (white text centred on black, 25 colours; source name sceapresents.tif). The second is the developer page: "Developed By" over the Idol Minds logo on a near-black background (255 colours; source name IdolMinds.tif).
- When they show is in SPEC-0026 (Visible boot order). The clean side may ship equivalent pictures as its own assets; it needs only the two pictures and their timing.

### Whole-resource expansion (High)

- The read queue has an expand-on-completion flag that would expand a finished read with the LZO1X rules. Every place that queues a read passes 0 for it, so no load in this build expands a whole resource. This agrees with the disc scan: no resource is a whole-resource compressed block. The clean side needs no whole-resource expansion.

- Byte entropy of every resource is at most about 7.1 bits per byte (measured over all 1358 non-empty resources). That check cannot show that a compressed block is absent. The scan in the Summary is the stronger test, and it covers LZO1X only.

## Edge cases and error handling

- Chunk size 0, or a size that runs past the readable length: stop walking and treat the resource as malformed. The game has no guard and would read out of bounds.
- The chain end relies on zero padding after the last chunk. When the last chunk ends exactly at the sector-rounded end, stop at the buffer end. No resource on disc ends this way; the rule is a guard.
- Treat negative or odd *E* values as malformed. Only multiples of 4 from 0 to 124 occur.

## Test vectors

Synthetic:

- Bytes `49 44 4D 01 | 12 00 00 00 | 20 00 00 00 | 00 00 00 00 | <16 payload bytes> | 49 44 4D 01 | 04 00 08 00 | 30 00 00 00 | 00…` followed by a 16-byte header with type 0 gives:
  - chunk A: type 18, E = 0, size 0x20, payload at 0x10, length 16
  - chunk B at 0x20: type 4, E = 8, size 0x30, payload at 0x38, length 0x18
  - end of chain at 0x50
- A buffer whose first bytes are `49 44 4D 07 …` still passes the `IDM` check (the 4th byte is ignored).
- A buffer that starts with the ASCII tag `VAGp`, with a data size of 4096 at +0x0C and a sample rate of 22,050 Hz at +0x10, is a VAG with 4096 data bytes at 22,050 Hz.
- Shader-list count (synthetic): the type-2 payload states capacity 2 at +0x1C and has a texture-slot array pointer at +0x18. Following type-4 chunks A and B fill slots 0 and 1. A third type-4 chunk C is fixed up, dropped without a slot write, and logged; the observed count advances to 3, so the character walker reports expected 2 versus saw 3 when the chain ends. The character asset loader still returns success if its other required records and capacity checks pass.

Executable (High):

- Boot block 1: compressed 8,544 bytes, decoded 917,568 = 64 + 512 x 448 x 4. Its picture's most common pixel is black (0, 0, 0, alpha 0x7F), 224,624 of 229,376 pixels; it has 25 distinct pixel values.
- Boot block 2: compressed 37,847 bytes, decoded 917,568. Its most common pixel is (12, 12, 12, alpha 0x7F), 201,312 pixels; it has 255 distinct pixel values.
- Both headers start with the segment count 1; the destination of block 1 lies 917,504 bytes (one 512 by 448 by 4 picture) above that of block 2.

Disc scan (test facts for the compressed-block check, LZO1X only; High):

- Compressed clip blocks found: 51,048 in total. The breakdown is open. The seven headerless banks hold 148 compressed entries (SPEC-0005). The top-level and nested clip chunks counted directly total 1,786 and 289, so the breakdown of 47,048 top-level and 3,847 nested compressed clip chunks does not reproduce.
- Scan of every 4-byte-aligned position in every non-empty resource (1,902,385,152 bytes), accepting a position only if the decoder gives exactly the decoded size and consumes exactly the compressed size: 51,048 hits. Limits: decoded size at least 64 bytes and compressed size at most decoded size.

## Open questions

- Resolved: the chunk types once listed as unidentified are identified in the chunk-type table, each with its owning spec. Resource 491 stays unidentified (no loader, contents not a save image).
- The resource number → typed slot mapping (which loader asks for which number) for resources outside the fixed set, the world packs and the character regions. Answered for those: Behavior ("Fixed resources and the kind of routine that loads them" and "Resources with no known loader"), SPEC-0006 and SPEC-0004.
- Answered: the odd VAGs 144 and 145 are the emote clip banks; see SPEC-0009, Emote voice banks.
- Resolved: **Stereo-stream (` KPV`) data start and size relation.** The stream header and interleave layout are in SPEC-0009, “Stereo stream”; playable block boundaries and the observed audible tail beyond the nominal per-channel size are further bounded in SPEC-0031, “KPV silence.” The samples begin at byte 0x800. The header's size is bytes per channel; do not infer a resource length from `2 × S` because some streams contain additional audible blocks and all resources are sector-padded.
- Resolved: **Sound-bank (`SBv2`) format.** The complete bank header, section pointers, directory, voice records, sample extents and embedded music container are specified in SPEC-0031, sections 1–6 and 8. This identification section retains the short tag signature for classification.
- Resolved: resources 529–531 are headerless structured data, not IDM chunks: resource 529 is the item/drop configuration blob (SPEC-0021, “Item config table”); resource 530 is the species/monster table and resource 531 is its villager/companion table (SPEC-0024, “Species and villager tables”; row use is further specified in SPEC-0029 and SPEC-0038). Answered: 520–525 and 491 are identified in the format table; 729, 1119, 1165 and 1166 are chunk files, not headerless.
- Whether the pet packages 534 to 543 load in a normal game (that depends on whether play can create a first pet), and whether 528 was meant as a fourth dome or as front-end scenery. Answered: 1 to 3, 544 to 575 and 577 to 677 have loaders (Behavior); 0 and 528 have none.
- Resolved: **Boot-time data.** The executable's two compressed blocks are the publisher page and the developer page, each a 64-byte upload header and a 512 by 448 picture uploaded to video memory at start-up (Boot pictures in the executable; High).
- Resolved: **Whole-resource expansion step.** No place sets the load job's expand flag; every queued read passes 0 for it, so the step never runs (Whole-resource expansion; High).
- **Other whole-resource compressors.** The scan covered LZO1X only. The whole-resource entropy survey does not rule out every codec (for example RLE); identified ADPCM families are audio encodings, not whole-resource wrappers. No unsupported claim that all possible compressors are absent is made.
- Resolved: **Nesting depth.** Across all 1,434 resource slots, 165 type-32 containers were found, all at resource top level. Their nested IDM chunk streams contain 925 direct child chunks and reach at most three child-stream levels below the container (848 chunks at level 2 and 284 at level 3); no fourth or deeper child-stream level occurs. All 529 nested streams end at a type-zero chunk terminator. [High for valid nested IDM streams in this disc build.]
- Resolved: **Shader-list texture count owner and extra-texture handling.** The type-2 payload owns the expected type-4 count at +0x1C and the relocated texture-slot array pointer at +0x18. The character walk fixes up every type-4 chunk, stores it while the observed count is below the capacity, and otherwise logs and drops the chunk while continuing the walk. At the end it logs an expected-versus-observed count mismatch. That mismatch alone does not fail the character asset loader: it returns success when its required records and capacity checks pass. The same count/capacity fields and slot behavior apply to the world-pack walk (SPEC-0003, Load-time handling of shader lists and texture chunks; static loader behavior, High).
- **Resolved: type-8 child terminology and role-id path.** Child chunk types 12/13 are read from header byte 4, while role IDs are read separately from header bytes 14–15 (+0x0E). World registration uses the role ID as its key; character extra-part slots use role IDs 10–20 after the first child is selected as root. SPEC-0004's type-13 outfit dispatch values 0/100/200/400 are also role IDs (SPEC-0004, Mesh container). The meanings of other role IDs remain open.
- Resolved: **Type-2 second texture-register fix-up.** The character type-2 loader runs the same conditional second format-0x1B register pass as the world-pack handler, after the primary shader-list fix-up when the group-array pointer at +0x10 is nonzero. The pass adds the texture manager's current allocation position to the texture base and adds zero to the palette base. This condition is the group-array pointer, not the +0x18 texture-slot-array pointer (SPEC-0003, Load-time handling of shader lists and texture chunks; High).


