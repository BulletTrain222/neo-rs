# SPEC-0008: Platform services, movie files and library behavior

- Answers: proactive (none requested; the "Replaced" category)
- Status: Draft
- Disc build: SCUS-97367 v1.00 (same fingerprint as SPEC-0001)
- Confidence: hash-based range draw: High for the formula and the four test vectors (recomputed); Medium-High that it is not reached in play. C runtime rules: reverse call walk and copy High; merge, both compaction passes, the sorted interval index and their owner (the C++ exception unwinder, never run in play) High; the occluder comparator High for its rule. Disc file inventory and movie headers: High. Library and runtime code categories: Medium. Random numbers: the game's generator, its state object, its seeding rule, the clock seed and the three draw rules High; the first draw being made at start-up Medium-High; draw counts Medium; the C-library generator (never used in play) High. Memory helpers: Medium-High, except the compare's stop-at-shared-zero rule, Medium-High (not run). Platform arithmetic (single-precision results truncated toward zero, float-to-integer truncation, seconds-to-ticks conversion, accumulating-timer rule): High; one-guard-bit add alignment and divide to nearest: Medium-High. Software float multiply: Medium. Cooperative task, keyed table and byte queue helpers: Low-Medium. Type registry walk: Low-Medium for the static caller and its zero-count behavior; indirect or external entry remains open. Table-driven bit readers of the movie video plane decoder: Medium for their role, Low-Medium for their field meanings. Inverse-transform stages: Medium. The C++ runtime note: Medium-High. C++ type-information family and unaligned word read helper (listed as replaced code): Medium; both are library code and the clean side needs neither. Platform services section: sound command size limit (500-byte argument block, one command in a 512-byte packet with a 12-byte header) High, confirmed in the send routine; sound one-pending and wait-for-processor behaviour Medium-Low (inferred, not run); file and disc services (stdio-style operations, disc-read wait) Medium, the wait being Medium-Low; graphics wait before drawing, critical sections, cache range operations and memory primitives Medium; assertion message content High for the existence of the message with file and line, Medium for its exact fields. Movie video plane decoder: Medium-High that it serves movies (its only known caller is the movie frame dispatcher); Medium-High that it is a Bink-style decoder and not a DEFLATE decoder (by structure; its table format was not confirmed against DEFLATE). Whether any non-movie disc data uses it is open. Generic sort with a comparison callback: High for its arguments, swap rule, algorithm family, sign convention and its two callers; High that neither caller's sort runs in play. Texture upload packet builder: specified in SPEC-0003, see there. Formatted output, 64-bit remainder: High for role, Medium for details. Character output and text helpers: High for the newline and rounding rules, Medium for the buffers. Allocator: Medium-High for allocation and free-path bin/coalescing rules; system-break trimming Medium; whether game code checks null remains open. Movie decoder helpers: chroma constants Medium; luma factor Medium-High for the main movie path (1.164) and Medium-High for the edge and 16-bit paths (0.982); 8x8 inverse transform Medium; block reconstruction Medium (its caller is the movie plane decoder, Medium-High); transform pass Low-Medium (movie audio chain by reachability); stream step Medium-Low; sample accumulator Medium (movie audio chain by reachability). Movie audio internals (overlap blend, track gains, sound buffer size, sound-track records, mixer and stream pump conversions; listed as replaced code): High. Movie container and header: High for the magic, the counts and the sizes; Medium-High for the frame-time unit (runtime conversion confirms the ratio produces the frame period in milliseconds) and Medium for the flag meaning. File-read queue: Medium-Low for the device detail; Medium-High for the request order and the pump rules. Debug overlays: High for the empty variable list (no registration path is known); Medium for the tuner, watch screen and overlay text; Medium-High for the open gate (sequence flag plus L3 and R3); Medium for the freeze and the write to address zero (inferred, not run). Developer and test leftovers (screenshot writer, PC-link command dispatcher, random-number statistics, vector-unit self-test): High for the list of what each routine does, Medium for the detail of each, and reachability open (see Developer and test leftovers). Display mode, tick rate and UI canvas (NTSC only, the start-up rate words, the fixed 512 x 448 canvas): High; the meaning of the other video-mode record values: Low. Memory card layer: High for the 24-code event set, its names and numbering, and the folder name; Medium-High for the request kinds and the request validator; Medium for the file sizes and the needed-space figure.

## Summary

About 15 % of the boot program's code (about 1,030 functions, about 275 KB) is PS2 platform plumbing that the rewrite replaces with its own engine, not recreates. Examples: the Bink movie player, the system kernel and C library, the C++ runtime type-information support, the sound driver interface, the disc driver, the controller driver and the graphics hardware packet builders. This spec says which disc files belong to that plumbing, what the clean side must provide instead, and what the original did that players can see, so the replacement behaves the same. It also records the platform services the game relies on (sound commands, file and disc access, graphics waits, critical sections, cache and memory primitives, assertion messages), the random-number rules and the small library behaviors the game relies on. It also covers the movie video plane decoder (a per-frame dispatcher for the luma and chroma planes, a block decoder, table-driven bit readers, and run and direct-value fills; it is a Bink-style decoder, not a DEFLATE decoder, and it runs only while a movie plays, never for sound), a generic in-place sort that takes a comparison callback, and one provisional graphics item. It also states the formatted-output routine and its number-to-text rules, the 64-bit remainder, the allocator's allocation path, the movie decoder helpers (colour conversion, an 8x8 inverse transform and block add, a transform pass, a stream step and a sample accumulator), the file-read queue, and the debug overlays (the variable tuner, the watch screen and the debug text routines). Once the clean side implements this spec, those functions count as done.

## Where it lives on the disc

| ISO path | What it is | Clean side |
| --- | --- | --- |
| `SYSTEM.CNF` | Boot record: boot program name, version 1.00, video mode NTSC | Read only for fingerprinting |
| `SCUS_973.67` | Boot program | Data tables only (other specs) |
| `GAMEFILE.DAT` | All game resources | SPEC-0001 onward |
| `IRX/*.IRX`, `IRX/IOPRP300.IMG` | I/O-processor drivers (sound, memory card, controller, USB, network) | Ignore |
| `CDPAD.BIN` | 10 MiB filler to position data on the disc | Ignore |
| `MOVIES/<group>/*.BIK` | Full-motion video | Play (below) |

Movie groups, by folder: `FRONTEND` (2 files: title and attract loop), `ACTINTRO` (4), `ACT1` (17), `ACT2` (11), `ACT3` (24), `ACT4` (17). Group sizes range from 4 MB (ACTINTRO) to 626 MB (ACT3). Together the six folders hold 75 movie files, 1,808,927,016 bytes in all (counts are in Test vectors).

Boot-program data words used by this spec (virtual addresses inside the boot program's data, read as 32-bit little-endian words):

| Address | Size | Meaning | Value on disc |
| --- | --- | --- | --- |
| 0x2F96A8 | 4 | State word of the C-library random generator (see Random numbers) | 1 |

Nothing on the disc holds the state of the game's own generator; it is the run-time object at 0x39BE50 (5,008 bytes, index word at 0x39D1D0, value 625 at boot meaning "not seeded"), filled at the first draw (see Random numbers). Its constant matrix table {0, 0x9908B0DF} (two 64-bit words) is the 16 bytes at 0x2EC9B8.

The movie video plane decoder, its bit readers and the sort described under Library behavior are program code, not disc data, so they have no disc address. Whether any non-movie disc resource is decoded by the movie video plane decoder is an open question (see Open questions).

GAMEFILE.DAT resources 144 and 145 are sound data, not movie data: each begins with a VAG sound header, and the movie decoder does not read them. They are the emote clip banks; their layout and player are in SPEC-0009 (Emote voice banks).

## Layout

### Movie files

Every `.BIK` file is a Bink video file, revision "BIKi" (first 4 bytes). It is not an MPEG program stream: no movie starts with an MPEG pack start code (High). The movie open check accepts Bink revisions f, g, h and i (High); every disc movie is revision i. Little-endian header (all fields read from the first 64 bytes of all 75 files):

| Offset | Size | Meaning | Values on disc |
| --- | --- | --- | --- |
| 0x00 | 4 | Magic `BIKi` | Same on every file |
| 0x04 | 4 | File size minus 8 | Equal to the file size minus 8 on every file |
| 0x08 | 4 | Frame count | 190 to 3,912 |
| 0x0C | 4 | Largest frame size in bytes | 40,996 to 80,504 |
| 0x10 | 4 | Frame count again | Equal to the value at 0x08 |
| 0x14 | 4 | Width | 512 |
| 0x18 | 4 | Height | 448 |
| 0x1C | 4 | Frame-time numerator, in 100-nanosecond units per second (Medium for the unit) | 10,000,000 |
| 0x20 | 4 | Frame-time denominator: the frame period in the same units (Medium) | 333,333 (30 fps, about 10,000,000 / 333,333) on 70 files; 333,667 (29.97 fps) on 5 files |
| 0x24 | 4 | Flags: no alpha plane and no grayscale plane when clear (Medium, from the decoder's flag tests) | 0 on every file |
| 0x28 | 4 | Audio track count | 1 |
| 0x2C | 4 | Largest decoded audio size of the track (unit not confirmed) | 69,120 (69 files); 138,240 (4 ACTINTRO files); 145,920 (ATTRACT and ACT2 210A) |
| 0x30 | 4 | Audio: low 16 bits are the sample rate; high 16 bits are flags | 0x1000 with 44,100 Hz on 69 files; 0x7000 with 44,100 Hz on 4 ACTINTRO files; 0x7000 with 48,000 Hz on ATTRACT and ACT2 210A |

The high flag bits at 0x30 are not mapped to a channel layout, so mono or stereo is not established (Low).

The clean side needs a Bink (version 1, revision i) video and audio decoder. Independent open-source decoders exist. Movies are 512 × 448 pixels and were shown stretched to the full 4:3 TV frame, so display them at 4:3 aspect, not square pixels.

When each movie plays (title, attract loop after idling on the title, act intros, story scenes) belongs to the game-flow spec.

### Random-generator state

The C-library generator keeps one 32-bit state word. Its initial value is the word at 0x2F96A8 on disc (value 1), and it is updated only by the generator itself (see Random numbers). It is never used in play.

The game's own generator state (run-time object at 0x39BE50, not on disc):

| Offset | Size | Meaning |
| --- | --- | --- |
| 0x0000 | 624 × 8 | State words: each slot holds one 32-bit word in its low half, upper half zero |
| 0x1380 | 4 | Index of the next word, 0 to 624; 625 means "not seeded" |
| 0x1388 | 4 | Copy of the seed |

## Behavior (what the replaced plumbing did, as players see it)

| Replaced area | Original behavior to keep | Clean-side provider |
| --- | --- | --- |
| Video timing | NTSC only: 60 fields per second and a 512 x 448 display. The PAL branch (50 Hz, 512 x 512) is unreachable in this build (Display mode, tick rate and UI canvas); game logic per SPEC-0007 | Bevy fixed timestep |
| Graphics hardware (GS, vector units, DMA) | Texture decoding (SPEC-0003), meshes and strips (SPEC-0004), skinning (SPEC-0005); frame waits (Platform services) | GPU renderer |
| Disc driver | Reads by resource number (SPEC-0001); streams audio and movies; disc-read waits (Platform services) | ISO reader in `neopets-data` |
| Movie player (Bink) | Full-screen 4:3 playback with its own audio track; skippable is a game-flow decision | Bink decoder |
| Sound driver interface | Clips, streams (SPEC-0009), banks (later spec); the command channel rules (Platform services) | Bevy audio |
| Controller driver | One DualShock 2: 16 digital buttons and two analog sticks, each axis 0–255 with 128 at rest; vibration on or off (an option in the Controls menu) | Bevy input, gamepad or keyboard |
| Memory card driver and card layer | One save file holding all four save slots, and the card messages the player answers (SPEC-0025, Card jobs as the player sees them); the card layer's request kinds and event codes (Memory card layer) | A file in the user's data folder |
| Kernel, threads, C library, math library, C++ type-information support | Number-to-text where the game shows it (Library behavior); the memory, remainder and random-number rules; the platform services section | Rust standard library, plus the rules below |
| USB and network drivers | Loaded at boot; no game feature uses them | Not needed |
| Movie video plane decoder (per-frame plane dispatcher, 8 by 8 block decoder, table-driven bit readers, run and direct-value fills) | Decodes the luma and chroma planes of every movie frame in 8 by 8 blocks (see Movie video plane decoder). Runs only while a movie plays. Use by any non-movie disc data is open | The Bink-compatible video decoder (the movie player's own decoder) |
| In-place sort with a comparison callback | Orders records by a key; no player-visible behavior of its own. Its result is visible only through the caller that uses it | Any sort over the same records with the same key order (see Edge cases on ties) |
| Texture upload packet builder | Places texture data into graphics memory before drawing. Specified in SPEC-0003 (Runtime upload packet builder); no separate item here | GPU renderer (SPEC-0046) |
| Movie decoder helpers (chroma and luma conversion, 8x8 inverse transform, block add, transform pass, stream step, sample accumulator) | Colour and sample scaling for movies and sound (see Movie decoder helpers) | Open Bink-compatible decoder; the sound path (SPEC-0009, SPEC-0031) |
| Debug overlays (variable tuner, watch screen, debug text routines) | Debug aid, not part of play. The watch screen can open after the SPEC-0033 sequence, but shows no variables in this build and certain buttons hang the game. Do not implement the screen or tuner as a feature (see Debug overlays) | None. An optional read-only debug view only |
| Developer and test leftovers (screenshot writer, PC-link command dispatcher, random-number statistics dump, vector-unit self-test) | Not part of play. Each routine is replaced and no play path is known that calls it (see Developer and test leftovers) | None needed for play. A clean-side screenshot may be added as a normal feature |

## Platform services the game relies on

The clean rewrite replaces every service in this section with its own platform layer. It does not need to recreate the original's structure, names or calls. It needs to preserve only the observable behaviors described here: the order in which work finishes, the one-command-at-a-time rule where it changes timing, and the file read semantics. Each subsection gives its confidence.

### Sound driver command channel

- The game sends sound commands to the sound driver, which runs on the input-output processor, through one command channel. Each command is a fixed command kind plus a packed argument block.
- **Size limit (High).** The argument block of one command is at most 500 bytes. The channel's packet holds the 500 bytes plus a 12-byte header, so a block of 501 bytes or more is refused. In the original, an oversized command is a fatal error: it reports an assertion message and then stops, and it does not recover. The clean side should return an error to the caller for an oversized command, and must not send a truncated one.
- **Buffered command submission (High for the recovered send path).** The nonblocking sender appends each command to the active one of two 4-KiB staging buffers, preserving send order. It waits only when that buffer is unavailable or cannot fit the next aligned command packet; while waiting it pumps the sound service and flushes the cache. When the transport is idle, the service submits the current nonempty batch asynchronously and switches to the other staging buffer, so another command may be queued while the earlier batch is in flight. This is a bounded batch queue, not a rule that every command must wait for the prior command to finish. Exact audible timing and transport-error behavior remain unmeasured. The clean side should preserve order and defer a send only when the active staging buffer cannot accept it.
- The game uses about 70 fixed command kinds, each through its own small wrapper that packs the arguments and sends one command. The list of kinds is not written here. The clean side needs only the observable effect of each command (what plays, stops or changes), which the sound specs (SPEC-0009, SPEC-0031) cover, and not the channel itself.

### File and disc services (Medium)

- File access goes through a remote-call layer to the input-output processor. It offers a stdio-style set of operations: open, close, read, write, seek and control. Each operation kind has its own lock, so one operation of a kind runs at a time.
- **Disc-read wait (Medium-Low, inferred).** A disc-read command does not return until the read has completed. The caller blocks for that time. This is inferred from the operation's names and its wait phase; it has not been observed by running the original.
- **Read semantics (Medium).** A read returns the bytes the disc holds at the requested position and length, and the caller receives them only after the read has finished. Resource reads through SPEC-0001 follow this rule. How the original reports a failed read is an open question.

### Asynchronous read queue (Medium-Low)

- Reads can be queued. Each queued request moves through three states: started, done and complete. While a request is started, each step copies at most 4,096 bytes from the device into the request's destination. When the last byte is copied the request is done, and when it is complete its completion callback runs once.
- **Order (Medium-High).** The queue is first in, first out. Only the request at the head advances, by one state per pump call: start the read; poll until the device reports the read done; then finish (expand the data when the request's expand flag is set, run the completion callback once, and retire the request). An asynchronous request therefore takes at least three pump calls, and completion callbacks run in the order the requests were queued. A later request never completes before an earlier one.
- **Who pumps (Medium-High).** The main loop makes one pump call per pass, and only when no movie is queued and none of the sound side's 25 stream slots is in its reading state. A pass in which music or a voice stream is reading from the disc does not advance the queue, so disc loads wait for streamed sound. A blocking request (one not queued as asynchronous) pumps the whole queue until it is empty, with one display field sync before each pump call, so it also completes every earlier request first. During that drain, a request that has not started yet is read at once with a blocking read and finishes on the next pump call (two calls instead of three); a request already polling keeps polling. Before a movie plays, the movie queue player waits for sound to go idle and then pumps until no transition is busy, no world is loading and the queue is empty (SPEC-0026, Playback). What the global flag that selects the device read function chooses between is not confirmed (see Open questions).
- **Expand flag (High).** No load in this build sets the expand flag: every place that queues a read passes 0 for it. The expand step exists but never runs on disc data (SPEC-0002).
- Game dependence: a resource is used only after its completion callback has run, and the bytes delivered are the same as a blocking read of the same range (see Read semantics above).
- Clean side: an asynchronous reader over the ISO reader in `neopets-data`, delivering each request's bytes and then calling its callback once, in queue order. The standard library's channels or futures give this. Block size is free to choose, but steps of up to 4,096 bytes give the same result. Where a load's completion is visible in play (an area appearing, a character becoming usable), keep the rule that it completes after every load queued before it and takes at least three pump calls (so an asynchronous load queued before the pump step of a pass completes no earlier than two passes later).

### Memory card layer (replaced; Medium-High unless marked)

The clean side replaces the whole card layer with its own save file (SPEC-0025). The player-visible card behaviour, the message records and the clean-side mapping are in SPEC-0025 (Card message records, Card jobs as the player sees them). The facts below are what that layer did.

- **Jobs and requests.** The game's card jobs run as global tasks (mask 3) while a front-end screen waits on them. Each job builds a request, a list of steps, and hands it to the card layer. The lists the game builds are: read; write; create folder then write; format; format then create folder then write; query; and card check. A request started with its asynchronous flag 0 runs to its end inside the start call, without yielding.
- **Step kinds.** The card layer runs each step by kind: 0 card query (counts the save folder's entries; when the folder is missing and the free space is below the need it raises code 21, Medium), 1 create folder, 2 read file (stores the number of bytes read), 3 and 4 write file (the save file; an icon file), 5 delete, 6 not implemented (raises code 23), 7 format, 8 new-card check (whether a card is present and whether it is the same card as before).
- **Validator.** Before a request runs, a validator checks how it was built and raises codes 0 to 11 for building errors (a missing card, folder or file field, bad names, a folder used before it is created, a wrong icon count, a wrong token name, a format that is not first or not followed by a create-folder step, a new-card check that is not alone, and a synchronous-mode error). It has no player-visible effect.
- **Event set (High).** The layer keeps one 24-bit event set: each code 0 to 23 is one bit. A raise sets the bit and a test reads it; the values -1 and 24 are ignored. A request that fails without raising any code raises 22. Codes 0 to 11 are validation errors and 12 to 23 run-time results: 12 no card, 13 not the same card, 14 unformatted, 15 already formatted, 16 no file found, 17 path not found, 18 file corrupted, 19 create folder: already exists, 20 save folder does not exist, 21 insufficient space, 22 general failure, 23 not yet implemented. Their names are developer text in a 24-entry pointer table at data address 0x2ECCF8 and are never shown. The game's jobs test only "any code", 21 and 22.
- **Names and sizes.** The save folder is `BASCUS-97367-0`, the product code joined directly to the suffix `-0` (High). The layer's file sizes are 0xC400 bytes for the save file, 100 bytes for a support file and 0xC2C2 bytes for the icon. From these it computes the free space the game needs, in KB, which the card texts show (Medium; 106 KB by the size sum).
- **Clean side.** None of this is reproduced. The clean side writes its own file (SPEC-0025) and needs no requests, steps, validator or event set.

### Graphics wait before drawing (Medium)

- Before the game hands a frame to the hardware, it waits until the previous graphics work has finished on each path: the two graphics DMA channels, the vector interface, and the vector units. If a path does not finish, the original reports an error message naming that path.
- Clean-side rule: a frame's drawing must not start until the previous frame's submitted work has completed. The GPU renderer (SPEC-0046) gives this ordering, so no extra wait is needed beyond that ordering.

### Display mode, tick rate and UI canvas (High)

- **NTSC only.** At start-up the display set-up fills a video-mode record: width 512 (stored twice) and height 448 (stored twice), together with other display values (Open questions). A PAL flag would make the height 512, but the flag is set only when the region detector reports PAL. In this build the region detector always returns 0 (NTSC), and the start-up configuration's override word is -1 (no override), so the PAL branch is unreachable and the 512 x 512 mode never occurs.
- **Rate words.** The same switch picks a row of the rate table at data address 0x30C598, which holds two rows of two floats: (60.0, 30.0) for NTSC and (50.0, 25.0) for PAL. Row 0 is always picked. At start-up the game writes, once: 0x368FA0 = 60.0 (ticks per second), 0x368FA4 = 0.5 (30 / 60) and 0x368FA8 = 1/60 (float32 0.016666668, divided to nearest). The 50 Hz row is never used. The tick rules that read these words are in SPEC-0007 and under Seconds to ticks.
- **UI canvas.** The canvas words (vertical scale 1.0 at 0x2EC900, height 448 at 0x2EC904, width 512 at 0x2EC908, and a further 1.0 at 0x2EC8FC) have exactly one writer, a setter that is never called. So the HUD and every front-end page always use a 512 x 448 canvas with scale 1.0 (canvas mapping in SPEC-0023).
- **Clean side.** Provide the NTSC 512 x 448 mode only, at 60 ticks per second. No PAL or 50 Hz mode is needed, and the canvas is fixed.

### Critical sections (Medium)

- Shared lists used by the system and the game, and the timer counter, are changed inside sections where interrupts are disabled, then re-enabled. Readers of the timer counter see a consistent value.
- Clean-side rule: any list or counter shared between a timer callback or interrupt-like context and the main loop needs mutual exclusion, and the timer counter must not show a torn value. The Rust standard library's synchronization types provide this. The exact ownership of the two lists (system or game queue) is an open question.

### Cache range operations (Medium)

- The original writes back or invalidates cache lines over address ranges aligned to 64 bytes. The clean side has no cache to manage and needs no equivalent. It must ensure that data the game wrote is the data later read, which normal memory gives it.

### Memory and byte-scan primitives (Medium)

- **Fill, move and copy:** the same rules as in Library behavior below.
- **Byte scan:** a primitive that scans a byte range for a given byte value, or for the end of a zero-terminated string, and returns the position where it stopped. Its shape is only partly described; the clean side should return the first match, and the end of the range or string when there is none.

### Assertion messages (High for the message, Medium for its fields)

- A failed assertion formats a message that names the source file and the line number, then stops. The clean side reports the same information (file and line) through its own error or log path, and the exact text of the original's messages is not reproduced here.

### Kernel service calls (replaced; Medium-High)

- The boot program reaches the console kernel through about 85 to 89 small wrapper routines. Each wrapper is a few instructions that place a service number and issue one kernel call. The count is 85 to 89, so treat it as approximate (Medium). The service numbers are not part of this spec; the clean side needs only the services themselves.
- Services the game uses through these wrappers, by function:
  - threads and semaphores, including the variants that may be called from an interrupt context (wait, signal and wake);
  - registering interrupt and DMA-completion handlers;
  - the vertical-sync flag and the graphics interrupt mask;
  - the I/O-processor transfer and register services used by the sound and file paths;
  - the debug channel call (text to the host), the memory-size query, and the cache flush.
- Clean side: none of these wrappers is reproduced. The platform layer supplies each service through its own mechanism. The observable rules that depend on these services are given where they apply (the graphics wait, the critical sections, the disc-read wait and the assertion messages). The memory-size query is used by the boot program to check for a 32 MiB console (High for that check).
- Service numbers and the wrapper layout are not part of the spec and are not needed by the clean side.

### Platform arithmetic (reproduce; High)

- The console's main-processor floating-point unit works in single precision, and the result of every multiply and every add (subtract included) is truncated toward zero to the nearest representable single-precision value, not rounded to nearest. The conversion of a float to an integer also truncates toward zero.
- **Add alignment, one guard bit (Medium-High).** Before an add or subtract, the operand with the smaller exponent loses precision while it is aligned to the larger one: only one bit below the larger operand's last mantissa bit is kept. In terms of the stored bits, when the two exponents differ by d (1 to 24), the low d − 1 bits of the smaller operand's 23-bit mantissa field are cleared; when they differ by 25 or more, the smaller operand counts as zero. The exact sum of the two aligned values is then truncated toward zero. When the exponents are equal, nothing is cleared. Example: 1.0 + (−1.5 × 2^-24) gives 0.99999994, because the smaller operand is first cut to −2^-24; truncating the exact sum would give 0.99999988. The rule matters for long runs of adds of a small step to a larger value (fades, bars, timers).
- **Divide (Medium-High).** A single-precision divide rounds to nearest. The tick length 1 ÷ 60 is therefore 0.016666668, and the fade and bar steps (target ÷ frames, SPEC-0052) are divided to nearest.
- **Square root (Medium-High).** The single-precision square-root operation rounds to nearest, like divide, rather than truncating toward zero like multiply and add. Synthetic vector: sqrt(2.0) = 1.4142135. The rule is operation-specific; reciprocal square root uses separate arithmetic and is not covered by this statement.
- Constants are stored as the nearest single-precision values (for example 0.01 is stored as 0.0099999998).
- A clean build must reproduce this rule wherever game rules multiply or add floats and the result is compared, converted to an integer or shown, because it gives different results from round-to-nearest arithmetic in common cases:
  - 25 x 0.01 gives 0.24999999 (not 0.25), so 1 + 25 x 0.01 gives 1.2499999; then 4 x 1.2499999 gives 4.9999995, which converts to the integer 4 (round-to-nearest arithmetic gives 5.0 and 5).
  - 100 x 0.01 gives 0.99999994 (round-to-nearest gives 1.0).
- Users of the rule: hero and enemy damage, damage scales and Mote charge amounts (SPEC-0036, Damage and its Platform arithmetic paragraph; SPEC-0021, Mote charge), and the HUD bar fill and frame widths, which take trunc(value x 0.6666667) (SPEC-0023, section 3); every script time in seconds and every accumulating timer (Seconds to ticks below); the screen-effect fades and bars (SPEC-0052).
- Clean side: run each such operation in single precision and truncate the result toward zero, for example by computing the exact product or sum in double precision and truncating it to single precision toward zero; or use a floating-point environment set to round toward zero for these rules. For adds, clear the smaller operand's low bits first as the alignment rule above says; a round-toward-zero environment alone does not do this.

#### Seconds to ticks (High)

- **Script calls.** A script call that takes a time in seconds s (Wait, FadeScreenOut, FadeScreenIn, LetterBoxOn, LetterBoxOff, OpenLetterBox, CloseLetterBox, LoopEntityAnim, LoopHeroAnim) forms n = trunc(f32(s) × R), where R is the frame-rate word at 0x368FA0 (60.0, SPEC-0007). The product is one single-precision multiply, truncated toward zero, and the conversion to an integer truncates again. No step uses double precision or rounding. The engine's own second constants are converted the same way.
- **Uses of n.** Wait yields n ticks (none when n <= 0). Fades and bars use max(1, n) frames, so at least 1 frame. The loop-animation calls treat a tick count of 0 as loop forever (SPEC-0030).
- **Fade and bar steps.** The per-update step is the target change divided by the frame count, ±target ÷ frames, divided to nearest (Divide above). The fade alpha and the bar progress then gain the step once per update through the truncating, aligned add (SPEC-0052).
- **Which values lose a tick.** A decimal s for which 60s is a whole number N gives N − 1 exactly when its stored single-precision value lies below the decimal (f32(s) < s); otherwise it gives N. Up to 3 s the values that give 60s − 1 are 0.35, 0.45, 0.65, 0.7, 0.9, 0.95, 1.05, 1.15, 1.3, 1.4, 1.55, 1.65, 1.8, 1.9, 2.05, 2.1, 2.3, 2.35, 2.55, 2.6, 2.8 and 2.85. Up to 60 s, 478 of the 1,200 multiples of 0.05 s do so. Values whose stored value lies above the decimal (0.1, 0.2, 0.3, 0.4, 0.6, 0.8, 1.1, 1.2) and values stored exactly (0.25, 0.5, whole numbers) give 60s.
- **Accumulating timers.** A timer t that starts at 0 and gains the tick length dt = 0.016666668 each tick through the truncating, aligned add stays a little below n/60 after n ticks. For every T that is a multiple of 0.05 s up to 120 s, compared as its stored single-precision value, both t >= T and t > T first hold on tick 60T + 1. The one exception is t >= 0.05, which first holds on tick 3 (t > 0.05 holds on tick 4). Round-to-nearest arithmetic gives 60T for many of these values (for example 0.3, 0.4, 0.5, 0.6, 5, 6, 8, 10 and 11.2 s), so a clean build that rounds to nearest, or counts exact ticks, acts one tick early.

## Library behavior the game relies on

### Random numbers: the game's generator (used by all play, High)

All random draws of play, in the boot program and in level scripts, gameplay and effects alike, come from one Mersenne-Twister generator (MT19937). Its state is one run-time object at data address 0x39BE50 (outside the file image, 5,008 bytes): 624 state words, each held in an 8-byte slot whose upper half is zero; a 32-bit index at +0x1380 (address 0x39D1D0); and a copy of the seed at +0x1388. Every call site passes this same object; there is no second state block and no call site takes the state from a function result (High). The generator's two-entry matrix table (0 and 0x9908B0DF, each a 64-bit word) is the 16 bytes at data address 0x2EC9B8; it is constant and is not the state. Nothing outside the generator reads or writes the state, so it is never saved, restored or re-seeded (not per fight, level, load or new game) (High).

- **Generation:** standard MT19937 (N = 624, M = 397, upper mask 0x80000000, lower mask 0x7FFFFFFF, matrix 0x9908B0DF). When the index is 624 or more, all 624 words are regenerated in place (the standard twist), then the index is 0. Each output takes word[index], increments the index and tempers it, in 32-bit arithmetic: y ^= y >> 11; y ^= (y << 7) AND 0x9D2C5680; y ^= (y << 15) AND 0xEFC60000; y ^= y >> 18.
- **Seeding (the original 1998 rule, not the 1812433253 rule):** word 0 = seed mod 2^32; word i = 69069 × word (i − 1) mod 2^32 for i = 1 to 623; the seed copy is stored; index = 624, so the first output after seeding regenerates first. With seed 4357 this gives the published reference output of the 1998 generator (see Test vectors), which checks an implementation.
- **Start-up:** a start-up initialiser sets the index to 625, meaning "not seeded". The first draw of the run sees 625 and seeds from the console clock before generating. A routine that seeds from a given value exists, but nothing calls it, so the game is not deterministic from power-on.
- **Seed value:** the console clock record has eight bytes (status, second, minute, hour, unused, day, month, year), each a binary-coded decimal (BCD) byte. The seed uses the raw byte values without converting them to decimal:

  seed = (second + 1) × (minute + 1) × (hour + 1) × (day + 1) × (month + 1) + year, computed in 32 bits.

- **When the seed is taken (Medium-High):** the first draw of every run is made during start-up, while the first hero is set up (its 10 to 12 second ambient cue timer, Range(600, 720) ticks, SPEC-0040), before the title screen. So the seed is the clock at boot, and the first draw any gameplay rule sees is the second output of the stream.
- Because the clock has one-second resolution, two boots in the same clock second give the same stream.

**Draw rules (the only three users of the generator's core; High).** Each costs exactly one output, except where it says "no draw". The remainder is taken on the output read as an unsigned 32-bit number, so integer results never leave their range.

| Rule | Call | Result | Range | No draw when |
| --- | --- | --- | --- | --- |
| Integer range | Range(lo, hi) | if hi < lo (signed) swap them; lo + (u mod (hi − lo + 1)), u the output as unsigned 32-bit, unsigned remainder | [lo, hi], both ends included | lo = hi (returns lo) |
| Index | Index(n) | u mod n, unsigned remainder (n taken as unsigned 32-bit) | [0, n); Index(1) draws and returns 0 | n = 0 (returns 0) |
| Real range | FRange(a, b) | if b < a swap to a0 <= b0; s = (b0 − a0) × 2^-32 in single precision, computed first; result = float(u) × s + a0, u converted as unsigned | [a0, b0) in practice; b0 only through float rounding (Medium) | a = b (returns a; −0.0 and +0.0 count as equal) |

The level-script host call Random(a, b) is Range(a, b) on the same generator (SPEC-0006, SPEC-0030).

Common call patterns (fixed constants unless the owning spec names a data word):

- Percent roll, 101 values: Range(0, 100), 82 call sites. Each site states whether it passes below or at most the chance.
- Percent roll, 100 values: Range(1, 100), 23 sites (common-enemy notice and trigger rolls, NPC reaction rolls).
- Percent roll, 100 values from 0: Index(100) = 0 to 99. No site uses Range(0, 99); every 0 to 99 roll is Index(100) (item drops: money chance, item band, sub-table band and the item-id fallback; the emote chance of design table 8). Each rule is per site; the two percent families do not conflict.
- Coin: Range(0, 1) (38 sites) or Index(2) (8 sites).
- Tick timer from seconds: Range(trunc(min × 60), trunc(max × 60)), 60 being the run-time time base (SPEC-0007), inclusive in ticks. A second form, trunc(60 × FRange(min, max)), is used by a few bosses (SPEC-0010 cooldowns); the two forms differ and each owning spec states which.
- Symmetric jitter: FRange(−x, x).

**Draws per wrapper (High for the formulas; site counts are for the boot program).**

| Wrapper | Formula | Range | Boot call sites | Specs whose rules use it |
| --- | --- | --- | --- | --- |
| Integer range | lo + (u mod (hi − lo + 1)), swap, lo = hi no draw | [lo, hi] closed | 272, plus host Random (about 440 sites in 47 level modules) | 0006, 0010 to 0016, 0019 to 0021, 0023, 0024, 0029, 0030, 0033, 0034, 0036, 0038, 0040, 0047 and the level-script specs |
| Index | u mod n, n = 0 no draw | [0, n) | 23 | 0012, 0013, 0014, 0018, 0019, 0021, 0024, 0029, 0033, 0036, 0040 |
| Real range | a0 + float(u) × (b0 − a0) × 2^-32, swap, a = b no draw | [a0, b0) | 115 | 0006, 0010, 0011, 0013, 0014, 0016, 0021, 0025, 0029, 0033, 0034, 0036, 0038, 0040, 0046 |
| Core (direct) | MT19937 output | 32-bit | only the three wrappers | 0008 |
| Hash draw | hash formula below | between a and b | 1 (not reached in play) | 0034 |
| C-library generator | linear congruential, below | 31-bit | 1 (not reached in play) | none |

**Draw counts (Medium).** Draws happen when a rule runs, not on a clock: one per percent roll or timer re-arm; two per spawn-point search (first try only); three per charm landing (spin, sign, bounce) plus two for the launch direction of each launched item; one per drop money roll; two per item roll (both drawn before the walk) plus one for the id fallback. Known continuous consumers: villager gesture choice, Range(0, 15) once per tick per conversing villager, and chain follow, Range(4, 8) and Range(5, 9) per part per tick (both SPEC-0040); rendering jitter, three FRange(−0.1, 0.1) per call of its routine (SPEC-0046); particle emitters, per particle spawned (SPEC-0034). Because the seed is the boot clock, the original sequence cannot be reproduced and draw parity is not observable; the clean side need not match these counts.

**Clean-side rule.** Use one generator object for every draw, seeded once from the local clock at the first draw and never re-seeded or saved. Implement the three rules above exactly (closed integer range with an unsigned remainder, index below n, real range, each with its no-draw shortcut). Using this exact MT19937 with the 69069 seeding is recommended, with a developer option to force a fixed seed for tests; players cannot see the difference.

### Random numbers: hash-based range draw (not reached in play, Medium-High)

A second random source keeps no state. Its only caller is a periodic effect counter (SPEC-0034). That counter, once per period, takes a new value from its own step count, passes it through this source as the seed, and stores the result. The counter itself has no caller and no table entry, so the hash draw is not part of play as far as the program shows (Medium-High; High for the formula). The caller passes a 32-bit seed x and two signed 8-bit limits a and b.

Steps:

1. y = x XOR (x shifted left by 13), in 32-bit unsigned arithmetic.
2. u = (y × (y × y × 0x3D73 + 0xC0AE5) + 0x5208DD0D) mod 2^32.
3. h = u AND 0x7FFFFFFF.
4. The result is a + (b − a) × (1 − h / 2^31), computed in single precision. The limits are not swapped, so the result lies between a and b in whichever order they are given.

The constants 0x3D73, 0xC0AE5, 0x5208DD0D and 0x7FFFFFFF are code constants. The source does not keep the generator's state, so a caller that wants a sequence must supply a different seed for each draw.

Test vectors (a = 0, b = 10; recomputed from the formula):

| Seed x | y | h | Result (rounded) |
| --- | --- | --- | --- |
| 1 | 8193 | 1316808037 | 3.8681 |
| 2 | 16386 | 758455919 | 6.4682 |
| 74565 | 610894661 | 1756498989 | 1.8207 |
| 3735928559 | 1768972015 | 3389909 | 9.9842 |

This source is not the shared Mersenne-Twister generator described above. The clean side does not need it for play; if the effect counter is ever implemented, it may use this formula with its own step count as the seed.

### Which draws use which source (High)

- **Mersenne-Twister.** Every gameplay and effect draw, in the boot program and in the level modules, goes through the three wrappers above with the one state object at 0x39BE50, the float wrapper included. There is no second state block.
- **Hash source.** Its one caller is the unreached effect counter (above).
- **C-library generator.** Its one user is an unreached float helper (below); no level module uses it.
- **Clean-side rule.** One generator object for every draw.

### C runtime helpers the clean side replaces (Medium unless marked)

The game relies on a few C runtime helpers. The clean side uses its own containers, so these are rules to keep, not code to port.

- **Reverse call walk (High).** A table of argument-less function entries. Slot 0 holds a count. Entries occupy slots 1 to count. When the stored count is -1, the count is found by scanning from slot 1 for the first empty slot (the count is the number of consecutive filled slots from slot 1). The entries are called from the highest counted slot down to slot 1. A count of 0, or an empty table, calls nothing. This walk runs at teardown.
- **Copy of a 368-byte record (High).** The copy moves 0x170 bytes in order, forward. Aligned and unaligned source and destination give the same result. The source and destination must not overlap.
- **Owner of the merge, the compaction and the sorted interval index (High).** All three belong to the C++ exception unwinder. Together with a heap sort they make up the unwinder's sort of its frame-description table: the compaction's first pass moves out the entries that break an ascending run, the heap sort orders them, and the merge puts them back. The sorted interval index is the unwinder's per-module table of code ranges, searched when it looks up the frame record for a code position. They run only when an exception is thrown, and no game code throws one: the throw entry points have no caller. None of them runs in play, and the clean side needs none of them.
- **Merge of two sorted lists (High).** Each element is a pointer to a record whose signed 32-bit key is at offset 8. The second list is merged into the first list, working from the back. First-list elements whose key is greater than the current second-list key move to later positions; the second-list element is then placed in the gap. The result is in ascending key order, and when keys are equal the first list's element comes first. The count becomes the sum of both counts. No capacity check is made, so the first list must already have room.
- **Compaction (High).** Each entry points to a record with a signed 32-bit key at offset 8. The first pass walks the entries in order and keeps a stack of the entries kept so far. For each entry, while the entry on top of the stack has a key strictly greater than the current entry's key, that top entry is appended to the output list, its slot is cleared and it is popped. The current entry is then pushed. Equal keys are never moved out. The second pass removes the cleared (zero) slots and keeps the order of the others; the count becomes the number kept. What remains is an ascending run; the output list holds the moved entries in the order they were popped.
- **Comparator of the occluder volume builder (High for the rule).** Each entry points to a record. If the first record's 32-bit id at offset 4 is -1, the comparator returns 1. Otherwise it returns the difference of the two records' signed 32-bit keys at offset 8, rounded half away from zero (the same value as the integer difference while it stays below 2^24 in size). The second record's id is not tested, so the comparator is not symmetric. In the builder the id is -1 for a candidate that lies wholly off screen and the key is the squared distance from the eye. A -1 entry usually ends after the others, but not always: with two records, (id 0, key 5) followed by (id -1, key 1) compare as 4, so they swap and the -1 entry ends first. The builder skips -1 entries after the sort, so the direction matters only when there are more candidates than its cap. The builder is not reached in this build (see In-place sort).

### Character output and text helpers (High for the rules; Medium for the buffers; low game importance)

The C library also has a linear congruential generator: state = state × 1103515245 + 12345 (mod 2^32), and each call returns the state AND 0x7FFFFFFF. Its state starts at 1 (the word at 0x2F96A8 on disc). One helper is built on it: a float in a range, a0 + (r × 2^-31) × (b − a), with r one generator output. That helper has no caller and no pointer to it anywhere in the program image, no level module imports it, and no code writes the state word except the generator itself, so this generator never runs in play (High). The clean side need not implement it.

### Memory helpers

- **Fill**: sets N bytes to the low byte of the given value and returns the destination.
- **Move**: copies N bytes, correct when the source and destination overlap (the copy runs backwards when needed). Returns the destination.
- **Copy**: copies N bytes forward. Same result as move when the regions do not overlap. Returns the destination.
- **Compare** (High for the difference value; Medium-High for the stop rule): compares N bytes of two regions and returns 0 when N is zero. Otherwise it returns the unsigned difference (first region's byte minus the second's) at the first differing byte. Equal bytes are skipped, except that an equal zero byte ends the compare at once with a result of 0, so bytes after a shared zero are not compared. The word-wide and block-wide paths follow the same rule. Clean side: the difference at the first differing byte is the observable result. The clean side must also stop at a shared zero byte wherever a game call site depends on it (see Open questions).
- **Unaligned 32-bit read** (Medium): a library helper that reads a 32-bit word from a byte address that need not be a multiple of four, and stores the word for the caller. The game's own reads of packed data are ordinary reads in the clean code; the clean side needs no equivalent helper, but any of its reads at a byte offset must work for offsets that are not multiples of four (see Edge cases).

The game relies on the fill, move and copy helpers only as ordinary memory operations. Whether the game ever passes overlapping regions to the copy helper was not confirmed; the clean side should use move semantics for any copy whose regions may overlap.

### Character output and text helpers (High for the rules; Medium for the buffers; low game importance)

- **Newline output.** The character output routine writes a newline as a carriage return followed by a line feed. Any other character is written unchanged (High). The rule matters only for byte-exact text files. Screen text does not need it.
- **Decimal digit rounding.** A routine rounds a string of decimal digits at its last digit. If the next digit is 5 or more, the last kept digit goes up by one, and the carry moves left through nines. If the carry runs past the first digit, the routine reports overflow by returning 0 (High for the rounding; the return value on success is not recorded).
- **Buffered byte writers.** Two byte writers, one for each stream, each fill a 128-byte buffer and flush it through a helper when it is full (Medium). The flush at the end of a stream is not recorded.
- Game dependence: these helpers affect only debug and log text, not play. The clean side needs no equivalent beyond its own standard text and file output.

### Formatted output: the printf engine and number-to-text (High for role, Medium for details)

The C library's formatted-output routine walks a format string. Characters other than the percent sign are copied. A percent sign starts a conversion: optional flags, an optional width and precision (each a number or a star that takes an integer argument), an optional length modifier, and a conversion character.

- Flags: space, plus, minus (left-justify), zero padding and hash (alternate form). The bit values are not needed for the rewrite.
- Length modifiers: one selects 16-bit values and another selects 64-bit values.
- Conversions: c; d and i (signed); u; o (octal; hash adds a leading zero); x and X (hexadecimal; hash adds a 0x prefix only for a nonzero value); p (printed with 0x and lowercase hex); e, f and g (through the number-to-text rules below); s (a null string prints the six characters null in parentheses); percent-percent prints one percent sign. Any other conversion character prints itself.
- For an integer conversion with precision 0, a value of zero prints no digits.
- The routine returns the number of characters it emitted.
- Game dependence: the game depends on this routine only where its text reaches the screen or a file. No on-screen number has yet been shown to come from a call of it, so the set of conversions the game needs is open (see Open questions). Until then, the clean side follows every rule here for each conversion it implements.
- Clean side: Rust's formatting covers decimal, hexadecimal, octal and string output with width, fill and precision. Use it, but match the rules above where they differ (the hash prefix on zero, the parenthesised null text).

Number to text (the e, f and g conversions):

- Infinite and not-a-number values print as special words, with a sign for negative infinity. The spelling and case are not recorded here.
- Otherwise the integer part is produced by repeated division by 10, and the fractional part by repeated multiplication by 10, each step's whole part becoming the next digit. A rounding step of one half is applied; where it is applied in the digit loop is not recorded. Digits are gathered in reverse and then put in order.
- Precision sets the number of fractional digits. For e, an exponent is appended with a sign and at least two digits. For f there is no exponent. For g, trailing zeros are removed, and the decimal point too if nothing follows it.
- Default precision when the caller gives none: not confirmed (six is assumed; see Open questions).
- Clean side: a correctly rounded float formatter from the standard library, with the precision set, gives the same digits in most cases. The original's repeated-multiplication method can differ from correct rounding in the last printed digit for some values (Medium, inferred from the method). Check the screen values once they are known. Trailing-zero removal for g is simple string work.

### 64-bit remainder (High for role, Medium for detail)

- A signed 64-bit remainder whose sign follows the dividend (the truncating remainder). The number-printing loop uses it for each digit and takes the quotient from the same division.
- Division by zero raises the processor trap, and the original stops. No call with a zero divisor is known.
- Clean side: Rust's `%` on signed 64-bit integers has the same sign rule, but it panics on a zero divisor and on the most negative value divided by minus one. Use a checked remainder, which returns no result for both cases, and return an error for a zero divisor. The most-negative case is open (see Open questions).

### Allocator, allocation path (Medium-High; free path not described)

- Requests are rounded up to a multiple of 16 bytes, with a minimum block of 32 bytes. Each block header carries the block size and an in-use flag.
- Free blocks are kept in bins: exact 8-byte steps for blocks under 504 bytes, and power-of-two size ranges above that.
- An allocation takes the smallest free block that fits (best fit), splits off the unused part as a free block, and returns the rest. If no free block fits, it takes space from the top of the heap, extending the heap with a system call when needed. If that also fails, it returns the null address.
- Game dependence: no player-visible behavior of the allocator is known. The only visible difference is a failed allocation, which the original reports as null. Whether game code checks that null return is open.
- Clean side: the Rust global allocator replaces all of this. By default a failed allocation in Rust aborts the process. That differs only if game code checks the null return, which is open.

### Allocator release path (High for the static rules; system-break trimming Medium)

- Releasing a null pointer does nothing. A non-null allocation has an eight-byte header immediately before its returned address; the size is stored with status bits in the low two bits.
- The release joins the block with an adjacent free block on either side. The next block's header records whether the released block is in use; when the previous block is free, a trailing size word locates it. Joined neighbors are removed from their free bins before the merged block is handled.
- If the merged block reaches the heap's top chunk, the top chunk is enlarged. When it reaches the allocator's trim threshold, the allocator attempts to return page-aligned space to the system. If that attempt fails, it refreshes the top chunk from the current system break when a remainder larger than 16 bytes is available.
- Otherwise, the merged block is inserted into its size bin in ascending size order; entries of equal size already in the bin remain ahead of the new entry. The allocator serializes release with the same lock used for allocation.
- No invalid-pointer or double-release behavior is specified. Whether game code checks a failed allocation remains open.
- Clean side: a normal allocator can replace this mechanism. Allocation failure policy remains a separate compatibility question.

### Per-thread library state

The C library keeps a per-thread block with its error number and the C-library generator state. The clean side has no equivalent and needs none; errno-style errors become Rust results.

### C++ runtime type-information support (Medium)

The boot program builds type-information records for the standard runtime families (fundamental, class, function, member pointer, array, exception) on first use. The family is about nine small routines. They match a type against a tree of base-class records, delegate to a helper or return a fixed null result, and destroy type records (clearing the record and releasing its storage when the release flag is set). This is library code and not game logic. The clean side has no equivalent and needs none.

### Movie video plane decoder (Bink-style) (Medium-High)

The boot program's movie video decoder decodes the picture of each movie frame. It is a Bink-style video decoder, not a DEFLATE decoder, and it is not a game rule. It runs only while a movie plays, and it is not used for sound. Its input is the frame data of the movie file in the MOVIES folder, not a GAMEFILE.DAT resource.

- **Dispatcher.** A per-frame dispatcher decodes the luma plane at full size, then the two chroma planes at half size, choosing for each plane whether a reference step and a chroma pass apply from the frame's flag words.
- **Block decoder.** For each 8 by 8 block the decoder reads a block-type value, then runs one of several paths: a coefficient bundle decoded with the table-driven bit readers below, a copy of a block from a reference plane using offsets from the stream, or a run or direct-value fill. Only the first six block types are described in detail (Medium for the role of types 0 to 5; Low for the rest).
- **Run and direct-value fills.** Runs fill a number of bytes with one value through the byte-fill helper. Direct-value paths write table values one per byte.

- **Role.** It decodes one plane of a movie frame at a time: the luma plane at full size, then the two chroma planes at half size. It works in 8 by 8 blocks. Each block is read from the frame's coded data with a bit-reader family and table helpers (not all read here), then passed through the inverse transform and the block add (see Movie decoder helpers).
- **Caller.** The movie frame dispatcher, which the movie stream step calls once per frame. No other caller is known (Medium-High). So movie data is the only data known to use this path.
- **Format.** Its plane split, 8 by 8 transform and block add match a Bink video block decoder. Its table helpers were not confirmed against any DEFLATE table format, so DEFLATE is excluded by structure, not by a direct check (Medium-High).
- **Size.** About 6 KB of code. A block-level helper that it calls about eight times per plane is not described.

Clean-side rule: the Bink-compatible video decoder covers this for movies, and a DEFLATE decoder is not needed for movies. A standard DEFLATE decoder is needed only if a non-movie disc stream is shown to use a DEFLATE-style format. Do not reproduce the original's routine.

### In-place sort with a comparison callback (High)

- A generic sort takes the base of an array, a record count, the size of one record in bytes, and a comparison function. It orders the records in place. It compares records by calling the comparison function, and it moves whole records by swapping them. It does not copy the array elsewhere.
- Algorithm family (High): the BSD-style quicksort, with a median-of-three pivot, and insertion sort for fewer than 7 records.
- Sign convention (High): a positive result means the first record belongs after the second. The insertion sort swaps neighbours while the comparison of the earlier with the later is positive, so the result is ascending under the comparison.
- Callers (High): exactly two.
  - The character attachment update (SPEC-0040) builds one record per bone or part: its squared distance from a reference point and its index. Its comparison is the second distance minus the first, truncated to an integer, so the order is farthest first and two distances less than 1.0 apart compare as equal. The sort runs only when a build switch word at data address 0x2DB418 is nonzero. That word is 0 on the disc and nothing writes it, so this sort never runs and the list stays in bone index order.
  - The occluder volume builder (SPEC-0046) sorts up to its candidate count with the comparator in C runtime helpers, then turns the first entries, up to a cap of 14 (the word at data address 0x2EC884; at most 32), into four-plane volumes, skipping off-screen (-1) entries, which still use up a place under the cap. Nothing calls or references the builder, so it does not run in this build and the volume list stays empty.
- The game takes no order from this sort in play. The "contact records" once named as its user were the attachment distance records above.
- Ties: this sort family does not keep records with equal keys in their original order. Neither caller's tie order is visible in play.

Clean-side rule: any sort that gives the same key order over the same records meets this section. The sort is a library helper, so the clean side may use the standard library's sort with a key comparison.

### Software single-precision multiply (replaced; Medium)

- A software multiply of two single-precision values. No direct caller is known, and no table of code pointers reaches it.
- Behavior: each operand is classed as zero, normal or special (infinity or not-a-number). The significands are multiplied to a 64-bit product, the product is normalised by at most one bit, and the result is rounded to nearest with ties to even, using the guard bits. The result is then packed.
- Clean side: not needed. This routine rounds to nearest, but the game's float arithmetic runs on the hardware floating-point unit, which truncates (Platform arithmetic). Do not use this routine's rounding for game rules; the rounding rule is recorded for reference only.

### Cooperative tasks and keyed table (replaced; Low-Medium)

- The boot program can start a cooperative task, a routine that runs in steps and yields between them. Starting one copies a fixed-size context template (about 760 bytes) found through a keyed table, sets an owner value from the caller, and runs the task to its first yield.
- Keyed lookup: the table is sorted by key and is walked until a key larger than the request is found. The matching entry is copied to the caller's buffer. When no entry matches, nothing is returned.
- Which game routines start tasks is not established, so the game-level use is open.
- Clean side: provide a cooperative task mechanism only where the game's own code needs one. The template table is not needed.

### Fixed byte queue (Medium for the rules; Low for its users)

- A ring queue of fixed capacity that holds fixed 272-byte records. A push copies one record and wraps at the end of the ring.
- The original asserts that the queue is not full and that its front and back do not collide, and stops on a failure. Clean side: a push to a full queue must be an error, not a silent drop or overwrite.
- Its users in the game are not established.

### Type registry walk (Low-Medium)

- A walk over a list of type ids. For each id found in the run-time type registry, a type object is created and its init step runs.
- Its sole known direct caller is a C++ runtime termination wrapper. That call supplies a list count of zero, so it creates no type objects from the supplied list. The wrapper itself has no known direct caller and belongs to the C++ runtime support above; no player-facing use is established. Clean side: no equivalent is needed for the known path.
- Test vector: a list count of 0 causes zero id lookups and zero type-object creations, regardless of the list contents.

### Table-driven bit readers of the movie video plane decoder (Medium for role; Low-Medium for field meanings)

- Three bit-reader routines decode the plane data. Each reads bits through a 32-bit buffer that refills from the stream when fewer bits remain than the next code needs. Bits are taken least significant first.
- Each table entry is one byte: the high nibble is the code length in bits, and the low nibble indexes a 16-entry value table held in the reader's state.
- Before block decoding begins, each plane's setup reads code descriptions from its compressed bitstream and expands them into local lookup maps. One setup pass builds 16 consecutive maps of 16 bytes each; additional maps are initialized in the same stage. These generated maps support the block readers and are distinct from the 16-value table indexed by each map entry. The exact mapping of every map to a block type is not established (Medium).
- One routine writes two decoded values per output byte (the first in the low nibble). Another writes a count-prefixed run or a list of direct values. A third does the same with signed values (a sign bit selects negation).
- Their callers are the plane block decoder described above. They are movie data only; no sound path is known (Medium).
- Clean side: none needed beyond the movie decoder, which the Bink-compatible decoder replaces.

### Texture upload packet builder (covered in SPEC-0003)

- The builder that this section first listed as provisional is specified in SPEC-0003 (Runtime upload packet builder). Its effect for the clean side is the texture upload described there, carried out by the GPU renderer (SPEC-0046). No separate item is kept here.

## Movie decoder helpers (Bink support)

The Bink decoder replaces the original's movie decode. The helpers below are the original's own pieces, located on the movie path in the boot program. They show what the original did, so the clean decoder's output can be confirmed against it. The video helpers are called from the movie frame path below. The audio helpers are called from the movie audio chain. The mixer that was once thought to be a game sound mixer is itself movie-library code (Movie audio internals), so no game sound path is known to call them (Medium-High).

### Movie frame path (order of work, Medium-High)

Once per game tick, for an open movie, the movie stream step runs. Per frame it does the following, in this order:

1. It decodes the frame's video planes through the movie frame dispatcher: the full-size luma plane, then the two half-size chroma planes. Each plane is decoded block by block (see Movie video plane decoder), with each block going through the inverse transform and the block add.
2. It runs the movie audio step for the frame's audio (see the audio helpers below).
3. The finished frame goes to the output step, which converts it with the colour kernels to 32-bit pixels (512 by 448). The frame is then shown as a full-frame texture.

The movie player opens the file and checks its magic first, then loops over the frames, waiting for the graphics transfer channel before each frame is handed over (SPEC-0026, Playback).

**Presentation and end of play (High).** Presented frames alternate between the two 512 by 448 display buffers (the same two buffers the boot pictures use; SPEC-0026, Visible boot order). The skip check, when enabled, runs once after each presented frame. The loop ends at the last frame, on a stream error, or on the check after the latch (SPEC-0026, Movie skip). In every case the player then closes the stream, stops the movie sound, and waits until the sound side no longer reports busy, before it returns. A skipped movie and a movie played to its end leave the player in the same state, and the last presented frame stays on screen.

### Colour conversion (Medium-High for the main path; the edge path is Medium-High)

- The main movie path converts the aligned middle of each row, eight pixels at a time. Luma is reduced by 16 (saturating at 0), and both chroma values by 128. It uses the standard BT.601 limited-range matrix: red = 1.164 Y + 1.596 Cr; green = 1.164 Y − (3209 / 8192) Cb − 0.813 Cr, where 3209 / 8192 is about 0.391724; blue = 1.164 Y + 2.018 Cb. Results are clamped to 0 to 255 and written as 32-bit pixels, the output form the movie path uses.
- The edge pixels (the unaligned start and end of a row, and single rows that start on an odd line) use a scalar converter. Its luma goes through a ramp with slope 32,173 over 32,768 (about 0.982). Its chroma uses the four constants 1.5947 (red from Cr), 0.8129 (green from Cr), 0.3907 (green from Cb) and 2.0170 (blue from Cb), in 15-bit fixed point.
- The 16-bit output modes (8 and 9) also use the ramp and the same chroma constants. Each pixel's three channels are five bits each, packed at bit offsets 0, 5 and 10 (15-bit RGB order). A 16-bit eight-pixel converter with the 1.164 constants also exists in the boot program but is not referenced by any known path, so it is not part of the movie behavior.
- A separate table builder makes four tables of 256 entries, built when a mode argument changes. Entry i holds round((i minus an offset) times a constant) in 15-bit fixed point. The constants are 1.5947 (red from Cr), 0.8129 (green from Cr), 0.3907 (green from Cb) and 2.0170 (blue from Cb). The table green-from-blue coefficient is 12,804 / 32,768 (about 0.390747). The main SIMD path uses 3,209 / 8,192 (about 0.391724) for the magnitude of its green-from-blue coefficient. These are distinct fixed-point approximations; the other chroma coefficients are also approximated separately. The order in which the tables are used is not confirmed.
- Reconciliation of the two luma values (Medium-High). They are two paths in the same build, not a conflict. The main movie path (the aligned middle of each row, 32-bit output) uses 1.164. The 0.982 ramp is used by the edge converter and the 16-bit output modes. The ramp maps entries 0 to 16 to 0 and entries 235 to 255 to 255, and the clamp tables used by the edge converter are built the same way. It reaches 214 at entry 234 and jumps to 255 at entry 235, which a 1.164 slope would not do. That jump is a quirk of the ramp table, not a viewer-visible rule. The default movie copy uses a zero origin and 512-pixel rows into an aligned buffer, so its four-pixel row converter has no scalar prefix or tail on disc movies (all 512 by 448). The separate edge and 16-bit modes remain in the library.
- Packing: the channel order of the 16-bit modes is not confirmed.
- Clean side: use the standard limited-range BT.601 matrix with the 1.164 luma factor for the movie path. Use the 0.982 ramp only if the clean side must match edge pixels and 16-bit output exactly. Packing is not needed for the 32-bit path; the clean side uses its own pixel formats. A standard colour-conversion routine or a GPU shader gives the conversion.
- Checking values (not a reimplementation rule): limited-range luma 16 with neutral chroma (128, 128) gives 0; luma 235 with neutral chroma gives 255.

### 8x8 inverse transform (Medium)

- Each 8x8 residual block is decoded from coefficients as follows. Each coefficient is multiplied by a per-block dequantisation table entry chosen by a block-type index, then shifted right by 11. The table itself is part of the movie stream and is not copied here.
- A row pass and then a column pass of butterfly operations run on the block. The multipliers are fixed-point constants in units of 1/2048: about 1.414 (0xB50), 1.848 (0xEC8), 1.082 (0x8A9) and 2.613 (0x14E8). These are the standard 8-point inverse transform constants.
- The final output pass adds 127, shifts the unsigned result right by 8, and stores the low byte as a signed residual. This fixed bias-and-shift is asymmetric at signed half-step boundaries; do not replace it with a generic symmetric rounding rule when matching output.
- Clean side: the open Bink-compatible decoder's transform. Use a standard 8x8 inverse DCT in floating point or integer form; the constants above are for checking output, not for copying the routine. Keep the rounding rule if the decoder is bit-exact to the original.

### Inverse-transform stages (summary; Medium)

- The 8x8 inverse transform above runs as three kinds of stage: a butterfly stage over 16 values (sums and differences of pairs, with one group multiplied by a constant from the argument block); a dispatcher that picks a path by block size (small sizes use direct leaves, larger sizes split in halves and recurse); and a finishing pass that writes the output. A second finishing pass variant exists.
- Callers: the dispatcher and its variant are called by two routines, and both of those are called by one routine that was not identified (Medium). Whether that routine is on the movie path only is not confirmed.
- Clean side: the game depends only on the decoded frames. The Bink-compatible decoder supplies the transform, so no copy of these stages is needed.

### Block reconstruction (Medium for the shape, Medium-High for the caller)

- A predictor block of 64 values (8 by 8) comes from a helper with two inputs. The 64-byte residual from the transform above is added to the predictor byte by byte with wrap-around (modulo 256, no clamp). The eight rows of eight bytes are written at a row pitch.
- Each pixel result is prediction plus residual.
- Caller: the movie video plane decoder (Medium-High; see its section above). So the block add serves movies.
- Clean side: part of the Bink decoder; the wrap rule is kept only if the decoder must match the original bit for bit.

### 32-point transform pass (Low-Medium)

- One pass over 32 float values in place, in three stages: sums and differences of the two halves; the same on each quarter, with rotation terms formed from two twiddle values; then a final combine. It is one pass of a fast transform of the cosine family.
- Caller: on the movie audio chain (reachability only). The mixer once listed in SPEC-0031 is movie-library code (Movie audio internals), so no game sound caller is known (Medium-High).
- Clean side: a standard fast transform gives the same values if the transform and twiddle values match. The twiddle values are not recorded here and must be read from the original data if that transform step is needed.

### Stream step (Medium-Low)

- Once per game tick, for an open movie stream: keeps a ring of frame timing records (each 0x17C bytes). For the 30 fps and 29.97 fps files on this disc, the ring has 30 slots, derived by rounding the header's frame-rate ratio to the nearest whole number. Its write index moves backward and wraps from slot 0 to slot 29. The stream step reads the game clock, starts the first frame after a latency value passed in by the caller, and copies decoded data into destination buffers, wrapping at the end of each buffer. It then calls an output routine with the timing record. It returns 1 when the stream has finished, otherwise 0. One non-clock lane records the value at state offset +0x2b4: a cumulative clock-delta accumulator. When a nonzero marker at +0x27c is consumed, the accumulator adds current clock minus that marker and the marker is cleared; the stream step refreshes the marker to the current clock at its end. The accumulator's intended metric remains unknown.
- Clean side: the decoder and video path own the timing. Frames show at the header's frame rate (about 30 fps; 29.97 for the act intros and ACT2 210A) on the 60 Hz tick, and the stream ends after its last frame. The first-frame latency the real caller passes is open.

### Sample accumulator (Medium)

- Caller: on the movie audio chain, reached from the movie stream step (Medium). The mixer once listed in SPEC-0031 is movie-library code (Movie audio internals), so no game sound caller is known (Medium-High).

- For each output sample it reads source sound data, scales it, and adds it into a 32-bit accumulator buffer. Sixteen modes are selected by a mode number:
  - An 8-bit unsigned sample u adds (u minus 128) times 2^19 (modes 0, 2, 6).
  - A 16-bit signed sample s adds s times 2^11 (modes 1, 3, 5). Both scales meet the same full scale: the extreme values of either give plus or minus 2^26, so 8-bit and 16-bit sources land at the same level.
  - A two-tap 8-bit mode (4) adds (b0 plus b1 minus 256) times 2^19, where b0 and b1 are two consecutive samples.
  - Two-channel modes (2, 3, 6, 10, 11, 14, 15): one source frame writes two accumulator slots, one per channel.
  - Gain modes (8 to 15) multiply by one or two integer gains held in the state. Mode 12: b0 times gain A plus b1 times gain B minus 128 times (A plus B). Mode 13: s times gain A plus the next sample times gain B.
- Read pointer: each output sample adds a phase increment to a phase kept to 31 bits. The carry out of the top bit selects one of two strides held in the state: the first stride when the carry is set, the second when it is clear. The read pointer advances by the selected stride.
- Clean side: the sound path converts each sample format to its mixer's format with the same scale. Relative to a full scale of 2^26, an unsigned 8-bit sample becomes (u minus 128)/128 and a signed 16-bit sample becomes s/32768. The gains, the two-tap rule and the stride rule belong to the sound specs (SPEC-0009, SPEC-0031) once the caller is known.

### Movie audio internals (replaced; High)

These routines were once listed in SPEC-0031 as game sound code. They all lie in the linked Bink movie library and run only for movie audio, so they are not part of the game's sound system. A Bink-compatible decoder gives this behaviour; the facts are kept only to check its output.

- **Overlap blend (the former "crossfade").** The first n samples of each decoded audio block (n = the overlap length in 16-bit samples) become out[i] = trunc((cur[i] x i + prev[i] x (n - i)) / n) for i = 0 to n - 1, with integer division toward zero. The block's last n samples are then saved as prev, and the block's playable length is its length minus the overlap. The first block after a reset is not blended. Test vector: n = 4, cur 100, prev 200: i = 0 gives 200, 1 gives 175, 2 gives 150, 3 gives 125.
- **Track gains (the former "stereo gain split").** Volume x (0 to 65536, unity 32768) and pan y (0 to 65536, centre 32768; setting the pan clamps y to at most 65536). Gain A = (x x (65536 - y)) >> s and gain B = (x x y) >> s, an unsigned shift with s = 12 when the track's format bit 0 is clear and s = 20 when it is set. Unity short-cut (library quirk): when the stored gain A is at least 0x7FFF, x <= 0x8001 and 0x7FFF <= y <= 0x8001, the scaling flag is cleared and the gains are left unchanged; the test reads the old gain A, not x. Test vector: x = 1000, y = 0x4000: s = 12 gives A = 12,000 and B = 4,000; s = 20 gives A = 46 and B = 15.
- **Sound buffer size (the former "stream chunk size").** k = (flags bit 0) + (flags bit 1); size = max(4096 k + 4096, 7096) bytes. The game's movie player is the only caller and passes flags 2, so the size is 8,192 bytes.
- **Sound-track records (the former "voice list" and "voices").** The 0x17C-byte records are the library's sound-track records, each with its own ring buffer filled from the track's source; the stream step's ring uses records of the same size (Stream step). Their fields are library internals and are not needed.
- **Track refill pump (the former "voice pump"; Medium).** Called from the movie stream step and from the library's per-frame buffer refill, with one flag that selects a fill variant. For each track in use it asks the track to refill its ring, copying with wrap at the ring end; when any track produced data the pump returns 1. When a track's source has ended or has no data, a set mode word makes the pump zero-fill a region of the ring (with wrap; the length comes from a stored word of the track) and clear the mode word; with the mode word clear nothing is written. The fill length on a looped track is not established.
- **Output mix pump (Medium-Low).** It has no direct caller. It takes a byte count in blocks of 1,024 bytes (count divided by 1,024, rounded down; a count under 1,024 mixes nothing). Each block zeroes a 2,048-byte scratch buffer and copies each active track's available bytes into it, wrapping at the end of the track's ring; whether the copy adds or overwrites is not established. A track with no data left is marked stopped. The buffer goes, with the output position, to one of two writers chosen by a stored flag, and the position advances by 1,024 bytes. It returns 1 when the output device exists, otherwise 0.
- **Movie close (the former "voice release"; Medium).** Called by the game's movie player when a movie ends. It stops playback, calls each track's stop hook in order and releases the track handles, releases the playback handle, and zeroes the library's 0x394-byte per-movie record so it can be reused.
- Clean side: a Bink-compatible decoder and the platform audio output replace all of these. The observable result is continuous movie audio for the movie's length, silence where the stream has no data, and every track stopped when the movie ends.
- **Mixer and stream pump conversions.** The library mixer turns 32-bit accumulator values into 16-bit samples by an arithmetic shift right of 11, clipped to plus or minus 0x7FFF (-0x8000 is never produced), and into 8-bit offset-binary samples (a zero sample gives 0x80). The stream pump's optional 16-bit to 8-bit conversion takes the high byte of the sample with the bias 0x80 subtracted.
- Whether the game sets a movie track volume or pan at all is open (the set-pan entry has no direct caller), so whether the movie gains are unity is not established (Medium).
- **Not player audio slots (moved from SPEC-0009; High).** An earlier reading grouped some routines as per-player audio slots. No such slot exists in the game's sound system:
  - The voice pump, the output mix pump, the voice release and the 0x17C-byte voice records are the track refill pump, the output mix pump, the movie close and the sound-track records above. All of them lie in the linked movie library, next to its file-open routine, and use its per-movie record (track count, track record list of 0x17C-byte records, 0x394-byte record). The movie close has one caller, the game's movie player, which calls it when a movie ends. They run only for movie audio.
  - The "audio request for a player slot" is the script-trigger fire step (SPEC-0030, the trigger record and the fire step): its "slots" are trigger records, its states 0, 1 and 2 are the record's state word, and the "handle" it starts is the trigger callback's task. It plays no sound.
  - Clean side: implement neither. Game sound effects are the sound slots and cue calls of SPEC-0031; the 0x170-byte record list once read as sound instances is the world-item pool (SPEC-0009, World-item records).
  - Test vector: a movie with two audio tracks ends. The movie close stops playback, calls the stop hook of track 0 and then of track 1, releases both track handles and the playback handle, and leaves all 0x394 bytes of the per-movie record zero. The movie close itself changes no game sound slot (SPEC-0031).

## Debug overlays (do not implement as a feature)

Current state: the boot program contains a debug watch screen with a variable tuner, and text routines that draw hero and character state as debug text. The watch screen can be reached in play after the SPEC-0033 button sequence (Medium-High), but it shows no variables in this build because no code registers any. Some of its controls then hang the game. **Do not implement the watch screen or the tuner as a feature.** The rewrite may offer its own read-only debug view, which must not change game state. The text routines at the end of this section are separate. A separate debug command list (see Developer and test leftovers) is installed at every start-up; no player-reachable call into it is known. Confidence: High for the empty variable list; Medium for the freeze and the write to address zero (inferred, not run).

### Variable list (data layout, Medium-High)

- The list holds up to 40 records. Each record is 36 bytes and holds: a kind (signed 8-bit, unsigned 8-bit, signed 16-bit, unsigned 16-bit, one of two 32-bit integer kinds (signedness not confirmed), or float); a reference to the value; a reference to an element index (empty for a scalar); a label of under 64 characters; a sub-field; a wrap limit (used by the 16-bit kinds); and a step (the integer or float step for the kind).
- A record whose value reference is empty is skipped when the tuner moves.
- Exact byte offsets are not given here. No registration path is known, so the layout has no use in this build (see Current state below).

### How the watch screen opens (Medium-High)

- The screen can open only after the session flag is set. The flag is set by the ten-step button sequence in SPEC-0033.
- While the flag is set, the first pad position is checked on every game tick. If L3 and R3 are both held, the screen opens on that frame. A button newly pressed on that tick counts as held.
- While the screen is open it blocks: the main loop does not advance until the screen closes.
- The check runs in the main loop with no game-mode test at its call site, so as far as the routine shows it can open in any mode. Mode gating is not confirmed (see Open questions).

### What the screen shows (Medium for drawing details)

- Each frame the screen reads the first pad position again, runs the tuner, draws the window and presents the frame.
- The window has one line per registered variable, up to 42 lines at a time. Each line has the label on the left and the value on the right. The selected line is marked with ">" to its left and "<" to its right. Lines are about 14 display units apart (approximate).
- Value text: an 8-bit signed value is shown as its number followed by its character in parentheses; an 8-bit unsigned value the same way with an unsigned number; 16-bit and 32-bit integers as plain numbers; a float as its integer part, a decimal point, and six digits of fraction.
- With no variables, the window has no lines and only the frame is drawn.

### Controls (High)

- Up and Down, held: select the previous or next variable, wrapping at the ends. Because they are held rather than newly pressed, the selection moves every frame while held.
- Left and Right, held: move the window by 42 entries. Left works only when the window is not at the start; Right only when it is not at the end.
- Triangle, held: add to the selected value. Cross, held: subtract from it.
- Start, newly pressed: set the selected value to 0.
- Edits change the live game variable, not a copy.

### Tuner rules (High for the logic, Medium for intent)

- Timing: the first change happens on the first frame of a hold. The tuner then waits about 19 ticks (about 0.32 s) and repeats the change on every tick while the button stays held.
- Step: each change is the variable's own step times a multiplier. The multiplier starts at 1 and is reset to 1 when no button, or only L1, is held. At each change it grows by the number of frames held divided by 30, rounded down. Holding longer therefore makes changes larger, roughly quadratically in the hold time.
- Wrap: an 8-bit value wraps naturally (modulo 256). A 16-bit value at or above its wrap limit becomes 0; below 0 it becomes the limit minus 1. A 32-bit value adds plainly. A float adds its float step times the multiplier.
- Hook: a hook is called after each edit, but nothing sets it, so no hook runs.

### Leaving the screen (Medium-High)

- A fresh press of R3 (newly pressed while held) closes the screen and returns to the game.
- The open check runs again on the next frame. If L3 and R3 are both still held, the screen reopens at once. To leave for good, release L3 and then press R3.

### Current state: the variable list is never filled (High for no registration path; Medium-High for empty at run time)

- No code in the boot program registers a variable. The routine that adds a variable has no caller and no reference to it. The other list routines (reset, save and restore of the list, clearing one record, and the edit-hook setter) also have no callers. The script host calls contain no registration either.
- So the screen shows no lines and the tuner has nothing to edit.
- Consequences, inferred from the code and not run (Medium):
  - Up or Down held: the search for a variable never finds one and never stops, so the game freezes.
  - Triangle, Cross or Start: the edit writes through the value reference, which is empty, so it writes to address zero. The result on the original hardware is not established here; a crash or corruption is likely.
  - Left and Right: no effect.
- A player who completes the sequence in SPEC-0033 and holds L3 and R3 therefore sees an empty screen, and certain buttons hang the game.
- The script-side debug query always returns false, and the screen's entry does not check it (High).
- **Do not implement the watch screen or the tuner as a feature.** If a debug view is wanted, it must be read-only and must not change game state.

### Debug text routines

- Four routines (overlay text for the hero's state and for two other state words) each read a state word, switch on it to choose a format-string set, and append about 15 text lines, some only when a flag is set. The output is a text overlay on the debug screen.
- A fifth routine builds one debug line for a character state index: a name from a table of 16 entries (a default name for other values), then names for flags whose conditions hold, then two numbers.
- Confidence: Medium for their role as debug overlays; the switch cases are not fully trusted (one of them falls through in the routine and is not confirmed). The exact wording is not needed by the clean side.
- Clean side: none needed for play. An optional debug text view may show the same kinds of values.

### Developer and test leftovers (replaced; not needed for play)

Confidence: the list of what each routine does is Medium-High; the detail of each is Medium; reachability is open. None of these routines is needed for play, and all are replaced by the clean side. They are recorded so that a reimplementation does not mistake them for game features.

- **Screenshot writer.** Takes a picture of a frame whose width and height are signed 16-bit values in a graphics-object record (record offsets +0x28 and +0x2A). Each pixel is 3 bytes, and the width times height times 3 bytes is read. Before writing, the first and third bytes of each pixel are swapped (blue-green-red order to red-green-blue order). The picture is then written to a file, and the routine logs a message naming the file written. A related failure message for the scaled screenshot (no file handle available) is present; its use is not established.
- **Screenshot driver.** The only known caller of the writer is a driver that reports completion, runs a scale step, and then reports starting a host-side program. The driver has no known caller in the program (see Open questions).
- **PC-link command dispatcher.** Reads a command number from a link request record and dispatches it. The command numbers read are 1, 5 (which takes a completion callback), 6 and 10 (which carries four bytes). The routine logs messages when the link opens and when it closes. It is the developer link to a host computer and has no play function.
- **Random-number statistics dump.** A debug routine. It runs 256 trials of Index(2) and Range(0, 1) on the game's generator (see Library behavior) and prints histograms with 8 buckets, with a heading and a per-divisor label; a companion routine prints Range(1, 10) results. Both use only the game's generator. It draws nothing from the game state and is not needed.
- **Vector-unit self-test.** Runs a fixed sequence of vector-unit operations on fixed inputs and compares four results with fixed expected values, within a tolerance of 0.1. Returns true when all four match. The rewrite does not use vector units, so this is not needed. If a platform check is wanted, it can be a clean unit test.
- **Debug command list.** A table of debug commands, including the commands that set and clear a range of quest flags, a command that sets the game stage, a command that sets the dialog camera, and a command that grants items. The table is installed at every start-up in every session. No player-reachable call into it is known (Medium; calls through computed function pointers were not excluded). The watch screen described in Debug overlays is the only player-reachable debug screen known, and it does not call these commands. The rewrite need not reproduce them.

Clean side: none of these needs an equivalent. The screenshot can be a normal feature of the clean build if wanted; the others are not.

## Edge cases and error handling

- If a movie fails to decode, skip it as if the player pressed skip. Gameplay doesn't depend on movie content.
- The original answers a missing, unformatted or unreadable memory card with a card message (no card, no save data, or load failed; SPEC-0025, Card jobs). The clean side has no card: it treats a missing save file as "no save data" and an unreadable file, or one that is not exactly 0xC400 bytes, as "load failed" (SPEC-0025, Clean-side mapping of the card jobs).
- **Oversized sound command (High).** A sound command whose argument block is over 500 bytes stops the original after an assertion message. The clean side returns an error and continues. Players should never reach this case, because the game's own commands are within the limit.
- **Overlapping sound commands (Medium-Low).** A command sent while another is pending is reported as a protocol collision. Whether any shipped sequence of commands triggers it was not confirmed.
- **Unsigned remainder (High).** The integer and index draws take an unsigned remainder of the 32-bit output, so Range never returns a value below lo or above hi and Index never returns a negative index. Index(1) consumes a draw and returns 0. A range whose size overflows 32 bits (hi − lo + 1 = 0) and Index with a negative n are not used by any call site; their result is not defined.
- **Clock read failure (Medium-High).** If the clock read fails, the record buffer is not written and the seed is computed from whatever bytes it holds, so the seed is not defined. The clean side should use a fixed fallback seed and log it.
- **Second clock reader (High).** A separate screen-level routine also reads the clock record and converts its BCD fields to decimal for display. The seed rule above does not convert them. Do not "fix" the seed to decimal; that would change the sequence.
- A float range with a equal to b returns a without consuming an output.
- **Unaligned word reads (Medium).** The unaligned read helper accepts any byte address. Clean code that reads packed data at byte offsets must handle offsets that are not multiples of four, and must not assume alignment or round the address. Whether the game reads any packed data this way during play was not confirmed.
- **Type-information family (Medium).** It is reached only by the runtime's own type checks. No player-visible behavior is known. The clean side must not add type-information behavior to match it.
- **Bad or truncated movie frame data (unknown).** What the original does with a truncated or invalid frame was not recorded. The clean side skips the movie (see the movie edge case above) and logs the failure. If a non-movie disc stream is ever shown to use the plane decoder, the same rule applies to it.
- **Compare with a shared zero byte (Medium-High, reproducible quirk).** Two regions that agree up to a zero byte compare as equal (result 0) at that byte, even if later bytes differ. A plain byte-by-byte compare would return the later difference. The clean side should match this only where a call site needs it.
- **Sort ties (High).** Records with equal keys may come out in a different order from the input (see the sort section). No caller's tie order is visible in play, so reproduce the key order only.
- **Sort with no or one record (expected, not confirmed).** Nothing should change when the count is 0 or 1.
- **Remainder by zero (High).** The 64-bit remainder traps on a zero divisor and the original stops. No such call is known. The clean side returns an error instead of panicking.
- **Remainder of the most negative value by minus one (unknown).** No such path is known. The checked remainder returns no result for this case in Rust (see the 64-bit remainder section).
- **Block add wraps (Medium, original behavior).** Adding a residual to a predictor wraps modulo 256 with no clamp, so a sum above 255 becomes a small value. Keep this only if the decoder must match the original's output exactly.
- **Failed allocation (Medium).** The original returns null. The clean side gets an error or an abort. This is visible only if the game checks the null return, which is open.
- **Debug tuner wrap (High).** A 16-bit value at or above its limit becomes 0, not its limit, and below 0 it becomes the limit minus 1. An 8-bit value wraps naturally. A 32-bit value adds plainly (see Tuner rules).
- **Tuner on an empty list (Medium).** Up or Down held freezes the game; Triangle, Cross or Start write through an empty value reference (address zero). See Debug overlays, Current state.
- **Watch screen blocks the game (Medium-High).** While it is open the game loop does not advance, and it closes only on a fresh R3 press. It is debug-only, and reachable in play only after the SPEC-0033 sequence.
- **Special values in number text (Medium).** Infinity and not-a-number print as special words, not digits. The spelling is not recorded.
- **Sample accumulator overflow (unknown).** Whether the 32-bit accumulator clamps is not recorded.

## Test vectors

- Header check: a file starting with the text BIKi, with width 512 and height 448 at offsets 0x14 and 0x18, is a supported movie. Revisions f, g, h and i are accepted by the open check. A file with any other magic should be logged and the movie skipped.
- Frame time: numerator 10,000,000 with denominator 333,333 (30 fps) and 990 frames gives a movie of about 33.0 s (990 x 333,333 / 10,000,000 = 32.99997 s).
- Disc counts (test facts, from the 75 files): six folders with FRONTEND 2, ACTINTRO 4, ACT1 17, ACT2 11, ACT3 24 and ACT4 17 files, 75 in all. Total size 1,808,927,016 bytes. Frame counts run from 190 to 3,912, 92,292 frames in total. Frame time 333,667 (29.97 fps) on 5 files (the four ACTINTRO files and ACT2 210A); 333,333 on the other 70. Audio rate 44,100 Hz on 73 files and 48,000 Hz on 2 (ATTRACT and ACT2 210A). Movie name table: 92 entries naming 75 distinct files, with 17 entries repeating a name already listed.
- Edge ramp (synthetic, from the ramp above): with neutral chroma, luma 234 on the edge path gives 214 and luma 235 gives 255. The main path gives 255 at luma 235 (as above).
- Standard movie output window: 512 by 448 pixels, destination origin (0, 0), pitch 2,048 bytes; each of 448 rows starts at a four-byte aligned address, with no scalar prefix/tail for a four-pixel row converter.
- Platform arithmetic (single precision, results truncated toward zero): 25 x 0.01 = 0.24999999; 1 + 25 x 0.01 = 1.2499999; 4 x 1.2499999 = 4.9999995, which converts to the integer 4; 100 x 0.01 = 0.99999994; 40 x 0.99999994 = 39.999996, which converts to 39.
- Add alignment: 1.0 + (−1.5 × 2^-24) = 0.99999994 (truncating the exact sum would give 0.99999988). Fade in over 60 frames from alpha 128 with step −128 ÷ 60 = −2.1333334: alpha 2.1333725 after update 59 and 0.000039 after update 60, so the clamp to 0 comes on update 61.
- Seconds to ticks, trunc(f32(s) × 60.0): 0.7 → 41 (0.699999988 × 60 = 41.9999993, truncated to 41.999996, then to 41); 0.9 → 53; 1.3 → 77; 1.8 → 107; 2.8 → 167; 3.6 → 215; 0.35 → 20. Values that keep 60s: 0.05 → 3, 0.1 → 6, 0.2 → 12, 0.3 → 18, 0.4 → 24, 0.5 → 30, 0.6 → 36, 0.8 → 48, 1.0 → 60, 1.2 → 72. 0.01 → 0 (Wait yields nothing; a fade or bar call uses 1 frame). 0.0 → 0, so a fade over 0 s uses 1 frame.
- Divide: 1 ÷ 60 = 0.016666668 (to nearest).
- Start-up rate words: the region detector returns 0 and the override word is -1, so row 0 of the rate table at 0x30C598 (60.0, 30.0) is used: 0x368FA0 = 60.0, 0x368FA4 = 30 / 60 = 0.5 and 0x368FA8 = 1 / 60 = 0.016666668, and the video-mode record holds width 512 and height 448. The PAL row (50.0, 25.0) and the 512 x 512 mode are never used.
- Card event set (synthetic): raise 21, then test 21 gives 1 and test 22 gives 0. A request that fails with an empty set ends with code 22 set. Raising -1 or 24 changes nothing.
- Accumulating timer, t += 0.016666668 from 0: t = 0.49999976 after 30 ticks, 0.79999948 after 48, 0.99999928 after 60 and 59.999142 after 3,600. First tick on which t > T (and t >= T): 0.5 s → 31, 0.8 s → 49, 1.0 s → 61, 1.3 s → 79, 1.5 s → 91, 4.5 s → 271, 5.0 s → 301, 11.2 s → 673, 15.2 s → 913. t >= 0.05 first holds on tick 3, t > 0.05 on tick 4.
- Sound command size (synthetic): an argument block of 500 bytes is accepted; 501 bytes is refused with an error. A 500-byte block with the 12-byte header is the largest that fits the 512-byte packet.
- Sound command order (synthetic): commands A, B, C sent in that order are appended in that order. If A and B fit in the active staging buffer, both can be queued before the batch is submitted; if C does not fit, the sender waits for buffer availability before appending it. This specifies queue order and capacity gating, not processor playback timing.
- Byte scan (synthetic): scanning the bytes 4, 9, 9, 0 for the value 9 returns position 1; scanning them for 7 returns the end of the range (position 4).
- Unaligned word read (synthetic, clean-side expectation, not confirmed against the original): reading 32 bits at byte offset 1 of the bytes 11 22 33 44 55 returns the word that holds 22 33 44 55 in that byte order, and reading at offset 0 returns the word that holds 11 22 33 44.
- C-library generator from state 1: the first three outputs are 1103527590, 377401575, 662824084.
- C-library float helper with r = 1103527590, a = 0, b = 10: about 5.1387.
- Game generator, first 10 outputs after seeding (word 0 = seed, each next word 69069 times the previous, mod 2^32; seed 4357 reproduces the published reference output of the 1998 generator):

| Seed | First 10 outputs |
| --- | --- |
| 1 | 3796174982, 4182529786, 2180050607, 3809380472, 2960793660, 4051247694, 3989107140, 2094500863, 3349580919, 3767281029 |
| 4357 | 3510405877, 4290933890, 2191955339, 564929546, 152112058, 4262624192, 2687398418, 268830360, 1763988213, 578848526 |
| 860282 | 1830660608, 153682920, 3398361290, 1057550717, 3031632102, 2934766831, 3404529811, 574691166, 610019998, 1706036388 |

- Game generator, seed 1, across the regeneration boundary: the 624th output is 751518116 and the 625th is 1141860845.
- Game float range, a = 0, b = 10, first output u = 3796174982: about 8.8387.
- Wrappers on a fresh seed-1 stream (each line a new stream): Range(0, 100): 92, 0, 48, 34, 72. Index(100) (same as Range(0, 99)): 82, 86, 7, 72, 60. Range(1, 100): 83, 87, 8, 73, 61. FRange(0, 1): 0.8838659, 0.9738211, 0.5075826. FRange(1, −1) (swapped to [−1, 1)): 0.7677318, 0.9476422, 0.0151652. Seed 860282, Range(0, 100): 56, 7, 49, 18, 43.
- Unsigned-remainder check: seed 1, the first Range(0, 100) is 92 (a signed remainder would give −77, which is wrong).
- No-draw checks: Range(5, 5) = 5, Index(0) = 0 and FRange(0.0, −0.0) = 0.0, and the next draw still returns the next output of the stream.
- Hash range draw, synthetic: seed 1 with limits 0 and 10 gives h = 1316808037 and about 3.8681 (table in the random section; recomputed, not read from the disc).
- Reverse call walk, synthetic: slots 1, 2 and 3 hold A, B and C, and the stored count is -1. The walk calls C, then B, then A. With a stored count of 2 it calls B, then A.
- Merge, synthetic: first keys 1, 3, 5 and second keys 2, 3, 6 give the order 1, 2, 3 (first list), 3 (second list), 5, 6.
- Compaction, synthetic: slots A, 0, B, 0, C give A, B, C with count 3.
- Comparator, synthetic: a record with id -1 returns 1. Records with keys 7 and 4 (ids not -1) return 3. A first record (id 0, key 5) against a second (id -1, key 1) returns 4.
- Sort with the occluder comparator (synthetic, two records, insertion path): (id 0, key 5), (id -1, key 1) gives (id -1, key 1), (id 0, key 5). (id -1, key 1), (id 0, key 5) gives (id 0, key 5), (id -1, key 1).
- Attachment comparison (synthetic): distances 10.0 and 10.6 compare as 0 (equal); distances 10.0 and 12.5 compare as 2 when the second is 12.5, so 12.5 sorts first.
- Compaction first pass (synthetic): keys 3, 5, 4, 6, 1 keep 1 and move out 5, 6, 4, 3 in that order. Keys 2, 1, 3 keep 1, 3 and move out 2. Keys 1, 2, 2, 3 keep all four and move out nothing.
- Read queue order (synthetic): requests A then B queued before the pump step of pass 0, no sound stream reading, each read reported done at its first poll: A starts at pass 0, polls done at pass 1 and completes (callback) at pass 2; B starts at pass 3, polls at pass 4 and completes at pass 5. If a sound stream is reading during pass 1, every step after pass 0 moves one pass later.
- Clock seed example: record bytes second 0x30, minute 0x15, hour 0x12, day 0x05, month 0x06, year 0x26 (BCD) give seed 860282, and the first three outputs are 1830660608, 153682920, 3398361290. These values are synthetic, not read from the disc.
- Sort (synthetic, clean-side expectation): records with keys 5, 2, 9, 2 sorted ascending by key come out with keys 2, 2, 5, 9. The order of the two records with key 2 is not specified.
- Sort, count 1: the record is unchanged.
- Movie video plane decoder: no test vector yet. The plane stream format must be confirmed before one can be written.
- Formatted output (synthetic; expectation from the flag rules above, not run against the original): hash octal of 8 gives a leading zero before 10, "010"; hash hex of 255 gives "0xff" and hash hex of 0 gives "0"; plain hex of 255 gives "ff"; plus flag on 5 gives "+5".
- 64-bit remainder (synthetic): minus 7 by 3 gives minus 1; 7 by minus 3 gives 1; minus 7 by minus 3 gives minus 1. A zero divisor gives an error.
- Number text (synthetic): with g and trailing-zero removal, 1.5 gives "1.5" and 2.0 gives "2".
- Colour (standard-factor expectation for the movie path): limited-range luma 16 with chroma 128 gives 0; luma 235 with chroma 128 gives 255.
- Colour packing (synthetic): modes 8 and 9 with channel values (1, 0, 0) give 1; (0, 1, 0) give 32; (0, 0, 1) give 1024. The channel order is not confirmed, so this shows only the bit positions.
- Block add (synthetic): predictor byte 250 plus residual byte 10 gives 4.
- Inverse transform rounding (synthetic): final-pass intermediate 127 -> residual 0; 128 -> 0; 129 -> 1; -127 -> 0; -128 -> -1 (stored byte 255); -129 -> -1 (stored byte 255). This follows the +127, unsigned-shift-by-8, low-byte conversion and distinguishes positive from negative half-step behavior.
- Sample accumulator: an 8-bit sample of 0 (mode 0) and a 16-bit sample of minus 32768 (mode 1) both add minus 2^26. An 8-bit sample of 255 adds 127 times 2^19.
- Allocator (synthetic): a 1-byte request gets a block of at least 32 bytes. Releasing a null address leaves allocator state unchanged. Releasing a block with free neighbors on both sides produces one merged free range before bin insertion.
- File-read queue (synthetic): a 10,000-byte request copies in steps of 4,096, 4,096 and 1,808 bytes, then its callback runs once.
- Tuner (synthetic): a 16-bit value with limit 100 goes to 0 when incremented from 100, and to 99 when decremented from 0. A float value of 1.0 with float step 0.5 and multiplier 1 becomes 1.5 after one change.
- Movie timing ring (disc header values): `(10,000,000 + floor(333,333 / 2)) / 333,333 = 30`; `(10,000,000 + floor(333,667 / 2)) / 333,667 = 30`. With 30 slots, writing from index 0 decrements to −1 and wraps to index 29. In the identified playback path, the three ring lanes sourced from movie-state offsets +0x140, +0x144 and +0x148 each contain zero.
- Movie timing ring accumulator: state +0x2b4 = 100, marker +0x27c = 1,000, and current clock = 1,030 gives accumulator 130 and clears the marker. The next ring write copies 130 into its corresponding lane unless another timing update intervenes.

## Open questions

Numbers of removed questions are not reused.

- The clock-read failure path: which bytes the original actually uses. Not established to the end.
- Resolved: the C-library generator's helper is never used in play (no caller, no pointer to it and no level-module import).
- Save-file contents, and the screen that uses the second clock reader: SPEC-0025 and SPEC-0026.
- Video-mode record: the meaning of its other values (0x90 stored twice, 0xA8, 0x80 and 0x31, besides 1 and 2), which look like display register fields. Not needed for the clean side, which sets its own display mode. **[Low]**
- Confirm the controller button-to-action map: player and menu specs.
- Answered: every queued movie is skippable. The movie skip check runs once per presented frame; Cross or Start held for five counted frames after a neutral frame latches the skip (SPEC-0026, Movie skip). A movie whose inputs never include a neutral frame does not arm.
- Resolved in part: the recovered nonblocking send path batches ordered commands in two alternating staging buffers and waits for buffer capacity, not for each preceding command to finish. Exact audible timing and the transport's response to an RPC collision remain open.
- Sound channel: the meaning of each of the roughly 70 command kinds. The command table is not written here; it belongs in the sound specs.
- Disc-read wait: the exact completion point (inferred from names, Medium-Low), and how the original reports a failed read.
- Critical sections: whether the two shared lists belong to the system or to the game's own queue.
- Graphics wait: whether the original's wait is needed for any visible behavior on the clean GPU path, or is only a hardware-synchronization detail. Not measured.
- Byte-scan and cache primitives: only partly read; their exact edge behavior (for example a zero-length range) is not confirmed.
- Text helpers: the flush of a partly filled stream buffer at the end of a stream, and the return value of the digit rounding on success, are not recorded. Whether any text routed through the newline rule or the buffers reaches play (debug or log output is assumed only) is not confirmed.
- The transport's exact behavior after an RPC collision or other submission failure is not established. Ordinary staging-buffer exhaustion is handled by waiting and then enqueueing; the error path is separate and remains open.
- C runtime type-information family and unaligned word read helper (Medium): whether any game code path calls either during play rather than only at start-up or in the runtime's own checks was not confirmed. Both are treated as library code that the clean side does not need; a caller identified later would change that.
- Whether any non-movie disc file or resource passes through the movie video plane decoder. Movie data does (Medium-High, see its section). Other data is not established, and the evidence covers program code and its callers, not other disc data. Until one is shown, the clean side needs no decoder beyond the movie decoder.
- Answered: resources 144 and 145 are the emote clip banks, not movie decoder input; see SPEC-0009, Emote voice banks.
- Whether the plane decoder's table helpers match any DEFLATE format (bit order within a byte, window size, table details). Not confirmed. Not needed for movies.
- The plane decoder's output-size limit and error behavior on bad frames: not recorded.
- The block-level helper that the plane decoder calls about eight times per plane is not described (its role is bit reading or bundle decoding, Low).
- Resolved: the in-place sort's sign convention is positive for "first belongs after second" (ascending), and no caller's tie order is visible in play (In-place sort; High).
- Resolved: the in-place sort has two callers, the character attachment distance list (switched off by a build word that is 0) and the occluder volume builder (not reached). Neither runs in play (In-place sort; High).
- Answered: the texture upload packet builder is specified in SPEC-0003, Runtime upload packet builder.
- Luma factor (partly established, Medium-High). The main movie path uses 1.164. The 0.982 ramp is used by the edge converter and the 16-bit output modes, so it is a second path in the same build, not the movie path. Still open: a decoded-frame check against the original's output, to confirm the 1.164 path and edge behavior. Disc data alone does not settle pixel rounding.
- Answered: the main SIMD path uses a green-from-blue magnitude of 3209 / 8192 (about 0.391724); the scalar edge and 16-bit table path uses 12804 / 32768 (about 0.390747). Both are in use and differ by about 0.000977. Synthetic coefficient check: for Cb − 128 = 64, the green correction is −25.0703125 in the main path and −25.0078125 in the table path. Exact integer ratios: High; mapping the scalar tables to their edge and 16-bit consumers: Medium-High.
- Colour table order and the packing modes: which modes exist (the packing is seen only for modes 8 and 9), the channel order, and whether any disc movie uses them.
- Answered: the block add helpers serve the movie plane decoder (Bink-style, not DEFLATE-style), so the Bink-compatible decoder must include them; the LZO-style decoder belongs to SPEC-0005. **[Medium-High]**
- Inverse transform — resolved in part: the final output pass adds 127, shifts the unsigned result right by 8 and stores the low byte as a signed residual (Movie decoder helpers; Test vectors). The dequantisation table encoding and range, negative-product semantics at the earlier right shift, whether the stream format specifies this arithmetic, and whether output must match the original bit for bit remain open.
- Transform pass: whether any game sound path calls it. Answered: its caller is the movie audio chain, and the mixer once listed in SPEC-0031 is movie-library code (Movie audio internals), so no game sound caller is known (Medium-High). Its twiddle values are read from data.
- Answered in part: the stream step allocates seven parallel arrays for the ring, each with 30 four-byte slots on every disc movie. The count is the header frame-rate ratio rounded to the nearest whole number; both 30 fps and 29.97 fps files yield 30. The write index moves backward modulo 30. Three non-clock lanes copy movie-state words at offsets +0x140, +0x144 and +0x148; the normal playback code initializes them to zero, so these three lanes are zero in the identified path. A fourth non-clock lane snapshots state +0x2b4, a cumulative clock-delta accumulator updated from a nonzero marker at +0x27c; the exact accumulator metric and its consumer role remain unknown. The two other non-clock lane meanings, the stream's own block/frame layout, and the first-frame latency remain open.
- Sample accumulator: the gain values and where they come from, whether the accumulator clamps, and whether the stride is chosen before or after the phase is kept to 31 bits. Answered: its caller is the movie audio chain; no game sound caller is known (Medium-High).
- File-read queue: what the global device flag selects, the request record layout, and how a failed read is reported. Resolved: callbacks follow queue order, and the pump rules are in Asynchronous read queue (Medium-High).
- Answered within the inspected scope: no debug-variable registration path was found in the boot program or the searched script-host interface. The variable-add routine has no identified caller or reference, and the list reset, save, restore, clear-record and edit-hook routines have no identified callers. The player-reachable watch screen therefore has no known registration source and is expected to have an empty list. This does not rule out a computed call or an unsearched module/runtime path; confidence Medium-High. A newly found registration path would change this conclusion.
- What the freeze and the write to address zero look like on the original hardware. Not confirmed. Settling it needs a test on the user's own build or documented behavior; this room does not run the original.
- The meaning of the L3 newly-pressed bit that a per-frame debug object update reads (that update also clears a shared word). Its role is not established.
- Debug tuner pad mapping: the screen reads the first pad position. How that position maps to a physical port was not confirmed here.
- Debug tuner: the signedness of the two 32-bit kinds is not confirmed. The 8-bit wrap is stated as natural (modulo 256) from the code, not from a test.
- Debug overlay switch: one of the overlay switches falls through in the routine and is not confirmed; the state meanings are not needed for play.
- Formatted output: which conversions and flags the game's on-screen text uses, and which callers of the routine are on screen. This decides how much of the formatter the clean side needs.
- Default precision for e and f when the caller gives none (six is assumed, not confirmed). The watch screen's float display assumes six digits too.
- Number text: whether the original's digit method gives the same last digit as a correctly rounded formatter for the values the game shows.
- The spelling and case of the infinity and not-a-number words.
- The most negative 64-bit value divided by minus one: what the original does (no such path is known).
- Resolved in part: the allocator free path joins adjacent free blocks, maintains size-ordered bins and can trim the heap top; see Allocator release path. Whether game code checks the null return remains open.
- Movie audio channel layout (mono or stereo) and the meaning of the high flag bits at the audio word (Low). The audio chain was only confirmed as reachable.
- Movie audio gains: whether the game sets a movie track volume or pan at all (the set-pan entry has no direct caller), so whether the movie track gains are unity (Movie audio internals). **[Medium]**
- Movie audio size unit: whether the value at offset 0x2C is decoded bytes per frame or per track (not confirmed).
- Resolved: the movie stream uses the header pair as a seconds-per-frame ratio. Its startup timing path computes `floor(1,000 × denominator / numerator)` milliseconds and uses that interval for the timing record; the frame-time numerator on all disc movies is 10,000,000, so the pair represents 100-nanosecond units per second. For the 30 fps pair 10,000,000/333,333 the result is 33 ms; for 10,000,000/333,667 it is also 33 ms (High for the calculation and values; Medium-High for the unit interpretation).
- Answered: the default movie copy uses destination origin (0, 0), dimensions 512 by 448, and a 2,048-byte row pitch. The resulting 32-bit destination base and each row start are four-byte aligned; 512 pixels divide evenly into four-pixel groups, so the scalar alignment prefix and row tail are empty on every standard disc movie. The presentation packet centers a 512 by 448 image in the 512 by 448 NTSC canvas (origin 0, 0). The library still has edge and 16-bit output modes for other calls. Evidence and confidence: Medium-High; a decoded-frame check remains open.
- Movie decoder: the stream's own block size, the frame layout and the number of frame timing slots (already listed above under the stream step) remain open for the Bink-compatible decoder.
- Developer leftovers, reachability: none of the four routines (screenshot writer, PC-link dispatcher, random-number statistics dump, vector-unit self-test) has a direct caller in the program, and no code-pointer table entry is known for them. The screenshot writer is reached only through its driver, which also has no known caller. A play path is therefore unlikely but not proven. Settle it by a black-box test of the user's own build (for example, whether any key or menu produces a screenshot file or a link message) before the clean side drops them for good.
- Developer leftovers, detail: the exact picture format written to the file (header, byte order of the file itself, and whether the scaled variant is used anywhere), and the meaning of the link command numbers beyond the four known ones. Answered: the statistics dump uses only the game's generator, through Index(2), Range(0, 1) and Range(1, 10).
- Developer leftovers, the vector-unit self-test's expected values: the four values were taken from the code and are not repeated here; they are needed only if a platform check is ever written.
- Resolved: two random sources. Every gameplay and effect draw, float draws included, uses the one Mersenne-Twister state at 0x39BE50, seeded from the console clock at the first draw; the hash draw and the C-library generator are not reached in play (see Random numbers).
- Hash range draw: whether an unlisted indirect path reaches the effect counter that is its one caller (Medium-High that none does). Answered for play: that counter has no caller or table entry.
- Which start-up routine creates the first hero and makes the first draw (its ambient cue timer) is reached through an indirect call; Medium-High that it is hero set-up.
- The console's float rounding for the real draw (whether float(u) for u near 2^32 rounds up so that b0 can be returned) is not observable; the clean side uses [a0, b0).
- Reverse call walk: which table uses the -1 count form.
- Platform arithmetic: whether any game value reaches the other non-standard traits of the console's floating-point unit (no infinity or not-a-number values, and very small values read as zero). **[Low]** Answered in part: square root rounds to nearest, not toward zero; see Platform arithmetic. Divide and add rounding are also described there (Medium-High). Whether play reaches the non-finite or denormal cases remains open.
- Resolved: **Compaction, first pass.** An entry moves out when a later entry has a strictly smaller key while it is on top of the kept stack; equal keys stay (C runtime helpers; High).
- Resolved: **Sorted interval index.** It is the C++ exception unwinder's code-range table; its only user is the unwinder's frame-record lookup, which never runs in play (High).
- Resolved: **Comparator direction.** A -1 entry usually sorts last but can sort first, because only the first record's id is tested; the builder that uses it is not reached (C runtime helpers; High).
- Library items still open: (a) the two callers of the compare, which decide whether the shared-zero stop matters to the game; (b) whether any non-movie disc data uses the movie video plane decoder's table-driven bit readers (they are movie plane code, and no sound path is known); (c) which game routines start cooperative tasks, and whether their order matters; (d) whether the software multiply has any caller at all. The type registry walk's known caller and its zero-count invocation are described above; indirect or external entry remains unestablished.


