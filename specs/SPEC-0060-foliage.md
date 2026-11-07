# SPEC-0060: Foliage patches (type 16)

- Answers: proactive (open close-out); SPEC-0006 open questions on the type-16 fields, build selector, thresholds, query point, items and purpose; SPEC-0028 open questions 57 to 59 (six-slot look-ahead) and 80 (type-16 field reading); proactive otherwise (bush cutting, drops, regrowth, trees and reeds); proactive (reed angles); R-0046 (in part; the LOD fields and the trunk model open in Open questions); R-0072 (the plant colour word in the draw); R-0129 (reed bend amount: contact shape, other bend sources, worked walk-past); proactive (open question sweep, round 6); proactive (open question sweep, round 9); proactive (open question sweep, round 12)
- Status: Draft
- Disc build: SCUS-97367 v1.00 (same fingerprint as SPEC-0001). Check the three data words in Where it lives before trusting the values.
- Confidence: per section. Record layout and field roles: High (counted on all 75 chunks). Build dispatch, families and model ids: High. Once-per-area gates and the slot quirk: High. Placement: Medium-High (jitter and scale rules High; the per-model ground probe Medium). Per-frame gates: High for the order, Medium-High for the source of the eye point. Fade bands: High. Grass logic: Medium-High. Bush rules (state machine, cut triggers, drops, regrowth, sparkle): High; hero bend formula Medium; plant-side melee capsule geometry High. Trees: Medium-High. Reeds: Medium-High (fall states High). Flowers: Low.
- Related specs: SPEC-0006 (Box records, Binary grids and the zone query, Area-pair test), SPEC-0028 (section 16 and the launched-object after-flight fall, both superseded here), SPEC-0021 (item ids, coin values, Clover patches, Stat-up items, Spawn routes, item model registry), SPEC-0025 (once-per-area sets in the save), SPEC-0036 (attack window, hero shot slots, luck), SPEC-0018 (ground probes, ceilings, cylinder contact), SPEC-0004 (type-8 model containers), SPEC-0034 (effect group 3, effect 139), SPEC-0046 (draw), SPEC-0027 (camera eye, near-tree list), SPEC-0026 (main loop draw step; its old "timed transition driver" is the bush step of this spec), SPEC-0007 (60 ticks per second).

## Summary

Every type-16 record of a world pack is one **foliage patch**: a box of plant instances laid out in rows. The record's key (0 to 39) picks one of six families, each with fixed model ids from the item model registry:

- **G grass** (keys 0 and 12): swaying grass, drawn only as camera-facing sprites in the far batch (Behavior 4, Plant colour in the draw). A close-range alpha fade exists in the grass logic but has no visible effect (section 5).
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
| +0x08 | 4 bytes | Colour of the instance: R, G, B, A in byte order, in GS vertex units where 128 = 1.0 (High for the channel order and units). Example: the bytes 40 40 40 80 (hex), as a little-endian word 0x80404040, give R 64, G 64, B 64, A 128. Only the far sprite batch reads it, and it reads all four bytes unchanged; the near model draw ignores the word. See Behavior 4, Plant colour in the draw. |
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
| 4, 13, 25 | H trees | 4: near 200 only, with a solid cylinder; 25: near 200, far 199; 13: far 220 | always (the near model is absent or belongs to this world) | 4: 226 (15 packs); 25: 458 (18 packs); 13: 8 (pack 721) |
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

The save words that hold these sets are at slot offsets 0x2EB2 (item 19), 0x2ECB (item 20), 0x2EE4 (item 151), 0x2EFD (item 174) and 0x2F16 (item 154); each is a 193-bit set whose bit is the world index (SPEC-0025, once-per-area sets; R-0046).

A clover or Negg patch whose item was gained in the area is therefore missing from the next load of the area. Quirk (reproduce): the gate tests the slot of the world that is current at build time, not the slot of the world being built (Edge cases).

### 2. Placement (Medium-High)

1. Centre C = ((x0 + x1)/2, y, (z0 + z1)/2), with y = +0x1C.
2. Patch radius = 0.7072 × max(x1 − x0, z1 − z0). The bounding radius used by the draw is the patch radius plus 0.5 × the near model's bounding size.
3. Plant k of row r starts at (x0 + plant offset, y, z0 + row offset).
4. Jitter: x and z each get a uniform value in [−J, J]. J = 0.9 for G and 0 for K. For B, C and H, J follows the +0x24 rule (0.4 × depth / rows). For S it is 0.27 × depth / rows. J is 0 when the flag is 0.
5. Ground snap (families and keys that snap; G key 12 does not): y becomes the ground height under (x, z) from the column probe of SPEC-0018. Bushes of models 198 (key 5) and 223 (key 17) probe from y + 8 instead (SPEC-0018, Ceilings supplied by callers; Medium). A plant whose ground is −1.0e8 or below (no ground) is not placed.
6. Each plant gets a random yaw (a random horizontal facing), built from two draws of the game's shared random generator, each uniform in [−1, 1] (the generator is seeded from the console clock at first use, so a real run cannot be reproduced). Draw order per placed plant: first the facing's x draw, then its z draw, then the scale draw of step 7; reeds make no jitter draws (J = 0). A plant with no ground makes none of the three. The pair (x, z) is used as an unnormalised horizontal vector, so the facing is the direction of a random point in a square, not a uniform angle: the facing angle is atan2(x, z), measured from +z toward +x, and its density is proportional to 1 / max(|sin|, |cos|)² (diagonal facings are twice as likely per degree as axis facings; 58.6 percent of plants face within 22.5 degrees of a diagonal). For reeds the matrix rows are unit length: forward = (x, 0, z) / |(x, 0, z)|, up = (0, 1, 0) (world up, never the ground normal), right = (z, 0, −x) / |(x, 0, z)|. The plant scale K of step 7 is not stored in the rows; it is multiplied into all three rows at draw time (Reed angles, build guide). Confidence High.
7. Each plant gets a uniform scale K in [s0, 1]. s0 is 1.0 for G, 0.6 for S and B, 0.85 for C and H, and 0.73 for K. Bushes of model 265 (key 34, not on disc) use [1.0, 3.5].
8. The plant keeps its four colour bytes (R, G, B, A) as stored. The alpha byte is kept a second time on the plant instance. Both are used only by the far sprite batch (Behavior 4, Plant colour in the draw). Disc count: 21,012 of the 29,853 plants have the bytes 64, 64, 64, 128; the alpha byte differs from 128 only in the grass, tree and reed keys that have a far sprite (5,844 plants).
9. A new bush starts in state 2 (grown) with counter 0, cut flag 0, countdown 0 and bend latch 0. Its effect wait is trunc((u + 0.5) × 60), with u uniform in [−0.2, 0.2], which gives 18 to 42 ticks.

### 3. Distances and fade bands (High)

With overrides (A or D nonzero), the four distances are:

| Distance | Value |
| --- | --- |
| Far distance F | D |
| Switch distance S | B + A |
| Out band O | B |
| Fade band W | C |

Without overrides they come from the near model's LOD node (SPEC-0004), as follows (High for the field reads, Medium for the clamp): the far distance F is the node's fade end (float at +0x64); the switch distance S is the node's float at +0x54; the fade band W is the node's fade length (float at +0x58); the out band O is S minus the float at +0x50. If the patch's build record has a zero flag word at +0xA4, W and O are both raised to the larger of the fade length and O, and S is lowered to F minus that value when that difference is below S. The cull value, the layer flags and the pass byte are not used for these four. With no model, F = 4000. Test vector: a node with S = 80, +0x50 = 30, fade length 12 and fade end 150 and flag word zero gives O = 50, W = 50, F = 150 and S = 80 (F minus 50 is 100, not below 80).

**Per-patch distances (High).** Every family build passes an inner (logic) distance of 100 for every patch, and 0 for an inert bush patch; no record field changes it. The patch constructor first sets defaults (inner 100, far 500, switch 100, out band 30, fade band 30), but the patch set-up of every built patch replaces them with the values above (overrides, or the LOD node, or F = 4000). So the defaults are never seen in play. Without overrides and without a far object, the LOD values are adjusted (Medium): O becomes max(O, W), W becomes that same value, and S is lowered to F − O when it is larger.

**Which patches have a far object (High).** A patch has a near model (the family's main model) and, for the families that pass one, a separate far object. Bush patches never have a far object, so every bush patch uses the near-only weights. Two-model weights apply only to the grass, tree and reed families that pass a far model (table in Behavior 1).

Weights for a plant at distance d from the eye (near weight a, far weight b):

- Only a near model: b = 0. a = 1 up to F − W, then a = 1 − (d − (F − W)) / W, clamped at 0.
- Only a far model: a = 0. b = 1 up to F, then b = 1 - (d - F) / W, clamped at 0. The patch gate keeps the patch centre inside F, so the fade shows only on plants that lie beyond F because of the patch radius (High for the shape; Medium-High for which input is which).
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
   - **Far entry.** On the add, the far entry's alpha byte is set to trunc(A × b), with A the alpha byte of the plant's colour word (High: the plant instance keeps a copy of that byte, and the add reads the copy). The entry's red, green and blue bytes are the plant's R, G and B bytes, unchanged (next subsection).
   - **Plant colour in the draw (High unless marked; answers R-0072).** The colour word reaches the screen on one path only, the far sprite batch; neither the near model draw nor any draw-context colour uses it.
     - **Near draw.** A plant drawn with its near model (a > 0) uses the fixed default draw colour k = (0.5, 0.5, 0.5) with its fourth lane set to the near weight a. By the rule of SPEC-0046 (Fade level quantizer and draw colour scale) this gives the colour scale s = (1, 1, 1, a') with a' from the opacity table for a. The plant's R, G, B and A bytes are not read: every patch and every plant of a patch is drawn with the same scale, whatever its colour word. A bush, flower bed, clover bed, reed or tree drawn with its near model therefore shows full model brightness, and its opacity is the near weight alone. So the "k = byte / 128" reading (64 giving the neutral 0.5) matches what the near draw does in effect, but the bytes are never converted: the 0.5 is the fixed default.
     - **Far batch.** Each far entry is one camera-facing textured quad, the same record and the same packet builder as the particle sprites of SPEC-0034 section 16: the entry's first 16 bytes hold R, G, B and A as four 32-bit words, of which the low byte of each is used, and the builder copies them without arithmetic into the quad's vertex colour. The four plant bytes are therefore GS vertex colour in GS units, the same units as every other sprite colour in the game (SPEC-0034: 128 = 1.0). Neither k nor the colour scale s exists on this path: the batch is queued as a ready-made packet, and the colour scale applies only to meshes that go through the vertex programs.
     - **Where the 0.5 versus 1.0 comes from.** The PS2 GS applies the texture function MODULATE as out = texel × vertex colour / 128 (clamped to 255). A vertex colour of 128 therefore leaves the texel unchanged (factor 1.0), 64 halves it (factor 0.5) and 255 gives about 2.0. The 0.5 neutral of the draw colour k and the factor 2 in s = 2k (SPEC-0046) are the float-side convention for the vertex programs, which write a colour byte for the GS; the far sprites skip that layer and hand the GS the bytes themselves. So for a far sprite the on-screen colour factor is byte / 128 per channel, and 64 grey is half brightness (High: the sprite material's texture function is MODULATE, see Sprite material state).
     - **Sprite material state (High).** Every far sprite is drawn with a material group of the far model's own shader list: the group named by the first mesh leaf of the far model (a single quad of 4 vertices). The load-time billboard binding (SPEC-0003) forces that group onto the billboard program. The sprite batch packet selects drawing context 2, so the group's second-set registers are the ones in force (TEX0, ALPHA and TEST of context 2; a one-context group stores them under those register numbers). On the disc all 28 far models agree:
       - The grass sprite model (id 283, from the shared boot pack) uses group 19 of that pack's shader list. The tree far models (ids 199 and 220) and the reed far model (id 203) use the group of their own world pack's list; 23 packs carry model 199, 3 carry 203 and one carries 220. They differ from the grass only in texture, size and wrap.
       - TEX0: TCC = 1 (texture alpha used) and TFX = 0, which is MODULATE. The other three texture functions (DECAL, HIGHLIGHT, HIGHLIGHT2) are not used by any far sprite. The grass texture is 8-bit indexed, 128 wide and 64 high (TW 7, TH 6).
       - Blend: the batch's primitive enables alpha blending, and ALPHA is the ordinary blend of the material: out = (Cs − Cd) × As / 128 + Cd, with As the fragment alpha (not a fixed factor).
       - Alpha test: enabled, method GEQUAL, reference 120, and the fail action is "write colour only" (the pixel is still blended and drawn but writes no depth). The depth test is enabled with GEQUAL. The test is applied to the fragment alpha after MODULATE, so it sees texel alpha × vertex A / 128. Only a sprite whose vertex alpha and texel alpha are both near 128 (product at least 120) writes depth; a sprite whose vertex alpha is below 120 (every grass sprite whose word has A of 64 or less, which is the common case, and every tree or reed sprite with b below about 0.94) is drawn without writing depth.
       - Wrap: clamp on both axes for the grass sprite and the tree sprites (the quad's texture coordinates run 0 to 1, so the clamp never matters); the reed sprite repeats. Sizes: grass 128 by 64, trees 128 by 128, reeds 64 by 128.
     - **Alpha.** On the far path the alpha byte is the sprite's opacity in GS units: 128 is 1.0, the value is trunc(A × b) after the far weight, and the GS multiplies it into the texel alpha (texture alpha is used) before the material's alpha test and blend (Sprite material state). The near draw does not read the alpha byte. The disc is consistent with this: the alpha byte differs from 128 in 5,844 plants, all of them in keys 0, 12, 13, 25 and 8 (the keys with a far sprite), and in none of the other 20,519 plants.
     - **Which plants use which path.** Grass (keys 0 and 12; 6,249 plants) has no near model and b = 1 at every distance up to F (beyond F it fades out over W, section 3), so the colour word is its only colour control: all grass is drawn with its word as GS vertex colour. Trees of keys 13 and 25 and reeds (key 8) show the near model close up (colour word ignored) and the sprite at range (colour word used, so a 64-grey sprite is half brightness while the near model is full brightness; Medium that no cross-fade correction exists, because nothing in the build or the draw compensates). All bush, flower and clover patches and key 4 trees have no far object, so their colour words (including the tinted words of key 16) have no effect at all.
     - **Clean-side rule.** Draw near models with the fixed default colour and opacity a; do not apply the colour word. For far sprites, multiply the texel by (R, G, B) / 128 and its alpha by trunc(A × b) / 128.
   - **Full batch (Medium-High).** A batch keeps a count of entries and a list of at most 20 groups; entries with the same far model, material and draw slot share a group. An add is refused when the count has reached the room (700 or 1,000), or when the entry needs a 21st group. A refused plant is simply not drawn on that frame: no error, no fallback to the near model, and the patch's other plants go on. Plants are added in patch order and row order, so the first entries of the frame are kept and the last ones are lost.

### 5. Grass, G (Medium-High)

- A plant within 20 units of the view point takes the patch's sway phase.
- A plant within 2.0 units has its alpha scaled. The factor is 0 within 0.25, and (d − 0.25) / 1.75 from 0.25 to 2.0. The scaled value goes into the plant's far entry, but the far add of Behavior 4 sets that entry's alpha again, to trunc(A × b) from the plant's own alpha byte, before the entry is queued, and nothing draws the entry otherwise. The scale therefore has no visible effect: grass does not fade near the camera (Medium-High). The clean side can leave it out.

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

**Spring-back.** A bush whose up axis is tilted, and which is not held by the hero bend, springs back each step: up = normalize(0.925 × up + 0.075 × (0, 1, 0)). Within 5 units the hero bends a bush away from himself or herself and sets the bend latch.

**Hero bend (every bush key except 5; Medium-High).** It runs each step after the spring-back, and only while the squared horizontal distance from the hero to the bush's raised test point (the bush base plus its up axis times the plant's height scale) is below 25 and the hero's body test against the bush passes (the segment test of radius 0.75 used for key 5). If the test passes and the bend latch is clear, the bush is bent (this repeats every step while the latch stays clear), and the latch is set only when the close case below applies (s at most 0.25). If the test passes and the latch is already set, nothing changes: the bush stays as it is, and it is not sprung back while the latch is set. The latch is cleared on every step in which the body test fails. (The reed uses the same structure; Reed angles, build guide.) The bend: let a be the horizontal unit vector from the hero's position to the bush base, and s the horizontal distance between the two minus the hero's body radius. The lean weight is w = 1 when s is at most 0.25, and otherwise w = 1 - 2 (s - 0.25). The target axis is T = normalize(w a + (1 - w) (0, 1, 0)). If the bush's current up axis has a smaller y than T (the bush is already leaning further than the target), the new up axis is normalize(0.075 T + 0.925 up); otherwise the new up axis is T at once. The frame is rebuilt around the new up axis. Test vector: hero radius 0.5, hero 1.0 unit from the bush base along +x (a = (1, 0, 0)), bush upright, latch clear: s = 0.5, w = 0.5, T = (0.7071, 0.7071, 0); the current up y (1.0) is not below T.y (0.7071), so the up axis becomes T at once and the latch stays clear (s is above 0.25). Hero 0.6 units away: s = 0.1, w = 1, T = (1, 0, 0), a bush lying flat away from the hero, and the latch is set.

**Cut triggers.**
- **Weapon.** Tor's or Roberta's weapon hits the bush inside the attack window (SPEC-0036). The weapon test compares the hero weapon segment with a bush capsule of radius 0.5 and length 6.0. The bush capsule is centered 3.0 units along the bush up axis from its base, so it spans the base to 6.0 units above it along that axis (High). The hero weapon segment is a capsule of radius 0.5 and full length L, centred on P = A + 0.25 D1 with axis D1, where A and D1 are the anchor and the Z row of attachment frame 0 of the hero's posed body this tick and L is the hero's swing length (the weapon reach plus 2.0, SPEC-0036, Hit detection); the weapon test runs only while the hero's attack window is active and the bush is not cut. The static capsule-against-capsule overlap runs first; if it misses, the swept capsule-against-triangle test of SPEC-0036 runs on the triangle (P, P + L D1, P + L D2), with D2 the same axis from the previous tick, with no separate tip-moved test (a weapon that did not move gives a triangle with no area). Reeds use the same bend rule, constants and weapon test (Medium-High). Test vector: weapon reach 1.5, so L = 3.5; the weapon axis along +x at anchor (0, 1, 0): P = (0.25, 1, 0) and the weapon segment runs from (-1.5, 1, 0) to (2.0, 1, 0). An upright bush with its base at (2.5, 0, 0) has its segment from (2.5, 0, 0) to (2.5, 6, 0): the axes are 0.5 apart, below the radii sum 1.0, so the bush is hit. With the base at (3.5, 0, 0) the axes are 1.5 apart and the static test misses. The hit cue is 20 for Tor and 28 for Roberta, played at the bush.
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

- **Bend.** A reed within 5 units (horizontally) of the hero is tested against the hero's body each step and leans away when touched; when it is free it springs back at 7.5 percent per step toward vertical. Exact rules, limits and numbers are in Reed angles (build guide) below.
- **What happens when the hero steps across one (static estimate).** Reeds do not have a weight or compression response, and the patch does not register a standable platform. They bend only when the hero-body contact test succeeds. For a close contact whose horizontal direction from hero to reed is nonzero, the shared bend rule chooses a horizontal target axis when `s = horizontal distance − hero radius` is at most 0.25; from upright, it applies that target in one step, rather than easing it down over time. Model 202's local upper stem reaches y = 5.693, and reed placement scale K is 0.73 to 1.0. A fully horizontal lean therefore removes about 5.69 × K = 4.16 to 5.69 units from the stem's vertical projection. Side branches and the rebuilt side axes can leave other vertices above or below this estimate. With the hero exactly over the base (zero horizontal offset) the shipped game destroys the frame (Medium): the horizontal direction normalises to the zero vector, the bend latch is set, the target axis is the zero vector, and the reed's up axis and the two other axes all become zero vectors. The spring-back step cannot rebuild them, because its cross products start from a zero axis, so the reed stays collapsed after the hero moves away. The side-branch displacement is therefore moot (Reed angles, build guide). Reproduce this: when the horizontal offset is exactly zero (both coordinates equal), zero the reed's three axes and set the latch. The case needs the hero's position to equal the reed base in both horizontal coordinates, so it is very rare in play.
- **Cut.** A hit by the hero's weapon in the attack window cuts the reed. The cue is 22 (Tor) or 28 (Roberta). The cut piece's initial direction comes from the change between two sampled weapon-pose points, each offset 2.5 units along its sampled frame axis. The vertical component of that change is discarded and the horizontal remainder is normalized, so the launch velocity is a horizontal unit vector (y = 0); the displacement step in the first fall state is therefore 0.05 units per tick. This is a motion-derived knock-away direction, not the weapon's full swing speed. A hit while the hero's body already holds the reed flat (bend latch set) cannot cut it (Reed angles, build guide).
- **Fall of the cut piece**, in three states:
  1. State 0: position += 0.05 × velocity per tick, until the piece is more than 0.5 from its anchor.
  2. State 1: y −= 0.07 per tick, until y is below (the ground probe result with ceiling y + 8.0) − 1.25 (SPEC-0018, Ceilings supplied by callers).
  3. State 2: if the up axis's y component has absolute value above 0.001, up becomes normalize(0.965 × up + 0.035 × velocity) and the side axes are rebuilt (the stalk topples toward the launch direction at 3.5 percent per step). Otherwise the reed becomes gone. The launch velocity is horizontal, so up.y shrinks toward 0 and the reed is gone after about 213 rotating steps (table in Reed angles, build guide).
- This is the three-state routine that SPEC-0028 calls the launched-object after-flight fall. Contrary to SPEC-0028, it runs in play, for the 24 reed records in packs 909, 934 and 1056.
- **Placements and a coincident duplicate (High for disc data).** The 24 patches contain 491 plant entries: pack 909 has 14 patches/298 plants, pack 934 has 8/101, and pack 1056 has 2/92. Both pack-1056 patches have the same box, base height, nine row counts and offsets, and all 46 plant records match; they are stored in separate type-16 chunks. Model 202 is present in all three packs, so both pack-1056 records pass the reed build's model-presence gate. They therefore describe two built patch instances at the same 46 plant positions. Each patch steps its own plants and every plant tests the selected hero and the weapon on its own, with no shared state between patches (Medium-High), so the two overlapping reeds at each of the 46 positions bend identically (the same tilt, so nothing differs on screen) and are cut by the same swing: each rolls its own drop (two chances at the Reed drop rule per position, so about twice the drop rate there) and throws its own cut piece in the same direction, which makes the two pieces overlap. Preserve both instances.
- **Reed state (High).** Each reed has a state word: 0 standing, 1 cut and falling, 2 gone. The logic step runs the bend and the weapon test in state 0 and the fall in state 1; it does nothing in state 2. The cut sets state 1 and the fall's own sub-state 0. Only a state-2 fall update that finds |up.y| ≤ 0.001 sets the reed to gone. Until then the cut piece remains in state 1 and is still drawn.
- **Gone reeds are not drawn and do not grow back (High).** A reed in state 2 fails its visibility test, so the draw skips it. Nothing sets a reed back to state 0 except the plant reset, which runs only when the patch is built again (the next load of the world). Bushes regrow (section 6); reeds do not.
- **Reed drop (High).** On each cut, one roll r is a uniform integer from 0 to 100, both ends included. r = 0 gives item 138 (25 NP), r from 1 to 5 gives 136 (5 NP), r from 6 to 19 gives 135 (1 NP), and r from 20 to 100 gives nothing. This is the same rule as the default bush drop (section 6), and luck plays no part. The drop spawns at the reed's position through the item spawn route of SPEC-0021.

### Reed angles (build guide)

Confidence: High for the frame conventions, yaw draws, spring-back, bend formula, latch, cut launch direction and the fall; High for the hero body contact on level ground (Medium-High for the end-cap cases), Medium for which of the two weapon-frame banks is older, and for the far sprite anchor. Everything here is for key 8 (models 202 near, 203 far). Each reed is one plant with its own frame; there is no wind, sway or idle animation anywhere in the reed logic (High). The only angle changes are the hero bend, the spring-back and the cut fall below.

**Frame and draw matrix (High).**
- Each plant has a frame of four rows: right (row 0), up (row 1), forward (row 2) and the position (row 3, w = 1). Model coordinates map as x to right, y to up, z to forward. The frame is a proper rotation (right × up = forward, no mirroring). All three rows are unit length while stored; the plant's uniform scale K (0.73 to 1.0, one draw) is held separately as a vector (K, K, K) and the draw multiplies the rows by it, so the drawn model is the near model scaled uniformly by K about its base point, then rotated by the frame, then translated to the base point. The base point is the ground height at the placed x and z (SPEC-0018 probe), and the plant keeps that point as its anchor; bending never moves it.
- Initial frame: up = (0, 1, 0) for every reed (world up, not the ground normal, so reeds stand vertically on slopes), forward = (x, 0, z) / |(x, 0, z)| and right = (z, 0, −x) / |(x, 0, z)| for the two yaw draws x and z of Behavior 2 step 6. Yaw angle φ = atan2(x, z): forward = (sin φ, 0, cos φ), right = (cos φ, 0, −sin φ), a right-handed rotation about +y by φ from +z toward +x. The two draws are uniform in [−1, 1] and not normalised first, so φ is not uniform (diagonals favoured).
- Frame rebuild after any change of the up axis (used by the bend, the spring-back and the fall). With the new up axis U (unit): forward = normalise(right × U), then right = U × forward. Equivalently, the stored right row is projected onto the plane perpendicular to U and renormalised, and forward = right × U. The stored right row is the only memory of the yaw (and of earlier twists): a tilt keeps the right row as close as possible to where it was, so the stalk does not roll about its own axis except as this projection dictates. If the right row is parallel to U the cross product is zero and the rebuild has no defined result (it needs a guard; Default if unresolved (rewrite choice): keep the previous frame when |right × U| is below 1e-6).
- Far sprites (Medium-High): the far sprite of a reed is a screen-aligned quad with spin angle 0 (never rotated), centred on the base point at the moment the reed is built, sized by K. It uses neither the yaw nor the tilt, and it does not follow a bent or falling reed. A reed in state 1 is still added to the far batch (at its original base point) until it is gone.

**Per-step order for a standing reed (state 0; High).** Reed logic runs only for patches whose centre is within 100 units of the camera eye and with game-state code below 9 (Per frame). In that case each standing reed runs, in this order, once per tick:
1. Spring-back (no distance gate; it runs everywhere the patch logic runs).
2. Body contact and bend (distance gate below).
3. Weapon test, only inside the same distance gate (below).
A reed in state 1 runs the cut fall only. State 2 does nothing.

**Spring-back (High).** If the bend latch is 0 and up.y < 0.999: up = normalise(0.925 × up + 0.075 × (0, 1, 0)), then the frame rebuild. Nothing else restores the reed. Tilt in degrees from vertical, starting flat (90 degrees): after 1 step 85.4, 2: 80.8, 3: 76.3, 5: 67.5, 10: 48.4, 15: 33.8, 20: 23.2, 30: 10.7, 40: 4.9; the step that first gives up.y ≥ 0.999 (tilt 2.56 degrees or less) is step 49. From 45 degrees it takes 38 steps; from 30 degrees 32 steps. So a flattened reed takes about 0.8 s to stand. A reed whose patch logic is paused (camera more than 100 units away) keeps its tilt until the camera returns.

**Body contact (High for the constants and the lateral test; Medium-High for the end-cap cases).** A standing reed is tested only when the horizontal distance d between the hero's position and the reed base is below 5 (squared distance below 25; the height is ignored for this gate). Inside it, the contact is a single sphere of radius 0.75 centred 1.4 units straight up (world up, not the reed's up axis) from the base, against the hero's body: a vertical segment that starts at the hero's position and runs up by the body height H = 3.5, with the body radius R = 1.175 (the weight 5.5, radius 1.175 and height 3.5 are fixed program constants for both heroes; nothing changes them at run time). Contact when the sphere centre lies within the segment's height span and its distance from the segment axis is below R + 0.75; at equal ground height that is d < R + 0.75 = 1.925. The reed has exactly this one sphere: no second sphere higher on the stalk, no larger radius, no term for the hero's speed, facing or action, and no dependence on the reed's scale K or its tilt (the 1.4 and 0.75 are used as stored, K only scales the drawn model). With the hero on the same ground as the reed the sphere centre is 1.4 above the hero's feet, inside the 3.5 span, so only the lateral rule applies. If the centre lies above or below the span (a hero on a ledge or in a jump: the reed base more than 2.1 above the hero's feet, or more than 1.4 below them) the contact is the exact touch test between the sphere and a cylinder with flat ends (Medium-High). Let g be how far the sphere centre is beyond the nearer end of the span, measured vertically, and l its horizontal distance from the body axis. If l is at most R the contact holds when g < 0.75 (the flat end face). If l is above R it holds when the distance to the end rim, the square root of (l - R) squared plus g squared, is below 0.75. The ends are flat, not rounded, so a capsule of radius R + 0.75 around the segment is wrong: it over-counts near the rim. Test vector: hero feet at height 0, reed base at height 2.7 (centre at 4.1, g = 0.6): contact at l = 0 and at l = 1.5 (rim distance 0.68), no contact at l = 1.7 (rim distance 0.80; the capsule would wrongly report contact there). With the base at 3.0 (g = 0.9) there is no contact at l = 0. The bend that follows a contact still uses only the horizontal distance d, exactly as below.

**Bend (High for the formula and the latch; High for the contact shape on level ground).** On a step where the contact test passes:
- If the latch is already 1: nothing happens (no bend, no spring-back this step).
- Otherwise let a = the horizontal unit vector from the hero's position to the reed base (the height is dropped before normalising) and s = d − R. The weight is w = 1 when s ≤ 0.25, else w = 1 − 2 (s − 0.25) (w falls from 1 at s = 0.25 to 0 at s = 0.75, which is the edge of contact, so w stays between 0 and 1). The target axis is T = normalise(w a + (1 − w)(0, 1, 0)); its tilt from vertical is atan(w / (1 − w)), i.e. 90 degrees at w = 1, 45 degrees at w = 0.5 and 0 at w = 0.
- If s ≤ 0.25 the latch is set to 1 (this is the only place it is set).
- If the reed's current up.y is smaller than T.y (it is already leaning further than the target): up = normalise(0.075 T + 0.925 up). Otherwise up = T at once (no easing; a reed goes from upright to the full target lean in one step). Then the frame rebuild.
- The reed has no speed threshold and no facing term: a stationary hero touching it bends it exactly as a moving one, always directly away from the hero.
The latch is cleared on every step in which contact does not hold (including when the hero is beyond 5 units). While it is set the reed stays flat at the full 90 degrees (up = T) and also cannot spring back. With the hero's R = 1.175 the lean starts at d = 1.925 (upright), reaches 45 degrees at d = 1.675, and is flat and latched at d ≤ 1.425. The lean is applied every step while 1.425 < d < 1.925, so a hero walking in or out drives the reed's angle directly from d; walking out, the blend limits the reed's return to 7.5 percent toward T per step on top of the spring-back's 7.5 percent toward vertical (spring-back runs first in the same step).
Degenerate case: if the hero stands exactly on the reed base horizontally, a is the zero vector and the frame is destroyed; the shipped game does not guard it. A port should keep the previous frame (Default if unresolved (rewrite choice)).

**Who can bend a reed, and how wide the flattened strip is (High).** The reed step is handed one object only: the currently selected hero (Tor or Roberta, whichever the player controls). The partner hero, enemies, petpets, and projectiles have no path into a reed: run and roll bend nothing by themselves, and nothing bends a reed except (1) the selected hero's body contact above, (2) the spring-back, and (3) the cut fall after the hero's weapon hits it. Reeds, unlike bushes, are not tested against the hero's shots. The weapon test fires whenever the hero's attack-active word is set (the animation event that opens the hit window, SPEC-0036), whatever the move, and needs the latch clear. The strip a walking hero leaves behind is therefore fixed by R: every reed whose base passes within 1.425 of the hero's path is driven flat and latched, and stays flat until the hero is farther than 1.925 away, so the flat strip is 2.85 wide and the partly bent fringe on each side is only 0.5 wide. A reed in the fringe takes the tilt of its closest approach (from the formula: 3 degrees at d = 1.9, 45 degrees at 1.675) and eases back as the hero leaves: while the hero is still in contact and moving away, the reed follows the target down at 7.5 percent per step on top of the spring-back; after contact ends only the spring-back runs. The spring-back stops once up.y reaches 0.999, so a released reed rests between 0 and 2.56 degrees off vertical (about 2.5 degrees after a long decay) and is not reset to exactly upright. At a typical walking pace (about 0.1 unit per step) a hero needs about 10 steps to cross the 0.5-wide fringe. A port that shows reeds barely moving should check first that it uses R = 1.175 (not a smaller hit radius such as 0.5) and applies the full 3.5 height span and the lateral bound R + 0.75.

**Weapon cut test (High for the geometry rules; Medium for pose timing).** Inside the 5-unit gate, while the hero's attack window is active and the latch is 0 (so a reed the hero is pressing flat cannot be cut), the hero weapon capsule (Behavior 6, Weapon) is tested against a reed capsule of radius 0.5 and length 6.0 centred 3.0 units along the reed's current up axis from its base. The current tilt matters: a bent reed's capsule lies along its bent axis. A hit does not change the reed's angle on that tick; it starts the fall below on the next tick.

**Cut set-up (High).** State 1, fall sub-state 0. The anchor stays the base point. The launch velocity v is computed once: take the weapon-tip point of the hero's current weapon-frame bank (frame position plus 2.5 times the frame's Z row) minus the same quantity in the older bank (SPEC-0005, Weapon frame blocks; SPEC-0036, Tick order of the tests), set the y component to 0 and normalise: v is a horizontal unit vector along the horizontal motion of the weapon tip between the last two posed ticks, so reeds fall in the direction of the swing. The roll for the drop (Reed drop) and the cue (22 for Tor, 28 for Roberta, played at the base) happen on the cut tick. If the tip did not move horizontally, v is the zero vector and the piece never leaves sub-state 0 (it stays standing, drawn, and cannot be cut again).

**Cut fall (High).** Ticks after the cut tick (the cut tick itself does not move the reed):
1. Sub-state 0: base += 0.05 v each tick; when the distance from the anchor is strictly above 0.5 the sub-state becomes 1 (11 ticks; at coordinates near 1000 float rounding makes each step slightly under 0.05, so the 10th tick still reads 0.4999 and the 11th switches). The up axis and right row do not change; the reed leans exactly as it stood when cut (upright, or tilted if cut while bent).
2. Sub-state 1: base.y −= 0.07 per tick, then ground probe at the new position with ceiling y + 8; when y is strictly below ground − 1.25 the sub-state becomes 2. On flat ground starting at the ground height that is 18 ticks (17 give −1.19, 18 give −1.26). The reed slides straight down into the ground with its angle unchanged.
3. Sub-state 2: while |up.y| > 0.001, each tick up = normalise(0.965 up + 0.035 v) and the frame rebuild; when the test fails the reed becomes gone (state 2) and is no longer drawn. The stalk topples toward v about its (now buried) base: tilt from vertical after n steps, upright start: 1: 2.1 degrees, 2: 4.2, 5: 10.3, 10: 20.3, 20: 38.1, 30: 52.5, 40: 63.3, 60: 76.7, 80: 83.5, 100: 86.8, 150: 89.5, 213: 89.94 (up.y = 0.0010). Step 213 is the last rotation; step 214 finds up.y ≤ 0.001 and sets gone. From a start tilt other than upright the count differs (a stalk already lying along v ends at once).
Total: gone at tick 11 + 18 + 213 + 1 = 243 after the cut, about 4.05 seconds. While falling, the base is up to 1.25 below the ground and the stalk lies along v, so most of the stalk ends below or at the ground surface.

**What a port most often gets wrong.**
- Treating yaw as a uniform angle (it is the direction of a random point in a square), or as a rotation about the reed's own axis after tilting (the rebuild projects the right row instead; the twist about the stalk follows from it).
- Aligning reeds to the ground normal or giving them a random initial lean (up is exactly (0, 1, 0)).
- Easing the hero bend in: an upright reed jumps straight to the target lean in one step; only a reed already leaning more than the target eases (7.5 percent).
- Adding hero speed or facing to the reed bend (that belongs to the key-5 bush) or adding idle sway (none).
- Setting the latch on any contact. It is set only when s ≤ 0.25 (flat), and then blocks both further bending, spring-back and weapon cuts until contact ends.
- Giving the cut piece a vertical launch or a spin, or ending the fall at the ground: the launch is horizontal, there is no angular velocity, no gravity acceleration (constant 0.07 per tick sink), no bounce, and the final state is a stalk toppling over while buried 1.25.
- Rotating far sprites with the reed or letting them follow the bend.

### 10. Rustle and clover beds, C, and flowers, S

- **C (High).** These follow SPEC-0021 Clover patches. A rustle effect plays each tick while the hero is inside a plant's footprint. Beds of models 239 and 268 also roll the clover find (items 19 and 20) while the item's once-per-area bit is clear.
- **S flowers (Medium-High).** Each source plant has eight independently updated parts. Each part clears its touched flag, then tests the hero relative to its frame. A hero at least 2.0 units above the part fails contact; the other tests use a 2.5-unit projected reach and a 0.75-unit close-contact band. Qualifying contact sets the flag. In the close-contact band, the part tilts away from the hero and rebuilds its side axes, changing the rendered pose. With no contact, the part gradually returns toward world up and marks the return complete when the up-axis dot product reaches 0.99. The exact mapping of each part to the flower mesh remains unknown.

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
- Grass logic alpha at d = 1.0 is (0.75 / 1.75) = 0.4286 of the plant alpha. At d = 0.2 it is 0. Neither value reaches the screen (section 5); the drawn alpha is trunc(A × b).
- Plant colour, near draw: any plant of a bush, flower, clover, reed or tree patch drawn with its near model at weight a = 0.5 uses the colour scale (1, 1, 1, 0.504) (index 128 of the opacity table), for the bytes 64, 64, 64, 128 and equally for 96, 96, 96, 6. The bytes change nothing.
- Plant colour, far sprite, real plant: pack 717, record 0 (key 0, grass), row 0, plant 0, colour bytes 40 40 40 26 (hex), R 64, G 64, B 64, A 38. Grass has only the far sprite, so b = 1 and the far entry's alpha is trunc(38 × 1) = 38. The vertex colour as applied is (64, 64, 64, 38) in GS units. With MODULATE the on-screen factor is 64 / 128 = 0.5 on each colour channel of the texel and 38 / 128 = 0.297 on its alpha.
- Plant colour, far sprite, full pixel path (synthetic texel, real grass state): the grass plant above, a texel of RGBA (200, 160, 80, 128) and a frame-buffer pixel of (60, 100, 40). MODULATE gives colour (200 × 64 / 128, 160 × 64 / 128, 80 × 64 / 128) = (100, 80, 40) and alpha 128 × 38 / 128 = 38. The alpha test (at least 120) fails, so the pixel is still blended but writes no depth. Blend with As = 38: R = 60 + ((100 − 60) × 38 >> 7) = 60 + 11 = 71; G = 100 + ((80 − 100) × 38 >> 7) = 100 − 6 = 94; B = 40. The result is (71, 94, 40). With the word 128 128 128 128 and the same texel the colour is (200, 160, 80) at alpha 128, the test passes, depth is written and the pixel is (200, 160, 80). With the word 64 64 64 128 the colour is half of the texel (100, 80, 40) at alpha 128, and it writes depth.
- Plant colour, far sprite, fading tree: a key 25 plant with bytes 64, 64, 64, 128 and weights a = 0.5, b = 0.5: the far entry's alpha is trunc(128 × 0.5) = 64, so the sprite has colour factor 0.5 and alpha factor 0.5; the near model of the same plant is drawn at the same time with scale (1, 1, 1, 0.504).
- Plant colour, translucent grass: a key 12 plant with bytes 96, 96, 96, 6: colour factor 96 / 128 = 0.75, alpha factor 6 / 128 = 0.047 (b = 1).
- Plant colour, no far object: a key 9 bush with bytes 64, 64, 64, 128 is drawn at full model brightness at every distance up to its fade-out. A key 16 bush with the bytes 66, 66, 70, 128 is drawn the same way (its tint is unused).
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
- Reed fall, synthetic non-disc velocity: velocity (1.5, 0, 0), anchor at the start point. The piece moves 0.075 per tick, so state 0 ends on the 7th tick (0.525 > 0.5; 0.45 after 6 ticks). In state 1, with ground probe result g, the piece sinks 0.07 per tick until y < g − 1.25.
- Reed fall, cut-launch velocity: any nonzero sampled weapon-pose displacement normalizes to length 1.0. Starting at the anchor, state 0 advances 0.05 per tick: after 10 ticks the distance is exactly 0.5 and the state remains 0 because the threshold is strict; tick 11 reaches 0.55 and switches to state 1. Vertical sinking starts on the following update. The launch direction is horizontal (y = 0), so the final state does not end at once: it tilts the stalk toward the launch direction and ends when |up.y| ≤ 0.001 (Reed angles, build guide). A zero horizontal displacement gives a zero launch vector: the piece never leaves its first fall state, stays upright and stays drawn (Default if unresolved: do not special-case it).
- Reed sinking, synthetic: start y = g + 2.0, where g is the ground probe result. The stopping boundary is g − 1.25, a 3.25-unit drop. After 46 updates y = g − 1.22, so the strict boundary has not been crossed; update 47 gives y = g − 1.29 and advances to the final fall state.
- Reed patch (real data), pack 909, record 63 (the 15-row key-8 patch above): row 0 plant 0 at x = −1829.364, z = 964.972; the other row-0 plants at x = −1819.564 and −1817.329; row 1 (z = 966.446) at x = −1827.129 and −1815.094; row 2 (z = 961.141) at x = −1824.034. Jitter is off, all plants have colour bytes 64, 64, 64, 128. Draw order: row by row, plants in stored order, three random draws per placed plant (facing x, facing z, scale).
- Reed yaw and scale, one plant with chosen draw values (the real draws come from the clock-seeded generator): draw fractions 0.75, 0.25 and 0.5 of the 32-bit range give facing x = −1 + 2 × 0.75 = 0.5, facing z = −1 + 2 × 0.25 = −0.5 and K = 0.73 + 0.27 × 0.5 = 0.865. Then φ = atan2(0.5, −0.5) = 135 degrees, forward = (0.7071, 0, −0.7071), right = (−0.7071, 0, −0.7071), up = (0, 1, 0); right × up = forward. The drawn model's top (local y 5.693) lies at 5.693 × 0.865 = 4.925 units straight above the base. Facing statistics: the chance that the facing is within 22.5 degrees of a diagonal is 0.586 (a uniform angle would give 0.5).
- Reed bend, same plant, hero radius R = 1.175, hero due west of the base at distance d (a = (1, 0, 0)), reed upright and latch clear:
  - d = 1.9: s = 0.725, w = 0.05, T = (0.0526, 0.9986, 0), tilt 3.0 degrees toward +x; latch stays 0.
  - d = 1.675: s = 0.5, w = 0.5, T = (0.7071, 0.7071, 0), tilt 45 degrees. Frame rebuild: right = (−0.4082, 0.4082, −0.8165), forward = (0.5774, −0.5774, −0.5774). The model top moves to 0.865 × 5.693 × T = (3.482, 3.482, 0) from the base. Latch stays 0.
  - d = 1.4: s = 0.225, w = 1, T = (1, 0, 0), tilt 90 degrees, latch set. Frame: up = (1, 0, 0), right = (0, 0, −1), forward = (0, −1, 0). The model top is at (4.925, 0, 0) from the base. While the hero stays in contact, nothing changes and the reed cannot be cut.
  - The hero steps back to d = 2.0 (no contact): latch cleared; spring-back then takes 49 steps from flat (tilt 85.4, 80.8, 76.3, ... degrees on steps 1, 2, 3).
  - Hero 1.0 unit from the base: s = −0.175, so w = 1 and the reed is flat and latched, as for d = 1.4.
- Reed walk-past (synthetic hero path, R = 1.175; base at the origin, hero walking along +x at 0.1 unit per step on the line z = −D, reed upright and latch clear at the start; each step runs the spring-back, then the contact test and bend; tilt from vertical after the step, degrees):
  - D = 1.8 (fringe): steps measured from the closest approach (step 0, d = 1.800). Steps −6 to 0 give d 1.897, 1.868, 1.844, 1.825, 1.811, 1.803, 1.800 and tilt 3.35, 7.31, 10.96, 14.07, 16.44, 17.93, 18.43 (an upright or easing-in reed takes the target at once). Steps 1 to 10 give tilt 17.93, 16.59, 15.25, 13.88, 12.42, 10.88, 10.06, 9.31, 8.62, 7.97 (the reed eases down behind the hero; the step-7 value is the first step with d above 1.925). The reed never latches; later the spring-back leaves it at 2.48 degrees from vertical.
  - D = 1.6 (deep fringe): tilt 4.73 at step −10 (d = 1.887), 12.26, 20.51, 29.06, 37.30, 44.70 (d = 1.676), 50.88, 55.67, 59.05, 61.04 and 61.70 at step 0 (d = 1.600), then 61.04, 59.05, 55.67, 51.92, 48.15, 44.28, 40.28, 36.16, 31.99, 27.85 on steps 1 to 10 and 25.83 at step 11 (contact over, spring-back only). The reed never reaches flat.
  - D = 1.3 (inside the flat band): tilt 1.71 at step −14 (d = 1.910), 11.82, 24.36, 38.62, 52.94, 65.58, 75.72, 83.45, 89.21 at step −6 (d = 1.432); at step −5 (d = 1.393, s = 0.218) the reed is flat (90 degrees) and latched. It stays flat and latched through step 14 (d = 1.910); at step 15 (d = 1.985) contact ends, the latch clears and the spring-back starts on step 16 (tilt 85.36).
- Reed cut fall, same plant (right = (−0.7071, 0, −0.7071), upright, base at ground height g), weapon tip moving horizontally toward +z so v = (0, 0, 1): tick 0 cut (drop roll, cue); ticks 1 to 11 the base moves 0.05 per tick along +z (z = 964.972 → about 965.52), tick 11 starts the sink; ticks 12 to 29 y falls 0.07 per tick to g − 1.26; ticks 30 to 242 the stalk rotates toward +z: up = (0, 0.9993, 0.0362) after the first rotation (tick 30), (0, 0.9838, 0.1791) after 5, (0, 0.938, 0.3466) after 10, (0, 0.7865, 0.6176) after 20, (0, 0.6093, 0.7929) after 30, (0, 0.2299, 0.9732) after 60, (0, 0.056, 0.9984) after 100, (0, 0.001, 1.0) after 213; with right rebuilt each step (after 10 steps right = (−0.7093, 0.2443, −0.6612), forward = (0.7049, 0.2458, −0.6654)); tick 243 sets gone and the reed is not drawn again until the world is loaded again.
- Reed drop: r = 0 gives one 25 NP coin; r = 5 gives a 5 NP coin; r = 6 and r = 19 give a 1 NP coin; r = 20 gives nothing. A reed cut and fallen in pack 909 is not drawn again until pack 909 is loaded again.
- Bush with the override set (200, 32, 50, 150), key 20: at d = 90 a = 1; at d = 125 a = 0.5; at d = 149 a = 0.02; at d = 150 the patch is off (gate |C − E|² < F² fails). b is 0 at every distance.
- Key-5 bush, hero 6 units away: the up axis does not change on that step, whatever its tilt. Hero 4 units away with no body contact and up axis (0, 0.99, 0.141) (not unit length): the new axis is normalize(0.02 × (0, 1, 0) + 0.98 × (0, 0.99, 0.141)) = normalize(0, 0.9902, 0.1382), about (0, 0.9904, 0.1382).
- Bush bend and speed: hero 3 units from a key-5 bush, body contact, speed 0.05 units/s: T is the current up axis (no lean). Same with speed 2.0 and the hero facing (1, 0, 0): T = 0.76 × (0, 1, 0) + 0.24 × (1, 0, 0) = (0.24, 0.76, 0); T.y < 0.994, so with the bush upright the new axis is normalize(0.02 × (0, 1, 0) + 0.98 × (0.24, 0.76, 0)) = normalize(0.2352, 0.7648, 0), about (0.2940, 0.9558, 0).
- Full far batch: 1,200 grass plants with b > 0 in view on one frame, all of one model: entries 1 to 1,000 are drawn and the last 200 are not; on the next frame the batch starts empty again. 750 far plants of other families: 700 drawn, 50 not.
- Group limit: far plants of 21 different far models in the 700-entry batch on one frame, 10 of each, added model by model: the 200 plants of the first 20 models are drawn; the 10 of the 21st model are not.
- Constructor defaults are replaced: a grass patch with overrides (50, 16, 10, 100) has F = 100, not 500, and its inner distance is 100.

## Open questions

- Default if unresolved (rewrite choice): draw only the documented primary, cut and far model choices; do not add a separate third-model draw without a checked use.
- Resolved (reed bend amount, High): the contact is the one sphere (radius 0.75, centre 1.4 above the base, independent of K) against the selected hero's vertical segment (height 3.5, radius 1.175), with no speed term and no other bend source; the weight formula, the 0.25 close threshold, the 0.075 and 0.925 blend, the 0.999 spring-back stop and the per-step order were all re-decoded and agree with Reed angles, build guide. Only the end-cap case (sphere centre above or below the segment) is unsettled; it needs the hero standing above or below a reed base, which is a player-reachable physics case (needs a runtime check); the default is the clamped segment distance in the next line.
- Default if unresolved (rewrite choice, reed angles): (a) hero body contact: use the sphere (radius 0.75, 1.4 above the base) against the hero's flat-ended vertical cylinder exactly as in Body contact above; (b) cut direction: the horizontal motion of the weapon tip between the previous and the current pose of the swing; (c) far sprite: centre on the base point, spin 0, sized by K, with the vertical extents of the existing far-sprite rule; (d) a zero launch vector or a zero horizontal hero offset: leave the reed unchanged instead of reproducing the broken frame.
- Resolved (reed angles, High): yaw is the direction of two uniform [−1, 1] draws (x, then z; before the scale draw), up is always world up, the hero bend jumps upright reeds to the target in one step, the latch is set only at s ≤ 0.25, the cut velocity is horizontal, the final fall state blends up 3.5 percent toward the launch velocity each step and ends on |up.y| ≤ 0.001, and there is no sway.

- Resolved (R-0046, part 2, High): key-4 trees have no separate trunk model. The tree builder loads model 200 as the near model and no far model for key 4, and it adds the one collision cylinder (section 4). The "trunk" is only that cylinder; model 200 is the whole tree. Key 25 uses the same model 200 as its near model (far model 199) and has no cylinder, so model 200 draws a complete tree on its own. Draw each key-4 patch with model 200 only and keep the cylinder as its solid part.

0. Resolved (R-0046, parts 1, 3 and 4): the LOD field mapping is in Behavior 3 (the far distance from the fade end, the fade band from the fade length, the switch from +0x54 and the out band from +0x50). The near and far draws differ in the model drawn and in the batch: a plant with a weight above zero is drawn with the patch's near model, while the far weight sends it to the shared far batch with the far model; the far entry's alpha byte is the plant's alpha times the far weight (Behavior 4; Medium, since no other difference was found). The once-per-area save words are the five sets in SPEC-0025 (the rows at 0x2EB2 to 0x2F16 slot offsets; Behavior 2).
- Resolved (R-0046, part 2): there is no trunk model, registry id or placement beyond the collision cylinder (see the first line of this section).
1. B-family third model ids are supplied during build, but the documented draw chooses only the primary model or the cut model; their appearance and any other use are not encoded anywhere readable (class D; the draw never chooses them, so a rewrite omits them). The H and K model roles are specified in Behavior 1. Resolved (Medium-High): S plants have eight independently updated subrecords. Hero contact sets a per-step touched flag; closer contact tilts the part frame and the rendered pose, and the frame returns gradually toward rest when contact ends (Behavior 10). Which petal or leaf of the flower mesh each subrecord drives is not encoded in the data (class D); Default if unresolved (rewrite choice): tilt the eight parts about their own frames and let the flower model's skeleton, if any, follow in order.
2. Resolved (Medium-High): the hero bend formula is given in Behavior 6 (Hero bend), for key-5 bushes in the helper steps above it, and reeds use the bend rule of the other bushes. The hero weapon segment is a capsule of radius 0.5 and full length L (the hero's swing length) centred on the anchor of attachment frame 0 plus 0.25 times its Z row, with that Z row as its axis; the swept fallback uses the triangle of SPEC-0036 (Behavior 6, Weapon).
3. Resolved: with b > 0 the plant is not drawn with a model of its own call; it is added to one of two shared far batches (1,000 entries for grass, 700 for the other families), emptied at the start of every frame and drawn once per frame (Behavior 4, Draw). When a batch is full (its room, or a 21st group of far model and material) the add is refused and that plant is not drawn on that frame, with no fallback (Behavior 4, Full batch). Resolved: the render path of a batch. Each far entry is one sprite element of the camera-facing billboard program (SPEC-0003, Billboard program 3; SPEC-0046, which program serves which draw). The program takes the entry's colour words unchanged (so the bytes are GS vertex colour, as in Plant colour in the draw), moves the centre through the view-projection matrix, and offsets four corners by 512 × the half-extent lanes, turned by the entry's spin sine and cosine, before the divide by clip w (SPEC-0034, section 16.4). A sprite with clip w of 1 or less, or with any corner beyond the 0 to 4094 screen range, is not drawn. Test vector for the colour path with the real grass plant of pack 717 (bytes 64, 64, 64 and alpha 38, b = 1): the strip's four vertices all carry (64, 64, 64, 38); a sprite centred on the view axis at clip w = 10 with half-extents (hx, hy, hz) = (0.5, 0.5, 1.0) and no spin spans 102.4 pixels across and 51.2 pixels down. (High for the program and its colour handling; the extent lanes of a plant entry are not described here.)
4. Resolved: yes. The eye is the position of the single camera object, which every camera mode moves (Behavior 4). (Medium-High)
5. Resolved: a cut reed does not grow back. When its fall ends (the final fall state finds |up.y| ≤ 0.001, about 243 ticks after the cut) it is gone (state 2) and is not drawn (Reed state). A gone reed returns only when the world is loaded again. Each cut also rolls the default coin drop (Behavior 9). (High)
6. Resolved: no spring-back at all beyond 5 units; within 5 units the axis springs back by 2% per step, toward a blend with the hero's facing on body contact (Behavior 6, Helpers). The value compared with 0.1 is the hero's movement speed in units per second, and U is the constant world up (0, 1, 0). (High for the 5-unit rule and U; Medium-High for the speed; Medium for the blend)
7. Resolved: no. The set is used only by 80 bush patches, which have no far object; they fade out from 100 to 150 (Edge cases). (High)
8. Resolved: the +0x3C word of the record (0xFFFFFFFF in packs 678 and 713) is never read by the game (Layout, Patch record). (Medium-High)
9. Resolved (R-0072): the plant colour word does not reach the near model draw (which uses the fixed default colour with the near weight as opacity). It reaches only the far sprite batch, as GS vertex colour in GS units (128 = 1.0), with the alpha byte multiplied by the far weight b. It is neither the draw colour k nor the colour scale s (Behavior 4, Plant colour in the draw). (High for the paths and for the final look: the far sprite material is MODULATE with the ordinary blend, see Sprite material state)
