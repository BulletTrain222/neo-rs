# Progress tracking

Run **Progress.cmd** for the current numbers. There are three measures, because no single percentage is honest for a clean-room rewrite:

| Measure | Question it answers | Who computes it |
| --- | --- | --- |
| **Code coverage** (headline) | How much of the original game's code has its behavior recreated here? | Research side, delivered as `specs/PROGRESS.md` |
| **Asset coverage** | How many resources on the disc can our code decode? | Clean side, by running its own tools on the ISO |
| **Spec pipeline and milestones** | Where is the work flow right now? | Clean side, from `specs/README.md` and `docs/ROADMAP.md` |

## Code coverage (like a decomp percentage)

The research side lists every function in the original boot program and every level code module, with its size in bytes. Each function gets a stage:

| Stage | Meaning |
| --- | --- |
| Unknown | Not studied yet |
| Analyzed | Research understands it, not yet in a spec |
| Specced | Its behavior is covered by an approved spec |
| Implemented | That spec's status is `Implemented` or `Verified` in specs/README.md |
| Verified | That spec's status is `Verified` |

Stages are cumulative: a Verified function also counts as Implemented, Specced, and Analyzed. Percentages are weighted by **code bytes** (big functions count more) and shown by function count too. The headline number is **Verified bytes / total bytes**.

Some functions are engine plumbing the rewrite replaces with Bevy (rendering hardware, DMA, the disc driver, memory cards). The research side marks those `Replaced` with the spec or note that covers the replacement. They count as done once that spec is Implemented. This avoids counting PS2 hardware code we will never recreate as "missing".

The function list stays on the research side. Only the totals below cross into this folder.

### Format of `specs/PROGRESS.md`

```
# Code coverage

- As of: YYYY-MM-DD
- Disc build: SCUS-97367 v1.00
- Functions listed: N (boot program: N, level modules: N in M modules)
- Coverage of list: Complete | Partial (explain)

| Stage | Functions | Code bytes | % of bytes |
| --- | --- | --- | --- |
| Total | 0 | 0 | 100.0 |
| Analyzed | 0 | 0 | 0.0 |
| Specced | 0 | 0 | 0.0 |
| Implemented | 0 | 0 | 0.0 |
| Verified | 0 | 0 | 0.0 |

## By subsystem

| Subsystem | Code bytes | % Specced | % Verified |
| --- | --- | --- | --- |
| Example: player movement | 0 | 0.0 | 0.0 |
```

No addresses, function names, or anything but counts, sizes, and subsystem labels. The spec audit checks this file like any spec.

## Asset coverage

Once M2 (archive access) lands, the clean side adds `--coverage` to the disc inspector. It prints the number of resources in GAMEFILE.DAT by type and how many of each the project can currently decode. It also writes the same numbers to `private/asset-coverage.txt` in this form, one line per type:

```
texture 0 123
model 0 45
```

That is: type name, decoded count, total count. Progress.cmd reads that file if it exists.

## Spec pipeline and milestones

Read directly from the request list, the spec index, and `**Done` markers in the roadmap. Useful for seeing what is blocked; not a measure of how much game exists.
