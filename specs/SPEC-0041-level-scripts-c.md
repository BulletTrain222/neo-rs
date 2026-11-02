# SPEC-0041: Level scripts (25 modules)

- Answers: proactive (level script behavior; 25 modules); R-0057 (gate helpers A to C, 1392); R-0058 (branch B object poses, 1371); R-0059 (strike callback attack word and strike boxes, 1382); R-0064 (985 activation filter); R-0078 (1324: plate geometry, the bells' floor and push counts); R-0081 (pack 951 has no placed-effect activation call), R-0093, R-0095, R-0098, R-0103 (1324 flag 239 live completion route; 1382 trigger 2 dimensions); proactive (Gelert release); proactive (arena pass 4); R-0135; R-0145; R-0183 (1363 proximity set choice and counter); R-0175 (pack 1428 boss-room return); R-0176 (pack 989 Update effect moves); R-0171 (985: first binding and clone wake), R-0163 (pack 800: leaving the raider leader's fight), R-0164 (pack 800 stage D pickup arguments and the writers of level states 1 and 3), R-0192 (1353 signs, 1428 CB6 aim override, 1096 aim target 1), R-0197 (1247 and 1249 object-swap scenes, plate gates and what the burrows need)
- Status: Draft
- Disc build: SCUS-97367 v1.00 (same fingerprint as SPEC-0001)
- Confidence: per section. Host call names and argument order: High. Trigger record state values (1 = entered): Medium-High. Argument values per module: High unless marked. Slot loads: the slot is the value stored by the module's slot-create call (Medium-High for the store; the meaning of the slot number is open), except the direct-return loads in 1176 and 1371 (Medium); the activate argument is true in every known load except one false load in 1096 (High). Some modules use switch dispatch (a computed index selects one of several branches), so their switch-selected branches are Medium at most. Sections marked Low or "partly described" have unknown branches or arguments and are listed under Open questions.

## Summary

This spec extends SPEC-0030 and SPEC-0032 to 25 more level script modules. The modules covered are ranked 31 to 55 by code size among the distinct level modules that are not in the pilot (SPEC-0032, 20 modules) or SPEC-0037 (30 modules) or SPEC-0039 (30 modules). The 25 modules are 19 Exterior, 4 Interior and 2 Transition. All 25 export an Exit name. Several are large: 1421, 1423, 1353, 1363 and 1090 each register many triggers or callbacks, and their per-tick routines run long decision trees. For those, this spec gives the behavior as rules and action summaries, not as a full transcription.

Resource numbers in this set of modules (world-pack resource numbers, SPEC-0030 section 1): 742, 800, 822, 951, 985, 989, 1078, 1090, 1096, 1130, 1176, 1247, 1249, 1324, 1353, 1363, 1371, 1382, 1392, 1394, 1409, 1414, 1421, 1423, 1428.

Times are seconds unless marked as ticks or frames. The game tick is 60 Hz (SPEC-0007), so 60 frames = 1 second.

## Where it lives on the disc

- Each module is the level-script chunk (type 37) of a world-pack resource in GAMEFILE.DAT, found by resource number as in SPEC-0032. No module in this set of modules repeats the payload of another, so each resource below is a distinct module.
- Imported host calls are matched by name against the host script table, as described in SPEC-0030.
- The module's three export names identify the family: Exterior, Interior or Transition. Every module in this set of modules exports Exit as well as Init and Update. Whether an Exit body does anything is stated in each section.
- Module data holds small integer tables (slot tables, scene tables), debug strings and pointer cells. Only slot tables are needed by the clean side, and each section says where one is used.

## Load and unload plumbing

- The load and unload hooks of these modules are not described here. The pilot (SPEC-0032) has empty start and stop lists in all 20 of its modules. Confidence for this set of modules: Low.

## Host API used by this set of modules

Argument conventions are the same as SPEC-0032 (WAIT 0 or 1, CONTROL and CAMERA_WAIT as plain integers, boolean 0 or 1, Sleep in ticks, Wait in seconds x 60). Call names used by this set of modules, :

QueryGameState, InitObjOn, LevelMsg, DeactivateObj, RegisterScriptTrigger, PlayAnim, PlayCamera, StopEffect, DebugPrintf, ActivateObj, Wait, LoopEntityAnim, AddAnimObjSound, StartEffect, Sleep, ReturnToNormalCamera, MoveHero, SetGameState, PlayEntityAnim, FadeScreenOut, FadeScreenIn, SetLevelState, IsButtonPressed, KillSound, RegisterEntityProximityCallback, PlaySound, DisplaySquareButtonContext, SlotLoad, ControlThreadStart, SetConversationTarget, EndConversation, PlayHeroAnim, Random, WalkEntity, QueryLevelState, MoveEntity, LoopAnim, DeactivateEntity, ReleaseEntityControl, SlotCreate, AddBlackboardMessage, SetAI_Agro, GrabEntityControl, SetToNormalCamera, LoopSound, GetNumberOfFramesPerSecond, PlayFMV, CharacterNavmeshAssign, SetHeroSwapMode, ControlThreadsAllocate, BossFightTweakHero, WalkHeroAndOrient, SlotAllocate, WalkEntityAlongPath, IsEntityWalkToPointActive, IsEntityTalking, GetSystemTime, StartExteriorReplacementLoad, RegisterActivateWorldCallback, LetterBoxOff, IsObjActive, IsDebugMode, BlockPlayerControl, SlotReset, ReturnEntityAnimToIdle, LetterBoxOn, DoesHeroHaveItem, SetHeroEquippedState, RegisterTeleportCallback, PlayLookAtAnimObjCamera, GrabHeroControl, DeactivateTimer, SetAnimObjAdditionRotY, SetAI_Behavior, SetWorldCollisMask, SetTimerColor, SetDefaultEntityIdleAnim, ResetAnimObjOffsets, IsObjAnimating, GetBlackboardMessageValue, ActivateTimer, ReturnHeroAnimToIdle, GetScriptGroupInstance, SwitchHero, SetAnimObjPositionOffset, MoveMovableBlockToLocator, IsAI_Dead, ActivateMovableBlock, WalkEntityAndOrient, ReacquireMovableBlockHeightPatch, GetAssetGroupCharacterRange, DisableVillagerSocialize, CharacterActivate, WaitTillLeaveTrigger, SetObjToTime, GiveItemToHero, GetAnimObjWorldPos, WalkHero, StartEnterInterior, StartCameraShakeMode, RestorePlayerControl, RemoveAllOfItemFromHero, RegisterOnEnterCallback, RegisterAIPlayerActivateCallback, IsHeroOccupied, IsHeroDead, ActivateEntity, memset, SetTargetActive, SetSpeedFactor, SetReturnWorldForFMV, RegisterMovableBlockSoundIndex, RegisterHeroDeathCallback, MoveEffectToAnimatedLocator, LoopHeroAnim, ItemActivate, HasMoviePlayed, WhichHeroIsActive, TriggerActStart, SetGameOver, RegisterItemAskAboutCallback, RegisterHeroUseItemCallback, NeedHeroAlive, IsHeroSwapInProgress, IsDialogActive, IsAnimatedCameraActiveAndFinished, IsActStartDone, GetHitPoints, GetCollisInfo, DisableInvisibility, ChangeMortogTimes, StopAnim, StartLeaveInterior, StartHeroLongFall, SetWorldLoadOverRide, SetMaxActiveAI, SetExternalMissileTarget, SetAI_ConversationTarget, RemoveItemFromHero, RegisterHeroPickupItemCallback, PlayTrackHeroCamera, PlayMainSound, PlayLookAtPlayerCamera, NumMortogsOnTarget, LockChest, IsObjAnimFinished, IsBreakableIntact, InitMortogBehavior, FrontendHUDReset, FloatToChar, Draik_SetBreakableIndices, AddItemToWorld.

Rows for these names in SPEC-0030 section 2 give the catalogue effect and confidence. Calls not in that catalogue are not used here. Where a module section gives a call's effect, it follows the section, not the catalogue.

Slot rule (SPEC-0030 section 5). SlotCreate loads nothing. It returns the number of the slot it reserves, or -1 when no slot is free. A module that stores that returned value and passes it to SlotLoad or SlotReset loads into that slot. Every slot call rejects a slot number outside the valid range. The slot argument of a load is read from a module data word, not passed as a fixed value, except where it is a SlotCreate return passed directly (1176 and 1371; see those sections, Medium). That word holds -1 in the file until reached code stores a SlotCreate result into it, so the load reads the stored slot. Two words are exceptions: 1096 and 1382 hold 0 in the file, and each is also stored by a SlotCreate result, so the load still uses the stored slot. There is no remap of -1 on the call path (High). A slot word (file value -1, never stored in reached code) gives a load that does nothing in the original (High for the code; no module of this set of modules is in that case).

## Shared rules used by the per-module sections

- A trigger callback runs when the engine fires its index. Callbacks that do not test the entered state fire on every engine call for that trigger (SPEC-0032 edge cases).
- Dormant routines are routines that no callback, thread or entry references. A dormant routine does not run. Routines named in a module's load-time constructor list are run by the load hook and are not dormant (SPEC-0030). Sections say when a routine is dormant. Reachability here is static, and flag gates decide whether a routine that is reached in the code is reached in play.
- Helper routines shared by several callbacks are described inside the section of the module that holds them, and are named by their role, not by number.
- Entity arguments: where a walk, idle, animation or entity-control call takes its entity from a record field, the entity is read from that record at run time, not a constant. This applies to the entity-walk routines of modules 951, 1409, 1414, 1421 and 1423 (on some paths; other paths are not known).
- Float arguments: a float argument (speed, offset or time) given as one value is the value known on the paths shown. Several are known on some paths only: SetAnimObjPositionOffset in 1363 and 1394, PlayAnim speed in 1392, and SetObjToTime in 1324.
- Gated calls: a host call reached only when a state or module-data gate holds is written as "reached when <gate>", with Medium or lower confidence where the gate is not resolved. Debug-only calls run only under the debug query, which is false in play.
- Module data words are live state: they hold their file values when the module is loaded and are then read and written by the module's routines (counters, once-words, step bits, latches). A gate on such a word is resolved by its file value and its writers, which the sections name. Lifetime (SPEC-0030 section 1.1; High): an Exterior module's words survive trips into an interior and back, swaps between the exterior and transition buffers, and resident streaming; they go back to their file values when the exterior is loaded by the world driver, when play streams to another exterior, and on a new game. An Interior module's words are fresh on every entry. Init runs only at the start of a lifetime, so it sees the file values, except for words that the module's load-time constructor writes just before Init (SPEC-0030; in this set only 1324's scene-done word, which takes the value of flag 239). "Once per load" in the sections means once in this lifetime.
- Debug-print helpers: several modules call a small helper that queries debug mode, prints and returns no result. Code after such a call is not debug-gated; it runs in play.
- Switch dispatch: a routine registered on several trigger indices often branches on the trigger index through a table; every case is an ordinary branch of that routine.
- Unregistered callbacks: a callback found by scanning the module's data rather than a registration seen in the module is not described as behavior. Each module that has one lists it under Open questions, with its registration marked unconfirmed.

## Modules

Each module section lists: family; Init steps in order; each callback or trigger with its index and what happens when it fires; threads; Update and Exit; edge cases; confidence; open questions. Sections are in resource number order.


### 742: Transition, proximity conversation and two watcher threads

- Family: Transition.
- Exit: present, no host calls.
- Update: present, no host calls.
- Hooks (load and unload): present, no host calls.
- Init, in order:
  1. Registers an entity-proximity callback for proximity id 7 (bool argument 1), named P7 below.
  2. Allocates 2 slot sets.
  3. Creates one slot (slot set argument 1) and then a second one. Each returned slot number goes into its own module word (the first and second stored slot); thread A loads into them.
  4. Allocates 2 control threads.
  5. Starts thread A and thread B, each with the world as its argument.
- P7 (proximity 7; the callback record's first word is the entity handle used below):
  - Both branches set Haplo as the conversation target first, and show one line.
  - If flag 562 is set: SetConversationTarget(entity, true), then LevelMsg(line from the rotation 0, 1, 2, wait 1), then EndConversation(entity). The rotation is module data that starts at 0 and is not saved, so the first talk shows line 0, the next line 1, then line 2, then line 0 again, and so on.
  - If flag 562 is clear: SetConversationTarget(entity, true), then LevelMsg(line 4, wait 1) on every talk.
- Thread A (flag watcher and slot loader; runs once). Before each load it calls a small debug-print helper that queries debug mode, prints, and returns no result; nothing tests that result, so the loads run in play:
  - Flag 76 set: SlotLoad(the first stored slot, slot set 0, keep true, activate true). Then, if flag 561 is set and flag 562 is clear: SlotLoad(the second stored slot, slot set 1, keep true, activate true).
  - Flag 76 clear: reads flag 75, and loads SlotLoad(the first stored slot, slot set 0, keep true, activate true) whether flag 75 is set or clear.
- Thread B (AI death watcher):
  - Gets the entity-id range for region group 1, which is entity ids 6 to 7 in pack 742. It checks only the first id, 6: the Giant Slorg at ordinal 0 of character region resource 744. It prints a debug line before polling. If the Giant Slorg is not dead, it sleeps 1 frame and retries; once it is dead, it sets flag 562. Haplo (entity 7) is in the same range but is not checked.
  - If dead: SetGameState(562, true).
  - If not dead: Sleep(1 frame), which implies a per-frame poll.
- Dormant routines: three contain host calls: a float-to-text conversion (debug output), a debug-mode button check (button 7 gated on debug mode), and a debug-mode check. The rest have no host calls. None is referenced by a callback or thread argument, so none runs.
- Helper routines: several contain only debug-mode checks. Callers are not shown.
- Threads: A and B, both started in init.
- Edge cases:
  - P7 can fire repeatedly; it changes no flags itself. Flag 562 is set by thread B and read by P7.
  - Thread A's slot loads do not depend on debug mode: set 0 always loads once, and set 1 loads too when flag 76 and flag 561 are set and flag 562 is clear.
- Confidence: High for init, P7, thread A and thread B's checked entity (pack group and kind-3 names). Medium-High for the poll loop and its order (from the sleep and dead branch).
- Open questions:
  - Resolved: thread B checks entity 6, the Giant Slorg in region resource 744; entity 7 Haplo is not checked.
  - What proximity id 7 is.
  - Meaning of flags 75, 76, 561, 562 (meaning unknown beyond their use here).
  - Whether any dormant routine is started by an outside name lookup.


### 800: Exterior, debug cutscene test chooser and boss-stage threads

- Family: Exterior.
- Exit: present. Exit checks whether the hero has item 9 (Raider Boss Eye Patch); if not, it gives item 9 to the hero (quantity 1, no notification).
- Init:
  - Allocates the slot table (4 entries) and creates slot 8 from a constant table.
  - Reads game-state flag 55.
  - Both branches, unconditional at this level: allocate 4 control threads, start control thread CB1, and register trigger index 1 with callback CB5.
  - If flag 55 is clear: creates three more slots (1 entry each).
  - If flag 55 is set: calls InitObjOn(object 0, not forced).
  - Only the slot creates and the InitObjOn call differ between the branches.
- CB1 (control thread, started at init; reads game-state flags 58, 57, 56, 55 in that order):
  - Each load uses the single slot number stored from Init's SlotCreate for the 8-entry table. The values below are set indices within that slot, not slot numbers.
  - If flag 58 is set: loads set 4 (wait mode 1, forced).
  - Else if flag 57 is set: loads set 3.
  - Else if flag 56 is set: loads set 2.
  - Else if flag 55 is set: loads set 1.
  - Else: loads sets 0, 5 and 6.
  - Debug lines are printed in every branch. The only host-visible effect is the slot load.
- Once-words: four module words (file value 0) mark stages A, B, C and D as done; each stage body sets its own word to 1 at its end. Each stage runs at most once per module load (High).
- CB5 (trigger callback, registered at init on trigger index 1): runs only when the trigger object's state word is 1 and its flag word is non-zero, flag 55 is clear and the stage A once-word is 0. It runs the boss-stage A helper (below), which starts CB2 at its end. This is the route by which the stage chain runs in play (High).
- CB2 (control thread, started from boss-stage A and from CB5; runs when both IsAI_Dead(10) and IsAI_Dead(11) are true and the stage B once-word is 0):
  - Prints one debug line. Then runs the stage B body (below) in full.
  - If either entity is still alive, it sleeps 1 frame and exits.
- CB3 (control thread, started from stage B and from CB2; runs when QueryLevelState(1, 0) is non-zero and the stage C once-word is 0):
  - Runs the stage C body (below) in full.
  - If the level state is zero, it sleeps 1 frame and exits.
- CB4 (control thread, started from stage C and from CB3; runs when QueryLevelState(3, 0) is non-zero and the stage D once-word is 0):
  - Runs the stage D body (below) in full, then sets flag 200 to true and flag 55 to true. This is the setter of flag 200 (High).
  - If the level state is zero, it sleeps 1 frame and exits.
  - Level states 1 and 3 are not set by this module (stage B sets level state 0 and stage C sets level state 2); the Chieftain's controller sets them (boss fight instance 2, mode 0; R-0164, High). State 1: in stage 1, on the first tick after the hold on which its health ratio (current health over 300) is below 0.5, that is health 149 or less (150 is exactly 0.5 and does not post), from any state including attacks in progress; it enters its transition state and posts event 1 once. State 3: in stage 2 (which begins when level state 2 is set by stage C), on the first tick on which the ratio is 0 or less, that is health 0; it enters its death state and posts event 3 once. In stage 1 there is no death test: a Chieftain brought to 0 health in stage 1 waits for level state 2 and then posts 3 on the next tick. Test vector: health 300 to 151: no post; 149 (after a hit of 2 from 151): state 1 posted on the next controller tick; stage C sets state 2; later health 0: state 3 posted on the next tick, and CB4 (stage D) starts on the pass after.
- Update (runs every tick; every host call sits under the debug gate):
  - Gate: IsDebugMode must be non-zero. The debug query is false in play, so the calls in this block run only under the debug query and do not run in play. With debug off, nothing happens.
  - Then a debug-only button chooser reading buttons 0, 1, 3 and 2 as held (pressed) or released. Each combination runs the shared cut-in routine (fade out 0.7 s, camera shot, fade in 0.4 s; once or twice, camera index varies by combination) and then one or more stage helpers:
    - Button 0 held (any state of button 1): boss-stage A, which starts CB2 at its end.
    - Button 0 and button 1 held: also stage B (which starts CB3).
    - Button 0 held, button 1 released, button 3 held: stage C (which starts CB4).
    - Button 0 released, button 1 held: stage B. If button 3 is also held, stage C runs too.
    - Button 0 and button 1 released, button 3 held: stage C.
    - Button 2 held (and not both 0 and 1): stage D is added on top of whatever else ran. Stage D has no thread start.
    - All of buttons 0, 1 and 3 released with button 2 held: the cut-in routine and stage D only.
    - All released: nothing runs.
  - Update does no per-tick state writes outside those calls.
- Stage A (boss-stage A, run by CB5 in play and also called from update under button 0; ends by starting CB2):
  - Block player control, boss fight tweak (instance 2, stage 0), reset slots, move entities 10 and 11 to positions 25 and 26, set idle animations on 9, 10 and 11, a letterbox on, camera shots (6 at 0.8 s, 13 at 1.6 s, 11 at 1.0 s) with fades between, walk the hero to point 13, dialogue lines 0 to 6 (each with the conversation target set and ended; camera shots 6, 10, 8, 9, 8, 10 and 11 come one before each of lines 0 to 6, and camera shot 7 follows line 6), a loop on entity 10, a 45 frame sleep, then the thread start and the stage A once-word set to 1.
- Stage B (the body CB2 runs; also called from update under button 0 and button 1 cases):
  - Fade out 0.5 s, reset slot 6, letterbox on, camera shot 10 (1.0 s), fade in 0.5 s, idle loop on entity 9, dialogue line 7, fade out 0.7 s, camera shot 14, fade in 0.4 s, idle loops on entity 9, dialogue line 8, a one-shot animation on entity 9, return to idle, set level state 0, letterbox off, return to normal camera, then starts CB3 and sleeps 1 frame.
- Stage C (the body CB3 runs; also called from update):
  - Fade out 0.5 s, hero move, idle loop, entity 9 moves to position 17, slot 7 load, fade in, camera shot 17, idle loop on 9, dialogue line 9, fade out 0.7 s, camera shot 15, fade in, entities 16 to 12 walk to positions 20 to 24 (1.1 s steps with sleeps 5, 40, 25), dialogue line 10, AI aggro set on entities 12 to 16 (the agro call puts them into the alerted search, SPEC-0038 Behavior 5, so they attack only after their next notice test), a 1 s wait, fade out 0.5 s, return to normal camera, release control of entity 9, set level state 2, starts CB4.
- Stage D (the body CB4 runs; also called from update under button 2):
  - Fade out 1.0 s, idle loops on entities 12 to 16 (animation 1), entity 9 moves to position 17, hero move, idle loop on 9, camera shot 13 with fade in/out around it, dialogue line 11, camera shot 19, 1 s wait, entities 12 to 16 walk to positions 27 to 31 (1.1 s steps), 1.5 s wait, slot reset, dialogue line 12, a one-shot animation on entity 9, a 55 frame sleep, then activates placement id 12 (R-0164, High for the arguments, Medium-High for the rest). The call is the placement-id form of ItemActivate (SPEC-0030, row 202): id 12, snap to ground 1, placed 1, position override none (0), facing override none (0); the lifetime is the default (placed items never expire). Pack 800's placement-id table entry 12 holds region byte 5 and ordinal 0: the first created slot whose last load recorded set 5 (the Chieftain's own slot; it is loaded by the opening with activate false, so its item placements were not spawned at load), and the item placement with ordinal 0 of that region (resource 806). That record has item key 9 (Raider Boss Eye Patch), flag byte 0, drop group -1, luck gate 0, count 0, position (486.1, 297.2, 2392.5) and facing (0.5425, 0, -0.8396). Because the luck gate is 0 and item 9 has no quest-style veto (it is a quest item, not equipment: Medium), the scripted add spawns the item at that point; the snap flag moves it to the ground plus 0.5 (basin floor about 296.9, so about 297.4); it is a static placed item that stays until picked up. The slot's region must still be loaded: the later slot resets in this stage are for the archers' slot, not the Chieftain's. The spawned point lies 14.2 from locator 14 (the hero's place in this scene) and 19.6 from locator 17. If the hero already holds item 9 the carry limit decides (Medium; the exit also gives item 9 if missing). Then the script runs the boss fight tweak (instance 2, stage 1), ActivateObj(object 0, not forced), SetGameState(55, true), and the stage D once-word set to 1. Ends without a thread start.
- Arena facts for pack 800 (script placement of the hero and actors; High for the locator numbers, Medium-High for the geometry):
  - Trigger 1 is a box of 27.53 by 13.71 by 10 units with low corner (445.67, 302.22, 2356.44), long side along (-0.45, 0, 0.89), standing on the approach ramp just west of the basin. The world spawn (locator 0, (419.99, 304.0, 2361.48)) is outside it. The CB5 gate (flag 55 clear, stage A not done) is checked each time the hero's body is inside it.
  - Stage A: the walk is WalkHero to locator 13 (459.8, 304.34, 2377.62) with the caller waiting. Near its end the hero is placed by MoveHero at locator 14 (473.16, 296.31, 2386.64) facing (0.779, 0, 0.627) (position and facing, control 1), the camera returns to the normal follow mode with release (the hero becomes free), entities 10 and 11 get SetAI_Agro, the weapon-ready word is set, and the letterbox goes. The only BlockPlayerControl call of the module is at the start of stage A. Entity 9 is not moved by stage A or stage B: it stays at its character record position (513.613, 306.913, 2411.887) on the raised rim fragment, held in stage 0.
  - Stage C moves the hero to locator 14 (MoveHero) and entity 9 to locator 17 before loading set 7. Stage D again places the hero at locator 14 and entity 9 at locator 17. In the scenes entity 9 loops animations 8, 31 and 32 and plays one-shot animation 4 (end of stage B) and animation 2 (stage D).
  - The basin: walking mesh x 458.0 to 516.9, z 2363.0 to 2425.8, floor y 296.86. Locators 20 to 24 and 27 to 31 are on five separate rim meshes about 10 units higher; locators 13 (approach ramp) and 14, 17, 25 and 26 (basin floor) follow from the same data. No module call changes music, plays a stinger or registers a hero death callback. After flag 55 is set, CB1 loads set 1 only (the single character of region 802).
- Encounter population for world pack 800 (entity table and enemy-row data):
  - Entities 0–2 are three ambient Ixi Henchmen. They are loaded with set 0 and are not part of the intro death gate.
  - Entity 9 is the Ixi Chieftain. Entities 10–11 are two Ixi Henchmen loaded with set 6. Both use the close-combat strafe variant (row 160; 65 HP). The opening progression gate waits only for these two entities to be dead.
  - Set 7 introduces entities 12–16, five Ixi Henchmen using the ranged keep-distance variant (row 157; 65 HP). The stage-C script explicitly sets them aggressive after their entrance sequence. That agro call puts them into the alerted search (SPEC-0038 Behavior 5), so they attack only after their next notice test, not at once (SPEC-0014, Minion AI).
  - Row 157 has two projectile attacks and one melee attack; row 160 has three melee slots, of which the second slot has zero selection chance. These enemies use the shared class-17 behavior described in the common-enemy specification. The pack-800 character-to-placement drop links make the ambient row-160 entity 0 and both row-160 gate Henchmen request item 150 (Red Juppie) on their first death tick; ambient entities 1 and 2 use group-0 drop placements, while the five row-157 wave Henchmen have no drop ordinal (SPEC-0021).
- Stage locator positions in world coordinates (x,y,z): entity 9 uses locator 17 at (501.491, 296.310, 2404.695). Intro entities 10 and 11 use locators 25 and 26 at (505.342, 296.310, 2400.324) and (499.512, 296.310, 2410.943). The five wave members enter in entity order 16 to 12 at locators 20 to 24: (506.575,305.824,2361.327), (516.515,307.144,2384.935), (512.512,306.962,2412.408), (494.978,306.962,2428.837), (469.990,306.962,2423.936). Finale staging uses entities 12 to 16 at locators 27 to 31: (511.458,305.824,2350.869), (528.143,307.144,2382.470), (527.099,306.962,2419.632), (498.754,306.962,2436.896), (465.906,306.962,2430.733).
- Threads: CB1 (at init, once), CB2 (from stage A or CB5), CB3 (from stage B or CB2), CB4 (from stage C or CB3). CB5 is a trigger, not a thread.
- In play the stage chain runs through the trigger route: trigger 1 (flag 55 clear) runs stage A; CB2 waits for AI 10 and 11 dead and runs stage B; CB3 waits for level state 1 and runs stage C; CB4 waits for level state 3, runs stage D and sets flags 200 and 55 (High).
- Edge cases and error handling:
  - With the debug gate off, the update does nothing; the trigger route above still runs the whole chain.
  - Leaving the fight (R-0163; High for the module, Medium for the world swap): nothing in this module or its geometry holds the hero in the fight. Its only trigger is the start volume (index 1), and its only BlockPlayerControl call is at the start of stage A. The basin's one entrance, the approach ramp from the west, is also its exit, with no wall across it. Walking back down the ramp past the pack's edge makes the neighbouring transition pack current (pack 837 or 851 by the cell under the hero; neighbour list of this pack: positions 1 to 5 world 309, positions 6 and 7 world 306), with this pack kept resident as the second world. While it is the second world its scripts, triggers and control threads are frozen, but its characters keep updating and are released beyond 280 units from the hero. On return: resident and not released, the fight continues as it was (stage, Chieftain health, level states, once-words, dead Henchmen); a released Chieftain or Henchman is bound again at full health in its default state; if the hero went on to another exterior and this pack was replaced (pack 822 through pack 837), the pack loads again as new: flag 55 clear, once-words 0, everything at full health, the opening played again by trigger 1, kill bits of the fight forgotten. SPEC-0014 (pack 800, Leaving the fight) has the cases. Default if unresolved (rewrite choice): keep these three cases; add no wall or lock.
  - Several button branches can apply in the same tick. The update code does not make them exclusive, so a held combination can start more than one thread. Partly described.
  - Load and unload hooks have no host calls.
  - A dormant routine holds only text data. Whether anything references it is not established.
- Confidence: Medium overall.
  - High for init, exit, CB1, the CB5 trigger gate, the once-words, the flag 200 and 55 writes, and the debug gate on every update host call.
  - Medium for the stage bodies (the sequence is flat, and camera numbers are read from call arguments).
  - Low for the exact ordering when two button branches both apply (partly described).
- Open questions:
  - What the debug chooser is for: it may be a cutscene test harness, but that purpose is not established. Meaning of flags 55 to 58 unknown. The CB2/CB3/CB4 polling waits have no timeout; whether streamed character binding or a missing death event can leave the stage chain waiting indefinitely is not established.
  - The encounter roles and positions for entities 9–16 and locators 17, 20–31 are listed above. The role of hero locator 13, unused locators 18–19 and 32, and the semantic meaning of the slot numbers remain unknown.
  - Resolved: item 9 is the Raider Boss Eye Patch; exit gives one if the hero does not already have it.
  - Trigger object fields tested by CB5 (state word and flag word): meaning unknown. Resolved: stage-D activation ID 12 is a placement index, not an item number; pack 800 maps it through slot 5 to resource 806 record 0, whose item key is 9 (Raider Boss Eye Patch; SPEC-0021). The call's arguments, the placement it names and its position are given in Stage D (R-0164): the pickup is a static placed item at about (486.1, 297.4, 2392.5), picked up like any placed item. The pickup's drawn look is the item model of item 9 (SPEC-0021).
  - Resolved: the module-data gates on the thread bodies are the four once-words (file value 0, each set by its own stage), so the flag 200 and 55 writes run in play once per load.


### 822: Exterior, villager and trigger room with a flag-gated conversation

- Family: Exterior.
- Exit: present. It makes no host calls. Load and unload hooks exist and make no host calls.
- Init, in order:
  1. Checks whether debug mode is on (the result is not used beyond a debug print).
  2. Allocates one slot, creates a slot (id 14), allocates three control threads.
  3. Starts control thread CB1 (slot loader), unconditionally.
  4. Registers the activate-world callback CB2.
  5. Sets object 7 (bool 0).
  6. Reads flag 55.
     - Flag 55 set: sets objects 18, 1, 2, 3, 14 (bool 0).
     - Flag 55 clear: sets objects 18, 19, 20, 21 and 14 (bool 0).
  7. Unconditional at this level: a looped animation step on objects 18 to 21 (speed 1), and nine trigger registrations (indices 2, 3, 8, 0, 4, 1, 5, 6, 7).
  8. Objects 4 and 5 are set (InitObjOn with the bool argument false) only when flag 621 is set and flag 57 is clear. Flag 55 is not on this path; both of its values reach this step. On that path a local helper runs with argument 1. The helper sets one module data cell to 1, loops the animations on objects 18, 19, 20 and 21 (speed 1), and then sets objects 5 and 4 (InitObjOn, bool false). Its loop covers objects 18 to 21 only: it does not loop an animation on objects 4 or 5, and it starts no effect. If flag 57 is set, whatever flag 621 holds, a cleanup helper stops effects 55 to 60. If flag 621 and flag 57 are both clear, a debug line is printed and another helper clears that same module data cell, requests looping animation at speed 0 for objects 18 to 21, and stops effects 55 to 60. Neither helper sets objects 4 or 5 or plays an animation on them. The meaning of the module data cell and the level-data identities of these effects and objects remain unknown.
- Callbacks:
  - CB1, slot loader (control thread, started by init and by CB9). A zone word (file value -1) is written by CB9 with 2, 1 or 0. The loader picks one slot set and loads it into the stored slot (activate true, with a debug print), by the first matching row (High for the table of sets; the loader also waits on a busy word of its own, Medium):
    - Flag 58 set: set 13 if flag 531 is set and flag 532 is clear, otherwise set 12.
    - Else flag 57 set: set 3 if the zone word is 2, set 7 if it is 1, otherwise set 11.
    - Else flag 56 set: set 2, 6 or 10 by the same zone rule.
    - Else flag 55 set: set 1, 5 or 9 by the same zone rule.
    - Else: set 0, 4 or 8 by the same zone rule.
    - While the zone word still holds its file value -1, the "otherwise" choice applies (sets 11, 10, 9 or 8).
  - CB2, activate-world: reads blackboard values for world 60 and world 50 (type 2).
    - If the value for world 60 is 1: moves the hero to locator 2 (control 1) and returns the normal camera.
    - Otherwise, if the value for world 50 is 50: moves the hero to locator 6 and walks it to point 8 (control 1).
    - In both of these cases it sets a module word (the scene-done word, file value 0) to 1; CB7 does not run once it is set.
    - Always, at the end: adds blackboard messages setting world 60 and world 50 to -1, and clears the exit-lock word (file value 1) to 0, which arms CB3.
  - CB3 (trigger 2): when the trigger record's state word is 1, its object word is non-zero and the exit-lock word is 0 (that is, after the first world activation): ActivateObj(object 7, not forced), AddBlackboardMessage(type 2, value 50, world 60), StartExteriorReplacementLoad(world 60, -1, no colour), which loads pack 864 (High).
  - CB4 (trigger 3): empty.
  - CB5 (trigger 8): reads flag 55.
    - Flag 55 set: if button 0 is pressed, shows message 8; if not pressed, shows prompt context 4.
    - Flag 55 clear, flag 621 set: nothing happens.
    - Flag 55 clear, flag 621 clear, button 0 not pressed: shows prompt context 4.
    - Flag 55 clear, flag 621 clear, button 0 pressed: walks the hero to point 242, plays hero animation 140, plays animation 14, sleeps, sets flag 621, loops animations 18 to 21 at speed 1, plays camera 6 and 7, plays sound 14, kills sound 14, and returns the normal camera.
    - The walk and the flag-55-set messages also need the trigger record field tests (non-flag; the meaning of the fields is not described).
    Partly described.
  - CB6 (trigger 0): if the object is valid and its state is 1: prints a debug line and sets the world load override to 861. The setter ignores an argument of 193 or more (SPEC-0006), so this call has no effect. Pack 861 itself has an empty level script: its init, update and exit do nothing and it makes no host calls (High).
  - CB7 (trigger 4): conversation and cutscene. Runs when the record's object word is non-zero and its state word is 1, flag 57 is set, flag 58 is clear and the scene-done word is 0. Locks the letterbox and player control, plays camera 8 and then camera 9, sets the conversation target to entities 24 and 25 with messages 6 and 7, plays camera 10, plays entity animation 10 on 24 and 25, walks entity 24 to point 11 and entity 25 to point 12, sets AI agro on 24 and 25, returns the camera, and clears the letterbox.
  - CB8 (trigger 1): if button 0 is not pressed: shows prompt context 3. If pressed: shows message 9.
  - CB9 (triggers 5, 6 and 7): when the record's object word and state word are non-zero, sets the zone word by the trigger index (index 5: 2, index 6: 1, index 7: 0). If the zone word now differs from the last value the loader used, it stores the new value as the last value, resets the stored slot and restarts the slot loader CB1 (High).
- Helpers:
- Navmesh and villager step (Update, every tick; High): the module keeps a table of 68 navmesh ids, one for each entity number 0 to 67, which Init resets to -1. Each tick, for every entity whose table value is -1, the step asks for the closest navmesh id of that entity in the module's own world and stores the answer (an answer of -1 stays -1 and is asked again on the next tick, and a debug line prints when an id is first found). Then, for every entity whose value is not -1, it assigns that navmesh to the entity. Then, for all 68 entities, it sets the villager-socialize block. So every character of this area is held to its nearest navmesh once one is found, and none takes part in villager pairing.
  - Camera and sound helper: plays camera 6 and 7, waits, plays sounds 14 and kills sound 14, returns the camera.
  - Animation and effect helpers: loop and play animations on objects 4, 5, 10 to 13, 15 and 17 to 21, start effects 55 to 60, and stop them.
- Dormant routine: a conversation chooser (flag 59). If flag 59 is set, sets the conversation target on two entities, shows message 0, and ends the conversation. If clear, shows message 3. Nothing refers to it: no call, no address constant and no data word, so it never runs (High). A second unreferenced routine waits while flag 531 is set and flag 532 is clear, and sets flag 532 as soon as the hero holds three or more of item 80; nothing starts it, so this module never sets flag 532 (High that nothing refers to it).
- Update (one per tick): runs the navmesh and villager step above for entities 0 to 67.
- The nearest-navmesh query is the one in the step above: it runs only for entities whose table value is still -1, and the debug line after it only prints (High).
- Edge cases:
  - The trigger registrations in init are unconditional, so all of them exist from the start. The trigger bodies are the gated part.
  - CB4 is present but does nothing.
  - Init-step-8 vectors: (flag 621 clear, flag 57 clear) selects the speed-0 loop and effect-stop helper; (flag 621 clear, flag 57 set) selects effect-stop only; (flag 621 set, flag 57 clear) selects the object 4/5 setup helper; (flag 621 set, flag 57 set) selects effect-stop only. Flag 55 does not change these outcomes.
  - Update vector: entity 5 has table value -1 and the closest-navmesh query answers 3. The table value becomes 3, a debug line prints, and CharacterNavmeshAssign(5, 3) runs in the same tick. On the next tick no query is made for entity 5, the assign repeats with no change, and the villager-socialize block is set for all 68 entities every tick.
- Confidence: Medium. Init, update, CB1, CB2, CB3, CB6, CB8, CB9 and the thread are described in full. CB5, CB7 and the navmesh conditions are only partly known.
- Open questions:
  - Meaning of flags 55, 57, 58, 59, 621 and 531 to 532: meaning unknown.
  - Effects of the two other step-8 routines (used when flag 57 is set, or when flag 621 and flag 57 are both clear): not described here.
  - Trigger volumes for indices 0 to 8: from level data, ids not yet mapped.
  - Entities 24 and 25, characters 0 to 2, locators 2, 6, 8, 11 and 12, and point 242: from level data, ids not yet mapped.
  - Slot set numbers 0 to 13 (the numbers in the slot loader) and the slot-create table id 14: meaning unknown. The slot loaded is the value stored by the module's slot-create call.
  - Resolved: the conversation chooser has no reference of any kind (no call, address constant or data word), so no indirect call reaches it.
  - Resolved: the debug line is not gated by the debug query; it is a print with no game effect.


### 951: Interior, proximity conversations, a story thread and a per-tick entity wander

- Family: Interior (its Init entry).
- Exit: present, empty. The exit export runs no host calls.
- Load and unload hooks: present, no host calls.
- Callbacks: nine proximity-trigger registrations across three callbacks, one teleport callback, and one thread body.
- Placed-effect activation: Init, Update, the thread and all callbacks make no StartEffect or switch call. The type-40 lock-chest entry has no placed-effect index (R-0081).
- Init (in order):
  1. RegisterTeleportCallback(teleport callback, index 0, CONTEXT_MESSAGE 26, world). Unconditional.
  2. QueryGameState(121) into a local.
  3. Unconditional at this level: SlotAllocate(world, 1); SlotCreate(world, 3, a pointer to a module data cell); LockChest(world, chest 0, value 15, value 124); ControlThreadsAllocate(world, 1), then ControlThreadStart(thread body, world); and RegisterEntityProximityCallback for nine trigger indices: callback 1 at indices 0, 3 and 7; callback 2 at indices 1, 4 and 8; callback 3 at indices 2, 5 and 6. All with bool 1.
  4. The state 121 test selects only the object call: if state 121 is clear, InitObjOn(object 0, bool 0); if set, InitObjOn(object 0, bool 1).
  - Note: the slot, lock, thread and nine proximity calls run whatever state 121 holds. Only the object call differs.
- Update (every tick):
  1. If state 119 is set, read state 123. If 123 is clear and the hero has item 98 (DoesHeroHaveItem(98) != 0): one debug line, then CharacterActivate(world, the index from module data, 0) and CharacterActivate(world, the index from module data, 1). Nothing else in this block. A module latch word holds it, so the block runs once per module load (High).
  2. Then, unconditionally, the shared entity-control routine runs (SPEC-0043, section "Shared entity-control routine", which holds its rules; High). Its one record: entity 4, idle animation 83, interval 5 to 7 s, animations 80 and 74 (2.5 s), no waypoints.
- Proximity callback 1 (trigger indices 0, 3, 7). Entity from the proximity record at +0x0.
  - If state 147 is set: SetConversationTarget(entity, true); LevelMsg(message 0, no world, bool 1, WAIT 1); EndConversation(entity).
  - If state 147 is clear: LevelMsg(message 4, no world, bool 1, WAIT 1). No conversation calls on this branch.
- Proximity callback 2 (trigger indices 1, 4, 8). Entity from +0x0.
  - If state 147 is set: SetConversationTarget(entity, true); LevelMsg(message 119, no world, bool 1, WAIT 1); EndConversation(entity).
  - If state 147 is clear: LevelMsg(message 123, no world, bool 1, WAIT 1).
- Proximity callback 3 (trigger indices 2, 5, 6). Entity from +0x0.
  - If state 147 is set: SetConversationTarget(entity, true); LevelMsg(message 55, WAIT 1); EndConversation(entity).
  - If state 147 is clear and state 122 is clear, and the hero has item 98: GiveItemToHero(item 32, 1, false); SetGameState(121, true); SetGameState(122, true); LevelMsg(message 59, WAIT 1).
  - If state 147 is clear, state 122 is clear, and the hero does not have item 98: LevelMsg(message 60, WAIT 1).
  - If state 147 is clear and state 122 is set: LevelMsg(message 58, WAIT 1).
- Teleport callback (index 0, the teleport trigger, not a proximity trigger; High):
  - Reads states 119 and 122.
  - If state 119 is set and 122 is clear: LevelMsg(message 126, bool 1, WAIT 1).
  - Otherwise (state 119 clear, or state 122 set): AddBlackboardMessage(type 2, value 10, eWorlds 68) and StartLeaveInterior(eWorlds 5, -1, -1, no colour), which leaves to pack 691.
- Thread body (started from Init; High). It runs once and ends; there is no loop:
  1. If state 119 is set and 120 is clear: SetGameState(120, true); PlayFMV(movie 2, bool 1); GiveItemToHero(item 15, 1, false); FadeScreenIn(2.0, WAIT 0).
  2. Then, unless state 220 is set, the first case that holds: state 2 set and 147 clear: nothing. State 147 set and 148 clear: RemoveAllOfItemFromHero(item 1), GiveItemToHero(item 0, 1, false), SetGameState(148, true), PlayFMV(movie 6, bool 1). States 147 and 2 both clear: if 121 is set and 123 clear, SlotLoad(world, the stored slot, 1, keep true, activate true); else, if 119 is set and 123 clear, SlotLoad(world, the stored slot, 1, keep false, activate true) and CharacterActivate(world, the index from module data, 2).
  3. Last, in every case: the entity record of entity 4 is added (the add step of the shared entity-control routine, below) and SetToNormalCamera(10.0, 0.0, bool 1).
- Helper "shared loop" (called by the thread): the add step of the shared entity-control routine (SPEC-0043) for entity 4: its LoopEntityAnim(entity 4, animation 83, 30.0 s, CONTROL 1) is the add's idle-loop start, and the small helpers it calls append the record's animations 80 and 74 (High).
- Dormant routines.
  - One dormant routine only calls the same empty helper that is reached nowhere else (no host calls behind it).
  - Other dormant routines have no host calls. One has no references; whether the other has a reference is not established.
  - Init, update, triggers, the teleport and the thread have no established call or registration for any of them. They are not described as running.
- Test vectors:
  - Init with state 121 clear: object 0 initialised with bool 0; the nine proximity registrations and the thread start still run.
  - Init with state 121 set: object 0 initialised with bool 1; the same registrations and thread start run.
  - Proximity callback 1 with state 147 clear: LevelMsg 4 only.
  - Proximity callback 3 with state 147 clear, state 122 clear and item 98 held: GiveItemToHero(32), SetGameState 121 and 122, LevelMsg 59.
- Edge cases: the update's character-activate block runs once per module load, the first tick its item and state gates hold (latch word).
  - The message ids of the proximity callbacks (0, 4, 119 and 123) are not known on some paths; those paths also reach 1, 2, 120 and 121.
- Confidence: Medium. Init, the three proximity callbacks, the teleport, the update block and its latch, and the thread body are High. The per-tick routine is Medium (same as 1409).
- Open questions:
  - Meaning of states 119, 120, 121, 122, 123, 147, 148, 220 and 2 (meaning unknown; game-state ids not yet named).
  - Meaning of item 98 and item 32 (item ids not mapped).
  - Resolved: the thread runs once and ends.
  - Resolved: the gates of the update block, the thread and the teleport are given above. Resolved (proactive (open close-out); Medium-High): state 147 is the completion flag of quest row 26 ("Find Your Sister!") and is set only by the Act 1 end of the Juppie fight (module 1217); until then every proximity callback takes its clear side. State 122 is set only by the package exchange of proximity callback 3 itself (together with 121), so that exchange happens once, on the first visit with item 98 held while 147 is clear; afterwards, with 147 still clear, callback 3 shows line 60. State 119 and 123 are set only by module 909 and 120 and 148 only by this module's own code, so the update block's gate (119 set, 123 clear, item 98 held) is open only after module 909's step has set 119 and before it sets 123.
  - Resolved: the per-tick CharacterActivate block has a latch (once per module load).
  - Object id 0, chest id 0, movie ids 2 and 6, and message ids are from level data or the message table, not yet mapped.
  - Resolved: the teleport rule above is complete (leave with value 10 unless state 119 is set and 122 is clear).


### 985: Exterior, two fight branches with level-state cutscene chains and a debug skip

- Family: Exterior.
- Exit: present. Its body makes no host calls.
- Hooks (load and unload): no host calls.
- Init:
  - Registers the AI-player activate callback (unconditional). The callback is the streaming filter of the embedded block, asked with a record ordinal. Flag 281 clear: admits ordinals 5 (Illusen), 7 (Gnarfas) and 8 (the Werelupe King) and refuses 0 to 4 (Gelert Assassins) and 6; asking about ordinal 7 also assigns walk mesh 1 to entity 7. Flag 281 set: admits 0 to 4 and 6 and refuses 5, 7 and 8; asking about an ordinal below 5 also assigns walk mesh 0 to that entity (High). The five consecutive Gelert records occupy ordinals 0 to 4; ordinal 0 aligns with block slot 0, the primary boss addressed as entity 0 in H10, and ordinals 1 to 4 align with the four clone slots (SPEC-0013, 4.10). First binding in the flag-281-set branch (High for the rule; Medium-High that nothing binds earlier): the hero arrives on entry record 0, 303 units or more from every embedded record (squared distance above 78,400), so nothing binds at load; the arrival callback then moves him to locator 9, and the first streaming pass after that binds, nearest first with the lower ordinal winning ties, Gelert 0, 1, 2, 3 and 4 (identical point, squared distance 803.9 from locator 9) into slots 0 to 4 and then Illusen's ordinal 6 (squared distance 8,465.5) into slot 5, all six on that one pass because the default cap is 6. The clones wake together (SPEC-0013, 4.10). CB3 rule 4 then grabs entity 8 (the King) and holds it for the whole fight (SPEC-0019, Activation in play).
  - Calls a local helper that reads game-state flag 281. The helper's result decides the branch below (the branch tests the result, not the flag).
  - If the helper's result is set (flag 281 set): registers the activate-world callback CB4, a trigger callback CB5 on trigger 4, allocates 1 control thread, and starts control thread CB6 (the level-state chain below).
  - If the result is clear (flag 281 clear): registers the activate-world callback CB1, a trigger callback CB2 on trigger 4, allocates 1 control thread, and starts control thread CB3 (the opening wait loop below).
- Step bits: the module keeps two words of step bits (file value 0), word A for the flag-281-clear branch and word B for the flag-281-set branch. Bit n is set by the module's own set-bit helper; the bits are cleared only by the activate-world callbacks below. "A1" means bit 1 of word A.
- Update (runs every tick):
  - Gate: IsDebugMode non-zero and button 2 held. With the debug gate off, nothing happens. Only this Update copy is debug-only.
  - Debug skip: shows a debug line, stops effects 7 to 12, deactivates entities 7 and 8, sets game state 214 to true, sets a world collision mask, boss fight tweak (instance 1, stage 1), deactivates entities 0 to 5, and registers trigger callback CB7 on trigger 4. Two further tests in that branch are unknown.
- CB1 (activate-world callback, flag 281 clear): moves the hero to locator 9 (control 1), normal camera (3 s, 3.14), sets game state 212 to true. If game state 214 is clear: sets level states 0, 1 and 2 to false, sets a collision mask, and runs a boss fight tweak (instance 1, stage 0). Then, always: clears step word A and sets the arrival word (file value 0) to 1.
- CB4 (activate-world callback, flag 281 set): moves the hero to locator 9 (control 1), normal camera (3 s, 3.14), sets a collision mask, clears step word B, sets B0 and sets the arrival word to 1.
- CB2 (trigger 4, flag-281-clear branch): makes no host call; each time it fires it sets A2.
- CB5 (trigger 4, flag-281-set branch): makes no host call; when the record's object word is non-zero it sets B1.
- CB3 (control thread, flag 281 clear; the Gnarfas fight, see SPEC-0020 "Arena flow"). Each tick, the first rule that applies runs, then it sleeps 1 tick and repeats:
  1. Level state 0 set and level state 1 clear: plays movie 12 (wait), moves the hero to locator 10 (control 1), moves entity 7 to position 12, normal camera (3 s), sets game state 213, sets level state 1, sleeps 30 ticks, shows dialogue line 14 (wait).
  2. Level state 2 set and A0 clear: the fight-end helper: debug line, stops effects 7 to 12, deactivates entities 7 and 8, sets game state 214, sets A1, sets a collision mask, boss fight tweak (instance 1, stage 1), sets A0.
  3. Game state 214 set:
     - If A1 is set and movie 13 has not played: plays movie 13 (wait), fades out, moves the hero to locator 26, deactivates entity 5, activates entity 6, loops entity 6 on animation 3, normal camera (10 s), fades in (1.0 s). The collision mask change of rule 2 clears only bit 1: the north-east wall (gate 1, two plane records) is gone and the south-west wall (gate 2) stays. Trigger 4 is a slab 7.0 thick, 67.26 long (along (0.766, 0, -0.643)) and 31.39 tall from y 118.18, with its low corner at (-623.97, 118.18, -492.79) and its near face (normal (0.643, 0, 0.766)) 36.5 in front of locator 26 along the hero's facing (the facing and the normal differ by under one degree, and the hero's line meets the face at the middle of the slab). The north-east wall planes lie 5.6 in front of the near face and 15.8 behind it, and Illusen's second record stands 26.4 behind the near face on the same line. The scene gives no text, marker or Illusen line: the hero is placed facing the exit with control, Illusen loops animation 3 straight ahead, and the open passage leads to the slab. Pack 985 has no door or teleport plane, so trigger 4 is the only way out of this branch.
     - Otherwise, if A2 is set (the hero has entered trigger 4) and movie 14 has not played: sets game state 64, sets the return world for movies to world 74 (pack 956), and plays movie 14 (wait). This is the setter of flag 64 (High).
  4. Game state 214 clear: if the arrival word is set, sleeps 1 tick, moves entity 7 to position 11, clears the arrival word, grabs control of entity 8 and plays its animation 0 (no wait, control option 0, so the clip's end never releases him); then, in every case, assigns the navmesh for character 7 (flag 1). The one-tick sleep comes first, and the arrival word is set by the activate-world callback, which has already moved the hero to locator 9: the region pass of the next tick (which runs before the script step, SPEC-0007) therefore binds every admitted record within 280 units (Illusen 5, Gnarfas 7 and the King 8, all bound in one pass under the cap of 6) before the grab runs. Entity 8 is bound when the grab and the animation call run (High for the order, Medium-High for the cell gate of the record positions). The grab and the animation call on an entity that is not bound do nothing and store nothing (the host call looks the entity up and returns when no bound character answers), so an implementation must bind the three records before running this rule, not hold the call for later. The filter is asked for each unbound record on every region pass, not once at activation.
  - Level states 0 and 2 are Gnarfas's fight events (SPEC-0020); level state 1 is set here.
- CB6 (control thread, flag 281 set). Each tick, the first rule that applies runs, then it sleeps 1 tick and repeats. The even level states are set by these stages; level states 1, 3, 5 and 7 are set outside this module, by the fight (boss fight instance 5; Medium):
  - Stage 0 (level state 0 clear and B0 set, that is, after the first world activation): intro cutscene (helper H10): letterbox on, move hero to locator 9, boss fight tweak (instance 5, stage 0), fade in, block player control (hero -1), camera shot 25 (1.0 s), entity 0 animation 25; the activate-world arrival callback had already moved the hero to locator 9 before this thread stage, so H10 reasserts the same position before the boss presentation. The activation filter admits Gelert ordinals 0 to 4 in this branch, and the fight runs after arrival at locator 9. The exact scheduler ordering of the first 280-unit stream check relative to that callback is not established by the module script alone. Then, once the animated camera 25 has finished (30 ticks later it moves to shot 26 and 30 ticks after that the first line opens), runs a dialogue exchange (lines 0 to 5, with the conversation target set; every line waits for the player), camera shots 26, 27, 28, 29, 30 and 33 in order, entity 0 animations 1, 3, 4 and 2 (the scene waits for animation 2 to finish and for camera shot 29, which plays about 238 ticks), equip state on, a 0.5 s wait (30 ticks), letterbox off (1.0 s, no wait), normal camera with the hero freed, release control of entity 0, and finally sets level state 0 to true (the call that releases the Gelert from its wait state, SPEC-0013 G9; High). The thread then takes the later stages, because level state 0 is set and stays set, so the scene runs once per load.
  - Stage 1 (level state 1 set, level state 2 clear): cutscene H11: fade out, letterbox on, move entity 0 to position 27, move hero to locator 28, fade in, animation 25, camera 31, animation 4, dialogue line 8, animation 23, letterbox off, camera 29, release control of entity 0, set level state 2 to true.
  - Stage 2 (level state 3 set, level state 4 clear): cutscene H12, the same template as stage 1 with animation 5 and dialogue line 9, ending with level state 4 set.
  - Stage 3 (level state 5 set, level state 6 clear): cutscene H13, the same template with animation 6, a longer dialogue (lines 10 to 12, with cameras 32 and 26 between them) and an end state of level state 6.
  - Stage 4 (level state 7 set, level state 8 clear): cutscene H14: letterbox on, move entity 0 and the hero, animation 7, dialogue line 13, animation 24, fade out, normal camera, sets game states 284 and 301 to true, sets a collision mask, deactivates entities 0 to 4, stops effects 7 to 12, boss fight tweak (instance 5, stage 1), letterbox off, sets level state 8 to true, and assigns navmesh for characters 0 to 4.
  - No stage applies and level state 0 is set:
    - Game state 284 clear: assigns the navmesh for characters 0 to 4 (H15).
    - Game state 284 set, B1 set and B2 clear: sets B2 and B3.
    - Game state 284 set, B3 set and B4 clear (the tick after the step above): sets B4; GiveItemToHero(item 30, 1, no notification); sets game state 67; sets the return world for movies to world 74 (pack 956); sets game states 283 and 302; plays movie 44 (wait). This is the end of the glade fight, after the hero walks into trigger 4 (High).
    - Game state 284 set, B5 set and B6 clear: sets B6 and runs the exit routine: debug line, fade out, 2 s wait, sets game state 67, starts an exterior replacement load for world 74 (no locator). No routine of this module sets B5, so this branch does not run with this module alone (Medium).
- CB7 (trigger 4, registered by the debug skip only): assigns the navmesh for character 7 (flag 1) when the trigger is active and a second test holds. Partly described.
- Helpers: H4 and H5 (collision mask on an argument, both unknown); H6 (the fight-end helper of CB3 rule 2, which the debug skip also uses); H7 (navmesh assign for character 7); H15 (navmesh assign for characters 0 to 4 with zero flag); the exit routine of CB6 (above).
- Threads: CB3 (flag 281 clear) and CB6 (flag 281 set). Only one of the two starts at init.
- Edge cases and error handling:
  - The two threads are mutually exclusive by flag 281.
  - Game state 214 is set in play by CB3 rule 2 when level state 2 (Gnarfas's death event) arrives; the Update copy of the same calls is the only debug-only part.
  - Re-entering the world clears the step word of the running branch, so a rule latched by a step bit can run again after a later activation if its other gates still hold (movies 13 and 14 are also gated on "not played").
- Confidence: Medium.
  - High for init, the callback routines, the step-bit logic, the rule order of CB3 and CB6, the flag 64 step and the item 30 / flags 67, 283, 302 / movie 44 step, and the exit and hooks having no host calls.
  - Medium for the cutscene helper bodies (flat code; the camera and entity numbers are call arguments) and for which fight events set the odd level states.
- Open questions:
  - Meaning of game-state flags 67, 212, 213, 214, 281, 283, 284, 301, 302 and of level states 0 to 8 beyond their use here.
  - The second test in CB7 (debug-only path).
  - Which level objects the entity numbers (0, 4 to 12, 22 to 33) and locators (9, 10, 26, 27, 28) refer to: "from level data, id not yet mapped".
  - Whether anything outside this module sets B5 (the exit routine to world 74).
  - Resolved: the Werelupe King during the Gnarfas fight. (1) Entity 8 is bound before the grab runs: the grab sits behind a one-tick sleep after the callback that places the hero at locator 9, and the region pass of the next tick binds records 5, 7 and 8 (21, 92 and 91 units from the hero) before the script step; the filter is asked on every region pass, not at activation (High for the order). (2) The King is meant to stand, looping his idle (action 0, 20 keys), in base state 33 from the first script tick after arrival until the defeat sequence of rule 2 deactivates entities 7 and 8 (the same tick Gnarfas's death event is seen); nothing releases him earlier, and he does not attack (High). He may run his own controller for at most the one pass of the bind tick, which cannot attack (its cooldown counters start at 0 and the first attack needs 481 ticks). (3) A grab or an animation call on an entity that is not bound does nothing and stores nothing (High). Default if unresolved (rewrite choice): bind records 5, 7 and 8 on the first region pass after the hero is placed at locator 9, run the script step after it, and hold the King at once.
  - Resolved: the way out after Gnarfas's death. Trigger 4 is the slab described in rule 3; it is 36.5 in front of locator 26 along the hero's facing, the NE wall opened by the defeat is crossed on the way (30.9 from locator 26), and Illusen's second record stands straight ahead beyond it. The game shows no line, marker or talk. Trigger 4 is the only exit of pack 985 (no door or teleport plane exists; the module registers no other trigger) (High). Test vector: hero at locator 26 (-621.28, 122.68, -542.68) walks along (0.643, 0, 0.766): after 30.9 units it passes the open wall plane, after 36.5 it touches the slab's near face and the hero stands at slab coordinate 34.1 of 67.26 (the hero's own facing line gives 33.4).  - Resolved: the module's level-state calls are the clears of states 0, 1 and 2 at the Gnarfas arrival (only while game state 214 is clear) and the sets of states 0, 1, 2, 4, 6 and 8. The set of level state 0 is the last call of the Gelert intro cutscene, after the dialogue and the closing camera shot, in the same script step as the hero's and the boss's release; it is what releases the Gelert from its wait state (High). In the Gnarfas branch level state 0 is set by the boss's own event 0.
- Arena facts (SPEC-0013 and SPEC-0020, Arena build guide):
  - Collision mask word: bit 1 turns on the north-east sealing wall and its trigger rectangle (gate 1), bit 2 the south-west sealing wall (gate 2). The Gnarfas arrival sets both only while game state 214 is clear; the Gelert arrival sets both unconditionally. The Gnarfas fight-end routine and the Gelert finale each clear bit 1 only, so the north-east passage opens and the south-west wall stays on for the visit. A visit with flag 214 set at the Gnarfas arrival has no wall.
  - Effects 1 to 6 (south-west mouth) are never stopped; effects 7 to 12 (north-east mouth) are stopped by the fight-end routine and the finale.
  - Locator numbers: 9 hero arrival (-637.03, 122.67, -567.24); 10 hero after the Gnarfas movie (-625.73, 122.67, -548.23); 11 Gnarfas arrival (-617.357, 123.335, -558.802); 12 Gnarfas after the movie (-647.11, 122.75, -526.51); 26 hero after Gnarfas (-621.28, 122.69, -542.68); 27 Gelert stage scenes (-624.26, 122.03, -545.80); 28 hero in Gelert stage scenes (-634.77, 122.03, -561.14).
  - Entity numbers 5 and 6 are the two Illusen records (same spot (-582.57, 121.48, -493.09), different facing); 7 is Gnarfas, 8 the Werelupe King (-580.80, 121.88, -496.08); 0 to 4 are the Gelert records.
  - Trigger 4 is the kind-4 rectangle with number 4: 67 long, 7 thick, 31 tall, near face through (-623.97, -492.79) along (0.77, 0, -0.64), 5.8 beyond the north-east wall.


### 989: Exterior, effect locators, a looping animation thread and a staged sequence

- Family: Exterior.
- Exit: present, no host calls.
- Init, in order:
  1. Debug prints (three lines).
  2. InitObjOn(object 2, false).
  3. InitObjOn(object 0, false).
  4. ControlThreadsAllocate(3).
  5. Starts thread A and thread B.
  6. RegisterActivateWorldCallback(W3).
- Update, per tick (short; High):
  - MoveEffectToAnimatedLocator for effects 0 to 7: effect k follows locator k of object 2 (flag false), unconditional. No other host call is reached from the update body.
  - Each effect, in detail (High): effects 0 to 7 are the eight placed-effect records of the pack, all effect id 102, zone 0, placed at (0.0, 9.11, 3.78) and active from load. The call moves effect k to the current world position of locator k of object 2 on every tick; it passes no direction (face false). The module tests neither the knight's phase nor level state 10 here. Locator k at rest is a point of a horizontal ring of radius 9.30 at height 17.371 centred on (0.0, 17.371, 3.784): locator 0 (-9.298, 17.371, 3.787), 1 (-6.572, 17.371, 10.354), 2 (-0.003, 17.371, 13.080), 3 (6.575, 17.371, 10.354), 4 (9.297, 17.371, 3.781), 5 (6.573, 17.371, -2.790), 6 (-0.001, 17.371, -5.503), 7 (-6.567, 17.371, -2.787). The position is the object's rest translation (0.0, 9.110, 3.784) plus its root offset plus its unscaled current rotation applied to the locator's offset from that translation (SPEC-0030, Note on animated locators), so the effects move only when object 2 moves: thread A's loop (started and stopped by level state 10) makes each effect sway on a closed 16-key, 1.0 s path (at most 1.02, 1.14, 1.12, 0.68, 0.60, 0.96, 1.16 and 1.10 units from the ring point for effects 0 to 7); without state 10 the effects sit on the ring points. Example: at key 2 of the loop effect 0 is at (-9.081, 17.654, 3.124). A build must run this move every tick from world activation; until the first update the effects are at their placement point.
- Thread A (one test per tick, Sleep(1) between tests; High for the numbers):
  - Start: level state 10 set and object 2 not animating: a diagnostic line, LoopAnim(object 2, speed 1.0), then a ramp-up loop: SetSpeedFactor(object 2, f), Sleep(1), f = f + 0.1 (float arithmetic), repeated while f is below 1.0, starting at f = 0.0. The applied factors are 0.0, 0.1, 0.2 ... 0.9 (ten ticks); the next value (1.0000001) ends the loop without being applied, so the object loops at factor 0.9 until state 10 clears.
  - Stop: level state 10 clear and object 2 animating: a ramp-down loop starting at f = 1.0: SetSpeedFactor(object 2, f), Sleep(5), f = f - 0.1, repeated while f is above 0.0. The applied factors are 1.0, 0.9 ... 0.1 (ten steps five ticks apart, 50 ticks); the next value (about -7e-8) ends the loop. Then StopAnim(object 2): the object stops at its current time and keeps that pose (it is not reset to its start pose).
  - While a ramp runs the thread does not test state 10. State 10 is set by the Black Knight's enraged high swing (SPEC-0012), and object 2 is the eight-locator fixture over the knight's start point.
- Thread B (staged sequence; these are proven state gates and transitions, with no named state meanings established by the module or region records):
  - Reads level states 2 and 3 as a staged gate.
  - Stage 1 (2 set, 3 clear), the pre-fight step: PlayFMV(46, wait 1), BossFightTweakHero(instance 7, stage 0), SetLevelState(3, true), SetLevelState(4, true). Movie 46 runs 31.13 s and is played only by this module.
  - After level states 3 and 4 are set, helper H1 sets level states 5 and 1 (Medium).
  - Level state 0 means the boss is defeated: the boss's defeat test sets it once (SPEC-0012). Level state 6 is set when level state 1 is set, level state 6 is clear and level state 0 is set (Medium-High). Level state 6 is therefore set before the post-fight block, not after movie 47.
  - Post-fight block, when level states 5 and 6 are set and level state 7 is clear, in this order (High): FadeScreenOut(0.1 s, wait 1); AddBlackboardMessage(type 2, value 2, world 78); DeactivateEntity(0); SetGameState(51, true), SetGameState(298, true), SetGameState(86, true); BossFightTweakHero(instance 7, stage 1); SetReturnWorldForFMV(78); PlayFMV(47, wait 1). Movie 47 runs 32.57 s.
  - Level state 7 is never set in this module, but the block still runs once: movie 47's return world (78) reloads the area when the movie ends, so the module is loaded again (High).
  - Several sub-branches are not fully known [partial].
- Helper H1: DebugPrintf(argument), SetLevelState(5, true), SetLevelState(1, true). Called from thread B after level states 3 and 4 are set (Medium).
- Helper H2: DeactivateEntity(0), SetGameState(51, true), SetGameState(298, true), SetGameState(86, true). Resolved: thread B calls it in the post-fight block, right after AddBlackboardMessage and before BossFightTweakHero(instance 7, stage 1) (High). Entity 0 is the Black Knight (the pack's one entity-table entry is ordinal 0 of its embedded region).
- W3 (activate-world callback, from init): DebugPrintf(argument) and SetLevelState(2, true). It also stores 1 in one module word and 0 in a second; no code in the module reads either word (High).
- Gate precision (High): thread B's post-fight block tests level state 6 set, level state 7 clear, and either level state 4 clear or level state 5 set. Level state 6 can only be set after level state 1, and level state 1 only by H1 (which needs level state 4 set), so in play the block needs 5 and 6 set as described above.
- Timing (High for the order, Medium for the one-tick offsets): thread B polls once per tick. After the movie 46 hand-off (SPEC-0026, Movies; for the exterior-current case only the transition buffer is reloaded and the arena world is untouched) it sets states 3 and 4 on the same pass, states 5 and 1 on the next, and then polls. When the knight sets level state 0 on tick h + 1 (its last part reached 0 on tick h), thread B sets level state 6 on the same or the next pass and starts the post-fight block on the pass after that, so movie 47 is queued about 9 ticks after the knight's defeat tick (6 ticks of fade plus two or three polling passes).
- Dormant routine with host calls (High): one routine that nothing names (no call, callback, thread argument or data word) calls PlayCamera(track 5, wait 1, end mode 0, speed 1.0), then ReturnToNormalCamera(-1, release true), then stores 0 in the module word that W3 sets to 1. Camera track 5 has 66 keys; tracks 0 to 4 have none and are never named by the module.
- One routine with no host calls is called from a helper that one of the two threads started by init calls. The helper's own calls are ungated; whether the thread's call to the helper is gated is not established.
- Other routines in the module image are runtime-library code (static-constructor runners, float helpers); apart from the camera routine above, none has a host call and none is dormant script code.
- Threads: A and B, both started in init. W3 is registered for world activation.
- Edge cases:
  - Thread A runs until the level state changes, so the first time it sees state 10 set it starts the loop.
  - Thread B's stage gates are sequential, so it only advances when the earlier flags are set.
- Confidence: Medium. Init, the update and the activate-world callback are High. Thread B's stage order and post-fight block are High (every gate was followed). Thread A's ramps are High for the factors and tick counts; what object 2 shows is Medium.
- Open questions:
  - Resolved: level state 0 means the boss is defeated (set once by the boss's defeat test, SPEC-0012). Meaning of level states 1 to 7 and game states 51, 86, 298: unknown.
  - What the boss-fight tweak enum values stand for.
  - Resolved: the update moves effects 0 to 7 to locators 0 to 7 of object 2.
  - Resolved for H1: thread B calls it after level states 3 and 4 are set. Resolved for H2: thread B calls it in the post-fight block.
  - Resolved: the module has one dormant routine with host calls (a PlayCamera and ReturnToNormalCamera pair on camera track 5), not three dormant routines without them.
  - Arena data of pack 989 (hall, walk mesh, start positions, objects, camera, music, door) is summarised in SPEC-0012 (Arena build guide).
  - Resolved: the post-fight block runs once (movie 47's return world reloads the area).


### 1078: Exterior, object state and button-press room with four trigger scripts registered from Init

- Family: Exterior.
- Exit: present. It makes no host calls. Load and unload hooks exist and make no host calls.
- Callbacks: thirteen script-trigger registrations are made by init without a flag test (step 4): triggers 3 to 8 to the large object script, triggers 26 to 30 to the button-press object script, trigger 0 to the button prompt with item check and trigger 1 to the short object sequence (High for the indices). All four routines run in play when their gates hold.
- Threads: ControlThreadsAllocate(1) in Init; no thread start.
- Init, in order:
  1. Reads game flag 309. If it is clear, sets it (first-entry gate).
  2. Reads game flag 310. If it is clear, sets object 1 (bool 0). If it is set, sets object 0 (bool 0). The flag 309 test does not select these object calls.
  3. Unconditional at this level: runs the init helper, which fills the six-record door table below and sets each record's start state: the door objects and the lever objects are shown in their open state when the record's flag is set and in their closed state when it is clear; the plate's "down" object is off; the record's effect group is stopped. Then effect groups 15 to 21 are stopped (High).
  4. Unconditional at this level: thirteen script triggers are registered (indices 7, 4, 8, 6, 3 and 5 to the large object script; 26 to 30 to the button-press object script; 0 to the button prompt; 1 to the short object sequence), the hero is tested for an item whose id is not identified here, and a control-thread allocation is made. None of these is tied to a flag.
- Update (one per tick):
  - If flag 670 is set and flag 40 is clear: sets flag 40.
  - Calls the flag 311 and 312 helper every tick (unconditional at this level). The item-18 test that sets flags 311 and 312 is inside that helper.
- Helpers:
  - Init helper (called from init): the first-entry object setup above.
  - Effects toggle helper (called from init and from one of the button-press scripts below): looks up a group instance by its argument and starts or stops an effect depending on a test of its own.
  - Flag 311 and 312 helper (called from update): the item 18 test above.
  - One empty helper.
- Button-press scripts (four routines). Each is registered as a script-trigger callback by init, with no branch inside init that skips the registration. The module starts no thread for them. Each runs when the engine fires its trigger index; the gates inside each are described below.
  - Door table (High; six records of 88 bytes in module data, filled by the init helper). Per record: plate trigger, lever trigger, flag, door A (shown and hidden objects) and its camera, an optional door B, lever objects, plate objects (down and up) and an effect group.

    | Rec | Plate | Lever | Flag | Door A (shown, hidden), camera | Door B | Lever objects | Plate objects | Group | Lever locator |
    | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
    | 0 | 7 | 29 | 669 | 4, 7, camera 5 | - | 5, 6, 75 | 61, 62 | 19 | 20 |
    | 1 | 8 | 27 | 672 | 8, 11, camera 6 | - | 9, 10, 71 | 57, 58 | 20 | 18 |
    | 2 | 5 | 28 | 673 | 16, 20, camera 8 | 17, 21 | 18, 19, 73 | 63, 64 | 17 | 19 |
    | 3 | 6 | 30 | 671 | 12, 15, camera 7 | - | 13, 14, 77 | 59, 60 | 16 | 21 |
    | 4 | 3 | none | 674 | 2, 3, camera 11 | - | - | 67, 68 | 18 | - |
    | 5 | 4 | 26 | 670 | 22, 24, camera 10 | 23, 25, camera 9 | 69 only | 65, 66 | 21 | 17 |

  - Plate script (large object script; triggers 3 to 8; High): runs when the trigger state is 1, the callback words +0x20 and +0x24 are both 0 and the record's flag is clear. With the meanings of those words (SPEC-0030 section 3) the plate ignores hero shots (shot word) and hero attacks (attack word). It waits until the plate's up-object stops; swaps to the down-object and plays it; starts the record's effect group; door A (and door B) is hidden, the other object shown and played; the first time only it plays the record's camera (1.0 s), waits 1.5 s and returns the camera. It then waits until the trigger state is 0 (the firer has left). If the record's flag was set in the meantime it stops there (the door stays open). Otherwise it closes the doors again (the camera is shown again only if it was shown this time), restores the plate and stops the effects. So a plate keeps its door open only while it is held.
  - Lever script (button-press object script; triggers 26 to 30; High): when the object is present and the record's flag is clear, the square-button prompt (context 4) is shown until a newly pressed Square (button code 0) is detected; a held Square without a new press does not satisfy this gate. Then it sets the record flag, walks and orients the hero to the table's lever locator, plays hero animation 140, and plays the lever objects; the doors open permanently.
  - Short object sequence (trigger 1). If flag 310 is clear and field 28 is set: sets flag 310, deactivates object 0, activates object 1 (bool 0), PlayCamera(4, 1.0 s), Wait(1 s), plays animation 1, Wait(0.3 s), a camera shake (0.5 s), starts effect group 15, Wait(1 s), and returns the normal camera (High).
  - Button prompt with item check (trigger 0; once per load through a module word, file value 0). If field 28 is set, flag 310 is set and the module word is 0: shows prompt context 6 until a newly pressed Square (button code 0) is detected. A held Square without a new press does not satisfy this gate. On a newly pressed Square:
    - Flag 312 set, or item 18 held: deactivates object 1, activates object 0, PlayCamera(4, 1.0 s), Wait(1 s), plays animation 0 (wait), Wait(1 s), and returns the camera.
    - Otherwise (flag 312 clear and item 18 not held): sets flag 311 (if it is clear) and shows message 1.
    (High for the gates: the opening needs item 18 or flag 312.)
- Update: the flag writes above only. No other host calls.
- Edge cases:
  - Flag 310 is tested in init and in the button-press scripts. Its meaning is not known.
  - Object arguments: the door, lever and plate scripts take their object, camera, locator and group ids from the door table above (module data filled by the init helper).
  - Item 18 is the only item named in the module's text so far. The gated init test of a further item (id not identified here) makes this statement open.
- Confidence: High for init, update, the trigger indices, the door table and all four trigger scripts.
- Open questions:
  - Flags 670 and 672 are set by the Oubliette levers: the lever of trigger 26 sets 670 and the lever of trigger 27 sets 672, each when the hero presses Square at the lever while its lever object is present and the flag is clear (the record's door pair then opens for good; SPEC-0049, Side-quest flag owners). Flag 670 also sets flag 40 in the per-tick update, and flag 40 has no other reader. Meaning of flags 309, 310, 311, 312 and 40: unknown.
  - Item 18: not yet mapped.
  - Object ids 0 to 10, 57, 71 and group instances 15, 16, 20, 21: from level data, ids not yet mapped.
  - Which fire paths reach the plates in play (they refuse hero shots and attack hits; a non-attacking hero on the trigger-hit path, or another firer, passes).
  - Resolved: the thirteen registrations and their callbacks are listed in init step 4.


### 1090: Exterior, hazard timer step, firing loops and hurt zones

- Family: Exterior.
- Exit: present, no host calls.
- Update: present, one short routine (High). It runs the hazard timer step twice, once per bank (see Update, per tick below).
- Init (High):
  - SetGameState(314, true) and SetGameState(42, true), unconditional.
  - Two hazard banks: bank A is objects 3 to 9 with hurt triggers 1 to 7 (routine T1), bank B is objects 10 to 16 with hurt triggers 8 to 14 (routine T2). Hazard i of a bank (i = 0 to 6) is the bank's first object + i with the bank's first trigger + i. For each hazard: AddAnimObjSound(its object, mode 0, 1.0, 1.0) and the registration of its trigger.
  - ControlThreadsAllocate(3), ControlThreadStart(thread C), ControlThreadStart(thread D), one frame-rate query and a debug print.
- Trigger callbacks:
  - T1 (triggers 1 to 7) and T2 (triggers 8 to 14): hurt zones. When the trigger record's object word (+0x1c) is non-zero, the routine finds the bank entry whose trigger equals the record's trigger index (+0x14). While the record's state word (+0x10) stays non-zero (the hero is inside): if that hazard is firing (its active word set), DamageHero(20) and Wait(1.0 s); then Sleep(1 tick) and test again. So a hero standing in a firing zone loses 20 hit points about once a second (High).
- Threads C and D (started in Init; High): the two fixed firing loops, C for bank A and D for bank B. Each loop runs four steps forever. A step arms four hazards of its bank one after another: arming sets the hazard's timer to the bank's duration times 60 as a float (0.7 s gives 41.999996 in bank A, 0.5 s gives 30.0 in bank B), and the last arm of the step waits the duration (41 ticks in bank A, 30 in bank B) before the next step starts. There is no rest between steps. The Update timer step turns an armed timer into a firing: the hazard is active for 42 ticks in bank A and 30 in bank B, counted from the first Update after the arm. Bank A steps (hazard indices): {0, 2, 4, 1}, {1, 3, 0, 5}, {2, 1, 6, 3}, {4, 5, 6, 2}. Bank B steps: {0, 6, 4, 1}, {2, 3, 5, 1}, {0, 1, 2, 3}, {4, 5, 6, 2}. A hazard that the next step arms again before its timer ends stays active without a break, and its object is not restarted.
  - Test vector (tick offsets are good to one tick): bank A, thread C arms hazards 0, 2, 4 and 1 (objects 3, 5, 7 and 4) on tick 0. They become active and play on tick 1 and would stay active for 42 ticks. Step 2 arms hazards 1, 3, 0 and 5 on tick 41, so hazards 1 and 0 stay active without a break, hazards 3 and 5 become active on tick 42, and hazards 2 and 4 become inactive on tick 43 and their objects are deactivated. Hazard 1 is armed by steps 1 to 3 and stays active from tick 1 to tick 124. In bank B, which arms every 30 ticks for 30 ticks, hazard 1 (steps 1 to 3) likewise never turns off between steps.
- Update, per tick (High): the update calls the hazard timer step twice, once for each bank's table of seven records (object id, timer as a float, active word, trigger index; 16 bytes each). For each record in order: when the timer is above 0, the timer drops by 1.0 (computed in double precision and stored back as a float) and, if the active word is clear, the active word is set and the hazard's object plays animation 0 (wait 0, speed 1.0); when the timer is 0 or below and the active word is set, the active word is cleared and the hazard's object is deactivated. There are no other tests: the values 0, 64, negative numbers and 2047 that a listing of the update shows belong to the compiler's double-precision routines (SPEC-0042, open question 9). The hurt zones read the active word.
- Helpers (every caller is known; High):
  - Hazard set-up helper, called by Init for each hazard: AddAnimObjSound on the hazard's object with mode 0.
  - Arm routine, called by threads C and D: the record's timer becomes seconds x 60 (one single-precision multiply), then Wait(seconds) when its wait argument is nonzero.
  - Timer step, called twice by Update, once per bank: as described under Update, per tick.
  - The other routines with no host calls are the double-precision conversion, subtraction and packing routines that the timer step uses.
- Dormant routines with no host calls: three. None is referenced by a callback or thread argument.
- Threads: C and D, both started in init (the two firing loops above).
- Edge cases:
  - All Init calls run on every load; all fourteen hurt triggers and sounds exist in play.
  - The four hazards of a step fire in the same phase; the two banks run independently.
  - The update runs every tick, so its calls run far more often than the init calls.
- Confidence: High for Init, the hurt zones, the firing loops of threads C and D and the Update timer step; Medium for the exact tick offsets of the test vector.
- Open questions:
  - Resolved: the update is the hazard timer step (see Update, per tick); it compares only the timer with 0.
  - Resolved: a hurt zone is active while its hazard fires in thread C's or D's loop (0.7 s in bank A, 0.5 s in bank B).
  - Resolved: the update reads no game state; it reads only the two hazard tables.
  - Resolved: Init fills the records, threads C and D arm the timers, and Update runs the timer step; the set-up and arm routines are called only from Init and the two threads.
  - Resolved: threads C and D are the two firing loops.
  - Resolved: the update has no gates.


### 1096: Exterior, level state machine with hero pickup, timed boss sequence and debug shortcuts

- Family: Exterior.
- Exit: present. It makes no host calls. Load and unload hooks exist and make no host calls.
- Init, in order:
  1. Sets game flags 315 and 44.
  2. Sets object states for 12 objects (ids 31, 33, 35, 16, 17, 19, 22, 6, 8, 10, 12 and 13), all (bool 0).
  3. Registers script triggers: indices 13, 14 and 15 to CB1; indices 0, 1, 2, 4 and 3 to CB2; index 5 to CB3; index 6 to CB4; index 7 to CB5.
  4. Registers the AI-player activate callback CB6 (the activation filter: it admits every record ordinal except 0, which it refuses; High).
  5. Unconditional at this level: registers the hero pickup callback CB10 (twice), fades the screen out (duration 0), and sets the Undead Draik's nine part breakable indices (17, 20, 22, 15, 16, 19, 18, 21, 23, in the call's argument order).
  6. Reads flag 45.
     - Flag 45 clear: allocates one control thread and starts the main state thread CB7. Allocates and creates one slot. Registers the activate-world callback CB8, and registers trigger index 8 to CB9.
     - Flag 45 set: registers the activate-world callback CB11 instead.
  - Flag 45 selects only the control thread, the slot, CB8 and CB9 (and CB11 on the other side); the rest of init runs whatever flag 45 holds.
- Callbacks:
  - CB1 (triggers 13 to 15): if the object's state is 13 and a further test passes: if button 0 is pressed, walks the hero to point 15, plays hero animation 140, plays animation 31, deactivates object 16, plays animations 6, 16, 15 and 17. If button 0 is not pressed: shows prompt context 4. Partly described. A second branch for state 14 walks the hero to point 16 and plays animations 33 and 17. A third branch for state 15 walks the hero to point 17 and plays animations 35 and 19. CB1 also sets the module word that arms index 0 of CB2 (Medium for which branch sets it).
  - CB2 (triggers 0 to 4; one routine whose body is a switch on the trigger index). When the record's object word is non-zero and its state word is 1, it first waits (one-tick sleeps) while the hero is occupied, then runs the case for its index. Each case that runs sets the normal camera (values 3, 0, true), grabs hero control, waits 0.3 s, plays its animations (speed value 0.7) and restores control for the active hero (selector -1, per SPEC-0030 section 2.1 and row 166; High for the calls and selector):
    - Index 0: only when a module word set by CB1 (file value 0) is set: deactivates object 6, then plays animation 7 (wait).
    - Index 1: plays animation 10 (no wait), then animation 8 (wait).
    - Index 2: only when a module word set by index 3 (file value 0) is set: deactivates objects 8 and 10, plays animation 11 (no wait), then animation 9 (wait).
    - Index 3: no host calls; sets the module word that arms index 2.
    - Index 4: plays animation 13 (no wait), then animation 12 (wait).
  - CB3 (trigger 5): plays animation 22 (no wait) if the object is valid.
  - CB4 (trigger 6): adds blackboard message (type 2, value 130, world 126) and starts an exterior replacement load to world 126 (mode -1, no palette), if the object is valid and its state is 1.
  - CB5 (trigger 7): sets flag 318 if the object is valid.
  - CB6 (AI-player activate): returns nonzero for every record ordinal except 0, so the streaming test never binds the world's record 0 by itself and binds every other record by the normal rules (High).
  - CB8 (activate-world, registered in the flag 45 clear branch; High). In order: (1) SlotLoad of the stored slot with j = 0, keep false and activate false, which loads character region 1097 (pack + 1 + 0; it holds only the thirteen dispenser item placements, below) and spawns nothing; this is the only call that waits (for the read). The slot word this load reads starts at 0 in the module file (not -1), and a SlotCreate result is also stored into it, so the load uses that stored slot. (2) The dispensers are enabled (a module word set to 1). (3) InitObjOn with false (start pose) for objects 1, 0, 2, 3, 4 and 5. (4) SetTargetActive(1, false) (aim target 1 off). (5) FadeScreenOut (black, 0 s, no wait). (6) A debug line. (7) PlayFMV(51), which queues the movie (SPEC-0030 row 138). (8) MoveHero(locator 10, probe true, control 1, face true). (9) SetToNormalCamera(5.0, 3.14, hand-back true). (10) ActivateObj(14, false), which marks object 14 active at time zero (SPEC-0030 row 3); it does not hide the object, and object 14 (an invisible four-wall block in the west doorway, SPEC-0029 6.12) stays solid. (11) ActivateEntity(0), the Undead Draik. (12) BossFightTweakHero(6, stage 0). (13) A debug line. (14) FadeScreenIn (black, 2.0 s, no wait). After step 1 nothing yields, so steps 2 to 14 run in one script step and the queued movie plays at that pass's movie step with the encounter already set up. No control grab is made.
  - CB9 (trigger 8): queries whether movie 51 has played. No other effect.
  - CB10 (pickup callback): if the picked item id is 117, sets level state 15.
  - CB11 (activate-world, flag 45 set branch): initialises object 1 with the flag true (High).
  - CB7 (main state thread, started from init when flag 45 is clear; High): the core of the module. While flag 45 is clear, each tick (one-tick sleep per pass):
    1. The two item dispensers run (below).
    2. Then the first rule that holds runs. The conditions are tests of level states only; no item, hero-inventory or timer condition appears in any of them (those belong to the dispensers of step 1). The rules are tried in this order: level state 0 set and 2 clear: the pair helper sets level states 1 and 2. The Undead Draik's second HP-threshold post of level state 0 finds level state 2 already set and does nothing. Level state 3 set and 5 clear: 4 and 5. Level state 6 set and 8 clear: 7 and 8. Level state 9 set and 11 clear: 10 and 11. Level state 12 set and 14 clear: the finale. A rule that holds therefore blocks the later ones for that tick, and each pair rule fires once, because its second state is set by the reaction itself.
    3. The thread then sleeps one tick. It never ends: once flag 45 is set, each pass only sleeps.
    - Finale, in order (High): SetLevelState(13); PlayCamera(5, no wait, end mode 1, speed 1.0) (track 5 is a fixed camera at (427.90, -1557.80, -550.90)); StartCameraShakeMode(0, 3.0 s, strength 1.0); DeactivateEntity(0) and DeactivateObj(0) (object 0 is the four-wall block at (526 to 555, -534 to -504) in the south-east corridor); Wait(1.0 s); PlaySound(7, no emitter, restart allowed); PlayAnim for objects 2 to 5 (speed 1.0, no wait); Wait(4.0 s); SetGameState(45) and SetGameState(316); BossFightTweakHero(6, 1) (no effect); SetToNormalCamera(5.0, 3.14, hand-back true); SetLevelState(14). It makes no control grab and no hero move: the hero keeps control throughout. It takes 300 ticks from its first call to the last. Setting flag 45 stops the dispensers and the rules for the rest of the level (the thread keeps sleeping).
  - Item dispensers (enabled by CB8; High): item 163 is kept at one of 6 placements (ordinals 0 to 5) and item 161 at one of 7 placements (ordinals 6 to 12) of the boss slot's region (resource 1097, loaded by CB8 into slot 0, so the ordinal argument of the item activation is that slot's placement ordinal; resolved). The six item 163 placements are at (423.81, -1611.66, -398.10), (469.00, -1612.98, -474.90), (393.26, -1609.46, -579.02), (443.76, -1612.51, -499.02), (435.77, -1612.64, -511.71) and (412.97, -1611.66, -561.78); all seven item 161 placements are the same point (341.00, -1610.15, -589.53), so that item is always at one place. Every placement has the snap flag set (the item is put at the floor height plus 0.5) and no drop group. The ordinal draw is the script Random call over the whole range of ordinals, both ends included. Each dispenser has a placed flag and a pending-placement flag; both dispensers start with a placement pending. Per tick, in this order: (1) if the item is placed and the item id reported by the latest hero pickup this tick equals the dispenser's item, the item counts as taken: the placed flag clears and the time of the pickup is stored; (2) if the item is not placed and either the hero holds none of that item or more than 30 s have passed since the stored pickup time, a placement is pending; (3) if the dispensers have been enabled by CB8 and a placement is pending, the item is activated at a random ordinal of its range, drawn again until it differs from the last ordinal used by that dispenser (so with no earlier use the first draw stands), the pending flag clears and the placed flag sets. The pickup word is the item id stored by the pickup callback CB10; it is cleared after both dispensers have run, so a pickup is seen for one tick only. The held-item test is therefore a condition only of the dispensers' re-placement after a pickup and plays no part in the level-state rules.
- Helpers:
  - Level-state pair helpers (four, called from the state thread by the rules above): each sets one pair of level states, namely 1 and 2, 4 and 5, 7 and 8, and 10 and 11.
  - Scene helper: the finale described above.
  - Dialog and camera helper (High): sleeps one tick at a time while a dialogue is active; Wait(0.7 s); PlayCamera(4, no wait, end mode 1, 1.0); PlayAnim(object 1, no wait; object 1 is the tower base door at (386.1, -1562.5, -588.9)); Wait(4.0 s); PlayCamera(6, wait, end mode 1, 1.0) then Sleep(60); PlayCamera(7, wait, end mode 1, 1.0) then Sleep(60); SetLevelState(16); SetGameState(317); SetToNormalCamera(5.0, 3.14, hand-back true); SetTargetActive(1, true). Camera tracks 4, 6 and 7 are keyed tracks of 44, 38 and 38 keys. No control grab.
  - Item and time helper: the dispenser step (whether the hero holds the item, the system time for the 30 s rule, and the random-draw routine below for the next placement; High that this helper is the dispenser step). It is called twice per tick, first for item 163 and then for item 161, before any level-state rule, and shares no condition with the pair helpers.
  - Trigger handlers CB1 to CB5 use their own short checks and do not call the level-state pair helpers.
- Switch-selected routine: this is CB2 above (the trigger 0 to 4 routine); its cases are selected by the trigger index (High).
- Update (one per tick): a debug block first, then a play block.
  - Play block (runs in play): every tick that flag 45 is set, level state 15 is set (CB10, the pickup of item 117) and level state 16 is clear, it calls the dialog and camera helper: when no dialog is active, cameras 4, 6 and 7 with waits, animation 1, target active on, the normal camera, then level state 16 and flag 317 are set. So flag 317 is set in play, once, after the boss sequence (which sets flag 45) and the item 117 pickup (High).
  - Debug block, with debug mode on and button 3 held, while level state 17 is clear and flag 45 is set: runs a debug sequence that sets level state 17, plays camera and animation steps with 60-tick sleeps, sets level state 16, sets flag 317, and sets the normal camera (blend 5, angle 3.14). Debug only.
  - Debug block, with debug mode on and button 3 not held: checks buttons 2, 1 and 0 in turn and, for each, sets or clears level states 12 to 17 and flags 45 and 317, and deactivates or activates objects 1 to 5. Debug only; partly described.
- Random-draw routine: called from the item and time helper above. That helper is reached from the main state thread (CB7), which init starts only when flag 45 is clear. Dormant routines: the other two are empty (no host calls) and have no references. The load-time constructor (its list entry and the body it calls) is not dormant: the load hook runs it at every load, and it only writes 0 into two module words that already hold 0 in the file, so play is the same as with the file values (High).
- Edge cases:
  - Flag 45 chooses between CB8 and CB11, and whether CB7 starts at all.
  - Debug-mode behaviour depends on the engine's debug mode flag, which is not in this module.
  - The module has no death callback, no music call, no save-point or start-pose call, and no load of its own area. The only script load of this area (area slot 105, world pack 1096) is the Knights Crypts exit trigger (SPEC-0042, module 1081), which sets flags 314 and 315 and loads it with no entry choice, so the hero is first put at entry record 0.
- Arena data the script uses (disc; the arena itself is described in SPEC-0029 section 6.12):
  - Entry (locator) records: 0 (363.21, -1612.31, -489.41) facing (0.881, 0, -0.473) is the arrival point; 10 (367.23, -1613.16, -493.38) facing (0.491, 0, 0.871) is the hero's fight start; 11 and 12 (400.25, -1608.45, -427.67) and (455.14, -1610.09, -431.84) are the arena chests; 13 (617.28, -1608.59, -610.13) is the east-room chest; 14 (324.80, -1399.09, -587.74) is a tower chest. Entity 0 is the Draik, record (390.46, -1605.27, -508.99) facing (-0.772, 0, 0.635).
  - Trigger 8 is a static vertical rectangle (corner (345.91, -1624.9, -493.22), 26.61 along (0.49, 0, 0.87), 22.01 high) across the west doorway, 18.6 west of locator 10. Triggers 9 to 12 are the four chest spheres (radius 3.69); the module registers no callback for them. Trigger 6 is a vertical rectangle at (313.55, -1216.35, -599.43), 27.4 by 24.4, at the top of the tower (the exit). Trigger 7 is a platform rectangle at (436.4, -1559.37, -622.43), 35.15 by 20.36. Triggers 0 to 5 and 13 to 15 are tower cylinders and rectangles (y -1556 to -1216).
  - Objects (rest positions): 0 to 5 at (541.9, -1611.9, -517.6) (0 is the four-wall block, 2 to 5 animated models); 1 at (386.1, -1562.5, -588.9) (the tower base door); 14 at (345.7, -1612.1, -480.5) (an invisible four-wall block in the west doorway); 6 to 13 and 15 to 22 belong to the tower and other levels; the pack's breakables 15 to 23 have no placement and are the Draik's part slots.
- Confidence: Medium for init and the trigger and pickup callbacks; High for the CB2 switch cases, CB7, the dispensers and the update's play block. Low for the update's debug branches, which are large and only partly described.
- Open questions:
  - Meaning of flags 45, 315, 316, 317, 318 and 44; meaning unknown.
  - Level states 0 to 17: meaning unknown beyond CB7's rules. The Undead Draik (class 12) posts level state 0 at its two HP-threshold transitions and state 12 after its death animation (SPEC-0029, Section 6.4). Resolved (proactive (open close-out); Medium-High): nothing sets level states 3, 6 or 9 in this world. This module only tests them, no other level module that can run in this world sets them, and the Draik posts only 0 and 12 (both of its thresholds post 0). So the 3, 6 and 9 rules of CB7 never fire in the shipped fight; the first 0 post fires the 1 and 2 pair, the second does nothing, and the 12 post at death runs the finale. A rewrite may keep the dead rules or drop them.
  - Resolved: the placement ordinal passed to the dispensers' item activation is a placement ordinal of region 1097, which CB8 loads into slot 0 (SPEC-0030 row 198); the slot number is the SlotCreate result, 0 (High).
  - Item 117 is a free placement at (634.34, -1603.92, -605.61) in the east room (the object of the pickup hook); movie 51 plays before the fight; boss fight instance 6 stage 0 is the refill of both heroes (SPEC-0030 row 165) and stage 1 does nothing. Item names: 161 Lava Mote (fire, kind 1), 163 Water Mote (kind 4).
  - Object ids and locators used by this module are placed in the arena data above; entities 14 to 16 and points 15 and 17 belong to the tower puzzles and are not described here.
  - Trigger volumes for indices 0 to 15 are placed in the arena data above (index 8 west doorway, 9 to 12 chest spheres, 6 the tower exit, 7 an upper platform, the rest tower volumes).
  - Resolved: aim target 1 (SetTargetActive) is a Rope Gun anchor of this world, the second entry of the world's aim-target list, at (404.617, -1542.583, -583.944): about 18.5 east of, 19.9 above and 5.0 north of the tower base door (object 1 at (386.1, -1562.5, -588.9)), 70 units above the arena floor. (Target 0 of the list is at (449.928, -1324.457, -624.420), high on the tower, and is never toggled.) The active word starts at 1. CB8 turns target 1 off at the fight start and the dialog and camera helper turns it on after the item 117 scene (the activate-world callback of the flag 45 side does not touch it, so it is on at load). While it is off the marker is not drawn and the hero's aim scan (SPEC-0030 row 126) cannot select it, so with the Rope Gun held the lock cannot land on the anchor during the fight. From the arena the anchor is inside the scan's 110-unit range from only part of the arena (for example 98.7 from the Draik's post, 120.7 from the hero's fight start). Position and active-word facts High; the fight-time effect Medium-High (whether the hero holds the Rope Gun there is not established).
  - Resolved: the calls once listed as additions to CB1 to CB5 are the switch cases of CB2 (the trigger 0 to 4 routine), selected by the trigger index; they run in play when their gates hold.


### 1130: Transition, three button zones, a one-shot switch and a movie zone

- Family: Transition (its Init entry).
- Exit: present, empty. The exit export runs no host calls.
- Load and unload hooks: present, no host calls.
- Callbacks: script-trigger registrations for trigger indices 0 to 7 (index 3 is not registered). Teleports: none. Threads: none. Dormant routines: two unreferenced older lever bodies (see Open questions).
- Init (in order):
  1. QueryGameState(675) into locals.
  2. If state 675 is clear:
     - InitObjOn(object 0, false), then InitObjOn(29, false), (31), (33), (27), (1), (12), (15), (18) and (11), all with bool 0. Each path has exactly one call for object 11, with no further test (High).
     - RegisterScriptTrigger: index 5 to callback 1; index 6 to callback 2; index 7 to callback 3; index 4 to callback 4; index 2 to callback 5; index 1 to callback 6; index 0 to callback 7 (the index-0 routine). All under the same 675 gate.
  3. If state 675 is set:
     - InitObjOn(29, true), (31, true), (33, true), (27, true), (1, true) and InitObjOn(11, false), (14, false), (17, false), (19, false).
     - No triggers are registered on this path.
  4. Unconditional at this level (both states): AddAnimObjSound for objects 11 (sound 10, floats 0.0 and 1.0; one call, with no further test), 7, 8, 9, 10 (sound 10), and objects 23, 24, 12, 13, 14, 15, 16, 17, 18, 20, 21, 22, 1, 2 (sound 8, floats 0.0 and 1.0).
- Update (every tick): while a shown-once word (file value 0) is 0, it checks IsBreakableIntact(breakable 0). When the breakable is broken it sets the word to 1 and shows LevelMsg(message 4, bool 1, WAIT 1). So message 4 is shown once per module load (High).
- Trigger index 0 (callback 7): gate: trigger record field at +0x1c non-zero and field at +0x10 == 1.
  - If movie 50 has not played and the hero is not occupied: PlayFMV(movie 50, bool 1).
  - If movie 50 has not played and the hero is occupied: Sleep(1 frame). The callback runs again on the next trigger frame.
  - If movie 50 has played: nothing.
- Trigger index 1 (callback 6, one call): LevelMsg(message 3, bool 1, WAIT 1), shown only the first time the trigger fires in a module load (a shown-once word, file value 0, is set before the message; High).
- Trigger index 2 (callback 5, the one-shot switch): gate: +0x10 == 1 and +0x1c non-zero. Then:
  - If object 2 is inactive and state 675 is clear:
    - SetGameState(675, true).
    - DeactivateObj(1); ActivateObj(2, bool 0); PlayAnim(2, WAIT 0, 1.0); ActivateObj(0, bool 0).
    - SetToNormalCamera(10.0, 0.0, bool 1).
  - Otherwise: nothing. This makes the switch one-shot through state 675.
- Button zones (indices 4 to 7): the water-lever puzzle that gates state 675 (High). Triggers 5, 6 and 7 are three levers; each keeps a lever word (pulled or not; file value 0). Trigger 4 is the test button. All four are registered only when state 675 is clear, and each runs on its trigger gate and a button press. The puzzle state is three junctions:
  - Junction 1 (lever 5): channel a or b. Junction 2 (lever 6): a or b. Junction 3 (lever 7): rest, c or d. Start: a, a, rest. A failed test (the reset tail) restores the start states and the levers.
  - Lever 5. Pull: if junction 3 is not c, junction 1 becomes b (object 12 shown); otherwise the pull is blocked (object 13 shown, the lever springs back, nothing changes). Push back (lever already pulled): junction 1 becomes a (object 14 shown).
  - Lever 6. Pull: if junction 3 is not rest, junction 2 becomes b (object 15); otherwise blocked (object 16, the lever springs back). Push back: junction 2 becomes a (object 17).
  - Lever 7. Pull: with junction 1 a and junction 3 rest, junction 3 becomes c (object 18); with junction 1 b and junction 3 rest, blocked (object 20). Push back from c: junction 2 a gives rest (object 19); junction 2 b gives d (object 21). Pull with junction 3 already c or d (possible only for d, after a push back): the lever animates to its pulled pose and stays pulled (its lever word is set), objects 18 to 22 and 10 are turned off, and nothing else happens: no camera, and junction 3 keeps its value, so a solved b, b, d stays solved. Push back from rest or d: the lever animates back and nothing else happens (High).
  - Test button: solved when junction 1 is b, junction 2 is b and junction 3 is d.
  - Solution: lever 7, lever 6, lever 7 again (push back), lever 5, then the test button. This is the shortest order: d needs junction 2 at b, b at junction 2 needs junction 3 off rest, and lever 5 must come last (pulled while junction 3 is rest it blocks lever 7; pulled while junction 3 is c it is blocked).
  - Test vector: start a, a, rest. Lever 7 -> a, a, c. Lever 6 -> a, b, c. Lever 7 -> a, b, d. Lever 5 -> b, b, d. Test -> solved, state 675 set. Pulling lever 5 first instead -> b, a, rest; then lever 7 is blocked.
- Trigger index 4 (callback 4, button zone with a reset tail). Gate: +0x1c non-zero and state 675 clear.
  - Always: LevelMsg(message 2, bool 1, WAIT 1), shown only the first time (shown-once word); IsButtonPressed(button 0, bool 1, unk 0).
  - If the button is not pressed: DisplaySquareButtonContext (message 4, text from module data).
  - If the button is pressed:
    - DeactivateObj(28); ActivateObj(27, bool 0); WalkHeroAndOrient(locator 4, WAIT 1, CONTROL 1); PlayHeroAnim(140, WAIT 0, CONTROL 1); PlayAnim(27, WAIT 0, 1.0); Wait(0.7); PlaySound(7, no position, bool 0), twice; PlayCamera(6, WAIT 0, CAMERA_WAIT 1, 1.0).
    - Deactivates objects 8, 9, 15, 16 and 17, then ActivateObj(8, bool 0); PlayAnim(8, WAIT 1, 2.0); ReturnToNormalCamera(-1, bool 1).
    - DeactivateObj(27); ActivateObj(28, bool 0); PlayAnim(28, WAIT 0, 1.0); Wait(0.6).
    - Then the shared reset tail (helper C below): LevelMsg(message 5, bool 1, WAIT 1) the first time only, and the object and lever reset.
    - Solved path (instead of the steps above after the first swap and sound): when the junctions are b, b, d: SetGameState(675, true); objects 7, 9, 10 and 11 shown (ActivateObj, bool 0) and played; object 1 played; PlayCamera(4, WAIT 1, CAMERA_WAIT 0); DeactivateObj(0). This is the only setter of state 675 besides the one-shot switch at index 2 (High).
    - Unsolved (High): after the hero animation and sound, selector 2 is cleared and camera 6 plays (WAIT 0, CAMERA_WAIT 1, 1.0). Junction 2 a: object 8 shown and played (WAIT 1, speed 2.0). Junction 2 b and junction 3 not d: object 9 shown and played (WAIT 1, 2.0). Junction 2 b, junction 3 d and junction 1 a: selector 3 is cleared as well, objects 9 and 10 shown, object 9 played (no wait, 1.0) and object 10 played (WAIT 1, 1.0). Then ReturnToNormalCamera(-1, bool 1), the swap back of objects 27 and 28, and the reset tail. Test vector: lever 6 alone (a, b, rest), then the test: object 9 at speed 2.0, then the reset; levers 7, 6, 7 (a, b, d), then the test: objects 9 and 10.
- Trigger index 5 (callback 1, button zone). Gate: +0x1c non-zero and state 675 clear.
  - Not pressed (IsButtonPressed(button 0, bool 0, unk 0) == 0): DisplaySquareButtonContext (message 4, text from module data).
  - Pressed: WalkHeroAndOrient(locator 5, WAIT 1, CONTROL 1); PlayHeroAnim(140, WAIT 0, CONTROL 1); DeactivateObj(30); ActivateObj(29, bool 0); PlayAnim(29, WAIT 0, 1.0); Wait(0.7); PlaySound(7, no position, bool 0).
  - Then, by the lever word and the junctions (the lever 5 rule above): push back (objects 29 off, 30 on), camera 6 and object 14; or a pull, camera 6 and object 12; or a blocked pull, camera 6 and object 13 and the lever springs back. The shared helper (selector 1) clears objects 7, 12, 13 and 14 first; PlayAnim and ReturnToNormalCamera(-1, bool 1) close the branch (High for the rule; Medium for the call order).
- Trigger index 6 (callback 2, button zone). Gate: same as index 5.
  - Not pressed: DisplaySquareButtonContext (message 4, text from module data).
  - Pressed: WalkHeroAndOrient(locator 6, WAIT 1, CONTROL 1); PlayHeroAnim(140, WAIT 0, CONTROL 1); DeactivateObj(32); ActivateObj(31, bool 0); PlayAnim(31, WAIT 0, 1.0); Wait(0.7); PlaySound(7, bool 0), twice.
  - Deactivates objects 8, 9, 15, 16 and 17, then PlayCamera(6, WAIT 0, CAMERA_WAIT 1, 1.0); ActivateObj(16, bool 0); PlayAnim(16, WAIT 1, 1.0); ReturnToNormalCamera(-1, bool 1); ActivateObj(15, bool 0); DeactivateObj(31); ActivateObj(32, bool 0); PlayAnim(32, WAIT 0, 1.0); Wait(0.6).
  - Which of objects 15, 16 and 17 is shown follows the lever 6 rule above (pull 15, blocked pull 16, push back 17; High for the rule, Medium for the call order).
- Trigger index 7 (callback 3, button zone). Gate: same as index 5.
  - Not pressed: DisplaySquareButtonContext (message 4, text from module data).
  - Pressed: WalkHeroAndOrient(locator 7, WAIT 1, CONTROL 1); PlayHeroAnim(140, WAIT 0, CONTROL 1); DeactivateObj(34); ActivateObj(33, bool 0); PlayAnim(33, WAIT 0, 1.0); Wait(0.7); PlaySound(7, bool 0).
  - Then PlayCamera(6, WAIT 0, CAMERA_WAIT 1, 1.0) and ActivateObj(18, bool 0) (both under extra predicates); PlayAnim(18, WAIT 1, 1.0); ReturnToNormalCamera(-1, bool 1).
  - Then object deactivations for 18, 19, 20, 21, 22 and 10 through the shared helper (selector 3). A pull shows object 18 (junction 3 to c) or, when blocked, object 20; a push back (lever word set) moves the lever back and plays camera 6 with object 19 (junction 2 a, rest) or 21 (junction 2 b, d), by the lever 7 rule above (High).
- Shared object-deactivation helper (called by indices 4, 5, 6 and 7 with a selector argument):
  - Selector 1: deactivates objects 7, 12, 13 and 14.
  - Selector 2: deactivates objects 8, 9, 15, 16 and 17.
  - Selector 3: deactivates objects 18, 19, 20, 21, 22 and 10.
  - Callers (High): lever 5 (index 5) passes selector 1, lever 6 (index 6) selector 2 and lever 7 (index 7) selector 3, on every branch. The test button (index 4) passes 2 on every unsolved test, adds 3 on the junction-3-d outcome, and passes 1, 2 and 3 on the solved path.
- Shared reset tail (helper C, called only by index 4; High): LevelMsg(message 5, bool 1, WAIT 1) the first time only (shown-once word); DeactivateObj for objects 7, 8, 9, 10, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21 and 22; then one marker per junction by its state before the reset: junction 1 a: object 12 shown, b: object 14 shown and played; junction 2 a: object 15, b: object 17 shown and played; junction 3 rest: object 18 shown (the call is made twice), c: object 19 shown and played, d: object 22 shown and played. Then the start states a, a, rest are restored, and each lever whose word is set (levers 5, 6 and 7, all three) has its word cleared and is played back to rest (objects 29 off and 30 on and played; 31 off and 32; 33 off and 34, each played with WAIT 1).
- Dormant routines: two routines that nothing references (older copies of lever bodies; see Open questions). They do not run.
- Test vectors:
  - Init with state 675 clear: seven triggers registered; sounds added; object 0 initialised with bool 0.
  - Init with state 675 set: no triggers; objects 29, 31, 33, 27 and 1 initialised with bool 1, and objects 11, 14, 17 and 19 with bool 0; the sound calls are added on both paths.
  - Trigger index 2 with object 2 inactive and state 675 clear: state 675 set, object 1 off, object 2 on, camera 10.0.
  - Trigger index 1: LevelMsg 3 the first time it fires; nothing on later firings.
  - Update with breakable 0 broken: LevelMsg 4 on the first such tick only.
- Edge cases: messages 2, 3, 4 and 5 each show once per module load (shown-once words). The index-0 callback Sleeps one frame per trigger frame while the hero is occupied.
- Confidence: Medium overall. Init, update, the index 1 and 2 callbacks, the shown-once words and the exit are High. The two dormant routines are High (no reference). The lever puzzle (junction rules, test, solution) and trigger 4's solved path (state 675) are High. The call order inside the lever branches is Medium. Helper C is High for its call list.
- Open questions:
  - Meaning of state 675 (game state id; meaning unknown).
  - Resolved: the update's LevelMsg 4 has a shown-once word, so it shows once.
  - Whether object ids 0 to 34 and the locator ids (4 to 7) come from level data (not yet mapped).
  - Resolved: the button zones' sub-branches are the junction rules of the lever puzzle. Resolved: init has no further test on its object-11 calls (one InitObjOn and one AddAnimObjSound each); trigger 4's three partial outcomes follow junction 2 and junction 3 (see Unsolved); a pull of lever 7 at junction 3 d leaves the junctions unchanged (see the lever 7 rule; High).
  - Resolved: the selectors are 1, 2 and 3 for levers 5, 6 and 7, and 2 (plus 3 on the junction-3-d outcome, or 1, 2 and 3 when solved) for the test button (High).
  - Resolved: every trigger that init registers (with state 675 clear) runs in play when its gates hold. The two trigger-like bodies that do not run are two routines nothing references: older copies of lever bodies (a lever animation with camera 6, and a button-prompt lever with locator 7, animation 33 and camera 6, each setting a lever word). They are dormant (High).
  - Resolved: this module registers no routine for index 3; Init's seven registrations (indices 0, 1, 2, 4, 5, 6 and 7) are its only ones (High).
  - Movie id 50 and sound ids 7, 8 and 10 are not yet mapped to asset names.
  - The text id for the button prompts (from module data, id not yet mapped).


### 1176: Exterior, hero death watch and scripted encounter threads

- Family: Exterior.
- Exit: present. It makes no host calls. Load and unload hooks exist and make no host calls.
- Init, in order:
  1. Sets the maximum active AI count to 12.
  2. Fades the screen out (duration 0, wait option 0; struct argument not known).
  3. Registers the hero death callback (CB3) and the hero use-item callback (CB4), both with bool 1.
  4. Allocates two control threads.
  5. Starts control thread CB1 (encounter thread) and control thread CB2 (death watcher). Neither start is gated.
- CB3, hero death callback: runs once for each hero death (see SPEC-0055). It loops hero animation 34 for 100 s with control option 1, then waits 0.7 seconds. It then sets a module word, the hero-death marker, to 1 (the death watcher CB2 reads it). It sets no hit points and no game state, and it never heals, kills or revives. The default death is skipped for this callback (no state 16, no respawn). The hero, at 0 HP in state 17 (the hit-reaction state a lethal hit writes before damage) when the callback starts, is put in state 55 looping clip 34 for 6000 ticks, then state 33. The 0.7 s wait is 41 ticks (SPEC-0055 section 5 and its timing table), so the marker is set 41 ticks after the call and health stays unchanged until the hero-death cutscene and the reload of world 90.
- CB4, hero use-item callback: plays main sound 142 with no position and bool 0.
- CB2, death watcher (control thread, started from init):
  - Loops, sleeping 1 tick per pass, while the AI entity held in the encounter word is not dead (the word has file value -1; the encounter thread CB1 stores the entity of the scene it runs, 16, 15, 14, 13 or 12) and the hero-death marker (set to 1 by CB3) is 0. The loop ends when either holds.
  - Hero-death path (the marker is 1 when the loop ends, whatever the AI's state): if flag 380 is clear, sets it to true; runs the hero-death cutscene routine (see below); adds blackboard messages (type 2 value 10 world 90, and type 1 value -1 world 90); starts an exterior replacement load to world 90 (mode 0, no palette). No HUD reset on this path.
  - AI-death path, the win (the AI is dead and the marker is 0): waits 1 second, then runs the death sequence helper. The helper grabs hero control, turns off the hero's equipped state, fades out (0.7, waiting), deactivates the encounter entity, moves the hero to locator 0 with control 0, fades in (0.7), plays camera 5, plays hero animation 182, and ends with the win line: message 54 when flag 202 is set, else message 55 when flag 150 is set, else message 56 (wait 1). Then: adds blackboard messages (type 2 value 10 world 90, and type 1 value 1 world 90), resets the HUD, sleeps 1 tick, and starts an exterior replacement load to world 90 (pack 1023; mode 0, no palette).
  - Confidence: High for the two paths and their calls (the marker store and both branches are straight code).
- CB1, encounter thread (control thread, started from init; the five-round arena, High):
  - Round r: 4 when flag 384 is set, else 3 when 383 is set, 2 when 382 is set, 1 when 381 is set, else 0. The opponent is entity 12 + r (the encounter word the death watcher reads).
  - Slots: three slots; slot A (region 1177) and slot C (region 1178) are loaded at start (keep true, activate true), and slot B, a five-entry group, loads region 1179 + r (group index 2 + r). These loads take the SlotCreate results directly, with no stored module word.
  - Scene, in order: GrabEntityControl(opponent); spectators 0 to 10 each get control grabbed and loop one of animations 40, 37, 38, 55, 26, 70 and 69 (Random(0, 6)) forever; GrabHeroControl; MoveHero(locator 0); MoveEntity(11, locator 1); fade in 0.7 s; PlayCamera(6, wait); a fade cut; MoveEntity(opponent, locator 2); PlayCamera(4); the opponent plays its intro animation (70, 13, 70, 73 and 68 for r = 0 to 4; wait); a fade cut; PlayCamera(5); PlayHeroAnim(182, wait); a fade cut; PlayCamera(7); line 53, 52, 51, 50 or 49 for r = 0 to 4; a fade cut; the opponent is released; the camera returns to normal and the hero to idle. Then WalkEntityAndOrient(11, locator 1, wait, 1.1) and LoopEntityAnim(11, 55, forever). After release the opponent is in class state 8 (see Edge cases); its emerge animation 10 plays before agro when the fighter has one (the Rogue has none and goes straight to agro). This holds for all five rounds (identical spawn words; SPEC-0063).
  - Result in pack 1023 (SPEC-0045, 1023 activate-world routine and the arena master, entity 66 of pack 1023's own entity table, region 1035 ordinal 1, Shanra; 1176 itself names that character as its entity 11): the type 1 blackboard value is 1 for a win and -1 for a loss. A win sets the next rank flag (381, then 382, 383, 384 and 385); a loss sets flag 380. Pack 1023 also raises the hero's hit points to 50 percent when they are below it. The ladder is summarised with the other side activities in SPEC-0059, section 8.
- Helpers:
  - Shared scene player (called from the encounter thread, many calls): the body of the scene shape above.
  - Message chooser (reached from the death sequence helper and from the hero-death cutscene routine): the win line (54, 55 or 56 by flags 202 and 150) and the lose line (57 to 61 by round).
  - Death sequence helper (reached only from the death watcher's AI-death path): see CB2.
  - Hero-death cutscene routine (live: called from the death watcher CB2 on its hero-death path, with no further test in front of the call; High): fades out (0.7 s), grabs control of the opponent and of the hero, turns off the hero's equipped state, moves the opponent to locator 2 and the hero to locator 0, fades in (0.7 s), plays camera 5, LoopHeroAnim(190, 4.0 s), waits 3 s, a fade cut, plays camera 4, the opponent plays its victory animation (71, 73, 73, 22 and 73 for r = 0 to 4), and the lose line: message 61, 60, 59, 58 or 57 for r = 0 to 4.
- Dormant routines (the empty ones have no host calls):
  - A conversation chooser over flags 380 to 385: sets a conversation target, shows one of messages 8 to 15, and ends the conversation. No caller found.
  - Two empty routines with no host calls and no references.
- Update: no host calls.
- Threads: two (CB1 and CB2), both started unconditionally from init.
- Edge cases:
  - The win line: flag 202 set gives message 54; flag 202 clear and flag 150 set gives message 55; both clear give message 56.
  - Arena bounds (level data, High): the ring is ground entry 5, a 16-sided floor of radius about 30 around (190.6, 316), flat at height 27.54. A permanent invisible wall loop (ground entry 4: 47 two-sided wall records of type 6, top 67.53, depth 49.1, mask 0, so it is always on) closes the ring and a small grass lobe on its north-west side. Together they cover about x 159 to 220 and z 283 to 358. The hero cannot leave this area: for a hero of height 3.5 the wall acts on feet from 14.7 to 66.98, the ground inside is 26.4 to 29.9, and the highest object top inside is about 40. No ladder, climb panel or ledge lies inside. The opponents are kept on navigation mesh 0, which covers the ring. The spectators stand on four ramps outside the wall (meshes 1 to 16). Entity 11 stands on a platform at height 29.9 outside the wall, south of the ring (locator 1).
  - Locators (High): locator 0 (177.66, 27.92, 311.07) and locator 2 (205.60, 27.92, 322.52) are on the ring; locator 1 (205.46, 30.79, 281.89) is on entity 11's platform outside the wall. Entry records 3 to 5 lie outside the arena: entry 3 near the river's west house, entry 4 on a pier by the river, entry 5 on the far bank. Entries 4 and 5 are the deadly-ground respawn points of the pack's river and are never used in play (SPEC-0055, Open question 40).
  - Opponents start in class state 8 (default-state word 8, emerge distance 40.0; SPEC-0038 Behavior 40). After the intro scene releases the opponent, it stays at home until the hero comes within 40 x F1 (28 for an idle hero, 60 for a walking one), then emerges and enters agro. The hero's arrival point is 30.19 units from locator 2, so an idle hero does not wake the opponent; a hit does.
  - Arrival (High): the load from pack 1023 passes plane index 0. Pack 1176 has no teleport plane (its one plane record has no door bit), so the plane index is refused and the hero lands at entry record 0, (177.66, 27.92, 311.07), on the ring.
- Test vectors (disc data):
  - Ground probe at (x, z) = (177.66, 311.07) with no ceiling limit: candidates entry 5 (27.54) and entry 12 (27.47); result entry 5, height 27.54.
  - Ground probe at (203.5, 444.5): result entry 44 (deadly, index 4), height 16.29.
  - Arena wall: feet at y = 27.54 (the ring) or y = 30.74 (top of a standing jump from the ring): the wall acts (14.7 < y < 66.98). Feet at y = 14.0 or y = 67.0: it does not.
  - Ground probe at respawn entry 4, (159.51, 449.40): candidates entry 15 (27.89) and entry 44 (16.29); result entry 15, height 27.89.
- Confidence: High for init, the death watcher, the encounter thread, the win and lose lines and the return loads; Medium for the death callbacks' detail.
- Open questions:
  - Meaning of flags 150, 202 and 380 to 385: meaning unknown.
  - Resolved: the opponent is entity 12 + r for round r (see CB1). Locators 0 to 2 and entry records 3 to 5 are placed in Edge cases (High).
  - Resolved: slots and scene variants (see CB1).
  - Resolved: the AI entity the death watcher checks is the encounter entity that CB1 stores (entity 16, 15, 14, 13 or 12 by the scene chosen), not the hero.
  - Whether the dormant routines are reached by an indirect call.


### 1247: Exterior, trigger set with object swaps, an item-and-AI check and two world callbacks

- Family: Exterior.
- Exit: present. Its body makes no host calls.
- Hooks (load and unload) and update: no host calls. Nothing runs per tick.
- Init:
  - Registers a movable-block sound index (17), unconditionally.
  - Stops effect 0, unconditionally.
  - Registers 32 animation-linked sounds (object and sound numbers are level data, "id not yet mapped"), unconditionally: they run whether flag 78 is set or clear.
  - Registers the teleport callback CB8 (context 13) and the activate-world callback CB9, unconditionally.
  - Stops effects 1 and 2 under tests on the results of the earlier stop calls (not described).
  - Reads game-state flag 78.
  - If flag 78 is set:
    - Inits 18 objects (1, 0, 7, 9, 11, 12, 15, 17, 19, 21, 23, 25, 27, 28, 31, 33, 35, 4; forced flag per object as written).
  - If flag 78 is clear:
    - Inits 18 objects (2, 3, 7, 8, 10, 12, 15, 16, 18, 21, 23, 24, 26, 29, 31, 32, 34 and 5; forced for 7, 12, 15, 23, 31 and 5, not forced for the rest). Forced puts an object at the end of its animation, not forced at its start pose (SPEC-0028, section 7.1). So the plate objects 7, 12, 15, 23 and 31 start in their "plate up" pose, the gate panels 3, 8, 10, 16, 18, 24, 26, 29, 32 and 34 start in their start pose (closed, see "Gate panels" below), panel 21 starts in its raised pose (open; no callback of this module drives objects 20 and 21) and panel 5 in its raised pose (open).
    - Registers the triggers: CB1 on trigger 0, CB2 on 1, CB5 on 2, CB6 on 3, CB7 on 4, CB3 on 5, CB4 on 6.
  - With flag 78 set, none of the triggers is registered and the 18 objects of that case are all in their start poses, which for 0, 4, 9, 11, 17, 19, 21, 25, 27, 28, 33 and 35 are the raised poses: every gate is open for the rest of the visit.
- Trigger callbacks. Rules shared by CB2, CB3, CB4, CB5 and CB6 (the plate scenes; High):
  - Words of the callback's own trigger record: +0x10 is the state (1 first tick of contact, 2 while contact continues, 3 two ticks after the last contact tick, 0 one tick later; SPEC-0030, Layout section 3), +0x1c the object word (1 for the selected hero's body), +0x20 the shot word and +0x24 the attack word. The record is live: the trigger pass keeps writing it every tick of contact while the scene thread is parked inside an animation wait, and every re-read below sees the current value.
  - Start gate: the object word is non-zero, or the object, shot and attack words are all zero. The hero's body, a pushed block, an NPC or an enemy that stands in the volume without attacking all pass; a hero shot or an attacking body on its own does not. When the gate fails the callback does nothing.
  - "Held" means the state word reads 1 or 2 and "released" means it reads 3 or 0. A plate is a small rectangle that responds to the hero and to a movable block alike (SPEC-0048); a block that rests on it refreshes the record every tick, so the plate stays held as long as the block stays.
  - Each scene is split into Part A (swap in, first animation), a re-read of the state, then either the quick exit or the inside part, then the wait and Part B (swap back). Part A and the re-read run only when the thread starts with state 1. A thread that starts with any other state (2, 3 or 0) skips straight to the wait and Part B.
  - The re-read after the first animation answers "is the plate still held now". The first animation is the plate's own animation, which has 6 keys and lasts 20 ticks (1/3 s). The thread resumes in the tick the animation ends, before that tick's state decay. With c the tick of the last contact and T the tick the thread started, the state reads released exactly when c + 3 is at most T + 20, so a hero whose contact lasts at most 18 ticks (0.30 s) counting the first tick takes the quick exit, and a hero who stays 19 ticks or more takes the inside part (Medium-High; one tick either way is possible).
  - Quick exit (state released at the re-read): the plate object is swapped back to its "up" object and that object's animation is played without waiting, then the callback ends. Nothing else moves: the gate panels stay at their start poses, no camera is played and no wait follows. The visible result is the pre-scene state.
  - Inside part (state still held): the hero is returned to idle, the scene cameras and gate animations listed per callback are played, then the callback goes to the wait.
  - Wait: one tick at a time (Sleep 1) until the state word is exactly 0. That is one tick later than the "released" test, because 3 is not enough here.
  - Part B: swaps the gate panels and the plate back (deactivate the "open" objects, activate the "closed" objects), plays their animations without waiting (each panel lowers over its own animation, 16 keys = 1 s for most), and returns the camera to normal (preset -1, release true). The animations are not played backwards; every PlayAnim speed in this module is +1.0.
  - The effects started in Part A are forced starts of the placed effects of the pack; the stops name the same slots (CB2 differs, see below).
  - Gate panels (High for the numbers; Medium-High for "closed at the start pose, open when raised"): the swapped panels are thin solid walls with no floor of their own (no height patch); each is one thin rectangular volume (0.9 to 2.3 thick, 11.7 to 15.9 wide, 13.4 to 15.5 tall). A raised panel is 10 units higher (panel 3: 7.55, panel 29: 10.15), which lifts its bottom edge clear of the hero's head (feet + 3.5), and a deactivated panel has no wall at all. World boxes at the start pose (x range; z range; y range): panel 3: -87.2 to -71.3; 169.8 to 172.1; 8.2 to 23.7. Panel 26: -44.0 to -43.2; 1.5 to 13.9; 0.7 to 16.2. Panel 16: -44.6 to -43.7; 36.5 to 48.9; 2.2 to 17.8. Panel 18: 122.5 to 123.4; -43.6 to -31.1; 9.9 to 25.5. Panel 24: 93.9 to 94.8; -30.5 to -18.1; 7.5 to 23.1. Panel 29: -151.8 to -150.9; 64.9 to 77.3; -0.6 to 14.9. Panel 8: 61.8 to 62.7; -3.6 to 8.6; -0.4 to 15.1. Panel 10: 61.6 to 62.5; 43.4 to 55.7; -0.4 to 15.2. Panel 32: 121.7 to 122.6; 101.1 to 113.5; 8.0 to 23.6. Panel 34: 153.2 to 154.1; 101.0 to 113.4; 7.7 to 23.2. Each panel has a twin object for its raised or lowered pose (the "other" object of the pairs below), which has the same box shifted by the rise.
  - CB1 (trigger 0; High): the gate is: level state 0 clear, the object word non-zero and the state word equal to 1. Then: sets level state 0, deactivates objects 0 and 1, activates objects 3 and 2 (start poses), starts effects 20, 21, 22 and 23 (forced), plays animation 2 (wait; the plate), stops effects 20 to 23, plays camera track 4 (a plain track with no target, not waiting, camera end mode 0, speed 1.0), plays animation 3 (wait; panel 3 rises 7.55 over 18 keys, 1.13 s) and returns the camera to normal (preset -1, release true). Nothing lowers panel 3 again, so the arrival gate stays open from then on.
  - CB2 (trigger 1; the plate near (32, -1, 34); gate panels 8 and 10, High):
    - Part A: deactivates 9, 11 and 7; activates 8, 10 and 6; starts effects 13, 13 again, 14 and 15 (forced; effect 12 is never started); plays animation 6 (wait); stops effects 12, 13, 14 and 15.
    - Quick exit: deactivates 6, activates 7, plays animation 7 (no wait).
    - Inside part: returns the hero to idle; look-at camera track 5 on object 8 (no wait, camera end mode 1, speed 1.0); animation 8 (wait; panel 8 rises); look-at camera track 5 on object 10 (same arguments); animation 10 (wait); camera to normal.
    - Part B: deactivates 8, 10 and 6; activates 9, 11 and 7; plays animations 7, 9 and 11 (no wait); camera to normal.
  - CB5 (trigger 2; the plate at (141, 1, -71); gate panels 24, 26 and 29; High). Panel 26 is the west doorway panel that the route to the altar needs (see "What the burrow needs"):
    - Part A: deactivates 25, 27, 28 and 23; activates 24, 26, 29 and 22; starts effects 16, 17, 18 and 19 (forced); plays animation 22 (wait); stops effects 16 to 19.
    - Quick exit: deactivates 22, activates 23, plays animation 23 (no wait).
    - Inside part: returns the hero to idle; camera track 25 (no target, no wait, camera end mode 1, speed 0.8); animation 24 (wait; 66 keys, 4.33 s, panel 24 rises); camera track 24 (no wait, end mode 1, speed 0.6); animation 26 (wait; 16 keys, 1.0 s, panel 26 rises); camera track 23 (WAIT, end mode 1, speed 0.5); animation 29 (wait; 24 keys, 1.53 s, panel 29 rises); camera to normal. So panel 26 starts to rise about 4.67 s after the thread starts and is up 1.0 s later.
    - Part B: deactivates 24, 26, 29 and 22; activates 25, 27, 28 and 23; plays animations 23, 25, 27 and 28 (no wait); camera to normal.
  - CB6 (trigger 3; the plate at (150, 8, -36); gate panels 32 and 34, High):
    - Part A: deactivates 33, 35 and 31; activates 32, 34 and 30; starts effects 0, 1, 2 and 3 (forced); plays animation 30 (wait); stops effects 0 to 3.
    - Quick exit: deactivates 30, activates 31, plays animation 31 (no wait).
    - Inside part: returns the hero to idle; camera track 10 (no target, no wait, camera end mode 1, speed 1.0); animation 34 (no wait; 71 keys, 4.67 s); animation 32 (wait; 111 keys, 7.33 s); camera to normal.
    - Part B: deactivates 32, 34 and 30; activates 33, 35 and 31; plays animations 31, 33 and 35 (no wait); camera to normal.
  - CB3 (trigger 5; the plate at (-13, -1, 33)) and CB4 (trigger 6; the plate at (-112, 1, 82)) (High for both, each reads the other's record): two plates that share the gate panels 16 and 18 (twins 17 and 19). At the start each reads the other plate's state word (CB3 reads trigger 6's record, CB4 reads trigger 5's); "the other plate is held" means that word is not 0, so a word of 3 still counts as held. CB3's plate objects are 12 (up) and 13 (pressed); CB4's are 15 (up) and 14 (pressed). Both use the shared start gate above, and the choice between the two branches is made once, from the other plate's word at the start.
    - Other plate held, own state 1 (short branch): CB3 deactivates 12, activates 13, starts effects 4, 5, 6 and 7, plays animation 13 (wait), stops effects 4 to 7. CB4 deactivates 15, activates 14, starts effects 8, 9, 10 and 11, plays animation 14 (wait), stops effects 8 to 11. There is no re-read, no camera and no gate animation; the callback goes to the wait.
    - Other plate free (word 0), own state 1 (long branch): deactivates 17, 19 and the up object (12 or 15); activates 16, 18 and the pressed object (13 or 14). CB3 starts effects 4 to 7, plays animation 13 (wait) and stops effects 4 to 7. CB4 plays animation 14 (wait) and then starts effects 4 to 7 (forced) and never stops them. Re-read of the own state: released gives the quick exit (deactivate the pressed object, activate the up object, play the up object's animation without waiting, end; the panels stay in their start poses). Held gives the inside part: the hero is returned to idle; camera track 9 (no target, no wait, camera end mode 1, speed 1.0); animation 16 (wait; panel 16 rises); camera track 6 (WAIT, end mode 1, speed 1.0); animation 18 (wait; panel 18 rises); camera to normal. Then the wait.
    - Own state not 1 at the start: straight to the wait.
    - Part B, after the wait, re-reads the other plate's word. Free (0): deactivates 16, 18 and the pressed object; activates 17, 19 and the up object; plays the up object's animation, then 17 and 19 (no wait); camera to normal. Held (not 0): deactivates the pressed object, activates the up object, plays the up object's animation (no wait); camera to normal. So the gate stays open while either plate is held and closes only when the last plate lets go.
  - CB7 (trigger 4; registered only while flag 78 is clear; High): the item 29 wait.
    - If the hero already holds item 29 when the trigger fires: SetGameState 78, 269 and 270 at once.
    - Otherwise, on a first fire (+0x1c non-zero, kind 1): deactivates object 5, activates object 4, plays camera 8 (1.0 s), plays animation 4 (wait), returns to normal camera. Then, if AI 10 is dead, SetAI_Agro(10), which has no effect on a dead AI (quirk; reproduce the call). Then it waits, one tick at a time, until the hero holds item 29; then Wait(1.8 s), deactivates object 4, activates object 5, plays camera 8, plays animation 5 (wait), sets game-state flags 78, 269 and 270, and returns to normal camera.
    - No AI-dead requirement: the flags depend only on item 29.
    - Panels 4 and 5 are the altar gate (x -140.6 to -128.9, z -45.6 to -44.7). Panel 5 starts in its raised pose (open, from Init); the first fire swaps in panel 4, which starts raised and lowers 10 over 24 keys (1.53 s), so the gate shuts behind the hero; after item 29 is held panel 5 starts at its low pose and rises 10 over 16 keys (1.0 s), which opens the gate again. The gate's box at the low pose spans y 13.5 to 26.9 over ground at 14.3 to 14.6.
  - What the burrow needs (Medium-High). A coarse walkability search (2-unit grid, the ground model, the static tall walls and the closed gate panels as solid boxes) from the arrival point gives: with every panel closed, only the arrival strip (363 cells); with panel 3 open, 4,435 cells and no way to the altar or the Death Knight; with panels 3 and 26 open, the altar and AI 10 are reached; with every panel open except 26, they are not. So CB1 (the first plate, whose panel 3 plugs the 7-wide doorway between the pillars at x -83 to -76, z about 170) and CB5 (the plate at (141, 1, -71), whose panel 26 plugs the whole 8.5-wide west doorway at x about -44, z 3.3 to 11.8) are required to reach item 29; CB2, CB3, CB4 and CB6 only open side rooms and the east gates and are not required. The gates close again about 3 ticks after the plate is released, so panel 26 stays open only while the hero or a movable block holds plate 2. The pack has four movable blocks (SPEC-0048) whose start positions lie 24 to 37 units from plates 1, 2, 3 and 6; a block pushed onto plate 2 keeps panel 26 open for good (the scene thread sits in its wait and the resting block refreshes the record every tick), and it must still be there when the hero leaves with item 29, because setting flag 78 does not move any panel. Default if unresolved (rewrite choice): build all seven plate and altar scenes with the lists above and let a block on a plate hold it.
  - CB8 (teleport callback, context 13): shows a square button prompt (context 13). If button 0 is held: starts an exterior replacement load of world 147 (no locator, no color), waits 3 s.
  - CB9 (activate-world callback): debug print; reads the blackboard message of type 2 for world 147. If the value is 10: moves the hero to locator 0 (control mode 1) and sets the camera to normal (zoom 15, second value 0). Otherwise nothing.
- Update: no host calls. Nothing runs per tick.
- Threads: none. No control thread is started in this module.
- Edge cases and error handling:
  - CB3 and CB4 each read the other plate's state word and branch on it being 0 or not (short branch when the other plate is held, long branch when it is free; the earlier reading of this branch pair, with the sequences swapped, was wrong).
  - CB7 sets three game-state flags (78, 269, 270) once the hero holds item 29, whatever the state of AI 10.
  - CB2 to CB6 do nothing when the start gate fails (the object word is 0 and a shot or attack word is non-zero); no state is written then.
  - A plate scene whose thread starts with state 2, 3 or 0 does not run Part A: it waits for state 0 and then runs Part B, which activates the "closed" objects and plays their animations. Part B is harmless when Part A did not run (it only restores start poses).
  - A hero who taps a plate (contact of at most 18 ticks) sees the plate dip and pop back and nothing else move; this is the quick exit and it is the original behaviour, not a missing scene.
  - Level state 0 (CB1) is set once and never cleared here.
- Confidence: Medium-High.
  - High for init, exit, update and hooks (no host calls) and for the trigger indices.
  - High for the flat call lists of CB1 to CB7 (every object, animation, effect, camera track, speed and wait flag).
  - Medium-High for the quick-exit tick bound (one tick either way) and for "closed at the start pose, open when raised".
  - Medium-High for the burrow-needs rule (see "What the burrow needs"): a coarse walkability search, not a play test.
- Open questions:
  - Resolved (proactive (open close-out); Medium-High): flags 78, 269 and 270 are main-path progress flags. Their only script setter is this trigger (the witch-scene step, SPEC-0058 step 376). Flag 269 is read by packs 1248 and 1249; flag 270 by pack 1251; 78 only here. 269 is the done flag of quest rows 128 and 129 and the start flag of row 130, and 270 is the done flag of row 130 (SPEC-0049 quest rows). All of 78, 268, 269, 270 and 460 are also set by the start-point stage rows of stages 2 and 3, so start indices 3, 6, 7 and 8 begin with them on. The purpose of level state 0 here is not encoded beyond its readers (unknowable).
  - What item 29 is. Resolved: CB7's tests are the item 29 checks (the fang necklace, SPEC-0058).
  - Resolved (High): CB4 reads trigger 5's state word and CB3 reads trigger 6's. The branch on that word is not the "set and clear" pair of the earlier text: the other plate held gives the short sequence (one swap, one animation, then the wait) and the other plate free gives the long one (gate scene).
  - Resolved (High): the objects of CB3 and CB4 are 12, 13, 16, 17, 18, 19 and the effects 4 to 11 (lists above). The objects of CB2, CB5 and CB6 and the camera tracks, waits and speeds are listed above; the earlier summaries that said "sleeps 1 frame" at the end were wrong (the only Sleep is the one-tick wait for the state word to reach 0).
  - Resolved (Medium-High): the re-read of the state after the first animation tests whether the plate is still held (state 1 or 2); a hero whose contact ended at least 2 ticks before the animation ends takes the quick exit, so the scene is not always played in full.
  - Open (needs a runtime check): the exact tick at which the animation wait returns relative to the state decay (the 18-tick bound could be 17 or 19). Default if unresolved (rewrite choice): use 18.
  - Open (needs a runtime check): whether another way into the west half of the burrow exists beyond the coarse search (none found). Default if unresolved (rewrite choice): require panels 3 and 26 open for the walk to item 29.
  - Resolved: the word at +0x10 is the trigger state (1 first fire, 3 decaying), +0x20 is the shot word and +0x24 the attack word (SPEC-0030, section 3).
  - Which level objects the object and sound numbers refer to: "from level data, id not yet mapped".


### 1249: Exterior, trigger set with object swaps and teleport and world callbacks

- Family: Exterior.
- Exit: present. Its body makes no host calls.
- Hooks (load and unload) and update: no host calls. Nothing runs per tick.
- Init, split on game-state flag 80 for the object calls only:
  - Flag 80 set: inits 16 objects (13, 16, 20, 24, 10, 17, 21, 25, 47, 43, 45, 9, 1, 5, 7, 6; not forced).
  - Flag 80 clear: inits objects 13, 16, 20 and 24 (forced), then objects 10, 15, 19, 23, 48, 44, 46, 8, 0, 2, 4 and 3 (not forced).
  - In both cases (unconditional after the branches join):
    - Registers trigger callbacks: CB1 on trigger 2, CB2 on 3, CB3 on 5, CB4 on 4, CB5 on 6, CB6 on 18, CB7 on 16, CB8 on 17, CB9 on 1, CB10 on 0.
    - Calls StopEffect for effects 0 through 15 in order. The index starts at 0 and increments after each call; the loop exits after effect 15. The StopEffect return value does not control the loop.
    - Registers 31 animation-linked sounds (each ties an animation object to a sound id; object and sound numbers are level data, "id not yet mapped"). These run whether flag 80 is set or clear.
    - Registers the teleport callback CB11 (context 13) and the activate-world callback CB12.
- CB1 (trigger 2; fires when the trigger object's word at +0x1c is non-zero, its state word at +0x10 is 1 (the first tick of entry), and a door word (file value 0) is 0; CB1 stores the kind value into the door word at its start, and CB9 clears it, so CB1 runs once until trigger 1 re-arms it):
  - Deactivates object 12, plays a look-at camera on object 12 (camera 5, 1.0 s), moves the hero (locator 2, control mode 1), sets object 10 to time 0, plays animation 10 (wait), waits 1 s, returns to normal camera. Object 10 is the entrance gate panel in its raised (open) pose and animation 10 lowers it 15.18 over 20 keys (1.27 s), so CB1 closes the gate (a box at x 250 to 263, z 51 to 64); locator 2 is (246.6, -19.9, 48.7), just west of it. CB9 puts it back at its raised pose at once. The camera is returned with preset -1.
- CB9 (trigger 1; the gate is the object word non-zero and the state word 1, no door-word test; it clears the door word) (High):
  - When the gate passes: look-at-player camera track 7 (no wait, camera end mode 1, speed 1.0), deactivates object 12, activates object 10, sets object 10 to time 0 (its raised pose, the open gate), clears the door word.
  - On every start, whether or not the gate passed: returns the camera to normal with preset 7 and release true. Preset 7 is a locator index (SPEC-0030 row 35), so the camera is placed at locator 7 and the active hero is put in state 1. Because the fire step starts the callback each tick the volume is hit, this runs every tick the volume is held.
- CB2, CB3, CB4, CB5 (triggers 3, 5, 4, 6; the four plate scenes; High). All four plates are small rectangles clustered at (194.4, -0.6), (220.6, -0.6), (220.6, 25.1) and (194.8, 25.2) (triggers 3, 4, 5, 6); the pack's single movable block can reach any of them (SPEC-0048).
  - Shared rules. The start gate is that the object word +0x1c is non-zero or the object, shot and attack words are all zero (as in module 1247); Part A and the re-read run only when the state word at +0x10 is 1. The state word of the callback's own record is live, so each re-read sees the plate's current state. "Held" is state 1 or 2, "released" is 3 or 0. The first animation of every scene is the plate's own animation, 6 keys (20 ticks). The scene is Part A, a re-read of the state, then either the quick exit or the inside part, then the wait and Part B. A thread that starts with a state other than 1 skips to the wait and Part B (module 1247 has the same shape; the tick bound for the quick exit is given there: contact of at most 18 ticks gives the quick exit).
  - Differences from module 1247: the wait is WaitTillLeaveTrigger on the callback's own record, which returns as soon as the state reads 3 or 0 (one tick earlier than 1247's wait for exactly 0); there is no hero-to-idle call in the inside part; every PlayAnim speed is +1.0; the effects are started with the current world.
  - Quick exit: the plate object is swapped back and its animation played without waiting; the callback ends. The gate panels stay in their start poses, no camera is played.
  - CB2 (trigger 3; plate at (194.4, -0.6)): Part A deactivates 10, 13 and 1, activates 0, 12 and 11, starts effects 12, 13, 14 and 15 (forced), plays animation 11 (wait), stops effects 12 to 15. Quick exit: deactivates 11, activates 13, plays animation 13 (no wait). Inside part: look-at camera track 5 on object 12 (no wait, camera end mode 1, speed 1.0), animation 12 (wait; 51 keys, 3.33 s), look-at camera track 4 on object 0 (same arguments), animation 0 (wait; 25 keys, 1.6 s), camera to normal (preset -1, release true). Part B: deactivates 12 and 11, activates 10 and 13, plays animation 13 and animation 10 (no wait), deactivates 0, activates 1, plays animation 1 (no wait), camera to normal.
  - CB4 (trigger 4; plate at (220.6, -0.6)): Part A deactivates 17 and 16, activates 15 and 14, starts effects 8, 9, 10 and 11, plays animation 14 (wait), stops effects 8 to 11. Quick exit: deactivates 14, activates 16, plays animation 16 (no wait). Inside part: look-at camera track 4 on object 15 (no wait, end mode 1, speed 1.0), animation 15 (wait; 20 keys, 1.27 s), camera to normal. Part B: deactivates 15 and 14, activates 17 and 16, plays animations 16 and 17 (no wait), camera to normal.
  - CB3 (trigger 5; plate at (220.6, 25.1)): Part A deactivates 21 and 20, activates 19 and 18, starts effects 4, 5, 6 and 7, plays animation 18 (wait), stops effects 4 to 7. Quick exit: deactivates 18, activates 20, plays animation 20 (no wait). Inside part: look-at camera track 4 on object 19 (no wait, end mode 1, speed 1.0), animation 19 (wait; 20 keys), camera to normal. Part B: deactivates 19 and 18, activates 21 and 20, plays animations 20 and 21 (no wait), camera to normal.
  - CB5 (trigger 6; plate at (194.8, 25.2)): Part A deactivates 25 and 24, activates 23 and 22, starts effects 0, 1, 2 and 3, plays animation 22 (wait), stops effects 0 to 3. Quick exit: deactivates 22, activates 24, plays animation 24 (no wait). Inside part: look-at camera track 4 on object 23 (no wait, end mode 1, speed 1.0), animation 23 (wait; 20 keys), camera to normal. Part B: deactivates 23 and 22, activates 25 and 24, plays animation 24 (no wait) and animation 25 (WAIT), camera to normal.
  - Gate panels (High for the numbers; Medium-High for "low pose closed, raised open"): the panels that the scenes raise are thin solid walls with no floor of their own. Start-pose boxes (x; z; y): panel 0: 142.4 to 144.4; 7.2 to 21.5; -21.4 to 4.0 (rises 15.67). Panel 12 (twin 10, the entrance gate that CB1 and CB9 also drive): about 248 to 265; 48.5 to 65.7; -21.1 to -0.2 (rises 15.98). Panel 15 (twin 17): 158.4 to 169.0; 51.8 to 62.2; -21.3 to -4.9 (rises 11.35). Panel 19 (twin 21): 159.1 to 171.6; -36.6 to -24.7; -21.3 to -4.6 (rises 11.35). Panel 23 (twin 25): 249.9 to 263.6; -34.8 to -21.1; -21.0 to -4.2 (rises 10.59). Panel 8 (twin 9; driven by no callback of this module): 197.7 to 223.0; -53.8 to -48.9; -21.5 to -0.9 (rises 13.87).
  - What the plates open (Medium-Low): a coarse walkability search (2-unit grid, ground model, static tall walls, drops allowed) from the arrival point reaches none of the three levers with the start poses. Opening only panel 0 or only panel 15 reaches lever 17 (115.6, -29.8, 105.3); only panel 19 reaches lever 18 (119.4, -29.8, -79.7); only panel 23 reaches lever 16 (303.4, -29.8, -77.9). So CB3 and CB5, and one of CB2 and CB4, are on the path to flag 80; the panels close about 3 ticks after the plate is released plus their lowering time (about 1 s), so the hero must either run through before they shut or leave a block on the plate (one block for four plates). Not measured: whether a run from the plate to each doorway fits inside the closing time, and the way back up from the lever level. Default if unresolved (rewrite choice): build all four scenes as listed and let a block on a plate hold it.
- CB6 (trigger 18), CB7 (trigger 16), CB8 (trigger 17): the three levers that set flag 80, one template (High). Per lever (up object, down object, hero point): CB6 (47, 48, 22), CB7 (43, 44, 20), CB8 (45, 46, 21). Init with flag 80 clear shows the down objects 48, 44 and 46, so all three levers start down.
  - Gate: trigger word +0x1c non-zero and state +0x10 non-zero. A square button prompt is shown: context 2 while the lever's up object is shown, else context 1.
  - On button 0: the hero walks to the lever's point and is oriented. Then:
    - Lever down (down object shown): hero animation 139; the lever goes up (down object swapped for the up object, its animation played; look-at camera 4 on object 8). If the other two levers are up as well: camera 6 and SetGameState(80). Return to normal camera; sleep 1 frame.
    - Lever up (up object shown): hero animation 140; the lever goes down (the reverse swap and animations, look-at camera 4 on object 8); return to normal camera. No flag write.
  - So pulling each lever once sets flag 80 on the third pull, in any order.
  - If button 0 is not pressed, the callback does nothing else.
- CB10 (trigger 0): when the trigger word +0x1c is non-zero and state +0x10 is 1, flag 269 is clear, object 36's animation has finished and a shown-once word (file value 0) is 0: shows dialogue line 0 (wait), once per module load.
- CB11 (teleport callback, context 13): shows the square button prompt (context 13). When button 0 is held, starts an exterior replacement load for world 147 (no locator, no color), waits 3 s. The arguments of the prompt are passed in from the caller.
- CB12 (activate-world callback, world 147): reads the blackboard message of type 2 for world 147. If the value is 30: moves the hero to locator 0 (control mode 1) and sets the camera to normal with a zoom of 15 (second value 0). Other values: nothing.
- Threads: none. No control thread is started in this module.
- Edge cases and error handling:
  - The gate for CB2 to CB5 needs either +0x1c non-zero or +0x1c, +0x20 and +0x24 all zero; otherwise the whole body is skipped. A thread that passes the gate with a state other than 1 does not run Part A: it waits for the state to read 3 or 0 and runs Part B.
  - A tap on a plate (contact of at most 18 ticks) takes the quick exit and moves nothing else (see module 1247 for the tick bound).
  - Trigger index 0 (CB10) and the teleport callback share no state.
  - Flag 80 is set only by CB6 to CB8, on the pull that leaves all three levers up. The flag is never cleared here. A lever pushed back down before the third pull must be pulled again.
- Confidence: Medium.
  - High for init, exit, update and hook having no host calls, and for the lever rule of CB6 to CB8.
  - High for the flat call lists of CB1, CB9 and CB2 to CB5 (objects, animations, effects, cameras and wait flags); Medium-High for the quick-exit tick bound; Medium-Low for which plates the levers need. Medium for the call order inside one lever move.
  - High for the meaning of the word at +0x10 (the trigger state of SPEC-0030) and of the words at +0x1c, +0x20 and +0x24 (object, shot and attack words).
- Open questions:
  - Resolved (proactive (open close-out); Medium-High): flag 80 is this module's own "all three levers up" marker; no other module or quest row reads or sets it. The 147 in the call list is a world id, not a flag. Flag 269 is the main-path flag set by the pack 1247 witch scene (see module 1247); here it only gates dialogue line 0 of CB10. The word at +0x10 is the trigger state (1 first tick of contact, 2 inside, 3 and 0 after leaving; SPEC-0030).
  - Which object and effect ids map to which in-game objects: "from level data, id not yet mapped".
  - Resolved (High): each of CB2 to CB5 has Part A, a re-read of the callback's own state, a quick exit or an inside part, a wait and Part B; the earlier template that said "deactivates two objects, activates two others" was wrong for CB2 (three and three) and omitted the quick exit, the wait kind and which pair the second part uses. The pair in each Part B is the "closed" twin of the gate panels (for CB2: 10, 13, 1 on, 12, 11, 0 off).
  - Resolved (High): CB9 returns the camera with preset 7 on every start.
  - Open (needs a runtime check): whether the lever doorways can be passed in the closing time of the plate gates, and the way back up from the lever level. Default if unresolved (rewrite choice): hold the plate with the block for one of them and run for the others.
  - Resolved: the init stop-effect loop calls effects 0 through 15 in order and exits after effect 15, independent of StopEffect's return value.
  - Resolved: the lever order of CB6 to CB8 does not matter (flag 80 on the third lever up).


### 1324: Exterior, scripted triggers, object sets and a camera sequence

- Family: Exterior (its Init entry).
- Exit: present, empty. The exit export runs no host calls.
- Load and unload hooks: present. The load hook runs the module's load-time constructor list (SPEC-0030, Which level routines run in play), which holds one routine. It runs once when the module is linked, before Init: it sets the scene-done word (below) to QueryGameState(239), the current value of flag 239, and stores GetSystemTime() in a second module word that no routine reads (High). The unload hook's list is empty.
- Init, in order:
  1. ControlThreadsAllocate(world, 1). Unconditional.
  2. RegisterTeleportCallback(teleport callback, index 0, CONTEXT_MESSAGE 13, world). Unconditional.
  3. LoopAnim(29, speed 0.5, no world). Unconditional.
  4. The scene-done word (a module word, file value 0) splits the rest of init. The load-time constructor sets it to the value of flag 239 just before Init, and the solved-scene helper sets it to 1 when the puzzle is completed. So Init takes the zero side on a load while flag 239 is clear and the non-zero side on a load while flag 239 is set (High). Where the word is zero: InitObjOn(21, false), InitObjOn(15, false), InitObjOn(17, false), InitObjOn(19, false); ControlThreadStart(thread body, null argument); RegisterScriptTrigger(trigger callback 1, index 1), then index 0; InitObjOn(14, false), InitObjOn(0, false); RegisterScriptTrigger(trigger callback 2, index 2), then index 3, then index 4; InitObjOn(1, false). Where the word is non-zero (the solved layout; High): a debug line; InitObjOn(21, false), InitObjOn(16, true), InitObjOn(18, true), InitObjOn(20, true) (the three stage-1 plates shown pressed); ActivateMovableBlock(0, 0), ActivateMovableBlock(1, 0), ActivateMovableBlock(2, 0); MoveMovableBlockToLocator(block 3, locator 2) and MoveMovableBlockToLocator(block 4, locator 1) (the stage-2 blocks on their plates, as the stage-2 plate handlers place them); ActivateMovableBlock(3, 0), ActivateMovableBlock(4, 0); the beam word is set to 9 and the beam helper runs: it hides the other beam objects and loops the animation of beam object 9 (LoopAnim; when the beam word is -1 it instead activates object 14); the beam rhythm word is set to 1 and a 1.0 s period is stored, which turns on the update's object-time rhythm (Medium for the period's use). No trigger is registered and no thread is started on this side, so the plates do nothing.
  5. Unconditional: RegisterMovableBlockSoundIndex(8) for three blocks. A debug line is printed on the side the module word selects.
- Update (every tick): calls one helper. That helper checks game state 239 and a shown-once word (file value 0). While state 239 is clear and the word is 0, it sets the word and calls LevelMsg(message 0, no world, bool 1, WAIT_OPTION 1). So message 0 is shown once per module load, on the first tick while flag 239 is clear (High). The solved-scene helper reached by trigger callback 2 sets flag 239 and the scene-done word. Then, while the beam word is set, the update runs the beam rhythm (below; High).
- Trigger record fields used by both callbacks: the state word at +0x10 (1 entered, 3 just left; SPEC-0030 section 3), the trigger index at +0x14, and the block number at +0x28: the number of the movable block that fired the trigger, or -1 when the hero fired it. Only blocks press the plates (High).
- The puzzle: stage 1 is three block plates (triggers 2, 3 and 4); stage 2 is two plates (triggers 0 and 1) that each hold a mark (A for plate 1, B for plate 0). A "beam" object shows the stage-1 progress. The puzzle is solved when stage 1 is finished and both stage-2 marks are set (High).
- Plate geometry, blocks, locators and the bells' floor (High; the figures are measured on the disc).
  - The five plates are kind-4 box volumes of the pack's collision chunk with flags 0x0020 and a third extent above zero (0.340 for the stage-1 plates, 1.0 for the stage-2 plates), so a block fires them with the moving-rectangle test (SPEC-0048, The block's moving-rectangle test). Stage-1 plates (triggers 2, 3 and 4): corners (-20.221, 31.162, 1.266), (-8.449, 31.162, 1.266) and (2.918, 31.162, 1.266); each is 5.553 long along X, 0.340 along Z and 10 tall (y 31.162 to 41.162). Stage-2 plates: trigger 0 at corner (54.14, -5.0, 30.18) and trigger 1 at (54.14, -5.0, -31.046); 1.0 by 1.0 and 10 tall (y -5 to 5).
  - Blocks 0 to 2 are the bells (size 3.305, height 9.2, step 1.652). Blocks 3 and 4 are the stage-2 blocks (size 9.22, height 20.076, step 4.61). Locators: 1 (51.01, 0, 30.49) and 2 (51.01, 0, -30.46) beside the stage-2 plates; 3, 4 and 5 (y 32.40, z 12.76) the hero's places after plates 2, 3 and 4; 6, 7 and 8 the bells' centres (x -17.429, -5.666 and 5.711; z 13.349, 10.198 and 15.091; y 33.076); 9 and 10 (x 45.15, y 0.26, z 36.29 and -36.98) the hero's places after the stage-2 plates.
  - The thread's three moves centre each bell's footprint on its locator and then probe the ground at the footprint's corner (x -19.082, -7.319 and 4.059; z 11.697, 8.546 and 13.439) with a ceiling of 33.076 + 3.0 = 36.076. The probe finds 31.191 at all three corners, on the inline height patch of placed object 21 (SPEC-0018, Ground probe, step 4): a flat one-cell patch with 2 by 2 samples, all 31.1915, listed by the grid cells under the corner with a non-zero patch id. The pack's own patches there give only 0.0 and 0.25, which are lower. Init switches object 21 on before the thread runs, and each of the objects 22 to 28 carries an identical flat patch at the same height, so the floor stays under the bells whichever scene object is shown. A bell therefore stands on y 31.191 (top 40.391), inside the plates' height range, not on y 0.
  - The pushes: a bell moves along -Z in steps of 1.652, and its plate fires when the bell's position is within 0.6 x 3.305 = 1.983 of the plate's footprint, that is z at or below 3.590. Bell 0 (z 13.349) fires on its sixth push (z reaches 3.565 on step 35 of 38), bell 1 (z 10.198) on its fourth, on the last step (z 3.588), and bell 2 (z 15.091) on its seventh (3.567 on step 37). The test runs only while the hero's grid cell lists the plate (SPEC-0048); the cells within two cells (17 units) of a stage-1 plate list it. The hero is carried with the block, 3.152 behind its position (half the size plus 1.5; SPEC-0033, Behavior 4A.2), so when a bell fires its plate he stands at z 6.72 to 6.74, inside those cells (grid rows 4 to 8 of the plate's block, he is in row 7).
- Trigger callback 2 (plates 2, 3 and 4; High). State 1, fired by a block: the block is marked placed and the plate marked pressed; the plate object is swapped (15 -> 16 for plate 2, 17 -> 18 for plate 3, 19 -> 20 for plate 4), sound 9 and the plate animation play; MoveHero(locator 3, 4 or 5, control). Then the plate scene:
  - Object 21 shown and objects 22 to 28 hidden; hero animation 0; the block frozen. PlayCamera(2, 1.0); Wait(3.6 s); KillSound(9); Wait(1.0 s); camera returned.
  - Shown by the set of plates pressed: one plate: object 26 (plate 2), 27 (plate 3) or 28 (plate 4); two plates: 23 (plates 2 and 3), 24 (2 and 4) or 25 (3 and 4); all three: 22. Object 21 hidden. Placed blocks stay frozen; every other block gets ReacquireMovableBlockHeightPatch with height 3.0.
  - While stage 1 is not finished, the beam object is 13 with one plate pressed (a 4.0 s rhythm), 12 with two (2.0 s) and 14 with three (1.0 s).
  - With all three pressed: PlaySound(4), plus PlaySound(5) when both stage-2 plates are held; stage 1 is marked finished; PlayCamera(8, wait, 0.5); when only one stage-2 mark is set, that stage-2 plate's handler runs again; the camera is returned. When both stage-2 marks are set, the callback reaches the solved-scene helper, which sets flag 239 before playing FMV 23.
- Trigger callback 1 (plates 0 and 1; only while the puzzle is unsolved, fired by a block; High).
  - State 1: plate 1 (index 1): block 3 moved to locator 2, camera 1, hero moved to locator 10 (the hero move only while stage 1 is unfinished), mark A set. Plate 0 (index 0): block 4 moved to locator 1, camera 0, hero to locator 9, mark B set. Each: FadeScreenOut(0.5, wait); the block move; PlayHeroAnim(0); FadeScreenIn(0.5); PlayCamera(camera, wait, 1.0); Wait(1.0 s). The helper then fades out, returns the camera to normal, fades in without waiting and issues a second normal-camera return.
  - State 3 (the block left the plate): the plate's mark is cleared and the beam object becomes 14 when no mark is left, else 10 or 11 (by the mark that remains).
- Solved-scene sequence (High for its call sequence and trigger condition): after stage 1 is complete and both stage-2 plate marks are set, the next event on a stage-1 plate callback reaches the scene helper. The helper plays the paired sounds, waits 2.0 seconds, sets flag 239 and the scene-done word, fades out for 2.0 seconds, stops sounds 4 and 5, plays FMV 23, fades in for 0.5 seconds, then repeats the paired sounds and 15-tick sleep ten times. It contains no camera return; the separate plate-camera helper returns its own camera before this completion branch.
- Flags (SPEC-0058 step 25): this module sets 239 only. Flags 238 and 240 are set by pack 1344's triggers 0 and 1 (SPEC-0039).
- Thread body (started from init): SetToNormalCamera(5.0, 0.0, bool 1); MoveMovableBlockToLocator(block 0, locator 6); MoveMovableBlockToLocator(block 1, locator 7); MoveMovableBlockToLocator(block 2, locator 8). No loop or wait calls, so the thread is a one-shot setup.
- Teleport callback (teleport index 0, message 13): AddBlackboardMessage(type 2, value 70, eWorlds 155); StartExteriorReplacementLoad(eWorlds 155, -1, no colour).
- Reached routines (static):
  - The beam rhythm (High), run by the update on every tick while the beam word (a module word, file value 0) is set. The plate scenes write the word and the period P: one plate pressed: P = 4.0 s; two: 2.0 s; three: 1.0 s; none: the word is cleared (P = 0). Init's non-zero side (puzzle already solved) sets the word with P = 1.0 s. Each tick: when the clock is more than P past the last beat, a beat: the beat stamp is set to the clock, a beat counter goes up by 1 and wraps to 0 when it reaches 61, SetObjToTime(object 0, counter / 30 s), and a target value goes up by 15 (when it was already 360 or more it is set to 15 and the running value to 0). Then, on every tick, while the running value is below the target it goes up by 3.0 and objects 1 to 7 are each set to time running value / 30 s; when the running value reaches 360 both values return to 0. So at each beat objects 1 to 7 advance 0.5 s of their animation over five ticks, and object 0 steps one frame of 1/30 s. Test vector: P = 4.0, beat stamp 0, counter 0, both values 0, clock 4.01: a beat; object 0 at 1/30 s; objects 1 to 7 at 0.1, 0.2, 0.3, 0.4 and 0.5 s on that tick and the next four; the next beat on the first tick with the clock above 8.01.
  - A routine with no host calls, reached from trigger callback 2 (no gate inside that callback).
- Other dormant routine: one routine has no caller or registration and makes no host call. The update's object-time helper and the flag-239/system-time constructor are live: the update calls the former, and the load hook runs the latter from its constructor list. The solved-scene sequence is reachable from trigger callback 2 (High, static).
- Test vectors:
  - Plate 0 (trigger 0) entered by a block (state 1, block number 2): MoveMovableBlockToLocator(4, 1), PlayCamera(0, wait, 1.0), hero to locator 9, mark B set.
  - Plate 2 entered by a block with plates 3 and 4 free: object 16 shown, hero to locator 3, then object 26 shown and the beam object 13 (4.0 s rhythm).
  - A plate entered by the hero (block number -1): nothing.
  - Bell floor (disc): after the thread's move to locator 6, bell 0's footprint corner is (-19.082, 33.076, 11.697) and the ground probe with ceiling 36.076 returns 31.191 (placed object 21's patch), so the bell stands at y 31.191 and plate 2 fires on its sixth push. With only the pack's own patches the answer would be 0.25 or 0.0 and no stage-1 plate could ever fire.
  - Teleport index 0: AddBlackboardMessage(2, 70, 155), then the exterior replacement load.
- Edge cases: the update message shows once per module load while state 239 is clear (shown-once word). The solved-scene FMV routine contains ten closing-chime repetitions.
- Confidence: Medium. Init order, the thread, the teleport, both trigger callbacks, the solved-scene FMV sequence and its trigger-callback route are High. The beam rhythm is High.
- Open questions:
  - Resolved: trigger fields A, B and C are the state word, the trigger index and the block number (-1 for the hero).
  - Resolved: the trigger indices map to the collision chunk's kind-4 box volumes in record order: 0 and 1 are the stage-2 plates (corners (54.14, -5.0, 30.18) and (54.14, -5.0, -31.046)), 2, 3 and 4 the stage-1 plates (Plate geometry, blocks, locators and the bells' floor, above; High).
  - Resolved: the update message has a shown-once word, so it does not repeat each tick.
  - Resolved: the word that splits init is the scene-done word. Init runs only at the start of a lifetime of the module (Shared rules, Lifetime), right after the load-time constructor has set the word to the value of flag 239. The non-zero side therefore runs in play on every load while flag 239 is set (High).
  - Resolved in part: movable blocks 0 to 4 and locators 1 to 10 are mapped (Plate geometry, blocks, locators and the bells' floor, above). Objects 15 to 20 are the plates (rest positions (-17.44, 30.79, 3.10), (-5.65, 30.79, 3.10) and (5.66, 30.79, 3.10), the unpressed object first), objects 21 to 28 are the floor objects (each with the flat patch at y 31.19) and objects 9 to 14 the beam objects (rest position (51.3, 0, 84.0)). The deactivate calls in the helpers also take objects 12, 13 and 14 on some paths.
  - Resolved: locator ids 1 to 10 (Plate geometry, blocks, locators and the bells' floor, above).
  - Resolved: unregistered callbacks. The only routine address held in the module's data is the load-time constructor list entry, which the load hook runs. Every other routine is reached by a call or a registration (High).
  - Resolved (R-0103, High for the script path): pack 1324's registered callback for triggers 2, 3 and 4 reaches the FMV-23 scene helper when all three plate indicators are active and both movable-block state words are nonzero. That helper sets flag 239 before FMV 23. The scene is a live completion branch, not an uncalled helper; physical trigger and block access remain unconfirmed.


### 1353: Exterior, proximity conversations, button triggers and three threads

- Family: Exterior.
- Exit: present, no host calls.
- Update: every tick it runs the statue-puzzle check (below). It runs in play.
- Init, in order:
  1. StopEffect(effect 0, world). Unconditional.
  2. Registers script triggers: index 2 (trigger callback Z2), index 1 (Z1), index 0 (Z0), index 3 (Z3), indices 7 and 8 (Z78), indices 4, 5 and 6 (Z456). The four registrations for indices 2, 1, 0 and 3 are made unless flag 353 and flag 104 are both set (that is, when flag 353 is clear, or flag 353 is set and flag 104 is clear). The five for indices 7, 8, 4, 5 and 6 are unconditional (High).
  3. Calls helper H1 unconditionally. H1 registers entity-proximity callbacks for ids 16 to 24 (P16 to P24), bool 1 each, with no flag test (High).
  4. RegisterTeleportCallback(index 0, teleport T0). Unconditional.
  5. RegisterActivateWorldCallback(W). Unconditional.
  6. ControlThreadsAllocate(3) and the starts of thread X and thread Z: unconditional. Only the slot setup depends on flag 104: when flag 104 is clear, SlotAllocate and one SlotCreate; when it is set, SlotAllocate and two SlotCreate calls.
- Proximity callbacks (same pattern for each: a conversation):
  - P16 (the map seller). If flag 612 is set and flag 686 is clear: RemoveAllOfItemFromHero(item 33), SetGameState(686, true), then the conversation with line 53. Otherwise it runs the shop, driven by a shop counter word S (a module word, file value 0, advanced by this routine only):
    - First it skips maps the hero already holds: if S is 1 and the hero has treasure map 36, S becomes 2; then if S is 2 and the hero has map 40, S becomes 3; then if S is 3 and the hero has map 38, S becomes 4.
    - S = 0: S becomes 1; line 54 (the greeting).
    - S = 1: conversation, yes/no prompt line 55. If the answer is yes and the hero has at least 200 Neopoints: GiveItemToHero(item 175, treasure map 36, no notification), RemoveNeopoints(200), line 62. If the answer is no: line 65. If the hero has fewer than 200 Neopoints: line 68. In every case S then goes up by 1.
    - S = 2: the same with prompt line 56, map 40 for 200 Neopoints, and lines 63 (bought), 66 (declined) or 68 (too few Neopoints); S goes up by 1.
    - S = 3: the same with prompt line 57, map 38 for 300 Neopoints, and lines 64, 67 or 68; S goes up by 1.
    - S = 4: line 58, and S goes back to 1, so the offers repeat (maps already held are skipped).
    - S = 5, 6 or 7: lines 59, 60 or 61, and S becomes 6, 7 or 5. No path of this routine sets S to 5, so these lines do not show (Medium).
    - Each chosen line is shown with the conversation target set to the entity and ended after the line (High for the calls and amounts; Medium for the line-per-outcome pairing).
  - P17: random conversation. With flags 690 and 688 clear, Random(1 to 3) picks one of lines 87, 90, 93.
  - P18: if flags 688 and 690 are set: SetConversationTarget, LevelMsg(line 20), EndConversation. If 688 is set and 690 is clear: LevelMsg(line 23).
  - P19: same pattern as P18 with lines 8 and 11.
  - P20: random conversation, lines 69, 72, 75 (flags 690 and 688 clear).
  - P21: random conversation, lines 35, 38, 41.
  - P22: random conversation, lines 44, 47, 50.
  - P23: random conversation, lines 26, 29, 32.
  - P24: random conversation, lines 78, 81, 84.
- Statue puzzle (Z0, Z1, Z2, the update check and Z3). Each of the three statues has a record in module data: its object, its turn angle in degrees, and a done word (file value 0). Thread Z sets the objects and start angles (below).
  - Z0, Z1, Z2 (one statue each): when the hero is inside and the door word (a module word, file value 0) is 0: without the button, a square-button prompt (context 4) is shown. With button 0 pressed: BlockPlayerControl(-1), KillSound(7) and PlaySound(7), the statue turns by 45 degrees over 30 ticks (SetAnimObjAdditionRotY steps), KillSound(7), the angle becomes angle + 45; the statue's done word is set to 1 when the angle (as a whole number) is a multiple of 360 and cleared to 0 otherwise; then Wait(0.5) and RestorePlayerControl(-1) (High).
  - Update check (every tick): when all three done words are 1 and the door word is 0: SetGameState(694, true), PlaySound(8), PlayAnim(3) and PlayAnim(4) (no wait), DeactivateObj(6), ActivateObj(5, not forced), PlayCamera(6, wait 1, camera wait 1), a wait, ReturnToNormalCamera(-1, true), and the door word set to 1. This is the setter of flag 694 (High).
  - Z3 (trigger 3): when the record's state word is 1, its object word is non-zero and the door word is set: AddBlackboardMessage(type 2, value 10, world 158) and StartExteriorReplacementLoad(world 168, -1, no colour), which loads pack 1392, the shrine cluster that leads on to pack 1394 (High).
- Script triggers (other):
  - Z4, Z5, Z6 (shared sign routine; High): they are the three shop signs. The routine loops while the record's state word is non-zero, one pass per tick (Sleep(1) at the end of each pass). In a pass where the record's object word is non-zero (the selected hero's body is in the volume; a shot or a block does not count), it shows the square-button prompt (context 3) and checks button 0; when button 0 is pressed it shows one line with LevelMsg(line, W, true, wait 1), chosen by the record's trigger index: index 4 (Z4) line 3, index 5 (Z5) line 2, index 6 (Z6) line 1. The line texts are the sign names: line 1 The Curious Aisha, line 2 Altadorian Trove, line 3 Altador Armory. The routine makes no object, effect, camera, flag or item call and sets no conversation target.
  - The sign volumes are moving rectangles (flag 0x0020, extents 4.742 by 7.0): Z4 at (214.14, 0.07, -42.89), Z5 at (361.66, -3.66, -133.68) and Z6 at (428.25, -3.78, -94.71). Standing in one and pressing button 0 shows its line; the call waits until the box closes, and the check then repeats on the next pass while the hero is still inside.
  - Z7 and Z8: starts thread Y with argument 3 (record +0x14 is 8) or argument 1 (record +0x14 is 7), when flag 104 is set, the record's state word is 1 and the argument differs from the one stored by the last start.
- Teleport T0 (teleport index 0): if the button is pressed and flag 104 is set: AddBlackboardMessage(type 2, value 10, world 158), StartEnterInterior(eWorlds 175, -1, -1) (pack 1405), wait 3. If the button is pressed and flag 104 is clear: LevelMsg(line 0, wait 1).
- Activate-world callback W: DebugPrintf, then reads blackboard value (type 2, world 158). If the value is 10: MoveHero(1, control 1). If it is 20: MoveHero(3, control 1). Then, always, the message is cleared (value -1). If the value read was not -1: SetToNormalCamera(10, 0, true); if it was 20: StartEffect(0) as well.
- Threads:
  - Thread X (started in init): reads flag 104. If set: DebugPrintf and SlotLoad(the stored slot, slot set 2, keep true, activate true). If clear: DebugPrintf and SlotLoad(the stored slot, slot set 0, keep true, activate true).
  - Thread Y (started from Z7 and Z8): SlotReset(the stored slot), then SlotLoad(the stored slot, slot set 1, activate true) for argument 1 or SlotLoad(the stored slot, slot set 3, activate true) for argument 3, each with a debug line.
  - Thread Z (started in init): sets up the three statue records: Z0's statue is object 2, Z1's is object 1 and Z2's is object 0. If flag 694 is clear their start angles are 270, 90 and 45 degrees; if it is set, all three are 0. It initialises the three statue objects (not forced) and applies their angles. Then, if flag 694 is set and not both flags 353 and 104 are set: InitObjOn(5, false), InitObjOn(3, true), InitObjOn(4, true) and the door word set to 1 (the puzzle stays solved and Z3's load is open). Otherwise: InitObjOn(6, false), InitObjOn(3, false), InitObjOn(4, false) (High).
- Helper H1: the helper init calls in step 3. It registers the nine proximity callbacks; init makes no other proximity registration.
- Helper H3: the statue-turn step: Sleep and SetAnimObjAdditionRotY in a loop of one-tick steps (used by Z0 to Z2).
- Dormant routine: conversation with entity argument, lines 4 and 6, gated on flags 691 and 690. No callback or thread references it, so it does not run.
- Test vectors:
  - Statue with start angle 270: two presses reach 360, so its done word is 1 after the second press; a third press makes it 405 and clears the word. Starts of 90 and 45 need 6 and 7 presses.
  - P16 with S = 1, no map 36 held, answer yes, 250 Neopoints: map 36 given, 200 Neopoints taken (50 left), line 62, S = 2.
- Edge cases:
  - Most proximity conversations use random line choice, so the same entity can say different lines.
  - Flags 688 and 690 are read by several conversations and written by none in this module.
  - The statue triggers stop working once the door word is set; on a later load with flag 694 set, thread Z sets the door word at once.
- Confidence: High for init, the statue puzzle, Z3, the map shop calls, T0, W and the threads. Medium for the P16 line pairing and for the proximity lines of P17 to P24.
- Open questions:
  - Meaning of flags 104, 353, 612, 686, 688, 690, 694 beyond their use here. Resolved for 694: the statue puzzle is solved.
  - Resolved: the Z4 to Z6 prompts refer to no object or effect; they are shop signs that show one line each (index 4 line 3, index 5 line 2, index 6 line 1; High).
  - The second value of GiveItemToHero for item 175 is the treasure-map id (36, 40 or 38), matching the map test used for the skip; how the item code stores it is in SPEC-0021.


### 1363: Exterior, gated init, teleports, triggers and a long Mortog thread

- Family: Exterior.
- Exit: present, no host calls.
- Update: present, no host calls.
- Init (High):
  - Flag 693 set: objects 3 and 4 at their end pose (InitObjOn true) and trigger 1 disabled; flag 693 clear: objects 3 and 4 at their start pose (InitObjOn false).
  - Flag 692 set: objects 0 and 2 at their end pose and the puzzle marked done (a module word); flag 692 clear: objects 0 and 2 at their start pose.
  - Unconditional: RegisterActivateWorldCallback(W14); RegisterScriptTrigger for indices 0 to 8; RegisterTeleportCallback for indices 0, 1 and 2; RegisterEntityProximityCallback for ids 37 to 42; InitMortogBehavior(world, 3, 6, 7, 25.0, 30.0) (the last two are the two hold times); SlotAllocate(1); the slot = SlotCreate with set list {0} when flag 106 is clear, or {1, 3, 2} when it is set; ControlThreadsAllocate(2); and ControlThreadStart(Mortog thread M), once.
- Ease curve used for object 2 (High): object 2's vertical offset at step n of L = 40 steps, with full depth a = -0.25, is a x 2 (n / L)^2 for n <= L / 2 and a x (1 - 2 (1 - n / L)^2) for n > L / 2. So step 0 gives 0, step 20 gives -0.125 and step 40 gives -0.25.
- Script triggers:
  - Index 0 (trigger A, a pressure plate; High): while the puzzle is not done, when the hero steps on it: object 2 is lowered fully, one step per tick to step 40 (on the first use in a module lifetime with PlayCamera(4, wait 0, camera wait 1, speed 1.0), which a camera latch word then turns off); object 1 deactivated, object 0 shown and played (PlayAnim(0, wait 1)). Then WaitTillLeaveTrigger. Then object 2 is raised back to the current puzzle level (the step the Mortogs hold it at), object 0 deactivated, object 1 shown, PlaySound(8) and object 1 played; ReturnToNormalCamera(-1, true) after the first-time camera. So the door is held open only while the hero stands on the plate.
  - Index 1 (trigger B): if inside and button 0 is pressed, PlayAnim(3) and PlayAnim(4), then SetGameState(693, true). If not pressed, shows a square-button prompt (context 6).
  - Index 2 (trigger C): if inside and button 0 is pressed: AddBlackboardMessage(type 2, value 10, world 178), AddBlackboardMessage(type 2, value 10, world 160), StartExteriorReplacementLoad(world 178, -1, no color), wait 3. If not pressed, a prompt (context 13).
  - Index 3 (trigger D, the shrine door; High): if inside and button 0 is pressed: if the puzzle is done (flag 692 set at load, or the Mortog thread finished in this visit) or flag 355 is set: AddBlackboardMessage(type 2, value 20, world 174), AddBlackboardMessage(type 2, value 20, world 160), StartExteriorReplacementLoad(world 174, -1, no color), wait 3. Otherwise: LevelMsg(line 0, wait 1).
  - Indices 4 and 5 (shared trigger E; High): runs while the record's +0x10 word (inside) is non-zero, one tick per pass (Sleep(1)), until the hero leaves. On each pass where the object word at +0x1C is non-zero, it shows the square-button prompt (context 3), and when the confirm button is pressed (button 0, bool 1) it shows LevelMsg line 1 for index 4 or line 2 for index 5 (bool 1, WAIT 1), chosen by the record's +0x14 word (the trigger index).
  - Indices 6, 7 and 8 (shared trigger F; High): when the record's +0x10 word is 1, the object word at +0x1C is non-zero and flag 106 is set, it starts thread G with an argument chosen by the trigger index in the +0x14 word: index 7 gives 0, index 6 gives 1, index 8 gives 2. There is no further test and no once-guard: every such fire starts a new thread G.
- Teleport callbacks:
  - Index 0: if button pressed and flag 106 set: AddBlackboardMessage(type 2, value 40, world 160), StartEnterInterior(eWorlds 184, -1, -1). Otherwise LevelMsg(line 0, wait 1).
  - Index 1: same shape with value 50, world 160 and eWorlds 185.
  - Index 2: same shape with value 30, world 160 and eWorlds 177.
- Activate-world callback W14 (world 160; High): DebugPrintf; reads the blackboard value (type 2, world 160) and moves the hero (with control): 10 to locator 1, 20 to 3, 30 to 5, 40 to 2, 50 to 4. The value is reset to -1, and when it was not -1, SetToNormalCamera(15.0, 0, true).
- Proximity callbacks, ids 37 to 42 (High). Each talk is SetConversationTarget(the talked entity, true), one line (wait 1) and EndConversation. A routine has three sets of three consecutive lines. Choice of set, per talk: if flag 101 is set it draws X = Random(1, 3); if flag 103 is set it draws Y = Random(1, 3) (a clear flag makes no draw). Set 1 is used when flag 101 is set and X is 1; otherwise set 2 when flag 103 is set and Y is 1; otherwise set 3, which is the fallback shown whenever flag 101 or flag 103 is clear or the draws are not 1. The routine keeps one counter in module data (0 at load) shared by its three sets and no marker: the line is the set's first line plus the counter, and the counter then goes 0, 1, 2, 0 on successive talks whichever set was used, so a change of set does not restart the cycle. With both flags set the chances are 1/3 for set 1, 2/9 for set 2 and 4/9 for set 3; with only flag 101 set 1/3 and 2/3 (sets 1 and 3); with only flag 103 set 1/3 and 2/3 (sets 2 and 3); with neither flag set, set 3 always. First line of sets 1, 2 and 3:
  - Id 37: 48, 51, 54 (counter cycles the first line, the next, the next: 48-50, 51-53, 54-56).
  - Id 38: 39, 42, 45 (sets 39-41, 42-44, 45-47).
  - Id 39: 21, 24, 27 (sets 21-23, 24-26, 27-29).
  - Id 40: 30, 33, 36 (sets 30-32, 33-35, 36-38).
  - Id 41: 12, 15, 18 (sets 12-14, 15-17, 18-20).
  - Id 42: 3, 6, 9 (sets 3-5, 6-8, 9-11).
  - Each id has its own counter. Example, id 37 with both flags set: a first talk with X = 1 shows line 48 (counter 0); a second with X = 3 and Y = 1 shows line 52 (set 2 plus counter 1); a third with X = 2 and Y = 2 shows line 56 (set 3 plus counter 2); a fourth with flag 101 clear and flag 103 clear shows line 54 (counter back at 0).
- Mortog thread M (started once from Init; High). It keeps a shake count, a rise counter and the last count, all 0 at start.
  1. When flag 106 is clear: SlotLoad(the slot, set 0).
  2. While the puzzle is not done, each tick: n = NumMortogsOnTarget.
     - If n is above the shake count and below 4: StartCameraShakeMode(0, 1.0, 0.3) and the shake count goes up by 1.
     - When n is above the last count (a rise): the rise counter goes up by 1; on odd-numbered rises only (1st, 3rd, ...) both hold times go up by 5 (ChangeMortogTimes 30 and 35 on the first, then 35 and 40, and so on); PlaySound(11); object 2 moves down ten steps of the ease curve, one per tick (SetAnimObjPositionOffset); KillSound(11).
     - When n is below the last count (a fall): object 2 moves up ten steps, one per tick.
     - When n is 4: the puzzle is done; ChangeMortogTimes(0, 0); PlayCamera(4, 1.0); object 1 deactivated, object 0 shown; PlaySound(7); PlayAnim(0, wait); Wait(1.0 s); camera returned; SetGameState(692). The thread then ends.
  - Because trigger D tests the done word, the shrine door opens in the same visit, right after flag 692 is set.
- Thread G (started from trigger F): SlotReset(the stored slot) and then SlotLoad(the stored slot, set, keep true, activate true) with set 3 for argument 0, set 2 for argument 1 and set 1 for argument 2; each branch prints a debug line. So trigger 7 loads set 3, trigger 6 set 2 and trigger 8 set 1, each replacing the characters loaded before (High). Test vector: flag 106 set, the hero enters trigger 8: thread G resets the slot and loads set 1.
- Switch-selected branches (Medium): two branches move the hero to locators 2 and 4 with control kept.
- Helper routine with no host calls.
- Dormant routines: none in this module.
- Transitions (five, all from registered callbacks): trigger C starts an exterior load to world 178 (pack 1411); trigger D starts an exterior load to world 174 (pack 1398) when the Mortog puzzle is done or flag 355 is set; teleport index 0 enters world 184 (pack 1423), teleport index 1 enters world 185 (pack 1425) and teleport index 2 enters world 177 (pack 1409). Each teleport index is listed with its own destination above. Which branch of trigger C or D runs in play depends on the gates noted here (Medium).
- Test vector (Mortog thread, flag 106 set): n goes 0, 1, 2 on three ticks: two camera shakes; hold times 30 and 35 after the first rise, unchanged after the second; object 2 at step 20 after both moves (offset -0.125). n then drops to 1: object 2 back to step 10.
- Edge cases:
  - The Mortog thread is started once per module lifetime; with flag 692 already set at load it makes only its step-1 slot load (when flag 106 is clear), because the puzzle is already done.
  - Hold times rise only on odd-numbered rises, so a Mortog that steps off and back on raises them every second time.
  - Resolved (proactive (open close-out); High): flags 692 and 693 are set and read only by this module; no other level module, quest row or stage row uses either. The pack 1398 entry gate described above is this module's own check at its shrine teleport.
- Confidence: High for init, the Mortog thread, trigger A, trigger D, the teleports and the activate-world body. High for triggers E and F and thread G. Medium for triggers B and C and the proximity lines.
- Open questions:
  - Resolved: Init's gates are flags 693, 692 and 106 as listed.
  - Meaning of flags 106, 355, 692, 693, 694 and 101 and 103 (meaning unknown).
  - Resolved: the Mortog count test is NumMortogsOnTarget against 4, and InitMortogBehavior's floats are 25.0 and 30.0. Resolved: the offset call stores X/Y/Z components, with Y as vertical (SPEC-0030 row 16; confirmed by module 1394 offset calls).
  - Which world 160 is (eWorlds id as written; the packs of the five transition destinations are given above).
  - Resolved: the gates of Init, the Mortog thread and triggers A and D are given above. Resolved: the gates of triggers E and F (see their bullets; High). Still open: the proximity fallbacks.


### 1371: Exterior, blackboard-gated start, triggers, hero death and a walk-in sequence

- Family: Exterior.
- Exit: present, no host calls.
- Update: calls the wave controller every tick (unconditional); the update body itself makes no other host call.
- Wave controller (Update's helper; runs in play). A wave index word (file value -1) is set by thread S to 6, 7 or 8 (the region index j it loaded). When the word is 0 or more: it reads the character range of the slot group that thread S loaded and tests every character in it; if any is still alive it does nothing this tick. When all are dead: sets flag 342 (index 6), 343 (index 7) or 344 (index 8); ActivateObj(object 10, bool 1), PlaySound(9), PlayAnim(10, no wait), DeactivateObj(4). Then, if flags 342, 343 and 344 are all set: fade out, PlayCamera(9, no wait, camera wait 1), fade in, PlaySound(9), PlayAnim(9, wait), fade out, ReturnToNormalCamera(-1, true), fade in. Finally it sets the wave index word back to -1, so each wave is scored once (High). These are the setters of flags 342, 343 and 344.
- Init (two branches, gated on blackboard value 20 for world 157). This pack is the arena: pack 1350 (world 157) posts blackboard value 20 (type 2, world 157) before it sends the player here (SPEC-0042, module 1350). Branch A is an arena round; branch B is a normal visit.
  - Common to both branches: GetBlackboardMessageValue(type 2, world 157), StopEffect(0, world), debug-mode checks. Init remembers whether the value was 20 for the start thread.
  - Unconditional at this level: RegisterTeleportCallback(index 0, teleport callback TP0); RegisterActivateWorldCallback(W); SlotAllocate(2 sets); ControlThreadsAllocate(2); ControlThreadStart(start thread T).
  - Branch A (value is 20): RegisterHeroDeathCallback(H, bool 1); RegisterHeroUseItemCallback(U, bool 1); InitObjOn(objects 10, 9 and 4, false); FadeScreenOut(0.1, wait 0).
  - Branch B (value is not 20): init calls H4, which makes the trigger registrations for indices 2, 3 and 4 (Z2), 1 (Z1) and 0 (Z0); this is the only registration of these triggers. Then the object poses, by flag: if flag 345 is clear, InitObjOn(object 8, false) (start pose); with flag 345 set, object 8 gets no call. If flag 100 is clear: object 1 is set to its end pose (InitObjOn(1, true)) when flag 342 is set and to its start pose (false) when it is clear; object 2 and object 3 get start poses (false) whatever flags 342 to 344 are. With flag 100 set, objects 1, 2 and 3 get no call. Object 10 gets its end pose (InitObjOn(10, true)) and object 9 its start pose (false) on every branch B load, whatever flags 100 and 345 are (High). Test vectors (High): flags 345 clear and 100 clear, 342 set: object 8 start, 1 end, 2 start, 3 start, 10 end, 9 start. Flag 345 clear, 100 clear, 342 clear: 8 start, 1 start, 2 start, 3 start, 10 end, 9 start. Flag 345 set, 100 clear, 342 clear: 8 no call, 1 start, 2 start, 3 start, 10 end, 9 start. Flags 345 and 100 both set: 8, 1, 2 and 3 no call, 10 end, 9 start. Branch A sets the arena-round word (file value 0) to 1, and branch B is taken when it is 0.
- Script trigger callbacks:
  - Index 0 (Z0; branch B only): when the record's +0x10 word is 1, the hero is inside and the exit word (file value 0, set by Z1 at its end) is set: AddBlackboardMessage(type 2, value 20, world 157), AddBlackboardMessage(type 1, value 1, world 157) and StartExteriorReplacementLoad(world 157, 0, no colour), which loads pack 1350 with a won result (High).
  - Index 1 (Z1): when the record's +0x10 word is 1, the hero is inside, flag 100 is clear, flags 342, 343 and 344 are set and flag 345 is clear: SetGameState(345, true), SetGameState(100, true), FadeScreenOut(1.0, wait 1), DeactivateObj(8), PlayFMV(61, wait 1), StartEffect(0, world, wait 1), FadeScreenIn(1.0, wait 1), with a debug line, and the exit word set to 1, which opens Z0 (High).
  - Indices 2, 3 and 4 (Z2, the wave-trial gates; High; summary in SPEC-0059, section 8): when the record's +0x10 word is 1, the hero is inside and flag 100 is clear, the gate of the record's trigger index (+0x14 word) is tested. The gates open the three waves in a fixed order:
    - Index 2: flags 342, 343 and 344 clear and object 1 not animating (IsObjAnimating): locator 23, gate object 1, thread S argument 1 (wave j 6, flag 342).
    - Index 4: flag 342 set, flags 343 and 344 clear, and object 1 not animating: locator 25, gate object 3, thread S argument 3 (wave j 7, flag 343).
    - Index 3: flags 342 and 343 set, flag 344 clear, and objects 3 and 2 not animating: locator 24, gate object 2, thread S argument 2 (wave j 8, flag 344).
    - A passing gate runs: WalkHeroAndOrient(its locator, wait 1, control 0), FadeScreenOut(1.0, wait), ActivateObj(10, 0), ActivateObj(9, 0), ActivateObj(4, 0), PlayCamera(8), FadeScreenIn(1.0, wait), PlaySound(10), PlayAnim(the gate object, wait), KillSound(10), FadeScreenOut(1.0, wait), ReturnToNormalCamera(-1), SlotReset(the stored slot), then starts thread S with the gate's argument. A failing gate does nothing. The flag of each wave is set by the wave controller when its characters are all dead.
- Hero-death callback H (branch A only), runs once for each hero death (see SPEC-0055). Record word 0 is the hero enum of the dead hero. The module keeps one "has died" mark per hero for the round:
  - The other hero has not died yet: mark this hero, SwitchHero(to the other hero, true) and SetHeroSwapMode(3).
  - The other hero has already died (both heroes down): LoopHeroAnim(34, 1, 100.0) and set the round-lost mark that the watcher thread reads.
- Hero use-item callback U: no host calls.
- Teleport callback TP0 (teleport index 0, branch A): if the button is pressed and flag 100 is set: AddBlackboardMessage(type 2, value 60, world 157), StartExteriorReplacementLoad(world 157, -1, no color), wait 3. If the button is pressed and flag 100 is clear: LevelMsg(line 2, wait 1). Nothing happens when the button is not pressed.
- Activate-world callback W: DebugPrintf lines, then reads blackboard value (type 2, world 163). If the value is 60: MoveHero(16, control 1), AddBlackboardMessage(type 2, value -1, world 163), SetToNormalCamera(15, 0, true).
- Start thread T (started once by init in both branches; regions follow the rule region = 1371 + 1 + j, SPEC-0006 "Which region a slot load names"):
  - Branch B (value not 20): SlotCreate(3 entries, j 6, 7 and 8) and stores the returned slot number as the stored slot. This reserves a buffer only (sized for regions 1378-1380); it loads nothing. Thread T then ends.
  - Branch A (value 20, an arena round): SlotCreate(5 entries, j 0 to 4) and stores the result as the round slot; SlotCreate(1 entry, j 5); SlotLoad(that second slot, j 5, keep true, activate true), which loads region 1377 (Sevaric) every round. Then the round index t is 4 if flag 599 is set, else 3 if 598, else 2 if 597, else 1 if 596, else 0; SlotLoad(the round slot, j t, keep true, activate true) loads region 1372 + t. So round 1 loads 1372 (Bandit), round 2 1373 (Dark Ixi Spearman; selected when flag 596 is set and 597 is clear), round 3 1374 (Giant Drackonack), round 4 1375 (Skeleton Fighter) and round 5 1376 (Werelupe Elite). Then it starts the watcher thread V, plays the round intro scene (below), WalkEntityAndOrient(entity 30, locator 15, 1, 0, 1.1) and LoopEntityAnim(entity 30, animation 55, 0, 0.0). Confidence: High.
- Arena round casts (one region per round; the record order gives the bearings 0, 45, 90, 180, 225 and 270 degrees on a ring of radius 26.4 about (0, 7.21, 0)):
  - Round 1 (region 1372): Bandit, Rogue, Thug, Bandit, Thug, Thief.
  - Round 2 (region 1373): Dark Ixi Spearman, Dark Ixi Spearman, Dark Ixi Archer, Dark Ixi Archer, Dark Ixi Spearman, Dark Ixi Archer (spearmen row 161, archers row 158).
  - Round 3 (region 1374): Giant Drackonack, Drackonack, Drackonack, Giant Drackonack, Drackonack, Drackonack.
  - Round 4 (region 1375): Skeleton Fighter, Skeleton Archer, Skeleton Wizard, Skeleton Fighter, Skeleton Archer, Skeleton Wizard.
  - Round 5 (region 1376): Werelupe Elite, Werelupe Basher, Werelupe Bone Thrower, Werelupe Bone Thrower, Werelupe Elite, Werelupe Basher.
  - Every record: default state 8, emerge distance 40.0, cue word 1, selector 0, no alarm group, no drop.
- Watcher thread V (arena rounds only): reads the character range of the round's region group (the group of j t) with GetAssetGroupCharacterRange. Each tick (Sleep(1) per pass) it checks the round-lost mark set by H, then whether every character in that range is dead (IsAI_Dead).
  - All dead (round won): Wait(1.0), the win scene H5, SetHeroSwapMode(2), AddBlackboardMessage(type 2, value 20, world 157), AddBlackboardMessage(type 1, value 1, world 157), StartExteriorReplacementLoad(world 157, 0, no colour).
  - Round lost (both heroes died): SetGameState(595, true) if flag 595 is clear, the loss scene, SetHeroSwapMode(2), AddBlackboardMessage(type 2, value 20, world 157), AddBlackboardMessage(type 1, value -1, world 157), StartExteriorReplacementLoad(world 157, 0, no colour).
  - Pack 1350 reads the type 1 value on return to decide whether the round was won (SPEC-0042, module 1350). Confidence: High for the order of calls.
- Thread S (started from the Z2 gates above, branch B; it loads into the slot that thread T reserved in branch B, then sets the wave index word to the loaded j, 6, 7 or 8, which the wave controller scores). For every argument the load is followed by FadeScreenIn(1.0) (Medium-High), which ends the fade-out of the gate sequence:
  - Argument 1: SlotLoad(the stored slot, j 6, keep true, activate true): region 1378.
  - Argument 3: SlotLoad(the stored slot, j 7, keep true, activate true): region 1379.
  - Argument 2: SlotLoad(the stored slot, j 8, keep true, activate true): region 1380.
- Helpers:
  - H1: a message chooser. Reads flags 599, 598, 597 and 596 and shows LevelMsg 47, 45, 44 or 43 depending on which flag is set first. Used by the round intro scene.
  - H2: with argument 1, LevelMsg(line 48). Otherwise the same chooser as H1 with lines 53, 51, 50 and 49. Used by the win and loss scenes.
  - H4: registers script triggers: indices 2, 3 and 4 with Z2; index 1 with Z1; index 0 with Z0. Init calls it in branch B only; the triggers are registered once.
  - H5 (the win scene, called by watcher V): GrabHeroControl, SetHeroEquippedState(false), FadeScreenOut(0.7, wait 1), GetAssetGroupCharacterRange, MoveHero(14, control 1), FadeScreenIn(0.7), PlayCamera(6, wait 0, camera wait 1, speed 1.0), PlayHeroAnim(182), LevelMsg(48, wait 1), and DeactivateEntity.
  - Round intro scene (called by thread T): takes hero and entity control, moves the hero and the round's entities (the hero's last move is to locator 14, the ring centre (0, 7.21, 0); the fighters are grabbed, moved and looped, then released), plays cameras with fades and waits, shows a round line chosen by flags (H1), and returns hero, entity and camera to normal. Step details: Medium.
  - Loss scene (called by watcher V): takes hero control, moves the hero and the living round entities, and plays camera, hero and entity animations with fades and lines chosen by flags. Step details: Medium.
- Dormant routines: one conversation with three to seven lines, and a copy of pack 1350's round-advance routine. No live routine references either.
- Threads: T (started by init), V (started by T in arena rounds) and S (started from Z2).
- Edge cases:
  - The branch in init depends on the blackboard value at load time, so the registered triggers differ between loads.
  - The first hero death in a round switches heroes, so later trigger and teleport calls act on the other hero; the round is lost only when both heroes have died.
  - Both the win and the loss path post value 20 again, so pack 1350 runs its round-result step after either outcome.
- Confidence: High for the init branches, threads T and V, the hero-death callback and the teleport callback. High for Z0, Z1, the wave controller and the Z2 gates. Medium for the step details of H5, the intro scene and the loss scene. That arena rounds occur in play rests on pack 1350 being reached: Medium-High.
- Open questions:
  - Meaning of blackboard value 60 and the roles of world 163 beyond this pack (meaning unknown). Resolved: value 20 (type 2, world 157) marks an arena round started from pack 1350, and the type 1 value for world 157 carries the result (1 won, -1 lost).
  - Resolved: init calls H4 in branch B (when the arena-round word is 0), once.
  - Locator ids 23, 24 and 25, and object ids 4, 8, 9 and 10 (meaning unknown).
  - Whether any caller starts the dormant conversation routine: none found.
  - Resolved: the SlotLoad calls without a long-lived slot word are the arena loads in thread T: the j 5 load passes the second SlotCreate return directly, and the j t load reads the round slot that thread T stored just before.


### 1382: Exterior, six element pedestals, floor switches and the final-fight gate

- Family: Exterior.
- Exit: present. Its body makes no host calls.
- Init (no game-state test anywhere in init or its helpers; every call below runs on each load, High):
  - Allocates 1 control thread, InitObjOn(object 2, false) and registers the activate-world callback.
- Activate-world callback (registered by Init): calls an empty helper. It reads no blackboard value, moves no hero, changes no flag and makes no camera or object call. The callback has no gameplay effect (High).
  - Pedestal set-up: for each of the six pedestal records, registers CB1 on its strike trigger and CB2 on its stand trigger, initialises the record's two objects (not forced), runs the put-out body (below) on the record, and finally sets the lit count word to 0.
  - Object set-up: InitObjOn for objects 15 to 20 (not forced).
  - InitObjOn for objects 10, 7, 4, 12, 6 and 13 (false) and registers CB5 on trigger indices 3, 4, 8, 6, 7 and 5.
  - Registers CB4 on trigger index 2.
  - Slot set-up: allocates 3 slots, creates slot entries 1, 1 and 6 (constant tables) and registers CB6 on trigger indices 9, 10, 14, 12, 13 and 11. SlotCreate only reserves buffers. The first single-entry slot (the fight slot) names set 7 (region 1390). The second single-entry reservation names j 0 (region 1383); its slot number is stored in a word that nothing reads, so region 1383 is never loaded (High). The six-entry slot serves CB6 (sets 1 to 6, regions 1384-1389).
  - InitObjOn(object 28, false); StopEffect(2), (3) and (1).
  - When the module is loaded, its six pedestal records start with the drop word set to 1 and the lit, locked and armed words at 0.
- Pedestal records (High). Each record names a strike trigger, a stand trigger, two set-up objects, two lit objects, five effects, a drop locator, the fight slot and two character indices of that slot, and the two Mote ids it accepts. The two Motes are the lesser and the greater Mote of one element (SPEC-0021, Motes and elements):

  | Record | Strike trigger | Stand trigger | Accepted Motes | Element | Drop locator | Set-up objects | Lit objects | Effects | Characters |
  | ---: | ---: | ---: | --- | --- | ---: | --- | --- | --- | --- |
  | 0 | 19 | 25 | 168, 169 | Smoke or Shadow | 5 | 47, 57 | 34, 35 | 4, 15, 21, 22, 33 | 0, 4 |
  | 1 | 21 | 27 | 160, 161 | Fire | 6 | 48, 56 | 36, 37 | 6, 13, 19, 23, 31 | 1, 8 |
  | 2 | 20 | 26 | 166, 167 | Fog or Wind | 7 | 51, 53 | 38, 39 | 9, 10, 16, 24, 28 | 5, 9 |
  | 3 | 16 | 22 | 170, 171 | Light | 8 | 46, 52 | 40, 41 | 5, 14, 20, 25, 32 | 2, 6 |
  | 4 | 18 | 24 | 162, 163 | Bubble or Water | 9 | 49, 55 | 42, 43 | 7, 12, 18, 26, 30 | 3, 10 |
  | 5 | 17 | 23 | 164, 165 | Leaf or Rock | 10 | 50, 54 | 44, 45 | 8, 11, 17, 27, 29 | 7, 11 |

- Pedestal volumes (High). The strike triggers 16 to 21 are boxes 3 x 3 x 0.6 (volume flags 0x0020: only the moving-rectangle bit, so the cylinder bit 3 is clear). Their third-extent word (+0x2C) is 0.6, above zero, so the moving-rectangle fire rule applies (bit 5, SPEC-0028 section 13) while the box does not move. The stand triggers 22 to 27 are cylinders of radius 3 (volume flag bit 3 set) at the same centres. The six centres lie on a ring of radius about 54 around (-4.8, -438, 298.6): record 0 at (-4.9, 353.0), 1 at (-52.0, 271.5), 2 at (-52.0, 325.8), 3 at (-4.9, 244.3), 4 at (42.2, 325.8) and 5 at (42.2, 271.4) (x, z). The drop locators 5 to 10 form an inner ring of radius about 18, each between the centre and its pedestal. Trigger 28 (cylinder radius 2.75) is at the ring centre.
- Pedestal triggers:
  - CB2 (stand trigger): finds its record by trigger index. When the trigger state is 1 or 2, the record's armed word becomes 1 if the callback's object word is non-zero (the hero's own trigger-hit path) and 0 if it is zero (a hero shot, a character or any other firer). In any other state the armed word becomes 0. Both hero paths fire these cylinders, so the value at the time of a strike depends on which fire ran last (Medium).
  - CB1 (strike trigger): finds its record by trigger index. It goes on only when the trigger state is 1 or 2, the callback's attack word (+0x24) or shot word (+0x20) is non-zero, and the record is armed. These boxes are fired by the hero's body contact (moving-rectangle rule) and by hero shots (the hero shot pass tests the same bit 5 for a moving box; SPEC-0028), so a shot that hits the box passes this test with the shot word at 1 and the attack word at 0 (High for the rule; Medium that a shot reaches the box in play). The hero's body-contact fire writes the attack word on every tick on which the selected hero's body overlaps the box: object 1 and the hero's attack-active value that tick (SPEC-0036, Attack window), so the word is 1 only on attack-window ticks. A weapon strike without the hero's body never fires a script trigger (SPEC-0030, section 3); a hero shot does. The callback body runs once per callback thread life: it starts on a fire when no earlier run is still alive, so it is tested on each overlapping tick that finds no live run, not only on the first window tick. A tick outside the window returns at once (attack word 0). Once a test passes, the wait of 0.5 s and the lit body keep that run alive for about 0.75 s, during which no new run starts (High for the rules; Medium for the thread-life timing). Test vector (synthetic; 4 ticks per frame): the hero's body overlaps box 16 from tick 200 to tick 260 and its attack window is at ticks 240 to 247. Ticks 200 to 239 return without a pass; tick 240 passes the attack test, so the wait and lit body start there.
  - Element test (High): it reads GetHeroAttackAlignment, which returns the hero's equipment word 2, the weapon Mote id (-1 when no Mote is equipped; SPEC-0021, Equipment slots). The test passes when that id equals either accepted Mote of the record. So an element is delivered by striking with a Mote of that element equipped in the weapon slot; the lesser and the greater Mote both count, and no item use is involved. A hero shot into the strike box from the stand cylinder passes the same test (Medium).
  - Passed: a wait of 0.5 s, then the lit body. Failed: the drop body.
- Lit body (High): when the record's locked word is zero: if its lit word was 0, the lit count rises by 1; the lit word becomes 1; PlaySound(7, at the origin); StartEffect(first effect); a wait of 0.25 s; ActivateObj(first lit object, not forced); StartEffect for the second, third and fourth effects; ActivateObj(second lit object); StartEffect(fifth effect); CharacterActivate for the record's two characters in the fight slot. Striking a lit pedestal again replays the calls but does not change the count.
- Put-out body (High): when the locked word is zero: if the lit word was 1, the lit count falls by 1; the lit word becomes 0; stops the five effects and deactivates the two lit objects. Only init and the completion helper run it. No trigger, thread or timer puts a lit pedestal out, so a lit pedestal stays lit until the pack is loaded again (init then puts all six out and sets the count to 0).
- Drop body (High): when the record's drop word is non-zero, adds the first accepted Mote (the lesser one) at the record's drop locator (AddItemToWorld, locator form: snap true, placed true, lifetime -1.0) and clears the drop word. So a wrong strike, or a strike with no weapon Mote, drops that pedestal's lesser Mote once per load.
- CB5 (floor switches, triggers 3 to 8; a switch on the trigger index, High): runs when the callback's object word is non-zero and the trigger state is 1. Each index has an object pair: 3 (10, 9), 4 (7, 8), 5 (13, 14), 6 (12, 11), 7 (6, 5) and 8 (4, 3). It plays sound 5, deactivates the first object of the pair and activates the second. If the index's once-word is clear and the reveal counter is not -1, it sets the once-word and runs the reveal step: a wait of 0.5 s, PlaySound(6), PlayAnim(15 + counter, wait), PlayCamera(4, 1.0 s, wait, camera wait 2), KillSound(6), ActivateObj(21 + counter); the counter (file value 0) then rises by 1, and after 5 it becomes -1. Then it waits until the hero leaves the trigger, plays sound 4 and swaps the pair back. When the once-words of all six indices are set, it sets the CB4 arming word. The markers rise in the order the switches are first stepped on, not in index order.
- CB6 (triggers 9 to 14; High): when the callback's object word is non-zero and set (index - 8) is not the last-loaded set (file value -1): resets the six-entry slot, loads that set into it (keep true, activate true) and stores it as the last-loaded set. So trigger 9 loads set 1, and trigger 14 loads set 6.
- CB4 (trigger 2, a vertical cylinder of length 3.0 and radius 90.0; its base is about (0, -220.464, -72.762), axis up): when the callback object is non-zero, its done word is clear and the arming word is set: marks the event done; initialises objects 0 and 1, deactivates object 2, waits 2.0 s, plays camera 5 looking at the player, activates object 27 at the end of its animation, deactivates objects 21 to 26, plays sound 6, then plays objects 15 to 20 forward and object 27 at speed -0.7 (waits for object 27). A negative PlayAnim speed starts at the end pose and runs backward, so object 27 descends from its end position toward its start. Object 27's attached two-by-one ground grid samples around y=-410.4 at the base pose; its vertical animation track raises the grid by about 189.99 units, placing the end pose near y=-220.4, within 0.4 units of the trigger's vertical band. A hero grounded on this moving platform can be carried when the same moving object remains the ground owner across ticks and the vertical-carry flag is enabled (SPEC-0018); the callback does not force initial contact or set that flag. The script does not directly move the hero. The lower terrain sampled between the platform base and pedestal ring continues around y=-410 and lower, but this does not prove an uninterrupted walkable route. It then deactivates object 1, kills sound 6 and starts CB3.
- CB3 (control thread, started by CB4): loads set 7 into the fight slot with keep false (activate true), then ends (High). Before CB3 runs, the fight slot is empty, so a pedestal lit earlier activates no character.
- Update (one per tick): when module word +0x1BCC (file value 1, never written) is non-zero, the lit count is 6 and the done word is 0, it runs the completion helper once:
  - Sets the done word; FadeScreenOut (black, 1.0 s, no wait); a 1.5 s wait; locks all six pedestals (put-out body, then the locked word set to 1); DeactivateEntity for entities 32 to 40, 43, 48 and 49; resets the fight slot; SetGameState(98, true) and SetGameState(337, true); PlayFMV(70, wait); DeactivateObj(28); StartEffect(1); registers the exit routine on trigger index 28. These are the setters of flags 98 and 337 (High).
  - The exit routine (trigger 28): when the callback's object word is non-zero and its once-word (file value 0) is clear: sets the once-word, fades out (0.5 s) and starts an exterior replacement load for world 178 (second argument 0), which loads pack 1411 (High).
- Dormant routines (no live reference, High): an enemy respawn loop (it would activate characters 0 to 3, set the hit points of entities 32 to 35 to 0, and every 2 s, while fewer than three pedestals are lit and at most one of those entities is alive, activate the next of characters 0 to 3 and its entity), the entity activation body that only this loop calls, a one-time look-at-player camera, and a hero animation and move routine. None of them runs.
- Threads: CB3, started by CB4. No other thread starts.
- Edge cases and error handling:
  - The pedestals can be lit in any order and there is no timer. "Lit at the same time" holds because nothing puts a lit pedestal out; leaving the pack before the sixth is lit loses all of them.
  - The completion helper runs once per module load (done word); flags 98 and 337 stay set.
  - A pedestal accepts only its own element. Mote 172 or 173 (Nova) and Motes of other elements fail.
- Test vectors (High):
  1. Weapon Mote 160 (Fire Mote) equipped, standing in trigger 27, striking box 21 (record 1): after 0.5 s the pedestal lights; lit count 0 to 1.
  2. Same pedestal with Mote 161 (Lava Mote): it lights as well.
  3. Same pedestal with Mote 162 equipped: nothing lights; Mote 160 appears at locator 6. A second wrong strike drops nothing.
  4. Striking the lit record 1 again with Mote 160: the count stays 1.
  5. Records 0 to 5 lit with Motes 168, 160, 166, 170, 162 and 164 in any order: on the next tick flags 98 and 337 are set and movie 70 plays once; trigger 28 then loads pack 1411.
  6. Five lit, then the pack is left and entered again: the count is 0 and all six are out.
- Confidence: High.
  - High for init's registrations, the record table, the element test, the lit, put-out and drop bodies, the lit count logic, the completion helper, the exit to world 178, CB4, CB5, CB6 and CB3, and for the exit and hooks having no host calls.
  - Medium for the arming outcome when both hero paths fire a stand cylinder in the same tick.
- Open questions:
  - Resolved: init has no game-state test; every registration runs on each load.
  - Resolved: each pedestal accepts the lesser and greater Mote of one element, tested against the weapon Mote word (record table above).
  - Resolved: no reached code puts a lit pedestal out; only a reload does.
  - Meaning of the object indices read from module data (ids not yet mapped), and whether the lower ring can be reached before trigger 2 (geometry).

### 1392: Exterior, many scripted triggers, button prompts and a per-tick hero fall

- Family: Exterior (its Init entry).
- Exit: present, empty. The exit export runs no host calls.
- Load and unload hooks: present, no host calls.
- Callbacks: 36 trigger registrations. Teleport callbacks: none. Threads: none. Dormant routines: none (every routine is reached from init, update, a trigger or a helper).
- Init (in order):
  1. RegisterScriptTrigger for every trigger index, by index (see the table in the Triggers part below).
  2. InitObjOn(object, false) for objects 10, 12, 14, 1, 2, 3, 4, 5, 6, 7, 8, 9, 29, 28, 27, 26, 30, 31, 33, 34, 32.
  3. QueryGameState(696) (High).
  4. Flag 696 set: the three gate levers count as pulled: InitObjOn(11, false), InitObjOn(13, false), InitObjOn(15, false) (the gate objects at their start pose) and InitObjOn(20, true), InitObjOn(22, true), InitObjOn(24, true) (levers 20, 22 and 24 at their end pose). Flag 696 clear: InitObjOn(20, false), InitObjOn(22, false), InitObjOn(24, false); StopEffect(13, 14, 15, 16, 17, 18) on the module world.
  5. In both cases (unconditional at this level): AddAnimObjSound for objects 20, 22 and 24 (sound 11, speed 0.5, volume 1.0), and for objects 29, 28, 27, 26, 30, 31, 33, 34 and 32 (sound 12, speed 0.3, volume 1.0).
- Update (every tick; High): the fall-in. Once per module load (a module latch word), while state 696 is clear, it runs one block:
  - MoveHero(locator 0, bool 0, CONTROL_OPTION 1, unk 1).
  - StartHeroLongFall.
  - PlayTrackHeroCamera(camera 10, WAIT 1, CAMERA_WAIT 1, 1.0).
  - Wait(3.0).
  - ReturnToNormalCamera(-1, bool 1).
  - When state 696 is set, or after the block has run once, Update does nothing.
- Triggers (High). Record fields: the object word at +0x1c (non-zero when present), the state word at +0x10 (1 entered; SPEC-0030 section 3) and the trigger index.
  - Index 0: if +0x1c is non-zero and +0x10 == 1, SetGameState(696, bool 1). This is the only writer of 696; no other pack reads 696.
  - Three gates (gate object, anim object, camera, sound id): gate A (11, 10, 5, 1), gate B (13, 12, 7, 2), gate C (15, 14, 9, 3). Opening a gate (shared helpers A, B and C below with first argument true): the gate object is deactivated, the camera plays, the anim object plays forward with loop sound 5, Wait(1.0 s), the camera returns, and the gate counts as open. Closing it (first argument false): the gate object is deactivated first (the same first call as opening), the anim object plays backward with loop sound 6, and the gate counts as closed. The open triggers (1 to 5, 7 to 12 and 14 to 22) test only the trigger record (object word set, state 1) and, for triggers 1, 7 and 14, the lever flag; they do not test whether the gate is already open. So a second open trigger crossed while the gate is open replays the whole opening sequence and sets the open state again. The close triggers (6, 13 and 23) run only when the gate is open, and do nothing when it is closed (High).
  - Gate A: trigger 1 opens it while lever 25 is not pulled; triggers 2 to 5 open it; trigger 6 closes it if it is open and then shows the gate object again when lever 25 was pulled.
  - Gate B: trigger 7 opens it while lever 26 is not pulled; triggers 8 to 12 open it; trigger 13 closes it as above.
  - Gate C: trigger 14 opens it while lever 27 is not pulled; triggers 15 to 22 open it; trigger 23 closes it as above.
  - Levers 25, 26 and 27 (each once per load, prompt context 4): each pulls gate A, B or C open for good (cameras 4, 6 and 8; effects 13 and 14, 15 and 16, 17 and 18; the gate object shown). The calls are:
  - Index 25 (own sequence, gate +0x1c != 0 via a button check): DisplaySquareButtonContext (message 4, text from module data), IsButtonPressed(button 0, bool 1). If pressed: MoveHero(16, bool 1, CONTROL 0, unk 1); PlayHeroAnim(140, WAIT 0, CONTROL 1); PlayAnim(20, WAIT 0, 1.0); PlayCamera(4, WAIT 1, CAMERA_WAIT 1, 1.0); PlaySound(7, no position, bool 0); StartEffect(13, bool 1); StartEffect(14, bool 1); Sleep(20 frames); ActivateObj(11, bool 0); Wait(2.0); ReturnToNormalCamera(-1, bool 1).
  - Index 26 (own sequence, same button shape): MoveHero(17), PlayAnim(22), PlayCamera(6), PlaySound(7), StartEffect(15), StartEffect(16), Sleep(20), ActivateObj(13, bool 0), Wait(2.0), ReturnToNormalCamera(-1). Uses PlayHeroAnim(140).
  - Index 27 (own sequence, same button shape): MoveHero(18), PlayHeroAnim(140), PlayAnim(24), PlayCamera(8, WAIT 1), PlaySound(7), StartEffect(17), StartEffect(18), Sleep(20), ActivateObj(15, bool 0), Wait(2.0), ReturnToNormalCamera(-1).
  - Indices 28 to 36 (nine helper levers, each once per load): each routine calls the shared button helper D with its own record (hero locator, camera, lever object, object, AI): trigger 31 (22, 20, 29, 1, 14); 30 (21, 18, 28, 2, 13); 29 (20, 15, 27, 3, 12); 28 (19, 14, 26, 4, 11); 32 (23, 21, 30, 5, 15); 33 (24, 23, 31, 6, none); 35 (26, 27, 33, 7, none); 36 (27, 30, 34, 8, none); 34 (25, 32, 32, 9, none). When an AI is named it gets SetAI_Behavior behaviour 8 with (0, 10.0, 1, 0).
- Shared helper A (gate A; used by indices 1 to 6): when the first argument is true (open): DeactivateObj(11); PlayCamera(5, WAIT 0, CAMERA_WAIT 1, 1.0); LoopSound(5, id 1, bool 0); PlayAnim(10, WAIT 1, forward); KillSound(5, id 1); Wait(1.0); ReturnToNormalCamera(11, bool 1). When the first argument is false (close): DeactivateObj(11) runs first, as on opening; then the anim plays backward with LoopSound(6, id 1, bool 0) and KillSound(6, id 1). The helper itself does not test the open state; the close triggers do, and the open triggers do not (see the gates bullet).
- Shared helper B (used by index 7 and indices 8 to 13): same shape as helper A, with DeactivateObj(13), PlayCamera(7), LoopSound(5, id 2), PlayAnim(12, WAIT 1), KillSound(5, id 2), Wait(1.0), ReturnToNormalCamera(12, bool 1); a first argument of false also starts with DeactivateObj(13) and then uses LoopSound(6, id 2) and KillSound(6, id 2).
- Shared helper C (used by indices 14 to 23): same shape again, with DeactivateObj(15), PlayCamera(9), LoopSound(5, id 3), PlayAnim(14, WAIT 1), KillSound(5, id 3), Wait(1.0), ReturnToNormalCamera(13, bool 1); a first argument of false also starts with DeactivateObj(15) and then uses LoopSound(6, id 3) and KillSound(6, id 3).
- Shared button helper D (used by indices 28 to 36). When the trigger record's gate holds (+0x1c == 1) and the record's first field is zero: DisplaySquareButtonContext (message 4, text from module data), then IsButtonPressed(button 0, bool 1). If pressed (and the record's extra predicates hold):
  - MoveHero(locator from record +0x4, bool 1, CONTROL 0, unk 1).
  - PlayCamera(camera from record +0x8, WAIT 0, CAMERA_WAIT 1, 1.0); Wait(1.0) in part of the variants.
  - PlayHeroAnim(141, WAIT 0, CONTROL 1); Wait(0.5) in part of the variants.
  - PlayAnim(animation from record +0xc, WAIT 0, 1.0); PlaySound(13, no position, bool 0); PlayAnim(animation from record +0x10, WAIT 1, 1.0); KillSound(13).
  - SetAI_Behavior(value from record +0x18, behaviour 8) when that value is at least 0 (the extra predicate).
  - ReturnToNormalCamera(-1, bool 1).
  - Helper D's record is the five values listed under indices 28 to 36 (High).
- Test vectors:
  - Trigger 0 fires with +0x1c != 0 and +0x10 == 1: state 696 becomes 1.
  - First Update with state 696 clear: hero moved to locator 0, long fall, track camera 10, Wait(3.0 s), camera returned; later Updates in the same load do nothing.
  - Trigger 31 fires with the hero inside and the button pressed: hero to locator 22, camera 20, lever object 29 and object 1 play, AI 14 gets behaviour 8.
  - Trigger 1 entered with lever 25 not pulled: gate A opens (object 11 off, camera 5, object 10 forward). Trigger 6 then closes it. Trigger 2 opens gate A (the gate is open afterwards); crossing trigger 3 while it is open replays the opening sequence in full, and the state stays open. Trigger 6 then closes it, and a further trigger 6 with the gate closed does nothing (High).
- Edge cases: the fall-in runs once per module load while 696 is clear, so re-entering the pack before trigger 0 replays it. Helper D needs the button to be pressed, so nothing runs if the press is missed.
- Confidence: High for Init, the update, the gates, the levers and the helper lever records. The call order inside the shared helpers is High; the speed of the gate animations is Medium.
- Open questions:
  - Resolved: the update block runs once per module load (a latch word).
  - Resolved: +0x10 is the state word; the first argument of helpers A to C is open (true) or close (false).
  - Resolved: the Init test is flag 696.
  - Whether the gates and levers block progress or only open shortcuts depends on the area geometry (open).
  - Object ids (1 to 34), movable-block ids, camera ids (4 to 10), locator ids (0, 16, 17, 18) and sound ids (5, 6, 7, 11, 12, 13) are from level data, not yet mapped.
  - Text message ids for the button prompts (from module data, id not yet mapped).


### 1394: Exterior, timer and button hero-swap triggers with a hero-death gate

- Family: Exterior.
- Exit: present, no host calls.
- Module words (all file value 0, so all clear at the start of each module lifetime): the exit-open word, the room word, the short-intro word, the final word, the busy word, one once-word per checkpoint lever, and the checkpoint (respawn locator and camera; locator 0 and camera 2 until the first lever is used, Medium for these start values).
- Init (High):
  - A dead hero is revived first: for each hero that IsHeroDead reports dead, NeedHeroAlive(hero), which revives the partner only when the swap mode is exactly 2 and does nothing in any other mode (SPEC-0030, row 178).
  - InitObjOn on objects 3, 4, 5, 6, 10, 12, 14, 16, 1, 7 and 8 (false for 3 to 6, 10 to 16 and 1; true for 7 and 8), AddAnimObjSound(object 3, sound 13, 0.0, 1.0) and AddAnimObjSound(object 4, sound 13, 0.0, 1.0), RegisterMovableBlockSoundIndex(11), RegisterHeroDeathCallback(hero-death H, false) (false: the default death handling runs as well, SPEC-0055), and the trigger registrations below.
  - Trigger registrations: indices 0 to 10 (kill zones, S0), index 11 (S11), index 12 (S12), index 13 (S13), index 14 (S14), index 15 (S1), index 16 (S3), index 17 (S4), and index 18 (S2).
  - QueryGameState(flag 0). No module sets flag 0, so in play it is clear and Init takes the clear side: the room word, the short-intro word and the final word stay clear, InitObjOn(object 9, false), StopEffect(3) and SetHeroSwapMode(3). (The set side, never taken, sets the exit-open, room and short-intro words and uses swap mode 2.) Nothing in the module depends on flag 0 in play; in particular the exit back to pack 1353 does not.
- The 50 second countdown (High): the module uses a 50 s on-screen countdown (ActivateTimer(50.0, colour 0), counting down; SPEC-0023, section 31). While it runs, object 2 (the sinking platform) has vertical offset (50 - t) x -0.46, where t is the time left, so it is 23 units down at t = 0. The timer colour changes at 10 s left and flashes from 5 s left. At 0, from Update: control grabbed, timer stopped, PlaySound(5); then, unless a hero death is being handled, a scene: object 2 deactivated, object 0 shown, PlayCamera(11), object 0 played, camera returned; then a fade, MoveHero(checkpoint locator), ReturnToNormalCamera(checkpoint camera), FadeScreenIn(2.0), the countdown re-armed at 50 s, control restored.
- Checkpoint levers S1 to S4 (indices 15, 18, 16, 17; High): each works once per module load (its once-word) and only while the busy word is clear; prompt context 4 while the button is not pressed. When pressed: PlaySound(5), SetTimerColor, DeactivateTimer, the hero move, PlayHeroAnim(140, control 1), the lever swap (DeactivateObj, PlayAnim, ActivateObj), Wait, PlaySound(6), SetAnimObjPositionOffset(2), GetAnimObjWorldPos(2), Sleep, KillSound(6), FadeScreenOut(0.5), SetHeroSwapMode(2), SwitchHero, SetHeroSwapMode(3), MoveHero(the new checkpoint locator), ReturnToNormalCamera(the new checkpoint camera), FadeScreenIn(1.0), ResetAnimObjOffsets(2), ActivateTimer(50.0). Index 15 plays camera 11 (through a helper), 16 camera 4, 18 camera 5, and 17 cameras 12, 7 and 6 with a second swap-mode step (5, then 2).
  - Per lever (hero move; objects off, played, on; SwitchHero; new checkpoint locator and camera):
    - Index 15 (S1): MoveHero(15); 10, 10, 11; hero 1; checkpoint (1, 3).
    - Index 16 (S3): MoveHero(16); 12, 12, 13; hero 0; checkpoint (15, 4).
    - Index 18 (S2): MoveHero(18); 16, 16, 17; hero 1; checkpoint (16, 5).
    - Index 17 (S4): MoveHero(17); 14, 14, 15; hero 0; checkpoint (8, 4), and the final word is set.
- Kill zones S0 (indices 0 to 10) and hero-death callback H (High): when record +0x10 is 1 and the hero is inside (S0), or on a hero death (H, once per death):
  - Final word set: a fade, MoveHero(8), a fade; no timer.
  - Final word clear: PlaySound(5), SetTimerColor, DeactivateTimer, (S0: Sleep), FadeScreenOut(0.8, wait 1), ResetAnimObjOffsets(2), MoveHero(checkpoint locator, control 1), ReturnToNormalCamera(checkpoint camera), Wait(1.0), FadeScreenIn(1.0, wait 1), and the countdown re-armed at 50 s, except after S14 has stopped it for good.
  - The death callback H takes the same two branches (final word set: fade out 1.0 s with wait 1, MoveHero(8, control 1), Wait(1.0), FadeScreenIn(1.0, wait 1); no timer and no camera return). Control option 1 of MoveHero writes hero state 1 to the selected hero, so the move takes the dying hero out of the death state about 48 ticks after the callback starts (60 ticks in the final-word branch), before the death clip reaches the 0.95 point at about tick 63 where the game-over choice would be made. No game-over page follows a death in this area, and the hero stands at the checkpoint in state 1 with the health the killing hit left (0 after a lethal hit). Timing and the test vector are in SPEC-0055 (module 1394 callback, open question 41). High for the state write; Medium-High for the margin of about 3 ticks in the final-word branch.
- Trigger S14 (index 14): when record +0x1c is non-zero and the exit-open word is 0: the exit-open word and the final word are set, DeactivateTimer (the countdown is stopped for good), SetHeroSwapMode(2), and Sleep(10) in two places.
- Trigger S11 (index 11, the shrine room; High): when the room word is 0, record +0x1c is non-zero and record +0x10 is 1: SetHeroSwapMode(2), DeactivateTimer, SetGameState(104, true), SetGameState(353, true), GrabHeroControl, FadeScreenOut(0.5), MoveHero(9, control 1), PlayFMV(65, wait 1), Sleep, DeactivateObj(9), StartEffect(3), FadeScreenIn(1.0), RestorePlayerControl(-1); then the room word is set.
- Trigger S12 (index 12, the exit; High): when the room word is set, record +0x1c is non-zero and record +0x10 equals the room word (1): StartExteriorReplacementLoad(world 158, -1, no colour), which loads pack 1353. So the exit opens by reaching the shrine room (S11) in the same visit.
- Trigger S13 (index 13, the intro): when record +0x1c is non-zero and record +0x10 is 1. With the short-intro word clear (always in play): FadeScreenOut(0.5), GrabHeroControl, several Wait calls, PlayCamera(9) and (10), LevelMsg(line 2), PlayCamera(12), LevelMsg(lines 3 and 4), PlayAnim(7, 8 and 2), DeactivateObj(1, 7 and 8), ActivateObj(2). Then: SetHeroSwapMode(2), SwitchHero(hero 0), SetHeroSwapMode(3), LevelMsg(line 5), MoveHero(0, control 1), FadeScreenIn(0.3), ReturnToNormalCamera(2), ResetAnimObjOffsets(2), and ActivateTimer(50.0) (High for the calls; Medium for the order inside the long part).
- Update (every tick): the countdown step above (object 2's offset, the timer colour, the run-out at 0) while the exit-open word is 0 and the countdown is armed; it also sets and clears the busy word that the levers test (Medium for the exact order).
- Threads: none.
- Helper routines (Medium for which caller uses which): H1 (ResetAnimObjOffsets(2), ActivateTimer(50.0)) is the re-arm; H2 (PlaySound(5), SetTimerColor, DeactivateTimer) is the stop; H3 is the respawn sequence of S0 and H; H4 (the stop, PlayCamera(11), Wait, PlaySound(6), SetAnimObjPositionOffset(2), GetAnimObjWorldPos(2), Sleep, KillSound(6)) is the camera-11 part of lever S1.
- Dormant routines: one routine with no host calls; whether any routine references it is not established.
- Test vectors:
  - Countdown at t = 25 s left: object 2's offset is -11.5. At t = 0 with no death in progress: the run-out scene, then the hero at the checkpoint and the countdown back at 50 s.
  - Lever S3 (index 16) pressed: hero switched to hero 0, checkpoint (15, 4); a later kill-zone touch moves the hero to locator 15 with camera 4. Pressing S3 again in the same load does nothing.
  - Death callback H before the final lever, hero Tor dies with the checkpoint (15, 4): about 48 ticks after the callback starts the hero is at locator 15 in state 1 with 0 HP, and the death clip never reaches the 0.95 point in state 16, so no game-over page follows; the fade-in ends about tick 168 and the countdown restarts at 50 s. After the final lever (locator 8) the move happens about tick 60 and the whole sequence ends about tick 180, with no countdown.
  - S11 then S12: flags 104 and 353 set, movie 65, then the load of world 158.
- Edge cases:
  - Quirk: the room word is clear at the start of every module lifetime, so on a return visit S11 plays movie 65 again and sets flags 104 and 353 again before S12 works. Reproduce it.
  - After S14 the countdown never runs again in that load; deaths and kill zones then use the final-word branch (locator 8).
- Confidence: High for Init, the countdown, the checkpoints, the kill zones and death callback, S11, S12 and S14; Medium for S13's order and for the Update's order of steps.
- Open questions:
  - Meaning of flags 104 and 353 (meaning unknown beyond their use).
  - Resolved: flag 0 is never set, and nothing in play depends on it; the exit to pack 1353 opens through S11's room word.
  - Resolved: the levers' once-words are set at the end of each lever sequence (once per load); the helpers are called from the update, the levers, S0 and H as listed.
  - Resolved: SetAnimObjPositionOffset stores X/Y/Z components; Y is vertical (SPEC-0030 row 16; confirmed by module 1394 offset calls).


### 1409: Interior, proximity conversations and per-tick entity wander

- Family: Interior (its Init entry).
- Exit: present, empty. The exit export runs no host calls.
- Load and unload hooks: present, no host calls.
- Init (in order):
  1. AddBlackboardMessage(type 2, value 30, world 160).
  2. QueryGameState(106) into a local.
  3. If state 106 is set:
     - Register proximity callback index 0 (callback 1) and index 1 (callback 2), both with the bool flag 1.
     - Run an init helper that calls GetNumberOfFramesPerSecond, Random, and LoopEntityAnim(entity from caller, animation from caller, speed 30.0, CONTROL_OPTION 1). Many branch variants of this helper are chained under the 106 gate; most of their predicates are unknown ("? != 0", "? == 0").
  4. Unconditional at this level (both states): SlotAllocate(world, 1); SlotCreate(world, 1, a pointer to a module data cell); ControlThreadsAllocate(world, 1); ControlThreadStart(world, thread body below, world).
- Callback 1 (proximity, index 0, entity from the proximity record at +0x0):
  - If state 102 is clear: nothing.
  - If state 102 is set:
    - SetConversationTarget(entity, bool 1).
    - If state 104 is set: LevelMsg(message 0, world null, bool 1, WAIT_OPTION 1).
    - If state 104 is clear: LevelMsg(message 3, world null, bool 1, WAIT_OPTION 1).
    - EndConversation(entity).
- Callback 2 (proximity, index 1, same entity rule):
  - Same shape as callback 1, with LevelMsg message 10 when state 104 is set and message 13 when it is clear.
- Thread body (started from init in both cases):
  - Reads state 106.
  - If state 106 is set, calls SlotLoad(world, the stored slot, 0, keep true, activate true).
  - Always calls SetToNormalCamera(15.0, 0.0, bool 1).
  - No wait or loop calls appear in the thread body. Whether it runs once and ends is inferred from that, not confirmed.
- Update (every tick): one unconditional call into the shared entity-control routine (SPEC-0043, section "Shared entity-control routine", which holds its record layout and per-tick rules; High). The init helper of step 3 is its add step, so the records exist only when state 106 is set: entities 0 and 1, each idle animation 0, interval 8 to 15 s, dwell 5 to 7 s, animations 40, 95 and 96 (2.5 s) and 41 (1.3 s), no waypoints. The walks are walks to a locator; no module walks along a path.
- Dormant routines exist with no references from init, update, callbacks, thread or exit.
  - One dormant routine calls a helper that is itself not reached from anywhere else.
  - The other dormant routine has no outgoing or incoming references. Neither is described as running.
- Test vectors:
  - Init with state 106 set: proximity callbacks 1 and 2 are registered; the slot and thread calls also run.
  - Init with state 106 clear: no proximity registration; the slot and thread calls run.
  - Proximity with state 102 set and 104 set: LevelMsg 0 (and 10 for callback 2).
  - Proximity with state 102 set and 104 clear: LevelMsg 3 (and 13 for callback 2).
  - Proximity with state 102 clear: no action.
- Edge cases: the proximity callbacks do nothing when state 102 is clear, even though they are always registered.
- Confidence: High. Callbacks and init are High (full call list); the per-tick helper is the shared entity-control routine of SPEC-0043 (High).
- Open questions:
  - Resolved: states 102, 104 and 106 are the Altador legend flags (see the gate note below).
  - Resolved: the per-slot values are the record fields of the shared entity-control routine (SPEC-0043), and the records are entities 0 and 1 (see Update).
  - Whether the thread body ends after one run (not confirmed).
  - Resolved (High for the setters; Medium-High for the gate map): the gates are the Altador legend flags, which turn on in play as each shrine is finished: 100 (Gladiator legend, Coliseum, movie 61), 102 (Treasurer legend, Treasury, movie 63), 104 (Rogue legend, Rogue shrine, movie 65) and 106 (Petpet-Handler legend, Petpet shrine, movie 67). Start stage 7 sets all four (SPEC-0049). Flag 106 gates the two proximity registrations and the idle records of this module, so before the Petpet-Handler legend it registers nothing and its Update has no records to run; flags 102 and 104 then choose the messages (the callbacks test 102 first, then 104).
  - The activate argument of that SlotLoad is true on every known path.


### 1414: Exterior, proximity conversations, teleport loads and a world-activation hero move

- Family: Exterior (its Init entry).
- Exit: present, empty. The exit export runs no host calls.
- Load and unload hooks: present, no host calls.
- Callbacks: four proximity-trigger callbacks, two teleport callbacks, one world-activation callback, one thread body. Dormant routines: see below.
- Init (in order):
  1. RegisterActivateWorldCallback(world-activation callback, world).
  2. RegisterTeleportCallback(teleport callback A, index 0, CONTEXT_MESSAGE 13, no world).
  3. RegisterTeleportCallback(teleport callback B, index 1, CONTEXT_MESSAGE 13, no world).
  4. QueryGameState(102) into a local.
  5. If state 102 is set:
     - A helper registers four proximity triggers: proximity callback 1 at index 6, callback 2 at index 9, callback 3 at index 8, callback 4 at index 7; all with bool 1.
     - A helper runs the init-time animation chain: GetNumberOfFramesPerSecond, Random, and LoopEntityAnim(entity 7, animation 25, speed 30.0, CONTROL 1), with several unknown predicates.
  6. Unconditional at this level (both states): SlotAllocate(world, 1); SlotCreate(world, 2, a pointer to a module data cell); ControlThreadsAllocate(world, 1); ControlThreadStart(thread body, world).
- Update (every tick): one unconditional call into the shared entity-control routine (SPEC-0043; High). The animation chain of Init step 5 is its add step, so the records exist only when state 102 is set: entities 7, 8 and 9, each idle animation 25, interval 5 to 7 s, dwell 8 to 12 s, animations 110, 95 and 40 (2.5 s) and 41 (1.3 s); waypoints (each with dwell and orient) locators 0 to 2 for entity 7, 3 to 6 for entity 8 and 7 to 11 for entity 9, walked in a cycle.
- Proximity callback 1 (trigger index 6). Entity from the proximity record at +0x0. Unconditional:
  - SetConversationTarget(entity, true); LevelMsg(message 0, no world, bool 1, WAIT 1); EndConversation(entity).
- Proximity callback 2 (trigger index 9). Unconditional:
  - SetConversationTarget(entity, false); LevelMsg(message 6, bool 1, WAIT 1); EndConversation(entity).
- Proximity callback 3 (trigger index 8). Unconditional:
  - SetConversationTarget(entity, false); LevelMsg(message 12, bool 1, WAIT 1); EndConversation(entity).
- Proximity callback 4 (trigger index 7). Unconditional:
  - SetConversationTarget(entity, false); LevelMsg(message 9, bool 1, WAIT 1); EndConversation(entity).
- Teleport callback A (teleport index 0): AddBlackboardMessage(type 2, value 50, eWorlds 159); StartExteriorReplacementLoad(eWorlds 159, -1, no colour).
- Teleport callback B (teleport index 1): AddBlackboardMessage(type 2, value 10, eWorlds 159); StartExteriorReplacementLoad(eWorlds 159, -1, no colour).
- Transitions: this module makes exactly one kind of transition, an exterior load to world 159 (pack 1358), made by both teleport callbacks. It makes no other load, enter or leave call, and no set-return call. The world-activation callback reads the blackboard value for world 180 but does not load world 180 (world 180 is loaded by module 1358, SPEC-0043).
- World-activation callback (runs when this world becomes active):
  1. Debug line with its argument.
  2. v = GetBlackboardMessageValue(type 2, eWorlds 180); module 1414 branches directly on this return value.
  3. Debug lines with v. Module 1358 writes value 50 or 10 for world 180 immediately before requesting that world; world 180 maps to pack 1414 in the Money District region data.
  4. If v == 50: MoveHero(locator 13, bool 1, CONTROL 1, unk 1); SetToNormalCamera(5.0, 0.0, bool 1); AddBlackboardMessage(type 2, value -1, eWorlds 180).
  5. If v == 10: MoveHero(locator 12, bool 1, CONTROL 1, unk 1); AddBlackboardMessage(type 2, value -1, eWorlds 180).
  6. If v is neither 10 nor 50: AddBlackboardMessage(type 2, value -1, eWorlds 180) only.
  - Step 6 clears the blackboard value on every path.
- Thread body (started from init in both cases; its body reads state 102):
  - Reads state 102.
  - If 102 is set: SlotLoad(world, the stored slot, 1, keep true, activate true); SetToNormalCamera(15.0, 0.0, bool 1).
  - If 102 is clear: SlotLoad(world, the stored slot, 0, keep true, activate true); Wait(3.0); SetAI_Agro(0); DisableInvisibility(0); SetAI_Agro(1); DisableInvisibility(1); SetToNormalCamera(15.0, 0.0, bool 1).
  - No loop calls, so the thread reads as a one-shot setup. Not confirmed.
- Reached routine: one routine with no host calls is called from a helper that init calls (no gate inside that helper; the gate on init's call to the helper is not established).
- Dormant routines: none is referenced by init, update, a callback, the thread, or a live routine.
  - Some have no host calls.
  - One calls a helper that itself has no host calls (a no-op in effect).
  - None is described as running.
- Test vectors:
  - Init with state 102 set: four proximity triggers registered; the thread also starts.
  - Init with state 102 clear: no proximity triggers; the thread starts and runs the wait and the agro/invisibility calls.
  - Teleport index 0: value 50 written to the blackboard for eWorlds 159, then the exterior replacement load.
  - World activation with blackboard value 50: MoveHero to locator 13, then the value is reset to -1.
  - World activation with blackboard value 10: MoveHero to locator 12, then reset.
- Edge cases: the world-activation step 6 resets the value on every path, so a second activation without a teleport finds value -1 (inferred from the code, not followed).
- Confidence: High for the init, the four proximity callbacks, the two teleports, the thread, the dormant-routine status, the per-tick routine (SPEC-0043) and the world-activation values and their direct transition writers.
- Open questions:
  - Meaning of state 102 and the semantic meaning of the blackboard values (unknown).

  - Meaning of the slot set words in the thread's SlotLoad calls (unknown); the slot is the stored slot-create value.
  - Conversation message ids 0, 6, 9 and 12 are not yet mapped to level data or text. The load for eWorld 159 goes to pack 1358; this module does not load eWorld 180.
  - Locators 12 and 13 are from level data, not yet mapped.
  - Whether the thread body ever loops (not confirmed).
  - Resolved (High for the setters; Medium-High for the gate map): the gates are the Altador legend flags, which turn on in play as each shrine is finished: 100 (Gladiator legend, Coliseum, movie 61), 102 (Treasurer legend, Treasury, movie 63), 104 (Rogue legend, Rogue shrine, movie 65) and 106 (Petpet-Handler legend, Petpet shrine, movie 67). Start stage 7 sets all four (SPEC-0049). Flag 102 gates the four proximity registrations and the three idle records, so before the Treasurer legend only the two teleports and the activation callback exist; no callback tests a flag.


### 1421: Interior, entity proximity conversation room

- Family: Interior.
- Exit: present. It makes no host calls. Load and unload hooks exist and make no host calls.
- Init, in order:
  1. Adds a blackboard message (type 2, value 30, world 159).
  2. Reads game flag 102.
  3. If flag 102 is set: registers the on-enter callback (CB1) with mode 0, registers the entity proximity callback (CB2) with proximity flag 1, and runs the idle helper with entity 0, animation 0, speed 30, control option 1. The idle helper's own gate has one extra test that is not known here.
  4. Unconditional at this level: allocates and creates one slot, allocates one control thread, and starts control thread CB3. The flag 102 test selects only the registration and idle-helper calls in step 3.
- CB1, on enter (no entity argument):
  - Flag 100 clear: nothing.
  - Flag 100 set and flag 106 set: shows message 0 (wait 1).
  - Flag 100 set and flag 106 clear: shows message 2 (wait 1).
- CB2, entity proximity (the record's first word is the entity id):
  - Every call first draws a random number from 1 to 3. The draw happens before any flag test, so it is consumed even when nothing else follows.
  - Flag 605 clear: nothing.
  - Flag 605 set, flag 679 clear: removes item 33 from the hero, sets flag 679, sets the conversation target to this entity, shows message 13 (wait 1), and ends the conversation with this entity. Flag 679 then blocks this branch from repeating.
  - Flag 605 set, flag 679 set, draw is 1: shows message 10 (wait 1).
  - Flag 605 set, flag 679 set, draw is 2 or 3, flag 100 set: shows message 4 (wait 1) if flag 106 is set, otherwise message 7 (wait 1).
  - Flag 605 set, flag 679 set, draw is 2 or 3, flag 100 clear: nothing.
- CB3, control thread (started only from the flag 102 clear branch of init):
  - Checks flag 102 once at its start. If set, it loads one slot (the stored slot-create value, activate true, with the set index chosen by flag) and then sets the normal camera (blend 15, mode 0, bool 1) twice.
  - Appears to run once and return.
- Helpers:
  - Idle helper: queries the frames-per-second value, picks a random animation, and loops an entity animation at speed 30 with control option 1. Called from init with constant arguments.
  - Registration helper: registers CB1 and CB2 (init, flag 102 set branch).
  - Large NPC helper: the shared entity-control routine of SPEC-0043 (rules there; High). The idle helper of Init step 3 is its add step, so with flag 102 set the table holds one record: entity 0, idle animation 0, interval 8 to 15 s, dwell 5 to 7 s, animations 40, 95 and 96 (2.5 s) and 41 (1.3 s), no waypoints. With flag 102 clear the table is empty and the helper does nothing. The extra test in the idle helper's gate is the add step's nine-record limit (Medium-High).
- Update: calls the large NPC helper every tick (no host call of its own).
- Dormant routines: they have no host calls.
- Threads: one (CB3), started in init in both cases.
- Edge cases:
  - Item 33 is removed only the first time the flag 605 set, flag 679 clear branch runs, because flag 679 is then set.
  - The draw of 1 is the only draw result that uses the message 10 branch; draws 2 and 3 share the flag 100 and 106 branches.
- Confidence: Medium. Init and all three callbacks are described in full. The large NPC helper's branches are only partly described.
- Open questions:
  - Resolved: flags 100, 102 and 106 are Altador legend flags and 605 and 679 are Package Delivery task 1 flags (see the gate note below); flag 105 is not used here.
  - Item 33 and world 159: from level data or game ids; not yet mapped.
  - Resolved: Update calls the large NPC helper every tick, and its one record is added in play when flag 102 is set (see Helpers).
  - Resolved (Medium-High): the extra gate on the idle loop in init step 3 is the add step's nine-record limit.
  - Resolved (High for the setters; Medium-High for the gate map): the gates are the Altador legend flags, which turn on in play as each shrine is finished: 100 (Gladiator legend, Coliseum, movie 61), 102 (Treasurer legend, Treasury, movie 63), 104 (Rogue legend, Rogue shrine, movie 65) and 106 (Petpet-Handler legend, Petpet shrine, movie 67). Start stage 7 sets all four (SPEC-0049). Flag 102 gates the registrations, the idle record and the thread's slot load; the on-enter callback then reads flags 100 and 106, and the proximity callback reads 605 and 679, which are Package Delivery task 1 accepted and delivered (SPEC-0049).


### 1423: Interior, entity proximity room with item ask-about

- Family: Interior.
- Exit: present. It makes no host calls. Load and unload hooks exist and make no host calls.
- Init, in order:
  1. Adds a blackboard message (type 2, value 40, world 160).
  2. Reads game flag 106.
  3. If flag 106 is set: runs the registration helper, which registers the on-enter callback (CB1), the entity proximity callback (CB2) with proximity flag 1, and the item ask-about callback (CB3) for item 151 with bool 1. Then runs the idle helper with entity 0, animation 0, speed 30, control option 1.
  4. Unconditional at this level: allocates and creates one slot, allocates one control thread, and starts control thread CB4. The flag 106 test selects only the registration and idle-helper calls in step 3.
- CB1, on enter (no entity argument):
  - Flag 104 clear: nothing.
  - Flag 104 set and flag 102 set: shows message 0 (wait 1).
  - Flag 104 set and flag 102 clear: shows message 3 (wait 1).
- CB2, entity proximity (the record's first word is the entity id):
  - Flag 609 clear: nothing.
  - Flag 609 set, flag 683 clear: removes item 33 from the hero, sets flag 683, sets the conversation target to this entity, shows message 7 (wait 1), and ends the conversation with this entity.
  - Flag 609 set and flag 683 set: reads flag 104. If flag 104 is set, it reads flag 102:
    - Flag 102 set: shows message 8 (wait 1), with the conversation target set for this entity.
    - Flag 102 clear: shows message 11 (wait 1), with the conversation target set for this entity.
  - Flag 609 set, flag 683 set, flag 104 clear: nothing further here.
- CB3, item ask-about (item 151): shows message 6 (wait 1). No state test.
- CB4, control thread (started by init on both values of flag 106, after the flag-106 branch rejoins; see init step 4):
  - Checks flag 106 once at its start. If set, it loads one slot (partly known) and sets the normal camera (blend 15, mode 0, bool 1) twice.
  - Appears to run once and return.
- Helpers:
  - Idle helper: the same shape as in 1421 (frames-per-second query, random animation, looped entity animation at speed 30).
  - Registration helper: registers CB1, CB2 and CB3.
  - Large NPC helper: the shared entity-control routine of SPEC-0043, as in 1421. Update calls it every tick, with no gate. With flag 106 set the idle helper adds one record: entity 0, idle animation 0, interval 8 to 15 s, dwell 5 to 7 s, animations 40, 95 and 96 (2.5 s) and 41 (1.3 s), no waypoints (High).
- Update: calls the large NPC helper every tick (no host call of its own).
- Dormant routines: they have no host calls.
- Threads: one (CB4), started in init in both cases.
- Edge cases:
  - Item 33 is removed once, the first time flag 609 is set and flag 683 is clear, because flag 683 is then set.
  - Messages 8 and 11 need flag 609 and flag 683 both set. The first of them, message 8, also needs flag 104 set.
- Confidence: Medium. Init, CB1, CB2, CB3 and CB4 are described in full. The large NPC helper is only partly described. The exact gate for the conversation target calls in CB2 is partly unknown.
- Open questions:
  - Resolved: flags 102, 104 and 106 are Altador legend flags and 609 and 683 are Package Delivery task 5 flags (see the gate note below).
  - Item 151 and item 33: not yet mapped.
  - World 160: from level data or game ids; not yet mapped.
  - Resolved (High for the setters; Medium-High for the gate map): the gates are the Altador legend flags, which turn on in play as each shrine is finished: 100 (Gladiator legend, Coliseum, movie 61), 102 (Treasurer legend, Treasury, movie 63), 104 (Rogue legend, Rogue shrine, movie 65) and 106 (Petpet-Handler legend, Petpet shrine, movie 67). Start stage 7 sets all four (SPEC-0049). Flag 106 gates the three registrations, the idle record and the thread's slot load; the on-enter callback then reads flags 104 and 102, and the proximity callback reads 609 and 683, which are Package Delivery task 5 accepted and delivered (SPEC-0049).
  - Resolved: Update calls the large NPC helper every tick, and its one record is added in play when flag 106 is set.


### 1428: Exterior, flag-gated trigger set, hero swap and act-start sequence

- Family: Exterior.
- Exit: present. Its body makes no host calls.
- Hooks (load and unload): no host calls.
- Init:
  - Stops effects 0, 24, 25, 26, 1 and 11 to 20 unconditionally.
  - Calls a local helper that reads game-state flag 322. Init branches on the helper's result, not on the flag itself.
  - Flag 322 set: registers the activate-world callback and an AI-player activate callback; registers nine trigger callbacks (see Callbacks). Trigger indices: CB1 on 5, CB2 on 4, CB3 on 2, CB4 on 1, CB5 on 0, CB6 on 3, CB7 on 6, CB8 on 7, CB9 on 8. Allocates 1 control thread and starts the control thread named THREAD-A (below). Inits objects 0, 1 and 2 (not forced). Sets level state 19 to true (High).
  - Flag 322 clear: registers a second AI-player activate callback (no triggers, no thread), and stops effects 2 to 10.
- Step bits: a module word of step bits (file value 0) drives THREAD-A. "S3" means bit 3. A set-bit helper sets or clears one bit.
- Update (runs every tick, when flag 322 is set): calls an update helper that sets level state 2 to the value of the pulse word (file value 0; CB4 sets it, and it is cleared again after one tick), runs H13 (the shot meter, below) every tick, and, while flag 329 is set and flag 330 is clear, reads the system time for the switch-press timing of CB4. A debug-mode test follows; its branch is debug-only.
- Callbacks (trigger callbacks, names follow registration order at init):
  - CB1 (trigger 5, the shot target): no host calls of its own; each shot hit on it adds a time stamp to the shot meter (below) while the meter is armed.
  - CB2 (trigger 4): when the record's object word is non-zero, sets S0, which starts THREAD-A's sequence (no host call of its own).
  - CB3 (trigger 2): if flag 327 is clear, sets it and arms the shot meter.
  - CB4 (trigger 1): when the record's object word is non-zero: stores the current game time and reads flag 328. If flag 328 is set and 329 is clear: sets flag 329 (the next taunt comes 0.5 s later). With flag 329 set and 330 clear: level state 2 is pulsed for one tick (the pulse word is set; the update copies it into level state 2 and it is cleared again), which stops the Darkest Faerie's summons, and the next taunt delay is set: 20 s when a switch was pressed less than 5 s before this touch, otherwise 0.5 s (5 s when the last taunt was under 3 s ago), counted from one second after the touch.
  - CB5 (trigger 0): when the record's object word is non-zero and flag 330 is set: sets S5 (no host call of its own).
  - CB6 (trigger 3; High): when the record's object word is non-zero (the selected hero's body is in the volume) and flag 327 is set and flag 328 is clear, it calls SetExternalMissileTarget(4). The call arms a one-shot aim override for the hero's next shot: the first shot of the next shot burst is aimed at the position of locator 4, (28.53, 11.16, -341.87), the centre of the trigger 5 shot target, instead of the normal aim point, whatever way the hero faces; the override then clears, and the other shots of a burst use the normal aim. The shot keeps the target it would have had (SPEC-0030 row 90). While the hero stands in trigger 3's sphere the callback re-arms the override every tick, so the first shot fired from there goes at the target; an armed override stays armed after the hero leaves the sphere or flag 328 is set, until one shot is fired. Flag 328 ends the help.
  - CB7 (trigger 6), CB8 (trigger 7), CB9 (trigger 8): the three puzzle switches 0, 1 and 2. When the record's object word is non-zero, flag 329 is set, flag 330 is clear and none of S2, S3, S4 is set: shows a square button context prompt (context 4); if button 0 is then pressed, sets S2 (CB7), S3 (CB8) or S4 (CB9), and THREAD-A runs the switch press H15 (High).
- THREAD-A (control thread started at init, when flag 322 is set). Each tick it runs the first rule that applies, then sleeps 1 tick:
  1. S0 clear: nothing (it waits for CB2).
  2. S7 set: the act-start sequence (below).
  3. One of S2, S3, S4 set: the hero event sequence H15 (below), for the record that matches the bit.
  4. S5 set and S6 clear: the flag 331 body: sets flag 331 to true, deactivates entities 1 and 0, sets S7.
  5. S1 clear: the boss-room return (below); it ends by setting S1.
  6. Otherwise: the per-tick hero helper (below): the hero-swap helper H17, the flag 330 check and the game-over check, in that order.
- Switch puzzle (High; the gate of flag 330). Switch k (k = 0, 1, 2; triggers 6, 7, 8) has three states, first, second and third, shown by the effects (2, 3, 4) for switch 0, (5, 6, 7) for switch 1 and (8, 9, 10) for switch 2; its object is k, its hero locator 5 + k, and it uses camera 4. Switch k is linked to switch k + 1, and switch 2 to switch 0. A press advances switch k and its linked switch by one state each (first -> second -> third -> first). When flag 328 is set, the switches are put in their start states: switch 0 first, switch 1 second, switch 2 third.
  - Flag 330 check (part of rule 6): when flag 329 is set, flag 330 is clear and all three switches are in the first state: sets flag 330, starts effect 0 (world), and the next taunt comes 0.5 s later. This is the setter of flag 330 (High).
  - Solution: press switch 0 twice and switch 2 once, in any order. Counted modulo 3 this is the only solution (switch 1 is pressed zero times).
  - Test vector: start (first, second, third). Switch 0: (second, third, third). Switch 0: (third, first, third). Switch 2: (first, first, first) -> flag 330.
- Game-over check (part of rule 6; see SPEC-0055): when flag 328 is clear, IsHeroDead(1) is non-zero and the hero does not hold item 50: if the active hero (WhichHeroIsActive) is hero 1, SetHeroSwapMode(1); otherwise SetGameOver(0) (High).
- Object effect helpers (called from triggers and from the start/stop sequence; callers not fully known):
  - The effect and object values passed to StartEffect and StopEffect in these helpers are read from record fields (from record) on the paths the code resolves.
  - Object-record helper (H15 calls it for a switch record): it advances the switch one state and shows it: from the first state it stops the first effect and starts the second; from the second it stops the second and starts the third; from the third it stops the third and starts the first. Record layout is data, not code.
  - Stop-three helper: stops three effects from its caller-supplied arguments.
  - Start-or-stop helpers (three, one per effect field): start the effect if the caller's flag is non-zero, otherwise stop it.
  - Object-scene helper (argument 4 or 5): sets the delay to the next taunt (argument 4: 0.5 s; argument 5: the trigger-1 delay rule of CB4).
- H13, the shot meter (run every tick by the update helper; High; the gate of flag 328): armed by trigger 2 (CB3). It keeps a ring of 10 time stamps: each shot hit on trigger 5's target writes the game time into the next slot, and a stamp older than 10.0 s drops out. The count of live stamps lights effects 11 to 10 + count (effect 1 stays on while counting). When the count reaches 10 (ten hits within 10 s): flag 328 is set, effect 1 is stopped and the three switches are set to their start states. After 328 is set the meter stops effect 1 and starts effect 0 (Medium).
  - Test vector: hits at 0.0, 1.0, ..., 8.0 s and a tenth at 9.5 s: count 10 at 9.5 s, flag 328. Nine hits at 0.0, 1.0, ..., 8.0 s and the tenth at 10.5 s: the 0.0 stamp has dropped out, so the count is 9 and no flag is set.
- Act-start sequence (THREAD-A rule 2):
  - Plays FMV 56 (wait), boss fight tweak (instance 12, stage 1), starts act 4, then waits (one-tick sleeps) until the act start is done.
  - When done: plays FMV 91, sets game state 333 to true, plays FMV 57, sets the return world for FMV to world 178, plays FMV 58.
  - Flag 333 effect (High): this set comes before the hero first reaches world 178 (pack 1411), and no level script clears 333. Pack 1411 trigger 0 plays movie 59 only while 333 is clear, so movie 59 never plays in play (SPEC-0044, 1411; SPEC-0058).
- Hero event sequence H15, the switch press (THREAD-A rule 3): picks switch k of the pending bit (S2: 0, S3: 1, S4: 2), then PlayCamera(4) (1.0 s, wait), MoveHero(locator 5 + k, control mode 1), Wait(0.3 s), hero animation 141, switch k's object plays, the object-record helper advances switch k and its linked switch, starts/stops effects 0, 147 and effect id 10820, waits until the object finishes animating, then activates it (not forced), returns to normal camera, and clears S2, S3 and S4.
- Taunt helper H17 (THREAD-A rule 6; H16 has the same body; High): the Darkest Faerie's taunts. When the taunt delay has run out, no hero is occupied, no swap is in progress and the active hero's hit points are above 0, it shows the next taunt line, chosen by the first set flag: 330: lines 0, 1, 2, then 1 and 2 alternating; 329: lines 3, 4, 5, then 4 and 5 alternating; 328: 6, 7, 8, then 7 and 8; 327: 9, 10, 11, then 10 and 11; none: 12, 13, 14, then 13 and 14 (Medium for the groups after the first, which follow the same pattern).
  - Delay to the next taunt: 20 s after a taunt for the first three taunts, then 60 s; 0.5 s right after flag 327, 328 or 329 is set (and after 330, by the flag 330 check); and the trigger-1 rule of CB4 (20 s after a switch press followed within 5 s by a trigger-1 touch; otherwise 0.5 s, or 5 s when the last taunt was under 3 s ago, one second after the touch).
- Boss-room return (THREAD-A rule 5; High, step order and arguments from the call sites): (1) FadeScreenOut(black, 0.5 s, wait), the colour read from a module data word that holds black (0, 0, 0); (2) PlayFMV(55, wait); (3) SetGameState(326, true); (4) ActivateEntity(1); (5) RemoveItemFromHero(item 117, count 1); (6) GiveItemToHero(item 116, count 1, not forced); (7) MoveEntity(entity 0 = Queen Fyora, locator 1, probe on, control 1, face on); (8) MoveEntity(entity 1 = the Darkest Faerie, locator 2, probe on, control 1, face on); (9) MoveHero(locator 3, probe on, control 1, face on); (10) SetToNormalCamera(distance 10.0, angle 3.14, release true); (11) BossFightTweakHero(instance 12, stage 0); (12) FadeScreenIn(black, 1.0 s, wait); (13) ActivateObj(object 3, not forced); (14) StartEffect 24, 25 and 26; (15) set S1. All three moves, the camera and the hero refill happen under the black fade, between the end of the 0.5 s fade-out and the start of the fade-in, with no wait between them, so the player never sees Queen Fyora, the Darkest Faerie or the hero jump. Entity 0 is placed at locator 1, entity 1 at locator 2 and the hero at locator 3 (positions in the Arena list below), each with the ground probe, the locator's facing and control option 1 (the body returns to its stored heading on the next tick). This routine is the only boss-room return on the disc; packs 1394 and 1300 have no such scene (module 1300's routine 44 is a different movie scene and calls no move or refill). Test vector: steps 3 to 11 contain no wait, so they run in one script pass on the tick the movie ends, and on the first tick of the 60-tick fade-in the hero is already at (17.91, probed height, -308.36), Queen Fyora at (0.81, probed height, -303.4) and the Faerie at (0.02, probed height, -359.43).
- Story order in play (High for the gates): trigger 4 (S0) runs the boss-room return (flag 326, movie 55); trigger 2 arms the shot meter (327); ten hits on the target within 10 s set 328; trigger 1 then sets 329; the switch puzzle (switch 0 twice, switch 2 once) sets 330; trigger 0 then sets S5, so THREAD-A sets 331 and S7 and runs the act-start sequence (movies 56, 91, 57, 58, act 4, flag 333, return world 178).
- Arena (world data of pack 1428; the duel positions and the layout summary are in SPEC-0054, Arena build guide):
  - Locators (entry records): 0 (0.0, 10.65, -269.6) facing -z, the arrival point in the entry corridor; 1 (0.81, 8.37, -303.4) facing -z and 2 (0.02, 8.37, -359.43) facing +z, where the boss-room return moves entity 0 (Queen Fyora) and entity 1 (the Darkest Faerie), 56.04 apart (their record positions, (0.2, 8.37, -325.03) and (-22.96, 8.37, -314.75), are only where they stand before that move); 3 (17.91, 10.65, -308.36) facing (-0.5, 0, -0.866), where the return puts the hero; 4 (28.53, 11.16, -341.87), the shot-target spot and the external missile target; 5, 6 and 7 (-44.04, 9.51, -320.33), (-44.04, 9.51, -325.36) and (-44.04, 9.51, -330.40) facing -x, the hero spots of switches 0, 1 and 2.
  - Volumes by trigger index: 0 sphere (43.5, 7.44, -326.96) radius 20; 1 sphere (-51.48, 7.44, -325.38) radius 21.3; 2 sphere (47.99, 7.44, -338.15) radius 31.01; 3 sphere (31.4, 7.44, -343.16) radius 21.07; 5 sphere (28.56, 11.16, -341.9) radius 2.05 (the shot target, 33.5 from the Faerie's post); 4 the static rectangle at z = -277.28 (origin (-10.16, 1.71, -277.28), normal +z, 29.31 wide in x, 20.31 tall) that the hero crosses on entering; 6, 7 and 8 the switch plates (4 x 4, origins (-46.92, 7.56, -321.83), (-46.92, 7.56, -326.85), (-46.92, 7.56, -331.88)). The chamber has one save point, (-11.158, 8.252, -257.921), in the corridor north of the gate.
  - Placed object 3 is a solid box (x -5.45 to 5.46, z -285.96 to -283.66, 16.1 tall, top y 21.79) that the boss-room return activates; it closes the north mouth behind the hero. The walk polygon is a 28-sided circle of radius about 40.4 round (0.0, -325.6); the floor is y 7.44 with a shallow round depression (6.64) at the centre and a well (about 5.0) in the east bay. Camera tracks 4 to 9 are fixed shots at x -41 to -32.8, y 13.5 to 19.3. Music row 187 (stream 173, bank 30, block kept, selector 0).
- Dormant routines (no live reference): two routines, each with no host calls and no references.
- Load-time constructor (High): the load hook runs it at every load, before Init. It clears one module record (17 words) that already holds 0 in the file, so Init and the callbacks see the same values as without it.
- Threads: THREAD-A (from init, while flag 322 is set). No other thread; the act-start sequence is a step of THREAD-A.
- Edge cases and error handling:
  - CB1 makes no host calls; it feeds the shot meter. CB2 and CB5 make none of their own but set step bits.
  - A switch pressed out of order is not a dead end: the switch states cycle, so any state can still reach the solution.
  - Effect ids come from level data, "from level data, id not yet mapped". Effect id 10820 (H15) is a fixed value.
  - Flags 322 and 326 to 333 are game-state ids; their meaning unknown.
- Confidence: Medium overall. High for init and trigger wiring, the step bits and THREAD-A's rule order, the shot meter, the switch puzzle, the taunt delays, the flag 328, 329, 330 and 331 setters, the act-start sequence and the game-over check. Medium for the taunt groups after the first. Low for the update's debug branch.
- Open questions:
  - Meaning of flags 322 and 326 to 333 (game-state ids; meaning unknown).
  - Resolved: H13's count is the number of shot hits in the last 10 s (a ring of 10 stamps).
  - Resolved: the object-scene helper sets the taunt delay.
  - Resolved: level state 2, pulsed by trigger 1, stops the Darkest Faerie's summons (SPEC-0054).
  - Resolved: the game-state write and the effect start once listed as unregistered callbacks are the flag 330 check (flag 330, effect 0), reached in play through THREAD-A's per-tick helper (rule 6); the talking check sits in the same helper chain (Medium for which call that is).

## Edge cases and error handling

- Large per-tick routines. Modules 1353, 1363, 1421, 1423, 1409, 951, 1324 and 1371 repeat long decision trees each tick. Their sections describe the rules in words and mark the parts not followed as partly described. The clean side must not infer the unlisted branches.
- Duplicate registrations: none. In 1353 and 1371 each trigger is registered once (1371 through its helper H4, called by init in branch B).
- One-shot dialogue in per-tick code: modules 1130 and 1324 guard their per-tick and trigger messages with shown-once words, so each line shows once per module load (High).
- Constant flag 0. Module 1394 reads game-state flag 0 (a fixed flag number). No module sets it, so its clear side always runs; the load back to pack 1353 does not depend on it (it opens through the shrine room trigger; see the 1394 section).
- Dormant code. Modules 742, 800, 822, 951, 989, 1090, 1096, 1130, 1176, 1324, 1353, 1371, 1382, 1394, 1409, 1414, 1421, 1423 and 1428 have routines that no entry, callback or thread references (static; each section says which). These do nothing at run time unless a caller is found. In 1324, the solved-scene sequence is live through the stage-1 trigger callback, the object-time helper is called by Update, and the flag-239/system-time constructor runs from the load-time list; one other routine remains unreferenced. Load-time constructor routines are not dormant: the load hook runs them at every load, before Init (SPEC-0030). Modules 951, 1096, 1324, 1382, 1409, 1414, 1421, 1423 and 1428 have them; only the constructor of 1324 makes host calls (section 1324), and the others only set module data to its start values (in 951, 1409, 1414, 1421 and 1423, the entity-control table reset of SPEC-0043). Modules 985 and 1078 have none. The large NPC helper of 1421 and 1423 is not dormant: Update calls it every tick. Module 1382's lit body is reached from trigger callbacks on gated paths, so it is not dormant either; its entity activation body is reached only from a dormant routine, so it never runs. Module 989's one dormant routine does make host calls (PlayCamera on track 5, then ReturnToNormalCamera(-1, true)); it does not run unless a caller is found (section 989).
- Empty Exit bodies. Modules 1409, 1324, 1392, 951, 1414, 1130 have an Exit with no effect (per section). Others are described in their sections.

### Pack 1324 camera return and FMV reachability

On the live plate callback, the selected stage-2 branch runs the camera helper before any FMV: it plays hero animation 0 with control mode 1, starts camera track 0 or 1 with wait, waits one additional second, fades out, returns to normal camera, fades in without waiting, and issues a second normal-camera return. The callback contains no explicit player-control block. Control mode 1 returns the hero to normal state when the one-shot animation ends, so the camera wait is not itself a script-owned input lock; the exact tick when input becomes available relative to the camera remains uncertain. When all three plate indicators are active and both movable-block state words are nonzero, the same registered callback reaches the solved-scene helper: it sets flag 239 before FMV 23, then performs the scene's fade and sound sequence. Thus the helper is live-reachable; physical plate and block access remain unconfirmed. (High for the script path; Medium for practical input timing and physical access.)

## Test vectors

- Sleep(15 frames) on the tick the hero enters a volume resumes 15 ticks later (SPEC-0030 Sleep rule).
- Wait(0.5) yields 30 ticks at 60 ticks per second.
- 1363, id 42 with flag 101 set and flag 103 clear, three talks with draws X = 1, 2, 1: lines 3 (set 1, counter 0), 10 (set 3, counter 1), 5 (set 1, counter 2); the counter shared by the sets goes back to 0.
- A trigger record with state 1 and a non-null object pointer enters the entered branch of the module sections that test state == 1.
- Module-specific vectors are in each section.
- 1247, plate scene CB2 (trigger 1), hero steps on at tick T and steps off after 18 contact ticks (last contact tick T + 17): the thread runs Part A (panels 8 and 10 and plate object 6 switched in, plate animation 6 for 20 ticks), reads the state at tick T + 20 as 3, and takes the quick exit: object 6 off, object 7 on, animation 7 once, end. Panels 8 and 10 stay at their start poses and no camera plays. With 19 contact ticks (last contact tick T + 18) the read at tick T + 20 sees 2, so the thread plays the two look-at cameras and panel animations 8 and 10, waits until the state reads 0 (the resume of tick T + 22), then switches panels 9 and 11 and plate object 7 in and plays 7, 9 and 11.
- 1247, plate scene CB5 (trigger 2), hero stays: Part A at tick T, plate animation 22 ends at T + 20, panel 24 starts rising at T + 20 and ends 4.33 s (260 ticks) later, panel 26 rises from T + 280 to T + 340, panel 29 from camera 23's end to 1.53 s later. Panel 26's bottom edge (0.72 at its start pose) clears the head of a hero standing on ground 1.3 (top 4.8) when it has risen about 4.1 units, about 0.4 s into its animation if the rise is even.
- 1247, panel 3 (CB1): box x -87.2 to -71.3, z 169.8 to 172.1, y 8.2 to 23.7; ground at x -79.5, z 170 is 9.2, so a hero there (y 9.2 to 12.7) is stopped; after the rise of 7.55 the box spans y 15.75 to 31.26 and the hero passes.
- 1247, CB3 with the other plate (trigger 6) held (state 1, 2 or 3) and own state 1: deactivate 12, activate 13, effects 4 to 7 started, animation 13 waited, effects stopped, then the wait for state 0 and then Part B with the other plate still held: deactivate 13, activate 12, animation 12, camera to normal; panels 16 and 18 are never touched.
- 1249, CB2 (trigger 3), hero stays: Part A at tick T (objects 10, 13, 1 off; 0, 12, 11 on; effects 12 to 15 started; animation 11 waited for 20 ticks; effects stopped), then look-at camera 5 on object 12 and animation 12 (3.33 s), look-at camera 4 on object 0 and animation 0 (1.6 s), camera to normal; then the wait returns when the state reads 3 (3 ticks after the last contact tick) and Part B plays 13, 10 and 1.
- 985, flag 281 set: the arrival callback moves the hero to locator 9; H10 then repeats that move before entity 0 boss presentation. The filter admits Gelert ordinals 0 to 4. The fight is reached from this arrival route; it is not established whether the first streamer pass precedes or follows the callback.

## Open questions

1. Resolved: in modules 1421 and 1423 the large helper for non-player characters is the shared entity-control routine of SPEC-0043; its record is added in play when flag 102 (1421) or 106 (1423) is set, and the extra idle-loop gate is the add step's nine-record limit (Medium-High).
2. Resolved: module 1176's slots, scene variants, locators, arrival point and arena bounds are given in its section (High).
3. Resolved: module 1078's four registered trigger callbacks, including the button gates and action branches, are described in section 1078. Its interactive button callbacks require a newly pressed Square; button code 0 and the newly-pressed query are documented in SPEC-0030, section 3. Record and object identity meanings that remain unknown are listed in section 1078.
4. Module 822: resolved for the two Init step-8 alternative helpers (section 822) and for the Update navmesh step (68 entities, nearest navmesh cached once found), the dormant chooser and a second unreferenced routine. Two trigger scenes (CB5 and CB7) are only partly known.
5. Module 1096: resolved for the state thread (CB7 rules, finale and dispensers; High). Still open: the debug update (Low).
6. Resolved for the per-tick helper: in 1409 and 951 it is the shared entity-control routine of SPEC-0043, identical in both, with no unknown predicate. Still open: one slot-load argument.
7. Resolved for 1324 (the plate puzzle; record fields are state, index and block number), 1392 (gates, levers and helper levers) and 1130 (the junction rules of the lever puzzle). Resolved as well: the three partial-flow outcomes of the 1130 test button follow junctions 2 and 3 (see its section; High).
8. Resolved: the 1392 fall-in runs once per module load while flag 696 is clear.
9. Resolved: module 1414 branches directly on the type-2 blackboard value for world 180; module 1358 writes the values 10 or 50 immediately before requesting the world, which maps to pack 1414 (section 1414; High).
10. Resolved: in 1428, H13 is the 10-stamp shot meter (10 s drop-out), the object-scene helper sets the taunt delay, and the prompt triggers are the three switches of the 3-state switch puzzle (solution: switch 0 twice, switch 2 once). Also resolved: level state 2, pulsed by trigger 1, stops the Darkest Faerie's summons.
11. Module 1249: resolved. The three levers set flag 80 on the third lever up, in any order. The word the section calls the kind word at +0x10 is the trigger state (1 first tick of contact, 2 while inside, 3 and then 0 after leaving; SPEC-0030, Layout section 3), so state 1 means the hero has just entered, the second parts of CB2 to CB5 run only while the hero is still inside after the first part, and the words at +0x1c, +0x20 and +0x24 are the object, shot and attack words (High).
12. Module 1247: resolved. Test T is the other plate's state word (Medium for CB4), and trigger 4 sets flags 78, 269 and 270 once item 29 is held (no AI-dead requirement).
13. Module 1382: resolved: init has no game-state test, and every trigger callback is registered and live. Resolved: the pedestal element test in CB1 (the weapon Mote word against the two Motes of the record), CB2 (it sets the armed word), and the CB5 and CB6 index pairings (section 1382).
14. Resolved: thread B checks entity 6, the Giant Slorg in region resource 744 ordinal 0; entity 7 (Haplo) shares the range but is not checked (section 742).
15. Module 1090: resolved: Init, the hurt zones, the firing loops (threads C and D) and the Update, which is the hazard timer step, with every helper caller known (High).
16. Module 989: Thread A reads only state 10; set state 10 gates looping and stopping animation on object 2. The Black Knight's enraged high swing sets level state 10 on the tick its arm-open step ends and clears it when the under-arm stun ends (SPEC-0012), so thread A's object-2 loop runs during the high-swing stun and its stop branch runs when the stun ends. Thread B uses states 1 to 7 as staged gates: state 2 gates the pre-fight movie and tweak before setting 3 and 4; helper H1 then sets 5 and 1; state 6 is set after 1 and boss-defeated state 0 when 6 is clear; the post-fight block requires 5 and 6 set and 7 clear. State 0 means boss defeated (SPEC-0012). Names or design intent for states 1 to 7 are not supported by the pack scene/trigger data. Resolved: the post-fight block runs once. Thread A ramps the speed factor up by 0.1 per tick (applied 0.0 to 0.9) and down by 0.1 per 5 ticks (1.0 to 0.1) before StopAnim.
17. Module 1353: resolved: the trigger 0 to 3 registrations are made unless flags 353 and 104 are both set, and the proximity registrations have no gate. The Init slot setup depends on flag 104.
18. Resolved: module 1363's Init gates, the Mortog thread (shakes, odd-numbered-rise hold times, ease steps, done at four Mortogs, flag 692) and InitMortogBehavior's floats (25.0 and 30.0) are given in its section; the shrine door also opens on the done word in the same visit.
19. Module 1371: resolved. The Z2 walk cases are three gates with no unknown test: trigger 2 opens wave 1 (flags 342 to 344 clear, object 1 not animating), trigger 4 wave 2 (342 set, 343 and 344 clear, object 1 not animating) and trigger 3 wave 3 (342 and 343 set, 344 clear, objects 3 and 2 not animating); see the 1371 section and SPEC-0059, section 8 (High). No duplicate registration (H4 is the only one), and Z0, Z1 and the wave controller are fully described.
20. Resolved for names: items are named by SPEC-0021 "Item names"; eWorlds ids are area slots named by SPEC-0006 "Area names" and SPEC-0051 "Map area names"; flags that are a quest start or done flag by SPEC-0049 "Titles and flag glosses"; entity ids per pack by SPEC-0006 "Entity names" (High). Still open: other flags, object ids and locator ids have no name source on disc and stay numbers ("from level data") unless the section says otherwise. The WAIT, CONTROL and CAMERA_WAIT values are settled in SPEC-0030 (Layout 2.1, Open question 5).
21. Switch-selected branches (modules 1096, 1363, 1382 and 1428): in 1096 the trigger 0 to 4 routine is fully described by index; in 1382 the CB5 and CB6 cases are fully described by index; in 1363 the trigger F switch is fully described by trigger index (6, 7 and 8; see its section); in 1428 only the listed branches are followed.

23. Unregistered callbacks: resolved for 985 (stage 0's level state 0 write), 1130 (two unreferenced older lever bodies) and 1428 (the flag 330 check, reached through THREAD-A). Resolved for 1324: the only routine address in its data is the load-time constructor, which the load hook runs (section 1324).
24. Module-data gates are resolved (file values and writers) in the sections of modules 742, 800, 822, 951, 985, 1078, 1090, 1096, 1130, 1176, 1247, 1249, 1324, 1353, 1363, 1371, 1382, 1392, 1394 and 1428. Elsewhere, calls under unresolved gates are written as "reached when" statements with Medium or lower confidence.
25. Resolved (High for the setters; Medium-High for the gate map): the gates of modules 1409, 1414, 1421 and 1423 are the Altador legend flags 100, 102, 104 and 106 and, in 1421 and 1423, the Package Delivery flags 605 and 679 (task 1) and 609 and 683 (task 5). The legend flags turn on in play when each shrine is finished (movies 61, 63, 65 and 67) and are all set by start stage 7. Each section gives the gate of its module. Test vector: with flags 102 and 106 clear, module 1414 and module 1421 register no proximity callback and add no idle record; with flag 102 set and 106 clear they register theirs, and modules 1409 and 1423 still register nothing.
26. Resolved: module 1394's exit back to pack 1353 opens through trigger S11 (room word), not flag 0, which nothing sets. The four checkpoint levers work once per load.
27. Resolved (R-0183): the 1363 proximity routines of ids 37 to 42 are three three-line sets chosen by two independent Random(1, 3) draws gated by flags 101 and 103, with one shared counter per id; the fallback set is the third set (High; see the 1363 section).

## Confidence

| Section | Confidence |
| --- | --- |
| Module selection (25 modules, ranks 31 to 55 after the pilot and the SPEC-0037 and SPEC-0039 sets) | High |
| Families (19 Exterior, 4 Interior, 2 Transition) and Exit export in all 25 | High |
| Per-module init and trigger sections | Medium, per section; Low where marked |
| Switch-selected branches (modules 1096, 1363, 1382, 1428) | High for 1096, 1363 (trigger F) and 1382; Medium for 1428 |
| Per-tick and thread bodies of the large modules | Low to Medium, partly described |
| Load and unload plumbing | Low (not established for this set of modules) |
| Host call names and conventions | High (SPEC-0030, SPEC-0032) |






- Hubert package exchange (R-0093; High): the first-time branch requires flag 147 clear, flag 122 clear, and item 98 (Wooden Sword) held. The callback gives item 32 (Package to Meridell), quantity 1, then sets flags 121 and 122 before showing line 59 (WAIT 1). Line 59 contains a second reveal-pass give of item 32, quantity 1, at the package phrase; this grant occurs during text reveal. Item 32 has category 7 and carry limit 1, so the two grant requests leave one package in inventory. Since flag 122 is set before the line opens, the callback cannot repeat this first-time branch while the text is displayed. Whether the reveal-time grant produces a separate inventory notification is unknown.

