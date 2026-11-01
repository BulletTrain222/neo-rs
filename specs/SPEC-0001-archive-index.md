# SPEC-0001: GAMEFILE.DAT archive index

- Answers: R-0001; proactive (open close-out)
- Status: Draft
- Disc build: SCUS-97367 v1.00. Check before trusting fixed locations:
  - `SCUS_973.67` SHA-256 = `a8cc770cf6157f7a67672eea71ed1b631bf780e7d684802956ccab24b71d8f25` (required)
  - `GAMEFILE.DAT` SHA-256 = `8a25924d197693052382380e52115b6c9f32f21a4d2eaef2f61ea4fdd5ad98c7` (optional; 1.9 GB, so slow)
  - `SYSTEM.CNF` SHA-256 = `436c88f3136e4b4d9a942653f0cedd7bbbfa6bdf79acf0fcf614c19c20b17f19` (optional, cheap)
- Confidence: High (the layout matches the real bytes exactly; see Behavior). Medium for the edge cases.

## Summary

GAMEFILE.DAT has no header and no index of its own. It is a plain run of 2048-byte sectors. The index lives in the boot executable `SCUS_973.67`: a fixed table of 1434 entries. Each entry gives a start sector and a sector count. A resource number is a zero-based position in this table. With this spec the clean side can list, size, and read every resource straight from the ISO.

## Where it lives on the disc

- Index: inside ISO file `SCUS_973.67` (the boot ELF). The table starts at ELF virtual address `0x002D7618`.
  - The ELF has exactly one loadable segment. It maps file offset `0x80` to virtual address `0x00100000`. So the table is at **file offset `0x001D7698`** in `SCUS_973.67`. Convert as: file offset = VA − 0x00100000 + 0x80. Better still, read the program header and do the conversion yourself, so a wrong build fails loudly.
  - Table size: 1434 × 8 = 11,472 bytes.
  - The entry count (1434) is not stored on the disc next to the table. The game uses it as a built-in constant. The clean side should hard-code 1434 for this build, gated on the fingerprint.
- Data: ISO file `GAMEFILE.DAT`, 1,902,385,152 bytes = 928,899 sectors of 2048 bytes. On this ISO its extent begins at LBA 885,373. Look this up through the ISO9660 directory anyway; don't hard-code it.
- The executable also names a `GAMEFILE.BLS`. That file is **not on the retail disc**. It was a development-time, host-side name list. Ignore it.

## Layout

Index entry (8 bytes, repeated 1434 times, no header, no terminator):

| Offset | Size | Type | Endian | Meaning | Valid range |
| --- | --- | --- | --- | --- | --- |
| 0 | 4 | u32 | little | Start sector, relative to the first sector of GAMEFILE.DAT | 0 … 928,899 |
| 4 | 4 | u32 | little | Length in 2048-byte sectors | 0 … 10,056 seen |

- Sector size: 2048 bytes (the ISO's logical block size). All resources start on a sector boundary.
- Byte offset of resource *n* in GAMEFILE.DAT = `start[n] × 2048`. Readable length = `count[n] × 2048`.
- The index gives **no exact byte length**. A resource's true length comes from its own header, if it has one (see SPEC-0002). The tail of the last sector may hold leftover bytes from the build tools, not zeros.

Invariants on this build (good for self-checks):

1. `start[0] == 0`.
2. `start[n+1] == start[n] + count[n]` for every n: the entries are back to back, with no gaps and no overlaps.
3. `start[1433] + count[1433] == 928,899`, the exact sector count of GAMEFILE.DAT.
4. Exactly 76 entries have `count == 0`.

## Behavior

- Resource numbers are table positions 0…1433. Game code asks for a resource by this number. Some numbers are fixed constants in the code. Others come from game data tables, which later specs will cover.
- To load resource *n*, the game:
  1. Finds GAMEFILE.DAT's starting LBA once at boot, through the disc's file system.
  2. Reads entry *n* and computes `disc LBA = GAMEFILE.DAT start LBA + start[n]`.
  3. Reads `count[n]` whole sectors into the caller's buffer.
  4. Reports `count[n] × 2048` as the number of bytes loaded.
- Some callers ask only for the size (`count × 2048`) so they can reserve a buffer first. A few take the largest size over a range of resource numbers.
- Sound banks and streamed audio use the same table. They take the resource's LBA and sector count from it and stream straight from the disc.

## Edge cases and error handling

- **Zero-count entries** (76 of them) are placeholders. Their start equals the next entry's start. Treat them as empty and never read bytes for them. Reading at their start would return the *next* resource's data.
- **Cut stinger slots:** the run of 72 empty entries 275–346 are the slots of cut stingers. Stinger clip n is resource 256 + n (SPEC-0009), and records 19 to 90 of the stinger chooser table (classes 19 to 65; SPEC-0024, table 14) would map to resources 275–346. The game does request those stinger indices, but its stinger routine drops every index of 19 or more, so these entries are never loaded. A clean build must play nothing for them (High).
- **Out-of-range numbers:** the game does not bounds-check the lookup it uses for reads. Its size and existence check accepts numbers up to and **including** 1434, which is one past the end. The clean side should reject any number ≥ 1434.
- **GAMEFILE.DAT missing:** the game prints a diagnostic message (text not recorded) and keeps going in a broken state. The clean side should fail disc validation instead.
- The last sector of a resource is loaded in full, even when the payload ends earlier.

## Test vectors

Synthetic, not taken from the disc:

- Index bytes `00000000 03000000 03000000 00000000 03000000 02000000`, read as three LE u32 pairs, give:
  - resource 0 = sectors 0–2 (bytes 0…6143, readable length 6144)
  - resource 1 = empty (count 0; don't read)
  - resource 2 = sectors 3–4 (byte offset 6144, length 4096)
  - The invariants hold, with a total of 5 sectors.
- GAMEFILE.DAT at LBA 1000 plus an entry with start 25 and count 4 gives a disc read of LBAs 1025–1028 (8192 bytes).
- Real-disc sanity check (hand-checkable): resource 0 begins at sector 0 of GAMEFILE.DAT. Its first 4 bytes are ASCII `IDM` followed by byte value 1.

### Test vectors: real index entries (disc)

- Entries are (start sector, sector count) pairs: resource 0 = (0, 58), resource 1 = (58, 58), resource 2 = (116, 58), resource 3 = (174, 74). Resource 3 therefore begins at byte 356,352 and has a readable length of 151,552 bytes.
- Resource 526 = (205,891, 347). Its entry bytes are the decimal values 67, 36, 3, 0, then 91, 1, 0, 0. Byte offset in GAMEFILE.DAT = 205,891 x 2048 = 421,664,768; readable length = 347 x 2048 = 710,656 bytes. With the data file starting at LBA 885,373, the disc read starts at LBA 1,091,264.
- Empty run: resource 274 = (195,985, 17) and resource 275 = (196,002, 0). Every entry from 275 to 346 has start 196,002 and count 0; resource 347 = (196,002, 113). Reading "resource 275" must return nothing, not the first sector of 347.
- Last entry: resource 1433 = (927,938, 961), and 927,938 + 961 = 928,899, the sector count of the data file. 76 entries in all have count 0.

## Open questions

- Answered: resource numbers map to world-pack slots and character-region runs in SPEC-0006, model and character asset resources in SPEC-0004, and music banks and streams in SPEC-0009 (bank and sound-block formats in SPEC-0031). This spec covers the archive index itself.
- Resolved (boot route) / default (other routes): the boot effect-group loader skips item-stream index 2, which would map to empty resource 8; in that route it reads resources 7 and 9. This does not rule out reads through other callers. No request for empty resources 4, 5, 6 or 8 is known, but a data-driven number cannot be excluded from the program alone. Default if unresolved (rewrite choice): a read of any empty entry returns no bytes and is not an error for the caller (needs a runtime check). Answered: the run 275–346 holds the slots of cut stingers, which are never loaded (Edge cases; SPEC-0024, table 14).
