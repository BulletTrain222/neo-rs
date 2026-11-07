# SPEC-0059: Minigames and side activities

- Answers: proactive (open close-out); proactive, none requested (the prize wheel, the betting race, the time trial, the find game, the delivery tasks, the walk-up greetings and fame lines, and a summary of the arena ladders and the wave trial)
- Status: Draft
- Disc build: SCUS-97367 v1.00 (same fingerprint as SPEC-0001). Check the pay-out words and cell-bit words of the three wheel copies (Where it lives) before trusting the values.
- Confidence: per section. Prize wheel: High (one rule set in three modules; the values are module data). Spin timing: High (float32 accumulation of the turn rate). Wheel odds and expected returns: High (arithmetic from the rules, assuming a uniformly random stop). Greetings: High. Fame lines: High for the rule, High that all 31 registered routines follow it (two head forms that differ only in when the first draw is taken), Medium for the line of each arena rank. Betting race: High for the rules; Medium-High for the win chances (estimates from simulating the rule). Time trial: High. Find game: High for the rules and values; Medium for the three-target time-out placement and for the reload edge case. Delivery tasks: High, including the draw defect and the package source. Pack 1150 fame lines: High. Race first pass: High. Time-trial clock across areas: High. Find-game entity pairs: Medium-High. Arena ladders and wave trial: High (summary only; the bodies belong to SPEC-0041 and SPEC-0042). Lifetime of the game words: High (module lifetime rule, SPEC-0030).
- Related specs: SPEC-0023 (section 31, the on-screen timer), SPEC-0050 (coins, wallet, fees), SPEC-0021 (coin items and the add path), SPEC-0030 (host rows 100, 212 and 240; module lifetime), SPEC-0026 (area changes), SPEC-0007 (tick rate), SPEC-0022 (world text lines), SPEC-0029 (creature AI), SPEC-0041 (packs 1176 and 1371), SPEC-0042 (pack 1350), SPEC-0043 (pack 1358), SPEC-0044 (packs 722, 1056 and 1327), SPEC-0045 (packs 758, 934 and 1023), SPEC-0058 (story steps).

## Summary

The game has six optional activities run by level scripts, plus two arena ladders and one wave trial that belong to the story. This spec owns the rules of the activities; the level specs keep the trigger and entity registrations and point here.

- **Prize wheel** (packs 1023, 758 and 1358). One rule set with three sets of data: fee 15, 25 or 100 NP. Five of eight wheel cells are lit. Each spin speeds up to a full rate, the player stops it with Square, and a lit cell under the pointer wins and goes dark. After each win the player may cash out or go on; five wins in a row pay the top prize; a loss pays nothing. Pay-outs are given as coins.
- **Betting race** (pack 722). Three racers get distinct odds from 1 to 4. A bet of 50 NP on the winner pays odds x 100 NP.
- **Time trial** (pack 758). The keeper starts a count-up clock; the finish gate freezes it; the judge spot sets one of three ladder flags for a time of at most 180, 90 or 45 s. No prize.
- **Find game** (pack 934). A petpet (or three) hides at random among eight places under a count-down clock of 300, 240 or 180 s. Catching it sets a tier flag by the time left. No prize.
- **Delivery tasks** (pack 1358). Eight "talk to the resident of house X" tasks. No prize. The task draw has a defect: the first listed task is never offered.
- **Greetings and fame lines** (pack 1023). Two walk-up greetings (arena master, wheel host) and fame lines in 31 villager talk routines (39 villagers), driven by the arena and wheel flags.

None of these games is timed or scored by the host. All game state is module data of exterior modules plus a few game flags.

## Where it lives on the disc

- Wheel pay-outs: five 32-bit words in the level-module data of each of packs 1023, 758 and 1358 (values in Layout).
- Wheel cell bits: eight 32-bit words in the same three modules, one mask bit per cell (Layout).
- Wheel game words (stage, round, angle, mask, lit cells): module data of the same modules, file value 0.
- Betting race: module data of pack 722: three racer entity words (file values 0, 1 and 2), the pick word (file value -1), six reset locator words (20 to 25) and three odds words (zero-filled at load).
- Find game: module data of pack 934: eight hiding-place locator words (3 to 10), eight navmesh words, and four colour bytes (220, 40, 40, 128) for the low-time blink.
- Boot program data: the timer frame step at virtual address 0x368FA8 (1/60, SPEC-0023) and the default timer colour at virtual address 0x2DD040 (0x80808080).
- Line, prompt, locator, object, entity and sound numbers: the world data of each pack (SPEC-0022 for lines). Lines are named by index only.

## Layout

### Prize wheel copies

| Copy | Trigger (gate) | Fee | Too-poor line | Start prompt | Go-on prompts (round r = 0 to 3) | Go-on line | Cash-out lines (r = 0 to 3) | Five-win line | Loss lines (r = 0 to 4) | Pay-outs (r = 0 to 3; five wins) | Flags (loss, cash-out, five wins) | Sounds (win, loss) | Wheel turn | Mask bit of cell i |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| 1023 | 36 (flags 64 and 11 clear) | 15 | 1042 | 1026 | 1027 to 1030 | 1032 | 1033 to 1036 | 1031 | 1037 to 1041 | 20, 40, 60, 80; 160 | 478, 479, 480 | 11, 10 | object 0 about its depth axis | 2^(i+2) |
| 758 | 3 (no gate) | 25 | 335 | 319 | 320 to 323 | 325 | 326 to 329 | 324 | 330 to 334 | 45, 65, 105, 175; 325 | 536, 537, 538 | 9, 8 | object 0 about locator 4's forward axis | 2^i |
| 1358 | 0 (flag 102 set) | 100 | 109 | 93 | 94 to 97 | 99 | 100 to 103 | 98 | 104 to 108 | 140, 180, 250, 400; 700 | none | 7, 6 | object 0 about locator 1's forward axis | 2^i |

### Game words (run-time; module data)

| Game | Word | File value | Meaning |
| --- | --- | --- | --- |
| Wheel | stage G | 0 | 0 new game, 1 last spin won, -1 last spin lost |
| Wheel | round r | 0 | number of go-on answers so far in this game (0 to 4) |
| Wheel | angle A | 0.0 | wheel angle in degrees, float32 |
| Wheel | lit L[0..7] | 0 | 1 when cell i is lit |
| Wheel | mask M | 0 | bit word of the unlit cells |
| 1023 greetings | latch 37, latch 39 | 0 | 1 after the greeting ran, until the partner trigger clears it |
| 1023 greetings | wheel-host count | 0 | number of wheel-host greetings shown in this module life |
| 1023 greetings | arena result | 0 | posted arena result (1 or -1) after an arena return |
| Race | odds[0..2] | 0 | odds of racers 0 to 2 (1 to 4), sum S |
| Race | pick | -1 | racer the hero bet on |
| Race | count[0..2] | (per race) | steps taken by each racer |
| Time trial | armed, finished | 0 | the run is started; the finish gate was passed |
| Time trial | T | 0.0 | the stored finish time in seconds |
| Time trial | hint | 1 | the hint trigger may talk |
| Find game | caught[0..2], busy | 0 | three-target caught marks; time-out in progress |

## Behavior

### 1. Common rules

Confidence: High.

- Times are seconds; the game runs 60 ticks per second (SPEC-0007). Sleep(n) waits n ticks; Wait(s) waits s seconds.
- Random(a, b) is uniform over a to b with both ends included (SPEC-0030 row 240). Every draw named here comes from that one generator, so the order of draws matters for a faithful replay.
- A "Square press" is a new press of Square on this tick (pressed now, not pressed on the previous tick). Square-button context n is the on-screen button prompt n (SPEC-0023, section 33).
- Coin items 135, 136, 137, 138 and 139 are worth 1, 5, 10, 25 and 100 NP (SPEC-0050). "Pay n NP" means: while n > 0, give the largest coin whose value is at most n (139, then 138, 137, 136, 135), subtract its value, and give it with GiveItemToHero(item, 1, no notice).
- The clock is the HUD timer of SPEC-0023 section 31 (step 1/60 per update, clamp to the limit, colour). ActivateTimer(value, limit, direction) sets it.
- **Lifetime of the game words.** All the words in Layout are data of exterior modules (1023, 758, 1358, 722, 934 and 1371 are exteriors). A word keeps its value while its pack stays in the exterior buffer: through interior visits, swaps between the exterior and a transition, and streaming steps that find the pack already resident. It returns to its file value on any driver load of the exterior buffer (an exterior replacement load, the self-reload after a hero death, the after-movie reload, loading a save), on streaming to a different exterior, and on a new game (SPEC-0030, module lifetime; SPEC-0026). Game flags are not module data: they keep their values until a new game.

### 2. Prize wheel (packs 1023, 758 and 1358)

Confidence: High. Values per copy in Layout.

**Trigger loop.** While the trigger state is non-zero, once per tick (Sleep(1) per pass): show square-button context 5. On a Square press:

1. If NP is below the fee: show the too-poor line. Nothing else.
2. Otherwise take hero control (GrabHeroControl) and run one round step (below). If the step accepts the game: music override (64, -1, -1, -1); the light show; a new pattern; then repeat Wait(1.0 s), one spin and one round step for as long as the round step says to go on.
3. In every case (declined or finished): music override (-1, -1, -1, -1), ReturnToNormalCamera and ReturnHeroAnimToIdle.

The fee check (step 1) comes before the start prompt.

**Light show.** Seven cycles of: cells 0, 2, 4 and 6 lit and cells 1, 3, 5 and 7 dark for 5 ticks, then the reverse for 5 ticks. 70 ticks in all, ending with the odd cells lit.

**New pattern.** All eight cells are lit; then three distinct cells are made dark by drawing Random(0, 7) until three different values have come up (a repeat is drawn again). Five cells stay lit. Init also runs one new pattern when the module loads.

**Mask write.** After every change to the lit cells: for each cell i, a dark cell sets its mask bit in M and a lit cell clears it; then SetWorldCollisMask(M) and SetWorldDisplayMask(M) (the display word stores 2M, SPEC-0030 row 212). The mask bit hides the cell's lit art. In 1023 the cell bits are bits 2 to 9; in 758 and 1358 they are bits 0 to 7.

**Spin.** The turn rate starts at 0 for each spin. Each tick:

- if the rate is below 9.0, add 0.1 to it (float32 arithmetic);
- otherwise show square-button context 4 and test for a Square press;
- then add the rate to A; if A is now above 360, subtract 360; turn the wheel object to A (Layout); Sleep(1).

The spin ends after the tick on which the press was seen, so that tick still advances A. In float32 the rate is 8.9999 after 90 additions, still below 9.0, so a 91st addition is made: the full rate is 9.0999985 degrees per tick (not 9.0). The full rate is reached on tick 91, by which point A has advanced 418.60 degrees; the stop prompt first shows, and a press is first accepted, on tick 92. At the full rate a 45-degree cell passes in about 4.95 ticks, so the player can aim. Then Wait(1.0 s) and the result.

**Result.** The cell under the pointer is c = 7 - floor(A / 45) for 0 <= A < 360 (cell 7 for 0 to under 45, down to cell 0 for 315 to under 360). This index addresses the same cell whose mask bit controls the art, so the cell the player sees is the cell scored. If L[c] is 1: a win; L[c] = 0, mask write, G = 1, the win sound. Otherwise: a loss; G = -1, the loss sound.

**1023 mesh crosswalk (High for the mask linkage; Medium-High for the local sector orientation).** Object 0's static type-12 model has eight wedge subtrees. Their visibility-mask values are 8, 16, 32, 64, 128, 256, 512 and 1,024, which equal twice the script mask bits for cells 0 through 7. In the model's XY plane, cell 0 occupies the lower-right sector and the cell index advances counterclockwise by one 45-degree sector; cell 7 is the lower-left sector. All eight wedge leaves use the same material group and store the same vertex-color tuple. This crosswalk is checked for pack 1023 only; it does not identify the fixed pointer's direction in this model frame.

**Round step.**

- G = 0 (new game): the start prompt. Yes: RemoveNeopoints(fee) and go on. No: end.
- G = 1 and r below 4: go-on prompt r. Yes: show the go-on line, r = r + 1, go on. No: cash-out line r, pay pay-out r, set the cash-out flag, G = 0 and r = 0, end.
- G = 1 and r = 4 (the fifth win): the five-win line, pay the five-win value, set the five-win flag, G = 0 and r = 0, end.
- G = -1: loss line r, set the loss flag, G = 0 and r = 0, end. Nothing is paid.

**Carry-over and resets.** A is never reset by the game: each spin starts where the last one stopped, across spins and across games, for as long as the module data lives (section 1). A returns to 0 when the module data resets (for 1023 this includes every return from the arena, which is a driver load). G and r reset at every game end. The pattern is redrawn at every game start, so cells darkened by wins stay dark only until the next game. Init draws a pattern at each module load. The result flags are set only; no module clears them.

**Odds and expected return** (stop uniformly random over the circle; five, four, three, two and one cells lit for spins 1 to 5, so the win chances are 5/8, 4/8, 3/8, 2/8 and 1/8). Expected NP paid for the plan "cash out after k wins", k = 1 to 5:

| Copy | Fee | k = 1 | k = 2 | k = 3 | k = 4 | k = 5 |
| --- | --- | --- | --- | --- | --- | --- |
| 1023 | 15 | 12.5 | 12.5 | 7.0 | 2.3 | 0.6 |
| 758 | 25 | 28.1 | 20.3 | 12.3 | 5.1 | 1.2 |
| 1358 | 100 | 87.5 | 56.3 | 29.3 | 11.7 | 2.6 |

Only "758, cash out after one win" returns more than its fee with a random stop. A player who aims at lit cells does better.

### 3. Pack 1023 walk-up greetings (triggers 37 to 40) and fame lines

Confidence: High for the greetings; High for the fame rule and for the list of routines that use it; Medium for the line of each arena rank.

Triggers 37 to 40 are not games. They are registered only when flags 64 and 11 are both clear.

- **Latches.** Trigger 37 or 39 (state 1, object present): when its latch is 0, the latch becomes 1 and the greeting runs: trigger 37 the arena master (entity 66), trigger 39 the wheel host (entity 65). Trigger 38 (state 1, object present) clears latch 37; trigger 40 clears latch 39. So a greeting repeats only after the hero has passed the partner trigger.
- **Arena master greeting.** Runs only while the arena-result word is 0. The arena return (SPEC-0045, 1023 activate-world routine) stores the posted result (1 or -1) in that word and also sets latch 37, so there is no arena-master greeting after an arena return until the module data resets. Line by the highest arena flag: 28 (385 set), 29 (384), 30 (383), 31 (382), 32 (381); else, with flag 380 set, lines 33 and 34 in turn; else lines 35 and 36 in turn.
- **Wheel host greeting.** Two greetings per module life. The first shows line 1016 when flag 480 is set, else 1017 when flag 384 is set, else 1018 when 479 is set, else 1019 when 478 is set, else 1020. The second shows line 1021. Later greetings show nothing until the module data resets. Quirk: flag 384 is an arena flag (the fourth arena win), tested inside the wheel host's chain between 480 and 479. Reproduce it.
- **Fame lines.** 31 villager talk routines of 1023 share one rule. They are registered for 39 villagers and are listed in SPEC-0045 as templates A, B and C. On each talk: A = the number of flags 381 to 385 that are set; d = Random(1, 6); then, when flag 480 is set, e = Random(1, 3) (drawn whether or not it is used). In ten of the routines (villagers 68, 77, 87, 79, 62, 95, 85, 92, 67, 84, 88, 91, 73, 59 and 96), d is drawn only when A > 0, so a talk with no arena win takes one draw fewer. The outcome is the same, because d is at least 1. Reproduce the draw count. If d < A: an arena-fame line chosen by the highest arena flag set (two- or three-line cycles). Else, if flag 480 is set and e = 1: a wheel-fame line (a three-line cycle). Else the routine's normal flag chain. Example, entity 69: wheel-fame lines 92, 93 and 94; arena-fame line 84 (highest 381), 85 (382), 86 (383), 87 and 88 (384), 89 to 91 (385). Since d is at least 1, an arena-fame line needs A >= 2, so its chance is (A - 1) / 6.
- The module also contains three more routines with this rule (lines 627 to 649, 771 to 799 and 900 to 928) that no villager is registered to. Those lines never show.
- Shared routines (one routine, several villagers): 60/61, 76/78, 89/93, 62/95, 85/92, 59/96 and 84/88/91.
- Flag 480 is set only by the 1023 prize wheel, after the fifth win in a row in one game: trigger 36, a won spin, yes to all four go-on prompts (lines 1027 to 1030), and a won fifth spin; the wheel then shows line 1031, pays 160 NP, resets its round and stage words and sets flag 480 (High). So the wheel-fame lines and the wheel host's line 1016 are reachable. The wheel host, these fame lines and four villager routines of pack 1150 (same rule, standard draw order) read it.
- **Pack 1150 fame lines (High).** Pack 1150 is the interior entered from the 1023 teleport at index 1 while flag 10 is clear and flag 136 is set (SPEC-0045), so it is reachable. Its Init registers four talk routines without any gate: entity 0, 1, 2 and 3, with line bases b = 9, 38, 67 and 96. Each talk takes d = Random(1, 6) first and then, with flag 480 set, e = Random(1, 3) (the standard order). The choice:
  - d < A (A as above): the arena-fame line by the highest arena flag set: 381 gives line b, 382 gives b + 1, 383 gives b + 2, 384 cycles b + 3, b + 4, and 385 cycles b + 5, b + 6, b + 7.
  - Else, flag 480 set and e = 1: the wheel-fame cycle b + 8, b + 9, b + 10.
  - Else the normal chain by the highest of flags 202, 193, 150, 142 and 138 set: 202 cycles b + 26 to b + 28, 193 cycles b + 23 to b + 25, 150 cycles b + 20 to b + 22, 142 cycles b + 17 to b + 19, 138 cycles b + 14 to b + 16; none of them set cycles b + 11 to b + 13.
  - Cycles: each routine keeps one mode word and one step word. A cycle line first sets the mode to that cycle and, when the mode changes, the step to 0; it then shows the line for the step and advances the step, wrapping to 0 after the last line. The single lines of ranks 381 to 383 leave both words unchanged.
  - Lifetime: 1150 is an interior, so both words return to their file values on every entry (section 1) and every visit starts each cycle at its first line.

### 4. Pack 722 betting race

Confidence: High for the rules; Medium-High for the win chances. Registrations and the race set-up and cameras: SPEC-0044.

- **Odds draw.** On each arrival with flags 76 and 562 set (after a 2.0 s wait), and after every race: the three odds are cleared; then for racer 0, 1 and 2 in turn, Random(1, 4) is drawn again until it differs from every odds value already drawn. S = the sum of the three (6 to 9). Odds 1 is the favourite.
- **Bet** (triggers 0 to 2 for racers 0 to 2; flag 562 set; no bet being handled). Each tick: square-button context 4. On a Square press: the bet-in-progress marks are set; the prompt is line 67, 68, 69 or 70 for the racer's odds 1, 2, 3 or 4. Yes: if NP is below 50, line 75; otherwise pick = the racer, RemoveNeopoints(50), and the race starts. No: no bet. After either answer: Wait(1.0 s), and the marks are cleared.
- **Race step** (one pass, after the set-up of SPEC-0044): r = Random(0, 2). If r equals the racer chosen on the previous pass and Random(1, 3) <= odds[r], r = Random(0, 2) once more. Then if Random(1, S) > odds[r], racer r plays its step animation (animation 2) and the race waits for it to end, and count[r] = count[r] + 1; otherwise Sleep(1). Cameras 6, 7 and 8 play the first time any racer's count reaches 5, 10 and 14. When count[r] exceeds 15 (16 steps), the race ends: ReturnToNormalCamera, hero to idle, the result, odds redrawn. Each pass ends with Sleep(1), so a pass without a step takes 2 ticks.
- **Previous racer (High).** The previous racer is set to none (-1) at the start of every race, so the first pass never redraws and makes no Random(1, 3) draw: its draws are Random(0, 2) and then Random(1, S). At the end of each pass the previous racer becomes the final r of that pass (after any redraw), whether or not r stepped.
- **Result** (winner w): line 71, 72 or 73 for w = 0, 1 or 2; fade 0.7 s; spectators and racers reset (entity 15 to one of locators 20 to 25 by Random(0, 5)); if pick = w, the hero receives odds[w] items 139, so a winning 50 NP bet pays odds x 100 NP.
- **Resets.** Counts per race. Odds per arrival and per race. The pick is not cleared after a race; the next bet overwrites it.
- **Win chances** (odds of racers 0, 1, 2 -> chance each racer wins; expected NP gained per 50 NP bet on each). (1, 2, 3): 0.835, 0.157, 0.008; +33.5, -18.6, -47.6. (1, 2, 4): 0.810, 0.189, 0.001; +31.0, -12.2, -49.4. (1, 3, 4): 0.947, 0.046, 0.007; +44.7, -36.3, -47.0. (2, 3, 4): 0.754, 0.193, 0.054; +100.8, +7.8, -28.6. Other orders of the same odds give the same chances per odds value. A bet on the favourite always gains on average. A race takes about 50 passes and 34 steps.

### 5. Pack 758 time trial (keeper entities 26 and 28; triggers 13, 14 and 15)

Confidence: High. Trigger 15 is the finish, not the start: the clock starts at the keeper.

- **Offer.** The keeper's line by the ladder flags: 147 (435 set; talk only, no race), else 148 (433 set), else 149 (431 set), else 150. Except with 435 set, that line is a yes/no prompt. No: line 158. Yes: line 159 and the start.
- **Start.** ActivateTimer(0, limit 0, count up), which holds the clock at 0; player control blocked; fade out; the course reset (objects 6 and 3, breakables 17 to 20 restored, armed and finished cleared); hero to locator 150; normal camera; fade in; line 157; control restored; armed = 1; ActivateTimer(0, limit 600, count up). The clock now counts up from 0 and stops at 600 s.
- **Trigger 15, the finish gate** (state 1): object 2 plays forward. If armed is 1 and finished is 0: finished = 1, T = the timer value, ActivateTimer(T, limit T, count down) so the clock shows T and stays there, and objects 6 and 3 play. Then wait until the hero leaves the trigger and play object 2 back.
- **Trigger 14, the judge** (state 1, finished = 1): the hero walks to locator 148; t = T with the fraction dropped. The run qualifies when t <= 45, or t <= 90 with flag 433 clear, or t <= 180 with flag 431 clear (flag 433 is only set after 431 on a normal ladder, so this equals the three-step ladder). A qualifying run: with 433 set, set flag 435 and line 151; else with 431 set, set flag 433 and line 152; else set flag 431 and line 153. A failing run: line 154 (433 set), 155 (431 set) or 156. Then the camera returns, the course is reset and the clock is turned off.
- So the three ladder steps need at most 180, 90 and 45 whole seconds. A fast first run (t <= 45 with no flags) still sets only 431.
- **Trigger 13, the hint** (state 1, hint word 1): the keeper talks and the hint word is cleared; trigger 5 sets it again. The talk alternates two lines per rank group: lines 141 and 142 with flag 433 set; lines 143 and 144 with 433 clear and 431 set; lines 145 and 146 with neither. A mode word and a step word keep the alternation; changing group restarts it at the first line. Flag 435 is not tested (High).
- **No reward.** No item or NP is given. Flags 431, 433 and 435 are read only by this module.
- **The clock across areas (High).** The clock is one engine-wide HUD timer (SPEC-0023 section 31), not module data. Only the script timer calls change its visible flag and its value; no area load, interior entry or exit, transition, death, save load or new game writes them. It steps on every in-play tick while it is visible and running, in whatever world is current. So during a run the clock keeps counting while the hero is in an interior or another area. The armed and finished words are 758 module data, which an interior trip keeps (section 1), so after an interior trip the run goes on and the time spent inside counts. Leaving 758 for good does not stop the clock: the 758 Exit routine makes no timer call.
- **Repeat.** Each start re-arms the course and restarts the clock from 0, so a run can be retried at once after a failed judgement. With flag 435 set the keeper only talks (line 147) and no further run is possible.

### 6. Pack 934 find game (Farm Road; SPEC-0058 step 36)

Confidence: High for the rules and values; Medium-High for the entity pairs; Medium for the three-target time-out placement; Medium-High for the reload edge case; Medium for the lines chosen outside the game.

- **Entity pairs (Medium-High).** Init registers the same talk routine for both ids of each pair in every load, without a gate: 9 and 42 (the offer, and the catch of that target), 1 and 43, 3 and 44, 0 and 40; entity 2 is registered twice to one routine. An id with no character in the loaded set never fires. Flag 248 picks the set: with 248 clear the find-game slot loads its one-character set (entity 9, one character activated); with 248 set it loads its three-character set (entities 42, 43 and 44, three characters activated). So entity 9 exists while 248 is clear and entities 42 to 44 while it is set.
- **Hiding places.** Place k (0 to 7) is locator 3 + k with navmesh 10, 11, 12, 13, 0, 14, 0 or 15 in that order.
- **Offer** (entity 9, or entity 42 in the other world state; flag 220 clear; no game running; no time-out in progress). The time limit T and the line come from the one-target results: flag 473 set: line 33, T = 180; else 472: line 32, T = 180; else 471: line 31, T = 240; else line 30, T = 300. Prompt 34. No: line 61. Yes: the start. With flag 248 clear the game has one target; with 248 set it has three. The three-target game also takes T from flags 471 to 473.
- **Start.** Fade out 0.7 s. The target or targets get AI behaviour 0 and are taken under control. One target: entity 9 goes to hiding place k = Random(0, 7). Three targets: entities 42, 43 and 44 go to three distinct hiding places (Random(0, 7) drawn again on a repeat). Each target loops animation 42 for 301 s. The caught marks are cleared. ActivateTimer(T, limit 0, count down). Flag 470 (one target) or 474 (three targets) is set. Fade in 0.7 s.
- **Clock watch** (a thread running every tick for the module's life). t = the timer value.
  - While a game runs and t < 10: the timer colour is (220, 40, 40, 128) when floor(2t) is even and the default colour when it is odd, a blink every half second (High; a null colour restores the default). Example: t = 9.7 gives floor(19.4) = 19, odd, default; t = 9.2 gives 18, even, red.
  - When a game runs and t = 0.0 (the time-out): busy = 1; line 60 (one target) or 56 (three targets); fade out; the game flag is cleared. One target: entity 9 goes back to locator 11 (navmesh 9) with AI behaviour 5. Three targets: only the targets already caught are placed at locator 11 with behaviour 5 again; the others stay at their hiding places (Medium). Hero to locator 14; fade in; then the replay prompt 59 (one target) or 57 (three targets), with T computed again from the flags: yes starts a new game; no shows line 61 or 58. Wait(1 s), busy = 0.
  - On the first tick with no game running after a game, the timer is hidden.
- **Catch, one target** (talking to entity 9 while flag 470 is set). Line 35, 36, 37 or 38 by the results before this game (none, 471, 472, 473, highest first). Then the time tier: t >= 120 sets flag 471; 60 <= t < 120 sets 472; t < 60 sets 473. The timer is hidden and flag 470 cleared. Then a fade, target and hero placed, and the result ladder, which sees the tier flag just set: 473 set: line 42 (set 473); else 472: line 41 (set 473); else 471: line 40 (set 472); else line 39 (set 471). So a first catch with t >= 120 leaves both 471 and 472 set, and the next offer has T = 180.
- **Catch, three targets.** Talking to entity 42, 43 or 44 while flag 474 is set marks it caught. A target that is not the last is sent back (fade, locator 11, behaviour 5). On the last one: t >= 120 sets flag 475, 60 <= t < 120 sets 476, t < 60 sets 477; the timer is hidden; fade, hero to locator 14, flag 474 cleared.
- **Quit** (teleport 0, or triggers 3, 4 and 5 at the edges of the area, while a game runs): prompt 211. Yes: line 213, fade out, timer stopped, reset and hidden, object 0 off, the game flag cleared (for 470 also entity 9 back to behaviour 5), fade in. No: line 212.
- **Exit** (module unload): StopTimer, ActivateTimer(0, 0), DeactivateTimer. The clock is engine-wide (section 5), so this also stops, zeroes and hides any clock another module left showing.
- **No reward.** Flags 470 to 477 are the only output.

### 7. Pack 1358 delivery tasks (entity 20)

Confidence: High.

- **Tasks** k = 1 to 8: task flag 604 + k (605 to 612), done flag 678 + k (679 to 686). Prerequisites: none for tasks 1 and 2; flag 100 for 3 and 4; flag 106 for 5 and 6; flag 104 for 7 and 8.
- **Completion.** Talking to the resident while the task flag is set and the done flag is clear removes every item 33 from the hero and sets the done flag. Residents: task 1 pack 1421 entity 0; task 2 pack 1417 entity 0; task 3 pack 1407 entity 0; task 4 pack 1419 entity 0; task 5 pack 1423 entity 0; task 6 pack 1425 entity 0; task 7 pack 1405 entity 0; task 8 pack 1353 entity 16. No prize is paid by 1358 or by the residents.
- **Giver, in order.** (1) Flag 604 clear: set it, line 32. (2) Else, for the first k whose task flag is set and done flag clear: reminder line 4 + k. (3) Else build the list of tasks, in k order, whose done flag is clear and whose prerequisite holds; n = its length. If n > 0: r = Random(1, n) and the offer is list entry r counted from 0, shown as prompt 12 + k (task k). Yes: set the task flag, line 20 + k. No: line 29, 30 or 31 by Random(1, 3). (4) If n = 0: line 33, 34 or 35 by Random(1, 3).
- **The package (item 33; High).** Item 33 is the package. Each accept line 20 + k (lines 21 to 28) carries a dialogue give command for one item 33 (SPEC-0022, give items), so showing the line gives the hero one item 33 with the item-received notice. No other line, script or placement gives it. A resident removes all item 33 on completion. A yes to the read past the list (draw defect, below) shows no accept line and gives no package.
- **Draw defect.** Because r runs from 1 to n but entries are counted from 0, entry 0 is never offered, and r = n reads the word just past the list. The list is 8 words in the talk routine's own stack frame, and only its first n words are written on this call. Talk routines run as script threads on one shared script stack, so the word just past the list is whatever the last thread or routine to use that depth of the stack left there: it depends on what ran before in this session, not on any data on the disc, and it can by chance be 1 to 8 (Medium). For n = 8 it is the routine's saved call record pointer, never a task number (High). When the word is not 1 to 8, the prompt index stays -1 (a built-in prompt, SPEC-0030 row 100) and a yes sets nothing and shows no task line. With n = 1 the one remaining task is never offered.
- **Clean behaviour.** Reproduce the skip of entry 0. For the read past the list, treat it as no offer: prompt index -1, nothing set.

### 8. Arena ladders and the 1371 wave trial (summary)

Confidence: High. The full scripts are in SPEC-0041 (1176, 1371) and SPEC-0042 (1350).

- **Meridell arena** (1023 entity 66, then pack 1176). The round is chosen by flags 381 to 384. Wins set 381, 382, 383, 384 and 385 in turn; a loss sets 380. On the return to 1023 the hero's hit points are raised to 50 percent when below it. Prize ladder (SPEC-0021, Neopoints): the first win gives the Wooden Shield (106), or 300 NP if the hero holds 107; then 500, 750, 1000, 2000, and 2000 again on every later win.
- **Altador arena** (1350 entity 24, then pack 1371 branch A). The round is chosen by flags 596 to 599. Each win gives item 147 x2 and sets the next of 596 to 600; a loss sets 595. The challenge closes once 600 is set.
- **1371 wave trial** (branch B, flag 100 clear). Three gates open the waves in a fixed order, each only while the objects named are not animating:
  - trigger 2, with flags 342, 343 and 344 clear and object 1 not animating: hero to locator 23, object 1 plays, wave 1 (region index 6); clearing it sets flag 342;
  - trigger 4, with 342 set, 343 and 344 clear and object 1 not animating: locator 25, object 3, wave 2 (index 7); sets 343;
  - trigger 3, with 342 and 343 set, 344 clear and objects 3 and 2 not animating: locator 24, object 2, wave 3 (index 8); sets 344.
  - Each gate: a 1 s fade, objects 10, 9 and 4 activated, camera 8, sound 10 while the gate object plays, camera back, slot reset, wave load. A wave is cleared when all of its characters are dead.
  - Trigger 1, with all three set: flags 345 and 100 set and movie 61. The prize is the story progress.

### 9. Rewards and flags

| Activity | Pack, entry | Cost (NP) | Reward | Flags set | Reset |
| --- | --- | --- | --- | --- | --- |
| Prize wheel | 1023 trigger 36 | 15 | 20 to 160 NP in coins | 478, 479, 480 | game words per game; angle per module life; flags permanent |
| Prize wheel | 758 trigger 3 | 25 | 45 to 325 NP in coins | 536, 537, 538 | as above |
| Prize wheel | 1358 trigger 0 | 100 | 140 to 700 NP in coins | none | as above |
| Betting race | 722 triggers 0 to 2 | 50 | odds x 100 NP (items 139) | none | odds per arrival and per race |
| Time trial | 758 keeper (entities 26, 28) | 0 | none | 431, 433, 435 | course per run |
| Find game | 934 entity 9 or 42 | 0 | none | 470 to 477 | game flag per game; result flags permanent |
| Delivery tasks | 1358 entity 20 | 0 | none (item 33 is given on accept and taken back on completion) | 604 to 612, 679 to 686 | permanent |
| Meridell arena | 1023 entity 66 | 0 | prize ladder (section 8; SPEC-0021, Neopoints), and hit points to 50 percent | 380 to 385 | permanent |
| Altador arena | 1350 entity 24 | 0 | item 147 x2 per win | 595 to 600 | permanent |
| Wave trial | 1371 triggers 1 to 4 | 0 | movie 61 | 342 to 345, 100 | permanent |

### 10. Kept elsewhere

- Registrations and trigger indices of each module: the level specs (SPEC-0043, SPEC-0044, SPEC-0045, SPEC-0041, SPEC-0042).
- The HUD timer: SPEC-0023. Coin values and the wallet: SPEC-0050.
- The Miamice hunt (1163) and cage (1398): SPEC-0045 and SPEC-0043; creature states: SPEC-0029. The Mortog zone (1363) and the Whinny pens (909, 1398): SPEC-0029 and SPEC-0041.
- The combat lessons (1056) and the target practice (1327, which ends with item 160 x100): SPEC-0044.

## Edge cases and error handling

- Wheel angle exactly 360.0: A is reduced only when it is above 360, so A = 360.0 is possible in principle. It gives c = -1, which reads the word just before the lit table (not a cell; in all three modules it is unrelated module data). It needs an exact float sum and is not reachable in practice. Recommended clean behaviour: treat c = -1 as a dark cell (a loss) and write nothing.
- Wheel, fee check: the NP test is made before the start prompt, and the fee is taken only on yes.
- Wheel, a press before tick 92: ignored; the press must be new on a tick at full rate.
- Wheel, cells darkened by wins: stay dark until the next game's new pattern; a later spin in the same game can land on them and lose.
- Wheel host and greetings: after the second wheel-host greeting, nothing is shown again until the module data resets (section 1). For 1023 a hero death, an arena trip or a save load resets it.
- Fame lines with only flag 381 set: A = 1, so no arena-fame line can show; the rank-381 line of each villager is unreachable while the arena flags are set in ladder order.
- Wheel fame lines need flag 480, which only five wins in a row at the 1023 wheel set. The 758 wheel sets 538 instead, and the 1358 wheel sets no flag.
- Race, odds draw: the redraw loop always ends, since three distinct values from four exist.
- Race, pick left from an earlier race: the race starts only through a bet, which sets the pick first, so a stale pick never pays.
- Time trial, trigger 15 with armed 0 or finished 1: only the gate animation plays.
- Time trial, no finish: the clock counts up to 600 and stays there; the judge does nothing until the finish gate is passed.
- Time trial, 758 reloaded during a run (a hero death self-reload, a save load or a later driver load of the exterior): armed and finished return to their file values of 0, but the clock is untouched. It keeps counting up to 600 s and stays shown; the finish gate and the judge do nothing. The next start at the keeper restarts it from 0; unloading pack 934 also hides it (section 6, Exit).
- Find game, game flag still set at load: this happens in normal play. Pack 934 has a save point, the search leaves the hero free and not busy, and the progress flags (470 and 474 among them) are saved and loaded with the slot. So a save made during a run, then loaded, starts in 934 with flag 470 or 474 set. The timer value is then 0 (the clock starts at 0 and every module exit sets it to 0), so the first tick of the clock watch runs the time-out sequence: line 60 or 56, the target or targets placed back, the hero to locator 14, then the replay prompt. In the three-target case no target is marked caught after a load, so none is placed at locator 11 (Medium-High).
- Delivery tasks, a list entry read past the list (section 7): no offer.
- Delivery tasks, item 33: each completion removes all of it, whether or not the hero holds any.

## Test vectors

All draws are synthetic.

Prize wheel:

- Spin from A = 0, press seen on tick 92 (the first tick it can be): after that tick A = 67.6998; cell 7 - floor(67.6998 / 45) = 6.
- A = 10: cell 7. A = 300: cell 1. A = 359.9: cell 0. A = 45.0: cell 6.
- Rate: after 90 additions 8.9999 (below 9.0); after 91 additions 9.0999985; no further additions.
- New pattern, draws 3, 3, 6, 0: cells 3, 6 and 0 dark; lit cells 1, 2, 4, 5 and 7. Mask in 758: bits 0, 3 and 6 set (M = 73). Mask in 1023: bits 2, 5 and 8 set (M = 292).
- 1023, NP 100: fee 15 (85). Win spin 1, cash out (r = 0): line 1033, 20 NP as two items 137 (105), flag 479.
- 1023, five wins in a row: line 1031, 160 NP as one item 139, two items 138 and one item 137; flag 480.
- 1023, NP 100: win spins 1 to 4 and go on each time, lose spin 5 (r = 4): line 1041, flag 478, NP 85.
- 758, NP 100: fee 25 (75), win spin 1, cash out: line 326, 45 NP as one item 138 and two items 137 (120), flag 537.
- 758, cash out after three wins (r = 2): line 328, 105 NP as one item 139 and one item 136.
- 758, five wins: line 324, 325 NP as three items 139 and one item 138; flag 538.
- 1358, NP 99: line 109, nothing else.
- 1358, cash out after two wins (r = 1): line 101, 180 NP as one item 139, three items 138 and one item 136.

Greetings and fame lines (1023):

- Flags 381, 382 and 383 set, 480 clear; draw d = 2: A = 3, 2 < 3: arena-fame line for rank 383. Draw d = 3: normal chain.
- Flags 381 to 385 set and 480 set; draws d = 6, e = 1: A = 5, 6 is not below 5, e = 1: a wheel-fame line.
- Flag 480 set, no arena flags; draws d = 1, e = 2: normal chain (both draws are made).
- Wheel host, flags 384 and 479 set, first greeting: line 1017. Second: line 1021. Third: nothing.

Fame lines (1150):

- Entity 0 (b = 9), flags 381 to 384 set, 480 clear, draw d = 2: A = 4, 2 < 4, rank 384: line 12; the next such talk: line 13; the one after: line 12.
- Entity 1 (b = 38), flags 381 and 382 set, d = 1: A = 2, 1 < 2: line 39 (a single line; the cycle words are unchanged).
- Entity 2 (b = 67), no arena flags, flag 480 set, draws d = 4, e = 1: wheel-fame line 75, then 76, then 77, then 75.
- Entity 3 (b = 96), no arena flags, 480 clear, flags 138 and 150 set, d = 3: normal chain, highest 150: line 116 on the first talk of the visit, 117 on the second. Leave and re-enter: line 116 again.
- Entity 0, flags 202 and 138 set, no arena flags, 480 clear: line 35, then 36, 37, 35.

Betting race (odds draw then passes):

- Draws Random(1, 4) = 2, 2, 4, 1: odds (2, 4, 1), S = 7.
- Odds (1, 3, 4), S = 8, previous pass racer 1. Draws r = 1, Random(1, 3) = 2 (2 <= 3, redraw), r = 0, Random(1, 8) = 1: 1 is not above 1, no step, the pass takes 2 ticks.
- Same odds, previous racer 2. Draws r = 0, Random(1, 8) = 5: 5 > 1, racer 0 steps.
- Racer 2 (odds 4) wins with pick 2: four items 139 (400 NP) for the 50 NP bet.
- First pass of a race, odds (1, 3, 4), S = 8. Draws r = 2, Random(1, 8) = 7: no redraw test (no previous racer), 7 > 4, racer 2 steps; two draws in all. Second pass, draws r = 2, Random(1, 3) = 3 (3 <= 4, redraw), r = 1, Random(1, 8) = 2: 2 is not above 3, no step; the previous racer is now 1.

Time trial (758):

- Count-up from 0: the whole-second value first reaches 46 on timer update 2761, 91 on update 5460 and 181 on update 10861.
- No flags, T = 150.4: t = 150, qualifies (<= 180): flag 431, line 153.
- Flag 431 set, T = 60.0: qualifies (<= 90 with 433 clear): flag 433, line 152.
- Flag 433 set, T = 46.0: t = 46, fails: line 154.
- No flags, T = 30.0: flag 431 only, line 153.
- Flag 431 set, T = 120.0: fails: line 155.
- Run started; after 1,200 in-play ticks (clock 20.0 s) the hero enters an interior, spends 1,800 in-play ticks there (no dialogue box, no pause) and returns: the clock reads about 50.0 s and the run is still armed. Passing the finish gate after another 600 ticks gives T of about 60.0, t = 60.
- Run started, the hero dies on deadly ground and 758 reloads itself: armed = 0, finished = 0, the clock goes on counting and pins at 600.0; trigger 15 plays only the gate animation; trigger 14 does nothing.

Find game (934):

- T = 300: the count-down value reaches exactly 0.0 on timer update 18002 (T = 240: 14401; T = 180: 10800). With T = 300 it is first below 120 on update 10803.
- Blink: t = 9.7: floor(19.4) = 19, odd: default colour. t = 9.2: floor(18.4) = 18, even: red.
- One target, no earlier results, hiding draw 5: entity 9 at locator 8 with navmesh 14.
- One target, no earlier results, caught with t = 130: line 35, flag 471, then the ladder: line 40 and flag 472. Next offer: line 32, T = 180.
- One target, flag 471 set, caught with t = 50: line 36, flag 473, ladder: line 42 (473 set again). Next offer: line 33, T = 180.
- Flag 248 clear: talking to entity 9 runs the offer; entity 42 is not in the loaded set, so its registration never fires. Flag 248 set: the offer comes from entity 42, and entities 42, 43 and 44 are the three targets.
- A clock left showing by another module (for example a 758 run, value 75.0, visible, running): pack 934 unloads; its Exit stops it, sets it to 0.0 and hides it.

Delivery tasks (1358):

- Flags 604 set, 100 set, 106 and 104 clear, no task taken, tasks 1 to 4 not done: list (1, 2, 3, 4), n = 4. Draw r = 2: task 3 offered, prompt 15; yes sets flag 607 and shows line 23. Task 1 is never offered while n > 1.
- Only task 2 left (n = 1): r = 1 reads past the list; no task is offered.
- Word past the list, n = 3 (tasks 1, 2 and 3 left), r = 3: the offer reads the fourth word of the list area. If it happens to hold 5, prompt 17 (task 5) is shown and a yes sets flag 609 although task 5 was not in the list; if it holds 0 or any value above 8, prompt index -1 is used and a yes sets nothing. The clean behaviour treats every such read as no offer.
- Find game across a save: flag 470 set, the hero saves at the 934 save point with 200 s left, then loads that slot later: on the first tick in 934 the clock reads 0, so line 60 plays, entity 9 goes to locator 11 with behaviour 5, the hero goes to locator 14 and prompt 59 offers a replay; no tier flag is set.
- Task 5 taken (flag 609 set, 683 clear): the giver shows reminder line 9. Talking to pack 1423 entity 0 removes all item 33 and sets flag 683.
- Task 3 accepted: line 23 is shown, and its give command adds one item 33 (held count 0 to 1, item-received notice). The reminder line 7 on a later talk gives nothing. Completion at pack 1407 entity 0: held count 1 to 0, flag 681 set.

Wave trial (1371, branch B):

- Flags 342 to 344 clear, trigger 4 entered: nothing (342 is needed). Trigger 2 entered, object 1 still: wave 1 loads.
- Flags 342 and 343 set, object 3 still animating, trigger 3 entered: nothing.

## Open questions

1. Resolved: 1358, the word read past the task list when n is below 8, has no fixed value: it is whatever the last script thread left at that depth of the shared script stack, so it depends on what ran before in the session and may by chance name a task 1 to 8 (section 7, Draw defect). The clean behaviour of section 7 treats it as no offer. (Medium)
2. Resolved: the clock is engine-wide and only script timer calls change it, so it keeps counting through an interior visit or any area change, and no area load resets it; an interior trip keeps the run armed, and a driver reload of 758 drops the run but leaves the clock counting (section 5 and Edge cases; High).
3. Resolved (proactive open close-out): object-0 wedge masks in packs 758 and 1358 map script cells 0 through 7 in mask order (display mask is twice the script cell bit), confirming those copies use the same mask-to-sector crosswalk. Pack 758 also has a separate downward-pointing needle model at the wheel’s top rim; its static direction falls on a sector boundary, so at A = 0 the needle stands on the edge between two sectors, which matches the scoring rule (cell 7 for A from 0 up to 45, falling by one cell for each further 45 degrees as the wheel turns): the script's rule is the authority and the needle needs no further phase. For any A that is not a multiple of 45 the cell the player sees under the needle is the cell scored (Medium; the model-frame direction of the needle in packs 1358 and 1023 is not recorded in the data and is not needed for the rules). The on-screen alignment at the exact edges (A a multiple of 45) needs a runtime check (class C); default if unresolved (rewrite choice): score by the floor rule above.
4. Resolved: both ids of each pair are registered in every load, and flag 248 picks the loaded set: entity 9 with 248 clear, entities 42 to 44 with 248 set (section 6, Entity pairs; Medium-High). Flag 470 or 474 survives to a reload in normal play, by saving at the 934 save point during a run and loading that slot; the first tick then runs the time-out (Edge cases; Medium-High).
5. Resolved: pack 1150 is a reachable interior; its four routines (entities 0 to 3) use the standard draw order, and their lines and cycles are given in section 3 (High).
6. Resolved: the accept lines 21 to 28 of pack 1358 each give one item 33 through a dialogue give command (section 7; High).
7. Resolved: the previous racer is none at the start of every race, so the first pass makes no redraw test (section 4; High).
8. Resolved: the target-practice input gates are specified with pack 1327 in SPEC-0044: step 229 requires a new Circle or R2 press, step 231 requires an uninterrupted hold of at least 31 ticks, and step 232 counts fresh presses toward three (Target-practice thread; High).
