# Neopets: The Darkest Faerie — clean-room Rust/Bevy rewrite

A native clean-room reimplementation of Neopets: The Darkest Faerie (USA 1.00, SCUS-97367). It plays only from your own ISO, read directly at runtime. No game assets, executable bytes, or BIOS are included or needed.

**AI handoff:** start a new AI session in this folder and say: "Read START_HERE.md and follow it." That makes it the clean-room implementer. The research side is a separate folder with its own START_HERE.md; see docs/CLEAN_ROOM.md. Never use one session for both.

**Current state: foundation only, not a playable game.** The disc inspector validates the ISO and indexes its files without extraction. The Bevy app shows disc status. Everything game-specific waits on specs (see specs/REQUESTS.md).

## Start here on Windows

1. Rust (stable, edition 2024) and Visual Studio 2022 C++ Build Tools with a Windows SDK. See docs/DEPENDENCIES.md.
2. **Set-ISO.cmd** chooses your ISO. The path is stored in private/iso-path.txt. The ISO is never copied or modified.
3. **Inspect-ISO.cmd** checks the disc without opening a window.
4. **Run.cmd** builds and opens the Bevy window.
5. **Check.cmd** runs tests, the Bevy compile check, the source audit, and the spec audit.
6. **Progress.cmd** shows how much of the game is done: code coverage, asset coverage, and milestones. See docs/PROGRESS.md.

`scripts\cargo.cmd run --locked -p neopets-data -- --iso "D:\Games\Neopets.iso"` picks an explicit image; `NEOPETS_ISO` also overrides the saved path.

## Folder map

| Folder | Purpose |
| --- | --- |
| crates/neopets-data | Bounded ISO9660 reader, disc identity check, range reads, mod-file override foundation |
| crates/neopets-sim | Engine-independent game state and input boundary; no gameplay yet |
| crates/neopets-app | Bevy 0.19.1 app; currently a disc-status window |
| third_party/ps2-float | PS2 float arithmetic library (MIT OR Apache-2.0) |
| specs | Approved specs from the research side; the only source of knowledge about the original |
| docs | Clean-room process, architecture, roadmap, status, provenance |
| mods | Local user replacements; contents ignored by Git |
| private | Local paths and logs; ignored by Git |

## Rules in short

Clean room: implementers never look at the original code or the research workspace; they build from `specs/`. ISO-only: all content is read from your disc at runtime; nothing is extracted or bundled. Details in AGENTS.md and docs/CLEAN_ROOM.md.

Publish only reviewed source from version control, excluding private/, mods/, and target/. No license has been chosen for new project code yet.
