# SPEC-0039: Level scripts B

- Answers: proactive (level script behavior; 30 modules, follows SPEC-0030 and SPEC-0032), R-0054, R-0029 (points 1 to 4 answered: flag 496 has no writer on the disc; Open question 19 resolved), R-0064 (what activates the King in 1246), R-0096
- Status: Draft
- Disc build: SCUS-97367 v1.00 (same fingerprint as SPEC-0001)
- Confidence: per module (High, Medium or Low) and per section. Host call names, argument order and numeric arguments: High unless marked. Slot ids: High for the stored-slot rule (every SlotLoad and SlotReset in this spec that goes through a module word uses the slot number that the module's SlotCreate stored); two loads take a SlotCreate return value directly (1230 and 1257; Medium); the call path does not remap -1 (High); the one case in this spec where a slot load passes a word that its module stores only on the other flag path is module 1344 (see that section); no slot result in this spec is stored by a debug-gated helper, because the debug query gates no slot or record store here. Branch conditions written in words: Medium where the module's flags nest deeply. Each module section has a "Coverage" line. Module 732 uses switch dispatch (a computed value selects one of several branches); its cases are given in full (High).

## Summary

This spec describes 30 level script modules from the world packs. They are resource numbers 706, 732, 753, 757, 790, 793, 874, 880, 886, 984, 1131, 1133, 1136, 1139, 1140, 1143, 1148, 1172, 1230, 1244, 1246, 1248, 1257, 1344, 1405, 1407, 1417, 1419, 1425 and 1432. They follow SPEC-0032 and SPEC-0037. Resource 1421 is not covered here.

Family split: 12 Exterior (757, 790, 880, 886, 1139, 1230, 1244, 1246, 1248, 1257, 1432, 753), 13 Interior (984, 1131, 1133, 1136, 1140, 1143, 1148, 1172, 1405, 1407, 1417, 1419, 1425) and 5 Transition (706, 732, 793, 874, 1344). Module 793 exports no Exit. All others export Init, Update and Exit (Update may be empty).

The modules use the host API of SPEC-0030. Most of them follow one of four patterns: (a) a dialogue chooser that picks a line by a cascade of game flags and talks to an entity or hero; (b) a trigger that plays a camera, a fade and a hero or entity walk; (c) a slot chooser that loads character set j chosen by flags; (d) a world-swap or teleport that moves the hero and starts a transition. Modules 1405, 1407, 1417, 1419 and 1425 use the shared entity-control routine of SPEC-0043 (section "Shared entity-control routine"): each Update calls it every tick, and each module adds one record when its registration flag is set (see their sections; High).

Times are seconds unless marked as ticks. Sleep(n) yields n ticks; Wait(s) yields about s x 60 ticks (SPEC-0030, Open question 1). Flag numbers are game-state ids as the module passes them; quest start and done flags can be glossed by quest title (Open question 12).

## Where it lives on the disc

- Each module is the level-script chunk (type 37) of the world-pack resource named above, as in SPEC-0030 section "Where it lives on the disc". The clean side finds it by resource number.
- No values are read from the boot program's data for this spec. Small integer tables used by slot calls are stored in the module's own data; their values are given in the modules below as fixed lists (they are 1 to 6 small integers each).
- Repeated payloads: the modules in this spec have no duplicate payloads among themselves. Module 1430, 1431 and 1433 are copies of 1429 and are not in this spec.

## Load and unload plumbing

- As in SPEC-0032: each module's load and unload hooks run start and stop lists that are empty at load, so they have no observable effect (Medium, from module data).
- Exit is missing only in module 793 (Transition). The clean side treats a missing Exit as a no-op.
- Lifetime of module data words (SPEC-0030, Module lifetime; High): every load reads the pack afresh from the disc, so Init always sees the file values. The words then last until their world buffer is released. For an exterior (E buffer) they survive entering and leaving an interior, swaps between E and the transition buffer T, streaming steps whose target pack is already resident, and a movie played with no return world while the exterior is current. They return to their file values on any driver load of E (a replacement load of the area, a script reloading its own pack, the after-movie reload of E, loading a save), on streaming to a different exterior, and on a new game. Interior and transition modules are fresh on every entry. While an exterior is resident but not current, its Update, triggers and control threads do not advance.

## Host API used by this spec

Names as in SPEC-0030 section 2. Argument conventions:
- WAIT mode: 0 = return at once, 1 = yield until the action finishes. CONTROL, CAMERA_WAIT: passed through as the integers the module uses (0 and 1 seen; 6, 7 and 8 do not appear in this spec). The values and how the host reads each are in SPEC-0030 (Layout 2.1, Open question 5).
- Booleans are 0 or 1.
- Calls used in this spec: InitObjOn, ActivateObj, DeactivateObj, PlayAnim, LoopAnim, IsObjActive, IsObjAnimating, IsObjAnimFinished, StartEffect, StopEffect, PlayCamera, ReturnToNormalCamera, SetToNormalCamera, IsAnimatedCameraActiveAndFinished, StartCameraShakeMode, LetterBoxOn, LetterBoxOff, FadeScreenOut, FadeScreenIn, PlayFMV, PlayTrackHeroCamera, MoveHero, MoveEntity, WalkHero, WalkHeroAndOrient, WalkEntity, WalkEntityAndOrient, WalkEntityAlongPath, PlayHeroAnim, PlayEntityAnim, LoopEntityAnim, GrabEntityControl, ReleaseEntityControl, DeactivateEntity, ActivateEntity, SetAI_Agro, IsAI_Dead, SetDefaultEntityIdleAnim, DisableInvisibility, LevelMsg, LevelPrompt, SetConversationTarget, EndConversation, IsDialogActive, IsEntityTalking, IsHeroTalking, DisplaySquareButtonContext, IsButtonPressed, SetGameState, QueryGameState, SetLevelState, QueryLevelState, AddBlackboardMessage, GetBlackboardMessageValue, DoesHeroHaveItem, DoesHeroHaveTreasureMap, GiveItemToHero, RemoveItemFromHero, RemoveAllOfItemFromHero, RemoveNeopoints (amount argument only), GetNeopoints (not described), SetWorldCollisMask, SetWorldDisplayMask, SlotAllocate, SlotCreate, SlotLoad, SlotReset, CharacterActivate, CharacterDeactivate, ControlThreadsAllocate, ControlThreadStart, RegisterScriptTrigger, RegisterScriptInstanceTrigger, RegisterEntityProximityCallback, RegisterOnEnterCallback, RegisterItemAskAboutCallback, RegisterHeroBuyItemCallback, RegisterTeleportCallback, RegisterActivateWorldCallback, RegisterAIPlayerActivateCallback, WaitTillLeaveTrigger, GetCollisInfo, GetScriptGroupInstance, GetScriptGroupMainTrigger, StartEnterInterior, StartExteriorReplacementLoad, StartHeroLongFall, SetHeroSwapMode, WhichHeroIsActive, SwitchHero, BlockPlayerControl, RestorePlayerControl, ReturnHeroAnimToIdle, SetHeroEquippedState, ActivateHero, IsHeroDead, NeedHeroAlive, BossFightTweakHero, SetHeroStartPos, Break, SetBreakableActive, IsBreakableIntact, AlignAxisToVectorToHero, GetAxisDotVectorToHero, PlaySound, KillSound, PlayAnim, AddAnimObjSound, SetMaxActiveAI, Random, GetNumberOfFramesPerSecond, IsDebugMode, HasMoviePlayed, DebugPrintf, memset.

Conventions used below:
- "entered" means the callback's record has state 1 (SPEC-0030 section 3, state word). "object present" means the record's object pointer is non-null. Trigger record fields are as in SPEC-0030.
- "entity record" means the entity id the proximity record carries (a field of the record, not a fixed value).
- "line N" is a LevelMsg dialogue line index of the module's own text.
- "set j" is the character region index passed to SlotLoad (region = pack + 1 + j, SPEC-0030 section 5).
- "the stored slot" is the slot number that the module's SlotCreate call returned and the module kept in a module data word (SPEC-0032 Slot rules). Every SlotLoad and SlotReset in this spec reads that word, except the direct-return loads named in their sections (1230, 1257). SlotCreate returns the slot number, or -1 when no slot is free. A word that no SlotCreate has stored on the path being run still holds its file value -1, and the call path does not remap -1 (High). Module 1344 is the case where a load reads such a word (Open question 2).
- "button 0" is the square/context button of the engine's button enum.
- Colour arguments for fades are written as "colour (not described)". The fade durations and wait modes are given.
- Gated calls: a host call that runs only when a game flag, level state, item test or module data word holds is written with that gate next to it. No host call in this spec runs only under the debug query: the query is read once (in 1139), and no branch tests its result to gate a call. Routines that no registration or call in the module names are dormant and are not described as behavior.
- Reachability (SPEC-0030): every routine described in this spec runs in play. A chain of calls, registrations or thread starts leads to it from Init, Update, Exit or a load or unload hook, and no link of the chain sits under a debug-mode test. Whether a given call runs is decided only by the gates written next to it (High). The routines that nothing references are listed under "Dormant routines" in Edge cases; they never run.
- Module data words are live state: they hold their file values when the module is loaded and change as the module's own routines write them. Init sees the file values.
- Line rotation: many dialogue choosers in this spec pick a "mode" from a flag cascade and then cycle through that mode's lines on successive talks. Two module data words hold the last mode and a step counter (both 0 at load). When the mode differs from the last talk's mode, the counter restarts at 0. Counter 0 shows the first line and sets the counter to 1, counter 1 shows the second line and sets 2, counter 2 shows the third line and sets 0. Some choosers alternate between two lines in the same way. A mode written with one line shows that line on every talk. A chooser that finds no line shows nothing. Unless a section says otherwise, each line is shown as SetConversationTarget(entity), LevelMsg(line), EndConversation(entity) (High). The counters last as long as the module's data words (section "Load and unload plumbing"): in an exterior they carry over across interior visits and returns from the adjoining transition; in an interior or a transition they start at 0 on every entry (High).

## Modules

### 706: Transition, flag-gated setup, character sets and a teleport

- Family: Transition.
- Init:
  1. SlotAllocate(1). SlotCreate(6, table 0,1,2,3,4,5), its return value stored for the loads. ControlThreadsAllocate(1). ControlThreadStart(thread 2, argument world). These run in every case.
  2. RegisterScriptTrigger(index 1, trigger 1), RegisterScriptTrigger(index 2, trigger 3), RegisterEntityProximityCallback(entity 0, handler 5, flag 1) and RegisterTeleportCallback(index 0, context 13, handler 6). These run in every case.
  3. SetGameState(62, 1) runs only when none of flags 67, 66, 65, 64, 63 and 62 is set. A helper routine tests them in that order and returns the first set flag's number, or -1 when none is set; Init sets flag 62 on -1 (High). Items 1 and 2 do not depend on that test.
  4. If flag 248 is set and the hero does not have item 14: InitObjOn(object 0, flag 0) and RegisterScriptTrigger(index 0, trigger 4). These run in play whenever that gate holds at Init (High).
  5. If flag 205 is set: SetWorldCollisMask(2) and SetWorldDisplayMask(2); otherwise SetWorldCollisMask(1) and SetWorldDisplayMask(1).
- Trigger 1 (handler 1): on entry, sets flag 207 to 1 if it is clear.
- Trigger 0 (handler 4, registered only under Init item 4): on entry (object present, state 1): LevelMsg(2), then LevelMsg(3).
- Trigger 2 (handler 3): when object present: if button 0 is not pressed, DisplaySquareButtonContext(context 3); if it is pressed, LevelMsg(0). The routine picks line 0 only for trigger index 2 (the only index it is registered under) and no line for any other index.
- Entity 0 proximity (handler 5): a three-line rotation (see "Line rotation" under Conventions). Flag 63 set: lines 7, 8, 9 on successive talks. Flag 63 clear: lines 4, 5, 6. The rotation restarts at its first line when the flag-63 state differs from the previous talk. Each talk is SetConversationTarget(entity record), the line, EndConversation(entity record) (High).
- Teleport 0 (handler 6): DisplaySquareButtonContext(context 13). If button 0 is pressed: when flag 67 is clear and flag 64 is set, LevelMsg(1); otherwise (flag 67 set, or flags 67 and 64 both clear) StartEnterInterior(eWorlds 76 (pack 984), -1, -1, no colour) and Wait(3.0 s) (High).
- Thread 2: chooses the character set from the first set flag in the order 67, 66, 65, 64, 63. Flag 67 gives set 5, 66 gives set 4, 65 gives set 3, 64 gives set 2, 63 gives set 1, and none gives set 0. Each branch is one SlotLoad(the stored slot, set j, true, true).
- Update and exit: none.
- Coverage: fully described.
- Confidence: High.
- Progression flags 62 to 67 (High): these form the Glade-stage selector shared with pack 956. Flag 62 is the fallback stage, set by pack 706 on entry only when none of flags 62 to 67 is already set. Pack 1098 sets 63 at trigger index 0; the Gnarfas fight in pack 985 sets 64 after the fight; pack 956 sets 65 when Act 3 flag 248 is set, 66 at trigger 10 and 67 at trigger 12. In Act 1, stage 62 or 63 is the pre-fight state; 64 marks the post-Gnarfas state. The exact authored names or broader narrative intent of flags 62 to 67 are not established. Flags 205 to 207 remain unmapped.

### 732: Transition, two entity lines, a blackboard value per teleport index

- Family: Transition.
- Init: DebugPrintf; GetScriptGroupInstance(group 2). A loop over the group's script trigger list makes one RegisterScriptTrigger(handler 1, index) per listed index; it registers nothing when the list is empty. On disc, script group 2 of pack 732 is a kind-0 entry whose trigger list is 0, 1, 2, so handler 1 is registered under trigger indices 0, 1 and 2 (High). RegisterActivateWorldCallback(handler 2), RegisterEntityProximityCallback(entity 1, handler 3, flag 1), ControlThreadsAllocate(1) and ControlThreadStart(thread 4, argument 0) follow the loop and always run (High).
- Entity 1 proximity (handler 3): flag 562 set: line rotation 0, 1, 2. Flag 562 clear: line 4 on every talk. Each talk is SetConversationTarget(record), the line, EndConversation(record) (High).
- Thread 4: SlotAllocate(1); SlotCreate(2, table 0,1). The return value is written to a module data word, and the two loads below read that stored slot. If flag 561 is set and flag 562 is clear: SlotLoad(the stored slot, set 0). Then it polls AI entity 0 until it is dead (IsAI_Dead(0)) with Sleep(1) between polls; when dead it sets flag 562 to 1. If flags 561 and 562 are both set: SlotLoad(the stored slot, set 1). The activate argument of both loads is true.
- Activate-world (handler 2), run in play on every activation: reads v = the blackboard value of type 2 for eWorlds 21 (pack 732). A switch on v selects a locator and a camera: v = 10 gives locator 1 and camera 8; 20 gives locator 2 and camera 7; 30 gives locator 3 and camera 6; 40 gives locator 4 and camera 5; 60 gives locator 5 and camera 4. Any other value (including -1) selects nothing. For a selected case: MoveHero(locator, 0, 1, 1). Then, in every case: AddBlackboardMessage(type 2, value -1, eWorlds 21) and FadeScreenIn(0.5 s, wait 0). For a selected case only: StartHeroLongFall(), PlayTrackHeroCamera(camera, wait 0, camera wait 1, 1.0) and Wait(1.5 s). Last, in every case: SetToNormalCamera(5.0, 0.0, flag 1) (High for the cases and the order; Medium for the meaning of the move arguments).
- Trigger handler 1 (indices 0, 1 and 2), on every fire with the object present (no state test; nothing when the object is null), in order:
  1. AddBlackboardMessage(type 2, v, eWorlds 22) with v = 70 for index 0, 60 for index 1 and 40 for index 2 (any other index would give -1).
  2. IsButtonPressed(0, 1, 0) is read. The prompt DisplaySquareButtonContext(context 13) is shown only when the button is not pressed and the index is not 0, 1 or 2, which never holds for the registered indices.
  3. So every contact runs the exit at once, with no button needed: FadeScreenOut(0.5 s, black, WAIT 1); then MoveHero(locator 0, 1, 1, 1).
- Update and exit: none.
- Coverage: described.
- Confidence: High.

### 753: Exterior, entity proximity, teleport and an effect group

- Family: Exterior.
- Init: SlotAllocate(2). SlotCreate(2, table 0,2) (the first stored slot) and SlotCreate(1, table 1) (the second stored slot). A test helper returns 1 when flag 256 is set and flag 258 is clear, else 0; call this "the event gate". ControlThreadsAllocate(1 + event gate). ControlThreadStart(slot thread, argument world) in every case. When the event gate is 1: ControlThreadStart(event thread, argument world) and RegisterScriptTrigger(handler 3, index 1) (High).
- Then in every case: RegisterEntityProximityCallback(entity 5, handler 4, flag 1); RegisterEntityProximityCallback(entity 6, handler 5, flag 1); RegisterTeleportCallback(index 0, context 13, handler 6); RegisterActivateWorldCallback(handler 7). The effect helper with argument 0: StopEffect for effects 15 to 48 (34 effects).
- If flag 252 is clear: RegisterScriptTrigger(handler 8, index 0) and SetWorldCollisMask(1).
- Effect helper: argument nonzero gives StartEffect(e, 0) for e = 15 to 48; zero gives StopEffect(e) for the same effects.
- Entity 5 proximity (handler 4), line rotation: flag 86 set: lines 0, 1, 2. Else flag 76 set: lines 3, 4, 5. Else flag 258 set: hero 0 active gives lines 6, 7, 8 and the other hero gives lines 9, 10, 11 (the two share one mode, so the counter carries over between them). Else flag 256 set: line 12 on every talk. Else: hero 0 active gives lines 13, 14, 15; the other hero gives line 16 (High).
- Entity 6 proximity (handler 5): the same chooser with lines 17 to 19 (flag 86), 20 to 22 (flag 76), 23 to 25 and 26 to 28 (flag 258, hero 0 or the other hero), 29 (flag 256), 30 to 32 (hero 0) and 33 (the other hero) (High).
- Slot thread (runs once): if flag 256 is set and flag 258 is clear: SlotLoad(the second stored slot, set 1, false, true). Then, if flag 427 is set and flag 428 is clear: SlotLoad(the first stored slot, set 2, true, true); otherwise SlotLoad(the first stored slot, set 0, true, true) (High).
- Trigger 1 (handler 3, registered only when the event gate was 1), when the object is present: if movie 36 has not played: PlayFMV(36, true), MoveHero(locator 3, 1, 1, 1), SetToNormalCamera(5.0, 3.14, flag 1), CharacterActivate(the character in the second stored slot, 0), the effect helper with argument 1 (effects 15 to 48 start), and SetLevelState(1, true). Then, in every case, SetLevelState(2, true) (High).
- Progression source (High for the flag writer and route gates; Medium for normal-route ordering): pack 777 registers library triggers 4 to 8 on every load. Trigger 8 is the middle-pedestal book trigger; its shared callback requires first-fire state, a present trigger object, flag 256 clear, and a newly pressed Square. Its record sets flag 256 and shows line 240; it names no animated object or camera track, and the visible book's resource identity is not linked by this script. The callback commits flag 256 after its wait, then its top-level guard silences all five library triggers. The record and module data establish no ordering requirement among flags 253 to 256. Once 256 is set, trigger 8 cannot run again. Static evidence does not establish whether re-entry can start a second callback during the pre-commit wait. The library exit remains shut until all four flags 253 to 256 are set, so a normal return from the library to the castle path reaches 753 with 256 on, assuming the player completes all four book triggers; this route ordering is an inference from the exit gate. Main-story step 30 places movies 36 and 37 in 753 after the library. This module's event-end path sets flag 258 after movie 37 and character deactivation; within this recovered event path, 258 is the completion latch that prevents the event from starting again. No other prior writer for 258 was established in the route-pack evidence checked here; its broader global ownership remains unclaimed.
- Event thread (started only when the event gate was 1), every tick: when flag 258 is clear and level state 1 is set: if level state 0 is set, the event end runs. Otherwise, if level state 2 is set, it clears level state 2; if level state 2 is clear, it sets level state 0 and the event end runs. Then Sleep(1). Because trigger 1 sets level state 2 again on each tick the hero stays inside, the event end runs on the first tick after the hero has left trigger 1 (Medium for that reading; High for the rules). Event end: PlayFMV(37, true), CharacterDeactivate(the character in the second stored slot, 0), SetGameState(258, 1), and the effect helper with argument 0 (effects 15 to 48 stop). Flag 258 then keeps the thread idle.
- Teleport 0 (handler 6), for index 0: AddBlackboardMessage(type 2, value 80, eWorlds 31 (pack 758)). If flag 68 is clear: SetReturnWorldForFMV(eWorlds 31) and PlayFMV(33, true), so the world change happens when the movie ends. If flag 68 is set: StartExteriorReplacementLoad(eWorlds 31, -1, no colour) and Wait(3.0 s). For any other index, only the wait (High).
- Activate-world (handler 7): reads the blackboard value of type 2 for eWorlds 31. Value 80: MoveHero(locator 0, 1, 1, 1), Sleep(30); if flag 660 is clear, LevelMsg(34), LevelMsg(35) and SetGameState(660, 1). Any value other than -1: SetToNormalCamera(5.0, 0.0, flag 1). Then AddBlackboardMessage(type 2, value -1, eWorlds 31) and the event-gate test helper (its result is not used) (High). Reachability boundary: Init registers this handler and the index-0 teleport handler unconditionally; that teleport handler posts value 80 for eWorlds 31 (pack 758), and pack 753 is part of the documented Brightvale Castle route (SPEC-0058 step 29). Therefore the value-80 flag-660 setter has a source-supported route in play. Static evidence here does not establish exact callback timing or flag 660 wider story meaning.
- Trigger 0 (handler 8, registered only when flag 252 was clear), entered with object present: if flag 252 is clear, LevelMsg(36).
- Update and exit: none.
- Coverage: fully described.
- Confidence: High, except where marked.

### 757: Exterior, boss-arena style encounter with a camera and AI state

- Family: Exterior.
- Init: InitObjOn(objects 2, 3, 4, 5; flag 1). RegisterScriptTrigger(handler 1, index 0); RegisterScriptTrigger(handler 2, index 1); RegisterScriptTrigger(handler 3, indices 4 to 10, one registration each).
- Trigger 1 (handler 2), when flag 664 is clear, object present and state 1:
  1. BlockPlayerControl(hero -1, the active hero); LetterBoxOn(1.0 s, wait 0); WalkHero(locator 4, wait 0, control 0).
  2. ActivateObj(0, false); ActivateObj(1, false); Sleep(1).
  3. PlayAnim(object 4, wait 0, speed -1); PlayAnim(object 5, wait 0, speed -1); PlayCamera(5, wait 1, camera wait 1, 1.0 s); PlayAnim(2, wait 0, speed -1); PlayAnim(3, wait 0, speed -1).
  4. SetAI_Agro(14, 15, 16); PlayCamera(6, wait 1, camera wait 1, 1.0 s); LetterBoxOff(1.0 s, wait 0); ReturnToNormalCamera(-1, flag 1).
  5. If AI entities 14, 15 and 16 are all dead (IsAI_Dead): Sleep(20); DeactivateObj(0) and DeactivateObj(1); PlayAnim(4, wait 0, 1.0); PlayAnim(5, wait 0, 1.0); PlayCamera(7, wait 0, camera wait 1, 1.0 s); PlayAnim(2, wait 0, 1.0); PlayAnim(3, wait 1, 1.0); SetGameState(664, 1).
- Trigger 0 (handler 1), when object present, state 1 and flag 665 clear: PlayCamera(4, wait 0, camera wait 1, 0.85); Sleep(1); poll until IsAnimatedCameraActiveAndFinished(4); FadeScreenOut(1.0 s, wait 1); SetGameState(665, 1); ReturnToNormalCamera(-1, 1); FadeScreenIn(1.0 s, wait 1).
- Triggers 4 to 10 (handler 3): when object present and state 1, the trigger index selects SetAI_Agro for AI entity (index - 2): index 4 gives entity 2, index 5 gives 3, index 6 gives 4, index 7 gives 5, index 8 gives 6, index 9 gives 7, index 10 gives 8. The code also has a branch for index 11 (entity 9) that is not registered.
- Update and exit: none (no calls; Exit exported, empty).
- Coverage: fully described.
- Confidence: High for triggers 4 to 10 and trigger 0; Medium for the trigger 1 order (two branches are flat in the recorded calls).
- Open: the meaning of flag 664 and 665; the AI entity ids as enemy identifiers.

### 790: Exterior, four triggers, a hero-button prompt and a camera-and-slot thread

- Family: Exterior.
- Init: InitObjOn(objects 3, 0, 1, 2, 18, 20; flag 0). RegisterScriptTrigger(handler 1, index 1); (handler 2, index 14); (handler 3, index 15); (handler 4, index 0); (handler 5, index 8). RegisterScriptInstanceTrigger(handler 5, group 0, index 6). SlotAllocate(1); SlotCreate(3, table 0,1,0), its return value stored. ControlThreadsAllocate(1).
- Trigger 1 (handler 1): runs only when the shot word (+0x20) and the attack word (+0x24) are both 0, a module latch (file value 0) is 0 and the state is 1 (first fire). Then PlayAnim(3, wait 1, 1.0 s). After it, the state word is read again:
  - State 0 (idle) or 3 (the hero has left): PlayAnim(4, wait 1, 1.0 s) and the handler ends; the latch stays 0, so a later entry runs it again.
  - State 1 or 2 (the hero is still inside): PlayAnim(0, wait 0, 1.0 s); PlayCamera(5, wait 1, camera wait 0, 1.0 s); ReturnToNormalCamera(-1, 1); the latch becomes 1; WaitTillLeaveTrigger(record); PlayAnim(4, wait 1, 1.0 s). With the latch at 1 the handler never runs again while the pack stays resident.
- Trigger 14 (handler 2), any state other than 0, object present: DisplaySquareButtonContext(context 6). If button 0 is pressed: PlayCamera(9, wait 0, camera wait 0, 1.0 s); WalkHeroAndOrient(locator 48, wait 1, control 1); PlayHeroAnim(140, wait 0, control 1); PlayAnim(18, wait 1, 1.0 s); PlayCamera(6, wait 0, camera wait 1, 1.0 s); PlayAnim(1, wait 1, 1.0 s); DeactivateObj(18); ActivateObj(19, false); ReturnToNormalCamera(-1, 1).
- Trigger 15 (handler 3): the same shape with prompt 6, WalkHeroAndOrient(locator 49), PlayAnim(20, wait 1), PlayCamera(7, wait 0, camera wait 1, 1.0 s), PlayAnim(2, wait 1, 1.0 s), DeactivateObj(20), ActivateObj(21, false), ReturnToNormalCamera(-1, 1).
- Trigger 0 (handler 4), entered: FadeScreenOut(0.5 s, wait 1); Sleep(3); LetterBoxOn(0 s, wait 1); FadeScreenIn(0.5 s, wait 1); WalkHero(locator 6, wait 0, control 1); PlayCamera(4, wait 1, camera wait 0, 1.0 s); LetterBoxOff(1.0 s, wait 0); ReturnToNormalCamera(-1, 1).
- Handler 5 is registered for trigger 8 (RegisterScriptTrigger) and through RegisterScriptInstanceTrigger(handler 5, t = 0, script group 6). Script group 6 of pack 790 is a kind-7 entry with trigger list 16, 17, so triggers 16 and 17 get handler 5 with owner value 6, and t is ignored (trigger 0 keeps handler 4).
- Handler 5: the record's owner value (+0x30) is -1 for trigger 8, which then uses its own record; for 16 and 17 it is 6, and the handler uses GetScriptGroupMainTrigger(6, 0), the record of trigger 16. On that record it needs state 1 and the object present. Index 8 gives a = 1 and index 16 gives a = 0 (any other index: debug text only). A module word holds the last a (file value -1): when a equals it, only debug text runs; otherwise the word takes a and ControlThreadStart(thread 6, a) runs.
- Thread 6: a = 1 (trigger 8): SlotReset(the stored slot), then SlotLoad(the stored slot, set 0, true, true). a = 0 (triggers 16 and 17): SlotReset(the stored slot), then SlotLoad(the stored slot, set 1, true, true). So the slot set changes only when the hero crosses from one side to the other; re-entering the same side reloads nothing.
- Update and exit: none.
- Coverage: described.
- Confidence: High.

### 793: Transition, one entity line and a thread

- Family: Transition. Exit: none exported.
- Init: RegisterEntityProximityCallback(entity 0, handler 1, flag 1); (entity 1, handler 2, flag 1); (entity 2, handler 2, flag 1). SlotAllocate(1). SlotCreate(2, table 0,1), its return values stored. ControlThreadsAllocate(1). ControlThreadStart(thread 3, argument world).
- Entity 0 proximity (handler 1): if flag 417 is set: SetConversationTarget(record), LevelMsg(16), EndConversation(record). Otherwise nothing.
- Entity 1 and entity 2 proximity (handler 2): the flags 417, 416, 414, 412 and 410 are tested in that order and choose one line: LevelMsg(0) when 417 is set; when 417 is clear: LevelMsg(3) if 416 is set, LevelMsg(6) if 414 is set, LevelMsg(9) if 412 is set, LevelMsg(12) if 410 is set, otherwise LevelMsg(13). SetGameState(412, 1) runs only when flags 417, 416, 414 and 412 are clear and flag 410 is set; in that case RemoveItemFromHero(item 13, count 1) runs first, then the line. Every path then runs SetConversationTarget(record), the chosen LevelMsg and EndConversation(record). Flag 417 only chooses the line; it does not gate the handler.
- Thread 3: if flag 417 is set: SlotLoad(the stored slot, set 0); otherwise SlotLoad(the stored slot, set 1). The activate argument is true.
- Update and exit: no update. No Exit.
- Coverage: fully described.
- Confidence: Medium (the cascade of line choices is flat; the order above follows the recorded calls).

### 874: Transition, two entity talks, a camera and a slot thread

- Family: Transition.
- Init: RegisterEntityProximityCallback(entity 6, handler 1, flag 1); (entity 7, handler 2, flag 1). SlotAllocate(3). SlotCreate(2, table 1,0); SlotCreate(1, table 2); SlotCreate(1, table 3), with the return values stored. ControlThreadsAllocate(1). ControlThreadStart(thread 3, argument world).
- Entity 6 proximity (handler 1): SetConversationTarget(record); LevelMsg(1); EndConversation(record); PlayCamera(5, wait 0, camera wait 1, 1.0 s); PlayEntityAnim(entity 6, animation 30, wait 1, control 1); WalkEntity(entity 6 to locator 3, wait 0, speed 1.1, control 1); FadeScreenOut(black, 1.0 s, wait 1); SetGameState(291, 1); SlotReset(the stored slot); ReturnToNormalCamera(-1, 1); FadeScreenIn(black, 0.7 s, wait 0).
- Entity 7 proximity (handler 2): SetConversationTarget(record); LevelMsg(0); PlayCamera(4, wait 0, camera wait 1, 1.0 s); MoveHero(locator 2, flag 1, control 0, extra 1); PlayEntityAnim(entity 7, animation 30, wait 1, control 1); EndConversation(record); WalkEntity(entity 7 to locator 1, wait 0, speed 1.1, control 1); FadeScreenOut(black, 1.0 s, wait 1); SetGameState(290, 1); SlotReset(the stored slot); ReturnToNormalCamera(-1, 1); FadeScreenIn(black, 0.7 s, wait 0).
- Thread 3: if flag 58 is set: SlotLoad(the stored slot, set 1, true, true), and a debug line. Then flags 291 and 290 select further loads (set 2 when 291 is clear; set 3 when 291 and 290 are clear), and when flag 58 is clear: SlotLoad(the stored slot, set 0). The nesting is flat in the recorded calls (Medium).
- Update and exit: none.
- Coverage: fully described for handlers 1 and 2; the thread is described through its condition list.
- Confidence: High for handlers; Medium for thread 3.

### 880: Exterior, boss-entry triggers, a treasure-map exchange and a slot thread

- Family: Exterior.
- Init:
  1. InitObjOn(objects 0 and 1; flag 0). SlotAllocate(2); SlotCreate(5, table 0,1,2,3,4); SlotCreate(1, table 4), with the return values stored.
  2. RegisterScriptTrigger(handler 1, index 5), (handler 1, index 6), (handler 1, index 7); (handler 2, index 0); (handler 3, index 1); (handler 9, index 2).
  3. ControlThreadsAllocate(1); ControlThreadStart(thread 4, argument world).
  4. RegisterScriptInstanceTrigger(handler 5, group 0, member 2); RegisterTeleportCallback(index 0, context 13, handler 6).
  5. RegisterEntityProximityCallback(entity 3, handler 7, flag 1) and (entity 7, handler 7, flag 1). RegisterActivateWorldCallback(handler 8).
  6. SetWorldCollisMask(1) when flag 252 is clear.
- Entity 3 and entity 7 proximity (handler 7): the treasure-map shop (the same scheme as module 1230). The handler offers the first map in this list that the hero does not own (DoesHeroHaveTreasureMap), with its prompt line and price: map 1 (prompt 6, 250 Neopoints), map 8 (prompt 7, 250), map 30 (prompt 8, 250), map 27 (prompt 9, 300), map 18 (prompt 10, 400), map 3 (prompt 17, 300), map 6 (prompt 18, 300), map 11 (prompt 19, 400). For an offer: SetConversationTarget(record) and LevelPrompt(prompt, -1). Declined: line 16. Accepted with GetNeopoints() below the price: line 15. Accepted with enough: GiveItemToHero(item 175, map id, false), RemoveNeopoints(price), line 14. When the hero owns all eight maps there is no prompt and the line is a rotation of 11, 12, 13. The line is then shown with SetConversationTarget(record), LevelMsg(line), EndConversation(record). The second argument of GiveItemToHero is the map id here, not a count (High).
- Trigger 0 (handler 2), entered with object present and a "talked" module word clear: sets the word, then runs the dialogue helper with entity 3 as the conversation target. Trigger 1 (handler 3), entered with object present: clears the word. So entity 3 speaks once each time the hero passes trigger 1 and then trigger 0 (High).
- Dialogue helper (flag 67), line rotation: flag 67 set gives lines 0, 1, 2; clear gives lines 3, 4, 5 (one counter for both; it restarts the first time flag 67 is seen set). Each is SetConversationTarget(entity 3), the line, EndConversation(entity 3).
- Triggers 5, 6 and 7 (handler 1): while the trigger state is non-zero, each tick: DisplaySquareButtonContext(context 3); if button 0 is pressed, LevelMsg(21) for index 5, LevelMsg(22) for index 6 or LevelMsg(24) for index 7; Sleep(1) (High).
- Trigger 2 (handler 9), entered with object present: if flag 252 is clear, LevelMsg(25).
- Instance trigger (group 0, member 2; handler 5): uses the group's main trigger when the record's group field is not -1. If flag 296 is clear: SlotReset(the first stored slot) and SlotLoad(the second stored slot, set 4, true, true). Then, in every case: WaitTillLeaveTrigger(the trigger), SlotReset(the second stored slot) and ControlThreadStart(thread 4, argument world), which reloads the flag-chosen set (High).
- Teleport 0 (handler 6), for index 0: if flag 296 is clear, LevelMsg(20) and Sleep(30). If flag 296 is set: AddBlackboardMessage(type 2, value 80, eWorlds 81) and StartExteriorReplacementLoad(eWorlds 81 (pack 991), -1, no colour). Then Wait(3.0 s) (High). So the way on to pack 991 opens once AI 21 to 23 are dead (Update below).
- Activate-world (handler 8): reads the blackboard value of type 2 for eWorlds 81. Value 80: MoveHero(locator 2, 1, 1, 1). Any value other than -1: SetToNormalCamera(5.0, 0.0, flag 1). Then AddBlackboardMessage(type 2, value -1, eWorlds 81).
- Thread 4: loads a character set by the first set flag in the order 52, 51, 50, 49: flag 52 gives set 3, flag 51 gives set 2, flag 50 gives set 1, flag 49 gives set 0. Each branch is one SlotLoad(the first stored slot, set j, true, true) and also prints a debug line. When none of the four flags is set, SlotLoad(the first stored slot, set 0, true, true) runs.
- Update: the AI check is in a routine that the update calls only while flag 296 is clear. When AI entities 21, 22 and 23 are all dead (IsAI_Dead in order, each test only after the previous one held), SetGameState(296, 1). A debug line follows.
- Exit: none.
- Coverage: fully described.
- Confidence: High, except where marked.

### 886: Exterior, a dialogue-prompt trigger, a camera-and-slot thread and an AI sequence

- Family: Exterior.
- Init: RegisterScriptTrigger(handler 1, index 5); (handler 2, index 2). If flag 250 is clear: InitObjOn(3, false). In both cases: SlotAllocate(3); SlotCreate(6, table 0,1,2,3,5,4); SlotCreate(1, table 5); SlotCreate(1, table 4), with the return values stored (flag 250 does not gate the slot calls; there is one SlotAllocate call in total). If movie 30 has not been played (HasMoviePlayed(30) false) and flag 248 is set with flag 249 clear: SetGameState(249, 1). This SetGameState(249, 1) is reached in Init when flag 248 is set, flag 249 is clear and movie 30 has not played (HasMoviePlayed(30) false); this movie/flag condition does not gate slot creation or the thread start. ControlThreadsAllocate(2); ControlThreadStart(thread 3, argument world); RegisterScriptTrigger(handler 4, index 0); flag 252 cond: if clear, SetWorldCollisMask(1).
- Trigger 5 (handler 1), entered and flag 250 clear: LevelMsg(6).
- Trigger 2 (handler 2), object present, state not 0: DisplaySquareButtonContext(context 3); if button 0 is pressed: LevelMsg(7).
- Trigger 0 (handler 4), entered: if flag 252 is clear: LevelMsg(8).
- Thread 3 (one-shot, started by Init): checks flags in priority order: 52 loads set 3, 51 loads set 2, 50 loads set 1, and 251 loads set 0, all into the first stored slot with keep and activate true. It does not check flag 49. If all four are clear, flag 250 set loads set 4 into the third stored slot; flag 250 clear loads set 5 into the second stored slot, then unhides entities 16–18 and sets agro for entities 16 and 17. The stored slots are separate buffers sized for the six-set, set-5-only and set-4-only tables. No activate-world callback is registered in this pack.
- Update: on every tick the update calls two helpers, both in play. The end helper: when flag 250 is clear and AI entities 16, 17 and 18 are dead (IsAI_Dead in order, each test only after the previous one held): DeactivateObj(3), set flag 250, Wait(6.0 s), FadeScreenOut(1.0 s, wait 1), PlayFMV(32, true), Sleep(1) and FadeScreenIn(1.0 s, wait 0) (High). This helper does not load set 4 or restart the slot thread; a second thread run would require module re-entry, which is not established. The hint helper, while flag 250 is clear: a module tick counter (file value 0) counts up by 1 per tick until it reaches its limit word (file value 300, so 5 s at 60 ticks per second); nothing else happens before that. Then, once: LevelMsg(1) unless hero 1 is active, and Sleep(1). Then, once, on the first tick on which hero 1 is active, AI 16 and 17 are dead and AI 18 is alive: Wait(2.0 s) and LevelMsg(3). Module words latch both hints (High).
- Exit: none.
- Test vectors: flags 52 and 51 both set select set 3 (highest priority); flags 52, 51, 50, 251 and 250 all clear select set 5 and unhide entities 16–18, with agro on 16 and 17; if only flag 49 is set, the same set-5 branch runs because this thread does not query flag 49; with the first four flags clear and flag 250 set, set 4 loads into the third stored slot. Setting flag 250 during the completion helper does not itself load set 4 during that same module lifetime.
- Coverage: partly described.
- Open: whether movie 32's return causes module re-entry and a second Init-started thread run is not established. The observed completion helper itself does not reload a character set or restart the thread.
- Confidence: High for init and update.

### 984: Interior, an entity proximity dialogue, on-enter dialogue and four item-ask lines

- Family: Interior.
- Init: RegisterEntityProximityCallback(entity 0, handler 1, flag 1). RegisterOnEnterCallback(handler 2, index 0). RegisterItemAskAboutCallback(item 153, handler 3, flag 1); (item 155, handler 4); (item 152, handler 5); (item 151, handler 6). ControlThreadsAllocate(2); ControlThreadStart(thread 7, argument world).
- Entity 0 proximity (handler 1): when flag 67 is set: SetConversationTarget(record), LevelMsg(12), EndConversation(record). Otherwise, if flag 202 is set: LevelMsg(15); else if flag 150 is set: LevelMsg(18); else LevelMsg(21).
- On-enter (handler 2): when flag 67 is set: SetConversationTarget(record), LevelMsg(0), EndConversation(record). Otherwise, if flag 202 is set: LevelMsg(2); else if flag 150 is set: LevelMsg(4); else LevelMsg(6).
- Item-ask handlers 3, 4, 5 and 6: each shows one line: item 153 gives LevelMsg(8); item 155 gives LevelMsg(9); item 152 gives LevelMsg(10); item 151 gives LevelMsg(11).
- Thread 7: SetToNormalCamera(5.0 s, 0.0, flag 1).
- Update and exit: none (Exit exported, empty).
- Coverage: fully described.
- Confidence: High.

### 1131: Interior, dialogue with an item check, a slot thread and four ask lines

- Family: Interior.
- Init: SlotAllocate(1); SlotCreate(2, table 0,1), with the return values stored. ControlThreadsAllocate(2). ControlThreadStart(thread 1, argument world); ControlThreadStart(thread 2, argument 0). RegisterEntityProximityCallback(entity 0, handler 3, flag 1). RegisterItemAskAboutCallback(item 106, handler 4, flag 1); (item 121, handler 5, flag 1). RegisterOnEnterCallback(handler 2, index 0).
- Entity 0 proximity (handler 3), line rotation: flag 301 set: line 36 when the hero has item 124, otherwise lines 20, 21, 22. Else flag 216: lines 23, 24, 25. Else flag 202: lines 26, 27, 28. Else flag 150: lines 29, 30, 31. Else lines 32, 33, 34 (High).
- Item-ask 106 (handler 4): one line by the first set flag: 301 gives line 10, 216 gives 11, 202 gives 12, 150 gives 13, 7 gives 14; none set: no line. Item-ask 121 (handler 5): the same with lines 15 to 19. The conversation target is entity 0 (High).
- On-enter (handler 2; the same routine is also started as thread 2 at Init, so it also runs once on arrival): two-line alternation. Flag 301 set: line 35 when the hero has item 124, otherwise lines 0 and 1. Else flag 216: lines 2 and 3; else flag 202: lines 4 and 5; else flag 150: lines 6 and 7; else lines 8 and 9. The conversation target is entity 0 (High).
- Thread 1: if flag 203 is set, SlotLoad(the stored slot, set 0, true, true). Then, in every case, SetToNormalCamera(5.0 s, 0.0, flag 1) (High).
- Update and exit: none.
- Coverage: described.
- Confidence: High.

### 1133: Interior, a multi-line dialogue with item checks and a slot thread

- Family: Interior.
- Init, in every case: SlotAllocate(1); SlotCreate(2, table 0,1), with the return values stored. ControlThreadsAllocate(2). ControlThreadStart(thread 1, argument world). ControlThreadStart(thread 2) with argument 1 or 0: the flag-10 test chooses the stored argument value and does not guard the call (High for the call, Medium for the argument's meaning). RegisterEntityProximityCallback(handler 3, flag 1) with entity 1 or entity 0, chosen by flag 10 in the same way (Medium). RegisterItemAskAboutCallback(item 99, handler 4, flag 1).
- Entity proximity (handler 3), line rotation by the first set flag: 301: lines 19, 20, 21. 216: lines 22, 23, 24. 202: lines 25, 26, 27. 150: lines 28, 29, 30. 141: lines 31, 32, 33. 138: the item trade below. None of these: lines 39, 40, 41.
- The item trade (flag 138 set, flags 301, 216, 202, 150 and 141 clear): the handler counts how many of items 40, 41, 42 and 43 the hero holds (DoesHeroHaveItem for each). With all four: RemoveAllOfItemFromHero for items 41, 42, 43 and 40, GiveItemToHero(item 60, count 1, false), SetGameState(141, 1) and line 34. With three: line 35; two: line 36; one: line 37; none: line 38. Flag 141 then moves the chooser to lines 31 to 33 (High).
- Item-ask 99 (handler 4): one line by the first set flag: 301 gives line 14, 216 gives 15, 202 gives 16, 150 gives 17, 7 gives 18; none set: no line. The conversation target is the entity chosen by flag 10 (High).
- Thread 2 (argument: that entity) runs once per entry and shows one line: the first set flag picks the pair, and the first call of each pair shows its first line. Flag 301: line 0; 216: line 2; 202: line 4; 150: line 6; 141: line 8; 139: line 10; none: line 12 (High). The second line of each pair (1, 3, 5, 7, 9, 11 and 13) needs a second call in the same module lifetime; interiors are fresh on every entry (SPEC-0030), so these lines are not reached in play (High for the code; Medium-High for the reset rule). One toggle word and one group word serve all pairs, and the toggle resets when the group changes. In the flag 139 branch the second call does not clear the toggle, so a second call there would repeat line 11 (High).
- Thread 1: SlotLoad(the stored slot, set 1 when flag 10 is set, set 0 when it is clear, true, true). Then, in every case, SetToNormalCamera(5.0 s, 0.0, flag 1) (High).
- Update and exit: none.
- Coverage: fully described.
- Confidence: High.

### 1136: Interior, dialogue with a flag-491 to flag-492 cascade and a slot thread

- Family: Interior.
- Init, in every case: SlotAllocate(1); SlotCreate(2, table 0,1), with the return values stored. ControlThreadsAllocate(2). ControlThreadStart(thread 1, argument world). ControlThreadStart(thread 2) with argument 1 or 0, the value chosen by flag 10 (Medium). RegisterEntityProximityCallback(handler 3, flag 1) with entity 1 or entity 0, chosen by flag 10 (Medium). The flag-10 test does not decide whether these calls run (High).
- Thread 2: the dialogue chooser: when flag 301 is set, line 0 or 2; otherwise the flag cascade 216, 202 and 150 selects lines 4, 5, 7, 9 and 11, using flags 491, 492 and 490 for the sub-branches.
- Entity proximity (handler 3), line rotation: flag 301 set: lines 13, 14, 15. Else flag 216: lines 16, 17, 18. Else, with flag 492 set: flag 202 gives lines 19, 20, 21; else flag 150 gives 22, 23, 24; else flag 7 gives 25, 26, 27; else no line. With flag 492 clear: if flag 490 is clear, SetGameState(490, 1) and line 32; if flag 490 is set and flag 491 is set, SetGameState(492, 1), RemoveItemFromHero(item 70, count 5) and line 28; if flag 490 is set and flag 491 is clear, lines 29, 30, 31 (High).
- Thread 1: SlotLoad(the stored slot, set 1 when flag 10 is set, set 0 when it is clear, true, true). Then, in every case, SetToNormalCamera(5.0 s, 0.0, flag 1) (High).
- Update and exit: none.
- Coverage: described.
- Confidence: High for the entity handler and thread 1; Medium for thread 2.

### 1139: Exterior, a teleport with a blackboard value and an entity dialogue

- Family: Exterior.
- Init: the debug query is read once and no branch tests its result, so the module makes no debug-only call. RegisterEntityProximityCallback(entity 0, handler 1, flag 1); (entity 1, handler 2, flag 1); (entity 2, handler 3, flag 1); (entity 3, handler 4, flag 1); (entity 4, handler 5, flag 1). RegisterActivateWorldCallback(handler 6). Each handler shows its line with SetConversationTarget(record), LevelMsg(line), EndConversation(record).
- If flag 35 is set: InitObjOn(0, true); if it is clear: InitObjOn(0, false). If flag 304 is clear: SetGameState(304, 1), a debug line, then SetGameState(13, 1). Init then calls a helper that registers the teleport callbacks: index 0 with context 13 and handler 7 in both cases, and index 1 with handler 7 and context 13 when flag 35 is set or context 3 when it is clear. The helper holds three registration call sites and each run executes two of them (Medium-High). Then AddBlackboardMessage(type 2, value 10, eWorlds 109).
- Entity 3 proximity (handler 4), line rotation: flag 319 set: lines 7, 8, 9; else flag 305: lines 4, 5, 6; else lines 1, 2, 3 (High).
- Entity 0 proximity (handler 1), line rotation: flag 319 set: lines 33, 34, 35; else flag 305: lines 30, 31, 32. Else, with flag 678 clear: SetGameState(678, 1) and line 36. With flag 678 set and the hero holding item 23: RemoveAllOfItemFromHero(item 23), AddBlackboardMessage(type 2, value 10, eWorlds 109), SetReturnWorldForFMV(eWorlds 109 (pack 1129)) and PlayFMV(49, true), with no line (the world change to pack 1129 follows the movie). With flag 678 set and no item 23: lines 27, 28, 29 (High).
- Entity 1 proximity (handler 2), line rotation: flag 319 set: lines 52, 53, 54; else flag 305: lines 49, 50, 51; else flag 678 set: lines 46, 47, 48; else line 55 (High).
- Entity 2 proximity (handler 3), a one-time sale: a module data word holds the offered item (file value -1). While it is negative, Random(1, 5) picks the offer: 1 gives item 50 with prompt 10, 2 gives item 152 with prompt 11, 3 gives item 54 with prompt 12, 4 gives item 55 with prompt 13, 5 gives item 56 with prompt 14. The item is stored in the word before the prompt. SetConversationTarget(record) and LevelPrompt(prompt, -1). Declined: line 16. Accepted with GetNeopoints() below 500: line 17. Accepted with enough: GiveItemToHero(the item, count 1, false), RemoveNeopoints(500), line 15. Once the word is set, later talks use a line rotation instead: flag 319 set: lines 24, 25, 26; else flag 305: 21, 22, 23; else 18, 19, 20. The offer is made only once while the module's data words last, whether it was accepted or not: entering an interior from 1139 and leaving, or walking into the adjoining transition and back, makes no new offer; leaving through the transition to another exterior and returning, any driver reload of the exterior buffer (including a save load) or a new game returns the word to -1, so a new offer with a new random pick is made (High).
- Entity 4 proximity (handler 5), line rotation: flag 319 set: lines 43, 44, 45; else flag 305: lines 40, 41, 42; else lines 37, 38, 39 (High).
- Activate-world (handler 6): reads the blackboard value of type 2 for eWorlds 114. Value 10 gives MoveHero(locator 1, flag 1, control 1, extra 1); value 20 gives MoveHero(locator 0, 1, 1, 1). Any value other than -1: SetToNormalCamera(5.0, 0.0, flag 1). Then AddBlackboardMessage(type 2, value -1, eWorlds 114) and SetToNormalCamera(5.0, 0.0, flag 1) again.
- Teleport (handler 7): index 0: StartExteriorReplacementLoad(eWorlds 90 (pack 1023), -1, no colour). Index 1: with flag 35 set, AddBlackboardMessage(type 2, value 10, eWorlds 109) and StartExteriorReplacementLoad(eWorlds 109 (pack 1129), -1, no colour); with flag 35 clear, LevelMsg(0) and no load. Then Wait(3.0 s) for every index (High).
- Update and exit: none.
- Coverage: fully described.
- Confidence: High.

### 1140: Interior, dialogue with a buy callback and a sequence of item checks

- Family: Interior.
- Init: SlotAllocate(2); SlotCreate(2, table 0,1), with the return values stored. ControlThreadsAllocate(2) runs in every case, and so do ControlThreadStart(thread 1, argument world) and ControlThreadStart(thread 2, argument 0). The flag-10 test selects a stored value and does not guard these calls (High). RegisterEntityProximityCallback(entity 0, handler 3, flag 1). RegisterItemAskAboutCallback(item 116, handler 4, flag 1). RegisterHeroBuyItemCallback(handler 5, flag 0).
- Thread 2 (argument 0): dialogue pattern C with lines 36, 1, 3, 5, 7, 9 (flags 301, 216, 481, 483, 202, 150 cascade).
- Entity proximity (handler 3): dialogue pattern C with lines 38, 19, 22, 25, 28, 35 (flags 499, 301, 216, 483, 202, 150); when flag 481 is set and flag 483 clear and the hero has item 69: SetGameState(483, 1), RemoveItemFromHero(item 69, count 1), LevelMsg(31); when the hero does not have item 69: LevelMsg(32); when flag 481 is clear: SetGameState(481, 1) is set with a line.
- Item-ask (handler 4): dialogue pattern C with lines 11 to 13 (flags 301, 216, 202, 150).
- Buy-item (handler 5): when the bought item is 116 and flag 220 is clear: LevelMsg(0).
- Thread 1: SetToNormalCamera(5.0 s, 0.0, flag 1). Thread 2: SlotLoad(the stored slot, set 1 or 0) by the flag 10 state, activate true (Medium).
- Update and exit: none.
- Coverage: described.
- Confidence: Medium.

### 1143: Interior, dialogue with item trades and a slot chooser

- Family: Interior.
- Init, in every case: SlotAllocate(1); SlotCreate(2, table 0,1), with the return values stored. ControlThreadsAllocate(1). ControlThreadStart(thread 1, argument world), and ControlThreadStart(thread 2) with argument 1 or 0, the value chosen by flag 10 (Medium). RegisterEntityProximityCallback(handler 3, flag 1) with entity 1 or entity 0, chosen by flag 10 (Medium). The flag-10 test does not decide whether these calls run (High).
- Handler 2 (thread 2, reading the chooser pattern C with a trade): when flag 301 is set, lines 0 and 1; when clear, the cascade by flags 216, 386, 387, 202, 150. When flag 386 is set and 387 clear: LevelMsg(4) and then (if 387 is also clear) SetGameState-type dialogue lines 6 and 10.
- Handler 3 (entity proximity): pattern C with lines 12, 15, 18, 21, 24 and 7. It includes a trade with item 2: if flags 387 and 386 are in the stated state and the hero has item 2, SetGameState(387, 1), RemoveItemFromHero(item 2, count 1), LevelMsg(27); if the hero does not have item 2: LevelMsg(28) and SetGameState(386, 1).
- Thread 1: SetToNormalCamera(5.0 s, 0.0, flag 1), or SlotLoad(the stored slot, set 1 or 0, true, true).
- Update and exit: none.
- Coverage: described.
- Confidence: Medium.

### 1148: Interior, dialogue with item trades, a camera and a walk-to-entity pattern

- Family: Interior.
- Init: SlotAllocate(1); SlotCreate(1, table 0), with the return value stored. ControlThreadsAllocate(1). ControlThreadStart(thread 1, argument world). RegisterEntityProximityCallback(entity 0, handler 2, flag 1); (entity 1, handler 3, flag 1); (entity 3, handler 4, flag 1); (entity 2, handler 5, flag 1).
- Each line below is shown with SetConversationTarget(record), LevelMsg(line), EndConversation(record), after the handler's flag and item steps.
- Handler 5 (entity 2): line rotation 0, 1, 2 (no flag test).
- Handler 3 (entity 1), by the first matching case:
  1. Flag 202 set: lines 8, 9, 10. Flag 151 set: lines 12, 13, 14. Flag 150 set: lines 15, 16, 17 (rotations).
  2. Flag 143 set: if the hero has item 0: SetGameState(150, 1), SetGameState(142, 1), RemoveItemFromHero(item 0, count 1), SetGameState(3, 1), SetGameState(8, 1), GiveItemToHero(item 120, count 1, false), line 18. Without item 0: lines 19, 20, 21.
  3. Flag 142 set: if the hero has item 38: SetGameState(143, 1), SetGameState(17, 1), SetGameState(2, 1), RemoveAllOfItemFromHero(item 38), line 22. Without item 38: lines 23, 24, 25.
  4. Flag 138 set: lines 26, 27, 28.
  5. Flag 137 set: if the hero has item 106: SetGameState(138, 1), GiveItemToHero(item 39, count 1, false), line 29. Without item 106: lines 30, 31, 32.
  6. Flag 137 clear: RemoveAllOfItemFromHero(item 32) and SetGameState(137, 1) in either case; if the hero has item 106, also SetGameState(138, 1), GiveItemToHero(item 39, count 1, false) and line 34; otherwise line 33 (High).
- Handler 4 (entity 3), the shop step:
  1. Flag 140 set: lines 35, 36, 37. Flags 140 and 139 clear: lines 38, 39, 40.
  2. Flag 140 clear and flag 139 set: SetConversationTarget(record) and LevelMsg(41) on the first talk in this state, LevelMsg(42) on later talks (a module word). Then LevelPrompt(43, -1). Accepted: with GetNeopoints() of at least 100, RemoveNeopoints(100), GiveItemToHero(item 42, count 1, false), SetGameState(140, 1), SetGameState(644, 1) and line 44; with less than 100, line 45. Declined: LevelPrompt(46, -1). Accepted: if the hero has item 98, RemoveAllOfItemFromHero(98) and the module's "traded item" word becomes 98; else if the hero has item 99, RemoveAllOfItemFromHero(99) and the word becomes 99; then, in all three cases, GiveItemToHero(item 42, count 1, false), SetGameState(140, 1), SetGameState(644, 1) and line 47. Declined again: line 48.
  3. After the line, when flag 140 is set, flag 143 is clear and the hero has neither item 98 nor item 99, the walk-back sequence runs (this includes the talk that has just set flag 140).
  Edge case: accepting prompt 46 without holding item 98 or item 99 still gives item 42 and sets flags 140 and 644 (an original quirk; High).
- Walk-back sequence: waits while IsHeroTalking() is non-zero (Sleep(1) each tick); LevelMsg(60); WalkHeroAndOrient(locator 4, wait 0, control 0); WalkEntityAndOrient(entity 3 to locator 3, wait 0, speed 1.1, control 1); WalkEntityAndOrient(entity 0 to locator 6, wait 1, speed 1.1, control 1); LevelMsg(61); WalkEntityAndOrient(entity 0 to locator 5, wait 1, speed 1.1, control 1); LevelMsg(62); GiveItemToHero(the traded-item word, count 1, false); WalkEntityAndOrient(entity 0 to locator 7, wait 0, speed 1.1, control 1); ReturnHeroAnimToIdle(). The traded-item word has the file value 99, so after a purchase with Neopoints the hero receives item 99 here (High for the calls; Medium for the argument roles of the walk calls).
- Handler 2 (entity 0), line rotation: flag 202 set: lines 49, 50, 51; else flag 150: lines 55, 56, 57; else line 58 when flag 139 is set and line 59 when it is clear (High).
- Thread 1: if flag 496 is set, SlotLoad(the stored slot, set 0, true, true). Then, in every case, SetToNormalCamera(5.0 s, 0.0, flag 1) (High).
- Update and exit: none.
- Coverage: fully described.
- Confidence: High, except where marked.

### 1172: Interior, return trigger, dialogue, two thread starts and a camera reset

- Family: Interior.
- Init: ControlThreadsAllocate(2). ControlThreadStart(thread 1, argument world). ControlThreadStart(thread 2, argument -1). RegisterEntityProximityCallback(entity 0, handler 3, flag 1).
- Handler 3 (entity 0): when flag 301 is set: SetConversationTarget(record), LevelMsg(10), EndConversation(record). Otherwise the chooser pattern C: flag 216 gives LevelMsg(13); flag 136 set gives lines 16, 19, 22 and 25, selected by flags 202 and 193 and 150; flag 136 clear gives LevelMsg(28).
- Thread 1: if flag 10 is set: SetToNormalCamera(5.0, 0.0, flag 1) or a debug line (no effect). Otherwise a debug line only.
- Thread 2 (argument -1): the on-enter chooser for flag 301 and flag 216: lines 0, 2, 4, 6, 7 and 8 under flags 202, 150 and 7.
- Update and exit: none.
- Coverage: described.
- Confidence: Medium.

### 1230: Exterior, a slot chooser, a treasure-map exchange and button-prompt triggers

- Family: Exterior.
- Init:
  1. SlotAllocate(2); SlotCreate(6, table 0,1,2,3,4,5), with the return values stored. ControlThreadsAllocate(2); ControlThreadStart(thread 1, argument world).
  2. RegisterScriptTrigger(handler 2, index 0), (handler 2, index 2), (handler 2, index 1); (handler 3, index 5).
  3. RegisterEntityProximityCallback(entity 46, handler 4, flag 1) and (entity 84, handler 4, flag 1).
  4. When flag 20 is set: RegisterScriptTrigger(handler 5, index 3) and (handler 6, index 4). When flag 19 is set, the same two registrations run again, whatever flag 20 is; with both flags set, each of the two triggers is registered twice. When flag 64 is set and flag 248 is clear: InitObjOn(object 0, flag 0) (High). When flag 248 is set: SetWorldCollisMask(1) and SetWorldDisplayMask(1); these two calls are not gated by flags 19, 20 or 64.
- Triggers 0, 1 and 2 (handler 2), when entered: the context prompt (context 3) is shown; if button 0 is pressed, the line depends on the trigger index: index 0 gives line 18, index 2 gives line 19, index 1 gives line 20.
- Trigger 5 (handler 3), when flag 64 is set and flag 248 is clear and the object is present: BlockPlayerControl(hero 0), LevelMsg(21), ReturnHeroAnimToIdle().
- Trigger 3 (handler 5), entered with object present and a "talked" module word clear: sets the word, then entity 46 speaks with a line rotation: flag 67 set gives lines 0, 1, 2; clear gives lines 3, 4, 5 (SetConversationTarget(entity 46), the line, EndConversation(entity 46)).
- Trigger 4 (handler 6), entered with object present: clears the "talked" word, so trigger 3 talks again the next time (High; the same pair as triggers 0 and 1 of module 880).
- Entity 46 and entity 84 proximity (handler 4): the treasure-map shop, the same scheme as module 880 with this list: map 1 (prompt 6, 250 Neopoints), map 8 (prompt 7, 250), map 30 (prompt 8, 250), map 27 (prompt 9, 300), map 18 (prompt 10, 500), map 10 (prompt 17, 300). The first unowned map is offered with LevelPrompt(prompt, -1). Declined: line 16. Accepted with GetNeopoints() below the price: line 15. Accepted with enough: GiveItemToHero(item 175, map id, false), RemoveNeopoints(price), line 14. All six owned: line rotation 11, 12, 13 with no prompt (High).
- Thread 1 (High): when flag 248 is set, the thread creates a new slot whose buffer is sized for set 6 (one SlotCreate entry, value 6) and loads set 6 into that new slot, with keep and activate true. This slot is not the stored slot and is not kept in a module word. Otherwise a set is chosen for the stored slot: with flag 20 set, flag 529 set and flag 530 clear give set 5, and every other case with flag 20 set gives set 4. With flag 20 clear, flag 19 gives set 3, flag 18 gives set 2, flag 52 gives set 1, and none gives set 0. Each of these loads uses the stored slot with keep and activate true; debug lines are printed in each branch.
- Update, every tick: unless flag 64 is set with flag 248 clear, if object 0 is active: DeactivateObj(object 0). So object 0 stays only while flag 64 is set and flag 248 is clear, the same gate as its InitObjOn (High).
- Exit: none.
- Coverage: fully described (one routine that nothing references is listed under Dormant routines).
- Confidence: High for the treasure-map handler, the trigger-button lines and the update; Medium for the thread.

### 1244: Exterior, a hero swap with dialogue, an entity talk and a break sequence

- Family: Exterior.
- Init: SetGameState(271, 1). If flag 273 is clear: InitObjOn(5, false), InitObjOn(0, false), InitObjOn(1, false), InitObjOn(4, false); RegisterScriptTrigger(handler 1, index 0); (handler 2, index 1); (handler 3, index 2); (handler 4, index 4); (handler 5, index 3).
- Trigger 0 (handler 1) is Roberta's entry scene and revives the parked hero when one is dead. It runs when the trigger object is present, flag 272 is clear, the trigger record is in its first-fire state (1), and GetHeroSwapMode() equals 2: for hero 0 and then hero 1, if IsHeroDead(hero) is true, NeedHeroAlive(hero) (the helper revives the partner of the selected hero when it is dead, SPEC-0055, section 7). The trigger volume is a sphere of radius 23.87 at (859.22, 133.30, -1963.18) with the hero-contact flag. Then BlockPlayerControl(-1); SetHeroEquippedState(false); FadeScreenOut(1.0 s, wait 1); ActivateHero(hero 1, locator 16, 0, 0, 0); Sleep(30); FadeScreenIn(1.0 s, wait 0); LetterBoxOn; PlayCamera(4, wait 1, camera wait 1, 0.3); LevelMsg(0), LevelMsg(1), LevelMsg(2), LevelMsg(3); then lines 4 to 10, each shown after IsDialogActive() has gone to zero (Sleep(1) per poll); MoveHero(locator 14, 1, 0, 1); PlayCamera(11, wait 0, camera wait 1); LevelMsg(11) after the dialogue ends; LetterBoxOff; ReturnToNormalCamera(19, 1); SetHeroSwapMode(1). The routine sets no flag (High for the calls and order).
- Trigger 1 (handler 2): when object present and flag 273 clear: dialogue line 13 with IsDialogActive; WalkHero(locator 13, wait 1, control 1); Sleep(1) while dialogue is active.
- Trigger 2 (handler 3): as handler 2 with line 12 and WalkHero(locator 14).
- Trigger 4 (handler 4): when entered, flag 272 is clear and level state 0 is clear (so it runs once while level state 0 stays set): SetLevelState(0, true); BlockPlayerControl(hero 1); LetterBoxOn(1.0 s, wait 0); WalkHeroAndOrient(locator 20, wait 0, control 1); PlayCamera(12, wait 1, camera wait 1, 1.0 s); dialogue line 14 and 15 and 16 with PlayCamera(13) and PlayCamera(16); WalkHero(locator 21, wait 0, control 0); LetterBoxOff(1.0 s, wait 0); ReturnToNormalCamera(-1, 1).
- Trigger 3 (handler 5), on its first fire (trigger-record state 1) while the object is present and flag 272 is clear, in this order:
  1. LetterBoxOn; BlockPlayerControl(hero 1); MoveHero(locator 22, flag 1, control 0, extra 1); SetHeroEquippedState(false); LevelMsg(17).
  2. PlayCamera(14, wait 1, camera wait 1); PlayCamera(16, wait 0, camera wait 1); PlaySound(7, no position, false); PlayAnim(5, wait 1); KillSound(7); StartCameraShakeMode(type 0); PlaySound(8); Break(breakables 17 to 24, one call each); Sleep(20).
  3. PlayCamera(17, wait 0, camera wait 1); PlayHeroAnim(125, wait 1, control 1); BlockPlayerControl(1); Sleep(10); PlayCamera(16, wait 0, camera wait 1); PlayHeroSound(1); LevelMsg(18).
  4. DeactivateObj(5), DeactivateObj(4), DeactivateObj(1); ActivateObj(6, false); PlaySound(9); PlayAnim(6, wait 1); KillSound(9); PlayCamera(16, wait 0, camera wait 1); LevelMsg(19); Sleep(1).
  5. FadeScreenOut(wait 1); MoveHero(locator 12, 1, 1, 1); PlayCamera(5, wait 0, camera wait 1); FadeScreenIn(wait 0); SetHeroSwapMode(0); ActivateHero(hero 0, locator 12, 0, 0, 0); LevelMsg(20); LevelMsg(21).
  6. SetGameState(272, 1); FadeScreenOut(wait 1); LetterBoxOff; ReturnToNormalCamera(-1, 1); Sleep(30); FadeScreenIn(wait 0).
  Flag 272 ends the sequence for good: it also blocks triggers 0, 3 and 4 (High).
- Update and exit: none (no Exit calls in the module's recorded calls).
- Coverage: fully described.
- Confidence: High for the calls and order; Medium for argument roles not named above.

### 1246: Exterior, the Werelupe King arena: an entry walk, a level-state fight cascade and the reward

- Family: Exterior.
- Init: InitObjOn(5, false); InitObjOn(3, false). RegisterScriptTrigger(handler 1, index 1); (handler 2, index 0).
- Volumes (High): trigger 1 is a sphere of radius 20.80 at (868.03, 5.63, -2146.51), the arena entrance. Trigger 0 is a sphere of radius 35.83 at (736.27, 0.31, -2126.62), the arena. Both have volume flag bit 3, so the hero's per-tick contact scan fires them while the hero is inside. Locator 10 lies inside sphere 1, locator 11 (752.0, 5.1, -2135.3) inside sphere 0, and locator 9 (696.3, 4.7, -2122.4) outside sphere 0 (about 40 units from its centre).
- Update, every tick: when flag 273 is clear and level state 1 is set: ForceTriggerScriptIfNotRunning(trigger 0). No call of this module sets level state 1, and nothing else in this world does either (below), so in play this path does not run; the hero's contact with sphere 0 drives trigger 0 instead (Medium).
- Trigger 1 (handler 1): when entered and level state 0 is clear: SetLevelState(0, true); WalkHero(locator 10, wait 0, control 1); FadeScreenOut(1.0 s, wait 1); MoveHero(locator 11, flag 1, control 1, extra 1); SetHeroStartPos(11); Sleep(1); BossFightTweakHero(encounter 14, stage 0); if movie 41 has not played: PlayFMV(41, true).
- Trigger 0 (handler 2; fired by the hero's contact with sphere 0, and by Update when that path runs). Each fire runs the routine when it is not already running. The routine tests no record field. Its branches, chosen in this order (High):
  1. Level state 8 set: GrabEntityControl(entity 16); DeactivateEntity for entities 16 to 24; SetGameState(273, true); Sleep(60 ticks).
  2. Level state 8 clear and flag 273 set: LoopEntityAnim(16, animation 27, 0.0 s, control 0); Wait(1.3 s); FadeScreenOut(1.0 s, wait 1); PlayFMV(42, true) if movie 42 has not played; SetLevelState(8, true); GrabEntityControl(16); DeactivateEntity for entities 16 to 24; SetHeroSwapMode(2); DeactivateObj(1), DeactivateObj(3), DeactivateObj(0); BossFightTweakHero(encounter 14, stage 1); ActivateObj(2, false); MoveHero(locator 9, flag 1, control 0, extra 1); ReturnToNormalCamera(-1, 1); FadeScreenIn(1.0 s, wait 0); Sleep(30); GiveItemToHero(item 14, count 1, false); LevelMsg(1); SetLevelState(8, true).
  3. Level state 8 clear and flag 273 clear: the fight cascade, tested from level state 7 down:
     - Level state 7 set, or 5 set, or 3 set (with the higher states clear): Sleep(1) and return (the fight is running).
     - Level state 6 set (7 clear): GrabEntityControl(16); FadeScreenOut; LetterBoxOn; MoveHero(locator 6); MoveEntity(16, locator 7); FadeScreenIn; PlayCamera(6); a wait; PlayEntityAnim(16, animation 17, wait); a wait; ReleaseEntityControl(16); ReturnToNormalCamera(-1, 1); LetterBoxOff; SetLevelState(7, true).
     - Level state 4 set (7, 6 and 5 clear): GrabEntityControl(16); FadeScreenOut; LetterBoxOn; MoveHero(locator 6); MoveEntity(16, locator 7); FadeScreenIn; PlayCamera(5); Wait(0.8 s); PlayEntityAnim(16, animation 16, wait); LevelMsg(0) after a wait; LetterBoxOff; ReleaseEntityControl(16); SetLevelState(5, true); ReturnToNormalCamera(-1, 1).
     - Level state 2 set (7 to 3 clear): ActivateObj(1) and ActivateObj(0); DeactivateObj(5); ActivateObj(4); DeactivateEntity(21); PlayCamera(4); LetterBoxOn; MoveEntity(16, locator 7); FadeScreenIn; Sleep(30); WalkHero(locator 8); PlayEntityAnim(16, animation 18, wait); a wait; FadeScreenOut; ReturnToNormalCamera(-1, 1); LetterBoxOff; ActivateEntity(21); FadeScreenIn; SetMaxActiveAI(12); ReleaseEntityControl(16); SetLevelState(3, true).
     - None of 2 to 7 set: if movie 41 has played, SetLevelState(2, true).
- Setters outside the module (High for the King; Medium for level state 1):
  - The Werelupe King (SPEC-0019) writes the world's level states with the same set as SetLevelState: level state 4 when its hit points fall below 75 percent (phase 2) and level state 6 below 37 percent (phase 3). Its death state sets flag 273 with the same set as SetGameState.
  - Level state 1: no setter in this world. The engine's other writers of level state 1 belong to the Faerie Sisters' shield (SPEC-0011), the Ixi Chief's stage transition (SPEC-0014), the Juppie fight (SPEC-0015) and a mouse-catch counter. None of those creatures or set pieces is in this world (Medium).
  - Level state 8 is set by branch 2 itself, after flag 273.
- Play order (High for the calls; Medium for the contact-fire timing):
  1. The hero enters sphere 1: walk, fade, placed at locator 11 inside the arena, movie 41 the first time, boss refill 14 stage 0.
  2. In sphere 0, the first fire after movie 41 sets level state 2; the next fire runs the intro and sets level state 3 (AI cap 12, entity 21 active). The King was bound on the first streaming pass after step 1 and is free (state 1) but has no attack available until about 481 ticks after its bind (SPEC-0019).
  3. The King below 75 percent: level state 4; the next fire plays the phase cutscene (camera 5, line 0) and sets level state 5. Below 37 percent: level state 6; cutscene (camera 6) and level state 7.
  4. The King dies: flag 273. The hero is still in sphere 0, so the next contact fire takes branch 2: movie 42 the first time, swap mode 2, boss refill stage 1, the hero moved to locator 9 outside the sphere, item 14 (one), line 1, level state 8.
  5. Entering sphere 0 again in the same load runs branch 1 only.
- What makes the King active (R-0064; High for the rules, Medium-High for the timeline). Nothing in this module does. The King is record 16 of the pack's embedded character block (entity 16); it is bound, in base state 1 with its controller running, by the area's character streaming as soon as it is among the nearest eligible records to the hero (SPEC-0006), which happens on the first streaming pass after the entrance script puts the hero at locator 11 (the King is 23.3 units from it, the third nearest record). The module registers no activation filter and its data holds no constructor list or callback table; its two triggers are registered once each from Init. BossFightTweakHero(14, 0) only refills the heroes, SetMaxActiveAI(12) raises the embedded block's cap from 6 to 12 (the intro's last step) so that more of the arena's enemies bind, and ActivateEntity is called only for entity 21 (the intro first removes it with DeactivateEntity(21) and force-binds it again at the end). The script's part is to hold the King: GrabEntityControl(16) (base state 33, controller suspended) in the phase cutscenes, MoveEntity and PlayEntityAnim in the intro, and ReleaseEntityControl(16) (state 1) afterwards. Worked timeline with distances and tick order: SPEC-0019, Activation in play.
- Edge cases: the MoveEntity calls in this module give 1 as their fifth argument. Branch 2 tests only level state 8 and flag 273, so a later load that starts with level state 8 clear and flag 273 set gives item 14 again on the first contact with sphere 0 (movie 42 is skipped) (Medium; level-state lifetime per SPEC-0030).
- Test vectors:
  1. Flag 273 clear, movie 41 played, no level state set, hero in sphere 0: one fire sets level state 2 and nothing else.
  2. Level states 3 and 4 set: one fire plays the camera-5 cutscene and sets level state 5.
  3. Flag 273 set, level state 8 clear: one fire gives item 14 once, sets swap mode 2 and level state 8, and leaves the hero at locator 9.
  4. Flag 273 set and level state 8 set: one fire removes entities 16 to 24 and gives nothing.
- Exit: none.
- Coverage: fully described; the camera waits and fade colours in the cascade are summarized.
- Confidence: High for the branches and their order, the volumes and the King's writes; Medium for the absence of a level-state-1 writer and for the reload edge case.

### 1248: Exterior, a multi-object sequence with sound and a dialogue exit state

- Family: Exterior.
- Init: if flag 79 is clear: InitObjOn for objects 3, 4, 5, 2, 0, 1 (flag 0 each). In both cases: InitObjOn for objects 30, 29, 28, 27, 43, 45, 41, 41, 37, 37, 33, 33, 25 and 26 (flag 0 each), which is 20 object calls when flag 79 is clear and 14 when it is set; objects 41, 37 and 33 are each set twice. LoopAnim(30, 29, 28, 27; speed 1.0); AddAnimObjSound(objects 3, 4, 5, 2, 0, 1; sound 16, 0.1 s, 1.0 s). RegisterScriptTrigger(handler 1, index 4); (handler 2, index 27); (handler 3, index 18); (handler 4, index 17); (handler 5, index 16); (handler 6, indices 1, 2, 3); (handler 7, index 0). RegisterTeleportCallback(handler 8, index 0, context 13, world). RegisterActivateWorldCallback(handler 9).
- Exit: IsObjActive(0), IsObjActive(2) and IsObjActive(1). If objects 0 and 2 are inactive and 1 is active: SetGameState(79, 1).
- PlayLookAtAnimObjCamera(camera, object, wait, camera wait) below: the first argument is a camera id and the second the object it looks at (Medium for the roles; the pairs match the looping objects 27 to 30 set up in Init).
- Trigger 4 (handler 1): object 43 active and animation state checks (IsObjAnimating, IsObjAnimFinished); when the object is finished and the hero is aligned to axis 2 (AlignAxisToVectorToHero(object 30, axis 2, 0.05 s)): PlayLookAtAnimObjCamera(camera 11, object 30, wait 0, camera wait 0); Wait(0.4 s) in a loop; then plays the object 25 and object 26 sequence (ActivateObj, PlayAnim, DeactivateObj), PlaySound(16) and Wait(2.0 s) (Low-Medium).
- Trigger 27 (handler 2), when the record field at +0x20 is non-zero: PlayLookAtAnimObjCamera(camera 11, object 30); PlayAnim(43, wait 0); PlayAnim(45, wait 1); ActivateObj(25); PlayAnim(25, wait 1); ActivateObj(26, false); PlaySound(15); ReturnToNormalCamera(-1, 1); ActivateObj(44) and ActivateObj(46); PlayAnim(44, wait 0); PlayAnim(46, wait 1); ActivateObj(43) and ActivateObj(45) (Medium).
- Triggers 16, 17 and 18 (handlers 5, 4 and 3), each when the record field at +0x20 is non-zero, play a camera sequence that animates and removes two of objects 0 to 5 (High for the calls and order):
  - Trigger 16: PlayLookAtAnimObjCamera(camera 8, object 27); PlayAnim(31, wait 0); PlayAnim(33, wait 1); camera 7 on object 5; PlayAnim(5, wait 0); DeactivateObj(5); camera 5 on object 1; PlayAnim(1, wait 0); DeactivateObj(1); ReturnToNormalCamera(-1, 1).
  - Trigger 18: camera 9 on object 29; PlayAnim(39, wait 0); PlayAnim(41, wait 1); camera 7 on object 4; PlayAnim(4); DeactivateObj(4); camera 6 on object 2; PlayAnim(2); DeactivateObj(2); ReturnToNormalCamera(-1, 1).
  - Trigger 17: camera 10 on object 28; PlayAnim(35, wait 0); PlayAnim(37, wait 1); camera 12 on object 3; PlayAnim(3); DeactivateObj(3); camera 4 on object 0; PlayAnim(0); DeactivateObj(0); ReturnToNormalCamera(-1, 1).
  So triggers 17 and 18 without trigger 16 leave objects 0 and 2 inactive and object 1 active, the state in which Exit sets flag 79.
- Trigger 0 (handler 7), entered with object present, a "shown" module word clear and flag 269 clear: when the animations of objects 6 and 12 have finished, LevelMsg(0) and the word is set, so the line shows once while the module stays loaded (High).
- Triggers 1, 2 and 3 (handler 6), state non-zero and object present: set a hold-off module word to 5. Update lowers it by 1 per tick while it is non-zero (Update makes no host call). Trigger 4 runs its body only while the word is 0, so it waits 5 ticks after the hero last touched triggers 1 to 3 (High for the stores; Medium for the reading).
- Teleport 0 (handler 8), when the object is present: DisplaySquareButtonContext(context 13); if button 0 is pressed: StartExteriorReplacementLoad(eWorlds 147 (pack 1257), -1, no colour) (High).
- Activate-world (handler 9): reads the blackboard value of type 2 for eWorlds 147. Value 20: MoveHero(locator 5, 1, 1, 1). Any value other than -1: SetToNormalCamera(5.0, 0.0, flag 1). The pack-1257 teleport that loads this pack (eWorlds 139) posts value 20 for eWorlds 147 first (High).
- Coverage: fully described.
- Confidence: High for init, exit, triggers 0 and 16 to 18, the teleport and the activate-world handler; Medium or lower for the rest.

### 1257: Exterior, a long trigger table, a teleport pair and a shop-style world swap

- Family: Exterior.
- Init: RegisterScriptTrigger(handler 1, index 17); RegisterScriptTrigger(handler 2, indices 1 to 16, each registered once). RegisterTeleportCallback(handler 3, index 0, context 13, world); (handler 4, index 2, context 13, world); (handler 5, index 1, context 13, world). RegisterActivateWorldCallback(handler 6). RegisterScriptTrigger(handler 7, index 0). SlotAllocate(1); ControlThreadsAllocate(1); ControlThreadStart(thread 8, argument 0).
- Thread 8: SlotCreate(1, table 0). Its return value is passed directly to the SlotLoad that follows, and is also written to a module data word that no SlotLoad or SlotReset in this module reads (Medium): SlotLoad(that SlotCreate return value, set 0, false, activate true).
- Trigger handler 2 (indices 1 to 16, entered by any listed index): GetCollisInfo(index 3) is compared with the record; prompt context 3; when button 0 is pressed, a 1-tick sleep loop and LevelMsg(1); otherwise GetCollisInfo(16) and GetCollisInfo(1) select LevelMsg(2) and LevelMsg(3). Medium.
- Handler 1 (index 17): when flag 459 is set and flag 77 is clear and the hero does not have item 29, and level state 0 is clear: SetLevelState(0, true) and LevelMsg(0). Medium.
- Handler 7 (index 0, state 1 and flag 676 clear): BlockPlayerControl(hero -1), FadeScreenOut(1.0 s, wait 1), LetterBoxOn(0 s, wait 0), MoveHero(locator 4, flag 1, control 0, extra 1), FadeScreenIn(0.7 s, wait 0), CharacterActivate(character from module data, 0), Sleep(2), SetDefaultEntityIdleAnim(entity 0, animation 112), LoopEntityAnim(entity 0, animation 112, 0 s, control 0), PlayCamera(4, wait 0, camera wait 1, 1.0 s), WalkHero(locator 5, wait 1, control 0), Sleep(30), a dialogue with LevelMsg(17), PlayCamera(5), SetConversationTarget(entity 0), LevelMsg(18), EndConversation(entity 0), PlayCamera(6, wait 1, camera wait 1, 1.0 s), PlayCamera(8), PlayEntityAnim(2, animation 10, wait 1, control 1), LoopEntityAnim(2, animation 1, 0 s, control 0), PlayCamera(7), SetHeroEquippedState(true), WalkEntity(entity 0 to locator 6, wait 0, 1.1 s, control 0), PlayEntityAnim(3, 10, wait 1, control 1), SetAI_Agro(3), LevelMsg(19), PlayEntityAnim(0, animation 104, wait 1, control 0), CharacterDeactivate(character from module data, 0), PlayEntityAnim(1, 10, wait 0, control 1), SetAI_Agro(1), SetAI_Agro(2), LetterBoxOff(1.0 s, wait 0), ReturnToNormalCamera(-1, 1), SetGameState(676, 1).
- Handler 3 (teleport 0): when button 0 is pressed: AddBlackboardMessage(type 2, value 10, eWorlds 147), a debug line and StartExteriorReplacementLoad(eWorlds 138, -1, no colour) with Wait(3.0 s) in the sequence. Handler 4 and handler 5 are the same shape with values 20 and 30 and eWorlds 139 and 140.
- Activate-world (handler 6): reads the blackboard value of type 2 for eWorlds 147. Value 20 gives MoveHero(locator 2, flag 1, control 1, extra 1); value 10 gives MoveHero(locator 0, ...); value 30 gives MoveHero(locator 1, ...). Otherwise SetToNormalCamera(15.0 s, 0.0, flag 1) and AddBlackboardMessage(type 2, value -1, eWorlds 147).
- Update and exit: exit is exported and empty.
- Coverage: described.
- Confidence: Medium for handlers 2, 6 and the teleports; High for handler 7 order (first-seen order of a single path).

### 1344: Transition, two movie triggers, an animated toggle pair and a slot thread

- Family: Transition.
- Init: RegisterEntityProximityCallback(entity 27, handler 1, flag 1). DebugPrintf. SlotAllocate(3). If flag 92 is clear: SlotCreate(1, table 3), with its return value stored. If flag 92 is set: SlotCreate(3, table 2,1,0), with its return value stored. In both cases, then: RegisterScriptTrigger(handler 2, index 0), (handler 3, index 1), (handler 4, index 3), (handler 5, index 4), (handler 6, index 2); InitObjOn(objects 2, 3, 4, 5, 1, 0; flag 0); ControlThreadsAllocate(1); ControlThreadStart(thread 7, argument world). The flag 92 test does not gate these calls (Medium-High; the join is after both slot creates).
- Entity 27 proximity (handler 1): SetConversationTarget(record), LevelMsg(0), EndConversation(record).
- Trigger 0 (handler 2), entered with object present: when flag 237 is set and flag 238 is clear: SetGameState(238, 1); PlayFMV(22, true); Sleep(30 ticks); LevelMsg(10).
- Trigger 1 (handler 3): when flag 239 is set and flag 240 is clear: SetGameState(240, 1); PlayFMV(24, true).
- Story role (High): triggers 0 and 1 are the only setters of flags 238 and 240. Flag 239, between them, is set by pack 1324 when its block-plate puzzle is solved (SPEC-0041), so the order in play is 237, 238 (here, movie 22), 239 (1324, movie 23), 240 (here, movie 24) (SPEC-0058 step 25).
- Trigger 3 (handler 4): with flag 322 set, the two objects 2 and 3 toggle: when neither is animating, PlayAnim(2, wait 0, 1.0 s) and PlayAnim(3, wait 0, 1.0 s); after WaitTillLeaveTrigger(record) they play in reverse (speed -1). Sleep(1) between polls. Medium.
- Trigger 4 (handler 5): the same shape for objects 4 and 5 (flag 322). Medium.
- Trigger 2 (handler 6; High): on the first tick of a fire (state 1) with a non-null object, when flag 236 is set and flag 239 is clear: LevelMsg(line 3, W, true, WAIT 1), then LevelMsg(line 4, W, true, WAIT 1), with no conversation bracket. Both lines show, in that order, on every entry while the flags hold. Test vector: flags 236 set, 239 clear: each entry shows line 3 and then line 4; with 239 set, nothing.
- Thread 7: flag 94 set gives SlotLoad(set 0, true, true); else flag 93 gives set 1; else flag 92 gives set 2; else (flags 239 and 240 set) gives set 3; each load passes activate true. The slot word that sets 0, 1 and 2 read is the word the flag-92-set create stores, so it holds a stored slot only when flag 92 was set at Init. The word that set 3 reads is the one the flag-92-clear create stores, so it holds a stored slot only when flag 92 was clear at Init. When a load reads the other word, it passes the file value -1 (Medium-High; both words). A debug line appears in each branch.
- Update and exit: none.
- Coverage: described.
- Confidence: High for the thread, the entity talk and trigger 2; Medium for the toggles.

### 1405: Interior, a long entity-control helper with treasure-map and item-ask lines

- Family: Interior.
- Init: SlotAllocate(1); SlotCreate(1, table 0), with the return value stored; ControlThreadsAllocate(1); ControlThreadStart(thread 5, argument world). These run in every case. Before them, when flag 104 is set, Init runs two helpers in turn: the registration helper (RegisterOnEnterCallback(handler 1, index 0), RegisterEntityProximityCallback(entity 0, handler 2, flag 1), RegisterItemAskAboutCallback(item 125, handler 3, flag 1) and (item 133, handler 4, flag 1)) and then the entity-control setup helper (the Random and GetNumberOfFramesPerSecond calls). When flag 104 is clear, neither runs, and the module registers no callback (High).
- On-enter (handler 1): dialogue pattern C with lines 0 and 2 (flags 100, 106).
- Entity 0 proximity (handler 2): flag 611 set gives SetConversationTarget(record), LevelMsg(14), EndConversation(record). Flag 685 controls the item RemoveAllOfItemFromHero(item 33) and SetGameState(685, 1) on the first path.
- Item-ask 125 (handler 3): LevelMsg(5) after WhichHeroIsActive. Item-ask 133 (handler 4): LevelMsg(6).
- Thread 5: a slot load (set 0) from the stored slot, and SetToNormalCamera(15.0 s, 0.0, flag 1) under flag 104 (Medium).
- Helper H4: the shared entity-control routine of SPEC-0043 (section "Shared entity-control routine"; one routine in 27 modules, High). The entity-control setup helper adds one record, only when flag 104 is set: entity 0, idle animation 0, idle-animation interval 5 to 8 s, dwell 5 to 7 s, animations 40 (2.5 s), 96 (2.5 s) and 41 (1.3 s), no waypoints. Without flag 104 the table stays empty.
- Update: exported; calls the shared entity-control routine every tick, ungated (High). Exit: exported, empty.
- Coverage: described.
- Confidence: High for the helper and Update; Medium for the registrations and handlers 1 to 4.

### 1407: Interior, the same long entity-control helper as 1405, with a different dialogue

- Family: Interior.
- Init: SetGameState(100, 1). Then flag 100 is tested; since Init has just set it, the test always passes, and Init runs the registration helper (RegisterOnEnterCallback(handler 1, index 0), RegisterEntityProximityCallback(entity 0, handler 2, flag 1), RegisterItemAskAboutCallback(item 105, handler 3, flag 1) and (item 118, handler 4, flag 1)) and then the entity-control setup helper. SlotAllocate, SlotCreate of one set from table 0 (return value stored), ControlThreadsAllocate and ControlThreadStart of thread 5 follow in every case (High).
- Entity 0 proximity (handler 2): if flag 607 is set and flag 681 is clear: RemoveAllOfItemFromHero(item 33), SetGameState(681, 1) and line 14. Otherwise a line rotation: flags 102 and 104 both set give lines 8, 9, 10; else lines 11, 12, 13. Each line is SetConversationTarget(record), the line, EndConversation(record) (High).
- Handlers 1, 3 and 4 follow the 1405 pattern with flags 102 and 104: lines 0 to 3 for on-enter, lines 5 and 7 for the item asks (each item-ask handler reads the active hero first). Medium.
- Helper H4: the shared entity-control routine (SPEC-0043). The setup helper adds one record (flag 100, which Init itself sets, so in every case): entity 0, idle animation 0, interval 8 to 14 s, dwell 5 to 7 s, animations 40 and 96 (2.5 s) and 41 (1.3 s), no waypoints (High).
- Thread 5: SetToNormalCamera(15.0 s, 0.0, flag 1) and a slot load (set 0) from the stored slot.
- Update: exported; calls the shared entity-control routine every tick, ungated (High). Exit: exported, empty.
- Coverage: described.
- Confidence: High for the helper and Update; Medium for the handlers.

### 1417: Interior, an on-enter dialogue, a random line and a long entity-control helper

- Family: Interior.
- Init: AddBlackboardMessage(type 2, value 20, eWorlds 159). SlotAllocate(1); SlotCreate(1, table 0), with the return value stored; ControlThreadsAllocate(1); ControlThreadStart(thread 3, argument world) run in every case. Before them, when flag 102 is set, Init runs the registration helper (RegisterOnEnterCallback(handler 1, index 0) and RegisterEntityProximityCallback(entity 0, handler 2, flag 1)) and then the entity-control setup helper (a random check, Random with a bound read from the record, with GetNumberOfFramesPerSecond feeding LoopEntityAnim(entity 0, animation 0, 30.0 s, control 1)). When flag 102 is clear, neither runs (High for the gate and the registrations; Medium for the setup helper's content).
- On-enter (handler 1): flag 100 set gives LevelMsg(0) if flag 106 is set, otherwise LevelMsg(2).
- Entity 0 proximity (handler 2): when flag 606 is set and flag 680 is clear: RemoveAllOfItemFromHero(item 33), SetGameState(680, 1), SetConversationTarget(record), LevelMsg(10), EndConversation(record). Other branches give lines 4 and 7 (flags 100 and 106). Medium.
- Thread 3: when flag 102 is set, SlotLoad(the stored slot, set 0, true, true) and SetToNormalCamera(15.0 s, 0.0, flag 1).
- Helper H4: the shared entity-control routine (SPEC-0043; records of 440 bytes, so the fields at 0x0, 0x1b8 and 0x370 are the entity words of the first three record slots). The setup helper adds one record, only when flag 102 is set: entity 0, idle animation 0, interval 8 to 15 s, dwell 5 to 7 s, animations 40, 95 and 96 (2.5 s) and 41 (1.3 s), no waypoints (High).
- Update: exported; calls the shared entity-control routine every tick, ungated (High). Exit: exported, empty.
- Coverage: described.
- Confidence: High for the helper and Update; Medium for the handlers.

### 1419: Interior, an on-enter dialogue, a random line and a long entity-control helper

- Family: Interior.
- Init: SlotAllocate, SlotCreate of one set (table 0, return value stored), ControlThreadsAllocate(1) and ControlThreadStart(thread 3) run in every case. Before them, when flag 100 is set, Init runs the registration helper (RegisterOnEnterCallback(handler 1, index 0) and RegisterEntityProximityCallback(entity 0, handler 2, flag 1)) and then the entity-control setup helper (a random and frame-rate check feeding LoopEntityAnim(entity 0, animation 0, 30.0 s, control 1)). When flag 100 is clear, neither runs (High for the gate and the registrations; Medium for the setup helper's content).
- On-enter (handler 1): flag 102 set and flag 104 set gives LevelMsg(0); otherwise LevelMsg(2).
- Entity proximity (handler 2): when flag 608 is set and flag 682 is clear: RemoveAllOfItemFromHero(item 33), SetGameState(682, 1), LevelMsg(13) with the conversation and EndConversation. Flags 102 and 104 choose lines 4 and 7, and line 10 when the random value is 1.
- Thread 3: SlotLoad(the stored slot, set 0, true, true) and SetToNormalCamera(15.0 s, 0.0, flag 1) under flag 100.
- Helper H4: the shared entity-control routine (SPEC-0043). The setup helper adds one record, only when flag 100 is set: entity 0, idle animation 0, interval 8 to 14 s, dwell 5 to 7 s, animations 40 and 96 (2.5 s) and 41 (1.3 s), no waypoints (High).
- Update: exported; calls the shared entity-control routine every tick, ungated (High). Exit: exported, empty.
- Coverage: described.
- Confidence: High for the helper and Update; Medium for the handlers.

### 1425: Interior, a blackboard value on init, an on-enter dialogue and a long entity-control helper

- Family: Interior.
- Init: AddBlackboardMessage(type 2, value 50, eWorlds 160). SlotAllocate(1); SlotCreate(1, table 0), with the return value stored; ControlThreadsAllocate(1); ControlThreadStart(thread 3, argument world) run in every case. Before them, when flag 106 is set, Init runs the registration helper (RegisterOnEnterCallback(handler 1, index 0) and RegisterEntityProximityCallback(entity 0, handler 2, flag 1)) and then the entity-control setup helper (the random and frame-rate checks feeding LoopEntityAnim(entity 0, animation 0, 30.0 s, control 1)). When flag 106 is clear, neither runs (High for the gate and the registrations; Medium for the setup helper's content).
- On-enter (handler 1): flags 102 and 104 choose LevelMsg(0), LevelMsg(2).
- Entity 0 proximity (handler 2): when flag 610 is set and flag 684 is clear: RemoveAllOfItemFromHero(item 33), SetGameState(684, 1), SetConversationTarget, LevelMsg(10), EndConversation. Otherwise lines 4 and 7 by flags 102 and 104.
- Thread 3: SlotLoad(the stored slot, set 0, true, true) and SetToNormalCamera(15.0 s, 0.0, flag 1) under flag 106.
- Helper H4: the shared entity-control routine (SPEC-0043). The setup helper adds one record, only when flag 106 is set: entity 0, idle animation 0, interval 8 to 15 s, dwell 5 to 7 s, animations 40, 95 and 96 (2.5 s) and 41 (1.3 s), no waypoints (High).
- Update: exported; calls the shared entity-control routine every tick, ungated (High). Exit: exported, empty.
- Coverage: described.
- Confidence: High for the helper and Update; Medium for the handlers.

### 1432: Exterior, the final fight (hero swaps, fight clock, finale and timeout ending)

- Family: Exterior. This is the final fight with the Darkest Faerie. Its full behavior (rules, timings, level states, script-local flags, the fight clock and both endings) is specified in SPEC-0054, sections "Finale (script pack 1432)" and "Fight flow (script pack 1432)". This section lists the module's calls by routine and does not repeat those rules. Every routine below runs in play (High).
- Init: SetGameState(364, 1); SetGameState(358, 1). A test helper that always returns 1 gates the next calls, so they always run: ControlThreadsAllocate(2); ControlThreadStart(fight control thread, argument world); ControlThreadStart(thread B, argument world); RegisterScriptTrigger(handler 3, index 0); RegisterActivateWorldCallback(handler 4); RegisterScriptTrigger(handler 5, index 1). Then RegisterAIPlayerActivateCallback(handler 6) and (handler 7); StopEffect(3) and StopEffect(4) (High). Handler 6 admits every record ordinal. Handler 7 refuses ordinals 7, 6 and 5 (a three-entry list held in module data, never written) and admits all others. A world keeps one filter, so the second registration replaces the first and handler 7 is the one in effect: streaming never binds the three Minion Grunts by itself (SPEC-0054; High).
- Activate-world (handler 4): SetLevelState(10, true); ActivateObj(2, false), ActivateObj(0, false), ActivateObj(1, false); sets script-local flag 9 (SPEC-0054).
- Trigger 0 (handler 3) sets script-local flag 0 (the fight entry); trigger 1 (handler 5) sets flag 4 under the landed-window gate (SPEC-0054, "Trigger 1 and the finale gate").
- Effect helper: argument nonzero gives StartEffect(e, 1) for e = 18 to 25; zero gives StopEffect(e) for the same effects.
- Fight control thread, each tick, applies the rules of SPEC-0054 in order and runs at most one of these routines:
  - Entry sequence (movie 78): FadeScreenOut(wait 1); PlayFMV(78, true); a zero-length FadeScreenOut; MoveHero(locator 3, 1, 1, 1); SetToNormalCamera; MoveEntity(entity 0 to locator 2, 1, 1, 1); ActivateObj(2), ActivateObj(0), ActivateObj(1) and a zero-length fade; SwitchHero(0, 0, 0) with the effect helper started while IsHeroSwapInProgress() is non-zero (Sleep(1) per poll); FadeScreenIn(wait 1); RestorePlayerControl(-1); SetHeroEquippedState(true); the fight clock set to 180.0 s and made visible (ActivateTimer); SetHeroSwapMode(3); StartTimer(); SetGameState(364, 1); SetLevelState(11, true); SetLevelState(10, false).
  - Fight start step: LetterBoxOn(0 s, wait 0); PlayCamera(4, wait 1, camera wait 1, 1.0 s); BossFightTweakHero(encounter 13, stage 0); FadeScreenIn(0.5 s, wait 0); StartEffect(3, false); StartEffect(4, false); Wait(0.5 s); MoveHero(locator 1, flag 1, control 1, extra 1); LetterBoxOff(0.7 s, wait 0); Wait(1.0 s); SetToNormalCamera(10.0 s, 3.14, flag 1).
  - Director: the landed-window counter (GetNumberOfFramesPerSecond), SetLevelState(16, false) and SetLevelState(13, true) when it runs out; on swap completion SetLevelState(12, false) and SetLevelState(14, true), or SetLevelState(13, false) and SetLevelState(15, true), then StartTimer(); the timeout test on GetTimer(); the stun swap (StopTimer(), SetHeroSwapMode(2), SwitchHero(1, 1, 1), SetHeroSwapMode(3)) and the landed-miss swap (StopTimer(), SetHeroSwapMode(2), SwitchHero(0, 1, 1), SetHeroSwapMode(3)). The entity activation helper of the stun-swap completion activates entities 7, 6 and 5.
  - Finale (movie 85): DeactivateTimer(); DeactivateObj(1); SetHeroSwapMode(2); DeactivateEntity(0), (5), (6), (7); BossFightTweakHero(encounter 13, stage 1); SetReturnWorldForFMV(eWorlds 151 (pack 1300)); AddBlackboardMessage(type 2, value 80, eWorlds 151); PlayFMV(85, true); SetGameState(94, 1) and SetGameState(365, 1). The world change to pack 1300 follows the movie.
  - Timeout ending (movie 80): DeactivateTimer(); FadeScreenOut(wait 1); DeactivateEntity(0), (5), (6), (7); SetHeroSwapMode(2); DeactivateObj(0), (1), (2); PlayFMV(80, true); FadeScreenOut(wait 1); SetGameOver(1) (SPEC-0055); then Sleep(1) for good.
- Thread B: this is control thread B of SPEC-0054. It brings in Minion Grunts 7, 6 and 5, in that order, at locator 4. Each tick, while level state 11 is set, level state 12 and script-local flag 6 are clear, and Roberta (hero 1) is active, it takes each entry in turn: when the entry's marker is set it first requires IsAI_Dead(entity); then Wait(8.0 s), a re-check that level state 12 is still clear and the same hero is active, MoveEntity(entity to locator 4, flag 0, control 1, fifth argument 1) and ActivateEntity(entity), and the marker is set (SPEC-0054, "Control thread B"; High).
- Update and exit: none.
- Coverage: fully described here and in SPEC-0054.
- Confidence: High for the calls and their routines; the rules and their confidence are in SPEC-0054.

## Edge cases and error handling

- A teleport or trigger registration with index -1 is ignored (SPEC-0030, the RegisterScriptTrigger test vector for index -1; no module in this spec registers -1).
- Repeated registration on the same index: 1344 registers index 3 and 4 once each; 1257 registers index 1 to 16 once each. Where a module registers the same index twice (none in this spec with two different callbacks), the later one is assumed to replace the earlier one (Medium, as SPEC-0032).
- Callbacks that test a state of 1 and callbacks that test non-zero: both kinds appear (for example trigger 14 in 790 tests non-zero state; most triggers test state 1). The clean side must use the same test per callback as written in the module descriptions.
- The thread argument -1 (732, 1172) is passed as written. Thread arguments 0 and 1 are used as flags by the thread body in 790. In 1131, 1133 and 1136 the argument of the dialogue thread is the entity it talks to (entity 0, or entity 1 or 0 by flag 10).
- Slot ids: on the paths found, every SlotLoad and SlotReset in this spec passes the slot number that the module's SlotCreate stored on the same path (SPEC-0032 Slot rules). The exception is module 1344 (sets 0 to 3, see that section), which can pass the file value -1. A slot of -1 would do nothing in the original (the call path does not remap -1; High).
- Dialogue lines that depend on module data words: the line rotation (Conventions) is fully listed for 706, 732, 753, 880, 1131, 1133, 1136, 1139, 1148, 1230 and 1407. In 1140, 1143 and 1172 the chooser lines given are the first line of each mode; the rotation partners are not listed.
- Random values, GetNeopoints results and several entity or hero ids are read from fields of the record or from other calls; they are not fixed values (see Open questions).
- Dormant routines (nothing references them, so they never run and the clean side omits them): in 1139, a routine that would register a script trigger for index 0, and a fade routine; in 886, a trigger body that would set flag 251 after a camera, slot and entity-walk scene; in 1230, a thread body that would set flag 530 when flag 529 is set and the hero holds at least 5 of item 78; in 1405, 1407, 1417, 1419 and 1425, a small routine that returns an entity's animation to idle. Every other routine of the 30 modules runs in play, including all of 1244, 880, 1248 and 1432 (High).
- Callback bodies: every registered callback body described above runs in play whenever its registration is made and its gate holds; tests on callback record fields (state, index, object) are ordinary gates (High).
- Debug lines (DebugPrintf) have no effect on game state. IsDebugMode is called once (in 1139) and its result is not tested by a branch.

## Test vectors

- 757, trigger index 7 entered with object present: SetAI_Agro(entity 5).
- 757, trigger index 11 (not registered): would give SetAI_Agro(entity 9); no clean-side behavior is required.
- 706, flag 64 set and flag 67 clear, thread 2 starts: SlotLoad(the stored slot, set 2, true, true).
- 706, any of flags 62 to 67 set on arrival: the helper returns that flag's number (not -1), so SetGameState(62, 1) is not called; the slot setup, the triggers and the thread start still run. With none of them set: SetGameState(62, 1).
- 984, on-enter with flag 202 set and flag 67 clear: LevelMsg(2).
- 1425, init with flag 106 clear: the slot setup and the thread start run; no on-enter or proximity callback is registered. With flag 106 set: both registrations are made, then the entity-control setup runs.
- 1407, init on any visit: flag 100 is set first, so the four registrations are always made.
- 880, entity 3 proximity, hero owns maps 1, 8, 30 and 27 but not 18, accepts, has 450 Neopoints: GiveItemToHero(175, 18, false), RemoveNeopoints(400), line 14.
- 1133, entity proximity with only flag 138 set and the hero holding items 40, 42 and 43 (three of four): line 35, no item change.
- 1139, entity 0 proximity with flags 319 and 305 clear, flag 678 set and item 23 held: item 23 removed, return world eWorlds 109 set, movie 49 plays; no line.
- 753, activate-world with blackboard value 80 for eWorlds 31 and flag 660 clear: MoveHero(locator 0, 1, 1, 1), Sleep(30), LevelMsg(34), LevelMsg(35), SetGameState(660, 1), SetToNormalCamera, then the value is reset to -1.
- Line rotation (706, entity 0, flag 63 set on three talks in a row, after a talk with flag 63 clear): lines 7, 8, 9; a fourth talk shows line 7 again.
- 793, entity 1 proximity with flag 417 clear, flag 416 set: LevelMsg(3).
- 874, entity 6 proximity: SetGameState(291, 1) at the fade-out step.
- 1344, trigger 0 entered with flag 237 set, flag 238 clear: SetGameState(238, 1), PlayFMV(22), Sleep(30 ticks), LevelMsg(10).
- 1139, activate-world with blackboard value 20: MoveHero(locator 0, 1, 1, 1). With value 10: MoveHero(locator 1, 1, 1, 1).
- 1257, activate-world with blackboard value 30: MoveHero(locator 1, 1, 1, 1).
- 1246, King activation: hero moved to locator 11 with the bound set {tentacle 15, Basher 21, tentacle 14, tentacle 13, Basher 20, Basher 17}: the first streaming pass binds the King (record 16) in place of tentacle 13; the intro's MoveEntity(16, locator 7) then finds it. With the cap raised to 12 by SetMaxActiveAI(12), the twelve nearest records bind: Bashers 21, 17, 20, 18, 22, 19, 23 and 24, the King and tentacles 15, 14 and 13.
- 732, the hero's body enters trigger index 1: AddBlackboardMessage(2, 60, 22), FadeScreenOut(0.5 s, black, WAIT 1), MoveHero(locator 0, 1, 1, 1); no prompt. Index 0 gives value 70 and index 2 gives 40.
- 790, after load the hero enters trigger 8: ControlThreadStart(thread 6, 1), SlotReset then SlotLoad(set 0). Entering trigger 8 again: debug text only. Then entering trigger 17 (or 16): thread 6 with 0, SlotLoad(set 1).
- 790, trigger 1 entered with a hero shot (shot word 1): nothing. Entered by the body, still inside after PlayAnim(3): PlayAnim(0), camera 5, latch set; a second entry later does nothing.

## Open questions

Default if unresolved (a rewrite choice, not a recovered fact), module 1246: bind the King on the first streaming pass after the entrance script places the hero at locator 11, and treat movie 41 as pausing the world update. The exact tick order of the movie, the first trigger run and the streaming pass is Medium.

1. Resolved: the WAIT, CONTROL and CAMERA_WAIT values and the hero selector are in SPEC-0030 (Layout 2.1, Open question 5). The value -1 in 757 and 790 is the active (selected) hero, not "all heroes".
2. Resolved for the first main-story entry to module 1344: pack 1335 sets flag 92 before this transition (SPEC-0058, steps 24–25). Init therefore creates and stores the three-set slot; thread 7 checks flags 94, 93 and then 92 before its final 239/240 branch, so the established entry loads set 2 from the slot created on that same Init. Still open: when flag 92 is clear but flag 94 or 93 is set, thread 7 selects set 0 or 1, whose load reads the slot word stored only by the flag-92-set Init path; if no earlier Init populated it, that word remains -1. This state is outside the documented first-entry route, and reachability elsewhere is not established. SlotCreate returns its reserved slot number or -1 if no slot is free, and the call path does not remap -1 (SPEC-0032 Slot rules; High).
3. The fifth argument of SlotLoad (activate) is true in every resolved call of this spec; the rule of SPEC-0032 is confirmed.
4. Resolved for module 874: all of its fade calls use black (RGB zero). Fade colours for modules 1244 and 1257 remain undetermined.
5. Resolved: the record-backed animation-choice call in the shared entity-control helper is Random(0, animation count - 1) (SPEC-0043, shared entity-control routine). The records in this spec give the exact counts: 3 animations in 1405, 1407 and 1419 (arguments 0, 2), and 4 in 1417 and 1425 (arguments 0, 3). Their separate randomized animation and dwell delays use the per-record intervals listed in each module section and SPEC-0043. Pack 1432 has no Random(int, int) level-script host call in its module listing; its boss-class random attack ranges are in SPEC-0054.
6. Resolved: GetNeopoints reads the shared Neopoints wallet used by both heroes and returns 0 when no wallet exists. RemoveNeopoints subtracts a positive amount from that shared wallet, flooring the balance at 0; nonpositive amounts do nothing (SPEC-0030, rows 192–193; SPEC-0021, Wallet).
7. Resolved: helper H4 of 1405, 1407, 1417, 1419 and 1425 is the shared entity-control routine of SPEC-0043, which gives its rules; the records are in each section. There are no path ids: every waypoint walk is a walk to a locator, and these five modules add no waypoints (High).
8. Resolved for 790: +0x20 is the shot word and +0x24 the attack word (SPEC-0030 section 3); +0x30 is the record's owner value, -1 for a plain registration and the script-group entry for an instance registration (High). Still open: the entity-helper fields (+0x0 entity id, +0x4, +0x88, +0x5c, +0x1130).
9. Resolved: the level-state cascade of 1246's trigger 0 (section 1246). Also resolved: the entity-control helper of 1405, 1407, 1417, 1419 and 1425 (question 7); 706, 732, 753, 880, 1131, 1133, 1139, 1148, 1230, 1244, 1248 and 1432 are fully described (1432 together with SPEC-0054).
10. Resolved: module 732 registers handler 1 under trigger indices 0, 1 and 2 (the trigger list of its script group 2), and the handler runs the fade and move on every contact without a button press (High).
11. Resolved: the eWorlds ids named in this spec map to these world packs: 21 to 732, 22 to 735, 31 to 758, 76 to 984, 81 to 991, 90 to 1023, 109 to 1129, 114 to 1139, 138 to 1247, 139 to 1248, 140 to 1249, 147 to 1257, 151 to 1300, 159 to 1358, 160 to 1363 (SPEC-0058 pack numbers).
12. Resolved: the ids used in this spec are named by the published name rules. Items: SPEC-0021 "Item names". eWorlds ids: SPEC-0006 "Area names" and SPEC-0051 "Map area names" (packs as in question 11). Flags that are a quest start or done flag: SPEC-0049 "Titles and flag glosses". Other flags have no name source on disc and stay numbers (High).
13. Resolved locally: module 1248's Exit sets flag 79 when objects 0 and 2 are inactive and object 1 is active. The same module's Init reads flag 79 and, when set, skips its initial activation calls for objects 3, 4, 5, 2, 0 and 1. Thus the established consumer is module 1248's own initialization behavior on a later Init. Whether another host or module also reads flag 79, and whether any broader route depends on it, is not established.
14. Resolved: resource 1421 is documented in SPEC-0041, section 1421 (Interior, entity proximity conversation room). That section gives its flag-102 registration gate, callbacks and thread behavior; no additional behavior is specified here.
15. Resolved: module 732's activate-world switch is fully described (values 10, 20, 30, 40 and 60; any other value selects nothing).
16. Resolved: module 753 always starts the slot thread; the event thread and trigger 1 start only when flag 256 is set and flag 258 is clear at Init.
17. Resolved (SPEC-0030, Module lifetime; section "Load and unload plumbing"): modules 1139, 880, 1230 and 1248 are exteriors, so they follow the exterior rule. Their line-rotation counters, the one-time sale of 1139, the talked latches of 880 and 1230 and the shown latch of 1248 keep their values while the pack stays in the exterior buffer (interior visits, swaps with the transition, streaming steps that find the pack resident, a movie with no return world) and return to their file values when the pack is next loaded (streaming to a different exterior, any driver load of the exterior buffer, including a replacement load, the after-movie reload and a save load, or a new game) (High).
18. Resolved: module 1246 level states. The Werelupe King sets level states 4 and 6 (phases 2 and 3) and its death sets flag 273; trigger 0 sets level states 2, 3, 5, 7 and 8. Level state 1 has no setter in that world, so the Update path does not run; the hero's contact with the arena sphere (trigger 0) drives the cascade, and after the death the next contact gives movie 42 and item 14 (section 1246).
19. Resolved for (1) to (4). (1) Flag 496 (the done flag of the second Bounty Hunt row; SPEC-0049) has no writer on the disc. The progress-flag bitmap has eleven writer routines (SPEC-0049, Who writes the flags), and none can set 496: the stage table (674 rows) has no row for 496; the start-up default table (743 words, all zero on disc) only clears flags; the one dialogue set code writes flag 416 alone; the engine setters write 707, 273 and 107 to 110; save load copies the bitmap; and the indirect setter (the six-entry pointer run) has no reference in the boot program or any level module. Every computed SetGameState argument in the 173 level modules resolves to a fixed set that excludes 496 (777: 252 to 256; 956: 208, 209, 210, 623 and 64; 1078: 669 to 674; 1171: 637 to 641 and 163 to 167; 1023: 128 to 135; 934: 470 to 474 by fixed id). The proximity callback of pack 934 runs a talk and sets no flag from its record. Consequence: the set-0 load (Korvin, region 1149) never runs in play, so the shop talk that needs entity 3 has no spawn route from pack 1148, and item 42 has no other placement on disc (Medium-High; the census is closed by the writer list, and the entity 3 spawn is not confirmed in play). (2) Resolved: items 106 (wooden shield, pack 1131, price 400) and 99 (Squire's Sword, pack 1133, price 500) are flag-1 shop placements, so both are bought through the shop of SPEC-0050 (the price is the placement's fixed count; High for the rule, Medium-High for these two prices). (3) Resolved: pack 1133's thread 2 runs once per entry and shows the first line of its pair (lines 0, 2, 4, 6, 8, 10 or 12 by the first set flag). The second line of each pair is not reached in play (see the 1133 section). (4) Resolved: the villager templates of 1023 and 1150 draw Random(1, 6) on every talk, before any flag test, and Random(1, 3) only when flag 480 is set. With no arena win, one Random(1, 6) draw is still taken; the random stream therefore advances on every such talk (High; SPEC-0045, arena-talk test).
20. Resolved: **what activates the King in 1246 (R-0064).** Character streaming binds record 16 (no filter, no activation call); the module only holds and releases it (section 1246, What makes the King active; SPEC-0019, Activation in play).

## Confidence

| Section | Confidence |
| --- | --- |
| Module list, families and Exit presence | High |
| Host call names and argument order | High |
| Numeric arguments (flags, lines, items, animations, speeds, durations) | High where stated; Medium where a branch nests |
| Slot stores and the stored-slot rule (every slot call in this spec) | High for the store of each result and for no remap of -1; High for 1344 (both slot words) |
| Branch conditions in words | High for the modules marked fully described; Medium elsewhere |
| Entity-control helper (1405, 1407, 1417, 1419, 1425) | High (the shared routine of SPEC-0043) |
| Modules listed as partly described | Medium or Low, per module |
| Slot tables (values) | High (taken from module data) |
| Switch-selected branches (module 732) | High |
