# SPEC-0064: Common enemy build guides

- Answers: proactive (weapon box); proactive (common enemy AI); proactive (open close-out); R-0124, R-0127; proactive (patrol paths); proactive (combat core pass 3); proactive (timing pass); proactive (enemy reach pass); proactive (reach pass 2); proactive (movement pass); proactive (enemy sweep); R-0146 (Park Shrine Drackonacks are undamageable); R-0149 (player's answer per ranged kind), R-0156 (when the Farm Road Thieves and Bandits load)
- Status: Draft
- Disc build: SCUS-97367 v1.00 (same fingerprint as SPEC-0001)
- Confidence: High for the row values, animation tables, clip gate ticks, attack arithmetic and the notice, wake, hidden and alarm rules. Medium-High for the idle states (wander, patrol, sentry), the tick offsets of the sequencer and the grab machine. Medium for weapon contact geometry, hurt-shape geometry and the places a section marks Medium. Each kind states its own confidence.

## Summary

The game has 49 kinds of ordinary enemy that fight the hero and are not bosses. All of them are behaviour class 17, the common enemy controller of SPEC-0038: they share the think loop, the pre-step (notice, alarm, hit hooks), the agro machine, the attack choosers, the attack sequencer, the attack manager and the death hook. They differ by their monster row (SPEC-0029) and by their own animation table, which is what each kind's guide records: the row numbers, the animation-id-to-clip map with gate ticks, the attack slots with their bands, rolls, damage and clips, the movement speeds, the defence and counter rolls, a worked timeline, the mistakes a port is most likely to make, and the open questions.

Where the kinds are covered:

- This spec and SPEC-0065 together: nine batches covering 41 kinds, in the order Minion Corps A (Grunt, Archer, Brute, Warrior), Shadowglen Plants (Flower Monster, Poison Flower Monster, Spore Monster, Plant Gunner, Plant Monster), Road and Ridge Brigands (Thief, Bandit, Thug, Drackonack, Giant Drackonack, plus the field Rogue), Skeleton Legion (Dumb Skeleton, Skeleton Fighter, Skeleton Archer, Skeleton Wizard), Minion Corps B (Wizard, Magician, Artillery, Dark Knight), Bog and Sewer Creatures (Swamp Tentacle, Tentacle, Slorg, Giant Slorg), Flyers (Biting and Shooting Spellbook, Crokabek, Mountain Crokabek, Swamp Crokabek), Crypt Spirits and Knights (Restless Spirit, Skull Spirit, Spirit Sword, Death Knight) and the Cogham Ixi Force (Ixi Spearman, Ixi Archer, Dark Ixi Spearman, Dark Ixi Archer, plus the Henchman rows in the field).
- Arena fighters: Rogue, Ixi Henchman, Sludgy, Skeleton Fighter and Werelupe Elite as the five rounds of the Meridell arena are in SPEC-0063. The batches here give what is different for the same kinds in the field, where the default state is 5, 6, 7 or 8 instead of the arena start.
- Spiders: Spyder, Poison Spyder and Giant Spyder are in SPEC-0017.
- Boss minions: the Tentacle of the Juppie fight, the Henchmen of the Ixi Chieftain and the Minion Warrior and Grunt summoned by the Darkest Faerie are in the boss specs (SPEC-0014, SPEC-0015, SPEC-0054); the batches describe the same rows where they also stand in the field.
- The Werelupe Basher, Werelupe Bone Thrower and Evil Harris, and the Werelupe Elite in the field, are in the last section, "Werelupe Burrows and Library Oddities", in SPEC-0065.
- The Altador Coliseum rounds (module 1371) are spread over the batches: round 1 (brigands) in the Road and Ridge Brigands section, round 2 (the Dark Ixi group) in the Cogham Ixi Force section, round 3 (Drackonacks) in Road and Ridge Brigands and the skeleton round in the Skeleton Legion section.

Whole-game facts the batches rely on: spawning is proximity streaming (an embedded block holds at most 6 live enemies, 12 in two packs, and releases an enemy when the hero is more than 280 units away); a killed enemy stays dead except the reanimating Skeleton rows; element-variant rows of one kind differ only by the target type and the status or projectile words, never by hit points or clips unless a section says so.

All times are ticks (60 per second) unless stated. "c" is the tick on which an attack is chosen, "s" the tick on which the sequencer requests the strike clip, and "gate" the number of ticks after a clip request at which the clip's progress first reaches 0.99. A clip with the common speed byte 2 advances 0.25 keys per tick at play rate 1.0, so a one-shot clip of F keys passes its gate 4(F - 1) + 1 ticks after the request (clips longer than 24 keys can pass one tick earlier; Timing table below).

This spec holds the roster, the shared field behaviour and the first five batches (Minion Corps A, Shadowglen Plants, Road and Ridge Brigands, Skeleton Legion, Minion Corps B). The other five batches (Bog and Sewer Creatures, Flyers, Crypt Spirits and Knights, Cogham Ixi Force, Werelupe Burrows and Library Oddities, which also holds the Altador Coliseum overview) are in SPEC-0065, which uses the shared field behaviour defined here.

## Roster

HP is per row (two values where rows of one kind differ). "Rows" are the monster rows of resource 530 (SPEC-0029). Section names are the batch headings: Minion Corps A, Shadowglen Plants, Road and Ridge Brigands, Skeleton Legion and Minion Corps B are in this spec; Bog and Sewer Creatures, Flyers, Crypt Spirits and Knights, Cogham Ixi Force and Werelupe Burrows and Library Oddities are in SPEC-0065.

| Kind | Rows | HP | Main areas | Where described |
|---|---|---|---|---|
| Minion Grunt | 183 to 189 | 50 | Meridell Plains, Fyora's Palace, Heroes Rest, Meridell Castle first floor | Minion Corps A |
| Minion Archer | 190 to 196 | 40 | Fyora's Palace, Meridell Village, Heroes Rest, Bogshot Swamp | Minion Corps A |
| Minion Brute | 169 to 174 | 300 | Meridell Village, Arena District, Bazaar District, Park District | Minion Corps A |
| Minion Warrior | 204 to 210 | 100 | Black Keep Grounds, Heroes Rest, Meridell Plains, Illusen's Glade | Minion Corps A |
| Flower Monster | 147, 149 | 23 (melee), 16 (spitter) | Shadowglen Woods, Shadowglen Burrows, Steppe Plateau, Farm Road | Shadowglen Plants |
| Poison Flower Monster | 148 | 25 | Shadowglen Woods | Shadowglen Plants |
| Spore Monster | 221 | 35 | Shadowglen Woods, Farm Road | Shadowglen Plants |
| Plant Gunner | 220 | 35 | Shadowglen Woods | Shadowglen Plants |
| Plant Monster | 219 | 50 | Shadowglen Woods | Shadowglen Plants |
| Thief | 152 | 50 | Bandit Caves, Farm Road, Meridell Plains, Brightvale Coast | Road and Ridge Brigands |
| Rogue | 153 | 80 | Bandit Caves, Farm Road, Western Brightvale | SPEC-0063 round 0; field differences in Road and Ridge Brigands |
| Bandit | 151 | 60 | Bandit Caves, Farm Road, Meridell Plains, Brightvale Coast | Road and Ridge Brigands |
| Thug | 154 | 120 | Bandit Caves, Western Brightvale, Coliseum, Farm Road | Road and Ridge Brigands |
| Drackonack | 144 | 55 | Abandoned Mines, Drackon Ridge, Bandit Caves, Cogham Steppes | Road and Ridge Brigands |
| Giant Drackonack | 145 | 300 | Drackon Ridge, Cogham Steppes, Meriload Mine, Abandoned Mines | Road and Ridge Brigands |
| Dumb Skeleton | 226 | 50 | Oubliette, Haunted Tombs, Meridell Plains, Werelupe Graveyard | Skeleton Legion |
| Skeleton Fighter | 227 | 75 | Crypt of the Fallen, Old Mine, Dungeon Ruins, Haunted Tombs | SPEC-0063 round 3; field variant in Skeleton Legion |
| Skeleton Archer | 224 | 50 | Crypt of the Exiled, Dungeon Ruins, Oubliette, Ancient Castle Spires | Skeleton Legion |
| Skeleton Wizard | 225 | 100 | Old Mine, Ancient Castle Spires, Haunted Tombs, Crypt of the Exiled | Skeleton Legion |
| Minion Wizard | 211 to 216 | 200 | Meridell Village, Heroes Rest, Illusen's Glade, Meridell Plains | Minion Corps B |
| Minion Magician | 197 to 202 | 80 | Market Town, Fyora's Palace, Arena District, Fyora's Palace Gardens | Minion Corps B |
| Minion Artillery | 162 to 167 | 300 | Bazaar District, Meridell Outskirts, Heroes Rest, Steppe Plateau | Minion Corps B |
| Dark Knight | 139 | 150 | Market Town, Black Keep Grounds, Meridell Village, pack 1098 | Minion Corps B |
| Swamp Tentacle | 238 | 75 | Bogshot Swamp, Bogshot Village, Werelupe Burrows | Bog and Sewer Creatures |
| Tentacle | 237 (sewers), 239 (Juppie pack, Shadowglen Woods) | 60, 18 | Meridell Sewers, Ancient Sewers, Secret Sewers | Bog and Sewer Creatures |
| Slorg | 229 | 75 | Meridell Sewers, Ancient Sewers, Black Keep Dungeon, Treasure Vault | Bog and Sewer Creatures |
| Giant Slorg | 230 | 250 | Secret Sewers, Treasure Vault, Black Keep Dungeon | Bog and Sewer Creatures |
| Sludgy | 231 | 80 | Bogshot Swamp, Meridell Treasure Vault, Werelupe Woods | SPEC-0063 round 2 |
| Biting Spellbook | 134 | 35 | Fyora's Library, Rathbone Family Crypts | Flyers |
| Shooting Spellbook | 135 | 35 | Fyora's Library | Flyers |
| Crokabek | 136 | 25 | Meridell Plains, Meridell Castle Courtyard, Drackon Ridge | Flyers |
| Mountain Crokabek | 137 | 55 | Cogham Steppes, Brightvale Outskirts, Steppe Plateau | Flyers |
| Swamp Crokabek | 138 | 35 | Bogshot Swamp, Werelupe Woods | Flyers |
| Restless Spirit | 222 | 75 | Ancient Castle Spires, Dungeon Ruins, Black Keep Dungeon | Crypt Spirits and Knights |
| Skull Spirit | 228 | 300 | Ruined Caves, Ancient Castle Spires, Crypt of the Sentries | Crypt Spirits and Knights |
| Spirit Sword | 140, 141 | 50, 65 | Knights Crypts, Crypt of the Sentries, Ancient Castle Spires | Crypt Spirits and Knights |
| Death Knight | 143 | 200 | Knights Crypts, Ancient Castle Spires, Crypt of the Fallen | Crypt Spirits and Knights |
| Ixi Spearman | 159 | 50 | Cogham Steppes, Cogham Village | Cogham Ixi Force |
| Ixi Archer | 156 | 65 | Cogham Steppes, Abandoned Mines, Cogham Village | Cogham Ixi Force |
| Ixi Henchman | 157, 160 | 65 | Cogham Steppes, Meridell Village | SPEC-0063 round 1; field notes in Cogham Ixi Force |
| Dark Ixi Spearman | 161 | 170 | Cogham Steppes, Abandoned Mines, Coliseum | Cogham Ixi Force |
| Dark Ixi Archer | 158 | 200 | Abandoned Mines, Cogham Steppes, Coliseum | Cogham Ixi Force |
| Spyder | 234 | 25 | Mountain Passage, Ruined Caves, Root Cellar | SPEC-0017 |
| Poison Spyder | 235 | 16 | Ruined Caves, Mountain Passage, Drackon Ridge | SPEC-0017 |
| Giant Spyder | 236 | 150 | Mountain Passage, Ruined Caves, Abandoned Mines | SPEC-0017 |
| Werelupe Basher | 240 | 75 | Illusen's Glade, Werelupe Burrows, Werelupe Woods | Werelupe Burrows and Library Oddities (SPEC-0065) |
| Werelupe Bone Thrower | 241 | 85 | Illusen's Glade, Werelupe Burrows | Werelupe Burrows and Library Oddities (SPEC-0065) |
| Werelupe Elite | 242 | 110 | Werelupe Burrows Lower | SPEC-0063 round 4; field notes in Werelupe Burrows and Library Oddities (SPEC-0065) |
| Evil Harris | 150 | 75 | Park Shrine, Fyora's Library | Werelupe Burrows and Library Oddities (SPEC-0065) |

## Shared field behaviour

This is the one description of the rules every batch reuses. The base machine (think loop, setter, pre-step, agro order, sequencer) is SPEC-0038; this section gives the field-facing rules in plain terms so that a kind's section only has to state its own numbers. Where a section says "the shared rules" it means this section.

### Start states and what a spawn record supplies

The spawn record (SPEC-0006, kind 3) carries a default-state word, an alarm group byte (255 means none), a drop ordinal (-1 means none), a summon key, and a block of type-dependent words:

| Default state | Words read | Meaning |
|---|---|---|
| 0 | none | Stand idle for ever. Wakes only by a hit or by the alarm group. No notice check |
| 5 wander | float R, pause range (two integers, either order), pause percent | Wander square of side R about home |
| 6 patrol | path index, loop flag (0 loop, else back and forth), point pause range, random-stop percent, random-stop range | Walk the entry records of an area path, pausing at every point |
| 7 sentry or guard | one flag word | 0: stand at home and look around. Nonzero: hold the placed facing |
| 8 hide | emerge distance (float), cue word, selector (always 0 on disc) | Wait hidden at home |
| 9 follow | target character id, stay distance, stop flag | Follow a named character (Cogham Village Spearmen, two Dumb Skeletons, Mountain Crokabeks) |

Start of life: default 8 starts state 8 directly; every other word runs the setter with 0. The setter's default case gives state 0 for a row whose Display in Default is 1, and the first pre-step, which the think loop runs in the same step right after this start (SPEC-0038 Behavior 1), enters the default state, so a visible kind is in state 5, 6 or 7 from its first think tick. On a later return (an exit from agro, below) the setter runs inside a handler, so the enemy stands one tick in state 0 and the next tick's pre-step enters the default state. For a row whose Display in Default is 0 the setter gives state 23 instead (see the hidden kinds below). Default 0 stays in state 0. A record bound by a summon key skips all of this and starts in agro. Every exit from agro (leash, lost-sight search, victory pose, or the search of state 2 giving up) runs the setter with 0 and so repeats this entry. The home point is the placed position, rewritten from the position where an enemy is released alive by streaming, except for default 7 which always returns to its disc position.

### Idle states

- **State 5, wander.** Each tick the pause timer counts down while positive. With no goal held, a goal is chosen: if R is below 0.1 the goal is the centroid of a random triangle of the enemy's current navmesh (so it can cross the whole mesh); otherwise it is a point in the square of side R centred on home (x and z each a whole number drawn from 0 to R - 1, minus R / 2), tested against the navmesh, moved halfway toward the enemy and retried up to 10 times, then home. While the pause timer is below 1 the enemy walks to the goal with the wander gait (id 2 on the ground, 98 for flyers). The goal counts as reached when it is within 0.5, or the navmesh blocked flag is set with its counter at 120, or the walker reports no route; then the goal is dropped and a roll int[1, 100] below the pause percent starts a pause: the pause timer and the fidget counter are each drawn from the pause range. A percent of 0 never rests (the enemy walks from goal to goal). While paused the enemy plays the idle pick and, whenever the fidget counter is below 1 and the table has animation 3, plays it once through and redraws the counter.
- **State 6, patrol.** The record's +0xB8 names a path of the area's path list, a run of entry records of the pack (SPEC-0006, Tagged tables, tag 0x0A); the route is set up on the first tick, once in the life of the enemy, and the first target is point 1 (point 0 is first reached only when the path wraps or turns back). Loop mode (+0xBC = 0) goes 1, 2, ..., last, 0, 1, ... and back-and-forth mode (any other value) goes 1, ..., last, last - 1, ..., 0, 1, ...; neither repeats an end point. While the hold timer is below 1 the enemy walks to the current point with the patrol gait (id 4, flyers 99) over the navigation mesh, at the root speed of that clip (1.88 to 11.26 units per second for ground kinds, SPEC-0038 Behavior 49); when its horizontal distance is below 0.5 it has arrived and the hold timer and fidget counter are drawn from the point pause range. The next point is chosen on the following tick; the enemy stands for the draw minus 1 ticks (no stand for a draw of 1 or less), at every point including the middle ones. On the one tick of each awareness cycle on which the awareness countdown is 0, and only while walking, a roll below the random-stop percent stops it for a draw from the random-stop range (a range of 0 to 0 stops nothing). Holding plays the idle pick and, when the table has animation 6 (flyers 142), plays it once through each time the fidget counter has run down. After a notice, an alarm, a chat or a fight the enemy resumes at the point it was heading to (the offset, direction and remaining hold are kept); only a streaming release and rebind restarts the route, from the saved position, toward point 1. An invalid index sets no route: the enemy walks to entry 0 of the pack and rests there for ever (Medium-High). Rows with Display in default 0 patrol unseen and replay the appear animation (81) after each return.
- **State 7, sentry or guard.** Farther than 0.5 from home: walk home. At home with the flag 0: stand with animation 7 (flyers 144); whenever the fidget counter is below 1 play animation 8 (flyers 145) once through, then draw the counter from int[60, 600] and return to 7. The counter is 0 at creation, so a fresh sentry looks first. With the flag nonzero the enemy aims at its stored look point (the placed facing) with its aim turn weight every tick and plays animation 7. A sentry pushed off its post walks back.
- **State 8, hidden at home.** Stands at home playing the hide pick (id 9, or 128 to 131 by the selector), walking home first when more than 0.35 away (a row with the distance-hide flag, the two Tentacles, does not walk home). The wake test runs every tick: the horizontal distance to the hero against the emerge distance times F1, where F1 is 0.7 for a standing hero, 0.75 slow walking, 1.5 walking, 1.75 running and 1.0 in any other action state; a distance of 0 never wakes. Passing sets sub-state 1; the next tick requests the emerge clip (id 10, 165 for flyers) and sets sub-state 2; the enemy enters agro when that clip passes its gate. A table without the emerge clip enters agro at once. Entering agro runs the setter with 19: outer sub-state 0, hold 0, recovery 0, all attack cooldowns 0, and the alarm broadcast.
- **State 9, follow.** Each tick find the character with the named id; if it is gone the wander body of state 5 runs in place (the state stays 9); otherwise adopt its gait number, steer toward it and walk while the horizontal distance is at least the stay distance, and stand in the idle pick inside it. The notice check runs, the alarm check does not include state 9, and socialize does not run.

### Noticing the hero

The notice check runs in states 3, 5, 6, 7, 9 and 2. The awareness countdown reloads with the row's awareness timer and counts down once per tick, so a cycle is the timer plus 2 ticks and the check can pass on one tick per cycle. On that tick roll A, int[1, 100], must be at most the awareness value. Then the row's distance band, scaled at both edges by F1, decides: nearer than the scaled near edge is noticed with no further roll; farther than the scaled far edge is never noticed; between them roll B, int[1, 100], must be strictly below the chance interpolated linearly from the near chance (at the near range) to the far chance (at the far range) over the unscaled band, halved when the hero is outside the view cone and halved again when the hero wears the Tattered Cloak. A blocked sight line replaces F1 by F2 (0.0 to 0.9 by action state) unless the row has the always-sees flag, and the Tattered Cloak makes a blocked line never noticed. The interpolated chance is not truncated: roll B passes when it is strictly below it, so a chance of 66 passes on 65 of 100 draws and a chance of 25.5 passes on 25.

**Stage 1 and stage 2 (visible kinds, Display in Default 1).** A pass in states 5, 6, 7 or 9 calls the setter with 2: it draws the search countdown from the row's search range, stores the hero's present position as the last-seen point and enters state 2. State 2 runs its body on that same tick and on every later tick:

1. If neither the alert clip (id 18; 149 for flyers) nor the lose-interest clip (id 17; 148) is current, request the look clip (id 16; 147). A missing id 16 plays the idle pick.
2. Aim at the last-seen point with the row's aim turn weight. The enemy does not walk.
3. Run the notice check again with fresh rolls on the same tick. It can pass only on a cycle tick, so after the entry tick it is tested once per cycle. A pass requests the alert clip and the enemy enters agro when that clip passes its gate; the alert then runs to its end even if later checks fail. A table without id 18 sends a pass straight to agro on that tick.
4. With no pass the search countdown goes down by 1 each tick, but only while the alert clip is not current. At 0 or below the enemy requests the lose-interest clip and, when it finishes, runs the setter with 0 (back to the default state). A table without id 17 runs the setter with 0 at once. A row with a search countdown of 0 therefore returns to its default state on the entry tick when the repeat test fails. The notice check keeps running during the lose-interest clip, and a pass there still leads to agro.

The first pass is therefore a false start unless a second pass follows within the search window; a hit at any time skips the whole sequence (damage doubled, agro at once).

**Hidden kinds (Display in Default 0): state 21 instead of state 2.** Every Minion Grunt, Archer, Brute, Warrior, Wizard, Magician and Artillery row has Display in Default 0. Such an enemy is **hidden** whenever its class state equals its record's default-state word, and in state 21 until animation 82 is the current animation. Unless the active hero wears the Tattered Cloak or a script has run the invisibility-off command, a hidden enemy is not drawn, not hittable (swings, shots and power objects pass through it, so a hit cannot wake or double it), not auto-targetable, and does not occupy a navmesh triangle for others; it is still simulated and still notices the hero. With the Cloak worn, or after the invisibility-off command (scripted encounters run it together with the agro command, which puts the enemy in state 2 instead of 21), none of this applies. The Cloak affects only this predicate; it does not reveal the cue-word hiding below.

For these rows a notice pass calls the setter with 2, which gives state 21 and stores neither a countdown nor a point: state 21 keeps the values it was created with (countdown -1, last-seen point at the origin). Each tick state 21 runs, in this order:

1. If neither animation 82 nor animation 17 (148) is current, request animation 16 (147).
2. Aim at the stored point with the aim turn weight.
3. Run the notice check again with fresh rolls. If it passes, or animation 82 is current: if 82 is not current and the table has it, request 82 and stay; if 82 is current, wait for its gate and then call the setter with 19 (agro); with no clip 82, go to agro at once.
4. If the check failed and 82 is not current: the stored countdown goes down by 1; at or below 0 (always, on the first tick) request animation 17 (148) and, when it finishes, call the setter with 0 (state 23, then the default state). The notice check repeats every tick meanwhile, so a pass during clip 17 still leads to 82.

The enemy becomes visible and hittable on the first tick of clip 82 and a hit then gives agro with doubled damage. State 23 plays animation 81 through its gate and then runs the setter with the default word; it is not a hidden state, so a freshly bound or returning enemy is visible and hittable while it plays 81. Every start and every return to the default state of a Display-0 row passes through state 23. Default 0 rows of this kind stand in state 0, hidden.

**Cue-word hiding (Display in Default 1, state 8).** A waiting state-8 enemy is hidden while its emerge sub-state is 0, its cue word is 0 and its current animation is the hide pick; the same applies in the first, buried phase of state 14 for the two Tentacles. Hidden by this rule means not drawn, not hittable, not auto-targetable. With cue word 1 the enemy is drawn and hittable while it hides, and a hit wakes it with doubled damage. A hit never wakes a cue-0 enemy that is still hidden. When the cue word is 0 an emerge also starts the ambush camera and sound cue (Medium).

### Alarm groups

A record with a group byte other than 255 broadcasts to its group every time it enters agro (including after a rise or a support cast). Each member checks for a new broadcast on a timer of int[15, 45] ticks. A member in state 0, 3, 5, 6, 7 or 14 goes to state 2 (state 21 for a Display-0 row) and requests the alert id 18 (149) directly; a member in state 8 plays its emerge clip and enters agro whatever its distance from the hero; members in any other state, and members in state 9, do nothing. For a visible kind the alert clip then runs to its gate and the member enters agro with no notice roll. For a Display-0 kind the alert request is replaced on the next tick by the look request, and the member must pass the notice check on its own or it returns to its default state. The group is by byte value and spans every enemy of the area with that byte, whatever its kind. A group number only means something inside its area, and a group of one is inert.

**Alarm with a missing alert clip.** A request of an id the table lacks plays clip 0 of the table's main set while the current-id word still reads the requested id; the state-2 handler waits for that clip's gate with no notice roll and then enters agro. The wait is the gate of whatever clip 0 is for the kind: 57 ticks for the Ixi Spearman (a jab swing), 79 for the Ixi Archer (a hold loop), 59 for the Dark Knight and Spirit Sword (a strafe-forward loop that also slides the knight about 10 units forward). A notice pass with a missing id 18 does not do this; it goes to agro at once.

**Notice gate between groups.** A hidden Display-0 enemy cannot notice the hero while another Display-0 enemy in agro or state 2 belongs to a different alarm group, both bytes being other than 255. Visible kinds are never gated.

### Hits, doubled damage and reactions

A hit on an enemy that is not in agro doubles the damage once, takes agro through the setter with 19, starts hit-reaction kind 0 (animation 57, outer sub-state 10; flyers use kind 6 or 7, below) and turns the enemy to face the attacker. It works from states 0, 2, 5, 6, 7, 8 (when not hidden), 14 and 17, and from the Display-0 states 21 and 23 once the enemy is visible. Hero swings and shots pass through a hidden enemy, so only noticing or an alarm wakes it.

### Socialize

Rows with the socialize flag (row word 0x40 equal to 1; rows 139, 142, 151 to 154, 156 to 161, 204 to 210, 224, 228, 233 to 235 and 240 to 243, which include the Thief, Bandit, Thug, Rogue, the Minion Warrior rows and the six Ixi rows) chat while in states 5, 6 or 7. The quiet countdown (a draw from the row's quiet range) starts at 0, so the first scan happens on the first tick; only the initiator decrements its own quiet countdown. At zero the enemy looks for a peer with a nonzero flag, not in states 1, 2, 3, 19 or 21, whose own quiet countdown is below 1, within the sum of both pair radii and on the same navmesh. Both go to state 3, walk toward each other, stop within 6.0, turn, and play a chat clip (id 14, or 111 to 114 by a draw; the idle pick when the table lacks it) for a talk time drawn from the row's talk range, then return to their default state. While the talk countdown is positive the notice ranges are halved. Drackonacks, Giants and the flyers never socialize.

### Map search and support casts

Rows with the map-search flag run a search every tick in every class state except state 18 itself, outer sub-state 11 (confused), a hidden Display-0 enemy standing in its default state, and any tick on which its own search cooldown is positive. The candidate is picked first (for the coin kinds, the nearest world item of each coin kind 135 to 139 in order), then a roll int[1, 100] below the row's chance is made every tick until it passes; the recovery countdown must be below 1 (types 2 to 8 need it at 0); the cooldown is drawn only on success. The coin kinds (Thief, Rogue, Thug) send the enemy to state 17 to fetch the coin when a route exists and the hero is farther than 10.0. The support types (Skeleton Wizard raising, Minion Wizard and Magician healing and buffing, polymorph, and the others in each kind's section) put the caster in state 18: it requests animation 137, aims at the target, fires its spell once on the first tick that the clip's event bit 0x20 is current, and at the end of the clip runs the setter with 19. That setter clears the recovery countdown and every attack cooldown, restarts the outer sub-state at 0 and re-broadcasts the alarm group, so a caster that cast from a non-agro state enters agro without having seen the hero. A row whose chance and range are both 0 never casts.

### Grab machine

A slot with the grab flag (0x08) runs the grab machine in place of a weapon test, every tick of sequencer state 2 (the strike state). On its choose tick the machine seizes the hero if the enemy's facing dot toward the hero is above 0.5, the hero is within the slot's melee range, the hero's stun word is clear and the straight path is clear. The hero is put in the held base state and the enemy requests the slot's follow-through clip (id 35 + 5k, the C clip) in the same tick, at the strike speed word. The hero stays held where he stands (he is not moved to the enemy's hand) while the enemy stays in the strike state, with a controller rumble repeating every 61 ticks. The throw happens on the first tick the current clip frame (of C, which now plays) carries event bit 0x20: the hero is thrown with the row's throw speed, the strike test reports a hit for that tick (a take-hit of damage 0, plus the steal roll for the Thug), and the sequencer plays the rest of C at rate 1.0; the grab flag skips the recovery clip, so the end path runs on the tick after C's gate. **A grab deals 0 damage on every row**, whatever damage word the slot lists. A hit that reaches the grabber frees the hero at once. If the choose test fails, the machine requests the slot's recovery clip E (id 37 + 5k) on the next tick when the table has it; the strike state waits for that clip's gate and the end path follows on the next tick (no contact; Medium). Per-kind ticks: Minion Brute, Thug and Plant Monster sections.

### Weapon contact and the hero's hurt shape (the general rule)

Every melee strike of every kind in these guides uses the asymmetric weapon-versus-body collision rule in SPEC-0018 (SPEC-0038 Behavior 13, Strike test and Weapon test). On a tick when the strike clip's current frame has the weapon window bit (bit 0x1 for weapon 1, 0x2 for weapon 2, 0x4, 0x8 and 0x10 for weapons 3 to 5), the weapon volume is tested against the hero's previous-tick body pose. Use the weapon's listed radius and full height, the clip's attachment point and frame, and the frame's Z row as its axis; the body side uses Tor's nine small capsules or Roberta's one (radius 0.37 to 0.44; SPEC-0036 Hero body cylinder in the enemy hit tests). The exact decision follows SPEC-0018's near-parallel summed-radius and axial-interval case, or its general finite-cylinder-to-facing-card case. Do not replace this asymmetric test with a symmetric round-cylinder overlap or an oriented-box approximation. No enemy-centre distance, extra facing gate or sight test is part of the weapon contact test. A hero's guard changes only what happens after contact (SPEC-0038 Behavior 13, steps 3 to 5). Each kind's "Weapon reach" table gives distances produced by that rule for its strike clips. A port that poses the attachment frames can reproduce the test; a port that cannot may use the table's "at the tick" range as a fixed band in front of the enemy (a rewrite choice that loses height and sideways limits).

### Weapon reach tables (how the per-kind figures are made)

Confidence: High for the method and for the window ticks; Medium for the generated distance ranges (Tor is idle and level). The contact shape is asymmetric as defined in SPEC-0018; diagonal and moving-target outcomes depend on its facing-card branch and the sampled poses.

Method. For each kind and each attack slot that has a strike clip, the clip is played tick by tick as in the timing table. The strike test of tick s + j uses the window bits of the cursor reached after j passes, and the attachment frames of the pose sampled by the pass of the tick before (one pass old). The first pass runs at the wind-up speed word, later passes at the strike speed word, and second clips (C) at 1.0. The clip's root motion moves the enemy from the next tick on. Each attachment frame is the bone's world matrix, then the pose entry's offset, then its three angle rotations (SPEC-0005 Attach points and carrying); the weapon centre is the frame's translation and its axis is the frame's third row. The enemy's placement matrix is the facing basis and there is no extra model scale; the size of the Giant kinds comes only from the per-joint translation scale of SPEC-0005. The collision decision is the exact asymmetric rule in SPEC-0018, evaluated against the previous-tick hero pose.

Hero shapes. Tor idle, with Tor turned to face the enemy: a stack of small capsules from 0.19 to 3.16 above his feet, 0.89 behind to 0.85 in front of his origin along the facing, and 0.66 to the side on one side and 0.34 on the other. Roberta idle: one capsule from 0.97 to 2.08 above her feet, radius 0.436, within 0.44 of her origin across the facing. Swinging or walking poses move Tor's capsules by up to about 1.0; idle breathing moves them by less than 0.05.

Conventions of every "Weapon reach" table.
- Slot (phase, clip): the attack slot, whether the strike clip is the first one requested on tick s (B) or the second one requested on tick sC after a charge (C), and the clip index.
- Weapon: the weapon number, radius by height.
- Window ticks: the ticks on which the clip's frame carries that weapon's window bit; "level contact" is the part of them on which some position of an idle, level hero straight ahead is touched. The tick on which a hit lands is the one given in the kind's hit and hurt windows text (that text is authoritative); the reach tables only add where the hero must stand.
- Centre forward / up / side: the range of the weapon cylinder's centre in the enemy's own frame (forward along its facing, up from its origin, then across its facing) over the window.
- Tor ahead, at the tick: the range of distances of Tor's origin ahead of the enemy's position on the same tick (negative is behind) over which some window tick touches him.
- Tor ahead, from where the enemy stood at s: the same range measured from the enemy's position on the tick the clip was requested, so it includes the enemy's own travel during the clip.
- Tor feet vs enemy origin: the range of heights of Tor's feet relative to the enemy's origin over which some window tick touches him.
- Each range is the union over all ticks of the row; a single tick touches a narrower band.

Test vectors.
- A weapon of radius 3.0 and height 1.0 with its axis vertical, centred 10 ahead of the enemy and 1 up, touches Roberta's capsule when her origin is 11 or 13.4 ahead and misses at 13.44 (the threshold is 10 + 3.0 + 0.436 = 13.436).
- Minion Grunt, slot 1, weapon 1 (radius 3.0, height 1.0), window s + 16 to s + 27. On ticks s + 16 and s + 17 the weapon centre is about 6.4 and 6.2 above the enemy's origin and no level hero is touched; the first level contact is s + 24 (a level Tor 4.0 to 6.8 ahead). On tick s + 27 the centre is about 1.6 ahead and 0.1 up, and a level Tor whose origin is 0.3 to 4.0 ahead of the Grunt is touched.

### Timing table and units (every common enemy; timing pass)

Confidence: High for the units, the counters and the gate arithmetic; Medium for first-touch ticks except where an exact fixed-pose sample is stated. Those samples hold Tor idle, level and straight ahead; they do not establish moving-target timelines.

**Units.** One tick is one pass of the main loop, 1/60 s (SPEC-0007). Every counter in these guides is a whole number of ticks that the enemy's think lowers by one per think tick: slot cooldowns, the recovery countdown, the defence hold, the strafe hold and lock, the awareness cycle, the alarm countdown, the stun countdown of a blocked charge, the loop-strike countdown, the death disappear countdown and the reanimation countdown. A range written int[a, b] is drawn with both ends included. Row fields written in seconds are already ticks on the disc except the status durations, which are drawn in ticks and then divided by 60 and truncated to whole seconds (SPEC-0038 Behavior 13). Clip time is in keys: at play rate R a clip of speed byte S advances R x 0.5 / S keys per tick, so speed byte 2 at R = 1 is 4 ticks per key; a window on keys a to b is read by the strike test on ticks s + 4a to s + 4b + 3 (s = the request tick), and a one-shot clip of F keys passes its 0.99 gate on s + 4(F - 1) + 1 for F of 24 keys or fewer (SPEC-0005, Clip progress; longer clips can pass one tick earlier). The rate words of a slot are stored before the same tick's re-time: the pass of the request tick of B runs at the wind-up word, the passes of state 2 (including the pass of the contact tick) at the strike word, and the passes of state 3 and later at 1.0.

**What does not count down.** Nothing of an enemy counts while the world is frozen (pause, dialogue boxes and the other code-9 screens), while its zone is inactive, or while its base state is one in which the think is not stepped (16 dead, 25, 33 script-held, 36, 42, 53, 55). The four slot cooldowns also stop on every tick on which the hero is not attackable (hurt, knocked down, held, talking and the other busy states of SPEC-0036); the recovery countdown and the defence hold keep counting on those ticks.

**Phases of one melee attack** (choice tick c; s = the tick the strike clip B is requested):

| Phase | Ticks | Notes |
| --- | --- | --- |
| Choice | c | Normal choice runs only while the hero is attackable, the recovery countdown is below 1 and the outer sub-state is 1, 3, 4 or 11; the cooldown is drawn here and again at the end path |
| Approach (sequencer state 0) | c to s - 1 | Walk at gait 0 until d is at most the slot's melee range, then state 1 on the next tick; the chase stop drops the step within the sum of the radii (enemy radius + 1.175 for either hero) |
| Wind-up A | s_A to gate | Skipped in the same tick when the kind has no A clip (most kinds) |
| Strike B, window | from the first window tick | The strike test runs every tick of state 2 |
| Hit | first window tick on which a weapon overlaps a hero capsule | Not the window start; one hit per window (hit list) |
| Follow-through C or B to its gate | after the hit | A window that reopens in the same clip re-enters state 2 one tick after it reopens |
| Recovery E, end path | gate of the last present phase, + 1 | Recovery countdown drawn; cooldown redrawn |
| Next choice | end path + recovery draw, at the earliest | And only while the hero is attackable |

**Window start against first touch** (the hit tick for an idle, level Tor straight in front, at the start distances the approach leaves; counted from the request tick of the clip named):

| Kind and attack | Window opens | First touch (hit tick) |
| --- | --- | --- |
| Minion Grunt slot 1 / slot 2 | s + 16 / s + 32 | Row 184 exact chase-stop samples: slot 1 s+26 at d=4.0–5.0 and s+24 at d=6.0–8.5; slot 2 s+38 at d=2.0–6.5, s+36 at d=7.0–9.0. Other rows and distances are not represented by these samples. |
| Dumb Skeleton and Skeleton Fighter slot 0 / slot 1 | s + 36 / s + 28 | s + 40 / s + 29 |
| Skeleton Fighter lunge (slot 2) | s + 29 | exact stationary samples, fixed level Tor straight ahead, with the chase stop: s+34 at 1.5; s+33 at 2.0–2.5; s+32 at 3.0; s+33 at 3.5–8.0; s+34 at 8.5; s+35 at 9.0–9.75 (no contact at 1.0). Without the stop: no contact at 1.0–6.0; s+33 at 6.5; s+32 at 7.0; s+33 at 7.5–8.0; s+34 at 8.5; s+35 at 9.0–9.75 |
| Dark Knight slot 0 / slot 1 / slot 2 / slot 3 | B + 6 / s + 48 / s + 16 / s + 20 | row 139 fixed-pose level samples: slot 0 with the chase stop (4.175): B+10 at d=3, B+8 at 3.5–4, B+7 at 4.5–10.5, B+10 at 11, no touch at 1–2.5 and 11.5–13 (without the stop: B+8 at 5, B+7 at 5.5–10.5, B+10 at 11, no touch at 1–4.5 and 11.5–13); slot 1 s+48 at d=1–2.5, s+49 at 3–9, s+50 at 9.5–12, s+49 at 12.5, s+50 at 13 (without the stop: s+48 at 2.5–6.5, s+49 at 7–9, s+50 at 9.5–12); slot 2 without the stop at d=4/6/8 touches s+16/+17/+19 (none at 1–2.5), with the stop s+16 at 1–2, s+17 at 2.5, s+18 at 3–3.5, s+19 at 4–8.5; slot 3 at d=2–4 touches s+20 |
| Rogue slot 3 (three windows) / slot 0 / slot 2 bite | s + 16, 40, 60 / s + 20 / c0 + 36 | s + 17, 41, 61 / row 153 exact chase-stop slot-0 samples: s+26 at d=3.0–4.5, s+27 at 5.0 / c0 + 38 |
| Spyder and Poison Spyder swing; Giant Spyder swing | s + 17 | s + 17 (hero up to 2.8 ahead; s + 18 from 2.9, s + 19 at 3.5, s + 21 at 4.0) |
| Werelupe Basher and Elite jab / Elite three-hit swing | s + 20 / s + 28, 52, 76 | s + 20 / first window row 242 exact no-stop samples: s+28 at start d=3.0–4.5, s+29 at 5.0; later hit ticks depend on the changing relative pose |
| Sludgy swipe / slam | s + 32 / s + 19 | s + 36 / s + 21 to 22 |
| Evil Harris slot 0 / slot 1 | s + 28 / s + 32 | s + 28 to 30 / s + 35 |
| Ixi Spearman and Henchman jab / lunge | s + 20 / c8 + 24 | no touch on a still level hero straight ahead at the 3.175-unit chase stop / c8 + 28 |
| Minion Brute slot 0 (rate 1.5) | s + 40 | s + 44 (5.0 to 12.5 units with the chase stop; s + 45 at 4.5 and at 13.0, and without the stop also at 6.5) |
| Flower Monster and Poison Flower claw / slam | s + 32 / s + 32 | s + 32 (7 or nearer), s + 36 (10) / s + 32 (8.5 or nearer), s + 33 (9), s + 34 (9.5 to 10) |
| Thief slot 0 / slot 2 bite | s + 20 / sC + 16 | s + 26 / sC + 16 |
| Bandit slot 0 / slot 1 / slot 2; Thug slot 0 / slot 2 | s + 28 / s + 23 / b + 4; s + 28 / b + 4 | s + 28 / s + 23 / b + 4 to b + 7 by distance (Bandit: b + 4 up to 4.0, b + 5 at 4.5, b + 7 at 5.0, b + 6 at 5.5, b + 7 from 6.0 to 9.0); Thug s + 29 at 4.0 to 6.0 and at 7.0, s + 28 at 6.5 and from 7.5 (s + 30 at 3.5) / b + 4 at 2.0 to 4.0, b + 5 at 4.5, b + 9 at 5.0 to 5.5, b + 8 at 6.0 to 10.5, b + 7 at 11.0 to 11.5 |
| Drackonack bite; Giant Drackonack bite | c0 + 4 | c0 + 4 (to 7.0 ahead; c0 + 5 at 7.5, c0 + 6 at 8.0); c0 + 4 at 2.5 to 4.5 (c0 + 6 at 5.0 to 6.0, c0 + 7 at 6.5 to 7.0; c0 + 6 at 1.5 and no touch at 2.0) |
| Minion Artillery bite | b + 4 | b + 4 (6 units) to b + 10 (10 units) |
| Minion Warrior slot 0 / slot 2 / charge C | s + 28 / s + 16 / sC + 24 | s + 28 (to 6.0 ahead; s + 29 at 6.5 to 8.0, s + 30 at 8.5 to 9.0, s + 31 at 9.5 to 10.0, s + 32 at 10.5) / s + 16 (to 4.0 ahead; s + 17 to s + 19 from 4.5 to 7.5) / sC + 24 (to 7.5 ahead; sC + 25 at 8.0, sC + 26 at 8.5 to 9.0) |

**Being hit.** There is no invulnerability timer on a common enemy: a countdown of 90 ticks is set by every hit but nothing reads it. A hit restarts the reaction clip (force start), so a reaction ends one gate length after the last hit: kind 0 (id 57) and kind 6 (id 152) run to their clip's gate (for example 37 ticks for a 10-key clip, 49 for 13 keys, 65 for 17 keys). The only times an enemy cannot be hit are its dodges (outer 5 and 6), the dead and rising states, the hidden kinds while hidden, a ghost in its ghost phase, a script hold, and a flyer while ids 74 to 76 of hit-reaction kind 7 are current. A hit cancels a running attack without the end path, so no recovery countdown is drawn and the slot cooldown keeps the value drawn at the choice.

**Death.** The death clip starts on the death tick and plays once; the disappear countdown (row value, ticks) starts on the first think tick on which the clip's progress is at least 0.95 and falls by one per tick; the corpse sinks 1/180 unit per tick once 30 ticks of the countdown have run (flyers do not sink); it is gone on the tick the countdown falls below 0. Reanimating skeletons never run this countdown.

**Worked exchange (Minion Grunt row 184 against Tor holding a weapon, both at full health, hero idle 12 units away; c = the choice tick).**

| Tick | What happens |
| --- | --- |
| c | Slot 1 chosen (49 percent per tick); cooldown drawn; sequencer state 0 |
| c + 1 to c + 31 | The Grunt flies at 0.2333 per tick; the step is dropped within 3.175 of Tor |
| s = c + 32 | State 0 sees d at most 5.0 on the tick before; B (clip 10) requested; the pass of s runs at 1.3 |
| s + 16 | Window opens; the weapon does not reach Tor yet |
| s + 24 | First touching tick: 25 damage (before armour and shield) and the status roll, inside the Grunt's think (item 6 of the tick). Tor is in the hurt state when his own update runs on s + 24, and his flinch clip (36 ticks with a weapon) starts in his animation pass of s + 24 |
| s + 25 to s + 61 | The Grunt's think (which runs before Tor's update in every tick) sees Tor hurt: its slot cooldowns stop counting and no new attack can be chosen; its recovery countdown still counts |
| s + 37 | B passes its gate (the passes after the hit run at 1.0) |
| s + 38 | End path: recovery int[15, 60] drawn, slot 1 cooldown redrawn (int[120, 240]) |
| s + 61 | Tor's flinch done test passes in his own update; he is back in state 1 |
| s + 62 | The Grunt sees Tor attackable again; its cooldowns resume |
| h (at least s + 61) | Tor presses a quick swing; the Grunt's melee defence sees it on h + 1 (the enemy thinks before the hero) and rolls a dodge at 49 of 100 |
| h + 8 to h + 23 | Tor's swing window (sub-state 2): if the Grunt did not dodge and is in reach, the first touching tick deals Tor's damage; the Grunt plays reaction 152 from that tick and leaves it 37 ticks after Tor's last hit |

**What a port most likely gets wrong (timing).**
1. Landing the hit on the first window tick. The hit tick is the first window tick on which the weapon touches the hero, often 4 to 8 ticks later (table above).
2. Counting cooldowns in seconds, or letting them run while the hero is hurt or busy. They are ticks and they freeze on every tick the hero is not attackable.
3. Playing the whole strike clip at the strike word after a hit. The rate falls to 1.0 from the tick after the contact, which lengthens the swing (the Grunt's slot-1 end path comes on s + 38 to s + 41 after a hit and on s + 36 after a miss).
4. Giving enemies invulnerability frames after a hit, or not restarting the reaction clip on every hit.
5. Using 30 Hz frames or 50 Hz: one key of a speed-byte-2 clip is 4 ticks at 60 Hz.
6. Starting the disappear countdown at the death tick instead of at the 0.95 point of the death clip.
7. Drawing the slot cooldown only once; it is drawn at the choice and again at the end path.

### Attack manager

All class-17 enemies in agro share one attack-manager list (SPEC-0038 Behavior 45). Each kind has a weight (its difficulty value); the starting gear has capacity 0, so only the first listed attacker may start an attack, and every later enemy that would start one waits in line (outer sub-state 12) when the weights of the entries before it sum to more than the capacity, unless its approach routine is 3 (keep distance, e.g. the Minion Archer, which never waits). A waiting enemy walks to a stand-off point drawn in a square of side equal to the row's stand-off value around the hero, faces the hero, and redraws the point every int[600, 800] ticks. Capacity 15 allows two weight-8 attackers and 39 allows five. A waiting enemy that rolls a counter is put back in the wait state, because the sequencer start is refused while it waits.

### Agro: idle pick, defence and counters

- **Agro idle pick.** In agro the idle pick (set-up on outer sub-state 0, and the idle animation of stand-still sub-states) is id 141 when the enemy is flying or its row has Can Fly 2, and id 1 otherwise. Outside agro the idle pick is id 0 (97 while a Can Fly 1 flyer is in the air). A table with no entry for the id plays clip 0 of its main set, which for some kinds is not an idle clip; each kind's animation table says which.
- **Defence is rolled once per swing.** The melee defence roll is latched per hero swing: it runs only when the defence hold is below 1, the enemy is not in a hit reaction and not mid-attack (except sequencer states 0, 1 and 5), and the hero is within 8.0 and in front. The first roll, int[1, 100] at or below the block chance for the swing class (quick, normal, heavy), gives a block; otherwise a second roll below the dodge chance gives a dodge. The defence timer then holds off further defence for its drawn range. Missile defence is rolled per shot meeting the capsule within the row's look-ahead.
- **Counters are rolled every tick of a hit reaction.** While the enemy is in a hit reaction, the counter choice rolls every tick for each slot whose trigger is "own hit reaction" (trigger 6, with that slot's chance), so a counter is near certain within the reaction's length. Other triggers: 1 and 2 (after a dodge or block), 5 (sequencer blocked while the hero holds L1).
- **Per-tick order in agro** (SPEC-0038 Behavior 44): recovery countdown, block-state fix, missile defence, melee defence, lost-sight test, normal choice (slots tested from the highest down, first pass wins), counter choice, distance-hide gate (the two Tentacles), outer switch, agro tail (route flag and leash). The normal choice runs only while the hero is attackable, the recovery countdown is below 1 and the outer sub-state is 1, 3, 4 or 11.

### Losing the hero, death and drops

Agro exits to the default state, through the setter with 0, by the leash (no route to the hero for 181 consecutive ticks while the walk waypoint is at least 15.0 from the hero), the lost-sight search (outer sub-state 9: the sight line is blocked, the enemy walks to the last-seen point and gives up after the search counter passes 120) and the victory pose after a killing blow (id 73, 163 for flyers). There is no distance give-up: an enemy with a route follows the hero indefinitely.

Death runs the death hook (SPEC-0038 Behavior 34 and 36): base state 16, death animation 58 (153 for a flyer that is not grounded), payout of the placement's drop ordinal, then the disappear countdown starts when the clip passes progress 0.95 and the corpse is released when it passes below 0. A killed enemy stays dead (SPEC-0038 Behavior 47) except the reanimating Skeleton rows. Only rows that steal carry Neopoints; most rows steal nothing and carry 0.

### Flyers

Rows with Can Fly 1 fly while their ground countdown is not positive; rows with Can Fly 2 always fly. Flyers pick only flying-kind navmeshes (ground rows only ground-kind ones), and the goal point is set to the enemy's own position when the mesh index first becomes valid. The goal height is set whenever a walk target is set, and it differs by Can Fly value. Can Fly 1 (Minion Grunt, Minion Archer, Crokabeks): the hero's height plus the smaller of the row's maximum fly height and 0.2 times the horizontal distance to the hero, raised to at least 2.0 above the ground under the goal point, and the hero's height plus 2.0 in sequencer states 7, 8, 9 and 12. Can Fly 2 (Minion Wizard, Minion Magician): the hero's own height, lowered to the ground under the goal plus the maximum fly height when it is higher than that, with no 0.2 term and no lower bound, so a Can Fly 2 enemy flies level with the hero's feet, not above the hero. Both use the home point's height while the enemy waits in state 8. The height moves at most 0.1 per tick and skips dead or parked characters; a hold sub-state keeps the last goal. Hit-reaction kind 7 (a Can Fly 1 flyer with flying-hit value 1) grounds the flyer for 300 to 900 ticks: the wait is on the ground gap (height above ground) at most 2.0, not on speed, and when the countdown ends the flyer relaunches (state 22) and returns to its default state; a missing clip 76 plays clip 0. The airborne attack-slot test is a Can Fly 1 test only: a trigger special of 1 (airborne only) passes only while a Can Fly 1 flyer is in the air, special 2 (grounded only) passes while its ground countdown is positive, and a Can Fly 2 row never passes special 1 and always passes special 2. A flying agro or idle velocity of the row is added along the class axis of the gait (forward, back or side) on top of the clip's own root travel, only while the gait clip of the current class is the one playing (SPEC-0038 Behavior 51 item 4).

### Movement tables (how to read the per-kind tables; movement pass)

Every kind below has a "Movement table". The shared rules behind it (root travel only, no walk or run switch, turning weights, circling direction, one extra step after a stop, flinch slides, corpse slides, giants, ledges) are in SPEC-0038 Behavior 51; read that first. Columns:
- Mode: the state or sub-state (wander = state 5 and the walk home of states 7 and 8; patrol = state 6; approach, hold, back off, retreat and circle = the agro archetype inners of SPEC-0038 Behavior 44; flinch = hit reaction; corpse = the held death clip).
- Source: "root" (clip root travel), "root + row" (flyers: clip root plus 2 x the row's flying velocity) or "none".
- Clip: the action id, then its key count; every clip has speed byte 2 (0.25 key per tick at rate 1) unless the row says S1 (0.5 key per tick).
- u/tick and u/s: the mean travel at rate 1 on flat ground, 60 ticks per second; a slow status halves both, a speed status doubles both. One-shots give the distance over the clip and its tick count instead.
- Turn: the facing rule in that mode ("steer" = the navigation blend 0.3 or 0.1; "aim w" = blend w toward the hero per tick; "snap" = 1.0).
- The patrol leg arithmetic and test vectors are in SPEC-0038 Behavior 49; the chase and flinch worked examples in SPEC-0038 Behavior 51.

Port mistakes shared by every kind: using a speed constant or a navmesh speed instead of the clip root; scaling travel by the giant size; speeding the walk clip up to "run" when the hero is far; turning on the spot; stopping exactly at a threshold (the original takes one more step); pushing the enemy back on a hit with a velocity (the slide is the reaction clip's root, straight away from the attacker); freezing corpses (a death clip that ends with root keeps sliding); circling toward the steering point (the side clip carries the enemy the other way).

### Missing animation ids

A requested id that the kind's table lacks is not an error. By category: a missing look id 16 plays the idle pick; a missing lose-interest id 17 runs the setter with 0 at once; a missing alert id 18 sends a notice pass straight to agro, and an alarm request plays clip 0 and waits for its gate (above); a missing emerge id plays nothing and the enemy enters agro at once; a missing attack phase id (wind-up, follow-through, recovery or blocked clip) is skipped in the same tick; any other missing id plays clip 0 of the main set, whose root travel then moves the enemy (the Poison Flower Monster's and the melee Flower Monster's clip 0 are walking clips, so their idle, hide and hold creep forward at 2.97 units per second; the Plant Monster's, Spore Monster's and Plant Gunner's clip 0 has no root travel). The detail for each kind is in its animation table.

### The batch sections

Five batch sections follow in roster order (the other four are in SPEC-0065). Each starts with the rules specific to the batch (placements and default-state counts, record values on disc, notice numbers, hidden or visible, animation tables) and then gives one build guide per kind. Where a batch section repeats a number of this section (a notice cycle, an emerge gate), the batch section's figure is the value for its own rows.


## Minion Corps A: rank and file

Four kinds make up the rank and file of the Darkest Faerie's minions: the Minion Grunt (monster type 80, variants 0 to 6, rows 183 to 189), the Minion Archer (type 80, variants 7 to 13, rows 190 to 196), the Minion Brute (type 78, variants 0 to 5, rows 169 to 174) and the Minion Warrior (type 82, variants 0 to 6, rows 204 to 210). All are class-17 common enemies and follow SPEC-0038 exactly (think loop, pre-step, attack choosers, sequencer, attack manager). Within a kind the variant changes the element (target type T), the status the hero suffers and a few timers, never the hit points or the clips. The Warrior built by the Darkest Faerie's bolt (row 204, key 2864) is specified in SPEC-0054; this section gives its field behaviour and the deltas of rows 205 to 210. Confidence: High for the row values, clips, attack arithmetic and the hidden rule; Medium-High for the start-state behaviours; Medium for the grab-and-throw of the Brute and for weapon contact geometry. All times are ticks (60 per second); "s" is the tick on which the sequencer requests the strike clip (the choice was one tick earlier), and every clip of these kinds has speed byte 2 (0.25 keys per tick at play rate 1.0, so a one-shot clip of F keys passes its 0.99 gate 4(F - 1) + 1 ticks after the request).

### Shared rules for the four minion kinds

#### What they have in common

- **Invisible kinds.** Every row of the four kinds has Display in default 0. This single field decides how they behave in the field (next heading): they are not drawn, not hittable and not auto-targetable while they stand in their default state, they use state 21 instead of state 2 when they notice the hero, and they decloak with animation 82.
- **Weights in the attack manager** (SPEC-0038 Behavior 45): Grunt 8, Archer 8, Brute 12, Warrior 12. With the starting gear (capacity 0) only the first listed attacker of any group may start an attack; every other enemy that would start one waits in line (outer 12) unless its approach routine is 3 (the Archer never waits).
- **Rows differ by element only** within a kind, with the named exceptions in each kind's table. The hero status each element applies: T 1 fire (status 7), T 2 poison (2), T 3 confusion (6), T 4 stun (0), T 5 slow (3), T 6 curse (1), T 7 none (Warrior row 210 only); the status roll is int[0, 100] strictly below the row chance (SPEC-0038 Behavior 13, Strike test), so a chance of 50 succeeds on 50 of 101 draws and 100 on 100 of 101; for missiles the compare includes the threshold (SPEC-0029 Behavior 9).
- **Hurt shapes** (body capsules of the model, SPEC-0040 Behavior 12A): Grunt and Archer: bone 0 length 2.95 radius 1.0 plus eleven small limb capsules (radius 0.15 to 0.51); Brute: seven capsules of length 2.25 with radii 2.62, 2.58, 1.56, 0.75, 1.1, 0.75, 1.1; Warrior: bone 0 length 1.02 radius 0.64 and bone 4 length 1.8 radius 1.01. Row radius and height are used only for the chase stop and spacing. Confidence Medium.
- **Drops.** The death hook pays the placement's drop ordinal (record offset 0xB0; -1 on 127 of 159 Grunt, 76 of 153 Archer, 39 of 58 Brute and 54 of 100 Warrior records in play) with the rules of SPEC-0038 Behavior 34; none of the four rows steals, so carried Neopoints are always 0.
- **Death.** Death hook, then base state 16 and the death clip once (id 58); the disappear countdown starts when the clip reaches progress 0.95 and the corpse is gone when it passes below 0 (Grunt and Archer timer 1, Warrior 4, Brute 6). Death clips: Grunt and Archer clip 30 (10 keys, gate s+37, 0.95 at s+36), Brute clip 11 (61 keys, gate s+240, 0.95 at s+231), Warrior clip 17 (27 keys, gate s+104, 0.95 at s+100). A Grunt or Archer is a can-fly-1 flyer, which the death hook grounds, so it plays 58 (not 153) and sinks like a ground corpse (SPEC-0038 Behavior 36).

#### The hidden rule: how an invisible kind lives in the field

An enemy of these rows is **hidden** (SPEC-0029 Behavior 2b, hidden test) whenever its class state equals the default-state word of its spawn record, or its class state is 21 and animation 82 is not the current animation. Unless the active hero wears the Tattered Cloak (item 126, SPEC-0021) or a level script has run the invisibility-off command on it, a hidden enemy:

1. is **not drawn** (the draw routine skips hidden characters), so the player cannot see it;
2. is **not hittable**: hero swings, hero shots and power attacks pass through it, so a hit cannot wake it and cannot be doubled;
3. is **not auto-targetable** and does not occupy a navmesh triangle for the navmesh pick of other characters.

It is still simulated and it still notices the hero. It stops being hidden when it leaves its default state or state 21 (to state 19, 2, 23, 10, 3 and so on) or when animation 82 starts. Examples with a default-state word of 5: while it wanders in state 5 it is hidden; in state 23 (the 81 clip, below) it is visible; in state 21 it is hidden until animation 82 plays and visible from then on.

With the Tattered Cloak worn by the active hero the whole rule is off for that hero: the enemies are drawn and hittable in every state. The cloak also affects noticing: a hero wearing it is not noticed when the sight line is blocked, and every clear-sight chance is halved (SPEC-0038 Behavior 3). Scripted encounters in Illusen's Glade, Western Brightvale, the Treasury and a few other places run the invisibility-off command together with the agro command on named entities; those enemies are never hidden and enter state 2 (the search animation, then agro) instead of state 21.

#### Where they stand: default states and counts

Counts are placements the player can meet (confirmed or plausibly loaded), without the CUT blocks; summon-reserved records are listed separately because they are created by a summoner and start in agro by the summon rules (SPEC-0029 Behavior 31).

| Kind | State 0 | State 5 (wander) | State 6 (patrol) | State 7 (sentry) | State 8 (hide, emerge) | Summon-reserved (key) |
|---|---|---|---|---|---|---|
| Grunt | 4 | 50 | 0 | 17 | 28 | 60 (2844 and the variant keys) |
| Archer | 0 | 61 | 0 | 33 | 53 | 6 (2807) |
| Brute | 0 | 6 | 2 | 11 | 39 | 0 |
| Warrior | 1 | 29 | 11 | 21 | 22 | 16 (2864) |

Main places (default state counts): Grunt: Fyora's Palace (5: 11, 8: 6), Meridell Castle first floor, Meridell Plains (8: 5, 5: 4), Heroes Rest (7: 10), Fyora's Palace Gardens (5: 9), Drackon Ridge, Fyora's Library (8: 5), Farm Road (8: 4). Archer: Fyora's Palace (5: 12, 8: 7, 7: 4), Meridell Village (5: 18, 8: 4), Bogshot Swamp (8: 14), Heroes Rest (7: 13), Park District (8: 10), Treasury (7: 8), Fyora's Palace Gardens (5: 7). Brute: Meridell Village (8: 11), Arena District (8: 9), Bazaar District (8: 6), Park District (8: 4, 7: 2), Coliseum regions (8: 5), Drackon Ridge (5: 2, 7: 3), Fyora's Library (6: 2). Warrior: Illusen's Glade (6: 10, 5: 4), Black Keep Grounds (8: 7, 7: 5, 5: 1), Heroes Rest (7: 9), Meridell Plains (8: 3, 5: 4, 6: 1), Fyora's Palace and Gardens (5: 12), Coliseum regions (8: 4). Alarm group byte (record offset 0xB7): 255 (none) on 145 of 159 Grunt, 85 of 153 Archer, 31 of 58 Brute and 79 of 100 Warrior records; the rest use groups 1 to 10.

#### Spawn record values that drive the field states

All are in the character's spawn record (SPEC-0006, kind 3). Words read as floats or integers depending on the state:

| State | Record values | Meaning |
|---|---|---|
| 5 | float at +0xBC (R); integers at +0xC0 and +0xC4 (pause range, ticks); integer at +0xC8 (pause percent) | Wander square side R. Pause range and chance below |
| 6 | s32 at +0xB8 (path index; the path is a run of entry records of the pack); integer at +0xBC (0 loop, nonzero back and forth); integers +0xC4, +0xC8 (pause range at each point, either order); +0xCC (percent), +0xD0, +0xD4 (random-stop range while walking) | Patrol of an entry range (SPEC-0038 Behavior 49) |
| 7 | integer at +0xC4 (0 or 1) | 0: fidget at home; 1: keep facing the placed direction |
| 8 | float at +0xC0 (emerge distance); integer +0xC8 (emerge selector, always 0 here); +0xC4 (camera cue flag) | Wake rule of SPEC-0038 Behavior 40 |
| any | byte +0xB7 (alarm group), word +0xAC (key) | Alarm broadcast; summon key |

Values on disc (Grunt / Archer / Brute / Warrior). State 5 wander side R: 0 on 17 / 20 / 1 / 6 records; 10 on 9 / 7 / 3 / 6, every one with a pause of 90 to 150 ticks at 50 percent; 50 on 13 / 7 / 2 / 5 (no pause); 15 on 16 Archers (no pause); and singles or small groups of 16, 20, 30, 32, 40, 60, 100 and 200, a few with a pause (Archer R = 30: 60 to 120 ticks at 25 percent; Grunt R = 40: 120 to 480 at 25; Warrior R = 16, 30 and 40: 0 to 60 at 50, 120 to 240 at 25, 60 to 480 at 25). State 8 emerge distance (records): Grunt 10 (3), 15 (6), 16 (4), 20 (2), 25 (3), 30 (3), 40 (4), 200 (3); Archer 0 (9, woken only by the alarm group), 1 (2), 10 (9), 12 (2), 15 (4), 16 (1), 20 (10), 25 (4), 30 (5), 40 (7); Brute 0 (14, alarm only), 10 (5), 15 (3), 40 (5), 45 (1), 100 (11); Warrior 0 (3), 1 (1), 10 (3), 15 (5), 20 (5), 40 (5). State 6 patrols (no pause at the points except one record): Warrior path group 4 (ping-pong on 8 records, loop on 2) and group 8 (ping-pong, pause 120 to 480 at each point); Brute groups 3 and 4 (loop). State 7: +0xC4 is 0 on every Grunt, Archer and Brute record and 1 on one Warrior.

#### State 5: wander (the pre-step notice check runs every tick)

The wander rules (goal square of side R about home, R below 0.1 meaning a random triangle centroid of the navmesh, pause rolls, fidget 3) are the common ones in "Shared field behaviour" at the top of this spec. The record values are in the table above.

#### State 6: patrol

The patrol rules (path group named by +0xB8, loop or ping-pong by +0xBC, point pauses, random stops) are the common ones in "Shared field behaviour" at the top of this spec.

#### State 7: sentry

The sentry rules (walk home, look first, stand and fidget, or hold the placed facing when +0xC4 is nonzero) are the common ones in "Shared field behaviour" at the top of this spec. Every record in play has +0xC4 = 0 except one Warrior.

#### State 8: hidden at home and the wake

The enemy stands at home playing the hide pick (id 9, which none of the four tables has, so clip 0 plays) or walks home (gait 0) when more than 0.35 away; the hidden rule applies because the class state equals the default word 8. The wake rule is SPEC-0038 Behavior 40: sub-state 0 tests the horizontal distance against the record's emerge distance times F1 of the hero's action state (0.7 standing, so 40.0 wakes at 28.0; 0.75 slow walking; 1.5 walking, states 3 and 9; 1.75 running, states 4, 6 and 10; 1.0 in any other state); a distance of 0 never wakes it. Sub-state 1 (the next tick) requests the emerge animation (id 165 for the flyer kinds, id 10 for Brute and Warrior) and sets sub-state 2; sub-state 2 goes to agro when the clip passes its gate. The enemy is hidden and untouchable for the whole emerge clip because state 8 equals the default word. An alarm broadcast of the enemy's group in state 8 plays the emerge clip the same way. Entering agro runs the setter (SPEC-0038 Behavior 5): outer sub-state 0, hold 0, recovery 0, all cooldowns 0.

#### Start of life, and returning to the default state

The think loop starts as follows (SPEC-0038 Behavior 1): default word 8 starts state 8 directly; any other word runs the setter with 0. For these kinds the setter's default case gives state 23 when the default word is nonzero (Display in default is 0), and state 0 when it is 0. State 23 plays animation 81 through its gate (Grunt and Archer clip 31, 18 keys, gate 69; Brute clip 29, 24 keys, gate 93; Warrior clip 18, 20 keys, gate 77) and then runs the setter with the default word, giving state 5, 6 or 7 (or 8 for a word of 8). State 23 is not a hidden state, so a freshly bound enemy is visible and hittable while it plays 81 (a hit then wakes it into agro with the doubled damage), and vanishes when 81 ends. The same route is the way home: every exit from agro (leash, search give-up, victory pose) runs the setter with 0 and passes through state 23, then the default state, where a wanderer or sentry walks back to its home point and a hide-in-place enemy (state 8) walks home hidden. Default word 0 enemies sit in state 0 (hidden, idle) and never move on their own.

#### Noticing the hero and decloaking (states 5, 6, 7 and the alarm group)

In states 5, 6 and 7 the pre-step runs the notice check every tick (it can pass only on the one tick of each cycle on which the awareness countdown is 0: every awareness timer + 2 ticks) and a pass calls the setter with 2. Because Display in default is 0, that setter gives state 21 and does not record the hero's position or draw a search countdown: state 21 keeps the values it already holds (last-seen point (0, 0, 0), countdown -1 at creation). State 21 (flyers use the ids in brackets) then runs each tick, in this order:

1. If neither animation 82 nor animation 17 (148) is current, request animation 16 (147). A request of the id that already plays changes nothing.
2. Aim at the stored point with the row's aim turn weight (row 0x84, or 0x88 for flyers).
3. Run the notice check again (a new pair of rolls on the same tick; the awareness countdown is still 0). If it passes, or animation 82 is current: if 82 is not current and the table has it, request 82 and stay; if 82 is current, wait until it passes its 0.99 gate and then call the setter with 19 (agro); if the table has no 82 go to agro at once.
4. If the check failed and 82 is not current: the stored countdown goes down by 1; at or below 0 (always, on the first tick) the enemy requests animation 17 (148) and, when that clip finishes, calls the setter with 0 (state 23, then the default state). The notice check is repeated every tick meanwhile, so a pass during the 17 clip still leads to 82.

So a noticed minion decloaks on the same tick with probability p (the chance of passing the notice check once, given in each kind's section; about 0.66 for a Grunt at 10 units, 0.5 for a Brute, 0.76 for a Warrior at 8). Otherwise the look clip (16) starts and the enemy is committed to animation 17 (172 ticks for the Grunt, 69 for the Warrior, 256 for the Brute) unless a later awareness cycle (62, 62 or 32 ticks apart) passes the check, which restarts the chain with clip 82. If none passes before 17 ends, the enemy runs the setter with 0 and goes back to its default state. The 82 clip: Grunt and Archer clip 32 (17 keys, gate 65, the model rises 9.0 units per second), Brute clip 24 (20 keys, gate 77), Warrior clip 5 (20 keys, gate 77). The enemy is hittable from the first tick of 82 (it is no longer hidden) and a hit then sends it to agro with doubled damage. Agro starts at the gate tick A; the agro machine first runs on A + 1 (outer sub-state 0 sets the archetype), the first attack choice is possible on A + 2.

Notice values per kind are in each table below. The chance is the interpolation of SPEC-0038 Behavior 3: the near-edge chance at the minimum range and the far-edge chance at the maximum range, both scaled by F1 of the hero's action state for the band edges only; a hero closer than the near edge is noticed with no roll B.

Alarm groups (SPEC-0038 Behavior 3): the first member of a group to enter agro broadcasts; every other member in state 0, 3, 5, 6, 7 or 14 goes, at its next 15 to 45 tick check, to state 2 (so state 21 for these kinds) and plays 18 (149), which the step 1 request replaces on the next tick. Such a member does not go to agro by itself: it must pass the notice check on its own, within its own notice band, or it plays 16, then 17 and returns to its default state. A member in state 8 plays its emerge clip and then enters agro whatever its distance from the hero. The group is by byte value and spans every enemy of the area that has that byte, whatever its kind.

Notice gate between groups (SPEC-0038 Behavior 3, step 1): a hidden minion cannot notice the hero at all while another Display-in-default-0 enemy that is in agro or state 2 belongs to a different alarm group, both group bytes being other than 255. Two groups of the same area therefore do not fight the hero at once; a minion with group 255 is never blocked and never blocks.

#### Losing the hero

Agro exits to the default state (setter 0, then state 23) by: the leash (no route to the hero for 181 consecutive ticks while the walk waypoint is at least 15.0 from the hero, SPEC-0038 Behavior 23); the lost-sight search (outer sub-state 9: the sight line is blocked, the enemy walks to the last-seen point and gives up after the search counter passes 120); the victory pose after a killing blow (73, 163 for flyers, then the default). There is no distance give-up and no leash on distance alone: a minion with a route follows the hero indefinitely. The Archer's sense flag "always has line of sight" does not stop the sight-line search; it only widens its notice test.

#### Altador exterior waves

The 11 Brute, Warrior and Grunt records of the Altador exterior wave regions (loaded by the legend-gate sequence, SPEC-0041) are ordinary state-8 minions: hidden, woken by the emerge distance or their alarm group, no script call hands them to agro, so a wave is fought with the same decloaking as the field. The wave controller counts a wave won when every character of the loaded region is dead.

#### Hero actions against the shared model

| Hero action | Result |
|---|---|
| Walks within range of a hidden minion | Notice rolls once per awareness cycle; on a pass the look-then-82 sequence of state 21 starts; nothing is visible until 82 plays |
| Swings or shoots at the spot of a hidden minion | Passes through; no wake, no damage, no doubling |
| Hits it while 82 or 81 plays, or in agro | It is hit: damage doubled if it was not in agro, agro at once, hit reaction |
| Wears the Tattered Cloak | Minions are drawn and hittable in every state; the hero is almost never noticed through a blocked sight line and is noticed at half the chance otherwise |
| Stands still within the wake distance of a state-8 minion | The emerge distance is scaled by 0.7 for a hero in action state 1 (40.0 wakes at 28.0), 0.75 slow walking, 1.5 walking, 1.75 running |
| Fights one member of an alarm group | The other members that are hidden or wandering decloak in the next 15 to 45 ticks (state 8 members emerge) |

#### What a port most likely gets wrong, shared by the four kinds

1. Drawing them. They are invisible in states 5, 6, 7, 8, 0 and in state 21 until 82 plays; the player sees a minion only after it decloaks (or with the cloak). The first thing the player sees is clip 82.
2. Letting a hit or a Mote wake a hidden minion. It cannot be hit; only noticing or an alarm wakes it.
3. Sending a noticed minion straight to agro. There is always state 21 and clip 82 first (65 to 77 ticks of vulnerable emerge) unless the script ran the invisibility-off command, and a false start (clip 16, then 17, then back to the default state) happens when the second notice roll fails.
4. Treating state 5 as a standing idle. It walks goal to goal over a square of side R about home (or the whole navmesh when R is 0), resting only when the pause percent says so.
5. Waking state-8 minions on a timer or notice roll. They use only the emerge distance and the alarm group; an emerge distance of 0 means the alarm group is the only trigger.
6. Returning at once to the default state. Every exit passes through state 23 and clip 81 (visible, hittable), then the default state.

### Minion Grunt (rows 183 to 189)

#### Identity and tunables

| Item | Value |
|---|---|
| Records | Monster type 80, variants 0 to 6 (rows 183 to 189), name "Minion Grunt"; one clip table shared by 171 of the 175 records (a different table in the Illusen's Glade regions, four records, is described under Edge cases). 93 placements of row 183 (64 of them summon-reserved, key 2844), 17 of row 184, 14 of 185, 11 of 186, 15 of 187, 22 of 188 and 8 of 189 |
| Class and approach | Class 17; approach routine 1 (rush); Can fly 1 (flies while the retarget countdown is at most 0, which it always is: its hit reaction never grounds it) |
| Hit points, body | 50; radius 2.0, height 2.0, weight 5.5; attack-manager weight 8; hostile; Display in default 0 |
| Flight | Row speeds idle 3.0, agro 7.0 (6 and 14 units per second on top of the clip's own root travel, SPEC-0040 Behavior 14D item 7); max fly height 10.0; wait-in-line radius 25 |
| Notice | View cosine 0.5, awareness 90, timer 60 (cycle 62 ticks), near range 0, far range 20, chance 98 at the near end and 50 at the far end, no always-has-line-of-sight; search timer 60 to 240 (unused: state 21 does not draw it) |
| Turn | Aim turn weight in the hold sub-states 1.0 (ground) and 0.8 (flying) |
| Agro bands | Rush band 8.0 to 15.0 (hold between them), desired distance 8 + 7 f for f from 0.20 to 0.80 (9.4 to 13.6), redrawn after every normal attack choice |
| Recovery after an attack | int[15, 60] ticks |
| Missile defence | Hold 120; dodge chance 10 (9 of 100); block 0 |
| Melee defence | Block 0, 0, 0 (quick, normal, heavy); dodge 50, 10, 0 (49, 9 and 0 of 100); defence timer after a defence int[120, 300] |
| Other | Blocking Angle 0 (never guards), Damage Reduction 0, no steal, no map search, knockback 0 on both slots, no projectiles |

Attack slots (slots 0 and 3 are unused, type 0xFFFF; both used slots are melee and both carry trigger special 1 = usable only while airborne, which is always true):

| Slot | Distance band (strict) | Chance | Facing arc (dot above) | Melee range | Delay after the choice | Damage | Flags | Counter |
|---|---|---|---|---|---|---|---|---|
| 1 | 0 to 15 | 50 | 0.966 | 5.0 | int[180, 300] (row 183), int[120, 240] (others) | 25 | blockable | trigger 2 (after a dodge), chance 25 |
| 2 | 1 to 20 | 40 | 0.966 | 5.0 | int[180, 360] (row 183), int[120, 240] (others) | 50 | unblockable (flag 0x1) | trigger 6 (own hit reaction), chance 5 (row 183) or 10 (rows 184 to 189) |

Wind-up and strike speed words: slot 1 1.3 (wind-up and strike), recovery 1.0; slot 2 all 1.0. Both slots have orient flag 1 (the Grunt aims at the hero with weight 1.0 in sequencer states 1 to 3). Weapon volumes (finite cylinders): weapon 1 radius 3.0 height 1.0, weapon 2 radius 3.0 height 1.0, weapon 3 radius 4.0 height 2.0, weapons 4 and 5 radius 3.0 height 1.0.

Differences between rows (everything not listed is identical to row 183):

| Row | T | Hero status on a hit (id, chance 50 on both slots) | Slot 1 duration | Slot 2 duration | Other |
|---|---|---|---|---|---|
| 183 | 1 | 7 fire | 30 to 60 s | 5 to 10 s | delays 180 to 300 and 180 to 360, slot 2 counter chance 5; the Darkest Faerie's three Grunts and the Minion Wizard summons use this row |
| 184 | 2 | 2 poison | 30 to 60 s | 5 to 10 s | delays 120 to 240 on both slots |
| 185 | 3 | 6 confusion | 30 to 60 s | 5 to 10 s | |
| 186 | 4 | 0 stun | 6 to 8 s | 6 to 8 s | |
| 187 | 5 | 3 slow | 30 to 60 s | 5 to 10 s | |
| 188 | 6 | 1 curse | 30 to 60 s | 5 to 10 s | |
| 189 | 6 | 0 stun | 30 to 60 s | 5 to 10 s | slot 2 band 0 to 30, range 8.0, delay int[0, 120], counter trigger 5 (the sequencer is in its blocked state and L1 is held) at chance 75 |

(Status durations are trunc(int[lo, hi] / 60) seconds; the roll is int[0, 100] strictly below 50.)

#### Animation ids and clips (shared by all rows)

| Id | Use | Clip |
|---|---|---|
| 0, 97 | idle pick | clip 0 (10 keys, loops); no fidgets (ids 115 to 119 absent) |
| 98, 99, 100 | flyer wander, patrol, named-target walk | clip 20 (12 keys, loops, root travel 8.68 units per second) |
| 101 | rush forward (flying) | clip 2 (10 keys, loops, no root travel; the speed is the row's 14 per second) |
| 102 | rush back-step (flying) | clip 3 (10 keys, loops, root travel -11.02 per second, plus the row's 14 backward) |
| 155 | rush hold (flying) | clip 4 (21 keys, loops) |
| 39 / 44 | slot 1 / slot 2 strike (B) | clip 10 (12 keys, one-shot, weapon bit 0x1 on keys 5 to 8) / clip 12 (13 keys, one-shot, bits 0x3 on keys 8 to 10) |
| 41 / 46 | blocked (D) | clip 9 (7 keys, gate s+25) / clip 11 (7 keys) |
| 33, 37, 38, 40, 42, 43, 45, 47 | wind-ups, follow-throughs, recoveries of slots 1 and 2 | none: skipped in the same tick (ids 34 to 36 and 49 have clips, but they belong to the unused slots 0 and 3 and are never requested) |
| 138, 139 | missile dodge (flying) | clip 18 (10 keys, one-shot, sideways 17.1 per second), gate s+37 |
| 140 | melee dodge (flying) | clip 17 (10 keys, one-shot, backward 21.9 per second), gate s+37 |
| 152 | hit reaction kind 6 (flying) | clip 19 (10 keys), gate s+37 |
| 58 | death | clip 30 (10 keys), gate s+37 |
| 16 (147), 17 (148), 18 (149) | look, lose interest, search (ground / flyer) | clip 21 (75 keys, gate s+295), clip 27 (44 keys, gate s+172), clip 1 (18 keys, gate s+69) |
| 81, 82, 165 | disappear, appear, flyer emerge | clip 31 (18 keys, gate s+69), clip 32 (17 keys, gate s+65), clip 32 |
| 73 (163) | victory pose | clip 15 (37 keys, gate s+144) |

#### Per-tick order, start and wake

The shared field model above gives the start (state 23 or state 8, wander, sentry, hidden, decloak). The Grunt's own numbers: a hidden Grunt in state 5 notices a standing hero (F1 0.7, far edge 14.0) at distance d with probability per cycle p = 0.9 x (chance - 1) / 100 where chance = 98 - 2.4 d (halved if the hero is outside the 60 degree half-cone), for example p = 0.66 at 10 units and p = 0.58 at 14; it then decloaks with clip 82 (gate 65) and enters agro 65 ticks after the request. A state-8 Grunt uses clip 165 (gate 65) the same way. In agro (SPEC-0038 Behavior 44) each tick runs: recovery countdown, missile defence, melee defence, lost-sight test, normal choice, counter choice, outer switch, leash. The outer switch starts at outer 0 (set-up), then outer 1 (rush).

#### State table (agro)

| State | Action id and clip | Keys | Gate / exits |
|---|---|---|---|
| Set-up (outer 0) | agro idle pick 141 while flying, else 1 (the table has neither: clip 0) | 10 | one tick, then outer 1 inner 1 |
| Rush hold (inner 1) | 155 -> clip 4 | 21, loops | d < 8.0 gives back off; d > 15.0 gives approach |
| Approach (inner 0) | 101 -> clip 2 | 10, loops | row speed 14 per second (0.2333 per tick); gait 2 (same clip) when d > 30; ends when d < desired |
| Back off (inner 2) | 102 -> clip 3 | 10, loops | about 25.0 per second backward (0.417 per tick) toward the point 17.0 from the hero; ends when d > desired |
| Slot 1 swing (outer 2) | 39 -> clip 10, speed 1.3 in states 1 and 2 | 12 | window s+16 to s+27; one hit at most on the first exact weapon/body contact. For row 184, a fixed level hero straight ahead at start distance 4.0–5.0 first touches at s+26; at 6.0–8.5 it first touches at s+24 (exact chase-stop samples). For row 184, the earlier s+19 “level hero” claim is contradicted by these level samples; hover-dependent timing is unconfirmed because the grid covers level Tor only. Other Grunt rows require their own exact samples. Because hit timing gates the follow-through, recalculate the end path from the actual contact tick; retain the independently derived no-contact gate s+35 and end path s+36. |
| Slot 2 swing | 44 -> clip 12 | 13 | window s+32 to s+43; one hit at most; earliest touch of a level hero s+36 (s+32 to s+35 reach only a hero high in the volume); a hero straight in front where the approach stops is first touched on s+38; gate s+49; end path s+50 |
| Blocked slot 1 (state 4) | 41 -> clip 9 | 7 | strike blocked on its contact tick b (usually s+24 to s+26: s+26 for a level hero 5.5 or nearer at the request, s+24 at 6.0 to 8.5; s+16 at the earliest): state 4 on b+1, gate b+26, end path about b+27 |
| Melee dodge (outer 5) | 140 -> clip 17 | 10 | gate s+37, then the defence exit |
| Missile dodge (outer 6) | 138 or 139 -> clip 18 | 10 | gate s+37, then the defence exit |
| Hit reaction kind 6 (outer 10) | 152 -> clip 19 | 10 | gate s+37, then outer 0 |
| Wait in line (outer 12) | 98 -> clip 20 | 12 | walks to a stand-off point (SPEC-0038 Behavior 45) |
| Victory pose (state 10) | 163 -> clip 15 | 37 | gate s+144, then the default state |
| Dead | 58 -> clip 30 | 10 | plays once; corpse removal as in the shared section |

#### Attack chooser

Each tick before the outer switch, while the hero is attackable and the recovery countdown is below 1 and the outer sub-state is 1, 3, 4 or 11: slot 2 first (cooldown below 1, 1 < d < 20, facing above 0.966, roll int[1, 100] below 40, a route exists or d < 10), then slot 1 (0 < d < 15, roll below 50, same route rule). Chance of a start in a tick with both eligible (d from 1 to 15): 1 - 0.61 x 0.51 = 69 percent; from 15 to 20: 39 percent (slot 2 only); inside 15 with slot 2 on cooldown: 49 percent. The sequencer starts in state 0; for a melee slot with d above 5.0 it flies toward the hero (gait 0, 14 units per second, the step dropped within 3.175 of the hero) until d is 5.0 or less, then state 1 on the next tick. With the wind-up missing, state 1 requests the strike clip B and enters state 2 in the same tick. After the choice the slot cooldown is drawn; at the end path it is drawn again and the recovery countdown int[15, 60] starts.

#### Movement

Rush archetype with the flyer gait set: approach at 14 per second while d is above the desired distance (hold 8 to 15), back off at 25 per second when d is below 8. Height: each tick the Grunt moves its height by at most 0.1 toward the hero's y + min(10.0, 0.2 x d), and at least 2.0 above the ground under the goal; during the sequencer states 7 to 9 and 12 (unused here) the target is the hero's y + 2.0. A route is needed for the approach archetype; with no route the leash of the shared rules applies. A Grunt in agro is never a ground walker.

Movement table (rows 183 to 189, all the same clips; row flying velocity 3 idle and 7 agro, so the row term is 6 per second outside agro and 14 in agro; Can Fly 1; radius 2.0; flying turn 0.8):

| Mode | Source | Clip (keys) | u/tick | u/s | Turn | Exit |
| --- | --- | --- | --- | --- | --- | --- |
| Wander, walk home, wait in line | root + row | 98 (12) | 0.245 | 14.68 (8.68 + 6) | steer | arrival 0.5, notice |
| Approach (inner 0) | row only | 101: no root | 0.233 | 14.0 | steer | d below the drawn desired 9.4 to 13.6 |
| Hold (inner 1), 8 to 15 | none | 155 | 0 | 0 | aim 0.8 | d below 8 or above 15 |
| Back off (inner 2), below 8 | root + row, facing the hero | 102 (10) | 0.417 backward | 25.02 (11.02 + 14) | steer, back gait | d above desired |
| Charge | none: slot 0 has the unused type, so its leftover charge words (kind 2, distances 40 to 50 on row 183, 15 to 45 on the others) are never read and the Grunt never charges | | | | | |
| Hit (reaction kind 6) | root | 152 (10) | | back 15.02 over 40 ticks, away from the attacker | snap to attacker | gate, outer 0 |
| Dodge 138 / 139, 140 | root | (10) each | | side 11.36 over 40 ticks; back 13.63 over 40 | | gate |
| Corpse | root, falls | 58 (10), then grounded by the death hook | | back 11.82 over 40 ticks | facing kept | disappear timer 1 tick: no held slide |

Height: Can Fly 1 rule (shared section): the goal is the hero's height + min(10, 0.2 d), at least 2.0 above the ground, moved 0.1 per tick. A Grunt holding 8 to 15 away hovers 1.6 to 3.0 above the hero's feet (at least 2.0 above the floor). Port mistakes: giving the approach a clip speed (101 has no root; all 14 per second come from the row); forgetting the 15-unit flinch slide (a Grunt hit at close range is thrown out of sword reach); lifting the 0.1-per-tick height limit.

#### Hit and hurt windows

Strike windows are tested against the hero's body capsules by weapon 1 (bit 0x1) and, for slot 2, weapons 1 and 2 (bits 0x1 and 0x2 together, keys 8 to 10). A single hit per window is registered; neither clip reopens a window, so each swing hits at most once. The window start is not the hit tick: the hit lands on the first window tick on which a weapon cylinder overlaps a hero capsule, which for a hero whose feet are level with the Grunt's origin is s+24 at the earliest (slot 1, window from s+16) and s+36 (slot 2, window from s+32), and for a hero straight in front where the approach stops s+24 to s+26 (slot 1: s+26 at 5.5 or nearer, s+24 at 6.0 to 8.5, for a level hero) and s+38 (slot 2). Damage before armour and shield (SPEC-0036 Enemy attack damage): 25 and 50 plus the status roll. Knockback on the hero is 0 on both slots. The hurt side: the Grunt can be struck in every agro sub-state except the melee and missile dodges (outer 5 and 6), while dead, or while hidden; the hero's swing tests its body capsules (shared rules).

#### Weapon reach

Conventions are those of "Weapon reach tables" in the shared field behaviour. Rows 183 to 188, Medium.

| Slot (phase, clip) | Weapon | Window ticks (level contact) | Centre forward / up / side | Tor ahead, at the tick | Tor ahead, from the start | Tor feet vs origin |
|---|---|---|---|---|---|---|
| 1 (B, clip 10) | w1 (3.0 x 1.0) | s+16 to s+27 (s+24 to s+27) | -1.2 to 4.6 / -1.0 to 6.4 / -3.2 to 1.4 | 0.0 to 7.4 | 2.0 to 9.3 | -7.0 to 9.0 |
| 2 (B, clip 12) | w1 (3.0 x 1.0) | s+32 to s+43 (s+36 to s+43) | -0.6 to 5.3 / -0.7 to 5.5 / -2.0 to -0.3 | -2.8 to 7.8 | -0.3 to 10.0 | -6.5 to 8.0 |
| 2 (B, clip 12) | w2 (3.0 x 1.0) | s+32 to s+43 (s+36 to s+43) | -0.1 to 5.3 / -0.7 to 5.7 / 0.1 to 1.6 | -2.8 to 8.0 | -0.3 to 10.2 | -7.0 to 8.0 |

The Grunt's hover height (1 to 3 above the hero) is not folded into the feet column. The sequencer starts the swing only at d <= 5.0; the clip's own root motion carries the Grunt about 2 forward over a swing (the difference between the two range columns). One placement in Illusen's Glade has an older animation layout; there a hover clip carries weapon 3 (4.0 x 2.0) on every key and touches a level hero 0.2 to 6.0 ahead, and the swing itself is the second clip.

#### Defence and counter rules

Melee defence (once per hero swing, hold below 1, not in a hit reaction, not mid-attack except sequencer states 0, 1 and 5, hero within 8.0 and in front): the block roll never passes (chance 0); the dodge roll passes below 50 for a quick swing, 10 for a normal one and 0 for a heavy one. A dodge sets outer 5, the hold to 120 and plays 140 (gate s+37); after the exit the hold is int[120, 300], so a second dodge cannot start for 120 to 300 ticks. Missile defence: for each live hero shot whose path within 120 ticks meets the Grunt's capsule, int[1, 100] below 10 gives outer 6 (clips 138 or 139, gate s+37). Hit reactions: kind 6 (clip 152) starts on every hit, restarts on the next, and ends 37 ticks after the last. Counters: slot 2 (trigger 6) each tick of the reaction when its cooldown is below 1, 1 < d < 20 and the hero is in front: roll int[0, 100] below 5 (row 183) or 10 (rows 184 to 189) starts the slot-2 swing (replacing the reaction); slot 1 (trigger 2) in outer 5 or 6 while the hero's recover window (Tor only) is open: roll below 25 starts the slot-1 swing. Row 189 adds slot 2 as the counter after a blocked swing with L1 held, at roll below 75.

#### Hero actions against this enemy

| Hero action | What the Grunt does |
|---|---|
| Stands still within 14 units of a hidden Grunt | Notice roll once per 62 ticks; the Grunt decloaks and attacks about 65 ticks after a pass |
| Walks or runs near a hidden Grunt | The far edge grows by F1: 30 units when walking (1.5), 35 when running (1.75) |
| Quick swing (Tor 5, 6; Roberta 5, 6, 7, 10) inside 8 units, in front | Dodge 49 of 100, once per swing |
| Normal swing | Dodge 9 of 100 |
| Heavy swing | Never dodged |
| Roberta's shots | 9 of 100 to dodge when the path meets the body inside 120 ticks |
| A landed hit | Damage by SPEC-0036 (doubled if it was not in agro), hit reaction 152 for 37 ticks, counter chance each tick |
| Guarding (L1) | Blocks slot 1 when the hero faces the Grunt (dot at most -0.5); does nothing against slot 2 |
| Hero hurt, knocked down or dead | No new choice; cooldowns stop; a running swing finishes |

#### Edge cases for implementers

- **Illusen's Glade Grunts (row 183, four records in two regions).** Their own clip table is an older layout with the swing split differently: B of each slot (ids 39 and 44) is a looping 10-key hover clip with weapon bit 0x4 on every key, and the true swing clip is the follow-through C (ids 40 and 45: slot 1 clip 10, bits on keys 5 to 8, gate 45; slot 2 clip 12, bits on keys 8 to 10, gate 49). Because the weapon-3 volume (radius 4.0, height 2.0) is live on the first tick of B, a hero inside it is struck on the first tick of state 2 (s+1), then C plays as follow-through with no further hits; with no contact B's gate ends the swing at s+30 (slot 1) or s+39 (slot 2); after the early hit the end path comes at s+48 (slot 1) or s+52 (slot 2). Their table also has idle fidgets 115 and 116. All other tables are identical.
- **Always airborne.** Both attack slots need the airborne flag, so a Grunt that was somehow grounded could not attack. Never ground a Grunt: its hit reaction is kind 6 (the stop-and-stagger), not kind 0.
- **Summon-reserved Grunts** (key 2844; the Darkest Faerie's burst, the Minion Wizard's lobbed summon, the fire Wizard row 211): created in state 21 with clip 82 requested by the spawner and enter agro after the gate (SPEC-0029 Behavior 31); they are visible only from the first tick of 82.
- The attack choice with the hero closer than 1 unit: slot 2 is out of band; slot 1 (0 < d < 15) still passes. The sequencer starts in state 1 and swings.
- The Grunt's step is dropped within 3.175 units of the hero while it is in sequencer states 0 to 3, so it can end its approach just outside the weapon volume; the strike still hits because the volume radius (3.0 or 4.0) exceeds the gap.
- Row 189's slot 2 has no waiting time worth the name (delay up to 120), so a hero who stays in range sees back-to-back unblockable stuns.

#### What a port most likely gets wrong

1. Making the Grunt visible and attackable while it wanders. It is invisible until clip 82 plays (shared rules).
2. Treating the wake as agro. There are 65 ticks of clip 82 (and, from state 5, the clip 16 / 17 false start) before the first attack choice, which is two ticks after agro.
3. Giving it ground movement. Its speeds are the row's 14 per second plus the clip's root, flying at hero height + min(10, 0.2 d); the back-step is 25 per second.
4. Using the wind-up ids. Only the strike clips 39 and 44 exist; slot 1 plays at speed 1.3 until the hit, then at 1.0.
5. A single melee slot. The Grunt has two: a blockable 25-damage swing and an unblockable 50-damage swing, 69 percent per tick to start when both qualify.
6. Letting it dodge freely. Dodge needs a quick swing (49 of 100); after one dodge there is a 120 to 300 tick quiet period.
7. Missing the status. Every hit rolls 50 of 101 for its row's status (stun for rows 186 and 189), duration 30 to 60 seconds for slot 1 on most rows.

#### Test vector

Row 184 Grunt, already in agro (t = 0 the tick the setter ran), hero standing still 12.0 units in front, hero guard off, nothing else near. t = 1: set-up. t = 2: first choice, 69 percent per tick (slot 2 39, else slot 1 49), so the choice falls on t = 2 with 69 percent, t = 3 with 21, later otherwise; call it c. The Grunt flies toward the hero at 0.2333 per tick from c + 1 and the state-0 test sees d <= 5.0 on tick c + 31 or c + 32; s = c + 32 (give or take 1). The Grunt then hovers about 1 unit above the hero's feet. The prior s+24/s+25 contact and resulting follow-through timing depended on that non-level relative pose and is not established by the level-only exact grid; keep the resulting hit, gates and 600-tick damage count unresolved pending an exact hover-pose query. For reference only, row 184's fixed level Tor samples at start d=4.0–5.0 touch slot 1 at s+26 and slot 2 at s+38; those are not substitutes for the hovering fight. Recovery/cooldowns continue as listed in the state and chooser tables.

#### Open questions

- Whether a Grunt in sequencer state 0 reaches d <= 5.0 on tick c + 31 or c + 32 (floating-point error at exactly 5.0; needs a runtime check). Default if unresolved (rewrite choice): s = c + 32.
- Weapon contact geometry. Resolved: the general rule of the shared field behaviour (weapon cylinders on attachment frames 0 to 2 against the hero's capsules, no facing test); the sequencer only starts the swing at d <= 5.0. The distances are in the Weapon reach table. Stand-in only for a port without attachment frames (rewrite choice): count a window tick as hitting when a level hero is inside the "at the tick" range of the slot's row (1.0 behind to 7.4 ahead for slot 1, 2.8 behind to 8.0 ahead for slot 2) and the tick is at or after the row's first level-contact tick.
- Resolved (timing pass): blocked-strike timing. Only slot 1 can be blocked (slot 2 is unblockable). The blocked tick is b; state 4 runs from b + 1 and requests D (id 41, clip 9, 7 keys), whose gate passes on b + 26 (4 x 6 + 1 ticks after the request); state 5 follows on b + 27 and, with no recovery clip, the end path runs on b + 27 (High for the arithmetic).
- Resolved (Medium-High; proactive (open close-out)): the actor render loops gate each actor with a drawable predicate that requires a render handle and that the hidden test, the hold word and the "cannot be seen" flag are all clear, so a hidden minion is skipped by those loops. Whether the main model submission uses the same predicate is not separable from data (needs a runtime check). Rewrite choice: do not draw hidden minions.

### Minion Archer (rows 190 to 196)

**Player's answer (R-0149).** Roberta's shots: it dodges a shot only 9 times in 100 and a quick swing 49 in 100 (normal 9, heavy never), so it is the easy ranged kind; it holds 15 to 60 units away and never gives up (SPEC-0038 Behavior 44).

#### Identity and tunables

| Item | Value |
|---|---|
| Records | Monster type 80, variants 7 to 13 (rows 190 to 196), name "Minion Archer"; one clip table for all 156 records. Placements: row 190 38, 191 15, 192 18, 193 16 (6 summon-reserved, key 2807), 194 34, 195 34, 196 1 |
| Class and approach | Class 17; approach routine 3 (keep distance); Can fly 1 (airborne always) |
| Hit points, body | 40; radius 2.0, height 3.5, weight 5.5; attack-manager weight 8 (never waits in line: approach 3); hostile; Display in default 0 |
| Flight | Row speeds idle 4.0, agro 8.0 (8 and 16 units per second on top of root travel); max fly height 15.0 |
| Notice | View cosine 0.5, awareness 90, timer 60, always has line of sight 1; row 190: range 5 to 80; rows 191 to 196: range 10 to 100; chance 98 at the near end, 50 at the far end |
| Turn | Hold aim weights 0.5 (ground) and 0.1 (flying: the Archer turns slowly in the hold) |
| Agro bands | Desired band 15.0 to 60.0 (hold between), desired distance 15 + 45 f (24 to 51), retreat band 0 to 0 (unused) |
| Recovery after an attack | int[15, 30] ticks |
| Defence | Same as the Grunt: missile hold 120, dodge 10, block 0; melee block 0, dodge 50, 10, 0; defence timer int[120, 300] |
| Other | Blocking Angle 0, Damage Reduction 0, no steal |

Attack slots (slots 0 and 1 unused):

| Slot | Kind | Band (strict) | Chance | Facing arc | Range | Delay | Damage | Counter |
|---|---|---|---|---|---|---|---|---|
| 2 | melee (a pose) | 15 to 100 | 5 | 0.966 | 25.0 | int[180, 600] | 0 | trigger 5 (blocked state with L1) chance 75: never reached |
| 3 | missile, flight mode 0 | 10 to 100 | 75 | 0.0 (anywhere in front) | 30.0 (unused for missiles) | int[60, 180] | 10 | none |

Slot 3 projectile: speed 2.0 per tick, range 70 (lifetime 35 ticks), radius 0.05, spawn count 2 (see Edge cases: only one arrow is fired), element T of the row, hero status at chance 100 (value trunc(int[lo, hi] / 60)), visual kind 23. Speed words: all 1.0 on rows 190 and 191; wind-up, strike and recovery 1.3 on slot 3 of rows 192 to 196.

Differences between rows (everything else identical):

| Row | T | Slot 3 status (id, duration) | Slot 3 speeds | Notice range |
|---|---|---|---|---|
| 190 | 1 | 7 fire, 10 to 30 s | 1.0 | 5 to 80 |
| 191 | 2 | 2 poison, 10 to 30 s | 1.0 | 10 to 100 |
| 192 | 3 | 6 confusion, 10 to 30 s | 1.3 | 10 to 100 |
| 193 | 4 | 0 stun, 6 to 8 s | 1.3 | 10 to 100 |
| 194 | 5 | 3 slow, 30 to 60 s | 1.3 | 10 to 100 |
| 195, 196 | 6 | 1 curse, 30 to 60 s | 1.3 | 10 to 100 |

#### Animation ids and clips

Locomotion, dodges, hit reaction, death, look, appear, disappear and emerge ids are the same as the Grunt's (the Archer and the Grunt share the rig and the 34-clip set). The keep-distance flyer gait set is: forward 107 (clip 5, 10 keys, loops, weapon bit 0x4 on every key, which is ignored outside a strike state; the speed is the row's 16 per second), back 108 (clip 3, root travel -11.02 per second plus the row's 16 backward, 27 per second in all), hold 156 (clip 4). Attack ids that exist: slot 2 wind-up 43 = clip 15 (37 keys, one-shot, no event bits, gate s+144); slot 3 strike 49 = clip 14 (20 keys, one-shot, the spawn event bit 0x20 on key 14 only) and blocked 51 = clip 9 (7 keys). Ids 33 to 42, 44 to 48, 50 and 52 have no clip. With speed 1.0 the spawn event is active on ticks s+56 to s+59 and the clip gate is s+77; with speed 1.3 the event is active on s+44 to s+46 and the gate is s+59.

#### Wake and start

As the shared rules, with these numbers: the notice check in states 5, 6, 7 passes on a standing hero within 56 units (row 190: far edge 80 x 0.7) with p = 0.9 x (chance - 1) / 100, chance = 98 - 0.64 (d - 5) on row 190 (for example p = 0.73 at 30 units) and sees through walls; decloak clip 82 (gate 65) then agro; state 8 uses emerge 165 (gate 65).

#### State table (agro)

| State | Action id and clip | Keys | Gate / exits |
|---|---|---|---|
| Hold (inner 4) | 156 -> clip 4 | 21, loops | d > 60 gives approach; d < 15 gives back off |
| Approach (inner 3) | 107 -> clip 5 | 10, loops | 16 per second; ends when d < desired (24 to 51) |
| Back off (inner 5) | 108 -> clip 3 | 10, loops | 27 per second, toward the point 60 units from the hero; ends when d > desired |
| Slot 3 shot | 49 -> clip 14 | 20, one-shot | arrow at s+56 (s+44 on rows 192 to 196), gate s+77 (s+59), end path s+78 (s+60) |
| Slot 2 pose | 43 -> clip 15 | 37, one-shot | gate s+144, end path s+145 |
| Dodges, hit reaction, death, look, appear | as the Grunt | | |

#### Attack chooser

Slot 3 first: cooldown below 1, 10 < d < 100, facing above 0.0, roll int[1, 100] below 75 (74 of 100); a missile slot needs no route. Then slot 2: cooldown below 1, 15 < d < 100, facing above 0.966, roll below 5 (4 of 100), a route to the hero (the pose is melee and d is above 10). While slot 3 is on cooldown (60 to 180 ticks after each shot, drawn at the choice and again at the end path) and recovery is over, slot 2 is tried every tick at 4 percent, so it starts within 25 ticks with probability 64 percent and within 100 ticks with 98 percent: the Archer almost always poses once between two arrows. With the pose chosen at d above 25.0 the sequencer state 0 flies toward the hero (gait 0, 16 per second) until d <= 25.0, then plays clip 15 for 145 ticks with no damage and an end path that starts the recovery int[15, 30]; the pose cooldown is drawn again at the end path (180 to 600). Inside 10 units the Archer has no attack at all; it backs off.

#### Movement

Keep-distance archetype with the flyer gait set: hold between 15.0 and 60.0 from the hero; beyond 60 approach to a drawn distance 24 to 51; under 15 back off toward a point 60 from the hero until beyond the drawn distance (27 per second, so it re-opens the gap quickly). The hold aim is slow (blend 0.1), but the sequencer aims at weight 1.0 in states 1 to 3, so the arrow leaves toward the hero. The flyer height rule is the Grunt's with max fly height 15.0.

Movement table (rows 190 to 196, all the same clips; row flying velocity 4 idle and 8 agro, so the row term is 8 per second outside agro and 16 in agro; Can Fly 1; radius 2.0; flying turn 0.1):

| Mode | Source | Clip (keys) | u/tick | u/s | Turn | Exit |
| --- | --- | --- | --- | --- | --- | --- |
| Wander, patrol, walk home | root + row | 98 or 99 (12) | 0.278 | 16.68 (8.68 + 8) | steer | arrival 0.5, notice |
| Approach (inner 3), d above 60 | row only | 107: no root | 0.267 | 16.0 | steer | d below the drawn desired 24 to 51 |
| Hold (inner 4), 15 to 60 | none | 156 | 0 | 0 | aim 0.25 then 0.1 (effective 0.325) | d above 60 or below 15 |
| Back off (inner 5), below 15 | root + row, facing the hero | 108 (10) | 0.450 backward | 27.02 (11.02 + 16) | steer, back gait | d above desired |
| Retreat (inner 6) | never: the retreat minimum is 0 | | | | | |
| Shot (sequencer) | none | slot clips (the slot-3 shot clip creeps 2.2 forward) | 0 | 0 | aim 1.0 in states 1 to 3 | gate |
| Hit (reaction kind 6) | root | 152 (10) | | back 15.02 over 40 ticks | snap to attacker | gate |
| Dodge 138 / 139, 140 | root | (10) each | | side 11.36 over 40; back 13.63 over 40 | | gate |
| Corpse | root, falls | 58 (10) | | back 11.82 over 40 ticks | facing kept | timer 1 tick: no held slide |

An Archer holding 15 to 60 away hovers 3 to 12 above the hero's feet (0.2 d, capped at 15, at least 2.0 above the floor).

#### Hit and hurt windows

The arrow is a pool projectile (SPEC-0029 Behavior 9): launch at emit point 0 (the event bit 0x20 selects point 0), aim point hero + (0, 2.5, 0), aim scatter 0 inside the camera cone and a fixed horizontal offset of 5.0 units in a random direction when the Archer is outside the camera cone (dot of camera forward and unit(Archer - camera) below 0.85) plus 3.0 when the attack manager would hold it (never for approach 3), velocity 2.0 per tick along the aim line, first contact on a hero standing 20 units away at tick 10 after the spawn (about 15 ticks for a hero 30 units away), expiry after 35 ticks (36 moves, 72 units of travel; a standing hero is hit out to about 70.5 units, Distances and ranged reach). Damage 10 (before armour and shield) with the row's status at 100 percent; no knockback. The Archer is hit-able in every state except dodges, death and hidden.

#### Defence and counter rules

As the Grunt: melee dodge 49 of 100 for quick swings, 9 for normal, 0 for heavy (once per swing, hero inside 8.0 and in front); missile dodge 9 of 100; the post-defence quiet period int[120, 300]; the hit reaction is kind 6 (clip 152, 37 ticks). The Archer has no usable counter: its slot 2 counter trigger needs the sequencer in its blocked state with L1 held, and the pose never strikes.

#### Hero actions against this enemy

| Hero action | What the Archer does |
|---|---|
| Stands 30 units away | Fires the first arrow 57 ticks after the choice (choice at t = 2: arrow at t = 59), then one every 140 to 260 ticks, with a 145-tick pose in between |
| Closes inside 15 units | Backs off at 27 per second to 24 to 51 units; it has no attack at all inside 10 units |
| Stands still in the open | Takes 10 damage per arrow plus the status (100 percent) when the Archer is inside the camera cone; an Archer outside the cone aims 5.0 units off in a random direction |
| Roberta's shots at it | 9 of 100 to dodge |
| Hits it in melee | Damage by SPEC-0036 (doubled if it was not in agro), reaction 152, no counter |
| Guarding (L1) | Does not stop arrows by the guard rule of melee; arrows follow the projectile guard rules (SPEC-0029 Behavior 9) |

#### Edge cases for implementers

- **One arrow per attack.** Spawn count 2 would need a second spawn-event window, and the strike clip has exactly one, so only one arrow is fired per attack, on every row. (SPEC-0029 and SPEC-0038 test vectors that mention "two arrows" are wrong for the Archer.)
- **The unfired slot.** Slot 2 (damage 0, no strike clip) is a 145-tick pose, not an attack; it is still an "attack" for the sequencer (outer 2, recovery, cooldown, melee defence allowed in states 0, 1 and 5) and it makes the Archer fly toward the hero until d <= 25. It never damages and never steals.
- **Arrows through walls.** The shot needs only that the hero is in front of the Archer (facing above 0.0); the Archer notices the hero through walls (always-has-line-of-sight), and in agro it keeps firing until the real sight-line test starts its search.
- Summon-reserved Archers (key 2807, six records, summoned by the Minion Wizard variant row 215) start in state 21 with clip 82 like the Grunt.
- Arrows are shared with every other enemy projectile: the pool has ten slots; a full pool fires nothing but the shot counter still advances.

#### What a port most likely gets wrong

1. Firing two arrows because the row says two. One window means one arrow.
2. Making the Archer a melee fighter. It has no damaging melee; inside 10 units it only retreats.
3. Treating the 145-tick pose as nothing. It is a full attack cycle and it makes the Archer step in toward 25 units first.
4. Aiming at a moving target with lead. The aim is the hero's present position + 2.5 up (+ 5.0 sideways off camera); a running hero escapes.
5. Using wind-up ids or the Grunt's rush bands. The Archer holds between 15 and 60 and retreats at 27 per second.
6. Ignoring the sight-line-free notice. The Archer notices through walls from 56 units and more.

#### Test vector

Row 190 Archer, in agro (t = 0 the setter tick), hero standing still 30.0 units in front, in the camera's cone. t = 1 set-up; t = 2 choice at 74 percent per tick (slot 3), else slot 2 at 4 percent; take slot 3 at c = 2: s = 3, arrow spawned on t = 59 at the hero's chest height, hits a hero 30 units away on t = 59 + 15 (about 15 ticks of flight at 2.0 per tick, depending on the capsule), 10 damage and fire (status 7, 10 to 30 s), end path t = 81. Recovery int[15, 30] (to t = 97 to 112) and slot 3's cooldown int[60, 180] from t = 81 (free between t = 141 and t = 261): during the wait slot 2 is tried at 4 percent per tick, so the Archer usually (about 95 percent) starts a pose before the cooldown ends: it flies in from 30 to 25 units (about 19 ticks), poses for 145 ticks (end path s + 145), recovers 15 to 30 ticks and draws the pose cooldown 180 to 600. The second arrow follows about 1.4 ticks after the later of the two clocks (the recovery end and the slot 3 cooldown end). In 600 ticks: 3 arrows (2 to 4) and 1 pose (0 to 2); 20 to 40 damage and a status at 100 percent from each arrow that connects.

#### Open questions

- Resolved (High; proactive (open close-out)): no lead and no flight-time compensation. Every shot aims once, at its release tick, at the hero's position at that tick plus 2.5 up, with the scatter of the arrow paragraph (SPEC-0038 Behavior 13 aim rule; SPEC-0029 Behavior 13); a running hero escapes.
- The hidden-state back-off point for flyers in state 8 (it walks home with the ground gait id). The state-8 handler walks home with gait 0 (SPEC-0038 Behavior 40), whose animation is the kind's walk id; only the model clip is open. Default if unresolved (rewrite choice): fly home with clip 20 and the idle row speed (needs a runtime check; visual only).
- Resolved: the arrow's contact geometry is the pool's sphere against the hero's body cylinder (radius 1.175, height 3.5, both heroes), tested after each move (SPEC-0029 Behavior 13a); hit ticks by distance are in Distances and ranged reach.

### Minion Brute (rows 169 to 174)

#### Identity and tunables

| Item | Value |
|---|---|
| Records | Monster type 78, variants 0 to 5 (rows 169 to 174), name "Minion Brute"; one clip table for every record in play (the five placements of variant 0 in two regions differ only in sound words). Placements: row 169 10 (5 in play), 170 21, 171 1, 172 11, 173 16, 174 7 |
| Class and approach | Class 17; approach routine 1 (rush); Can fly 0 |
| Hit points, body | 300; radius 4.0, height 7.0; weight 3.0 (row 169) or 5.5 (rows 170 to 174); attack-manager weight 12; hostile; Display in default 0; footstep camera shake class 2 and rumble class 2 (SPEC-0029 Behavior 32) |
| Notice | View cosine 0.92388, awareness 80, timer 60 (cycle 62), near range 5, far range 20, chance 90 at the near end and 10 at the far end, line of sight required; search timer 15 to 45 (unused) |
| Turn | Hold aim weight 0.2 (ground) |
| Agro bands | Rush band 9.0 to 17.0 (hold between them), desired distance 9 + 8 f (10.6 to 15.4); wait-in-line radius 30 |
| Recovery after an attack | int[15, 60] (row 169) or int[15, 120] (rows 170 to 174) |
| Missile defence | Hold 45; dodge 0; block 75 (74 of 100) |
| Melee defence | Block 75, 75, 75 (row 169) or 25, 25, 25 (rows 170 to 174), the roll passing at or below the chance (75 or 25 of 100); dodge 0; defence timer after any defence int[30, 60] (row 169) or int[60, 240] |
| Guard | Blocking Angle 0.92388: in the block state (base state 20) a hero swing from the front (dot of facings below 0.924) is blocked; a non-power swing with the dot at or above 0.924 (from behind or very close to behind) has no effect at all (SPEC-0036, Hit detection step 4); power attacks ignore the guard |
| Other | Damage Reduction 0, no steal; no projectiles; weapon volumes: weapon 1 radius 5.0 height 2.0, weapon 2 radius 4.0 height 1.0, weapon 3 radius 20.0 height 20.0, weapons 4 and 5 radius 5.0 height 2.0 |

Attack slots (all four are melee; slot k phase ids are 33 + 5k + p):

| Slot | Role | Band (strict) | Chance | Arc | Range | Delay (cooldown) | Damage | Knockback | Flags | Counter |
|---|---|---|---|---|---|---|---|---|---|---|
| 0 | club swing | 4 to 20 | 25 | 0.924 | 10.0 | int[60, 180] | 25 | 6.0 | blockable, status | trigger 6 (own hit reaction), chance 40 |
| 1 | grab and throw | 10 to 20 | 25 | 0.966 | 12.0 | int[600, 900] | 50 | 0 (throw speed 17.0) | flag 0x8 | none |
| 2 | slam | 5 to 30 | 20 | 0.707 | 10.0 | int[30, 120] | 100 | 14.0 (rows 169 to 172) or 7.0 (rows 173, 174) | unblockable | none |
| 3 | roar (a pose) | 0 to 20 | 10 | 0.0 | 20.0 | 0 | 0 | 0 | none | trigger 5 (sequencer blocked, L1 held), chance 75 |

Slot 0 speeds: wind-up and strike 1.5, recovery 1.0; the other slots 1.0. The hit on slot 0 carries the row's status (50 of 101): fire (T 1), poison (2), confusion (3), stun (4, 6 to 8 s), slow (5), curse (6), 30 to 60 s except stun; slot 3's status fields are never used (damage 0). Differences between rows: the element and slot 0 status above; row 169 has recovery 15 to 60, melee block 75, defence timer 30 to 60, weight 3.0; rows 170 to 174 have recovery 15 to 120, block 25, timer 60 to 240, weight 5.5; slot 2 knockback 7.0 on rows 173 and 174; row 172 uses stun for slot 0 (6 to 8 s).

#### Animation ids and clips

| Id | Use | Clip |
|---|---|---|
| 0 | idle | clip 0 (35 keys, loops) |
| 2, 4, 11 | walk (wander, patrol, named target) | clip 23 (40 keys, loops, root travel 1.88 units per second) |
| 21 | rush forward | clip 20 (20 keys, loops, 13.54 units per second) |
| 23 | rush back-step | clip 2 (20 keys, loops, -6.05 per second) |
| 22 | rush hold | clip 17 (25 keys, loops) |
| 34 | slot 0 strike | clip 4 (30 keys, one-shot, weapon bit 0x2 on keys 15 to 18) |
| 36 | slot 0 blocked | clip 3 (13 keys, bits 0x2 on keys 0 to 2, gate s+49) |
| 39 | slot 1 strike | clip 7 (10 keys, bit 0x1 on key 9) |
| 40 | slot 1 follow-through and grab hold | clip 6 (35 keys; bit 0x1 on keys 0 to 22; bits 0x21 on key 23 = the release event) |
| 41, 42 | slot 1 blocked, recovery | clip 5 (14 keys), clip 25 (20 keys) |
| 44 | slot 2 strike | clip 9 (28 keys, bit 0x4 on keys 24 to 27, so the window opens at s+96 and stays open to the clip's end) |
| 47 | slot 2 recovery | clip 8 (31 keys, bit 0x2 on keys 15 to 18, not tested in recovery) |
| 48 | slot 3 wind-up (the roar) | clip 19 (38 keys, one-shot, gate s+148) |
| 56 | block | clip 10 (1 key, loops) |
| 57 | hit reaction kind 0 | clip 12 (7 keys, knocked back 9.2 per second), gate s+25 |
| 58 | death | clip 11 (61 keys), gate s+240 |
| 10, 82 | emerge, appear | clip 24 (20 keys), gate s+77 |
| 81 | disappear | clip 29 (24 keys), gate s+93 |
| 16, 17, 18 | look, lose interest, search | clip 21 (49 keys, gate 192), clip 16 (65 keys, gate 256), clip 19 (38 keys, gate 148) |
| 73 | victory pose | clip 26 (116 keys at speed byte 1, gate s+230) |

Ids absent: 33, 35, 37, 38, 43, 45, 46, 49 to 52 (phase ids), 9 (hide pick: clip 0 plays), 53 to 55 (dodges, never used: dodge chances 0), 126 (block-hit cue: a request of it starts clip 0).

#### Wake and start

As the shared rules, with the Brute numbers: notice band edges 3.5 and 14.0 for a standing hero, chance 90 at 5 and 10 at 20, so p = 0.8 x (chance - 1) / 100 per cycle (for example p = 0.5 at 10 units); the view cone is narrow (22.5 degrees), so a hero outside the cone halves the chance. Decloak with 82 (gate 77) then agro; state 8 emerges with clip 10 (gate 77) once the hero is closer than the emerge distance x F1: 100 (11 records, a 70-unit trigger for a standing hero), 40 (5), 45, 15, 10 or 0 (8 records woken only by their alarm group). State 6 records walk the patrol group at 1.88 units per second.

#### State table (agro)

| State | Action id and clip | Keys | Gate and exits |
|---|---|---|---|
| Set-up (outer 0) | agro idle pick 1 (absent: clip 0, the same 35-key idle clip) | 35 | one tick, then outer 1 inner 1 |
| Hold (inner 1) | 22 -> clip 17 | 25, loops | d < 9.0 gives back off, d > 17.0 gives approach |
| Approach (inner 0) | 21 -> clip 20 | 20, loops | 13.54 per second (0.2257 per tick); ends when d < desired |
| Back off (inner 2) | 23 -> clip 2 | 20, loops | 6.05 per second toward the point 19.0 from the hero |
| Slot 0 swing | 34 -> clip 4, speed 1.5 | 30 | weapon 2 window s+40 to s+50; row 169 exact no-stop samples: a fixed level hero straight ahead at start distance 6.0 misses; 6.5 first touches at s+45; 7.0–12.5 first touch at s+44; 13.0 touches at s+45; with the chase stop (5.175) 4.5 first touches at s+45, 5.0–12.5 at s+44 and 13.0 at s+45. A hit is 25 plus status, knockback 6.0. After the hit the clip finishes at speed 1.0: a hit on s+44 gives gate s+94 and end path s+95, a hit on s+45 gate s+93 and end path s+94. No contact: gate s+78, end path s+79; blocked on the contact tick b: state 4 at b+1, 36 -> clip 3, gate b+50, end path b+51. These are fixed-pose samples, not a continuous interval between samples. |
| Slot 1 grab | 39 -> clip 7 then 40 -> clip 6 | 10, 35 | see the grab section: held from s to about s+93, throw (no damage) at about s+93, end path about s+138 |
| Slot 2 slam | 44 -> clip 9 | 28 | weapon 3 window opens s+96 (radius 20.0, height 20.0); one hit at s+96; gate s+108; recovery 47 -> clip 8 (31 keys) with gate s+120 after it is requested at s+109; end path s+229 (hit or miss) |
| Slot 3 roar | 48 -> clip 19 | 38 | gate s+148, end path s+149; no damage |
| Block (outer 7) | 56 -> clip 10 | 1, loops | while the hero's swing predicate is true, then the defence exit |
| Hit reaction kind 0 (outer 10) | 57 -> clip 12 | 7 | gate s+25, then outer 0 |
| Dead | 58 -> clip 11 | 61 | plays once |

#### Attack chooser

Slots are tested 3, 2, 1, 0, first pass wins (cooldown below 1; band strict; facing above the arc; roll int[1, 100] below the chance; route rule: with charge type -1 a route exists or the slot is melee and d < 10, which is every Brute slot at d < 10). Per tick when all cooldowns are 0 and the hero is in front at 13 units: slot 3 9 percent, then slot 2 19, then slot 1 24, then slot 0 24, so any attack starts with 57 percent per tick; at 25 units only slot 2 qualifies (19 percent). Slot 3 has no cooldown at all, so after every recovery the Brute roars with about 9 percent of its choice ticks and 148 ticks of nothing. Slot 1 (cooldown 600 to 900) is rare. Closing: the sequencer state 0 runs the rush gait (0.2257 per tick) until d is at most the slot's range (10, 12, 10 or 20) and then goes on to state 1.

#### Movement

Rush archetype: hold between 9 and 17 from the hero, approach if beyond 17, back off toward the point 19.0 from the hero when closer than 9 (slowly: 6.05 per second). The Brute does not retreat quickly and does not strafe. Its footsteps shake the camera and rumble the pad (SPEC-0029 Behavior 32).

Movement table (rows 169 to 174, all the same clips; ground; radius 4.0; ground turn 0.2):

| Mode | Source | Clip (keys) | u/tick | u/s | Turn | Exit |
| --- | --- | --- | --- | --- | --- | --- |
| Wander, patrol, walk home, wait in line | root | 2 or 4 (40) | 0.0313 | 1.88 | steer | arrival 0.5, notice |
| Approach (inner 0), above the desired distance | root | 21 (20), 0.147 to 0.257 per tick inside the cycle | 0.226 | 13.54 | steer | d below the drawn desired 10.6 to 15.4 |
| Hold (inner 1), 9 to 17 | none | 22 | 0 | 0 | aim 0.2 | d below 9 or above 17 |
| Back off (inner 2), below 9 | root, facing the hero | 23 (20) | 0.101 backward | 6.05 | steer, back gait | d above desired |
| Strike clips | root | slot clips | | slot 0 clip creeps 2.35 forward, slot 1 lunges 2.43 (at the slot's speed word) | sequencer aim | gate |
| Hit (reaction kind 0) | root | 57 (7) | | back 4.26 over 28 ticks | snap to attacker | gate |
| Dodge | none: ids 53 to 55 missing (clip 0, no root) | | 0 | 0 | | |
| Corpse | root | 58 (61) | | wobbles, net 4.04 back over 244 ticks | facing kept | timer 6 ticks: released before the clip ends |

At 13.54 per second the Brute's approach equals a walking hero (13.51): a walking hero neither gains nor loses ground; a running hero (19.29) escapes.

#### Grab and throw (slot 1)

The strike state of slot 1 runs the grab machine of SPEC-0029 Behavior 23 in place of a weapon test: on the first tick of the strike state, if the hero is closer than 12.0, the dot of the Brute's facing toward the hero is above 0.5, the hero is not stunned and the path test is clear, the hero is put in base state 53 (held) and the Brute plays the follow-through clip (id 40, 35 keys). The hero stays held while the Brute remains in its strike state; a controller rumble (magnitude 0.7, 1 s) repeats every 61 ticks. When the Brute's clip shows the release event (bit 0x20, key 23, 92 ticks after the clip starts) the hero is thrown with velocity normalize(15 x facing + 8 x up) x 17.0 and takes no damage (the grab path deals 0 damage on every row; the slot damage word 50 is not applied), and the sequencer moves to its follow-through; the clip then runs out (gate about 136 ticks after it started) and the end path follows without a recovery clip. Any hit that reaches the Brute while it holds the hero frees the hero at once. The hero is held where he stands, not moved to the Brute's hand. If any grab test fails the machine falls to its recover state, which requests the slot's recovery clip (id 42, clip 25, 20 keys) on s+2; the strike state waits for that clip's gate (s+79) and the end path runs on s+80, with no contact (Medium). Confidence Medium; the open questions list the unknown timing offsets.

#### Hit and hurt windows

Slot 0: window s+40 to s+50 (weapon 2: radius 4.0, height 1.0 at attachment point 1); a level hero is touched only on s+44 and s+45, so the hit lands on s+44 (on s+45 for a hero 4.5 or 13.0 ahead at the request, and at 6.5 when the chase stop does not apply). Slot 2: s+96, weapon 3, radius 20.0 and height 20.0, centred on attachment point 2, a very large volume pointing forward and about 24 degrees down: it touches a level hero from 12.4 behind to 11.8 ahead of the Brute (Weapon reach; the slam is a ground-pound area attack, unblockable, 100 damage and knockback 14.0), and a hero inside 10 is struck on s+96. Single hit per window; neither clip reopens a window. Knockback on the hero is the slot's value (SPEC-0029 Behavior 10.1a: velocity 0.6 K along the facing and 0.8 K up). Damage before armour and shield.

#### Weapon reach

Conventions are those of "Weapon reach tables" in the shared field behaviour. Rows 169 to 174, Medium.

| Slot (phase, clip) | Weapon | Window ticks (level contact) | Centre forward / up / side | Tor ahead, at the tick | Tor ahead, from the start | Tor feet vs origin |
|---|---|---|---|---|---|---|
| 0 (B, clip 4) | w2 (4.0 x 1.0) | s+40 to s+50 (s+44 to s+45) | -2.5 to 8.7 / 1.8 to 5.5 / -7.2 to 7.5 | 4.4 to 12.8 | 6.5 to 14.6 | -2.5 to 3.0 |
| 2 (B, clip 9) | w3 (20.0 x 20.0) | s+96 to s+108 (s+96 to s+108) | -0.7 / 0.8 to 1.6 / 0.0 | -12.4 to 11.8 | -12.4 to 11.8 | -22.0 to 14.0 |

Slot 0 never touches a level hero closer than 4.4 ahead and then only on two ticks. In slot 2 the volume is 20 by 20 and its axis points forward and about 24 degrees down, so "roughly 20 units" above is the volume's size and the real reach is the range in the table (12.4 behind to 11.8 ahead, feet from 22 below to 14 above the Brute's origin).

#### Defence and counter rules

Melee defence is rolled once per hero swing (hold below 1, hero within 8.0 and in front, not mid-attack except sequencer states 0, 1, 5, not in a hit reaction): r1 = int[1, 100] at or below the block chance (75 or 25) gives outer 7 (block) with the hold set to 45; dodge is 0. In the block the Brute plays 56 and is in base state 20 (guarding) while the hero swings; a frontal non-power swing is blocked (no damage), a swing from behind has no effect at all, a power attack hits normally. After the swing the defence exit sets the hold to int[30, 60] (row 169) or int[60, 240], so a second block cannot start sooner. Missile defence: dodge 0; block roll below 75 (74 of 100) gives outer 8 (play 56 while a hero shot is live, then exit). Hit reactions: kind 0, clip 57, 25 ticks; each hit restarts it. Counters: slot 0 (trigger 6) every tick of a hit reaction when its cooldown is below 1, 4 < d < 20 and the hero is in front: roll int[0, 100] below 40 starts the slot-0 swing (40 of 101); so a Brute with slot 0 off cooldown counters a hit in about 2 to 3 ticks; if slot 0 is still on cooldown (60 to 180 ticks after each use) it staggers for the 25 ticks. Slot 3 (trigger 5): when a blockable swing of the Brute is blocked by a hero holding L1 (sequencer state 4), the roar starts at roll below 75.

#### Hero actions against this enemy

| Hero action | What the Brute does |
|---|---|
| Stays beyond the wake distance | Hidden, invisible |
| Swings at a Brute that is blocking from the front (rows 170 to 174: 25 percent of swings, row 169: 75 percent) | Blocked: no damage, no hit reaction; swings from behind do nothing at all |
| Swings while the Brute's defence hold is above 0 | Cannot be blocked: hits as normal |
| Power attack | Never blocked |
| Guard (L1) against slot 0 | Blocks it (hero facing the Brute), the Brute roars with 75 percent per tick of the blocked state; slot 2 and the grab are unaffected |
| Roberta's shots | 74 of 100 blocked when the path meets the body |
| Stays at 13 to 17 units in front | Chooser starts an attack 57 percent per tick; the hero is hit by whichever lands first |
| Runs away | The Brute hunts at 13.54 per second (the hero walks at 13.5, runs at 19.3) |

#### Edge cases for implementers

- Slot 3 and slot 1 do not damage by a swing: slot 3 is only the 148-tick roar (the Brute is in outer 2 and cannot start anything else), slot 1 deals no damage at all; the throw only moves the hero.
- The slam's window never closes once open (the clip ends on its last key with the bit set): the hit list allows one hit per window, so one hit.
- Each slam costs 229 ticks of sequencer time even when it misses; with recovery up to 120 ticks the Brute can be attack-free for 350 ticks.
- A Brute that is blocking is in base state 20; the hero's non-power swing from behind is silently dropped, so stunlocking from behind works only for power attacks.
- Weights: the Brute's attack-manager weight is 12; with the starting gear (capacity 0) a second enemy in agro waits in line while the Brute is listed first.

#### What a port most likely gets wrong

1. Letting the hero hit a hidden Brute (it is invisible and untouchable in states 5, 6, 7, 8, 21 before clip 82).
2. Treating the Brute as a rusher. It holds between 9 and 17 units and attacks from range 10 to 12 after a short approach; back-stepping is slow (6.05 per second).
3. Giving the roar damage or a cooldown. Slot 3 is a 148-tick pose with a zero cooldown and no damage; it is chosen first at 9 percent per tick.
4. Making the slam hit like a swing at the club's reach. The window is weapon 3 (radius 20, height 20) and opens at s+96, with 100 damage and a knock of 14.
5. Always blocking. Row 169 blocks 75 of 100 swings, the other rows 25 of 100, once per swing, with a quiet period of 30 to 240 ticks, and a swing from behind never connects while it blocks.
6. Skipping the missing wind-ups. Slot 0 and 2 have only strike phases (slot 2 also a recovery), so a swing starts when the Brute is in range.

#### Test vector

Row 170 Brute, already in agro (t = 0 the setter tick), hero standing still 13.0 units in front, in front of the Brute. t = 1 set-up; the first choice from t = 2 at 57 percent per tick: 9 percent roar (slot 3), else 19 slam (slot 2), 24 grab (slot 1, needs d from 10 to 20 and arc 0.966), 24 club (slot 0). Take slot 0 at c = 2: the Brute closes from 13.0 to about 10.0 at 0.2257 per tick (14 moves), s = c + 15 or 16. Row 170 exact no-stop contact at a fixed level target d=10 is s+44; the live approach's exact final separation must be retained to choose the matching contact sample. A hit deals 25 plus the poison roll, then the clip finishes at speed 1.0: gate s+94 and end path s+95 for the hit on s+44; slot 0 cooldown redrawn int[60, 180]. Take slot 2 at c = 2: closes 3 units, s about c + 16, slam hit at s + 96 for 100 (knockback 14.0), end path s + 229. Roar chosen at c = 2: s = c + 1, end path s + 149. Over 600 ticks the Brute makes 2 to 4 attack choices (typically 3, about 1 in 11 of them a roar); a hero who stays loses up to 25 or 100 in each (the grab deals no damage).

#### Open questions

- Resolved (enemy sweep): grab timing. The strike state first runs on s + 1, where the choose step seizes the hero and requests clip 40; the release event is seen 92 ticks later: grab on s + 1, throw on s + 93, C gate s + 137, end path s + 138 (no recovery clip with the grab flag). A failed choose step: recovery clip 42 on s + 2, end path s + 80.
- Resolved: the thrown hero takes no damage; the grab path deals 0 damage on every row (bog and sewer creatures, Swamp Tentacle grab).
- The slam volume's geometry (weapon 3 radius 20, height 20 centred on the anchor of attachment point 2). Resolved by the Weapon reach table: the volume points forward and about 24 degrees down and touches a level hero from 12.4 behind to 11.8 ahead of the Brute, with the hero's feet from 22 below to 14 above the Brute's origin. Default if unresolved (rewrite choice): hit a level hero within those limits.
- Resolved (Medium-High; proactive (open close-out)): a hero shot is judged by the shot rules, not by the Blocking Angle (SPEC-0036, hero shot against a guarding target), so there is no facing test: a shot whose path meets the body is blocked 74 of 100 from any side (the missile-defence roll above).

### Minion Warrior (rows 204 to 210)

Row 204 is the summon of the Darkest Faerie (SPEC-0054, Minion AI: Minion Warriors, with the full swing and charge arithmetic). This guide adds the field behaviour and the deltas of rows 205 to 210.

#### Identity and tunables

| Item | Value |
|---|---|
| Records | Monster type 82, variants 0 to 6 (rows 204 to 210), name "Minion Warrior". Clip tables: one set of tables for variants 0 to 5 (they differ only in sound words), a different table for the one row-210 record (Edge cases). Placements: row 204 38 (16 summon-reserved, key 2864), 205 8, 206 9, 207 7, 208 8, 209 36, 210 1 |
| Class and approach | Class 17; approach routine 4 (strafe); Can fly 0 |
| Hit points, body | 100; radius 1.25, height 3.0, weight 5.5; attack-manager weight 12; hostile; Display in default 0; wait-in-line radius 10 |
| Notice | View cosine 0.92388, awareness 80, timer 30 (cycle 32), near range 10, far range 30, chance 90 at the near end and 30 at the far end; search timer 120 to 240 (unused); socialize flag 1 (below) |
| Turn | Hold aim weight 1.0 |
| Agro bands | Strafe band 12.0 to 20.0, desired distance 12 + 8 f (13.6 to 18.4) |
| Recovery after an attack | int[15, 60] |
| Defence | Missile hold 60, dodge 100 (99 of 100), block 0; melee block 60 at or below (60 of 100) for all swing classes, then dodge 59, 39 or 29 of 100 for quick, normal, heavy; defence timer after a defence 0 to 0 |
| Guard | Blocking Angle 0.92388 (same rule as the Brute) |
| Other | Damage Reduction 0, no steal, no projectiles; weapons 1 to 3 radius 3.0 height 2.0 |

Attack slots (slot 3 unused), identical on every row:

| Slot | Kind | Band | Chance | Arc | Range | Delay | Damage, knockback | Counter |
|---|---|---|---|---|---|---|---|---|
| 0 | melee | 3 to 10 | 60 | 0.9239 | 4.0 | int[30, 60] | 7, 1.0 | trigger 5 (blocked, L1 held), chance 25 |
| 1 | charge kind 2 | 10 to 17 | 50 | 0.966 | 8.0 | int[180, 300] | 7, 3.0 | none; charge distance 8 to 20 |
| 2 | melee flurry | 3 to 10 | 40 | 0.707 | 4.0 | int[120, 240] | 15, 3.0, unblockable | trigger 6 (hit reaction), chance 25 |

Differences between rows 204 to 210: only the element T (1, 2, 3, 4, 5, 6, 7) and no status on any slot; the status fields are all 0, so a Warrior's hit applies no status on any row; row 210 has T 7 (matched only by Nova). Rows 208 to 210 also differ in an unused word.

#### Animation ids and clips (rows 204 to 209)

Locomotion: strafe forward 28 -> clip 3 (6 keys, loops, 12.5 units per second), back 32 -> clip 4 (6 keys, 12.5 per second backward), sidesteps 29 and 30 -> clip 36 (6 keys, 12.5 per second sideways), hold 31 -> clip 2 (8 keys), walk 2 and 4 -> clip 37 (10 keys, 6.0 per second), idle 0 -> clip 0 (20 keys), sentry stand 7 -> clip 22 (40 keys, loops), sentry fidget 8 -> clip 23 (68 keys, one-shot, gate 268). Attacks: slot 0 B 34 -> clip 6 (13 keys, bit 0x1 on keys 7 to 9), D 36 -> clip 7 (10 keys); slot 1 B 39 -> clip 32 (6 keys, loops; the run, 12.5 per second), C 40 -> clip 11 (18 keys, bits 0x3 on keys 6 to 8), D 41 -> clip 9 (11 keys); slot 2 B 44 -> clip 12 (30 keys, bit 0x1 on keys 4 to 6 and 19 to 22, bit 0x2 on keys 11 to 14). Reactions: hit 57 -> clip 21 (13 keys, gate s+49), melee dodge 55 -> clip 19 (10 keys, gate s+37), missile dodge 53 and 54 -> clip 20, block 56 -> clip 13 (15 keys, loops), block-hit cue 126 -> clip 14 (10 keys), death 58 -> clip 17 (27 keys, gate s+104). Field clips: look 16 -> clip 26 (43 keys, gate 168), lose interest 17 -> clip 27 (18 keys, gate 69), search 18 -> clip 35 (23 keys, gate 89), appear 82 -> clip 5 (20 keys, gate 77), disappear 81 -> clip 18 (20 keys, gate 77), emerge 10 -> clip 5. Absent: 33, 35, 37, 38, 42, 43, 45 to 47 (phase ids) and the dodge shortcuts of the flyers.

#### Wake and start

Shared rules. The Warrior's own numbers: standing hero (F1 0.7) band edges 7.0 and 21.0; chance 90 at 10 and 30 at 30, 96 at 8; p per 32-tick cycle = 0.8 x (chance - 1) / 100 (0.76 at 8 units), halved outside the 22.5 degree cone; decloak with clip 82 (gate 77), agro at the gate, first choice two ticks later. A state-6 patroller notices while it walks and then stops where it is. State 8 wakes at the emerge distance x F1 (40 with a camera cue on 4 records, 20, 15, 10, 0); the emerge clip is 10 (gate 77). Socialize: all Warrior rows have the socialize flag with talk range 120 to 180 ticks, quiet range 420 to 480 and a peer test using row words 20.0 and 5.0; its effect on wanderers is limited to sending one nearby enemy to the group alert wait (SPEC-0038 Behavior 10).

#### State table (agro)

| State | Action id and clip | Keys | Gate and exits |
|---|---|---|---|
| Hold (inner 10) | 31 -> clip 2 | 8, loops | hold timer int[60, 180], then hold, left or right circle; d > 20 gives approach; d < 12 gives back off |
| Approach (inner 7) | 28 -> clip 3 | 6, loops | 12.5 per second (0.2083 per tick); ends when d < desired |
| Circle left / right (inner 8, 9) | 30 / 29 -> clip 36 | 6, loops | 12.5 per second sideways; a wall contact ends it with a lock of 120 ticks |
| Back off (inner 11) | 32 -> clip 4 | 6, loops | 12.5 per second backward; ends when d > desired |
| Slot 0 swing | 34 -> clip 6 | 13 | window s+28 to s+39, hit at s+28; gate s+49; end path s+50 |
| Slot 1 charge | 39 -> clip 32, then 40 -> clip 11 | 6 (loops), 18 | run until d < 8.0 or the drawn distance; C requested on sC (the tick after the run ends); window sC+24 to sC+35, hit on sC+24 (sC+25 for a hero 8.0 ahead); gate sC+69; end path sC+70 |
| Slot 2 flurry | 44 -> clip 12 | 30 | hits at s+16, s+45, s+77; gate s+116; end path s+117 |
| Melee dodge (outer 5) | 55 -> clip 19 | 10 | gate s+37 |
| Missile dodge (outer 6) | 53 or 54 -> clip 20 | 10 | gate s+37 |
| Block (outer 7) | 56 -> clip 13 | 15, loops | while the hero swings; block-hit cue 126 -> clip 14 once |
| Hit reaction kind 0 (outer 10) | 57 -> clip 21 | 13 | gate s+49 |
| Dead | 58 -> clip 17 | 27 | gate s+104, 0.95 at s+100 |

#### Chooser, movement, windows, defence

Identical to SPEC-0054's Minion AI for row 204, with the field start replacing the summon start: strafe archetype (hold 31, forward 28, back 32, sidesteps 29 and 30 at 12.5 per second, lock of 120 ticks after a wall contact); slot test order 2, 1, 0 (39 percent, 49 percent, 59 percent per tick; slot 1 needs d from 10 to 17 and a route); slot 0 hit at s+28 for a hero up to 6.0 ahead (s+29 to s+32 from 6.5 to 10.5; gate s+49, end path s+50, 7 damage, knockback 1.0, blocked D at s+29 to gate s+66); slot 2 hits at s+16, s+45, s+77 for 15 each (unblockable; gate s+116, end path s+117); slot 1 runs clip 32 until the hero is within 8.0 or the drawn charge distance (8 to 20) is covered, then C (requested on sC): hit at sC+24 for 7 and knockback 3.0 (sC+24 for a hero up to 7.5 ahead when C starts, sC+25 at 8.0, sC+26 at 8.5 to 9.0), gate sC+69, end path sC+70. Hit reaction kind 0 (clip 57, 49 ticks); counters: slot 2 each tick of a reaction at roll below 25 (25 of 101); slot 0 after a blocked swing with L1. Melee defence is rolled once per hero swing at the start of the swing: block at or below 60, then dodge below 59, 39 or 29; no waiting time after a defence. Hero shots meeting the body inside 60 ticks are dodged in 99 of 100.

Movement table (rows 204 to 209; ground; radius 1.25; ground turn 1.0; strafe band 12 to 20):

| Mode | Source | Clip (keys) | u/tick | u/s | Turn | Exit |
| --- | --- | --- | --- | --- | --- | --- |
| Wander, patrol, walk home, wait in line | root | 2 or 4 (10) | 0.100 | 6.01 | steer | arrival 0.5, notice |
| Approach (inner 7), above 20 | root | 28 (6) | 0.208 | 12.50 | steer | d below the drawn desired 13.6 to 18.4 |
| Hold (inner 10) | none | 31 | 0 | 0 | aim 1.0 (snap) | after int[60, 180] ticks: stay, circle 8 or circle 9 |
| Circle (inners 8 and 9) | root, sideways | 30 (inner 8) or 29 (inner 9), 6 keys | 0.208 | 12.50 | snap to the hero (side gait) | hold count, the blocked flag (lock 120), d above 20, d below 12 |
| Back off (inner 11), below 12 | root, facing the hero | 32 (6) | 0.208 backward | 12.50 | steer, back gait | d above desired or blocked |
| Charge (slot 1) | root | 39 = clip 32 (6) at slot speed 1.0, then C | 0.208 | 12.50, then C lunges 8.22 over 72 ticks | none (straight) | within 8.0 of the hero or the drawn 8 to 20 covered |
| Hit (reaction kind 0) | root | 57 (13) | | back 6.08 over 52 ticks | snap to attacker | gate (49 ticks) |
| Dodge 53 / 54, 55 | root | (10) each | | side 7.67 over 40; back 7.97 over 40 | | gate |
| Return to the default state (state 23, clip 81) | root | 81 (20) | | back 8.81 over 80 ticks | kept | end, then the default state |
| Corpse | root | 58 (27) | | back 3.36 over 108 ticks | facing kept | timer 4 ticks: a held slide of about 2 ticks at 0.084 |

The Warrior's rush and keep-distance ids (20 to 26) are missing; they are never requested by its strafe archetype. The record of pack 1269 (developer showroom) has a different table and is not in play.
#### Weapon reach

Conventions are those of "Weapon reach tables" in the shared field behaviour. Rows 204 to 209, Medium. The hit ticks stated above are authoritative; the window columns below are the ticks on which the weapon volume is live.

| Slot (phase, clip) | Weapon | Window ticks (level contact) | Centre forward / up / side | Tor ahead, at the tick | Tor ahead, from the start | Tor feet vs origin |
|---|---|---|---|---|---|---|
| 0 (B, clip 6) | w1 (3.0 x 2.0) | s+28 to s+39 (s+28 to s+39) | 0.7 to 3.7 / -0.5 to 3.8 / -2.2 to 1.3 | -2.2 to 7.4 | 0.9 to 10.5 | -5.0 to 3.5 |
| 1 (C, clip 11) | w1 (3.0 x 2.0) | sC+24 to sC+35 (sC+24 to sC+35) | 0.7 to 3.7 / 0.6 to 3.4 / -1.5 to 0.4 | -1.4 to 7.2 | 2.4 to 11.2 | -5.0 to 6.5 |
| 1 (C, clip 11) | w2 (3.0 x 2.0) | sC+24 to sC+35 (sC+24 to sC+35) | 0.8 to 4.0 / 0.2 to 3.3 / -0.3 to 1.9 | -2.0 to 7.4 | 1.4 to 11.4 | -6.0 to 6.0 |
| 2 (B, clip 12) | w1 (3.0 x 2.0) | s+16 to s+27 (s+16 to s+27) | 0.0 to 2.4 / 0.5 to 2.3 / -2.4 to 1.9 | -2.4 to 6.0 | -0.7 to 7.7 | -4.5 to 3.5 |
| 2 (B, clip 12) | w2 (3.0 x 2.0) | s+44 to s+59 (s+44 to s+59) | -1.9 to 3.0 / 0.7 to 2.7 / -2.6 to 2.8 | -4.6 to 6.6 | -1.2 to 10.4 | -3.5 to 3.5 |
| 2 (B, clip 12) | w1 (3.0 x 2.0) | s+76 to s+91 (s+76 to s+91) | -0.6 to 2.7 / 0.6 to 2.3 / -2.0 to 2.3 | -3.2 to 6.4 | 2.3 to 12.2 | -4.0 to 4.0 |

#### Hero actions against this enemy

| Hero action | What the Warrior does |
|---|---|
| Walks up to a hidden Warrior | Notice roll once per 32 ticks (p about 0.76 at 8 units); then 77 ticks of clip 82 before the first attack |
| Hits it during clip 82 | Doubled damage, agro at once |
| Swing from the front (inside 8.0) | 60 percent block (no damage), else dodge 59, 39 or 29 percent |
| Power attack | Not blocked or dodged |
| Roberta's shots | 99 percent dodged when they will meet the body |
| Stays at 10 to 17 units | Charge (7 damage, knockback 3.0) at 49 percent per tick, then melee flurries when it is close |
| Retreats beyond 20 | The Warrior walks in with clip 28 (12.5 per second) to 13.6 to 18.4 units |

#### Edge cases for implementers

- **The one row-210 record** (Western Brightvale, region 887, default state 5, key 2866) has a different clip table: it has no strafe forward 28, hold 31 or back 32 (a request of a missing id starts clip 0, the idle loop, so it plays the idle clip and does not move), but it has 21 (clip 3), 23 and 26 (clip 4), 29 and 30; slot 1 has a 30-key wind-up A (id 38, clip 8, gate 116), no run clip B (so the charge goes straight to its end state) and a recovery E (id 42, clip 10, 25 keys, gate 96); slot 0 and slot 2 are the same as the others. Result: this Warrior circles and swings when the hero is in range but never approaches or backs off.
- The Illusen's Glade patrols (10 records on path group 4) and the Meridell Plains patroller (path group 8) are hidden while they walk; the scripted event in Illusen's Glade runs the invisibility-off command on named entities.
- A summoned Warrior (key 2864) starts in state 21 playing 82 (SPEC-0054); a field Warrior has a wake of its own.
- Row 206's patroller on group 8 pauses 120 to 480 ticks at each point.

#### What a port most likely gets wrong

1. Showing patrolling Warriors. They are invisible while hidden.
2. Giving rows 205 to 210 a status or a different damage; only the element changes.
3. Attacking at 20 units. The Warrior holds at 12 to 20 and attacks from 3 to 10 (melee) or charges from 10 to 17 only when the hero is within 15 degrees of dead ahead.
4. Applying the block roll as "below": it is at or below (60 of 100) while the dodge is strictly below (59, 39, 29 of 100).
5. Wake distance from the hero standing: a hero in state 1 shrinks the emerge distance to 0.7.

#### Test vector

Row 205 Warrior in state 7 (+0xC4 = 0), hero standing still 8.0 units away inside its 22.5 degree cone, hero in action state 1. The notice check can pass only on the tick the 32-tick awareness cycle shows 0 (the first such tick falls within the first 32 ticks); at 8 units the chance is 96, so p = 0.8 x 0.95 = 0.76 per cycle; on a pass the setter gives state 21 and the same tick's second roll (p again) requests 82, otherwise clip 16 (168 ticks) then clip 17 (69 ticks) unless a later cycle passes. Taking a pass at tick T with a second pass at T: 82 requested on T, agro at T + 77, set-up T + 78, first choice T + 79 at distance 8 (slot 2 39 percent, else slot 0 59 percent, so 75 percent per tick). The Warrior then closes the 4 units to its 4.0 reach at 0.2083 per tick (about 20 ticks), so the strike clip is requested near T + 101: slot 0 hits at about T + 129 (end path T + 151), slot 2 hits at about T + 117, T + 146 and T + 178 (end path T + 218); a Warrior that picks neither on T + 79 backs off to 12 to 20 units (clip 32 backward, 12.5 per second) and uses the charge. In 600 ticks after the decloak the Warrior makes 2 to 5 attack choices; damage per choice 7, 7 + 15 x 3 or 15 x 3. Charge: from 12 to 17 units at 49 percent per tick with the hero within 15 degrees of dead ahead: B (clip 32) at 12.5 per second until the hero is within 8.0, then C on the next tick (sC): hit 7 at sC + 24.

#### Open questions

- Resolved (High, flag checked on rows 204 to 210: flag 1, pair values 20.0 and 5.0; proactive (open close-out)): the Warrior rows carry the socialize flag, so the shared Socialize rules apply to them unchanged (peer test and pair radii as in Socialize above and SPEC-0038 Behavior 10; talk range 120 to 180, quiet range 420 to 480). The effect needs two Warriors in states 5, 6 or 7 within the pair distance, which is rare because most Warrior records start hidden. The earlier default of "no socialize effect" is withdrawn.
- The geometry of the weapon volumes (3.0 by 2.0 on attachment points 0 to 2). Resolved: the general rule of the shared field behaviour. The distances are in the Weapon reach table. Stand-in only for a port without attachment frames (rewrite choice): a window tick hits a level hero inside the "at the tick" range of the slot's row (for example 2.2 behind to 7.4 ahead for slot 0).
- First state-21 step on tick Q versus Q + 1 for summons (SPEC-0054). Default if unresolved (rewrite choice): Q.
- Whether the row-210 record's missing ids are an intended design or a leftover table. Default if unresolved (rewrite choice): reproduce the missing clips (the Warrior idles in place).


## Shadowglen Plants

Six monster rows make the five plant kinds of Shadowglen Woods, Shadowglen Burrows and a few farm-side areas: the Flower Monster (two builds: a walking melee plant, row 147, and a rooted spitter, row 149), the Poison Flower Monster (row 148), the Spore Monster (row 221), the Plant Gunner (row 220) and the Plant Monster (row 219). All six run the common-enemy think routine of SPEC-0038 (class 17). This section gives, for each kind, everything a rewrite needs on top of that spec: the row values, how the kind starts from each default state, which animation plays for each state, tick counts, the attack chooser, defence and counters, summons, death, test vectors and open points. Confidence: High for the row values, the clip tables, the attack arithmetic, the chooser, defence and counter rules, the summon rule and the start from state 8; Medium-High for the wander, patrol and sentry states (their steps are read from the routines but no area has been walked tick by tick); Medium for weapon reach geometry, the Plant Monster grab and the hidden Poison Flower Monster pose.

Time and notation are as in SPEC-0063: ticks are 1/60 s; s is the tick in which the attack sequencer requests a clip; "gate" is the number of ticks after the request tick on which the clip's 0.99 progress gate first passes; a phase whose animation id has no clip in the kind's table is skipped in the same tick; int[a, b] is a closed integer draw; a roll "below N" from int[1, 100] passes on N - 1 of 100 values (so "chance 50" is 49 %). Every plant clip has speed byte 2 (4 ticks per key at play rate 1.0) except the Plant Monster family's backward walk (speed byte 1, 2 ticks per key).

### Family facts shared by all five kinds

#### Rows at a glance

| Kind | Row | Type / variant | Spawn key | Clips in the set | HP | Approach | Body (radius / height) | Weight (attack manager) | Target type T | Death timer |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| Flower Monster, melee | 147 | 68 / 1 | 620 | 23 | 23 | 1 (rush) | 2.5 / 3.0 | 12 | 4 | 300 |
| Poison Flower Monster | 148 | 68 / 2 | 621 | 23 | 25 | 1 (rush) | 2.5 / 2.0 | 12 | 2 | 300 |
| Flower Monster, spitter | 149 | 69 / 0 | 619 | 7 | 16 | 3 (keep) | 1.0 / 5.0 | 1 | 2 | 10 |
| Plant Monster | 219 | 85 / 0 | 1621 | 24 | 50 | 1 (rush) | 3.5 / 7.0 | 5 | 4 | 300 |
| Plant Gunner | 220 | 85 / 1 | 1622 | 24 | 35 | 3 (keep) | 3.5 / 7.0 | 5 | 2 | 300 |
| Spore Monster | 221 | 85 / 2 | 1623 | 24 | 35 | 1 (rush) | 3.5 / 7.0 | 5 | 2 | 300 |

The spawn key is the character kind key in the placement record (+0xAC). The Spore Monster's lobbed summon asks for key 619, the spitter build of the Flower Monster. The melee build has key 620 and no summoner ever asks for it. All six rows: class 17, hostile, Display in default 1, can fly 0, no map search, no anger rule, no ghost phase, no reanimation, no distance hide, no socialize timers, Damage Reduction 0, steal fields unused, drop fields as the placement says. The animation table of every record has 166 entries. A target type of 4 makes the kind immune to stun (the Water Mote), a type of 2 makes it immune to poison (the Leaf Mote); neither matters for the hero's plain swings.

#### Notice, search and defence values

| Kind | View cone cosine | Awareness percent | Awareness timer (ticks) | Notice band near / far | Chance at near / far | Sight-line test | Search timer (ticks) | Turn weight of the hold |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| Flower Monster, melee (147) | 0.707 | 75 | 30 | 3.5 / 50.0 | 70 / 10 | required | 0 to 0 | 1.0 |
| Poison Flower Monster (148) | 0.707 | 75 | 30 | 3.5 / 50.0 | 70 / 10 | required | 0 to 0 | 1.0 |
| Flower Monster, spitter (149) | 0.707 | 100 | 60 | 1.0 / 10.0 | 90 / 10 | not needed | 0 to 0 | 1.0 |
| Plant Monster (219) | 0.9239 | 100 | 30 | 5.0 / 100.0 | 70 / 25 | required | int[60, 239] | 0.7 |
| Plant Gunner (220) | 0.9239 | 100 | 30 | 5.0 / 100.0 | 70 / 25 | required | int[60, 239] | 0.7 |
| Spore Monster (221) | 0.9239 | 100 | 30 | 5.0 / 120.0 | 70 / 25 | required | int[60, 239] | 0.7 |

| Kind | Missile look-ahead and defence hold (ticks) | Missile dodge / block | Melee block (quick, normal, heavy) | Melee dodge (quick, normal, heavy) | Defence timer after any defence |
| --- | --- | --- | --- | --- | --- |
| 147, 148 | 60 | 100 / 0 | 0, 0, 0 | 75, 50, 30 | int[60, 240] |
| 149 | 0 | 0 / 0 | 0, 0, 0 | 0, 0, 0 | 0 (no defence) |
| 219 | 60 | 25 / 0 | 0, 0, 0 | 90, 80, 75 | int[15, 45] |
| 220 | 60 | 25 / 0 | 0, 0, 0 | 40, 25, 10 | int[15, 45] |
| 221 | 60 | 25 / 0 | 0, 0, 0 | 40, 25, 10 | int[60, 120] |

Blocking Angle is 0 on rows 147 to 149 and 0.924 on rows 219 to 221, but no plant has a block chance above 0, so no plant ever guards and the angle never matters. No plant has an attack that can be blocked by anything but the hero's guard.

#### Where each kind is placed and what default state it starts in

The default-state word (record u16 +0x84) decides how a placed plant behaves until it meets the hero: 5 wander, 6 patrol, 7 sentry at its post, 8 hidden at home until the hero comes near. Counts below are the placements in shipped areas (the unreached test pack is left out). Placements come in pairs of regions (two copies of the same field for the story variants), so about half of each count is present in one play of an area.

| Kind | Hidden (8) | Wander (5) | Patrol (6) | Sentry (7) | Summon-reserved (start in agro) | Main places |
| --- | --- | --- | --- | --- | --- | --- |
| Flower Monster, melee (147) | 43 | 2 | 16 | 0 | 0 | Shadowglen Burrows 20 hidden and 12 patrol; Shadowglen Woods 17 hidden and 2 wander; Woods transition 6 hidden and 4 patrol |
| Poison Flower Monster (148) | 3 | 0 | 0 | 0 | 0 | Shadowglen Woods |
| Flower Monster, spitter (149) | 100 | 1 | 0 | 3 | 90 | Woods 79 hidden and 80 reserved; Steppe Plateau 12 hidden; Ellis Family Farm 5 hidden and 3 sentry; Farm Road 4 hidden, 1 wander and 5 reserved |
| Plant Monster (219) | 0 | 0 | 1 | 0 | 0 | Shadowglen Woods |
| Plant Gunner (220) | 0 | 0 | 6 | 0 | 0 | Shadowglen Woods |
| Spore Monster (221) | 0 | 2 | 6 | 9 | 0 | Woods 2 wander, 6 patrol, 8 sentry; Farm Road 1 sentry |

Every Spore Monster region also holds five summon-reserved spitter records (key 619, default word 5, no drop). They are bound only by the Spore Monster's lobbed shot (below) and then start in agro, not in state 5. The default word 5 of a reserved plant matters only after it has lost the hero.

Emerge distances of the hidden plants (record f32 +0xC0, in world units, before the hero-state factor): melee Flower Monster 10.0 on 24 records, 15.0 on 1, 20.0 on 10, 30.0 on 2 and 45.0 on 6; Poison Flower Monster 10.0 on all 3; spitter 10.0 on 2, 15.0 on 30, 20.0 on 12, 30.0 on 1, 40.0 on 25, 50.0 on 14, 60.0 on 12 and 70.0 on 4.

Alarm groups (record byte +0xB7, 255 = none): 32 of the 43 hidden melee Flower Monsters, 37 of the 100 hidden spitters and none of the Poison Flower Monsters or the other three kinds are in a group. Group members wake together (below).

#### Starting from each default state

All start states share the think loop of SPEC-0038 Behavior 1: at creation the loop runs the setter with 8 when the default word is 8 and otherwise with 0; the first pre-step then runs the setter with the default word. A record bound by a spawn-by-key request skips all of this and takes agro (class state 19) at once.

**State 8, hidden at home (all hidden plants).** Home is the record position (the character's placed position; it is rewritten from the character's position whenever the character is released alive by streaming, except for default state 7, see below). The think loop's state-8 handler: when the plant is within 0.35 of home it stands and requests the hide pose (animation 9, or 128 to 131 by the record's +0xC8 selector, which is 0 on every plant, so 9); otherwise it walks home with gait 0. A plant that was never moved is therefore always standing in its hide pose. The pre-step wake test runs every tick: sub-state 0 compares the horizontal-or-3D distance to the hero (open question 5) with the emerge distance times F1, F1 being the hero's action-state factor (SPEC-0038 Behavior 18: 0.7 for the idle state 1, 0.75 for states 2 and 5, 1.5 for 3 and 9, 1.75 for 4, 6 and 10, 0.5 for 21, 22 and 41, 0.3 for 23, and 1.0 for any other state). Below the threshold it sets sub-state 1. On the next tick sub-state 1 requests the emerge animation (id 10, 165 for a flyer, or 133 to 136 by the selector, which is 0 here): if the table has no clip for it the plant takes agro on that same tick, otherwise it plays the clip and sub-state 2 takes agro when the clip's gate passes. Let a be the tick of the distance test. The request is on a + 1, class state 19 on a + 1 + G (G is the emerge gate), the first agro-machine tick on a + 2 + G (it only sets the outer sub-state) and the first normal attack choice on a + 3 + G (Medium-High, plus or minus one tick). A hit on a hidden plant skips all of this: the take-hit doubles the damage, takes agro and starts hit reaction kind 0 at once. A group wake (below) also skips the distance test.

Emerge gates: melee Flower Monster: animation 10 is clip 18, 35 keys, G = 136. Spitter: animation 10 is clip 2, a one-key loop, G = 3 (so it is in agro 4 ticks after the test). Plant Monster, Plant Gunner and Spore Monster are never hidden. The Poison Flower Monster has no clip for animation 10: it takes agro on a + 2.

**State 5, wander.** The plant picks a goal, walks to it, optionally rests, and repeats. Every tick: a rest timer (T1) and a fidget counter (T2) are counted down as below; if no goal is held, one is chosen: when the record's +0xBC float is below 0.1 the goal is the centroid of a random triangle of the plant's navmesh; otherwise the goal is home + (u, 0, v) with u and v each an integer drawn from 0 to trunc(+0xBC) - 1 minus half of +0xBC, accepted if it lies on the navmesh, else moved half the way toward the plant (up to 10 tries, then home). While T1 is below 1 the plant walks toward the goal (gait 0 of state 5: animation 2) and tests arrival: the horizontal distance to the goal below 0.5, or the blocked flag with the stuck counter at 120, or no route. On arrival the goal is dropped and a roll int[1, 100] below the record's +0xC8 starts a rest: T1 = int[+0xC0, +0xC4] and T2 = a second draw of the same range (both words are read as integers here; the order of the two ends does not matter). While T1 is 1 or more the plant stands: it plays the idle pick (animation 0) and, whenever T2 has run out and the table has animation 3, plays animation 3 once as a fidget, then redraws T2. So a rest is idle, fidget, idle, fidget until T1 ends. With +0xC8 = 0 (every wander record except two) the roll never passes: the plant walks from goal to goal without stopping. Values: melee Flower Monster 2 records, +0xBC = 128.0 (a square 128 on a side around home), no rests; spitter 1 record, +0xBC = 50, rests with chance 4 % (+0xC8 = 5) of int[30, 60] ticks; Spore Monster 2 records, +0xBC = 10, rest chance 49 % (+0xC8 = 50) of int[60, 300] ticks. Wander speed is the root travel of animation 2: 2.97 units per second for the melee Flower Monster (clip 0), 3.92 for the Plant Monster family (clip 1); the spitter's animation 2 is a one-key pose with no travel, so a spitter in state 5 never reaches a goal (it stands in place).

**State 6, patrol.** The record's +0xB8 selects a path of the area (a path of the area's path list, SPEC-0006, Tagged tables); when it is outside the list the plant sets no route and steers toward entry 0 of the pack's entry list without advancing (Medium-Low; SPEC-0056, Path walk). The plant walks the path points in order (gait of state 6: animation 4), arrival tolerance 0.5 horizontal. The next point is the following one; at the last point a record with +0xBC = 0 wraps to point 0 (all shipped patrol records), while a non-zero +0xBC would turn the plant around (ping-pong). On arrival the plant pauses for int[+0xC4, +0xC8] ticks (a second draw of the same range is the fidget counter) with the same idle and fidget pattern as state 5 but the fidget animation is 6. Mid-walk pauses (chance +0xCC per awareness-cycle tick, length int[+0xD0, +0xD4]) exist in the routine but every shipped record has +0xD0 = +0xD4 = 0, so they never pause the plant. Pause ranges on disc: melee Flower Monster: no pause on 12 records, int[0, 5] on 2, int[3, 10] on 2; Plant Monster and Plant Gunner: int[120, 480] on 5 records and int[0, 240] on 2; Spore Monster: int[120, 480] on all 6. Patrol walk speed is the root travel of animation 4: 2.97 units per second (melee Flower Monster), 3.92 (Plant Monster family).

**State 7, sentry.** The plant stays at its post: if it is more than 0.5 from home it walks home (gait 0, animation 2) and does nothing else. At the post, a record whose +0xC4 is 0 (all shipped sentries) stands in animation 7 and counts a timer down; when the timer has run out it plays animation 8 once and then redraws the timer from int[60, 600] ticks and returns to animation 7 (the timer is 0 at creation, so a fresh sentry looks first). A non-zero +0xC4 would instead turn the plant toward the record's facing and hold animation 7. Sentries are the one kind of placement whose release keeps the disc position: a sentry that has been streamed out and back in is bound again at its record position.

**Notice from states 5, 6 and 7.** Each pre-step the plant runs the notice check of SPEC-0038 Behavior 3: one tick of each awareness cycle (the countdown is 0; the cycle is the awareness timer plus 2 ticks, so 32 ticks for the 30-tick rows and 62 for the spitter) draws roll A (at most the awareness percent) and, inside the band, roll B (below the interpolated chance, halved outside the view cone). The plant takes class state 2 (alerted search) on that tick. State 2 runs its handler on the same tick and repeats the notice test once more with fresh rolls (the awareness countdown has not moved yet). If that second test passes, the plant plays animation 18 when its table has a clip for it (melee Flower Monster: clip 17, 30 keys, gate 116, it hops forward about 3.8 units; spitter: clip 1, a 5-key loop, gate 19) and takes agro when that clip's gate passes (or at once when the table has no clip for 18: Plant Monster, Plant Gunner, Spore Monster). If the second test fails, the search timer decides: for rows 147 to 149 it is 0, so the plant is sent back to its default state on that very tick (the notice was a false start); for rows 219 to 221 it counts int[60, 239] failing ticks, during which the plant stands (animation 16, absent from the table, so the idle pick) and faces the last-seen point, and any later passing notice test takes agro at once; when the timer runs out the plant returns to its default state.

**Alarm groups.** Entering agro (by notice, wake or hit) posts a broadcast for the plant's group when the group byte is not 255. Every group member checks for a new broadcast on a timer of int[15, 45] ticks. A member in default-mode states 0, 3, 5, 6, 7 or 14 takes state 2 and requests animation 18 (the plant then plays that request's clip, or clip 0 of its set when the table has none, waits for its gate and takes agro); a member in state 8 plays the emerge animation 10 (taking agro at once when the table has none) and skips the distance test. So hitting or waking one plant of a group wakes the rest within 15 to 45 ticks (+ the emerge gate). Members of one group are not tied by distance. Groups on disc: melee Flower Monster groups 0, 1, 3, 7, 10, 11, 12 (the largest group, number 10, has 10 members in each of the two Burrows copies); spitter groups 1, 2, 4, 7.

**Returning to the default state.** Leaving agro always runs the setter with 0 and then the default word: the leash (no route to the hero for 180 ticks with the walk waypoint 15 or more units from the hero), the lost-sight search (more than 120 ticks of search at the last-seen point with the sight line still blocked), and the search timeout of state 2. A hidden plant then walks home with gait 0 and, at 0.35 from home, takes its hide pose again; a wanderer, patroller or sentry goes back to its routine from wherever it stands. A plant that is released alive by streaming (hero farther than 280 units) is bound again at the position and facing it had, at full health, except the sentries, which return to their disc position.

#### Agro in one paragraph

All six kinds use the agro machine of SPEC-0038 Behavior 44 unchanged. The rush plants (147, 148, 219, 221) enter outer sub-state 1 with inner 1 (hold, faces the hero with the row's turn weight, animation 22); the keep-distance plants (149, 220) enter outer 3 with inner 4 (hold, aim blend 0.25, animation 25). The normal attack choice runs every agro tick in outer 1, 3, 4 and 11 with the recovery countdown below 1, and the counter choice runs every tick in every outer sub-state. The think-loop details that differ per kind are in the kind sections. Movement is only the root motion of the animation that plays, so a kind whose walk clips carry no root travel does not move (the spitter) and a missing animation id plays clip 0 with its root travel.

#### Death and drops (all kinds)

A killing hit sets base state 16; the death animation (58) plays once; the corpse stays until its death countdown (row +0x80) has run out after the clip passes progress 0.95 and then the streaming block releases it. Non-flying corpses sink 1/180 unit per tick once the countdown is more than 30 ticks into its run. Coin and item payout follows the placement's drop ordinal (record +0xB0; SPEC-0021) on the first death tick; summon-reserved spitters have ordinal -1 and pay nothing. A killed record is remembered and never rebound (the plants have no reanimation).

| Kind | Death animation | Clip, keys, gate (0.99 / 0.95) | Total ticks from the killing hit until the corpse is released |
| --- | --- | --- | --- |
| 147, 148 | 58 | clip 8, 30 keys, 116 / 112 | about 413 |
| 149 | 58 | clip 4, 3 keys, 9 / 9 | about 20 (the 10-tick countdown ends before the sink starts) |
| 219, 220, 221 | 58 | clip 13, 33 keys, 128 / 124 | about 425 |

### Flower Monster

Two different enemies share this name. The melee build (row 147, spawn key 620) walks and fights in the Shadowglen Burrows and the Woods (Medium-High to High throughout). The spitter build (row 149, spawn key 619) is a rooted plant that lobs a slowing seed; it is the plant the Spore Monster summons and the most common enemy in the game (195 records, 90 of them reserved for summoning). The record's key and variant tell them apart; the screen name is the same. The two builds are given one after the other.

#### Flower Monster, melee build (row 147)

**Identity and tunables.** Type 68, variant 1; clip set of 23 clips shared with the Poison Flower Monster; the other values are in the family tables above (HP 23, radius 2.5, height 3.0, weight 12, target type 4). Rush band: hold while the distance is between 4.0 and 17.0; walk when above 17.0 (gait 2 above 34.0, the same clip as gait 0); back off when below 4.0 to the point 19.0 units behind the hero. Desired distance 4.0 + 13.0 f with f = int[20, 80] / 100 (6.6 to 14.4), drawn at creation and after every normal choice. Recovery after an attack int[15, 30]. Turn weight in the hold 1.0 (the plant faces the hero exactly on the first hold tick). Weapon volumes: weapon 1 radius 3.0 length 1.0; weapons 2 to 5 radius 5.0 with lengths 2.0, 3.0, 2.0, 2.0.

| k | Type | Band (strict) | Chance, per tick | Facing cosine above | Range | Cooldown draw | Speeds (wind-up, strike, recovery) | Damage / knockback | Status | Counter |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| 0 | melee, no use | 0 to 0 | 0 | 0.9239 | 13.0 | int[60, 120] | 1, 1, 1 | 7 / 0 | none | none |
| 1 | melee claw lunge | 0 to 20 | 50, 49 % | 0.9914 | 10.0 | int[120, 300] | 1, 1, 1 | 5 / 0 | none | none |
| 2 | melee slam | 0 to 9 | 50, 49 % | 0.0 | 7.0 | int[60, 240] | 1, 1, 1 | 10 / 0 | none | code 6, chance 50 |
| 3 | unused | | | | | | | | | |

Slot 0 can never be chosen: its band 0 to 0 is empty (the test is strict at both ends) and its chance is 0. Its animation (clip 1, 23 keys) exists but is unreachable. Counter code 6 on slot 2 fires during hit reactions.

**Animations (this kind's table).** Idle 0 is clip 12 (30 keys, loop). Walk in the wander and patrol states (ids 2 and 4) is clip 0 (15 keys, loop; 2.97 units per second). Rush walk 21 is clip 11 (13 keys, loop; 12.57 units per second, 10.89 units per cycle). Rush hold 22 is clip 6 (20 keys, loop, no travel). Back off 23, and the melee dodge 55, are clip 13 (13 keys, loop; 4.71 units per second backward, 4.08 units per cycle, gate 51). Missile dodge 53 and 54 are clip 14 (13 keys, one shot, gate 49; it side-steps 2.9 units). Hit reaction 57 is clip 10 (15 keys, gate 57; the plant is pushed 9.07 units backward over the clip). Death 58 is clip 8 (30 keys, gates 116 and 112). Emerge 10 is clip 18 (35 keys, no travel, gate 136). Hide 9 is clip 7 (one key). Search 16 is clip 16 (50 keys), alert 18 is clip 17 (30 keys, hops forward 3.77 units, gate 116). Fidget 3 is clip 9 (50 keys, gate 196). Victory 73 is clip 9. Ids 63 and 64 are clips 20 and 22.

**Start and wake.** See the shared section. Hidden records (43): the emerge distance is 10, 15, 20, 30 or 45 times F1, the emerge animation is clip 18 (G = 136), and the plant enters agro on a + 137. Patrol records (16, Burrows and the Woods transition) walk their path at 2.97 units per second and notice the hero as in the shared section: false starts are common because the search timer is 0 (the second notice test, same tick, fails 25 % of the time even inside the near edge). The two wander records (Woods, +0xBC = 128) roam a square 128 units wide at the same speed and never rest.

**Per-tick order.** As SPEC-0038 (Behavior 44, steps 1 to 5). Normal choice tests slot 2, then slot 1 (slots 3 and 0 never pass). The counter choice runs after it every tick, so during a hit reaction (outer 10) slot 2 gets a roll every tick (below).

**State table.** All gates are ticks after the request tick at play rate 1.0.

| State | Animation id -> clip | Keys | Gate | Exits |
| --- | --- | --- | --- | --- |
| Hidden (class 8) | 9 -> clip 7 | 1 | none | wake test, hit, group wake |
| Emerge | 10 -> clip 18 | 35 | 136 | agro |
| Rush hold (outer 1, inner 1) | 22 -> clip 6 | 20, loop | none | d below 4.0 to inner 2; d above 17.0 to inner 0 |
| Rush walk (inner 0) | 21 -> clip 11 | 13, loop | none | d below desired to inner 1 |
| Back off (inner 2) | 23 -> clip 13 | 13, loop | none | d above desired to inner 1 |
| Slot 1 claw | A 38 skipped; B 39 -> clip 3 (11 keys, travels forward 4.09 units); C 40 -> clip 4 (6 keys, 1.09 units) after a hit; D 41 -> clip 2 (15 keys, backward 9.09 units) when blocked; E 42 skipped | | B 41, C 21, D 57 | end path |
| Slot 2 slam | A 43 skipped; B 44 -> clip 5 (18 keys, travels forward 6.56 units); C, D, E skipped | | B 69 | end path |
| Melee dodge (outer 5) | 55 -> clip 13 | 13, loop | 51 | defence exit |
| Missile dodge (outer 6) | 53 or 54 -> clip 14 | 13 | 49 | defence exit |
| Hit reaction (outer 10, kind 0) | 57 -> clip 10 | 15 | 57 | outer 0 |
| Alert (state 2 after notice or group wake) | 18 -> clip 17 | 30 | 116 | agro |
| Dead | 58 -> clip 8 | 30 | 116 (0.95 at 112) | corpse released after 413 ticks |

**Attack chooser.** Slots are tested from the highest down and the first pass wins (SPEC-0038 Behavior 7): slot 2 when 0 < d < 9, the plant is facing the hero (cosine above 0), the slot cooldown int[60, 240] is spent and a roll int[1, 100] is below 50; then slot 1 when 0 < d < 20, facing cosine above 0.9914 (about 7.5 degrees), cooldown int[120, 300] spent and a second roll below 50. Both pass the charge rule (a route exists, or the slot is melee and d is under 10). Per tick from the hold band: d under 9: slot 2 49 %, else slot 1 about 25 % (74 % in all); d from 9 to 17: slot 1 49 %. Both slots then walk to their range first (state 0 pursues with gait 0 while d is above 10.0 for slot 1 or 7.0 for slot 2), so a claw chosen at d = 17 begins about 33 ticks later, and a slam chosen at d = 8.5 begins about 8 ticks later.

Claw (slot 1), s = first tick of state 1: B 39 requested on s. The weapon 3 window (B keys 8 to 10) is open on ticks s + 32 to s + 40 (the clip's gate passes on s + 41). One hit, 5 damage, on the first touching tick h: for row 147 and a fixed level hero straight ahead, exact samples give h=s+32 at start distances 1.0–7.0 and h=s+36 at 10.0 (no chase stop). The continuous interval and any non-level/hover poses are not established between samples. A hit goes to state 3, where C is requested on h + 1, its gate passes on h + 22 and the end path runs on h + 23 (s + 55 for h = s + 32, s + 59 for h = s + 36). A miss ends on s + 42. A hero guard blocking at facing dot at most -0.5 plays D (gate 57) and the strike still deals its reduced damage.

Slam (slot 2): B 44 requested on s, weapon 4 (B keys 8 and 9) open on s + 32 to s + 39, one hit of 10 on s + 32 for a hero up to 8.5 ahead (s + 33 at 9.0, s + 34 at 9.5 to 10.0, s + 36 to s + 38 at 10.5 to 11.5, no touch beyond); with or without a hit the clip's gate passes on s + 69 and the end path runs on s + 70 (there is no C, so a hit just waits for the gate).

The end path sets outer 0, the recovery countdown int[15, 30] and redraws the slot cooldown. The plant moves while striking by the root travel of B: the claw carries it forward about 4.1 units (it stops stepping inside the sum of the radii while in sequencer states 0 to 3, SPEC-0038 Behavior 48), the slam about 6.6.

**Movement.** Rush outer 1 only: inner 1 holds and turns to the hero at weight 1.0; the plant walks at 0.2095 units per tick (12.57 per second); the first step comes one tick after the request. Back off walks backward at 4.71 per second while facing the hero. It never strafes. Wander and patrol speed 2.97 per second.

Movement table (row 147; ground; radius 2.5; ground turn 1.0; rush band 4 to 17, desired 6.6 to 14.4):

| Mode | Source | Clip (keys) | u/tick | u/s | Turn | Exit |
| --- | --- | --- | --- | --- | --- | --- |
| Wander, patrol, walk home; any missing id (20, 24 to 32, 127) | root | 2, 4 or clip 0 (15) | 0.0495 | 2.97 | steer | arrival 0.5, notice |
| Approach (inner 0) | root | 21 (13), 0.127 to 0.298 per tick inside the cycle | 0.2095 | 12.57 | steer | d below desired |
| Hold (inner 1) | none | 22 (20) | 0 | 0 | aim 1.0 | d below 4 or above 17 |
| Back off (inner 2), below 4 | root, facing the hero | 23 (13); the keys swing between 0.13 forward and 0.28 backward per tick | 0.0785 backward | 4.71 | steer, back gait | d above desired |
| Hit (reaction kind 0) | root | 57 (15) | | back 9.19 over 60 ticks | snap to attacker | gate |
| Dodge 53 / 54, 55 | root | (13) each | | side 2.91 over 52; back 4.08 over 52 | | gate |
| Confused (outer 11) | root | 69 -> clip 19 (55, one-shot; frame-stored root, mode 0) | 0 to 0.042 inside the clip, then 0.0836 held | 6.14 forward over 220 ticks (1.67), then 5.01 while held | aim at a random direction 0.05 | status end |
| Corpse | root | 58 (30) | | back 0.87 over 120 ticks, nothing held | kept | timer 300 |

**Hit and hurt windows.** Attack windows as above. The plant is hittable in every state except the dodges (outer 5 and 6) and base state 16. The hit-reaction clip restarts on every hit; each restart pushes the plant back by the clip's root travel (9.07 units by the 57-tick gate, 9.19 over the whole clip), so a chain of fast hits drives it backward without letting the clip finish.

**Weapon reach.** Conventions are those of "Weapon reach tables" in the shared field behaviour. Medium. The window columns are computed from the clips; where the timeline above names a window or a hit tick, that text is authoritative. The claw uses weapon 3 (radius 5.0, height 3.0) and the slam weapon 4 (radius 5.0, height 2.0), not weapon 1.

| Slot (phase, clip) | Weapon | Window ticks (level contact) | Centre forward / up / side | Tor ahead, at the tick | Tor ahead, from the start | Tor feet vs origin |
|---|---|---|---|---|---|---|
| 1 (B, clip 3) | w3 (5.0 x 3.0) | s+32 to s+41 (s+32 to s+41) | 3.2 to 10.2 / 1.6 to 2.6 / -0.5 to 0.1 | -1.2 to 12.4 | -1.3 to 15.4 | -5.5 to 7.5 |
| 1 (C, clip 4) | w3 (5.0 x 3.0) | sC+1 to sC+7 (sC+1 to sC+7) | 3.8 to 11.2 / 1.5 to 2.5 / -0.1 to 0.2 | 1.8 to 15.4 | 2.5 to 15.4 | -6.5 to 7.0 |
| 2 (B, clip 5) | w4 (5.0 x 2.0) | s+32 to s+39 (s+32 to s+39) | 0.5 to 0.9 / 1.5 to 2.5 / -0.7 to -0.5 | -4.6 to 6.6 | -2.0 to 11.5 | -5.5 to 5.5 |

The claw reaches far: the weapon swings out to 10 to 11 ahead of the plant, so a level hero is touched up to 12.4 ahead on the strike clip and up to 15.4 on the follow-up clip after a hit. The slam stays close to the body (the weapon is 0.5 to 0.9 ahead) and touches 6.6 ahead at most at the tick (11.5 measured from where the plant started, after its own travel). Slot 0 has no table because it can never be chosen.

**Defence and counters.** Melee dodge: once per hero swing, on the first plant tick that sees the swing (not in outer 10, in outer 2 only in sequencer states 0, 1 and 5), when the defence hold is below 1, the plant is within 8.0 units and 60 degrees of the hero's facing. The block roll never passes (chance 0). The dodge roll passes with 74 % (quick Tor sub-states 5 and 6), 49 % (normal 2, 3, 9) or 29 % (heavy 4, 7, 8, 12). On a dodge the defence hold is 60 ticks, clip 13 plays for 51 ticks (the plant backs 4 units, unhittable) and the exit sets the hold to int[60, 240]. Missile defence: any live hero shot whose next 60 ticks of path touch the plant's body is dodged with chance 99 % (first roll below 100), the hold is 60 and the exit sets int[60, 240]. Counter code 6 (slot 2): during every tick of a hit reaction (57 ticks) the counter choice finds slot 2 eligible when 0 < d < 9, the plant faces forward and the slot cooldown is spent, then rolls int[0, 100] below 50 (50 of 101): the slam starts at once, interrupting the reaction. A hit on a plant within 9 units therefore almost always draws a slam (about 1 - 0.505^k over k ticks).

**Hero actions against this enemy.**

| Hero action | Result |
| --- | --- |
| Swing at a hidden plant | Hittable; damage doubled; agro at once; no emerge; no dodge. Starting sword (stat A 4): 8, so HP 23 leaves 15 |
| Tor swing at a plant in agro, in front, hold below 1 | Dodged 74 % (quick), 49 % (normal) or 29 % (heavy), else 4 damage (finisher 8), hit reaction kind 0 with the push-back, slam counter next |
| Tor swing while the plant attacks (sequencer 2 to 4) | Not defended; hits; the hit cancels the attack without the end path |
| Tor power step with a Mote | Never defended; Water (stun) does not work (target type 4); Leaf poison works |
| Roberta shot | 99 % dodged when the hold is below 1 and the shot path meets the body; the first shot after a dodge is hit-eligible only once the hold (60) has run down |
| Hero standing still | Wakes a hidden plant inside emerge distance x 0.7; notices from the wander and patrol states by the band of the shared table |
| Hero guard | Reduces the plant's strikes; a blocked claw plays D |
| Killing a plant | 23 HP; death clip 8; corpse released after about 413 ticks; payout by the placement ordinal |

**Edge cases for implementers.**
- Slot 0 has a full set of numbers (damage 7, range 13) and a clip, and must still never be chosen: the strict empty band decides.
- The claw's 0.9914 cone needs the exact facing of the hold: the plant turns at weight 1.0 each tick, so it passes on the second agro tick.
- Hit reaction kind 0 interrupts any attack with no end path; the only wait left is the slot cooldown drawn at the choice.
- Both slot cooldowns are drawn twice (at the choice and at the end path).
- A false-start notice (search timer 0) returns the plant to wander or patrol with the notice cycle restarted (32 ticks).
- Hidden plants of one alarm group wake together within 15 to 45 ticks of the first agro entry.
- The back-off clip (13) is also the dodge clip; a dodging plant therefore moves exactly like a backing-off plant.

**What a port most likely gets wrong.**
1. Treating the Flower Monster as one enemy: rows 147 and 149 have different movement, attacks and defence.
2. Letting slot 0 fire (it is the only slot with a high base damage and a clip, but it can never pass).
3. Forgetting the 99 % missile dodge against Roberta.
4. Playing clip 0 for the missing phases (every slot has only a B clip; slot 1 also C and D).
5. Using 50 % instead of 49 % per tick and ignoring the 0.9914 cone of the claw.
6. Giving the plant ground speed by a row value; the walk is root motion of 12.57 units per second.
7. Making the search after a notice last: a false start returns the plant to its routine immediately.
8. Stopping the counter after one roll: the slam counter is rolled on every tick of the 57-tick reaction.

**Timeline test vector (hidden record, emerge distance 20.0, hero idle, 12.0 units in front, hero facing the plant).** Tick 0: wake test passes (12 below 20 x 0.7 = 14). Tick 1: animation 10 (clip 18) requested. Tick 137: class state 19. Tick 138: outer 1, inner 1, the plant faces the hero. Tick 139: first normal choice: slot 2 fails (d = 12 above 9), slot 1 passes with 49 % per tick: choice tick c = 139 + G with G geometric (mean 1.0). The claw starts walking (d 12 to 10 takes about 10 ticks at 0.2095 per tick): s about c + 11. B on s with the hero 10 ahead; window from s + 32, first touch and hit on s + 36 (5 damage; the hero is hurt 36 ticks); C on s + 37; end path s + 59; recovery int[15, 30]. After the lunge the plant is about 5 units from the hero, so slot 2 is eligible (49 % per tick) as soon as the recovery has run out: next attack starts about s + 75 to s + 91, hits on that start + 32 (10 damage), end path 70 ticks after its start. The slot cooldowns (claw 120 to 300, slam 60 to 240, both redrawn at each end path) let the next claw come no earlier than about s + 175 and the next slam no earlier than about s + 190. By tick 600 the hero has taken between 4 and 6 attacks (5 and 10 damage alternately), minus armour. A hero who is hit and then moves away more than 17 units makes the plant walk after it; it never leaves agro unless the leash or lost-sight rules fire.

**Open questions.**
1. Weapon reach geometry. Resolved: the general rule of the shared field behaviour (weapon 3 for the claw and weapon 4 for the slam, radius 5.0 each, against the hero's capsules; the anchor is wherever the clip puts the attachment frame), with the distances in the Weapon reach table above. Stand-in only for a port without attachment frames (rewrite choice): the claw touches a level hero from 1.2 behind to 12.4 ahead of the plant, the slam from 4.4 behind to 6.6 ahead.
2. Wake test: Resolved, horizontal (High; proactive (open close-out); batch question 1). Chooser: Default if unresolved (rewrite choice): horizontal (needs a runtime check).
3. First step latency of the walk after the choice (plus or minus one tick). Default: one tick.

#### Flower Monster, spitter build (row 149)

**Identity and tunables.** Type 69, variant 0; 7 clips; HP 16; approach 3 (keep distance): desired band 3.0 to 35.0 (desired = 3.0 + 32.0 f), retreat band 0 to 0; radius 1.0, height 5.0; weight 1; target type 2; hold turn weight 1.0 but the keep-distance hold aims at blend 0.25 per tick; recovery int[15, 30]; death timer 10. Awareness 100 percent, cycle 60 ticks, notice band 1.0 to 10.0 (times F1), sight-line test skipped. No defence at all: it never dodges or blocks. One attack slot.

| k | Type | Band (strict) | Chance | Facing cosine above | Cooldown draw | Speeds | Damage | Projectile |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| 0 | missile, flight mode 1 (lob) | 3 to 60 | 100, 99 % | 0.9914 | int[60, 240] | 1, 1, 1 | 10 | launch speed 0.5, speed at the near edge 0.1, radius 1.0, status 3 (slow) 25, value 0, area 0 and 0, effects 1400, none, 1399, none, 1399, 1399 |

Slots 1 to 3 unused. Counter and charge: none.

**Animations.** Hold 25 is clip 0 (20 keys, loop). Keep-distance walk 24 and back 26 and the alert 18 are clip 1 (5 keys, loop, no travel). Idle 0 and all of 2 to 12 (including emerge 10 and hide 9) are clip 2, a one-key loop pose. Spit 34 is clip 5 (13 keys, one shot, release frame 6). Hit reaction 57 is clip 6 (20 keys, gate 77). Death 58 is clip 4 (3 keys, gate 9). Id 81 is clip 2 and 82 is clip 3. The table has no clip for 21, 22, 23, 55 or 53 and 54: a missing id plays clip 0, but none of those ids is requested by this kind's states. Because the walking and hold clips carry no root travel, the plant never moves.

**Start and wake.** Hidden records (100): emerge distance 10 to 70 times F1; the emerge animation is the one-key clip 2, so the plant enters agro 4 ticks after the test (class state 19 on a + 4, first agro tick a + 5, first choice a + 6). Sentries (3, Ellis Family Farm) stand at post; the wander record (Farm Road) tries to reach goals it can never reach and so stands; summon-reserved records start in agro (below).

**Per-tick order.** As SPEC-0038 Behavior 44: recovery countdown, guard reset, missile defence (never passes: no defence), melee defence (never passes), lost-sight test, normal choice (slot 0 only), counter choice (nothing to counter), distance-hide gate (off), outer sub-state switch, agro tail (route flag and leash). The lost-sight test and the leash can send the plant back to its default state; a plant that can see the hero from its fixed position does not.

**State table.**

| State | Animation id -> clip | Keys | Gate | Exits |
| --- | --- | --- | --- | --- |
| Hidden / sentry / wander / alert | 9, 7 or 8, 2, 18 -> clips 2, 2, 2, 1 | 1 or 5 | 3 or 19 | agro |
| Keep hold (outer 3, inner 4) | 25 -> clip 0 | 20, loop | none | d above 35 to inner 3 (walk, no motion); d below 3 to inner 5 (back, no motion) |
| Lob (slot 0) | A 33 skipped; B 34 -> clip 5; C, D, E skipped | 13 | B 49 | end path |
| Hit reaction (kind 0) | 57 -> clip 6 | 20 | 77 | outer 0 |
| Dead | 58 -> clip 4 | 3 | 9 | corpse released after about 20 ticks |

**Attack chooser.** Only slot 0: 3 < d < 60, facing cosine above 0.9914, cooldown int[60, 240] spent, roll below 100, route or missile rule satisfied (a missile slot always passes). In the outer-3 hold the 0.25 blend is followed in the same tick by the hold aim at the row's turn weight 1.0, an effective blend of 1.0 (SPEC-0038 Behavior 44), so the plant faces the hero exactly on its first hold tick whatever the start angle and the 0.9914 cone passes on the next tick. Lob: choice on c, state 0 (missile: continues once the hero and the steer point are on the walk mesh), s = c + 1: B requested; the spawn event bit 0x20 is on key 6, seen first on s + 24; one projectile is created there (emit point 0); the gate passes on s + 49; the end path runs on s + 50 with recovery int[15, 30] and the cooldown redrawn.

Lob flight: aim point = hero position (feet) + scatter e (uniform 0 to 0, so none) + 5.0 when the plant is outside the camera cone (cosine below 0.85). With d the horizontal distance to the aim point, u = 1 - (d - 3) / 57 (not clamped) and speed s_v = 0.5 - 0.4 u^2, flight time T = d / s_v ticks: d = 5 gives T 39, d = 10 gives 52, d = 20 gives 66, d = 40 gives 89, d = 60 gives 120. T is the time to the aim point at the feet; a standing hero's body cylinder is met earlier, on the way down: hit ticks after release 16 (d 4), 40 (10), 58 (20), 65 (25), 83 (40), 113 (59) (Distances and ranged reach). The shot is ballistic (gravity 32 / 3600 per tick squared); lifetime trunc(60 T) + 1, so it always lands before it expires. A contact with the hero's body (shot radius 1.0) in flight deals 10 (minus shield, armour), and a roll int[0, 100] at most 25 (26 of 101) applies slow with value 0 seconds (a zero-length slow). A blocked hit (hero guarding, shot heading against the hero's facing) deals max(1, a quarter). On landing the shot enters area mode with duration 0 and radius 0, so it ends on the next tick and the ground does nothing.

**Movement.** None. The turn is the hold's aim only. Every clip of this table, the walk ids 2 and 4 (a one-key clip), the keep-distance gaits, the hit reaction and the death clip included, has zero root travel, so the spitter never changes position in any state: no wander, no patrol, no flinch slide and no corpse slide (movement pass; High). A port must still run its steering and gait requests (they decide which pose plays) but must move nothing.

**Hit and hurt windows.** Spit release s + 24 (single). The plant is hittable in every state; it has no dodge state. A hit restarts clip 6 (77 ticks) and cancels a lob in progress without an end path; the lob that was already created keeps flying.

**Defence and counters.** None (all defence values 0; counter -1).

**Hero actions against this enemy.** Any swing or shot hits (no defence); first hit while hidden or not in agro doubles the damage (starting sword: three swings, 8 then 4 and 4, against HP 16). The plant cannot hurt a hero closer than 3.0 units: the band is strict, so a hero standing at the foot of the plant is safe from the lob. A hero farther than 60 units is never targeted. The lob can be walked out of: flight time is 39 to 120 ticks and the aim point is fixed at the launch tick.

**Edge cases for implementers.**
- The sixteen-HP plant has its own table of 7 clips; the keep-distance gaits play real clips with zero root travel, so any "walk toward the hero" step must move nothing.
- With the spit band starting at 3.0, a summoned plant that lands at the hero's feet (distance about 0) cannot fire until the hero moves more than 3 units away.
- Only slot 0 exists, so the counter choice never runs anything.
- The hit reaction (77 ticks) is the plant's only recovery; with the starting sword (4 per swing, 8 for the finisher) it dies in four plain swings or in three when the first one is doubled (8, 4, 4).
- A hidden spitter with a group byte wakes when a member of its group enters agro, 15 to 45 ticks later.
- The wake animation is one key: no visible emerge.

**What a port most likely gets wrong.**
1. Moving the plant (it is rooted; the "keep distance" numbers 3 to 35 only decide which hold or walk animation plays).
2. Giving the lob a straight flight. It is a slow lob, 0.13 to 0.5 units per tick.
3. Letting the lob's slow last: the status value is 0 seconds.
4. Firing at a hero inside 3.0 units.
5. Giving the spit a ground area: area duration is 0.
6. Waking hidden spitters only by distance: groups wake together.
7. Treating the 90 reserved records as streamed placements.

**Timeline test vector (hidden record, emerge distance 40.0, hero idle, 25.0 units away; the plant faces 90 degrees off the hero).** Tick 0: wake test passes (25 below 28). Tick 1: animation 10 requested (clip 2). Tick 4: class state 19. Tick 5: outer 3, inner 4; the hold's aim (effective blend 1.0) turns the plant fully onto the hero in this tick. Tick 6: the cone test passes; c = 6 + G (G geometric, 99 % per tick, so G is 0 almost always). s = c + 1 = 7: B requested. Tick 31: the lob is created (aim point at the hero's feet; d = 25 gives u = 0.614, speed 0.349, about 72 ticks to the aim point): it meets the body of a hero who has not moved on tick 96 (s + 89; 10 damage, then 26 of 101 for a zero-length slow). Tick 56: gate. Tick 57: end path, recovery int[15, 30], slot cooldown redrawn int[60, 240]. The next lob starts (s) no earlier than tick 118 and no later than about tick 300 (cooldown drawn at the choice on tick 6 has long run out; the redrawn one decides, plus recovery plus a geometric wait). By tick 600 the plant has thrown between 3 and 5 seeds; a standing hero takes 10 each time it hits.

**Open questions.**
1. Resolved (Medium-High; proactive (open close-out)): the lob's world test runs once it is more than 2.0 units from its start and tests the static meshes and the world's collision primitives of the cell, so trees and walls with collision catch it (SPEC-0029 Behavior 9, World collision). Whether a given tree model has collision is level data.
2. Resolved (enemy sweep): facing in the keep-distance hold. The 0.25 blend is followed in the same tick by the aim at the row turn weight 1.0 (SPEC-0038 Behavior 44), so the plant faces the hero exactly every hold tick.
3. Whether a wander record whose goal cannot be reached ever times out. No timeout exists in the wander body (only the agro leash has one). Default if unresolved (rewrite choice): it stands still and keeps trying (needs a runtime check).

### Poison Flower Monster

Row 148, type 68, variant 2, spawn key 621. It shares the melee Flower Monster's body and 23-clip set, but its own animation table is shorter and its attack list differs: a poisoning claw, a slam, a poison lob and one attack that has no animation at all. Only 3 placements exist, all hidden (default state 8) in Shadowglen Woods, none in an alarm group, emerge distance 10.0 on each. Confidence: High for the row and table; Medium-High for the phantom slot and the lookup-miss consequences; Medium-Low for the hidden-pose drift.

**Identity and tunables.** HP 25; approach 1 (rush band 4.0 to 17.0, desired 4.0 + 13.0 f, back off to 19.0 behind the hero); radius 2.5, height 2.0; weight 12; target type 2 (immune to poison); recovery int[15, 30]; hold turn weight 1.0; notice, search and defence exactly as the melee Flower Monster. Weapon volumes as the melee Flower Monster (weapon 1 radius 3.0; weapons 2 to 5 radius 5.0).

| k | Type | Band (strict) | Chance | Facing cosine above | Range | Cooldown draw | Damage / knockback | Status | Counter |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| 0 | melee, no animation | 8 to 17 | 50, 49 % | 0.9239 | 13.0 | int[60, 120] | 7 / 0 | none | none |
| 1 | melee claw lunge | 0 to 20 | 50, 49 % | 0.9914 | 10.0 | int[120, 300] | 5 / 0 | status 2 (poison), chance 50, duration int[1800, 3600] / 60 = 30 to 60 s | none |
| 2 | melee slam | 0 to 9 | 50, 49 % | 0.0 | 7.0 | int[60, 180] | 10 / 0 | none | none |
| 3 | missile, lob, flight mode 1 | 15 to 30 | 25, 24 % | 0.0 | 20.0 (unused) | 0 | 10 / 0 | status 2, chance 50, 30 to 60 s | none |

Slot 3 projectile: launch speed 1.0, speed at the near edge 0.7, radius 0.2, range 30 (unused by the lob), area duration 90 ticks, area radius 3.0, effects 1400, none, 1401, none, 1401, 1401. The rush plant has no counter attack at all (every counter code is -1).

**Animations (this kind's table).** Walk 2, 4 and 5 are clip 0 (15 keys, loop; 2.97 units per second); rush walk 21 is clip 15 (10 keys, loop; 10.51 units per second, 7.0 units per 40-tick cycle); back 23 and melee dodge 55 are clip 13 (as the melee Flower Monster); 53 and 54 clip 14; hit 57 clip 10; death 58 clip 8; victory 73 clip 9. Slot 1: B 39 clip 3, C 40 clip 4, D 41 clip 2. Slot 2: B 44 clip 5. Slot 3: B 49 clip 1 (23 keys, one shot, the release bit on key 9, gate 89). Slot 0: ids 33 to 37 all absent. Absent from the table: idle 0, hide 9, emerge 10, hold 22, search 16, alert 18, 34 and every other id not listed. A request for an absent id plays clip 0 of the set, with its forward root travel.

**Start and wake.** Hidden at home. The hide pick (animation 9) is absent, so the hidden plant plays clip 0 and its root travel moves it about 0.2 units forward every 4 ticks until it is 0.35 from home, when the state-8 handler turns it and walks it back (gait 0, also clip 0): the plant shuffles around its home point instead of standing still (Medium-Low, open question 1). The wake test is the usual emerge distance 10.0 x F1 (7.0 for an idle hero); the emerge animation is absent, so agro starts on the tick after the test: wake test on a, sub-state 1 on a + 1 (animation missing: class state 19 at once), first agro tick a + 2, first choice a + 3. A hit wakes it as for any hidden plant (damage doubled, hit reaction kind 0).

**Per-tick order.** As SPEC-0038 Behavior 44: recovery countdown, guard reset, missile defence, melee defence, lost-sight test, normal choice (slots 3, 2, 1, 0), counter choice (none eligible), distance-hide gate (off), outer switch, agro tail.

**State table.**

| State | Animation id -> clip | Keys | Gate | Exits |
| --- | --- | --- | --- | --- |
| Hidden (class 8) | 9 -> absent, plays clip 0 | 15, loop | none | wake test, hit |
| Rush hold (outer 1, inner 1) | 22 -> absent, plays clip 0 | 15, loop | none | d below 4.0 to inner 2; d above 17.0 to inner 0 |
| Rush walk | 21 -> clip 15 | 10, loop | none | d below desired to inner 1 |
| Back off | 23 -> clip 13 | 13, loop | none | d above desired to inner 1 |
| Slot 0 (no animation) | A to E all skipped | | none | end path on the first tick of state 5 |
| Slot 1 claw | A skipped; B 39 -> clip 3; C 40 -> clip 4 after a hit; D 41 -> clip 2 | 11, 6, 15 | B 41, C 21, D 57 | end path |
| Slot 2 slam | B 44 -> clip 5; others skipped | 18 | B 69 | end path |
| Slot 3 lob | A skipped; B 49 -> clip 1; C, D, E skipped | 23 | B 89 | end path |
| Melee dodge, missile dodge, hit reaction, dead | 55, 53/54, 57, 58 -> clips 13, 14, 10, 8 | | 51, 49, 57, 116 | as the melee Flower Monster |

**Attack chooser.** Slots are tested 3, 2, 1, 0 and the first to pass wins.
- Slot 3: 15 < d < 30, any facing in front (cosine above 0), roll below 25 (24 % per tick), no cooldown (cooldown draw 0). It passes the missile rule.
- Slot 2: 0 < d < 9, cosine above 0, cooldown int[60, 180], roll below 50.
- Slot 1: 0 < d < 20, cosine above 0.9914, cooldown int[120, 300], roll below 50.
- Slot 0: 8 < d < 17, cosine above 0.9239, cooldown int[60, 120], roll below 50. When it is chosen the sequencer finds no clip for any phase and ends in the same tick: recovery int[15, 30] and a fresh slot-0 cooldown, no damage, no animation. If d is above 13 the sequencer first walks the plant toward the hero until d is at most 13 (state 0), so slot 0 looks like an approach. In the hold band (4 to 17) the other slots are tried first, so slot 0 appears only when slots 3, 2 and 1 fail their rolls.
Walking rule of state 0 for the real melee slots: the plant walks with gait 0 (clip 15) while d is above the slot's range (10 for the claw, 7 for the slam).

Claw: s = first tick of state 1; B on s; weapon 3 open on s + 32 to s + 40; one hit of 5 on the first touching tick h (s + 32 for a hero 7 or less ahead at s, s + 36 for one 10 ahead, as the melee Flower Monster) plus a poison draw (int[0, 100] below 50, 50 of 101, duration 30 to 60 s, default amount 2 HP every 4 s, never below 1 HP); C on h + 1; end path h + 23 (s + 55 to s + 59); miss: end path s + 42. Slam: hit 10 on s + 32, end path s + 70. Lob: choice on c, s = c + 1, B 49 requested on s, the projectile is created on s + 36 (release bit on key 9, first seen on s + 36), gate on s + 89, end path s + 90. Flight: u = 1 - (d - 15) / 15, speed s_v = 1.0 - 0.3 u^2, T = d / s_v ticks: d = 15 gives 21, d = 20 gives 23, d = 25 gives 26, d = 30 gives 30. A flight contact with the hero (radius 0.2) deals 10 and rolls the poison (a draw 0 to 100 at most 50). The shot then lands and enters area mode for 90 ticks with radius 3.0, but the area never pulses: the ground test reloads the pulse timer to 60 on every tick the landed shot lies on or under the floor (SPEC-0029 Behavior 9, Area mode), so the puddle deals nothing. Only the flight contact hurts; a standing hero 29 units away is missed (the lob lands 1.6 short; Distances and ranged reach).

**Movement.** Rush outer 1 only. Walk 10.51 units per second (0.175 per tick) from the first step one tick after the request; the hold plays clip 0, so the plant creeps forward at 2.97 units per second (0.05 per tick) toward the hero while it holds, until d falls below 4.0 and the back-off starts. The back-off ends when d is above the desired distance (6.6 to 14.4).

Movement table (row 148; ground; radius 2.5; ground turn 1.0; rush band 4 to 17):

| Mode | Source | Clip (keys) | u/tick | u/s | Turn | Exit |
| --- | --- | --- | --- | --- | --- | --- |
| Wander, patrol, walk home, hide pick, hold and every other missing id | root | clip 0 (15) | 0.0495 | 2.97 | steer; in the hold aim 1.0 | as the state |
| Approach (inner 0) | root | 21 (10) | 0.175 | 10.51 | steer | d below desired |
| Back off (inner 2), below 4 | root, facing the hero | 23 (13), keys swing 0.13 forward to 0.28 backward per tick | 0.0785 backward | 4.71 | steer, back gait | d above desired |
| Hit (reaction kind 0) | root | 57 (15) | | back 9.19 over 60 ticks | snap to attacker | gate |
| Dodge 53 / 54, 55 | root | (13) each | | side 2.91 over 52; back 4.08 over 52 | | gate |
| Confused (outer 11) | root | 69 -> clip 19 (55, one-shot; frame-stored root, mode 0), as the Flower Monster | 0 to 0.042 inside the clip, then 0.0836 held | 6.14 forward over 220 ticks (1.67), then 5.01 while held | aim at a random direction 0.05 | status end |
| Corpse | root | 58 (30) | | back 0.87, nothing held | kept | timer 300 |

Because the hold creeps at 2.97 and the back-off retreats at only 4.71, a Poison Flower Monster held at its desired distance closes on a still hero at 2.97 per second until d is below 4, then backs off: it oscillates between about 4 and its desired distance instead of standing still.

**Hit and hurt windows.** Claw weapon 3 at s + 32 to s + 40, slam weapon 4 at s + 32 to s + 39, lob release s + 36 (the landed puddle never pulses). Hittable in every state except the two dodges and the dead state.

**Weapon reach.** Conventions are those of "Weapon reach tables" in the shared field behaviour. Row 148, Medium. The window columns are computed from the clips; where the timeline above names a window or a hit tick, that text is authoritative. The geometry is the same as the melee Flower Monster's (the weapons are weapon 3 for the claw and weapon 4 for the slam, not weapon 1).

| Slot (phase, clip) | Weapon | Window ticks (level contact) | Centre forward / up / side | Tor ahead, at the tick | Tor ahead, from the start | Tor feet vs origin |
|---|---|---|---|---|---|---|
| 1 (B, clip 3) | w3 (5.0 x 3.0) | s+32 to s+41 (s+32 to s+41) | 3.2 to 10.2 / 1.6 to 2.6 / -0.5 to 0.1 | -1.2 to 12.4 | -1.3 to 15.4 | -5.5 to 7.5 |
| 1 (C, clip 4) | w3 (5.0 x 3.0) | sC+1 to sC+7 (sC+1 to sC+7) | 3.8 to 11.2 / 1.5 to 2.5 / -0.1 to 0.2 | 1.8 to 15.4 | 2.5 to 15.4 | -6.5 to 7.0 |
| 2 (B, clip 5) | w4 (5.0 x 2.0) | s+32 to s+39 (s+32 to s+39) | 0.5 to 0.9 / 1.5 to 2.5 / -0.7 to -0.5 | -4.6 to 6.6 | -2.0 to 11.5 | -5.5 to 5.5 |

**Defence and counters.** As the melee Flower Monster (melee dodge 74 %, 49 %, 29 % by swing weight; missile dodge 99 %; defence timer int[60, 240]) but with no counters.

**Hero actions against this enemy.** As the melee Flower Monster. The extra rule: Leaf Motes do not poison it (target type 2). A hero standing at 15 to 30 units draws the lob 24 % of ticks; a hero at 8 to 17 sometimes draws the empty slot 0, which wastes the plant's time (15 to 30 ticks plus cooldown).

**Edge cases for implementers.**
- Slot 0 has damage 7 and a range but no animation: it must run as an empty attack (recovery and cooldown only), not as an error and not as a clip 0 loop.
- Missing animation ids play clip 0 of the set; this plant's clip 0 is a walking clip with forward travel, so its idle, hide and hold all creep forward.
- The puddle never pulses (its pulse timer is reloaded every tick by the ground test; SPEC-0029 Behavior 9).
- The claw's poison roll is a draw 0 to 100 below 50 (50 of 101); the lob's roll is a draw 0 to 100 at most 50 (51 of 101).
- No alarm group, no notice from other states: only the distance wake and hits start it.

**What a port most likely gets wrong.**
1. Giving it the melee Flower Monster's idle and hold animations (this table lacks them).
2. Skipping the slot-0 phantom attack or giving it a swing and damage.
3. Putting a puddle that hurts at all; the area never pulses.
4. Applying the claw's poison on every hit; it is a 50 % roll.
5. Letting it counter after a hit (it has no counters).
6. Lobbing from the hold band: the lob band is 15 to 30, outside the usual hold distance of 6.6 to 14.4, so it lobs only when the hero stands far (the plant also walks when d is above 17).

**Timeline test vector (hidden, hero idle 6.0 units in front).** Tick 0: wake test passes (6 below 7.0). Tick 1: sub-state 1, animation 10 absent, class state 19. Tick 2: outer 1, inner 1 (the hold plays clip 0 and the plant creeps forward, facing the hero). Tick 3: first choice: d about 6; slot 3 fails (d under 15), slot 2 passes 49 % (0 < 6 < 9), slot 1 49 % of the rest (about 25 %), slot 0 fails (d under 8). With the plant's first pick c = 3 + G (G geometric with success about 0.74): slot 2 start on s = c + 1 (d is under the range 7): hit on s + 32 (10 damage), end path s + 70, recovery 15 to 30. The claw (if picked: s + 32 hit 5 and poison 50 %, end path s + 55) comes next as slot 2's cooldown (60 to 180) is redrawn at s + 70. Between tick 3 and tick 600 the plant makes 4 to 7 attacks (slams of 10 and claws of 5, at intervals of about 90 to 180 ticks); each claw hit also poisons with probability 50 of 101.

**Open questions.**
1. Resolved (movement pass): a character that plays a clip with root travel moves by it in every state, holds included: the motion step of a region character has no state gate and takes the root step of whatever clip is playing (SPEC-0038 Behavior 51; SPEC-0040 Behavior 14D). The plant shuffles around its home while it hides and creeps toward the hero at 2.97 units per second while it holds (High).
2. Resolved (Medium-High; proactive (open close-out)): the lob is caught by static world collision after 2.0 units of flight, exactly as for the melee-build Flower Monster above (SPEC-0029 Behavior 9).
3. Resolved: reach geometry of the claw and slam is the general rule of the shared field behaviour, with the distances in this kind's Weapon reach table (the same as the melee Flower Monster's).

### Spore Monster

Row 221, type 85, variant 2, spawn key 1623. A stationary-to-slow caster that lobs seeds which grow into spitter Flower Monsters (key 619) where they land. 18 placements: 8 sentries and 1 farm-road sentry (state 7), 6 patrols (state 6), 2 wanderers (state 5), plus 1 cut. No hidden spore monsters and no alarm groups. Confidence: High for the row, the attack arithmetic and the summon rule; Medium-High for the sentry and patrol behaviour.

**Identity and tunables.** HP 35; approach 1: rush band 10.0 to 30.0, desired 10.0 + 20.0 f (14.0 to 26.0), back off to the point 32.0 units behind the hero; radius 3.5, height 7.0; weight 5; target type 2; recovery int[15, 60]; hold turn weight 0.7; weapons 1 to 3 radius 4.0, length 1.0. Notice and search as in the family table (awareness 100, cycle 30, band 5 to 120, chance 70 to 25, search timer int[60, 239]).

| k | Type | Band (strict) | Chance | Facing cosine above | Range | Cooldown draw | Damage / knockback | Status | Counter |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| 0 | unused | | | | | | | | |
| 1 | melee, four-hit swipe, orient 1 | 0 to 15 | 40, 39 % | 0.9239 | 7.0 | int[120, 300] | 10 / 0 per hit | status 3 (slow), chance 25, 10 to 15 s | code 6, chance 5 |
| 2 | unused | | | | | | | | |
| 3 | missile, lobbed summon, flight mode 1, flag 0x02 | 10 to 60 | 100, 99 % | 0.0 | 5.0 (unused) | int[15, 60] | 5 / 0 (unused) | none | none |

Slot 3: launch speed 0.5 (the summon flies at exactly this speed), spawn count 5, summon key 619, summon limit 5. The row's speed at the near edge (0.1) and range (10) are not used by the summon.

**Animations (this kind's table).** Idle 0 and the sentry stand 7 are clip 0 (30 keys, loop). Fidget 8 and the wander rest fidget 3 are clip 15 (47 keys, gate 184). Wander and patrol walk 2 and 4 are clip 1 (15 keys, loop, 3.92 units per second). Rush walk 21 is clip 3 (10 keys, loop; 15.0 units per second, 10.0 units per cycle). Rush hold 22 is clip 2 (20 keys, loop). Back off 23 and melee dodge 55 are clip 19 (30 keys at speed byte 1, 60 ticks per cycle; 9.9 units per second backward, gate 60). Missile dodge 53 and 54 are clip 14 (20 keys, gate 77, side-step 13 units). Hit reaction 57 is clip 18 (15 keys, gate 57; pushed 10.41 units backward). Death 58 is clip 13 (33 keys, gates 128 and 124). Hide 9 is clip 4 and emerge 10 is clip 17 (35 keys, gate 136); neither is used (no hidden records). Slot 1: B 39 clip 8 (30 keys, four windows), others absent. Slot 3: B 49 clip 12 (10 keys, release bit 0x20 on key 1, gate 37), others absent. Search 16 and alert 18 are absent from this table.

**Start and wake.** Sentries (9) stand at their post with fidgets; patrols (6) walk their path with pauses int[120, 480]; wanderers (2, Woods; +0xBC = 10, rest chance 49 %, rest int[60, 300]) roam a 10-unit square. Notice as in the family section; a notice with a false start sends it back for the search timer int[60, 239]; a notice with a pass goes to agro on the same tick (no alert animation). A hit takes agro, doubles the damage and starts hit reaction kind 0. The think loop's first normal choice follows the first agro tick (one tick after the setter).

**Per-tick order.** As SPEC-0038 Behavior 44: recovery countdown, guard reset, missile defence, melee defence, lost-sight test, normal choice (slots 3, then 1), counter choice (slot 1 during hit reactions), distance-hide gate (off), outer switch, agro tail. Cooldowns count down only while the hero is attackable (not hurt, not knocked down).

**State table.**

| State | Animation id -> clip | Keys | Gate | Exits |
| --- | --- | --- | --- | --- |
| Rush hold (outer 1, inner 1) | 22 -> clip 2 | 20, loop | none | d below 10.0 to inner 2; d above 30.0 to inner 0 |
| Rush walk (inner 0) | 21 -> clip 3 | 10, loop | none | d below desired to inner 1 |
| Back off (inner 2) | 23 -> clip 19 | 30, loop (speed byte 1) | none | d above desired to inner 1 |
| Slot 1 swipe | B 39 -> clip 8; A, C, D, E skipped | 30 | B 116 | end path |
| Slot 3 lob | B 49 -> clip 12; A, C, D, E skipped | 10 | B 37 | end path |
| Melee dodge (outer 5) | 55 -> clip 19 | 30 | 60 | defence exit |
| Missile dodge (outer 6) | 53 or 54 -> clip 14 | 20 | 77 | defence exit |
| Hit reaction (outer 10) | 57 -> clip 18 | 15 | 57 | outer 0 |
| Dead | 58 -> clip 13 | 33 | 128 (0.95 at 124) | corpse released after about 425 ticks |

**Attack chooser.** Slot 3, then slot 1.
- Slot 3 (summon): 10 < d < 60, cosine above 0, cooldown int[15, 60] spent, roll below 100 (99 %), and fewer than 5 of its summoned plants alive (the live count is the number of its spawned characters that still exist and are not dead). A missile slot passes the charge rule.
- Slot 1: 0 < d < 15, cosine above 0.9239, cooldown int[120, 300], roll below 40 (39 %). If d is above 7.0 the swipe first walks (clip 3) until d is at most 7.0.
So from the hold band the plant lobs whenever its summon cooldown, recovery and live limit allow, and swipes only when the hero is within 15 units (it backs off below 10).

Lob timeline: choice on c, s = c + 1: B 49 requested on s; the release bit is on key 1 (ticks 4 to 7 of the clip), so one lobbed shot is created on s + 4 (emit point 0), gate on s + 37, end path s + 38 (recovery int[15, 60], slot cooldown redrawn int[15, 60]). The shot count of 5 in the row is not reached: the clip has one release window, a second spawn needs a second window, and the clip ends first. One attack lobs exactly one seed. The seed is a kind-8 projectile with the aim point at the hero's feet (scatter 0, plus 5.0 outside the camera cone, plus 3.0 while waiting in line), speed 0.5, T = d / 0.5 ticks to the aim point (d = 20 gives 40, d = 28 gives 56, d = 60 gives 120), ballistic, harmless to characters. It leaves 5.4 above the ground and lands (its centre 1.0 above the ground) before T and short of the aim point: at d = 20 30 ticks after release, 18.5 from the Spore Monster; at d = 28 46 ticks after release, about 26.5 away; at d = 40 70 ticks, 38.5 away (Distances and ranged reach). When it lands or touches the world after 2.0 units of flight, a world-level spawn-by-key request for key 619 runs at that point, facing along the shot's forward axis (which the row's spin rate of 1.0 turns every tick in flight, so the facing is effectively arbitrary; the plant turns to the hero in its hold): the embedded block is searched first and then each slot in order; the first unbound reserved spitter record is bound there at full health in agro (class 17 state 19), with the Spore Monster as its summoner. The request fails (nothing appears) when no reserved record is free (five per Spore Monster region) or the block's active count is at its cap. An expiry in the air spawns nothing (it cannot happen: the flight always ends first).

Swipe: B 39 requested on s (orient 1: the plant aims at the hero at weight 1.0 in states 1 to 3). Weapon 1 and 2 windows (keys 7 to 9 and 17 to 20 for weapon 1, keys 12 to 15 and 22 to 25 for weapon 2): four separate windows, each re-arming the hit list, so up to four hits per swing at s + 28, s + 49, s + 69 and s + 90 for a hero straight ahead 5.5 to 8.5 units away at the request (the second and third are one tick after their window opens, the tick the sequencer re-enters its strike state; the fourth window's first touch on a level hero is s + 90, one tick later than that re-entry tick s + 89, and s + 91 to s + 93 when the hero is only 2.0 to 3.0 away). Each hit deals 10 and rolls slow (draw 0 to 100 below 25, 25 of 101, 10 to 15 s). Gate s + 116, end path s + 117.

**Movement.** Rush outer 1: holds between 10 and 30; backs off at 9.9 units per second (0.165 per tick) when the hero is within 10; walks at 15.0 units per second (0.25 per tick) when the hero is farther than 30; chase stop inside the radii sum in sequencer states 0 to 3. Wander 3.92 per second.

Movement table (row 221; the Plant Gunner (220) and Plant Monster (219) use the same clips; ground; radius 3.5; ground turn 0.7; rush band 10 to 30, desired 14 to 26):

| Mode | Source | Clip (keys) | u/tick | u/s | Turn | Exit |
| --- | --- | --- | --- | --- | --- | --- |
| Wander, patrol, walk home | root | 2 or 4 (15) | 0.0653 | 3.92 | steer | arrival 0.5, notice |
| Approach (inner 0) | root | 21 (10) | 0.250 | 15.01 | steer | d below desired |
| Hold (inner 1) | none | 22 | 0 | 0 | aim 0.7 | d below 10 or above 30 |
| Back off (inner 2), below 10 | root, facing the hero | 23 (30, S1); the keys swing from 0.19 forward to 0.65 backward per tick | 0.165 backward | 9.90 | steer, back gait | d above desired |
| Hit (reaction kind 0) | root | 57 (15) | | back 10.41 over 60 ticks | snap to attacker | gate |
| Dodge 53 / 54, 55 | root | 53/54 (20), 55 = the back-off loop | | side 13.00 over 80; back 9.90 per second while it plays | | gate |
| Corpse | root | 58 (33) | | back 5.80 over 132 ticks, nothing held | kept | timer 300 |
| Missing ids (20, 125, 28 to 32) | none | clip 0, no root | 0 | 0 | | |

The back-off clip moves in surges (one key in four carries most of the travel); a port that averages it to 0.165 per tick moves the same distance but loses the stagger.

**Hit and hurt windows.** Swipe windows above; lob release s + 4. Hittable in every state except the dodges and dead.

**Weapon reach.** Conventions are those of "Weapon reach tables" in the shared field behaviour. Row 221, Medium. The window columns are computed from the clip; the hit ticks above are authoritative (the second and third hits land one tick after their window's first tick, because the sequencer re-enters its strike state then; the fourth lands on its first touching tick, s + 90).

| Slot (phase, clip) | Weapon | Window ticks (level contact) | Centre forward / up / side | Tor ahead, at the tick | Tor ahead, from the start | Tor feet vs origin |
|---|---|---|---|---|---|---|
| 1 (B, clip 8) | w1 (4.0 x 1.0) | s+28 to s+39 (s+28 to s+39) | 5.2 to 7.9 / 0.5 to 4.5 / -3.7 to 0.9 | 1.2 to 12.4 | 3.0 to 13.9 | -3.5 to 5.0 |
| 1 (B, clip 8) | w2 (4.0 x 1.0) | s+48 to s+63 (s+48 to s+63) | 3.1 to 5.6 / 0.5 to 2.9 / 0.2 to 3.7 | 1.2 to 10.0 | 4.3 to 13.0 | -4.5 to 5.0 |
| 1 (B, clip 8) | w1 (4.0 x 1.0) | s+68 to s+83 (s+69 to s+83) | 4.3 to 8.1 / 1.3 to 4.6 / -4.5 to -0.2 | 3.0 to 12.6 | 7.1 to 16.6 | -2.5 to 7.0 |
| 1 (B, clip 8) | w2 (4.0 x 1.0) | s+88 to s+103 (s+90 to s+103) | 3.3 to 5.5 / 0.2 to 2.7 / 0.3 to 4.3 | 1.4 to 10.2 | 5.8 to 14.6 | -4.5 to 3.5 |

No swipe touches a level hero closer than 1.2 ahead of the plant at the tick. The reach is long (12.4 to 12.6 ahead at the tick, 16.6 from where the plant started) because the plant lunges forward during the swipe.

**Defence and counters.** Melee dodge chances 40, 25, 10 (39 %, 24 %, 9 % by swing weight), block 0; missile dodge 25 % (24 of 100); the hold 60; timer int[60, 120]. Counter code 6 on slot 1 with chance 5: the counter choice is evaluated on every tick of a hit reaction (57 ticks); slot 1 is eligible when 0 < d < 15, cosine above 0.9239 and its cooldown is spent, and each tick rolls int[0, 100] below 5 (5 of 101), so a hit on a Spore Monster within 15 units draws a swipe in about 1 - 0.95^56 = 94 % of reactions.

**Hero actions against this enemy.**

| Hero action | Result |
| --- | --- |
| Swing at a sentry, patrol or wander plant before it has noticed | Damage doubled (8 with the starting sword), agro at once, hit reaction kind 0 (57 ticks, pushed back 10.4 units) |
| Tor swing at a plant in agro | Defended by dodge with 39 % (quick), 24 % (normal), 9 % (heavy); else 4 damage and the push-back; the swipe counter likely follows when the hero is within 15 |
| Roberta shot | 24 % dodged |
| Hero between 10 and 60 units away | The Spore Monster lobs a seed every 55 to 100 ticks until five plants live |
| Hero within 10 units | The plant backs off at 9.9 units per second and uses the swipe only inside 15 |
| Killing a Spore Monster | 35 HP; the plants it summoned stay and fight |
| Hero standing where a seed lands | The new spitter appears at the hero's feet (d under 3): it cannot spit until the hero moves out of its 3-unit band |

**Edge cases for implementers.**
- The row says shot count 5 and live limit 5; only the live limit applies. The shot count is the number of release windows the clip would need.
- The plants it summons are not counted as dead until their death clip says so; killing them frees summon slots.
- The Spore Monster aims the seed at the hero's feet, not at a free spot.
- Its first normal choice after waking is the lob whenever the hero is 10 to 60 away: the lob is the highest slot (3) and has a 99 % chance.
- The flight starts 4 ticks after the B request (s + 4, s = c + 1); nothing lands before the landing time of the Distances table (for example s + 34 at 20 units).

**What a port most likely gets wrong.**
1. Spawning five plants per attack.
2. Spawning the plants at a fixed spot or in agro without the 4-tick lead and the flight time.
3. Letting the summon limit be 5 per room instead of 5 per Spore Monster.
4. Forgetting that reserved records are consumed in record order and never reused after death.
5. Counter code 6 chance 5 read as rare: it is rolled every tick of the 57-tick reaction.
6. Giving the swipe one hit; it has four windows.
7. Missing the slow roll per hit.

**Timeline test vector (Spore Monster in agro, outer 1 hold, hero standing 28.0 units away, no seeds alive, cooldowns 0).** Tick 0: first choice: slot 3 passes (99 %): c = 0 (expected). Cooldown int[15, 60] drawn. Tick 1: B 49 requested. Tick 5: seed created (aim point at the hero's feet; speed 0.5; 56 ticks to the aim point). Tick 38: gate. Tick 39: end path: recovery int[15, 60], cooldown redrawn int[15, 60]. Tick 50: the seed lands (46 ticks after release, about 26.5 units from the Spore Monster, 1.5 short of the hero); a spitter (key 619) is bound there in agro. Next lob: the recovery and the cooldown both must be spent: at the earliest tick 54 and the latest about 100, plus a 1 % geometric wait. Lobs about every 55 to 100 ticks: seeds 2 to 5 land between ticks 104 and 150, 159 and 250, 214 and 350, and 264 and 450. With five plants alive the slot is refused (the live limit), so after the fifth seed the plant stays in the hold and uses only the swipe if the hero is inside 15. A hero standing at 28 units is hit by none of the spitters if they land within 3 units of him; once he steps 3 or more units away each spitter fires its 10-damage lob.

**Open questions.**
1. Reach geometry of the four swipe windows. Resolved: the general rule of the shared field behaviour with the distances of the Weapon reach table above. Stand-in only for a port without attachment frames (rewrite choice): a swipe touches a level hero from 1.2 to 12.6 ahead of the plant (to 16.6 from where it started).
2. Which block's cap applies to a summon landing in a region loaded into a created slot. Default if unresolved (rewrite choice): the cap of the block that holds the reserved record (needs a runtime check; the Spore Monster is the only plant whose summon can land in another region).
3. Whether a seed that touches a tree before it reaches the ground still spawns the plant there. The summon kind spawns on any release that reaches the ground rule (SPEC-0029 Behavior 9, kinds 2 to 8 release at the ground point); a world-primitive release is a plain release. Default if unresolved (rewrite choice): no plant spawns when the seed is released by a tree or wall (needs a runtime check).

### Plant Gunner

**Player's answer (R-0149).** Close in: both of its slots work down to 0 units, it dodges only 39, 24 and 9 swings in 100 (shots 24 in 100) with a 15 to 45 tick window, and its keep-distance never gives up, so the answer is simply to stay in melee range (SPEC-0038 Behavior 44).

Row 220, type 85, variant 1, spawn key 1622. A keep-distance shooter that fires a burst of four fast poison darts. Six placements, all patrols (default state 6) in Shadowglen Woods; none hidden, none in an alarm group. Confidence: High for the row, the burst arithmetic and the movement rule; Medium-High for the patrol routine.

**Identity and tunables.** HP 35; approach 3 (keep distance): desired band 10.0 to 20.0 (desired = 10.0 + 10.0 f, 12.0 to 18.0), retreat band 20.0 to 30.0 (the retreat distance is a draw from 20.0 to 30.0 made with the desired distance); radius 3.5, height 7.0; weight 5; target type 2; recovery int[60, 120] (the row stores 60 and 120 in reverse order; the draw takes either order); hold aim: the keep-distance blend 0.25 then the row turn weight 0.7 in the same tick (effective 0.78, SPEC-0038 Behavior 44); notice and search as the family table (band 5 to 100, chance 70 to 25). Weapon volumes unused by its one attack.

| k | Type | Band (strict) | Chance | Facing cosine above | Cooldown draw | Speeds (wind-up, strike, recovery) | Damage / knockback | Status | Counter |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| 0, 1, 3 | unused | | | | | | | | |
| 2 | missile, flight mode 0 (straight), four shots | 5 to 40 | 50, 49 % | 0.0 | int[120, 240] | 2.0, 1.0, 1.0 | 16 / 0 per shot | status 2 (poison), chance 25, int[600, 900] / 60 = 10 to 15 s | code 6, chance 25 |

Projectile: launch speed 2.0 units per tick, range 60 (lifetime trunc(60 / 2) = 30 ticks), radius 0.2, spawn count 4, no aim inaccuracy, no effect ids, visual kind 0.

**Animations (this kind's table).** Idle 0 is clip 0 (30 keys, loop). Patrol and wander walk 2 and 4 are clip 1 (15 keys, loop; 3.92 units per second). Keep-distance hold 25 is clip 2 (20 keys, loop). Walk toward the hero 24 is clip 3 (10 keys, loop; 15.0 units per second); back-off 26 and 27 are clip 19 (30 keys at speed byte 1, 60 ticks per cycle; 9.9 units per second backward). Slot 2: A 43 is clip 9 (15 keys, one shot), B 44 is clip 10 (43 keys, one shot, travels forward 13.0 units over the clip, release bit 0x20 on keys 7, 14, 22 and 30). Melee dodge 55 is clip 19; missile dodge 53 and 54 are clip 14 (20 keys, gate 77, 13 units sideways); hit reaction 57 is clip 18 (15 keys, gate 57, pushed back 10.41 units); death 58 is clip 13 (33 keys); victory 73 is clip 16. Emerge 10 is clip 17 and hide 9 is clip 4 but this kind is never hidden. Alert 18 and search 16 are absent.

**Start and wake.** Patrol (state 6): the path index in the record, pauses int[120, 480] on four records and int[0, 240] on two. Notice as in the family section; a pass goes to agro on the same tick (no alert animation), a failed second test starts the search timer int[60, 239] (the plant stands facing the hero's last position and re-tests at each awareness cycle).

**Per-tick order.** As SPEC-0038 Behavior 44: recovery countdown, guard reset, missile defence (24 %), melee defence, lost-sight test, normal choice (slot 2), counter choice (slot 2 during hit reactions), distance-hide gate (off, and the plant is not in outer 1 anyway), outer switch (outer 3 keep-distance sub-states), agro tail.

**State table.**

| State | Animation id -> clip | Keys | Gate | Exits |
| --- | --- | --- | --- | --- |
| Keep hold (outer 3, inner 4) | 25 -> clip 2 | 20, loop | none | d above 20.0 to inner 3; d below 10.0 to inner 5; otherwise d below 20.0 to inner 6 |
| Walk in (inner 3) | 24 -> clip 3 | 10, loop | none | d below 20.0 to inner 6 (the retreat minimum equals the desired maximum, so the desired band is never reached) |
| Back off (inner 5) | 26 -> clip 19 | 30, loop | none | d above desired to inner 4 |
| Retreat (inner 6) | 24 -> clip 3, faces away | 10, loop | none | d at or above the retreat distance to inner 4 |
| Burst (slot 2) | A 43 -> clip 9 (rate 2.0); B 44 -> clip 10; C, D, E skipped | 15, 43 | A 29, B 167 | end path |
| Melee dodge (outer 5) | 55 -> clip 19 | 30 | 60 | defence exit |
| Missile dodge (outer 6) | 53 or 54 -> clip 14 | 20 | 77 | defence exit |
| Hit reaction (outer 10) | 57 -> clip 18 | 15 | 57 | outer 0 |
| Dead | 58 -> clip 13 | 33 | 128 (0.95 at 124) | corpse released after about 425 ticks |

**Attack chooser.** Only slot 2: 5 < d < 40, facing cosine above 0.0 (the plant must face roughly toward the hero: during a retreat it faces away and cannot fire), cooldown int[120, 240] spent, recovery below 1, roll below 50 (49 % per tick). A missile slot passes the charge rule. The choice can be made in outer 1, 3, 4 and 11; with approach 3 the plant is in outer 3.

Burst: choice on c; state 0 for a missile slot continues once the hero and the steer point are on the walk mesh; s = c + 1: A 43 requested with the wind-up speed word 2.0 (the clip plays at twice the rate; gate 29); B 44 is requested on s + 29 by state 1, so the pass of that tick still runs at the wind-up word 2.0 (half a key) and the later passes at the strike word 1.0; the four shots are created on s + 56, s + 84, s + 116 and s + 148 (the first tick on which each release bit is seen, at B + 27, + 55, + 87 and + 119; each run of the bit lasts 4 ticks and the pending flag re-arms between runs, so each run fires one shot, emit point 0). After the fourth shot the sequencer goes to state 3, waits for B's gate on s + 196 (B + 167) and the end path runs on s + 197 (recovery int[60, 120], cooldown redrawn int[120, 240]). A whole burst lasts about 198 ticks. The plant does not re-aim during the burst (orient flag 0); each shot is aimed independently from the launch point to the hero's position + (0, 2.5, 0) at the moment it is created, with no scatter, plus 5.0 units of random horizontal scatter (a random unit direction times 5.0) when the plant is outside the camera cone (dot of the camera's forward vector and the direction from the camera to the plant below 0.85), plus 3.0 while it is waiting in line. During B the clip's root travel carries the plant forward 13 units in the direction it faces (4.5 units per second), within the usual chase-stop rule near the hero.

Shot: speed 2.0 per tick, lifetime 30 ticks (it reaches 60 units), radius 0.2, straight flight with no gravity, ends on the ground, the world or a character. On contact with the hero's body: 16 damage (minus shield and armour), then a draw int[0, 100] at most 25 (26 of 101) applies poison for 10 to 15 s (default 2 HP every 4 s, never below 1 HP). A blocked hit (hero guarding, shot heading against the hero's facing) deals max(1, a quarter of the reduced damage) and no poison.

**Movement (keep distance, outer 3).** The row's retreat band begins where the desired band ends (both at 20.0), so the plant never settles inside the desired band. Inner 4 (hold) sends it to walk-in when d is above 20, to back-off when d is below 10 and otherwise to retreat (inner 6) because d is below the retreat minimum 20. The walk-in (inner 3) sends it to retreat as soon as d is below 20 (the check that follows the desired-distance check always wins, since the desired distance is at most 18). The retreat runs away at 15 units per second (the forward walk clip, facing away) until d is at least the retreat distance (a draw from 20 to 30), then the hold sends it to walk in again while d is above 20. The Gunner therefore paces between 20 and its retreat distance (up to 30) from a standing hero, facing the hero during the walk-in and the hold and facing away during the retreat; one cycle takes up to about 80 ticks. It fires only while facing roughly toward the hero (walk-in and hold), not during a retreat. Back-off (d below 10, entered only when the hero closes in) moves backward at 9.9 units per second facing the hero.

Movement table: the Spore Monster's table (same clips) with the keep-distance ids: walk-in and retreat play 24 (the 15.01 per second forward clip, 0.250 per tick; the retreat turns its back by the steering blend 0.3 or 0.1, so it turns in an arc of a few ticks before running), back-off plays 26 (9.90 per second backward), hold plays 25 (no root, aim 0.25 then 0.7). Each pacing reversal therefore takes the turning arc of SPEC-0038 Behavior 51 item 6 (about 7 ticks at 0.3 to come within 10 degrees), plus one extra step at each switch (item 9).

**Hit and hurt windows.** Shot creation ticks above. The plant is hittable in every state except the two dodges and dead. A hit cancels a burst in progress without the end path; shots already in the air keep flying.

**Defence and counters.** Melee dodge 40, 25, 10 (39 %, 24 %, 9 % by swing weight), block 0; missile dodge 25 % (24 of 100); hold 60; timer int[15, 45]. Counter code 6 on slot 2 (chance 25): on every tick of the 57-tick hit reaction the counter choice finds slot 2 eligible when 5 < d < 40, the facing cosine is above 0 and the cooldown is spent, and rolls int[0, 100] below 25 (25 of 101); the burst begins within about 4 ticks of the hit and interrupts the reaction.

**Hero actions against this enemy.**

| Hero action | Result |
| --- | --- |
| Swing at a patrolling plant that has not noticed | Damage doubled (8 with the starting sword), agro and reaction kind 0 at once; HP 35 leaves 27 |
| Tor swing at a plant in agro | Dodged 39 % (quick), 24 % (normal), 9 % (heavy); else 4 damage, push-back 10.4 units (restarted on every hit), burst counter nearly certain |
| Roberta shot | 24 % dodged |
| Hero beyond 40 units | The plant never fires; it walks in while d is above 20 |
| Hero inside 5 units | No burst (the band begins at 5); the plant backs off or retreats |
| Hero walks straight at the plant | The plant retreats at 15 units per second, faster than the hero walks, and fires when it stops and turns |
| Killing a Plant Gunner | 35 HP; corpse after about 425 ticks |

**Edge cases for implementers.**
- The wind-up clip runs at twice speed because the row's wind-up speed word is 2.0; the strike clip runs at 1.0.
- Each release window gives exactly one shot; the row's count of 4 equals the four windows, so the burst ends on the fourth.
- Row recovery 60 and 120 in reverse order are one range int[60, 120].
- The plant retreats facing away, so it cannot start a burst in the middle of a retreat; the burst starts in the walk-in or the hold.
- Shots ignore the plant's facing: a plant that turned its back during the sequencer still fires toward the hero.
- The burst carries the plant 13 units forward along its facing.

**What a port most likely gets wrong.**
1. Giving the Gunner a fixed standoff distance: it oscillates between walking in and retreating.
2. Firing at once at 4 shots a tick or all four on one tick: the shots are 28 to 32 ticks apart.
3. Forgetting the 5-unit scatter when the plant is outside the camera cone.
4. Using the wind-up speed word as a stat of the strike (the 2.0 applies to the wind-up clip only).
5. Poison on every shot (25 %) or a lasting 120 s poison (10 to 15 s).
6. Letting the plant fire from d under 5 or over 40.
7. Forgetting that the burst interrupts a hit reaction by the counter.

**Timeline test vector (patrol plant, hero standing 25.0 units away, already noticed and in agro, outer 3 walking in, cooldowns 0, plant facing the hero).** Tick 0: normal choice, d = 25 inside 5 to 40: c = 0 + G (G geometric, 49 % per tick, mean 1; the plant may have walked a little closer first). s = c + 1: A requested (plays at rate 2.0). s + 29: B requested; during B the plant drifts forward about 13 units along its facing (to about 12 units from the hero). Shots at s + 56, s + 84, s + 116, s + 148; each is aimed at the hero's feet + 2.5 up at the moment of creation and hits a standing hero about 3 to 11 ticks later (speed 2.0; the emit point moves forward with the clip, Distances and ranged reach) (a standing hero in the camera view takes 16 per hit, 64 for the volley, and each hit has a 26 in 101 chance of 10 to 15 seconds of poison). s + 196: gate; s + 197: end path (recovery 60 to 120, cooldown redrawn 120 to 240). The next burst begins no earlier than s + 197 + 120 and no later than about s + 197 + 240 + waiting time: so by tick 600 the plant has fired two volleys (the second with its choice between about 320 and 440 and its shots over the following 150 ticks); a third burst begins after tick 600 in most draws. Between volleys the plant paces between 20 and 30 units as described.

**Open questions.**
1. Whether the 5 to 40 band is tested against horizontal or full 3D distance. The shared chooser distance is the horizontal one (see the Flower Monster item above). Default if unresolved (rewrite choice): horizontal (needs a runtime check).
2. Resolved (Medium-High; proactive (open close-out); same return rule as every patrol, shared rule 8 of the brigand field rules): whether a patrolling plant that is hit while walking away from the path point resumes its path from the start. After losing the hero it goes to state 0, then to the default state again, which picks the path point it had.
3. The facing used while a Gunner is retreating (it faces its move direction, away from the hero). Default: as stated.

### Plant Monster

Row 219, type 85, variant 0, spawn key 1621. A heavy melee plant with a hold-and-throw grab (no damage) and a four-hit poison claw; the player meets one, patrolling in Shadowglen Woods, and beats it with heavy swings (it dodges most quick ones) while staying out of the grab's 60-degree front. One placement in play (default state 6, Shadowglen Woods; path 6, pauses int[120, 480]), one in the cut test pack. Confidence: High for the row, the claw and the defence; Medium for the grab (see below); Medium-High for the patrol.

**Identity and tunables.** HP 50; approach 1 (rush band 3.0 to 10.0; desired 3.0 + 7.0 f = 4.4 to 8.6; back-off point 12.0 behind the hero); radius 3.5, height 7.0; weight 5; target type 4 (immune to stun: Water Motes do not stun it); recovery int[15, 60]; hold turn weight 0.7; notice and search as the family table; death 300; weapons 1 to 4 radius 4.0, length 1.0.

| k | Type | Band (strict) | Chance | Facing cosine above | Range | Cooldown draw | Speeds | Damage / knockback | Status | Counter |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| 0 | melee grab-and-throw (flag 0x08), orient 1 | 0 to 20 | 25, 24 % | 0.0 | 9.0 | int[600, 900] | 1, 1, 1 | 20 / 0 (not applied, see below) | none | code 6, chance 50 |
| 1 | melee claw, four windows | 0 to 10 | 40, 39 % | 0.9239 | 7.0 | int[120, 180] | 1, 1, 1 | 7 / 0 per hit | status 2 (poison), chance 50, int[600, 900] / 60 = 10 to 15 s | none |
| 2, 3 | unused | | | | | | | | | |

Throw speed 15.0 (slot 0). Mote drain 0.

**Animations (this kind's table).** Idle 0 is clip 0 (30 keys, loop); wander and patrol walk 2 and 4 are clip 1 (15 keys, loop; 3.92 units per second); rush walk 21 is clip 3 (10 keys, loop; 15.0 units per second); hold 22 is clip 2 (20 keys); back-off 23 and melee dodge 55 are clip 19 (30 keys at speed byte 1; 9.9 units per second backward, gate 60); missile dodge 53 and 54 are clip 14 (gate 77); hit reaction 57 is clip 18 (gate 57); death 58 is clip 13 (gates 128 and 124); emerge 10 is clip 17, hide 9 clip 4 (unused, never hidden); 81 clip 4; 82 clip 17. Slot 0: A 33 absent; B 34 is clip 5 (7 keys, one shot, gate 25, bits 0x4 and 0x8 on the last key); C 35 is clip 6 (38 keys, bit 0x4 on keys 1 to 27, bit 0x20 and 0x4 on key 28, gate 148); D 36 is clip 7 (12 keys); E 37 is clip 20 (10 keys). Slot 1: B 39 is clip 8 (30 keys, gate 116, weapon bit 1 on keys 7 to 9 and 17 to 20, weapon bit 2 on keys 12 to 15 and 22 to 25); A, C, D, E absent. Alert 18 and search 16 absent.

**Start and wake.** Patrol (state 6) as in the family section; a notice pass goes to agro on the same tick; a failed second test starts the int[60, 239] search. A hit takes agro with doubled damage and reaction kind 0.

**Per-tick order.** As SPEC-0038 Behavior 44: recovery countdown, guard reset, missile defence (24 %), melee defence (74 to 89 %), lost-sight test (not run while a grab slot's sequencer is active), normal choice (slots 1 then 0), counter choice (slot 0 during hit reactions), distance-hide gate (off), outer switch, agro tail. The held hero is released as soon as the plant's sequencer leaves its strike state.

**State table.**

| State | Animation id -> clip | Keys | Gate | Exits |
| --- | --- | --- | --- | --- |
| Rush hold (outer 1, inner 1) | 22 -> clip 2 | 20, loop | none | d below 3.0 to inner 2; d above 10.0 to inner 0 |
| Rush walk | 21 -> clip 3 | 10, loop | none | d below desired to inner 1 |
| Back off | 23 -> clip 19 | 30, loop | none | d above desired to inner 1 |
| Slot 1 claw | B 39 -> clip 8; others skipped | 30 | B 116 | end path |
| Slot 0 grab | A skipped; B 34 -> clip 5; on a catch C 35 -> clip 6 (the hold and throw); on a failed catch E 37 -> clip 20; D not played | 7, 38, 10 | B 25, C 148, E 37 | end path the tick after C's gate (catch) or E's gate (miss); the grab flag skips E after a catch |
| Melee dodge (outer 5) | 55 -> clip 19 | 30 | 60 | defence exit |
| Missile dodge (outer 6) | 53 or 54 -> clip 14 | 20 | 77 | defence exit |
| Hit reaction | 57 -> clip 18 | 15 | 57 | outer 0 |
| Dead | 58 -> clip 13 | 33 | 128 | corpse after about 425 ticks |

**Attack chooser.** Slot 1 is tested before slot 0 (slots go down from the highest used: 1, then 0). Slot 1: 0 < d < 10, cosine above 0.9239, cooldown int[120, 180], roll below 40 (39 %). Slot 0: 0 < d < 20, cosine above 0.0, cooldown int[600, 900], roll below 25 (24 %). A melee slot walks to its range first (7.0 for the claw, 9.0 for the grab) with gait 0 (15 units per second).

Claw: s = first tick of state 1. B 39 on s; four hits of 7 at s + 28, s + 49, s + 69 and s + 90 (weapons 1 and 2, each window re-arming the hit list; the second and third hits come one tick after their window opens, the fourth on its first touching tick, s + 90, for a hero 3.5 or more ahead); each hit rolls poison with a draw int[0, 100] below 50 (50 of 101) for 10 to 15 s. Gate s + 116, end path s + 117. B moves the plant forward 4.28 units.

Grab: choice on c; state 0 walks until d is at most 9.0, then state 1 (the orient flag aims the plant at the hero at weight 1.0 in states 1 to 3, except while the hero is held); B 34 requested on s. On s + 1 (the first tick of the strike state) the grab machine runs its choose step: if the plant's facing dot toward the hero is above 0.5, d is below 9.0, the hero's stun word is clear and the path to the grab point is clear, the hero is put into base state 53 (held where he stands) and a rumble of 0.7 for 1.0 s starts (repeating every 61 ticks), and in the same tick the machine requests the follow-through C (id 35, clip 6, 38 keys) at the strike word 1.0, replacing clip 5. The hero stays held while any enemy is in the strike state with a grab slot. Clip 6 carries the release bit 0x20 on key 28, first seen on s + 113: the hero is thrown with velocity normalize(15 x facing + 8 x up) x 15.0 and takes a take-hit of damage 0 (the 20 in the row is never applied: a grab deals 0 on every row). The sequencer then plays the rest of clip 6 at 1.0 (gate s + 149) and the end path runs on s + 150 (recovery int[15, 60], cooldown redrawn int[600, 900]; no recovery clip after a catch, flag 0x08). If the choose step fails, the machine's recover state requests the recovery clip E (id 37, clip 20, 10 keys) on s + 2; the strike state waits for its gate (s + 39) and the end path runs on s + 40, with no contact (Medium). So on shipped data the Plant Monster's grab is a hold of 112 ticks ended by a harmless throw.

**Movement.** Rush outer 1 only; 0.25 units per tick forward, 0.165 backward, with the chase stop in sequencer states 0 to 3. Movement table: the Spore Monster's table (same clips, same speeds, turn 0.7, radius 3.5) with the rush band 3 to 10 (desired 4.4 to 8.6, back-off below 3). Its clip 0 has no root, so its idle, hide and hold stand still (it does not creep like the Flower Monsters).

**Hit and hurt windows.** Claw hits above; grab s + 1, throw s + 113; hittable in every state except the dodges and dead. The hit reaction pushes the plant back about 10.4 units over 57 ticks.

**Weapon reach.** Conventions are those of "Weapon reach tables" in the shared field behaviour. Row 219, Medium. The window columns are computed from the clip; the hit ticks above are authoritative. The claw is the same clip and geometry as the Spore Monster's swipe.

| Slot (phase, clip) | Weapon | Window ticks (level contact) | Centre forward / up / side | Tor ahead, at the tick | Tor ahead, from the start | Tor feet vs origin |
|---|---|---|---|---|---|---|
| 1 (B, clip 8) | w1 (4.0 x 1.0) | s+28 to s+39 (s+28 to s+39) | 5.2 to 7.9 / 0.5 to 4.5 / -3.7 to 0.9 | 1.2 to 12.4 | 3.0 to 13.9 | -3.5 to 5.0 |
| 1 (B, clip 8) | w2 (4.0 x 1.0) | s+48 to s+63 (s+48 to s+63) | 3.1 to 5.6 / 0.5 to 2.9 / 0.2 to 3.7 | 1.2 to 10.0 | 4.3 to 13.0 | -4.5 to 5.0 |
| 1 (B, clip 8) | w1 (4.0 x 1.0) | s+68 to s+83 (s+69 to s+83) | 4.3 to 8.1 / 1.3 to 4.6 / -4.5 to -0.2 | 3.0 to 12.6 | 7.1 to 16.6 | -2.5 to 7.0 |
| 1 (B, clip 8) | w2 (4.0 x 1.0) | s+88 to s+103 (s+90 to s+103) | 3.3 to 5.5 / 0.2 to 2.7 / 0.3 to 4.3 | 1.4 to 10.2 | 5.8 to 14.6 | -4.5 to 3.5 |

No claw touches a level hero closer than 1.2 ahead of the plant at the tick; the 12.4 to 12.6 reach is far beyond the 3 units a stand-in point would give.

**Defence and counters.** Melee block 0; melee dodge chances 90 (quick), 80 (normal) and 75 (heavy), so 89 %, 79 % and 74 % of swings are dodged (once per swing, first plant tick that sees it, hold below 1, hero within 8.0 and 60 degrees); the dodge animation 55 (clip 19, gate 60) throws the plant backward about 9.9 units and it cannot be hit for 60 ticks; defence hold 60, timer int[15, 45]. Missile dodge 25 % (24 of 100), animation 53 or 54 (clip 14, gate 77, 13 units sideways). Counter code 6 on slot 0, chance 50: on every tick of a hit reaction the counter choice finds slot 0 eligible when 0 < d < 20, facing cosine above 0 and the cooldown is spent (the first time it is always spent), and rolls int[0, 100] below 50 (50 of 101): a hit on a Plant Monster almost always starts the grab within a few ticks, interrupting the reaction. After the grab the 600 to 900 tick cooldown prevents another for 10 to 15 seconds.

**Hero actions against this enemy.**

| Hero action | Result |
| --- | --- |
| Swing at a patrolling plant | Damage doubled (8); agro; reaction kind 0; grab counter within a few ticks |
| Tor swing at a plant in agro | Dodged 89 % (quick), 79 % (normal), 74 % (heavy); a hit does 4 damage and restarts the 57-tick push-back |
| Roberta shot | 24 % dodged |
| Hero held by the grab | Hero action state 53 for 112 ticks, then thrown (speed 15.0, no damage); a hit on the plant frees the hero at once (the take-hit hook releases a held hero) |
| Killing a Plant Monster | 50 HP; corpse after about 425 ticks |

**Edge cases for implementers.**
- The grab picks its grab point (attachment 0 to 4) from the plant's clip at the choose tick; on s + 1 the clip is on its first key, so the point is attachment 0.
- Freeing the hero by hitting the plant works only while the grab phase is 2 and the hero is held.
- The throw is reachable (the catch itself requests clip 6, which holds the release bit); its speed is 15.0 and its damage 0: the slot's damage word 20 is never applied.
- The claw's later three hits come one tick after their window opens; the first is on the window's first tick.
- The Plant Monster counters with the grab on every tick of any hit reaction, including the reaction it enters from the very first (doubled) hit.

**What a port most likely gets wrong.**
1. Applying the grab's 20 damage, or ending the hold at clip 5's gate: the catch switches to clip 6, holds 112 ticks and throws for 0 damage.
2. Forgetting that the plant dodges 89 % of quick swings with a 60-tick, 9.9-unit leap.
3. A single-hit claw (it has four windows, 28 damage in all).
4. Holding the hero until the plant's recovery ends, or letting the hero act while held.
5. Treating the grab's cooldown as 10 to 15 ticks (it is 600 to 900).
6. Giving it a stun (target type 4 blocks Water).

**Timeline test vector (agro, hero 6.0 units in front, first hit just landed on tick 0, cooldowns 0).** Tick 0: take-hit: damage doubled (8 with the starting sword, HP 42), outer 10 kind 0 (clip 18, 57 ticks), the plant faces the hero. Ticks 1 on: counter choice: slot 0 (0 < 6 < 20) passes with 50 of 101 per tick: c = 1 + G (mean 1). The grab starts: d is under 9, so state 1 runs on the next tick and s = c + 1; on s the plant requests B (clip 5). On s + 1 the hero is grabbed (base state 53) if the plant faces within 60 degrees, and clip 6 starts; on s + 113 the hero is thrown (15.0, no damage) and lands some units away; s + 149 C's gate, s + 150 end path (recovery int[15, 60], cooldown int[600, 900]). From s + 150 + recovery the claw is chosen with 39 % per tick once the hero is inside 10 (the plant walks to 7.0 first). Claw: hits of 7 at +28, +49, +69, +90, end path +117, poison on 50 of 101 of the hits. The claw's own cooldown int[120, 180] restarts at its end path. By tick 600 the hero has been held and thrown once and clawed 1 to 2 times (up to 28 damage each and, with probability 1 - 0.505^hits, poisoned), minus armour.

**Open questions.**
1. Resolved (enemy sweep): the throw is reachable. The catch requests the follow-through clip 6 in the same tick (SPEC-0038 Behavior 15), and its release bit on key 28 throws the hero on s + 113 with 0 damage.
2. Reach geometry of the claw windows. Resolved: the general rule of the shared field behaviour with the distances of the Weapon reach table above. Stand-in only for a port without attachment frames (rewrite choice): a claw touches a level hero from 1.2 to 12.6 ahead of the plant (to 16.6 from where it started).
3. Whether the grab's choose step can run on a later tick when the first fails. Resolved (Medium; proactive (open close-out)): no; the choose step runs once on the first strike tick and a failure gives the recovery clip and the end path (see the Brute-class grab timing, which is the same machine). A runtime check could only confirm it.



## Road and Ridge Brigands

Kinds: Thief (monster row 152), Bandit (row 151), Thug (row 154), Drackonack (row 144) and Giant Drackonack (row 145). The Rogue (row 153) already has its full guide in SPEC-0063 (round 0); this section only adds what is different for a field Rogue. All five kinds are class-17 common enemies (SPEC-0038). They share the think loop, pre-step, notice check, agro machine, attack chooser, sequencer, counter choice, hit hook and death hook with every other class-17 enemy; this section gives their rows, their animation tables, the numbers that follow from them, and the start behaviour that the field placements use (the arena guides only cover default state 8). Confidence: High for the row values, the animation tables, the gate ticks and the strike windows; High for the wake and notice rules; Medium-High for the wander, patrol and guard states; Medium for the Thug grab-and-throw and for hurt-shape geometry; Medium-Low where an item says so.

### Common field rules for this batch

#### Where they stand

Placements in play (reached packs and confirmed slot loads; unreached test blocks excluded), by the spawn record's default-state word. "0" is the idle default: the enemy never runs the notice check and only wakes by a hit or by a group alarm.

| Kind | Placements | Default 0 | Default 5 (wander) | Default 6 (patrol) | Default 7 (guard) | Default 8 (hidden) | Main places |
|---|---|---|---|---|---|---|---|
| Thief | 36 | 4 | 6 | 1 | 12 | 13 | Bandit Caves 16, Farm Road 7, Meridell Plains 4, Brightvale Coast 3, Drackon Ridge 3 |
| Bandit | 32 | 3 | 6 | 4 | 6 | 13 | Bandit Caves 13, Farm Road 6, Meridell Plains 5, Brightvale Coast 4 |
| Thug | 12 | 1 | 1 | 1 | 6 | 3 | Bandit Caves 4, Western Brightvale 2, Coliseum 2, Farm Road 1 |
| Drackonack | 85 | 0 | 36 | 16 | 3 | 30 | Abandoned Mines 27, Drackon Ridge 23, Bandit Caves 15, Cogham Steppes 7, Steppe Plateau 7, Coliseum 4, Park Shrine 2 |
| Giant Drackonack | 40 | 0 | 22 | 8 | 6 | 4 | Drackon Ridge 20, Cogham Steppes 5, Meriload Mine 5, Abandoned Mines 5, Park Shrine 3, Coliseum 2 |
| Rogue (field) | 16 | 4 | 6 | 0 | 3 | 3 | Bandit Caves 8, Farm Road 3, Western Brightvale 2 |

Cut placements (unreached packs, never-loaded regions): Thief 7, Bandit 5, Thug 6, Rogue 9, Drackonack 1, Giant Drackonack 1. They are not described.

When the Farm Road Thieves and Bandits exist (High): the Farm Road exterior (the pack with the find-the-petpet game, SPEC-0045) loads them from its region set 8 (region 943, the Thieves and Bandits of the road) at once when the area starts, in two story stages: whenever flag 5 is set, and when flags 5 and 4 are clear and flag 3 is set. When flag 5 is clear and flag 4 is set, it loads region set 7 (region 942, the Minion group) instead and no set 8 is loaded, so the road then holds Minions and no Thieves or Bandits from that load. With flags 5, 4 and 3 all clear neither set loads. The load needs no trigger and no area entry, it is never reset, and the characters stay until the exterior unloads (SPEC-0045, threads 18 to 20). A placement count that includes the Farm Road Thieves and Bandits therefore counts them only in the flag-5 and flag-3 stages. Default if unresolved (rewrite choice): none needed; a rewrite that omits these loads shows no Thieves or Bandits on the road after the farm.

Word meanings of the spawn record's seven-word block (offsets +0xBC to +0xD4) depend on the default state (SPEC-0038 open question 20; SPEC-0056, Placement values). The block values on disc for these kinds:

| Default | Words | Values on disc for these kinds |
|---|---|---|
| 5 wander | +0xBC radius r (float); +0xC0 and +0xC4 pause range (ticks); +0xC8 pause chance (percent) | Drackonack: r 20 to 200, usually 20, 30, 32 or 200; pause range 0 to 120 at chance 25 on 12 of 36, 30 to 260 at chance 15, 45, 50 or 60 on 4, 60 to 240 at chance 25 on 2, none on 18. Giant: r 0 to 100; pause 60 to 200 or 240 at chance 25 on 8 of 22. Bandit: r 60 or 200, pause 60 to 240 at chance 50 on all 6. Thief: r 18 to 60, pause 60 to 240 at chance 50 on 2 of 6. Thug: r 32, no pause |
| 6 patrol | +0xB8 route index into the area's path list (0 to 40 on disc, always valid); +0xBC loop mode (0 loop, other = back and forth); +0xC4 and +0xC8 point-pause range; +0xCC random-stop chance; +0xD0 and +0xD4 random-stop range | Drackonack 16 placements, Giant 8, Bandit 4, Thief 1, Thug 1. Values are per placement: point pauses of 60 to 240, 240 to 480 or 0 to 500 ticks, random-stop chances of 15 to 50 with stops of 60 to 240 or 240 to 480 ticks, and a few patrols with every value 0 (they never pause) |
| 7 guard | +0xC4 non-zero: face the home facing; zero: do not turn. +0xBC is not read by this state | +0xC4 is 0 on 27 of the 33 guard placements of these kinds and 1 on 6 (1 Drackonack, 1 Giant, 3 Thieves, 1 Thug); +0xBC holds 0 to 100 and is not read |
| 8 hidden | +0xC0 emerge distance (float); +0xC4 cue word; +0xC8 selector (0 on every placement) | Thief: 10 (1), 15 (3), 20 (1), 30 (4), 40 (2), 120 (2). Bandit: 10 (1), 15 (4), 20 (4), 30 (1), 40 (2), 120 (1). Drackonack: 5 (13), 10 (2), 15 (9), 40 (4), 60 (1), 100 (1). Giant: 10 (2), 40 (2). Thug: 0 (1) and 40 (2). The cue word is 1 on every Drackonack, on all Coliseum records and on one wave record, and 0 on the remaining brigand ambushes |

Alarm groups (spawn byte +0xB7, 255 = none): groups 1 to 6 appear on Thief, Bandit, Drackonack and Giant placements in Abandoned Mines (groups 2, 4 and 5 hold 14 hidden Drackonacks with emerge distance 5 or 15; group 3 holds Giants), Bandit Caves (groups 1, 4, 5 and 6, mixed Thief, Bandit and Drackonack), Farm Road, Meridell Plains (groups 2 to 4, one hidden Thief plus one hidden Bandit per group, emerge distance 15 to 30), Brightvale Coast, Western Brightvale and Drackon Ridge. A group number only means something inside its area.

#### Start, wake and return (applies to every kind in this batch)

1. Start. The think loop enters hidden state 8 when the default word is 8, and otherwise sets state 0, which the pre-step turns into the default state on the same tick (so default 5, 6 and 7 are entered at once). Default 0 stays in state 0.
2. State 0 (default 0): plays the idle pick (clip of id 0, or a fidget 115 to 119 when the table has it, after int[120, 600] ticks). No notice check. It leaves only by a hit (agro, damage doubled) or by a group alarm (below).
3. Notice check (states 5, 6, 7 and 2 only; SPEC-0038 Behavior 3). The awareness countdown is reloaded with the row's awareness timer and counts down once per tick, so a cycle is the timer plus 2 ticks and the check can pass on one tick per cycle (Thief 32, Bandit 62, Thug 17, Drackonack and Giant 32). On that tick: roll A int[1, 100] must be at most the awareness value; then the distance band of the row scaled by F1 of the hero's action state (0.7 standing, 0.75 slow walk, 1.5 walk, 1.75 run; SPEC-0038 Behavior 18); closer than the scaled near edge is noticed with no further roll; farther than the scaled far edge is never noticed; between them roll B int[1, 100] must be below the chance interpolated from the near chance to the far chance over the unscaled range values, halved when the hero is outside the view cone (Thief, Drackonack and Giant cosine 0.92388; Bandit and Thug 0.707) and halved again by the Tattered Cloak; a blocked sight line replaces F1 by F2. No cone gate exists inside the near edge. A hero standing still is much harder to notice than a walking one (scaled ranges 0.7 against 1.5).
4. Wake from 5, 6 or 7. A pass moves the enemy to state 2 (alerted search): it stops walking, turns to the last-seen point (turn blend is the row's ground turn value) and plays id 16 (long search idle) when the table has it. The notice check runs again in the same tick, and then again once per awareness cycle. A pass while id 18 is not playing requests id 18 (alert) and the enemy goes to agro when that clip reaches its 0.99 gate; the alert clip then plays to its end even if later checks fail. Alert clip gate ticks: Thief id 18 77, Bandit and Thug 132, Drackonack and Giant 96. With no pass the search countdown (int of float[60, 240)) runs down, only while id 18 is not playing; at zero id 17 plays (Thief 69, Bandit and Thug 156, Drackonack and Giant 108) and the enemy goes back to its default state. So noticing is two stages: a pass puts the enemy in search, and a second pass inside the 60 to 239 tick search window (the next cycle ticks) starts the alert; the chance of the second stage on the first tick is the same chance again.
5. A hit at any time (take-hit hook, SPEC-0038 Behavior 37): damage doubled once when the state was not agro, state 19, hit reaction kind 0 on the same tick, attacker turned to face. This works from states 0, 2, 5, 6, 7, 8 and 17.
6. Group alarm. When any member of an alarm group enters agro, every other member runs its alarm check at its next check tick (every int[15, 45] ticks, so up to 45 ticks later). Members in states 0, 3, 5, 6, 7 and 14 go to state 2 and play id 18 at once, then to agro when it ends (no second notice needed); members in state 8 play their emerge clip (id 10) and, when the cue word is 0, start the ambush camera and sound cue (Medium); members in any other state do nothing. A hidden member without an emerge clip goes to agro at once.
7. Hidden state 8 (SPEC-0038 Behavior 40, same as Round 0): sub-state 0 stands at home playing the hide pick id 9 (a missing id starts clip 0), walking home first when more than 0.35 away (walk id 2, state 8 uses the walk gait); wakes when the hero is closer than the emerge distance times F1 (a standing hero 0.7: distances 5, 10, 15, 20, 30, 40, 60, 100 and 120 become 3.5, 7, 10.5, 14, 21, 28, 42, 70 and 84; a running hero 1.75); sub-state 1 requests the emerge clip id 10 on the next tick; the enemy enters agro when that clip's gate passes. A distance of 0.0 never wakes. While hidden with the cue word 0 the enemy is also not hittable and not targetable (SPEC-0036 invisible rule: the hide pick is its current animation); with cue word 1 (all Drackonacks, the Coliseum, the Rogue arena) it can be hit and a hit wakes it with doubled damage.
8. Return to the default state after losing the hero. Three exits leave agro for the default state: the leash (no route to the hero and the walk waypoint 15.0 or more from the hero for more than 180 ticks), the lost-sight search timing out (counter above 120), and a victory pose. The setter then gives state 0, which on the next tick becomes the default: 5 resumes wandering from the current spot, 6 resumes the patrol, 7 walks back to the home point and stands, 8 walks home and hides again (the whole wake and emerge sequence repeats), 0 stands idle. Nothing else brings an enemy home; a hero who simply stays out of reach is chased until the leash or the lost-sight search fires.
9. Socialize (among the brigands: Thief, Bandit, Thug and Rogue carry row socialize flag 1, pair radius 20.0; the full flagged-row list, which also has the Warriors, Ixi and others, is in the shared Socialize section). While idle in state 5, 6 or 7 the quiet countdown (int of float[420, 480)) runs; at zero the enemy looks for a peer with a non-zero socialize flag, not in states 1, 2, 3, 19 or 21, whose own quiet countdown has also run out and who is within the sum of both pair radii (40.0) and on the same navmesh. Both go to state 3 (walk toward each other, stop within 6.0, turn, play a chat clip (id 14 or 111 to 114 by a draw of 0 to 4; the idle pick when the table lacks it) for a talk time of int of float[120, 180) ticks), then return to the default state. While the talk countdown is positive the notice ranges are halved. Drackonacks and Giants never socialize (flag 0). A hidden peer in state 8 is not excluded by the peer test (Medium-Low whether this can pull an ambusher out of hiding; see open questions).

#### Idle states in detail (default 5, 6 and 7)

These are the class-17 states 5, 6 and 7 (SPEC-0038 Behavior 5). Each tick the pre-step runs first (awareness countdown, notice check, socialize check, map search for the coin kinds), then the state body.

The wander (state 5), patrol (state 6) and guard (state 7) bodies are the common ones in "Shared field behaviour" at the top of this spec (Medium-High); the record values for this batch are in the table above. Two points specific to this batch: the patrol marks its route as just reached on its first tick, so the first target is route point 1; and the avoid-point retarget of SPEC-0038 Behavior 25 is live only when a script has set its scalar, which none of these kinds has been seen with (Low).

Walk speeds at play rate 1 (root travel of the clip, units per second): walk id 2 Thief 4.25, Bandit and Thug 5.63, Drackonack 7.50, Giant 3.28; patrol id 4 Thief 4.25, Bandit and Thug 3.96, Drackonack 3.28, Giant 3.28.

#### Packs, the attack manager and waiting in line

All class-17 enemies in agro share one attack-manager list (SPEC-0038 Behavior 45). With starting gear the capacity is 0, so exactly one of them (plus any entry that only has weight-0 enemies before it) may start an attack at a time; the others are held out of the sequencer and play the wait-in-line behaviour (outer sub-state 12): they walk (id 2) to a stand-off point drawn around the hero within the row's stand-off radius (Thief and Rogue 10, Drackonack and Giant 15, Bandit and Thug 20), face the hero, and re-pick a point every int[600, 800] ticks. Difficulty values: Thief, Bandit, Thug and Drackonack 8, Giant 12. Strafers (approach 4) are held back like the rest; only approach 3 enemies are exempt. A waiting enemy that is hit and rolls a counter is put back in the wait state, because the sequencer start is refused while it waits (counters are not a way around the queue). A pack of 6 therefore fights mostly one at a time until the hero wears better gear (capacity 15 allows two weight-8 attackers, 39 allows five).

#### Coins and the detour (Thief, Thug and the Rogue)

The Thief and the Rogue (map search flag 1, chance 50, greedy range 10.0) and the Thug (flag 1, chance 100, greedy range 10.0) run the coin search of SPEC-0038 Behavior 9 every tick in every state (their display-in-default value is 1, so the default-state exception does not apply): for coin kinds 135 to 139 in order, the nearest world item of the kind anywhere in the world pool; a roll int[1, 100] below the chance (Thief 49 of 100, Thug 99 of 100 per kind per tick), a route to the coin and the hero farther than 10.0 send the enemy to state 17 with that coin; the last passing kind wins. In state 17 it runs to the coin with the gait set that the previous state loaded (the walk id 2 after states 5, 7 and 8; after a fight the clip of the gait number the last agro tick left: usually the approach gait, id 28 for the Thief and Rogue and id 21 for the Thug, but id 32 (backing off) or the side clips if it was backing off or circling; see Open questions item 5; Medium-High), deletes it on arrival within 1.0 unit, and gives up at once (default state) when the hero comes within 10.0. This covers coins lying anywhere in the area, not only stolen ones: a Thief or Thug that is wandering, guarding or fighting at range leaves its post for any coin on the ground. Bandits and Drackonacks have no map search.

#### Deaths and drops (all five kinds)

Death hook and corpse as SPEC-0038 Behaviors 34 and 36: no carried Neopoints (the steal does not add to them), the death clip plays once, the corpse sinks and is removed after the death-disappear timer once the clip passes 0.95 progress: Thief, Bandit and Thug 600 ticks, Drackonack 300, Giant 420. Death clip gate ticks: Thief 73, Bandit and Thug 77, Drackonack and Giant 85. Drop placement ordinal (spawn word +0xB0): -1 on 31 of the 85 Drackonack placements in play and 19 of the 40 Giants; the rest name a placement in the area's item list (values 0 to 21). Killed enemies stay dead (SPEC-0038 Behavior 47).

### Thief

The Thief (monster type 72, variant 0, monster row 152; on-screen names Thief, Drub, Gil the Fence and Drool, all the same row) is the Rogue's cheaper twin: the same skeleton, the same strafe machine, the same Neopoint steal and the same coin detour, but with 50 hit points, a shorter fuse, a ranged shot, no three-hit combo, and its own set of clips. Read Round 0 for the machinery; this guide gives every Thief value and every place it differs.

#### Identity and tunables

| Item | Value |
|---|---|
| Row | 152; class 17; approach 4 (strafe); 50 hit points; difficulty 8; target type 0; hostile; socialize flag 1 (pair radius 20.0) |
| Body | Radius 2.0, height 2.0, weight 5.5. Hurt shape: one capsule on the root bone, radius 1.05, length 3.04, base 0.82 below the root joint (the Rogue's capsule; no other hurt capsule). Model scale is not part of the row (Medium) |
| Weapon volumes | Weapon 1 radius 3.0, full height 1.0 (attach point 0, bone 20); weapon 2 radius 5.0, full height 1.0 (attach point 1); weapons 3 to 5 size 0 |
| Notice | View cosine 0.92388, awareness 60, timer 30 (cycle 32 ticks), chance 60 at range 3 falling to 10 at range 30 (scaled by F1 of the hero's action state; standing hero: near 2.1, far 21) |
| Strafe | Desired band 6 to 16 (desired = 6 + 10 f, f = int[20, 80] / 100, so 8.0 to 14.0, redrawn after every attack choice); backbiter 1.0; turn value 0.9 |
| Recovery after an attack | int[15, 120] ticks (row timer max 120, min 15) |
| Missile defence | Hold 60, dodge 75, block 0. Melee defence: block 0 / 0 / 0, dodge 50 / 25 / 25 (quick / normal / heavy), defence timer 0 to 0, blocking angle 0, damage reduction 0, knockback 0 on every slot |
| Map search | Enabled: type 1 (coins), chance 50, distance 100, greedy range 10.0 (Round 0, "Stealing and the coin detour"; common rules above) |
| Death | Disappear timer 600; carried Neopoints 0 |
| Spawn values | 36 placements in play: default 8 on 13, 7 on 12, 5 on 6, 0 on 4, 6 on 1 (table above). Emerge cue word 0 on 11 of the 13 hidden ones: those can neither be hit nor targeted until they emerge |

Attack slots (slot 3 is unused: type 0xFFFF). All wind-up, strike and recovery speed words are 1.0, so every clip runs at its natural rate (speed byte 2: 0.25 keys per tick, 4 ticks per key).

| Slot | Kind | Band (strict) | Chance | Arc (dot above) | Range | Delay (cooldown) | Charge | Damage | Flags | Counter |
|---|---|---|---|---|---|---|---|---|---|---|
| 0 | melee | 0 to 10 | 50 | 0.92388 | 5.0 | int[30, 60] | none | 4 | none (blockable) | trigger 6, chance 25 |
| 1 | missile | 5 to 50 | 50 | 0.966 | 40 (unused by a missile) | int[30, 120] | none | 7 | none | none |
| 2 | melee | 5 to 20 | 50 | 0.92388 | 5.0 | int[60, 120] | kind 2, distance float[15, 25) | 1 | steal | trigger 6, chance 25 |

Slot 1 shot: flight mode 0 (straight), speed 1.2 units per tick, range 75 (life trunc(75 / 1.2) = 62 ticks), radius 0.2, inaccuracy 0, spawn count 1, visual kind 1, no status, no area. Steal slot 2: Neopoints int[10, 100], chance 50, selector 0.

| Slot | Phase ids and clips |
|---|---|
| 0 | A 33 none; B 34 = clip 8 (15 keys, loops, weapon 1 on keys 5 to 12); C 35 none; D 36 = clip 11 (11 keys); E 37 none |
| 1 | A 38 = clip 19 (15 keys, one-shot, gate 57); B 39 = clip 9 (20 keys, one-shot, release bit on key 12, gate 77); C, D, E (40 to 42) none |
| 2 | A 43 none; B 44 = clip 10 (12 keys, loops, run); C 45 = clip 33 (18 keys, one-shot, weapon 2 on keys 4 to 6, gate 69); D 46 = clip 11; E 47 none |

Animation ids and what the Thief's table gives them (clips from its own 166-entry table; one table serves every Thief placement in play):

| Id | Use | Clip |
|---|---|---|
| 0, 7 | idle, guard stand | clip 0 (25 keys, loops) |
| 2, 4 | walk (states 5, 7, 8) and patrol walk | clip 22 (15 keys, loops, 4.25 u/s) |
| 3, 6, 8 | rest fidget, patrol fidget, guard look | clips 18 (37 keys), 18, 19 (15 keys) |
| 9, 10 | hide pick, emerge | clip 37 (10 keys, one-shot), clip 36 (15 keys, one-shot, gate 57, travels 11.1 units forward) |
| 16, 17, 18 | search idle, search end, alert | clips 30 (45 keys), 31 (18 keys, gate 69), 32 (20 keys, gate 77, travels 4.2 units) |
| 28 | gait 0 (approach, charge, coin run) | clip 7 (12 keys, loops, 10.95 u/s) |
| 32 | gait 3 (back step) | clip 6 (12 keys, loops, 11.58 u/s backward; unlike the Rogue's one-shot hop it keeps going) |
| 29, 30 | circles | clips 20 and 21 (8 keys, loops, 6.5 u/s sideways) |
| 31 | strafe hold | clip 5 (23 keys, one-shot, gate 89) |
| 53, 54 | missile dodge | clips 16, 17 (13 and 12 keys, 10.6 units sideways, gates 49 and 45) |
| 55 | melee dodge | clip 15 (20 keys, one-shot, 10.0 units backward, gate 77) |
| 56 | block | none (never used) |
| 57 | hit reaction kind 0 | clip 25 (11 keys, one-shot, 6.2 units backward, gate 41) |
| 58 | death | clip 24 (19 keys, gate 73) |
| 70, 71, 73 | intro, victory, victory pose | clips 35 (38 keys), 12 (28 keys), 12 (gate 108) |
| 111 to 114 | chat | none (idle pick) |
| 115 to 119 | idle fidgets | clips 26, 34, 29, 13, 19 |

#### Start and wake

Default 8 (13 placements): hidden at home, wakes inside emerge distance x F1 (common rules). The emerge clip id 10 exists, so the full sequence is: wake test tick w (sub-state 1), tick w + 1 requests the emerge clip, the clip carries the Thief about 11 units forward over its 58 ticks, tick w + 58 sets agro, w + 59 is the set-up tick (outer 4, hold 0), and the first attack choice can happen on w + 60. Unlike the arena Rogue there is no skipped emerge. Default 5, 6 and 7 (6, 1 and 12 placements): the idle states and notice rule of the common rules; the alert clip is 77 ticks, so a notice at tick n starts agro at about n + 78 plus the wait for the second notice pass. Default 0 (4): idle until hit or alarmed.

#### Per-tick order (agro)

As Round 0: pre-step (awareness, alert flag, retarget, alarm check, coin search), agro machine (recovery countdown, guard reset, missile defence, melee defence, lost-sight test, normal choice, counter choice, distance-hide gate that does nothing, outer sub-state switch, leash), class-19 handler (defence hold, hold-state facing and id 31, idle pick), animation pass, motion pass. A slot picked in tick c runs its first sequencer state in tick c.

#### Attack chooser

Slots are tested 3 (unused), 2, 1, 0, once per tick while the hero is attackable, the outer sub-state is 1, 3, 4 or 11 (the Thief's movement state is outer 4) and the recovery countdown is below 1. A slot passes when its cooldown is below 1, d is inside its band (strict), the facing dot is above its arc, a roll int[1, 100] is below its chance (the backbiter is 1.0, so the roll threshold is the chance whatever the hero faces), and: slot 2, a route to the hero exists; slot 1, always; slot 0, a route exists or d is below 10.0. One roll is drawn per slot that passes the earlier tests, in order, and the first success ends the test. All three chances are 50 (49 of 100).

| Distance d to the hero (all cooldowns 0, hero in front) | Slots in play | Probability that an attack starts in a tick |
|---|---|---|
| 0 < d <= 5 | 0 | 49 % |
| 5 < d < 10 | 2, 1, 0 | 1 - 0.51 x 0.51 x 0.51 = about 87 % (slot 2 49 %, slot 1 25 %, slot 0 13 %) |
| 10 <= d < 20 | 2, 1 | about 74 % (slot 2 49 %, slot 1 25 %) |
| 20 <= d < 50 | 1 | 49 % (the arc for slot 1 is 15 degrees) |
| d >= 50 | none | 0 |

On a choice: the slot's cooldown is drawn (slot 0 int[30, 60], slot 1 int[30, 120], slot 2 int[60, 120]), the sequencer starts in state 0 (slots 0 and 1) or 6 (slot 2), and the desired and charge distances are redrawn from the chosen slot's ranges (slot 2 gets float[15, 25); a normal choice of slot 0 or 1 sets slot 2's charge distance to 0). The counter choice does not redraw them (counters below).

#### State table

Ticks are relative to the tick s in which the state requests its clip; "gate" is the first tick in which the clip's progress is at least 0.99.

| State | Action id and clip | Keys | Speed | Gate and exits |
|---|---|---|---|---|
| Hidden (class 8, sub 0) | 9 -> clip 37 | 10, one-shot | 1.0 | Wake rule; the clip plays once and holds its last pose |
| Emerge (class 8, sub 1 and 2) | 10 -> clip 36 | 15, one-shot | 1.0 | Gate s + 57, then agro the same tick |
| Wander / patrol / guard | 2 / 4 / 7 (see common rules) | | | Notice leads to state 2 |
| Search (class 2) | 16 -> clip 30, then 18 -> clip 32 or 17 -> clip 31 | 45 / 20 / 18 | 1.0 | 18: gate s + 77 then agro; 17: gate s + 69 then default state |
| Agro set-up (outer 0) | agro idle pick 1 (absent: clip 0, the same idle clip) | 25 | 1.0 | One tick, then outer 4 with inner 10 and hold 0 |
| Strafe hold (outer 4, inner 10) | 31 -> clip 5 | 23, one-shot | 1.0 | Hold timer int[60, 180]; then the circle or approach tests |
| Strafe approach (inner 7) | 28 -> clip 7 | 12, loops | 1.0 | Until d is below the desired distance |
| Circle (inner 8 travels to the Thief's right, inner 9 to its left) | 30 -> clip 21 / 29 -> clip 20 | 8, loops | 1.0 | Hold timer; d above 16 gives inner 7, d below 6 gives inner 11 |
| Back off (inner 11) | 32 -> clip 6 | 12, loops | 1.0 | Until d is above the desired distance (it keeps travelling at 11.6 u/s) |
| Slot 0 swing | 34 -> clip 8 | 15, loops | 1.0 | Window from s + 20; one hit on first contact; row 152 exact chase-stop samples: s+26 at d=3.0–4.5, s+27 at 5.0, s+28 at 5.5 and s+29 at 6.0 for fixed level Tor straight ahead. Recompute the gate/end path for the actual hit tick; s+59/s+60 remains the d=3.0–4.5 example only. |
| Slot 1 shot | A 38 -> clip 19, then B 39 -> clip 9 | 15 / 20 | 1.0 | A gate s + 57, B requested the same tick (call it b = s + 57), shot at b + 48 = s + 105, B gate b + 77, end path b + 78 = s + 135 |
| Slot 2 charge run (state 7) | 44 -> clip 10 | 12, loops | 1.0 | 10.95 u/s until the hero is within 5.0 (then state 8) or the charge distance is travelled (then state 12) |
| Slot 2 strike (state 8) | 45 -> clip 33 | 18, one-shot | 1.0 | Hit at c0 + 16; gate c0 + 69; end path c0 + 70 (c0 = tick the strike clip is requested) |
| Melee dodge (outer 5) | 55 -> clip 15 | 20, one-shot | 1.0 | Gate s + 77, then the defence exit |
| Missile dodge (outer 6) | 53 -> clip 16 or 54 -> clip 17 (int[0, 1]) | 13 or 12 | 1.0 | Gate s + 49 or s + 45, then the defence exit |
| Hit reaction kind 0 (outer 10) | 57 -> clip 25 | 11, one-shot | 1.0 | Gate s + 41, then outer 0 |
| Victory pose (class 10) | 73 -> clip 12 | 28 | 1.0 | After a strike leaves the hero at 0 HP; at the gate the default state runs |
| Coin fetch (class 17) | the gait set left by the previous state | | | Common rules |
| Dead (base state 16) | 58 -> clip 24 | 19, one-shot | 1.0 | Plays once; the disappear countdown (600) starts at progress 0.95 |

Defence exit: outer 0, base state 1, defence hold int[0, 0] = 0.

#### Movement

The strafe rules are the Rogue's (Round 0, "Movement rules") with the band 6 to 16 in place of 6 to 20: inner 10 holds still (face the hero, id 31) for int[60, 180] ticks and then rolls int[1, 3] (1 stay, 2 circle inner 8, which travels to the Thief's right, 3 circle inner 9, which travels to its left; SPEC-0038 Behavior 51 item 7; a circle is refused while the lock is above 0); d above 16 gives the approach (inner 7, 10.95 u/s), d below 6 with no lock gives the back off (inner 11, 11.58 u/s; it does not stop after one hop). Inside 6 to 16 the Thief does not close in by itself; the sequencer does the closing in for melee slots (state 0: gait 0 at 10.95 u/s until d is at most the range 5.0; the step is dropped within the sum of the body radii, 2.0 plus the hero's 1.175). The hero outruns it (walk 13.5 and run 19.3 u/s against 10.95). Slot 2's charge (state 7) runs straight, never steers, and stops at the drawn distance of 15 to 25 units, 82 to 137 ticks of running. The slot 1 wind-up does not turn the Thief (the orient flag is 0): it stands where it chose and shoots at the hero's position of the release tick.

Movement table (row 152; the Rogue in the field differs as noted in its section; ground; radius 2.0; ground turn 0.9; strafe band 6 to 16, desired 8 to 14):

| Mode | Source | Clip (keys) | u/tick | u/s | Turn | Exit |
| --- | --- | --- | --- | --- | --- | --- |
| Wander, patrol, walk home | root | 2 or 4 (15) | 0.0708 | 4.25 | steer | arrival 0.5, notice |
| Wait in line | root | 2 (15) | 0.0708 | 4.25 | steer, then aim | waiting ends |
| Approach (inner 7), above 16; sequencer state 0 | root | 28 (12) | 0.1825 | 10.95 | steer | d below desired; state 0: d at most 5.0 |
| Hold (inner 10) | none (keys sway 0.02) | 31 (23) | 0 | 0 | aim 0.9 | int[60, 180] ticks, then the roll |
| Circle inner 9 / inner 8 | root, sideways | 29 / 30 (8) | 0.108 / 0.110 (0.07 to 0.15 per tick inside the cycle) | 6.47 / 6.58 | snap to the hero | hold count, blocked (lock 120), d out of the band |
| Back off (inner 11), below 6 | root, facing the hero | 32 (12) | 0.193 backward | 11.58 | steer, back gait | d above desired or blocked |
| Charge (slot 2) | root | 44 (12) | 0.1825 | 10.95 | none (straight) | drawn 15 to 25 covered, or blocked |
| Waking (decloak or emerge 82 or 10) | root | (15) | | forward 11.10 over 60 ticks | | gate |
| Hit (reaction kind 0) | root | 57 (11) | | back 6.02 over 44 ticks | snap to attacker | gate |
| Dodge 53 / 54, 55 | root | (13), (12), (20) | | side 10.81 over 52 / 10.85 over 48; back 10.00 over 80 | | gate |
| Confused (outer 11) | root | 69 (75, one-shot) | | 5.59 forward and 3.60 sideways over 300 ticks | aim at a random direction 0.05 | status end |
| Corpse | none | 58 | 0 | 0 | kept | timer 600 |

Worked example (circling). A Thief 11 units from a still hero rolls 2 (inner 8) and the hold redraws 120: it slides along its right side at 0.110 per tick, re-facing the hero every tick, and covers 13.2 units of arc in 120 ticks (about 69 degrees around the hero) while its distance grows by less than 0.1. A wall push ends the circle at once (inner 10, lock 120).
#### Hit and hurt windows

| Attack | Window (ticks after the request tick) | Hits | Damage per hit before the hero's armour and shield |
|---|---|---|---|
| Slot 0 (clip 8) | s + 20 to s + 51 | one, on the first touching tick (row 152 exact level straight-ahead samples: s+26 at d=3.0–4.5; s+27 at 5.0) | 4, blockable |
| Slot 2 (clip 33, weapon 2, c0 = request of C) | c0 + 16 to c0 + 27 | one, at c0 + 16 | 1 (0 after any body armour), blockable, steal roll 50 of 101, int[10, 100] Neopoints |
| Slot 1 (shot) | release at b + 48 (frame 12 is seen on b + 48 to b + 51; one projectile) | one shot, 1.2 units per tick for up to 62 ticks, radius 0.2, aimed at the hero's position plus 2.5 up, plus U[0, 5.0] scatter when the Thief is outside the camera cone and 3.0 more when it waits in line | 7, pool projectile rules of SPEC-0029 Behavior 9 |

He can be struck while hidden (cue word 1 only), walking, circling, holding, attacking, recovering, reacting or fetching coins; not during a melee dodge or missile dodge (outer 5 and 6), not dead. A hit while attacking cancels the attack (counters below). A hurt, knocked-down or dead hero is not attackable: no new choice and cooldowns stop, but a running swing continues and can land.

#### Weapon reach

Conventions are those of "Weapon reach tables" in the shared field behaviour. Row 152, Medium.

| Slot (phase, clip) | Weapon | Window ticks (level contact) | Centre forward / up / side | Tor ahead, at the tick | Tor ahead, from the start | Tor feet vs origin |
|---|---|---|---|---|---|---|
| 0 (B, clip 8) | w1 (3.0 x 1.0) | s+20 to s+51 (s+20 to s+51) | -0.5 to 2.8 / 0.7 to 1.3 / -2.1 to -0.7 | -1.8 to 4.0 | -1.1 to 6.4 | -5.0 to 4.0 |
| 2 (C, clip 33) | w2 (5.0 x 1.0) | sC+16 to sC+27 (sC+16 to sC+27) | 0.6 to 3.8 / 1.2 to 3.7 / 0.0 to 1.5 | -2.4 to 9.2 | -1.4 to 10.4 | -6.5 to 6.0 |

The slot 0 swipe is short: 4.0 ahead at most at the tick. The steal swing reaches 9.2 ahead because it is a lunge.

#### Defence and counters

- Melee defence: decided once per hero swing, on the first Thief tick on which the hero's swing query is true, only when the defence hold is below 1, the Thief is not in outer 10 and not in the middle of a sequencer state other than 0, 1 and 5, the hero is within 8.0 and inside 60 degrees. Two rolls int[1, 100]: the block roll uses chance 0 and never passes; the dodge passes when the second roll is below 50 (Tor quick: 49 of 100) or below 25 (normal and heavy: 24 of 100). A dodge sets outer 5, hold 60 and plays id 55 at once (gate s + 77, hop back about 10 units, unhittable throughout). Tor's power step (sub-state 10) is never dodged.
- Missile defence: a hero shot whose 60-tick path meets his body: roll int[1, 100] below 75 gives outer 6 (id 53 or 54, gate s + 49 or s + 45), hold 60.
- No block: he is never a guarding target, so every swing that reaches his capsule hits, from any side (SPEC-0036 Hit detection step 4 never applies).
- Hit reaction: every hit sets outer 10, kind 0 (id 57, 41-tick gate), restarts the clip on a repeated hit, and no knockback. The first hit on a non-agro Thief is doubled.
- Counters (trigger 6, every tick of the reaction, no other gate): slot 0 is eligible when its cooldown is below 1 and d is below 10 with the hero within 22.5 degrees; slot 2 when its cooldown is below 1 and 5 < d < 20 with the hero within 22.5 degrees; the later slot wins (slot 2 over slot 0); one roll int[0, 100] per tick below 25 (25 of 101) starts it. With a slot eligible the Thief counters inside the 41-tick reaction with probability about 99.999 percent and on average about 4 ticks in. A counter by slot 2 uses the stale charge distance of the last normal choice: after a normal choice of slot 0 or 1 it is 0 and the charge ends at once unless the hero is within 5.0 (no run, no hit); after a normal choice of slot 2 it is the drawn 15 to 25. Slot 1 has no counter. The Thief is only staggered for the full 41 ticks when no slot is eligible: the hero inside 5 with slot 0 on cooldown, or behind it.
- Stealing and coins: as Round 0. After a damaging slot 2 strike (needs a strike result of at least 1, so an armoured hero is never robbed) a roll below 50 (50 of 101) takes int[10, 100] Neopoints (limited to the wallet) and spawns the coins at the hero's position. Then the coin detour of the common rules applies: once the hero is farther than 10.0 it runs to the coins, deletes them, and walks home.

#### Hero actions against this enemy

| Hero action | What the Thief does |
|---|---|
| Stands still beyond the notice ranges (or beyond the emerge distance) | Nothing: wanders, patrols, guards, idles or stays hidden |
| Walks inside the scaled notice range | Notice chance per cycle (32 ticks) 0.6 x chance(d) in the cone; then two-stage wake as in the common rules |
| Tor quick swing | One defence evaluation at the swing's start; dodge 49 of 100 if within 8.0 and 60 degrees, else the swing can hit |
| Tor normal or heavy swing | Same evaluation, dodge 24 of 100 |
| Tor power step | Never dodged; damage by the power table |
| Roberta's near swat | As Tor's quick swing |
| Roberta's shots | Missile defence: 74 of 100 dodge when the path touches within 60 ticks, otherwise the shot hits; a shot cannot bounce off him |
| A landed hit | Damage by SPEC-0036 (no defence); doubled when not in agro; reaction kind 0; counter roll each tick when a slot is eligible |
| Hero guard (shield) | Does not change his AI. It blocks slots 0 and 2 when the hero faces him (dot at most -0.5); the shot (slot 1) follows the enemy projectile rules of SPEC-0029 Behavior 9 and SPEC-0036 |
| Hero hurt, knocked down or dead | No new choice, cooldowns stop, a running swing continues |
| Hero swap | Next tick every test uses the new hero |
| Hero moves out of the band during a swing | A melee approach (state 0) keeps chasing at 10.95 u/s; a running swing plays out and misses; a charge runs its distance |

#### Edge cases for implementers

- The recovery countdown is drawn from 15 to 120, so after a swing, shot or charge the Thief may stand strafing for up to two seconds before it chooses again; the slot cooldowns run in parallel (the chosen slot's cooldown is drawn at the choice and again at the end path, so it restarts when the attack ends).
- A shot that fires at b + 48 travels the hero's range at 1.2 per tick: at 16 units it arrives 10 ticks later (the emit point is 1.83 in front and 3.51 up). The shot is aimed once, at release; a hero who moves in the 105 ticks before release is not tracked during the wind-up, only at the release tick.
- Missing phases are skipped in the same tick (slot 0 has only B, slot 2 has B and C): playing clip 0 for ids 33, 35, 37, 43 or 47 is wrong.
- A dodge or hit reaction aborts the sequencer without an end path: the recovery countdown is not started and the chosen slot keeps the cooldown drawn at the choice.
- A strike that leaves the hero at 0 hit points sends the Thief to the victory pose (id 73, gate 108), after which it runs the default-state step.
- A hidden Thief with cue word 0 is not hittable and not targetable (a shot or lock-on passes it) until its emerge clip starts; with the Rogue-style cue word 1 it is hittable and a hit wakes it with doubled damage.
- Group alarm wake of a hidden Thief: the emerge clip id 10 plays (58 ticks), with the ambush cue when the cue word is 0.
- The Thief fetches any coin on the ground (even one dropped by another enemy) while the hero is more than 10.0 away, from any state, 49 of 100 per tick per coin kind; a Thief that is wandering or patrolling therefore breaks its route for coins.

#### What a port most likely gets wrong

1. Treating the Thief as a Rogue with fewer hit points: it has no slot 3 combo, it shoots (7 damage at up to 50 units, after a 105-tick delay), and its melee hits are 4 and 1.
2. Skipping the shot's 57-tick wind-up (clip 19) or dropping the release frame: the first shot arrives about 120 ticks after the choice.
3. Giving the charge the Rogue's strike clip: the Thief's charge strike is clip 33 with a 12-tick window starting 16 ticks in, on weapon 2 (radius 5.0).
4. A one-hop back step. The Thief's back step loops and carries it away at 11.6 u/s while the back-off state lasts.
5. Waking from the default states with one roll; it needs two passes of the notice check with the search window between them, and the alert clip is 77 ticks.
6. Ignoring the coin detour from the idle states, and the cue-word rule that keeps ambushing Thieves untargetable.
7. Counter charges with a stale distance of 0.
8. Forgetting the shared attack manager: in a pack only one Thief or Bandit attacks at a time at starting gear.

#### Timeline test vector

Hero standing (F1 0.7), a hidden Thief (default 8, emerge distance 40, cue word 0, alarm group none) 27 units away, hero never moves. Tick numbers count from the wake test; random draws are shown as ranges.

| Tick | Event |
|---|---|
| 0 | 27 < 40 x 0.7 = 28: sub-state 1 |
| 1 | Emerge clip 36 requested (gate 57); the Thief leaps about 11 units toward the hero over the clip: d is about 16 at tick 58 |
| 58 | Gate passes; class state 19 set |
| 59 | Set-up tick: outer 4, inner 10, hold 0 |
| 60 | First choice tick: slot 2 49 %, else slot 1 25 %, else none (26 %); each later tick the same until one passes; also inner 10 rolls (d about 16 is inside 6 to 16: no approach) |
| 60 + G | Slot 2 chosen (G geometric): charge starts (state 6 skips A, B requested, state 7 the next tick); run at 0.1825 per tick, reaches d <= 5 about 60 ticks later |
| about 122 + G | State 8: clip 33 requested (c0); hit at c0 + 16 = about 138 + G (damage 1, steal roll); gate c0 + 69; end path c0 + 70 = about 192 + G; recovery int[15, 120]; slot 2 cooldown int[60, 120] counted from c0 + 70 |
| 60 + G | Alternatively slot 1 chosen: A requested, gate at 117 + G, B requested the same tick, shot released at 165 + G (7 damage if it hits), B gate 194 + G, end path 195 + G |
| 195 to 330 | Next choice from end path + N (N = 15 to 120): over 600 ticks a still hero is attacked about 2 to 3 times from range, with the charge hitting for 1 and the shot for 7 |

If the hero walks in and swings at the hidden cue-0 Thief, nothing happens until the wake test has passed (not hittable). A swing that lands after the wake tick is doubled and sends the Thief straight to agro, cutting the emerge short; it then reacts for 41 ticks and counters with 25 of 101 per tick when slot 0 or 2 is eligible.

#### Open questions

1. Resolved (Medium-High; proactive (open close-out)): a hidden cue-0 ambusher is not drawn, because the same hidden test that makes it untargetable and unhittable (SPEC-0036 invisible rule) is part of the actors' drawable predicate (see the hidden-minion item in the Minion Grunt section). Whether the main model submission uses that predicate needs a runtime check. Rewrite choice: do not draw it until the emerge clip starts.
2. Model scale of the hurt capsule: Resolved (Medium-High; proactive (open close-out)): scale 1; neither the row nor the spawn record carries a scale and the spawn bind applies none (SPEC-0006; SPEC-0038 Open question 94). Use the Rogue's capsule values unchanged.
3. The second-stage wake chance: Resolved (High; proactive (open close-out)): the state-2 notice check on the entry tick is a separate roll with fresh draws from the pre-step's pass that entered the state (SPEC-0038 Behavior 5, state 2). So there are two independent passes on the first tick (the second is the same chance again), then one per awareness cycle.
4. Weapon contact geometry (as Round 0): Resolved by the general rule of the shared field behaviour. The distances are in the Weapon reach table. Stand-in only for a port without attachment frames (rewrite choice): count a contact on a window tick when a level hero is between 1.8 behind and 4.0 ahead of the Thief (slot 0) or between 3.0 behind and 9.2 ahead (slot 2) at the tick.
5. The gait left for the coin run after a fight: Resolved (Medium-High; proactive (open close-out)). Entering state 17 changes no mode word and no gait-set word, and the run plays the clip of the gait number that the last gait call left, with the facing rule of that gait. So after a fight the clip depends on the last moving sub-state: the approach or a sequencer closing step leaves gait 0 (the strafe set's word 0, id 28 for the Thief and Rogue, id 21 for the Thug); a back-off leaves gait 3 (id 32 for the Thief; the enemy then faces away from the coin and runs backwards toward it); a circle leaves gait 5 or 6 (ids 29 and 30, with the side-gait facing); the hold sub-state makes no gait call and keeps the earlier one. After the idle states 5, 7 and 8 it is the walk id 2. Rewrite: store the last gait number per enemy and use it unchanged.

### Bandit and Thug

One guide for both: the Bandit (monster type 71 variant 0, row 151; on-screen names Bandit, Sid the Vicious, Lump) and the Thug (type 73 variant 0, row 154; Thug, Thud, Korvin the Krusher) are rush-approach melee brawlers on the same class-17 machine and nearly the same animation table (42-bone human skeleton, 43-clip set; the ids below give the same clip for both unless a column says otherwise). They differ in row values, in the Bandit's three blockable swings against the Thug's grab-and-throw, and in the Thug's dead slot. Both can guard against the hero's swings and shots, which no other kind in this batch does.

#### Identity and tunables

| Item | Bandit (row 151) | Thug (row 154) |
|---|---|---|
| Hit points | 60 | 120 |
| Body: radius, height, weight | 2.0, 4.0, 5.5 | 2.5, 5.0, 3.0 |
| Class, approach | 17, approach 1 (rush) | 17, approach 1 (rush) |
| Difficulty, target type | 8, 0 | 8, 0 |
| Hurt shape | One capsule on bone 1: radius 0.97, length 2.01, base at the bone | One capsule on bone 1: radius 1.41, length 2.93, base offset (0, -0.09, 0.05) from the bone |
| Weapon volumes (finite cylinders; attach point 0 is bone 29, point 1 bone 11) | Weapon 1: radius 4.0, full height 1.5 | Weapon 1: radius 5.0, height 1.5; weapon 2: radius 3.0, height 1.0 |
| Notice | Cone cosine 0.707, awareness 90, timer 60 (cycle 62), chance 70 at range 1 falling to 5 at range 30 | Cone 0.707, awareness 100, timer 15 (cycle 17), chance 90 at range 5 falling to 10 at range 40 |
| Rush band | Rush melee distance 3 to 6; desired = 3 + 3 f, f = int[20, 80] / 100 (3.6 to 5.4, redrawn after every attack choice) | same |
| Turn value | 1.0 | 0.5 |
| Stand-off radius (wait in line) | 20 | 20 |
| Recovery after an attack | int[60, 180] | 0 to 0 (none) |
| Missile defence | Hold 10, dodge 0, block 50 | same |
| Melee defence | Block 50 / 40 / 30 (quick / normal / heavy), dodge 0, defence timer int[30, 120], blocking angle 0.92388 | Block 50 / 30 / 30, dodge 0, defence timer int[30, 120], blocking angle 0.92388 |
| Socialize | flag 1, pair radius 20 | flag 1, pair radius 20 |
| Map search | none | Type 1 coins, chance 100, greedy range 10 (common rules) |
| Death | Disappear 600 | Disappear 600 |
| Placements in play | 32: default 8 on 13, 5 on 6, 7 on 6, 6 on 4, 0 on 3 | 12: default 7 on 6, 8 on 3, 0, 5 and 6 on 1 each |

Attack slots (slot numbers from 0; test order 3, 2, 1, 0). Speeds are the wind-up, strike and recovery words.

| Slot | Kind | Band (strict) | Chance | Arc | Range | Delay | Damage | Speeds | Flags | Counter |
|---|---|---|---|---|---|---|---|---|---|---|
| Bandit 0 | melee | 0 to 10 | 75 | 0.92388 | 7.0 | int[60, 180] | 10 | 1 / 1 / 1 | none | none |
| Bandit 1 | melee | 0 to 6 | 50 | 0.92388 | 5.0 | int[60, 120] | 15 | 1 / 1.25 / 1 | none | trigger 6, chance 25 |
| Bandit 2 | melee | 0 to 15 | 20 | 0.92388 | 7.0 | int[60, 120] | 20 | 1 / 1 / 1 | none | none |
| Thug 0 | melee | 0 to 10 | 25 | 0.92388 | 7.0 | int[60, 180] | 10 | 1 / 1 / 1 | none | none |
| Thug 1 | melee | 0 to 6 | 25 | 0.92388 | 5.0 | int[60, 180] | 15 | 1 / 1.25 / 1 | none | none (trigger 0) |
| Thug 2 | melee | 0 to 15 | 20 | 0.92388 | 7.0 | int[60, 240] | 25 | 1 / 1 / 1 | none | none |
| Thug 3 | melee, grab-and-throw | 0 to 15 | 75 | 0.92388 | 6.0 | int[120, 240] | 0 (steal) | 1 / 1 / 1 | steal and grab (0x04 and 0x08) | trigger 6, chance 50 |

No slot has a charge, knockback, status or projectile. Slot 3 of the Bandit is unused; slot 3 of the Thug is the grab. Thug steal: Neopoints int[20, 200], chance 50, selector 0; throw speed 15.

| Slot | Phase ids and clips |
|---|---|
| Bandit / Thug 0 | A 33 none; B 34 = clip 12 (19 keys, one-shot, weapon 1 on keys 7 and 8, root 1.6 units forward); C 35 none; D 36 = clip 33 (Bandit, 13 keys) or clip 13 (Thug, 13 keys); E 37 none |
| Bandit 1 | A 38 none; B 39 = clip 14 (20 keys, weapon 1 on keys 7 to 9); C 40 none; D 41 = clip 34 (10 keys); E 42 none |
| Bandit / Thug 2 | A 43 = clip 38 (4 keys, gate 13); B 44 = clip 37 (12 keys, weapon 1 on keys 1 to 3, root 2.1 forward); C 45 none; D 46 = clip 35 (Bandit, 9 keys; none for the Thug); E 47 none |
| Thug 1 | None of the five ids has a clip: ids 38 to 42 are all missing |
| Thug 3 | A 48 none; B 49 = clip 16 (10 keys, one-shot, weapon 2 on key 9); C 50 = clip 18 (32 keys, one-shot, weapon 2 on keys 0 to 24, release bit on key 25, gate 124); D 51 = clip 17 (14 keys); E 52 = clip 39 (35 keys, gate 136) |

Animation ids and clips (identical for both unless stated; speed byte 2, 0.25 keys per tick, except clip 0):

| Id | Use | Clip |
|---|---|---|
| 0, 7 | idle and guard stand | clip 0 (40 keys, speed byte 1: 0.5 keys per tick, 80 ticks a pass, loops) |
| 2 / 4 | walk (states 5, 7, 8) / patrol walk | clip 1 (12 keys, loops, 5.63 u/s) / clip 31 (15 keys, loops, 3.96 u/s) |
| 3, 8 / 6 | rest fidget and guard look / patrol fidget | clip 3 (11 keys) / clip 2 (35 keys) |
| 9 | hide pick | Bandit clip 28 (20 keys, one-shot, gate 77); Thug none (clip 0 plays) |
| 10 | emerge | clip 29 (15 keys, one-shot, gate 57, 6.1 units forward) |
| 16, 17, 18 | search idle, search end, alert | clips 8 (50 keys), 9 (40 keys, gate 156), 10 (34 keys, gate 132) |
| 21 | rush forward (gaits 0 and 2) | clip 32 (8 keys, loops, 12.6 u/s) |
| 22 | rush hold | clip 30 (16 keys, loops) |
| 23 | rush back step | clip 11 (9 keys, loops, 11.2 u/s backward) |
| 28 to 32, 53 to 55 | strafe set and dodges | none (never requested: the approach is rush and the dodge chances are 0) |
| 56 | block | clip 19 (10 keys, one-shot, gate 37) |
| 126 | block-hit | clip 36 (8 keys, gate 29) |
| 57 | hit reaction kind 0 | clip 20 (11 keys, one-shot, gate 41) |
| 58 | death | clip 21 (20 keys, gate 77) |
| 73 | victory pose | clip 27 (33 keys, gate 128) |
| 14, 111 to 114 | chat | Bandit: id 14 clip 25, ids 111 to 114 clips 4, 5, 6, 7; Thug: id 14 clip 5, ids 111 to 113 clips 4, 6, 7 (114 missing); 30 to 52 keys |
| 115 to 119 | fidgets | clips 2, 3, 5, 0, 0 |

#### Start and wake

Default 8 (Bandit 13, Thug 3): hidden at home; 11 of the Bandit's 13 hidden placements and 1 of the Thug's 3 have cue word 0 (not hittable until the emerge starts); the other 3 (Coliseum) have cue word 1. The Bandit's hide pick plays its clip 28 (20 keys) once; the Thug has no hide clip and shows clip 0. Emerge: id 10, clip 29, 57-tick gate: wake test tick w, request w + 1, agro w + 58, set-up w + 59, first choice w + 60. A Thug with emerge distance 0.0 never wakes by distance (only by hit or alarm). Default 5, 6 and 7: idle states and the two-stage wake of the common rules; the alert clip is 132 ticks and the search-end clip 156 ticks. The Thug's short notice cycle (17 ticks) and high awareness make it notice at once inside the near range (5 x F1): a standing hero inside 3.5 units is always noticed; the Bandit's near range is 0.7 standing.

#### Per-tick order (agro)

As Round 0 with these differences: the rush machine replaces the strafe machine (below); the coin search runs only for the Thug; the Bandit and Thug evaluate the melee defence for blocks, not dodges. Pre-step, agro machine (recovery, guard-state reset, missile defence, melee defence, lost-sight test, normal choice, counter choice, hide gate, outer switch, leash), class-19 handler, animation and motion passes.

#### Attack chooser

Slots tested 3, 2, 1, 0, once per tick while the hero is attackable, the outer sub-state is 1, 3, 4 or 11 (the rush machine is outer 1) and the recovery countdown is below 1. A slot passes when its cooldown is below 1, d is inside the band (strict), the facing dot is above 0.92388 (22.5 degrees), a roll int[1, 100] is below the chance (rush enemies have no backbiter factor), and a route exists or d is below 10.0. Probabilities per tick with all cooldowns 0, recovery 0 and the hero in front:

| d | Bandit | Thug |
|---|---|---|
| 0 < d < 6 | 89 % (slot 2 19 %, slot 1 40 %, slot 0 31 %) | 88 % (slot 3 74 %, slot 2 5 %, slot 1 5 %, slot 0 4 %); slot 1 is the empty feint |
| 6 <= d < 10 | 79 % (slot 2 19 %, slot 0 60 %) | 84 % (slot 3 74 %, slot 2 5 %, slot 0 5 %) |
| 10 <= d < 15 | 19 % (slot 2 only) | 79 % (slot 3 74 %, slot 2 5 %) |
| d >= 15 | 0 | 0 |

On a choice the slot's cooldown is drawn, the sequencer starts in state 0 (no slot has a charge), and the desired distance is redrawn. State 0: while d is above the slot's range the enemy walks toward the hero at the rush speed (12.6 u/s; the step is dropped within the sum of the radii: Bandit 2.0 plus the hero's 1.175, Thug 2.5 plus 1.175), then state 1. So a Bandit at d between 7 and 15 that picks slot 2 first closes to 7.0, winds up and strikes.

#### Movement (rush, outer sub-state 1)

Entered with inner 0 and the gait set 21, 21, 21, 23 and 55. With a route: inner 0 gait 0 (or gait 2 when d is above 12), id 21 at 12.6 u/s, steering at the hero, until d is below the desired distance (3.6 to 5.4), then inner 1; inner 1 holds, facing the hero (turn blend: Bandit 1.0, Thug 0.5) with id 22; d below 3 gives inner 2 (back off, id 23 at 11.2 u/s, to the point 8 units from the hero along the hero line, until d is above the desired distance), d above 6 gives inner 0. Without a route the enemy walks the waypoint list and the leash counter runs (common rules). Because the hero walks at 13.5 and the rush at 12.6 u/s, a walking hero keeps its distance; a running hero (19.3) escapes. The wait-in-line state (outer 12, id 2 walking) holds the rest of a pack (common rules).

Movement table (row 151 Bandit and row 154 Thug, the same clips; ground; radius 2.0 (Bandit) and 2.5 (Thug); ground turn 1.0 (Bandit) and 0.5 (Thug); rush band 3 to 6, desired 3.6 to 5.4). The Thug's size selector enlarges its body only; its travel is the Bandit's (SPEC-0038 Behavior 51 item 16).

| Mode | Source | Clip (keys) | u/tick | u/s | Turn | Exit |
| --- | --- | --- | --- | --- | --- | --- |
| Wander, walk home, wait in line, named-target follow | root | 2 or 11 (12) | 0.094 | 5.63 | steer | arrival 0.5, notice |
| Patrol | root | 4 (15), 0.008 to 0.161 per tick inside the cycle | 0.066 | 3.96 | steer | arrival 0.5 |
| Approach (inner 0), sequencer state 0 | root | 21 (8) | 0.210 | 12.60 | steer | d below desired; state 0: the slot range |
| Hold (inner 1) | none | 22 (16) | 0 | 0 | aim 1.0 (Bandit) or 0.5 (Thug) | d below 3 or above 6 |
| Back off (inner 2), below 3 | root, facing the hero | 23 (9) | 0.187 backward | 11.20 | steer, back gait | d above desired |
| Waking (10 or 82) | root | (15) | | forward 6.08 over 60 ticks | | gate |
| Confused (outer 11) | root | 69 (38, loop) | 0.130 | 7.81 | aim at a random direction 0.05 | status end |
| Hit (reaction kind 0) | none (0.02) | 57 (11) | 0 | 0 | snap to attacker | gate |
| Dodge | none: ids 53 to 55 missing (clip 0, no root) | | 0 | 0 | | |
| Corpse | none | 58 | 0 | 0 | kept | timer 600 |

Worked example: the chase from 30 units in SPEC-0038 Behavior 51 (the Bandit stops at 4.17 with a desired distance of 4.5, 123 ticks after it starts walking). The Bandit and Thug do not flinch: a hit stops them where they stand.
#### State table

Ticks are relative to the tick s in which the state requests its clip.

| State | Action id and clip | Keys | Speed | Gate and exits |
|---|---|---|---|---|
| Hidden (class 8, sub 0) | 9 -> clip 28 (Bandit) or clip 0 (Thug) | 20 / 40 | 1.0 | Wake rule |
| Emerge | 10 -> clip 29 | 15, one-shot | 1.0 | Gate s + 57 then agro |
| Search (class 2) | 16 -> clip 8, 18 -> clip 10, 17 -> clip 9 | 50 / 34 / 40 | 1.0 | 18: gate s + 132 then agro; 17: gate s + 156 then the default state |
| Agro set-up (outer 0) | agro idle pick 1 (absent: clip 0, the same idle clip) | 40 | 0.5 keys per tick | One tick, then outer 1 |
| Rush approach (inner 0) | 21 -> clip 32 | 8, loops | 1.0 | Until d is below the desired distance |
| Rush hold (inner 1) | 22 -> clip 30 | 16, loops | 1.0 | Faces the hero; leaves on d below 3 or above 6 |
| Back off (inner 2) | 23 -> clip 11 | 9, loops | 1.0 | Until d is above the desired distance |
| Slot 0 swing | B 34 -> clip 12 | 19, one-shot | 1.0 | Window s + 28 to s + 35; hit on the first touching tick: Bandit s + 28, Thug s + 29 for a level hero 4.0 to 6.0 ahead (s + 30 at 3.5, s + 28 at 6.5, s + 29 at 7.0, s + 28 from 7.5 to 12.5); gate s + 73; end path s + 74 |
| Bandit slot 1 swing | B 39 -> clip 14 | 20, one-shot | 1.25 | Hit at s + 23; the request tick's pass runs at the wind-up word 1.0 and the later passes at 1.25; gate s + 62 (miss path) or s + 71 (after a hit, state 3 runs at 1.0); end path one tick after the gate |
| Slot 2 swing | A 43 -> clip 38, then B 44 -> clip 37 | 4 / 12, one-shot | 1.0 | A gate s + 13; B requested the same tick (b = s + 13); window from b + 4; hit on the first touching tick, which for a level hero straight ahead is, for the Bandit, b + 4 up to 4.0 units, b + 5 at 4.5, b + 7 at 5.0, b + 6 at 5.5 and b + 7 from 6.0 to 9.0, and for the Thug b + 4 at 2.0 to 4.0 units, b + 5 at 4.5, b + 9 at 5.0 to 5.5, b + 8 at 6.0 to 10.5 and b + 7 at 11.0 to 11.5 (Medium); B gate b + 45; end path b + 46 = s + 59 |
| Blocked swing (hero blocked it) | D: 36 -> clip 33 or 13 / 41 -> clip 34 / 46 -> clip 35 | 13 / 10 / 9 | 1.0 | D plays to its end (gates 49 / 37 / 33), then state 5 and the end path |
| Block (outer 7) | 56 -> clip 19 | 10, one-shot | 1.0 | Held while the hero's swing query is true; block-hit 126 -> clip 36 once on a blocked hit; then the defence exit |
| Hit reaction kind 0 (outer 10) | 57 -> clip 20 | 11, one-shot | 1.0 | Gate s + 41, then outer 0 |
| Victory pose | 73 -> clip 27 | 33 | 1.0 | Gate s + 128, then the default-state step |
| Dead | 58 -> clip 21 | 20, one-shot | 1.0 | Plays once; 600-tick disappear countdown from progress 0.95 |

Defence exit: outer 0, base state 1, defence hold int[30, 120].

#### Thug slot 3: grab, steal and throw (Medium)

The Thug's slot 3 does not use the weapon test. When it is chosen and the Thug is within 6.0 (state 0 closes in first):

1. Tick s: state 1 (A 48 missing) requests B (id 49, clip 16) and enters state 2.
2. Tick s + 1 (the grab machine's choose step; it is in its choose state from the start of the sequencer): if the Thug's facing dot toward the hero is above 0.5 and the hero is within 6.0, the hero's stun word is clear and the straight path to the grab point is open, the hero is put in base state 53 (held; the hero cannot act, animation 143 each tick) and the grab point is the attach point named by the event bits of the current frame (none on clip 16's first keys, so point 0). A rumble of magnitude 0.7 for 1.0 s starts and repeats every 61 ticks while the hero is held. In the same tick the machine requests C (id 50, clip 18; call that tick c = s + 1). If the dot is 0.5 or lower, or d is 6.0 or more, or the stun word or path test fails, the grab is abandoned and the hero is not touched: on s + 2 the machine's recover state requests the recovery clip E (id 52, clip 39, 35 keys), the strike state waits for its gate on s + 138 and the end path runs on s + 139 (Medium).
3. From tick s + 2 the held hero stays where he was seized: the grab point is computed but the hero is not moved to it (SPEC-0029 Behavior 23). Each tick the machine checks that the hero's touched flag is clear and the straight path from the last good position is open; if not, the hold ends (hero back in base state 1 at the last good position).
4. The release bit is on key 25 of clip 18, first seen on tick c + 100 = s + 101. On that tick the hero's velocity is set to normalize(15 x facing + 8 x up) x 15 (throw speed) and the machine ends (state 4).
5. In the same tick s + 101 the strike test returns a hit (strike result 1). The sequencer goes to state 3 on the next tick, the hero's take-hit runs with damage 0 (with the throw velocity the hero is knocked into action state 48, SPEC-0038 Behavior 15; no health is lost to the throw itself), and the steal rolls: a roll int[0, 100] below 50 (50 of 101) takes int[20, 200] Neopoints (limited to the wallet) and spawns the coins at the hero's position. The steal needs no armour check here: armoured heroes are robbed too (the grab path forces a strike result of 1).
6. State 3 plays C to its end (C is already current: gate c + 124 = s + 125); state 5 skips the recovery clip E because the slot has the grab flag (SPEC-0038 Behavior 13, state 5), so the end path runs on s + 126: recovery int[0, 0], slot 3 cooldown int[120, 240] counted from that tick. The whole grab cycle takes about 2.1 seconds.
7. The hero is held for about 100 ticks (1.7 seconds). A hit on the Thug while it holds the hero frees the hero at once (take-hit hook step 4) and starts the 41-tick reaction.

The slot has damage 0 and no weapon-volume path: guarding, blocking angle and the unblockable flag play no part, and the hero's shield does nothing against it.

#### Hit and hurt windows

| Attack | Window (ticks after request) | Hits | Damage (before armour) |
|---|---|---|---|
| Slot 0 (both) | s + 28 to s + 35 | one, on the first touching tick (Bandit s + 28; Thug s + 29 from its hold distance) | 10, blockable |
| Bandit slot 1 (R 1.25) | s + 23 to s + 31 | one, at s + 23 | 15, blockable |
| Slot 2 (both), B requested at b | b + 4 to b + 15 | one, on the first touching tick (b + 4 to b + 9 by distance, state table) | Bandit 20, Thug 25, blockable |
| Thug slot 3 | no window; hero seized at s + 1, thrown at s + 101 | steal roll at s + 101 | 0, steal 50 of 101, 20 to 200 Neopoints |
| Thug slot 1 | none (no clip) | none | 0 |

Hittable: always except while dead or script-held, and while hidden with cue word 0. The Bandit and Thug are hittable while blocking (the guard rule below) and during hit reactions. A hurt, knocked-down or dead hero is not attackable; a running swing still lands.

#### Weapon reach

Conventions are those of "Weapon reach tables" in the shared field behaviour. Rows 151 (Bandit) and 154 (Thug), Medium.

| Kind, slot (phase, clip) | Weapon | Window ticks (level contact) | Centre forward / up / side | Tor ahead, at the tick | Tor ahead, from the start | Tor feet vs origin |
|---|---|---|---|---|---|---|
| Bandit 0 (B, clip 12) | w1 (4.0 x 1.5) | s+28 to s+35 (s+28 to s+33) | 2.1 to 3.7 / 0.5 to 2.2 / -1.8 to 2.5 | 1.8 to 7.2 | 3.5 to 9.0 | -6.0 to 5.5 |
| Bandit 1 (B, clip 14) | w1 (4.0 x 1.5) | s+23 to s+32 (s+23 to s+32) | -1.4 to 2.9 / 0.7 to 2.0 / -2.9 to 1.4 | -5.2 to 7.4 | -3.5 to 8.4 | -5.5 to 5.0 |
| Bandit 2 (B, clip 37) | w1 (4.0 x 1.5) | b+4 to b+15 (b+4 to b+15) | 2.2 to 3.8 / 0.3 to 5.1 / -0.8 to -0.4 | -0.2 to 8.2 | 1.0 to 9.6 | -6.5 to 9.0 |
| Thug 0 (B, clip 12) | w1 (5.0 x 1.5) | s+28 to s+35 (s+28 to s+33) | 3.0 to 5.1 / 0.7 to 3.1 / -2.5 to 3.5 | 3.6 to 9.2 | 5.4 to 11.0 | -6.5 to 7.0 |
| Thug 2 (B, clip 37) | w1 (5.0 x 1.5) | b+4 to b+15 (b+4 to b+15) | 3.1 to 5.2 / 0.5 to 7.1 / -1.2 to -0.5 | 0.4 to 10.6 | 1.6 to 12.0 | -7.5 to 12.0 |

The Thug's slot 0 swing never touches a level hero closer than 3.6 ahead (the weapon is 3.0 to 5.1 in front of it). One Thug placement in a different map resource has its slot 2 on a different clip (index 15) with the window b+24 to b+35 and reach 0.2 to 10.8 ahead. Thug slot 3 (grab), slot 1 (no clip) and the Bandit's unused slots have no weapon test.

#### Defence and counters

- Block (outer 7), melee. Decided once per hero swing as in Round 0 (latch; hold below 1; not in outer 10 and not mid-sequence except sequencer states 0, 1 and 5; hero within 8.0 and inside 60 degrees). Two rolls int[1, 100]: the first at most the block chance (Bandit 50 quick, 40 normal, 30 heavy; Thug 50, 30, 30) gives outer 7; the dodge chances are 0. In outer 7 the enemy faces the hero, plays 56 and is in the guard state for as long as the hero's swing query is true; a blocked hit plays 126 once; then the defence exit sets the defence hold to int[30, 120], during which no new defence decision is made. Tor's power step (sub-state 10) is never blocked or dodged.
- Missile block (outer 8): a hero shot whose 10-tick path (hold value 10) meets the body: dodge roll below 0 never, block roll below 50 gives outer 8, which plays 56 while a shot is live; Roberta's quick shots bounce off a guarding enemy (SPEC-0036).
- What a guard does to the hero's swing: for a guarding Bandit or Thug and a hero swing that is not a power swing, let c = dot(enemy facing, hero facing). If c is below the blocking angle 0.92388 the hit is blocked: no damage, no reaction, a block effect, and the block-hit clip. If c is 0.92388 or more (the hero swings from behind within about 22 degrees of the enemy's own heading) the swing has no effect at all: no damage, no reaction, no cue. This is one of three states in which a swing does nothing (SPEC-0036 Hit detection step 4).
- Hit reaction: every unblocked hit sets outer 10, kind 0 (id 57, 41-tick gate); the first hit on a non-agro enemy is doubled and wakes it. No knockback.
- Counters: Bandit slot 1 only (trigger 6, chance 25): eligible when its cooldown is below 1 and d is below 6.0 with the hero within 22.5 degrees; one roll int[0, 100] below 25 per tick of the reaction (25 of 101): almost certain within 41 ticks. Thug slot 3 (trigger 6, chance 50): eligible when its cooldown is below 1 and d is below 15 within 22.5 degrees, 50 of 101 per tick: the Thug answers a hit with a grab almost every time slot 3 is off cooldown. The Bandit's slots 0 and 2 and the Thug's slots 0, 1 and 2 do not counter. A counter does not reset the shared recovery.

#### Hero actions against this enemy

| Hero action | What the Bandit or Thug does |
|---|---|
| Stands still outside the notice range | Idles, guards, wanders, patrols or hides |
| Tor quick swing in front of it (within 8.0, 60 degrees) | Blocks with 50 of 100 (the first roll must be at most 50), even if the swing would hit |
| Tor normal swing | Bandit blocks 40 of 100, Thug 30 of 100 |
| Tor heavy swing | Bandit 30 of 100, Thug 30 of 100 |
| Tor power step | Never blocked; damage by the power table |
| Swing from behind the guard (hero facing nearly the same way) | No effect at all when the enemy is guarding; a normal hit when it is not |
| Roberta's shots | Bandit and Thug block 49 of 100 (hit path in 10 ticks); a blocked shot bounces |
| A landed hit | Normal damage; doubled when not in agro; reaction kind 0; Thug counter-grab 50 of 101 per tick; Bandit slot 1 counter 25 of 101 |
| Hero guard (shield) | Blocks slots 0 to 2 when the hero faces the attacker (dot at most -0.5); does nothing against the Thug's grab |
| Hero hurt, knocked down or dead | No new choice; cooldowns stop; a running swing continues |
| Hero held by the Thug | Cannot act for about 100 ticks; a hit on the Thug frees it |
| Hero swap | Next tick all tests use the new hero |

#### Edge cases for implementers

- The Thug's recovery countdown is 0, so after slot 0 or 2 it can choose again as soon as the slot cooldowns allow; slot 3's cooldown (120 to 240) and the long grab keep the cadence slow.
- The Thug's slot 1 is a feint: when chosen (about 5 percent of choices) the Thug walks up to 5.0 units from the hero, finds no wind-up and no strike clip, and ends the swing the same tick (state 5, end path): damage 0, no animation, recovery 0, cooldown int[60, 180]. It costs the tick of the choice and some walking.
- The Bandit's slot 1 plays its strike at 1.25 times speed; once the hit lands the rest of the clip drops to 1.0, so a swing that connects lasts 71 ticks and one that misses 62.
- A blocked swing (hero shield up and facing) adds the D clip (49 / 37 / 33 ticks) before the end path.
- The first state-2 tick of slot 3 grabs a hero who is within 6.0 even if the hero is mid-swing or mid-step; the hero's sub-state does not matter, only the stun word, facing and path.
- Bandits and Thugs that are waiting in line (outer 12) walk to a stand-off point 20 units around the hero and do not swing; a group of four Bandits at starting gear produces one attacker at a time.
- The two defence rolls are made at the start of the hero's swing, before its window opens; a Bandit that blocked cannot block again for 30 to 120 ticks.
- Group alarms: the Bandit Caves, Farm Road and Meridell Plains ambush groups mix Bandits, Thieves and Thugs; the first to see the hero wakes the others within 45 ticks; hidden members play their emerge clip (57 ticks) and, with cue word 0, start the ambush camera cue.

#### What a port most likely gets wrong

1. Giving the Bandit and Thug a strafe machine and dodges. They rush, hold 3.6 to 5.4 units from the hero and block; there is no dodge clip.
2. Missing the block rules: a block is one decision per hero swing, takes the whole swing (hit from the side is blocked, from behind within 22 degrees does nothing, power steps always land).
3. Treating the Thug as a heavy puncher: its main attack deals no damage, steals 20 to 200 Neopoints at 50 of 101 (even through armour), and holds the hero for about 100 ticks.
4. Playing a clip for the Thug's slot 1; every id of that slot is missing, so the choice is an instant no-op that still consumes the cooldown.
5. Using one recovery: the Bandit rests 60 to 180 ticks after each swing, the Thug none.
6. Missing the Bandit's slot 2 wind-up (13 ticks) and the 1.25 times speed of slot 1.
7. Giving every ambusher a hittable hidden state; only cue-word-1 records are hittable.
8. Waking from the idle states with a single roll; the two-stage wake and the 132-tick alert apply.
9. Letting every Bandit in a pack swing at once (the attack manager caps it).

#### Timeline test vector

Bandit at default 7 (guard, +0xC4 = 0), awareness 90 with timer 60, hero standing (F1 0.7) 20 units in front, hero never moves, Bandit facing the hero. Notice at 20 units: near 0.7, far 21; the chance is 70 + (5 - 70) x (20 - 1) / 29 = 27.4 percent inside the cone, so one check passes with probability 0.9 x 0.274 = 24.7 percent per 62-tick cycle.

| Tick | Event |
|---|---|
| 0 to 62 x k | Notice checks on ticks 0, 62, 124, ... (24.7 % each; expected first pass at the fourth check, tick about 186) |
| n | Pass: state 2, id 16 starts; a second check on the same tick (same chance) and then one every 62 ticks inside the search window of 60 to 239 ticks (at most 4 more checks) |
| n or n + 62 ... | Second pass: id 18 requested (gate 132); the alert plays to its end even if later checks fail |
| n' + 132 | Agro set; n' + 133 set-up; n' + 134 first choice |
| n' + 134 on | Rush: d = 20, no slot band reaches it: id 21 at 12.6 u/s; at d < 15 slot 2 becomes eligible (19 % a tick) |
| about n' + 134 + 24 | d about 15; at d about 10 slot 0 enters; choices in the 7 to 10 range come with slot 2 19 %, slot 0 60 % |
| choice at d <= 7 | Slot 0: B at s, hit s + 28 (10 damage, blockable), end path s + 74, recovery int[60, 180]: next swing 135 to 255 ticks after the first |
| 600-tick horizon | A still hero is struck about 2 to 4 times by one Bandit; each cycle is 135 to 255 ticks |

A Thug in the same set-up (default 7, awareness 100, timer 15, near 5, far 40; standing hero at 20 units: chance 90 - 80 x 15 / 35 = 55.7 percent in the cone, 55.7 percent per 17-tick cycle) notices within the first 2 or 3 checks (tick 0 to 34), plays the 132-tick alert, then rushes to about 4.5 units, chooses slot 3 at 74 percent a tick (arc permitting), walks to 6.0, grabs on s + 1, throws on s + 101, steals on the same tick with 50 of 101, and ends on s + 126; the next grab is eligible when slot 3's cooldown (120 to 240 counted from the end path) has run out, so after the first grab one or two more fit in 600 ticks, with swings of slots 2, 1 and 0 in between (53 percent a tick to start one at d below 6 while slot 3 is cooling: slot 2 19 %, slot 1 19 %, slot 0 15 %).

#### Open questions

1. Resolved (enemy sweep, Medium): an abandoned grab plays the recovery clip E (id 52) from s + 2; the strike state ends at its gate (s + 138) and the end path runs on s + 139.
2. Whether a hero in a shield or guard animation can be grabbed. Default if unresolved (rewrite choice): yes; only the stun word, facing and path matter.
3. Resolved: the hero's reaction when thrown. The throw velocity is set first and the 0-damage take-hit then puts the hero in action state 48 (knocked), with no health change (SPEC-0038 Behavior 15).
4. Weapon contact geometry: Resolved by the general rule of the shared field behaviour. The distances are in the Weapon reach table. Stand-in only for a port without attachment frames (rewrite choice): count a window tick as a hit when a level hero is inside the "at the tick" range of that slot's row (for example 3.6 to 9.4 ahead for the Thug's slot 0).
5. Resolved (Medium-High; proactive (open close-out)): a hidden Bandit or Thug in state 8 is not excluded by the peer test (it excludes only states 1, 2, 3, 19 and 21; SPEC-0038 Behavior 10), so a hidden brigand within 40 units of a wandering one can be pulled out to chat.
6. Resolved (High; proactive (open close-out)): a guarding Bandit hit from behind (c at least its Blocking Angle) is not landed, so Tor pays no magic (SPEC-0036, block rule).

### Drackonack and Giant Drackonack

One guide for both. The Drackonack (monster type 66 variant 0, row 144) and the Giant Drackonack (type 67 variant 0, row 145) are the ridge and mine beasts: a 31-bone quadruped skeleton and a 40-clip set, two bite-charge attacks each, no projectiles, no steal, and no block. The Drackonack strafes (approach 4) and sprints at 26 u/s; the Giant rushes (approach 1), is nearly impossible to hit with a normal swing because it dodges 84 to 89 percent of them, and bites for 25 or 50. Both attacks are charges of kind 2 (a straight run with no steering, then a short bite window), which is the part a port most often gets wrong.

#### Identity and tunables

| Item | Drackonack (row 144) | Giant Drackonack (row 145) |
|---|---|---|
| Hit points | 55 | 300 |
| Body: radius, height, weight | 1.5, 2.0, 5.5 | 3.5, 4.5, 5.5 |
| Class, approach | 17, approach 4 (strafe) | 17, approach 1 (rush) |
| Difficulty, target type | 8, 1 | 12, 1 |
| Hurt shape | One capsule on the root bone: radius 1.6, length 2.18, base (0, -0.52, 0.57) | Root bone: radius 1.63, length 1.96, base (0, -0.58, 0.61) (model units; row scale is not applied, Medium) |
| Weapon volumes | Weapon 3 only: radius 5.0, full height 3.0, at attach point 2 (bone 9) | Weapons 1 to 5 all radius 3.0, height 3.0 (the clips use weapon 3, attach point 2) |
| Notice | Cone 0.92388, awareness 75, timer 30 (cycle 32 ticks), chance 65 at range 3 falling to 10 at range 50 | same |
| Turn value | 1.0 | 0.5 |
| Movement band | Strafe desired 4 to 10 (desired = 4 + 6 f, 5.2 to 8.8), backbiter 0.5 | Rush melee distance 5 to 15 (desired = 5 + 10 f, 7 to 13) |
| Stand-off radius | 15 | 15 |
| Recovery after an attack | int[15, 30] | int[15, 30] |
| Missile defence | Hold 30, dodge 75, block 0 | same |
| Melee defence | Block 0; dodge 50 / 50 / 50 (quick / normal / heavy); defence timer 0 to 0 | Block 0; dodge 85 / 85 / 90; defence timer 0 to 0 |
| Other | Blocking angle 0, no damage reduction, knockback 0 on both slots, no map search, no socialize | same; its row also carries a tough-hide word of 1 that no class-17 routine reads (Low) |
| Death | Disappear 300 | Disappear 420 |
| Placements in play | 85: default 5 on 36, 8 on 30, 6 on 16, 7 on 3 | 40: default 5 on 22, 6 on 8, 7 on 6, 8 on 4 |
| Alarm groups | Abandoned Mines groups 2, 4 and 5 (14 hidden Drackonacks in two region copies, emerge 5 or 15; 9 more hidden ones there have no group), Bandit Caves group 1, Drackon Ridge group 2 | Abandoned Mines group 3 (idle and patrolling), Drackon Ridge group 2 |

Attack slots (slots 2 and 3 are unused on both rows; test order 1, 0):

| Slot | Band (strict) | Chance | Arc | Range | Orient | Delay | Charge | Damage | Speeds | Counter |
|---|---|---|---|---|---|---|---|---|---|---|
| Drackonack 0 | 0 to 10 | 25 | 0.966 | 5.0 | yes | int[240, 420] | kind 2, float[5, 15) | 7 | 1 / 1 / 1 | trigger 2, chance 50 |
| Drackonack 1 | 0 to 15 | 50 | 0.966 | 5.0 | yes | int[240, 420] | kind 2, float[5, 15) | 7 | 0.8 / 0.8 / 0.8 | trigger 6, chance 30 |
| Giant 0 | 0 to 10 | 25 | 0.966 | 5.0 | no | int[30, 120] | kind 2, float[5, 15) | 25 | 1 / 1 / 1 | trigger 2, chance 60 |
| Giant 1 | 0 to 25 | 50 | 0.966 | 5.0 | yes | int[60, 120] | kind 2, float[5, 15) | 50 | 0.8 / 0.8 / 0.8 | trigger 6, chance 60 |

Slot 1 of each also has a damage-special flag 0x100 that nothing in class 17 reads, and a blocked-charge value that the kind-2 charge never uses (kind 2 never enters the blocked-charge state). The backbiter 0.5 of the Drackonack halves the roll threshold of both slots (to int(0.5 x chance): 12 and 25) whenever dot(enemy facing, hero facing) is 0 or less, that is, whenever the hero is not facing away from the Drackonack; the Giant has no strafe and no backbiter.

| Slot | Phase ids and clips |
|---|---|
| 0 | A 33 none; B 34 = the run clip (Drackonack clip 35: 10 keys, loops, 30.4 u/s; Giant clip 34: 10 keys, loops, 13.8 u/s); C 35 = clip 19 (8 keys, one-shot, weapon 3 on key 1, gate 29); D 36 = clip 18 (14 keys, backs 22 units; played only after a blocked charge, which kind 2 does not produce); E 37 none |
| 1 | A 38 none; B 39 = the run clip at speed 0.8 (24.3 / 11.0 u/s); C 40 = clip 19; D 41 = clip 18; E 42 none |

Animation ids (the Drackonack column first, then the Giant where it differs; one table serves every placement in play):

| Id | Use | Clip |
|---|---|---|
| 0 | idle | clip 15 (18 keys, loops) |
| 2 / 4 | walk (states 5, 7, 8) / patrol walk | Drackonack clip 26 (8 keys, loops, 7.5 u/s) / clip 30 (10 keys, 3.28 u/s); Giant clip 30 for both (3.28 u/s) |
| 3, 6, 7, 8 | rest fidget, patrol fidget, guard stand, guard look | clip 11 (43 keys), clip 20 (20 keys) or Giant clip 22 (30 keys), clip 24 (50 keys, one-shot) or Giant clip 33 (22 keys), clip 29 (18 keys) or Giant clip 11 |
| 9, 10 | hide pick, emerge | clip 2 (20 keys, loops), clip 1 (22 keys, one-shot, gate 85, 0.6 units forward) |
| 16, 17, 18 | search idle, search end, alert | clips 23 (40 keys), 13 (28 keys, gate 108), 25 (25 keys, gate 96) |
| 28 | Drackonack strafe gait 0 | clip 21 (12 keys, loops, 26.5 u/s) |
| 29, 30 | Drackonack circles | both clip 27 (8 keys, loops, 7.5 u/s sideways); id 29 plays it mirrored (root x negated), so id 29 travels along +x and id 30 along -x, the same signs as the Thief's two clips |
| 31 | Drackonack strafe hold | clip 0 (32 keys, speed byte 1: 0.5 keys per tick, loops) |
| 32 | Drackonack back step | clip 32 (8 keys, loops, 7.5 u/s backward) |
| 21, 22, 23 | Giant rush forward, hold, back | clips 34 (10 keys, loops, 13.8 u/s), 33 (22 keys, loops), 37 (10 keys, loops, 13.8 u/s backward) |
| 53, 54 | missile dodge | both clip 10 (9 keys, one-shot, 8.4 units to one side, gate 33) |
| 55 | melee dodge | Drackonack clip 8 (23 keys, one-shot, 19.8 units backward over its length, gate 89); Giant clip 37 (10 keys, loops, 9.2 units backward, gate 39) |
| 57 | hit reaction kind 0 | Drackonack clip 31 (14 keys, 8.3 units backward, gate 53); Giant clip 36 (9 keys, 3.8 units backward, gate 33) |
| 58 | death | clip 9 (22 keys, gate 85) |
| 73 | victory pose | clip 22 (30 keys, gate 116) |
| 115 to 117 | fidgets | Drackonack clips 11, 20, 24 (118 and 119 missing); the Giant has none |

#### Start and wake

Default 8 (Drackonack 30, Giant 4): hidden; the cue word is 1 on every one of them, so a hidden Drackonack or Giant can be hit and targeted; a hit wakes it with doubled damage. Emerge distances in play: Drackonack 5 (13 placements, 3.5 for a standing hero, 8.75 for a running one), 15 (9), 10 (2), 40 (4, Coliseum), 60 and 100 (1 each); Giant 10 (2) and 40 (2). The emerge clip is id 10 (22 keys, 85-tick gate): wake test tick w, request w + 1, agro w + 86, set-up w + 87, first choice w + 88. Groups: when one wakes, its group mates in states 8 wake 15 to 45 ticks later and play the same 85-tick clip; there is no ambush cue (cue word 1). Default 5, 6 and 7: the idle states and two-stage wake of the common rules: the alert clip is 96 ticks, the search idle 156 and the search end 108; at 20 units a standing hero is noticed with probability 0.75 x (65 - 55 x 17 / 47) = 34 percent per 32-tick cycle in the cone. Return to the default after losing the hero as the common rules.

#### Per-tick order (agro)

As Round 0 (the Drackonack in the strafe machine, the Giant in the rush machine). The coin search and the distance-hide gate do nothing for these rows.

#### Attack chooser

Slots tested 1 then 0, once per tick while the hero is attackable, the outer sub-state is 1, 3, 4 or 11 and the recovery countdown is below 1. A slot passes when its cooldown is below 1, d is inside the band (strict), the facing dot is above 0.966 (15 degrees), a roll int[1, 100] is below the threshold, and a route to the hero exists (a charge slot needs a route even inside 10 units). The threshold is the chance, except for the Drackonack when the hero is not facing away (above): 25 for slot 1 and 12 for slot 0.

| d | Drackonack, hero facing it or sideways | Drackonack, hero facing away | Giant |
|---|---|---|---|
| 0 < d < 10 | 32 % (slot 1 24 %, slot 0 11 %) | 61 % (slot 1 49 %, slot 0 24 %) | 61 % (slot 1 49 %, slot 0 24 %) |
| 10 <= d < 15 | 24 % | 49 % | 49 % (slot 1 only) |
| 15 <= d < 25 | 0 | 0 | 49 % |
| d >= 25 | 0 | 0 | 0 |

On a choice the slot's cooldown is drawn (long: Drackonack 240 to 420 ticks per slot), the sequencer starts in state 6 (the charge), and all distances are redrawn (the charge distance for either slot is float[5, 15)). A counter choice starts the same state 6 without redrawing, so it uses the value of the last normal choice. The Drackonack's slot cooldowns are so long that over 600 ticks each slot is used about twice and the Drackonack spends most of the time strafing.

#### Charge (sequencer states 6, 7, 8, 12), the only way these enemies hurt

1. Tick c (the choice tick, sequencer state 6). The enemy aims at the hero once, with blend weight equal to its turn value (Drackonack 1.0 = faces the hero exactly, Giant 0.5 = half way from its heading). A 33 or 38 is missing, so the run clip B is requested in the same tick at the slot's strike speed (slot 0: 1.0, slot 1: 0.8) and state 7 begins on the next tick. The start point is the enemy's position.
2. State 7 (run), from c + 1: the root travel of the run clip moves the enemy along its fixed heading, no steering and no strike test. Per-tick travel: Drackonack slot 0 0.5065, slot 1 0.405; Giant slot 0 0.230, slot 1 0.184. Each tick: if the hero is within the melee range 5.0 (distance between the two positions), state 8; otherwise if the distance travelled from the start point has reached the drawn charge distance (5 to 15), state 12. Also state 12 when more than 100 ticks have passed and the speed is at most 0.001.
3. State 8 (bite): its first tick c0 is the tick after state 7 saw the hero within 5.0, and it requests C (id 35 or 40, clip 19) at rate 1.0: weapon 3 is live only on key 1 of the clip, which is seen on ticks c0 + 4 to c0 + 7, so one hit at c0 + 4 for a level hero 2 to 5 units ahead (the small Drackonack keeps c0 + 4 out to 7.0 units, c0 + 5 at 7.5 and c0 + 6 at 8.0; Giant: c0 + 4 at 2.5 to 4.5 units, c0 + 6 at 5.0 to 6.0, c0 + 7 at 6.5 to 7.0, and at 1.5 units c0 + 6 with no touch at 2.0; Medium) (damage 7, 25 or 50, blockable). C passes its gate at c0 + 29, state 12 is set, and the end path runs on c0 + 30 (E is missing): recovery int[15, 30], slot cooldown redrawn.
4. Because the run does not steer, a hero who steps aside is missed. Because it does not test strikes, the run never damages the hero by contact; only the bite does. A hero who steps aside is run past (the chase-stop rule applies only to sequencer states 0 to 3). A hero who stays is never run into: the run ends when he is within 5.0, more than the sum of the radii (2.675 and 4.675). A kind-2 charger stays solid in states 7 and 8 (SPEC-0038 Behavior 13, state 7), so if the bite clip's small forward travel brings the bodies together, the hero's own body push moves the hero, not the enemy (SPEC-0033 Behavior 53).
5. If the drawn charge distance is shorter than the hero's distance minus 5.0 the charge stops short with no bite; for a standing hero at distance d between 10 and 20 the probability that the bite is reached is (20 - d) / 10, at 10 or less it is 1, at 20 or more 0. A counter charge may use the stale distance of the last normal choice.

#### Movement

Drackonack (strafe, outer 4): as the Thief's strafe rules with the band 4 to 10: inner 10 holds (face the hero at turn 1.0, id 31 clip 0) for int[60, 180] ticks and then rolls int[1, 3] (stay, circle inner 8 travelling to its right, circle inner 9 travelling to its left, with the lock rule; SPEC-0038 Behavior 51 item 7); d above 10 gives the approach (inner 7, id 28 at 26.5 u/s, which no hero can outrun); d below 4 with no lock gives the back off (inner 11, id 32 at 7.5 u/s, to the point 10 units from the hero, until d exceeds the desired distance 5.2 to 8.8); circling is 7.5 u/s. Giant (rush, outer 1): inner 0 walks at 13.8 u/s (id 21) toward the hero until d is below the desired distance (7 to 13), inner 1 holds facing the hero at turn 0.5 (id 22), d below 5 gives inner 2 (back off, id 23 at 13.8 u/s, to the point 17 units from the hero) and d above 15 gives inner 0; with no route the waypoint walk and leash apply. Sequencer state 0 is never used (both slots charge).

Movement table, Drackonack (row 144; ground; radius 1.5; ground turn 1.0; strafe band 4 to 10, desired 5.2 to 8.8):

| Mode | Source | Clip (keys) | u/tick | u/s | Turn | Exit |
| --- | --- | --- | --- | --- | --- | --- |
| Wander, walk home, wait in line, named-target follow | root | 2 or 11 (8) | 0.125 | 7.50 | steer | arrival 0.5, notice |
| Patrol | root | 4 (10) | 0.0547 | 3.28 | steer | arrival 0.5 |
| Approach (inner 7), above 10 | root | 28 (12) | 0.441 | 26.49 | steer | d below desired |
| Hold (inner 10) | none | 31 (clip 0) | 0 | 0 | aim 1.0 | int[60, 180] ticks, then the roll |
| Circle inner 9 / inner 8 | root, sideways | 29 / 30 (8) | 0.125 | 7.50 | snap to the hero | hold count, blocked, band |
| Back off (inner 11), below 4 | root, facing the hero | 32 (8) | 0.125 backward | 7.50 | steer, back gait | d above desired or blocked |
| Charge run (state 7) | root | 34 or 39 (10) at 1.0 / 0.8 | 0.5065 / 0.405 (0.31 to 0.72 inside the cycle at 1.0) | 30.38 / 24.30 | none (straight) | within 5.0, the drawn 5 to 15, or blocked |
| Bite (state 8) | root | 35 or 40 (8) | | forward 0.78 over 32 ticks | none | gate |
| Hit (reaction kind 0) | root | 57 (14) | | back 8.38 over 56 ticks | snap to attacker | gate |
| Heavy hit 62 | root | 62 (35) | | back 22.80 over 140 ticks | | gate |
| Dodge 53 / 54, 55 | root | (9), (23) | | side 7.89 over 36; back 19.79 over 92 | | gate |
| Corpse | root | 58 (22) | | back 6.01 over 88 ticks, nothing held | kept | timer 300 |

Movement table, Giant Drackonack (row 145; radius 3.5; ground turn 0.5; rush band 5 to 15, desired 7 to 13; size selector 1, body about 1.9 times larger, travel not scaled):

| Mode | Source | Clip (keys) | u/tick | u/s | Turn | Exit |
| --- | --- | --- | --- | --- | --- | --- |
| Wander, patrol, walk home, wait in line | root | 2 or 4 (10) | 0.0547 | 3.28 | steer | arrival 0.5, notice |
| Approach (inner 0) | root | 21 (10) | 0.230 | 13.79 | steer | d below desired |
| Hold (inner 1) | none | 22 (22) | 0 | 0 | aim 0.5 | d below 5 or above 15 |
| Back off (inner 2), below 5 | root, facing the hero | 23 (10) | 0.230 backward | 13.79 | steer, back gait | d above desired |
| Charge run (state 7) | root | 34 or 39 = the walk clip 21 (10) at 1.0 / 0.8 | 0.230 / 0.184 | 13.79 / 11.03 | none (straight) | within 5.0, the drawn 5 to 15, or blocked |
| Bite (state 8) | root | 35 or 40 (8) | | forward 0.78 over 32 ticks | none | gate |
| Hit (reaction kind 0) | root | 57 (9) | | back 3.71 over 36 ticks | snap to attacker | gate |
| Dodge 53 / 54, 55 | root | (9), 55 = the back-off clip | | side 7.89 over 36; back 13.79 per second while it plays | | gate |
| Corpse | root | 58 (22) | | back 6.01 over 88 ticks, nothing held | kept | timer 420 |

The Giant's id 11 (named-target follow) is missing and plays clip 0 (no root), so a Giant in state 9 cannot follow. Port mistakes: scaling the Giant's speed by its size (its charge is less than half the small kind's, 13.79 against 30.38); letting the charge steer; using one speed for the Drackonack's wander (7.50) and patrol (3.28).
#### State table

| State | Action id and clip | Keys | Speed | Gate and exits |
|---|---|---|---|---|
| Hidden (class 8, sub 0) | 9 -> clip 2 | 20, loops | 1.0 | Wake rule |
| Emerge | 10 -> clip 1 | 22, one-shot | 1.0 | Gate s + 85 then agro |
| Search (class 2) | 16 -> clip 23, 18 -> clip 25, 17 -> clip 13 | 40 / 25 / 28 | 1.0 | 18: gate s + 96 then agro; 17: gate s + 108 then the default state |
| Agro set-up (outer 0) | agro idle pick 1 (absent: clip 0, the 32-key strafe hold loop; the out-of-agro idle 0 is clip 15) | 32, loops | 1.0 | One tick, then outer 4 (Drackonack) or outer 1 (Giant) |
| Strafe hold / approach / circle / back off (Drackonack) | 31 -> clip 0; 28 -> clip 21; 29, 30 -> clip 27; 32 -> clip 32 | 32 / 12 / 8 / 8 | 0.5 keys per tick for clip 0, otherwise 0.25 | As the strafe rules |
| Rush approach / hold / back off (Giant) | 21 -> clip 34; 22 -> clip 33; 23 -> clip 37 | 10 / 22 / 10 | 1.0 | As the rush rules |
| Charge run (state 7) | 34 or 39 -> run clip | 10, loops | slot 0 1.0, slot 1 0.8 | Until the hero is within 5.0 or the charge distance is travelled |
| Bite (state 8) | 35 or 40 -> clip 19 | 8, one-shot | 1.0 | Hit at c0 + 4; gate c0 + 29; end path c0 + 30 |
| Melee dodge (outer 5) | 55 -> clip 8 (Drackonack) or clip 37 (Giant) | 23 / 10 | 1.0 | Gate s + 89 (Drackonack) or s + 39 (Giant); then the defence exit |
| Missile dodge (outer 6) | 53 or 54 -> clip 10 | 9 | 1.0 | Gate s + 33; then the defence exit |
| Hit reaction kind 0 (outer 10) | 57 -> clip 31 (Drackonack) or clip 36 (Giant) | 14 / 9 | 1.0 | Gate s + 53 / s + 33, then outer 0 |
| Victory pose | 73 -> clip 22 | 30 | 1.0 | Gate s + 116 |
| Dead | 58 -> clip 9 | 22, one-shot | 1.0 | Plays once; disappear countdown (300 or 420) from progress 0.95 |

Defence exit: outer 0, base state 1, defence hold int[0, 0] = 0.

#### Hit and hurt windows

| Attack | Window | Hits | Damage per hit before armour |
|---|---|---|---|
| Drackonack slot 0 or 1 bite | c0 + 4 to c0 + 7 | one, at c0 + 4 | 7, blockable |
| Giant slot 0 bite | c0 + 4 to c0 + 7 | one | 25, blockable |
| Giant slot 1 bite | c0 + 4 to c0 + 7 | one | 50, blockable |

The run is not a hit. The enemy can be struck in every state except a melee dodge or missile dodge (outer 5, 6), while dead, while script-held, and (never for these rows) while hidden with cue word 0. A hit while charging cancels the charge (hit reaction); a hit in the bite window cancels the bite. A hurt, knocked-down or dead hero is not attackable (no new choice, cooldowns stop), but a charge already running continues and its bite still lands.

#### Weapon reach

Conventions are those of "Weapon reach tables" in the shared field behaviour. Rows 144 and 145, Medium. Both slots use the same bite clip, so the two slots give the same figures; c0 is the tick the bite clip is requested.

| Kind, slot (phase, clip) | Weapon | Window ticks (level contact) | Centre forward / up / side | Tor ahead, at the tick | Tor ahead, from the start | Tor feet vs origin |
|---|---|---|---|---|---|---|
| Drackonack 0 and 1 (C, clip 19) | w3 (5.0 x 3.0) | c0+4 to c0+7 (c0+4 to c0+7) | 1.1 to 1.9 / 1.6 to 2.3 / -0.1 | -4.2 to 7.0 | -3.5 to 7.8 | -6.0 to 6.0 |
| Giant Drackonack 0 and 1 (C, clip 19) | w3 (3.0 x 3.0) | c0+4 to c0+7 (c0+4 to c0+7) | 0.7 to 2.7 / 3.9 to 4.4 / -0.1 to 0.1 | -1.8 to 6.2 | -1.1 to 7.1 | -2.0 to 7.0 |

The figures are measured from where the bite clip starts; the run before it is not counted. The Giant's volume is smaller but sits about 4 above its origin and its skeleton is scaled up, so it reaches 6.2 ahead against the small one's 7.0, and its feet band is narrow (2 below to 7 above its origin).

#### Defence and counters

- Melee dodge (outer 5): one decision per hero swing, on the first tick the swing query is true, only when the defence hold is below 1, not in outer 10, not in mid-sequence except sequencer states 0, 1 and 5, hero within 8.0 and inside 60 degrees. The block roll never passes (block 0). The dodge passes when the second roll int[1, 100] is below the chance: Drackonack below 50 for every swing class (49 of 100); Giant below 85 for quick and normal swings (84 of 100) and below 90 for heavy swings (89 of 100). The dodge sets the hold to 30 and plays id 55; the Drackonack's dodge is a 89-tick hop of 19.8 units backward (mostly early), the Giant's a 39-tick slide of 9.2 units; both are unhittable for the whole dodge. Tor's power step is never dodged; Roberta's swat is a quick swing.
- Missile dodge (outer 6): a hero shot whose 30-tick path meets the body: roll below 75 (74 of 100) gives outer 6 and hold 30; id 53 or 54 (both clip 10) for 33 ticks.
- No block, no guard: every swing that reaches the capsule hits (SPEC-0036 Hit detection step 4 never applies).
- Hit reaction: kind 0 (id 57), Drackonack 53 ticks, Giant 33 ticks; no knockback on the hero; the reaction moves the enemy backward (8.3 and 3.8 units). First hit on a non-agro enemy is doubled.
- Damage taken from the hero's sword (R-0146; High): both rows have defence 0 and no element or area rule beyond the Mote match (target type 1, matched by Mote kind 4), so a plain swing deals the sword's stat A (7 for the Squire's Sword, 40 for the Altadorian Sword), the finishing swing twice that and the chained repeat 0.75 times it (SPEC-0036, Damage). Drackonack 55 hit points: sword 105 kills it in 2 plain swings (or 1 on a first hit, 80); the Giant Drackonack, 300 hit points, in 7 swings from a first hit. The Park Shrine (pack 1398) Drackonacks (2 Drackonacks and 3 Giant Drackonacks) are the one exception: the pack script keeps them undamageable at every tick, so a sword hit computes its normal figure and plays the reaction but removes no hit points; they cannot be killed and no player action hurts them (SPEC-0043, 1398; SPEC-0038 Behavior 37). Every other placement, including Drackon Ridge (pack 714), takes the full damage.
- Counters. Slot 1 (trigger 6): eligible each tick of the reaction when its cooldown is below 1, d is inside its band (Drackonack 0 to 15, Giant 0 to 25) and the hero is within 15 degrees of dead ahead; one roll int[0, 100] below 30 (Drackonack) or 60 (Giant) per tick. For the Giant that is 60 of 101 a tick over 33 ticks, so a hit from inside 25 units is answered by the 50-damage charge unless slot 1 is on cooldown (60 to 120 ticks after its last use). The Drackonack answers 30 of 101 a tick over 53 ticks (essentially always) but its slot 1 cooldown is 240 to 420 ticks. Slot 0 (trigger 2, chance 50 or 60): eligible only while the enemy is in a dodge (outer 5 or 6) and the active hero's swing recover window is open (Tor's attack clips carry it; Roberta's never do), with the cooldown below 1, d below 10 and the hero within 15 degrees: one roll per tick; a success replaces the dodge with a charge. The slot-0 counter therefore only appears against Tor, in the first few ticks of a dodge, and is rare.
- Drackonack: also, every counter uses the stale charge distance of the last normal choice.

#### Hero actions against these enemies

| Hero action | What the Drackonack does | What the Giant does |
|---|---|---|
| Stands still outside the notice range | Wanders, patrols, guards or hides | same |
| Walks into the scaled notice range | Notice chance per 32-tick cycle 0.75 x chance(d) in the cone; two-stage wake and 96-tick alert | same |
| Tor quick or normal swing in front, within 8.0 | Dodges 49 of 100 (89-tick hop back 19.8 units) | Dodges 84 of 100 (39-tick slide back 9.2 units) |
| Tor heavy swing | 49 of 100 | 89 of 100 |
| Tor power step | Never dodged | Never dodged |
| Roberta's shots | Dodges 74 of 100 when the path touches within 30 ticks | same |
| A landed hit | Damage by SPEC-0036 (no defence); doubled when not in agro; reaction 53 ticks; slot 1 counter 30 of 101 a tick | Reaction 33 ticks; slot 1 counter 60 of 101 a tick |
| Hero guard (shield) | Blocks the bite when the hero faces it (dot at most -0.5) | same |
| Hero runs from a charge | The run goes straight to its drawn distance (5 to 15) and ends | same |
| Hero walks past a sprinting Drackonack | The approach is 26.5 u/s and the charge 30.4 u/s; walking (13.5) and running (19.3) cannot outrun it | The Giant's rush is 13.8 u/s, slightly faster than the hero's walk |
| Hero hurt, knocked down or dead | No new choice; cooldowns stop; a running charge continues | same |
| Hero swap | Next tick every test uses the new hero | same |

#### Edge cases for implementers

- The charge distance is a draw of 5 to 15 units and the bite range is 5.0, so an attack from d > 20 never connects. A standing hero at d = 12 is bitten with probability 80 percent per charge (draw at least 7); at d = 17 with probability 30 percent.
- The Giant's slot 1 is eligible out to 25 units (49 percent a tick), so a Giant that wakes 25 units from a standing hero usually spends its first charge on empty ground.
- A dodge or hit reaction aborts the sequencer with no end path: no recovery countdown, and the slot keeps the cooldown drawn at the choice.
- The Drackonack's strafe approach runs at 26.5 u/s and circles at 7.5 u/s; the lock rule (wall contact gives 120 ticks without circling) is the strafe rule of Round 0.
- Alarm groups in the Abandoned Mines hold 1 to 3 hidden Drackonacks each (group 3 holds Giants that are not hidden); waking one wakes the rest of its group within 45 ticks.
- A strike that leaves the hero at 0 hit points sends the enemy to the victory pose (id 73, 116 ticks), then the default-state step.

#### What a port most likely gets wrong

1. Treating the Drackonack bite as a normal swing. It is a run of 5 to 15 units at 24 to 30 u/s (Giant 11 to 14) followed by a 4-tick window; a hero who is more than 20 units away is never hit.
2. Steering the charge. The run keeps its heading from the choice tick and the bite hits only a hero who is still within 5.0.
3. The cooldowns. The Drackonack's slots wait 240 to 420 ticks each; the Giant's 30 to 120.
4. Dodge rates. The Giant dodges 84 to 89 percent of swings and the Drackonack 49 percent; the dodge animation carries it back 9 to 20 units.
5. The Giant's counter: 60 of 101 a tick on a hit makes a 50-damage charge almost certain when it is off cooldown.
6. Forgetting that hidden Drackonacks and Giants are hittable (cue word 1) and that the group wakes together.
7. Using the strafe machine for the Giant (it rushes) or the rush machine for the Drackonack (it strafes).
8. The backbiter: a facing hero halves the Drackonack's attack rolls, a hero looking away doubles the rate.
9. Playing the blocked-charge clip or the stun clips; kind-2 charges never enter them.

#### Timeline test vector

Drackonack, default 8, emerge distance 5, cue word 1, alarm group none; the hero stands still facing it, 3 units away from tick 0 (3 < 5 x 0.7 = 3.5).

| Tick | Event |
|---|---|
| 0 | Wake test passes: sub-state 1 |
| 1 | Emerge clip 1 requested (gate 85) |
| 86 | Gate passes: agro; 87 set-up (outer 4, inner 10, hold 0) |
| 88 | First choice: d = 3, slot 1 24 %, slot 0 11 % per tick (hero facing it); also inner 10 sees d below 4: inner 11, back step at 7.5 u/s |
| 88 + G | A slot chosen (G geometric, mean about 2): state 6; B requested; d at least 3 (the Drackonack may have backed off a little) |
| 89 + G | State 7: the hero within 5.0, so state 8 next tick |
| 90 + G | State 8: C requested (c0) |
| 94 + G | Hit, 7 damage (weapon 3 window c0 + 4 to c0 + 7) |
| 119 + G | C gate; 120 + G state 12 end path (c0 + 30): recovery int[15, 30], the used slot's cooldown int[240, 420] |
| about 135 to 150 + G | The other slot (cooldown 0) may start: slot 1 24 % a tick, slot 0 11 %: the second bite lands within about 160 ticks; both slots are then cooling for 240 to 420 ticks |
| 600 | A motionless hero has taken about 3 to 4 bites, 21 to 28 damage with no armour; the Drackonack strafes between them |

Giant, default 8, emerge distance 40, hero standing 25 units away facing it: wake at tick 0 (25 < 28); agro at 86; first choice at 88 with d about 25: slot 1 49 percent a tick (arc permitting). The charge (R 0.8, 0.184 per tick) runs 5 to 15 units (27 to 82 ticks) and stops short at d = 10 to 20: no bite (probability 0 of reaching 5.0 from 25), end path, recovery int[15, 30], cooldown int[60, 120]. The rush approach (id 21, 13.8 u/s) then closes to 7 to 13 units; slot 1 again at 49 percent a tick from d of 25 down to 10 (bite reached with probability (20 - d) / 10) and 100 percent below 10, with slot 0 (24 percent, 25 damage) added inside 10. After arriving, the Giant starts a new bite about every 55 to 125 ticks (each slot repeats every 31 ticks of attack plus its 30 to 120 tick cooldown, and the two slots interleave): about 7 to 9 bites over 500 ticks on a motionless hero, 50 or 25 damage each, which is why the Giant is dangerous to stand in front of.

#### Open questions

1. The length of the bite window: the Drackonack's C clip has weapon 3 only on key 1 (seen on 4 ticks). Resolved by the Weapon reach table (window c0 + 4 to c0 + 7). Default if unresolved (rewrite choice): the bite hits a level hero from 4.2 behind to 7.0 ahead of the Drackonack (1.8 behind to 6.2 ahead for the Giant) during those 4 ticks.
2. Whether the hero's position for the "within 5.0" test of state 7 is 3D or horizontal. Default if unresolved (rewrite choice): horizontal (needs a runtime check).
3. The first charge distance at creation (before any normal choice): the creation draw reads the two row words just below slot 0's charge range (SPEC-0038 Edge cases, charge distance draws). On the Drackonack rows these are the all-ones word (a not-a-number float) and 0.0, so the drawn value depends on how the float draw treats a not-a-number bound, which only a run can show. It matters only if a counter-attack starts before the first normal choice. Default if unresolved (rewrite choice): float[5, 15), a fresh draw at creation (needs a runtime check; proactive (open close-out)).
4. Resolved (movement pass): ids 29 and 30 share clip 27, id 29 mirrored; inner 8 (id 30) travels to the Drackonack's right and inner 9 (id 29) to its left, as the Thief (SPEC-0038 Behavior 51 item 7).
5. Whether the dodge hop of clip 8 (19.8 units) is always inside the navmesh. Default if unresolved (rewrite choice): the wall rule of the common movement stops it at the mesh edge (needs a runtime check; depends on the level mesh near each placement).
6. The tough-hide word of the Giant: no class-17 routine reads it (same as the Rogue's word). Resolved (Medium-High; proactive (open close-out)): no effect.

### Rogue in the field (differences from Round 0)

The Rogue (row 153, monster type 72 variant 1) keeps every value of Round 0: hit points 80, strafe band 6 to 20, the three-hit unblockable slot 3, the steal charge of slot 2, the 53-tick hit reaction and the coin detour. In the field only the start differs. 16 placements are in play: default 0 on 4 (Farm Road 3 and one pack record, idle until hit or alarmed), default 5 on 6 (all in the Bandit Caves, wandering), default 7 on 3 (two in the Bandit Caves, one in Western Brightvale, guards with cue word 0), default 8 on 3 (Western Brightvale, Meridell Village, and one Coliseum record). Cut: 9 more. Field-specific facts:

- Notice (used from states 5, 6, 7 and 2): cone 0.92388, awareness 75, timer 30 (cycle 32 ticks), chance 75 at range 10 falling to 10 at range 60; a standing hero inside 7 is noticed whenever roll A passes (75 percent per cycle) and out to 42 with falling chance (37 percent per cycle at 30 units in the cone); a running hero out to 105. This is a much longer sense range than the Thief (21).
- Alert and search clips: id 18 (clip 32, 20 keys, gate 77), id 17 (clip 31, gate 69), id 16 (clip 30, 45 keys, gate 176). The stand-by clips: walk id 2 and patrol id 4 are clip 1 (12 keys, loops, 6.26 u/s), rest fidget id 3 and patrol fidget id 6 clip 18 (37 keys), guard look id 8 clip 19 (15 keys).
- Hidden placements: the table has no hide clip (id 9) and no emerge clip (id 10), so the Rogue shows clip 0 while hidden and goes straight to agro on the tick after the wake test (Round 0, "Arena start and wake-up"); it is hidden from hits only when its cue word is 0 (one of the three hidden ones; the Coliseum and Meridell Village records have cue word 1 and emerge distance 40).
- The three field records with alarm groups are group 1 (Farm Road, default 0), groups 3 and 4 (Western Brightvale, with a Thug and a Bandit).
- Socialize: flag 1, pair radius 20.
- Weapon reach: the same as Round 0 (SPEC-0063, Round 0 "Weapon reach"): slot 0 touches a level hero from 1.8 behind to 4.0 ahead at the tick, the steal swing 2.4 behind to 9.0 ahead, and the slot 3 three-hit sequence 4.2 behind to 8.6 ahead.
- Movement (movement pass; the Thief's table with these differences): wander and patrol 6.26 u/s (0.104 per tick, with a small side drift on the last key); approach 28 and charge 44 are clip 40 at 10.90 u/s (0.142 to 0.242 per tick inside the cycle); the named-target follow (id 11) is the same 10.90 clip; circles 29 and 30 as the Thief (6.47 and 6.58). The back-off id 32 is a one-shot clip of 12 keys (constant 0.417 per key): it moves 0.104 per tick backward for its first 48 ticks and then, held finished, 0.209 per tick (12.5 u/s) for as long as the back-off lasts (SPEC-0038 Behavior 51 item 15). The hit reaction 57 slides 2.84 back over 56 ticks; dodges as the Thief; corpse no travel.

### Coliseum rounds (Altador Coliseum, pack 1371)

Not covered by the Meridell arena spec. Round 1 loads region 1372 (a mixed brigand ring) and round 3 loads region 1374 (a Drackonack ring) (round numbers as in SPEC-0041, module 1371); the round is won when all six characters of the region are dead and lost when both heroes are dead. Each region holds six records on a ring of radius about 26.4 around the point (0, 7.2, 0), at bearings 0, 45, 90, 180, 225 and 270 degrees. Every record has default 8, emerge distance 40.0 (28 for a standing hero, 70 for a running one), emerge selector 0, cue word 1 (hittable while hidden), alarm group none and no drop.

| Round | Ring, in bearing order | Notes |
|---|---|---|
| 1 | Bandit (0), Rogue (45), Thug (90), Bandit (180), Thug (225), Thief (270) | Emerge clips: Bandit, Thug and Thief 57 ticks; the Rogue none. Attack manager at starting gear: one attacker at a time, the rest wait in line (stand-off 20 for Bandits and Thugs, 10 for the Thief and Rogue) |
| 3 | Giant Drackonack (0), Drackonack (45), Drackonack (90), Giant Drackonack (180), Drackonack (225), Drackonack (270) | Emerge clip 85 ticks. Capacity 0 allows one attacker (weights 12 and 8); the others wait in line at stand-off 15 |

With no alarm group each enemy wakes by its own distance test; a hero standing anywhere near the centre of the ring (every record within 28 units) wakes all six on the same tick. A hit on any of them while hidden wakes it with doubled damage. Everything else is the single-kind guides above (the Thug's grab, the Bandit's blocks, the Thief's shot and coin detour, the Drackonack charges).

### Batch open questions

1. Which hero distance feeds the notice and wake tests when the hero is airborne. Resolved for the wake test (High; proactive (open close-out)): the horizontal distance (SPEC-0038 Behavior 40 worked example: a hero 30 units above and 6.99 horizontal units away still wakes the enemy). Default if unresolved (rewrite choice) for the notice test: horizontal distance (needs a runtime check).
2. Resolved (Medium-High; proactive (open close-out)): same as the Thief item above: a hidden cue-0 ambusher is not drawn (rewrite choice: do not draw it; a runtime check could only confirm).
3. The reach of a chat pair: Resolved (Medium-High; proactive (open close-out)): the test compares the separation of the two actors' current positions with the sum of their pair values (SPEC-0038 Behavior 10); the peer's home plays no part.
4. Whether scripts rewrite the default state of the default-0 placements (Farm Road, two small packs) when their scenes trigger. Default if unresolved (rewrite choice): leave them idle until hit or alarmed; level-script handling is in SPEC-0030 (SetAI_Behavior).


## Skeleton Legion

Kinds: Dumb Skeleton (monster type 88, variant 0, row 226), Skeleton Fighter (type 88, variant 1, row 227; field variant, the arena copy is Round 3 of the arena-fighters spec), Skeleton Archer (type 87, variant 0, row 224) and Skeleton Wizard (type 87, variant 1, row 225). All four are class 17 (common enemy, think loop and agro machine of the common-enemy spec) and use one skeleton rig: the four character records carry the same 34 clips and differ only in which action id points at which clip. Three of the rows (224, 226, 227) are the reanimating rows: they do not stay dead. The Wizard is the one enemy that can raise a dead skeleton before its timer runs out. Act 3 is where the player meets them (Oubliette, Haunted Tombs, the Crypts, Dungeon Ruins, Old Mine, Ancient Castle Spires, Black Keep Dungeon, Werelupe Graveyard, Rathbone Family Crypts, Meridell Plains and the Coliseum skeleton round).

Confidence: High for the row values, the clip data, the tick counts of each phase and the placement counts; High for the reanimation steps, the Wizard's spell steps and the streaming consequences; Medium-High for the ordering of a few same-tick steps (plus or minus one tick, stated where it matters); Medium for the weapon-volume geometry (same limit as the arena guides); Medium-Low for the state-9 target of two Haunted Tombs records (see Open questions).

Sections below: shared rules of the legion (animation table, timings, field start states and wake rules, notice values, the alert sequence, alarm groups, breakable summons, dying and rising), then one build guide per kind. Rules that the common-enemy spec already states (think loop, pre-step, notice, setter, attack sequencer, counter choice, defence, take-hit hook, attack manager, kill memory, streaming) are used as written there; this section gives only the values and the combinations that are specific to the legion.

### Shared rules of the legion

#### Animation ids and clips (all four kinds)

The four tables hold 34 clips (numbers below are clip numbers of the character's own set; every clip has speed byte 2, 0.25 keys per tick at play rate 1.0, except clips 13 and 14 which have speed byte 1, 0.5 keys per tick). Ids not listed are absent from that table (a lookup miss starts clip 0, except the slot-phase ids and fidgets, which are skipped).

| Use (action id) | Dumb (88/0) | Fighter (88/1) | Archer (87/0) | Wizard (87/1) |
| --- | --- | --- | --- | --- |
| Idle base (0) | 0 | 0 | 5 | 5 |
| Wander walk (2), sentry walk home (2) | 2 | 2 | 6 | 6 |
| Patrol walk (4) | 2 | 2 | 6 | 6 |
| Wander rest fidget (3), sentry fidget (8) | 15 | 15 | 15 | 15 |
| Patrol pause fidget (6) | 16 | 19 | 16 | 16 |
| Sentry hold (7) | absent (clip 0) | absent (clip 0) | absent (clip 0) | absent (clip 0) |
| Hide pick (9) | 11 | 11 | 11 | 11 |
| Emerge (10), arrival (82) | 10 | 10 | 10 | 10 |
| Search while alerted (16) | 17 | 17 | 32 | 29 |
| Alert start (18) | 18 | 18 | 18 | 18 |
| Give-up (17) | 2 | 2 | absent | absent |
| Rush walk (21), rush back-off (23) | 24, 12 | 24, 12 | not used | not used |
| Rush hold (22) | 25 | 19 | not used | not used |
| Keep-distance walk (24), hold (25), back-off (26) | not used | not used | 24, 20, 12 | 24, 25, 12 |
| Strike (34) / second strike (39) / third (44) | 4 / 3 / not used | 4 / 3 / 21 | 9 / not used | 9 / 9 (never chosen) |
| Blocked (36, 41, 46) | 22, 22, not used | 22, 22, 22 | not used | not used |
| Melee dodge (55) | 27 | 27 | 33 | 27 |
| Missile dodge (53, 54) | 7, 8 | 7, 8 | 7, 8 | 7, 8 |
| Block (56), block flash (126) | 23, 28 | 23, 28 | 23, none | 23, 28 |
| Hit reaction kind 0 (57) | 1 | 1 | 1 | 1 |
| Death (58) | 13 | 13 | 13 | 13 |
| Rise (85) | 14 | 14 | 14 | 14 |
| Victory pose (73) | 19 | 20 | 20 | 20 |
| Map-search attack (137) | absent | absent | absent | 9 |
| Idle fidgets (115, 116, 117, 118, 119) | 16, 19, 20, 15, 17 | 19, 20, 16, 15, 17 | 19, 15, 20, 32, 16 | 15, 19, 16, 32, 20 |

Clip data (keys, loop, ticks to the 0.99 progress gate at play rate 1.0, root travel over the whole clip in units along the facing; z is forward):

| Clip | Keys, loop | Gate (ticks) | Root travel / events |
| --- | --- | --- | --- |
| 0 | 22, loop | 87 per cycle | none |
| 1 (hit reaction) | 17, one-shot | 65 | -5.72 (pushed back) |
| 2 (walk/give-up) | 17, loop | 67 per cycle | +3.52 per cycle (3.11 units per second) |
| 3 (strike B of slot 1) | 17, one-shot | 65 | +5.49; weapon 1 on keys 7 and 8 |
| 4 (strike B of slot 0) | 14, one-shot | 53 | +3.53; weapon 1 on keys 9 to 12 |
| 5 (Archer and Wizard idle) | 18, loop | 71 per cycle | none |
| 6 (Archer and Wizard walk) | 10, loop | 39 per cycle | +3.95 per cycle (5.92 units per second) |
| 7, 8 (missile dodge right, left) | 13 and 15, one-shot | 49 and 57 | +6.07 and -6.08 sideways |
| 9 (Archer and Wizard cast) | 15, one-shot | 57 at rate 1.0 | none; release bit 0x20 on key 10 |
| 10 (emerge, arrival) | 46, one-shot | 180 | none |
| 11 (hide pick) | 1, loop | 3 | none |
| 12 (back-off) | 10, loop | 39 per cycle | -11.64 per cycle (-17.45 per second) |
| 13 (death) | 56, speed byte 1, one-shot | 110 | -0.82 (back) and 0.16 to the side (-x) over 112 ticks, at most 0.05 per tick; frame-stored root (mode 0); last key 0 |
| 14 (rise) | 39, speed byte 1, one-shot | 77 | +0.82 |
| 15 (fidget) | 22, one-shot | 85 | none |
| 16 (patrol pause) | 19, one-shot | 73 | -0.92 |
| 17, 32 (search) | 71, one-shot | 280 | none |
| 18 (alert) | 32, one-shot | 124 | none |
| 19 | 32, one-shot | 124 | none |
| 20 | 36, one-shot | 140 | none |
| 21 (slot 2 lunge, Fighter) | 14, one-shot | 53 at rate 1.0 | +9.62; weapon 1 on keys 5 to 13 |
| 22 (blocked) | 15, one-shot | 57 | -11.07 |
| 23 (block) | 16, loop | 63 per cycle | none |
| 24 (walk forward) | 11, loop | 43 per cycle | +11.0 per cycle (15.01 units per second, 0.2502 per tick) |
| 25 (Dumb hold, Wizard hold) | 22, loop | 87 per cycle | none |
| 27 (melee dodge, Dumb, Fighter, Wizard) | 17, one-shot | 65 | none |
| 28 (block flash) | 6, one-shot | 21 | -1.67 |
| 29 (Wizard search) | 37, one-shot | 144 | +0.55 |
| 33 (Archer melee dodge) | 22, one-shot | 85 | none |

The gate is the first tick after the request tick in which the clip's progress reaches 0.99 (common-enemy spec, Behavior 13). Hit windows for the melee strikes and the release bit of clip 9 are given in the kind sections.

#### Field placements and start states

Records are counted over the shipped, reachable placements (the unreached test block excluded). Default-state word 5 = wander, 6 = patrol, 7 = sentry, 8 = hide and emerge, 9 = approach a named target.

| Kind | State 5 | State 6 | State 7 | State 8 | State 9 | Total | Summon-reserved (included) |
| --- | --- | --- | --- | --- | --- | --- | --- |
| Dumb Skeleton | 26 | 1 | 1 | 56 | 2 | 86 | 6 (Haunted Tombs breakables) |
| Skeleton Fighter | 4 | 2 | 1 | 29 | 0 | 36 | 2 (Haunted Tombs breakables) |
| Skeleton Archer | 16 | 1 | 11 | 20 | 0 | 48 | 0 |
| Skeleton Wizard | 4 | 0 | 6 | 4 | 0 | 14 | 0 |

Main places. Dumb: Oubliette 23 (20 wanderers, 3 hidden), Haunted Tombs 15, Meridell Plains 9 (three groups of three, hidden), Werelupe Graveyard 9 (hidden), Black Keep Dungeon 6, Old Mine 6, Crypt of the Exiled 6, Rathbone Family Crypts 5, Dungeon Ruins 4, Brightvale Outskirts 3. Fighter: Crypt of the Fallen 10 (all hidden), Old Mine 6, Dungeon Ruins 5, Haunted Tombs 5, Crypt of the Exiled 4, Rathbone Family Crypts 2, Coliseum 2, Ancient Castle Spires 1, Meridell arena 1. Archer: Crypt of the Exiled 10, Dungeon Ruins 9, Oubliette 8, Ancient Castle Spires 6 (all wanderers over a 1000-unit square), Black Keep Dungeon 5, pack 685 3 (sentries), Old Mine 3, Coliseum 2, Rathbone Family Crypts 2. Wizard: Old Mine 3, Ancient Castle Spires 3, Haunted Tombs 2, Crypt of the Exiled 2, Black Keep Dungeon 1, Dungeon Ruins 1, Coliseum 2.

Coliseum skeleton round: six records (two Archers, two Wizards, two Fighters), all default state 8, emerge distance 40.0, hittable while hidden (cue word 1), no alarm group. They start exactly like the arena Fighter of the arena-fighters spec.

How a record reads its words (spawn record offsets; confidence High):

| Default state | Words read | Behaviour |
| --- | --- | --- |
| 5 wander | +0xBC side of the wander square (float), +0xC0 and +0xC4 rest range in ticks (integers, either order), +0xC8 rest chance in percent | Per tick the rest timer counts down while positive. With no goal a goal is chosen: when +0xBC is below 0.1, the centroid of a random triangle of the enemy's navmesh; otherwise home plus (u, 0, v) with u and v each an integer from 0 to trunc(+0xBC) - 1 minus +0xBC / 2, accepted when it lies on the navmesh and otherwise moved half the way toward the enemy (up to 10 times, then home). While the rest timer is below 1 the enemy walks to the goal with the wander gait (action id 2) and tests arrival (horizontal distance below 0.5, or blocked with the stuck counter at 120, or no route). On arrival the goal is dropped and a roll int[1, 100] below +0xC8 starts a rest of int[+0xC0, +0xC4] ticks (idle, with the rest fidget id 3 whenever a second draw of the same range has run out). +0xC8 = 0 means the enemy never rests: 23 of 26 Dumb wanderers, 2 of 4 Fighters and 10 of 16 Archers keep walking from goal to goal |
| 6 patrol | +0xB8 path index of the area (valid from 0 up to the area's path count), +0xBC (0 = loop the path, nonzero = reverse at the ends), +0xC4 and +0xC8 pause range at each point, +0xCC mid-walk pause chance, +0xD0 and +0xD4 its range (0 on all shipped records, so a mid-walk pause lasts 0 ticks) | Walks the path points in order with the patrol gait (id 4, arrival 0.5 horizontal); at each point pauses int[+0xC4, +0xC8] ticks with the pause fidget (id 6). Shipped: Dumb path 0 with a pause of 120 to 300, Fighter path 5 (pause 20 to 120, mid-walk chance 20 %) and path 1 (no pause), Archer path 4 (no pause) |
| 7 sentry | +0xC4 | More than 0.5 from home: walks home with the wander gait. At home with +0xC4 = 0: stands, plays the sentry fidget (id 8, clip 15) each time a timer of int[60, 600] ticks has run out (the timer is 0 at creation, so a fresh sentry looks first); hold id 7 is absent from the tables, so the base idle of clip 0 plays. With +0xC4 nonzero (3 Archers, 1 Fighter: value 1 or -1): turns to the record's facing and holds. A released sentry is bound again at its disc position |
| 8 hide | +0xC0 emerge distance, +0xC4 cue word, +0xC8 selector (0 on every skeleton record: ids 9 and 10) | Stands at home playing the hide pick (id 9, clip 11). The first think tick on which the hero's horizontal distance is below +0xC0 times F1 sets sub-state 1; the next tick requests the emerge (id 10, clip 10, 180 ticks) and sets sub-state 2; the tick the gate passes the enemy enters agro (setter 19). F1 by hero action state: 0.7 idle, 0.75 slow walk, 1.5 walk, 1.75 run, 1.0 otherwise. Cue word 0: hidden kind (not hittable, not targetable and not an auto-target while the hide clip is current; the emerge makes it visible). Cue word 1: hittable while hidden; a hit skips the emerge, doubles the damage and enters agro |
| 9 approach | +0xBC target character id (integer), +0xC0 stop distance, +0xC4 | Walks toward the character with that id until the horizontal distance is below +0xC0, then (+0xC4 = 0) stands in the idle pick. A missing target runs the wander routine. Only two Dumb Skeletons use it (Haunted Tombs, target id 10, stop distances 5.0 and 10.0) |

Emerge distances per kind (state 8): Dumb 1.0 (1), 4.0 (5), 5.0 (10), 6.0 (13), 10.0 (5), 15.0 (8), 20.0 (14) with cue word 1 on 19 and 0 on 37; Fighter 0.0 (3), 4.0 (11), 6.0 (3), 10.0 (2), 15.0 (4), 18.0 (1), 20.0 (2), 40.0 (3) with cue word 1 on 6; Archer 4.0 (2), 5.0 (1), 7.0 (2), 10.0 (9), 12.0 (1), 16.0 (1), 20.0 (2), 40.0 (2) with cue word 1 on 3; Wizard 10.0 (2) and 40.0 (2) with cue word 1 on 2. A distance of 0 never passes the emerge test (the test is strict and the product is 0): the three Fighters with distance 0 (one in Old Mine with no alarm group, two in the Crypt of the Exiled in alarm groups 3 and 6) wake only through their alarm group or a script, and with cue word 0 they cannot be hit while they hide.

Losing the hero. Every way out of agro that does not end in a death ends in the default case of the setter (setter 0): the leash (no route to the hero for 181 consecutive ticks while the walk waypoint is at least 15.0 from the hero), the lost-sight search giving up (counter above 120), the search timer of state 2 running out, and the victory pose after the hero dies. The setter leaves class state 0 (these rows show in their default state), and the pre-step of the next tick runs the setter with the record's default-state word. The enemy then wanders, patrols, returns to its sentry post, or walks home and hides again in state 8 (the same emerge distance and cue word as before). The enemy keeps no memory of the lost fight.

#### Notice values (states 3, 5, 6, 7 and 9)

Notice is evaluated once per awareness cycle (the countdown is 0 for exactly one tick; cycle length = awareness timer + 2 ticks). Roll A is int[1, 100] at most the awareness value; then the band, the sight line and roll B as in Behavior 3 of the common-enemy spec.

| Item | Dumb | Fighter | Archer | Wizard |
| --- | --- | --- | --- | --- |
| Awareness (roll A passes when at most) | 65 | 80 | 65 | 100 (always) |
| Cycle length | 32 ticks | 32 | 52 | 52 |
| Sense band (near / far, times F1) | 3 / 5 | 3 / 5 | 10 / 120 | 10 / 120 |
| Chance at the near edge / far edge | 50 / 1 | 75 / 1 | 50 / 100 | 50 / 100 |
| View cone (cosine; chance halved outside) | 0.9239 | 0.9239 | 0.0 | 0.0 |
| Always sees through obstacles | no | no | no | no |
| Search countdown after a notice | int[60, 119] | same | same | int[60, 179] |

Inside the near edge the check passes at once after roll A. Between the edges the chance falls (Dumb, Fighter) or rises (Archer, Wizard) linearly with distance, computed over the unscaled band (so for a hero whose F1 is below 1 the chance just outside the near edge can exceed the near-edge value: a Dumb Skeleton gives 62 % at 2.5 units against a standing hero). Example: Dumb with a standing hero (F1 0.7) is noticed at 2.1 units or nearer; with a walking hero (F1 1.5) at 4.5 or nearer, and between 4.5 and 7.5 with a falling chance; a running hero (1.75) is noticed from 5.25. An Archer notices a standing hero inside 7 units for certain (after roll A), and out to 84 units with a chance of about 49 % at 7 units rising to about 84 % at 84 units (the interpolation runs over the unscaled 10-to-120 band); a running hero is noticed out to 210 units, and beyond 120 units the chance is above 100 %. Sight blocked: the band is multiplied by F2 instead of F1 (0.0, 0.25, 0.5, 0.75 or 0.9 by hero action state), so a standing hero behind a wall is never noticed. Taking a hit sends the skeleton to agro at once (the first hit is doubled).

#### Alert sequence (every notice or alarm in the field)

1. Notice from state 3, 5, 6, 7 or 9, or an alarm-group check of a member in state 0, 3, 5, 6, 7 or 14: the setter for state 2 draws the search countdown, stores the hero's position as the last-seen point, and the state-2 handler runs on the same tick with a second notice test (fresh rolls). A hidden member (state 8) instead starts its emerge (sub-state 2): 180 ticks, then agro.
2. State 2, each tick: face the last-seen point; if neither the alert clip nor the give-up clip is current, play the search clip (id 16; Dumb and Fighter clip 17, Archer clip 32, Wizard clip 29), and use the idle pick when id 16 is absent; run the notice test; when it fails and the alert clip (id 18, clip 18) is not current, the search countdown goes down by 1 (60 to 119 ticks; the Wizard 60 to 179).
3. When the notice test passes, or the alarm check requested clip 18 directly: play id 18 (32 keys, 124 ticks to the gate; the skeleton stands still and keeps turning toward the last-seen point) and, when the gate passes, enter agro (setter 19). The notice result no longer matters once clip 18 is current.
4. When the countdown reaches 0 with the test failing: Dumb and Fighter play id 17 (clip 2, a looping walk clip; 67 ticks to the gate) and then run the setter with 0 (back to the default state). Archer and Wizard have no id 17 and leave at once.
5. A hit at any point of the sequence: the take-hit hook enters agro immediately, the first hit's damage is doubled, and the skeleton starts its hit reaction.

So a field skeleton that notices the hero starts fighting 124 ticks (about 2 seconds) after its first passing test or alarm check, and the whole sequence is interrupted by any hit.

#### Alarm groups

A record with alarm group byte not 255 broadcasts to its group when it enters agro (setter 19, including after a rise and after a Wizard cast). Every 15 to 45 ticks each member checks for a new broadcast: members in states 0, 3, 5, 6, 7, 14 enter state 2 and play id 18 (124 ticks, then agro); a member in state 8 plays the emerge clip (180 ticks) and enters agro (cue word 0 also starts a camera and sound cue). Groups of the legion: Haunted Tombs 0, 1, 2, 3, 8, 10 (groups of 2, 3, 2, 2, 2 and 1 records; groups 2, 3 and 10 are breakable summons), Crypt of the Exiled 1 (4 Archers), 2 (3 Dumb, 1 Wizard), 3 (2 Fighters), 4 (4 Archers, 1 Wizard), 6 (1 Archer, 2 Fighters; two of them sentries), Crypt of the Fallen 1 (3 Fighters), Oubliette 3 (3 Archers), Meridell Plains 5 (three Dumb in each of three regions), Werelupe Graveyard 1 (3 Dumb), Rathbone Family Crypts 3 (2 Fighters), Black Keep Dungeon 15 (2 Dumb, 2 Archers) and 20 (2 Dumb, 2 Archers, 1 Wizard). A group of one is inert. Waking a group is the usual way a crowd arrives together: one hit on one member wakes the whole group.

#### Breakable summons (Haunted Tombs)

Six Dumb Skeleton records and two Skeleton Fighter records of Haunted Tombs have the summon byte set and no position. Six breakable objects of that area carry the spawn key of the Dumb Skeleton (1950) and two carry the key of the Skeleton Fighter (1941). Breaking one makes a world-level spawn by key at the break point: the first unbound reserved record with that key is bound there, in agro, with the arrival flag (animation 82 is requested once; the agro think tick replaces it with the first idle or gait request, so it shows for at most a tick). The request does nothing when the embedded block already has 6 active characters (corpses included) or no reserved record with the key is free. The reserved records carry drop ordinals 31 to 38. A summoned corpse that is later released by streaming is not bound again by streaming (summon byte); a new break can bind it again.

#### Dying, rising and streaming (rows 224, 226, 227)

Death (High). The killing hit runs the death hook: HP 0, class state 16, the reanimation countdown drawn once as int[min, max] ticks (Dumb 8000 to 12000, Archer 3600 to 18000, Fighter 12000 to 18000; the death-disappear timer 240 or 320 never runs), payout of the drop (below), status timers cleared. The skeleton keeps its body and plays the death clip (id 58, clip 13, 110 ticks to the gate; it holds the last key). It is not a hit candidate, is dead for scripts and for the streaming dead test while the countdown is above 0, cannot be hurt or pushed by the hero, and takes no part in the attack manager (leaving agro releases its entry on the next frame).

Countdown (High). Each tick that the think loop is stepped (the enemy's zone active, world not frozen) a countdown of 0 or more plays the death clip and goes down by 1. A countdown that has reached -1 starts the rise.

Rise (High). Each tick: if the rise clip (id 85, clip 14, 39 keys, 77 ticks) is not current, it is requested only when the clear-area test passes (the hero's horizontal distance is at least 5.0 and no other character of the shared pool is within 5.0 horizontally; the skeleton itself is skipped); otherwise the death clip is requested again and nothing else happens. While the rise plays the skeleton is still class state 16 and cannot be hit. When the rise has passed its gate (77 ticks), HP is set to its maximum and the setter for agro runs (alarm broadcast to its group, outer sub-state 0, all cooldowns 0, attack manager entry): the skeleton stands up where it fell, already fighting, without a notice test or an alert clip. The countdown stays at -1, so a second death draws a new countdown and pays the drop again.

Wizard raising (High for the steps). A Skeleton Wizard's spell sets the countdown of its target to -1. A skeleton whose countdown is still running therefore skips the rest of it, and rises 1 tick after contact if the clear-area test passes (77 more ticks for the rise itself). A living target is unaffected (its countdown is already -1).

Drops (High). Each death, the first one and every later one after a rise, pays out the record's drop ordinal (SPEC-0021 groups and fixed items) and splits any carried Neopoints. Counts of records with no payout: Dumb 31 of 87, Fighter 12 of 37, Archer 29 of 49, Wizard 14 of 15; the paying records draw from drop group 2 (Dumb 27, Fighter 5, Archer 11), a fixed item 150 (Dumb 10), fixed coins of 50 (Dumb 9) or 17 (Fighter 4), and a group 0 (Fighter 6). A skeleton that is killed repeatedly pays repeatedly.

Streaming and cap (Medium-High). A corpse stays bound to its object slot and counts as active in its block's cap (6, or 12 in the two packs that raise it) for as long as it lies there: its death-disappear countdown never runs, so it is not released for being dead. It is released only when the hero is more than 280 units away (or the cell flag is clear), or when a nearer record swaps it out as the farthest bound character (the swap condition of the streaming rule). A released skeleton writes its current position and facing back to its record (a sentry still in mode 7 keeps its disc position) and is eligible again at once, because the reanimating rows ignore the kill bit: when the hero comes back within 280 units it is bound fresh from its row, full HP, in its default state, at that position. So the commonest way a player sees a skeleton "come back" is to leave the area and return. Entering another world and coming back restarts every skeleton from its disc point.

Interactions to keep (Medium): a pile of corpses can fill the six slots of a block and keep further records from being bound until the hero moves away; a corpse does not block movement of the hero beyond what its body does (unknown); the kill bit of a corpse is set every tick and ignored by the bind.

### Dumb Skeleton

The rush skeleton. It is the Skeleton Fighter's smaller sibling (same two swing clips, same rush movement, no lunge) and the most numerous undead in the game. Confidence: High for the row, clips and tick counts; Medium for the weapon geometry.

#### Identity and tunables

| Item | Value |
| --- | --- |
| Character record, monster row | name "Dumb Skeleton", type 88, variant 0, row 226 (resource 530) |
| Class, approach | 17 (common enemy), approach 1 (rush) |
| Hit points, radius, height, weight | 50, 2.0, 3.5, 5.5 |
| Difficulty (attack-manager weight), alignment, hostile, display in default | 5, 0, 1, 1 |
| Reanimation | flag set; countdown int[8000, 12000] ticks; death-disappear timer 240 never runs |
| Damage reduction, anger, can fly, map search, distance hide | 0, 0, 0, 0, 0 |
| Rush band | 3.5 to 5.0; desired distance 3.5 + 1.5 f with f = int[20, 80] / 100 (3.8 to 4.7), redrawn after every normal attack choice; back-off aims at the point hero minus direction x 7.0 |
| Recovery after an attack | int[15, 30] ticks |
| Search countdown on a notice | int[60, 119] |
| Melee defence | block chance quick / normal / heavy swing 40 / 25 / 10; dodge 0 / 0 / 0 (never dodges); defence timer after a defence int[45, 90]; Blocking Angle 0.707 |
| Missile defence | defend time 60 ticks; dodge 0; block 20 (19 of 100 rolls pass); only Roberta's shots trigger it |
| Weapon volumes (radius x length) | weapon 1: 2.0 x 2.0 (the only one its clips use); weapon 3: 3.0 x 3.0 (unused) |
| Stand-off radius (wait in line) | 12.0 |
| Notice | awareness 65, timer 30, cone 0.9239, band 3 to 5, chance 50 to 1 (shared table above) |

Attack slots (slots 2 and 3 unused). Ids A to E are 33 + 5k to 37 + 5k; "none" means the table has no clip and the phase is skipped in the same tick.

| Slot | Band (strict) | Chance (roll below) | Arc | Melee range | Cooldown draw | Speed words (wind-up / strike / recovery) | Damage | Knockback | Counter code / chance | Clips by phase |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| 0 | 0 < d < 10 | 60 (59 of 100) | 0.9239 | 7.0 | int[30, 240] | 1.0 / 1.0 / 1.0 | 30 | 0 | 3 (after its own block) / 25 | A none, B 34 = clip 4 (14 keys), C none, D 36 = clip 22, E none |
| 1 | 0 < d < 10 | 40 (39 of 100) | 0.707 | 7.0 | int[60, 120] | 1.0 / 1.0 / 1.0 | 40 | 0 | none | A none, B 39 = clip 3 (17 keys), C none, D 41 = clip 22, E none |

Flags, status and steal fields are all clear. Only weapon 1 is ever active, because both strike clips carry bit 0x1 only.

#### Start and wake

The 86 shipped placements start in default state 5 (26), 6 (1), 7 (1), 8 (56, 6 of them summon-reserved) and 9 (2). Start behaviour, notice values, the alert sequence, alarm groups and the way back to the default state are in the shared rules above. Kind-specific numbers:

- Wanderers (state 5): the Oubliette crowd (20 records) and one more walk a 50-unit square around home with the wander gait (clip 2, 3.11 units per second) and never rests; they notice only within 5 units of the hero (see the notice table). Three other wanderers rest (60 to 240 or 120 to 300 ticks, 25 to 50 %).
- Hidden (state 8): emerge distances 1.0 to 20.0; 37 records have cue word 0 (hidden kind: not hittable and not targeted while the hide clip plays), 19 have cue word 1 (hittable while hidden: Haunted Tombs, Meridell Plains and others). The hide clip is a single key, so a hidden skeleton is a standing pose.
- Meridell Plains: three regions each hold a group of three hidden skeletons (emerge distance 5.0, cue word 1, alarm group 5): the first one to be hit or to emerge wakes the other two within 15 to 45 ticks (they emerge, 180 ticks).
- Haunted Tombs: 6 reserved records are bound by breakables (shared rules); 2 records use state 9 (see Open questions).
- Return home after a lost fight: a hidden record walks home with the rush gait and hides again.

#### Per-tick order (agro)

As in the arena Skeleton Fighter guide: pre-step (pause test, awareness, alert flag, retarget, alarm check, map search flag (row flag 0, never), class switch); then the agro machine (recovery countdown; melee and missile defence; lost-sight test; normal attack choice; counter choice; distance-hide gate, never fires; outer sub-state switch; route flag and leash); then the class handler (defence hold down by one, hold animation or pursuit step), the animation pass and the motion pass.

#### State table

| State | Condition | Action id -> clip | Keys / speed | Gate ticks | Exits |
| --- | --- | --- | --- | --- | --- |
| 5 wander / 6 patrol / 7 sentry | default state of the record | 2 -> clip 2 (gait), 0 -> clip 0 (idle), 3 -> clip 15, 6 -> clip 16, 8 -> clip 15 | see shared table | see shared table | notice -> state 2 |
| 8 hide | hero not near | 9 -> clip 11 | 1 key, loop | none | hero inside emerge distance x F1 -> sub 1 |
| 8 sub 1 / sub 2 | next tick, then wait | 10 -> clip 10 | 46 keys | 180 | agro (setter 19) |
| 2 alerted search | notice or alarm | 16 -> clip 17, then 18 -> clip 18, give-up 17 -> clip 2 | 71 / 32 / 17 keys | 280 / 124 / 67 | 18 gate -> agro; timer out -> default |
| Agro outer 1 inner 1 (hold) | d between 3.5 and 5.0 | 22 -> clip 25 | 22 keys, loop | none | d < 3.5 inner 2; d > 5.0 inner 0 |
| inner 0 (approach) | d above desired | 21 -> clip 24 | 11 keys loop | none | d < desired inner 1 |
| inner 2 (back off) | d < 3.5 | 23 -> clip 12 | 10 keys loop, -0.2908 per tick | none | d > desired inner 1 |
| Outer 2, slot 0 | chosen | A skipped; B 34 -> clip 4; D 36 -> clip 22; E skipped | 14 keys | 53 | end path at +54 |
| Outer 2, slot 1 | chosen | B 39 -> clip 3; D 41 -> clip 22 | 17 keys | 65 | end path at +66 |
| Outer 2 state 4 | own strike blocked | D -> clip 22 | 15 keys | 57 | state 5, end path |
| Outer 5 / 7 | hero swing edge | 55 -> clip 27 (never) / 56 -> clip 23, 126 -> clip 28 | 17 / 16 / 6 keys | 65 / 63 per cycle / 21 | defence exit |
| Outer 6 / 8 | Roberta shot | 53 or 54 (never) / 56 | | | defence exit |
| Outer 10 kind 0 | any damaging hit | 57 -> clip 1 | 17 keys, -5.72 push-back | 65 | outer 0 |
| Class 10 | its strike leaves the hero at 0 HP | 73 -> clip 19 | 32 keys | 124 | setter 0, default state |
| Class 16 | killed | 58 -> clip 13 | 56 keys, 0.5 per tick | 110, holds | countdown, then rise 85 -> clip 14 (77), then agro |
| Breakable arrival | spawned by key | 82 -> clip 10 requested once, replaced by the first agro request | 46 keys | not waited for | agro |

#### Attack chooser

Runs each agro tick in outer 1 (3, 4 or 11) while the recovery countdown is below 1 and the hero is attackable (the hero's hurt, knockback and busy states pause the choice and the cooldown countdown). Slots are tested 1 then 0. A slot passes when its cooldown is below 1, 0 < d < 10, the facing cosine exceeds the arc (slot 1: 0.707, about 45 degrees; slot 0: 0.9239, about 22.5 degrees), and a roll int[1, 100] is below the chance. Per eligible tick: slot 1 39 %, then slot 0 59 % of the rest, so a choice has probability 1 - 0.61 x 0.41 = 75 % when both are off cooldown; 39 % when only slot 1 is, 59 % when only slot 0 is. On a choice the cooldown is drawn, the sequencer starts in state 0 and the desired distance is redrawn. The end path draws the cooldown a second time, so each cooldown is counted from the end of its swing; slot 0 stays off for 30 to 240 ticks, slot 1 for 60 to 120. Counter choice: slot 0 triggers after this skeleton's own block while the hero's swing is in its recovery window (Tor only), roll int[0, 100] below 25; the slot must be off cooldown, in band and in arc.

Sequencer: for d above the melee range 7.0 the sequencer first walks at the rush speed toward the hero (state 0, gait 0) until d is at most 7.0 (about 12 ticks from d = 10), then starts the strike clip. There is no wind-up, follow-through or recovery clip.

#### Movement

Rush archetype exactly as the arena Fighter: hold 3.5 to 5.0 from the hero, walk at 0.2502 units per tick (clip 24), back off at 0.2908 per tick, never faster. Field walking skeletons (states 5, 6, 7) move at 3.11 units per second only.

Movement table (row 226 Dumb Skeleton and row 227 Skeleton Fighter, the same clips; ground; radius 2.0; ground turn 1.0; rush band 3.5 to 5.0, desired 3.8 to 4.7):

| Mode | Source | Clip (keys) | u/tick | u/s | Turn | Exit |
| --- | --- | --- | --- | --- | --- | --- |
| Wander, patrol, walk home, wait in line, follow, confused (69, Dumb only) | root | 2, 4, 11 or 69 (17); 0.000 to 0.102 per tick inside the cycle | 0.0518 | 3.11 | steer | arrival 0.5, notice |
| Approach (inner 0), sequencer state 0 | root | 21 (11) | 0.250 | 15.01 | steer | d below desired |
| Hold (inner 1) | none | 22 | 0 | 0 | aim 1.0 | d below 3.5 or above 5.0 |
| Back off (inner 2), below 3.5 | root, facing the hero | 23 (10) | 0.291 backward | 17.45 | steer, back gait | d above desired |
| Strike clips | root | slots 0, 1, 2 (B) | | forward 3.49 over 56 ticks (slot 0), 5.62 over 68 (slot 1), 9.55 over 80 (Fighter slot 2 at speed 0.7) | sequencer aim | gate |
| Hit (reaction kind 0) and heavy hit 62 | root | 57 or 62 (17) | | back 5.68 over 68 ticks | snap to attacker | gate |
| Dodge 53 / 54 | root | (13) / (15) | | side 5.95 over 52 / 6.06 over 60 | | gate |
| Lying (class state 16) | root (held) | 58 (56, S1) | see below | | kept | the reanimation countdown |

The rush is the fastest gait of the legion: from the hold band, a hero that steps back is followed at 15 u/s, faster than a walking hero (13.5). Lying corpse (High for the data; Medium-Low that the step is applied to a dead body, SPEC-0040 Behavior 14D): the death clip 58 (clip 13, 56 keys at speed byte 1, 112 ticks) is a frame-stored clip (mode 0; SPEC-0005), so its root record is the last eight bytes of each frame record, not the root table. Its keys move the falling skeleton smoothly backward, at most 0.05 units per tick, 0.82 back and 0.16 to the side (-x) over the clip; the last key is 0, so the held clip adds nothing and the corpse lies still for the whole reanimation countdown. (Read from the root table instead, the clip would seem to jump between about +1.6 and -1.5 per key and slide the corpse at 67 u/s; a port must not do that.)

#### Hit and hurt windows (tick 0 = tick the sequencer requests the strike clip; the choice happened one tick earlier)

| Slot | Window (weapon 1) | First tick any level hero can be touched | First touch of a level hero straight ahead at the strike request | Gate tick | End path tick | Damage | Forward travel of the clip |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 0 | keys 9 to 12 -> ticks 36 to 51 | 36 | 40 (any start distance from 3.5 to 7) | 53 | 54 | 30 | 3.53 |
| 1 | keys 7 to 8 -> ticks 28 to 35 | 29 | 29 (any start distance from 3.5 to 7) | 65 | 66 | 40 | 5.49 |

One hit per swing, on the first window tick on which the weapon touches a hero capsule; the window start is not the hit tick. A level hero (feet at the skeleton's origin height) is touched by slot 0 only on ticks 36 to 44 of its window and by slot 1 on ticks 29 to 35. The "straight ahead" column counts the chase stop of the sequencer (SPEC-0038: in states 0 to 3 the root step is dropped while the two bodies are within the sum of their radii, 2.0 + 1.175 = 3.175 for Tor), so the clip's forward travel parks the skeleton against the hero and the first touch does not depend on where in the band it started. The hit list is cleared when the next attack starts. The weapon-1 cylinder is radius 2.0, length 2.0 at the attachment frame (Medium).

#### Weapon reach

Conventions are those of "Weapon reach tables" in the shared field behaviour. Row 226, Medium. The hit table above is authoritative for the first tick that touches; the table below gives where the hero must stand.

| Slot (phase, clip) | Weapon | Window ticks (level contact) | Centre forward / up / side | Tor ahead, at the tick | Tor ahead, from the start | Tor feet vs origin |
|---|---|---|---|---|---|---|
| 0 (B, clip 4) | w1 (2.0 x 2.0) | s+36 to s+51 (s+36 to s+44) | -1.5 to 3.6 / 0.9 to 3.8 / -2.3 to 3.0 | -2.8 to 6.0 | -0.4 to 9.1 | -3.5 to 5.5 |
| 1 (B, clip 3) | w1 (2.0 x 2.0) | s+28 to s+35 (s+29 to s+35) | 2.2 to 3.3 / 0.5 to 4.8 / -0.4 to 1.0 | 0.6 to 5.6 | 3.1 to 7.7 | -4.5 to 6.0 |

The weapon is small (radius 2.0), so slot 1 touches a level hero only from 0.6 to 5.6 ahead at the tick, and slot 0 from 2.8 behind to 6.0 ahead. The skeleton walks 3.5 (slot 0) or 5.5 (slot 1) forward during the clip (the forward travel column above), which is why the "from the start" column is larger.

#### Defence and counters

- Melee defence is evaluated once per hero melee swing (on the swing-start edge), only when the defence hold is below 1, the skeleton is within 8.0 units of the hero and inside the hero's front arc (dot above 0.5), and not mid-swing. One roll int[1, 100] at or below the block chance blocks: quick swing 40 %, normal 25 %, heavy 10 %. Tor's quick swing = attack sub-state 6, normal = 2, 3, 9, heavy = 4, 7, 8, 12; sub-state 10 never triggers a defence.
- Block (outer 7): faces the hero, plays 56 (clip 23, loops), base state 20 while the hero's swing lasts; a blocked swing sets the block-hit flag and plays 126 (clip 28) once; defence hold 60, then redrawn int[45, 90] at the exit. Hero swings arriving while it guards are blocked when the dot of the two facings is below 0.707; from behind (dot at least 0.707) they do nothing at all.
- Missile defence (Roberta's shots): a shot whose 60-tick path meets the body is blocked with probability 19 %; it never dodges.
- Counter: slot 0 after its own block (25 %), see the chooser.

#### Hero actions against this enemy

| Hero action | Result |
| --- | --- |
| Tor swing on a hittable skeleton, not guarding | damage = swing product, truncated, no reduction; hit counter, face attacker, outer 10 kind 0 (clip 1 restarted at key 0 on every hit), cancels a swing in progress (no recovery countdown; the slot cooldown keeps running), alert flag |
| Tor finisher (step factor 2.0) | double damage; block chance 10 % |
| First hit while not in agro (states 2, 3, 5, 6, 7, 8) | damage doubled once; the skeleton enters agro with full setter work and starts the hit reaction; a hidden skeleton with cue word 1 skips the emerge; with cue word 0 it cannot be hit until it begins to emerge |
| Killing blow | class 16, countdown drawn, drop paid, the rise or a Wizard spell brings it back |
| Hit on a rising or dying skeleton | none: not a hit candidate |
| Roberta shot | 19 % block; otherwise a normal hit |
| Hero guard (L1, state 20, shield) | blocks the strike when the facing dot is at most -0.5; the hero takes the strike's damage reduced by the shield's stat A; no counter for slot 1 (none); slot 0's counter only follows the skeleton's own block |
| Hero standing still | a hidden skeleton emerges inside emerge distance x 0.7; a wanderer notices within 2.1 units only |
| Hero running | emerge distance x 1.75; notice from 5.25 units |

#### Edge cases for implementers

- A hidden record with cue word 0 is invisible to the hero's swing and to auto-targeting until its emerge starts (sub-state nonzero); do not make it hittable just because it stands there.
- Slot chances are strict rolls (59 and 39 of 100). The cooldown is drawn twice (choice and end path).
- Notice happens once per 32 ticks; a standing hero outside 2.1 units is never noticed. The skeleton then needs 124 ticks of alert clip before it fights.
- The back-off, hold and approach inner states chatter at the band edge when the hero stands still.
- Death does not remove the skeleton. It keeps a slot in its block's cap (see the shared rules) and rises again; drops pay on every death.
- The summoned records of Haunted Tombs start in agro with no notice and no alert clip.

#### What a port most likely gets wrong

1. Making skeletons permanent corpses (or respawning them on a timer): the right rules are the countdown and the rise, plus fresh binds after streaming.
2. Giving every field skeleton the arena start (all in state 8). Only 56 of 86 are hidden; 26 wander and walk about.
3. Treating the hide state as hittable for all records, or as unhittable for all.
4. Missing the 124-tick alert clip after a notice, or using one notice radius for every hero action state.
5. Using the Fighter's numbers (arc 0.707 for both slots, damage 50 and 60, block 50 %): the Dumb Skeleton's slot 0 needs a tight 22.5 degree arc and does 30.
6. Playing a wind-up or substituting clip 0 for the missing phases.
7. Letting the first swing hit at once, or on the first window tick: the windows open 28 or 36 ticks after the strike request, and a hero standing in front (5 to 7 units, where the walk-in stops) is first touched 29 (slot 1) or 40 (slot 0) ticks after it.

#### Timeline test vector

Setup: a Dumb Skeleton in default state 8, emerge distance 20.0, cue word 1, no alarm group; the hero idle 12 units away on its facing line, never acting. T0 = first think tick of the skeleton with the hero in range (12 < 20 x 0.7 = 14).

| Tick | Event |
| --- | --- |
| T0 | State 8 sub 0: sub 1 |
| T0 + 1 | Request clip 10; sub 2 |
| T0 + 181 | Gate passes; setter 19 (alarm broadcast skipped: no group; cooldowns 0; attack manager entry) |
| T0 + 182 | Outer 0: outer 1, inner 1 |
| T0 + 183 | d = 12 > 5.0: inner 0, request clip 24; chooser first runs (d above 10: nothing) |
| T0 + 184 | AI sees d = 12.00; first root step applied after the AI |
| T0 + 192 | d = 9.998 < 10: chooser 75 % per tick (slot 1 first); expected choice at T0 + 192 to 194 |
| T0 + 193 to 206 | Sequencer state 0 pursues until d <= 7.0 (12 more ticks), then state 1 |
| T0 + 205 to 207 | Strike clip requested |
| T0 + 233 to 235 (slot 1) or T0 + 241 to 243 (slot 0) | Weapon window opens |
| T0 + 234 to 236 (slot 1) or T0 + 245 to 247 (slot 0) | First hit on the hero (29 or 40 ticks after the strike request): 40 or 30 damage |
| T0 + 259 to 261 (slot 0) or T0 + 271 to 273 (slot 1) | End path; recovery int[15, 30] |
| later | Next choice from the other slot (or the same one after its cooldown) at once after the recovery |

From the end of the emerge, with every swing connecting and the hero always attackable: 5 to 7 swings in 600 ticks (mean 5.8); slot 1 about 57 % of them (slot 0's cooldown is 30 to 240, slot 1's 60 to 120).

#### Open questions

1. Weapon-1 placement (attachment frame and axis) decides the first contact tick. Exact fixed-pose samples resolve slot 1 at tick 29 and slot 0 at tick 40 for the listed straight-ahead stationary cases; slot 2 has the non-monotone distance map in the timing table above. The distances are in the Weapon reach table. Stand-in only for a port without attachment frames (rewrite choice): count a contact on a window tick when a level hero is inside the "at the tick" range of the slot's row (2.8 behind to 6.0 ahead for slot 0, 0.6 to 5.6 ahead for slot 1).
2. State 9 target (two Haunted Tombs records, target character id 10). Resolved (High for the data, Medium for self-follow; proactive (open close-out)): a character of the pack's own block has as id its plain ordinal, and ordinal 10 of the Haunted Tombs block is the second of the two state-9 Dumb Skeletons itself (the first is ordinal 9; stop distances 5.0 and 10.0; neither is hidden or reserved). So the first skeleton walks to the second and stops within 5.0, and the second finds itself at distance 0, inside its 10.0 stop distance, and stands in the idle pick; neither runs the wander body. Rewrite choice if a port excludes the searcher from the lookup: the second one would then wander (no known exclusion).
3. Whether the sequencer's state 0 runs in the tick of the choice (a one-tick ordering question; needs a runtime check). Default if unresolved (rewrite choice): same tick.
4. Whether a corpse collides with the hero (player-reachable physics; needs a runtime check). Default if unresolved (rewrite choice): it does not.

### Skeleton Fighter (field variant)

The arena Skeleton Fighter (Round 3 of the arena-fighters spec, row 227) is this same enemy with the arena start. Everything in that guide applies to the field Fighter once it is in agro: identity and tunables, the three slots (slot 2 lunge), phase timing, movement, defence, hero actions, the chooser order 2, 1, 0, and all test vectors from agro on. This section gives only what changes in the field. Confidence: High.

#### What differs from the Dumb Skeleton (same rig and rush movement)

| Item | Dumb (226) | Fighter (227) |
| --- | --- | --- |
| Hit points, difficulty | 50, 5 | 75, 8 |
| Reanimation countdown | 8000 to 12000 | 12000 to 18000 |
| Slots | 0 and 1 | 0, 1 and 2 (lunge: range 50, chance 100, arc 0, fixed cooldown 180, strike speed 0.7, damage 50, clip 21) |
| Slot 0 arc / damage / cooldown / counter | 0.9239 / 30 / int[30, 240] / 3 at 25 | 0.707 / 50 / int[30, 180] / 3 at 50 |
| Slot 1 arc / damage / cooldown / counter | 0.707 / 40 / int[60, 120] / none | 0.707 / 60 / int[60, 180] / 5 at 50 (own strike blocked while the hero holds L1) |
| Melee block (quick / normal / heavy) | 40 / 25 / 10 | 50 / 35 / 10 |
| Missile block | 20 | 90 |
| Notice: awareness / near-edge chance | 65 / 50 | 80 / 75 |
| Rush hold clip (id 22) | clip 25, loops | clip 19, 32 keys, stays on its last key after 124 ticks |
| Victory clip (73) | clip 19 | clip 20 |

#### Weapon reach

Conventions are those of "Weapon reach tables" in the shared field behaviour. Row 227, Medium. Slots 0 and 1 have the same clips as the Dumb Skeleton (table in that guide); the lunge is new. The window columns are computed from the clip; the tick text of the arena guide is authoritative for hit ticks.

| Slot (phase, clip) | Weapon | Window ticks (level contact) | Centre forward / up / side | Tor ahead, at the tick | Tor ahead, from the start | Tor feet vs origin |
|---|---|---|---|---|---|---|
| 0 (B, clip 4) | w1 (2.0 x 2.0) | s+36 to s+51 (s+36 to s+44) | -1.5 to 3.6 / 0.9 to 3.8 / -2.3 to 3.0 | -2.8 to 6.0 | -0.4 to 9.1 | -3.5 to 5.5 |
| 1 (B, clip 3) | w1 (2.0 x 2.0) | s+28 to s+35 (s+29 to s+35) | 2.2 to 3.3 / 0.5 to 4.8 / -0.4 to 1.0 | 0.6 to 5.6 | 3.1 to 7.7 | -4.5 to 6.0 |
| 2 (B, clip 21) | w1 (2.0 x 2.0) | s+29 to s+74 (s+32 to s+74) | -0.1 to 3.7 / 1.8 to 4.5 / -2.4 to -0.6 | 1.6 to 5.6 | 6.7 to 14.7 | -3.0 to 6.0 |

The lunge runs the weapon for 46 ticks while the skeleton travels about 9 units forward, so a level hero is touched from the 4th tick of its window on, 1.6 to 5.6 ahead of the skeleton at that tick and 6.7 to 14.7 ahead of where it started (this assumes the lunge plays at 0.7 throughout; the real speed drops to 1.0 after the first contact, a small effect).

#### Field start states

36 shipped placements: state 5 (4), 6 (2), 7 (1), 8 (29; 2 of them Haunted Tombs breakable summons).

- State 8, 29 records, the arena's own state. Emerge distance 0.0 (3 records, never emerge by distance), 4.0 (11), 6.0 (3), 10.0 (2), 15.0 (4), 18.0 (1), 20.0 (2), 40.0 (3: the Coliseum round and the arena region). Cue word 1 (hittable while hidden) on 6 records, cue word 0 (hidden kind) on 23. Arena rules for the hero's distance and F1 apply unchanged; 4.0 and 6.0 mean the Fighter springs up only when the hero is almost on it (2.8 to 7 units).
- State 5, 4 records: a 1000-unit wander square with a 10-tick rest at 10 % (Ancient Castle Spires), squares of 20, 32 and 40 units (rest 60 to 240 ticks at 25 % on the 40-unit one, none on the others).
- State 6, 2 records: Dungeon Ruins and Haunted Tombs; paths 5 and 1.
- State 7, 1 record (Crypt of the Exiled, alarm group 6): its +0xC4 is -1 (nonzero), so it turns to its record facing and holds there; it is in a group with two Archers.
- The three emerge-distance-0 Fighters (Old Mine with no group; Crypt of the Exiled in groups 3 and 6) have cue word 0.

A wandering, patrolling or sentry Fighter notices at 3 to 5 units (awareness 80), then runs the alert sequence (search clip 17 for up to 60 to 119 ticks, alert clip 18 for 124 ticks) before agro; the first hit skips all of it.

#### Arena versus field, in one list

1. No scripted intro and no camera: the field Fighter begins in its record's default state.
2. Death: in the arena the win sequence deactivates it about 100 ticks after the kill. In the field it is a corpse for int[12000, 18000] ticks (200 to 300 seconds of loaded play), then rises where it fell with full HP and fights at once; or a Skeleton Wizard raises it earlier; or leaving and re-entering the area (280 units) rebinds it fresh.
3. The Fighter's drop is paid on every death.
4. Alarm groups (Crypt of the Fallen: three Fighters in group 1; Crypt of the Exiled: two in group 3) wake each other.
5. In the Haunted Tombs the two reserved Fighters are bound by breakables in agro, with no alert clip, in alarm group 3.

#### Hero actions, edge cases, port mistakes

As in the arena guide, plus the field items above and the Dumb Skeleton lists (hidden kind, 124-tick alert, summoned records, corpse behaviour, streaming). What a port most likely gets wrong in the field: treating all Fighters as emerge-at-40 ambushers (the emerge distance is 0 to 40 by record), and making the Fighter permanently dead after its first kill.

#### Timeline test vector (field notice)

A Fighter in default state 7 (hold), facing +z, hero idle 2.0 units in front of it (inside the near edge 3 x 0.7 = 2.1). N = the tick of its awareness cycle (every 32 ticks, phase arbitrary).

| Tick | Event |
| --- | --- |
| N | Roll A (at most 80, 80 %): pass -> noticed at once (inside the near edge, no roll B); state 2; the handler repeats the test (80 %): pass -> requests clip 18; fail -> search clip 17 plays |
| N + 32 k | Each later cycle: 80 % again; at most 60 to 119 ticks of search remain before the Fighter gives up (clip 2, 67 ticks, then back to hold) |
| M (the first passing tick) | Clip 18 requested |
| M + 124 | Gate passes; setter 19 |
| M + 125 | Outer 0: outer 1, inner 1; d = 2.0 < 3.5, inner 2 |
| M + 126 | Chooser: slot 2 at 99 %, else 1, 0 (d = 2.0 inside every band) |
| M + 127 | Strike clip 44 (lunge) requested; window opens 29 ticks later (M + 156) |
| M + 160 | First touching tick, 33 ticks after the request: the exact fixed-pose, level, straight-ahead sample at start distance 2.0 with the chase stop in force touches on tick 33; 50 damage before armour (the pass of this tick still runs at 0.7). The lunge is a melee swing, so the chase stop applies and the fighter does not step into Tor; a Tor displaced before the strike can shift the tick. Gate M + 190 (63 ticks after the request), end path M + 191 |

From the first agro tick, 600 ticks hold 7 to 8 swings (arena guide). A hit on the Fighter at any tick before M + 124 cancels the alert and enters agro at once (double damage on that hit).

#### Open questions

1. Old Mine Fighter with emerge distance 0, cue word 0 and no group: Resolved (High; proactive (open close-out)): the emerge test is strict (distance below 0 times F1), so it never passes; the placement has no alarm group, no script call wakes it and a hidden cue-0 enemy cannot be hit, so it stays hidden, undrawn, untouchable and non-solid for the whole game (SPEC-0038 Behavior 40, "Every way out of state 8" and the list of twelve such placements, which names this Fighter). Reproduce it as an inert placement.
2. Hit geometry and the same-tick start of the sequencer: as in the arena guide (same defaults).

### Skeleton Archer

**Player's answer (R-0149).** Close in with heavy swings: it dodges quick and normal swings 99 times in 100 and heavy ones 49 in 100, and a missile 89 in 100, but each dodge opens 30 to 120 ticks; it holds 15 to 25 units away, backs off below that, and never gives up (SPEC-0038 Behavior 44).

The keep-distance skeleton archer. One missile slot (a straight arrow), no melee, dodges instead of blocking. Confidence: High for the row, clips, and timings; Medium-High for the order of the cast; Medium for the arrow launch point and scenery collision.

#### Identity and tunables

| Item | Value |
| --- | --- |
| Character record, monster row | name "Skeleton Archer", type 87, variant 0, row 224 (resource 530) |
| Class, approach | 17, approach 3 (keep distance) |
| Hit points, radius, height, weight | 50, 2.0, 2.5, 5.5 |
| Difficulty, alignment, hostile, display in default | 8, 0, 1, 1 |
| Reanimation | flag set; countdown int[3600, 18000] ticks (1 to 5 minutes); death-disappear timer 320 never runs |
| Keep-distance bands | desired min 15.0, desired max 25.0; retreat min 20.0, retreat max 30.0. Desired distance = 15 + 10 f (17.0 to 23.0), retreat distance = a draw in [20, 30]; both redrawn after every normal choice |
| Recovery after an attack | int[30, 120] ticks |
| Search countdown on a notice | int[60, 119] |
| Melee defence | block 0 / 0 / 0; dodge quick / normal / heavy 100 / 100 / 50 (rolls below: 99 %, 99 %, 49 %); defence timer int[30, 120] |
| Missile defence | defend time 120; dodge 90 (89 % per shot; clip 7 or 8 at random); block 0 |
| Stand-off radius | 20.0 (never used: approach-3 enemies are not held back by the attack manager) |
| Notice | awareness 65, timer 50, cone 0.0 (the front half-plane), band 10 to 120, chance 50 rising to 100 (shared table) |

Attack slot 0 (the only slot; slots 1 to 3 unused):

| Item | Value |
| --- | --- |
| Type, band, arc | missile, 0 < d < 75, facing cosine above 0.0 |
| Chance, cooldown | 75 (74 of 100 rolls), int[30, 60] |
| Speed words wind-up / strike / recovery | 0.9 / 0.8 / 1.0 |
| Projectile | straight (flight mode 0), speed 2.0 per tick, range 100.0 (lifetime 50 ticks, 51 moves, at most 102 units), radius 0.1, scatter 0, spawn count 1, visual kind 10, no status, no area |
| Damage, knockback word | 50, 1.0 (the projectile path does not read the knockback word) |
| Counter code / chance | 4 / 100 (cannot fire: see below) |
| Clips | A none, B 34 = clip 9 (15 keys, release bit 0x20 on key 10), C none, D none, E none |

#### Start and wake

48 shipped placements: state 5 (16), 6 (1), 7 (11), 8 (20). Shared rules apply. Specifics:

- Wanderers: the six Ancient Castle Spires archers walk a 1000-unit square (a goal within 500 units of home on each axis, moved toward the archer until it lies on the mesh) and rest 10 ticks with 10 % chance; the Old Mine, Oubliette and Dungeon Ruins ones use squares of 20 to 50 units and never rest. Walking speed 5.92 units per second (clip 6).
- Sentries (11): bound at their disc point when released; three of them (+0xC4 = 1) hold the record facing; eight play fidget id 8 from a timer of 60 to 600 ticks.
- Hidden (20): emerge distances 4.0 to 40.0, cue word 0 on 17 (not hittable while hiding). The Coliseum pair has 40.0 and cue word 1.
- Notice: far, forgiving and facing-dependent (awareness 65 % per 52-tick cycle times a chance of about 50 to 100 %; half that behind its back). A standing hero is noticed at once inside 7 units and has an about 49 % to 84 % chance per roll out to 84 units. An alerted Archer plays the search clip 32 for up to 60 to 119 ticks, then leaves at once (no give-up clip).
- Alert and alarm: clip 18 for 124 ticks, then agro. Groups: Crypt of the Exiled 1 and 4 (four Archers each), Oubliette 3, Black Keep Dungeon 15 and 20.

#### Per-tick order (agro)

As the common-enemy agro machine: recovery countdown; missile defence; melee defence; lost-sight test; normal choice; counter choice; outer sub-state switch (outer 3 = keep distance); route flag and leash; class handler; animation pass; motion pass.

#### State table

| State | Condition | Action id -> clip | Keys / speed | Gate ticks | Exits |
| --- | --- | --- | --- | --- | --- |
| 5, 6, 7, 8, 2 | as the shared rules | wander/patrol clip 6, idle clip 5, hide 11, emerge 10, search 32, alert 18 | | | agro |
| Outer 0 | agro entry | agro idle pick 1 (absent: clip 0; the out-of-agro idle 0 is clip 5) | | | outer 3, inner 4 |
| Outer 3 inner 4 (hold) | 20 <= d <= 25 | 25 -> clip 20 | 36 keys one-shot, then holds the last key | 140 | d > 25 inner 3; d < 15 inner 5; d < 20 inner 6 |
| Inner 3 (approach) | d above desired | 24 -> clip 24 | 11 keys, +0.2502 per tick | | d < desired inner 4; d < 20 inner 6 |
| Inner 5 (back off) | d < 15 (from the hold) | 26 -> clip 12 | -0.2908 per tick | | d > desired inner 4; d < 20 inner 6 |
| Inner 6 (retreat) | 15 <= d < 20 | 24 -> clip 24 (walks away) | +0.2502 per tick, steering to hero minus direction x 32 | | d at least the retreat distance inner 4 |
| Outer 2 slot 0 | chosen | A skipped; B 34 -> clip 9 at 0.9 for the first tick, then 0.8 until the arrow; C, D, E skipped | 15 keys | 67 | end path at 68 |
| Outer 5 / 7 | hero swing | 55 -> clip 33 | 22 keys | 85 | defence exit |
| Outer 6 / 8 | Roberta shot | 53 / 54 -> clip 7 / 8 (89 %), 56 -> clip 23 (never) | 13 / 15 keys | 49 / 57 | defence exit |
| Outer 10 kind 0 | hit | 57 -> clip 1 | 17 keys | 65 | outer 0 |
| Class 16 | killed | 58 -> clip 13, then rise 85 -> clip 14 | 110 / 77 | | agro |

Aim turn: the hold blends 0.25 and then aims at the row turn weight 1.0 in the same tick, so it faces the hero exactly (SPEC-0038 Behavior 44); approach and retreat face their steering direction.

#### Attack chooser

Runs in outer 1, 3, 4 or 11 whenever the recovery countdown is below 1 and the hero is attackable. Slot 0 passes when its cooldown is below 1, 0 < d < 75, the Archer faces within 90 degrees of the hero, and int[1, 100] is below 75 (74 %). The charge rule passes for a missile slot with or without a route. The Archer therefore shoots from as far as 75 units, on the first agro tick in outer 3 that satisfies this (agro tick 2 from the setter), while it is still approaching. On a choice: cooldown int[30, 60], sequencer state 0, desired and retreat distances redrawn.

Sequencer for the missile slot: tick c choice (state 0 goes to state 1 at once, no walking); tick c + 1 state 1 stores the wind-up word 0.9, finds no wind-up clip, requests the strike clip (rate 0.9 for that tick) and enters state 2; ticks c + 2 onward store 0.8 until the release bit; the arrow spawns on the first tick on which the clip's key 10 (release bit 0x20) is current, which is tick c + 51 (tick 50 of the clip); the shot counter reaches 1 (the spawn count), so the next tick is state 3, and from then on nothing replaces the rate, so the clip plays at 1.0; with no follow-through clip state 3 waits for the strike clip's gate (tick c + 68; progress 0.99), state 5 follows (no recovery clip), and the end path runs in the next tick (tick c + 69): outer 0, recovery int[30, 120], cooldown int[30, 60] again.

Counter choice: code 4 needs the sequencer in state 5; the slot it would pick is already the selected slot, and a re-pick of the selected slot draws no roll and changes nothing. The counter never fires (High for the rule, Medium-High for the tick).

#### Movement

Keep distance. The archer stops moving for the whole cast (about 69 ticks) because the missile sequencer never walks. Between casts it holds between 20 and 25 units, approaches when d exceeds 25 and retreats when the hero comes inside 20: the retreat walks the forward clip toward a point 32 units behind it (hero minus direction x 32) and the archer turns to run, not to back off; the back-off (inner 5) is used only below 15 units and is the backward walk (17.45 u/s, faster than the forward walk), which the hold's test order reaches before the retreat test. Retreating or backing off never blocks a shot; it is interrupted by the sequencer.

Movement table (row 224 Skeleton Archer and row 225 Skeleton Wizard, the same gait clips; ground; radius 2.0; ground turn 1.0; Archer desired 15 to 25 (draw 17 to 23), retreat 20 to 30; Wizard desired 15 to 30 (draw 18 to 27), retreat 20 to 40; size selector 1, travel not scaled):

| Mode | Source | Clip (keys) | u/tick | u/s | Turn | Exit |
| --- | --- | --- | --- | --- | --- | --- |
| Wander, patrol, walk home, wait in line, follow, confused | root | 2, 4, 11 or 69 (10), with a small side sway | 0.0987 | 5.92 | steer | arrival 0.5, notice |
| Approach (inner 3) | root | 24 (11) | 0.250 | 15.01 | steer | d below desired, then d below 20 sends it to retreat |
| Hold (inner 4) | none | 25 | 0 | 0 | aim 0.25 then 1.0 (a snap) | d above 25 (Wizard 30), below 15, below 20 |
| Back off (inner 5), below 15 | root, facing the hero | 26 (10) | 0.291 backward | 17.45 | steer, back gait | d above desired |
| Retreat (inner 6), below 20 | root, back to the hero | 24 (11) | 0.250 | 15.01 | steer (an arc of a few ticks to turn round) | d at least the retreat draw |
| Shot or spell (sequencer) | none | slot clips | 0 | 0 | sequencer aim | gate |
| Hit (reaction kind 0) | root | 57 (17) | | back 5.68 over 68 ticks | snap to attacker | gate |
| Heavy hit 62 | root | Archer 62 (21); Wizard 62 = 57 | | Archer back 20.78 over 84 ticks; Wizard as 57 | | gate |
| Dodge 53 / 54 | root | (13) / (15) | | side 5.95 / 6.06 | | gate |
| Lying | root (held) | 58 (56, S1), as the Dumb Skeleton: back 0.82 over 112 ticks, then nothing held | | | kept | Archer (reanimating): the reanimation countdown; Wizard (not reanimating, base state 16): the disappear timer of 320 ticks, then the release |

Speeds: approach and retreat 0.2502 units per tick (15.01 per second), back-off 0.2908 per tick.

#### Arrow

Launch at the archer's emit point 0 (the event word 0x420 carries no emit bit, so point 0) on the release tick, aimed at the hero's position plus (0, 2.5, 0) with scatter 0 (plus 5.0 only if the archer is outside the camera cone). It flies straight at 2.0 units per tick for at most 51 moves. Hero hit test: sphere radius 0.1 against the hero's body cylinder (radius 1.175, height 3.5; SPEC-0029 Behavior 13a). Hit: damage 50 (armour and shield reduce it), no status. Blocked (hero guarding and the shot's direction against the hero's forward): the hero takes a quarter of the damage, minimum 1 (12); element 0 shots are never reflected. A hit releases the slot. The arrow is destroyed by the ground and by static scenery once it is more than 2.0 units from its start. The pool has ten slots shared by all enemies; a full pool drops the shot (the shot counter still counts).

#### Hit and hurt windows (tick 0 = tick the sequencer requests the strike clip)

| Event | Tick |
| --- | --- |
| Release bit seen, arrow spawned | 50 (rate 0.9 for the first tick, 0.8 after) |
| Strike clip gate | 67 |
| End path | 68 |
| Arrow reaches a standing hero at distance d | about 50 + (d - 3) / 2 (5: 51; 20: 58; 40: 68; 74: 85; Distances and ranged reach) |

The Archer can be damaged at every tick except during the melee dodge and missile dodge (outer 5 and 6) and while it plays the dying and rising clips.

#### Defence and counters

- Melee: no block. The first roll (block, chance 0) never passes; the second (dodge) passes with 99 % for quick and normal swings and 49 % for heavy. A dodge plays 55 (clip 33, 85 ticks, no root travel) with the archer standing still and not hittable; defence hold 120, redrawn to int[30, 120] at the exit. Same conditions as the other kinds: hold below 1, within 8.0 units of the hero and inside the hero's front arc, not mid-sequence (sequencer states 0, 1, 5 allowed).
- Missile: Roberta's shots only; 89 % per shot to dodge (clip 7 or 8 at random: 6.07 units right or 6.08 left over 49 or 57 ticks), no block.
- Counter: none (code 4 cannot fire).

#### Hero actions against this enemy

| Hero action | Result |
| --- | --- |
| Tor swing, archer in agro, defence hold below 1, within 8.0 and in front | 99 % dodge for quick or normal swings (49 % heavy): the swing misses, the archer is not hittable for 85 ticks, and it cannot dodge again until its hold (30 to 120) runs out; a hero who swings again in that window hits it |
| Tor swing at an archer in a non-agro state or in its defence hold | hits; first hit doubles damage, ends the alert sequence and enters agro |
| Roberta shot | 89 % dodge |
| Hero standing 25 units away in front | arrows every 99 to 190 ticks |
| Hero hugging the archer | it retreats at 15 units per second, shooting whenever the cooldown allows |
| Hero killing it | corpse for int[3600, 18000] ticks, rises at full HP, fights at once (drops pay again) |

#### Edge cases for implementers

- Slot band 75 with arc 0.0: an Archer that faces away (just after a retreat turn) cannot shoot until it turns.
- A hit during the cast cancels it (hit reaction kind 0), the arrow not yet spawned is lost; the slot cooldown keeps its value.
- The spawn count of 1 means one arrow per cast.
- The map search flag is 0: Archers do not raise skeletons.
- No melee swing exists: an archer in the hero's face never strikes; it retreats and shoots.
- Hidden archers with cue word 0 cannot be shot, hit or targeted before they emerge.

#### What a port most likely gets wrong

1. Making the Archer move while it shoots, or giving it a wind-up clip: the cast is 69 ticks, motionless, with the arrow at tick 50.
2. A 0.8 rate for the whole clip (the arrow tick would be 51 and the gate 71); the rate returns to 1.0 after the arrow.
3. Using the rush band or an attack distance of 10: the band is 0 to 75.
4. Giving it a block: it dodges, with 99 % for normal swings.
5. Letting the counter code 4 fire a second shot.
6. Forgetting the search clip's missing give-up: when the search timer runs out an Archer returns to its default state at once.
7. Treating the notice as a short radius: it reaches 84 units for a standing hero and 210 for a running one.

#### Timeline test vector

Setup: the Archer in agro (alert finished), hero idle 40 units away in front (outer 0 at the agro entry A0; 600 ticks).

| Tick | Event |
| --- | --- |
| A0 + 1 | Outer 0: outer 3, inner 4; d = 40 > 25: inner 3 |
| A0 + 2 | Request clip 24; chooser: d < 75, facing ok, 74 %: expected choice at A0 + 2 to A0 + 3 (the roll repeats each tick) |
| c (about A0 + 2) | Choice; the Archer stops walking; cooldown int[30, 60] |
| c + 1 | Strike clip requested (rate 0.9) |
| c + 51 | Arrow spawned at the release bit; with the hero at 40 units it hits 18 ticks later (c + 69, s + 68) and deals 50 |
| c + 68 | Gate; c + 69 end path: recovery int[30, 120], cooldown int[30, 60] |
| c + 69 + max(recovery, cooldown) | Next choice (mean 77, range 30 to 120), then the same 69 ticks |
| About c + 150 to c + 190 | Second request, arrow 50 ticks later |

In 600 ticks of agro: 4 to 5 arrows (mean spacing about 147 ticks, range 99 to 189). After its first cast the archer closes to 20 to 25 units and holds there unless the hero comes closer.

#### Open questions

1. Resolved: position of emit point 0 at the arrow launch: 0.80 to the side, 1.37 up and 1.65 in front of the archer's feet (posed strike clip on the release tick; Distances and ranged reach).
2. Resolved (Medium-High; proactive (open close-out)): scenery with collision (static meshes, the world's collision primitives and world objects of the cell) and the ground stop an arrow once it is more than 2.0 units from its start, because the ignore-world flag is clear (SPEC-0029 Behavior 9, World collision); the pool tests the hero's body only, so other characters do not stop it. Which props carry collision is level data.
3. The cone: facing cosine strictly above 0.0 for the slot and the half-plane for notice. Default if unresolved (rewrite choice): as stated.

### Skeleton Wizard

**Player's answer (R-0149).** Use heavy swings or shots in the 30 to 60 tick windows between its dodges (swing dodge 74, 49 and 24 in 100 for quick, normal, heavy; shot 49 in 100); it holds 15 to 30 units away and never gives up (SPEC-0038 Behavior 44).

The keep-distance caster and the only enemy that raises the dead. Its normal attack is a slow bolt; its map search casts the raising spell at a skeleton that has died and is waiting to rise (a character whose reanimation countdown is above 0) within 120 units and in sight. Confidence: High for the rows, clips, the map-search steps and the spell contact effect; Medium for what the spell does to targets other than class-17 enemies.

#### Identity and tunables

| Item | Value |
| --- | --- |
| Character record, monster row | name "Skeleton Wizard", type 87, variant 1, row 225 |
| Class, approach | 17, approach 3 (keep distance) |
| Hit points, radius, height, weight | 100, 2.0, 2.5, 5.5 |
| Difficulty, alignment, hostile, display in default | 12, 0, 1, 1 |
| Reanimation | none (flag 0): a dead Wizard stays dead (base state 16, death clip, removal after 320 ticks counted from 0.95 of the death clip, sinking 1 unit per 180 ticks after the first 30) |
| Keep-distance bands | desired min 15.0, desired max 30.0; retreat min 20.0, retreat max 40.0. Desired distance = 15 + 15 f (18.0 to 27.0), retreat distance a draw in [20, 40] |
| Recovery after an attack | int[30, 180] |
| Search countdown on a notice | int[60, 179] |
| Melee defence | block 0 / 0 / 0; dodge quick / normal / heavy 75 / 50 / 25 (74 %, 49 %, 24 %); defence timer int[30, 60] |
| Missile defence | defend time 120; dodge 50 (49 %); block 0 |
| Map search | flag 1, type 2, distance 120.0, chance 75, timer int[120, 300]; greedy hostile range 0 |
| Notice | awareness 100, timer 50, cone 0.0, band 10 to 120, chance 50 rising to 100 |

Attack slots: slot 0 is the bolt; slot 1 is a dead slot (chance 0: its roll can never pass, damage 0, strike clip 39 = clip 9, never used).

| Slot | Band | Chance | Arc | Cooldown | Speed words | Projectile | Damage | Counter | Clips |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| 0 | 0 < d < 25 | 100 (99 of 100) | 0.0 | int[30, 60] | 1.0 / 1.5 / 1.0 | straight, speed 1.0 per tick, range 100.0 (lifetime 100, 101 moves), radius 0.1, scatter 0, count 1, visual kind 10 | 50 (knockback word 4.0, unread) | none | B 34 = clip 9; A, C, D, E none |
| 1 | 0 < d < 75 | 0 | 0.0 | 0 | 1.0 / 2.0 / 1.0 | speed 2.0, range 75 | 0 | none | B 39 = clip 9 |

#### Start and wake

14 shipped placements: state 5 (4: Ancient Castle Spires three with a 100-unit square and a 10-tick rest at 10 %; Crypt of the Exiled one in group 2 with a 64-unit square), state 7 (6: Old Mine 3, Haunted Tombs 2, Crypt of the Exiled 1), state 8 (4: emerge distance 10.0 twice and 40.0 twice; Coliseum pair and Dungeon Ruins, Black Keep Dungeon). Shared rules apply. Awareness 100 means roll A always passes: a Wizard notices a standing hero at once inside 7 units and with about 49 to 84 % per 52-tick cycle out to 84 units (210 for a running hero). It gives up with no give-up clip.

The map search (below) starts independently of notice.

#### Per-tick order

The pre-step of every tick includes the map-search step (step 6 of the pre-step), in every class state. Then as the Archer.

#### State table

| State | Condition | Action id -> clip | Keys / speed | Gate ticks | Exits |
| --- | --- | --- | --- | --- | --- |
| 5, 6, 7, 8, 2 | shared rules | wander clip 6, idle clip 5, search clip 29 (37 keys, 144 ticks), alert 18 (124) | | | agro |
| Outer 3 inner 4 (hold) | 20 <= d <= 30 | 25 -> clip 25 | 22 keys, loop | none | d > 30 inner 3; d < 15 inner 5; d < 20 inner 6 |
| Inner 3, 5, 6 | | 24 / 26 / 24 -> clip 24 / 12 / 24 | +0.2502, -0.2908, +0.2502 per tick | | as the Archer, with the Wizard's bands |
| Outer 2 slot 0 | chosen | A skipped; B 34 -> clip 9 at 1.0 for the first tick, then 1.5 until the bolt; C, D, E skipped | 15 keys | 43 | end path at 44 |
| Class 18 (map-search cast) | map search | 0 (setter), then 137 -> clip 9 | 15 keys at 1.0 | 57 | setter 19 |
| Outer 5 / 7 | hero swing | 55 -> clip 27 | 17 keys | 65 | defence exit |
| Outer 6 / 8 | Roberta shot | 53 / 54 -> clip 7 / 8 (49 %) | 13 / 15 keys | 49 / 57 | defence exit |
| Outer 10 kind 0 | hit | 57 -> clip 1 | 17 keys | 65 | outer 0 |
| Class 16 | not used | death is the plain base-state death: 58 -> clip 13 | 56 keys | 110, then removal | |

#### Attack chooser (the bolt)

As the Archer's with the Wizard's numbers: slot 1 (dead) is tested first and never passes; slot 0 passes with cooldown below 1, 0 < d < 25, facing cosine above 0.0 and int[1, 100] below 100 (99 %). The Wizard therefore fires when the hero is inside 25 units; its hold zone is 20 to 30 units, so it shoots from 20 to 25 units, or while approaching or retreating inside 25. Sequencer: choice tick c, state 0 to 1 at once; c + 1 request (rate 1.0 for that tick); c + 2 onward rate 1.5; bolt spawned at the first tick with key 10 current, tick c + 28 (tick 27 of the clip: the quicker rate makes the release window only 4 ticks); state 3 from then on at rate 1.0; gate at tick 43 of the clip; state 5; end path at tick 44 (c + 45); recovery int[30, 180], cooldown int[30, 60]. The bolt moves 1.0 unit per tick, so a bolt launched at 25 units arrives 22 ticks later (emit point 1.68 in front) and a hero running sideways usually avoids it; it deals 50.

#### Map search and the raising spell (High)

Each tick (pre-step step 6, every class state, 8 and 16 included): the cooldown counts down while positive; the step is skipped while the cooldown is positive, while the Wizard is confused (outer 11) or already in class state 18. Otherwise:

1. Pick a candidate from the list of active region characters (enemies and the other characters of the list; not the heroes): not the Wizard, horizontal distance below 120.0, current HP below maximum, not playing animation 4, and three sight lines to it clear (the line from a point 1.5 above the Wizard's sample point to the candidate's aim point, and two parallel lines offset 1.0 to each side). Up to ten candidates are kept and one is chosen at random. No candidate: nothing else happens this tick.
2. Roll int[1, 100]: the step continues only when the roll is below 75 (74 %). The roll is drawn only on ticks that have a candidate and is repeated every tick until it passes, since the cooldown is set only on success.
3. The recovery countdown must be below 1; otherwise the step ends (and repeats next tick).
4. Success: target stored; class state 18 (the setter requests the idle clip), the cooldown becomes int[120, 300] ticks.

State 18, per tick: if clip 137 (the cast, clip 9) is not current, request it; otherwise, if it has not passed its gate, aim at the target at blend 0.1; on the first tick that the clip's release bit (key 10) is current, fire the spell (once). When the clip passes its gate: recovery countdown int[30, 180] and the setter for agro runs (alarm broadcast to the group, outer 0, all four slot cooldowns 0, attack manager entry). So after any cast the Wizard is fighting the hero, whether or not it had noticed the hero.

Timeline (tick 0 = tick the cast starts): state 18 at tick 0 (the request is made this tick); spell spawned at tick 40; gate at tick 57; setter for agro at tick 57. The spell is a straight projectile (kind 2) from the Wizard's sample point toward the target's position plus (0, 1.5, 0), speed 1.0 per tick, lifetime trunc(distance) + 5 ticks, radius 1.0, no damage. It touches only its target. On contact: the target's reanimation countdown is set to -1, and the spell is released. It is released early by the ground or world contact.

Effect on a corpse of a reanimating skeleton (High): the next tick the skeleton passes to the rise step; it rises if the clear-area test passes (hero at least 5.0 away horizontally, no other character within 5.0), plays clip 14 (77 ticks) and stands up at full HP in agro. Effect on a living skeleton or any other class-17 enemy: none (the countdown is already -1). Effect on other character classes: unknown (Low; see Open questions). Living characters, wounded or not, are never picked: the search accepts only a character whose reanimation countdown is above 0. Each cast puts the Wizard into agro.

Spell lead times: a corpse at 30 units: spell at tick 40, contact at about tick 70, rise starts tick 71, the skeleton stands at about tick 148 and joins the fight.

#### Movement

Keep distance with the Wizard's bands. It never walks while it casts a bolt or a spell. Movement table: the Skeleton Archer's (same gait clips), with the Wizard's bands and corpse rule given there.

#### Hit and hurt windows (tick 0 = request tick)

| Event | Tick |
| --- | --- |
| Bolt spawned (slot 0) | 27 |
| Strike gate / end path | 43 / 44 |
| Map-search spell spawned | 40 |
| Map-search gate and agro | 57 |
| Bolt hits a standing hero 25 units away | 27 + 22 = 49 (the emit point is 1.68 in front and the hero's body radius is 1.175) |

(Tick 0 is the strike request, one tick after the choice tick c; the end path therefore runs at c + 45.)

#### Defence and counters

- Melee: dodge only: 74 % (quick), 49 % (normal), 24 % (heavy), clip 27 (65 ticks, no travel), not hittable while dodging; defence hold 120, redrawn to int[30, 60] at exit.
- Missile: 49 % per Roberta shot, clip 7 or 8 at random.
- Counter: none.
- The take-hit hook cancels a cast in class state 18: the Wizard is not in agro (class state 18), so the hit is doubled and the agro setter runs at once.

#### Hero actions against this enemy

| Hero action | Result |
| --- | --- |
| Kill a skeleton near a living Wizard | the Wizard casts the raising spell at the corpse on the first ticks after its reanimation countdown starts (74 % per tick once the corpse is in sight and the Wizard's search cooldown is spent), raising it 70 to 150 ticks later unless the hero stands within 5 units of the corpse |
| Wound a skeleton and leave it alive | the Wizard casts at it anyway (no effect on it) and goes into agro |
| Hit the Wizard during its cast | cast cancelled, damage doubled (not in agro), agro entered |
| Hero inside 25 units | bolts every 100 to 230 ticks |
| Hero at melee reach | 74 % dodge (quick): needs a second swing after its defence hold |
| Kill the Wizard first | no more raising; the Wizard does not reanimate |

#### Edge cases for implementers

- The map search ignores class state: a hidden (state 8) Wizard casts at a waiting corpse in sight and, when finished, enters agro with its group alerted. A dead (class 16) skeleton never reaches the sight test if the Wizard is behind a wall.
- The roll is drawn every tick while a candidate exists and the cooldown is 0; do not roll once per cooldown.
- The candidate's HP must be below its maximum. A corpse has 0.
- A Wizard casting while its recovery countdown is positive is delayed (the roll is spent).
- The cast sets all slot cooldowns to 0 at its end (the agro setter), so a bolt can follow the spell as soon as the recovery countdown (30 to 180 ticks) expires.
- The map search can start while a bolt cast is in progress (it needs only the recovery countdown below 1, which holds during a cast): the bolt cast is abandoned at once, even before the bolt has spawned, and the Wizard casts the spell instead.
- Slot 1 is a decoy: never choose it.
- The spell's lifetime is the distance plus 5 ticks, and only the chosen target is tested.

#### What a port most likely gets wrong

1. Not implementing the raising spell at all (the Wizard then plays as a weak archer), or making it heal.
2. Running the map search only in agro: it runs in every state.
3. Letting a skeleton rise while the hero stands within 5 units, or with no clear-area test.
4. Choosing slot 1 or giving the bolt the Archer's speed 2.0: it is 1.0, with damage 50.
5. Using a single rate for the bolt clip: the strike runs at 1.5 until the bolt and 1.0 after.
6. Forgetting that the cast ends in the agro setter (cooldowns cleared, group alerted).
7. Giving the Wizard the Archer's notice (awareness 65): the Wizard's is 100.

#### Timeline test vector

Setup A (bolt): Wizard in agro (alert finished), hero idle 40 units away, no waiting corpse in sight.

| Tick | Event |
| --- | --- |
| A0 + 1 | Outer 3, inner 4; d = 40 > 30: inner 3 (approach, 0.2502 per tick) |
| A0 + 62 to A0 + 63 | d < 25: chooser 99 % per tick, choice c |
| c + 1 | Strike clip requested |
| c + 28 | Bolt spawned; hits the hero (just inside 25 units) 22 ticks later (c + 50) |
| c + 45 | End path: recovery int[30, 180], cooldown int[30, 60] |
| next choice | about 106 ticks later on average (30 to 180) |

In 600 ticks of agro: 4 to 5 bolts (mean spacing about 152 ticks).

Setup B (raising): a Dumb Skeleton lies dead (countdown running) 30 units from a sentry Wizard; the hero is 60 units from both and not noticed. Tick 0 = first tick on which the corpse is a candidate (HP 0, in sight, within 120).

| Tick | Event |
| --- | --- |
| 0 to 2 | Roll below 75 (74 % per tick); state 18; map-search cooldown int[120, 300] |
| m + 40 | Spell spawned (m = the casting tick) |
| m + 57 | Gate: recovery int[30, 180], agro setter, group alert |
| about m + 70 | Spell touches the corpse (distance 30 plus the aim offset): countdown -1 |
| m + 71 | Rise requested (clear-area passes) |
| m + 148 | Rise gate: full HP, agro, group broadcast; the skeleton and the Wizard now fight the hero |
| m + 120 to m + 300 | Next possible map-search cast |

#### Open questions

1. Resolved (Medium-High; proactive (open close-out)): the search scans the other characters of the active region list, but the Wizard's search type requires a positive reanimation countdown (SPEC-0038 Behavior 43, type 2) and no candidate filter, and only skeletons that died and wait to rise have such a countdown, so townsfolk and wild pets are never candidates and the spell's contact on them never arises.
2. The map-search cooldown at creation. Default if unresolved (rewrite choice): 0 (the first cast can start on the first tick; needs a runtime check, since no creation write is known).
3. The three sight lines' geometry (a centre line and two lines offset 1.0). Default if unresolved (rewrite choice): as stated.
4. The spell's start point. Default if unresolved (rewrite choice): 1.5 above the Wizard's position.
5. Resolved: position of emit point 0 for the bolt: 0.83 to the side, 1.64 up and 1.68 in front of the feet on the release tick (Distances and ranged reach).


## Minion Corps B: casters and heavies

Four kinds: the Minion Wizard (monster type 83, rows 211 to 216), the Minion Magician (type 81, rows 197 to 202), the Minion Artillery (type 77, rows 162 to 167) and the Dark Knight (type 64 variant 0, row 139). All are class-17 common enemies and follow SPEC-0038 (think loop, pre-step, notice, attack choosers, sequencer, counter choice, take-hit hook, death hook). Two warnings first. The six Wizard rows and the six Magician rows are not element variants of one enemy: each row has its own attack slots, and the Wizard rows differ in what they cast (a summoner, a fire-patch lobber, a homing volley, a stunning guard, a ghost, a curse-bolt healer), so the Wizard section has a per-row guide. The Artillery and Dark Knight rows do share one design per kind and differ by element only. Confidence: High for row values, clips, attack arithmetic, gate ticks and the hidden rule; Medium-High for the caster map-search spells and the start-state behaviours; Medium for weapon contact geometry, the Artillery's lunge reach and the flyer reaction wait. All times are ticks (60 per second). "c" is the tick on which an attack is chosen, "s" the tick on which the sequencer requests its first clip (s = c + 1 for a slot whose first phase needs no approach), and "s + j" means j ticks after a clip request (the tick a gate or an event window is seen by the behaviour step, SPEC-0038 Behavior 13).

### Placements and start states (in play, all four kinds)

Counts are placements the player can meet (confirmed or plausibly loaded); cut placements (test block, never-loaded regions) are left out: Wizard 10 (four each of rows 213 and 216 in one Illusen's Glade region set, one of row 211 in a Root Cellar region and one in the test block), Artillery 1, Magician 1, Dark Knight 2. The Wizard, Magician and Artillery are never summon-reserved; their records are streamed in by distance like any other enemy.

| Kind | Placements | State 0 | State 5 (wander) | State 6 (patrol) | State 7 (sentry) | State 8 (hidden) |
|---|---|---|---|---|---|---|
| Wizard | 56 (row 211: 14, 212: 3, 213: 14, 214: 3, 215: 3, 216: 19) | 0 | 10 | 4 | 11 | 31 |
| Magician | 49 (row 197: 7, 198: 7, 199: 13, 200: 4, 201: 6, 202: 12) | 2 | 8 | 2 | 21 | 16 |
| Artillery | 28 (row 162: 8, 163: 11, 164: 1, 165: 5, 166: 2, 167: 1) | 1 | 4 | 0 | 13 | 10 |
| Dark Knight | 42 (row 139) | 1 | 18 | 17 | 6 | 0 |

Main places (default state in brackets). Wizard: Meridell Village 11 (8), Heroes Rest 9 (7), Illusen's Glade 8 (5: 4, 6: 2, 7: 1, 8: 1), Meridell Plains 6 (5: 3, 6: 1, 8: 2), Coliseum-area wave regions 5 (8), Shadowglen Woods 4, Money District 3 (8), Park District 3 (8), Arena District 2 (8). Magician: Market Town 8 (7: 7, 8: 1), Fyora's Palace 8 (7: 5, 8: 2, 0: 1), Arena District 6 (8: 5, 6: 1), Fyora's Palace Gardens 5 (5), Black Keep Grounds 4 (7: 3, 8: 1), Fyora's Library 4 (7), Money District 3 (8), Western Brightvale 3, Meridell Outskirts 3, Coliseum-area wave regions 2 (8). Artillery: Bazaar District 6 (8), Meridell Outskirts 5 (5: 2, 7: 1, 8: 2), Steppe Plateau 4 (7), Heroes Rest 4 (7), Meridell Plains 3 (7: 2, 8: 1), Farm Road 1 (8), Fyora's Palace 1 (0). Dark Knight: one pack of 12 (5, wander over a 90-unit square), Market Town 11 (6: 10, 7: 1), Black Keep Grounds 10 (6: 5, 7: 3, 5: 2), Meridell Village 5 (5: 4, 0: 1), Crossroads 3 (7: 2, 6: 1).

Spawn record values that drive the states (record offsets, SPEC-0006; meanings in the field rules of the Minion Corps A and Road and Ridge Brigands sections):
- State 5 wander: side R at +0xBC; pause range +0xC0 to +0xC4, percent +0xC8. Wizard: R 50 on 5 records, 20, 30, 100 and 0 (whole navmesh) on one each, one with pause 120 to 480 at 15 percent, the rest never pause. Magician: R 10 with pause 90 to 150 at 50 percent on 5, R 100 on 2, R 20 on 1. Artillery: R 50 (2), 100 (1), 0 (1), no pauses. Dark Knight: R 90 on 12 (no pause), R 15 with pause 120 to 480 at 25 percent (2), R 40 with 0 to 120 at 50 and 60 to 300 at 25, R 20 with 120 to 240 at 25, R 20 with no pause.
- State 6 patrol: path index +0xB8, loop mode +0xBC (0 loop, other back-and-forth), point pause +0xC4 to +0xC8. Wizard: path 4 back-and-forth with no pause (2), path 11 pause 120 to 480, path 7 pause 60 to 150. Magician: path 1 (no pause) and path 0 pause 60 to 240. Dark Knight: 17 different paths (0, 2 to 5, 9 to 16 and others), pauses 50 to 300 ticks at the points, loop on about half and back-and-forth on the rest.
- State 7 sentry: +0xC4 zero on every Wizard and Magician record (they fidget at home), nonzero on 6 of 13 Artillery records and 2 of 6 Dark Knight records (they hold the placed facing).
- State 8 hidden: emerge distance +0xC0, cue word +0xC4 (camera cue only; these kinds are hidden by Display anyway), selector +0xC8 always 0. Wizard: 100 (11), 10 (6), 40 (5, cue 1), 15 (5), 30 (2), 20 (2), one 10 and one 15 with cue 1. Magician: 20 (5), 10 (3), 40 (4, two with cue 1), 15 (2), 30 (1), 1 (1). Artillery: 20 (5), 100 (2), 200 (1), 30 (1), 0 (1, woken only by its alarm group).
- Alarm groups (+0xB7, 255 = none): Wizard 22 of 56 records use groups 1 to 10; Magician 25 of 49; Artillery 15 of 28; Dark Knight 17 of 42. Drop ordinal (+0xB0) is -1 (no placement drop) on 28 of 56 Wizards, 13 of 49 Magicians, 13 of 28 Artillery and 14 of 42 Dark Knights. None of the four rows steals.

### Shared rules for the hidden-start kinds (Wizard, Magician, Artillery)

These three rows have Display in default 0 (the Dark Knight has 1). The hidden rule, the start through state 23, the false start (look clip, lose-interest clip, back to the default state), the decloak through state 21 and the alarm-group behaviour are exactly those of the Minion Corps A section ("Shared rules for the four minion kinds"); only the numbers differ and are given per kind below. The points a port must keep:

1. A hidden enemy (class state equal to its default-state word, or state 21 until its decloak clip 82 starts) is not drawn, not hittable, not auto-targetable and it does not cast map-search spells. It still runs the notice check every tick of its awareness cycle.
2. A pass of the notice check in state 5, 6 or 7 enters state 21 (not state 2) and the state-21 handler repeats the check on the same tick with fresh rolls. A second pass requests the decloak clip 82 and the enemy goes to agro when that clip passes its gate; a failed second pass starts the look clip (ids 147 for flyers, 16 on the ground) and then the lose-interest clip (148, 17), after which the enemy runs the setter with 0 (state 23, clip 81, then the default state). A pass during the look or lose-interest clip still leads to clip 82.
3. State 8 enemies wake only by the emerge distance (scaled by F1 of the hero's action state: 0.7 standing, 0.75 slow walk, 1.5 walk, 1.75 run) or by their alarm group, play the emerge clip (flyers 165, ground 10) and go to agro when it passes its gate. Emerge distance 0 never wakes by distance.
4. A hit while 82, 81, the emerge clip or any non-hidden non-agro state plays is a normal hit: doubled damage, agro, hit reaction. A hidden enemy cannot be hit.
5. Every exit from agro (leash 181 ticks without a route while the walk waypoint is 15.0 or more from the hero, lost-sight search above 120 ticks, victory pose) passes through state 23 (clip 81, visible and hittable) and then the default state.
6. Group alarm: a group member entering agro (including re-entering it after a map-search cast) broadcasts; members in states 0, 3, 5, 6, 7 and 14 go to state 21 within int[15, 45] ticks, members in state 8 emerge.

Per-kind numbers for the notice and decloak chain (rows within a kind differ only where stated; "gate" = the tick j after the request at which progress reaches 0.99, SPEC-0005):

| Item | Wizard | Magician | Artillery |
|---|---|---|---|
| Awareness (roll A at most) | 100 | 90 | 75 |
| Awareness timer, cycle length | 50, 52 ticks | 60, 62 | 60, 62 |
| View cosine (half the chance outside) | 0.92388 | 0.707 | 0.92388 |
| Notice range near, far (unscaled) | 10, 20 | 3.5, 50 | 3.5, 200 |
| Chance at near end, at far end | 50, 100 (row 212: 90) | 70, 50 | 80, 30 |
| Sees through walls (always has line of sight) | no | yes | no |
| Standing hero (F1 0.7): always noticed inside, never beyond | 7, 14 | 2.45, 35 | 2.45, 140 |
| Look clip, gate | 147: clip 28, 65 keys, 256 | 147: clip 22, 60 keys, 236 | 16: clip 26, 45 keys, 176 |
| Lose-interest clip, gate | 148: clip 26, 18 keys, 69 | 148: clip 20, 30 keys, 116 | 17: clip 24, 38 keys, 148 |
| Decloak clip 82, gate | clip 6, 35 keys, 136 | clip 6, 23 keys, 89 | clip 2, 24 keys, 93 |
| Emerge clip (state 8), gate | 165: clip 13, 30 keys, 116 | 165: clip 6, 89 | 10: clip 2, 93 |
| State 23 clip 81, gate | clip 30, 23 keys, 89 | clip 30, 17 keys, 65 | clip 14, 10 keys, 37 |
| Alarm clip 18 (149 flyers) | absent | clip 6 (replaced next tick) | clip 2 (replaced next tick) |
| First agro choice after the decloak request | request + 136 + 2 | + 89 + 2 | + 93 + 2 |

The chance of one notice check passing is P(roll A) x P(roll B below the chance), where the chance is interpolated linearly between the near and far values over the unscaled range, and halved outside the view cone: for a standing hero in the cone, Wizard at 10 units 0.49, at 12 units 0.59, at 14 units 0.69, inside 7 units 1.0; Magician at 10 units 0.9 x 0.67 = 0.60, at 30 units 0.9 x 0.58 = 0.52, inside 2.45 units 0.9; Artillery at 20 units 0.75 x 0.75 = 0.56, at 60 units 0.75 x 0.65 = 0.49, at 100 units 0.75 x 0.55 = 0.41, inside 2.45 units 0.75. A walking hero (F1 1.5) or running hero (1.75) widens the far edge to 1.5 and 1.75 times the unscaled far range.

#### Flyer rules (Wizard and Magician: Can fly 2, always airborne)

- Gait ids for flyers: wander 98, patrol 99, named-target walk 100, sentry stand 144 and fidget 145, patrol fidget 142, keep-distance forward 107, back 108, hold 156, melee dodge 140, missile dodge 138 or 139, hit reaction 152 (kind 6) or 76 (kind 7), death 153, victory pose 163. A flyer moves by the root travel of its clip plus the row speed: 2 x flying idle velocity outside agro and 2 x flying agro velocity in agro (Wizard 1.8 and 1.8, Magician 1.0 and 2.0 units per second), along the class axis of the animation being played.
- Height (corrected, movement pass): both kinds are Can Fly 2, so the goal height, set whenever a walk target is set, is the hero's own height, lowered to the ground under the goal plus the row's maximum fly height (+0xB4, 20.0 for both kinds) when it is higher than that; there is no 0.2 x distance term and no 2.0 floor (those belong to Can Fly 1 rows). The home point's height replaces it only while the enemy sits in state 8's waiting sub-state. The height moves at most 0.1 per tick toward the goal and a hold sub-state keeps the last goal. Can Fly 2 rows keep gravity (SPEC-0040 Behavior 14D item 7), so a Wizard or Magician fights at about the hero's foot height, within a few tenths of a unit of the ground when both stand on the same floor, and rises only toward a hero standing higher (at 6 units per second, against gravity). The earlier text (hovering 3 to 8 units above the hero) applied the Can Fly 1 rule and was wrong.
- Attack-slot trigger specials: the airborne-only special (1) is a can-fly-1 test, so a Can fly 2 row never passes it, and the grounded-only special (2) always passes for a Can fly 2 row. A Wizard or Magician slot carrying special 1 would stay closed for ever, and one carrying special 2 would be open on every tick.
- The flyer test is true for both kinds, so death plays 153 (not 58), the death hook never grounds them (that applies to Can fly 1 only) and the corpse does not sink.

#### Map-search spells (Magician rows 197 to 199 and 201; Wizard rows 213 and 216)

These rows carry a map-search flag and cast support spells on other characters while they are awake. The pre-step runs the search every tick for an enemy whose row has the flag, in every class state except: state 11 outer (confused), state 18 itself, a Display-0 enemy standing in its default state (hidden), and any tick on which its own search cooldown is positive. Each cast:

1. Candidate pick. The enemy scans the list of active region characters (not itself) and keeps the ones that pass the type's filters: within the row's search distance of the caster (the same distance lane as the notice check), not currently playing the animation id named for the type, not hidden, not script-held, not the active petpet or a petpet-keyed character, not one of a short list of excluded character keys (the minion kinds' own keys 2831 to 2837, 2851 to 2857, 2862, 2863, 2871, 2874 to 2877 and three other keys) and, where the type says so, wounded (current HP below maximum). Their sight line from the caster must not be blocked. One of at most ten eligible characters is picked at random. No allegiance test exists: enemies, townsfolk and other creatures are all candidates. The hero is never a candidate.
2. Gate. A roll int[1, 100] must be below the row's map-search chance (Magician 25 percent per tick, Wizard 213 100 percent; Wizard 216 has no roll but needs the caster itself wounded), and the caster's recovery countdown must be 0 (it is 0 while an attack runs, so a cast can interrupt a wind-up).
3. Cast. The enemy enters class state 18 and draws its search cooldown (Magician int[120, 300], Wizard 213 int[300, 600], Wizard 216 int[120, 300]) at once. State 18 requests animation 137 and plays it to its gate (Magician: clip 8, 12 keys, gate 45, event bit 0x20 first seen at j = 4, so the spell leaves 4 ticks after the request; Wizard 213: clip 8, 86 keys at speed byte 1, gate 170, event at j = 110; Wizard 216: clip 12, 36 keys at speed byte 1, gate 71, event at j = 22; the other Wizard rows map 137 to a 65-key clip with no event, which their missing map-search flag never plays), aims at the target with weight 0.1, fires its spell once, on the first tick the clip frame carries event bit 0x20, and when the clip passes its gate draws a recovery countdown and runs the setter with 19 (agro). That setter clears the recovery countdown and all four attack cooldowns, restarts the outer sub-state at 0, and re-broadcasts the alarm group. So after every cast the caster can start an attack on the next tick, and a caster that cast from a non-agro state (state 21, 23, 2, 3) enters agro without having seen the hero.
4. Spell. A projectile (kind by type, speed 1.0 unit per tick, lifetime distance + 5 ticks, damage 0, radius 1.0) flies from the caster's launch point to the target's position + 1.5 up and touches only that target. Contact effects, by the type of the row: type 3 (row 197) status 4 strength for 60 s (enemy damage times 1.5); type 4 (row 198) status 11 (poison attack) for 30 s; type 5 (row 201) status 5 speed for 60 s; type 6 (row 199) target HP set to its maximum (candidates must be wounded); type 8 (Wizard 213) the target's variant becomes a random whole number 0 to 5, its monster row is chosen again and a colour preset is loaded (for a minion the element and status change, and Medium: its HP reloads from the new row); type 7 (Wizard 216) see that row. Candidate animation filters: type 3 skips characters playing id 4, type 4 id 11, type 5 id 5, type 7 id 0; types 6 and 8 have none.
5. A caster that is hit while in state 18 takes doubled damage and the hit sends it to agro at once, ending the cast (state 18 is not state 19, so the take-hit hook treats it like any non-agro state).

The Magician rows 200 and 202 and the Wizard rows 211, 212, 214 and 215 have no map-search flag.

### Minion Wizard (rows 211 to 216)

**Player's answer (R-0149).** It never closes in and never gives up its hold of 25 to 60 units, so beat it with shots or swings timed to its dodge: each dodge (57 ticks melee, 61 missile) is followed by 0 to 60 open ticks in which everything lands, and its melee dodge is off while it stands still in its strike states (SPEC-0038 Behavior 44).

Six rows, one rig (monster type 83, variants 0 to 5, name "Minion Wizard"), one body of rules, six different weapons. Records: 56 in play (row 211 14, 212 3, 213 14, 214 3, 215 3, 216 19). Every row table in play has the same 166 entries for the flyer ids used here; the rows differ in which attack ids are mapped (below).

#### Identity and tunables (all six rows unless a row is named)

| Item | Value |
|---|---|
| Class, approach | Class 17; approach 3 (keep distance); Can fly 2 (always flying) |
| Hit points, body | 200; radius 1.75, height 6.0, weight 5.5; hostile; Display in default 0 (invisible kind); attack-manager weight 12 but approach 3 never waits in line |
| Element T and name | Row 211 T 1 (fire), 212 T 2, 213 T 3, 214 T 4, 215 T 5, 216 T 6. T sets the damage element and the hero status of the row's own statuses |
| Flight | Idle velocity 0.9, agro velocity 0.9 (1.8 units per second on top of root travel); maximum fly height 20.0; hold aim weight (flying) 0.7 |
| Notice | Awareness 100, timer 50 (cycle 52 ticks), view cosine 0.92388, range 10 to 20, chance 50 at 10 and 100 at 20 (row 212: 90 at 20), no line-of-sight flag, search timer 60 to 240 (unused: state 21 does not draw it) |
| Agro bands | Hold between 25.0 and 60.0; desired distance 25 + 35 f, f from 0.20 to 0.80 (32.0 to 53.0), redrawn after every normal attack choice; retreat when below 20.0, retreat distance drawn in 20.0 to 25.0 |
| Recovery after an attack | int[30, 60] ticks |
| Missile defence | Hold 60 ticks, dodge chance 60 (59 of 100), block 0 (row 214: block 50, 50 of 100 on the later roll) |
| Melee defence | Dodge 90 for quick, normal and heavy swings (89 of 100); block 0 (row 214: 100, 85, 75 for quick, normal, heavy, the roll passing at or below); defence timer after a defence int[0, 60]; row 214 Blocking Angle 0.9914 (others 0.92388, never in a block state) |
| Hit reaction | Flying hit value 1: kind 7 (animation 76; the rig has no clip for 76 or 74, so clip 0 plays) |
| Death | Flyer: animation 153 = clip 17 (70 keys at speed byte 1, gate 138); the corpse is gone 180 ticks after the clip passes progress 0.95 (about tick 132 after death), so about 313 ticks; the corpse does not sink |
| Other | No steal, no knockback on the hero (projectile knockback is not applied), weapon volumes of radius 1.0 and height 1.0 (never used: no melee slot). Footstep shake and rumble 0 |

#### The six rows (attack slots; slots are tested from the highest slot down, so slot 1 first)

All slots are missile slots (type 1), spawn count 1 or more, orient flag 0, speed words 1.0, and none has a counter except where stated. Slot ids: slot 0 uses animation ids 33 to 37, slot 1 uses 38 to 42; only the strike id (B) is mapped on every row (no wind-up, follow-through, blocked or recovery clips: those phases are skipped in the same tick). Strike clips are one-shot at speed byte 1 (0.5 keys per tick, 2 ticks per key). "Release j" is the tick j after the B request on which the spawn event (event bit 0x20 of the clip frame) is seen; one projectile leaves per separate event window, up to the slot's spawn count.

| Row | Slot | Role | Band (strict), chance, arc | Cooldown | B clip, gate, release j | Projectile | Status on hero |
|---|---|---|---|---|---|---|---|
| 211 | 1 | Fire patches | 10 to 60, 75, 0.0 | int[300, 480] | 39 = clip 7 (86 keys), 170; 110, 114, 118, 122 | 4 lobbed shots (flight mode 1), damage 5, launch speed 4.0 per tick at 60 units falling to 0.04 at 10 units, scatter U[0, 10] (+5 outside the camera cone), landing area radius 5.0 for 600 ticks | 7 fire, 100 percent per pulse, 30 to 60 s |
| 211 | 0 | Summon Grunts | 10 to 60, 50, 0.0 | int[180, 300] | 34 = clip 10 (41 keys), 81; 50 | Lobbed summon of key 2844 (Minion Grunt row 183), launch speed 1.0 per tick at the hero's position, live cap 4 | none |
| 212 | 1 | Poison orb | 0 to 100, 100, 0.0 | int[180, 300] | 39 = clip 11 (96 keys), 190; 156 | 1 straight shot, damage 1, speed 0.06 per tick (3.6 per second), radius 1.5, lifetime 1666 ticks | 2 poison, 100 percent, 30 to 60 s |
| 212 | 0 | Bolt | 0 to 100, 100, 0.9914 | int[300, 600] | 34 = clip 8 (86 keys), 170; 110 | 1 straight shot, damage 5, speed 3.0, radius 0.5, lifetime 33 | none (the slot's mote-drain flag is not read by a missile) |
| 213 | 1 | Homing volley | 0 to 100, 100, 0.0 | int[30, 300] | 39 = clip 12 (36 keys), 71; 22, 26, 30, 34, 38 | 5 homing shots (flight mode 2) from the second emit point, damage 10, launch speed 1.0, radius 0.5, spin 5.0, lifetime 200 | none |
| 214 | 1 | Stun bolt | 0 to 100, 100, 0.0 | int[180, 300] | 39 = clip 8 (86 keys), 170; 110 | 1 straight shot, damage 10, speed 2.0, radius 0.5, lifetime 50 | 0 stun, 100 percent, int[380, 460] / 60 = 6 or 7 s |
| 215 | 1 | Slow patches (ghost) | 10 to 60, 75, 0.0 | int[300, 480] | 39 = clip 7 (86 keys), 170; 110, 114, 118, 122 | 4 lobbed shots, damage 2, launch speed 0.5 per tick at 60 units falling to 0.02 at 10, scatter U[0, 10], area radius 5.0 for 600 ticks | 3 slow, 100 percent per pulse, 30 to 60 s |
| 216 | 1 | Curse bolt | 0 to 100, 100, 0.0 | int[30, 300] | 39 = clip 8 (86 keys), 170; 110 | 1 straight shot, damage 10, speed 2.0, radius 0.5, lifetime 50 | 1 curse, 100 percent, 30 to 60 s |

Row 213 also carries map-search type 8 (polymorph; distance 200, chance 100, cooldown int[300, 600]) and row 216 type 7 (below), with the support-spell rules of the shared section. Row 214 has the counter: slot 1 trigger 6 (own hit reaction) chance 75. The slot cooldowns are drawn at the choice and again at the end path (restarting when the attack ends).

Row 216's map-search spell (type 7): it runs only while the caster itself is wounded (current HP below maximum) and needs no chance roll. At the build tick the caster deals its own missing HP (maximum minus current) to the picked target as plain damage (High: the damage is the shared plain HP subtraction, not the take-hit hook, so there is no doubling, no alert and no agro change; when it reaches 0 the death hook runs), and the spell then sets the target's HP to its maximum on contact. The caster's own HP is not changed. A target whose HP is at or below the caster's missing HP dies at the build tick (death hook, drops). The candidate filter skips characters playing animation id 0.

Hero-status roll for projectiles: a random integer 0 to 100 at most the chance, so 100 percent means every hit applies the status (status 0, stun, is skipped while a stun is already active). A projectile deals damage first, then the status; the area shots (rows 211 and 215) roll the status on every pulse.

#### Animation ids and clips (flyer ids; the same for every row except the attack and cast ids above)

| Id | Use | Clip |
|---|---|---|
| 0, 97 | idle pick | clip 0 (30 keys, loops); fidget ids 115 to 119 absent |
| 98, 99, 100 | wander, patrol, named-target walk | clip 25 (30 keys, loops, 2.51 units per second root travel) |
| 107 | keep-distance forward, retreat | clip 1 (10 keys at speed byte 1, loops, root travel 45.0 per second; with the row speed 46.8 per second) |
| 108 | back off | clip 2 (10 keys, loops, root -15.0 per second; with the row speed -16.8) |
| 156 | hold | clip 4 (30 keys, loops) |
| 144, 145 | sentry stand, sentry fidget | clip 24 (30 keys, loops), clip 20 (60 keys, gate 236) |
| 142 | patrol hold fidget | clip 23 (60 keys, gate 236) |
| 138, 139 | missile dodge | clip 19 (31 keys at speed byte 1, gate 61, sideways 12.1 per second) |
| 140 | melee dodge | clip 18 (15 keys, gate 57, backward 13.9 per second) |
| 153 (58) | death | clip 17 (70 keys at speed byte 1, gate 138, backward 6.4 per second) |
| 147, 148 | look, lose interest | clips 28 (65 keys, gate 256) and 26 (18 keys, gate 69) |
| 82, 165 | decloak, emerge | clip 6 (35 keys, gate 136), clip 13 (30 keys, gate 116) |
| 81 | state 23 | clip 30 (23 keys, gate 89) |
| 163 | victory pose | clip 15 (40 keys, gate 156) |
| 76, 74, 149, 56, 150, 151 | hit kind 7, its follow-up, alert, block | absent (row 214 maps 56 and 150/151 to clip 14, a 1-key pose) |

#### Start, wake, per-tick order

Start and wake are the shared rules above with the Wizard column of the notice table. A Wizard in states 5, 6 and 7 is hidden; the first thing the player sees is clip 82 (35 keys, a 136-tick appear). Agro runs the standard order (SPEC-0038 Behavior 44): recovery countdown, missile defence, melee defence, lost-sight test, normal attack choice, counter choice, distance-hide gate (off), outer switch, leash. The outer switch starts at outer 0 (set-up) and moves to outer 3 (keep distance). The map search of the shared section also runs in the pre-step.

#### State table (agro)

| State | Action id and clip | Keys | Gate and exits |
|---|---|---|---|
| Set-up (outer 0) | agro idle pick 141 (Can fly 2; absent: clip 0, the same 30-key idle clip) | 30 | one tick, then outer 3 inner 4 |
| Hold (inner 4) | 156 -> clip 4 | 30, loops | d > 60 gives approach; d < 25 gives back off |
| Approach (inner 3) | 107 -> clip 1 | 10, loops | 46.8 per second (0.78 per tick) toward the hero; ends when d < desired (32 to 53) |
| Back off (inner 5) | 108 -> clip 2 | 10, loops | 16.8 per second (0.28 per tick) toward the point 60 from the hero; ends when d > desired; d < 20 gives retreat |
| Retreat (inner 6) | 107 -> clip 1 | 10, loops | 46.8 per second away (steers to hero - dir x 27, facing away); ends when d is at least the drawn retreat distance (20 to 25) |
| Attack (outer 2) | the row's B id | per row table | missile slot: sequencer state 0 goes to state 1 at once, state 1 requests B (no wind-up) and enters state 2 on the same tick (s = c + 1); shots at s + release j; the shot counter reaches the spawn count or B passes its gate, then state 3 (no C), state 5 (no E), end path at gate + 1 |
| Melee dodge (outer 5) | 140 -> clip 18 | 15 | gate 57, then the defence exit (outer 0, base state 1, hold int[0, 60]) |
| Missile dodge (outer 6) | 138 or 139 -> clip 19 | 31 | gate 61, then the defence exit |
| Melee block (outer 7, row 214 only) | 56 -> clip 14, base state 20 | 1 | while the hero swings; then the defence exit |
| Missile block (outer 8, row 214 only) | 56 -> clip 14 | 1 | while a hero shot is live |
| Hit reaction kind 7 (outer 10) | 76 -> clip 0 | 30 | see Hit and hurt windows |
| Cast (class state 18) | 137 -> clip per row | per row | map-search rows only; setter 19 at the gate |
| Victory pose (state 10) | 163 -> clip 15 | 40 | gate 156, then the default state |
| Dead | 153 -> clip 17 | 70 | plays once |

#### Attack chooser

Each agro tick while the hero is attackable, the recovery countdown is below 1 and the outer sub-state is 1, 3, 4 or 11: slot 1 first, then slot 0 where the row has it (rows 211 and 212). A slot passes when its cooldown is below 1, d is inside its band (strict), the facing dot is above its arc (0.0 means in front of the Wizard; row 212's bolt needs 0.9914, 7.5 degrees) and the roll int[1, 100] is below the chance (74 of 100 for chance 75, 49 of 100 for 50, 99 of 100 for 100). A missile slot needs no route. Row 211: fire first, so a tick with both off cooldown starts fire 74 percent, summon 13 percent (0.26 x 0.49); with fire on cooldown the summon 49 percent per tick. The summon slot also needs fewer than 4 live Grunts spawned by this Wizard. Rows 213 and 216 have cooldowns of only 30 to 300, so they attack again as soon as their recovery (30 to 60) and cooldown (re-drawn at the end path) allow. After a choice the cooldown is drawn and the desired distance is redrawn. The sequencer cannot aim at weight 1.0 (orient flag 0), but state 2 aims at the hero every tick with the row's hold aim weight 0.7; projectile directions come from the emit point to the hero's position at the release tick anyway.

#### Movement

Keep-distance archetype with the flyer gaits (shared section): approach at 46.8 per second when d is above 60, back off at 16.8 per second when below 25, retreat at 46.8 per second when below 20, hold in between, so a Wizard never closes inside 25 on its own and out-runs any hero. It flies level with the hero's feet (Can Fly 2 height rule, corrected in the shared section; it does not hover 5 to 12 units above the hero). A Wizard that reaches the leash (no route for 181 ticks) goes home; a hero in the open is fought indefinitely.

Movement table (rows 211 to 216; row flying velocity 0.9 idle and agro, so the row term is 1.8 per second; Can Fly 2; radius 1.75):

| Mode | Source | Clip (keys) | u/tick | u/s | Turn | Exit |
| --- | --- | --- | --- | --- | --- | --- |
| Wander, patrol, walk home | root + row | 98 or 99 (30) | 0.0718 | 4.31 (2.51 + 1.8) | steer | arrival 0.5, notice |
| Approach (inner 3), d above 60 | root + row | 107 (10, S1) | 0.780 | 46.82 (45.02 + 1.8) | steer | d below the drawn desired 32 to 53 |
| Hold (inner 4), 25 to 60 | none | 156 | 0 | 0 | aim 0.25 then 0.7 (effective 0.775) | d above 60, below 25, below 20 |
| Back off (inner 5), below 25 | root + row, facing the hero | 108 (10) | 0.280 backward | 16.81 | steer, back gait | d above desired |
| Retreat (inner 6), below 20 | root + row, back to the hero | 107 (10, S1) | 0.780 | 46.82 | steer | d at least the retreat draw 20 to 25 |
| Cast (sequencer) | root | slot clips; row 211's have no root, the 86-key cast clip of rows 212, 214 and 216 creeps 9.1 forward over its 172 ticks | 0 to 0.053 | 0 to 3.2 | aim (sequencer) | gate |
| Hit (reaction kind 7, flying-hit value 1) | none | 76 missing: clip 0, no root | 0 | 0 | snap to attacker | ground gap at most 2.0, then outer 0 |
| Dodge 138 / 139, 140 | root | 138/139 (31, S1), 140 (15) | | side 12.48 over 62 ticks; back 13.73 over 60 | as dodge | gate |
| Corpse | root (held) | 153 = clip 17 (70, S1), constant -0.214 per key | 0.107 during the clip, then 0.214 held | 6.4, then 12.8 | facing kept | disappear timer 180 ticks; a flying corpse does not sink |

The Wizard has no ground gaits (ids 2, 4 and 21 to 32 are missing), so every ground request plays clip 0 (no root). Its corpse drifts backward 14.98 units during the death clip and then slides at the doubled last key until the navmesh border or its release (SPEC-0038 Behavior 51 item 15).

#### Hit and hurt windows

- Shots leave on the release ticks of the table; the first contact on a hero standing 30 units away is 10 ticks after release for the 2.0 bolts of rows 214 and 216 and 6 ticks for the 3.0 bolt of row 212 (their emit point is 8.1 in front: the clip carries the Wizard 3.98 forward), 24 ticks for the first homing shot of row 213, or 407 ticks (the poison orb, 0.06 per tick, not a practical threat for a moving hero); per-row figures are in Distances and ranged reach. Lobbed patches (rows 211 and 215) fly the ballistic arc of SPEC-0038 Behavior 13: for a hero 30 units away and a launch speed ramp u = 1 - (d - 10) / 50 = 0.6, speed 4.0 + 0.36 x (0.04 - 4.0) = 2.57 per tick (row 215: 0.5 + 0.36 x (0.02 - 0.5) = 0.33), flight 11.7 ticks (row 215: 91 ticks), and the landing point is the aim point (hero position at the release tick + scatter). Correction (reach pass 2): a landed lob never pulses. Its ground test reloads the pulse timer to 60 on every tick while it lies on or under the floor, so the timer never reaches 0 (SPEC-0029 Behavior 9, Area mode); the 600-tick area only keeps the pool slot busy. Only a direct hit of the lob itself hurts (5 or 2 and the status), and with the row's 0.1 radius (row 215: 0) the fast row-211 lob passes through a hero standing 25 or more units away.
- The Wizard can be hit in every agro sub-state except its dodges (outer 5 and 6), while dead and while hidden. During the cast (state 18) and the decloak (82, 81) a hit counts as a hit from a non-agro state: damage doubled, agro at once.
- Ghost row 215: hits are absorbed during the ghost phase (shared ghost rule below).

#### Defence and counters

Melee defence is rolled once per hero swing (latch), only when the defence hold is below 1, the Wizard is not in a hit reaction, not mid-attack except sequencer states 0, 1 and 5, the hero is within 8.0 and in front within 60 degrees: r1 at most the block chance (row 214: 100, 85, 75 for quick, normal, heavy swings) gives a block; otherwise r2 below 90 gives a dodge (clip 18, 57 ticks, then the hold int[0, 60] ticks, so a second dodge can follow after at most 60 ticks). Quick swings: Tor 5, 6, Roberta 5, 6, 7, 10; heavy: Tor 4, 7, 8, 12 and Roberta 4 (SPEC-0038 Behavior 44). A blocked row-214 Wizard is untouchable from the front by non-power swings (SPEC-0036 Hit detection step 4); the guard dot is 0.9914. Missile defence: for each live hero shot whose next 60 ticks of path meet the capsule, int[1, 100] below 60 gives a dodge (clip 19), then on row 214 int[1, 100] below 50 gives a block that overrides it. Counters: only row 214, slot 1, trigger 6, chance 75: eligible on every tick of a hit reaction when the slot's cooldown is below 1 (the roll int[0, 100] is below 75 on 74 of 101 draws per tick). Because the reaction lasts about 2 ticks, a hit on a Wizard whose bolt is off cooldown is answered with a stun bolt about 90 percent of the time.

#### Ghost row 215 (3 placements: Meridell Plains 1 patrol, Heroes Rest 2 sentry)

Row 215 is a ghost row: solid phase int[120, 180] ticks, ghost phase int[120, 180] ticks, opacity 0.2 (ghost) to 0.7 (solid), reveal int[30, 180] ticks (SPEC-0029 Behavior 33). Its hidden state forces the ghost branch with S = 0, so a decloaked row-215 Wizard starts in its ghost phase: hits are absorbed (no damage, no hit reaction, no doubling), the first hit starts a reveal of int[30, 180] ticks during which it is hittable and fades in, and a ghost that is attacking is targetable at all times. Solid phases are hittable and are the only time the hero's auto-target locks it. The row's unused slot 0 still holds the Archer key 2807, but a slot of the unused type is never chosen and the key is never read, so this Wizard summons nothing.

#### Hero actions against this enemy

| Hero action | Result |
|---|---|
| Stands 30 units away | Fire row: fireballs 111 to 125 ticks after the choice; poison row: orb, drifts to the hero in about 8 s; others: a 10-damage bolt (row 214 stuns for 6 to 7 s) 110 ticks after the choice |
| Closes inside 25 units | The Wizard backs off at 16.8 per second; inside 20 it retreats at 46.8 per second; the hero cannot catch it |
| Melee swing within 8 units | Dodged on 89 of 100 swings (row 214: blocked on 100, 85, 75 percent of quick, normal, heavy swings); after one dodge there are at most 60 quiet ticks |
| Roberta's shots | Dodged on 59 of 100 (row 214 blocks half of the survivors); hold 60 ticks |
| A landed hit | Damage by SPEC-0036 (doubled out of agro), hit reaction of about 2 ticks, attack cancelled, counter chance (row 214) |
| Stands in a fire patch | Nothing: the landed lob never pulses (SPEC-0029 Behavior 9, Area mode); only a lob that touches him in flight hurts (5 and fire; row 215's radius-0 lob never touches) |
| Kills a Wizard row 211 | Its Grunts keep fighting; their summon cap frees as they die |

#### Edge cases for implementers

- The summoned Grunts (key 2844, row 183) are summon-reserved records in the Wizard's own block (four in every block that holds a row-211 Wizard; the larger regions hold 8 to 14 reserved records for the Wizards and other summoners). A request binds the first unbound reserved record of the key. They start in state 21 with clip 82 requested by the spawner and enter agro 65 ticks later; they are visible and hittable from the first tick of 82. A Grunt that lands where the hero stands is already inside its swing range when it appears.
- Row 215's slot 0 key (2807) is never read: slot 0 has the unused type, so no chooser or sequencer can reach it.
- The pool of ten enemy projectiles is shared: four patches of a row-211 or 215 volley occupy four slots for about 600 ticks each, so two Wizards of those rows fill the pool and starve every other enemy's shots (the counter still advances and the attack still ends on schedule).
- Kind 7 hit reaction waits on the character's height above the ground (the motion record's ground gap, zero when its floor flag is set), not on a speed: the reaction ends on the first tick after the request on which that gap is at most 2.0. A Wizard hovering more than 2 units above the ground under it (hero on a ledge or a stair, ground dropping away) stays in the reaction, unable to attack or move by archetype, until it descends (Medium; no case seen on level ground).
- The map-search spell and the attack both set up at the pre-step and choice; a cast that begins while a wind-up runs abandons the wind-up (the shot window may already have fired).
- Row 214's Blocking Angle 0.9914 makes the block cover everything except a swing from within 7.5 degrees of the Wizard's own facing.
- Death-pose and corpse: flyers do not sink; the corpse is gone about 313 ticks after death.

#### What a port most likely gets wrong

1. Treating the six rows as one enemy. Rows 211, 215 and the others have different weapons, ranges and cooldowns; the element (T) is not the only difference.
2. Giving the Wizard a melee fear or a retreat that the hero can catch: it retreats at 46.8 per second and approaches at the same speed, and it hovers at the hero's own height.
3. Playing a hit-stagger clip. The kind-7 reaction has no clip on this rig; it lasts about 2 ticks and only cancels the attack.
4. Firing the fire volley as one shot. Rows 211 and 215 release four shots on four separate event windows 4 ticks apart, each landing as a 600-tick area that never pulses (it only holds a pool slot); the poison orb crawls at 3.6 per second.
5. Ignoring the summon cap (4 live Grunts per Wizard) or letting Grunts appear without the 82 clip.
6. Letting a row-215 ghost take damage during its ghost phase.
7. Using the Magician's casts here: only rows 213 and 216 cast support spells, and they cast on any nearby character, not only enemies of the hero.
8. Forgetting the double damage on a Wizard hit during its decloak (136 ticks), its state 23 (89 ticks) and its cast (state 18).

#### Test vector

Row 211 Wizard, agro at t = 0 (the tick the setter ran), hero standing still 30 units in front inside the camera cone, nothing else near. t = 1 set-up. t = 2 first choice: fire 74 percent, otherwise summon 13 percent, otherwise try again each tick (c = 2 with 87 percent). Fire at c: s = c + 1; fireballs leave at c + 111, c + 115, c + 119, c + 123; each lands 11 ticks later (hero 30 units away, scatter up to 10 units) without hurting him (the lob steps past his body; the landed area never pulses); end path c + 172; recovery int[30, 60]; fire cooldown int[300, 480] from c + 172 (next fire no earlier than c + 472). With fire on cooldown the Wizard chooses the summon at 49 percent per tick after its recovery: lob leaves at c' + 51, the Grunt appears 25 ticks later, 26.7 units from the Wizard (lob speed 1.0, d = 30; it lands when its centre is 1.0 above the ground), plays 82 for 65 ticks and enters agro at c' + 146; end path c' + 83; summon cooldown int[180, 300] from c' + 83. Over 600 ticks: 2 to 4 choices (mean 3: 1.3 fire volleys, 1.7 summons), up to 4 live Grunts, and a hero who stands still 30 units away takes no fire damage at all (no lob touches him and no landed area pulses). Row 216 under the same conditions: curse bolt released at c + 111, hits 10 ticks later for 10 and a 30 to 60 s curse, end path c + 172, next choice after recovery int[30, 60] and the cooldown int[30, 300] from c + 172: 2 or 3 bolts in 600 ticks.

#### Open questions

- Whether the kind-7 reaction's ground-gap wait is reached on level ground at all (the Wizard's own height versus the ground under it). Default if unresolved (rewrite choice): the reaction ends 2 ticks after the hit.
- Whether row 216's missing-HP damage goes through the target's take-hit hook (doubling, alert, agro): Resolved (High; proactive (open close-out)): no. The spell builder calls the shared HP subtraction directly (current HP minus the amount, or HP 0 and the death hook when the amount is not below the current HP); the take-hit hook is never entered.
- Whether row 213's polymorph reloads the target's HP from the new row: Resolved (High; proactive (open close-out)): no. The variant re-pick selects the monster row again and loads the colour preset only; the target's current and maximum HP words are not touched (SPEC-0029 Edge cases, run-time variant re-pick), so no clamp is needed either.
- The distance lane of the candidate filter: Resolved (High; proactive (open close-out)): horizontal. The picker takes the distance from the shared two-actor helper, whose second value is the distance with the height difference dropped (the same helper as the wake test of SPEC-0038 Behavior 40).
- Resolved: the first fire lob does not strike a hero standing at 30 units even with scatter 0: its step (about 2.6 per tick) carries it past the body cylinder between two tests and it lands at his feet; the landed area never pulses (Distances and ranged reach; SPEC-0029 Behaviors 9 and 13a).

### Minion Magician (rows 197 to 202)

**Player's answer (R-0149).** It holds 15 to 40 units away and dodges a quick swing 89 times in 100 and a normal or heavy swing 24 times in 100; each dodge leaves 30 to 120 open ticks, so heavy swings and shots are the answer, and it never closes in or gives up (SPEC-0038 Behavior 44).

Monster type 81, variants 0 to 5, name "Minion Magician"; the six rows share one table of animation ids and clips except the death clip (below) and, on rows 200 and 202, the slot-1 ids 38 and 39. 49 records in play (row 197 7, 198 7, 199 13, 200 4, 201 6, 202 12). A weak flyer (80 hit points) that stays at range, throws one slow 25-damage bolt, and on four of its six rows buffs or heals other characters.

#### Identity and tunables

| Item | Value |
|---|---|
| Class, approach | Class 17; approach 3 (keep distance); Can fly 2 (always flying) |
| Hit points, body | 80; radius 2.5, height 5.0, weight 2.0; attack-manager weight 12 (approach 3 never waits); hostile; Display in default 0 |
| Element T | Row 197 T 1, 198 T 2, 199 T 3, 200 T 4, 201 T 5, 202 T 6 (damage element only; the bolt carries no status) |
| Flight | Idle velocity 0.5, agro velocity 1.0 (1.0 and 2.0 units per second on top of root travel); maximum fly height 20.0; hold aim weight 0.5 |
| Notice | See the notice table: awareness 90, cycle 62, cone 0.707, range 3.5 to 50, sees through walls |
| Agro bands | Hold between 15.0 and 40.0; desired distance 15 + 25 f (20.0 to 35.0); retreat when below 15.0, retreat distance drawn in 15.0 to 35.0 |
| Recovery after an attack | int[15, 60] |
| Missile defence | Hold 120 ticks, dodge 50 (49 of 100), block 0 |
| Melee defence | Dodge 90 for quick swings (89 of 100), 25 for normal and heavy (24 of 100); block 0; defence timer int[30, 120] |
| Hit reaction | Flying hit value 0: kind 6 (animation 152 = clip 16, 20 keys, gate 77, pushed back 12.2 units per second) |
| Death | Animation 153, a clip that differs by row: 197 clip 27 (20 keys, gate 77), 198 clip 26 (22 keys, gate 85), 199 clip 23 (17 keys, gate 65), 200 clip 11 (21 keys, gate 81), 201 clip 24 (50 keys at speed byte 1, gate 98), 202 clip 25 (14 keys, gate 53). The corpse is gone 6 ticks after the clip reaches progress 0.95 (disappear timer 5): about 80 to 105 ticks after death; no sink |
| Other | No steal; footstep effects 0 |

Attack slots (strict bands; the attack-choice order is slot 1 then slot 0):

| Rows | Slot | Band, chance, arc | Cooldown | Clips (B gate; release j) | Projectile | Counter |
|---|---|---|---|---|---|---|
| 197, 198, 199, 201, 202 | 0 | 10 to 60, 50, 0.0 | int[180, 420] | B id 34 = clip 7 (44 keys): 172; j = 136 (event at ticks 136 to 139) | 1 straight shot, damage 25, speed 3.0, radius 0.5, range 150 (lifetime 50 ticks), no status | trigger 6, chance 10 |
| 200, 202 | 1 | 10 to 60, 50, 0.0 | int[60, 180] | A id 38 = clip 9 (38 keys): 148; B id 39 = clip 8 (12 keys): 45; j = 4 | 1 straight shot, damage 0, speed 3.0, radius 0.5, lifetime 50; row 200 status 3 (slow) 100 percent, row 202 status 1 (curse) 100 percent, each 30 to 60 s | none |

Map-search rows (shared section): 197 type 3 (strength), 198 type 4 (status 11), 201 type 5 (speed), 199 type 6 (heal to full); each with search distance 50, chance 25 percent per tick, cooldown int[120, 300]. Rows 200 and 202 have none.

#### Animation ids and clips (flyer ids)

| Id | Use | Clip |
|---|---|---|
| 0, 97, 99, 100, 157 to 161, 164 | idle pick and unmapped ids | clip 0 (30 keys, loops, no root travel) |
| 98 | wander | clip 18 (10 keys, loops, 6.76 per second root; 7.8 with the row speed) |
| 107 | forward / retreat | clip 3 (10 keys, loops, root 18.0 per second; 20.0 with the row speed) |
| 108 | back off | clip 2 (12 keys, loops, root -10.5; -12.5 with the row speed) |
| 156 (141) | hold | clip 5 (25 keys, loops) |
| 137 | map-search cast | clip 8 (12 keys, gate 45, event bit 0x20 at ticks 4 to 7) |
| 138, 139 | missile dodge | clip 13 (10 keys, gate 37, sideways 12.7 per second) |
| 140 | melee dodge | clip 12 (15 keys, gate 57, backward 11.1 per second) |
| 152 | hit reaction kind 6 | clip 16 (20 keys, gate 77, backward 12.2 per second) |
| 153 | death | the row's clip (table above) |
| 147, 148 | look, lose interest | clip 22 (60 keys, gate 236), clip 20 (30 keys, gate 116) |
| 82, 165, 149 | decloak, emerge, alert | clip 6 (23 keys, gate 89, rises 13 units per second) |
| 81 | state 23 | clip 30 (17 keys, gate 65) |
| 144, 145, 142 | sentry stand, sentry fidget, patrol fidget | clip 15 (30 keys, loops), clip 14 (30 keys, gate 116), clip 19 (63 keys, gate 248) |
| 163 | victory pose | clip 10 (57 keys, gate 224) |

#### Start, wake, per-tick order

Shared hidden rules with the Magician column of the notice table; sentries stand (144) and fidget (145) at home; wanderers (5) walk clip 18 and pause with fidget 3 (absent: clip 0). A decloaked Magician enters agro 89 ticks after the 82 request and chooses first two ticks later. The agro order, the keep-distance movement, the map search and the counter choice are the standard ones.

#### State table (agro)

| State | Action id and clip | Keys | Gate and exits |
|---|---|---|---|
| Hold (inner 4) | 156 -> clip 5 | 25, loops | d > 40 gives approach; d < 15 gives back off |
| Approach (inner 3) | 107 -> clip 3 | 10, loops | 20.0 per second (0.333 per tick); ends when d < desired (20 to 35) |
| Back off (inner 5) | 108 -> clip 2 | 12, loops | 12.5 per second backward; d < 15 gives retreat |
| Retreat (inner 6) | 107 -> clip 3 | 10, loops | 20.0 per second away until d is at least the drawn retreat distance |
| Bolt (slot 0) | B 34 -> clip 7 | 44 | s = c + 1; bolt leaves at s + 136; B gate s + 172; end path s + 173 |
| Slow or curse bolt (slot 1) | A 38 -> clip 9, then B 39 -> clip 8 | 38, 12 | A requested at s = c + 1, gate s + 148; B requested on that tick; bolt leaves at s + 152; B gate s + 193; end path s + 194 |
| Cast (class state 18) | 137 -> clip 8 | 12 | request; spell at +4; gate +45; setter 19 |
| Melee / missile dodge | 140 -> clip 12 / 138 or 139 -> clip 13 | 15 / 10 | gate 57 / 37, then the defence exit hold int[30, 120] |
| Hit reaction kind 6 (outer 10) | 152 -> clip 16 | 20 | gate 77 after the last hit, then outer 0; restarts on every hit |
| Dead | 153 -> the row's clip | 14 to 50 | plays once |

#### Attack chooser

Each tick while the hero is attackable, recovery is below 1 and the outer sub-state is 1, 3, 4 or 11: slot 1 (rows 200, 202): cooldown below 1, 10 < d < 60, facing above 0.0 (anywhere in front), int[1, 100] below 50 (49 of 100); then slot 0: the same band and roll. Rows with one slot start an attack on 49 percent of eligible ticks. A missile slot needs no route. The hold lies at 15 to 40, so a Magician always sits inside the band of a hero that stays still, and a hero closer than 10 is only retreated from (no attack inside 10). Cooldowns: slot 0 int[180, 420] and slot 1 int[60, 180], drawn at the choice and again at the end path, so a bolt is fired at most once per 180 to 420 ticks after its previous end path. A counter (trigger 6, chance 10) rolls int[0, 100] below 10 on every tick of a hit reaction while its slot's cooldown is below 1, 10 < d < 60 and the hero is in front: a 77-tick reaction therefore ends with a counter bolt on essentially every hit that lands while the slot is off cooldown (99.96 percent), so a Magician with its bolt ready answers a hit by starting the wind-up that interrupts the reaction.

#### Movement

Keep-distance with the flyer gaits: approach at 20.0 per second above 40, hold in 15 to 40, back off at 12.5 per second below 15, retreat at 20.0 per second below 15 (the retreat check follows the back-off, as for the Wizard). The hero (walk 13.5, run 19.3 per second) cannot outrun the 20.0 approach but also cannot catch the retreat. It flies level with the hero's feet (Can Fly 2 height rule, corrected in the shared section; it does not hover 3 to 8 units above the hero).

Movement table (rows 197 to 202; row flying velocity 0.5 idle and 1.0 agro, so the row term is 1.0 per second outside agro and 2.0 in agro; Can Fly 2; radius 2.5):

| Mode | Source | Clip (keys) | u/tick | u/s | Turn | Exit |
| --- | --- | --- | --- | --- | --- | --- |
| Wander, walk home | root + row | 98 (10) | 0.129 | 7.76 (6.76 + 1.0) | steer | arrival 0.5, notice |
| Patrol | row only | 99: clip 0, no root | 0.0167 | 1.00 | steer | arrival 0.5 |
| Decloak on waking (82) | root | 82 (23) | | forward 19.91 over 92 ticks | aim | gate, then agro |
| Approach (inner 3), d above 40 | root + row | 107 (10) | 0.334 | 20.01 (18.01 + 2.0) | steer | d below the drawn desired 20 to 35 |
| Hold (inner 4), 15 to 40 | none | 156 | 0 | 0 | aim 0.25 then 0.5 (effective 0.625) | d above 40 or below 15 |
| Back off (inner 5), below 15 | root + row, facing the hero | 108 (12) | 0.208 backward | 12.46 | steer, back gait | d above desired |
| Retreat (inner 6) | root + row, back to the hero | 107 (10) | 0.334 | 20.01 | steer | d at least the retreat draw 15 to 35 |
| Hit (reaction kind 6, flying-hit value 0) | root | 152 (20) | | back 16.28 over 80 ticks, away from the attacker | snap to attacker | gate, outer 0 |
| Dodge 138 / 139, 140 | root | 138/139 (10), 140 (15) | | side 7.54 over 40 ticks; back 11.06 over 60 | | gate |
| Corpse | root (held) | 153 (17), last key -2.078 | 0.52 per tick during the last key, then 1.04 held | | facing kept | disappear timer 5 ticks: released about 3 ticks after the clip ends |

The Magician has no ground gaits; a ground request plays clip 0 (no root).

#### Hit and hurt windows

The bolt is a straight shot at 3.0 per tick from the emit point to the hero's position + 2.5 at the release tick; first contact for a hero 25 units away is 7 ticks after release (emit point 7.05 up and 1.05 in front), 25 damage (before armour and shield), no status, no knockback. The 0-damage bolts of rows 200 and 202 still apply their status at 100 percent. A Magician can be hit in every agro sub-state except the two dodges, while dead, while hidden; a hit during the cast (state 18), the decloak and state 23 is doubled. Hurt shapes (Medium): bone 0 length 2.95 radius 1.0 plus eight limb capsules (radius 0.40 to 0.51).

#### Defence and counters

Melee defence once per swing (hold below 1, not mid-attack except sequencer states 0, 1, 5, hero within 8.0 and in front): quick swing 89 of 100 dodges, normal and heavy swings 24 of 100 (a 57-tick dodge clip, then hold int[30, 120]). Missile defence: dodge 49 of 100 per live hero shot meeting the capsule inside 120 ticks, clip 13 (37 ticks). Hit reaction kind 6 as above; a hit cancels a running wind-up or cast. Counter on slot 0 as above (chance 10 per tick).

#### Hero actions against this enemy

| Hero action | Result |
|---|---|
| Stands 25 units away | Bolt 137 ticks after the choice (choice about 3 ticks after agro), hits 7 ticks later for 25; next bolt 180 to 420 ticks after the first bolt's end path |
| Walks in | The Magician retreats (20.0 per second) below 15; the hero cannot reach it |
| Quick melee swing inside 8 | Dodged on 89 of 100 swings |
| Normal or heavy swing inside 8 | Dodged on 24 of 100 |
| Roberta's shots | Dodged on 49 of 100 when the path meets the body inside 120 ticks |
| A landed hit | Damage by SPEC-0036 (doubled out of agro and out of the cast), hit reaction 77 ticks, counter bolt at about 10 percent per tick |
| Wounded allies within 50 units | Rows 197, 198, 199, 201 buff or heal them with a 45-tick cast (clip 8's gate) |
| Kills the Magician | Corpse gone 80 to 105 ticks later; no drop beyond the placement's ordinal |

#### Edge cases for implementers

- Four rows are support casters: the first Magician to cast also re-broadcasts the alarm group, and its caster state 18 gives the hero a double-damage window (the cast takes 45 ticks).
- Row 199's heal needs a wounded candidate; rows 197, 198 and 201 buff any eligible character in range, wounded or not, so a Magician near a boss or another Magician gives them strength, speed or poison attacks repeatedly (every 120 to 300 ticks).
- The map search cooldown, the recovery countdown and the slot cooldown are three separate clocks; the cast clears the recovery and slot cooldowns on exit, so the Magician bolts at once after a cast.
- Rows 200 and 202 have a wind-up (A, 148 ticks) before the 0-damage bolt; row 202 chooses slot 1 first.
- Row 197's wounded-ally filter is absent for the buff types; the cast is never aimed at the hero.
- Wander gait 98 plays a walking clip, patrol gait 99 plays the idle clip (patrolling Magicians slide at the row speed of 1.0 per second).

#### What a port most likely gets wrong

1. Making the Magician a melee or rush enemy: it never approaches closer than 15 and never attacks inside 10.
2. Using the Grunt's stagger: the Magician reaction is 77 ticks with a pushback, and its counter bolt fires at 10 percent per tick.
3. Firing the bolt at once. The strike clip has a 136-tick windup before the event; a hit during that time cancels it.
4. Skipping the support casts, or aiming them at the hero.
5. Omitting the bolt-less statuses of rows 200 and 202 (damage 0, status 100 percent).
6. Forgetting that row 200 and 202 slot 1 are checked before slot 0.

#### Test vector

Row 197 Magician, agro at t = 0, hero standing still 25 units in front in the camera cone, no other character within 50 units. t = 1 set-up. t = 2: choice 49 percent per tick (mean about t = 3). Slot 0: s = c + 1, bolt released at c + 137, hits at c + 144 for 25, end path c + 174, recovery int[15, 60], cooldown int[180, 420] from c + 174, so the second choice falls between c + 356 and c + 598 and its bolt releases between c + 493 and c + 735. In 600 ticks: 1 to 2 bolts (25 or 50 damage), no statuses, plus a counter bolt only if the hero hit it. With a wounded character within 50 units (row 199): a cast roughly every 120 to 300 ticks, each costing the Magician 45 ticks.

#### Open questions

- Whether the buff types exclude characters that are not enemies of the hero (townsfolk, petpets): Resolved (High; proactive (open close-out)). There is no class or allegiance test. The candidate list is every other character of the active region list (enemies, bosses, townsfolk and other creatures; never the heroes), minus the hidden ones, the script-held ones, the active petpet and petpet-keyed characters, and the fixed list of excluded character keys given in the shared section (the casters' own keys; Dark Knights and Artillery are not excluded).
- The distance lane (horizontal or three-dimensional) of the candidate test: Resolved (High; proactive (open close-out)): horizontal (see the Wizard item of the same name).
- Whether the strength, speed and status-11 spells stack or refresh on a target that already carries the status: Resolved (High; proactive (open close-out)): they refresh. The enemy status-apply method sets the status timer to the new duration (seconds times 60) whatever its old value, and has no other gate for these three codes (SPEC-0029 Behavior 27.1).

### Minion Artillery (rows 162 to 167)

**Player's answer (R-0149).** Run in while it fires its volley or runs its charge (it cannot dodge a swing in its strike states, and inside 15 units it cannot fire at all); every dodge (41 ticks, 99 of 100) leaves a 60 to 120 tick window, and a retreat (20.2 per second against the run's 19.3) ends at a wall or at its drawn retreat distance, never in a give-up (SPEC-0038 Behavior 44).

Monster type 77, variants 0 to 5, name "Minion Artillery"; one table of animation ids and clips for every record in play; 28 records in play. A slow ground gunner (300 hit points) that fires a ten-shot volley from range and answers anyone who comes within 40 units with a 100-damage lunge. The Artillery has a ground rig (Can fly 0): no flyer ids, no map search.

#### Identity and tunables

| Item | Value |
|---|---|
| Class, approach | Class 17; approach 3 (keep distance); Can fly 0 |
| Hit points, body | 300; radius 2.0, height 4.0, weight 5.5; attack-manager weight 8 (approach 3 never waits); hostile; Display in default 0; footstep shake class 1, rumble class 1 |
| Element T and statuses | Row 162 T 1 fire (7), 163 T 2 poison (2), 164 T 3 confusion (6), 165 T 4 stun (0, 6 to 8 s), 166 T 5 slow (3), 167 T 6 curse (1); others 30 to 60 s |
| Notice | Awareness 75, cycle 62, cone 0.92388, range 3.5 to 200, chance 80 at 3.5 falling to 30 at 200, line of sight required (F2 0 for a still hero) |
| Agro bands | Hold between 40.0 and 75.0; desired 40 + 35 f (47 to 68); retreat when below 25.0 (retreat distance drawn in 25 to 100); hold aim weight 0.1 |
| Recovery after an attack | int[30, 120] |
| Missile defence | Hold 30 ticks, dodge 100 (99 of 100), block 0 |
| Melee defence | Dodge 100 for quick, normal, heavy (99 of 100), block 0; defence timer int[60, 120] |
| Hit reaction | Ground: kind 0 (animation 57 = clip 15, 10 keys, gate 37, pushed back 2 per second); the follow-up kind 5 (animation 127) is absent |
| Death | Animation 58 = clip 11 (36 keys, gate 140); the corpse is gone 600 ticks after progress 0.95 (about tick 735) and sinks after the first 30 of those ticks |
| Other | No steal; weapon 1 radius 3.0 height 4.0, weapons 2 to 5 radius 2.0 height 2.0 |

Slots (tested slot 1 then slot 0):

| Slot | Role | Band (strict), chance, arc | Range, cooldown | Damage, status | Clips |
|---|---|---|---|---|---|
| 1 | Charge (kind 2) | 15 to 40, 50, 0.966 (15 degrees) | melee range 10.0; charge distance float[30, 50); cooldown int[60, 120]; orient 0 | 100, unblockable (flag 0x1), knockback 17 (hero), row status at 25 percent per hit, 30 to 60 s (stun 6 to 8 s) | A absent; B id 39 = clip 5 (10 keys, loops, root 19.3 per second at rate 1.0); C id 40 = clip 6 (15 keys, gate 57, weapon 3 (radius 2.0) at ticks 4 to 23, root 17.1 per second); D, E absent |
| 0 | Ten-shot volley | 15 to 100, 25, 0.0 | cooldown int[300, 900]; orient 1 | 4 per shot, row status at 100 percent per hit | A id 33 = clip 4 (20 keys, gate 77); B id 34 = clip 3 (40 keys, gate 156, ten event windows); C, D, E absent |

Speed words: slot 0 1.0 for all phases; slot 1 wind-up 1.0, strike 1.5, recovery 1.0 (the run clip B is requested by state 6, whose tick runs at the wind-up word 1.0; from the first run tick state 7 plays it at the strike word 1.5: 28.95 per second, 0.4826 per tick; the bite C plays at rate 1.0). Slot 0 projectile: straight shot, speed 1.3 per tick, range 110 (lifetime 84 ticks), radius 0.1, aim scatter 0 (+5 outside the camera cone), visual kind 23, launch from the first emit point; spawn count 10 and the clip has exactly ten event windows, so ten shots leave.

#### Animation ids and clips

| Id | Use | Clip |
|---|---|---|
| 0 | idle | clip 0 (30 keys, loops); fidgets 115, 116 = clips 23, 22 (48 keys) |
| 2 | wander | clip 29 (30 keys, loops, root 2.26 per second) |
| 4 | patrol | clip 25 (20 keys, loops, 3.75 per second) |
| 3, 6 | wander fidget, patrol fidget | clip 21 (48 keys, gate 188), clip 23 (48 keys) |
| 7, 8 | sentry stand, fidget | clip 7 (40 keys, loops), clip 19 (58 keys, gate 228) |
| 24 | keep-distance forward, retreat | clip 16 (15 keys, loops, root 20.2 per second) |
| 25 | hold | clip 1 (20 keys, loops) |
| 26 | back off | clip 20 (15 keys, loops, root -15.3 per second) |
| 53, 54, 55 | missile dodge, melee dodge | clip 13 (11 keys, gate 41, sideways 16.5 per second), clip 12 (11 keys, gate 41, backward 18.7) |
| 57 | hit reaction kind 0 | clip 15 (10 keys, gate 37) |
| 58 | death | clip 11 (36 keys, gate 140) |
| 16, 17, 18, 82, 10, 81 | look, lose interest, alert, decloak, emerge, state 23 | clips 26, 24, 2, 2, 2 (gate 93; the model advances 7.25 per second) and 14 |
| 73 | victory pose | clip 28 (45 keys, gate 176) |

#### Start, wake, per-tick order

Shared hidden rules with the Artillery column. A state 8 Artillery (10 records: Bazaar District 6, Meridell Outskirts 2) wakes at its emerge distance (20 on half of them, up to 200 on one) scaled by F1 and plays the emerge clip 10 (93 ticks, advancing 7.25 per second); agro runs two ticks after the gate. The Bazaar District group is an alarm group.

#### State table (agro)

| State | Action id and clip | Keys | Gate and exits |
|---|---|---|---|
| Hold (inner 4) | 25 -> clip 1 | 20, loops | d > 75 gives approach; d < 40 gives back off |
| Approach (inner 3) | 24 -> clip 16 | 15, loops | 20.2 per second (0.337 per tick); ends when d < desired (47 to 68) |
| Back off (inner 5) | 26 -> clip 20 | 15, loops | 15.3 per second backward toward the point 75 from the hero; ends when d > desired; d < 25 gives retreat |
| Retreat (inner 6) | 24 -> clip 16 | 15, loops | 20.2 per second away; ends when d is at least the drawn retreat distance (25 to 100) |
| Volley wind-up (state 1) | 33 -> clip 4 | 20 | s = c + 1, gate s + 77, B requested on that tick |
| Volley fire (state 2) | 34 -> clip 3 | 40 | shots at B + 8, 16, 28, 40, 56, 68, 84, 96, 108, 120; B gate B + 156; state 3 then state 5; end path B + 157 |
| Charge run (states 6 then 7) | 39 -> clip 5 | 10, loops | B requested on the choice tick c (that tick's pass at the wind-up word 1.0); state 7 from c + 1 at the strike word 1.5; ends when the hero is within 10.0 (state 8) or the drawn distance (30 to 50) is covered (state 12) |
| Bite (state 8) | 40 -> clip 6 | 15 | requested on the tick after state 7 saw the hero within 10.0 (call it b); weapon 3 live at b + 4 to b + 23; gate b + 57; end path b + 58 |
| Charge end (state 12) | E absent | | end path on the next tick |
| Dodges, reaction, death | as the table above | | gates 41, 41, 37, 140 |

#### Attack chooser

Tick by tick while the hero is attackable, recovery is below 1 and the outer sub-state is 1, 3, 4 or 11: slot 1 (charge): cooldown below 1, 15 < d < 40, facing above 0.966, a route to the hero exists (a charge needs one even inside 10), int[1, 100] below 50 (49 of 100). Then slot 0: cooldown below 1, 15 < d < 100, in front, a roll below 25 (24 of 100). Start probabilities per tick: 15 < d < 40 facing: 49 percent charge, otherwise 24 percent volley (61 percent in all); 40 <= d < 100: volley 24 percent; d <= 15: nothing (the Artillery backs off and then retreats). After a choice the slot's cooldown is drawn (charge 60 to 120, volley 300 to 900) and again at the end path; the desired distance and the charge distance (float[30, 50)) are redrawn.

#### Movement

Ground keep-distance: hold in 40 to 75, approach at 20.2 per second above 75, back off at 15.3 per second below 40 (toward a point 75 from the hero) until beyond the drawn desired distance, retreat at 20.2 per second below 25 (running away from the hero to a distance of 25 to 100). Because the retreat minimum (25) is below the desired minimum (40), the retreat is reached only from the back-off state, never from the hold. The charge is committed: the heading is set once at the choice (turn blend 0.1), the run is straight at 28.95 per second and ignores the hero afterwards.

Movement table (rows 162 to 167; ground; radius 2.0; ground turn 0.1; desired 40 to 75 (draw 47 to 68), retreat 25 to 100):

| Mode | Source | Clip (keys) | u/tick | u/s | Turn | Exit |
| --- | --- | --- | --- | --- | --- | --- |
| Wander, walk home, wait in line | root | 2 (30) | 0.0377 | 2.26 | steer | arrival 0.5, notice |
| Patrol | root | 4 (20) | 0.0625 | 3.75 | steer | arrival 0.5 |
| Waking (emerge 10 or decloak 82) | root | (24) | | forward 11.57 over 96 ticks | | gate, agro |
| Return to the default state (81, state 23) | root | 81 (10) | | forward 11.76 over 40 ticks | kept | end |
| Approach (inner 3) and retreat (inner 6) | root | 24 (15); surges from 0.0 to 0.94 per tick inside the 60-tick cycle | 0.337 | 20.23 | steer | desired; retreat: d at least the retreat draw |
| Hold (inner 4) | none | 25 | 0 | 0 | aim 0.25 then 0.1 (effective 0.325) | d above 75 or below 40 |
| Back off (inner 5), below 40 | root, facing the hero | 26 (15); 0 to 0.61 per tick inside the cycle | 0.255 backward | 15.31 | steer, back gait | d above desired |
| Charge run (slot 1, state 7) | root | 39 (10) at 1.5 | 0.4825 | 28.95 | none (straight) | the drawn 30 to 50, or blocked |
| Bite after the run (C) | root | 40 (15) | | forward 16.79 over 60 ticks | none | gate |
| Hit (reaction kind 0) | root | 57 (10) | | back 1.34 over 40 ticks | snap to attacker | gate |
| Dodge 53 / 54, 55 | root | (11) each | | side 12.03 over 44; back 13.44 over 44 | | gate |
| Corpse | root | 58 (36) | | back 12.30 over 144 ticks, nothing held | kept | timer 600 |

The approach and back-off clips move in surges (a lunge every cycle), so a port that averages them to a constant speed travels the right distance with the wrong rhythm. Because the ground turn is 0.1, a hero that circles the Artillery is tracked slowly in the hold; the retreat's turn to run takes the steering arc of SPEC-0038 Behavior 51 item 6 (weight 0.3 or 0.1 per tick, with the near-opposite side nudge), so the Artillery runs out in a curve.
#### Hit and hurt windows

- Volley: ten shots 8 to 120 ticks after B, each aimed at the hero's position + 2.5 up at its own release tick; flight 1.3 per tick (first contact for a still hero 50 units away 36 ticks later). With the hero standing: 10 hits of 4 damage, each with the row's status at 100 percent (the shots are removed by walls after 2.0 units and by the pool when full; the counter still advances). A hero who moves sideways while the volley runs has each shot re-aimed at where he stands at that tick, with no lead.
- Charge: the run does no damage. The bite's weapon 3 (radius 2.0, height 2.0) is live at b + 4 to b + 23 and the bite clip lunges the Artillery forward 16.8 units over its 57 ticks, unevenly (0.07 to 0.66 per tick, most of it early). The former single-tick first-contact examples (b+4 at 6 units, b+6 at 8, b+10 at 10, b+13 at 12, b+15 at 14) are withdrawn: this is a solid kind-2 charge and the body can push Tor, so a stationary-target weapon grid cannot establish the actual contact tick. Whether contact occurs, and when, remains open until body movement/push and exact weapon contact are simulated together.
- Hurt: the Artillery can be hit in every state except the dodges (outer 5 and 6), dead and hidden; the charge and the volley are cancelled by a hit (hit reaction kind 0, 37 ticks, restarting on every hit). Hurt shapes (Medium): body capsule of length 2.54 radius 1.64 and a smaller one of length 1.14 radius 0.86.

#### Weapon reach

Conventions are those of "Weapon reach tables" in the shared field behaviour. Rows 162, 163, 165 and 166, Medium. The bite window is b + 4 to b + 23 as above; b is the tick the bite clip is requested.

| Slot (phase, clip) | Weapon | Window ticks (level contact) | Centre forward / up / side | Tor ahead, at the tick | Tor ahead, from the start | Tor feet vs origin |
|---|---|---|---|---|---|---|
| 1 (C, clip 6) | w3 (2.0 x 2.0) | b+4 to b+23 (b+4 to b+19) | 2.7 to 4.0 / 0.9 to 6.9 / 0.1 to 0.4 | 0.4 to 6.6 | 2.7 to 16.6 | -3.5 to 8.5 |

A level hero is touched from b + 4 to b + 19 and only 0.4 to 6.6 ahead of the Artillery at the tick; measured from where the Artillery stood when the bite clip began, the contact band runs from 2.7 to 16.6 ahead, because the bite clip carries it forward (about 10 units by b + 19, 16.8 over the whole clip). The weapon centre moves between 0.9 and 6.9 above the Artillery's origin over the window, so a hero's feet from 3.5 below to 9 above its origin are covered across the window.

#### Defence and counters

Melee: dodge on 99 of 100 swings in reach (8.0, in front), hold 60 to 120 ticks after any dodge; missile: dodge on 99 of 100 shots meeting the capsule inside 30 ticks, hold 30. No counter slot. The reaction (clip 15, 37 ticks) starts on every hit and restarts on the next.

#### Hero actions against this enemy

| Hero action | Result |
|---|---|
| Stands 50 units away | One ten-shot volley every 300 to 900 ticks after the first (first shot 86 ticks after the choice) |
| Walks in below 40 | The charge starts at 49 percent per tick while the Artillery faces him within 15 degrees: a 100-damage bite about 40 to 60 ticks later if the hero is still there |
| Stands beside the Artillery | The Artillery backs off (15.3 per second), retreating below 25; the hero (19.3 running) cannot be outrun by the charge (28.95) |
| Swings at it | 99 of 100 dodged when in reach |
| Roberta's shots | 99 of 100 dodged inside the 30-tick hold |
| A landed hit | Damage by SPEC-0036 (doubled out of agro), reaction 37 ticks, volley or charge cancelled |

#### Edge cases for implementers

- Ten shots fill the whole projectile pool; any other shooter that tries during the volley gets nothing, and a full pool makes the Artillery's own shots vanish (the counter still advances, so the volley still ends at its gate).
- The first two ticks after waking: the volley chooser needs d > 15 and d < 100; a hero closer than 15 only provokes the back-off.
- The charge distance is a draw of 30 to 50, always at least the 25 units needed to close from 35 to 10, so a hero at 35 or less is always reached by the run unless the heading is off. The run ignores the hero's sidesteps.
- Row 167 (one record, pack 1414) and the other rows are identical apart from element.
- State 0 (idle until alarmed or hit): one record in Fyora's Palace; alarm groups wake it into state 21 like the others.

#### What a port most likely gets wrong

1. Firing one projectile because the row says ten: ten windows, ten shots, 4 damage each with the status every time.
2. Missing the 77-tick wind-up before the volley.
3. Making it approach the hero: it keeps 40 to 75 and runs away inside 25.
4. Letting the hero outrun or out-dodge the charge: the run is fixed in heading and fast, the bite is unblockable.
5. Forgetting that melee and Roberta's shots are dodged 99 percent of the time.
6. Using one walk speed: forward 20.2, back 15.3, wander 2.26, patrol 3.75.

#### Test vector

Row 163 Artillery, agro at t = 0, hero standing still 50 units in front, in the camera cone. t = 1 set-up. t = 2 on: volley chosen at 24 percent per tick (mean about t = 5). With c the choice tick: s = c + 1, B at c + 78, shots at c + 86, 94, 106, 118, 134, 146, 162, 174, 186, 198, each reaching the standing hero 36 ticks later (c + 122 to c + 234) for 4 damage and poison; B gate c + 234, end path c + 235, recovery int[30, 120], cooldown int[300, 900] from c + 235. In 600 ticks: one volley (40 damage, 10 poison applications), a second only when the cooldown is below 365 (about 9 percent). If the hero steps to 30 units: 49 percent per tick the charge starts (cooldown 0 at first): B at c, run 0.4826 per tick, in range 10 about 41 ticks later, bite requested at b (the next tick), weapon window b + 4 to b + 23, first touch on b + 10 for the hero 10 ahead, 100 damage and poison at 25 percent per hit, end path b + 58, recovery int[30, 120], charge cooldown int[60, 120].

#### Open questions

- The Artillery's weapon 3 contact geometry during the lunge. Resolved by the general rule of the shared field behaviour (the lunge moves the enemy and its attachment frame together, and the test follows the frame tick by tick). The distances are in the Weapon reach table. Stand-in only for a port without attachment frames (rewrite choice): count the bite as hitting if a level hero is 0.4 to 6.6 ahead of the Artillery at a tick of b + 4 to b + 19.
- The first charge distance before any normal choice: Resolved (High; proactive (open close-out)). The creation draw reads the two row words below slot 0's charge range, which on all six Artillery rows are an all-ones word (not a number) and 0.0, so its value is meaningless; but it is never used. The Artillery has no counter slot, and its charge (slot 1) can start only through a normal choice, which redraws the distance from the slot's own range float[30, 50) before the run begins. Every charge therefore runs float[30, 50); no creation draw is needed.
- Whether the volley's shots are blocked by the guard. Default if unresolved (rewrite choice): the pool rules of SPEC-0029 Behavior 9 (guard blocks shots from the front).

### Dark Knight (row 139)

Monster type 64 variant 0, name "Dark Knight"; one table of animation ids and clips for all 42 records in play (the 41-clip set of the armoured knight rig). A visible, armoured brawler with four melee swings and no ranged attack, met on patrol in Market Town, Black Keep Grounds, Meridell Village and one wander pack of twelve. Unlike the hidden minion kinds it has Display in default 1: it is drawn and hittable in every state and it uses the ordinary state-2 search (the rules of the Road and Ridge Brigands section: two-stage notice, alert and search clips), not state 21.

#### Identity and tunables

| Item | Value |
|---|---|
| Class, approach | Class 17; approach 4 (strafe); Can fly 0 |
| Hit points, body | 150; radius 3.0, height 5.0, weight 3.0; defence value 10.0 (SPEC-0036 damage formula); attack-manager weight 12 (approach 4: it can wait in line, stand-off radius 15.0); hostile; Display in default 1; element T 6 |
| Notice | View cosine 0.92388, awareness 100 (roll A always passes), timer 10 (cycle 12 ticks), range 10 to 50, chance 50 at 10 and 40 at 50, no line-of-sight flag; search timer float[180, 300) ticks |
| Turn | Hold aim weight 0.1 |
| Strafe band | 6.0 to 16.0 (hold between); desired distance 6 + 10 f (8.0 to 14.0) redrawn after every normal attack choice; backbiter factor 1.0 (no effect) |
| Recovery after an attack | int[60, 180] ticks |
| Slot cooldowns | All four slots draw int[0, 0]: a slot is never on cooldown |
| Missile defence | Hold 60, dodge 75 (74 of 100), block 0 |
| Melee defence | Dodge 75 for every swing class (74 of 100), block 0; defence timer int[120, 300]; Blocking Angle 0 (never guards) |
| Hit reaction | Ground: kind 0 (animation 57 = clip 26, 13 keys, gate 49, pushed back 5.5 per second) |
| Death | Animation 58 = clip 25 (56 keys, gate 220); corpse gone 510 ticks after progress 0.95 (about 720 ticks after death); it sinks after the first 30 of them |
| Socialize | Flag 1, pair radius 15.0 (separation at most 30), talk time float[120, 180), quiet countdown float[600, 900); the settled socialize rules apply (chat id 14 = clip 24, 38 keys, on a draw of 0; ids 111 to 114 are absent and play the idle) |
| Other | No steal; footstep shake class 0, rumble class 1; weapon volumes: weapons 1 and 2 radius 6.0 height 2.0, weapons 3 to 5 radius 4.0 height 2.0 |

Attack slots (all four melee; slots tested 3, 2, 1, 0; the slot bands overlap):

| Slot | Band (strict) | Chance (rolled below) | Arc (dot above) | Melee range | Damage and effect | Orient | Counter | Clips (phase ids 33 + 5 k + p) |
|---|---|---|---|---|---|---|---|---|
| 3 | 0 to 8 | 100 (99 of 100) | 0.9239 | 7.0 | 10, blockable, no status | 1 | none | B 49 = clip 16 (20 keys, gate 77, weapon 5 radius 4.0 at ticks 20 to 43 of B); others absent |
| 2 | 0 to 10 | 60 (59 of 100) | 0.707 | 15.0 | 50, blockable | 0 | none | B 44 = clip 10 (10 keys, gate 37, weapon 1 radius 6.0 from tick 16); D 46 = clip 11 (10 keys, gate 37) |
| 1 | 7 to 15 | 15 (14 of 100) | 0.707 | 12.0 | 50, unblockable (flag 0x1), curse (status 1) at chance 75 (75 of 101 draws), 30 to 60 s | 0 | trigger 5 (sequencer blocked with L1 held), chance 50 | B 39 = clip 13 (20 keys, gate 77, weapon 1 from tick 48 of B); D 41 = clip 14 (10 keys, gate 37) |
| 0 | 0 to 16 | 50 (49 of 100) | 0.707 | 13.0 | 20, blockable | 1 | none | A 33 = clip 17 (10 keys, gate 37); B 34 = clip 18 (10 keys, rate 1.5 then 1.0, gate 25 at rate 1.5; weapon 1 radius 6.0 ticks 6 to 10 of B); D 36 = clip 9 (12 keys, gate 45, backward 12.9 per second) |

Speed words: slot 0 wind-up 1.0, strike 1.5, recovery 1.0; all other slots 1.0. Knockback on the hero is 0 on every slot. Phase ids not listed have no clip and are skipped in the same tick; slots 1 to 3 have no wind-up, so their strike clip is requested on the first sequencer tick.

#### Animation ids and clips

| Id | Use | Clip |
|---|---|---|
| 0 (13, 15) | idle pick | clip 28 (40 keys, loops); fidgets 115, 116 = clip 31 (63 keys, gate 248), clip 32 (60 keys, gate 236) |
| 2, 4 | wander, patrol walk | clip 5 (30 keys, loops, 3.0 per second) |
| 3 | wander fidget | clip 6 (30 keys, loops, 1.6 per second) |
| 6 | patrol fidget | clip 21 (40 keys, one-shot, gate 156) |
| 7, 8 | sentry stand, fidget | clip 30 (40 keys, loops), clip 32 (60 keys, gate 236) |
| 28 | strafe forward (approach, charge-less pursuit) | clip 0 (15 keys, loops, 10.0 per second) |
| 29, 30 | circle (side gaits) | clip 1 (12 keys, +4.2 per second sideways), clip 2 (12 keys, -4.5 per second) |
| 31 | strafe hold | clip 23 (20 keys, loops) |
| 32 | strafe back off | clip 7 (12 keys, loops, -6.0 per second) |
| 53, 54, 55 | missile dodge, melee dodge | clip 3 and clip 4 (13 keys, gate 49, sideways 16.3 per second), clip 27 (10 keys, gate 37, backward 7.7) |
| 57 | hit reaction kind 0 | clip 26 (13 keys, gate 49) |
| 58 | death | clip 25 (56 keys, gate 220) |
| 16, 17 | look, lose interest | clip 21 (40 keys, gate 156), clip 29 (23 keys, gate 89) |
| 18 | alert | absent |
| 81, 82 | state 23 / hidden clips | clip 23 and clip 22 (not used by this kind: Display is 1, state 21 never runs) |
| 73 | victory pose | clip 24 (38 keys, gate 148) |
| 14 | chat | clip 24 |

#### Start, wake, return

1. Start: default word 5, 6 or 7 enters the default state on the first think tick (the start sets state 0 and the pre-step of the same tick runs the setter with the default word). Default 0 (one Meridell Village record) stands in state 0: no notice check, it wakes only by a hit or by its alarm group.
2. Wake from 5, 6, 7: the notice check (shared rules, Dark Knight values) passes on the one tick of each 12-tick awareness cycle. A pass enters state 2 (search countdown float[180, 300)); the state-2 handler runs on the same tick, aims at the stored hero position with blend 0.1, requests the look clip 16 (clip 21, 40 keys, 156-tick gate), and repeats the check with fresh rolls. A second pass on that tick or on a later cycle tick finds no alert clip (id 18 is absent from the table) and runs the setter with 19: the Dark Knight is in agro at once, with no further wait. If no pass occurs before the countdown ends, it requests the lose-interest clip 17 (89 ticks) and runs the setter with 0 afterward (state 0, then on the next tick the default state: there is no state-23 step because Display is 1).
3. Chance that a still hero within the cone is noticed per cycle: inside 7.0 units 1.0 (no roll B), otherwise (ceil(c) - 1) / 100 with c = 50 - 10 (d - 10) / 40: 0.49 at 10 to 12 units, 0.47 at 20 units, 0.44 at 30 units, 0.43 at 35 units; halved outside the cone. Expected wake: within one to three cycles (12 to 36 ticks) for a hero who stays; the search countdown allows 15 to 24 further tests.
4. Alarm group: a member entering agro broadcasts; Dark Knights in states 0, 3, 5, 6, 7 go to state 2 within int[15, 45] ticks and request id 18 directly. Because the table has no clip for 18, the lookup miss plays clip 0 of the table, which on the knight rig is the strafe-forward loop (15 keys, 59-tick gate, about 10 units per second of forward travel, so about 10 units of forward slide over the wait), and the state-2 handler waits for that clip's gate and then enters agro: always 59 ticks after the request, with no notice roll in between. (Medium: the slide length.)
5. A hit at any time wakes it into agro with doubled damage and a kind-0 reaction (13 keys, 49 ticks), from state 0, 2, 3, 5, 6, 7. Return to the default state: the leash (181 ticks without a route while the waypoint is 15.0 or more from the hero), the lost-sight search (above 120 ticks) or the victory pose end in the setter with 0, and the next tick's pre-step runs the default state (walk home and wander, patrol or stand).

#### Per-tick order (agro) and state table

Standard agro order. The outer switch starts at outer 0 and moves to outer 4 (strafe, inner 10).

| State | Action id and clip | Keys | Gate and exits |
|---|---|---|---|
| Set-up (outer 0) | agro idle pick 1 (this section's table lists no id 1: clip 0, the strafe-forward loop; the Spirit Sword table of the same rig maps id 1 to its strafe hold clip 23 - Medium) | 15 or 20 | one tick, then outer 4 inner 10 |
| Strafe hold (inner 10) | 31 -> clip 23 | 20, loops | hold counts down from int[60, 180]; at 0 a roll int[1, 3]: 1 stay, 2 circle inner 8 (travels to the knight's right), 3 circle inner 9 (travels to its left) (a circle is refused to a stay while the strafe lock is above 0) |
| Circle (inner 8 / 9) | 30 / 29 -> clip 2 / 1 | 12, loops | steer 2.0 to the side; sideways about 4.5 / 4.2 per second; hold int[60, 180]; ends at hold 0 (back to inner 10), blocked (lock 120), d > 16 (approach) or d < 6 with the lock below 1 (back off) |
| Approach (inner 7) | 28 -> clip 0 | 15, loops | 10.0 per second (0.167 per tick); ends when d < desired (8 to 14) |
| Back off (inner 11) | 32 -> clip 7 | 12, loops | 6.0 per second backward toward the point 16 from the hero; ends when d > desired |
| Slot 3 swing | 49 -> clip 16 | 20 | s = c + 1; window from s + 20; first touch of a level hero 2 to 4 units ahead s + 20; gate s + 77; end path s + 78 |
| Slot 2 swing | 44 -> clip 10 | 10 | s = c + 1; window s + 16 to s + 37; first touch of a level hero s + 16 (4 units ahead), s + 17 (6), s + 19 (8) without the chase stop; with the stop (the knight cannot step inside 4.175 of Tor) s + 19 at 4 to 8.5, s + 18 at 3 to 3.5, s + 17 at 2.5, s + 16 at 2 or less; gate s + 37; end path s + 38 |
| Slot 1 swing | 39 -> clip 13 | 20 | s = c + 1; window s + 48 to s + 77; first touch of a level hero s + 48 (3 to 6.5 units ahead; up to 2.5 with the chase stop), s + 49 (7 to 9, and 3 to 9 with the chase stop), s + 50 (9.5 to 12 and 13), s + 49 at 12.5; gate s + 77; end path s + 78 |
| Slot 0 swing | 33 -> clip 17 then 34 -> clip 18 | 10, 10 | A requested at s = c + 1 (after any approach to 13.0), gate s + 37; B requested then (rate 1.5), window B + 6 to B + 10 (s + 43 to s + 47). Row 139 exact fixed-pose samples, level and straight ahead, with the chase stop (4.175): no touch at start distances 1.0–2.5 or 11.5–13.0; B+10 at 3.0, B+8 at 3.5–4.0, B+7 at 4.5–10.5, B+10 at 11.0; without the stop: no touch at 1.0–4.5 or 11.5–13.0, B+8 at 5.0, B+7 at 5.5–10.5, B+10 at 11.0. After a hit the clip finishes at rate 1.0: a hit on B+7 or B+8 gives gate B+33 and end path B+34 (s+71), a hit on B+10 gate B+32 and end path B+33 (s+70); no contact: gate B+25, end path B+26 (s+63). This replaces the old broad reach and first-touch bands; gaps between sampled distances and moving Tor are not tabulated.
| Blocked swing (state 4) | D 36 -> clip 9 (slot 0), D 46 -> clip 11 (slot 2), none for slot 3, D 41 for slot 1 (never blocked) | 12, 10 | blocked on the hit tick; state 4 on the next tick; gates 45, 37; then state 5 and the end path |
| Melee dodge (outer 5) | 55 -> clip 27 | 10 | gate 37, then the defence exit (hold int[120, 300]) |
| Missile dodge (outer 6) | 53 or 54 -> clip 3 / 4 | 13 | gate 49, then the defence exit |
| Hit reaction kind 0 (outer 10) | 57 -> clip 26 | 13 | gate 49 after the last hit, then outer 0 |
| Wait in line (outer 12) | walk 2 -> clip 5 | 30 | stand-off point 15.0 from the hero (SPEC-0038 Behavior 45) |
| Victory pose (state 10) | 73 -> clip 24 | 38 | gate 148, then the default state |
| Dead | 58 -> clip 25 | 56 | plays once |

#### Attack chooser

Each tick while the hero is attackable, recovery is below 1 and the outer sub-state is 1, 3, 4 or 11: slot 3 (0 < d < 8, facing within 22.5 degrees, 99 of 100), then slot 2 (0 < d < 10, within 45 degrees, 59 of 100), then slot 1 (7 < d < 15, 14 of 100), then slot 0 (0 < d < 16, 49 of 100). A route to the hero is required for a slot only when d is 10 or more (a melee slot with no charge passes when a route exists or d < 10). With the Dark Knight facing the hero (the hold and circle states face him), the per-tick chance of starting some attack and which slot: d < 7 about 99.8 percent (slot 3 99 percent); 7 to 8 slot 3 99 percent; 8 to 10 82 percent (slot 2 59, slot 1 6, slot 0 17); 10 to 15 56 percent (slot 1 14, slot 0 42); 15 to 16 slot 0 49 percent. Because there is no cooldown, the next attack starts as soon as recovery (int[60, 180]) ends, at the per-tick chances above. A melee slot with d above its range first walks toward the hero (gait 0, id 28 at 10.0 per second, the step dropped within the sum of the radii 3.0 + 1.175): slot 3 to 7.0, slot 1 to 12.0, slot 0 to 13.0. After a choice the desired distance is redrawn.

#### Movement

Strafe archetype with the settled rules (hold 60 to 180 ticks then a draw of stay, inner 8 (travels to its right), inner 9 (travels to its left); circle with the strafe lock; approach beyond 16; back off below 6). The Dark Knight is slow: approach 10.0 per second, circle 4.2 to 4.5, back off 6.0, wander and patrol 3.0 per second; the hero (13.5 walking, 19.3 running) out-walks every gait of the knight. An approach is steered; a swing's own root travel carries it forward 0.9 to 11.3 per second during the strike clip (slot 2: 11.3 per second, so it closes distance by up to 7 units during its 37 ticks).

Movement table (row 139; ground; radius 3.0; ground turn 0.1; strafe band 6 to 16, desired 8 to 14):

| Mode | Source | Clip (keys) | u/tick | u/s | Turn | Exit |
| --- | --- | --- | --- | --- | --- | --- |
| Wander, patrol, walk home, wait in line | root | 2 or 4 (30) | 0.050 | 3.00 | steer | arrival 0.5, notice |
| Named-target follow (state 9) | root | 11 (30) | 0.027 | 1.60 | steer | distance rule |
| Approach (inner 7), above 16; sequencer state 0; every missing id (20 to 26, the alert 18, the emerge 10) | root | 28 = clip 0 (15) | 0.167 | 10.01 | steer | d below desired |
| Hold (inner 10) | none | 31 (20) | 0 | 0 | aim 0.1 | int[60, 180] ticks, then the roll |
| Circle inner 9 / inner 8 | root, sideways | 29 / 30 (12) | 0.070 / 0.075 | 4.22 / 4.52 | snap to the hero | hold count, blocked, band |
| Back off (inner 11), below 6 | root, facing the hero | 32 (12) | 0.100 backward | 5.98 | steer, back gait | d above desired or blocked |
| Confused (outer 11) | root | 69 (20) | 0.074 | 4.44 | aim at a random direction 0.05 | status end |
| Hit (kinds 0 and 2, heavy 64) | root | 57, 60 or 64 (13) | | back 4.72 over 52 ticks | snap to attacker | gate |
| Dodge 53 / 54, 55 | root | (13), (10) | | side 14.16 over 52; back 5.15 over 40 | | gate |
| Corpse | root | 58 (56) | | back 1.33 over 224 ticks, nothing held | kept | timer 510 |

With turn 0.1 the knight's hold tracks a moving hero slowly (a 90-degree change needs about 22 ticks to come within 10 degrees). Its dodge (14.16 sideways in 52 ticks) is faster than any of its walks. A group alarm makes it play clip 0 (the walk) for 59 ticks before agro, sliding about 10 units forward (SPEC-0038 Behavior 3).
#### Hit and hurt windows

Slot 3: B requested at s = c + 1, weapon 5 (radius 4.0) live from s + 20 to the end of the clip, one hit of 10 per window; slot 2: weapon 1 (radius 6.0) live from s + 16, one hit of 50; slot 1: weapon 1 from s + 48, one hit of 50 plus the curse roll; slot 0: weapon 1 from B + 6 (s + 43) to B + 10, one hit of 20. In every slot the window start is not the hit tick: the hit comes on the first window tick on which the weapon touches a hero capsule, which depends on where the hero stands (first-touch ticks in the state table; slot 0 first touches a level hero on B + 7, not B + 6). A hit on the hero is the strike test of SPEC-0038 Behavior 13 (the hero's capsules against the weapon volume; contact geometry Medium). Hit damage before armour and shield: 10, 50, 50, 20. A guarding hero (L1, facing the Dark Knight) blocks slots 0, 2 and 3 (state 4, no damage) and not slot 1; blocking slot 0 or 2 at 7 < d < 15 opens slot 1 as a counter (chance 50 per tick of the blocked state, which lasts 45 or 37 ticks, so almost certain when the hero holds guard). The Dark Knight is hittable in every state except its dodges (outer 5 and 6) and while dead; hurt shapes (Medium): body capsule length 2.55 radius 1.45 and limb capsules of radius 0.6 to 1.0.

#### Weapon reach

Conventions are those of "Weapon reach tables" in the shared field behaviour. Row 139, Medium. Ticks in this table count from the tick on which the slot's clip is requested (for slot 0 that is B above); the first-touch ticks in the state table are authoritative. The Spirit Sword (rows 140 and 141) has the same figures.

| Slot (phase, clip) | Weapon | Window ticks (level contact) | Centre forward / up / side | Tor ahead, at the tick | Tor ahead, from the start | Tor feet vs origin |
|---|---|---|---|---|---|---|
| 0 (B, clip 18) | w1 (6.0 x 2.0) | s+6 to s+10 (s+7 to s+10) | 3.3 to 4.9 / 1.2 to 4.0 / -3.1 to 2.9 | 3.0 to 9.4 | 4.8 to 11.1 | -7.0 to 9.0 |
| 1 (B, clip 13) | w1 (6.0 x 2.0) | s+48 to s+77 (s+48 to s+77) | 1.4 to 4.4 / 0.1 to 5.2 / -2.4 to -0.6 | -4.0 to 10.8 | -0.8 to 15.1 | -8.5 to 10.0 |
| 2 (B, clip 10) | w1 (6.0 x 2.0) | s+16 to s+37 (s+16 to s+37) | 0.3 to 4.6 / 1.1 to 2.5 / -2.6 to -0.6 | -0.4 to 6.4 | 2.6 to 13.7 | -8.0 to 8.0 |
| 3 (B, clip 16) | w5 (4.0 x 2.0) | s+20 to s+43 (s+20 to s+43) | 0.0 / 1.9 to 2.4 / -0.5 to -0.3 | -4.2 to 4.6 | -0.9 to 8.3 | -4.5 to 6.0 |

Slot 0 never touches a hero closer than 3.0 ahead; slots 1 and 2 reach 10.8 and 6.4 ahead at the tick and 15.1 and 13.7 from where the knight started (the clips carry it forward). The weapon 5 jab of slot 3 stays close to the body (4.2 behind to 4.6 ahead).

#### Defence and counters

Melee: once per hero swing (hold below 1, not in a reaction, not mid-attack except sequencer states 0, 1 and 5, hero within 8.0 and in front), a dodge on 74 of 100 swings of any class (clip 27, 37 ticks, then a hold of 120 to 300 ticks with no further defence). Missile: 74 of 100 per live shot meeting the capsule inside 60 ticks (clips 3 and 4, 49 ticks). The hold after any dodge (120 to 300 ticks) means one dodge per 2 to 5 seconds; swings in that window land. Counters: slot 1 only, trigger 5 (above). The hit reaction does not trigger a counter (no slot has trigger 6).

#### Hero actions against this enemy

| Hero action | Result |
|---|---|
| Stands within 7 units of a patrolling knight | Noticed on the next awareness tick (within 12 ticks); agro on the same tick if the repeat check passes, otherwise within a few cycles |
| Stands at 12 units | About 49 percent per cycle; the knight circles at about 4.4 per second and attacks with slot 0 (20) or slot 1 (50 and curse) every 100 to 270 ticks |
| Closes inside 8 | Slot 3 (10 damage, 77 ticks) at 99 percent per tick when the knight faces within 22.5 degrees; slot 2 (50) and slot 0 (20) at the ranges above |
| Quick, normal or heavy swing inside 8 | Dodged on 74 of 100 swings, then no dodge for 120 to 300 ticks |
| Roberta's shots | Dodged on 74 of 100 (hold 60 ticks) |
| Guards (L1) | Slots 0, 2 and 3 are blocked and may be answered by slot 1; slot 1 hits through the guard |
| A landed hit | Damage (150 hit points, defence value 10; doubled out of agro), reaction 49 ticks that cancels the swing, no counter |
| Alarm group | Group members wake within 45 ticks, slide forward about 10 units during a 59-tick wait, then enter agro |

#### Edge cases for implementers

- Dark Knights are never hidden and never use state 21. A port that gives them the Minion decloak is wrong; a port that gives them the Brigands' alert clip is wrong too: id 18 is absent, so a noticed knight is in agro on the tick of the second pass.
- All four slot cooldowns draw 0 to 0, but the recovery int[60, 180] and the choice rolls decide the pace; slot 3 at 99 percent dominates inside 8 units, which makes the fastest and weakest swing (10 damage, 77 ticks) the common one.
- The swing clips carry strong root travel (slot 2 moves the knight forward about 7 units), so a knight that chose slot 2 at 9 units can end inside the hero's position; the chase-stop (step dropped within the sum of the radii) applies only in sequencer states 0 to 3.
- Weapon 1 has radius 6.0: slot 2 and slot 1 hit from farther than the nominal range of 12 to 15 units suggests only if the swing clip's weapon moves that far; use the melee ranges for the approach and the volume for the hit.
- The pack of twelve in pack 1098 wanders over a 90-unit square without pausing and socializes; the Market Town knights patrol 10 different paths with pauses of 50 to 300 ticks.
- Default 0 (one record): idle until hit or alarmed.

#### What a port most likely gets wrong

1. Treating it as one of the hidden minions: it is visible and hittable from the start and has real search and alert behaviour.
2. Playing an alert clip that does not exist: the second notice pass goes to agro immediately.
3. Using the slot order as a priority list from slot 0: the order is 3 first, and slot 3 (10 damage) is the usual swing at close range.
4. Giving the knight cooldowns: none exist; only recovery and the rolls.
5. Missing the unblockable curse of slot 1 and its counter use after a blocked swing.
6. Forgetting the 74 percent dodge with a 120 to 300 tick quiet period after each dodge.
7. Giving it a fast gait: approach 10.0, circle 4.4, back off 6.0 per second.

#### Test vector

Dark Knight in agro (t = 0 the tick the setter ran), hero standing still 12 units in front, hero guard off. t = 1 set-up. From t = 2 the per-tick chance of starting an attack is 56 percent (slot 1 14 percent, slot 0 42 percent), so the first choice is at t = 2 with 56 percent, t = 3 with 25 percent, later otherwise (mean 2.8). Slot 0 at c: d = 12 is inside its range 13, so s = c + 1 with no approach, A to s + 37 (A has no root travel), B requested at c + 38. The exact row-139 query misses at fixed d=12; at d=10 it touches at B+7 (c+45). Slot 1 at c: window from c + 49, exact row-139 stationary level contact at d=12 is c+51 (50 damage and curse on 75 of 101 draws), end path c + 79. Recovery int[60, 180] follows the end path, then the next choice at 56 percent per tick, so one attack every 130 to 270 ticks. In 600 ticks: 3 to 5 attacks (mean 3.7: about 2.8 of slot 0 and 0.9 of slot 1): at 12 units only the slot-1 swings land, about 0.9 x 50 = 45 damage on a hero who never moves (about 100 at 10 units, where slot 0 lands too), and a 74 percent curse on each slot-1 hit. Over the same window a hero who strikes it is dodged on 74 of 100 swings.

#### Open questions

- Weapon contact geometry (volume anchor and the hero's capsule pose). Resolved by the general rule of the shared field behaviour. The distances are in the Weapon reach table. Stand-in only for a port without attachment frames (rewrite choice): a window hits when a level hero is inside the "at the tick" range of the slot's row (4.2 behind to 4.6 ahead for weapon 5, and for weapon 1 3.0 to 9.6, -4.0 to 10.8 or -0.4 to 6.4 ahead by slot).
- Whether the group-alarm wait (clip 0 as id 18) ends at that clip's 59-tick gate with the full forward slide: Resolved (High for the length, Medium-High for the slide; proactive (open close-out)). The animation request stores the requested id (18) as the current animation even when the table has no such id, so the state-2 handler sees "alert playing", does not request the look clip, and waits for the progress gate of the clip that actually plays (clip 0, 59 ticks) before entering agro. The clip's root travel moves the knight by the ordinary root-motion rule, about 10 units forward over the wait.
- Which wait clip a Dark Knight with a nonzero +0xC4 sentry plays (it holds the placed facing): Resolved (Medium-High; proactive (open close-out)). The shared sentry state applies: it aims at the stored look point every tick and plays animation 7, the stand clip 30 (the fidget animation 8 is not used while the flag is nonzero). Two Dark Knight placements use it (one in Black Keep Grounds, one in Market Town).



## Distances and ranged reach (every kind of this spec)

Confidence: High for the row values and the rules (SPEC-0038 Behaviors 3, 4, 7, 13, 23, 44 and 45; SPEC-0029 Behaviors 9, 13 and 13a); Medium-High for the emit points (posed strike clip of the most common record group, root travel included); Medium for the hit ticks of arcs (flat ground, hero standing still, feet level with the enemy's ground). The melee weapon reach is in each kind's own section and is not repeated here. The Rogue, the Skeleton Fighter and the Ixi Henchman row 160 are in SPEC-0063.

**Reading the tables.** d is the distance from the enemy to the selected hero, as the agro machine measures it. The notice band is the row's near and far range; a hero standing still scales both by 0.7 and a running hero by 1.75 (SPEC-0038 Behavior 18); through a wall the band is scaled by F2 instead (0 for a standing hero) unless the row sees through walls. The sight cone only halves the notice chance outside it; it is not a hard edge. "Walk-in target" is the drawn desired distance (min + (max - min) x f, f from 0.2 to 0.8), redrawn after every normal attack choice; "back-off point" is the point that far from the hero on the hero-to-enemy line. A slot is chosen only while its band holds strictly and the hero is inside its arc; a melee slot then walks in until d is at most its close-in distance. "Last seen + x" is the lost-sight search point moved x further along the enemy-to-hero line. Shared by every kind and not repeated: hold and circle sub-states face the hero every tick with the row's turn weight (there is no separate facing distance); the strafe circle steers to a point 2.0 to the side every tick, so it keeps roughly the current d and drifts outward until d passes the band's top; the only leash is 181 ticks with no route to the hero while the walk waypoint is 15.0 or more from the hero (an enemy with a route never gives up by distance); the lost-sight search gives up after 121 ticks at its point; streaming releases an enemy beyond 280; a melee slot with no charge and no route is still allowed below d = 10.0; the chase stop is the enemy's radius + 1.175 (the hero's body radius, both heroes); a confused enemy may attack at radius + 1.425; the wait-in-line square is the side of the square around the hero in which a held-back enemy picks its stand-off point (later picks use 50), and keep-distance kinds are never held back. Hover heights are above the hero's feet (Can Fly 1: min(max fly height, 0.2 d), at least ground + 2; Can Fly 2: the hero's own height).

**Ranged reach.** s is the tick the strike clip B is requested (c + 1 when the slot has no wind-up clip; after the wind-up gate otherwise, see each kind's timing). The emit point is given on the release tick as side, up and forward from the enemy's feet in its own frame (side is the model's x axis), root travel of the clip included; it turns with the enemy. Every shot aims once, at release, from the emit point at the hero's feet + (0, 2.5, 0) for straight and homing shots or at the feet for arcs, plus the row's scatter; there is no lead. The hit volume is the shot's sphere against the hero's body cylinder (radius 1.175, height 3.5, the same for Tor and Roberta), tested only at the positions after each move (SPEC-0029 Behavior 13a). Every shot is stopped by the hero, a facing guard, the ground and the world once 2.0 from its start, and its lifetime; other enemies never stop it. "Reach" is the farthest standing hero (feet level, straight in front, scatter 0) that is still hit. The arcs' area words never deal damage (SPEC-0029 Behavior 9, Area mode), so no kind of this spec has a working splash. Worked examples: hero standing still at distance D, scatter 0, enemy facing the hero.

| Kind, row, slot | Fires when | Release tick; emit point (side, up, forward) | Flight | Lifetime; travel; reach | Radius | Hero at D: tick of the hit |
|---|---|---|---|---|---|---|
| Minion Archer 190, 191, slot 3 | 10 < d < 100, in front, airborne; holds 15 to 60 | s + 56; (-0.61, 3.90, -1.00), the Archer hovering min(15, 0.2 d) above the hero | straight 2.0 per tick | 35; 72; 70.5 | 0.05 | 12: s + 62; 20: s + 66; 30: s + 71; 50: s + 81; 70: s + 91 |
| Minion Archer 192 to 196, slot 3 | as above | s + 44; (-0.27, 3.74, -0.45) | as above | 35; 72; 71 | 0.05 | 30: s + 59 |
| Minion Wizard 211, slot 0 (lobbed summon) | 10 < d < 60, in front, fewer than 4 live Grunts | s + 50; (0.28, 4.42, 0.68), the Wizard at the hero's height | arc at 1.0 per tick, aimed at the feet, touches no one | lands when its centre is 1.0 above the ground | 1.0 (no hit test) | lands and binds a Grunt: 15: s + 61, 12.7 from the Wizard; 30: s + 75, 26.7 from the Wizard |
| Minion Wizard 211, slot 1 (four fire lobs) | 10 < d < 60, in front | s + 110, 114, 118, 122; (-2.33, 8.48, -0.29) | arc, speed 4.0 at d 60 ramping to 0.04 at d 10; scatter U[0, 10] | lands | 0.1 | first lob, scatter 0: 12: s + 136; 20: s + 122; 30, 45 and 59: lands at the hero's feet (s + 121, 122, 124) without touching, because one step is longer than the cylinder |
| Minion Wizard 212, slot 0 | 0 < d < 100, within 7.5 deg | s + 110; (0.12, 5.94, 8.10), the clip carries the Wizard 3.98 forward | straight 3.0 | 33; 102; 111.5 | 0.5 | 10: s + 110; 30: s + 116; 60: s + 126 |
| Minion Wizard 212, slot 1 (poison orb) | 0 < d < 100, in front | s + 156; (-1.11, 7.90, 3.38) | straight 0.06 per tick | 1666; 100; 105.75 | 1.5 | 10: s + 248; 30: s + 563 |
| Minion Wizard 213, slot 1 (five homing shots) | 0 < d < 100, in front | s + 22, 26, 30, 34, 38; emit point 1 (0.87, 4.81, 4.06) | homing: launch 1.0, then 0.75 old direction + 0.25 toward the hero's feet + 2.0 each tick (speed at most 1.0) | 200; at most 200; 206 | 0.5 | first shot: 15: s + 31; 30: s + 46; 60: s + 76 |
| Minion Wizard 214, 216, slot 1 | 0 < d < 100, in front | s + 110; (0.12, 5.94, 8.10) | straight 2.0 | 50; 102; 111.5 | 0.5 | 15: s + 112; 30: s + 120; 60: s + 135 |
| Minion Wizard 215, slot 1 (four slow lobs) | 10 < d < 60, in front | s + 110, 114, 118, 122; (-2.33, 8.48, -0.29) | arc, 0.5 at d 60 ramping to 0.02 at d 10; scatter U[0, 10] | lands | 0 | never touches (radius 0); lands 12: s + 295; 20: s + 211; 30: s + 200; 59: s + 227 |
| Minion Magician 197 to 199, 201, 202, slot 0 | 10 < d < 60, in front | s + 136; (-1.36, 7.05, 1.05) | straight 3.0 | 50; 153; 155.5 | 0.5 | 15: s + 140; 25: s + 143; 30: s + 145; 59: s + 154 |
| Minion Magician 200, 202, slot 1 | 10 < d < 60, in front | s + 4 (s after the 38-key wind-up); (-0.82, 5.00, 1.84) | straight 3.0 | 50; 153; 156 | 0.5 | 15: s + 7; 30: s + 12; 59: s + 22 |
| Minion Artillery 162 to 167, slot 0 (ten shells) | 15 < d < 100, in front (aims at weight 1.0) | s + 8, 16, 28, 40, 56, 68, 84, 96, 108, 120; (0, 1.73 to 1.92, 0.76 to 1.31) | straight 1.3 | 84; 110.5; 112.5 | 0.1 | each shell, after its own release: 16: + 10; 30: + 21; 50: + 36; 60: + 44; 99: + 74 |
| Flower Monster 149 (spitter), slot 0 | 3 < d < 60, within 7.5 deg | s + 24; (0, 1.17, 0) | arc, 0.1 at d 3 ramping to 0.5 at d 60 | lands | 1.0 | 4: s + 40; 10: s + 64; 20: s + 82; 40: s + 107; 59: s + 137 |
| Poison Flower Monster 148, slot 3 | 15 < d < 30, in front | s + 36; emit point 2 (0, 1.72, 2.91) | arc, 0.7 at d 15 ramping to 1.0 at d 30 | lands | 0.2 | 16: s + 54; 20: s + 56; 29: lands 1.6 short on s + 60 (miss) |
| Spore Monster 221, slot 3 (lobbed summon) | 10 < d < 60, in front, fewer than 5 live seeds | s + 4; (-1.77, 5.38, 3.06) | arc at 0.5 per tick, touches no one | lands when its centre is 1.0 above the ground | 1.0 (no hit test) | lands and binds a spitter: 11: s + 17 at 9.9; 20: s + 34 at 18.5; 40: s + 74 at 38.5; 59: s + 113 at 58.0 from the Spore Monster |
| Plant Gunner 220, slot 2 (four shots) | 5 < d < 40, in front | s + 27, 55, 87, 119 (s after the wind-up); first (-4.02, 5.53, 7.41): the clip carries the plant 3.15 forward and moves the emit point between shots (up to 13.4 forward) | straight 2.0 | 30; 62; 70.5 (first shot) | 0.2 | first shot: 6: s + 28; 15: s + 30; 30: s + 37; 39: s + 42 |
| Thief 152, slot 1 | 5 < d < 50, within 15 deg | s + 48 (s after the wind-up); (-0.72, 3.51, 1.83) | straight 1.2 | 62; 75.6; 78.75 | 0.2 | 6: s + 50; 16: s + 58; 20: s + 62; 49: s + 86 |
| Skeleton Archer 224, slot 0 | 0 < d < 75, in front | s + 50; (-0.80, 1.37, 1.65) | straight 2.0 | 50; 102; 104.75 | 0.1 | 5: s + 51; 20: s + 58; 25: s + 61; 40: s + 68; 74: s + 85 |
| Skeleton Wizard 225, slot 0 | 0 < d < 25, in front | s + 27; (-0.83, 1.64, 1.68) | straight 1.0 | 100; 101; 103.75 | 0.1 | 5: s + 29; 15: s + 39; 25: s + 49 |

Slots that fire nothing: Skeleton Wizard slot 1 (chance 0); the Archer's slot 2 pose (no shot, no damage).

**Grabs: the real catch volume (Minion Brute slot 1, Thug slot 3, Plant Monster slot 0).** A grab slot has no weapon test. On the first tick of sequencer state 2 (s + 1) the grab machine makes one test: the horizontal distance from the enemy to the hero must be below the slot's close-in distance, the dot of the enemy's facing with the direction to the hero above 0.5 (within 60 degrees), the hero's stun word clear, the hero's motion record not touched and the straight path clear. That single tick decides: a failure ends the grab for this attack (no retry while the clip plays). The catch volume is therefore a 60-degree sector of a flat disc around the enemy, of radius 12.0 for the Minion Brute, 6.0 for the Thug and 9.0 for the Plant Monster, with no height limit and no weapon cylinder. The hero is held where he stands (his position is kept as the last good position, not moved to the enemy's hand; Medium), and the throw (Brute 17.0, Thug 15.0, Plant Monster 15.0) starts from there. Worked example: a Brute chooses slot 1 at d = 15, walks in until d is at most 12 and requests B on s (it has no wind-up); a hero still standing 11.9 away and within 60 degrees of its facing on s + 1 is caught; one at 12.1, or 61 degrees off, is not, and the Brute plays its recovery clip with no contact (end path s + 80).

#### Minion Grunt

| Distance (units) | Value |
|---|---|
| Notice band (x0.7 standing, x1.75 running) | 0 to 20 (0 to 14; 0 to 35) |
| Sight cone half-angle | 60 deg |
| Rush: hold band; walk-in target; back-off point | 8 to 15; 9.4 to 13.6; 17 |
| Slot 1 | melee, 0 < d < 15, within 15 deg, closes to 5, only while airborne |
| Slot 2 | rows 183 to 188: melee, 1 < d < 20, within 15 deg, closes to 5, only while airborne; row 189: melee, 0 < d < 30, within 15 deg, closes to 8, only while airborne |
| Lost-sight search point | last seen |
| Wait-in-line square | 25 |
| Hover height above the hero | min(10, 0.2 d), floor ground + 2 |
| Chase stop; confused strike | 3.17; 3.42 |

#### Minion Archer

| Distance (units) | Value |
|---|---|
| Notice band (x0.7 standing, x1.75 running) | row 190: 5 to 80 (3.5 to 56; 8.75 to 140); rows 191 to 196: 10 to 100 (7 to 70; 17.5 to 175); sees through walls |
| Sight cone half-angle | 60 deg |
| Keep distance: hold band; walk-in target; back-off point | 15 to 60; 24 to 51; 60 |
| Retreat: below; to point; stops at | none (0 to 0) |
| Slot 2 | melee pose, 15 < d < 100, within 15 deg, closes to 25, only while airborne |
| Slot 3 | missile, 10 < d < 100, in front, only while airborne |
| Lost-sight search point | last seen + 15 |
| Hover height above the hero | min(15, 0.2 d), floor ground + 2 |
| Chase stop; confused strike | 3.17; 3.42 |

#### Minion Brute

| Distance (units) | Value |
|---|---|
| Notice band (x0.7 standing, x1.75 running) | 5 to 20 (3.5 to 14; 8.75 to 35) |
| Sight cone half-angle | 22.5 deg |
| Rush: hold band; walk-in target; back-off point | 9 to 17; 10.6 to 15.4; 19 |
| Slot 0 | melee, 4 < d < 20, within 22.5 deg, closes to 10 |
| Slot 1 | grab, 10 < d < 20, within 15 deg, closes to 12 (catch below 12, within 60 deg) |
| Slot 2 | melee, 5 < d < 30, within 45 deg, closes to 10 |
| Slot 3 | melee, 0 < d < 20, in front, closes to 20 |
| Lost-sight search point | last seen |
| Wait-in-line square | 30 |
| Chase stop; confused strike | 5.17; 5.42 |

#### Minion Warrior

| Distance (units) | Value |
|---|---|
| Notice band (x0.7 standing, x1.75 running) | 10 to 30 (7 to 21; 17.5 to 52.5) |
| Sight cone half-angle | 22.5 deg |
| Strafe: circle band; walk-in target; back-off point | 12 to 20; 13.6 to 18.4; 20 |
| Slot 0 | melee, 3 < d < 10, within 22.5 deg, closes to 4 |
| Slot 1 | melee, 10 < d < 17, within 15 deg, charge run 8 to 20 (ends early when d <= 8) |
| Slot 2 | melee, 3 < d < 10, within 45 deg, closes to 4 |
| Lost-sight search point | last seen + 12 |
| Wait-in-line square | 10 |
| Chase stop; confused strike | 2.42; 2.67 |

#### Minion Wizard

| Distance (units) | Value |
|---|---|
| Notice band (x0.7 standing, x1.75 running) | 10 to 20 (7 to 14; 17.5 to 35) |
| Sight cone half-angle | 22.5 deg |
| Keep distance: hold band; walk-in target; back-off point | 25 to 60; 32 to 53; 60 |
| Retreat: below; to point; stops at | 20; 27; 20 to 25 |
| Slot 0 | row 211: missile, 10 < d < 60, in front; row 212: missile, 0 < d < 100, within 7.5 deg; rows 213 to 216: none |
| Slot 1 | rows 211, 215: missile, 10 < d < 60, in front; rows 212, 213, 214, 216: missile, 0 < d < 100, in front |
| Lost-sight search point | last seen + 25 |
| Hover height above the hero | 0 (cap ground + 20) |
| Map search distance; greedy range | row 213: 200; 0; row 216: 120; 0; other rows: none |
| Chase stop; confused strike | 2.92; 3.17 |

#### Minion Magician

| Distance (units) | Value |
|---|---|
| Notice band (x0.7 standing, x1.75 running) | 3.5 to 50 (2.45 to 35; 6.12 to 87.5); sees through walls |
| Sight cone half-angle | 45 deg |
| Keep distance: hold band; walk-in target; back-off point | 15 to 40; 20 to 35; 40 |
| Retreat: below; to point; stops at | 15; 37; 15 to 35 |
| Slot 0 | rows 197, 198, 199, 201, 202: missile, 10 < d < 60, in front; row 200: none |
| Slot 1 | rows 200, 202: missile, 10 < d < 60, in front; other rows: none |
| Lost-sight search point | last seen + 15 |
| Hover height above the hero | 0 (cap ground + 20) |
| Map search distance; greedy range | rows 197, 198, 199, 201: 50; 0; rows 200, 202: none |
| Chase stop; confused strike | 3.67; 3.92 |

#### Minion Artillery

| Distance (units) | Value |
|---|---|
| Notice band (x0.7 standing, x1.75 running) | 3.5 to 200 (2.45 to 140; 6.12 to 350) |
| Sight cone half-angle | 22.5 deg |
| Keep distance: hold band; walk-in target; back-off point | 40 to 75; 47 to 68; 75 |
| Retreat: below; to point; stops at | 25; 102; 25 to 100 |
| Slot 0 | missile, 15 < d < 100, in front |
| Slot 1 | melee, 15 < d < 40, within 15 deg, charge run 30 to 50 (ends early when d <= 10) |
| Lost-sight search point | last seen + 40 |
| Chase stop; confused strike | 3.17; 3.42 |

#### Dark Knight

| Distance (units) | Value |
|---|---|
| Notice band (x0.7 standing, x1.75 running) | 10 to 50 (7 to 35; 17.5 to 87.5) |
| Sight cone half-angle | 22.5 deg |
| Strafe: circle band; walk-in target; back-off point | 6 to 16; 8 to 14; 16 |
| Slot 0 | melee, 0 < d < 16, within 45 deg, closes to 13 |
| Slot 1 | melee, 7 < d < 15, within 45 deg, closes to 12 |
| Slot 2 | melee, 0 < d < 10, within 45 deg, closes to 15 |
| Slot 3 | melee, 0 < d < 8, within 22.5 deg, closes to 7 |
| Lost-sight search point | last seen + 6 |
| Wait-in-line square | 15 |
| Chase stop; confused strike | 4.17; 4.42 |

#### Flower Monster

| Distance (units) | Value |
|---|---|
| Notice band (x0.7 standing, x1.75 running) | row 147: 3.5 to 50 (2.45 to 35; 6.12 to 87.5); row 149: 1 to 10 (0.7 to 7; 1.75 to 17.5), sees through walls |
| Sight cone half-angle | 45 deg |
| Spacing | row 147 rush: hold 4 to 17; walk-in target 6.6 to 14.4; back-off point 19. Row 149 keep distance: hold 3 to 35; walk-in target 9.4 to 28.6; back-off point 35; no retreat |
| Slot 0 | row 147: never chosen (band 0 to 0, chance 0); row 149: missile, 3 < d < 60, within 7.5 deg |
| Slot 1 | row 147: melee, 0 < d < 20, within 7.5 deg, closes to 10; row 149: none |
| Slot 2 | row 147: melee, 0 < d < 9, in front, closes to 7; row 149: none |
| Lost-sight search point | row 147: last seen; row 149: last seen + 3 |
| Wait-in-line square | row 147: 10; row 149: never waits |
| Chase stop; confused strike | row 147: 3.67; 3.92; row 149: 2.17; 2.42 |

#### Poison Flower Monster

| Distance (units) | Value |
|---|---|
| Notice band (x0.7 standing, x1.75 running) | 3.5 to 50 (2.45 to 35; 6.12 to 87.5) |
| Sight cone half-angle | 45 deg |
| Rush: hold band; walk-in target; back-off point | 4 to 17; 6.6 to 14.4; 19 |
| Slot 0 | melee, 8 < d < 17, within 22.5 deg, closes to 13 |
| Slot 1 | melee, 0 < d < 20, within 7.5 deg, closes to 10 |
| Slot 2 | melee, 0 < d < 9, in front, closes to 7 |
| Slot 3 | missile, 15 < d < 30, in front |
| Lost-sight search point | last seen |
| Wait-in-line square | 10 |
| Chase stop; confused strike | 3.67; 3.92 |

#### Spore Monster

| Distance (units) | Value |
|---|---|
| Notice band (x0.7 standing, x1.75 running) | 5 to 120 (3.5 to 84; 8.75 to 210) |
| Sight cone half-angle | 22.5 deg |
| Rush: hold band; walk-in target; back-off point | 10 to 30; 14 to 26; 32 |
| Slot 1 | melee, 0 < d < 15, within 22.5 deg, closes to 7 |
| Slot 3 | missile (lobbed summon), 10 < d < 60, in front |
| Lost-sight search point | last seen |
| Wait-in-line square | 20 |
| Chase stop; confused strike | 4.67; 4.92 |

#### Plant Gunner

| Distance (units) | Value |
|---|---|
| Notice band (x0.7 standing, x1.75 running) | 5 to 100 (3.5 to 70; 8.75 to 175) |
| Sight cone half-angle | 22.5 deg |
| Keep distance: hold band; walk-in target; back-off point | 10 to 20; 12 to 18; 20 |
| Retreat: below; to point; stops at | 20; 32; 20 to 30 (the retreat minimum is not below the hold maximum, so it paces between 20 and its retreat draw) |
| Slot 2 | missile, 5 < d < 40, in front |
| Lost-sight search point | last seen + 10 |
| Chase stop; confused strike | 4.67; 4.92 |

#### Plant Monster

| Distance (units) | Value |
|---|---|
| Notice band (x0.7 standing, x1.75 running) | 5 to 100 (3.5 to 70; 8.75 to 175) |
| Sight cone half-angle | 22.5 deg |
| Rush: hold band; walk-in target; back-off point | 3 to 10; 4.4 to 8.6; 12 |
| Slot 0 | grab, 0 < d < 20, in front, closes to 9 (catch below 9, within 60 deg) |
| Slot 1 | melee, 0 < d < 10, within 22.5 deg, closes to 7 |
| Lost-sight search point | last seen |
| Wait-in-line square | 30 |
| Chase stop; confused strike | 4.67; 4.92 |

#### Thief

| Distance (units) | Value |
|---|---|
| Notice band (x0.7 standing, x1.75 running) | 3 to 30 (2.1 to 21; 5.25 to 52.5) |
| Sight cone half-angle | 22.5 deg |
| Strafe: circle band; walk-in target; back-off point | 6 to 16; 8 to 14; 16 |
| Slot 0 | melee, 0 < d < 10, within 22.5 deg, closes to 5 |
| Slot 1 | missile, 5 < d < 50, within 15 deg |
| Slot 2 | melee, 5 < d < 20, within 22.5 deg, charge run 15 to 25 (ends early when d <= 5) |
| Lost-sight search point | last seen + 6 |
| Wait-in-line square | 10 |
| Map search distance; greedy range | 100; 10 |
| Chase stop; confused strike | 3.17; 3.42 |

#### Bandit

| Distance (units) | Value |
|---|---|
| Notice band (x0.7 standing, x1.75 running) | 1 to 30 (0.7 to 21; 1.75 to 52.5) |
| Sight cone half-angle | 45 deg |
| Rush: hold band; walk-in target; back-off point | 3 to 6; 3.6 to 5.4; 8 |
| Slot 0 | melee, 0 < d < 10, within 22.5 deg, closes to 7 |
| Slot 1 | melee, 0 < d < 6, within 22.5 deg, closes to 5 |
| Slot 2 | melee, 0 < d < 15, within 22.5 deg, closes to 7 |
| Lost-sight search point | last seen |
| Wait-in-line square | 20 |
| Chase stop; confused strike | 3.17; 3.42 |

#### Thug

| Distance (units) | Value |
|---|---|
| Notice band (x0.7 standing, x1.75 running) | 5 to 40 (3.5 to 28; 8.75 to 70) |
| Sight cone half-angle | 45 deg |
| Rush: hold band; walk-in target; back-off point | 3 to 6; 3.6 to 5.4; 8 |
| Slot 0 | melee, 0 < d < 10, within 22.5 deg, closes to 7 |
| Slot 1 | melee, 0 < d < 6, within 22.5 deg, closes to 5 |
| Slot 2 | melee, 0 < d < 15, within 22.5 deg, closes to 7 |
| Slot 3 | grab, 0 < d < 15, within 22.5 deg, closes to 6 (catch below 6, within 60 deg) |
| Lost-sight search point | last seen |
| Wait-in-line square | 20 |
| Chase stop; confused strike | 3.67; 3.92 |

#### Drackonack

| Distance (units) | Value |
|---|---|
| Notice band (x0.7 standing, x1.75 running) | 3 to 50 (2.1 to 35; 5.25 to 87.5) |
| Sight cone half-angle | 22.5 deg |
| Strafe: circle band; walk-in target; back-off point | 4 to 10; 5.2 to 8.8; 10 |
| Slot 0 | melee, 0 < d < 10, within 15 deg, charge run 5 to 15 (ends early when d <= 5) |
| Slot 1 | melee, 0 < d < 15, within 15 deg, charge run 5 to 15 (ends early when d <= 5) |
| Lost-sight search point | last seen + 4 |
| Wait-in-line square | 15 |
| Chase stop; confused strike | 2.67; 2.92 |

#### Giant Drackonack

| Distance (units) | Value |
|---|---|
| Notice band (x0.7 standing, x1.75 running) | 3 to 50 (2.1 to 35; 5.25 to 87.5) |
| Sight cone half-angle | 22.5 deg |
| Rush: hold band; walk-in target; back-off point | 5 to 15; 7 to 13; 17 |
| Slot 0 | melee, 0 < d < 10, within 15 deg, charge run 5 to 15 (ends early when d <= 5) |
| Slot 1 | melee, 0 < d < 25, within 15 deg, charge run 5 to 15 (ends early when d <= 5) |
| Lost-sight search point | last seen |
| Wait-in-line square | 15 |
| Chase stop; confused strike | 4.67; 4.92 |

#### Dumb Skeleton

| Distance (units) | Value |
|---|---|
| Notice band (x0.7 standing, x1.75 running) | 3 to 5 (2.1 to 3.5; 5.25 to 8.75) |
| Sight cone half-angle | 22.5 deg |
| Rush: hold band; walk-in target; back-off point | 3.5 to 5; 3.8 to 4.7; 7 |
| Slot 0 | melee, 0 < d < 10, within 22.5 deg, closes to 7 |
| Slot 1 | melee, 0 < d < 10, within 45 deg, closes to 7 |
| Lost-sight search point | last seen |
| Wait-in-line square | 12 |
| Chase stop; confused strike | 3.17; 3.42 |

#### Skeleton Archer

| Distance (units) | Value |
|---|---|
| Notice band (x0.7 standing, x1.75 running) | 10 to 120 (7 to 84; 17.5 to 210) |
| Sight cone half-angle | 90 deg |
| Keep distance: hold band; walk-in target; back-off point | 15 to 25; 17 to 23; 25 |
| Retreat: below; to point; stops at | 20; 32; 20 to 30 |
| Slot 0 | missile, 0 < d < 75, in front |
| Lost-sight search point | last seen + 15 |
| Chase stop; confused strike | 3.17; 3.42 |

#### Skeleton Wizard

| Distance (units) | Value |
|---|---|
| Notice band (x0.7 standing, x1.75 running) | 10 to 120 (7 to 84; 17.5 to 210) |
| Sight cone half-angle | 90 deg |
| Keep distance: hold band; walk-in target; back-off point | 15 to 30; 18 to 27; 30 |
| Retreat: below; to point; stops at | 20; 42; 20 to 40 |
| Slot 0 | missile, 0 < d < 25, in front |
| Slot 1 | never chosen (chance 0); band 0 < d < 75 |
| Lost-sight search point | last seen + 15 |
| Map search distance; greedy range | 120; 0 |
| Chase stop; confused strike | 3.17; 3.42 |

**What a port most likely gets wrong (distances).**
1. Firing from the enemy's centre or a fixed offset. The emit point is a posed attachment frame and can be 8 units in front (Minion Wizard rows 212, 214, 216, carried by the clip's root travel), 13 units in front (Plant Gunner's later shots) or 4 units behind and 8 up (Skull Spirit, SPEC-0065); hit ticks move by up to 5 ticks.
2. Testing shots against the bone capsules, or sweeping them. The pool tests the body cylinder (1.175 by 3.5) at the post-move positions only, so the Wizard's fast fire lob passes through a hero 25 or more units away.
3. Giving the arcs a burning or slowing patch. The area pulse never fires (SPEC-0029 Behavior 9, Area mode); only a direct hit hurts.
4. Counting the first move on the tick after the spawn. The first move is in the spawn tick (hit on T0 + n - 1).
5. Adding a leash radius or a give-up distance. Only the no-route counter (waypoint 15.0 from the hero, 181 ticks), the lost-sight search and streaming at 280 end a chase.
6. Making the view cone a hard edge, or the grab a weapon test: the cone halves a chance; the grab is a one-tick 60-degree disc test.

## Test vectors

Every kind has its own worked timeline and numbers in its section. These cross-kind vectors check the shared rules:

1. **Emerge threshold.** A state-8 enemy with emerge distance 40.0 wakes for a standing hero (F1 0.7) closer than 28.0, a slowly walking hero (0.75) closer than 30.0, a walking hero (1.5) closer than 60.0 and a running hero (1.75) closer than 70.0. An emerge distance of 0.0 never wakes by distance, whatever the hero does.
2. **Clip gate.** A one-shot clip of 18 keys at speed byte 2 and play rate 1.0 passes its gate 4 x 17 + 1 = 69 ticks after the request (the Minion Grunt's clip 81 is this clip). A clip of 30 keys passes it after 117 ticks.
3. **Notice chance.** A Dark Knight or Spirit Sword (awareness 100, near range 10 with chance 50, far range 50 with chance 40, cycle 12 ticks) notices a still hero 20 units away, in its view cone, on 0.47 of its cycle ticks. Outside the cone the chance is halved. A hero inside 7.0 units (10 x 0.7) is noticed on every cycle tick.
4. **Two-stage notice.** An Ixi Spearman (row 159) that passes the notice check at tick t0 is in agro on that tick with the probability of the check (the repeat is made on the same tick and no Ixi table has an alert clip). If the repeat fails it stands in the search state for a countdown of 60 to 239 ticks plus the 57-tick search-end clip, tested once per 32-tick cycle, so it gives up 117 to 296 ticks after the first pass if none of those tests passes.
5. **Alarm with no alert clip.** A group alarm sent to an Ixi Spearman in state 5 reaches it within 15 to 45 ticks, plays clip 0 (a swing) for 57 ticks and then the Spearman is in agro with no notice roll. The same alarm sent to an Ixi Archer waits 79 ticks, and to a Dark Knight 59 ticks (sliding about 10 units forward). A notice pass on the same rows never waits: it goes to agro on the pass tick.
6. **Hidden decloak.** A Minion Grunt (Display in Default 0) in state 5 that passes the notice check enters state 21 on that tick. If the repeat check on the same tick passes, clip 82 (17 keys, gate 65) is requested at once and the Grunt is in agro at the tick the gate passes; if it fails, the look clip 16 starts and the 17 clip is committed, and a later cycle pass restarts the chain with clip 82.
7. **Agro idle pick.** On the first agro tick (outer sub-state 0) a Can Fly 2 row (Minion Wizard, Magician, Restless Spirit, Death Knight) requests id 141 and a ground row requests id 1; a table with no entry for the id plays clip 0.
8. **Grab.** A grab that seizes the hero deals 0 damage on every row; the Thug's grab on a throw tick takes a steal roll of 50 of 101 draws for 20 to 200 Neopoints.

## Open questions

Each kind's open questions are in its own section ("Open questions" under the kind). The questions that apply to every kind:

- **Weapon and hurt-shape contact geometry.** Resolved: the general rule of the shared field behaviour (weapon cylinders on the attachment frames against the hero's bone capsules; no arc or facing test at contact). The per-kind "Weapon reach" tables give the distances for each kind. Stand-in only for a port without attachment frames (rewrite choice): a weapon window hits when a level hero is inside the "at the tick" range of the slot's row.
- **Facing of a sentry with a nonzero flag.** The look point is taken from the placed facing. Default if unresolved (rewrite choice): the facing stored in the spawn record.
- **Length of the clip-0 wait for an alarm with a missing alert clip.** The wait is the gate of clip 0 of each kind's table; whether the slide of the knight rig is played as the clip's root travel is Medium. Default if unresolved (rewrite choice): play clip 0 with its root travel.
- **Streaming of enemies the hero never sees.** Release at 280 units and rebinding in the default state are applied to every enemy; whether level scripts rewrite the default state of some default-0 placements is not settled. Default if unresolved (rewrite choice): leave them idle until hit or alarmed.
- **Order of the shared bookkeeping on the exact tick of a state change** (setter, pre-step, handler in one tick) can move a kind's timeline by one tick; the sections give their offsets to plus or minus one tick.
