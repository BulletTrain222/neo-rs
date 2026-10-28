# SPEC-0031: Sound banks and sound blocks

- Answers: R-0002 (open audio families); R-0021 (game-side decisions answered; driver mixing out of scope); R-0022 (answered: material field, landing trigger and SBlk tick unit; Open question 47 resolved); R-0111 (animated-object sound call arguments, registration gates and requested cue payload sequences for packs 758, 909, 1081, 1247, 1249, 1381 and 1398)
- Status: Draft
- Disc build: SCUS-97367 v1.00 (same fingerprint as SPEC-0001). Counts below apply to this build.
- Confidence: per section. Handle stop with retries: Medium (Low on the exact retry count). Bank header, directory and voice-record offsets, sample extents, loop and one-shot classes, and the KPV silence rules: High. Bank playback path (bank select, program change, note-on): Medium; the bank number 10, the pan conversion, the pitch bend, the note value, the voice ranges and the voice allocator rule are High. Voice tuning bytes (+0x02, +0x03), the pitch rule, the volume and pan formulas and the envelope register fields: High (section "Driver volume, pan and pitch"). Voice envelope words: written to the SPU2 envelope registers at every note-on (High). Clip sets (resources 144 and 145): layout High; use High (they are the emote clip banks, SPEC-0009). Hero voice uses (event table, dropped stingers, no cooldowns, PlayHeroSound gate): High. Absence of barks, greetings and hero animation sounds: High. Cue playback call (its checks, arguments, the 25 bank slots, the 48-record voice pool, the default volume, the radii): High; the keep-alive start value and step High, its exact stop tick Medium. Slot map (main, hero voice, weapon, shield, power, area and creature blocks): High. Packed sound words in animation tables: High. Positional volume and pan: High for the formulas; the camera-matrix direction open. Placed sound emitters: kind 8 and kind 1 High. Animated-object sounds (AddAnimObjSound): High for the rules, Medium-High for the effects of the quirks. Cue instance limit (cue entry byte +5): High for the rule, Medium for the compared volume field. Main sound slot holding sound block 347 and the cue ids that index its cue entries: High for the queue; cue meanings Medium (debug cue names plus call sites). Cue use table: High for the ids and their routines, Medium or Low for the uses as marked. Save-point cue 0xA8 (call, 15-unit gate, keep-alive): High; emitter quirk effect: Low. Sound-block layout and counts: High; command meanings: per type in section 7 (High to Low); volume and pitch: High (Driver volume, pan and pitch); tick unit: 1/240 second (Medium; Open question 47, resolved). Music track selection: see SPEC-0009. Music sequences (section 8): child header, event stream, command messages, timing and looping High (all 529 playable children parse exactly to their end of track). The routines once listed here as the voice list, the mixer and stream pump conversions, the crossfade, the stereo gain split and the stream chunk size are movie-library code (SPEC-0008, Movie audio internals); the channel level ramp is the screen-effect object (SPEC-0052); the channel value setter is the music register setter (SPEC-0009) (High).

## Summary

Sound banks are the disc's main sample library. Each bank (archive resources 10 to 143, 134 banks) holds a small directory, a table of voice records, a pool of PS-ADPCM samples (the format in SPEC-0009), and a trailing MMID container that holds the bank's music: MIDI-like sequences whose embedded command messages start, stop and switch the container's children (section 8). Across all banks there are 20,718 samples. With this spec the clean side can locate every sample, extract its bytes, decide whether it is a one-shot or a loop, and play bank sounds through the MIDI instrument path: bank select picks the bank, program change picks a directory group, and a note starts the group's voices whose key windows contain it. The counts across the 134 banks are 10,790 directory records, 28,860 voice records and 20,718 samples. Sound blocks (SBlk, resources 347 to 490, 144 blocks) carry their own cue table, command list, voice table and sample pool. Gameplay code plays sounds through a cue call on one of 25 bank slots: the main slot holds sound block 347 from start, the hero's voice, weapon, shield and power slots hold blocks chosen from the hero and the equipment, the area slot holds the block of the current music row, and creature blocks are loaded while characters that use them are in the world. The cue id indexes the slot block's cue entries, each of which starts a short command program. The cue call, the slot map, positional volume and pan, the driver's volume, pan and pitch formulas and the cue-use table are in the Behavior section. Sound selection is by index, not by name: a debug cue-name table in the boot executable labels every cue, but playback never reads it.

## Where it lives on the disc

- Sound banks: archive resources 10 to 143. Each starts with a 0x20-byte file header, then a tagged section at file offset 0x20, then the sample pool, then a tail header and the MMID container.
- Sound blocks: archive resources 347 to 490.
- Resources 144 and 145 are sets of short voice clips with the same clip header as the sound-effect clips in resources 256 to 274 (SPEC-0009). Their layout is in Layout section 9. Resources 256 to 274 are the sound-effect clips (SPEC-0009).
- Stereo streams (" KPV", resources 146 to 255) are SPEC-0009.
- Packed sound words: word B of every entry of a type-36 tag-2 animation table (SPEC-0005, word B; layout in Layout section 10).
- Boot-executable data (file offset = VA - 0x100000 + 0x80):
  - Debug cue-name table at data 0x2DF708: 144 pointers, one per sound block 347 to 490 (index = block - 347). Each points to an array of name-string pointers, one per cue entry of that block; name i labels cue id i. Block 347's array is at data 0x2DD8F0 with 192 entries. Debug only: started voices keep the name for logging, and playback never reads it (High).
  - A parallel table of u32 name counts at data 0x2DF948, passed with each block queue. Its values are shifted 24 entries relative to the blocks (an original data quirk; it only bounds the debug name lookup, so it has no audible effect) (High).
  - Packed-sound group tables (Layout section 10): block bases at data 0x31A7C8 (347, 350, 395, 455, 461, 469, 477) and block-index bases at data 0x31A7E8 (0, 3, 48, 108, 114, 122, 130) (High).

All offsets below are byte offsets from the start of the named structure, unless marked "file" (from the start of the resource).

## Layout

### 1. Bank file header (file offsets, little-endian u32)

| Offset | Size | Meaning | Observed on disc | Confidence |
| --- | --- | --- | --- | --- |
| 0x00 | u32 | 1 | 134 of 134 | High |
| 0x04 | u32 | 3 | 134 of 134 | High |
| 0x08 | u32 | 0x20 (offset of the section tag) | 134 of 134 | High |
| 0x0C | u32 | A: section length | A = B - 0x20 in 134 of 134 | High |
| 0x10 | u32 | B: file offset of the sample pool (bank 10: 0x1710) | | High |
| 0x14 | u32 | C: sample pool length in bytes (bank 10: 0xC3080) | B + C = D in 134 of 134 | High |
| 0x18 | u32 | D: file offset of the tail header (bank 10: 0xC4790) | | High |
| 0x1C | u32 | E: read after the pool; meaning unknown | tail header holds E - 0x10 | Medium |

The section (A bytes) starts at file offset 0x20, not at file offset 0.

Tail header at D: u32 2, u32 1, u32 0x10, then u32 (E - 0x10). The MMID container starts at D + 0x10 (134 of 134 banks).

### 2. Bank section (base = file 0x20; all section offsets below are relative to that base)

| Section offset | Size | Meaning | Confidence |
| --- | --- | --- | --- |
| +0x00 | 4 | ASCII "SBv2" | High |
| +0x04 | u32 | 2 | Medium |
| +0x08 | u32 | 4 | Medium |
| +0x0C | 4 | bank name, ASCII (for example CAVE, BSVL, BSSW, WWOO, XROD) | High |
| +0x10 | u32 | MIDI bank number: bank select (MIDI controller 0) picks the first loaded bank whose value equals it | 10 in 134 of 134 | High (value), Medium (role) |
| +0x14 | u16 | count of object records; the one record is at section +0x34 (section 2a) | 1 in 134 | High |
| +0x16 | u16 | n2: number of directory records (134 banks: 10,790 in total) | High |
| +0x18 | u16 | n1: number of voice records (28,860 in total) | High |
| +0x1A | u16 | unknown; 113 to 197 (31 distinct values); not read on the note path | Low |
| +0x1C | u32 | pointer to the bank's single 28-byte object record at section +0x34 (section 2a) | Medium (role) |
| +0x20 | u32 | w8: section offset of the directory | High |
| +0x24 | u32 | w9: section offset of the voice table | High |
| +0x28 | u32 | bias 0x5040 (the same value in every bank) that the loader adds to each voice's pool offset at load (section 4, +0x10) | High |
| +0x2C | u32 | C (same as the file header) | High |
| +0x30 | u32 | 0 | Medium |
| +0x34 | 28 bytes | the bank's object record (section 2a); the record ends at 0x50, where the directory starts | High |
| +0x38 | 4 | bank name again | Medium |
| +0x3C | 4 | bank name again | Medium |

Two rules hold for all 134 banks: the voice table ends exactly at the section length (w9 + 24 x n1 = A), and the directory ends exactly where the voice table starts (w8 + 8 x n2 = w9).

### 2a. Object record (28 bytes at section +0x34, one per bank)

| Offset | Size | Meaning | Values (134 banks) | Confidence |
| --- | --- | --- | --- | --- |
| +0x00 | u8 | object type | 5 in 134 of 134 | High |
| +0x04 | 4 | bank name (the same name as in section +0x0C) | equal in 134 of 134 | High |
| +0x10 | u16 | default volume | 127 (130 banks), 120 (4 banks) | Medium |
| +0x12 | u8 | 0 (read together with +0x13 as one u16 it gives 256) | 0 in 134 | High (value) |
| +0x13 | u8 | voice-allocator range of the music: range 1, voices 24 to 47 (Driver volume, pan and pitch, Voice ranges) | 1 in 134 | High |
| +0x14 | u16 | default argument | 0 (130 banks), 64 (4 banks) | Low |
| +0x17 | u8 | flags | 2 in 134 | Low |
| +0x18 | u32 | runtime pointer, zero on disc | 0 in 134 | High (on disc), Medium (role) |

The record is reached through the bank's +0x1C pointer. It is the bank's music sound: a multi-sequence sound whose sequences are the children of the MMID container (section 8). The game starts the music as bank sound 0, which starts child 0 (High). The runtime pointer at +0x18 is zero on disc; where it is filled at run time is not described (Open question 5).

### 3. Directory record (8 bytes, n2 records at w8)

| Offset | Size | Meaning | Confidence |
| --- | --- | --- | --- |
| +0 | u8 | voice count of the group (0 to 32); 0 means an empty group | High |
| +1 | u8 | group (instrument) volume, 65 to 127; 100 in about 70%, 127 in about 20% | Medium-High |
| +2 | u16 | group pan angle in degrees, added to the channel pan on the volume step (Driver volume, pan and pitch); 0 in 9,516 records, nonzero in 1,274 (10, 15, 25, 330, 335, 340) | High |
| +4 | u32 | section offset of the group's first voice record (w9 + 24 x k), or the sentinel 0x310 | High |

Of the 10,790 directory records, 5,346 non-empty groups point exactly to a voice-record boundary. The 5,242 empty groups (count 0) have pointers that do not land on a boundary; these pointers are never read. 202 records use the sentinel 0x310: 200 of them are empty groups, and 2 (banks 24 and 91) have count 9 (see Edge cases and Open question 4).

### 4. Voice record (24 bytes, n1 records at w9)

Offsets are within the record. Record k starts at section offset w9 + 24 x k.

| Offset | Size | Meaning | Observed on disc | Confidence |
| --- | --- | --- | --- | --- |
| +0x00 | u8 | voice priority, passed to the voice allocator at note-on | 0 in 28,624 voices; 8 (144); 1 (60); 7 (32) | High (priority); the allocator rule is under Driver volume, pan and pitch (Voice allocator) |
| +0x01 | u8 | voice volume (D on the volume step, squared curve) | 100 in about 80% | High |
| +0x02 | u8 | centre (root) note = absolute value of the byte read as signed, which is 256 minus the byte when its top bit is set | top bit set in all 28,860 voices; byte values 136 to 224 (centre 32 to 120) | High |
| +0x03 | u8 | centre fine tune, in 1/128 semitone, added in the pitch rule | 0 to 126 | High |
| +0x04 | i16 | voice pan angle in degrees, added on the volume step; +0x05 is its high byte (255 for negative angles) | 0 in 21,766; 231 in 1,368; 10 in 1,002; +0x05: 0 in 24,880, 255 in 3,978 | High |
| +0x06 | i8 | key window low (inclusive) | 0 in 2,192; 36 to 71 typical in zoned banks | High |
| +0x07 | i8 | key window high (inclusive) | 127 in 2,114; 55 to 76 typical in zoned banks | High |
| +0x08 | i8 | bend range down, in semitones. It scales a negative pitch bend (Pitch in Driver volume, pan and pitch); it is read at every note-on and at every pitch-bend event of the channel, exactly as the sound-block voice byte +8 is | 0 in 28,760; 2 in 70; 1 in 20; 12 in 8; 244 (read as signed, -12) in 2 | High |
| +0x09 | i8 | bend range up, in semitones. It scales a positive pitch bend | 0 in 28,762; 2 in 78; 1 in 20 | High |
| +0x0A | u16 | SPU2 ADSR1 register value, written unchanged at every note-on (fields below). Its high byte is the byte at +0x0B | 0x80FF 20,396; 0x80F0 2,560; 0x3CF0 1,422; 0xBCF0 778; 0xADFF 648; 40 distinct | High |
| +0x0C | u16 | SPU2 ADSR2 register value, written unchanged at every note-on (fields below) | 0x9FCC 9,675; 0x9FCD 5,873; 0x9FC5 4,008; 0x9FCE 2,428; 0x9FC0 to 0x9FFF for 96%; 44 distinct | High |
| +0x0E | u16 | flag word. Bit 0 = effect (reverb) send on, both sides. Bit 0x10 = voice removed from the dry mix. Bit 0x20 = allocator lock (Voice allocator). Bits 1 and 2 select priority modes. Bits 0x10, 0x20, 1 and 2 are never set on disc | 1 in 25,034; 0 in 3,826 | High (bits 0 and 0x10), Medium (others) |
| +0x10 | u32 | sample pool offset, relative to B. The loader adds 0x5040 (section 2, +0x28); the sum is the sample's address for playback. The extent rule in section 5 uses the stored value | | High |
| +0x14 | u32 | sample id (per bank: 0 up to about 0x9B) | 113 to 197 samples per bank | High (not used for playback) |

Tuning bytes: the byte at +0x02 is read as a signed byte and its absolute value is the centre note (256 minus the byte, since its top bit is set in every bank voice), in the range 32 to 120. The byte at +0x03 is the centre fine tune, 0 to 126, added (not subtracted) in the pitch rule. A centre byte without the top bit would also scale the pitch by 44100/48000 (no bank voice has one). The pitch rule is in Behavior, Driver volume, pan and pitch. The 16-bit value formed by the two bytes is not a sample rate. Zone test: the centre note lies inside the voice's own key window for only 774 of 27,012 voices with a restricted window, so the centre note and the key window are separate concepts in the data. Confidence High.

Envelope words: each note-on writes +0x0A to the voice's SPU2 ADSR1 register and +0x0C to its ADSR2 register, unchanged (High). Fields (the public SPU2 register layout): ADSR1 bit 15 = attack exponential, bits 8 to 14 = attack rate, bits 4 to 7 = decay rate, bits 0 to 3 = sustain level; ADSR2 bit 15 = sustain exponential, bit 14 = sustain decreasing, bits 6 to 12 = sustain rate, bit 5 = release exponential, bits 0 to 4 = release rate. The envelope timing is the public SPU2 envelope rule; nothing in this game changes it. Examples: 0x80FF = exponential attack, attack rate 0, decay rate 15, sustain level 15; 0x9FCC = exponential sustain, increasing, sustain rate 127, linear release, release rate 12.

Voice records that share a sample id can differ in their envelope words and tuning bytes. For example, two voice records of the same sample in different directory groups of bank 10 differ only in the first envelope word (0x80FF and 0xADFF). Which records play together is open (Open question 7).

### 5. Sample pool and sample extents

The pool is C bytes at file offset B. A sample is identified by its sample id and begins at its pool offset.

Extent rule:

1. Collect the distinct sample ids of the bank. Each id has exactly one pool offset (0 violations in 134 banks).
2. Sort the samples by pool offset. The first sample starts at 0.
3. A sample ends where the next sample starts. The last sample ends at C.

This gives 20,718 samples in total (113 to 197 per bank). Every sample length is a multiple of 16 bytes and at least one frame. Lengths range from 21 to 3,389 frames (median 213). Together they hold 6,624,754 frames, so the pool is fully tiled by samples.

Every frame is a valid PS-ADPCM frame: filter index at most 4, shift at most 12, flag bits at most 7. Frame 0 of every sample has flags 0. The last frame of every sample has the end bit (bit 0) set (20,718 of 20,718).

### 6. Loop and one-shot classification (from ADPCM flags)

Flag bits follow the standard SPU convention: bit 0 = end, bit 1 = repeat, bit 2 = loop start. Each sample falls into one of three classes, judged from its last two frames and the presence of other flag bits.

| Class | Count | Last frame | Second-to-last frame | Other frames |
| --- | --- | --- | --- | --- |
| One-shot | 12,726 | 0x07 | 0x01 (end, no repeat) | all others are 0 or repeat-only frames |
| Loop | 7,680 | 0x03 (end and repeat) | 0x02 (6,916 samples) or 0x06 (764 samples) | every loop sample has a bit-2 frame before its last frame (7,680 of 7,680) |
| Other | 312 | 0x07 | 0x02 | no end-bit frame before the last |

Playback reading (Medium): a one-shot stops at its 0x01 frame, so the 0x07 frame after it is never reached. A loop repeats from its bit-2 frame each time the 0x03 frame is reached. The "other" class (312 samples) has no end frame before its final frame, and its final frame carries all three bits: it is both the loop start and the end-with-repeat frame. The SPU takes a loop-start bit as "the repeat address is this frame" when it begins to play the frame, and it acts on the end and repeat bits only after the frame has played. So when the voice reaches the last frame it sets the repeat address to that frame, plays it, and jumps back to it. The class therefore does not stop: it loops its last frame alone (one frame, 28 decoded samples) until the voice is keyed off and its envelope fades to zero, or until its sustain level makes it silent. The frame before it (flag 0x02, repeat bit without end) has no effect on the hardware. None of the 312 has a loop-start frame earlier in the sample, and the last frame holds non-silent data in all 312 (so the repeated fragment is audible, not a silent dummy frame). The bank voice start writes only the sample start address, never a separate repeat address, so nothing else sets the repeat point (Medium-High: the flag rule is the public SPU rule, and the driver's bank voice start is as described here). The class is 312 of 20,718 samples, 1.5 percent.

### 7. Sound block (SBlk, resources 347 to 490)

A sound block has a 0x18-byte file header, then a section (base = file 0x18, tag "SBlk"). In order, the parts are: a table of 12-byte cue entries from section +0x40; a list of 8-byte command records; a 24-byte voice table; and the block's own PS-ADPCM sample pool.

File header (file 0x00 to 0x17, little-endian u32): 0x00 = 3; 0x04 = 2; 0x08 = 0x18; 0x0C = A, the section length; 0x10 = B, the file offset of the sample pool, equal to A + 0x18; 0x14 = C, the pool length in bytes. A tail of 44 to 2,040 bytes follows the pool and is not described.

| Section offset | Size | Meaning | Observed on disc | Confidence |
| --- | --- | --- | --- | --- |
| +0x00 | 4 | ASCII "SBlk" | 144 of 144 | High |
| +0x04 | u32 | 3 | 144 of 144 | High |
| +0x08 | u32 | 4 | 144 of 144 | High |
| +0x14 | u16 | 0 | | Medium |
| +0x16 | u16 | n2: number of 12-byte cue entries (cue ids run from 0 to n2 minus 1) | total 1,950 cue entries; (w8 minus 0x40) = 12 x n2 in 144 of 144 blocks | High |
| +0x18 | u16 | n1: number of 8-byte command records | total 7,716 | High |
| +0x1C | u32 | 0x40: start of a table of 12-byte entries that runs to w8 | 144 of 144 | High |
| +0x20 | u32 | w8: section offset of the 8-byte command list | | High |
| +0x24 | u32 | bias 0x5040 that the loader adds to the pool offset of each voice that a type-1 command refers to (voice record +0x10) | 0x5040 in 144 of 144 blocks | Medium-High |
| +0x34 | u32 | w13: section offset of the 24-byte voice table | voice table fits inside the resource in 144 of 144 | High for position |
| +0x38 | u32 | slot for a child-name table | 0 in all 144 blocks | High |
| +0x3C | u32 | second table pointer | 0 in all 144 blocks | High |

Block 347 (the main sound block, see Behavior): n2 = 192 cue entries, n1 = 1,142 command records, command list at section offset 0x940, voice table at 0x2CF0 (region 15,232 bytes).

Other facts:

- Cue entry (12 bytes, from +0x40; entry k is cue id k): byte +0 default volume (0 to 127; used when a play request passes -1; the game's cue path always passes -1, so this byte is the cue volume Vc, Medium-High); byte +1 volume group (0 on disc; Driver volume, pan and pitch), also the voice-allocator range of the cue's voices (range 0, Voice ranges; Medium-High); byte +2 default pan angle (0 on disc; used when a request's pan is -1 or -2); byte +4 number of command records in the cue (0 in 390 entries, which are silent); byte +5 maximum number of simultaneous instances, used when flag 0x8 is set (Cue instance limit); word +6 flags (bit 0x2: a new play stops the running instance of the same cue; bit 0x4: an extra call in the voice start; bit 0x8: instance limit on; bit 0x10: at the limit, replace the quietest instance; bit 0x20: at the limit, replace the oldest instance; bit 0x4000 is set while the cue is played and cleared at load); word +8 offset of the cue's first command record, from the start of the command list. Runs are contiguous. Confidence High for the layout and for flags 0x8, 0x10 and 0x20; Medium for the other flag meanings.
- Command record (8 bytes; n1 records from the command list): bytes +0 to +2 are a 24-bit operand (byte +0 is the low byte; byte +2 is a runtime counter for types 25 and 39, zero on disc); byte +3 is the type (0 to 44; 25 type values occur on disc, listed below); word +4 is the delay in ticks before the record runs, counted from the previous record (for the first record, from the start of the cue); word +6 is zero on disc. For type 1 the operand is a byte offset into the voice table. Of the 4,820 type-1 records, 4,127 have an operand on the 24-byte grid and 693 do not; the 693 are ordinary voice records that sit after a 16-byte or 32-byte record of the same table (Voice table and pool; Open question 48, resolved).
- Voice record (24 bytes; a type-1 operand selects one): byte +0 voice priority, passed to the voice allocator (0 in most voices; not a key); byte +1 level, 0 to 127 (-1 to -4 select an instance variable, -5 or lower a shared driver byte; a negative result clamps to 0); byte +2 centre note = absolute value of the byte read as signed (256 minus the byte when the top bit is set; 4,125 of the referenced voices have the top bit set, 11 do not, and those also scale the pitch by 44100/48000); byte +3 centre fine tune, added; word +4 signed pan angle in degrees (0 to 359; negative values select an instance value, variable x 360 / 127); word +6 not read (zero on disc); byte +8 bend range down and byte +9 bend range up, in semitones (0 to 5 and 0 to 30 on disc); word +0x0A SPU2 ADSR1 and word +0x0C SPU2 ADSR2 values, written unchanged at each note-on (fields as in section 4); word +0x0E flag word (bit 0 = effect (reverb) send; bit 0x8 = noise voice, see Driver volume, pan and pitch; bit 0x10 = removed from the dry mix; bit 0x20 = allocator lock (Voice allocator); bits 1 and 2 select priority modes and are clear in every referenced voice; bit 0x40 is set in 384 referenced voices and its meaning is open); word +0x10 sample offset from the start of the block's own pool (file offset B), to which the loader adds 0x5040 for type-1 voices; word +0x14 sample length in bytes, a multiple of 16. Confidence High for the offset, length, envelope, tuning, pan and bend bytes; Medium for the priority and the flag bits other than 0 and 0x8.
- Because the +0x38 and +0x3C slots are zero in every block, no on-disc block contains a name table for child lookup.
- The file header words are listed above (High).

**Voice table and pool.** The voice table runs from +0x34 to the end of the section. Its record count is not stored. The region is one table that holds three kinds of record back to back, each referenced by exactly one command: 24-byte voice records (referenced by commands 1 and 9; 4,820 and 16 on disc), 16-byte channel records (command 4; 81) and 32-byte child-cue records (command 5; 24). The first record starts at offset 0 and the records tile the region exactly, with no gap, in all 144 blocks. The region length is therefore 24 a + 16 b + 32 c bytes, which is why it is a multiple of 24 in 119 blocks and 8 or 16 more in 20 and 5 blocks (the blocks that hold command 4 or 5 records). There is no second table: an operand that is not a multiple of 24 (693 of the 4,820 command-1 operands, 711 over commands 1, 5 and 9) lands on a voice record that follows a 16-byte or 32-byte record, and each such record has the normal voice layout and a pool range inside the block's pool (High). The record kind is known only from the command that names it.

The pool is C bytes at file offset B and holds PS-ADPCM frames in the format of SPEC-0009. It belongs to the block; it is not a bank pool. Every referenced voice has a pool offset and a length that lie inside the pool, and the length is a multiple of 16.

**Command types used on disc.** 25 type values occur. Types 0, 2, 3, 6 to 8, 10 to 19, 28, 33, 37 and 44 do not occur. The counts are over all 7,716 command records and over the 1,142 records of block 347.

| Type | All blocks | Block 347 | Meaning |
| --- | --- | --- | --- |
| 1 | 4,820 | 617 | Voice start. The operand is the byte offset of a voice record in the voice table. Allocates one voice, sets its level, pan and key, and starts it. Returns 0. |
| 4 | 81 | 5 | Channel set (modulator definition). Sets up one of the instance's four channels from a 16-byte record of the voice table: byte 0 = channel (0 in 48 records, 1 in 30, 3 in 3), byte 1 = amount (1 to 5 on disc; 0 clears the channel), byte 3 = kind (1 in 62, 5 in 11, 3 in 4, 4 in 4), word at +4 = 1024 in all 81, word at +6 = a rate or period (1024 in 39, 512 in 10, 20 in 9, others from 10 to 768), byte at +8 (0, 1 or 2), word at +0x0A (0 in 57, else 429 to 1514), and a 32-bit value at +0x0C. At most 10 channels run at once across the whole driver, and a running channel changes the instance level (the 127 plus a channel offset of Driver volume, pan and pitch). What each kind does is not established (Low). The record layout is High. |
| 5 | 24 | 7 | Child cue start. Starts another cue of the same sound block. The 32-byte record holds: +0 level (127 in 22 records, 110 and 65 in one each; a negative value selects an instance variable or driver byte as for a voice level; the absolute value is used, at most 127), +4 pan angle in degrees (0 in all 24; negative selects an instance value), +8 unused (0), +0x0C the cue id to start (6 to 147; below the block's cue count in all 24), and +0x10 onwards a 16-byte name (zero on disc) that is searched, first in the global list and then in this block, when the cue id is negative. It runs only while the playing instance's cue volume is not 0. The child gets the record's level and pan. Block 347 uses it in 7 records (cues 138, 139 and 147 are started this way) and 12 other blocks use it in 17 (High for the layout, Medium-High for the start). |
| 9 | 16 | 5 | Voice start for a noise voice. The driver runs the very same routine as for type 1 (both entries of its dispatch table are one routine). All 16 voices it names have flag bit 0x8 (noise voice) and pool offset 0 and length 0, so no sample base is involved (High). |
| 20 | 71 | 5 | No-op. Returns 0, a timing marker. |
| 21 | 91 | 3 | Loop start. No effect by itself; marks the start of a loop. |
| 22 | 91 | 3 | Loop back. Jumps back to just after the nearest preceding type-21 record. It has no count or exit test. |
| 23 | 8 | 2 | Break. Jumps past the next type-22 record. |
| 24 | 7 | 0 | Hold. The cue stays on this record until its caller stops the cue. |
| 25 | 572 | 147 | Random variant group. Chooses variant k at random from the variant count in byte 0. If k equals the last pick (byte 2), k moves to the next variant, modulo the count. Stores k in byte 2. Each variant is byte 1 records long; the group ends after byte 0 times byte 1 records. |
| 26 | 71 | 22 | Random delay. Returns a random value from 0 to the 24-bit operand, which is added to the next record's delay. |
| 27 | 1,252 | 169 | Random pitch bend. Draws a random signed 16-bit value, multiplies it by byte 0, divides by 100, and sets the result as the instance's bend; each voice scales the bend by its bend ranges (voice bytes +8 down, +9 up, in semitones; Driver volume, pan and pitch). High. |
| 29 | 10 | 0 | Bend set. Sets the instance's bend to a value from byte 0 (byte 0 out of 127 of the full 16-bit range) plus a base value whose source is not known (Medium). |
| 30 | 12 | 2 | Variable set. Sets the variable selected by byte 0 to byte 1. |
| 31 | 357 | 143 | Random variable. Sets the variable selected by byte 0 to a random integer from byte 1 to byte 2, inclusive. |
| 32 | 16 | 2 | Variable increment. Adds 1 to the variable selected by byte 0, capped at 127. |
| 34 | 17 | 2 | Wait on variable. Compares the variable selected by byte 0 with byte 2 under condition byte 1 (0 less than, 1 equal, 2 greater than). Holds while the condition is true; otherwise advances. Medium-Low. |
| 35 | 13 | 0 | Label. No effect; the target of type 36. |
| 36 | 13 | 0 | Goto. Jumps to the first type-35 label in the same cue with the same byte 0. |
| 38 | 1 | 0 | Wait for voices. Waits on the instance's voice slots. The hold or advance result is not resolved (Low). |
| 39 | 17 | 1 | Round-robin variant group. As type 25, but the variant is the counter in byte 2, which then advances modulo the count. |
| 40 | 7 | 0 | Variable add. Adds byte 0 (signed) to the variable selected by byte 1, clamped to -128 to 127. |
| 41 | 62 | 6 | Release voices. Releases the instance's voices normally. |
| 42 | 14 | 0 | Kill voices. Stops the instance's voices at once. |
| 43 | 73 | 1 | End. The cue ends after this record. |

Variables: each playback instance has four variables. A non-negative selector picks one of them; a negative selector picks a shared driver byte (which bytes, open).

**Play rule.** Medium-High; the tick unit is 1/240 second (Tick unit, Open question 47, resolved).

1. A play request names a sound block, a cue id, a cue volume (or -1), a request volume (1,024 = 1.0), a pan angle (or -1 or -2), a pitch offset and a bend. Nothing plays if the cue id is negative or not below the cue entry count, or if the entry's command count is 0.
2. If the cue volume is -1, the cue's default volume (entry byte 0) is used. If the pan is -1 or -2, the cue's default pan (entry byte +2) is used. The game's cue path sends cue volume -1, pitch offset 0 and bend 0, and pan -1 when the voice has no emitter.
3. If the entry's flag 0x2 is set and an instance of the same cue is running, that instance stops first.
4. A playback instance starts at the cue's first command record. That record's delay is the countdown; a delay of 0 runs the record at once.
5. Each update, the countdown drops by one. When it reaches 0 or less, the current record's handler runs and returns a value r. If r is below 0, the record is held and runs again at the next update. Otherwise the cue moves to the next record, whose countdown becomes its own delay plus r. A countdown of 0 or less runs that record in the same update.
6. Variant groups (types 25 and 39) skip the records of the variants that are not chosen and resume after the group. Loops and gotos (types 21, 22, 23 and 36) change which record follows, as the table above describes.
7. When the last record has run, or a type-43 record runs, the cue issues no more records. Voices already started keep sounding until they end, are released (type 41) or are killed (type 42), or until the caller stops the cue.

**Unreached content.** High (from the cue programs as written).

- 390 of the 1,950 cue entries have a command count of 0. They play nothing.
- 220 command records in 90 cues (58 blocks) cannot be reached by their cue's own program. These are records after an end (43) or a hold (24), and loop-back records that a break skips.
- 17 non-empty cues have no type-1, 5 or 9 record. They only set variables, pitch or timing.

**Partition rule (all 144 blocks).** Each cue's first command offset equals 8 times the sum of the command counts of the earlier cues. The sums reach n1.

**Random source.** The sound driver uses a 16-bit generator with a table of 250 16-bit values and an index. Each draw takes the entry at the index and replaces it with the exclusive-or of itself and a partner entry. The partner is 103 entries after the index when the index is below 147, and 147 entries before it otherwise. The index then advances by one, wrapping after 250 entries, and the draw is the new entry. The table and index are stored in the sound driver module on the disc, not in the sound block; their location is open (Open question 49). From the stored initial state the first four draws are 1173, 53403, 52352 and 35341 (Medium).

**Sample address and pan.** For voices referenced by type 1, the sample starts at the pool offset plus 20,544 (0x5040) and ends at its start plus its length. The 16 voices referenced by type 9 (in 10 cues) are noise voices with pool offset 0 and length 0, so the base does not matter for them (High). The instance's pan angle is added to the voice's pan angle, and the sum is taken modulo 360.

**Volume and pitch.** High. The formulas are in Behavior, Driver volume, pan and pitch.

- Volume: the request volume (1,024 = 1.0), the cue volume (entry byte 0), the instance level (127 plus a channel offset, limited to 0 to 127; 127 unless a type-4 command changes it) and the voice level (byte +1, or an instance variable when negative). Cue and voice levels use a squared curve.
- Pitch: every block instance plays at key 60, fine 0, with pitch offset 0 and bend 0 at start; types 27 and 29 change the bend. The voice's centre note (+2), centre fine (+3) and bend ranges (+8, +9) give the pitch.
- Voice byte 0 is a priority, not a key. Flag word bits 1 and 2 (priority modes) are clear in every referenced block voice.

**Tick unit (Medium).** The SBlk command interpreter runs in the 989SND I/O-processor driver, not in the game's per-frame sound-manager update. Each command delay counts down by one per step of the driver's sound tick, and the driver's sound tick runs 240 times per second. So one delay unit is 1/240 second (4.17 ms) and a delay of 240 is one second. The driver's own stall check treats a backlog of 480 unprocessed ticks as two seconds, which fixes the rate; the step link from the tick to the countdown is Medium. Open question 47 (resolved).

### 8. MMID music container (bank tail, at D + 0x10)

| Offset (from MMID) | Size | Meaning | Observed on disc | Confidence |
| --- | --- | --- | --- | --- |
| +0x00 | 4 | ASCII "MMID" | 134 of 134 | High |
| +0x04 | u16 | 2 | 134 of 134 | High |
| +0x07 | u8 | child count: 6 (93 banks), 2 (28), 3 (8), 5 (5) | sums to 134 | High |
| +0x08 | 4 | bank name | | High |
| +0x10 | u32 x count | child offsets, each relative to the MMID start | | High |

There are 663 children in all. Child 0 of every bank is a setup segment (SPEC-0009, Interactive segments); the other 529 are the playable children.

#### 8a. "MID " child header (offsets from the "MID " tag; little-endian)

Confidence: High.

| Offset | Size | Meaning | On disc |
| --- | --- | --- | --- |
| +0x00 | 4 | ASCII "MID " | 663 of 663 |
| +0x04 | u16 | version, 1 | all |
| +0x06 | u8 | 8; not used for playback | all |
| +0x07 | u8 | leftover byte; ignore | varies |
| +0x10 | 4 | bank name (the same as the bank's) | all |
| +0x14 | u32 | run-time bank link; 0 on disc. A sequence plays with the instruments of the bank that holds it | 0 |
| +0x18 | u32 | offset of the first event from the tag: 0xE8 (232) | all |
| +0x20 | u32 | initial tempo in microseconds per quarter note | 300,000 to 833,333 (31 distinct values) |
| +0x24 | u32 | ticks per quarter note (PPQ): 480 | all |
| +0x28 to +0xE7 | 0xC0 bytes | run-time state area; stale values on disc; ignore | |

#### 8b. Event stream

Confidence: High.

From the first event: a delta time, then an event, repeated until end of track. The delta is a variable-length quantity (standard 7-bit groups, most significant first; a set top bit means another byte follows), in MIDI ticks. A byte with the top bit set is a new status; otherwise the previous status is reused (running status). Every status byte, including SysEx (0xF0) and meta (0xFF), becomes the running status; on disc a byte without status follows only channel messages. Channel n is the low 4 bits of a channel status.

- 0x8n note, velocity: note off.
- 0x9n note, velocity: note on; velocity 0 is a note off (none on disc). Ignored while channel n is muted (8c).
- 0xDn note (one data byte): **note off** for that note. This replaces the standard channel-pressure meaning. On disc it is the only note-off form used (1,262,584 of them in the playable children, for 1,262,579 note-ons).
- 0xBn controller, value: only four controllers act. 0 = bank select (picks the first loaded bank with that MIDI bank number; never sent by the music on disc); 7 = channel volume; 10 = channel pan; 64 = sustain (value 64 or more holds, below 64 releases held notes). All other controllers are ignored.
- 0xCn program: the channel's program becomes that directory group of the bank (Playing a bank sound).
- 0xEn two bytes: pitch bend (Driver volume, pan and pitch, Pitch bend).
- 0xAn, and any status not listed here: error; the sequence stops.
- 0xFF 0x2F: end of track (8d).
- 0xFF 0x51 0x03 t1 t2 t3: tempo = t1 x 65,536 + t2 x 256 + t3 microseconds per quarter note (3 such events in the playable children, all 480,000).
- 0xFF with any other type: error.
- 0xF0 0x75 ...: command message (8c).
- 0xF0 with any other second byte: skipped up to and including the first 0xF7.

Disc counts over the 529 playable children: 1,262,579 note-ons, 17,890 program changes, controllers 7 (12,332) and 10 (14,099) only, 43,575 pitch bends, no 0x8n events and no velocity-0 note-ons. Program numbers are always below the bank's directory count. 99.5% of notes find a voice; 1,674 hit an empty group and 4,830 fall outside every key window, and those play nothing.

Start state of a child: all 16 channels unmuted, volume 127, program 0, pan 0, sustain off, repeat count 0.

#### 8c. Command messages (F0 75 ... F7)

Confidence: High.

A command message is a list of commands. Each command is one opcode byte followed by its operands; the message ends when the byte after a command is 0xF7. Each playing instance (one started bank sound) has 16 registers, shared by all its children, each holding 0 to 127. G is the driver's global excite, 0 to 127, set by the game (SPEC-0009, Intensity inputs). "The running child" is the child whose event is being run.

Opcodes 0x00 to 0x16 are defined (23 opcodes); 0x17 and above are ignored. By function:

- **Start, jump and stop children.** 0x12 s starts child s (no effect if it is already playing). 0x11 s starts child s and stops the running child; the rest of the message is ignored. 0x03 s stops child s; if s is the running child it stops at once and the rest of the message is ignored. 0x0E r and 0x0D r do the same as 0x12 and 0x11 with child number = register r minus 1 (unused on disc).
- **Registers.** 0x13 r x sets register r to x. 0x14 r adds 1 (at most 127) and 0x15 r subtracts 1 (at least 0) (both unused on disc).
- **Register tests.** 0x16 r x runs the next command only if register r = x. 0x06 r x: only if register r < x. 0x07 r x: only if register r > x (0x06 and 0x07 unused on disc).
- **Global excite tests.** 0x00 x runs the next command only if G > x. 0x01 x: only if G = x + 1. 0x02 x: only if G < x (0x01 and 0x02 unused on disc).
- **Macros.** 0x0B m, bytes, F7 defines macro m as the bytes up to its own 0xF7; a definition is processed even while skipping. 0x0C m runs macro m as a nested message; if the macro stops the running child, the outer message stops too.
- **Channel groups.** 0x0F g s, then (c, lo, hi) triples, then F7 defines group g with selector s (0 = G; otherwise register s minus 1) and a list of channel windows. 0x10 g applies group g to the running child: each listed channel c is unmuted when lo <= value <= hi and muted otherwise, where the value is G or the selected register.
- **Skip blocks.** 0x04: if in skip-one, enter skip-block. 0x05: if in skip-block, return to normal (both unused on disc).
- **No effect.** 0x08, 0x09 and 0x0A.

Skip rule: a failed test enters skip-one; the next command's operands are consumed and it is not executed, and then the state is normal again. Opcodes 0x04, 0x05 and 0x08 to 0x0B do not count as "the next command". In skip-block every command is consumed without effect until 0x05, except 0x0B, which always defines its macro.

Muting affects only new note-ons; notes already sounding still end on their note-off. Starting a child gives it the start state (8b) and plays it from its first event. A child started by a command during a sequencer tick is appended to the driver's active list and is reached later in the same tick, so its first events run in that tick when its first delta is 0 (Medium-High; the order rule is SPEC-0009, Same-tick order of children).

**Stopping a child (High).** A child stops in four ways: command 0x03 naming it, command 0x11 (which stops the running child after starting the new one), a command message that stops the running child inside a macro, and its own end of track with a repeat count that reaches 0 (none on disc) or a sequence error. All four end the same way. The child is removed from the active list, so none of its later events run, and every voice it started is keyed off: the voice goes into the release phase of its own envelope (ADSR2 release shift in bits 0 to 4, exponential flag in bit 5) and is freed when the envelope reaches zero. Voices are neither cut at once nor left to end on their own, so a sustained or looped note fades out over its release time instead of sounding forever. Details:
- The key-off request is sent for every voice the child owns, whether or not a note-off for it was ever reached, and including voices held by sustain (controller 64); a stopped child's pending sustain releases are therefore moot.
- A voice that the child started in the same sequencer tick, before the stop, and that the driver has not yet keyed on, is cancelled and never sounds.
- Command 0x03 naming a child that is not playing does nothing. Command 0x03 naming a child that is already stopped by an earlier 0x11 or 0x03 in the same tick also does nothing.
- Stopping the whole bank sound stops each of its children the same way (Medium).
- The one exception is a child that is paused when it is stopped: its voices are silenced at once (volume set to zero), then freed. Neither the event stream nor the command messages can pause a child, so this does not occur in the music on disc (Medium).
- Release length is the public SPU2 envelope rule applied to each voice's own ADSR2 word. In the 28,860 bank voices the release field is 12 (linear) in 10,121 voices, 13 in 5,895, 5 in 4,008, 14 in 2,450 and 15 in 1,460; 970 voices use 12 with the exponential flag. The linear rule: with release shift s of 11 or more, the level falls by 8 every 2^(s - 11) samples; with s below 11 it falls by 8 x 2^(11 - s) every sample. From full level (32,767) a linear release therefore takes about 4,096 x 2^(s - 11) samples at 48,000 per second (s = 12: 8,192 samples, 171 ms; s = 5: 64 samples, 1.3 ms). A voice keyed off part-way through its envelope takes proportionally less (Medium: the constants are the public rule for the sound chip).

#### 8d. Timing and looping

Confidence: High.

- The sequencer runs on the sound driver at 240 ticks per second and treats one tick as T = 4,170 microseconds (100,000,000 / trunc(999 x 240 / 10)).
- Rate, in hundredths of MIDI ticks per sequencer tick: rate = trunc(1000 x T / trunc(tempo / trunc(PPQ / 10))). It is recomputed at the start and at each tempo event.
- For each delta d (MIDI ticks): a = 100 d + carry. If a < 0 or a < trunc(rate / 2), the next event runs in the same sequencer tick and carry = a. Otherwise the sequencer waits W ticks, where u = trunc(tempo x trunc(a / 100) / PPQ) microseconds and W = trunc((u - 1 + T) / T), that is, u / T rounded up; and carry = a - rate x W. A clean side may instead schedule events at their nominal times (MIDI ticks x tempo / PPQ); the difference is at most one 1/240 s tick.
- End of track: the repeat count (0 for every child on disc) is decreased by 1. If it is then 0 the child ends; if it is below 0 it is reset to 0 and the child restarts at its first event in the same tick. A second end of track in the same tick stops the child (a guard against empty loops). So every child on disc loops forever.
- Commands appear every 480 MIDI ticks (one quarter note) in all 93 interactive banks, so segment changes happen on beats. Children that run together have equal lengths, so they stay aligned (SPEC-0009, Interactive segments).

### 9. Clip sets (resources 144 and 145)

Confidence: layout High; use High.

Resource 144 holds 996 clips and resource 145 holds 358 clips. Each clip is a short mono voice clip at 22,050 Hz. They are the emote clip banks: resource 144 for characters (table 7) and resource 145 for the heroes (table 8), played when a dialogue emote request starts its animation (SPEC-0009, Emote clip lookup and Emote clip playback; SPEC-0022). Of the hero bank's 358 clips, 69 can be reached; of the character bank's 996, 873 can be reached for some voice set (SPEC-0009, Emote clip census).

- No index. There is no index, directory or count in front of the clips. The first clip starts at offset 0 of the resource.
- Start. Each clip starts at a multiple of 2,048 bytes from the start of the resource.
- End and next start. A clip's data ends at its start + 0x30 + its data size. The next clip starts at the first multiple of 2,048 at or after that end. The gap is zero length when the data ends exactly on a boundary (8 cases in resource 144).
- Gaps. The bytes between a clip's data end and the next clip start are not padding in most cases. They hold PS-ADPCM frames, and every varied frame in a gap also occurs as a frame in a clip of the same resource. In 5 gaps of resource 144 and 13 gaps of resource 145 the bytes are only the fill value 0xCD. Read a clip only up to its data size.
- Tail. The bytes after the last clip are not described here (608 bytes in resource 144, 1,520 bytes in resource 145).

Clip header (0x30 bytes; numbers are big-endian, unlike the banks):

| Offset | Size | Meaning | Confidence |
| --- | --- | --- | --- |
| +0x00 | 4 | ASCII "VAGp" | High |
| +0x04 | u32 | version field: 0x20 or 0 in resource 144, 0 in resource 145. Meaning open | High (value) |
| +0x08 | u32 | 0 | High |
| +0x0C | u32 | data size in bytes, a multiple of 16. Resource 144: 2,192 to 134,944 (median 15,280). Resource 145: 3,328 to 83,312 (median 14,128) | High |
| +0x10 | u32 | sample rate: 22,050 in every clip | High |
| +0x14 | 12 bytes | 0 | High |
| +0x20 | 16 bytes | name, NUL-terminated ASCII, may be empty. 680 clips in resource 144 have a name; no clip in resource 145 has one | High |
| +0x30 | data size | PS-ADPCM frames, 16 bytes each, in the format of SPEC-0009. Frame count = data size / 16 | High |

Frames. The end flag is set in the final frames of every clip: on the second-to-last frame in most clips (331 of 400 sampled in resource 144) and on the last frame in the rest. No clip sets the loop-start flag, and no frame has a filter index above 4. Duration of a clip = (data size / 16 frames) x 28 samples / 22,050 s. Resource 144: 3,836 to 236,152 samples (0.174 to 10.71 s), median 26,740 samples (1.21 s); by length 3 clips under 0.25 s, 62 from 0.25 to 0.5 s, 300 from 0.5 to 1 s, 431 from 1 to 2 s, 145 from 2 to 4 s, 52 from 4 to 8 s and 3 above 8 s; 1,550 s in all. Resource 145: 5,824 to 145,796 samples (0.264 to 6.61 s), median 1.12 s; 38 from 0.25 to 0.5 s, 112 from 0.5 to 1 s, 138 from 1 to 2 s, 58 from 2 to 4 s and 12 from 4 to 8 s; 511 s in all. Test vectors: the shortest character clip has data size 2,192 = 137 frames = 3,836 samples = 0.174 s; the longest has 134,944 = 8,434 frames = 236,152 samples = 10.71 s; a clip of data size 15,280 = 955 frames = 26,740 samples = 1.213 s.

Content. The names in resource 144 are short English labels of emotions or gestures (for example awe, giggle, laugh, cheer, fear, shy, plead, stretch), written as the label, an underscore and a two-digit variant; 677 of the 680 names fit that form and 679 end in 01, so a name tells the emotion but not the speaker. About 58 distinct labels (case-folded) occur, many of them 9 to 14 times in the bank. Clips are mostly 0.5 to 2 s long and are non-looping. The frames of these clips do not match frames of the sound banks, sound blocks or effect clips in the sampled data (Medium). The clips are voiced and unlooped, which fits voice rather than music (Medium-High).

### 10. Packed sound word (animation tables)

Confidence: High.

A packed sound word is a 32-bit value. It is word B of each entry of a type-36 tag-2 animation table (SPEC-0005, word B): the entry for animation id k carries the sound of animation k. This refines SPEC-0005's word B reading: its "cue byte" is the block index within a group and its "category" is the group.

| Bits | Meaning |
| --- | --- |
| 0-7 | block index within the group; 255 = none |
| 8-10 | group: 0 = block 347 + index (index 0 to 2); 1 = block 350 + index (0 to 44, creature blocks); 2 = block 395 + index (0 to 59, area blocks) |
| 11-19 | cue id within the block; 511 = none |
| 20-30 | not part of the sound reference (SPEC-0005) |
| 31 | keep-alive: the cue is started as a keep-alive voice and refreshed while the animation stays current |

The group-to-block mapping uses the block-base and block-index-base tables at data 0x31A7C8 and 0x31A7E8 (Where it lives). On disc, 14,504 of the 415,574 animation-table entries carry a sound; every one is group 1 (a creature block), every cue id is below its block's cue count, 1,197 have the keep-alive bit, and 39 creature blocks are referenced.

Test vectors: word 0x80002101 gives index 1, group 1, cue 4, keep-alive, so block 351, cue 4. A word with index 255 names no block and registers nothing.

## KPV silence (update to SPEC-0009)

This answers SPEC-0009's open question about the playable end of each stream, and records the padding behaviour SPEC-0009 currently describes only as "invalid frames".

Facts over all 110 stream resources (146 to 255) on disc:

- The only frames whose filter index is above 4 are silent frames: header byte 0xC0 (filter 12, outside the defined range) followed by 15 zero bytes. There are 1,228 whole silent blocks and 164 partly silent blocks. No other invalid frame exists in any stream (High).
- Every stream's last 2,048-byte block is a whole silent block (110 of 110).
- Silent blocks sit around the nominal end of each stream. The first silent block falls between 7 blocks before and 1 block after floor(2S / 2048), where S is the per-channel length from the stream header (High for the count; the range is the observed spread).
- In stream 146 (S = 1114144, so 2S = 2228288, not a block multiple), the silent interleaved blocks are 1084 to 1089 and 1094 to 1099. Blocks 1090 to 1093 between them hold ordinary audible frames, decoding to normal levels. The 0xC0 blocks are therefore not only a tail (High for the frame data).
- In 106 of 110 streams, at least one audible block lies after block floor(2S / 2048) and before the trailing silence (1 to 6 blocks). These blocks lie past the end that the original's stream player uses (Playable end, below), so they are never heard.

Reading: the 0xC0 frames are zero data. The clean side should output silence for these frames and must not treat them as an error (Medium that they are padding, not a codec feature).

**Playable end (Medium-High).** The console's sound library reads each stream from its start sector only; the game passes no length. The library takes the stream length from the file header: it keeps S as the number of bytes still to read per channel, sends the data in chunks of up to half its ring buffer, subtracts each chunk from the count, and cuts its last chunk to the bytes that remain. Nothing after that is sent to the sound chip. So a stream plays S bytes per channel (2S bytes of the interleaved data), which is S / 16 frames and S / 16 × 28 samples per channel; S is a multiple of 16 in all 110 streams. The audible blocks that follow this point on disc (106 of 110 streams) are never played, and neither are the silent blocks after them. The played part of the data holds 45,212 silent frames, all in the last stretch of each channel that begins 1 to 5 channel blocks before the end (the music's own tail). The only open detail is a request mode in the library that would round the last chunk up to a whole 2,048-byte block; if the game uses it, at most one more block per channel is played, and the audible blocks on disc past that are still not.
- Test vector: stream 146 has S = 1,114,144. Each channel plays 69,634 frames (1,949,752 samples, 88.4 s at 22,050 Hz). Frames 0 to 69,631 are the 544 whole blocks of the channel, and frames 69,632 and 69,633 are the first 32 bytes of the next block of that channel (interleaved blocks 1088 for the left channel and 1089 for the right). Interleaved blocks 1090 to 1093 hold audible frames but are not played.

**History at silent frames (rewrite choice; Medium-Low for the original).** The sound chip decodes the frames itself, so no game or library routine resets its prediction history. A frame with filter index 12 is outside the five defined filters; the public descriptions of the decoder give such an index no coefficients, so the frame outputs zeros and the history after it is zero. After any silent run longer than two frames the history is zero on any reading. Each run of silence is followed by a frame that uses filter 1 to 4 (never 0): 198 runs, 104 of them with filter 2, 52 with filter 4, 26 with filter 1 and 16 with filter 3. Of these runs, 106 lie past the played end. The other 92 (in 46 streams) sit in the last 25 to 350 frames before the end (0.03 to 0.44 s at 22,050 Hz), so the choice affects only the last fraction of a second of those streams; decoding the first frame with zero history gives a median level of about 1,600 against about 2,500 for the frames that follow. Default if unresolved (rewrite choice): decode a 0xC0 frame as 28 zero samples and set the two history samples to zero, so every frame after a silent run starts from zero history.

## Behavior

### Loading and addressing a bank

1. Read the 0x20-byte file header. Check the tag and the consistency rules in section 1.
2. Read the section (A bytes at file offset 0x20). Read the sample pool (C bytes at file offset B).
3. A bank sound is played through the bank's directory and voice records, selected by MIDI bank number, program and note. The steps are in "Playing a bank sound" below.
4. To play a sample, decode its pool bytes (the PS-ADPCM rules in SPEC-0009, Layout and Behavior) from its extent (Layout item 5 of this spec), using the loop or one-shot class (Layout item 6 of this spec).
5. One bank at a time (Medium-High). Only the music controller loads sound banks; every other slot load is a sound block. When a new bank is staged while one is loaded, the old bank is asked to unload first, and the new bank's load waits until that unload is no longer pending. Test vector: bank 10 loaded, bank 74 staged: bank 10 unloads, then bank 74 loads; at no point are both loaded.

### Playing a bank sound (MIDI instrument path)

Confidence: Medium overall. The bank number 10, the key-window test, the tuning, the volume and pan step, the envelope writes, the pan conversion, the voice allocator, the note value and the pitch bend: High.

1. Bank select (MIDI controller 0). The channel takes the first loaded bank whose MIDI bank number (section +0x10) equals the value. Every bank has 10, so value 10 selects the first loaded bank (Open question 42). The music on disc never sends bank select: a sequence plays with the instruments of the bank that holds it (section 8a).
2. Program change. The program number is the directory group index. A program at or above the bank's directory count is an error: the driver reports error code 0x4E and stops.
3. Channel controls. Channel volume (controller 7) and pan (controller 10) are stored per channel. Pan is an angle: the controller value is converted to degrees by the rule in Driver volume, pan and pitch (Pan from controller 10). When a voice starts, the object pan offset, the directory group pan (+2) and the voice pan (+4) are added and the sum is wrapped into 0 to 359 degrees.
4. Note-on (velocity above 0, channel not muted). The group's voice records are scanned in directory order. Each voice whose key window contains the note is kept, up to 32 voices. Each kept voice starts as follows:
   a. A voice is taken from range 1 (voices 24 to 47) by the voice allocator, with the voice priority from byte +0x00 and the lock from flag bit 0x20 (Voice allocator). When the allocator gives no voice, that voice of the note plays nothing.
   b. The note value is the key (the MIDI note) times 128, plus fine 0, plus the sound's pitch offset (0 for the game's music), plus the bend term of the channel's current bend, in 1/128 semitone units (Note value at a MIDI note-on).
   c. The pitch register follows from the note value, the centre note (+0x02) and the centre fine (+0x03) by the pitch rule in Driver volume, pan and pitch (High).
   d. The volume step (bank form, Driver volume, pan and pitch) takes the object master volume, the note velocity times the channel volume, the group volume (directory +1) and the voice volume (+0x01), and the summed pan angle of step 3; it gives the left and right levels (High).
   e. +0x0A and +0x0C are written unchanged to the voice's ADSR1 and ADSR2 registers.
   f. The sample address is the voice's pool offset (+0x10) plus 0x5040 (section 2, +0x28).
   g. Bit 0 of the flag word (+0x0E) turns the voice's effect (reverb) send on or off; bit 0x10 would remove it from the dry mix.
   h. The voice is keyed on.
5. Note-off (MIDI note-off, a note-on with velocity 0, or the one-byte 0xDn note-off of section 8b). Releases the voices that the channel started for that note and group. While sustain (controller 64) is held, the release waits until sustain is released.
6. Pitch bend. Sets the channel bend (Pitch bend) and re-pitches at once every voice that the channel holds for that sound; later note-ons use it in step 4b.

### Selecting a sound (what is known)

- On disc, a sound block's child sounds are addressed by index. The name-table slot (section 7, +0x38) is empty in every block, so name lookup is not used on disc. The boot executable's debug cue-name table (Where it lives) labels each cue of each block, but no playback decision reads it.
- Each bank carries its 4-character name at section +0x0C, and the same name at +0x38 and +0x3C. For MIDI, a bank is selected by its MIDI bank number (section +0x10, 10 in all 134 banks), so bank select alone cannot tell the banks apart. The name is the only per-bank label on disc.
- The game-side cue call and its slot sources are in "Cue playback (game side)" below.
- A game-side sound is a (slot, cue id) pair; which block each slot holds is in Sound slots and their blocks below. The effect-sound path (VAG 256 to 274) is SPEC-0009.

### Reverb send clamp (formerly "Cue count rule"; High)

The routine once read here as a cue start with a count n and a kind k is the music controller's reverb send (SPEC-0009, Reverb). Its "kind" is the SPU core mask (1 or 2) and its "count" is the effect type, clamped to at most 4 with no lower bound; any other core mask sends nothing. It starts no cue. Its two callers are the music controller's two reverb pairs.

**The effect types as sounds (High for the sizes, Medium for the character).** The type is the sound library's reverb mode. The driver sets aside a block of sound memory for the effect, and that block's length is the one per-mode fact the disc holds (the mode-to-size table in the driver module): type 0 uses none, type 1 uses 9,920 bytes, type 2 8,000, type 3 18,496 and type 4 28,640. As 16-bit samples at 48 kHz these are delay spans of 0.103, 0.083, 0.193 and 0.298 seconds. In words: type 0 is a dry signal with no effect; type 1 is a small, hard room with a very short reverberation of about a tenth of a second; type 2 is the smallest and tightest studio booth, with the shortest tail (about 0.08 s); type 3 is a medium studio room, a short dense reverb of about a fifth of a second that thickens the sound without separate echoes; type 4 is a large studio, a longer, softer tail of about 0.3 s. Types 5 and above are sent as 4. The music rows use only types 3 and 4 (table values 3 to 5). The coefficients of each preset (the exact decay curve and filtering) are not on the disc: they belong to the console's sound library, so a clean side chooses any stereo reverb whose longest delay is near the span above and whose tail ends within about twice that span; this is a quality choice and not data. Depth is the effect return volume the row sends, applied equally to both channels on a 0 to 32,767 scale: the first core's rows use 12,987 or 15,185 (40 and 46 percent of full scale) and the second core's rows use 800 to 12,987 (2 to 40 percent). A voice feeds the effect only when its flag word has bit 0 set (25,034 of the 28,860 bank voices, 87 percent), and its feed is its full dry level, so the wet level of a note is the dry level times the return volume. Test vector: a row with type 4 and return volume 15,185 sends core mask 2 the type 4 reverb (28,640 bytes, about 0.30 s) at 46 percent of full scale.

### Keep-alive animation cue (formerly "Looping cue"; High)

This is the keep-alive branch of the creature animation sound (Creature blocks and animation sounds). When the new animation's packed sound word has bit 31 set, the cue starts as a keep-alive voice with the loop flag set, but only when the named block's state word is 1 (loaded and ready; Medium-High for the state name); otherwise nothing starts and nothing is reported. The slot lookup is the group-to-block rule of section 10: the block table index is the word's block index plus 0 for group 0, plus 3 for group 1 and plus 48 for group 2 (any other group adds 0); index i is sound block 347 + i. The cue id is bits 11 to 19 of the word and the emitter is the character's position. The sound-block manager keeps one state word and one slot id per block table entry (144 entries, blocks 347 to 490).

### Not sound: instance chunk fix-up (formerly "Cue timer" and "Cue chunks"; High)

The routines once read here as a cue timer and a chunk cursor are the load-time pointer fix-up of a world pack's instance chunk (chunk type 20; at most two per world). For each record of the chunk, the step s is the load base of the chunk data, not a time:
- The word at +0x2C is a pointer to the record's list: when the count word at +0x28 is 1 or more it is rebased by adding s, otherwise it is set to 0.
- The words at +0x10 and +0x14 are pointers: each nonzero one is rebased by s. When the chunk header it then points to has type 12 or 13, the pointer is moved past that header (16 bytes plus the signed size at +0x06) and the node tree that follows is relocated as a model node tree (SPEC-0004).
Nothing reads +0x2C as a timer and nothing plays a cue on this path. The clean side needs no cue timer; it resolves these offsets when it reads the chunk (SPEC-0006).

A separate routine that reads the cursor at +0x14 and sets a state word to 3 for a type-12 or type-13 chunk (cue id = base id plus the 16-bit field at +0x0E) was also listed here. No sound caller of it is known (Low).

### Cue playback (game side)

Confidence: the call's checks, arguments, slot range, voice pool size, default volume and radii: High. The keep-alive counter: High for its start value and step, Medium for the exact stop tick. Cue use: per row in the table.

Gameplay code plays a sound through one routine. Its arguments: a sound manager; a hearing radius R and a full-volume radius r0 (floats); a bank slot number; a cue id; an emitter (a pointer to the sound's source position, a 4-float vector, or none; with none the voice plays at its requested volume with no distance attenuation and the cue's default pan, and the two radii are not used); a "do not restart" flag; a volume (1,024 = 1.0, negative = default); a 16-bit extra value that is stored on the voice and not used for playback; a keep-alive flag; and a loop flag (High).

Rules, in order:

1. If the sound system is off (its on flag is zero), nothing plays.
2. If the bank slot is outside 0 to 24, nothing plays. There are 25 slots.
3. If the bank slot is empty (its active value is zero) or its block is not loaded, nothing plays (the slot-empty rule).
4. If the cue id is negative, nothing plays.
5. If the volume is negative, the sound manager's default volume is used. It is 1,024 at sound-manager set-up, and every tick the music controller rewrites it as trunc(current music row Scale x Sound-effects gain) (Scale is 1,024 in every row; SPEC-0009, Default cue volume and the area slot), so 768 at gain 0.75. 201 of the 204 gameplay call sites pass a negative volume; the front-end menu sound routine passes 1,024 and the music bank start passes its own start level. A volume below 1 after this step is refused: no voice is taken and nothing plays.
6. Same-voice test (used by "do not restart", by the keep-alive refresh and by stop): a voice matches when it is active (including start pending), its bank is the slot's current bank, its cue is the same, and either the caller's emitter is none (which matches any emitter) or the voice's emitter is the same pointer. If "do not restart" is set and a voice matches, nothing new starts.
7. Mode: keep-alive if the keep-alive flag is set, else loop if the loop flag is set, else one-shot.
8. Radii: if R > 0 the voice uses R and r0; otherwise it uses the manager radius, 200.0 units (set at sound-system start), and r0 = 0. A second start constant, 150.0, is stored next to the radius but nothing reads it. Every direct boot call site passes R = r0 = 0.0, as do the keep-alive start sites, except the two placed-emitter routines (Placed sound emitters). So R = 200 and r0 = 0 apply to all other gameplay cues.
9. Otherwise a free voice is taken from a pool of 48 voice records and given the slot, the cue, the volume and the other call values. With an emitter, the positional volume is computed first (Positional volume and pan); if the emitter is at or beyond R, nothing starts. If the slot's bank record is not ready, the voice is released and nothing plays (Medium).
10. The play request sent to the driver carries the volume, the pan angle (-1, meaning the cue's default pan, when there is no emitter), pitch offset 0 and bend 0 (sound block play rule, section 7).

Voice pool: the 48 voice records of rule 9 are 0x44 bytes each (High for the count; Medium for the record size). They are the pool that SPEC-0034 section 24 lists as a timed slot pool of 48 slots; it is a sound pool, not an effect pool.

### Sound slots and their blocks

Confidence: High.

Every sound plays on one of the 25 bank slots, and a slot holds one sound block. The sources:

| Slot | Block | When it is (re)loaded |
| --- | --- | --- |
| Main | 347 | Queued once at start; queued again when the sound system restarts |
| Hero voice (hero record +0x198) | 348 for Tor, 349 for Roberta (by the hero kind of that hero record) | When the hero record's voice slot is unset (-1). A sound-system restart resets both hero records' voice slots to -1 |
| Hero weapon (hero record +0x19C) | Tor: 461 + (sword id - 98); Roberta: 469 + (wand id - 111); 466 when the index is negative or there is no weapon | Checked every hero tick |
| Hero shield (hero record +0x1A0) | Tor: 455 + (shield id - 105); Tor with no shield: 456; Roberta: always 459 | Checked every hero tick |
| Hero power | 477 + (weapon Mote id - 160) (equipped word 2, SPEC-0021); 485 when the index is negative (no Mote) | Checked every hero tick |
| Current area | 395 + the music row's block offset (SPEC-0009, Per-tick controller step 8) | When the row's block changes; an offset below -1 unloads it |
| Creature | 350 + creature index (0 to 44) | On demand, while its reference count is above 0 (Creature blocks) |

Change rule for the weapon, shield and power slots, every hero tick: if the slot already holds the wanted block, nothing happens. Otherwise an unset slot is queued into any free slot; a loaded slot is told to unload and nothing is queued this tick; a slot that is unloading waits; a slot in any other state is queued again in place with the new block. So the old block is unloaded first and the new one loaded after it. A block number outside 347 to 490 is never queued. While a slot is not loaded, every cue on it plays nothing, so after an equipment change the new sounds are silent for the few ticks a load takes.

The power slot is used only when the active hero's kind matches the caller (Tor-only routines ask for kind 0, Roberta-only routines for kind 1); otherwise the routine gets no slot and plays nothing.

Edge cases (from the id ranges in SPEC-0021): Roberta with no wand loads 466 (a sword block). Tor with a shield id from 176 to 180 gives a block above 490: nothing is queued and his shield cues are silent. Tor's weapon rule has no upper bound, so a wand id in Tor's weapon slot would load a wand block. A Mote id above 173 gives a block above 490 (nothing queued).

Test vectors: Tor, sword 100 gives 463; Tor, no sword gives 466; Roberta, wand 111 gives 469, wand 118 gives 476, no wand gives 466; Tor, shield 106 gives 456, shield 110 gives 460, no shield gives 456; Roberta's shield slot is 459; weapon Mote 160 gives 477, 173 gives 490, none gives 485; Tor's voice block is 348 and Roberta's 349.

Block families (cue meanings from the debug cue names; Medium for the meanings):

- 347 main: footsteps, landings, world interactions, item and HUD sounds (Cue use table).
- 348 and 349 hero voice, 12 cues each: 0 hit small, 1 hit big, 2 knocked out, 3 grabbed, 4 quick attack, 5 normal attack, 6 heavy attack, 7 power-up build loop (never sent), 8 power-up release, 9 quick exert, 10 jumping, 11 pushing strain (uses under Cue uses, Hero voice slot uses).
- 455 magic shield, 4 cues: block small, block big metallic, block big organic, shield loop. 456 to 460 shields, 3 cues: block small, block big metallic, block big organic.
- 461 to 468 swords, 31 cues: 0 unsheath, 1 sheath, 2 to 4 swings 1 to 3, 5 power attack, 6 slash, 7 spin attack, 8 dash attack, 9 jab, 10 jump attack, 11 to 30 sword hit on stone, wood, metal, flesh, muck, wood bridge, treasure, lava, water, grass, leaf, flower, pollen, straw roof, loose rock, coal, fast water, twigs, bone, metal grate (in that order).
- 469 to 476 wands, 29 cues: 0 wand swing, 1 spin whoosh, 2 to 4 shots 1 to 3, 5 power-attack loop, 6 to 15 wand hit on stone, wood, metal, flesh, muck, wood bridge, treasure, lava, water, grass, 16 jump burst, 17 dash slip, 18 to 28 wand hit on leaf, pollen, straw roof, metal grate, loose rock, coal, fast water, twigs, bone, dirt, flower.
- 477 to 490 powers, 6 or 7 cues: 0 sword-activate loop, 1 wand shot, 2 sword hit, 3 wand hit, 4 extinguish, 5 move loop.
- 350 to 394 creature voices (attack, die, idle and walk loops and similar).
- 395 to 454 area ambience (torch, wind, creek and waterfall loops, block slide and similar), 3 to 31 cues.

### Creature blocks and animation sounds

Confidence: High.

Creature block lifetime: when a character is registered in the world, each of the first 167 entries of its main animation table whose packed sound word names a block (index not 255; Layout section 10) adds 1 to that block's reference count; unregistering subtracts 1 (never below 0). Once per tick, every creature block with a count above 0 that is not loaded is queued, and every loaded creature block whose count is 0 is unloaded. Blocks 347 to 349 and 395 to 490 are never loaded this way.

Animation sounds: when a creature's animation changes and the new animation's word names a loaded block and a cue, the cue plays once at the character's position; with the keep-alive bit, a keep-alive voice is started instead and refreshed each tick while the same word stays current (Positional volume and pan, keep-alive rule). The hero uses its own rule: it takes only the cue field of the current animation's word and plays it on the main slot with no position; a cue field of 511 plays nothing. The six hero action tables carry no sound word: word B is the same in all 1,158 of their entries, with block index 255, group 0 and cue 511. So the hero's rule always gets cue 511 and plays nothing (High).

No barks or greetings (High). No townsfolk or other voiced character carries a sound word: the 640 placements with a voice set (SPEC-0024, Villager row voice set) have 0 sound words, and their tables are all "none". Idle fidgets, socialising, greetings and scripted entity animations therefore play no animation sound and no voice. Only creatures (placements with no voice set; 2,265 of those 2,396 have sound words) make animation sounds, through their creature blocks. The only character voice is the emote clips of dialogue lines (SPEC-0009).

Boss sounds (High): bosses play their voices and attack sounds only through their animation tables. Every boss sound word is in group 1, so it plays on the boss's creature-block slot, once, at the boss's position, on the first tick of the animation (start frame 0, no keep-alive bit). No boss controller plays a cue for its own sounds; the only cue a boss routine plays is the Black Knight flying sword's breakable contact cue (Breakable contact cues). The per-boss tables are in the boss specs (SPEC-0011 to SPEC-0020 and SPEC-0054).

Tor's weapon hits on four special creature kinds play the creature's own block instead of the weapon cue. The kind is the s16 at +0xAC of the character record (High):

| Kind | Character | Block, cue |
| --- | --- | --- |
| 240 | Black Knight | 365, 34 |
| 480 | Undead Draik | 391, 1 |
| 405 | Dark Knight | 355, 15 |
| 1941, 1942, 1949, 1950 | Skeleton Fighter, Skeleton Wizard, Skeleton Archer, Dumb Skeleton | 361, 22 |

Each of these cues is the "impact special" cue of its block, and no animation table uses it, so it plays only through this rule. The block is 350 + the last non-zero creature index found in that type's animation table when a character of the type was last registered (one remembered index per kind; -1 until then, which plays nothing). Any other victim, and every Roberta hit, plays weapon cue 14. If the hero is in attack state 10, power cue 2 plays instead.

Script sound calls: the level-script sound calls (PlaySound, LoopSound, VisitLoopSound, AddAnimObjSound, KillSound) use the current area slot; the main-sound call uses the main slot; the hero calls use the hero's weapon, shield or voice slot. PlaySound starts a one-shot voice. LoopSound starts a keep-alive voice, not a loop voice: of the cue routine's last two flags it sets the keep-alive flag. Its emitter is the locator's point. VisitLoopSound is the matching refresh: it finds the area-slot voice with that cue and emitter (same-voice test) and adds 1 to its keep-alive counter, and it starts nothing. So a LoopSound voice lasts only while a script refreshes it every tick, and an unrefreshed one stops about 4 updates after its start or last refresh (keep-alive rule, Positional volume and pan). KillSound stops only the first matching voice of the area slot (High). AddAnimObjSound and RemoveAnimObjSound are under Animated-object sounds.

### Cue uses

#### Breakable contact cues

The contact routines on world breakables (SPEC-0036, Contact routines and cues; SPEC-0048) play one positioned cue on the main slot, with the breakable's position (entry +0x30) as the emitter (High). The cue id comes from the breakable's contact material word (entry +0x5C, SPEC-0048 Layout): material 2 plays cue 143, material 3 plays cue 145, material 4 plays cue 146, and any other value plays cue 144 (High). On disc the material is 0 in 1,557 entries, 1 in 294, 3 in 19, 4 in 203 and 2 in none, so cue 143 is never chosen by disc data. Roberta's shot on a breakable always plays cue 144, whatever the material (High). The cue plays even when the contact does not break the entry (an intact entry in state 3, SPEC-0048). These ids are cue ids of block 347; no effect id is involved.

#### Cue use table

Uses come from the block's debug cue names together with the routines that send each id. All rows are on the main slot (block 347) unless marked. Confidence: High for the ids (a fixed id at the call site, or the computed rule given), Medium for the uses unless marked.

| Cue id (block 347) | Use | Trigger | Confidence (id / use) |
| --- | --- | --- | --- |
| 0 to 59 | footsteps: 20 surfaces x 3 gaits (rule below) | each new foot contact of the grounded hero | High / High |
| 60 to 77 | landings, 18 surfaces (rule below) | hero landing | High / High |
| 78 to 137 | big footsteps, the same 20 x 3 layout from base 78 | no call site; possibly unused | High / Low |
| 138, 139 | wood creak (walk, run) | no call site | High / Medium |
| 140 | ladder-climb step, at the climber's position | 2 call sites | High / Medium |
| 141 | rope-slide loop: keep-alive, do-not-restart, at the hero | rope slide | High / Medium |
| 142 | chest open (also played by level scripts) | chest open | High / Medium |
| 143, 144, 145, 146 | smash: treasure chest, crate, bones, pot; chosen by breakable material (Breakable contact cues) | breakable contact | High / High |
| 147 | wood debris | no call site | High / Medium |
| 148, 149 | pause, unpause | pause menu | High / Medium |
| 150 | text pop-up | no fixed-id call site | High / Medium |
| 151, 152, 153 | text dismiss, button advance, button back | HUD and front end | High / Medium |
| 154 | d-pad press (menu cursor move) | front end, HUD, game flow (13 call sites) | High / Medium |
| 155, 156 | gain health, gain magic | after a successful consumable use whose heal fraction for that pool is above 0.1 | High / High |
| 157, 158 | low health, low magic | every hero tick while the pool is below 0.15 of its maximum, with do-not-restart, so the cue repeats as each instance ends | High / High |
| 159, 160 | use food, use potion: 160 for the two restore ids 58 and 59 and some other branches, 159 otherwise | consumable use | High / Medium (split) |
| 161 | use cloak | consumable use | High / Medium |
| 162 | use fail (refused use or invalid menu choice) | 10 call sites | High / Medium |
| 163, 164 | Mote on, Mote off | Mote activate and deactivate | High / Medium |
| 165, 166, 167 | coin pickup, coin spend (no call site), coin bounce (positioned) | coins | High / Medium |
| 168 (0xA8) | save-point loop, kept alive within 15 units of the nearest save point (Save-point cue) | save points | High / High |
| 169 | save complete | HUD and level scripts | High / Medium |
| 170, 171, 186 to 189 | hero impact by attack kind and armour (rule below) | hero takes damage | High / High |
| 172 | projectile impact: never chosen (Edge cases) | none | High / Medium |
| 173, 174, 177, 179 to 181 | vine climb, plant smash, egg smash, plant walk, run and jog | no boot call site | High / Medium |
| 175 | book open | book reader | High / Medium |
| 176 | hero swap | hero switch | High / Medium |
| 178, 182 | baggus smash, Mote chest open | world routines | High / Medium |
| 183, 184, 185 | rope gun: fire; hit (keep-alive, do-not-restart, at the hit point); reel loop (keep-alive, do-not-restart, no position) | rope gun | High / Medium |
| 190, 191 | sent to the weapon slot, where no block has that many cues, so they play nothing (Edge cases) | ground routine | High / High |

Footsteps (High): cue = gait offset + surface offset, played on the main slot (block 347) at the hero.

When it fires: on a hero tick when the hero record's grounded word is set, a current animation frame carries event bit 0x80 or 0x100 (SPEC-0005), and the integer part of the layer-0 frame cursor differs from the remembered frame. The comparison is exact integer equality, so a frame that stays under the cursor for several ticks plays one cue. The fired frame becomes the remembered frame; the memory is reset to -1 when the animation id changes.

Gait offset, from the hero's locomotion state: run (state 4) +40, walk (state 3) +20, any other state (the slow walk, state 2, included) +0. In the block's cue names the three groups are the run set (40 to 59), the jog set (20 to 39) and the walk set (0 to 19).

Surface offset o(k), from the ground material code k under the hero. The map is a code constant, not disc data. Surface ranks 0 to 19 in cue order are dirt, stone, grass, metal, metal grate, wood, wood bridge, water, treasure, muck, lava, leaf, twigs, flower, pollen, straw roof, loose rock, coal, fast water, bone; the hero reaches only ranks 0 to 10:

| Ground code k | o(k) | Surface | Dust entry (SPEC-0034) |
| --- | --- | --- | --- |
| 0, 18 | 0 | dirt | 1 |
| 1 | 2 | grass | 2 |
| 2 | 10 | lava | none |
| 3 | 3 | metal | none |
| 4 | 4 | metal grate | none |
| 5 | 9 | muck | 5 |
| 6 | 1 | stone | none |
| 7 | 8 | treasure | none |
| 8, 11 | 7 | water | 8 |
| 9 | 5 | wood | 9 |
| 10 | 6 | wood bridge | none |
| 12 to 17, 19 and up | no cue | - | none |

So the hero's footstep cues lie in 40 to 50 (run), 20 to 30 (walk) and 0 to 10 (other states); ranks 11 to 19 are never chosen. The same footstep frame also starts the footstep dust; the dust entry indexes the footstep dust table at 0x3150A8, and only entries 1, 2 and 8 hold a definition id (SPEC-0034, Hero footstep dust).

Landings, cues 60 to 77 in the order stone, wood, metal, metal grate, water, grass, dirt, muck, treasure, wood bridge, leaf, flower, pollen, straw roof, loose rock, coal, fast water, bone. Hero landing by ground material code: 0 gives 66, 1 gives 65, 3 gives 62, 4 gives 63, 5 gives 67, 6 gives 60, 7 gives 68, 8 and 11 give 64, 9 gives 61, 10 gives 69; 2 (lava) and any other code give no cue. Cues 70 to 77 are never chosen (High).

The material code comes from the ground entry under the hero: the unsigned 16-bit word at entry +0x40 (values 0 to 19 on disc, the 20 surfaces). When the ground search picks the entry, the hero's own landing word (hero +0x188) takes that value; the landing routine reads the word from there (High for the copy and the switch; Medium-High for the field's meaning: the values 0 to 19 match the 20 surfaces, and the kind word at +0x38 is a different field). The air-phase handlers also write the float 1.0 to the same word when they start the air and landing steps. The landing cue reads the word as a code, so 1.0 matches no case and plays nothing (Medium for the order of these writes against the ground search). The landing cue routine is called from the air phase's landing steps: the hard-landing handler (air phase 10, landing clip 118) and the soft-landing handler (air phase 6, clip 68) play it when their landing clip is requested and is not already the current clip, and the moving-landing handler (air phase 7) plays it on every run of its step (Medium-High). It is not keyed to a state change or a contact flag.

Hero impact (the hero takes damage of attack kind k; the armour is the kind-5 equipment id, SPEC-0021): k = 0 (melee): Roberta 170; Tor 187 when the id is 1, 186 when it is 2 or more, else 170. k = 1 (projectile): Roberta 170; Tor 189 when the id is 1, 188 when it is 2 or more, else 170. k = 2 or 3 (magic): 171. Any other k: no cue. The same hit plays hero voice cue 0 or 1 on the voice slot: a fair draw, draw 0 gives cue 1 (hit big), otherwise cue 0 (hit small) (High).

Hero voice slot uses (High; cross-references SPEC-0009 and SPEC-0024 table 14):

Every hero voice line is a cue on the hero's voice slot (block 348 for Tor, 349 for Roberta). It plays at the default cue volume, trunc(1024 x Sound-effects gain), so it follows the Sound effects option, not the Voice option (the Voice gain applies only to emote clips, SPEC-0009). With each one the game also requests a stinger whose index is k for Tor and k + 24 for Roberta (by the hero's kind word). Every such index is 19 or more, so the stinger player drops it: these are the cut hero-voice stingers (SPEC-0024, table 14). No hero voice cue has a cooldown: a new call starts a new voice unless the call is marked do-not-restart, and only cues 2 and 9 are.

| Event | Voice cue | Position | Do-not-restart | Dropped stinger, Tor / Roberta |
| --- | --- | --- | --- | --- |
| Hero takes damage d (any attack kind, unless a world event is busy) | fair draw: 1 (hit big) on draw 0, else 0 (hit small) | none | no | 20 / 44 when d > 0.05 x max health, else 19 / 43 |
| Hero enters state 16 (dead; the "knocked out" cue) | 2 | hero | yes | 21 / 45 |
| Enemy grab on the hero | 3 | none | no | none |
| Tor press-and-hold attack, released before the first threshold | 4 (with weapon cue 2) | none | no | 26 |
| The same, released between the two thresholds | 5 (weapon cue 3) | none | no | 27 |
| The same, held to the second threshold (charged) | 6, then 8 | none | no | 28, then 30 |
| Roberta attack steps 1, 2 and 3 | 4, 5, 6 (with wand cues 2, 3, 4) | none | no | 50, 51, 52 |
| Roberta power shot (with a Mote) / release (no Mote) | power cue 1 / voice 8 | none | no | 54 / none |
| State 30 lunge | 9 | hero | yes | none |
| Jump take-off | 10 | none | no | 32 / 56 (one attack sub-kind path sends 56 for either hero) |
| Block push (state 18) | 11, together with the area-slot cue that the level registers as the movable-block sound | none | no | 33 / 57 |
| Post-leap fall (state 48), every tick of the fall animation | none | - | - | 25 / 49 |
| Add an item that raises a stat (main cue 159) | none | - | - | 37 / 61 |
| Item showcase start | none | - | - | 42 / 66 |
| Level script PlayHeroSound(h, cue, pos, dnr) | the given cue, only when h is -1 or equals the selected hero's flag (0 Tor, 1 Roberta) | the given position, else the hero | as given | none |

- PlayHeroSound gate: the call plays on the selected hero's voice slot, and only when its hero argument is -1 or matches the selected hero; otherwise nothing plays. The one script use on disc is in pack 1244: Roberta, cue 0, no position, after a scripted hero animation.
- Cue 7 (power-up build loop) is never sent: no event or script plays it.
- Voice ids of 12 or more play nothing (the blocks have 12 cues): consumable use sends 35 to 38 (Tor) or 59 to 62 (Roberta), and projectile and enemy-strike hits send 20, 22 or 28 to the victim's voice slot.
- Census: 11 of the 12 cues of each hero voice block are reachable (all but cue 7).

Weapon slot uses: 0 unsheath and 1 sheath (draw and put away); 2 to 4 combo swings; 5 charge loop (keep-alive while charging); 7 spin, 8 dash, 10 jump attack; 14 creature hit (Creature blocks and animation sounds); 16 in one attack state (sword: hit wood bridge; wand: jump burst); world hits per Weapon hits on the world.

Shield slot uses: 0 block small, and a computed big-block cue. Power slot uses: 0 activate loop (keep-alive), 1 shot (do-not-restart), 2 and 3 hits, 4 extinguish, 5 move loop (keep-alive).

Area slot uses: the level-script calls PlaySound, LoopSound, AddAnimObjSound and KillSound with small cue ids (5, 7 and 11 the most common), the movable-block sound, and the placed sound emitters (Placed sound emitters).

#### Save-point cue (0xA8)

Confidence: High for the call and the gate; Medium for the exact keep-alive stop tick (Positional volume and pan, keep-alive rule); Low for the audible effect of the emitter quirk.

- Cue 0xA8 (168, the save-point activation cue by the cue-name index rule) is played by the boot program. Once per tick, the world's own character region finds the save point nearest the selected hero (SPEC-0006 kind 4; the full rule is in SPEC-0025). When the squared distance to it is below 225 (15 units, strict), the cue is started or kept alive on the main bank slot (the slot that also plays 0xAF and 0xA2).
- Start or keep alive: if a voice with the same slot, cue and emitter is already playing, that voice is refreshed (a per-voice keep-alive counter is raised by 1). Otherwise the cue is started with the "do not restart" flag and the default volume. When the hero leaves the 15-unit range the step stops refreshing it.
- Emitter quirk (reproduce or decide). The emitter passed with the cue is meant to be the save point, but in the branch that serves zero-flag save points (all 88 save points on disc) it is taken from the other branch's remembered index, which stays at "none". It then resolves to a slot just before the save-point list that is empty unless the region's last spawned-character entry is in use. So in practice the cue starts with no emitter position, not at the save point. A clean implementation may play it at the save point or as a plain cue; the audible difference is not established.

Music: no cue id plays music. Music uses the MIDI and stream paths described in SPEC-0009. Ambient sound plays on the area slot (blocks 395 to 454): level-script calls and the placed sound emitters below (High).

Call counts: 204 direct call sites of the cue routine in 119 functions (one of them inside the keep-alive start). 166 pass a fixed cue id (41 distinct ids); 38 pass a computed id. The keep-alive start has 25 call sites in 24 functions. Neither routine is ever called through a pointer: no data word holds either address and no code builds either address in a register (Open question 17). High for the counts; Medium for completeness.

### Positional volume and pan, keep-alive and loop voices

Confidence: High for the formulas and the keep-alive counter rule and per-pass order; Medium for the loop quirk.

Listener: once per frame the listener point is the camera position (camera +0x30) plus 5.0 times the camera axis at +0x20, and the listener matrix is the camera's first three rows (+0x00, +0x10, +0x20) (SPEC-0027).

Positional volume of a voice with base volume V (its start volume), hearing radius R and full-volume radius r0: d = the 3D distance from the listener to the emitter. If d >= R the voice is out of range. Otherwise e = d - r0; if e < 0 the volume is V; otherwise S = R - r0 when r0 < R, else S = R, and the volume is trunc(V x (1 - e / S)). A result of 0 or less counts as out of range. A cue whose emitter is out of range does not start.

Pan angle: v = emitter - listener; u = v.x x row0 + v.y x row1 + v.z x row2 (the offset as a row vector times the 3x3 listener matrix). phi = +pi/2 or -pi/2 (the sign of u.z) when |u.x| < 1e-6, else atan(u.z / u.x), plus pi when u.x < 0. Pan = trunc((1 - (phi / pi + 0.5) / 2) x 359) degrees, in single-precision arithmetic. The volume and pan are sent to the driver together, recomputed on every pass from the voice's start volume (not from the last result). The driver's angle convention is 0 front, 90 right, 180 behind, 270 left (Driver volume, pan and pitch).

Direction of the listener rotation (High for the arithmetic; Medium for the screen side). The listener matrix rows are the camera's right, up and forward axes in world space (SPEC-0027). The sum above multiplies the world offset by those rows as a camera-to-world rotation, which is the inverse of the world-to-camera rotation (the per-axis dot products v . row0, v . row1, v . row2). For a level camera, u is the camera-space offset turned about the vertical by twice the camera's yaw. The pan formula also maps u.x > 0 to 269 (left) and u.z > 0 to 179 (behind). Together:
- With the camera facing world +z or -z, an emitter on the camera's right (along row 0) gives pan 269 and is heard on the left; one ahead gives 179 (rear, the weaker channel phase-inverted).
- With the camera facing world +x or -x, the same emitter gives pan 89 and is heard on the right; one ahead gives 359 (front).
- At the diagonals (yaw 45 degrees and so on) an emitter on the camera's right is heard near the centre.
- So the stereo side of a world sound swaps as the camera turns. A clean side that wants the original sound keeps the sum as written; a clean side that wants correct stereo uses the dot products and pan = 90 - phi in degrees. The screen side of row 0 is taken from SPEC-0027 and SPEC-0046 (camera-space x to the right of the screen).

Start pan: the cue start computes its first volume and pan with the listener matrix pointer left from the last per-pass update. That pointer names a copy of the camera rows on the per-pass routine's stack, which has been reused by the time a gameplay cue starts. The start volume and the range test use the listener point, which is stored by value, so they are exact; the start pan is unreliable for at most one pass, until the next per-pass update recomputes it from a fresh copy (Medium). A clean side computes the start pan from the current camera.

Per-frame voice update (all 48 voices):

- A voice with no emitter sends its base volume again with pan 0 (every cue's default pan is 0 on disc, so this matches its start); if its base volume is 0 it is stopped, unless it is a loop voice.
- A voice with an emitter recomputes its volume and pan and sends them. If it is out of range it is stopped, except a loop voice, which keeps playing at its last sent volume and pan (quirk, Medium).
- Keep-alive voices start with a counter of 4. On each sound-manager voice update, a keep-alive voice subtracts 1; if the result is below 1, it is stopped on that update. A same-voice match on a keep-alive refresh adds 1. Therefore four consecutive voice updates with no intervening refresh stop the voice on the fourth update; a refresh before an update offsets that update's decrement. The game's per-pass audio routine calls the voice update before its ambient-controller refresh, so an ambient refresh in that routine follows the decrement. A cue started by that ambient controller starts after the voice update and is first eligible for decrement on the next pass. The counter value, decrement, threshold and this call order are High.
- Loop voices: when the driver reports the end of a loop voice, the voice stays allocated, and a loop voice with no driver handle and no pending start is sent to the driver again (restarted).
- Stopping: a stop on a voice with no driver sound counts retries and frees the record after the third retry; otherwise the driver stop is sent once (Handle stop with retries).

Test vectors: base 1024, R 200, r0 0: d = 0 gives 1024; 50 gives 768; 100 gives 512; 199.9 gives 0 (kept, silent); 200 does not start. Base 768 (gain 0.75), R 200, r0 0: d = 50 gives 576; 100 gives 384; 199.9 gives 0 (still playing, silent); 200 is out of range (not started). Base 768, R 30, r0 10: d = 5 gives 768; 20 gives 384; 30 is out of range. Pan: u = (1, 0, 0) gives 269; (-1, 0, 0) gives 89; (0, 0, 1) gives 179; (0, 0, -1) gives 359.

### Placed sound emitters

Confidence: kind 8 High; kind 1 High.

Each frame, while the sound system is on and the area slot is loaded, the camera tick finds the type-19 grid cell at the camera and walks its cell list A:

- Kind-8 entries (ambient loops and adventure zones; the n4 list of the type-11 chunk, the 0x50-byte fourth-list records of SPEC-0028 section 6): handled only while the listener is inside the entry's sphere (centre +0x30, radius +0x3C); outside it the entry's start-delay counter (+0x4F) is reset to 0. The world loader also clears this counter when the area loads, so every loop starts from a count of 0 (High). When the sub-kind byte (+0x4E) is 0, the volume factor (+0x44) is 0 or more and the cue (+0x48) is 0 or more: if a matching voice exists it is refreshed and the counter reset to 0; otherwise, while the counter is below 6 it increases by 1, and a start is tried whenever the new value is 3 or more (at most 4 tries until the next reset). The start is a keep-alive, do-not-restart voice on the area slot, at the entry's centre when the positioned byte (+0x4D) is set and with no position otherwise, with R = +0x3C, r0 = +0x40 and volume trunc(default volume x factor). So an ambient loop starts on the third tick inside its sphere. A non-zero sub-kind is an adventure zone (SPEC-0009, Intensity inputs).
- Kind-1 entries (sphere sound emitters). These are the type-11 sphere primitives (the sphere list, SPEC-0028 section 6.2) in cell list A whose header flags (+0x12) have bit 0x10. They are not the AddAnimObjSound path (Animated-object sounds).
  - Gate: the distance from the listener point to the sphere centre (+0x30) must be below the sphere radius (+0x3C) plus 1.0 (strict).
  - Cue: the signed 16-bit header field +0x1E, played on the area slot.
  - If a voice with the same slot, cue and emitter (the sphere centre) exists, it is refreshed. Otherwise a keep-alive, do-not-restart voice is started at the centre, with hearing radius R = max(30.0, the radius at +0x28), r0 = 0 and the default cue volume. There is no delay counter (unlike kind 8): the loop starts on the first frame inside the gate.
  - Spheres without bit 0x10 make no sound. Entry-list records with kind-1 headers (flags 0x0020 or 0x0021) never do.
- Volume source: both kinds use the audio settings object's copy of the default cue volume, trunc(Scale x Sound-effects gain), which the music controller rewrites each tick (SPEC-0009, Default cue volume and the area slot). The copy lives in zero-filled memory, so it is 0 until the first controller tick, and no emitter can start before then (High).
- Disc census of kind 1 (High): 216 sphere records, all with flags exactly 0x0010, in 12 packs: 1244 (30), 1245 (38), 1246 (39), 1250 (15), 1251 (2), 1255 (15), 1257 (14), 1371 (8), 1393 (27), 1395 (12), 1396 (8) and 1411 (8). Cue 0 in 202 records, 1 in 5, 3 in 6, 4 in 2 and 500 in 1 (pack 1246). Cue 500 is at or above the area block's cue count, so it is silent. Radius 40 in 136 records; the others 11.6 to 110.3. The area blocks of these packs (447, 448, 450, 451, 432, 435 and 440) all label cue 0 as a torch loop, 1 as a wind loop and 3 as a waterfall loop, and cue 4 is the block's fifth loop (Medium for the meanings, from the debug cue names).
- Edge case: when the radius r is 30 or more, R = r, so a listener at r <= d < r + 1 passes the gate but is out of range, and nothing starts there (High).
- Test vectors (kind 1): sphere radius 40, default volume 768, listener 30 units from the centre: a voice at R 40 with volume trunc(768 x (1 - 30/40)) = 192. Listener at 40.5: the gate passes and the start is refused (out of range). Sphere radius 11.599 (pack 1371): gate d < 12.599; at d = 12, R = 30 and the volume is trunc(768 x 0.6) = 460; at d = 12.6, no voice.

### Animated-object sounds (AddAnimObjSound)

Confidence: High for the rules; Medium-High for the effects of the quirks.

- Table: at world setup each animated placed object (the world's animated-object list, SPEC-0028 section 7.1) gets 3 sound slots. A slot holds a cue, a start time, an end time and the object. All slots start empty (cue -1).
- AddAnimObjSound(W, i, cue, a, b): if i is below the object count, the first empty slot of object i is filled with:
  - start time = max(a, 0.00001) x D;
  - end time = min(b, 0.99999) x D;
  - where D = (F - 1) / 15 s is the object's duration (F keys).
  - If all 3 slots are in use, nothing happens. The call needs the world's sound slot to be set.
- RemoveAnimObjSound(W, i, cue): empties the first slot of object i with that cue. A voice already playing is not stopped (quirk).
- Per tick, for each animated object in state 0, 2, 5 or 6:
  - Let p be the object's time before its advance and q the time after it, including the end clamp and loop wrap of SPEC-0028 section 7.1. Let lo = min(p, q) and hi = max(p, q).
  - Start: the first slot (in slot order) with lo <= start < hi plays its cue once on the area slot, with do-not-restart, the default cue volume, R = 200 and the object's reference centre (+0x180) as the emitter.
  - End: the first slot with lo <= end <= hi stops that cue at that emitter.
  - At most one start and one stop per object per tick.
  - SetAnim (set time) runs the same check between the old and the new time.
- Consequences (Medium-High):
  - Played once forward from time 0 with (0, 1): the cue starts on the first tick and is stopped on the tick the object reaches D, so the sound lasts as long as the motion.
  - Looped (LoopAnim): on the wrap tick lo is the small new time and hi the old time, so a start strictly inside that interval fires again at the wrap, as well as at its normal crossing. A start of 0.00001 x D fires only when the loop is started from time 0, and an end near D is almost never crossed, so a looped (0, 1) sound plays once at the start and is never stopped by the window.
  - Reverse play from D (negative speed): the start fires when the object reaches time 0, the end of its motion.
  - A start fraction of 1.0 gives start = D. The time never exceeds D (it is clamped or wrapped), so lo <= D < hi never holds and such a sound never plays (quirk; 6 sites, all in module 1090).
- Script census (High): 140 AddAnimObjSound sites in 14 modules (758, 909, 1081, 1090, 1130, 1171, 1247, 1248, 1249, 1381, 1391, 1392, 1394 and 1398).
  - (a, b) pairs: (0, 1) 110, (0.5, 1) 12, (0.3, 1) 9, (0.1, 1) 6, (1, 1) 6, (0.2, 1) 1 and (0.7, 1) 1.
  - Most common cues: 8 (35 sites), 7 (28), 9 (15), 12 (15), 6 (13) and 5 (11).
- Cue resolution per module (High for the blocks and entry counts; Medium-High for module 1391, whose pack has two rows). Each module's calls play on the area slot, which holds the sound block of its own pack's music row (row block offset + 395). Module (block): cue ids used.
  - 758 (398): 6. 909 (443): 6, 7. 1081 (446): 5, 6. 1090 (419): 0. 1130 (428): 8, 10. 1171 (428): 7.
  - 1247 (449): 5, 6, 8, 9. 1248 (449): 16. 1249 (449): 5, 6, 8, 9, 12, 13.
  - 1381 (434): 7, 8. 1391 (453): 6. 1392 (439): 11, 12. 1394 (440): 13. 1398 (438): 7, 8.
  - Every cue id is below its block's cue entry count, and 139 of the 145 slot fills name an entry with at least one command, so they play.
  - The 6 fills of module 1090 name cue 0 of block 419, whose entry has no commands. They are silent twice over: the start fraction 1.0 never fires, and the entry would play nothing (quirk).
  - Test vectors: module 1249 adding cue 13 to an object plays cue 13 of block 449 when the object's time crosses the start. Module 1090 adding cue 0 with (1, 1): no sound at any time.
- **Call-site arguments and gates (R-0111; High for object, cue, fractions and direct gates; Medium for inferred whole-game reachability).** The host arguments are object index, cue id, start fraction and end fraction, in that order. Every window listed below ends at 1.0. Pack 758 registers object 2/cue 6 over 0–1 without an Init caller gate. Object 2 has six keys (D=1/3 s); its trigger callback plays it forward at +1 when the collision kind is 1, waits for trigger exit, then plays it backward at −1. A full forward pass crosses the start on the first update and the end on update 20; reverse playback crosses the end first and the start on its final update at time zero. Trigger entry and collision kind remain route conditions. Pack 909 registers object 6/cue 7 and object 7/cue 6 over 0–1; both are added by the helper selected when the last active world is 71. Its game-state checks 1–6 select branches in which both registrations occur, so they do not change the pair. Pack 1081 registers object 6/cue 5 and object 5/cue 6 over 0–1 at the end of Init, after the state-41 branch rejoins. The state-41 split changes initial objects and effects, but does not gate these sound calls; the nearby debug-mode check gates a trigger registration only. In pack 1081, trigger 2 can play object 6 from time zero at speed +1 when flag 41 is clear; object 6 has 12 keys (0.733333… s). On the encounter success branch, object 5 is activated at time zero and played at speed +1; it has 66 keys (4.333333… s). The first advancing object update crosses each registration's clamped start (0.00001 × duration), and the terminal update crosses the end (0.99999 × duration). Cue 5 and cue 6 each contain three reachable voice starts. These windows are therefore crossed on their described script routes, though route conditions and driver playback still determine whether output is audible.
  - Pack 1247 has 32 registrations over 0–1. In site order, object/cue pairs are: (2,5), (3,8), (6,5), (7,6), (8,8), (10,8), (11,9), (11,9), (22,5), (23,6), (24,8), (26,8), (29,8), (25,9), (27,9), (28,9), (30,5), (31,6), (32,8), (34,8), (33,9), (35,9), (13,5), (12,6), (14,5), (15,6), (16,8), (18,8), (17,9), (19,9), (5,8), (4,9). The sites follow a join after the game-state-78 branch, so that state does not gate them; all 32 run whether state 78 is clear or set.
    - The object data gives F=6 for objects 6, 7, 12–15, 22, 23, 30 and 31; F=10 for object 2; F=16 for objects 5, 8, 10, 11, 16–19, 25–28, 33 and 35; F=18 for object 3; F=24 for objects 4 and 29; F=66 for object 24; F=71 for object 34; and F=111 for object 32. Thus D=(F−1)/15 ranges from 1/3 s to 22/3 s. Each registered object has a script PlayAnim call at speed +1. From time zero, the first 1/60 s update crosses the clamped 0–1 start and the terminal update crosses its end. Object 11/cue 9 has two identical slots; the per-object first-matching-slot rule means only the first slot can match the shared start crossing on a monotonic pass, so the duplicate does not imply two starts. Callback collision, level-state and branch checks determine whether a given animation runs; these crossings are conditional on a full forward playback, not proof of normal-route execution or audible output.
  - Pack 1249 has 31 registrations. The object/cue pairs are (0,8), (1,9), (2,13), (3,13), (4,13), (5,13), (6,13), (7,13), (8,8), (10,9), (11,5), (12,8), (13,6), (14,5), (15,8), (16,6), (17,9), (18,5), (19,8), (20,6), (21,9), (22,5), (23,8), (24,6), (25,9), and (43–48,12). Object 12/cue 8 starts at 0.2; objects 43–48/cue 12 start at 0.5; object 1/cue 9 starts at 0.7. All others start at 0. The state-80 branch controls two groups of initial object activation, then rejoins before the sound registrations; all 31 calls run with state 80 clear or set. **[High for the branch join and registrations]**
    - Every registered owner has a script PlayAnim call at speed +1. For a complete pass from zero, the partial starts cross at object 12/cue 8 on update 41 of 200, objects 43/45/47 on updates 29 of 56, objects 44/46/48 on updates 25 of 48, and object 1/cue 9 on update 68 of 96. All full-window starts cross on update 1; all ends cross on the terminal update. These are conditional on the corresponding trigger and branch running and the object starting at zero; they do not establish ordinary-route execution or audible output.
  - Pack 1381 has 19 registrations over 0–1: objects 50–55, 58, 62, 66, 70, 74, 78, 170, 173, 176, 179, 182 and 185 use cue 7; object 18 uses cue 8. Init calls a helper that performs all 19 registrations unconditionally. Registration therefore happens during Init; whether a particular playthrough loads the pack is a separate route question.
    - Shipped key counts are F=61 for objects 50–55 and 18, F=9 for objects 58, 62, 66, 70, 74 and 78, and F=46 for objects 170, 173, 176, 179, 182 and 185. A complete zero-to-end 0–1 pass would cross starts on update 1 and ends on updates 240, 32 and 180, respectively. The six active-object helper branches map all 18 cue-7 owners to matching PlayAnim object ids; their requests include half-speed and full-speed cases. Each cue-7 callback is paired with a trigger callback that accepts contact states 1 or 2, requires an object-contact record, and holds a temporary gate while waiting for the hero to leave. During that interval, the cue-7 callback accepts an attack event or, when no attack event is active, a shot event at one of its six registered indices; those indices dispatch to active-object branches that can play the mapped cue-7 owners. These are the callback and index gates. In the world geometry, each radius-3.5 cue sphere lies wholly inside its paired radius-20 gate sphere; their center separations range from about 0.60 to 1.83 units, below the 16.5-unit containment allowance. A hero event in a cue sphere can therefore occur while the paired wait-for-exit gate is active. Ordinary-route visitation, callback scheduling, event timing, animation start time and window crossing are not established. The six separate control-thread parameter rows list objects 0–5 and do not drive this cue-7 mapping. For the six-station sequence, Init puts object 18 at its start time and the final goal-contact callback requests it forward only after all six completion flags are set. The separate activate-world callback can set object 18 to its end pose when the world is entered with all six flags already set and flag 337 clear; it is not the final goal-contact path. Therefore a first-time in-session completion from an uncompleted state crosses cue 8's start on update 1 and end on update 240; a reload into an already completed state may start at the end pose and does not demonstrate that forward crossing. Ordinary-route visitation, window crossings and audible output are not established.
  - Pack 1398 has nine registrations over 0–1: objects 1, 2, 5, 6, 7 and 9 use cue 7; objects 3, 4 and 8 use cue 8. Init makes its state-106-dependent object-activation choices, including state-720–724 choices on one branch, then joins before the nine registrations. These state checks and the other unresolved route predicates do not gate the sound-registration family.
  - **Cue payloads for the requested R-0111 registrations (High for stored fields and sequential command order).** In pack 758, cue 6 starts voice keys 176/176 at volumes 110/85, with the second 25 ticks later. Pack 909 cue 6 starts keys 176 and 181 together; cue 7 starts key 181, releases after 50 ticks, then starts key 178 and two key-174 voices (one after a further 25 ticks). Pack 1081 cues 5 and 6 each start three voices and then end; later voice records are unreachable after the end command. In the shared cue block used by packs 1247/1249, cues 5 and 6 apply random-pitch operand 100, start keys 176/170/156, then release after 285 ticks; cues 8 and 9 start three voices, kill after 250 ticks, then end, with later voice records unreachable. Pack 1249 cue 12 starts three voices, delaying the second by 65 ticks; cue 13 starts two voices, delaying the second by 40 ticks. Pack 1381 cues 7 and 8 start three and four voices respectively, then end, making later voice records unreachable. Pack 1398 cue 7 starts two voices with the second delayed 85 ticks; cue 8 does the same with a 180-tick delay. Stored cue defaults are volume 120 except cues 8/9 of the shared block (110); the called cue entries carry their recorded flags and command counts, but the meanings of their extra byte and flag values remain open. Voice key/volume/sample-byte fields and all command ordering are asset-specific; waveform identity and audible timbre are not established. These commands run only when the registered animation window is crossed; registration itself starts no sound.
- Test vectors (synthetic; an object with F = 31 keys, so D = 2.0 s):
  - Slot (cue 5, a 0.25, b 1.0): start 0.5, end 1.99998. A tick with p = 0.49, q = 0.51 plays cue 5. A tick with p = 1.99, q = 2.0 (play-once clamp) stops cue 5.
  - Looping, p = 1.99, q = 0.0067 (wrap): lo 0.0067, hi 1.99, so the start at 0.5 fires again.
  - a = 0: start 0.00002. The first tick from 0 (p 0, q 0.01667) plays it.
  - a = 1.0: never plays.

### Weapon hits on the world

Confidence: High for the mapping; Medium that it is unintended (it relies on the debug cue names matching the sounds).

A weapon or shot hitting the world plays a weapon-slot cue chosen from the surface. For collision faces, by face material code: 0, 6, 12, 16 give 6; 1, 13, 14, 15, 17 give 15; 2 gives 13; 3, 4 give 8; 5 gives 10; 7 gives 12; 8, 11 give 14; 9, 18, 19 give 7; 10 gives 11; others give 6. For grid objects, by their material byte: 1, 3, 10 give 6; 2 gives 8; 4, 8, 11, 16, 18 give 7; 5 gives 15; 6 gives 14; 7 gives 13; 9, 14 give 25; 12, 13 give 20; 15 gives 19; 17 gives 26; others give 6. These numbers follow the wand block's cue order (6 stone, 7 wood, 8 metal and so on). On Tor's sword block the same numbers name other cues (6 slash, 7 spin, 8 dash, 13 hit metal, 14 hit flesh, 15 hit muck and so on), so Tor's world hits play mismatched sounds. Reproduce.

### Driver volume, pan and pitch

Confidence: High.

**Sound-block voices.** Inputs: request volume V (1,024 = 1.0; the voice volume from Cue playback and Positional volume), cue volume Vc (cue entry byte 0), instance level C = clamp(127 + channel offset, 0, 127) (127 unless a type-4 command changes it), voice level Lv (voice byte +1, or the selected variable; negative clamps to 0), instance pan P (the request pan, or the cue's default pan at entry byte +2 when the request pan is -1 or -2, plus the instance pan offset) and voice pan Pv (voice word +4, or variable x 360 / 127).

- A = floor(V x 127 / 1024); B = trunc(Vc^2 / 127); D = trunc(Lv^2 / 127).
- X = trunc(trunc(trunc(B x 258 x D / 127) x C / 127) x A / 127). X = 0 is silent. X is capped at 32,766.
- theta = (P + Pv) mod 360. q = theta + 90 when theta < 270, else theta - 270. Pan table T(i) = (trunc(16383 cos(i/2 degrees)), trunc(16383 sin(i/2 degrees))) for i = 0 to 179 (equal-power).
- q < 180: L = trunc(X x T(q).first / 16383), R = trunc(X x T(q).second / 16383).
- q >= 180 (rear half): L = trunc(X x T(q - 180).second / 16383), R = trunc(X x T(q - 180).first / 16383), and the weaker channel (R when R < L, else L) is phase-inverted (negated) for a voice whose previous outputs were both non-negative; a channel already inverted stays the inverted one. A global mono mode sets L = R = X; a stereo mode without inversion skips the negation. The default mode is stereo with inversion.
- Volume group: cue entry byte +1 names a group (0 on disc). For a group g below 15, each level v becomes trunc(min(v, 32766) x f / 1024), where f = trunc(M' x d^2 / 1024 / 65536), M' = M when M is 0 or 65536 and M^2 / 65536 otherwise; the defaults M = 65536 and d = 1024 give f = 1024 (no change). Where M and d are set is Open question 54.
- The SPU2 voice volume register is the 16-bit pattern of the level shifted right by 1 (a 15-bit signed level, full scale 16,383).

**Bank (MIDI) voices.** The same step, with A = the object master volume (linear), B = velocity x channel volume / 127, C = the directory group volume (+1) and D = the voice volume (voice +0x01), with B, C and D on the squared curve (Medium-High for the order of truncation), and theta = channel pan + object pan offset + directory group pan (+2) + voice pan (+4), in degrees.

**Pitch.** Note value N, in 1/128 semitone = key x 128 + fine + pitch offset + bend term, where the bend term is trunc(down x bend x 128 / 32768) for bend < 0 and trunc(up x bend x 128 / 32767) for bend >= 0; down = voice byte +8 and up = voice byte +9 (bend ranges in semitones). Note = trunc(N / 128), note fine = N - 128 x note. Sound-block instances start with key 60, fine 0, pitch offset 0 and bend 0; command types 27 and 29 set the bend. The pitch register is 4096 x 2^(((note - centre) x 128 + note fine + centre fine) / 1536), where centre = |voice byte +2 read as signed| and centre fine = voice byte +3; 4,096 is 48 kHz. When byte +2 is below 128 the result is further multiplied by 44100/48000. Bank voices use the same rule with key = the MIDI note. Disc check: with fine added, 1,335 of the 4,125 block voices with the top bit set give a round native rate (595 of them exactly 16,000 Hz), against 519 if fine were subtracted.

**Noise voices and flags.** Voice flag bit 0x8 (sound blocks only) makes a noise voice: noise on, noise clock = clamp(signed byte +2 + bend x 32 / 32767, 0, 63), and no pitch or sample address is set. Flag bit 0 turns the effect (reverb) send on for both sides; bit 0x10 removes the voice from the dry mix. Voice byte 0 is the voice priority given to the allocator (bank voices pass their byte 0 to the same allocator argument).

Test vectors:

- Block voice, V = 1024, Vc = 120, C = 127, Lv = 110, theta = 0: A = 127, B = 113, D = 95, X = 21,808, L = R = 15,419, registers 7,709.
- V = 1024, Vc = Lv = C = 127: theta 0 gives (23,168, 23,168); theta 270 gives (32,766, 0); theta 90 gives (0, 32,766); theta 180 gives (-23,168, 23,168).
- V = 768 (emitter at 50 units, R 200), Vc 120, Lv 110, theta 0 gives (11,534, 11,534). V = 512, Vc 100, Lv 127, theta 45 gives (3,819, 9,221). Vc = 0 is silent.
- Pitch: byte +2 = 0xB0 (centre 80), byte +3 = 124, key 60 gives 1,364 (about 15,989 Hz); byte +2 = 0xC4 (centre 60), key 72 gives 8,192; byte +2 = 60 (top bit clear), key 60 gives 3,763 (a 44.1 kHz sample).
- Note: key 60, bend +32767, up 2 gives note 62 fine 0; bend -32768, down 2 gives note 58; bend 16384, up 1 gives note 60 fine 64.

**Pan from controller 10 (High).** A controller-10 value v becomes the channel pan angle 270 + trunc(90 v / 64) for v < 64, and trunc(90 (v - 64) / 63) for v >= 64. The channel pan is 0 (front) when a child starts. Test vectors: 0 gives 270 (left), 32 gives 315, 63 gives 358, 64 gives 0 (front), 96 gives 45, 127 gives 90 (right).

**Pitch bend (High).** A 0xEn event with 14-bit value b = lsb + 128 x msb sets the channel bend to trunc(b x 65535 / 16383) - 32768. Every voice of that channel and sound is re-pitched at once. A child starts with bend 0 on all channels. Test vectors: b 0 gives -32768, 4096 gives -16384, 8192 gives +1 (not 0), 12288 gives 16386, 16383 gives 32767.

**Note value at a MIDI note-on (High).** Key = the MIDI note; fine = 0; bend = the channel's current bend; pitch offset = the sound's pitch offset (the start request's pitch argument, changed later only by a set-pitch command; the game's music start passes 0). N = 128 x key + 0 + pitch offset + bend term (bend term as in Pitch above). Test vectors: key 60, up 2, b 12288 gives note 61 fine 0; b 8192 gives note 60 fine 0; b 0 with down 2 gives note 58.

**Voice ranges (High).** The game sets voice range 0 to voices 0 to 23 and range 1 to voices 24 to 47. Sound-block voices allocate from range 0 (the range number is cue entry byte +1, 0 on disc). The bank music allocates from range 1 (bank object record byte +0x13, 1 in every bank).

**Voice allocator (High for the rule).** A request carries a range, a priority p (voice byte +0x00) and a lock flag (voice flag bit 0x20).

1. The first voice of the range, in index order, whose state is free is taken. Its start time, range, priority and lock are recorded.
2. Otherwise a candidate is chosen. It starts as the range's first voice. Each later voice that is neither reserved nor locked becomes the candidate when its priority is lower than the candidate's, or equal with an earlier start time.
3. The candidate is stolen (its owner is told and its sound ends) only when it is not locked, its priority is at most p, its priority is below 126 and it is not reserved. Otherwise the request gets no voice, and that note or voice plays nothing.

- Quirk: the range's first voice becomes the initial candidate without the locked and reserved tests, so a locked or reserved first voice that stays the candidate blocks stealing.
- Reserved voices are taken out by an explicit reserve call. Streams take voices with priority 127 and the lock.
- Test vectors (synthetic, a range of 3 busy voices): (p 0, t 5), (p 0, t 3), (p 8, t 1) with a request at p 0: the voice with t 3 is stolen. All three at p 8 with a request at p 0: no voice.

**Cue instance limit (cue entry byte +5; High for the rule, Medium for the compared volume field).** When cue flag 0x8 is set, byte +5 is the maximum number of simultaneous instances of that cue. Instances being stopped are not counted.

- Below the limit, the play goes ahead.
- At the limit with flag 0x10: if the quietest instance (the first in list order among equals) has a lower request volume than the new play, it is stopped and the new one plays.
- Otherwise, with flag 0x20: the oldest instance (or that quietest one, when 0x10 is also set) is stopped and the new one plays.
- Otherwise the new play is refused.
- Disc: byte +5 is non-zero exactly when flag 0x8 is set. 514 cues are limited: 268 with flag 0x8 alone (refused at the limit), 36 with 0x10 (limits 1 to 3) and 210 with 0x20 (limits 1 to 4, and one cue with 13). Block 347 cues 0 to 11 (footsteps) have limit 4.
- Voice flag bit 0x8 (noise) is a separate field of the voice record; it still selects the noise voice (Noise voices and flags).
- Test vectors: limit 2 with flag 0x20: a third play stops the oldest instance. Limit 2 with flag 0x10 and instances at volumes 500 and 700: a new play at 600 stops the 500 one; a new play at 400 is refused.

### Not part of the sound system

Confidence: High.

- The per-frame level ramp from 0 to 128 with modes 0, 1 and 2, once listed here as a channel level ramp, is the screen-effect object (fade alpha and letterbox bars). It is specified in SPEC-0052 and has nothing to do with sound.
- The routines once listed here as the voice list (0x17C-byte records), the mixer sample conversion, the stream pump conversion, the crossfade, the stereo gain split and the stream chunk size all belong to the linked Bink movie library. They run only for movie audio, and the clean side's Bink-compatible decoder replaces them. Their facts are in SPEC-0008 (Movie audio internals).
- The routine once listed here as the channel value setter is the music register setter: SPEC-0009 (Where the music runs).

### Handle stop with retries (Medium)

1. A missing handle reference does nothing and returns 0.
2. If the handle has no sound attached (or its sound id is unset), each call adds one to the handle's retry counter. Once the counter has reached 3, the next call releases the handle and returns 1; before that the call returns 0. So a stop is retried up to three times (Low on the exact count).
3. If a sound is attached, the stop is sent once, when the sound manager is on and the handle has not been marked as stopped. One of two stop routines is chosen by a test on the sound (the test is not identified). The handle is then marked as stopped, and the call returns 1.
4. A caller may ask for a value stored on the handle to be kept unchanged across the call.

### Music containers

MMID containers hold 2, 3, 5 or 6 music sequences. A music track does not select one child. The track value (design table 4, SPEC-0024) selects a whole sound bank: the bank whose archive resource is the value plus 10 (SPEC-0009, "Music selection"). Starting the bank's music sound always starts child 0, the setup segment; its command message starts the companion children and jumps to child 1, and from then on the command messages in the children (section 8c) decide which children play, using the registers and the global excite that the game sets. The templates found on disc and the game's inputs are in SPEC-0009 (Interactive segments; Intensity inputs).

### Where the music runs

Confidence: High.

The sequencer, the command interpreter, the 16 registers per instance and the global excite live in the sound driver on the I/O processor, clocked at 240 ticks per second (section 8d). The game program only sends commands to it: start bank sound 0 with a volume; set volume; set register (instance, register 0 to 15, value 0 to 127); set global excite (0 to 127); and set reverb (core mask, effect type, effect return volume, with delay and feedback 0). The driver clamps register numbers to 0 to 15 and values to 0 to 127. A clean side may run the sequencer in its own audio thread with the same rules.

Game-side decisions for bank music (R-0021, High): the game chooses the bank and the child from the music row (SPEC-0009, Music rows and the bank choice), and the volume of the playing bank, which the bank volume step calls A. A is the value the game sends: min(127, trunc(V x 127 / 1024)), with V = row Level x Music gain (SPEC-0009, Volume and ramps). The game re-sends it on each tick while the volume ramps (15 per tick, or 20 for the outgoing bank), so a change during play reaches the driver as it happens. The object record's default volume (+0x10) is a separate field and is not found to feed this send (Medium). A stopped child's voices are keyed off and fade through their own release phase (Stopping a child), and a sample of the "other" class loops its last frame (Loop and one-shot classification); both follow the original driver and a rewrite should do the same to sound alike.

## Edge cases and error handling

- 5,242 directory records are empty groups (count 0) whose pointers do not land on a voice record. Their pointers are never read, and they must not be treated as voices.
- The sentinel 0x310 appears in 202 directory records. In 200 of them the group is empty (count 0). The other two (banks 24 and 91) have count 9 with the sentinel pointer, and read nine voice records that no other group covers. This is an original-data quirk (Open question 4).
- A program number at or above the bank's directory count stops the driver with error code 0x4E.
- The note path keeps at most 32 voices per note. A group is read only up to its count.
- A cue call with a negative cue id, a bank slot outside 0 to 24, or an empty bank slot plays nothing and is not an error.
- A sample id can have several voice records with different tuning, key windows or envelope words. Each group plays its own voices by the note-on rule above. Whether any other rule applies between records is not known (Open question 7).
- The sample pool is fully tiled. A sample length that is not a multiple of 16, or a frame with filter above 4, is a malformed bank.
- The sound block's voice table can be far from its command list (for example 347: command list at 0x940, voice table at 0x2CF0). Do not assume they are adjacent.
- A cue id at or above the cue entry count of its block plays nothing.
- **Kind-1 emitter just outside its hearing radius**: with a sphere radius of 30 or more, a listener between the radius and the radius plus 1 passes the gate but is out of range, so nothing starts (Placed sound emitters) (High).
- **AddAnimObjSound start fraction 1.0**: never plays (6 sites in module 1090); a loop wrap re-fires starts inside the wrapped interval; RemoveAnimObjSound does not stop a playing voice (Animated-object sounds) (High for the rules, Medium-High for the effects).
- **Voice allocator, locked or reserved first voice**: it stays the initial candidate and blocks stealing for the whole range (Voice allocator) (High).
- **Cue at its instance limit**: without flag 0x10 or 0x20 the new play is refused and nothing sounds (Cue instance limit) (High).
- **Stop retries.** Once a handle has no sound attached, its stop calls count retries; the call after the third retry releases the handle and returns 1 (Medium).
- **Negative reverb type**: a reverb send with a negative effect type passes that type on unchanged, because the clamp has no lower bound (Reverb send clamp; SPEC-0009, Reverb).
- **Keep-alive animation cue on a block not in state 1**: nothing starts, and no error is raised (Keep-alive animation cue).
- **Instance record with a count word below 1**: its list pointer (+0x2C) is set to 0 at load instead of being rebased (Not sound: instance chunk fix-up).
- **Instance pointer on a chunk of another type**: the pointer is only rebased; nothing is skipped and no node tree is relocated.
- **Music sequence errors**: a 0xAn status, an unlisted status or an unknown meta type stops that sequence; no such event occurs in the music on disc (High).
- **Empty music loop**: a second end of track in the same sequencer tick stops the child instead of looping again (High).
- **Start of a playing child**: a start command (0x12) for a child that is already playing has no effect; it does not restart it (High).
- **Save-point cue emitter**: the save-point cue 0xA8 is started with no emitter position for zero-flag save points (all save points on disc), because the emitter comes from the other branch's unset index (Save-point cue).
- **Voice-slot cue ids of 12 or more**: the hero voice blocks have 12 cues, so voice-slot calls with cue ids 12 and up play nothing. Consumable use sends 35 to 38 (Tor) or 59 to 62 (Roberta) on the voice slot, and projectile and enemy-strike hits send 20, 22 or 28 to the victim's voice slot (High).
- **Dropped hero-voice stingers**: every gameplay hero voice event also requests a stinger of index 19 or more (Tor k, Roberta k + 24), which the stinger player drops; no stinger sounds with a hero voice (High).
- **Hero animation sounds**: the hero's animation-sound rule always reads cue 511 (the hero action tables carry no sound word), so it never plays (High).
- **Cues 190 and 191 on the weapon slot**: sword and wand blocks have 31 and 29 cues, so these play nothing (High).
- **Roberta's creature hit** plays wand cue 14 (wand hit on water), not 9 (wand hit on flesh) (High for the id; Medium that it is unintended).
- **Rope-gun reel loop (cue 185)**: the start passes no emitter and the refresh passes one, so the refresh never matches the voice; the voice ends 4 updates after its start and the next tick starts it again, so the loop restarts about every 5 ticks (Medium). Reproduce.
- **Tor's weapon hits on the world** play mismatched sword cues, because the surface-to-cue mapping follows the wand block's order (Weapon hits on the world).
- **Cue 172** (projectile impact) is never chosen; Tor without armour hit by a projectile plays 170 (High).
- **Looped positioned voice out of range**: it keeps its last volume and pan instead of stopping (Medium).
- **Stereo side follows the camera's yaw**: because the listener rotation is applied in the inverse direction, a fixed world sound swaps between the left and right channels as the camera turns through 90 degrees (Positional volume and pan) (High for the arithmetic).
- **Start pan from a stale matrix copy**: a cue started during the world update gets its first pan from a reused stack copy of the camera rows; the next per-pass update corrects it (Medium).
- **Shield ids 176 to 180 on Tor** and **Mote ids above 173** give blocks above 490; nothing is queued and those slots play nothing (Sound slots and their blocks).
- **Equipment change**: cues on the weapon, shield or power slot are silent until the new block is loaded (Sound slots and their blocks).

## Test vectors

Synthetic (not from the disc):
- Listener rotation: camera facing world +z (rows right (1, 0, 0), up (0, 1, 0), forward (0, 0, 1)); emitter 10 units along right: u = (10, 0, 0), pan 269, heard left. Camera facing world +x (rows right (0, 0, -1), up (0, 1, 0), forward (1, 0, 0)); emitter 10 units along right, world offset (0, 0, -10): u = -10 x forward = (-10, 0, 0), pan 89, heard right. The same camera with the emitter 10 units ahead, offset (10, 0, 0): u = 10 x right = (0, 0, -10), pan 359 (front). The dot-product form would give camera-space (10, 0, 0) and pan 269 in both right-side cases.
- Save-point cue (synthetic): the nearest save point is at squared distance 100 on tick 1 and no 0xA8 voice is playing, so cue 0xA8 starts on the main slot. On tick 2 (squared distance 90) the same voice is found (same slot, cue and emitter) and refreshed; no second voice starts. At squared distance 225 exactly, no start and no refresh (the test is strict).
- Pool-extent rule: three voice records with sample ids 5, 9, 2 and pool offsets 0x0000, 0x01A0, 0x0330, and pool length C = 0x04A0. Samples: id 5 = 0x1A0 bytes (26 frames), id 9 = 0x190 bytes (25 frames), id 2 = 0x170 bytes (23 frames). Their order by offset is 5, 9, 2.
- One-shot pattern, 4 frames with flag bytes 00, 00, 01, 07: last 0x07, second-last 0x01, so one-shot. Playback stops at the frame with flag 0x01.
- Loop pattern, 4 frames with flag bytes 00, 04, 02, 03: last 0x03, second-last 0x02, a bit-2 frame earlier, so loop. Playback repeats from the bit-2 frame.
- A 0xC0 frame followed by 15 zero bytes: a silent frame. With zero prediction history, decoding it yields 28 zero samples.
- Cue call, manager on, slot 3 active, cue 146, volume negative: the cue starts at the default volume, 1,024 at Sound-effects gain 1.0 and 768 at 0.75. At gain 0 the default is 0 and the cue is refused.
- Cue call with slot 25: nothing plays. Cue -1 on slot 0: nothing plays. Cue call with the "do not restart" flag set while the same slot and cue already play: nothing new starts.
- Breakable contact cue (synthetic materials; SPEC-0036, Contact routines and cues): material 2 plays cue 143; 3 plays 145; 4 plays 146; 0, 1 and 5 play 144, each on the main slot at the breakable's position. Roberta's shot on a breakable with material 4 plays cue 144.

- Centre and fine (synthetic): tuning byte +0x02 = 0x88 gives centre note 120; 0xE0 gives centre note 32. Tuning byte +0x03 = 0x7E gives centre fine 126, added in the pitch rule.
- Note-on (synthetic): a voice whose envelope words are 0x80FF and 0x9FCC starts with exactly those two words written to its envelope settings.
- Key window (synthetic): a voice with window 58 to 62 plays for notes 58 and 62, and not for 57 or 63.
- Bank select (synthetic): value 10 selects the first loaded bank whose MIDI bank number is 10. Value 11 matches no bank on disc.
- Music child header (bank 10, child 0): first event at +0xE8, tempo 300,000, PPQ 480. Its setup message sets registers 0, 1 and 2 to 0, starts children 2 and 3, and jumps to child 1.
- Music note-off (bank 10, child 1): a note-on on channel 1, note 38, velocity 102 at MIDI tick 1, and a 0xD1 event with note 38 at tick 1916, which ends that note.
- Macro (synthetic): the command bytes 0B 04 16 00 01 11 04 F7 define macro 4. Then 0C 04 with register 0 = 1 jumps to child 4; with register 0 = 0 it does nothing.
- Group (synthetic): 0F 00 00 00 00 31 01 00 31 F7 defines group 0 on G with channels 0 and 1 both unmuted for G 0 to 49. Then 10 00 with G = 30 unmutes channels 0 and 1; with G = 50 it mutes them.
- Skip (synthetic): 16 02 01 13 00 01 with register 2 = 2 leaves register 0 unchanged; with register 2 = 1 it sets register 0 to 1.
- Running status (synthetic): delta 00, 91 3C 64, delta 10, 3E 64 gives two note-ons on channel 1 (notes 60 and 62), the second 16 MIDI ticks after the first.
- Timing (T = 4,170): tempo 300,000 gives rate 667; three deltas of 480 wait 72 sequencer ticks each (carry -24, -48, then -72); a delta of 1 after that runs in the same tick (carry 28). Tempo 500,000 gives rate 400, and a delta of 480 waits 120 ticks (carry 0). Tempo 759,493 gives rate 263 and beats of 183, 182 and 183 ticks.
- End of track (synthetic): a child with repeat count 0 reaches end of track; the count goes to -1, is reset to 0, and the child restarts at its first event in the same tick.
- Clip extents (synthetic, resources 144 and 145): data size 0x20 at offset 0 gives a next clip at 0x800; data size 0x7B0 at offset 0 gives a next clip at 0x800; data size 0x7E0 at offset 0 gives a next clip at 0x1000.
- Handle stop, synthetic: a handle with no sound attached, counter 0. Four calls in a row return 0, 0, 0 and 1; the fourth releases the handle.
1. Reverb send clamp (synthetic): core mask 1 with effect type 6 sends type 4. Core mask 2 with type 3 sends 3. Core mask 0 with type 6 sends nothing. Core mask 1 with type -2 sends -2.
2. Keep-alive animation cue: word 0x80002101 (group 1, index 1, cue 4, bit 31 set) uses table index 4, block 351; with that block in state 1 it starts cue 4 as a keep-alive voice at the character, and with state 0 or 3 nothing starts. Group 2 with index 5 uses table index 53, block 400. Group 0 with index 2 uses table index 2, block 349.
3. Instance chunk fix-up (synthetic): load base 4096, record with count word 1 and +0x2C = 64 gives +0x2C = 4160; the same record with count word 0 gives 0. A pointer +0x10 = 32 whose target header has type 12 and size 48 becomes 4096 + 32 + 16 + 48 = 4192.
4. Separate +0x14 routine (synthetic; Low, no sound caller known): a type 12 chunk with base id 140 and a 16-bit field of 3 starts cue 143 and sets the state word to 3. A type 13 chunk does the same. A type 5 chunk sets the state word to 0.
5. Sound block random source (from the stored initial state on disc; see section 7, Medium): the first four draws are 1173, 53403, 52352 and 35341.
6. Sound block cue (synthetic): a cue entry with command count 0 plays nothing. A cue id equal to the cue entry count plays nothing. A record whose handler returns -1 runs again at the next update; a record whose handler returns 0 hands over to the next record.
7. Sound block cue 140 of block 347 (disc values, decimal): 6 command records. Record 0 is type 27 with byte 0 = 100. Record 1 is type 25 with byte 0 = 2 and byte 1 = 2. Records 2 and 3 are variant 0 (voice offsets 10,840 and 10,864; delays 0 and 15 ticks). Records 4 and 5 are variant 1 (voice offsets 10,888 and 10,912; delays 0 and 15 ticks). One variant plays. Default volume 120.
8. Sound block cue 143 of block 347 (disc values): 4 command records. Type 27 (byte 0 = 100), then three type-1 records at voice offsets 10,984 (delay 0), 11,008 (delay 5) and 11,032 (delay 10). Default volume 110.
9. Sound block cue 185 of block 347 (disc values): 6 command records. Record 4 is type 43, so record 5 is never reached.
10. Sound block cue 0 of block 490 (disc values): 6 command records. Type 1 at voices 0, 24, 48 and 72, then type 43 at record 4, so the type-1 record at record 5 (voice offset 96) is never reached. Default volume 125.
11. Sound block cue 0 of block 348 (disc values): 4 command records. Type 27 (byte 0 = 75), type 25 (byte 0 = 2, byte 1 = 1), type 1 at voice offset 0, type 20 (no-op). Default volume 95.
12. Hero voice (synthetic): Tor, max health 100, hit for 6: voice cue 1 or 0 on block 348, stinger request 20 (dropped). The same hit for 5: request 19 (dropped).
13. Hero voice (synthetic): Tor charged attack: voice 6, then 8; stinger requests 28 and 30, both dropped. Roberta attack step 2: voice 5 with wand cue 3; request 51 (dropped).
14. Hero voice (synthetic): Sound effects 75 and Voice 0: every hero voice cue is called with the default cue volume 768 and plays; emote clips play nothing (volume 0).
15. PlayHeroSound (synthetic): Tor selected, PlayHeroSound(1, 0, none, 0) plays nothing; PlayHeroSound(-1, 0, none, 0) plays cue 0 of block 348 at the hero.
16. Footsteps (synthetic): run on ground code 2 gives cue 50; walk on code 6 gives 21; slow walk on code 9 gives 5 with dust entry 9; code 12 gives no cue; the same integer cursor frame on two ticks gives one cue.
17. Bank voice playback (disc values, decimal): bank 10, directory group 7 (9 voices, group volume 127, group pan 0), a note-on of key 66 with velocity 100 on a channel with channel volume 100, channel pan 0 and master volume 127. The key lies in one voice's window (65 to 67): the group's voice 2, with priority 0, voice volume 100, centre note 78, centre fine 88, pan 0, bend ranges 0 and 0, envelope words 0x80FF and 0x9FCC, flag word 1 (reverb send on), pool offset 35,136 and sample id 13. The sample is 3,072 bytes (192 frames, 5,376 samples); its last frame has flags 0x07 after a 0x01 frame, so it is a one-shot. Its file offset is 5,904 + 35,136 = 41,040 and its sound-memory address is 35,136 + 20,544 = 55,680. Pitch: note value 8,448 gives note 66, fine 0; exponent = ((66 - 78) x 128 + 88) / 1536 = -0.94271; register = trunc(4096 x 2^-0.94271) = 2,130, so the sample plays at 48000 x 2130 / 4096 = 24,961 Hz (5,376 samples last 0.215 s). Volume: B = trunc(100 x 100 / 127) = 78, then the squared curves give B = 47, voice 100 gives D = 78, group 127 gives C = 127, A = 127; X = trunc(trunc(trunc(47 x 258 x 78 / 127) x 127 / 127) x 127 / 127) = 7,447. Pan: theta = 0, q = 90, T(90) = (11,584, 11,584), so L = R = trunc(7,447 x 11,584 / 16,383) = 5,265 and the registers hold 2,632 each (centred). The voice also feeds the music's reverb at its dry level. [Medium for the truncation order of the bank volume step]
18. Child cue (disc values): block 347, a command-5 record with level 127, pan 0 and cue id 138 starts cue 138 of the same block at level 127, pan 0, when the playing instance's cue volume is not 0.
19. Stopped child (synthetic): a child has three voices sounding with ADSR2 words whose release fields are 12 (linear), 5 (linear) and 12 with the exponential flag. Command 0x03 naming that child, or the running child's 0x11 jump, keys all three off in the same tick: no later event of the child runs, each voice then fades by its own release rule (the first falls from full level in about 171 ms, the second in about 1.3 ms) and is freed at zero. A voice started by the child in that same tick before the key-on was sent never sounds. A second 0x03 naming the child does nothing.
20. Release time (synthetic): linear release, shift 11 gives 4,096 samples at 48,000 per second (85 ms) from full level; shift 12 gives 8,192 samples (171 ms); shift 10 steps by 16 each sample and takes 2,048 samples (43 ms); starting from half level halves each time.
21. Other-class sample (synthetic): 5 frames with flag bytes 00, 00, 00, 02, 07 (no end-bit frame before the last): playback runs frames 0 to 3 once, then plays frame 4 again and again; frames 0 to 3 are never replayed. Contrast with flag bytes 00, 04, 02, 03, which repeat from frame 1, and 00, 00, 01, 07, which stops after frame 2.

## Open questions

1. Resolved: voice tuning bytes (+0x02, +0x03). The centre note is the absolute value of byte +0x02 read as signed (256 minus the byte when its top bit is set), the centre fine is byte +0x03 and is added, and a byte +0x02 without the top bit also scales the pitch by 44100/48000; the pitch register is 4096 x 2^(((note - centre) x 128 + note fine + centre fine) / 1536) (Driver volume, pan and pitch). Disc check: 1,335 of 4,125 block voices then give a round native rate, against 519 if fine were subtracted (High).
2. Voice key window (+0x06, +0x07). A voice plays when the note lies between the two values, inclusive; the default window is 0 to 127. Byte +0x00 is the voice priority, +0x01 the voice volume and +0x04 the voice pan (Layout section 4). Resolved: bytes +0x08 and +0x09 of a bank voice are the bend ranges down and up in semitones (Layout section 4); a voice with 0 and 0 (28,760 of 28,860) ignores pitch bend. A voice whose down range is 244 reads it as -12, so a downward bend would raise its pitch (2 voices; Medium).
3. Resolved: envelope words and flag word. +0x0A and +0x0C are the SPU2 ADSR1 and ADSR2 register values (fields in Layout section 4). Flag bit 0 = effect (reverb) send, bit 0x10 = removed from the dry mix, bit 0x8 (sound blocks) = noise voice (High). Still open: the fixed values written by a separate start routine (Open question 44) and block-voice flag bit 0x40 (384 referenced voices).
4. Directory. Byte +0 is the group voice count (0 to 32), byte +1 is the group volume (65 to 127), and a count of 0 means an empty group whose pointer is never read. The 5,242 pointers that do not land on voice records belong to empty groups. Still open: the two sentinel groups (banks 24 and 91) have count 9 and pointer 0x310, so they read nine voice records that no other group covers. The intended pointer is not known (an original-data quirk; Medium-Low).
5. Bank +0x1C. It points to one 28-byte object record at section +0x34, a type-5 object with the bank name (section 2a). Resolved in part: the object is the bank's multi-sequence music sound, which the game starts as bank sound 0, starting child 0 of the MMID container (High). Still open: where the runtime pointer at record +0x18, zero on disc, is filled at run time.
6. Resolved: sound block entry fields. Cue entry byte +1 is the volume group and also the voice-allocator range of the cue's voices, byte +2 the default pan, and byte +5 the maximum instance count used with flag 0x8 (flags 0x10 and 0x20 choose the replacement; Cue instance limit); voice bytes +8 and +9 are the bend ranges down and up in semitones; voice byte 0 is a priority (section 7; Driver volume, pan and pitch) (High). Still open: the 693 type-1 command references that do not land on the 24-byte voice grid (Open question 48).
7. Voice records of one sample id that differ in envelope words or tuning (for example 0x80FF and 0xADFF): whether any of them play together, given that a note plays only the voices of one group whose key window contains it.
8. Resolved (Medium-High): the stream player stops after S bytes per channel, so the audible blocks past 2S are never played (section "KPV silence", Playable end).
9. Whether the original resets prediction history at silent 0xC0 blocks. Not game-visible: the sound chip decodes the frames. Default if unresolved (rewrite choice): zero output and zero history after a silent frame (section "KPV silence", History at silent frames).
10. Resolved: game sound ids map to (slot, cue id), and the slot map gives the block: main 347, hero voice 348 or 349, weapon 461 to 476 (default 466), shield 455 to 460, power 477 to 490 (default 485), area 395 + the music row's block offset, creature 350 to 394 on demand (Sound slots and their blocks) (High).
11. Resolved: MMID child event bytes and which child plays. The child header, event stream and command messages are in section 8; child 0 is a setup segment that starts the companion children and jumps to child 1, and the command messages switch children by the registers and the global excite (SPEC-0009, Interactive segments) (High).
12. Resolved: cue meanings of block 347. Every cue is labelled by the debug cue-name table, and the uses by id range and call site are in the Cue use table (Medium for the uses; 78 to 137 and several others have no call site).
13. Resolved: the contact routines on world breakables play cues 143 to 146 on the main slot, chosen by the breakable's material word (+0x5C): 2 -> 143, 3 -> 145, 4 -> 146, other -> 144; Roberta's shot always plays 144 (Breakable contact cues). These are cue ids, not effect ids.
14. Resolved: occupants of the hero voice, weapon, shield and power slots and of the area slot, with their reload rules (Sound slots and their blocks) (High).
15. Resolved: local cue ids on the hero slots index the hero voice block (12 cues), the weapon block (31 or 29 cues), the shield block (3 or 4 cues) or the power block (6 or 7 cues); their uses are listed after the Cue use table. Ids at or above the block's cue count play nothing (Edge cases) (High for the ids, Medium for the uses).
16. Resolved: the emitter argument is a pointer to the sound's source position. Distance to the listener (camera position plus 5 units along the camera's +0x20 row) sets the volume, and the emitter offset rotated by the camera matrix sets the pan (Positional volume and pan) (High).
17. Resolved: no data word holds the address of the cue routine or of the keep-alive start, and no code builds either address in a register, so neither is called through a pointer table. The direct calls are the whole set: 204 for the cue routine (119 functions; 166 fixed ids, 38 computed) and 25 for the keep-alive start (Cue use, Call counts) (High).
18. Resolved: deleted. The 0-to-128 ramp with modes 0, 1 and 2 is the screen-effect object (fade alpha and letterbox bars) of SPEC-0052, not sound (Not part of the sound system) (High).
19. Resolved: moved. This routine is movie-library code, not part of the game's sound system (Not part of the sound system); its facts are in SPEC-0008 (Movie audio internals).
20. Resolved: moved. This routine is movie-library code, not part of the game's sound system (Not part of the sound system); its facts are in SPEC-0008 (Movie audio internals). The blend divisor is n (SPEC-0008).
21. Resolved: moved. This routine is movie-library code, not part of the game's sound system (Not part of the sound system); its facts are in SPEC-0008 (Movie audio internals). The centre short-cut and the shift rule are in SPEC-0008.
22. Resolved: moved. This routine is movie-library code, not part of the game's sound system (Not part of the sound system); its facts are in SPEC-0008 (Movie audio internals). The size rule and its one caller are in SPEC-0008.
23. Resolved: moved. This routine is movie-library code, not part of the game's sound system (Not part of the sound system); its facts are in SPEC-0008 (Movie audio internals). The 0x17C-byte records are the movie library's sound-track records; the game's 48-record voice pool (Cue playback) is a separate structure.
24. Resolved: moved. This routine is movie-library code, not part of the game's sound system (Not part of the sound system); its facts are in SPEC-0008 (Movie audio internals). The priority that governs the game's voices is the driver allocator's (voice byte +0x00; Voice allocator).
25. Resolved: moved. This routine is movie-library code, not part of the game's sound system (Not part of the sound system); its facts are in SPEC-0008 (Movie audio internals).
26. Resolved: moved. This routine is movie-library code, not part of the game's sound system (Not part of the sound system); its facts are in SPEC-0008 (Movie audio internals).
27. Resolved: moved. This routine is movie-library code, not part of the game's sound system (Not part of the sound system); its facts are in SPEC-0008 (Movie audio internals).
28. **Handle family owners.** The five-slot object update (two routines), the two readiness checks, and the handle create and start routines look like the sound handle family. Their link to the cue call is not established (the former voice list is movie-library code, Open question 23). **[Low]**
29. Resolved: the channel value setter is the music register setter of SPEC-0009 (Where the music runs). The 0-to-128 ramp is the screen-effect object (SPEC-0052) and is not related (High).
30. Resolved: moved. This routine is movie-library code, not part of the game's sound system (Not part of the sound system); its facts are in SPEC-0008 (Movie audio internals).
31. Resolved: the "looping cue" is the keep-alive branch of the creature animation sound, and its lookup is the group-to-block rule of section 10: table index = block index + 0 (group 0), + 3 (group 1) or + 48 (group 2), block = 347 + table index; the cue starts only when that block's state word is 1 (Keep-alive animation cue) (High).
32. Resolved: there is no cue timer. The step is the load base of a world pack's instance chunk, added to pointers at load (Not sound: instance chunk fix-up) (High).
33. Resolved: the "accumulator" at +0x2C is the instance record's list pointer, rebased at load when the count word is 1 or more and cleared otherwise; no sound code reads it (Not sound: instance chunk fix-up) (High).
34. Resolved: **0x5040 bias.** 0x5040 is the first byte address of the driver's sample-memory free list (the driver sets up the free list from 0x5040 for 2,076,607 bytes of the 2 MB sound memory), so a pool offset plus 0x5040 is the sound-memory address of the sample when its pool is the first thing placed there. A clean side that keeps each pool in its own buffer does not need the bias, and the lengths and relative offsets are unchanged by it. **[Medium]**
35. Resolved: resources 144 and 145 are the emote clip banks. The design-table resource names both as path records with full directories, and dialogue emote requests play them through the emote-clip player (SPEC-0009, Emote clip lookup; SPEC-0024, Path records). 69 of the 358 hero clips and 873 of the 996 character clips can be reached (High).
36. Resources 144 and 145: meaning of the version field (0x20 or 0 in resource 144, 0 in resource 145), the reason the gaps repeat frames from other clips, and the bytes after the last clip (608 in resource 144, 1,520 in resource 145).
37. Resolved in part: resources 144 and 145 speakers, language and words. Lengths and counts are fixed (Frames, above): 996 clips of 0.17 to 10.7 s and 358 clips of 0.26 to 6.6 s, with the median near 1.2 s. The labels are English emotion names, which fits short exclamations and short phrases. Which clips carry spoken words, and the speaker of each voice set, cannot be told from the disc structure (Low). Default if unresolved (rewrite choice): play each clip as an opaque mono 22,050 Hz clip chosen by the directory position, with no speaker or language handling.
38. Resolved: the volume step. Block form X = trunc(trunc(trunc(B x 258 x D / 127) x C / 127) x A / 127) with A = V x 127 / 1024, B = Vc^2 / 127, C = instance level, D = Lv^2 / 127, capped at 32,766; bank form with A = object master volume, B from velocity and channel volume, C = directory group volume and D = voice volume; equal-power pan, rear phase inversion, register value = level shifted right by 1 (Driver volume, pan and pitch) (High).
39. Resolved: voice allocator. The first free voice of the range is taken; otherwise the lowest-priority voice (the oldest among equals) is stolen when it is not locked or reserved, its priority is at most the request's and below 126; otherwise the request gets no voice. Flag bit 0x20 is the lock. Sound blocks use voices 0 to 23 and the music voices 24 to 47 (Driver volume, pan and pitch, Voice ranges and Voice allocator) (High).
40. Resolved: note value. N = 128 x key + fine 0 + the sound's pitch offset (0 for the game's music) + the bend term, and the channel bend is trunc(b x 65535 / 16383) - 32768 for a 14-bit bend value b (Note value at a MIDI note-on; Pitch bend) (High).
41. Directory byte +2 and bank section +0x1A (113 to 197): meanings. The range of +0x1A equals the range of per-bank sample counts (113 to 197). A second count of the per-bank samples gives 117 to 196; the range 113 to 197 is used here. [Low]
42. Resolved: no music sequence on disc sends bank select, and a sequence plays with the instruments of the bank that holds it (section 8b) (High). At most one sound bank is loaded at a time: the music controller's bank load is the only load of a sound bank (every other slot load is a sound block, 347 to 490), staging a new bank first asks the old one to unload, and the new load waits while that unload is pending (Loading and addressing a bank) (Medium-High).
43. Resolved: pan mapping. Controller-10 value v gives 270 + trunc(90 v / 64) for v < 64 and trunc(90 (v - 64) / 63) for v >= 64 (Pan from controller 10) (High).
44. Other start routine: which routine calls the start routine that writes the fixed envelope values 0x000F and 0x1FC0 and derives pitch from a rate in hertz (pitch = rate x 4096 / 48000). It is reached only through a wrapper that is an entry of the driver's table of 107 external commands (position 62, counting from 0), not from the music or bank note path, so it is a command the game's own sound library would have to send; no music or bank sound uses it (Medium). Which game call, if any, sends that command is open. [Low]
45. Resolved: sound block voice volume formula (Open question 38, block form). The fixed value 10 selects the squared curve for the cue and voice levels (High).
46. Resolved: sound block pitch. Block instances play at key 60 with fine 0; types 27 and 29 set the bend, scaled by the voice's bend ranges (+8 down, +9 up); byte 0 is a priority, not a key; flag bits 1 and 2 are clear in every referenced voice (Driver volume, pan and pitch) (High). Still open: the base value of type 29.
47. Resolved: **Tick unit.** The SBlk command interpreter runs in the 989SND I/O-processor driver, and each delay decreases once per step of the driver's 240-per-second sound tick, so one unit is 1/240 second (Tick unit; Medium). The tick is not the 60-Hz game pass; it is the same rate as the MMID sequencer of section 8d, which the lock-out arithmetic fixes at 480 ticks for 2 seconds.
48. Resolved: **Second voice table.** There is none. The voice region mixes 24-byte voice records with 16-byte channel records (command 4) and 32-byte child-cue records (command 5); the 693 off-grid type-1 operands (711 over commands 1, 5 and 9) are voice records that follow the shorter or longer ones, and the 8- and 16-byte remainders of 25 blocks are the same mix (Voice table and pool). **[High]**
49. **Random source start.** The location of the initial 250-entry table and index in the sound driver module, and whether the driver restarts the generator when it starts. [Open]
50. Resolved: the cue call's two float arguments are the hearing radius R and the full-volume radius r0. They are used only when R is above 0; otherwise R = 200 and r0 = 0. Every direct call site passes 0.0 in both, except the two placed-emitter routines, so the default applies to all other cues (Cue playback; Placed sound emitters) (High).
51. Resolved: hero animation sounds. No hero animation entry carries a packed sound word: all 1,158 entries of the six hero action tables name no block and cue 511, so the hero's main-slot animation cue rule never plays anything. No townsfolk or voiced character table carries one either (640 records, 0 words); animation sounds come only from creatures (Creature blocks and animation sounds). [High]
52. Resolved: camera-matrix direction for the pan. The listener rotation is the inverse (camera-to-world) of the world-to-camera rotation, so the stereo side of a world sound depends on the camera's yaw: an emitter on the camera's right is heard on the left with the camera facing world +z or -z and on the right with it facing world +x or -x (Positional volume and pan, Direction of the listener rotation). [High for the arithmetic; Medium for the screen side of row 0]
53. Resolved: kind-1 placed emitters are the type-11 sphere primitives in cell list A whose header flags have bit 0x10 (216 records in 12 packs); the old activity test is that flag bit. AddAnimObjSound is a separate path (Placed sound emitters; Animated-object sounds) (High). Still open: whether every flagged sphere is listed in cell list A of every grid cell it covers (Medium-High).
54. Volume groups: where the group master values M and d are set (defaults 65536 and 1024, no change). [Medium-Low]
55. Resolved: special-victim creature kinds. The kind is the s16 at +0xAC of the character record: 240 is the Black Knight (block 365, cue 34), 480 the Undead Draik (391, 1), 405 the Dark Knight (355, 15), and 1941, 1942, 1949 and 1950 the Skeleton Fighter, Skeleton Wizard, Skeleton Archer and Dumb Skeleton (361, 22). Each cue is that block's "impact special" cue and no animation table uses it (Creature blocks and animation sounds). [High]
56. Resolved: AddAnimObjSound cues per site. Each module's cues index the sound block of its own pack's music row; all 145 slot fills are in range, 139 name playable entries, and the 6 fills of module 1090 name the empty cue 0 of block 419 (Animated-object sounds, Cue resolution per module) (High).
57. Resolved (R-0021 1; High): when a child stops, by command 0x03, by the running child's 0x11 jump, or at its own end, its voices are keyed off and fade through their own release phase; they are not cut at once and not left to end. Voices not yet keyed on are cancelled, and a paused child's voices are the one case that is cut (Stopping a child). The clean side's choice to release them is correct.
58. Resolved (R-0021 2; Medium-High): the "other" sample class loops. Its last frame (flag 0x07) carries the loop-start bit itself, so the repeat address is that last frame and the voice repeats that one frame (28 samples) until key-off and the end of its release; the 0x02 frame before it has no effect. No driver code sets a different repeat address (Loop and one-shot classification).
59. Resolved (R-0021 3; High; see Game-side decisions for bank music): the bank volume step's A is the object master volume, and the value the game sends for it is min(127, trunc(V x 127 / 1024)), with V = row Level x Music gain. It is not the object record's default volume (+0x10) and not a product of the two. The game re-sends it while the volume ramps, so a change during play reaches every voice of the bank through the master volume term of the volume step. How the driver applies it internally is the driver's own mixing and is out of scope.
