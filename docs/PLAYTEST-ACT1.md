# Playtest guide: the first Act 1 missions and Act 2

What to expect, step by step, from the new-game start through the Meridell enlistment errands. Steps are named in our own words. The quest rows are the journal's rows (SPEC-0049), and the flags are the story flags the step sets. Every step below has been run in an automated trace on this build, except where it says **not traced**. Nothing has been played by hand yet, so anything that looks, sounds or feels wrong is worth noting.

## Running it

- `Run.cmd --play --title` (or `target\release\neopets-app.exe --play --title` after a release build). Pick New Game.
- Developer menu (this build's own tool): **F1** opens it (cheats such as god mode, Neopoints, warp to another area, game speed, healing both heroes); **F2** toggles a free camera, and the menu can then move the hero to where the free camera is (useful to skip a climb that fails); **F4** pauses and **F5** steps one tick.
- To jump to a step without replaying the earlier ones, start the app with the flags of the steps before it, for example `--play --pack 909 --flag 107 --flag 108 --flag 109 --flag 110`.

## The farm (pack 909, the house 951)

1. **Opening** (row 0 starts, flag 107): Lucy's two tutorial lines appear at once.
2. **Herd the three Whinnies into the pen** (rows 0 to 2, flags 108, 109, 110): each penned Whinny sets one flag. Traced: all three penned.
3. **Talk to Mum, take the bucket** (row 3, flag 111): picking up the bucket (by the house) also completes this step, as the script says. Patricia is in the garden.
4. **Fill the bucket at the pump by the barn** (row 4, flag 112): Square at the pump. A short scene plays and the pump breaks. **Changed today:** the scene used to hold you for about 28 seconds; it is now about 3 (R-0092 asks for the exact waits). Check that it looks right.
5. **Bring the water to Mum** (row 5, flag 113).
6. **Tell Dad in the barn** (row 6, flags 114 and 115): he asks for his pliers, on top of the barn by the weather vane.
7. **Get the pliers from the barn roof** (row 7): **not traced end to end.** The intended route is to push the box under the lower ladder (the tutorial line says so), stand on it, jump to the ladder, climb to the ledge, take the upper ladder to the roof. Standing beside the box shows the push prompt, and Square grabs it (traced); then push it east and north to under the lower ladder. If you keep walking into the box for a moment without grabbing it, Tor climbs onto it instead, which is how you get on top once it is in place. A jump from the ground falls short of the ladder; from the box it should reach. The pickup on the roof itself works. **This is the step most likely to break; please try it.**
8. **Give Dad the pliers** (rows 8 and 9, flags 116 to 119): his scene plays and gives control back.
9. **Go into the house** (row 10, flag 120): you get the chest key on entering.
10. **Open the chest upstairs** (row 11): it gives the wooden sword.
11. **Talk to Dad downstairs** (row 12, flags 121 and 122): he gives the package. **Known oddity:** you end up holding two packages, because the script and the dialogue line both give one (asked as R-0093). It should not block anything.

## To Meridell (the road 691, the village 1023, the Recruitment Centre 1148)

12. **Leave the house** with the package: the door takes you out and the road (pack 691) loads.
13. **The package is stolen** (row 14, flag 127): entering the spot by the village gate (trigger 17) starts a long scene with Heermedjet and the thief. Control returns after it (about 38 seconds in the trace).
14. **Chase the thief** (flags 128 to 135, then 136): the thief runs through eight waypoints and waits for you each time; stay within about 30 units. The whole chase took about 2.5 minutes in the trace. At the end the package is returned (row 15 starts).
15. **Hand in the package** at the Recruitment Centre (row 15, flag 137): Officer Kendrick takes it and sends you to find a shield (the next mission).
16. **Get a shield** (row 19, flag 138): the only Wooden Shield you can reach is for sale in Armourer Inga's shop (pack 1131, a door off the village) for **400 Neopoints**. Buying it works in the trace, as does Kendrick accepting it. You will need to collect coins first (from enemies and breakables); if that takes too long, the developer menu's Neopoints entry tops up the wallet. (A second Wooden Shield sits in pack 1259, a cut map New Game cannot reach.)
17. **Ask Master Torak to sponsor you** in the tavern (pack 1150, flag 139): he needs his broken sword replaced.
18. **Collect the sword pieces** (flags 140 and 644): three lie in the village (items 40, 41 and 43). The hilt (item 42) is sold by the thief in the Recruitment Centre jail, Tark, for 100 Neopoints. **Fixed today:** Tark used to give only his three idle lines, so the hilt could not be bought.
19. **Have the smith reforge it** (pack 1133, Rognus, flag 141): he turns the four pieces into Torak's sword.
20. **Give Torak the sword** (flag 142): he gives the signed sponsorship paper.
21. **Hand the paper to Kendrick** (flag 143): he gives the permission form, and the farm moves to its next stage.
22. **Find Lucy** (flag 145 is set when you reach the farm at this stage): she is in Shadowglen Woods and the burrows beyond (pack 1217; entering sets 661).
23. **The Juppie fight** (pack 1217; winning sets 147): lure its head slam onto the marked spots so it gets stuck, then hit the head. Traced: the intro, stage 1 (the stun and the hits) and the start of stage 2. **Not traced:** a stage-2 or stage-3 hit. The automated hero got stuck instead of moving to the next spot, so this fight is worth a careful look: does each stage end, and does the win scene play?
24. **Next** (not traced yet): your parents sign the form (148) and you take it back to Kendrick (150).

## Act 2: Roberta in Faerieland (traced in pieces)

Act 2 starts at the end of the castle escape in pack 1174 (Act 1's last step, not traced yet). To try it directly: `Run.cmd --play --pack 1300 --flag 218 --flag 219 --flag 220`. In windowed play this start shows Tor, because the act start that switches to Roberta runs in 1174; the traces used Roberta. Each step below ran in a trace, but some traces started near the step (noted), so the walking between steps has not been tested.

1. **Fyora's Palace Gardens (1300), report to the council** (flag 222): walk up the stairs to Kekou at the top (near (0, -213)). A long scene with Kekou, Seradar and the Old Lady plays (about 24 seconds), then control returns. Traced (started at the top of the stairs).
2. **Collect the three wand parts** (flags 225, 226, 227):
   - The amplifier (item 10) lies on the ground near (40, -16). Traced.
   - The infuser: talk to Painter Howell (near (-51, -83)); he asks for faerie dust and gives you a jar (item 61). Walk into the spot by the east garden (trigger 0, near (86, -123)) and the dust replaces the jar (item 62). Traced. You get two jars, the same double give as Dad's package (asked as R-0095).
   - The handle (item 11) is up in the tree near (-108, 1561, -65), where the Old Lady's "Harris" is. Climb the tree trunk at about (-120, -50): face the trunk squarely and push into it to grab, climb to the top, move sideways round the trunk until Roberta stops on the face pointing at the branch, then press Square (Stop Climbing) to drop onto the branch, and walk to the handle. Traced. **Changed today:** sideways climbing used to slide off the panels into thin air; it now goes round the trunk and stops at the edge.
3. **Fyora's Library (1327), magic training** (flags 224, 228 to 235): enter the library through the gardens' door. Talk to Seradar (he takes the parts and gives the wand), then follow his steps: shoot once, hit the enchanted book, hold the attack to charge a shot (about half a second), fire three quick shots, then destroy the book (35 hit points). He gives you an outfit, Kekou calls you to Fyora, and you return to the gardens. Traced (started next to Seradar). You get 101 Motes of item 160 (one more than the script gives; R-0095).
4. **Fyora's Palace (1335), night** (flags 236, 92, 402, 403, 237): enter by the gardens' door near (-138, -129). Press Square at the bed (near (-212, -175)) to sleep; walk past the two spots near (-118, -184) and (-126, -207); then the call for help plays near the middle of the palace (0, 0). Traced in pieces. Seradar repeats a line if you keep talking to him.
5. **Draw off the Dark Faerie** (flag 238): through the side passage into pack 1344, reach the landing at the top of the stairs near (-38, -128) (a Minion Warrior guards the passage near (-7, -109); grunts and archers are around). Traced from the landing.
6. **The bell puzzle (1324)** (flag 239): through the palace's clocktower door. Push the three bells south onto the three plates in front of them, then push the two big blocks onto their plates. The scripts were traced with the blocks moved by a test tool (each bell plate, stage 1, both big plates and the solved scene); pushing was traced only for the first bell. **Fixed today:** finishing with a big block left Roberta frozen; control now returns (asked as R-0095).
7. **Rescue the Aisha** (flag 240): back in 1344, walk into the room near (-11, -109). Traced. **Get to the West Staircase** (241): the spot near (-386, 265) in the palace. Traced.
8. **The Endless Staircase (1299)** (flags 242, 243): the palace door near (311, 231) takes you there. **Not traced:** finding the way down. The trace could not find a walkable route from the top (y 62) to the exit door at the bottom (y 8, near (-111, -79)); it dropped to a floor at y 24 and stopped. **Please try this one.** The exit door itself works: it sets 243 and takes you to the library.
9. **Escape with Solarin** (Act 3 starts): the library's door near (-72, 124). The act start runs (Tor and Roberta both playable, Tor selected) and Brightvale (886) loads with the first Act 3 tutorial line. Traced.

Not built in Act 2: all movies (16, 18, 19, 20 to 26), the entity path walks in the gardens and palace, and Roberta's health refill during the practice. A flower bed near (-73, -125) in the gardens has no ground data behind it: walking into the crystal plants there drops you through the world (if that happens, F1 can warp you).

## Petpets (new, can be tried any time)

- The first wild petpet is in Meridell Outskirts (pack 1066), near (-763, 553). You need a petpet food (item ids 145 to 157; for example a Chokato, 146) chosen as your selected quick item (open the quick panel, pick the food and close it).
- Walk up to the petpet and face it: the Square prompt reads "Feed petpet". Square drops the food in front of you; the petpet walks over, eats it and becomes yours. It then follows you, its meter shows next to its portrait at the top left of the HUD, and it gives its bonus (this one, kind 0, adds 25 % to your hits). It leaves when its time runs out (7.5 minutes for a Chokato). Feed it again to top it up.
- Traced: the feed, the eat and the join. Not tested by hand: how it looks and feels. Wild petpets wander toward you when you are more than 10 units away; that follows the spec as written but is asked about (R-0094).
- Without a food selected, Square at the prompt plays the "can't" sound.

## Known gaps that may show in these missions

- Movies are not played; the game marks them as seen and carries on (the opening movies, for example).
- Ledge hanging and climbing up from a hang are not built (R-0080), and parts of wall climbing are left out (R-0085). The barn route uses ladders only, which are built.
- Lucy repeats her lines while you stand next to her and keep pressing Square; that is how the talk rule works, but tell me if it feels wrong.

## How to report

Write what you see in this build: where you were, what you did, and what happened (for example "pushed the box east from the barn wall and it would not move"). A screenshot (Windows: Win+Shift+S) helps. If a note compares with the original game ("in the real game X happens"), mark it as such. That is information about the original, so it goes to the research side as a question before anything is built from it.
