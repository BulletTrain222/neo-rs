# SPEC-0060: Foliage patches (type 16)

- Answers: SPEC-0006 open questions on the type-16 fields, build selector, thresholds, query point, items and purpose; SPEC-0028 open questions 57 to 59 (six-slot look-ahead) and 80 (type-16 field reading); proactive otherwise (bush cutting, drops, regrowth, trees and reeds); R-0046 (in part; the LOD fields and the trunk model open in Open questions); R-0072 (the plant colour word in the draw)
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
6. Each plant gets a random yaw (a random horizontal facing).
7. Each plant gets a uniform scale K in [s0, 1]. s0 is 1.0 for G, 0.6 for S and B, 0.85 for C and H, and 0.73 for K. Bushes of model 265 (key 34, not on disc) use [1.0, 3.5].
8. The plant keeps its four colour bytes (R, G, B, A) as stored. The alpha byte is kept a second time on the plant instance. Both are used only by the far sprite batch (Behavior 4, Plant colour in the draw). Disc census: 21,012 of the 29,853 plants have the bytes 64, 64, 64, 128; the alpha byte differs from 128 only in the grass, tree and reed keys that have a far sprite (5,844 plants).
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
     - **Which plants use which path.** Grass (keys 0 and 12; 6,249 plants) has no near model and b = 1 at every distance, so the colour word is its only colour control: all grass is drawn with its word as GS vertex colour. Trees of keys 13 and 25 and reeds (key 8) show the near model close up (colour word ignored) and the sprite at range (colour word used, so a 64-grey sprite is half brightness while the near model is full brightness; Medium that no cross-fade correction exists, because nothing in the build or the draw compensates). All bush, flower and clover patches and key 4 trees have no far object, so their colour words (including the tinted words of key 16) have no effect at all.
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

**Hero bend (every bush key except 5; Medium-High).** It runs each step after the spring-back, and only while the squared horizontal distance from the hero to the bush's raised test point (the bush base plus its up axis times the plant's height scale) is below 25 and the hero's body test against the bush passes (the segment test of radius 0.75 used for key 5). If the test passes and the bend latch is clear, the latch is set and the bush is bent once. If the test passes and the latch is already set, nothing changes: the bush stays as it is, and it is not sprung back while the latch is set. The latch is cleared on every step in which the body test fails. The bend: let a be the horizontal unit vector from the hero's position to the bush base, and s the horizontal distance between the two minus the hero's body radius. The lean weight is w = 1 when s is at most 0.25, and otherwise w = 1 - 2 (s - 0.25). The target axis is T = normalize(w a + (1 - w) (0, 1, 0)). If the bush's current up axis has a smaller y than T (the bush is already leaning further than the target), the new up axis is normalize(0.075 T + 0.925 up); otherwise the new up axis is T at once. The frame is rebuilt around the new up axis. Test vector: hero radius 0.5, hero 1.0 unit from the bush base along +x (a = (1, 0, 0)), bush upright, latch clear: s = 0.5, w = 0.5, T = (0.7071, 0.7071, 0); the current up y (1.0) is not below T.y (0.7071), so the up axis becomes T at once and the latch is set. Hero 0.6 units away: s = 0.1, w = 1, T = (1, 0, 0), a bush lying flat away from the hero.

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

- **Bend.** A reed within 5 units of the hero tilts away. When it is free it springs back as a bush does.
- **What happens when the hero steps across one (static estimate).** Reeds do not have a weight or compression response, and the patch does not register a standable platform. They bend only when the hero-body contact test succeeds. For a close contact whose horizontal direction from hero to reed is nonzero, the shared bend rule chooses a horizontal target axis when `s = horizontal distance − hero radius` is at most 0.25; from upright, it applies that target in one step, rather than easing it down over time. Model 202's local upper stem reaches y = 5.693, and reed placement scale K is 0.73 to 1.0. A fully horizontal lean therefore removes about 5.69 × K = 4.16 to 5.69 units from the stem's vertical projection. Side branches and the rebuilt side axes can leave other vertices above or below this estimate. Exact on-top/zero-horizontal-offset behavior and the rendered displacement are not established.
- **Cut.** A hit by the hero's weapon in the attack window cuts the reed. The cue is 22 (Tor) or 28 (Roberta). The cut piece's initial direction comes from the change between two sampled weapon-pose points, each offset 2.5 units along its sampled frame axis. The resulting vector is normalized, so the initial velocity has length 1; the displacement step in the first fall state is therefore 0.05 units per tick. This is a motion-derived knock-away direction, not the weapon's full swing speed.
- **Fall of the cut piece**, in three states:
  1. State 0: position += 0.05 × velocity per tick, until the piece is more than 0.5 from its anchor.
  2. State 1: y −= 0.07 per tick, until y is below (the ground probe result with ceiling y + 8.0) − 1.25 (SPEC-0018, Ceilings supplied by callers).
  3. State 2: if |velocity.y| > 0.001, up becomes normalize(0.965 × velocity + 0.035 × up) and the side axes are rebuilt. If |velocity.y| ≤ 0.001, the reed becomes gone immediately. Because velocity stays fixed, a launch with a non-negligible vertical component keeps updating this state indefinitely; a nearly horizontal launch skips the alignment update and becomes gone.
- This is the three-state routine that SPEC-0028 calls the launched-object after-flight fall. Contrary to SPEC-0028, it runs in play, for the 24 reed records in packs 909, 934 and 1056.
- **Placements and a coincident duplicate (High for disc data).** The 24 patches contain 491 plant entries: pack 909 has 14 patches/298 plants, pack 934 has 8/101, and pack 1056 has 2/92. Both pack-1056 patches have the same box, base height, nine row counts and offsets, and all 46 plant records match; they are stored in separate type-16 chunks. Model 202 is present in all three packs, so both pack-1056 records pass the reed build's model-presence gate. They therefore describe two built patch instances at the same 46 plant positions. Whether the overlapping instances produce duplicate bend, cut or drop effects in play is not established; preserve both instances unless a single visible/interactive reed per position is established.
- **Reed state (High).** Each reed has a state word: 0 standing, 1 cut and falling, 2 gone. The logic step runs the bend and the weapon test in state 0 and the fall in state 1; it does nothing in state 2. The cut sets state 1 and the fall's own sub-state 0. Only a state-2 fall update with |velocity.y| ≤ 0.001 sets the reed to gone. A cut piece with |velocity.y| > 0.001 remains in state 1 while its frame continues to update, so it remains eligible for drawing.
- **Gone reeds are not drawn and do not grow back (High).** A reed in state 2 fails its visibility test, so the draw skips it. Nothing sets a reed back to state 0 except the plant reset, which runs only when the patch is built again (the next load of the world). Bushes regrow (section 6); reeds do not.
- **Reed drop (High).** On each cut, one roll r is a uniform integer from 0 to 100, both ends included. r = 0 gives item 138 (25 NP), r from 1 to 5 gives 136 (5 NP), r from 6 to 19 gives 135 (1 NP), and r from 20 to 100 gives nothing. This is the same rule as the default bush drop (section 6), and luck plays no part. The drop spawns at the reed's position through the item spawn route of SPEC-0021.

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
- Reed fall, cut-launch velocity: any nonzero sampled weapon-pose displacement normalizes to length 1.0. Starting at the anchor, state 0 advances 0.05 per tick: after 10 ticks the distance is exactly 0.5 and the state remains 0 because the threshold is strict; tick 11 reaches 0.55 and switches to state 1. Vertical sinking starts on the following update. If the fixed launch direction has |y| ≤ 0.001 when state 2 begins, the reed becomes gone on its first state-2 update. If |y| > 0.001, the state-2 axis update repeats each tick without reaching the gone assignment. A zero displacement cannot produce a valid normalized launch direction; its handling for shipped hits is not established.
- Reed sinking, synthetic: start y = g + 2.0, where g is the ground probe result. The stopping boundary is g − 1.25, a 3.25-unit drop. After 46 updates y = g − 1.22, so the strict boundary has not been crossed; update 47 gives y = g − 1.29 and advances to the final fall state.
- Reed drop: r = 0 gives one 25 NP coin; r = 5 gives a 5 NP coin; r = 6 and r = 19 give a 1 NP coin; r = 20 gives nothing. A reed cut and fallen in pack 909 is not drawn again until pack 909 is loaded again.
- Bush with the override set (200, 32, 50, 150), key 20: at d = 90 a = 1; at d = 125 a = 0.5; at d = 149 a = 0.02; at d = 150 the patch is off (gate |C − E|² < F² fails). b is 0 at every distance.
- Key-5 bush, hero 6 units away: the up axis does not change on that step, whatever its tilt. Hero 4 units away with no body contact and up axis (0, 0.99, 0.141) (not unit length): the new axis is normalize(0.02 × (0, 1, 0) + 0.98 × (0, 0.99, 0.141)) = normalize(0, 0.9902, 0.1382), about (0, 0.9904, 0.1382).
- Bush bend and speed: hero 3 units from a key-5 bush, body contact, speed 0.05 units/s: T is the current up axis (no lean). Same with speed 2.0 and the hero facing (1, 0, 0): T = 0.76 × (0, 1, 0) + 0.24 × (1, 0, 0) = (0.24, 0.76, 0); T.y < 0.994, so with the bush upright the new axis is normalize(0.02 × (0, 1, 0) + 0.98 × (0.24, 0.76, 0)) = normalize(0.2352, 0.7648, 0), about (0.2940, 0.9558, 0).
- Full far batch: 1,200 grass plants with b > 0 in view on one frame, all of one model: entries 1 to 1,000 are drawn and the last 200 are not; on the next frame the batch starts empty again. 750 far plants of other families: 700 drawn, 50 not.
- Group limit: far plants of 21 different far models in the 700-entry batch on one frame, 10 of each, added model by model: the 200 plants of the first 20 models are drawn; the 10 of the 21st model are not.
- Constructor defaults are replaced: a grass patch with overrides (50, 16, 10, 100) has F = 100, not 500, and its inner distance is 100.

## Open questions

- Default if unresolved (rewrite choice): draw only the documented primary, cut and far model choices; do not add a separate third-model draw without a verified use.

- Resolved (R-0046, part 2, High): key-4 trees have no separate trunk model. The tree builder loads model 200 as the near model and no far model for key 4, and it adds the one collision cylinder (section 4). The "trunk" is only that cylinder; model 200 is the whole tree. Key 25 uses the same model 200 as its near model (far model 199) and has no cylinder, so model 200 draws a complete tree on its own. Draw each key-4 patch with model 200 only and keep the cylinder as its solid part.

0. Resolved (R-0046, parts 1, 3 and 4): the LOD field mapping is in Behavior 3 (the far distance from the fade end, the fade band from the fade length, the switch from +0x54 and the out band from +0x50). The near and far draws differ in the model drawn and in the batch: a plant with a weight above zero is drawn with the patch's near model, while the far weight sends it to the shared far batch with the far model; the far entry's alpha byte is the plant's alpha times the far weight (Behavior 4; Medium, since no other difference was found). The once-per-area save words are the five sets in SPEC-0025 (the rows at 0x2EB2 to 0x2F16 slot offsets; Behavior 2).
- Resolved (R-0046, part 2): there is no trunk model, registry id or placement beyond the collision cylinder (see the first line of this section).
1. B-family third model ids are supplied during build, but the documented draw chooses only the primary model or the cut model; their appearance and any other use remain unknown. The H and K model roles are specified in Behavior 1. Resolved (Medium-High): S plants have eight independently updated subrecords. Hero contact sets a per-step touched flag; closer contact tilts the part frame and the rendered pose, and the frame returns gradually toward rest when contact ends (Behavior 10). The visible identity of each subrecord in the flower mesh remains unknown.
2. Resolved (Medium-High): the hero bend formula is given in Behavior 6 (Hero bend), for key-5 bushes in the helper steps above it, and reeds use the bend rule of the other bushes. The hero weapon segment is a capsule of radius 0.5 and full length L (the hero's swing length) centred on the anchor of attachment frame 0 plus 0.25 times its Z row, with that Z row as its axis; the swept fallback uses the triangle of SPEC-0036 (Behavior 6, Weapon).
3. Resolved: with b > 0 the plant is not drawn with a model of its own call; it is added to one of two shared far batches (1,000 entries for grass, 700 for the other families), emptied at the start of every frame and drawn once per frame (Behavior 4, Draw). When a batch is full (its room, or a 21st group of far model and material) the add is refused and that plant is not drawn on that frame, with no fallback (Behavior 4, Full batch). Resolved: the render path of a batch. Each far entry is one sprite element of the camera-facing billboard program (SPEC-0003, Billboard program 3; SPEC-0046, which program serves which draw). The program takes the entry's colour words unchanged (so the bytes are GS vertex colour, as in Plant colour in the draw), moves the centre through the view-projection matrix, and offsets four corners by 512 × the half-extent lanes, turned by the entry's spin sine and cosine, before the divide by clip w (SPEC-0034, section 16.4). A sprite with clip w of 1 or less, or with any corner beyond the 0 to 4094 screen range, is not drawn. Test vector for the colour path with the real grass plant of pack 717 (bytes 64, 64, 64 and alpha 38, b = 1): the strip's four vertices all carry (64, 64, 64, 38); a sprite centred on the view axis at clip w = 10 with half-extents (hx, hy, hz) = (0.5, 0.5, 1.0) and no spin spans 102.4 pixels across and 51.2 pixels down. (High for the program and its colour handling; the extent lanes of a plant entry are not described here.)
4. Resolved: yes. The eye is the position of the single camera object, which every camera mode moves (Behavior 4). (Medium-High)
5. Resolved: a cut reed does not grow back. When its fall ends (a final fall state with |velocity.y| ≤ 0.001) it is gone (state 2) and is not drawn; a cut piece whose launch keeps |velocity.y| > 0.001 stays in state 1 and is still drawn (Reed state). A gone reed returns only when the world is loaded again. Each cut also rolls the default coin drop (Behavior 9). (High)
6. Resolved: no spring-back at all beyond 5 units; within 5 units the axis springs back by 2% per step, toward a blend with the hero's facing on body contact (Behavior 6, Helpers). The value compared with 0.1 is the hero's movement speed in units per second, and U is the constant world up (0, 1, 0). (High for the 5-unit rule and U; Medium-High for the speed; Medium for the blend)
7. Resolved: no. The set is used only by 80 bush patches, which have no far object; they fade out from 100 to 150 (Edge cases). (High)
8. Resolved: the +0x3C word of the record (0xFFFFFFFF in packs 678 and 713) is never read by the game (Layout, Patch record). (Medium-High)
9. Resolved (R-0072): the plant colour word does not reach the near model draw (which uses the fixed default colour with the near weight as opacity). It reaches only the far sprite batch, as GS vertex colour in GS units (128 = 1.0), with the alpha byte multiplied by the far weight b. It is neither the draw colour k nor the colour scale s (Behavior 4, Plant colour in the draw). (High for the paths and for the final look: the far sprite material is MODULATE with the ordinary blend, see Sprite material state)
