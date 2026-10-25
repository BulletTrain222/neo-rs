# Spec requests (clean side -> research side)

Questions only. Mark a request `Answered by SPEC-NNNN` once its spec is approved.

| ID | Needed for | Question | Status |
| --- | --- | --- | --- |
| R-0000 | Progress tracking | Standing request: keep a coverage count of the original game's code (every function in the boot program and every level code module, with its size) mapped to the specs that capture its behavior. Deliver `PROGRESS.md` in the format in docs/PROGRESS.md with each batch of specs. Aggregate numbers only. | Open (standing); first delivery in specs/PROGRESS.md |
| R-0001 | M2 archive | How does the game locate resources inside GAMEFILE.DAT? Where is the index stored (file/ELF data address), record layout, sector size, record count, and how resource numbers map to records. Include a disc-build fingerprint (e.g. hash of the boot executable) the clean side should check before trusting fixed locations. The clean side can already stream SHA-256 over any named disc file, so ideally give: the ISO file name(s) to hash and the expected SHA-256 as 64 hex digits. | Answered by SPEC-0001 |
| R-0002 | M2 archive | Once a resource's bytes are read, how is its type identified (texture, model, animation, level, audio, code module, other)? Header/magic layout and any compression/encoding wrapper. | Answered by SPEC-0002 |
| R-0003 | M3 textures | Texture resource format: header, pixel formats, palettes (CLUT) and their ordering/swizzle, dimensions, mip levels, alpha conventions, and how GS memory layout affects decoding. | Answered by SPEC-0003 |
| R-0004 | M4 models | Static model/mesh format: chunk structure, vertex attributes and their encodings/scales, index/strip conventions, material-to-texture binding, coordinate system and handedness. | Answered by SPEC-0004 |
| R-0005 | M5 skeleton | Skeleton and animation formats: bone hierarchy, bind pose, keyframe encoding and interpolation, skinning weights. | Answered by SPEC-0005 |
| R-0006 | M6 area | Level/area data: how an area's geometry, textures, objects, and spawn points are grouped and found from an area identifier. | Answered by SPEC-0006 |
| R-0007 | M7 player slice | Game-logic timing: at what rate does gameplay update (fixed ticks per second, tied to the video field/frame rate, or variable time step)? If variable, how is elapsed time measured, clamped, and passed to systems? Is input sampled once per update? Behavior in plain language only. | Answered by SPEC-0007 |
