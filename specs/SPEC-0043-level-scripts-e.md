# SPEC-0043: Level scripts E

- Answers: R-0060, R-0062, R-0100, R-0103 (1398 reordered thread-6 region table), R-0104 (pack 956 aim-target activation is the same with or without item 36), proactive (level script behavior; extends SPEC-0030, SPEC-0032, SPEC-0037, SPEC-0039, SPEC-0041)
- Status: Draft
- Disc build: SCUS-97367 v1.00 (same fingerprint as SPEC-0001)
- Confidence: per module (see the Confidence line in each section and the table below). Host call names and argument values: High unless marked. Branch conditions: Medium or Low as marked. Unresolved values: marked unknown and listed under Open questions. Statements about which host calls a branch makes are Medium unless marked otherwise.

## Summary

This spec describes the level script modules of 15 world packs: 956, 1008, 1011, 752, 1020, 749, 1398, 1066, 748, 1014, 1335, 750, 1122, 1173 and 1358. It continues the level-script series of SPEC-0032, SPEC-0037, SPEC-0039, SPEC-0041 and SPEC-0042. The same script as module 956 is also stored under resource 970; it is described once, under 956.

Eight of the fifteen are Interior-family modules and seven are Exterior. The modules use the host API catalogue of SPEC-0030 and the trigger, thread and slot rules of SPEC-0030 and SPEC-0032. Every module exports Init, Update and Exit. Most are built from a few parts: a proximity dialogue selector that picks one line by a cascade of game flags, an item-ask or on-enter line, a teleport out to an interior or exterior world, a world-activate arrival that moves the hero, and slot-set loads chosen by flags. Nine modules (eight Interior modules and 1173) also use a shared entity-control routine that drives idle animations and waypoint walks for a table of entity records; their Update calls it once per tick. The same routine sits in 27 modules across the level-script specs; this spec is its home, and its full rules and the record tables are in the section "Shared entity-control routine" below (High).

Times are seconds unless marked as ticks. Sleep(n) yields n ticks; Wait(s) yields about s x 60 ticks (SPEC-0030, Open question 1). Notation follows SPEC-0037: "flag N" is a game flag, "item N" a world item id, "object N" an animated object id, "entity N" an entity id, "line N" a dialogue line id of the world's text, "set N" a slot character set, "eW N" a world identifier, "locator N" a locator id, "track N" a camera track id, and "fade black" a colour of zero values. "Set" for a flag means non-zero. "Trigger record" is the callback record of SPEC-0030 section 3 (state word at +0x10, trigger index at +0x14, object pointer at +0x1C).

Where a call argument is taken from a record field or from module data it is written as "from record" or "from module data", and the value is not given. Where an argument's value is not known it is written "unknown".

SlotLoad calls in this spec are written as SlotLoad(slot word, set N, true). The trailing true is the fifth argument, called "activate" (SPEC-0030 row 205): when true, every character of the slot that has a record is spawned after a successful load (nonzero result). The fourth (keep) argument is not written in these calls. The fifth argument is true (1) in every resolved SlotLoad call of this spec.

Slot word. No level module in this spec passes a fixed -1 as its slot number. Each SlotLoad and SlotReset reads the slot number from a data word of its own module. In the module file that word holds -1. Where reached code stores the result of a SlotCreate call into the word (normally during init, before the threads that load it start), the load goes to that slot. The first SlotCreate in a world returns 0 (SPEC-0030 row 207). A load or a reset through a word that no reached code stores into does nothing and returns 0 (SPEC-0030 section 5). Each module section says whether its words are stored. "Slot word" in this spec means such a module data word.

Routines are named by role in this spec (for example "proximity routine", "on-enter routine", "teleport routine", "activate-world routine", "thread A") because the module's own routine names are not part of the clean-side vocabulary.

Reachability: "reached" and "reachable" in this spec mean static code reachability: code that init, update, exit, a load or unload hook, or a registered callback calls, starts as a control thread or registers. Flag gates decide whether a reached routine runs in play. Gate values are not evaluated here, so play reachability is not claimed.

## Where it lives on the disc

- Each module is the level-script chunk (type 37, "SNR2") of the world pack with the same resource number, as in SPEC-0030 and SPEC-0032.
- Each module's family is the one whose Init name it exports (Exterior or Interior). Every module in this spec also exports Update and Exit.
- Module data holds slot tables passed to SlotCreate and a few words passed to SlotLoad. Their contents are not copied here; where a table is used it is named in the module section.

## Family and exit summary

| Module | Family | Exit exported | Triggers and callbacks registered from Init | Control threads | Update |
| --- | --- | --- | --- | --- | --- |
| 956 | Exterior | yes (empty) | about 30 triggers, 9 entity proximity, 1 teleport, 1 activate-world | 1 from Init; 3 more started by triggers and helpers | per-tick intro and navigation |
| 1008 | Interior | yes (empty) | 2 proximity, 1 on-enter, 3 item-ask, 1 teleport, 1 activate-world | 1 | per-tick shared entity-control routine |
| 1011 | Interior | yes (empty) | 2 proximity, 1 on-enter, 2 item-ask, 1 teleport, 1 activate-world | 1 | per-tick shared entity-control routine |
| 752 | Interior | yes (empty) | 1 proximity (set up in a helper) | 1 allocation and 2 starts from Init | per-tick shared entity-control routine |
| 1020 | Interior | yes (empty) | 2 proximity, 1 on-enter, 2 item-ask, 1 teleport, 1 activate-world | 1 | per-tick shared entity-control routine |
| 749 | Interior | yes (empty) | 1 on-enter, 1 proximity, 1 item-ask | 1 allocation and 2 starts from Init | per-tick shared entity-control routine |
| 1398 | Exterior | yes (empty) | 9 triggers, 1 teleport | 1 allocation; 2 starts from Init, and up to 4 more under flag tests | per-tick entity and boss-cage checks |
| 1066 | Exterior | yes (empty) | 4 triggers, 4 proximity | 1 allocation and 3 starts from Init | per-tick world masks |
| 748 | Interior | yes (empty) | 1 on-enter, 1 proximity (both in a helper) | 1 allocation and 2 starts from Init | per-tick shared entity-control routine |
| 1014 | Interior | yes (empty) | 4 proximity, 1 on-enter, 1 teleport, 1 activate-world | 1 allocation and 1 start from Init | per-tick shared entity-control routine |
| 1335 | Exterior | yes (empty) | 6 proximity, 10 triggers, 3 teleports, 1 activate-world | 1 allocation and 3 starts from Init (2 under flag tests); 1 more started by a trigger | per-tick navmesh and movement setting |
| 750 | Interior | yes (empty) | 1 on-enter, 4 proximity | 1 allocation and 2 starts from Init | per-tick shared entity-control routine |
| 1122 | Exterior | yes (empty) | 5 triggers, 1 more trigger and 1 activate-world under flag tests, 4 proximity | 1 allocation and 2 starts from Init (a third under a flag test) | per-tick navmesh assignments |
| 1173 | Exterior | yes (empty) | 2 triggers, 1 activate-world, 1 teleport, 1 proximity | 1 allocation and 1 start from Init | per-tick shared entity-control routine |
| 1358 | Exterior | yes (empty) | 6 teleports, 4 triggers (3 more under a flag test), 6 proximity, 1 activate-world | 1 allocation and 1 start from Init; 1 more started by routine 19 under a flag test | empty |

Update makes host calls directly in 956, 1066, 1122, 1335 and 1398. In 748, 749, 750, 752, 1008, 1011, 1014, 1020 and 1173 Update calls the shared entity-control routine once per tick (High for the call). Only 1358 has an empty Update.

## Modules

Modules are grouped by family in the order of their numbers in the table above.

### 956: Exterior, long intro sequence and trigger ladder

- Family: Exterior. Exit exported (empty). The same script is also stored under resource 970 and is not described again.
- Confidence: High for Init, the knight records and cages, the zone thread, the lift, triggers 10 to 16, the teleport, threads 3 and 4, the Update and the aftermath scene. Medium for the proximity line mapping.
- Glade stage h: the highest of flags 67, 66, 65, 64, 63 and 62 that is set, or none when all are clear (High). Flag 62 is set by pack 706's Init, 63 by pack 1098's trigger 0, 64 after the Gnarfas fight (pack 985), 65 by this Init when flag 248 is set, 66 by trigger 10 and 67 by trigger 12.
- Slot words: S (the set 7, 8 and 9 slot), Z (the zone slot), K (the set 10 to 12 slot) and a set-1 word that no code stores. Each holds -1 in the file until Init stores a SlotCreate result into it.
- Init, in order (High):
  1. The first-update word is set to 1. The activate-world routine and the teleport routine (index 0, context 13) are registered.
  2. Flag 214 clear: objects 2 and 1 at their start pose. Flag 214 set and flag 283 clear: object 2 at its start pose.
  3. SlotAllocate(2) when (flag 248 clear and flag 64 clear) or (flag 248 set and flag 67 set); otherwise SlotAllocate(1).
  4. Slots: flag 248 clear: S = SlotCreate{7, 8}, and when flag 214 is also clear, Z = SlotCreate{5, 4, 3, 2}. Flag 248 set and 67 clear: S = SlotCreate{9}. Flags 248 and 67 set: K = SlotCreate{10, 12, 11} only; S is not stored.
  5. Object 17 at its start pose. When the lift is not ready (Lift, below) and flag 66 is set: object 10 at its start pose. Otherwise object 9 at its start pose and the lift is "down".
  6. Registrations: triggers 8 and 9 (the lift calls), animated trigger 9 (kind 0), triggers 1, 0, 2, 14, 15, 16, 10, 11 and 12, and the zone triggers 3 to 6.
  7. Knights: with flag 211 clear, for each knight record its cage trigger is registered, its cage object set to its start pose and its lock made not breakable. With flag 211 set, the cage objects are set to their end pose and the locks broken.
  8. The proximity set (below); ControlThreadsAllocate(5); thread 3 started.
  9. By h: 62 or 63: effects 9 to 51 started, object 0 at its start pose. 64 or 65: effects 9 to 51 stopped, object 0 at its start pose. 66, 67 or none: effects stopped, object 0 deactivated.
  10. A 52-entry navmesh cache is reset (all unknown).
  11. Flag 248 set: SetGameState(65).
- Knight records (High). Fields: knight entity, release line, flag, zone trigger, cage trigger, cage object, guard AI, lock (breakable), hero locator, knight locator, camera, lock line, region of the zone load.
  - Record 0: 5, 61, 208, 3, 21, 18, 23, 3, 54, 55, 26, 62, 962.
  - Record 1: 4, 77, 209, 4, 23, 20, 24, 5, 58, 59, 27, 78, 961.
  - Record 2: 3, 75, 210, 5, 22, 19, 25, 4, 56, 57, 28, 76, 960.
  - Record 3: 2, 73, 623, 6, 24, 21, 26, 6, 60, 61, 25, 74, 959.
  - Record 4 (no knight): region 963, flag 64; it is the trigger-3 zone load once flag 208 is set.
- Trigger routine 1 (indices 0, 1, 2): if the trigger record's state is 1 and the object pointer is non-null, it maps the trigger index to a choice value (index 1 -> 0, index 0 -> 1, index 2 -> 2). If the value equals the choice word (file value -1), nothing more happens. Otherwise it stores the value into that word and starts thread 4. So each change of trigger starts thread 4 once (High).
- Zone triggers (indices 3, 4, 5 and 6; High): if the state is 1 and the object pointer is non-null, and flags 214 and 248 are both clear, it picks record g: trigger 4 -> g 1, 5 -> g 2, 6 -> g 3; trigger 3 -> g 0 while flag 208 is clear, g 4 once flag 208 is set. If the record's own flag is set (208, 209, 210 or 623 for records 0 to 3; 64 for record 4) it does nothing more. Otherwise it waits (one tick at a time) until the module's start-up word is set and starts thread 5 with argument g. Result: trigger 3 loads region 962, or 963 once flag 208 is set; triggers 4, 5 and 6 load regions 961, 960 and 959.
- Thread 5, the zone thread (argument g; High): waits until the previous zone thread has finished; SlotReset(Z); SlotLoad(Z, j of record g, true); waits until the hero leaves that zone trigger (WaitTillLeaveTrigger); SlotReset(Z) again. So a zone's characters (the guard and the knight) exist only while the hero stands in that zone.
- Cage triggers (indices 21 to 24, one per knight record; High). On a first fire with flag 211 set the routine returns at once. Otherwise, while the trigger state is non-zero, once per tick:
  - Guard AI alive: the square-button prompt (context 3) is shown; a press does nothing.
  - Guard dead and lock intact: once per module load, the lock is made breakable and the lock line is shown.
  - Lock broken and the record's flag clear: prompt context 6; on Square: FadeScreenOut(1.0, wait); PlayCamera(camera, no wait, 1.0); the cage object to its end pose; MoveHero(hero locator, control); MoveEntity(knight, knight locator, control); Sleep(30); FadeScreenIn(1.0, wait); a conversation with the release line; FadeScreenOut(0.5); SlotReset(Z); for record 0 only, thread 6 loads Z with record 4 (region 963); SetGameState(record flag); when all of 208, 209, 210 and 623 are set, SetGameState(211); Sleep(30); camera returned; FadeScreenIn(0.5).
  - Freeing a knight therefore takes: kill the guard, break the lock, press Square at the cage.
- Thread 6 (started with argument 4 by record 0's cage trigger, just before it sets flag 208): SlotLoad(Z, j of record 4, true), which loads region 963 at once (Medium-High).
- Lift (triggers 8 and 9; High). Ready = flag 64 clear and (flag 211 set, or all of 208, 209, 210 and 623 set, in which case flag 211 is set). Trigger 8 (lift at the top) and the plain trigger 9 (lift at the bottom) run the lift move when object 9 is idle and the lift is ready.
  - Going up: LoopAnim(17, speed -1); entity 6 loops animation 64 (control 1); PlaySound(8); PlayAnim(9, +1); wait while object 9 animates; KillSound(8); entity 6 released; StopAnim(17). Going down: the same with the speeds reversed and no sound.
  - Animated trigger 9 (state 1): not ready: a line: none when flag 211 is set; line 70 when flag 208 is clear; otherwise by the number of flags 209, 210 and 623 still clear: 1 -> line 66, 2 -> 67, 3 -> 68. Ready: both heroes blocked, WalkHeroAndOrient(locator 44 at the top, 43 at the bottom), Wait(2.0 s), MoveHero(same locator, control), the lift move, camera returned, then it waits until the hero leaves the trigger.
  - In Act 3 (flag 64 set) the lift never runs.
- Trigger 16: if the state is 1, the object pointer is non-null and flag 212 is clear: WaitTillLeaveTrigger(record), then SetGameState(flag 212, true) (High).
- Trigger 15 (state 1, object non-null): if flag 212 is set and flag 214 is clear: PlayFMV(11, wait), then StartExteriorReplacementLoad(eW 77, -1, no colour), which loads pack 985. Otherwise (flag 212 clear, or flags 212 and 214 both set), if flag 66 is set and flag 283 is clear: fade out (wait), StartExteriorReplacementLoad(eW 77, -1, no colour) and fade back in (High for the calls; Medium for the fades).
- Trigger 14 (state 1, object non-null): if flag 214 is set and flag 216 is clear: DeactivateObj(3), DeactivateObj(4), DeactivateObj(5); then PlayFMV(13, wait) if movie 13 has not been played (High).
- Trigger 10 (state 1, object non-null): if flag 248 is set and flag 281 is clear: SetGameState(flag 281, true); SetGameState(flag 66, true); DeactivateObj(9); ActivateObj(10); PlayFMV(43, wait). If the hero does not hold item 36, one line is selected by active hero (line 65 for one hero and line 64 for the other; which is which remains unresolved). Whether the item is held or not, targets 0, 1 and 2 are then made active. Thus the item check selects dialogue; it does not gate activation of the three targets (High for the call and branch result).
- Trigger 11 (state 1, object non-null): if flag 281 is set and flag 282 is clear: SetGameState(flag 282, true) (High).
- Trigger 12 (state 1, object non-null): if flag 282 is set and flag 284 is clear, in order: SetGameState(flag 284), SetGameState(flag 67), SetGameState(flag 20), SetGameState(flag 302) (all true); PlayFMV(44, wait); RemoveAllOfItemFromHero(item 14); GiveItemToHero(item 30, 1, no notification) (High).
- Teleport routine (index 0, kind 13): shows the square-button prompt (context 13) and reads button 0. When it is pressed: if flag 67 is clear and flag 64 is set, LevelMsg(line 71) and nothing else. Otherwise (flag 67 set, or flags 67 and 64 both clear): AddBlackboardMessage(type 2, value 250, eW 74) and StartEnterInterior(eW 76, -1, -1, no colour), which enters pack 984 (High).
- Proximity routines (entities 1, 5 and 45 to 51; entities 1 and 47 share one routine): each shows a dialogue line from the entity's flags (flags 59, 86, 208, 209, 210 and 211 are read). Example: the routine for entity 1 gives line 2 when flag 59 is set (with the conversation bracket) and line 12 otherwise. Lines used across these routines are 0 to 58 (the mapping of each entity to line is Medium).
- Thread 3 (started from Init and by the aftermath scene; High): flags 248 and 67 set: a SlotReset(S) and a SlotLoad(set-1 word, set 1, true); both words are unstored in this case, so both calls do nothing and region 958 is never loaded. Flag 248 set and 67 clear: SlotLoad(S, set 9, true) (region 966). Flag 248 clear and 64 set: SlotLoad(S, set 8, true) (region 965). Both clear: SlotLoad(S, set 7, true) (region 964). Debug lines throughout.
- Thread 4 (started by trigger routine 1): if flag 67 is clear it does nothing (debug lines only). If flag 67 is set: SlotReset(K), then by the choice word: 0 -> SlotLoad(K, set 10, true); 1 -> set 12; 2 -> set 11 (regions 967, 969 and 968). So trigger 1 loads set 10, trigger 0 set 12 and trigger 2 set 11 (High for the calls and the mapping).
- Regions this module loads through the record table: 959 to 963. Region 958 (the set-1 load in thread 3) is never loaded (High).
- Update (each tick; High):
  1. On the first update only (the first-update word, set by Init, is then cleared): targets 0, 1 and 2 are made inactive when flag 281 is clear and active when it is set.
  2. With flag 214 set and flag 216 clear: the aftermath scene (below) runs once (it sets flag 216).
  3. Every tick, for characters 0 to 51: the closest navmesh id is found once and cached; when it is known, CharacterNavmeshAssign(character, cached id); and DisableVillagerSocialize(character).
- Aftermath scene (after the Gnarfas fight; High; this is the only long scripted sequence of the module): SetGameState(true) for flags 216, 10, 18, 30, 47 and 64, in that order; PlayFMV(14, wait) if movie 14 has not been played; SetGameState(217); thread 3 started; Sleep(1). Then: fade out 0 s (wait); letterbox on; fade in 1.8 s; PlayCamera(30, wait, 1.0); hero equipped state off; MoveHero(locator 2); hero animation 137; Sleep(20); camera shake (0.6, 0.5); Sleep(30); hero animation 158 (wait); Sleep(30); PlayCamera(31, 1.0); line 63; fade out 0.5 s (wait); fade in 0.5 s; PlayCamera(32, wait, 0.6); fade out 0.5 s (wait); entity 28 made visible; fade in 0.5 s; entity 28 movement type 7; WalkEntity(28, locator 46, stop 1.5); PlayCamera(33, wait, 1.0); Sleep(30); PlayCamera(34, 1.0); hero equipped state on; Wait(1.2 s); line 72; fade out 0.5 s (wait); SetAI_Agro(28); letterbox off over 1.0 s; camera returned; fade in 0.5 s.
- Path values (SPEC-0058): step 18 (Act 1, h 62 or 63, flag 214 clear): effects on, slots S and Z, cages live. Step 20 (flag 214 set, flag 64 set by pack 985): one slot; the aftermath scene runs. Step 39 (flag 248 set, h 64 then 65 or 66): no cages, the lift does not run.
- Test vector: knight record 1, guard AI 24 dead, lock 5 intact, hero in trigger 23: the lock is made breakable and line 78 is shown (once). After the lock breaks: Square -> knight 4 moved to locator 59, line 77, flag 209 set.
- Edge cases: the aftermath scene does not repeat once flag 216 is set. The lock line of a cage shows once per module load. With flag 211 set at load the cages are open and their triggers are not registered.
- Open: the meanings of flags 62 to 67, 208 to 211, 214, 216, 248, 281 and 283 are not mapped. Resolved: the first-update word is set by Init and used once by the first Update; the knight record fields; the entity and locator pairing of the cage scenes (the record table); the object each Init call names. The call pair once counted as unreached is thread 3's reset and set-1 load with flags 248 and 67 set, which act on unstored words.

### 1008: Interior, item-ask lines, on-enter line and an arrival

- Family: Interior. Exit exported (empty).
- Confidence: High for the Init registrations, the thread and the entity routine; Medium for the arrival.
- Init:
  - Proximity routine registered for entities 0 and 1 (the same routine for both).
  - On-enter routine registered with index 1.
  - Item-ask routines registered for items 102, 109 and 115.
  - Entity 1 added to the entity table (the add starts LoopEntityAnim(1, 0, 30.0 s, control 1); record in the shared entity-control routine section).
  - In play, with no flag or debug gate in Init: SlotAllocate(1); SlotCreate(2 entries, table in module data), whose result is stored in the slot word; ControlThreadsAllocate(1); ControlThreadStart(thread A, world).
  - Teleport routine registered (index 0, kind 13); activate-world routine registered.
- Proximity routine (entities 0 and 1): if flag 86 is set, SetConversationTarget; LevelMsg(line 12, bracket); EndConversation. Otherwise LevelMsg(line 15, bracket).
- On-enter routine (index 1), always with the conversation bracket (SetConversationTarget; LevelMsg; EndConversation): if flag 86 is set, lines 0, 1 and 2 in turn on successive entries; otherwise lines 3, 4 and 5 in turn. A module data counter holds the position and restarts at the first line of a group when the flag-86 case changes (High).
- Item-ask routines: item 102 -> LevelMsg(line 7; line 6 on another path); item 109 -> LevelMsg(line 9; line 8 on another path); item 115 -> LevelMsg(line 11; line 10 on another path). The alternative line is chosen by a module data cell (Medium).
- Thread A (runs once): loads the slot word with j 1 when flag 86 is set, else j 0 (region 1010 or 1009; region = 1008 + 1 + j), fifth argument true. The debug-mode query in this thread only guards debug prints around the load, so the load runs in play (High). Then flag 298 is read and, if it is clear, item price multiplier 4 is set (it runs in play when flag 298 is clear; see SPEC-0050, section 5).
- Teleport routine (index 0): debug line; AddBlackboardMessage(type 2, value 20, eW 81); if the trigger index is 0: StartLeaveInterior(eW 7, -1, -1, no colour), which leaves to pack 699; Wait(3.0 s).
- Activate-world routine: reads the blackboard value for eW 81. If it is 20: MoveHero(locator 0, true, CONTROL 1); SetToNormalCamera(5.0, 0.0, true); AddBlackboardMessage(type 2, value -1, eW 81).
- Dormant routines: the routines that nothing references make only debug calls (debug-mode queries, a debug button check, an on-screen debug message and a one-tick sleep); none of them shows a dialogue line. Lines 0 to 5 come from the live on-enter routine above.
- Helpers: the shared entity-control routine (see its section).
- Update: calls the shared entity-control routine once per tick with the module's entity table (see the shared entity-control routine section; High for the call). Exit: empty.
- Edge cases: the thread does nothing in play mode; the teleport waits 3 s in every case.

### 1011: Interior, item-ask lines, on-enter line and an arrival

- Family: Interior. Exit exported (empty).
- Confidence: High for Init, the teleport and the entity routine; Medium for the arrival and the thread.
- Init: proximity routine for entities 0 and 1; on-enter routine (index 1); item-ask routines for items 154 and 174; entity 1 added to the entity table (as 1008); the same slot and thread setup as 1008, run in play with no gate (SlotAllocate(1), SlotCreate(2 entries, result stored in the slot word), ControlThreadsAllocate(1), ControlThreadStart(thread A)); teleport routine (index 0); activate-world routine.
- Proximity routine (entities 0 and 1): if flag 86 is set: LevelMsg(line 10, bracket); otherwise LevelMsg(line 13, bracket).
- On-enter routine (index 1): if flag 86 is set: LevelMsg(line 0, bracket); otherwise LevelMsg(line 3, bracket).
- Item-ask routines: item 154 -> LevelMsg(line 7; line 6 on another path); item 174 -> LevelMsg(line 9; line 8 on another path) (Medium).
- Thread A: as 1008: loads j 1 (flag 86 set) or j 0, which is region 1013 or 1012, in play; the price multiplier call runs in play when flag 298 is clear.
- Teleport routine (index 0): debug line; AddBlackboardMessage(type 2, value 30, eW 81); if the trigger index is 0: StartLeaveInterior(eW 7, -1, -1, no colour), which leaves to pack 699; Wait(3.0 s).
- Activate-world routine: if the blackboard value for eW 81 is 30: MoveHero(locator 0, true, CONTROL 1); SetToNormalCamera(5.0, 0.0, true); AddBlackboardMessage(type 2, value -1, eW 81).
- Update: calls the shared entity-control routine once per tick with the module's entity table (see the shared entity-control routine section; High for the call). Exit: empty.

### 752: Interior, one proximity routine and a shared entity-control routine

- Family: Interior. Exit exported (empty).
- Confidence: Medium for the proximity routine; High for Init.
- Init calls a helper that writes records five times; a debug-mode query inside that helper gates only those record writes (its body makes no host call). These records form a debug flag and item list; they are not the entity table of the shared entity-control routine. In play, Init also makes: a blackboard message (type 2, value 30, world eW 20), a control-thread allocation and two control-thread starts, and the entity-table setup (one record, with four waypoints; see the shared routine section). Init also reaches a helper that makes the proximity registration, which runs in play.
- Proximity routine (one entity): if flag 266 is set: SetConversationTarget; LevelMsg(line 0, bracket); EndConversation. Else if flag 265 is set: LevelMsg(line 3). Else, if flag 76 is set: SetGameState(flag 265, true) and LevelMsg(line 6). Otherwise LevelMsg(line 7).
- Shared entity-control routine: see its section.
- Update: calls the shared entity-control routine once per tick with the module's entity table (see the shared entity-control routine section; High for the call). Exit: empty.
- Threads (High: the routines are the arguments of the starts): the first Init start runs the debug-gated per-tick entity loop (the routine described under SPEC-0042 module 986): its first call is the debug-mode query, which returns false in play, so it ends at once; the three routines it calls inside its debug branch do not run in play. The second runs SetToNormalCamera(5.0, 0.0, true) once and ends.

### 1020: Interior, item-ask lines, on-enter line and an arrival

- Family: Interior. Exit exported (empty).
- Confidence: High for Init, the teleport and the entity routine; Medium for the arrival.
- Init: proximity routine for entities 0 and 1; on-enter routine (index 1); item-ask routines for items 58 and 59; entity 1 added to the entity table (with three waypoints; see the shared routine section); the slot and thread setup as in 1008 (run in play; the slot word is the SlotCreate result); teleport routine (index 0); activate-world routine.
- Proximity routine (entities 0 and 1): if flag 86 is set: LevelMsg(line 10, bracket); otherwise LevelMsg(line 13, bracket).
- On-enter routine (index 1): if flag 86 is set: LevelMsg(line 0); otherwise LevelMsg(line 3).
- Item-ask routines: item 59 -> LevelMsg(line 7; line 6 on another path); item 58 -> LevelMsg(line 9; line 8 on another path) (Medium).
- Thread A: as 1008: loads j 1 (flag 86 set) or j 0, which is region 1022 or 1021, in play; the price multiplier call runs in play when flag 298 is clear.
- Teleport routine (index 0): debug line; AddBlackboardMessage(type 2, value 70, eW 81); if the trigger index is 0: StartLeaveInterior(eW 7, -1, -1, no colour), which leaves to pack 699; Wait(3.0 s).
- Activate-world routine: if the blackboard value for eW 81 is 70: MoveHero(locator 0, true, CONTROL 1); SetToNormalCamera(5.0, 0.0, true); AddBlackboardMessage(type 2, value -1, eW 81).
- Update: calls the shared entity-control routine once per tick with the module's entity table (see the shared entity-control routine section; High for the call). Exit: empty.

### Shared entity-control routine (home section; used by 27 modules)

- Confidence: High. One routine, identical in all 27 modules apart from its call targets, with every branch described below.
- Users: Update calls it once per tick, with the module's entity table, in 722, 745, 748, 749, 750, 752, 909, 951, 1008, 1011, 1014, 1020, 1098, 1150, 1163, 1173, 1405, 1407, 1409, 1411, 1414, 1417, 1419, 1421, 1423 and 1425. There is no gate except in 745, where Update calls it only while flag 264 is clear. Module 1017 holds a copy that nothing calls. It is not started by a trigger or a control thread. SPEC-0039, SPEC-0041, SPEC-0044 and SPEC-0045 point here for its rules.
- Records are filled in play. Each module's setup helper adds them with calls that no debug query gates (in Init, or in an arrival or scene helper; the gate of each is in the table below or in the module's own section). The debug-gated "record writes" of 748, 749, 750, 752 and of the SPEC-0044 modules build a debug flag and item list and never touch this table.

Table (module data): 10 record slots of 440 bytes each, followed by the record count (offset 4400 = 0x1130 from the table start). At most 9 records are ever added: the add step refuses when count + 1 is not below 10.

| Record field | Meaning |
| --- | --- |
| entity | entity id (-1 in an unused slot) |
| active | 0 = the record is skipped |
| hold | countdown in ticks; while non-zero no new idle animation is chosen (0 on add; no module writes another value) |
| anims[0..8], anim count | idle-animation list: an animation id and seconds per entry (an append is ignored unless count + 1 stays below 10) |
| idle anim | the default loop animation |
| current anim | last chosen animation id (-1 on add) |
| anim timer | ticks the chosen animation still owns the entity |
| points[0..8], point count, point index, order mode, path on | waypoint list. Each point: target locator, path flag (0 in every module), dwell flag, orient flag, stop distance (1.1 when given as -1), point animation id (-1 = none), point animation seconds. Path on becomes 1 when the first point is added. Order mode is never written (0 = cycle). |
| anim min and max, dwell min and max | intervals in seconds |
| anim delay, move delay | ticks: anim delay = Random(trunc(60 x anim min), trunc(60 x anim max)) on add; move delay = 10 on add |
| anim clock, move clock | tick counters, 0 on add |

Setup steps (made by each module's setup helper):
- Reset: every slot gets entity -1, active 0, anim count 0 and point count 0, and the record count becomes 0. A static constructor clears the table the same way when the module is loaded.
- Add(entity, idle anim, anim min, anim max, dwell min, dwell max): fills the next record with active 1 and calls LoopEntityAnim(entity, idle anim, 30.0 s, control 1).
- Add animation(entity, anim, seconds) and Add point(entity, locator, dwell flag, point anim, point seconds, orient flag, stop distance): append to the first record whose entity matches; ignored when its list is full.
- Set active(entity, 0 or 1): parks a record during a cutscene and wakes it again (1163 entity 15; 909; 1098).

Per tick, for each record r below the count:
1. Skip it when not active, or when IsEntityTalking(entity).
2. If the anim timer is above 0, subtract 1; if hold is above 0, subtract 1.
3. Skip the rest when IsEntityWalkToPointActive(entity) (walk classes 23 to 26).
4. Move step, only when path on and move clock >= move delay: move clock = 0; the point index advances (cycle: index + 1, back to 0 after the last point; modes 1 = ping-pong and 2 = random exist but are never selected); move delay = Random(trunc(60 x dwell min), trunc(60 x dwell max)) if the new point's dwell flag is set, else 0; anim timer + 1; GrabEntityControl(entity); WalkEntityAndOrient(entity, locator, no wait, stop, control 1) when the orient flag is set, else WalkEntity(entity, locator, no wait, stop, control 1) (WalkEntityAlongPath when the path flag is set, which no module does); ReleaseEntityControl(entity).
5. If the anim timer is 0: anim clock + 1 and move clock + 1.
6. Animation step, when hold is 0, anim clock >= anim delay, and (anim count > 0, or path on and the current point's animation is not -1): anim delay = Random(trunc(60 x anim min), trunc(60 x anim max)); anim clock = 0; take (id, s) from the current point when path on and its animation is not -1, else from anims[Random(0, anim count - 1)]. If s is -1: PlayEntityAnim(entity, id, no wait, control 1) and s = 1.5; else LoopEntityAnim(entity, id, s, control 1). Anim timer = trunc(60 x s).
7. If the anim timer is 0: LoopEntityAnim(entity, idle anim, 0.01 s, control 1). 0.01 s converts to 0 ticks, which the host stores as "forever" (SPEC-0030 row 104), so the idle loop is asked for again on every idle tick.

Edge cases: a record whose entity is talking is frozen (no timer moves). During a walk only the two countdowns move. A point with its dwell flag clear gives move delay 0, so the next move starts on the first idle tick after arrival. The move step and the animation step can both run on one tick; the animation call then replaces the walk's animation request. Every waypoint uses walk-to-locator; path walking is never selected on disc.

Test vector (745, entity 3: anims 51 (2.5 s) and 147 (1.2 s), anim 2 to 5 s, dwell 5 to 7 s, points 8, 9, 7 with dwell and orient): after the add, the first idle tick with move clock >= 10 walks entity 3 to locator 9 (index 0 to 1) with WalkEntityAndOrient(3, 9, no wait, 1.1, control 1). After arrival it stays idle 300 to 420 ticks before walking to 7, then 8, then 9 again; meanwhile every 120 to 300 idle ticks it plays 51 for 150 ticks or 147 for 72 ticks.

Records per module, condensed. Format: entity: idle anim; anim interval; dwell interval; anims (seconds); points (d = dwell, o = orient). Dwell is 3 to 4 s and anims are 2.5 s unless stated; "no points" where none are added.

| Pack | When added | Records |
| --- | --- | --- |
| 722 | Init and every arrival, only with flag 76 clear | flag 75 set: entities 7 to 10; clear: 3 to 6. Each: 146; 4-7 s; 147 |
| 722 | race set-up (the table is reset first) | 11, 12, 14, 13, 15: 36; 4-7 s; 37. The race result routine resets the table, so it stays empty until the next arrival |
| 745 | Init | 0 and 2: 98; 2-3 s; no anims. 1: 74; 5-8 s; 146, 147. 3: 0; 2-5 s; dwell 5-7 s; 51, 147 (1.2); points 8, 9, 7 (d, o) |
| 748 | Init | 0: 0; 2-5 s; dwell 5-7 s; 40, 110, 96, 41 (1.3) |
| 749, 750 | Init of each pack | 749: its entity 0 (Brigson, the only entry of 749's entity table); 750: its entity 2 (Primrose, embedded ordinal 2); each id is in that pack's own entity table: 0; 2-5 s; dwell 5-7 s; 40, 96, 41 (1.3) |
| 752 | Init | 0: as 749; points 1, 2, 3, 4 (d) |
| 951 | the story thread (Init starts it) | 4: 83; 5-7 s; 80, 74 |
| 1008 | Init | 1: 0; 5-8 s; 95, 96, 41 |
| 1011 | Init | 1: 0; 5-8 s; 41, 95, 96 |
| 1014 | Init | 2: 0; 5-8 s; 36, 26, 27; points 1, 2, 3 (d, o). 3: 0; 5-8 s; 95, 96, 41; points 4, 5 (d, o) |
| 1020 | Init | 1: 0; 5-8 s; 95, 96, 41; points 1, 2, 3 (d, o) |
| 1150 | setup helper | 1 and 2: 65; 3-10 s; 68, 67. 3: 0; 2-3.5 s; dwell 5-6 s; 64, 13, 12, 81 (1.5 each); points 1, 2, 3 (d, o). 4: 65; 4-6 s; 68, 70. 0: 0; 5-9 s; 40, 41, 51, 95, 96 |
| 1163 | Init | 15: 65; 4-7 s. 6: 65; 3-7 s. 8: 65; 6-8 s. 13: 65; 6-8 s. 14: 65; 5-9 s (each 67, 68). 7: 0; 5-9 s; 40, 96. No points |
| 1173 | Init | 0: 0; 1-3 s; 54, 49, 52, 62, 55; points 3 (d, o), 4 (d), 5, 6 |
| 1405 | Init, only with flag 104 set | 0: 0; 5-8 s; dwell 5-7 s; 40, 96, 41 (1.3) |
| 1407 | Init (flag 100, which Init itself sets) | 0: 0; 8-14 s; dwell 5-7 s; 40, 96, 41 (1.3) |
| 1409 | Init, only with flag 106 set | 0 and 1: 0; 8-15 s; dwell 5-7 s; 40, 95, 96, 41 (1.3) |
| 1411 | Init | eleven adds in the order 10, 6, 7, 8, 9, 4, 1, 0, 2, 3, 5; all 8-12 s. Idle 0 (entity 0: idle 50). Anims: 10, 6, 4, 0: 51, 40; 7, 1: 51, 40, 95; 8, 2: 96, 42; 9, 3: 96, 95, 41 (1.2); 5: 96, 95, 41. By the nine-record limit the adds for entities 3 and 5 are refused, and their animation appends find no record (Medium: follows from the limit) |
| 1414 | Init, only with flag 102 set | 7, 8, 9: 25; 5-7 s; dwell 8-12 s; 110, 95, 40, 41 (1.3); points 0-2 (entity 7), 3-6 (8), 7-11 (9), all (d, o) |
| 1417 | Init, only with flag 102 set | 0: 0; 8-15 s; dwell 5-7 s; 40, 95, 96, 41 (1.3) |
| 1419 | Init, only with flag 100 set | 0: 0; 8-14 s; dwell 5-7 s; 40, 96, 41 (1.3) |
| 1421, 1423 | Init, only with flag 102 set (1421) or 106 set (1423) | 0: 0; 8-15 s; dwell 5-7 s; 40, 95, 96, 41 (1.3) |
| 1425 | Init, only with flag 106 set | 0: 0; 8-15 s; dwell 5-7 s; 40, 95, 96, 41 (1.3) |
| 909, 1098 | several scene helpers | each helper resets the table and adds the record set of one story stage; Set active parks and wakes records during cutscenes. The per-stage lists are not given here |

Without its gate a module's table stays empty and the routine does nothing. Whether a host asked for a repeat of the same idle loop restarts the clip is a host question (SPEC-0005).

### 749: Interior, on-enter line, proximity line and item-ask line

- Family: Interior. Exit exported (empty).
- Confidence: Medium for the routines; High for Init.
- Init: AddBlackboardMessage(type 2, value 40, eW 20); ControlThreadsAllocate(2); ControlThreadStart(thread A, world); ControlThreadStart(thread B, world). The on-enter routine (index 0), the proximity routine (entity 0) and the item-ask routine (item 57) are registered through a helper that Init calls. A debug-mode query inside a helper that Init calls three times gates only that helper's record writes (its body makes no host call). Init then calls, with no branch, the entity-table setup helper: it runs the idle-loop helper (a frames-per-second query, a random draw and LoopEntityAnim) for the module's entity and adds animation ids 40, 96 and 41 to its record. These calls run in play (High).
- On-enter routine (index 0): if flag 76 is set: SetConversationTarget; LevelMsg(line 0, bracket); EndConversation. Otherwise LevelMsg(line 2).
- Proximity routine (entity 0): if flag 76 is set: LevelMsg(line 4, bracket). Otherwise LevelMsg(line 6) with the bracket and LevelMsg(line 9).
- Item-ask routine (item 57): LevelMsg(line 4).
- Thread A (the first Init start): the debug-gated per-tick entity loop (the routine described under SPEC-0042 module 986): its first call is the debug-mode query, which returns false in play, so it ends at once; the three routines it calls inside its debug branch do not run in play. This module makes no query of flags 75 or 76 (High).
- Thread B (runs once): SetToNormalCamera(5.0, 0.0, true).
- Shared entity-control routine: Update calls it every tick (see the shared entity-control routine section).
- Update: calls the shared entity-control routine once per tick with the module's entity table (see the shared entity-control routine section; High for the call). Exit: empty.

### 1398: Exterior, triggers for a boss cage and a dialogue set

- Family: Exterior. Exit exported (empty).
- Confidence: High for the four area stages below (calls, flags and their gates); Medium for the trigger-34 sequence detail and the Init object setups.
- Init:
  - Control threads: ControlThreadsAllocate(3). Two control-thread starts run in play after the flag 106 branch below joins; up to four more run under flag tests in the branch where flag 106 is clear (see the bullets below).
  - Triggers (index, routine): 34 (routine 1), 38 (routine 2), 33 (routine 3), 40, 41, 42 and 43 (one shared routine), 35 (routine 4) and 37 (routine 5).
  - Teleport routine (index 0, kind 13).
  - Flag 355 is set and the Whinny pen list is set (below), both in play, before a branch on flag 106. When flag 106 is clear: objects 0, 1, 2 and 5 are initialised and effect 15 is stopped, and further object setups run under tests of flags 720 to 724 (below). When flag 106 is set: a different group of object setups runs.
  - Slots (in play, after the flag 106 branch joins): SlotAllocate(2); SlotCreate with a 5-entry table, whose result is stored in module data word 0x370c; SlotCreate with a 1-entry table, whose result is stored in word 0x3710. These are the slot words that thread 6 loads (Medium).
  - Area stages (flag 106 clear): the area has five stages, finished by flags 720 to 724 in turn, each with a done word in module data (file value 0). Init sets the done word of each stage whose flag is already set (and sets up that stage's finished objects; flag 723 also marks stage 4 armed), and starts thread 6 once, for the first unfinished of the stages of flags 720, 721, 722 and 724 only: flag 720 clear -> argument 0; else flag 721 clear -> argument 3; else flag 722 clear -> argument 1, together with InitMiamouseBehavior(15, 16); else flag 724 clear -> argument 2 (High). When flag 106 is set, every stage is marked done and no thread 6 start is made by Init.
- Whinny pen list (High): Init calls SetWhinnySwitches (SPEC-0030 row 89, the Whinny pen-list setter) with three four-entry tables copied from module data and a count of 4, so the fifth argument is the pen count. Pen i (0 to 3) gets inner-area id 7 + i, entrance id 11 + i and the pen point of locator 61 + i: pens (7, 11, 61), (8, 12, 62), (9, 13, 63) and (10, 14, 64), each unoccupied. The inner-area ids 7 to 10 are the navmesh ids that the stage 5 helper tests (entities 9 to 12 from region 1401 standing on navmesh 7, 8, 9 or 10), so stage 5 is done when all four listed entities stand in pen inner areas. Test vector: after Init, pen 2 has inner area 9, entrance 13 and the point of locator 63.
- Trigger 35 (routine 4) arms stage 4, and trigger 34 (routine 1) runs it; see below.
- Stage 1, routine 2 (trigger 38): when stage 1 is not done, the state is 1 and the trap is not armed: it stores the hero's hit points (GetHitPoints(-1)), arms the trap, DeactivateObj(11); ActivateObj(10) and plays it (1 s); DeactivateObj(3) and (4); ActivateObj(1) and (2); PlayCamera(track 4, 1 s); plays object 1 and object 2 (wait); ReturnToNormalCamera. While the trap is armed, an Update helper checks each tick: if the hero's hit points are below the value stored on the previous tick, or any of five listed entities is reacting to a hit, the trap trips: DeactivateObj(1) and (2); ActivateObj(3) and (4); PlayCamera(track 4, 1 s); plays objects 3 and 4 (wait); Wait(1 s); ReturnToNormalCamera; DeactivateObj(10); ActivateObj(11) and plays it (wait 1 s); the trap is disarmed, so trigger 38 can arm it again (High).
- Stage 1, routine 3 (trigger 33): if stage 1 is not done: marks it done, SetGameState(flag 720, true) and starts thread 6 with argument 3 (High).
- Stage 4 arm, routine 4 (trigger 35; stage 4 is the stage of flag 723): if the state is 1 and stage 4 is not armed: PlayCamera(track 10, 1 s); plays object 8; Wait(0.3 s); a camera shake; sleeps one tick at a time while object 8 is animating; Wait(1 s); ReturnToNormalCamera; starts thread 7; marks stage 4 armed (High).
- Teleport routine (door 0): AddBlackboardMessage(type 2, value 20, eW 160) and StartExteriorReplacementLoad(eW 160), pack 1363, with no prompt in the routine and no wait (High).
- Trigger 37 (routine 5): when flag 356 is set: fade out 0.5 s, MoveHero(locator 34), fade in 0.5 s, StartEffect(16), SetToNormalCamera(15.0, 3.14159, true) (High).
- The two ungated Init thread starts: the one-entry slot (word 0x3710) loads set 5 with keep false and activate true (region 1404); and SetToNormalCamera(10.0, 0.0, true) once (High).
- Stage 4, routine 1 (trigger 34): runs when stage 4 is armed and not done, no pass is in progress, and the record words at +0x1C, +0x20 and +0x24 are all zero. With the meanings of those words (SPEC-0030 section 3: object word 1 = the hero's own trigger-hit path, +0x20 = a hero shot, +0x24 = a hero attack), this refuses every fire by the hero; only another firer starts a pass. Each pass: the pass count (module data, file value 0) goes up by 1; fade out over 0.2 s; the triggering character's position is read; entities 16 to 21, the six Evil Harrises in region 1403, get SetAI_Behavior(entity, 0) and are moved to random points of script group instance 1; the triggering character is deactivated; the one-entry slot (word 0x3710) is activated; entity 23 is moved to the read position; PlayCamera(track 7); fade in; a wait; effects are moved along the points of script group instance 2; the slot characters are deactivated and reactivated; entity 22 is moved and walks between points 46 + count and 52 + count; PlayCamera(track 9) for counts 1 to 3, track 8 for counts 4 to 7; Wait(5 s); fade out; SetAI_Behavior(entity, 5) for the six entities; fade in; ReturnToNormalCamera. When the count reaches 6 (once): stage 4 is marked done; PlayCamera(track 11, 1 s); DeactivateObj(8); plays objects 7 and 9 (wait); SetGameState(flag 723, true); thread 6 is started with argument 2; ReturnToNormalCamera (High for the calls and the count rule; Medium for the order inside a pass).
- Thread 6 (slot thread; started by Init with the first unfinished stage's argument 0, 3, 1 or 2, by routine 3 with argument 3, by the stage-2 Update helper with argument 1, by trigger 34 at count 6 with argument 2, and by thread 7 with argument 4): SlotReset(slot word 0x370c) and SlotLoad(slot word 0x370c, j = the thread argument, true), with a debug line. Its SlotCreate table reorders regions: arguments 0 to 4 load regions 1403, 1401, 1402, 1400 and 1399 respectively (High).
- Thread 7 (started by routine 4 with argument 0): starts thread 6 with argument 4 and sets character avoidance point 33 for the six entities 16 to 21 (High).
- The routine that sets AI behaviour 0 and moves those six entities with control kept is part of trigger 34's pass (above); it is not a separate entry (High).
- Update (each tick, Medium):
  1. Collision checks for entities 16 to 21 and 9 to 12 (CollideEntityWithTriggers for each).
  2. Undamageable flags set true for entities 0 to 4 and 16 to 21.
  3. Stage 2 helper (while stage 2 is not done): it counts how many of AI 13, 14 and 15 are dead. On a tick when the count from the previous tick is 3: stage 2 is marked done; PlayCamera(track 5, 1 s); plays object 5 and object 6 (wait); SetGameState(flag 721, true); thread 6 started with argument 1; Wait(1 s); ReturnToNormalCamera; InitMiamouseBehavior(15, 16) (High).
  3a. Stage 3 helper, the cage check (while stage 3 is not done): when the Miamice count in the cage (NumMiamiceInCage) is 4: stage 3 is marked done; PlayCamera(track 6, wait); DeactivateObj(8); plays object 7 (wait 1 s); SetGameState(flag 722, true); Wait(1 s); ReturnToNormalCamera (High).
  3b. Stage 5 helper (while stage 5 is not done): it counts how many of entities 9 to 12 stand on navmesh 7, 8, 9 or 10 (GetEntityCurrentNavmeshID). When all four do: the stage is marked done; SetGameState(flag 724, true); PlayFMV(67, wait); DeactivateObj(0); StartEffect(15); SetGameState(flag 356, true); SetGameState(flag 106, true) (High). Triggers 40 to 43 (one shared routine) count entities in and out while stage 4 is done and stage 5 is not.
  4. The stage-1 trap check (see routine 2) runs while stage 1 is not done and the trap is armed.
- Update-called routine: while stage 4 is armed and not done, Update calls a routine that sets the hit-point percentage of entities 16 to 18 to full.
- Edge cases: each stage helper is one-shot through its done word; a fresh module image starts with every done word 0 and Init sets it from the stage flags.
- Miamice stage (stage 3; High): the catch, carry and cage rules are those of SPEC-0029 Behavior 28, with field navmesh 15 and cage navmesh 16. Region 1400 holds four Miamice on navmesh 15, at (67.45, 40.00, 835.20), (-25.01, 40.00, 907.23), (-67.95, 40.00, 842.98) and (-33.40, 40.00, 931.19). The cage is navmesh 16, a near-rectangle x -2.54 to 2.15, z 915.85 to 919.57. The stage starts with InitMiamouseBehavior(15, 16), which sets the count to 0 and turns catching on; the cage check ends it when the count is 4, which means all four Miamice. The pack sets no level states for this stage. A reload of 1398 before the fourth delivery runs the setup again from Init (flag 722 still clear), so the count and the Miamice start over.
- Open: none for the pen list. Resolved: the Whinny table is the four-pen list above (High).

### 1066: Exterior, per-tick world masks with a button pass

- Family: Exterior. Exit exported (empty).
- Confidence: High for Init, Update and the four world-mask helpers.
- Init, in play: three trigger registrations at the top; a flag 17 test: when flag 17 is set, a debug line and mask helper A; when it is clear, a debug line, SetGameState(flag 16, true) and mask helper B; then a flag 414 test: helper C when flag 414 is clear, helper D when it is set (see Helpers for the values). Then SlotAllocate(4) and four SlotCreate calls (tables of 3, 2, 1 and 1 entries), each result stored in a module data word (0x38b0, 0x38b4, 0x38b8 and 0x38bc in call order). Then ControlThreadsAllocate(3), three ControlThreadStart calls, four proximity registrations and one more trigger registration. A debug-mode query inside a registrar that Init calls ten times gates only that registrar's record writes (no host call in its body).
- Update (each tick):
  1. If flag 2 is set: if object 0 is active, it is deactivated.
  2. Then two button tests, each with a true third argument: IsButtonPressed(button 0, newly pressed, true) would swap helpers A and B, and IsButtonPressed(button 3, newly pressed, true) would swap helpers C and D. A true third argument makes the call always false (SPEC-0033), so in play neither swap runs and the Update never changes the masks (High).
- Helpers (High): the module keeps a mask word (module data 0x38c8, file value 0). Each helper changes bits of that word and then calls SetWorldCollisMask(W, word) and SetWorldDisplayMask(W, word) with the new value. A: bit 0 off, bit 1 on. B: bit 1 off, bit 0 on. C: bit 2 on. D: bit 2 off. A and B also store 1 and 0 in a toggle word (0x38c0), and C and D store 1 and 0 in a second toggle word (0x38c4); only the dead Update swaps read them.
- World masks in play (High): set once per load by Init, from flags 17 and 414: 5 (flag 17 clear, 414 clear), 1 (17 clear, 414 set), 6 (17 set, 414 clear) and 2 (17 set, 414 set). Each helper sets or clears every bit it owns, so the result does not depend on the word's earlier value. The display word then holds twice the value (SPEC-0030 row 212). Bit 0 is the layer used before flag 17 (set when item 38 is handed over in pack 1148), bit 1 the layer used after it, and bit 2 a layer kept until flag 414 (set when entity 17 or 18 takes item 21). The pack's collision holds 134 entries with mask 0 (always used), 28 with mask 1, 19 with mask 2 and 3 with mask 4 (SPEC-0018 mask gate); its trigger index 0 volumes are 18 with mask 1 and 4 with mask 2. Exactly one of bits 0 and 1 is always on, so one of the two layouts is always present. A flag change during a visit takes effect at the next load (High for the values and counts; Medium for what each layer shows).
- Routines that init registers or starts (all live; each conversation line below is shown with the conversation bracket, High for the calls):
  - Line cycling: where a case below lists several lines, they are shown in turn on successive visits; each routine keeps a position counter in module data that restarts at the case's first line when the case changes (High).
  - Proximity callback C1 (entities 17 and 18, one routine). If flag 4 is set and flag 6 is clear: lines 23, 24, 25. Otherwise (flag 4 clear, or flags 4 and 6 both set), in this order: if flag 414 is set: line 26. Else if flag 410 is set: if the hero holds item 21: SetGameState(flag 414, true), RemoveItemFromHero(item 21, 1) and line 27; otherwise lines 28, 29, 30. Else if flag 409 is set: if the hero holds item 13: SetGameState(flag 410, true) and line 31; otherwise lines 32, 33, 34. Else: SetGameState(flag 409, true) and line 35 (High).
  - Proximity callback C2 (entity 16): flag 143 set: lines 14, 15, 16; else flag 127 set: lines 17, 18, 19; else lines 20, 21, 22, after which it repeats 21 and 22 (High).
  - Proximity callback C3 (entity 19): if flag 144 is clear, SetGameState(flag 144, true); then, in every case, one line with the conversation bracket, chosen by a position counter that starts at 0. The line ids on successive visits are 0, 1, 2, 3, 1, 2, 3, 1, 2, 3 and so on: counter 0 shows line 0 and sets 1; counter 1 shows line 1 and sets 2; counter 2 shows line 2 and sets 3; any other value shows line 3 and sets 1. Lines 0 to 3 are used by no other routine of this module (High).
  - Trigger routine T1 (index 3; object present, state 1): if flag 145 is clear: r = Random(0, 1); BlockPlayerControl; LevelMsg(line 4 + r) when flag 143 is clear, line 6 + r when it is set; ReturnHeroAnimToIdle; WalkHero(locator 75, wait, control kept) (High).
  - Trigger routine T2 (indices 0, 1 and 2; object present): if button 0 is not pressed, shows the square-button prompt (context 3); when pressed, LevelMsg of line 10 for index 0, line 11 for index 1 and line 12 for index 2 (High).
  - The per-tick entity loop is one of the three control threads that Init starts (see the description under SPEC-0042 module 986); in play it ends at once at its debug-mode query.
- Control thread (slot-set chooser, the first thread Init starts; runs once). It first picks a stage: 20 if flag 20 is set, else 19 if flag 19 is set, else 18, 17 or 16 by flags 18, 17 and 16 in that order, else none (no load). Then (all loads with activate true; set N loads region 1067 + N):
  - Stage 16: word 0x38b0 set 0; word 0x38b8 set 3; and, when flag 414 is clear, word 0x38b4 set 5.
  - Stage 17: word 0x38b0 set 0; word 0x38b4 set 5 when flag 414 is clear; word 0x38b8 set 3 when flag 145 is clear; word 0x38bc set 6 when flags 148 and 144 are both clear.
  - Stage 18: word 0x38b0 set 1.
  - Stage 19: word 0x38b0 set 2; and, when flag 414 is clear, word 0x38b4 set 4.
  - Stage 20: word 0x38b4 set 5 when flag 414 is clear.
  The four words are stored by the Init SlotCreate calls (see Init), so the loads act on those slots (High).
- Registrations: Init registers C2 for entity 16, C1 for entities 17 and 18, and C3 for entity 19 (four proximity registrations, three routines), T2 for trigger indices 0, 1 and 2, and T1 for index 3.
- Exit: empty.
- Edge cases: the update never changes the masks in play; with flag 2 clear it does nothing, and with flag 2 set it only deactivates object 0 when that object is active.

### 748: Interior, item-check dialogue and shared entity-control routine

- Family: Interior. Exit exported (empty).
- Confidence: Medium for the dialogue routines; High for Init and the entity-control routine.
- Init: a helper that writes records is called eight times; a debug-mode query inside that helper gates only those record writes (its body makes no host call). They form a debug flag and item list, not the entity table; the entity-table setup runs in play (see the shared routine section). In play, Init also makes: a blackboard message (type 2, value 20, world eW 20), a control-thread allocation and two control-thread starts. Init also registers, through a helper, the on-enter routine (index 0) and the proximity routine (entity 0). The Neopoints routines of this module sit inside the debug branch of the per-tick entity loop and never run in play (Open question 28).
- The helper registers Routine 1 as the on-enter routine (trigger index 0) and Routine 2 as the proximity routine (entity 0). The registration argument lists identify them (High).
- Routine 2 (proximity, entity 0; every line with the conversation bracket): if flag 407 is set: lines 8, 9 and 10 in turn. If flag 407 is clear and flag 405 is set: if the hero holds item 24, RemoveItemFromHero(item 24, 1), SetGameState(flag 407, true) and line 11; if not, lines 12, 13 and 14 in turn. If flags 407 and 405 are clear and flag 76 is clear: lines 16, 17 and 18 in turn. If flags 407 and 405 are clear and flag 76 is set: SetGameState(flag 405, true) and line 15. One position counter serves all the "in turn" lines of this routine; it is set to 0 once, on the first visit that finds flag 407 set. Each visit shows the counter's line and advances it (counter 0 shows the first line and sets 1; 1 shows the second and sets 2; 2 shows the third and sets 0) (High for the cases; High for the in-turn order).
- Routine 1 (on-enter, trigger index 0; every line with the conversation bracket): if flag 406 or flag 407 is set: lines 0 and 1 in turn. Otherwise, if flag 405 is set: lines 2 and 3 in turn. Otherwise, if flag 76 is set: lines 4 and 5 in turn; otherwise lines 6 and 7 in turn. One position counter serves all four cases; it is set to 0 once, on the first visit that finds flag 406 or flag 407 set. Each visit shows the counter's line and advances it (counter 0 shows the first line and sets 1; counter 1 shows the second and sets 0) (High).
- Shared entity-control routine: see its section.
- Threads (High: the routines are the arguments of the starts): the first Init start runs the debug-gated per-tick entity loop (the routine described under SPEC-0042 module 986): its first call is the debug-mode query, which returns false in play, so it ends at once; the three routines it calls inside its debug branch do not run in play. The second runs SetToNormalCamera(5.0, 0.0, true) once and ends; it is live (the module has no fade-in call). The loop-animation calls of this module are in reached code (see the shared routine section).
- Dormant routines: five routines, none with a host call of its own (four referenced only by dormant code, one with no reference). The load-time constructor's routines are not dormant: the load hook runs them at every load, and they clear the entity-control table as the Reset rule says.
- Update: calls the shared entity-control routine once per tick with the module's entity table (see the shared entity-control routine section; High for the call). Exit: empty.

### 1014: Interior, proximity pair, on-enter line and a flag-gated item removal

- Family: Interior. Exit exported (empty).
- Confidence: High for Init; Medium for the routines.
- Init:
  - Proximity routine (entity 2 and entity 0, same routine 1).
  - On-enter routine (index 3).
  - Proximity routine (entity 3 and entity 1, same routine 2).
  - Entities 2 and 3 added to the entity table (each add starts LoopEntityAnim(entity, 0, 30.0 s, control 1); waypoints in the shared routine section), in play; no gate in Init.
  - SlotAllocate(1); SlotCreate(2 entries, table in module data), whose result is stored in module data word 0x3928 (in play).
  - ControlThreadsAllocate(1); ControlThreadStart(thread, world) (in play).
  - Teleport routine registered (index 0, kind 13) and activate-world routine registered (in play).
- Routine 1 (proximity, entities 0 and 2): if flag 580 is set: SetConversationTarget; LevelMsg(line 1, bracket); EndConversation. If flag 580 is clear: if flag 578 is set and flag 579 is set: RemoveAllOfItemFromHero(item 83); SetGameState(flag 580, true); LevelMsg(line 3). If flag 578 is set and flag 579 is clear: LevelMsg(line 4). If flag 578 and 579 are clear: if flag 86 is set: LevelMsg(line 7); else LevelMsg(line 10).
- Routine 2 (proximity, entities 1 and 3, Josh): this is the routine that starts the Pink Posies quest. When flag 86 is set and flags 578 and 580 are clear, it sets flag 578 (the start flag of the Pink Posies row). It also shows the Josh lines: SetConversationTarget; if flag 86 is set: LevelMsg(line 16, bracket); else LevelMsg(line 19) with the bracket (Medium for the line choice; High for the flag 578 setter and its gate).
- Routine 3 (no conversation bracket; registered as the on-enter routine for index 3, Low): if flag 580 is set: LevelMsg(line 22). If flag 580 is clear and flag 578 is clear: if flag 86 is set: LevelMsg(line 28); otherwise LevelMsg(line 29). If flag 580 is clear and flag 578 is set: LevelMsg(line 25) and SetGameState(flag 578, true), which changes nothing because 578 is already set; this routine is the on-enter routine and is not the start-flag setter (that is routine 2). (The fall-through of these tests is partly described; Low.)
- Shared entity-control routine: see its section (records for entities 2 and 3).
- Thread (started by Init, runs once): SlotLoad(slot word 0x3928, set 1, true) when flag 86 is set, else set 0 (regions 1016 and 1015); then, when flag 298 is clear, the item price multiplier call (as 1008) (High).
- Teleport routine (index 0, kind 13): if the trigger index is 0: AddBlackboardMessage(type 2, value 40, eW 81) and StartLeaveInterior(eW 7, -1, -1, no colour), which leaves to pack 699; then Wait(3.0 s) in every case (High).
- Activate-world routine: if the blackboard value for eW 81 is 40: MoveHero(locator 0, true, CONTROL 1); when the value is not -1, SetToNormalCamera(5.0, 0.0, true); then AddBlackboardMessage(type 2, value -1, eW 81) (High).
- Update: calls the shared entity-control routine once per tick with the module's entity table (see the shared entity-control routine section; High for the call). Exit: empty.

### 1335: Exterior, six proximity routines, slot choices, a walk-to-entity sequence and movie triggers

- Family: Exterior. Exit exported (empty).
- Confidence: Medium for the proximity routines; High for the trigger and teleport calls and their flag gates below; Medium for the walk sequence.
- Init:
  - SetGameState(flag 235, true).
  - SlotAllocate(3); SlotCreate(6 entries, table in module data) and SlotCreate(1 entry, table in module data). The six-entry result is stored in module data word 0x39b4 and the one-entry result in word 0x39b8 (both in play).
  - Proximity routines (registered through a helper): entities 31, 32, 33, 34, 35 and 13.
  - Trigger routines: indices 3, 2, 1 and 0 (one shared routine); 5, 7, 6, 8, 9 and 4 (one routine each; the index-4 routine makes no host call).
  - Teleport routines (kind 13): indices 2, 1 and 0, one routine each.
  - Activate-world routine registered.
  - One object is initialised in play: object 1 when flag 241 is clear, object 0 when it is set.
  - ControlThreadsAllocate(5); then the slot-chooser thread is started in play, the walk thread when flag 92 is clear, and the line-37 thread when flag 239 is clear.
- Proximity routine for entity 34: Random(1, 3); if the result is not 1: SetConversationTarget; LevelMsg(line 3, bracket); EndConversation. If the result is 1, nothing is shown (Medium).
- Proximity routines: entity 31 shows line 15 with the bracket; entity 32 shows line 9; entity 33 shows line 12; entity 35 shows line 6; entity 13 shows line 24 with the bracket.
- Slot-chooser thread (started by Init, runs once): if flag 94 is set, nothing. Else if flag 93 is set: SlotLoad(word 0x39b4, set 2, true). Else if flag 92 is set: SlotLoad(word 0x39b4, set 1, true) when flag 241 is set, set 0 when it is clear. Else (flags 92 to 94 all clear): SlotLoad(word 0x39b8, set 7, true) (High).
- Shared trigger routine (indices 0 to 3): if flag 92 is clear, the state is 1 and the object pointer is non-null, it waits (one tick at a time) while a module busy word is set, then starts the slot thread with the trigger index. Slot thread: index 0 -> set 3, 1 -> set 4, 2 -> set 5, 3 -> set 6. If that set differs from the set it loaded last (module data, file value 0), SlotReset(word 0x39b4) and SlotLoad(word 0x39b4, set, true); the busy word is set while it runs (High).
- Trigger 5 (button gated): if button 0 is not pressed, the square-button prompt (context 32). When pressed: fade out (wait); if flag 235 is set and flag 92 is clear: SlotReset(word 0x39b4), SlotReset(word 0x39b8), SlotLoad(word 0x39b4, set 0, true), SetGameState(flag 236, true), SetGameState(flag 92, true), PlayFMV(20, wait); then, in every pressed case, Wait(1.0 s) and a fade back in (High for the calls).
- Trigger 6 (state 1, object non-null): if flag 236 is set and flag 237 is clear: SetGameState(flag 237, true); LevelMsg(line 30); LevelMsg(line 31); PlayCamera(track 5); LevelMsg(line 32); Wait(0.5 s); ReturnToNormalCamera (High).
- Trigger 7 (state 1, object non-null): if flag 240 is set and flag 241 is clear: SetGameState(flag 241, true); fade out; DeactivateObj(1); ActivateObj(0); SlotReset(word 0x39b4); SlotLoad(word 0x39b4, set 1, true); PlayFMV(25, wait) (High).
- Trigger 8: if flag 402 is set and flag 403 is clear: SetGameState(flag 403, true) and PlayFMV(21, wait) (High).
- Trigger 9: if flag 236 is set and flag 402 is clear: SetGameState(flag 402, true) (High).
- Teleport routine 0 (state 1, object non-null): if flag 236 is set and flag 93 is clear: LevelMsg(line 35) and no transition. Otherwise (flag 236 clear, or flags 236 and 93 both set): AddBlackboardMessage(type 2, value 20, eW 151) and StartExteriorReplacementLoad(eW 151, -1, no colour), which loads pack 1300 (High).
- Teleport routine 1 (state 1, object non-null): if flag 241 is set and flag 242 is clear: SetGameState(flag 242, true); AddBlackboardMessage(type 2, value 80, eW 150) and AddBlackboardMessage(type 2, value 80, eW 155); SetReturnWorldForFMV(eW 150), pack 1299; PlayFMV(26, wait). Otherwise LevelMsg(line 34) (High).
- Teleport routine 2 (state 1, object non-null): if flag 238 is set and flag 239 is clear: AddBlackboardMessage(type 2, value 70, eW 153) and AddBlackboardMessage(type 2, value 70, eW 155); StartExteriorReplacementLoad(eW 153, -1, no colour), which loads pack 1324. Otherwise LevelMsg(line 33) (High).
- Transitions: the module's transitions are the two exterior loads (eW 151, pack 1300, from teleport 0; eW 153, pack 1324, from teleport 2) and the movie return to eW 150 (pack 1299) after movie 26 (teleport 1). It makes no enter-interior or leave-interior call (High).
- Walk thread (started by Init when flag 92 is clear): SetAI_MovementType(36, 1); looks up script group instance 0. When the group has points, entity 36 walks along them (WalkEntity to a point read from the group record) and the routine tests the hero's distance to locators read from the record each tick. It shows line 21 (with wait) if the hero is not near, line 22 when the distance test passes, and line 23 after a wait of 10 s (Medium for the branch order).
- Line-37 thread (started by Init when flag 239 is clear): loops while flag 239 is clear. Each pass: if movie 22 has been played, it sleeps 3600 ticks (one tick at a time), shows LevelMsg(line 37) and sleeps 4 ticks; then it sleeps 4 ticks. So once movie 22 has played, line 37 repeats about every 3600 ticks until flag 239 is set (High).
- Activate-world routine: reads the blackboard value for eW 155: 70 -> MoveHero(locator 3, true, CONTROL 1); 20 -> MoveHero(locator 1, ...); 80 -> MoveHero(locator 2, ...); any other value moves nothing. Then AddBlackboardMessage(type 2, value -1, eW 155), and SetToNormalCamera(10.0, 0.0, true) when the value read was not -1 (High).
- Update (each tick): CharacterNavmeshAssign(36, 4); SetAI_MovementType(36, 1).
- Exit: empty.
- Edge cases: the update runs every tick and re-assigns the navmesh of entity 36 (Medium).
- Open: flags 92 to 94, 235 to 242, 402 and 403 and eW values 150, 151, 153 and 155 are not mapped to story meanings.

### 750: Interior, on-enter line, four proximity lines and a flag-gated line set

- Family: Interior. Exit exported (empty).
- Confidence: High for Init and the proximity routines; Medium for the on-enter routine.
- Init:
  - On-enter routine (index 2).
  - Proximity routines (registered through a helper that Init calls, with the on-enter routine): entity 3 (routine 1), entity 1 (routine 2), entity 0 (routine 3) and entity 2 (routine 4).
  - Debug-mode query: a helper that writes records is called seven times; the query gates only those record writes (no host call in the helper body; they form a debug flag and item list, not the entity table). In play, Init also makes: a blackboard message; a slot allocation and one slot creation whose result is stored in the slot word that the control-thread routine described below loads from; a control-thread allocation; and two control-thread starts.
- Line cycling (routines 1 to 4 and the on-enter routine): every line below is shown with the conversation bracket (SetConversationTarget; LevelMsg; EndConversation). Where a case lists several lines they are shown in turn on successive visits. Each routine keeps one position counter in module data, shared by its cases (so a case can start at its second or third line); the counter is set back to the first line once, the first time the routine's top case (flag 420; flag 421 for the on-enter routine) is taken (High).
- Routine 1 (entity 3): flag 420 set: lines 0, 1, 2. Flag 420 clear and flag 76 set: lines 3, 4, 5. Otherwise lines 6, 7, 8.
- Routine 2 (entity 1): flag 420 set: lines 9, 10, 11. Flag 420 clear and flag 76 set: lines 12, 13, 14. Otherwise lines 15, 16, 17.
- Routine 3 (entity 0): flag 420 set: lines 18, 19, 20. Flag 420 clear and flag 76 set: lines 21, 22, 23. Otherwise lines 24, 25, 26.
- On-enter routine (index 2): flag 421 set: lines 27, 28. Else flag 420 set: lines 29, 30. Else flag 76 set: lines 31, 32. Otherwise lines 33, 34 (two lines in turn per case).
- Routine 4 (entity 2): flag 420 set: lines 35, 36, 37. Else, if flag 419 is set: SetGameState(flag 420, true) and line 38 (so the next visit takes the flag-420 case). Else flag 76 set: lines 39, 40, 41. Otherwise lines 42, 43, 44 (High).
- Shared entity-control routine: see its section (record for entity 2).
- Control-thread routine (the first of the two control threads that init starts; the second runs the debug-gated per-tick entity loop (the routine described under SPEC-0042 module 986): its first call is the debug-mode query, which returns false in play, so it ends at once; the three routines it calls inside its debug branch do not run in play): if flag 76 is clear: SlotLoad(slot word, set 0, true), on both the flag-75-set and the flag-75-clear paths (the two branches make the same call); if flag 76 is set, no load. Then SetToNormalCamera(5.0, 0.0, true) in every case. The slot word (module data offset 0x3b38) is stored by the slot creation in Init, which runs in play (see Init), so this load acts on that slot (High).
- Update: calls the shared entity-control routine once per tick with the module's entity table (see the shared entity-control routine section; High for the call). Exit: empty.

### 1122: Exterior, per-tick navmesh assignments, a wave sequence and the Faerie Sisters arena

- Family: Exterior. Exit exported (empty).
- Confidence: High for Update; High for the Init calls, the triggers and the arena thread's calls and stage tests listed below; Medium-High that this module is the Faerie Sisters arena (SPEC-0011).
- Init, in play: triggers 0, 5, 1, 2 and 3 (one routine each); a flag 31 test for objects: when flag 31 is set, object 2 is initialised and object 4 is initialised with argument 1 when flag 320 is set and 0 when it is clear; when flag 31 is clear, objects 1 and 4 are initialised (argument 0). Then SlotAllocate(3); two SlotCreate calls (3 entries each), whose results are stored in the two slot words used below (module data words 0x3b3c and 0x3b40, in call order); ControlThreadsAllocate(3, or 4 when flag 31 is set); ControlThreadStart of the slot thread and of the per-tick entity loop. When flag 31 is set, Init also starts the arena thread, registers trigger 6 and registers the activate-world routine. Last, proximity routines for entities 10, 7, 8 and 9 (one routine each). A debug-mode query gates only the record writes in a registrar that Init calls three times and in a helper that it calls twice (no host call in their bodies).
- Update (each tick): CharacterNavmeshAssign(slot, mesh) for the listed slots with fixed mesh values: slots 0, 1, 2 and 3 get mesh 0; slots 4 and 9 get mesh 4; slots 5 and 8 get mesh 3; slots 6 and 7 get mesh 2; slots 11 to 18 get mesh 1 (slot 10 is not assigned). Medium.
- Proximity routines (entities 10, 7, 8 and 9), each line with the conversation bracket. Lines of a case are shown in turn on successive visits (a module data counter that restarts at the case's first line when the case changes). Entity 10: flag 29 set: lines 1, 2, 3; else flag 28 set: line 4; else flag 27 set: lines 5, 6, 7; else lines 8, 9, 10. Entities 7, 8 and 9 test flags 29, 28 and 24 in the same way, with lines 11 to 20 (entity 7), 21 to 30 (entity 8) and 31 to 40 (entity 9): flag 29 the first three lines of the block, flag 28 the fourth, flag 24 the next three, otherwise the last three (High).
- Slot thread (started by Init, runs once): if flag 31 is set: SlotLoad(word 0x3b3c, set 0, true); else if flag 30 is set: set 1; else set 2 (High).
- Trigger 0: ActivateTransitionAssets (no gate). Triggers 1, 2 and 3: RegisterTransitionMap(50), (51) and (52) respectively, with no gate (High).
- Trigger 5 (the wave sequence; object non-null, state 1, flag 320 clear, and a module latch word with file value 0 still 0; the routine stores the state value 1 into the latch, so it runs at most once per module load). In order:
  1. PlayCamera(track 4); MoveHero(locator 25, with control); DeactivateObj(1); ActivateObj(2); Wait(0.5 s); play object 2 (wait); Wait(1 s); ActivateObj(0); DeactivateObj(4); ActivateObj(5); PlayCamera(track 5); Wait(0.5 s); play object 5 (wait); Wait(1 s); ReturnToNormalCamera; ActivateObj(4).
  2. Wave 1: SlotLoad(word 0x3b40, set 3, true); SetAI_Agro and DisableInvisibility for AI 11, 12 and 13; then it checks every 5 ticks until every character of that slot is dead (IsAI_Dead over SlotCharacterCount).
  3. Wait(3 s); SlotReset(word 0x3b40); SlotLoad(word 0x3b40, set 4, true); agro and visible for AI 14, 15 and 13; wait until all are dead (as above).
  4. Wait(3 s); SlotReset; SlotLoad(word 0x3b40, set 5, true); agro and visible for AI 17 and 18; wait until all are dead.
  5. Wait(3 s); SetGameState(flag 320, true); MoveHero(locator 24, with control); DeactivateObj(5); ActivateObj(4); PlayCamera(track 5); Wait(0.5 s); play object 4 (wait); Wait(1 s); DeactivateObj(2); ActivateObj(1); PlayCamera(track 4); Wait(0.5 s); play object 1; DeactivateObj(0); ReturnToNormalCamera.
  (High for the calls and their order.)
- Activate-world routine (registered when flag 31 is set): if flag 31 is set, SetLevelState(4, true) (High).
- Arena progress (Faerie Sisters fight, boss-fight instance 11; see SPEC-0011 "Arena flow"). The arena thread keeps six stage bits in a module data word (file value 0): bit 0 entry requested, bit 1 entry done, bit 2 shield break seen, bit 3 movie 53 played, bit 4 all sisters dead seen; bit 5 is tested but never set.
  - Trigger 6 (registered when flag 31 is set): when the record's word at +0x1C equals 1, it sets stage bit 0. It makes no host call (High).
  - The arena thread loops forever, one tick per pass (Sleep(1) at the end of each pass). Each pass, when flag 320 is set:
    1. It deactivates entities 4 to 18 (DeactivateEntity for each, every pass).
    2. A debug-mode branch (button 2 moves the hero to locator 23) does not run in play.
    3. It tests level state 321 with QueryLevelState. The index is outside 0 to 63, so the query returns 0 (SPEC-0030 row 153) and the step below always runs. (Game flag 321, which the boss end sets, is a different store.)
    4. It runs one of four steps, by the stage bits:
       - Entry (bit 0 set, bit 1 clear): fade out over 0.1 s (wait; colour from module data); PlayFMV(52, wait); fade out over 0 s (wait); BossFightTweakHero(instance 11, stage 0), the boss-fight refill; MoveHero(locator 26, with control); SetToNormalCamera(8.0, 3.14, true); entities 4 to 18 deactivated; ActivateObj(3); SetHeroEquippedState(on); fade in over 0.5 s (wait); Wait(0.5 s); stage bit 1 set; SetLevelState(4, false), which clears level state 4.
       - Else shield broken (bit 2 set, bit 3 clear): PlayFMV(53, wait); stage bit 3 set. This plays movie 53 once.
       - Else boss end (bit 4 set, bit 5 clear): fade out over 0.3 s (wait); SetGameState(true) for flags 321, 93 and 322, in that order; BossFightTweakHero(instance 11, stage 1); AddBlackboardMessage(type 2, value 50, eW 151); SetReturnWorldForFMV(eW 151), pack 1300; PlayFMV(54, wait). The routine sets no stage bit; the movie's return to eW 151 leaves the arena (Medium for the world change ending the loop).
       - Else (none of the above): the level-state check, then the hero-down check. Level-state check: if level state 1 is set (the sisters' event 1, shield broken) and bit 2 is clear, set bit 2; if level state 2 is set (all sisters dead), set bit 4. Hero-down check, only while bit 1 is set and bit 3 is clear (from the entry until movie 53 has played): if hero 1 is dead and the hero does not hold item 50: SetHeroSwapMode(1) when the active hero is hero 1, otherwise SetGameOver(0).
  - So, in play: trigger 6 starts the entry on the next pass; level state 1 leads to movie 53 one pass after it is seen; level state 2 leads to the boss end one pass after it is seen (High for the order).
- Per-tick entity loop: the module's other Init thread (see SPEC-0042 module 986); in play it ends at once at its debug-mode query. Routines that static code reachability does not reach (four) are not run by the clean side.
- Exit: empty.

### 1173: Exterior, teleport and arrival with a button-gated trigger

- Family: Exterior. Exit exported (empty).
- Confidence: Medium for the trigger and teleport routines; High for the Init calls listed below.
- Init, in play: object 1 initialised; trigger index 1 (routine 2); activate-world routine; ControlThreadsAllocate(1); ControlThreadStart(thread 4, world); a read of flag 630; trigger index 0 (routine 3); teleport routine (index 0, kind 13); entity 0 proximity routine (routine 4), registered through a helper; and object setups under flag tests. A debug-mode query gates only the record writes in a helper that Init calls three times (no host call in its body).
- Routine 2 (trigger index 1): when the record's trigger index is 1 it works on object 1 and locator 7, and tests once whether flag 162 is set and flag 167 is clear. If object 1 is active: when button 0 is not pressed, the square-button prompt (context 4). When it is pressed and that flag test held: WalkHeroAndOrient(locator 7, wait, control); PlayHeroAnim(140); play object 1 (wait); DeactivateObj(1); ActivateObj(2); then PlayCamera(track 4); PlaySound(11); play object 0; a wait; SetGameState(flag 630, true); ReturnToNormalCamera. When it is pressed and the flag test did not hold: LevelMsg(line 0) (Medium-High: the calls and gates are certain; the wait lengths are about 1 s).
- Routine 3 (trigger index 0): if flag 630 is set and the trigger's object is non-null: if the confirm button is not pressed: DisplaySquareButtonContext(context 13). If it is pressed: AddBlackboardMessage(type 2, value 120, eW 124) and StartExteriorReplacementLoad(eW 124, -1, no colour), which loads pack 1171 (Medium).
- Teleport routine (index 0, kind 13): AddBlackboardMessage(type 2, value 110, eW 119) and StartExteriorReplacementLoad(eW 119, -1, no colour), which loads pack 1152, with no gate (High).
- Transitions: two exterior loads: the teleport routine to eW 119 (pack 1152), and routine 3 to eW 124 (pack 1171) when flag 630 is set and the confirm button is pressed (High).
- Entity 0 proximity routine (routine 4): if flag 30 is set: SetConversationTarget; LevelMsg(line 1, bracket); EndConversation. Else LevelMsg(line 4).
- Activate-world routine: reads the blackboard value for eW 126. 120: MoveHero(locator 2, true, CONTROL 1); 110: MoveHero(locator 0, true, CONTROL 1); 130: MoveHero(locator 1, true, CONTROL 1) and SetGameState(flag 319, true); other values move nothing. Then, in every case, AddBlackboardMessage(type 2, value -1, eW 126) and SetToNormalCamera(20.0, 0.0, true) (High).
- Thread 4 (started from Init, with no gate): the debug-gated per-tick entity loop (the routine described under SPEC-0042 module 986): its first call is the debug-mode query, which returns false in play, so it ends at once; the three routines it calls inside its debug branch do not run in play. The loop animation on the entity list is made by Init's idle-loop helper, not by this thread.
- Dormant: a routine that shows line 9 with the bracket.
- Update: calls the shared entity-control routine once per tick with the module's entity table (see the shared entity-control routine section; High for the call). Exit: empty.
- The camera, hero animation, sound, object and flag-630 calls found in this module's reached code are those of routine 2 above.

### 1358: Exterior, six teleports, proximity dialogue set and boss-area triggers

- Family: Exterior. Exit exported (empty).
- Confidence: High for the teleports, the entity 20 and 21 routines, triggers 0 to 2 (prize wheel), triggers 3 to 5 and 7, the activate-world routine and the slot thread; Medium for the entity 16 to 19 routines.
- Init:
  - Activate-world routine registered.
  - Teleport routines: index 4, 3, 1, 0, 2 and 5 (kind 13).
  - Trigger routines at indices 5 and 7 (the sign routine, routine 20) and 3 and 4 (the slot-swap routine, routine 19) (High).
  - Proximity routines: entity 20 (routine 1), entity 21 (routine 2), entity 16 (routine 3), entity 17 (routine 4), entity 19 (routine 5), entity 18 (routine 6).
  - SlotAllocate(2) runs in play. Then, under the flag-102 gate, SlotCreate with a 1-entry table (result stored in module data word 0x3ef0) and SlotCreate with a 2-entry table (result stored in word 0x3ef4); when flag 102 is clear, one SlotCreate with a 1-entry table (result stored in word 0x3ef0). These are the slot words of routine 18 and of the flag-102 loads below (Medium).
  - Under the flag-102 gate, three more triggers are registered (indices 1, 2 and 0). Init therefore registers seven triggers: four in play and three when flag 102 is set.
  - The Random calls reached from Init are those of the wheel set-up (Random(0, 7) draws; the new pattern of SPEC-0059, section 2), which Init runs last (High).
  - Init allocates control threads (ControlThreadsAllocate(2)) and starts the slot thread in play (flag-102 loads, below). Routine 19 starts a further control thread under a flag test (see below). Init also initialises object 0 in play and then runs the wheel set-up.
- Proximity routine 1 (entity 20; High): the giver of the eight delivery tasks (task flags 605 to 612, done flags 679 to 686, intro flag 604; no prize), specified in SPEC-0059, section 7, with the task draw defect (the first listed task is never offered, and a draw of n reads past the list). Every line is shown with the conversation bracket (SetConversationTarget; LevelMsg; EndConversation). The done flags are set by the residents in packs 1421, 1417, 1407, 1419, 1423, 1425, 1405 and 1353.
- Proximity routine for entity 21: SetConversationTarget; LevelMsg(line 90, bracket); EndConversation.
- Trigger 1 (registered when flag 102 is set): if the state is 1, the object pointer is non-null, the record's trigger index is 1 and a module latch word (file value 0) is 0: it stores 1 in the latch and shows a line for entity 21 with the conversation bracket: line 87 the first time, line 88 the second time, nothing after that until the module data resets (SPEC-0059, section 1). Trigger 2 (registered when flag 102 is set) sets the latch back to 0 (High).
- Trigger 0 (registered when flag 102 is set; High): the prize wheel, 1358 copy (fee 100, too-poor line 109, start prompt 93, pay-outs 140, 180, 250, 400 and 700 NP, no result flags, win sound 7 and loss sound 6, object 0 turned about locator 1's forward axis), specified in SPEC-0059, section 2. The wheel set-up that Init runs last is that section's new pattern.
- Proximity routines for entities 16 to 19: a Random(1, 4) draw is made when flag 105 is set; the draw result 1 selects one line set and any other value another. Entity 16 uses lines 75, 78 and 81; entity 17 uses lines 63, 66 and 69; entity 19 uses lines 39, 42 and 45; entity 18 uses lines 51, 54 and 57 (Medium for the exact pairing).
- Teleport routines (kind 13). Each shows the square-button prompt (context 13) and acts when button 0 is pressed. Index 4: blackboard value 50 for eW 180, StartExteriorReplacementLoad(eW 180), Wait(3 s).
  - Index 3: blackboard value 10 for eW 180, StartExteriorReplacementLoad(eW 180), Wait(3 s).
  - Index 2: blackboard value 40 for eW 173; SetGameState(flag 348, true) when flag 348 is clear; StartExteriorReplacementLoad(eW 173) and Wait(3 s) in either case (the flag does not gate the load).
  - Index 5: if flag 695 is set: blackboard value 60 for eW 173, StartExteriorReplacementLoad(eW 173), Wait(3 s); otherwise LevelMsg(line 0).
  - Indices 0 and 1 (interior entry): if flag 102 is set: blackboard value 20 (index 0) or 30 (index 1) for eW 159 and StartEnterInterior(eW 181 or eW 183 respectively), Wait(3 s); otherwise LevelMsg(line 0) (High for the teleport calls).
- Activate-world routine (High): reads the blackboard value of type 2 for eW 159 (this module's world) and moves the hero with control: 10 to locator 2, 20 to 3, 30 to 4, 40 to 5, 50 to 6, 60 to 77. The value is reset to -1, and when it was not -1 the camera is set to normal (SetToNormalCamera(15.0, 0.0, true)). These are the branches once listed as table-selected.
- Routine 19, the slot swap (triggers 3 and 4; High): when the state is 1, the object pointer is non-null and flag 102 is set, it starts the slot-swap thread (routine 18) with choice 0 (index 3) or 1 (index 4). The thread acts only when the choice differs from the current one (module data, file value -1): it resets the two-entry slot (word 0x3ef4) and loads set 2 (choice 0, region 1361) or set 3 (choice 1, region 1362), activate true, and stores the new choice.
- Flag-102 loads (the slot thread that Init starts; runs once): SlotLoad(word 0x3ef0, set 1, true) when flag 102 is set, set 0 when it is clear (High).
- Routine 20, the sign routine (triggers 5 and 7; High): while the trigger state is non-zero and the object is present, one tick per pass: the square-button prompt (context 3); on button 0, line 2 (index 5) or line 4 (index 7).
- Update: empty. Exit: empty.
- Open: the story meanings of flags 100, 102, 104, 105, 106, 348 and 695. Flags 604 to 612 and 679 to 686 are the delivery-task flags (SPEC-0059, section 7).

## Edge cases and error handling

- Null world registration: refused, nothing registered (SPEC-0030). No module here relies on a null-world registration.
- Trigger index not in the table: ignored (SPEC-0030).
- Debug-mode query: in this spec it gates only record writes inside helpers or registrars that Init calls, in 748, 749, 750, 752, 1066, 1122 and 1173 (their bodies make no host call). It does not gate any host call that Init makes, so those calls run in play. The debug query returns false in this build, so the gated record writes are not made in play (SPEC-0030). In thread A of 1008, 1011 and 1020 the query guards only debug prints; the slot loads there run in play (High).
- Dormant routines: each of the fifteen modules holds routines that static code reachability does not reach. In 956, 1008, 1011, 1020 and 1335 every such routine is dormant. In 748, 749, 750, 752, 1014, 1066, 1122, 1173, 1358 and 1398 several routines that have no export name are live: they are registered or started by Init, or called from Update, a registered callback or a control thread, as the sections above say. In 748, 749, 750, 752, 1066, 1122 and 1173 three of them sit inside the debug branch of the per-tick entity loop and do not run in play. A reimplementation must not run the dormant ones.
- Update work: 956, 1066, 1122, 1335 and 1398 make host calls in Update; 748, 749, 750, 752, 1008, 1011, 1014, 1020 and 1173 call the shared entity-control routine every tick. Only 1358 has an empty Update (High).
- Exit: all fifteen export an empty Exit. Nothing is run for them on exit (SPEC-0030).
- Slot words without a store: a slot word that no reached code stores into holds -1 and gives a load or reset that does nothing (SPEC-0030 rows 205 and 208). In this spec this applies to the set-1 word of 956 (thread 3), which has one set-1 load and no store anywhere in the module, so region 958 is never loaded (High); and to the S word of 956 when flags 248 and 67 are both set (Init then stores only K), so thread 3's reset in that case also does nothing (High). Word 0x3b38 of 750 is stored by a slot creation in Init that runs in play (High); its only load is in the control-thread routine described in the 750 section, which init starts (static code reachability). The words of 1008, 1011 and 1020 are stored by the SlotCreate call in Init, which runs in play, and the word of 1122 by a SlotCreate call in Init that also runs in play. Every other slot word in this spec is stored by a SlotCreate call in reached code. A slot word is never given a fixed -1 (see this spec statement).
- Slot word values: the stored value is the SlotCreate result; the first SlotCreate in a world returns 0 (SPEC-0030 row 207).
- Module data lifetime (SPEC-0030 section 1.1; High): an Exterior module's words (latches, counters, choice and first-update words) survive trips into an interior and back, swaps between the exterior and transition buffers, and resident streaming; they go back to their file values when the exterior is loaded by the world driver, when play streams to another exterior, and on a new game. Interior modules are fresh on every entry. Init runs only at the start of a lifetime, so it always sees file values. "Once per module load" in this spec means once in this lifetime.
- Blackboard value -1: used as a reset after a read in 1008, 1011, 1014, 1020, 1173, 1335 and 1358 (SPEC-0037 pattern).
- Level-state index out of range: 1122's arena thread queries level state 321, which is outside 0 to 63, so the query always returns 0 (SPEC-0030 row 153) and does not gate anything in play.
- Line numbers with an alternative ("line N; line M on another path"): the line number is taken from a module data cell on some paths, so the listed line is not the only possibility (Medium).
- Debug lines: have no game-state effect and may be skipped.
- Money and hit-point checks: the 1358 prize wheel needs at least 100 Neopoints (GetNeopoints) and takes 100 (RemoveNeopoints) when the first prompt is accepted; the 1398 stage-1 trap compares the hero's hit points (GetHitPoints) with the previous tick (see those sections).

## Test vectors

- 1008, activate-world with blackboard value 20 for eW 81: MoveHero(locator 0, true, CONTROL 1), SetToNormalCamera(5.0, 0.0, true), then the value for eW 81 is reset to -1.
- 1011, activate-world with value 30 for eW 81: MoveHero(locator 0, ...) as above. With value 20: nothing.
- 1020, teleport routine with trigger index 0: AddBlackboardMessage(2, 70, eW 81), StartLeaveInterior(eW 7, -1, -1, no colour), Wait(3.0 s).
- 1014, proximity routine 1 with flag 580 set: SetConversationTarget, LevelMsg(line 1), EndConversation.
- 750, routine 1 (entity 3), first visit with flag 420 clear and flag 76 set: LevelMsg(line 3) with the conversation bracket when the routine's counter is at its start; the next visit in the same case shows line 4.
- 750, routine 4 (entity 2) with flag 420 clear and flag 419 set: flag 420 is set and line 38 is shown; the next visit shows line 35.
- 1398, Update with the boss-cage count at 4 and entity 13 dead, flag 722 clear: camera 6 played (wait), object 8 deactivated, object 7 played (wait 1 s), flag 722 set, Wait(1 s), camera returned.
- 1066, Update with flag 2 set and object 0 active: object 0 deactivated on that tick.
- 1066, Init with flag 17 clear and flag 414 clear: flag 16 set; collision mask 5, display word 10. With flag 17 set and flag 414 set: collision mask 2, display word 4. Square or Cross pressed during the visit: masks unchanged.
- 1335, Update: CharacterNavmeshAssign(36, 4) and SetAI_MovementType(36, 1) on every tick.
- 956, first Update with flag 281 set: targets 0, 1 and 2 made active; later Updates leave them alone.
- 956, thread 3 with flags 248 and 67 set: the SlotReset reads the S word, which Init did not store in this case, and does nothing; the set-1 SlotLoad reads the set-1 word, which is never stored, so it returns 0 and loads nothing (region 958 is not loaded).
- 956, trigger 3 entered (state 1) with flags 214, 248 and 64 clear: flag 208 clear loads region 962 (record 0, j 5) into Z until the hero leaves the zone; flag 208 set loads region 963 (record 4, j 6). With flag 214 set, nothing is loaded.
- 1398, thread 6 started with argument 3: slot reset, then region 1400 is loaded through table index 3.
- 1358, proximity routine 2 (entity 21): LevelMsg(line 90, bracket) with the conversation bracket.
- 1358, prize wheel and delivery tasks: see the test vectors of SPEC-0059.
- 1122, arena: trigger 6 fires with the record word at +0x1C equal to 1 while flag 320 is set: on the next pass, movie 52, BossFightTweakHero(11, 0), hero to locator 26, level state 4 cleared. Later level state 1 set: movie 53 on the next pass, once. Level state 2 set: flags 321, 93 and 322 set, BossFightTweakHero(11, 1), blackboard (2, 50, eW 151), return world eW 151, movie 54.
- 1122, hero-down check after the entry and before movie 53, hero 1 dead, item 50 not held, active hero 1: SetHeroSwapMode(1); active hero 2: SetGameOver(0).
- 956, trigger routine 1 entered on index 0 with flag 67 set and the choice word at -1: the word becomes 1 and thread 4 resets the slot and loads set 12 (region 969). Entering index 0 again does nothing.
- 1066, slot chooser with flag 17 set (flags 18 to 20 clear), flags 414, 145, 148 and 144 clear: sets 0, 5, 3 and 6 are loaded (regions 1067, 1072, 1070 and 1073).
- 1066, entity 19 callback on seven successive visits: lines 0, 1, 2, 3, 1, 2, 3, each with the conversation bracket.
- 748, on-enter routine (Routine 1) with flag 406 set, on three successive visits: lines 0, 1, 0, each with the conversation bracket.
- 748, proximity routine (Routine 2) with flag 407 set, on four successive visits: lines 8, 9, 10, 8.
- 748, proximity routine (Routine 2) with flag 407 clear, flag 405 set and item 24 held: RemoveItemFromHero(item 24, 1), SetGameState(flag 407, true), line 11 with the bracket.

## Open questions

1. Slot words (all modules): each slot word holds the SlotCreate result that reached code stores into it. The meaning of the SlotCreate result for each slot beyond the first is SPEC-0030 row 207 (open for this spec as a whole). The only never-stored word is the set-1 word of 956, and its S word is unstored when flags 248 and 67 are both set (see Edge cases); the slot creation for word 0x3b38 of 750 runs in play. Whether any other store reaches a slot word through another address is not established for every word (Open; SPEC-0030 question 26).
2. Activate (fifth) SlotLoad argument: its meaning is known (SPEC-0030 row 205), and it is true (1) in every resolved call of this spec. The set value that comes from module data or from a routine argument is not identified for 1358 routine 18. Resolved for 956 (threads 5 and 6: the record table; thread 4: sets 10 to 12), 1398 (thread 6: arguments 0 to 4, mapped through its reordered region table), 1335 (every load names a fixed set) and the 1358 flag-102 loads (sets 1 and 0).
3. Resolved: trigger record fields +0x20 and +0x24. +0x20 is 1 when the hero shot pass fired the trigger (object word 0; spheres, cones and cylinders with flag bit 3, static rectangles with bit 8 and moving rectangles with bit 5) and 0 on every other fire path. +0x24 is the hero's attack-active word when the hero's own body contact fired it (object word 1), and 0 on every other path (SPEC-0030 section 3). In this spec only 1398 trigger 34 tests them (all of +0x1C, +0x20 and +0x24 must be 0, so no hero fire passes).
4. Meanings of the flags tested here: 2, 4, 6, 16, 17, 20, 27 to 31, 59, 62 to 67, 75, 76, 86, 92, 93, 94, 102, 105, 106, 143, 144, 145, 148, 214, 216, 235, 236, 237, 239, 240 to 242, 248, 265, 266, 281, 283, 298, 319, 320, 321, 348, 355, 405, 406, 407, 414, 419 to 421, 578, 579, 580, 604 to 607, 630, 679 to 681, 695, 720 to 724.
5. Resolved for entities: entity ids are per pack, and each resolves through the pack's entity table to a placement name (SPEC-0006 "Entity names"); entities 0 to 5, 13, 16 to 21, 24, 28, 31 to 36 and 45 to 51 are named that way in each module's own pack (for example 749 entity 0 is Brigson and 750 entity 2 is Primrose) (High). Still open: trigger identifiers have no name source on disc. Locators 54 to 61 in 956 are the hero and knight places of the cage scenes (knight records); what they are in the level is not mapped.
6. Resolved: each eW id is an area slot; the slot names its pack and the pack's map area gives its name (SPEC-0006 "Area names", SPEC-0051 "Map area names"). E exterior, I interior, T transition: 7 = 699 T, Western Brightvale; 20 = 722 E, Bogshot Village; 81 = 991 E, Market Town; 119 = 1152 E, Meridell Castle 1st Floor; 124 = 1171 E, Meridell Sewers; 126 = 1173 E, no map area, adjoins Meridell Treasure Vault, Meridell Castle 1st Floor and Meridell Sewers; 150 = 1299 E, Fyora's Library; 151 = 1300 E, Fyora's Palace Gardens; 153 = 1324 E, Fyora's Clocktower; 155 = 1335 E, Fyora's Palace; 159 = 1358 E, Money District; 173 = 1397 E, Treasury; 180 = 1414 E, no map area, adjoins Money District; 181 = 1417 I and 183 = 1421 I, inside Money District; 184 = 1423 I, inside Park District (High).
7. Resolved for item names (SPEC-0021 "Item names"): 24 Kreludan Metal; 57 Starlight Potion; 58 Kauvara's Potion; 59 Fyora's Potion; 83 Pink Posies; 102 Battle Blade; 109 Golden Shield; 115 Magician's Wand; 154 Starry Negg; 174 Red Negg (High). Still open: the meaning of the item-activation argument.
8. Resolved: the shared entity-control routine's record layout, per-tick rules and per-module records are in its section (High). Still open: whether the refused adds of 1411 (entities 3 and 5) still start their idle loop.
9. Resolved: the four 1066 world-mask helpers set bits 0 to 2 of one mask word (see the 1066 Helpers and World masks in play). Still open: what each masked layer looks like in the level.
10. Resolved: the 749 loop animation runs in play (Init calls its setup helper with no branch). The registrations of 749 and 1122 run in play (see those sections).
11. 752: none for the proximity registration, which a helper that Init reaches makes (see the 752 section).
12. Resolved: the trigger and teleport routines of 1335 and 1398 and the teleport routines of 1358 are now described by index (see those sections).
13. Dormant routines (see Edge cases): whether any host path starts them.
14. Lines of the world text: line numbers are as given; the text is in the world's text (not copied).
15. Resolved in part: the Miamice cage of 1398 is navmesh 16 and the stage needs a count of 4, all four Miamice of region 1400 (1398 section, Miamice stage; High). Still open: the Miamice identity of entity 13.
16. Resolved: the activate-world value-to-locator mappings of 1335 and 1173 are given in their sections.
17. Resolved: the 1066 world masks are 5, 1, 6 or 2 by flags 17 and 414, set once by Init; the Update button swaps never run in play because their third argument is true (High).
18. Resolved: the 1066 line chosen by a draw of 0 or 1 is trigger T1 (index 3): line 4 + r or 6 + r by flag 143.
19. Resolved: in 956 the cage scenes take their knight, locators, guard, lock and camera from the knight records (see the 956 section).
20. Resolved: in 1398 the routine that moves the six entities 16 to 21 is part of trigger 34's pass, and the set-5 load (keep false, region 1404) is one of the two ungated Init thread starts.
21. Resolved: the 1173 camera, hero animation, sound, object and flag-630 calls belong to routine 2 (trigger index 1).
22. Resolved: in 1358 the flag-102 loads are made by the slot thread that Init starts, and the branches once called table-selected are the activate-world value mapping (see the 1358 section).
23. Resolved: the family is the compiler's software double-precision library: an unpack routine (a type word plus mantissa and exponent words), add and subtract, and a pack routine. The console's floating-point unit has no double instructions, so a module that mixes a float with a double constant links the library. The unpack and pack routines are identical byte for byte in 52 level modules, including 748, 749, 750, 752 and 1358 of this spec. Module code calls them only to widen a float, add or subtract a double and narrow the result again; for example the hazard timer of module 1090 drops by 1.0 per tick this way (SPEC-0041). They carry no game data, and no money or score value is a tagged number (High).
24. Resolved: the 956 shared record loop is the cage trigger of the knight records, and the straight-line cutscene is the aftermath scene that the Update runs once (see the 956 section).
25. Resolved: the set-1 word of 956 is written by no instruction in the module (every use is a read), so its load never runs (High). The slot creation for word 0x3b38 of 750 runs in play (see the 750 section).
26. Ordering: the Init store into the S word (956) happens before Init starts thread 3. Store-before-load order across this spec follows the role of each entry (init first, then callbacks and threads) and is not established per module (Medium).
27. Resolved: 1066 word 0x38bc (the fourth SlotCreate result) is loaded with set 6 by the slot-set chooser at stage 17 when flags 148 and 144 are both clear.
28. Resolved: the money routines of 748 (the Neopoints check, give and remove helpers and the button handler that calls them) are reached only from the debug branch of the module's per-tick entity loop, which the debug query keeps closed in play, and nothing registers them (High). In 1358 every routine with message, sleep or prompt calls is registered by Init (proximity, trigger, teleport and activate-world callbacks) or started as a control thread; the other unreferenced routines of 1358 are double-precision library routines (High).
29. Resolved (SPEC-0059, section 7; High): the offered task is list entry Random(1, n) counted from 0, so entry 0 is never offered, and a draw of n reads the word just past the list, left over from earlier use of that memory. For n = 8 that word is never a task number; with n = 1 the one remaining task is never offered. When the word is not 1 to 8 the prompt index stays -1 (a built-in prompt, SPEC-0030 row 100) and a yes sets no flag and shows no task line. Clean behaviour: reproduce the skip of entry 0 and treat the read past the list as no offer. Still open (SPEC-0059 open question 1): the value of that word when n is below 8.
30. Resolved: 1122 hero-down check: in the module it runs only from the arena entry until movie 53 has played (stage bits 1 and 3; bit 3 is set by the movie routine and never cleared); after the shield break the module no longer swaps heroes or calls game over. Nothing else in the module covers hero 1 going down: the module registers no death callback and writes swap mode only in that check. The engine's ordinary death rules apply from movie 53 on (SPEC-0055, section 8A): a fallen selected Roberta is replaced by Tor (swap mode 2) and the game ends only when both heroes are down. Test vector: after movie 53, Roberta selected and killed, Tor alive: switch to Tor at 0.95 of her death clip, no SetGameOver, no swap-mode write (High for the module code; Medium-High for the mode, SPEC-0011).

## Confidence

| Section | Confidence |
| --- | --- |
| Container and family (all fifteen export Init, Update and Exit) | High (from the raw code) |
| Interior proximity, on-enter and item-ask routines (1008, 1011, 1020, 1014, 750, 749, 748) | High for the calls; Medium for the flag branches |
| Teleport and activate-world routines (1008, 1011, 1020) | High for the calls; Medium for the values |
| Slot words (all modules in this spec) | High for the store or absence of a store in reached code (the 956 set-1 word has no store, High; 1066 words 0x38b0, 0x38b4, 0x38b8 and 0x38bc are stored by Init in play; 750 word 0x3b38 is stored by a slot creation in Init that runs in play, High); Medium for the identity of each word not named here |
| SlotLoad activate (fifth) argument | High (true in every resolved call) |
| 956 Init, knight cages, lift, triggers and aftermath scene | High; Medium for the proximity line mapping |
| 1398 area stages, Update and triggers | High for the five stages (calls, flags and gates); Medium for the trigger-34 pass order |
| 1066 world masks | High for the values and for the Update swaps being dead; Medium for the meaning of each layer |
| 1335 proximity, triggers, teleports and Update | High for the Update, the trigger and teleport routines and the threads; Medium for the proximity routines and the walk sequence |
| 1122 Update (navmesh table) | High for the calls; Medium for the table values |
| 1122 wave sequence and Faerie Sisters arena flow | High for the calls, stage bits and order; Medium-High for the arena identity |
| 1173 and 1358 routines | High for the teleport routines, the 1358 task giver, prize wheel, slot swap, signs and activate-world routine, and the 1173 activate-world routine; Medium-High for 1173 routine 2 |
| Shared entity-control routine | High (one routine in all 27 modules; rules and records in its section); Medium for the refused 1411 adds |
| Shared tagged-arithmetic routines (compiler double-precision library) | High |
| Debug-mode query in Init calls | High for the query (SPEC-0030); High for the play-mode result in 748, 749, 750, 752, 1066, 1122 and 1173; Medium for thread bodies |
| Host calls made by each branch | Medium unless marked |

31. Resolved (R-0100, High): the stage-5 list is entities 9 to 12, the four type-109 variant-0 Whinnies in region 1401; the stage-4 list is entities 16 to 21, six type-70 variant-0 Evil Harrises in region 1403. The thread-6 region table is [1403, 1401, 1402, 1400, 1399] for arguments 0 to 4. The module checks both entity lists for trigger collisions each update. The Harris are Can Fly 0, so they do not take the flyer-only hit reaction and do not use its flyer strafe clips. Their undamageable setting suppresses hit-point loss but does not itself suppress hit reactions. Kind-7 waits for speed to fall to 2.0 or below; it does not create a launch speed or direction. Each Harris begins in idle-wander state 5 with a zero wander radius; that state can choose a random point from the actor's current navigation mesh and walk toward it. Their starts and the stage-4 locator ring share the horizontal footprint of one candidate mesh, but its triangles sit about 14.36 units above their sampled ground. This is a possible ordinary movement source, not proof of a usable route or trigger contact. The first Harris contact with trigger 34 remains open.
32. Open (R-0100): which current navigation mesh is assigned to a region-1403 Harris at run time, how the mesh at y≈24 reconciles with its ground near y≈9.64, and whether state-5 movement or another caller reaches trigger 34? The flyer-specific reaction is inapplicable; its speed threshold is only an exit condition. Static x-z overlap and trigger-collision checks do not prove a usable three-dimensional route.
