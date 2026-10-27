# SPEC-0011: Boss behavior: the Faerie Sisters

- Answers: proactive, none requested (boss AI); R-0065
- Status: Draft
- Disc build: SCUS-97367 v1.00 (same fingerprint as SPEC-0001)
- Confidence: tuning values: High (the values held in the boot program's data). Ring radius (25) and the 120 and 240 degree slots: High. The index-0 offset (+25) and the index-1 fallback offset (−25), both on the world x axis: High. The alive count and the group word: the same value, High. Per-sister state machine, dodges, orbit and attacks: Medium-High; orbit direction flip on body contact or the navmesh blocked flag: Medium-High; chooser bands, including the special missile below 0.33: High; missile-dodge roll: High; melee-dodge roll: Medium (the trigger is described, the purpose of the animation is not established); move speed by alive count on the path step: High; orbit waypoint offset and speed input: High, while actual displacement depends on route and collision handling. HP-based animation-rate factor and its order before clip re-time: High. Behavior class ids of the three sister routines (13, 15, 16): High (SPEC-0029 section 3). Alternate loop energy ball: High for the speed, damage, life, turn blend, release rule, launch height and cooldown doubling; Medium-High for the aim point and for the random aim-offset construction, with the fixed reference vector's runtime orientation established from the startup vector record (see Behavior). Alternate loop selection by level flag 0: High for the flag bit test, Medium for the fight wiring. Dome start: High that it never starts in the shipped game (the group flag is only ever stored as 0); High for the unreachable intended rule and its thresholds. Taking damage and the shield: High. Arena flow (module 1122): Medium-High. Normal fight on the main loop: High for the level flag 0 setter in module 1174, Medium that module 1122 never sets it. Projectile kinds, visual kinds, statuses and the pool hit rule: High. Mote-eater drain and Leach MP drain on a hit: High (rules owned by SPEC-0034 3.13); the drain running faster with more sisters alive: Medium-High. Animation sounds and the code-started effects (4008, 222, 460): High; clip effect lists: Medium-High. Main-loop launch and aim points, element, status values and starting cooldowns: High (the starting cooldowns after the first start of a session: Medium-High). Thorn hazard: Medium for the strike counting, Medium-Low for its spawn height.

## Summary

Three sisters fight together. Each runs its own state machine: idle, orbit the hero, dodge, three kinds of ranged attack, flinch and death. They share one value, the **alive count** (0–3). As sisters fall, the survivors attack more often, fire faster projectiles and orbit faster. The sisters start shielded: they take no damage until a hero shot first hits one of them, which removes the shield for all three. A group dome attack (all living sisters on a ring around the hero) is present in the behavior but never starts in the shipped game (see Group (dome) attack). All timings use the 60 Hz tick (SPEC-0007). ticks = trunc(seconds × 60). "Random a–b" for times means a random whole number of ticks in [a × 60, b × 60] inclusive.

Each sister runs one of three controller routines, under behavior class ids 13, 15 and 16 (SPEC-0029 sections 3 and 4). The three routines share one state set and these tuning values. The controller's state numbers are given in the Layout section so that SPEC-0029 section 4 and this spec can be read together.

## Where it lives on the disc

- Tuning values: 32-bit words in the boot program's data, read at runtime by virtual address, in the range 0x2DAAC0–0x2DAB9F.
- "Animation n" means entry *n* of a sister's main animation table (SPEC-0005).
- Behavior class ids 13, 15 and 16 are entries of the behavior class table (SPEC-0029 section 3).

## Layout

Values indexed **by alive count** (shown for 1 / 2 / 3 sisters alive):

| Parameter | Type | Address of the "1" value | Values for 1 / 2 / 3 alive |
| --- | --- | --- | --- |
| Missile-dodge chance (`dodgeMissileChance1`–`3`) | int % | 0x2DAAC8 (+4, +8) | 50 / 60 / 75 |
| Melee-dodge chance (`dodgeMeleeChance1`–`3`) | int % | 0x2DAAD4 (+4, +8) | 40 / 50 / 60 |

Values with a **reversed** index: the value labelled "3" is used when 1 sister is alive, "2" with 2 alive, and "1" with 3 alive.

| Parameter | Type | Address (labels 1, 2, 3) | Values (min–max) for labels 1 / 2 / 3 |
| --- | --- | --- | --- |
| Energy-ball cooldown (`energyBallTimeMin1`–`3`; max `energyBallTimeMax1`–`3`) | float s | 0x2DAAE0, 0x2DAAE8, 0x2DAAF0 (min; max at +4) | 2.5–5 / 2.5–5 / 2.5–5 |
| Status-missile cooldown (`effectMissileTimeMin1`–`3`; max `effectMissileTimeMax1`–`3`) | float s | 0x2DAAF8, 0x2DAB00, 0x2DAB08 | 1.75–4 each |
| Special-missile cooldown (`specialMissileTimeMin1`–`3`; max `specialMissileTimeMax1`–`3`) | float s | 0x2DAB10, 0x2DAB18, 0x2DAB20 | 1.25–3 each |
| Group-attack interval (`groupAttackTimeMin1`–`3`; max `groupAttackTimeMax1`–`3`) | float s | 0x2DAB28, 0x2DAB30, 0x2DAB38 | 30–45 / 20–40 / 0–0 |
| Projectile speed (`projectileSpeed1`–`3`) | float units/tick | 0x2DAB94, +4, +8 | 0.5 / 0.75 / 0.9 |

The projectile-speed words are the launch speed: the launch velocity is the unit direction toward the aim point times the selected word, so the word is a speed in the launch-velocity units, not a scale (High).

Single values:

| Parameter | Type | Address | Value |
| --- | --- | --- | --- |
| Orbit band, minimum distance (`minDistance`) | float | 0x2DAB48 | 15 |
| Orbit band, maximum distance (`maxDistance`) | float | 0x2DAB4C | 30 |
| Movement speed, 3 alive (`movementSpeed`) | float units/s | 0x2DAB50 | 7.5 (overwritten with 10.0 by alternate loop A; see Controller routines) |
| Ring radius (`DF_SISTER_CIRCLE_RADIUS`) | float | 0x2DAAC0 | 25 |
| Ring travel speed (`arenaOrbitSpeed`) | float units/s | 0x2DAB54 | 5.0 (the speed at which a sister is moved along the ring during a dome hit, and used by the ring-slot helper; divided by 60 per tick) |
| Energy-ball damage (`DarkEnergyBallDamage`) | int | 0x2DAB58 | 25 |
| Slow / poison / curse missile damage (`SlowMissileDamage`, `PoisonMissileDamage`, `CurseMissileDamage`) | int | 0x2DAB5C / 0x2DAB64 / 0x2DAB6C | 10 each |
| Slow / poison / curse effect chance (`SlowMissileEffectChance`, `PoisonMissileEffectChance`, `CurseMissileEffectChance`) | int % | 0x2DAB60 / 0x2DAB68 / 0x2DAB70 | 100 each |
| Mote-eater damage (`MoteEaterAttackDamage`) | int | 0x2DAB74 | 20 |
| Leach drain damage (`LeachAttackDrainDamage`) | int | 0x2DAB7C | 50 |
| Thorn damage, thorn maximum hits (`ThornAttackDamage`, `ThornAttackMaxHits`) | int | 0x2DAB80, 0x2DAB84 | 10, 3 |
| Dome duration (`DomeExpandTime`) | float s | 0x2DAB88 | 40 |
| Dome shot interval (`DomeAttackMissileLaunchRate`) | float s | 0x2DAB90 | 3 |
| Group-attack damage threshold, three alive (`groupAttackDamageThreshold1`); read only by the unreachable dome-start test | float | 0x2DAB40 | 150 |
| Group-attack damage threshold, two alive (`groupAttackDamageThreshold2`); read only by the unreachable dome-start test | float | 0x2DAB44 | 80 |
| Stored, no reader found: mote-eater drain count (`MoteEaterDrainNumber`) | int | 0x2DAB78 | 8 (the drain's 8 picks are a fixed value, not this word) |
| Separate dome damage (`DomeAttackDamage`); read only by the dome set-up, which never runs in the shipped game | int | 0x2DAB8C | 100 |

The mote-eater drain count is registered but not applied. The dome damage word is passed to the dome object when the dome forms, so it has no effect while the dome never starts; the dome's shots use the energy-ball damage. The two damage thresholds are read only by the dome-start test, which never passes in the shipped game (see Group (dome) attack).

Fixed values in the behavior (code constants, not data words): orbit speed 10 units/s with 2 alive and 12.5 with 1 alive; melee-dodge trigger distance 7; "close enough" tolerance 0.5; dodge cooldown random 1–3 s.

Alternate-loop energy-ball constants (code constants unless a word is named): release-point lifetime 240 ticks; flight effect id 460 (the ball's only effect id; see Sounds and effects); turn blend 0.25 of the remaining angle per tick toward the aim point; the ball starts 2.5 units above the sister's attachment point; scale 1.0. Its speed and damage are the words 0x2DAB94 (0.5, the value the alternate loop writes at its start) and 0x2DAB58 (25).

### Controller state numbers

The sister controller (SPEC-0029 section 4) numbers its states 0 to 9. The table gives the number of each behavior in this spec. The mapping is High for states 2 to 4 (the state routines use the animation ids in this table) and Medium for the rest (matched by animation ids).

| Controller state | Behavior in this spec | Animations |
| --- | --- | --- |
| 0 | Idle | 1 |
| 1 | Move (orbit) | 1 |
| 2 | Dodge missile | 13 or 14 |
| 3 | Dodge melee | 15 or 16 |
| 4 | Energy ball | 17 |
| 5 | Status missile (kind 0 poison, 1 curse, 2 slow) | 18 (all kinds; action 18 maps to the same sister clip as action 17) |
| 6 | Special missile (kind 0 thorn, 1 leach, 2 Mote-eater) | 19 (the projectile pairs use 22, 21 and 20) |
| 7 | Group (dome) attack | 23 (phase 3), other indices per phase |
| 8 | Flinch | 23 or 24 |
| 9 | Dead | 26 |

## Behavior

### Shared bookkeeping (each tick)

- alive = the number of sisters not in the dead state. It is the run-time word at data 0x36DC64 (outside the file image), which a start-up initialiser sets to -1 at boot (SPEC-0026, SPEC-0029). It is recomputed once per pass: the bookkeeping keeps the last pass counter it saw (the game's pass counter at 0x368F98, +1 at the end of every pass) and recounts only when the counter has changed, so several sisters calling it in one pass recount once (High). When it reaches 0, the fight ends: the bookkeeping posts level event 2 (all sisters dead), once.
- The same bookkeeping stores 0 into the group-attack start flag (the word at data 0x36DC68) every frame before testing it, so the group attack never starts (see Group (dome) attack). High.
- The alive count is also the shared group word: the ring-slot rule (below) tests it against 3. The alternate loop that writes 3 to it at its start (see Controller routines) is overwritten by the next frame's recomputation.

### Per-sister states

| State | Controller state | Animations | Behavior |
| --- | --- | --- | --- |
| Idle | 0 | 1 | Wait for the chooser |
| Move | 1 | 1 | Orbit (below) |
| Dodge missile | 2 | 13 or 14 (random) | Until done, then idle |
| Dodge melee | 3 | 15 or 16 (random) | Until done, then idle |
| Energy ball | 4 | 17 | On the animation's release point, fire one ball at the hero: damage 25, speed by alive count. Then idle, with energy cooldown = random(min, max). (The alternate loop's energy ball is different; see the section below. Launch and aim of the main loop: see Main-loop launches.) |
| Status missile | 5 | 18 (all kinds) | Fire one missile of the sister's current kind (poison, curse or slow): damage 10, applies the status with 100 % chance. The projectile visual kind is 18 (poison), 19 (curse) or 17 (slow), separate from the sister's animation action. Then idle, with status cooldown set |
| Special missile | 6 | 22 (thorn), 21 (leach), 20 (mote eater) | One projectile: thorn 10 damage, leach 50, mote eater 20. A thorn shot leaves a thorn hazard that three hero strikes remove (see Main-loop launches). Then idle, with special cooldown set |
| Group attack | 7 | various | The dome sequence (below) |
| Flinch | 8 | 23 or 24 | When hit; until done, then idle |
| Dead | 9 | 26 | Until done, then removed; alive drops |

**Per-sister projectile kind.** On entry, the three controller routines initialize their shared kind selector at block +0x08 to 0, 1 and 2, respectively: class 13 uses kind 0, class 15 uses kind 1, and class 16 uses kind 2. The same selector selects both the state-5 status missile and state-6 special missile (SPEC-0029, Behavior 4). These assignments map class 13 to Malice (row 125, type 53), class 15 to Spite (row 126, type 54), and class 16 to Vanity (row 127, type 55), matching the placement data (SPEC-0029, Open question 138; High). Therefore their state-5/status and state-6/special pairs are Malice: poison/thorn, Spite: curse/leach, and Vanity: slow/Mote-eater.

**Chooser** (in idle or move only; High). Let f = HP / max HP. The first rule that holds applies:

- If f < 0.33 and the special cooldown is 0 or less: special missile (state 6).
- Otherwise, if f < 0.66 and the status cooldown is below 1: status missile (state 5).
- Otherwise, if the energy cooldown is below 1: energy ball (state 4).
- Otherwise: move (state 1).

So below 0.33 a sister picks the special missile when its cooldown has expired, and otherwise falls through to the middle band. All three sister controllers handle state 6. The cooldowns count down by 1 per tick unless level state 4 is set. The special cooldown is drawn uniformly from 1.25 to 3.0 s (75 to 180 ticks).

**Speed scale.** Before the state handler each tick, the sister's class routine writes P = 2 − f, where f = current HP / maximum HP. The NPC update has already reset P to 1.0 and applied status factors; this direct write replaces that accumulated P value. Clip re-time follows the class routine, so the rate for that tick is R = character speed scale × (2 − f). At character speed scale 1.0, the factor is 1 at full HP, 1.5 at half HP and 2 at zero HP. The root step evaluated at that rate is used by the later motion-record pass.

**Dodges** (each tick, unless dead or in a dodge cooldown):

- If the hero is within 7 units and roll(0–100) < melee-dodge chance(alive): dodge melee.
- Otherwise, for each of up to 6 hero projectiles nearby, if roll < missile-dodge chance(alive): dodge missile.
- Either dodge starts a random 1–3 s dodge cooldown.
- The two rolls: the missile roll runs only when a hero projectile passes a proximity test, and its success starts the missile-dodge state (High). The melee roll runs only while the hero is within 7 units, and its success starts the melee-dodge state (animation 15 or 16), which has no damage or movement of its own. No incoming hit is tested (Medium).

**Orbit** (move state):

- If the horizontal distance d from the sister to the hero (x and z only) is outside [15, 30], pick a target radius R = random in [15, 30] and move radially until |d − R| < 0.5. (Confidence: High for the band and the distance test.)
- Then circle the hero sideways in a random direction. On every tick on which the sister's body contact flag or her navmesh blocked flag (SPEC-0056) is set, the direction flips. The blocked flag stays set for 120 ticks after one push by a navmesh edge (or one restore onto the mesh), so a sister pushed by an arena edge reverses on each of those ticks, up to 120 flips, and makes no net sideways progress until the flag clears (quirk, reproduce; see Edge cases). (Confidence: Medium-High.)
- Speed (High): the path-following move step uses 12.5 units/s with 1 alive and 10.0 with 2 alive (code constants), and otherwise the word 0x2DAB50 (7.5 on disc); per tick it passes speed / 60 to the shared route step. Alternate loop A overwrites 0x2DAB50 with 10.0 (see Controller routines). In orbit mode, 0.1 units per tick changes the tangential waypoint; the actor then follows that waypoint through the same route step, subject to its speed and collision response. [High for waypoint and speed inputs; actual displacement depends on the route step.]

### Projectiles and damage to the hero (Confidence: High)

Every Sisters attack is a projectile in the shared enemy projectile pool (SPEC-0029 Behavior 9). None of them has a melee hit. The pool's hero-hit rule applies to all of them, so the hero's shield and armour reduce the damage (unlike the bosses' melee hits, SPEC-0029 Behavior 10.5).

| Projectile | Behaviour kind | Visual kind | Damage | Status (chance 100 %) | Other |
| --- | --- | --- | --- | --- | --- |
| Energy ball (main loop) | 10 | 16 | 25 | none | lifetime 1000 ticks, radius 1.0 |
| Energy ball (alternate loop) | 10 | 16 | 25 | none | element 6, lifetime 240 ticks, radius 1.0 |
| Slow missile | 11 | 17 | 10 | 3 (slow) | lifetime 1000 ticks, radius 1.0 |
| Poison missile | 12 | 18 | 10 | 2 (poison) | lifetime 1000 ticks, radius 1.0 |
| Curse missile | 13 | 19 | 10 | 1 (curse) | lifetime 1000 ticks, radius 1.0 |
| Mote eater | 14 | 20 | 20 | none | on a hit that is not blocked: the Mote-eater drain (below); lifetime 1000 ticks, radius 1.0 |
| Leach | 15 | 21 | 50 | none | on a hit: effect 4039 and a 50 MP drain (below); lifetime 1000 ticks, radius 1.0 |
| Thorn | 16 | 22 | 10 | none | lifetime 1000 ticks, radius 1.0; one hit on the hero, then a thorn hazard (see Main-loop launches) |

- **Release frame.** In region 1123, Malice main-table actions 17, 18 and 19 all map to clip 19. That clip releases at frame 14, which carries event bits 0x1 and 0x20. The main state-4 handler requests action 17; the state-5 handler requests action 18 for every kind.
- **Blocking and reflection.** Kind 10 is reflectable by a matching shield (SPEC-0029 Behavior 9): the ball is sent back at the sister and loses its damage. Kinds 11 to 16 are not reflectable: a blocked missile deals max(1, 0.25 D) and applies no status.

### Main-loop launches, element, status values and starting cooldowns (High unless marked)

The energy ball (state 4), the three status missiles (state 5) and the three special missiles (state 6) of the main loop are launched in the same way:

- **Launch point and aim.** The launch point is the sister's attachment point raised 2.5 units, the same point the alternate loop uses. The aim point is the hero's position raised 2.0 units. The main loop has no target selector and no random aim offset; those belong to the alternate loop only. The velocity is the unit vector from the launch point to the aim point times the speed word for the alive count: 0.75 with 2 alive, 0.9 with 1 alive, and 0.5 otherwise (3 alive, and also the boot value -1 and the value 0).
- **Release.** A projectile is launched on the first tick on which the release event (bit 0x20) is active or the animation's progress value is above 0.5, whichever comes first, provided the sister's fired flag is clear (progress rule: Medium-High). The flag is set at that moment, even if the pool has no free slot; in that case nothing is launched. The flag is cleared when the next attack state is entered.
- **Element.** Every main-loop projectile (energy ball, the three status missiles, thorn, leach and Mote-eater) carries element -1, meaning no element recorded. It is not 0. The alternate-loop ball is the only sister projectile with a positive element (6). Under the pool rules (SPEC-0029 Behavior 9) -1 counts as a nonzero element that equals no Mote element, so only an all-element shield Mote (kind 7) reflects a main-loop energy ball; the missiles of kinds 11 to 16 are never reflected. The breaks-objects flag is 0 for the main-loop projectiles (the alternate-loop ball sets it to 1).
- **Status values.** The poison, curse and slow missiles carry status ids 2, 1 and 3 with chance 100. The setup record's value field is never written by the sisters, so it keeps its default of -1, and the two extra words of the status request are also -1. The hero's status apply (SPEC-0036) reads -1 as the status's default duration: poison 120 s, curse 60 s, slow 60 s. Thorn, leach and Mote-eater carry no status.
- **Starting cooldowns.** When a sister's controller starts, it draws four cooldowns by the alive-count rule and sets the dodge cooldown to 0. The draw order is energy, status, group, special, each as a whole number of ticks (seconds x 60, truncated) drawn uniformly and inclusively; a draw whose two bounds are equal returns that bound without using the random stream. The selector for the energy ball treats alive -1 like 3 alive (bounds 150 and 300 ticks); the selectors for status, special and group give bounds of 0 and 0 for alive -1 and for alive 0. At the first start after boot the shared alive word is -1, so each sister starts with: energy cooldown a uniform whole number from 150 to 300, status cooldown 0, special cooldown 0, group interval 0, dodge cooldown 0 (High). With the alive word at 1, 2 or 3 the draws are energy 150 to 300, status 105 to 240 and special 75 to 180 (the disc values are equal for all three alive counts), and group per the table above. With the alive word at 0 all four are 0. The alive word keeps its last value between fights in one session (0 after a won fight), so a later start uses that value (Medium-High). Because the chooser needs the energy cooldown below 1, the status cooldown below 1 and the special cooldown at 0 or less, a first-start sister in the 0.33 to 0.66 band fires a status missile on her first chooser pass, while one above 0.66 cannot fire an energy ball for 150 to 300 ticks.
- **Thorn hits (Medium).** The thorn shot is an ordinary pool shot: its first hit on the hero deals 10 once and frees the slot (a ground hit or the end of its life frees it too). The "up to 3 hits" word (0x2DAB84 = 3) belongs to a second object, a thorn hazard from the five-record thorn pool (SPEC-0029 Layout 1a). On the first tick on which a thorn shot's slot is free again, the sister whose kind selector is 0 claims the first free hazard record (none free: nothing spawns) and places it at the shot's last position, with its height set by the ground probe started 8 units above it (Medium-Low). The hazard deals 10 (word 0x2DAB80) to the hero on contact (a body test of radius 1.0, word 0x2DCB20), then waits 120 ticks before it can hurt again. It grows for 60 ticks; after that each hero melee swing whose weapon volume (radius 1.0, length 2.5) reaches it counts one strike, once per swing, and the hazard is removed at the third strike. The other kinds (leach, Mote-eater) leave nothing behind.

**Mote-eater drain (kind 14; High, rules owned by SPEC-0034 3.13).** When a Mote-eater missile hits the hero and is not blocked, and no drain is running, the hit starts the drain. SPEC-0034 3.13 gives the full rules and data words; in outline:

- 8 picks from the twelve Mote ids 160 to 171, each a uniform whole number from 0 to 11 that steps to (pick mod 12) + 1 while it repeats an earlier pick. Quirk: a pick that steps past 11 is not a Mote and adds no slot, so the drain holds 7 or 8 Motes.
- The drain runs 1801 steps (30 s at one step per tick). It reveals one slot every 1801 / slots steps in whole numbers: 225 steps for 8 slots (257 for 7). Each revealed slot shows that Mote's effect above the hero and uses 20 charge of that Mote (SPEC-0021 Mote charge). A Mote whose charge reaches 0 is removed from the inventory and unequipped from the active hero's equipment slots.
- **Who steps it.** Each sister's control loop advances the drain once after its state handler, including while state 9 plays the death animation. The handler's terminal state-9 pass still runs this post-handler step; after that pass, the shared death path's base state 16 prevents further AI steps. The active group's rate therefore follows the sister loops still running, not the living-sister counter: a sister contributes throughout state 9 and one last time on its transition pass (Medium-High). With three, two or one loops running throughout, 1801 steps take about 10, 15 or 30 seconds, respectively. For the shipped arena region 1123, all three sisters' main action 26 selects clip 8: 50 frames with speed divisor S = 2. At baseline character and time scale 1, the HP-based rate gives a cursor advance of (2 − f) / 4 frames per tick, where f is current HP divided by maximum HP. The one-shot ends at frame 49, so the playback takes ceil(196 / (2 − f)) advances after it starts: 98 ticks at zero HP and 196 ticks when an overkill hit leaves full HP. A non-overkill lethal hit leaves zero HP; an overkill hit leaves the pre-hit HP, so its animation duration depends on that remaining fraction. The death-state duration therefore comes from the linked clip and current HP, not the alive counter.

**Leach (kind 15; High).** A Leach hit plays effect 4039 once at the hero's model point, then drains 50 MP from the hero (word 0x2DD458 = 50; MP does not go below 0, and the MP-empty rule runs). The missile's own damage is the separate word 0x2DAB7C (50). SPEC-0034 3.13 and SPEC-0029 (enemy projectile kinds 14 and 15) hold the shared rules.

### Taking damage (Confidence: High)

- A sister is hit by the hero's melee swing (the hero's attack is active and the hero's single-weapon reach test, SPEC-0029 Behavior 10.2, finds the hero's weapon touching the sister) or by a hero shot (each of the hero's six shot slots is tested; a shot that hits is removed).
- A sister in a dodge state (missile or melee), the flinch state or the dead state takes no hit.
- **Shield.** At the start of the fight the sisters are shielded (one shared flag for all three, reset to the shielded setting when the main controller starts). While shielded:
  - a melee hit deals no damage, plays the shield flash (one-shot effect 4008) at the sister and puts the hero into hit reaction 13;
  - a shot hit deals no damage, raises level event 1 and removes the shield for all three sisters, for the rest of the fight.
- **After the shield is gone**, every hit deals damage d (the hero's damage for that hit):
  - d is added to the sister's damage-taken sum;
  - if d is more than the sister's health, she dies (dead state);
  - otherwise the shared hit flash (one-shot effect 222) plays 1.5 units above her, she enters the flinch (animation 23 or 24, chosen at random), her health drops by d, and if her health is then below 1 she dies.
- Level events: 1 = shield broken (first shot hit), 2 = all sisters dead (see Shared bookkeeping).

### Group (dome) attack

**Start (shipped behavior: never; High).** Each frame the shared bookkeeping stores 0 into the group-attack start flag and then tests it. Every store to that flag in the boot program writes 0, so the chooser never enters the group attack, and the dome, its ring, its shots and its hit sub-state below are unreachable in the shipped game. A clean build reproduces this (no dome).

**Intended rule (held by the behavior but unreachable, because the flag it is gated on is always 0; recorded for reference; High for the test order and constants).** Start the group attack when all of these hold:

- every living sister's group-attack interval timer is 0 or less;
- at least two sisters are alive;
- the sum, over the living sisters, of the damage each took since her last group attack is at least 150 with three alive (word 0x2DAB40) or at least 80 with two alive (word 0x2DAB44).

Each sister's damage-taken sum is reset to 0 when her group attack ends.

The steps below describe the group attack as it would run if started:

1. **Begin**: every living sister moves to its slot on a ring of radius 25 around the ring centre. The slots are by sister index:
   - index 0 takes the ring centre + (25, 0, 0), a fixed offset along the world x axis;
   - index 1 takes the point 120 degrees around the ring when exactly three sisters are alive (the group word equals 3); otherwise it takes the ring centre + (−25, 0, 0), on the same world x axis;
   - index 2 takes the point 240 degrees around the ring.

   The ring centre is the hero by default, or the anchor object when the level selects one (Medium-Low for the selection). Speed: the movement speed word (7.5 units/s, the 3-alive value) divided by 60 per tick, with no alive scaling (High). Confidence: High for the lane of the offsets (world x), the radius and the 120 and 240 degree points.
2. **Start**: once all are in place, a dome of radius 20 (0.8 × 25) forms. The first shot waits a random 0–3 s.
3. **Attack**, for up to 40 s (2,400 ticks) while the dome stands: each sister fires an energy ball (25 damage) every 3 s.
4. **Hit**: a sister hit during the dome moves along the ring at the ring travel speed (5.0 units/s, divided by 60 per tick), then resumes attacking while the dome holds.
5. **End**: when the dome ends, each sister sets its next group-attack interval by alive count (30–45 s, 20–40 s, or immediately with 1 alive), resets her damage-taken sum to 0 and sets her energy cooldown, then returns to idle.

### Arena flow (module 1122, boss-fight instance 11; owned by SPEC-0043; Confidence: Medium-High)

The fight's arena is run by level script module 1122. In outline:

- **Entry.** A trigger (index 6, on entry) starts the sequence: movie 52, the boss-fight refill (instance 11, stage 0), the hero to locator 26, and level state 4 cleared.
- **Shield broken.** When level state 1 is set (the sisters' level event 1), movie 53 (15.97 s) plays, once.
- **All sisters dead.** When level state 2 is set: fade out (0.3 s), game states 321, 93 and 322 set, the boss-fight call (instance 11, stage 1), a blackboard message (type 2, value 50, world 151), world 151 set as the world to return to, and movie 54 (130.4 s).
- **Hero down (only from the entry until movie 53).** The arena script runs the hero-down check once per pass of its arena thread (one pass per tick, while game state 320 is set), on passes where no entry, shield-broken or boss-end step runs. The check acts only after the entry has finished and before movie 53 has played (SPEC-0043, module 1122, stage bits 1 and 3). In that window the test is: hero 1 (Roberta) is down (state 16) and item 50 is not held. When it holds and Roberta is the selected hero, the module sets hero swap mode 1, which locks the selection to Roberta; the death-state handler's switch to Tor is then refused, so timer E starts and sets the game-over flag after more than 1.5 s. When it holds and Tor is the selected hero, the module calls SetGameOver(0). This is not a swap to the other hero: either way Roberta down in that window ends the game, even with Tor alive. Tor going down is not tested by the module and follows the normal death rules. The test has the same shape as the one in module 1217 (High; SPEC-0055, section 4, and SPEC-0053 for swap mode 1). Before the entry has finished, and from movie 53 on (the shield-broken movie) to the end of the fight, the module makes neither call (High for the window).
- **Alternate loop flag.** Level flag 0 (the alternate loop, see Controller routines) is set by module 1174 (the earlier encounter with the Darkest Faerie at the Act 2 start, not the final fight, which is module 1432) at its setup, right after it creates its character slot. Module 1122 does not set it, so the normal Sisters fight runs the main loop (High for the 1174 call; Medium that 1122 never sets it).

### Shipped fight walkthrough (pack 1122; High unless marked)

The live Sisters fight uses the main loop, not the alternate loop. Pack 1122's trigger 6 plays movie 52, starts boss-fight instance 11 at stage 0, moves the hero to locator 26, and clears level state 4. Its live branch loads the sisters' region 1123: Malice, Spite and Vanity, each with 400 HP (SPEC-0029, monster rows 125 to 127). Pack 1122 does not set the alternate-loop flag; the earlier Act 1 encounter in pack 1174 does.

The shared shield protects all three sisters. Melee contact while it is raised causes the shield response without HP loss. A hero projectile hit breaks the shield globally and posts level event 1; the arena thread consumes that event by playing movie 53 once. Once the shield is down, each sister's health and chooser run independently. Their health bands are strict: below 66% enables the status missile when its cooldown is ready, and below 33% enables the special missile when its cooldown is ready. The special band is tested first. Since each starts at 400 HP, 264 HP is still outside the status band and 263 enters it; 132 HP is still outside the special band and 131 enters it.

The three kind selectors determine the distinct attack pairs: Malice uses poison and thorn, Spite uses curse and leach, and Vanity uses slow and Mote-eater. All their launched shots use the shared enemy projectile pool and its shield/armour hit rule. When the shared alive count reaches zero, the sisters post level event 2 once; the arena script fades out, sets game states 321, 93 and 322, ends instance 11 at stage 1, sets return world 151 and plays movie 54. The arena's special hero-down check only operates after entry and before movie 53; from that movie onward the ordinary hero and partner death rules apply (SPEC-0055).

Test vector: at 400 HP with the initial shared alive word -1, the status and special cooldowns start at zero but their health gates fail; the energy cooldown is drawn from 150 to 300 ticks, so the sister moves until it reaches zero. At 263 HP, with status cooldown below 1 and special not eligible, choose the status missile; at 131 HP with special cooldown at zero, choose the special missile ahead of status. The shield-breaking shot changes the shared shield state but does not subtract HP.

### Alternate-loop energy ball (state 2 of the alternate loop)

This is the energy ball used by the alternate loop (see the next section). Confidence: High for the values and the release rule; Medium-High for the aim point. Each tick while the state runs:

1. The sister turns toward the aim point P at 0.25 of the remaining angle per tick (applied on every tick of the state, before the animation logic). P is the hero's position raised 2.0 units. If the level's target selector names a specific target entity, that entity's position is used instead, and the 2.0 raise applies only to the default hero point.
2. If animation 17 is not playing, it starts. Otherwise, if it has not finished:
   - when the release event (event 0x20) reads active and this sister's fired flag is clear, one ball is launched and the flag is set;
   - the ball starts at the sister's attachment point raised 2.5 units, and flies in the unit direction from there to the aim point, at speed 0.5 (word 0x2DAB94) per tick;
   - its life is 240 ticks; its damage is 25 (word 0x2DAB58); its flight effect is id 460 and its other five effect ids are -1; its scale is 1.0; its owner is the sister; it is an enemy-pool projectile of behaviour kind 10, visual kind 16 and element 6 (SPEC-0029 Behavior 9), with a flag of 1;
   - the sister's per-shot field is cleared.
3. When animation 17 finishes: the state becomes idle, and the energy cooldown is set to twice an energy-ball cooldown draw (High).

The aim point is the hero's position raised 2.0 units unless the selector names another entity. The alternate-loop speed of 0.5 is the same value as the three-alive main-loop speed.

**Target flag and offset rule.** The alternate-loop decision draws a random integer from 0 to 100. If the roll is 40 or more, the target flag is 0; below 40, the target flag is 1 when a target entity exists (High). The second selector returns a vector. It is zero unless the target flag is not 1 or there is no target entity, and the hero's object type is one of 30, 34, 35, 38 or 47. In that case it returns a sum of two independent uniform draws in [−8.0, 8.0], each scaled along one of two normalized axes perpendicular to the unperturbed shot direction. Let d be the unit vector from the raised launch point to the selected aim point, and u the game's fixed reference vector: a = normalize(d × u), b = normalize(a × d), and the offset is r₁a + r₂b. The caller adds this to the aim point and recomputes the launch direction (High for the two draws and perpendicular plane; Medium-High for the ordered axis formulas; u = (0, 1, 0, 0), the startup world-up vector (SPEC-0027 OQ47).

### Controller routines and the alternate loop

- The three sisters run under behavior classes 13, 15 and 16 (SPEC-0029 section 3). Each has its own state block. Classes 13 and 15 use alternate loop B; class 16 uses alternate loop A. Placement data maps class 13 to Malice, class 15 to Spite, and class 16 to Vanity (SPEC-0029 OQ138).
- **Alternate loops.** When level flag 0 is set (bit test: High), Malice (class 13) and Spite (class 15) run alternate loop B instead of their main loops, while Vanity (class 16) runs alternate loop A (the fight wiring is Medium). Loop B does not write the shared alive word. Loop A writes 3 to the shared alive word at entry. Both loops have only idle, move and energy ball (the energy ball described above). In both loops the move step uses the movement speed word doubled while a sister is avoiding the hero (within 5.0 horizontally of her; see the movement speed item in the resolved list). Loop A overrides movement speed to 10, the "label 1" projectile speed to 0.5 and the "label 1" energy cooldown to 1.5–3 s. Class 13 has no unique alternate loop (SPEC-0029, Behavior 4; High).
- The main loop is the normal fight. Which loop the fight uses at a given moment is set by level flag 0 (Medium).
- **Alternate loop A overwrites two words.** Vanity's alternate loop A (class 16, run when level flag 0 is set) stores 10.0 into 0x2DAB50 (the 3-alive movement speed) and 0.5 into 0x2DAB94 (projectileSpeed1) when it starts, and nothing restores them. They keep these values for the rest of the session (High). The alternate loops run only in the Act 1 escape (pack 1174, main-story step 21), which comes before the normal Sisters fight (pack 1122, step 43). The words are program memory and are not saved, so the normal fight sees 10.0 as its three-alive speed when the escape has been played in the same session, and 7.5 when the session was started from a save made after the escape (Medium-High for the step order).

### Sounds and effects (Confidence: High unless marked)

**Sounds come only from the animation tables.** The sister controllers start no sound of their own. All three sisters use creature block 387 (SPEC-0031); the tables of regions 1123 and 1175 are identical. Every sound entry of a sister's main animation table names a cue of block 387 with start frame 0 and no keep-alive bit. So when a sister's main animation changes to an entry with a sound, that cue plays once, at the sister, on the first tick of the animation. A request of the animation that is already playing starts nothing. No sound plays while the sister is in base state 42 (the animation-sound step is skipped there; SPEC-0031).

Animation to cue of block 387, given as Malice / Spite / Vanity. Animations not listed have no sound.

- 4 (laugh): 0 / 7 / 14.
- 13 to 16 (dodges): 5 / 12 / 19.
- 17 (energy ball): 4 / 4 / 18.
- 18 (missile): 3 / 3 / 17.
- 19 (missile): 3 / 10 / 17.
- 20 (cast): 6 / 13 / 20.
- 23 (take damage): 1 / 8 / 15.
- 24 (take damage): 8 / 8 / 15.
- 25 and 26 (die): 2 / 9 / 16.

Quirks (reproduce): Spite's animations 17 and 18 play Malice's ball and missile cues (4 and 3). Malice's animation 24 plays Spite's take-damage cue (8). Cue 11 of block 387 (Spite's ball cue) is named by no table and no code, so it never plays.

**Effects started by the code.**

- Shielded melee hit: one-shot effect 4008 at the sister. It is two model emitters, 15 blue sparks, a sprite and a blue point light of radius 18.
- Damaging hit (the flinch path, not an overkill): the shared hit flash, one-shot effect 222, 1.5 units above her (SPEC-0029 Behavior 35, SPEC-0034).
- Energy ball, on both loops: pool visual kind 16 with flight effect 460 as its only effect id. The blocked or reflected hit, lifetime expiry, world impact and unblocked hit ids are all -1, so none of those events shows an effect (event roles: SPEC-0029 Behavior 9).
- Leach hit: effect 4039 at the hero (see Projectiles and damage to the hero).

**Clip effect lists** (SPEC-0034; Medium-High for which list belongs to which clip): the die animations 25 and 26 show effect 1200 at 0.9 of the clip (a purple burst with a light) and effect 4021 over the whole clip (a grey sprite trail).

## Edge cases and error handling

- Status ids passed to the hero: curse 1, poison 2, slow 3. Durations and effects belong to the player-combat spec.
- **No dome in the shipped game.** The group-attack start flag is always 0, so the dome never starts, whatever the timers and damage sums are. Under the intended rule a single surviving sister could not start it either (at least two alive are required), even though her group-attack interval is 0.
- **Shield until the first shot.** A hero who fights only in melee never breaks the shield: every melee hit is blocked and puts the hero into hit reaction 13. The shield falls only to a hero shot.
- **Overkill skips the flinch.** A hit whose damage is more than the sister's health kills her at once, without the flinch or the hit effect.
- If a sister would fire while her animation slot is busy, she returns to idle and starts the cooldown without firing.
- Alternate loop A changes the 3-alive speed word from 7.5 to 10 at its start and never restores it, so a main loop that runs later in the same session also moves at 10 with 3 alive. In story order the escape (pack 1174, step 21) runs first, so the normal fight (pack 1122, step 43) sees 10.0 in the same session and 7.5 after a restart from a later save.
- **Energy-ball speed depends on the loop.** The main-loop energy ball uses the speed for the alive count (0.5, 0.75 or 0.9). The alternate-loop energy ball always uses the word 0x2DAB94, which the alternate loop sets to 0.5 at its start, so its speed is 0.5 whatever the alive count.
- **Alternate-loop energy ball fires once per release.** The fired flag gates the launch. The alternate-loop decision clears it when it enters the energy-ball state, before the target roll. The energy-ball routine sets it at the launch and does not clear it.
- **Stored words are overwritten at runtime.** The alternate loop's start writes 10 to the movement-speed word, 0.5 to projectileSpeed1, 1.5 to energyBallTimeMin1 and 3 to energyBallTimeMax1. Nothing restores them, so they keep these values for the rest of the session, including any later Sisters fight. The tables above give the disc values.
- **HP-based animation rate (High).** Each sister's class routine writes P = 2 − f after the NPC update's reset and status processing, but before clip re-time. The value therefore changes that tick's clip rate and replaces the earlier status multiplier in P. A lower HP fraction speeds the clip up; the later root-motion pass uses the root step produced at that rate.
- **Mote-eater drain speed (quirk, reproduce; Medium-High).** The drain advances once per tick for each sister loop still being stepped, not once per living sister and not once per tick globally. A sister keeps advancing it during state 9's death animation, including once on the pass that changes her to base state 16; later passes do not run her AI. Three, two or one loops running throughout finish its 1801 steps in about 600, 901 or 1801 ticks. Do not normalize it to a fixed 30 seconds. In the main arena the death clip has 50 frames and S = 2; at baseline scales its duration is ceil(196 / (2 − f)) ticks after playback begins, where f is the sister's current HP fraction (Who steps it).
- **Mote-eater drain of 7 (quirk, reproduce).** A pick that steps past the last Mote id adds no slot, so some drains hold 7 Motes (SPEC-0034 3.13).
- **Low-HP band falls through.** Below 0.33 HP a sister whose special cooldown has not expired uses the middle band (status missile, then energy ball, then move).
- **Orbit flips while blocked (quirk, reproduce; Medium-High).** The orbit direction flips on every tick the body contact flag or the navmesh blocked flag is set, not once per push. One push by a navmesh boundary edge keeps the blocked flag set for 120 ticks, so it gives up to 120 flips: the sister jitters in place sideways (an even count ends on the starting direction) until the flag clears or she is pushed again. Do not turn this into a single reversal.

## Test vectors

- 3 alive: missile dodge 75 %, energy cooldown 2.5–5 s (150–300 ticks), projectile speed 0.5 units/tick, movement speed 7.5 units/s.
- 1 alive: missile dodge 50 %, projectile speed 0.9, movement speed 12.5 units/s, group interval 0.
- 2 alive: projectile speed 0.75, group interval 20–40 s.
- Dome (unreachable in the shipped game; for the intended rule only): 40 s at one shot per 3 s gives up to 13 or 14 shots per sister, 25 damage each.
- Dome-start rule (unreachable): 3 alive, all group timers 0, damage sums 60 + 50 + 39 = 149: no start; 60 + 50 + 40 = 150: start. 2 alive, sums 40 + 39 = 79: no start; 80: start. Shipped game: no start in any case.
- Shield: shielded, melee hit 20: no damage, hero hit reaction 13. Shielded, shot hit 10: no damage, level event 1, shield off for all three.
- Damage: unshielded, shot 10 on a sister with 30 health: flinch, health 20, damage-taken sum +10. Shot 31 on a sister with 30 health: dead at once, no flinch. Shot 30 on a sister with 30 health: flinch, health 0, then dead (health below 1).
- Mote-eater group during a death (synthetic, Medium-High): with three sister loops running and an active drain, the group advances by 3 on every tick while one sister is in state 9. On the terminal state-9 pass, the death handler changes her to base state 16 and the post-handler helper still advances the group by 3. On later passes, the group advances by 2. The living-sister count falls at state 9, before the group-step rate changes.
- Main-arena death clip (disc and synthetic): action 26 maps to clip 8 (50 frames, S = 2) for each sister. With baseline scales and HP fraction f = 0, the playback advances 0.5 frame per tick and ends after 98 advances; with f = 1 it advances 0.25 frame per tick and ends after 196 advances.
- A sister at f = 0.5 with the status cooldown at 0: status missile. At f = 0.8 with the energy cooldown at 30 ticks: move.
- f = 0.30, special cooldown 0: special missile (state 6). f = 0.30, special cooldown 5, status cooldown 0: status missile (state 5). f = 0.30, all cooldowns above 0: move.
- Controller kind assignment: class 13 initializes selector 0 (poison/thorn), class 15 initializes 1 (curse/leach), and class 16 initializes 2 (slow/Mote-eater).
- 2 alive: the path step moves 10 / 60 units per tick. 1 alive: 12.5 / 60.
- Orbit: a sister whose horizontal distance to the hero is 14 is outside the band and picks a new radius in [15, 30]. At 20 it does not.
- Orbit waypoint and route-speed inputs (synthetic): at position (0, 0, 0) with unit tangent (0, 0, 1), the orbit waypoint is (0, 0, 0.1). With three, two or one sister alive, the shared route step receives speed scales 7.5/60 = 0.125, 10/60 ≈ 0.1667 or 12.5/60 ≈ 0.2083 units per tick respectively. These are independent inputs; the final actor displacement is governed by the route step and collision response.
- Orbit flip: the blocked flag is set at tick n by one edge push and nothing pushes the sister again. The direction flips on ticks n + 1 to n + 120, 120 flips, ending on the starting direction.
- Alternate loop (level flag 0 set, class 15 or 16): a sister with 3 alive moves at 10 units/s, not 7.5, and at double that speed (20 units/s) while she is avoiding the hero.
- Alternate-loop energy ball (synthetic): the release event reads active on two ticks in one animation with the fired flag clear on the first. One ball is launched on the first tick and the flag is set, so the second tick launches nothing. A ball launched with the aim point 10 units away and the sister's attachment point at the origin moves 0.5 units per tick in that direction, with damage 25 and life 240 ticks.
- Alternate-loop energy-ball cooldown: an energy-ball draw of 100 ticks gives a 200-tick cooldown.
- Arena hero-down window: entry finished, movie 53 not yet played, hero 1 dead, item 50 not held, Roberta selected: swap mode 1 is set, the switch to Tor is refused and timer E sets the game-over flag on its 91st tick (more than 1.5 s). The same with Tor selected: SetGameOver(0) at once. Hero 1 dead with item 50 held: neither call. The same after movie 53 has played: neither call from the arena module.
- Arena region selection: flag 31 set installs the Sisters arena trigger and loads set 0 from world pack 1122 (character region 1123); flag 31 clear and flag 30 set loads set 1 (1124); both clear loads set 2 (1125). The wave thread loads sets 3, 4 and 5 (regions 1126, 1127 and 1128).
- Pool rule (synthetic): a mote eater (kind 14, damage 20) blocked by a guarding hero deals max(1, 0.25 x 20) = 5. A curse missile (kind 13, damage 10) blocked applies no curse; the same missile unblocked on a hero with no shield and no armour deals 10 and applies status 1. An energy ball reflected by a matching shield flies back at the sister with damage 0.
- Mote-eater drain, 8 slots, one sister alive: reveals on ticks 1, 226, 451, 676, 901, 1126, 1351 and 1576, stop on tick 1801. Two sisters alive: last reveal on tick 788, stop on tick 901 (SPEC-0034 3.13).
- Mote-eater drain on a Fire Mote with charge 15: one reveal uses 20, so the Mote is removed and unequipped. With charge 100 it keeps 80.
- Leach hit on a hero with 30 MP: effect 4039, MP 0. With 120 MP: MP 70.
- Sounds (disc tables): Spite's main animation changes to 17: block 387 cue 4 plays once at her on that tick. Malice's changes to 24: cue 8. Vanity's changes to 25: cue 16. A sister already in animation 23 that is asked for 23 again: no new cue. No table entry names cue 11.
- Effects: a shielded melee hit plays effect 4008 at the sister. An unshielded shot of 10 on a sister with 30 health plays effect 222 1.5 units above her, then the flinch. A Sisters energy ball blocked by the hero without a reflection: its blocked-hit id is -1, so no effect plays.
- Ring: sister index 2 takes the 240 degree slot at radius 25 around the hero (a point 25 units from the hero, 240 degrees from the reference direction). Sister index 1 takes the 120 degree slot only while exactly three sisters are alive (the group word equals 3).
- Ring offsets (unreachable in play; synthetic): ring centre (10, 0, 4). Index 0 goes to (35, 0, 4). Index 1 with two sisters alive goes to (−15, 0, 4).
- Missile life: a poison missile launched at tick 0 that touches nothing is removed by its lifetime at tick 1000 (pool lifetime rule, SPEC-0029 Behavior 9); its hit test uses radius 1.0, the same as the main-loop energy ball.
- Main-loop energy ball (synthetic): attachment point (0, 0, 0), hero at (10, 0, 0), 3 alive. Launch point (0, 2.5, 0), aim point (10, 2, 0), direction (0.99875, -0.04994, 0), velocity (0.49938, -0.02497, 0) per tick, damage 25, element -1, life 1000.
- Poison missile (synthetic): hero unshielded and unarmoured, hit not blocked, the status roll passes: the hero takes 10 and is poisoned for the default 120 s. A curse missile gives the default 60 s and a slow missile the default 60 s.
- Starting cooldowns (synthetic): first start after boot (alive word -1): energy cooldown in 150 to 300, status 0, special 0, group 0, dodge 0; only one random draw is made per sister. Alive word 3: status in 105 to 240, special in 75 to 180.
- Thorn (synthetic): a thorn shot hits the hero for 10 and a hazard appears at the shot's last position. After the 60-tick growth, three separate hero melee swings that reach it remove it; two swings do not.

## Open questions

- **Default if unresolved (rewrite choice).** If the thorn hazard's spawn height cannot be settled, place it at the ground height under the shot's last position. If a later start needs the alive word and it is unknown, use 0 (every cooldown starts at 0).
- **Resolved (R-0065).** Main-loop launch point (attachment point + 2.5), aim point (hero + 2.0), element (-1), status value (-1 = default duration), starting cooldowns and the thorn hazard are in Main-loop launches (High; thorn lifecycle Medium).
- **Group-attack trigger.** Resolved: the dome never starts in the shipped game. The bookkeeping stores 0 into the group flag (data 0x36DC68) every frame before testing it, and only zero-stores to it exist. The unreachable intended rule (all living sisters' group timers at 0 or less, at least two alive, damage sum since the last dome at least 150 with three alive or 80 with two alive; words 0x2DAB40 and 0x2DAB44) is given under Group (dome) attack (High).
- **Normal loop versus alternate loop.** Resolved: level flag 0 is set by module 1174 (the Act 2 earlier encounter with the Darkest Faerie) at its setup, and the Sisters arena module 1122 does not set it, so the normal fight runs the main loop (High for the 1174 call; Medium that 1122 never sets it). Resolved: class 13 follows alternate loop B, shared with class 15, when level flag 0 is set. The class-13 startup diagnostic names Malice, consistent with the placement mapping; no unique class-13 loop remains open (SPEC-0029 Behavior 4 and OQ9).
- **Hero down after movie 53.** Resolved: the arena module stops its hero-down check (swap mode 1 or game over) once movie 53 has played, because the movie sets its stage bit 3 and nothing clears that bit; the module registers no death callback and writes the swap mode nowhere else. For the rest of the fight the ordinary death rules apply (SPEC-0055, section 8A). The swap mode then is the one the story left, which is 2 (free) at this point of the story (set by the act-start setup and the Werelupe King scripts, SPEC-0053 section 3.7; Medium-High). So when Roberta is selected and falls, her death handler requests a switch to Tor at 0.95 of her death clip and the request is accepted while Tor lives: the fight goes on with Tor, and Roberta stays down at 0 HP in state 16 until a Potion of Revival, a boss refill, a save load or the Continue reset (a world load does not revive a parked hero; SPEC-0055, section 2A). When Tor is selected and Roberta is already down, nothing ends the game. When both heroes are in state 16, the death-state handler starts timer E and the game-over page follows after more than 1.5 s. Test vectors: Roberta selected, movie 53 played, Tor alive, lethal hit on tick 500: no game over, Tor selected from about tick 500 + 1 s; the same hit before movie 53 with no item 50: swap mode 1 or SetGameOver(0), game over. Tor then falls after movie 53 with Roberta down: death mode 1, timer E, game over 91 additions later (SPEC-0043, open question 30).
- **Arena module and character region.** Resolved in part: module 1122 is the Sisters' arena (Medium-High). Its arena thread and entry trigger are installed only when flag 31 is set; on that live-fight branch its slot thread loads set 0, which maps to character region resource 1123 for world pack 1122 (SPEC-0043 module 1122; SPEC-0030 row 205; High for the mapping). The same Init routine selects set 1 / region 1124 when flag 31 is clear and flag 30 is set, or set 2 / region 1125 when both are clear. The three combat waves load sets 3, 4 and 5 (regions 1126, 1127 and 1128) in the second slot.
- **Ring lane and centre rule**: Resolved for the lane: the +25 and −25 offsets are on the world x axis (see Group (dome) attack; High). The ring centre is the hero by default; the selection between the hero and a second object is Medium-Low.
- **Chooser versus controller.** Resolved: below 0.33 the chooser picks the special missile (state 6) when the special cooldown is 0 or less, and otherwise falls through to the middle band; all three sister controllers handle state 6 (High).
- **Special missiles at low HP.** Resolved: as above.
- **Move speed by alive count.** Resolved: 12.5 units/s with 1 alive, 10.0 with 2 (code constants), otherwise the word 0x2DAB50 (7.5 on disc, 10.0 after alternate loop A has run) (High).
- **Animation 17**: Resolved in part. The state-4 handler requests sister main action 17 and state 5 requests action 18 for every missile kind. In region 1123, all three sisters’ main tables map both actions to clip 19, so the sister uses the same clip in both attack contexts. “17” for the slow missile is the projectile pool visual kind, not a sister action. The clip’s authored design label is not established.
- **Animation 23**: resolved for shipped play. The flinch state can request animation 23 (its other variant is 24). The group attack requests 23 only in phase 3, but that attack is unreachable because the group-start flag is reset to 0 before it is tested. Thus the reachable class-13/15/16 controller path that requests animation 23 is flinch; the clip's authored design label remains unknown.
- Resolved: the kind-selector byte maps 0/1/2 to poison/curse/slow in state 5 and thorn/leach/Mote-eater in state 6; controller classes 13, 15 and 16 initialize it to 0, 1 and 2, respectively (SPEC-0029 Behavior 4; High). Resolved (High): class 13 is Malice (row 125, type 53), class 15 is Spite (row 126, type 54), and class 16 is Vanity (row 127, type 55), matching the placement data (SPEC-0029 Open question 138). Combined with the selector mapping above, Malice uses poison/thorn, Spite curse/leach, and Vanity slow/Mote-eater.
- **Projectile lifetimes and hit radii.** Resolved: every sister projectile has radius 1.0; the main-loop energy ball and all six status and special missiles live 1000 ticks, and the alternate-loop ball 240 ticks (see the projectile table; High).
- **Fired-flag reset.** Resolved: the alternate-loop decision clears the flag when it enters the energy-ball state, before the target roll (High). The decision runs when the energy cooldown has counted down to 0 or less, no launch is pending and the sister is not in her avoiding mode (below): it then sets state 2, clears the fired flag and only then draws the target roll. A flag left at 1 from an earlier ball therefore cannot suppress the next ball. Test vectors: flag 1 from the last ball, cooldown 1 at the start of the tick: the cooldown reaches 0, the state becomes 2 and the flag is 0 before the release event is read; the release event on a later tick of that animation then launches one ball and sets the flag to 1. In the avoiding mode (hero within 5.0) the decision does not run and the flag is not touched.
- **Category values of the alternate-loop ball.** Resolved: the three values 10, 16 and 6 are the pool behaviour kind (10, a straight reflectable shot), the visual kind (16) and the element (6) of the projectile (SPEC-0029 Behavior 9) (High).
- **Second selector.** The alternate-loop launch uses it (see the target flag and offset rule). The magnitude and axis construction are described below; the fixed reference vector resolves to startup world up (0, 1, 0, 0; SPEC-0027 OQ47).
- **Alternate routine classes.** Resolved (High; SPEC-0029 Behavior 4): Spite (class 15) and Malice (class 13) use alternate loop B and do not reset the shared alive word; Vanity (class 16) uses alternate loop A and sets the shared alive word to 3 on entry.
- **Random aim offset (second selector).** Resolved in part: the offset is r₁a + r₂b, with two independent uniform draws r₁ and r₂ in [−8.0, 8.0], a = normalize(d × u), and b = normalize(a × d), where d is the unperturbed shot direction and u is the fixed reference vector. The axes are perpendicular to d. Resolved: u = (0, 1, 0, 0), the startup world-up vector recorded in SPEC-0027 OQ47. The ordered axis formulas remain Medium-High because they are established from the controller data path, without an independent runtime trace (High for draw bounds and perpendicular plane).
- **Orbit step.** Resolved: the 0.1 scale advances the tangential waypoint, while the actor follows that waypoint through the same path step as the radial approach. That path step receives speed / 60, selected by the alive count. The actor's resulting displacement can be smaller or otherwise changed by route and collision handling. [High for both inputs; see Orbit.]
- **Dodge cooldown unit.** Resolved: the cooldown is a random whole number from 1 to 3 times the frame rate (60 ticks per second, set once at start-up; see SPEC-0007), so it is 60, 120 or 180 ticks (1, 2 or 3 s).
- **Movement speed or orbit speed.** Resolved: the word is the movement speed of the path-following step, in units per second; the per-tick step is the word divided by 60. Three routines read it: the main-loop move step (the word as it stands when 3 sisters are alive, 10.0 with 2 alive and 12.5 with 1 alive), the group-attack ring step (the word, with no alive scaling) and the alternate loops' move step (the word, doubled while the sister is in her avoiding mode, the mode word at +0x30 of her block being 2; the mode is entered when the hero is within 5.0 horizontally of her and ends when she is within 2.0 of the point she runs to or is blocked; High for the reads and the doubling, Medium for the mode conditions). The table name is kept; the test vectors say movement speed. Test vectors: with the word at 7.5 and 3 alive, the main-loop step is 0.125 per tick; with 2 alive 10.0 / 60 = 0.16667; with 1 alive 12.5 / 60 = 0.20833; the group-attack ring step is 0.125 per tick at 7.5. After alternate loop A has set the word to 10.0, an alternate-loop move step is 0.16667 per tick, and 0.33333 per tick while she is avoiding the hero. In an alternate loop of class 13 or 15 with the word still 7.5 the step is 0.125 and 0.25 while avoiding.
- **Projectile word: speed or scale.** The launch velocity is the unit direction times the selected word, so the word is a speed in the launch-velocity units.
- **Drain after a sister's death.** Resolved in part: each sister loop advances the active drain through state 9 and once on its terminal pass, then stops on later passes after the base-state-16 transition. In the main arena, action 26 selects a 50-frame clip with S = 2; at baseline scales the duration from the first playback advance is ceil(196 / (2 − f)) ticks, where f is current HP divided by maximum HP (Who steps it). The exact runtime duration also depends on the sister's HP after the lethal hit and the playback-start tick order (Medium-High).
