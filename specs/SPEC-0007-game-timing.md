# SPEC-0007: Game-logic timing

- Answers: none (includes the in-game calendar section)
- Status: Draft
- Disc build: SCUS-97367 v1.00 (same fingerprint as SPEC-0001)
- Confidence: High for the fixed step, pacing and input order. Medium for the slow-path extra waits (see Open questions). The start-up words that gameplay code reads at run time (frame-rate word 60, tick-length word 1/60 s, half-rate word 0.5) are written at start-up from the boot rate table (High for the values; High for the start-up timing of the write, see Rate selection). They are not disc data. The per-element step word (1/30 s) is field +0x04 of the physics block, written once at start-up from fixed values (High; SPEC-0033, Physics block). The pass counter and game-time word start at zero and advance at the end of every pass (High). The up vector and the random generator state are set at start-up and are not tuning values (High). Packed time difference to ticks: Medium for the arithmetic, the field meanings are open. Calendar: High for the day-count, leap-year, weekday and date-normaliser arithmetic; Medium for the clock step factor (4 times the tick) and the clock record layout; High for the year turn (the calendar breaks after December 31) and for the boot-only reset of the clock.

## Summary

Gameplay runs on a **fixed time step of exactly 1/60 second**, one update per pass of the main loop, and one pass per video vertical blank. Elapsed time is never measured. The original does not clamp, accumulate or catch up: if a pass takes longer than one field, the game simply runs slower, and the logic still advances by 1/60 s per pass. Controller input is sampled once per pass, before anything else updates. The clean side can drive the simulation with a fixed 60 Hz tick that consumes one input snapshot per tick. The same pass also advances the in-game calendar and play-time totals, which run at four in-game seconds per real second; see "In-game calendar and clock".

## Where it lives on the disc

Nothing needs to be read from the disc. The values below are constants. They come from a small rate table in the boot executable, and on this build it always selects the same row and column.

For reference, the boot data also holds the 32-bit float 60.0 at data address 0x30C598 (verified in the boot file image). It agrees with the rate above. The clean side can use the constant 60.0 directly and does not need to read it from the disc.

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

These derived values are written at start-up to run-time words that gameplay code reads (see "Per-tick words read by gameplay code"): the rate goes to a frame-rate word (60, ticks per second, also used as an integer count of ticks: 60 means one second), dt goes to a tick-length word (1/60 s), and step30 goes to a half-rate word (0.5). The words are zero-filled in the file image (beyond its end), so they are not disc data; the start-up write fills them from the table entry above. The clean side takes the values 60, 1/60 and 0.5 from this spec. The write is done once, by the start-up frame setup, into the frame object at data address 0x368F00: the rate goes to its +0xA0 field (data address 0x368FA0), dt to +0xA8 and the half-rate to +0xA4.

### One pass of the main loop

In this order:

1. **Input**: snapshot every controller's latest state once. All gameplay in the same tick sees that snapshot. Button-combo and hold detectors count passes, so "N frames" means N passes.
2. **Update**: all game systems update once with dt = 1/60 s. There is no inner loop running several logic steps per pass, and there is no frame skip.
3. **Render**: submit the frame.
4. **Pace**: wait until at least one vertical-blank start has occurred since the previous pace point, then reset the vblank counters to zero. Any extra vblanks that passed during an overlong pass are discarded. The two counters (blank starts and blank ends) are counted by the vertical-blank interrupt handlers and are also cleared once at boot, just before the first pass, after a field-sync wait (SPEC-0026, Boot order), so the first pass is paced from that point (High).
5. **Clock**: game time (a float, in seconds, data address 0x368F94) += dt, and the pass counter (an integer, data address 0x368F98) += 1. Both start at zero: the counter is 0 and game time is 0.0 before the first pass. This step is the last thing in every pass, so during pass n (counting from 0) the counter reads n and game time reads n × dt.

The full order of the game systems inside one pass, the gates of the world update and what keeps running while a screen freezes the world are in SPEC-0026 (The main loop, one pass; Pause and freezing screens). A movie plays inside the pass and blocks it until the movie ends, so game time, the pass counter and the calendar do not advance while a movie plays (High).

### Math

- After *n* passes, game time = n / 60 s up to single-precision float accumulation: the original adds the 32-bit float 1/60 (0.016666668) each pass with the console's truncating add (SPEC-0008, Platform arithmetic), so the sum stays a little below n / 60 (0.99999928 after 60 passes). A timer that compares this kind of sum with a time T in seconds (a multiple of 0.05 s) first passes it on tick 60T + 1 (SPEC-0008, Seconds to ticks).
- Real NTSC field rate is 59.94 Hz, but the engine assumes 60.0. Measured against a wall clock, game time therefore runs about 0.1 % fast. The clean side does not need to reproduce that: running the tick at exactly 60 Hz gives the same per-tick behavior.
- When a pass overruns (takes longer than one field), the original runs slower than real time. A modern fixed-step loop that catches up with several ticks per frame would behave differently in slow-down. For original-like feel under load, cap catch-up at one tick per displayed frame, or make it a user option.

### Per-tick words read by gameplay code

Some gameplay routines read run-time words that are not on disc. Several of them lie beyond the end of the boot file image, in the zero-filled part of the loaded segment, and the game sets them while it runs. The roles of the first three words below are resolved by the start-up writes in "Rate selection", the per-element step word (0x3A1A74) follows, and the clock words, the up vector and the random generator state are listed last:

- Frame-rate word (data address 0x368FA0): 60, written at start-up from the boot rate table (not disc data). It is the tick count per second, used as an integer count of ticks (the locomotion and character code use it as a tick count, for example a cooldown of 60 ticks is one second).
- Tick-length word (data address 0x368FA8): 1/60 s, written at start-up. It is the time step of the per-second rates in gameplay code.
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

The result is passed to a method of the timer object that asked for it. When the two coarse fields are equal the coarse term is zero, so the same formula covers that case. The unit of each field is not identified, so the clean side must not assume that the coarse field is seconds and the fine field is frames. Confidence: Medium for the arithmetic; Low for what the two fields mean.

### In-game calendar and clock

The game keeps one clock record in memory (data address 0x3866D0; the game object's +0x08 word points to it). It is created and cleared once at boot, by the start-up loaders, after the tick words above are set. The clean side should keep it as a plain record with the layout below. The record holds two play-time totals and a calendar.

**Never restored (High).** No save, load, new game, game over or quit path writes the clock record, and the save file does not hold it. The calendar and both play-time totals run on from boot through every load and new game.

Starting state (set once at boot; the only way the calendar gets a date):

- Year 2003, month index 0 (January), day of month 1, day count 1, hour 12, minute 0, seconds 0.0.
- Weekday Wednesday (index 3).
- Both play-time totals and their derived fields zero, and the tick counter zero.

**Step.** Each pass the clock is advanced by one step of 4 × tick length = 4 × (1/60) = **1/15 s** of in-game time. The clock therefore runs at four in-game seconds per real second: one in-game minute is 15 real seconds, one in-game hour is 15 real minutes, one in-game day is 6 real hours. The factor 4 is a fixed constant. The tick length it multiplies is the start-up word above, which is set once at start-up to 1/60 s (High). The calendar step is Medium only because of the factor 4.

**Per-pass order.** In each pass: (1) the tick counter += 1; (2) play-time total A and total B each += the step, and their derived fields are recomputed; (3) the calendar advances by the step.

**When the clock runs.** The clock record pass runs on every pass while a world object exists, which is always the case after boot. It runs whatever the game-state code and the world-enable word: also under pause and the other freezing screens (game-state code 9), with the world disabled, and on the title screen and the front end before play. It does not run while a movie plays, because the movie blocks the whole loop. Play time and the calendar therefore include paused and menu time (High for the place in the pass; SPEC-0026, main loop step 8.2).

**Play-time totals.** Each total is a float in seconds. After adding a delta, let T be the whole part of the new total. Then days = floor(T / 86400), hours = floor(T / 3600), minutes = floor(T / 60), each computed from T (not from the previous remainder). These are stored as whole numbers. Total B is fed a value that is effectively zero (see Edge cases), so it stays at zero in practice.

**Advancing the calendar by a delta** (delta in seconds, may be fractional; also used by the hour and day helpers below):

1. Let m = floor(whole part of delta / 60). The seconds part is s = delta − 60 × m, which is in [0, 60) and keeps the fraction.
2. seconds += s. If seconds >= 60: seconds −= 60 and a minute carry of 1 is set.
3. minutes += (m mod 60) + minute carry. If minutes > 59: minutes −= 60 and an hour carry is set.
4. hours += (floor(m / 60) mod 24) + hour carry. If hours > 23: hours −= 24 and one day is added.
5. If the stored day count is already at least the year length (365, or 366 in a leap year), one more day is added to it and the year length is subtracted from the hours field. Then day count += floor(m / 1440) + the day carry from step 4. No path ever changes the year.
6. Recompute month, day of month and weekday from the year and the day count (see Date from day count below). The weekday call gets (year, day count).

Each carry is a single subtraction, not a loop. A delta that would overflow a field by more than one unit loses the excess (see Edge cases).

**Date from day count.** Month lengths are 31, 28 (29 in a leap year), 31, 30, 31, 30, 31, 31, 30, 31, 30, 31 for January to December. The routine subtracts month lengths in order from January while the count is greater than the month's length; the remainder is the day of month, and the stopping month is the month index. The day count is 1-based (January 1 is 1). The month index found by the walk then passes through a month map that keeps 0 to 11 and turns any other value into 0.
- **Range test (High).** The lookup fails when the day count is greater than the year's length: 366 or more in a non-leap year, and 367 or more in a leap year. A leap-year day count of 366 is accepted as December 31. On failure, the lookup leaves the day-of-month field unchanged and the caller sets the month index to 0.
- **Failure (High).** On a failure the month index is set to 0 (January) and only the day of month keeps its value; the weekday is still recomputed.

**Year turn (High).** From the default start (year 2003, not a leap year), the rollover into December 31 sets day count 365, and December 31 shows for one pass. On the next pass the step-5 test fires (365 is at least 365): the day count becomes 366, the hours fall by 365, and the date lookup fails. The month is set to 0 and the day of month stays 31; the weekday is recomputed. From then on, every pass adds one day and subtracts 365 from the hours. The lookup continues to fail, so the month stays 0 and the day of month stays 31, and the hour field never returns to 0 to 23. Every "hour in band" time-of-day test is then false, and every "before H:M" test is true. Reaching December 31 takes 363.5 in-game days from January 1 12:00, which is 471,096,000 passes, about 90.9 days of uninterrupted running.

**Leap years and year length.** A year is a leap year if it is divisible by 4 and either not divisible by 100 or divisible by 400. Year length is 366 in a leap year, otherwise 365. For example 2000 and 2004 are leap, 1900 and 2100 are not.

**Weekday.** Sunday is 0. With Y the year and D the day count as passed in, the weekday is (floor((Y − 2000) × 365.25 + D) + 6) mod 7, computed in single precision. Valid for Y >= 2000. The calendar passes the 1-based day count, so January 1 of a year uses D = 1. Under that convention January 1 2003 is Wednesday (correct), but January 1 2004 is Friday (the real day was Thursday). The clean side should use the same convention if it wants the original's weekday display; this is an original quirk, not an error to fix.

**Setting a full date** (used when a caller supplies every field). Each field is first normalised: seconds >= 60 becomes 0; minutes > 59 becomes 0; hours > 23 becomes 0; month > 11 becomes 0; day of month > 31 becomes 0. Nothing carries and the day is not checked against the month length. The year is not checked. Then the day count = day of month + the lengths of all months before the given month (leap-aware), and the weekday is computed as above.

**Hour and day helpers.** Advancing by N hours calls the advance with N × 3600 seconds. Advancing by N days calls it with N × 86400 seconds. Both are whole numbers in seconds, so they go through the same carries.

**Seconds to ticks and calendar steps.** Seconds to game ticks: multiply by 60 and keep the whole part (2 seconds is 120 ticks). Seconds to calendar steps: the whole part of (seconds ÷ step), which is 15 × seconds for the 1/15 s calendar step above (2 seconds is 30 steps).

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
| 0x14 | Play-time total B, seconds | float | effectively 0 (see Edge cases) |
| 0x18 | Play-time B, days | int | 0 |
| 0x1C | Play-time B, hours | int | 0 |
| 0x20 | Play-time B, minutes | int | 0 |
| 0x24 | Calendar seconds | float | 0 to < 60 |
| 0x28 | Calendar minutes | int | 0 to 59 |
| 0x2C | Calendar hours | int | 0 to 23; negative and falling after the year turn |
| 0x30 | Month index | int | 0 to 11 (0 = January) |
| 0x34 | Weekday | int | 0 to 6 (0 = Sunday) |
| 0x38 | Day of month | int | 1 to 31 |
| 0x3C | Day count in year | int | 1 to year length; past it, rising by 1 per pass, after the year turn |
| 0x40 | Year | int | 2003 at start |
| 0x44 | Step per pass, seconds | float | 1/15 |
| 0x48 | Second-channel rate word | 4 bytes | see Edge cases |

## Edge cases and error handling

- A few slow paths (screen fades, loading) wait for extra vertical blanks outside the normal pace point. During those, no gameplay updates happen. The logic step stays 1/60 s per pass.
- The PAL row (50/25 Hz) is unreachable on this build. Don't use it. Reason: the row comes from the video-mode test (always NTSC) and a mode argument that is always 0.
- The single-precision game-time float loses resolution over very long sessions: at about 2^17 s (36 hours), 1/60 s is near the float's spacing. Systems that use game time as an absolute clock may see jitter after many hours. Using f64 or the integer pass counter on the clean side is acceptable and avoids this.
- Run-time per-tick words: the clean side must not use a value from the disc for them, and must not assume a value other than the one the game sets (60 for the frame-rate word, 1/60 s for the tick-length word). The clock words start at zero, and the up vector and generator state are set at start-up (see Per-tick words read by gameplay code).
- Calendar, weekday convention: the calendar passes a 1-based day count to the weekday formula. Jan 1 2004 therefore shows Friday, one day ahead of the real calendar. This is kept as original behavior.
- Calendar, field overflow: each field takes at most one carry per advance. If a field would overflow by more than one unit (for example minutes at 59 plus a delta of 2 hours is handled by the split in the advance steps, but a field whose sum exceeds twice its range loses the excess), the result is not the true sum. Large advances (hours or days helpers) pass through the same carries; the clean side should split very large advances into whole-minute steps if it wants exact results.
- Calendar, second play-time channel: its step is a 32-bit word that holds the clock record's own address, not a time, so the channel adds about 5 × 10^-39 per pass and stays at zero. Keep it at zero; do not treat it as a second clock.
- Calendar, lookup failure: a day count greater than the year length makes the date lookup fail. The month index is set to 0 (January), the day of month keeps its last value, and the weekday is still recomputed. A day count of 366 fails in a non-leap year and succeeds as December 31 in a leap year.
- Calendar, year turn: the year never advances. After December 31 in the default non-leap year, the month becomes January while the day of month stays 31; the hour field then becomes negative and continues falling. Reproduce it only if original-like behaviour past 363.5 in-game days is wanted.
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
- Step: 900 passes (60 s of in-game time) take the start to 12:01:00.0. 15 passes make 1 in-game second. 648,000 passes (12 in-game hours) from the start give January 2 2003 at 00:00:00.0, day count 2, weekday Thursday.
- Date normaliser: seconds 60.0 becomes 0; month 12 becomes 0; day 32 becomes 0; hours 24 becomes 0.
- Play-time totals: total 90,000 s gives days 1, hours 25, minutes 1,500.
- Time-of-day tests: hour 6 passes "5 < hour < 12"; hour 5 fails it; hour 20 passes "5 < hour < 21"; hour 21 fails it; no hour passes "21 < hour < 5".
- Year turn, step 1: record December 30 2003, 23:59:59.95 (day count 364), advanced by 1/15 s: seconds about 0.0167, minutes 0, hours 0, day count 365, month 11, day of month 31, weekday 3 (Wednesday).
- Year turn, step 2 (the next advance): day count 366, hours -365, month 0, day of month 31 (kept), weekday (trunc(3 × 365.25 + 366) + 6) mod 7 = 1467 mod 7 = 4.
- Year turn, step 3 (the advance after): day count 367, hours -730, month 0, day of month 31 (kept), weekday 5. Hour -730 fails every "hour in band" test and passes "before 6:00".
- Year turn, reach: from the default start, December 31 2003 00:00 comes after 363.5 × 86,400 × 15 = 471,096,000 passes (7,851,600 s, about 90.9 days of running).
- Date lookup: (2003, 365) gives December 31; (2003, 366) fails and leaves day 31; (2003, 367) fails and leaves the prior day; (2004, 365) gives December 30; (2004, 366) gives December 31; (2004, 367) fails; (2004, 60) gives February 29.
- Load: a save loaded after 2 hours of play leaves the calendar and both play-time totals unchanged; they keep running from their values at the moment of the load.
- Script clock reads (synthetic): a level script that calls GetSystemTime on pass 600 receives the game-time word of that pass (600 additions of 1/60 s, a little below 10.0 s after the truncating adds), and GetTimeStep returns the tick-length word, 1/60 s. No script call returns the in-game hour, so running the calendar to 18:00 or to 06:00 changes nothing in any level.

- Packed time difference, synthetic: Δc = 2 and Δf = 5 give 60 × 125 = 7,500 ticks. Δc = 1 and Δf = −1 give 60 × 59 = 3,540 ticks. Δc = 0 and Δf = 3 give 180 ticks.

## Open questions

Numbers of removed questions are not reused.

- Resolved: Calendar year turn. Every advance whose stored day count is already at least the year length adds one more day and subtracts the year length from the hours; no path changes the year. In the default non-leap year, December 31 shows for one pass, then the day count rises by 1 and the hours fall by 365 on every pass, with month 0 and day of month 31 (Date from day count, Year turn; High). It needs about 90.9 days of uninterrupted running from the default start.
- Resolved: Calendar weekday call. The date recompute passes (year, day count) to the weekday formula (High).
- Calendar second play-time channel: whether the record's word at 0x48 is meant to be a rate; the value that reaches it is the record's own address (see Edge cases).
- Resolved: save-file date text comes from the console real-time clock record, not the in-game calendar (SPEC-0025, Location, date and progress; Medium-High).
- Resolved: Script callers of the time-of-day tests and the getters. There are none: the tests, the getters and the delta helper have no caller and are not in the script command table; scripts read only game time and the tick length (Who reads the clock; High).
- The day-count, weekday and leap rules are complete for years at and after 2000. Behavior before 2000 is unknown; the formula gives negative results there.
- Two calendar helpers (one that writes a 5.0 timer record, and one that calls a routine whose role is not established) are not calendar and are not in this spec.

- Exactly which fades and loading screens insert extra vblank waits, and how many. Per-feature specs will state this where it matters.
- Two more values in the rate table (1.5 and 1.0) are not used by rate selection. Their purpose is unknown.
- Whether the controller hardware delivers new pad data more often than once per field does not matter: the game reads only the latest snapshot once per pass.
- Rate words. Resolved: written once at start-up from the boot table entry at data address 0x30C598 (60.0, row 0, column 0), giving 60, 1/60 s and 0.5 (High).
- Packed time difference (Medium): what the coarse and fine fields measure. The factor 60 on the coarse term is a fixed constant. If the coarse unit is a second and the fine unit is a frame, the result would differ from the formula above, so the unit must be confirmed before the clean side uses this conversion. The owner of the timer method that receives the ticks is not identified.
- Up vector and generator state. Resolved: 0x3932E0 is the constant up vector (0, 1, 0, 0), written once at boot; 0x39BE50 is the random generator state (SPEC-0008), not a tuning value.
