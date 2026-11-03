# What is still missing

Written by `scripts/missing.py` from the specs, the code and docs/STATUS.md, with no disc. These are leads from text matching, not proof: an item may be built under other wording, and a level with no leads may still play wrongly. Nothing here says the game has been completed by hand, because no run has.

| Check | Count |
| --- | --- |
| Level modules in the specs | 163 |
| Modules with no code found | 0 |
| Modules with a spec handler that has no code arm | 14 |
| Comments in the code naming something not built | 94 |
| Specs no game code cites | 4 of 66 |
| Walkthrough steps with no status note saying they were traced | 14 of 45 |
| Automated-player goals in the spec, none run end to end | 384 (Act 1: 89, Act 2: 55, Act 3: 153, Act 4: 87) |
| Requests waiting for an answer | 20 |

## Requests waiting for an answer

- R-0100: Pack 1398 stages 4 and 5, and flyers in play (SPEC-0043 1398; SPEC-0038 Behaviors 44 and 48; SPEC-0040 Behavior 14D item 7) (no status recorded in its row)
- R-0101: Pack 1382: how the hero gets from the floor-switch level to the pedestal ring (SPEC-0041 1382 CB4 and the activate-world callback); pack 1397: how block 0 reaches trigger 0 (SPEC-0042 1397 Open) (no status recorded in its row)
- R-0102: The hero's candidate list and Roberta's aimed shots (SPEC-0040 Behavior 20; SPEC-0036 Roberta shot launch) (no status recorded in its row)
- R-0103: Conflicts and gaps after the R-0080 to R-0102 update (SPEC-0041 1324 and 1382; SPEC-0043 1398; SPEC-0039 886; SPEC-0058 step 25) (no status recorded in its row)
- R-0203: State 47's obstacle probe on a thin wall (SPEC-0033 Behavior 51; SPEC-0018 character plane test)
- R-0204: Id 106 on kind 1 and 6 ground (SPEC-0033 Behavior 4A.1, 5.3)
- R-0205: The moving platform pointer (SPEC-0040 Behavior 14A item 6b, +0x190)
- R-0206: Roberta's beams and Bubble beam (SPEC-0036 Family C; Object pairing 162)
- R-0207: Moving under a ceiling panel (SPEC-0033 State 21, sub-modes 4 and 6)
- R-0208: Damage from Roberta's Rock, Fog and Supernova bolt (SPEC-0036 Charm attack hits; Attack spawn; Aimed timed object)
- R-0209: Effects without an id (SPEC-0033 Behavior 50 plunge; SPEC-0036 Charm attack hits)
- R-0210: A winning input sequence for the Juppie, both acts (SPEC-0015; SPEC-0045 1217; SPEC-0062 Player-facing fight cues)
- R-0211: A winning input sequence for the Plague Serpent (SPEC-0016 How the fight is won; SPEC-0044 735; SPEC-0062 R-0165)
- R-0212: A winning input sequence for the Undead Draik from its fight start (SPEC-0029 Behavior 6, G11; SPEC-0041 1096)
- R-0213: The Undead Draik's lethal hit against the script's dead test (SPEC-0029 G12 item 2; SPEC-0041 1096)
- R-0214: The Ixi Chieftain's HP below zero and the arena after the win (SPEC-0014 Phases, Hero actions; SPEC-0044 864; SPEC-0055 8A)
- R-0215: The final fight's timeout, seen from a trace (SPEC-0054 What the player must do item 7; SPEC-0058 Endings)
- R-0216: Whether Queen Fyora can be hit (SPEC-0029 Behaviors 19 and 30; SPEC-0054 pack 1428)
- R-0217: Pack 874's slot thread (SPEC-0039 874 Thread 3; SPEC-0044 864)
- R-0218: Flag 706 after the opening (SPEC-0058 Act 1 goal 1; SPEC-0045 909 Update and arrival step)

## Walkthrough steps with no status note (SPEC-0058)

- Step 1, 691 and 909: First boot: the publisher and developer pages, card check, logo page (301 passes), legal p
- Step 8, 909: Tor's sister has gone missing
- Step 13, 1098, 1163: Assigned to ride out with Sir Harlag
- Step 14, 842 Cogham Village: Fight off the raiders attacking the village; speak with its leader
- Step 15, 800 Cogham Steppes: Beat the raider leader
- Step 16, 842: Bring back the leader's eye patch
- Step 17, 1098: Before the king; knighthood; Tor is left behind. Flag 205 is set by the rest prompt after 
- Step 18, 956 Illusen's Glade: Learn what befell the glade; free the four captive knights. Freeing a knight: kill its gua
- Step 19, 956 to 985: Defend Illusen. Boss instance 1 in 985 (flag 281 clear) is Gnarfas (entity 7): the script 
- Step 20, 956: Aftermath; ride to warn the king
- Step 21, 1098 to 1174: Flee the castle: the escape run where the hero first meets the Darkest Faerie. Pack 1174 i
- Step 36, 934 Farm Road, 1184, 1217: Free Tor's farm: find the missing farmers. Flag 277 is the burrow-entrance scene of pack 1
- Step 37, 842, 822, 864 Abandoned Mines, 874: Free Cogham: rescue the trapped miners. Afterwards 842 trigger 2 (58 on, 59 off) gives the
- Step 39, 956, 909, 985: Revive Illusen: fetch the fruit-picking tool from Tor's farm, use her charm, climb to the 

## Level modules with no code found


## Level modules with spec handlers and no code arm

Weak leads. Many handlers are built as fixed rules or tables with no arm of that number (984, 1432, 1344, 1140 and 1133 were read by hand and do have code), so read the section before building.

- 706 (SPEC-0039): handlers [6] of [1, 3, 4, 5, 6]
- 732 (SPEC-0039): handlers [2] of [1, 2, 3]
- 753 (SPEC-0039): handlers [4, 7] of [3, 4, 5, 6, 7, 8]
- 880 (SPEC-0039): handlers [6, 8] of [1, 2, 3, 5, 6, 7, 8, 9]
- 984 (SPEC-0039): handlers [1, 2, 3, 4, 5, 6] of [1, 2, 3, 4, 5, 6]
- 1133 (SPEC-0039): handlers [3, 4] of [3, 4]
- 1139 (SPEC-0039): handlers [6, 7] of [1, 2, 3, 4, 5, 6, 7]
- 1140 (SPEC-0039): handlers [3, 4, 5] of [3, 4, 5]
- 1148 (SPEC-0039): handlers [4, 5] of [2, 3, 4, 5]
- 1230 (SPEC-0039): handlers [4] of [2, 3, 4, 5, 6]
- 1248 (SPEC-0039): handlers [1, 2, 8, 9] of [1, 2, 3, 4, 5, 6, 7, 8, 9]
- 1257 (SPEC-0039): handlers [3, 4, 5, 6] of [1, 2, 3, 4, 5, 6, 7]
- 1344 (SPEC-0039): handlers [1, 4, 5] of [1, 2, 3, 4, 5, 6]
- 1432 (SPEC-0039): handlers [3, 4, 5, 6, 7] of [3, 4, 5, 6, 7]

## Citations of each spec (game code, status notes), fewest first

- SPEC-0065 common-enemies-build-guides-2: 0 in code, 4 in notes
- SPEC-0017 encounter-spyder-nest: 0 in code, 5 in notes
- SPEC-0061 not-needed-for-play: 0 in code, 6 in notes
- SPEC-0062 boss-encounter-atlas: 0 in code, 7 in notes
- SPEC-0063 arena-fighters: 1 in code, 5 in notes
- SPEC-0064 common-enemies-build-guides: 1 in code, 9 in notes
- SPEC-0057 environmental-hazards: 4 in code, 4 in notes
- SPEC-0066 unused-and-developer-worlds: 4 in code, 4 in notes
- SPEC-0002 resource-identification: 5 in code, 4 in notes
- SPEC-0007 game-timing: 5 in code, 5 in notes
- SPEC-0058 main-story: 5 in code, 25 in notes
- SPEC-0049 story-progression: 7 in code, 13 in notes
- SPEC-0001 archive-index: 11 in code, 8 in notes
- SPEC-0024 design-tables: 11 in code, 11 in notes
- SPEC-0052 cutscenes: 13 in code, 4 in notes
- SPEC-0013 boss-gelert-assassin: 13 in code, 8 in notes
- SPEC-0050 shops-economy: 16 in code, 2 in notes
- SPEC-0020 boss-gnarfas: 16 in code, 7 in notes
- SPEC-0015 boss-juppie: 16 in code, 12 in notes
- SPEC-0014 boss-ixi-chief: 16 in code, 14 in notes
- SPEC-0008 platform-media: 17 in code, 8 in notes
- SPEC-0012 boss-black-knight: 19 in code, 18 in notes
- SPEC-0055 hero-death-respawn: 20 in code, 10 in notes
- SPEC-0054 boss-darkest-faerie: 21 in code, 14 in notes
- SPEC-0010 boss-cloud-beast: 22 in code, 13 in notes

## Comments in the code naming something not built

- hud_view.rs:894: Mote, and nothing for "No Mote" (the name beside it is not built: its format in the
- play.rs:124: level-script runtime (not implemented), so they do nothing here.
- play.rs:3029: immediate). The borrowed-slot case (cycle word 0, the 1023 chase) is not built.
- play.rs:3607: (+0x2C). Not built: the shared model's draw and move (the chunk-29
- play.rs:3726: PlayHeroAnim and the equipment display are not reproduced.
- play.rs:4763: and the Act 3 dazed loop are not built. Nor the Darkest Faerie, whose absorb test
- play.rs:4934: the positional sound, shake and rumble are not reproduced.
- play.rs:6227: the hero within about 2.0 of an object's locator, then 0.3 s. Not built: their camera shake
- play.rs:6862: hero is not busy, the full showcase (mode 4) and its pose task begin. Not built: the
- play.rs:7642: of the world that is neither hidden nor dead. Not built: the grid entries of type 1 or 3 (the spec names no
- play.rs:9752: not built), the reward, a 1 s wait, camera 1, a 1.5 s wait, the normal camera).
- play.rs:10464: callback (a common enemy's attack mode 2) are not built.
- play.rs:10486: object's header and the double turn of a rotated object are not built.
- play.rs:10507: each with the hero radius. Not implemented: cell lists A and B as such, the
- play.rs:11261: breakable; her forced aim point (aim mode) is not built here.
- play.rs:11355: the current world. The class-9 controller is not built: the boss stays where it spawned.
- play.rs:11819: reset targets are not built; Medium); the controller steps the yaw and pitch and the
- play.rs:11977: companion-row branch is not built).
- play.rs:12724: equipped slot routine, which is not implemented: their use is refused.
- play.rs:14256: not built).
- play.rs:14369: from the grip) reaches one counts a strike, and the third removes it. Not built: any
- play.rs:14887: flash 222 at the hero; the hero's reaction state 0x36 is not built (the neck look is).
- play.rs:15504: None while it is hidden. In a pack whose object calls are not reproduced yet, an object no
- play.rs:15759: piece spawns and the timed effect are not built.
- play.rs:16078: An object a type-40 group names (the group kinds whose setup is not built) is also
- play.rs:16159: (SPEC-0034 5.4; R-0045); fire's two more instances at points 0 and 1 are not built.
- charm.rs:243: have their own spawns above. Not built: the Bubble beam 162 (R-0206).
- contact.rs:4: the preferred-face rule (Medium-Low in the spec) are not implemented.
- enemy.rs:1393: word, +0xC4, +0xC8, state 14) are not built.
- enemy.rs:1712: State 6 with no route stands (the spec's steer toward entry 0 is not built).
- grab.rs:11: 1.0 sideways, 1.0 to 4.5 up to 1.25, -0.5 to 6.0 up to 1.5). Not built: the exact-plane case
- ledge.rs:7: hand-over. Not built: the busy gate of the drop test, and the edge offer's replace rule beyond the facing test.
- ledge.rs:473: the hang (state 1). Not built: the pull-up's early run-out from frame 37.
- level.rs:367: for its trigger volumes and push boxes (the ground probe's mask gate is not built).
- level.rs:1586: H1; the scene's moves and cameras are Medium in the spec and not reproduced) and entity
- level.rs:1651: revive is not reproduced; the hero-death callback is on_hero_death.
- level.rs:1702: prompt routines of triggers 1 and 2 (Low) are not reproduced.
- level.rs:1774: not reproduced.
- level.rs:1799: to Medium) are not reproduced.
- level.rs:1966: flag 58, else set 0 (875); the further loads by 291 and 290 are not reproduced.
- level.rs:1994: entity-control routine are not reproduced.
- level.rs:2012: routine (Update) is not reproduced.
- level.rs:2034: object range are not built.
- level.rs:2060: 987); with 88 set breakable 0 disabled. The debug registrar is not built; the navmesh
- level.rs:2134: other control threads are not reproduced. The four instance-trigger
- level.rs:2177: fade in follows on the flag-45-set side) are not reproduced.
- level.rs:2250: Update's comparison tree (Low in the spec) are not reproduced.
- level.rs:2273: state 10) and thread B run from Update; the effect moves of Update are not built.
- level.rs:2320: ActivateTransitionAssets is not built); the navmesh table runs in Update.
- level.rs:2354: groups (given only by example) are not reproduced.
- level.rs:2391: (Low or not listed) are not reproduced.
- level.rs:2484: mapping), the navmesh cache and the locks' breakables are not built.
- level.rs:2538: not reproduced.
- level.rs:2651: callback (it refuses ordinal 73) and the empty breakable routine are not built.
- level.rs:2708: object 2. The animated-object offset loop of groups 0 to 5 is not built.
- level.rs:2737: swaps (their slots are not given) are not reproduced.
- level.rs:2790: group 0's points) is not built (the navmesh assignment runs in Update); the line-37
- level.rs:2952: object sound is not reproduced).
- level.rs:3028: its load flag clear) and the slot each load names are not reproduced (open in the
- level.rs:3416: not built.
- level.rs:3470: are not built.
- level.rs:3582: and movie 4 (the zone reload is not reproduced).
- level.rs:3977: with argument 2. Not built: the six entities' SetAI_Behavior 0 and 5 and their moves to random
- level.rs:4341: are not built.
- level.rs:5752: Trigger 0, the opening (once per load, before the area is active). Not built: the
- level.rs:6134: 14 and line 1. The boss-fight tweak (encounter 14, stage 1) is not reproduced.
- level.rs:6423: The reward scene: object 5 shown (its colour ramp is not built) and object 4 off.
- level.rs:8007: BlockPlayerControl and ReturnHeroAnimToIdle calls are not reproduced).
- level.rs:8725: locators 12 to 62. The sparkle effect is not built.
- level.rs:8860: +-3-degree swing, which ends where it began, is not reproduced.)
- level.rs:8955: data, not reproduced).
- level.rs:8983: sequencers, no damage) are not reproduced.
- level.rs:9067: not given) are not reproduced.
- level.rs:9397: item 36 (64 or 65; which hero gets which is not resolved) is not built.
- level.rs:9582: listed until the end); the character (de)activations are not built.
- level.rs:9655: SetHeroStartPos is not built.
- level.rs:10097: the avoidance points are not given or not built.
- level.rs:10323: data) are not reproduced.
- level.rs:11411: 1). NeedHeroAlive and the weapon sheathe sound are not reproduced.
- level.rs:13074: 77 to 79 with animation 82, and AI 5 made aggressive (the spec's quirk). Not built: the
- level.rs:13280: its load. The loss scene (Medium in the spec) is not reproduced.
- level.rs:13333: normal camera (preset 10). ReturnHeroAnimToIdle is not reproduced.
- level.rs:13664: waypoints. The Harlag cutscene's parking of entity 15 is not built (its cutscene runs as
- level.rs:13745: it also makes are not reproduced.
- level.rs:14633: (object 8 and the entity animations are not reproduced; the fade is taken as
- level.rs:15808: objects, effects and navmesh steps are not built; the hero-death callback is on_hero_death.
- movie.rs:2: The dead second counter (newly pressed buttons, threshold 3) is not built, as the spec says.
- object.rs:4: record, which only the MoveObjToLocator link forms set; they are not built).
- roberta.rs:506: rebuilt-frame quirk of a second bounce is not reproduced.
- rope.rs:9: rule for them). Not built: the reticle and the aim view object (whose marker model is absent
- serpent.rs:8: recoveries). The tail slap and the wait state are never entered and are not built.
- shadow.rs:9: Not built: the clip of the swept segment against the five culling planes (step 4: only a
- sisters.rs:6: overrides). The group (dome) attack never starts in the shipped game and is not built.
- wheel.rs:4: Not covered: the 758 and 1358 copies' triggers (their modules are not built) and the music
