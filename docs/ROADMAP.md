# Roadmap

Every milestone after M1 is built only from approved specs in `specs/` (see [CLEAN_ROOM.md](CLEAN_ROOM.md)). If a milestone is blocked on a spec, file or refine the request in `specs/REQUESTS.md` and work on whatever is unblocked: tooling, tests, viewer infrastructure, robustness.

| # | Milestone | Needs spec | Done when |
| --- | --- | --- | --- |
| M0 | Foundation | — | ISO reader, mod resolver, sim boundary, Bevy status window. **Done.** |
| M1 | Clean-room split | — | Research tooling moved out, spec channel and audits in place. **Done 2026-10-18.** |
| M2 | Archive access | R-0001, R-0002 | `neopets-data` reads the resource index from the ISO at runtime, checks the disc fingerprint, streams any resource by number with bounds checks, and identifies its type. The inspector's `--coverage` flag reports resources by type, decoded vs total (docs/PROGRESS.md). Synthetic tests for malformed indexes. **Done 2026-10-25** (SPEC-0001, SPEC-0002). |
| M3 | Texture viewer | R-0003 | One real texture decoded from the ISO and shown in a Bevy viewer (resource browser: pick a number, see it). **Done 2026-10-25** (SPEC-0003: decoding and material texture lookup). |
| M4 | Model viewer | R-0004 | One real static model with its textures shown in Bevy with an orbit camera. **Done 2026-10-25** (SPEC-0004: node trees, packets, layouts A/B/C, strips, material binding). |
| M5 | Animated character | R-0005 | Player model skinned and playing an animation. |
| M6 | Area viewer | R-0006 | One area's geometry rendered with a free-fly camera. |
| M7 | Player slice | movement/camera/collision behavior specs | Player walks around one area with the original's movement and camera feel. `neopets-sim` owns the logic, fixed tick, input in, state out. |
| M8 | One playable area | interaction, combat, inventory, save specs | A verified playable slice. |
| M9 | Expansion | per system | World by world, system by system, each with a spec and verification report. |
| M10 | Mod UX | — | Named resource overrides, versioned manifests, load-order UI. |
| M11 | Platforms | — | Windows first; other targets investigated separately. |

## Rules for every increment

- Placeholder or diagnostic geometry is labeled as such on screen. Never present it as decoded game content.
- Committed tests use synthetic bytes built in the test, never disc-derived fixtures.
- Record in `docs/STATUS.md`: what spec it implements, what was checked against the real ISO, what is unverified.
- Passing scaffold tests is not a game-correctness milestone. Behavioral correctness comes from research-side `VER-` reports.
