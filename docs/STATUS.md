# Project status — 2026-10-18

## Implemented

- Rust workspace with neopets-data, neopets-sim, neopets-app, and a local Git repository (no remote).
- Read-only ISO9660 indexing, serial/version and executable-header checks, bounded file/range reads.
- Explicit ordered mod-file resolution with disc fallback and path containment checks.
- Engine-independent simulation boundary. Only a scaffold tick counter; no original gameplay.
- Bevy 0.19.1 status window with 3D/UI features enabled for later viewers.
- **Range-read hardening (groundwork, no spec):** `Disc::stream_range` feeds any file range to a callback in bounded chunks with one reused buffer; whole-tree caps on indexing (65,536 files, 64 MiB of directory data); I/O errors after indexing (e.g. image truncated) report offset and length. `read`/`read_range` share one bounds check.
- **Fingerprint plumbing (groundwork for R-0001):** `neopets_data::fingerprint` streams SHA-256 over any disc file (`sha2` 0.10 crate), compares against an expected digest, and parses 64-digit hex. The inspector takes `--sha256 <disc file>` (repeatable) and prints digests without comparing them. Which file to hash and the expected digest are deliberately not chosen: they must come from the R-0001 spec. SHA-256 itself is an assumption; if the spec names another hash, swap it there.
- **Fixed-timestep loop (groundwork, no spec):** `neopets_sim::FixedStep` converts host-supplied elapsed nanoseconds into whole ticks with an exact integer remainder (no drift at 60 Hz), caps catch-up per call and counts dropped ticks, and gives an interpolation alpha. `Simulation::run_for` runs the due ticks with one input. The app samples WASD/arrows/left stick into `PlayerInput` (movement only, buttons unmapped), steps the sim, then draws a diagnostic HUD. The 60 Hz rate and 8-tick catch-up cap are placeholders pending R-0007.
- **Resource-browser shell (groundwork, no spec):** the app has a resource-number picker (type digits + Enter, `[`/`]` ±1, PgUp/PgDn ±100). Every number shows "not decoded yet"; nothing is read from GAMEFILE.DAT. Labeled on screen as a shell.
- **Viewer scaffold (groundwork for M3/M4/M6, no spec):** the app now renders through a 3D orbit camera (drag any mouse button to orbit, scroll to zoom, pitch clamped short of the poles) over a diagnostic grid with X/Y/Z axes, labeled on screen as not game content. Tab hides the HUD. This is viewer tooling, not the original game's camera. Input mapping and the browser picker moved into `neopets-app/src/controls.rs`, orbit math into `orbit.rs`, both unit-tested; Check.cmd now runs `cargo test -p neopets-app` instead of only `check`.
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

None approved yet. Open requests: R-0001 to R-0007 in specs/REQUESTS.md. M2 (archive access) is blocked on R-0001 and R-0002.

## Next task

START_HERE.md step 4 groundwork is done. Next: have the user review the window (Run.cmd) and make the first commit. Then wait for SPEC for R-0001/R-0002 and implement M2. Other unblocked options until then: a free-fly camera mode for the M6 area viewer, or a viewer hook that shows the selected resource's raw byte length once M2 lands.

## Check log

- 2026-10-18 (setup session, post-split): Check.cmd passed: 4 core tests, neopets-app compile check, source audit, clean-room audit. A planted bad spec (decompiler name + MIPS line) was rejected by the audit. Inspect: real ISO recognized, SCUS-97367 v1.00, 100 files. The research workspace (`../neopets-research`) compiles separately. Known quirk: scripts/cargo.cmd prints a harmless 'vswhere.exe is not recognized' line on its second call within one Check run.
- 2026-10-18 (implementation session, host setup): Installed rustup 1.29.1 (stable rustc/cargo 1.99.1, x86_64-pc-windows-msvc) and Git 2.55.0 via winget. VS 2019 Community C++ tools were already present. audit-source.ps1 and audit-specs.ps1 passed. `cargo test`, `cargo check -p neopets-app`, and Inspect-ISO could not run from the AI session's command sandbox: every freshly built executable (including a plain hello-world and Cargo build scripts) exits 0xc000012f there. This is an AI sandbox restriction, not a project defect; to be rerun from a normal terminal.
- 2026-10-18 (same session, follow-up): Root cause was not the sandbox. The host's System32 VC++ runtime DLLs (vcruntime140, vcruntime140_1, msvcp140) were zero-filled, so every dynamic-CRT binary failed with 0xc000012f. Reinstalled VC++ 2015–2022 x64 redistributable 14.51.36247.0 via winget (DLLs now valid; the installer still returned 1603 because a stale 14.44 "Additional Runtime" package cache is damaged, harmless for this project). Check.cmd then passed: 4 core tests, neopets-app check (Bevy 0.19.1), source audit, clean-room audit. Inspect-ISO failed: private/iso-path.txt points to a pre-move path that no longer exists; needs Set-ISO.cmd with the current ISO location.
- 2026-10-18 (same session): User reselected the ISO with Set-ISO.cmd. Inspect passed: SCUS-97367 v1.00 USA, 100 files indexed read-only, SYSTEM.CNF 57 B, SCUS_973.67 2,291,560 B, GAMEFILE.DAT 1,902,385,152 B. Baseline is green; M2 still blocked on R-0001/R-0002.
- 2026-10-18 (same session, hardening + fingerprint): Check.cmd passed: 8 neopets-data tests (4 new, synthetic: chunked streaming incl. chunk 1/2/3/1 MiB, zero-length, bad chunk sizes, out-of-range, sink error stops the stream; SHA-256 of "hello" against the public test value, verify match/mismatch, hex parsing incl. a rejected leading '+'; cyclic directory, file past image end, overlong name, multi-extent record rejected; image truncated after indexing reports an error), neopets-app check, both audits. Real ISO: `--sha256 SCUS_973.67 --sha256 GAMEFILE.DAT` streamed both files (about 40 s including build) and printed digests. Digests intentionally not recorded here; the reference value must come from the spec. Not tested: the 65,536-entry and 64 MiB directory caps (no synthetic image that large).
- 2026-10-18 (same session, loop + browser shell): Check.cmd passed: 8 neopets-data tests, 5 new neopets-sim tests (60 Hz exact over uneven frames, carry/alpha, catch-up cap and dropped count incl. u64::MAX elapsed, zero rate rejected, run_for tick count), neopets-app check, both audits. `neopets-app --smoke-test` exited 0 after five frames (sim at tick 18, catch-up path exercised). Not done: manual visual review of the window and HUD, keyboard/gamepad input in a live window, and app-side unit tests (neopets-app is not a default member, so Check.cmd only compile-checks it).
- 2026-10-18 (same session, viewer scaffold + app tests): Check.cmd passed: 8 data + 5 sim + 7 new app tests (key axes, strongest stick wins and clamps, browser typing/commit/cancel/overflow/non-digit, saturating steps, orbit distance invariant, pitch limit with finite rotation, yaw wrap and zoom clamp), both audits. `--smoke-test` exited 0 with the 3D scene (tick 20 after five frames). The first `cargo test -p neopets-app` builds Bevy at the test profile's opt-level 1: about 8.5 minutes cold, cached afterwards. Not verified: how the window looks, mouse orbit and Tab in a live window, and gamepad input.
