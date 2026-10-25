# Neopets research room ("dirty room")

This folder is the **research side** of a clean-room rewrite of Neopets: The Darkest Faerie (USA 1.00, SCUS-97367). The implementation side lives in `../neopets-rust/`. Read START_HERE.md, then `../neopets-rust/docs/CLEAN_ROOM.md`.

## What you may do here

- Read and analyze the original game: the boot executable, level code, raw disc bytes, decompiler/disassembler output, anything in this folder.
- Install and run research tools (Java, Ghidra, a PS2 Emotion Engine processor extension, Python, emulator-free disc readers). Use official sources and record versions in `notes/tools.md`.
- Copy files out of the user's ISO into `work/` for analysis. The ISO path is in `../neopets-rust/private/iso-path.txt`; read it, never modify it.
- Use up to 10 Haiku subagents at a time when the user allows it. Give each a narrow brief; they report findings with evidence to you and never write specs.

## What you must never do

- Write, edit, or delete anything in `../neopets-rust/`, except reading `specs/REQUESTS.md`, `specs/TEMPLATE.md`, `docs/CLEAN_ROOM.md`, and `private/iso-path.txt`.
- Obtain, read, copy, or configure a PS2 BIOS.
- Put code, decompiler output, pseudo-code, decompiler names (FUN_/DAT_/LAB_/undefined*), code addresses, register-level descriptions, hex dumps, copied tables, or research-side code into a spec.
- Publish, push, or share anything without the user asking.

## Output

- Answer requests from `../neopets-rust/specs/REQUESTS.md` as `specs-outbox/SPEC-NNNN-<topic>.md`, following `../neopets-rust/specs/TEMPLATE.md`.
- A spec carries facts needed to interoperate with the disc and reproduce behavior: layouts, offsets, sizes, endianness, valid ranges, where data lives on the disc, behavior in plain language or math, small test vectors, open questions, and confidence levels.
- Evidence (function lists, addresses, decompiled snippets) goes in `notes/`, never in a spec.
- Black-box verification of the clean build goes in `specs-outbox/VER-NNNN-<topic>.md`: pass/fail and failing inputs only.
- The user reviews every spec and copies it into the clean side. Stop and report when a batch is ready.

## Layout

- `specs-outbox/`: finished specs for the user to review.
- `notes/`: private evidence and working notes, including `tools.md`.
- `work/`: files extracted from the ISO, Ghidra projects, scratch output. Never commit or share.
