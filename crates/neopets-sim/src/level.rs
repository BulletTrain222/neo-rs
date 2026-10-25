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
    /// WalkEntity or WalkEntityAndOrient (rows 106 and 108) with stop 0 (class 23 or 25).
    WalkEntity { entity: usize, locator: usize, wait: bool, orient: bool },
    /// SetConversationTarget(entity), LevelMsg(line, WAIT 1), EndConversation(entity).
    Converse { entity: usize, line: u16 },
    /// Sleep(n ticks).
    Sleep { ticks: u32 },
    /// Wait until no world or character load is active.
    WaitLoads,
    /// Wait until no dialogue box is up.
    WaitDialog,
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
        _ => Vec::new(),
    }
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
                out.extend([Action::WalkEntity { entity: 44, locator: 16, wait: true, orient: false }, fade_out(), release_camera(), fade_in()]);
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
                out.push(Action::WalkEntity { entity: 44, locator: 16, wait: true, orient: false });
                out.push(Action::PlaySound { cue: 12, locator: None });
                out.extend([Action::WalkEntity { entity: 44, locator: 15, wait: true, orient: false }, fade_out()]);
                out.extend([Action::LoadRegion { slot: 3, region: Some(926) }, Action::LoadRegion { slot: 3, region: Some(928) }, Action::WaitLoads]);
                state.set(119, true);
                out.extend([Action::MoveEntity { entity: 45, locator: 21, face: true }, camera(15), fade_in()]);
                out.extend([Action::WalkEntity { entity: 45, locator: 23, wait: true, orient: false }, fade_out()]);
                out.extend([Action::LoadRegion { slot: 3, region: None }, release_camera(), fade_in()]);
            }
            if state.flag(278) {
                state.set(279, true);
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
        _ => Vec::new(),
    }
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
            vec![Action::LoadRegion { slot: 3, region: Some(927) }, Action::MoveEntity { entity: 44, locator: 15, face: true }, Action::WalkEntity { entity: 44, locator: 16, wait: true, orient: false }]
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
