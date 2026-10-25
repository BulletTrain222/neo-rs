//! Level behaviour written from the per-level specs (SPEC-0030 section 7: each level as data
//! plus rules in the host-API vocabulary, never by running module code). This holds the game
//! state the handlers read and write (story flags, the blackboard, held items), the actions they
//! return for the host to carry out, and the handlers described so far:
//! - 909 (SPEC-0045): teleport callbacks 0 and 1 (interior entries to slots 72 and 73), the
//!   activate-world routine's flag reset and blackboard placement, and its activation thread's
//!   slot loads by story stage (the farm family and the fourth slot).
//!   Its script triggers (the camera-pair volumes 0 to 3 with line 192, the pump exchange of
//!   index 5, the tutorial line of index 7, the door of index 8) and the flag and item steps of
//!   its Update.
//! - 951 (SPEC-0041): Init (teleport, the chest lock, the thread body's flag, item and slot
//!   steps), the three proximity callbacks and teleport callback 0.
//! - 955 (SPEC-0032): teleport callback 0 (message 40, leave to slot 5).
//! - The road module 678 (SPEC-0042), which packs 678, 686 and 691 carry: Init's slot and trigger
//!   setup by flags, trigger 0's line, the road-fight intro (trigger 1), the fight watch and end
//!   scene, the line handler of triggers 2, 3, 5 and 6, trigger 7's prompt and proximity P.
//! - 1023 (SPEC-0045) villager talk: the three line templates (story chains; the arena and
//!   wheel talk need flags 381 to 385 and 480, which nothing sets yet, and their draws are not
//!   taken), entities 74, 65 and 90, and the return locators of the activate-world routine.
//! - 1148, 1133 (SPEC-0039) and 1150 (SPEC-0042): the Recruitment Centre errands.
//! - 1023 (SPEC-0045): the Meerca chase (trigger 17's chase-start scene, the chase thread and
//!   the load thread's first-slot load and chase resume), the signs (triggers 3, 4 and 6 to 16),
//!   triggers 0 to 2 and the teleports to the interiors (indices 1 to 8) and their line
//!   branches.
//! - The lock chest press of script groups of kind 1 (SPEC-0048 Lock chests).
//! The scenes are staged with the calls of SPEC-0052 (fades, camera plays and returns, hero and
//! entity moves and walks, waits); the host runs each handler's list as its own script thread.
//! Not covered: every other level, and in these levels the objects, effects, movies, shakes and
//! entity animations. Where the specs leave a fade's time, a camera's end mode or a walk's wait
//! unstated, the choices are named at the scene (R-0025).

use std::collections::{BTreeSet, HashMap};

/// What a handler asks the host to do, in order.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Action {
    /// StartEnterInterior(eW slot, -1, -1): only while E is current; entry record 0.
    EnterInterior { slot: usize },
    /// StartLeaveInterior(eW slot, -1, -1): only while I is current; E's entry record 0, then
    /// T reloaded with the slot's pack.
    LeaveInterior { slot: usize },
    /// MoveHero to entry record (locator) i of the current world.
    MoveHero { locator: usize },
    /// LevelMsg: show dialogue line n of the current world.
    Message { line: u16 },
    /// The proximity talk frame around a line: SetConversationTarget(entity, 1), LevelMsg(line,
    /// wait 1), EndConversation(entity).
    Talk { line: u16 },
    /// SetWhinnyPenmesh(world, inner, entrance, locator k): one pen (SPEC-0029 Behavior 5).
    WhinnyPen { inner: usize, entrance: usize, locator: usize },
    /// SlotLoad into one of the level's slots: the region resource to load, or None to empty
    /// the slot (its characters leave).
    LoadRegion { slot: u8, region: Option<u32> },
    /// The sound host calls (SPEC-0031 Script sound calls): PlaySound (a one-shot), LoopSound (a
    /// keep-alive start), VisitLoopSound (a refresh, starting nothing) and KillSound (stop the
    /// first matching voice) on the area slot, at an entry record (locator) or with no position.
    PlaySound { cue: i32, locator: Option<usize> },
    LoopSound { cue: i32, locator: Option<usize> },
    VisitLoopSound { cue: i32, locator: Option<usize> },
    KillSound { cue: i32, locator: Option<usize> },
    /// PlayHeroSound(h, cue, locator, do-not-restart) on the selected hero's voice slot, only
    /// when h is -1 or the selected hero's flag (0 Tor).
    PlayHeroSound { hero: i32, cue: i32, locator: Option<usize>, do_not_restart: bool },
    /// PlayStingStream(id, volume): stingers 0 to 18 (SPEC-0030 row 227).
    PlayStinger { id: i32, volume: f32 },
    /// SetMidiExciteLevel(n) and SetMidiAdventureLevel(f) (SPEC-0009 Intensity inputs).
    SetExcite { level: i32 },
    SetAdventure { level: f32 },
    /// RegisterScriptTrigger(index, routine): the pack-local routine number for a trigger index
    /// (ignored outside the world's capacity).
    RegisterTrigger { index: u16, routine: u8 },
    /// LockChest(id, key item, message): the world's two-entry lock table (SPEC-0048).
    LockChest { id: i32, key: i32, message: i32 },
    /// FadeScreenOut and FadeScreenIn (black) over a time in seconds; with `wait` the thread
    /// yields until the fade reaches its target (SPEC-0030 rows 136 and 137).
    FadeOut { seconds: f32, wait: bool },
    FadeIn { seconds: f32, wait: bool },
    /// PlayCamera(track, WAIT, end mode, speed) (SPEC-0030 row 34).
    PlayCamera { track: usize, wait: bool, end: u8, speed: f32 },
    /// ReturnToNormalCamera(preset locator or -1, release) (row 35).
    ReturnCamera { preset: i32, release: bool },
    /// GrabHeroControl (row 164): the hero in script hold until a release.
    HoldHero,
    /// WalkHero or WalkHeroAndOrient (rows 158 and 159) to a locator.
    WalkHero { locator: usize, wait: bool, orient: bool },
    /// MoveEntity(entity, locator, probe, CTL, face) (row 112), probe on.
    MoveEntity { entity: usize, locator: usize, face: bool },
    /// WalkEntity or WalkEntityAndOrient (rows 106 and 108): stop 0 or less arrives within 1.0
    /// (classes 23 and 25), a positive stop within that radius (24 and 26).
    WalkEntity { entity: usize, locator: usize, wait: bool, orient: bool, stop: f32 },
    /// SetConversationTarget(entity), LevelMsg(line, WAIT 1), EndConversation(entity).
    Converse { entity: usize, line: u16 },
    /// Sleep(n ticks).
    Sleep { ticks: u32 },
    /// Wait until no world or character load is active.
    WaitLoads,
    /// Wait until no dialogue box is up.
    WaitDialog,
    /// PlayEntityAnim (row 105; WAIT waits for its end) or, with `looped`, LoopEntityAnim
    /// (row 104): the entity plays animation id `anim` under script control.
    EntityAnim { entity: usize, anim: u32, looped: bool, wait: bool },
    /// ReleaseEntityControl (row 107) or ReturnEntityAnimToIdle (row 110): base state 1, idle.
    ReleaseEntity { entity: usize },
    /// SetAI_Agro (row 115).
    Agro { entity: usize },
    /// DeactivateEntity (row 114) and ActivateEntity (row 116).
    DeactivateEntity { entity: usize },
    ActivateEntity { entity: usize },
    /// SetAI_HitPointsPercentage (row 49).
    HitPointsPercent { entity: usize, percent: f32 },
    /// LetterBoxOn and LetterBoxOff (rows 140 and 139).
    LetterBox { on: bool, seconds: f32, wait: bool },
    /// Sleep one tick at a time while any of these entities is still walking.
    WaitWalks { entities: [usize; 3] },
    /// SetGameState and GiveItemToHero made by a thread at its point in the scene (not when
    /// the handler decides).
    SetFlag { flag: u16, on: bool },
    GiveItem { item: u16, count: u32 },
    /// Sleep one tick at a time while the hero is farther than `distance` from the locator.
    WaitHeroNear { locator: usize, distance: f32 },
    /// LoopEntityAnim with an animation drawn by Random(1, 2) from the pair.
    RandomLoop { entity: usize, anims: [u32; 2] },
    /// LevelPrompt(line): a yes or no question; the answer runs `on_answer(pack, routine)` in
    /// the same thread.
    Prompt { line: u16, routine: u8 },
    /// Give the item held in a module word (1148's traded-item word; 0 means its file value).
    GiveWordItem { word: u8, default: u16 },
    /// The price multiplier (SPEC-0050 section 5): every live flag-1 item of the world.
    ScalePrices { factor: f32 },
}

/// Wait(f): trunc(f x 60) ticks, the product one console multiply (SPEC-0030 section 4).
fn wait(seconds: f32) -> Action {
    Action::Sleep { ticks: (ps2_float::fpu::mul(seconds, 60.0) as i64).max(0) as u32 }
}

/// The fade time used where a farm scene's fade has no stated time: 0.5 s, the commonest
/// FadeScreenOut and FadeScreenIn time on the disc (SPEC-0030 rows 136 and 137; R-0025).
const FADE: f32 = 0.5;

fn fade_out() -> Action {
    Action::FadeOut { seconds: FADE, wait: true }
}

fn fade_in() -> Action {
    Action::FadeIn { seconds: FADE, wait: true }
}

/// A camera play whose end mode the spec leaves out: mode 1 (held until the scene's return),
/// the commonest on the disc (SPEC-0030 Note on camera calls), with no wait.
fn camera(track: usize) -> Action {
    Action::PlayCamera { track, wait: false, end: 1, speed: 1.0 }
}

fn release_camera() -> Action {
    Action::ReturnCamera { preset: -1, release: true }
}

/// One run of a trigger callback: the record's state word and object word as the fire step left
/// them, and whether Square went down this pass (the button query's newly-pressed word).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct TriggerCall {
    pub index: u16,
    pub state: u8,
    pub object: bool,
    pub square: bool,
}

/// What a trigger callback did: its actions and the Square context it posted (SPEC-0023 section
/// 33), if any.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct TriggerOut {
    pub actions: Vec<Action>,
    pub prompt: Option<u16>,
}

/// Story flags (QueryGameState and SetGameState), the blackboard (AddBlackboardMessage, read by
/// the activate-world routines) and the hero's items.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct GameState {
    pub flags: BTreeSet<u16>,
    /// (message type, eW slot) to value; a read resets it to -1 (absent).
    pub blackboard: HashMap<(u8, usize), i32>,
    pub items: HashMap<u16, u32>,
    /// Rotation words of the proximity routines, by (pack, routine). Module data: a pack's
    /// words return to 0 when it is loaded again (`reload_module`).
    pub rotations: HashMap<(u32, u8), Rotation>,
    /// Other module data words, by (pack, word): latches and the trigger routines' words. Back to
    /// 0 when the pack loads again.
    pub words: HashMap<(u32, u8), i32>,
    /// The shared collected table (SPEC-0048 Script groups, Key; SPEC-0051): rows set to 1.
    pub collected: BTreeSet<u32>,
    /// The current hit points of the current world's entities by entity id (filled by the host
    /// before each Update; GetAI_HitPoints).
    pub entity_hp: HashMap<usize, i32>,
    /// The wallet (GetNeopoints, RemoveNeopoints).
    pub neopoints: i32,
    /// The pack that was current before this one (the "last active world" an activate-world
    /// routine reads).
    pub last_pack: Option<u32>,
}

impl GameState {
    pub fn flag(&self, f: u16) -> bool {
        self.flags.contains(&f)
    }

    pub fn set(&mut self, f: u16, on: bool) {
        if on {
            self.flags.insert(f);
        } else {
            self.flags.remove(&f);
        }
    }

    pub fn has_item(&self, item: u16) -> bool {
        self.items.get(&item).is_some_and(|&n| n > 0)
    }

    /// Read and reset a blackboard message (-1 when none).
    pub fn take_message(&mut self, kind: u8, slot: usize) -> i32 {
        self.blackboard.remove(&(kind, slot)).unwrap_or(-1)
    }

    /// A pack was loaded again: its module data words return to their file values.
    pub fn reload_module(&mut self, pack: u32) {
        self.rotations.retain(|(p, _), _| *p != pack);
        self.words.retain(|(p, _), _| *p != pack);
    }

    fn word(&self, pack: u32, w: u8) -> i32 {
        self.words.get(&(pack, w)).copied().unwrap_or(0)
    }

    fn set_word(&mut self, pack: u32, w: u8, v: i32) {
        self.words.insert((pack, w), v);
    }

    /// GiveItemToHero(item, n).
    pub fn give(&mut self, item: u16, n: u32) {
        *self.items.entry(item).or_insert(0) += n;
    }

    /// RemoveAllOfItemFromHero(item).
    pub fn remove_all(&mut self, item: u16) {
        self.items.remove(&item);
    }

    /// RemoveItemFromHero(item, n).
    pub fn take(&mut self, item: u16, n: u32) {
        if let Some(c) = self.items.get_mut(&item) {
            *c = c.saturating_sub(n);
            if *c == 0 {
                self.items.remove(&item);
            }
        }
    }

    /// The farm story stage: the highest of story flags 1 to 6 that is set (SPEC-0045, 909
    /// activation thread).
    pub fn stage(&self) -> Option<u16> {
        (1..=6).rev().find(|&f| self.flag(f))
    }
}

/// The registered teleport callback of a pack's door: its prompt context (13 "Enter", 26
/// "Exit"; 0 starts on contact), or None when no handler is described.
pub fn teleport_context(pack: u32, door: usize) -> Option<u16> {
    match (pack, door) {
        (909, 0 | 1) => Some(13),
        (951, 0) => Some(26),
        (955, 0) => Some(13),
        // SPEC-0045 1023: every teleport shows context 13 (index 10 shows 3 while flag 35 is
        // clear; taken as 13 here).
        (1023, 0..=10) => Some(13),
        _ => None,
    }
}

/// Run a pack's teleport callback for a door.
pub fn on_teleport(pack: u32, door: usize, state: &mut GameState) -> Vec<Action> {
    match (pack, door) {
        // SPEC-0045 909: routine 8 enters world 72 (pack 951), routine 9 world 73 (pack 955).
        (909, 0) => vec![Action::EnterInterior { slot: 72 }],
        (909, 1) => vec![Action::EnterInterior { slot: 73 }],
        // SPEC-0041 951: with state 119 set and 122 clear, line 126; otherwise message 10 for
        // eW 68 and leave to eW 5 (pack 691).
        (951, 0) => {
            if state.flag(119) && !state.flag(122) {
                vec![Action::Message { line: 126 }]
            } else {
                state.blackboard.insert((2, 68), 10);
                vec![Action::LeaveInterior { slot: 5 }]
            }
        }
        // SPEC-0045 1023 teleports. The exterior replacement loads of indices 0, 9 and 10 are
        // not built; their line branches are.
        (1023, 1 | 5) => {
            if state.flag(10) {
                vec![Action::Message { line: 3 }]
            } else if !state.flag(136) {
                vec![Action::Message { line: 5 }]
            } else {
                vec![Action::EnterInterior { slot: if door == 1 { 118 } else { 117 } }]
            }
        }
        (1023, 2 | 3 | 4 | 6 | 7 | 8) => {
            if !state.flag(136) {
                vec![Action::Message { line: 5 }]
            } else {
                let slot = match door {
                    2 => 125,
                    3 => 116,
                    4 => 113,
                    6 => 111,
                    7 => 112,
                    _ => 115,
                };
                vec![Action::EnterInterior { slot }]
            }
        }
        (1023, 0) if !state.flag(15) && !(state.flag(150) && !state.flag(11)) => vec![Action::Message { line: 23 }],
        (1023, 9) if !state.flag(13) => vec![Action::Message { line: 3 }],
        (1023, 10) if !state.flag(35) => vec![Action::Message { line: 6 }],
        // SPEC-0032 955: message 40 for eW 68, leave to eW 5.
        (955, 0) => {
            state.blackboard.insert((2, 68), 40);
            vec![Action::LeaveInterior { slot: 5 }]
        }
        _ => Vec::new(),
    }
}

/// Run a pack's Init (once per load of the pack): the calls with world effects described so far.
pub fn on_init(pack: u32, state: &mut GameState) -> Vec<Action> {
    let reg = |index, routine| Action::RegisterTrigger { index, routine };
    match pack {
        // SPEC-0045 909 Init: SetWhinnyPenmesh first (inner 0, entrance 5, locator 8); script
        // triggers 0 to 3 routine 10, 5 routine 11, 8 routine 7, and 7 routine 4 while 117 is
        // clear.
        909 => {
            let mut out = vec![Action::WhinnyPen { inner: 0, entrance: 5, locator: 8 }];
            // The object helper sets the word of Update's flag-110 step when 110 is already
            // set, so that step does not run again.
            if state.flag(110) {
                state.set_word(pack, 5, 1);
            }
            out.extend((0..4).map(|i| reg(i, 10)));
            out.push(reg(5, 11));
            out.push(reg(8, 7));
            if !state.flag(117) {
                out.push(reg(7, 4));
            }
            out
        }
        // SPEC-0041 951 Init: LockChest(chest 0, key item 15, message 124); then the thread
        // body, which runs once: the first-entry step (120, movie 2, item 15) and the slot step.
        951 => {
            let mut out = vec![Action::LockChest { id: 0, key: 15, message: 124 }];
            if state.flag(119) && !state.flag(120) {
                state.set(120, true);
                state.give(15, 1);
            }
            // The first case that holds, unless 220 is set: 2 set and 147 clear does nothing.
            if !state.flag(220) && !(state.flag(2) && !state.flag(147)) {
                if state.flag(147) && !state.flag(148) {
                    state.remove_all(1);
                    state.give(0, 1);
                    state.set(148, true);
                } else if !state.flag(147) && !state.flag(2) && !state.flag(123) && (state.flag(121) || state.flag(119)) {
                    // SlotCreate(world, 3, ...) with j 1: region 953.
                    out.push(Action::LoadRegion { slot: 0, region: Some(953) });
                }
            }
            out
        }
        678 | 686 | 691 => road_init(pack, state),
        1023 => village_init(state),
        // SPEC-0050 section 5: the Market Town shops' x4 prices until flag 298, once the
        // shopkeeper's set (1 with flag 86, else 0) has loaded with the stock.
        1005 | 1008 | 1011 | 1014 | 1020 => {
            let mut out = vec![Action::LoadRegion { slot: 0, region: Some(pack + 1 + state.flag(86) as u32) }, Action::WaitLoads];
            if !state.flag(298) {
                out.push(Action::ScalePrices { factor: 4.0 });
            }
            out
        }
        // SPEC-0050 section 4: pack 1140 registers its buy hook only when flag 10 is set.
        1140 => {
            state.set_word(pack, 0, state.flag(10) as i32);
            Vec::new()
        }
        // SPEC-0039 1148 thread 1: set 0 (region 1149) when flag 496 is set.
        1148 if state.flag(496) => vec![Action::LoadRegion { slot: 0, region: Some(1149) }],
        // SPEC-0042 1150 thread T: set 0 (region 1151) while flag 142 is clear.
        1150 if !state.flag(142) => vec![Action::LoadRegion { slot: 0, region: Some(1151) }],
        // SPEC-0039 1133 thread 1: set 1 with flag 10, else set 0.
        1133 => vec![Action::LoadRegion { slot: 0, region: Some(1133 + 1 + state.flag(10) as u32) }],
        _ => Vec::new(),
    }
}

/// SPEC-0045 1023 Init: the trigger registrations (routines: 1 the signs, 2 the square-message
/// routine of 0 to 2, 6 the chase start) and the load thread's first-slot load (slot 0; region
/// 1023 + 1 + j). The zone loaders (27 to 34), the greetings and the prize wheel, the cloak
/// slots and the villager talk are not built.
fn village_init(state: &mut GameState) -> Vec<Action> {
    let reg = |index, routine| Action::RegisterTrigger { index, routine };
    let mut out: Vec<Action> = [6u16, 7, 8, 9, 10, 11, 12, 13, 4, 3, 14, 15, 16].into_iter().map(|i| reg(i, 1)).collect();
    out.extend([reg(1, 2), reg(2, 2), reg(0, 2), reg(17, 6)]);
    if !state.flag(11) {
        if state.flag(10) {
            out.push(Action::LoadRegion { slot: 1, region: Some(1023 + 1 + 6) });
        } else if state.flag(127) && !state.flag(136) {
            // The chase resumes: set 0, the Meerca at the first waypoint whose flag is clear.
            let k = (0..8).find(|&k| !state.flag(128 + k)).unwrap_or(8) as usize;
            out.extend([Action::LoadRegion { slot: 0, region: Some(1024) }, Action::WaitLoads, Action::MoveEntity { entity: 0, locator: 477 + k, face: true }]);
            out.extend(chase(state));
        } else {
            out.push(Action::LoadRegion { slot: 0, region: Some(1023 + 1 + 2) });
        }
    }
    out
}

/// SPEC-0045 1023 chase thread: from the first waypoint k (0 to 7) whose flag 128 + k is clear
/// through the catch at k = 8: the Meerca walks to locator 477 + k (stop 1.1), then loops
/// animation 28 or 48 until the hero is within 30 units; below 8 the waypoint's flag is set and
/// it plays animation 44 and waits 0.2 s; at 8 flag 136, the fade, set 2 and item 32 return
/// (movie 4 and the zone reload are not reproduced).
fn chase(state: &GameState) -> Vec<Action> {
    use Action::*;
    let first = (0..8).find(|&k| !state.flag(128 + k)).unwrap_or(8);
    let mut v = Vec::new();
    for k in first..=8 {
        let locator = 477 + k as usize;
        v.extend([WalkEntity { entity: 0, locator, wait: true, orient: true, stop: 1.1 }, RandomLoop { entity: 0, anims: [28, 48] }]);
        v.push(WaitHeroNear { locator, distance: 30.0 });
        if k < 8 {
            v.extend([SetFlag { flag: 128 + k, on: true }, EntityAnim { entity: 0, anim: 44, looped: false, wait: false }, wait(0.2)]);
        } else {
            v.extend([SetFlag { flag: 136, on: true }, HoldHero, FadeOut { seconds: 1.0, wait: true }, LoadRegion { slot: 0, region: Some(1023 + 1 + 2) }]);
            v.extend([GiveItem { item: 32, count: 1 }, WaitLoads, ReturnCamera { preset: -1, release: true }, FadeIn { seconds: 1.0, wait: true }]);
        }
    }
    v
}

/// SPEC-0045 1023 trigger 17, the chase start (step 1, the cutscene; step 2, the chase). The
/// Meerca's set (set 1) is loaded into slot 1 for the scene and released after it. PlayHeroAnim,
/// the movement types, object 4, the navmesh assignment and the quick fades are not reproduced.
fn chase_start(state: &mut GameState) -> Vec<Action> {
    use Action::*;
    let cam = |track| PlayCamera { track, wait: false, end: 1, speed: 1.0 };
    let cut = |track| [FadeOut { seconds: 0.7, wait: true }, PlayCamera { track, wait: false, end: 1, speed: 1.0 }, FadeIn { seconds: 0.4, wait: false }];
    let anim = |entity, anim, wait| EntityAnim { entity, anim, looped: false, wait };
    let walk = |entity, locator| WalkEntity { entity, locator, wait: false, orient: false, stop: 1.1 };
    state.set(7, true);
    // The thief (entity 0, region 1024 record 0 per SPEC-0058 step 6) and Heermedjet (entity 1,
    // region 1025) for the scene; which set the slot-swap thread loads is R-0028.
    let mut v = vec![HoldHero, FadeOut { seconds: 1.0, wait: true }, LoadRegion { slot: 1, region: Some(1023 + 1 + 1) }, LoadRegion { slot: 2, region: Some(1024) }, WaitLoads];
    v.extend([MoveHero { locator: 488 }, FadeIn { seconds: 1.0, wait: false }, cam(8), WalkHero { locator: 489, wait: false, orient: false }, wait(2.8)]);
    v.extend(cut(9));
    v.extend([MoveHero { locator: 490 }, WalkHero { locator: 491, wait: true, orient: true }]);
    v.extend(cut(10));
    v.extend([wait(2.5), cam(11), anim(1, 41, true), anim(1, 49, true), PlayCamera { track: 12, wait: true, end: 1, speed: 1.0 }, wait(1.0)]);
    v.extend([cam(13), anim(0, 49, true), anim(0, 41, true)]);
    v.extend(cut(14));
    v.extend([MoveHero { locator: 492 }, WalkHero { locator: 493, wait: true, orient: true }, walk(1, 495), wait(1.0), cam(15), wait(0.3), cam(16)]);
    v.extend([MoveEntity { entity: 1, locator: 496, face: true }, PlaySound { cue: 12, locator: None }, MoveHero { locator: 493 }]);
    v.extend(cut(17));
    v.push(wait(1.0));
    v.extend(cut(19));
    v.extend([Message { line: 25 }, cam(20), Converse { entity: 1, line: 26 }]);
    v.extend(cut(18));
    v.extend([MoveEntity { entity: 0, locator: 499, face: true }, wait(0.2), walk(0, 500), wait(0.6), cam(21), Converse { entity: 1, line: 27 }]);
    v.extend([walk(1, 499), wait(1.0), cam(22), MoveEntity { entity: 0, locator: 501, face: true }, walk(0, 502), wait(1.5)]);
    v.extend([FadeOut { seconds: 2.0, wait: true }, LetterBox { on: false, seconds: 0.0, wait: false }, MoveEntity { entity: 0, locator: 477, face: true }]);
    v.extend([EntityAnim { entity: 0, anim: 48, looped: true, wait: false }, ReturnCamera { preset: -1, release: true }, FadeIn { seconds: 1.0, wait: true }]);
    // Step 2: the parcel is taken and the chase starts.
    state.remove_all(32);
    state.set(127, true);
    v.extend(chase(state));
    v
}

/// SPEC-0042 module 678 Init. Routine numbers: 0 trigger T0, 1 the intro S1, 2 the line
/// handler N, 7 trigger T7.
fn road_init(pack: u32, state: &mut GameState) -> Vec<Action> {
    let reg = |index, routine| Action::RegisterTrigger { index, routine };
    let region = |j: u32| Some(pack + 1 + j);
    let mut out = Vec::new();
    let f = |n| state.flag(n);
    if !f(149) && f(148) {
        state.set(149, true);
    } else if !f(149) && !f(148) && !f(146) && !f(143) {
        if f(126) {
            // Thread A: slot load of set 3; proximity P on entity 9.
            out.push(Action::LoadRegion { slot: 0, region: region(3) });
        } else {
            // The first visit: thread B loads set 0 (S1 waits for it) and loops animation 1
            // on entities 1, 2 and 0; triggers S1 (index 1) and T0 (index 0).
            out.extend([Action::LoadRegion { slot: 0, region: region(0) }, Action::WaitLoads, Action::Sleep { ticks: 1 }]);
            for entity in [1, 2, 0] {
                out.push(Action::EntityAnim { entity, anim: 1, looped: true, wait: false });
            }
            out.insert(0, reg(1, 1));
            out.insert(0, reg(0, 0));
        }
    }
    let tail = [reg(2, 2), reg(3, 2), reg(5, 2), reg(6, 2), reg(7, 7)];
    let at = out.iter().take_while(|a| matches!(a, Action::RegisterTrigger { .. })).count();
    out.splice(at..at, tail);
    out
}

/// SPEC-0042 678 S1: the road-fight intro, through the hand-over to the fight. Lines 3, 7 and
/// 8 are spoken with entity 0 as the conversation target; 4, 5 and 6 are plain LevelMsg.
fn road_intro() -> Vec<Action> {
    use Action::*;
    let anim = |entity, anim| EntityAnim { entity, anim, looped: false, wait: false };
    let looped = |entity, anim| EntityAnim { entity, anim, looped: true, wait: false };
    let cam = |track| PlayCamera { track, wait: false, end: 1, speed: 1.0 };
    let walk = |entity, locator, stop, wait| WalkEntity { entity, locator, wait, orient: true, stop };
    let mut v = vec![PlayStinger { id: 4, volume: 1.0 }, HoldHero, FadeOut { seconds: 1.0, wait: true }, WaitLoads];
    // Step 2.
    v.extend([LetterBox { on: true, seconds: 0.0, wait: false }, MoveHero { locator: 19 }, wait(0.5), cam(17), anim(0, 1)]);
    v.extend([FadeIn { seconds: 1.8, wait: false }, SetExcite { level: 80 }, wait(2.3), FadeOut { seconds: 1.0, wait: true }, wait(0.6)]);
    v.extend([cam(4), FadeIn { seconds: 0.5, wait: false }, looped(3, 75), wait(2.3), FadeOut { seconds: 0.8, wait: true }, cam(11)]);
    v.push(FadeIn { seconds: 0.8, wait: false });
    // Step 3: the three walk to points 23, 22 and 21 (stop 0.35); S1 waits for all three.
    v.extend([walk(2, 23, 0.35, false), walk(1, 22, 0.35, false), Sleep { ticks: 15 }, walk(0, 21, 0.35, false), looped(3, 42)]);
    v.push(WaitWalks { entities: [2, 1, 0] });
    // Step 4.
    v.extend([cam(12), MoveEntity { entity: 2, locator: 27, face: true }, anim(2, 14), cam(16), anim(3, 75), cam(5)]);
    v.extend([walk(1, 26, 0.4, true), EntityAnim { entity: 1, anim: 3, looped: false, wait: true }, anim(1, 3), wait(1.0)]);
    v.extend([cam(8), walk(0, 25, 0.4, true), anim(0, 14), wait(1.0)]);
    // Step 5: the conversation.
    v.extend([Converse { entity: 0, line: 3 }, cam(13), Sleep { ticks: 60 }, anim(0, 119), Message { line: 4 }, cam(14), anim(1, 112)]);
    v.extend([Message { line: 5 }, cam(15), cam(16), looped(3, 111), Message { line: 6 }, ReleaseEntity { entity: 3 }, cam(6)]);
    v.extend([Converse { entity: 0, line: 7 }, Sleep { ticks: 80 }, cam(19), anim(2, 115), anim(1, 115), anim(0, 115)]);
    v.extend([Converse { entity: 0, line: 8 }, anim(1, 113), cam(14), cam(18), anim(2, 115), anim(1, 115), anim(0, 115)]);
    // Step 6: the hand-over to the fight.
    v.extend([ReleaseEntity { entity: 3 }, ReleaseEntity { entity: 2 }, ReleaseEntity { entity: 1 }, ReleaseEntity { entity: 0 }]);
    v.extend([FadeOut { seconds: 0.7, wait: true }, LetterBox { on: false, seconds: 0.0, wait: false }, MoveHero { locator: 3 }]);
    v.extend([ReturnCamera { preset: -1, release: true }, FadeIn { seconds: 0.7, wait: false }, Message { line: 22 }]);
    v.extend([Agro { entity: 1 }, Agro { entity: 2 }, PlayStinger { id: 4, volume: 1.0 }]);
    v
}

/// SPEC-0042 678 end scene, run by the fight watch once the raiders have lost 31 hit points.
fn road_end(pack: u32) -> Vec<Action> {
    use Action::*;
    let anim = |entity, anim| EntityAnim { entity, anim, looped: false, wait: false };
    let cam = |track| PlayCamera { track, wait: false, end: 1, speed: 1.0 };
    let walk = |entity, locator, wait| WalkEntity { entity, locator, wait, orient: false, stop: 0.0 };
    let mut v = vec![HoldHero, FadeOut { seconds: FADE, wait: true }];
    v.extend([MoveEntity { entity: 0, locator: 6, face: true }, MoveEntity { entity: 1, locator: 11, face: true }, MoveEntity { entity: 2, locator: 13, face: true }]);
    v.extend([Sleep { ticks: 30 }, MoveHero { locator: 7 }, LetterBox { on: true, seconds: 0.0, wait: false }, cam(20)]);
    v.extend([anim(1, 0), anim(2, 0), Sleep { ticks: 1 }, walk(1, 12, false), walk(2, 14, false), WalkHero { locator: 9, wait: false, orient: false }]);
    v.extend([FadeIn { seconds: FADE, wait: false }, Sleep { ticks: 30 }, anim(0, 3), Sleep { ticks: 30 }, cam(21), anim(0, 17)]);
    v.extend([Converse { entity: 0, line: 10 }, anim(0, 117), cam(22), Sleep { ticks: 20 }, walk(0, 4, true), FadeOut { seconds: FADE, wait: true }]);
    v.push(DeactivateEntity { entity: 0 });
    // Movie 3 is not played. Slot reset and the post-fight set 3; entity 10 waits.
    v.extend([LoadRegion { slot: 0, region: Some(pack + 1 + 3) }, WaitLoads, DeactivateEntity { entity: 10 }]);
    v.extend([FadeIn { seconds: FADE, wait: false }, MoveHero { locator: 45 }, cam(38), WalkHero { locator: 46, wait: true, orient: false }]);
    v.extend([Sleep { ticks: 60 }, Converse { entity: 9, line: 1 }, FadeOut { seconds: FADE, wait: true }, FadeIn { seconds: FADE, wait: true }]);
    v.extend([DeactivateEntity { entity: 9 }, ReturnCamera { preset: -1, release: true }, Sleep { ticks: 45 }, LetterBox { on: false, seconds: 0.0, wait: false }]);
    v.extend([wait(1.0), Message { line: 2 }, ActivateEntity { entity: 10 }, PlayStinger { id: 4, volume: 1.0 }]);
    v.extend([HitPointsPercent { entity: 10, percent: 25.0 }, EntityAnim { entity: 10, anim: 10, looped: false, wait: true }, Agro { entity: 10 }]);
    v
}

/// Run a registered script trigger routine of a pack for one fire of its record.
pub fn on_trigger(pack: u32, routine: u8, call: TriggerCall, state: &mut GameState) -> TriggerOut {
    let mut out = TriggerOut::default();
    let first = call.state == 1 && call.object;
    match (pack, routine) {
        // SPEC-0045 909 trigger routine 10 (indices 0 to 3): words A (0) and B (1), the line-192
        // once-word (2). Cameras 5 and 6 are not played.
        (909, 10) if first => {
            match call.index {
                0 => state.set_word(pack, 0, 1),
                1 => state.set_word(pack, 1, 1),
                _ => {
                    state.set_word(pack, 0, 0);
                    state.set_word(pack, 1, 0);
                }
            }
            if state.word(pack, 0) != 0 && state.word(pack, 1) != 0 {
                if call.index == 0 {
                    if state.word(pack, 2) == 0 && !state.flag(220) {
                        out.actions.push(Action::Message { line: 192 });
                        state.set_word(pack, 2, 1);
                    }
                    state.set_word(pack, 1, 0);
                } else if call.index == 1 {
                    state.set_word(pack, 0, 0);
                }
            }
        }
        // SPEC-0045 909 trigger routine 11 (index 5), the pump: word 3 is set by the exchange
        // (and cleared by the flag-117 cutscene). The objects, effects, camera 27, fades and
        // sounds of the exchange are not reproduced.
        (909, 11) => {
            if !call.square {
                out.prompt = Some(4);
            } else if state.word(pack, 3) != 0 {
                out.actions.push(Action::Message { line: 167 });
            } else if state.flag(111) && !state.flag(112) && state.has_item(6) {
                // Fade out, hero to locator 29, camera 27, fade in, sounds 9 and 10, lines 166
                // and 162, camera returned (objects 9, 17 to 20, effects 3 and 4 and the shake
                // are not reproduced).
                out.actions.extend([Action::HoldHero, fade_out(), Action::MoveHero { locator: 29 }, camera(27), fade_in()]);
                out.actions.push(Action::PlaySound { cue: 9, locator: None });
                out.actions.push(Action::PlaySound { cue: 10, locator: None });
                state.remove_all(6);
                out.actions.push(Action::Message { line: 166 });
                state.give(7, 1);
                out.actions.push(Action::Message { line: 162 });
                out.actions.push(release_camera());
                state.set(112, true);
                state.set_word(pack, 3, 1);
            } else {
                // The spec names lines 163 to 165 by flags 111 and 112 and items 6 and 7 without
                // the mapping (R-0024); provisional: 163 before the task, 164 without the bucket,
                // 165 once the water is drawn.
                let line = if !state.flag(111) { 163 } else if !state.flag(112) { 164 } else { 165 };
                out.actions.push(Action::Message { line });
            }
        }
        // SPEC-0045 909 routine 4 (index 7): line 191 (the push and ladder hint).
        (909, 4) if first => out.actions.push(Action::Message { line: 191 }),
        // SPEC-0045 909 routine 7 (index 8): enter world 71 (pack 950).
        (909, 7) if first => out.actions.push(Action::EnterInterior { slot: 71 }),
        // SPEC-0045 1023 signs: context 3 while inside; on the press the index's line.
        (1023, 1) if call.object && call.state != 0 => {
            if call.square {
                let line = match call.index {
                    3 => 9,
                    4 => 10,
                    6 => 14,
                    7 => 7,
                    8 => 8,
                    9 => 12,
                    10 => 13,
                    11 => 11,
                    12 => 15,
                    13 => 16,
                    14 => 17,
                    15 => 19,
                    _ => 18,
                };
                out.actions.push(Action::Message { line });
            } else {
                out.prompt = Some(3);
            }
        }
        // Triggers 0 to 2: with 64 set and 248 clear, line 22 (BlockPlayerControl around it).
        (1023, 2) if first && state.flag(64) && !state.flag(248) => out.actions.push(Action::Message { line: 22 }),
        // Trigger 17: the chase start, while flag 127 is clear.
        (1023, 6) if first && !state.flag(127) => out.actions = chase_start(state),
        // SPEC-0042 678 T0 (index 0): line 23 once per module load (word 0).
        (678 | 686 | 691, 0) if first && state.word(pack, 0) == 0 => {
            state.set_word(pack, 0, 1);
            out.actions.push(Action::Message { line: 23 });
        }
        // S1 (index 1): the intro, once (flag 124 is the guard); it ends by setting 124 and
        // starting the fight (the fight watch runs in Update).
        (678 | 686 | 691, 1) if first && !state.flag(124) => {
            out.actions = road_intro();
            state.set(124, true);
        }
        // N (indices 2, 3, 5 and 6): after the fight has started (word 1), each index's line
        // once per load (words 2 to 6): 9 for 2 and 3, 20 for 5 and 6.
        (678 | 686 | 691, 2) if first && state.word(pack, 1) != 0 && state.word(pack, 2 + call.index as u8) == 0 => {
            state.set_word(pack, 2 + call.index as u8, 1);
            out.actions.push(Action::Message { line: if call.index <= 3 { 9 } else { 20 } });
        }
        // T7 (index 7): the Square context 3; on the press, line 21.
        (678 | 686 | 691, 7) if call.object && call.state != 0 => {
            if call.square {
                out.actions.push(Action::Message { line: 21 });
            } else {
                out.prompt = Some(3);
            }
        }
        _ => {}
    }
    out
}

/// Run a pack's Update for one tick of the current world. `dialog` is whether a dialogue box
/// is up (the "wait until no dialog is active" steps hold until it closes).
pub fn on_update(pack: u32, state: &mut GameState, dialog: bool) -> Vec<Action> {
    let mut out = Vec::new();
    match pack {
        // SPEC-0045 909 Update: the flag and item steps and the scenes.
        909 => {
            // The Act 1 start: 707 on and 1 off sets 1 and clears 706, fades out at once and
            // moves entity 31 to locator 69 (movies 88 and 1 and the act-start call are not
            // reproduced; camera 30 is left out: no return of it is described, R-0025).
            if state.flag(707) && !state.flag(1) {
                state.set(1, true);
                state.set(706, false);
                out.extend([Action::FadeOut { seconds: 0.0, wait: false }, Action::MoveEntity { entity: 31, locator: 69, face: true }]);
            }
            // The talk thread (while 117 and 123 are clear): each round-up flag, the first time
            // it is seen set in this visit (words 10 to 13, cleared at each activation), plays a
            // talk with entity 31 (its cameras 30 and 31 are left out, R-0025).
            if !state.flag(117) && !state.flag(123) {
                for (k, f) in (107u16..=110).enumerate() {
                    let w = 10 + k as u8;
                    if state.flag(f) && state.word(pack, w) == 0 && !dialog {
                        state.set_word(pack, w, 1);
                        let talk = |line| Action::Converse { entity: 31, line };
                        match f {
                            107 => out.extend([talk(181), talk(182)]),
                            108 => out.extend([Action::MoveEntity { entity: 31, locator: 69, face: true }, talk(183)]),
                            109 => out.extend([Action::MoveEntity { entity: 31, locator: 69, face: true }, talk(184)]),
                            _ => out.extend([Action::Sleep { ticks: 60 }, talk(185), talk(186), talk(187), talk(188)]),
                        }
                    }
                }
            }
            // Flag 110 with its word clear (Init sets the word when 110 was set at load): hero
            // to locator 7, camera 7, sound 5 (objects 2, 3, 10 and 11 are not reproduced). The
            // keyed camera 7 is played with end mode 2, which frees the hero at its end (R-0025).
            if state.flag(110) && state.word(pack, 5) == 0 {
                state.set_word(pack, 5, 1);
                out.extend([Action::HoldHero, Action::MoveHero { locator: 7 }, Action::PlayCamera { track: 7, wait: false, end: 2, speed: 1.0 }]);
                out.push(Action::PlaySound { cue: 5, locator: None });
            }
            if !state.flag(111) && state.has_item(6) && state.flag(110) {
                state.set(111, true);
            }
            if !state.flag(618) && state.flag(116) && state.has_item(34) {
                state.set(618, true);
            }
            // Flag 114 (Hubert sends Tor for the pliers): fade out, slot R with Hubert's region
            // 927, entity 44 to locator 15, fade in, camera 12, entity 44 walks to locator 16,
            // fade out, camera returned, fade in. The walk waits (R-0025).
            if state.flag(114) && !state.flag(115) && !dialog {
                state.set(115, true);
                out.extend([Action::HoldHero, fade_out(), Action::LoadRegion { slot: 3, region: Some(927) }, Action::WaitLoads]);
                out.extend([Action::MoveEntity { entity: 44, locator: 15, face: true }, fade_in(), camera(12)]);
                out.extend([Action::WalkEntity { entity: 44, locator: 16, wait: true, orient: false, stop: 0.0 }, fade_out(), release_camera(), fade_in()]);
            }
            // Flag 117 (the pliers handed over): the first scene (hero to locator 17, camera 12,
            // entity 44 to locator 16, sound 12, back to locator 15, slot R with region 926),
            // then region 928 for the second (119 set, entity 45 to locator 21, camera 15, its
            // walk to locator 23), then slot R emptied and the camera returned. The pump word is
            // cleared by this cutscene.
            if state.flag(117) && !state.flag(118) && !dialog {
                state.set(118, true);
                state.set_word(pack, 3, 0);
                out.extend([Action::HoldHero, fade_out(), Action::MoveHero { locator: 17 }, camera(12), fade_in()]);
                out.push(Action::WalkEntity { entity: 44, locator: 16, wait: true, orient: false, stop: 0.0 });
                out.push(Action::PlaySound { cue: 12, locator: None });
                out.extend([Action::WalkEntity { entity: 44, locator: 15, wait: true, orient: false, stop: 0.0 }, fade_out()]);
                out.extend([Action::LoadRegion { slot: 3, region: Some(926) }, Action::LoadRegion { slot: 3, region: Some(928) }, Action::WaitLoads]);
                state.set(119, true);
                out.extend([Action::MoveEntity { entity: 45, locator: 21, face: true }, camera(15), fade_in()]);
                out.extend([Action::WalkEntity { entity: 45, locator: 23, wait: true, orient: false, stop: 0.0 }, fade_out()]);
                out.extend([Action::LoadRegion { slot: 3, region: None }, release_camera(), fade_in()]);
            }
            if state.flag(278) {
                state.set(279, true);
            }
        }
        // SPEC-0042 678 fight watch: with 124 set by the intro it marks the fight started
        // (word 1), stores the start sum of entities 0, 2 and 1's hit points (word 8), then
        // waits until they have lost 31 or more in total; the end scene follows and 125, 124
        // and 126 are set; item 170 x100 is Kell's gift in the end scene.
        678 | 686 | 691 => {
            let sum: Option<i32> = [0, 2, 1].iter().map(|e| state.entity_hp.get(e).copied()).sum();
            if state.flag(124) && !state.flag(125) && state.word(pack, 1) == 0 {
                if let Some(sum) = sum {
                    state.set_word(pack, 1, 1);
                    state.set_word(pack, 8, sum);
                }
            } else if state.word(pack, 1) == 1 && !state.flag(125) {
                let current = sum.unwrap_or(0);
                if state.word(pack, 8) - current >= 31 {
                    state.set_word(pack, 1, 2);
                    out = road_end(pack);
                    state.set(124, true);
                    state.set(126, true);
                    state.give(170, 100);
                    state.set(125, true);
                }
            }
        }
        _ => {}
    }
    out
}

/// What a lock chest press does after its gate (SPEC-0048 Lock chests, press rule step 4).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ChestPress {
    /// The key item is missing: show the lock message.
    Locked { line: u16 },
    /// The chest opens (the key item, if any, was removed).
    Open,
}

/// The lock step for group `id` against the world's lock table of (id, key item, message).
pub fn chest_press(locks: &[(i32, i32, i32)], id: i32, state: &mut GameState) -> ChestPress {
    match locks.iter().find(|l| l.0 == id) {
        Some(&(_, key, message)) if key > 0 => {
            if state.has_item(key as u16) {
                state.remove_all(key as u16);
                ChestPress::Open
            } else {
                ChestPress::Locked { line: message.max(0) as u16 }
            }
        }
        _ => ChestPress::Open,
    }
}

/// Run a pack's activate-world routine when its world becomes current.
pub fn on_activate(pack: u32, state: &mut GameState) -> Vec<Action> {
    match pack {
        // SPEC-0045 909 activate-world routine: an unfinished round-up (flags 107 to 110 not
        // all set) is cleared; the type-2 message for eW 68 places the hero.
        909 => {
            if !(107..=110).all(|f| state.flag(f)) {
                for f in 107..=110 {
                    state.set(f, false);
                }
            }
            let message = state.take_message(2, 68);
            let locator = match message {
                10 => Some(2),
                30 => Some(1),
                40 => Some(3),
                50 if state.stage() == Some(6) && !state.has_item(36) => Some(28),
                50 => Some(4),
                _ => None,
            };
            let mut actions: Vec<Action> = locator.map(|locator| Action::MoveHero { locator }).into_iter().collect();
            actions.extend(farm_slots(state, message == 10));
            // The talk thread's round-up flags: one already set when the world becomes current
            // counts as seen, so only a flag that rises during the visit plays its talk (the
            // spec's "first time it is seen set in this visit"; R-0025).
            for k in 0..4u8 {
                let seen = state.flag(107 + k as u16);
                state.set_word(pack, 10 + k, seen as i32);
            }
            // The thread's arrival step: with 706 clear, set it, wait until no load is active
            // and fade in over 1 s.
            if !state.flag(706) {
                state.set(706, true);
                actions.extend([Action::WaitLoads, Action::FadeIn { seconds: 1.0, wait: false }]);
            }
            // Its stage events (movie 5 and the stage-6 scene are not covered).
            match state.stage() {
                // Stage 1 with 122 set and 123 clear: hero to locator 13, camera 10, entity 47
                // says line 168 and entity 39 line 169, entities 47, 39 and 33 to locators 10,
                // 11 and 9, camera 11, the hero walks to locator 14, a fade, camera returned
                // (object 8 and the entity animations are not reproduced; the fade is taken as
                // out then in, R-0025).
                Some(1) if state.flag(122) && !state.flag(123) => {
                    state.set(123, true);
                    actions.extend([Action::WaitLoads, Action::HoldHero, Action::MoveHero { locator: 13 }, camera(10)]);
                    actions.extend([Action::Converse { entity: 47, line: 168 }, Action::Converse { entity: 39, line: 169 }]);
                    for (entity, locator) in [(47, 10), (39, 11), (33, 9)] {
                        actions.push(Action::MoveEntity { entity, locator, face: true });
                    }
                    actions.extend([camera(11), Action::WalkHero { locator: 14, wait: true, orient: false }, fade_out(), release_camera(), fade_in()]);
                }
                Some(2) if !state.flag(147) && !state.flag(145) => state.set(145, true),
                _ => {}
            }
            actions
        }
        1023 => village_return(state).into_iter().collect(),
        _ => Vec::new(),
    }
}

/// SPEC-0045 1023 activate-world routine: the hero goes back to the return locator of the
/// world it came from (eW 106 pack 1098 to 16, 109/1129 to 17, 111/1131 to 10, 112/1133 to 9,
/// 113/1136 to 7, 114/1139 to 12, 115/1140 to 15, 116/1143 to 13, 117/1148 to 11, 118/1150
/// to 8, 125/1172 to 14). The arena return (value 10 for eW 90) is not built.
fn village_return(state: &GameState) -> Option<Action> {
    let locator = match state.last_pack? {
        1098 => 16,
        1129 => 17,
        1131 => 10,
        1133 => 9,
        1136 => 7,
        1139 => 12,
        1140 => 15,
        1143 => 13,
        1148 => 11,
        1150 => 8,
        1172 => 14,
        _ => return None,
    };
    Some(Action::MoveHero { locator })
}

/// The 1023 villager routines: (entities, template, first line b). Template 0 is A, 1 B, 2 C;
/// 3 is entity 74's lines 0 then 1, 2; 4 is the wheel host's 1023 to 1025.
const VILLAGERS: [(&[usize], u8, u16); 34] = [
    (&[69], 0, 84), (&[70], 0, 107), (&[71], 0, 246), (&[5], 0, 362), (&[6], 0, 385), (&[4], 0, 408),
    (&[7], 0, 431), (&[57], 0, 454), (&[83], 0, 477), (&[82], 0, 500), (&[58], 0, 523), (&[63], 0, 546),
    (&[76, 78], 0, 650), (&[68], 0, 702), (&[77], 0, 725), (&[87], 0, 748), (&[79], 0, 858),
    (&[80], 1, 130), (&[86], 1, 159), (&[81], 1, 188), (&[64], 1, 217), (&[60, 61], 1, 569), (&[89, 93], 1, 598),
    (&[62, 95], 1, 673), (&[85, 92], 1, 800), (&[67], 1, 829), (&[84, 88, 91], 1, 929), (&[73], 1, 958), (&[59, 96], 1, 987),
    (&[3], 2, 310), (&[2], 2, 336), (&[74], 3, 0), (&[65], 4, 1023), (&[], 0, 0),
];

fn villager(e: usize) -> Option<usize> {
    VILLAGERS.iter().position(|(es, _, _)| es.contains(&e))
}

/// The story chains of the templates (the arena talk needs two or more arena wins and the
/// wheel talk flag 480; neither can hold yet, so their draws are not taken: R-0029).
fn village_talk(k: u8, s: &mut GameState, rot: &mut Rotation) -> Option<u16> {
    let (_, template, b) = *VILLAGERS.get(k as usize)?;
    let three = |o: u16| cycle(&[b + o, b + o + 1, b + o + 2]);
    match template {
        3 => rot.pick(0, &Lines::FirstOnce(0, vec![1, 2])),
        4 => rot.pick(0, &cycle(&[1023, 1024, 1025])),
        2 if s.flag(138) && !s.flag(141) => {
            let line = rot.pick(10, &three(14));
            if line == Some(b + 14) && !s.flag(644) {
                s.set(644, true);
            }
            line
        }
        2 => cascade(s, rot, &[(202, three(23)), (193, three(20)), (150, three(17)), (7, three(11))]).flatten(),
        1 => cascade(s, rot, &[(301, three(26)), (216, three(23)), (202, three(20)), (193, three(17)), (150, three(14)), (7, three(11))]).flatten(),
        _ => cascade(s, rot, &[(202, three(20)), (193, three(17)), (150, three(14)), (7, three(11))]).flatten(),
    }
}

/// SPEC-0045 1023 entity 90: the story lines, the 136-clear line and the one prompt by flags
/// 485 to 488 (its answer is stored in word 20 and read by nothing else).
fn village_90(s: &mut GameState, rot: &mut Rotation) -> Vec<Action> {
    let talk = |line: Option<u16>| line.map(|line| vec![Action::Talk { line }]).unwrap_or_default();
    if let Some(l) = cascade(s, rot, &[(301, cycle(&[269, 270, 271])), (216, cycle(&[272, 273, 274]))]) {
        return talk(l);
    }
    if !s.flag(136) {
        return talk(Some(287));
    }
    match s.word(1023, 20) {
        0 if s.flag(488) => talk(Some(288)),
        0 => {
            let line = if s.flag(487) { 289 } else if s.flag(486) { 292 } else if s.flag(485) { 295 } else { 298 };
            vec![Action::Prompt { line, routine: 90 }]
        }
        1 => {
            s.set_word(1023, 20, 2);
            Vec::new()
        }
        _ => talk(cascade(s, rot, &[(202, cycle(&[275, 276, 277])), (193, cycle(&[278, 279, 280])), (150, cycle(&[281, 282, 283]))]).flatten().or_else(|| rot.pick(9, &cycle(&[284, 285, 286])))),
    }
}

/// SPEC-0039 1148 handlers 2 to 5 (routines 0 to 3).
fn recruitment_talk(r: u8, s: &mut GameState, rot: &mut Rotation) -> Option<Vec<Action>> {
    let talk = |line: Option<u16>| line.map(|line| vec![Action::Talk { line }]);
    match r {
        3 => talk(rot.pick(0, &cycle(&[0, 1, 2]))),
        0 => talk(cascade(s, rot, &[(202, cycle(&[49, 50, 51])), (150, cycle(&[55, 56, 57]))]).flatten().or(Some(if s.flag(139) { 58 } else { 59 }))),
        1 => {
            if let Some(l) = cascade(s, rot, &[(202, cycle(&[8, 9, 10])), (151, cycle(&[12, 13, 14])), (150, cycle(&[15, 16, 17]))]) {
                return talk(l);
            }
            if s.flag(143) {
                if s.has_item(0) {
                    for f in [150, 142, 3, 8] {
                        s.set(f, true);
                    }
                    s.take(0, 1);
                    s.give(120, 1);
                    return talk(Some(18));
                }
                return talk(rot.pick(3, &cycle(&[19, 20, 21])));
            }
            if s.flag(142) {
                if s.has_item(38) {
                    for f in [143, 17, 2] {
                        s.set(f, true);
                    }
                    s.remove_all(38);
                    return talk(Some(22));
                }
                return talk(rot.pick(4, &cycle(&[23, 24, 25])));
            }
            if s.flag(138) {
                return talk(rot.pick(5, &cycle(&[26, 27, 28])));
            }
            if s.flag(137) {
                if s.has_item(106) {
                    s.set(138, true);
                    s.give(39, 1);
                    return talk(Some(29));
                }
                return talk(rot.pick(6, &cycle(&[30, 31, 32])));
            }
            s.remove_all(32);
            s.set(137, true);
            if s.has_item(106) {
                s.set(138, true);
                s.give(39, 1);
                return talk(Some(34));
            }
            talk(Some(33))
        }
        2 => {
            if s.flag(140) {
                return talk(rot.pick(0, &cycle(&[35, 36, 37])));
            }
            if !s.flag(139) {
                return talk(rot.pick(1, &cycle(&[38, 39, 40])));
            }
            let line = if s.word(1148, 0) == 0 { 41 } else { 42 };
            s.set_word(1148, 0, 1);
            Some(vec![Action::Talk { line }, Action::Prompt { line: 43, routine: 0 }])
        }
        _ => None,
    }
}

/// SPEC-0039 1148 walk-back sequence, after the shop's line, when 140 is set, 143 clear and
/// the hero holds neither item 98 nor 99.
fn walk_back(s: &GameState) -> Vec<Action> {
    use Action::*;
    if !(s.flag(140) && !s.flag(143) && !s.has_item(98) && !s.has_item(99)) {
        return Vec::new();
    }
    let walk = |entity, locator, wait| WalkEntity { entity, locator, wait, orient: true, stop: 1.1 };
    vec![
        WaitDialog,
        Message { line: 60 },
        WalkHero { locator: 4, wait: false, orient: true },
        walk(3, 3, false),
        walk(0, 6, true),
        Message { line: 61 },
        walk(0, 5, true),
        Message { line: 62 },
        GiveWordItem { word: 1, default: 99 },
        walk(0, 7, false),
    ]
}

/// SPEC-0050 section 4: the buy hooks. Each shows its line on a buy of its item while its flag
/// is clear; every hook passes handled 0, so the purchase then completes.
pub fn on_buy(pack: u32, item: u16, s: &GameState) -> Vec<Action> {
    let line = match (pack, item) {
        (856, 113) if !s.flag(220) => 19,
        (859, 130) if !s.flag(220) => 34,
        (1140, 116) if !s.flag(220) && s.word(1140, 0) == 1 => 0,
        (1429, 109) if !s.flag(248) => 20,
        _ => return Vec::new(),
    };
    vec![Action::Message { line }]
}

/// The answer to a LevelPrompt made by a routine of a pack (yes = Circle).
pub fn on_answer(pack: u32, routine: u8, yes: bool, s: &mut GameState) -> Vec<Action> {
    let line = |line| Action::Message { line };
    match (pack, routine, yes) {
        // 1148 prompt 43: buy item 42 for 100 Neopoints.
        (1148, 0, true) if s.neopoints >= 100 => {
            s.neopoints -= 100;
            s.give(42, 1);
            s.set(140, true);
            s.set(644, true);
            let mut v = vec![line(44)];
            v.extend(walk_back(s));
            v
        }
        (1148, 0, true) => vec![line(45)],
        (1148, 0, false) => vec![Action::Prompt { line: 46, routine: 1 }],
        // Prompt 46: trade the sword (98, else 99) for item 42 (the quirk: with neither held it
        // still gives item 42).
        (1148, 1, true) => {
            for item in [98u16, 99] {
                if s.has_item(item) {
                    s.remove_all(item);
                    s.set_word(1148, 1, item as i32);
                    break;
                }
            }
            s.give(42, 1);
            s.set(140, true);
            s.set(644, true);
            let mut v = vec![line(47)];
            v.extend(walk_back(s));
            v
        }
        (1148, 1, false) => vec![line(48)],
        // 1023 entity 90: the answer lines by the flag that chose the prompt; P becomes 1 or 2.
        (1023, 90, _) => {
            s.set_word(1023, 20, if yes { 1 } else { 2 });
            let base = if s.flag(487) { 289 } else if s.flag(486) { 292 } else if s.flag(485) { 295 } else { 298 };
            vec![line(base + if yes { 1 } else { 2 })]
        }
        _ => Vec::new(),
    }
}

/// SPEC-0042 1150 P0 to P4 (routines 0 to 4).
fn barracks_talk(r: u8, s: &mut GameState, rot: &mut Rotation) -> Option<u16> {
    if r == 4 {
        // Master Torak: line 4 gives item 38 by its dialogue command.
        if s.flag(142) {
            return rot.pick(0, &cycle(&[5, 6, 7]));
        }
        if s.flag(139) {
            if s.has_item(60) {
                s.remove_all(60);
                s.remove_all(39);
                s.set(142, true);
                return Some(4);
            }
            return rot.pick(1, &cycle(&[1, 2, 3]));
        }
        if s.flag(138) {
            s.set(139, true);
            return Some(0);
        }
        return Some(8);
    }
    // P0 to P3: step 4 of the shared structure (the arena and wheel branches need flags 381 to
    // 385 and 480; their draws are not taken: R-0029).
    let b = [9u16, 38, 67, 96][r as usize];
    let three = |o: u16| cycle(&[b + o, b + o + 1, b + o + 2]);
    cascade(s, rot, &[(202, three(26)), (193, three(23)), (150, three(20)), (142, three(17)), (138, three(14))]).flatten().or_else(|| rot.pick(9, &three(11)))
}

/// SPEC-0039 1133 handler 3, with the trade of items 40 to 43 for item 60.
fn smithy_talk(s: &mut GameState, rot: &mut Rotation) -> Option<u16> {
    let three = |a: u16| cycle(&[a, a + 1, a + 2]);
    if let Some(l) = cascade(s, rot, &[(301, three(19)), (216, three(22)), (202, three(25)), (150, three(28)), (141, three(31))]) {
        return l;
    }
    if s.flag(138) {
        let held = [40u16, 41, 42, 43].iter().filter(|&&i| s.has_item(i)).count();
        if held == 4 {
            for i in [41, 42, 43, 40] {
                s.remove_all(i);
            }
            s.give(60, 1);
            s.set(141, true);
            return Some(34);
        }
        return Some([38, 37, 36, 35][held]);
    }
    rot.pick(9, &three(39))
}

/// A line choice of a proximity routine (SPEC-0022 Line rotation patterns). Lines given as
/// (Tor, Roberta) pairs pick by the active hero; the hero is Tor here.
#[derive(Clone, Debug, PartialEq)]
pub enum Lines {
    One(u16),
    /// "a, b, c": in turn, then again from a.
    Cycle(Vec<u16>),
    /// "a; then b, c": a once per reset, then b, c in turn.
    FirstOnce(u16, Vec<u16>),
    /// Lines shown once in order, then the cycle (909 routine A's flag-111 branch).
    Then(Vec<u16>, Vec<u16>),
}

/// The rotation words of one routine (module data: back to 0 when the pack loads again).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Rotation {
    marker: u16,
    counter: u16,
}

impl Rotation {
    /// Reset check, show, step for branch k (marker k + 1, so the file value 0 means none).
    fn pick(&mut self, k: u16, lines: &Lines) -> Option<u16> {
        if self.marker != k + 1 {
            self.marker = k + 1;
            self.counter = 0;
        }
        let c = self.counter as usize;
        let (line, next) = match lines {
            Lines::One(a) => (*a, 0),
            Lines::Cycle(v) => (v[c % v.len()], ((c + 1) % v.len()) as u16),
            Lines::FirstOnce(a, v) => {
                if c == 0 {
                    (*a, 1)
                } else {
                    let i = (c - 1) % v.len();
                    (v[i], (1 + (i + 1) % v.len()) as u16)
                }
            }
            Lines::Then(first, v) => {
                if c < first.len() {
                    (first[c], (c + 1) as u16)
                } else {
                    let i = (c - first.len()) % v.len();
                    (v[i], (first.len() + (i + 1) % v.len()) as u16)
                }
            }
        };
        self.counter = next;
        Some(line)
    }
}

/// The proximity routine registered for an entity id of a pack (RegisterEntityProximityCallback
/// with the range flag set), as a routine number local to that pack.
pub fn proximity_routine(pack: u32, entity: usize) -> Option<u8> {
    match (pack, entity) {
        // SPEC-0045 909: entities 37 to 42 routine A, 43 to 50 routine B, 31 to 36 routine C.
        (909, 37..=42) => Some(0),
        (909, 43..=50) => Some(1),
        (909, 31..=36) => Some(2),
        // SPEC-0041 951: callback 1 at 0, 3 and 7; callback 2 at 1, 4 and 8; 3 at 2, 5 and 6.
        (951, 0 | 3 | 7) => Some(0),
        (951, 1 | 4 | 8) => Some(1),
        (951, 2 | 5 | 6) => Some(2),
        // SPEC-0042 678 proximity P on entity 9 (registered with flag 126 set).
        (678 | 686 | 691, 9) => Some(0),
        // SPEC-0039 1148: entity 0 handler 2, 1 handler 3, 3 handler 4, 2 handler 5.
        (1148, 0..=3) => Some(entity as u8),
        // SPEC-0042 1150: P0 to P4 on entities 0 to 4.
        (1150, 0..=4) => Some(entity as u8),
        // SPEC-0039 1133: entity 1 with flag 10 set, else entity 0.
        (1133, 0 | 1) => Some(0),
        (1023, 90) => Some(90),
        // SPEC-0045 1023 villagers (routine numbers: the index in VILLAGERS, 100 + k).
        (1023, e) => villager(e).map(|k| 100 + k as u8),
        _ => None,
    }
}

/// One cycle branch: (flag, lines).
fn cycle(lines: &[u16]) -> Lines {
    Lines::Cycle(lines.to_vec())
}

/// Run a proximity routine: the first branch whose test holds picks a line (with its side
/// effects); the talk frame (SetConversationTarget, LevelMsg, EndConversation) wraps it.
pub fn on_talk(pack: u32, routine: u8, state: &mut GameState) -> Vec<Action> {
    let key = (pack, routine);
    let mut rot = state.rotations.get(&key).copied().unwrap_or_default();
    let line = match (pack, routine) {
        (909, 0) => farm_routine_a(state, &mut rot),
        (909, 1) => farm_routine_b(state, &mut rot),
        (909, 2) => farm_routine_c(state, &mut rot),
        (951, r) => return house_routine(r, state),
        (1148, r) => {
            let a = recruitment_talk(r, state, &mut rot).unwrap_or_default();
            state.rotations.insert(key, rot);
            return a;
        }
        (1150, r) => barracks_talk(r, state, &mut rot),
        (1133, _) => smithy_talk(state, &mut rot),
        (1023, r) if r >= 100 => village_talk(r - 100, state, &mut rot),
        (1023, 90) => {
            let a = village_90(state, &mut rot);
            state.rotations.insert(key, rot);
            return a;
        }
        (678 | 686 | 691, 0) => return vec![Action::Message { line: 0 }],
        _ => None,
    };
    state.rotations.insert(key, rot);
    line.map(|line| vec![Action::Talk { line }]).unwrap_or_default()
}

/// SPEC-0041 951 proximity callbacks 1 to 3 (routines 0 to 2). With 147 clear they show their
/// line with LevelMsg alone, outside the conversation frame.
fn house_routine(routine: u8, s: &mut GameState) -> Vec<Action> {
    let talk = |line| vec![Action::Talk { line }];
    let msg = |line| vec![Action::Message { line }];
    match routine {
        0 if s.flag(147) => talk(0),
        0 => msg(4),
        1 if s.flag(147) => talk(119),
        1 => msg(123),
        2 if s.flag(147) => talk(55),
        2 if !s.flag(122) && s.has_item(98) => {
            s.give(32, 1);
            s.set(121, true);
            s.set(122, true);
            msg(59)
        }
        2 if !s.flag(122) => msg(60),
        2 => msg(58),
        _ => Vec::new(),
    }
}

/// The (flag, lines) cascade shared by the farm routines' story branches: the first set flag
/// in the list picks its lines.
fn cascade(state: &GameState, rot: &mut Rotation, list: &[(u16, Lines)]) -> Option<Option<u16>> {
    let k = list.iter().position(|(f, _)| state.flag(*f))?;
    Some(rot.pick(k as u16, &list[k].1))
}

/// SPEC-0045 909 routine A (entities 37 to 42; Tor's lines of the x/y pairs).
fn farm_routine_a(s: &mut GameState, rot: &mut Rotation) -> Option<u16> {
    let story = [
        (302, cycle(&[0, 2, 4])),
        (278, cycle(&[6, 8, 10])),
        (216, cycle(&[12, 13, 14])),
        (202, cycle(&[15, 16, 17])),
        (193, cycle(&[18, 19, 20])),
        (150, cycle(&[21, 22, 23])),
        (147, cycle(&[24, 25, 26])),
        (143, cycle(&[27, 28, 29])),
        (126, cycle(&[30, 31, 32])),
        (122, cycle(&[33, 34, 35])),
        (119, cycle(&[36, 37, 38])),
        (117, cycle(&[39, 40, 41])),
        (113, cycle(&[42, 43, 44])),
    ];
    if let Some(line) = cascade(s, rot, &story) {
        return line;
    }
    let k = story.len() as u16;
    if s.flag(112) {
        s.set(113, true);
        s.items.remove(&7);
        return rot.pick(k, &Lines::One(45));
    }
    if s.flag(111) {
        // 46, then 189 once (the computed selector), then 47, 48, 46 in turn.
        return rot.pick(k + 1, &Lines::Then(vec![46, 189], vec![47, 48, 46]));
    }
    if s.flag(110) {
        s.set(111, true);
        return rot.pick(k + 2, &Lines::One(49));
    }
    rot.pick(k + 3, &Lines::FirstOnce(50, vec![51, 52]))
}

/// SPEC-0045 909 routine B (entities 43 to 50).
fn farm_routine_b(s: &mut GameState, rot: &mut Rotation) -> Option<u16> {
    if s.flag(587) && !s.flag(588) {
        s.items.remove(&85);
        *s.items.entry(86).or_insert(0) += 1;
        s.set(588, true);
        return rot.pick(0, &Lines::One(53));
    }
    if s.flag(516) && !s.flag(518) {
        if s.has_item(76) {
            s.items.remove(&74);
            s.items.remove(&76);
            s.set(518, true);
            return rot.pick(1, &Lines::One(55));
        }
        return rot.pick(2, &Lines::One(57));
    }
    let story = [
        (302, cycle(&[59, 61, 63])),
        (278, cycle(&[65, 67, 69])),
        (216, cycle(&[71, 72, 73])),
        (202, cycle(&[74, 75, 76])),
        (193, cycle(&[77, 78, 79])),
        (150, cycle(&[80, 81, 82])),
        (147, cycle(&[83, 84, 85])),
        (143, cycle(&[86, 87, 88])),
        (126, cycle(&[89, 90, 91])),
        (122, cycle(&[92, 93, 94])),
        (119, Lines::One(95)),
    ];
    // Story branches are numbered after the three item branches.
    if let Some(k) = story.iter().position(|(f, _)| s.flag(*f)) {
        return rot.pick(3 + k as u16, &story[k].1);
    }
    let k = 3 + story.len() as u16;
    if s.flag(117) {
        s.set(119, true);
        return rot.pick(k, &Lines::One(96));
    }
    if s.flag(114) {
        if s.has_item(34) {
            s.set(117, true);
            s.set(116, true);
            s.items.remove(&34);
            return rot.pick(k + 1, &Lines::One(97));
        }
        let line = rot.pick(k + 2, &cycle(&[98, 99, 100]));
        if line == Some(98) {
            s.set(116, true);
        }
        return line;
    }
    if s.flag(113) {
        s.set(114, true);
        return rot.pick(k + 3, &Lines::One(101));
    }
    let tail = [(112, cycle(&[102, 103, 104])), (111, Lines::One(105)), (110, cycle(&[106, 107, 108]))];
    if let Some(i) = tail.iter().position(|(f, _)| s.flag(*f)) {
        return rot.pick(k + 4 + i as u16, &tail[i].1);
    }
    rot.pick(k + 7, &Lines::FirstOnce(109, vec![110, 111]))
}

/// SPEC-0045 909 routine C (entities 31 to 36). The second flag-110 test is dead (line 156).
fn farm_routine_c(s: &mut GameState, rot: &mut Rotation) -> Option<u16> {
    let story = [
        (302, cycle(&[112, 113, 114])),
        (278, cycle(&[118, 119, 120])),
        (216, cycle(&[124, 125, 126])),
        (202, cycle(&[127, 128, 129])),
        (193, cycle(&[130, 131, 132])),
        (150, cycle(&[133, 134, 135])),
        (147, cycle(&[136, 137, 138])),
        (126, cycle(&[139, 140, 141])),
        (122, cycle(&[142, 143, 144])),
        (114, cycle(&[145, 146, 147, 148, 149])),
        (111, cycle(&[150, 151, 152])),
        (110, cycle(&[153, 154, 155])),
        (109, Lines::One(157)),
        (108, Lines::One(158)),
    ];
    if let Some(line) = cascade(s, rot, &story) {
        return line;
    }
    rot.pick(story.len() as u16, &cycle(&[159, 160, 161]))
}

/// SPEC-0045 909 activation thread: the slot loads by story stage s (the highest of flags 1 to
/// 6). Slot 0 is the fourth slot (regions 910 to 913); slots 1, 2 and 3 are P (914 to 919),
/// Q (920 to 925) and R (926 to 933). The arrival step, stage events and talk thread that
/// follow are not covered.
fn farm_slots(state: &GameState, entry_10: bool) -> Vec<Action> {
    let s = state.stage();
    let f = |n| state.flag(n);
    let load = |slot, region| Action::LoadRegion { slot, region };
    let mut out = Vec::new();
    match s {
        Some(4) => out.push(load(0, Some(913))),
        Some(2) => out.push(load(0, Some(912))),
        None | Some(1) => out.push(load(0, Some(if f(110) { 911 } else { 910 }))),
        _ => {}
    }
    let pqr = |p: Option<u32>, q: Option<u32>, r: Option<u32>| [load(1, p), load(2, q), load(3, r)];
    let slots = match s {
        Some(1) if entry_10 && f(119) && !f(121) => pqr(None, None, None),
        Some(1) => {
            let p = if f(126) { 917 } else if f(122) { 916 } else { 914 };
            let q = if f(122) { 922 } else if f(126) { 920 } else if f(121) { 921 } else { 920 };
            let r = if f(122) {
                Some(930)
            } else if f(119) {
                None
            } else if f(117) {
                Some(926)
            } else if f(114) {
                Some(927)
            } else {
                Some(926)
            };
            pqr(Some(p), Some(q), r)
        }
        Some(2) if f(147) && !f(148) => pqr(None, None, None),
        Some(2) if f(147) => pqr(Some(914), Some(920), Some(926)),
        Some(2) => [load(2, Some(923)), load(3, Some(931)), load(1, None)],
        Some(4) => pqr(Some(918), Some(924), Some(932)),
        Some(5 | 6) if state.has_item(36) => pqr(Some(914), Some(920), Some(926)),
        Some(5 | 6) => pqr(Some(919), Some(925), Some(933)),
        _ => pqr(Some(914), Some(920), Some(926)),
    };
    out.extend(slots);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn talks(pack: u32, routine: u8, s: &mut GameState, n: usize) -> Vec<u16> {
        (0..n)
            .filter_map(|_| match on_talk(pack, routine, s).first() {
                Some(Action::Talk { line }) => Some(*line),
                _ => None,
            })
            .collect()
    }

    #[test]
    fn farm_routine_a_test_vectors() {
        // SPEC-0045: flags 302 to 111 clear; with 110 clear, talks 1 to 5 show 50, 51, 52, 51,
        // 52; with 110 set, talk 1 shows 49 and sets 111, then 46, 189, 47, 48, 46, 47, 48.
        let mut s = GameState::default();
        assert_eq!(talks(909, 0, &mut s, 5), vec![50, 51, 52, 51, 52]);
        let mut s = GameState::default();
        s.set(110, true);
        assert_eq!(talks(909, 0, &mut s, 8), vec![49, 46, 189, 47, 48, 46, 47, 48]);
        assert!(s.flag(111));
    }

    #[test]
    fn farm_routine_b_item_handoffs() {
        // SPEC-0045 test vector: 516 set, 518 clear, item 76 held: items 74 and 76 removed, 518
        // set. 587 set, 588 clear: item 85 removed, item 86 given, 588 set.
        let mut s = GameState::default();
        s.set(516, true);
        s.items.insert(74, 1);
        s.items.insert(76, 1);
        assert_eq!(talks(909, 1, &mut s, 1), vec![55]);
        assert!(s.flag(518) && !s.has_item(74) && !s.has_item(76));
        let mut s = GameState::default();
        s.set(587, true);
        s.items.insert(85, 1);
        assert_eq!(talks(909, 1, &mut s, 1), vec![53]);
        assert!(s.flag(588) && s.has_item(86) && !s.has_item(85));
        // New game: 109; then 110, 111.
        assert_eq!(talks(909, 1, &mut GameState::default(), 4), vec![109, 110, 111, 110]);
    }

    #[test]
    fn rotation_restarts_on_a_branch_change_and_reload() {
        let mut s = GameState::default();
        assert_eq!(talks(909, 2, &mut s, 2), vec![159, 160]);
        s.set(108, true);
        assert_eq!(talks(909, 2, &mut s, 1), vec![158]);
        s.set(108, false);
        assert_eq!(talks(909, 2, &mut s, 1), vec![159], "a new branch restarts the count");
        talks(909, 2, &mut s, 1);
        s.reload_module(909);
        assert_eq!(talks(909, 2, &mut s, 1), vec![159]);
    }

    #[test]
    fn farm_doors_and_returns() {
        let mut s = GameState::default();
        assert_eq!(on_teleport(909, 1, &mut s), vec![Action::EnterInterior { slot: 73 }]);
        // Leaving 955 posts 40 for eW 68; 909's activation then moves the hero to locator 3 and
        // resets the message.
        assert_eq!(on_teleport(955, 0, &mut s), vec![Action::LeaveInterior { slot: 5 }]);
        assert_eq!(on_activate(909, &mut s)[0], Action::MoveHero { locator: 3 });
        assert!(!matches!(on_activate(909, &mut s)[0], Action::MoveHero { .. }));
        // 951 with state 119 set and 122 clear shows line 126 and posts nothing.
        s.set(119, true);
        assert_eq!(on_teleport(951, 0, &mut s), vec![Action::Message { line: 126 }]);
        s.set(122, true);
        assert_eq!(on_teleport(951, 0, &mut s), vec![Action::LeaveInterior { slot: 5 }]);
        assert_eq!(on_activate(909, &mut s)[0], Action::MoveHero { locator: 2 });
    }

    #[test]
    fn farm_family_by_stage() {
        // No stage: fourth slot 910, then P 914, Q 920, R 926.
        let mut s = GameState::default();
        let load = |slot, region| Action::LoadRegion { slot, region };
        let loads = |a: Vec<Action>| a.into_iter().filter(|a| matches!(a, Action::LoadRegion { .. })).collect::<Vec<_>>();
        assert_eq!(loads(on_activate(909, &mut s)), vec![load(0, Some(910)), load(1, Some(914)), load(2, Some(920)), load(3, Some(926))]);
        // SPEC-0045 test vector: flags 1 and 122 set, no entry value 10: 910 or 911 by flag
        // 110, P 916, Q 922, R 930.
        s.set(1, true);
        s.set(122, true);
        // A finished round-up (107 to 110 all set) is kept, so flag 110 picks 911.
        for f in 107..=110 {
            s.set(f, true);
        }
        assert_eq!(loads(on_activate(909, &mut s)), vec![load(0, Some(911)), load(1, Some(916)), load(2, Some(922)), load(3, Some(930))]);
    }

    #[test]
    fn farm_activation_keeps_a_finished_roundup_only() {
        let mut s = GameState::default();
        s.set(107, true);
        s.set(108, true);
        on_activate(909, &mut s);
        assert!(!s.flag(107) && !s.flag(108));
        for f in 107..=110 {
            s.set(f, true);
        }
        on_activate(909, &mut s);
        assert!((107..=110).all(|f| s.flag(f)));
    }

    fn fire(s: &mut GameState, routine: u8, index: u16, square: bool) -> TriggerOut {
        on_trigger(909, routine, TriggerCall { index, state: 1, object: true, square }, s)
    }

    #[test]
    fn farm_camera_pair_vectors() {
        // SPEC-0045 routine 10: fires 1, 0, 0: line 192 on the second fire, B cleared; the third
        // plays nothing. A new load with fires 0, 1, 0: the third shows line 192.
        let mut s = GameState::default();
        assert!(fire(&mut s, 10, 1, false).actions.is_empty());
        assert_eq!(fire(&mut s, 10, 0, false).actions, vec![Action::Message { line: 192 }]);
        assert!(fire(&mut s, 10, 0, false).actions.is_empty());
        s.reload_module(909);
        assert!(fire(&mut s, 10, 0, false).actions.is_empty());
        assert!(fire(&mut s, 10, 1, false).actions.is_empty(), "camera 6, A cleared");
        assert_eq!(fire(&mut s, 10, 0, false).actions, vec![Action::Message { line: 192 }]);
        // Once per load: a further 1, 0 crossing shows nothing.
        fire(&mut s, 10, 1, false);
        assert!(fire(&mut s, 10, 0, false).actions.is_empty());
        // With flag 220 set the line never shows.
        let mut s = GameState::default();
        s.set(220, true);
        fire(&mut s, 10, 1, false);
        assert!(fire(&mut s, 10, 0, false).actions.is_empty());
    }

    #[test]
    fn farm_pump_exchange() {
        let mut s = GameState::default();
        assert_eq!(fire(&mut s, 11, 5, false).prompt, Some(4));
        s.set(110, true);
        s.give(6, 1);
        // Update: the bucket held after the round-up sets 111.
        on_update(909, &mut s, false);
        assert!(s.flag(111));
        let out = fire(&mut s, 11, 5, true);
        let lines: Vec<&Action> = out.actions.iter().filter(|a| matches!(a, Action::Message { .. } | Action::MoveHero { .. })).collect();
        assert_eq!(lines, vec![&Action::MoveHero { locator: 29 }, &Action::Message { line: 166 }, &Action::Message { line: 162 }]);
        assert_eq!(out.actions.first(), Some(&Action::HoldHero));
        assert_eq!(out.actions.last(), Some(&Action::ReturnCamera { preset: -1, release: true }));
        assert!(s.flag(112) && s.has_item(7) && !s.has_item(6));
        assert_eq!(fire(&mut s, 11, 5, true).actions, vec![Action::Message { line: 167 }]);
        // Mum's talk (routine A) takes the water; Hubert's sends Tor for the pliers.
        assert_eq!(talks(909, 0, &mut s, 1), vec![45]);
        assert!(s.flag(113) && !s.has_item(7));
        assert_eq!(talks(909, 1, &mut s, 1), vec![101]);
        assert!(s.flag(114));
        let scene = |a: Vec<Action>| a.into_iter().filter(|a| matches!(a, Action::LoadRegion { .. } | Action::MoveEntity { .. } | Action::WalkEntity { .. })).collect::<Vec<_>>();
        assert!(scene(on_update(909, &mut s, true)).is_empty(), "the scene waits for the box to close");
        assert_eq!(
            scene(on_update(909, &mut s, false)),
            vec![Action::LoadRegion { slot: 3, region: Some(927) }, Action::MoveEntity { entity: 44, locator: 15, face: true }, Action::WalkEntity { entity: 44, locator: 16, wait: true, orient: false, stop: 0.0 }]
        );
        // Without the pliers Hubert's line 98 sets 116; picking them up then sets 618.
        assert_eq!(talks(909, 1, &mut s, 1), vec![98]);
        s.give(34, 1);
        on_update(909, &mut s, false);
        assert!(s.flag(618));
        assert_eq!(talks(909, 1, &mut s, 1), vec![97]);
        assert!(s.flag(116) && s.flag(117) && !s.has_item(34));
        let regions: Vec<Action> = on_update(909, &mut s, false).into_iter().filter(|a| matches!(a, Action::LoadRegion { .. })).collect();
        let load = |region| Action::LoadRegion { slot: 3, region };
        assert_eq!(regions, vec![load(Some(926)), load(Some(928)), load(None)]);
        assert!(s.flag(118) && s.flag(119));
    }

    #[test]
    fn house_chest_sword_and_parcel() {
        // SPEC-0041 951: the first entry with 119 set gives item 15, sets 120 and loads 953.
        let mut s = GameState::default();
        s.set(119, true);
        let init = on_init(951, &mut s);
        assert_eq!(init, vec![Action::LockChest { id: 0, key: 15, message: 124 }, Action::LoadRegion { slot: 0, region: Some(953) }]);
        assert!(s.flag(120) && s.has_item(15));
        assert!(!on_init(951, &mut s).is_empty() && s.items[&15] == 1, "item 15 once");
        // The leave door holds Tor in with line 126 until the parcel.
        assert_eq!(on_teleport(951, 0, &mut s), vec![Action::Message { line: 126 }]);
        // SPEC-0048: the chest takes all of item 15 and opens; without it, the lock line.
        let locks = [(0, 15, 124)];
        assert_eq!(chest_press(&locks, 0, &mut GameState::default()), ChestPress::Locked { line: 124 });
        assert_eq!(chest_press(&locks, 0, &mut s), ChestPress::Open);
        assert!(!s.has_item(15));
        assert_eq!(chest_press(&locks, 1, &mut GameState::default()), ChestPress::Open, "no lock entry");
        // SPEC-0041 test vector: callback 3 with 147 and 122 clear and item 98 held.
        assert_eq!(on_talk(951, 2, &mut s), vec![Action::Message { line: 60 }]);
        s.give(98, 1);
        assert_eq!(on_talk(951, 2, &mut s), vec![Action::Message { line: 59 }]);
        assert!(s.flag(121) && s.flag(122) && s.has_item(32));
        assert_eq!(on_talk(951, 2, &mut s), vec![Action::Message { line: 58 }]);
        assert_eq!(on_talk(951, 0, &mut s), vec![Action::Message { line: 4 }]);
        assert_eq!(on_teleport(951, 0, &mut s), vec![Action::LeaveInterior { slot: 5 }]);
    }

    #[test]
    fn road_fight_watch() {
        // SPEC-0042 678 test vector: raiders at 80, 60 and 50 (sum 190); 55, 55 and 50 (30
        // lost) keep waiting; 54, 55 and 50 (31 lost) run the end scene and set 125.
        let mut s = GameState::default();
        let init = on_init(691, &mut s);
        assert!(init.contains(&Action::LoadRegion { slot: 0, region: Some(692) }));
        assert!(init.contains(&Action::RegisterTrigger { index: 1, routine: 1 }));
        let intro = on_trigger(691, 1, TriggerCall { index: 1, state: 1, object: true, square: false }, &mut s);
        assert!(s.flag(124) && intro.actions.contains(&Action::Agro { entity: 1 }));
        let hp = |s: &mut GameState, v: [i32; 3]| s.entity_hp = [(0, v[0]), (2, v[1]), (1, v[2])].into_iter().collect();
        hp(&mut s, [80, 60, 50]);
        assert!(on_update(691, &mut s, false).is_empty());
        hp(&mut s, [55, 55, 50]);
        assert!(on_update(691, &mut s, false).is_empty());
        assert!(!s.flag(125));
        hp(&mut s, [54, 55, 50]);
        let end = on_update(691, &mut s, false);
        assert!(end.contains(&Action::LoadRegion { slot: 0, region: Some(695) }));
        assert!(s.flag(125) && s.flag(126) && s.items[&170] == 100);
        // The intro does not run again; the next visit loads set 3 (thread A).
        assert!(on_trigger(691, 1, TriggerCall { index: 1, state: 1, object: true, square: false }, &mut s).actions.is_empty());
        s.reload_module(691);
        assert_eq!(on_init(691, &mut s).iter().filter(|a| matches!(a, Action::LoadRegion { .. })).collect::<Vec<_>>(), vec![&Action::LoadRegion { slot: 0, region: Some(695) }]);
    }

    #[test]
    fn village_chase() {
        // SPEC-0045 1023: trigger 17 starts the chase (7 and 127 set, item 32 taken); the chase
        // thread walks waypoints 477 to 485 and sets 128 to 135 and then 136 as it goes.
        let mut s = GameState::default();
        s.give(32, 1);
        let init = on_init(1023, &mut s);
        assert!(init.contains(&Action::RegisterTrigger { index: 17, routine: 6 }));
        assert!(init.contains(&Action::LoadRegion { slot: 0, region: Some(1026) }));
        let start = on_trigger(1023, 6, TriggerCall { index: 17, state: 1, object: true, square: false }, &mut s);
        assert!(s.flag(7) && s.flag(127) && !s.has_item(32));
        let flags: Vec<u16> = start.actions.iter().filter_map(|a| match a {
            Action::SetFlag { flag, .. } => Some(*flag),
            _ => None,
        }).collect();
        assert_eq!(flags, (128..=136).collect::<Vec<_>>());
        assert!(start.actions.contains(&Action::GiveItem { item: 32, count: 1 }));
        // A reload mid-chase (128 to 130 set) resumes at waypoint 3 (locator 480).
        let mut s = GameState::default();
        for f in [127, 128, 129, 130] {
            s.set(f, true);
        }
        let init = on_init(1023, &mut s);
        assert!(init.contains(&Action::MoveEntity { entity: 0, locator: 480, face: true }));
        assert!(init.contains(&Action::LoadRegion { slot: 0, region: Some(1024) }));
        // The shops stay shut until the catch.
        assert_eq!(on_teleport(1023, 5, &mut s), vec![Action::Message { line: 5 }]);
        s.set(136, true);
        assert_eq!(on_teleport(1023, 5, &mut s), vec![Action::EnterInterior { slot: 117 }]);
    }

    #[test]
    fn recruitment_errands() {
        // SPEC-0058 step 7 through the handlers: the recruiter takes the parcel (137) and, with
        // the shield (106), gives the knight's letter (39, 138); Torak sets 139; the shop sells
        // item 42 (140, 644); the smithy trades 40 to 43 for 60 (141); Torak takes 60 and 39
        // (142; line 4 gives 38); the recruiter takes 38 (143, 17, 2).
        let mut s = GameState::default();
        s.give(32, 1);
        s.give(106, 1);
        s.neopoints = 150;
        assert_eq!(on_talk(1148, 1, &mut s), vec![Action::Talk { line: 34 }]);
        assert!(s.flag(137) && s.flag(138) && s.has_item(39) && !s.has_item(32));
        assert_eq!(on_talk(1150, 4, &mut s), vec![Action::Talk { line: 0 }]);
        assert!(s.flag(139));
        let shop = on_talk(1148, 2, &mut s);
        assert_eq!(shop, vec![Action::Talk { line: 41 }, Action::Prompt { line: 43, routine: 0 }]);
        let bought = on_answer(1148, 0, true, &mut s);
        assert_eq!(bought[0], Action::Message { line: 44 });
        assert!(bought.contains(&Action::GiveWordItem { word: 1, default: 99 }), "the walk-back sequence gives item 99");
        assert!(s.flag(140) && s.flag(644) && s.has_item(42) && s.neopoints == 50);
        assert_eq!(on_talk(1133, 0, &mut s), vec![Action::Talk { line: 37 }], "one of the four");
        for i in [40, 41, 43] {
            s.give(i, 1);
        }
        assert_eq!(on_talk(1133, 0, &mut s), vec![Action::Talk { line: 34 }]);
        assert!(s.flag(141) && s.has_item(60) && !s.has_item(42));
        assert_eq!(on_talk(1150, 4, &mut s), vec![Action::Talk { line: 4 }]);
        assert!(s.flag(142) && !s.has_item(60) && !s.has_item(39));
        s.give(38, 1);
        assert_eq!(on_talk(1148, 1, &mut s), vec![Action::Talk { line: 22 }]);
        assert!(s.flag(143) && s.flag(17) && s.flag(2) && !s.has_item(38));
        // The trade answer: prompt 46 yes with the sword held.
        let mut s = GameState::default();
        s.set(139, true);
        s.give(98, 1);
        assert_eq!(on_answer(1148, 0, false, &mut s), vec![Action::Prompt { line: 46, routine: 1 }]);
        // The walk-back sequence then gives the traded sword (word 1 = 98) back.
        let traded = on_answer(1148, 1, true, &mut s);
        assert_eq!(traded[0], Action::Message { line: 47 });
        assert!(traded.contains(&Action::GiveWordItem { word: 1, default: 99 }));
        assert!(!s.has_item(98) && s.has_item(42) && s.word(1148, 1) == 98);
    }

    #[test]
    fn village_templates() {
        // Template A (entity 69, b 84): flag 7 gives b+11 to b+13; nothing set: silent.
        let mut s = GameState::default();
        let r = proximity_routine(1023, 69).unwrap();
        assert!(on_talk(1023, r, &mut s).is_empty());
        s.set(7, true);
        assert_eq!(talks(1023, r, &mut s, 4), vec![95, 96, 97, 95]);
        // Template C (entity 3, b 310): 138 set and 141 clear: b+14 sets 644.
        s.set(138, true);
        let c = proximity_routine(1023, 3).unwrap();
        assert_eq!(talks(1023, c, &mut s, 2), vec![324, 325]);
        assert!(s.flag(644));
        // Shared routine 76 and 78.
        assert_eq!(proximity_routine(1023, 76), proximity_routine(1023, 78));
        // The return from the Recruitment Centre (eW 117) puts the hero at locator 11.
        s.last_pack = Some(1148);
        assert_eq!(on_activate(1023, &mut s), vec![Action::MoveHero { locator: 11 }]);
    }

    #[test]
    fn act_1_start() {
        // SPEC-0058 test vector 8: 909 Update with 707 on and 1 off: flag 1 on, 706 off.
        let mut s = GameState::default();
        s.set(707, true);
        on_activate(909, &mut s);
        assert!(s.flag(706));
        on_update(909, &mut s, false);
        assert!(s.flag(1) && !s.flag(706));
        assert_eq!(s.stage(), Some(1));
        // Back from the house with the parcel (entry value 10): the flag-122 scene sets 123.
        for f in [119, 121, 122] {
            s.set(f, true);
        }
        s.blackboard.insert((2, 68), 10);
        assert_eq!(on_activate(909, &mut s)[0], Action::MoveHero { locator: 2 });
        assert!(s.flag(123));
        // Stage 2 with 147 and 145 clear: 145 (Tor's sister has gone missing).
        s.set(2, true);
        on_activate(909, &mut s);
        assert!(s.flag(145));
    }

    #[test]
    fn value_50_goes_to_locator_28_at_stage_6_without_item_36() {
        let mut s = GameState::default();
        s.set(6, true);
        s.blackboard.insert((2, 68), 50);
        assert_eq!(on_activate(909, &mut s)[0], Action::MoveHero { locator: 28 });
        s.items.insert(36, 1);
        s.blackboard.insert((2, 68), 50);
        assert_eq!(on_activate(909, &mut s)[0], Action::MoveHero { locator: 4 });
    }
}
