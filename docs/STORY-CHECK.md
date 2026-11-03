# Story check

Written by `scripts/story_check.py` from SPEC-0058's goal lists, in one continuous headless run. The hero is teleported to each goal and fights end by the developer defeat, so this tests script logic, not walking, fighting or feel. A failed goal is a lead that may be the checker's own fault: its Done flags and items are forced so the next goal can run, and a failure can follow from one before it.

The run reached the end of the list (exit 0).

| Result | Goals | Meaning |
| --- | --- | --- |
| notrun | 0 | Not reached |
| fail | 58 | The Done condition never held |
| gameover | 3 | The hero died and the world was torn down |
| manual | 42 | Kind not automated (minigames, pushes, climbs, shots) |
| nocheck | 95 | Ran, but its Done condition is not machine-readable |
| already | 23 | Done already held from earlier goals |
| pass | 174 | Done condition reached |

## Failed goals

- Act 1 goal 1 WAIT (909): New Game opening: movies 88 and 1, act start 1 (about 95 s). The hero stands at entry 0, (-1852.802, y, 1137.4 Done: 1 on, 706 on.
- Act 1 goal 19 SQUARE (951): The chest (lock chest 0, key item 15): Square inside the sphere. Done: item 15 gone, item 98 held. Needed and not met: item15.
- Act 1 goal 49 TALK (951 to 1148): Leave 951 (goal 21), walk the legs of goal 26 to town, then talk to Officer Kendrick (1148, town door 5). Done: 150 and 142 on, 3 and 8 on, item 0 gone, item 120 held.
- Act 1 goal 55 WALK (1056): Stand in the chain talk zone again. script trigger 10 Done: 632 on.
- Act 1 goal 56 TALK (1056 to 1163): 1056 door 1 (centre (140.3, 22.3, 573.4)); arrival locator 15 (-43.7, -126.3, 100.4). Then talk to the Miamice Done: 156 on.
- Act 1 goal 60 WALK (1056): Spin talk zone again. script trigger 11 Done: 633 on.
- Act 1 goal 61 TALK (1056 to 1152): 1056 door 0 (centre (140.3, 22.3, 483.6)) to 1098; then 1098 door 2 (centre (78.3, -130.2, -54.6)); arrival in Done: 162 on.
- Act 1 goal 63 SQUARE (1173 to 1171): Script trigger 0: cylinder base (28.9, -148.6, -12.1), radius 7.5, height 7.8; press Square at the prompt. Done: pack 1171 current (arrival locator 2 (-345.4, 2.1, -71.4)).
- Act 1 goal 65 WALK (1171 to 1056):  Done: 1056 current; flag 24 on (set at 1056's start from 167).
- Act 1 goal 69 WALK (1056): Mote talk zone again. script trigger 12 Done: 634 on.
- Act 1 goal 70 TALK (1098): Entity 49 (reach 1098 as goals 49 and 50 show, or by 1056 door 0). Done: 174 on.
- Act 1 goal 72 TALK (1098): Entity 49. Done: 180 and 185 on.
- Act 1 goal 77 TALK (1098): Entity 35 or 36 (the cutscene routine). Done: 194 on.
- Act 1 goal 78 TALK (1098 to 1163): 1098 door 5 (centre (-69.8, -128.6, -54.6)) to 1163; Sir Harlag, entity 15, placed at (-84.7, -126.6, -28.8). Done: 195 on.
- Act 1 goal 88 TALK (1098): Entity 39. Done: 206 on.
- Act 1 goal 95 WALK (956): Leave the top sphere (script trigger 16, centre (-702.1, 68.2, -824.9), radius 11.1) by more than 11.1 units,  Done: 212 on.
- Act 2 goal 114 TALK (1300): Painter Howell (entity 18). Done: 399 on, item 61 held.
- Act 3 goal 300 WAIT (886): Brightvale arrival after movie 30 (return world 64). The module's Init sets 249 because 248 is on and movie 30 Done: control back; 249 on.
- Act 3 goal 306 DOOR (758 to 777): Pack 758 teleport 1, centre (2085.3, 81.3, 510.2), 21.4 wide (the library road). Done: pack 777 current (hero placed at start position 8).
- Act 3 goal 317 WALK (753): Walk out of the sphere (more than 30 from its centre). Done: movie 37 plays on the next tick; 258 on.
- Act 3 goal 328 FIGHT (735): The Plague Serpent (boss instance 10, AI 73; SPEC-0016). Its only damage route is one strike on the head while Done: 263, 75, 50 and 70 on (set by the victory scene).
- Act 3 goal 332 DOOR (722 to 745): Door index 3 again. Done: pack 745 current.
- Act 3 goal 335 TALK (752): The bridge builder (the pack's one entity, entity 0). Done: 265 on (line 6).
- Act 3 goal 336 FIGHT (880): Brightvale road guards AI 21, 22 and 23. Walk to the plaza (the treasure seller stands at (1815.6, 166.2, -933 Done: 296 on.
- Act 3 goal 340 DOOR (1017 to 991): Pack 1017 door 0, centre (31.6, 13.5, -0.2), 10.7 wide (or door 1, centre (-33.3, -0.4, -0.2), 10.2 wide). Done: pack 991 current at locator 8 (door 0) or 7 (door 1).
- Act 3 goal 347 TALK (1017): Guildmaster Marl, entity 0 (entity 4 is the second placement and works too). On arrival with 298 on and 300 of Done: 299 on; item 23 (Golden Trove of Riches) held.
- Act 3 goal 359 TALK (874): Otto the Old Miner (entity 6) and Elspeth (entity 7). Enter 874 from 864 by the cell exit. Each talk is a shor Done: 291 and 290 on.
- Act 3 goal 363 FIGHT (864): The Ixi Chieftain (boss instance 8, 300 HP; SPEC-0014). Hero: either (Tor's sword or Roberta's wand); a fronta Done: death sets level state 0; 293 on.
- Act 3 goal 370 TALK (1251): The witch (the room's only talker). Done: 460 on.
- Act 3 goal 375 PICK (1247): The fang necklace, item 29 (a drop placed at the altar; the necklace holder is AI 10). Walk to the altar volum Done: item 29 held.
- Act 3 goal 383 WALK (1250 to 1244): Walk into the transition volume toward 1255 (script trigger 3: footprint centre (1237.0, 103.2, -1315.7), 96.3 Done: pack 1244 current.
- Act 3 goal 391 WAIT (1246): Death scene: movie 42 (first time), swap mode 2, entities deactivated, item 14 given, hero moved to locator 9. Done: item 14 held; swap mode 2.
- Act 3 goal 394 WALK (696 to 909): Legs 696, 1066, 691, 909 (the farm). The farm's story stage 6 plays a cutscene on entry when item 36 is absent Done: item 36 held; 279 on.
- Act 3 goal 397 WALK (956): Trigger index 11: sphere centre (-702.1, 68.2, -824.9), radius 5.6 (the top of the lift shaft). The lift does  Done: 282 on. Needed and not met: 281.
- Act 3 goal 400 WAIT (956): Movie 44 and the glade scene (Illusen). The scene takes item 14 (from the Werelupe King) away and gives item 3 Done: 67, 20, 283, 302 on; item 30 held.
- Act 3 goal 402 WALK (696): Cloud wall: walk toward the town road; the first of the registered cylinders fires movie 48: index 1 at base ( Done: movie 48 plays; 301 on; line 1.
- Act 3 goal 404 WAIT (1023): With the cloak worn, wait 5 s in town: the watcher plays the cloak scene (lines 20 or 21). Done: 303 on.
- Act 3 goal 405 TALK (1023): Tattered-cloak seller Rory the Shady Guy, entity 94 at this point of the story (entities 72 and 75 run the sam Done: 13 on (line 884); 1000 Neopoints removed.
- Act 3 goal 413 WALK (1129 to 1130): Walk to the cell exit toward 1130 (the water channels). Done: pack 1130 current.
- Act 3 goal 418 SQUARE (1130): Test button, trigger index 4: cylinder base (-583.0, -3.4, 273.4), radius 3.1. Square. Done: 675 on; the channel opens (objects 7, 9, 10, 11, 1 played, camera 4).
- Act 3 goal 419 WALK (1130 to 1080): Cross the opened channel; trigger index 0 (rectangle centre (-583.8, -24.7, 306.0), 60.8 wide, 15.2 deep) play Done: pack 1080 current.
- Act 3 goal 421 PICK (685): Item 18 (the key of the Oubliette gate and of the 1079 gate). Resolved: it is dropped by one Restless Spirit,  Done: item 18 held.
- Act 3 goal 431 WALK (1081): Trigger index 2: rectangle footprint centre (-490.9, -46.8, -9.8), 15.5 wide, 7.0 deep (locator 2 at (-490.9,  Done: 41 on; item 103 held; 313 on.
- Act 3 goal 443 WALK (1098 to 1121): Walk up the castle to the second floor; the cell exit toward 1121 (Meridell Castle second floor). Done: pack 1121 current.
- Act 3 goal 444 WALK (1121 to 1122): Cross 1121 (its two floor rectangles at (-43.6, -91.9, -67.2) and (-43.6, -107.3, -7.1) lead down) and take th Done: pack 1122 current (a walk, so no arrival locator; locator 23 (-27.0, -.
- Act 3 goal 447 WAIT (1122): Wave end: 320 set; hero moved to locator 24 (0.7, -46.7, -162.7); gate objects swap; cameras 4 and 5. Done: 320 on; control back.
- Act 4 goal 500 WAIT (1122 to 1300): Movie 54 ends and pack 1300 (Fyora's Palace Gardens, world 151) loads. Resolved: the boss end posts value 50 f Done: pack 1300 current; hero at record 1; control back. Needed and not met: 322.
- Act 4 goal 528 WALK (1371): Wave gate 3, script trigger 3: rectangle corner (-1.1, 5.8, -1.2), extents 0.8 by 2.2. Hero moved to record 24 Done: 344 on.
- Act 4 goal 529 WALK (1371 to 1350): Finish cylinder, script trigger 1: base (0.2, -15.6, -144.7), radius 21.3, length 10.9. Walk in: movie 61 play Done: 345, 100 on; back in pack 1350 at record 6 (0.0, 0.5, -338.1).
- Act 4 goal 532 SQUARE (1397): Buttons (script triggers 1 to 6, cylinders radius 1.4 on the row z -140.2, y 16.9, x 113.8, 107.9, 100.9, 94.7 Done: 699 on; the grate object removed.
- Act 4 goal 533 SQUARE (1397): Statues: turn pads at triggers 17 (440.5, -5.1, 37.0), 18 (440.5, -5.1, -37.1), 19 (589.5, -5.1, -37.1) and 20 Done: 700 on.
- Act 4 goal 534 WALK (1397): Trigger 12: box corner (377.9, -50.7, -325.0), extents 10.0 along x by 17.7 (height 10.2). Walk in: movie 63 p Done: 102 on.
- Act 4 goal 539 SQUARE (1353): Statue 2, script trigger 2: cylinder base (272.5, 1.2, -202.7), radius 7.1. Start angle 45: press 7 times. The Done: 694 on; pack 1392 current.
- Act 4 goal 556 WALK (1398): Stage 4: walk onto the arm trigger 35: box corner (-43.6, -8.5, 1079.8), extents 56.9 by 72.0 (height 88.4); t Done: 723 on (count 6); region 1399 loads.
- Act 4 goal 574 WALK (1382 to 1411): Trigger 28 (ring centre): cylinder base (-4.8, -438.2, 298.6), radius 2.8, length 7.0. Walk in: pack 1411 load Done: pack 1411 current. Needed and not met: 337.
- Act 4 goal 583 WAIT (1432 to 1300): Finale: the clock is hidden, object 1 (north barrier) is removed, swap mode 2, entities 0 and 5 to 7 are deact Done: pack 1300 current; 94, 365 on.
- Act 4 goal 584 WALK (1300): Epilogue: the activation sets 366, places the hero at landing 2 (0.05, 1537.66, -124.13) facing -z and gives c Done: movie 79 plays; 367 on. Needed and not met: 366.
- Act 4 goal 585 WALK (1300): Walk back north across the band of trigger 3 (z 87.2 to 105.8): it fades out, plays movie 83 and sets 368. Done: 368 on.

## Hero died

- Act 3 goal 326 WALK (735): Walk to the bridge pad that joins the western island to the arena ring (pad x 3731.8 to 3743.6, z -645.2 to -6 Done: intro starts (level state 8).
- Act 3 goal 330 PICK (735): Berry item 3 at (3780.6, 111.6, -614.6). Done: items 3, 4 and 5 held.
- Act 3 goal 361 WALK (864): Fight-start cylinder, trigger index 1: base (-828.5, 40.6, 2292.5), radius 19.1, height 20.1. The loader of gr Done: intro starts (stage bit 0).

## Not reached


## Not automated

- Act 1 goal 2 GAME (909): Whinny round-up: drive all three Whinnies into the pen. Pen entrance surface x -1853.8 to -1836.6, z 1121.4 to Done: 110 on (107, 108, 109, 110 turn on as 0, 1, 2, 3 are penned).
- Act 1 goal 10 PUSH (909): The farm crate (the pack's one movable block, block 0): from its start centre (-1938.887, 1075.668) to the cen Done: crate centre within 0.1 of the target.
- Act 1 goal 11 CLIMB (909): From the crate, jump with the stick toward the ladder face at x about -1934, z 1077.45 to 1079.08 (ladder base Done: hero y at least 58.5 within 6 of (-1934, 1078).
- Act 1 goal 12 CLIMB (909): Second ladder: face origin (-1914.73, 73.44, 1059.04), spans y 55.865 to 73.441, to the roof at y about 73.4. Done: hero y at least 73.
- Act 1 goal 28 GAME (1023): Chase Meerouladen: for k = 0 to 7 get within 30 units of locator 477 + k, in order: (-25.464, 29.043, 547.337) Done: 128 to 135 on in turn, then 136 on, item 32 held, movie 4 played.
- Act 1 goal 54 ATTACK (1056): Chain lesson: perform the chain attack (attack state 4) so that it hits the dummy, entity 4, while standing in Done: 155 on.
- Act 1 goal 57 GAME (1163): Catch five Miamice in the field (x -108.0 to -57.4, z 60.5 to 109.3; start points (-88.51, -125.82, 63.55), (- Done: 158 and 23 on.
- Act 1 goal 59 ATTACK (1056): Spin lesson: one spin attack (attack states 7 and 8) must hit all three dummies, entities 7, 8 and 9, inside t Done: 161 on.
- Act 1 goal 64 ATTACK (1171): Clear the five drains: for drain i break its three breakables in order (23, 24, 25; 26, 27, 28; 29, 30, 31; 32 Done: 637 to 641 on, 163 to 167 on, 33 on.
- Act 1 goal 68 ATTACK (1056): Mote tasks in the mote area (script trigger 7, slab centre (-33.8, 27.1, 327.1), 75.0 wide, 45.6 deep): a fire Done: 171, 172, 658 and 173 on.
- Act 1 goal 71 GAME (1056): Break the five nest breakables 24, 27, 30, 33 and 36 (entities 15 to 19 are the targets). Done: 179 on (175 to 178 and 625 to 629 on along the way).
- Act 1 goal 74 ATTACK (1056): Shield lesson: stand in the shield area (script trigger 9, slab centre (-29.2, 8.6, 420.5), 83.9 wide, 41.7 de Done: 189 on.
- Act 1 goal 75 ATTACK (1056): Shield talk zone; then stay for the next talk. script trigger 14 Done: 190 and 191 on. Needed and not met: 189.
- Act 1 goal 90 ATTACK (956): Knight record 0: enter the zone slab (script trigger 3) around (-703.8, -40.1, -860.0), kill guard AI 23, brea Done: 208 on.
- Act 1 goal 91 ATTACK (956): Record 1: zone sphere (script trigger 4) centre (-550.9, -3.6, -697.4), radius 77.4; guard AI 24; lock 5; cage Done: 209 on.
- Act 1 goal 92 ATTACK (956): Record 2: zone sphere (script trigger 5) centre (-547.4, 8.7, -534.5), radius 72.2; guard AI 25; lock 4; cage  Done: 210 on.
- Act 1 goal 93 ATTACK (956): Record 3: zone sphere (script trigger 6) centre (-676.2, 8.7, -413.1), radius 99.7; guard AI 26; lock 6; cage  Done: 623 on, then 211 on.
- Act 2 goal 121 USE (1327): If the wand is not shown in hand, equip item 111 from the item screen (weapon slot). Done: wand drawn.
- Act 2 goal 122 ATTACK (1327): Lesson 1: press Circle or R2 once, facing the target (entity 0, 5 health, unnamed type 102). Done: 230 on.
- Act 2 goal 123 ATTACK (1327): Lesson 2: after the scene, shoot the target so that it reacts to a hit. Done: 231 on.
- Act 2 goal 124 ATTACK (1327): Lesson 3: hold Circle or R2 for at least 31 ticks (hold about 90 ticks for a full charge, limit 80) and releas Done: 232 on.
- Act 2 goal 125 ATTACK (1327): Lesson 4: three new presses within about 30 ticks (the press counter gains 1 per press and loses 1 every 30 ti Done: 233 on; item 160 x100 held.
- Act 2 goal 126 USE (1327): Optional: equip the Fire Mote (Down opens the second quick panel; pick item 160). Done: Mote equipped.
- Act 2 goal 127 ATTACK (1327): Destroy the respawned target (entity 0). Roberta's health is refilled every tick during the lesson. Done: 234, 235 on; movie 19; pack 1300 current at locator 9 (0.0, 1538.2, -2.
- Act 2 goal 136 PUSH (1324): Stage-2 block 3 (start (51.02, 0.25, -45.04), size 9.22, step 4.61) north (+z) onto plate trigger 1 (centre (5 Done: the plate fires: block 3 is placed at locator 2 (51.01, 0, -30.46), ma.
- Act 2 goal 137 PUSH (1324): Stage-2 block 4 (start (43.46, 0.25, 15.63)): three steps north (+z, to z 29.46), then two steps east (+x, to  Done: the plate fires: block 4 placed at locator 1 (51.01, 0, 30.49), mark B.
- Act 2 goal 139 PUSH (1324): Bell block 0: push toward -z six times (step 1.652) onto plate trigger 2 (corner (-20.221, 31.162, 1.266), 5.5 Done: plate 2 pressed (object swap, camera 2).
- Act 2 goal 140 PUSH (1324): Bell block 1: four pushes toward -z onto plate trigger 3 (corner (-8.449, 31.162, 1.266)). Done: plate 3 pressed.
- Act 2 goal 141 PUSH (1324): Bell block 2: seven pushes toward -z onto plate trigger 4 (corner (2.918, 31.162, 1.266)). The bells may go in Done: all three pressed; with both stage-2 marks set the solved scene plays:.
- Act 4 goal 505 SWITCH (1428): Make Roberta the active hero (L1 and R1). Done: Roberta selected.
- Act 4 goal 508 SHOOT (1428): The small shot target, script trigger 5: sphere centre (28.56, 11.16, -341.9), radius 2.05 (33.5 from the Faer Done: 328 on.
- Act 4 goal 551 GAME (1363): The Mortog yard. Four Mortogs must be holding inside the target zone at the same time: a disc of squared radiu Done: 692 on (done) and 693 off.
- Act 4 goal 555 GAME (1398): Stage 3, the Miamice: four Miamice on field navmesh 15 at (67.45, 40.00, 835.20), (-25.01, 40.00, 907.23), (-6 Done: 722 on (cage count 4).
- Act 4 goal 557 GAME (1398): Stage 5, the Whinny pens: herd the four Whinnies (entities 9 to 12 of region 1401) into the four pens, one eac Done: 724 on, movie 67, 356 on, 106 on.
- Act 4 goal 568 STRIKE (1382): Pedestal 3 (Light, Motes 170 or 171): stand in the cylinder, radius 3.0, centred on (-4.9, ., 244.3) and strik Done: lit count +1.
- Act 4 goal 569 STRIKE (1382): Pedestal 1 (Fire, Motes 160 or 161): centre (-52.0, ., 271.5), triggers 21 and 27. Done: lit count +1.
- Act 4 goal 570 STRIKE (1382): Pedestal 2 (Fog or Wind, Motes 166 or 167): centre (-52.0, ., 325.8), triggers 20 and 26. Done: lit count +1.
- Act 4 goal 571 STRIKE (1382): Pedestal 0 (Smoke or Shadow, Motes 168 or 169): centre (-4.9, ., 353.0), triggers 19 and 25. Done: lit count +1.
- Act 4 goal 572 STRIKE (1382): Pedestal 4 (Bubble or Water, Motes 162 or 163): centre (42.2, ., 325.8), triggers 18 and 24. Done: lit count +1.
- Act 4 goal 573 STRIKE (1382): Pedestal 5 (Leaf or Rock, Motes 164 or 165): centre (42.2, ., 271.4), triggers 17 and 23. Done: lit count 6.
- Act 4 goal 580 SWITCH (1432): Automatic: after the third stun the script swaps (about 80 ticks, clock stopped): Roberta appears at Tor's spo Done: Roberta active; level state 14; the 3000-tick absorb window opens.
- Act 4 goal 581 SHOOT (1432): Roberta fires three quick shots into the risen Faerie, each within 179 ticks of the last and each at hover 15, Done: absorb count 3: she falls (about 50 ticks) and lands; level 16.

## No readable Done condition

- Act 1 goal 3 WAIT (909): Lucy's completion exchange (entity 31, lines 185 to 188, starts 1 s after 110). Done: no dialogue active, control back.
- Act 1 goal 18 WALK (951): Upstairs chest: locator 15 (-14.967, 11.971, -20.998). The chest is a Square-press sphere at (-15.0, 13.5, -21 Done: hero within 3.7 of the chest.
- Act 1 goal 21 DOOR (951 to farm): Exit rectangle (17.5, -0.8, 22.5). Done: farm current; hero at locator 2 (-1891.032, 40.960, 1172.344).
- Act 1 goal 22 WALK (909 to 691): East along the road to the road-fight sphere (script trigger 1): centre (-1622.8, 49.3, 1124.6), radius 14.4;  Done: scene starts (124 on after goal 23).
- Act 1 goal 24 FIGHT (691): Darrick (entity 0, 80 HP), Sid (entity 1, 60) and Drub (entity 2, 50). Kell (entity 3) is not hostile. Done: total damage dealt to entities 0, 1 and 2 reaches 31; end scene starts.
- Act 1 goal 27 WAIT (1023): Chase-start cutscene (the Meerca snatches the parcel). Done: control back, hero near locator 493.
- Act 1 goal 43 WALK (1217): From the entry toward the arena centre (0, 0, 0); the contact plane of script trigger 0 has its footprint cent Done: intro starts.
- Act 1 goal 44 WAIT (1217): Juppie intro (hero moved through locators 88, 89 and 90). Done: first Tentacle at locator 13 (7.312, 0, -6.795), control back.
- Act 1 goal 46 WAIT (1217): Outro (the hero ends at locator 94 (16.957, 0, 11.528)). Done: control back.
- Act 1 goal 47 DOOR (1217 to 909): Teleport plane 0: point (18.375, -3.416, 46.995), facing (-0.279, 0, -0.960), half-width 26.754. Done: farm current.
- Act 1 goal 82 WALK (800): Back to 851, then 800: the stage-start slab, script trigger 1: centre (445.6, 302.2, 2371.7), 13.7 wide, 27.5  Done: boss chain starts.
- Act 1 goal 94 SQUARE (956): Lift: walk into the lift sphere at the hero's level; the bottom sphere (script trigger 8) is centred at (-704. Done: hero at y about 69.
- Act 2 goal 102 WALK (1098): Script trigger 6: slab centre (-0.1, -107.8, 57.9), 67.0 wide along x (x -33.6 to 33.4), 7.0 deep, 18.5 tall.  Done: scene starts (218 on).
- Act 2 goal 103 WAIT (1098 to 1174): Movie 15 plays, the hero is placed at locator 11 (0.0, -128.1, 56.9), pack 1174 loads; its arrival step fades  Done: control back in 1174. Needed and not met: 218.
- Act 2 goal 104 WALK (1174): Section 0 to section 1: cross plane 2 (trigger 2): centre (0.0, -134.6, 49.6), 78.6 wide along x (x -39.3 to 3 Done: stage bit for section 1 set (no flag).
- Act 2 goal 105 WALK (1174): Section 1 to section 2: cross plane 3 (trigger 3): centre (-0.1, -132.3, -23.8), 26.9 wide along x (x -13.5 to Done: scene starts.
- Act 2 goal 106 WAIT (1174): Section 2 scene: the hero is moved to locator 3 (0.0, -125.8, -51.7), cameras 7 and 8, the hero runs to locato Done: control back at locator 4.
- Act 2 goal 107 WALK (1174): Section 2 to section 3: from locator 4 go west to plane 4 (trigger 4): centre (-59.7, -133.1, -62.0), 22.4 wid Done: scene starts.
- Act 2 goal 108 WAIT (1174): Section 3 scene: the hero is moved to locator 5 (-70.5, -124.6, -58.7) and runs to locator 2 (-75.8, -126.9, - Done: control back near locator 2.
- Act 2 goal 109 WALK (1174): North along the west strip (z rising from -39 to 69, west of the wall whose west face is x -36.6) to the end r Done: act start begins.
- Act 2 goal 112 WAIT (1300): First-arrival scene (flags 92, 93, 94 and 222 all clear): entity 9 walks to locator 42 (0.0, 1515.5, 112.3), e Done: control back.
- Act 2 goal 135 WALK (1324): Descend from the entry (y 117) to the lower floor (y 0) at x 43 to 55, z -45 to 31, where the two big blocks s Done: hero near block 4.
- Act 2 goal 138 WALK (1324): Up to the bell platform (y 31.2): bells at (-17.429, 33.1, 13.349), (-5.666, 33.1, 10.198) and (5.711, 33.1, 1 Done: hero beside the bells.
- Act 2 goal 142 DOOR (1324 to 1335): Teleport 0: centre (61.9, 114.5, -13.9), 9.5 wide; Square. Done: 1335 current at locator 3 (0.1, 0.0, 72.1).
- Act 2 goal 146 WALK (1299): Walk down the staircase toward the lower pad. Pad B (script class 501): centre (-136.9, 33.2, -86.5), 21.0 wid Done: hero at pad A level.
- Act 2 goal 147 WALK (1299): Pad touch 1: touch pad A (the hero is moved to B's side, one turn up). Done: state 1 (no flag).
- Act 2 goal 148 WALK (1299): Pad touch 2: touch pad B (moved to A's side). Done: state 2.
- Act 2 goal 149 WALK (1299): Pad touch 3: touch pad A. Done: state 3.
- Act 2 goal 150 WALK (1299): Pad touch 4: touch pad A again. Done: state 4.
- Act 2 goal 151 WALK (1299): Pad touch 5: touch pad B. Done: state 5.
- Act 2 goal 152 WALK (1299): Pad touch 6: touch pad A; at state 5 it does not move the hero, so keep walking down. Done: hero below pad A.
- Act 2 goal 154 WALK (1327): Cross the library to door 0: centre (-72.3, 8.3, 123.6), 11.8 wide (about 178 units of walking from locator 27 Done: hero in the door.
- Act 3 goal 302 WAIT (886): Movie 32 (the guards fall) ends; no pack change follows (no return setting). Done: control back in 886.
- Act 3 goal 304 DOOR (753 to 758): Pack 753 teleport 0, centre (1780.5, 80.2, -16.1), 23.6 wide. Done: movie 33 plays on the first use; then 758 loads.
- Act 3 goal 307 WALK (777): Trigger index 0: rectangle footprint centre (1.5, 35.6, -159.9), 16.3 wide, 14.6 deep. Walk in. Done: movie 34 plays; hero moved to locator 2.
- Act 3 goal 309 WALK (777): Library entry plate, trigger index 4: footprint centre (-60.5, 30.5, -85.6), about 4 by 4 (fires on contact, n Done: the entry animation plays (no new flag; 252 stays on). Needed and not met: 252.
- Act 3 goal 316 WALK (753): Event sphere, trigger index 1: centre (1804.5, 70.6, -143.9), radius 30. Walk in. Done: movie 36 plays; hero moved to locator 3; effects 15 to 48 run.
- Act 3 goal 327 WAIT (735): Serpent intro (camera 22, entity 73 moved to point 28, bridge pad removed, camera 23). Done: control back; level state 9 set.
- Act 3 goal 329 WAIT (735): Victory scene: movie 38, hero moved to locator 5 (3756.2, 112.4, -622.4), line 2. Done: control back.
- Act 3 goal 343 WAIT (989): Movie 46 (31 s), then the boss refill (instance 7). Done: control back; level states 3 and 4 set.
- Act 3 goal 351 WAIT (1217): Juppie intro B: the hero is moved to locator 1 (2.28, 24.51, -0.15), falls, camera tracks 8 and 18 to 21, the  Done: control back.
- Act 3 goal 353 WAIT (1217): Outro B: the eleven main records close one at a time, entities 4, 6 and 5 walk to locators 22, 24 and 23, line Done: control back.
- Act 3 goal 360 TALK (864): Mayor Jurgin, entity 43 (talk only, line 3). Done: no flag.
- Act 3 goal 362 WAIT (864): Intro: cameras 8, 10, 11 and 7, line 6, camera shake; level state 3 clears at the end. Done: control back.
- Act 3 goal 364 WAIT (864): End sequence: 668 on, cameras 15, 5 and 6. Done: control back.
- Act 3 goal 367 SQUARE (any): Wear the tattered cloak: item screen, Items tab, select item 126 and press Square. Either hero can wear it. An Done: cloak worn.
- Act 3 goal 373 PICK (any): Collect Neopoints (enemy drops and coins in the woods and burrows) until the wallet holds at least 1000, neede Done: wallet at least 1000 Neopoints.
- Act 3 goal 374 WALK (1247): Two floor plates open the way to the altar; each is a small rectangle on the floor. Index 0 at (-93.4, 8.0, 18 Done: arrival gate up; block on plate 2; panel 26 open.
- Act 3 goal 385 WALK (1244): Trigger 4 scene (once): rectangle footprint centre (655.7, 144.9, -2118.5), 50.3 wide, 7.0 deep: hero walks to Done: level state 0 set.
- Act 3 goal 387 WALK (1244 to 1246): Leave 1244 by the transition 1255, then into 1246 (the lair). Aim: entrance sphere, trigger index 1 of 1246: c Done: entrance sphere entered.
- Act 3 goal 388 WAIT (1246): Entrance scene: walk to locator 10, fade, hero placed at locator 11, boss refill (instance 14), movie 41 (firs Done: control back.
- Act 3 goal 389 WALK (1246): Arena sphere, trigger index 0: centre (736.3, 0.3, -2126.6), radius 35.8; locator 11 is already inside it. Done: fight cascade starts.
- Act 3 goal 409 WAIT (1129): Fall and arrival in the Ruined Caves. Init sets 35. Done: control back.
- Act 3 goal 414 SQUARE (1130): Water-channel lever 7 first: cylinder base (-619.0, -3.4, 278.7), radius 3.1. Square (pull). Done: junction 3 becomes c.
- Act 3 goal 415 SQUARE (1130): Lever 6: cylinder base (-552.1, -3.5, 284.4), radius 3.1. Square (pull). Done: junction 2 becomes b.
- Act 3 goal 416 SQUARE (1130): Lever 7 again (push back from c): same cylinder as goal 414. Square. Done: junction 3 becomes d.
- Act 3 goal 417 SQUARE (1130): Lever 5: cylinder base (-565.5, -3.4, 319.7), radius 3.1. Square (pull). Done: junction 1 becomes b.
- Act 3 goal 426 WALK (1078): Floor plates (a plate keeps its door open only while a character stands on it): index 3 at (-415.5, -4.1, -665 Done: crate 0 rests on plate 7; record 0 open.
- Act 3 goal 429 SQUARE (1079): Gate triggers, hold item 18: index 0 at footprint centre (-613.6, -45.6, 84.1) and index 1 at (-613.6, -45.6,  Done: passed the gate; no flag.
- Act 3 goal 432 FIGHT (1081): Hostile crypt guardians AI 12, 13 and 14, if the scene leaves them active. Done: AIs dead.
- Act 3 goal 435 FIGHT (1096): The Undead Draik (boss instance 6, 200 HP, entity 0; SPEC-0029 section 6.12). Three stages, with transitions a Done: death state sets level state 12; the finale sets 45 and 316 (and level.
- Act 3 goal 445 WALK (1122): Wave trigger, index 5: rectangle footprint centre (0.2, -47.9, -128.4), 7.0 wide, 17.5 deep (the middle floor) Done: wave fights start.
- Act 3 goal 446 FIGHT (1122): Wave 1: AI 11, 12, 13 (set 3). Wave 2: AI 14, 15, 13 (set 4) after a 3 s pause. Wave 3: AI 17, 18 (set 5) afte Done: all three waves dead.
- Act 3 goal 448 WALK (1122): Go up to the top floor, then east through the arena entry sheet, trigger index 6: vertical rectangle at x -10. Done: movie 52; boss refill (instance 11); hero at locator 26 (7.6, -25.3, -.
- Act 3 goal 451 PICK (1096): Arena chest spheres (optional, radius 3.69, no script): locators 11 (400.2, -1608.5, -427.7), 12 (455.1, -1610 Done: items collected.
- Act 3 goal 452 WALK (1089): Optional side route: crusher room 1089 (cell neighbour of 1081) with four buttons (cylinders radius 4.2, bases Done: n/a.
- Act 4 goal 503 SQUARE (1428): Optional save: the pack's one save point at (-11.158, 8.252, -257.921) in the corridor (ordinary Save Game pro Done: save written.
- Act 4 goal 507 WALK (1428): Stay at the point of goal 506 (the missile target is set by trigger 3). Done: missile target is record 4.
- Act 4 goal 510 SQUARE (1428): Switch 0, trigger 6: plate 4 x 4 at x -46.92, z -321.83; hero spot (-44.04, 9.51, -320.33) facing -x. Press Sq Done: switch states (second, third, third).
- Act 4 goal 511 SQUARE (1428): Switch 0 again (same plate and spot). Second press. Done: states (third, first, third).
- Act 4 goal 515 WALK (1411): Look at the plaza: 1411 records 0 (-8.0, 136.9, 48.0), 1 (80.0, 130.5, 48.0), 2 (-76.1, 136.8, 56.3), 3 (-56.0 Done: control back.
- Act 4 goal 520 SQUARE (1411 to 1363): Exit trigger 3: rectangle corner (90.6, 126.6, 41.9), extents 26.5 along (0.01, 0, 1.0) by 12.0. Stand inside  Done: pack 1363 (Altador Park) current; hero at record 1 (0.0, 31.0, 79.2).
- Act 4 goal 522 WALK (1370 to 1350): District 1350 (the Arena): walk to its arena door, teleport 2. The door is a vertical plane at z -346.4 (the h Done: hero at the arena door.
- Act 4 goal 524 WALK (1371): Wave gate 1, script trigger 2: rectangle corner (-5.1, 5.8, -1.2), extents 0.8 along x by 2.2, height 2.3. Wal Done: the gate scene runs: fade out, camera 8, the gate object plays, the he.
- Act 4 goal 526 WALK (1371): Wave gate 2, script trigger 4: rectangle corner (2.9, 5.8, -1.2), extents 0.8 by 2.2. Hero moved to record 25  Done: wave 2 loaded.
- Act 4 goal 530 WALK (1350 to 1358): Back to the hub 1370 and into district 1358 (the Money district). Its teleport 2 is a vertical door plane at x Done: hero at the Treasury door.
- Act 4 goal 537 SQUARE (1353): Statue 0, script trigger 0: cylinder base (272.5, 1.2, -231.5), radius 7.1. Square at the prompt (context 4);  Done: statue 0 at 360.
- Act 4 goal 538 SQUARE (1353): Statue 1, script trigger 1: cylinder base (301.4, 1.2, -231.5), radius 7.1. Start angle 90: press 6 times. Done: statue 1 at 360.
- Act 4 goal 540 WALK (1392): Arrival: the hero falls in at record 0 (6.3, 33.6, 10.9) with a long fall and camera track 10 (3 s) once per l Done: pack 1395 reached.
- Act 4 goal 541 WALK (1395, 1393): Pack 1395 is a connector (neighbours 1392 and 1393). Pack 1393 has five guard rooms: each room has a plate lev Done: pack 1396 reached.
- Act 4 goal 542 WALK (1396, 1394): Pack 1396 leads to pack 1394 (neighbour). Walk into the intro trigger 13: box corner (-65.6, 8.4, -1203.9), ex Done: the intro scene runs; the 50 s timer starts.
- Act 4 goal 543 SQUARE (1394): Lever S1, trigger 15: cylinder base (-236.4, -3.7, -1406.0), radius 3.1 (hero record 15 is the same point). Sq Done: lever used; checkpoint (record 1, camera 3).
- Act 4 goal 544 SQUARE (1394): Lever S3, trigger 16: cylinder base (132.7, -3.7, -1392.7), radius 3.1. Done: lever used; checkpoint (record 15, camera 4).
- Act 4 goal 545 SQUARE (1394): Lever S2, trigger 18: cylinder base (-115.8, 0.1, -1573.3), radius 3.1. Done: lever used; checkpoint (record 16, camera 5).
- Act 4 goal 546 SQUARE (1394): Lever S4, trigger 17: cylinder base (22.4, -2.4, -1541.1), radius 3.1. Done: lever used; the final word is set; checkpoint (record 8, camera 4).
- Act 4 goal 547 WALK (1394): Trigger 14: box corner (-68.0, -3.5, -1591.0), extents 14.9 along (0, 0, -1) by 21.9 (height 21.6). Walk in: t Done: exit open.
- Act 4 goal 550 WALK (1353 to 1370): Back to the hub 1370 and on to pack 1363's shrine door. Resolved route from the arrival at pack 1353 record 1  Done: hero near pack 1363 trigger 3.
- Act 4 goal 558 WALK (1398): After the movie, trigger 37 (cylinder base (0.5, -3.6, 1210.8), radius 1.1) moves the hero to record 34 (0.1,  Done: hero near teleport door 0.
- Act 4 goal 562 PICK (1411, 1363, 1350, 1358): Collect one Mote of each of the six pedestal elements before leaving for pack 1382 (none can be bought there a Done: one Mote of each element held (168 or 169, 160 or 161, 166 or 167, 170.
- Act 4 goal 564 WALK (1391): Script trigger 2, the lift scene: cylinder base (0.0, 0.1, 220.6), radius 20.0, length 0.5. Stand in it: the c Done: scene over.
- Act 4 goal 566 WALK (1382): The six floor switches, script triggers 3 to 8, each a box 0.8 by 3.2 (height 3.2): 3 corner (-154.8, -212.8,  Done: all six once-words set; the marker objects 21 to 26 have risen.
- Act 4 goal 567 WALK (1382): Trigger 2: vertical cylinder base (0.0, -220.5, -72.8), radius 90.0, length 3.0. Walk to (0.0, -220.5, -72.8)  Done: the pedestal ring reachable.
- Act 4 goal 577 WAIT (1432): Fight-start step (about 90 ticks): letterbox, fixed camera 4, fade in; both heroes' health and magic refilled; Done: control back.
- Act 4 goal 579 FIGHT (1432): Stage 1, three stuns (Tor). The Faerie takes no damage. She shoves any hero within 10 of A (no damage), and ot Done: stun count 3: level states 12 and 10.
- Act 4 goal 586 WAIT (1300): On the next play pass the credits-pending flag is set and the credits roll (front-end state 18; cross and tria Done: credits end; title screen.
