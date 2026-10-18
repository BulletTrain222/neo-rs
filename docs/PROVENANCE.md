# Source provenance

Created 2026-10-18 from the user's local source downloads:

- third_party/ps2-float: ../ps2-float-main, package 0.3.0, https://github.com/FIZL0/ps2-float (MIT OR Apache-2.0).

These were archive directories without Git revision metadata. Package versions do not uniquely identify their full source. Upstream license files are retained. Build outputs, Python caches, local configs, private context, Git metadata, and editor-agent hooks are not part of the snapshots.

ps2-float's source is unmodified. The original parent directories remain intact. Upstream README dependency examples may name older versions than the manifests.

New workspace code and notes were authored for this project. No copied retail binary, extracted model/texture, decompiler-generated function body, BIOS, save state, or capture is included. Synthetic test images are assembled from test fixtures during tests and deleted afterwards.

OpenTennis is a methodological reference, not a dependency or source-code import: https://github.com/FIZL0/opentennis . Its Rust/Bevy implementation informed the separation of data, simulation, presentation, and research tools.

## Clean-room split (2026-10-18)

The ps2-oracle snapshot, the decomp kit, the neopets-oracle crate, the kit wrapper and tool notes, and all research notes (including the six copied Python decoders) were moved to `../neopets-research/`. This workspace now contains no research-derived material. Its pre-split scaffold was reviewed and contains only generic ISO9660/mod code; see docs/CLEAN_ROOM.md. From this date, game-specific knowledge enters only through `specs/`.
