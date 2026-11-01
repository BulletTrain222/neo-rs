# SPEC-0054: Boss behavior: the Darkest Faerie

- Answers: proactive (open close-out); proactive (boss behavior); proactive (fight AI build guide); proactive (minion AI: Minion Warriors); proactive (second pass: the player's route, hero actions against her, edge cases); R-0069 (the repeated shove: Chooser and Test vectors); proactive (boss consistency pass 3); proactive (arena pass 4); proactive (timing pass); proactive (reach pass 3); R-0136; R-0137; R-0142; proactive (verify pass)
- Status: Draft
- Disc build: SCUS-97367 v1.00 (same fingerprint as SPEC-0001)
- Confidence: per section. Monster row values and tuning words: High. Routing between the three controllers: High. Main-controller cycle and its timings: High for the transitions, Medium-High for what the level states mean. Projectile speeds and reflect rule: High for the code and for the hit amount (the word 0x2DA634 is the orb damage; no effect definition 10 exists). Animation sounds (block 388, Minion Warrior block 377), the orb flight and impact effects and the cast glow: High; clip effect lists: Medium-High. Lunge and loop B: Medium or Low as marked. Stun gates, stun animations and invulnerability: High. Fight flow (script side): High for the rule order, the entry, the clock, the window counter and the timeout and finale calls; Medium for the swap conditions; High for the trigger 0 volume's record values and its fire path; High that the trigger-1 volume exists (cylinder record 8 of the final-fight world's cylinder list); High that the finale is reachable in play (the full fire path is established; no play check). Earlier-encounter trigger: High for its flag-15-clear gate and call sequence; Low for the flag writer and the reason for the gate. Hit points: High that no hero hit lowers them in any controller, so the death path is not reached in play. Damage paths (raw orb hit, no lunge damage, summons by key for the detonation and the burst): High; the burst finds no summon-reserved record in the final-fight pack and the bolt finds the four reserved Warriors: High. Fight AI build guide: animation ids, clips, key counts and tick numbers High (event and gate ticks Medium-High for a one-tick offset); the guard-at-release switch, the guard and shield-Mote block rule and the stun count mechanics High; the start in code 1 Medium; the pack 1428 duel loop High for the rules and Low for its unset start values.

## Summary

The Darkest Faerie is a flying boss with a hover height, a set of attacks, and an absorb window in which she takes the hero's missiles. Her behavior is one of three controllers chosen each tick by a progress flag and a level state. The main controller (used in the final fight) cycles through idle, orb and bolt attacks, a burst request of four summons (which finds no summonable record in the final fight), a 50-second absorb window, a fall after three absorbs, a landing, and a new burst. A guarding hero with a matching shield Mote can turn an elemental orb back on her; the reflected orb stuns her for 3 seconds and is her only weak point. She has 5 hit points, stored in her monster row, and nothing lowers them. The earlier arena encounter uses the same class with the simpler loop A, and the Queen Fyora chamber uses the duel loop B. The clean side can build the boss from the row values, the tuning words and the rules below. The script side of the final fight (the entry, the clock, the landed window, the finale and the timeout ending) is in the Fight flow section. The earlier-encounter trigger is described as calls and facts, not as complete logic.

Times are seconds. One second is 60 ticks (SPEC-0007). Ticks given below are exact unless marked. Distances are horizontal unless stated. "Level state n" and "game state n" follow SPEC-0030.

## Fight AI build guide

Scope. Three shipped situations use her class. (1) The final fight in script pack 1432 is the playable boss fight (main controller). (2) The Queen Fyora chamber in script pack 1428 is a scripted trap scene in which she runs the duel loop (loop B) beside Queen Fyora; it is not won by hurting her. (3) The Act 1 escape in script pack 1174 holds a stationary copy that runs loop A; it is a knockback hazard, not a fight. Her stored 5 hit points never change in any of them, so progress is always script-led. The unplaced or unreached placements (packs 1098, 1344 and 1259) are not part of the shipped fight and are not described here. Times are ticks at 60 per second unless a unit is written.

### What the player must do (final fight)

Prerequisites. Nothing in the fight supplies them except the Nova Mote below. Tor needs an item in the shield family (equipment category 2; SPEC-0021) so that the shield button works, and a Mote of any kind in the shield Mote slot (slot 1), equipped through the Mote panel (SPEC-0021, second quick panel). With no Mote of the right kind no orb can be turned back, the stun count stays 0 and the fight can only end by the clock. The Nova Mote (item 172, element kind 7) turns back every orb kind. Every Minion Warrior her bolts make drops one when it dies (Minion AI), so a hero without a suitable Mote can get one in the fight: guard so that she throws bolts, kill a Warrior, pick up its Nova Mote and equip it.

0. Getting from the arrival point to the hall needs nothing from the player. The pack loads with the hero at entry record 0 (0.05, 270.63, -265.06) in the north alcove, behind the solid north barrier (a walk straight toward her from there stops against the barrier's alcove-side face, at about (7.9, -281.6)). The pack's own fight-start step, which starts by itself on the first control-thread tick after the pack activates, fades in and, 30 ticks later, moves the hero to entry record 1 (0.04, 270.63, -325.72) in the hall centre; control returns at tick 90. There is no door, trigger or movable object to use in the alcove. This also holds for a start from a developer menu that loads the pack: the step runs whenever the pack's level state 11 is clear (it is until the entry sequence runs), so the hero is moved on the same schedule.
1. Walk east from the hall centre (where the fight-start step leaves the hero) into the east barrier, crossing the entry rectangle 36.4 from the start point (Arena build guide, A1 and A2). The rectangle fires when the hero's centre is within 1.175 of its plane (x 35.225 and above), and the barrier's face stops the centre at about x 35.64, so pressing into the barrier starts the entry; the hero never has to cross the plane. Tor (hero 0) is active, movie 78 plays, and the 180 s fight clock starts (Fight flow). The clock stops during hero swaps and runs through the landed window.
2. Walk to within 10 horizontal units of her. She waits in code 1 until then and attacks nothing (Medium-High; see Initial values), and the clock keeps running, so a hero who never approaches loses to the clock. The first approach gets a shove (no damage: the hero is thrown back and knocked down) and from then on she attacks. Being under 10 at any later moment shoves again (a 153-tick cycle), so the hero must stay 10 or more away while she casts and may only come closer while she is stunned.
3. Survive her orbs and bolts, and reflect an orb. To reflect, the hero must be guarding (the shield button held; state 20), facing the orb, with a shield Mote (equipment slot 1) of the orb's element, or the all-element Mote (kind 7), equipped. The orb colour is shown by the cast glow from the first tick of the cast (Orb kinds table); the Motes that answer each kind are in the Hero attack-kind table. Orb elements are uniform over six kinds, so a single element Mote answers one orb in six. Her orbs and bolts are not enemy pool projectiles, so the automatic idle guard (the shield that rises by itself when something attacks in front) is not raised by them; only the button works. Guard is tested at three moments: at the pick (a guarding hero always gets a bolt), at the release on tick 24 of the cast (a guarding hero gets a bolt instead of the orb) and at arrival (guard, facing, element). So the working routine is: shield down until the orb has been released, raise it during the flight and face the orb. The flight lasts 13 ticks from 10 units away, 26 ticks from 20 and 53 from 40 (Attacks), and the shield must already be up on the tick before arrival (Edge cases for implementers). A bolt cannot be turned back; it makes a Warrior next to the hero and does no damage itself. A blocked orb costs the hero nothing. An orb that is not blocked deals 10 and puts the hero in the hurt state.
4. A reflected orb that reaches her stuns her for 180 ticks in whatever she was doing (reflected orb flight from 10 units away takes 13 ticks or more). She stands on the floor during the stun. While she is stunned the hero walks up (the shove is off in the stun) and lands a form-2 attack: Tor's third chain swing (press the attack button 20 to 33 ticks into the first swing and 17 to 30 ticks into the second), or one gesture swing (a clockwise or counter-clockwise stick turn made just before the attack press), or the air plunge. The weapon must overlap one of her five small hurt capsules while the swing's hit window is open; the window of the third swing opens 8 ticks after it starts, so a chain from the first swing needs about 45 to 70 ticks after the first swing starts. The first two swings, the flick lunge and the power attack only make her flinch. A form-2 hit ends the stun after one 37-tick animation and counts as one completed stun. Three completed stuns are needed (the count is not reset by a pause between them).
5. The third stun sets the swap to Roberta (hero 1; the clock is stopped for the roughly 80-tick swap) and starts the burst and the 50 s absorb window (3000 ticks). She rises from the floor to 15 above the ground at 0.05 per tick, attacks nothing and ignores the hero (no shove, no orbs). Roberta must have three of her shots overlap her body in that time, each within 179 ticks of the one before (Absorb window and falls). Each shot that overlaps is absorbed, and the third makes her fall. Roberta needs at least 1 MP for each shot. The three Minion Grunts and any Warriors from earlier bolts attack her meanwhile. Roberta's shots fly at her only when her lock-on holds her (Roberta's shots against the hovering Faerie) or while she is still low.
6. After the third absorb she falls at 0.1 per tick and lands at the finale volume (she never moves horizontally). Level state 16 opens a 15 s window (900 ticks); Roberta must walk to within about 7 units of the volume axis. That plays the finale (movie 85). If the window closes first, the swap goes back to Tor, level 15 resets the cycle, and steps 3 to 6 repeat with the stun count back at 0. If the 50 s window ends before the third absorb, she descends (code 8), no landed window opens and the finale cannot run; the same swap back to Tor and reset follow.
7. If the fight clock reaches 0.0 first, the timeout ending (movie 80) runs and the game is over. The clock is stopped only during the director's swaps. A failed round costs up to 50 s of it (absorb window timeout, plus the descent) or 15 s (landed window missed), and the time spent stunning her again. A hero death is game over at once (swap mode 3; SPEC-0055).

### Initial values (main controller)

- Code 0, chooser hold flag 0, first-tick flag 0, stun count 0, kind 0. Cooldown: a whole number drawn uniformly from 60 to 240 inclusive (1 to 4 s). The invulnerable flag is set at once and never cleared. Hit points 5 (never read).
- Position: the placement point (57.66, 270.64, -326.07), on the floor (the floor height there is about 270.63), then the entry sequence moves her to the horizontal point of world locator 2 (28.713, -326.005), which is the axis of the finale volume. The move asks for a ground probe (the locator's height is 272.489, so the probe ceiling is 276.489) and control option 1; the probed height replaces the locator's, so she ends on the floor at the locator's horizontal point (270.628, not at the locator's own height) in the free state, not script-held. Whether the locator's facing is also taken is not established; code 0 turns her toward the hero anyway. In codes 0 to 6 and 10 the normal ground follow holds her on the floor; codes 7, 8 and 9 set the no-gravity flag on every tick and set her height themselves. Medium.
- The gate sets code 1 on any controller pass that occurs while level state 10 is set, or while game state 364 is clear (Init sets 364 on, so only level state 10 acts here). The activate-world callback sets level state 10 in the first script step after the world becomes current; the entry sequence clears it only much later. First bind and first think (Medium-High): her record is the only one of the embedded region that the streaming fill can bind (the four Minion Warriors are summon-reserved and the three Minion Grunts are refused by the activation filter), she stands 83.9 units from the arrival point (inside the 280-unit range), and the hero is already placed, so she binds in the first region pass of the world. Within a tick the region passes, including the first think of a character bound in that pass, run before the world's script step (SPEC-0007, SPEC-0030). So her first controller pass is in code 0 (level state 10 is still clear), the callback then sets level state 10 later in that same tick, and her second pass is forced to code 1. The single code-0 pass turns her a quarter of the way toward the hero (blend 0.25), requests animation 6 and counts the cooldown down once (the cooldown is at least 60, so nothing is chosen); code 1 does not turn her. Code 1 then persists until level 14 or 15, or until the near-range shove takes the code. The cooldown counts down during this wait.
  Default if the order above is not wanted (rewrite choice): start in code 1 with the cooldown counting; the only difference is the missing one-pass quarter turn toward the hero. In both cases she leaves code 1 only by the near-range shove (hero under 10), the stun swap (level 14) or the cycle reset (level 15).
- Tuning words and fixed constants: see Layout (cooldown 60 to 240 ticks from the words 1.0 s and 4.0 s, window 3000 ticks, hold 180, stun 180, projectile speed 0.75, hover heights 15, 10 and 5, vertical steps 0.05 and 0.1, orb damage 10).

### Per-tick order

Each tick (before the animation pass, which then starts any requested clip on the same tick and advances the cursor after sampling):
1. Clear the per-tick steering flag.
2. Step every active projectile slot, in slot order 0 to 3 (Projectiles).
3. Move the cast glow with her mouth point while the code is 3, otherwise stop it.
4. Gate: level state 10 set (or game state 364 clear) sets code 1.
5. Choice (Chooser), unless the hold flag is set.
6. Run the handler of the current code once. A handler that ends its animation sets the next code in that same tick, and the next code's handler first runs on the next tick.
7. Yield one frame.
The reflected-orb arrival (step 2) sets code 5 whatever the current code is, so the stun can interrupt a cast, a bolt, a shove, the window or a fall.

### State table

Tick 0 is the tick on which the handler first requests the action. Every action id below has a clip in her own table (27 entries, all with clips). Every clip has speed byte 2 and is played at R = 1.0, so the cursor moves 0.25 key per tick. "Gate" is the first tick on which the clip's progress reaches 0.99 (Clip progress, SPEC-0005); the handler leaves the state on that tick. The common gate rule gives tick 4(F - 1) + 1 for a clip of up to 24 keys and 4(F - 1) for 25 to 74 keys (17 keys: 65; 24 keys: 93; 39 keys: 152; a 16-key loop: 63).

| Code | Action: clip (keys, loop) | Per tick | Exit |
| --- | --- | --- | --- |
| 0 idle | 6: 20 keys, loop (80 ticks per cycle) | Turn toward the hero, blend 0.25. Request 6. | Chooser |
| 1 wait/burst | 6 each tick | No turn. On level 14: clear 14 and 10, 4 burst requests, absorb count 0, window 3000, code 7. On level 15: clear 15 and 10, stun count 0, hold flag 0, code 0 | Level 14 or 15, or the shove |
| 2 shove | 9: 39 keys, one-shot | Tick 0 request. Tick 1 push (once). Hero collidable word cleared every tick | Gate at tick 152: code 0 and request 6 (153-tick cycle) |
| 3 orb volley | 7: 17 keys, one-shot, release at key 6 | Tick 0 request and cast glow. Turn toward hero 0.25 until the gate. Tick 24 (first tick the cursor is in key 6): release | Gate at tick 65: code 0, new cooldown |
| 4 aimed bolt | 8: 24 keys, one-shot, release at key 8 | Tick 0 request. Turn toward hero 0.25. Tick 32: release | Gate at tick 93: code 0, new cooldown |
| 5 stun | 14: 50 keys, loop (hold); 13: 10 keys, one-shot (flinch) | Timer 180 set on entry. See Stun | Form-2 hit: code 6 (tested before the timer drops, no timer test). Timer 0: code 0, but not while a flinch clip is still playing (then when it finishes) |
| 6 stun count | 13 | Tick 0 request 13 (if not current). Gate at tick 37 (at once if 13 is already current and finished; when a flinch 13 is still playing, when that clip finishes): count + 1 | Count 1 or 2: code 0. Count 3: levels 12 and 10 set, hold flag set, stay |
| 7 window | 14 each tick | Hover and absorb (Absorb window and falls) | 3 absorbs: code 9. Timer 3000 ticks: code 8 (the tick that drops the timer to 0 does nothing else) |
| 8 timeout descent | none requested (14 keeps playing) | Descend 0.05 per tick | Within 0.05 of the ground: levels 13 and 10 set, code 1 |
| 9 fall | 20: 25 keys, loop | Descend 0.1 per tick | Within 0.1 of the ground: snap to the ground, level 16 set, code 10 |
| 10 landed | 20 each tick | Wait | Level 13 set: level 10 set, code 1 |

### Chooser, in priority order (each tick, skipped while the hold flag is set)

1. Cooldown down by 1 if above 0 (before any test).
2. Horizontal distance d to the hero under 10 and the code not 5 or 6: pick code 2 unless it already is 2. This ignores the cooldown and the ready test and overrides any other code, including 1, 7, 8, 9 and 10 (unless the hold flag is set, which is so from the third stun until the level-15 reset).
3. Otherwise (d at least 10, or the code is 5 or 6): nothing unless the code is ready (0, 7 or 8) and the cooldown is at most 0. Then: draw a whole number 0 to 10 inclusive. If it is 0 (1 in 11), or the hero is guarding (SPEC-0036, Guard predicate): pick code 4. Else pick code 3 with a kind drawn uniformly from 1 to 6.
4. Each pick clears the first-tick flag. On the end of code 3 or 4 the cooldown is redrawn (60 to 240). The next pick therefore comes 65 + cooldown ticks after an orb pick, and 93 + cooldown ticks after a bolt pick, with the hero far away: 125 to 305 and 153 to 333 ticks.
5. Order of draws and queries at a far-branch pick: the 0 to 10 draw is always made first; the hero guard query is made only when the draw is not 0; the orb kind draw is made only when code 3 is chosen. The cooldown also counts while the code is 1 and 2, so after the opening shove the first pick is due at once.
6. The hero in every test is the selected hero. The shield state the controller sees is the one left by the previous tick (she updates before the selected hero within a pass), so a shield raised on the tick of a pick, a release or an arrival does not count on that tick.

### Attacks

- Orb volley (code 3). Tick 24: the orb opens at her mouth point in the first free slot of the four shared slots (if all four are in use nothing opens and the cast still ends normally). If the hero is guarding at that instant, an aimed bolt opens instead (kind 0, no glow effect). Each slot step the orb moves 0.75 toward the hero's aim point (a point on the hero's model, the status-display anchor, re-read every tick, so a jumping or running hero is followed in three dimensions). With first-step separation s0, the hit tick is the (floor((s0 - 1.5) / 0.75) + 2)th slot step, the first step being the tick after the release: 13 steps from s0 = 10, 20 from 15, 26 from 20, 40 from 30, 53 from 40 (a hero standing still). A hero moving slower than 0.75 per tick (walking is 0.225) is always caught. Not blocked: 10 damage, once, unreduced, with the normal hero hurt reaction (state 17 unless the hero is charging an attack). Blocked: see Projectiles.
- Aimed bolt (code 4 or a guarded orb). Same flight. When the separation before a step is under 2.0 the slot clears and one Minion Warrior is summoned (key 2864) 4 units from the hero on her side, facing the hero, with drop item 172. It deals no damage itself and cannot be turned back. Step count: floor((s0 - 2.0) / 0.75) + 2. If no reserved Warrior record is free (all four alive, or the six-character cap reached) nothing appears.
- Shove (code 2). No damage; see Chooser and Attacks, summary for the velocity.
- Burst (code 1, on level 14): four summon requests of key 2844 at her front, back and two sides (4 units on her forward axis and on her side axis, each sign). No record with that key is summon-reserved in this pack (the three Minion Grunts are ordinary placements), so the requests find nothing and no entity appears from the burst. The enemies the player meets are the Grunts placed by the script (Fight flow, control thread B and the director) and the Warriors from the aimed bolt. High.

### Movement

She never moves horizontally. Only codes 7, 8 and 9 change her height: 0.05 per tick toward the hover height (code 7 rises or falls, at most 0.05 per tick), 0.05 per tick down in code 8, 0.1 per tick down in code 9. The hover height above the ground probe is 15 (absorb count 0), 10 (count 1) or 5 (count 2). Facing: code 0, 3 and 4 turn toward the hero with blend 0.25 per tick; no other code turns her.

### Hit and hurt windows

- She is invulnerable for the whole fight. The only weak point is the reflected orb (stun 180 ticks, then Stun) and the only damage-like events are absorbs (Absorb window and falls).
- Stun hit: Tor in attack sub-state 4, 7, 8 or 12, or Roberta in sub-state 4, with the weapon sweep reaching her body, on any tick of code 5 or its flinch, with no timer test (a form-2 hit on the very tick the timer would end still counts, and so does one while a flinch is still playing after the timer has run out). The "hero attack test" is the open hit window of the hero's current clip. Her body for this test is five small capsules of her skeleton (lengths 1.0 to 1.4, radii 0.23 to 0.66; the upper body and arms): the hero's weapon cylinder (radius 1.0, length 2.0 plus the weapon reach) must overlap one of them, and the hero must be within 65 units. A swing at her feet can miss.
- Other attack forms play the 13 flinch once (37 ticks), after which the hold animation 14 resumes; the timer keeps running during the flinch. A second flinch waits until the hero's attack test has been false for a tick. A flinch changes nothing else: no count, no timer change.
- Outside code 5 she ignores every hero attack: the hero's swings, shots and Mote attacks skip her (she is not hittable), so there is no spark, no sound and no reaction; nothing in her controller reads them.

### Phase changes and resets

- Stun count (0 to 3) is kept until code 1 sees level 15. Absorb count (0 to 3) is set to 0 at each burst and drops by one at the end of each 180-tick hold when it is 1 or 2; each drop starts a new 180-tick hold, so a count of 2 left alone falls to 1 after 180 ticks and to 0 after 360.
- Cycle: code 1 (wait) -> level 14 -> code 7 (3000 ticks) -> 3 absorbs: code 9 -> code 10 at the ground (level 16, 900-tick landed window) -> window closes with level 13 set -> code 1 -> level 15 -> code 0 (stun count 0). Timeout: code 7 -> 8 -> code 1 with levels 13 and 10 set.
- No health thresholds exist.

### Roberta's shots against the hovering Faerie

- Shots that overlap her body are taken only by the code-7 scan. In every other code Roberta's shots fly through her (she is not hittable) and are not used up.
- A quick shot is aimed at launch (the rapid shot, sub-state 10, and the Water full release also steer toward the lock target every tick). With a lock-on target it flies at that character's aim point; with none it flies at the point 20 ahead of her and 2 up from her feet, which is level, and cannot reach a body more than a few units above the floor. She locks onto her (SPEC-0040, hero candidate list) when the 3D distance between the two position points is under 20.0 and, once Roberta has started an attack, the dot product of Roberta's forward axis with the 3D direction to her is above 0.25. With v the height of her position over Roberta's, the horizontal distance must be above 0.258 v and under the square root of (400 - v squared): about 3.9 to 13.2 at hover height 15, 2.6 to 17.3 at 10, 1.3 to 19.4 at 5, and anywhere under 20 at floor level. Standing right under her, or farther than that, gives a level shot.
- The lock goes to the nearest in-range candidate, and any Grunt or Warrior in range counts, so a nearer enemy takes the lock and the shot (candidates within 1.0 of each other swap places every tick).
- When the window opens she is on the floor and rises 0.05 per tick, so for roughly the first 40 to 50 ticks a level shot at chest height already overlaps her body; the first absorb does not need the lock.
- Shots of the full release with a Mote in the weapon slot of kinds Wind, Smoke, Light or Nova pick a random hittable enemy as target; she is not hittable, so they never choose her and only absorb if they cross her body. The Water shot follows the lock-on target.
- A shot is a sphere of radius 1.0 (3.0 for the default row) against her body cylinder (radius 1.175, height 3.5 from her position upward, rounded ends); the shot moves 1.0 to 1.5 per tick, so it cannot skip over her.

### Arena hand-offs

Level state 10 (burst request), 12 (third stun), 13 (landed window over or timeout descent done), 14 (stun swap done), 15 (landed-miss swap done) and 16 (landed) are the script's and her controller's shared signals (Level and blackboard signals). The script owns the swaps, the clock, the finale volume and the endings (Fight flow).

### Death

None. Her hit points are never reduced, no controller reads them and the script ignores her death. The fight ends only by the finale or the timeout ending.

### Queen Fyora chamber (pack 1428)

The trap scene is driven by the script (SPEC-0041, module 1428; SPEC-0058). She is entity 1, Queen Fyora (class 38) is entity 0 and four summon-reserved Minion Warriors (key 2864) are entities 2 to 5. Their record positions ((0.2, 8.37, -325.03) for Queen Fyora and (-22.96, 8.37, -314.75) for her) are not where the duel is played: the scripted boss-room return moves Queen Fyora to locator 1, (0.81, 8.37, -303.4) facing -z, and the Faerie to locator 2, (0.02, 8.37, -359.43) facing +z, 56.04 apart, and puts the hero at locator 3, (17.91, 10.65, -308.36), 17.8 from Queen Fyora and 54.1 from the Faerie (Arena build guide, pack 1428). She runs loop B. The player is not meant to hurt her; the scene advances through the shot-target meter, the touch volume and the three-switch puzzle, and ends when the script deactivates both entities. Her job is to harass:

- Start: she waits (one tick at a time) until Queen Fyora exists, then sets her invulnerable flag and base state 20 (a guarding target, so hero swings are blocked). State and cooldown start as left in the stack: use state 0 and cooldown 0 (Low; rewrite default).
- Each tick: step nothing of her own (Queen Fyora's controller steps the shared slot array), run the chooser, turn toward Queen Fyora with blend 1.0 (except in state 5), run the state handler.
- Chooser, in priority order (cooldown down by 1 if above 0 first): (1) a Queen Fyora orb (type 1) within 1.5 horizontal of her: state 2, flag cleared; (2) the hero under 15.0 horizontal: state 4, flag cleared (every tick); (3) state 0 or 2, no type-0 orb active and cooldown under 1: state 5 if the summon test passes, else state 3; event flag cleared. The summon test passes when level state 2 is clear, a draw modulo 4 is 0 (1 in 4), and none of her earlier summons is alive.
- State 0: request 6 (80-tick loop). State 3: request 7 (17 keys); tick 24 open a type-0 orb at her mouth toward Queen Fyora (0.5 per tick, horizontal, keeps height); gate at tick 65, then state 0 with a cooldown of 30 to 45 ticks. State 5: request 7, tick 24 open a type-2 orb that homes at 0.5 per tick to the hero's aim point; she turns toward the hero (blend 0.25); gate at 65, then cooldown 120 to 240 ticks. The type-2 orb, when its horizontal distance before a step is under 2.0, frees itself and (level state 2 clear) summons one Warrior at the hero facing away from the hero's forward direction.
- State 2: request 10 (16-key loop, gate at 63). One tick after the state starts (the animation is then current, unfinished, flag clear) the first Queen Fyora orb is relaunched as a deflected orb (type 3) in a random sideways direction (forward share |f| up to 0.5 where f is a uniform draw in [0,1) minus 0.5, side share 1 - f when f is positive or 1 + f when negative, to the right or left), 0.5 per tick for up to 60 ticks, damaging the hero for 5 when within 1.5 horizontal.
- State 4 (field around her): request 9 (39 keys, gate 152). On every tick the first-tick flag is clear, the hero is thrown: velocity zeroed, vertical +12.0, horizontal 9.0 along the direction from her to the hero, the vector scaled to length 10.0 (velocity (6.0 d, 8.0)), hero action state 0x30, collidable word cleared. Because the chooser clears the flag every tick the hero is under 15, the throw repeats every tick during animation 9; it stops when the hero is out or the animation is finished (the finished animation 9 stays current while the hero remains under 15, so no second throw until the hero has been out for at least one tick). No push-out of the body overlap.
- Queen Fyora's side: her state 3 cast (animation 22; release at tick 32, gate 65, cooldown 30 to 45) opens a type-1 orb homing on the Faerie; her state 2 deflect (animation 23, gate 47) relaunches the Faerie's orb. Her controller is in SPEC-0029 (Behavior 30).
- Level state 2 is pulsed for one tick by the touch volume (SPEC-0041) and only blocks a summon test or type-2 arrival on that tick.

### Pack 1174 (Act 1 escape)

Loop A: she stands still at (0, -126.25, 97.2), requests action 0 every tick, and when the hero is within 10 horizontal units with the countdown spent, throws the hero (velocity (9 d, 12) scaled to 15.0, hero action state 0x30, body overlap pushed out by 5 times the shortfall) and sets the countdown to 180. She has no other attack, takes no damage and ends nothing. It is a hazard on the section 0 tiles, not a fight.

### Port pitfalls

- Animation 14 (not 20) is what the absorb window plays; the timeout descent requests nothing.
- Guard at release turns the orb into a bolt; blocking needs the guard predicate, facing, and the shield Mote element.
- The burst creates nothing in the shipped fight.
- The aimed-bolt summon appears next to the hero, not where the bolt was.
- The hold flag removes the shove, attacks and cooldown counting from the third stun until the level-15 reset.
- Stun by a reflected orb works in every code, including mid-window and mid-fall.
- Chooser distances are horizontal. Orb and bolt arrival tests are 3D and use the separation before the step.
- A shove cycle is 153 ticks; an orb volley is 66 ticks of code 3 plus 60 to 240 cooldown; clip lengths are 4 ticks per key (S = 2), not 1 key per tick.
- In pack 1428 the throw repeats every tick while the hero is under 15; it is not once per animation.

### Edge cases for implementers

- Guard timing: she updates before the selected hero in each pass, so a shield raised on the very tick an orb would arrive does not count; the shield must have been up at the end of the previous tick. The same holds for the guard test at a pick and at the release.
- A guarding hero never gets an orb: a guarded pick is a bolt, and a guarded release (tick 24) turns an orb pick into a bolt. A bolt is never turned back. Her projectiles do not raise the automatic idle guard, but a Warrior or Grunt attacking within 10 in front of an idle hero with weapons drawn does, which can turn her next orb into a bolt by accident.
- Blocking spends nothing (no Mote charge, no MP), shows no hero reaction and does no damage. Not blocking costs 10, unreduced, and the hurt state (unless the hero is charging an attack).
- Mote kinds answer by equality, not by the damage pairs: kind n answers only orb kind n; kind 7 answers all; no Mote, or an item that is not a Mote, answers nothing.
- The slot array holds four projectiles for the whole game, not per boss. When all four are busy a new orb or bolt is simply not made.
- A reflected orb always lands the stun, in any code, including mid-window and mid-fall, and also restarts a stun already running (timer 180, flinch latch clear). It does not count as a completed stun by itself.
- A form-2 hit is accepted on any tick of code 5, before the timer drops and with no timer test; while a flinch clip is still playing after the timer has run out she stays in code 5 until the clip ends and a form-2 hit is still accepted. A form-2 hit while a flinch is playing finishes the count when that clip finishes, not 37 ticks later.
- The stun count is kept (it does not fall back after a long pause between stuns) until code 1 sees level 15. After the third count the hold flag stops the shove, the picks and the cooldown until that reset, so Roberta can stand right under her.
- Absorb timing: on the tick the 3000 timer reaches 0 nothing else happens (an absorb on that tick is lost). Each absorb must follow the previous within 179 ticks; on the 180th tick the drop comes before the scan, so that absorb only replaces the dropped count. A count of 2 left alone becomes 1 after 180 ticks and 0 after 360.
- Several shots overlapping her on one tick each count; the fall starts when the count is exactly 3.
- The window timeout (code 8) opens no landed window, so the finale cannot be reached after it; the director swaps back to Tor and level 15 resets her.
- The hover height follows the count before the scan, so a rise or fall begins on the tick after the absorb. She snaps to the target when within 0.05.
- Her height in the stun phase is the floor; the stun does not test height.
- Hero death in this fight is game over (swap mode 3), whatever she is doing. The pack has no death callback.
- The Grunts and Warriors are not removed when she falls or when the finale starts; the finale deactivates entities 0, 5, 6 and 7 only.
- Beyond the five hurt capsules her body takes no hero attack at all. The only thing she does to the hero's body is the shove, which also clears the hero's collidable word on every tick of its animation.

### Hero actions against this boss

| Hero action | Effect on her | When |
| --- | --- | --- |
| Tor swing 1 or 2, flick lunge, power attack (release after 50 ticks), chained variants | Nothing outside code 5. In code 5: one flinch (animation 13, 37 ticks), no count | Only while the weapon overlaps a hurt capsule in an open hit window |
| Tor swing 3, a clockwise or counter-clockwise gesture swing, or the air plunge | Form 2: ends the stun into code 6 and counts one completed stun after the 37-tick animation | Code 5 only, any tick |
| Tor or Roberta shield guard (L1, state 20) | Turns an orb of the Mote's kind (or any kind with the Nova Mote) back at her if facing it; makes her throw bolts instead of orbs when held at a pick or a release | See Guard timing |
| Roberta spin (sub-states 6 and 7) | Counts as guarding for the pick, the release and the block (the facing and Mote rules still apply) | Same |
| Dodge or roll | None exists for either hero. A jump does not avoid an orb (it homes in three dimensions) | n/a |
| Roberta quick shots, rapid shot, full-release shots | Nothing outside code 7. In code 7: absorbed when the shot overlaps her body, one absorb each | Needs lock-on or a low target (Roberta's shots against the hovering Faerie). Her scan only removes the shot slot and counts the absorb: an absorbed shot pays no magic and no Mote charge, and outside code 7 a shot that flies through her pays nothing. In the final fight she never writes the guard stance, so nothing is blocked, bounced or disarmed |
| Mote power-attack objects of either hero | Skip her (she is not hittable) | Always |
| Items, spells, status effects | None reach her | Always |
| Walking inside 10 units (horizontal) | Shove, unless the code is 5 or 6 or the hold flag is set | Chooser |
| Hero swap chord | Refused in this fight (swap mode 3); only the director swaps | Always |

In pack 1428 she is in the guarding base state, so a hero swing or shot that reaches her is blocked or bounced and nothing hurts her. In pack 1174 she is a stationary knockback hazard and takes nothing.

### Minion AI: Minion Warriors (key 2864)

The Warriors are the only enemies her own attack creates (the burst creates nothing, and the three Minion Grunts are placed by the script). A Warrior is an ordinary common enemy (SPEC-0038 holds the rules; this section gives the numbers). All times are ticks at 60 per second; s is the tick on which the attack sequencer first runs its state 1 and Q is the tick the Warrior is created. Confidence: High for the row, clips and swing arithmetic; Medium-High for the spawn start and the cap; Medium for the charge flow.

**Who and how many.** Entities 1 to 4 of pack 1432 are four summon-reserved Minion Warrior records (monster type 82, variant 0, monster row 204, key 2864, position (0, 0, 0): their point comes from the summoner). They use clip set 1 of the pack's region (38 clips) and a 166-entry action table. A bolt that reaches the hero (Projectiles) binds the first unbound one at 4 units from the hero on her side, facing the hero, with drop item 172 (Nova Mote on its first death tick). Alive at once: at most 4 (one per record); also at most 6 characters may be active in the pack's character block at a time (the script never raises that cap), and the Faerie and any Grunts the script has brought in count, so with all three Grunts present only 2 Warriors fit, and a bolt that finds no free place makes nothing. A Warrior killed is released about 105 ticks after its death tick, after which its record can be summoned again. No script call deactivates entities 1 to 4 (it only deactivates entities 0, 5, 6 and 7), so a Warrior lives until the player kills it, whatever her code does (Medium-High). Bolt rate: a bolt is code 4, chosen 1 time in 11 at a volley pick, or at every pick while the hero is guarding, so with a guarding hero up to one Warrior per 153 to 333 ticks.

Row 204 values: 100 hit points, radius 1.25, height 3.0, approach 4 (strafe, desired band 12 to 20), attack-manager weight 12, element 1, a display-in-default of 0 (an invisible kind), recovery after a swing 15 to 60 ticks, death disappear timer 4. Weapons 1 to 3 have radius 3.0 and length 2.0. It is created at full health.

**Start.** The summon plays action 82 (appear: clip 5, 20 keys, gate at Q + 77) and puts the Warrior in the pursue state (class state 21), not in agro. It can be hit while 82 plays (it is excluded from hits only when it is in its default state or in state 21 and not playing 82). When 82 passes its 0.99 gate (tick Q + 77) the Warrior enters agro; the agro machine runs from Q + 78: outer sub-state 0, then the strafe archetype from the next tick. A hit before then (the first hit is doubled) sends it to agro at once.

**Action ids with a clip** (every clip plays at 0.25 keys per tick, 4 ticks per key; clip, keys): locomotion 28 strafe forward (clip 3, 6 keys, loop, 12.5 units per second = 0.208 per tick), 32 back (clip 4, 6 keys, 0.208 per tick backward), 29 and 30 sidesteps (clip 36, 12.5 per second), 31 hold (clip 2, 8 keys, loop), 2 walk (clip 37, 6.0 per second), 0 idle (clip 0, 20 keys); reactions 57 hit (clip 21, 13 keys, gate s + 49, knocked back 7.0 per second), 55 melee dodge (clip 19, 10 keys, gate s + 37, back-hop 12.0 per second), 53 and 54 missile dodge (clip 20, 10 keys, gate s + 37, sidestep 11.5 per second), 56 block (clip 13, 15 keys, loop), 58 death (clip 17, 27 keys, gate s + 104, progress 0.95 at s + 100), 18 search (clip 35, 23 keys, unused here), 82 appear (clip 5). Attack phases: slot 0 has B = 34 and D = 36; slot 1 has B = 39, C = 40 and D = 41; slot 2 has B = 44 only. Ids 33, 35, 37, 38, 42, 43, 45 to 47 are missing, so no swing has a wind-up or a recovery clip; slot 3 is unused.

**Attack chooser** (every agro tick while the hero is attackable and the recovery counter is 0; slots tested k = 3, 2, 1, 0; first to pass wins; the roll is 1 to 100 and must be below the chance, so 40 is 39 in 100, 50 is 49 and 60 is 59; cooldown drawn at the choice and again at the end path):

| k | Type | Band | Chance per tick | Facing cosine above | Range | Cooldown | Damage, knockback | Counter |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| 2 | melee flurry (unblockable) | 3 to 10 | 39 % | 0.707 | 4.0 | 120 to 240 | 15, 3.0 (three hits) | on a hit reaction: 25 in 101 per tick |
| 1 | charge, kind 2 (needs a route) | 10 to 17 | 49 % | 0.966 | 8.0 | 180 to 300 | 7, 3.0 | none |
| 0 | melee | 3 to 10 | 59 % | 0.9239 | 4.0 | 30 to 60 | 7, 1.0 | while blocked with L1 held: 25 in 101 |

**Swing timelines** (hero inside the strike volume for the whole window; the hit list allows one hit per window):
- Slot 0: B (id 34, 13 keys) requested on s; frames 7 to 9, window s + 28 to s + 39, one hit at s + 28; gate s + 49, end path s + 50. The clip carries about 3.5 units of forward root travel. A blocked hit plays D (id 36, 37 ticks).
- Slot 2: B (id 44, 30 keys) requested on s; three windows, s + 16 to s + 27, s + 44 to s + 59 and s + 76 to s + 91; hits on s + 16, s + 45 and s + 77 (the follow-through state re-enters the strike state one tick after each new window opens); gate s + 116, end path s + 117. Up to 45 damage per swing; about 5.9 units of forward root travel. It cannot be blocked.
- Slot 1 (charge): on the tick after the choice B (id 39, the 12.5 units-per-second run) starts and the Warrior runs 0.208 per tick toward the hero until the hero is within 8.0 units, or it has run its drawn charge distance (8 to 20 units, drawn on entering agro and after each normal choice). No strike test runs during the run. Then C (id 40) is requested on tick c; frames 6 to 8 carry weapons 1 and 2, window c + 24 to c + 35, one hit (7 damage, knockback 3.0) at c + 24; gate c + 69, end path c + 70 (no recovery clip). From 17 units the run lasts about 43 ticks.

**Movement (strafe archetype, SPEC-0038 Behavior 44).** Desired distance 12 + 8 f (f from 0.20 to 0.80: 13.6 to 18.4). Beyond 20.0 it walks forward (28); within the desired distance it holds (31) and every 60 to 180 ticks picks hold, left sidestep or right sidestep (0.208 per tick); closer than 12.0 it backs off (32) until beyond the desired distance. So it stays 12 to 20 units away and attacks by the charge, while the melee slots fire when the hero is closer than 10: straight after the summon (4 units) and whenever the hero walks up. The Warrior is not held back by the budget unless several enemies are in agro: with starting gear (capacity 0) only the first listed attacker may start a swing and the others wait in line (weight 12 each, shared with every other enemy in agro).

**Hurt side.** Melee defence (once per hero swing, hold 0 required, hero within 8.0 and in front): block 60 %, then dodge 59, 39 or 29 in 100 for quick, normal and heavy swings, so 84 %, 76 % and 72 % of swings are defended; there is no waiting time after a defence ends (the post-defence hold is 0). Hero shots meeting the body within 60 ticks are dodged in 99 of 100. A hit reaction (kind 0) plays 57 and a restarted hit restarts it. Death plays 58, removes the body about 105 ticks after the death tick, and pays item 172.

**Timeline test vector** (hero standing still; Q = tick the bolt creates the Warrior). Q: created 4.0 units from the hero, facing the hero; 82 requested; pursue state. Q + 77: 82 gate passes, agro entered. Q + 78: agro set-up. Q + 79: first choice at distance 4.0: slot 2 39 %, else slot 0 59 % (slot 1 needs distance above 10); 25 % of ticks choose nothing. Take slot 0 chosen on Q + 79: the distance 4.0 is not above range 4.0, so s = Q + 80; hit (7) on Q + 108; end path on Q + 130; recovery 15 to 60; the next choice from Q + 131 + N, subject to slot cooldowns (30 to 60 from the end path). Slot 2 chosen on Q + 79: hits (15 each) on Q + 96, Q + 125 and Q + 157; end path on Q + 197. If the hero retreats beyond 10 units the Warrior backs off to 12 to 20 units and charges when 10 < distance < 17 and it faces the hero within 15 degrees.

**Open.** Whether the 6-character cap also counts a bound Faerie when she has been deactivated by the script at the finale (the script deactivates entity 0 at some points); the exact think-loop tick on which the first state-21 step runs (Q or Q + 1).

### Minion Grunts (entities 5 to 7)

The three Grunts are ordinary common enemies (SPEC-0038 holds the rules); they are the other half of the pressure on Roberta. The three placements use three different monster rows of the Grunt family (High): entity 6 uses variant 0 (row 183), entity 7 variant 4 (row 187) and entity 5 variant 5 (row 188). All three have 50 hit points, radius 2.0, height 2.0, class 17, are flyers (Can Fly 1), recover 15 to 60 ticks after an attack, and share the damage numbers below; they differ in element class (1, 5 and 6), status and cooldowns. Attack slot 1: damage 25, chance 50, distance band 0 to 15, facing cosine 0.966, range 5.0, with a 50 percent status request. Attack slot 2: damage 50, unblockable, chance 40, distance band 1 to 20, with a 50 percent status request. Row 183 (entity 6): status 7 (fire, no effect on a hero), slot 1 cooldown 180 to 300 ticks, slot 2 cooldown 180 to 360. Row 187 (entity 7): status 3 (slow: the hero's speed is multiplied by 0.75 while it lasts), both cooldowns 120 to 240. Row 188 (entity 5): status 1 (curse: outgoing damage multiplied by 0.75), both cooldowns 120 to 240. The status lasts 30 to 60 s from slot 1 and 5 to 10 s from slot 2 (Medium for the unit of the stored range). A Roberta who is hit by the entity-7 Grunt can therefore be slowed for most of the 50 s window. Their placement points are (24.33, 286.91, -325.46), (24.33, 286.91, -328.41) and (24.33, 286.91, -322.96) for entities 5, 6 and 7, about 16 units above the floor and 4 to 5 units from her axis. The director activates all three together when the stun swap completes (Director); control thread B moves them to locator 4 one at a time, 8 s apart, while Roberta is active (Control thread B), and brings back each one that dies. They are not summoned by a key request here, so the Warriors' direct Nova Mote drop does not apply to them.

### Timing table (timing pass)

Confidence: High (from the chooser, attacks and phase rules above). Units: ticks of 1/60 s. The pick cooldown is a whole number of ticks lowered by one at the start of every chooser pass (before any test), also while the code is 1 or 2.

| Item | Timing |
| --- | --- |
| Pick cooldown | int[60, 240], redrawn at the end of an orb or bolt; with the hero far away the next pick comes 65 + cooldown ticks after an orb pick (125 to 305) and 93 + cooldown after a bolt pick (153 to 333) |
| Orb volley | the orb opens on tick 24 of the cast in the first free slot of the four; it moves 0.75 per slot step |
| Aimed bolt | same flight; clears when under 2.0 from the hero and summons one Minion Warrior; steps floor((s0 - 2.0) / 0.75) + 2 |
| Shove | code 2 whenever the hero is within 10 (overrides the cooldown) |
| Stun (code 5) | 180 ticks after a reflected orb; a form-2 hero hit on any tick of it counts |
| Flinch | 37 ticks; the stun timer keeps running |
| Hover window (code 7) | 3000 ticks; absorb hold 180 ticks per count |
| Landed window | 900 ticks |
| Height | 0.05 per tick toward the hover height (code 7), 0.05 down (code 8), 0.1 down (code 9) |

Hero side: she is never hurt, so the only exchange is the reflected orb and the stun window; she reads the hero's guard as the previous tick left it (she updates before the hero).

Worked exchange (hero far away, cooldown 0, ready code): pick on tick P (draw 0 to 10, then the guard query); an orb cast: the orb opens on P + 24; the next pick is due on P + 65 + int[60, 240]. A hero who guards on P + 24 gets an aimed bolt instead of an orb.

What a port most likely gets wrong (timing): counting the cooldown only in ready codes (it also counts in codes 1 and 2); reading the hero's shield of the same tick; letting the flinch pause the stun timer.

## Arena build guide (packs 1432 and 1428)

Coordinates are world (x, y, z) with y up. "A" is the Faerie's axis in the final fight, (28.713, 270.63, -326.005) (the horizontal point of locator 2). "H" is the hall centre, (0.0, 270.63, -326.0). A locator is an entry record of the pack, counted from 0 (SPEC-0006). Confidence is High unless marked.

### A1. Final fight (pack 1432): top-down layout

- Floor. One flat ground patch (ground kind 0, surface 6) at y = 270.63 under the whole complex: hall, both bays and the north alcove (x -68.3 to 68.3, z -366 to -243). Two lower patches of ground kind 2 (deadly ground, y = 268.63, 2.0 below the floor) lie under the east and west thirds (east x 22.1 to 88.1, west x -85.9 to -20.0). The floor patch is higher wherever both apply, so they are never the chosen ground on the hall floor. Nothing in the pack is deadly or hurting in play: no water, kill volume, crusher or timer.
- Hall. A circle of radius about 40.1 round H. Its wall is a closed outline of invisible vertical panels (120 tall, top edge y 289.04, facing inward) that follows the circle and three mouths: an east mouth (x 37.2, z -340.6 to -310.3, 30.3 wide) into the east bay, a west mouth (x -37.2, same z) into the west bay, and a north mouth (z -286.2, x -7.7 to 7.8, 15.5 wide) into the north alcove. The bays run out to x = ±68.2 with rounded ends on raised floors (y 273.22, 2.59 above the hall, edged by a 2.67 high side wall); the alcove flares to x ±14.1 at z -261.5 and ends in a rounded wall at z -243.
- Barriers. Three placed objects, each a closed solid box 4.57 thick and 22.72 tall (y 268.77 to 291.49): object 2 closes the east mouth (x 36.81 to 41.37, z -340.48 to -310.44), object 0 the west mouth (x -40.95 to -36.39, same z) and object 1 the north mouth (x -8.50 to 8.66, z -287.38 to -282.81). They are solid from the moment the pack activates, so the hall is a sealed round room for the whole fight. The finale removes object 1 only; the timeout ending removes all three. Rows of effect 4035 stand on their planes (8 at x 37.2, 8 at x -38.0, 5 at z -286.5; Medium-Low for their look).
- Free floor for the hero: the hall out to the wall (about 40.1 from H, less the hero's wall push), nothing beyond the barriers. A is 28.7 east of H, 8.1 west of the east barrier face and 65.1 east of the west barrier face. The hero's farthest legal point from A is therefore about 64.
- AI walk meshes. One ground-kind and one flying-kind mesh with the same outline (68 triangles each, area 3715.1): the eight-sided polygon (35.14, -325.50), (23.68, -297.68), (-0.07, -287.54), (-23.69, -297.50), (-34.49, -325.51), (-22.43, -355.20), (-0.07, -363.45), (23.28, -354.01), radius about 35 round (0.3, -325.5), with four square holes about 3.3 wide centred near (-12.5, -301.7), (12.4, -301.6), (-12.6, -349.2) and (12.5, -349.5). The mesh height value (332.51) is not used for y. Enemies are confined to this polygon: the Warriors (ground) and Grunts (flying) cannot enter the outer 5 units of the hall, and the Faerie, who never moves sideways, stands inside it at A.
- Pillars and columns. Four solid cylinders (radius 2.12, height 30.41, base y 270.64) stand in the holes at (±12.53, -301.67) and (±12.53, -349.24); they stop the hero and a hero shot. A ring of about 28 more solid cylinders (radius 1.8 to 2.0) stands in the wall line at radius 37 to 41 and down the alcove walls; a few low solid cylinders (statues) stand in the west bay. None of them is between A and the hall centre.
- Entry plane (trigger 0): the vertical rectangle x = 36.40 (normal -x), y 269.2 to 290.05, z -344.0 to -306.75. It lies 0.41 in front of the east barrier's west face (x 36.81). The hero's centre cannot reach the plane while the barrier is solid (the wall push keeps it one body radius, 1.175, from the barrier face, at x about 35.64), and it does not need to: the plane test fires the trigger whenever the hero's centre is within 1.175 of the plane, on either side, and inside the rectangle's height (from 1.175 below its lower edge to just under its top) and width (1.175 beyond each end), so it fires from x 35.225. The hero fires it from the hall side (west) by pressing into the east barrier; it is the only writer of the entry flag, so it is the only way the fight starts (Medium-High: the stop point depends on the barrier's wall records, but the firing band is wider than the gap between the stop point and the plane). Roberta's shots would also fire it, but she is not active at the start (see Open question 44).
- Finale cylinder (trigger 1): base (28.702, 270.608, -326.005), axis up, length 3.0, radius 6.0, on A. The hero (body radius 1.0) fires it within 7.0 of A.
- Air space. The wall tops are at y 289.04, 18.4 above the floor. Her body top at hover 15 is 289.13, level with the wall tops; the Grunts hover at 286.6 to 286.9.

### A2. Start positions

- Arrival. The selected hero (normally Tor) is placed at locator 0, (0.05, 270.63, -265.06) facing +z, inside the north alcove behind the north barrier. The Faerie stands at her placement point (57.66, -326.07) facing +x on the east bay floor, behind the east barrier. Once the activate-world callback sets level state 10, her controller is forced into code 1 and attacks nothing; whether an earlier first think occurs in code 0 is unresolved (see Initial values).
- Fight-start step (90 ticks). Tick 0: letterbox on, camera 4 (a fixed shot, eye (-10.55, 274.85, -330.69), focal length 493, no keys; it looks east along the hall, Medium-Low), fade-in over 30 ticks, both heroes refilled and a downed partner revived. Tick 30: the hero is placed at locator 1, (0.04, 270.63, -325.72), facing +x. Tick 90: normal camera, control returns. The hero is then 36.4 from the entry plane and 28.7 from A.
- Entry. After the plane fires, the hero Tor is placed at locator 3, (-10.79, 270.63, -325.42), facing +x, 39.5 from A. The Faerie is moved to A on the floor (270.63, not the locator's 272.49), free. The clock starts at 180 s and swap mode 3 is set.
- Partner hero. It is parked and not placed by the script. At each director swap the incoming hero takes the outgoing hero's position and facing (the swap's copy placement, SPEC-0053), so Roberta appears where Tor stood at the third stun (within weapon reach of A, 1 to 4 units from her body) and Tor appears where Roberta stood after a missed landed window.
- Grunts. Entities 5, 6 and 7 wait at their records, (24.33, 286.91, -325.46), (24.33, 286.91, -328.41), (24.33, 286.91, -322.96): 16.3 above the floor and 4.4 to 5.8 west of A. All three are activated when the stun swap completes; control thread B then moves entity 7, 6 and 5 to locator 4, (22.94, 286.63, -326.01), 480, 960 and 1440 ticks after Roberta's turn starts (a Grunt killed earlier returns 480 ticks after the thread reaches its entry). Entities 1 to 4 (Warriors) have no position of their own; each is placed 4 units from the hero by a bolt.

### A3. Hazards, objects and pickups

- Damage sources: her orbs (10 each), the Warriors and the Grunts. The shove does no damage. There is no deadly ground, hurting ground, water, kill volume, breakable, chest, item dispenser or save point in the pack. The only pickup is the Nova Mote (item 172) that each Warrior drops when it dies.
- Projectile sources: the Faerie only (four shared slots). Enemy melee comes from the Warriors and Grunts.
- Ambient sound: two loops (cue 1 of the area's sound block, spheres of radius 40 centred (48.89, 275.31, -325.72) and (-48.59, 275.31, -323.70)): audible within 40 of either centre, so only the middle strip x = -8.6 to 8.9 of the hall is silent.

### A4. Script timeline (final fight)

1. Pack activation: level state 10 on, the three barrier objects active, script flag 9 on, game states 364 and 358 on, effects 18 to 25 on, effects 3 and 4 off.
2. Fight-start step (A2), once per load. Control at tick 90; script flag 10 on.
3. The hero walks to the east barrier. Crossing the entry plane sets flag 0.
4. Entry sequence (SPEC-0054, Fight flow): fade out, movie 78 (877 frames, 29.23 s; the whole loop is held while it plays), hero to locator 3, Faerie to A, barriers re-activated, Tor selected, fade in, control, clock 180.0 s starts, swap mode 3, level state 11 on, level state 10 off, flag 1 on.
5. The fight proper: the Faerie's cycle (code 1 until the hero comes within 10, then the attack loop), the director's swaps and the landed window (Fight flow). The clock runs through everything except the swaps (about 80 ticks each).
6. Endings: the finale (movie 85) or the timeout ending (movie 80, then game over).

### A5. Camera and music

- Camera. Only the fight-start step uses a scripted camera (track 4 above). During the fight the normal follow camera runs with the floor profile (ground kind 0), with no boss mode and no fixed track (SPEC-0027). Tracks 0 to 3 are unused fixed shots.
- Music. The pack's music record gives row 191 of the music table: stream resource 173, bank resource 30, the current sound block kept, cap 700, level 750, scale 1024, selector 0 (no story alternate; the script sets no override). The combat segment plays while at least one class-17 enemy is in agro; the Faerie never counts, the Warriors and Grunts do (SPEC-0009). The movies carry their own sound. Pack 1428 uses row 187, which holds the same values.

### A6. Death and retry

- Any hero death in the fight is game over: the pack has no death callback and the swap mode is 3 after the entry, so the partner cannot take over. About 63 ticks of death clip, then the game-over flag after about 91 more (SPEC-0055). The fight clock and the Faerie keep running meanwhile. Before the entry sequence nothing in the hall can damage the hero.
- The timeout ending is not a death: movie 80, a fade, game over (value 1), flags 3 and 6 on.
- Retry. Continue after game over re-applies the snapshot of the last save or load made in the session (or leads to the title screen if there was none) and reloads the packs. There is no save point in the arena, so the player always replays the approach from the snapshot's place, including the movies that lead to this pack (the entry into pack 1432 is the return world set by the movie before the final fight, SPEC-0058). Everything in the arena starts again from the disc state: Faerie code, cooldown, stun and absorb counts, the clock (180.0 s on the next entry), level states, script flags, Grunts, Warriors, barriers (re-activated by the pack's activation) and both heroes' health and second meter (refilled again by the fight-start step). Persisting: the snapshot's flags, items (so a Nova Mote won in a failed try is lost unless it was held at the snapshot), money and movie-played words. Movie 78 plays again at the next entry.

### A7. Victory and exit

- The finale (flag 4, only during the 15 s landed window, Fight flow): the clock is hidden, object 1 is removed, swap mode 2, entities 0, 5, 6 and 7 are deactivated, movie 85 (1629 frames, 54.3 s) plays and blocks the loop, game states 94 and 365 are on, and the game continues in pack 1300 (return world 151) with entry message 80: flag 366, the hero at landing 2, movies 79 and 83, then the credits (SPEC-0058). Movie 85 is followed by nothing else in the game flow but that load: no closing scene starts by itself, no save prompt and no title screen. The pack-1300 activation sets flag 366 and moves the hero (entry record 2, (0.05, 1537.66, -124.13), facing -z) with control and the normal camera, and the player is free: walking south over the garden the hero first crosses the band of trigger 3 (z 87.2 to 105.8, inert while flag 367 is clear), then reaches trigger 2 (z 174.6 to 181.6), which plays movie 79 and sets flag 367; walking back north over trigger 3 plays movie 83 and sets flag 368, and the next play pass opens the credits (front-end state 18) from the in-play flag-368 gate. The credits cannot be skipped; their end tears the world down to the logo, attract and title. The garden has one save point (15.43, 1514.5, 101.27), the ordinary Save Game prompt, available any time before the credits; nothing offers a save at the end. Lines 115 to 117 are the ordinary proximity lines (Square) of the two palace entities 61 and 76. Whether the walk from the arrival point to trigger 2 is open (no wall or gate between z -124 and z 175) follows the garden's ground; no script blocks it (Medium-High). Default if unresolved (rewrite choice): start the epilogue triggers exactly as above and, if a rewrite cannot walk the garden, play movie 79 and movie 83 in order from a menu action so the credits still open. There is no reward item and no walk out of the arena; the Warriors and Grunts still alive are not removed except entities 5 to 7. The arena cannot be entered again afterwards in the story flow (the only way in is the return world set by the pre-fight movie).

### A8. The player's route in this arena

- Tor's station for orbs. Stand west of A on the line to H at 15 to 30 units from A (x about 14 down to -1), shield down at her pick and at her release, shield up and facing her within the next flight. The flight from a distance d takes 13, 20, 26, 40 and 53 steps for d = 10, 15, 20, 30 and 40, and the shield must be up on the tick before arrival, so the choice window is (steps - 1) ticks: 19 ticks at 15, 25 at 20, 39 at 30. The hall allows d up to about 63 (85 steps, 1.4 s), but the stun is only 180 ticks: after a reflect the hero needs (d - 3) / 0.225 ticks to reach her plus 45 to 70 for the third swing, and the reflected orb's own flight overlaps the walk, so d up to about 35 still lands the third swing inside the stun and d near 40 does not reliably.
- After the form-2 hit the stun count completes 37 ticks later and she can shove again as soon as she is back in code 0: the hero must be at least 10 away by then (37 ticks of walking cover 8.3 units, so start retreating at once). After the third stun the hold flag stops every shove and attack until the reset.
- Roberta's stations. She appears at Tor's spot beside A with the Faerie on the floor and the Grunts 16 up and about 5.8 west of A. The first absorb needs no lock: for about 40 ticks a level shot at chest height overlaps her rising body. For the other two, Roberta must be inside the lock annulus around A (horizontal distance 3.9 to 13.2 at hover 15, 2.6 to 17.3 at hover 10, 1.3 to 19.4 at hover 5), facing her, with a free lock (the nearest candidate in range wins, and a Grunt counts). Best station: 4 to 7 units from A on the east side, between A and the east barrier (x about 32 to 35.5), because east of A the Faerie is the nearest lock candidate (the Grunts are west of A), and the spot is inside the 7.0 finale disc, so the landing fires the finale one or two ticks after the landed window opens. West of A, at hover 15, a Grunt is nearer than the Faerie and takes the lock.
- Time budget: the fight clock is 10,800 counting ticks. A best-case run (every orb reflectable, one stun per cycle) needs about 1000 ticks for three stuns, 80 for the swap, 150 to 350 for the absorbs and 50 to 100 for the fall; a typical run with a single element Mote needs a mean of six orbs per stun (about 215 ticks each plus the stun), so 60 to 110 s of the 180. A failed round costs up to 3000 + 300 ticks (absorb window and descent) or 900 (landed window) plus swaps and a new set of three stuns.

### A9. Queen Fyora's chamber (pack 1428): layout and duel

- Layout. A round hall like the final-fight hall: 28-sided walk polygon of radius about 40.4 round (0.0, -325.6) (x -40.19 to 40.34, z -366.06 to -285.1, area 5026, flying and ground meshes identical), floor y 7.44 with a round shallow depression (y 6.64, radius about 16 round the centre, 0.8 deep, no wall) and a well (y about 5.0) near (54, -325) in the east bay. East and west bays run out to x = ±68 behind inward walls about 19 tall (top y 26.5); the east bay holds a solid round pillar (centre about (54.5, -325.5), radius 7.5); the west bay is the switch alcove (three 4 x 4 prompt rectangles at x -46.92, z -321.83, -326.85, -331.88; hero spots (-44.04, 9.51, -320.33), (-44.04, 9.51, -325.36) and (-44.04, 9.51, -330.40) facing -x; solid cylinders at x -46.6 to -49.3). The north gate (object 3, a solid box x -5.45 to 5.46, z -285.96 to -283.66, 16.1 tall) is activated by the boss-room return and closes the north mouth. North of it is the entry corridor: the hero arrives at locator 0, (0.0, 10.65, -269.6) facing -z, and the pack's one save point is at (-11.158, 8.252, -257.921).
- Start positions. Crossing the entry rectangle (trigger 4: vertical, normal +z, origin (-10.16, 1.71, -277.28), 29.31 wide in x, 20.31 tall) starts the boss-room return. It places Queen Fyora at locator 1, (0.81, 8.37, -303.4) facing -z, the Faerie at locator 2, (0.02, 8.37, -359.43) facing +z, and the hero at locator 3, (17.91, 10.65, -308.36) facing (-0.5, 0, -0.866). Faerie to Fyora is 56.04; hero to Fyora 17.8; hero to the Faerie 54.1. Her shove field (15 round her) covers the south end, z below about -344. The Warriors have no position until a type-2 orb makes one 4 units from the hero.
- Volumes. Sphere 0 (trigger 0, exit zone) centre (43.5, 7.44, -326.96) radius 20, live after flag 330; sphere 1 (trigger 1, touch volume that pulses level state 2) centre (-51.48, 7.44, -325.38) radius 21.3; sphere 2 (trigger 2, arms the shot meter) centre (47.99, 7.44, -338.15) radius 31.01; sphere 3 (trigger 3, sets the missile target to locator 4) centre (31.4, 7.44, -343.16) radius 21.07; sphere 4 (trigger 5, the shot target) centre (28.56, 11.16, -341.9) radius 2.05, 33.5 from the Faerie; kind-4 rectangles for triggers 6, 7, 8 (the three switches). The Faerie's loop-B shove cannot reach the shot target region or the switches.
- Script, camera, music. The scene is the step list of SPEC-0041 (module 1428). No boss camera: the fixed tracks 4 to 9 (eye x -41 to -32.8, y 13.5 to 19.3, z -326 to -313.9) are used by the switch scenes through a camera index held in each switch record (Medium). Music row 187 (same values as row 191).
- Death and retry. Roberta down without item 50: game over when Tor is selected, swap mode 1 when she is selected, only until the shot meter completes (flag 328); other deaths use the engine rules. Retry is from the last save or load of the session (the corridor save point is the nearest in the pack). Victory: both entities are deactivated and the act-start sequence runs (SPEC-0041); the exit zone is sphere 0.

### A10. Pack 1174, one line

The Act 1 escape copy stands still at (0.0, -126.25, 97.2) and throws any hero within 10 horizontal units once per 180 ticks; it is a hazard on the section 0 tiles, not an arena.

### A11. What a port most likely gets wrong in this arena

1. The hall is sealed by three solid barrier boxes from the start; the entry plane is 0.41 in front of the east one, and the Faerie waits behind it until the entry sequence moves her to A.
2. She is placed on the floor at A (270.63), not at the locator height 272.49; her hover is measured from the floor, and her body top at hover 15 is above the wall tops.
3. The fight-start step puts the hero at the hall centre (locator 1) and refills both heroes; the arrival point is the north alcove.
4. Roberta appears at Tor's spot (copy placement), the Grunts are 16 up and 5.8 west of A and steal the lock-on, and the finale disc (7.0) overlaps the lock annulus: she should stand 4 to 7 units from A, east side.
5. The three Grunts are three different monster rows (status fire, slow, curse; two with 120 to 240 cooldowns).
6. Enemies are confined to the 35-radius mesh while the hero reaches 40; the four pillars are holes in the mesh.
7. In pack 1428 the duel pieces start at locators 1 and 2 (56.04 apart), not at their record positions.
8. Game over on any death in the fight, a full reset on retry, and no walk out of the arena after the finale.

### A12. End-to-end test vector (final fight; synthetic draws, every orb reflectable, no hero damage; ticks)

1. Tick 0: pack activates; barriers solid; hero at (0.05, 270.63, -265.06) facing +z; Faerie at (57.66, ground, -326.07). She binds in this tick's region pass and her first controller pass is in code 0 (a quarter turn toward the hero, animation 6, cooldown down by 1); the activate-world callback then sets level state 10 in the same tick's script step, and from the next pass she is forced to code 1 until the entry sequence clears level state 10 and beyond (Medium-High; see Initial values).
2. Tick 0 to 90: fight-start step; both heroes full; tick 30 hero to (0.04, 270.63, -325.72) facing +x; tick 90 control.
3. The hero walks east at 0.225 per tick: the plane x = 36.4 is reached 162 ticks after control (tick 252 = E); flag 0 on at E.
4. E + 1: entry sequence; movie 78 plays for about 1754 ticks (the game logic is held); then Tor at (-10.79, 270.63, -325.42) facing +x, the Faerie at A, clock 180.0 (call this tick C0), swap mode 3, level 11 on, level 10 off.
5. C0 + 131: Tor passes x = 18.71 (10 from A): shove (no damage); the cycle ends at C0 + 283; the cooldown has long expired, so the first pick is on C0 + 284 (orb, draw 1 to 10, shield down).
6. Orb picked at C0 + 284 with Tor 20 away: release C0 + 308, arrival C0 + 334 (26 steps), shield up from C0 + 309 to C0 + 333, facing her: reflected; her stun starts about C0 + 360, Tor walks 17 units (76 ticks), swings the chain (third swing hit about 60 ticks later): form-2 hit before C0 + 500; count 1 at hit + 37.
7. Three such cycles (each about 250 to 300 ticks) put the third count at about C0 + 1000 = S: levels 12 and 10, hold flag, the stun swap (about 80 ticks, clock frozen), level 14 at B = S + 81: burst, window 3000, Grunts activated, Roberta at Tor's spot (example (22.0, 270.63, -326.0), 6.7 from A, facing A).
8. First shot B + 10 (level, she is at 271.1): absorb 1, hover target 10. Shot at B + 100 with lock (she is at 275.6, 5.0 up, Roberta 6.7 away: inside 1.3 to 19.4): absorb 2, target 5. Third at B + 200 (100 ticks after the second, under 179): absorb 3: code 9, fall 5.0 / 0.1 = 50 ticks, landing at B + 251: level 16.
9. The director arms the window on its next tick (counter 900, flag 8). Roberta is 6.7 from A, inside the 7.0 disc: within 2 ticks flag 4 is set, the next control tick runs the finale: clock hidden, movie 85 (1629 frames, about 3258 ticks of video), game states 94 and 365, return to pack 1300. About 1254 counting ticks are used by then, so the clock reads about 159 s in this best case.
10. Variant: Roberta stands 8.5 from A at the landing: flag 4 stays clear; the window closes at tick T + 900; level 13, swap back to Tor at his old spot; level 15; she resets to code 0 with the stun count at 0.

## Reach and distances (boss reach pass)

She has no weapon volume and no melee hit (row 128 has no weapons, and no clip of hers is tested for weapon bits); every reach below is a distance test or a projectile. Distances are horizontal unless marked 3D. Confidence: High for the values (they are the controller's own constants, gathered here), Medium for the worked tick.

- Final fight, shove (code 2): chosen whenever the hero is within 10.0 (it overrides the cooldown); no damage. On the first tick of animation 9 the hero's velocity becomes (6.0 d, 8.0, 6.0 d) in motion-record units (0.2 units per tick out and 0.267 up, before gravity), with a push out of her body when he is closer than her radius 1.175 plus his radius plus 0.1. So the hero is kept beyond about 10 from her while she is free; codes 5 and 6 (stun) skip the shove.
- Orbs (codes 3 and 4): open at her mouth point on cast tick 24, move 0.75 per tick toward the hero's aim point (re-aimed every tick, so they follow him in 3D; walking at 0.225 per tick never escapes). An elemental orb strikes when the 3D separation before a step is under 1.5 (10 damage, once); the hit comes on slot step floor((s0 - 1.5) / 0.75) + 2 after the release for a first-step separation s0. A bolt acts under 2.0 (step floor((s0 - 2.0) / 0.75) + 2) and puts a Minion Warrior 4 units from the hero on her side. The slot rules (Projectiles) name no lifetime, so an orb ends only by a hit, a block, a reflection or a summon. A blocked orb (guarding, facing it within 60 degrees, matching shield Mote) flies back at 0.75 per tick to her aim point and stuns her under 1.5.
- Her body: cylinder radius 1.175, height 3.5 from her position upward (hero shots, the absorb window); the stun hit needs the hero's weapon cylinder (radius 1.0, length weapon reach plus 2.0) to reach one of five upper-body capsules within the 65.0 3D gate, so a swing at her feet can miss. Hover heights 15, 10 and 5 above the ground; Roberta's lock-on reaches her only from 3.9 to 13.2, 2.6 to 17.3 and 1.3 to 19.4 horizontally at those heights (Roberta's shots against the hovering Faerie).
- Minion Warriors (row 204; the common Minion Warrior tables, SPEC-0038): slot 0 (window s + 28 to s + 39) touches a level hero 1.1 to 10.5 ahead of where it started; slot 1 charge strike (window c + 24 to c + 35) 1.4 to 11.4; slot 2 (windows s + 16 to s + 27, s + 44 to s + 59, s + 76 to s + 91) -0.7 to 7.7, -1.2 to 10.4 and 2.3 to 12.2; chooser ranges 4.0 (slots 0 and 2, band 3 to 10) and 8.0 (charge, band 10 to 17); strafe band 12 to 20. Minion Grunts: the common Grunt reach (SPEC-0038).
- Queen Fyora's chamber (pack 1428): the hero is thrown (velocity (6.0 d, 8.0)) on every tick he is within 15.0 while animation 9 plays; a Queen Fyora orb within 1.5 starts her deflect; her own type-0 orb flies flat at 0.5 per tick toward Queen Fyora; the type-2 orb homes at 0.5 per tick and summons a Warrior at the hero under 2.0; a deflected orb flies 0.5 per tick for up to 60 ticks (30 units) and deals 5 within 1.5.
- Pack 1174: she throws the hero (velocity (9 d, 12.0), length 15.0) when he is within 10.0 and the 180-tick countdown is spent; no damage.
- Arena limits: she never moves horizontally, so every reach is measured from her fixed point; the hero is only ever pushed away from it.
- Safe spots: none from an orb except blocking it (or another character taking it); beyond 10.0 from her the hero is never shoved; under her at hover height 15 Roberta cannot lock on (inside 3.9).
- Worked example: hover code 0, the hero standing still and not guarding, first-step separation s0 = 30.0 when the orb opens on cast tick 24 of a pick made on tick P: the orb strikes on slot step 40, that is on tick P + 64 (Medium for the one-tick offset between the cast tick and the first slot step), for 10 damage.

## Player summary

- She cannot be damaged. The fight is won by turning her orbs back and then by Roberta's shots. Have a shield and a shield Mote equipped (the Nova Mote answers every orb colour).
- Keep at least 10 units away while she casts; closer than that she shoves you down. Hold the shield up, facing the orb, so that it is up the tick before the orb arrives; a reflected orb stuns her for about 3 seconds. A bolt cannot be turned back (it makes a Minion Warrior beside you, which drops a Nova Mote when it dies).
- While she is stunned, walk up and land Tor's third chain swing, a gesture swing or the air plunge on her; the first two swings only make her flinch. Three completed stuns switch you to Roberta.
- Roberta must put three shots into the risen Faerie within the 50-second window (each within about 3 seconds of the last; she needs magic for each), then walk to the spot where the Faerie lands within 15 seconds. The whole fight runs on a 180-second clock; at zero the game ends, and any hero death is game over.

## Where it lives on the disc

- Monster table: resource 530. Rows 128 (variant 0, the final-fight placement) and 129 (variant 1, the earlier-encounter placement). The row layout is the common enemy row layout of SPEC-0029. Rows are found through the per-type first-row index (type 56 maps to row 128, and variant 1 is the next row).
- Tuning values: 32-bit words in the boot program's data, read at runtime by virtual address. The Faerie's words are at 0x2DA618 to 0x2DA64C and at 0x2DAAB0 to 0x2DAAB8. The disc has no names for them; the names in the Layout section are descriptive.
- Animations: "animation n" means entry n of the boss's own animation table (SPEC-0005).
- Placements (script side):
  - Script pack 1432 (final fight): a type-32 world-character record named "Darkest Faerie" with behavior type 56, variant 0. Boss-fight instance 13.
  - Script pack 1174 (earlier encounter): loads region 1175 at controller start (variant 1, row 129). Boss-fight instance 3.
  - Script pack 1098: loads region 1108 (variant 0) only while game state 30 is set and game state 31 is clear. Its hero-death callback restarts that pack and posts a message to its world; whether this placement is a fight remains open.
  - Script pack 1344: region 1349 (variant 0, "The Darkest Faerie"). No load of that region is established in the local world-load graph; a direct or unknown load remains possible.
  - Script pack 1428 (Queen Fyora's chambers, instance 12) is an active scripted trap encounter. It places Queen Fyora and the Darkest Faerie together, then deactivates both and advances the Act 4 sequence at the scripted exit. It is not a normal boss fight that ends through the Faerie's hit points.
  - A block in pack 1259 holds a variant-0 record with no name. Pack 1259 is not reached in the local world-load graph, so an active placement is not established; a direct or unknown load remains possible.
- The hero, the arena and the return world: the hero is the player's character. The return world for both finale and earlier-encounter movies is world slot 151.

## Layout

### Monster row fields (rows 128 and 129)

Offsets are from the start of the row. Values are as stored (High).

| Field | Offset | Type | Row 128 | Row 129 | Meaning |
| --- | --- | --- | --- | --- | --- |
| Hit Points | +0x30 | u16 | 5 | 5 | Starting and maximum hit points |
| Radius | +0x28 | f32 | 1.175 | 1.175 | Body radius |
| Height | +0x2C | f32 | 3.5 | 3.5 | Body height |
| Draw Distance | +0x32 | u16 | 300 | 300 | |
| Low Res Distance | +0x34 | u16 | 200 | 200 | |
| Default Behavior | +0x36 | u16 | 37 | 37 | Behavior class (SPEC-0029 class 37) |
| Max Fly Height | +0xB4 | f32 | 100 | 100 | |
| Attack 1 Spawn Type | +0x3CC | u16 | 2864 | none | Summon key (SPEC-0029 Behavior 31) used by the detonation of an aimed bolt |
| Attack 1 Spawn Count | +0x3D4 | u16 | 4 | 4 | Listed count; see Open questions |
| Attack 2 Spawn Type | +0x3CE | u16 | 2844 | none | Summon key (SPEC-0029 Behavior 31) used by the burst |
| Attack 2 Spawn Count | +0x3D6 | u16 | 4 | 4 | Burst entity count |
| Death Disappear Timer | +0x80 | u16 | 510 | 510 | Same value on both rows |

The attack type and speed fields of both rows are 0 or unset. The attack choice is therefore in the behavior class, not in the row (Medium).

### Tuning words (boot program data)

Each word is a 32-bit value. "f" is an IEEE single, "i" a signed 32-bit integer.

| Address | Type | Value | Name used here | Use |
| --- | --- | --- | --- | --- |
| 0x2DA618 | f | 1.0 s | cooldown minimum | Lower bound of attack cooldowns |
| 0x2DA61C | f | 4.0 s | cooldown maximum | Upper bound of attack cooldowns |
| 0x2DA628 | f | 50.0 s | absorb window | Length of the absorb window |
| 0x2DA62C | f | 3.0 s | absorb hold | Hold after each absorb |
| 0x2DA630 | f | 10.0 | dive speed | Shove speed (code 2; the hero's speed after the shove); the loop A knockback uses this times 1.5 |
| 0x2DA634 | i | 10 | orb hit amount | Damage of an elemental orb hit (not an effect id: no effect definition 10 exists) |
| 0x2DA638 | f | 0.75 per tick | projectile speed | Speed of all three projectile kinds |
| 0x2DA63C | f | 3.0 s | stun time | Stun after a reflected orb |
| 0x2DA640 | f | 15.0 | hover height, count 0 | Target height above the ground probe |
| 0x2DA644 | f | 10.0 | hover height, count 1 | |
| 0x2DA648 | f | 5.0 | hover height, count 2 | |
| 0x2DA64C | f | 0.05 per tick | descent step | Timeout descent speed |
| 0x2DAAB0 | f | 2.4 | loop B sidestep marker half-span | Scales each in-plane half-axis of the camera-facing quad drawn for the deflected projectile (loop B code 2); it does not change projectile collision or damage |
| 0x2DAAB4 | f | 15.0 | loop B knockback range | Horizontal range that starts the hero knockback (code 4) in loop B |
| 0x2DAAB8 | f | 10.0 | loop B knockback speed | Speed the loop B hero knockback is rescaled to |

Fixed values used by the behavior (not stored on the disc; High unless marked):
- Spawn offset of the burst: 4.0 units on two axes.
- Ground probe: taken 8.0 units above the boss.
- Aimed bolt detonation distance: 2.0 (3D).
- Orb arrival distance: 1.5 (3D). Reflected orb arrival distance to the boss: 1.5.
- Facing test for blocking: dot product above 0.5.
- Fall step: 0.1 per tick.
- Aim blend for facing: 0.25.
- Shove push-out: 5 times the shortfall when the hero is inside the sum of the two radii plus 0.1. Shove steps 9.0 (horizontal, along the direction to the hero) and 12.0 (vertical) (High for the lanes).
- Chooser odds for an aimed bolt: 1 in 11.
- Orb kinds: 1 to 6, uniform.
- Loop B cooldowns: 0.5 to 0.75 s for code 3; 2 to 4 s for code 5.
- Loop A knockback speed: 1.5 times the dive speed (15.0).
- Loop A countdown: 3 s, set after each knockback.
- Loop B proximity to a class-38 entity: 1.5 (horizontal).
- Frame rate: 60 ticks per second (SPEC-0007).

### Orb kinds

Kinds 1 to 6 are elemental orbs. Kind 0 is the aimed bolt. All ids in this table are effect ids of the shared effect group (resource 9; SPEC-0034), listed as decimal numbers (High). None of them is a sound id.

| Kind | Flight effect, impact effect | Cast glow effect |
| --- | --- | --- |
| 0 (aimed bolt) | 2127, 412 | none |
| 1 | 1603, 991 | 1617 |
| 2 | 1605, 993 | 1619 |
| 3 | 1608, 988 | 1622 |
| 4 | 1604, 992 | 1618 |
| 5 | 1606, 989 | 1620 |
| 6 | 1607, 987 | 1621 |

- The flight effect starts when the projectile is launched and follows it. When the slot clears, the flight effect is drained and the impact effect plays once at the projectile's last position.
- The cast glow is held on the boss while she casts an orb of that kind (see Sounds and effects). Each glow is five looping emitters (two coloured sprites, a halo, a large sprite and a point light of radius 15) in the colour of the element.

The burst summons and the aimed bolt's summon pass direct drop item 172 (Nova Mote; SPEC-0029 Behavior 31), so each summoned character drops a Nova Mote when it dies. 172 is an item id, not an effect id.

### Hero attack-kind table (used for blocking)

The item id of the Mote in the hero's equipment slot 1 (the shield Mote word) maps to an element kind; an empty slot or any other item gives kind 0. Ids are given as decimal numbers.

| Mote item id | Kind |
| --- | --- |
| 160, 161 | 1 |
| 162, 163 | 4 |
| 164, 165 | 2 |
| 166, 167 | 5 |
| 168, 169 | 6 |
| 170, 171 | 3 |
| 172, 173 | 7 |
| any other | 0 |

Kind 7 blocks every orb kind. (SPEC-0036 defines the hero's attacks.)

## Behavior

### Routing (each tick)

The boss's behavior class runs one of three controllers, chosen in this order:
1. If game state 364 (the progress flag) is set: the main controller.
2. Otherwise, if level state 18 is set: loop A.
3. Otherwise: loop B.

Game state 364 is set by script pack 1432 at init, and by the entry sequence (which also sets level 11 and clears level 10). Level state 18 is set by script pack 1174 at init. So the final fight runs the main controller, and the earlier encounter runs loop A.

Confidence: High.

### Main controller: one tick

Each tick, in order:
1. Clear the boss's per-tick shared movement-steering flag.
2. Run each active projectile slot (see Projectiles).
3. Run the chooser: a gate, then the choice (see Chooser).
4. Run the handler of the current member code (see the code table).
5. Yield one frame.

The controller owns four projectile slots. Its four-slot state block is cleared once at controller entry, before the frame-yielding loop; the clear does not run again on each frame of the live controller (High; see SPEC-0029, Behavior 7). A separate controller restart would clear it again. Each slot holds one projectile at a time.

The flag is not a hit-eligibility gate. The shared steering update uses it to decide whether the second controller-input axis contributes to the vertical component of the actor's movement vector. The main controller clears it before dispatch; behavior handlers that set it enable that contribution for the current update. It does not bypass the boss's invulnerability or change the code-5 weapon-reach test (High for the steering consumer and the separation from hit handling; the broader authored purpose of this shared field is not established).

### Member codes and timings

The boss has eleven codes (0 to 10). Each code has its own timers. Animation numbers are the boss's own table entries.

| Code | Name | Animation | What happens | Next |
| --- | --- | --- | --- | --- |
| 0 | idle | 6 (aim blend 0.25) | Faces the hero | A chooser pick |
| 1 | burst | 6 every tick | Waits for level 14 (or 15). On level 14: clears level 14, sets the absorb count to 0 and the window timer to 3000 ticks, clears level 10, and makes four summon requests of key 2844 (SPEC-0029 Behavior 31) at 4 units in front of her, 4 behind, and 4 to each side along her side axis, with direct drop item 172 (Nova Mote) and her forward direction. No record with key 2844 is summon-reserved in the final-fight pack, so the requests spawn nothing there (see Open question 6) | Code 7 when the window starts. Level 15 set: clears levels 15 and 10, resets the stun count, clears the chooser hold flag, code 0 |
| 2 | lunge (shove) | 9 | When the hero is near (under 10 horizontal). On the first tick pushes the hero away (velocity 6.0 along the direction to the hero, 8.0 up; knocked state); once per animation. She does not move. It has no hit routine and deals no damage | Code 0 when animation 9 ends |
| 3 | orb volley | 7 | On the first tick of animation 7, starts the cast glow of the chosen kind on her. At the animation's event (key 6, tick 24 after the request), fires one elemental orb of that kind (random 1 to 6) from her mouth point, or an aimed bolt instead when the hero is guarding at that tick, and stops the glow. Cooldown 1 to 4 s | Code 0 at tick 65 (gate of animation 7) |
| 4 | aimed bolt | 8 | At the animation's event (key 8, tick 32 after the request), fires one aimed bolt. Cooldown 1 to 4 s | Code 0 at tick 93 (gate of animation 8) |
| 5 | stun | 14 (held), 13 (flinch) | Counts a 3 s timer. If the hit gates pass, goes to code 6 | Code 0 when the timer ends |
| 6 | stun count | 13 | Counts completed stun animations. The third sets level 12 and level 10 | Code 0 after the first two. After the third, the boss stays in this code until the gate sets code 1 on the next tick (see Stun) |
| 7 | absorb window | 14 (requested every tick) | Hovers and absorbs missiles for 50 s. Three absorbs: code 9 | Code 9 after 3 absorbs; code 8 at 50 s |
| 8 | timeout descent | none (animation 14 keeps playing) | Descends 0.05 per tick to the ground | At the ground: levels 13 and 10 set; code 1 |
| 9 | fall | 20 | Falls 0.1 per tick to the ground | At the ground: level 16 set; code 10 |
| 10 | landed | 20 | Waits while level 13 is clear | Level 13 set: level 10 set; code 1 |

Timings in ticks and seconds:
- Cooldown draw: a whole number of ticks from 60 to 240 inclusive (1 to 4 s), uniform.
- Absorb window: 3000 ticks (50 s).
- Absorb hold: 180 ticks (3 s).
- Stun timer: 180 ticks (3 s).
- Loop A countdown: 180 ticks (3 s).
- Descent at 0.05 per tick: 20 ticks per unit of height (3 units per second).
- Fall at 0.1 per tick: 10 ticks per unit of height (6 units per second).

The cycle that the level script drives is: code 1 (burst), code 7 (window), code 9 (fall after 3 absorbs), code 10 (landed), code 1 again. A timeout takes code 7 to code 8, then code 1.

Confidence: High for the transitions; Medium-High for the level-state meanings.

### Chooser

The chooser has two parts, run in order each tick.

Gate. If level state 10 is set, or game state 364 is clear, the gate sets the current code to 1 (burst). (Medium-High; in the final fight only the level-state test can be reached.)

Choice. The whole choice is skipped while the chooser's hold flag is set. The third completed stun count (code 6) sets that flag, and the level-15 return of code 1 clears it. Otherwise the cooldown counts down one tick (when above 0), and then, with the horizontal distance from her to the hero:
- If the distance is under 10 and the current code is not 5 or 6: the choice is code 2 (the shove), unless the current code is already 2. This branch tests neither the cooldown nor the ready test, so it replaces any other code (0, 1, 3 and 4, and 7 to 10 when the hold flag is clear, which the shipped cycle never has in those codes) on the tick the hero comes near, and it starts again on the tick after a shove ends if the hero is still near.
- Otherwise (distance 10 or more, or the current code is 5 or 6): nothing changes unless the current code is ready and the cooldown is 0. Then code 4 (aimed bolt) is chosen when a draw of 0 to 10 inclusive gives 0 (1 in 11) or when the hero's guard query is true (the hero is blocking); otherwise code 3 (orb volley) with a kind 1 to 6, uniform.
- Each new choice clears the code's first-tick flag.

Repeated shove (R-0069, High). The shove handler has three cases. If animation 9 is not the current animation, it requests animation 9 and does nothing else on that tick, so the push waits one tick. If animation 9 is current and its progress is below the 0.99 gate, it applies the push on the first such tick (guarded by the first-tick flag that every new choice clears) and clears the hero's collidable word on every such tick. If animation 9 is current and its progress has reached the gate, it sets code 0 and requests animation 6 in that same tick. The chooser runs before the handler, and while the current code is 2 it never picks again, so on the tick that sees the animation end the code is still 2 and nothing is re-requested. The request that sets code 0 also makes animation 6 the current animation, so on the next tick the chooser (hero still under 10) picks code 2 with animation 6 current, and the handler requests animation 9 as a real change from 6. The same-id rule of SPEC-0005 therefore never applies here: a request for animation 9 while 9 is current and finished does not occur, and there is no separate force-restart request on this path. Each shove starts animation 9 from its first frame, and one tick of animation 6 sits between shoves. A shove cycle lasts K + 1 ticks, where K is the number of ticks from the request tick to the first tick on which animation 9 reports its 0.99 gate. Animation 9 plays a one-shot clip of 39 keys (no loop), whose speed byte is 2, so at the normal play rate of 1.0 the cursor moves 0.25 key per tick (one key every 4 ticks) and reaches its last key, 38, 152 ticks after the request tick. Her controllers never change the play rate. The gate (progress at least 0.99 of 39 keys, SPEC-0005 Clip progress) first passes on that same tick 152, because progress counts the stored fraction twice: on tick 151 it is (37.75 + 0.5) / 39 = 0.981, on tick 152 it is (38 + 0.75) / 39 = 0.994. So K = 152 ticks (2.53 s), a shove cycle lasts 153 ticks (2.55 s), and a shove runs once per play of animation 9 plus one tick, not every second tick (High; Medium-High that no status effect slows her clip). The push lands on tick 1 of each cycle (the tick after the request), on every shove while the hero stays under 10 units away.

Ready codes are 0, 7 and 8 (and any value below 1 or above 10). Codes 1 to 6, 9 and 10 are not ready. Codes 5 and 6 are sent to the far branch only so that the ready test then fails: the chooser never changes code 5 or 6, so the stun and the stun count cannot be cut short by the shove or by a new attack.

Confidence: High for the gate, the cooldown draw, both branches and the ready test.

### Projectiles (the four slots)

All three projectile kinds move at 0.75 units per tick (45 units per second). Each moves toward its target.

- Elemental orb (slot type 0; kind 1 to 6). Moves toward the hero's aim point each tick (re-aimed every tick). The tests use the 3D separation measured before the step. When it is under 1.5 units:
  - If the hero is guarding (the guard predicate of SPEC-0036: shield state, Roberta's spins, or the automatic guard timer above 0) and faces the orb (the dot product of the hero's forward direction and the horizontal unit direction from the hero to the orb is above 0.5; the vertical part is dropped) and the element kind of the Mote in the hero's equipment slot 1 (the shield Mote word; SPEC-0036, Mote elements) equals the orb kind or is 7, the orb is blocked. A hero who is not guarding takes the hit whatever the Mote. The facing test uses the orb's travel direction of that tick, flattened and reversed, against the hero's forward axis. The guard and facing the test sees are those left by the previous tick. A blocked orb becomes a reflected orb (slot type 2) in the same slot, from the same position, and is aimed at the boss's aim point (her attachment point 7 by default); the block spends no Mote charge and no MP, and plays no hero reaction.
  - Otherwise the orb hits the hero once for 10 (the orb damage, word 0x2DA634; the 10 is not an effect id, since no effect definition 10 exists), and the slot clears. The 10 goes straight to the hero's take-hit with no attacker (attacker id -1); like other boss hits it is not reduced by the hero's shield, armour, armour Mote or petpet factor (SPEC-0029 Behavior 10.5). These four slots belong to the boss, not to the shared enemy projectile pool.
- Aimed bolt (slot type 1; kind 0). Moves toward the hero. The bolt itself deals no damage. When the 3D separation before the step is under 2.0 units, the slot clears and a summon is made by the row's key 2864 (row +0x3CC; SPEC-0029 Behavior 31) at the hero's position plus 4 units along the horizontal unit direction from the hero toward her, facing the hero (the opposite direction), with direct drop item 172 (Nova Mote). The bolt's own position is not used. Any damage comes from the summoned character. On disc, key 2864 is carried by Minion Warrior spawn records (monster row 204: weapons 1 to 3 radius 3.0, length 2.0; attack damage 7, 7 and 15; class 17), which hit through the shared enemy chain (SPEC-0029 Behavior 10.5). The final world holds four reserved records with this key, and the summon binds the first free one (High).
- Reflected orb (slot type 2). Moves toward the boss's aim point at 0.75 per tick. When the separation before a step is under 1.5 units, the slot clears and the boss is stunned: code 5 with a 180-tick timer and the flinch latch clear, whatever code she was in (a cast, bolt, shove, window or fall is cut off). This is her weak point.

The flight and impact effects of each slot are given by the kind table. The slots play no sound: the flight effect is drained when the slot clears, and the impact effect then plays at the slot's last position.

Confidence: High for the speeds and distances; Medium-High for the raw orb hit and the summon by key; High for which spawn record the final world resolves for key 2864 (the first free of four reserved Warrior records).

### Attacks, summary

- Elemental orb volley (code 3): one orb of a random kind 1 to 6 at the animation's event. Then a cooldown of 1 to 4 s.
- Aimed bolt (code 4): one bolt at the animation's event. Then a cooldown of 1 to 4 s.
- Lunge (code 2), which is a shove of the hero: she does not move. On the first tick of animation 9, with d the horizontal unit direction from her to the hero and h the horizontal distance: the hero's motion-record velocity is set to zero and 12.0 is added to its vertical (y) lane. If h is less than (her motion radius + the hero's motion radius + 0.1), the hero's position is moved along d by 5 times that shortfall (out of her body). Then 9 x d is added to the velocity, and the velocity is normalized and scaled to 10.0 (the word 0x2DA630). The result is (6.0 d.x, 8.0, 6.0 d.z). The hero is put into its knocked state (hero action state 0x30), the same push as the Ixi Chief's (SPEC-0014, Push). On every tick of animation 9 the hero's collidable word (SPEC-0040) is cleared. The value is a motion-record velocity: each tick the position moves by velocity x 1/30 (SPEC-0040 Behavior 14A), so the shove starts at 0.2 units per tick along d and 0.267 units per tick upward, before gravity. The shove has no hit routine, so it deals no damage to the hero (High).
- Burst (code 1): see the code table. The burst itself deals no damage; it makes four summon requests of key 2844. On disc, key 2844 belongs to a Minion Grunt record (monster row 183; attack damage 25, 25 and 50; class 17). In the final-fight pack that Grunt record (entity 6) is an ordinary placement whose summon byte is 0, and a summon request binds only records with a nonzero summon byte, so all four requests find nothing and no character appears (High for the data and the search rule). The Grunts the player meets are the three placed by the script (Fight flow, control thread B and the director).

Confidence: Medium-High for the numbers; High for the shove's lanes and units.

### Sounds and effects (Confidence: High unless marked)

**Sounds come only from the animation tables.** None of her three controllers starts a sound of its own, and her projectile slots play none. Her main animation table (regions 1108 and 1349, identical) names cues of creature block 388 (SPEC-0031). Every sound entry has start frame 0 and no keep-alive bit, so when her main animation changes to an entry with a sound, that cue plays once, at the boss, on the first tick of the animation. A request of the animation that is already playing starts nothing. No sound plays while she is in base state 42 (the animation-sound step is skipped there; SPEC-0031).

Animation to cue of block 388 (animations not listed have no sound):

- 4 and 12: cue 0 (laugh).
- 7 (code 3, orb volley): cue 2 (magic bolt).
- 9 (code 2, lunge): cue 4 (rod blast).
- 13 and 18: cue 1 (take damage).
- 16: cue 5 (disappear).
- 22: cue 3 (Fyora shoot).
- Silent: 6 (idle and the code 1 wait), 8 (code 4, aimed bolt), 10, 14 (the stun hold, the absorb window and the timeout descent) and 20 (fall and landed).

Block 388 has no unused cue: every cue is named by the table.

**Earlier-encounter record.** The variant-1 record of region 1175 (script pack 1174) uses the Faerie Sisters' block 387 instead: animations 4, 10, 11, 12 and 20 play cue 0, 7 plays cue 7 and 9 plays cue 14 (the three sisters' laugh cues; SPEC-0011).

**Minion Warrior (block 377).** The summoned Minion Warrior's table names cues of block 377: animations 6, 10, 19, 53, 54, 55 and 82 play cue 0 (appear); 34 cue 3 (swipe); 36, 41 and 57 cue 6 (take damage); 40 cue 4 (big swipe); 44 cue 5 (claw flurry); 58 cue 7 (die); 59 cue 8 (wall slam); 81 cue 1 (vanish). Cue 2 of block 377 is named by no table and never plays. Same rules as above (start frame 0, one shot, at the character).

**Effects started by the code.**

- **Cast glow.** On the first tick of animation 7 (code 3), the glow effect of the chosen orb kind (1617 to 1622, see Orb kinds) starts at her cast point and stays on her until the orb is released at the animation's event, when it is stopped. The aimed bolt (kind 0, animation 8) has no glow.
- **Orb visuals.** Each slot starts its kind's flight effect at launch; when the slot clears, the flight effect is drained and the impact effect plays at its last position (see Orb kinds).
- The burst's summons start no effect of their own; 172 is their direct drop item (Nova Mote), not an effect id (see Orb kinds).

**Clip effect lists** (SPEC-0034; Medium-High for which list belongs to which clip): her own clips carry no clip effect list entries. The Minion Warrior's die animation 58 shows effect 1623 at 0.9 of the clip (sparkles with a light), its vanish 81 shows 1654 at 0.83 and its appear 82 shows 1646.

### Absorb window and falls (codes 7, 8, 9)

Code 7 (absorb window):
- Each tick of code 7, in this order: the window timer drops by 1 and, at 0 or below, the code becomes 8 and the tick ends (no animation request, no hover step, no scan); animation 14 is requested; the hold step (below) and the target height from the count; the hover step; the missile scan.
- For 50 seconds, she hovers at a target height above the ground probe. The target depends on the absorb count before the scan: 15 units for count 0, 10 for count 1, 5 for count 2 (from the tuning words). The hover step moves her 0.05 per tick toward the target and sets her exactly on it when within 0.05. She starts the window on the floor, so it takes 300 ticks to reach 15.
- Each armed hero shot slot (six slots of the selected hero, Roberta's shots) that overlaps her body is removed and adds one to the absorb count, including during the hold; every overlapping slot on a tick counts, and the fall starts when the count is exactly 3. The overlap test treats each armed hero shot slot as a sphere (the slot point and the slot radius) against her vertical body cylinder (her motion radius and height). Every absorb sets the hold to 180 ticks; the hold does not pause the missile scan.
- Hold step: with count 1 or 2 the hold drops by 1 each tick. When it reaches 0 the count drops by one and the hold restarts at 180 (count 2 becomes 1, count 1 becomes 0); the target height of that tick is still the one for the old count. With count 0 nothing counts down. So an absorb adds to the count only if it comes within 179 ticks of the previous one: on the 180th tick the drop is applied before the scan, so that absorb just restores the dropped count.
- Three absorbs: code 9 (fall). The window timeout at 50 s: code 8 (timeout descent).

Code 8 (timeout descent): descends at 0.05 per tick to the ground. At the ground: sets levels 13 and 10, then code 1.

Code 9 (fall): descends at 0.1 per tick to the ground, with animation 20. At the ground: sets level 16, then code 10.

Code 10 (landed): waits while level 13 is clear. When level 13 is set: sets level 10, then code 1.

Confidence: High for the states, steps and per-tick absorb scan; Medium-High for the missile overlap test, including its side and end boundaries (see Open questions).

### Stun and the reflect weak point

- The weak point is the reflection. A hero who blocks an elemental orb (see Projectiles) turns it into a reflected orb. When the reflected orb reaches the boss, the boss enters code 5 with a 3 s timer. Stun length is 180 ticks.
- Code 5, each tick (High), in this order. (1) If the hero's in-attack test is false, the flinch latch is cleared. (2) If it is true, the hero's attack form is form 2 and the hero's weapon reaches her body (the weapon sweep of SPEC-0029 Behavior 10.2 against her five hurt capsules, tried only when the hero is within 65.0 units), she enters code 6 at once, before the timer drops. (3) The timer drops by 1. (4) If animation 13 is not playing she requests 14 and, with the timer at 0 or below, enters code 0. If animation 13 is playing and unfinished, nothing more happens (so with the timer spent she stays until the flinch clip finishes, and step 2 still applies). If it is playing and finished, she enters code 0 when the timer is spent, otherwise requests 14. (5) On a tick where animation 13 was not playing and the timer is still above 0: if the in-attack test is true, the latch is clear and the weapon reaches her, the latch is set and she requests 13 (form 2 was already taken in step 2).
- Attack form 2 is Tor in attack sub-states 4, 7, 8 or 12, or Roberta in attack sub-state 4 (SPEC-0036). Sub-state 4 is the third swing of either hero's chain.
- A hit by any other attack form plays animation 13 once (a flinch) and leaves her in code 5. A flag stops a second flinch until the hero's in-attack test turns false. When animation 13 ends she returns to animation 14 while the timer runs.
- When the timer reaches 0 without a form-2 hit, she returns to code 0. A stun hit deals no damage (see Hit points).
- In code 6, each completed stun animation adds one (animation 13 is requested if it is not the current one; when a flinch 13 is already playing, the count completes when that clip finishes, and when it is current and finished the count is added on the first tick). The first two return to code 0. The third sets level 12 and level 10 and leaves the boss in code 6. On the next tick the per-tick gate sees level 10 set and sets code 1. The chooser runs after the gate in that same tick, so a hero under 10 units away can still turn that code 1 into code 2 (see Chooser).
- The stun count is cleared only when code 1 runs with level 15 set (see the code table). The count is therefore kept per cycle, not per fight.

The stun does not check the boss's height (Medium).

Confidence: High for the reflect and the 3 s timer; Medium for the stun count rule; Low for the gates.

### Level and blackboard signals

Level states used by the controller (meanings Medium-High):
- Level 10: a burst request. The chooser gate sets code 1 when it is set. Code 1 clears it. Set by code 6 (third stun), by code 8 at the ground, by code 10 when level 13 is set, and when pack 1432 activates. Cleared by the entry sequence (see Fight flow).
- Level 11: set by the entry sequence. It is never cleared in pack 1432. Control thread B (see Fight flow) reads it and writes no levels (High).
- Level 12: set on the third completed stun. The director clears it at stun-swap completion.
- Level 13: set by code 8 and by the director at the end of the landed window. Code 10 waits for it. The director clears it when the landed-miss swap completes.
- Level 14: set by the director at stun-swap completion, to start the burst and the window. Code 1 clears it.
- Level 15: set by the director at landed-miss swap completion. When code 1 sees level 15 set, it clears levels 15 and 10, resets the stun count, and returns to code 0.
- Level 16: set by code 9 at the ground. The director clears it at the end of the landed window.
- Level 18: selects loop A (see Routing).

Game states:
- Game state 364: progress flag (routing). Set at init of pack 1432, and by the entry sequence.
- Game state 358: set at init of pack 1432. It is the Ixi Chief's fight-mode-2 flag (progress flag 0x166, SPEC-0014), and pack 1432 is its only writer (High).
- Game states 94 and 365: set at the finale.
- Game states 219, 220 and 221: set in the earlier encounter.

Blackboard messages:
- Finale: message type 2 with value 80 for world 151, posted by the finale block.
- Hero death in pack 1098 posts message type 2 with value 120 for its own world (a different placement; see Edge cases).

Confidence: Medium-High for the level-state uses; Low for the meaning of each state.

### Loop A (level state 18 set, game state 364 clear)

- Animation 0. A countdown runs down each tick, set to 180 ticks after each knockback.
- When the hero is under 10 units away and the countdown is zero: hero knockback. The boss does not move. The hero is put in state 48 and given a velocity of 12.0 up and 9.0 along the horizontal direction from the boss to the hero, with a -5.0 push-out when the hero is inside the sum of the two radii plus 0.1; the velocity is then rescaled to 15.0 (10.0 times 1.5). The countdown is reset to 180 ticks, and the controller returns to its waiting state.

Confidence: High for routing, the knockback and the push-out lane (5 times the shortfall, along the direction to the hero).

### Loop B (level state 18 clear, game state 364 clear)

- Waits until an entity with behavior class 38 is in the character list. Monster row 132 (type 58, 50 hit points) is Queen Fyora. Then runs a chooser and codes 0 to 5 each tick.
- The loop B chooser: if a class-38 entity of projectile type 1 is within 1.5 units of the boss, the choice is code 2 (a sidestep with a projectile). Otherwise, if the hero is under 15 units (from the tuning word), the choice is code 4 (hero knockback, animation 9: the same knockback as loop A, rescaled to 10.0 instead of 15.0 and without the body push-out; the hero is put in state 48 and a burst effect plays 0.5 above the boss). The choice is made, and the first-tick flag cleared, on every tick the hero is under 15, so the knockback applies on every tick of animation 9 while the hero stays inside 15 (not once); once animation 9 has finished and stays current, the knockback does not repeat until the hero has been outside 15 for a tick. Otherwise the choice is code 3 or code 5 by a test.
- Code 3 (loop B): animation 7 (17 keys; release at tick 24, gate at tick 65); one projectile at the event; cooldown a whole number from 30 to 45 ticks (0.5 to 0.75 s).
- Code 5 (loop B): animation 7; one shot at the event (tick 24); cooldown a whole number from 120 to 240 ticks (2 to 4 s).
- Code 2 (loop B): animation 10 (16-key loop; gate at tick 63); one projectile per side-step, relaunched on the tick after the state starts. Its deflected projectile uses a camera-facing quad marker whose in-plane half-axes are scaled by 2.4, giving a 4.8-unit full span along each unit basis axis. This scale changes the rendered marker only, not the projectile's direction or collision test.
- Start values: the state, cooldown and flags of loop B are not initialised (they hold whatever was on the stack). Default if unresolved (rewrite choice): state 0, cooldown 0, all flags clear.

Synthetic check for the marker: for centre C and perpendicular unit in-plane axes R and U, the four corners are C ± 2.4R ± 2.4U. Opposite corners span 4.8 along R and 4.8 along U.

Confidence: High for routing, Queen Fyora row identity and the knockback range; Medium for the codes. Projectile-slot behavior is specified in SPEC-0029, Behavior 19.

### Hit points and how they drop

- Hit points are 5 at spawn, current and maximum (the row value, High). They never change in play (High; see the next item).
- The controller never reads the boss's hit points. There are no health phases and no percentage thresholds in this boss. Phases are the absorb count (0 to 3), the stun count (0 to 3), and the level states.
- She is invulnerable for her whole life (High). Each of her three controllers sets her invulnerable flag (the flag of SPEC-0030 row 132) when it starts, and nothing clears it: no routine of her class does, and pack 1432 makes no invulnerability call. While the flag is set her hit-candidate test fails, so the hero's melee scan, shot hits and power effects skip her and never call her take-hit step (High for the melee scan, Medium-High for the other hero hit routes). So she loses no hit points, inside or outside her stun. The stun is not a damage window: the stun hit only moves her to code 6 (see Stun and the reflect weak point).
- Death path, not reached in play. If her hit points did reach 0, the shared AI death method would set her base state to 16 (she is class 37, not one of the exempt classes 1, 11 and 32), and her row's death-countdown fields are 0. No flag, level state, timer or callback in pack 1432 reacts to her death. The fight is won only by the finale and lost only by the timeout (see Fight flow).

Confidence: High for the stored value and the absence of health phases; Medium-High that no run-time write exists (see Open questions); Medium for the death path.

### Finale (script pack 1432)

Signals in the final fight. The finale block hides the fight clock first. It also sets game states 94 and 365, deactivates object 1 and entities 0, 5, 6 and 7, sets the hero swap mode to 2, sets boss-fight instance 13 to stage 1 (through the boss-fight tweak call with the stage), sets the return world to 151, posts blackboard message 2 with value 80 for world 151, and plays movie 85 and waits for it to finish. Flag 6 is set after the movie ends.

The finale block has one call site in this pack: the fight control thread, under the script-local flag 4 gate (see Fight flow). Control thread B does not run it.

Stage 0 is set at the boss's setup. Movie 78 plays in the entry sequence, before the fight clock starts (High). Movie 80 is the timeout ending and never plays before the fight (see Fight flow).

Confidence: High for the calls; Medium for reading the finale as the win; High that the trigger-1 volume exists, High (no play check) that the finale runs in play when the hero reaches it during the landed window (see Trigger 1 and the finale gate).

### Fight flow (script pack 1432)

This is the script side of the final fight. It runs beside the boss's controller above. The flags named here are the script-local flags of pack 1432 (SPEC-0049 calls them level flags). All of them are clear when the pack loads. Times are seconds at 60 ticks per second. "The fight clock" is the countdown timer described below. Confirmed on the disc (High): the 180 s clock, the order of the rules, the flag sites, and the single call site of the finale.

Triggers. The pack registers two script triggers at init, by index: index 0 and index 1.
- Trigger index 0 (the fight entry) fires from one volume in the final-fight world: a flat vertical rectangle, 20.85 units tall and 37.25 units wide. Its reference corner is (36.40, 269.20, -343.99). Its normal is (-1, 0, -0.0024), so it stands across the x direction; its height runs along +y and its width along (-0.0024, 0, 1). The rectangle is static (its switch value is zero; SPEC-0028 section 13). When the selected hero's movement segment crosses it, the hero's own body contact (the hero class's trigger-hit method) fires the trigger with the callback object set to 1, not 0. The entry only sets flag 0, so the object value does not matter for it. Confidence: High for the record values (each record carries its own header, and this one's flags and index are confirmed) and High for the fire path.
- 52 of the 53 cylinder volumes of the final-fight world (flags 0x0001) never fire a script trigger. For the hero they run the use-press step (the hero state goes to 13 and the mode comes from the volume's mode byte) and the contact push. The 53rd is the trigger-1 cylinder below. Confidence: High.
- Trigger index 1 (the finale trigger) fires from one volume in the final-fight world: cylinder record 8 of the cylinder list (kind 3, mode 0, flags 0x0008, gate mask 0, trigger index 1). Its base point is (28.702, 270.608, -326.005), its axis is (0, 1, 0), its length is 3.0 and its radius is 6.0. It stands on the arena floor (ground height 270.628), on the boss's side of the arena, inside the arena's walk-mesh footprint (x from -34.5 to 35.1, z from -363.5 to -287.5) and 7.7 units inside the trigger-0 rectangle. Its centre is the position of locator 2 (28.713, 272.489, -326.005), where the entry sequence moves entity 0. The hero fires it when the hero's body cylinder (radius 1.0, 3.5 tall from the feet; SPEC-0018) overlaps it, that is, within about 7.0 units of its axis while standing on the floor. Confidence: High for the record values; High for the fire path (no play check).
- The Darkest Faerie's placement record (monster type 56, variant 0, position (57.66, 270.64, -326.07)) carries no volume and no trigger field. The final-fight module calls no force-trigger host function and no slot load or slot create function. The world pack has no character region of its own; the Faerie is a world character in the pack's embedded region. Confidence: High.

Activation. Before Init, the new world's 64 level-state flags are clear, and pack 1432's script-local flag word is zero (High; see Open question 24). When the pack activates, it sets level state 10 on, activates objects 0, 1 and 2, and sets flag 9 on. Init sets game states 364 and 358 on. Init order (High): it sets game states 364 and 358 on; allocates two control threads and starts the fight control thread and control thread B; registers script trigger index 0 (its callback sets flag 0) and index 1 (its callback sets flag 4 when its object argument is non-null and flag 8 is set); registers the activate-world callback described above; registers two entity activation filters; and last stops effects 3 and 4. The activate-world callback also starts the world effects 18 to 25 (the same effects the director starts during its swap). A world keeps one activation filter, so the second registration replaces the first: the effective filter refuses entities 5, 6 and 7 (the three Minion Grunts) and admits every other entity, so streaming never binds the Grunts by itself and only control thread B and the director place them (control thread B and director sections; a forced bind ignores the filter). The first filter admits every entity and is never in effect. No hero-death callback is registered.

Fight control thread. Each tick, the fight control thread checks these rules in order. The first rule that applies takes the tick, and the rest are skipped.
1. Flag 6 set: nothing. Flag 6 is never cleared, so the thread stays idle.
2. Flag 4 set and flag 5 clear: the finale. Flag 5 is never set in this pack, so flag 4 alone starts the finale.
3. Flag 2 set and flag 3 clear: the timeout ending.
4. Flag 0 set and flag 1 clear: the entry sequence.
5. Level state 11 set: the director, if rules 1 to 4 do not take the tick (see Director).
6. Level state 11 clear, flag 9 set and flag 10 clear: the fight start step (see below).

Fight start step (rule 6, runs once). It is a short cinematic. It turns letterboxing on, plays camera 4 (a fixed shot with no keys, so there is nothing to wait for) and fades in over 0.5 s without waiting. It starts effects 3 and 4, waits 0.5 s (30 ticks), and moves the selected hero to locator 1 (the hall centre; locator 1, not hero 1). It turns letterboxing off over 0.7 s without waiting, waits 1 s (60 ticks), and returns to the normal camera, which frees the hero: control comes back about 90 ticks after the step starts. The step also sets boss-fight instance 13 to stage 0, which revives the inactive hero if it is down and refills the health and second meter of both heroes (so both heroes enter the fight full, once per load). Flag 10 is set after the cinematic, and rule 6 no longer applies once it is set.

Entry sequence (runs once). Entering trigger index 0 sets flag 0 and does nothing else. On the next tick, rule 4 runs the entry sequence:
1. Fade to black and wait for it.
2. Play movie 78 and wait for it to finish.
3. Fade to black again with a zero-length fade. Move the hero and set the normal camera.
4. Move entity 0 to locator 2.
5. Reactivate objects 0, 1 and 2, then run a zero-length colour fade.
6. Switch to hero 0. While the swap runs, start effects 18 to 25 and wait for the swap to finish.
7. Fade in, return control to the player, and set the hero's equipped state.
8. Set the fight clock to 180.0 s with a floor of 0.0, make it visible and counting down, and start it. Set hero swap mode 3. The clock starts after movie 78, so the movie does not count.
9. Set game state 364 on.
10. Set level state 11 on.
11. Set level state 10 off.
12. Set flag 1 on. This is the last step, so the sequence runs only once. Flag 0 is never cleared.

Director (rule 5). Rule 5 takes a tick only when rules 1 to 4 do not. On a director tick, level state 11 is set, flag 6 is clear, flag 4 is clear (flag 5 is never set), flag 2 is clear or flag 3 is set, and the tick is not the entry tick (flag 0 set, flag 1 clear). The entry tick is therefore never a director tick. On each director tick:

(a) Landed window (level state 16 set).
- On the first director tick where level state 16 is set and flag 8 is clear, set the window counter to 15.0 times the run-time frame rate, and set flag 8 on. At 60 frames per second the counter is 900, which is 15 s. No other step runs on that tick.
- On each later tick, subtract 1 from the counter. When the counter is below 1, which happens N ticks after the arm tick (N is the value set on arm; 900 at 60 frames per second), clear flag 8, clear level state 16, and set level state 13. The fight clock is not paused during the window.

(b) Otherwise (level state 16 clear), in this order:
- Timeout arm: if flag 7 is clear and the fight clock reads exactly 0.0, set flag 2 on.
- Swap completion: if flag 7 is set and no hero swap is running:
  - If level state 12 is set: activate entities 7, 6 and 5, clear level state 12, and set level state 14 on.
  - Otherwise, if level state 13 is set: clear level state 13 and set level state 15 on.
  - Clear flag 7 and restart the fight clock.
- Swap start (after the steps above, in the same tick):
  - If level state 12 is set: when hero 0 is active and no swap is running, stop the fight clock, set flag 7 on, and switch to hero 1 with swap modes 2 and then 3. This is the stun swap.
  - Otherwise, if level state 13 is set: when hero 1 is active and no swap is running, stop the fight clock, set flag 7 on, and switch to hero 0. This is the landed-miss swap.

The fight clock is stopped at the start of each swap the director starts and restarted when that swap completes. It is frozen for the whole swap. Pack 1432 registers no hero-death callback, so no death in this world changes the fight clock (High). The clock is changed only by the entry sequence (start at 180.0 s), the director's swap stops and restarts, and the finale and the timeout ending, which hide it.

Swap length (Medium; the swap task is SPEC-0053 section 3.4). The director's switches pass both flags set, so each swap has the sound, the effects and the white flash. Let W be the number of ticks the outgoing hero spends in its exit one-shot (cue 0xB1, state 0x33). The task waits for that one-shot, then at least 3 more ticks and until the 37-tick fade to white has reached its target; the bind, placement and selection run in one pass; the wait for the outgoing one-shot (step 12) ends at once, because that hero is no longer in state 0x33; then the hold of 41 ticks. So the swap lasts about max(W + 3, 37) + 41 ticks, plus a tick or two for the task start and for the director to see the end: about 80 ticks (1.3 s) when W is 34 or less. The sound settle of step 5 freezes the whole game pass, so it adds real time but no ticks. Because the clock is stopped for the whole swap, the swap length never takes time off the 180 s; it only changes the real time the fight lasts.

Clock and window in ticks (High). The clock counts only on ticks when it is both running and visible. Each such tick takes the frame step (1/60 in single precision, 0.016666668) off the value and then floors it at 0.0. From 180.0 the value after 10,799 counting ticks is about 0.0165, and the 10,800th counting tick takes it to exactly 0.0. It starts on the entry tick (activate at 180.0 with floor 0.0, counting down, then start), stops on the tick a director swap starts, restarts on the tick that swap completes, and is hidden (and so frozen) by the finale and by the timeout ending. The landed window is open from its arm tick T to tick T+899 (flag 8 set on those 900 ticks) and closes on tick T+900; the 900 is 15.0 times the frame rate that the script reads at the arm tick, truncated to a whole number.

#### Trigger 1 and the finale gate

Flag 4 is set only by trigger index 1, only while the landed window is open (flag 8 is set) and only when the trigger's callback block has a non-null object. Entering trigger 1 otherwise does nothing and does not latch.

The trigger-1 cylinder fires on two paths each tick that the hero overlaps it:
- The hero shot pass: a live hero shot inside a kind-3 volume with bit 3 set fires its trigger with object 0 (SPEC-0028, "Trigger pass"). That fire cannot pass the gate.
- The hero's contact walk over the volume list of its grid cell: for kind 3 with bits 1 and 2 clear and bit 3 set, it runs the overlap test against the hero's body cylinder, and on a hit calls the hero class's trigger-hit method. That method fires trigger 1 with object 1, with record-lookup argument 0 (a plain lookup, no remap). The contact walk runs every simulated tick for the selected hero. Its contact mode is fixed by the character's class: heroes always use the mode that walks the cell's volume list (monsters use a mode that does not). No hero state, script call, swap or camera skips it.

Fire step rules. A fire with object 1 always overwrites the callback block (object, actor and the rest) and starts the callback whenever none is running; the world pass frees a finished callback, so the callback reruns about every tick or two while the hero stays inside. A fire with object 0 writes the block only when the trigger's state is 0, so a shot fire cannot reset the object to 0 after a hero hit. The trigger-1 callback sets flag 4 when the block's object is non-null and flag 8 (the landed window) is set. On the next control tick the finale runs and movie 85 plays.

So the finale runs whenever the hero stands in or walks into the cylinder during the 15 s landed window. A hero already inside when the window opens also sets flag 4, because the callback keeps rerunning. The boss does not need to be inside the cylinder; her landing spot is not tested. From locator 3 (-10.79, -325.42), where the entry sequence places the hero, the hit range starts about 32.5 units away, about 2.4 s at the walk speed of 13.51 units per second (SPEC-0033). Both endings are reachable in play: the finale (movie 85) and the timeout ending (movie 80). Pack 1432 never blocks player control after the entry sequence (both of its hero moves leave the hero free, and the entry ends by restoring control), and no swap starts while the window is open, so the player can walk the selected hero into the cylinder. A hero in a blocked state that stands inside also fires it, because the contact walk does not test the hero's state. Either hero can fire it (both use the hero class). Confidence: High (no play check).

The finale gate has two parts:
- The object part. The callback object is 1 (non-null) only when the hero's own trigger-hit method fires the trigger. Every other fire path passes 0: the hero shot pass, the group pass, the monster trigger-hit method and the force-trigger host call. The Darkest Faerie cannot pass 1. Confidence: High for these paths; Medium for which method table her class uses.
- The volume part: the trigger-1 cylinder (see Triggers). High.

Trigger 0 fires from the static rectangle described under Triggers, not from the cylinders (High for the record values).

Finale. On the next fight-control tick after flag 4 is set, rule 2 runs the finale (see Finale above). It hides the fight clock first. It plays movie 85 and waits for it to finish, and sets flag 6 after the movie ends, which leaves the thread idle.

Timeout ending. When flag 2 is set and flag 3 is clear (flag 6 is covered by rule 1: while it is set, the thread is idle), rule 3 runs the timeout ending on the next tick. It hides the fight clock (its value is kept), fades to black, deactivates entities 0, 5, 6 and 7 and objects 0, 1 and 2, sets the hero swap mode to 2, plays movie 80 and waits for it to finish, sets game-over with value 1, sets flags 3 and 6 on, and then stops for good. The game-over behavior is in SPEC-0055, section 4. This is the timeout failure of the final fight. It is a normal outcome of play, not a death.

Mutual exclusion. The finale and the timeout ending both set flag 6, after which rule 1 keeps the thread idle. The two endings cannot both run in one fight. Whichever starts first ends the fight.

Control thread B (High). A second control thread of pack 1432 runs beside the fight control thread. It brings the three Minion Grunts of the final-fight world into the fight and brings each one back after it dies. Entity ids are the positions in the world's character list: entity 0 is the Darkest Faerie, entities 1 to 4 are four summon-reserved Minion Warriors (key 2864), and entities 5, 6 and 7 are Minion Grunts (keys 2842, 2844 and 2841). The thread never tests the Faerie. Medium-High for the list positions.
- Its table has three entries, in the order entity 7, entity 6, entity 5, each with a marker that starts clear.
- Once per tick it walks the three entries in order. An entry goes ahead only when level state 11 is set, flag 6 is clear, level state 12 is clear and hero 1 (Roberta) is the active hero. When the entry's marker is set, its entity must also be dead (IsAI_Dead of that same entity).
- An entry that goes ahead waits 8.0 s. During the wait the thread does nothing else. It then checks again that level state 12 is clear and that the active hero is still the same. If so, it moves the entity to locator 4 (MoveEntity with flag 0, control 1 and fifth argument 1), activates it, and sets the marker to the active hero's index (1). Otherwise it goes on to the next entry.
- After the third entry it sleeps one tick and starts again. It writes no level states, flags or timers, and it never runs the finale or the ending.
- So while Roberta fights before the third stun, the first Grunt arrives 8 s after the rules first pass, the next after 8 s more, and so on. A dead Grunt returns 8 s after the thread reaches its entry. The third stun sets level 12, which stops the thread. The director then activates entities 7, 6 and 5 once more (see above). The Grunts carry no trigger volume, so their moves do not touch the finale gate.

Typical chain (for reading the rules above):
1. Entry starts the clock at 180.0 s and sets level 11.
2. Three reflected orbs stun the boss. The third stun sets level 12, and the director starts the stun swap to hero 1.
3. When the stun swap completes, level 14 is set and the boss's burst begins.
4. After three absorbs the boss falls and sets level 16. The landed window opens.
5. Either the hero reaches the trigger-1 cylinder (within about 7 units of (28.70, -326.01)) inside the 15 s landed window, trigger 1 fires with a non-null object, and the finale runs, or the window closes with no trigger. In the second case, if hero 1 is active, the landed-miss swap runs, and level 15 is set on completion.
6. If the clock reads 0.0 when no swap is pending and level 16 is clear, the timeout ending runs instead.

Confidence: High for the order of the rules, the clock start and stop, the window counter, and the calls in the endings. Medium for the swap conditions and the swap modes. High for the record values of the trigger 0 rectangle and the trigger-1 cylinder; High (no play check) that the finale is reachable in play.

### Earlier encounter (script pack 1174)

- At controller start, region 1175 (variant 1, row 129) loads. Level states 0 and 18 are set at init, so loop A runs. Boss-fight instance 3 is set to stage 0 at setup when no world or character load is active.
- The trigger (trigger index 1; it runs only while script-local flag 15 of the pack is clear, Medium for that flag's setter): fades out over 0.3 s; sets game states 219 and 220; plays movie 16; sets instance 3 to stage 1 (with the hero tweak call); starts act 2. It then yields one tick at a time until the act start is done, plays movie 89, sets game state 221, sets the return world to 151 and plays movie 18.
- Movie indices (High; positions in the boot program's 92-entry movie name table): 16 is ACT1 115A1 (index 17 repeats the same file), 89 is the Act 2 intro (ACTINTRO ACT2), 18 is ACT2 200A. In the final fight, 78 is ACT4 416A2 (fight entry), 80 is ACT4 417A2 (timeout ending; indices 81 and 82 repeat it) and 85 is ACT4 418A (finale). Each play call passes its index unchanged, and the movie is queued even if it has played before. The three final-fight play calls sit on the fight control thread's rules 4, 3 and 2, so each needs flag 6 clear; movie 85 needs flag 4 set and flag 5 clear, movie 80 needs flag 2 set and flag 3 clear with the finale rule not taking the tick, and movie 78 needs flag 0 set and flag 1 clear with neither ending rule taking the tick.
- The pack has no hero-death callback and no death check for the Faerie. The lose path is open.

Confidence: High for the calls; Low for the trigger.

## Edge cases and error handling

- A stun does not check height. A reflected orb can stun her while she is on the ground.
- A hold that ends at count 0 does nothing. Missile slots continue to be scanned during the hold; each overlap consumes the missile, increments the absorb count and restarts the hold.
- A hit at exactly 1.5 units is not a hit this tick (arrival is strict: under 1.5).
- A blocked orb is reflected only when the hero faces it; a hero who does not face the orb takes the hit even if the kinds match.
- Kind 7 blocks any orb kind.
- Loop B's hero knockback starts only when the hero is under 15 units; at exactly 15.0 it does not.
- The earlier-encounter pack has no death handling; the final fight has none in its script pack either.
- Script pack 1098 registers a hero-death callback that restarts its own pack and posts blackboard message 2 with value 120. That pack loads the variant-0 record while game state 30 is set and 31 is clear. Whether that placement is a fight is open.
- The fight clock is not paused by the landed window. It is paused only while a swap that the director started is running.
- A clock at exactly 0.0 arms the timeout only when flag 7 is clear and level 16 is clear. During a swap, the clock is frozen, so it cannot reach 0.0 until the swap completes.
- Trigger index 1 entered outside the landed window, or with a null object field, does nothing and does not latch. Trigger index 0 entered after the entry sequence has run does nothing.
- The hero shot pass's object-0 fire of trigger 1 never sets flag 4; only the hero's contact path (the hero's own trigger-hit method, object 1) does. A rebuild must keep both paths and the fire-step write rule (object 1 always overwrites; object 0 writes only when the state is 0), or the gate fails.
- The trigger-1 cylinder needs only the hero inside it, not the boss: her landing spot is not tested.
- A hero standing inside the cylinder when the landed window opens, even one whose control is blocked, sets flag 4 within about two ticks.
- Walking onto any of the 53 cylinders does not start the fight entry; only crossing the static rectangle does (High).
- Level 16 set while the clock reads 0.0 does not arm the timeout. The arm happens only after the window closes and level 16 is clear.
- The finale and the timeout ending cannot both run in one fight (see Fight flow).

## Test vectors

Synthetic checks (values chosen for the test, not taken from the disc):
- Movie map (disc values): play index 85 opens the file ACT4 418A; index 80 opens ACT4 417A2, and so do 81 and 82; index 78 opens ACT4 416A2; 16 and 17 both open ACT1 115A1; 89 opens ACTINTRO ACT2; 18 opens ACT2 200A.
- Final-fight movie gates: flags 0, 2 and 4 set, flags 1, 3, 5 and 6 clear: only movie 85 is played that tick (rule 2). Flags 0 and 2 set, 4 clear: movie 80 (rule 3). Flag 0 set only: movie 78 (rule 4). Flag 6 set: no movie.
- Game state 364 set and level 18 set: the main controller runs.
- Game state 364 clear and level 18 set: loop A runs.
- Both clear: loop B runs.
- Cooldown draw: the smallest value is 60 ticks and the largest is 240 ticks; both are possible.
- Hero at horizontal distance 9.9 with the boss free: lunge (code 2). Distance 10.0: the far branch applies.
- Orb at 1.49 units (3D) from the hero, hero facing it (dot 0.6), orb kind 3, hero slot-1 kind 3: reflected. At 1.5 exactly: no hit this tick.
- Orb of kind 5 with hero slot-1 kind 7, hero facing: reflected.
- Reflected orb at 1.49 units from the boss: code 5 with a 180-tick timer.
- Orb not blocked, hero with 100 HP and armour: HP 90 (the orb hit is not reduced).
- Lunge with the hero touching her for the whole of animation 9: no damage.
- Repeated shove, hero staying under 10 units away, animation 9 reporting its gate on the 152nd tick after the request (K = 152). Tick 0: chooser picks code 2, handler requests animation 9, no push. Tick 1: push applied, collidable word cleared. Ticks 2 to 151: collidable word cleared, no push. Tick 152: gate passes (progress 0.994), code 0, animation 6 requested. Tick 153: chooser picks code 2 again and animation 9 is requested (tick 0 of the next cycle), no push. Tick 154: push applied. The period is 153 ticks.
- Shove, hero at horizontal offset (3, 0, 4) from her: d = (0.6, 0, 0.8), hero velocity (3.6, 8.0, 4.8), hero action state 0x30. The next integrator tick moves the hero by (0.12, 0.267, 0.16) before gravity.
- Shove push-out: radii 1.0 (hers) and 0.5 (hero), hero at horizontal distance 1.0: shortfall 0.6, so the hero is first moved 3.0 units along d (to distance 4.0), then given the same velocity as above.
- Chooser, hero at distance 5 while code 3 runs and the hold flag is clear: the code becomes 2 on that tick (the orb volley is cut off, and an orb already released keeps flying), and code 0 follows at tick 152 of animation 9. Same with code 5 or code 6: no change. In the absorb window (code 7), the fall and the landing the hold flag is set (it is set by the third stun and cleared by the level-15 reset), so the hero at distance 5 changes nothing.
- Chooser, hero at distance 20, code 0, cooldown 0, hero blocking: code 4 (aimed bolt), whatever the draw. Not blocking, draw 0 of 0 to 10: code 4; draw 1 to 10: code 3.
- Chooser with the hold flag set (after the third stun count): no code changes and the cooldown does not count down.
- Aimed bolt reaching 1.9 units (3D) from the hero: the slot clears and one summon by key 2864 is made 4 units from the hero toward her (not at the bolt's position), facing the hero; the hero takes no damage from the bolt.
- Guard timing: an orb whose pre-step separation is 1.4 on tick T, a hero in the shield state since tick T-1 facing it (dot 0.6) with a matching Mote: reflected. The same hero whose shield state begins on tick T: hit for 10, because she updates first.
- Pick with the hero guarding (draw 4): code 4, no draw for a kind, no glow. Pick with the hero not guarding (draw 4): code 3; if the hero is guarding on tick 24, a bolt opens instead and the glow stops.
- Orb flight (hero standing still): s0 = 10 gives 13 steps, 15 gives 20, 20 gives 26, 30 gives 40, 40 gives 53. Bolt: s0 = 10 gives 12 steps (floor(8 / 0.75) + 2).
- Absorb decay: the second absorb on tick t (count 2, hold 180). No more absorbs: tick t+180 count 1 (hold 180 again), tick t+360 count 0. A third absorb on tick t+179: count 3, code 9 next. A third absorb on tick t+180: count 2 again (drop, then add).
- Window timeout: the burst on tick B sets timer 3000; the code-7 handler first runs on B+1 and the 3000th run (tick B+3000) takes the timer to 0: code 8, nothing else that tick, an absorb on that tick is lost; one on B+2999 counts.
- Hover: count 0, she is at the floor (ground 270.63) on the first window tick: height 270.68 after the first step; 300 steps to 285.63. Count 1 with her at 285.63: target 280.63, 100 steps down.
- Stun with a flinch playing: a sub-state 2 swing reaches her on stun tick 10 (flinch 13 starts), a sub-state 4 swing reaches her on tick 20: code 6 on tick 20; the count completes on tick 47, when the flinch clip passes its gate (10 + 37), not on tick 57; the timer is not changed by either.
- Stun with the timer spent: the timer reaches 0 on tick 179 while a flinch started on tick 160 is still playing: she stays in code 5 until tick 197, then code 0; a form-2 hit on tick 190 is still accepted.
- Roberta lock: Roberta at the floor, she at hover 15, horizontal distance 13.0: 3D distance 19.85, alignment 0.655: locked. Horizontal 13.3: 3D 20.05, not locked. Horizontal 3.8: alignment 0.246, not engaged: the shot goes level.
- Fall from 5.0 units above the ground at 0.1 per tick: 50 ticks to land.
- Timeout descent from 0.5 units at 0.05 per tick: 10 ticks.
- Absorb count from 0 to 1: target height moves from 15 to 10 above the probe; the hold is 180 ticks. When that hold ends, the count returns to 0.
- Absorb overlap geometry, body base (0,0,0), up (0,1,0), height 4, body radius 1, missile radius 0.5: missile center (1.49,2,0) overlaps; (1.50,2,0) misses at side tangency. At the upper rounded end, center (0,5.50,0) overlaps at tangency; (0,5.51,0) misses.
- Absorb count 3: code 9 (fall).
- Loop B knockback: hero at 14.9 units is knocked back; at 15.0 the hero is not.
- Three completed stun animations (across the stun entries): level 12 set.
- Five hero hits of any kind, inside or outside a stun: hit points stay 5. Her take-hit step does not run.
- Stun (code 5, timer 180) and Tor in attack sub-state 4 whose weapon reaches her on stun tick 30: code 6 on that tick.
- Stun and Tor in attack sub-state 2 whose weapon reaches her: animation 13 once, still code 5, then animation 14. With no form-2 hit the timer runs out on tick 180 and she returns to code 0.
- Control thread B, rules passing from tick T with all markers clear: entity 7 is moved to locator 4 and activated at T+480, entity 6 at T+960 and entity 5 at T+1440 (8 s waits at 60 ticks per second). After that an entry goes ahead only when its Grunt is dead.

Timeline A (first 600 ticks of the attack loop; the hero stands still 20.0 horizontal units from her, aim-point separation s0 = 20.0 at each release, not guarding, shield Mote none; the first controller tick is tick 1, code 0, hold flag clear; values marked R depend on random draws):
- Tick 1 to 99: code 0, animation 6 requested each tick, facing blend 0.25. The cooldown starts at 100 (R: a whole number 60 to 240) and reaches 0 on tick 100.
- Tick 100: chooser picks. Draw 1 to 10 (R, here 5): code 3 with kind 2 (R, uniform 1 to 6). Animation 7 requested, glow effect 1619 starts. Cue 2 of block 388 plays (first tick of the clip).
- Tick 124: key 6 reached (tick 24 of the state): glow stops, orb of kind 2 opens at her mouth (the hero is not guarding).
- Tick 125 to 150: orb moves 0.75 per tick. Tick 150 is its 26th step: separation before the step is 20 - 0.75 x 25 = 1.25, under 1.5. The hero takes 10 damage, once, and the slot clears.
- Tick 165: gate of animation 7 (tick 65 of the state): code 0, animation 6 requested, cooldown 120 (R, 60 to 240).
- Tick 285 (165 + 120): cooldown reaches 0. Draw 0 (R, 1 in 11): code 4. Animation 8 requested.
- Tick 317: key 8 reached (tick 32 of the state): aimed bolt opens at her mouth.
- Tick 343: 26th step: separation before the step 20 - 0.75 x 25 = 1.25, under 2.0. The slot clears and one Minion Warrior (key 2864) is summoned at the hero's position plus 4 units toward her, facing the hero. No damage to the hero from the bolt.
- Tick 378: gate of animation 8 (tick 93): code 0, cooldown 90 (R). Next pick at tick 468.
- Tick 468: second orb pick (R: kind 5). Release at tick 492, hit at tick 518, gate at tick 533, cooldown 60 (R), next pick at tick 593. At tick 600 the third pick is in its tick-7 cast: animation 7 at key 1.75.
- Spacing bounds with the hero far away: after an orb pick the next pick comes 65 + c ticks later (c from 60 to 240), after a bolt pick 93 + c ticks later; the first pick is on tick c0 with c0 from 60 to 240.
- If the hero were guarding at tick 124 the same tick would open an aimed bolt instead of the orb (and no damage on arrival); the slot flight and summon follow the bolt timings (26th step).

Timeline B (stun sequence; hero Tor standing next to her, not guarding; a reflected orb reaches her on tick R0, any code):
- Tick R0: code 5, timer 180, flinch latch clear. Handler this tick: timer 179, animation 14 requested.
- Tick R0 + 30: Tor in attack sub-state 4 with the weapon reaching her: code 6 (timer not tested).
- Tick R0 + 31: code 6 handler requests animation 13. Tick R0 + 68 (37 ticks later): gate; stun count 1; code 0.
- Same sequence with a sub-state 2 swing on tick R0 + 30: code 5 stays, animation 13 requested once; the flinch gate comes 37 ticks later, then animation 14 resumes; the form-2 swing is still available while the timer runs.
- No form-2 hit: the timer reaches 0 on the handler run of tick R0 + 179, and code 0 starts on tick R0 + 180.
- Third completed stun on tick S: levels 12 and 10 set; tick S + 1 the gate sets code 1; the director's stun swap runs (about 80 ticks), then level 14: on that tick code 1 clears 14 and 10, sets code 7 with timer 3000; the code-7 handler runs from the next tick and moves to code 8 on the 3000th run.
- Hold flag: from tick S on, the chooser makes no pick (no shove at distance 5, no attack, no cooldown count) until a level-15 reset.

Timeline C (pack 1428, loop B, after the boss-room return; Queen Fyora at locator 1, (0.81, 8.37, -303.4), she at locator 2, (0.02, 8.37, -359.43), hero at locator 3 or anywhere beyond 15 of her, state 0, cooldown 0 at tick 1):
- Tick 1: chooser: state 0, no type-0 orb, cooldown under 1: summon test (R: 1 in 4 to pass, and none of her summons alive): state 3 otherwise. Animation 7 requested.
- Tick 25: orb opens at her mouth (type 0), moving 0.5 per tick toward Queen Fyora's aim point (horizontal separation 56.04, so Queen Fyora's deflect test passes after about 110 steps, 110 ticks after the release; Medium, because the deflect distance is not fixed here and the aim points sit above the position points); tick 66: state 0, cooldown 30 to 45 (the next cast waits until the orb is gone).
- A hero at 14.9 horizontal units on any tick: state 4 on that tick; the next tick animation 9 is current and the hero is thrown with velocity (6 d, 8.0); at 15.0 nothing.

Sounds and effects (disc values):
- Orb volley of kind 3: on the first tick of animation 7, cue 2 of block 388 plays once at her and glow effect 1622 starts on her (it resolves to definition 330 of resource 9). At the release the glow stops and an orb with flight effect 1608 is launched. When the orb's slot clears, the flight effect is drained and impact effect 988 plays at its last position.
- Aimed bolt (animation 8): no sound and no glow; flight effect 2127, impact effect 412.
- Her main animation changes to 13: block 388 cue 1 once. A second request of 13 while it plays: nothing.
- Orb hit amount: the word 0x2DA634 reads 10, and no effect definition 10 exists, so the 10 is the orb's damage.
- Minion Warrior animation 58 (die): block 377 cue 7 once, and effect 1623 at 0.9 of the clip.

Fight flow (synthetic; durations are chosen for the test, except where a fixed value is given):
- Hero segment from (40.0, 280.0, -330.0) to (32.0, 280.0, -330.0) crosses the plane through (36.40, 269.20, -343.99) with normal (-1, 0, -0.0024), inside the rectangle (height offset 10.8 of 20.85, width offset about 14.0 of 37.25): trigger index 0 fires with object 1 (the hero's own body contact) and flag 0 is set. The same segment at y = 295.0 (height offset 25.8, above 20.85) does not fire.
- Hero standing on a cylinder volume (flags 0x0001): no trigger fires; flag 0 stays clear.
- Trigger index 0 entered on tick 0. The entry sequence runs on tick 1 and sets the clock to 180.0 with level 11 on. After the first counting tick the clock reads 179.9833 (one step of 1/60 s).
- Timeout, nothing else happening: the clock reaches 0.0 after 10,800 counting ticks (180 s). Flag 2 is set one tick later, and the timeout ending runs one tick after that. It plays movie 80 and waits for it to finish. Game over (value 1) is set, and flags 3 and 6 are set, after the movie ends.
- Landed window: level 16 set on director tick T with flag 8 clear. The counter becomes 900 (15 s at 60 frames per second) and flag 8 is set. Trigger index 1 entered on tick T+180 with a non-null object field: flag 4 is set. The finale starts on the next thread tick, plays movie 85 and waits for it to finish; flag 6 is set after the movie ends.
- Same as above with a null object field: flag 4 stays clear, nothing else changes, and the window still closes on tick T+900.
- Window closes on tick T+900 with no trigger 1: level 16 clear, level 13 set, flag 8 clear.
- Trigger index 1 entered on tick T+1000 with a non-null object field: nothing happens. Flag 4 stays clear.
- Trigger-1 cylinder, landed window open (flag 8 set), hero standing at (28.70, 270.63, -326.00): within 2 ticks the callback sees object 1 and sets flag 4; the next control tick plays movie 85, and flag 6 is set after the movie ends.
- Same, hero at (20.0, 270.63, -326.0) (8.7 units from the axis): no hit; flag 4 stays clear and nothing happens.
- Hero inside the trigger-1 cylinder with flag 8 clear: nothing. When flag 8 becomes set while the hero is still inside, flag 4 is set within 2 ticks.
- Clock at 0.0 during the window (level 16 set): no flag 2. The window closes on tick T+900; flag 2 is set on tick T+901 and the timeout ending runs on tick T+902.
- Stun swap: level 12 set while hero 0 is active and flag 7 is clear. On that tick the clock stops, flag 7 is set, and the switch to hero 1 starts. The clock stays frozen for the whole swap (about 80 ticks; see the swap length rule below). On completion: level 12 clear, level 14 set, entities 5, 6 and 7 activated, flag 7 clear, and the clock restarts from its frozen value.
- Landed-miss swap: window closes with hero 1 active. The next director tick starts a swap to hero 0 and stops the clock. On completion: level 13 clear, level 15 set, clock restarted.
- Window closes with hero 0 active: no landed-miss swap starts, and level 13 stays set.
- Finale, then the thread: after the finale sets flag 6, no later timeout ending runs, even if the clock reaches 0.0 later.
- Init registrations (synthetic): trigger 1 fires with a non-null object while flag 8 is clear: nothing happens; the same call with flag 8 set sets flag 4. Trigger 0 firing sets flag 0 whatever its object. A world load with the three Minion Grunts in the character list leaves entities 5, 6 and 7 unbound by streaming; the first one appears 8 s after control thread B first passes its checks (Control thread B).

## Open questions

Default if unresolved (rewrite choice): start the main controller in code 1 with the cooldown counting down, so that the first approach within 10 units gives the shove and starts the attacks (Initial values). Default if unresolved (rewrite choice): loop B starts in state 0 with cooldown 0 and all flags clear. Default if unresolved (rewrite choice): the event bit of a clip is read from the cursor's whole part at the start of the controller tick, giving orb release at tick 24 and bolt release at tick 32; a port that reads it one tick later changes every number by one tick only. Default if unresolved (rewrite choice): her mouth point is her model's mouth attachment (the one the cast glow follows), and the hero's and her aim points are attachment point 7 of each model (SPEC-0004); their heights above the position points are model data not extracted here, and a different choice changes an orb's flight by a tick at most. Default if unresolved (rewrite choice): Roberta's shot cost is paid on a launch that finds a free slot, and a shot needs at least 1 MP; the hero's lock-on candidate list of range 20 is the list a quick shot reads.

38. Resolved (Medium-High): the player-facing route is complete and every step is reachable with the stated prerequisites: shield and a shield Mote (the Nova Mote from a Warrior if none is held), a form-2 swing inside the stun, Roberta's lock-on or a low target for three absorbs, and the walk into the finale volume (see What the player must do). Not checked in play.
39. Resolved: the stun hit needs the weapon to overlap one of five small hurt capsules of her skeleton; her skeleton has them (High for the data).
40. Resolved: the hover hold re-arms at each drop, the 3000 timer tick does nothing else, and extra overlaps on one tick all count (High).
41. Resolved: the window timeout opens no landed window and the finale cannot follow it; level 15 resets her through the director's swap back to Tor (High).
42. Resolved: hero death is game over in this fight (swap mode 3; SPEC-0055) (High).
43. Resolved (Medium-High): her four-slot orbs and bolts do not raise the hero's automatic guard timer, because that threat check scans the enemy pool and attacking characters, not her projectile slots. Nearby Warriors or Grunts can raise it when they attack in front of the hero. The hero must hold the guard button to defend against her projectiles (Fight AI build guide, step 3).
44. Resolved (Medium-High): the entry plane x = 36.40 fires when the hero's centre is within 1.175 of it (x 35.225 or more, with the width and height windows of the Entry plane paragraph), so pressing into the east barrier (solid face at x 36.81, centre stop about 35.64) starts the fight without crossing the plane; the trigger is also fired by a hero shot that reaches the plane. Test vector: hero at x 35.30, z -325.7, y 270.63: fires; hero at x 35.10: does not; hero at x 35.64 (stopped by the barrier): fires. Default if unresolved (rewrite choice): fire trigger index 0 when the hero's centre is within 1.175 of the plane inside its rectangle.
45. Resolved: how the hero gets from the arrival record to the hall. The pack's fight-start step moves the hero from entry record 0 (alcove) to entry record 1 (hall centre) at tick 30 of the step, with control at tick 90; the step runs once per load whenever the pack's level state 11 is clear and flag 9 (set by the pack's activation) is set. A load that places the hero at entry record 0 and runs the pack's activation (the normal arrival and a developer start alike) gets the same move; nothing else (door, trigger, movable object) is involved (High).

34. Resolved (Medium-High): the embedded Faerie record is the only record the streaming fill can bind, and it binds in the first region pass of the world, which runs (with her first think) before that tick's script step. Her first controller pass is therefore in code 0, and the activate-world callback's level state 10 forces code 1 from the second pass (see Initial values). Remaining doubt is only whether the load-completion step runs an extra region pass or script step before the first full tick; none is listed in the main loop. The default rewrite choice differs only by the one-pass quarter turn.
35. Resolved: the burst's four requests (key 2844) find no summon-reserved record in the final-fight pack, because its three Grunt records carry key 2844, 2842 and 2841 but a summon byte of 0. They spawn nothing; the Grunts come from the script (control thread B and the director). The aimed bolt's key 2864 does find the four reserved Warriors (High for the records and the search rule).
36. Resolved: in pack 1428 the Faerie's loop B knockback repeats on every tick of animation 9 while the hero is under 15 units (High).
37. Resolved: the orb is blocked only when the hero's guard predicate is true; the element is read from the Mote in equipment slot 1 (the shield Mote word), not from an attack id (High). A guarding hero at the release tick makes her fire the aimed bolt instead of the orb (High).

1. Resolved: her hit points stay 5. All three controllers set her invulnerable flag and nothing clears it, so no hero hit reaches her take-hit step (High).
2. Resolved: code 5's gates are the hero's in-attack test, attack form 2 and the hero's weapon reach (see Stun and the reflect weak point; High). The per-tick field cleared by the main controller is used by shared steering to enable the second input axis as vertical movement; it is not a hit gate. The field's wider role in other actors' steering remains outside this spec.
3. Resolved: the absorb test compares each armed hero-shot sphere (slot point and radius) against her vertical body axis from base to base plus up times height. In the central axial band, the coordinate along the axis may reach either cap plane inclusively, while the radial distance must be strictly less than the sum of radii; side tangency misses. Outside that band, the rounded ends use an inclusive point-to-segment distance check, so end tangency counts. The broad-phase distance check is inclusive. The lunge has no contact test (High for the lunge; Medium-High for the absorb geometry because the shared helper includes a vector-unit path).
4. Resolved: the finale trigger (trigger index 1) fires from cylinder record 8 of the final-fight world (High). The record values given under Triggers are the values stored in the world pack's cylinder list (kind 3, mode 0, flags 0x0008, gate mask 0, trigger index 1, base (28.702, 270.608, -326.005), axis (0, 1, 0), length 3.0, radius 6.0), and it is the only one of the list's 53 cylinder records with the trigger flag 0x0008. Its object field is non-null only on the hero's own trigger-hit path, which the hero's contact walk reaches (items 17, 18 and 32). Test vector: the hero standing on the arena floor with its axis 6.5 units from the record's axis overlaps it (6.0 plus the hero radius 1.0 reaches 7.0) and fires trigger index 1; at 7.5 units it does not. Resolved: control thread B's only script-local flag test is flag 6 (the fight has ended), besides level states 11 and 12 and the active hero (High).
5. Resolved: trigger 1 of script pack 1174 runs only when its module-local flag 15 is clear; its call sequence is in Earlier encounter. Flag 15 has no writer: every call of the module's set routine passes a constant bit number, and 15 occurs only in this one test (the stage-word table of SPEC-0044, module 1174, lists bit 15 as set by nothing). So the gate is always open and trigger 1 always runs when the hero crosses its rectangle; the reason the original author added the test cannot be known (High for the missing writer).
6. Resolved: the final-fight world list identifies entities 1–4 as summon-reserved Minion Warriors with key 2864 and entities 5–7 as Minion Grunts with keys 2842, 2844 and 2841 respectively (see Control thread B; Medium-High for list positions). The summon system chooses an unbound reserved record with the requested key. Attack 1's spawn count 4 is the live-summon cap for that slot; each aimed bolt makes one summon request. Attack 2's spawn count 4 is the number of requests in the burst, matching the four explicit burst requests (SPEC-0029 Behavior 31; High for count meanings). The final-fight pack holds no summon-reserved record with key 2844 (item 35), so the burst requests spawn nothing there.
7. Resolved: the lunge is a shove of the hero, not a move of hers. 12.0 is the vertical lane, 9.0 the horizontal direction from her to the hero, -5.0 the push-out factor on the radius shortfall, and 10.0 the final speed, so the hero's velocity is (6.0 d.x, 8.0, 6.0 d.z). It is a motion-record velocity, applied as velocity x 1/30 per tick (High; see Attacks).
8. Resolved: the near-range shove waits for neither the cooldown nor the ready test, and replaces any code except 2, 5 and 6. Codes 5 and 6 enter the far branch only to fail the ready test, so the chooser never changes them. The far branch also picks the aimed bolt whenever the hero is blocking (High; see Chooser).
9. Resolved: code 5 holds animation 14 and plays animation 13 once as a flinch when an attack of another form reaches her (High).
10. Resolved: level state 10 is the burst request: set when pack 1432 activates and by the controller's codes 6, 8 and 10, it makes the chooser pick code 1, and the entry sequence clears it (Level states used by the controller, Fight flow). Level state 13 is set by code 8 and by the director when the 15-second landed window ends; the director's landed-miss swap consumes it (Director, part (a) and (b)). Level state 14 is set by the director when the stun swap completes, to start the burst and the window, and code 1 clears it (High for the sets and clears; Medium-High for the meanings). Test vector: after the third stun and the swap to Roberta completing, level state 14 is set on the first tick after the swap; the next burst (code 1) clears it. Resolved for game state 358: it is the Ixi Chief's mode-2 flag (SPEC-0014), set only by the init of pack 1432 (High).
11. Resolved: one aimed-bolt detonation makes one summon-by-key request for key 2864. Its spawn count of 4 is the live-summon cap for that slot, not a request to summon all four reserved records at once; the summon system chooses one unbound matching record, if available (SPEC-0029, Behavior 31; High for request count and cap, Medium for which record binds).
12. Resolved: the lunge deals no damage (it has no hit routine), and the detonation deals none itself: it is a summon by key 2864, whose summoned character attacks with its own row values (High for the lunge and the summon). Resolved, the names part: every id of the kind table is an effect id. The first pair is the flight effect and the impact effect of the projectile; the third column (1617 to 1622) is the cast glow held on her during animation 7, not a sound. Her sounds are the block 388 cues of her animation table (see Sounds and effects; High).
13. Resolved: class 38 and row 132 are Queen Fyora (SPEC-0029, Behaviors 7 and 30; High). Loop B code 3 opens a type-0 homing slot from her sample point; code 5 opens a type-2 shot that homes to the aim point returned by the hero method and, on arrival, requests a spawn using the class-37 character row slot-0 key; the sidestep path opens a type-3 deflected slot. These are the shared four-slot behaviors in SPEC-0029, Behavior 19 (High for slot types and arrival action; Medium for the specific sample point and rendered projectile setup).
14. Resolved: the movie table positions are confirmed against the boot program: 16 = ACT1 115A1, 18 = ACT2 200A, 78 = ACT4 416A2, 80 = ACT4 417A2, 85 = ACT4 418A, 89 = ACTINTRO ACT2 (High; Earlier encounter, Movie indices).
15. Resolved in part: pack 1428 is an active scripted trap encounter in Queen Fyora's chambers, with both Queen Fyora and the Darkest Faerie; the script ends the scene by objective/flag progression rather than Faerie hit points (High). Pack 1259 is not reached in the local world-load graph, so its unnamed variant-0 record is not established as an active placement (High for the graph survey; direct or unknown loading is not excluded). Pack 1344's named region has no established load in the local graph (High for the graph survey; direct or unknown loading is not excluded). Pack 1098 conditionally loads its variant-0 region and has a hero-death restart callback, but those facts do not establish whether it is a fight (Medium-High). The roles of 1098, 1344 and 1259 remain open where noted.
16. Resolved: missile slots are scanned during the active absorb hold. Each overlapping active missile is removed, adds one to the absorb count and resets the hold; the hold does not pause scanning (High). Three total absorbs still start the fall.
17. Resolved: the object field is 1 only when the hero's own trigger-hit method fires the trigger; every other fire path passes 0 (High for the paths; Medium for the Faerie's method table). Resolved: on the contact-walk path the hero's trigger-hit method receives record-lookup argument 0 (a plain lookup, no remap), so it fires (High).
18. Resolved in part: trigger index 0 fires from the static rectangle (High for the record values), and trigger index 1 fires from the cylinder described under Triggers (High for the record values). Resolved: entities 5, 6 and 7 are Minion Grunts, which carry no volume, so control thread B's moves do not affect any trigger (High).
27. Play check for the fight entry: a hero walking across the rectangle near (36.4, 269.2, -344.0) in the final-fight world should start the entry sequence (flag 0, then movie 78 and the 180 s clock). If it does not, the trigger 0 path above is incomplete. Default if unresolved (rewrite choice): fire the entry from the rectangle as described (needs a runtime check).
28. Resolved: the rectangle is in the volume lists of the grid cells the selected hero walks, and the hero's own body contact fires trigger 0 with object 1 (SPEC-0028, "Trigger pass"; High).
29. Resolved: the trigger-0 rectangle's reference point is its low corner for the runtime test. The test measures both in-plane coordinates from that point over 0 to the corresponding extent; pack 1432's extents are positive (High for the test and record values). This establishes operational geometry, not an editor or authored label (see Triggers).
30. Resolved statically, with a reachability limit: when trigger 0 sets flag 0 while flag 1 is clear, the next control tick takes the entry rule before the fight-start rule; the entry rule has no flag-9 or level-11 gate. The entry sequence later sets level state 11 and flag 1, so the fight-start rule is suppressed. Thus an early crossing is not harmless if it precedes the first fight-start tick. Whether a clean-route hero can cross early enough to do so depends on route timing (High for rule order and writes). Default if unresolved (rewrite choice): implement the rule order as stated and add no guard (needs a runtime check).
31. Resolved: the trigger-1 volume is in the final-fight world pack itself (cylinder record 8, see Triggers). No outside source is needed (High).
32. Resolved: the hero's contact mode is a class constant (always the full mode) and the walk has no hero-state gate, so it runs during the landed window. The finale is reachable in play (High). A play check is still useful.
19. Resolved: the step is the frame-step word, 1/60 as a single-precision value (0.016666668), written once at start-up (SPEC-0023 section 31; High). The clock reads exactly 0.0 after its 10,800th running tick.
20. Resolved: a director swap lasts about max(W + 3, 37) + 41 ticks, where W is the outgoing hero's exit one-shot length, so about 80 ticks when W is 34 or less; the clock is stopped throughout, so the length never uses clock time (Swap length; Medium for the exact count, High that it uses no clock time).
21. Resolved: the player cannot switch by hand during the fight. The entry sets swap mode 3 and every director swap ends with mode 3, and mode 3 refuses the chord (SPEC-0053 section 3.1), so hero 1 is still active when the window closes and the landed-miss swap can run (High).
22. Resolved: pack 1432's init registers two control threads (the fight control thread and control thread B), trigger callbacks 0 and 1, one activate-world callback and two entity filters (the second replaces the first; Activation), but no hero-death callback. Trigger 0 sets flag 0; trigger 1 sets flag 4 when its object argument is non-null and flag 8 is set; the activate-world callback sets level state 10, activates objects 0, 1 and 2, starts effects 18 to 25 and sets flag 9; the effective filter refuses entities 5 to 7. Its clock start, stop, restart and hide paths are confined to the fight-entry, director and ending control-thread roots. A survey of all 163 distinct known level modules found timer-control calls only in packs 1432, 1394, 758 and 934. The other three are ExteriorDLLInit modules; because the world system has one E buffer and releasing a world tears down its script, none can remain resident alongside pack 1432 in E. Pack 1427 is a TransitionDLLInit module and has no timer-control call. Thus no other known level-pack callback can alter the clock while pack 1432 occupies E (High, conditional on the documented buffer lifecycle). Resolved (High): no engine-owned callback changes the clock. The timer's state (value, limit, direction, running and visible flags, colour) is written only by the six timer host calls and by the timer's own per-pass update and draw; nothing else in the boot program stores to it. The only routines of the boot program that call the timer's hide operation are the world-exit paths of the front end (the quit confirmation on the pause page, the credits, the game-over page's quit hold and the front-end world load). They hide the clock (visibility only; the value and running flag are kept) when the player leaves the world, never during the fight. Test vector: pause during the fight with the clock at 100.0 s: the value holds while the page is up (SPEC-0023, section 31), and resumes counting from 100.0 when play resumes; only a confirmed quit hides it.
23. Resolved: she cannot lose hit points at all, inside or outside her stun, because she is invulnerable for her whole life (High; see Hit points and how they drop).
24. Resolved: all 64 level-state flags start clear: the world constructor allocates a zeroed 9-byte flag block when possible and reset clears it; without the block, QueryLevelState returns 0. Pack 1432's 32-bit script-local flag word is zero in the module payload and no constructor entry changes it before Init, so all its local flag bits start clear. SetLevelState true sets a flag and false clears it (High; see Activation).
25. Whether 180 s is enough for a typical player to reach the finale in play. Settled from the disc: swaps use no clock time (Swap length), and each landed window uses 15 s of clock (900 ticks), so the clock is spent only on the fighting between swaps and on the windows. How long a player takes to stun her and to land the finale is a player-skill measure (needs a runtime check); the rewrite uses the 180 s clock unchanged.
26. Resolved: each entry's death check tests that entry's own entity (7, 6 or 5). All three are Minion Grunts; entity 7 is not the Darkest Faerie, who is entity 0 (High for the test, Medium-High for the list positions).
33. Resolved: not reached in play, because her hit points never fall (High).

