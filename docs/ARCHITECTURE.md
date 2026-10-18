# Architecture

ISO -> neopets-data -> decoded assets / explicit tuning data -> neopets-sim + Bevy presentation.
Selected mod folders -> asset resolver -> decoded replacements. The original ISO is validated before mods apply.

The runtime never executes the original program. This workspace has no oracle or interpreter; those live only in the separate research workspace (see CLEAN_ROOM.md) and never become dependencies here. No BIOS support is configured.

## Data and portability

The ISO reader supports ordinary 2048-byte ISO9660 sectors, bounded recursive directory indexing, and range reads. It rejects multi-extent and interleaved files. A single read is capped at 64 MiB; large GAMEFILE.DAT blocks must be streamed by range. Current disc checks verify SYSTEM.CNF serial/version, required files, and ELF header, not a cryptographic revision fingerprint. Add the boot-program fingerprint check from the archive spec (R-0001) before using any fixed locations against arbitrary images.

Build GAMEFILE.DAT decoding from approved specs in specs/, then texture/model/animation/world loading. Read tuning tables from the user's disc at runtime. Do not paste extracted tables into Rust.

Simulation must accept explicit inputs and elapsed game ticks, without Bevy, rendering, file access, or wall-clock dependencies. Its current tick counter is scaffolding, not recovered game timing. Add replay state comparison only after real state and update ordering are specified.

Bevy handles host input, rendering, audio, UI, and asset presentation. A native Bevy renderer replaces PS2 hardware interfaces; it does not automatically reproduce VU/GS behavior or enable console builds.

## Mods

The implemented resolver takes an explicit ordered list of mod directories. Later entries win; missing files fall back to the ISO. Paths are normalized to uppercase disc-style paths. Parent traversal and resolved paths outside a mod root are rejected. This is a local file-replacement foundation, not sandboxed code execution.

Next layer: semantic resource keys (outfit/world/material IDs), a versioned manifest and an explicit load-order UI, GLB/image replacements, and eventually scripting with bounded permissions. None of those later features is currently implemented. Do not build mods around mutable guest RAM addresses.
