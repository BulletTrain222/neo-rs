# SPEC-0007: Game-logic timing

- Answers: proactive (open close-out); none otherwise (includes the in-game calendar section)
- Status: Draft
- Disc build: SCUS-97367 v1.00 (same fingerprint as SPEC-0001)
- Confidence: High for the fixed step, pacing and input order. Medium for the slow-path extra waits (see Open questions). The start-up words that gameplay code reads at run time (frame-rate word 60, tick-length word 1/60 s, half-rate word 0.5) are written at start-up from the boot rate table (High for the values; High for the start-up timing of the write, see Rate selection). They are not disc data. The per-element step word (1/30 s) is field +0x04 of the physics block, written once at start-up from fixed values (High; SPEC-0033, Physics block). The pass counter and game-time word start at zero and advance at the end of every pass (High). The up vector and the random generator state are set at start-up and are not tuning values (High). Packed time difference to ticks: Medium for the arithmetic, the field meanings are open. Calendar: High for the day-count, leap-year, weekday and date-normaliser arithmetic; Medium for the clock step factor (4 times the tick) and the clock record layout; High for the year turn (the calendar breaks after December 31) and for the boot-only reset of the clock.

## Summary

Gameplay runs on a **fixed time step of exactly 1/60 second**, one update per pass of the main loop, and one pass per video vertical blank. Elapsed time is never measured. The original does not clamp, accumulate or catch up: if a pass takes longer than one field, the game simply runs slower, and the logic still advances by 1/60 s per pass. Controller input is sampled once per pass, before anything else updates. The clean side can drive the simulation with a fixed 60 Hz tick that consumes one input snapshot per tick. The same pass also advances the in-game calendar and two play-time totals; the calendar and total A run at four in-game seconds per real second, total B at one per real second; see "In-game calendar and clock".

## Where it lives on the disc

Nothing needs to be read from the disc. The values below are constants. They come from a small rate table in the boot executable, and on this build it always selects the same row and column.

For reference, the boot data also holds the 32-bit float 60.0 at data address 0x30C598 (checked in the boot file image). It agrees with the rate above. The clean side can use the constant 60.0 directly and does not need to read it from the disc.

## Behavior

### Rate selection (once, at boot)

At start-up the game sets the tick rate once, from the boot rate table (data address 0x30C598). The rate used is the entry for row 0 and column 0 of that table, which is **60.0** (the first of the four rates in the table below). From it, dt = 1/60 s and the half-rate value is 30/60 = 0.5. The write is part of the start-up sequence and happens before the first gameplay pass, after the start-up static initialisers have run. Nothing changes the rate afterwards. The table's other four entries are not used on this build.

- The engine has a rate table with two video rows and two columns:

  | | Column 0 | Column 1 |
  | --- | --- | --- |
  | Row 0 (NTSC) | 60 updates/s | 30 updates/s |
  | Row 1 (PAL) | 50 updates/s | 25 updates/s |

- The table is indexed as column + 2 times row. On this build the row is always 0, because the video-mode test always reports NTSC, and the column is always 0 (a fixed boot setting). So the rate is **60 updates per second**. Nothing changes it after boot.
- From the rate the engine derives:
  - `dt = 1 / rate` = 1/60 s, used by game systems as their time step;
  - `step30 = 30 / rate` = 0.5, a "fraction of a 30 Hz frame" factor;
  - a vblank count per pass = column + 1 = 1.

The clean side should hard-code rate = 60.0, dt = 1/60 and step30 = 0.5. Keep the 30 Hz column only if the clean side wants a debug option; the original never uses it.

These derived values are written at start-up to run-time words that gameplay code reads (see "Per-tick words read by gameplay code"): the rate goes to a frame-rate word (60, ticks per second; 2026-11-11, Medium-High, corrected: the foliage code and the Cloud Beast timer code read it as a 32-bit float holding 60.0 and multiply a float by it, so it is not read there as an integer count), dt goes to a tick-length word (1/60 s), and step30 goes to a half-rate word (0.5). The words are zero-filled in the file image (beyond its end), so they are not disc data; the start-up write fills them from the table entry above. The clean side takes the values 60, 1/60 and 0.5 from this spec. The write is done once, by the start-up frame setup, into the frame object at data address 0x368F00: the rate goes to its +0xA0 field (data address 0x368FA0), dt to +0xA8 and the half-rate to +0xA4.

### One pass of the main loop

In this order:

1. **Input**: snapshot every controller's latest state once. All gameplay in the same tick sees that snapshot. Button-combo and hold detectors count passes, so "N frames" means N passes.
2. **Update**: all game systems update once with dt = 1/60 s. There is no inner loop running several logic steps per pass, and there is no frame skip.
3. **Render**: submit the frame.
4. **Pace**: wait until at least one vertical-blank start has occurred since the previous pace point, then reset the vblank counters to zero. Any extra vblanks that passed during an overlong pass are discarded. The two counters (blank starts and blank ends) are counted by the vertical-blank interrupt handlers and are also cleared once at boot, just before the first pass, after a field-sync wait (SPEC-0026, Boot order), so the first pass is paced from that point (High).
5. **Clock**: game time (a float, in seconds, data address 0x368F94) += dt, and the pass counter (an integer, data address 0x368F98) += 1. Both start at zero: the counter is 0 and game time is 0.0 before the first pass. This step is the last thing in every pass, so during pass n (counting from 0) the counter reads n and game time reads n × dt.

The full order of the game systems inside one pass, the gates of the world update and what keeps running while a screen freezes the world are in SPEC-0026 (The main loop, one pass; Pause and freezing screens). A movie plays inside the pass and blocks it until the movie ends, so game time, the pass counter and the calendar do not advance while a movie plays (High).

### Order of work inside one tick

One tick is one pass. There is no separate per-frame cadence: every item below runs at most once per pass, a pass ends with exactly one frame submit and one vertical-blank wait, and the pass counter moves once, last (High). Items differ only in their gates: the world branch (items 6 to 8) runs only while the game-state code is below 9 and the world-enable word is nonzero; the effect manager advances only while the code is 8 or less; everything else runs on every pass. Units that look like a second cadence are tick counts: a clip frame lasts 4 ticks at play rate 1, a script Sleep(1) yields for one tick, and one in-game second is 15 ticks. The order below is High. SPEC-0026 (The main loop, one pass) holds the gates, the pause rules and the rest of each step.

1. **Input.** One snapshot of every pad. Every later item of the tick reads that snapshot.
2. **Status.** Load-busy word, timers, accumulators and the game-state code. The code decides which later items run.
3. **Rumble.** Vibration sync and timeline (skipped under code 9). It reads the rumble requests made on earlier ticks.
4. **Camera.** Mode update on every pass; the follow update only when the play-active word is set; then the shake applier and the view and projection set-up. It runs before any character, so it reads the hero's pose, position and root step from the end of the previous tick: the camera lags the hero by one tick (SPEC-0027).
5. **World tick.** The world's six sub-tables (none is ever filled in play, so they do nothing), placed-object animation list and movable blocks.
6. **Region characters (enemy and NPC think).** The current world's own region, then its slot regions, then the second world's regions. Each active character, in slot order, runs three calls in this order: (a) its update hook, which is its think (state machine, class coroutine, requests for clips and effects); (b) its motion-record tick (ground follow, contacts and collision against the world and the moving objects, gravity, integrator, root travel using the root step stored by the previous tick); (c) its animation pass (start the newly requested clip, evaluate, store the root step for the next tick, pose the skeleton). Everything here reads the hero as left by the previous tick (pose, bones, body cylinder, state), because the hero updates later in the tick. A hit that an enemy lands on the hero is delivered inside this item, so the hero's update of the same tick sees the new state (High; every route is listed in "When a hit on a hero takes effect" below).
7. **Pools.** Placed-effect lists, then the projectile pool (10 records), the common-enemy attack manager and the item showcase step. The boundary-contact word is then cleared.
8. **Selected hero.** Only the selected hero runs; the other slot does not update. In order: (a) the hero update hook (speed scale reset, attack window from the clip frame, shot step, action-state switch, which requests clips); (b) the hero's motion-record tick (facing, root step of the previous tick, gravity, driver with the contact and trigger walks, integrator); (c) the hero's animation pass (starts the clip requested in (a) on this same tick, evaluates, stores the root step for the next tick); (d) the attack-family and effect driver (Charm Mote objects), which reads this tick's hero pose. Enemy hits delivered by the hero in (a) or (d) reach an enemy that already ran this tick, so that enemy's think sees them on the next tick. The hero's attack and melee tests read its pose from the end of the previous tick (SPEC-0036, Tick order of the tests). The active petpet then runs the same three calls (hook, motion tick, animation pass).
9. **Animation-effect pool.** Attachment points, reading the poses of items 6 and 8.
10. **Level script step.** Special threads, then (when not hero-swapping) the Update thread, trigger callbacks and control threads, for the current world; then step 1 of the second world. Triggers fired by the hero's contact walk of item 8 and by the region characters of item 6 have already set their callback state, so the callbacks run on the same tick; anything a script requests of a character takes effect from the next tick, since that character has already run (SPEC-0030).
11. **Always-on bookkeeping.** Actor-list rebuild, clock record pass (calendar and play-time totals), alert-board countdown, thread-pool sweeps; then the global tasks (hero swap, memory card), then the camera volume blend.
12. **Draw phase.** World, character, item and projectile draw submission, using this tick's poses from items 6 and 8; the atmosphere-zone update, from this tick's camera eye; the effects pass, which advances the effect manager (code 8 or less) and draws it, so an effect started during items 6 to 10 gets its first advance on that same tick; shadows.
13. **Screen fade** step and draw.
14. **Front end.** The current page's tree is drawn, then the state handler runs; for the in-play state that is the HUD timers, the openers and the swap chord, and it sets the play-active word for the next pass. The HUD reads this tick's health and pools, but a change the handler makes to the tree reaches the screen on the next pass (SPEC-0023).
15. **End-of-pass apply.** Pending hero placement, memory-card step, then the sound-cue pool and music mixer. A cue requested anywhere above is started on the same tick.
16. **Frame submit, pace, DMA start.**
17. **Late items.** The two button-combination detectors, the movie player, the render frame counter, the area-load driver, the buffer flip and last of all the clock step (game time and pass counter).

Rules that follow from the order (High unless marked):
- For a character that has a motion record, think comes before motion, and motion before animation, in the same tick. A clip requested by a think on tick T therefore starts on tick T, plays its first sample on tick T and is drawn on tick T. The root step it stores is used by the motion tick of tick T + 1 (one tick of lag between a clip's root motion and the travel).
- The hero runs after every region character, the projectile pool and the attack manager, and before the script step, the draw phase, the HUD and the audio. So enemies react to the hero's state of the previous tick, while the script, the HUD and the audio see the hero of the current tick.
- A progress or done test on a clip, made in a think on tick T + k, reads the state left by the animation pass of tick T + k - 1 (for the hero, and for every character, since the animation pass follows the think). The done tick of an attack step entered on tick T whose one-shot clip lasts L ticks is T + L + 1 (SPEC-0036, Done tick at the 0.99 gate).
- No item runs twice in a tick, and there is no catch-up: a slow pass is one tick.

**Worked tick timeline (Tor, a one-shot attack clip of 12 frames, 44 ticks at play rate 1; request on tick T = 100).**

| Tick | What happens |
|---|---|
| 99 | The camera (item 4) used the hero pose left by the end of tick 98. |
| 100 | Item 1 reads the press. Enemies think and move (item 6) using the hero as left by tick 99. In item 8(a) the router enters the attack step and requests the clip; 8(b) moves the hero by the root step stored on tick 99; 8(c) starts the clip and samples its first frame, so the draw phase shows frame 0 on tick 100. The camera of tick 101 will follow this pose. |
| 101 | The router's frame counter F is 0 on entry, so no done test runs; F becomes 1 after this tick's check. The camera follows the tick-100 pose. The attack window and melee tests of this tick read the pose of tick 100. |
| 144 | The animation pass of tick T + 44 reaches the last key and sets the finished flag. The router's progress read (item 8(a), before the animation pass) is (12 - 0.25) / 12 = 0.979, below 0.99, so the step is not done. |
| 145 | The router's progress read is 1.0 (finished flag set by tick 144's pass): done test passes on tick T + L + 1 = 145, at F = 44. The router leaves the step on this tick; any clip it requests next starts in 8(c) of tick 145. |
| 145 (after) | The HUD (item 14), the audio (item 15) and the script (item 10) of tick 145 already see the new state. |

### When a hit on a hero takes effect

A hit on a hero is applied at once, inside the call that delivers it. Nothing is queued: no pending-damage word exists, and no later item of the tick applies a hit that an earlier item delivered (High). The hero's take-hit method does all of its work before it returns, in this order: the hit cue and effect, the dead-hero return, the reaction (hurt state written and the flinch clip requested, or the block reaction), the weapon draw, the stun clear, the health subtraction, the death handler when health reaches 0 (the death state is written inside the same call) and the health-display refresh (SPEC-0036, Taking a hit). The caller may then add its own steps after the return (status roll, knockback velocity, projectile disarm). What differs between routes is only where in the tick the call happens.

Three kinds of effect have different ticks:
- Health, the hurt or death state, the hit cue, the knockback velocity and the status change on the tick of the call. Everything that runs later in that tick sees them: the hero's own update if it has not run yet, the script step, the health display (item 14) and the sound apply (item 15, which starts the hit cue on that same tick).
- The flinch clip is only requested by the call. It starts in the hero's next animation pass, which is item 8(c). A hit delivered before item 8(c) of tick T draws the first flinch frame on tick T; a hit delivered after it draws that frame on tick T + 1. In both cases the hurt-state update of the following ticks requests a flinch clip only when neither flinch clip is the one already requested. A request for the clip that is already requested starts nothing (SPEC-0005, Choosing an animation), so a second hit while the hero is hurt does not restart the clip.
- Readers that run before the hero in a tick see a hit delivered in item 8 or later only on the next tick: the camera (item 4) always sees it on tick T + 1, and so does an enemy that thinks before the hit lands. An enemy that thinks after a hit delivered in item 6 sees it on the same tick, because it reads the live hero.

| Route | Where in the tick | What takes effect, and when |
|---|---|---|
| Common-enemy strike (class 17) and the melee weapon volumes of bosses and other enemies | Item 6(a), inside the enemy's think (class coroutine step) | The test reads the hero's body and bones as left by tick T - 1. The take-hit, then the status roll, then the knockback (velocity set and hero written to state 48 when the attack's knockback is above 0) all happen in the same call. The hero's update of tick T already runs in the hurt state (or in state 48 after a knockback), its motion tick uses the new velocity, and the flinch clip starts on tick T (High). The attack must already have been chosen: the common-enemy attack choice runs only while the hero is hittable (see Hit reaction and invulnerability below). |
| Boss attack objects (chain and arc objects, beams, balls) that a boss's think steps | Item 6(a) | Same call rule as the row above. A boss value reaches the take-hit unchanged (SPEC-0036, Taking a hit, step 6). A boss routine that skips the take-hit on a block changes nothing (High). |
| Enemy projectile pool (boss projectiles included) | Item 7, in the pool's own step, before the hero | Take-hit, the status request and the disarm of the shot happen in one call, so a shot cannot hit twice. The hero's update of the same tick sees the new state, and the flinch clip starts on tick T (High). The guard timer used by the block test has not yet been lowered this tick: the hero update that lowers it runs later (SPEC-0036, Guard timer and per-attacker block). |
| Hero ground rules (hurting surface, deadly ground, deep overlay) | Item 8(b), inside the hero's motion tick, after the hero's update | Health, the hurt or death state and the cue change on tick T. The hero's update sees the hurt state on tick T + 1. The flinch clip is requested before item 8(c), so its first frame is drawn on tick T (Medium-High). |
| Poison and other status ticks | Item 8(a), in the hero update's status step | Attacker id -2 with the no-reaction flag: health and the block-style cue change at once, and no hurt state is written. The rest of the hero's update of that tick sees the lowered health. |
| Level-script damage and kills (host damage and kill calls, trigger callbacks, zone helpers) | Item 10, after the hero | The damage call goes through the take-hit with attacker id -1 and reaction flag 0. Health, state and the health display change on tick T; the flinch clip starts on tick T + 1, because the animation pass of tick T has already run (High). The plain deduct, set and percentage calls only change health (and run the death handler at 0); they write no hurt state and request no clip. |
| Hero hit by a Mote object or a hero shot | None | Heroes are never on the character lists those objects walk (no friendly fire; SPEC-0040, Behavior 17). |

Hit reaction and invulnerability, for every route:
- No route sets an invulnerability window. The take-hit sets no timer, and the hurt state does not block a later hit: neither the take-hit, the common-enemy strike, the boss attack objects, the enemy projectile pool nor the level-script damage tests the hero's hurt state or its hittable query before hurting the hero. A hero in the hurt state (or in state 48 after a knockback) can therefore be hit again on the next tick by an attack that is already running, by a boss, or by a pool shot (High). The repeat rewrites the hurt state and nothing else: the flinch request names the clip that is already requested, so the clip does not restart and the hurt state is not extended (High). Only a hero in the death state ignores a hit (the hit cue still plays; no damage), and every hit is ignored while the input-lock word of the game-state record is set (a dialogue box or list picker is up).
- What limits repeats is the attacker: the common-enemy strike keeps a per-attack hit list (cleared at each attack start), a pool shot is disarmed on its hit, and each boss keeps its own latch or timer (boss specs).
- One enemy route is gated by the hero's hittable query: the common-enemy attack choice (SPEC-0038, Behavior 7). While the hero is not hittable, no common enemy picks a new attack and its four attack cooldowns do not count down (High). A hero is not hittable in the hurt state 17, in states 16, 25, 33, 36, 48, 50 to 55, 57, 59 and 60, while Roberta is in her leap or landing sub-states (8 and 13), and while the script-hold word is set (SPEC-0036, Hit detection, Hittable predicate). An attack that was already picked (in its wind-up or its strike window) still lands, once per attack through its hit list. The counter-attack choice (SPEC-0038, Behavior 14), the boss routines and the projectile pool do not use the query. So the hurt state is a pause for common enemies that have not started an attack, not immunity (High).
- Roberta is the only hero with a protected state against enemy hits: while she spins (attack sub-states 6 and 7) the common-enemy strike does nothing to her (SPEC-0036, Guard timer and per-attacker block), and while she is in sub-states 6 to 8 and 13 an enemy pool shot is not tested against her at all (High).
- Length of the hurt state (High for the clip lengths, which are one-shot clips of 4 x (frames - 1) ticks at play rate 1): the flinch clip is 36 ticks for Tor with a weapon and 40 ticks unarmed; for Roberta 43 ticks for id 32 (a looping 11-key clip that passes the 0.99 gate on its 43rd tick) and 45 ticks for id 192 (23 keys at speed byte 1); for both heroes the clip is chosen by Tor's weapon entry (id 32 when Tor holds a weapon, else 192). The hero returns to state 1 in its update of the tick on which the done test passes, one tick after the clip length counted from its first drawn frame (the rule of the worked example above). A repeat hit does not move that tick.
- No blink, flicker or alpha change exists (Medium-High). A hit whose damage is above 0 starts the 14-frame reddish tint of SPEC-0046 (draw step 6) on the hero; no visibility or alpha toggle is tied to the hurt state or to any timer set by a hit. The tint has no effect on whether the hero can be hit.
- A hero that is busy (Tor charging or in its power step, Roberta charging or in her power step or full release, with a weapon-slot item) takes the damage but is not moved to the hurt state and plays no flinch or block clip.

Test vectors (synthetic):
- Tick 200, hero idle with 50 health; a common enemy's strike (damage 8, knockback 0) lands in its think. On tick 200 health is 42, the hero is in the hurt state when its own update runs, the flinch clip (the weapon one when Tor holds a weapon) starts in the animation pass of tick 200, the health display shows 42, and the hit cue starts at the end of tick 200.
- The same strike with knockback 5: the velocity is set and the hero is written to state 48 in the same call; the hero's update of tick 200 runs in state 48 and its motion tick moves it with the new velocity on tick 200.
- Two strikes from different enemies on ticks 200 and 201: both deduct in full (50, 42, 34 with damage 8); there is no immunity after the first. The second hit does not restart the flinch clip (the same clip is already requested), so Tor with a weapon, whose clip starts drawing on tick 200, returns to state 1 in his update of tick 237 whatever the second hit did.
- A common enemy with all four attack cooldowns at 0 and a fresh pick available, a hero that was hurt on tick 200 (state 17 until tick 237): the enemy picks no attack on ticks 201 to 237 (and on tick 200 when it thinks after the enemy that struck) and its cooldowns stay at their values; the hero returns to state 1 in his update of tick 237, which runs after the enemies, so the choice resumes on tick 238. An enemy that had already begun a strike window on tick 199 still hits on tick 200 or later in that window, once (its hit list).
- Tick 300, a script damage call of 5 from a trigger callback (item 10): health drops on tick 300 and the display shows it on tick 300; the hero is in the hurt state from tick 300, and its flinch clip starts on tick 301.
- Tick 400, a pool shot (damage 6) reaches the hero at item 7 while the guard timer is 1: it is blocked (the timer reaches 0 only in the hero's update later in the tick). One tick later, with the timer at 0, the same shot is not blocked.

### Math

- After *n* passes, game time = n / 60 s up to single-precision float accumulation: the original adds the 32-bit float 1/60 (0.016666668) each pass with the console's truncating add (SPEC-0008, Platform arithmetic), so the sum stays a little below n / 60 (0.99999928 after 60 passes). A timer that compares this kind of sum with a time T in seconds (a multiple of 0.05 s) first passes it on tick 60T + 1 (SPEC-0008, Seconds to ticks).
- Real NTSC field rate is 59.94 Hz, but the engine assumes 60.0. Measured against a wall clock, game time therefore runs about 0.1 % fast. The clean side does not need to reproduce that: running the tick at exactly 60 Hz gives the same per-tick behavior.
- When a pass overruns (takes longer than one field), the original runs slower than real time. A modern fixed-step loop that catches up with several ticks per frame would behave differently in slow-down. For original-like feel under load, cap catch-up at one tick per displayed frame, or make it a user option.

### Per-tick words read by gameplay code

Some gameplay routines read run-time words that are not on disc. Several of them lie beyond the end of the boot file image, in the zero-filled part of the loaded segment, and the game sets them while it runs. The roles of the first three words below are resolved by the start-up writes in "Rate selection", the per-element step word (0x3A1A74) follows, and the clock words, the up vector and the random generator state are listed last:

- Frame-rate word (data address 0x368FA0): 60, written at start-up from the boot rate table (not disc data). It is the tick count per second. 2026-11-11 (Medium-High): the foliage code (the bush effect wait, trunc((u + 0.5) x 60.0)) and the Cloud Beast timers read it as the float 60.0 and multiply a float by it; the earlier statement that it is used as an integer count of ticks is replaced. Other tick-count readers (locomotion and character code, where a cooldown of 60 ticks is one second) are not re-checked here and may convert it from the float.
- Tick-length word (data address 0x368FA8): 1/60 s, written at start-up. It is the time step of the per-second rates in gameplay code (2026-11-11, High: confirmed as the per-pass addend of the foliage grass phase clock and read by the effect draw routines). The effect spawner tick instead reads a boot data word at 0x2ECCC4 that also holds 1/60 (SPEC-0034, 3.2).
- Half-rate word (data address 0x368FA4): 30 divided by the frame rate, 0.5, written at start-up. It is listed here for completeness.
- Per-element step word (data address 0x3A1A74): 1/30 s. It is field +0x04 of the physics block, written once at start-up from fixed values (SPEC-0033, Physics block). The address lies past the end of the boot file image, so it is not on the disc. It is the time scale of the per-element movement step of SPEC-0040, Behavior 19 (Per-element movement step and ground snap).
- Game-time word (data address 0x368F94, frame object +0x94): float seconds, 0.0 at start, += dt at the end of every pass (see One pass of the main loop).
- Pass counter (data address 0x368F98, frame object +0x98): integer, 0 at start, += 1 at the end of every pass. Readers that need a float convert it.
- Up vector (data address 0x3932E0): the constant vector (0, 1, 0, 0). It is one of five constant 16-byte vectors at 0x3932C0 (0, 0, 0, 0), 0x3932D0 (1, 0, 0, 0), 0x3932E0 (0, 1, 0, 0), 0x3932F0 (0, 0, 1, 0) and 0x393300 (0, 0, 0, 1), written once at boot by the start-up table walk (SPEC-0026) and never changed. Every other access to the block is a read.
- Random generator state (data address 0x39BE50): the state object of the gameplay random generator (a Mersenne Twister with 624 entries; SPEC-0008). It is not a tuning value.

The clean side keeps the 60 Hz tick above and does not take a disc value for any of these run-time words.

### Packed time difference to ticks (Medium; field meanings open)

Some timers take the difference between two packed time values and convert it to ticks. Each packed time value is a 64-bit quantity made of two 32-bit fields: the low field (called the coarse field here) and the high field (called the fine field here). For an earlier value A and a later value B:

- coarse difference: Δc = coarse(B) − coarse(A), signed;
- fine difference: Δf = fine(B) − fine(A), signed;
- ticks = 60 × (60 × Δc + Δf).

The result is passed to a method of the timer object that asked for it. When the two coarse fields are equal the coarse term is zero, so the same formula covers that case. The conversion routine is never run: no call, jump, code-pointer word or built address in the boot program or the 173 level modules refers to it, so no timer ever receives these ticks and the units of the two fields never matter (Medium-High; a register-computed jump is not excluded in principle). The clean side need not implement it. If it is implemented anyway, it must not assume that the coarse field is seconds and the fine field is frames. Confidence: Medium for the arithmetic; Medium-High that it is unused.

### In-game calendar and clock

The game keeps one clock record in memory (data address 0x3866D0; the game object's +0x08 word points to it). It is created and cleared once at boot, by the start-up loaders, after the tick words above are set. The clean side should keep it as a plain record with the layout below. The record holds two play-time totals and a calendar.

**Never restored (High).** No save, load, new game, game over or quit path writes the clock record, and the save file does not hold it. The calendar and both play-time totals run on from boot through every load and new game.

Starting state (set once at boot; the only way the calendar gets a date):

- Year 2003, month index 0 (January), day of month 1, day count 1, hour 12, minute 0, seconds 0.0.
- Weekday Wednesday (index 3).
- Both play-time totals and their derived fields zero, and the tick counter zero.

**Step.** Each pass the calendar and play-time total A are advanced by one step of 4 × tick length = 4 × (1/60) = **1/15 s** of in-game time; play-time total B is advanced by the plain tick length, **1/60 s**. The clock therefore runs at four in-game seconds per real second: one in-game minute is 15 real seconds, one in-game hour is 15 real minutes, one in-game day is 6 real hours. The factor 4 is a fixed constant. The tick length it multiplies is the start-up word above, which is set once at start-up to 1/60 s (High). The clock set-up passes the tick length itself as the second step, so total B runs at the real-time rate (High). The calendar step is Medium only because of the factor 4.

**Per-pass order.** In each pass: (1) the tick counter += 1; (2) play-time total A += 1/15 and total B += 1/60, and their derived fields are recomputed; (3) the calendar advances by the step.

**When the clock runs.** The clock record pass runs on every pass while a world object exists, which is always the case after boot. It runs whatever the game-state code and the world-enable word: also under pause and the other freezing screens (game-state code 9), with the world disabled, and on the title screen and the front end before play. It does not run while a movie plays, because the movie blocks the whole loop. Play time and the calendar therefore include paused and menu time (High for the place in the pass; SPEC-0026, main loop step 8.2).

**Play-time totals.** Each total is a float in seconds. After adding a delta, let T be the whole part of the new total. Then days = floor(T / 86400), hours = floor(T / 3600), minutes = floor(T / 60), each computed from T (not from the previous remainder). These are stored as whole numbers. Total A therefore counts in-game time (four times real time) and total B counts real-time play (one second per 60 passes). Neither is read by anything found. Both are single-precision sums with truncating adds, so each stops growing once its step is smaller than the float spacing: total B (and the game-time word) stops at 262,144 s and total A at 1,048,576 s, both after about 16.7 million passes (about 77 hours of running; Medium).

**Advancing the calendar by a delta** (delta in seconds, may be fractional; also used by the hour and day helpers below):

1. Let m = floor(whole part of delta / 60). The seconds part is s = delta − 60 × m, which is in [0, 60) and keeps the fraction.
2. seconds += s. If seconds >= 60: seconds −= 60 and a minute carry of 1 is set.
3. minutes += (m mod 60) + minute carry. If minutes > 59: minutes −= 60 and an hour carry is set.
4. hours += (floor(m / 60) mod 24) + hour carry. If hours > 23: hours −= 24 and one day is added.
5. If the stored day count is already at least the year length (365, or 366 in a leap year), one more day is added to it and the year length is subtracted from the hours field. Then day count += floor(m / 1440) + the day carry from step 4. No path ever changes the year.
6. Recompute month, day of month and weekday from the year and the day count (see Date from day count below). The weekday call gets (year, day count).

Each carry is a single subtraction, not a loop. A delta that would overflow a field by more than one unit loses the excess (see Edge cases).

**Date from day count.** Month lengths are 31, 28 (29 in a leap year), 31, 30, 31, 30, 31, 31, 30, 31, 30, 31 for January to December. The routine subtracts month lengths in order from January while the count is greater than the month's length; the remainder is the day of month, and the stopping month is the month index. The day count is 1-based (January 1 is 1). The month index found by the walk then passes through a month map that keeps 0 to 11 and turns any other value into 0.
- **Range test (High).** The lookup fails when the day count is 367 or more, in any year, and also when it is exactly 366 in a leap year. In a non-leap year a day count of 366 is accepted: the walk runs past December into a thirteenth month of 30 days, leaving day of month 1 and month index 12, which the month map turns into 0, so it reads as January 1. A leap-year day count of 365 gives December 30, and December 31 is never shown in a leap year. On failure, the lookup leaves the day-of-month field unchanged and the caller sets the month index to 0.
- **Failure (High).** On a failure the month index is set to 0 (January) and only the day of month keeps its value; the weekday is still recomputed.

**Year turn (High).** From the default start (year 2003, not a leap year), the rollover into December 31 sets day count 365, and December 31 shows for one pass. On the next pass the step-5 test fires (365 is at least 365): the day count becomes 366, the hours fall by 365, and the date lookup accepts 366 as month 0, day of month 1 (January 1); the weekday is recomputed. On the pass after, the day count is 367 and the lookup fails. From then on every pass adds one day and subtracts 365 from the hours, the lookup keeps failing, the month stays 0 and the day of month stays 1, and the hour field never returns to 0 to 23. Every "hour in band" time-of-day test is then false, and every "before H:M" test is true. Reaching December 31 takes 363.5 in-game days from January 1 12:00, which is about 471.1 million passes (471,096,000 in exact arithmetic, a few thousand more with the truncating adds), about 90.9 days of uninterrupted running.

**Leap years and year length.** A year is a leap year if it is divisible by 4 and either not divisible by 100 or divisible by 400. Year length is 366 in a leap year, otherwise 365. For example 2000 and 2004 are leap, 1900 and 2100 are not.

**Weekday.** Sunday is 0. With Y the year and D the day count as passed in, the weekday is (floor((Y − 2000) × 365.25 + D) + 6) mod 7, computed in single precision. Valid for Y >= 2000. The calendar passes the 1-based day count, so January 1 of a year uses D = 1. Under that convention January 1 2003 is Wednesday (correct), but January 1 2004 is Friday (the real day was Thursday). The clean side should use the same convention if it wants the original's weekday display; this is an original quirk, not an error to fix.

**Setting a full date** (used when a caller supplies every field). Each field is first normalised: seconds >= 60 becomes 0; minutes > 59 becomes 0; hours > 23 becomes 0; month > 11 becomes 0; day of month > 31 becomes 0. Nothing carries and the day is not checked against the month length. The year is not checked. Then the day count = day of month + the lengths of all months before the given month (leap-aware), and the weekday is computed as above.

**Hour and day helpers.** Advancing by N hours calls the advance with N × 3600 seconds. Advancing by N days calls it with N × 86400 seconds. Both are whole numbers in seconds, so they go through the same carries.

**Seconds to ticks and calendar steps.** Seconds to game ticks: multiply by 60 and keep the whole part (2 seconds is 120 ticks). Seconds to calendar steps: the whole part of (whole seconds ÷ step) in single precision. The stored step is slightly above 1/15, so the quotient falls just short of the exact multiple: 2 seconds gives 29 steps, not 30 (the quotient is 29.999998). No caller of this conversion was found.

**Who reads the clock.** Only the readers listed here were found:

- Hour (record offset 0x2C) and minute (record offset 0x28) are read by time-of-day tests. The hour-only tests are strict: "hour > 5 and hour < 12" is true for 6 to 11; "12 < hour < 17" for 13 to 16; "17 < hour < 21" for 18 to 20; "5 < hour < 21" for 6 to 20; "21 < hour < 5" is never true. "Before H:M" is true when hour < H, or hour = H and minute < M. "After H:M" is true when hour > H, or hour = H and minute > M. Two getters return the hour and the minute, and one helper advances the calendar by a given delta. None of these routines is reached on this build: no call, no stored code pointer and no entry of the 406-entry script command table (the by-name interface that the level modules call) leads to the five hour tests, the two band tests, the two getters or the delta helper, and no level module refers to them (High). They are reference behaviour only.
- The HUD reader and the save-file date reader were not found reading these calendar fields. The save-file date is written from the console clock record instead (see SPEC-0008 and SPEC-0025).
- Events and scripts: nothing in play reads the calendar fields. The script command table offers only GetSystemTime (game time, data address 0x368F94) and GetTimeStep (the tick-length word) as clock reads, so a level script can read game time but never the in-game hour, minute or date (High). The calendar runs but has no effect on gameplay.

### Clock record layout

Offsets are from the start of the clock record. Every field is 4 bytes.

| Offset | Meaning | Type | Range |
| --- | --- | --- | --- |
| 0x00 | Tick counter | int | 0 and up |
| 0x04 | Play-time total A, seconds | float | 0 and up |
| 0x08 | Play-time A, days | int | 0 and up |
| 0x0C | Play-time A, hours | int | 0 and up |
| 0x10 | Play-time A, minutes | int | 0 and up |
| 0x14 | Play-time total B, seconds | float | 0 and up (real-time rate) |
| 0x18 | Play-time B, days | int | 0 and up |
| 0x1C | Play-time B, hours | int | 0 and up |
| 0x20 | Play-time B, minutes | int | 0 and up |
| 0x24 | Calendar seconds | float | 0 to < 60 |
| 0x28 | Calendar minutes | int | 0 to 59 |
| 0x2C | Calendar hours | int | 0 to 23; negative and falling after the year turn |
| 0x30 | Month index | int | 0 to 11 (0 = January) |
| 0x34 | Weekday | int | 0 to 6 (0 = Sunday) |
| 0x38 | Day of month | int | 1 to 31 |
| 0x3C | Day count in year | int | 1 to year length; past it, rising by 1 per pass, after the year turn |
| 0x40 | Year | int | 2003 at start |
| 0x44 | Step per pass for the calendar and total A, seconds | float | 1/15 |
| 0x48 | Step per pass for total B, seconds | float | 1/60 |

## Edge cases and error handling

- A few slow paths (screen fades, loading) wait for extra vertical blanks outside the normal pace point. During those, no gameplay updates happen. The logic step stays 1/60 s per pass.
- The PAL row (50/25 Hz) is unreachable on this build. Don't use it. Reason: the row comes from the video-mode test (always NTSC) and a mode argument that is always 0.
- The single-precision game-time float loses resolution over very long sessions: at about 2^17 s (36 hours), 1/60 s is near the float's spacing. Systems that use game time as an absolute clock may see jitter after many hours. Using f64 or the integer pass counter on the clean side is acceptable and avoids this.
- Run-time per-tick words: the clean side must not use a value from the disc for them, and must not assume a value other than the one the game sets (60 for the frame-rate word, 1/60 s for the tick-length word). The clock words start at zero, and the up vector and generator state are set at start-up (see Per-tick words read by gameplay code).
- Calendar, weekday convention: the calendar passes a 1-based day count to the weekday formula. Jan 1 2004 therefore shows Friday, one day ahead of the real calendar. This is kept as original behavior.
- Calendar, field overflow: each field takes at most one carry per advance. If a field would overflow by more than one unit (for example minutes at 59 plus a delta of 2 hours is handled by the split in the advance steps, but a field whose sum exceeds twice its range loses the excess), the result is not the true sum. Large advances (hours or days helpers) pass through the same carries; the clean side should split very large advances into whole-minute steps if it wants exact results.
- Calendar, second play-time channel (total B): its step is the plain tick length (1/60 s), so it counts real-time play at one second per 60 passes, a quarter of the speed of total A. Nothing reads it.
- Calendar, lookup failure: a day count of 367 or more (or 366 in a leap year) makes the date lookup fail. The month index is set to 0 (January), the day of month keeps its last value, and the weekday is still recomputed. A day count of 366 in a non-leap year does not fail: it reads as January 1.
- Calendar, year turn: the year never advances. After December 31 in the default non-leap year, the month becomes January with day of month 1 and stays so; the hour field then becomes negative and continues falling. Reproduce it only if original-like behaviour past 363.5 in-game days is wanted.
- Calendar, date setter: an invalid day for the month (for example February 30) is stored without a check, and the day count then runs into the next month.

## Test vectors

- 60 passes → game time 0.99999928 s (12 float steps below 1.0, from the truncating adds), pass counter 60.
- 3,600 passes → 59.999142 s (the accumulated f32 error stays below 0.001 s), counter 3,600.
- A "hold for 30 frames" check fires on the 30th consecutive pass with the button down, which is 0.5 s of game time.
- step30 = 30 × dt = 0.5. A quantity tuned "per 30 Hz frame" advances by half that amount per pass.
- The boot data value 60.0 (at 0x30C598) gives dt = 1/60 s, the same as the rate above.
- Weekday, using the day count as passed: weekday(2000, 0) = 6 (Saturday); weekday(2001, 0) = 0; weekday(2004, 0) = 4; weekday(2003, 1) = 3 (Wednesday, the default start); weekday(2004, 1) = 5 (Friday, the quirk above).
- Leap and length: 2000 leap, 2004 leap, 1900 and 2100 not leap; year length of 2003 is 365 and of 2004 is 366.
- Default start: 2003 January 1, 12:00:00.0, Wednesday, day count 1.
- Step: because of the truncating adds, 900 passes from the start give 12:00:59.9993, and the minute turns on pass 901 (12:01:00.066). About 15 passes make 1 in-game second. 648,000 passes (12 in-game hours) give 23:59:59.51 on January 1; the date turns to January 2 2003 at 00:00:00.04 (day count 2, weekday Thursday) on pass 648,008.
- Date normaliser: seconds 60.0 becomes 0; month 12 becomes 0; day 32 becomes 0; hours 24 becomes 0.
- Play-time totals: total 90,000 s gives days 1, hours 25, minutes 1,500. After 3,600 passes total B reads 59.999142 s (the same sum as the game-time word) and total A reads 239.9966 s.
- Time-of-day tests: hour 6 passes "5 < hour < 12"; hour 5 fails it; hour 20 passes "5 < hour < 21"; hour 21 fails it; no hour passes "21 < hour < 5".
- Year turn, step 1: record December 30 2003, 23:59:59.95 (day count 364), advanced by 1/15 s: seconds about 0.0167, minutes 0, hours 0, day count 365, month 11, day of month 31, weekday 3 (Wednesday).
- Year turn, step 2 (the next advance): day count 366, hours -365, month 0, day of month 1 (January 1, accepted by the lookup), weekday (trunc(3 × 365.25 + 366) + 6) mod 7 = 1467 mod 7 = 4.
- Year turn, step 3 (the advance after): day count 367, hours -730, month 0, day of month 1 (kept after the failed lookup), weekday 5. Hour -730 fails every "hour in band" test and passes "before 6:00".
- Year turn, reach: from the default start, December 31 2003 00:00 comes after 363.5 × 86,400 × 15 = 471,096,000 passes in exact arithmetic (7,851,600 s, about 90.9 days of running); the truncating adds add a few thousand passes.
- Date lookup: (2003, 365) gives December 31; (2003, 366) gives January 1 (month 0, day 1); (2003, 367) fails and leaves the prior day; (2004, 365) gives December 30; (2004, 366) fails and leaves the prior day; (2004, 367) fails; (2004, 60) gives February 29.
- Load: a save loaded after 2 hours of play leaves the calendar and both play-time totals unchanged; they keep running from their values at the moment of the load.
- Script clock reads (synthetic): a level script that calls GetSystemTime on pass 600 receives the game-time word of that pass (600 additions of 1/60 s, a little below 10.0 s after the truncating adds), and GetTimeStep returns the tick-length word, 1/60 s. No script call returns the in-game hour, so running the calendar to 18:00 or to 06:00 changes nothing in any level.

- Packed time difference, synthetic: Δc = 2 and Δf = 5 give 60 × 125 = 7,500 ticks. Δc = 1 and Δf = −1 give 60 × 59 = 3,540 ticks. Δc = 0 and Δf = 3 give 180 ticks.

## Open questions

Numbers of removed questions are not reused.

- Resolved: Calendar year turn. Every advance whose stored day count is already at least the year length adds one more day and subtracts the year length from the hours; no path changes the year. In the default non-leap year, December 31 shows for one pass, then the day count rises by 1 and the hours fall by 365 on every pass, with month 0 and day of month 1 (Date from day count, Year turn; High). It needs about 90.9 days of uninterrupted running from the default start.
- Resolved: Calendar weekday call. The date recompute passes (year, day count) to the weekday formula (High).
- Resolved: calendar second play-time channel. The word at 0x48 is the plain tick length (1/60 s), so total B counts real-time play at a quarter of total A's speed; nothing reads it (Edge cases; High). An earlier reading that it held an address and stayed at zero was wrong.
- Resolved: save-file date text comes from the console real-time clock record, not the in-game calendar (SPEC-0025, Location, date and progress; Medium-High).
- Resolved: Script callers of the time-of-day tests and the getters. There are none: the tests, the getters and the delta helper have no caller and are not in the script command table; scripts read only game time and the tick length (Who reads the clock; High).
- Resolved: the day-count, weekday and leap rules are complete for years at and after 2000. The year starts at 2003 and never changes, and no script reads the calendar, so a year before 2000 cannot occur in play; the formula gives negative results there and the clean side need not support it (High).
- Resolved: two calendar helpers (one that writes a 5.0 timer record, and one that calls a routine outside the calendar) are not calendar and are not in this spec; the clean side leaves them out.

- Resolved in part (Medium): extra vertical-blank waits come from three kinds of place. (1) The start-up hold of 180 blanks (3 s at 60 per second) before the front end. (2) Single blank waits, each followed by a display-buffer flush, in the world load, world unload and world swap steps (SPEC-0026), plus the resident-world post-load step. (3) Two blank waits per queue-drain step of the movie player, repeated while its output queue is not empty. No gameplay tick runs during any of them; each wait ends on the next blank, so a step costs one or two blanks plus any disc time. The number of blanks a whole load spans depends on disc read speed (needs a runtime check). Default if unresolved (rewrite choice): a load takes as many ticks as it takes, with no gameplay update, and the start-up hold is 180 ticks.
- Resolved: two more values in the rate table (1.5 and 1.0) are never read by the game; their purpose is developer intent and not encoded anywhere.
- Frame-rate word integer use (2026-11-11). The older text said other code reads the word as an integer tick count, while the foliage and Cloud Beast code read it as a float 60.0. The two readings cannot both describe one stored value; which other routines use it, and how, is not established. Default if unresolved (rewrite choice): treat it as the float 60.0 everywhere. [Medium]
- Whether the controller hardware delivers new pad data more often than once per field does not matter: the game reads only the latest snapshot once per pass.
- Rate words. Resolved: written once at start-up from the boot table entry at data address 0x30C598 (60.0, row 0, column 0), giving 60, 1/60 s and 0.5 (High).
- Packed time difference: Resolved (Medium-High). The conversion routine is never run, so there is no owner and no unit to establish; the clean side leaves it out (Packed time difference to ticks).
- Up vector and generator state. Resolved: 0x3932E0 is the constant up vector (0, 1, 0, 0), written once at boot; 0x39BE50 is the random generator state (SPEC-0008), not a tuning value.
