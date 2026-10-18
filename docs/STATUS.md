# Project status — 2026-10-18

## Implemented

- Rust workspace with neopets-data, neopets-sim, neopets-app, and a local Git repository (no commits yet, no remote).
- Read-only ISO9660 indexing, serial/version and executable-header checks, bounded file/range reads.
- Explicit ordered mod-file resolution with disc fallback and path containment checks.
- Engine-independent simulation boundary. Only a scaffold tick counter; no original gameplay.
- Bevy 0.19.1 status window with 3D/UI features enabled for later viewers.
- **Clean-room split (M1):** research tooling moved to `../neopets-research/`; `specs/` channel, request list, template, and spec audit added. See docs/CLEAN_ROOM.md.

## Verified

- Before the split: 4 core tests passed (synthetic ISO reads/ranges, malformed images, missing program, mod precedence, path traversal). Real ISO recognized as SCUS-97367 v1.00, 100 files indexed, no extraction. Full Bevy build and `--smoke-test` exited 0 (startup only, not visual review).
- After the split: see the check log below.

## Not implemented / not verified

- No archive index, texture, model, animation, area, gameplay, audio, save, or mod UI.
- Disc check is serial/version only, not a build fingerprint.
- The status window has not been manually reviewed.

## Progress tracking

Progress.cmd and docs/PROGRESS.md added: code coverage (from research-side `specs/PROGRESS.md`, request R-0000), asset coverage (arrives with M2 `--coverage`), and pipeline/milestones. Currently 2 of 12 milestones done; no code or asset coverage measurable yet.

## Specs

None approved yet. Open requests: R-0001 to R-0006 in specs/REQUESTS.md. M2 (archive access) is blocked on R-0001 and R-0002.

## Next task

Do the unblocked groundwork listed in START_HERE.md step 4 (resource-browser shell, fingerprint plumbing, range-read hardening, fixed-timestep loop) until SPEC for R-0001/R-0002 arrives, then implement M2.

## Check log

- 2026-10-18 (setup session, post-split): Check.cmd passed: 4 core tests, neopets-app compile check, source audit, clean-room audit. A planted bad spec (decompiler name + MIPS line) was rejected by the audit. Inspect: real ISO recognized, SCUS-97367 v1.00, 100 files. The research workspace (`../neopets-research`) compiles separately. Known quirk: scripts/cargo.cmd prints a harmless 'vswhere.exe is not recognized' line on its second call within one Check run.
