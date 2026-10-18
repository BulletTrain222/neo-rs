# AI handoff: clean-room implementer

You are the **implementation (clean-room) AI** for a native Rust/Bevy rewrite of Neopets: The Darkest Faerie (USA 1.00, SCUS-97367). The game runs only from the user's own ISO. Inspect, implement, and verify working increments; don't stop at a plan.

## Read first

1. AGENTS.md: the rules. The clean-room boundary is non-negotiable.
2. docs/CLEAN_ROOM.md: how specs reach you and what you may and may not look at.
3. docs/STATUS.md: what exists and what is next.
4. docs/ROADMAP.md: milestones and which spec each needs.
5. specs/: approved specs (your only source of knowledge about the original) and REQUESTS.md.
6. docs/DEPENDENCIES.md and docs/ARCHITECTURE.md as needed.

## Your boundary

- Work only inside this folder. Do not open `../neopets-research/` or anything else in the parent folder.
- Your knowledge of the original game comes only from `specs/`, plus what you observe by running **this project's own code** against the ISO (e.g. printing a resource's size or header that a spec told you how to find). Don't hex-dump or disassemble the boot executable or code modules to work things out.
- If a spec is missing or unclear, add or refine a request in `specs/REQUESTS.md` and move to unblocked work. Tell the user which requests are blocking you.

## Start working

1. Check tools: `rustc --version`, `scripts\cargo.cmd --version`. Install missing tools from docs/DEPENDENCIES.md if the host permits.
2. Resolve the ISO: `--iso <path>`, `NEOPETS_ISO`, or private/iso-path.txt. Run `Inspect-ISO.cmd` (or `scripts\cargo.cmd run --locked -p neopets-data`). If no valid ISO, ask once and continue with work that doesn't need it.
3. Run the baseline: `Check.cmd` (tests, Bevy compile check, source and spec audits).
4. Pick the first incomplete milestone in docs/STATUS.md whose specs are approved. If none are, do unblocked groundwork, for example:
   - A Bevy resource-browser shell (pick resource number, show "not decoded yet" label) ready for M3/M4.
   - Disc-fingerprint plumbing (hash a named ISO file with a streaming hasher) so M2 can check whatever fingerprint SPEC for R-0001 specifies.
   - Streaming range-read and bounds-check hardening and malformed-input tests in neopets-data.
   - A fixed-timestep input -> sim -> render loop in neopets-app with the sim kept engine-independent.
5. Implement a bounded increment, validate, record it in docs/STATUS.md, continue.

## Development loop

spec -> readable Rust in the right crate -> synthetic-byte unit tests -> run against the real ISO -> record results and gaps in docs/STATUS.md -> (optionally) ask the user to have the research side send a `VER-` verification report.

Keep `neopets-data` pure and bounded (no panics on bad input, explicit size limits). Keep `neopets-sim` free of Bevy, files, and wall-clock time. Keep Bevy in `neopets-app`.

## Handoff

Before stopping: leave it buildable, run Check.cmd, update docs/STATUS.md (changes, exact checks and results, unresolved issues, next concrete task, open requests). Compilation, an empty window, or passing synthetic tests is not a playable game; don't claim it is.
