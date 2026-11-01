# SPEC-0009: Audio clips, music streams and music selection

- Answers: proactive; proactive (open close-out)
- Status: Draft
- Disc build: SCUS-97367 v1.00 (same fingerprint as SPEC-0001)
- Confidence: World-item records (below): High that they are the world-item pool of SPEC-0021 and play no sound; timed queue Medium-Low. VAG one-shots: High. Emote voice banks (resources 144 and 145): layout, directory positions and table values High; roll rule and directory lookup High; hero shift High; character stride and voice sets High; emote clip playback High; survey of reachable clips High (character reach in play Medium); emote requests are described in SPEC-0022. Stereo streams: Medium-High (layout and decoding), Medium (exact end). Music selection: music row to sound bank, stream and sound block Medium-High; the level-script override (value 64 starts bank 74, -1 clears it) High for value and path; the override clears High (screen names Medium); interactive segments (setup child, command-driven switching, looping, templates on disc) High; the game's two intensity inputs (register 2 and the global excite) High, the hero excite target Medium-High (bosses never counting Medium-High), the adventure zones High; the same-tick order of children High; the missed-combat quirk Medium-High; the world-load audio reset High; the controller on front-end passes High; the music register setter High; reverb pairs High (effect mode names Medium-High); the run-time flag Medium for its story meaning and High for its rule; the per-tick controller Medium-High (High for the order of its steps); the area-to-row rule Medium-High for packs with a type-1001 record and Medium for the fallback; volume ramps and stop fades High; stream crossfade Medium-High; the 21 banks that no known code selects Medium (banks 18 and 35 are reached by the front-end bank overrides, High). Stingers: the clip mapping (table 14) High; the bound on indices High; one-at-a-time, cooldown and music ducking Medium-High. Track selector for tracks 163 to 166: not reached in known code (Medium). The SetMusic host call: no script call site (Medium-High). Track selector fade pair, wait and stop: Medium-Low; per-track flags: Low. Default cue volume and the area-slot users: High. Gain chain (Music, Sound-effects and Voice gains, start-up gains 1.0, emote-clip volume): High. Pause level and stream gate: High. Sound banks and sound blocks: SPEC-0031. Player audio slots: none exist; the routines once listed there are the trigger fire step (SPEC-0030) and movie-library code (SPEC-0008): High.

## Summary

Two of the disc's audio families are plain Sony PS-ADPCM, a publicly documented 4-bit ADPCM format with 16-byte frames of 28 samples: **mono one-shots** (VAG) and **stereo streams**. With this spec the clean side can decode them to PCM and play them. Music is played from the sound banks (resources 10–143, SPEC-0031, section 8): a music row selects a whole bank, and the same row selects a stereo stream (resources 146–255) and a sound block. This spec describes the per-tick music controller, how the game's two intensity inputs (combat intensity and the adventure level) switch the interactive segments of a bank, the volume ramps and the stream crossfade, the level-script overrides and the track selector for music requests as far as they are known. The sequence format and the interactive-music commands are in SPEC-0031, section 8. Sound banks (resources 10–143) and sound blocks (347–490) use the sound driver's bank format and are in SPEC-0031.

## Where it lives on the disc

| Family | Resources (SPEC-0002) | Count |
| --- | --- | --- |
| VAG mono one-shots (stingers) | 256–274 | 19 |
| Emote voice banks (VAG-headed clips, no index table; see Emote voice banks) | 144–145 | 2 |
| Stereo streams (" KPV") | 146–255 | 110 |
| Sound banks used for music (MIDI container at the bank tail, SPEC-0031) | 10–143 | 134 |

Two design tables in resource 533 (SPEC-0024) drive the choices in this spec: table 4 (music row to bank, stream and block offsets) and table 14 (stinger clip choice). The area-to-row keys (a 1001 record in each world pack, a default record at boot-executable data address 0x2DA3D0, and two 193-slot tables at data addresses 0x31F2B0 and 0x31EFA8) are described under Music selection, Area to music row.

Resource 576 is not a sound bank. It holds 88 book records, which are not clips and are not decoded by this spec. It is not counted in any family in the table above.

## Layout

### PS-ADPCM frame (both families; public format)

16 bytes per frame, 28 samples:

- Byte 0: high nibble = filter index (0–4), low nibble = shift (0–12).
- Byte 1: flags. Bit 0 = loop end / end of data, bit 1 = loop repeat, bit 2 = loop start.
- Bytes 2–15: 28 signed 4-bit samples, low nibble first.

Decode with the standard filter pairs (0, 0), (60/64, 0), (115/64, −52/64), (98/64, −55/64), (122/64, −60/64), carrying two history samples per channel.

**Who decodes, and the predictor rounding (High that the game program holds no decoder; Medium-High for the hardware rule).** The game's own program has no routine that unpacks these frames: it holds no filter-coefficient table and no nibble-unpacking loop, and none of the sound modules shipped on the disc holds the coefficient pairs either. One-shots, bank samples, music blocks and streams are all passed to the sound modules and the sound chip as raw 16-byte frames, and the chip decodes them while it plays. The movie audio is a different codec (SPEC-0008). So the game has no software rule to follow, and the rule is the sound chip's. That rule is public: for each sample, take the signed 4-bit value, move it up by 12 bits, move the result down by the frame's shift with a sign-keeping shift, and add a prediction from the two previous output samples h1 (newest) and h2. The prediction is (c1 x h1 + c2 x h2 + 32) moved down by 6 with a sign-keeping shift, where (c1, c2) is the filter's pair in 64ths: (0, 0), (60, 0), (115, -52), (98, -55) and (122, -60). The sum is limited to -32768 to 32767 and becomes the new h1. This is the rounding form (the +32 before the shift), not plain truncation. A decoder in the rewrite should use it. Dropping the +32 changes a sample by a few units at first, growing to about 25 within one frame (see the real clip vector); the hardware figures cannot be measured from the disc, so this rests on the public documentation of the chip.

### VAG (resources 256–274)

Header fields are **big-endian**:

| Offset | Size | Meaning |
| --- | --- | --- |
| 0x00 | 4 | ASCII `VAGp` |
| 0x04 | 4 | Version (32) |
| 0x0C | 4 | ADPCM data size in bytes |
| 0x10 | 4 | Sample rate: 22,050 in all 19 |
| 0x20 | 16 | Name, ASCII, NUL-terminated (a short label such as "Revive") |
| 0x30 | | ADPCM frames |

Sample count = (data size / 16) × 28. All 19 use filters 0–4 throughout and contain an end flag. Lengths are about 3–9 s.

These 19 clips are the game's numbered effect clips (the stingers). Clip *n* (0–18) is at resource 256 + n. Which clip an effect index plays is set by design table 14 (see Effect and stinger index).

### Stereo stream (" KPV", resources 146–255)

Header fields are **little-endian** u32:

| Offset | Size | Meaning |
| --- | --- | --- |
| 0x00 | 4 | ASCII ` KPV` (a space, then K, P, V) |
| 0x04 | 4 | *S*: ADPCM bytes per channel |
| 0x08 | 4 | Interleave block size: 2,048 |
| 0x0C | 4 | 0xA000 (not needed) |
| 0x10 | 4 | Sample rate: 22,050 (104 streams) or 44,100 (6) |
| 0x14 | 4 | Channel count: 2 in all 110 |
| 0x18 | | Zero up to 0x800 |

ADPCM data starts at byte 0x800. It is interleaved in 2,048-byte blocks: left, right, left, right… Each block holds 128 frames (3,584 samples) of one channel. A stream lasts about S / 16 × 28 / rate seconds.

Near the end, every stream contains silent padding: frames whose header byte is 0xC0 and whose other 15 bytes are zero (1,228 whole silent blocks and 164 partly silent ones over the 110 streams; no other invalid frames exist). Audible blocks can follow silent ones: in 106 of 110 streams, up to 6 audible blocks lie past 0x800 + 2S. The player stops after S bytes per channel (2S bytes of the interleaved data), so decode only that much (S / 16 frames per channel) and output silence for any frame with header byte 0xC0; the audible blocks that follow on disc are never played (SPEC-0031, KPV silence). Set the prediction history to zero after a silent frame. Trailing silence can be trimmed.

### Emote voice banks (resources 144 and 145)

Confidence: layout, counts and table values High; directory positions High; the only triggers are the emote control records in dialogue text (SPEC-0022, Dialogue control records) (High).

Resource 144 has 996 character emote clips with data, at 1,185 directory positions. Resource 145 has 358 hero emote clips with data, at 367 directory positions. Zero-length positions play nothing. The design-table resource (533, SPEC-0024) holds both directories as path records and names both banks by resource number, so they are not cut content. Their use is described under Emote clip lookup.

- The resources have no index, count or name table. Clip 0 starts at byte 0 of the resource.
- Each clip is a 0x30-byte header followed by its ADPCM frames. The next clip starts at the end of the previous clip, rounded up to a multiple of 2,048 bytes. So every clip starts on a 2,048-byte boundary.
- Synthetic example: a clip with data size 0x3F0 occupies 0x420 bytes, so the next clip starts at 0x800.
- Directory counts: resource 144 has 1,185 positions, 996 with a nonzero length and 189 with length zero. Resource 145 has 367 positions, 358 with a nonzero length. The positions are contiguous, and their lengths add up to the bank size in sectors (10,056 for 144 and 3,327 for 145), so each directory covers its whole bank.
- The directory for resource 145 is the path record in the design-table resource (SPEC-0024). It holds 367 entries of (start sector, length in sectors), starting at +0x104 of the record; the count 367 is the u32 at +0x100. Nine entries have length zero, at positions 9 to 11, 41 to 43 and 103 to 105 (High).
- Every clip, including the last one (position 366 of resource 145), has its own directory entry. A decoder reads each clip from its directory position, and a zero-length position plays nothing.

Clip header (big-endian, the same 0x30-byte shape as the VAG clips above):

| Offset | Size | Meaning |
| --- | --- | --- |
| 0x00 | 4 | ASCII `VAGp` in every clip |
| 0x04 | 4 | Version: 0x20 in 862 character clips; 0 in the other 134 character clips and in all hero clips |
| 0x0C | 4 | ADPCM data size in bytes, always a multiple of 16. Character clips: 2,192 to 134,944 (median 15,280). Hero clips: 3,328 to 83,312 (median 14,128) |
| 0x10 | 4 | Sample rate: 22,050 in every clip |
| 0x20 | 16 | Name, NUL-terminated. Non-empty in 680 character clips; empty in every hero clip and in the other 316 character clips |
| 0x30 | 16 per frame | ADPCM frames, mono, 28 samples per frame (PS-ADPCM frame rules above) |

The clip names are labels for the emote, not for the character. A clip does not record which character it belongs to; the character comes from the index arithmetic in Emote clip lookup.

## Music selection (partly understood)

Confidence: Medium-High for the row fields and how each one sets the bank, stream and sound block. High for the interactive segments: the bank start runs child 0, a setup segment, and the other children are reached through the commands in the music data (SPEC-0031, section 8). High for the level-script override's value and effect and for the places that clear it. High for the volume units and ramps. Medium for the run-time flag's story-state meaning. Medium-High for the area-to-row rule for world packs that carry the type-1001 record; Medium for the fallback. The 21 banks that no known code selects: Medium.

One music row chooses three things at once: a sound bank from the MIDI container of resources 10 to 143 (SPEC-0031, section 8), a stereo stream from resources 146 to 255, and a sound block (SPEC-0031). The stream is therefore selected by the same row as the bank. The game always starts a bank's music at child 0 of the bank's container; which of the other children sound is then decided by the music data and the game's two intensity inputs (see Interactive segments and Intensity inputs).

### Track row and its fields

Each music row is one record of table 4 of resource 533 (SPEC-0024). Table 4 has 194 records of 40 bytes. The controller reads the row whose id is the hero's current music row (a word in the hero's state). Row ids 0 to 192 are looked up. An id of 193 or more uses these defaults: bank offset 1, stream offset -1 (keep), block offset 1, level 1024, scale 1024, cap 0, reverb values 0.

| Field | Record offset | Meaning | Values in the table |
| --- | --- | --- | --- |
| Stream offset | +0x00 | Stream resource = this value + 146, plus 55 with the run-time flag. | 0 to 54; -1 keeps the current stream |
| Bank offset | +0x04 | Bank resource = this value + 10, plus 67 with the flag (so 77 to 143 with the flag). | 0 to 66; -1 keeps; -3600 or -6000 fades out and stops (the value is the fade time in milliseconds) |
| Block offset | +0x08 | Sound block resource = this value + 395. | 0 to 59; -1 keeps; below -1 stops |
| Cap | +0x0C | Stream fade-in ceiling: the stream's volume rises to Cap x Sound-effects gain (see Streams). | 300 to 900 |
| Level | +0x10 | Target volume of the playing bank before the Music gain (see Volume and ramps). | 600 to 750 |
| Scale | +0x14 | Default sound-effect cue volume: each tick the controller sets the sound manager's default cue volume to trunc(Scale x sound-effect gain) (see Default cue volume). Not part of the bank volume rule. | 1024 in every row |
| Reverb pair 1 | +0x18 (effect type), +0x1C (effect return volume) | Reverb of the second SPU core (core mask 2). | Type 3 to 5 (5 is sent as 4); volume 12987 or 15185 |
| Reverb pair 2 | +0x20 (effect type), +0x24 (effect return volume) | Reverb of the first SPU core (core mask 1). | Type 3 to 5; volume 800 to 12987 |

The two reverb pairs are not MIDI channels or layers; see Reverb.

### Per-tick controller

Runs once per game tick (SPEC-0007) while music is active. The order of the work is: bank, reverb, register 2, global excite, streams, sound block.

Pass gate (High): the controller and the per-pass voice update run on a pass when no movie is playing and either the world-enable word is set (play), or the title-start word is set and the game-state code is 9. The title sets the title-start word on its passes, and only the game-globals initialiser clears it. So from the first title pass on, the controller also ticks on every front-end pass (title, options, credits, pause). Before the first title pass (card check, logo, legal screens and attract movies) neither runs.

1. Read the row record for the current music row (defaults above for ids 193 and up).
2. Bank override: if an override is in force, it replaces the table bank offset for this tick (see Overrides).
3. Run-time flag: evaluate it (see Run-time flag). It adds to the table bank and stream offsets only, and it is added to negative offsets as well (see Edge cases). It is not added to an override, and it does not change the block.
4. Bank offset:
   - -1: keep the current bank, stream and block.
   - -3600 or -6000: fade the current music out over that many milliseconds and stop it, including the stream (see Volume and ramps).
   - 0 or more: the target bank is the offset + 10 (plus 67 with the flag). If it differs from the playing bank, ramp the playing bank down and then start the target at child 0 (see Volume and ramps, and Interactive segments).
5. Reverb: send the row's reverb pairs when they change (see Reverb).
6. Intensity: send register 2 and the global excite (see Intensity inputs).
7. Stream offset: if it is not -1, the stream resource is the offset + 146 (plus 55 with the flag), and the two stream channels crossfade to it (see Streams). If it is -1, the current stream is kept.
8. Block offset: if it is 0 or more, the sound block is the offset + 395 and it is queued into the current area slot (SPEC-0031, Sound slots and their blocks). -1 keeps the current block; below -1 unloads it. The default row gives block 396. A block change takes effect at once (no fade).

### Default cue volume and the area slot

Confidence: High.

- Default cue volume: each tick the controller sets the sound manager's default cue volume (the volume a cue call uses when it passes a negative volume, SPEC-0031, Cue playback) to trunc(row Scale x Sound-effects gain). Scale is 1024 in every row and for row ids 193 and up, so the default is trunc(1024 x gain): 1,024 at gain 1.0, 768 at 0.75, 512 at 0.5, 10 at 0.01 and 0 at 0. Before the first controller tick the default is 1,024. Because the controller runs on front-end passes once the title has been shown (Per-tick controller, Pass gate), front-end cues played through the cue call get trunc(Scale x Sound-effects gain) there too; before the first title pass the default is 1,024. 201 of the 204 gameplay cue sites use the default volume. A cue whose volume is below 1 is refused (no voice is taken), so at Sound effects 0 no default-volume cue plays. Front-end menu sounds from the front-end menu sound table use a fixed 1,024 and do not follow the gain.
- Area slot users: the area slot holds the row's block (395 to 454). It is played by the level-script sound calls (PlaySound, LoopSound, AddAnimObjSound, KillSound; small cue ids, 5, 7 and 11 the most common), by the movable-block sound that a level registers by cue index (played with the hero's pushing strain), and by the placed sound emitters of the world pack (SPEC-0031, Placed sound emitters). Nothing else plays on it.

### Interactive segments (which child plays)

Confidence: High. The sequence format, the timing and the command set are in SPEC-0031, section 8.

- Starting a bank always starts child 0 of its container, with start argument 0. Before the new bank starts, any music still playing is unloaded.
- Child 0 is a **setup segment** in all 134 banks. Its first event is one command message that defines the bank's channel groups and macros, sets registers, starts companion children, then jumps to child 1 and ends itself. The rest of child 0 never plays (its bytes after the setup are a stale copy of header data).
- Children 1 and up are reached only through the commands. Every child loops forever: at its end of track it restarts at its own first event. There are no jumps inside a child; changes happen only at command events, which the data places on every beat (every 480 MIDI ticks, one quarter note) in all 93 interactive banks. Children that run together have equal lengths (93 of 93 interactive banks), so they stay aligned.
- The game drives only register 2 of the playing instance and the driver's global excite G (see Intensity inputs). All other registers are set only by the music data.

Templates on disc (child 0 is the setup in all 134 banks):

| Template | Banks | What plays |
| --- | --- | --- |
| Interactive (6 children) | 93: all banks not listed below | Setup clears registers 0, 1 and 2, starts children 2 and 3 and jumps to 1. Children 1, 2 and 3 are three synchronised exploration layers; each beat each one unmutes or mutes its own channels by the range of G. Child 4 is the combat segment, entered when register 2 = 1. Child 5 is a combat accent layer whose channels sound only when register 2 = 2. |
| Plain loop (2 children) | 26: 18, 21, 23, 30, 31, 32, 35, 39, 51, 54, 55, 59, 74, 85, 88, 90, 97, 98, 99, 102, 106, 118, 121, 122, 126, 141 | Setup jumps to 1; child 1 is a looping track with no commands. |
| Two layers (3 children) | 8: 29, 43, 50, 70, 96, 110, 117, 137 | Setup starts 2 and jumps to 1; child 1 sounds for G 0 to 49, child 2 for G 50 to 100. |
| Never-set registers | 7: 34, 101 (2 children); 52, 119 (5 children); 80, 81, 82 (5 children) | 34 and 101: child 1 mutes its channels unless register 3 is 1 to 100, so it is silent. 52 and 119: child 1 would switch to children 2 to 4 by register 6, so only child 1 plays. 80, 81 and 82: children 1, 3 and 4 are three G layers; child 2 is reached only when register 4 = 1. The game never sets registers 3, 4 or 6. |

Interactive template, per beat (all 93 banks; banks 75 and 142 order child 5's commands differently with the same effect):

- Child 1 applies its exploration group. When register 2 = 1 it sets registers 0 and 1 to 1 and jumps to child 4 (child 1 stops). While register 0 = 0 it starts child 3 if child 3 is not playing.
- Children 2 and 3 apply their exploration groups and jump to child 4 when register 0 = 1.
- Child 4 (combat): when register 2 = 0 it clears registers 0 and 1 and jumps back to child 1. While register 1 = 1 it starts child 5 if it is not playing.
- Child 5 (combat accent): applies its group on register 2 (audible only at 2). When register 1 = 0 it starts child 2; when register 2 = 0 it stops itself.

Exploration windows (values of G that unmute each layer's channels), by bank count: 44 banks use 0 to 49, 50 to 69 and 70 to 100 for children 1, 2 and 3; 17 use 0 to 49 and 0 to 69 (some channels each) for child 1, 50 to 100 and 70 to 100; 15 use 0 to 49, 50 to 100 and 70 to 100; the other 17 banks use similar mixes (13 of them keep 70 to 100 for child 3, and 4 use other mixes).

Result: at G 0 only child 1's channels sound; G 50 to 69 adds child 2's; G 70 to 100 adds child 3's. Combat begins on the first child-1 beat with register 2 = 1 and ends (children 1, 2 and 3 restart from their starts) on the first child-4 beat with register 2 = 0.

#### Same-tick order of children

Confidence: High (Medium-High for the first events of a child started inside a tick).

- The driver keeps one list of active sounds. Each start (a game start, or a start or jump command) appends the new child at the end of the list, so the list is in start order.
- Every 1/240 s sequencer tick walks the list from the head. A child started during the walk is appended and is reached in the same tick; if its first delta is 0, its first events run in that tick.
- Interactive template: child 0 starts 2 and 3 and then jumps to 1, so the order is 2, 3, 1. Combat entry gives child 4, and child 4 starts child 5 after itself, so the order is 4, 5. Child 4 therefore always runs before child 5.
- On the combat exit beat, child 4 clears registers 0 and 1 and jumps to 1; child 5 then sees register 1 = 0, starts child 2 and stops itself. The exploration children restart in the order 1, 2, 3.

#### Missed combat (reproduce)

Confidence: Medium-High.

- Child 1's per-beat macros test register 2 = 1 exactly: they set registers 0 and 1 and jump to child 4 only then. Children 1 to 3 have no command that reacts to register 2 = 2.
- Register 2 is 1 on ticks 1 to 43 after the first agro enemy appears (E = trunc(1.15 n), 1 to 49) and 2 from tick 44 (Intensity inputs).
- A fight in which no child-1 beat falls inside those 43 ticks never enters the combat segment while E stays at 50 or more, which holds for the whole fight because the target stays at least 89.75 while any agro enemy is left (Intensity inputs). Child 3's layer plays instead (G = 100).
- The window opens again only when k returns to 0: X is then set to 0 at once, and the next agro enemy starts a new 43-tick window. So a fight with a lull (every enemy dead, or all out of state 19 for a tick) gets a second chance; a continuous fight does not.
- With a uniform beat phase the miss share is max(0, 1 - 0.7167 / beat length in seconds). It is non-zero in the 22 interactive banks whose beat is longer than 0.7167 s (tempo above 716,667 microseconds per quarter: banks 10, 16, 25, 26, 27, 40, 41, 42, 58, 62, 63, 77, 83, 92, 93, 94, 107, 108, 109, 125, 129 and 130). Child-1 tempos on disc in those banks: 750,000 (banks 26, 42, 93, 109; share 0.044), 759,493 (10, 16, 77, 83; 0.056), 779,220 (41, 63, 108, 130; 0.080) and 800,000 (25, 27, 40, 58, 62, 92, 94, 107, 125, 129; 0.104). All other interactive banks have a beat of 0.7167 s or less and never miss.
- World packs whose row uses one of those banks (the same bank pair with the flag off and on): 720, 732, 796, 906 and 908 (banks 16 / 83, share 0.056); 721 (42 / 109, 0.044); 1083 (10 / 77, 0.056); 1090 (40 / 107, 0.104); 1091 (41 / 108, 0.080); 1300 (26 / 93, 0.044); 1324, 1335 and 1344 (27 / 94, 0.104); 1327 (25 / 92, 0.104); 1381, 1382 and 1391 (58 / 125, 0.104); 1392 (62 / 129, 0.104); 1393 to 1396 (63 / 130, 0.080).

#### Fights that never reach the combat segment

Confidence: High for the rows and templates; Medium for which packs hold fights.

The combat segment exists only in the interactive template. A fight in a pack whose row selects any other template, or a stop row, never switches music, whatever the enemies do:

- Plain loop banks (no reaction to register 2 or G): pack 985, Illusen's Glade (banks 31 / 98; the Gnarfas, Werelupe King and Gelert Assassin arenas); pack 989, the Black Knight arena (39 / 106); pack 1096, the Meridell Treasure Vault (23 / 90; Slorg, Giant Slorg and Undead Draik fights); pack 1174, Meridell Castle 1st Floor (30 / 97; any fight there); pack 1217, the Juppie arena (51 / 118); pack 1299, Fyora's Library (55 / 122; the Evil Harris fights); pack 1411, the Hall of Heroes (59 / 126; the gladiator fights); and packs 1323 and 1427 to 1433, Fyora's Tower and the final arenas (30 / 97; Minion Grunt, Minion Warrior, Vorsh and the Darkest Faerie).
- Two-layer bank 70 / 137: packs 1253 to 1256 (Werelupe Sinkholes and Werelupe Woods; Spyder and Slorg fights). A fight there holds G at 100, so child 2 (G 50 to 100) sounds, but no combat segment exists.
- Stop rows (music faded out): packs 797 (Brightvale Outskirts), 799 (Rathbone Family Crypts), 1082 (Meridell Village), 1089 (Ancient Castle Spires) and 1370 (Altador). Fights there are silent of music.
- Pack 989 (Black Knight arena, music row during the fight): the alternate row is stream 221, bank 106, block 417. It switches to the plain row when flag 298 is set just before movie 47.
- Pack 703 (Brightvale Outskirts, selector 6): bank 15 is interactive, but with the run-time flag on the row gives bank 82, whose layers are driven by G only, so a fight there has no combat segment while the flag is on.
- Boss-only fights in an interactive pack: no boss enters state 19 (Intensity inputs), so only class-17 adds start the combat segment.
- A fight that carries on across a bank change: see Transitions.

All other world packs (166 of the 191 with a type-1001 record) reach the combat segment unless the window is missed. Fourteen of them use bank 13, 14 or 15, whose flag-on bank (80, 81 or 82) has no combat segment, but their selector is 0, so the flag is never on there.

### Run-time flag

The flag is on when both of these hold. It is evaluated once per tick.

- Progress bit 216 is set.
- The selector test for the loaded world pack passes. The selector is a number from the pack's type-1001 record (see Area to music row).

Progress bits are a bitmap of 743 bits. Bit n is the n-th bit counting from the least significant bit of the bitmap's first byte. Selector meanings, as flag-number rules:

| Selector | The selector test passes when |
| --- | --- |
| 0 | Never. No progress bit is tested. Used by 58 packs (including pack 532). |
| 1 | Bit 83 is set and bit 5 is clear. |
| 2 | Bit 10 is set and bit 322 is clear. |
| 3 | Bit 64 is set and bit 283 is clear. |
| 4 | Bit 57 is set and bit 58 is clear. |
| 5 | Bit 74 is set and bit 263 is clear. |
| 6 | Bit 85 is set and bit 298 is clear. |
| 7 | Bit 92 is set and bit 94 is clear. |
| 8 | Bit 248 is set (no clear test). |
| 9 | Bit 332 is set and bit 100 is clear. |
| 10 | Bit 332 is set and bit 102 is clear. |
| 11 | Bit 332 is set and bit 104 is clear. |
| 12 | Bit 332 is set and bit 106 is clear. |
| 13 | Always. Not used on disc. |

Selector counts over the 192 world packs on disc: 0 (58 packs), 1 (13), 2 (45), 3 (6), 4 (16), 5 (14), 6 (19), 7 (6), 8 (4), 9 (3), 10 (2), 11 (6). Selectors 12 and 13 do not occur.

When the flag is on, the bank offset gets +67 and the stream offset gets +55. The block is not changed.

Story span of each selector (Medium; the bits are progress flags whose setters are in SPEC-0049 and SPEC-0058). Bit 216 is set in the aftermath of the Gnarfas fight (walkthrough step 20) and is never cleared, so no pack plays the alternate music before that point. After that, a selector's span is from the story moment that sets its first bit until the moment that sets its second bit:

| Selector | Span in which the alternate music plays (progress bit 216 also set) |
| --- | --- |
| 1 | from the start of Act 3 (bit 83) until the Act 3 Juppie fight is won (bit 5) |
| 2 | from the aftermath of the Gnarfas fight (bit 10) until King Skarl is freed (bit 322) |
| 3 | from the Gnarfas defeat (bit 64) until the Gelert Assassin is beaten and Illusen revived (bit 283) |
| 4 | from the start of Act 3 (bit 57) until bit 58 is set by the exit cutscene after the Ixi Chieftain is beaten in the Cogham mines (walkthrough step 37; script trigger 22 of pack 864 with flag 293 on). Stage rows of stages 2 and 6 also set bit 58, so a game begun from start index 3, 7 or 8 never plays the alternate music of this selector (High) |
| 5 | from the start of Act 3 (bit 74) until the Plague Serpent is beaten (bit 263) |
| 6 | from the start of Act 3 (bit 85) until the Black Knight is beaten (bit 298) |
| 7 | from Roberta's night in Fyora's palace in Act 2 (bit 92) until the final fight is won (bit 94) |
| 8 | from the start of Act 3 (bit 248) to the end of the game |
| 9, 10, 11 | from the start of Act 4 (bit 332) until the legend flag 100, 102 or 104 is set (the Altador shrine legends) |

Test vector (synthetic): a pack with selector 5, bits 216 and 74 set and bit 263 clear plays bank offset + 67 and stream offset + 55; when the Plague Serpent is beaten (bit 263 set) the next tick plays the plain row. A pack with selector 8 and bit 216 clear plays the plain row even when bit 248 is set.

### Volume and ramps

Confidence: High; the stinger ramp limit during a bank change Medium.

- Gains: the Music gain is the Music option x 0.01, the Sound-effects gain the Sound effects option x 0.01 and the Voice gain the Voice option x 0.01, each clamped to 0 to 1 (SPEC-0035, Behavior 3). All three are 1.0 from start-up until the options are first applied (the first pass of the Audio page's Music row, or a slot load), not 0.75 (SPEC-0035). The Music gain scales the bank music (through V below). The Sound-effects gain scales the stream ceiling, the stingers and, through the default cue volume (Default cue volume and the area slot), every sound-block cue played at the default volume. The Voice gain has one reader, the emote-clip player (Stinger and emote-clip playback).
- Gain changes: sounds already playing keep the volume they started with; only sounds started afterwards use a new Sound-effects or Voice gain. The bank music moves toward its new target by at most 15 per tick. A lowered stream ceiling applies at once; a raised one is approached at 1 per tick (Streams).
- Target: the bank's target volume is V = row Level x Music gain, or the one-shot level override's value when one is set this tick. V is in 1/1024 units of the bank sound's own volume 127: the volume sent to the driver is min(127, trunc(V x 127 / 1024)).
- Same bank: the volume moves toward V by at most 15 per tick (900 per second).
- Bank change: the outgoing bank's volume falls by 20 per tick (1,200 per second). Once it is at 1 or below, the outgoing bank stops and the new bank starts from child 0, then rises toward its own V by 15 per tick. Example: from 100 the volume goes 80, 60, 40, 20, 0, and the new bank starts (Medium for the exact tick).
- Stop rows (bank offset -3600 or -6000): the volume falls by trunc(V / (|offset| x 0.06)) per tick, so the offset is a fade time in milliseconds at 60 ticks per second. Truncation makes the fade longer than nominal: offset -3600 at V 562 falls 2 per tick and takes 281 ticks instead of 216.
- Stinger ducking: while a stinger or emote-clip voice is still playing (they share one handle), the target is 0.7 x V. During a bank change the ramp limit is then V / 6 per tick (Medium).

### Reverb

Confidence: High for the sends; Medium-High for the effect mode names.

- Reverb pair 1 (row +0x18 type, +0x1C volume) sets the effect of the second SPU core (core mask 2). Reverb pair 2 (row +0x20 type, +0x24 volume) sets the first SPU core (core mask 1).
- The type is clamped to at most 4 (there is no lower clamp). The volume is used when it is 0 to 0x7FFF; 0x8000 and above sends 0. The volume is written as both the left and the right effect return volume. Delay and feedback are 0.
- Type numbers follow the PS2 sound library's effect modes: 0 off, 1 room, 2 studio A, 3 studio B, 4 studio C. On disc the types are 3 to 5 (5 is sent as 4).
- Pair 1 is taken from the row when the row's bank offset is 0 or more; otherwise from the one-shot reverb overrides (third and fourth arguments of OverRideMidiSettings) when set, else the last values are kept. It is sent while music is active and only when a value changed.
- Pair 2 is taken from the row when the row's block offset is 0 or more. It is sent when the sound-block slot is ready and only when a value changed.

### Intensity inputs (excite and adventure levels)

Confidence: High for the configuration, the per-tick rule, the setters, the adventure zones and the agro count; Medium-High for bosses never counting; Medium for the enemy states that interrupt the count.

The game drives exactly two music inputs: **register 2** of the playing instance (combat intensity: 0 none, 1 low, 2 high) and the driver's **global excite G**, 0 to 100 (the adventure level). Both are read by the music data's commands (see Interactive segments).

Configuration, set once at start-up: excite maximum M = 100; adventure base B = 49; combat hold value H = 100; adventure range A = H - B = 51; intensity register 2; no second intensity register. A further start-up value of 70 is stored with them; nothing reads it (see Open questions). The global excite sent to the driver is clamped to 0 to M.

- **Excite level E** (integer 0 to 99). SetMidiExciteLevel(n) stores n only when 0 <= n < 100; other values are ignored. The hero sets it every tick to trunc(X), where X is a float the hero keeps (start 0):
  - k = the number of entries of the active character list (the per-pass list of at most 45 characters, rebuilt every pass from the active character regions; SPEC-0029, Behavior 17) whose AI state word (+0xA34) is exactly 19 (agro) and whose base action state (+0x144) is neither 16 (dead) nor 42 (parked) (High). These two values are base action states, not flags.
  - No distance, line-of-sight or facing test is applied in the count. Distance enters only through how a character reaches state 19 (High for the count; Medium for the paths):
    - A class-17 enemy that notices the hero (the Sense Min to Sense Max band, SPEC-0038) goes to the alerted state 2 (or 21) and enters state 19 when its alert animation has finished; the alert event and the end of the hurt sequence also lead to 19, the hurt sequence only when the hero is within 20 units.
    - SetAI_Agro sets state 2, so a script-alerted enemy also counts once it has passed through the alert step.
  - A character stops counting on every tick it is not in state 19, even while it is still fighting (Medium). The class-17 states that interrupt agro are: 18 (an attack chosen by the map search, not the normal attack, which stays in 19), 14 (the distance hide), 16 (inactive or dead), 22 (retarget) and 20 (a global interrupt that is never set in play, below). A strafing enemy (approach 4) also drops to the default state 0 after more than 120 ticks of its strafe hold, and must notice the hero again before it counts (Medium-Low). State 20 would be entered by every class-17 character not in state 8 or 16 while the game-globals word +0x20 (run-time data 0x35C9A0) is non-zero. Nothing in the game sets that word: it is zero at start-up and the main loop clears it at the end of every pass, and no boot routine, host call or level module writes a non-zero value to it. So state 20 never occurs in play and never ends combat music (High).
  - Who can count (Medium-High): only three paths put a character in AI mode 19: the class-17 common-enemy machine, the alarm-group broadcast and the wave spawner (which puts its spawned characters straight into 19). The boss classes (1, 8, 9 to 16, 18, 32, 37 and 38) run their own state blocks and never enter mode 19. So a boss alone never raises the excite level or starts the combat segment; in a boss fight, combat music comes only from class-17 adds in agro.
  - Target: k = 0 gives 0, except 101 - h when h = HP / (0.15 x max HP) is below 1 and 101 - h is at least 101 (that is, only at HP 0). k = 1 to 3 gives 100. k of 4 or more gives 101 - 0.25 k. Because the list holds at most 45 characters, the target for any k of 1 or more is at least 101 - 0.25 x 45 = 89.75.
  - X moves toward the target by +1.15 per tick upward (not past it) and -1.3125 per tick downward, but a target of 0 or less sets X to 0 at once. So E drops to 0 on the tick that no agro enemy is left.
  - Because 100 is refused, E stops at 98 while X climbs past 99. The level-script call SetMidiExciteLevel(22) in module 735 is overwritten by the hero on the next tick.
- **Adventure counter C.** SetMidiAdventureLevel(f) sets C = trunc(B + A x f) (f = 1.0 gives 100, f = 0 gives 49). The call SetMidiAdventureLevel(1.0) in module 735 therefore sets C = 100. The adventure zones below also call it every tick.
- **Each tick while music is active:**
  - If E >= 1: C = H (100); register 2 = min(127, trunc(E / (M x 0.5)) + 1), which is trunc(E / 50) + 1: 1 for E 1 to 49 and 2 for E 50 to 99; G = 100.
  - If E < 1: register 2 = 0; G = C; then C decreases by 1 if it is above 0. So after combat G stays at 100 and then counts down by 1 per tick (60 per second).
  - Register 2 and G are sent only when they differ from the last values sent. A bank change resets the last-sent values to 0.
- **Adventure zones.** The kind-8 volume records of the pack's type-11 chunk (fourth list; SPEC-0028, section 6) number 2,589. Byte +0x4E is 1 in 209 of them, over 56 world packs, and 0 in the other 2,380. Each tick the zone walk takes the grid cell at the camera position. For each record there with byte +0x4E = 1:
  - Gate: the sound listener (camera position + 5 x camera axis +0x20, SPEC-0031) must lie inside the zone sphere (centre +0x30, outer radius +0x3C) widened by 1.0.
  - Factor: d is the distance from the controlled hero's base position (the camera's focus character) to the centre, not from the listener; inner is the radius at +0x40. f = 1 when d <= inner; f = 1 - (d - inner) / (outer - inner) when inner < d < outer; and f = 1 when d >= outer (quirk). The game then calls SetMidiAdventureLevel(f).
  - So while the camera is still inside a zone but the hero has stepped past its outer radius, the adventure level jumps to 100.
  - Outer radii on disc are 25 to 250 units, inner radii 10 to 80. Records with +0x4E = 0 are positional ambient loops on the area slot (SPEC-0031, Placed sound emitters).
- Example: no enemies, inside a zone's inner radius: C = 100, G = 100, so child 3's layer sounds. After leaving the zone, G falls by 1 per tick: to 70 after 0.5 s and to 49 after about 0.85 s.

### Overrides (level scripts)

- **Bank override**: OverRideMidiSettings' first argument. It persists: it replaces the table bank offset every tick until it is cleared. The value 64 gives bank 74, and the flag adds nothing to it. The dialogue choice that calls OverRideMidiSettings(64, -1, -1, -1) is in modules 1023, 1358 and 758 (High for value and path; Medium for the audible result, which is not confirmed). The other dialogue choice calls it with -1 in the first argument, which clears the override.
- **Level, reverb type and reverb volume overrides** (second to fourth arguments): each applies to the tick it is set on and is cleared at the end of that tick.
- **Pause level and stream gate.** Front-end table B states 5 and 7 (Resume Game focus and the pause page) write all four overrides each pass: level trunc(400 x Music gain), and -1 for the bank and both reverb overrides. Writing -1 to the bank override clears a level-script bank override, so the bank-74 choice ends as soon as the pause page runs. They also close the stream gate (controller +0x68), which skips the stereo-stream step. The gate opens on the first pass of play and closes at start-up, title entry and credits entry (SPEC-0035, Behavior 3b). High.
- **Clears.** All four overrides are set to -1 on every area activation (hero switch, area load and area transition) and when play resumes from these front-end screens: title start, new game, save (done or cancelled), the book reader, the map page, the lead-in pages (never reached), quit (through the area activation that follows), and two more screen exits whose screens are not named (one of them restores the world at the last save area or at area 68). The pause page and the Resume Game focus clear the bank override too (see Pause level and stream gate). The 5-second logo countdown writes no override. So the bank-74 override from modules 1023, 1358 and 758 lasts until the next area activation, pause or front-end return. High for the clears; Medium for the screen names (SPEC-0035).
- **Front-end bank overrides.** Two front-end screens set the bank override on every pass instead of clearing it: the title sets 25 (bank resource 35) on every pass that does not start a game, and the credits set 8 (bank resource 18). Both values are table offsets, so the bank is the value plus 10, and the run-time flag adds nothing to them. The value 8 left by the credits stays in force on the Options page until the title or another writer replaces it (High for the writes; SPEC-0035, Behavior 3a).
- **SetMidiExciteLevel(22)** and **SetMidiAdventureLevel(1.0)**, called unconditionally inside a trigger callback in module 735. They set E and C (see Intensity inputs). They do **not** select a track.

### Streams

Confidence: Medium-High.

- There are two stream channels. The target stream is row +0x00 + 146 (plus 55 with the flag); -1 keeps the playing stream.
- If a channel already plays the target stream, it fades in and the other channel fades out. If neither plays it and a channel is free, the target starts on the free channel at volume 51/1024. The start waits until the bank change has settled, no transition is running, and the stream gate (controller +0x68) is open; while the gate is closed the whole stream step is skipped (see Overrides, Pause level and stream gate).
- Fade in: +1/1024 per tick up to trunc(Cap x Sound-effects gain), where Cap is row +0x0C (300 to 900). When the ceiling is lowered (a lower gain), the volume drops to the new ceiling at once. Fade out: -1/1024 per tick; at 0 the channel stops. A cap of 700 at gain 1.0 is reached after 649 ticks (about 10.8 s).
- A channel that has a volume but is not playing is restarted at its volume.

### Transitions

Confidence: Medium-High; the world-load reset High.

- Area change with the same bank and no synchronous world load: the music continues uninterrupted; the segment, layers and combat state carry over. This covers ordinary overworld travel (the background streaming of the neighbour pack into the transition or exterior buffer) and both interior jobs (entering and leaving an interior by a door or by StartEnterInterior and StartLeaveInterior): none of these loaders stops the sound system (High).
- Area changes that are synchronous world loads (the world-load audio reset below): a door from the exterior or a transition into an exterior that is not resident, both level-script exterior replacement loads (StartExteriorReplacementLoad with its fade and StartExteriorReplacementLoadWithoutFade), the end of a movie, loading a save, and the front-end world (re)entry; also the world teardown after a memory-card job. After each of these the music restarts from child 0 and every sound block reloads (High).
- Test vectors: walking from pack 909 into the resident transition 691: no audio reset; when both rows name the same bank the music, layers and combat state carry over, otherwise the bank change fades as below. Entering an interior by a door: no audio reset; the new pack's row applies on the next controller tick (a bank change fades as below). StartExteriorReplacementLoad from an exterior: full audio reset, music from child 0.
- Area change to a new bank: the old bank fades (20 per tick), the new bank starts at child 0 (so exploration child 1 plays) and rises at 15 per tick. The streams crossfade as in Streams; the sound block switches at once.
- Bank change during a fight (High for the game side; Medium-High for the result, which rests on the music data): staging a new bank clears the per-register cache of the music register setter to 0. The controller calls the setter on every tick, and the setter sends register 2 once the new music voice has its driver handle. A fight that is past its first 43 ticks has E of 50 or more, so the value sent is 2. Child 1 of the new bank enters combat only on register 2 = 1, so the new bank stays in exploration (with G = 100) until E falls to 0 and a new fight starts. If the value 2 reached the driver before the setup segment ran, the setup's clear leaves register 2 at 0 and the cache still holds 2, which gives the same result. The same holds after a synchronous world load during a fight, and when the run-time flag changes the bank mid-fight. A fight that changes bank inside its first 43 ticks sends 1 and can still enter combat on the new bank's next child-1 beat inside the window.
- Boss arenas have their own rows like any area (or a stop row). Combat music comes from the excite path (Intensity inputs), so in a boss fight only class-17 adds in agro start it. Stop rows fade the music out: packs 797, 799, 1082 and 1089 use 3.6 s, and pack 1370 uses 6 s.
- World-load audio reset (High). The flag that decides the reset is the sound system's on flag (set when the sound system starts). The per-frame world loader serves each pending load request with a synchronous load, and its teardown does the following whenever sound is running: resets the music state; stops both stream channels; clears the area slot; resets the music object; clears the creature-block counts; resets both heroes' voice slots; and stops the sound system, unloading all 25 slots and freeing all 48 voices. At the end of the same frame step the sound system restarts with its stored values, the main block 347 is queued again and the area slot is -1. So after every such load the music restarts from child 0 and every sound block reloads.
- Load requests come from: loading a save, the front-end world (re)entry, the end of a movie, a world-changing hero interaction, and two level-script host calls (one is StartExteriorReplacementLoadWithoutFade).

### Where the music runs

The sequencer, the command interpreter, the registers and the global excite live in the sound driver on the I/O processor, which runs the sequencer at 240 ticks per second (4,170 microseconds per tick; SPEC-0031, section 8). The game program only sends commands: start bank sound 0 with a volume, set volume, set register (instance, register 0 to 15, value 0 to 127), set global excite (0 to 127) and set reverb (core mask, type, volume). A clean side may run the sequencer in its own audio thread with the same rules.

Music register setter (High): setting register r of the playing music instance does nothing until the music voice has its driver handle (the driver has acknowledged the start). The value is clamped to 0 to 127 (a negative value becomes 0, a value above 127 becomes 127). The command is sent, and the value cached for that register, only when the clamped value differs from the cached one. Test vectors: a request of 200 is sent as 127; a request of -5 is sent as 0; a request equal to the cached value sends nothing.

### SetMusic host call

SetMusic(t) starts the sound bank whose resource is t + 10, with the loop flag set. It does not use table 4. It is never called (High): none of the 173 level modules on the disc lists SetMusic among its imported host calls, and the boot code has no call to it; its only reference is its entry in the host table. The bank-start routine it wraps has no other caller. So every bank that plays is chosen by table 4 (and the override value 64), never by a track number.

### Area to music row

A world pack's music row comes from the pack's own record. The area key is only a fallback.

Where the data lives:

- **Pack record**: each world pack (the game archive's world resources, numbers 532 to 1433, 192 packs) has one chunk of type 1001 except pack 532. The chunk's payload (after the chunk header and its extra bytes) starts with a 16-byte child header (magic IDMX). The pack record follows at payload +0x10: the key is the 32-bit word at record +0 (payload +0x10; only the low 16 bits are used) and the selector is the 32-bit word at record +8 (payload +0x18), 0 to 11 on disc. Both descriptions name the same word (High).
- **Default record**: at the start of every world load the record is reset to a default record in the boot executable at virtual address 0x2DA3D0 (key -1, selector 0). A pack without a type-1001 chunk keeps this default, so its selector is 0 (High).
- **Value table**: 193 slots of 32-bit values at virtual address 0x31F2B0 in the data of the boot executable (the game's ELF). A data address in the ELF is converted to a file offset with the program header of the segment that holds it: file offset = virtual address minus the segment's virtual address plus the segment's file offset (for this segment, virtual address - 0x100000 + 0x80). All values are distinct; the largest is 11070 (High).
- **Area key table**: 193 slots of 32-bit values at virtual address 0x31EFA8 in the same boot executable (it ends 4 bytes before the value table). Its values are the pack numbers 678 to 1433: 191 distinct numbers, two of which appear twice. Both table scans run 193 slots at a 4-byte stride (High).

How to derive the row:

1. Chunk key to row: the row is the index of the first value-table slot whose 32-bit value equals the key. If no slot matches, the row is 193 (the default row). The key matches for all 191 packs that have a chunk.
   - Example: pack 678 has key 11. Key 11 is found in slot 5, so its row is 5.
   - Example: pack 1077 gives row 109; pack 1432 gives row 191.
   - Example: pack 735 (key 207, selector 5) gives row 22, whose table 4 entry is stream 2, bank 2, block 2. That gives bank 79, stream 203 and block 397 while bit 74 is set and bit 263 clear, and bank 12, stream 148 after.
   - Rows 187 and 191 are identical (stream 173, bank 30, block kept, selector 0); pack 1428 uses row 187 and pack 1432 uses row 191.
2. The chunk row replaces any row that the area key set earlier in the same load. Ten packs have a chunk row that differs from their area-key row, for example pack 678 (area-key row 0, chunk row 5).
3. Area-key fallback (only used when the pack has no chunk, which on disc is pack 532 only): the area id is looked up in the area key table. The row is the index k of the first slot holding that id. An id that is not in the table gives row 193. Pack 532 is not in the table, so its row is 193 (default: bank offset 1, stream kept, block 1). Its selector is 0, so the flag is never set for it.

### Banks not selected by known code (Medium-High)

Sound banks are resources 10 to 143. Known code reaches them as follows:

- Flag off (bank = table offset + 10): 10 to 76, except the 9 banks listed below. Bank 74 (offset 64) is reached only by the level-script bank override. Banks 35 (offset 25) and 18 (offset 8) are reached only by the front-end bank overrides of the title and the credits (Overrides; High for the writes). The controller ticks on those screens once the title has been shown (Per-tick controller, Pass gate), so both banks are heard (High).
- Flag on (bank = table offset + 77): 77 to 143, except the 12 banks listed below.
- Banks 11 and 78 are reached by the default row (offset 1).

Twenty-one banks are not reached by any known code: 21, 29, 32, 34, 43, 50, 52, 54, 64, 85, 88, 96, 99, 101, 102, 110, 117, 119, 121, 131 and 141. They are the banks of the twelve table offsets 8, 11, 19, 22, 24, 25, 33, 40, 42, 44, 54 and 64, which no world pack's rows use (the plain banks are offset + 10 and the flagged banks are offset + 77), less banks 18 and 35, which the front end reaches. The flagged banks of offsets 8 and 25 (85 and 102) stay unreached, because the run-time flag is not added to an override.

By template (see Interactive segments), these 21 banks are mostly non-interactive: 9 plain loops (21, 32, 54, 85, 88, 99, 102, 121, 141), 6 two-layer banks (29, 43, 50, 96, 110, 117), 4 that depend on never-set registers (34, 101, 52, 119) and 2 interactive banks (64, 131) (High for the templates). Banks 18 and 35 are plain loops.

Confidence is Medium-High. No known code selects them. Two of the three routes once left open are closed: SetMusic and the track selector for tracks 163 to 166 are never called (High; see SetMusic host call and Track selector). The remaining possible route, the area-key step run for the second world slot at a save load, is closed too: the pack's own row replaces its result (see Area to music row).

The music sequences themselves do not provide another bank-selection route: none of the 529 playable children contains a MIDI bank-select controller, and their embedded commands operate on children, channel groups and per-instance registers. This rules out a selected bank's sequence switching playback to one of these banks; the area-key row path cannot select one either.

### Track selector for music requests (never runs)

Confidence: Medium for the recorded behavior of the routine; High that it never runs. The boot code has no call to it and holds no pointer to it (no table entry, no code-pointer word), and level modules can reach boot code only through named host calls, none of which leads to it. The rules below are kept only as a record; a clean side need not implement them, and tracks 163 to 166 are never selected in play.

The routine handles tracks 163 to 166. Its inputs are a request record. Tracks are chosen by the request's state word. The selector is separate from the per-tick controller, which takes its value from the table; the two paths never meet, because the selector never runs.

Inputs:

- State word: the integer at +8 of the request record. The requesting trigger fills it; the routine never runs, so its meaning is moot.
- Cue value: the integer at +0xC of the request record. Zero means no cue.
- Hero index: a runtime value in the hero object at +0x14C. 0 is Tor and 1 is Roberta.

Track choice. The ranges do not overlap, so the order of the rows does not change the result:

| Hero | State range | Track (decimal) |
| --- | --- | --- |
| Tor (0) | 0x62 to 0x69 | 0xA4 (164) |
| Roberta (1) | 0x6F to 0x76 | 0xA4 (164) |
| Tor (0) | 0x6A to 0x6E | 0xA6 (166) |
| Tor (0) | 0x77 to 0x7D | 0xA5 (165) |
| Roberta (1) | 0x7F to 0x86 | 0xA5 (165) |
| Any other | Any other | 0xA3 (163), the default |

Steps:

1. Both per-track flags on the playback handle (a field of the hero object at +0x168) are set to 0. The pair 0.945 and 10.0 is sent to a helper at the start, before the track is chosen. The routine never runs, so the roles of these arguments have no effect. The handle value is used here and released at the end.
2. The track is chosen from the table above.
3. The start depends on the track:
   - 0xA4, 0xA5 and 0xA6 (the non-default rows): for 0xA4 and 0xA6 both per-track flags are set to 1; 0xA5 leaves them at 0. The release helper runs on the handle value. Then a volume call with volume 1.0 runs with sub-id 8 (for 0xA4) or 6 (for 0xA5 and 0xA6). The track is then started by start helper A with first argument 0.
   - Default (163) with cue value zero: the track is started by start helper A with first argument 0. No volume call is made.
   - Default with cue value nonzero (the cue path): a volume call with volume 1.0 runs with sub-id 7. The track is then started by start helper B with first argument 1.0.
4. Cue name. If the cue value is nonzero, for any track, a request is sent with the cue value, a name and a runtime index, through a helper with a zero second argument. The name is the hero name Tor when the hero index is 0 and Roberta otherwise. The index is a runtime value that is 4 higher for Roberta.
5. Wait. Let T0 be the timer reading taken after step 4. The loop runs while the timer is below T0 + 2.0, or while the cue query returns nonzero. Each pass calls start helper B with first argument 0 for the track, reads the timer again, and yields one tick. The loop ends when the timer is at least T0 + 2.0 and the cue query returns zero. The timer's unit and the cue query's exact meaning do not matter because the routine never runs (a clean build omits it).
6. End. A stop call runs with arguments minus one and one. Then the handle is released, a clear step runs, and a final step runs. Their roles have no effect because the routine never runs.

### Effect and stinger index

A script effect call takes an index k. The index selects a clip from design table 14 of resource 533 (SPEC-0024): 91 records of two bytes, each a weight and a class. Classes 0 to 18 each have exactly one entry, with weight 100 (High). So index k (0 to 18) plays resource 256 + k with certainty.

Indices 19 and above are rejected: the effect routine checks the index first and plays nothing (High for the bound). Calls with the fixed indices 33, 37, 42, 57, 61 and 66 therefore play nothing.

Classes 19 to 65 of table 14 hold multiple records, but the stinger caller rejects indices 19 and above before calling the chooser. For reachable classes 0 to 18, each class has one record with weight 100, so class k always plays resource 256 + k. The chooser's multi-record behavior is defective: it repeatedly reads the first record's weight and never advances to later records. The table's class-start values are also offset after class 18. These defects affect no reachable stinger index (High).

Stinger and emote-clip playback (High; the ducking test Medium-High):

- Stingers and emote clips share one playback handle, so only one of them plays at a time: an emote clip cannot start while a stinger or another emote clip is still playing, and the other way round.
- Stinger 4 does not start again within 40 s of its last start. The last-start time begins at clock 0, so no class-4 stinger plays in the first 40 s of the game clock. A group-alarm wake of an enemy whose cue word is 0 is also a caller of stinger 4 (SPEC-0038, Behavior 3), rate-limited to once per 40 s.
- The stinger volume is trunc(trunc(f x 1024) x Sound-effects gain), with the caller's factor f = 1.0 at every known call.
- The emote-clip player (Emote clip lookup) plays a clip at trunc(trunc(f x 1024) x Voice gain), with f = 1.0 at both known call sites (1,024 at gain 1.0, 768 at 0.75, 409 at 0.4). It plays only while the bank music is active.
- Hero voice cues (blocks 348 and 349) are not emote clips. They are sound-block cues at the default cue volume, so they follow the Sound-effects gain and not the Voice gain (SPEC-0031, Hero voice slot uses).
- A stinger or emote clip whose computed volume is 0 or below is not started, and the music is not ducked for it.
- While a stinger or emote-clip voice is still playing, the bank music's target volume is 70% of normal (see Volume and ramps).

## Player audio slots (not part of the sound system)

Moved: no per-player audio slot exists; the routines once listed here are described in SPEC-0008 (Movie audio internals, Not player audio slots), and the game's own sound playback is the sound slots, cue calls and 48-voice pool of SPEC-0031 (High).

## World-item records (not sound instances) (High)

The list of 0x170-byte records that this spec once described as a sound instance list is the world-item pool of SPEC-0021 (World items, Pool and record): 50 slots of 0x170 bytes per loaded world. It plays no sound and stops no sound. SPEC-0021 owns its rules; this section only maps the old wording so that no clean-side audio code is built for it (High).

| Offset | Old wording here | Meaning (SPEC-0021) |
| --- | --- | --- |
| 0x50 | scale word, 1.0 at construction and at reset | life: the draw alpha, 1.0 at spawn; it falls by 2.12 per second once the item has outlived its lifetime |
| 0x54 | attached sub-record holding a 16-bit sound id at +0x0C | the item's config record; +0x0C is the item id |
| 0x60 to 0x8F | embedded volume block, active byte at 0x80, 16-bit volume at 0x8A | copy of the placement's data; the 16-bit value at 0x8A is the buyable price |
| 0x94 | stop timer, positive infinity at construction | lifetime L in seconds |
| 0x98 | cleared at reset | age in seconds |
| 0xB0 | owner id | the id that the find-by-id lookup matches |
| 0xC0 onward (0xFC, 0x100, 0x101, 0x108, 0x140) | priority start block | an embedded object that the spawn sets up with the item id; not part of the sound system (Medium) |

Corrected rules (High):

- Lifetime writers. Construction sets L to positive infinity. The spawn sets L: L = Larg when Larg is 0 or more; otherwise L is infinite for a placed item, 15.2 s for a charm (config type 17) and 11.2 s for any other item (SPEC-0021, World items, Spawning, step 5). No other routine writes L. Drops by enemies, breakables and clover patches pass Larg = -1, so their L is finite; every placed item and every script AddItemToWorld call on the disc gives an infinite L.
- The "stop" on reset is the important-item give: when L is finite and a config record is attached, an item whose text row flag +0x1F is 1 is added to the party inventory with count 1 (SPEC-0021, Important-item give). The call goes to the party inventory, not to the sound manager. With an infinite L nothing is given and nothing is left playing.
- The "volume scaling" is the script price multiplier ItemSetPriceMultiplier: every shop slot (flag byte 1) gets price p' = trunc(k x p), capped at 32767 (SPEC-0021, Shop purchase rule).

Test vectors (from SPEC-0021 rules):
- An enemy drop of item 153 (Larg -1, not placed): L = 11.2 s. A drop of Mote 164 (charm): L = 15.2 s. Item 153 placed at region load: L infinite, so a reset gives nothing.
- A dropped Black Feather (item 78, important) that expires: the party inventory gains 1 Black Feather; no sound call is made.
- Price multiplier 0.5 on a slot with price 1000 gives 500; 2.0 on 20000 gives 32767.

### Timed queue of 7-word entries (High)

A separate queue holds entries of seven 32-bit words each (28 bytes). Its header holds the entry count at offset 0 and a serial counter at offset 8. Entries start at byte offset 12. Entry fields, from the start of an entry: +0 value; +4 type byte; +8 duration; +12 and +16 two parameters; +20 valid flag; +24 serial number.

- **Append** at index e: the fields are stored and the valid flag is set to 1. The serial number is the counter, which then increases by 1. The count increases by 1. If e is above 0, the duration of entry e − 1 is reduced by the new entry's duration. No bound check is made on e.
- **Remove** entry k when k is below the count: if k is above 0, the duration of entry k − 1 is increased by the duration of entry k. Entries after k move down one place. The count drops by one, and the slot just past the new last entry is cleared (valid flag 0, parameter at +12 set to -1). Removing the last entry follows the same rule with no move. An index at or above the count does nothing.

The queue is the enemy group alert board of SPEC-0029 (High). It holds at most 40 entries. A duration is a delta in ticks: the last entry of the array is the soonest to expire and counts down; each earlier entry's duration is its life minus the lives of all later entries. Posting an entry with life p walks from the end of the array toward the front: while p is not below the duration of the entry just before the slot, p is reduced by that duration and that entry moves one place toward the end; the new entry is stored where the walk stops, and its predecessor's duration is reduced by p by the append rule above. The post refuses (returns 0) when 40 entries are already held. Each post also steps the board's change counter (header word at offset 4) by 1 modulo 100. Each tick the last entry's duration drops by 1; an entry whose duration is below 2 is removed instead, and its duration (1) is added to the entry before it, so the entry before it lives one tick longer than a plain countdown.

Test vectors: a board holding A (life 100) receives B with life 30: B is stored after A, B has duration 30 and A has duration 70. After 29 ticks B has duration 1; on the next tick B is removed and A has duration 71. Posting C with life 130 to a board holding only A (100): C is not below A's duration, so p becomes 30, A moves to the end of the array and C is stored first with duration 30; A (100) expires first, then C 30 ticks later.

### Owner of the timed queue

The timed queue is the group alert board (SPEC-0029, Behavior on the shared alert board); it is not part of the world-item pool and plays no sound. Its entries are posted when an enemy enters the alert mode and its spawn record has a group byte.

## Behavior

- The original streams the KPV streams from the disc while playing, rather than loading them whole. The clean side may load them whole; the largest is about 12 MB.
- Volume: the sound code uses 1,024 for 1.0 when it stores volumes.
- A music row's bank offset is a bank choice (offset plus 10 as a resource number, plus 67 with the run-time flag). Music played in normal play is set by the per-tick controller from table 4, and by level-script overrides (see Music selection).
- The start argument of a bank start is zero in this build.

### Emote clip lookup

Confidence: directory and roll rule High; hero shift High; character stride and voice sets High; playback rules High. The random source is not described here. Emote requests come only from dialogue control records in line text (SPEC-0022, Dialogue control records); no other game event requests an emote clip (High).

The emote tables are in resource 533: table 7 (character emotes, 148 rows) and table 8 (hero emotes, 193 rows), SPEC-0024. The row number is the emote id, and the emote id is also an animation id of the actor: table 7 has one row per entry of the 148-entry main and face animation tables of a townsfolk character, and table 8 one row per entry of the 193-entry hero action tables. A row gives that animation an optional voice clip. Each row is 3 bytes:

| Offset | Size | Meaning |
| --- | --- | --- |
| +0 | 1 | Clip index. 255 means no clip |
| +1 | 1 | Chance in percent. Values 0 and 100 only |
| +2 | 1 | Variance byte. Zero means no step; nonzero means a random step. Only zero or nonzero is tested |

When: the lookup, with its roll and step, runs when the request runs, that is when the text reveal reaches the control record (SPEC-0022), not when the clip later plays. Its result (directory, position, emote id) is queued for the actor (SPEC-0040, Behavior 22).

Steps for one lookup:

1. Row check: if the emote id is negative or at or past the row count (148 for table 7, 193 for table 8), no row is read, no draw is made and the entry carries no clip.
2. Roll: draw r from 0 to 99, one draw for each valid row. The roll passes only if r is below the row's chance. The chance is 0 or 100 in every row, so a chance of 100 always passes and a chance of 0 never does.
3. If the roll fails, the entry carries no clip and no further draw is made. A row that fails the roll or has no clip still queues its entry, so the gesture plays, silently.
4. If the roll passes and the variance byte is nonzero, draw s from 0 to 2 and set the index to the clip index plus s minus 1 (so -1, 0 or +1). If the variance byte is zero, the index is the row's clip index and no draw is made.
5. An index of 255 in the row means no clip. A computed index of -1 also means no clip.
6. Playback position: the index is a position in the bank's directory (SPEC-0024 path record). Every position counts, including those of zero length. A zero-length position is read and plays nothing. The directory bound is tested as an unsigned comparison, so a negative position counts as past the end and plays nothing.
7. Character emote (request kind 0x10, on the selected hero's talk partner): position = index + 74 x c. The voice set c is the signed 16-bit value at +0x14 of the partner's villager row (resource 531, SPEC-0024). The row is chosen at spawn: the first-row index of the placement's species t (placement data +0xB4) plus its variant (placement data +0xB2), at a row stride of 0x60. On disc c is -1 (no voice set) for 2,396 of the 3,036 placements and 0 to 15 for the other 640. 16 voice sets x 74 + 1 = 1,185 directory positions. Set 7 has no clips (positions 518 to 591), and its only user is one record in a pack that no route loads. Table 7 has zero variance in every row, so no step is drawn.
8. Hero body emote (request kind 0x17, SPEC-0022, on the selected hero): position = index + step + hero shift. The hero shift is 183 when the selected hero is Roberta and 0 for Tor. The shift is the hero directory's entry count shifted right by one: 367 >> 1 = 183. Hero face emote (request kind 0x18): position = index + step, with no hero shift for either hero.
9. Hero index ranges: indices 4 to 178 with a step of -1 to +1 give body-emote positions 3 to 179 for Tor and 186 to 362 for Roberta. The two ranges do not overlap. Face emotes use positions 3 to 179 for both heroes.

Table values (counts from the disc):

- Table 7: 148 rows. 59 always play (chance 100, clip index 0 to 74, variance zero, so no step). 89 never play (chance 0): 77 with no clip and 12 with a clip index. Character lookups never apply a step.
- Table 8: 193 rows. 175 rows have no clip and never play. 18 rows always play (chance 100, clip index 4 to 178, nonzero variance byte, so each takes the random step). The always-on row with clip index 10 is row 124 (variance 13); row 10 has no clip. The variance byte equals the clip index plus 3 in all 18 always-on rows, but the lookup tests only zero or nonzero.
- Disc check of the animation link: the hero body emote ids 182 to 191 have a main animation for both heroes and no face animation; the hero face emote ids 122 to 132 have a face animation for both heroes and no main animation; townsfolk face tables have animations at the face ids 126 and 127.

### Emote clip playback

Confidence: High.

- The emote consumer starts the animation itself: a body emote becomes the actor's main animation id, and a face emote becomes its face override id (SPEC-0040, Behavior 22). It then starts the entry's clip, if the entry has one. An entry with no clip (row index 255, chance 0, or a failed roll) still plays its gesture, silently.
- The clip starts only at that moment: when the consumer starts the body animation, or when it sets a new face emote. A clip refused at that moment is not retried.
- A clip is refused when the entry has no directory, when its position is negative or past the directory (the unsigned bound of step 6), when the position's length is zero, when the shared stinger and emote-clip handle is busy, when the bank music is not active, or when its volume (Stinger and emote-clip playback) is 0 or below.
- With the unsigned bound, a talk partner with c = -1 is silent for every clip index below 74: the position index - 74 is negative. Clip index 74 (table 7 row 98) with c = -1 gives position 0, which has data (4 sectors) and plays. Clip index 74 with c = 15 gives position 1,184, which has length 0 (silent).
- Body and face on the same tick: if a new body emote and a new face emote both have clips on the same tick, only the face clip plays.
- Lines carry no spoken dialogue. The only audio a dialogue line can start is these emote clips, short gesture vocalisations: about 1.1 to 1.2 s at the median, 10.7 s at most in the character bank and 6.6 s at most in the hero bank (SPEC-0022).

### Emote clip survey

Confidence: High for the bank counts; Medium for the character reach in play.

- Hero bank (resource 145): 69 of the 358 positions with data can be reached. Tor body positions 3 to 5, 44 to 46, 88 to 90, 142 to 144, 156 to 158, 162 to 164 and 168 to 170 (21); Roberta body the same plus 183 (21); face rows 122 to 132, 30 positions with data, the same for both heroes, 3 of them shared with Tor's body set. The other 289 are never reached (126 in Tor's half, 163 in Roberta's).
- Character bank (resource 144): 873 of the 996 positions with data can be reached for some voice set; 123 never are. They are offsets 1 to 7, 44, 51 and 69 to 73 of the 74-position blocks: no always-on row of table 7 has these clip indices.
- In play: counting only lines named by a live call in reached packs (SPEC-0022, Line use) and the voice sets placed in each pack gives at most 788 character positions; matching line speaker names to placements gives 523 (Open questions).

Test vectors for this section are under Test vectors.

## Edge cases and error handling

- World-item records: their edge cases (infinite lifetime, double give, price cap) are in SPEC-0021; none of them affects sound (High).
- Timed queue: append does not check the index against the capacity. Removing an index at or above the count does nothing (High).
- Resources 144 and 145 hold VAG-headed mono clips in the layout under Emote voice banks. They decode with the PS-ADPCM frame rules above. Resolved: the original plays them through the shared stream path with the entry length as a sector count (SPEC-0024, Open questions 14 and 23); the rewrite decodes the VAG-headed clip with the PS-ADPCM rules above and plays it as a one-shot (Medium).
- Hero emote for Roberta with a row clip index of 178 and a +1 step: position 362, the largest reachable hero position.
- Zero-length positions 9 to 11 of the hero directory are reached by the face request of row 124 (SPEC-0022, kind 0x18), and play nothing. Positions 41 to 43 and 103 to 105 are not reached by any row of table 8.
- The hero directory lookup accepts a position equal to its entry count (367), one past the last entry, and then reads bytes outside the directory. No reachable input gives that position.
- Character emote with no voice set (c = -1): the position is the clip index minus 74. Below 74 it is negative and the unsigned bound rejects it, so the clip is silent and the gesture still plays. Clip index 74 gives position 0 and plays the first clip of the bank (High).
- Body and face clips due on the same tick: only the face clip plays; the body clip is not retried later (High).
- A VAG with a filter index above 4 in any frame: stop decoding at that frame.
- Table 4 value of -1: keep the current music. Values of -3600 and -6000: stop the music. No record on the disc holds any other value (SPEC-0024, Table 4), so the behavior for values outside 0 to 66 and these three never arises; a clean build may treat any other value as keep.
- The bank override is not reset by the per-tick controller, but it is cleared on every area activation, on every pass of the pause page and the Resume Game focus, and on the front-end returns listed under Overrides. A dialogue choice that does not call the override with -1 leaves the bank choice in force only until the next of those (High). The level, reverb type and reverb volume overrides are one-shot and are cleared at the end of each tick.
- Reverb type values above 4 (table value 5) are clamped to 4, so such a row selects the same effect as value 4. There is no lower clamp. An effect return volume of 0x8000 or more is sent as 0 (High).
- Adventure zone, hero past the outer radius while the listener is still inside the gate: the factor is 1, not 0, so C jumps to 100 (High). Reproduce.
- Missed combat: a fight whose first 43 ticks contain no child-1 beat never enters the combat segment until every agro enemy has left state 19 and a new window opens (Interactive segments, Missed combat) (Medium-High). It can happen only in the 22 packs on slow banks listed there, at 4.4 % to 10.4 % of fight starts.
- Fights in 25 world packs never reach the combat segment at all: plain-loop, two-layer and stop rows, and pack 703 with the flag on (Interactive segments, Fights that never reach the combat segment) (High for the rows).
- A bank change or a synchronous world load during a fight that is past its first 43 ticks leaves the new bank in exploration until the fight ends (Transitions) (Medium-High).
- Agro count with no distance test: an agro enemy far away in an active region still holds the combat music, and an enemy fighting from a state other than 19 does not count (Intensity inputs) (High for the rule).
- Bosses alone never drive the excite level, because no boss class enters AI mode 19 (Intensity inputs) (Medium-High).
- Every synchronous world load restarts the whole audio system, so the music restarts from child 0 even when the new area uses the same bank (Transitions) (High).
- The adventure level is not a divisor: register 2 is trunc(E / 50) + 1 with the fixed excite maximum 100, so no input divides by zero (High).
- SetMidiExciteLevel with a value outside 0 to 99 is ignored, so E never reaches 100; the hero's excite float can climb past 99 while E stays at 98 (High).
- The run-time flag's offsets are added to negative row values too: with the flag a stop row's bank offset is -3533 or -5933 (a slightly shorter fade), and a stream offset of -1 would become 54 (stream 200). The default row is the only row with stream -1, and its pack never sets the flag, so stream 200 is not reached on disc (Medium-High).
- Stop-row fade: the per-tick fall is truncated, so the fade is longer than the nominal time (281 ticks instead of 216 for -3600 at V 562), and a V small enough to give a fall of 0 never reaches 0 (Medium for the last point).
- Pack 532 has no type-1001 record, so it keeps the default record (key -1, selector 0) that every world load sets first, and the run-time flag is never set for it. Its area-key step reads an area id that the previous world load left in place (Medium).
- An effect index of 19 or more: plays nothing, and is not an error (High for the bound).
- Hero index other than 0 or 1: it matches no hero range, so the default track is used. The cue name is Tor only for index 0; any other index gets Roberta (Medium).
- A state word in the range of the other hero (for example 0x6F with Tor) does not match: Tor's ranges apply only to Tor, and Roberta's only to Roberta (High).
- A cue value on track 164, 165 or 166: the track is chosen as in the table and the cue name is still sent, but the volume call with sub-id 7 is not made (Medium).
- No cap on the wait: while the cue query returns nonzero, the loop keeps yielding ticks (Medium).

## Test vectors

Synthetic:

- A VAG frame `00 00 11 11 …` (filter 0, shift 0): every sample nibble is 1, so every output sample is (1 << 12) >> 0 = 4,096 (16-bit PCM). With shift 12 the same nibbles give 1. Use any public PS-ADPCM reference decoder to cross-check.
- Header: VAG size 0x13540 (79,168 bytes) at 22,050 Hz gives 4,948 frames, 138,544 samples and 6.28 s.
- Stream: S = 1,114,144 at 22,050 Hz gives about 1,114,144 / 16 × 28 / 22,050 = 88.4 s per channel. The left channel's first block is file bytes 0x800–0xFFF and the right channel's first block is 0x1000–0x17FF.
- Table 4 value 3 with the run-time flag clear starts the bank in resource 13. With the flag set it starts resource 80 (3 + 67 + 10).
- Table 4 value -1 keeps the current music. Value -3600 stops it.
- Override value 64 starts resource 74 whether or not the run-time flag is set (the override is not offset by the flag).
- World pack 678 (type-1001 key 11, which is row 5; row bank offset 18, stream offset 14, block offset 16). Flag clear: bank resource 28, stream resource 160, block 411. Flag set by selector 1 (progress bit 216 set, bit 83 set, bit 5 clear): bank resource 95, stream resource 215, block 411 (the block is unchanged by the flag).
- Default row (id 193, used by pack 532): bank offset 1, which gives bank 11 (the flag cannot be set for this pack, so bank 78 is not reached), stream kept, block 396.
- Intensity: E = 22 gives register 2 = 1 and G = 100. E = 49 gives 1; E = 50 gives 2; E = 99 gives 2. E = 0 gives register 2 = 0 and G = C. SetMidiExciteLevel(100) is ignored.
- Adventure: f = 1.0 gives C = 100; f = 0.5 gives C = 74; f = 0 gives C = 49.
- Adventure zone (synthetic): outer radius 150, inner radius 50, hero 100 units from the centre: f = 0.5 and C = 74, so with no enemies G = 74 and child 3's window (70 to 100) is open. The same zone with the hero 160 units from the centre and the listener inside the gate: f = 1 and C = 100 (quirk).
- Same-tick order (synthetic): combat entry starts child 4, which starts child 5 in the same tick; child 5's first events (first delta 0) run in that tick, after child 4's.
- Missed combat (synthetic): tempo 833,333 (beat 0.8333 s) gives a miss share of 1 - 0.7167 / 0.8333 = 0.14; tempo 759,493 gives 0.056; tempo 600,000 gives 0.
- Hero excite ramp (synthetic): from X = 0 with 2 agro enemies, X after n ticks is 1.15 n until it reaches 100 (tick 87). E = 1 at tick 1, and register 2 changes to 2 at tick 44 (E = 50). When the last agro enemy leaves state 19, X and E are 0 on that tick.
- After combat (synthetic): E falls to 0; G stays 100 on that tick and then counts down 99, 98, ... by 1 per tick.
- Agro count (synthetic): three class-17 enemies in the list, in states 19, 19 and 18: k = 2, target 100. The same with the second one in base action state 16 (dead): k = 1, target 100. One enemy in state 19 and in base action state 42: k = 0, target 0, so X and E are 0.
- Target floor (synthetic): k = 45 gives 101 - 11.25 = 89.75, so E stays at 89 or more and register 2 at 2 for the whole fight.
- Miss share (from the disc): bank 16 (tempo 759,493, beat 0.7595 s) gives 1 - 0.7167 / 0.7595 = 0.056; bank 58 (tempo 800,000) gives 0.104; bank 41 (tempo 779,220) gives 0.080; bank 26 (tempo 750,000) gives 0.044.
- Bank change mid-fight (synthetic): E = 80 when the hero crosses into a pack with a new interactive bank: the new bank starts at child 0, register 2 is sent as 2 once the voice has its handle, and child 1 never jumps to child 4. When the last enemy dies, register 2 becomes 0; the next fight starts a new 43-tick window with register 2 = 1. The same crossing at E = 20 (tick 18 of the fight) sends 1 and enters combat on the first child-1 beat before tick 44.
- No combat segment (from the disc): pack 1096 uses row bank offset 13 (bank 23, plain loop), so a Slorg fight there changes no music; pack 1253 uses bank 70 (two layers), so a Spyder fight there opens child 2's layer (G = 100) and no combat segment.
- Ramp (synthetic): an outgoing volume of 100 falls to 80, 60, 40, 20 and 0 on successive ticks; once it is at 1 or below the new bank starts from child 0.
- Target volume (synthetic): row Level 700 and Music gain 0.75 give V = 525, sent as trunc(525 x 127 / 1024) = 65.
- Stop row: offset -3600 and V = 562 give a fall of trunc(562 / 216) = 2 per tick.
- Stream: Cap 700 and Sound-effects gain 0.75 give a ceiling of 525; from 51 the fade-in takes 474 ticks.
- Reverb: row pair 1 of (5, 15185) sends type 4, volume 15185 to core mask 2; (3, 40000) sends type 3, volume 0.
- Stinger ducking (synthetic): V = 600 with a stinger playing gives a target of 420.
- Gains (synthetic): all gains 0.75: default cue volume 768, emote clip 768, stinger 768. Sound effects 0: default cue volume 0, so no default-volume cue, stinger or stream rise; front-end menu sounds still play at 1,024. Voice 40: emote clip volume trunc(1024 x 0.4) = 409. Fresh boot (gains 1.0): default cue volume 1,024; row Level 700 gives V = 700, sent to the driver as 86.
- Pause level (synthetic): Music gain 0.75 gives a level override of trunc(400 x 0.75) = 300; with a stinger or emote clip playing, 210.
- Timed queue, synthetic: append entry A (duration 5), then entry B (duration 3). A now has duration 2. Remove B (the last entry) and A has duration 5 again.
- Effect index 0: plays resource 256. Effect index 5: plays resource 261 (High). Effect index 19: plays nothing.
- Track selector (synthetic; the routine never runs in play): state 0x65 with Tor and cue 0 gives track 164. The same state with Roberta gives track 163 with no volume call.
- Track selector (synthetic): state 0x6F gives 164 with Roberta and 163 with Tor. State 0x6A with Tor gives 166, and 0x6F with Tor gives 163 (0x6F is outside 0x6A to 0x6E).
- Track selector (synthetic): state 0x7D with Tor gives 165, and 0x7E with Tor gives 163. State 0x86 with Roberta gives 165, and 0x87 with Roberta gives 163.
- Track selector (synthetic): Tor, state 0x81, cue 5 gives track 163. The volume call with sub-id 7 runs, the track starts with first argument 1.0, and a cue request with value 5 and the name Tor is sent.
- Track selector (synthetic): hero index 2 with state 0x65 gives track 163.
- Wait (synthetic, timer values arbitrary): T0 is 10.0. A timer of 11.0 with the cue query at zero continues the loop. A timer of 12.0 with the query at zero ends it. A timer of 12.2 with the query nonzero continues it. A timer of 12.5 with the query at zero ends it.
- Emote lookup (synthetic): character emote, row clip index 5, character index 2 gives position 153 (5 + 74 x 2). Hero body emote, row clip index 10, step -1, Roberta gives position 192 (10 - 1 + 183). The same row for Tor with no step gives position 10. Hero face emote, the same row with step -1 and Roberta, gives position 9 (no shift).
- Roll (synthetic): a row with chance 100 and a nonzero variance byte always passes the roll (0 to 99), and then the second draw (0 to 2) sets the step. A row with chance 0 never passes, and no second draw is made.
- Zero-length positions (synthetic): face request of row 124 (clip index 10, variance 13) with a step of -1, 0 or +1 gives positions 9, 10 or 11. Those positions have zero length, so nothing plays.
- Hero range (synthetic): Tor with index 4 and step -1 gives position 3. Roberta with index 178 and step +1 gives position 362.
- Emote chance (synthetic): a row with chance 0 gives no clip for any roll, including 0. A row with chance 100 plays for roll 99. A row with clip index 255 gives no clip.
- Hero emote (synthetic): row clip index 178, step +1, Roberta gives position 362 (see Edge cases).
- Character emote (from the disc): a talk partner of species 33 (Hubert, voice set 9), kind 0x10 with emote id 51 (clip index 36): position 702, 8 sectors, plays.
- The same request with an enemy as the partner (c = -1): position -38, rejected by the bound, silent; the gesture (main animation id 51) still plays.
- Emote id 98 (clip index 74) with c = -1: position 0, plays. The same id with c = 15: position 1,184, length 0, silent.

### Test vectors: a real clip decoded, and real clip extents (disc)

- Resource 258 (the "Buy Item" stinger): the header is the text VAGp, version 32, ADPCM data size 43,552 (big-endian at 0x0C), sample rate 22,050. That is 43,552 / 16 = 2,722 frames, 76,216 samples and 3.4565 s.
- Frame 0 (filter 0, shift 0, flags 0): all 28 nibbles are 0, so 28 zero samples.
- Frame 1 (filter 1, shift 11, flags 0). Nibbles in order: 0, 0, 0, 0, 0, 15, 13, 14, 0, 1, 3, 1, 0, 4, 4, 5, 6, 5, 4, 0, 2, 1, 3, 1, 0, 3, 11, 14. With the sample = ((signed nibble x 4096) >> 11) + ((60 x previous + 32) >> 6), the 28 output samples are 0, 0, 0, 0, 0, -2, -8, -11, -10, -7, -1, 1, 1, 9, 16, 25, 35, 43, 48, 45, 46, 45, 48, 47, 44, 47, 34, 28. (Sample 5: nibble 15 = -1, (-4096 >> 11) = -2. Sample 6: nibble 13 = -3 gives -6, plus (60 x -2 + 32) >> 6 = -2, so -8.)
- Frame 2 (filter 2, shift 9) carries the last two samples of frame 1 (28 and then 34 before it) into its predictor. Its first eight samples are 23, 11, 9, 15, 4, 19, 23, 50 (sample 0: nibble 0, (115 x 28 - 52 x 34 + 32) >> 6 = 1484 >> 6 = 23).
- The same frames decoded with no +32 rounding term differ: frame 1 ends 31, 25 instead of 34, 28, and frame 2 starts 19, 5, 1, 5. The sound chip uses the rounded form (see the hardware rule under the PS-ADPCM frame), so the rounded figures above are the reference values.
- Emote bank, resource 144 (clip headers as in the VAG table; every clip's own data size is at 0x0C, big-endian): clip 0 starts at 0 with data size 7,232, so it ends at 48 + 7,232 = 7,280 and clip 1 starts at 8,192 (sector 4). Clip 1 has size 21,984, ends at 30,224 and clip 2 starts at 30,720 (sector 15). Clip 2 has size 11,728, ends at 42,496 and clip 3 starts at 43,008 (sector 21). Clip 3 has size 13,168, ends at 56,224 and clip 4 starts at 57,344 (sector 28). The first four clips have empty names.

- Rounding form, in one line: the hardware rule above is the +32 form. The rounded figures in the lines above are the reference values for resource 258; the unrounded figures are given only to show the size of the difference.
- A second real clip with a heavy filter: resource 256 (the first stinger). Its header has the text VAGp, ADPCM data size 79,264 (big-endian at 0x0C) and sample rate 22,050: 4,954 frames, 138,712 samples, 6.29 s.
- Frame 22 of resource 256 (counting from 0; it starts 48 + 16 x 22 = 400 bytes into the resource): filter 4 (pair 122, -60), shift 5, flags 0. Decoding from frame 0 with zero history and the +32 form, the two samples that enter the frame are h1 = -621 and h2 = -526. The 28 nibbles in order are 1, 4, 13, 12, 15, 15, 1, 5, 0, 3, 12, 2, 12, 12, 13, 0, 4, 15, 4, 7, 14, 12, 14, 14, 12, 2, 4, 4.
- Sample 0: nibble 1, (1 x 4096) >> 5 = 128. Prediction (122 x -621 - 60 x -526 + 32) >> 6 = (-75,762 + 31,560 + 32) >> 6 = -44,170 >> 6 = -691 (the shift rounds toward minus infinity: -690.16 becomes -691). Output 128 - 691 = -563.
- Sample 1: nibble 4, (4 x 4096) >> 5 = 512. Prediction (122 x -563 - 60 x -621 + 32) >> 6 = (-68,686 + 37,260 + 32) >> 6 = -31,394 >> 6 = -491 (-490.5 becomes -491). Output 512 - 491 = 21.
- All 28 outputs with the +32 form: -563, 21, 184, -181, -646, -1190, -1535, -1170, -791, -27, 178, 621, 505, -132, -1109, -1990, -2242, -2536, -2220, -958, -1, 384, 477, 293, -401, -783, -605, 93. The next frame starts from h1 = 93 and h2 = -605.
- The same frame and the same entering history with no rounding term: -563, 20, 181, -186, -653, -1199, -1546, -1183, -806, -44, 159, 600, 482, -156, -1134, -2016, -2268, -2562, -2246, -984, -27, 359, 453, 270, -422, -802, -622, 78. Sample 0 agrees; sample 1 is 20 instead of 21. The gap grows by about two units per sample to 26 at samples 15 to 20 and falls to 15 at the last sample (78 instead of 93).

## Open questions

- Answered: sound banks and sound blocks are specified in SPEC-0031.
- Resolved: the sound chip decodes the frames, so no game routine resets history; a silent frame outputs zeros and the rewrite sets the history to zero (SPEC-0031, KPV silence; rewrite choice). Header word 0x0C (0xA000) is compared by the sound library with its own stream buffer size, and a stream whose word differs is refused; it is not otherwise used (Medium).
- Sound instance list, classification. Resolved: the 0x170-byte records are the world-item pool of SPEC-0021, not sounds; the sub-record id is an item id and the "stop" call is the important-item give to the party inventory (World-item records) (High).
- Sound instance list, timer. Resolved: the word at 0x94 is the world item's lifetime. The only writers are the construction (positive infinity) and the spawn, which stores the caller's lifetime or the default (infinite when placed, 15.2 s for a charm, 11.2 s otherwise). A reset never stops a sound (World-item records) (High).
- Sound instance list, fields. Resolved: 0x50 is the item's life (draw alpha), 0x8A its buyable price, 0x98 its age; the block from 0xC0 is an embedded object set up with the item id, not a sound start (World-item records) (High; Medium for the embedded object). The word at 0x04 is not used by any audio rule.
- Timed queue: Resolved. It is the enemy group alert board of SPEC-0029; a duration is a delta in ticks against the entries after it (Timed queue of 7-word entries; High).
- Resolved: Music, children 1 and up. Child 0 is a setup segment that starts companion children and jumps to child 1; the command messages in the music data switch the others (Interactive segments; SPEC-0031, section 8) (High).
- Resolved: Music, channel numbers. The row "layer pairs" are reverb settings for the two SPU cores, not MIDI channels (Reverb; Intensity inputs) (High).
- Resolved: Music, stream gate. The further controller input is the stream gate (controller +0x68); while closed the stereo-stream step is skipped (Overrides; SPEC-0035, Behavior 3b) (High).
- Resolved: Music, banks 18 and 35. The title (bank 35) and credits (bank 18) bank overrides select them, and both are heard (Overrides; Per-tick controller, Pass gate) (High).
- Resolved: Music, target level units. V = row Level x Music gain, in 1/1024 of volume 127; driver volume = min(127, trunc(V x 127 / 1024)) (Volume and ramps) (High).
- Resolved: Music, cap. Row Cap is the stream fade-in ceiling, multiplied by the Sound-effects gain (Streams) (High).
- Resolved: Music, row Scale (+0x14). It sets the default cue volume each tick to trunc(Scale x Sound-effects gain) (Default cue volume and the area slot) (High).
- Resolved: Default cue volume in the front end. Front-end cues get trunc(Scale x Sound-effects gain) once the title has been shown, and 1,024 before (Default cue volume and the area slot; Per-tick controller, Pass gate) (High). This also answers SPEC-0035 Open question 5b.
- Resolved: Movie audio. The movie player applies no option gain: its tracks start at unity level and the start-up path has no caller that changes them (SPEC-0008, Movie audio internals). Default if unresolved (rewrite choice): play movie audio at full level regardless of the Music, Sound-effects and Voice sliders (needs a runtime check for any master gain in the sound processor).
- Resolved: Music, order of children in one sequencer tick. One active list in start order is walked each 1/240 s tick; child 4 always runs before child 5 (Interactive segments, Same-tick order of children) (High).
- Resolved: Music, missed combat. A fight whose 43-tick window holds no child-1 beat never enters combat (Interactive segments, Missed combat) (Medium-High). Reproduce.
- Resolved: Music, register 2 at a bank change. Either order gives the same result: a fight past its first 43 ticks sends 2, which child 1 ignores, and a value that arrives before the setup is cleared and not sent again. So the new bank stays in exploration until the fight ends (Transitions) (Medium-High).
- Music, global interrupt. Resolved: nothing sets the game-globals word +0x20 (run-time data 0x35C9A0). It starts at zero, the main loop clears it every pass, and no boot routine, host call or level module stores a non-zero value. State 20 never occurs, so this path never ends combat music (Intensity inputs) (High).
- Resolved: Music, listener of the adventure-zone walk. The gate uses the sound listener and the cell at the camera; the factor uses the controlled hero's position (Intensity inputs) (High).
- Music, reset flag on world load. Resolved: the flag is the sound system's on flag, so every synchronous world load restarts the whole audio system. Streamed overworld travel and the interior jobs are not synchronous loads and keep the music; exterior replacement loads (script or door), movie ends, save loads and front-end entry restart it (Transitions) (High).
- Resolved: Music, boss combat. No boss class enters AI mode 19, so bosses alone never drive the excite level (Intensity inputs) (Medium-High).
- Resolved: Music, start-up value 70. The value is kept in the configuration object's word after the adventure base, and nothing reads it: the hero's excite step reads only the maximum and the hold value, the adventure-level setter reads only the base and the range, and the configuration getter has no caller. No code reaches the object by a fixed address either (Medium-High that no indirect read exists). Test vector: replacing 70 with any other value changes no excite or adventure value on any tick.
- Resolved: Music, selector location. The selector at record +8 is the word at type-1001 payload +0x18; both descriptions name the same word (see Area to music row).
- Music, flag for packs without a selector. Resolved: selector 0 never passes the selector test, whatever the progress bits are, and the flag is computed only from progress bit 216 and the selector of the active pack record. That record is either the default record (selector 0) or the pack's own record, and nothing stores into it while the world runs. So no route sets the flag on the default row: a pack that uses the default record always plays the flag-off bank of that row (High). The same holds for the 57 other packs whose record has selector 0.
- Resolved: music, record count: table 4 has 194 records but ids 0 to 192 are looked up. The controller reads a row only for ids 0 to 192; id 193 and above take the built-in default values, and the area lookup gives 193 as its "no match" row, so record 193 of the table is never read (High). Test vector: a pack whose key matches no slot gets row 193 and plays with bank offset 1, stream kept and block offset 1, whatever record 193 holds.
- Resolved: Music, table addresses. Value table 0x31F2B0, area key table 0x31EFA8 (193 slots each), default record 0x2DA3D0 (see Area to music row).
- Resolved (Medium): music, story meaning of the selector bits. Each selector names a story span, from the event that sets its first bit to the event that sets its second (Run-time flag, Story span of each selector).
- Music, second-hero path. Resolved: there is no hero-specific route. The area-key step is run by every world start (with the pack's area id) and twice by the save-load restore for the two world slots, and each run is followed by that world's pack load, which resets the record and then applies the pack's own row from its type-1001 chunk. The area-key row is therefore replaced for every pack that has a chunk (all but pack 532), so the step cannot select one of the 21 unreached banks (Medium-High).
- Resolved: Music, excite counter. It is the adventure counter C, reloaded to 100 on every tick with E >= 1 and counting down by 1 otherwise (Intensity inputs) (High).
- Resolved: Music, adventure level 0. The level is not a divisor; SetMidiAdventureLevel(0) gives C = 49 (Intensity inputs) (High).
- Music, track selector and SetMusic. Resolved: neither is ever called (SetMusic host call; Track selector). No level module imports SetMusic, and the boot code has no call to either routine and no pointer to the selector. The 21 banks in Banks not selected by known code are therefore never played by these paths (High).
- Resolved: Event-to-clip mappings for effects. The index of each event is held in the callers' own data: the progress-flag cue table (SPEC-0024, Table 2), the item showcase (SPEC-0021) and the script effect calls (SPEC-0030); only indices 0 to 18 ever play (Effect and stinger index; Medium-High).
- Resolved: Stinger chooser, reachable classes. Classes 0–18 each have one record with weight 100, so class k always maps to resource 256 + k. The stinger caller rejects indices 19 and above. Multi-record chooser defects and the offset class starts affect only rejected classes (Effect and stinger index) (High).
- Table 14 classes 19 to 65: any reader of these classes outside the stinger chooser (none is known). Answered: callers do compute indices of 19 or more (SPEC-0021 treasure-map add and item showcase, hero gameplay voice events), and the bound drops every one, so none plays (Edge cases; High; the showcase player is SPEC-0021 Open question 132).
- Resolved: Track selector internals (the fade pair 0.945 and 10.0, the units of the 2.0 wait, the cue query, the per-track flags and handle values). The selector never runs in any play path (see "Track selector reach" below), so none of these affects play; a clean build omits the routine (High that it never runs).
- Track selector reach. Resolved: the selector never runs, so tracks 163 to 166 are never reached in play (Track selector) (High).
- Track selector and per-tick controller. Resolved: they are separate paths and only the per-tick controller runs (Track selector) (High).
- Resolved: Emote triggers. The emote control records sit only in dialogue line text and run when the line reveal reaches them (SPEC-0022); idle, hit, pickup, bark and greeting voices do not exist (SPEC-0031). Per line record there are 8,998 / 923 / 293 / 10 requests of kinds 0x10 / 0x17 / 0x18 / 0x16, of which 2,861 / 358 / 131 / 7 are in lines named by a live call in reached packs (High).
- Resolved: Hero record for body requests. Kinds 0x17 and 0x18 and the Roberta shift use the selected (controlled) hero (High).
- Resolved: Random source. The roll and step draws use the gameplay generator, one Mersenne Twister shared with all gameplay draws (SPEC-0007; SPEC-0008, Two random sources); they do not use the sound driver's own generator (SPEC-0031).
- Resolved: Emote character stride. The stride is 74 over voice sets 0 to 15 (16 x 74 + 1 = 1,185 directory positions); offsets 44, 45 and 70 to 73 are empty in 15 of the 16 blocks, set 7 (positions 518 to 591) holds no clip, and clip index 74 of set c is clip index 0 of set c + 1 (High).
- Resolved: Emote playback format. The stream path is given the clip's sector range and plays the VAG-headed clip as stored; the rewrite decodes it with the PS-ADPCM rules (Edge cases; Medium).
- Resolved: Emote character index. It is the voice set c of the talk partner's villager row, -1 or 0 to 15 (Emote clip lookup, step 7) (High).
- Emote partner per line: which conversation, and so which talk partner and voice set, each kind-0x10 line belongs to. The reachable character positions in play (523 to 788, Emote clip survey) depend on it. Speaker names often differ from placement names, for example "Papa Hubert" against the placement "Hubert" (Medium). This changes only the survey count: the rewrite follows the play-time rule (the talk partner's voice set), so the count does not affect behavior.


