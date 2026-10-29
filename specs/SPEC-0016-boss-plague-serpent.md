# SPEC-0016: Boss behavior: Plague Serpent

- Answers: proactive, none requested (boss AI); proactive (fight AI build guide); R-0096
- Status: Draft
- Disc build: SCUS-97367 v1.00 (same fingerprint as SPEC-0001)
- Confidence: per section. Tunable defaults and addresses: High (taken from the boot program image). Health phases and the bite/belch engagement rule: High. The engagement gates (height above 115.0, distance below 20.0, the suppression by anchor slot, and the no-op while biting): High for the comparisons, High for the distance metric (horizontal, from the shared helper; Open question 16). The hero action code 30 check: High for the comparison; the code is the hero's own state number, so 30 is the ledge hang of SPEC-0033 (Medium-High; Open question 10). Anchors and relocation: High for the rules and for the arrival placement (point A or C at the probed ground height). Phase change: High for the two-pass order, the 0.99 facing test, the 0.1 turn blend, the target (point B of the stored slot), the slot writes at the start of a change, animation 21 and the phase increment; High for the starting slot values (none); High for setting the matching level-state flag and its swamp break-scene response (see Arena break-scene response); Medium-High that the soft-lock is not reachable in normal play (Open question 13). Bite, belch, mesmerize and stun sub-states: High for transitions, Medium for effect payloads. Stun strike damage: High for the shape, Medium for "normal attack damage". Poison cloud: High for the movement, Medium for the damage meaning (the damage itself is the standard poison status). Tail slap: the routine is fully described (High for its rules); it is unreachable in the known routines (Medium-High, see Open questions). Lunge weapon, window, raw damage and block: High. Neck look (6A): High for the rule, Medium-High overall; whether the neck visibly turns in play is open. Sounds and effects (11B): High for the cue list, the trigger and the effect ids; Medium for the exact attach points of effect 413 and the bite flash. Lunge weapon position and reach: Medium-High (model scale 1). Block predicate, selected-hero rule, random draw rules, strike damage factors and the hero-side table: High. Controller start before the release: Medium.

## Summary

The Plague Serpent is a boss that sits on one of eight arena anchors. Each anchor is a set of three level points. Each tick it re-checks which anchor is nearest the hero and relocates there when that changes. When the hero is close and the serpent is not suppressed, it lunges with a bite, sometimes preceded by a mesmerize set-up that stuns the hero. When the hero is not in bite range, it belches a poison cloud on a random 5 to 7 second cooldown. Health at or below 75, 50 and 25 percent advances a phase counter from 1 to 4, and each advance plays a turn-and-roar phase change that faces two stored anchors in turn. The stage value is always 0 in this build, so only the stage-0 values apply to the bite decision and the stun. No known routine enters the tail-slap or wait state, and they are not to be made reachable. The serpent has 100 health and is invulnerable to every ordinary hero hit at all times: the only way to hurt it is the single head strike allowed while it is stunned, and the only way to stun it is to block its lunge. It never walks; it submerges and reappears at the anchor nearest the hero.

## Fight AI build guide

This is the ordered recipe for writing the serpent's per-tick code. Details and confidence for each item are in the sections named. Time unit: one tick = 1/60 s. All of the serpent's clips have speed byte 1 and the play rate is 1.0 except in the bite windup, so every clip advances 0.5 key per tick. "Request tick s" is the tick whose state handler asks for an animation; "gate tick" is the tick s + j on which the same handler first sees that clip at 0.99 progress or more. For these clips j = 2F - 1 for a one-shot clip of F keys below 50, and j = 2F - 2 for F of 50 or more (so 46 keys give 91, 61 keys give 120). Event frames: a handler on tick s + j reads the event bits of key floor(j / 2). Confidence High unless marked.

**1. Initial values** (Layout, Per-fight state; Arena hand-offs).
- Health 100 of 100 (the serpent's monster row: radius 1.175, height 3.5; lunge weapon 3 radius 5.0 and length 5.0). State 0 (idle), stage 0, armed 1, anchor slot 0, anchor flag 0, relocation variant 0, phase 1, belch cooldown 0, phase-change slots both none, play rate 1.0. Tunables come from the boot words in Layout.
- The fight starts when the intro script releases the serpent. The script has put it at slot 0's point C (level object 104) and played animation 3 for 1.0 s and animation 4 for 1.0 s under script hold. Start the controller at the release tick with the values above and the hero's position at that tick (rewrite choice, Medium). The original differs in one way: its controller coroutine is created when the intro activates the character, which is at least 120 ticks (the 0.5 s fade in and a 2.0 s wait) before the script moves the serpent to slot 0's point C, and the controller then keeps running through the hold (at least another 120 ticks of script animation, plus the camera and letterbox steps). It makes its ordinary decisions for the hero's position at those times; during the hold its animation requests are stored and not applied, its checks read the clip the script is playing, and its own placement steps still move the serpent. The rewrite's single first decision at release is the same kind of decision (relocate, belch or bite), but in the original the first cloud may already have left, and the controller's slot, flag and cooldown may differ from the initial values (Open questions, Default if unresolved).

**2. Per-tick order** (Behavior 1). States: 0 idle, 1 move, 4 belch, 5 relocate, 6 arrive, 7 bite, 11 phase change; 2 and 3 are never entered. "The hero" everywhere in this spec is the currently selected hero, read again on every tick (switching heroes mid-fight changes every test from the next tick; the parked partner is ignored). The controller runs as a character think, before the projectile pool update and before the selected hero's update in the same tick (SPEC-0007), so every hero value it reads (position, action state, guard timer, attack-active word, shot slots, posed weapon) is the one left at the end of the previous tick, and a cloud it launches on tick g is already updated by the pool on tick g.
1. Play rate := 1.0 and the invulnerable word := 1. The serpent is never a hit candidate for hero swings, hero shots or spell areas (Hurt side).
2. Belch cooldown minus 1 when above 0.
3. Phase check (Behavior 2), unless the state is 1, 5, 6, 7 or 11.
4. Engagement: bite or belch (Behavior 5), under the same state gate, and only while the state is not 7.
5. Anchor chooser (Behavior 4), in states 0, 4 and 6 only. It can replace a state chosen in step 4 in the same tick: a belch chosen by step 4 is replaced by a relocation when the nearest anchor differs from the current one.
6. Run the handler of whatever state exists after steps 3 to 5. A state chosen this tick runs its handler this tick.
7. Presentation (not covered); wait one tick.

**3. State table** (j counted from the request tick s).

| State | Action id and clip (keys) | Duration | Exit |
| --- | --- | --- | --- |
| 0 idle | 5, clip 0 (60, loop), requested every tick; neck look every tick; no body turning | until step 3, 4 or 5 picks something | phase change, bite, belch or relocate |
| 1 move | 3, clip 3 (46) | j = 91; turns toward the anchor axis at blend 0.25 per tick | state 0 on the gate tick |
| 4 belch | 10, clip 5 (91) | cloud at the first tick with a free pool slot among j = 34 to 135 (normally 34); gate j = 180 | state 0 on the gate tick; cooldown := random whole number in [300, 420] |
| 5 relocate, variant 0 | 4, clip 2 (61) | j = 120 | state 6 on the gate tick |
| 5 relocate, variant 1 | 6, clip 4 (41), then 4 as variant 0 | j = 81 for animation 6, which clears the variant; animation 4 is requested on the next tick (s + 82) and gates at s + 202 | state 6 on that gate tick |
| 6 arrive | none | one tick: the chooser places the serpent at point C (flag 0) or A (flag 1) of the current slot, then state 1 in the same tick, whose handler requests animation 3 | state 1 |
| 7 bite | sub-states below | | |
| 11 phase change | 5 while turning, then 21, clip 1 (91) per pass | per pass: turn ticks, then j = 180 | state 0, phase + 1 after the second pass |

Bite sub-states (Behavior 8). Sub 0 runs on the engagement tick itself; every other sub-state runs from the next tick.

| Sub | Action id and clip (keys) | j | Exit |
| --- | --- | --- | --- |
| 0 decide | none | same tick | sub 1 (roll below the phase chance: 0 / 40 / 60 / 0 for phases 1 to 4) or sub 2 |
| 1 mesmerize | 20, clip 15 (81) | hero stunned for 300 ticks at j = 52 when not already stunned; gate j = 160 | sub 2 (also at once if the hero is jumping or has action code 30) |
| 2 windup | 11, clip 6 (45) at play rate m (the drawback draw) | gate j: m 1.0 gives 89, 1.1 gives 81, 1.2 gives 74; 0.9 gives 98; 0.8 gives 110 (plus or minus 1) | sub 4 on the gate tick; sub 3 when the hero has blocked more than 30 consecutive ticks |
| 3 spit | 12, clip 7 (71) | cloud at the first free-slot tick in j = 20 to 105 (normally 20); gate j = 140 | state 5 on the gate tick |
| 4 lunge | 13, clip 8 (49) | weapon 3 tested on j = 38 to 51; gate j = 97 | landed: sub 5 at once; blocked: sub 6 at once; otherwise sub 10 on the gate tick and state 5 on the next tick |
| 5 landed | 14, clip 9 (27) | j = 53 | state 5 on the gate tick |
| 6 blocked recoil | 22, clip 10 (41) | j = 81 | sub 7 with the stun counter at 300 |
| 7 stunned | 23, clip 11 (52, loop), requested every tick | up to 300 ticks | strike: sub 8 at once; counter below 1 (the 301st tick): sub 9 |
| 8 after strike | 28, clip 16 (108), requested on the strike tick | j = 214 | state 6 on the gate tick |
| 9 recover | 24, clip 13 (41) | j = 81 | state 5 on the gate tick |

**4. Choosing between actions** (Behavior 5, Behavior 2, Behavior 4), every tick in the allowed states, in this priority.
1. Phase change when the health band differs from the stored phase (Behavior 2).
2. Bite candidate (strict comparisons): hero feet height above 115.0, horizontal distance from the serpent to the hero below 20.0, current anchor slot not suppressed (slot 0 always; slot 3 unless phase 4). If a candidate, the hero's action code is not 30 and the state is not already 7: start the bite. If the hero's action code is 30, nothing at all happens (no belch either).
3. Otherwise, belch when the cooldown is 0 and the state is not 4.
4. Then the anchor chooser, in idle or belch only, starts a relocation when the nearest anchor (horizontal distance to point B, lowest slot on ties) is not the current slot. This replaces a belch chosen in step 3 or a belch in progress; it never replaces a phase change or a bite.
Chances are rolls per decision, not per tick: the mesmerize roll is an integer 0 to 100 inclusive drawn once per bite start (even when the chance is 0 or the hero is jumping) and succeeds below the chance, so 40 succeeds with probability 40/101. The belch has no chance, only the cooldown. All draws use the shared generator (SPEC-0008, Random numbers): an integer draw in [lo, hi] is lo plus the unsigned output modulo (hi - lo + 1); a float draw in [lo, hi) is lo plus the unsigned output times (hi - lo) / 2^32, never reaching hi; equal bounds draw nothing. Draw sites and order: the anchor flag (0 or 1) when the chooser starts a relocation; at a bite start the mesmerize roll first, then, only when no mesmerize follows, the drawback float; the belch cooldown (300 to 420) when a belch ends.

**5. Movement.** No walking, no navigation mesh, no root motion (every clip's root record is zero). The position changes only at arrival. Facing: idle never turns; bite sub-states 0, 1 and 2 turn toward the hero at blend 0.25 per tick, so the lunge fires along the heading the serpent had when the windup ended; the move state turns toward the anchor axis at 0.25; the phase change turns toward the stored anchor at 0.1. A hero that is far away, low, or unreachable is simply belched at; the serpent still relocates to the anchor nearest the hero whatever the distance.

**6. Hit and hurt windows.** Lunge: weapon 3 (radius 5.0, length 5.0) on lunge-clip keys 19 to 25 = ticks j = 38 to 51 after the lunge request; first overlap on a non-blocking hero deals 20 raw (not reduced by hero defences), sends the hero to its get-up state (54), ends the lunge. The weapon is centred on the serpent's third attach point (the jaw joint, joint 17, plus 2.09 along its forward axis and 0.65 down), which on keys 20 to 25 sits about 9.6 to 10.0 units ahead of the serpent's origin and about 6.8 up, with the cylinder axis horizontal and forward. It is tested against every hurt capsule of the hero (not the hero's hittable test), so a hero in the air, in hurt state 17 or in state 54 is hit when a capsule overlaps; the cylinder spans about 7.1 to 12.5 units ahead, 5.0 to each side and about 1.7 to 11.8 up. In practice the lunge reaches a hero whose centre is within about 13 units ahead of the serpent and within 5.0 of its heading; a hero 14 to 20 units away is a bite candidate but is never hit (Medium: model scale 1). Block: weapon overlap while the hero is blocking (guard predicate, Hero actions table) and dot(u, -f) above 0.5 gives no damage and a 5 s serpent stun. Spit and belch deal no hit damage; the cloud poisons (15 s) within 6.0 horizontal units. Hurt side: the serpent cannot be hurt except by one head strike per stun, during sub 7 (Behavior 8, Stun strike). Stun window: 300 ticks after the 81-tick recoil.

**7. Phase changes** (Behavior 2). Checked only in states 0 and 4, so only after a stun strike has run its course (state 8, arrive, move, then idle). Health fraction at or below 0.75, 0.5, 0.25 wants phase 2, 3, 4; a change runs two passes at the current anchor's two slots and lasts 362 + n ticks, n the second turn (15 to 24 ticks; Test vectors). A health drop past two thresholds chains two changes.

**8. Arena hand-offs** (Arena hand-offs, SPEC-0044 module 735). Intro (level state 8) refills the heroes and releases the serpent after its two script animations; the phase change posts level states S1 then S2 at the animation event; the swamp's break scenes answer them. Victory (level state 11) follows the serpent's death.

**9. Death.** Health 0 (reached only by a stun strike) runs the shared death hook: base state 16, the controller is no longer stepped, level state 11 is set by the fight monitor, the victory scene plays (Arena hand-offs).

**10. What a port gets wrong easily.**
- Treating the serpent as hittable. It is invulnerable on every tick; spells and ordinary hits do nothing.
- Playing clip 0 (or any clip) for ids 0, 1, 2, 7, 8, 9: they have no clip, but the serpent never requests them.
- Relocation variant 1 takes two clips (hiss, then submerge): 202 ticks to arrive, not 81.
- The anchor chooser runs after the engagement step and overrides a belch. On the very first tick of the fight a belch is replaced by a relocation unless the hero is nearest slot 0, so the fight opens with a submerge, not a belch (a bite candidate is not replaced).
- Windup length depends on the drawback draw (74 to 110 ticks); every other duration is fixed.
- The mesmerize roll and chance are per bite start, and phase 1 and 4 never mesmerize.
- Distance is horizontal (height only matters for the 115.0 gate); all comparisons are strict.
- Keys per tick is 0.5, not 1: a 91-key clip lasts 180 ticks.
- Engagement does not run in state 1, 5, 6, 7 or 11, and the phase check does not run there either; both can only start something from idle or belch.

**11. Edge cases for implementers** (settled points that are easy to miss; High unless marked).
- Every relocation that does not come from the anchor chooser (after a spit, a lunge, a landed bite, the end of a stun) is the 120-tick submerge at the same slot and the same anchor flag, so the serpent comes back up at the same point. Only a chooser relocation changes the slot and redraws the flag, and only a chooser relocation can take the longer hiss route (variant 1, 202 ticks).
- After a stun strike the serpent goes to arrive, not relocate: it is put back at the current slot's point and rises (91 ticks), and the state is idle again on strike tick + 306. Its next decision is the phase check, then engagement, then the chooser.
- The chooser never runs in the bite, relocate, move or phase-change states. A hero that runs to another anchor during a bite is bitten at the old one, and the serpent only relocates to the new nearest anchor after the bite sequence, its submerge and its rise are over, on its first idle tick.
- A bite interrupts a belch in progress (the belch cloud is lost if its window had not opened; the cooldown is not drawn and stays 0, so the next belch can start as soon as the serpent is idle again and the hero is out of bite range).
- A hero who is a bite candidate but stands more than about 13 units from the serpent cannot be hit; the whole bite is then wasted (the lunge cannot be blocked either, so it can never lead to a stun) and the serpent submerges and rises again at the same point, idle again about 400 ticks after the bite started.
- The mesmerize chain is almost always a sure hit: the stun lasts 300 ticks from tick s + 52 and the lunge window opens at about s + 283 or later, so the hero is stunned through the windup and the lunge. A stunned hero is forced to the idle state every tick and cannot start a block; only a guard timer already running could still count as blocking (Medium). Any hit, including a poison tick, ends the stun; sub 1 applies it again on the next tick while its counter is above 51 and the hero is not stunned.
- The blocked-lunge stun is 300 ticks counted from the end of the 81-tick recoil; the strike check starts on the next tick; a strike on the 301st tick is lost because the counter test comes first. An unarmed hero (damage 0) still uses up the one strike.
- Which hero: the selected hero, every tick. After a hero switch the new hero is the target of the next tick's tests. The bite, the stun, the block test and the cloud all use that hero.
- A fatal lunge: the take-hit runs first, the get-up state (54) is written over whatever state the hit left, and in the swamp the level's death callback kills the active hero by script one pass later (state 16). The serpent has no special case for a dead hero; it keeps its tick logic for whichever hero is selected.
- The phase check, engagement and chooser read the state left by the previous tick's handler, so on the tick a move, belch or stun ends the next tick is the first one that can choose something new; there is no same-tick chaining from a handler's final state write.
- Clouds are slow: launched at the mouth (about 4.5 units above the serpent's origin when the belch cloud leaves, about 8.3 for the spit), a cloud sinks 0.1 per tick to 2.0 above the ground under the hero (25 to 45 ticks), then drifts 0.1 per tick toward the hero for the rest of its 300 ticks, so it reaches at most about 26 units horizontally from where it sank (Medium: model scale 1). Its launch velocity (2 per tick) is not used by its motion.
- Random draws are not made when bounds are equal; the mesmerize roll is drawn first in a bite start, then (if no mesmerize) the drawback float; the float never reaches its upper bound.
- Hero out of reach, off the mesh or on a platform: nothing in the serpent reads the navigation mesh or ground under the hero except the cloud's drift rule; only horizontal distance to point B (anchor choice), horizontal distance to the serpent and the hero's y (bite gate) count.
- Two events on one tick: phase check wins over engagement (state 11 closes the gate); engagement runs before the chooser, and the chooser overrides a belch (never a bite or phase change); a mesmerize abort (hero jumps) and the sub-1 gate on the same tick are the same transition to sub 2.

**12. Hero actions against this boss** (hero rules: SPEC-0033, SPEC-0036; the hero's checks on the serpent run inside the serpent's own tick, so they read the hero as left at the end of the previous tick).

| Hero action | Against the serpent | When |
| --- | --- | --- |
| Sword swing, combo, Roberta's staff swing | No effect on the serpent (it is never a hit candidate): no damage, no hit flash, no hit-list entry, no magic or Mote charge paid. Exception: it counts as a strike when the serpent is stunned (sub 7) and the swing's attack window (event bit 0x1) is active with the hero's weapon cylinder touching the head cylinder (radius 6.0, length 10.0 on joint 18's frame) | Stun window, 300 ticks, once per stun |
| Tor power step, charged power attack | Same as a swing: no effect except as the one strike. The strike damage uses the swing step factor of the hero's current attack sub-state (Tor combo finishers, sub-states 4 and 12, double; sub-states 5 and 6 times 0.75; Roberta 2.0, 0.5, 1.5 or 0.33 by sub-state); the power-step table value is not used (multiplier 1.0 is passed) | Same |
| Roberta's shots, Tor's and Roberta's projectile Motes | Ignored unless the serpent is stunned and a live hero shot is within 5.0 plus its own radius of the head; then it counts as the one strike and the shot is removed. Damage is the melee damage query (not the shot's own damage) | Stun window |
| Spells and Mote area objects (Nova, Wind, Fog, Water, Smoke, Shadow, Light, Sun, beams, bubbles) | No effect and no status on the serpent at any time, even in the stun (they use the eligibility test, which refuses it); they cannot count as a strike | Never |
| Block (L1 shield, Roberta's spin) | The only way to stun it. A lunge overlap while the hero is guarding (action state 20, Roberta spin sub-states 6 and 7, or guard timer above 0) and facing the serpent (dot of the direction to the hero with the hero's reversed facing above 0.5) deals no damage; the serpent recoils (81 ticks) and is stunned 300 ticks. A guarding hero not facing it is hit for 20. The guard timer (40 ticks) is set by an idle, shielded hero with weapons drawn when an attacking character is within 10 units in front or a projectile is in front, so a block can happen without pressing L1 | Lunge window (keys 19 to 25 of the lunge clip); the windup watches the same guard predicate: more than 30 consecutive guarded ticks turns the windup into a spit instead of a lunge |
| Jump / air state | While the hero is jumping (states 7 to 11, 43 to 46) or hanging from a ledge (30), the mesmerize is not used: at the bite start it is skipped, and during the mesmerize the serpent drops straight to the windup. A hero hanging (30) on the tick the serpent would start a bite makes the serpent do nothing at all that tick (no bite, no belch). A mid-air hero is still hit by the lunge if a capsule overlaps the weapon | At bite start, and each tick of the mesmerize |
| Dodge / roll | No dodge or roll exists for either hero; the take-hit sets no invulnerability, and the serpent ignores the hero's hurt state. Moving out of the weapon's reach (more than about 13 units, or off the heading by more than 5.0) before the window avoids the hit; the lunge never turns | Before keys 19 to 25 of the lunge |
| Get hit (lunge lands) | 20 damage, unreduced by shield or armour; the hero goes to the get-up state (54, about 80 ticks, no control) and any stun ends. Landing the lunge does not stun the serpent | Lunge window |
| Mesmerize | Stun status 5 s on the hero if not already stunned; the hero cannot act, block or move; a poison or lunge hit ends it early | Counter 52 of the mesmerize clip |
| Poison cloud | Poison status for 15 s if the hero is not poisoned and within 6.0 horizontal units; drifts slowly; each poison tick is a hit that also ends a stun | Cloud life, 300 ticks |
| Items and Motes held | No item or Mote acts on the serpent. The Mote in equipment word 2 changes the strike damage only through its damage factors (a Nova Mote also gets its type factor because the serpent's type is 0) | Any time |
| Hero switch | The new hero becomes the target on the next tick; the strike check, block test, distance and height gate use the new hero | Any time |

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

1. Play rate set to 1.0, and the invulnerable word of the serpent set to 1 (it is set on every tick and never cleared; see Hurt side, section 8A).
2. If the belch cooldown is above 0, subtract 1.
3. Health band check (section 2), under the state gate of section 2.
4. Engagement check (section 5), only when armed (always) and the same state gate allows it.
5. Anchor check (section 4), in the states of section 4 (idle, belch and arrive). It runs after step 4 and can replace the state step 3 or 4 just chose: a belch started by step 4 becomes a relocation when the nearest anchor differs from the current one. A phase change or a bite chosen earlier in the tick is not replaced (the chooser's state gate excludes states 7 and 11).
6. Run the handler of the state that exists after steps 3 to 5 (sections 6 to 9); a state chosen this tick runs its handler this tick.
7. Aim and presentation (not covered); then wait one tick.

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
- Phase change timing (High for the structure, Medium-High for the turn counts). Let p be the tick on which the phase check starts the change (the first idle tick after a strike's recovery and move). On tick p the slots are written, animation 5 is requested, and the handler of state 11 runs in the same tick. A pass has two parts. Turn part: on each tick while the dot product of the serpent's facing with the horizontal direction to the stored slot's point B is below 0.99, the serpent turns with blend 0.1 (no animation change; animation 5 is requested again). Roar part: on the first tick the dot product is 0.99 or more, animation 21 (91 keys) is requested; the event frame (key 21) is read on ticks j = 42 and j = 43 after that request, so the level-state flag is set on j = 42 when it is clear; the pass ends on the gate tick j = 180, when the stored slot becomes none (first pass) or the state becomes 0 and the phase increases (second pass). The second pass starts on the tick after the first pass ends.
  - Turn counts (from the facing the move state leaves, which is the anchor axis, to point B, simulated with the blend and threshold above; plus or minus 1 tick). The first pass needs 0 turn ticks at every anchor (the axis is within 5 degrees of the slot's own point B). The second pass, which turns to the partner slot's point B: anchor 1: 15 (flag 0) or 22 (flag 1); anchor 2: 15 or 23; anchor 4: 16 or 24; anchor 5: 16 or 24; anchor 6: 17 or 23; anchor 7: 15 or 23.
  - Total: the idle state resumes on tick p + 362 + n, n being the second pass's turn count (about 377 to 386 ticks, 6.3 to 6.4 s). The flags are set on ticks p + 42 and p + 181 + n + 42. The serpent keeps no other behaviour during the change: no bite, no belch, no relocation, no phase check.
  - If the partner slot's direction were within the 0.99 threshold of the first pass's facing, the second pass would end on its first tick without playing animation 21 (the clip is still the current one and already passed its gate). The slot geometry never gives that case.
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

Relocate (5): with variant 0 it plays animation 4 (61 keys); when that clip passes its gate (120 ticks after the request) the state becomes arrive (6) on that tick. With variant 1 it plays animation 6 (41 keys) first; when that clip passes its gate (81 ticks after the request) the variant is cleared to 0 and the state stays 5; on the next tick the variant-0 branch requests animation 4 and the state becomes 6 only when animation 4 passes its gate, so a variant-1 relocation lasts 82 + 120 = 202 ticks to the arrive state. The variant is the anchor flag value held before the relocation chose a new one (step 3 above), so it is 1 whenever the serpent's last placement was at point A. (High; an earlier text said variant 1 ended after animation 6.)

Arrive (6) lasts one tick: the state-5 handler sets state 6 on its gate tick, and on the next tick step 5 of the tick order places the serpent and sets state 1, after which the move handler runs in that same tick.

Move (1): the serpent turns to face along the axis from point C to point A (anchor flag 0) or from A to C (flag 1), blending its heading toward that direction with a factor of 0.25 per tick. It plays animation 3 (46 keys). When animation 3 passes its gate (91 ticks after the request tick) the state becomes idle (0) on that tick, and the idle handler first runs on the next tick. A full relocation from the first relocate tick R: variant 0 reaches state 6 on tick R + 120, places and starts the move on R + 121, and the idle state begins on R + 213; variant 1 reaches state 6 on R + 202 and idle on R + 295.

Phase-change slots (used by Behavior 2): when a phase change starts, the serpent records two slot numbers from its current anchor k. The first slot is k itself. The second is its partner in the pair 1 and 2, 4 and 5, or 6 and 7, so the pair swaps (anchor 1 records 1 and 2; anchor 2 records 2 and 1). Anchors 0 and 3 record nothing and keep their previous values. The slots are written only at the start of a phase change, not on relocation. The stored values are anchor slot numbers, not animation ids (Open questions 2 and 17).

### 5. Engagement: bite or belch

Runs only when armed, and in the states where the phase check runs (every state except 1, 5, 6, 7 and 11, which in play means idle or belching). Not run during move, relocate, arrive, bite, or phase change, so a bite can interrupt a belch. The check does nothing at all while the serpent is in the bite state.

- Bite candidate: the hero's height is above 115.0 (strict), the hero is within 20.0 horizontal units (strict), and the anchor slot is not suppressed. The anchor slot is suppressed when it is 0, and when it is 3 unless the phase is 4. The height is the hero's vertical position; the distance is the horizontal distance (Open question 16).
- If a bite candidate: if the hero's action code is 30, nothing happens. Otherwise start the bite (state 7). The hero's action code is the hero's state number (SPEC-0033), so code 30 is the hero hanging from a ledge: the serpent neither bites nor mesmerizes while the hero hangs (Medium-High; Open question 10).
- If not a bite candidate: if the belch cooldown is 0 and the state is not already belch, start belch (state 4).
- Consequence: when the hero is out of range (or the serpent is suppressed), the serpent belches every time the cooldown reaches 0. The cooldown starts at 0, so the first belch can happen on the first tick the state allows it.

### 6. Idle, move and wait

- Idle (0): plays animation 5 (clip 0, 60 keys, looping, so a cycle is 120 ticks) and, on every tick of this state, requests the neck look at the active hero (6A). The body never turns in idle. The state lasts until the phase check, the engagement check or the anchor check picks another state; with the cooldown at 0 and the hero out of bite range that is the same tick it starts.
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

- Plays animation 10 (clip 5, 91 keys). The handler requests it on the first tick of the state (the request tick s) and from then on waits for the clip. The projectile is fired once per belch: on the first tick on which animation 10 shows event bit 0x20 and a projectile slot is free. The bit is on for keys 17 to 67, so with a free slot the cloud leaves on tick s + 34 (key 17 at half a key per tick) and the window closes after tick s + 135.
- Origin: the serpent's mouth point (the head joint, joint 18 of the serpent's skeleton). Its height above the serpent's origin is about 4.5 at belch key 17 (the normal launch tick), 4.9 at key 30, 5.4 at key 45 and 6.7 at key 67, and about 8.3 for the spit at key 10 (model scale taken as 1, Medium); its sideways offset at key 17 is about 8 units to one side of the heading, so the cloud starts beside the serpent, not in front of it. Launch velocity: unit vector from origin to the hero, times the poison projectile speed (2 units per tick). Life: 300 ticks (5 s). Projectile effect id: 414.
- Projectile slots: a pool of about 10. If the pool is full on a window tick, nothing is fired on that tick and the handler tries again on the next tick of the window, because the once-per-belch latch is set only when a cloud is actually launched. A belch whose whole window (keys 17 to 67) finds the pool full fires nothing.
- When animation 10 passes its gate (180 ticks after the request tick): state 0 on that tick and the belch cooldown is set to a random whole number of ticks in [300, 420] (5 to 7 s, inclusive). The cooldown then falls by 1 at the top of each later tick, so with a draw X on tick g the cooldown is 0 on tick g + X and a new belch can start on that tick (g + 300 to g + 420) if the hero is not a bite candidate; the cycle from one belch start to the next is 180 + X ticks (480 to 600) when nothing else intervenes.
- A health-band change (state 11) or a bite (state 7) chosen by the same tick's earlier steps replaces a belch in progress; a nearer anchor replaces it by relocation. In those cases the cooldown is not drawn (it keeps its value, normally 0), so the belch can start again as soon as the serpent is back in idle.
- The belch chances (10, 25, 40) are not used. The belch timing depends only on the cooldown.

### 8. Bite and mesmerize (state 7)

The bite is a sub-state machine. In this section "finishes" and "gate" mean the clip's 0.99 progress gate read by the handler (SPEC-0005, Clip progress) and "request tick s" is the tick on which the handler first asks for the clip (the handler asks on every tick until the clip is current, which is the next tick). Tick counts below assume the play rate of 1.0 and half a key per tick; they were computed for the shipped clips. The bite starts on the engagement tick E (state 7, sub 0) and the sub 0 handler runs on that same tick; sub 1 or sub 2 first runs on tick E + 1. Sub-states:

- **Sub 0, decide (tick E only).** The serpent turns toward the hero at blend 0.25 (no animation request). An integer roll in [0, 100] is drawn first, every time. Mesmerize is considered only when the hero is not in the air and the hero's action code is not 30. "In the air" is the shared hero jumping test: the hero's action code is one of 7, 8, 9, 10 or 11 (the jump states) or 43 to 46 (further air states) (High). If it is not considered, the bite proceeds. When it is considered, the roll succeeds when it is less than the mesmerize chance for the current phase (phase 2 uses 40, phase 3 uses 60, phases 1 and 4 use 0, phase 5 and above 0); success goes to sub 1 with the mesmerize counter at 0. Otherwise go to sub 2, reset the block counter and the fired latch, and draw the drawback multiplier m uniformly from the stage-0 range [1.0, 1.2] (a float draw).
- **Sub 1, mesmerize.** Each tick: turn toward the hero at 0.25; if the hero is in the air or has action code 30, go to sub 2 at once (drawn as below). Otherwise request animation 20 (clip 15, 81 keys, request tick s = E + 1). A counter counts the ticks while animation 20 is current and not finished: it is k on tick s + k. After it passes 10 (tick s + 11) the mesmerize effect objects are started and updated each tick. When the counter minus 10 is above trunc(0.7 x 60) = 41, that is from counter 52 (tick s + 52), on every tick where the hero is not already stunned, the hero receives the stun status for 5 s (300 ticks; status kind 0 with duration 5 seconds, amount and period at their defaults). While stunned the hero is put in its idle state every tick and cannot act (SPEC-0036, Status effects); any hit on the hero, a poison tick included, ends the stun, and this test then applies it again on the next tick if sub 1 is still running. The effect's beams run from the serpent's head side toward the hero. Sub 1 ends when animation 20 passes its gate (tick s + 160) or when the abort test above holds. When it ends, the effects stop, the block counter and fired latch reset, and the serpent goes to sub 2 with the drawback drawn from a range chosen by phase, not by stage: phase 2 uses [0.9, 1.1], phase 3 uses [0.8, 1.1], and phases 1 and 4 use [1.0, 1.2].
- **Sub 2, bite windup.** Each tick: turn toward the hero at 0.25, set the play rate to the drawback m (this is how the drawback acts: the windup clip runs at m x 0.5 keys per tick, and the rate is back to 1.0 on the next tick's reset), request animation 11 (clip 6, 45 keys) while it is not current. Once it is current: if it has passed its gate, go to sub 4 on that tick; otherwise count the ticks in which the hero is blocking (reset the count when the hero is not blocking), and if the count exceeds trunc(0.5 x 60) = 30 go to sub 3 (spit) on that tick. The gate tick after the request tick is j = 89 for m = 1.0, 81 for 1.1, 74 for 1.2, 98 for 0.9, 110 for 0.8 (plus or minus 1). The gate test comes first, so a count that passes 30 on the gate tick still lunges. Because the count needs 31 consecutive blocking ticks, a hero who starts blocking within the last 30 ticks of the windup does not trigger the spit and is hit by a lunge it can block.
- **Sub 3, spit.** Requests animation 12 (clip 7, 71 keys) while it is not current. Fires the poison projectile on the first tick that animation 12 shows event bit 0x20 (keys 10 to 52) and a pool slot is free, under the same rules as section 7 (so on request tick + 20 normally). The serpent does not turn. When animation 12 passes its gate (140 ticks after the request tick), the bite ends and the state becomes relocate (5) on that tick.
- **Sub 4, lunge.** The serpent does not turn. Requests animation 13 (clip 8, 49 keys) while it is not current. On every tick while the clip is current and not finished, the weapon-3 hit test runs when the clip's current key has event bit 0x4 (keys 19 to 25, ticks request tick + 38 to + 51). If the weapon overlaps the hero and the hero is blocking with dot(u, -f) above 0.5 (see below), the lunge is blocked: sub 6, no damage. If it overlaps and the block test does not hold, the hero takes the lunge damage (20, raw), is put into its reaction state 0x36 (54, the get-up state), and the serpent goes to sub 5 (the test then stops for this lunge). If there is no overlap the test repeats on the next tick of the window. When animation 13 passes its gate (97 ticks after the request tick) the sub-state becomes 10 and one tick later the state becomes relocate (5).
- **Sub 5, after a landed lunge.** Requests animation 14 (clip 9, 27 keys); when it passes its gate (53 ticks after the request tick) the state becomes relocate (5) on that tick.
- **Sub 6, after a blocked lunge.** Requests animation 22 (clip 10, 41 keys); the strike latch is cleared on entry. When it passes its gate (81 ticks after the request tick) the serpent goes to sub 7 with a stun counter of trunc(5.0 x 60) = 300 ticks.
- **Sub 7, stun.** Requests animation 23 (clip 11, 52 keys, looping) every tick (the repeat request changes nothing). Each tick, first: if the counter is below 1, go to sub 9. Then the strike check (see Stun strike) runs; on a strike go to sub 8 on that tick. If there was no strike, the counter goes down by 1. With sub 7 entered on tick t0, strikes are possible on ticks t0 + 1 to t0 + 300 and sub 9 starts on tick t0 + 301 (a 300-tick window).

**Hero action code 30 (Medium-High).** The serpent reads the hero's action-state word, which holds the hero's state number (SPEC-0033; the jump tests above use the same numbering: 7 to 11 and 43 to 46). State 30 is the ledge hang. The hero enters it from the in-air ledge chooser or from the drop off a ledge (state 37), both ordinary hero moves; no boss sets it. The serpent tests it as a no-mesmerize and abort-windup condition, so a hero hanging from a ledge is never bitten or mesmerized. The hero's hang handler is described in SPEC-0033 (state 30).
- **Sub 8, recovery after a stun strike.** Animation 28 (clip 16, 108 keys) is requested on the strike tick itself. When it passes its gate (214 ticks after the strike tick) the state becomes arrive (6) on that tick; the serpent is re-placed at the current slot's point (the same point, since the slot and flag are unchanged), plays the move animation (91 ticks); state 0 is set on tick strike + 306, and the first idle tick, whose phase check first sees the strike's health, is strike + 307.
- **Sub 9, end of stun.** Animation 24 (clip 13, 41 keys) is requested on the first tick the sub-state runs (the tick after sub 9 is set). When it passes its gate (81 ticks after the request tick) the state becomes relocate (5) on that tick.

Block test for the lunge: the hero is blocking and dot(u, -f) > 0.5, where u is the horizontal unit vector from the serpent to the hero and f is the hero's forward direction. That is, the hero faces the serpent within 60 degrees. The lunge is then blocked: the hero's take-hit is not called, and the serpent goes to sub 6. "Blocking" is the hero's guard predicate (SPEC-0036, Guard timer and per-attacker block): the hero is in action state 20, or Roberta is in attack sub-state 6 or 7, or the hero's guard timer is above 0 (a 40-tick timer that an idle hero with weapons drawn and a shield sets when an attacking character is within 10 units in front of it or an enemy projectile is in front). A hero that overlaps the weapon while guarding but not facing the serpent is hit normally. The same predicate feeds the windup's block counter. The overlap itself is the monster weapon test against every hurt capsule of the hero, without the hero's hittable test, so a hero in the air or in a hurt or get-up state can be hit (Lunge hit shape and window).

**Lunge hit shape and window (High).** The lunge uses the monster weapon test (SPEC-0029 Behavior 10.2) with weapon 3 of the serpent's monster row (row 10, as stored in its fight region, resource 740): radius 5.0, length 5.0. The row's other weapons are size 0. Weapon 3 is centred on the serpent skeleton's third attach point (attach index 2: joint 17, offset 0.0, -0.649, 2.091 in that joint's own frame, no rotation) with its axis along that frame's forward direction. On lunge frames 20 to 25 the anchor is 9.6 to 10.0 units ahead of the serpent's origin and 6.7 to 6.8 up, with the axis horizontal and forward; on frame 19 it is 4.4 ahead and 10.5 up (Medium-High: model scale taken as 1). Weapon 3 is tested only while event bit 0x4 is on, which is frames 19 to 25 of the 49-frame lunge clip; at clip speed byte 1 that is ticks 38 to 51 after the clip starts (SPEC-0005; Medium for the exact first tick). The damage, 20 (word 0x2DB060), goes straight to the hero's take-hit and is not reduced by the hero's shield, armour, armour Mote or petpet factor (SPEC-0029 Behavior 10.5).

Stun strike: while stunned, a single strike is possible. A strike is either the hero's melee attack (only when the hero's attack is active) touching the head, or a hero projectile within 5.0 plus its own radius of the head (the projectile is removed). The head test is a cylinder of radius 6.0 and length 10.0 along the head bone's axis, starting at the head bone. It is tested with the finite-cylinder overlap of SPEC-0018 (static; it returns a hit and a signed depth, and the hit is what counts here). The stunned sub-state of the serpent's bite handler performs this head test as part of its once-per-stun strike check (High; Open question 19). On a successful strike, the serpent loses the integer returned by the hero's damage query with multiplier 1.0. That value is passed to the serpent health subtraction directly; there is no fixed stun-strike damage or additional serpent-phase multiplier. Health is clamped at zero. The strike is recorded, so a second strike in the same stun does nothing. The damage query's internal hero-stat and equipment calculation is defined by the hero spec (SPEC-0036, Damage). For this target the query is the melee formula with attack power 1.0 (the power-step table value is not used), a defence of 0 (the serpent's row) and a target type of 0, so only a Nova Mote in word 2 gets the type factor; the step factor is that of the hero's attack sub-state on the strike tick (Tor's combo finishers double it, for example). A hero with no weapon makes the query return 0: the strike still happens (the latch is set and the serpent moves to sub 8) but removes nothing. The strike is evaluated on each stunned tick after the counter test, only when the hero's attack-active word is set or a hero shot slot is live, and then only if the head test passes; it is checked against the hero's state as left at the end of the previous tick (Per-tick order). A strike also plays a short flash and screen shake with no gameplay effect.

### 8A. Hurt side: what can damage the serpent (High for the write, Medium-High for the consequences)

- The serpent sets its invulnerable word to 1 at the top of every controller tick and nothing clears it. The shared hittable predicate (SPEC-0040, Behavior 12A) refuses any target whose invulnerable word is set, and the hero's swings, shots and spell areas all go through it (SPEC-0029, hit filter). So no ordinary hero hit, shot or spell reaches the serpent's take-hit in any state, including the stun. Its base state stays 1, so it is not a guarding target either.
- The only damage is the stun strike (above): once per stun, only in sub 7, using the hero's melee volume against the head cylinder or a hero projectile near the head, and the amount is the hero damage query with multiplier 1.0. A hero that cannot block the lunge cannot stun the serpent, so every point of damage starts with a blocked lunge.
- There is no dodge, block, parry, counter, hit-stun or damage scaling on the serpent's side. It plays no hit reaction. The stun counter is not shortened by a strike (the strike ends the stun by moving to sub 8).
- Health 100: the number of strikes needed is 100 divided by the hero damage value (rounded up); the 75, 50 and 25 percent bands are crossed when health is 75 or less, 50 or less and 25 or less.

### Shape tests that are not the serpent's

These two shape tests are not the serpent's. They belong to two hero spell areas of SPEC-0036 (Hit volumes: the Nova area, radius 4.0 and height 2.0, and the Wind/Fog area, radius 4.0 and height 5.5, both centred on the spell object's stored point). Each spell walks the world's character list and, for every character that passes the character's own targetable query, tests that character's body cylinder (its motion record's radius and length, centred on the character's position) against the spell's cylinder with the finite-cylinder overlap of SPEC-0018; a touched character that is not yet in the spell's hit list is hit. Both cylinders use the constant up vector (0, 1, 0, 0) at ELF data address 0x3932E0 as their axis, so every call takes the parallel branch of SPEC-0018. That vector is one of five constant vectors written once at boot by a start-up initialiser (SPEC-0026, Start-up table walk) and never changed; it is not in the file image, so the clean side sets it itself. The serpent has no part shapes of its own (High). It is not touched by these spells: the spells use the same eligibility query, which refuses a target whose invulnerable word is set, and the serpent sets that word on every tick (Hurt side above; Medium-High, because the spell routines' own eligibility call is read as the shared predicate).

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

### 11C. Arena hand-offs and defeat (High for the script steps, Medium where marked)

The swamp level script (SPEC-0044, module 735) owns the start and the end of the fight; the serpent's controller only owns the fight in between.

- Start. Touching the arena trigger while the swamp victory flag (263) is clear sets level state 8. The boss watcher then runs the intro: the active hero is blocked, the screen fades out, the serpent's character set is loaded and its character activated (the world's activation filter refuses the serpent's record, so nothing starts it earlier), the boss-fight refill of both heroes' health is applied (instance 10, stage 0), the serpent is moved to locator index 28 under script control (slot 0's point C; facing taken from the locator when the script asks for it, Medium) and plays animation 3 for 1.0 s and then animation 4 for 1.0 s, with camera tracks 22 and 23 in between. The intro ends by releasing the serpent to free control (base state 1), setting level state 9, and storing both heroes' and the serpent's current health for the hint timer. The serpent's controller coroutine is created when the intro activates the character (SPEC-0029, Think loop), so it starts well before the release; the rewrite starts it at the release tick (Fight AI build guide, section 1).
- Before the move to locator 28 (at least 120 ticks after the activation) the serpent is free at its stored pose far from the arena, and its controller runs normally on the hero's position. While the script then holds the serpent (base state 33), the original controller's state decisions keep running but its animation requests are stored, not applied (SPEC-0030, Script control and the entity's AI); its clip checks read the clip the script is playing (animation 3, then 4), and its own placement step still moves the serpent. At the release the stored requests apply. The first visible action therefore starts at the release; the decision (relocate, belch or bite) is the one the controller reached for the hero's positions during the intro (Medium: the intro ticks are not simulated, because the hero's position at the arena trigger is not established). Rewrite choice: start the controller at the release tick with the initial values, which gives the same kind of first decision.
- During the fight. The controller sets level states 1 to 7 only through phase changes (Behavior 2). Nothing in the level script holds, pauses or resets the serpent while a break scene plays; the scene's camera and the serpent's change animation run side by side. The monitor only watches: if the serpent's health is unchanged while either hero's health has dropped below its stored value, a timer starts, and after 5 s of game clock the level shows hint line 3 once per load.
- Defeat. A stun strike that brings health to 0 runs the shared death hook (the serpent's behaviour class, 2, is not one of the exempt classes): base state 16, the death countdown of 510 ticks is loaded, statuses are cleared, and the controller is no longer stepped. The corpse step then requests animation 58, which this 29-entry action table does not have, so the idle clip loops while the body sinks by 1/180 unit per tick (Medium-Low for the visual). The monitor sees the serpent dead and sets level state 11; the victory scene sets flags 263, 75, 50 and 70, moves the hero to locator 5, plays movie 38, applies the boss-fight tweak (instance 10, stage 1), and sets level state 12. After the victory the watcher and the arena trigger are not registered on later loads.
- A hero that dies during the fight is handled by the level's death callback (SPEC-0044, SPEC-0055). The serpent's per-fight state lives in the controller's own locals and is not saved (Medium); a new controller starts with the initial values above.

### 12. Timings (60 Hz)

| Item | Value | Ticks |
| --- | --- | --- |
| Belch cooldown | random integer in [5 x 60, 7 x 60] | 300 to 420 |
| Poison projectile life | 5 x 60 | 300 |
| Bite block threshold | count over 0.5 x 60 | 30 |
| Mesmerize effect start | counter over 10, effect from 10 + 0.7 x 60 | 52 |
| Stun, stage 0 | 5 s | 300 |
| Poison status | 15 s | 900 |
| Phase change | per pass: turn to face (0 ticks first pass, 15 to 24 second pass), then animation 21 (91 keys) to its gate; two passes | 362 + n (about 377 to 386) |
| Clip gate after a request, per clip (half a key per tick) | one-shot clip of F keys: 2F - 1 for F below 50, 2F - 2 for F of 50 or more | move (46) 91; submerge (61) 120; hiss (41) 81; belch (91) 180; windup (45, rate 1.0) 89; spit (71) 140; lunge (49) 97; landed (27) 53; mesmerize (81) 160; blocked recoil (41) 81; stun end (41) 81; strike recovery (108) 214; phase roar (91) 180 |
| Windup with drawback m | gate from the rate m x 0.5 keys per tick | m 0.8: 110; 0.9: 98; 1.0: 89; 1.1: 81; 1.2: 74 |
| Lunge hit window | keys 19 to 25 of the lunge clip | request tick + 38 to + 51 |
| Belch / spit cloud | first free-slot tick in the event bit window | belch + 34 (window to + 135); spit + 20 (window to + 105) |
| Relocation to idle | submerge + arrive + move | variant 0: R + 213; variant 1: R + 295 (R = relocate request tick) |
| Bite cycle, no mesmerize, hit landed | engagement E, windup 89, lunge, landed clip, relocation | idle again on E + 397 (m 1.0, variant 0) |

### 13. Inputs and outputs

- Inputs: hero position, hero height, hero action code, hero blocking flag, hero's forward direction relative to the serpent, the hero's attack and projectile state, the serpent's health and maximum health, animation playback (playing, finished, event frame), level object positions by id, the 60 Hz tick, the random number generator, and the two phase-change slots.
- Outputs: animation requests on the serpent (ids above), the hero's reaction state 0x36 (54) when a lunge lands, projectile spawns (effect id 414), damage to the hero, the stun and poison statuses, the belch cooldown, the phase, the anchor placement, the phase-change level-state flags, the stun counter, and the animation cues and effects of 11A.

## Edge cases and error handling

- Healing into a higher band repeats phase changes. The phase check compares the stored phase with the band phase and increments the phase on each change, so the phase can go past 4. For example, at 60% health with phase 3: the phase check triggers (band 2 is not phase 3), the phase becomes 4. The next check (band 2 is not 4) triggers again, and so on. Reproduce this, as the original does (reached only by healing the boss).
- Phase values 5 and above have the same mesmerize and drawback values as phase 1 (0 % mesmerize, range [1.0, 1.2]).
- Anchor 0 suppresses bites, so a serpent at slot 0 never lunges until it relocates. Anchor 3 suppresses bites unless the phase is 4.
- In state 6 the placement uses the slot already set (not the nearest one). Relocation picks the nearest slot only in idle and belch.
- Belch: the cooldown starts at 0. The first belch can start on the first allowed tick when the hero is out of bite range. Projectiles are fired only inside the event-bit window of the clip, on the first tick with a free pool slot; a pool that stays full for the whole window drops that projectile. The first tick of the fight is the exception: the nearest-anchor step replaces the belch with a relocation unless the hero is nearest slot 0.
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
- Unarmed strike: a hero with no weapon (damage query 0) touches the head with an active swing during the stun: the serpent's health is unchanged, the latch is set and the serpent goes to sub 8 as for any strike.
- Strike damage factors (synthetic): hero weapon stat 8, no Mote, no petpet, no status, Tor's combo finisher (step factor 2.0): query 16; the next stun with a plain swing (factor 1.0): 8. Serpent defence 0 and type 0.
- Block predicate (synthetic): hero idle, shield equipped, weapons drawn, serpent attack bit on, hero 9 units in front: the guard timer is set to 40 by the hero's update on the first window tick (after the serpent's think), so the serpent's overlap tests from the second window tick on count it as guarded; with the hero facing the serpent (dot 0.8) the lunge is blocked, with the hero facing away (dot -0.8) it lands.
- Lunge reach (model scale 1): hero centre 12.0 units ahead of the serpent, within 5.0 of its heading, at the serpent's ground height plus 5: reached on keys 21 to 25. Hero centre 15.0 units ahead: never reached; the bite completes without damage.
- Hero hit while stunned by the mesmerize: the lunge lands (20 damage), the hero goes to state 54 and the stun is cleared on that tick.
- Spell area against the serpent: a Nova area centred 4.9 units horizontally from the serpent touches the body cylinder of an ordinary character of radius 1.0 (4.9 < 4.0 + 1.0), but the serpent is refused by the eligibility query on every tick (its invulnerable word is set), so the spell does nothing to it at any distance. Hero sword swings and shots are refused the same way; only the stun strike removes health.
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

### Timeline vectors (60 Hz; tick 1 is the first controller tick after the intro releases the serpent)

All use full health, phase 1, anchor flag 0, relocation variant 0, belch cooldown 0, and the serpent starting at slot 0. Numbers that depend on a random draw are marked and given with their minimum and maximum. Gate ticks follow the rule in the build guide (half a key per tick).

**A. Hero out of bite range (for example 25 units away, height 111), nearest anchor slot 4, standing still.**
- Tick 1: the engagement step starts a belch, then the anchor step replaces it with a relocation (slot 4 becomes current, variant 0 stays, the anchor flag is redrawn: 0 or 1). Animation 4 is requested on tick 1. No cloud is fired.
- Tick 121: animation 4 passes its gate; state 6. Tick 122: the serpent is placed at point C (flag 0) or point A (flag 1) of slot 4, probed to the ground, and the move state requests animation 3.
- Tick 213: animation 3 passes its gate; state 0. Tick 214: the idle tick; the cooldown is 0 and the hero is not a bite candidate, so a belch starts and animation 10 is requested on tick 214.
- Tick 248: the cloud leaves the mouth toward the hero (key 17 at tick 214 + 34). It sinks 0.1 per tick while more than 2.0 above the ground under the hero, then drifts 0.1 per tick toward the hero, for 300 ticks; the hero is poisoned (15 s) if the cloud comes within 6.0 horizontal units.
- Tick 394: animation 10 passes its gate; state 0; cooldown drawn X, 300 to 420 (random). The next belch starts on tick 394 + X (694 to 814) and is followed by its own cloud on tick 394 + X + 34.
- Within the first 600 ticks the serpent therefore submerges once (ticks 1 to 121), rises (122 to 213), belches once (214 to 394) and idles; the second belch always starts after tick 600 (tick 694 at the earliest).

**B. Hero in bite range (height 120, 12 units away directly ahead of the serpent's heading, standing still and not blocking), serpent already idle at slot 4, phase 1.** At 12 units the lunge's weapon-3 cylinder (about 7.1 to 12.5 ahead on keys 20 to 25) reaches the hero; at 15 to 20 units the hero is a bite candidate but the lunge misses and the miss branch below applies.
- Tick 1: bite starts (sub 0); the mesmerize roll fails (chance 0); drawback m drawn in [1.0, 1.2] (random). Tick 2: windup, animation 11 requested. The windup gate is on tick 2 + j, j from 74 (m = 1.2) to 89 (m = 1.0); sub 4 starts on that tick.
- With m = 1.0: sub 4 on tick 91, animation 13 requested on tick 92, weapon 3 tested on ticks 130 to 143. Take the case where the weapon first overlaps the hero on tick 130 (a hero close in front, within about 6 units; for a hero 10 to 12 units away the first overlap comes a few ticks later, about tick 134 to 138, and every later number of this vector shifts by the same amount). Then the hero loses 20 health, enters state 54 (get-up), the serpent goes to sub 5 on tick 130, requests animation 14 on tick 131, and the state becomes relocate on tick 184.
- Relocation (variant 0): request tick 185, state 6 on 305, placement and move request on 306, state 0 on 397, first idle tick 398, where a second bite starts if the hero is still in range (cycle length 397 ticks). A variant-1 relocation (the last placement was at point A) takes 82 ticks longer.
- If the hero stays outside the volume for the whole window: animation 13 passes its gate on tick 189 (sub 10), state 5 on tick 190, relocation request 191, idle again on 404.

**C. Hero blocks the lunge and strikes the head.** Same as B up to tick 130, with the hero blocking and facing within 60 degrees of the serpent on tick 130.
- Tick 130: blocked, sub 6, no damage. Tick 131: animation 22 requested. Tick 212: gate; sub 7 with the stun counter at 300. Strikes are possible on ticks 213 to 512.
- Hero melee attack active and touching the head on tick 220 with damage query value 17: health 100 becomes 83; sub 8; animation 28 requested on tick 220; gate on tick 434 (state 6); tick 435 placement at the same anchor and move request; state 0 on tick 526; first idle tick 527.
- No strike: sub 9 on tick 513; animation 24 requested on 514; gate on 595 (state 5); relocation request 596; idle again on 596 + 213 = 809.

**D. Phase change (continues C with a strike that leaves health 74 of 100, serpent at anchor 4, flag 0).** Idle tick 527 (p): health 0.74 is band 2 and the stored phase is 1, so the change starts: slots 4 and 5, state 11, animation 21 requested at once (the serpent is already within 2 degrees of point B of slot 4).
- Tick 569: level state 4 is set (clear before). Tick 707: pass 1 ends. Ticks 708 to 723: 16 turn ticks toward point B of slot 5. Tick 724: animation 21 requested. Tick 766: level state 5 is set. Tick 904: pass 2 ends, state 0, phase 2. Tick 905: the first idle tick. Total p + 378 = 905.
- The swamp then plays break group 4 (breakables 13 to 16, falling object 62, platform 8) and group 5 (breakables 17 to 21, object 63, platform 9), lower number first.

**E. Mesmerize (phase 2, roll below 40).** Bite start tick 1: sub 1. Animation 20 requested on tick 2; mesmerize effects start on tick 13; the hero is stunned for 300 ticks from tick 54 (if not already stunned); animation 20 gate on tick 162; sub 2 with m drawn in [0.9, 1.1] (random); animation 11 requested on tick 163; windup gate on tick 163 + j (j 81 to 98); sub 4; the lunge window opens 38 ticks after its request (earliest tick 245 + 38 = 283). The hero is still stunned (to tick 354), so it cannot start a block and the lunge lands if the weapon reaches it (Medium: only a guard timer already running could still count as guarding). A lunge hit also ends the stun.

**F. Hero moves across anchors.** Hero out of range, serpent belching since tick 214 at slot 4 (as A); on tick 300 the hero is nearest slot 5. Tick 300: the engagement step finds the cooldown 0 but the state is already 4, so nothing; the anchor step sets state 5, the variant to the current anchor flag, slot 5, and redraws the flag. The belch ends without finishing; the cooldown stays 0, no cloud had been fired after tick 248, and the serpent relocates (variant 0: animation 4 requested on tick 300, state 6 on 420, placement 421, idle 513) and belches again at once on tick 513 if the hero is still out of range.

## Open questions

Default if unresolved (rewrite choice):
- Hero reach of the bite gate (Open question 21): keep the strict comparisons (height above 115.0, distance below 20.0) and let the level geometry decide; do not lower the gate. If the rewrite's arena cannot put the hero above 115.0, the fight cannot start a bite, and the only action is the belch loop.
- Facing at the intro placement: use the facing of locator index 28 (slot 0's point C), and let the move state turn it to the anchor axis after the first relocation.
- Start of the controller: at the release tick of the intro, with the hero's position at that tick.
- Weapon 3 volume: a cylinder of radius 5.0 and length 5.0 centred on joint 17 plus 2.09 along its forward axis and 0.65 down (the third attach point), axis along that joint's forward direction (Medium-High; model scale 1).
- Pre-release controller ticks: not simulated; start the controller at the release tick (Fight AI build guide, section 1).
- Poison amount and period given by the cloud: the hero's defaults (2 health every 4 s, never below 1 health) unless the poison status carries its own.
- Hero stunned and guard timer: treat a stunned hero as not guarding.
- Corpse after defeat: loop the idle clip while the body sinks 1/180 unit per tick until the countdown ends.

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
22. Resolved (High for the write, Medium-High for the consequence): the first step of every controller tick sets the serpent's invulnerable word, not a pose or active flag, and nothing clears it. Hero swings, shots and spell areas never reach the serpent; only the stun strike hurts it (Hurt side, section 8A).
23. Resolved (High): relocation with variant 1 plays animation 6 and then animation 4, 202 ticks in all, not animation 6 alone (Behavior 4). The belch projectile fires on the first free-slot tick inside the event window, not only on the first window tick (Behavior 7).
24. Resolved (High): every action id the shipped fight requests (3, 4, 5, 6, 10, 11, 12, 13, 14, 20, 21, 22, 23, 24, 28) has a clip in the serpent's own action table; ids 0, 1, 2, 7, 8, 9 have none and ids 29 and above are out of range, but none is requested. Each clip has speed byte 1, play rate 1.0 (the windup excepted), zero root motion, and the durations in the build guide.
25. Resolved (Medium-High) for the lunge weapon: attach point 2 (joint 17, offset 0.0, -0.649, 2.091), see Behavior 8. The head cylinder of the stun strike is anchored on joint 18's posed frame (radius 6.0, length 10.0 along its axis). Open (Medium): whether a stunned hero's guard timer can still be set (the hero is put in the idle state every tick, which is the state the threat sense runs in), and the model scale of the serpent (taken as 1). See Default if unresolved.
26. Open (Medium): the hero's position when the intro starts the serpent's controller, and therefore the controller's state at the release; the original controller runs from the activation (Fight AI build guide, section 1). See Default if unresolved.
27. Resolved (High): the serpent reads the selected hero every tick; the lunge's block test, the stun, the windup block counter, the strike and the cloud all follow a hero switch (Fight AI build guide, Per-tick order).




