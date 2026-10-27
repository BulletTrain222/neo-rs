# Playtest guide: the first Act 1 missions

What to expect, step by step, from the new-game start to handing in the package in Meridell. Steps are named in our own words. The quest rows are the journal's rows (SPEC-0049), and the flags are the story flags the step sets. Every step below has been run in an automated trace on this build, except where it says **not traced**. Nothing has been played by hand yet, so anything that looks, sounds or feels wrong is worth noting.

## Running it

- `Run.cmd --play --title` (or `target\release\neopets-app.exe --play --title` after a release build). Pick New Game.
- Developer menu (this build's own tool): **F1** opens it (cheats such as god mode, warp to another area, game speed); **F2** toggles a free camera; **F4** pauses and **F5** steps one tick.
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

## Known gaps that may show in these missions

- Movies are not played; the game marks them as seen and carries on (the opening movies, for example).
- Ledge hanging and climbing up from a hang are not built (R-0080), and parts of wall climbing are left out (R-0085). The barn route uses ladders only, which are built.
- Lucy repeats her lines while you stand next to her and keep pressing Square; that is how the talk rule works, but tell me if it feels wrong.

## How to report

Write what you see in this build: where you were, what you did, and what happened (for example "pushed the box east from the barn wall and it would not move"). A screenshot (Windows: Win+Shift+S) helps. If a note compares with the original game ("in the real game X happens"), mark it as such. That is information about the original, so it goes to the research side as a question before anything is built from it.
