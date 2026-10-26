# SPEC-0004: Static models and meshes

- Answers: R-0004
- Status: Draft
- Disc build: SCUS-97367 v1.00 (same fingerprint as SPEC-0001)
- Confidence: per section. Container, node tree, packet and vertex layouts, strip rule, bounds and material binding: High. Load status of mesh resources: Medium (per line). Load-time handling of type-8 containers: High for the dispatch and the child codes below; Medium for what each code stores. World placement: High for the level mesh, Medium-High for placed objects. Vertex program choice, batch tag fields, two-pass batches, skinning arithmetic, the absence of back-face culling and multi-packet leaves: High. Vertex colour scale: High; unlit overflow: Medium-High; lighting: Medium-High (the light model is in SPEC-0046). Vertex pass size (128 positions per pass): Medium-Low. Coordinate handedness: Medium-High. Type-33 block: High for the header, unit, list and count rules, the radius rule and the load-time conversion; Medium for the sphere centre and the triangle index byte; Low for the bank index mapping. Packet pointer relocation at load time: Medium. Item model registry key and the disc id mapping: High. Attach-point table and per-frame attach matrices: High; attach-point roles: Medium-High.

## Summary

All drawable geometry is stored as **node trees**: hierarchies of group nodes and mesh leaves with exact bounding volumes. Each mesh leaf owns one **packet**, a prebuilt PS2 vector-unit (VIF) command stream. It holds a series of triangle-strip **batches** of at most a few dozen vertices each. A batch stores its vertex attributes as separate arrays (all positions, then all colours, and so on), and the order of the arrays identifies which of three vertex layouts it uses. A per-vertex flag says whether that vertex completes a triangle. Each mesh leaf names one material group from the shader list (SPEC-0003), which supplies its textures, its blend state and the vertex program (unlit, lit, or skinned and lit) of each pass. Level geometry is stored in world coordinates. Characters, props and placed objects are stored in their own model space and placed by a 4 × 4 matrix. Held items (weapons, shields and other carried items) are standalone model-space trees registered by item id and placed on one of nine per-character attach points. With this spec the clean side can turn every mesh on the disc into indexed triangle lists with positions, normals, UVs, colours and skin weights, and draw a static level.

## Where it lives on the disc

Node trees are found in four places, all inside IDM chunk files (SPEC-0002):

| Where | Chunk | Space | Seen on disc |
| --- | --- | --- | --- |
| Mesh container ("core object") | type 8, whose children are type-13 (or type-12) chunks, each one node tree | model space | 214 type-8 chunks, 1,459 children |
| Level display tree (world packs) | type 3, payload is one node tree | world space | 191 world packs |
| Placed objects (world packs) | type 29, each object holds an inline node tree | object space + matrix | 121 packs, 3,524 objects |
| Instanced meshes (world packs) | type 20, each record holds an inline node tree | object space + per-instance matrix | 23 packs, 1,265 instances |
| Standalone type-13 chunks (resources 544–575, the held-item display models, and resource 7) | type 13, one node tree per chunk (see "Standalone type-13 chunks") | model space, placed by an attach-point matrix, a held-entry or world-item matrix, or the item showcase | 65 chunks, 208 mesh leaves |

A model set (chunk type 18 with children; SPEC-0002) groups one model's pieces as its own chunk chain: shader list (2), texture chunks (4), skeleton table (a flat type-18 chunk; SPEC-0005), mesh container (8), and, in 571 of the 1,339 model-set wrappers that have IDM children (43 %), a type-33 chunk (see "Optional per-model block (type 33)"). The character outfit resources 504–519 use the same pieces as a flat top-level chain: 2, 4, 18, 8, 33.

**Character regions keep their materials inside the model sets (High).** A character region (a resource with a type-36 table; 575 on disc) has no top-level shader list, texture chunk, mesh container or type-33 block. Its chain is the skeleton and animation chunks 22 to 27, then the model sets, then the type-36 table. Every top-level type-18 chunk of a region is a model-set wrapper (1,157 in all; 10 regions have none, one has 8). A kind-3 record's model-set index (SPEC-0006, data +0xA4) is the position of its wrapper among the region's top-level type-18 chunks in file order, counted from 0, and at most 7. Inside a wrapper the children come in this order: shader list, its texture chunks (their count is the list's own count B), the flat bone table, the mesh container and, in some, a type-33 block. The texture chunks of a wrapper belong to the wrapper's own shader list in file order; no other list is searched. A reader that looks for the shader list and textures at the top level of a region, as it can for an outfit, finds none, and then every part of every region character is drawn with no material: no texture and no colour. This is the first thing to check when region characters are white and untextured while the heroes (flat outfit chain) are fine. All 1,157 wrappers and all 16 outfits have a type-13 first mesh child, which is the condition for the per-part colour slots (SPEC-0040, Behavior 26, item 9).

## Layout

All values are little-endian. Offsets inside a node tree are **relative to the start of the payload that holds the tree** (the payload of the type-13/12 child, of the type-3 chunk, or of the type-29 chunk for placed objects). Offset 0 means "none".

### Mesh container (chunk type 8)

- The payload starts with 0, 8 or 12 zero bytes (the chunk's extra-header length E), then a chain of child chunks of type 13 (or 12), back to back, then a 16-byte IDM end marker (type 0).
- Each child's header bytes 14–15 hold a u16 **role id**. The child payload is one node tree with its root at payload offset 0.
- 64 type-8 chunks on the disc have no children (just the end marker). They are empty slots in world-pack chains; skip them.
- A character's main mesh is the first child. Later children with role ids 10–20 are optional extra parts that the game attaches by role. World packs register each child (other than role 401) under its role id as a **global model id** of the shared common-object registry: id 0 to 97, 98 to 190, 191 to 282 or 283 to 322, with the index inside the range being the id minus the range start (an id above 322 registers nothing, though the child is still relocated). Role 401 is the separate special path. The models are then found by id: item models (SPEC-0021, Item model registry), flowers and other placed models (SPEC-0028, section 16) and the model particles of effects (SPEC-0034, section 2.3). Across the world packs the ids seen are 0 to 401 (150 distinct values), and the same id in two packs is the same model slot. Role meanings for character containers are in Child role dispatch below.
- **Load-time handling (High)**: when the world loader reaches a type-8 chunk, it walks the child chain. For ordinary type-12/13 children it applies the node-tree pointer fix-up to the child payload using that payload as the relocation base and passes the current shader list for mesh-leaf material handling (SPEC-0003). The fix-up applies to the child node tree; the type-8 container header/pad is not itself relocated by this handler. Empty containers are skipped. The walk order and the other chunk handlers are in SPEC-0006.
- **Child role dispatch (High for the source and dispatch; Medium for stored-field meanings)**: this outfit loader walks type-13 children inside the type-8 container and reads each child's role ID from header bytes 14–15 (+0x0E), separately from the child's chunk type at byte 4. It then acts on that role ID:
  - code 0: sets a count from the child's 16-bit field at offset 0x5C. In a mesh leaf that field is the material group index (above), so whether code 0 reads the same field is open (Open questions).
  - code 100 (0x64): stores a pointer, taken to point at the child. The slot it is stored in is not identified.
  - code 200 (0xC8): stores a pointer in the same way.
  - code 400 (0x190): reads one material group (0x270 bytes, SPEC-0003) into a state word.
- **Types 2000 and 2001 (0x7D0 and 0x7D1)**: the dispatcher links pointers and runs a fixup step for each of these two chunk types. They are the book text and book glyph layout chunks of resource 576. See SPEC-0024, "Book records", for their layouts.

### Node tree

Every node starts with an 80-byte common header:

| Offset | Size | Type | Meaning |
| --- | --- | --- | --- |
| 0x00 | 4 | u32 | Node type: 0 = group, 3 = group with a visibility selector, 4 = group with a distance cull, 1 = mesh leaf |
| 0x04 | 4 | f32 | Cull distance squared (see Behavior). 1e8 means "never". |
| 0x08 | 4 | f32 | Bounding-sphere radius R |
| 0x0C | 4 | f32 | 0 |
| 0x10 | 12 | 3 × f32 | Bounding-box minimum corner (x, y, z); +0x1C is 0 |
| 0x20 | 12 | 3 × f32 | Bounding-box **size** (not the maximum corner); +0x2C is 1.0 |
| 0x30 | 12 | 3 × f32 | Bounding-sphere centre |
| 0x3C | 4 | f32 | R squared |
| 0x40 | 4 | u32 | Offset of first child |
| 0x44 | 4 | u32 | Offset of next sibling |
| 0x48 | 4 | u32 | Offset of parent |
| 0x4C | 4 | u32 | Serial number; not needed |

Bounds are in the same space as the vertices below the node. They are exact: for every mesh leaf, the box equals the extent of its own vertex positions, and the sphere is tight (the farthest vertex lies exactly at R). For group nodes, they cover all descendant vertices. All 8,519 checkable nodes match.

Node sizes and extra fields:

| Type | Size | Extra fields |
| --- | --- | --- |
| 0 | 0x50 | none |
| 3 | 0x60 | +0x50 u32: **zone column** (only the low byte is used). In a level display tree the node and its whole subtree are drawn only while the zone-visibility query of SPEC-0006 (Binary grids) passes for this column with the active block (key −1); the query passes when no grid is loaded. 1,232 type-3 nodes occur in the level trees: the column is 0 in 175 of them and 1 to 73 in 1,057 (115 packs). It is not an always-visible selector, because column 0 is an ordinary column of the matrix. In all 120 packs that have a zone grid, every type-3 column is below the width of the grid's block 0; the 72 packs without a grid have only 0. +0x54..+0x5F zero |
| 4 | 0x80 | +0x58 f32: fade length F (0 or below = no fade); +0x5C u32: story-layer flags, a 32-bit mask (not u16; 0 in 19,445 of the 19,659 type-4 nodes of the level display trees; the layer rule is in SPEC-0046, Culling); +0x62 s8: **pass byte** (0 to 3 on disc; in the 19,659 level-tree type-4 nodes: 0 in 7,143, 1 in 4,649, 2 in 5,157, 3 in 2,710; the sky dome's translucent layer uses 2). In the level draw it chooses which of four ordered sub-lists of a material bucket the leaves below the node join (below, Drawing a node tree); +0x64 f32: fade end distance E; +0x70, +0x74, +0x78 u32: three more child offsets. Only the +0x78 link is ever non-zero on disc and only that link is walked. F and E give the distance fade of SPEC-0046 (Fade level quantizer and draw colour scale); the layer flags gate the subtree (SPEC-0046, Culling; in character models they are matched against the character draw mask, below); the other bytes are not read |
| 1 | 0x90, followed by its packet | see below |

Mesh leaf (type 1) fields after the common header:

| Offset | Size | Type | Meaning |
| --- | --- | --- | --- |
| 0x50 | 4 | u32 | Offset of the packet. It is always this node + 0x90. |
| 0x5C | 2 | u16 | **Material group index** into the shader list's groups (SPEC-0003). Always below the group count. |
| 0x5E | 1 | u8 | Packet kind: 0 = one packet (all but 13 leaves); 1 = CPU-animated, double-buffered leaf (see "Multi-packet leaves"); 2 = a raw vertex-program record, sent whole (none on disc) |
| 0x64 | 4 | u32 | Total vertex count over all batches |
| 0x68 | 4 | u32 | Number of batches |
| 0x6C | 4 | u32 | 1 |
| 0x70 | 32 | 8 × f32 | **Not read by anything** (High): the level draw, the character draw, the held-item draw, the packet submit, the relocation pass and the vertex programs use only the packet offset, the material group index and the packet kind of the leaf. The values look like an exporter bound: a unit axis x, y, z with an angle in 0 to π (the angle is the fourth float), then a point x, y, z and a fourth float that is 0.5 in 20,595 of the 21,503 leaves on disc (771 resources, one count per leaf occurrence). The axis has length 1 in every leaf; the angle is 0 in 231 leaves and at least 3.14 in 63; the point lies inside the leaf's bounding box in 15,189. A rewrite may skip these 32 bytes and draw identically |

In a type-3 level tree, nodes are stored back to back, each mesh leaf followed by its packet. Walking the links reaches every node.

### Standalone type-13 chunks

Top-level type-13 chunks are mesh containers outside any type-8 container. There are 65 of them: one in each of resources 544–575 (32 chunks, each followed by bytes that are not chunks), and 33 in resource 7, which also holds a type-2 chunk before them and a type-4 chunk after them. In resources 544–575 the extra-header length E is 0. In resource 7 the first type-13 chunk has E = 12. The payload starts at chunk + 16 + E, and node offsets are payload-relative, as above.

Each payload is a node tree:

- The root is a group node (type 0) at payload offset 0, with cull value 1e8.
- Its first child is a visibility group (type 3). That group's first child is an LOD group (type 4).
- The LOD node's child offset at +0x78 leads to a second group node (type 0). Its children are mesh leaves (type 1), linked by first-child and sibling offsets. Each leaf's packet is at leaf + 0x90, as above.
- Some chunks hold a second LOD group in the upper part of the tree.

A walk from payload offset 0, following first-child, sibling and LOD child links and marking each node and packet, covers every payload byte of all 65 chunks (nodes and packets), with no uncovered bytes and no errors. There are 208 mesh leaves in total: 2 to 8 per chunk in resources 544–575, and 1 to 4 per chunk in resource 7. Every leaf uses packet kind 0. Across the leaves: material group index 1 to 180 (174 distinct values), vertex count 8 to 392 per leaf, batch count 1 to 9, and packet quadword count 15 to 1,141.

Resource 7 is loaded at boot as part of the item data (see "Load status of mesh resources"). Resources 544–575 are the held-item display models, loaded at run time by the item display slots (SPEC-0002 for the resource rule, SPEC-0021 for the slots).

Materials of resources 544–575 (High): these resources have no shader list of their own. When a display slot finishes loading one, it fixes the tree up against resource 7's material list, and every held-item draw uses that list (SPEC-0046, Held items and character attachments). Their mesh leaves use material group indices 1 to 5 and 74 to 180 of resource 7's shader list, which is loaded at boot. Resource 7's own 33 containers use indices 6 to 73; resource 575 also uses 6 to 8 and 35 to 39, shared with resource 7's containers. Both sets fit inside resource 7's list (112,996 bytes, about 181 groups of 0x270 bytes). So the clean side resolves a leaf's material group index in any of 544–575 against resource 7's shader list and its texture chunk.

Test vector: resource 544 holds one chunk of size 10,848 (payload 10,832 bytes).
- Root group at 0, first child at 0x50.
- Visibility group at 0x50, first child at 0xB0.
- LOD group at 0xB0, with its +0x78 child at 0x130.
- Group at 0x130, first child at 0x2410.
- Mesh leaves at 0x2410, 0x11D0 and 0x180, linked as siblings. The leaf at 0x180 has its packet at 0x210, with quadword count 252.
- The walk covers 10,832 of 10,832 payload bytes.

Confidence: High for the layout and the coverage; Medium for the node types of the upper tree, which are taken from the node format above.

### Item model registry key (High)

Every standalone type-13 tree is kept in the item model registry (SPEC-0021) under an item id. The id comes from the chunk header's u16 at byte 14 (+0x0E), the same header field that holds the role id of a type-8 container's children:

- **Run-time models (544–575).** When an item display slot finishes loading one of these resources, it checks that the chunk type is 12 or 13 (otherwise the slot stays empty) and registers the model under id = header u16 + the slot's id base. The base is 98 for the weapon slot and the shield slot, and 0 for the fixed slot and the world-item slot.
- **Boot models (resource 7).** The 33 containers are registered at boot under id = 98 + header u16.

Header values on disc, and the ids they give:

| Resources | Header u16 | Slot (base) | Registry ids | Items |
| --- | --- | --- | --- | --- |
| 544–551 | 0–7 | weapon (98) | 98–105 | Tor's swords |
| 552–559 | 13–20 | weapon (98) | 111–118 | Roberta's wands |
| 560–564 | 8–12 | shield (98) | 106–110 | shields |
| 565–574 | 50–59 | world item (0) | 50–59 | potions |
| 575 | 36 | fixed (0) | 36 | Rope Gun |
| resource 7 (33 models) | 37–41, 47–49, 51, 52, 54, 55, 57, 58–75, 77, 83 | boot (98) | 135–139, 145–147, 149, 150, 152, 153, 155, 156–173, 175, 181 | boot item models (156–173 are the Motes; 175 is the treasure map) |

So each run-time model registers under its own item id. In resource 7 the containers are stored in the order 47, 48, 49, 51, 52, 37–41, 58–75, 77, 83, 54, 55, 57, not in id order.

Slot buffer sizes (bytes): weapon 51,200; shield 47,104; fixed 20,480; world item 61,440. Each loaded resource fits its slot's buffer. The slot rules (when each slot loads, unloads and reloads) are in SPEC-0021.

### Load status of mesh resources

Facts about which mesh-bearing resources the game loads. "Reached" packs are the world packs that can be entered from the new-game start (SPEC-0006, Reach). SPEC-0006 gives 174 of 191 packs as reached, and the world-pack and character-region counts below use that reach (all but 15 world packs and 2 packs of a closed pair are reached). A character region is one of the resources stored after its world pack (SPEC-0006); it is loaded only by a region-slot load (SlotLoad) in a level script module. A region named only in a slot-buffer reservation (SlotCreate) is not loaded (SPEC-0006, Region slots).

- **Standalone containers.** Resource 7 holds 33 standalone type-13 containers and is loaded at boot as part of the item data (High). Resource 9 (a shader list, a type-8 container of 38 models, a type-34 effect-definition chunk and two textures) is loaded at boot as well (Medium). The 32 standalone type-13 containers in resources 544–575 are held-item display models, loaded at run time by the item display slots (SPEC-0002 rule; High). Resource 575 (the model of item 36) loads on the hero's first update; 544–564 load when the weapon or shield is equipped; 565–574 load when a potion is spawned as a world item. They are not unused.
- **Sky domes.** Resources 1–3 (a shader list, a mesh container with two mesh children, a texture chunk) are the sky domes; one is loaded on every world load, chosen by the world's slot index (SPEC-0002; High). Resource 0 is a byte copy of domes 1 and 2 and is never selected (Medium-High). Resource 528 has the dome layout (a mesh container with 13 mesh children) and no loader (Medium-High).
- **Outfit chains** 504–519 are loaded in play by the outfit routine (base number plus an offset from the equipped item; Medium-High).
- **Character regions.** Of the 555 character regions stored after world packs, 436 are loaded in play by a region-slot load in a reached pack's level script, 59 only by scripts of packs that are not reached, and 60 by nothing. Every branch that reaches a load call counts (in pack 909 one call receives eight region indices from eight branches). Never loaded, for example: 700, 731, 817–821, 848, 915, 958, 1048, 1060, 1109, 1349 and 1383. Loaded only from a pack that is not reached, for example: 1304 and 1309 (named in a reached pack's reservation, but loaded only by pack 1259). Confidence: Medium for the counts (reach basis above); High for each script's region set.
- **Pet packages** 534–543 (one character each, for pet kind k at 534 + k) load only when a saved pet of that kind is present (Medium; see SPEC-0002).
- **Totals.** Of the 926 resources that hold a texture, a mesh or a skeleton (SPEC-0002):
  - 646 load in reached play: 174 world packs, 436 character regions and 36 fixed resources (among them the sky domes 1–3 and the item-36 model 575).
  - 132 fixed resources load when a play condition holds: 544–574 when the item is equipped or spawned in the world, and the front-end pictures 577–677 when their screen is opened (SPEC-0002). For 13 of them no route in reached play is established: 547, 551, 552, 553 and 562 belong to items that have sources only in packs that are not reached on SPEC-0021's own, narrower reach basis, and 589, 592 and 595–600 belong to front-end screens with no known route (SPEC-0035). Which of the area-map pictures 601–677 are shown depends on each area's map id.
  - 10 load only with a saved pet (534–543).
  - 61 load only from packs that are not reached (2 world packs, 683 and 1003, and 59 character regions).
  - 77 have no known load route: 15 world packs, 60 character regions, and the fixed resources 0 and 528.

  A coarser grouping of the same 926 resources gives 642 reached, 10 with a saved pet, 61 from packs that are not reached and 213 other. It differs from the classes above only in where it puts the 132 condition-loaded fixed resources and the fixed resources 1–3 and 575: it counts them as "other" (642 = 646 − 4; 213 = 77 + 132 + 4).

  Confidence: High for the per-resource content; Medium for the load classes (the world-pack and region counts use the 174-pack reach, an upper bound for play).

The clean side loads resources only on request, so these facts do not change any loader; they tell a tool which models it will never see in normal play.

### Packet

A packet is a 16-byte header followed by a VIF command stream. Its total length is its quadword count × 16.

| Offset | Size | Meaning |
| --- | --- | --- |
| 0x00 | 16 | A GS GIF tag (public GS documentation) for the **first batch**: NLOOP (bits 0–14) = the first batch's vertex count; EOP (bit 15) = 1; PRE (bit 46) = 1; PRIM (bits 47–57); FLG = 0 (packed); NREG (bits 60–63) = 3; REGS (bits 64–75). |
| 0x02 | 2 | u16 **quadword count of the whole packet**, header included. It is stored in bits 16–31 of the tag, which the GS ignores. |
| 0x04 | 1 | u8 slot size N (tag bits 32–39): the number of entries in each attribute array of the batch, equal to the batch's vertex count |
| 0x05 | 1 | Bits 0–3 (tag bits 40–43): record kind, 4 (batch) in every batch tag on disc. Bit 4 (tag bit 44): "more passes follow". Bit 6 is the GS PRE bit. |
| 0x0C | 1 | u8 data offset (tag bits 96–103): quadwords from the tag to the first attribute array; 1 normally, 2 in two-pass batches |
| 0x0D | 1 | u8 number of GIF tags per batch: 1 normally, 2 for meshes whose material is mode 2 (two passes) |

The tag is also the vertex program's record header: the bits the GS ignores (bytes 0x02, 0x04, 0x05 bits 0–4, 0x0C and 0x0D) carry the fields above. All 227,451 batch tags on disc have record kind 4; 937 of them have the "more passes" bit. A batch carries no choice of vertex program: the program comes from the pass's material record (see "Vertex program").

PRIM and REGS values seen:

- **Textured**: PRIM 0x25C (triangle strip, Gouraud shading, texture on, alpha blending on, context 2) with REGS ST, RGBAQ, XYZ2. Used exactly when the mesh's material group has mode 1 or 2.
- **Untextured**: PRIM 0x24C (the same with texture off) with REGS NOP, RGBAQ, XYZ2. Used exactly when the material group has mode 0.
- Mode-2 meshes have a second tag per batch for the second pass: PRIM 0x05C (textured, context 1). Its material state is the group's second register set (SPEC-0003).

### Batches inside the VIF stream

The stream is plain VIF1 codes (public PS2 VIF documentation): u32 code = immediate (bits 0–15), num (bits 16–23), command (bits 24–30). Each batch is:

1. STMASK (command 0x20) with mask word 0x3F3F3F3F. This makes a "masked" UNPACK write only the W component.
2. One UNPACK (commands 0x60–0x7F) per vertex attribute. Every UNPACK of a batch has num = N, the batch's vertex count (N is 2 … 74 on disc), except that the UV array of a two-pass batch holds 2N entries (see "Two-pass batches"). With d the tag's data offset (1, or 2 in two-pass batches), attribute array *k* goes to vector-unit address d + k·N. The masked flag UNPACK goes to address d, into the W lane of the positions. All UNPACKs have the FLG bit set (double-buffered addressing); ignore it.
3. MSCNT (command 0x17), which runs the vertex program on this batch. The last batch of a packet has no MSCNT; the engine starts it from outside. Don't use MSCNT to detect batch ends; use the next STMASK or the stream end.
4. Between batches: an UNPACK V4-32 to address 0 with num = the tag count (byte 0x0D). It holds the next batch's GIF tag(s), in the same form as the packet header. Its NLOOP is the next batch's vertex count.

Decode each attribute array with the standard UNPACK rules: V3-32 = 3 floats per vertex; V2-32 = 2 floats; V4-8 = 4 bytes (unsigned when the UNPACK's USN bit, immediate bit 14, is set, which it is on disc); masked S-8 = 1 signed byte.

### Two-pass batches (High)

A batch of a mode-2 mesh (two passes, SPEC-0003) differs from a one-pass batch as follows:

- It has two GIF tags (byte 0x0D = 2). The first has the "more passes" bit and data offset 2. The second has PRIM 0x05C and data offset 0.
- Its arrays start at vector-unit address 2 + k·N.
- Its UV array holds 2N entries: entries 0 to N − 1 are the pass-0 UVs and entries N to 2N − 1 are the pass-1 UVs. Every other array is shared by both passes.

Example (resource 0 group 2, a sky dome): a batch of 46 vertices has its positions at vector-unit address 2, its colours at 48 and its 92 UVs at 94.

### Vertex layouts

The sequence of UNPACK formats identifies the layout. Counts are over all batches on the disc, with byte-identical duplicate resources counted once:

| Layout | UNPACK sequence (attribute order) | Used for | Batches |
| --- | --- | --- | --- |
| A, skinned | V3-32 position · S-8 flag (masked) · V4-8 colour · V3-32 normal · V3-32 skin weights · V4-8 bone slots · V2-32 UV | characters and creatures | 183,862 |
| B, plain | V3-32 position · S-8 flag (masked) · V4-8 colour · V2-32 UV | level geometry (type 3) and placed objects (type 29) | 160,762 |
| C, lit | V3-32 position · S-8 flag (masked) · V4-8 colour · V3-32 normal · V2-32 UV | lit props and objects | 14,606 |

(The masked flag UNPACK targets the same address as the positions and doesn't take an array slot. So in layout A the colours are at 1 + N, the normals at 1 + 2N, and so on.)

### Vertex program (High)

The vertex program that draws a batch is the pass's **material program word** (SPEC-0003, "Material record": group offset 0x44 for pass 0, 0x54 for pass 1). The layout does not choose it. Programs 0, 1, 2 and 6 read their arrays from fixed slots in a fixed order and ignore the pass's attribute mask; programs 4 and 5 (and the UV-mode handlers) place the slots by the mask. Any program value other than 0 to 6 ends the batch and nothing is drawn. Pairing over all batches on disc, counted per resource occurrence and including model-set wrappers:

| Layout | Program | Batches |
| --- | --- | --- |
| B | 0 (unlit), every B batch | at least 219,600 |
| C | 4 (unlit, skipping the normal slot) | at least 3,042 |
| C | 1 (lit) | at least 2,000 |
| A | 2 (skinned and lit): attribute mask 0x3B in 178,281, 0x3F in 2,824 | 181,105 |
| A | 6 (skinned and lit, with the specular alpha term of SPEC-0046) | 3,392 |

The B and C counts are lower bounds (the batches of two-pass packets are counted once). These counts use a different basis from the layout table above, which counts byte-identical resources once.

Edge case (Low): 26 layout-C batches, in model-set wrappers of resources 697, 698, 865, 903, 904, 948, 1216 and 1265, use a program-2 material. Program 2 reads six fixed slots, so their weights, bone slots and UVs come from the wrong arrays. What the original shows for them is not established. The clean side may draw them with program 1.

Attribute meanings:

- **Position**: x, y, z as f32, in the node tree's space.
- **Flag**: bit 7 (value 0x80, so the signed byte is negative) = "no triangle at this vertex". Other bits are 0. About 35–39 % of vertices are flagged.
- **Colour**: 4 unsigned bytes R, G, B, A. The scale follows the vertex program, not the layout (High; the light block and the per-draw colour scale s are in SPEC-0046, "Light slots and the lit draw" and "Fade level quantizer and draw colour scale"):
  - Lit programs 1, 2 and 6: 16 means 1.0 (4.4 fixed point); most values are exactly 16 on textured meshes, and program-1 vertices have bytes of 16 or less in 98 %. The vertex program computes a lit colour L (0 to 255 per channel) from the normal and the draw's lights, then outputs RGB = min(255, L × (byte / 16) × s_rgb) and alpha = min(255, 128 × (alpha byte / 16) × s_a), each truncated to an integer.
  - Unlit programs 0 and 4: 128 means 1.0. This includes layout-C batches drawn with program 4 (their alpha byte is 100 or more in 92 % of about 190,000 vertices). In layout B, alpha is 128 in 99 % of vertices. The output is byte × s per channel, truncated, with no limit (see Edge cases for values above 255).
  - The GS then modulates the texture by output / 128 (TFX = modulate): 128 leaves the texture unchanged and values above 128 brighten, up to about 2.0.
  - **Character meshes (High):** the draw colour scale s of a character part is not a constant. Before each part is drawn, s is set to the colour of the part's character colour slot (material group byte 0x27, SPEC-0003): (red, green, blue, 1.0) built from the character's colour word of that slot (SPEC-0040, Behavior 26). The vertex colour of an untextured NPC part is a baked grey shade (bytes 20 to 31, that is 1.25 to 1.94 after the divide by 16) and carries no hue, so the whole hue of such a part is s. Without it the part is white or grey. Textured parts are multiplied by s as well.
- **Normal**: unit vector (all checked normals have length 1).
- **Skin weights**: three f32 weights that sum to 1, sorted largest first in 99.8 % of vertices.
- **Bone slots**: bytes 0–2 are matrix-palette addresses, always multiples of 3 (three rows per bone matrix). Bone index = byte / 3. Byte 3 is never read (High): the skinned vertex programs load only the first three bytes of each slot vector, in the three-bone path and in the first-bone-only path of both program 2 and program 6. On disc it carries no meaning: over 4,279,291 vertices in 168,439 skinned batches it is 205 (the export fill byte) in 62.5 %, 63 in 9.7 %, 62 in 6.9 % and 0 in 4.7 %, with 73 distinct values in all; it equals byte 2 in 3.4 % of the vertices. Ignore it on load. Test vector: slot bytes (6, 3, 75, 62) select bones 2, 1 and 25; the 62 changes nothing. How bone indices map to the skeleton is part of SPEC-0005.
- **UV**: u, v as f32 in texture space. 0..1 spans the material's power-of-two sampling size (TW × TH, SPEC-0003). v = 0 is the texture's first (top) row (see Coordinate conventions). Values outside 0..1 are common and wrap or clamp per the material's CLAMP register.

### Placed objects (chunk type 29)

| Offset in payload | Size | Meaning |
| --- | --- | --- |
| 0x00 | 4 | s32 object count *N* (1 … 241) |
| 0x04 | 4 | u32 offset of the object offset table (0x10 in nearly all packs) |
| 0x08 | 4 | s32 count of a second table (meaning not decoded) |
| 0x0C | 4 | u32 offset of the second table |
| 0x10 + 4k | 4 | u32 offset of object *k* |

Object record (offsets from the object's start; offsets it contains are payload-relative):

| Offset | Size | Meaning |
| --- | --- | --- |
| 0x000 | 64 | Not meaningful on disc (identity or fill); ignore |
| 0x0C0 | 64 | **Placement matrix**: 16 × f32, column-major. Columns 0–2 are the object's X, Y and Z axes; elements 12–14 are the translation; elements 3, 7 and 11 are 0 and element 15 is 1. |
| 0x1D0 | 4 | Offset of the object's node tree (a type-12 chunk; payload = tree), 0 = none |
| 0x1D4 | 4 | Offset of an inline type-5 model set, 0 = none |
| 0x1D8 | 4 | Offset of an inline type-11 header, 0 = none |
| 0x1DC | 4 | Count of 0x70-byte part records |
| 0x1E0 | 4 | Offset of the part-record array |

The loader needs the type-5 model sets and the type-10 table before it in the chain than the placed-object chunk. This holds in every pack that has a type-29 chunk (SPEC-0006 load order).

### Instanced meshes (chunk type 20)

One mesh drawn at many places, such as trees or rocks. 23 world packs, 67 records, 1,265 instances.

| Offset in payload | Size | Meaning |
| --- | --- | --- |
| 0x00 | 4 | s32 record count *N* (1 … 8) |
| 0x04 | 4 | u32 offset of the record offset table (0x10) |
| 0x10 + 4k | 4 | u32 payload-relative offset of record *k* |

Record:

| Offset | Size | Meaning |
| --- | --- | --- |
| 0x00 | 16 | Unused (fill or leftover words) |
| 0x10 | 4 | u32 offset of the near mesh chunk, relative to the record start (0x30 in all 67 records: the inline chunk below). The loader turns it into an address |
| 0x14 | 4 | u32 offset of an optional far mesh chunk, relative to the record start (0 in all 67 records: none). The loader turns a non-zero value into an address |
| 0x18 | 4 | f32 fade length F (0 in all 67 records) |
| 0x1C | 12 | 3 × f32 distances D1, D2 and a third. On disc D1 is 4000 in 52 records, 400 in 3, 300 in 3, 200 in 3, 600 in 2, 100 in 2, 250 in 1 and 1000 in 1; D2 is 200 in 66 records and 1000 in one; the third is 32 in all 67. D1 is the draw distance: an instance whose distance from the eye is D1 or more is not drawn. D2 is the near/far switch: from D2 on the far mesh is used when the record has one; with no far mesh (every record on disc) the near mesh is used at all distances. The third value is not read. With F = 0 there is no fade; with F above 0 an instance between D1 − F and D1 away is drawn with opacity (D1 − distance) / F. |
| 0x28 | 4 | s32 instance count *M* |
| 0x2C | 4 | u32 offset of the instance array, relative to the **record** start |
| 0x30 | | An inline type-12 chunk whose payload is the mesh's node tree (object space) |

Instance (0x50 bytes): a column-major 4 × 4 matrix in the first 64 bytes. Columns 0–2 are the axes, column 3 at +0x30 is the translation with w = 1. Then at +0x40 a byte **zone column** (0 to 57 on disc: 1 in 367 instances, 6 in 331, 7 in 210, 3 in 65, 8 in 61, 0 in 48, and the rest spread over about 30 other values; the other three bytes of that word are 0). An instance is drawn only while the zone-visibility query of SPEC-0006 (Binary grids) passes for this column with the active block (key −1); the query passes when no grid is loaded. The remaining 12 bytes hold the floats 1.0, 2.0 and 3.0 in all 1,265 instances and are not read. Draw test, in order: the zone query, then the distance test D1 (the frame position of this step is in SPEC-0046, Per-frame draw order). Unlike type 29, these matrices may scale: the axes are always orthogonal, but their lengths range about 0.54–4.5 and differ between axes in about 30 instances. Draw the record's mesh once per instance, with that matrix.

Reconciling older readings of the byte at +0x40: the ids of 1, 5, 6 and 7 that were first noticed are only the commonest values. All 1,265 instances carry a zone column, and the column is a zone number of the pack's own ground (zone numbers 0 to 23 and 48 to 57 occur on disc), not an object kind or a model choice. Packs with no zone grid (pack 1148) draw every instance.

Worked instance (pack 706, record 0). The record has D1 = 4000, D2 = 200, no far mesh, no fade and 9 instances; instance 0 has the matrix columns (−1.1513, 0, −0.2382), (0, 1.3057, 0), (0.2382, 0, −1.1513), translation (−99.0448, 53.368, 12.7303) and zone column 5. The axes have lengths 1.1757, 1.3057 and 1.1757 (a scale of 1.18 across the ground, 1.31 in height, turned about the vertical axis). The pack's zone grid has block 0 of 10 by 10 cells; its row 5 has a 1 at column 5, so the instance draws while the camera stands in zone 5; row 2 has 0 at column 5, so it is hidden while the camera stands in zone 2. An eye at (0, 50, 0) is 99.9 units from the instance, below D1, so it draws at full opacity when its zone passes.

World position of an object's vertex = matrix × (x, y, z, 1). Apply the matrix exactly as stored. About 325 matrices have axes that are not quite orthogonal (determinant 0.95–1.0); don't orthonormalize them. 277 objects carry an identity matrix and are already in world space. Between 2 and 12 objects per field hold offsets that don't resolve (stale data); skip those references.

### Optional per-model block (type 33)

A type-33 chunk is a flat chunk with no nested tree. It sits in two places: as the last chunk of the outfit chains in resources 504–519 (16 flat chunks, one per outfit), and as a child of a type-18 model-set wrapper (571 of the 1,339 wrappers that have IDM children). All 587 chunks parse exactly to their payload end.

The block describes shadow-casting geometry for pieces of a character that are attached to bones. Each unit holds one piece: a bounding sphere, a list of the piece's edges, a list of its triangles, and its hull points, plus a bone matrix bank index. The size of each list is a count byte in the unit, so the whole layout follows from the counts.

Header (payload-relative offsets):

| Offset | Size | Meaning | Range and checks |
| --- | --- | --- | --- |
| 0x00 | u32 | n, the number of units | 2 to 40; never 0 |
| 0x04 | u8 | load guard (see "Load-time conversion" below) | 0 in all chunks on disc |
| 0x05 to 0x0F | | zero | zero in all chunks |
| 0x10 + 0x20k | 0x20 | unit k, for k = 0 to n − 1 (table below) | units follow each other with no gaps |

The lists that the units point to follow the unit table. Each list's length comes from its count, and the last list ends at the payload end.

Unit (0x20 bytes, unit-relative offsets):

| Offset | Size | Meaning | Range and checks |
| --- | --- | --- | --- |
| +0x00 | 3 × f32 | bounding-sphere centre (x, y, z), bone space | Medium: the stored centre is the sphere centre, not the mean of the points |
| +0x0C | f32 | sphere radius w: the largest distance from the centre to any hull point of the unit | exact in all 14,741 non-empty units; positive infinity in the 451 empty units |
| +0x10 | u32 | offset of the edge list | payload-relative on disc |
| +0x14 | u32 | offset of the triangle list | payload-relative on disc |
| +0x18 | u32 | offset of the hull point list | payload-relative on disc |
| +0x1C | u8 | edge count | 0 to 112 seen; 14 in the most common units |
| +0x1D | u8 | triangle count | 0 to 84 seen; 12 in most units |
| +0x1E | u8 | bone matrix bank index | 0 to 255 seen (62 distinct values in non-empty units); one bone index across three matrix arrays (see Bank index) |
| +0x1F | u8 | hull point count | 0 to 50 seen; 8 in most units |

Edge list: 4-byte records, count = unit +0x1C.

| Byte | Meaning | Range and checks |
| --- | --- | --- |
| 0 | first of the two triangles that share the edge | index into the unit's triangle list |
| 1 | second of the two triangles that share the edge | always different from byte 0 (checked on all edges) |
| 2 | first hull point at the end of the edge | index into the unit's hull point list |
| 3 | second hull point at the end of the edge | index into the unit's hull point list |

Triangle list: 32-byte records, count = unit +0x1D.

| Offset | Size | Meaning | Range and checks |
| --- | --- | --- | --- |
| +0x00 | 3 × f32 | unit normal of the triangle | length 1.0 in all triangles |
| +0x0C | f32 | 0.0 | 0.0 in all triangles |
| +0x10 | 3 × u8 | indices of the triangle's three hull points | all below the unit's hull point count; 24 triangles on disc repeat an index |
| +0x13 | u8 | facing flag for the current frame: 1 = front-facing to the light, 0 = back-facing | 0 on disc; rewritten every frame (SPEC-0046, "Character shadow volumes") |
| +0x14 | u32 | index of this triangle within its unit | 0, 1, 2, … in units 0 and 1 of resource 504 (other units not checked) |
| +0x18 | 8 bytes | zero | zero in all triangles |

Hull point list: 16-byte records, count = unit +0x1F.

| Offset | Size | Meaning | Range and checks |
| --- | --- | --- | --- |
| +0x00 | 3 × f32 | hull point position, bone space | |
| +0x0C | f32 | w = 1.0 | 1.0 in all points |

Load-time conversion (once per chunk): when the guard byte at +0x04 is zero, each unit's three list offsets (+0x10, +0x14, +0x18) that are non-zero are changed to memory pointers by adding the start of the payload. The conversion then sets the guard byte to 1, so converting the same chunk again does nothing. Zero offsets are left as they are.

Empty units (all three counts zero, sphere centre zero, radius positive infinity) have no edges, triangles or hull points, and the per-frame shadow code adds nothing for them.

Test vector (resource 504, the top-level chunk with n = 19, payload 13,600 bytes), unit 0:

- Sphere centre (0.0431, −0.1329, 0.0779); radius 0.6228.
- Counts: edges 14, triangles 12, bank index 2, hull points 8.
- Lists at payload offsets 0x3010 (edges), 0x270 (triangles) and 0x24F0 (hull points). The triangle list starts at the end of the unit table (0x10 + 19 × 0x20 = 0x270).
- Hull points (x, y, z), numbered 0 to 7: (0.5420, −0.1524, 0.3812), (0.4124, 0.3036, 0.3010), (−0.3412, 0.3036, 0.3010), (−0.4708, −0.1524, 0.3812), (0.3226, 0.3036, −0.1650), (−0.2513, 0.3036, −0.1650), (−0.4708, −0.1524, −0.2452), (0.5420, −0.1524, −0.2452). Two levels in y and two in z: a skewed box.
- Distances from the centre to the points: 0.5842, 0.6137, 0.6229, 0.5970, 0.5724, 0.5798, 0.6073, 0.5947. The largest is point 2, which matches the stored radius to the precision of the stored centre.
- Triangles (point indices), numbered 0 to 11: (0,1,2), (3,0,2), (1,4,5), (2,1,5), (5,4,6), (7,6,4), (7,0,3), (6,7,3), (7,4,1), (0,7,1), (6,2,5), (3,2,6). Triangle 0 has the normal (0.0000, 0.1732, 0.9849), the same as triangle 1; the coplanar pairs make six quad faces, each split in two.
- First two edge records: (triangle 0, triangle 9, point 0, point 1) and (triangle 0, triangle 3, point 1, point 2). Triangle 0 and triangle 9 both contain points 0 and 1; triangle 0 and triangle 3 both contain points 1 and 2.

Bank index (High for the rule; Medium for the roles of the three arrays): the byte b is one bone-matrix index counted across the render object's three bone-matrix arrays, taken in order. Let n0 be the first 32-bit word of the render object's first skeleton record and n1 that of its second skeleton record (the bone counts, Medium). If b < n0 the matrix is entry b of the first array; else if b < n0 + n1 it is entry b − n0 of the second array; otherwise it is entry b − n0 − n1 of the third. Each entry is a 4 × 4 matrix (64 bytes). The two thresholds are therefore the two counts, read from the render object each frame, not from the type-33 block. Test vectors with n0 = 30 and n1 = 10: b = 2 gives first array entry 2 (unit 0 of resource 504); b = 35 gives second array entry 5; b = 40 gives third array entry 0; b = 45 gives third array entry 5.

### Attach-point table (in the flat type-18 skeleton table)

Confidence: High for the layout; Medium-High for the roles.

The flat type-18 skeleton table (SPEC-0005) holds a 9-entry pose table at the payload offset stored at +0x24, with its entry count as a u16 at +0x3A (9 in every skeleton table on disc). This is the character's **attach-point table**: each entry names a bone and a fixed placement relative to it. Entry k is 0x30 bytes:

| Offset | Size | Type | Meaning |
| --- | --- | --- | --- |
| +0x00 | 4 | u32 | k, the entry's own index |
| +0x04 | 4 | s32 | bone index b; a negative value means the attach point is not updated |
| +0x10 | 12 | 3 × f32 | Euler angles (rx, ry, rz), radians |
| +0x20 | 12 | 3 × f32 | offset t in bone space; +0x2C is 1.0 |

Roles of the nine attach points:

| k | Role |
| --- | --- |
| 0 | Hand: the weapon in the hand, the held entry, a thief's stolen item; also the attack point for animation event bit 0x01 or when no event bit is set |
| 1–4 | Attack points for animation event bits 0x02, 0x04, 0x08 and 0x10 (SPEC-0005 events); bits are tested in that order and the first set bit wins |
| 5 | Shield on the arm (shield raised) |
| 6 | Weapon sheathed |
| 7 | Status display anchor above the head: the effects of the status display (SPEC-0034; effect definitions 0xBE to 0xCD) and the immunity-block effects start at its origin T_7 and follow it each tick (High) |
| 8 | Shield stowed on the back |

Attach point 0 is on the −X side of the skeleton (bone 19 in both heroes), which is the character's right side under Coordinate conventions (Medium-High). Characters that have no sheath or back slot (Roberta) store bone 0 with zero angles and a zero offset in those entries. How the draw uses points 0, 5, 6 and 8 is in SPEC-0046 (Held items and character attachments).

## Behavior

### Turning a batch into triangles

1. Read the N positions, flags and other attributes of the batch.
2. For each vertex *i* from 2 to N − 1 whose flag bit 7 is clear, emit the triangle (i − 2, i − 1, i).
3. Flagged vertices emit nothing but still advance the strip. Every batch is independent: strips never continue across batches.
4. The triangle's corner order within a strip alternates (98 % of strip runs are self-consistent under the usual even/odd swap). But the side the winding marks as "front" agrees with the vertex normals only about half the time. **Draw all meshes double-sided (no back-face culling; High).** No vertex program has a winding, area or cross-product test; the only per-triangle rejection is the clip test of SPEC-0046 ("Per-triangle clipping in the vertex program"). Use the vertex normals, not the winding, for lighting.

### Skinning and lighting in the vertex program (High for the arithmetic)

Programs 2 and 6 skin layout-A vertices. Let a vertex have bone-slot bytes b0, b1, b2, weights (w0, w1, w2), position p and normal n. Slot byte b addresses a bone matrix stored as three 16-byte vectors starting b quadwords after the bone-matrix base; vector j holds column j of the rotation in x, y, z and translation component j in w. So M_b p = col0 x + col1 y + col2 z + t, and R_b is the rotation part.

- Near state (far bit clear, SPEC-0046): P = w0 M_b0 p + w1 M_b1 p + w2 M_b2 p. The weights are not renormalised. The normal is n' = R_b0 n: the rotation of the **first** slot only, not blended and not renormalised.
- Far state (far bit set): P = M_b0 p and n' = R_b0 n.
- Both programs then light n' with the program-1 formula of SPEC-0046 ("Light slots and the lit draw"). Program 6 adds the specular alpha term of SPEC-0046 in the near state only.

How the bone slots map to the skeleton is part of SPEC-0005.

### Attach-point matrices (per frame, High)

Each character render object keeps two banks of nine 4 × 4 matrices: the current attach matrices W_0 … W_8 and the previous frame's copies. Resetting the render object sets all eighteen to identity.

Each frame, after the skeleton pose is built, for each entry k below the entry count whose bone index b is 0 or more:

1. R = Rz(rz) · Ry(ry) · Rx(rx), with standard right-handed axis rotations: Rz maps X to (cos, sin, 0) and Y to (−sin, cos, 0); Ry maps X to (cos, 0, −sin) and Z to (sin, 0, cos); Rx maps Y to (0, cos, sin) and Z to (0, −sin, cos). An angle whose magnitude is at most 0.001 is skipped (that factor is the identity). The original's sine and cosine match the standard functions to float accuracy.
2. W_k = B · [R | t], where B is bone b's current world matrix (the animated bone pose times the character's world matrix). A point p in attach space goes to B(t + R p).
3. Before W_k is replaced, its old value is copied into the previous-frame bank, so the previous bank always holds last frame's W_k (the original uses it for swept tests).

Entries with a negative bone index keep their last value (identity after a reset). In the rest of this spec and in SPEC-0046, X_k, Y_k and Z_k are the first three columns of W_k (its axes) and T_k its fourth column (the attach point's origin).

### Packet pointer relocation (load time, Medium)

- When a packet record is loaded, the loader turns two offsets stored in the record (its first two words) into addresses by adding the base address of the loaded payload to each one.
- It then sets the record's flag word to 1 and its third word to the caller's parameter. Which word is the flag is not confirmed (Open questions).

### Drawing a node tree

1. Start at the root (payload offset 0). Visit children via the first-child and sibling links. For type-4 nodes, also visit the extra child offset at 0x78, before the node's own children (the links at 0x70 and 0x74 are 0 on disc and are not walked).
2. **Distance cull** (type-4 nodes): let d be the distance from the camera to the node's sphere centre. Skip the node's whole subtree when its cull value (+0x04) is less than d² − R² (the camera is farther than the threshold from the sphere's surface). A value of 1e8 never culls. Type-4 nodes don't choose between their children; all visible children are drawn.
3. The type-3 zone column and the type-4 layer flags gate subtrees (for example parts that appear or disappear). A type-3 node is skipped, with its subtree, when the zone-visibility query for its column (the low byte of +0x50) with key −1 fails (SPEC-0006, Binary grids); when no grid is loaded the query passes. Layer flags of 0 mean "always visible". A type-4 node with nonzero layer flags is skipped, with its subtree, when its flags share no bit with the world layer mask, before its distance cull; the flags are read as a full 32-bit word, and the world layer mask (story layers from story flags, plus the world record's mask word) is in SPEC-0046 (Culling). Test vector: a type-3 node with column 5 under a camera whose zone row holds 1 at column 5 and 0 at column 6 is drawn; a sibling with column 6 is not.
3a. **Character models (type-13 trees of region and hero files).** The same layer-flag word is matched against the character draw mask, not the world mask. A type-4 node with flags 0 always draws its +0x78 subtree. With nonzero flags the +0x78 subtree is drawn only when the flags share a bit with the draw mask, and when it does not match the subtree is simply left out (type-4 nodes of character models have no children of their own). The draw mask of a region character is the region's mask word, which holds bit 30 (0x40000000) normally and bit 31 (0x80000000) while the region's colour override is on (SPEC-0040, colour presets and overrides), OR the character record's own mask (kind-3 record, SPEC-0006), which is −1 (no restriction) on every record on disc and so adds nothing. The Draik enemy writes the record's mask at run time from its damage stage (kind-3 record, SPEC-0006). On disc, 427 type-4 nodes in region character models carry flags 0x80000000 and 427 carry 0x40000000, so the two variants come in equal numbers: the bit-30 node is the normal part and the bit-31 node the part shown while the override is on. The focus hero and the active petpet are drawn with a mask of all ones, so every flagged part of their models draws. Test vector: a region at its normal mask 0x40000000 draws the 0x40000000 node and skips the 0x80000000 node; switching the override on draws the 0x80000000 node and skips the other.
3b. **Pass byte (level trees).** Mesh leaves are queued by material bucket (one bucket per shader list and texture chunk, in order of first use; at most 32 buckets). Each bucket keeps four sub-lists. The pass byte of the most recently entered type-4 node, 0 at the start of a frame, picks the sub-list of every leaf queued after it, and the flush draws sub-list 0 first and 3 last within a bucket, each in queue order (Medium-High). Test vector: leaves queued as A (pass 0), B (pass 2), C (pass 0) in one bucket draw in the order A, C, B.
4. Frustum culling by the node bounds is optional for the clean side. The bounds are exact, so they are safe to use. The original classifies the eight corners of each box: group nodes against all six sides (+x, −x, +y, −y, far, near), mesh leaves against every side except far, so a leaf partly beyond the far plane is not clipped; a box with all corners beyond one side is culled. The full rule is in SPEC-0046 (Culling).
5. Each mesh leaf draws its batches with its material group (SPEC-0003): textures, blend and alpha test. Mode-2 materials draw the mesh twice with the same positions, colours and normals: first pass with the group's first register set and the first half of the UV array, second pass with its second set (usually additive) and the second half (see "Two-pass batches").
6. **Vertex pass size (Medium-Low).** The original's vertex transform routine works on up to 128 positions per pass, and its scratch area is sized for 128 positions. Transform each position by the object's 4 x 4 matrix, then test the edges of the batch against four side planes (see SPEC-0046, Vertex pass). How the batches of this spec (a few dozen vertices each) map onto these passes is not confirmed; the clean side should not assume a 128-vertex limit on batch size. The draw order and per-frame culling that surround this step are in SPEC-0046.

### Spaces and placement

- Level display tree (type 3): vertices are already in world space. All 74,821 leaves lie within the root box; no transform is needed.
- Placed objects (type 29): multiply by the object's placement matrix.
- Mesh containers inside characters and model sets: model space. Characters are skinned by their skeleton (SPEC-0005) before placement.
- Lighting: level geometry (layout B, program 0) is unlit; its vertex colours are baked. Batches drawn with the lit programs 1, 2 and 6 are lit per draw from the draw's light block, with the light directions given in the drawn model's space (SPEC-0046). Layout-C batches drawn with program 4 are unlit.

### Coordinate conventions

- **Y is up.** Character skeletons span about 0.1 to 3.3 on Y, and level floors lie along X–Z.
- Characters are mirror-symmetric in X, and **face +Z**: feet point toward +Z and tails extend toward −Z, in every outfit and character model set checked.
- Units are the game's own; a character is about 3.3 units tall.
- Handedness (Medium-High): the space is right-handed with +Y up, and texture v = 0 is the top row of the image. No axis is mirrored. With Y up and facing +Z, the character's right side is at −X.
- Check: world pack 1023 (Meridell Village) has a shop sign whose texture carries readable text, placed in front of a sign board. Under the reading above the text reads correctly from the open side, with +u running to the viewer's right and +v running down the world Y axis, and the board lies behind the text. A left-handed reading mirrors the text. All three copies of the sign in that pack agree.

### Radius from a box (High for its foliage use)

For a foliage patch box with X and Z sizes W and D, the loader computes a base radius r = 0.7072 × max(W, D). Patch setup adds one half of the near model's bounding size to r and uses the result as the patch bounding-sphere radius for the per-frame foliage frustum test (SPEC-0006, Type 16; foliage draw in SPEC-0046). This is distinct from a mesh-leaf's stored sphere radius and squared-radius field in Node bounds. The constant is exactly 0.7072 in the computation; why that value was chosen is unknown.

Synthetic check: W = 4, D = 2 gives base radius 2.8288. With a near-model bounding size of 2, the patch radius is 2.8288 + 0.5 × 2 = 3.8288.

### Draw pages (Low)

Drawing state is grouped into numbered pages. Each page has its own pool of 0x30-byte records. A new record is taken from the current page's pool in order, and the request returns nothing when that pool is full. Setting the current page to a different id runs the page setup once for the change. Setting the same id again does nothing.

## Edge cases and error handling

- **Empty mesh containers** (64 type-8 chunks with only the end marker): no geometry.
- **Multi-packet leaves** (packet kind 1; High): the 13 leaves are resource 9, type-12 children 16 to 28. Children 16–23 use material group 20 and children 24–28 group 21, both with CPU vertex animator 3 (water, SPEC-0003, group byte 0x29). No other leaf names a group with a nonzero animator byte. The packet header has a quadword count of 0. It holds pointer fields (at packet offsets 0x1C–0x28 and 0x3C–0x40) to four small tables and to two byte-identical draw packets, which are a double buffer for the CPU rewrite:
  - at load, the fix-up relocates the packet's pointer fields and binds the group's CPU vertex animator into the packet;
  - each frame, the animator rewrites the draw packet of the current frame parity, and the draw sends the packet whose pointer is at packet + 0x3C on even parity and at packet + 0x40 on odd parity.
  Each draw packet decodes as a normal packet. Its batches use V4-32 positions (w = 1.0) with V4-32 UVs (z = w = 0) and a zero second array. A clean side that does the water animation itself may draw either copy.
- Stream padding: after the last batch, a packet may contain zero or 0xCDCD fill up to its quadword count. Stop parsing at an unknown VIF command or at the packet end.
- A batch whose UNPACK sequence doesn't match layout A, B or C: skip it and log it. None occur on this disc apart from the multi-packet case above.
- A placed-object offset outside the payload: ignore that reference.
- **Layout-C batches with a program-2 material** (26 batches): see "Vertex program".
- **Unlit colour above 255** (Medium-High): programs 0 and 4 output trunc(byte × s) with no limit. The GS (public PACKED RGBAQ format) keeps only bits 0–7 of each lane, so the value wraps modulo 256. For example, byte 200 with s = 1.5 gives 300, which the GS sees as 44.
  - On disc, level-geometry colour bytes are at most 245 (no 255s; 13.5 million vertices, 3.7 % with a channel above 127), so no wrap happens while every scale lane is at most 255 / 245 ≈ 1.041. The default draw colour gives 1.0.
  - The reddish negative-opacity scale (1.2 on red, SPEC-0046) would wrap the 30 level vertices whose red byte is above 212, if it were ever applied to level geometry; no such path is known.
  - Clean side: reproduce the wrap for strict parity; otherwise clamp to 255.

## Test vectors

Synthetic:

- Strip: 6 vertices with flags 0x80, 0x80, 0x00, 0x00, 0x80, 0x00 give triangles (0, 1, 2), (1, 2, 3) and (3, 4, 5). Vertex 4 emits nothing.
- A batch of N = 3 in layout B is: STMASK 0x3F3F3F3F; UNPACK V3-32 ×3 → address 1; masked S-8 ×3 → address 1; UNPACK V4-8 (unsigned) ×3 → address 4; UNPACK V2-32 ×3 → address 7.
  - As u32 codes: 0x20000000 followed by the mask word 0x3F3F3F3F; then 0x68038001, 0x72038001, 0x6E03C004, 0x64038007, each followed by its data.
  - (Command 0x68 = V3-32, 0x72 = masked S-8, 0x6E = V4-8, 0x64 = V2-32; immediate bit 15 = FLG and bit 14 = USN.)
- Packet header: u16 at 0 = 0x801A (EOP, NLOOP 26). u16 at 2 = 0x0105 (packet is 261 quadwords = 4,176 bytes). u32 at 4 = 0x312E441A (slot size N = 26 in byte 4; record kind 4 in the low nibble of byte 5; PRE = 1, PRIM = 0x25C, NREG = 3). REGS u64 at 8 = 0x0000_0101_0000_0512 (ST, RGBAQ, XYZ2). The bytes 0x0C = 1 (data offset) and 0x0D = 1 (one tag per batch) sit in the REGS bits the GS ignores.
- Disc packet header (resource 544, the leaf at payload offset 0x180, packet at 0x210): NLOOP 52 with EOP, quadword count 252, slot size 52, byte 5 = 0x44 (kind 4, PRE), PRIM 0x25C, data offset 1, one tag per batch.
- Two-pass batch, N = 46: first tag byte 5 = 0x54 (kind 4, "more passes", PRE), data offset 2; positions at 2, colours at 48 (layout B), UVs at 94; pass-0 UVs are entries 0–45, pass-1 UVs entries 46–91.
- Node bounds: a leaf whose vertices span x −1…3, y 0…2, z −2…0 has min (−1, 0, −2) and size (4, 2, 2). R is the largest distance from the stored sphere centre to any vertex, and +0x3C = R².
- Placement: matrix columns X = (0, 0, −1), Y = (0, 1, 0), Z = (1, 0, 0), translation (10, 0, 5) (a yaw of 90°) maps local (1, 0, 0) to world (10, 0, 4).
- Colour: lit programs (1, 2, 6) byte 16 → 1.0, byte 24 → 1.5; unlit programs (0, 4) byte 128 → 1.0, byte 64 → 0.5.
- Unlit overflow (programs 0 and 4): byte 245 with s = 1.0 gives 245. Byte 220 with s = 1.2 gives 264, which the GS sees as 8.
- Same layout, different program: a layout-C vertex with byte 128 drawn with program 4 and s = 1 gives 128. The same byte drawn with program 1 and L = 128 gives min(255, 128 × 8) = 255.
- Skinning (synthetic, identity rotations): bone b0 translates by (0, 1, 0), bone b1 by (0, 0, 2); p = (1, 0, 0), weights (0.5, 0.5, 0). Near: P = 0.5 (1, 1, 0) + 0.5 (1, 0, 2) = (1, 0.5, 1). Far: P = (1, 1, 0). With weights (0.5, 0.25, 0) the near result is 0.5 (1, 1, 0) + 0.25 (1, 0, 2) = (0.75, 0.5, 0.5): the weights are not renormalised.
- Lit vertex (the scene default lights of SPEC-0046, identity model rotation, colour scale (1, 1, 1, 1)): normal (0, 1, 0) and colour bytes (16, 16, 16, 16) give output RGBA (148, 148, 156, 128); normal (0, −1, 0) gives RGB (50, 50, 50), the ambient alone.
- NPC part (disc, region resource 1002, the first kind-3 record, "Lady Marissa", model set 0): leaf group 0 is untextured with slot 1, and the record's colour word 1 holds bytes 255, 152, 83. With vertex colour byte 20 (1.25), lit colour L = 128 per channel and s = (1.0, 0.59608, 0.32549, 1.0), the part's output is (160, 95, 52) in GS units, an orange part. With s left at (1, 1, 1, 1) the same vertex gives (160, 160, 160), grey-white.
- Unlit vertex, colour scale (1, 1, 1, 1): bytes (128, 64, 32, 128) give output (128, 64, 32, 128), so the texture is drawn × (1.0, 0.5, 0.25) with alpha 1.0.
- Handedness (synthetic): a quad with (u, v) = (0, 0) at (0, 1, 0), (1, 0) at (1, 1, 0) and (0, 1) at (0, 0, 0) has dP/du = (1, 0, 0) and dP/dv = (0, −1, 0). Its readable side faces dP/dv × dP/du = (0, 0, 1): a viewer on +Z looking toward −Z sees the text unmirrored, with u increasing to the right.

Attach points (disc; angles not listed are 0):

- Tor (resources 504 and 505, 58 bones): k0 bone 19, t (−0.184, −0.093, 0.059). k1 bone 11, t (0.184, −0.093, 0.059). k2–k4 bone 0, zero offset. k5 bone 10, t (0.300, 0.170, 0). k6 bone 33, rz −0.9312 (−53.35°), t (−0.186, 0.189, 0). k7 bone 6, t (0, 0.463, 0.306). k8 bone 33, ry π, t (0, −0.034, −0.117).
- Roberta (resources 512 and 513, 62 bones): k0 bone 19, t (−0.147, −0.085, 0). k1 bone 11, t (0.224, −0.074, 0). k5 bone 10, t (0.25, 0.15, 0). k7 bone 6, t (0, 0.366, 0.376). k2–k4, k6 and k8 are bone 0 with zero angles and a zero offset (no sheath and no back slot).
- Synthetic: with B a pure translation to bone 19's bind position (−1.558, 2.045, 0.078), Tor's T_0 is (−1.742, 1.952, 0.137). With B a pure translation to bone 33's bind position (0, 1.864, −0.208), Tor's W_6 has X axis (0.597, −0.802, 0), Y axis (0.802, 0.597, 0), Z axis (0, 0, 1) and origin (−0.186, 2.053, −0.208).
- Previous bank: after two frames with W_0 = A and then W_0 = A', the previous bank holds A.

Item model registry key: resource 549 has header u16 5 and loads into the weapon slot (base 98), so it registers as item 103 (Sword of the Ancients). Resource 561 has header u16 9 and loads into the shield slot, giving item 107 (Iron Shield). Resource 575 has header u16 36 and loads into the fixed slot (base 0), giving item 36.

## Open questions

Numbers of removed questions are not reused.

- Resolved: **Vertex colour scale**: it follows the vertex program. Lit programs 1, 2 and 6 use 16 = 1.0; unlit programs 0 and 4 use the byte times the per-draw colour scale with 128 = 1.0 (SPEC-0046, Light slots and the lit draw). **[High]**
- Resolved: **Handedness**: right-handed, +Y up, texture v = 0 the top row, no mirroring (see Coordinate conventions). **[Medium-High]**
- Resolved: **Mesh kind per path**: there is no per-packet mesh kind; the program is the pass's material program word (see "Vertex program"). **[High]**
- Resolved: **Skinned extra alpha term**: it is the specular highlight of program 6, near state only, with no header flag (SPEC-0046, "Specular alpha term of program 6"). **[High for the arithmetic]**
- **Skinning**: how the bone slots map to the skeleton's matrices (SPEC-0005). Answered: the vertex-program arithmetic is in "Skinning and lighting in the vertex program". **[High for the arithmetic]**
- **Layout-C batches with a program-2 material** (26 batches in resources 697, 698, 865, 903, 904, 948, 1216 and 1265): the load-route census rules out 697 and 698 (no SlotLoad route), 865 (its apparent owner-module loads use a slot word never assigned), and 1265 (its only loader is the unreached pack 1259). Resolved SlotLoad arguments in reached module 893 include 903 and 904; the reached pack 947 loads region 948 from its first slot. Region 1216 remains possible only through unresolved SlotLoad arguments in reached module 1184. These facts establish availability for loading, not per-frame submission: actor activation, node visibility and camera culling still apply. Even when submitted, the original appearance is unknown because program 2 reads fixed attribute slots that do not match layout C. **[Medium for load-route boundaries; Low for draw submission and appearance]**
- **Alpha bytes of lit skinned vertices**: about a third of program-2 vertices have alpha bytes of 100 or more (not 16). Under the 16 = 1.0 rule they saturate the alpha output to 255. Whether this is intended (opaque groups with an alpha test) or those bytes follow another convention is not established. **[Low]**
- Resolved: **Type-20 instance id and secondary distances.** The id at +0x40 is a zone column for the zone-visibility query, and the distances are D1 (draw), D2 (near/far switch) and an unread third value (Instanced meshes; SPEC-0046, Per-frame draw order). **[High]**
- **Draw pages.** What a page stands for (for example a group of packets or a render stage), how many pages exist, and whether the 0x30-byte records are the packet buffers. The owner of the page setup is not confirmed.
- Role ids of mesh-container children: character extra parts are 10–20, and world ids are global common-object registry ids (Mesh container; 401 is the special path). Open here: the type-5 model sets and type-11 headers inside placed objects, the 0x70-byte part records, and the second table of type 29: later specs. The type-3 node selector is a zone column (Drawing a node tree). The type-4 layer flag values and their story-layer rule are in SPEC-0046 (Culling).
- **Multi-packet leaves**: the contents of the four small tables. Answered: the per-parity packet choice of the 13 water leaves of resource 9 is in Edge cases. **[High for the packet choice]**
- Resolved: the mesh-leaf floats at 0x70–0x8F are read by nothing (Mesh leaf table). The type-4 distance fields that drawing does not use stay unread. **[High]**
- **Vertex pass size** (Medium-Low, see Drawing a node tree step 6): whether the 128-position pass is a real batch limit or only a scratch limit, which caller and mesh layout feed it, and how a batch above 128 positions would be handled. Not established.
- Resolved: **Object-level distance cull**: the 350-unit submit cull (SPEC-0046) applies to the character render objects of the frame's object lists; level geometry uses the node culls of Drawing a node tree. **[High]**
- **Resolved: type-8 child-payload fix-up.** The world type-8 handler does not relocate the container header or zero pad. It applies pointer relocation to ordinary child node-tree payloads, with each child payload as its own relocation base and the current shader list supplied for material handling. The special role-401 path remains separate. Whether a type-8 chunk can come before its shader list in a chain is not established here (SPEC-0006 open questions); the role of every relocated field is specified under Mesh container and Node tree.
- **Resolved: type-8 child dispatch source.** The values 0, 100, 200 and 400 are the child role IDs in header bytes 14–15 (+0x0E), not chunk types or payload codes. The values select the actions described under Child role dispatch. Still open: the meaning of the count read for role 0, and the downstream consumers and semantics of the stored pointers/material selection for roles 100, 200 and 400.
- **Types 2000 and 2001**: their layouts are in SPEC-0024 ("Book records"). Which pointers their fixup step changes is not checked in this spec.
- **Loader of the top-level type-13 chunks**: Resolved: resource 7 loads at boot with the item data, and resources 544–575 load through the item display slots, which register them as in "Item model registry key" (SPEC-0002, SPEC-0021). **[High]**
- Resolved: **Load status of computed loads.** Every computed region-slot load has a bounded set of region indices, so no region depends on an unknown run-time argument; a region named only in a slot-buffer reservation is not loaded (see "Load status of mesh resources").
- Resolved: **Load status recount.** The counts of "Load status of mesh resources" now use SPEC-0006's 174 reached packs: 174 world packs reached, 2 reachable only from the unreached pair 683 and 1003, 15 with no route; 436 character regions reached, 59 only from unreached packs, 60 never loaded. SPEC-0021 still counts on its own earlier basis. **[Medium]**
- **Packet pointer relocation** (Medium): which record type and loader call use this fixup, and which word is the flag. Answered: the packet-kind-1 relocation is in "Packet pointer relocation" and Edge cases, Multi-packet leaves.
- **Standalone type-13 chunks**: seventeen LOD child links point to a node that is not a group node; their target type is not examined. Answered: where they are drawn is in SPEC-0046 (Held items and character attachments) and SPEC-0021, with resource 7's shader list. **[High]**
- Resolved: **Attach point 7**. It is the status display anchor: every status effect start, the per-tick follow of the shown status effect and the immunity-block effects read its origin T_7. **[High]**
- Resolved: **Type-33 bank index**. The byte is one bone index across the render object's three bone-matrix arrays; the two thresholds are the counts held at the start of its first and second skeleton records (see Bank index). Its use in the shadow behaviour is in SPEC-0046. **[High for the rule; Medium for the roles of the arrays]**
- **Type-33 edge lists**: whether each unit's edge list is complete (unit 0 of resource 504 has 14 edges; see SPEC-0046 open questions).
