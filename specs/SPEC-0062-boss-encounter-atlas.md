# SPEC-0062: Boss encounter progression atlas

- Answers: proactive, none requested (boss encounter overview)
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
| Undead Draik | Pack 1096; instance 6; behavior in SPEC-0029 | Movie 51, hero locator 10; three vulnerable stages | 200 HP; 40 accepted 5-HP hits at full health without healing; state 12 starts the finale and flags 45/316. |
| Faerie Sisters | Pack 1122; instance 11; SPEC-0011 | Trigger 6, movie 52, hero locator 26; shared shield breaks on a hero shot | Three independent 400-HP pools; event 2 at zero living sisters starts movie 54 ending. The dome attack does not occur in the shipped loop. |
| Black Knight | Pack 989; instance 7; SPEC-0012 | Movie 46 starts the fight after stage setup | Six weak-point pools govern defeat; event state 0 starts the post-fight path and movie 47 return. |
| Gelert Assassin | Pack 985, flag-281-set branch; instance 5; SPEC-0013 | Arrival and opening sequence place hero at locator 9; five records map to primary plus four clones | Boss states 1, 3 and 5 drive three intermediate scenes; state 7 drives the final scene and instance stage 1. First stream-check scheduler order is unknown. |
| Ixi Chieftain | Pack 864; region set 4; instance 8; SPEC-0014 | Script trigger 1 starts the intro; clearing level state 3 releases the boss | Normal-mode death sets level state 0; the level thread then runs the end sequence and ends instance stage 1. |
| Juppie | Pack 1217; SPEC-0015 | Three level flags move the boss from dormant through stages 1–3 | HP thresholds are 5/6, 1/2 and 0; Tentacles increase from one to two to three across the stages. |
| Plague Serpent | Swamp pack 735; SPEC-0016 | HP bands 75%, 50% and 25% begin two-pass anchor changes | Each pass posts an anchor-number state; the swamp thread plays its matching break scene. Platform route/access remains unresolved. |
| Spyder Nest | Nest and valves; SPEC-0017; spiders use SPEC-0038 | Encounter setup binds the local character records and assigns spiders to three valves | All three valves destroyed defeats the nest. This is an objective encounter rather than a single boss health bar. |
| Werelupe King | Pack 1246; ordinary streamed record; SPEC-0019 | The King starts when the eligible record binds near the hero | Health bands are strictly below 75% and 37%; pack 985's alternate branch holds him instead of running this fight. |
| Gnarfas | Pack 985, flag-281-clear branch; instance 1; SPEC-0020 | Event and post-movie script hand-off jointly enable phase 2 | Phase 2 restores hero health; rage begins below integer percent 25; death event 2 drives the ending. |
| Darkest Faerie | Final pack 1432; earlier pack 1174; SPEC-0054 | Final arena entry starts a 180-second script clock; timed absorb windows drive the fight | Covered hero-hit paths do not reduce her stored 5 HP; progress is script-led through absorbs, fall/landing and finale. |
| Queen Fyora chamber encounter | Pack 1428; Queen Fyora and Darkest Faerie; SPEC-0054 | Active scripted trap encounter in the Queen's chambers | The scene advances through objective/flag progression; an HP-based defeat is not established. |
| Cloud Beast | Controller behavior in SPEC-0010 | No verified placed encounter or start script in the known world graph | Do not invent a shipped arena or defeat sequence; the controller-only behavior remains separate. |

## Behavior

### Reading the encounter rows

The table gives only the fight-to-script hand-off. Boss behavior remains authoritative for attack thresholds and per-tick order. In particular, a boss's displayed health may not equal its true defeat test (Black Knight), a phase can depend on a level event rather than boss health (Gnarfas), and a boss-like encounter can complete through objectives rather than a boss health pool (Spyder Nest and Darkest Faerie).

### Reproducible progression boundaries

- Draik at 200 HP takes 14 accepted stage-1 hits to reach 130 HP, then 13 stage-3 hits to reach 65 HP, then 13 head hits to reach zero. The first two phases need a reflected-breath stun window.
- Sisters at 400 HP each are outside the status band at exactly 264 HP and enter it at 263; the special band begins below 132, so 131 HP qualifies. The shared shield-breaking shot subtracts no sister HP.
- Juppie stage thresholds and Werelupe King health bands use different comparisons: Juppie completes at or below its stage fraction; the King advances only strictly below 75% or 37%.
- The Serpent's paired phase flags are handled in ascending state order. If it posts state 2 before state 1, scene 1 still runs first.
- Ixi's normal-mode death event and Gnarfas's script-gated phase-2 hand-off are level-state transitions; do not replace them with health-only transitions.

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
- Ixi in normal mode reaches zero HP: level state 0 is set; the arena thread marks the ending, then ends boss-fight instance 8 at stage 1.
- Gnarfas has event bit 0 set but bit 1 clear: phase 2 does not start until the script sets bit 1 after its mid-fight scene.

## Open questions

- Cloud Beast's shipped placement and intended arena are not established by the known disc graph.
- Black Knight part-frame-to-anatomy mapping and the weak-point cylinder centre/axis remain unresolved.
- Gelert's first stream-check scheduler phase relative to the arrival callback remains unresolved.
- The physical routes formed by the Plague Serpent's activated platforms remain unresolved.
- Camera art, exact presentation timing at thread boundaries, and some unnamed script flags are left to the individual encounter specifications where not established.


