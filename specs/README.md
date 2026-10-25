# Specifications (the only inbound channel)

Clean-room implementers build from the specs in this folder and nothing else from the research side. Read [docs/CLEAN_ROOM.md](../docs/CLEAN_ROOM.md) first.

- `REQUESTS.md`: questions the clean side needs answered. The research side reads it.
- `TEMPLATE.md`: the format every spec uses.
- `SPEC-NNNN-<topic>.md`: approved format/behavior specs.
- `VER-NNNN-<topic>.md`: black-box verification reports on clean builds.
- `PROGRESS.md`: aggregate coverage numbers from the research side (see docs/PROGRESS.md). Numbers only.

A spec is only in this folder once the user has reviewed it and `scripts/audit-specs.ps1` passes. If a spec is wrong or incomplete, add a follow-up request to `REQUESTS.md`; do not guess, and do not go and look at the original yourself.

## Index

Status moves `Approved` -> `Implemented` (clean side: code + tests done, run against the ISO) -> `Verified` (a passing `VER-` report). The clean side updates this column; `Progress.cmd` and the research side's coverage count read it.

| ID | Topic | Status |
| --- | --- | --- |
| SPEC-0001 | archive index | Implemented |
| SPEC-0002 | resource identification | Implemented |
| SPEC-0003 | textures | Approved |
| SPEC-0004 | models | Approved |
| SPEC-0005 | skeleton animation | Approved |
| SPEC-0006 | areas | Approved |
| SPEC-0007 | game timing | Approved |
| SPEC-0008 | platform media | Approved |
| SPEC-0009 | audio clips | Approved |
| SPEC-0010 | boss cloud beast | Approved |
| SPEC-0011 | boss faerie sisters | Approved |
| SPEC-0012 | boss black knight | Approved |
| SPEC-0013 | boss gelert assassin | Approved |
| SPEC-0014 | boss ixi chief | Approved |
| SPEC-0015 | boss juppie | Approved |
| SPEC-0016 | boss plague serpent | Approved |
| SPEC-0017 | encounter spyder nest | Approved |
| SPEC-0018 | ground and navmesh | Approved |
| SPEC-0019 | boss werelupe king | Approved |
| SPEC-0020 | boss gnarfas | Approved |
| SPEC-0021 | items | Approved |
| SPEC-0022 | dialogue | Approved |
| SPEC-0023 | hud | Approved |
| SPEC-0024 | design tables | Approved |
| SPEC-0025 | save game | Approved |
| SPEC-0026 | game flow | Approved |
| SPEC-0027 | camera | Approved |
| SPEC-0028 | world objects | Approved |
| SPEC-0029 | enemies | Approved |
| SPEC-0030 | level script api | Approved |
| SPEC-0031 | sound banks | Approved |
| SPEC-0032 | level scripts pilot | Approved |
| SPEC-0033 | input locomotion | Approved |
| SPEC-0034 | effects | Approved |
| SPEC-0035 | front end | Approved |
| SPEC-0036 | hero combat | Approved |
| SPEC-0037 | level scripts a | Approved |
| SPEC-0038 | common enemies | Approved |
| SPEC-0039 | level scripts b | Approved |
| SPEC-0040 | character runtime | Approved |
| SPEC-0041 | level scripts c | Approved |
| SPEC-0042 | level scripts d | Approved |
| SPEC-0043 | level scripts e | Approved |
| SPEC-0044 | level scripts f | Approved |
| SPEC-0045 | level scripts g | Approved |
| SPEC-0046 | rendering | Approved |
| SPEC-0047 | petpets | Approved |
| SPEC-0048 | world interactables | Approved |
| SPEC-0049 | story progression | Approved |
| SPEC-0050 | shops economy | Approved |
| SPEC-0051 | map page | Approved |
| SPEC-0052 | cutscenes | Approved |
| SPEC-0053 | hero switching | Approved |
| SPEC-0054 | boss darkest faerie | Approved |
| SPEC-0055 | hero death respawn | Approved |
| SPEC-0056 | npc ai walk | Approved |
| SPEC-0057 | environmental hazards | Approved |
| SPEC-0058 | main story | Approved |
| SPEC-0059 | minigames | Approved |
| SPEC-0060 | foliage | Approved |
