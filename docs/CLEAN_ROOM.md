# Clean-room process

This project is a clean-room reimplementation of Neopets: The Darkest Faerie (USA 1.00, SCUS-97367) in Rust/Bevy. Everything it plays comes from the user's own ISO at runtime.

Clean room means the people (or AI sessions) who **write the game code** never look at the original program's code. A separate research side studies the original and writes plain-language specifications. Only those specifications cross over.

This is an engineering process for keeping the new code independently authored. It is not legal advice and does not on its own guarantee any legal outcome.

## Two rooms, two folders

| | Research room ("dirty") | Implementation room ("clean") |
| --- | --- | --- |
| Folder | `../neopets-research/` | this folder, `neopets-rust/` |
| May read | Original executable, Ghidra/decompiler output, the oracle, prior recompilation, old Python decoders, raw disc bytes, this clean folder | `specs/`, this folder's own source and docs, public Rust/Bevy/PS2-hardware documentation, output of this project's own tools run against the ISO |
| Writes | Specs in `../neopets-research/specs-outbox/`, private notes | Rust code, tests, docs in this folder |
| Must never | Write implementation code for neopets-rust; paste code, decompiler output, or game data into specs | Open anything in `../neopets-research/`, the parent recompilation, decompiler output, or disassemble the boot executable/level code |

**One AI session = one room, for its whole life.** A session that has read anything from the research side must never write code here. Start a new session for each room. If a clean session is accidentally exposed, stop, note it in `docs/STATUS.md`, and have a different session redo any affected work.

If the research folder is empty, `docs/research-kit/` holds starter AGENTS.md and START_HERE.md files (process only, no knowledge of the original) for the user to copy there.

## What may be in a spec

Allowed (facts needed to interoperate with the user's disc and reproduce behavior):

- Data layouts: field offsets, sizes, types, endianness, flags, valid ranges.
- Where data lives in the user's files (ISO file name, archive index, ELF virtual address of a data table) so clean code can **read it from the disc at runtime**.
- Behavior in plain language and math (e.g. "speed approaches target by 1/8 of the difference each tick, clamped to …").
- Individual constants needed for behavior. Large tables are never copied; the spec says where to read them.
- Small test vectors (input → expected output), and black-box verification results against clean builds.
- Open questions and confidence levels.

Forbidden:

- Decompiled, disassembled, or pseudo-code that mirrors the original's structure line by line.
- Decompiler names (`FUN_`, `DAT_`, `LAB_`, `undefined4`, …), code addresses, register-level descriptions.
- Copied comments, raw hex dumps, extracted assets, or game data tables.
- Python/Rust code from the research side, including `notes/reference/model-export`.

`scripts/audit-specs.ps1` (run by `Check.cmd`) flags the common forbidden patterns. It is a tripwire, not a substitute for review.

## Hand-off flow

1. Clean side needs something → adds an entry to `specs/REQUESTS.md` (questions about formats or behavior only).
2. Research side answers by writing `SPEC-NNNN-<topic>.md` from `specs/TEMPLATE.md` into `../neopets-research/specs-outbox/`. Evidence (addresses, function lists) goes in the research side's own notes, **not** in the spec.
3. **The user reviews** the spec and copies it into `specs/`, then runs `Check.cmd` (spec audit). Only then is it `Approved`.
4. Clean side implements from the spec, adds synthetic tests, and runs against the ISO.
5. Research side may black-box verify the clean build (run it, compare results with the original via the oracle) and report as `VER-NNNN-<topic>.md` through the same flow: pass/fail and failing inputs only.

## ISO-only runtime

- The app reads the user's ISO directly via `neopets-data` with bounded range reads. No extraction step, no extracted-folder mode, no bundled game data.
- Any future cache under `private/` must be regenerable from the ISO and optional.
- Mods are optional user-selected overrides layered on top of a validated disc; they never replace disc validation.

## Existing material

The first scaffold (ISO9660 reader, mod resolver, sim boundary, Bevy status window) was written by an AI session that also had research notes open. On 2026-10-18 it was reviewed: it contains only generic ISO9660/mod logic and a serial check, with no recovered game behavior or tables. It is kept. Research material (oracle crate, ps2-oracle, decomp kit, old Python decoders, `NEOPETS.md` notes) was moved to `../neopets-research/`. The session that did that split also read the research notes, so it wrote process docs only, no implementation code.
