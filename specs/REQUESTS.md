# Spec requests (clean side -> research side)

Questions only. Mark a request `Answered by SPEC-NNNN` once its spec is approved.

| ID | Needed for | Question | Status |
| --- | --- | --- | --- |
| R-0001 | M2 archive | How does the game locate resources inside GAMEFILE.DAT? Where is the index stored (file/ELF data address), record layout, sector size, record count, and how resource numbers map to records. Include a disc-build fingerprint (e.g. hash of the boot executable) the clean side should check before trusting fixed locations. | Open |
| R-0002 | M2 archive | Once a resource's bytes are read, how is its type identified (texture, model, animation, level, audio, code module, other)? Header/magic layout and any compression/encoding wrapper. | Open |
| R-0003 | M3 textures | Texture resource format: header, pixel formats, palettes (CLUT) and their ordering/swizzle, dimensions, mip levels, alpha conventions, and how GS memory layout affects decoding. | Open |
| R-0004 | M4 models | Static model/mesh format: chunk structure, vertex attributes and their encodings/scales, index/strip conventions, material-to-texture binding, coordinate system and handedness. | Open |
| R-0005 | M5 skeleton | Skeleton and animation formats: bone hierarchy, bind pose, keyframe encoding and interpolation, skinning weights. | Open |
| R-0006 | M6 area | Level/area data: how an area's geometry, textures, objects, and spawn points are grouped and found from an area identifier. | Open |
