# SPEC-0016: Boss behavior: Plague Serpent

- Answers: proactive, none requested (boss AI); R-0096
- Status: Draft
- Disc build: SCUS-97367 v1.00 (same fingerprint as SPEC-0001)
- Confidence: per section. Tunable defaults and addresses: High (taken from the boot program image). Health phases and the bite/belch engagement rule: High. The engagement gates (height above 115.0, distance below 20.0, the suppression by anchor slot, and the no-op while biting): High for the comparisons, High for the distance metric (horizontal, from the shared helper; Open question 16). The hero action code 30 check: High for the comparison; the code is the hero's own state number, so 30 is the ledge hang of SPEC-0033 (Medium-High; Open question 10). Anchors and relocation: High for the rules and for the arrival placement (point A or C at the probed ground height). Phase change: High for the two-pass order, the 0.99 facing test, the 0.1 turn blend, the target (point B of the stored slot), the slot writes at the start of a change, animation 21 and the phase increment; High for the starting slot values (none); High for setting the matching level-state flag and its swamp break-scene response (see Arena break-scene response); Medium-High that the soft-lock is not reachable in normal play (Open question 13). Bite, belch, mesmerize and stun sub-states: High for transitions, Medium for effect payloads. Stun strike damage: High for the shape, Medium for "normal attack damage". Poison cloud: High for the movement, Medium for the damage meaning (the damage itself is the standard poison status). Tail slap: the routine is fully described (High for its rules); it is unreachable in the known routines (Medium-High, see Open questions). Lunge weapon, window, raw damage and block: High. Neck look (6A): High for the rule, Medium-High overall; whether the neck visibly turns in play is open. Sounds and effects (11B): High for the cue list, the trigger and the effect ids; Medium for the exact attach points of effect 413 and the bite flash.

## Summary

The Plague Serpent is a boss that sits on one of eight arena anchors. Each anchor is a set of three level points. Each tick it re-checks which anchor is nearest the hero and relocates there when that changes. When the hero is close and the serpent is not suppressed, it lunges with a bite, sometimes preceded by a mesmerize set-up that stuns the hero. When the hero is not in bite range, it belches a poison cloud on a random 5 to 7 second cooldown. Health below 75, 50 and 25 percent advances a phase counter from 1 to 4, and each advance plays a turn-and-roar phase change that faces two stored anchors in turn. The stage value is always 0 in this build, so only the stage-0 values apply to the bite decision and the stun. No known routine enters the tail-slap or wait state, and they are not to be made reachable.

## Where it lives on the disc

- The tuning values are 4-byte words in the boot program image, at the virtual addresses in the Layout tables. The clean side reads them through the ELF program headers of the boot program (SCUS_973.67). Integers are signed 32-bit, and "float" means IEEE single. Use the disc values as defaults. A runtime tool may overwrite them, so read them at start-up.
- The serpent's block is 38 words from 0x2DAFF8 to 0x2DB08C, plus two internal constants at 0x2DB090 and 0x2DB094. The Spyder Nest block (SPEC-0017) is at 0x2DB0A0 to 0x2DB0B4 and is not part of this boss.
- Arena anchors are level objects, looked up by a 16-bit object id in the current level (see Layout).
- Action-frame (attack-active) queries. Every non-hero boss is an AI-controlled character (only three character method tables exist: base, hero and AI-controlled character). When a boss asks the action-frame query of itself, the answer is true when any of animation event bits 0x1 to 0x10 is set on its current frame. When it asks the same query of the hero, the answer is the hero's attack-active word (event bit 0x1 of the hero's current frame, SPEC-0036). High. The serpent's stun strike uses the hero-side form (the hero's attack is active).
- Animation numbers below are entries of the serpent's own animation table (SPEC-0005).

## Layout

### Tunable values

All defaults below are taken from the boot image on this disc. Times are seconds; a tick is 1/60 s (SPEC-0007), so ticks = trunc(seconds x 60). "Chance" is an integer percent roll (see Behavior).

| Tunable (descriptive name) | Type | Address | Default | Unit | Used |
| --- | --- | --- | --- | --- | --- |
| Mesmerize chance, phase 1 | int | 0x2DB038 | 0 | % | yes (phase 1 never mesmerizes) |
| Mesmerize chance, phase 2 | int | 0x2DB03C | 40 | % | yes |
| Mesmerize chance, phase 3 | int | 0x2DB040 | 60 | % | yes |
| Bite drawback range, stage 0, min / max | float | 0x2DB044 / 0x2DB048 | 1.0 / 1.2 | multiplier | yes (bite decision; also the phase 1 and phase 4 range after a mesmerize) |
| Bite drawback range, stage 1, min / max | float | 0x2DB04C / 0x2DB050 | 0.9 / 1.1 | multiplier | yes (phase 2 range after a mesmerize; not the bite decision) |
| Bite drawback range, stage 2, min / max | float | 0x2DB054 / 0x2DB058 | 0.8 / 1.1 | multiplier | yes (phase 3 range after a mesmerize; not the bite decision) |
| Bite block time | float | 0x2DB05C | 0.5 | s | yes |
| Lunge damage | int | 0x2DB060 | 20 | HP | yes |
| Stun time, stage 0 | float | 0x2DB064 | 5.0 | s | yes (the only stun time used; stage is always 0) |
| Stun time, stage 1 | float | 0x2DB068 | 4.0 | s | no (stage is always 0) |
| Stun time, stage 2 | float | 0x2DB06C | 3.0 | s | no (stage is always 0) |
| Poison time | float | 0x2DB070 | 15.0 | s | yes (duration of the poison status the cloud gives the hero) |
| Poison projectile speed | int | 0x2DB074 | 2 | units per tick | yes (belch launch speed; stored but not used by the cloud's own movement) |
| Poison projectile radius | float | 0x2DB078 | 6.0 | world units (horizontal) | yes (the cloud's trigger radius) |
| Poison projectile life | float | 0x2DB07C | 5.0 | s | yes |
| Mesmerize effect time | float | 0x2DB080 | 5.0 | s | no (the stun payload uses a fixed 5 s, see Behavior) |
| Mesmerize particle lifetime | float | 0x2DB084 | 0.5 | s | no |
| Belch cooldown, minimum | float | 0x2DB088 | 5.0 | s | yes |
| Belch cooldown, maximum | float | 0x2DB08C | 7.0 | s | yes |
| Mesmerize effect start (internal) | float | 0x2DB090 | 0.7 | s | yes (42 ticks after the 10-tick delay) |
| Mesmerize delay before effect (internal) | int | 0x2DB094 | 10 | ticks | yes |

Registered but never used by the covered behavior. Do not apply them:

| Tunable (descriptive name) | Address | Default |
| --- | --- | --- |
| Idle attack chance, first (lower tier) | 0x2DAFF8 | 30 |
| Idle attack chance, second | 0x2DAFFC | 15 |
| Idle attack chance, third | 0x2DB000 | 0 |
| Idle tail-slap chance, first to third | 0x2DB004, 0x2DB008, 0x2DB00C | 60, 60, 60 |
| Tail-slap continue chance, stage 0 / 1 / 2 (`SlapStage1IdleChance`, `SlapStage2IdleChance`, `SlapStage3IdleChance`; see Stage numbering below) | 0x2DB014, 0x2DB01C, 0x2DB024 | 50, 30, 10 |
| Tail-slap damage (`SlapDamage`) | 0x2DB028 | 20 |
| Tail-slap strike limit, stage 0 / 1 / 2 (`SlapStage1Max`, `SlapStage2Max`, `SlapStage3Max`; see Stage numbering below) | 0x2DB010, 0x2DB018, 0x2DB020 | 3, 4, 6 |
| Belch chance, first to third | 0x2DB02C, 0x2DB030, 0x2DB034 | 10, 25, 40 |

Stage numbering: the developers' names for the six slap words (`SlapStage1Max` to `SlapStage3IdleChance`) number stages from 1, so `SlapStage1` is stage 0 in this spec. This spec numbers stages from 0 throughout. The values agree with the names; do not shift the index.

No reachable path reads these words. No load or store to 0x2DAFF8 to 0x2DB00C, the strike limits (0x2DB010, 0x2DB018, 0x2DB020), the belch chances (0x2DB02C to 0x2DB034), or the mesmerize time and particle lifetime (0x2DB080, 0x2DB084) exists in the boot program (High). The tail-slap continue chances, the strike limits and damage are used only by the tail-slap routine, which is linked into the boss but not entered (Medium-High). The belch cooldown (0x2DB088, 0x2DB08C) is the only belch timing used.

Fixed values used by the engagement and the phase change (not data words, so not read from the disc data): the bite height threshold 115.0, the bite distance 20.0, the phase-change facing threshold 0.99, the phase-change turn blend 0.1 per tick, and the arrival ground-probe start height 8.0 (the probe starts 8.0 units above the arrival point).

### Arena anchors

There are eight anchor slots, numbered 0 to 7. Each slot has three level object ids: A, B and C. Id B is the point the hero's distance is measured to. Ids A and C are the two ends of the anchor's facing axis. Within the serpent's state each slot keeps the three object references in the same order (A, then B, then C), so the nearest-anchor search reads only the B references.

| Slot | A | B | C |
| --- | --- | --- | --- |
| 0 | 106 | 105 | 104 |
| 1 | 103 | 102 | 101 |
| 2 | 124 | 123 | 122 |
| 3 | 121 | 120 | 119 |
| 4 | 118 | 117 | 116 |
| 5 | 115 | 114 | 113 |
| 6 | 112 | 111 | 110 |
| 7 | 109 | 108 | 107 |

Ids are for the serpent's level. The same numeric ids 101 to 106 are used by another boss's arena (SPEC-0010), so look ids up only inside this boss's level.

### Per-fight state (not on disc)

- Phase: integer 1 to 4, starts at 1.
- Stage: integer, always 0 in this build (see Behavior 3). Starts at 0.
- Current anchor slot (0 to 7), starts at 0. Anchor flag (0 or 1), starts at 0. Relocation variant (0 or 1), starts at 0.
- Phase-change slots S1 and S2: each is an anchor slot number (0 to 7) or none (−1). They are written at the start of each phase change (Behavior 2 and 4; Open question 17). Both start at none (−1) when the controller starts (High).
- Belch cooldown in ticks, starts at 0. Armed flag, starts at 1.
- Stun strike latch: one strike per stun.

## Behavior

### 0. Conventions

- Distances are horizontal: d = sqrt(dx^2 + dz^2). Height is ignored everywhere except the engagement gate in section 5 (the hero's height).
- Percent rolls are integers from 0 to 100 inclusive; a roll is a success when it is less than the chance (SPEC-0014).
- The random generator is the shared one, seeded once from the console clock (SPEC-0014).
- A tick is 1/60 s. Level flags are set and cleared through the level's flag array (SPEC-0014).

### 1. Tick order

Each tick, in this order:

1. Pose parameter reset to 1.0 and the active flag set.
2. If the belch cooldown is above 0, subtract 1.
3. Health band check (section 2).
4. Engagement check (section 5), only when armed and the state allows it.
5. Anchor check (section 4).
6. Run the current state (sections 6 to 9).
7. Aim and presentation (not covered).

### 2. Phases

Let f = current health / maximum health.

| f | Band phase |
| --- | --- |
| above 0.75 | 1 |
| above 0.50 up to and including 0.75 | 2 |
| above 0.25 up to and including 0.50 | 3 |
| 0.25 or less | 4 |

- The phase check runs every tick in every state except 1 (move), 5 (relocate), 6 (arrive), 7 (bite) and 11 (phase change); this is the same state gate as the engagement check. States 2 and 3 are never entered (Open question 1), so in play the check runs only while idle (0) or belching (4) (High). It does not run during move, relocate, arrive, bite or phase change.
- If the band phase is not 1, and the stored phase is not already equal to the band phase, a phase change starts (state 11). In practice: at f <= 0.75 the stored phase must be 2, at f <= 0.50 it must be 3, and at f <= 0.25 it must be 4. Nothing happens when f is above 0.75.
- Phase change (state 11). The change runs in up to two passes, over the two stored phase-change slots S1 (first) and S2 (second). A pass runs for each stored slot that is not none:
  1. While the serpent does not face the centre point (point B) of the stored slot, with a facing dot product below 0.99, it plays animation 5 and turns toward that point with blend 0.1 per tick. The hero's position is not used.
  2. Once it faces the point, it plays animation 21.
  3. At the animation's event 0x20, it sets level-state flag S (the slot number) if that flag is clear. The event is not latched: the setter can run on each tick the animation event reads active while the flag remains clear. In the swamp, the matching numbered break scene is queued by the level watcher (Arena break-scene response).
  4. When animation 21 ends, the pass ends and its stored slot becomes none. After the first pass the second pass starts on the next tick. After the second pass the state becomes 0 (idle) and the phase increases by 1.
  If S1 is none, the second pass runs alone. If S2 is none, the change does not complete (the state stays 11), even if S1 was set; the slot writer writes both slots together, so this mixed case does not arise from it. If both are none, the routine returns without action on every tick and the change does not finish (Edge cases; Open question 13).
  Confidence: High for the order, the 0.99 test, the 0.1 blend, the target, animation 21, the phase increment and the linked break-scene response.
- Net effect: a fall from 100% to 20% in one go chains three phase changes, one after another. See Edge cases for the healing case.

### 3. Stage (fixed at 0)

- The stage is 0 for the whole fight. It starts at 0 and nothing in the covered behavior writes it.
- Stage 0 values apply to the bite decision (section 8) and to the stun (section 8).
- The post-mesmerize drawback draw selects the ranges by the current phase: phase 2 uses the second registered pair (0.9–1.1), phase 3 the third (0.8–1.1), and phases 1 and 4 the first pair (1.0–1.2). The stage field does not select this draw (High).

### 4. Anchors and relocation

Runs each tick when the state is idle, belch or arrive.

1. Measure the horizontal distance from the hero to point B of each of the 8 anchor slots. Choose the nearest slot k. Compare with strict less-than, starting from a very large value, so on a tie the lower slot wins.
2. If k equals the current slot, do nothing (except in state 6, see 4.3).
3. If k differs and the state is not 6: enter the relocate state (5). Set the relocation variant to the previous value of the anchor flag. Set the current slot to k. Then draw a fresh random integer 0 or 1: if it is 1, set the anchor flag to 0; otherwise set it to 1.
4. In state 6 (arrive): the chooser uses the current slot even when it differs from the nearest one. The serpent's position is set to the chosen point: point C when the anchor flag is 0, point A when it is 1 (the flag drawn at the relocation that chose this anchor). Then its height is replaced by the ground height under (x, z), found by the ground probe started 8.0 units above the point (SPEC-0018). No effect is shown. Then it enters the move state (1). (High.)

Relocate (5): plays animation 4 when the variant is 0, and animation 6 when it is 1. When it finishes, the serpent enters arrive (6).

Move (1): the serpent turns to face along the axis from point C to point A (anchor flag 0) or from A to C (flag 1), blending its heading toward that direction with a factor of 0.25 per tick. It plays animation 3. When animation 3 finishes, it returns to idle (0).

Phase-change slots (used by Behavior 2): when a phase change starts, the serpent records two slot numbers from its current anchor k. The first slot is k itself. The second is its partner in the pair 1 and 2, 4 and 5, or 6 and 7, so the pair swaps (anchor 1 records 1 and 2; anchor 2 records 2 and 1). Anchors 0 and 3 record nothing and keep their previous values. The slots are written only at the start of a phase change, not on relocation. The stored values are anchor slot numbers, not animation ids (Open questions 2 and 17).

### 5. Engagement: bite or belch

Runs only when armed, and in the states where the phase check runs (every state except 1, 5, 6, 7 and 11, which in play means idle or belching). Not run during move, relocate, arrive, bite, or phase change, so a bite can interrupt a belch. The check does nothing at all while the serpent is in the bite state.

- Bite candidate: the hero's height is above 115.0 (strict), the hero is within 20.0 horizontal units (strict), and the anchor slot is not suppressed. The anchor slot is suppressed when it is 0, and when it is 3 unless the phase is 4. The height is the hero's vertical position; the distance is the horizontal distance (Open question 16).
- If a bite candidate: if the hero's action code is 30, nothing happens. Otherwise start the bite (state 7). The hero's action code is the hero's state number (SPEC-0033), so code 30 is the hero hanging from a ledge: the serpent neither bites nor mesmerizes while the hero hangs (Medium-High; Open question 10).
- If not a bite candidate: if the belch cooldown is 0 and the state is not already belch, start belch (state 4).
- Consequence: when the hero is out of range (or the serpent is suppressed), the serpent belches every time the cooldown reaches 0. The cooldown starts at 0, so the first belch can happen on the first tick the state allows it.

### 6. Idle, move and wait

- Idle (0): plays animation 5 and, on every tick of this state, requests the neck look at the active hero (6A).
- Move (1): see 4.
- Wait (2) is never entered (see Open questions).

### 6A. Neck look (Medium-High)

The serpent uses the shared look-at controller in its neck mode (SPEC-0040, Behavior 23A). The request is stored on the serpent's animation player and applied by the pose pass after the animation, then cleared.

- Joints: the six neck joints of the joint list at ELF data address 0x320F48 (joints 6, 7, 8, 11, 14 and 17). They turn about the world up axis (0, 1, 0) only; there is no pitch.
- Angle: h_t is the horizontal unit vector from joint 6 to the hero, and h_b the horizontal unit vector from joint 6 to joint 17. A = acos(h_t · h_b), with no clamp of the dot product. The side test is s = unit(h_b × up) · h_t. Each joint's target is τ = −A/6 when s > 0, else +A/6.
- Step: each of the six angles steps by 0.01 per tick. Within 0.01 of τ it takes τ. Otherwise it may only increase when s > 0, and only decrease when s ≤ 0.
- Each joint is then turned about the up axis by its angle, on top of the animated pose.
- On a tick with no request (any state other than 0) the six angles are set to 0.

### 7. Poison belch (state 4)

- Plays animation 10. The projectile is fired once per belch, at the animation's event frame, if a projectile slot is free.
- Origin: the serpent's mouth point. Launch velocity: unit vector from origin to the hero, times the poison projectile speed (2 units per tick). Life: 300 ticks (5 s). Projectile effect id: 414.
- Projectile slots: a pool of about 10. If the pool is full at the event frame, that belch fires nothing.
- When animation 10 finishes: state 0 and the belch cooldown is set to a random whole number of ticks in [300, 420] (5 to 7 s, inclusive).
- The belch chances (10, 25, 40) are not used. The belch timing depends only on the cooldown.

### 8. Bite and mesmerize (state 7)

The bite is a sub-state machine. Sub-states:

- **Sub 0, decide.** Mesmerize is considered only when the hero is not in the air and the hero's action code is not 30. "In the air" is the shared hero jumping test: the hero's action code is one of 7, 8, 9, 10 or 11 (the jump states) or 43 to 46 (further air states) (High). If it is not considered, the bite proceeds. When it is considered, roll an integer in [0, 100]. If the roll is less than the mesmerize chance for the current phase (phase 2 uses 40, phase 3 uses 60, phases 1 and 4 use 0), go to sub 1. Otherwise go to sub 2 and draw the drawback multiplier uniformly from the stage-0 range [1.0, 1.2]. Reset the block counter.
- **Sub 1, mesmerize.** Plays animation 20 (windup). A counter counts the ticks while animation 20 plays. When the counter passes 10, the mesmerize effect starts (it is started once). From tick 52 (10 + 0.7 s x 60), on every tick where the hero is not already stunned, the hero receives the stun status for 5 s (300 ticks). The effect's beams run from the serpent's head side toward the hero. Sub 1 ends when animation 20 finishes, or earlier if the hero is in the air (the same test) or has action code 30 (then the bite goes to sub 2 as well). When it ends, go to sub 2 with the drawback drawn from a range chosen by phase, not by stage: phase 2 uses [0.9, 1.1], phase 3 uses [0.8, 1.1], and phases 1 and 4 use [1.0, 1.2].
- **Sub 2, bite windup.** Sets the playback multiplier of animation 11 to the drawback and plays animation 11. While the windup plays, count the ticks in which the hero is blocking; reset the count when the hero is not blocking. If the count exceeds 30 (0.5 s x 60), go to sub 3 (spit). If the windup animation finishes first, go to sub 4 (lunge).
- **Sub 3, spit.** Plays animation 12. Fires the poison projectile at its event frame, under the same rules as section 7. When animation 12 finishes, the bite ends and the serpent goes to relocate (state 5).
- **Sub 4, lunge.** Plays animation 13. The hit test runs during the lunge window, at most once per lunge (see Lunge hit shape and window). If the hero is blocking and the lunge is within the block test (the hero faces the serpent, see below), go to sub 6 (blocked) with no damage. Otherwise, if the hit test passes, the hero takes the lunge damage (20, raw), is put into its reaction state 0x36 (54), and the serpent goes to sub 5. If neither happens, keep going. When animation 13 finishes, the bite ends and the serpent goes to relocate (state 5).
- **Sub 5, after a landed lunge.** Plays animation 14. When it finishes, the serpent goes to relocate (state 5).
- **Sub 6, after a blocked lunge.** Plays animation 22. When it finishes, the serpent goes to sub 7 with a stun counter of 300 ticks (5 s).
- **Sub 7, stun.** Plays animation 23. The counter goes down by 1 each tick. When it is below 1, go to sub 9. While the stun runs, the serpent can be struck once (see Stun strike): the first valid strike goes to sub 8.

**Hero action code 30 (Medium-High).** The serpent reads the hero's action-state word, which holds the hero's state number (SPEC-0033; the jump tests above use the same numbering: 7 to 11 and 43 to 46). State 30 is the ledge hang. The hero enters it from the in-air ledge chooser or from the drop off a ledge (state 37), both ordinary hero moves; no boss sets it. The serpent tests it as a no-mesmerize and abort-windup condition, so a hero hanging from a ledge is never bitten or mesmerized. The hero's hang handler is described in SPEC-0033 (state 30).
- **Sub 8, recovery after a stun strike.** Plays animation 28. When it finishes, the serpent goes to arrive (state 6).
- **Sub 9, end of stun.** Plays animation 24. When it finishes, the serpent goes to relocate (state 5).

Block test for the lunge: the hero is blocking and dot(u, -f) > 0.5, where u is the horizontal unit vector from the serpent to the hero and f is the hero's forward direction. That is, the hero faces the serpent within 60 degrees. The lunge is then blocked: the hero's take-hit is not called, and the serpent goes to sub 6.

**Lunge hit shape and window (High).** The lunge uses the monster weapon test (SPEC-0029 Behavior 10.2) with weapon 3 of the serpent's monster row (row 10, as stored in its fight region, resource 740): radius 5.0, length 5.0. The row's other weapons are size 0. Weapon 3 is tested only while event bit 0x4 is on, which is frames 19 to 25 of the 49-frame lunge clip; at clip speed byte 1 that is ticks 38 to 51 after the clip starts (SPEC-0005; Medium for the exact first tick). The damage, 20 (word 0x2DB060), goes straight to the hero's take-hit and is not reduced by the hero's shield, armour, armour Mote or petpet factor (SPEC-0029 Behavior 10.5).

Stun strike: while stunned, a single strike is possible. A strike is either the hero's melee attack (only when the hero's attack is active) touching the head, or a hero projectile within 5.0 plus its own radius of the head (the projectile is removed). The head test is a cylinder of radius 6.0 and length 10.0 along the head bone's axis, starting at the head bone. It is tested with the finite-cylinder overlap of SPEC-0018 (static; it returns a hit and a signed depth, and the hit is what counts here). The stunned sub-state of the serpent's bite handler performs this head test as part of its once-per-stun strike check (High; Open question 19). On a successful strike, the serpent loses the integer returned by the hero's damage query with multiplier 1.0. That value is passed to the serpent health subtraction directly; there is no fixed stun-strike damage or additional serpent-phase multiplier. Health is clamped at zero. The strike is recorded, so a second strike in the same stun does nothing. The damage query's internal hero-stat and equipment calculation is defined by the hero spec.

### Shape tests for the serpent's parts

These two shape tests are not the serpent's. They belong to two hero spell areas of SPEC-0036 (Hit volumes: the Nova area, radius 4.0 and height 2.0, and the Wind/Fog area, radius 4.0 and height 5.5, both centred on the spell object's stored point). Each spell walks the world's character list and, for every character that passes the character's own targetable query, tests that character's body cylinder (its motion record's radius and length, centred on the character's position) against the spell's cylinder with the finite-cylinder overlap of SPEC-0018; a touched character that is not yet in the spell's hit list is hit. Both cylinders use the constant up vector (0, 1, 0, 0) at ELF data address 0x3932E0 as their axis, so every call takes the parallel branch of SPEC-0018. That vector is one of five constant vectors written once at boot by a start-up initialiser (SPEC-0026, Start-up table walk) and never changed; it is not in the file image, so the clean side sets it itself. The serpent is hit by these spells like any other character, through its body cylinder; it has no part shapes of its own (High).

### 9. Tail slap (state 3): not entered

The main loop's state dispatch has an entry for the tail-slap state, so the tail-slap routine is linked into the boss. The routine is fully described. Its rules, for reference:

- Entry: the strike counter starts at 0. The strike limit is set from the stage: 3 at stage 0, 4 at stage 1, 6 at stage 2.
- Phase A: the routine plays animation 7 once.
- Phase B (strikes): the serpent plays animation 8, or animation 9 when the variant flag is 1. A swing would deal 20 damage (word 0x2DB028) once, when the monster weapon test passes (one hit per swing, with a per-strike latch). The serpent's animation table has no clip for animations 7, 8 and 9, so no event bit is ever on and the strikes would never hit even if the state were entered (High).
- After each finished swing the strike counter goes up by one. When the counter reaches the strike limit, the routine goes to relocate (state 5). This exit happens only at the limit, not on a failed roll.
- Otherwise, a roll below the continue chance (50 at stage 0, 30 at stage 1, 10 at stage 2) plays animation 7 and draws the variant again before the next swing. A failed roll starts the next swing at once.

The state is entered only from state value 3, and no known routine writes that value (see Open questions 1). The tail slap is therefore unreachable through the known routines, and it is not entered by the original. Do not make it reachable. Its counters and limits have no effect, and the port needs no runtime counters for it.

### 10. Poison cloud (projectile type 9)

- The cloud is a projectile of the shared enemy projectile pool (SPEC-0029 Behavior 9): behaviour kind 9 (sinking cloud), visual kind 15, damage 0 (High).
- Life 300 ticks.
- Each tick, if the horizontal distance from the cloud to the hero is below the poison radius 6.0, and the hero is not poisoned, the hero receives the poison status for the poison time (15 s = 900 ticks). The standard poison status applies its own damage per tick; the cloud deals no direct hit damage.
- The cloud has two mutually exclusive movement branches each tick. When its height is strictly more than 2.0 above the ground height sampled under the hero, it moves down by 0.1 and does not drift toward the hero on that tick. Otherwise, it takes the horizontal drift step toward the hero at 0.1 and does not sink on that tick. If it crosses the 2.0 threshold while sinking, the drift branch starts on the next update.
- The stored velocity from the launch (direction times 2) is not used for this movement.

### 11. Damage and effects

- Lunge: 20 to the hero, once per lunge, unreduced by the hero's defences; a blocked lunge deals nothing.
- Stun strike: the integer returned by the hero damage query with multiplier 1.0, applied once per stun.
- Mesmerize: the hero receives the stun status for 5 s (fixed, not the mesmerize-time tunable).
- Poison cloud: the hero receives the poison status for 15 s.
- Tail slap: not reachable (its damage, 20, is listed for completeness).
- Phase change: sets the level-state flag equal to the stored slot at animation 21's event 0x20, if that flag is clear. The swamp watcher turns states 1–7 into the matching break scenes and clears each after its scene; when multiple are pending, lower state numbers play first (Arena break-scene response).

### 11A. Arena break-scene response (Confidence: High)

The phase-change event is also the trigger for the swamp's break scenes. When animation 21 reaches its event, the serpent sets the level-state flag matching the stored anchor slot. The swamp watcher checks states 0 through 7 in ascending order. For each set state, it plays that group's scene and clears the state before continuing. The scene breaks that group's registered breakables in index order, runs its paired falling-object animation, then activates the group's platform. If both flags are pending together, the lower-numbered group runs first regardless of the order in which the serpent posted the flags.

For example, a phase change that posts states 2 and 1 in that order is presented as group 1 followed by group 2: group 1 breaks objects 4 to 6, shows falling object 59, and activates platform 5; group 2 breaks objects 7 and 8, shows falling object 60, and activates platform 6. The thread pauses its one-tick polling loop while each scene runs, so it does not interleave the two scenes.

The seven groups map states 1–7 to consecutive breakable ranges 4–31, falling objects 59–65, and platforms 5–11. The physical route made by those platforms, and whether a particular path is needed to reach the serpent's height-based bite gate, remain unknown.

### 11B. Sounds and effects

Sounds (High). The serpent's controller plays no sound itself; every serpent sound comes from its animation table (SPEC-0031, animation sounds). The cues are in group 1, creature block 366. Every sound entry has start frame 0 and no keep-alive bit. When the serpent's main animation id changes to k, the cue of entry k plays once, on the first tick, on block 366's slot at the serpent's position. A request of the id that is already current plays nothing. No cue plays while the serpent is in base state 42.

| Animation | Cue | Sound |
| --- | --- | --- |
| 3 | 0 | rise (move) |
| 4 | 1 | submerge (relocate, variant 0) |
| 6 | 2 | hiss (relocate, variant 1) |
| 8 | 3 | tail slap (not reached) |
| 9 | 4 | tail slap (not reached) |
| 10, 12 | 5 | poison belch (belch, spit) |
| 11 | 6 | bite draw-back (windup) |
| 13 | 8 | lunge |
| 14 | 9 | bite hits the hero |
| 15 | 10 | bite blocked |
| 17, 22 | 11 | stunned flinch |
| 18, 24 | 12 | stunned recover |
| 19 | 13 | angry recover |
| 20 | 14 | mesmerize |
| 21 | 15 | standing stone destroyed (phase change) |
| 28 | 1 | submerge (recovery after a stun strike) |

Animations 5 (idle), 7 and 23 (stun) are silent; animation 7 has neither a clip nor a sound. Cue 7 of block 366 is never played.

Effects (group 3, effect resource 9):

- Poison projectile 414: a cloud of 17 olive sprites every 2.7 s, 3 s life, size 5 (High).
- Clip effects of animations 3 (rise) and 4 (submerge): effect 480 twice (30 grey model particles, once, 1 s), on attach slot 4, positional mode 0 (Medium-High).
- Clip effects of animations 10 and 12 (belch): effect 4040 (one olive sprite every 0.02 s, 2 s life) over progress 0.2 to 0.8 of the clip, on attach slot 7 (Medium-High).
- Mesmerize: while animation 20 plays, two objects of effect 413 (blue model particles every 0.1 s, 0.7 s life) are kept on two head points; both stop when the bite leaves animation 20 (High for the effect and the rule; Medium for the exact points).
- Landed bite: the shared hit-flash helper (SPEC-0034), which starts a one-shot detached effect 222 (red flash, two models, 15 orange sparks, red light, 0.2 to 0.4 s), or 220 when its flag is 1. The serpent's bite plays 222 (High for the id; Medium for the point).

### 12. Timings (60 Hz)

| Item | Value | Ticks |
| --- | --- | --- |
| Belch cooldown | random integer in [5 x 60, 7 x 60] | 300 to 420 |
| Poison projectile life | 5 x 60 | 300 |
| Bite block threshold | count over 0.5 x 60 | 30 |
| Mesmerize effect start | counter over 10, effect from 10 + 0.7 x 60 | 52 |
| Stun, stage 0 | 5 s | 300 |
| Poison status | 15 s | 900 |
| Phase change | per pass: turn to face, then animation 21 to completion; two passes | variable |

### 13. Inputs and outputs

- Inputs: hero position, hero height, hero action code, hero blocking flag, hero's forward direction relative to the serpent, the hero's attack and projectile state, the serpent's health and maximum health, animation playback (playing, finished, event frame), level object positions by id, the 60 Hz tick, the random number generator, and the two phase-change slots.
- Outputs: animation requests on the serpent (ids above), the hero's reaction state 0x36 (54) when a lunge lands, projectile spawns (effect id 414), damage to the hero, the stun and poison statuses, the belch cooldown, the phase, the anchor placement, the phase-change level-state flags, the stun counter, and the animation cues and effects of 11A.

## Edge cases and error handling

- Healing into a higher band repeats phase changes. The phase check compares the stored phase with the band phase and increments the phase on each change, so the phase can go past 4. For example, at 60% health with phase 3: the phase check triggers (band 2 is not phase 3), the phase becomes 4. The next check (band 2 is not 4) triggers again, and so on. Reproduce this, as the original does (reached only by healing the boss).
- Phase values 5 and above have the same mesmerize and drawback values as phase 1 (0 % mesmerize, range [1.0, 1.2]).
- Anchor 0 suppresses bites, so a serpent at slot 0 never lunges until it relocates. Anchor 3 suppresses bites unless the phase is 4.
- In state 6 the placement uses the slot already set (not the nearest one). Relocation picks the nearest slot only in idle and belch.
- Belch: the cooldown starts at 0. The first belch can start on the first allowed tick when the hero is out of bite range. Projectiles are fired only at the event frame. A full projectile pool silently drops that projectile.
- The mesmerize stun is applied on every tick after tick 52 where the hero is not stunned, so a hero who is stunned during the effect is not stunned again until the stun ends.
- Mesmerize chance 0 for phases 1 and 4 means the serpent never mesmerizes in those phases.
- The stun can be ended by a strike. A strike during the stun is recorded once and then ignored.
- Tail slap is dispatched but unreachable (Medium-High): the dispatch entry runs the routine only when the state value is 3, and no known routine writes 3. The routine's exit to relocate happens only when the strike counter reaches its limit. An implementation that adds a trigger for it is outside this spec. Do not add the tail-slap counters or limits to the serpent's runtime state.
- Unused tunables: the stage-1 and stage-2 stun times, the tail-slap idle chances, the belch chances, the mesmerize effect time and particle lifetime (see the Layout table). Do not apply them.
- **Phase change with no slot (soft-lock)**: if both phase-change slots are none when the change starts, the state stays 11 and nothing else happens. The change never completes, so the serpent does not return to idle, and the phase check (which does not run in state 11) never runs again. This is an original defect: keep it for any path that reaches it (Open question 13).
- **Phase change at anchor 0 or 3**: the slot writer records nothing at these anchors, so the slots keep the values they held before (none at the start, and none after any completed change), and such a change would soft-lock. In normal play this does not happen (Medium-High): the serpent loses HP only through the stun strike, which follows a bite at its current anchor. Bites never start at anchor 0, and start at anchor 3 only in phase 4. After the strike the serpent arrives and moves at the same anchor, and the phase check runs before relocation in the tick order, so the change starts at the strike's anchor. Chained changes run at the same anchor too. So a change never starts at anchor 0, and at anchor 3 only in phase 4, when no change is due.
- **Neck moves one way per side (reproduce)**: the allowed step direction points away from τ, so from 0 an angle changes only by snapping to a τ within 0.01. When τ is more than 0.01 away the neck freezes, until τ comes back within 0.01 or a tick outside state 0 sets the angles to 0 (6A).
- **Neck reset**: the neck angles go back to 0 on every tick without a request, so leaving idle snaps the neck to the animated pose (6A).
- **Unused cue**: cue 7 of block 366 is named by no animation entry and no code, so it never plays. Re-requesting the current animation does not replay its cue (11A).
- **Phase-change event repeats**: the serpent checks the matching level-state flag each tick event 0x20 is active. If clear, it calls the level-state setter with that slot number and true; no separate boss-side latch exists. Once the flag is set, later event ticks do not call the setter. (High).

## Test vectors

Synthetic values. Health is written as health / max.

- Mesmerize gate: phase 3, bite decide, hero action code 8 (a jump): mesmerize is not considered and the bite proceeds. Hero action code 11: not considered. Hero action code 12: considered; roll 59 gives mesmerize (59 < 60). Hero jumps (code 7) during the windup at tick 30: the bite goes to sub 2 that tick, before tick 52, so no stun is given.

- 100 / 100, phase 1: no change. 75 / 100 (f = 0.75 exactly), phase 1: band 2, phase change starts. 50 / 100, phase 2: band 3, phase change starts. 25 / 100, phase 3: band 4, phase change starts. 20 / 100, phase 4: no change.
- 60 / 100, phase 3: phase change starts (healing case in Edge cases).
- Phase check gate: 20 / 100 with phase 3 wants phase 4. In state 0 or 4 the change starts (state 11). In state 1, 5, 6, 7 or 11 nothing happens that tick; the check runs again on the first tick in a state outside that set (for example idle after arrival). A change is therefore never started while moving, relocating, arriving or biting.
- Belch cooldown roll 300 gives 300 ticks (5 s). Roll 420 gives 420 ticks (7 s).
- Hero 19 units away, hero height 120, slot 2, phase 1, hero action code 30: bite candidate, but the code is 30, so no bite and no belch.
- Hero 19 units away, hero height 120, slot 2, phase 1, hero action code 0, cooldown 0: bite starts. Roll for mesmerize chance 0: always bite.
- Hero height 115.0 exactly: not a bite candidate (strictly above is needed).
- Hero 20.0 units away: not a bite candidate (strictly below is needed).
- Serpent in the bite state with a hero in range: the engagement check changes nothing.
- Phase 2, roll 39 (chance 40): mesmerize. Roll 40: bite.
- Bite decision drawback (stage 0): uniform in [1.0, 1.2].
- Phase 2 drawback after a mesmerize: uniform in [0.9, 1.1].
- Mesmerize counter: at tick 52 of animation 20 the hero is stunned for 300 ticks if not already stunned. At tick 51 it is not.
- Block count 30 at windup: still windup. Block count 31: spit.
- Lunge: hero blocking and facing the serpent (dot(u, -f) = 0.6): blocked, recoil, then a 300-tick stun.
- Lunge window (disc clip): weapon 3 is tested on lunge frames 19 to 25 only. A hero with 100 HP and armour, inside weapon 3 on frame 20 and not blocking: HP 80.
- Stun strike: if the hero damage query with multiplier 1.0 returns 17 and the serpent has at least 17 HP, a successful first strike removes 17 HP; at 10 HP it leaves 0 HP and triggers the health-zero path. Any later strike in that stun removes no more HP.
- Spell area against the serpent (synthetic body radius 1.0, heights overlapping): a Nova area centred 4.9 units horizontally from the serpent's position touches it (4.9 < 4.0 + 1.0); at 5.1 units it does not. The test is the same for every character.
- Poison cloud: hero at 5.9 horizontal units and not poisoned: poisoned for 900 ticks. At 6.0: not poisoned.
- Serpent at slot 0, idle, hero nearest slot 5: state 5 with variant set to the previous anchor flag, slot 5 current, and a new random anchor flag. Placement at slot 5 happens in arrive, at point C if the flag is 0, or point A if it is 1.
- Slot 3 ids: A 121, B 120, C 119.
- Phase change, slots S1 = 2 and S2 = 1 (anchor 2 is current), phase 1 at 75 %: the first pass faces the centre point of slot 2 (turn at 0.1 per tick), plays animation 21, sets level-state flag 2 at its event 0x20 if that flag is clear, and sets S1 to none when 21 ends. The second pass does the same for slot 1, then the state is idle and the phase is 2. With both slots none at the start of a change (a change started at anchor 0 or 3, which normal play does not produce), the state stays 11 on every tick.
- Phase change, S1 = none, S2 = 5: only the second pass runs.
- Phase change, S1 = none, S2 = none: the state stays 11 on every tick (Edge cases).
- Controller start: S1 = none, S2 = none, phase 1, current anchor slot 0.
- Neck look (synthetic, 6A): A = 0.6 with s ≤ 0 gives τ = 0.1; from 0 nothing moves. A = 0.05 gives τ = 0.00833, taken at once. If A then rises to 0.2 in one tick, τ = 0.0333 and the angles stay at 0.00833. A tick outside state 0 sets them to 0.
- Sounds (disc, 11A): relocate with variant 1 starts animation 6, so block 366 cue 2 plays once on that tick. A landed lunge then animation 14: cue 9 once. Re-requesting animation 14 while it is current plays nothing.
- Arrival (synthetic): anchor flag 1, point A at (10, 5, 20), the ground under (10, 20) at height 3.2: the serpent is placed at (10, 3.2, 20) and enters move.
- Reachability: the idle, move, belch, relocate, arrive, bite and phase-change states never set the state value 3 (the known routines), so no sequence of those states reaches the tail-slap state. A tail-slap test is not required.
- Tail-slap rule (synthetic, not reachable in play): stage 0, strike limit 3. After the third finished swing the routine relocates. After the second, a failed continue roll (roll 50 or more at stage 0) starts the next swing at once.

## Open questions

1. Resolved: tail slap (3) and wait (2) are never entered. The tail-slap dispatch entry exists, but no code stores its value; every store to the state word in the controller and its helpers writes 0, 1, 4, 5, 6, 7 or 11 (Medium-High for the helpers; High that no code outside the serpent's own routines can write the state; Medium-High that none of the serpent's routines writes 2 or 3): the state word is a local variable of the serpent's per-tick controller routine, and the routine hands its address only to the serpent's own state handlers and helpers. No generic actor code and no script call ever receives it, so a write of the same number elsewhere in the program cannot reach it. States 2 and 3 are therefore not entered in play, and the tail slap and the wait are not to be implemented as reachable. Test vector: whatever the hero does (stunning the serpent, standing in the slap's reach, any phase), the serpent's state after any tick is one of 0, 1, 4, 5, 6, 7 or 11, never 2 or 3. The wait handler (state 2) sets state 0 when animation 5 finishes; no known routine writes state 2 either (Medium-High).
2. **Slot values are anchor slots, not animation ids.** Resolved: the phase-change routine plays animation 21 in both passes and uses the two stored values as anchor slot numbers (the point it faces, and the level-state flag it sets). The pairs 1 and 2, 4 and 5, and 6 and 7 swap; slots 0 and 3 record nothing (High). Test vectors: current anchor 5 when a phase change starts stores S1 = 5 and S2 = 4, so the first pass faces point B of slot 5 and sets level-state flag 5, and the second pass faces slot 4 and sets flag 4. Current anchor 2 stores S1 = 2 and S2 = 1. Current anchor 6 stores S1 = 6 and S2 = 7. Current anchor 0 or 3 stores nothing, so the slots keep their old values (none at the controller start, which leaves the change stuck in state 11; Edge cases).
3. **Idle look.** Which bones joints 6 and 17 are (head and jaw, or neck base). This decides whether A is near 0 or near π when the serpent faces the hero, and so whether the neck visibly turns in play (Medium). Answered: the idle state requests the shared neck look on six joints about the up axis (6A).
4. **Arrival placement.** Resolved: the position is set to the chosen point (point C for flag 0, point A for flag 1), with its height replaced by the ground probed from 8.0 units above it; no effect is shown (Behavior 4; High).
5. The busy predicate used by mesmerize and the abort rule. Resolved: it is the hero jumping test (action codes 7 to 11 and 43 to 46), checked at the decide step and again each tick of the mesmerize windup, which sends the bite to the windup (sub 2) when it holds (Sub 0 and Sub 1; High).
6. **Post-mesmerize drawback range selection.** Resolved: when the mesmerize ends or aborts into the bite windup, the current phase selects the registered range pair: phase 2 uses pair 2 (0.9 to 1.1), phase 3 uses pair 3 (0.8 to 1.1), and phases 1 and 4 use pair 1 (1.0 to 1.2). This is phase-indexed; the fixed stage value does not select the pair (High).
7. Serpent sounds and effects. Resolved: the cues of creature block 366, played only through its animation table, and the effects listed in 11B (High).
8. Tick rate: the original scales seconds by a runtime tick rate that is written at start-up; this spec assumes 60 (SPEC-0007).
9. Resolved (High for the caller contract): the stunned strike calls the hero damage query with multiplier 1.0 and passes its integer result directly to the serpent health subtraction. There is no fixed strike amount or additional serpent-phase multiplier; the query's internal hero-stat/equipment calculation belongs to the hero damage spec.
10. **Hero action code 30.** Resolved (Medium-High): the code is the hero's state number, so 30 is the ledge hang of SPEC-0033, entered by the hero's own ledge moves. The serpent excludes it from mesmerize consideration and aborts a mesmerize windup to the bite windup if it appears. An earlier reading of a handler that requests animation 13 and then moves to action 27 belongs to a different class's dispatcher and does not apply to the hero.
11. The "shared volume test". Resolved: the monster weapon test of SPEC-0029 Behavior 10.2 (row 10, weapon 3 for the lunge); the tail-slap strikes have no clips, so they could not hit (8, Lunge hit shape and window; High).
12. **Phase-change target.** Resolved (High): the facing target is point B, the centre (second) point of the stored slot's anchor, the same point that the nearest-anchor choice measures. Each tick the direction runs from the serpent's own position to that point (with the height difference dropped and the result normalized; Medium for the height drop). While the dot product of the serpent's forward direction with that direction is below 0.99, the serpent requests animation 5 and blends its heading toward the direction by 0.1 per tick. The hero's position is never read in this step. Test vector: serpent at (0, 0, 0) facing +Z, stored slot 2 with point B at (10, 0, 0): the dot product is 0, so animation 5 plays and the heading blends 0.1 toward +X each tick; wherever the hero stands (for example at (-30, 0, 5)), the target does not change. Once the dot product reaches 0.99, animation 21 starts.
13. **Soft-lock with both slots none.** Resolved (Medium-High): whether any path reaches the soft-lock: in normal play a change can only start at the anchor where the stun strike landed, which is never 0 and is 3 only in phase 4, so it is not reached (Medium-High). Answered: the rule itself is in Edge cases (High); keep the defect for any path that does reach it.
14. **Phase-change level-state flag.** Resolved for the boss-side operation: at animation 21 event 0x20, the serpent checks level-state flag S and calls the level-state setter for S with true only when the flag is clear (High; SPEC-0030 SetLevelState). The level script's response is in pack 735: its breaker thread turns level state k (1 to 7) into the break scene of group k (breakables, a falling object and a platform object; SPEC-0044, module 735). A phase change at anchor 1 or 2 posts states 1 and 2, at anchor 4 or 5 states 4 and 5, and at anchor 6 or 7 states 6 and 7, one per pass, so each phase change plays two break scenes (High). The story meaning beyond that is not stated by the data. The bite decision has no direct dependency on these group rows or level states: it uses the hero height, horizontal distance, anchor suppression and boss state. Whether activated platforms physically enable the hero to reach the required height is a separate world-geometry question (Open question 21).
15. **Arrival final position.** Resolved: the chosen point with its height replaced by the probed ground height; the second store only forces the fourth component (w) to 1 (High; see item 4).
16. **Engagement metric.** Resolved (High): the engagement distance is the horizontal distance from the boss to the hero (x and z), and the height test uses the hero's height directly. The distance comes from the shared boss-to-hero helper, whose distance output is the horizontal length. Confidence: High. Test vector: serpent at (0, 0, 0), hero at (3, 130, 4): horizontal distance 5.0 (height ignored), height 130 > 115, so the bite candidate holds; hero at (3, 100, 4): same distance, height 100, no bite candidate.
17. **Writer of the phase-change slots.** Resolved (High): the two slots are written at the start of a phase change, from the current anchor (pairs 1 and 2, 4 and 5, 6 and 7 swap; anchors 0 and 3 write nothing). They are not written on relocation (High).
18. **Values before the first change.** Resolved: both phase-change slots none (−1), phase 1, current anchor slot 0 (Per-fight state; High).
19. Resolved: the two part-shape routines are the Nova and Wind/Fog spell area tests of SPEC-0036, run against every character's body cylinder, not serpent parts (see Shape tests for the serpent's parts; High). The 6.0 by 10.0 head cylinder is passed by the stunned sub-state of the serpent's bite handler for its once-per-stun melee check (Stun strike; High).
20. The shared axis vector at ELF data address 0x3932E0. Resolved: the constant up vector (0, 1, 0, 0), so the parallel branch of the overlap test applies (Shape tests for the serpent's parts; High).
21. **Static navigation mesh and the anchor area.** Resolved for the typed navigation mesh only (High): pack 735 has 20 navigation meshes, whose combined x extent ends at 3675.441. Its 24 serpent anchor locators have x values from 3736.490 to 3820.560, and none lies in a navigation-mesh triangle. The closest possible x separation is therefore at least 61.049 units. However, the pack also has top-level height patches covering every anchor x-z position (5–8 patches per anchor); sampled height is 110.788 at 23 anchors and 110.870 at one. Coarse height samples every 2 units from x=3675 to 3753 at z=-605, -615, -625 and -635 found ground-field coverage at all 160 sample points, with y=110.289–111.516. A denser transect at 0.5-unit x spacing has height-field coverage at all 636 samples along z=-605, -615, -625 and -635 from x=3675 to 3754, with y=110.244–111.516. This supports continued terrain-data coverage toward the western anchor rows, but patch-seam changes mean coverage alone does not prove continuous walkability, collision connectivity or player access. A candidate low-mesh sequence (objects 52, 53, 50, 51 and 54) runs from just east of the typed-mesh x boundary toward anchor 104; their stored heights are near 111.2 and center gaps are 15.6 to 20.7 units. These records have no inline height patch or collision parts, so this is not proof of hero support or a route to bite height. Near anchor slot 7, on a 0.125-unit x-z grid, one terrain point at (3760.375, -635.000) sampled y=111.859512. Adding the nominal ordinary standing-jump apex gives 115.062312, above the strict y>115 bite gate. The point is 9.406 units from slot 7's B point and 16.506 horizontal units from its A endpoint, satisfying the static nearest-slot and bite-range comparisons if the serpent arrives at A; its C endpoint is 20.445 units away. This is a candidate, not a route or proof that the hero can stand, jump, or retain the required horizontal separation there. The maximum height on the four transects above, 111.516, plus the ordinary standing-jump apex is 114.7188, below the bite gate; that result applies only to those four transects, not all bite-range terrain. Test vector: 3675.441 to 3736.490 gives a 61.049-unit navigation-mesh gap; at (3760.375, -635.000), sampled terrain plus nominal jump apex gives 115.062312, while slot 7's A and C endpoint distances are 16.506 and 20.445. These compare static terrain data, ordinary jump height and boss gates; they do not establish a traversable route or actual bite. [High for the resource bounds, samples, distances and arithmetic; open for physical traversal and bite access]




