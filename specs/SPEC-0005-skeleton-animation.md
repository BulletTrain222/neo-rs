# SPEC-0005: Skeletons, animation and skinning

- Answers: R-0005; R-0039 (in part: carry offset owner and the damage-bit choice for the hero's carry); R-0109 (Roberta's two secondary-motion chains, outfit 512 bind-pose geometry, and the absence of a separate hair/cap child model or distinct nonzero visibility branch in outfits 512–519; anatomy, external generic attachments, rendered visibility and the rendered head gap remain open; see SPEC-0040 Behavior 25); R-0126 (corrected rest pitches, axes and signs, write-back into the sub-part joint records, rest pose and worked example for the mode-2 chains; SPEC-0040 Behavior 25); R-0124 (class-17 play rate is 1.0 times the slot speed word; speed byte 2 is 4 ticks per key; the speed byte comes from the clip the character's own table names for the current id); R-0127 (the tag-0x0A range table is a path list, not a clip table); proactive (combat core pass 3); proactive (boss close); proactive (open close-out)
- Status: Draft
- Disc build: SCUS-97367 v1.00 (same fingerprint as SPEC-0001)
- Confidence: per section. Bone table, skinning, plain and compressed animation layout, pose math: High (checked end to end, see Behavior). Speed rule and the speed-byte counts: High for the expression and counts; the play rate R: High for its expression, Medium for the per-action values. Fraction source: High. Playback records, layers, clip start, crossfade and the blend factor: High. Sub-part and face indexing: High. Mirrored clips: High for the flag and the partner field, Medium for the visual effect. Per-bone translation scale: High for the read, the counts, the application to bosses and the size of the Plague Serpent, Medium for its purpose. Clip-start resets: High; no reset on loop wrap: High; stop and seek: open. Pose rate limit: High for its constants, signed branch operand and one-way behavior. Cursor, end-of-clip and key-pair rules: High for the arithmetic, Medium-High for the disc check. Sampling: High for the rotation and translation blends; nothing normalises rotations: High for the quaternion-to-matrix step, Medium-High for the other steps. Root record: Medium-High for its layout; the root step (root record of key k times the clip speed): High; its use as the travel of heroes and of other characters: High (SPEC-0040, Behaviors 14C and 14D); the sample and root scale 0.000977: High; body 526 table-A root travel: High; the accumulator arithmetic: High, its consumer open. Hero locomotion root records: High. Bone hurt-capsule fields (+0x30, +0x40, +0x44): High for the layout and the disc counts. Table B frame bits: High for the bit test, the role and reader of every bit, the four-record query and the disc counts; Medium for the multi-frame rule (SPEC-0036); High for the held-item draw gates; Low-Medium for the character inventory and attachment details. Events (the pair list is never used): High. Clip progress: High. Mode-0 entries: High for layout and key pair. Weapon frame blocks: High for the selection, writer and attachment-frame identity (any character); Medium for their use. Attach points and carrying: High for selection, pose-entry frame math, and default Miamouse carry-angle initialization; Medium-Low for any later writer and the separate optional-offset composition. Object clip time and mode: High. Joint pose sample: dormant (High). Sub-part and face mirroring: unreachable on the shipped data (Medium-High). Skinning selector: Medium. Random idle picker: High for the draw ranges and the miss rule; Medium for the mode meanings and the timer. Compressed blocks: High for the selector, the decode, the expanded layout and the counts; Medium-High for expansion at clip start; the bytes left after the channel stream: High that they repeat the previous clip's block in the headerless banks, Medium that nothing reads them. Headerless clip banks: High for the bank layout, the entry chain and the channel walk; the skeleton header fields Medium. Hero animation data: High for the resource numbers, counts, table size, index rule and sample rows; Medium for the body-to-table binding. Resource 526 as the shared townsfolk clip bank of character records with main-family value -7: High for the users, the index range, the rig check, the table counts, the id-to-clip rows, the timing rule, the absence of bone-0 travel and the clip-start range test; Medium for the clip-group labels. Forward kinematics (parent-times-local, root from the supplied matrix): Medium; joint-index upper guard (indices 83 and above return before output): High; largest shipped skeleton 74 joints: High; active parent-table source, full-word root comparison and null-root mechanics: High; the known character callers supply a root matrix: High; malformed out-of-range parent/count behavior and possibility of an unknown null caller: Low-Medium (Open questions). Per-bone bounds: Low. Action-id table and word A fields (counts, lookup, missing-id rules): High. Word B as a packed sound word (block group, block index, cue id, keep-alive; SPEC-0031, Layout 10): High for the layout and for bits 20–30 (the cue start frame). Word A bit 13 (head-look allowed): High for the reader, Medium for the look target. Turn toward a direction: High. Hero action ids by role: High for the roles listed and for the second main id of the air attacks. Animation-set lookup diagnostic: Medium-High. Pose entries as attach points: High for the role and the bone-local transform order; Medium for the separate Miamouse carry offset fields and runtime angle source. Secondary-motion chains: Medium for the Verlet integration, the gravity, the rest-length constraint and the constants; Medium-Low for the chain field layout and the joint rebuild of mode 0; High for the mode selection and the users of each mode; High for the formulas of modes 1 and 2 (SPEC-0040, Behavior 25).

## Summary

Every skinned model carries a **bone table**: parent links and a rest pose of translations only, with no rest rotations. Skinned vertices are stored in that rest pose. Each vertex names up to three bones by index, with weights. **Animations** come in sets, one set for each of three families: main body, sub-parts and face. An animation entry holds a frame count, a loop flag, a bone mask and, per animated bone, a rotation quaternion and a translation stored as 16-bit fixed-point values, one set per frame. At runtime each playing clip has a cursor measured in frames. The game samples the clip at the cursor and blends linearly between the two nearest keys (translations linearly, rotations component by component after a sign flip, with no spherical blend). A character's player has four playback records (main layers 0 and 1, face, sub-part); every clip start fades the new clip in over a few ticks from the pose shown last (Playback records and layers; Clip start; Combining a sample into the pose). It then builds each bone's world transform down the hierarchy and multiplies by the inverse of the rest position to get the skin matrix. Character clip cursors are in frames and advance in keys per tick. Playback 1a gives the rate: at 60 ticks per second, R = 1.0 and S > 0 gives 30 / S keys per second; S = 0 gives 30 keys per second. Character clips do not convert their cursor to seconds (Open questions, Frames and time). Animated characters also keep five per-weapon frame blocks, of which one is chosen each tick by the damage-window bits, and a carrier places a carried character at one of its attach points (Weapon frame blocks; Attach points and carrying). The two hero characters, Tor and Roberta, use the same clip layout in raw resources of their own, with their own action tables (Hero animation data). The heroes travel by root motion: each tick the root record of the current key, times the clip speed, is the step that moves the hero (Playback 5; SPEC-0040, Behavior 14C). Some clips are stored LZO1X-compressed and expand to the same layout (Compressed entries). With this spec the clean side can pose and skin every character for all animations on the disc, including the compressed hero clips.

## Where it lives on the disc

- **Bone table**: the flat (childless) chunk type 18 inside a model set or character outfit (SPEC-0004), 1,355 on disc.
- **Skeleton definitions**: chunk types 22 (main), 23 (sub-parts) and 24 (face), in character regions.
- **Animation sets**: chunk types 25 (main), 26 (sub-parts) and 27 (face). 566, 607 and 613 chunks hold 16,565, 20,093 and 51,475 entries.
- **Headerless resources**: seven flat clip banks (resources 493, 495, 497, 499, 501, 503 and 526, 843 entries) and six flat skeleton headers (resources 492, 494, 496, 498, 500 and 502). See Headerless resources (Layout).
- **Animation tables** (which clip to play for an action): type-36 records of kind 2 (SPEC-0006). There are 2,804 such tables in the 575 character regions (see Choosing an animation). The hero tables are not type-36 records; they are listed below.
- **Hero action tables**: raw resources 520 to 525 (Tor main, sub-part and face; Roberta main, sub-part and face), 2,048 bytes each. See Hero animation data.
- **Secondary-motion constants**: three 32-bit floats in the boot program's data at 0x2EC874 (blend 0.1), 0x2EC878 (gravity −0.075) and 0x2EC87C (gravity scale 1.0). See Secondary-motion chains.
- A region's chunks of types 22 and 25, 23 and 26, and 24 and 27 are numbered in parallel: the *k*-th type-22 chunk pairs with the *k*-th type-25 chunk, and so on.

## Layout

All values are little-endian. "Payload" is chunk + 16 + E (SPEC-0002).

### Bone table (flat chunk type 18)

| Offset | Size | Type | Meaning |
| --- | --- | --- | --- |
| 0x00 | 16 | 4 × f32 | (0, 0, 0, 1) |
| 0x10 | 16 | 4 × f32 | (0, 0, 0, 1) |
| 0x20 | 4 | u32 | Offset of the bone records = 0x40 |
| 0x24 | 4 | u32 | Offset of the pose-entry table = 0x40 + n × 0x50 |
| 0x38 | 2 | u16 | Bone count *n* (7 … 74) |
| 0x3A | 2 | u16 | Pose-entry count = 9 |
| others | | | Fill; ignore |

The payload length is exactly 0x40 + n × 0x50 + 9 × 0x30 (1,355 of 1,355).

A chunk of type 18 is the flat form when its payload does not start with an IDM chunk. The other form is the model-set wrapper of SPEC-0004.

Bone record *i* (0x50 bytes at 0x40 + i × 0x50):

| Offset | Size | Type | Meaning |
| --- | --- | --- | --- |
| 0x00 | 2 | u16 | Bone index = *i* |
| 0x04 | 2 | u16 | Number of children |
| 0x08 | 4 | u32 | Offset of the parent's record, relative to the first record (so parent index = value / 0x50); 0xFFFFFFFF = root |
| 0x0C | 4 | u32 | Offset of the first child's record; same encoding |
| 0x10 | 16 | 4 × f32 | Rest local translation (x, y, z) relative to the parent; w is usually 1 (see Edge cases) |
| 0x20 | 16 | 4 × f32 | Rest position in model space: root = its local translation; child = parent's rest position + its local translation (59,411 of 59,411, within 2e-6) |
| 0x30 | 16 | 4 × f32 | Hurt-capsule base point (x, y, z, 1), bone-local; (0, 0, 0, 1) in most records |
| 0x40 | 4 | f32 | Hurt-capsule length. Above 0: the bone is a hurt capsule; 0 or below: no capsule |
| 0x44 | 4 | f32 | Hurt-capsule radius |
| 0x48 | 8 | | Fill, not used |

Parents always have a lower index than their children. Each table has exactly one root.

Hurt capsules (High for the layout and the counts). The fields at 0x30, 0x40 and 0x44 are the bone's hurt-capsule fields: base point, length and radius. They do not affect posing. The combat tests pose each capsule from the bone's current world frame and test it as a finite cylinder from the posed base point along the bone's local Y axis (SPEC-0036, Hit detection, Hurt shapes). On disc, 5,093 bone records have a length above 0, in 776 of the 1,355 bone tables; the other 579 tables have no capsule, so those characters cannot be struck by melee or shots. The most common size is length 0.367 with radius 0.367 (424 bones). Example: in resource 504 (58 bones), bones 0, 1, 2, 3, 24, 25, 27 and 30 are 0.367 by 0.367, and bone 4 has length 0.823 and radius 0.431.

Pose entry (0x30 bytes, 9 per table): u32 index *k* at 0x00, u32 bone index at 0x04, then three angles in radians at 0x10 (x, y, z; w = 1) and an offset at 0x20 (x, y, z; w = 1). Entries 0 and 1 are usually mirror images (x negated) on deep limb bones. These entries are the skeleton's attach points 0 to 8 (a negative bone index means none); they are not needed for posing. A carrier places a carried character at one of them (Attach points and carrying; High for the role).

### Skeleton definitions (chunk types 22, 23, 24)

| Offset | Size | Meaning |
| --- | --- | --- |
| 0x00 | 4 | u32 record count *N* |
| 0x14 | 4 | u32 K = floats per bone in the per-bone extra block: 0, 1, 2 or 4 in chunks (0, 4, 8 or 16 bytes per bone), 28 in header 492 (type 22 and main headers only; 0 elsewhere) |
| 0x18 | 4 | u32 offset of the records = 0x30 |
| 0x2C | 4 | u32 end of the records = 0x30 + 12N |
| 0x30 | 12 × N | Records: u32 parent (0xFF = root), u32 mirror partner, u32 offset into the extra block (type 22) or 0 |

After the records, type-22 chunks may hold an N × K block of floats (163 of 563 chunks): the per-bone translation scale sets (Per-bone translation scale). The second record field is the joint's mirror partner (Mirrored clips). In a main definition the parent is a main joint index. Types 23 and 24 list the joints that sub-part and face animations drive; a parent p below N_main (the main skeleton's count) is main joint p, and p at or above N_main is element p − N_main of the same list (Families).

### Animation set (chunk types 25, 26, 27)

| Offset in payload | Size | Meaning |
| --- | --- | --- |
| 0x0C | 4 | u32 entry count *E* |
| 0x24 | 4 | u32 offset of the first entry = 0x30 |
| 0x28 | 4 | u32 **expansion buffer size**: 0 when no entry of the set is compressed; otherwise the largest expanded size *U* among the set's compressed entries plus 0x3C (an entry header). It holds in every set on disc: 763 of the 2,075 animation set chunks and 5 of the 7 headerless banks (below) have a non-zero value, the other 1,312 chunks and 2 banks have 0 (High). The loader copies it into the set holder; when the character's animation player is bound to the set and the value is non-zero, a buffer of that many bytes (4-byte aligned) is allocated for the player. A clip start with compression flag 1 expands into that buffer (Compressed entries). The clean side may allocate by this value, or by the largest *U* it meets plus 0x3C (Medium-High for the allocation link) |

Payload bytes 0x00 to 0x0B are zero in all 2,075 chunks and all seven banks and are never read. Entries form a chain. Walk exactly *E* entries; the last entry's next-link is not valid.

Entry header (offsets from the entry start; the three offsets in it are payload-relative):

| Offset | Size | Type | Meaning |
| --- | --- | --- | --- |
| 0x00 | 8 | u64 | **Bone mask**: bit *b* set means the entry animates bone *b* of the bone table |
| 0x10 | 4 | u32 | Animated bone count *nb* = number of set bits in the mask (88,121 of 88,133) |
| 0x14 | 4 | u32 | Frame count *F* (0 … 146) |
| 0x1C | 4 | f32 | 1.0 (the clip's full weight) |
| 0x21 | 1 | u8 | Root flag: when non-zero, starting the clip clears a pose word. 0 on every clip on disc |
| 0x24 | 1 | u8 | Fade-in length in ticks (1 to 30 on disc; 4 on most; see Speed byte S and fade length) |
| 0x25 | 1 | u8 | Fade-out length in ticks (1 to 20 on disc). Read only by fade-out starts that nothing calls, so it has no effect in the shipped game |
| 0x26 | 1 | u8 | 0xFF on every clip; not read by playback |
| 0x27 | 1 | u8 | Loop flag: zero = one-shot, non-zero = looping |
| 0x28 | 1 | u8 | Main layer index (0 or 1) that a main clip plays on. 0 on every clip on disc (88,133 chunk entries and 843 bank entries) |
| 0x29 | 1 | u8 | Evaluation mode (0 or 2 on disc; see Mode 0 and Mode 1) |
| 0x2A | 1 | u8 | Speed byte S (Playback 1a; 0 to 4 on disc) |
| 0x2B | 1 | u8 | Compression flag (see Compressed entries) |
| 0x30 | 4 | u32 | Offset of table A (F × 8 bytes) |
| 0x34 | 4 | u32 | Offset of table B (F × 4 bytes, from entry + 0x3C; see Encoding) |
| 0x38 | 4 | u32 | Offset of the next entry |

The channel stream starts at table A's offset + 8F.

**Plain channel stream**: 7 channels per animated bone, bones in increasing bit order of the mask. For each bone: rotation x, y, z, w, then translation x, y, z. Each channel is one of:

- **Constant** (16-bit flag 0): 12 bytes. Bytes 0–1 = 0; bytes 4–5 = s16 value; bytes 2–3 and 6–11 are not needed by the walk. The channel has this value in every frame. In the seven headerless banks (80,526 constant channels), bytes 2–3 are 0 in every constant channel; the u16 at bytes 10–11 is a per-bank tag (0 in banks 493, 495, 497, 499 and 503, 129 in bank 501, 113 in bank 526; the u32 at bytes 8–11 has this tag as its high half); the u16 at bytes 6–7 is 0 in 76,911 of them and a small value otherwise. The walker treats a channel as constant when the u32 at bytes 0–3 is 0; it reads the s16 value at bytes 4–5 and moves on 12 bytes. Bytes 6–11 are never read: they are padding, and the per-bank tags have no effect (High).
- **Animated** (16-bit flag 1): 4 bytes (u16 1, u16 0), then F × s16 samples, padded with unused samples to a multiple of 4 samples (so the channel takes 4 + 2 × ((F + 3) rounded down to a multiple of 4) bytes).

Value = sample × 0.000977 (s16, so about −32 … +32), with no per-channel scale. The scale is the single-precision constant 0.000977 (bit pattern 0x3A800EAE), about 0.045 % larger than 1/1024; the clip samplers use it for channel samples and for root records alike, so 1024 gives 1.000448, not exactly 1.0 (High). The data facts in this section (unit quaternions, equal samples) hold with either scale; values quoted below as sample ÷ 1024 are data values, not runtime values. There are no key times: an animated channel stores one sample for every frame, and the only reduction is the constant channel. There is no delta coding: every sample is an absolute value, and each frame's quaternion is unit length on its own (High). In the seven headerless banks, 80,526 of 142,212 channels (56.6 %) are constant; of the 61,686 animated channels, 9,688 (15.7 %) have all samples equal and are still stored in full. The stream of a plain entry ends exactly where the next entry starts. 40,961 of 88,133 entries are plain under this rule: 15,845 main, 12,228 sub-part and 12,888 face. In every plain entry, every bone's four rotation channels form a unit quaternion (norm within 0.01) at every frame. That holds for all 18.8 million bone-frames.

**Compressed entries.** Byte 0x2B of a clip entry is the compression flag: 0 means plain (as above); 1 means the entry's table-B position holds a compressed block. Every flag value seen is 0 or 1. Compressed entries are counted across all places they occur: 47,048 top-level clip chunks, 3,847 clip chunks nested inside type-32 containers, and 148 entries in the seven headerless clip banks (see Headerless resources), for 51,043 in total. The counts above (88,133 entries and 40,961 plain) cover the top-level chunks only.

For a compressed entry, the bytes at the table-B offset are a block:

| Offset in block | Size | Meaning |
| --- | --- | --- |
| 0x00 | 4 | u32 *U*: expanded size in bytes |
| 0x04 | 4 | u32 *C*: compressed size in bytes |
| 0x08 | *C* | LZO1X compressed data (the public LZO1X format, ending with the standard end-of-stream marker) |

Expand to exactly *U* bytes; the decoded size must equal *U*. The result takes the place of everything after the entry header, laid out as for a plain entry with its base at the table-B position: table B (F u32) at offset 0, table A (F × 8 bytes) at (table-A offset − table-B offset), and the channel stream (7 × nb channels) at that position + 8F. A plain entry has the same layout in its own bytes. The compression is the standard, generic LZO1X format; it is not a key-reduction or delta scheme. The clean side may use any conforming LZO1X implementation.

Decoding gives exactly *U* bytes and consumes exactly *C* bytes in all 51,043 blocks. The decoded bytes hold the same layout as a plain entry. All 51,043 decoded streams give 7 channels per animated bone, and every bone's first four channels form a unit quaternion at every frame (tolerance 0.02).

**Block-size count (High).** Across the 50,895 compressed entries in type-25/26/27 chunks (47,048 top-level and 3,847 nested), every compressed payload fits inside its chunk, has *C* < *U*, and has *U* ≥ 64 bytes. The other 148 compressed entries are the seven headerless banks; their separate count also has no block with *C* ≥ *U* or *U* < 64. Thus all 51,043 shipped compressed animation blocks are smaller than their expanded data and at least 64 bytes after expansion.

After the 7 × nb channels, some decoded bytes remain: 9,134 blocks (8,430 top-level, 684 nested, 20 headerless). The channel walk does not read these tails. In the headerless banks every tail (20 of 20) is a byte-for-byte copy of the end of the previous entry's expanded block. In the type-25/26/27 chunks, 9,081 of 9,114 tails (8,400 top-level, 681 nested) likewise match the previous entry's expanded-block suffix; the 33 nonmatches (30 top-level, 3 nested) are repeated instances of one identical expanded type-27 mode-2 record (11 bones, 28 frames, 1,312 expanded bytes) with a 52-byte tail. This narrows the exceptions to one replicated data pattern but does not explain the tail's meaning or use. Do not treat all trailing bytes as proven copies or clip fields. That no other evaluator reads them remains Medium.

**When the block is expanded (Medium-High for the trigger, High for the copy).** Expansion happens when a clip starts playing, not at load; the loader only relocates the entry's offsets. At clip start, an entry with flag 0 is used in place. For an entry with flag 1, if the playing instance has a working buffer, the 0x3C-byte entry header is copied to the start of that buffer, the table offsets are shifted to the copy, and the block is expanded into the buffer right after the copied header, so the expanded table B starts at copy + 0x3C. The buffer is per instance, so each playing instance works on its own expanded copy. If the instance has no buffer, the original uses the raw compressed entry, which would not walk (Open questions). The boot path also decodes two blocks held in the executable's data, which are not clip entries.

**Decoder note.** The original has two LZO1X decoders. The one used for clips and boot data checks only the end of the input. A second, bounds-checked decoder exists in the executable but is dormant: nothing calls it, no data word holds its address and no address is built for it, and no level module imports it (High). Its LZO1X compressor neighbour is dormant in the same way. Clip blocks are expanded when a clip starts, not when its resource loads; no load expands a whole resource (SPEC-0002). A clean implementation needs only the standard LZO1X decoding rules with the size checks above.

Test vector (synthetic): a compressed clip in a resource loaded at pass 0 and first played at pass 40 is expanded at pass 40, at its clip start; a clip that is never started is never expanded.

**Mode 0.** Entry byte 0x29 is an evaluation mode: 2 for almost all entries (the channel stream above). 97 main-family entries (all uncompressed) use **mode 0**, a fixed per-frame layout:

- Frame *f* starts at the table-A offset + f × (16 × nb + 8). There is no channel stream and no per-frame header. The entry spans exactly F × (16 × nb + 8) bytes from table A (97 of 97).
- Per animated bone, in mask bit order, 16 bytes: rotation x, y, z, w as four s16 ÷ **4096**, then translation x, y, z as three s16 ÷ **4096**, then 2 bytes of padding (0). Every quaternion is unit length (norm error at most 0.0002).
- Then one 8-byte root record: four s16, read as in Encoding. x, y, z × 0.000977 are small root-motion values. The fourth value is unscaled; across the 97 mode-0 entries it is nonzero and varies from 2484 to 4096 (119 distinct values across 5,102 frames). Preserve it as stored when reproducing the four-lane root accumulator; its semantic meaning remains open (Open questions).
- Sampling follows Playback 4 and Layer step, with the same key pair (*k*, *k*₂), wrap flag and blend factor as mode 2 (High).

**Mode 1.** Evaluation mode 1 is a packed-rotation layout in the original. No disc entry uses it (0 of 88,133), and its layout is not specified here.

### Encoding: table A, table B and the loop flag

- **Table A** (F × 8 bytes) holds the **root record** of each key in the mode-2 channel layout: four s16 values, of which the first three (x, y, z) are multiplied by the sample scale 0.000977 (not exactly 1/1024). The fourth input value is not used by the mode-2 root sampler. It is nevertheless populated in shipped data: among 99,204 mode-2 entries with at least one frame (1,998,991 key records), 1,594,586 fourth-lane values are nonzero, with 33 distinct raw values from 0 to 30,676. Preserve this lane when losslessly copying records, but it is not required to reproduce the mode-2 sampled root from the basis here. A static consumer survey found no additional reachable playback reader for this input lane; three auxiliary animation-data summary routines call the shared root converter, but the executable and level-module scans found no incoming reference to those routines. The channel stream starts at table A + 8F, so table A's location and size are needed to find the bone channels. Table A is also the root source for root motion (Playback 5), and its root record is needed for the root of a complete pose. For the heroes it is the travel source: x is the side, y the up and z the forward displacement per key in the character's frame (Hero animation data, Locomotion root records).
- **Table B** occupies F × 4 bytes starting at entry + 0x3C (the header offset is normally that position). The sampler reads it as an event list: a pair count at entry + 0x18, then (frame, value) pairs. The pair count is 0 on every entry on disc (88,133 of 88,133, and 843 of 843 in the headerless banks), so the event list is never used from raw data, and table B does not affect posing on disc (Playback 7). Table B's F values are not all zero: in the headerless banks they are nonzero in 54 of 161 Tor main clips and 41 of 152 Roberta main clips, with values such as 128, 256, 1024 and 2048 at particular frames. The layout (F u32, one per frame) is High. The table-B offset equals entry + 0x3C on 88,128 entries; the other 5 are type-27 entries, cause open.
- **Table B frame bits** (High for the bit test and the disc counts; Medium for the multi-frame rule). Apart from the pair list, the per-frame u32 values are read directly by frame index as event bits. Bits 0x1, 0x2, 0x4, 0x8 and 0x10 are the attack windows of weapons 1 to 5: weapon N is active on a frame whose value has bit 2^(N-1) set. The heroes' attack-active word is bit 0x1 (SPEC-0036, Hit detection, Attack window); monster weapon tests read the bit of each weapon (the weapon collision lengths and radii of SPEC-0029). The same five bits choose the weapon frame block (Weapon frame blocks), and bits 0x2 to 0x10 choose a carrier's attach point (Attach points and carrying). A query covers the character's four playback records (main layers 0 and 1, sub-part and face): a bit set on any of them counts. It uses the multi-frame and loop rules of SPEC-0036 (Hit detection, Attack window): with a per-tick frame step above 1 it ORs the frames stepped over; when the whole part of the cursor equals F on a clip whose loop byte is exactly 1, it takes the first frame AND the last frame (the loop byte is 0 or 1 in all 843 bank clips); and a record past its clip gives nothing. Over 17,425 clips (the type-25 clips plus the hero and shared banks), bit 0x1 is set in 1,105 clips (5,705 frames), 0x2 in 415, 0x4 in 649, 0x8 in 148 and 0x10 in 91. Bits 0x20 to 0x1000 also occur (0x80 and 0x100 in about 2,160 clips each). They are not attack windows; their roles and readers follow.
- **Table B frame bits above 0x10** (High unless stated). Each bit has one role and a fixed set of readers:
  - 0x20, release event: a projectile, rope or boss spawn fires on a frame with this bit; each such routine fires once per armed cycle. Readers: the boss and enemy release steps, Roberta's shot release and the hero's rope gun (its fire id 167 carries the bit on the fire frame).
  - 0x40, cancel window: read only by hero state tests (state exits and chained actions, for example the state-20 exit of state 13 and the state-4 exit of state 19; SPEC-0036 and SPEC-0033 Behavior 3C).
  - 0x80 and 0x100, footsteps: one bit per foot. The hero's footstep cue and dust (SPEC-0031; SPEC-0034) and the monster's shake and rumble (SPEC-0029 Behavior 32) fire on them; nothing else reads them.
  - 0x200, sound: no reader; it has no effect.
  - 0x400, effect: read only by the Werelupe King's summon animation (id 26; SPEC-0019). On every other character, Tor's attack clips included, it has no effect.
  - 0x800, recover: the hero's opening after a swing. Enemy counter-attack trigger codes 1 to 3 wait for it (SPEC-0038 Behavior 14). Only the hero reports it; the same query on any other character reports none.
  - 0x1000: while it is set on a frame of a region character's clip, the renderer may enter the held-item-1 draw branch, subject to its selector and non-null model gates (High for the gates; visible use is not established).
  - Bits above 0x1000 do not occur.
  - Disc counts (hero banks): Tor's 161 main clips carry 0x1 in 14 clips, each of 0x2 to 0x10 in 1, 0x20 in 4, 0x40 in 12, 0x80 in 38, 0x100 in 37, 0x200 in 1, 0x400 in 11 and 0x800 in 8 (the clips of attack ids 14 to 20 and 27). Tor's id 31 (clip 63, F 33) carries every bit from 0x1 to 0x800, a test-like clip. Roberta's 152 main clips carry 0x1 in 5, 0x20 in 13, 0x40 in 2, 0x80 in 30 and 0x100 in 28, and no other bit; her attack clips carry 0x20 (shot release) and never 0x800, so trigger codes 1 to 3 never fire against Roberta. Body 526 carries only 0x80 and 0x100, in one clip each.
  - Data quirks (High): the soft landing (id 68) and the boosted launch (id 133) have no 0x40 frame in any family of either hero, so the windows that test 0x40 on those ids never open (the chained jump and L1 window of the soft landing, and the run-cancel of the boosted launch; SPEC-0033). Roberta's hard landing (id 118) has 0x40 only in its sub-part and face clips; the query covers those records, so its window opens. Roberta's hurt (id 32, clip 59) is a looping clip of 44 ticks per cycle at R = 1, where Tor's (clip 48) is a one-shot of 36 ticks, so a gate on progress 0.99 ends her hurt at the 0.99 point of its first cycle.
- **Loop flag**: entry byte 0x27 is zero for a one-shot clip and non-zero for a looping clip. It is non-zero on about 47,700 of 88,133 entries (the exact count depends on the set counted).

### Speed byte S and fade length

The speed byte S (entry byte 0x2A) is the divisor in Playback 1a. Its values on disc, over 88,133 entries:

| S | Entries | Share | Speed at R = 1.0 (frames per tick) |
| --- | --- | --- | --- |
| 3 | 57,525 | 65.3 % | 1/6 |
| 2 | 29,324 | 33.3 % | 1/4 |
| 1 | 767 | 0.9 % | 1/2 |
| 0 | 479 | 0.5 % | 1/2 (no division; see Playback 1a) |
| 4 | 38 | 0.04 % | 1/8 |

Confidence: High for these counts. The fade-in length (entry byte 0x24) is 1 to 30 on disc, and is 4 on 77,431 entries (7,414 of type 25, 19,313 of type 26, 50,704 of type 27). The fade-out length (byte 0x25) is 4 on 6,932, 19,335 and 50,709 entries of the same types. Body 526: fade-in 4 on 112 clips, 5 on 38 and 8 on 4, with the same fade-out lengths. Hero walk and run clips: Tor fade-in 16 (fade-out 5), Roberta fade-in 4 or 5 (fade-out 8). The fade step is the full weight 1.0 divided by the fade-in length (Clip start; High).

### Table size rules (loader)

When the loader reserves memory for animation tables and bone records, it uses three size rules. Each result is rounded up to the next multiple of an alignment value A that the caller passes in:

| Block | Size before rounding | Note |
| --- | --- | --- |
| Table of N 8-byte entries with a u32 count (as in "Choosing an animation") | 8 N + 4 | Matches the table layout exactly |
| N records of 0x50 bytes after a 0x10-byte header | 0x50 N + 0x10 | The record size matches the bone record; the block's identity is not confirmed |
| Fixed block | 0xE0 (224) | Size of a fixed-size record; its owner is not confirmed |

Synthetic check: N = 3 and A = 16 give 8 × 3 + 4 = 28, rounded to 32 for the table.

Confidence: High for the arithmetic. Which table each rule serves is open (see Open questions).

### Skinning upload blocks

Confidence: High for the record fields, the block table and the sizes; Medium for the matrix element type; Low for the meaning of the two result arrays.

A skinning job record holds these fields (offsets in the job record):

- +0x18: address of the bone-matrix array. It holds one 3x4 matrix per bone, 48 bytes per bone (three rows of four 32-bit floats; Medium), stored consecutively in bone order across all blocks.
- +0x3C and +0x40: two result base addresses. A selector word (the run-time word at 0x2EC830, zero in the disc image) chooses one of them for the whole job (Skinning upload).
- +0x48: the number of blocks.
- +0x4C: the address of the block table.

The block table has one 16-byte entry per block, as four 32-bit fields:

| Offset | Meaning |
| --- | --- |
| +0 | n, the number of bones in the block |
| +4 | offset A, added to the chosen result base for the first result array |
| +8 | offset B, added to the chosen result base for the second result array |
| +12 | not used |

Per block the job reads 48 n bytes of bone matrices and writes two result arrays of n 16-byte vectors each.

### Headerless resources (clip banks and skeleton headers)

Confidence: High for the clip bank layout, the entry chain and the channel walk; Medium for the skeleton header fields.

Thirteen resources have no IDM chunk header. Their payload starts at byte 0 of the resource, so the offsets in this section are resource offsets. Seven are clip banks and six are skeleton headers. Twelve are the family bodies and headers of Tor and Roberta (Hero animation data). The thirteenth, clip bank 526, is a shared main-family clip bank for other characters (Hero animation data, Body 526). The six hero action tables (520 to 525) are also raw.

**Clip banks** (resources 493, 495, 497, 499, 501, 503 and 526):

| Offset | Size | Type | Meaning |
| --- | --- | --- | --- |
| 0x00 | 12 | — | Zero in all seven banks; never read |
| 0x0C | 4 | u32 | Entry count *E* |
| 0x10 | 20 | — | Zero in all seven banks; never read |
| 0x24 | 4 | u32 | Offset of the first entry (0x30 in all seven) |
| 0x28 | 4 | u32 | Expansion buffer size, as in Animation set: 10,952 (493), 0 (495), 0 (497), 19,412 (499), 4,840 (501), 9,112 (503) and 20,328 (526). 0 exactly in the two banks without compressed entries; elsewhere the largest *U* plus 0x3C (High) |
| 0x2C | 4 | u32 | Zero in all seven banks; never read |

The bank header is exactly 0x30 bytes and entry 0 starts at 0x30. The words at 0x30 and 0x34 are therefore not header fields: they are the 64-bit bone mask of entry 0 (the first row of the entry header). Its value is 0x00000000FFFFFFFF in the 32-bone banks 493, 499 and 526, 0x0001FFFF00000000 in bank 501 and 0x00001FFF00000000 in banks 495, 497 and 503, which matches the masks of Sub-part and face indexing (sub-part and face bits start at bit 32). A bank reader must not take them as separate fields.

| Resource | Entries *E* | Bones per entry *nb* |
| --- | --- | --- |
| 493 | 161 | 32 |
| 499 | 152 | 32 |
| 526 | 154 | 32 |
| 501 | 121 | 17 |
| 503 | 102 | 13 |
| 495 | 89 | 13 |
| 497 | 64 | 13 |

The seven banks hold 843 entries in total. Every entry of a bank has the same *nb* as the bank.

Each entry uses the entry header and channel layout of the Animation set section, with its offsets taken from byte 0 of the resource. Walk exactly *E* entries from the first entry, following the next-link at entry + 0x38; the link of the last entry is not followed.

- **Compressed entries.** 148 of the 843 entries have compression flag 1: 18 in 493 (Tor main), 10 in 499 (Roberta main), 1 in 501 (Roberta sub-part), 9 in 503 (Roberta face) and 110 in 526. Every other entry has flag 0 and is plain. For every entry, the table-B offset (entry + 0x34) equals entry + 0x3C and the table-A offset equals table B + 4F, so the LZO1X block of a flag-1 entry starts at entry + 0x3C. All 148 blocks expand to exactly *U* bytes and consume exactly *C* bytes (*U* from 2,716 to 20,268, *C* from 716 to 10,466), and the expanded layout is as in Compressed entries. The evaluation mode byte (0x29) is 2 in all 843 entries.
- **Channel walk.** For all 843 entries, the plain or expanded stream gives 7 × *nb* channels with only flag values 0 or 1, and every bone's first four channels are a unit quaternion at every frame (largest norm error 0.00082; tolerance 0.02). Each animated channel has F samples, and tables B and A have F entries each. All 695 plain entries end exactly at the next entry; for the last entry of a bank, the end is its next-link target (the record after the bank, which is not a clip). 128 of the 148 compressed entries end exactly at *U*. The other 20 have bytes after the channels, which repeat the end of the previous entry's expanded block and are ignored (Compressed entries).
- **Clip index.** Each bank in a slot is turned into an array of its entries in chain order. A clip is named by its 0-based position in that array, and an index is valid only when it is below *E*. For the hero banks, word A bits 0 to 11 of the hero action table of the same family give the index (Hero animation data). For body 526, the index comes from word A bits 0 to 11 of the tables of the character records that use it (Body 526).
- **Loader pairing.** Each hero family pairs its header with its body: 492 with 493 (Tor main), 494 with 495 (Tor sub-part), 496 with 497 (Tor face), 498 with 499 (Roberta main), 500 with 501 (Roberta sub-part) and 502 with 503 (Roberta face). Body 526 is also loaded with header 492, into a fourth global main-family set (Body 526). Bones per entry match the header bone counts for these pairings: 32 bones (492 and 498 with 493, 499 and 526), 17 bones (500 with 501) and 13 bones (494, 496 and 502 with 495, 497 and 503). Other pairings are not confirmed.

**Two-window animation stream (High for the readiness rule and primary-object use; Medium for transfer lifecycle).** The main character animation stream uses two alternating 32 KiB windows. Each window has an available-record count: −1 before initialization, zero while unavailable, and a positive count when records are ready to consume. When the player is waiting (state 2), the main animation update skips that object until the other window has a positive count. It then switches to that window and submits the previous window for refill. This is a streaming-buffer wait and handoff, not a clip request or a crossfade. The known facts do not establish which specific clip banks or clip indices use this path (Open questions).

**Skeleton headers** (resources 492, 494, 496, 498, 500 and 502). Each has the layout of Skeleton definitions with no chunk header:

| Offset | Size | Meaning |
| --- | --- | --- |
| 0x00 | 4 | Bone count *N* |
| 0x14 | 4 | K, floats per bone in the extra block: 0x1C (28) in 492, 0 in the others (Per-bone translation scale) |
| 0x18 | 4 | Offset of the records = 0x30 |
| 0x2C | 4 | End of the records = 0x30 + 12*N*. The loader adds this value to the third field of each record |
| 0x30 | 12 × N | Records, as in Skeleton definitions |

| Resource | *N* | Value at 0x2C |
| --- | --- | --- |
| 492 | 32 | 0x1B0 |
| 494 | 13 | 0xCC |
| 496 | 13 | 0xCC |
| 498 | 32 | 0x1B0 |
| 500 | 17 | 0xFC |
| 502 | 13 | 0xCC |

Resource 491 is headerless in appearance and not identified (Open questions). Resources 520 to 525 are the hero action tables (Hero animation data).

### Hero animation data

Confidence: High for the resource numbers, the counts, the table size, the index rule and the sample rows; Medium for the body-to-table binding; High for the role of resource 526 (Body 526 below).

The two hero characters, Tor and Roberta, are not built from character regions. Each hero has three animation families: main, sub-part and face. A family is a header resource and a body resource, both raw (no chunk wrapper). Each hero also has one raw action table per family. No generic character region holds these resource numbers. Generic character records reach the global sets only through a negative main-family value (Body 526 below).

Resource numbers. The action table numbers are fixed values in the game's code. The family header and body numbers are loaded at run time from 16-bit data words. The main header number is at 0x313388 and the sub-part header and body numbers at 0x313390 and 0x313398. The face header and body numbers are at 0x3133A0 and 0x3133A8. Each of these tables has one entry per hero: the entry for hero index *h* is at its base plus 2 × *h*, with *h* = 0 for Tor and 1 for Roberta. The main body number is the one after the main header (493 and 499).

| Hero | Family | Header | Body | Action table |
| --- | --- | --- | --- | --- |
| Tor | main | 492 | 493 | 520 |
| Tor | sub-part | 494 | 495 | 521 |
| Tor | face | 496 | 497 | 522 |
| Roberta | main | 498 | 499 | 523 |
| Roberta | sub-part | 500 | 501 | 524 |
| Roberta | face | 502 | 503 | 525 |

**Locomotion root records (High).** In the hero locomotion clips the table-A root record is the same at every key. Both heroes have the same values except where noted. Walk (animation ids 3 and 8): z = 461 (0.4504), speed byte S = 1. Run (ids 4 and 9): z = 658 (0.6429), S = 1. Slow walk (ids 2 and 7): z = 171 (0.1671), S = 1; Tor's clip also has a side sway of x = ±31. Reverse (ids 5, 6, 10 and 11): z = −922 (−0.9008), S = 2, except Tor's ids 5 and 10 (his clip 96, 12 keys), which have z = −256 (−0.2501), S = 2 and so a step of 0.0625 per tick (3.75 units per second). Sidestep: x = ±922 (±0.9008) in most clips, S = 2. With the sample scale 0.000977 (Encoding) and the speed rule (Playback 1a) at R = 1 these give root steps of 0.225198 units per tick (13.512 units per second) for walk, reverse (other than Tor's ids 5 and 10) and sidestep, 0.3214 for run and 0.0835 for slow walk. The resulting ground speeds are in SPEC-0040, Behavior 14C. The walk cycle has 20 keys and the run cycle 14 keys; each covers 9.0 units per cycle.

Resource 526 is a fourth body of 154 clips with 32 bones. It is not a hero family: it is the shared main-family clip bank described in Body 526 below.

**Body 526, the shared townsfolk bank (High unless a line says otherwise).** The hero setup, which runs once at game start, loads header 492 and body 526 into a fourth global main-family set. No action table is loaded with it. A character record's main-family value (the signed 16-bit value at kind-3 data + 0x86; SPEC-0006) normally names a main family of the record's own region. Negative values select a global set instead: -1 means none, -5 selects the Roberta main set (hero index 1), -6 the Tor main set (hero index 0), and -7 the fourth set, body 526 with header 492 (SPEC-0040, Pools and caps). The record's own sub-part and face families are bound as usual.

- **Users.** Body 526 is used only through character records whose main-family value is -7. Neither hero uses it: each hero binds its main set by a hero main-set index word (one 5-short row per hero at data address 0x2DB422, rows 10 bytes apart), and those words are 0 for Tor and 1 for Roberta, the two hero main sets.
- 553 of the 2,402 kind-3 character records in resources 534 to 1426 have main-family value -7. They are in 266 resources, numbered 707 to 1426. All of them are non-combat NPCs (townsfolk such as guards, nobles, knights, merchants, officials, villagers, servants and children) of 26 monster types between 16 and 50. No record with any other main-family value uses these 26 types, and every one of their monster rows (SPEC-0029) has behaviour class none. There are no enemies or bosses among them.
- **Rig check.** For each of the 553 records, 32 plus the bone count of its sub-part family plus the bone count of its face family (0 when the face index is -1) equals the bone count of a mesh skeleton (flat type-18 bone table) in the same region: 553 of 553. As a control, the same sum with each record's own main family holds for 1,151 of 1,151 records with main-family value 0. So the -7 characters play body 526 on the 32-bone main rig of header 492. Their mesh skeletons have 45 to 62 bones. Resources 707 and 724, for example, hold sub-part and face families but no main family.
- The main-set state word copied from the record (data + 0x98) is 0 in all 553 records, so the diagnostic path for a main set that is not initialised (Missing ids and sets) is never taken for them.
- **Index range.** Their main tables (kind-2 records named by the record's table index, data + 0x9E) give clip indices, by word A bits 0 to 11, of 0 to 153 only: 147 distinct indices in 67,988 references, none at 154 or above. 154 is body 526's clip count *E*.
- **Main tables.** Every -7 record has a 148-entry main table, and only four distinct tables occur among the 553 records: variant A (111 records), B (198), C (101; all its users have female names) and D (143). Each variant serves a fixed set of monster types, except that type 29 records use C or D. The four variants agree for most ids and differ in the walk and run clips, some gestures and the seated set. Word A bits 12 and 13 and the effect index are unset in all 67,988 non-empty entries.
- **Id-to-clip facts** (ids are those the NPC routines request; SPEC-0040, Behavior 3C, and SPEC-0056):
  - Idle id 0 gives clip 48 in all four variants. Clips 50, 51 and 52 are byte-identical copies of clip 48; they are referenced by other filler ids, so they are not unused.
  - Walk id 1 gives clip 104 (A and D), 108 (B) or 107 (C). Walk id 8 (the walk slot after speed code 6) gives 104 (A, B and D) or 107 (C).
  - Run id 6 gives 105 (A, B and D) or 111 (C). Fast-run id 5 gives 106 in all variants.
  - Id 3 (the walk slot of movement classes 3 and 4, and the enemy-style run id) and the enemy-style walk id 2 give the idle clip 48, so these requests show no leg motion.
  - Lying ids: 97 gives clip 113 in all variants; 98 gives 112 (B, C and D) or 17 (A). Sit id 65 gives clip 119 (seated leg swing).
  - Fidget ids 115 to 119 give clips 51, 52, 102, 123 and 126. In variant B, ids 118 and 119 have no clip. Ids 141 and 142 have no clip in any variant. A request for an id with no clip starts clip 0 of body 526 (Lookup miss, under Clip start).
  - Id 30 (the timed wait animation of SPEC-0056, mode 13) gives clip 71.
- **Clip groups** (Medium for the labels; counts High). The 154 clips are 95 loops and 59 one-shots:
  - Standard idle: 4 clips (48 and its copies 50 to 52), all loops.
  - Walks and runs: 8 loops. Walk cycles 104, 107 and 108; run cycles 105 and 111; fast run 106; a step-in-place loop 109; a carry-walk loop 110.
  - Lying: 3 loops (17, 112 and 113), root height about 0.17 with the torso near horizontal.
  - Seated on the ground: 12 clips (23, 114 to 122, 137 and 143), root height about 0.22; all loop except 121.
  - Kneeling: 1 loop (124).
  - Standing idles, leans and gestures: the other 126 clips, 68 loops (idles, leans, holds and talking loops) and 58 one-shots (bows, waves, points, claps, cheers, jumps, nods, shrugs and similar gestures).
- **Root travel is in table A, not in bone 0 (High).** In body 526, bone 0 has no horizontal travel in any clip, including the walk and run cycles; the largest horizontal shift of bone 0 within a clip is 0.17 units. As for the heroes (Playback 5), the travel is in table A: its root record has the forward component z in the third s16 lane. Per key: walk cycles 104 and 107 z = 512; walk variant 108 z = 175; run cycles 105 and 111 z = 1382, with a small side sway in x; fast run 106 z = 1975; carry walk 110 z = 128; the idle clips 0. All 154 clips use the hero sampler layout (evaluation mode 2). With S = 3 and R = 1 the root steps are 0.0833707 units per tick for walk 104 and 107 (512 × 0.000977 ÷ 6; 5.00 units per second), 1.71 units per second for 108, 13.50 for the runs, 19.30 for the fast run and 1.25 for the carry walk; idle moves 0. These are the townsfolk walking speeds (SPEC-0040, SPEC-0056).
- **Timing.** At play rate R = 1 (Playback 1a and 3), a looping clip's period is 2 × S × F ticks and a one-shot reaches its last key after 2 × S × (F − 1) ticks. Examples: clip 48 (F 33, S 2, loop) has a period of 132 ticks; clip 104 (F 8, S 3, loop) 48 ticks; clip 67 (F 60, S 1, one-shot) lasts 118 ticks. Over the bank, S is 3 in 102 clips, 2 in 39 and 1 in 13, and the fade length is 4 in 112 clips, 5 in 38 and 8 in 4.
- **Unreferenced clips.** Clips 18, 54, 66, 72, 73, 74 and 135 are not named by any of the 553 tables. A reimplementation still loads them, because a clip index is a position in the chain.
- **Main-family value -6 (resources 1278 and 1279).** 16 records have main-family value -6 and so bind the Tor main set (161 clips). Their main tables have 113 entries, of which 101 are non-empty, with clip indices 924 to 1,082. Every such index fails the clip-start range test (Edge cases), so these records can never start a main clip. Their sub-part and face indices are in range for their own families and play normally. Their meshes have 52 bones, which does not match 32 plus their sub-part and face bone counts (58).

**Family header** (resources 492, 494, 496, 498, 500 and 502). The layout is that of the flat skeleton headers in Layout above:

| Offset | Size | Type | Meaning |
| --- | --- | --- | --- |
| 0x00 | 4 | u32 | Bone count *N*: 32 (main), 13 (Tor sub-part, Tor face, Roberta face) or 17 (Roberta sub-part) |
| 0x18 | 4 | u32 | Offset of the records = 0x30 |
| 0x30 | 12 × N | | Records: u32 parent index (0xFF = root), u32 mirror partner, u32 block offset (0 in sub-part and face headers; in main headers the offset of the joint's block, relative to the records' end) |

Header 492 also holds a 112-byte block per bone (28 floats), starting at 0x1B0 (the value of the records' end for N = 32): the per-bone translation scale sets (Per-bone translation scale). Sub-part and face headers have no blocks. Their parent and partner values at or above N_main (32) name elements of their own list (Families; Mirrored clips), which is why they list indices up to 39, 43 and 46 with only 13 or 17 records.

**Family body** (resources 493, 495, 497, 499, 501, 503 and 526). The layout is that of the headerless clip banks above. Field +0x0C is the clip count *E*, field +0x24 is the offset of the first clip (0x30), and every clip's bone count equals the header's *N* of its family.

**Counts per family** (every clip of the six family bodies has the bone count of its header):

| Hero | Family | Header bones | Clips *E* | Action table *N* | Ids with a clip | Ids with no clip | Largest clip index | Clips that parse as plain |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| Tor | main | 32 | 161 | 193 | 170 | 23 | 160 | 143 of 161 |
| Tor | sub-part | 13 | 89 | 193 | 150 | 43 | 88 | 89 of 89 |
| Tor | face | 13 | 64 | 193 | 152 | 41 | 63 | 64 of 64 |
| Roberta | main | 32 | 152 | 193 | 172 | 21 | 151 | 142 of 152 |
| Roberta | sub-part | 17 | 121 | 193 | 157 | 36 | 120 | 120 of 121 |
| Roberta | face | 13 | 102 | 193 | 123 | 70 | 101 | 93 of 102 |
| shared (-7 records) | body 526 (header 492) | 32 | 154 | none (character-record tables) | - | - | 153 | 44 of 154 |

For every hero family the largest clip index in its action table equals its body clip count minus one (0 exceptions in the 924 non-empty entries). For body 526 the largest index in the -7 records' tables is 153, also its clip count minus one.

"Clips that parse as plain" counts clips with compression flag 0, whose plain channel walk (Animation set section) ends exactly at the next clip. Every other clip has compression flag 1 and walks exactly after expansion (Compressed entries), so every clip of the seven bodies is usable.

Frame counts per body: Tor main 4 to 62, Tor sub-part 2 to 62, Tor face 3 to 62, Roberta main 4 to 66, Roberta sub-part 4 to 66, Roberta face 2 to 76, body 526 4 to 111.

**Action table** (resources 520 to 525). Field +0x00 is u32 *N* = 193 for all six. Entry *i* (0 to 192) is 8 bytes at 0x04 + 8 × *i*: word A and word B, laid out as in Choosing an animation. For hero entries, word A bits 0 to 11 are the clip index in the family body (0xFFF = none), bit 13 is the head-look flag and bits 14 to 21 are the effect index (Medium). Word B is the no-cue value in all 1,158 hero entries, so hero animations start no cue (High). The bytes after the 193 entries (from 0x60C to the end of the 2,048 bytes) are not used.

**Hero clip selection.** The hero's action id *i* selects entry *i* of the action table of the family being played. Ids 0 to 192 are valid. Entry word A gives the clip. An id of 193 or more, or a word A clip of 0xFFF, is a lookup miss: the lookup gives clip index 0 of that family (Lookup miss, under Clip start).

**Hero action clip speed bytes (High).** The hero action table does not contain a playback-rate value. Selection is two-step: action id *i* reads the 8-byte entry at `0x04 + 8*i` in that hero family's action table; word A bits 0–11 give clip index *c*; clip *c* in the paired family body supplies speed byte S from its entry at +0x2A. The normal slot speed is then R × 0.5 / S, or R × 0.5 when S is zero (Playback 1a). Across action entries with a clip, the unique referenced clips have these S counts: Tor main, 157 clips (S=1: 33, S=2: 124); Tor sub-part, 89 (all S=2); Tor face, 64 (S=2: 50, S=3: 14); Roberta main, 150 (S=1: 9, S=2: 141); Roberta sub-part, 121 (S=1: 5, S=2: 116); Roberta face, 98 (S=1: 4, S=2: 91, S=3: 3). Repeated action ids can reference the same clip, so these counts are by distinct clip, not by action entry. Example: Tor main action 3 selects clip 90 (S=1), while Roberta main action 3 selects clip 108 (S=1); both use speed 0.5R frames per tick. Tor main action 0 selects clip 1 (S=2), giving 0.25R frames per tick. This per-clip S is separate from the per-action rate multiplier used by some non-hero character controllers (Open questions, Play rate R).

**Hero action ids by role** (High for the roles listed; the routines that request them are in SPEC-0033 and SPEC-0036). Each item gives the main-family clip as Tor / Roberta, with F and L (loop) or O (one-shot) where it matters; "none" means no clip, so the lookup gives clip 0.

- Idle: 0 base idle (1 F30 L / 67 F33 L); 1 base idle with the weapon out (10 / 71); 35 idle while L2 is held (50 / 135). Long idles 158 to 161 (Tor 0, 2, 3, 4 / Roberta 66, 68, 69, 70; loops) and 157 with the weapon out (11 / 72). Fidgets 107 to 111 (Tor 5 to 9, one-shots / Roberta 52 to 56, loops) and 146 with L2 held (134 O / 140 L). Ids 155 and 156 have clips but are never requested. The idle picker that chooses among these is in SPEC-0033.
- Guard: 63 guard stance (46 / 21) and 119 guard (145 / 22), loops.
- Walk: 2, 3 and 4 slow walk, walk and run; 5 and 6 reverse; 7 to 11 the same with the weapon out; 106 walk on ground kinds 1 and 6. All loops, carrying the footstep bits 0x80 and 0x100 except Tor's clip 96 (ids 5 and 10).
- Strafe: side steps 98, 99, 101, 38 and 151 to 154, forward 3 and 4 (8 and 9 with the weapon), back 6 (11), turn in place 149 and 150.
- Jump: launches 70 standing (74 F6 O / 82 F6 O), 76 moving (79 / 84) and 133 boosted (86 F10 O / 88 F11 O). At progress 0.99 they continue as 70 → 67, 76 → 78 and 133 → 78. Landings: 68 soft (72 F8 O / 86 F8 O) and 118 hard (144 F9 O / 99 F11 O). Strafe jumps 102 and 104 (side) and 65 (back), with tails 103, 105 and 66. Plunge (Tor): 23 and 24 (clips 39 and 40, bits 0x401).
- Air attacks (second main id; Clip start): Roberta 19 and 20 (clips 7 and 8); Tor 25 (none).
- Hurt: 32 when a usable weapon item is in slot 1 (48 F10 O / 59 F11 L), 192 otherwise (60 F11 O / 65 F23 O). Stagger: 58 (27 / 35).
- Death: 34 (47 F17 O / 61 F17 O). On deadly ground: 136 (143 / 63, loops), 173 for ground sub-kind 2 (155 / 131) and 174 for sub-kind 8 (156 / 62). Arrival pose (hero state 59; a hover above the ground, SPEC-0055): 175 (157 / 132, loops). Get up: 137 (146 F21 O / 102 F21 O). Held: 143 (150 / 64).
- Traversal: block push and pull 40 to 42, 144 and 145; hang 59 to 61 and 71 to 73; climbing panels 43 to 58 and 121; sidle 79 to 81; sneak 82, 85 and 87; ladder 90 to 95, 120 and 135; grab point 112 and 114 to 117; climb up 75 and 134; rope gun 167, 168, 171 and 172; feeding a petpet 138; Miamouse 178 to 181 (178 and 181 carry bit 0x20).
- Attacks 12 to 31: SPEC-0036. Further ids requested by hero routines as constants (High):
  - 6: reverse walk (strafe back, state 32).
  - 12, 17, 27 and 30: attack ids (SPEC-0036).
  - 64: shield-block impact. A blocked hit plays the block cue and requests 64 (Tor clip 51, F 5, one-shot, 16 ticks; Roberta clip 23, F 12, 44 ticks).
  - 147 and 148: spin ends (SPEC-0036).
  - 97 and 98: never requested for a hero; their only requester is the NPC mode word, modes 11 and 12 (SPEC-0040).
  - In state 57 the id 12 is requested on the carried Miamouse, not on the hero (Attach points and carrying).

## Behavior

### Playback

1. **Cursor.** Each playing clip has a cursor *c* in frames. For a clip of *F* frames, the keys are the integers 0 to *F* − 1. The cursor does not use seconds in this path. At 60 ticks per second, R = 1.0 and S > 0 gives 30 / S keys per second; S = 0 gives 30 keys per second (1a; Open questions, Frames and time). A character's player has four playback records (Playback records and layers), and each record has its own clip, cursor, finished flag, blend weight and speed in frames per tick (1a).

1a. **Speed.** Each playback record has a speed in frames per tick. The speed is set when the clip starts (1c). For the heroes it is also recomputed on every tick from the current play rate R (High; SPEC-0040, Behavior 3A), so a hero's clip speed follows its stick tilt and status from tick to tick. Every other character (townsfolk, region NPCs, common enemies and all bosses, including both Darkest Faerie controllers and the Black Knight) is re-timed the same way: its playing clip's speed is rewritten once per update, after the character's controller has run, in every base state except 42 (the re-time rule table below gives who changes the rate and who does not; with R = 1.0 the rewrite equals the speed set at clip start). For a clip with speed byte S (entry byte 0x2A) and play rate R given by the request:
- If S is 0: speed = R × 0.5. The division is skipped.
- If S is not 0: speed = R × 0.5 ÷ S.

The 0.5 is a code constant. With R = 1.0 (1b), the speed is 1/6 frame per tick for S = 3 (about 65 % of entries), 1/4 for S = 2 (about 33 %), 1/2 for S = 1 or 0, and 1/8 for S = 4 (see Speed byte S and fade length). So S is ticks per key divided by 2: S = 2 is 4 ticks per key, not keys per tick. Item 2c adds the speed to the cursor each tick. Confidence: High for the expression and the entry counts.

Which clip's S is used (High): the re-time at the end of each character update takes the character's current animation id, looks it up in that character's own bound main table (word A clip field) and reads S from that clip's entry in the bound main set; the speed is written to the record on the clip's layer (byte 0x28), and the sub-part record is re-timed from its own clip's S. If the current id has no clip, the lookup gives clip 0 (Lookup miss). A clip start uses the S of the clip it starts. The skip of an action id that has no clip applies to the class-17 attack sequencer's table-checked requests (SPEC-0038 Behavior 13); a direct request for an id with no clip still falls back to clip 0 (for example, the Plague Serpent's corpse request of id 58 plays its idle clip).

Clip duration at R = 1.0 (follows from 1a and item 3; S of 1 or more): a looping clip wraps every 2 × S × F ticks, and a one-shot clip reaches its last key and sets the finished flag after 2 × S × (F − 1) ticks. Body 526 gives disc examples (Hero animation data, Body 526).

1b. **Play rate R (High for the expression; Medium for the per-action values).** For character actions, R = (character speed scale, character + 0x3D4) × (character time scale, character + 0x8B8). Both scales initialize to 1.0. The worked-out behavior code also writes 0 to the character time scale in specific controller states, and other controller paths replace it from action/state data or multiply it by a factor; the exact values and reachability of all such paths are open (Open questions). One play request passes R = 1.0, and two pass R = 0.5 (for clip index 0). For the heroes, R = (character speed scale, 1.0 at hero creation) × (a per-tick scale that the hero update resets to 1.0 at the start of every tick and then multiplies by the stick tilt factor and the status factors; SPEC-0033, SPEC-0036 and SPEC-0040, Behavior 3A). So the heroes' locomotion clips play with R other than 1.0 at partial tilt and under status effects (High). A representative shipped class-17 family sample is known: resource-530 skeleton rows 224–227 use per-slot wind-up/strike/recovery factors from 0.9–1.0, 0.7–2.0, and 1.0 respectively; the sequencer directly stores the selected phase factor to +0x8B8 before same-update clip re-time (SPEC-0038 Behavior 13). Exhaustive values across class-17 rows and other controllers remain open (Open questions).
Class 17 (High): the character speed scale +0x3D4 is 1.0 for every character (its only writers store 1.0 at character setup), and P (+0x8B8) is reset to 1.0 at the start of every NPC update, then acted on by status factors, then replaced by the attack speed words in the sequencer states that store them (SPEC-0038 Behavior 13). No monster-row field other than those three words, and no spawn-record field, scales R. Example: the Rogue (row 153) has all speed words 1.0, so its S = 2 attack clips run at 0.25 keys per tick.
For concrete non-hero values, the filled slots of the shipped skeleton family are:

| Resource-530 row | Slot | Wind-up P | Strike P | Recovery P |
|---:|---:|---:|---:|---:|
| 224 | 1 | 0.9 | 0.8 | 1.0 |
| 225 | 1 | 1.0 | 1.5 | 1.0 |
| 225 | 2 | 1.0 | 2.0 | 1.0 |
| 226 | 1 | 1.0 | 1.0 | 1.0 |
| 226 | 2 | 1.0 | 1.0 | 1.0 |
| 227 | 1 | 1.0 | 1.0 | 1.0 |
| 227 | 2 | 1.0 | 1.0 | 1.0 |
| 227 | 3 | 1.0 | 0.7 | 1.0 |

Class 17 stores the selected slot's wind-up, strike, or recovery value into character +0x8B8 during that phase, before the same-update clip re-time (SPEC-0038 Behavior 13). These factors are separate from the world +0xB8 time scale and the clip's speed byte S. Rows 224–227 are a representative sample; this is not the complete value survey for all class-17 rows. The values above do not assign visual names to clips.

A separate non-hero example is the Faerie Sisters' controller: it sets the character time scale from current health as P = 2 − HP / maximum HP. This is health-derived, not a per-action table. With base scale 1 and clip S = 2, the rate is 0.25 frames/tick at full health, 0.375 at half health, and 0.5 at zero health. This example does not establish the values or selection rules of other non-hero action tables.

Gnarfas supplies a second controller-derived example (class 14): each controller tick directly replaces the character time scale P with its animation-speed setting before dispatching the current state. The shipped setting is 1.0, so with base character scale 1 and a clip whose S is 2, the rate is 0.25 frames/tick. This is a controller-wide setting rather than a per-action table; because it is a direct replacement, it also replaces any earlier time-scale multiplier on that update. This does not establish the other controllers' table values or selection rules.

**Re-time rule table (High for the mechanism; Medium for the per-controller values).** There is one non-hero rule and no per-controller exception to it: each update of a townsfolk, region NPC, common enemy or boss character sets P to 1.0, applies the status factors, runs the controller (which may store or multiply P), and then rewrites the playing clip's speed to R × 0.5 ÷ S (S = 0: R × 0.5) with R = speed scale × P. The rewrite uses the clip of the character's current animation id and also re-times the sub-part clip from its own S. It happens in every base state except 42, which returns before it, and only for a character whose model is bound. A clip started by a request starts in that tick's animation pass at the speed of the final P of the tick; the next update re-times it. Nothing in any level script writes P. What differs between characters is only what the controller does to P before the rewrite:

| Character or controller | P at the rewrite | Consequence |
| --- | --- | --- |
| Townsfolk, region NPCs, the Darkest Faerie (all three loops and the router), the Undead Draik, and any controller not listed below | 1.0 times the status factors | Clip speed follows status effects; with none active it is always 0.5 ÷ S. |
| Common enemies, class 17 attack sequencer (SPEC-0038) | The slot's wind-up, strike or recovery word in the phases that store it; otherwise 1.0 times the status factors | A stored word replaces the status factors for that update. |
| Faerie Sisters | 2 − HP ÷ maximum HP, every update | Clip speed changes as health drops, without a restart. |
| Gnarfas | The animation-speed setting (1.0 as shipped), every update | Replaces status factors; no change at 1.0. |
| Black Knight | 1.0, stored every update | Replaces the status factors: a stunned or slowed knight still plays at 0.5 ÷ S (0.25 key per tick for S = 2). |
| Werelupe King | 1.0, stored every update (its phase speed values are not applied) | Same as the Black Knight. |
| Plague Serpent, Cloudbeast base loop | 1.0 stored every update; the serpent's bite recovery and the Cloudbeast air-flow stages (1.25, 2.0, 3.0) and one frozen Cloudbeast state (0) store another value | Changes only in those states. |
| Gelert controllers | 2.0 stored in the shadow back-stab and mirror-attack states | Those clips play at twice the normal speed. |
| A character parked in base state 42 (the Gelert Assassin's waiting mirror images) | Not re-timed | Keeps the speed set at clip start. |

Whether R stays 1.0 matters only in these cases: a status factor is active (and the controller does not overwrite it), the controller stores a value other than 1.0 (class 17 slot words, Sisters, Gelert 2.0 states, Cloudbeast stages, serpent bite), or the character is parked in base state 42. When R stays 1.0 the rewrite writes the same value as the clip start, so a clip with S = 2 advances 0.25 key per tick for its whole length either way.

Test vectors (S = 2 for all clips). (1) A Sister at full health, then at half health on the next update, while one clip plays: speed 0.25, then 1.5 × 0.5 ÷ 2 = 0.375 key per tick, with no clip restart. (2) A class-17 enemy that is slowed (status factor 0.5) while walking plays at 0.125 key per tick; once it enters an attack wind-up whose stored word is 1.0, the rewrite gives 0.25, because the stored word replaces the status factor. (3) A stunned Black Knight (stun sets P to 0 before the controller runs) still plays at 0.25 key per tick, because the controller stores 1.0 afterwards. (4) A Darkest Faerie clip under no status plays at 0.25 key per tick from its start to its end.

1c. **Cursor start.** Starting a clip, through a play request or a clip change, sets the cursor to 0, clears the finished flag, sets the weight to 0 with a fade in from the clip, and sets the root accumulator to (0, 0, 0, 1) (Clip start). Restarting a clip resets the cursor, the finished flag and the accumulator. A clip change is a restart. The cursor and accumulator are never reset on a loop wrap (item 3). Confidence: High for the start resets; Medium-High that the start path is the only clip-change path.

2. **Advance, per tick.** On each game tick (SPEC-0007), for a playback record with a clip, the steps of Layer step run in order: the speed-0 rule (main layers), the end rule (item 3), the fraction and key pair (item 4), the fade step, the sample and combine (items 4 and 5; skipped while the weight is 0), events (item 7), and the cursor advance by the record's speed (1a) while the finished flag is clear.
3. **End of clip.** Applied at the start of each tick, before sampling:
   - If *c* ≥ *F* − 1:
     - **One-shot clip** (byte 0x27 is zero): *c* = *F* − 1 and the finished flag is set. The clip stays on its last key with fraction 0 (item 4), so the pose is exactly key *F* − 1.
     - **Looping clip** (byte 0x27 is non-zero): when *c* ≥ *F* − 1, the wrap flag is set and *F* is subtracted from *c*. If that result is negative, *F* is added back. Thus a cursor in [*F* − 1, *F*) is unchanged, but the wrap flag is still set on every such tick (including exactly *F* − 1). At *c* ≥ *F*, the cursor becomes *c* − *F*, keeping the overshoot (at *c* = *F* + 0.25 the cursor becomes 0.25, not 0).
   - If *c* < 0 (after the rule above): a one-shot clip sets *c* = 0 and the finished flag; a looping clip adds *F* to *c*.
   - Worked case: *F* = 19, looping. Cursor 18.5 stays 18.5, so the pose blends key 18 and key 0 with fraction 0.5. Cursor 19.25 becomes 0.25, so the pose blends key 0 and key 1 with fraction 0.25. Cursor 19.0 becomes 0.0 (key 0).
   - The root accumulator is not reset on wrap (item 5). Because the wrap flag is already set throughout [*F* − 1, *F*), the wrap-specific root accumulation described in item 5 also applies on those boundary-blend ticks, before the cursor itself reaches *F*.
   - Data fact (Medium-High): on plain looping entries, key *F* − 1 equals key 0 within 1.5/1024 on every channel in about 19 % of entries. On the other 81 % the blend across the wrap changes the pose, so the wrap-around blend is the normal behaviour of a loop, not an error.
4. **Sampling** at cursor *c*: key *k* = the whole part of *c*. Fraction *t* = *c* − *k*. The next key is *k*₂ = (*k* + 1) mod *F* when the speed is positive, and (*k* − 1) mod *F* when it is negative, with the reduction done on an unsigned 32-bit value (Edge cases). For a looping clip the last key blends into key 0. The sampler does not clamp the key. For each animated channel, the values at *k* and *k*₂ are sampled and blended with fraction *t*:
   - Translation: linear, (1 − *t*) × T_*k* + *t* × T_*k*₂.
   - Rotation: if the dot product of the two quaternions is negative, the second quaternion is negated. Then the four components are blended linearly, (1 − *t*) × Q_*k* + *t* × Q_*k*₂, with no spherical blend. The blend does not normalize its result, and nothing later does (Posing a skeleton, quaternion to matrix); the length of a blend of adjacent keys stays within a fraction of a percent of 1.
   - Step interpolation is not used.
   - Fraction storage (High): the stored fraction is *t* = *c* − trunc(*c*), where *c* is the cursor after the end rule for this tick (item 2a). It is refreshed only on ticks where the speed is not zero, so a clip with speed 0 keeps its last fraction.
   - Blend use (High): the record's weight *w* selects the path. If *w* is exactly 1.0, the sampled pose replaces the output. If *w* is not 1.0, the sampled pose is mixed with the pose already in the output, using the blend factor φ as the mix parameter (Combining a sample into the pose).
5. **Root record and root motion.** The root record is sampled at key *k* and key *k*₂ (Encoding: table A, or the last eight bytes of each frame in mode 0). The sampled root value is multiplied by the clip speed (1a), stored as the layer's root step and added to the layer's root accumulator, whose four float lanes are accumulated (High). In mode 2, only the first three table-A s16 values feed the scaled x, y and z root sample; its fourth source short is not used by the root sampler (Encoding). On a tick where the wrap flag is set the accumulator first adds the sampled root value of key *F* − 1, unscaled (High). With the mirror flag, the root step's x is negated; the accumulator is not (Mirrored clips). A finished main-layer clip with non-zero speed has its root step doubled (Layer step, step 8). A main layer with speed 0 sets its root step and accumulator to (0, 0, 0, 1) (Layer step, step 1). Sub-part and face records store the root value of key *k* unscaled, and nothing reads it. The root step comes from the newest clip on its first sampled tick, whatever its weight, so travel switches at once on a clip change. In all hero locomotion clips the root record is the same at every key, so the choice of key does not change their travel. For the heroes, the layer-0 root step is the travel source: the next tick it is turned by the hero's facing basis and added to the hero's position (High; SPEC-0040, Behavior 14C, which also gives the sign convention: z along the facing, x to the side, y up). The accumulator is zeroed at clip start (1c) and is not reset on wrap. For characters other than the heroes the position step is the same as for the heroes: the layer-0 root step, turned by the character's facing basis and added to its position the next tick (SPEC-0040, Behavior 14D, which also gives the extra terms for flying characters and the cases where the step is discarded) (High).
6. **Crossfade** (High). Every clip start begins a fade in from weight 0 (Clip start); the clip that was on the record stops at once, and the fade runs from the joint values already in the output (the pose shown last). The weight step and the blend factor are given in Layer step (step 4), and the mix in Combining a sample into the pose. Fade out is dormant: nothing starts one.
7. **Events** (High). The record keeps an index into the clip's (frame, value) pair list. Each tick, if the cursor's whole part equals the next pair's frame, the record's event word is set to that pair's value and the index advances; otherwise the event word is 0. The index wraps to 0 past the count. The pair list is never used: the pair count is 0 in every entry, the expanded copy of a compressed entry keeps it 0, and nothing reads the record's event word. The animation events of the game are (a) the table-B frame bits, read directly by frame index (Encoding, Table B frame bits), (b) the word-B sound cue at its start frame (Choosing an animation), and (c) the effect lists of SPEC-0034 section 1.2.
8. **Pending look and secondary step** (High). After the four records have run each tick, the player applies its pending look request (the hero head look or the Plague Serpent neck look, SPEC-0040 Behavior 23A) and clears it, then runs the secondary step chosen by its mode word (Secondary-motion chains). Each clip's evaluation mode (byte 0x29) selects the sampler, independently of the secondary mode.
9. **Joint composition.** Each joint's world transform is its parent's world transform times its local transform, and the root's world transform is the character's placement times its local transform. Joints are composed in index order (parents first), and families as in Families below. A joint's local transform is the sampled value if it is animated, else the rest value (Posing a skeleton).

### Playback records and layers (High)

Offsets below are in the character's animation player (the character's animation sub-object + 0x30; SPEC-0040).

- The player has four playback records of 0x60 bytes each: main layer 0 at +0x40, main layer 1 at +0xA0, face at +0x100 and sub-part at +0x160. Each record holds: clip, finished flag, mirror flag, weight *w*, fade step *s*, fade count *n* (a byte), fade mode (0 none, 1 in, 2 out), cursor *c*, fraction *t*, speed *v*, root step, root accumulator, event index, event word, clip index and group.
- At set-up every record is idle: no clip (0), finished set, mirror 0, weight 0, fade step 1.0, fade count 0, fade mode 0, cursor 0, fraction 0, speed 0.5, root step and accumulator (0, 0, 0, 0), copied from the boot-written zero vector at 0x3932C0 (SPEC-0007). A blend-enable flag at player + 0xF40 is set to 1 and never changed (Combining a sample into the pose).
- Per tick the player runs, in this order: main layer 0, main layer 1, then the sub-part record (only when the character has a sub-part family and a sub-part clip has been started), then the face record (only with a face family), then the pending look and the secondary step (Playback 8).
- A main clip plays on the layer named by its byte 0x28. That byte is 0 on every clip on disc, so layer 1 never plays from data.
- Main layers 0 and 1 write into the main joint list. Layer 1 runs after layer 0, so with weight 1.0 it would replace layer 0 on the joints in its mask. The sub-part and face records each write only into their own joint list (Families). No two records write the same list except the two main layers.
- Layer-1-only rule (dormant): a one-shot on layer 1 that reaches its last key has its clip removed that tick, and it is not sampled on that tick.

### Clip start (High unless stated)

Starting clip C on a record (for a main clip, the record of layer C[0x28]):

- Cursor 0, fraction 0, finished clear, event index 0, root accumulator (0, 0, 0, 1).
- Speed *v* = R × 0.5 ÷ S if S = C[0x2A] > 0, else R × 0.5 (Playback 1a and 1b).
- Weight *w* = 0; fade mode = fade in; fade count *n* = C[0x24]; fade step *s* = C[0x1C] ÷ C[0x24] (C[0x1C] = 1.0 on disc, so *s* = 1 ÷ C[0x24]).
- Mirror flag = the request's mirror bit (Mirrored clips); the clip index is stored (main).
- The clip that was on that record stops at once and is not sampled again. The fade runs from the frozen output.
- Compressed clips are expanded here (Compressed entries).
- If C[0x21] is non-zero, a pose word is cleared (0 on every clip on disc).

Which starts happen (per tick, before the record steps; the animation pass runs after the character's update hook and its motion tick, and the hero and the region characters are ordered in SPEC-0007, Order of work inside one tick):

- The main request id selects a main clip. If its layer already holds that clip index with the same mirror flag, nothing starts, so a finished one-shot is not restarted by the same id. Otherwise the main clip starts, and the sub-part and face clips for the same id start too, with no already-playing test for them. So a main clip change also restarts the sub-part and face clips.
- A second main id (when set to 0 or more) starts its main clip the same way, on the layer its clip names (layer 0 on disc). Its range test lets an index equal to the clip count through (Edge cases). A second-id lookup miss gives clip 0, as for the main id. Only the hero air attacks set a second main id (High): Roberta's air shot sets 19 or 20 for one tick and then requests the same id as the main id, so the attack clip starts one tick early; because layer 0 already holds it, the main request then starts nothing, and the sub-part and face clips of the jump keep playing (quirk). Tor's air slash (SPEC-0033 Behavior 50, strafe jumps only) keeps second main id 25 until landing. Tor's main table has no id 25, so the second lookup gives clip 0 (F 60, loop, fade 20, no event bits). Each tick:
  1. The main start finds layer 0 holding clip 0 and restarts the main clip, together with its sub-part and face clips.
  2. The second start finds the main clip and restarts clip 0.
  3. Layer 0 then steps clip 0 from cursor 0 with weight 0: φ = 1/20, so the shown pose moves 5 % toward clip 0's first key each tick. The main clip is never sampled.

  Gates on the main clip's progress find no record holding it (progress 0), so the jump tails never run, and the second id's 0.99 progress test never ends the slash. Event-bit queries see only clip 0 and the restarted sub-part and face clips, none of which has event bits, so no 0x1 damage window opens (High).
- The face clip uses the face override id when it is set, else the main id. It starts unless the face record already holds that clip with the same mirror flag. Its range test also lets an index equal to the clip count through (Edge cases). When the override's face clip reaches progress 0.99 or more (Clip progress; data word 0x2DD854 = 0.99), the override is cleared and the base face clip starts. The emote stream (SPEC-0040, Behavior 22) sets the override for face emotes (Medium-High for the 0.99 test).
- **Lookup miss.** When an id has no entry (id at or above the table's N, word A clip 0xFFF, or no table), the main, sub-part and face lookups return clip index 0. A miss therefore starts clip 0 of that family, unless clip 0 is already playing. The only no-start cases are a family that is not present, a second main id below 0, and an index that fails the clip-start range test (Edge cases). (Medium-High.)
- **Fade out (dormant).** Two starts would set fade mode out with step *w* ÷ C[0x25] and count C[0x25]. Nothing in the program calls either, and no data word holds their addresses, so no record ever fades out (High for no caller). Two further starts with an explicit weight, and one with an explicit layer and no fade, also have no known caller (Open questions).

### Layer step (High unless stated)

For a record with a clip, each tick:

1. **Speed 0 (main layers only).** If *v* = 0: the root step and root accumulator are set to (0, 0, 0, 1), and nothing else happens: no end rule, no sample, no cursor change. The joints keep their last values.
2. **End rule** (Playback 3).
3. **Fraction and keys.** *t* = *c* − trunc(*c*) (stored when *v* ≠ 0). *k* = trunc(*c*). *k*₂ = *k* + 1 if *v* > 0, *k* − 1 if *v* < 0, then reduced modulo *F* as an unsigned 32-bit number. Mode 0 and mode 2 use the same *k*, *k*₂, wrap flag and factor.
4. **Fade step.** With *w* the weight before this step:
   - Fade in: φ = *w* + *s*(1 − *w*); *w* ← *w* + *s*; *n* ← *n* − 1; when *n* reaches 0, fade mode ← none and *w* ← C[0x1C] (1.0).
   - Fade out (dormant): φ = *w* − *s*(1 − *w*); *w* ← *w* − *s*; *n* ← *n* − 1; when *n* reaches 0, the clip is removed, *w* ← 0 and fade mode ← none.
   - No fade: φ = *w*.
   The weight is linear in ticks; φ is not the new weight. Float order as written (High).
5. If *w* (after the step) is 0, the record is skipped this tick.
6. Sample keys *k* and *k*₂ with fraction *t* and combine each sampled joint into the output (Combining a sample into the pose).
7. Root (Playback 5).
8. **Finished root doubling (main layers; quirk).** If the finished flag is set and *v* ≠ 0, the root step of that tick is multiplied by 2.0. A one-shot held at its end therefore produces root step 2 × root(*F* − 1) × *v* every tick it stays held. (High for the rule; Medium for its gameplay effect, because which held one-shots have a non-zero last-key root record is not counted.)
9. Events (Playback 7), then *c* ← *c* + *v* if finished is clear.

### Combining a sample into the pose (High)

For each animated joint, A = (rotation, translation) at key *k* and B at key *k*₂:

- **Weight exactly 1.0:** if dot(qA, qB) < 0, qB ← −qB. Output ← A(1 − *t*) + B*t*, rotation and translation alike, then mirrored (Mirrored clips) if the mirror flag is set. The previous output is ignored.
- **Weight not 1.0:** if dot(qA, qB) < 0, qB ← −qB. S = A(1 − *t*) + B*t*, then mirrored if the flag is set. If dot(q_out, qS) < 0, qS ← −qS, where q_out is the value already in the output joint. Output ← (1 − φ) × output + φ × S, component by component, for rotation and translation.
- Nothing is normalised in either path.
- The output joint lists are not cleared between ticks, so "output" is the pose shown last tick (or, for layer 1, the value written earlier in the tick by layer 0).
- The blend-enable flag (player + 0xF40) is always 1, so every joint takes this weighted path. (With it clear, the first joint of a list would be overwritten unblended; not reachable.)
- Consequence: a clip change is a crossfade from the frozen last pose. Because φ = *w* + *s*(1 − *w*) is larger than the share needed for a linear fade, the old pose's share falls faster than linear (Test vectors).

### Clip progress (High)

State routines gate on the progress of an animation id. The clip is the one the id names in the main table, and the record read is the main layer named by the clip's byte 0x28 (layer 0 on disc); the four records are searched for that clip.

- Progress = (c + t) / F, where c is that record's cursor, t its stored fraction (Playback 4) and F the clip's frame count.
- It is 1.0 when that record is finished or has no clip, and 0 when no record holds the clip.
- Each caller compares it with a threshold that it passes as a constant: 0.99 in most callers, 0.95 in some. The threshold is not stored state.
- Quirk: the fraction is added on top of the cursor, which already holds it, so progress runs ahead by up to 1/F and can fall back by up to 1/F from one tick to the next. On a looping clip a 0.99 gate can pass only because of this term when 0.99 × F is above the last cursor before the wrap (Test vectors).
- Worked example: the Black Knight's parry (action 8) is a looping clip that still passes the 0.99 gate at tick 79 (SPEC-0012, Fight AI build guide).
- Which tick sees what (High). Every gate here, every frame-bit read (attack window, spawn event, recover window, footstep) and every attachment frame is read by a think or a hit test that runs before the same tick's animation pass, so it sees what the previous tick's pass left. A clip requested on tick s starts in the pass of tick s. At play rate 1 and speed byte 2 the cursor that the think of tick s + j sees is 0.25 j for j of at least 1, so the frame index whose bits are read on tick s + j is floor(0.25 j): a window on frames a to b (a at least 1) is read on ticks s + 4a to s + 4b + 3, and a one-shot clip of F keys reads as finished from tick s + 4(F - 1) + 1. A 0.99 gate read before the animation pass on tick T + k sees cursor k v and the fraction of cursor (k - 1) v, so a one-shot clip passes it on T + 2S(F - 1) + 1, one tick earlier from F = 50 / S keys (two from 150 / S), and a looping clip of S = 2 and 3 to 24 keys passes on T + 4F - 1. At another rate R the cursor is 0.25 R j: the window starts on the first j with 0.25 R j at least a and ends on the last j with 0.25 R j below b + 1 (SPEC-0036, Windows at other play rates, for the hero; SPEC-0038 Behavior 13, Phase durations, for the enemies). The enemy strike test reads its weapon frames and event bits this way (SPEC-0038 Behaviors 13 and 50).

### Mirrored clips (High for the flag and the partner field; Medium for the visual effect)

- Word A bit 12 is the mirror flag (bit 13 is the head-look flag; Choosing an animation). The main family's flag comes from the main table entry of the main id. The sub-part flag comes from the sub-part table at the second main id, and the face flag from the face table at the face override id. With those ids normally −1, only main clips mirror in practice.
- The second field of each skeleton-definition record (type 22 and the family headers) is the joint's mirror partner. A mirrored clip writes the sample of mask joint *i* into output joint partner(*i*). The sample is mirrored as rotation (x, y, z, w) → (x, −y, −z, w) and translation (x, y, z) → (−x, y, z). The root step's x is negated (Playback 5).
- Disc: in 550 of 563 type-22 chunks the partner field is an involution (partner(partner(*i*)) = *i*), with 13,756 non-self pairs. In 496 of those 550, a mesh bone table of the region has partner rest translations equal to (−x, y, z) of the joint's own for at least 90 % of joints. Header 492 pairs 5 ↔ 7, 8 ↔ 16 and 9 ↔ 17, among others; joints 0 to 4 are their own partners.
- Use counts: bit 12 is set in 7, 7, 0, 9, 11 and 0 entries of hero tables 520 to 525, and in 1,592 of 156,115 non-empty character-record main-table entries (counted with repeats).
- Sub-part and face partners: self-partners are stored as own indices, but paired partners as N_main + *j* (for example 34 ↔ 35 in header 494). The original writes at the raw value, which is past the end of the 13- or 17-entry list for paired joints. A clean side should not mirror sub-part or face clips, or should map a partner *p* ≥ N_main to *p* − N_main.
- Reachability (Medium-High): sub-part mirroring needs a second main id. The only second ids are 19 and 20 (Roberta) and 25 (Tor), and their sub-part entries have bit 12 clear. The face override is written only by the emote consumer (SPEC-0040, Behavior 22). Hero face tables have no bit-12 entry. A talk partner's face override is 0, 126 or 127, and region face tables carry bit 12 only on ids 29 (9 tables), 54 (11) and 139 (91). So no sub-part or face clip is ever mirrored in the shipped game, and the past-the-end partner write never happens.

### Per-bone translation scale (High for the read and the counts; Medium for the purpose)

- The per-bone extra block of a main skeleton definition holds K floats per joint, where K is the header word at 0x14 (0, 1, 2 or 4 in chunks; 28 in header 492, whose block is 28 × 4 = 112 bytes per joint).
- Before combining, a main-family sample's translations (both keys) are multiplied by block[joint][sel − 1] when the character's scale selector sel is not 0. The translation's w stays 1.
- sel is the character record's signed 16-bit value at kind-3 data + 0x8C (SPEC-0006): a 1-based set number, 0 for none. For the heroes it is the fourth short of the hero's row (0x2DB428 + 10 × hero index), which is 0 for both heroes, so the heroes are not scaled.
- Disc: every record with a local main family has sel ≤ K (pairs seen: (0, any), (1, 1), (1, 2), (1, 4), (2, 2), (2, 4), (3, 4), (4, 4)). Records on the global 492 set have sel 0 to 25, all ≤ 28. Header 492's values lie in 0.28 to 3.92 (joint 0: 1.046 for sel 1). These are character-size factors for the shared townsfolk rig.
- Quirk: the player's selector starts at −1 before the character data is copied in. A clean side should treat a negative selector as no scaling.
- Resolved (boss records; High): bosses use the selector exactly like common enemies. The character setup copies the record's value into the player for every character record, right after the player is built and with nothing in a boss routine writing it afterwards; all three sampling modes (0, 1 and 2) apply the multiply; the root step, the attachment offsets and the weapon radii and lengths are not scaled. The Plague Serpent's record carries selector 1 and its block holds 2.0 for every joint (1.999 for joint 0), and the mesh's rest offsets are 2.0 times the clip translations for all but six joints, so it is posed at twice the clip's size. The hit volumes that read the pose (the lunge weapon, the mouth point) therefore use the doubled positions (SPEC-0016, Reach and distances). The earlier default of pose scale 1.0 is withdrawn.

### Clip length unit (High)

A clip's length in seconds is L = (F minus 1) divided by 15, where F is the signed 16-bit frame count at +0x1E4 of the clip's object record (see Object record copy). A clip with F = 1 has length 0.

### Clip time and mode (High)

A time request t (in seconds) on an object clip of length L = (F − 1) / 15: if t < L, the time becomes max(t, 0); if t ≥ L, it becomes L and the mode 8. The pose is sampled, and then the mode becomes 10 unless it is 8. So a request below L always ends in mode 10 (stopped at t), whatever the earlier mode; mode 9 is only a passing value and never persists.

| Mode | Each tick |
| --- | --- |
| 0, 5 | time += dt × speed; past the end, wrap by whole lengths (loop) |
| 2 | advance; at the end, mode 8 |
| 6 | advance; at the end, mode 4 |
| 4, 10 | no advance |
| 8 | advance, clamped to L (finished) |

The end test applies at or past L with a positive speed, or below 0 with a negative speed.

Callers: SetObjToTime and SetObjToMayaFrame (SPEC-0030 rows 10 and 29); a jump-to-end helper (t = L); a world-object restore that sets mode 9 and then requests time 0, or the end when the object's saved flag says it was already used (SPEC-0028, Medium).

### Clip state on a request (High)

The companion byte (+0x1B9) is a two-setter latch. Its low bit records that the vector setter ran, and its next bit records that the angle/quaternion setter ran since the most recent clear: 0 = neither, 1 = vector only, 2 = angle only, 3 = both. Each setter adds its bit without clearing the other. Object initialization/reset, PlayAnim, LoopAnim, Deactivate and the explicit clear operation reset it to 0. The worked-out per-tick animation update does not read the byte; within the worked-out runtime, the setters read it only to preserve the other setter's bit. The reason downstream code might care about this latch is unknown.

When a clip request is applied, the request value (8 bytes) is stored. If the previous mode was 4, 8 or 10, the current clip update immediately runs with a zero time delta. If the object has a valid clip, the update clamps its stored time to the clip interval [0, L] and evaluates the pose at that time; an in-range time does not advance, and modes 4, 8 and 10 remain unchanged. This makes the current pose evaluation available immediately after the request (High).

### Clip rate pair (High)

Input: a lower rate, an upper rate, and two words (a key and a second word). The two words are stored first. Then:

- Lower: if the lower rate is below 0.00001, it is set to 0.00001 and the raw lower field keeps its old value. Otherwise the raw lower rate is stored.
- Upper: if the upper rate is above 0.99999, it is set to 0.99999 and the raw upper field keeps its old value. Otherwise the raw upper rate is stored.
- Scaled values: scaled lower = clamped lower times L, and scaled upper = clamped upper times L. The scaled values are stored from the clamped values.

Example: lower 0 and upper 1.0 on a clip with L = 2.0 give scaled lower 0.00002 and scaled upper 1.99998. The raw lower and upper fields keep their old values.

### Frame counter with ping-pong (High for the wrap; Medium for the ping-pong start)

While a clip is active, its frame index I (from 0 to F minus 1, where F is the playing frame count of the clip) advances once per tick. A direction flag D is used only when the owning record's ping-pong byte (+0xBC) is set.

- Without ping-pong: if I is F minus 1 or more, I becomes 0. Otherwise I increases by 1.
- With ping-pong and D = 0: if I is below 1, I becomes 1 and D becomes 1. Otherwise I decreases by 1.
- With ping-pong and D = 1: if I is F minus 1 or more, D becomes 0 and I decreases by 1. Otherwise I increases by 1.

A "changed" flag is set when a clip starts and cleared at the end of every step. When the clip is not active, the step does nothing except a helper call (not described).

### Path range table (not a clip table; High)

The tag-0x0A range table is not a clip table. Its only reader is the route set-up of a walking character: the +0xB8 word of the character's placement selects path k, and the route is the entry records A[k] to B[k] of the pack's entry list, both ends included (SPEC-0006, Tagged tables; SPEC-0056, Path walk). Nothing in the skeleton or clip system reads the table, so no clip start frame or frame count comes from it. [High]

### Object record copy (Medium)

Copying a clip object record from a source record: the flag word at +0x1E8 and the 16-bit frame count at +0x1E4 are copied. Four row vectors (16 bytes each) are stored as given at +0xC0, +0xD0, +0xE0 and +0xF0 (row 0 to row 3). A transposed copy of the same matrix is written in the +0x40 to +0x7F range (partly described). A helper runs before the copy (not described).

### Pose rate limit

A separate pass applies to six pose values, once per call. It builds horizontal unit directions a = normalize((T − P) with y set to 0) and b = normalize((Q − P) with y set to 0), where P and Q are the two sampled pose points and T is the supplied target. The angle is arccos(a · b). The branch sign is σ = a · (up × b), where up = (0, 1, 0): if σ > 0, the angle is negated and the upward branch runs; otherwise the downward branch runs. Each call moves each value toward the resulting target angle divided by 6.0 by at most 0.01.

- Upward branch: a value more than 0.01 below the target steps up by 0.01. A value more than 0.01 above the target is left unchanged.
- Downward branch (mirror): a value more than 0.01 above the target steps down by 0.01. A value more than 0.01 below the target is left unchanged.
- In either branch, a value within 0.01 of the target is set to the target.

The movement is one-directional per branch. This quirk is in the original and should be reproduced. The constants 0.01 and 6.0 are code constants, not disc data. Confidence: High.

In the pending-switch path, selector 1 applies this rate limit to the supplied target. Selector 0 dispatches to a separate target/update routine and does not use this limiter through that path (High). One selector-1 request is the Plague Serpent's idle animation (id 5): while its controller is in state 0, it supplies the hero's world position as the target. The target is therefore a position vector in this call, not four joint indices. The other callers and the meanings of any other target values remain open (SPEC-0016; High for this call path).

### Weapon frame blocks

Confidence: High for the selection, writer and attachment-frame identity; Medium for the use.

- An animated character keeps five 64-byte frames, one per weapon (index k = 0 to 4, at 64-byte spacing).
- The frame used this tick is chosen by the table-B damage-window bits on the character's playing records (Encoding): bit 0x1 gives frame 0, 0x2 frame 1, 0x4 frame 2, 0x8 frame 3 and 0x10 frame 4, tested in that order. With none set, frame 0 is used. The same query serves any character, the heroes included.
- Layout (High): in the character's locomotion sub-object, frame k starts at +0x13F0 + 0x40 k and holds four 16-byte rows. The fourth row (inside the frame at +0x30, so at +0x1420 + 0x40 k) is the frame's translation: the posed world position of attach point k, with the fourth lane 1. Two lookups read the selected frame. The single-vector lookup returns that translation, choosing frames 0 to 4 for the bits 0x1, 0x2, 0x4, 0x8 and 0x10 in that order and frame 0 when none is set. The full-frame lookup returns all four rows but tests only the bits 0x1, 0x2 and 0x4 (frames 0 to 2), and returns frame 0 for 0x8, 0x10 and for none. Test vector: a playing clip whose damage bits are 0x8 only gives the position stored at +0x14E0 from the single-vector lookup and frame 0 from the full-frame lookup.
- Readers take it as the active weapon's frame. For example, an enemy's projectile starts at the frame's position and is aimed at the hero (SPEC-0038).
- These blocks are not root motion and do not feed the root step (SPEC-0040, Behavior 10).
- The per-frame character render update builds the skeleton attach-point matrices and stores the current matrices in the frame bank; the previous bank is copied first for swept tests. The five weapon frames are attach points 0 to 4 in this same bank (High; see SPEC-0004 attach points).

### Attach points and carrying

Confidence: High for the selection and attach-table transform; Medium-Low for the separate optional carry offset.

- The bone table's pose entries (0x30 bytes each; Layout) are attach points: +4 bone index (negative means none), +0x10 three angles (x, y, z, radians), +0x20 translation.
- Let B be the selected bone’s current world matrix (already composed from character placement and animated pose), t the entry offset at +0x20, and (rx, ry, rz) its angles at +0x10. R = Rz(rz) × Ry(ry) × Rx(rx), using right-handed rotations. The attach frame is W = B × Translate(t) × R, so an attach-space point p maps to B(t + Rp). The table offset is added in bone-local coordinates before the attachment rotation is applied. Angles with magnitude at most 0.001 are treated as identity; sine/cosine are evaluated by the shipped approximation. This describes the entry transform only; it does not establish the semantics or sign of the separate optional carry offset.
- A character can carry one other character. Each tick that its carry slot is set, the carried character is placed at the carrier's attach point k (its direction and up vectors normalised), and the slot is cleared. The carrier sets the slot again every tick it holds.
- k is 0, or 1, 2, 3 or 4 while the carrier's frame has damage bit 0x2, 0x4, 0x8 or 0x10 (the first set bit in that order). The same choice applies to the hero's Miamouse carry: the hero's own frame bits decide k. The hero's actions 178 and 181 carry bit 0x20 only (one release frame each), and actions 179 and 180 carry no table-B bit on any frame, so k is 0 on all four (High).
- Users:
  - the grab-and-throw enemy (SPEC-0038 Behavior 15) carries the held hero (hero state 53);
  - the hero's Miamouse carry (hero state 57) carries the Miamouse. The Miamouse is also put in its state 33 with its id 12 requested. The optional offset belongs to the carrier's record: a vector at +0x950, a scale at +0x960 and a flag at +0x964. The hero's grab sets the flag to 1, the scale to 1.0 and the vector to the reset value (0, 0, 0, 1) (High for the values and writers). The carry step passes the first three components of the vector to two axis-rotation builders that turn the attach frame, so the zero angles leave the frame unchanged (Medium). The fourth component is not used by the rotations. The translation part of the offset is not settled (Low-Medium).

### Posing a skeleton

For each joint *i* of the active skeleton family, in index order (parents first), the composition reads the parent from that family's 12-byte parent-record table; the main-family table is the pointer at +0x10 of its runtime skeleton header. The parent is a u32 at each record start, and the root test compares the whole u32 with 0x000000FF. This is the skeleton-definition table (types 22 to 24, or the matching headerless family header), not the flat type-18 mesh bone table. The mesh bone table supplies the rest positions used by the skin-matrix step below.

- Local transform: L_i = translate(T_i) × rotate(Q_i). T_i and Q_i come from the sampled animation if bone *i* is in its mask; otherwise T_i = the rest local translation and Q_i = identity.
- World transform: W_i = W_parent × L_i (root: W_i = the character's placement × L_i).
- Skin matrix: S_i = W_i × translate(−rest position_i).

The shared skeleton pose pass writes three arrays of composed 4x4 joint transforms, in main, sub-part and face order, after the animation playback update. It is used for every character (heroes, region characters and the petpet). The three arrays are the main, sub-part and face outputs of this pass; the character part draw reads them as the pose matrices of the skin palette (SPEC-0046, Character part draw and the skin palette). The separate skinning upload job described above reads its own 3x4 bone-matrix array, which is a different block.

Quaternion (x, y, z, w) to rotation uses the standard (Hamilton) formula with column vectors. For example, (0, sin 45°, 0, cos 45°) is a 90° turn about +Y that maps +X to −Z. The original's own conversion gives this result on known inputs. Matrices are column-major, as in SPEC-0004.

**No normalisation (High for the conversion; Medium-High for the other steps).** No step normalises a rotation: not the samplers, the blends, the layer steps, the joint composition or the quaternion-to-matrix conversion. The conversion applies the standard formula (diagonal terms 1 − 2(y² + z²) and so on) to the raw, possibly non-unit values. For q with |q|² = s, the matrix is (1 − s) I + s R(q / |q|), where R is the rotation of the unit quaternion. A clean side must not normalise before the build.

**Families (High).** A character can play a main, a sub-part and a face animation at the same time. Each family samples into its own joint list: main (N_main joints, the main skeleton's count; 32 for the global sets), sub-part (N_sub) and face (N_face).

- **Skin bone order.** The bone table the mesh skins against is the concatenation main, then sub-part, then face. Main joint *i* is skin bone *i*; element *i* of the sub-part list is skin bone N_main + *i*; element *i* of the face list is skin bone N_main + N_sub + *i*.
- **Masks.** A main clip's mask bit *b* drives main joint *b*. A sub-part or face clip's mask bit *b* drives element *b* − N_main of its own list. Both families use the base N_main; the face base is not N_main + N_sub. No sub-part or face mask bit on disc falls outside [N_main, N_main + list size).
- **Parents.** For a sub-part or face record, a parent *p* below N_main is main joint *p*; *p* at or above N_main is element *p* − N_main of the same list.
- **No overlap.** The lists are separate, so sub-part and face never compete for a joint; only main layers 0 and 1 share a list (Playback records and layers).
- **Composition order.** The main list first (its root times the placement), then the sub-part list, then the face list.
- **Disc checks.** Face frame-0 translations match the mesh rest translations under this rule in 386,278 of 546,573 bone-clips (70.7 %), against 5,611 (1.0 %) if the mask bit were the skin bone. Parents resolve to a valid joint in 6,132 of 6,132 face records and 5,991 of 5,991 sub-part records. Tor outfit 504 has 58 = 32 + 13 + 13 bones, with 13 of 13 sub-part and 13 of 13 face joints matching. For the sub-part list the two rules give the same skin bone, which is why a direct mask-to-bone test also matched sub-part bones.
- **Talking-NPC pose check** (resource 707, a -7 townsfolk record; 54-bone mesh; N_main 32, N_sub 10, N_face 12): posing body 526 clip 48 with the record's sub-part clip 0 and each of 111 face clips (compressed ones included) by full forward kinematics gives a worst segment-length error per clip of 0.0004 to 0.296 under this rule (median 0.092; 17 clips at or below 0.01; the rest are face joints with animated translation), and 0.491 to 0.833 in every clip with face channels on bones 32 + *i*. The two ranges do not overlap.

### Forward kinematics rules (Confidence: Medium)

These rules are the joint-composition step of the posing pass above:
- A joint's world transform is its parent's world transform multiplied by its own local transform. The composition can be computed as a recursion from each joint up its parent chain, or as one pass in index order. The two give the same result, because every parent has a lower index than its children (Layout).
- The root's world transform is the root matrix supplied by the caller, multiplied by its local transform. In the character path this is the character's placement (see the item above).
- When the caller supplies no root matrix, the root output slot remains its local transform because the external root multiplication is skipped. A child still composes from that slot, so the chain is relative to the local root at the origin and has no external placement. The known character update callers supply a matrix pointer (the character object's +0x260 matrix workspace in the attachment/chain paths); no known direct caller supplies null. Whether an indirect or unknown caller omits the matrix remains open.
- The composition accepts at most 83 joints. The largest bone count on disc is 74 (Layout). Behaviour above 83 joints is not specified.

### Secondary-motion chains (Confidence: Medium for the integration and the constants; Medium-Low for the field layout and the joint step)

After the pose is built, the animation player's mode word (player +0x30) selects one secondary step. Its default, written when every player is built, is −1 (none). Only three characters set it (High):
- 0, the bone chains below: Gnarfas only. His behaviour sets it at its start, with four chains of 4, 4, 5 and 5 links on main joints 46 to 49, 38 to 41, 29 to 33 and 15 to 19.
- 1, the pendulum chain with one chain: Tor only, set at his spawn. It is his 5-link tail on sub-part joints 8 to 12 (outfits 504 to 511).
- 2, the pendulum chain with two chains: Roberta only, set at her spawn. They are two 4-link chains on her head, on sub-part joints 7 to 10 and 11 to 14 (outfits 512 to 519).

In Roberta outfits 512–518, the same 548-vertex upper-region leaf recurs and every vertex has some weight from chain joints 39–46; the leaf is untextured with full stored vertex alpha. The two chain roots are parented to head-side joint 6, and the bind-pose surfaces extend rearward along their paired tracks. This supports an integrated, chain-skinned appendage as a plausible source of a moving head-side shape, but neither its anatomy nor its connection to a rendered head gap is established. Outfit 519 is structurally different: its broad upper/body leaf has only sparse chain influence. Every outfit 512–519 has one core model child, one selector and one layer node, each selector/layer stored as zero; no separate hair/cap child or distinct nonzero leaf branch appears in these outfit trees. Their type-33 companion data feeds shadow-volume construction, not a visible hair/cloth model. The documented held-item route uses hand, arm, sheath and back anchors; no head-worn model is mapped. No generic attachment, held-item model, face-family mesh or second draw with another node mask covers the top of her head: the head is made of the leaves of the core model only. In the rest pose the chain-skinned strips fall behind the head and the part of the leaf near the head stays on it; the earlier wrong pitches (−3.5 and −6.0) fold the strips up and forward and uncover the sides of the head top (the head cover note is in SPEC-0004). (High for hierarchy, stored words, vertices and weights; Medium for shape interpretation.)

Roberta's Behavior 25 rest-angle values are radians. The chain frame uses the joint's X, Y and Z axes as its local basis, and the bind-pose chains extend along −Z. The yaw axis is the link joint matrix's Y row, the pitch axis its X row and the chain's forward axis its −Z; positive pitch raises a link and negative pitch lowers it, and each link's frame is the previous link's frame turned by its rebuilt rotation, so pitches add down the chain. The stored rest pitches are 0.25, −1.25, −0.75 and 0.2 rad with the pitch minimum equal to the rest pitch, giving cumulative pitches of +0.25, −1.00, −1.75 and −1.55 rad: each chain leaves the head slightly upward and backward and then hangs almost straight down behind the head. The joint result is written back as the sub-part joint's local rotation quaternion and local translation (0, 0, −a), relative to its link joint; the last joint of a chain is drawn with an identity local rotation. (High; the worked example, the per-tick order and the causes of upright or jittering chains in a port are in SPEC-0040, Behavior 25.)

Pendulum chain (modes 1 and 2; summary, the full rule, the per-link constants and test vectors are in SPEC-0040, Behavior 25). Each link keeps a point at a fixed distance L from its joint. Each tick the point gets a spring toward its rest direction (rest yaw and pitch per link, spring k), keeps a fraction b of its last move, and gets a drag c along the owner's unit displacement for the tick. It is then put back at distance L with its yaw and pitch clamped to per-link limits, and the joint is turned to aim at it. There is no gravity term. These links index the sub-part joint list; mode 0 indexes main joints.

Chain step (mode 0):
- A skeleton has up to five chains. Each chain has a link count, an active flag and an initialised flag. Each link names one bone of the skeleton and keeps two points, current and previous, and a rest length.
- On the first run (initialised flag clear) each link's rest length is set to the distance between its bone and its parent bone in the current pose (0 for a link with no parent).
- Each tick, for each active chain, each link in order except the last is moved by a Verlet step: new = 2 × current − previous + g, with g = (0, −0.075 × k, 0) per tick, where −0.075 is the gravity word and k = 1.0 is the scale word. Gravity acts on Y only (Y is up; SPEC-0004).
- The new point is pulled back to rest length from its anchor: new = anchor + normalize(new − anchor) × rest length. The anchor is the previous link's new point; for the first link it is the chain root (the position of the chain's root bone in the current pose).
- Previous takes the old current, and current takes the new point.
- The joint transform of the link's bone is then rebuilt from the new points with a blend weight of 0.1, and the bone's position is written back into the pose.

The rotation rebuild compares the current link direction with the direction from the old anchor to the constrained new point. It creates a correction quaternion between those directions, including explicit parallel and antiparallel cases, composes that correction with the bone's current local quaternion, and computes `normalize((1 − 0.1) × current + 0.1 × corrected)`. This is component-wise quaternion blending, not spherical interpolation. The updated local rotation and the constrained position are written to the pose before the next link is processed.

Constants (32-bit floats in the boot program's data): blend 0.1 at 0x2EC874, gravity −0.075 at 0x2EC878, gravity scale 1.0 at 0x2EC87C. The gravity is per tick (60 ticks per second, SPEC-0007), not scaled by the frame time.

### Skinning

A layout-A vertex (SPEC-0004) is skinned as v′ = Σ_k weight_k × S_(bone_k) × v, with bone_k = slot byte_k / 3, three weights that sum to 1, and v in rest (model) space. Bone indices are direct indices into the model's own bone table: in 13,905 skinned meshes, no index reaches the bone count. Up to 22 different bones appear in one batch, and up to 74 in one skeleton.

**End-to-end check** (resource 534, 37 bones, 11 plain animations, sampled frames): after skinning, triangle edges that cross bone boundaries keep their rest length to a median of 0.1 % (90th percentile 11 %). With the bone mapping deliberately shifted by one, the 90th percentile becomes 36 %. Use this as a regression test.

### Skinning upload

1. Blocks are processed in table order. The bone matrices are taken from the running matrix pointer, which starts at +0x18 and advances by 48 n bytes after each block.
2. For a block, the 48 n bytes of matrices are sent to the vertex program. Then the program's two result arrays, of n vectors each, are copied out to (chosen result base plus offset A) and (chosen result base plus offset B).
3. A clean implementation may use any transfer method, but it must produce the same result arrays for the same job and the same matrices.
4. The meaning of the two result arrays (for example two halves of a bone palette for a vertex) is open (Open questions).
5. Selector (Medium). The selector word (0x2EC830) is stored by the character model draw from its third argument, both before the skin stream and before the draw walk, so one draw uses one result slot for both. The skin job is the record that the mesh node's +0x50 word points to. The stream visits mesh nodes (type 1) across the model's node trees: an LOD node (type 4) visits its +0x78 child first, then children through first-child +0x40 and sibling +0x44. On disc, slots 0 and 1 are identical, so the selector value does not change results. The value the draw's caller passes is open (Open questions).

### Root-motion sampler query (dormant; Medium-High that it never runs)

Dormant: the query's only caller passes a throw-away buffer for the outputs, does nothing with them except bump a counter, and is itself never called, and no code pointer to either routine exists in the boot data. No output is consumed by travel, drawing or any other system, so the clean side leaves the query out. The description below is kept for completeness.

Inputs: the requested frame index f, the playing frame counter cur, and the clip's frame count N. Stored fields used (offsets in the sampler state): stored position +0x10, +0x14 and +0x18; playing counter +0x0C; frame count +0x2C4; step S at +0x2C8; seven per-frame value tables at +0x2CC, +0x2D0, +0x2D4, +0x2D8, +0x2DC, +0x2E0 and +0x2E4 (each N entries of 32-bit integers); cumulative table at +0x10C; fields at +0x154, +0x158, +0x2BC and +0x2C0.

1. **Timing bookkeeping.** If a start stamp (+0x27C) is set, it is cleared and the time since it is added to a running total (+0x2B4). If a second stamp (+0x28C) is set, it is cleared and the time since it is recorded as one of two longest intervals (+0x290 and +0x298), each with the playing counter at which it was recorded. Nothing reads these records (the query is dormant).
2. **Frame clamp.** If f is 0 or f is at least N, f becomes N − 1. Then if cur is less than f, f becomes cur − 1, and if that is 0, f becomes 1. So a request for frame 0 is returned as the last frame, not the first.
3. **Index.** The output index is f. The table index is I = (f + S) mod N (one subtraction if the sum reaches N).
4. **Deltas.** For each of the seven tables T, the query subtracts T[I] from T[S], where I is the wrapped index from item 3. Thus the arithmetic is T[S] − T[(f + S) wrapped once by N]; S is the first lookup index, while f + S (with one subtraction of N when needed) is the second. The seven results go to output positions 4, 5, 6, 10, 7, 8 and 9, in the order of the tables listed above (+0x2CC to output 4, +0x2D4 to output 5, +0x2D0 to output 6, +0x2D8 to output 10, +0x2DC to output 7, +0x2E0 to output 8, +0x2E4 to output 9). This query is not the character travel source established in Playback 5; it has no gameplay use because the query never runs.
5. **Time value.** Output position 13 is a helper's result on a = T[cur] − T[cur − f − 1] over the cumulative table at +0x10C, b = the field at +0x2BC, and c = (f + 1) times the field at +0x2C0. The helper is not needed because the query never runs.
6. **Copies.** Output positions 0 to 2 are the stored position, output 3 is f, and outputs 11 and 12 are the values at +0x154 and +0x158.

The output has 14 positions (0 to 13). The sampler writes seven delta-like values, which no routine reads.

### Joint pose sample (High)

Dormant: the routine has no caller and no stored or computed address. Its two sibling routines, which use the same helpers, are the secondary-motion steps of mode words 1 and 2 (Secondary-motion chains). A clean implementation omits it.

### Per-bone bounds (Low)

- For each bone, a bounding radius is computed from the bone's extents: the radius is 0.25 times the largest of the bone's extent values. The factor 0.25 is a code constant (the value is present in the boot image).
- The consumer of the radius is not identified (Open questions).

### Turn toward a direction (High)

Many routines make a call on the character just before a play request, with a decimal weight w. This call is a facing turn, not a play rate; it does not change R or the clip speed.

- Input: the character's stored facing c (a unit vector), a unit direction d and the weight w.
- v = w × d + (1 − w) × c.
- If dot(c, d) < −0.99, 0.1 times the side vector of the character's motion record is added to v, so a turn to the exact opposite direction still has a side to turn through.
- The facing becomes v / |v|.
- Weight 1.0 snaps to d; weights such as 0.25, 0.1 and 0.05 are partial blends per call. The routine that wants the turn makes the call on every tick it runs, so the facing closes on d over several ticks.

Examples (High for the pattern):

- Hero special action, sub-mode 1: turn at weight 0.25, then play id 167, only when the current id is not already 167.
- Hero special action, sub-mode 3: turn at weight 1.0, then play id 172.
- Hero special action, sub-modes 2 and 4: play ids 168 and 171 with no turn.
- A common enemy or NPC routine: turn at weight 0.25, then play id 6.
- A class routine: turn at weight 0.1, then play the id given by a mapping of a parameter. Parameter 1 gives id 10, 2 gives 24, 3 gives 34, 4 gives 47 and 5 gives 55. Any other parameter gives -1.
- The hero's strafe state turns toward the camera's forward direction at weight 0.25 (SPEC-0033).

The play call that follows plays a table id (Choosing an animation); two further arguments are -1 on every call seen. Ids 167, 168, 171 and 172 are out of range for every character-region table on disc. The hero tables have entries for them (Hero animation data and Edge cases).

### Choosing an animation

A character's animation tables (type-36 kind-2 records; SPEC-0006) map an action number to a clip. The hero tables use the same entry layout (Hero animation data). A table is a u32 count *N*, then N 8-byte entries. Entry *i* is the entry for id *i*. An id of N or more has no entry; the lookup then gives clip index 0 (see Missing ids and sets).

Table sizes in the character regions: 2,804 tables in the 575 character regions. N is 148 for 1,620 tables and 166 for 982 tables. The other 202 tables have other sizes (examples: 110, 113, 27, 21, 19). The largest N among these 2,804 tables is 166, so none of them has an entry for id 166 or above. The six hero tables (Hero animation data) have N = 193 and are not part of these counts. A class-17 enemy uses the main table of its own character record: N = 166 on 1,653 of the 1,654 class-17 records and 138 on one (resource 1273; SPEC-0038).

Counts over the character-region tables (415,574 entries in all): 207,380 entries have no clip and 194,748 have no cue. Word B is not a copy of word A's clip index: among entries that are not all-default, the two words agree in 480 cases and differ in 12,966.

**Word A** (clip and effect):

| Bits | Meaning |
| --- | --- |
| 0–11 | Clip index into the entries of the animation set the table is bound to. 0xFFF = none. |
| 12 | Mirror flag (Mirrored clips). |
| 13 | Head-look allowed (High; Medium for the look target). Only the hero reads it: on each tick that the hero's current main id has bit 13 set, the hero's pose step turns the head joints (4 and 6) toward the hero's look target (SPEC-0040 Behavior 23A). Set on 73 of Tor's 170 non-empty main entries (idles, locomotion, hangs and climbs; of the attack ids 12 to 31 only 22). Other characters never read it. |
| 14–21 | Effect-list index. 0xFF = none. |

**Word B** is a **sound cue request**, not a playback field. It does not select a clip, a loop or a speed. It is the packed sound word of SPEC-0031 (Layout 10), which gives the block numbers for each group.

| Bits | Meaning |
| --- | --- |
| 0–7 | Block index within the group. 0xFF = none (the word names no block). |
| 8–10 | Group: the sound-block family that the block index counts in (0 main block, 1 creature blocks, 2 area blocks; SPEC-0031, Layout 10). It is not a channel or category. |
| 11–19 | Cue id within the block. 0x1FF = none, and no cue starts. |
| 20–30 | Cue start frame *f* (High): the frame of the clip at which the cue starts (start rule below). |
| 31 | Keep-alive flag. When set, the cue is started as a keep-alive voice that stays alive while the same entry stays current (below; SPEC-0031, Layout 10). |

**Cue start frame (High).** When the main id changes, *f* (word B bits 20–30 of the new id's entry) is stored, and a previous-cursor value is set to −1.0. Each tick, if previous < *f* and *f* ≤ the layer-0 cursor, the cue starts (as below); previous then takes the cursor. A cue with *f* = 0 therefore starts on the first tick of the clip. A looping clip starts its cue again each cycle on the tick the cursor crosses *f* from below; with *f* = 0 that never happens again, because the cursor after a wrap is not below 0. After the start test, the keep-alive refresh runs every tick. The six hero tables hold no cue: every word B of their 1,158 entries is the no-cue value.

**Sound cue request (word B).** When the cue routine runs for a part of the character whose table index is in range and whose blocking flag is clear, the cue for the character's current id is started, as follows:

- The block is found from the group and the block index (SPEC-0031, Layout 10); the cue id selects the cue inside that block.
- If bit 31 is clear, the block and cue id are passed to the sound manager, which plays the cue once on a free voice. A voice already playing the same cue is reused unless a restart is requested.
- If bit 31 is set, the entry is stored on the character and the cue is started as a keep-alive voice, refreshed while the same entry stays current. A later stop request stops the stored cue while the id is unchanged.
- A cue id of 0x1FF plays nothing. For a creature, a block index of 0xFF also plays nothing. The hero uses only the cue id and plays it on the main sound slot (SPEC-0031, Creature blocks and animation sounds).

Confidence: High for the word B layout (SPEC-0031, Layout 10); Medium for the cue path (the stop and restart details of a stored entry are only partly described).

The character record's three table indices (SPEC-0006, kind-3 data + 0x9E) pick the main, sub-part and face tables. Its skeleton indices (data + 0x86, + 0x88, + 0x8A) pick the paired skeleton and animation-set chunks. A negative main-family value (data + 0x86) selects a global set instead: -1 none, -5 Roberta main, -6 Tor main, -7 the shared townsfolk bank of body 526 (Hero animation data, Body 526; High for the mapping and the names). In 1,822 of 1,832 main-table entries the clip index is below the bound set's entry count.

**Missing ids and sets.**

- An id of N or more, or a word A clip of 0xFFF (or no table): the main, sub-part and face lookups return clip index 0, so clip 0 of that family starts unless it is already playing, and no message is reported (Lookup miss, under Clip start; Medium-High). A character whose action table lacks an id that a status routine writes into its locomotion slots (the Werelupe King, under confusion) plays clip 0 of its main family.
- A word B cue id of 0x1FF, or a character table index that is out of range or unset: no cue starts.
- Main set not initialised (its initialisation flag is -1): the current-index lookup and the previous-index lookup each report a diagnostic message and return 0 (the previous-index lookup is attempted only when its stored index is nonnegative). The check happens on every lookup; it is not suppressed after the first warning, so repeated calls can report repeatedly. Direct callers include character update and controller paths as well as the character render path; exact indirect/script reach is not bounded here. This is the only missing-set case that reports a message; an id miss alone does not (High for the guard, return values and repeat behavior; Medium for caller categories).
- Sub-part or face set not initialised: the lookup returns 0 and reports nothing (Medium-High).

### Random idle picker

Confidence: High for the draw ranges, the miss rule and the timer's use; Medium for the mode meanings. A character's idle mode word selects how its next idle animation is chosen. The picker is called from a mode setter, and the mode setter is called from one character routine.

- Mode 1: draw one whole number uniform from 0 to 15 (16 values). The index is looked up in a 16-entry mapping that is part of the picker's own code. The mapping is not copied into this spec (see Open questions). All 16 entries are reachable.
- Mode 2: draw one whole number uniform from 0 to 15 (16 values). The mapping in the code has 20 labelled entries, but the draw never exceeds 15, so entries 16 to 19 are unreachable. 16 of the 20 entries are reachable.
- Any other mode word: no draw; the stored animation index from before is replayed.
- Lookup: the chosen index is looked up in the character's animation set (Choosing an animation). If the lookup finds nothing, the stored index is set to 0 and animation 0 plays. Otherwise the index plays as its animation, by its animation number.
- Timer (Medium): when the mode setter is called while the mode word is 0, it first draws a whole number of ticks uniform from 300 to 720 (5 to 12 seconds at 60 ticks per second) and stores it as the idle timer. The timer is the villager pair's talk or wait duration: it counts down by 1 on each tick of the pair's talk step, and at 0 both villagers go to the reset role (High; SPEC-0040, Behavior 3B, step 6).
- This picker is the gesture pick of the villager conversation (SPEC-0040, Behavior 3B): its mode word is the villager's role word (1 or 2). The ids of both lists, including the four mode-2 entries the draw cannot reach, are given there (High for the ids; Medium for the unreachable entries).

### Two-window animation stream (High for the readiness switch; Medium for transfer lifecycle)

The character's primary animation data is streamed through two alternating windows. Initialization marks both windows unavailable, selects window 0, sets the stream state to 0 and starts loading the first window. The loader records how many animation records are available in each window and advances the source offset as it prepares the next one.

When the primary character animation object is in state 2, its update checks the alternate window. If that window has no available records, the state remains 2 and the object is skipped for that update. Once the alternate window's available-record count is positive, the stream returns to state 1, switches the active window, and submits the old window for refill; the update can then read the newly active window. State 2 is therefore a wait for animation data, not a pending clip request. This mechanism is separate from the per-clip pose crossfade (Clip start).

Confidence: High for the state transition, the record-count readiness test and use by the primary character animation object; Medium for the general read-ahead description because the queued transfer's completion lifecycle is not fully established. The exact clip banks and indices using this stream remain open (Open questions).

### Model-set setup (eight slots)

Confidence: Low for the meaning of the three parameters; High for the slot count and the order.

- The owner has eight model-set slots. The first time a slot is non-empty, three objects are created with the parameters 30, 100 and 150 and stored in the owner.
- After that, every slot receives an update call with those objects and its own parameter.
- Three further sub-objects (of 0x5C, 0x4C and 0x4C bytes) are created on first use at fixed positions in the owner and linked back to it. Each is created only once.

What the parameters 30, 100 and 150 mean (time, distance or count) is not known.

## Edge cases and error handling

- Clip index ≥ the set's entry count (10 table entries): play nothing. The main and sub-part clip-start tests compare the index with the bound set's clip count as unsigned numbers, so an out-of-range main clip index starts nothing on the main family, and the clip-length query returns 0 for it.
- Off-by-one range tests (quirk): the second-main start and the face start let an index equal to the clip count through. Disc tables stay below the count, so this is not reached from disc data; a clean side may reject index = count. The 16 records with main-family value -6 in resources 1278 and 1279 hit this for every main request (indices 924 to 1,082 against the Tor main set's 161 clips), so they never start a main clip (Hero animation data, Body 526).
- An entry with F = 0 (some face entries): treat as no animation.
- Cursor in [*F* − 1, *F*) on a looping clip: not an error. The pose is a blend of the last key and key 0 (Playback 3 and 4).
- One-shot clip at or past its end: the cursor is held at *F* − 1 and the pose is exactly key *F* − 1.
- Cursor below 0: one-shot clips are held at 0 and finish; looping clips add *F*.
- Table B pair count is 0 on every disc entry, so pair-list events never fire from disc data (Playback 7). The per-frame bits of table B are still read by frame index (attack windows and the 0x40 test; Encoding).
- Table A's fourth value is not used by the sampler, in either layout.
- Speed byte S = 0: the speed is R × 0.5, with no division (Playback 1a).
- Speed 0 (for example R = 0): the cursor does not move, and the stored fraction keeps its last value (Playback 4). On a main layer, speed 0 also skips the end rule and the sample, so the joints keep their last values, and resets the root step and accumulator to (0, 0, 0, 1) (Layer step, step 1).
- Finished one-shot held (quirk): its root step is doubled on every tick it is held with non-zero speed (Layer step, step 8).
- Negative speed (quirk): *k*₂ = *k* − 1 reduced modulo *F* as an unsigned 32-bit number, so at *k* = 0 it gives (2³² − 1) mod *F*, not *F* − 1. Not reachable on disc.
- Fade-in length: a fade-in of n ticks gives φ = 1.0 only after n ticks; the weight reaches exactly 1.0 on tick n and the sample overwrites from tick n + 1.
- Sub-part or face mirror with a paired partner: the original writes past the end of the list (Mirrored clips).
- Scale selector −1 before the character data is copied: treat a negative selector as no scaling (Per-bone translation scale).
- Pose rate limit, one-way quirk: a value more than 0.01 on the far side of the target is not moved toward it in that branch (Pose rate limit).
- Secondary-motion chains: the last link of a chain is never moved by the Verlet step, so a chain with link count 1 does not move. A link with no parent has rest length 0, so its pull-back puts it on its anchor; the normalize of a zero vector at that point is not specified (Medium-Low).
- Idle picker, unreachable entries: mode 2 entries 16 to 19 are never drawn (Random idle picker).
- Idle picker, lookup miss: an index with no match in the character's set plays animation 0 and stores 0 as the index.
- Idle picker, other mode words: no draw; the stored index is replayed.
- In 50 bone tables every bone's local-translation w is the same value other than 1 (for example 1.5 or 2.0). Use x, y, z only and ignore w.
- 11 entries whose mask popcount differs from *nb*: skip them.
- Compressed entry started without a working buffer: the original reads the raw compressed entry, which does not walk as a channel stream. A clean implementation should always expand before playing (Open questions, In-place start).
- Bytes after the channel stream of an expanded block: not part of the clip; ignore them (Compressed entries).
- Weapon frame blocks: when more than one damage bit is set, the lowest bit wins (0x1 before 0x2, and so on). When none is set, frame 0 is used. The rule is the same for every character.
- Attach points: bit 0x1 does not select an attach point; a carrier frame with only 0x1 uses attach point 0 (Attach points and carrying).
- Forward kinematics, no root matrix: the root output slot holds its local transform, and children compose from it; no external placement is applied (Forward kinematics rules).
- Forward kinematics, joint cap: a skeleton above 83 joints is outside the composition's limit; the disc maximum is 74, so the case does not occur on disc.
- Skinning upload, zero-bone block: a block whose count is 0 reads and writes nothing, and the matrix pointer does not move.
- Skinning upload, selector: the result base is chosen once per draw by the selector word. On disc both result slots are identical, so either value gives the same result (Skinning upload, item 5).
- Root-motion sampler, request 0: the frame becomes N − 1 (Behavior, step 2). Frame 0 therefore cannot be returned by this query; a request of 1 returns frame 1.
- Root-motion sampler, request equal to the playing counter: the frame is not clamped by step 2, and the time value reads the cumulative table at index −1 (Medium). Whether the original avoids this case is not established.
- **Request equal to the length**: a request at or above L sets mode 8 and stores L, not the requested value.
- **Request below zero**: the stored time is 0 and the mode becomes 10 (Clip time and mode).
- **Zero rate**: a lower rate of 0 is raised to 0.00001 on the scaled value. The raw lower field is not updated.
- **Frame count of 1**: L is 0, so every non-negative request gives mode 8. A frame count of 0 gives a negative L (not handled in the code).
- **Ping-pong on one frame**: F = 1 keeps I at 0.
- **Hero ids 166 to 192.** The hero's attack and special-action routines request ids 166 (0xA6), 167 (0xA7), 168 (0xA8), 171 (0xAB), 172 (0xAC) and 192 (0xC0). The hero tables have entries for ids 0 to 192 (Hero animation data), so these ids resolve. Some resolve to no clip: Roberta main id 166 has word A 0xFFF, and Tor face id 169 (0xA9) has none. For those the lookup gives clip 0 of that family (Lookup miss).
- **Hero id 193 and above.** The hero tables have 193 entries, so these ids have no entry; the lookup gives clip 0 (Lookup miss).
- **Id 166 on a 166-entry character-region table.** Valid ids are 0 to 165. Id 166 is one past the end, so the lookup gives clip 0 (Lookup miss).
- **Cue id 0x1FF.** No cue starts, whatever the group and block index in the same word hold.
- **Word B is not a clip.** Only word A bits 0–11 select a clip.
- **Mapping miss in a class routine.** The routine that turns at weight 0.1 has a parameter mapping that gives -1 for parameters other than 1 to 5. A play of id -1 finds no entry, and the lookup miss rule applies (Turn toward a direction; Clip start).
- **Second-id lookup miss.** A second main id with no clip gives clip 0 (not no start); only a second id of -1 starts nothing (Clip start).
- **Progress overshoot (quirk).** Progress adds the stored fraction on top of the cursor, so it can exceed 1.0 on a looping clip and can drop between ticks (Clip progress).
- **Turn to the opposite direction.** With dot(c, d) < −0.99 the side nudge is added before normalising; with w = 0.5 and d = −c the blend is zero and the facing becomes the side vector (Turn toward a direction).
- **Non-unit quaternions.** A quaternion of squared length s gives (1 − s) I + s R, a scaled-and-shifted matrix, not a pure rotation; reproduce it rather than normalising (Posing a skeleton).
- **Event bits with no reader.** Bit 0x200 has no effect anywhere, and bit 0x400 has an effect only on the Werelupe King's summon (Encoding, Table B frame bits).
- **Mode-0 bone-0 overflow (quirk; High).** In resources 1218, 1219 and 1268 (type 25, entry 14, 41 bones, F 102), bone 0's y translation overflows the 16-bit range. From frame 84 it falls by about 4,300 to 5,200 raw per frame. At frame 96 it reads +29,752 instead of about −35,784 (the true path continues smoothly to about −8.74 units). The engine reads the stored s16, so the bone jumps from −7.69 to +7.26 between frames 95 and 96 (interpolation sweeps across). Reproduce as stored.
- **Tor's air slash (quirk; High).** The second main id 25 resolves to clip 0, the main clip and clip 0 restart each other every tick, the pose drifts toward clip 0's first key, and no damage window opens (Clip start).
- **Sub-part and face mirroring** is unreachable on the shipped data (Mirrored clips).

## Test vectors

Synthetic:

- Key rate from S at R = 1.0: S = 3 gives 1/6 key per tick, 10 keys per second; S = 2 gives 1/4, 15 per second; S = 1 and S = 0 give 1/2, 30 per second; S = 4 gives 1/8, 7.5 per second. A 31-key looping clip with S = 3 wraps every 186 ticks (3.1 s).
- A constant channel whose value field is 0x0400 gives 1.000448 (0x0400 × 0.000977, the sample scale; not exactly 1.0).
- Animated channel with F = 5: a 4-byte header (u16 1, then u16 0), then 8 samples (5 used, 3 padding) = 20 bytes total.
- Cursor 7.5 in a clip with F = 20: key *k* = 7, next key 8, fraction *t* = 0.5.
- Blend: q_k = (0, 0, 0, 1), q_(k+1) = (0, 0, 0, −1) gives dot < 0, so q′ = (0, 0, 0, 1) and the blend stays at identity for any *t*.
- Looping clip, F = 19: cursor 18.5 stays 18.5 (key 18 with key 0, fraction 0.5). Cursor 19.25 becomes 0.25 (key 0 with key 1, fraction 0.25). Cursor 19.0 becomes 0.0 (key 0, fraction 0). A rule that wraps at F − 1 with the overshoot dropped would give cursor 0.0 at cursor 18.5, which is not the expected result.
- One-shot clip, F = 19, cursor 18.5: the cursor becomes 18 with the finished flag set, and the pose is key 18 exactly.
- Pose: a two-bone chain, root rest (0, 1, 0), child local (1, 0, 0), so child rest (1, 1, 0). Rotating the root 90° about +Y moves the child to (0, 1, −1), and S_child maps rest point (1, 1, 0) to (0, 1, −1).
- Mask 0x0000_0000_0000_0FF8 means bones 3 … 11, so nb = 9.
- Weapon frame selection (synthetic damage bits): none set → frame 0. Bits 0x2 and 0x8 → frame 1 (0x2 outranks 0x8). Bits 0x4 and 0x10 → frame 2. Bit 0x10 alone → frame 4.
- Attach point selection (synthetic): carrier frame bits 0x6 → attach point 1 (0x2 is tested first); bit 0x1 only → attach point 0. Attach-frame math (disc entry plus synthetic bone world): Tor resource 504, entry 6 uses bone 33, rz = −0.9312 radians, and t = (−0.186, 0.189, 0). With B a pure translation to (0, 1.864, −0.208), W maps the attach origin to (−0.186, 2.053, −0.208) and its X axis to approximately (0.597, −0.802, 0).
- Tor's air slash (synthetic): the share of the pre-slash pose after n ticks of the slash is 0.95^n (n = 10: 0.5987; n = 20: 0.3585).
- Mode-0 overflow (disc, resource 1218 entry 14, bone 0 y, raw): frame 95 −31,485 (−7.69), frame 96 +29,752 (+7.26).
- Speed at R = 1.0: S = 3 gives 1/6 frame per tick; S = 2 gives 1/4; S = 1 gives 1/2; S = 0 gives 1/2; S = 4 gives 1/8. A clip with S = 3 started at cursor 0 has cursor 1.0 after six advances.
- Speed at other R: S = 3 with R = 0.5 gives 1/12 frame per tick; S = 3 with R = 2.0 gives 1/3.
- Clip duration at R = 1.0 (disc, body 526): clip 48 (F 33, S 2, loop) wraps every 2 × 2 × 33 = 132 ticks; clip 104 (F 8, S 3, loop) every 2 × 3 × 8 = 48 ticks; clip 67 (F 60, S 1, one-shot) finishes after 2 × 1 × 59 = 118 ticks.
- Clip-start range (synthetic): a main set of 161 clips with a requested clip index of 924 starts nothing, and the clip-length query gives 0; index 160 starts normally.
- Cursor and fraction: a clip with S = 3, R = 1.0 and F = 20, started at cursor 0, has cursor 0.5 after three advances. The next tick stores fraction 0.5.
- Speed zero: R = 0 gives speed 0. The cursor stays where it was, and the stored fraction does not change.
- Root step (synthetic, Playback 5): cursor 7.5 in a clip whose key 7 root record is (0, 0, 0.4) and key 8 root record is (0, 0, 0.8), with speed 0.5. The root step is (0, 0, 0.2), from key 7, not (0, 0, 0.4) from key 8 and not a blend.
- Root step, hero walk (disc, Locomotion root records): root record z = 461, S = 1, R = 1: speed 0.5 keys per tick, root step (0, 0, 461 × 0.000977 × 0.5) = (0, 0, 0.225198), 13.512 units per second. Run: z = 658 gives 0.3214. Reverse: z = −922 with S = 2 gives speed 0.25 and step (0, 0, −0.225198).
- Root step, townsfolk walk (disc, Body 526): clip 104, z = 512, S = 3, R = 1: speed 1/6 keys per tick, root step (0, 0, 512 × 0.000977 ÷ 6) = (0, 0, 0.0833707), about 5.00 units per second.
- Per-tick speed recompute (hero, Playback 1a): a hero walking with R = 1 (speed 0.5) whose R drops to 0.6 on tick n plays at speed 0.3 from tick n; its root step becomes (0, 0, 0.1351) (461 × 0.000977 × 0.3).
- Pose rate limit, upward branch, target 0.5: value 0.2 → 0.21; value 0.8 → 0.8 (unchanged); value 0.495 → 0.5 (within 0.01).
- Pose rate limit, downward branch, target 0.5: value 0.8 → 0.79; value 0.2 → 0.2 (unchanged).
- Pose rate-limit sign (synthetic): P = (0, 0, 0), Q = (1, 0, 0). T = (0, 0, 1) gives a = (0, 0, 1), b = (1, 0, 0), σ = −1 and the downward branch; T = (0, 0, −1) gives σ = +1, negates the 90-degree angle, and selects the upward branch.
- Forward kinematics (synthetic): a three-joint chain where joint 0 is the root, joint 1 is its child with local translation (1, 0, 0), and joint 2 is the child of joint 1 with local translation (0, 2, 0). With root matrix = translation (0, 5, 0): world of joint 1 has translation (1, 5, 0), and world of joint 2 has translation (1, 7, 0). With no root matrix, joint 0 retains its local transform in the output slot; joint 1 composes from that local root, with no external placement.
- Secondary-motion chain (synthetic): a chain with link count 2, so only its first link moves; that link has rest length 1.0, is anchored at the chain root (0, 0, 0), current = previous = (1, 0, 0). Tick 1: Verlet gives (1, −0.075, 0); pulled back to length 1 from the root it becomes about (0.99720, −0.07479, 0); previous becomes (1, 0, 0). Tick 2: Verlet gives about (0.99440, −0.22458, 0), which is pulled back to about (0.97543, −0.22030, 0). A link hanging straight down at rest (root (0, 2, 0), current = previous = (0, 1, 0), rest length 1) stays at (0, 1, 0): Verlet gives (0, 0.925, 0) and the pull-back restores length 1.

Counts (disc test facts for compressed entries; all decode as described above):

- Compressed blocks: 51,043 in total. Top-level clip chunks 47,048 (type 25: 623, type 26: 7,865, type 27: 38,560); nested in type-32 containers 3,847 (type 25: 32, type 26: 643, type 27: 3,172); headerless clip banks 148.
- Exact decoded size (*U*): 51,043 of 51,043. Exact input consumption (*C*): 51,043 of 51,043.
- Streams with 7 channels per bone and unit quaternions (tolerance 0.02): 51,043 of 51,043.
- Streams that end exactly at *U* after the channels: 41,909 (top-level 38,618, nested 3,163, headerless 128). Streams with trailing bytes: 9,134 (top-level 8,430, nested 684, headerless 20).

Disc vectors for headerless clip banks (resource numbers of the archive; entry numbers count from 0 in chain order; "entry at" is the entry's offset in the resource). For flag 1, *U* and *C* must match exactly and the channel walk end is measured from the start of the expanded block; "tail" is the number of bytes after the channels. Bone 0 values are sample ÷ 1024 (data values; the runtime scale 0.000977 makes them 0.045 % larger), rounded to three decimals, as rotation (x, y, z, w) and translation (x, y, z). "Norm" is the largest deviation of bone quaternions from unit length over all frames.

| Resource | Entry | Entry at | Flag | *U* / *C* | *nb* | *F* | Walk end | Tail | Bone 0, first frame | Bone 0, last frame | Norm |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| 526 | 1 | 0x944 | 1 | 9,040 / 3,997 | 32 | 30 | 9,040 (= *U*) | 0 | (−0.004, 0.000, 0.000, 1.000; 0.000, 1.145, 0.000) | (0.004, 0.000, 0.000, 1.000; 0.000, 1.143, −0.001) | 0.00066 |
| 526 | 19 | 0xF6BC | 1 | 4,120 / 1,466 | 32 | 10 | 4,040 | 80 | (0.000, 0.000, 0.000, 1.000; 0.000, 1.074, 0.000) | (0.000, 0.000, 0.000, 1.000; 0.000, 1.070, 0.000) | 0.00067 |
| 493 | 114 | 0xAC3D0 | 1 | 3,676 / 901 | 32 | 15 | 3,612 | 64 | (−0.001, −0.062, −0.011, 0.998; 0.000, 1.140, 0.000) | same as first | 0.00056 |
| 503 | 35 | 0x13D90 | 1 | 2,716 / 716 | 13 | 22 | 2,556 | 160 | (−0.062, 0.000, 0.000, 0.998; 0.000, 0.267, 0.110) | same as first | 0.00047 |
| 495 | 0 | 0x30 | 0 | plain | 13 | 60 | resource offset 4,048, the next entry | 0 | (0.000, 0.000, 0.000, 1.000; 0.000, 0.000, −0.149) | same as first | 0.00055 |

- In each flag-1 row the expanded block begins at entry + 0x3C (for example 0x980 for resource 526 entry 1).
- Resource 526 entry 1: table B holds 30 u32 values and table A 30 records, and the walk of 32 × 7 channels ends exactly at *U*.
- Tails: resource 526 entry 2 has a 1,236-byte tail equal to the last 1,236 bytes of entry 1's expanded block. Tails occur in entries 2, 14, 19, 28, 34, 42, 44, 60, 61, 68, 72, 78, 90, 94 and 98 of resource 526, 114 and 116 of resource 493, 50 of resource 499, and 32 and 35 of resource 503; each repeats the end of the previous entry's block (20 of 20).
- Plausibility: bone 0's translation y (root height) is 1.07 to 1.15 in these resource 526 and resource 493 clips, which fits the same 32-bone rig. Bone 31 of resource 526 entry 1 is at translation (−0.062, −0.215, 0.269) on its first and last frame, a rest offset that recurs in other clips. Entries 526/19, 493/114 and 503/35 start and end on the same pose; entry 526/1 ends within 0.002 of its start.

Disc vector (resource 541, type-25 entry 6: 21 bones, F = 20, loop flag set, evaluation mode 2). Values are sample ÷ 1024 (data values; the runtime scale 0.000977 makes them 0.045 % larger), rounded to four decimals. The bone is the first animated bone (bone 0). Cursor 19.5 gives key 19, next key 0, fraction 0.5.

| Key | Rotation (x, y, z, w) | Translation (x, y, z) |
| --- | --- | --- |
| 19 | (0.0254, 0.0664, −0.0713, 0.9951) | (−0.0332, 0.6396, 0.0) |
| 0 | (0.1006, 0.0596, −0.0791, 0.9902) | (−0.0332, 0.6348, 0.0) |
| Expected at cursor 19.5 | (0.0630, 0.0630, −0.0752, 0.9927) | (−0.0332, 0.6372, 0.0) |

The dot product of the two rotation keys is positive, so no sign flip applies. The wrong result from wrapping at F − 1 (cursor 0.5, keys 0 and 1) differs in rotation, for example x = 0.1401 and y = 0.0547, so the test separates the two rules.

- Idle picker, mode 2: a draw of 16 cannot occur; 15 is the last reachable index. Mode word 3: no draw, and the stored index plays.
- Idle picker, miss (synthetic): stored index 9 with no match in the character's set: the index becomes 0 and animation 0 plays.
- Idle picker, timer (synthetic): the mode word changes from 0 to 1 with a timer draw of 450 ticks: the timer holds 450 (7.5 s), inside the 300 to 720 range.

- Skinning upload, layout: a job with two blocks, the first with 3 bones and offsets 0 and 48, the second with 1 bone and offsets 96 and 112. The first block reads 144 bytes of matrices and writes three vectors at the base and three at base plus 48. The second reads the next 48 bytes (bytes 144 to 191) and writes one vector at base plus 96 and one at base plus 112 (synthetic).
- Root-motion frame clamp, N = 10: request 0 gives 9; request 12 gives 9; request 4 with playing counter 10 gives 4; request 7 with playing counter 5 gives 4; request 3 with playing counter 1 gives 1; request 9 with playing counter 9 gives 9 (synthetic).
- Root-motion table index, N = 10, S = 2, f = 8: I = (8 + 2) mod 10 = 0, and the first delta is T[2] minus T[0] (synthetic).
1. Clip time (synthetic): L = 2.0 (F = 31). t = 0.5 from mode 2 gives mode 10 at 0.5, and the next tick does not advance. t = 3.0 gives mode 8 at 2.0. t = −1 gives 0.0, mode 10. F = 16 (L = 1.0): t = 0.5 from mode 6 or mode 1 gives 0.5, mode 10.
2. Rate pair (synthetic): lower 0, upper 1.0, L = 2.0 gives scaled lower 0.00002 and scaled upper 1.99998. The raw lower and upper fields keep their old values.
3. Frame counter, F = 4, ping-pong: indexes 0, 1, 2, 3, 2, 1, 0, 1, 2, 3, 2 (the second pass turns at 3 and at 0).
4. Frame counter, F = 4, no ping-pong: indexes 0, 1, 2, 3, 0, 1 and so on.
5. Clip start (synthetic): A = 10 and B = 3 give F = 8 and start frame 3.
7. Word A (synthetic) 0x00084005: clip index 5 (bits 0–11), effect index 0x21 (bits 14–21), bits 12 and 13 clear.
8. Word B (synthetic, decoding only) 0x80052A10: keep-alive bit set (bit 31), cue id 0xA5 (bits 11–19), group 2 (bits 8–10), block index 0x10 (bits 0–7), so block 395 + 16 = 411 (an area block); bits 20–30 clear.
9. Table with N = 3 (synthetic): id 2 has an entry; id 3 has none, so the lookup gives clip index 0. Id 5 on the same main table starts clip 0 unless clip 0 is already on layer 0.
10. Turn (synthetic): facing (0, 0, 1), d = (1, 0, 0), w = 0.25: v = (0.25, 0, 0.75), facing (0.3162, 0, 0.9487). Facing (0, 0, 1), d = (0, 0, −1), w = 0.5: dot −1 < −0.99, v = 0 + 0.1 × side; with side (1, 0, 0) the facing becomes (1, 0, 0). The clip speed is unchanged by either call.
11. Clip progress (synthetic): a looping clip with F = 20 and speed 0.25. Between ticks (cursor c; stored fraction t of the sampled cursor): c 19.0, t 0.75 → 0.9875; c 19.25, t 0 → 0.9625; c 19.5, t 0.25 → 0.9875; c 19.75, t 0.5 → 1.0125, which passes a 0.99 gate. Without the fraction term the largest value before the wrap is 19.75 ÷ 20 = 0.9875, which never passes. A one-shot that has finished reports 1.0; an id whose clip is on no record reports 0.
12. Quaternion to matrix (synthetic): q = (0, 0.6, 0, 0.6), s = 0.72. The matrix maps (1, 0, 0) to (0.28, 0, −0.72); the unit quaternion of the same axis, (0, 0.7071, 0, 0.7071), maps it to (0, 0, −1).
13. Cue start frame (synthetic): *f* = 5 on a looping clip with F = 10 and speed 0.5. The cursor is 0 on the first tick of the clip and reaches 5.0 on the eleventh tick, when the cue starts; it starts again when the next cycle's cursor reaches 5.0, 20 ticks later.
14. Event bits (disc): the frames of Tor's id 14 OR to 0xDC1 (weapon 1, cancel, both feet, effect and recover bits); Roberta's id 14 ORs to 0x1A1 (weapon 1, release and both feet; no recover bit). An enemy slot with trigger code 1 is eligible on the 0x800 frames of Tor's id 14 and never against Roberta.
15. Setter follow-up (synthetic): F = 31 gives L = 2.0 s. With mode 10 and stored time 0.5 s, the conditional update receives delta 0, leaves time at 0.5 and mode at 10, then evaluates the pose at 0.5 s. With mode 8 and stored time 2.25 s, it clamps time to 2.0 s, leaves mode at 8, and evaluates the end pose.
16. Companion latch (synthetic): start at 0; vector setter -> 1; angle/quaternion setter -> 3; explicit clear -> 0; angle/quaternion setter -> 2; vector setter -> 3; LoopAnim -> 0. No animation-tick read of this byte is specified.

Playback records, crossfade, mirror, scale and indexing:

- Speed at clip start: S = 1, R = 1 → *v* = 0.5. S = 2, R = 0.6 → 0.15. S = 0, R = 1 → 0.5.
- Rogue strike clips (disc, Rogue main set, R = 1): clip 38 (one-shot, F 21, S 2): *v* = 0.25, 4 ticks per key; the cursor reaches 20 after 80 ticks, the next tick sets the finished flag, and a 0.99 progress gate read before the animation pass passes 81 ticks after the start tick (80 ticks after the start the progress is 20.75 / 21 = 0.988). Clip 8 (looping, F 15, S 2): the gate first passes 59 ticks after the start (cursor 14.75 plus fraction 0.5 gives 1.017; at 58 ticks, 14.5 plus 0.25 gives 0.983). Clip 0 (looping, F 25, S 2) wraps every 100 ticks.
- Fade-in 4 (synthetic): weight 0, *n* = 4, *s* = 0.25. Ticks 1 to 4: φ = 0.25, 0.4375, 0.625, 0.8125, and stored *w* = 0.25, 0.5, 0.75, 1.0. Tick 5: *w* = 1.0 exactly, so the sample overwrites.
- Fade-in 5 (body 526 clips; Roberta's run clip 110): φ = 0.2, 0.36, 0.52, 0.68, 0.84, then 1.0. The old pose's share after each tick is 0.8, 0.512, 0.2458, 0.0786, 0.0126, then 0.
- Fade-in 8: φ = 0.125, 0.2344, 0.3438, 0.4531, 0.5625, 0.6719, 0.7813, 0.8906, then 1.0.
- **Hero walk to run, Tor (disc).** The run clip (table 520 id 4 → clip 92, F 14, S 1, loop, fade-in 16) starts on layer 0 at weight 0 with *s* = 0.0625; at R = 1 the cursor moves 0.5 per tick.
  - φ for ticks 1 to 16: 0.0625, 0.1211, 0.1797, 0.2383, 0.2969, 0.3555, 0.4141, 0.4727, 0.5313, 0.5898, 0.6484, 0.7070, 0.7656, 0.8242, 0.8828, 0.9414. From tick 17 the weight is 1.0 and the run overwrites.
  - Stored weight: 0.0625 × tick, reaching 1.0 on tick 16.
  - The frozen walk pose's share: 0.9375, 0.8240, 0.6759, 0.5149, 0.3620, 0.2333, 0.1367, 0.0721, 0.0338, 0.0139, 0.0049, 0.0014, 0.0003, …, 0.
  - The root step is the run's from tick 1: 658 × 0.000977 × 0.5 = 0.3214 per tick (walk: 461 × 0.000977 × 0.5 = 0.225198). Travel does not ramp.
- Blend (synthetic): output rotation (0, 0, 0, 1), sample (0, 0.7071, 0, 0.7071), φ = 0.25 → (0, 0.1768, 0, 0.9268), not normalised. A sample of (0, −0.7071, 0, −0.7071) is negated first and gives the same result.
- Speed 0 on layer 0 (synthetic): the cursor and joints do not change, and the root step becomes (0, 0, 0, 1).
- Table B frame bits (synthetic): a frame value of 0x401 has weapon 1 active (bit 0x1); 0x400 has no weapon active; 0x6 has weapons 2 and 3 active; 0x40 has no weapon active but passes the 0x40 test.
- Table B on a looping clip (synthetic): F = 4 with values 0x1, 0, 0, 0x1; at a cursor whole part of 4 the value is 0x1 AND 0x1 = 0x1 (window active). With values 0x1, 0, 0, 0 it is 0 (window inactive).
- Hurt capsule (synthetic): base point (0, 0, 0, 1), length 0.8, radius 0.3, bone world frame = identity translated by (1, 2, 3): the capsule runs from (1, 2, 3) to (1, 2.8, 3), centre (1, 2.4, 3), radius 0.3. A length of 0 gives no capsule.
- Finished doubling (synthetic): one-shot, F 10, S 2, R 1 (*v* 0.25), root(9) z = 0.4. Once held, the root step is z = 0.4 × 0.25 × 2 = 0.2 per tick.
- Mirror (synthetic): sample q = (0.1, 0.2, 0.3, 0.927), t = (0.5, 1.0, 0.2) on main joint 5 of header 492 → written to joint 7 as q = (0.1, −0.2, −0.3, 0.927), t = (−0.5, 1.0, 0.2). A root step (0.1, 0, 0.4) becomes (−0.1, 0, 0.4).
- Translation scale (disc, header 492): selector 1, joint 0 translation y 1.145 → 1.145 × 1.046 = 1.198.
- Indexing (disc, resource 707, -7 record): N_main 32, N_sub 10, N_face 12, mesh 54 bones. Face mask bit 35 → face element 3 → skin bone 45. Sub-part mask bit 35 → sub-part element 3 → skin bone 35.
- Indexing (disc, Tor): header 496 face record 12 (skin bone 57) has parent 32, so its parent is face element 0 (skin bone 45). In outfit 504, bone 57's parent is bone 45, and bone 45's parent is main bone 6. Header 494 sub-part record 8 has parent 32 → sub-part element 0 = bone 32.

Hero id-to-clip sample (Hero animation data; roles in Hero action ids by role: 0x20 hurt, 0x22 death, 0x44 soft landing, 0x46 standing launch, 0x4C moving launch, 0x76 hard landing, 0x85 boosted launch, 0x89 get up, 0xAF arrival pose (state 59), 0xC0 hurt without a weapon). Each cell is the clip index in the named family body; a dash means no clip (word A 0xFFF):

| Id | Tor main | Tor sub-part | Tor face | Roberta main | Roberta sub-part | Roberta face |
| --- | --- | --- | --- | --- | --- | --- |
| 0x13 | 42 | 27 | 10 | 7 | 11 | 19 |
| 0x20 | 48 | 33 | 16 | 59 | 48 | 55 |
| 0x22 | 47 | - | 0 | 61 | 108 | 95 |
| 0x44 | 72 | 57 | 0 | 86 | 76 | - |
| 0x46 | 74 | 55 | 0 | 82 | 78 | 66 |
| 0x4C | 79 | 54 | 0 | 84 | 69 | 66 |
| 0x76 | 144 | - | - | 99 | 112 | 101 |
| 0x85 | 86 | 0 | - | 88 | - | - |
| 0x87 | 35 | 20 | 8 | 98 | 86 | 75 |
| 0x89 | 146 | 0 | - | 102 | 110 | 98 |
| 0xA6 | 154 | 72 | 58 | - | - | - |
| 0xA7 | 124 | 65 | 55 | 116 | 92 | 81 |
| 0xA8 | 128 | 64 | 54 | 120 | - | 85 |
| 0xAB | 125 | 62 | 52 | 117 | 93 | 82 |
| 0xAC | 127 | 66 | 56 | 121 | 95 | 84 |
| 0xAF | 157 | 75 | 61 | 132 | 103 | 93 |
| 0xC0 | 60 | 39 | 16 | 65 | 49 | 56 |

### Test vectors: a real bone table (disc)

- Resource 504 (Tor's outfit), the flat type-18 chunk at resource offset 43,200 (payload at 43,216): bone count 58, pose-entry count 9, payload length 5,136 = 64 + 58 x 80 + 9 x 48. Every bone's rest position equals its parent's rest position plus its local translation (0 mismatches over 58 bones).
- Bone 0 (root): 3 children, parent field 0xFFFFFFFF, first-child offset 80 (bone 1). Local translation = rest position = (0, 1.1398, 0). Its hurt capsule has length 0.3675 and radius 0.3675.
- Bone 1: parent offset 0 (bone 0), first-child offset 240 (bone 3), 1 child. Local (0, 0.0789, 0.0790), rest (0, 1.2188, 0.0790).
- Bone 33: no children (first-child field 0xFFFFFFFF), parent offset 240 (240 / 80 = bone 3). Local (0, 0.2848, -0.3509); bone 3's rest position is (0, 1.5793, 0.1431), so bone 33's rest position is (0, 1.8641, -0.2078).
- Bone 19: parent offset 1,440 (bone 18, whose rest position is (-1.3350, 2.0447, 0.0775)), 2 children. Local (-0.2232, 0, 0), rest (-1.5582, 2.0447, 0.0775). Bones 19 and 33 have no hurt capsule (length 0).

## Open questions

- Default if unresolved (rewrite choice): drive character travel from the layer-0 root step; keep the pose-block root group and mode-2 sampler-local output separate unless a direct connection is established.


- **Hero animation region (answered; see Hero animation data).** The hero's action tables and clip sets are the raw resources 492 to 503 and 520 to 525. They are not in any of the 575 character regions. The hero clip counts, bone counts, id-to-clip rows and table sizes are given in that section.
- **Hero clips that do not parse as plain.** Resolved: all 148 (18 Tor main, 10 Roberta main, 1 Roberta sub-part, 9 Roberta face, 110 body 526) have compression flag 1. Each expands to exactly *U* bytes from exactly *C* bytes and then walks exactly (Compressed entries; Headerless resources).
- **Body 526.** Resolved: it is the shared townsfolk clip bank (154 clips, 32 bones, header 492) for character records whose main-family value is -7 (553 records in 266 resources), indexed through those records' own tables (Hero animation data, Body 526). High for the mapping and the rig.
- **Body 526 users.** Resolved: the -7 records are 553 non-combat NPCs of 26 monster types with behaviour class none; neither hero uses the bank. All 553 pass the rig check (32 plus sub-part and face bone counts equals a mesh skeleton bone count of the region), so they share the 32-bone rig of header 492 (Body 526; High).
  - **Body 526 clips.** Resolved in part: the code-named ids (idle, walk, run, fast run, lying, sit, fidgets) and the clip groups with their counts are given in Body 526. Clips 18, 54, 66, 72, 73, 74 and 135 are unreferenced by every table. The four clips with fade-in length 8 are 26 (looping; table id 53 for every user), 124 (looping; table id 111 for every user), 73 and 74 (one-shots, unreferenced by the 553 user tables). Their pose labels are uncertain and the fade value alone does not establish a narrative meaning. Which level-script animation calls target a specific Body 526 character for ids 12–19 or 24–99, and whether any such call reaches clips 73 or 74, cannot be settled from the import-site records because they do not bind call arguments to a character record (needs a runtime check). Default if unresolved (rewrite choice): play only the clips that a character's own table names; never play the unreferenced clips 18, 54, 66, 72, 73, 74 and 135. [Medium]
- **Main-family value -6.** Resolved: value -6 binds the Tor main set. The 16 records of resources 1278 and 1279 carry main clip indices 924 to 1,082, which all fail the clip-start range test, so they never start a main clip (Body 526; Edge cases). What those tables were made for is developer intent and not encoded anywhere (no main bank on disc has more than 161 clips). [High for the range test]
- **Townsfolk walking displacement.** Resolved: bone 0 has no horizontal travel in any body 526 clip, but table A does: walk 104 and 107 z = 512 per key, 108 z = 175, runs 105 and 111 z = 1382, fast run 106 z = 1975, carry walk 110 z = 128, idle 0. A walking -7 NPC moves by this root step exactly as a hero does (Body 526; Playback 5; SPEC-0040, Behavior 14D). High.
- **Lying idle switch.** Resolved: the NPC id 97 (and fidget 142) is the flying idle. The state test is "flying": the character's row has Can Fly 1 and its grounded countdown is not positive (SPEC-0040, Behavior 3C). Townsfolk never fly, so their lying ids 97 and 98 come only from the NPC modes 11 and 12 (SPEC-0040). High.
- **Outfit skeleton and family headers.** Resolved: the mesh bone table is main, then sub-part, then face (Tor outfit 504: 58 = 32 + 13 + 13). The sub-part and face header values at or above 32 are parents and mirror partners that name elements of their own list (value − 32), not bone-table indices (Families; Mirrored clips). High.
- **Ids for idle, run, jump, hurt and death.** Resolved: the NPC routines request idle id 0, walk ids 1 and 8, run id 6, fast-run id 5, lying ids 97 and 98 and sit id 65 (SPEC-0040, Behavior 3C; Body 526 gives their clips for the townsfolk). Hero: idle 0, 1 and 35 with the idle picker (SPEC-0033), walk 2 to 6 (7 to 11 with the weapon out), run 4, jump launches 70, 76 and 133 with tails 67 and 78, landings 68 and 118, hurt 32 and 192, death 34 (deadly ground 136, 173 and 174), game-over hold 175, get up 137 (Hero action ids by role; sample rows in Test vectors). High.
- **Enemy table.** Resolved: a class-17 enemy uses the main table of its own character record. N is 166 on 1,653 of the 1,654 class-17 records and 138 on one (resource 1273, monster type 9), so code 137 is in range for all of them; a code at or above a table's N is a lookup miss (SPEC-0038). High.
- **Ids 166 and above in character-region tables.** Resolved: an id at or above a table's N is a lookup miss, and the lookup gives clip index 0 of that family (Lookup miss, under Clip start). No second table answers it. The hero tables hold ids 0 to 192 (Hero animation data). Medium-High.
- **Word B bits 20–30 and word A bit 13.** Resolved: word A bit 12 is the mirror flag (Mirrored clips), bit 13 is the head-look flag that only the hero reads (Choosing an animation), and word B bits 20–30 are the cue start frame (Cue start frame). Bit 13 is set on 73 of 170 non-empty entries of Tor's main table 520, on 40,777 of 156,115 character-table entries and on none of the 553 townsfolk tables; only the hero's entries take effect. High.
- **Cue category and restart path.** Resolved: bits 8–10 are the block group and bits 0–7 the block index within it, not a channel category; bit 31 is the keep-alive flag (SPEC-0031, Layout 10). The cue starts on the tick the layer-0 cursor first reaches the start frame in word B bits 20–30, again each loop cycle, and the keep-alive refresh runs every tick after that test (Cue start frame; High). Still open: the stop and restart details of the entry stored on the character (some argument values are not known).
- **Mode-0 and mode-1 key pair.** Resolved for mode 0: it uses the same key pair (*k*, *k*₂ with wrap), wrap flag and blend factor as mode 2 (Layer step; High). Mode 1 is not used on disc.
- **Mode-0 root record.** Partly resolved: the fourth s16 is nonzero and varies from 2484 to 4096 across all 5,102 frames of the 97 mode-0 entries, so it is not zero padding or a constant homogeneous value. Its semantic meaning is not encoded anywhere; retain the raw lane for the four-lane accumulator, which is all the rewrite needs. Mode 1's packed-rotation layout (not used on disc) is not specified.
- **Root accumulator.** Resolved: the accumulator adds the speed-scaled root record of key *k* (all four lanes) each sampled tick and, on a wrap tick, first adds the root record of key *F* − 1 unscaled; it is reset to (0, 0, 0, 1) at clip start and by speed 0 on a main layer (Playback 5; Layer step; High). The speed-scaled root record of key *k* is the layer's root step, and heroes, NPCs, enemies and bosses use that layer-0 step for travel (SPEC-0040, Behaviors 14C and 14D; High). Partly resolved: in the mode-2 slot sampler, *k*₂ = (*k* + 1) mod *F* selects a second table-A root record; its sampled x/y/z are written to a sampler-local output field and added to a sampler-local accumulator. A wrap tick also adds the final-key root record to that accumulator. Within the documented paths, neither sampler value feeds the layer-0 travel step, the separate animation-instance root accumulator, or the stored pose-block root group. An indirect data hand-off remains possible and is not established. [High for the sampler-local rule and the distinct travel source; Medium for the direct-path boundary]
- **Root step.** Resolved: heroes use the layer-0 root step, transformed by their facing basis and added to position on the next tick (SPEC-0040, Behavior 14C). Townsfolk, enemies and bosses use the same layer-0 root step through the shared non-hero motion path; eligible flyers may add row-derived velocity, and their height correction is a separate update (SPEC-0040, Behavior 14D). High for the step source and motion path; the precise row-predicate meanings remain open in SPEC-0040.
- **Accumulator reset on stop and seek.** No character stop or seek path that resets the accumulator exists in the code that was followed; the accumulator is reset at clip start and by speed 0 on a main layer (Root accumulator). Default (rewrite choice): those two resets only; a later stop leaves the accumulator as it is (needs a runtime check only if a script-driven stop is found to clear it). The clip-start reset is High (1c), and there is no reset on loop wrap (Playback 3). A separate area/world-record reset clears fields at the same offsets used for a playback cursor, fraction and speed, but its known callers do not identify a character playback record; it does not clear the finished flag or root accumulator. It therefore does not establish a character stop or seek path, or an accumulator reset on either (Medium-Low).
- **Frames and time.** Resolved: character clips have no seconds conversion; their cursor moves in keys per tick (Playback 1a), which at 60 ticks per second is 30 / S keys per second at R = 1.0. The 14.999999 frames-per-second helper belongs to the animated world objects, whose time word is in seconds (SPEC-0028): it has two users, the object stream sampler and the script's current-frame query (SPEC-0030 row 25). No character clip path uses it (High).
- **Play rate R.** Resolved for the expression: R = character speed scale (+0x3D4) × character time scale (+0x8B8), and the speed is R × 0.5 ÷ S, or R × 0.5 when S = 0 (Playback 1a and 1b; High). The heroes' locomotion clips play with R other than 1.0 at partial tilt and under status effects, recomputed every tick. The game initializes the actor time scale to 1.0 and includes guarded controller paths that set it to 0; other paths copy an action/state value or multiply the current value. This is the per-character animation factor, separate from the world play-time clock. Resolved: no routine of the boot program or of the level modules writes a base speed scale other than 1.0, so no script changes a character's base scale (SPEC-0040, Behavior 1C; High); the per-tick factor P is reset to 1.0 every update and modified only by the stick tilt and status rules (SPEC-0033, Behavior 5; SPEC-0040, Behavior 3 and Clip speed within the hero update), and the per-action values are in those specs. Resolved (R-0124, High): for class 17, R is 1.0 times P, and P is the slot speed word in the attack phases that store it, else 1.0 unless a status factor applies; no row or spawn value scales it (Playback 1b).
- **Speed byte S.** Resolved: S is the key spacing of the clip in 30 Hz frames. At R = 1.0 a clip plays 30 / S keys per second: 10 for S = 3, 15 for S = 2, 30 for S = 1, 7.5 for S = 4, and 30 for S = 0, which skips the division (High for the rates, which follow from Playback 1a at 60 ticks per second; Medium for the name).
- **Negative speed.** No source of the play rate listed under Play rate R is known to supply a negative value, and whether a script can set one is a runtime question (needs a runtime check). Default (rewrite choice): never produce a negative speed. If one did occur, *k*₂ = *k* − 1 is reduced modulo *F* as an unsigned number, so key 0 does not pair with key *F* − 1 (Edge cases).
- **Direct-rate play path.** Resolved: unused. The play start that sets the speed to R with no 0.5 ÷ S factor is reached only from one wrapper, and the boot program holds no call, jump, stored pointer or address build of that wrapper. Playback 1a is the only speed rule (High).
- **Two-slot combination.** Resolved: the earlier two slots are the sub-part and face records, which write separate joint lists. Only main layers 0 and 1 share a list, layer 1 running after layer 0, and layer 1 is never used on disc (Playback records and layers; Combining a sample into the pose). High.
- **Blend factor.** Resolved: with *w* the weight before the step, the factor is φ = *w* + *s*(1 − *w*) for a fade in (then *w* ← *w* + *s*, and *w* ← 1.0 on the last tick); output = (1 − φ) × previous output + φ × new sample, for rotation and translation; weight exactly 1.0 overwrites (Layer step; Combining a sample into the pose). Float order as stated. High.
- **Animation requests.** Resolved: every clip start sets the weight to 0 and fades in over C[0x24] ticks; the previous clip is dropped at once and the fade runs from the frozen output. A main clip change also restarts the sub-part and face clips. Fade out is never started (Clip start; High). The call that routines make before a play with a decimal weight is a facing turn, not a play rate (Turn toward a direction; High).
- **Events source.** Resolved: the pair list is never used (pair count 0 in every entry and in every expanded copy; nothing reads the record's event word). The events of the game are the table-B frame bits, the word-B cue at its start frame and the effect lists of SPEC-0034 section 1.2 (Playback 7). High.
- **Compressed expansion.** Resolved: no loader expands them; the loader only relocates offsets. The block is expanded at clip start into the playing instance's working buffer, after a copy of the entry header, with the table offsets shifted to the copy (Compressed entries; Medium-High for the trigger).
- **Trailing bytes.** Resolved count: 9,134 compressed blocks have bytes after the 7 × nb channel walk (8,430 top-level, 684 nested, 20 headerless). Under one consistent entry-chain walk, 9,101 are byte-for-byte suffix copies of the previous entry's expanded block: 8,400 top-level, 681 nested and all 20 headerless tails. The remaining 33 (30 top-level, 3 nested) are repeated copies of one identical expanded type-27 mode-2 record with 11 bones and 28 frames; each has the same 52-byte tail. The data pattern is narrowed to one replicated record, but the tail's meaning and source remain open. The channel walk itself never reads any tail. Whether a separate evaluator reads them remains open (Medium). The earlier 8,452 top-level count was not reproduced; the consistent recount is 8,430.
- **Tables A and B meaning.** Resolved for table B: every frame bit from 0x1 to 0x1000 has a known role and reader, and 0x200 has none (Encoding, Table B frame bits; High, Medium-Low for 0x1000). Still open: the meaning of table A's 8-byte records beyond the root record use. Low.
- **Constant-channel bytes.** Resolved: bytes 6–11 of a constant channel are never read; they are padding, and the per-bank tags have no effect (Encoding; High).
- **Headerless bank loaders.** The hero setup is the only loader path found for these banks (Hero animation data). The routines that load them are not described here, and whether another path loads them is not established.
- Resolved: **Headerless bank header words.** The word at 0x28 is the expansion buffer size (largest compressed *U* plus 0x3C, or 0), and the words at 0x30 and 0x34 are the bone mask of entry 0, not header fields (Headerless resources; Animation set). **[High]**
- **Skeleton header word at 0x14.** Resolved: it is K, the number of floats per bone in the extra block (28 in resource 492), and the block holds per-bone translation scale sets chosen by the character's scale selector (Per-bone translation scale). High.
- **Clip index for body 526.** Resolved: the -7 character records index it by word A bits 0 to 11 of their own tables, with indices 0 to 153 (Hero animation data, Body 526).
- **In-place start.** The copy branch runs when the playing instance has a working buffer. When it has none, the original uses the raw compressed entry, which would fail to walk. Resolved in part: the player gets its buffer when it is bound to its main animation set, sized by the set's word at payload +0x28 (Animation set), and that word is 0 exactly for sets that have no compressed entry, so on disc the case of a compressed entry without a buffer does not arise for the main set. Sub-part and face clips (one compressed entry in bank 501, nine in bank 503) fit the main set's buffer, because the main sets of their families are larger than the largest face or sub-part block (Medium-High). A rewrite that sizes each decode buffer from the entry's own uncompressed size is always safe.
- **Bytes after chain ends.** Most bytes after the last entry of each headerless bank are nonzero, but the chain does not reference them. Nothing references them and their meaning is not encoded anywhere; ignore them. Resolved for entries: taking each bank's last entry to end at its next-link target, no plain entry has bytes after its channels, and the 20 compressed entries that do hold copies of the previous entry's block (Headerless resources).
- **Unidentified misc resources.** Resource 491 (51,200 bytes) has no loader that this spec follows and no meaning encoded anywhere; the rewrite never loads it. Resources 520 to 525 are identified as the hero action tables (Hero animation data).
- **Unplaced compressed entries.** Five compressed entries that were once counted in the headerless total (153) do not occur in the seven banks. They are excluded from the totals in this section. Where they are, if anywhere, cannot be known from the disc, and nothing in play needs them.
- Resolved: **Stored or tiny compressed blocks.** None of the 51,043 shipped compressed animation blocks has *C* ≥ *U* or *U* < 64 (Compressed entries, Block-size count; High). The scan covers all 50,895 chunk entries and the separate headerless-bank audit covers the remaining 148.
- Resolved: **Bounds-checked decoder.** It is dormant: no call, no stored or built address and no module import (Decoder note; High).
- **Helpers not specified.** The mode-1 packed rotation and its translation reader, and the LZO1X decoder's internals (the spec allows any conforming implementation). The pose rate limit's constants and selector-1 pending-switch use are given (Pose rate limit). One selector-1 caller is the Plague Serpent's idle animation and supplies the hero's position (Pose rate limit; SPEC-0016). No other caller or target value is known; the rewrite applies the rate limit only for the Serpent idle, as given in Pose rate limit.
- Resolved: **Pose rate limit, branch.** The sign test uses the signed orientation scalar σ = a · (up × b), where a is the horizontal unit direction from the first sampled pose point to the supplied target, b is the horizontal unit direction from the first sampled pose point to the second sampled pose point, and up = (0, 1, 0). Positive σ negates the arccos angle and selects the upward branch; zero or negative σ selects the downward branch (Pose rate limit; High).
- **Sub-part and face indexing.** Resolved: sub-part and face mask bit *b* drives element *b* − N_main of the family's own list (both families use base N_main); record parents below N_main are main joints, at or above are element *p* − N_main of the same list; and the skin bone table is main, then sub-part, then face, so face element *i* is skin bone N_main + N_sub + *i* (Families; High). The earlier 55 % sub-part test agrees, because for the sub-part list both rules give the same bone.
- **Blend and crossfade in clips with overlapping masks.** Resolved: every consecutive clip on a record crossfades from the held pose (Clip start). All requests use it. High.
- **Entry flags.** Resolved for 0x25 (fade-out length, dormant), 0x26 (0xFF everywhere, not read), 0x28 (main layer index, 0 on disc), and 0x21 (root flag, 0 on disc) (Animation set). In the layer-family play-start path, entry byte +0x28 indexes the 0x60-byte playback record at player +0x40 + 0x60 × k. The shipped value is always 0, selecting main layer 0. These playback records hold per-clip timing and root-step/accumulator state; they are distinct from the five per-weapon frame blocks (Playback records and layers; Weapon frame blocks; High).
- **Mode-0 jumps.** Resolved: a 16-bit overflow in the data. Bone 0's y in resources 1218, 1219 and 1268 (entry 14) wraps from about −7.69 to +7.26 between frames 95 and 96; the engine reads the stored s16, so reproduce the jump as stored (Edge cases; High).
- **Face animations.** Resolved: the 2 % match came from using the wrong base. Under the face base N_main + N_sub, face frame-0 translations match the mesh rest in 70.7 % of bone-clips, and the talking-NPC pose check separates the rules (Families; High).
- **Precedence.** Resolved: sub-part and face write separate lists, so they never overlap; only main layers 0 and 1 share a list, and layer 1 runs second (Families; Playback records and layers). High.
- **Type-22 extra floats and the pose-entry table.** Resolved for the extra floats: they are the per-bone translation scale sets (Per-bone translation scale). Resolved for the pose-entry table: its entries are attach points (Attach points and carrying; High).
- **Stored root group.** The character pose block's four-vector root group is read by the active-hero helper and the region-character path. These paths use it in request and scene-state/draw calls and do not write character travel position; heroes and non-heroes travel through the layer-0 root step above. The root group is a rendering input only, so its writer does not affect travel or any rule; the rewrite derives it as SPEC-0040 (Behavior 10) describes. [High for the known reads and separate travel route; Medium for the direct-writer boundary]
- **Collision job.** Resolved (High): there is no collision job. The routine that was taken for one is the hero's held-item draw, which has no radius and no wall response; the root step only moves the character through its own motion record, whose contacts are in SPEC-0018 and SPEC-0040 (Behaviors 3 and 14C). Test vector: a walk clip's root step of 0.225198 per tick is applied to the position and then limited only by the ground and wall rules of SPEC-0018.
- **Root-sample blocks.** Resolved: they are the weapon frame blocks. The "layer flags" are the table-B damage bits 0x1 to 0x10, and the blocks are per-weapon frames, not root samples, so they are not the stored root group of SPEC-0040 (Weapon frame blocks; High).
- **Root-sample blocks, hero.** Resolved: the same damage-bit query serves any character, the heroes included (Weapon frame blocks; High). The hero's travel does not depend on the blocks: it reads the layer-0 root step directly (Playback 5).
- **Root-sample blocks, mapping.** Resolved: bit 0x1 gives frame 0, 0x2 frame 1, 0x4 frame 2, 0x8 frame 3 and 0x10 frame 4, first set bit in that order, frame 0 when none (Weapon frame blocks; High).
- **Weapon frame writer.** Resolved: the per-frame character render update writes the attachment matrices used as weapon frames; points 0 to 4 occupy the first five slots of the same nine-point bank (Weapon frame blocks; High).
- **Idle picker mappings.** Resolved: the two id lists are given in SPEC-0040, Behavior 3B (villager gesture lists), as individual constants; they are part of the picker's code, not disc data.
- **Idle picker timer.** Resolved (High): the timer is the villager pair's talk or wait duration. It is drawn once per villager when its role leaves 0 for 1 or 2 (uniform 300 to 720 ticks, that is 5 to 12 seconds), counted down by 1 on every tick of the pair's talk step, and when it reaches 0 or below both villagers are set to role 3 (reset), which draws no timer. It is the same timer as step 6 of the villager conversation (SPEC-0040, Behavior 3B). Test vector: a villager leaves role 0 with the draw 400: it counts 399, 398, ... and on the 400th tick of the talk step it reaches 0; both villagers then take role 3.
- **Idle picker callers.** Resolved (High): the mode setter is called by the villager pairing and talk routines (SPEC-0040, Behavior 3B): the pairing step gives one villager role 1 and the other role 2 (a random choice), and the end of the timer sets role 3. Mode 1 and mode 2 are those two roles.
- **Table size rules.** Which tables the three size rules (see "Table size rules") allocate, what the fixed 224-byte block is and the alignment values passed by the callers are memory-layout details of the original allocator; a rewrite sizes its own tables from the stated counts, so they do not affect behavior (D).
- **Model-set parameters.** The meaning of 30, 100 and 150 in the model-set setup, and which owner holds the eight slots, is developer intent and not encoded anywhere; no rule in the other specs depends on them (D).
- **Two-slot flip.** Resolved: it serves the primary character animation stream, and state 2 waits for a positive available-record count in the alternate window before switching and refilling the previous one (Headerless resources; High for the tested condition and primary-object use, Medium for transfer lifecycle). Which clip banks use this path and the exact transfer-completion timing are disc-streaming details that do not change which clip plays; a rewrite that holds all banks in memory ignores the flip (needs a runtime check for any visible stall).
- **Resolved in part: Root matrix absent.** With a null root argument, the root output slot remains the local matrix and descendants compose from it, without external placement. All known direct callers provide a matrix; the checked character update paths pass their root matrix workspace (including +0x260 in the attachment/chain paths). No caller that passes null is known; the case is not reachable from any followed path and the rewrite always supplies a root matrix (Forward kinematics rules; High for routine behavior and known callers).
- **Joint cap.** The composition helper accepts joint indices 0 through 82. A request for index 83 or greater returns before reading the skeleton record or writing that joint's output. The largest shipped skeleton has 74 joints (indices 0 through 73), so the guard does not affect shipped skeletons. Malformed data with an out-of-range joint count or parent index does not occur on the disc, so no result is defined for it; reject such data. **[High for the helper guard; Medium-High for the shipped-data boundary]**
- **Resolved: Parent-table width and source.** Active character composition reads the parent index from the 12-byte records reached through the runtime skeleton header (main header +0x10); its parent is a u32 and the root comparison is full-word equality to 0x000000FF. This is the type-22/23/24 skeleton-definition family table (or equivalent headerless family header), separate from the type-18 mesh bone table, which supplies rest positions. The no-root-matrix behavior is stated under Root matrix absent (Parent source and root comparison; High).
- **Per-bone bounds.** Which bone extents the radius reads and which code uses the radius are not known to affect any rule: no hit test, cull or draw rule in the other specs uses it, so the rewrite computes nothing from it (D). Low.
- Resolved in part: **Animation set not initialised.** Both current-index and previous-index main lookups return 0 and report on each call while the main set flag is -1. Direct callers include actor update, controller and render paths; the complete indirect/script caller reach remains open (Missing ids and sets; High for the per-call behavior, Medium for the caller boundary).
- **Pose-entry lookup.** Resolved (High): the carry placement builds the attach frame from both the angles and the translation of entry k, with k chosen by the carrier's damage bits (Attach points and carrying). The separate single-vector lookup returns the translation (posed world position, fourth lane 1) of weapon frame k, and its index k comes from the same damage bits: frames 0 to 4 for the bits 0x1, 0x2, 0x4, 0x8 and 0x10 in that order, frame 0 when none is set (Weapon frame blocks, Layout). The full-frame lookup tests only 0x1, 0x2 and 0x4. Test vector: bits 0x4 only give k = 2, the position at sub-object +0x14A0.
- **Attach frame maths.** Resolved for the pose-entry transform: the bone-local offset is added before the entry rotation, with R = Rz(rz) × Ry(ry) × Rx(rx), and W = B × Translate(t) × R (Attach points and carrying; High). The separate optional Miamouse carry vector is initialized to (0, 0, 0, 1) by a callback in the one-time frame-update initializer; its first three components feed the carry path's axis-rotation builders. A static boot-image reference scan found no other direct reference to the vector. Still open: fourth-component role, possible computed/indirect later writer, and complete optional-offset composition. [High for initializer and checked references; Low-Medium for absence of other writers]
- **Attach points and weapon bits.** Resolved: damage bits 0x2 to 0x10 choose the carrier's attach point 1 to 4 (point 0 by default); the users are the grab-and-throw enemy (hero state 53) and the Miamouse carry (hero state 57) (Attach points and carrying; High).
- **Skinning job owner and selector.** Resolved in part: the skin job is the record named by each mesh node's +0x50 word, and the character model draw stores the selector word (0x2EC830) from its third argument before the skin stream and the draw walk; on disc the two result slots are identical, so the value does not change results (Skinning upload, item 5; Medium). The value the draw's caller passes cannot change the picture on this disc, so it is not needed.
- **Skinning result arrays.** Resolved for implementation: the two result arrays are internal to the vertex program's palette hand-off; a clean build skins with the formula in Skinning (weights times bone matrix times rest vertex), which gives the same vertices, and needs neither array. The internal split is not encoded anywhere outside that program.
- **Root-motion query use.** Resolved for character travel: the seven outputs of this query are not shown feeding the live hero travel step. The known query wrapper calls it with frame 0, discards its stack output, and has no incoming direct reference in the extracted call index; broader indirect or external entry remains possible. The live travel path instead consumes the sampled layer-0 root step described in Playback 5 (SPEC-0040, Behavior 14C). [High for the known path; Medium for the static reachability boundary]
- **Root-motion output count.** Resolved: no output is consumed. The query is dormant (see its section), so the seven delta outputs feed nothing. [Medium-High]
- **Root-motion time value.** Resolved: output 13 is never read because the query is dormant; the helper and its two field arguments are not needed. [Medium-High]
- **Root-motion timing records.** Resolved: nothing reads the interval records or the running total; they are diagnostic bookkeeping of a dormant routine. [Medium-High]
- **Joint pose sample arithmetic.** Resolved: the routine is dormant (no caller and no stored or computed address), so a clean side does not need its arithmetic (Joint pose sample; High).
- Resolved: **Joint pose sample, run-time words.** The three words at 0x3932E0, 0x3932E8 and 0x3932EC are the x, z and w components of the up vector (0, 1, 0, 0), written once at boot (SPEC-0007); all three are 0.0. [High]
- Partly resolved: **Skinning and pose order.** The shared skeleton pose pass writes the three composed joint-transform output arrays (main, sub-part and face), after animation playback updates. The three arrays are the pose matrices that the character part draw reads for every character (SPEC-0046). The skinning upload job reads its own separate 3x4 bone-matrix array. The separate joint pose-sample routine is dormant and is not part of this order. [High for the pose-output writer, its users and pass order]
- **Sub-part and face mirroring.** Resolved: unreachable on the shipped data. The only second main ids (19, 20 and 25) have sub-part entries with bit 12 clear, hero face tables have no bit-12 entry, and no face override written by the emote consumer names a face id with bit 12 set in its table (Mirrored clips; Medium-High).
- **Dormant starts.** The fade-out starts and the explicit-weight starts have no caller. The explicit-layer, no-fade start has one other caller in a routine with no known static caller, so it does not run in play; the rewrite leaves all three starts out. [Medium-High]
- **Second main id.** Resolved: only the hero air attacks set it, Roberta's (19 or 20) for one tick and Tor's (25) for the whole air slash in the strafe jumps (Clip start; High).
- **Tor's air slash.** Resolved: it runs only in the strafe jumps 43 to 46, keeps the strafe-jump state with sub-state 11 and second main id 25 until landing, and the main clip and clip 0 restart each other every tick there; it never happens in states 7 to 11 (Clip start; SPEC-0033 Behavior 50; High).
- **Hero ids with unconfirmed roles.** Resolved: 6 reverse walk; 12, 17, 27 and 30 attack ids; 64 shield-block impact; 147 and 148 spin ends; 97 and 98 never requested for a hero (NPC modes 11 and 12 only); the state-57 id 12 is the carried Miamouse's request (Hero action ids by role; High).
- **Held-item bit 0x1000.** Resolved in part: the region-character draw checks this event across the character's four playback records. A passing event enters the branch only when a character-method result is below 5 and the character's selected model set supplies a non-null slot-1 model. Slot 1 is populated from the character's selected type-18 model set by a type-8 child whose stored index maps to that slot; the selected model-set index comes from the character record's model-set field, independently of its animation-family indices. A second character method supplies a matrix for the draw call. This static gate and submission path do not prove visible use. Still open: which character records select a model set with a usable slot-1 model, the meaning of the selector result, the concrete transform method/composition, and whether the item is visibly drawn in play. [High for the pointer binding and branch; Low-Medium for the inventory and transform; runtime visibility not checked]
- **Finished root doubling in play.** Resolved for hero main tables at the data level: one-shot action ids with a non-zero final-key xyz root record are Tor 12, 14–17, 20, 26–27, 32–33, 50–51, 54–55, 58–59, 64, 69, 74–75, 90–91, 94–95, 102, 104, 118, 134–135, 139–141, 168 and 178; Roberta 12–13, 16–17, 23, 26, 28, 31, 34, 43–44, 50–51, 54–55, 57, 59, 66–67, 70, 74–75, 77, 90–91, 94–95, 118, 134, 140, 168–169, 178 and 182. If one of these one-shots remains selected after it finishes, the main-layer rule doubles its final-key root step while the finished flag and nonzero clip speed remain. This inventory does not establish that every listed action reaches or stays at its finished key in play. Whether a listed action rests on its final key in play depends on the hero state machine (needs a runtime check). Default if unresolved (rewrite choice): apply the doubling rule exactly as stated whenever the finished flag is set with nonzero speed, so the result is right whichever way the state machine behaves. The region-character main-bank data inventory is not listed because the rule applies the same way to every bank (Layer step, step 8; SPEC-0040, Behavior 14C; High for the data and rule, Medium for in-play reachability).
- **Normalisation.** Resolved: nothing normalises rotations, including the quaternion-to-matrix conversion, which applies the standard formula to the raw values: for |q|² = s the matrix is (1 − s) I + s R(q / |q|) (Posing a skeleton; High for the conversion, Medium-High for the other steps).
1. Clip mode meanings: Resolved. Mode 9 never persists; a request ends in mode 8 (finished at L) or mode 10 (stopped at t), and the per-tick rules of modes 0, 2, 4, 5, 6, 8 and 10 and the callers are given in Clip time and mode (High).
2. Resolved: after the vector or quaternion setter runs in modes 4, 8 or 10, a zero-delta current-clip update clamps stored time to [0, L] and immediately evaluates the pose at the resulting time; an in-range time is unchanged, and these modes stay unchanged (Clip state on a request; Test vector 15). The companion byte records which of the vector and angle/quaternion setters have run since its last clear; no downstream consumer of the companion byte is known, so the rewrite does not need it. The pose is read by the normal pose pass. **[High for the update, clamp and latch transitions; Medium-High for immediate pose visibility; Low for downstream use]**
3. Resolved (R-0127): the tag-0x0A range table is the area's path list, not a clip table; its first array holds each path's first entry index and its second array the last, and the only reader is the walking character's route set-up (SPEC-0006, Tagged tables). No clip start or frame count comes from it. [High]
4. Rate pair unit: Resolved for the animated-object sound caller (SPEC-0031). It scales the start and end fractions by the object clip duration D = (F − 1) / 15 seconds; the sound table has three slots per animated object. The available caller and table support does not connect this initializer to SPEC-0018 type-5 height patches, so those patches have no rate pair; the question is closed.
5. Resolved: The random candidate picker is used by the enemy item-steal flow and is specified in SPEC-0029, Behavior 11b; its selector and item-selection limits are stated there.
6. Object record copy: the transposed copy range (+0x40 to +0x7F) is described as far as any rule depends on it; the helper that runs before the copy is a memory-layout step with no effect on behavior, so the rewrite needs nothing more (D).
7. Secondary motion: Resolved for modes 1 and 2 (the pendulum chains of Tor and Roberta; SPEC-0040, Behavior 25) and for the choice of bones (Gnarfas's four chains, Secondary-motion chains). Resolved for mode 0: for each non-final link, the step derives a correction rotation that turns the current link direction toward the constrained new link direction, composes that correction with the link bone's current local rotation, linearly blends the current and corrected quaternions at weight 0.1, then normalizes the result before writing it to the pose. Parallel and antiparallel direction cases have explicit handling in the direction-to-rotation helper. The blend is not spherical interpolation. [Medium-High; exact behavior and limits are in Secondary-motion chains.]



