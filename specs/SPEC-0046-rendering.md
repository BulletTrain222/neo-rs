# SPEC-0046: Rendering and draw order

- Answers: proactive (open close-out) (frame draw submission, distance culling, near and far state selection, part visibility, sort order and flush, per-object light slots; and the screen-space colour sheet; the view and projection build; camera-facing billboard draw), R-0084 (atmosphere overlay blend, frame order, and fade-quantizer use), R-0086 (2) (Mote-glow vertex colour, animated alpha, and additive blend), R-0087 (3, rope-manager segment model key and spacing, render traversal, and cue/sample-record references; mesh identity and audible sound remain open); R-0104 (Rope Gun strand model and cue records; visual mesh and audible identity remain open); R-0126 (base alpha of program-6 leaves; Roberta's head dome); R-0128 (program 6 is drawn with texture function highlight 2, so vertex alpha 0 is a solid surface and the specular alpha is additive); proactive (Meridell mountain draw: what the mountains are, their depth, culling and sky relation, and the likely causes of mountain and sky-dome artifacts); R-0155 follow-up (which model draws pass a light context; coin placement as screen fractions and depth); R-0172 (Gnarfas's shadow hulls and how they follow his bones); R-0202 (the fade layer's blend, depth and the ghost look); R-0221 (the pack 1299 mask tree and its private draw mask); R-0227 (the Mote glow models are not gated by the weapon-drawn flag); proactive (open question sweep, rendering); proactive (open question sweep, round 5); proactive (open question sweep, round 9); proactive (open question sweep, round 10); proactive (draw program read); R-0232 (answered from the code; not run in the original); R-0260 (answered from the code; not run in the original)
- Status: Draft
- Disc build: SCUS-97367 v1.00 (same fingerprint as SPEC-0001)
- Confidence: per section. Display scale clamp, fade quantizer and default colour word: High for the values; the fade quantizer's use as the draw colour scale High; Medium for the use of the scale clamp and the colour word. 3D book open angle: High (there is no blob shadow). Object lists and the special object: High, including the two fixed sets of the second-world pairing test (decoded in SPEC-0006). Frame sequence and per-frame draw order: High. Submit distance cull (350 units): High. Far flag (50 units): High for the mechanism and for its effect (one-bone skinning in the vertex program). Vertex-program state bits: High (the fade bit has no reader). Per-triangle clipping in the vertex program: High for the rules, Medium-High for the clip record values. Brightness factor and the specular alpha term of program 6: High for the arithmetic, Medium for the frame of the reference point. Distance metric: High (3D for the submit cull, the far flag and the shadow casters; horizontal for the draw class B level-of-detail flag). Draw classes: High for the dispatch (every character render object uses class B); High for the absence of a reader of the class B flag. Draw list capacity: High. Sort rule and translucency (character fade): High. Per-part visibility classifier: High for the steps, the side masks and the three outcomes. Corner builder and eight-corner clip test (below): High for the arithmetic; Low for the caller of the standalone builder. Projection values: High; the 600 and 10560 clip sets and the second GS variant unused in play High. Story-layer mask of world nodes: High for the rule and for the role of the world mask word. Runtime four-plane volume list: High for the mechanism, Medium-Low for its meaning, inactive on disc. Light slots: High for four slots per light context, the 5 percent blend and its direction (toward the new value). Lit draw (the consumer of the slots): High for the light block and its source, Medium-High for the vertex arithmetic. Scene default lights: High. Draw colour scale and opacity: High. The 0x50-byte records once called light templates are the swing-trail colour gradients of SPEC-0034 section 18 (used by the swing trail of SPEC-0034 3.8) (High). Hit flash (draw step 6): Medium-High for the ramp and the override value; High for the override-slot rule (the blend reads slot 0; the part draw reads slot 1 as the object colour, so the flash tint is shown); Medium for the on-screen pulse. Type 30 light sets (character path only): High for the reader chain, the walk timing, the walk order, the gates, the slot rules (including the displacement and the cross-fade), the slot kinds, the type 3 weight and the dynamic point lights. Character mesh path on the main processor and the first vector unit: Medium-High. Fog: there is no fog (High). Vertex pass size: Medium. Main-character screen bounds: High for their use as the fade composite rectangle. Screen-space colour sheet (never drawn on this disc, High): layout High; colour conversion rules High; the two overlay blocks (C and D) Low; whether the sheet is player-visible or a debug display Low. View and projection build: eight 4x4 products High for the count, Medium for their use; culling frustum (tangent 1.0, aspect term) High; the aspect convention Medium. HUD spinning coin (item 135, turning 1 radian per second): High for the gate, the model, the spin, the screen-fraction and depth meaning of the placement, the depth test and the light context, Medium-High for the sideways direction of the offset; the separate camera-facing quad builder Medium-High. Bounding radius of a box: High for the arithmetic, Medium for its use. Packet submit size limit: Medium-High. Chain point mover, weight routines and corner routines: Low. Character shadow volumes: High for the gate, the caster choice, the light direction, the extrusion, the caps, the count colours and the resolve state; High for the visible size of the darkening (8 per channel); no rendered comparison made. Sky domes: High for the resources, the per-world choice, the buffer and the drawing; Medium-High for resource 528 being unused and for the per-frame rotation rate. Held items and character attachments: High for the pass order, the gates, the matrices and the scales; Medium for the Mote-glow anchors and the rope effect. Items marked **[Low]** must not be implemented as settled. Character light slots (the slot chooser): High for the storage and the slot choice.
- Related specs: SPEC-0003 (material blend and alpha), SPEC-0004 (mesh batches, node culling, attach points and the item model registry key), SPEC-0005 (skeleton and skinning), SPEC-0006 (level display tree), SPEC-0007 (game timing), SPEC-0008 (platform render-state brackets), SPEC-0013 (Gelert Assassin mirror images, hidden by base state 42), SPEC-0017 (despawn flag), SPEC-0018 (ground and collision, not render), SPEC-0021 (item display slots, item model registry, item showcase), SPEC-0027 (camera, view point and fade writer), SPEC-0028 (world objects and atmosphere; placed objects are drawn by the world draw, not from the object lists of this spec), SPEC-0030 (character slot records), SPEC-0034 (effects pass), SPEC-0035 (3D book), SPEC-0040 (hero and base actor state), SPEC-0047 (petpets), SPEC-0049 (story flags), SPEC-0052 (screen fade and letterbox bars), SPEC-0060 (foliage patches: grass, bushes, trees, reeds and flowers are drawn by the foliage pass with its two shared far batches, not from the object lists of this spec).

## Summary

Every frame, the game does not draw objects as it finds them. It first collects the objects to draw into one short list, then issues all of the draws at once at the end of the frame. Collection walks a few fixed character lists that are updated once per frame (the two petpet asset slots, the current world's character slots, the neighbour world's character slots, the active petpet as a special object, and the hero). Each object is kept only if it is drawable and closer than 350 units to the view point. Kept objects are inserted into a sorted list that holds at most 18 entries per frame: opaque objects come before translucent (fading) ones, and within each group the farther object is drawn first. At the end of the frame the sorted list is walked once and each object draws itself. Each object's distance also selects one of two global vertex-program state sets: a near set, or a far set when the object is more than 50 units away; the far set skins each vertex with one bone. Models are culled part by part from the eight corners of each part's bounding box, with three outcomes: culled, inside, or clipped. Character shadows are stencil shadow volumes for at most three nearby casters. The full order of one frame's draws (background, sky dome, worlds, characters, effects, book, screen fade, HUD) is in Per-frame draw order. A second volume test against runtime-built four-plane volumes exists, but its list is empty on disc. Lit models (drawn with the lit vertex programs 1, 2 and 6, chosen by the material) are lit in the vertex program from a per-draw light block of one ambient and three directional lights, built from the four slots of a light context; level geometry is unlit, and there is no fog. A box bounding radius (0.7072 times the larger of the box's X and Z extents) and a size-limited packet submit are also described, with a few curve, chain and corner helpers at Low confidence. Characters also build their own four-slot light state from the area's light set and the frame's dynamic point lights, once per main-loop pass (see Lights, Character lights); that state is the light context the character's model draws with. Each world has a sky dome: on every world load one of three dome resources (1, 2 or 3) is chosen by the world's slot number and read into a fixed buffer (see Sky domes). The dome is drawn first every frame, in world space at the world origin, with one layer whose opacity follows the atmosphere-zone blend factor.

A UI node of kind 8 (the front-end and HUD node tree of SPEC-0035) draws a screen-space colour sheet, but no node of that kind exists on the disc, so the sheet is never drawn: a hue spectrum block and a value ramp block, both in the colours of one chosen hue, saturation and value. The three colour indices are run-time state. They are derived from a colour record when it is loaded and are stepped by the pad. The sheet is built from quads with a colour per vertex and no texture. The frame setup also builds a view block from the camera, multiplies it by eight source matrices of the object, and builds the culling frustum (tangent 1.0 on the vertical axis, used only for culling).

The clean side can use this spec to build the per-frame draw order, the two distance thresholds, the per-part visibility test, the colour sheet and the view and projection build. The 18-entry limit is an original fixed size, not a rule for the clean side. The HUD spinning coin (item 135, a three-dimensional model drawn in the HUD pass) is also part of rendering and has its own section.

## Where it lives on the disc

Most values here are fixed constants of the boot program, not data words on the disc, and no data address is needed for them: the 350-unit cull, the 50-unit far threshold (both stored as squared distances, 122500 and 2500), the 128 translucency threshold, the 18-entry capacity, the 0.001 bias in the edge clip, the colour sheet's block sizes and origins, the culling frustum tangent of 1.0, and the 0x80 alpha byte of the sheet's vertices. The class B level-of-detail values (threshold 150 units for hero render records, 0 for others, margin 25.0), the shadow caster limits (three casters, squared distance 2500, kind key 720), the side masks of the classifier, the story-layer rules and the camera projection values are fixed constants too. The bounding radius factor (0.7072) and the packet size limit (1024 bytes) are also fixed constants. The factor's 32-bit and 64-bit encodings do not occur in the boot data.

The display scale clamp (0.1 to 1.0), the base size pair (512 and 448), and the default colour word are fixed constants too; no disc data is needed for them. The fade quantizer's index arithmetic (the 256 multiplier and the 0 to 256 index clamp) is also fixed; its range bounds are run-time words (see below). The values below are data words in the boot program's data, read at run time. No code writes the gate words; their disc values are the values in play (High).

| Value | ELF virtual address | Disc value | Use |
| --- | --- | --- | --- |
| Light blend weight | 0x2EC9D4 | 0.05 (32-bit float) | Step size of the kind-0 light slot update (see Light slots and the lit draw) |
| Volume cap | 0x2EC884 | 14 (32-bit integer) | Upper bound of the runtime volume list (it is clamped to 32) |
| Volume count | 0x2EC880 | 0 | Number of runtime volumes. Zero on disc; the list starts empty and is built at run time |
| Book open angle start | 0x2DB2E0 | 1.2 (32-bit float) | Start value of the 3D book's open angle (see 3D book open angle) |
| Book open angle floor | 0x2DB2E4 | 0.12 (32-bit float) | Lowest value of the 3D book's open angle |
| Petpet and hero submit gate | 0x2D7518 | 1 (32-bit integer) | The special object and the hero are submitted while nonzero |
| Frame copy tint gate | 0x2D752C | 1 | The frame copy with the atmosphere colour tint runs while nonzero |
| Light-flash pass gate | 0x2D7534 | 1 | The character light-flash pass runs while nonzero |
| Hero hit-flash force | 0x2D7600 | 0 | When nonzero, the hit flash is forced on the hero (see draw step 6) |
| Effects pass gate | 0x2D7540 | 1 | The effects pass runs while nonzero |
| Petpet shadow gate | 0x2D7608 | 0 | Petpet slot characters cast shadows only while nonzero (so on disc they cast none) |
| Background fade level | 0x2D7614 | 128.0 (32-bit float) | Fade level of the wrap around the background pass |
| Shadow pass gate | 0x2ECCD0 | 1 | The shadow volume pass runs while nonzero |
| Shadow bracket skip | 0x2ECCD4 | 0 | When nonzero, the render-state bracket around the shadow pass is skipped: the shadow buffer is not cleared or resolved, so no shadow shows |
| Shadow extrusion length | 0x2ECCC8 | 16.0 (32-bit float) | Length E of the shadow volume extrusion along the light direction (see Character shadow volumes) |
| Shadow straight-down switch | 0x2ECCCC | 0 | When nonzero, every caster's shadow light direction is (0, −1, 0) |
| Shadow light y limit | 0x2EC9E4 | −0.4 (32-bit float) | Highest y component of a shadow light direction |
| Shadow direction ease, slot change | 0x2EC9D8 | 0.05 (32-bit float) | Rate of the shadow direction ease after a slot change; the ease has no effect (see Edge cases) |
| Shadow direction ease, settle | 0x2EC9DC | 0.5 (32-bit float) | Scale of the settling ease rate; no effect (see Edge cases) |
| Shadow direction ease, frames | 0x2EC9E0 | 20 (32-bit integer) | Frame count of the settling ease; no effect (see Edge cases) |
| Shadow count colour, add | 0x323368 | 0x80080808 (packed R, G, B, A bytes 8, 8, 8, 128) | Colour of a shadow polygon whose outward side faces the camera (+8) |
| Shadow count colour, subtract | 0x32336C | 0x80F8F8F8 (bytes 248, 248, 248, 128) | Colour of any other shadow polygon (248, that is −8 modulo 256) |
| Shadow resolve colour | 0x2D7610 | 0x20080808 (bytes 8, 8, 8, 32) | Vertex colour of the resolve draw that darkens the frame |
| Second-world excluded pairs | 0x31EE80 | 4 pairs of 32-bit integers: (797, 758), (797, 753), (1392, 1370), (991, 699) | Pairs of current and second world pack that never draw the second world (either order) |
| Second-world excluded packs | 0x31EEA0 | 16 32-bit integers (listed in Object lists and the special object) | Current world packs that never draw the second world |
| Dome step, role-2 layer | 0x2DB488 | 0.0009 (32-bit float) | Angle step in radians per frame of the role-2 sky dome layer (see Sky domes) |
| Dome step, role-3 layer | 0x2DB48C | 0.0006 (32-bit float) | Angle step in radians per frame of the role-3 sky dome layer (see Sky domes) |
| Dome draw switch | 0x2D7528 | 1 (32-bit integer) | The sky dome is drawn each frame while this is nonzero |
| Sheathed weapon size | 0x2DD8AC | 2.6 (32-bit float) | Numerator of the sheathed-weapon scale (see Held items and character attachments) |
| Sheathed weapon scale cap | 0x2DD8B0 | 0.6 (32-bit float) | Upper bound of the sheathed-weapon scale |
| Brightness low bound | 0x2ECAE4 | 0.2 (32-bit float) | Mean brightness at or below which the brightness factor f is its floor (see Brightness factor) |
| Brightness high bound | 0x2ECAE8 | 0.6 (32-bit float) | Mean brightness above which f is its ceiling |
| Brightness factor floor | 0x2ECAEC | 0.3 (32-bit float) | Lowest f, and f when no light colour is nonzero |
| Brightness factor ceiling | 0x2ECAF0 | 1.0 (32-bit float) | Highest f |

The fade quantizer's range words are not on disc. Its two pairs of range bounds are at data addresses 0x3971A8 and 0x3971AC (variant A) and 0x39A1E0 and 0x39A1E4 (variant B), and its two 257-entry tables start at 0x394170 and 0x3971B0. All of these lie beyond the end of the boot file image, so they are zero on disc. They are set once, by the graphics init of the start-up frame setup, when the two tables and the variant-A reset record (data address 0x39A1F0) are built: variant A 0.008 and 1.0, variant B 0 and 1.0 (see Fade level quantizer and draw colour scale). The clean side must not read them from the disc; it uses these values.

The colour sheet's three colour-state words (boot data 0x2EC8B0 to 0x2EC8C8, the three derived indices and their three sources) are zero in the boot data (all seven words). They are run-time state, not tuning constants. The clean side must not read them from the disc; it derives the colour state as described in Colour sheet.

The object lists that the frame draws from are character slot records (SPEC-0030 section 4) and world character slot tables; see Object lists and the special object. They are run-time state, not disc data.

The camera setup record (data address 0x35C928) is zero on disc. It is filled at boot with the fixed projection values of Camera projection; the clean side must not read it from the disc.

Sky domes (High): archive resources 1, 2 and 3 of GAMEFILE.DAT (SPEC-0001, SPEC-0002) are the three sky domes. Resource 0 is a byte-identical copy of resources 1 and 2 and is never selected. Resource 528 has the dome layout but nothing loads it (Medium-High). The dome buffer is 460,800 bytes, allocated once at boot. The choice of dome is a fixed rule of the boot program (see Sky domes), not a data table.

## Layout

These are logical fields of the per-frame structures. Byte offsets are not part of the spec.

Draw record (one per collected object, at most 18 per frame):

| Field | Type | Meaning |
| --- | --- | --- |
| Object | reference | The object whose draw method is called at flush |
| Position copy | 3 values | The object's position at the time of submission |
| Squared distance | float | Squared distance from the view point to the object's position |
| Far flag | bit | Set when the squared distance is more than 2500 (50 units squared). At flush it selects the far vertex-program state set; clear selects the near set |
| Translucency class | opaque or translucent | From the object's fade record (see Sort order) |
| Fade record present | bit | Whether a fade record is passed with the object. If so, its draw is wrapped in the fade begin and end (see Translucency) |

Frame draw list: an ordered list of draw records, a count, and a capacity of 18.

Fade record (the render sub-record; one per character render object and per frame buffer): fade on (flag), fade level (integer 0 to 128; its writers are listed under Sort order), and the character's screen rectangle (see Main-character bounds).

Light context (four slots; slot 0 is the ambient light, slots 1 to 3 directional lights; see Light slots and the lit draw):

| Field | Type | Meaning |
| --- | --- | --- |
| Kind | integer | Slot kind, 1 by default. Kind 0 is the only kind that blends toward its new value |
| Active | integer | The slot is in use when this is nonzero |
| Entry | reference | The slot's light entry (the 0x50-byte entry layout of SPEC-0028 section 2) |
| Weight | float | The entry's strength, or the type 3 falloff (see Character lights) |
| Vector | 4 floats | The stored light colour (16 bytes) |
| Second value | integer | Used by kind-0 slots only; the blend applies when it is 0 |

Light block (sent with a lit draw and held by the vertex program until the next block; eight vectors of 4 floats):

| Field | Count | Meaning |
| --- | --- | --- |
| Directions, stored transposed | 3 | With T_k = −normalize(D_k) for slots 1 to 3 (D_k the light's travel direction in the drawn model's space): vector 1 = (T_1.x, T_2.x, T_3.x, f), vector 2 = (T_1.y, T_2.y, T_3.y, 0), vector 3 = (T_1.z, T_2.z, T_3.z, 0). f is the brightness factor (see Brightness factor), read only by program 6 |
| Fourth row | 1 | (0, 0, 0, 1) (no bias term) |
| Light colours | 3 | C_1 to C_3, the stored colours of slots 1 to 3 |
| Ambient | 1 | A, the stored colour of slot 0 |

The diffuse terms d_k = T_k · n come out of the transposed block unchanged.

Colour-scale record: a vector of 4 floats and a lane mask (1 red, 2 green, 4 blue, 8 alpha). Sending it replaces only the masked lanes of the draw colour scale s.

Per-part visibility result (per model part): a signed value with three meanings: -1 culled, 0 inside, 1 clip (see Model parts).

Colour sheet state (run-time, one set per colour sheet node):

| Field | Range | Meaning |
| --- | --- | --- |
| Hue index H | 0 to 180 by pad steps | The hue is 2 H degrees |
| Saturation index S | 0 to 63 by pad steps | The saturation byte is 4 S + 3 (3 to 255) |
| Value index V | 0 to 63 by pad steps | The value byte is 4 V (0 to 252) |

Colour sheet cell (quad vertex): position (screen units), hue, saturation, value, and a fixed alpha byte of 128. Each vertex carries three colour bytes and the alpha byte.

View block (run-time, per object): four rows of a view matrix built from the camera, and two vectors used by the projection. Its width and height come from the view record (see View and projection build).

Extent box (input to the bounding radius, see Bounding radius of a box): four 32-bit float values in this order: low X, high X, low Z, high Z. The rule has no Y extent.

## Behavior

### Per-frame draw order (High; step 1 Medium)

In one frame, after the updates, draws are queued to the GPU in this order:

1. **HUD 3D pass (first, before the sky and the worlds).** When the game-globals request word (values 1 or 2; cleared every frame; the only setters are the three item-screen tab pages, which write 1 on every pass of the Items and Charms tabs and 2 on every pass of the Quest-items tab, so the word is nonzero exactly while the item screen is open; Medium-High) is 1 or 2, the atmosphere colour and render scalar are applied (SPEC-0028 section 11); otherwise (the word is 0, which is every in-play frame) the pass draws the HUD spinning coin (HUD coin model and camera-facing quads) and nothing else; there is no separate backdrop draw (the earlier reading of this branch as a front-end backdrop was wrong; High). While the item screen is open the coin is therefore not drawn. The pass's list is then flushed inside a fade wrap at level 128 (data word 0x2D7614 = 128.0). [Medium]
2. **Sky dome**, while the data word 0x2D7528 is nonzero (see Sky domes).
3. **Worlds**: the second world (when the current world is not itself the second-world slot, the second world is loaded and the pairing test passes; see Object lists and the special object), then the current world: its level display tree (with the culling of Culling), its instanced meshes (type 20), its placed objects and its world effect lists.
   - **Instanced meshes (type 20; High for the order and the per-instance rules, Medium for the queueing).** Each world draws the instanced meshes right after its level display tree and before its placed objects, in the same call, so the second world and the current world each draw their own. A world holds at most two type-20 chunks (SPEC-0006). Records are drawn in table order and instances in array order. For each instance (record layout in SPEC-0004, Instanced meshes): (1) the zone-visibility query of SPEC-0006 (Binary grids) with the instance's zone column and key −1 (the active block: the main loop selects block 0 for the current world and the neighbour's block for the second world); a failed query skips the instance, and the query passes when no grid is loaded. (2) d is the straight-line (3D) distance from the eye point of the world draw to the instance translation; the comparison is made on squared values. The instance is skipped when d is at or above the record's draw distance D1. (3) Opacity is 1.0 unless the record's fade length F is above 0 and d is above D1 − F, in which case it is (D1 − d) / F (F is 0 in all 67 records on disc, so every drawn instance is fully opaque). (4) The mesh is the record's near mesh, or its far mesh when the record has one and d is at or above D2 (no record on disc has a far mesh). (5) The mesh's node tree is queued with the instance matrix as its model-to-world matrix, through the same material buckets as the level tree (so its type-4 and distance nodes still apply and its materials are unlit, like all level geometry). Test vector: a record with D1 = 4000 and an instance 100 units from the eye draws at opacity 1.0 when its zone column passes; at 4,000.0 units or more it is skipped.
4. **Held items and character attachments**: the focus hero's items, other characters' held items, world items and the item showcase (see Held items and character attachments).
5. **World draw list flush.**
6. **Character light-flash pass (hit flash)** (data word 0x2D7534): a hit on the hero, or a hit through the enemy class's hit method, starts a 14-frame two-stage ramp on the hit character's render object (4 frames, then 10). While the ramp runs, the pass sets that render object's colour-override slot 1 to (1.5, 0.5, 0.5, 1.0), a reddish tint. While slot 1 is on, the part draw uses its vector, in place of the slot colour of each part, as the draw colour scale of every part of that character, so the whole character shows the tint for the length of the ramp (SPEC-0040, Behavior 26, item 9; Question 208). The ramp's own keys and colour bytes (SPEC-0034 section 32) are not read by any draw. When the data word 0x2D7600 (0 on disc) is nonzero, the flash is forced on the hero. The ramp's keys, colours and stage lengths are in SPEC-0034 section 32. The pass draws nothing itself. [Medium-High for the ramp; High that the part draw reads slot 1; Medium-High that the tint shows on screen]
   - **Colour-override slots** (High for the rule; Medium for the pulse shape on screen). Each character render object has two override slots, each holding an active flag, a target vector (r, g, b, a), an RGB-mask flag, an alpha-mask flag, a frame counter and a mode. Clearing a slot sets it inactive with the default colour-scale vector. Setting a slot that is not already active starts its counter at 0 in mode 1 and at 30 in mode 0. The blend below reads slot 0 only; the part draw reads slot 1 as the object colour (see the hit flash above). Each draw of the object, while slot 0 is active: if the mode is 1 the counter rises by 1; with c = counter mod 60, the weight is w = c / 30 for c up to 30 and w = (60 − c) / 30 above; for each of the object's 15 colour-scale records of the current buffer, the lane mask is set from the two mask flags and the record's vector becomes (1 − w) × its vector + w × the target. Mode 1 therefore pulses with a 60-frame period (full target at c = 30, none at c = 0); mode 0 holds w = 1, a steady tint. Users: the status display (SPEC-0034, section 5.4) sets slot 0 every tick in mode 1 with the status colour, each channel cut to 1.0, and clears it when the shown status changes or when all three channels are 1.0 (so fire shows no tint); the stun start sets slot 0 in mode 0 (a steady tint). Confirmed 2026-11-11 (High): the setter is used by every status tint step and by the stun start; the stun start's colour is (0, 0.5, 1.7, 1.0) with all three flags set, and the tint steps set the quad of the per-channel lows (alpha 1.0) with flags (1, 1, 1), or clear the slot when all three lows are exactly 1.0. Test vectors: mode 1, counter 0, 15, 30, 45, 60 gives w = 0, 0.5, 1, 0.5, 0; mode 0 gives w = 1 on every frame.
7. **Shadow volumes** (see Character shadow volumes).
8. **Characters and effects**: the character pass (the sorted list of Frame sequence of the character pass: the two petpet lists, current world, second world, petpet, hero), then the effects pass (data word 0x2D7540; SPEC-0034). While the hero's fade is on, the effects pass comes first and the character pass second.
9. **3D book**, when it is open (SPEC-0035; see 3D book open angle).
10. **Frame copy with the atmosphere colour tint** (data word 0x2D752C; SPEC-0028; from the game code, not run in the original). Two things happen. (a) Earlier in the frame, vector A (the first colour block of the zone record) is drawn as a full-screen source-alpha sprite onto a separate copy of the frame, weight A alpha / 128 (result = Cd + (A rgb - Cd) x alpha_A / 128 in the copy). (b) At the end of the frame (after the draw chain is built) a composite pass runs, in its mode 3, in two draws. Draw 1 draws that frame copy back over the frame as a full-screen textured sprite with the blend (Cd - Cs) x Ad + Cs, where Cs is the copy, Cd the frame colour and Ad the frame's destination alpha with 128 meaning 1.0 (not 255, which would be about 2.0, overdriven); the depth test is GREATER against the far value 0x7FFF with no depth write, so it changes only pixels whose stored depth is far or empty. Opaque world pixels hold Ad = 128 (the fragment alpha, vertex alpha times texel alpha / 128, is written to the 32-bit frame) and are left unchanged; the pass therefore shows only where Ad is below 128: the far dome-horizon band whose vertex alpha ramps to 0, translucent fringes and empty background (Medium-High). Draw 2 runs only when vector B's alpha byte is nonzero: a full-screen untextured sprite coloured with vector B's RGB, drawn with plain source alpha, result = Cd + (B rgb - Cd) x alpha_B / 128, in the frame buffer's gamma-encoded byte space, depth test ALWAYS, no depth write (High). The tint covers the HUD and letterbox bars too (it is a late full-screen pass; which buffer it targets relative to the draw chain's own render is Medium-Low). The overlay strength is therefore alpha_B / 128, not 1 - Ad; there is no blend factor b on it (b weights only the sky dome's layer D). Gated zones have alpha_B = 28 (weight 0.21875); ungated records keep their disc alpha (Meridell base: 10, weight 0.078125). Test vectors (the GS computes ((Cs - Cd) x As) >> 7 + Cd with an arithmetic shift; Medium on the rounding): As = 28, B = (35, 0, 55), Cd = (200, 150, 100) gives (163, 117, 90); As = 10, B = (255, 214, 128), Cd = (200, 150, 100) gives (204, 155, 102). A pipeline that blends in linear light should blend the gamma-encoded values, or use the per-pixel equivalent (a fixed linear alpha of 0.78 is far too strong: the tint is about 22 percent, or 8 percent ungated, of the way to the tint colour in encoded space). This pass does not use the fade quantizer. The screen fade and bars follow in step 11, and the HUD is drawn last in step 12.
11. **Screen fade and letterbox bars** (one object, SPEC-0052).
12. **HUD and menus** (SPEC-0023, SPEC-0035), on top of the fade and the bars.

Every gate word named above is 1 on disc, and nothing writes these words at run time (see Where it lives on the disc).

### Placed-object draw frames (High for the data flow; Medium for the visible effect)

Each placed object (SPEC-0028, section 7) that passes its draw gates is queued as one draw of its mesh child with two frame inputs: a 4 × 4 matrix and a view point. Both also reach the vertex program as its camera block (SPEC-0003, UV modes: the eye point E and the matrix M of the environment-map mode).

- **Matrix, the environment frame.** It is the object's current block: the group of four rows at +0x100 (three rotation rows already multiplied by the scale words at +0x140, and the translation row P at +0x130; SPEC-0028, section 7.1). It is the model-to-world matrix of the object. The draw record keeps the product of this matrix and the projection set. The environment-map mode uses only its three rotation rows, with no translation (R' = M3 R in SPEC-0003), so the reflection of a model-space normal is turned into world space by the object's own rotation and scale, then normalised.
- **View point, the eye frame.** It is a 4-vector kept in the object at +0x170 and passed as the draw's view point. Each time the object's animation update runs it recomputes it: E = (P + mesh corner) − C. Here P is the object's position for that update (root accumulation plus base, plus the script offset when the mode byte is 1 or 3; SPEC-0028, section 7.1, step 5), the mesh corner is the corner vector at offset 0x10 of the mesh child, added without rotation exactly as for the reference centre at +0x180, and C is the camera's world position (the translation row of the camera matrix). An object with no mesh child uses P − C. The world update stores C before it updates the placed objects, and the world draw stores the same vector again, so every update and draw of a tick uses the current camera position. An object whose update has not run yet, or never runs (an object still in its disc state, or one no script animates), keeps its disc value (0, 0, 0, 1), so its E is the origin.
- After the last placed object of a world is queued, the draw context's view point is reset to C. Items queued afterwards see the camera position unless they set their own; the hero's draw sets camera position minus the hero's position (Open question 92).
- **Quirk (Medium).** E is a world-space offset from the camera to the object's corner, while the vertex positions of the mesh are in the object's own coordinates (the matrix carries P). The eye vector in the model frame would be C − P; E has the opposite sign for that term and adds the corner. For an object away from the origin the reflection is therefore not the true world reflection. Reproduce the formula as written. It matters only for placed objects whose materials use the environment-map UV mode.

Test vector (synthetic values, real formulas): an object with the identity rotation and scale 1 at P = (100, 0, 0), mesh corner (−2, 0, 0), camera C = (100, 5, 20). Then E = (98, 0, 0) − C = (−2, −5, −20). A vertex at (0, 0, 0) with normal (0, 1, 0): D = vertex − E = (2, 5, 20), D · N = 5, R = D − 2 (D · N) N = (2, −5, 20), M the identity so R' = R, |R'| = 20.712, R^ = (0.0966, −0.2414, 0.9656), u = 0.5 (1 − 0.9656) = 0.0172 and v = 0.5 (1 − 0.2414) = 0.3793. A static object that was never updated has E = (0, 0, 0, 1): a vertex at (1, 0, 0) with the same normal gives D = (1, 0, 0), D · N = 0, R = D, and u = v = 0.5.

### Held items and character attachments (draw step 4; High unless marked)

Steps 3 and 4 run as one pass. In queue order it does:

1. The focus hero's items (below).
2. The world draw of step 3.
3. The held items of the characters of the two petpet lists.
4. The held items of the characters of the current world's character table, then of the second world's table when it is loaded.
5. A fixed array of ten further records: the enemy projectile pool of SPEC-0029, drawn in slot order (Open question 87).
6. World items.
7. The item showcase. Its spin, offsets, modes and pose animations are described in SPEC-0021 (item showcase); this spec only places it in the pass.

Everything queued here is drawn at the world draw list flush of step 5. All held-item models draw with resource 7's material list, except world items, which take their material list from the item model registry (SPEC-0021). Models come from the item model registry and the item display slots (SPEC-0021); their registry ids are in SPEC-0004 (Item model registry key). W_k are the attach-point matrices of the character (SPEC-0004, Attach-point matrices), with axes X_k, Y_k, Z_k and origin T_k.

**Shared flags.** The weapon-drawn flag and the shield-raised flag live on the party inventory, so the two heroes share them. Their writers (attacks, the guard, hits, interactions, state changes, the showcase) are in SPEC-0040.

**Focus hero.** Let H be the focus hero, with hero index 0 for Tor and 1 for Roberta. None of H's items (shield, weapon, held entry) is drawn when H's render-active word is not 1, or when H's body entry holds item 126 (Tattered Cloak). Otherwise:

1. **Shield.** The shield entry's model is set to the shield slot's geometry.
   - Shield-raised flag set: the shield matrix S = W_5 (on the arm).
   - Flag clear: S has the axes (−X_8, Z_8, Y_8) and the origin T_8 (stowed on the back). When the shield entry is not empty and the shield model is loaded, the three axes are scaled by f = 1.6 / |bx − sx|, where bx is the minimum X of the model's root-node bounding box and sx its box size in X (SPEC-0004, Node tree).
   - The shield is drawn at S only when the hero index is not 1, so Roberta's shield is never drawn.
2. **Weapon.** The weapon entry's model is set to the weapon slot's geometry.
   - Weapon-drawn flag set: the weapon matrix M = W_0 (in the hand).
   - Flag clear: M has the axes (Y_6, Z_6, X_6) and the origin T_6 (sheathed). When the weapon entry is not empty and the weapon model is loaded, the axes are scaled by g = min(2.6 / |bz − sz|, 0.6), where bz is the root-node box minimum Z and sz its box size in Z. The 2.6 and 0.6 are the data words 0x2DD8AC and 0x2DD8B0.
   - The weapon is drawn at M when the hero index is 0 or the weapon-drawn flag is set. So Tor's sword is always drawn (in the hand or sheathed), and Roberta's wand is drawn only while drawn, in her hand.
3. **Held entry.** Only while the weapon-drawn flag is 0, and only when the current held entry e (SPEC-0021, the holder's current held entry) is not empty:
   - The held matrix Q has the axes (Z_0, Y_0, −X_0) and the origin T_0 (the hand).
   - e is drawn at Q when e's count is above 0 and its category is not 5 (the body family). A further test for hero action state 15 can never hold, because no code sets that state (state 15 is an empty route; High), and is left out together with one other condition that can never hold.
   - When e is item 36 (Rope Gun), its model is resource 575, and a separate rope-manager object is drawn with it (Medium-High). The manager maintains a pool of 200 segment records, each using model-registry key 159 rather than the held prop's model. During outbound and reel motion, active segment transforms advance in one-unit steps along the rope axis, with integer indices below the measured span length. The record at the integer part of the span is positioned like the others, but its first basis row (the one that is also the step along the rope) is multiplied by the fractional remainder, so that last segment is drawn shortened along the model's X axis to the leftover length (Medium-High); the rope therefore has the exact measured length with no gap or overshoot. A record whose index is not below the span is switched off. The manager does not select a separate endpoint model. Its update runs through the held-object render path once per qualifying render traversal; the manager's own state machine (outbound, attached, reel) sets how many segments are active on each pass, so no fixed ratio to hero ticks exists. Key 159 is the boot model of resource 7 with header value 61 (98 + 61; SPEC-0004, Item model registry key). It is a five-sided prism one unit long: 22 vertices in one batch, the cross-section ring of radius about 0.05 (Y from −0.053 to 0.038, Z from −0.049 to 0.047) at X = 0 and X = 1, flat end caps, vertex programme 1 (lit) and a gold and tan striped 64 by 64 texture (material group 6 of resource 7). Its long axis is local X. With the segments placed at one-unit steps along the rope axis, the rope reads as a continuous thin gold cord about 0.1 across. High that the model's X axis follows the rope axis: each segment's matrix copies the rope basis, whose first row is a unit vector (normalised whenever the manager is outbound, attached or reeling; in the idle state it is the negated first row of the hero's own matrix block), and segment k sits at the start point plus k times that same row, so the step is exactly one unit and the model's X axis lies along the rope. The other two rows are copied from the basis, so the roll of the prism about the rope comes from the rope basis (the hero's own up and side rows when idle). Draw segment k at the hero-side end plus k units along the rope axis, with the model's X axis along the rope.
   - Sound cues are requested by the rope gameplay manager, not by the held-item draw: cue 0xB7 when the rope launches, cue 0xB8 when the travelling rope reaches its attached state, and cue 0xB9 during reeling while the remaining distance is above 3.0 units. All three requests use the same main sound slot, whose command bank is resource 347; the records are 183, 184 and 185, respectively. Cue 183 has five direct sample-voice commands referencing sample ids 3088, 3088, 3424, 9104 and 320; cue 184 has four referencing 2800, 3216, 1920 and 6784; cue 185 has one direct sample reference, id 6032, plus two alternate-voice commands with no established sample-pool offset. These bank-local references identify records but have no names or confirmed recognizable audio identities.
   - The hero's held entry only ever holds item 36: it is set to 36 in hero state 27, and every other write clears it (High).
4. **Hero-item effect matrices** (High for the matrix copies, the Tor gate and the glow; Medium for the look): each focus-hero item-render pass copies S and M into two matrix slots of the shared auxiliary effect object, whether or not the shield and weapon are drawn. The effect is Tor's weapon and shield glow, drawn from these matrices. When Tor is the selected hero and a Mote is equipped in word 2 (the weapon Mote) or word 1 (the shield Mote) of his equipment (SPEC-0021, Equipment slots), a glow model is attached at the saved weapon or shield transform. The weapon glow is common model 299 plus Tor's weapon index (the weapon item id minus 98, so 299 to 306); the shield glow is common model 307 plus the shield index (the shield item id minus 106, so 307 to 311). The glow mesh's CPU animator copies the selected Mote's RGB into every vertex colour; it replaces each vertex alpha with a pulse that ranges from 64 to 250. The preset alpha is initialized to 64 before the pulse updates it. The pulse advances per eligible mesh draw, not once per game tick (SPEC-0003, UV modes). The vertex program is unlit; the texture is modulated by the vertex colour, and the material uses additive blending (source colour × source alpha + destination colour). The RGB values come from the 14-row Mote colour table indexed by Mote id minus 160 (red for 160 and 161, blue for 162 and 163, green for 164 and 165, cyan for 166 and 167, magenta for 168 and 169, yellow for 170 and 171, grey (120, 120, 120) for 172 and white for 173). The row's fourth byte is unused for the glow; its initial alpha comes from a separate value of 64, then the mesh pulse changes it. The glow is started when an equipment word changes or the hero is switched or reset, and cleared when the word becomes -1. Roberta has no glow: every start and update routine returns unless the hero index is 0, and her copied matrices are not used. The weapon glow has no weapon-drawn test (R-0227; Medium-High): its draw needs only that the glow was started, the hero is Tor and the weapon entry is not empty, and it is drawn at the weapon matrix M, which is the sheathed pose (with the scale above) while the flag is clear. So the glow model stays on the sword on Tor's back; only the Mote weapon line and tip light of SPEC-0034 section 3.17 stop while the sword is sheathed.

The two rescale formulas use |min − size|, the distance between the box minimum and the box size, not the box size itself. This is an original quirk with a visible effect on the stowed and sheathed sizes; reproduce it as stated.

**Other characters** (petpet lists and world tables). For each active character with a render object:

- **Stolen item** (a thief): while its carried-item flag is set and its countdown is above 0, the stolen item's registry model is drawn at that character's Q with its three axes scaled by 0.45, and the countdown falls by 1 per draw (also when the item has no registry model). On a draw where the flag is set and the countdown is 0 or less, the flag clears and nothing is drawn. A steal sets the countdown to 3 seconds of frames (180 at 60 frames per second), so the item shows in the thief's hand for 3 s.
- **Held entry**: when the character's own holder has a current held entry whose item id has a registry model, it is drawn at that character's Q. There are no other conditions, no scale, and no weapon, shield or sheath logic for these characters. A non-hero character gets a held entry only at its set-up: when the 16-bit field at +0xAE of its placement record (SPEC-0006, kind 3 records) is not −1, that item is added to the holder and selected. The field is −1 in all 3,036 placement records, so on disc no petpet or world character shows a held entry; the thief's stolen item above is separate (High). The weapon model of an enemy is not this holder entry: it is the slot 0 attachment of its model set, drawn by the second walk below. Confirmed 2026-11-09 (High) against the held-attachment routine: the entry is drawn at attach point 0 with the axes (Z_0, Y_0, −X_0) in the same axis swap as the heroes, origin T_0 and no scale; the draw anchor's offset is the view point passed to the draw minus the character's drawn position (the render object's position at +0x18B0), as for the hero; an id with no registry range draws nothing; the shared material list (game material list group 3) is bound before the frame is submitted; the routine runs per active slot from the last to the first and returns the sum of the burst-emitter results it triggers.

**Second walk and attachment slots (High for the walk, its gates and its draws; Medium-High for the slot roles).** A character's model set holds up to eleven optional attachment models, numbered 0 to 10. A set gets slot k from a child model whose role id is 10 + k (SPEC-0004). The first walk above draws only the item holder's entries. After the world draw flush the pass walks every world table a second time, from the highest slot of the table down to slot 0, and draws directly (not queued). A slot is skipped unless it is in use and its character passes all of these: the character is active; it is not in the hidden or dead state; its pause word (+0xE3C) is 0; its character flag 42 is clear; its draw-enabled word (+0xC8C) is nonzero; and its render object has a material list. The character's own material list is selected for its draws. For one such character the walk draws, in this order:

1. **Slot 0 model, or the thrown model.** If the character's thrown-model word (+0xD10) is 0 and its model set has a slot 0 model, that model is drawn with the hand matrix Q (axes Z_0, Y_0, −X_0, origin T_0, as for the hero's held entry) and with the character's own light state. If the thrown-model word is nonzero, that model is drawn instead at the 64-byte frame stored in the character (+0xCD0) with no light context, and the word is cleared at once, so a thrown weapon is drawn for one frame and must be re-armed every tick. The Black Knight's flying sword and the Werelupe King's boomerang set the word every tick while they fly, each with its slot 0 model. So a thrown weapon is not drawn in the hand at the same time. No other character sets it.
2. **Slot 6 model.** When the model set has one, it is drawn at W_5 (the attach point the hero uses for its shield arm), unscaled, with no light context and with no other condition.
3. **Slot 1 model.** Drawn only when the animation event word has bit 0x1000 on and the character's current attack row has a projectile type (visual kind) below 5 (SPEC-0029, Behavior 9), and the model set has a slot 1 model. The matrix is one of W_0 to W_4, chosen by event bits of the current clip: bit 1 or none of the bits below selects W_0, bit 2 selects W_1, bit 4 selects W_2, bit 8 selects W_3 and bit 0x10 selects W_4 (tested in that order). No light context.

Roles of the slots (the code that reads them; Medium-High):
- **0**: the weapon model of a character: in the hand (step 1), and the model of a thrown weapon (Black Knight sword, Werelupe King boomerang).
- **1**: the model of the first shot of a ranged attack. The Gelert Assassin's dagger takes this model, and the common ranged enemies take it for shot 0 of a volley and for any shot from number 5 on. It is also shown on the attach point while the throw animation holds event bit 0x1000 (step 3).
- **6**: a part fixed to attach point 5 (step 2), such as a shield on the arm.
- **7, 8, 9, 10**: the models of shots 1, 2, 3 and 4 of a common ranged enemy's volley (the volley's shot counter picks the slot). Only used while the attack's projectile type is below 5; otherwise the shot has no model.
- **2, 3, 4, 5**: no model set on disc has them and no code reads them.
On disc, of 1,355 character model sets, 1,103 have no attachment model; 140 have slot 1 only; 86 have slot 0 only; 18 have slots 0 and 6; 5 have slots 0 and 1; and 3 have slots 1, 7, 8, 9 and 10 (the five-shot shooters).

Test vectors: a table with 4 slots in use where slot 1's character is inactive: the walk visits slots 3, 2, 0 in that order. A character with slot 0 model, thrown-model word 0: slot 0 at Q with its light state. The same character with the thrown-model word set: the thrown model at the stored frame with no light; the word reads 0 afterwards, and the next walk draws the hand model again. A shooter with event bit 0x1000 on, projectile type 4, event bit 4 on in the clip: its slot 1 model is drawn at W_2. With projectile type 5, nothing of slot 1 is drawn.

**World items.** World potions draw the model of the world-item slot. There is only one world-item slot, so only one potion model is resident at a time: spawning a world potion of a different kind unloads the first kind's model, and world potions of the first kind have no model until it is loaded again (see Edge cases; slot rules in SPEC-0021).

### Frame sequence of the character pass (High)

This is the character pass, step 8 of Per-frame draw order. Each frame, after the game state update, in this order:

0. The per-frame update pass runs over the object lists (Medium for its place in the order).
1. Clear the frame's draw list (count to zero, no records).
2. Open the render state for the frame (platform side, SPEC-0008).
3. Submit the characters of the first object list, then those of the second object list (the two petpet asset slots). Each list is walked from its highest character down to character zero.
4. Submit the scene sub-list (the current world's character slots), then the second sub-list (the neighbour world's character slots) when it is loaded and the pairing test passes.
5. Submit the special object (the active petpet) with the far flag clear (near set) and no fade record.
6. Submit the main character (the focus hero, SPEC-0040) with the far flag clear (near set) and its fade record. Its screen bounds are updated first (see Main-character bounds).
7. Flush: draw every record in list order.
8. Close the render state.

Steps 5 and 6 run only while the data word 0x2D7518 is nonzero (1 on disc). Nothing is drawn during steps 3 to 6. All draws happen in step 7. The sky dome is not a draw record: it is added to the frame's material draw list right after that list is reset, before the worlds, the characters and the objects above (see Sky domes). The colour sheet is a display-tree node: it is drawn when the display tree visits it (see Colour sheet), and its quads are not draw records of this list (Open question 24).

### Object lists and the special object (High)

- **First and second object lists**: the two petpet asset slots (the first and the second petpet asset slot). Each is an ordinary character slot record (SPEC-0030 section 4): an active character count, character references and one active flag per character. Each list is walked from its highest character down to character 0; a character is used only when its active flag is 1.
- **Scene sub-list**: the current world's character slot table. The table's main record, if there is one, is submitted first; then each slot record in table order whose load-result word is nonzero.
- **Second sub-list**: the second (neighbour) world kept loaded during streaming, with the same table layout. It is submitted only when it is loaded and the pairing test passes (SPEC-0006):
  1. the current world pack is not one of 784, 1398, 1371, 1397, 1414, 1139, 1176, 1163, 1171, 1299, 1327, 990, 1369, 1300, 989, 1056 (data words at 0x31EEA0);
  2. the current pack is a transition (the 51-pack set of SPEC-0006), or the current world's area slot is in the 92-slot swap set (SPEC-0006, Neighbour-world test, which lists the members);
  3. the pair (current pack, second pack) is not one of (797, 758), (797, 753), (1392, 1370), (991, 699), in either order (data words at 0x31EE80). The draw callers pass the second world's pack, so these pairs do stop the second world from being drawn (they never block a world swap; SPEC-0006).
- **Special object**: the active petpet, which is character 0 of the pet slot that loaded it. It is also an active character of that slot, so the original submits the petpet twice per frame: once from its slot (with its own far flag) and once as the special object, with no fade record and the far flag clear. Both copies are drawn (original quirk; see Edge cases).
- **Main character**: the focus hero (SPEC-0040), submitted last, with the far flag clear and its fade record.

Every object in these lists is a character render object (draw class B, see Draw classes). Placed world objects are not in these lists; they are drawn by the world draw (Per-frame draw order, step 3).

### Object submission and culling (High)

For each slot of an object list:

1. The slot is used only when it is marked active (its flag equals 1).
2. The object must be drawable (see Drawable test below).
3. Compute the squared distance from the view point to the object's position.
4. Set the far flag when the squared distance is greater than 2500 (50 units squared). A squared distance of exactly 2500 does not set it.
5. Submit the object with its far flag. Submission drops the object when its squared distance from the view point is 122500 (350 units squared) or more. Otherwise a draw record is appended.

The submit cull and the far flag use the full 3D squared distance (x, y and z) from the view point to the object's render position (High). The view point is the translation of the camera record's matrix (the base point of SPEC-0027), read in the frame set-up after the camera update. It is taken from the camera record itself, not from the shaken render copy, so a camera shake moves the picture but not the cull and far distances (High). In the follow camera it is the eye at the end of the tick (Medium-High). Test vector: camera position (0, 5, -10) with a shake that moves the render copy 0.3 units: an object at distance 349.9 from (0, 5, -10) is still submitted on every tick of the shake.

Designer properties (High). The monster table carries two 16-bit designer fields named draw distance (row offset 0x32) and low-resolution distance (row offset 0x34). Their editor range ends at 2500; 2500 is a range bound, not a default, and the default is 0. No code reads either field: the game reads a row only through its field accessors (hit points, radius, height, behaviour class, timers and the like) and there is no accessor for these two, the character keeps only a pointer to its row and never copies the row. On disc the draw distance is 300 in 153 of the 259 rows, 100 in 50, 110 in 38, 200 in 11 and 0 in 7; the low-resolution distance is 200 in 148, 50 in 94, 100 in 9, 75 in 1 and 0 in 7. None exceeds 2500, and the draw distance is never below the low-resolution distance. Neither changes what is drawn, so a clean side must not use them to hide or simplify a character. The clean side should use 2500 (50 units squared) for the far threshold and 350 units for the submit cull.

Drawable test (High for the tests; meanings per bullet). An object is drawable only when all of the following hold:

- its object record exists and its render context is set;
- its cell flag is set. This is the character's own cell-enabled word (+0x18C), which the region update recomputes on every tick for each active slot. It is 1 when the character has no zone record, and otherwise it is the result of the zone cell test (SPEC-0006). A character whose cell is disabled for the current zone is neither updated nor drawn. (High for the write; Medium for the meaning of the cell test.)
- its draw-state test passes (see below);
- its despawn flag (+0xE3C, SPEC-0017) is zero. The flag is set to 1 when a dead spider (state 16) has finished its death animation 58, and cleared to 0 when its valve launches it again; it is also cleared when the character object is reset. (High.)
- its base actor state is not 42. The excluded value is the base actor state of the character (SPEC-0053 calls it the action state). Base state 42 is the parked state. Objects are created and reset in 42, and every bind (streamed, forced, and spawn by key) sets base state 1 before the character is first drawn. A released character that is not dead is set to 42, and so is a not-dead object in an inactive slot, on every tick. Inactive slots are already skipped by the slot-active test, so in play the rule hides only bound characters whose AI has put them into 42. The only such characters are the Gelert Assassin's waiting mirror images (SPEC-0013, 4.10). (High.)
- the shared character-visibility predicate (SPEC-0029 Behavior 2b, hidden test) does not mark it hidden. A common enemy whose monster row has Display in default 0 is therefore not drawn while its class state equals its spawn record's default-state word, or while it is in state 21 and animation 82 is not playing, unless the active hero wears the Tattered Cloak or the invisibility-off command has run for it. A common enemy whose row has Display in default non-zero is hidden (not drawn) while its emerge sub-state is 0, its spawn record's cue word (offset 0xC4) is 0 and its current animation is the hide pick (id 9, or 128 to 131 by the emerge selector), and in class state 14 at phase 1. Examples: the Swamp Tentacle, the sewer Tentacle, the Slorg and the Giant Slorg in default state 8. (High.)

Characters hidden in play by these tests (High unless marked):

| Characters | When hidden | Hidden by |
| --- | --- | --- |
| Gelert Assassin mirror images (block slots 1 to 4) | From the fight's set-up until the boss's mirror attack wakes them, and again after each hit on them | Base state 42 |
| Any not-dead object in an inactive slot | Always (the slot-active test already excludes it) | Slot-active test; base state 42 |
| Townsfolk, common enemies, bosses, summons | Never by base state 42 (they are in state 1 from the bind onward); common enemies with Display in default 0 (for example Minion Grunt, Archer, Brute, Warrior) are hidden by the visibility predicate in their default state and in state 21 before animation 82; common enemies with Display in default 1 are hidden by the predicate while waiting in state 8 with cue word 0, and in state 14 at phase 1 | Visibility predicate |
| Spiders whose death animation 58 has finished | Until their valve launches them again; their slot stays active | Despawn flag |
| Characters of a non-streaming block standing in a cell disabled for the current zone (Medium) | While the cell stays disabled; in a streaming block such a character is released instead | Cell flag |

Draw-state test (Medium for the branches, Low for the meanings). It depends on the object's kind and on whether it has a model:

- With no model: a kind check. One kind value compares against a second value, one branch also needs a sub-flag, and one branch treats a hero-type object specially.
- With a model and no "hidden" state word: a state check against a per-object id. One branch allows the draw when a second test passes. Another branch refuses the draw when a model-group flag is set.

What the values are (Medium-High): a character with a nonzero word at +0xE60 is never drawn. In the no-model branch the "kind value" is the character's current state number and the second value is the state number held in its spawn record at +0x84 (its display state); the character is drawn when the two are equal, or when the state is 21 and the last requested animation is not 82 (the sub-flag), and in either case the draw is refused while the hero's current clip entry has type 126 (the hero-type exception). In the with-model branch the per-object id is 9 when the spawn record's selector word (+0xC8) is 0 and 0x80, 0x81, 0x82 or 0x83 for selector values 1 to 4, and it is compared with the last requested animation number. With the hidden-state word (+0xE3C) clear: when the model-group flag (+0xBC0) is clear, the character is refused if the spawn record's word at +0xC4 is nonzero and otherwise drawn only while the last requested animation equals that id; when the flag is set, it is drawn only in state 14 and only while the word at +0xC04 is 1. With the hidden-state word set the test passes (the separate hidden-state test then stops the draw).

### Draw classes (High for the dispatch; High for the absence of a reader of the level-of-detail flag)

Each render object points to a method table. The flush calls one method from that table for each record, and the method draws the object. Two tables exist:

- Class A: the model-part draw driver. It runs the per-part visibility test, selects the clip or inside state, sets up the light context, uploads the skin matrices once, walks the model's node tree, and draws each part with its material group. This is the class described in Model parts below. Class A exists only as the base of class B: no render object keeps class A after construction.
- Class B: the derived class. Every character render object uses it: the three hero render records and every slot character.

Class B draw, in order:

1. If the object's skip-once flag is set, clear it and draw nothing this frame (the Cloud Beast air-flow states set it on two characters).
2. Level-of-detail flag update (below).
3. The class A model-part draw.

Level-of-detail flag. Let h = (v_x − p_x)² + (v_z − p_z)², the horizontal squared distance from the view point v to the render object's position p (x and z only; y is ignored). The candidate flag is 1 when h ≥ T², else 0. The stored flag changes to the candidate only when the candidate differs from it and |h − T²| > 25.0 (a squared-distance margin). T = 150 units for the three hero render records; T = 0 for every other render object, so their flag becomes 1 once h > 25. Nothing reads the flag (High; Open question 80). No routine of the boot program and no level module loads the flag's word, by its own offset or through a wider load or copy of the render object: the render objects are fixed arrays that are never block-copied, and the only other code that touches the word is this update. The flag therefore changes nothing that is drawn, and distance detail switching in the game is only the 50-unit far set (Far state at flush) and the 350-unit submit cull (Object submission and culling). The clean side leaves the flag out.

### Draw list capacity (High)

The frame's draw list holds at most 18 records. In the original, a 19th or later insert only prints a debug message: the record is not added, the count stays 18 and the frame goes on (nothing stops or is reset). Test vector: 20 submissions in one frame keep the first 18 (sorted as usual), print two messages and draw 18 records.

Guidance for the clean side: do not limit the number of draws per frame. The 18-record size is an original fixed buffer. A clean implementation should keep every collected object and draw all of them, using whatever dynamic storage it likes. Reproducing the drop and its message is not required.

### Far state at flush (High for the mechanism; Medium-High for the vertex-program effect)

The far flag does not switch the model or the light set. It selects one of two global vertex-program state sets used when the record is drawn:

- far flag clear: the near set (set 0);
- far flag set: the far set (set 1).

Each set is a small state record sent to the vertex program. It is re-sent only when the selected index changes from the last one used. The draw method itself does not test the flag.

The vertex program keeps a state word with three bits:

- clip (value 1): set per model part from the part classifier (inside 0, clip 1); it selects the clipping path for the part's batches (see Per-triangle clipping in the vertex program);
- fade (value 2): from a separate 257-entry table, where entry k sets the bit for k > 0 and stores the fade level k/256 (0 to 1). This bit is dead (High): the vertex program stores the bit and its level but no program path reads either, and the selector of the 257-entry table has no caller in the boot program or in the level modules. The table is built at start-up only. The clean side leaves it out;
- far (value 4): from the far flag.

Only the clip bit and the far bit have effects.

Effect of the far bit (High). In the skinned vertex programs (2 and 6), the far bit selects rigid skinning: each vertex is transformed by the matrix of its first bone slot only, at full weight, and the normal by that bone's rotation. With the far bit clear, the full three-bone weighted blend is used for the position (SPEC-0004, "Skinning and lighting in the vertex program"); the normal still uses the first bone's rotation. Lighting runs in both cases. The specular alpha term of program 6 runs only with the far bit clear. Unskinned programs ignore the far bit. Neither set enables fog (see Fog).

Which objects get the far set: characters submitted from the petpet slots and the world slots get it beyond 50 units; the special petpet copy and the hero are always near.

Clean side: beyond 50 units from the view point, skin each vertex with its first bone at full weight.

### Sort order (High)

Each new record is inserted so that the list is ordered as follows:

1. Opaque records come before translucent records. A record is **translucent** when its fade record has fade on and a fade level below 128. Every other record is **opaque**.
2. Within one class, a record goes before an existing record when its squared distance from the view point is larger. So within a class, the farther object is drawn first (back to front). A new record with a distance equal to an existing one goes after it.

Meaning of the value 128 (High). The fade level is the character's opacity on the scale 0 to 128 (128 fully opaque). Fade records are written in exactly three places (High):
- the camera fade ramp writes the focus hero's record (SPEC-0027, Fade writer);
- the enemy-class character update writes the record of each monster and boss every update, from that character's fade level (see Translucency, Fading objects);
- the background pass writes its own wrap record, at level 128 (Per-frame draw order, step 1); that record is not a draw record and never enters the sort.
Every character submitted from a character slot record (the two object lists and both world sub-lists) carries its fade record, and so does the main character; only the special object is submitted without one. So the hero and the fading characters listed under Translucency can sort translucent; every other record is opaque.

### Translucency (High)

Drawing a character whose fade record has fade on:

1. The character's screen rectangle (from its model bounds through the view, in GS units with the screen centre at 2048; see Main-character bounds) is clamped to the 512 by 448 screen.
2. The character is drawn between a fade begin and a fade end. The end composites that screen rectangle back over the frame as one textured sprite whose colour is (128, 128, 128, level), so the whole character is blended at opacity level / 128.
3. If the rectangle lies wholly off screen, nothing is composited.

A character with fade on and level 128 is opaque but is still drawn inside the wrap. A character with fade off is drawn directly.

Layer and blend details (High for the sequence and the render states; Medium for the layer's pixel format, taken to be the frame's own 32-bit format with 8-bit alpha). At the start of the wrap the frame target switches to a separate screen-sized layer and the clamped rectangle of that layer is cleared to colour (0, 0, 0) and alpha 0, writing colour only. The character is then drawn into the layer with its own part materials and render states, unchanged by the fade: part colours, part alpha, part blend settings and the part depth test and depth write all work as for an opaque character, and the layer is depth-tested against the shared depth buffer, which already holds the world and the characters drawn before it. So a part hidden behind the world leaves the layer clear there, and the character's visible parts write their depth into the shared depth buffer. At the end of the wrap the target switches back to the frame and the rectangle of the layer is drawn as one sprite: texture function modulate with texture colour and alpha used, nearest-texel filtering, vertex colour (128, 128, 128, level) so the sprite alpha is layer alpha x level / 128, depth test always, and no depth write (the alpha test is set to fail every pixel with the fail action "write the frame buffer only", so colour is written and depth is not). The blend is the plain source-over blend: result = destination + (layer colour - destination) x a, where a = sprite alpha / 128 (128 = 1.0). Where an opaque part was drawn, the layer alpha is 128 and a = level / 128. Consequences: the character fades as one object (parts that overlap each other do not show through one another; each pixel keeps only its nearest surface, then the pixel is blended once); opaque parts blend like every other part; a part whose own texel alpha is below 128 contributes a layer pixel that is both pre-weighted by that alpha and weighted again by the sprite (a slightly darker fringe on such edges); and a character at level 0 is invisible yet still writes depth, so depth-tested draws that follow in the frame (the effects pass, except while the hero fades) are hidden behind its silhouette.

While the hero's fade is on, the effects pass is drawn before the character pass instead of after it (Per-frame draw order, step 8).

Fading objects (High for the writers and the per-update feed; Medium-High for the two bosses; Medium for the ghost cycle). These are the objects whose fade record is set, so they are drawn through the off-screen composite:
- **The hero (camera-near fade)**: the camera fade ramp of SPEC-0027 writes the focus hero's record directly, so the hero fades out as the camera comes close to it.
- **Monsters and bosses (character fade level)**: each such character keeps an integer fade level, 128 after set-up. Every update it feeds the record: a level below 128 sets fade on with that level; exactly 128 sets fade off; above 128 sets fade on with level 128 (drawn through the composite path, but sorted opaque). The level is changed by:
  - **Ghost monsters**: the ghost opacity cycle of SPEC-0029 (monster fields Ghost Opacity Min and Ghost Opacity Max, 0 to 1) sets the level to trunc(opacity x 128), so a ghost is translucent while its opacity is below 1.0 and is drawn directly, with no layer, at 1.0. Every ghost row on disc has a maximum opacity of 0.5 or 0.7, so no shipped ghost is ever fully opaque: a level of 32 (opacity 0.25) shows one quarter of the ghost, not nothing, and a level of 0 shows nothing.
  - **Gelert Assassin**: a fade-in counter rises by 1 per tick while its fade-in flag is set, up to 128, and the level is the counter (from 0, about 2.1 s to opaque).
  - **Cloud Beast**: sets its level to 129, which gives fade on with level 128, so it is drawn through the composite path but sorted opaque.
- No other writer of a character's fade level is known; with the set-up value 128 every other monster has fade off.

Clean side: render a fading character to its own layer and blend that layer over the frame with opacity level / 128 (whole-character opacity, no per-triangle sorting).

### Flush (High)

The flush walks the sorted list once, in order. For each record:

1. If the record has a fade record and its fade is on, begin the fade wrap (see Translucency).
2. Select the vertex-program state set from the record's far flag (near or far).
3. Call the object's draw method with the record's stored values.
4. If the record has a fade record and its fade is on, end the fade wrap (composite the rectangle).

The number of objects drawn is counted. If the count differs from the number of records, the original prints a debug message and returns as usual. The clean side should draw every record and need not check this.

### Sky domes (High; Medium-High for the per-frame rate)

Resources:
- Resources 1, 2 and 3 are the sky domes. Each is a chunk chain of one shader list (SPEC-0003), one mesh container with two mesh children, role ids 1 and 10 (SPEC-0004), and one texture chunk. The textures are 8-bit indexed textures of 256 by 256, 128 by 128 and 32 by 32 texels with 16 by 16 colour tables.
- Resources 1 and 2 are byte-identical (118,784 bytes each), so dome 2 looks the same as dome 1. Resource 3 differs (151,552 bytes, with a larger texture chunk).
- Resource 0 is byte-identical to resources 1 and 2. No rule selects it (the dome number is never below 1).
- Resource 528 has the dome layout (a shader list, a mesh container with 13 mesh children of role ids 0 to 4 and 100 to 107, and textures including two of 256 by 256) but no loader names it (Medium-High). The clean side need not load it.

Selection on every world load:
1. Take the world's slot number s (its row in the 193-row area table, SPEC-0006).
2. Dome number: 2 when s is 151 to 156 or 186; 3 when s is 157 to 185; 1 for every other slot.
3. If that dome is already the loaded dome, nothing is read.
4. Otherwise the dome resource is read into the 460,800-byte dome buffer (set up once at boot), its chunks are fixed up, and it becomes the loaded dome.

By world pack (High): dome 2 is used by the world packs 1300, 1323, 1324, 1327, 1335, 1344 and 1427. Dome 3 is used by the world packs 1350, 1353, 1358, 1363, 1369 to 1371, 1381, 1382, 1391 to 1398, 1405, 1407, 1409, 1411, 1414, 1417, 1419, 1421, 1423 and 1425. Every other world uses dome 1.

Dome layers. The dome keeps the loaded dome's shader list and four mesh instances (layers), filled from the children of its mesh container by role id:

| Role id | Layer | Behaviour |
| --- | --- | --- |
| 1 | A | Static, identity rotation |
| 2 | B | Rotates about the world Y axis by −0.0009 radians per frame (data word 0x2DB488) |
| 3 | C | Rotates about the world Y axis by −0.0006 radians per frame (data word 0x2DB48C) |
| 10 | D | Static; drawn only while the atmosphere blend factor is above 0, with that factor as its master opacity |

Other role ids are ignored (0, 4 and 100 to 107 of resource 528 would be skipped). Exactly two roles rotate: role 2 (layer B) and role 3 (layer C), both about the vertical world Y axis through the dome centre (0, 0, 0), both with the sign "minus" (the angle only decreases), at a fixed step per rendered frame with no time scaling (High for the roles, axis, sign and steps; the two steps are single-precision words 0.0009 and 0.0006 in the boot data):
- layer B: −0.0009 radians per frame = −0.054 radians per second at 60 frames per second = −3.09 degrees per second; one full turn takes 6981.3 frames (116.4 s);
- layer C: −0.0006 radians per frame = −0.036 radians per second = −2.06 degrees per second; one full turn takes 10471.98 frames (174.5 s).
Layers A (role 1) and D (role 10) never rotate. The stored angle is not wrapped; the sine and cosine reduce it themselves. Layers B and C start from angle 0 at every dome load, and the angle is stepped before the layer is drawn, so the first drawn frame already uses −0.0009 (B) or −0.0006 (C). Test vector: after 100 frames layer B's angle is −0.09 radians (−5.16 degrees) and layer C's −0.06 radians (−3.44 degrees); with a = −0.09 the rule above sends the point (1, 0, 0) of layer B to (cos a, 0, −sin a) = (0.99596, 0, 0.08988).

On disc, domes 1, 2 and 3 hold only roles 1 and 10, so no dome layer rotates in the shipped game (High, from the role ids of the three dome files). Resource 528 holds roles 1, 2 and 3 and would therefore have a static layer and two rotating layers, but it is never loaded (it also has no role 10, so it would have no atmosphere-fade layer). A rewrite that adds a fourth dome from resource 528 must give role 2 and role 3 the rotations above.

Load: each time a different dome is read, all four layers are disabled and both angles are set to 0. Then each child with role 1, 2, 3 or 10 gets an identity matrix and its mesh, and its layer is enabled. Every layer's translation is (0, 0, 0).

Each frame, while the boot data word 0x2D7528 is nonzero (1 on disc):
1. The dome is the first thing added to the frame's material draw list, right after that list is reset and before the worlds, the characters and the other objects. Its material buckets are flushed first, so it is drawn before everything else.
2. The dome's shader list is made current.
3. Layer A, if enabled, is drawn with its matrix.
4. Layer C, if enabled: angle_C = angle_C − 0.0006. It is then drawn with the rotation whose rows are (cos a, 0, −sin a), (0, 1, 0) and (sin a, 0, cos a), with a = angle_C and no translation: a point (x, y, z) goes to (x cos a + z sin a, y, −x sin a + z cos a).
5. Layer B, if enabled: the same with angle_B and the step 0.0009.
6. Layer D, if enabled, only when the atmosphere blend factor b is above 0. b is the blend factor of the type 41 atmosphere zone pass (SPEC-0028 section 11; the "fade fraction" of SPEC-0027), kept in the atmosphere state object at +0x500 (SPEC-0006). For this draw the master opacity is b, and it is restored afterwards, so the layer's alpha scale follows b through the opacity table (see Fade level quantizer and draw colour scale).

Placement: world space, centred on the world origin. The camera position is not added, so the dome does not follow the camera. Extents of domes 1 to 3: the role-1 layer spans about x −5316 to 5316, y −1324 to 3271 and z −5384 to 5384 (sphere centre (0, −1294, 0), radius 5385); the role-10 layer spans about x −4516 to 4516 and y −904 to 2313 (radius 4804).

Depth: the dome uses its materials' own state (SPEC-0003): depth test GEQUAL, and an alpha test against 120 whose failing pixels are written to the frame buffer only. No special depth state is set. Because the dome is drawn first, everything else covers it.

Fog: none (see Fog).

Act 2 packs (High; from the pack data). Dome 2 serves the packs 1300, 1324, 1327, 1335 and 1344; dome 1 serves 1098, 1174 and 1299. Atmosphere zones (SPEC-0028 section 11): packs 1098, 1174 and 1299 have none, so the all-zero default applies (no post-pass colour, no screen tint, blend factor 0). Packs 1300, 1324, 1327, 1335 and 1344 each have one base zone with key 0 (ungated), so the blend factor is 0 and the dome's role-10 layer is not drawn there; the output is that zone itself. The values are read from each pack's type-41 chunk (the per-pack figures are in the pack specs: SPEC-0043 for 1335, SPEC-0044 for 1327, SPEC-0045 for 1300, SPEC-0041 for 1324, SPEC-0039 for 1344). Check values: pack 1335 has post-pass colour (138, 131, 203, 80) and screen tint (160, 84, 171, 16); pack 1300's tint alpha is 0, so it has no tint. Test vector (pack 1335): a pixel (200, 150, 100) with tint B = (160, 84, 171) and weight 16 / 128 gives (200 + (160 - 200) x 0.125, 150 + (84 - 150) x 0.125, 100 + (171 - 100) x 0.125) = (195.0, 141.75, 108.875) before the byte rounding. Because every Act 2 entry is a world load far from the previous eye position, the zone output of the new pack applies at once with no crossfade. While the current pack is 1299, 1300 or 1327 the neighbour world is never drawn (pairing test above).

Culling: the dome's distance nodes never cull while the eye is inside the dome, because the cull distance is measured to the bounding sphere's surface (max(0, d² − R²), SPEC-0004). Leaves are frustum-tested as usual.

Mesh content (domes 1 to 3 share the geometry): the role-1 layer holds an untextured band at the bottom (vertex colour (62, 99, 128), alpha 0 or 128) and the textured sky (a 256 by 256 texture, ordinary blending, vertex colour 128). The role-10 layer holds an untextured cap (colour (110, 79, 122, 64)) and a textured layer whose material has two register sets (drawn twice, SPEC-0004), under a type-4 fade node with pass byte 2, fade length 10 and fade end 4000. The dome's textured materials scroll their UVs in the vector unit (UV mode 2 of SPEC-0003; High): in each of domes 1 to 3 one single-pass material scrolls along U, and the two-pass material draws two texture layers that scroll in opposite directions. The scroll speeds are material data and the offset follows the render frame counter (SPEC-0003); the dome geometry does not move for it.

### Meridell mountains and the depth buffer (world pack 1023; High unless marked)

What the mountains are (High). The mountains seen from Meridell Village are ordinary mesh leaves of the pack's level display tree (SPEC-0004). They are not a backdrop model, not placed objects, not camera-facing sprites and not a sky-dome layer. They sit inside one distance-group node (type 4) whose cull value is 1e8 (it never culls), whose fade length is 0 (no distance fade), whose pass byte is 0 and whose layer flags are 0. That node lies under the one zone-column node of column 0, which holds the pack's terrain. Its bounds are x −1850.8 to 3093.4, y −81.4 to 1559.1, z −3089.3 to 1378.1, with sphere centre (550.3, 95.3, −1460.0) and radius 2929.5.

Mountain leaves (High):

| Group | Leaves | Vertices | Triangles | Texture | Union box (x, y, z) | Distance from the village centre (100, 30, 650) |
| --- | --- | --- | --- | --- | --- | --- |
| Material group 10 | 16 | 3,807 | 2,583 | 128 by 128, 4-bit | −1851 to 3093, −33 to 1397, −3089 to −90 | nearest box point 785, farthest corner 4,620 |
| Material group 11 | 8 | 1,972 | 1,350 | 32 by 32, 4-bit | −1532 to 2619, 69 to 1559, −2826 to −239 | nearest 890, farthest 4,196 |

Group 11 holds the tall peaks (the highest point is y 1559, at z about −2800 to −1500); group 10 holds the broad ridges and the low hills nearest the village. The mountain leaves lie north of the village (negative z); the village itself spans about x −245 to 495 and z 210 to 950. Every texel of both textures has alpha 128, so the mountains are fully opaque. The mountain triangles are large (mean area about 4,800 square units in group 10 and 1,700 in group 11, up to 48,000), and no two mountain triangles coincide: of the 3,933 mountain triangles, none has the same three corners as another. Neighbouring leaves share edge vertices (1,045 vertices of group 10 and 526 of group 11 are shared with another leaf), which is a seam and not an overlap. The foothills around the village (groups 2 to 6, 16 leaves, all within 840 units of the village centre, all in the same column-0 node) use mip-mapped textures; the mountain groups do not.

Material state of the mountains (High). Both groups use vertex program 0 (unlit, baked vertex colours), one pass, no UV animation, texture function modulate, repeat in both axes, bilinear magnification and minification at the base level only (no mip levels: the level is fixed), alpha blend (Cs − Cd) × As + Cd, alpha test GEQUAL 120 with failing pixels written to the frame buffer only (never triggered here, because every texel has alpha 128), and depth test GEQUAL. There is no fog and no back-face test (see Fog and Per-triangle clipping).

Draw order and pass (High). The dome is queued first, then the second world if one is loaded, then this tree. In the tree the mountain leaves are queued in tree order into material buckets, one per texture chunk (group 10 goes to the bucket of texture chunk 2, group 11 to that of chunk 0), in sub-list 0 (pass byte 0). There is no separate opaque and translucent pass: the mountains share the first sub-list of their buckets with the other opaque world geometry. Within a bucket, sub-lists 0 to 3 are drawn in order, and inside a sub-list in queue order.

Visibility (High):
- Story layer: the node's layer flags are 0, so it always passes.
- Zone column: the terrain node has column 0, and in the pack's visibility grid every row holds 1 in column 0 (61 of 61 rows). The mountains therefore never switch with the camera's cell. The village props, by contrast, hang under 58 other column nodes (columns 1 to 60), and those do switch per cell (SPEC-0006, Binary grids).
- Distance: the node's cull value 1e8 is compared with the squared distance to the sphere surface (SPEC-0004), so it never culls. The cull value 16,000,000 stored on the group node just below it is not tested (only type-4 nodes are). There is no fade and no level-of-detail switch, so there is no hysteresis to reproduce.
- Frustum: each box is classified by its eight corners (Culling). A group node is tested against all six sides and a leaf against five (not the far side). A box is culled only when all eight corners lie beyond the same side. The far side is 5,280 units of view depth. The farthest mountain vertex from any eye position in the village is under 4,700 units, so the far side never culls a mountain by depth; only the left, right, top and bottom sides do (tangents 0.625 and 0.46875). No mountain leaf box comes near the near plane unless the eye stands inside it, and the village eye positions are all outside.

Depth buffer format (High for the format, test and write; Medium for the conversion to an integer). The frame's depth buffer is 24-bit (Z format PSMZ24), the depth test is GEQUAL (larger is nearer, equal passes) and depth writes are on, for the main frame buffer. The vertex program supplies depth as k / z + c (Camera projection) with near N = 1.0 and far F = 10,560, and the integer part is stored. The near-plane value B is the atmosphere render scalar of the area (SPEC-0028, type 41). Its start value is 2^20 and the buffer is 2^24 values deep, so the game uses at most the lowest 1/16 of the range, and in Meridell much less. In pack 1023 two base atmosphere zones exist: the ungated one sets B = 300,000, and the one gated by key 2 (story flag 10 set and flag 322 clear) sets B = 100,000. During a world-change crossfade the scalar is interpolated.

Depth values and resolution (k and c from the Camera projection formulas; the step is the world distance covered by one depth value, z² / k):

| View depth z | Z with B = 300,000 | step | Z with B = 100,000 | step |
| --- | --- | --- | --- | --- |
| 500 | 572.6 | 0.8 | 191.5 | 2.5 |
| 1,000 | 272.6 | 3.3 | 91.5 | 10.0 |
| 2,000 | 122.6 | 13.3 | 41.5 | 40.0 |
| 3,000 | 72.6 | 30.0 | 24.9 | 90.0 |
| 4,000 | 47.6 | 53.3 | 16.5 | 160.0 |
| 5,000 | 32.6 | 83.3 | 11.5 | 250.0 |

With the default B = 2^20 the steps at 1,000, 2,000, 3,000 and 4,000 are 1.0, 3.8, 8.6 and 15.3. So two mountain surfaces closer in depth than one step end up with the same stored depth, and because the test is GEQUAL, the later draw wins the tie. The original is therefore stable only because its draw order is fixed; nothing in the data makes two mountain surfaces coincide, so a clean renderer with ordinary float or 24-bit depth and a near plane of 1.0 or more has no mountain surfaces to fight over.

Sky dome against the mountains (High). The dome (dome 1 for this pack) is drawn first with the same depth test and writes depth; the sky never draws over geometry that is nearer. From the village the dome's role-1 layer lies 3,198 to 5,827 units from the eye (the band 4,265 to 6,234, the role-10 layer 2,121 to 5,306), and along any ray to a mountain vertex the dome sphere is at least 1,100 units behind that vertex for six eye positions spread across the village. At B = 100,000 that is more than five depth values, so the mountains and the dome can never tie. The role-10 layer is drawn only while the atmosphere blend factor is above 0 (Sky domes); in pack 1023 the base zone is ungated unless story flag 10 is set and flag 322 clear, so by default that layer is off.

Dome facets (High). The role-1 layer is 162 sky triangles (186 vertices, vertex colour (128, 128, 128, 128), y −164 to 3271) and 72 band triangles (146 vertices, colour (62, 99, 128), alpha 128 on 110 vertices and alpha 0 on 36, y −1324 to −61). The role-10 layer is 156 triangles (alpha 128 on 165 vertices, alpha 0 on 23) and an 8-triangle cap (colour (110, 79, 122), alpha 64). The consecutive-vertex spacing is 1,300 to 2,200 units (up to 6,800 in the role-10 layer). The eye is inside the dome, so facets lie on every side: from the village about 25 of the 162 sky facets, 8 of the band, 18 of the role-10 layer and 4 of the cap straddle the plane of the camera for any view direction (nearer than depth 1 at one corner, farther at another). The original sends the dome's boxes through the per-triangle clip of Per-triangle clipping in the vertex program (the box holds the eye, so the classifier returns clipped); such facets are cut against w = 1 and the screen rectangle and drawn as a fan, never dropped. A straight-edged black wedge in the sky is therefore not something the original ever shows: the clear colour appears where a dome facet is missing.

Story-state scenery in the same node (High for the data). Five bowl-shaped leaves of material group 31 (278 vertices each, size 659 by 659 by 101, a UV-scroll texture, vertex alpha 128 on 244 vertices and 0 on 34) sit in type-4 nodes whose layer flags are single bits: 2^21 (story layers 1 and 12; box min (−1250, 136, −299)), 2^22 (layer 2; (−221, 199, 280), directly above the village, y 199 to 300), 2^24 (layer 4; (−296, 181, 1290)), 2^25 (layer 5; (2853, 181, −867)) and 2^26 (layer 6; (756, 348, −178)). Layer 2 shows under the same pair of flags (10 set, 322 clear) that gates the 100,000 depth scale. Where a pixel's alpha falls below 120 it blends but writes no depth, so a mountain drawn later in the same bucket replaces it.

Worked example (High for the geometry and culling; Medium for the integer conversion). Eye (30, 32, 700), looking toward −z (view depth = 700 − z; screen right is +x), story flag 10 clear (B = 300,000, k = 300,027.4, c = −27.41):
- Nearest hill: the group-10 leaf with box min (−1462, −33, −724), size (1271, 302, 635) spans view depth 790 to 1424: Z 352 to 183. It is inside the frustum.
- Tallest peaks: the group-11 leaf with box min (122, 468, −2826), size (975, 1091, 1343) spans depth 2184 to 3526: Z 110 to 58. The group-10 leaf with box min (877, 108, −3089), size (977, 1080, 1792) spans 1997 to 3789: Z 123 to 52.
- Culled: the group-10 leaf with box min (1718, 125, −1823), size (1004, 670, 645) spans depth 1878 to 2523 with x offsets 1688 to 2692; at every corner the offset exceeds 0.625 times the depth, so all eight corners are beyond the right side and the leaf is dropped. Of the 24 mountain leaves, 23 pass the frustum test from this eye and this heading.
- Fog factor: none (colour is the texture times the baked vertex colour). No leaf is clipped; every mountain leaf is fully inside the near and far sides.
- The same view with flag 10 set and flag 322 clear (B = 100,000, k = 100,008.5, c = −8.47): the nearest hill gives Z 118 to 62, the tallest peak 37 to 20, and the farthest ridge 42 to 18. The visible result is the same; only the stored values and steps change.

Likely causes of unstable mountains in a port, most likely first, each with the original's rule:
1. Zone gate applied to the wrong nodes (High that the rule is as stated). The terrain node is column 0 and passes in every row of the grid. A port that gates every column node by "the camera's zone equals the column", or that treats column 0 as the camera's own cell, makes the mountains and foothills appear only in one cell and pop as the camera crosses cell boundaries. Rule: the query reads the byte at row R, column c of the grid (SPEC-0006); the "equals" mode never runs.
2. Bowl and dome translucency handled with the wrong depth rule (Medium-High). In the original a pixel with alpha below 120 blends but writes no depth, and nothing is sorted by alpha: the bowls, the dome's alpha ramps and the role-10 layer simply follow bucket and queue order. A port that depth-writes every blended pixel, discards them instead, or re-sorts the bowls per frame makes the bowl edges and the mountains behind them swap places as the camera moves. Check also that story layers 1, 2, 4, 5, 6 follow the flag rule of Culling, not a per-frame guess.
3. Frustum test on the big boxes (Medium-High). The leaf boxes are 1,000 to 1,800 units wide. Rule: eight corners against five or six sides, culled only when all eight are beyond one side; the far side is 5,280 view depth and never culls these leaves. A test on the box centre or the sphere, or on distance (for example the 350-unit object cull or a 50-unit far-state rule borrowed from characters), makes whole ridges blink at the screen edges and with distance.
4. Depth precision (Medium). Use near 1.0 or larger and at least 24-bit depth; the data has no coincident mountain surfaces, so a z-fight on the mountains means the port lost precision (near plane far below 1.0, 16-bit depth, or a different matrix path for the leaves of one mesh so shared seam vertices stop agreeing). If the port follows the area's depth scalar exactly, it must use the 24-bit integer rule above, with ties decided by draw order and an equal depth passing; a depth test of strictly-less makes tied surfaces flip with the draw order.
5. Depth scale that changes per frame (Medium). The scalar differs between the two zones (300,000 and 100,000) and is interpolated during a crossfade. A port that rebuilds a quantised depth mapping every frame, or mixes the original scalar with its own near and far, changes the stored values from frame to frame. Rule: it affects only stored values, never the order of surfaces, so a port that keeps float depth may ignore it.
6. Level of detail or fade applied to the wrong node (Medium-Low). The mountains' node has fade length 0 and cull 1e8; other nodes of the tree have fade length 10 and cull 16,000,000 (and the fade of a leaf comes from its nearest type-4 ancestor only). A default fade or a cull measured to the sphere centre instead of the surface would blink content at about 4,000 units, which is beyond every village view.
7. Sky drawn after the world or with depth off (High that the original does not). The dome is first. Drawn later with the test off, the sky overpaints the mountains; drawn later with the comparison reversed, it fails to show behind them.

The black diagonal in the sky, most likely first:
1. A dome facet dropped instead of clipped. With the eye inside the dome, tens of its facets straddle the camera plane in every view; the original clips them. A port that discards a triangle when one vertex is behind the camera (or has w at or below zero), or that clips with a wrong rule, leaves a clear-coloured wedge with a straight edge.
2. A far plane below the dome. The dome's farthest vertex is about 5,830 units from the village eye. The 5,280 value is only the culling far; the projection far is 10,560 and nothing is clipped on the far side. A port with far at 5,280 or less cuts the farthest facets along a plane, which looks like a straight diagonal.
3. Strip decode: a vertex whose flag bit 7 is set emits no triangle but still advances the strip; every batch is independent. A wrong rule adds or loses facets.
4. Back-face culling enabled. The dome is drawn from inside and the original draws every mesh double-sided.
5. Alpha ramps treated as pre-multiplied or as a black matte. Dome vertices carry alpha 0 on one edge of the band and the role-10 layer, so a facet fades diagonally; blend with (Cs − Cd) × As + Cd with 128 as 1.0.

Default if unresolved (rewrite choice): when the rewrite cannot reproduce the 24-bit integer depth rule, keep floating depth with near 1.0 and far 10,560 and ignore the atmosphere depth scalar; the visible order of the mountains, the dome and the bowls is unchanged.

### Level water (High for the material rule and the scroll; Medium-High that these leaves are the water)

Level water has no draw path of its own: no pass, no flag and no animator. A water surface is an ordinary flat mesh leaf of the level display tree (SPEC-0004), drawn by the same tree walk and material draw list as the rest of the level, and its look comes from its material (SPEC-0003).

- **Material (High).** Vertex program 0 (unlit; the vertex colour byte 128 is 1.0), blended onto the frame by the material's own blend register. On disc 294 large flat leaves (horizontal extent above 2,500 square units, height extent below 2) in 63 packs use a UV-scroll material; all 294 use program 0, 71 are one-pass and 223 two-pass, and 292 blend in pass 0 with the plain source-over setting (source alpha against destination), the other 2 (one-pass) with a destination-alpha-weighted additive setting. The second pass of the 223 two-pass leaves blends plain additively in 26 and destination-alpha additively in 197 (High; the second pass is the lighter, glittering layer over the first).
- **Animation (High).** Only the vector-unit UV scroll of SPEC-0003 (UV mode 2): each pass adds (s × N − trunc(s × N)) to its own U and V, with s the pass's speeds per main-loop pass and N the 16-bit pass counter. A two-pass water material gives two texture layers that drift independently, in opposite directions on 131 of the 223 two-pass leaves, in the same direction on 79, and with one layer standing still on 13. Speed magnitudes on these 294 leaves run from 0.000125 to 0.02 per pass, that is 0.0075 to 1.2 texture sizes per second at 60 passes per second (for example (0, −0.005), (−0.00075, −0.00075), (0, 0.00125), (0, 0.02)); across the whole disc the largest stored scroll speed is 0.125 per pass, 7.5 per second (SPEC-0003, "What the scrolled materials are on disc"). The vertices, the surface height and the vertex alpha never change.
- **Alpha (High).** There is no animated alpha. The surface alpha is the vertex alpha (128 = 1.0 in the example below) times the texture's alpha, through the material's blend register. The CPU water animator of SPEC-0003 (vertex animator 3, which does pulse the alpha) belongs to the 13 leaves of the shared effect-model resource only; no level leaf has a nonzero vertex-animator byte. Some water is not scrolled at all (UV mode 0 with the same blend).
- **Kill zone.** The deep-water kill is separate ground data (a ground entry with surface word 8 and its overlay link, SPEC-0018 and SPEC-0057). The visible surface and the kill area line up only by authoring.
- **Footsteps.** The water footstep asks for effect definition 162, which does not exist in the effect table (SPEC-0034), so stepping in water starts no effect. The water death splash (one-shot effect 0xBC) does exist.
- **Fire, lava and energy are not level-mesh animations (High for the disc data; Medium-High for the drawing).** Level meshes (the display tree, placed objects and instanced meshes) have no group with a flame, fire or lava texture that scrolls or runs a flipbook. The only level-mesh animations are the scrolled sheets of SPEC-0003 (water, haze and cloud layers, a small additive glow shimmer, a few lightning and light columns) and one 32-cell speckle flipbook in a single pack. Flames in levels are effect particles: sprite particles (SPEC-0034; flame pictures with additive blending, no UV mode) and the 24 effect definitions that draw a model with a scrolled or flipbook material (SPEC-0003, "Effect model particles"). A model particle is drawn with the ordinary mesh path: its material gives the blend (additive or plain alpha), the alpha test and the scroll. Pack 1174's first room has no animated or additive material anywhere near its doorway (z 52.5, x within 9.1): the leaves there are the wall and frame, a 256 by 128 plank slab (x ±11.7, y −128.1 to −105.2, z 52.07 to 52.88) and the barrier gate objects, all one-pass, program 0, ordinary alpha blend, no UV mode. A sheet of flame in that doorway therefore comes from effect particles, not from a scrolling material (High).

Worked example, pack 1414. Ground entry 91 has surface word 8 and spans x −43.2 to 42.9, z −24.3 to 43.0 at height 1.23. The level tree holds two single-pass flat leaves of 85.8 by 67.1 units over that area at height 1.2, each with 65 vertices and vertex colour (61, 59, 48, 128): layer 1 uses material group 19 (a 64 by 64 palette texture, source-over blend, scroll (0, +0.00125) per pass) and layer 2 uses group 20 (a 128 by 128 texture, the destination-alpha blend with value 0x58, scroll (0, −0.00125)). After 600 passes layer 1's V offset is +0.75 and layer 2's is −0.75; after 800 passes both are 0 again (a 13.3 second cycle at 60 passes per second). Pack 1176 shows the static case: its water overlay (ground entry 45, surface word 8) is at height 22.78, and the level tree holds one flat leaf of 301.7 by 319.7 units at height 22.7 to 23.0 whose material is unlit, source-over and unscrolled.

### Packet submission (Medium-Low)

A helper builds one packet from a data block and submits it to the platform path (SPEC-0008):

1. A data block longer than 1024 bytes is refused and an error is logged. Nothing is submitted.
2. Otherwise a 1024-byte packet buffer is cleared and the data is copied to its start. Two 32-byte header blocks are taken from the helper's two inputs, and a fixed header word is set.
3. The packet is submitted with a length equal to the data size plus 88 bytes.
4. If the submit fails, a cleanup routine runs and an error is logged.

The header formats belong to the platform spec and are not given here. The callers are not all identified (Open question 39).

### Model parts (High)

For each part of a model, the draw driver runs a visibility test on the part's bounding box:

1. Build the part's eight corners from the box (see Corner builder and eight-corner clip test below).
2. Classify the corners with the side mask of model parts (all sides except far; see Culling).
3. Use the outcome:
   - **culled** (value -1): the part is not drawn;
   - **inside** (value 0): drawn with the clip bit clear;
   - **clipped** (value 1): drawn with the clip bit set, so its batches take the clipping path (see Per-triangle clipping in the vertex program).

The part's drawn result is then passed to its material draw. The model-part driver also selects the skinned bone matrices for the part (three source groups by bone index; SPEC-0005 and SPEC-0004) before the draw.

Test vectors for the model-part classifier are in Test vectors.

### Character part draw and the skin palette (High unless marked)

This is the full sequence of the class A draw for one character render object and one view, from the gates to the last queued packet. The result is a list of queued items (palette upload, light block, matrix records, colour records, material changes, mesh packets); the platform side (SPEC-0008) and the vertex programs (SPEC-0004) consume them.

**Gates.** Nothing is drawn when the object has no parts, when its render-ready flag is clear (What the render path does not contain, Level of detail), or when every part is culled by the classifier. The view index of the draw context (0 for the main view; Medium for the meaning of other values) selects which of the two stored classifier results of a part is used. When the view index is 0, the object's "palette already queued" flag is cleared before the classifier runs. The flush always passes the "reuse" argument as zero, so in play the palette and light block are queued for every call.

**Which path.** Each render object has a path word, set once when the object is built from its model file: 1 when the first child of the file's model container is a type-13 chunk, 0 when it is a type-12 chunk (SPEC-0004). Every model container on the disc starts with a type-13 child (a type-12 child occurs only later inside one container), so every character render object takes path 1, the vector-unit-1 path described next. Path 0 (the main-processor and first-vector-unit path of Light slots and the lit draw) is never selected on disc (Resolved, Open question 73). Its differences, for a rewrite that wants to support it: the palette has no world matrix folded in and is kept in the first vector unit; each part's matrix record uses the object's world matrix instead of the identity; no draw colour scale step runs; the light block is loaded into the first vector unit instead of being sent; and each leaf queues the vertex program's per-buffer output packet instead of its own packet.

**Steps of path 1.**
1. Classify the parts for this view (Model parts).
2. Build and queue the skin palette (next paragraphs), then set the "palette already queued" flag.
3. Refresh the character's four light slots and build and queue the light block (Light slots and the lit draw). On path 1 the block is queued for the vector units; path 0 only loads it into the first vector unit.
4. Reset the count of drawn leaves, and apply the colour-override slot 0 pulse to the 15 colour-scale records of the current buffer (Per-frame draw order, step 6).
5. For each part in order, skip it when its classification for this view is culled (−1) or when its mesh-count field is 0. Otherwise: make the part's shader list current; queue the part's matrix record with the identity as the model matrix and the draw's view point as the reference point (the palette already contains the object's world matrix); select the vertex-program state set with the clip bit equal to the part's classification (0 inside, 1 clipped); then walk the part's node tree.
6. Node walk, from the part's root: a type 1 node is a mesh leaf and is drawn as below (its children are not walked); a type 4 node first walks its third subtree link when that link is set and its layer flags are 0 or share a bit with the character's draw mask (Linked subtree), and then, like every other node type, walks its children in sibling order.
7. Mesh leaf: (a) let g be the leaf's 16-bit material group index (at node offset 0x5C) and j the colour slot byte of group g (group offset 0x27; j = 0 when g is out of range); (b) choose the colour record: while colour-override slot 1 is on (the hit flash), the object's override record (its target vector and the two mask flags), otherwise the object's record for slot j in the current buffer; queue it unless it is equal, in every lane it masks, to the colour record queued last (so a run of parts with the same slot sends it once); (c) make material group g current (SPEC-0003; a texture upload is queued when the group's texture is not resident); (d) queue the leaf's mesh packet (SPEC-0004): its own packet when the leaf's kind byte (offset 0x5E) is 0, which is the case for all 315 leaves of the 16 outfit files (a nonzero kind would select the per-buffer output packet of path 0); (e) add 1 to the count of drawn leaves.
8. The call returns the count of drawn leaves (the flush's object count, Flush).

The double buffer index b is the frame parity (0 or 1, flipped at the end of every frame). The palette buffer, the matrix record block and the colour records of an object each exist twice, selected by b, so a record queued this frame is never overwritten before the platform side has consumed it.

**Skin palette (path 1).** Let N be the bone count of the object's first part (the skeleton, SPEC-0005); N is at most 89 because the object's palette buffer holds 4288 bytes (the largest skeleton on disc has 74 bones). For bone i from 0 to N − 1:
- Pose matrix P_i. It is a 4 × 4 matrix of four 16-byte rows in row-vector form (a point row p maps to p·P), rows 0 to 2 the rotation and row 3 the translation. The pose matrices live in up to three arrays of the render object, chosen by bone index: bones 0 to n0 − 1 use the main array; when a second skeleton exists, bones n0 to n0 + n1 − 1 use the second array and the remaining bones use the third array; when there is no second skeleton, the remaining bones use the third array if it exists and otherwise continue in the main array. Within an array the matrix for a bone is the next one in order (the n-th bone of the group takes element n). n0 and n1 are the bone counts of the main and second skeletons.
  - Which skeleton each array belongs to (High). The three arrays are the outputs of the character's three skeleton families (SPEC-0005, Families): the first holds the main joints (bones 0 to n0 − 1), the second the sub-part joints (n1 of them, bones n0 to n0 + n1 − 1) and the third the face joints, so the bank order equals the skin bone order of SPEC-0005. They belong to whichever character the render object draws, not to one class: a hero (Tor or Roberta), a region character (an NPC or an enemy) or the petpet. The first array always exists. The second and third exist only when the character's record names a sub-part skeleton and a face skeleton; otherwise the pointer is null and the palette continues in the arrays that exist (the rule above).
  - Who fills them (High). Each array is allocated, once, when the character's body is built, together with a matching array of 32-byte joint records that the animation sampler writes: the hero slot builder runs the main, the sub-part and the face constructor for each hero slot, and the region character builder runs the same three constructors for every region character; the petpet is found in the same world character table, so it is built the same way (Medium). Then, every tick, the shared skeleton pose pass (SPEC-0005, Posing a skeleton; SPEC-0040, Behavior 3) turns the joint records into the three matrix arrays, parent first, main family, then sub-part, then face. The palette routine only reads them, after the pass of the same tick.
- Bind translation B_i: the identity with row 3 set to (−b_x, −b_y, −b_z, 1), where b is the bone's bind position (bone record offset 0x20, SPEC-0004 and SPEC-0005).
- World matrix W: the render object's world matrix (the object's frame, row-vector form).
- Combined matrix M_i = B_i · P_i · W (row-vector product). Its upper 3 × 3 block is the pose rotation followed by the world rotation, and its row 3 is the translation: ((t_pose − b·R_pose) · R_world) + t_world.
- Stored form: three 16-byte vectors per bone, at bone offset 48 i. Vector j (j = 0, 1, 2) is (M[j][0], M[j][1], M[j][2], M[3][j]): row j of the 3 × 3 block with translation component j in w. This is the column-vector form of SPEC-0004: vector j holds column j of the rotation R = Mᵀ in x, y, z and translation component j in w. (A fourth, translation row vector is also written but the next bone overwrites it and it is not part of the palette.)
- Meaning: a vertex position p bound to bone i is carried to the world by W · P_i · (p − b_i) in column-vector terms: take p relative to the bone's bind position, apply the bone's animated pose, then the object's world matrix. A bone at its bind pose (pose rotation identity, pose translation equal to b_i) with an identity world matrix gives vectors (1, 0, 0, 0), (0, 1, 0, 0) and (0, 0, 1, 0).

**Upload.** The N × 3 vectors are written to the object's palette buffer for the frame buffer index b, flushed from the processor cache, and queued as one vector-unit-1 memory upload of 3N quadwords to data-memory quadword 128 (Medium-High), behind a program-synchronisation word. A vertex's bone-slot byte (3 × bone, SPEC-0004) is therefore added to quadword 128 to find its bone matrix. The upload is queued in draw order, before the object's own mesh packets, so each character sees its own palette. Nothing is shared between characters and nothing is cached between frames: the palette is rebuilt in full at every draw.

Confirmed and extended 2026-11-09 (High; the palette builder was read in full): the palette described above (rows 0 to 2 with the translation component in w, 0x30 bytes per bone, the fourth row overwritten by the next bone, M = translate(−bind) × pose × world, the three source arrays chosen by bone index, the no-second-skeleton rule and the cache write-back of N × 0x30 bytes, the buffer at model +0x1268 + 4 × b) is what the builder does. A SECOND palette builder exists (not described before): the same product without the root (world) multiply, written to the fast scratch memory (the scratchpad) at 0x30 per bone and then copied (N × 3 quadwords) into the first vector unit's data memory from address 0; both builders are called from the class A character draw; which one serves a given object (probably the path word of the object, Medium) was not established. Separately, a scratchpad vertex skinning job (used by the object-tree walk) works through batches: for each batch (count, output offset A, output offset B; the batch count at job +0x48 and the entries at +0x4C) it transfers count × 48 bytes (three quadwords per vertex) from the source (job +0x18, advancing 0x30 per vertex) into the scratchpad, waits for the transfer, runs the transform (input at scratchpad 0, output at scratchpad 0x2000), transfers count quadwords from scratchpad 0x2000 to the destination plus offset A and from 0x3000 to the destination plus offset B, waiting after each; the destination is job +0x3C + 4 × the double-buffer selector (0x2EC830, buffer 0 or 1). A rewrite that skins with the SPEC-0005 formula needs none of this; the transfer sizes are recorded only for a platform-accurate port.

Test vectors (real skeleton data, first outfit file): bone 1 of that skeleton has bind position (0, 1.21875, 0.07903). With identity world, identity pose rotation and pose translation (0, 1.31875, 0.07903) (the bone raised by 0.1), M's row 3 is (0, 0.1, 0): the stored vectors are (1, 0, 0, 0), (0, 1, 0, 0.1) and (0, 0, 1, 0), and a vertex at the bind position with weight 1 lands at (0, 1.31875, 0.07903). With the world translation (10, 0, 5) added, the w components become 10, 0.1 and 5. A 58-bone skeleton (that file) uploads 174 quadwords (2784 bytes); the 89-bone limit uploads 267 quadwords (4272 bytes, inside the 4288-byte buffer). Colour records: three consecutive leaves with colour slots 3, 3, 5 queue the slot-3 record once and then the slot-5 record, provided the two records differ; with colour-override slot 1 on, one record (the override vector) is queued for all three. Pose arrays (real data, Tor outfit 504: 32 main, 13 sub-part and 13 face joints): bone 5 takes element 5 of the main array; bone 40 takes element 8 of the second array (one of the tail joints 8 to 12); bone 45 takes element 0 of the third array; bone 57 takes element 12 of the third array. A townsfolk model with 32 main, 10 sub-part and 12 face joints (54 bones): bone 32 is element 0 of the second array and bone 42 is element 0 of the third array. A character with no sub-part skeleton, a face skeleton and more than n0 bones takes its bones from n0 on from the third array; with neither skeleton the extra bones continue in the main array.

### Corner builder and eight-corner clip test (High for the arithmetic; Low for the standalone builder's caller)

The part test is made of two steps.

1. **Corner builder.** It takes a box as a centre and a half-extent on each of the three axes. It produces eight corners, one for each combination of the centre plus or minus the half-extent on each axis. The arithmetic is High. A standalone copy of the builder has no direct caller in the boot program (Open question 36); the model-part and mesh-leaf paths build their corners the same way from their boxes.
2. **Eight-corner clip test.** It classifies the eight corners with a side mask M, by the rules of Culling: a frustum pass, then the near-plane rule, then a guard-band pass.

The clip classification is written over N points, not only eight: the same all-points and no-point rules apply to any count (Medium; the callers with other counts are not identified, Open question 45). Two further corner routines are Low. One computes four corner values from two signed fixed-point inputs centred on the screen centre (2048, as in SPEC-0027). The other computes the four corners of a box or quad from vectors and scalars, projects them through a helper that is not described here, and stores six values. Their callers are not identified (Open question 45).

Test vector: the corner builder, centre (0, 0, 0) and half-extent (1, 2, 3), gives eight corners: x is -1 or 1, y is -2 or 2, and z is -3 or 3, with each combination once.

### Culling (High)

The draw path culls at three levels. There is no grid-cell culling in the draw path.

**Object level.** The 350-unit submit cull and the 50-unit far flag use the 3D distance from the view point (see Object submission and culling).

**Part and leaf classifier.** Each corner gets out-code bits, one per side: +x 1, −x 2, +y 4, −y 8, far 16, near 32 (the near bit means the corner is nearer to the eye than the near plane, outside the view volume). Side mask M: 0x2F (every side except far) for model parts and mesh leaves; 0x3F (all six sides) for group nodes.

1. Frustum pass, against the frustum clip matrix: if one side bit is set on all eight corners, the box is **culled** (−1). Otherwise, if no corner has any bit of M, it is **inside** (0).
2. Near-plane rule: if any corner has the near bit and M contains the near side, the box is **clipped** (1).
3. Guard-band pass, against the guard-band clip matrix: if any corner is outside a side of the guard band that is in M, the box is **clipped** (1); otherwise it is **inside** (0).

So the far side never forces clipping for parts and leaves, while all corners beyond the far plane still cull. The sixth side bit is the near plane, handled by the near-plane rule.

**Geometry near the eye (High for the rules; the camera side is in SPEC-0027, Camera collision and occlusion).** The near plane is at 1.0 and the focal length is 512. A box with all eight corners nearer than the near plane is culled whole by the all-corners rule. A box with some corners nearer is classified clipped, and its batches are cut triangle by triangle at the w = 1 plane (Per-triangle clipping in the vertex program), so a wall or door that the eye is touching is clipped, not dropped. There is no back-face removal, so from inside a closed mesh its inner faces are drawn and the eye does not see through it. No rule fades, hides or sorts scenery meshes because they lie between the eye and the hero: the only camera-near fade is the hero's own fade ramp (below 1.7 units clear, 1.7 to 5.0 partly clear). Whether a part of the level draws depends on its zone column and layer flags only, never on the eye's distance to the part except the submit and distance culls above. The zone row used by those column tests is the zone of the followed hero's ground entry in the normal follow camera, and the zone of the camera's own ground entry in every other mode and during a scripted play, so an eye that is in another zone than the hero changes nothing in the normal follow camera. In the interior world, the other world is not drawn.
Test vectors: a box whose corners all lie at view depth below 1.0: culled. A box with corners at depths 0.5 and 3.0: clipped; a triangle with one vertex at w 0.4 and two at w 4 is cut at w = 1 and drawn as a fan.

**World tree** (the level display tree of SPEC-0006, node layout in SPEC-0004). A type-3 node is skipped with its subtree when the zone query for its column (SPEC-0004, node table) fails. A type-4 node is skipped with its subtree when its layer flags are nonzero and share no bit with the world layer mask; then the distance cull of SPEC-0004 applies. The layer flags are a 32-bit word (SPEC-0004, node table).

World layer mask = the world record's mask word, OR, for each story layer i that is visible, bit (20 + i) for i = 1 to 8 and bit (17 + (i − 8)) for i = 9 to 12. Layers 1 and 12 share bit 21. Story layer visibility, from the story flags of SPEC-0049 ("a and not b" means flag a set and flag b clear):

- layer 1: 83 and not 5; layer 2: 10 and not 322; layer 3: 64 and not 283; layer 4: 57 and not 58;
- layer 5: 74 and not 263; layer 6: 85 and not 298; layer 7: 92 and not 94; layer 8: flag 248 set;
- layer 9: 332 and not 100; layer 10: 332 and not 102; layer 11: 332 and not 104; layer 12: 332 and not 106.

World mask word (High). The world record's mask word is 0 when the world is set up, and only the script call SetWorldDisplayMask(W, m) writes it, with the value 2 x m (SPEC-0030). The same word is also the node filter of the world's placed-object draw. Level scripts pass m = 1 and m = 2 in most modules, combinations of the bits 1, 2 and 4 in the module of pack 1066 (SPEC-0043), and m = 15 in the two transition modules of packs 696 and 714 (SPEC-0042), so the word is 0, 2, 4, 6, 8, 10, 12, 14 or 30 in those modules, and any combination of the eight table bits in packs 758, 1023 and 1358 (below). Packs 699 and 986 also pass m = 1 or 2 (pack 699) or a one-bit word (pack 986); the full list of calls is in SPEC-0042, World mask call sites. After the call with m = 15, low bits 1 to 4 (values 2, 4, 8 and 16) are all enabled. The level display trees of packs 696 and 714 hold no node whose layer flags use these low bits (only story-layer bits 21 to 26 and zero), so there the word changes no world node. A node whose only layer bit is bit 3 is drawn only in a world whose script passes a value with that bit (the survey has one node with bit 3). Test vectors: word 4 (m = 2) with no story layer visible: layer flags 0x4 drawn, 0x2 skipped, 0x8 skipped, 0 drawn. Word 0: every node with only low bits is skipped.

Disc survey (High): of the 19,659 type-4 nodes in the level display trees of world packs 532 to 1433, 19,445 have layer flags 0. Layer bits 21 (34 nodes), 22 (31), 23 (30), 24 (34), 25 (31) and 26 (31) occur in early packs (678, 686, 691, 696, 699, 702 to 706, 713, 714 and others), so story layers 1 to 6 swap scenery there. Bits 18, 19 and 20 (3 nodes each, packs 1370, 1371 and 1398) are layers 9 to 11. Low bits 1 (8 nodes), 2 (5) and 3 (1), in packs 699, 703, 706, 837, 1066, 1176, 1184 and 1230, can be enabled only by the world mask word.

Where the display layer bits come from (High). The layer flags of a node are stored on the disc and no game code assigns or edits them; the world word is written only by the script call. A collision gate value v (SPEC-0018, Primitive class bits and mask gate) pairs with the node value 2 v, because the call stores 2 x m. So a module that passes the same number m to the collision and display setters switches matching collision records and display nodes together: collision gate 1 with node value 2 (bit 1), gate 2 with node 4 (bit 2), gate 4 with node 8 (bit 3) and gate 8 with node 16 (bit 4).

Low layer bits also occur in the placed-object models (the meshes of type 29 objects, whose nodes the placed-object draw filters with the world mask word; none of the 3,495 objects on disc has a mask of its own). Of 3,542 type-4 nodes in those meshes, the only ones with low bits are in three packs: pack 758 and pack 1358 have eight, with the values 2, 4, 8, 16, 32, 64, 128 and 256 (one node each), and pack 1023 has eight with the values 8, 16, 32, 64, 128, 256, 512 and 1024. In each of these three modules the world word is rebuilt from eight state words each time a state changes: the value of table entry k (1, 2, 4, ... in packs 758 and 1358; 4, 8, 16, ... in pack 1023) is included while state word k is 0 and removed while it is nonzero, and the whole word is passed to both setters (SPEC-0042, World mask call sites). The node values are twice the table values, so layer k is drawn exactly while state word k is 0. Test vector (pack 1358): state words 0 and 3 are 0 and the others nonzero: the passed number is 1 + 8 = 9, the display word is 18, and only the nodes with the values 2 and 16 are drawn. Test vector (pack 1066, module word 6): display word 12; the nodes with the values 4 and 8 are drawn and the node with the value 2 is skipped. No collision record of packs 758, 1023 and 1358 is gated, so there the collision call changes nothing.

The pack 1299 mask tree (High). The display chunk with id 401 is not part of the level display tree. The world keeps it as a separate small tree with its own mask word, which the pack's looping-door puzzle writes (SPEC-0028, Pack 1299 looping-door puzzle), and draws it every frame the world is drawn. The tree walk uses the same layer rule as the level tree (a type-4 node is skipped with its subtree when its layer flags are nonzero and share no bit with the draw mask in force), but for this tree the draw mask is that private word. The world's display word, written by SetWorldDisplayMask, is not used, and the story layers do not apply. The tree has twelve type-4 nodes with the single layer bits 0x2 up to 0x1000, so exactly one variant draws while the puzzle's mask has one bit set (0x80 at build; 0x4, 0x200, 0x400, 0x20 and 0x1000 for the states 1 to 5). Test vector: mask 0x200 draws only the node with layer flags 0x200; mask 0 draws none of the twelve.

Display pairing for packs 696 and 714 (High). Neither pack has a node, in its world tree or in any placed-object mesh, whose layer flags have a low bit. The call with m = 15 (display word 30, bits 1 to 4) therefore draws and hides nothing in these packs. The gate value 4 that the same module passes to the collision setter acts on four wall rings, not on anything drawn (SPEC-0018, Primitive class bits and mask gate).

### Camera projection (High for the values; High for the 600 and 10560 sets being unused in play)

The camera render record starts from one fixed set of values, set at boot (these add to SPEC-0027, Projection):

| Value | Setting |
| --- | --- |
| Viewport | x 0, y 0, width 512, height 448 |
| Near plane N | 1.0 |
| Clip far planes | 5280, 600 and 10560 (three clip matrix sets) |
| GS projection far F | 10560 |
| GS split | 0.001 |
| Focal length | 512 |
| Depth value at the far plane A | 1.0 |
| Depth value at the near plane B | 1048576 (2^20) |
| Widescreen | off |

- Screen mapping (4:3): x_screen = 2048 + 512 × 0.8 × X / Z = 2048 + 409.6 X / Z; y uses 512 × 448 / 480 = 477.87 in the same way. The field of view is 64.0 by 50.2 degrees; the 512 by 448 buffer is shown as 4:3.
- Depth: Z_buf(z) = k / z + c, with k = F N (B − A) / (F − N) and c = (A F − B N) / (F − N). The near plane maps to B and the far plane to A (larger is nearer). There are two GS variants, over [N, F − 0.001] and [N + 0.001, F]. Every draw in play uses the first; the second is rebuilt with it but is read only by a part-matrix packet builder that nothing calls (High).
- Depth buffer (High for the format, test and write): the main frame's Z buffer is 24-bit (PSMZ24) with depth writes on and the test GEQUAL, so the integer depth values run 0 to 16,777,215 and only the lowest B of them are used (B is 2^20 by default and 300,000 or 100,000 in Meridell; see Meridell mountains and the depth buffer). An equal depth passes, so the later draw wins a tie.
- Clip matrices: for each far plane, a frustum set (640 by 480 basis, focal 512) and a guard-band set (a 4094-unit basis, the GS coordinate range of ±2047 around the centre). All three pairs are rebuilt whenever the projection is rebuilt, each also composed with the view. Only the far-5280 pair is used in play: the world draw, the character parts, the shadow pass and the other part-matrix packets all load it. The composed far-600 pair has no reader, and the composed far-10560 pair is read only by a second part-matrix packet builder that nothing calls (High). The clean side needs only the far-5280 pair.
- The per-area atmosphere render scalar (SPEC-0028, type 41) replaces B when it is not above B's start value, and the projection and clip matrices are rebuilt. It changes only the depth value at the near plane. The vertex program's copy of B (in the clip record below) is not refreshed (quirk; see Per-triangle clipping in the vertex program).

The culling frustum under View and projection build is not this camera projection; it only culls (Open question 84).

### Per-triangle clipping in the vertex program (High for the rules; Medium-High for the clip record values)

This runs only for batches drawn with the part's clip bit set (see Model parts). Its constants come from a clip record built once, when the camera render record is created: the viewport width W and height H (512 and 448), the focal length 512, and the near-plane depth value B (2^20). There are two copies, one per frame parity, and the copy of the frame's parity is sent each frame.

For each strip vertex v that completes a triangle (its no-triangle bit clear), let dx = x − 2048 and dy = y − 2048 be its screen offsets, each multiplied by the sign of its clip w. Two sets of flags are computed:
- **Inside flags:** |dx| ≤ W/2, |dy| ≤ H/2 and w ≥ 1.
- **Guard flags:** |dx| ≤ 2047, |dy| ≤ 2047 and w ≥ 1.
No far test is made in either set.

Then for the triangle (v − 2, v − 1, v):
1. All three vertices inside: keep it.
2. All three outside the same one of the five planes (left, right, top, bottom, near): drop it by setting the no-triangle bit on v.
3. Any vertex failing a guard flag (outside the ±2047 guard band, or nearer than w = 1): clip it.
   - The original triangle is dropped.
   - The polygon is clipped in homogeneous space against, in this order: w = 1, x = 2048 − W/2, x = 2048 + W/2, y = 2048 − H/2, y = 2048 + H/2 (that is x 1792 to 2304 and y 1824 to 2272).
   - S·w, T·w, w, the colour and the homogeneous depth are interpolated linearly along the cut edges, then divided by w again.
   - The polygon is drawn as a triangle fan (GS primitive type 5) after the batch.
4. Otherwise: keep it; the GS scissor trims it.

The homogeneous depth of a clipped-polygon input vertex is Z·w when |w| > 1, and B when |w| ≤ 1. This is the only use of B by the vertex program; it is not a fog constant. B is the clip record's copy taken at creation: when the atmosphere scalar lowers B (Camera projection), this copy keeps 2^20 (quirk), so near-plane vertices of clipped triangles keep the old near depth. The viewport never changes size in play (High): the display mode is chosen once at boot and is always 512 by 448 on this disc (SPEC-0027, Screen size), and the projection changes made in play (the atmosphere scalar and a camera track's focal length) leave the viewport size alone. So W/2 = 256 and H/2 = 224 stay valid for the whole game.

There is no far clip and no back-face test.

Clean side: hardware clipping gives the same image, except for the stale near depth of near-clipped vertices (negligible) and the absent far clip.

### Volume test (Medium-Low; inactive on disc)

A second, separate test rejects a box when all eight of its corners fail the same runtime volume. A volume is four planes. The list begins empty on disc. In the known boot image, neither the builder nor the volume-test routines have a direct caller in the call index, so no populated volume list or active volume rejection is established for this image. If called, the builder sorts up to 32 candidates (the cap is the smaller of 32 and the stored cap of 14 on disc) and builds one volume per candidate with a usable transform. Unindexed or external module calls remain outside this static boundary.

The clean side should treat this test as absent unless a later spec shows a level that fills the list.

### Light slots and the lit draw (High for the slots and the light block; Medium-High for the vertex arithmetic)

A light context has four slots (see Layout). A model draw may pass a light context or none. A draw that passes one is lit from it: a scene light context (built fresh for the draw from the scene default lights, see Lights) for world items, the HUD coin, the 3D book, the item showcase and the test shape, or the character's own context for a character model and the held items of the hero and of world-table characters (see Character lights). A draw that passes none sends no light block (see the end of this section for the full list of both kinds and for what a draw without one uses). World objects of the record lists and effect models are in the second kind. Slot kinds: a slot starts with kind 1; its first colour fill replaces the stored colour and sets the kind to 0, so every later fill of a persistent context (a character's) eases 5 percent toward the new colour, unless a dynamic light took the slot in this walk (its second value marks this; then the fill replaces). Scene light contexts are rebuilt for every use, so their fills always replace. There is no light loop or falloff in the functions that set the slots; the falloff is part of the weight.

When a lit draw is prepared:
1. For each slot with an entry: new = the entry's colour (all four lanes) × the slot weight w.
   - If the slot's kind is 0 and its second value is 0, the stored colour moves 5 percent of the way toward the new value: stored' = 0.95 × stored + 0.05 × new. The weight 0.05 is the data word above; the rule is the same as stored + 0.05 × (new − stored).
   - Otherwise stored' = new, and the slot's kind becomes 0.
2. Slot 0 is the ambient light: its stored colour is the ambient colour A. Its direction is not used.
3. Slots 1 to 3 are directional: C_k is the stored colour, and D_k is the light's travel direction in the drawn model's space:
   - type 2: the entry direction, rotated by the transpose of the model's 3 × 3 rotation;
   - type 3: normalize(model position − light position), rotated the same way;
   - any other type: zero, so the slot adds nothing.
4. The light block (see Layout) is built from A, C_1 to C_3 and D_1 to D_3 and sent with the draw. A block is not sent again when it is the same block record as the last one sent: the test compares the record's identity, not its contents. Each light context owns two block records, one for each frame parity, so the first lit draw of a frame always sends its block, and repeated draws from one record in a frame send it once. The remembered last record is never cleared by a frame, a world change or a mode change.

Weight w: the entry strength for types 1 and 2; the falloff of Character lights for type 3.

The vertex program of a batch is the pass's material program word (SPEC-0003, Material record; SPEC-0004, Vertex program), not the mesh layout.

Which program serves which draw (High; the programs are the seven paths of the vertex-program word, the rest of the word values draw nothing):
- World geometry and sky domes: program 0 (unlit, layout B), with program 1 (lit, layout C) and program 4 (unlit, layout C) for the lit and shaded pieces. The UV modes (environment map, scroll, flipbook region) are separate handlers run before the batch's vertices, chosen per pass by the material's UV mode.
- Characters, including Tor and Roberta and every skinned outfit: program 2 (three-bone blend, lit) and program 6 (the same plus the specular alpha term), which only some outfits use (Specular alpha term of program 6; Tor's starting outfit has no program-6 part). Both read the bone palette uploaded with the draw.
- Camera-facing sprites: program 3. Its users are the particle sprite batches (SPEC-0034 section 16), the far foliage sprites (SPEC-0060) and the billboard material binding (SPEC-0003). It reads three vectors per sprite, takes the colour unchanged, ignores the light block and the draw colour scale, and produces one four-corner strip per sprite (SPEC-0003, Billboard program 3).
- Effects drawn as meshes use the same programs 0, 1 and 4 with a UV mode; no program is specific to effects.
- Program 5 (lit, mask-driven slots) is never selected on disc.
No program tests triangle winding, so every mesh is drawn double-sided, and every program that skins or lights does so per vertex. There is no per-pixel work, no fog and no far clip.

Lit vertices (programs 1, 2 and 6, and 5, which is unused on disc), with n the vertex normal in model space (for programs 2 and 6, the skinned normal n' of SPEC-0004):
1. For k = 1, 2, 3: d_k = clamp(n · T_k, 0, 1), with T_k = −D_k / |D_k|. There is no bias term.
2. L = A + d_1 C_1 + d_2 C_2 + d_3 C_3, per RGB channel, in 0 to 255 units.
3. RGB out = min(255, L × (vertex colour byte / 16) × s_rgb), truncated to an integer.
4. Alpha out = min(255, 128 × (vertex alpha byte / 16) × s_a), truncated. Program 6 may replace it with its specular alpha (below).

Unlit vertices (programs 0 and 4; layout B, and layout C drawn with program 4): out = trunc(vertex byte × s) per channel, with 128 = 1.0 and no upper limit. The GS keeps the low 8 bits of each lane, so a value of 256 or more wraps modulo 256 (see Edge cases).

s is the draw colour scale (see Fade level quantizer and draw colour scale). The GS then modulates the texture by out / 128, so 128 leaves the texture unchanged. The lit colour L is therefore in GS units: L = 128 with a colour byte of 16 and s = 1 leaves the texture unchanged. The skinned lit programs 2 and 6 use the same block and formula on the skinned normal (SPEC-0004, Skinning and lighting in the vertex program); the far state uses the first bone slot alone for both position and normal.

Brightness factor (High). Whenever a light block is built, a factor f is computed and stored in lane 4 of the block's first vector. For each of A, C_1, C_2 and C_3, take m = max(red, blue); the green lane is ignored (original quirk). Let K be the number of nonzero m and S their sum.
- If K = 0: f = 0.3.
- Otherwise b = S / (128 K), and f = 0.3 when b < 0.2, f = 1.0 when b > 0.6, and f = 0.3 + 1.75 (b − 0.2) otherwise.

The bounds 0.2 and 0.6 and the values 0.3 and 1.0 are the data words 0x2ECAE4, 0x2ECAE8, 0x2ECAEC and 0x2ECAF0 (see Where it lives). Only program 6 reads f.

Specular alpha term of program 6 (High for the arithmetic and for the frame of the reference point, which is the camera position minus the character's world position while the skinned positions are in world space: Open question 92; Medium for the exact X > 0 test, which the program makes with a flag test on the sign and zero bits of X). It applies in the near state only (far bit clear). There is no header flag; the term is inert when both material lanes below are 0.
1. Let R0 be the reference point (the view point sent with the draw's matrix record, in the same space as the skinned positions) and V̂ = normalize(R0 − P), with P the skinned position.
2. Let L′ = −(T_1.x, T_2.x, T_3.x), the negated first vector of the transposed light block (original quirk: the x components of the three light directions, not a light direction).
3. c = 2 (n′ · V̂)(n′ · L′) − V̂ · L′: the reflection of V̂ about n′, dotted with L′.
4. Let p be the float at group offset 0x6C and z the float at group offset 0x68 (lanes 4 and 3 of the pass-0 UV-mode data, SPEC-0003; the UV mode is 0 in every program-6 group). Let I(x) be the 32-bit pattern of float x read as a signed integer, and F(i) the reverse. Then X = F(trunc(p · I(c) + z)), truncating toward zero and saturating to the 32-bit range. The conversion of I(c) to a number, the product and the sum are all single-precision float operations (the pattern loses its low bits above 2^24, a relative error of about 6e-8 on X), which does not change the visible result. This approximates g · c^p with g = 2^((z + (p − 1) · I(1.0)) / 2^23).
5. If X > 0, the output alpha is replaced by A = trunc(255 · f · min(1, X)). Otherwise (X ≤ 0, which includes every c ≤ 0 and p = z = 0) the normal lit alpha is kept.

On disc, all 146 program-6 groups have p from 3.0 to 13.59 and a negative z; for example resource 506 group 12 has p = 3.0 and z = −2134851328.0. Their GS state is: texture on with texture alpha used and texture function 3 (highlight 2) in all 145 textured program-6 groups on disc (the one untextured program-6 group is in resource 1260); the blend (Cs − Cd) × As + Cd; alpha blending on in the primitive; the alpha test "greater or equal 120" with the framebuffer-only fail rule; per-pixel alpha blending off (it is set to 0 once at start-up). With highlight 2 the fragment colour is texel × vertex colour / 128 plus the vertex alpha, per channel (clamped to 255), and the fragment alpha is the texel alpha. So the vertex alpha of program 6 is an additive highlight and is never the opacity: the surface is drawn solid wherever the texel alpha is 120 or more (128 in every palette entry the hero head leaves use), and the specular term A brightens it by A on each channel (High). The base alpha byte of every vertex of the program-6 leaves in the hero outfits 506 to 518 is 0 (Roberta's head dome, material group 12 of outfits 512 to 518, among them), so those surfaces get no highlight where c ≤ 0 and show their plain lit texture (see SPEC-0004, Roberta's head). Test vector: texel (29, 30, 35, 128), output colour (128, 128, 128), output alpha 0 gives the pixel (29, 30, 35) with depth written; output alpha 100 gives (129, 130, 135). A texel alpha below 120 would not write depth and would blend by the texel alpha (no such texel in the hero program-6 textures).

The pixel (from the code; not run in the original; High for the arithmetic, Medium-High on the clamp). The part is drawn in one pass with its own texture. On the pixel, each of R, G and B equals min(255, texel × lit colour / 128 + A), where A is the output alpha of steps 1 to 5 (the same number on all three channels, 0 when X ≤ 0), and the pixel alpha is the texel alpha. The texel alpha 128 passes the alpha test and gives a blend factor of 1, so the pixel is the solid colour above and the vertex alpha is never an opacity. Colour clamps at 255 (Medium-High on the clamp). A is interpolated across each triangle with the vertex colour. In a rewrite, one textured pass is enough. Test: texel (29, 30, 35, 128), lit colour 128, A = 132 gives (161, 162, 167).

Who has the term (High). Tor's starting outfit (model 504) has no program-6 part, nor do 505 and 511, so the highlight does not appear on Tor at the start of the game; it appears on models 506 to 510 (items 121 to 125): 506 groups 12 and 14; 507 groups 11, 12 and 14; 508 group 5; 509 groups 3, 4 and 5; 510 groups 6 and 12. Roberta's models 512 to 518 have group 12 only. p is 3.0 and z is -2134851328.0, except 510 group 6 (p 3.212, z -2359929344.0) and 510 group 12 (p 3.0, z -2137614720.0). One of the 146 program-6 groups on disc has no texture (resource 1260, nested group 11).

Which model draws pass a light context (High for the call sites; Medium for the role names):
- With a scene light context (the four scene default lights put into a fresh context for each draw; every fill replaces): world items (SPEC-0021), the HUD spinning coin (HUD coin model and camera-facing quads), the 3D book of the front end (SPEC-0035), the item showcase draw (SPEC-0021) and the one test shape.
- With the character's own light state (its four-slot state of Character lights): the held items of the focus hero (the models in its hands) and the main attachment model (slot 0) of each world-table character.
- With none: the thief's stolen item, the slot 6 and slot 1 attachment models of world-table characters (Held items and character attachments), the fixed array of ten projectile records, the model particles and model effects of SPEC-0034 (burst emitters, the distance-faded effect list and the effect pool draw), the world record lists (props, debris and display entries), the inert grid object, the aim-view model drawn by the ground render pass, and the small draw methods of world object classes that draw a model by the common model draw (each behind a distance test).
- Character meshes use the character part draw (Character part draw and the skin palette), which lights from the character's own state, and level geometry (layout B) is drawn with the unlit programs; neither is in the lists above.

A draw with no light context builds and sends no light block (High). The vertex program keeps the last block it was sent in its data memory, across draws, frames and world changes, so a lit draw without a context is lit by the block of whichever lit draw reached the vector unit before it in packet order (order-dependent: queued draws go out at the flush in material-bucket order, directly drawn ones go out at once). Only lit materials (programs 1, 2, 5 and 6) read the block; unlit materials and the billboard program never do, so only lit parts drawn without a context are affected. Before the first block of a session the memory holds whatever the machine left there; the game never sets it. In the second walk of the attachment slots a character's slot 0 draw (its own light state) comes first, then its slot 6 and slot 1 draws, which pass no context: for a character that drew slot 0 they are lit by its own block, and for a character with slot 1 only (the bandit family) or for a thrown weapon they are lit by the block of the previous lit draw, usually the previous character's slot 0 or the last lit draw of the world flush. Default if unresolved (rewrite choice): keep one current light set that every draw with a context replaces and every lit draw without a context reuses, in submission order, and start it from the scene default lights; do not light such draws from the scene default lights every time.

Character mesh path (Medium-High). A character model is drawn by one of two paths, chosen per render object, and both use the character's light block. The vector-unit-1 path is the one described above. The other path runs on the main processor and the first vector unit: the light block is loaded into the first vector unit, and a small vertex program (its microcode is a data blob at data address 0x2FB9A0) computes, per vertex:
1. n' = R_1 n, with R_1 the rotation of the vertex's first bone.
2. d_k = clamp(T_k · n', 0, 1) for k = 1, 2, 3, and L = A + d_1 C_1 + d_2 C_2 + d_3 C_3 (the same light formula as above).
3. RGB out = min(255, L × vertex colour byte / 16), truncated; alpha out = 128. No draw colour scale is applied in this program.
4. Position = w_1 M_1 p + w_2 M_2 p + w_3 M_3 p, the weighted sum of the vertex's three bone transforms (SPEC-0005).
No render object on disc takes this path: it is chosen only when the first child of the model container is a type-12 chunk, and every container on disc starts with a type-13 child (Open question 73, Resolved).

Reach: type 30 light sets reach characters only. World geometry (the level display trees, layout B) is unlit. The scene default lights light every draw that passes a scene light context (list above). The swing-trail colour gradients of SPEC-0034 section 18 (once called light templates) never enter this path.

Coordinates: model and world spaces are right-handed with +Y up (SPEC-0004, Coordinate conventions); the light directions and normals use the same axes.

The light packet allocator reports a fault when its pool is full. Each view has a pool of 64 light blocks.

Test vectors are in Test vectors (Lit draw and Light slot blend).

### 3D book open angle (High)

There is no blob shadow. The draw that writes a fixed value 127 and steps a value down to a floor of 0.12 is the 3D book draw (SPEC-0035, books):

- Each book draw sets the value 127 into two register entries of every material group of the book's shader list (the groups are 0x270 bytes apart, SPEC-0003).
- The book's open angle a starts at 1.2 (data word 0x2DB2E0) and falls by s = (1.2 − 0.12) / 75 = 0.0144 per draw. If the new value is below 0.12 (data word 0x2DB2E4), it is set to 0.12, so the angle stops at 0.12 after 75 draws.
- s is computed from the two data words when the book is set up; it is not a separate stored value.

The book is drawn after the characters and effects (Per-frame draw order, step 9).

### Character shadow volumes (High unless marked; High for the visible size of the darkening (8 per channel); no rendered comparison made)

Character shadows are stencil shadow volumes; there is no other kind of shadow. Each shadow-casting unit is one bone-attached hull from the model's type-33 block (SPEC-0004, "Optional per-model block (type 33)"). The steps below run as one pass per frame, while the data word 0x2ECCD0 is nonzero (1 on disc). The pass is wrapped in a render-state bracket unless the data word 0x2ECCD4 is nonzero (0 on disc).

1. **Choosing casters.** Candidates:
   - the hero, always;
   - petpet slot characters, only while the data word 0x2D7608 is nonzero (0 on disc, so petpets cast no shadow);
   - current-world slot characters whose spawn record kind key is 720. On disc only the two Gnarfas placements (character regions 985 and 1264) have kind key 720.

   A candidate is taken only when its 3D squared distance from the view point is below 2500 (50 units). While fewer than three casters are held, it is appended. Otherwise it replaces the held caster with the largest distance among those farther than it; if none is farther, it is dropped. So the pass keeps the three nearest casters within 50 units; a candidate at the same distance as the farthest held caster does not replace it.
2. **Moving the hull.** For each unit of a chosen instance, the sphere centre, the hull points and the triangle normals are moved into world space. The matrix used is the bone matrix that the unit's bank index names (SPEC-0004, Optional per-model block: the byte is one bone index across the render object's three bone-matrix arrays) combined with the instance's matrices. Normals take no translation.
2a. **Light direction** (one per caster, per frame). It comes from the caster's own character light context (the four slots that light its model; see Character lights):
   - S = the sum of w_k × D_k over the slots holding a type 3 light, plus w_j × D_j for the one type 2 slot with the largest weight (the first such slot on equal weights). w is the slot weight and D the slot's travel direction: the entry direction for type 2, normalize(character position − light position) for type 3. The ambient slot is not used. So one direction per caster, made from all its point lights and its strongest directional light.
   - L = normalize(S). If L.y > −0.4 (data word 0x2EC9E4), L.y is set to −0.4 and L is normalised again. A zero S therefore gives (0, −1, 0).
   - If any slot of the context is empty, or the context has no owner, the direction is (0, 0, 0) and no limit is applied. A character's context always has four full slots after its walk (Character lights), so this does not occur in play.
   - When the data word 0x2ECCCC is nonzero (0 on disc), L = (0, −1, 0) for every caster.
   - The original also eases this direction toward the previous frame's direction, but the previous direction is not kept, so the ease has no lasting effect (see Edge cases). The clean side uses L as stated.
3. **Facing.** For each triangle, f = (moved normal) · L. When f ≥ 0 the triangle faces away from the light, and its facing flag is set to 1. When f < 0 it faces the light, and its flag is set to 0. The flag is rewritten every frame.
4. **Extrusion.** E = 16.0 (data word 0x2ECCC8). The 25.0 that the instance stores beside its shadow block pointer is never read.
   - The unit's sphere centre C is the unit centre plus the translation of the unit matrix (the centre is not rotated). Its radius R is the unit radius plus the fourth value of the matrix's translation row (1 for an ordinary matrix).
   - The swept segment runs from P0 = C to P1 = C + E L. It is clipped against the camera's five culling planes (SPEC-0027; plane normals point into the view). [Medium-High for the plane roles]
   - Front plane (the first of the five): let d0 and d1 be the signed distances of P0 and P1. If both are below R, the unit is skipped. If one is below R, that end moves along the segment to the plane offset by R: by t = (R − d_low) / (|d0| + |d1|) × length + 0.001, with d_low the distance of that end and length the current segment length.
   - Each of the four side planes, in turn: if both ends are below −R, the unit is skipped. If one is, that end moves inward by t = (R + d_out) / (|d0| + |d1|) × length + 0.001, with d_out the distance of that end. The length is recomputed after every clip.
   - With P0' and P1' the clipped ends: A = P0' − C and B = P1' − P0'.
   - Near copy of a hull point = its moved position + A. Far copy = near copy + B. For an unclipped segment A = 0 and B = 16 L.
5. **Silhouette edges.** For each edge of the unit, the facing flags of its two triangles are compared. Where they differ, the edge is a silhouette edge. With Pa and Pb the near copies of its two hull points, one quad (Pa, Pb, Pa + B, Pb + B) is emitted as two triangles (the shadow wall). An edge whose two triangles agree emits nothing.
6. **Caps.** Every triangle is emitted exactly once: a triangle with flag 0 (facing the light) with its three near copies, as part of the near cap; a triangle with flag 1 with its three far copies, as part of the far cap.
7. **Count colours.** Every emitted polygon has one colour for all of its vertices. It is +8 (R, G, B = 8, A = 128; data word 0x323368) when the camera is on the polygon's outward side, and otherwise 248, which is −8 modulo 256 (R, G, B = 248, A = 128; data word 0x32336C). With V the camera position:
   - Cap triangle: outward when (V − P_0) · N ≥ 0, with N the triangle's moved normal and P_0 its first near copy.
   - Wall of edge (Pa, Pb): e = Pa − Pb when the edge's first triangle has flag 0, else e = Pb − Pa; N = e × L; outward when (V − Pa) · N ≥ 0.
8. **Shadow buffer and resolve.** One off-screen buffer of screen size with 16 bits per pixel (5 bits per colour channel and 1 alpha bit).
   1. Clear its colour to (120, 120, 120) with the alpha bit set.
   2. Draw all volume polygons, depth-tested against the scene depth (a polygon passes where it is nearer than the scene surface, that is where its depth value is larger), with no depth write. Blend: out = Cs × As / 128 + Cd with As = 128 and no clamp, so each channel wraps at 256. Each pixel then holds (120 + 8 k) mod 256, with k the number of camera-facing volume polygons in front of the surface minus the number of the others.
   3. Write 0 to every pixel keeping only the top bit of each channel. A channel becomes 128 when (120 + 8 k) mod 256 ≥ 128, that is when k mod 32 is 1 to 16, and 0 otherwise.
   4. Draw the buffer over the frame as a full-screen texture: Cs = texel (5 bits expanded to 8) × vertex colour / 128, with the vertex colour (R, G, B = 8, A = 32; data word 0x2D7610). The alpha test passes above 1. Blend: frame = max(frame − Cs, 0) per channel. Net effect: the frame's R, G and B fall by 8 where shadowed. [High]
   - Texel alpha. The game sets the GS texture-alpha register once, in its display set-up, to 0 for a clear alpha bit and 128 for a set one (no expansion of colour-0 texels), and nothing changes it later. The buffer's alpha bit is set everywhere (the clear sets it, every volume polygon writes alpha 128, and step 3 keeps it), so each texel reads alpha 128 and the drawn alpha is 128 × 32 / 128 = 32, which passes the test. Unshadowed texels have colour 0 and subtract nothing.
   - Darkening size. A kept channel is 16 of 31 in 5 bits, which reads as 128 in 8 bits, so Cs = 128 × 8 / 128 = 8. The resolve colour is written into the packet from data word 0x2D7610 before every send (the packet's own default of 32 per channel is always overwritten), and no code writes that word. So a shadow is a uniform drop of 8 out of 255 per channel (about 3 percent), never 32.
   - When the data word 0x2ECCD4 is nonzero (0 on disc), steps 1, 3 and 4 are skipped, so no shadow shows.

Units with no edges, triangles or hull points (the empty units) add nothing. A unit whose swept sphere lies outside the view (step 4) adds nothing either. Confidence: High for steps 1 to 8 and the 50-unit and three-instance limits; Medium-High for the roles of the five planes; High for the visible size of the resolve (no rendered comparison made). Test vectors are in Test vectors (Shadow volumes).

**Gnarfas's hulls (the only non-hero caster; High).** Both of his regions (pack 985's embedded block and region 1264) carry the same hull block, 25,776 bytes of payload: 40 units, 640 edges, 516 triangles and 338 hull points in all. 38 units are 8-point, 12-triangle skewed boxes with 12 to 17 edges each; one unit on bone index 20 is a 26-point, 48-triangle, 72-edge hull (the head or jaw), and the other unit on bone index 20 is an 8-point box. His skeleton has 62 main joints and no sub-part skeleton, so every bone index (all below 62) names an entry of the main pose array; the used indices are 0 to 12, 14, 20 to 26, 28, 34, 35, 37, 42, 43, 45 and 50 to 60 (index 20 twice). No unit is attached to any of the 18 chain joints (15 to 19, 29 to 33, 38 to 41 and 46 to 49): the four hanging chains swing by their secondary motion (SPEC-0005, Secondary-motion chains) but their shadows do not. The hulls nearest the chains sit on the chain anchors, joints 14, 28, 37 and 45 (the parents of the first link of each chain; units 38, 39, 36 and 37), and move only with the animated pose of those anchors. Joints 13, 27, 36, 44 and 61 also carry no unit. Each unit is moved by its bone's current pose matrix exactly as in step 2, with no scaling.
- Test vector: unit 38 (bone index 14) has sphere centre (0.5032, -0.2572, 0.0531) and radius 1.0778 in bone space; its hull point 0 is (-0.1129, -0.3334, 0.5078), point 1 (0.9552, -0.9152, 0.6644), point 2 (1.2400, -0.1072, 0.6630); triangle 0 has normal (-0.1212, 0.0444, 0.9916) on points 0, 1 and 2; edge 0 joins triangles 0 and 6 at points 0 and 1. Unit 24 (bone index 20) has centre (0.0, 0.0399, 0.5726), radius 0.8993 and first point (-0.4690, -0.5516, 0.9670).

### Display base size scale (High for the rule; Medium for use)

The display setup takes a scale factor f.

1. If f is below 0.1, f becomes 0.1. If f is above 1.0, f becomes 1.0.
2. The display base size is 512 by 448. The stored width is 512 times f, and the stored height is 448 times f, each truncated toward zero to a whole number. They are stored in the display record and passed to the display-size setter.

Test vectors: f = 0.05 gives f = 0.1 and a size of 51 by 44. f = 1.5 gives f = 1.0 and 512 by 448. f = 0.5 gives 256 by 224.

### Fade level quantizer and draw colour scale (High)

The vertex program holds a per-draw colour scale s = (s_r, s_g, s_b, s_a) (SPEC-0004, vertex colour). It is (1, 1, 1, 1) at start-up. It is changed by colour-scale records (see Layout), which replace only their masked lanes.

Material draw list. Mesh leaves are collected each frame into a draw list, in buckets by material. The buckets are flushed in the order their shader lists were registered. Within a bucket there are four sub-lists chosen by the pass byte of the most recently entered type-4 node (SPEC-0004, Drawing a node tree, 3b); sub-list 0 is drawn first. Each entry carries a colour: the draw context colour k with its fourth lane replaced by the opacity o, and its light block. When the list is reset, the opacity is 1.0, the context colour is the default draw colour (0.5, 0.5, 0.5, 1.0) and no light block is set. At flush a new colour scale is sent only when an entry's colour differs from the previous entry's.

Colour scale of an entry:
- If k's red, green and blue all lie within 0.49 to 0.51 (as with the default colour): s = (1, 1, 1, a'), where a' is the opacity table value for o (variant A below).
- Otherwise: s = (2 k_r, 2 k_g, 2 k_b, o) on all four lanes.

Opacity o = master opacity × f. The master opacity is 1.0 unless a caller sets it (the sky dome's role-10 layer does; see Sky domes). f is the distance fade of the nearest type-4 node above the leaf (SPEC-0004, node fields F at +0x58 and E at +0x64). With q = sqrt(max(0, |centre − eye|² − R²)), the distance from the eye to the node's bounding sphere: f = (E − q) / F when F > 0 and E − F < q < E, and f = 1 otherwise. A type-4 node inside another type-4 node multiplies its f by the outer node's f.

Quantizer. A level v (a real number) is quantized in two variants. Both use the same index arithmetic, and each has its own range bounds and its own table of 257 entries. Each entry is 48 bytes (0x30) and is a colour-scale record. The tables and their bounds are built at start-up.

Common steps:

1. Clamp v to the range [lo, hi] of the variant's bounds. A value above hi becomes hi, and a value below lo becomes lo.
2. Multiply by 256.
3. Add one half if the product is zero or positive, or subtract one half if it is negative (round half away from zero). Truncate to a whole number.
4. Clamp to the range 0 to 256.

The result is an index from 0 to 256 into the variant's table of 257 entries. When the index differs from the last index used, the entry is applied; when it is the same, nothing happens.

Variants:

- **Variant A, the opacity table** (the table starting at 0x394170; bounds lo = 0.008 and hi = 1.0). Entry i is the record (1, 1, 1, 0.008 + 0.992 × i / 256) on all four lanes. So a' = 0.008 + 0.992 × i / 256 with i = round(256 × clamp(o, 0.008, 1.0)). A negative level selects no index: it sends one separate fixed record, (1.2, 0.5, 0.5, 1.0) on all four lanes (a reddish tint). That reset record is at data address 0x39A1F0 and is built at start-up with the tables.
- **Variant B, the darken table** (the table starting at 0x3971B0; bounds 0 and 1.0). Entry i is the record (i / 256, i / 256, i / 256, 1); its lane mask covers red, green and blue only, so it sets those three to i / 256 and leaves alpha unchanged. It has no negative branch. Nothing calls the variant B quantizer (no call, jump or stored pointer in the boot program), so no draw is ever darkened through this table; it is built at start-up and unused (High). A separate helper that nothing calls would send entry 256 of both tables (full strength, white): the same restore as sending level 1.0, so it adds no behavior (High). The clean side need not build it.
- **Variant A callers** (High): the shadow pass and the world draw each send level 1.0 (full opacity) before they draw, and the material draw sends each entry's opacity o, or the default colour scale's fourth lane (1.0) when it restores the default after a list. An entry's opacity is the colour scale given when its object was submitted. The only submitter that gives a negative scale (−1.0) is a 5 by 5 debug grid object whose enable word is never set, so in play no entry is negative and the reddish reset record is never sent (High). The other variable scale, the radial fade of a record list, is (outer radius − distance) / inner radius and is used only inside the outer radius, so it is above 0.
- **Foliage plants** (High): a plant drawn with its near model goes through the material draw with the default colour k = (0.5, 0.5, 0.5) and the near weight as opacity, so s = (1, 1, 1, a'); the plant's colour word is not read there. The colour word is used only by the far sprite batch, which is a ready-made packet and takes no colour scale; its sprites use the MODULATE texture function, so a colour byte of 128 is full brightness and 64 grey is half (SPEC-0060, Behavior 4, Plant colour in the draw).

The reset that clears both tables (when the game sends its reset sentinel) first writes a vector of four 1.0 values at a fixed address before the first table, then clears each of the 257 entries of both tables and the separate entry. The four-1.0 vector is the default colour scale (Medium).

Test vectors: v = 0 gives 2 in variant A (raised to its low bound 0.008 first) and 0 in variant B. v = 0.5 gives 128. v = 0.998 gives 255 (255.988 truncated). v = 1.0 gives 256. v = 1.5 gives 256. For v = −0.001, variant A sends the fixed (1.2, 0.5, 0.5, 1.0) record, and variant B gives index 0. More are in Test vectors.

### Frame flush entry points (Medium for the storage; Low for the flush)

Four entry points each store a view or transform block into fixed frame globals, copying it from fixed offsets of the caller's records (about 0x60 bytes in total), and then call a flush routine (described under "Flush"). They differ as follows:

- Two of them store the caller's count into the current draw list, when a current list exists and the count is not zero (for the first) or always when a list exists (for the second), and then flush.
- The other two first make the list of the caller's record the current list, when that record names one. Each then flushes only when that list holds records.

Answered in part: The world draw is a confirmed user of one list-setting entry; it supplies an identity matrix and the camera position. An object-tree draw uses a count-storing entry and supplies a model-to-world matrix and eye position. The copied block is therefore caller-selected and is used for both world-camera and per-object draws; it is not inherently camera-only. The remaining list-setting entry has no identified direct caller, so its use and the complete caller map remain open.

### Default colour word (High)

Two kinds of object have their own default, and each keeps it:
- 2D draw records (draw rectangles and the 0x30-byte draw records) start with the colour word 0x80808080: three colour bytes of 128 and a fourth byte of 128, which is GS alpha 1.0 (opaque). One of the two initializers also sets a scale of 16 units.
- The view object holds the background colour, which is the atmosphere zone's vector B (the screen overlay colour, SPEC-0028 section 11). It starts as 128, 128, 128 and a fourth byte of 0. The camera volume blend later writes all four bytes from the volume's colour, each rounded half away from zero (SPEC-0027). Its fourth byte is the source-alpha weight (alpha / 128) of the late full-screen tint of step 10 draw 2, which is drawn only when that byte is nonzero; the blend (Cd − Cs) × Ad + Cs, with Ad the destination alpha already in the frame buffer, belongs to the frame-copy draw before it and does not use this colour (High).

### Fog (High)

The game has no fog. Neither vertex-program state set enables fog, and the mesh GIF tags have the fog enable bit clear (PRIM 0x25C, 0x24C and 0x05C), so the GS fog stage is never used. The only fog-related text is the name of a source file in a text string attached to a texture error routine. The clean side must not implement fog.

### Main-character bounds (High for the use; Medium for the clamp details)

Before the main character is submitted, its four screen bounds (minimum x, maximum x, minimum y, maximum y, in screen units) are recomputed from its model through the matrix helper and stored on the character. A rectangle-merge routine then combines these bounds with a running screen rectangle:

- If the running rectangle is empty, it is replaced by the character's rectangle.
- Otherwise the running rectangle grows to the union of the two.

The screen coordinates use the same centre convention as SPEC-0027 (screen centre 2048). The rectangle is stored on the character's fade record and is the region that the fade wrap composites back as one sprite (see Translucency).

### Vertex pass (Resolved: the shadow-unit transform; High for its use, Medium for the size)

This pass and the edge clip below belong to the character shadow volumes (Character shadows, steps 2 and 4), not to ordinary mesh drawing: the caller is the per-caster unit walk, and it passes the extrusion length of step 4. A type-33 unit has at most 50 hull points, so the scratch size of 128 is never reached; ordinary batches (SPEC-0004) do not go through it. A vertex routine transforms up to 128 positions per pass by a 4 x 4 matrix (each position with its fourth component equal to 1). The results go into a scratch area holding up to 128 positions. A second loop adds a translation to a second copy. Then edges are tested against four side planes (see Edge clip below).

Edge clip (Medium). For one edge, given as two end points, against four planes on the object, each at a limit value:

- For each plane that the edge crosses, the cut point is at a fraction t of the edge, where t is (signed distance of the near end minus the limit) divided by (sum of the absolute distances of both ends), and the cut point is then found by moving that fraction of the edge length along the edge direction plus a bias of 0.001 units (the single-precision value 0.001, added to the distance after it is scaled by the edge length, not to the fraction).
- An edge outside any one plane at both ends is rejected (outside). Otherwise some part survives (visible).

The mesh data that feeds this routine is the hull points of a type-33 unit (SPEC-0004). The clip of the extruded segment in shadow step 4 is this routine's clip; it uses the camera's culling planes (the front plane, then four side planes), and the volume test above is a different test.

### Colour sheet (UI node kind 8; never drawn on this disc)

Confidence: layout High; conversion and pad rules High; blocks C and D Low (only their entries are described); never drawn in the shipped game High.

Reachability (High). Kind 8 is a value of the node kind byte (+0x2B) of the UI node tree that the front end and the HUD draw (SPEC-0035, node-kind draw dispatch; SPEC-0023). The only UI database on the disc is the type-38 chunk of resource 532, and its 1,366 nodes have kinds 0 to 6 only (SPEC-0035, node types on disc). No node of kind 8 exists, so the colour sheet is never drawn and the player never sees it. The clean side does not need it for the game; the rest of this section describes the code path only. Test vector: walking all 1,366 nodes of resource 532 gives no kind 8 node and no colour sheet quad.

The colour sheet is UI node kind 8. When the node tree visits it, the node draws screen-space quads with a colour per vertex (no texture). It has two blocks that are always drawn, A and B, and two more (C and D) that overlay them at the same origins.

Colour state. The three indices (Layout table) define the colour. They are set when the level's colour record is loaded (a reload flag):

- if the record's fourth word is zero, the record holds three colour bytes (R, G, B). Convert them to hue (degrees), saturation and value with the rule below, then set H = round(hue / 2), S = round((saturation − 1) / 4) − 1 and V = round(value / 4);
- otherwise the record holds three numbers for hue, saturation and value, used directly with the same derivation.

Rounding is half away from zero.

RGB to hue, saturation and value. Value = max(R, G, B). If value is 0, saturation is 0 and the hue is the grey marker 361. Otherwise saturation = (max − min) × 256 / value (truncated). Hue: 0 when max = min; when R is the maximum, hue = 60 × (G − B) / (max − min); when G is the maximum, hue = 120 + 60 × (B − R) / (max − min); when B is the maximum, hue = 240 + 60 × (R − G) / (max − min); a negative hue gets 360 added. Integer divisions are truncated.

Pad steps. Each update, a pad pair steps S (clamped to 0 to 63), another pair steps H (clamped to 0 to 180) and a third pair steps V (clamped to 0 to 63). Which physical button each pair is is not in this spec (Open questions).

HSV to RGB, used to colour the quads. Hue in degrees, reduced modulo 360 and split into six 60-degree sectors; saturation and value are bytes (saturation 255 is full, value 252 is the top of the value range). This is the standard conversion done in integers with truncation (High). With h the hue reduced modulo 360, the sector is floor(h / 60) and the in-sector fraction is f = floor(256 x (h / 60 - sector)), 0 to 255 (computed as the hue shifted left 16 bits, divided by 60, taking bits 8 to 15 of the fraction). With S the saturation byte and V the value byte, the three working values are p = floor(V x (255 - S) / 255), q = floor(V x (255 - floor(S x f / 255)) / 255) and t = floor(V x (255 - floor(S x (255 - f) / 255)) / 255), every division truncating. The outputs (red, green, blue) are, by sector 0 to 5: (V, t, p), (q, V, p), (p, V, t), (p, q, V), (t, p, V), (V, p, q). Do not round to nearest.

Block A (origin x = 5, y = 5). Six columns by eight rows of cells; each cell is 30 units wide and 8 units tall, so the block covers x 5 to 185 and y 5 to 69. For the cell at column c (0 to 5) and row r (0 to 7), with cell left edge x0 = 5 + 30 c and top edge y0 = 5 + 8 r:

- the left vertices take hue 2 (x0 − 5) degrees, and the right vertices take 2 (x0 − 5 + 29) degrees (mod 360);
- the top vertices take saturation 4 (y0 − 5) + 3, and the bottom vertices take 4 (y0 − 5 + 8) − 1;
- every vertex takes value 4 V.

The result is a hue spectrum across the block, with saturation falling toward the top.

Block B (origin x = 190, y = 5). A 16 by 64 unit ramp, drawn as two quads: the first covers y 5 to 37 and the second y 37 to 69, both 16 units wide (x 190 to 206). Hue 2 H and saturation 4 S + 3 for every vertex. Value 255 at y = 5, 128 at y = 37 (first quad), 127 at y = 37 and 0 at y = 69 (second quad). The 127 and 128 step at the join is as stated. The result is a ramp from the colour to black.

Blocks C and D: drawn with the same colour rules as A and B at the same origins, overlaying them. Their layout and content are not described (Open questions).

Vertex output. A screen position (float) is converted to a vertex record by rounding half away from zero, with three colour bytes and the fixed alpha byte 128.

Test vectors for the colour rules are in Test vectors.

### View and projection build

Confidence: eight products High for the count, Medium for their use; the frustum build (tangent 1.0, aspect term, five planes, per-frame call) High; the frustum having no visible effect Medium.

Each frame, the frame setup for the object does:

1. Build the view block from the camera (see SPEC-0027). Its four rows are stored in the object.
2. Multiply the view block by eight source 4 x 4 matrices of the object. Each product takes four output rows (32 rows in all). The products are written to output blocks of the object. The source matrices are at eight places in the object; their placement is not in this spec.
3. Build the culling frustum. The setter takes the view record's distance field (+0x14), the tangent of 45 degrees (1.0) and the aspect term (the view record's height times that tangent, divided by its width: 0.75 for the 640 by 480 record, so 4:3). It scales the camera matrix's first lateral row by distance × 1.0, its second lateral row by distance × 0.75 and its forward row by distance, builds the far corners from them and stores five plane records in the view object (four sides and the far plane). The frustum is 90 degrees wide and 4:3, wider than the camera projection of 64.0 by 50.2 degrees (Camera projection); it is not the GS projection. The builder runs once per frame on the global view object with the camera matrix, and is also reached from two other places. The only reader found of the five planes is a five-plane sphere test that nothing calls, so the frustum has no visible effect (Medium).

The frustum setter is also called from a short wrapper that passes an object, a 64-bit value and three words. The wrapper's callers are not all identified.

### What the render path does not contain (High, as a negative for the routines described)

The routines described here have no loop over lights and no fog colour or fog distance blend. Lighting is done in the vertex program from the per-draw light block (see Light slots and the lit draw). There is no fog.

Level of detail (High for characters; Medium for the world):

- Characters: no model is switched by distance. Distance detail switching is exactly two rules: the 350-unit submit cull (a character farther than that is not drawn at all) and the far set (one-bone skinning beyond 50 units, see Far state at flush; the vertex shape and the textures do not change, only the skin blend). The class B level-of-detail flag (see Draw classes) is written and never read, so it has no effect (High).
- World and models: the type-4 distance cull and distance fade of SPEC-0004 (cull and fade distances up to about 4000) are the world's level of detail; they do not choose between models. Type-20 instances carry their own draw distance (SPEC-0004).
- The designer draw-distance and low-resolution-distance fields are not read by the draw path (see Object submission and culling).
- The value 4000 that is sometimes taken for a level-of-detail distance on render objects is not a distance: it is the byte offset 4000 (0xFA0) of the render object's render-ready flag. The flag is set after the object's first pose and cleared on release; a render object with the flag clear draws nothing.

Linked subtree (Medium-High). A model tree node of type 4 carries three subtree links (+0x70, +0x74 and +0x78), stored on the disc as offsets and turned into pointers when the model is loaded; nothing else writes them. The class A draw walk draws a type 1 node as a leaf (its children are not walked). For a type 4 node it first walks the subtree of the third link (+0x78) when that link is set, then the node's own children in sibling order; the first two links are not walked by this walk. There is no child choice: the third link adds a subtree. The one condition is the node's layer flags: with flags 0 the subtree always draws, and with nonzero flags it draws only when the flags share a bit with the character's draw mask (SPEC-0004, Drawing a node tree, character models); otherwise the subtree is left out and, since these nodes have no children, nothing below the node draws. Test vector: a type 4 node with children A then B and a third link to a tree whose only leaf is C draws C, A, B; with the link empty it draws A, B.

### Curves and bounds (not draw code)

A curve evaluator (weighted sum of control points) and a tangent helper use vector-unit arithmetic, but they belong to path and track logic, not to the draw pass. A bounding-box accumulation over a point list (minimum and maximum vectors, skipping invalid entries) is used for mesh or object bounds (Medium). Neither is part of the draw sequence above.

Per-part padded bounding box (High, corrected 2026-11-09; earlier Medium-High and called the chain point mover). CORRECTED 2026-11-09 (High; the earlier text said a shared transformed point list and a margin of one quarter of the LARGEST of the three size components, both withdrawn): for each bone table of the model (model +4, count at +0x14), the routine takes the WORLD positions of joints 0 to N − 1 (the translation row of each joint's frame matrix) and keeps their axis-aligned box as a low corner and a size (maximum minus minimum). The margin is one quarter of max(size.x, size.z); the y size is ignored (the y component of the maximum is overwritten by the z value, so the margin is not taken from the largest of the three components). The low corner moves down by the margin on x, y and z and the size grows by twice the margin on x, y and z (the w lane has margin 0). The box is stored over the two (0, 0, 0, 1) filler vectors at the start of the bone table header (low corner at +0, size at +0x10; SPEC-0005, Layout, Bone table). It is rebuilt every draw, so it does not grow. In the earlier description the box was kept in a part record with a point list; the headline rule is now as above. It feeds the per-part visibility classification against the frustum planes. It moves nothing and is not a rope or cloth. For a part with no points the box keeps its starting values (a very large low corner and a very large negative size, then padded by the rule above), an inverted box; a part with no points has nothing to draw, so its classification does not matter. (2026-11-09: this "no points" sentence was written for the earlier point-list reading; the joint-based routine was not checked for N = 0, Medium. The default stays: skip such a part.)

Weight and parameter routines (Low). One routine computes a set of weights over N points with divisions, for a divided-difference or spline parameter. Another computes a normalised parameter as (A minus B) divided by (C minus 2 D plus 1) from four helper values. Neither is known to be in the draw path. Which spec uses them is Open question 40.

### Bounding radius of a box (Medium)

For a box given by the four values of the Layout (low X, high X, low Z, high Z), the X extent is high X minus low X and the Z extent is high Z minus low Z. The radius is 0.7072 times the larger of the two extents. The factor is the fixed constant 0.7072. It is close to one over the square root of two (0.70711) but is not that value, so the clean side must use 0.7072. The Y extent does not enter. The known direct caller is the type-16 foliage patch builder: it uses this value as the base of a patch bounding sphere, adds half the near model bound, and uses the composite sphere for foliage frustum culling (SPEC-0004, Radius from a box). This does not establish a use for generic model-part bounds, collision shapes or effect range. Confirmed 2026-11-11 (High): the foliage placement routine applies this same radius rule, and the near and far draw weights of SPEC-0060 Behavior 3 are what the distance-blend routine computes, comparing squared thresholds (S - O)^2 and (S + W)^2 before the square root.

The known direct caller uses the radius for foliage patch frustum culling. Generic model-part bounds, collision, effect range, and any indirect or external-module uses remain unestablished (Open question 38). Test vectors are in Test vectors.

### HUD coin model and camera-facing quads (Medium-High)

The routine once described as a camera-facing billboard is the spinning coin of the HUD's 3D pass. The pass calls it every in-play frame (the main object manager's mode word is not 1 or 2); it draws only while the money line is shown and both side panels are closed (SPEC-0023). It fetches the coin item model (item id 135, the first coin kind; SPEC-0021) from the item model registry and draws it in camera space. The placement (0.65, 0.24, 6.0) is not a position in world or camera units: 0.65 and 0.24 are fractions of the screen width and height (from the left and from the top), and 6.0 is the depth in units straight ahead of the camera. On the 512 by 448 canvas the coin's centre is at (332.8, 107.5). The anchor is the camera position plus the sideways and up camera axes scaled by the screen offsets at that depth, plus the forward axis times 6.0, so the anchor projects back to exactly those screen fractions. The model is scaled uniformly by 0.2625 divided by the length of the midpoint of the model's two bound vectors. It is drawn with the depth test on (greater-or-equal, against the depth left by the world draw, which is not cleared before the HUD 3D pass) and lit from a scene light context built from the scene default lights. The full worked placement, size and caption relation are in SPEC-0035 section 15.2. The coin turns about the vertical axis at 1 radian per second (about 6.28 s per turn): the angle is the rate 1.0 times a time value that falls by one frame step on every drawn frame, and the rotation is built from the sine and cosine of that angle. It is a 3D model drawn in the HUD pass, not a screen-space sprite (the HUD layer is SPEC-0023). [High for the gate, the model, the spin, the screen-fraction and depth meaning of the placement, the depth test and the light context; Medium-High for the sideways direction of the offset]

A separate camera-facing quad builder serves the Darkest Faerie class-38 loop-B sidestep projectile: its loop-B handler opens a type-3 effect entry with the computed deflection direction, and the effect-opening path submits the four-corner primitive, whose in-plane half-axes are scaled by 2.4 (SPEC-0054). Its other users, the specific texture, the anchor convention and whether it goes through the frame draw list remain open. [Medium-High for this caller path]

### Collision routines (not render)

The segment sweep and the overlap test that use vector-unit arithmetic are collision routines (SPEC-0018 and the movement specs). They are not part of this spec.

## Lights

Confidence: the 5 percent slot blend High; scene default lights High; character lights High (see that section).

Swing-trail colour gradients (pointer; High): the 0x50-byte records once described here as light templates (a default and eight presets) are the swing-trail colour gradients of SPEC-0034 section 18 (used by the swing trail of SPEC-0034 3.8), not lights, and never reach a light context or the lit draw.

### Scene default lights (High)

Every scene light context starts from four default lights. They are fixed constants of the boot program, not disc data. Each is a light entry of the SPEC-0028 layout, put into an empty slot by the slot rules of Character lights (by type and flag):

| Entry | Type | Flag | Strength | Slot | Direction (travel) | Colour (R, G, B) |
| --- | --- | --- | --- | --- | --- | --- |
| 0 | 1 (ambient) | 0 | 1.0 | 0 | not used | (50, 50, 50) |
| 1 | 2 | 1 | 0.25 | 1 | (0.666, −0.524, 0.531) | (255, 255, 195) |
| 2 | 2 | 0 | 0.25 | 2 | (0.193, −0.598, −0.778) | (205, 205, 255) |
| 3 | 2 | 0 | 0.25 | 3 | (−0.649, −0.665, 0.368) | (205, 205, 255) |

All four have radius −1 (no radius). The directions are travel directions (they point down); the lit draw uses their negation. So with these lights, slot colours are C_1 = (63.75, 63.75, 48.75) and C_2 = C_3 = (51.25, 51.25, 63.75), and the ambient is (50, 50, 50).

Resource 527 is never read by the game. Its text describes the same rig: its three directions are these travel directions, and the fourth value of each colour line is the strength (the text gives 0.25, 0.25 and 0.20; the game uses 0.25 for all three). The ambient (50, 50, 50) exists only in the code.

### Character lights (High)

The area's light set (SPEC-0028, section 2) is read by characters only. Timing: the walk below runs once per main-loop pass, after every effect pool has updated and published its lights, for the focus hero, the player's live petpet (the companion of SPEC-0047), and every character of that pass's active-character list (all characters of the loaded region records). It runs only while a world is loaded and the game mode is below 9 (normal play). The dynamic point-light list is emptied at the start of each pass, so every walk sees all the dynamic lights of the same pass. For each of these characters the walk runs only when the character stands on a ground entry and passes the zone query with column = the zone byte of that ground entry and key = the character's world id. This is the same test as the cell flag of SPEC-0006 (Character activation, Cell flag); the query passes when no grid is loaded. When the test fails, the walk is skipped and the character keeps the light state of the last pass that ran it. The walk fills the character's light state, which has four slots. This light state is the light context that the character's model draw builds its light block from (see Light slots and the lit draw, including the character mesh path).

Order of the walk:

1. Reset: clear all four slots.
2. Own entries: for each slot still empty, take the character's own light entry for that slot. A character's own entries are the four scene default lights (the scene's light object receives them at area activation), and scene light contexts take the same four. Since they are taken first, all four slots are always full when the area's light set is walked.
3. Pass 1: go through the area's light-set records in table order. Skip each record whose gate word is nonzero. For each other record, take its type 2 entries in order, then its type 1 entry (the first array C entry only).
4. Pass 2: go through the records again in table order, skipping gated records. For each other record, take its type 3 entries in order.
5. Dynamic point lights: take the pass's dynamic point lights (below) in list order, with the same gates and slot rules plus the dynamic rule.

Gates, applied to each entry before its slot is chosen:

- Zone gate: the entry's zone key (its low byte, 0 to 44) must pass the zone query with the light state's zone value. That value is set to -1 at initialisation, and -1 is also used when the state's first word is zero. The query passes when no grid is loaded.
- Radius gate: for a type 1 or type 3 entry with a radius above zero, the entry is rejected unless the squared distance from the entry position to the character's position is less than the entry's radius squared. Entries with radius -1 or 0 have no radius gate.

Weight, the value a slot carries for its entry:

- Types 1 and 2: the weight is the entry's strength, with no falloff.
- Type 3, with strength s, radius r, and d the distance (the length, not its square) from the entry position to the character's position:
  - d up to 0.5 r: weight is s (full strength);
  - d above 0.5 r and up to 0.9 r: weight falls linearly from s to 0, so weight = s × (1 − (d − 0.5 r) / (0.4 r));
  - d above 0.9 r: weight is −1, a sentinel. Such an entry is never stored (see the slot choice).

Slot choice, after the gates (w is the new entry's weight):

- Type 1: slot 0, replacing any earlier occupant.
- Type 2 with flag 1: slot 1, replacing any earlier occupant. A type 3 entry with flag 1 would also go to slot 1; none exists on disc.
- Type 2 with flag 0, and type 3 with flag 0: a free slot of slots 2 and 3, slot 2 first. Because the own entries fill every slot, no slot is ever free here, so a displacement candidate c is chosen instead:
  - both slots 2 and 3 hold type 2 lights: the one with the smaller weight (slot 2 on equal weights); but if exactly one of the two is locked, the unlocked one;
  - only slot 2 holds a type 2 light: slot 2 if it is unlocked, else no candidate;
  - only slot 3 holds a type 2 light: slot 3 if it is unlocked, else no candidate;
  - neither holds a type 2 light: no candidate.
- A type 2 entry with no candidate is rejected. Otherwise it replaces c when w ≥ weight(c).
- A type 3 entry with a candidate c:
  - if d > 0.9 r (the entry lies between 0.9 r and r, since the radius gate passed), the entry is not stored. Instead c's weight becomes (d − 0.9 r) / (0.1 r) × the strength of c's entry, and c is locked for the rest of this walk. This is a cross-fade: as the character moves from r in to 0.9 r, the directional light it displaces fades from full strength to 0, and inside 0.9 r the point light takes the slot with a weight that rises from 0;
  - otherwise it replaces c when w ≥ 0, and is rejected when w < 0.
- A type 3 entry with no candidate (slots 2 and 3 already hold point lights, or are locked) is rejected, unless it is a dynamic light with w > 0 (the dynamic rule). Then it replaces slot 2 when weight(2) < weight(3), and slot 3 otherwise, whatever its own weight.
- A slot taken by a dynamic light is marked for this walk, so its colour is replaced at once at the next fill, with no 5 percent ease (see Light slots and the lit draw).
- Any other type: not stored.

So, in table order, the first two point lights within 0.9 of their radius take slots 2 and 3 from the default directional lights, and later ones are dropped; dynamic point lights then replace the weaker of the two.

Dynamic point lights: they are built only by the particles of light-emitter emitters (SPEC-0034 4.5). On disc 37 emitter records are light emitters, used by 167 effect definitions (one such emitter each). Each spawner of such an emitter owns one light entry. Each live particle writes that entry each pass and appends a reference to it to the list (at most 200 per pass):
- type 3; position = the particle's anchor position;
- colour = the particle's curve channel 1 colour × 255, with alpha 0; strength = channel 1's alpha;
- radius = the particle's size (the x value of curve channel 2), and its square as the radius squared.
- The flag and zone-key words are not written, so they keep the spawner record's values, taken to be 0 [Medium-Low].
- A spawner with k live particles is therefore listed k times, every time with the last particle's values (original quirk). By the dynamic rule a repeat can put the same light into both slots 2 and 3 (when its weight is not below the other slot's). A clean side may list each spawner once; mark the difference.
79 placed effect spawns (type 39, SPEC-0028) in 13 packs use 11 of these effect ids (1300, 2202 to 2207, 4012, 4023, 4030 and 4031); the other light emitters are started by animations and scripts. Dynamic point lights do not come from world records.

Light kinds: type 1 entries (array C of a type 30 record) are the ambient light. They fill slot 0, and their direction is not used; a type 1 entry with a positive radius is also cut off beyond it. Type 2 entries are directional lights along the entry direction, which is the light's travel direction. Type 3 is a point light at the entry position, with the falloff above; at draw time its direction is normalize(object position − light position).

The swing-trail colour gradients of SPEC-0034 section 18 are not lights and never enter this walk.

Type 30 light sets: the layout is in SPEC-0028, section 2. Type 30 light sets reach characters only: the character walk above is their only reader. World geometry is unlit, and other lit props use the scene default lights. Test vector: resource 685 (a world pack) has 29 records. Record 0 has counts A = 3, B = 0 and C = 1, so four entries and a size of 352 bytes. Its gate word is 1, so the character walk skips it and it never reaches a character. Record 1 has the same counts and gate word 0, so use record 1 for character-light vectors. Record 0 and record 1 differ in 21 bytes: the gate word, and in each of the four entries the radius, the radius squared and the zone key. Positions, directions, strengths and colours are the same. Record 0's first entry is type 2 with flag 1, strength 0.15, position (−294.9, −15.08, −691.0, 1), unit direction (0.1005, −0.9917, 0.0801), colour (203, 255, 143, 255) and radius −1.

Negative facts:

- No fog colour and no fog distance are set or used.
- A sky dome exists (see Sky domes): one of resources 1, 2 and 3 is loaded for every world. No other backdrop is known.
- No time-of-day lighting exists.
- The only hint of fog is the name of a source file that appears in one text string. The routine that uses that string is not identified.

### Character light slots (High for the storage and the slot choice; the weights follow Character lights)

Before a character is drawn, the walk of Character lights fills the character's four light slots (0 to 3) from the area light set. The slot chooser runs once per light entry and either stores the entry in one slot or refuses it ("no slot", -1).

Inputs: the character's light state; the light entry (the 0x50-byte entry of SPEC-0028 section 2: position at +0x00, colour at +0x10, type at +0x30 with values 0 to 3, strength at +0x34, flag at +0x38, radius at +0x3C, radius squared at +0x40, zone key at +0x44); the character, which supplies the model transform for the stored value; the character position; and the dynamic flag (nonzero for a dynamic point light).

Per-slot storage in the light state, for slot s = 0 to 3:
- the occupant: an entry reference at +0x1C + 4s; zero means empty. Only slots 2 and 3 are tested for emptiness.
- the weight (float) at +0x3C + 4s, written when the slot is accepted and also by the cross-fade.
- the lock flag at +0xD0 + 4s, so +0xD8 and +0xDC are the locks of slots 2 and 3.
- the 16-byte value at +0x90 + 16s: the entry's colour or direction transformed for the character, written when the slot is accepted.

Steps, in order:
1. A missing entry, character or position: no slot.
2. Zone gate: the entry's zone key must pass the zone query with the character's zone value (Character lights); otherwise no slot.
3. Radius gate: for types 1 and 3 only, and only when the radius is above 0. Let D2 be the squared distance from the entry position to the character. D2 at or beyond the radius squared gives no slot. Types 0 and 2 skip this gate.
4. Weight w: the strength for types 1 and 2; the falloff of Character lights for type 3, with d the distance and r the radius.
5. Choice by type:
   - Type 0: no slot.
   - Type 1: slot 0.
   - Type 2 or 3 with the flag nonzero: slot 1, with no occupancy or weight test.
   - Type 2 with the flag 0: the first free slot of slots 2 and 3 (slot 2 first) is accepted with no weight test. Otherwise take the displacement candidate c (step 6). No candidate: no slot. w below weight(c): no slot. Otherwise accept c.
   - Type 3 with the flag 0: the first free slot of slots 2 and 3 is accepted at once. Otherwise take the displacement candidate c. With no candidate: when the dynamic flag is nonzero and w is above 0, take slot 2 when weight(2) < weight(3) and slot 3 otherwise; else no slot. With a candidate c: if c is locked, no slot. Let x = d - 0.9 r. When x is above 0, c's weight becomes x / (0.1 r) times the strength of c's entry, c is locked, and the result is no slot. Otherwise w below 0 gives no slot, and any other w accepts c.
6. Displacement candidate (only when slots 2 and 3 are both occupied). Let F be slot 3 when weight(2) > weight(3), otherwise slot 2.
   - Both slots hold type 2 lights: both unlocked gives F; only slot 3 locked gives 2; only slot 2 locked gives 3; both locked gives F.
   - Otherwise: slot 2 holding a type 2 light and unlocked gives 2; else slot 3 holding a type 2 light and unlocked gives 3; else no candidate.
7. Accept slot s: store the entry reference in slot s, store w as its weight, and write its 16-byte value.

## Edge cases and error handling

- **Full draw list** (original): a 19th or later insert prints a debug message and is dropped; the frame goes on. The clean side should keep the object (see Draw list capacity).
- **Draw count mismatch** (original): a debug message after the flush, nothing else. The clean side should not need it.
- **Far flag** selects the near or far vertex-program state set at flush. It does not change which model or lights are used; in the skinned programs the far set skins each vertex with its first bone only and turns off the specular alpha term of program 6 (High).
- **Active petpet drawn twice** (original quirk): the active petpet is submitted from its slot and again as the special object, so it is drawn twice per frame, once possibly with the far set and once with the near set. A clean implementation may draw it once; reproducing the double draw is not required.
- **Fade on at level 128**: opaque in the sort, but still drawn inside the fade wrap.
- **Fade rectangle off screen**: nothing is composited for that character.
- **Shared vertex-program state set**: the set is rebuilt only when the selected index changes, so a draw uses the set last built for that index.
- **Exact boundary**: a squared distance of exactly 122500 is dropped; exactly 2500 does not set the far flag.
- **Empty sub-list entries** (load-result word zero) are not submitted.
- **Object list slots** that are not marked active (flag not equal to 1) are skipped without checks.
- **Part classifier, far side**: model parts and mesh leaves leave the far side out of their side mask. A part whose corners are all beyond the far plane is still culled (all-corners rule over all six bits). A part with some corners beyond the far plane only and the rest inside is inside (0), not clipped.
- **Empty volume list**: on disc the volume test rejects nothing.
- **Light slot with a nonzero second value**: it is replaced directly, not blended.
- **Lit draw without a light context**: no light block is sent, so the draw uses the last block sent (order-dependent). The draws concerned (effect models, world record lists and other draws that pass none) are listed under Light slots and the lit draw; world items, the HUD coin, the 3D book and the item showcase do pass a scene light context. Only lit materials are affected, and the last block stays in the vector unit across frames. Default if unresolved (rewrite choice): keep one current light set that draws with a context replace and lit draws without one reuse (Light slots and the lit draw).
- **Unlit colour above 255** (Medium-High): the unlit programs 0 and 4 multiply the colour byte by the colour scale with no upper limit (a scale above 1.0 is possible with a non-default draw colour). The GS keeps only the low 8 bits of each lane, so the value wraps modulo 256. Disc level-geometry colour bytes never exceed 245, so no wrap happens while every scale lane is at most 1.041 (SPEC-0004, Edge cases). The clean side reproduces the wrap for strict parity, or clamps to 255. The lit programs limit their output to 255.
- **Fade bit**: the vertex program stores the fade bit and its level but never reads them, and the 257-entry fade-state table is never selected; it has no effect.
- **No far clip**: the vertex program never clips against the far side. A vertex beyond F = 10,560 units gets a depth below 1.0, tending to c = (A F − B N) / (F − N) ≈ −98.3, and a negative depth wraps in the 24-bit Z buffer (c depends on the area's depth scalar B; ≈ −98.3 is for B = 2^20, and it is −27.4 for B = 300,000). No drawn part is known to reach that far: the farthest sky-dome vertex of Meridell is about 5,830 units from the village eye, and the part cull uses the 5,280 far plane. [Low]
- **Stale near depth (quirk)**: the clip record's copy of the near-plane depth value B is taken once at camera creation and is not refreshed when the atmosphere scalar lowers B, so near-clipped vertices keep the depth 2^20.
- **Brightness factor ignores green (quirk)**: f uses max(red, blue) of each light colour; a pure green light counts as zero.
- **Specular light vector (quirk)**: program 6 uses the negated x components of the three light directions (the first vector of the transposed block) as its light vector, not a light direction.
- **Negative opacity request**: selects the fixed colour scale (1.2, 0.5, 0.5, 1.0), a reddish tint, not an index of the opacity table.
- **Distance fade window**: a type-4 node fades only while the eye's distance q to its sphere is inside (E − F, E). At q = E or beyond, f is 1 again; the node's cull distance decides whether it is drawn.
- **Sky dome layer D with b at or below 0**: not drawn.
- **Sky dome translations (original quirk)**: at world activation the original writes the saved translations of layers B and C into each other's instance. This is harmless, because B and C rebuild their matrices every frame and every translation is (0, 0, 0). The clean side need not reproduce it.
- **Translucency**: a fade record with fade on and a level of 128 or more is opaque.
- **Book open angle**: a value below 0.12 after the step is set to 0.12, so the angle stops at 0.12.
- **Shadow casters**: at most three; petpets never cast on disc (gate word 0); a world character casts only with kind key 720.
- **Shadow direction ease (original quirk)**: the shadow light direction routine blends toward a previous direction (rate 0.05 after a slot change, data word 0x2EC9D8; otherwise 0.5 × k / 20 with k counting frames up to 20, data words 0x2EC9DC and 0x2EC9E0), but the previous direction is not kept between frames, so the blend has no effect. Only on a context's first use is the blended value normalize(0.95 D_s + 0.05 S), with D_s the direction of the stronger of slots 2 and 3; the y limit then applies as usual. The clean side need not reproduce it and uses L of step 2a.
- **Shadow extrusion length**: the 25.0 stored beside the instance's shadow block pointer is never read; the length is 16.0 (data word 0x2ECCC8).
- **Shadow count wrap**: the count buffer wraps at 256 (32 steps of 8), so a pixel where camera-facing polygons are outnumbered by 16 (k = −16) also reads as shadowed, and one with k = 17 reads as lit. Reproduce the wrap only if the clean side uses the same 8-step counting.
- **Weight −1 entries**: never stored. Own entries leave no free slot, the cross-fade path stores nothing, and the dynamic rule needs a weight above 0.
- **Point light at exactly 0.9 r**: the computed weight is a tiny negative number (0.4 is stored as 0.39999998), so the entry is rejected, and no cross-fade happens either (d − 0.9 r is not above 0).
- **Radius-0 point light (original quirk)**: one type 3 entry on disc has radius 0 (SPEC-0028 section 2: pack 1174, record 44, zone key 3, strength 0.65). It has no radius gate, and at any d > 0 the cross-fade divides by 0.1 × 0: the original gives the displaced directional light the largest float times its strength, so that light saturates every lit vertex facing it. The clean side treats a type 3 entry with r ≤ 0 as absent; mark the difference.
- **Dynamic light listed k times (original quirk)**: a spawner with k live particles is listed k times with the last particle's values (Character lights).
- **Character light slots, types 0 and 2**: the radius gate is not run for them (Character light slots, step 3). Type 0 never takes a slot.
- **Character light slots, flag-set entries**: a type 2 or type 3 entry with the flag nonzero takes slot 1 without checking whether slot 1 is occupied, so it overwrites the occupant.
- **Character light slots, shared weight word**: the cross-fade writes the displaced light's weight into the same word that an accept writes, so the cross-faded weight lasts until that slot is next accepted.
- **Character light slots, first free slot**: a slot taken as the first free slot skips the weight test (type 2) and the lock test (type 3). In the character walk the own entries fill every slot, so this path is not reached there.
- **Character light slots, dynamic fallback**: the dynamic rule can take slot 2 or 3 when both are occupied and no displacement candidate was found.
- **Character light slots, gate boundary**: an entry whose squared distance equals the radius squared is refused (at or beyond).
- **Grey colour record** (value 0 in the colour record): the hue is the marker 361. Its derived hue index is round(361 / 2) = 181, which is above the pad range of 0 to 180. Its saturation index, by the stated rule, is round(−1 / 4) − 1 = −1. Whether the original clamps these derived values is not established, and it has no effect on play: the colour sheet is drawn only when the UI node tree visits a kind 8 node, which no node on this disc is (see Reachability above). Default if unresolved (rewrite choice): clamp them to the pad ranges and mark the difference.
- **Full-brightness colour record**: a value of 255 gives a derived value index of round(255 / 4) = 64, above the range 0 to 63 of the pad steps. The same open question applies.
- **Zero-width colour sheet strip**: a block with no width emits no quads (for the cell loops).
- **Colour sheet visit**: the sheet is drawn only when the UI node tree visits a kind 8 node, which never happens on this disc (see Colour sheet, Reachability).
- **Culling frustum with a zero view width**: the builder divides by width and has no explicit zero-width guard. It is only a culling frustum (Open question 84), not a camera rule.
- **Packet larger than 1024 bytes**: refused and logged; nothing is submitted (Medium-High).
- **Per-part padded box with no points**: the box keeps its starting values (very large low corner, very large negative size) before the padding; the part has nothing to draw (Open question 41, Resolved). Default if unresolved (rewrite choice): skip such a part.
- **Held-item rescale (original quirk)**: the stowed shield and the sheathed weapon are scaled by 1.6 / |min − size| and min(2.6 / |min − size|, 0.6) of the model's root box, not by the box size alone. Reproduce the formulas as stated.
- **Tattered Cloak**: while the focus hero wears item 126, none of the hero's held items is drawn, whatever the flags.
- **Roberta's shield**: never drawn, in either flag state. Her wand is not drawn while the weapon-drawn flag is clear (she has no sheath point).
- **Shared flags**: there is one weapon-drawn flag and one shield-raised flag for both heroes, not one pair per hero; the hero swap clears both (SPEC-0040).
- **One world-item slot (original quirk)**: only one potion model is resident. After a world potion of a second kind is spawned, world potions of the first kind have no model, so nothing is drawn for them until that model is loaded again.
- **Held entry and a drawn weapon**: while the weapon-drawn flag is set, the held entry is not drawn at all (the weapon takes the hand).

- **Display scale.** A scale factor below 0.1 or above 1.0 is clamped before the size is computed (High).
- **Fade level.** A level below the variant's low bound or above its high bound is clamped to that bound first (0.008 and 1.0 for variant A, 0 and 1.0 for variant B), then the index is clamped to 0 or 256 (High). Variant A with a negative level selects no index and sends the fixed scale (1.2, 0.5, 0.5, 1.0) instead (High).

## Test vectors

Cull (SPEC-0046 threshold, 350 units):
- Object at 349.9 units: squared distance 122430.01, which is less than 122500. Kept.
- Object at 350.0 units: squared distance 122500.00. Not less than 122500. Dropped.

Far flag (50 units):
- Object at 49.9 units: squared distance 2490.01. Far flag clear.
- Object at 50.0 units: squared distance 2500.00. Far flag clear (strictly greater is needed).
- Object at 50.1 units: squared distance 2510.01. Far flag set, so the far vertex-program state set is used at flush.

Drawable test (base state, despawn flag):
- A townsfolk record bound by streaming: its base state is 42 after the object reset, then 1 at once. It is drawable on its first frame when its slot is active, its cell flag is 1 and its despawn flag is 0.
- Gelert Assassin fight, mirror image in block slot 2 after set-up: slot active, base state 42, so it is not drawn. The boss's animation 42 passes one half: the image's base state becomes 1 and it is drawn from the next frame, placed at +10 lateral. A hit on the image: base state 42 and a puff effect, so it is not drawn.
- A dead spider (state 16) whose animation 58 has finished: despawn flag 1, so it is neither drawn nor updated, though its slot is still active.

Translucency class (fade on):
- Level 127: translucent. Level 128: opaque. Fade off with level 0: opaque.
- Level 64 with fade on: sorts translucent and composites at opacity 0.5. Level 128 with fade on: opaque but still wrapped. Fade off: opaque, no wrap.
- Ghost levels (R-0202): opacity 0, 0.25, 0.5, 0.7 and 1.0 give levels 0, 32, 64, 89 and 128, that is 0, 25, 50, 69.5 and 100 percent. Pixel example: frame (100, 100, 100), ghost surface (200, 40, 40), level 64: (150, 70, 70). Where two ghost parts overlap on screen, only the nearer part's colour enters the blend. At level 0 nothing is visible but the ghost's silhouette still hides depth-tested draws behind it.

Distance (view point (0, 0, 0)):
- Object (30, 40, 0): squared distance 2500, far flag clear. Object (30, 40, 1): 2501, far flag set (y counts).
- Object (210, 280, 0): squared distance 122500, dropped.

Level-of-detail flag (T = 150, T² = 22500, stored flag 1):
- Object at horizontal distance 149 (h = 22201): |h − T²| = 299 > 25, so the flag becomes 0.
- Then at 150.05 (h = 22515.0): candidate 1, but |h − T²| = 15.0 is not above 25, so the flag stays 0.
- Then at 150.1 (h = 22530.01): 30.01 > 25, so the flag becomes 1.
- A height difference of 100 changes nothing.

Shadow casters: held squared distances 100, 400 and 900. A new candidate at 225 replaces 900. A new candidate at 1000 is dropped. A candidate at 2500 is not a candidate.

Shadow volumes:
- Light direction, scene default lights only (ambient and three type 2 lights at 0.25; slot 1 wins the tie): S = 0.25 × (0.666, −0.524, 0.531), L = (0.666, −0.524, 0.531); its y is below −0.4, so there is no limit.
- Light direction, a type 3 light in slot 2 with weight 0.5 at (0, 10, 0), caster at (3, 0, 4), slots 1 and 3 holding default lights 1 and 3 (0.25 each; slot 1 is the strongest type 2): D = (0.2683, −0.8944, 0.3578); S = (0.3007, −0.5782, 0.3116); L = (0.4162, −0.8004, 0.4314).
- Y limit: S = (1, −0.2, 0) normalises to y = −0.196, above −0.4, so L = normalize(0.9806, −0.4, 0) = (0.9259, −0.3777, 0). S = (0, 0, 0) gives (0, −1, 0).
- Unclipped extrusion with L = (0, −1, 0): far copy = near copy + (0, −16, 0).
- Counts (buffer value per channel): k = 0 gives 120 (lit); k = 1 gives 128 (shadow); k = 16 gives 248 (shadow); k = 17 gives 0 (lit, wrap); k = −1 gives 112 (lit); k = −16 gives 248 (shadow, wrap). Resolve: a shadowed frame pixel (100, 60, 5) becomes (92, 52, 0).
- Resolve: a shadowed pixel (kept channel 128, alpha bit set) with frame colour (100, 50, 5) becomes (92, 42, 0); an unshadowed pixel (kept channels 0) keeps its colour; the drawn alpha is 32 for every texel, so no texel fails the alpha test.

Character lights (point light r = 10, s = 0.5):
- Weight: d = 0 gives 0.5; d = 6 gives 0.375; d = 8 gives 0.125; d = 9.5 gives −1.
- Cross-fade with a default directional occupant (strength 0.25): d = 9.5 sets the occupant's weight to 0.125 and the point light is not stored; d = 9.9 gives 0.225; d = 10.0 fails the radius gate (100 is not below 100), so nothing changes.
- Slot ease (persistent context, kind 0): stored 0, new value 100 gives 5, 9.75 and 14.26 after 1, 2 and 3 fills; half way after about 13.5 fills.
- Walk with the four default lights as own entries and one record holding two point lights within 0.5 r (strengths 0.5 and 0.3): the first replaces slot 2 (equal default weights pick slot 2; 0.5 ≥ 0); the second finds slot 3 the only type 2 slot and replaces it. A third point light in a later record finds no candidate and is dropped.
- Character light slots, radius gate (synthetic): a type 1 entry with radius 10.0 (radius squared 100.0) and the character at 10.0 units is refused; at 9.9 units it takes slot 0.
- Character light slots, type 0 (synthetic): a type 0 entry is refused at any distance.
- Character light slots, first free slot (synthetic): a type 3 entry with the flag 0 and slot 2 empty takes slot 2. With slot 2 occupied and slot 3 empty, it takes slot 3.
- Character light slots, displacement (synthetic): slots 2 and 3 hold type 2 lights with weights 3.0 and 2.0, both unlocked: the candidate is slot 3.
- Character light slots, cross-fade (synthetic): the same setup, a type 3 entry with radius 10.0, the character 9.5 units from it, and slot 3's light of strength 0.6. Then x = 9.5 - 9.0 = 0.5, slot 3's weight becomes 0.5 / 1.0 x 0.6 = 0.3, slot 3 is locked, and the entry is refused. With the character 8.5 units away, x is negative and the entry is accepted into slot 3 (its weight is at least 0).

Camera projection (A = 1, B = 2^20, N = 1, F = 10560):
- Depth: z = 1 gives 1048576; z = 2 gives 524238.8; z = 10 gives 104769.1; z = 100 gives 10388.4; z = 1000 gives 950.4; z = 5280 gives 100.3; z = F gives 1.0.
- Screen: X = 6.25, Z = 10 gives x = 2048 + 256 = 2304, the right edge of the 512-wide buffer.

3D book: the open angle 1.2 reaches 0.12 after 75 draws (1.2 − 75 × 0.0144 = 0.12) and stays there.

Story layer: a type-4 node with layer flags 0 is always drawn. A node with only bit 22 (0x400000) is drawn only while flag 10 is set and flag 322 is clear (layer 2), or while the world mask word has bit 22.

Draw order: with the hero fading and one enemy at 20 units, the frame queues background, dome, world, items, flush, shadows, effects, then [enemy, hero (translucent)], book, tint, fade and bars, HUD.

Sort order, example 1 (insertion order: A opaque at squared distance 10; C translucent at 30; B opaque at 20):
- Insert A: list is [A]. Insert C: translucent goes after the opaque A: [A, C]. Insert B: opaque, and both B and A are opaque with B farther (20 over 10), so B goes before A: [B, A, C].
- Result: B, A, C.

Sort order, example 2 (insertion order: T40 translucent at 40; T60 translucent at 60; O5 opaque at 5):
- Insert T40: [T40]. Insert T60: same class, larger, goes first: [T60, T40]. Insert O5: opaque goes before translucent: [O5, T60, T40].
- Result: O5, T60, T40.

Frame order, synthetic: with two object lists of one object each, a scene sub-list with one nonzero entry, one special object and the main character, the draws happen in step 7 in the sorted order of all five records, not in the submission order.

Model-part classifier (side mask 0x2F, every side except far):
- All eight corners beyond the far plane only: culled (−1).
- Some corners beyond the far plane only, the others inside: inside (0).
- One corner in front of the near plane: clipped (1).
- One corner outside +x of the frustum but inside the guard band, no other corner outside any side: inside (0).
- The same corner outside +x of the guard band: clipped (1).
- No corner beyond any side: inside (0).

Light slot blend (kind-0 slot, weight 0.05): stored (0, 2, −1, 0.5), new (1, 2, 1, 0.5). Result after one update (0.05, 2, −0.9, 0.5); after a second update with the same new value (0.0975, 2, −0.805, 0.5).

Lit draw (scene default lights, identity model rotation, colour scale (1, 1, 1, 1)):
- n = (0, 1, 0), vertex colour (16, 16, 16, 16): d = (0.524, 0.598, 0.665); C_1 = (63.75, 63.75, 48.75), C_2 = C_3 = (51.25, 51.25, 63.75); L = (148.13, 148.13, 156.06); output RGBA (148, 148, 156, 128).
- n = (0, −1, 0): all d = 0; output RGB (50, 50, 50), the ambient alone.

Brightness factor f (A, C_1, C_2, C_3 as listed):

| Colours | f |
| --- | --- |
| all four (128, 128, 128) | 1.0 |
| all four (32, 200, 32) | 0.3875 |
| all four (51.2, 0, 0) | 0.65 |
| all four (0, 255, 0) | 0.3 (green ignored) |
| (64, 0, 0), (0, 0, 0), (0, 0, 0), (64, 64, 64) | b = 0.5, f = 0.825 |

Specular alpha of program 6, p = 3, z = −2134851328 (vector-unit rounding may change the last bit):

| c | X | A at f = 1 | A at f = 0.3875 |
| --- | --- | --- | --- |
| 1.0 | 0.75294 | 192 | 74 |
| 0.9 | 0.47647 | 121 | 47 |
| 0.5 | 0.09412 | 24 | 9 |
| 0.1 | 0.00064 | 0 (alpha replaced by 0) | 0 |
| 0 | — | normal alpha kept | normal alpha kept |

- p ≈ 5.1176 (stored to full float precision), z = −4392340992 and c = 1.0: X = 0.66666, A = 169 at f = 1.
- Geometry: n′ = (0, 1, 0), V̂ = (0, 1, 0), light-block vector 1 = (0, −1, 0, f), so L′ = (0, 1, 0). Then n′ · V̂ = 1, n′ · L′ = 1, V̂ · L′ = 1 and c = 1.

Position test vector for the specular term (arithmetic High; the camera offset and the light are assumed values, so this checks the arithmetic, not a screen). Pack 1098, entry 0: hero at Q1 = (0, -127.348, -98.918), camera offset C − Q = (0, 5.44, -9.55), vertex offset from the hero's origin (0, 3, 0.3), n′ = (0.29994, 0.79984, 0.51990), L′ = (1, 0, 0), p = 3, z = -2134851328, f = 1. Af = trunc(255 · f · min(1, X)) is 30 at Q1, 132 at Q1 + (100, 0, 0), 15 at Q1 + (0, 0, 100), and none (the plain lit texture) at the world origin. At Q1 + (100, 0, 0) a pixel with texel (29, 30, 35, 128) and lit colour 128 becomes (161, 162, 167).

Real skinned vertex, near state, program 2 (Tor's first outfit file, the model with 58 bones; arithmetic High, all inputs from the disc). The pose is Tor's main-skeleton idle clip (the clip his action 0 selects; 30 frames, plain) at frame 12, with an identity world matrix, the four scene default lights (Scene default lights), a draw colour scale of (1, 1, 1, 1) and a vertex whose own values are:
- Position p = (−0.37071, 1.29426, 0.06934), normal n = (−0.80813, −0.37281, −0.45600).
- Colour bytes (16, 16, 16, 16). Weights (0.37036, 0.35455, 0.27509), sum 1. Bone-slot bytes (3, 75, 6), so the bones are 1, 25 and 2.

Pose inputs (clip samples are 16-bit values times 0.000977, not normalised; a rotation is q = (x, y, z, w), a local translation is relative to the parent). Bone 0 is the root: q = (−0.00098, −0.06155, −0.01075, 0.99849), translation (−0.13092, 1.17142, 0). Bone 1 (parent 0): q = (0, 0.00195, −0.00098, 1.00045), translation (0, 0.07914, 0.07914). Bone 2 (parent 0): q = (−0.00293, 0.00098, −0.04201, 0.99947), translation (0, −0.07034, 0.09282). Bone 25 (parent 2): q = (−0.03810, 0.07034, 0.13385, 0.98872), translation (−0.22178, 0, 0). Bind positions: bone 1 (0, 1.21875, 0.07903), bone 2 (0, 1.13982, 0.07205), bone 25 (−0.22173, 1.13982, 0.07205). Bone 1's quaternion has length 1.00045, which the conversion leaves as it is (Posing a skeleton, SPEC-0005).

Palette vectors that result (vector j is column j of the 3 × 3 rotation in x, y, z and translation component j in w; M = B · P · W of the skin palette section):

| Bone | Vector 1 | Vector 2 | Vector 3 |
| --- | --- | --- | --- |
| 1 | (0.99262, −0.02331, 0.11906, −0.15820) | (0.02352, 0.99972, −0.00039, 0.03214) | (−0.11902, 0.00319, 0.99289, 0.00050) |
| 25 | (0.98732, 0.15610, −0.03189, 0.03526) | (−0.15827, 0.98380, −0.08508, 0.03159) | (0.01813, 0.08897, 0.99590, 0.08351) |
| 2 | (0.98708, −0.10524, 0.12086, −0.25546) | (0.10555, 0.99441, 0.00380, −0.03270) | (−0.12059, 0.00900, 0.99267, 0.01630) |

Each bone carries the point as M_b p = p.x × vector 1 + p.y × vector 2 + p.z × vector 3 (xyz parts) plus (w of vector 1, w of vector 2, w of vector 3):
- Bone 1: (−0.50398, 1.33490, 0.02470).
- Bone 25: (−0.53433, 1.25318, 0.05426).
- Bone 2: (−0.49313, 1.29396, 0.04525).
- Blended position P = 0.37036 × bone 1 + 0.35455 × bone 25 + 0.27509 × bone 2 = (−0.51176, 1.29467, 0.04083). The far state would use the first slot alone: P = (−0.50398, 1.33490, 0.02470).
- Skinned normal n′ = the rotation part of the first slot (bone 1) applied to n, with no blend and no renormalisation: (−0.75667, −0.35532, −0.54883), length 1.000006.

Light: T_1 = −normalize(0.666, −0.524, 0.531) = (−0.66597, 0.52402, −0.53098), so d_1 = clamp(n′ · T_1, 0, 1) = 0.60916. T_2 and T_3 give negative dot products, so d_2 = d_3 = 0. L = A + d_1 C_1 = (50 + 0.60916 × 63.75, the same, 50 + 0.60916 × 48.75) = (88.833, 88.833, 79.696). Output bytes: RGB = trunc(L × 16 / 16) = (88, 88, 79) and alpha = trunc(128 × 16 / 16) = 128. The position then goes through the camera matrix and the divide by w as in Camera projection.

Per-triangle clipping (W = 512, H = 448, all w = 10, clip bit set):
- x = (2400, 2500, 2600): all outside the right edge, dropped.
- x = (2000, 2400, 2500): not all outside one edge and inside the guard band, kept whole (the GS scissor trims it).
- One vertex at x = 4200 (outside the guard band): clipped to x ≤ 2304 and drawn as a fan.
- One vertex at w = 0.5: clipped at w = 1; that input vertex enters the interpolation with the homogeneous depth B = 2^20.
- Same triangles with the clip bit clear: all drawn as stored.

Draw colour scale:
- Default colour (0.5, 0.5, 0.5, 1.0), opacity 0.5: i = 128, a' = 0.504, s = (1, 1, 1, 0.504).
- Colour (0.25, 0.5, 0.75), opacity 1.0: s = (0.5, 1.0, 1.5, 1.0).
- Distance fade, synthetic: F = 10, E = 4000. q = 3995 gives f = 0.5; q = 3985 and q = 4005 give f = 1.

Meridell mountains (pack 1023), eye (30, 32, 700) looking toward −z, flag 10 clear (B = 300,000):
- A leaf spanning view depth 790 to 1424 stores depth values 352 to 183; one spanning 2184 to 3526 stores 110 to 58.
- One depth value covers 3.3 units at depth 1,000, 13.3 at 2,000 and 53.3 at 4,000 (with B = 100,000: 10, 40 and 160).
- A leaf with all eight corners at x offsets of at least 1688 at depths 1878 to 2523 is beyond the right side (0.625 × 2523 = 1577) and is culled; 23 of the 24 mountain leaves pass.
- The terrain node is column 0, whose cell byte is 1 in all 61 rows, so the mountains draw for every camera cell.

Sky dome:
- Angles after n frames: angle_B = −0.0009 n and angle_C = −0.0006 n. Layer B makes a full turn after 6,981 frames, layer C after 10,472.
- After 1,000 frames layer B maps (1, 0, 0) to (cos 0.9, 0, sin 0.9) = (0.6216, 0, 0.7833).
- Layer D with the default colour and no node fade: b = 0 or below, not drawn; b = 0.5, alpha scale 0.504; b = 1.0, alpha scale 1.0.

Book open angle floor (synthetic): an angle of 0.13 falls by 0.0144 to 0.1156, which is below 0.12, so it is set to 0.12. An angle of 0.30 falls to 0.2856 and is kept.

Bounding radius (synthetic): low X 0, high X 2, low Z 0, high Z 4 gives an X extent of 2, a Z extent of 4 and a radius of 0.7072 x 4 = 2.8288. Low X 0, high X 4, low Z 0, high Z 2 gives the same radius, 2.8288. Low X 0, high X 1, low Z 0, high Z 1 gives 0.7072.

Packet size (synthetic): a data block of 1024 bytes is submitted with a length of 1112 (1024 plus 88). A block of 1025 bytes is refused and logged.

Colour sheet, cell geometry (synthetic, from the block rule):
- Cell column 0, row 0 (x0 = 5, y0 = 5): left hue 0, right hue 58; top saturation 3, bottom saturation 31.
- Cell column 5, row 7 (x0 = 155, y0 = 61): left hue 300, right hue 358; top saturation 227, bottom saturation 255.

Colour sheet, derived indices from a colour record (synthetic, from the stated rules; the last column is the derived value index before any clamp):
- (255, 0, 0): saturation byte 256, hue 0. H = 0, S = 63, V = 64 (outside 0 to 63; see Edge cases).
- (64, 128, 64): value 128, saturation 128, hue 120 (green is the maximum: 120 + 60 × 0 / 128). H = 60, S = 31, V = 32.
- (128, 128, 128): grey, hue marker 361. H = 181 (outside 0 to 180), S = −1 (outside 0 to 63), V = 32.

Colour sheet, HSV to RGB (synthetic, integer conversion with truncation): H = 0, S = 63 (saturation 255), V = 63 (value 252) gives (252, 0, 0). Hue 90 degrees with saturation 255 and value 252 is sector 1 with fraction 128, so q = floor(252 x 127 / 255) = 125 and the colour is (125, 252, 0); round-to-nearest would give 126.

Colour sheet, vertex alpha: every colour sheet vertex has alpha byte 128.

View and projection (synthetic): a view record of width 640 and height 480 gives an aspect term of 480 × 1.0 / 640 = 0.75. The vertical field of view is 90 degrees.

- Display size, synthetic: a factor of 0.05 gives 51 by 44 (after the clamp to 0.1). A factor of 1.5 gives 512 by 448.
- Fade quantizer, variant A (bounds 0.008 and 1.0): v = 0.5 gives index 128 (a' = 0.504); v = 0.998 gives 255; v = 0.001 is raised to 0.008 and gives index 2 (a' = 0.01575); v = 1.5 gives 256 (a' = 1.0); v = −0.001 sends the fixed (1.2, 0.5, 0.5, 1.0). Variant B (bounds 0 and 1.0): v = −0.001 gives index 0, and v = 0.25 gives index 64 (RGB scale 0.25).

Held items (root-box values from the disc models; W_k from SPEC-0004):

- Tor, 98 Wooden Sword (resource 544, box min Z −0.417, size Z 2.188): weapon-drawn set, drawn at W_0. Clear, drawn at the sheath point with scale min(2.6 / 2.605, 0.6) = 0.600 (the cap applies).
- Tor, 103 Sword of the Ancients (549): sheathed scale 2.6 / |−0.854 − 3.724| = 2.6 / 4.578 = 0.568.
- Tor, 105 Altadorian Sword (551): sheathed scale 2.6 / 4.748 = 0.548.
- Tor, 107 Iron Shield (561, box min X −0.773, size X 1.546): shield-raised set, drawn at W_5. Clear, drawn at the back point with scale 1.6 / |−0.773 − 1.546| = 1.6 / 2.319 = 0.690.
- Tor, 110 Legendary Shield (564): stowed scale 1.6 / 2.396 = 0.668.
- Roberta, 113 Enchanter's Wand (554): weapon-drawn set, drawn at W_0. Clear, not drawn.
- Roberta with a shield equipped: no shield is drawn in either flag state.
- Tor with item 126 worn: no shield, weapon or held entry is drawn.
- Held entry 36 Rope Gun (575): drawn at Q (from W_0) only while the weapon-drawn flag is clear.
- Synthetic axis swap: with W_8 the identity at origin (1, 2, 3) and f = 1, the stowed shield matrix has the axes (−1, 0, 0), (0, 0, 1), (0, 1, 0) and the origin (1, 2, 3). With W_6 the identity, the sheathed weapon matrix has the axes (0, 1, 0), (0, 0, 1), (1, 0, 0). With W_0 the identity, Q has the axes (0, 0, 1), (0, 1, 0), (−1, 0, 0).
- Thief: a steal sets the countdown to 180; the item is drawn at scale 0.45 on 180 draws (the countdown goes 180 → 0), and on the 181st draw the flag clears and nothing is drawn.

- Held entry, state test: a held entry with count 0 is not drawn in any state; hero action state 15 does not occur.
- Status tint: a strength status sets colour-override slot 0 to (1.0, 0.3, 0.2, 1.0) in mode 1; at counter 30 the weight is 1, so each colour-scale record becomes (1.0, 0.3, 0.2). A fire status leaves slot 0 off.
- Opacity table: submit scale 1.0 gives index 256, a' = 1.0; scale 0.5 gives index 128, a' = 0.504; scale 0.004 is clamped to 0.008 and gives index 2, a' = 0.01575. No path in play gives a negative scale.
- Background colour: default bytes (128, 128, 128, 0); a volume colour (204.6, 99.5, 30.2, 127.4) is stored as (205, 100, 30, 127).
- Sky dome choice: slot 0 gives dome 1; slot 150 gives 1; slot 151 gives 2; slot 156 gives 2; slot 157 gives 3; slot 185 gives 3; slot 186 gives 2; slot 187 gives 1; slot 192 gives 1. Loading slot 160 and then slot 170 reads resource 3 once (the second load finds dome 3 already loaded); then slot 186 reads resource 2; then slot 0 reads resource 1 even though its bytes equal those of resource 2.
- HUD coin spin (synthetic): the coin's time value falls by one frame step per drawn frame, so at 60 ticks per second after 60 drawn frames the angle is 1 radian; one full turn takes 6.28 s (about 377 drawn frames). The coin is not drawn while a side panel is open or no money line is shown, and its time value does not advance on those frames.
- Held-item glow (synthetic): Tor with Mote 163 in word 2 and a weapon of item id 100: weapon glow model 299 + (100 - 98) = 301, tint blue (0, 0, 255), alpha 128. Tor with Mote 172 in word 1 and a shield of item id 107: shield glow model 307 + (107 - 106) = 308, tint grey (120, 120, 120). Roberta with any Mote: no glow.
- Culling frustum (synthetic): view record 640 by 480, distance field 100.0: the lateral extents at the far plane are 100.0 on the first lateral axis and 75.0 on the second (tangents 1.0 and 0.75).

## Open questions

Default if unresolved (rewrite choice): for the depth buffer of Meridell and every other area, use floating-point or 24-bit depth with near 1.0 and far 10,560 and ignore the area's depth scalar. The integer conversion of the stored depth (truncation toward zero is assumed) and the exact cap on the scalar are not needed to reproduce the visible result.

1. Resolved: **Which object list is which.** The two object lists are the two petpet asset slots (character slot records, SPEC-0030 section 4); the scene sub-list is the current world's character slot table; the optional second sub-list is the neighbour world's table, behind the pairing test; the special object is the active petpet; the main character is the focus hero. They are characters, not the placed objects of SPEC-0028 (see Object lists and the special object). The nine-entry array of the per-frame pass is not described here. **[High]**
2. Resolved: **Which objects use each draw class.** Every character render object uses class B; class A is only its base. Class B adds a skip-once flag and a horizontal level-of-detail flag before the class A model-part draw (see Draw classes). **[High]**
3. Resolved: **Purpose of the far render-state set.** It is a vertex-program state bit: in the skinned programs it selects rigid one-bone skinning (first bone slot only) instead of the three-bone blend; lighting still runs, the specular alpha term of program 6 is off, and no fog is set (see Far state at flush). **[High]**
4. Resolved: **Translucency value.** The sub-record is the character's fade record; the value is the fade level 0 to 128 (its writers are listed under Sort order), and a fading character is composited as one sprite with alpha = level (see Translucency). The second use reported in the list submit path reads a different field (the spawn record combined with the slot flag word), so there is no conflict. **[High]**
5. **Parallel pass family.** A group of parallel submit paths reads a render-related word stored on the object. The flush does not use that word. What the parallel pass is (shadow, reflection or overlay) is not encoded anywhere readable, and the flush ignores the word (class D). Default if unresolved (rewrite choice): draw no extra pass for it. **[Low]**
6. Resolved: **Lighting consumer.** Every lit draw builds its light block from the four slots of its light context (see Light slots and the lit draw). Resource 527 is never read; its rig is the scene default lights, which are fixed constants (Lights, Scene default lights). **[High]**
7. Resolved: **Light blend direction.** Toward the new value: stored' = 0.95 × stored + 0.05 × new. **[High]**
8. Resolved: **Side bit of the part classifier.** The side left out of the model-part and mesh-leaf mask is the far side (mask 0x2F; group nodes use 0x3F). The sixth bit is the near plane; any corner in front of it makes the box clipped. So the far side only culls (all corners beyond it) and never forces clipping (see Culling). **[High]**
9. Answered: **Volume builder trigger.** In the known boot image the builder has no direct caller in the call index; the volume count starts at zero, and the builder is the only known incrementer. No runtime population is established. Calls from unindexed or external modules remain outside this result. **[High for the boot call/data survey]**
10. Answered: **Objects tested by the volume functions.** The known boot call index has no direct callers for the volume rejection routines, so no object type is established as using this test in the boot image. With the count remaining zero in known paths, the test rejects nothing there. Calls from unindexed or external modules remain outside this result. **[High for the boot call survey]**
11. Resolved: **Distance metric.** The submit cull, the far test and the shadow caster choice use 3D distance (x, y, z). Only the class B level-of-detail flag uses horizontal distance (x, z). **[High]**
12. Resolved: **Draw-distance properties.** They are 16-bit monster-table fields (row offsets 0x32 and 0x34; 0x0C and 0x0E in the companion rows); 2500 is their editor range bound, not a default (the default is 0). Nothing in the program reads them: the whole boot program holds only five 16-bit loads at offset 0x32 or 0x34, and all five belong to other data (a fixed-point property accessor pair, a model part record of the part draw loop and two stack locals), while the loads that follow a character's row pointer and companion-row pointer use other offsets. Level modules reach rows only through host calls, and none returns these fields. The far test uses the fixed 2500 (50 units squared) and the submit cull uses 350 units. **[High]** Test vector: two rows with draw distances 100 and 300 draw at the same distances, 50 units for the far test, and a row with draw distance 0 is not hidden.
13. Resolved: **Switch node flag.** It is not a flag: the word is the third subtree link of a type 4 node, set from the disc offsets at model load; the draw walk draws that subtree first and then the node's own children, with no child choice (see Linked subtree). **[Medium-High]**
14. Resolved: **Per-part material draw and the skinned-palette upload.** The part loop, the leaf draw (colour record, material group, mesh packet), the pose-matrix groups, the palette matrix M = bind translation · pose · world, its stored form and the upload to quadword 128 of vector unit 1's data memory are described in Character part draw and the skin palette. The three pose arrays are the main, sub-part and face outputs of the shared skeleton pose pass of the same character, in that order (High; see the same section). The one remaining doubt is the exact data-memory base (Medium-High). Default if unresolved (a rewrite choice): skin every vertex with W · P · (p − bind) per bone. **[High for the matrix and the arrays; Medium-High for the upload address]**
15. Resolved: **Fog.** Neither state set enables fog, and the mesh GIF tags have the fog bit clear; the game has no fog (see Fog). **[High]**
16. **Other vector-unit routines in the render-adjacent list** (some copy, divide and square-root helpers) are not described. No render rule in this spec depends on them; Default if unresolved (rewrite choice): none needed (class D).
17. Resolved: **View point.** It is the camera record's own position (the base point), read after the camera update and before the shake is applied to the render copy, so a shake does not move it; in the follow camera it is the end-of-tick eye (see Object submission and culling). **[High for the unshaken source; Medium-High for the eye]**
18. Resolved: **Kind 42 exclusion.** The excluded value is base actor state 42, the parked state (creation, reset, release and inactive slots). Every bind sets base state 1, so in play the rule hides only the Gelert Assassin's waiting mirror images (see Drawable test and Open question 79). **[High]**
19. Answered: **Scene sub-list entry flag.** It is the slot record's load-result word; only loaded slot records are submitted. **[High]**
20. Resolved: **Overlay rectangle.** The merged screen rectangle is the fade composite region of a fading character (see Translucency). **[High]**
21. Resolved: **Vertex pass.** The caller is the per-caster shadow unit walk and the data is a type-33 unit's hull points (at most 50 per unit); 128 is only the scratch size, not a batch limit of ordinary drawing (see Vertex pass). **[High for the caller and data; Medium for the scratch size]**
22. Resolved: **Fault behaviour.** Both are debug messages only. A 19th or later insert is not added (the count stays 18) and the frame goes on; a flush count mismatch prints and returns as usual (see Draw list capacity). **[High]**
23. Resolved: **Blob shadow.** There is no blob shadow; that draw is the 3D book's open-angle animation (0.0144 per draw down to 0.12, see 3D book open angle). Shadows are the stencil shadow volumes, gated by the data word 0x2ECCD0, with the hero and kind-key-720 world characters as casters (petpets only while 0x2D7608 is nonzero, 0 on disc) and the three nearest casters within 50 units kept (see Character shadow volumes). **[High]**
24. Resolved: **Colour sheet: player-visible or debug.** Never drawn: kind 8 is a UI node kind, and the only UI database on the disc (resource 532) has no kind 8 node among its 1,366 nodes (see Colour sheet, Reachability). The clean side does not need to draw it. **[High]**
25. Answered: **Colour sheet: pad buttons.** Not needed for the game: the sheet is never drawn (item 24). **[Not specified]**
26. Answered: **Colour sheet: derived ranges.** Not needed for the game: the sheet is never drawn (item 24). **[Medium-Low]**
27. Answered: **Colour sheet: blocks C and D.** Not needed for the game: the sheet is never drawn (item 24). **[Low]**
28. Answered: **Colour sheet: update trigger.** Not needed for the game: the sheet is never drawn (item 24). **[Low]**
29. Answered: **Colour sheet: HSV rounding.** Not needed for the game: the sheet is never drawn (item 24). **[Low]**
30. **Projection: first argument.** The first argument of the projection setter (the view record's other field) has no effect on the picture described in Camera projection; Default if unresolved (rewrite choice): ignore it. **[Low]**
31. Resolved: **Projection: aspect convention.** The camera projection shows the 512 by 448 buffer as 4:3: focal 512 with a factor 0.8 on x (409.6) and a factor 448 / 480 on y (477.87); see Camera projection. The separate culling-frustum build uses the aspect term 1.0 × view height / view width (0.75 for the 640 by 480 record) with no zero-width guard, and runs once per frame on the global view object (View and projection build, step 3; Open question 84). **[High for the camera projection arithmetic; Medium for the frustum's role]**
32. **View and projection: source matrices.** The placement of the eight source matrices in the object and the product convention (row or column order). The later products follow the same pattern (mostly). Default if unresolved (rewrite choice): build the view and projection in the rewrite's own convention from Camera projection; the original's storage order is not visible in the picture. **[Medium]**
33. **Projection wrapper callers.** The callers of the short wrapper around the projection setter do not matter to the picture: the projection is the one in Camera projection (class D). **[Low]**
34. Resolved: **Billboard draw: users and inputs.** The routine described as a camera-facing billboard is the HUD's spinning coin model (HUD coin model and camera-facing quads): one caller (the HUD 3D pass), gate and inputs as listed there. The loop-B marker quad is a separate builder. **[High for the coin; Medium-High for the marker quad's caller path]**
35. **Other render-side routine.** One further routine with a render-side role and one callee has no part in the draw path described here (class D; omit it). **[Low]**
36. **Caller of the corner builder.** Resolved (proactive (open close-out), Medium-High): the standalone box corner builder has no direct caller and no data reference in the boot program, and none of the 163 distinct level modules (all 173 share their code) calls it, jumps to it or holds its address. It is unused; the model-part path builds its own eight corners from the part box, so the part box rule stands and the rewrite omits the standalone builder.
37. Resolved: **Sixth-side check.** Bit 0x20 is the near side: after the frustum pass, any corner in front of the near plane makes the box clipped (1), before the guard-band pass (see Culling). **[High]**
38. **Resolved in part — Bounding radius use.** The known direct caller is the type-16 foliage patch builder; the composite patch sphere is used for foliage frustum culling (SPEC-0004, Radius from a box). No support connects it to generic model-part bounds, collision or effect range. Other consumers are not known; Default if unresolved (rewrite choice): use it only for foliage patch culling. [High for the known caller and foliage path; Low for indirect/external consumers]
39. **Packet submit owner.** Whether the packet helper and its header formats belong to this spec or to the platform spec (SPEC-0008). Spec ownership is a documentation choice (class D); the platform spec holds the packet formats. **[Medium]**
40. **Weight and parameter routines.** Which spec uses the weight routine and the normalised parameter (the camera path spec SPEC-0027 or rendering), and where their four helper values come from: no draw or camera result described in this spec depends on them (class D). **[Low]**
41. Resolved: **Chain point mover** is the per-part padded bounding box of the model draw (see the Per-part padded bounding box paragraph). Answer (2026-11-09, High; the padded-box open question is closed): the box is built over the world positions of the first N joints of each bone table of the model (not a shared transformed point list); the margin is one quarter of max(size.x, size.z), NOT of the largest of the three components; the low corner is lowered by the margin and the size raised by twice the margin on x, y and z (w margin 0); the result is stored in the bone table header's first two vectors (SPEC-0005 Layout). It moves no points. Its only caller is that class's per-part classifier. **[High]**
42. **Two-sprite submit.** A routine writes five low-level draw-state values for a 2D rectangle or sprite. Spec ownership is a documentation choice (HUD spec SPEC-0023 or this one); no rule here depends on it (class D). **[Low]**
43. **Two-sub-record wrapper.** A wrapper calls three draw helpers on two sub-records. Spec ownership is a documentation choice (SPEC-0004 or this one); no rule here depends on it (class D). **[Low]**
44. **Render-state reset.** A routine zeroes about 40 state fields and sets three of them to 1. Its caller is not known; Default if unresolved (rewrite choice): start every frame from the render state this spec describes (class D). **[Low]**
45. **Corner routines and N-point clip callers.** The callers of the two corner routines and of the clip test with counts other than eight. The fixed-point steps of the screen-corner routine are assumed to use the screen centre 2048 (Default if unresolved (rewrite choice); no described draw depends on it). **[Low]**
46. Resolved: **Vector clip-status loop and transform helper.** Neither routine is referenced anywhere: no call, jump, table entry or built address in the boot program or in any of the 173 level modules names them, so the camera and the renderer never run them. Implement nothing for them; the clip tests that matter are the corner and per-triangle tests described above. **[High]**
47. Resolved: **Default colour fourth byte.** The two defaults belong to different objects: 0x80808080 to 2D draw records (alpha 128 = opaque) and 128, 128, 128, 0 to the view object's background colour, whose fourth byte is the source-alpha weight (alpha / 128) of the late tint, so a default of 0 draws no tint (see Default colour word and the frame order, step 10). **[Medium-High]**
48. Resolved: **Fade level users.** The two 257-entry tables are the draw colour-scale tables, built at start-up: variant A is the per-draw opacity, used by every material draw list entry with the default colour; variant B is a darken that sets RGB only. Each entry is a colour-scale record (a vector and a lane mask). Which draws use variant B is Open question 77. **[High]**
49. Resolved: **Fade range bounds.** They are written once, by the start-up graphics init, when the tables are built: 0.008 and 1.0 for variant A, 0 and 1.0 for variant B. **[High]**
50. Resolved: **Variant A reset and default vector.** A negative level sends the separate entry, the fixed scale (1.2, 0.5, 0.5, 1.0) on all four lanes; the four-1.0 vector is the default colour scale (Medium). Every caller uses variant A (variant B has no caller): the shadow pass and the world draw send 1.0, the material draw sends the entry opacity or the default's 1.0 (see Variant A callers). No entry carries a negative opacity in play: the only negative submit scale comes from an inert debug grid, so the reddish tint is never shown (see Variant A callers). **[High]**
51. Answered in part: **Flush entry point callers.** The world draw uses a list-setting entry with identity matrix and camera position; an object-tree draw uses a count-storing entry with a model-to-world matrix and eye position. The stored block is caller-selected, serving both cases. One list-setting entry has no identified direct caller, so its use and the complete caller map remain open. **[Medium]**
52. Resolved: **Far-copy formula.** The extrusion length is 16.0 (data word 0x2ECCC8); the stored 25.0 is never read. The swept segment from the unit centre along 16 L is clipped against the five camera culling planes, near copy = moved point + A, far copy = near copy + B, and for an unclipped unit far copy = near copy + 16 L (see Character shadow volumes, step 4). **[High]**
53. Resolved: **Packed light words.** The two words (data words 0x323368 and 0x32336C) are vertex colours for additive counting, +8 and −8 modulo 256 per channel with alpha 128, chosen by whether the camera is on the polygon's outward side (see Character shadow volumes, step 7). **[High]**
54. Answered for resource 504 unit 0: the 14 listed edges are the hull-edge candidates; the 12 triangles create 18 unique triangle edges, of which four are internal triangulation diagonals joining adjacent triangles with equal or near-equal stored normals. Those four diagonals are omitted from the edge list. This establishes the count discrepancy for this unit; it does not establish that all type-33 units use this pattern. **[High for unit 0]**
55. Resolved: **Light direction source.** One direction per caster per frame, from the caster's own character light context: the weighted sum of its point-light slots and its strongest directional slot, normalised, with y at most −0.4 (see Character shadow volumes, step 2a). **[High]**
56. Resolved: **Sphere-first variant.** It has no caller, so it never runs and is not part of the game's behaviour; the clean side leaves it out. **[High]**
57. Resolved: **Cap selection.** Every triangle is emitted once: facing the light (flag 0) with its near copies, facing away (flag 1) with its far copies (see Character shadow volumes, step 6). **[High]**
58. Resolved: **Per-tick advance of a light.** The records are swing-trail colour gradients, not lights; their advance is the sample ageing of the swing trail (SPEC-0034 3.8). **[High]**
59. Resolved: **Light slot consumer.** The lit draw reads the four slots and sends the light block with the draw; meshes drawn with the lit programs 1, 2 and 6 are lit through them (see Light slots and the lit draw and question 6). **[High]**
60. Resolved: **Preset triggers.** The eight presets are the swing-trail colour gradients of SPEC-0034 section 18; the attacker's element picks one at each trail start (SPEC-0034 3.8). **[High]**
61. Resolved: **Template constants.** 0.02 and 0.3 are a segment-length fade range, 1 is the colour steps per advance and 2.8 the texture length per unit of trail length, all in the swing-trail colour gradients of SPEC-0034 section 18. **[High]**
62. Resolved: **Character own entries.** A character's own entries are the four scene default lights (the scene's light object receives them at area activation), the same four that scene light contexts take, so all four slots are full before the area's light set is walked (see Character lights). **[High]**
63. Resolved: **Displacement rule.** With no free slot, the candidate is the weaker of two unlocked type 2 occupants of slots 2 and 3 (slot 2 on equal weights; the unlocked one when exactly one is locked), or the only unlocked type 2 occupant; a point light between 0.9 r and r cross-fades the candidate and locks it instead of taking its slot; a dynamic light with no candidate replaces the weaker of slots 2 and 3 (see Character lights). **[High]**
64. Resolved: **Falloff distance.** The type 3 weight uses the distance (the length); only the radius gate uses the squared distance. **[High]**
65. Resolved: **Weight -1 entries.** They are never stored: the own entries leave no free slot, the cross-fade path stores nothing, and the dynamic rule needs a weight above 0. The radius-0 entry is an edge case of its own (see Edge cases). **[High]**
66. Resolved: **Dynamic point lights.** They come only from the particles of light-emitter emitters (37 emitter records in 167 definitions), one entry per spawner, listed once per live particle, at most 200 per pass; the dynamic rule is the last case of the slot choice (see Character lights). Still open: the flag and zone-key words of a dynamic entry are not written and are taken to be 0. **[High; Medium-Low for those two words]**
67. Resolved: **Character slot content.** Each slot holds a light entry and a weight; its stored vector is the entry colour times the weight, and the lit draw reads the slots (see Light slots and the lit draw). Both character mesh paths apply the resulting block (Open question 73). **[High for the slot content]**
68. Resolved: **Character walk users.** The walk runs once per main-loop pass for the focus hero, the second record and every character of the active-character list, while a world is loaded and the game mode is below 9 (see Character lights). The second record is the player's live petpet. The zone value is the zone byte of the ground entry the character stands on, tested with the character's world id, as in the SPEC-0006 cell flag; a character that fails keeps its previous light state (see Character lights). **[High]**
69. Resolved: **Sky dome drawing.** The dome is drawn first each frame, from four layers chosen by mesh-child role id (1 static, 2 and 3 rotating about world Y, 10 drawn with the atmosphere blend factor as its opacity), in world space at the world origin, with its materials' own depth state and no fog (see Sky domes). Its textured materials also scroll their UVs in the vector unit (UV mode 2, two layers in opposite directions; Sky domes, SPEC-0003). **[High]**
70. Resolved: **Resource 528 and the rotating layers.** Its mesh children have role ids 0 to 4 and 100 to 107, so a dome renderer would fill the layers of roles 1, 2 and 3. Roles 2 and 3 rotate about world Y by −0.0009 and −0.0006 radians per frame; role 1 is static (see Sky domes). Nothing loads resource 528 and the dome files 1 to 3 hold only roles 1 and 10, so no dome layer rotates on disc. Whether 528 was meant as a fourth dome has no answer in the data. Default if unresolved (a rewrite choice): do not load resource 528. **[High for the roles, axis, rate and sign; Medium-High for unused]**
71. Resolved: **Mesh kind per path.** There is no per-packet mesh kind: every batch tag has record kind 4, and the program is the pass's material program word (SPEC-0003, group offsets 0x44 and 0x54). Programs 0 and 4 are unlit, 1 and 5 lit, and 2 and 6 skinned and lit. On disc layout B always uses program 0, layout C program 1 or 4, and layout A program 2 or 6; 26 layout-C batches name a program-2 material but belong to a role-0 container child that the model-set loader skips, so they are never drawn (SPEC-0004, Vertex program). Programs 0, 1, 2 and 6 use fixed slot orders and ignore the attribute mask. **[High]**
72. Resolved: **Skinned extra alpha term.** It is the specular alpha term of program 6, near state only: c = 2 (n′ · V̂)(n′ · L′) − V̂ · L′, raised to the power p by the float bit-pattern approximation with the offset z (group offsets 0x6C and 0x68), and A = trunc(255 f min(1, X)) when X > 0, with f the brightness factor (data words 0x2ECAE4 to 0x2ECAF0). There is no header flag. See Light slots and the lit draw. **[High for the arithmetic]**
73. Resolved: **Character mesh path.** The main-processor and first-vector-unit path loads the character's light block into the vector unit and applies the same light formula: normal rotated by the first bone, alpha 128, RGB min(255, L × colour byte / 16), position skinned by three bones, with no draw colour scale (see Light slots and the lit draw, Character mesh path). Which render objects take it: none on disc. A render object takes this path only when the first child of its model container is a type-12 chunk, and every container starts with a type-13 child (Character part draw and the skin palette). **[Medium-High for the path's arithmetic; High for the choice of path]**
74. Resolved: **Light templates.** They are the swing-trail colour gradients of SPEC-0034 section 18, not lights, and reach no light context or lit draw. **[High]**
75. Resolved: **Draws without a light context.** Draws that pass a scene light context (the four scene default lights): world items, the HUD coin, the 3D book, the item showcase and the test shape. Draws that pass the character's own light state: the focus hero's held items and the slot 0 attachment of world-table characters. Everything else passes none and uses the last block sent: the thief's stolen item, the slot 6 and slot 1 attachments, the ten-record projectile array, effect models, the world record lists, the inert grid, the aim-view model and the small world-object draw methods (full list under Light slots and the lit draw). The earlier statement that only three kinds of draw pass a context was wrong. **[High for the call sites; Medium for the role names]**
76. Answered: **Dome step rate.** One pass of the main loop is one 1/60 s tick paced to the vertical blank (SPEC-0026, main loop steps 16 and 17), so the dome steps 60 times per second, fewer only while passes overrun a blank. **[Medium-High]**
77. Resolved: **Darken table users.** None. The variant B quantizer has no caller, jump or stored pointer in the boot program, so the darken table is built and never used. **[High]**
78. Resolved: **Unlit overflow.** The unlit programs 0 and 4 do not limit byte × s, and the GS keeps the low 8 bits, so the value wraps modulo 256. Disc level-geometry colour bytes never exceed 245, so no wrap happens with scale lanes up to 1.041; the default draw colour gives 1.0 (see Edge cases). **[Medium-High]**
79. Resolved: **Base state 42 and townsfolk.** The drawable test and SPEC-0040 use the same base actor state. Every bind (streamed, forced, and spawn by key) sets state 1 after object reset and before the first draw; the streaming pass also returns kept characters found in 42 to state 1. Bound townsfolk therefore draw normally from their first active frame. State 42 remains the parked state, and the drawable rule hides a bound character only when its behavior places it there; the Gelert Assassin mirror images are the documented in-play case (Drawable test; SPEC-0040, Behavior 3). **[High]**
80. Resolved: **Reader of the class B level-of-detail flag.** There is none: no direct load, no wider load or copy of the render object and no level-module access reads the flag's word. It is written by the class B draw and never used (see Draw classes, Level of detail). **[High]**
81. Resolved: **Other clip sets and the second GS variant.** The far-600 and far-10560 pairs and the GS variant over [N + 0.001, F] are rebuilt with the projection but unused in play: the composed far-600 pair has no reader, and the far-10560 pair and the second GS variant are read only by two part-matrix packet builders that nothing calls. Every draw uses the far-5280 pair and the [N, F − 0.001] variant (see Camera projection). **[High]**
82. Resolved: **World mask word.** It is the script display mask: 0 at world set-up and written only by SetWorldDisplayMask as 2 x m; scripts use m = 1 and 2 in most modules, bit combinations in a few, and 15 in packs 696 and 714, so low bits 1 to 4 and, in packs 758, 1023 and 1358, bits up to 10 can be enabled (see Culling, World mask word). Display nodes carrying low bits exist in packs 699, 703, 706, 837, 1066, 1176, 1184 and 1230 (world tree) and 758, 1023 and 1358 (placed-object meshes); none exist in packs 696 and 714, so the call with 15 there changes nothing drawn. The pairing sets are in SPEC-0006. **[High]**
83. Resolved: **Shadow resolve.** A 16-bit off-screen buffer is cleared to 120, the volumes add +8 or −8 per polygon with wrap, only the top bit of each channel is kept, and the buffer is subtracted from the frame with the vertex colour of data word 0x2D7610. The texture-alpha register is set once at display set-up so a set alpha bit reads 128, the alpha bit is set in every texel, and the drawn alpha is 32, so the test passes everywhere; shadowed pixels lose exactly 8 per channel (the word has no writer and always overwrites the packet default of 32). See Character shadow volumes, step 8. **[High]**
84. Resolved: **Culling frustum (once called the vertical-scale-1.0 projection).** It is the per-frame culling-frustum build of View and projection build, step 3: called by the main draw routine on the global view object with the camera matrix, by a helper of the character-position code and by a wrapper. Its five planes are read only by a sphere test that nothing calls, so it has no visible effect. **[Medium]**
85. Resolved: **Fade bit consumer.** None. The vertex program stores the fade bit (value 2) and its level, but no path reads either, and the selector of the 257-entry fade-state table has no caller in the boot program or in the level modules. Only the clip bit and the far bit have effects (see Far state at flush). The type-4 node fade of SPEC-0004 goes through the draw colour scale instead. **[High]**
86. Resolved: **Near depth value and the atmosphere scalar.** The vertex program reads the near-plane depth value B only as the homogeneous depth of clipped-polygon vertices at or nearer than the near plane; it is not a fog constant, and there is no fog. Its copy is taken when the camera render record is created and is not refreshed when the atmosphere scalar lowers B (quirk; see Per-triangle clipping in the vertex program). **[High]**
87. **Held-item pass, other entries.** Resolved for the array (High): the fixed array of ten records drawn in step 5 of Held items and character attachments is the enemy projectile pool of SPEC-0029 (Behavior 9): ten records in slot order 0 to 9, each 0x15D0 bytes. For each record whose active word is nonzero the draw rebuilds the record's frame from its forward axis and the world up vector (side axis = the normalised cross product of the two, then the up axis re-derived from side and forward), and, when the visual kind (record offset 0x15A8) is below 5 and the record has a trail effect, plays the trail at the record's position; the record's model draw sends no light context. This is the visual update of SPEC-0029 (Visual update). A record that is not active draws nothing. Test vector: with records 2 and 7 active and the rest free, the draw visits records 0 to 9 in order and builds frames for 2 and 7 only. Resolved for the second walk (High for the walk, Medium-High for the slot roles): after the world draw flush the pass walks each world table again from the highest slot down to slot 0 and draws directly, for each active character with a render object, its slot 0 attachment model (or, when set, the one-shot thrown model of the Black Knight sword and the Werelupe King boomerang), the slot 6 model at attach point 5 and the slot 1 model at an attach point chosen by event bits; see "Second walk and attachment slots" in Held items and character attachments for the gates, the matrices, the slot roles and test vectors. **[High for the array and the walk; Medium-High for the slot roles]**
88. Resolved: **Action state 15.** No code sets hero action state 15: the set-state method is never called with 15, and the calls with a computed state never produce it (one only restores a saved state). State 15 is an empty route, so the held-entry test for it never holds and is left out. **[Medium-High]**
89. Resolved in part: the held-item pass stores shield and weapon matrices in the shared auxiliary effect object, and the Tor-only consumer is the element glow on his weapon and shield (Held items and character attachments, step 4). Roberta has no consumer. For item 36, the render path starts a separate rope-manager object with a 200-record segment pool using model-registry key 159; active transforms advance by one-unit steps up to the measured span length, without a separate endpoint model. The update runs once per qualifying held-object render traversal, with no fixed ratio to hero ticks established. Gameplay manager calls request cues 0xB7, 0xB8 and 0xB9 at launch, attachment and during reeling; these use records 183–185 of the main sound bank, resource 347. The referenced sample ids are given in Held entry. No world resource has a type-8 child that registers key 159; the key is registered at boot from resource 7 (header value 61), a one-unit five-sided prism with a gold and tan striped texture (see Held items and character attachments, step 3), so the rope is drawn as a thin gold cord. An earlier reading that the key had no geometry was wrong. Visible endpoint contact and recognizable audio identities remain open. **[High for the key, the model, the cue records, resource counts and render call placement; Medium for segment spacing and orientation; Low for audible interpretation]**
90. Resolved: **Held entries of other characters.** A non-hero character gets a held entry only at set-up, from the 16-bit field at +0xAE of its placement record, which is −1 (none) in all 3,036 placement records; so no non-hero shows a held entry on disc (the thief's stolen item is separate). The hero's held entry is only ever item 36 (see Held items and character attachments). **[High]**
91. Resolved: **Hit-flash consumer.** The status and stun tints are the slot 0 blend (see Colour-override slots under draw step 6). Slot 1, which the hit flash sets to (1.5, 0.5, 0.5, 1.0), is read by the part draw: while it is on, every part of the hit character is drawn with that vector as its colour scale instead of its slot colour. Still open: what the ramp's own vector and colour bytes (SPEC-0034 section 32) feed. **[High for the slots and the part draw; Low for the ramp bytes]**
92. Resolved in part: **Reference point frame of program 6.** The reference point is the vector sent with the matrix record (the view point argument of the class B and part draws), and it is subtracted from the skinned positions. For the hero the draw sets that vector to the camera's world position minus the hero's position (the +0x50 vector of its motion record), just before queueing it (High). For a placed object the view point is the object's own vector (Placed-object draw frames; High). Reconciled: the palette folds the character's world matrix into the bone matrices, and that matrix's translation row is the character's own position vector (the same vector that is subtracted in the view point; the character update copies it into the matrix every tick). So the skinned positions P are in world space, and the reference point is C − Q with Q the character's world position, which is not in the same frame. The specular vector is V = normalize((C − Q) − P) = normalize(C − 2Q − local offset of the vertex from the character's origin), not the vector toward the camera. It is correct only for a character at the world origin. The shine of program 6 therefore depends on where the hero stands; reproduce it as written. The same camera-minus-position vector is used by the held-entry draws and the slot 0 attachment draw of world-table characters; the body draw of region characters and the petpet takes its view point through the draw method of its class, which is not shown here (Medium that it is the same vector). Test vector: C = (10, 5, 0), hero at Q = (4, 0, 0), a skinned vertex at P = (4, 1, 0.5): reference point (6, 5, 0), V = (2, 4, −0.5), |V| = 4.5, V^ = (0.444, 0.889, −0.111); the vector toward the camera would be (6, 4, −0.5). Default if unresolved (a rewrite choice): none needed for the formula; use the vector above for the hero and for every skinned draw. **[High for the hero's vector and the frames; Medium for region characters and the petpet]**
93. Resolved: **Program 6 on screen.** The surface is solid and the highlight is additive: all textured program-6 groups use texture function highlight 2 (colour = texel × output colour / 128 + output alpha; alpha = texel alpha), so a vertex alpha of 0 gives a plain lit solid surface and a positive specular alpha brightens it. The part is drawn in one pass with its own texture, and the vertex alpha is never an opacity. No on-screen comparison has been made; the user's description of Roberta's black hair agrees. **[High for the GS arithmetic; Medium-High that the on-screen look matches]**
94. Resolved: **Clip record after a viewport change.** The viewport never changes size: the display mode is fixed at boot to 512 by 448 (SPEC-0027, Screen size), and the projection changes made in play keep the size, so W/2 = 256 and H/2 = 224 stay valid (see Per-triangle clipping in the vertex program). **[High]**
95. Resolved: **Meridell mountain draw (proactive).** The mountains of pack 1023 are 24 ordinary mesh leaves (material groups 10 and 11) of the level display tree, in one type-4 node with cull value 1e8, fade length 0 and pass byte 0, under the column-0 terrain node. That column is 1 in all 61 grid rows, so no cell switches them. They are drawn after the dome in the first sub-list of their texture buckets, with depth test GEQUAL on a 24-bit Z buffer, opaque, with no fog, no back-face test, no level-of-detail switch and no hysteresis. The only per-frame gates are the eight-corner frustum test and, for the story-state cloud bowls beside them, the layer flags. The unresolved part is the port: nothing in the data makes the mountains unstable, so the causes are listed as port mistakes in Meridell mountains and the depth buffer. **[High for the data; Medium for the ranking of causes]**
96. Resolved (R-0172): **Gnarfas's shadow hulls.** The second shadow kind on disc is the hull block of Gnarfas (40 units on 39 distinct main-skeleton joints; joint 20 carries two), identical in both of his regions; no hull is attached to a chain-link joint, so his hanging chains cast no moving shadow of their own (Character shadow volumes, Gnarfas's hulls). **[High]**
97. Resolved (R-0202): **How a ghost's level is drawn.** The level only sets the sprite alpha of the character fade layer (Translucency, Layer and blend details); the character's parts are drawn unchanged into the layer, opaque parts included, with normal depth writes, and the blend is source-over at level / 128 with no depth write. A port that has no per-character alpha should draw the ghost to its own layer and blend it at level / 128. Default if unresolved (rewrite choice): blend the whole silhouette at the opacity itself. **[High for the path; Medium for the layer's pixel format]**
