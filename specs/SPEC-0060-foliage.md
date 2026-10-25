# SPEC-0060: Foliage patches (type 16)

- Answers: SPEC-0006 open questions on the type-16 fields, build selector, thresholds, query point, items and purpose; SPEC-0028 open questions 57 to 59 (six-slot look-ahead) and 80 (type-16 field reading); proactive otherwise (bush cutting, drops, regrowth, trees and reeds)
- Status: Draft
- Disc build: SCUS-97367 v1.00 (same fingerprint as SPEC-0001). Check the three data words in Where it lives before trusting the values.
- Confidence: per section. Record layout and field roles: High (counted on all 75 chunks). Build dispatch, families and model ids: High. Once-per-area gates and the slot quirk: High. Placement: Medium-High (jitter and scale rules High; the per-model ground probe Medium). Per-frame gates: High for the order, Medium-High for the source of the eye point. Fade bands: High. Grass logic: Medium-High. Bush rules (state machine, cut triggers, drops, regrowth, sparkle): High; hero bend formula Medium; plant-side melee capsule geometry High. Trees: Medium-High. Reeds: Medium-High (fall states High). Flowers: Low.
- Related specs: SPEC-0006 (Box records, Binary grids and the zone query, Area-pair test), SPEC-0028 (section 16 and the launched-object after-flight fall, both superseded here), SPEC-0021 (item ids, coin values, Clover patches, Stat-up items, Spawn routes, item model registry), SPEC-0025 (once-per-area sets in the save), SPEC-0036 (attack window, hero shot slots, luck), SPEC-0018 (ground probes, ceilings, cylinder contact), SPEC-0004 (type-8 model containers), SPEC-0034 (effect group 3, effect 139), SPEC-0046 (draw), SPEC-0027 (camera eye, near-tree list), SPEC-0026 (main loop draw step; its old "timed transition driver" is the bush step of this spec), SPEC-0007 (60 ticks per second).

## Summary

Every type-16 record of a world pack is one **foliage patch**: a box of plant instances laid out in rows. The record's key (0 to 39) picks one of six families, each with fixed model ids from the item model registry:

- **G grass** (keys 0 and 12): swaying grass with a close-range alpha fade.
- **S flowers** (keys 1, 6 and 7).
- **B bushes** (22 keys): cuttable bushes that drop coins or items, shrink away and regrow. They include the coin bush and the Red, Golden and Starry Negg bushes.
- **H trees** (keys 4, 13 and 25): key 4 trees have a solid trunk.
- **K reeds** (key 8): reeds that bend away from the hero and can be cut.
- **C rustle and clover beds** (keys 10, 11, 14, 23, 24, 26, 27 and 35): these follow SPEC-0021 Clover patches.

Each frame a patch is considered only when its zone is visible and the camera eye is within its far distance. Within 100 units, and while the game-state code is below 9, its family logic runs. Its plants are then drawn with a near/far model fade. Clover and Negg patches whose item has already been gained in the area are not built. Times are in ticks of 1/60 s (SPEC-0007).

## Where it lives on the disc

- Type-16 chunks of the world packs: 75 chunks in 74 packs (one pack has two), 4,205 records, 18,287 rows and 29,853 plants. Up to ten chunks per world are kept (SPEC-0006).
- Plant models: type-8 children of the world pack (SPEC-0004). A child registers under the registry id in its u16 at chunk +0x0E. The grass model, registry id 283, comes from resource 9.
- Boot data words (ELF virtual addresses; read them at run time):
  - 0x2D7540: grass pass enable. It is 1 on disc.
  - 0x2EB340: shot test scale along the bush's up axis. It is 0.0 on disc.
  - 0x2EB344: shot test margin. It is 1.5 on disc.
- All other values in this spec are fixed constants of the program. They are given as values. The drop tables are among them, so they are described by their rules.

## Layout

All offsets in a type-16 chunk, including every stored offset, are relative to the chunk header, which is 16 bytes before the payload (SPEC-0006, Box records). The chunk header and the record table position are given there.

### Patch record (0x40 bytes)

| Offset | Size | Meaning | Confidence |
| --- | --- | --- | --- |
| +0x00 | 4 x f32 | Box x0, x1, z0, z1 (x min, x max, z min, z max). Every row and plant lies inside it. | High |
| +0x10 | u32 | Pool count = number of rows (1 to 18, and 27) | High |
| +0x14 | u32 | Header-relative offset of the first row | High |
| +0x18 | u32 | Key, 0 to 39: selects the family and models (Behavior 1) | High |
| +0x1C | f32 | Patch height y: the base height of the patch before ground snapping. It is not a radius. | High |
| +0x20 | u32 | Zone number: the type-35 column of the zone query (SPEC-0006, Binary grids). Only the low byte is used. Values on disc are 0 to 17 and 48 to 57. In all 4,008 records of packs with a type-35 grid it is below block 0's width. | High |
| +0x24 | u32 | Jitter flag (0 in 4,115 records, 1 in 90). When it is nonzero, the plants of the B, C and H families get a jitter half-range J = 0.4 × (z1 − z0) / pool count, and the S family gets J = 0.27 × (z1 − z0) / pool count. | High |
| +0x28, +0x2C, +0x30, +0x34 | 4 x f32 | Draw-distance overrides A, B, C, D. They are used when A or D is nonzero (117 records). Otherwise they are zero. | High |
| +0x38 | u32 | Fill (0xCDCD0000 in most records) | High |
| +0x3C | u32 | Never read by the game: the chunk loader, the record counter, the build dispatch, the family builders and the placement step all stop at +0x34. 0xFFFFFFFF in packs 678 and 713. | Medium-High |

Override sets on disc (A, B, C, D):

| Set | Records |
| --- | --- |
| (200, 32, 50, 150) | 80 |
| (50, 16, 10, 100) | 22 |
| (100, 16, 10, 200) | 7 |
| (100, 16, 20, 150) | 4 |
| (200, 32, 75, 250) | 2 |
| (200, 32, 10, 4000) | 1 |
| (50, 16, 20, 100) | 1 |

### Row (pool entry, 0x14 bytes)

| Offset | Size | Meaning |
| --- | --- | --- |
| +0x00 | u32 | Plant count of the row |
| +0x04 | f32 | Row offset: the row lies at z = z0 + this value |
| +0x08, +0x0C | 2 x u32 | Fill |
| +0x10 | u32 | Header-relative offset of the row's plants (plant count x 16 bytes) |

### Plant (16 bytes)

| Offset | Size | Meaning |
| --- | --- | --- |
| +0x00 | f32 | Plant offset: the plant lies at x = x0 + this value |
| +0x04 | f32 | 3.0 in all 29,853 plants. Never read. |
| +0x08 | 4 bytes | Colour of the instance: R, G, B, A in byte order, 128 = full (Medium for the channel order). Example: the word 0x80404040 gives R 64, G 64, B 64, A 128. |
| +0x0C | f32 | 173.48 in all 29,853 plants. Never read. |

Disc checks: 18,287 of 18,287 rows satisfy z0 ≤ z0 + row offset ≤ z1, and 29,853 of 29,853 plants satisfy x0 ≤ x0 + plant offset ≤ x1.

### Bush instance (run-time, 0xD0 bytes; for reference)

| Offset | Meaning |
| --- | --- |
| +0x00 | 4 x 4 transform: rows right, up, forward, position |
| +0x50 | Draw scale, four floats (s, s, s, 0.0) |
| +0x70 | Cut flag |
| +0x74 | Owning patch |
| +0x7C | Base scale K (the random placement scale) |
| +0xA0 | Bend latch |
| +0xA4 | Cut countdown |
| +0xA8 | State: 0 cut, 1 regrowing, 2 grown |
| +0xAC | Regrow counter |
| +0xB0 | Effect wait, in ticks |

## Behavior

### 1. Build at world load (High)

Each type-16 chunk takes a free world slot (up to ten). Every record builds one patch, unless a rule below skips it. Each patch keeps its world. The key picks the family and its models (registry ids):

| Key | Family | Models | Built when | Records on disc |
| --- | --- | --- | --- | --- |
| 0, 12 | G grass | far model 283 (resource 9) | always. Key 0 snaps to the ground; key 12 does not. | 0: 811 (30 packs); 12: 122 (5 packs) |
| 1, 6, 7 | S flowers | 194 (keys 1 and 7), 193 (key 6) | the model belongs to this world | 6: 55; 7: 34; 1: none |
| 2, 3, 5, 9, 16 to 22, 28 to 34, 36 to 39 | B bushes | see the bush table below | always built. Without its main model, a patch has no models and an inner distance of 0, so it is inert. | 1,437 (inert: 66 of key 19, 2 of key 22, 1 of key 28, all 7 of key 32) |
| 4, 13, 25 | H trees | 4: near 200 plus a trunk; 25: near 200, far 199; 13: far 220 | always (the near model is absent or belongs to this world) | 4: 226 (15 packs); 25: 458 (18 packs); 13: 8 (pack 721) |
| 8 | K reeds | near 202, far 203 | model 202 belongs to this world | 24 (packs 909, 934 and 1056) |
| 10, 11, 14, 23, 24, 26, 27, 35 | C rustle and clover beds | 10: 197; 11: 204; 14: 221; 23: 233; 24: 234; 26: 235; 27: 239 (Four-Leaf Clover); 35: 268 (Twelve-Leaf Clover) | the model belongs to this world | 10: 62; 11: 325 (106 not built); 14: 463; 23: 166 (57 not built); 26: 1; 27: 10 (10 packs); 35: 3 (packs 706, 1184, 1300) |
| 15, 40 and up | none | none | never | none on disc |

Bush models (main / second / third). The second model is the cut model.

| Key | Models | Key | Models |
| --- | --- | --- | --- |
| 3, 9 | 196 / 192 / 191 | 28 | 248 / 249 / 250 |
| 2, 29 | 195 / 244 / 245 | 30 | 253 / 254 / 255 |
| 5 | 198 / 201 / 225 | 31 | 256 / 257 / 258 |
| 16 | 222 / 251 / 252 | 32 | 259 / 260 / 261 |
| 17 | 223 / 246 / 247 | 33 | 262 / 263 / 264 (coin bush) |
| 18 | 224 / none / none | 34 | 265 / 266 / 267 |
| 19 | 226 / 242 / 243 | 36 | 271 / 272 / 273 (Red Negg bush) |
| 20 | 227 / 228 / 236 | 37 | 274 / 275 / 276 |
| 21 | 229 / 230 / 237 | 38 | 277 / 278 / 279 (Golden Negg bush) |
| 22 | 231 / 232 / 238 | 39 | 280 / 281 / 282 (Starry Negg bush) |

Bush records on disc by key: 9: 866; 5: 199; 16: 123; 19: 70; 20: 58; 17: 44; 33: 23 (22 packs; pack 893 has two); 21: 21; 31: 9; 28: 7; 32: 7; 22: 3; 39: 3 (packs 1225, 1238, 1253); 30: 2; 36: 2 (packs 714, 909). Keys 1, 2, 3, 18, 24, 29, 34, 37 and 38 do not occur on disc.

"Belongs to this world" means that the registry entry for that id was registered by this pack: the model is a type-8 child of this pack, and its material list is this pack's (SPEC-0004).

**Once-per-area gates (High).** These patches are not built when the current area slot's bit is set in the item's once-per-area set (SPEC-0021 Stat-up items, SPEC-0025):

| Key | Item |
| --- | --- |
| 27 | 19 (Four-Leaf Clover) |
| 35 | 20 (Twelve-Leaf Clover) |
| 36 | 174 (Red Negg) |
| 38 | 151 (Golden Negg) |
| 39 | 154 (Starry Negg) |

A clover or Negg patch whose item was gained in the area is therefore missing from the next load of the area. Quirk (reproduce): the gate tests the slot of the world that is current at build time, not the slot of the world being built (Edge cases).

### 2. Placement (Medium-High)

1. Centre C = ((x0 + x1)/2, y, (z0 + z1)/2), with y = +0x1C.
2. Patch radius = 0.7072 × max(x1 − x0, z1 − z0). The bounding radius used by the draw is the patch radius plus 0.5 × the near model's bounding size.
3. Plant k of row r starts at (x0 + plant offset, y, z0 + row offset).
4. Jitter: x and z each get a uniform value in [−J, J]. J = 0.9 for G and 0 for K. For B, C and H, J follows the +0x24 rule (0.4 × depth / rows). For S it is 0.27 × depth / rows. J is 0 when the flag is 0.
5. Ground snap (families and keys that snap; G key 12 does not): y becomes the ground height under (x, z) from the column probe of SPEC-0018. Bushes of models 198 (key 5) and 223 (key 17) probe from y + 8 instead (SPEC-0018, Ceilings supplied by callers; Medium). A plant whose ground is −1.0e8 or below (no ground) is not placed.
6. Each plant gets a random yaw (a random horizontal facing).
7. Each plant gets a uniform scale K in [s0, 1]. s0 is 1.0 for G, 0.6 for S and B, 0.85 for C and H, and 0.73 for K. Bushes of model 265 (key 34, not on disc) use [1.0, 3.5].
8. The plant's colour is its RGBA bytes.
9. A new bush starts in state 2 (grown) with counter 0, cut flag 0, countdown 0 and bend latch 0. Its effect wait is trunc((u + 0.5) × 60), with u uniform in [−0.2, 0.2], which gives 18 to 42 ticks.

### 3. Distances and fade bands (High)

With overrides (A or D nonzero), the four distances are:

| Distance | Value |
| --- | --- |
| Far distance F | D |
| Switch distance S | B + A |
| Out band O | B |
| Fade band W | C |

Without overrides they come from the near model's LOD node (SPEC-0004). With no model, F = 4000.

**Per-patch distances (High).** Every family build passes an inner (logic) distance of 100 for every patch, and 0 for an inert bush patch; no record field changes it. The patch constructor first sets defaults (inner 100, far 500, switch 100, out band 30, fade band 30), but the patch set-up of every built patch replaces them with the values above (overrides, or the LOD node, or F = 4000). So the defaults are never seen in play. Without overrides and without a far object, the LOD values are adjusted (Medium): O becomes max(O, W), W becomes that same value, and S is lowered to F − O when it is larger.

**Which patches have a far object (High).** A patch has a near model (the family's main model) and, for the families that pass one, a separate far object. Bush patches never have a far object, so every bush patch uses the near-only weights. Two-model weights apply only to the grass, tree and reed families that pass a far model (table in Behavior 1).

Weights for a plant at distance d from the eye (near weight a, far weight b):

- Only a near model: b = 0. a = 1 up to F − W, then a = 1 − (d − (F − W)) / W, clamped at 0.
- Only a far model: a = 0 and b = 1.
- Both models:
  - d < S − O: near only (a = 1, b = 0);
  - S − O ≤ d < S: a = 1 and b = (d − (S − O)) / O;
  - S ≤ d ≤ S + W: b = 1 and a = (S + W − d) / W;
  - beyond S + W: far only (a = 0, b = 1).
- No model: a = b = 1.

### 4. Per frame (High for the order; Medium-High for the eye source)

The foliage pass runs inside the world draw step of the main loop (SPEC-0026), once per tick. It covers each loaded world: the current world first, then the second world when the area-pair test passes (SPEC-0006). E is the camera eye position (SPEC-0027): the position row of the one camera object's matrix for this frame. The game has a single camera object that every camera mode, cut-scene camera included, moves, so E is the camera position in every mode (Medium-High). The G patches run in a separate pass that runs only while the data word at 0x2D7540 is nonzero (1 on disc).

For each patch:

1. **Gate.** The patch goes on only when all three of these hold:
   - It has a model or a material.
   - The zone query passes, with column = the zone (+0x20, low byte). The key is −1 when the patch's world is the current world, and that world's id otherwise (SPEC-0006, Query).
   - |C − E|² < F².
2. **Logic.** When |C − E|² < 100² and the game-state code is below 9, the family logic (sections 5 to 9) runs. An inert patch (inner distance 0) never runs it.
3. **Draw.** The patch's bounding sphere must lie inside the view frustum. Each plant is then drawn with its weights a and b (section 3; SPEC-0046). A plant is skipped when its own visibility test fails (a reed whose fall has ended, section 9). With a > 0 the plant is drawn directly with the patch's near model. With b > 0 the plant is added to a shared far batch instead: grass patches use one batch (room for 1,000 entries) and every other family uses another (room for 700). Both batches are created once at boot, emptied at the start of every frame and drawn once per frame after the patches (Medium-High).
   - **Far entry.** On the add, the far object's alpha byte is set to trunc(plant alpha × b), with the plant alpha the plant instance's own alpha byte (Medium that it is the A byte of the plant colour).
   - **Full batch (Medium-High).** A batch keeps a count of entries and a list of at most 20 groups; entries with the same far model, material and draw slot share a group. An add is refused when the count has reached the room (700 or 1,000), or when the entry needs a 21st group. A refused plant is simply not drawn on that frame: no error, no fallback to the near model, and the patch's other plants go on. Plants are added in patch order and row order, so the first entries of the frame are kept and the last ones are lost.

### 5. Grass, G (Medium-High)

- A plant within 20 units of the view point takes the patch's sway phase.
- A plant within 2.0 units has its alpha scaled. The factor is 0 within 0.25, and (d − 0.25) / 1.75 from 0.25 to 2.0.

### 6. Bushes, B (High unless marked)

**State machine.** State 2 is grown, state 0 is cut and state 1 is regrowing. Each logic step does the following, in order:

1. **Helpers.** They are skipped when the state is 1 and the regrow counter is at most 450 at the start of the step. A regrowing bush therefore cannot be hit, cut or bent until its counter passes 450 of 600.
   - Cut flag set: the up axis springs back (step below), and the countdown drops by 1. When it reaches 0, the cut flag becomes 0, the state 1 and the counter 0.
   - Otherwise, for every bush key except 5: spring-back, then the hero bend and the weapon test within 5 units, then the shot test.
   - Otherwise (key 5, model 198): one contact helper, active only while the hero is within 5 units. It runs the weapon test only, so Roberta's shots never cut a key-5 bush. It does no spring-back while the hero is farther than 5 units: beyond 5 units (squared distance 25 or more from the bush's raised test point) the helper returns at once and the bush keeps its tilt (High). Within 5 units, on each step (Medium):
     - The target axis T is the current up axis. When the hero's body test against the bush (a segment test with radius 0.75) hits and the hero's speed is above 0.1 units per second, T = 0.76 × U + 0.24 × the hero's facing direction instead, with U the world up vector (0, 1, 0, 0), a constant set once at boot (High). The speed is the length of the hero's movement-body velocity, the same velocity that moves the hero each tick (High for the value; Medium-High that it is the hero's own body). A hero standing still against the bush, or turning on the spot, does not bend it.
     - When T.y is below 0.994, the up axis becomes normalize(0.02 × U + 0.98 × T) and the bush's frame is rebuilt around it. So the bush both bends under contact and springs back while the hero is near, but only by 2% per step.
     - The weapon test then runs.
2. **Scale.**
   - Cut flag set: v = K × countdown × (1/3800).
   - Otherwise, in state 1: the counter rises by 1 while it is below 600; at 600 the state becomes 2. Then v = K × counter × (1/600).
   - Otherwise, v = K.

   Products are single precision, with no rounding to integers. The draw scale becomes (v, v, v, 0.0).
3. **Effect wait.**
   - Wait above 0: the wait drops by 1.
   - Wait 0 in state 2, when the bush may sparkle (below): effect 139 of effect group 3 (SPEC-0034) starts at the bush, facing the camera. The wait is re-armed to trunc((u + 0.5) × 60), 18 to 42.
   - Look (High for the values, Medium for the picture; SPEC-0034 section 38): definition 139 makes one small pale-yellow additive spark per start (colour (1.0, 1.0, 0.643), life 2.0 ± 0.3 s). It is invisible for about the first third of its life, then shows at full alpha, drifts sideways at about 2.5 units/s under strong drag with a slight upward pull, shrinks from size 1.0 to 0.5 and fades out in its last 6.5% of life. Shown within 70 units only. So a grown coin bush or an unclaimed Negg bush twinkles with a new spark every 0.3 to 0.7 s.
   - Test vector (computed, life exactly 2.0 s): the spark's alpha is 0 through tick 40 after the start, about 0.977 from tick 46 to tick 112, and 0 again on tick 120.
   - Any other case: nothing happens.

A coin bush (model 262) may always sparkle. A Negg bush may sparkle only while the current world's bit is clear in the once-per-area set of its item: 174 for model 271 (Red Negg), 151 for model 277 (Golden Negg) and 154 for model 280 (Starry Negg). Because of the re-arm, the effect repeats every (wait + 1) ticks while the bush stays grown. In the other states the wait stays at 0, so the first effect comes on the tick the bush is fully grown again.

**Draw.** The draw multiplies the instance's right, up and forward rows by scale lanes 0, 1 and 2. In state 0 the cut model (the second model) is drawn in place of the main model.

**Spring-back.** A bush whose up axis is tilted, and which is not held by the hero bend, springs back each step: up = normalize(0.925 × up + 0.075 × (0, 1, 0)). Within 5 units the hero bends a bush away from himself or herself and sets the bend latch (Medium for the bend formula).

**Cut triggers.**
- **Weapon.** Tor's or Roberta's weapon hits the bush inside the attack window (SPEC-0036). The weapon test compares the hero weapon segment with a bush capsule of radius 0.5 and length 6.0. The bush capsule is centered 3.0 units along the bush up axis from its base, so it spans the base to 6.0 units above it along that axis (High). The hit cue is 20 for Tor and 28 for Roberta, played at the bush.
- **Shots.** The shot test applies to every bush key except 5. Each of Roberta's six shot slots (SPEC-0036, Hero shot slots) that is armed is tested. The test point is the bush position + s × its up axis, where s is the word at 0x2EB340 (0.0 on disc, so the point is the bush base). The slot cuts the bush when the distance from the test point to the shot position is strictly below the shot's radius + m, where m is the word at 0x2EB344 (1.5 on disc).

**Cut.** A cut sets the counter to 0, the countdown to 3,800, the cut flag to 1 and the state to 0. It throws two of the patch's eight debris pieces and rolls one drop. Quirk (reproduce; Medium-High): every trigger that fires in one step cuts again. The weapon test fires first, then each shot slot in range. Each cut rolls its own drop and throws two more debris pieces, so a weapon hit plus two shots on the same step give three drops.

From one cut, the cut model shrinks from K to 0 over 3,800 ticks (63.3 s). Then the main model regrows from 0 to K over 600 ticks (10 s), and the bush is grown again 4,400 ticks after the cut. This holds only while the logic keeps running (section 4).

### 7. Bush drops (High)

One roll r is a uniform integer from 0 to 100, both ends included (SPEC-0021). The drop depends on the bush's main model. A drop spawns at the bush through the item spawn route of SPEC-0021. Coin values: item 135 = 1 NP, 136 = 5 NP, 138 = 25 NP (SPEC-0021, Coin pickups). "Nothing" means no drop.

| Main model | Rule |
| --- | --- |
| 196, 195, 198, 222, 224, 226 (and any other not listed) | r = 0: item 138. 1 ≤ r ≤ 5: item 136. 6 ≤ r ≤ 19: item 135. Otherwise nothing. |
| 223, 227, 265 | r ≤ 9: item 149. 10 ≤ r ≤ 24: item 150. Otherwise nothing. |
| 229 | r ≤ 19: item 146 |
| 231 | r ≤ 14: item 156 |
| 248 | r ≤ 19: item 157 |
| 253 | r ≤ 49: item 147 |
| 256, 274 | r = 0: item 152 (Silver Negg). 1 ≤ r ≤ 6: item 155. 7 ≤ r ≤ 24: item 153. Otherwise nothing. |
| 259 | r ≤ 1: item 138. 2 ≤ r ≤ 11: item 136. 12 ≤ r ≤ 39: item 135. Otherwise nothing. |
| 262 (coin bush) | v = floor(r/2) + 3 × luck (luck 0 to 24, SPEC-0036). Four coins of one kind: item 135 (1 NP) when v ≤ 30, item 136 (5 NP) when 31 ≤ v ≤ 60, item 138 (25 NP) when v ≥ 61. The first coin spawns at the bush. Each of the other three is offset by a uniform value in [0.2, 0.7] in x and in z. |
| 271 | item 174 (Red Negg), only while the current world's bit of item 174 is clear |
| 277 | item 151 (Golden Negg), only while the current world's bit of item 151 is clear |
| 280 | item 154 (Starry Negg), only while the current world's bit of item 154 is clear |

The Negg bushes do not use r. Picking up the Negg sets its once-per-area bit (SPEC-0021), so the bush stops sparkling and later cuts drop nothing. The patch is not built on the next load (section 1).

### 8. Trees, H (Medium-High)

- Key 4 builds one solid vertical cylinder primitive for the patch. Its flags are 0x0001 (solid), its mode byte is 4 and its height is 25.
- In each logic step of a key-4 patch, for each tree of the patch in turn, the cylinder is moved to the tree, given radius 5.5 × the tree's scale, and tested against the hero (cylinder contact, SPEC-0018). Trunks are therefore solid only near the camera (section 4).
- Each logic step of a tree patch also adds the patch to the world's near-tree list (at most 15 entries, rebuilt each pass; Medium for which tree keys do this). The camera's collision reads this list (SPEC-0027).

### 9. Reeds, K, and the cut-piece fall (Medium-High; fall High)

- **Bend.** A reed within 5 units of the hero tilts away. When it is free it springs back as a bush does.
- **Cut.** A hit by the hero's weapon in the attack window cuts the reed. The cue is 22 (Tor) or 28 (Roberta). The cut piece is launched along the blade direction with a velocity.
- **Fall of the cut piece**, in three states:
  1. State 0: position += 0.05 × velocity per tick, until the piece is more than 0.5 from its anchor.
  2. State 1: y −= 0.07 per tick, until y is below (the ground probe result with ceiling y + 8.0) − 1.25 (SPEC-0018, Ceilings supplied by callers).
  3. State 2: up = normalize(0.965 × up + 0.035 × velocity) per tick, until |up.y| ≤ 0.001. Then the piece is done.
- This is the three-state routine that SPEC-0028 calls the launched-object after-flight fall. Contrary to SPEC-0028, it runs in play, for the 24 reed records in packs 909, 934 and 1056.
- **Reed state (High).** Each reed has a state word: 0 standing, 1 cut and falling, 2 gone. The logic step runs the bend and the weapon test in state 0 and the fall in state 1; it does nothing in state 2. The cut sets state 1 and the fall's own sub-state 0. When the fall's last sub-state ends, the reed's state becomes 2.
- **Gone reeds are not drawn and do not grow back (High).** A reed in state 2 fails its visibility test, so the draw skips it. Nothing sets a reed back to state 0 except the plant reset, which runs only when the patch is built again (the next load of the world). Bushes regrow (section 6); reeds do not.
- **Reed drop (High).** On each cut, one roll r is a uniform integer from 0 to 100, both ends included. r = 0 gives item 138 (25 NP), r from 1 to 5 gives 136 (5 NP), r from 6 to 19 gives 135 (1 NP), and r from 20 to 100 gives nothing. This is the same rule as the default bush drop (section 6), and luck plays no part. The drop spawns at the reed's position through the item spawn route of SPEC-0021.

### 10. Rustle and clover beds, C, and flowers, S

- **C (High).** These follow SPEC-0021 Clover patches. A rustle effect plays each tick while the hero is inside a plant's footprint. Beds of models 239 and 268 also roll the clover find (items 19 and 20) while the item's once-per-area bit is clear.
- **S (Low).** Each plant has 8 parts. A plant gets a touched flag when the hero is within about 2.5 units at a similar height. Its visible effect is open.

## Edge cases and error handling

- **Wrong slot in the once-per-area tests (quirk, reproduce).** The build gate, the Negg sparkle and the Negg drop all test the current world's area slot. A patch built or stepped in the second world (streamed beside the current one) is gated by the current world's bit, not its own.
- **Logic only near the camera.** The bush countdown, the regrowth, the sparkle, the tree trunks and the reed bend all run only while the camera eye is within 100 units of the patch centre and the game-state code is below 9. They pause when the camera is far away and under pause (code 9).
- **Hit gate while regrowing.** A regrowing bush with counter 450 at the start of a step ignores weapon hits and shots; with 451 it can be cut. A cut bush (state 0) cannot be cut again.
- **Key-5 bushes and shots.** Bushes of key 5 (model 198) are never cut by Roberta's shots.
- **Several drops in one step.** See section 6, Cut.
- **Inert bush patches.** A bush patch whose main model is not in the pack is built with no models and an inner distance of 0, so it never runs logic. On disc there are 76 such patches (66 of key 19, 2 of key 22, 1 of key 28 and 7 of key 32).
- **Keys without a family.** Key 15 and keys of 40 or more build nothing. Neither occurs on disc.
- **Far model and the common override set.** With (200, 32, 50, 150) the far band starts at S − O = 200, beyond F = 150. A two-model patch with this set would therefore never show its far model, and it would be cut off at 150. On disc no two-model patch uses this set: all 80 records with it are bush patches (key 20: 52, key 21: 16, key 31: 6, key 28: 4, key 22: 2, in 18 packs), which have no far object. Those bushes use the near-only rule: full size to 100, fading out from 100 to 150 (High). With (200, 32, 75, 250) the far model fades in from 200 to 232 and the near model fades out from 232 to 307, but the patch is cut off at 250. Reproduce from the rules in section 3.
- **Never-read plant words.** The plant words +0x04 (3.0) and +0x0C (173.48) are not read.
- **Over ten type-16 chunks.** The extra chunks are dropped (SPEC-0006). This does not happen on disc.

## Test vectors

Disc values:

- Pack 678, record 0: box (−1015.503, −1012.813, 1320.539, 1325.009), 4 rows, key 9 (bush, models 196/192/191), height 74.1661, zone 0, jitter flag 0, no overrides.
  - Centre (−1014.158, 74.166, 1322.774).
  - Patch radius 0.7072 × 4.470 = 3.161.
  - Row 0 holds 5 plants at z = 1320.539 + 2.073 = 1322.612. Its plant 1 lies at x = −1015.503 + 1.2959 = −1014.207, with no jitter.
  - The plant colour word is 0x80404040 (R 64, G 64, B 64, A 128).
- Pack 713, record 0: key 5, zone 1, height 118.589.
- Pack 909, record 63: key 8 (reeds), box (−1833.791, −1806.153, 958.214, 979.709), 15 rows, height 57.6007, zone 1.
  - Patch radius 0.7072 × 27.638 = 19.546.
  - Row 0 lies at z = 964.972. Its plant 0 lies at x = −1829.364 (J = 0 for reeds).

Synthetic:

- Jitter: a B record with flag 1, depth z1 − z0 = 10 and 4 rows gives J = 1.0. An S record with the same values gives J = 0.675. A G record gives J = 0.9 whatever the flag.
- Fade bands with overrides (50, 16, 10, 100), so F = 100, S = 66, O = 16 and W = 10:
  - A near-only patch at d = 95 has a = 0.5.
  - A two-model patch at d = 58 has a = 1 and b = 0.5. At d = 71 it has a = 0.5 and b = 1. At d = 80 it is far only.
- Gate: a patch at distance 99 from the eye with F = 150 runs its logic. At distance 101 it is drawn but its logic does not run. With the game-state code 9 its logic does not run.
- Grass alpha at d = 1.0 is (0.75 / 1.75) = 0.4286 of the plant alpha. At d = 0.2 it is 0.
- Regrow, K = 0.8, state 1, counter 0: after 1 step the scale is 0.8 / 600 (about 0.00133); after 300 steps it is about 0.4; after 600 steps it is about 0.8, with the state still 1. On step 601 the state becomes 2 and the scale is 0.8.
- Cut on step t, K = 1.0:
  - Step t: scale 1.0, cut model drawn.
  - Step t + 1: 3799/3800 (about 0.99974).
  - Step t + 3799: 1/3800 (about 0.000263).
  - Step t + 3800: the countdown reaches 0, the state becomes 1 and the counter 1, so the main model is drawn at 1/600.
  - Step t + 4399: the counter is 600 (scale 1.0).
  - Step t + 4400: the state is 2.
- Shot test with the disc words: a shot of radius 0.5 whose centre is 1.99 from the bush base cuts the bush. At 2.0 it does not, because the test is strict.
- Shots against key 5: a key-5 bush in range of one of Roberta's shots, with no weapon contact, is not cut.
- Several triggers: a weapon hit and two shots in range on the same step give three drop rolls and six debris pieces.
- Default bush drops: r = 0 gives item 138 (25 NP); r = 3 gives 136 (5 NP); r = 19 gives 135 (1 NP); r = 20 gives nothing. The chances are 1, 5 and 14 in 101.
- Coin bush:
  - Luck 0, r = 61: v = 30, four 1 NP coins (4 NP).
  - Luck 0, r = 62: v = 31, four 5 NP coins (20 NP).
  - Luck 0, r = 100: v = 50, 20 NP.
  - Luck 21, r = 0: v = 63, four 25 NP coins (100 NP).
  - Luck 24: v ≥ 72 for every r, so the drop is always 100 NP.
- Sparkle:
  - Coin bush grown, wait 0: effect 139 starts on this tick. The wait becomes w (18 to 42), and the next effect is on tick + w + 1.
  - Red Negg bush (model 271) grown, wait 0, bit of item 174 clear for the current world: the effect starts. With the bit set: no effect, and a cut drops nothing.
- Once-per-area build: a key-27 patch in an area whose slot bit in the item-19 set is set is not built. A key-36 patch built while another world is current is gated by that other world's bit.
- Reed fall: velocity (1.5, 0, 0), anchor at the start point. The piece moves 0.075 per tick, so state 0 ends on the 7th tick (0.525 > 0.5; 0.45 after 6 ticks). In state 1, with ground probe result g, the piece sinks 0.07 per tick until y < g − 1.25.
- Reed drop: r = 0 gives one 25 NP coin; r = 5 gives a 5 NP coin; r = 6 and r = 19 give a 1 NP coin; r = 20 gives nothing. A reed cut and fallen in pack 909 is not drawn again until pack 909 is loaded again.
- Bush with the override set (200, 32, 50, 150), key 20: at d = 90 a = 1; at d = 125 a = 0.5; at d = 149 a = 0.02; at d = 150 the patch is off (gate |C − E|² < F² fails). b is 0 at every distance.
- Key-5 bush, hero 6 units away: the up axis does not change on that step, whatever its tilt. Hero 4 units away with no body contact and up axis (0, 0.99, 0.141) (not unit length): the new axis is normalize(0.02 × (0, 1, 0) + 0.98 × (0, 0.99, 0.141)) = normalize(0, 0.9902, 0.1382), about (0, 0.9904, 0.1382).
- Bush bend and speed: hero 3 units from a key-5 bush, body contact, speed 0.05 units/s: T is the current up axis (no lean). Same with speed 2.0 and the hero facing (1, 0, 0): T = 0.76 × (0, 1, 0) + 0.24 × (1, 0, 0) = (0.24, 0.76, 0); T.y < 0.994, so with the bush upright the new axis is normalize(0.02 × (0, 1, 0) + 0.98 × (0.24, 0.76, 0)) = normalize(0.2352, 0.7648, 0), about (0.2940, 0.9558, 0).
- Full far batch: 1,200 grass plants with b > 0 in view on one frame, all of one model: entries 1 to 1,000 are drawn and the last 200 are not; on the next frame the batch starts empty again. 750 far plants of other families: 700 drawn, 50 not.
- Group limit: far plants of 21 different far models in the 700-entry batch on one frame, 10 of each, added model by model: the 200 plants of the first 20 models are drawn; the 10 of the 21st model are not.
- Constructor defaults are replaced: a grass patch with overrides (50, 16, 10, 100) has F = 100, not 500, and its inner distance is 100.

## Open questions

1. What the third models of the B, H and K families show, and what the S family's 8 parts and touched flag do.
2. The exact hero bend formula for bushes and reeds. For weapon contact, the plant-side capsule is established as described in Behavior 6; the exact construction of the hero weapon segment from its pose remains open.
3. Resolved: with b > 0 the plant is not drawn with a model of its own call; it is added to one of two shared far batches (1,000 entries for grass, 700 for the other families), emptied at the start of every frame and drawn once per frame (Behavior 4, Draw). When a batch is full (its room, or a 21st group of far model and material) the add is refused and that plant is not drawn on that frame, with no fallback (Behavior 4, Full batch). Still open: the render path of a batch (SPEC-0046). (Medium-High)
4. Resolved: yes. The eye is the position of the single camera object, which every camera mode moves (Behavior 4). (Medium-High)
5. Resolved: a cut reed does not grow back. When its fall ends it is gone (state 2) and is not drawn; it returns only when the world is loaded again. Each cut also rolls the default coin drop (Behavior 9). (High)
6. Resolved: no spring-back at all beyond 5 units; within 5 units the axis springs back by 2% per step, toward a blend with the hero's facing on body contact (Behavior 6, Helpers). The value compared with 0.1 is the hero's movement speed in units per second, and U is the constant world up (0, 1, 0). (High for the 5-unit rule and U; Medium-High for the speed; Medium for the blend)
7. Resolved: no. The set is used only by 80 bush patches, which have no far object; they fade out from 100 to 150 (Edge cases). (High)
8. Resolved: the +0x3C word of the record (0xFFFFFFFF in packs 678 and 713) is never read by the game (Layout, Patch record). (Medium-High)
