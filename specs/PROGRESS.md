# Code coverage

- As of: 2026-10-23
- Disc build: SCUS-97367 v1.00
- Functions listed: 14148 (boot program: 9259, level modules: 4889 in 163 distinct modules)
- Coverage of list: Partial. The boot program is fully listed. Level code modules (chunk type 37; 173 world packs, 163 distinct modules) are listed function by function (from research-side analysis); identical modules are counted once. A function is counted as Analyzed only when a spec in this batch captures its behavior; platform code the rewrite replaces is counted under SPEC-0008. "Specced" stays 0 until the user approves specs.

| Stage | Functions | Code bytes | % of bytes |
| --- | --- | --- | --- |
| Total | 14148 | 3162920 | 100.0 |
| Analyzed | 14148 | 3162920 | 100.0 |
| Specced | 0 | 0 | 0.0 |
| Implemented | 0 | 0 | 0.0 |
| Verified | 0 | 0 | 0.0 |

## By subsystem

| Subsystem | Code bytes | % Specced | % Verified |
| --- | --- | --- | --- |
| HUD (SPEC-0023) | 23664 | 0.0 | 0.0 |
| NPC AI and walk (SPEC-0056) | 18532 | 0.0 | 0.0 |
| archive (SPEC-0001) | 936 | 0.0 | 0.0 |
| areas and world packs (SPEC-0006) | 28332 | 0.0 | 0.0 |
| audio clips and music (SPEC-0009) | 13840 | 0.0 | 0.0 |
| boss: Black Knight (SPEC-0012) | 24884 | 0.0 | 0.0 |
| boss: Cloud Beast (SPEC-0010) | 16252 | 0.0 | 0.0 |
| boss: Darkest Faerie (SPEC-0054) | 10280 | 0.0 | 0.0 |
| boss: Faerie Sisters (SPEC-0011) | 16104 | 0.0 | 0.0 |
| boss: Gelert Assassin (SPEC-0013) | 24188 | 0.0 | 0.0 |
| boss: Gnarfas (SPEC-0020) | 11400 | 0.0 | 0.0 |
| boss: Ixi Chief (SPEC-0014) | 13328 | 0.0 | 0.0 |
| boss: Juppie (SPEC-0015) | 7712 | 0.0 | 0.0 |
| boss: Plague Serpent (SPEC-0016) | 7240 | 0.0 | 0.0 |
| boss: Werelupe King (SPEC-0019) | 15412 | 0.0 | 0.0 |
| camera (SPEC-0027) | 58848 | 0.0 | 0.0 |
| character runtime (SPEC-0040) | 115392 | 0.0 | 0.0 |
| chunk files and resource types (SPEC-0002) | 2140 | 0.0 | 0.0 |
| common enemies (class 17) (SPEC-0038) | 58904 | 0.0 | 0.0 |
| cutscenes (SPEC-0052) | 3424 | 0.0 | 0.0 |
| design tables (SPEC-0024) | 6964 | 0.0 | 0.0 |
| dialogue and flags (SPEC-0022) | 34544 | 0.0 | 0.0 |
| encounter: Spyder Nest (SPEC-0017) | 2544 | 0.0 | 0.0 |
| enemy framework (SPEC-0029) | 145096 | 0.0 | 0.0 |
| foliage patches (SPEC-0060) | 14700 | 0.0 | 0.0 |
| front end (SPEC-0035) | 58560 | 0.0 | 0.0 |
| game flow and movies (SPEC-0026) | 23408 | 0.0 | 0.0 |
| game timing (SPEC-0007) | 4196 | 0.0 | 0.0 |
| ground and navmesh (SPEC-0018) | 72868 | 0.0 | 0.0 |
| hero combat (SPEC-0036) | 101676 | 0.0 | 0.0 |
| hero death and respawn (SPEC-0055) | 2436 | 0.0 | 0.0 |
| hero switching (SPEC-0053) | 5528 | 0.0 | 0.0 |
| input and locomotion (SPEC-0033) | 112244 | 0.0 | 0.0 |
| items and drops (SPEC-0021) | 55560 | 0.0 | 0.0 |
| level script host API (SPEC-0030) | 66392 | 0.0 | 0.0 |
| level scripts (batch A) (SPEC-0037) | 25228 | 0.0 | 0.0 |
| level scripts (batch B) (SPEC-0039) | 71884 | 0.0 | 0.0 |
| level scripts (batch C) (SPEC-0041) | 150524 | 0.0 | 0.0 |
| level scripts (batch D) (SPEC-0042) | 209348 | 0.0 | 0.0 |
| level scripts (batch E) (SPEC-0043) | 200916 | 0.0 | 0.0 |
| level scripts (batch F) (SPEC-0044) | 205216 | 0.0 | 0.0 |
| level scripts (batch G) (SPEC-0045) | 342584 | 0.0 | 0.0 |
| level scripts (pilot) (SPEC-0032) | 7600 | 0.0 | 0.0 |
| map page (SPEC-0051) | 23788 | 0.0 | 0.0 |
| models and meshes (SPEC-0004) | 8544 | 0.0 | 0.0 |
| petpets (SPEC-0047) | 13896 | 0.0 | 0.0 |
| platform services, replaced (SPEC-0008) | 346696 | 0.0 | 0.0 |
| rendering (SPEC-0046) | 107424 | 0.0 | 0.0 |
| save game (SPEC-0025) | 22492 | 0.0 | 0.0 |
| shops and economy (SPEC-0050) | 1828 | 0.0 | 0.0 |
| skeletons and animation (SPEC-0005) | 50412 | 0.0 | 0.0 |
| sound banks (SPEC-0031) | 29484 | 0.0 | 0.0 |
| story progression (SPEC-0049) | 1696 | 0.0 | 0.0 |
| textures and materials (SPEC-0003) | 15144 | 0.0 | 0.0 |
| visual effects (SPEC-0034) | 83828 | 0.0 | 0.0 |
| world interactables (SPEC-0048) | 20588 | 0.0 | 0.0 |
| world objects and lights (SPEC-0028) | 46272 | 0.0 | 0.0 |
