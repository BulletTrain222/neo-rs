# SPEC-0062: Boss encounter progression atlas

- Answers: proactive (open close-out); proactive, none requested (boss encounter overview); proactive (boss consistency pass 3); proactive (boss close); proactive (serpent win); R-0136 (the Darkest Faerie arena entry); R-0137 (player-facing instructions per boss); R-0144 (arrival records per boss arena); proactive (verify pass: boss start position and facing table)
- Status: Draft
- Disc build: SCUS-97367 v1.00 (same fingerprint as SPEC-0001)
- Confidence: High for the listed shipped entry, stage and completion links where cited; Medium for Gelert's ordinal-to-slot alignment and some encounter branch labels. This atlas summarizes individual boss specifications and does not replace their attack and collision rules.

## Summary

This atlas connects each boss's behavior to the world-pack script that starts, advances or ends its encounter. It also distinguishes active fights from objective encounters and unplaced controller behavior. Use the linked boss specification for attack selection, hit shapes, damage, animation timing and per-tick details.

## Where it lives on the disc

The encounter-specific rules come from the boss specifications and the world-pack level-script specifications listed in each row. The table records pack and boss-fight instance identities when established. A pack number identifies an archive resource.

## Layout

| Encounter | World and controller link | Entry and progression link | Completion or boundary |
|---|---|---|---|
| Undead Draik | Pack 1096; instance 6; behavior in SPEC-0029 | Movie 51, hero locator 10; three vulnerable stages | 200 HP; 40 accepted 5-HP hits at full health without healing; state 12 starts the finale and flags 45/316. Arena: SPEC-0029 section 6.12. |
| Faerie Sisters | Pack 1122; instance 11; SPEC-0011 | Trigger 6, movie 52, hero locator 26; shared shield breaks on a hero shot | Three independent 400-HP pools; event 2 at zero living sisters starts movie 54 ending. The dome attack does not occur in the shipped loop. |
| Black Knight | Pack 989; instance 7; SPEC-0012 | Movie 46 starts the fight after stage setup | Six weak-point pools govern defeat; event state 0 starts the post-fight path and movie 47 return. The hero spawns 53.26 units from the knight; movie 47 is queued about 9 ticks after the defeat tick. |
| Gelert Assassin | Pack 985, flag-281-set branch; instance 5; SPEC-0013 | Arrival and opening sequence place hero at locator 9; five records map to primary plus four clones | Boss states 1, 3 and 5 drive three intermediate scenes; state 7 drives the final scene and instance stage 1. The first eligible range pass precedes the activation-callback body and sees the pre-callback hero pose; actual record binding remains conditional. |
| Ixi Chieftain (Cogham Steppes) | Pack 800; entity 9; instance 2; SPEC-0014 | Two entity-10/11 Henchmen gate the opening; five ranged Henchmen enter at the half-health phase transition | Mode 0 uses level states 0–3 for release, transition, phase release and death; the finale activates placement ID 12, which maps to item 9 (Raider Boss Eye Patch), sets flag 55 and grants that item on exit if absent. |
| Ixi Chieftain (Abandoned Mines) | Pack 864; region set 4; instance 8; SPEC-0014 | Script trigger 1 starts the intro; clearing level state 3 releases the boss | Mode 1 death sets level state 0; the level thread runs the end sequence and ends instance stage 1. |
| Juppie | Pack 1217; SPEC-0015 | Three level flags move the boss from dormant through stages 1–3 | HP thresholds are 5/6, 1/2 and 0; Tentacles increase from one to two to three across the stages. A stage whose locators are all used without reaching its HP threshold is replayed (SPEC-0015, Stages). |
| Plague Serpent | Swamp pack 735; SPEC-0016 | HP bands 75%, 50% and 25% begin two-pass anchor changes | Each pass posts an anchor-number state; the swamp thread plays its matching break scene. The serpent is posed at twice its clip's size, so its lunge (12 to 21 units ahead) touches only a hero whose feet are at least 6.0 above its origin (about 114.4; 5.25 in a jump pose); a hero standing on the ring, a pad or a risen platform (at most 112.55) is out of reach, and nothing guards in the air. The fight is won from a pillar top: the seven pillars (objects 59 to 65, tops 116.3 to 118.9, one per slot 1 to 7) are climbed by a standing jump, a hang and a pull-up (from the ring for slots 1, 2, 3, 6 and 7; slots 4 and 5 from the platform that rose beside them), the hero then stands 8.1 to 10.1 above the serpent's origin, is a bite candidate without jumping, and holds the shield towards the lunge. Slot 3 never bites before phase 4, so use the other pillars. Two pillars fall at each phase change and their platforms rise (the platform of group 3 never rises). The small ring pad near slot 1 (top 112.06) and the risen platforms only let a jump pass 115.0. A fall off the ring is game over. See SPEC-0016, Arena build guide and Reach and distances (How the fight is won). |
| Spyder Nest | Nest and valves; SPEC-0017; spiders use SPEC-0038 | Encounter setup binds the local character records and assigns spiders to three valves | All three valves destroyed defeats the nest. This is an objective encounter rather than a single boss health bar. |
| Werelupe King | Pack 1246; ordinary streamed record; SPEC-0019 | The King starts when the eligible record binds near the hero | Health bands are strictly below 75% and 37%; pack 985's alternate branch holds him instead of running this fight. Play rate 1.0, 1.2, 1.4 by phase; arena guide in SPEC-0019. |
| Gnarfas | Pack 985, flag-281-clear branch; instance 1; SPEC-0020 | Event and post-movie script hand-off jointly enable phase 2 | Phase 2 restores hero health; rage begins below integer percent 25; death event 2 drives the ending. The hold, knockback and death gates test whether the hero is down (action state 16). Roberta's shots flinch him but deal 0 damage; only melee lowers his health (SPEC-0020). |
| Darkest Faerie | Final pack 1432; earlier pack 1174; SPEC-0054 | Final arena entry starts a 180-second script clock; timed absorb windows drive the fight | Covered hero-hit paths do not reduce her stored 5 HP; progress is script-led through absorbs, fall/landing and finale. Arena: a sealed round hall of radius about 40.1 centred at (0, 270.63, -326.0), with her axis A at (28.713, -326.005); three Grunt rows (183 fire, 187 slow, 188 curse); Roberta's station is 4 to 7 units east of A (SPEC-0054 arena guide). |
| Queen Fyora chamber encounter | Pack 1428; Queen Fyora and Darkest Faerie; SPEC-0054 | Active scripted trap encounter in the Queen's chambers | The scene advances through objective/flag progression; an HP-based defeat is not established. |
| Cloud Beast | Controller behavior in SPEC-0010 | No checked placed encounter or start script in the known world graph | Do not invent a shipped arena or defeat sequence; the controller-only behavior remains separate. |

### Where the hero stands when each fight starts (for a start that skips the walk-in)

Entry records are counted from 0 in the pack's entry list (SPEC-0006). A load that names no teleport plane places the hero at entry record 0, so a pack's record 0 is only a real fight arrival where the table says so. "Use" is the record a developer start should use: it is the placement the game's own script makes, so the fight's guards, walls and refills behave as they do in play. Heights are the stored values; the ground probe snaps the hero to the floor (SPEC-0006).

| Fight | Pack | What the game does | Use (entry record, position, facing) |
|---|---|---|---|
| Gnarfas (flag 281 clear) | 985 | Load at record 0 (far away), then the pack's activation callback moves the hero to record 9 at once; one tick later Gnarfas is moved to record 11 and the King grabbed | Record 9 (-637.03, 122.67, -567.24), facing (0.546, 0, 0.838); run the activation callback (walls, refill) |
| Gelert Assassin (flag 281 set) | 985 | Same callback; the intro moves the hero to record 9 again | Record 9, as above |
| Werelupe King | 1246 | The hero walks in from the neighbouring pack; touching the entrance sphere starts a walk to record 10, then after a fade the script places the hero at record 11; the intro later walks the hero to record 8 | Record 11 (752.00, 5.08, -2135.30), facing (-0.906, 0.061, 0.418); the arena sphere then fires the entry script. Record 10 (839.37, 5.61, -2149.50) is the entrance walk target, record 8 (747.13, 0.27, -2132.90) the fight start after the intro |
| Plague Serpent | 735 | No arrival record: the hero walks over the bridge pad and the fight starts when its body overlaps the trigger box on the ring | Record 5 (3756.20, 112.41, -622.41), facing (-0.809, 0, -0.588): inside the box, on the ring (ground 110.13), not on the pad the intro removes |
| Undead Draik | 1096 | Arrival at record 0 (363.21, -1612.31, -489.41); the entry script fades out, plays movie 51 and moves the hero to record 10 | Record 10 (367.23, -1613.16, -493.38), facing (0.491, 0, 0.871) |
| Faerie Sisters | 1122 | The hero walks east from the west wing (record 23 is the arrival point); crossing the entry sheet plays movie 52, then the script places the hero at record 26 | Record 23 (-26.97, -25.33, -113.19), facing +x, to run the entry; record 26 (7.64, -25.34, -134.62), facing -z, to start after it |
| Black Knight | 989 | Load at record 0; the pack queues movie 46 when it becomes current | Record 0 (0.00, 2.77, -49.12), facing +z (floor 0.0) |
| Ixi Chieftain, Cogham Steppes | 800 | The world spawn lies outside the fight-start box; the hero walks into it; the scene walks the hero to record 13 and places it at record 14 when the dialogue ends | Record 14 (473.16, 296.31, 2386.64), facing (0.779, 0, 0.627) after the scene; the Henchmen belong at records 25 and 26 |
| Ixi Chieftain, Abandoned Mines | 864 | Script trigger 1 (a cylinder round the floor) starts the intro, which places the hero at record 45 and the Chieftain at record 46 | Record 45 (-839.08, 40.60, 2282.72), facing (0.908, 0, 0.419); the record lies inside the trigger cylinder |
| Juppie, Act 1 | 1217 | Arrival at record 0; the start volume lies 9.9 ahead; intro A moves the hero to record 88, walks it to 89 and places it at record 90 | Record 0 (12.67, 0.05, -33.00), facing (-0.579, 0, 0.815), then walk ahead; record 90 (0.00, 0.00, 6.35) is where the intro leaves the hero |
| Juppie, Act 3 | 1217 | The activation starts the intro, which moves the hero to record 1 and lets it fall | Record 1 (2.28, 24.51, -0.15), facing (-0.579, 0, 0.815) |
| Darkest Faerie | 1432 | Load at record 0 in the alcove; the fight-start step moves the hero to record 1 at tick 30 | Record 0 (0.05, 273.84, -265.06), facing +z, with the pack's activation; or record 1 (0.04, 271.79, -325.72), facing +x, with the fight-start step marked done |
| Queen Fyora chamber | 1428 | Arrival at record 0; the entry rectangle starts the trap, which places the hero at record 3 | Record 0 (0.00, 10.65, -269.60), facing -z; record 3 (17.91, 10.65, -308.36), facing (-0.5, 0, -0.866), after the trap |
| Cloud Beast, Spyder Nest | none | No shipped arena | No record; do not invent one |

Confidence: High for the record numbers and positions (taken from the packs' entry lists and the scripts' placement calls); Medium-High for the Serpent (the footing at record 5 follows the ring data) and for the Draik and Sisters entry order (the movie hand-offs are queued, SPEC-0026).

### Where each boss starts and which way it faces

A boss is an ordinary placed character (SPEC-0006, kind 3). It is bound at its record's position and heading, and a script may then move it. Headings are in degrees, atan2(x, z) of the facing vector, counted from +Z toward +X (+Z 0, +X 90, -Z 180, -X 270; SPEC-0038, Spawn facing and what overrides it). The entity is the index in the pack's entity table. Heights are stored values; the ground probe replaces them where a row says so. "Held heading" is the heading the boss shows once the fight runs. A script move with the face option writes the body's axes for one tick only unless the entity is script-held (SPEC-0038, same section).

| Boss | Pack, entity | Placement record: position, heading | Script placement at the fight start: position, heading | Held heading |
| --- | --- | --- | --- | --- |
| Undead Draik | 1096, entity 0 | (390.46, -1605.28, -509.00), 309.4 | none; the hero arrives at locator 10 (367.23, -1613.16, -493.38), heading 29.4, 28.0 away | 309.4 (cosine 0.995 to the hero's start) |
| Faerie Sisters | 1122, entities 1 to 3 (Vanity, Spite, Malice) | (4.38, -25.30, -175.08), (9.53, -25.30, -183.50) and (14.86, -25.30, -180.21); all 0.0 | none; the hero is placed at locator 26 (7.64, -25.34, -134.62), heading 180.0 | 0.0 (toward the hero's start) until each sister's own steering |
| Black Knight | 989, entity 0 | (0.00, 0.00, 4.14), 180.2 | none; the hero starts at locator 0 (0.00, 2.77, -49.12), heading 0.0, 53.26 away | 180.2 (toward the hero) |
| Gelert Assassin | 985, entities 0 to 4 (primary and four clones, five identical records) | (-623.68, 121.88, -542.24), 209.1 | none at the opening (hero at locator 9 (-637.03, 122.67, -567.24), heading 33.1); the stage scenes move entity 0 to locator 27 (-624.26, 122.03, -545.80), heading 218.6 | 209.1 |
| Gnarfas | 985, entity 7 | (-617.36, 126.22, -558.80), 0.0 | one tick after the callback, a move to locator 11 (-617.36, 123.34, -558.80), heading 245.2, face on, control 1; only the height and the heading differ from the record | 0.0; the locator heading shows for one tick (Medium) |
| Werelupe King | 1246, entity 16 | (731.08, 5.29, -2124.88), 117.8 | the intro and scenes move him to locator 7 (737.43, 0.31, -2128.46), heading 114.6, face on (control 1 in the intro, 0 in the scenes). In pack 985's alternate branch entity 8 stands held at (-580.80, 121.88, -496.08), heading 217.1 | 117.8; 114.6 while a scene holds him |
| Ixi Chieftain, Cogham Steppes | 800, entity 9 | (513.61, 306.91, 2411.89), 243.8 | stages C and D move him to locator 17 (501.49, 296.31, 2404.70), heading 231.3 | 243.8, or 231.3 while a scene holds him (Medium) |
| Ixi Chieftain, Abandoned Mines | 864, entity 44 | (-798.61, 45.34, 2283.74), 267.8 | the fight start moves him to locator 46 (-805.07, 41.66, 2282.24), heading 279.3; the hero goes to locator 45 (-839.08, 40.60, 2282.72), heading 65.2, 34.0 away | 267.8 (279.3 while held) |
| Juppie | 1217, entities 7 and 8 (two identical records, one in each act's region) | (4.31, 1.91, 25.92), 184.2 | none; the Tentacles (entities 0 to 2) are placed at locators 13 to 18, all heading 0.0 (locator 13 is (7.31, 0.00, -6.80)); Tentacle entity 0's own record is (-0.35, 3.61, 4.23), heading 185.5 | 184.2 |
| Plague Serpent | 735, entity 73 | (2158.30, 111.64, -3161.63), 90.2 | the intro and each arrival place it at the slot's point C or A, position only; slot 0's point C is locator 28 (3736.49, 110.69 replaced by the probed 108.39, -630.63) | 90.2 until the move state turns it toward the anchor axis at 0.25 per tick (Medium) |
| Darkest Faerie, final fight | 1432, entity 0 | (57.66, 270.64, -326.08), 97.7 | the entry script moves entity 0 to locator 2 (28.71, 272.49, -326.01), heading 270.0, face on, control 1; the hero goes to locators 3 (-10.79, 272.49, -325.42, heading 90.0) and then 1 | 97.7; the locator heading shows for one tick (Medium-Low) |
| Queen Fyora chamber | 1428, entities 0 (Fyora) and 1 (Darkest Faerie) | Fyora (0.20, 8.37, -325.03), 298.9; Faerie (-22.97, 8.37, -314.76), 90.0 | the return to the boss room places Fyora at locator 1 (0.81, 8.37, -303.40), heading 180.0 and the Faerie at locator 2 (0.02, 8.37, -359.43), heading 0.0 | per move (Medium) |
| Cloud Beast, Spyder Nest | none | none placed as a boss | none | none |

Confidence: High for the record positions and headings and the locator positions and headings (disc); Medium for the held-heading column, which depends on the stored-heading rule of SPEC-0038 and on each boss's own controller turning it.

## Behavior

### Reading the encounter rows

The table gives only the fight-to-script hand-off. Boss behavior remains authoritative for attack thresholds and per-tick order. In particular, a boss's displayed health may not equal its true defeat test (Black Knight), a phase can depend on a level event rather than boss health (Gnarfas), and a boss-like encounter can complete through objectives rather than a boss health pool (Spyder Nest and Darkest Faerie).

### Player-facing fight cues (R-0137)

| Encounter | How to open or deal damage | What to avoid |
|---|---|---|
| Cloud Beast | No shipped fight or checked placement is known; the known controller does not establish a usable retail strategy. | Do not infer an arena or a normal boss defeat from the controller alone. |
| Faerie Sisters | Shoot one sister once to break the shared shield; then use sword swings, Roberta's staff or shots when a sister is in a damage window and not dodging. Defeat all three. | Dodge their energy balls and missiles; a shield reduces a ball, but only a matching shield Mote reflects it. |
| Black Knight | Wait for one of its attacks to miss and expose a weak point, then hit that part from its vulnerable side; under-arms and neck need a jump attack or Roberta's strike. Armed quick shots also hurt an open part. | Swings against closed parts are blocked. Face and guard only against its slow and angry sword swings; the kick, charge, high swing, floor strike and sword spin require evasion. |
| Gelert Assassin | Use sword hits or Roberta's quick shots while it is in a hittable state. A hit removes a clone; the primary flinches when a valid hit lands. | Wait out its hidden, dodge and deflect states. Facing swings are blocked and shots are returned during the deflect pose; guard its lunge, sword swing and jump thrust, and evade the attacks that cannot be blocked. |
| Ixi Chieftain | In the Cogham fight, defeat the two opening Henchmen to release him. Then hit him in damage-eligible states; when he guards, attack from the side or behind. | Only the forward swipe is blockable. Step out of the wide swipe, slam, whirl, buck kick and charge; the later ranged Henchmen join at the health transition. |
| Juppie | Defeat the current one, two or three Tentacles. Lure a head slam onto an unused marked floor spot, then hit the stuck head during its stun: melee in Act 1, Roberta's shots into the mouth in Act 3. | Leave the slam line or its targeting band; in Act 3, face and shield against spit. |
| Plague Serpent | Stand on a pillar top, face the lunge and guard it to stun the serpent; while stunned, land one sword hit or shot near its head. Repeat until its health is gone. | The low ring and raised platforms alone are too low for a standing guard to meet the lunge. An unblocked lunge hurts, the belch/spit applies poison, and falling into the swamp ends the fight. |
| Undead Draik | Reflect its breath beam with a guard and a shield Mote matching the beam's fire or ice element. While stunned in stages 1 and 3, use sword swings, staff swats or shots on the vulnerable body groups; in stage 5, hit the head. | Outside the stun, the first two stages block attacks. The claw, bite, spit, wind and beam ignore the shield. |
| Werelupe King | Use sword swings, staff and shots, or Mote attacks during free states. When he guards, flank him or use a Mote power step, Mote attack or full-release shot. | Time hits after each flinch. A shield does not stop the boomerang or flame rings; move above or out of the ring path. |
| Gnarfas | Only Tor's sword or Roberta's staff swat lowers his health. Strike after an attack ends, during exhaustion or laughter, or in the jump sequence; a sidestepping Gnarfas can still be hit. | Avoid standing in front of the chain whip, rapid strike, head butt or body slam; a shot only flinches him and deals no damage. |
| Darkest Faerie | She cannot be damaged directly. Reflect an elemental orb with a guard and matching shield Mote to stun her; during the stun, use Tor's form-2 hit or third chain swing. After three stuns, Roberta must have three shots absorbed during the 50-second window (lock-on or aim low), then reach the landing spot. | Stay at least 10 units away while she is free to avoid the shove; an aimed bolt cannot be reflected, and the fight has a 180-second clock. |

These cues condense the player-summary sections of the encounter specifications. They do not replace the state, range, hit-shape or timing rules in those specifications. Confidence is High for the stated damage routes and gates; Medium where the individual boss spec marks a runtime or control-dependent limit.

### Reproducible progression boundaries

- Draik at 200 HP takes 14 accepted stage-1 hits to reach 130 HP, then 13 stage-3 hits to reach 65 HP, then 13 head hits to reach zero. The first two phases need a reflected-breath stun window.
- Sisters at 400 HP each are outside the status band at exactly 264 HP and enter it at 263; the special band begins below 132, so 131 HP qualifies. The shared shield-breaking shot subtracts no sister HP.
- Juppie stage thresholds and Werelupe King health bands use different comparisons: Juppie completes at or below its stage fraction; the King advances only strictly below 75% or 37%.
- The Serpent's paired phase flags are handled in ascending state order. If it posts state 2 before state 1, scene 1 still runs first.
- Ixi's normal-mode death event and Gnarfas's script-gated phase-2 hand-off are level-state transitions; do not replace them with health-only transitions.

## Cross-boss rule matrix

Every boss guide uses the same shared rules; this matrix says which of them each boss triggers. Unhittable means the boss sets its word at +0x7F0, so the generic hero walks (swing resolver, Roberta's shot walk, Mote objects, random target finder) never select it. Guard means base state 20, which still lets the swing and shot code meet it as a guarding target.

**Hero side**

| Boss | Unhittable word | Guard (base state 20) and Blocking Angle | Real damage routes | Shots |
|---|---|---|---|---|
| Cloud Beast (cut) | 1 on both bodies on every tick outside its reaction state, never cleared | Reaction state only; row angle 0 | None from the hero; only its own 20-HP reply to a reflected beam | Pass outside the reaction state; bounce inside it; full release disarmed; nothing paid |
| Faerie Sisters | Set once after a 5-tick wait, never cleared | Never | Own routine: melee trunc(A x B x C x D x E x G) with power scale 1.0, shot trunc(S x C x T); only after the shared shield is broken by a shot | Own routine pays quick-shot magic and Mote charge on every touch, even absorbed; full release pays nothing |
| Black Knight | 1 on every tick | Every tick except stun-gate ticks; angle written as 1.0 | Own weak-point routine: trunc(A x B x C x D x E), power 1.0, in an open stun window and the right half-space | Hurt a part from any hero action; never pays; quick shots bounce otherwise and return once the deflect flag is set |
| Gelert Assassin | 1 from creation, never cleared | Only while the deflect clip plays; angle 0.0 | Own hit step in its hittable states: trunc(hero hit x damage factor) | Pays on a touch that hurts it; deflected or passing shots pay nothing |
| Ixi Chieftain | Cleared at the head of each pass; set by the idle, hold and stage-transition handlers | Idle, hold, stage transition, both block states; angle 0.0 | Own check (every state but hold, get-hit, stage transition) plus the shared path in non-guarding hittable states | A guarding boss bounces shots; the own check counts an overlapping shot and pays |
| Juppie | 1 from start | None recorded | Own test: stun-loop melee at 4 x, dazed-loop shots inside the mouth cone | Counted shots pay; other touches bounce and pay nothing |
| Plague Serpent | 1 on every tick | None (base state 1) | One head strike per stun: melee query with power 1.0, by swing or by a shot near the head | Counted as the strike, never pays |
| Werelupe King | Set only on flinch, catch, roll, jump and hold ticks | State 7 and its recovery; angle 0.0 | Shared path: hero damage, defence 0, first call of its take-hit doubled | Pays on a hit; bounces off a block with no deflect flag |
| Gnarfas | 1 from controller start, never cleared | Block-missile state only; angle 0.0 | Own check: melee at power 1.0, applied when the flinch ends | Flinch with damage 0, nothing paid (slot cleared before the damage query) |
| Darkest Faerie | 1 (flag) | Only in the pack-1428 loop; the final fight never guards | None; a form-2 hit counts a stun, a Roberta shot in code 7 counts an absorb | Absorbed shots pay nothing |
| Undead Draik | 1 once before its loop | Every tick except states 1, 3, 26, 27 and stage 5; angle written as 2.0 | Own check: 5 HP per accepted hit by stage and state | Counted shots never pay; a full release is disarmed on contact |

**Timing and ending**

| Boss | Forced lock write | Play rate R | Missing ids | End rule and hand-off |
|---|---|---|---|---|
| Cloud Beast (cut) | Its own id every tick while the companion exists | 1.0; gust stages 1.25, 2.0, 3.0 | Stand-in table must hold every requested id | No end; fights on at 0 HP |
| Faerie Sisters | Candidate record forced engaged (not the lock id) | 2 minus HP fraction | None missing | State 9 for good; event 1 at the first shot, event 2 at zero alive |
| Black Knight | Its id every tick | 1.0 stored | None (ids 0 to 47 have clips) | All six parts down; stays guarding; event state 0 |
| Gelert Assassin | None | 1.0; 2.0 in back-stab and mirror attack | None requested | Band states 1, 3, 5, 7 drive the scenes |
| Ixi Chieftain | None | 1.0 (Henchmen strikes 1.3) | Henchman attack phases skip a missing id, other requests play clip 0 | Mode 0 level states 0 to 3; mode 1 death sets level state 0 |
| Juppie | None | 1.0 | None | HP at or below 5/6, 1/2, 0 with locator use; the controller stops at death |
| Plague Serpent | None (target-point override only) | 1.0; bite windup differs | States 2 and 3 never entered | Bands 75, 50, 25 percent post anchor states |
| Werelupe King | Its id every pass | controller speed word copied to P: 1.0, 1.2 after the phase 2 entry, 1.4 after the phase 3 entry | None | Below 0.75 and 0.37; state 20 at 0 or less; hero down means action state 16 |
| Gnarfas | None | 1.0 setting | Ids 23 and 25 never requested | Health below 1 percent; hero down means action state 16 |
| Darkest Faerie | None | 1.0 | None | Script finale; 180-second clock |
| Undead Draik | None | 1.0 | Ids 11, 25, 35, 48 and 57 up never requested | Stage 6 after the head hit at HP 5; level state 12 |

**Shared rules behind the rows**

- Guard rule. A guarding target never takes damage from a non-power swing. It blocks only when the dot product of its facing and the hero's facing is below its row's Blocking Angle; a blocked swing counts as landed (Tor pays his swing magic), and Tor recoils into attack sub-state 13 when his sub-state is 2, 3, 4, 6, 11 or 12 (Roberta never recoils). A swing that is not blocked has no effect at all. A Tor power step with a Mote skips the block test and needs the target to be hittable.
- Shots against a target that is guarding and not hittable: a quick shot with a zero arming counter bounces from any angle and pays nothing; a full release is disarmed with no damage.
- Payment. Quick-shot magic and Mote charge are paid inside the shared shot-damage routine, so every boss that asks it about a touching shot pays (the Sisters, Gelert, Ixi, Juppie, and the generic path for hittable targets such as the Werelupe King); bosses that remove the shot slot themselves (Black Knight, Serpent, Darkest Faerie, Draik) or clear it first (Gnarfas) pay nothing.
- Gate rule. At rate 1.0 a clip of speed byte S moves 1 / (2S) key per tick from the request tick T. A boss reads progress before the animation pass, so a one-shot clip of F keys passes the 0.99 gate on tick T + 2S(F - 1) + 1, one tick earlier from F = 50 / S (S = 1: 50 keys; S = 2: 25 keys) and two ticks earlier from three times that. Examples: S = 2, 20 keys: 77; 25 keys: 96; 33 keys: 128. S = 1, 46 keys: 91; 61 keys: 120. Other rates use the same stepping with v = R / (2S).

## Edge cases and error handling

- The Black Knight's landed or completed enraged attacks reset its phase to the slow-swing phase; a missed angry swing is the route back to phase 2.
- The Sisters' dome gate is reset before it can be tested, so no dome is part of the shipped fight.
- Werelupe King phase 3 can regress to phase 2 if health rises into the middle band; preserve the observed rule if implementing healing.
- The Plague Serpent has a documented state-11 soft-lock when both stored phase-change slots are absent; the normal reachable route avoids it.
- The Cloud Beast controller's presence does not establish a shipped encounter.

## Test vectors

- Draik from full health, no healing: after 13 accepted phase-1 hits, 135 HP remains and no phase change occurs; hit 14 leaves 130 and passes the 66% threshold.
- Sisters: 264/400 is not below 66%; 263/400 is. 132/400 is not below 33%; 131/400 is.
- Plague Serpent posts states 2 then 1 in one phase transition: scene 1 completes and clears before scene 2 begins.
- Pack 800 Ixi reaches less than half health in mode 0: level state 1 is set; after the script introduces the ranged Henchmen, it sets level state 2 and the boss enters stage 2. In pack 864 mode 1, zero HP sets level state 0 and the arena thread later ends instance 8 at stage 1.
- Gnarfas has event bit 0 set but bit 1 clear: phase 2 does not start until the script sets bit 1 after its mid-fight scene.

## Open questions

- Cloud Beast's shipped placement and intended arena: the disc holds no placed record and no arena for it (unknowable, developer intent; the fight is a test-spawn only, SPEC-0010).
- Resolved: Black Knight part-frame-to-anatomy mapping. Joints 37 and 34 are the knees (about 1.16 high, on the -x and +x side), 24 and 9 the under-arm (arm-root) joints (about 4.6), 4 the back (about 3.85), 7 the neck (about 5.44) (SPEC-0012, Weak-point test and Open question 21; High). The cylinder centre and axis rule is stated there too.
- Which Gelert records bind on the first range pass depends on stream eligibility, occupied slots and candidate order at run time. Default if unresolved (rewrite choice): bind in record order as each comes into range (needs a runtime check).
- Resolved: how the Plague Serpent is stunned at its true size (twice the clip's size). The hero stands on a pillar top and blocks there; no airborne guard exists (SPEC-0016, Open question 30 and How the fight is won). Remaining Medium: the hang offer of the pillar walls and the exact pillar-to-platform routes for slots 4 and 5. Default if unresolved (rewrite choice): treat pillar walls as not hangable and allow only the jump routes of SPEC-0016 (needs a runtime check).
- Camera art, exact presentation timing at thread boundaries, and unnamed script flags: the camera art is unknowable (D); timing at thread boundaries is set in each encounter specification, and a flag with no named reader is stored and ignored.


