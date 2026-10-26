# SPEC-0032: Level scripts (pilot set)

- Answers: proactive (level script behavior; pilot of 20 modules)
- Status: Draft
- Disc build: SCUS-97367 v1.00 (same fingerprint as SPEC-0001)
- Confidence: per section. Host call names and argument order: High. Trigger record state values (1 = entered): Medium-High. Argument values listed per module: High unless marked. All host calls and arguments are as given by each module's host-call records. Slot rules: High for SlotCreate's return value, for the rejection of out-of-range slots and for the absence of any remap on the call path; a load through a slot word (file value -1, never written by any reachable part of the module) does nothing (High for 851). Remaining Medium items are the open questions below.

## Summary

Each world pack on the disc may carry one level script module (chunk type 37, "SNR2"). A module is a small linked program with three entry points: an init hook called when the world loads, an update hook called every game tick, and an exit hook called when the world unloads. Behavior is set up in init by registering triggers (volumes that fire on a hero or entity event), proximity and teleport callbacks, world-swap callbacks, and control threads (cooperative scripts that run one step per tick and yield while they wait). This spec describes 20 modules. Most of them only register a few triggers and call presentation APIs (camera, fade, dialogue line, hero walk, slot load). Several are empty no-op modules.

Times are seconds unless marked as ticks or frames. The game tick is 60 Hz (SPEC-0007), so 60 frames = 1 second.

## Where it lives on the disc

- Each module is the level-script chunk (type 37) of a world-pack resource in GAMEFILE.DAT. The clean side finds it by resource number (SPEC-0004 and the world loader rules). Resource numbers below are world-pack resource numbers.
- The module image is linked at load. Imported host calls are resolved by name against the host script table. Module-internal addresses are not part of this spec.
- The three export names identify the family: Exterior, Interior or Transition. Each family has the same three hook roles (init with the world handle, update, exit). Some modules omit exit; treat a missing exit as a no-op.
- Module data holds small integer tables (slot tables), debug strings and pointer cells. The clean side does not need these except for the slot tables (see Slot rules below).

## Load and unload plumbing (all pilot modules)

- Each module has a load hook and an unload hook that run its own start and stop lists (function-pointer lists in module data, filled by the loader). In all 20 pilot modules these lists are empty at load, so the hooks have no observable effect and need no implementation.
- A once-only guard keeps the lists from running twice. No behavior depends on it here.
- Confidence: High for the list layout; Medium for "empty at load" (from each module's data, not observed at run time).

## Host API used by this set (names as in the host table)

Argument conventions (from the host signatures, values from the modules):
- WAIT mode: 0 = return at once, 1 = yield until the action finishes. Other values not seen.
- CONTROL mode, CAMERA_WAIT mode: passed through as raw integers (values 0 and 1 in this set). CONTROL is compared with 1; CAMERA_WAIT is the end mode of a scripted camera play, 0, 1 or 2 on disc (SPEC-0030, Layout 2.1).
- Sleep(frames): yields that many ticks. Wait(seconds): yields trunc(seconds × 60) ticks, one single-precision multiply by the tick rate 60 (SPEC-0030, Seconds to ticks).
- Boolean arguments are 0 or 1.
- WORLD handle: the value passed to init (or to the registration call) is passed through as the world argument of later calls.
- Trigger callbacks receive a COLLIS record. Its state word (offset 0x10 in the record) is 1 when the hero or entity has entered, and its object pointer (offset 0x1C) must be non-null for the trigger to act. Several modules test whether the state is 1; others test whether it is not zero. Both are written per module.
- Registration calls: RegisterScriptTrigger(world, callback, index), RegisterTeleportCallback(callback, index, context kind, world), RegisterScriptWorldSwap(world, callback), RegisterActivateWorldCallback(callback, world), RegisterHeroPickupItemCallback(world, callback, bool), RegisterTransitionMap(id).
- Teleport context kind 13 (0xD) is the only one used in this set.

## Slot rules (this set)

- SlotAllocate(W, n) creates n empty slots for the world and returns 1 (High).
- SlotCreate(W, n, table) loads nothing. It reserves a buffer for the next slot number (slot numbers count up from 0 in creation order) and returns that slot number. When all n slots already have buffers it returns -1 and reserves nothing (High). A module that stores this return value and passes it to SlotLoad loads into a real slot. Every module of this set that loads a slot does so with a stored SlotCreate result, except module 851 (see its section).
- SlotLoad(W, slot, set j, keep, activate) loads character set j (region pack + 1 + j) into the slot. A slot number outside 0 to n - 1, where n is the count given to SlotAllocate, is rejected: nothing is loaded, released or spawned, and the call returns 0 (High).
- A slot word whose file value is -1 is not a default slot. The slot argument of a load comes from a module data word whose file value is -1. Most modules store a SlotCreate result into that word before any load. The call path does not remap the value (High). A SlotLoad through a slot word that no reachable part of the module writes (file value -1) does nothing (no load, no spawn); module 851 is such a case (dormant). Module 851 is the only module of this set whose slot word is never written by a reachable part of the module. Elsewhere the slot is the stored SlotCreate result.
- The activate argument (fifth argument of SlotLoad) is 1 in every load of this set. Activation only spawns characters when the load succeeds and the slot number is below the number of slots that SlotCreate has created (SPEC-0030 row 205).
- The fifth argument of SlotLoad is known for this set; see Open questions 12.

## Modules

Each module section lists: family; init steps in order; each registered callback with its index and what happens when it fires; threads; update and exit; edge cases; confidence; open questions.

### 1003: Exterior, no-op module

- Family: Exterior.
- Init, update and exit do nothing. No host calls, no triggers, no threads.
- Confidence: High.

### 798: Transition, no-op module

- Family: Transition.
- Init, update and exit do nothing. No host calls, no triggers, no threads.
- Confidence: High.

### 1245: Exterior, store world only

- Family: Exterior.
- Init: stores the world handle in a module cell. Nothing else reads that cell in this module.
- Update and exit: nothing.
- Host calls: none.
- Confidence: High.

### 1256: Transition, debug line and world handle

- Family: Transition.
- Init: prints one diagnostic line (text not given), then stores the world handle.
- Update and exit: nothing.
- Host calls: DebugPrintf only.
- Confidence: High.

### 1254: Transition, sinkhole world-swap

- Family: Transition.
- Init: registers a world-swap callback (W1) on the world.
- W1 (world swap, receives a record whose first word is an event code):
  - Always prints one debug line with the event code.
  - If the event code is 1253: MoveHero(locator 0, flag 1, control 1, extra 1); PlayCamera(mode 6, wait 0, camera wait 0, 1.0 s); PlayHeroAnim(animation 136, wait 0, control 1); Wait(3.0 s); ReturnToNormalCamera(-1, flag 1).
  - Any other code: debug line only.
- Update and exit: nothing.
- Confidence: High (all calls and arguments).
- Open: what event code 1253 is (the sinkhole fall, most likely).

### 685: Transition, two triggers set the transition map

- Family: Transition.
- Init: stores the world handle; registers trigger index 0 with callback T0 and trigger index 1 with callback T1.
- T0 (trigger 0): RegisterTransitionMap(54). No state test; fires on any call the engine makes for that trigger.
- T1 (trigger 1): RegisterTransitionMap(43). No state test.
- Update and exit: nothing.
- Confidence: High for the calls and their order; Medium for "no state test" (the callbacks take no branch).

### 1255: Transition, flag-gated transition map

- Family: Transition.
- Init: stores the world handle. If game-state flag 273 is set and flag 274 is clear, sets flag 274 to 1. Then registers six scripted triggers:
  - index 4 with callback TM5 (sets transition map 5), then index 3 with TM5, then index 4 again with callback TM5b (sets transition map 5), then index 5 with TM5b, then index 0 with TM7 (transition map 7), then index 1 with TM7.
- Index 4 is registered twice. The later registration replaces the first: registering writes the callback into the trigger record of that index, so index 4 ends up with TM5b (SPEC-0030 row 250 and section 3). Both set map 5, so play is the same either way (High).
- TM5, TM5b: RegisterTransitionMap(5). TM7: RegisterTransitionMap(7). No state tests.
- Update and exit: nothing.
- Confidence: High for calls and for the replace rule.
- Transition map ids are map-area ids (SPEC-0051 "Map area names"): 5 is Werelupe Burrows Upper and 7 is Werelupe Burrows Lower (Open question 10).

### 702: Transition, button prompt and a message on entry

- Family: Transition.
- Init: stores the world handle. Registers trigger index 1 with callback B1. If game-state flag 252 is clear, also registers trigger index 0 with callback M0 and calls SetWorldCollisMask(world, 1).
- B1 (trigger 1): if the object is non-null and state is non-zero:
  - DisplaySquareButtonContext(context 3).
  - If button 0 is pressed (IsButtonPressed(button 0, flag 1, extra 0)), then LevelMsg(line 3, world, wait 1).
- M0 (trigger 0): if the object is non-null and state is exactly 1, and flag 252 is still clear: LevelMsg(line 5, world, wait 1).
- Update and exit: nothing.
- Confidence: High.
- Edge: M0 does not set any flag; it can repeat each time the trigger fires while flag 252 is clear.

### 851: Transition, dormant character-set chooser

- Family: Transition.
- Init, update and exit: nothing (each returns at once).
- The module contains a chooser routine (the body of a character-set loader). No import, export, thread start or data pointer in this module refers to it. As shipped, nothing in this module runs it.
- If it were run with world W, it would do the following in order:
  - If flag 58 is set: SlotLoad(W, slot, set 1, keep 1, activate 1), with debug lines before and after.
  - Else if flag 57 is set: SlotLoad(W, slot, set 2, keep 1, activate 1), with debug lines.
  - Else if flag 53 is set: SlotLoad(W, slot, set 0, keep 1, activate 1).
  - Else: print a debug line and load nothing.
- Edge: the slot word the chooser reads has the file value -1, and no reachable part of the module writes a slot-create result into it. Each load would therefore do nothing in the original (Slot rules). The three loads sit in the module's first routine, which no named export or callback starts (Open). The fifth argument of each load is 1.
- Confidence: High that nothing in the module calls it; High for the chooser rules.
- Open: is this a leftover from a removed thread, or started by the host by name?

### 816: Transition, breakable switch and pickup flag

- Family: Transition.
- Init: stores the world handle.
  - If flag 516 is set and flag 517 is clear: SetBreakableActive(breakable 11, false, world).
  - Otherwise: SetBreakableActive(breakable 12, false, world).
  - Registers a hero-pickup callback P0 with flag 0 (the bool argument is 0).
  - ControlThreadsAllocate(world, 1). ControlThreadStart(world, thread X, arg 0).
- P0 (hero picks up an item): if the item id (first word of the pickup record) is 76, SetGameState(flag 517, 1).
- Thread X: returns at once (empty body).
- Update and exit: nothing.
- Confidence: High.

### 955: Interior, teleport out to world 5

- Family: Interior.
- Init: stores the world handle; RegisterTeleportCallback(T0, index 0, context kind 13, world).
- T0 (teleport index 0):
  - AddBlackboardMessage(type 2, value 40, world id 68).
  - StartLeaveInterior(eWorlds 5, -1, -1, no color).
- Update and exit: nothing.
- Edge: the module clears a small local buffer before the call; the buffer is not passed on (the color argument is null).
- Confidence: High.
- eWorlds 5 is pack 691 (Farm Road) and eWorlds 68 is pack 909 (Ellis Family Farm) (Open question 3).

### 1080: Exterior, hole trigger with a camera shake

- Family: Exterior.
- Init: stores the world handle; SetGameState(flag 39, 1); RegisterScriptTrigger(world, S2, index 2).
- S2 (trigger 2): always prints a diagnostic line (text not given).
  - If the object is non-null and state is 1:
    1. Sleep(15 frames).
    2. PlayLookAtPlayerCamera(mode 4, wait 0, camera wait 1, 1.0 s).
    3. PlayHeroAnim(animation 137, wait 0, control 1).
    4. StartCameraShakeMode(shake type 0, duration 0.3 s, strength 0.3).
    5. ReturnToNormalCamera(2, flag 1).
  - If the object is null: ReturnToNormalCamera(2, flag 1) only.
  - If the object is non-null but state is not 1: ReturnToNormalCamera(2, flag 1) as well.
- Update and exit: nothing.
- Edge cases: none (the value is 2 on every path).
- Confidence: High for the entered sequence; Medium for the calls on the other paths.

### 1055: Transition, world swap with two paths

- Family: Transition.
- Init: debug line; InitObjOn(world, object 0, flag 0); RegisterScriptWorldSwap(world, W1).
- W1 (world swap, first word is the event code):
  - If the event code is 1098:
    1. MoveHero(locator 1, flag 1, control 1, extra 1).
    2. ActivateObj(object 0, flag 0).
    3. PlayCamera(mode 4, wait 0, camera wait 1, 1.0 s).
    4. PlayAnim(object 0, wait 0, 1.0 s).
    5. Sleep(60 frames).
    6. WalkHero(locator 2, wait 1, control 1).
    7. Sleep(60 frames).
  - Otherwise:
    1. MoveHero(locator 3, flag 1, control 1, extra 1).
    2. ActivateObj(object 0, flag 0).
    3. PlayCamera(mode 5, wait 0, camera wait 1, 1.0 s).
    4. PlayAnim(object 0, wait 0, 1.0 s).
    5. Sleep(60 frames).
    6. WalkHero(locator 0, wait 1, control 1).
    7. Sleep(60 frames).
- Update and exit: nothing.
- Confidence: High for the first path; Medium for the second (its arguments mirror those of the first path).

### 1170: Interior, return trigger, teleport and blackboard-driven arrival

- Family: Interior (no exit export).
- Init: stores the world handle; RegisterTeleportCallback(T0, index 0, kind 13, world); RegisterScriptTrigger(world, S0, index 0); RegisterActivateWorldCallback(A0, world).
- S0 (scripted trigger 0): if the object is non-null and state is 1:
  - AddBlackboardMessage(type 2, value 20, world 106).
  - StartLeaveInterior(eWorlds 107, -1, -1, no color).
- T0 (teleport 0): prints one debug line, then AddBlackboardMessage(type 2, value 10, world 106), then StartLeaveInterior(eWorlds 107, -1, -1, no color).
- A0 (activate world, when the world becomes active):
  1. v = GetBlackboardMessageValue(type 2, value id 123, eWorlds).
  2. Prints one debug line with v.
  3. If v is 20: MoveHero(locator 1, flag 1, control 1, extra 1).
  4. Otherwise (v is 10 or any other value): MoveHero(locator 0, flag 1, control 1, extra 1).
  5. SetToNormalCamera(10.0, 0.0, flag 1).
- Update: nothing.
- Test vectors: blackboard value 20 -> MoveHero locator 1. Value 10 -> locator 0. Value 7 -> locator 0.
- Confidence: High.

### 699: Exterior, world masks and two triggers

- Family: Exterior.
- Init:
  1. Stores the world handle.
  2. If flag 265 is set: SetWorldDisplayMask(world, 2), SetGameState(flag 266, 1), base mask = 2.
  3. Otherwise: SetWorldDisplayMask(world, 1), base mask = 1.
  4. If flag 220 is clear: add bit value 4 to the base mask (bitwise OR). Otherwise add bit value 8.
  5. SetWorldCollisMask(world, base mask).
  6. RegisterScriptTrigger(world, S0, index 0); RegisterScriptTrigger(world, S1, index 1).
- S0 (trigger 0): if the object is non-null and state is 1:
  1. If flag 259 is clear: SetGameState(flag 259, 1).
  2. If flag 220 is set and flag 258 is clear: LevelMsg(line 6, world, flag 1, wait 1).
- S1 (trigger 1): if the object is non-null and state is 1 and flag 659 is clear: LevelMsg(line 5, world, 1, 1), then SetGameState(flag 659, 1).
- Update and exit: nothing.
- Confidence: High (S0, S1 and Init).

### 797: Transition ("Village"), world swap with two paths

- Family: Transition.
- Init: RegisterScriptWorldSwap(world, W1).
- W1 (world swap, first word is the event code): always prints a diagnostic line (text not given).
  - If the event code is 753:
    1. Debug line.
    2. PlayCamera(mode 4, wait 0, camera wait 0, 1.0 s).
    3. MoveHero(locator 1, flag 1, control 1, extra 1).
    4. Debug line.
    5. ActivateObj(object 0, flag 0).
    6. PlayAnim(object 0, wait 0, 1.0 s).
    7. Debug line.
    8. WalkHero(locator 0, wait 1, control 1).
    9. Debug line.
    10. ReturnToNormalCamera(-1, flag 1).
  - Otherwise:
    1. Debug line.
    2. PlayCamera(mode 5, wait 0, camera wait 0, 1.0 s).
    3. MoveHero(locator 0, flag 1, control 1, extra 1).
    4. Debug line.
    5. ActivateObj(object 0, flag 0).
    6. PlayAnim(object 0, wait 0, 1.0 s).
    7. Debug line.
    8. WalkHero(locator 1, wait 1, control 1).
    9. Debug line.
    10. ReturnToNormalCamera(-1, flag 1).
- Update and exit: nothing.
- Confidence: High (every call and argument; the debug lines are not reproduced).

### 950: Interior, long fall and a fade-to-black trigger

- Family: Interior.
- Init: ControlThreadsAllocate(world, 1); ControlThreadStart(world, thread F, arg world); RegisterScriptTrigger(world, S0, index 0).
- Thread F (runs once):
  1. StartHeroLongFall().
  2. PlayCamera(mode 4, wait 1, camera wait 1, 0.5 s).
  3. Wait(0.5 s).
  4. SetToNormalCamera(10, flag 0).
  5. ReturnHeroAnimToIdle().
- S0 (scripted trigger 0): if the object is non-null and state is 1:
  1. FadeScreenOut(color black (0,0,0), 0.5 s, wait 1).
  2. AddBlackboardMessage(type 2, value 30, world 68).
  3. StartLeaveInterior(eWorlds 5, -1, -1, no color).
- Update and exit: nothing (Interior exit not exported).
- Confidence: High.

### 1074: Exterior, world init with slots and an item check

- Family: Exterior.
- Init:
  1. SlotAllocate(world, 1).
  2. SlotCreate(world, 2, slot table T1). The returned slot number is stored in a module cell and used by the loads below.
  3. ControlThreadsAllocate(world, 2).
  4. ControlThreadStart(world, thread L, arg world).
  5. ControlThreadStart(world, thread I, arg world).
- Thread L (world load, runs once):
  1. Diagnostic line (text not given).
  2. If flag 481 is clear: SlotLoad(world, the stored slot, set 0, 1, 1).
  3. Else if flag 482 is set: SlotLoad(world, the stored slot, set 0, 1, 1).
  4. Else: SlotLoad(world, the stored slot, set 1, 1, 1).
  5. Diagnostic line (text not given).
- Thread I (item watch, loops):
  1. If flag 481 is clear, end.
  2. If flag 482 is set, end.
  3. Loop:
     - If the hero has item 69 (DoesHeroHaveItem(69) returns more than 0): SetGameState(flag 482, 1).
     - Sleep(1 frame).
     - If flag 481 is clear: end.
     - If flag 482 is set: end.
     - Repeat.
- Update and exit: nothing.
- Confidence: High (the slot store is High, the slot number meaning is the SlotCreate numbering in Slot rules).
- Edge: the flag-482 branch of thread L and the loop check run in the same order as above; flag 482 ends the watch once set.

### 1156: Interior, library teleport and a slot-set chooser

- Family: Interior (no exit export).
- Init:
  1. Stores the world handle; diagnostic line (text not given).
  2. SlotAllocate(world, 1); SlotCreate(world, 2, slot table T2). The returned slot number is stored in a module cell.
  3. ControlThreadsAllocate(world, 1); ControlThreadStart(world, thread C, arg world).
  4. RegisterTeleportCallback(T0, index 0, kind 13, world).
- T0 (teleport 0): diagnostic line (text not given); AddBlackboardMessage(type 2, value 70, world 106); StartLeaveInterior(eWorlds 107, -1, -1, no color).
- Thread C (runs once):
  - If flag 31 is set: SlotLoad(world, the stored slot, set 1, 1, 1).
  - Else if flag 30 is set: SlotLoad(world, the stored slot, set 0, 1, 1).
  - Then SetToNormalCamera(10.0, 0.0, flag 1).
- Update: nothing.
- Confidence: High.

### 1159: Interior, teleport, slot chooser and a looping entity animation

- Family: Interior (no exit export).
- Init: stores the world handle; SlotAllocate(world, 1); SlotCreate(world, 3, slot table T3), with the returned slot number stored in a module cell; ControlThreadsAllocate(world, 1); ControlThreadStart(world, thread C, arg world); RegisterTeleportCallback(T0, index 0, kind 13, world).
- T0 (teleport 0): diagnostic line (text not given); AddBlackboardMessage(type 2, value 60, world 106); StartLeaveInterior(eWorlds 107, -1, -1, no color).
- Thread C (runs once):
  - If flag 31 is set: SlotLoad(world, the stored slot, set 2, 1, 1).
  - Else if flag 30 is set: SlotLoad(world, the stored slot, set 1, 1, 1).
  - Else: SlotLoad(world, the stored slot, set 0, 1, 1); LoopEntityAnim(entity 0, animation 97, 0.0 s, control 1).
  - Then SetToNormalCamera(10.0, 0.0, flag 1).
- Update: nothing.
- Confidence: High.

## Edge cases and error handling

- Two registrations on the same trigger index (1255 index 4): the later one replaces the earlier one (SPEC-0030 row 250). High.
- Callbacks that do not test the entered state fire on every engine call for that trigger (685, 1255). The clean side should follow the same rule.
- Slot word with file value -1 (851, the only load of this set through a slot word that no reachable part of the module writes): the loads do nothing in the original (Slot rules). High for the reachable paths.
- Slot numbers at or above the created count do not spawn characters on activation (Slot rules). Loads of slots without a SlotCreate buffer are not covered here (Open question 13).
- Debug lines are printed only in the original's debug path. The clean side may print them or skip them; they carry no game state.

## Test vectors

- 1170, activate world with blackboard value 20: MoveHero(locator 1, 1, 1, 1), then SetToNormalCamera(10.0, 0.0, 1).
- 1170, activate world with blackboard value 10: MoveHero(locator 0, 1, 1, 1), then SetToNormalCamera(10.0, 0.0, 1).
- 816, pickup with item id 76: SetGameState(flag 517, 1). Pickup with item id 75: nothing.
- 699, init with flag 265 set and flag 220 set: SetWorldDisplayMask(world, 2), SetGameState(266, 1), SetWorldCollisMask(world, 10), the bitwise OR of 2 and 8.
- 1254, world-swap event code 1253: MoveHero(0, 1, 1, 1) first, Wait(3.0 s) last before ReturnToNormalCamera(-1, 1).
- 1074, init with slot table T1: SlotCreate returns 0 (the first slot of the world); the thread L load uses slot 0.
- 1255, after Init the hero enters trigger index 4: the routine that runs is TM5b (the second registration), giving RegisterTransitionMap(5); index 3 runs TM5 and also gives map 5.

## Open questions

1. Resolved: the values used on disc and how the host reads each are in SPEC-0030 (Layout 2.1, Open question 5): WAIT is 0 or 1 (tested nonzero), CONTROL is 0 or 1 (compared with 1), and CAMERA_WAIT is 0, 1 or 2 (the play end mode).
2. Resolved: Wait(s) yields trunc(s × 60) ticks, one single-precision multiply by the rate word 60 written at start-up (SPEC-0030 Open question 1; SPEC-0007, Rate selection).
3. Resolved: an eWorlds id is an area slot; the slot names its world pack and the pack's map area gives its name (SPEC-0006 "Area names", SPEC-0051 "Map area names"). 5 = pack 691 (transition), Farm Road; 68 = pack 909 (exterior), Ellis Family Farm; 72 = pack 951 and 73 = pack 955, both interiors inside Ellis Family Farm (SPEC-0045, pack 909 doors; High); 106 = pack 1098 (exterior), no map area of its own, and its script registers map areas 48 and 49 (Meridell Castle 1st Floor and Meridell Castle 2nd Floor); 107 = pack 1121 (transition), Meridell Castle 2nd Floor (High).
4. Resolved: a later registration on the same index replaces the earlier one, because registering writes the callback into that index's trigger record (SPEC-0030 row 250). In 1255 index 4 runs TM5b; both routines set map 5 (High).
5. Resolved: the trigger state values are 0 idle, 1 first fire, 2 repeated fire and 3 decaying toward 0 (SPEC-0030 Layout 3). The pilot uses only "1" (first tick of contact) and "non-zero", which is true for the ticks of contact and the short decay after the last fire (High).
6. Resolved: the three codes are compared inside world-swap scene callbacks (registered by the transition packs 797, 908, 1055 and 1254), and those callbacks run only when a script starts a transition scene. No live script call does that, so the callbacks never run and the codes 1253, 753 and 1098 are never delivered in play (High; SPEC-0030 rows 268, 273 and 283). The numbers look like world-pack numbers (1098 is an exterior pack, 1253 sits next to the transition pack 1254), and the 1253 branch of pack 1254 reads as a sinkhole fall (hero to locator 0, camera 6, hero animation 136), but this is moot while the callbacks are unreachable (Medium).
7. Module 851: whether any host path starts its chooser, and what the intended slot id is. Whether the first routine that holds its three loads is ever called is not established; the no-op verdict holds either way.
8. Resolved for quest flags: a flag that is a quest row's start or done flag takes that row's title (SPEC-0049 "Titles and flag glosses"). 259 = start Repair Brightvale Bridge; 265 = done Repair Brightvale Bridge; 273 = done Get Illusen's Charm; 481 = start A Magic Skull; 482 = done/start A Magic Skull; 516 = start Mayor's Broken Watch; 517 = done/start Mayor's Broken Watch (High). The other flags used here (e.g. 39, 53, 57, 58, 220, 258, 266, 274, 659) are not quest start or done flags and have no name source on disc.
9. Module 1080: ReturnToNormalCamera gets 2 on every path.
10. Resolved: transition map ids are map-area ids, the name the map page shows, not world ids (SPEC-0051 "Map area names"). 43 = Ruined Caves; 54 = Oubliette; 5 = Werelupe Burrows Upper; 7 = Werelupe Burrows Lower (High).
11. SlotCreate's return value is the slot number of the buffer it reserves, in creation order, or -1 when no buffer is left (Slot rules). The call path does not remap -1 (High). A load through a slot word (file value -1) does nothing only where no reachable part of the module writes a slot-create result into that word (module 851, dormant). This also answers SPEC-0037 open question 1.
12. The fifth argument of SlotLoad (activate) is 1 in every pilot load.
13. A slot below the created count that has no SlotCreate buffer: the region read and its result are not covered here (Open).
