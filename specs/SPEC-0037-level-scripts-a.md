# SPEC-0037: Level scripts (set A)

- Answers: R-0061, proactive (level script behavior; extends SPEC-0030 and SPEC-0032), R-0192 (1251 thread A item activation), R-0225 (1427 requests and the resident packs 1429 to 1431)
- Status: Draft
- Disc build: SCUS-97367 v1.00 (same fingerprint as SPEC-0001)
- Confidence: per module (see the Confidence line in each section and the table below). Host call names, argument values and the order of calls: High unless marked. Slot ids: High for the stored-slot rule (the slot a load uses is the SlotCreate result that the module stores); the call path does not remap -1 (High); a load through a slot word that no code stores does nothing (High for 837; the slot word of 906 is stored by Init, so its load uses the stored slot). Branch conditions: Medium. Values not established: marked unknown and listed under Open questions.

## Summary

This spec describes the level script modules of 30 world packs: 1225, 1369, 1427, 908, 1299, 809, 1391, 1253, 1259, 796, 788, 799, 787, 789, 947, 1220, 783, 784, 837, 1238, 857, 776, 858, 906, 786, 1251, 855, 1152, 1429 and 856. The modules use the host API catalogue of SPEC-0030 and the trigger, thread and slot rules of SPEC-0030 and SPEC-0032. Each section lists the family, the init steps, every registered trigger, proximity, teleport and callback with its index and its actions, the control threads, the update and exit roles, and the edge cases.

Times are seconds unless marked as ticks. One tick is 1/60 s (SPEC-0007). Argument conventions follow SPEC-0032: WAIT 0 returns at once, WAIT 1 waits until the action finishes; CONTROL and CAMERA_WAIT values are passed through as the same integers the module passes.

Mapping of notation used below:
- "flag N" is game flag number N, read with the query call and set with the set call. A flag tested "set" is non-zero; "clear" is zero.
- "item N" is a world item id. "held" means the hero's item count is above zero unless stated otherwise.
- "object N" is an animated object id, "locator N" a locator id, "entity N" an entity id, "line N" a dialogue line id of the world's text, "set N" a slot character set (see SPEC-0030 section 5), "track N" a camera track id, "stream N" a sound stream id, "sound N" a sound id.
- "eW N" is a world identifier (area slot) passed to a world call. The pack and area name of each identifier used here are in Open question 4 (SPEC-0006 "Area names").
- "fade black" is a colour of zero values (the colour buffer is cleared before the call).
- "the trigger record" is the callback record of SPEC-0030 section 3: state word at +0x10, trigger index at +0x14, object pointer at +0x1C, shot word at +0x20 (1 when a hero shot fired the trigger) and attack word at +0x24 (1 when the selected hero's body fired it during a hit-window frame). Modules 784 and 799 test the last two: a test that both are zero ignores hero shots and attacking bodies (Open question 2).
- Calls shown as "Wait(T)" use seconds; "Sleep(N)" uses ticks.
- "Dormant" means a routine that is present in the module but is not reached from init or from a registered callback. Its behavior is recorded for completeness; per-level notes must not run it unless stated. "Reachable" means statically reachable in the module's code from init, update, exit, the load and unload hooks, or a registered callback; whether play reaches a statically reachable routine depends on the flag gates stated in its section.
- "The stored slot" means the slot number that the module's SlotCreate call returned and the module kept in a cell. SlotCreate returns the slot number of the buffer it reserves, in creation order, or -1 when no buffer is left (SPEC-0032 Slot rules). The slot word of a load is read from module data. In the modules where Init stores the SlotCreate result into that word, the load uses the stored slot. A SlotLoad or SlotReset through a slot word that no code stores does nothing; in this spec that is the 837 chooser (file value -1). The call path does not remap the value (Open question 1).

## Where it lives on the disc

- Each module is the level-script chunk (type 37, "SNR2") of the world pack with the same resource number, as in SPEC-0030 and SPEC-0032.
- The resource numbers are the world-pack numbers in this spec. Each module's family is the one whose Init name it exports.
- Several modules reference data tables inside the module that the clean side reads from the disc at run time, for example a slot-set table passed to SlotCreate, a nav-mesh table and a per-index callback table. Their contents are not copied here; the table is named in each section. Table contents are not described in this spec unless stated.

## Family and exit summary

| Module | Family | Exit exported | Triggers and callbacks registered from Init | Control threads |
| --- | --- | --- | --- | --- |
| 1225 | Transition | yes (empty) | none | 1 |
| 1369 | Exterior | yes (empty) | 2 triggers (index 0, 1) | none |
| 1427 | Transition | yes (empty) | 8 triggers (index 0 to 7) | none |
| 908 | Transition | yes (empty) | world-swap callback | none |
| 1299 | Exterior | yes (empty) | 2 teleport, 1 activate-world, 1 trigger | none |
| 809 | Transition | yes (empty) | none | 1 (thread D, started by Init) |
| 1391 | Transition | yes (empty) | 3 triggers (index 0, 1, 2) | none |
| 1253 | Exterior | yes (empty) | 1 trigger, 1 activate-world | none |
| 1259 | Exterior | yes (empty) | 50 triggers (index 0 to 49) | none |
| 796 | Exterior | yes (empty) | 2 triggers (index 0, 1) | none |
| 788 | Interior | no | 1 proximity, 1 teleport, 1 activate-world | dormant only |
| 799 | Transition | yes (empty) | 3 triggers (index 0, 1, 3) | none |
| 787 | Interior | yes (empty) | 1 proximity, 1 teleport, 1 activate-world | dormant only |
| 789 | Interior | no | 1 proximity, 1 teleport, 1 activate-world | none |
| 947 | Exterior | yes (empty) | 1 teleport, 3 triggers (index 0, 1, 2) | 2 started |
| 1220 | Transition | yes (empty) | 1 trigger, 1 activate-world | 1 |
| 783 | Interior | no | 1 proximity, 1 teleport, 1 activate-world | dormant only |
| 784 | Interior | no | 1 proximity, 1 trigger | 1 started (conditional) |
| 837 | Transition | yes (empty) | none | none |
| 1238 | Exterior | yes (empty) | 1 pickup callback | 1 or 2 |
| 857 | Interior | yes (empty) | 1 proximity | 2 |
| 776 | Interior | no | 1 proximity, 1 teleport, 1 activate-world | dormant only |
| 858 | Interior | yes (empty) | 1 proximity | 2 |
| 906 | Exterior | yes (empty) | 3 script triggers, 1 activate-world, 1 pickup callback (all from Init) | 1 (started from Init) |
| 786 | Interior | yes (empty) | 1 proximity, 1 teleport, 1 activate-world | none |
| 1251 | Interior | yes (empty) | 1 proximity, 1 on-enter, 1 item-ask | 1 |
| 855 | Interior | yes (empty) | 1 proximity | 2 |
| 1152 | Exterior | no | 1 proximity, 2 teleport, 1 activate-world | 1 |
| 1429 | Interior | yes (empty) | 1 proximity, 1 teleport, 1 buy-item | 2 |
| 856 | Interior | yes (empty) | 1 proximity, 3 item-ask, 1 buy-item | 2 |

Exit is missing in 788, 789, 783, 784, 776 (Interior) and 1152 (Exterior). Update makes no host call in most modules. The exceptions are 906 (per-tick character navigation, see its section), 1427 (clears its internal once-flag each tick, see its section) and 837 (watches breakable 2 and, once the hero has broken it, runs its break sequence and sets flag 636, see its section).

## Modules

### 1225: Transition, slot-set ladder on flags

- Family: Transition.
- Init: SlotAllocate(W, 1). SlotCreate(W, 4 entries, slot table in module data); its return value is stored for the loads below. ControlThreadsAllocate(W, 1). ControlThreadStart(W, thread A, W).
- Thread A (runs once, started by Init): tests flag 84, then 83, then 82, then 81. The first set flag selects a load: flag 84 -> SlotLoad(W, the stored slot, set 3, true, true); flag 83 -> set 2; flag 82 -> set 1; flag 81 -> set 0. Each load is followed by a diagnostic line (text not recorded). If none is set, nothing loads.
- Triggers, proximity, teleport: none.
- Update: empty. Exit: empty.
- Edge cases: the slot argument is the stored SlotCreate result in every load of this module, so the loads use a real slot. The fifth argument (activate) is true.
- Confidence: High for the calls and their order and for the slot store; the meaning of the slot number is the SlotCreate numbering (SPEC-0032 Slot rules).

### 1369: Exterior, two triggers with a button-gated walk

- Family: Exterior.
- Init: InitObjOn(W, object 0, false); InitObjOn(W, object 1, false); AddBlackboardMessage(type 2, value 10, eW 157); RegisterScriptTrigger(W, T1, index 1); RegisterScriptTrigger(W, T0, index 0).
- Trigger index 1 (T1): if the trigger record's object pointer is non-null, StartExteriorReplacementLoad(eW 157, fade -1, no colour). This trigger has no state test and no once-word: the only test is the object pointer (High).
- Trigger index 0 (T0): if the object pointer is non-null and the module's once-word (file value 0) is still 0:
  - If the confirm button (host button 0) is not pressed: DisplaySquareButtonContext(context 4).
  - If pressed: the once-word is set to 1 first, so the sequence plays once per module load and the prompt is not shown again; then WalkHeroAndOrient(locator 1, WAIT 1, CONTROL 1); PlayHeroAnim(animation 140, WAIT 0, CONTROL 1); PlayAnim(object 1, WAIT 1, 1.0 s); PlayCamera(track 4, WAIT 0, CAMERA_WAIT 1, 1.0 s); PlayAnim(object 0, WAIT 1, 1.0 s); Wait(1.0 s); ReturnToNormalCamera(-1, true).
- Threads: none. Update: empty. Exit: empty.
- Edge cases: trigger index 1 acts on any engine call that has an object pointer. The engine gives a non-null object pointer only on the selected hero's own trigger-hit path (SPEC-0028, section 13; SPEC-0051), so the load starts only when the hero's body touches the volume, and it can be requested again on every tick the hero stays inside (High).
- Confidence: High for the calls; Medium for the trigger state conditions.

### 1427: Transition, eight triggers that set the world load override once per tick

- Family: Transition.
- Init: stores the world handle. Then registers eight triggers, index 0 to 7, in a loop. Each index gets a callback from a per-index callback table in module data. The callbacks, by index:
  - index 0: callback E0, which requests world override 187.
  - index 1 and 2: callback E1, world override 188.
  - index 3 and 4: callback E2, world override 189.
  - index 5 and 6: callback E3, world override 190.
  - index 7: callback E4, world override 191.
- Each callback calls the shared helper H with its id. H: if the module's once-flag is clear, SetWorldLoadOverRide(id), then set the once-flag. If it is set, nothing happens.
- Triggers do not test the trigger state or the object pointer; the callbacks run for any engine call of the trigger.
- Update: clears the once-flag (so the first trigger callback in each tick sets the override). Exit: empty.
- Edge cases: the first callback in a tick wins; later callbacks in the same tick do nothing. The override ids 187 to 191 are area slots 187 to 191, packs 1428 to 1432 (Open question 4).
- What a request does: the pending slot makes the pending step stream the named pack into the exterior buffer. 1427 stays current in T, nothing swaps, and the hero stays in 1427. 1427 has no door, no entry record and no character block. Its cells hold only the values 1 and 2, which name 1428 and 1429 through its neighbour list, so when no trigger fires the cell under the hero asks for 1428 or 1429 (SPEC-0006, Area numbering). Packs 1428 and 1432 lie in the stairwell's coordinate frame; 1429, 1430 and 1431 are copies of a shop interior around the world origin, far from the stairwell, and the hero can never stand in them. They are resident beside 1427 and run no module (1429's section; SPEC-0006, Edge cases, Interior-kind packs in E).
- Confidence: High (confirmed against the module code).

### 908: Transition, world-swap with a camera and a hero walk

- Family: Transition.
- Init: RegisterScriptWorldSwap(W, W1). InitObjOn(object 4, false); InitObjOn(object 0, false); InitObjOn(object 1, false); InitObjOn(object 2, false).
- World-swap W1 (first word of the record is an event code e):
  - Diagnostic line, then one of two paths.
  - If e is 906: PlayCamera(track 5, WAIT 1, CAMERA_WAIT 1, 1.0 s); MoveHero(locator 0, true, CONTROL 1).
  - Otherwise: PlayCamera(track 4, WAIT 1, CAMERA_WAIT 1, 1.0 s); MoveHero(locator 1, true, CONTROL 1).
  - Then shared reset (both paths): SetObjToTime(object 0, 0.0 s); SetObjToTime(object 1, 0.0 s); SetObjToTime(object 2, 0.0 s); SetObjToTime(object 4, 0.0 s); PlayAnim(object 0, WAIT 0, 1.0 s); PlayAnim(object 1, WAIT 0, 1.0 s); PlayAnim(object 2, WAIT 0, 1.0 s); PlayAnim(object 4, WAIT 1, 1.0 s).
  - Then: if e is 906, WalkHero(locator 1, WAIT 1, CONTROL 1); otherwise WalkHero(locator 0, WAIT 1, CONTROL 1). Then ReturnToNormalCamera(-1, true).
- Threads: none. Update: empty. Exit: empty.
- Edge cases: the event code 906 selects the first path; the events themselves are not named (Open).
- Confidence: High for the calls and order; Medium for the event meaning.

### 1299: Exterior, two teleports, a world-activate hook and a sting trigger

- Family: Exterior.
- Init: RegisterTeleportCallback(index 1, T1, context kind 13, W); RegisterTeleportCallback(index 0, T0, context kind 13, W); RegisterActivateWorldCallback(A, W); RegisterScriptTrigger(W, S0, index 0).
- Teleport index 0 (T0): if flag 248 is set: AddBlackboardMessage(type 2, value 80, eW 154); StartExteriorReplacementLoad(eW 154, fade -1, no colour). If flag 248 is clear: LevelMsg(line 0, W, true, WAIT 1).
- Teleport index 1 (T1): if flag 248 is set: AddBlackboardMessage(type 2, value 80, eW 155); StartExteriorReplacementLoad(eW 155, -1, no colour). If flag 248 is clear: SetGameState(flag 243, true); AddBlackboardMessage(type 2, value 80, eW 154); StartExteriorReplacementLoad(eW 154, -1, no colour).
- Activate-world A: diagnostic line; v = GetBlackboardMessageValue(type 2, eW 150); AddBlackboardMessage(type 2, value -1, eW 150); if v is not -1: SetToNormalCamera(10.0, 0.0, true).
- Trigger index 0 (S0): if object pointer non-null and state is 1: PlayStingStream(stream 18, 1.0).
- Threads: none. Update: empty. Exit: empty.
- Confidence: High for the calls; Medium for the teleport branches (flag meanings open).

### 809: Transition, slot chooser thread and debug-mode flag writes

- Family: Transition.
- Init: tests the debug-mode query (SPEC-0030: returns false in play). Only when the query returns nonzero does Init first set flags 54, 55, 56 and 57 to true (SetGameState, in that order). These four flag writes are the only host calls in this module that the query controls. In play and in debug mode alike, Init then runs SlotAllocate(W, 1); SlotCreate(W, 5 entries, slot table in module data), storing its return value in the module's slot word; ControlThreadsAllocate(W, 1); ControlThreadStart(W, thread D, W).
- Thread D (runs once, started by Init in play): tests flag 58, 57, 56, 55, 54 in that order. The first set flag loads: 58 -> SlotLoad(the stored slot, set 4, true, true); 57 -> set 3; 56 -> set 2; 55 -> set 1; 54 -> set 0. Each load has a diagnostic line. The slot is the stored SlotCreate result, read from the slot word that Init stores, and the activate argument is true.
- Triggers: none. Update: empty. Exit: empty.
- Edge cases: thread D starts in play, so a character set loads whenever one of flags 54 to 58 is set in the game state. The debug flag writes do not run in play, so flags 54 to 57 are set in play only when the game state already holds them.
- Confidence: High (the debug query is the SPEC-0030 query that returns false; the store of the slot result by Init is confirmed against the module code).

### 1391: Transition, three triggers with a camera shake and an animation poll

- Family: Transition.
- Init: InitObjOn(W, object 0, false); RegisterScriptTrigger(W, S2, index 2); RegisterScriptTrigger(W, S0, index 0); RegisterScriptTrigger(W, S1, index 1); AddAnimObjSound(W, object 0, sound 6, 0.0 s, 1.0 s); diagnostic line.
- Trigger index 2 (S2): if object non-null, the module's once-word (file value 0) is 0, and state is 1; the once-word is set to 1 before step 1, so the sequence plays once per module load:
  1. PlayLookAtPlayerCamera(track 4, WAIT 0, CAMERA_WAIT 0, 1.0 s).
  2. StartCameraShake with seven values: 5.5, 0.3, 25, 0.5, 0.8, 1.5708, 0.2.
  3. Wait(1.5 s).
  4. PlayAnim(object 0, WAIT 0, 1.0 s).
  5. Wait(4.0 s).
  6. SetToNormalCamera(10.0, 3.14159, true).
  7. GrabHeroControl().
  8. Poll: while IsObjAnimFinished(object 0) is false, Sleep(1 tick).
  9. ReturnHeroAnimToIdle().
- Trigger index 0 (S0): no actions recorded.
- Trigger index 1 (S1): if object non-null: SetWorldLoadOverRide(165).
- Threads: none. Update: empty. Exit: empty.
- Edge cases: S0 does nothing; S1 has no state test.
- Confidence: High for the calls and values; Medium for the poll structure.

### 1253: Exterior, a teleport-like trigger and a blackboard-driven arrival

- Family: Exterior.
- Init: RegisterActivateWorldCallback(A, W); RegisterScriptTrigger(W, S0, index 0).
- Trigger index 0 (S0): if object non-null and state is 1: AddBlackboardMessage(type 2, value 40, eW 141); StartExteriorReplacementLoad(eW 141, -1, no colour).
- Activate-world A: diagnostic line; v = GetBlackboardMessageValue(type 2, eW 143); diagnostic line. Then:
  - Every value runs the same steps with a locator L and a camera track K: MoveHero(locator L, false, CONTROL 1); FadeScreenIn(fade black, 0.5 s, WAIT 0); StartHeroLongFall(); PlayTrackHeroCamera(track K, WAIT 1, CAMERA_WAIT 1, 1.0 s); Wait(1.5 s); ReturnToNormalCamera(-1, true); AddBlackboardMessage(type 2, value -1, eW 143).
  - v is 20: L = 1, K = 6. v is 30: L = 2, K = 7. v is 10: L = 0, K = 5. Any other value (including -1, no message): L = 0, K = 5, the same as 10 (only the diagnostic line differs).
  - So every arrival in this world, even one with no blackboard message, ends with the long fall from a locator.
- Threads: none. Update: empty. Exit: empty.
- Confidence: High.

### 1259: Exterior, a shared trigger that reloads a slot set by index

- Family: Exterior.
- Init: stores the world handle in a module cell, then SlotAllocate(W, 1); SlotCreate(W, 50 entries, slot table in module data), its return value stored in the slot word. Then RegisterScriptTrigger(W, S, index i) once for each entry of that slot table, in table order: 50 registrations. The table entries are 0 to 49 in order, so the registered indices are 0 to 49. The same routine S serves all of them.
- Trigger routine S: if the object pointer is non-null and the trigger record's index (+0x14) is not -1: SlotReset(W, the stored slot); SlotLoad(W, the stored slot, set = the trigger record's index, true, true).
- No dormant routine: the one block that looks like an unreached routine starts at the slot table that Init passes to SlotCreate, so it is that table, not code (High: it holds the entries 0 to 49 described above).
- Threads: none. Update: empty. Exit: empty.
- Edge cases: the world handle used by the calls is read from the cell that Init stores before its SlotAllocate call (the world argument of Init). The slot word is read by the trigger routine after Init has stored it.
- Confidence: High for the trigger calls, the 50 registrations and the index sequence (confirmed against the module code).

### 796: Exterior, a button-gated sequence and a fade-and-load trigger

- Family: Exterior.
- Init: InitObjOn(object 1, false); InitObjOn(object 0, false); InitObjOn(object 17, false); RegisterScriptTrigger(W, S1, index 1); RegisterScriptTrigger(W, S0, index 0).
- Trigger index 1 (S1): if the confirm button is not pressed: DisplaySquareButtonContext(context 4). If it is pressed:
  1. WalkHeroAndOrient(locator 7, WAIT 0, CONTROL 0).
  2. PlayAnim(object 0, WAIT 1, 1.0 s).
  3. PlayAnim(object 17, WAIT 0, 1.0 s).
  4. DeactivateScriptGroup(group 19, W).
  5. ReActivateScriptGroup(group 20, W).
  6. ReActivateScriptGroup(group 21, W).
  7. PlayAnim(object 1, WAIT 0, 1.0 s).
  8. PlayCamera(track 5, WAIT 1, CAMERA_WAIT 1, 1.0 s).
  9. PlayCamera(track 4, WAIT 1, CAMERA_WAIT 1, 1.0 s).
  10. ReturnToNormalCamera(-1, true).
- Trigger index 0 (S0): if object non-null and state is 1:
  1. AddBlackboardMessage(type 2, value 80, eW 22).
  2. FadeScreenOut(fade black, 1.0 s, WAIT 0).
  3. StartExteriorReplacementLoad(eW 22, -1, no colour).
  4. FadeScreenIn(fade black, 1.0 s, WAIT 0).
- Threads: none. Update: empty. Exit: empty.
- Confidence: High for the calls; Medium for the order of the last two steps of S0.

### 788: Interior, proximity line, a teleport and a blackboard-driven arrival (no Exit)

- Family: Interior. Exit is not exported.
- Init: RegisterEntityProximityCallback(W, entity 0, P, true); RegisterTeleportCallback(index 0, T, kind 13, W); RegisterActivateWorldCallback(A, W).
- Proximity P (entity): SetConversationTarget(entity, true); LevelMsg(line 4, W, true, WAIT 1); EndConversation(entity).
- Teleport index 0 (T): diagnostic line; AddBlackboardMessage(type 2, value 50, eW 31); if the trigger index is 0: StartLeaveInterior(eW 44, -1, -1, no colour). Then Wait(3.0 s).
- Activate-world A: diagnostic line; v = GetBlackboardMessageValue(type 2, eW 31); diagnostic line; if v is 50: MoveHero(locator 0, true, CONTROL 1); SetToNormalCamera(5.0, 0.0, true); AddBlackboardMessage(type 2, value -1, eW 31).
- Dormant routine (not registered): SetConversationTarget(argument, true); LevelMsg(line 2, W, true, WAIT 1); EndConversation(argument).
- Threads: none. Update: empty.
- Edge cases: the teleport waits 3.0 s after the call in both branches; the StartLeaveInterior call happens only when the trigger index is 0.
- Confidence: High for the calls; Medium for the teleport branch.

### 799: Transition, a button-driven lever sequence and a query-only trigger

- Family: Transition.
- Init: InitObjOn(object 1, false); InitObjOn(object 2, false); InitObjOn(object 0, false); RegisterScriptTrigger(W, S0, index 0); RegisterScriptTrigger(W, S1, index 1); RegisterScriptTrigger(W, S3, index 3).
- Trigger index 0 (S0): if the record's field at +0x20 is zero:
  - If the confirm button is not pressed: DisplaySquareButtonContext(context 6).
  - If pressed and flag 416 is set: RemoveAllOfItemFromHero(item 16); DeactivateObj(object 0); PlayAnim(object 1, WAIT 0, 1.0 s); SetSpeedFactor(object 1, 1.0); PlayAnim(object 2, WAIT 1, 1.0 s); SetSpeedFactor(object 2, 1.0); SetGameState(flag 663, true); WaitTillLeaveTrigger(record); Wait(1.5 s); PlayAnim(object 1, WAIT 0, 1.0 s); SetSpeedFactor(object 1, -1.0); PlayAnim(object 2, WAIT 0, 1.0 s); SetSpeedFactor(object 2, -1.0); ActivateObj(object 0, false).
  - If pressed and flag 416 is clear: LevelMsg(line 0, W, true, WAIT 1).
- Trigger index 1 (S1): empty.
- Trigger index 3 (S3): if object non-null and state is 1, the hero does not hold item 12 (DoesHeroHaveItem(item 12) is zero) and flag 416 is set: if the module's alternation counter (file value 0) is above zero, LevelMsg(line 1, W, true, WAIT 1); then the counter becomes (counter + 1) mod 2. Line 1 is therefore shown on every second qualifying entry (the first qualifying entry after load shows nothing and sets the counter to 1). When the hero holds item 12 or flag 416 is clear, nothing happens and the counter is unchanged.
- Threads: none. Update: empty. Exit: empty.
- Edge cases: the reverse playback is written as a negative speed factor. The S0 test of +0x20 is the shot word: a hero shot into volume 0 does nothing, any other fire runs S0. The S3 counter lives in module data, so it restarts at 0 whenever the module image is loaded fresh.
- Confidence: High for the calls, the S0 conditions and the S3 counter.

### 787: Interior, a money-gated line, a world-activate hook and a teleport (Exit exported)

- Family: Interior.
- Init: RegisterEntityProximityCallback(W, entity 0, P, true); RegisterTeleportCallback(index 0, T, kind 13, W); RegisterActivateWorldCallback(A, W).
- Proximity P (entity), one line per talk, each in the bracket SetConversationTarget(record entity, true); LevelMsg(line, W, true, WAIT 1); EndConversation:
  - Flag 447 set: m = GetNeopoints(). If m >= 500: SetGameState(flag 448, true), RemoveNeopoints(500), line 3. If m < 500: line 4. Flag 448 is not tested, so every talk with 500 or more Neopoints takes another 500.
  - Flag 447 clear: a rotation over lines 5, 6, 7, 8, 9 and 10 kept in a module counter (file value 0): the counter value c gives line 5 + c and then becomes (c + 1) mod 6. This is an interior, so the counter is 0 again on each entry and the first talk of a visit is line 5.
- Teleport index 0 (T): diagnostic line; if the trigger index is 0: AddBlackboardMessage(type 2, value 30, eW 31) and StartLeaveInterior(eW 44, -1, -1, no colour); Wait(3.0 s).
- Activate-world A: diagnostic line; v = GetBlackboardMessageValue(type 2, eW 31). If v is 30: MoveHero(locator 0, true, CONTROL 1). If v is not -1: SetToNormalCamera(5.0, 0.0, true). Then, in every case: AddBlackboardMessage(type 2, value -1, eW 31).
- Dormant routine: SetConversationTarget(argument); LevelMsg(line 0, W, true, WAIT 1); EndConversation(argument).
- Threads: none. Update: empty. Exit: empty.
- Confidence: High.

### 789: Interior, a random-free proximity pair and a teleport (no Exit)

- Family: Interior. Exit is not exported.
- Init: RegisterEntityProximityCallback(W, entity 0, P, true); RegisterTeleportCallback(index 0, T, kind 13, W); RegisterActivateWorldCallback(A, W).
- Proximity P (entity):
  - If the active hero is hero 1 (WhichHeroIsActive() == 1): Random(1, 3) is drawn (the result is not used by any recorded call); SetConversationTarget(entity); LevelMsg(line 10, W, true, WAIT 1); EndConversation.
  - Otherwise, if flag 86 is set: SetConversationTarget; LevelMsg(line 4); EndConversation.
  - Otherwise: SetConversationTarget; LevelMsg(line 7); EndConversation.
- Teleport index 0 (T): diagnostic line; AddBlackboardMessage(type 2, value 40, eW 31); if the trigger index is 0: StartLeaveInterior(eW 44, -1, -1, no colour); Wait(3.0 s).
- Activate-world A: diagnostic line; v = GetBlackboardMessageValue(type 2, eW 31); if v is 40: MoveHero(locator 1, true, CONTROL 1); SetToNormalCamera(5.0, 0.0, true); AddBlackboardMessage(type 2, value -1, eW 31).
- Dormant routine: if flag 86 is set: SetConversationTarget; LevelMsg(line 0); EndConversation. Otherwise LevelMsg(line 2) with the same bracket.
- Threads: none. Update: empty.
- Confidence: High for the calls; Medium for the draw (its use not recorded).

### 947: Exterior, two threads, a slot load, a death watch and a hit-point watch

- Family: Exterior.
- Init: SlotAllocate(W, 2); SlotCreate(W, 2 entries, slot table in module data), its return value stored in the slot word (one call, one stored value); ControlThreadsAllocate(W, 2); ControlThreadStart(W, thread G, W); RegisterTeleportCallback(index 0, T0, kind 0, W). Then, by flag 1:
  - If flag 1 is set: ControlThreadStart(W, thread H, W).
  - On both flag-1 paths (set and clear): RegisterScriptTrigger(W, S0, index 0); RegisterScriptTrigger(W, S1, index 1); RegisterScriptTrigger(W, S2, index 2); RegisterMovableBlockSoundIndex(W, 7). Only the thread H start depends on flag 1.
- Teleport T0 (kind 0): empty (returns at once).
- Trigger index 2 (S2): if object non-null: sets the module's "armed" word (file value 0) to 1. No host call.
- Trigger index 0 (S0): if the armed word is non-zero, flag 375 is clear, the object is non-null and the state is 1: diagnostic line; LevelMsg(line 0, W, true, WAIT 1); then the armed word is cleared. So line 0 is shown once each time the hero passes through trigger 2 and then enters trigger 0, until flag 375 is set (by thread G).
- Trigger index 1 (S1): if object non-null and state is 1: diagnostic line; StartExteriorReplacementLoad(eW 69, 0, no colour).
- Thread G (runs once): if flag 375 is clear: SlotLoad(W, the stored slot, set 0, true, true); diagnostic line. Then it polls while flag 375 is clear: if IsAI_Dead(entity 3) is true: SetGameState(flag 375, true); LevelMsg(line 1, W, true, WAIT 1); LevelMsg(line 2, W, true, WAIT 1). Otherwise Sleep(1 tick) and poll again.
- Thread H (started when flag 1 is set): each tick it reads GetHitPointsPercentage(-1). When the reading is below 0.2 (strictly less), it shows LevelMsg(line 3, W, true, WAIT 1) and the thread ends after that tick's Sleep(1 tick); otherwise Sleep(1 tick) and poll again. Line 3 is shown at most once per thread start. The provider of GetHitPointsPercentage is not described here (SPEC-0030).
- Update: empty. Exit: empty.
- Confidence: High for Init, Thread G, the S0 and S2 armed word, and Thread H's threshold and end; Medium for the scale of the hit-point reading (the threshold is 0.2 in whatever unit the provider returns).

### 1220: Transition, slot ladder, a camera-and-walk trigger and a blackboard arrival

- Family: Transition.
- Init: SlotAllocate(W, 1); SlotCreate(W, 4 entries, slot table in module data), its return value stored; ControlThreadsAllocate(W, 1); ControlThreadStart(W, thread A, W); RegisterScriptTrigger(W, S0, index 0); RegisterActivateWorldCallback(A2, W).
- Thread A (runs once): flag 84 -> SlotLoad(W, the stored slot, set 3, true, true); flag 83 -> set 2; flag 82 -> set 1; flag 81 -> set 0 (first set flag wins; the stored slot is used in each load; each load is followed by a diagnostic line).
- Trigger index 0 (S0): if object non-null and state is 1:
  1. Diagnostic line.
  2. PlayCamera(track 5, WAIT 0, CAMERA_WAIT 1, 1.0 s).
  3. WalkHero(locator 26, WAIT 0, CONTROL 1).
  4. FadeScreenOut(fade black, 1.0 s, WAIT 1).
  5. MoveHero(locator 29, true, CONTROL 1).
  6. AddBlackboardMessage(type 3, value 1, eW 129).
  Note: the order of steps 3 and 4 is as recorded (walk starts, then fade); the fade is set to wait.
- Activate-world A2: diagnostic line; v = GetBlackboardMessageValue(type 3, eW 131). If v is 1:
  1. FadeScreenOut(fade black, 0.0 s, WAIT 0).
  2. Wait(1.0 s).
  3. FadeScreenIn(fade black, 0.5 s, WAIT 0).
  4. MoveHero(locator 25, false, CONTROL 1).
  5. StartHeroLongFall().
  6. PlayCamera(track 4, WAIT 1, CAMERA_WAIT 1, 0.5 s).
  7. Wait(0.5 s).
  8. ReturnToNormalCamera(-1, true).
  9. AddBlackboardMessage(type 3, value -1, eW 132).
- Update: empty. Exit: empty.
- Confidence: High for the calls; Medium for step order in A2.

### 783: Interior, a line ladder on flags, a teleport and an arrival (no Exit)

- Family: Interior. Exit is not exported.
- Init: RegisterEntityProximityCallback(W, entity 0, P, true); RegisterTeleportCallback(index 0, T, kind 13, W); RegisterActivateWorldCallback(A, W).
- Proximity P (entity):
  - If flag 421 is set: SetConversationTarget; LevelMsg(line 9); EndConversation.
  - Else if flag 420 is set: SetGameState(flag 421, true); SetConversationTarget; LevelMsg(line 12); EndConversation.
  - Else if flag 419 is set: SetConversationTarget; LevelMsg(line 13); EndConversation.
  - Else: SetGameState(flag 419, true); SetConversationTarget; LevelMsg(line 16); EndConversation.
- Teleport index 0 (T): diagnostic line; AddBlackboardMessage(type 2, value 60, eW 31); if the trigger index is 0: StartLeaveInterior(eW 44, -1, -1, no colour); Wait(3.0 s).
- Activate-world A: diagnostic line; v = GetBlackboardMessageValue(type 2, eW 31); if v is 60: MoveHero(locator 0, true, CONTROL 1); SetToNormalCamera(5.0, 0.0, true); AddBlackboardMessage(type 2, value -1, eW 31).
- Dormant routine: if flag 421 is set: SetConversationTarget; LevelMsg(line 0); EndConversation. Else if flag 419 is set: LevelMsg(line 2). Else LevelMsg(line 4). (Each with the same bracket.)
- Threads: none. Update: empty.
- Confidence: High for the calls; the dormant routine is Medium.

### 784: Interior, conditional thread, a proximity line and a trigger with a hero walk (no Exit)

- Family: Interior. Exit is not exported.
- Init, in every case: SlotAllocate(W, 1); SlotCreate(W, 1 entry, slot table in module data), its return value stored; RegisterEntityProximityCallback(W, entity 0, P, true); ControlThreadsAllocate(W, 1); ControlThreadStart(W, thread K, W).
  - Flag 543 clear: InitObjOn(object 2, false) and RegisterScriptTrigger(W, S1, index 1).
  - Flag 543 set: InitObjOn(object 3, false) is the call on this side.
- Proximity P (entity): if flag 591 is set: SetConversationTarget; LevelMsg(line 0, W, true, WAIT 1); EndConversation. If flag 591 is clear: SetGameState(flag 591, true); SetConversationTarget; LevelMsg(line 3, W, true, WAIT 1); EndConversation.
- Thread K (runs once): if flag 590 is set and flag 591 is clear: SlotLoad(W, the stored slot, set 0, true, true); SetEntityUndamageable(entity 0, true, W).
- Trigger index 1 (S1), on every fire (no state test). Nothing happens when the shot word is set. Otherwise, in order:
  - If the confirm button (button 0) is not pressed and flag 543 is clear: DisplaySquareButtonContext(context 4), and the routine ends.
  - Otherwise (button pressed, or flag 543 set), and only when flag 543 is clear, the object is non-null and the attack word is zero:
    - Flag 542 clear: LevelMsg(line 5, W, true, WAIT 1).
    - Flag 542 set: WalkHeroAndOrient(locator 2, WAIT 1, CONTROL 1); PlayHeroAnim(animation 140, WAIT 0, CONTROL 1); PlayAnim(object 2, WAIT 0, 1.0 s); Wait(0.6 s); Wait(0.8 s); Sleep(15); LevelMsg(line 4, W, true, WAIT 1); SetGameState(flag 543, true).
  - With flag 543 set nothing runs, so the sequence plays once per save.
- Threads: thread K as above. Update: empty.
- Confidence: High.

### 837: Transition, a flag-driven world mask, a dormant chooser and a break routine called from Update

- Family: Transition.
- Init, branch on flag 636:
  - If flag 636 is clear: InitObjOn(object 0, false); SetBreakableActive(breakable 3, false, W); SetWorldCollisMask(W, 1); SetWorldDisplayMask(W, 1).
  - If flag 636 is set: SetWorldCollisMask(W, 2); SetWorldDisplayMask(W, 2).
- No triggers or threads are registered.
- Chooser (not reached on any path found): tests flag 58, then 57, then 56. Flag 58 -> SlotLoad(W, slot word (file value -1, never stored in reached code), set 1, true, true); flag 57 -> set 2; flag 56 -> set 3; none -> set 0. Each has a diagnostic line. The four loads sit in the module's first-code entry, which no named export or callback starts; the runtime never reaches that entry (Open question 14). Each load does nothing in the original (SPEC-0032 Slot rules).
- Break routine (called from Update; see Update; live in play): PlaySound(sound 13, no position, false); PlayCamera(track 5, WAIT 0, CAMERA_WAIT 1, 1.0 s); MoveHero(locator 0, true, CONTROL 1); StartCameraShakeMode(shake type 2, 1.0, 1.0); Sleep(30); PlayAnim(object 0, WAIT 0, 0.8 s); WalkHero(locator 1, WAIT 0, CONTROL 0); Sleep(180); PlayCamera(track 6, WAIT 0, CAMERA_WAIT 1, 1.0 s); Sleep(40); StartCameraShakeMode(shake type 0, 1.0, 1.0); SetBreakableActive(breakable 3, true, W); Sleep(1); Break(breakable 3); SetWorldCollisMask(W, 2); SetWorldDisplayMask(W, 2); Wait(1.3 s); StartCameraShakeMode(shake type 0, 0.5, 1.0); Wait(2.0 s); ReturnToNormalCamera(-1, true).
- Update (runs every tick), using a module "armed" word (file value 0, written only by Update):
  - If flag 636 is set: nothing.
  - Else, if the armed word is 0: if IsBreakableIntact(breakable 2) is true, the armed word becomes 1. Nothing else happens on this tick.
  - Else (armed word non-zero): if IsBreakableIntact(breakable 2) is false: the break routine above runs, then SetGameState(flag 636, true).
  So the sequence runs once, on the first tick after breakable 2 has been seen intact and is then no longer intact (the hero broke it). Flag 636 then stops Update and makes the next Init take the flag-636-set branch (masks 2). The break routine and the flag 636 set are live in play, not dormant. Exit: empty.
- Edge cases: the dormant chooser's loads read a slot word that holds -1 in the file and is never stored in reached code (the same pattern as SPEC-0032 module 851), so they would do nothing in the original.
- Confidence: High for the Init branches, the break routine's calls, and the Update gates and flag 636 set.

### 1238: Exterior, a pickup watch, slot sets by flag and two threads

- Family: Exterior.
- Init: SlotAllocate(W, 3) when flag 50 is set, SlotAllocate(W, 2) when it is clear; SlotCreate(W, 2 entries, slot table A), its return value stored in the table A word; ControlThreadsAllocate(W, 1) (2 on one path, by a module-data condition); ControlThreadStart(W, thread T1, W); RegisterHeroPickupItemCallback(W, pickup P, false). Then, by flag 50:
  - If flag 50 is set: SlotCreate(W, 2 entries, table B), its return value stored in the table B word.
  - On both values of flag 50: SlotCreate(W, 1 entry, table C), its return value stored in the table C word.
  - If flag 50 is set: ControlThreadStart(W, thread T2, W).
- Pickup P (hero picked up an item; the first word of the record is the item id): diagnostic line. If the item id is 83 and flag 579 is clear: SetGameState(flag 579, true); diagnostic line. Picking up item 83 therefore sets flag 579 once.
- Thread T1 (runs once), in order: diagnostic line.
  1. If flag 50 is set: diagnostic line; SlotLoad(W, the table A word, set 3, true, true). Then: if flag 567 is set and flag 568 is clear, SlotLoad(W, the table B word, set 1, true, true); otherwise (flag 567 clear, or flags 567 and 568 both set) SlotLoad(W, the table B word, set 2, true, true). Diagnostic line.
  2. If flag 50 is clear: SlotLoad(W, the table A word, set 0, true, true), with diagnostic lines (flag 49 is tested only to choose the diagnostic text; the load is the same either way).
  3. Then, on both flag-50 paths: if flag 578 is set and flag 579 is clear, SlotLoad(W, the table C word, set 4, false, true), in addition to the loads above. Diagnostic line.
  Each load reads the stored slot of the table named here (the table A word for sets 0 and 3, the table B word for sets 1 and 2, the table C word for set 4).
- Thread T2 (started only when flag 50 is set): while flag 567 is set: if flag 568 is clear and the hero holds at least four of item 82, SetGameState(flag 568, true); Sleep(1 tick). The loop ends when flag 567 is clear.
- Update: empty. Exit: empty.
- Edge cases: the table B word is stored only when flag 50 is set, and T1 loads through it only on the flag-50-set path, so every load reads a stored slot.
- Confidence: High for Init, the pickup callback (flag 579), T1's set choice and T2 (the count test was confirmed against the module code).

### 857: Interior, two threads and a proximity line selector

- Family: Interior.
- Init: ControlThreadsAllocate(W, 2); ControlThreadStart(W, thread A, argument -1); ControlThreadStart(W, thread B, W); RegisterEntityProximityCallback(W, entity 0, P, true).
- Thread A (argument -1; runs once): if flag 59 is set: SetConversationTarget(argument); LevelMsg(line 0, W, true, WAIT 1); EndConversation. Else if flag 56 is set: LevelMsg(line 2) with the same bracket. Else LevelMsg(line 4) with the same bracket.
- Thread B (runs once): SetToNormalCamera(5.0, 0.0, true).
- Proximity P (entity): if flag 59 is set: SetConversationTarget(entity); LevelMsg(line 6); EndConversation. Else if flag 56 is set: LevelMsg(line 9) with the bracket. Else LevelMsg(line 12) with the bracket.
- Update: empty. Exit: empty.
- Confidence: High.

### 776: Interior, a proximity selector, a world-activate hook and a teleport (no Exit)

- Family: Interior. Exit is not exported.
- Init: RegisterEntityProximityCallback(W, entity 0, P, true); RegisterTeleportCallback(index 0, T, kind 13, W); RegisterActivateWorldCallback(A, W).
- Proximity P (entity): one routine; every line is shown with the conversation bracket (SetConversationTarget(entity, true); LevelMsg; EndConversation). Order (High):
  1. WhichHeroIsActive() is read first. If it is 1, Random(1, 3) is drawn, and the cycle of step 2 is taken only when the draw is 1. Otherwise go to step 3.
  2. Cycle (hero 1 and draw 1): lines 14, 15 and 16 in turn, from a position counter that starts at 0. Counter 0 shows line 14 and sets the counter to 1; counter 1 shows line 15 and sets it to 2; any other value shows line 16 and sets it to 0. Step 3 is skipped.
  3. Flag branch: flag 68 set -> line 8; flag 68 clear -> line 11. The position counter is set to 1 on this path.
  The routine shows exactly one line per visit. No test of this module shows line 0 or line 2 (see the dormant routines below).
- Teleport index 0 (T): diagnostic line; AddBlackboardMessage(type 2, value 10, eW 31); if the trigger index is 0: StartLeaveInterior(eW 44, -1, -1, no colour); Wait(3.0 s).
- Activate-world A: diagnostic line; v = GetBlackboardMessageValue(type 2, eW 31); if v is 10: MoveHero(locator 1, true, CONTROL 1); SetToNormalCamera(5.0, 0.0, true); AddBlackboardMessage(type 2, value -1, eW 31).
- Dormant routines: two routines that nothing registers or starts. One shows line 4 or line 5, the other line 6 or line 7, each chosen by the result of a query without arguments (both with the conversation bracket) (High).
- Update: empty.
- Confidence: High for the calls and the proximity order.

### 858: Interior, two threads and a proximity routine with item checks

- Family: Interior.
- Init: ControlThreadsAllocate(W, 2); ControlThreadStart(W, thread A, argument 0); ControlThreadStart(W, thread B, W); RegisterEntityProximityCallback(W, entity 0, P, true).
- Thread A (argument 0; runs once): if flag 59 is set: SetConversationTarget(argument); LevelMsg(line 0); EndConversation. Else if flag 56 is set: LevelMsg(line 2). Else LevelMsg(line 4).
- Thread B (runs once): SetToNormalCamera(5.0, 0.0, true).
- Proximity P (entity), in order:
  1. If flag 56 is set and flag 512 is clear: SetGameState(flag 512, true); LevelMsg(line 6) with the bracket.
  2. If flag 56 and flag 512 are set, and flag 514 is clear, and the hero holds item 73: RemoveItemFromHero(item 73, 1); SetGameState(flag 513, true); SetGameState(flag 514, true); LevelMsg(line 7).
  3. Otherwise a rotating line set chosen by flags: flag 59 set -> lines 8, 9, 10 (set 1); flag 59 clear and flag 56 set -> lines 11, 12, 13 (set 2); both clear -> lines 14, 15, 16 (set 3). Two module words keep the last set (file value 0) and a counter (file value 0). When the chosen set differs from the last set, the last set becomes the chosen one and the counter becomes 0. Then counter 0 gives the first line of the set and counter 1, counter 1 gives the second line and counter 2, any other counter gives the third line and counter 0.
  - Branch 1 runs only when flag 56 is set and flag 512 is clear. Branch 2 is tested when branch 1 does not run: flag 56 set, flag 514 clear and item 73 held. Branch 3 runs when neither does. Exactly one line is shown per talk, with the bracket SetConversationTarget(record entity, true); LevelMsg(line, W, true, WAIT 1); EndConversation.
  - This is an interior, so both words are 0 again on each entry.
- Update: empty. Exit: empty.
- Confidence: High.

### 906: Exterior, a per-tick navigation pass over 18 character slots, with routines registered from Init

- Family: Exterior.
- Init (no branch; every call runs on every load, High), in order: the world pointer W is kept in a module word; the table reset (below); RegisterActivateWorldCallback(A, W); RegisterScriptTrigger(W, button routine, index 2); RegisterScriptTrigger(W, long-fall routine L0, index 0); RegisterScriptTrigger(W, long-fall routine L1, index 1); SlotAllocate(W, 1); SlotCreate(W, 1 entry, slot table in module data), its return value stored in the slot word; RegisterHeroPickupItemCallback(W, pickup P, false); ControlThreadsAllocate(W, 1); ControlThreadStart(W, thread T, argument 0).
- Table reset (first step of Init): sets all 18 entries of the nav-mesh table (module data) to -1 (unassigned). No host call.
- Update (runs every tick, ungated): calls the navigation routine, then the assignment routine.
  - Navigation routine: for each slot 0 to 17 whose table entry is -1: entry = GetClosestNavMeshId(W, slot) (the result is always written back; a diagnostic line when it is not -1). Slots whose entry is already assigned are not queried again.
  - Assignment routine: for each slot 0 to 17: if the entry is not -1, CharacterNavmeshAssign(slot, entry); then DisableVillagerSocialize(slot), for every slot. Both run every tick, so an assigned slot is re-assigned each tick.
- Pickup P (hero picked up an item; the first word of the record is the item id): if the item id is 24: SetGameState(flag 406, true). Nothing else.
- Trigger index 2 (button routine; no record test): if the confirm button is not pressed: DisplaySquareButtonContext(context 6). If pressed: diagnostic line; StartExteriorReplacementLoad(eW 65, -1, no colour).
- Trigger index 0 (L0; no record test): StartHeroLongFall; PlayCamera(track 0, WAIT 1, CAMERA_WAIT 0, 1.0 s); SetToNormalCamera(0.3, 0.0, true).
- Trigger index 1 (L1; no record test): the same as L0 with camera track 1: StartHeroLongFall; PlayCamera(track 1, WAIT 1, CAMERA_WAIT 0, 1.0 s); SetToNormalCamera(0.3, 0.0, true).
- Activate-world A (no blackboard test; runs on every activation): sets the fade colour to black; FadeScreenOut(fade black, 0.0 s, WAIT 1); MoveHero(locator 0, true, CONTROL 1); SetToNormalCamera(10.0, 0.0, true); FadeScreenIn(fade black, 1.0 s, WAIT 0).
- Thread T (started by Init, runs once): SlotLoad(W, the stored slot, set 0, false, true). Init stores the SlotCreate result (count 1) into the word that this load reads, so the load uses that slot.
- Every routine above is live in play: Init registers or starts each one with no test in front of it, and the only gates inside them are the ones stated (button, item id, table entries).
- Exit: empty.
- Edge cases: the nav-mesh table is zero in the module file but Init resets it to -1 before Update first runs, so every slot is queried on the first tick. The two long-fall triggers have no record or state test, so they act on every call the runtime makes for them (SPEC-0030 trigger rules).
- Confidence: High for the Init order and arguments, the table reset, the Update routines and the write-back, the pickup flag, the three trigger bodies, activate-world A and the slot load.

### 786: Interior, a three-flag proximity line ladder, a teleport and an arrival (Exit exported)

- Family: Interior.
- Init: RegisterEntityProximityCallback(W, entity 0, P, true); RegisterTeleportCallback(index 0, T, kind 13, W); RegisterActivateWorldCallback(A, W).
- Proximity P (entity):
  - If flag 425 is set: SetConversationTarget; LevelMsg(line 10); EndConversation.
  - Else if flag 424 is set: SetGameState(flag 425, true); SetConversationTarget; LevelMsg(line 13); EndConversation.
  - Else if flag 423 is set: SetConversationTarget; LevelMsg(line 14); EndConversation.
  - Else if flag 76 is set: SetGameState(flag 423, true); SetConversationTarget; LevelMsg(line 17); EndConversation.
  - Else: SetConversationTarget; LevelMsg(line 18); EndConversation.
- Teleport index 0 (T): diagnostic line; AddBlackboardMessage(type 2, value 20, eW 31); if the trigger index is 0: StartLeaveInterior(eW 44, -1, -1, no colour); Wait(3.0 s).
- Activate-world A: diagnostic line; v = GetBlackboardMessageValue(type 2, eW 31); diagnostic line. If v is 20: MoveHero(locator 1, true, CONTROL 1); SetToNormalCamera(5.0, 0.0, true); AddBlackboardMessage(type 2, value -1, eW 31).
- Dormant routine: flag 425 set -> LevelMsg(line 0); flag 425 clear and flag 423 set -> LevelMsg(line 2); flag 76 set -> LevelMsg(line 4); else LevelMsg(line 7). With the bracket. The dormant routine's other paths also reach lines 1, 3 and 5, and one path is not determined (Low).
- Update: empty. Exit: empty.
- Confidence: High for the calls; Medium for the proximity chain.

### 1251: Interior, an item-activation thread, an on-enter callback and an item-ask dialogue chain

- Family: Interior.
- Init: SlotAllocate(W, 1); SlotCreate(W, 1 entry, slot table in module data), its return value stored; ControlThreadsAllocate(W, 1); ControlThreadStart(W, thread A, W); RegisterEntityProximityCallback(W, entity 0, P, true); RegisterOnEnterCallback(callback E, W, index 0); RegisterItemAskAboutCallback(W, item 154, callback Q, true).
- Thread A (runs once): SlotLoad(W, the stored slot, set 0, true, false). Then ItemActivate(W, the stored slot, ordinal, snap false, placed true, no position override, no facing override), with ordinal 0 when flag 460 is set and 1 when it is clear; the result (the item handle) is kept in a module word. The first integer is the SlotCreate result (the stored slot, 0 in play), not a separate item slot, and the ordinal counts the item placements of the region that the load put in that slot (High). The load has activate false, so the region spawns none of its own placements; only the chosen one appears.
  - The region of set 0 holds two Witch's Wand (item 114) shop records at the same point (-21.669, 2.918, -0.756): ordinal 0 is priced 3000 Neopoints and ordinal 1 is priced 10000. So until the hand-in of proximity branch 3 (flag 460 clear) the wand is on sale at 10000, and once flag 460 is set it is on sale at 3000 (SPEC-0021 and SPEC-0050 describe the shop records). Snap false keeps the stored height 2.918 (no ground snap); placed true makes it a static placed item that never expires; with no override vectors it sits at the placement's own point and facing (High).
- On-enter callback E (index 0): flag 460 set: line 18 when flag 273 is set, else line 19. Flag 460 clear and flag 457 set: line 20 when the hero holds item 49, else line 21. Both clear: line 22 when the hero holds item 49, else line 23. Each with the bracket SetConversationTarget(record entity, true); LevelMsg(line, W, true, WAIT 1); EndConversation.
- Item-ask callback Q (item 154): LevelMsg(line 24, W, true, WAIT 1). No condition is recorded.
- Proximity P (entity): at most one line per talk, with the same bracket as E. The first branch that applies runs:
  1. Flag 585 set and flag 586 clear: RemoveAllOfItemFromHero(item 84); SetGameState(flag 586, true); line 28.
  2. Flag 460 set: flag 273 set -> line 29; else flag 270 set -> line 30; else the rotating pair 31, 32.
  3. Flag 457 set and item 49 held: SetGameState(flag 460, true); line 33 when flag 270 is set, else line 34; then RemoveAllOfItemFromHero(item 49); ItemDeactivate(W, the item handle); ItemActivate(W, the stored slot, 0, false, 1, 0, 0). The line is shown after these item steps.
  4. Flag 457 set and item 49 not held: the rotating set 35, 36, 37.
  5. Otherwise (flags 460 and 457 clear): SetGameState(flag 457, true) and line 38 when flag 273 is set, else line 39 when flag 270 is set, else line 40.
  - Rotations: one set word and one counter in module data (both file value 0) serve both rotating sets, and both sets use the same set number 2. On entry to a rotating branch the counter is set to 0 only when the set word is not yet 2. Pair 31, 32: counter 0 gives 31 and counter 1; counter 1 gives 32 and counter 0; counter 2 (left by the other set) gives no line and stays 2. Set 35, 36, 37: counters 0, 1, 2 give 35, 36, 37 and the counter advances to 1, 2, 0.
  - This is an interior, so both words are 0 again on each entry. The item handle is the word that thread A's ItemActivate result was written to; the stored slot is the SlotCreate result.
- Update: empty. Exit: empty.
- Confidence: High.

### 855: Interior, two threads and an item-gated proximity line

- Family: Interior.
- Init: ControlThreadsAllocate(W, 2); ControlThreadStart(W, thread C, argument 0); ControlThreadStart(W, thread B, W); RegisterEntityProximityCallback(W, entity 0, P, true).
- Thread C (argument 0; runs once): if flag 463 is set: LevelMsg(line 0). Else if flag 462 is set and the hero holds item 22: LevelMsg(line 2). Else if flag 462 is set and item 22 is not held: LevelMsg(line 3). Else if flag 59 is set: LevelMsg(line 5). Else if flag 56 is set: LevelMsg(line 6). Else LevelMsg(line 8). Each with the bracket (target = the argument).
- Thread B (runs once): SetToNormalCamera(5.0, 0.0, true).
- Proximity P (entity):
  - If flag 463 is set: LevelMsg(line 10).
  - Else if flag 462 is set and item 22 is held: SetGameState(flag 463, true); RemoveItemFromHero(item 22, 1); GiveItemToHero(item 122, 1, false); LevelMsg(line 13).
  - Else if flag 462 is set and item 22 is not held: LevelMsg(line 14).
  - Else if flag 59 is set: SetGameState(flag 462, true); LevelMsg(line 17).
  - Else if flag 56 is set: LevelMsg(line 18).
  - Else LevelMsg(line 21).
- Update: empty. Exit: empty.
- Confidence: High.

### 1152: Exterior, a proximity ladder, two teleports, a world-activate branch and a slot choice

- Family: Exterior. Exit is not exported.
- Init: SlotAllocate(W, 1); SlotCreate(W, 3 entries, slot table in module data), its return values stored; ControlThreadsAllocate(W, 1); ControlThreadStart(W, thread S, W); RegisterEntityProximityCallback(W, entity 0, P, true); RegisterTeleportCallback(index 1, T1, kind 13, W); RegisterTeleportCallback(index 0, T0, kind 13, W); RegisterActivateWorldCallback(A, W).
- Proximity P (entity), one line per talk with the bracket SetConversationTarget(record entity, true); LevelMsg(line, W, true, WAIT 1); EndConversation. The first test that holds picks the line:
  - Flag 29 set: rotating set 1, lines 1, 2, 3.
  - Flag 28 set: line 4 (no rotation; the set word and counter are unchanged).
  - Flag 27 set: rotating set 2, lines 5, 6, 7.
  - Flags 23 and 633 both set: flag 167 set -> rotating set 3, lines 8, 9, 10; else flag 162 set -> rotating set 4, lines 11, 12, 13; else SetGameState(flag 162, true) and line 14.
  - Otherwise (flag 23 clear or flag 633 clear): rotating set 5, lines 15, 16, 17.
  - Rotation rule: a set word and a counter in module data (both file value 0). When the chosen set differs from the set word, the set word takes the chosen set and the counter becomes 0. Counter 0 gives the first line and counter 1; 1 gives the second and 2; 2 gives the third and 0. This is an exterior, so the words keep their values while the pack stays resident (SPEC-0030, Module lifetime).
- Teleport index 1 (T1): diagnostic line; AddBlackboardMessage(type 2, value 90, eW 106); StartExteriorReplacementLoad(eW 106, -1, no colour).
- Teleport index 0 (T0): diagnostic line; AddBlackboardMessage(type 2, value 90, eW 126); StartExteriorReplacementLoad(eW 126, -1, no colour).
- Activate-world A: v = GetBlackboardMessageValue(type 2, eW 119). If v is 110: MoveHero(locator 1, true, CONTROL 1); SetToNormalCamera(10.0, 0.0, true). Otherwise (any other value, including 90): MoveHero(locator 0, true, CONTROL 1); SetToNormalCamera(10.0, 0.0, true).
- Thread S (runs once): if flag 31 is set: SlotLoad(W, the stored slot, set 2, true, true). Else if flag 30 is set: SlotLoad(the stored slot, set 1). Else SlotLoad(the stored slot, set 0). The activate argument is true in each load. Each has a diagnostic line in the original (not copied).
- Update: empty. Exit: none exported.
- Confidence: High.

### 1429: Interior, two threads, a proximity line selector, a buy-item line and a teleport

- Family: Interior.
- Init: RegisterEntityProximityCallback(W, entity 0, P, true) (registered by the init and again by a helper with a null world, which is refused; see Edge cases); RegisterTeleportCallback(index 0, T, kind 13, W); RegisterHeroBuyItemCallback(W, B, false); ControlThreadsAllocate(W, 2); ControlThreadStart(W, thread K, W); ControlThreadStart(W, thread C, argument 0).
- Thread C (argument 0; runs once): if flag 93 is set: SetConversationTarget(argument); LevelMsg(line 0); EndConversation. Else if flag 92 is set: LevelMsg(line 3). Else LevelMsg(line 6). Each with the bracket.
- Thread K (runs once): SetToNormalCamera(5.0, 0.0, true).
- Proximity P (entity): a rotating line with one counter in module data (file value 0) shared by all three sets and never reset on a change of set: flag 322 set -> lines 11, 12, 13; else flag 236 set -> lines 14, 15, 16; else lines 17, 18, 19. Counter c (0, 1 or 2) gives the set's line c + 1 (first, second, third) and becomes (c + 1) mod 3. With the bracket SetConversationTarget(record entity, true); LevelMsg(line, W, true, WAIT 1); EndConversation. This is an interior, so the counter is 0 on each entry.
- Teleport index 0 (T): AddBlackboardMessage(type 2, value 60, eW 151); StartLeaveInterior(eW 156, -1, -1, no colour). (No diagnostic line and no wait recorded.)
- Buy-item callback B: if the item id (first word of the record) is 109 and flag 248 is clear, it writes 1 into the record's flag word and shows LevelMsg(line 20, W, true, WAIT 1). The nonzero flag word cancels the buy (SPEC-0030 row 260), so item 109 (Golden Shield) cannot be bought here while flag 248 is clear. Any other buy keeps the flag word 0 (the registration passes false) and goes through.
- Update: empty. Exit: empty.
- Edge cases: the second entity-proximity registration (from the helper) passes a null world and is refused (SPEC-0030 null-world rule), so only the first registration takes effect.
- Resident in E: the module of 1429 is the same module as in 1430, 1431 and 1433, and 1427 streams 1429, 1430 and 1431 into the exterior buffer. A module is started by the family of the buffer it is loaded into, so none of the routines above runs for 1429, 1430 or 1431 in E: no proximity talk, no teleport callback (door 0 has no callback and takes the default door rule, which has no destination for these packs), no buy-item hook, no threads, no Update or Exit. The module runs only in 1433, which is entered from 1300 as an interior (SPEC-0006, Edge cases, Interior-kind packs in E). The hero cannot stand in 1429, 1430 or 1431 in play.
- Confidence: High.

### 856: Interior, an item-ask line set, a buy-item line and a proximity selector

- Family: Interior.
- Init: ControlThreadsAllocate(W, 2); ControlThreadStart(W, thread C, argument 0); ControlThreadStart(W, thread K, W); RegisterEntityProximityCallback(W, entity 0, P, true); RegisterItemAskAboutCallback(W, item 100, Q1, true); RegisterItemAskAboutCallback(W, item 107, Q2, true); RegisterItemAskAboutCallback(W, item 113, Q3, true); RegisterHeroBuyItemCallback(W, B, false).
- Thread C (argument 0; runs once): if flag 59 is set: SetConversationTarget(argument); LevelMsg(line 0); EndConversation. Else if flag 56 is set: LevelMsg(line 2). Else LevelMsg(line 4). With the bracket.
- Thread K (runs once): SetToNormalCamera(5.0, 0.0, true).
- Proximity P (entity): if flag 59 is set: LevelMsg(line 10). Else if flag 56 is set: LevelMsg(line 13). Else LevelMsg(line 16). With the bracket.
- Item-ask Q1 (item 100): SetConversationTarget(0); LevelMsg(line 6); EndConversation.
- Item-ask Q2 (item 107): SetConversationTarget(0); LevelMsg(line 7); EndConversation.
- Item-ask Q3 (item 113): line 8 when WhichHeroIsActive() returns 0, else line 9, in the bracket SetConversationTarget(0); LevelMsg; EndConversation. (Item-ask callbacks never fire on disc; SPEC-0030 row 257.)
- Buy-item callback B: if the item id is 113 and flag 220 is clear, it writes 1 into the record's flag word, which cancels the buy (SPEC-0030 row 260), and shows LevelMsg(line 19, W, true, WAIT 1). So item 113 (Enchanter's Wand) cannot be bought here while flag 220 is clear; other buys go through.
- Update: empty. Exit: empty.
- Confidence: High.

## Edge cases and error handling

- Null world registration: a registration whose world argument is null is refused and registers nothing (SPEC-0030). 1429 relies on this for a duplicate registration.
- Trigger index not in the table: ignored (SPEC-0030).
- Dormant routines: present in 788, 789, 783, 787, 776 and 786, and the chooser in 837 (not reached from init or from a registered callback; no reference to any of them exists in its module). A reimplementation must not run them. The routines of 906 registered or started by Init (button trigger, both long-fall triggers, activate-world A, pickup P, thread T) and the 837 break routine called from Update are live in play, not dormant.
- Update work: 906 does per-tick work (navigation and villager calls). 837's Update arms a word while breakable 2 is intact and, when it is later found broken, runs its break routine and sets flag 636. 1427 clears its once-flag. All other modules have an empty Update.
- Missing Exit: 788, 789, 783, 784, 776 and 1152 do not export Exit. Nothing is run for them on exit (SPEC-0030).
- Slot ids: the loads in 1225, 809, 1259, 947, 1220, 784, 1238, 1251 and 1152 use the slot number that the module's SlotCreate stored (SPEC-0032 Slot rules). The loads through a slot word that no code stores are in 837 (4 loads, High); all four are in a dormant routine and do nothing in the original. The call path does not remap the file value. The 906 word is stored by Init (SlotCreate, count 1), so its load in the thread T routine uses that slot.
- Slot words stored in play (confirmed against the module code): each SlotCreate result of this spec is stored by the straight-line code that follows its call, with no branch in between, so none is stored under the debug query. The only flag gate on a store is 1238's table B, which is created and stored only on the flag-50-set path. 1225, 1220, 784, 1251 and 1152 store one word each. 809 stores its word in Init (after its SlotCreate call, in play). 1259 and 947 store their word in Init. 1238 stores three words: table A always, table C always, and table B only on the flag-50-set path. The loads listed above read these same words.
- Slot rule: SlotCreate returns the slot number, or -1 when no slot is free. A SlotLoad or SlotReset through a slot word that no code stores does nothing in the original (the call path does not remap -1); a stored slot number that has no SlotCreate buffer is not covered here (SPEC-0032 open question 13).
- Diagnostic lines: diagnostic output has no game-state effect and may be skipped.
- Blackboard value -1: used as a reset after a read (several modules write -1 back after reading the value).
- Trigger fields +0x20 (shot word) and +0x24 (attack word): 784 S1 ignores a hero shot and needs a non-attacking body for its line and walk; 799 S0 ignores a hero shot (SPEC-0030 section 3).
- Partly described line sets: only the 786 dormant routine; the 1251 on-enter lines and the 856 item-ask Q3 line are fully given in their sections.
- Rotating line sets (787, 858, 1251, 1152, 1429) keep their set word and counter in module data; an interior restarts them at their file value 0 on each entry.
- Buy refusals: 1429 (item 109 while flag 248 is clear) and 856 (item 113 while flag 220 is clear) cancel the buy by writing 1 into the buy record's flag word.

## Test vectors

- 776, proximity routine with WhichHeroIsActive() 2 and flag 68 clear: LevelMsg(line 11) with the conversation bracket. With hero 1 and the draw 1 on the first visit: LevelMsg(line 14); the next such visit shows line 15, then line 16, then line 14 again.
- 1427, the first trigger callback in a tick is index 0 and the override is 187; a second index 2 callback in the same tick does nothing.
- 1152, activate-world with blackboard value 110 -> MoveHero(locator 1, true, CONTROL 1), then SetToNormalCamera(10.0, 0.0, true). Value 90 -> locator 0.
- 1253, activate-world with value 20 -> MoveHero(locator 1, false, CONTROL 1), FadeScreenIn(0.5 s), StartHeroLongFall, PlayTrackHeroCamera(track 6, 1.0 s), Wait(1.5 s), ReturnToNormalCamera.
- 1238, with flag 567 set and flag 568 clear, the hero holding four or more of item 82 sets flag 568 on the next tick.
- 809, with the debug-mode query returning false (play mode): SlotAllocate(W, 1), SlotCreate(W, 5 entries) with its return value stored, ControlThreadsAllocate(W, 1) and ControlThreadStart(W, thread D, W) run; no SetGameState call runs.
- 857, thread A (argument -1) with flag 59 clear and flag 56 set -> LevelMsg(line 2).
- 1259, trigger index 2 with the object pointer set -> SlotReset(W, the stored slot), then SlotLoad(W, the stored slot, set 2, true, true).
- 837, flag 636 clear and the armed word 0: tick 1 with breakable 2 intact -> the armed word becomes 1, no other call. Tick 2 with breakable 2 still intact -> nothing. Tick 3 with breakable 2 broken -> the break routine runs, then SetGameState(flag 636, true); later ticks do nothing.
- 799, trigger index 3 entered three times (object set, state 1, item 12 not held, flag 416 set), counter starting at 0 -> no line, LevelMsg(line 1), no line.
- 947, thread H readings 0.5, 0.2, 0.19 on three ticks -> LevelMsg(line 3) on the third tick only, then the thread ends.
- 906, pickup of item 24 -> SetGameState(flag 406, true); pickup of any other item -> nothing.
- 1253, activate-world with value 30 -> MoveHero(locator 2, false, CONTROL 1), then the fall with PlayTrackHeroCamera(track 7). With no message (value -1) -> MoveHero(locator 0, false, CONTROL 1) and track 5.
- 787, flag 447 set: talk with 520 Neopoints -> SetGameState(448, true), RemoveNeopoints(500), line 3; a second talk (20 left) -> line 4. Flag 447 clear, seven talks in one visit -> lines 5, 6, 7, 8, 9, 10, 5.
- 784, trigger index 1 with flag 543 clear, flag 542 set, button pressed, object set, shot and attack words 0 -> WalkHeroAndOrient(2), PlayHeroAnim(140), PlayAnim(2), Wait(0.6 s), Wait(0.8 s), Sleep(15), LevelMsg(4), SetGameState(543, true). The same with the attack word 1 -> nothing. Button not pressed -> DisplaySquareButtonContext(4) only.
- 858, flags 56 and 512 set, flag 514 clear, item 73 not held, flag 59 clear, four talks -> lines 11, 12, 13, 11. Then flag 59 set -> line 8 (the set changed, so the counter restarts).
- 1251, flag 585 clear, flag 460 set, flags 273 and 270 clear, three talks -> lines 31, 32, 31. On a new visit with flag 460 clear, flag 457 set and item 49 not held, four talks -> lines 35, 36, 37, 35.
- 1251 item activation: entering with flag 460 clear puts the 10000-Neopoint Witch's Wand at (-21.669, 2.918, -0.756). Talking with flag 457 set and item 49 held then sets flag 460, removes the 10000 wand and places the 3000-Neopoint wand at the same point. Entering with flag 460 set places the 3000 wand at once.
- 1152, only flag 23 and flag 633 set, flags 167 and 162 clear -> SetGameState(162, true), line 14; the next talk (flag 162 now set) -> line 11 (first talk of set 4 since the pack was loaded).
- The 1429 module running in 1433, a buy of item 109 with flag 248 clear -> line 20 and no purchase; with flag 248 set -> the purchase runs and no line.

## Open questions

1. Slot id and SlotCreate's return value (determined in part). SlotCreate returns the slot number of the buffer it reserves, in creation order, or -1 when no buffer is left (SPEC-0032 Slot rules; SPEC-0030 row 207). The loads in 1225, 809, 1259, 947, 1220, 784, 1238, 1251 and 1152 use the slot number that the module stored from its SlotCreate call. The call path does not remap -1 (High). Loads through a slot word that no code stores (file value -1): 837 (4 loads, not reached on any path found, High). They do nothing in the original. The 906 slot word is stored by Init (SlotCreate, count 1), and its load in the thread T routine uses that slot.
2. Resolved: +0x20 is the shot word (1 when a hero shot fired the trigger) and +0x24 the attack word (1 when the selected hero's body fired it during a hit-window frame), SPEC-0030 section 3. 784 S1 does nothing on a shot and runs its line and walk only for a non-attacking body; 799 S0 does nothing on a shot (High).
3. Resolved: 1369 trigger index 1 has no state test (only the object pointer); 1391 trigger index 2 tests the object pointer, a once-word and state 1, in that order, and sets the once-word before its first action; 1369 trigger index 0 tests the object pointer and a once-word that its confirmed press sets (High). Test vector: 1391 trigger 2 fired with state 2 does nothing; fired with state 1 it plays the camera, shake and animation once, and a second state 1 fire in the same module load does nothing.
4. Resolved: each world id is an area slot; the slot names its pack and the pack's map area gives its name (SPEC-0006 "Area names", SPEC-0051 "Map area names"). E exterior, I interior, T transition: 22 = 735 E, Bogshot Swamp; 31 = 758 E, Brightvale Village; 44 = 797 T, Brightvale Outskirts; 65 = 893 E, Drackon Ridge; 69 = 934 E, Farm Road; 106 = 1098 E, no map area of its own (its script registers 48 and 49, Meridell Castle 1st and 2nd Floor); 119 = 1152 E, Meridell Castle 1st Floor; 126 = 1173 E, no map area, adjoins Meridell Treasure Vault, Meridell Castle 1st Floor and Meridell Sewers; 129 = 1184 E, Shadowglen Woods; 131 = 1220 T, Shadowglen Burrows; 132 = 1225 T, Shadowglen Woods; 141 = 1250 E, Werelupe Woods; 143 = 1253 E, Werelupe Sinkholes; 150 = 1299 E, Fyora's Library; 151 = 1300 E, Fyora's Palace Gardens; 154 = 1327 E, Fyora's Library; 155 = 1335 E, Fyora's Palace; 156 = 1344 T, Fyora's Palace; 157 = 1350 E, Arena District; 165 = 1382 E, Heroes Rest. The override ids are slots too: 187 = 1428 E, Fyora's Tower; 188, 189 and 190 = 1429, 1430 and 1431, interior-kind packs streamed into the exterior buffer (SPEC-0006), no map area, adjoin Fyora's Palace; 191 = 1432 E, no map area, adjoins Fyora's Palace Gardens and Hall of Heroes (High).
5. Event codes 906 (908) and the world-swap events of the other modules.
6. Resolved: 1238's pickup callback sets flag 579 (when the item id is 83 and flag 579 is clear), and thread T1's set choice depends on flags 50, 567, 568, 578 and 579 as given in its section.
7. Resolved for 787: the money test is GetNeopoints() >= 500 (500 or more takes 500 and sets flag 448; flag 448 is not tested). Resolved for 947 thread H: the reading is compared strictly below 0.2, line 3 is shown once and the thread ends. Still open: the provider of GetHitPointsPercentage (SPEC-0030 row 52).
8. Resolved for item names (SPEC-0021 "Item names"): 12 Rathbone Family Tree; 16 Rathbone Crypt Key; 22 Hammer of Thunder; 24 Kreludan Metal; 49 Wild Wartroot; 73 Growth Pod; 82 Poison Seed; 83 Pink Posies; 84 Creepy Package; 100 Knight's Sword; 107 Iron Shield; 109 Golden Shield; 113 Enchanter's Wand; 122 Defender's Armour; 154 Starry Negg (High). Resolved: the item-activation arguments of 1251 are the stored slot, the placement ordinal (0 when flag 460 is set, else 1), snap false, placed true and two null overrides; they select the 3000 or 10000 Witch's Wand record (module 1251 section).
9. Resolved: 784 trigger 1, the 1251 on-enter and proximity chains, the 858 proximity chain and the 1152 proximity chain are fully given in their sections, with their rotating line sets (High). Line text is in the world's text (not copied).
10. Resolved: every routine of 906 that Init registers or starts (button trigger, both long-fall triggers, activate-world A, pickup P, thread T) runs in play with only the gates stated in its section, and the 837 break routine runs once, when breakable 2 goes from intact to broken while flag 636 is clear.
11. Resolved: Init resets all 18 entries of the 906 nav-mesh table to -1, and the navigation routine writes each GetClosestNavMeshId result back into its slot's entry. Which values the host returns depends on the level's navigation meshes.
12. The fifth argument of SlotLoad (activate) is established: true in every call of this spec whose value is determined except 1251 thread A (false). The ItemActivate fifth, sixth and seventh arguments are 1, 0 and 0 in 1251: placed true and no position or facing override, so the item appears at the placement's own point (resolved).
13. Resolved: 1253 handles 20, 30 and 10 by locator and track and treats every other value like 10; the 1152 proximity chain is in its section; the 1429 buy-item callback cancels the buy of item 109 while flag 248 is clear (High).
14. 837: the four chooser loads sit in the module's first-code entry, which no named export or callback starts. Two of the four loads are reached only from dormant routines and two from no root on any path found. Resolved (proactive (open close-out); Medium-High): the runtime does not reach that entry. The host runs only the load hook (its start list is empty in this module), the family Init, Update and Exit, and the callbacks and threads they register (SPEC-0030, Which level routines run in play), so the four loads are dead code and the no-op verdict stands.
15. Resolved: 906 calls SlotAllocate(W, 1); its script triggers are index 2 (button routine), index 0 and index 1 (long-fall routines, no state condition); the pickup callback sets flag 406 for item 24; the activate-world callback is the fade-and-move sequence, with no blackboard test.
16. Resolved: the helper called first in 906's Init resets the nav-mesh table to -1 and makes no host call; it does not change any later argument.
17. Resolved: 837's Update arms a module word on the first tick that breakable 2 is intact; on a later tick with the word armed and breakable 2 no longer intact it runs the break routine and then sets flag 636.
