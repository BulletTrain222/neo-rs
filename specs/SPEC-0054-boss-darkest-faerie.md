# SPEC-0054: Boss behavior: the Darkest Faerie

- Answers: proactive (boss behavior); R-0069 (the repeated shove: Chooser and Test vectors)
- Status: Draft
- Disc build: SCUS-97367 v1.00 (same fingerprint as SPEC-0001)
- Confidence: per section. Monster row values and tuning words: High. Routing between the three controllers: High. Main-controller cycle and its timings: High for the transitions, Medium-High for what the level states mean. Projectile speeds and reflect rule: High for the code and for the hit amount (the word 0x2DA634 is the orb damage; no effect definition 10 exists). Animation sounds (block 388, Minion Warrior block 377), the orb flight and impact effects and the cast glow: High; clip effect lists: Medium-High. Lunge and loop B: Medium or Low as marked. Stun gates, stun animations and invulnerability: High. Fight flow (script side): High for the rule order, the entry, the clock, the window counter and the timeout and finale calls; Medium for the swap conditions; High for the trigger 0 volume's record values and its fire path; High that the trigger-1 volume exists (cylinder record 8 of the final-fight world's cylinder list); High that the finale is reachable in play (the full fire path is established; no play check). Earlier-encounter trigger: High for its flag-15-clear gate and call sequence; Low for the flag writer and the reason for the gate. Hit points: High that no hero hit lowers them in any controller, so the death path is not reached in play. Damage paths (raw orb hit, no lunge damage, summons by key for the detonation and the burst): High; which spawn records the keys resolve to: Medium-Low.

## Summary

The Darkest Faerie is a flying boss with a hover height, a set of attacks, and an absorb window in which she takes the hero's missiles. Her behavior is one of three controllers chosen each tick by a progress flag and a level state. The main controller (used in the final fight) cycles through idle, orb and bolt attacks, a burst of four entities, a 50-second absorb window, a fall after three absorbs, a landing, and a new burst. A blocking hero can turn an elemental orb back on her; the reflected orb stuns her for 3 seconds and is her only weak point. She has 5 hit points, stored in her monster row. The earlier arena encounter uses the same class with the simpler loop A. The clean side can build the boss from the row values, the tuning words and the rules below. The script side of the final fight (the entry, the clock, the landed window, the finale and the timeout ending) is in the Fight flow section. The earlier-encounter trigger is described as calls and facts, not as complete logic.

Times are seconds. One second is 60 ticks (SPEC-0007). Ticks given below are exact unless marked. Distances are horizontal unless stated. "Level state n" and "game state n" follow SPEC-0030.

## Where it lives on the disc

- Monster table: resource 530. Rows 128 (variant 0, the final-fight placement) and 129 (variant 1, the earlier-encounter placement). The row layout is the common enemy row layout of SPEC-0029. Rows are found through the per-type first-row index (type 56 maps to row 128, and variant 1 is the next row).
- Tuning values: 32-bit words in the boot program's data, read at runtime by virtual address. The Faerie's words are at 0x2DA618 to 0x2DA64C and at 0x2DAAB0 to 0x2DAAB8. The disc has no names for them; the names in the Layout section are descriptive.
- Animations: "animation n" means entry n of the boss's own animation table (SPEC-0005).
- Placements (script side):
  - Script pack 1432 (final fight): a type-32 world-character record named "Darkest Faerie" with behavior type 56, variant 0. Boss-fight instance 13.
  - Script pack 1174 (earlier encounter): loads region 1175 at controller start (variant 1, row 129). Boss-fight instance 3.
  - Script pack 1098: loads region 1108 (variant 0) only while game state 30 is set and game state 31 is clear. Its hero-death callback restarts that pack and posts a message to its world; whether this placement is a fight remains open.
  - Script pack 1344: region 1349 (variant 0, "The Darkest Faerie"). No load of that region is established in the local world-load graph; a direct or untraced load remains possible.
  - Script pack 1428 (Queen Fyora's chambers, instance 12) is an active scripted trap encounter. It places Queen Fyora and the Darkest Faerie together, then deactivates both and advances the Act 4 sequence at the scripted exit. It is not a normal boss fight that ends through the Faerie's hit points.
  - A block in pack 1259 holds a variant-0 record with no name. Pack 1259 is not reached in the local world-load graph, so an active placement is not established; a direct or untraced load remains possible.
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

The hero's slot-1 attack id maps to an element kind. Attack ids are given as decimal numbers.

| Attack id | Kind |
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
| 1 | burst | 6 until level 14 is set, then 14, then 10 | Waits for level 14. Spawns four burst entities by summon key 2844 (SPEC-0029 Behavior 31) at plus or minus 4 units on two axes, with direct drop item 172 (Nova Mote) and the boss's forward direction. Clears level 14 and level 10. Starts the 50 s absorb window | Code 7 when the window starts. Level 15 set: clears levels 15 and 10, resets the stun count, code 0 |
| 2 | lunge (shove) | 9 | When the hero is near (under 10 horizontal). On the first tick pushes the hero away (velocity 6.0 along the direction to the hero, 8.0 up; knocked state); once per animation. She does not move. It has no hit routine and deals no damage | Code 0 when animation 9 ends |
| 3 | orb volley | 7 | On the first tick of animation 7, starts the cast glow of the chosen kind on her. At the animation's event, fires one elemental orb of that kind (random 1 to 6) from her mouth point and stops the glow. Cooldown 1 to 4 s | Code 0 when animation 7 ends |
| 4 | aimed bolt | 8 | At the animation's event, fires one aimed bolt. Cooldown 1 to 4 s | Code 0 when animation 8 ends |
| 5 | stun | 14 (held), 13 (flinch) | Counts a 3 s timer. If the hit gates pass, goes to code 6 | Code 0 when the timer ends |
| 6 | stun count | 13 | Counts completed stun animations. The third sets level 12 and level 10 | Code 0 after the first two. After the third, the boss stays in this code until the gate sets code 1 on the next tick (see Stun) |
| 7 | absorb window | 20 (hover) | Hovers and absorbs missiles for 50 s. Three absorbs: code 9 | Code 9 after 3 absorbs; code 8 at 50 s |
| 8 | timeout descent | 20 | Descends 0.05 per tick to the ground | At the ground: levels 13 and 10 set; code 1 |
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
- If the distance is under 10 and the current code is not 5 or 6: the choice is code 2 (the shove), unless the current code is already 2. This branch tests neither the cooldown nor the ready test, so it replaces any other code (0, 1, 3, 4 and 7 to 10) on the tick the hero comes near, and it starts again on the tick after a shove ends if the hero is still near.
- Otherwise (distance 10 or more, or the current code is 5 or 6): nothing changes unless the current code is ready and the cooldown is 0. Then code 4 (aimed bolt) is chosen when a draw of 0 to 10 inclusive gives 0 (1 in 11) or when the hero's guard query is true (the hero is blocking); otherwise code 3 (orb volley) with a kind 1 to 6, uniform.
- Each new choice clears the code's first-tick flag.

Repeated shove (R-0069, High). The shove handler has three cases. If animation 9 is not the current animation, it requests animation 9 and does nothing else on that tick, so the push waits one tick. If animation 9 is current and its progress is below the 0.99 gate, it applies the push on the first such tick (guarded by the first-tick flag that every new choice clears) and clears the hero's collidable word on every such tick. If animation 9 is current and its progress has reached the gate, it sets code 0 and requests animation 6 in that same tick. The chooser runs before the handler, and while the current code is 2 it never picks again, so on the tick that sees the animation end the code is still 2 and nothing is re-requested. The request that sets code 0 also makes animation 6 the current animation, so on the next tick the chooser (hero still under 10) picks code 2 with animation 6 current, and the handler requests animation 9 as a real change from 6. The same-id rule of SPEC-0005 therefore never applies here: a request for animation 9 while 9 is current and finished does not occur, and there is no separate force-restart request on this path. Each shove starts animation 9 from its first frame, and one tick of animation 6 sits between shoves. A shove cycle lasts K + 1 ticks, where K is the number of ticks from the request tick to the first tick on which animation 9 reports its 0.99 gate. Animation 9 plays a one-shot clip of 39 keys (no loop), whose speed byte is 2, so at the normal play rate of 1.0 the cursor moves 0.25 key per tick (one key every 4 ticks) and reaches its last key, 38, 152 ticks after the request tick. Her controllers never change the play rate. The gate (progress at least 0.99 of 39 keys, SPEC-0005 Clip progress) first passes on that same tick 152, because progress counts the stored fraction twice: on tick 151 it is (37.75 + 0.5) / 39 = 0.981, on tick 152 it is (38 + 0.75) / 39 = 0.994. So K = 152 ticks (2.53 s), a shove cycle lasts 153 ticks (2.55 s), and a shove runs once per play of animation 9 plus one tick, not every second tick (High; Medium-High that no status effect slows her clip). The push lands on tick 1 of each cycle (the tick after the request), on every shove while the hero stays under 10 units away.

Ready codes are 0, 7 and 8 (and any value below 1 or above 10). Codes 1 to 6, 9 and 10 are not ready. Codes 5 and 6 are sent to the far branch only so that the ready test then fails: the chooser never changes code 5 or 6, so the stun and the stun count cannot be cut short by the shove or by a new attack.

Confidence: High for the gate, the cooldown draw, both branches and the ready test.

### Projectiles (the four slots)

All three projectile kinds move at 0.75 units per tick (45 units per second). Each moves toward its target.

- Elemental orb (slot type 0; kind 1 to 6). Moves toward the hero. Within 1.5 units (3D):
  - If the hero faces the orb (dot product of the hero's forward direction and the direction from the hero to the orb above 0.5) and the hero's slot-1 kind equals the orb kind or is 7, the orb is blocked. It becomes a reflected orb (slot type 2) and is aimed at the boss.
  - Otherwise the orb hits the hero once for 10 (the orb damage, word 0x2DA634; the 10 is not an effect id, since no effect definition 10 exists), and the slot clears. The 10 goes straight to the hero's take-hit with no attacker (attacker id -1); like other boss hits it is not reduced by the hero's shield, armour, armour Mote or petpet factor (SPEC-0029 Behavior 10.5). These four slots belong to the boss, not to the shared enemy projectile pool.
- Aimed bolt (slot type 1; kind 0). Moves toward the hero. The bolt itself deals no damage. Within 2.0 units (3D), the slot clears and a summon is made by the row's key 2864 (row +0x3CC; SPEC-0029 Behavior 31) at its position, with direction from the boss and direct drop item 172 (Nova Mote). Any damage comes from the summoned character. On disc, key 2864 is carried by Minion Warrior spawn records (monster row 204: weapons 1 to 3 radius 3.0, length 2.0; attack damage 7, 7 and 15; class 17), which hit through the shared enemy chain (SPEC-0029 Behavior 10.5). Which record the final world resolves is Medium-Low.
- Reflected orb (slot type 2). Moves toward the boss. Within 1.5 units of the boss, the slot clears and the boss is stunned: code 5 with a 3 s timer. This is her weak point.

The flight and impact effects of each slot are given by the kind table. The slots play no sound: the flight effect is drained when the slot clears, and the impact effect then plays at the slot's last position.

Confidence: High for the speeds and distances; Medium-High for the raw orb hit and the summon by key; Medium-Low for which spawn record the final world resolves for key 2864.

### Attacks, summary

- Elemental orb volley (code 3): one orb of a random kind 1 to 6 at the animation's event. Then a cooldown of 1 to 4 s.
- Aimed bolt (code 4): one bolt at the animation's event. Then a cooldown of 1 to 4 s.
- Lunge (code 2), which is a shove of the hero: she does not move. On the first tick of animation 9, with d the horizontal unit direction from her to the hero and h the horizontal distance: the hero's motion-record velocity is set to zero and 12.0 is added to its vertical (y) lane. If h is less than (her motion radius + the hero's motion radius + 0.1), the hero's position is moved along d by 5 times that shortfall (out of her body). Then 9 x d is added to the velocity, and the velocity is normalized and scaled to 10.0 (the word 0x2DA630). The result is (6.0 d.x, 8.0, 6.0 d.z). The hero is put into its knocked state (hero action state 0x30), the same push as the Ixi Chief's (SPEC-0014, Push). On every tick of animation 9 the hero's collidable word (SPEC-0040) is cleared. The value is a motion-record velocity: each tick the position moves by velocity x 1/30 (SPEC-0040 Behavior 14A), so the shove starts at 0.2 units per tick along d and 0.267 units per tick upward, before gravity. The shove has no hit routine, so it deals no damage to the hero (High).
- Burst (code 1): see the code table. The burst itself deals no damage; it summons four characters by key 2844. On disc, key 2844 is carried by Minion Grunt spawn records (monster row 183; attack damage 25, 25 and 50; class 17), which hit through the shared enemy chain (SPEC-0029 Behavior 10.5). Which record the final world resolves is Medium-Low.

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
- Silent: 6 (idle), 8 (code 4, aimed bolt), 10 and 14 (the burst; 14 may also be the stun) and 20 (absorb window, descent, fall and landed).

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
- For 50 seconds, she hovers at a target height above the ground probe. The target depends on the absorb count: 15 units for count 0, 10 for count 1, 5 for count 2 (from the tuning words).
- Each active hero missile (six slots) that overlaps her body is removed and adds one to the absorb count, including during the 3 s hold. The overlap test treats each armed hero shot slot as a sphere (the slot point and the slot radius) against her vertical body cylinder (her motion radius and height). Every absorb resets the 3 s hold; the hold does not pause the missile scan.
- When a hold ends and the count is 1 or 2, the count drops by one (she rises one step). When the count is 0, nothing happens.
- Three absorbs: code 9 (fall). The window timeout at 50 s: code 8 (timeout descent).

Code 8 (timeout descent): descends at 0.05 per tick to the ground. At the ground: sets levels 13 and 10, then code 1.

Code 9 (fall): descends at 0.1 per tick to the ground, with animation 20. At the ground: sets level 16, then code 10.

Code 10 (landed): waits while level 13 is clear. When level 13 is set: sets level 10, then code 1.

Confidence: High for the states, steps and per-tick absorb scan; Medium-High for the missile overlap test, including its side and end boundaries (see Open questions).

### Stun and the reflect weak point

- The weak point is the reflection. A hero who blocks an elemental orb (see Projectiles) turns it into a reflected orb. When the reflected orb reaches the boss, the boss enters code 5 with a 3 s timer. Stun length is 180 ticks.
- Code 5, each tick (High). The timer drops by 1. She holds animation 14. The stun hit needs three things on the same tick: the hero's in-attack test is true, the hero's attack form is form 2, and the hero's weapon reaches her body (the weapon sweep of SPEC-0029 Behavior 10.2 against her body parts, tried only when the hero is within 65.0 units). When all three hold, she enters code 6.
- Attack form 2 is Tor in attack sub-states 4, 7, 8 or 12, or Roberta in attack sub-state 4 (SPEC-0036). Sub-state 4 is the third swing of either hero's chain.
- A hit by any other attack form plays animation 13 once (a flinch) and leaves her in code 5. A flag stops a second flinch until the hero's in-attack test turns false. When animation 13 ends she returns to animation 14 while the timer runs.
- When the timer reaches 0 without a form-2 hit, she returns to code 0. A stun hit deals no damage (see Hit points).
- In code 6, each completed stun animation adds one. The first two return to code 0. The third sets level 12 and level 10 and leaves the boss in code 6. On the next tick the per-tick gate sees level 10 set and sets code 1. The chooser runs after the gate in that same tick, so a hero under 10 units away can still turn that code 1 into code 2 (see Chooser).
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
- The loop B chooser: if a class-38 entity of projectile type 1 is within 1.5 units of the boss, the choice is code 2 (a sidestep with a projectile). Otherwise, if the hero is under 15 units (from the tuning word), the choice is code 4 (hero knockback, animation 9, once: the same knockback as loop A, rescaled to 10.0 instead of 15.0; the hero is put in state 48 and a burst effect plays 0.5 above the boss). Otherwise the choice is code 3 or code 5 by a test.
- Code 3 (loop B): animation 7; one projectile at the event; cooldown 0.5 to 0.75 s.
- Code 5 (loop B): animation 7; one shot at the event; cooldown 2 to 4 s.
- Code 2 (loop B): animation 10; one projectile per side-step. Its deflected projectile uses a camera-facing quad marker whose in-plane half-axes are scaled by 2.4, giving a 4.8-unit full span along each unit basis axis. This scale changes the rendered marker only, not the projectile's direction or collision test.

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

Fight start step (rule 6, runs once). It is a short cinematic. It turns letterboxing on, plays camera 4 and waits for it, and fades in over 0.5 s. It starts effects 3 and 4, waits 0.5 s, and moves hero 1. It turns letterboxing off over 0.7 s, waits 1 s, and returns to the normal camera. The step also sets boss-fight instance 13 to stage 0. Flag 10 is set after the cinematic, and rule 6 no longer applies once it is set.

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
- Chooser, hero at distance 5 while code 7 (the absorb window) runs and the cooldown is 120: the code becomes 2 on that tick, and code 0 follows when animation 9 ends. Same with code 5 or code 6: no change.
- Chooser, hero at distance 20, code 0, cooldown 0, hero blocking: code 4 (aimed bolt), whatever the draw. Not blocking, draw 0 of 0 to 10: code 4; draw 1 to 10: code 3.
- Chooser with the hold flag set (after the third stun count): no code changes and the cooldown does not count down.
- Aimed bolt reaching 1.9 units (3D) from the hero: the slot clears and one summon by key 2864 is made at the bolt's position; the hero takes no damage from the bolt.
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

1. Resolved: her hit points stay 5. All three controllers set her invulnerable flag and nothing clears it, so no hero hit reaches her take-hit step (High).
2. Resolved: code 5's gates are the hero's in-attack test, attack form 2 and the hero's weapon reach (see Stun and the reflect weak point; High). The per-tick field cleared by the main controller is used by shared steering to enable the second input axis as vertical movement; it is not a hit gate. The field's wider role in other actors' steering remains outside this spec.
3. Resolved: the absorb test compares each armed hero-shot sphere (slot point and radius) against her vertical body axis from base to base plus up times height. In the central axial band, the coordinate along the axis may reach either cap plane inclusively, while the radial distance must be strictly less than the sum of radii; side tangency misses. Outside that band, the rounded ends use an inclusive point-to-segment distance check, so end tangency counts. The broad-phase distance check is inclusive. The lunge has no contact test (High for the lunge; Medium-High for the absorb geometry because the shared helper includes a vector-unit path).
4. Resolved: the finale trigger (trigger index 1) fires from cylinder record 8 of the final-fight world (High). The record values given under Triggers are the values stored in the world pack's cylinder list (kind 3, mode 0, flags 0x0008, gate mask 0, trigger index 1, base (28.702, 270.608, -326.005), axis (0, 1, 0), length 3.0, radius 6.0), and it is the only one of the list's 53 cylinder records with the trigger flag 0x0008. Its object field is non-null only on the hero's own trigger-hit path, which the hero's contact walk reaches (items 17, 18 and 32). Test vector: the hero standing on the arena floor with its axis 6.5 units from the record's axis overlaps it (6.0 plus the hero radius 1.0 reaches 7.0) and fires trigger index 1; at 7.5 units it does not. Resolved: control thread B's only script-local flag test is flag 6 (the fight has ended), besides level states 11 and 12 and the active hero (High).
5. Resolved: trigger 1 of script pack 1174 runs only when its module-local flag 15 is clear; its call sequence is in Earlier encounter. Flag 15 has no writer: every call of the module's set routine passes a constant bit number, and 15 occurs only in this one test (the stage-word table of SPEC-0044, module 1174, lists bit 15 as set by nothing). So the gate is always open and trigger 1 always runs when the hero crosses its rectangle; the reason the original author added the test is not recoverable (High for the missing writer).
6. Resolved: the final-fight world list identifies entities 1–4 as summon-reserved Minion Warriors with key 2864 and entities 5–7 as Minion Grunts with keys 2842, 2844 and 2841 respectively (see Control thread B; Medium-High for list positions). The summon system chooses an unbound reserved record with the requested key. Attack 1's spawn count 4 is the live-summon cap for that slot; each aimed bolt makes one summon request. Attack 2's spawn count 4 is the number of requests in the burst, matching the four explicit burst summons and the four Minion Grunt records with key 2844 in the Darkest Faerie row (SPEC-0029 Behavior 31; High for count meanings, Medium for which reserved 2844 records are available in the final world at each request).
7. Resolved: the lunge is a shove of the hero, not a move of hers. 12.0 is the vertical lane, 9.0 the horizontal direction from her to the hero, -5.0 the push-out factor on the radius shortfall, and 10.0 the final speed, so the hero's velocity is (6.0 d.x, 8.0, 6.0 d.z). It is a motion-record velocity, applied as velocity x 1/30 per tick (High; see Attacks).
8. Resolved: the near-range shove waits for neither the cooldown nor the ready test, and replaces any code except 2, 5 and 6. Codes 5 and 6 enter the far branch only to fail the ready test, so the chooser never changes them. The far branch also picks the aimed bolt whenever the hero is blocking (High; see Chooser).
9. Resolved: code 5 holds animation 14 and plays animation 13 once as a flinch when an attack of another form reaches her (High).
10. Resolved: level state 10 is the burst request: set when pack 1432 activates and by the controller's codes 6, 8 and 10, it makes the chooser pick code 1, and the entry sequence clears it (Level states used by the controller, Fight flow). Level state 13 is set by code 8 and by the director when the 15-second landed window ends; the director's landed-miss swap consumes it (Director, part (a) and (b)). Level state 14 is set by the director when the stun swap completes, to start the burst and the window, and code 1 clears it (High for the sets and clears; Medium-High for the meanings). Test vector: after the third stun and the swap to Roberta completing, level state 14 is set on the first tick after the swap; the next burst (code 1) clears it. Resolved for game state 358: it is the Ixi Chief's mode-2 flag (SPEC-0014), set only by the init of pack 1432 (High).
11. Resolved: one aimed-bolt detonation makes one summon-by-key request for key 2864. Its spawn count of 4 is the live-summon cap for that slot, not a request to summon all four reserved records at once; the summon system chooses one unbound matching record, if available (SPEC-0029, Behavior 31; High for request count and cap, Medium for which record binds).
12. Resolved: the lunge deals no damage (it has no hit routine), and the detonation deals none itself: it is a summon by key 2864, whose summoned character attacks with its own row values (High for the lunge and the summon). Resolved, the names part: every id of the kind table is an effect id. The first pair is the flight effect and the impact effect of the projectile; the third column (1617 to 1622) is the cast glow held on her during animation 7, not a sound. Her sounds are the block 388 cues of her animation table (see Sounds and effects; High).
13. Resolved: class 38 and row 132 are Queen Fyora (SPEC-0029, Behaviors 7 and 30; High). Loop B code 3 opens a type-0 homing slot from her sample point; code 5 opens a type-2 shot that homes to the aim point returned by the hero method and, on arrival, requests a spawn using the class-37 character row slot-0 key; the sidestep path opens a type-3 deflected slot. These are the shared four-slot behaviors in SPEC-0029, Behavior 19 (High for slot types and arrival action; Medium for the specific sample point and rendered projectile setup).
14. Resolved: the movie table positions are confirmed against the boot program: 16 = ACT1 115A1, 18 = ACT2 200A, 78 = ACT4 416A2, 80 = ACT4 417A2, 85 = ACT4 418A, 89 = ACTINTRO ACT2 (High; Earlier encounter, Movie indices).
15. Resolved in part: pack 1428 is an active scripted trap encounter in Queen Fyora's chambers, with both Queen Fyora and the Darkest Faerie; the script ends the scene by objective/flag progression rather than Faerie hit points (High). Pack 1259 is not reached in the local world-load graph, so its unnamed variant-0 record is not established as an active placement (High for the graph census; direct or untraced loading is not excluded). Pack 1344's named region has no established load in the local graph (High for the graph census; direct or untraced loading is not excluded). Pack 1098 conditionally loads its variant-0 region and has a hero-death restart callback, but those facts do not establish whether it is a fight (Medium-High). The roles of 1098, 1344 and 1259 remain open where noted.
16. Resolved: missile slots are scanned during the active absorb hold. Each overlapping active missile is removed, adds one to the absorb count and resets the hold; the hold does not pause scanning (High). Three total absorbs still start the fall.
17. Resolved: the object field is 1 only when the hero's own trigger-hit method fires the trigger; every other fire path passes 0 (High for the paths; Medium for the Faerie's method table). Resolved: on the contact-walk path the hero's trigger-hit method receives record-lookup argument 0 (a plain lookup, no remap), so it fires (High).
18. Resolved in part: trigger index 0 fires from the static rectangle (High for the record values), and trigger index 1 fires from the cylinder described under Triggers (High for the record values). Resolved: entities 5, 6 and 7 are Minion Grunts, which carry no volume, so control thread B's moves do not affect any trigger (High).
27. Play check for the fight entry: a hero walking across the rectangle near (36.4, 269.2, -344.0) in the final-fight world should start the entry sequence (flag 0, then movie 78 and the 180 s clock). If it does not, the trigger 0 path above is incomplete. Not established in play.
28. Resolved: the rectangle is in the volume lists of the grid cells the selected hero walks, and the hero's own body contact fires trigger 0 with object 1 (SPEC-0028, "Trigger pass"; High).
29. Resolved: the trigger-0 rectangle's reference point is its low corner for the runtime test. The test measures both in-plane coordinates from that point over 0 to the corresponding extent; pack 1432's extents are positive (High for the test and record values). This establishes operational geometry, not an editor or authored label (see Triggers).
30. Resolved statically, with a reachability limit: when trigger 0 sets flag 0 while flag 1 is clear, the next control tick takes the entry rule before the fight-start rule; the entry rule has no flag-9 or level-11 gate. The entry sequence later sets level state 11 and flag 1, so the fight-start rule is suppressed. Thus an early crossing is not harmless if it precedes the first fight-start tick. Whether a clean-route hero can cross early enough to do so is not established without a play check (High for rule order and writes; open for route timing).
31. Resolved: the trigger-1 volume is in the final-fight world pack itself (cylinder record 8, see Triggers). No outside source is needed (High).
32. Resolved: the hero's contact mode is a class constant (always the full mode) and the walk has no hero-state gate, so it runs during the landed window. The finale is reachable in play (High). A play check is still useful.
19. Resolved: the step is the frame-step word, 1/60 as a single-precision value (0.016666668), written once at start-up (SPEC-0023 section 31; High). The clock reads exactly 0.0 after its 10,800th running tick.
20. Resolved: a director swap lasts about max(W + 3, 37) + 41 ticks, where W is the outgoing hero's exit one-shot length, so about 80 ticks when W is 34 or less; the clock is stopped throughout, so the length never uses clock time (Swap length; Medium for the exact count, High that it uses no clock time).
21. Resolved: the player cannot switch by hand during the fight. The entry sets swap mode 3 and every director swap ends with mode 3, and mode 3 refuses the chord (SPEC-0053 section 3.1), so hero 1 is still active when the window closes and the landed-miss swap can run (High).
22. Resolved: pack 1432's init registers two control threads (the fight control thread and control thread B), trigger callbacks 0 and 1, one activate-world callback and two entity filters (the second replaces the first; Activation), but no hero-death callback. Trigger 0 sets flag 0; trigger 1 sets flag 4 when its object argument is non-null and flag 8 is set; the activate-world callback sets level state 10, activates objects 0, 1 and 2, starts effects 18 to 25 and sets flag 9; the effective filter refuses entities 5 to 7. Its clock start, stop, restart and hide paths are confined to the fight-entry, director and ending control-thread roots. A census of all 163 distinct recovered level modules found timer-control calls only in packs 1432, 1394, 758 and 934. The other three are ExteriorDLLInit modules; because the world system has one E buffer and releasing a world tears down its script, none can remain resident alongside pack 1432 in E. Pack 1427 is a TransitionDLLInit module and has no timer-control call. Thus no other recovered level-pack callback can alter the clock while pack 1432 occupies E (High, conditional on the documented buffer lifecycle). Resolved (High): no engine-owned callback changes the clock. The timer's state (value, limit, direction, running and visible flags, colour) is written only by the six timer host calls and by the timer's own per-pass update and draw; nothing else in the boot program stores to it. The only routines of the boot program that call the timer's hide operation are the world-exit paths of the front end (the quit confirmation on the pause page, the credits, the game-over page's quit hold and the front-end world load). They hide the clock (visibility only; the value and running flag are kept) when the player leaves the world, never during the fight. Test vector: pause during the fight with the clock at 100.0 s: the value holds while the page is up (SPEC-0023, section 31), and resumes counting from 100.0 when play resumes; only a confirmed quit hides it.
23. Resolved: she cannot lose hit points at all, inside or outside her stun, because she is invulnerable for her whole life (High; see Hit points and how they drop).
24. Resolved: all 64 level-state flags start clear: the world constructor allocates a zeroed 9-byte flag block when possible and reset clears it; without the block, QueryLevelState returns 0. Pack 1432's 32-bit script-local flag word is zero in the module payload and no constructor entry changes it before Init, so all its local flag bits start clear. SetLevelState true sets a flag and false clears it (High; see Activation).
25. Whether 180 s is enough for a typical player to reach the finale in play. Settled from the disc: swaps use no clock time (Swap length), and each landed window uses 15 s of clock (900 ticks), so the clock is spent only on the fighting between swaps and on the windows. Still open: how long a player takes to stun her and to land the finale, which needs play.
26. Resolved: each entry's death check tests that entry's own entity (7, 6 or 5). All three are Minion Grunts; entity 7 is not the Darkest Faerie, who is entity 0 (High for the test, Medium-High for the list positions).
33. Resolved: not reached in play, because her hit points never fall (High).

