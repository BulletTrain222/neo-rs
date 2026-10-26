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

/// The treasure map item (SPEC-0021 Treasure maps).
pub const TREASURE_MAP: u16 = 175;

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
    /// SetLevelState: a level flag of the current world (the boss's and the script's shared
    /// flags, SPEC-0014; SPEC-0015), set or cleared at this point of the thread.
    LevelState { flag: u16, on: bool },
    /// Set and clear bits of a module word of the thread's pack at this point of the thread.
    Bits { word: u8, set: i32, clear: i32 },
    /// PlayHeroAnim.
    HeroAnim { anim: u32, wait: bool },
    /// The hero's equipment display on or off.
    Equip { on: bool },
    /// StartExteriorReplacementLoad of an eW slot, arriving on a plane (-1: entry record 0).
    ReplaceExterior { slot: usize, plane: i32 },
    /// The hero's magic refilled (the 1056 mote area).
    RefillMagic,
    /// InitMiamouseBehavior(field mesh, cage mesh) and MiamiceCatchDisable (SPEC-0029
    /// Behavior 28).
    MiamouseInit { field: i32, cage: i32 },
    MiamouseDisable,
    /// SetHeroSwapMode (SPEC-0030 row 167; SPEC-0053 3.1).
    SwapMode { mode: u32 },
    /// SwitchHero(h, a, b) (row 173; SPEC-0053 3.2 and 3.5): a request to hero h with a as the
    /// first flag (swap sound and effects), b as the second (the white flash) and extra flag 0;
    /// no request when h is -1 or already selected.
    SwitchHero { hero: i32, effects: bool, flash: bool },
    /// ActivateHero(h, locator, block, fade, cue) (row 187): MoveHero(locator) on the hero
    /// selected now, then SwitchHero(h, fade, cue), then BlockPlayerControl(h) when block is set.
    ActivateHero { hero: i32, locator: usize, block: bool, fade: bool, cue: bool },
    /// TriggerActStart(stage) (row 51; SPEC-0053 3.7). IsActStartDone is always true, so the
    /// scripts' wait after it ends at once.
    ActStart { stage: i32 },
    /// StartEffect(slot, W = 0, camera) and StopEffect(slot) on the thread's world's placed
    /// effects (SPEC-0030 rows 5 and 6).
    StartEffect { slot: usize, camera: bool },
    StopEffect { slot: usize },
    /// MoveEffect(slot, locator, face off) (SPEC-0030 row 28): the placed effect's position
    /// becomes the locator's (entry record) position.
    MoveEffect { slot: usize, locator: usize },
    /// The animated-object calls of SPEC-0030 rows 1 to 4, 9 and 11 on the thread's world:
    /// InitObjOn and ActivateObj (start or end pose), PlayAnim (with WAIT the thread waits while
    /// the object plays once), LoopAnim, StopAnim and DeactivateObj.
    InitObjOn { obj: usize, end: bool },
    PlayAnim { obj: usize, speed: f32, wait: bool },
    LoopAnim { obj: usize, speed: f32 },
    StopAnim { obj: usize },
    DeactivateObj { obj: usize },
    /// The prize wheel (SPEC-0059 section 2) on the thread's world: the cells lit by the light
    /// show's half (even or odd cells), a new pattern, a spin (the thread waits until it
    /// stops), the result, and the round step (`wheel_round`).
    WheelLights { even: bool },
    WheelPattern,
    WheelSpin,
    WheelResult,
    WheelRound,
    /// The 1217 object records (SPEC-0045 1217): a request to a main record (kind 0, with an
    /// inner request unless -1), a gate (1) or a cage (2); the close of main record `index`
    /// (request 4 unless in state 5, inner request 4 unless in 5 or 7); thread routine 2 (the
    /// staggered request 1 of the eleven main records); a wait while a cage is in state 4; a
    /// sleep of Random(lo, hi) ticks.
    BurrowsRequest { kind: u8, index: u8, request: i8, inner: i8 },
    BurrowsClose { index: u8 },
    BurrowsStagger,
    WaitCage { index: u8 },
    /// A lock chest's reward step (SPEC-0048 Lock chests, step 6): the drop action on group g's
    /// reward placement, the character of reward [1], and its collected entry set.
    ChestReward { group: u16 },
    /// A dig site's steps (SPEC-0051 section 10, dig sequence) for script group g: 8 (effect
    /// 790 at P + 2.9 D facing D), 11 (the reward) and 13 (the collected entry).
    DigStep { group: u16, step: u8 },
    /// The switch setter's effect on a placed record (SPEC-0030 section 2.1; SPEC-0028
    /// section 9): on sets its active flag (it starts nothing itself), off clears it and drains
    /// the record.
    SwitchEffect { slot: usize, on: bool },
    /// SetObjToTime(obj, seconds).
    SetObjTime { obj: usize, seconds: f32 },
    /// SetObjToTime(obj, r / 100) with r = Random(0, 100), both ends included (the host draws).
    SetObjTimeRandom { obj: usize },
    /// RegisterScriptGroupType(W, kind, run, init) (SPEC-0048 Script groups, type
    /// registration): the host registers the first trigger of every entry of that kind (with
    /// its group as owner, replacing an earlier registration) and runs `group_init` for each.
    RegisterGroupType { kind: u32 },
    /// Wait while the trigger record `index` stays non-zero (the firer is still inside).
    WaitInside { index: u16 },
    /// RegisterScriptInstanceTrigger(W, routine, t, i) (SPEC-0030 row 261): for a kind-7 entry i
    /// every member trigger is registered (t ignored); otherwise trigger t.
    RegisterInstance { routine: u8, trigger: u16, group: usize },
    /// Wait until a module word of the thread's pack is 0.
    WaitWord { word: u8 },
    /// StartCameraShake(T, A, S, p4, R, phi, Z) (SPEC-0027 Camera shake).
    CameraShake { t: f32, a: f32, s: f32, p4: f32, r: f32, phi: f32, z: f32 },
    /// StartCameraShakeMode(type, duration, strength).
    CameraShakeMode { kind: i32, duration: f32, strength: f32 },
    /// DeactivateScriptGroup(i) (SPEC-0030 row 270): its member triggers start nothing and its
    /// member objects are deactivated.
    DeactivateGroup { group: usize },
    /// ReActivateScriptGroup(i) (row 271): the built-in set-up of its kind runs again.
    ReactivateGroup { group: usize },
    SleepRandom { lo: u32, hi: u32 },
}

/// The prize wheel's module words in pack 1023: G, r, the angle A (f32 bits) and the lit cells.
pub const WHEEL_WORDS: [u8; 4] = [20, 21, 22, 23];

/// The wheel's game words of a pack.
pub fn wheel_of(pack: u32, s: &GameState) -> crate::wheel::Wheel {
    let lit = s.word(pack, WHEEL_WORDS[3]);
    crate::wheel::Wheel {
        g: s.word(pack, WHEEL_WORDS[0]),
        r: s.word(pack, WHEEL_WORDS[1]) as u32,
        angle: f32::from_bits(s.word(pack, WHEEL_WORDS[2]) as u32),
        lit: std::array::from_fn(|i| lit & (1 << i) != 0),
    }
}

/// Store the wheel's game words.
pub fn set_wheel(pack: u32, s: &mut GameState, w: &crate::wheel::Wheel) {
    s.set_word(pack, WHEEL_WORDS[0], w.g);
    s.set_word(pack, WHEEL_WORDS[1], w.r as i32);
    s.set_word(pack, WHEEL_WORDS[2], w.angle.to_bits() as i32);
    s.set_word(pack, WHEEL_WORDS[3], (0..8).filter(|&i| w.lit[i]).fold(0, |m, i| m | 1 << i));
}

/// The spin sequence: Wait(1.0 s), a spin, Wait(1.0 s), the result, the next round step.
fn wheel_spin() -> Vec<Action> {
    vec![Action::Sleep { ticks: 60 }, Action::WheelSpin, Action::Sleep { ticks: 60 }, Action::WheelResult, Action::WheelRound]
}

/// The round step (SPEC-0059 section 2): G 0 the start prompt (routine 60); G 1 with r below 4
/// go-on prompt r (routine 61); G 1 with r = 4 the five-win line and pay-out; G -1 the loss
/// line. A game end resets G and r; the hero is let go with the camera returned.
pub fn wheel_round(pack: u32, s: &mut GameState) -> Vec<Action> {
    let c = crate::wheel::copy_of(pack);
    let mut w = wheel_of(pack, s);
    let end = |s: &mut GameState, w: &mut crate::wheel::Wheel| {
        w.g = 0;
        w.r = 0;
        set_wheel(pack, s, w);
    };
    match (w.g, w.r) {
        (0, _) => vec![Action::Prompt { line: c.start_prompt, routine: 60 }],
        (1, r) if r < 4 => vec![Action::Prompt { line: c.go_on_prompts[r as usize], routine: 61 }],
        (1, _) => {
            s.neopoints += c.payouts[4];
            if let Some(f) = c.flags {
                s.set(f[2], true);
            }
            end(s, &mut w);
            vec![Action::Message { line: c.five_win }, release_camera()]
        }
        (_, r) => {
            if let Some(f) = c.flags {
                s.set(f[0], true);
            }
            end(s, &mut w);
            vec![Action::Message { line: c.loss[(r as usize).min(4)] }, release_camera()]
        }
    }
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
    /// Held treasure maps by treasure number (SPEC-0021 Treasure maps; SPEC-0051 section 10).
    pub maps: BTreeSet<u32>,
    /// The shared entity-control tables of the level modules, by pack (SPEC-0043).
    pub entity_tables: HashMap<u32, crate::entityctl::Table>,
    /// IsAI_Dead for the current world's entity ids (SPEC-0030 row 45), filled by the host
    /// each tick: a bound character's dead test, else its kill memory of this load.
    pub ai_dead: BTreeSet<usize>,
    /// The current hit points of the current world's entities by entity id (filled by the host
    /// before each Update; GetAI_HitPoints).
    pub entity_hp: HashMap<usize, i32>,
    /// The wallet (GetNeopoints, RemoveNeopoints).
    pub neopoints: i32,
    /// The pack that was current before this one (the "last active world" an activate-world
    /// routine reads).
    pub last_pack: Option<u32>,
    /// The current world's level flags (SetLevelState and the boss posts; cleared when a world
    /// loads).
    pub level_flags: BTreeSet<u16>,
    /// A Random(1, 3) draw the host makes before each proximity routine (the routines that
    /// draw read it; the shared generator is the host's).
    pub talk_roll: i32,
    /// A Random(1, 6) draw the host makes before each proximity routine (the fame rule's d).
    pub talk_d: i32,
    /// A Random(1, 5) draw the host makes before a proximity routine of pack 1139 (its one-time
    /// sale's pick; drawn only there so that other packs' random streams are unchanged).
    pub talk_five: i32,
    /// A raw draw the host makes before a proximity routine of pack 1358 (its Random(1, n) is
    /// 1 + raw mod n; drawn only there).
    pub talk_raw: u32,
    /// The hero is in a conversation (a box is up), for the routines that wait on it.
    pub talking: bool,
    /// What the host reports of the hero before each Update (the lesson rules of SPEC-0044
    /// 1056 read it).
    pub hero: HeroView,
    /// The object records of pack 1217 while its fight is due (module data).
    pub burrows: Option<crate::burrows::Burrows>,
}

/// Carry out a 1217 record request or close on the records (the host calls it in thread order).
pub fn burrows_request(s: &mut GameState, a: Action) {
    let Some(b) = s.burrows.as_mut() else { return };
    match a {
        Action::BurrowsRequest { kind: 0, index, request, inner } => {
            if let Some(m) = b.mains.get_mut(index as usize) {
                m.request = request as i32;
                if inner >= 0 {
                    m.inner.request = inner as i32;
                }
            }
        }
        Action::BurrowsRequest { kind: 1, index, request, .. } => {
            if let Some(g) = b.gates.get_mut(index as usize) {
                g.request = request as i32;
            }
        }
        Action::BurrowsRequest { index, request, .. } => {
            if let Some(c) = b.cages.get_mut(index as usize) {
                c.request = request as i32;
            }
        }
        Action::BurrowsClose { index } => {
            if let Some(m) = b.mains.get_mut(index as usize) {
                if m.state != 5 {
                    m.request = 4;
                }
                if !matches!(m.inner.state, 5 | 7) {
                    m.inner.request = 4;
                }
            }
        }
        _ => {}
    }
}

/// Thread routine 2: request 1 to the eleven main records, Random(10, 20) ticks apart.
pub fn burrows_stagger() -> Vec<Action> {
    (0..11u8).flat_map(|index| [Action::BurrowsRequest { kind: 0, index, request: 1, inner: -1 }, Action::SleepRandom { lo: 10, hi: 20 }]).collect()
}

/// The hero as the lesson Update reads it this tick.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct HeroView {
    /// Tor's attack sub-state (0: no melee attack).
    pub sub: u8,
    /// The entities of the current world the hero's swing hit this tick.
    pub hits: Vec<usize>,
    /// The hero is occupied (a busy state or a scripted hold).
    pub occupied: bool,
    /// The Motes in equipment words 2 (attack) and 1 (shield).
    pub attack_mote: Option<u16>,
    pub shield_mote: Option<u16>,
    /// The game clock in ticks.
    pub tick: u64,
    /// GetHeroSwapMode, the selected hero (0 Tor, 1 Roberta), IsHeroDead by hero and
    /// IsHeroSwapInProgress (SPEC-0053 3.3).
    pub swap_mode: u32,
    pub selected: usize,
    pub dead: [bool; 2],
    pub swapping: bool,
}

impl GameState {
    pub fn flag(&self, f: u16) -> bool {
        self.flags.contains(&f)
    }

    /// A level flag of the current world.
    pub fn level(&self, f: u16) -> bool {
        self.level_flags.contains(&f)
    }

    pub fn set_level(&mut self, f: u16, on: bool) {
        if on {
            self.level_flags.insert(f);
        } else {
            self.level_flags.remove(&f);
        }
    }

    /// An entity of the current world counts as dead when its hit points are 0 or it is gone.
    pub fn entity_dead(&self, e: usize) -> bool {
        self.entity_hp.get(&e).is_none_or(|&hp| hp <= 0)
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
        self.level_flags.clear();
    }

    fn word(&self, pack: u32, w: u8) -> i32 {
        self.words.get(&(pack, w)).copied().unwrap_or(0)
    }

    /// A module word (for the host's Bits action).
    pub fn word_of(&self, pack: u32, w: u8) -> i32 {
        self.word(pack, w)
    }

    pub fn set_word(&mut self, pack: u32, w: u8, v: i32) {
        self.words.insert((pack, w), v);
    }

    /// GiveItemToHero(item, n).
    pub fn give(&mut self, item: u16, n: u32) {
        // Item 175 takes a treasure number: the give marks map n held (refused when it is held
        // already) and the item count is not changed.
        if item == TREASURE_MAP {
            self.maps.insert(n);
            return;
        }
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
    if (pack, door) == (947, 0) {
        return Some(0);
    }
    if (pack == 1139 && door <= 1) || (pack, door) == (1173, 0) || (pack == 1358 && door <= 5) {
        return Some(13);
    }
    if (pack, door) == (880, 0) || (pack, door) == (1429, 0) || (pack == 1414 && door <= 1) || (pack == 1152 && door <= 1) || (pack == 1299 && door <= 1) || (pack == 991 && door <= 7) || (pack == 1017 && door <= 1) || (door == 0 && eastside_interior(pack).is_some()) {
        return Some(13);
    }
    match (pack, door) {
        (909, 0 | 1) => Some(13),
        (951, 0) => Some(26),
        (955, 0) => Some(13),
        // SPEC-0045 1023: every teleport shows context 13 (index 10 shows 3 while flag 35 is
        // clear; taken as 13 here).
        (1023, 0..=10) => Some(13),
        // SPEC-0045 1163: index 0 on contact (kind 26, no prompt), index 1 context 26.
        (1163, 0) => Some(0),
        (1163, 1) => Some(26),
        // SPEC-0044 1056: the arrival registers indices 0 to 2 with message 13.
        (1056, 0..=2) => Some(13),
        // SPEC-0045 1217: context 0 (on contact).
        (1217, 0 | 1) => Some(0),
        // SPEC-0045 1098: 0, 1 and 3 act on contact; 4, 6 and 7 show context 26; 2 and 5 show
        // a square-button prompt whose context is not given (13 here; R-0037).
        (1098, 0 | 1 | 3) => Some(0),
        (1098, 4 | 6 | 7) => Some(26),
        (1098, 2 | 5) => Some(13),
        _ => None,
    }
}

/// Run a pack's teleport callback for a door.
pub fn on_teleport(pack: u32, door: usize, state: &mut GameState) -> Vec<Action> {
    match (pack, door) {
        // SPEC-0043 1358 teleports (kind 13): 4 and 3 post 50 or 10 for eW 180 and go there; 2
        // posts 40 for eW 173, sets 348 and goes there; 5 with 695 posts 60 for eW 173 and goes
        // there, else line 0; 0 and 1 with 102 post 20 or 30 for eW 159 and enter eW 181 or 183,
        // else line 0; then 3 s.
        (1358, 4 | 3) => {
            state.blackboard.insert((2, 180), if door == 4 { 50 } else { 10 });
            vec![Action::ReplaceExterior { slot: 180, plane: -1 }, wait(3.0)]
        }
        (1358, 2) => {
            state.blackboard.insert((2, 173), 40);
            state.set(348, true);
            vec![Action::ReplaceExterior { slot: 173, plane: -1 }, wait(3.0)]
        }
        (1358, 5) if state.flag(695) => {
            state.blackboard.insert((2, 173), 60);
            vec![Action::ReplaceExterior { slot: 173, plane: -1 }, wait(3.0)]
        }
        (1358, 0 | 1) if state.flag(102) => {
            state.blackboard.insert((2, 159), if door == 0 { 20 } else { 30 });
            vec![Action::EnterInterior { slot: if door == 0 { 181 } else { 183 } }, wait(3.0)]
        }
        (1358, 0 | 1 | 5) => vec![Action::Message { line: 0 }],
        // SPEC-0043 1173 teleport 0: value 110 for eW 119 and the replacement to eW 119 (1152).
        (1173, 0) => {
            state.blackboard.insert((2, 119), 110);
            vec![Action::ReplaceExterior { slot: 119, plane: -1 }]
        }
        // SPEC-0039 1139 teleports: index 0 replaces the exterior with eW 90 (pack 1023); index 1
        // with flag 35 posts 10 for eW 109 and replaces it with eW 109 (pack 1129), else line 0;
        // then 3 s. (Index 1's context is 3 while flag 35 is clear; 13 is shown here.)
        (1139, 0) => vec![Action::ReplaceExterior { slot: 90, plane: -1 }, wait(3.0)],
        (1139, 1) if state.flag(35) => {
            state.blackboard.insert((2, 109), 10);
            vec![Action::ReplaceExterior { slot: 109, plane: -1 }, wait(3.0)]
        }
        (1139, 1) => vec![Action::Message { line: 0 }, wait(3.0)],
        // SPEC-0041 1414 teleports 0 and 1 (kind 13): value 50 or 10 for eW 159 and the exterior
        // replacement to eW 159 (pack 1358).
        (1414, 0 | 1) => {
            state.blackboard.insert((2, 159), if door == 0 { 50 } else { 10 });
            vec![Action::ReplaceExterior { slot: 159, plane: -1 }]
        }
        // SPEC-0037 1299 teleports: with 248 set, value 80 for eW 154 (index 0) or 155 (index 1)
        // and the replacement to it; with 248 clear index 0 shows line 0, and index 1 sets 243 and
        // goes to eW 154 as well.
        (1299, 0) if !state.flag(248) => vec![Action::Message { line: 0 }],
        (1299, 0 | 1) => {
            let w = if door == 1 && state.flag(248) { 155 } else { 154 };
            if door == 1 && !state.flag(248) {
                state.set(243, true);
            }
            state.blackboard.insert((2, w), 80);
            vec![Action::ReplaceExterior { slot: w, plane: -1 }]
        }
        // SPEC-0037 1152 teleports 1 and 0 (kind 13): value 90 for eW 106 or 126 and the exterior
        // replacement to that world.
        (1152, 0 | 1) => {
            let w = if door == 1 { 106 } else { 126 };
            state.blackboard.insert((2, w), 90);
            vec![Action::ReplaceExterior { slot: w, plane: -1 }]
        }
        // SPEC-0037 1429 teleport 0 (kind 13): value 60 for eW 151 and the leave to eW 156.
        (1429, 0) => {
            state.blackboard.insert((2, 151), 60);
            vec![Action::LeaveInterior { slot: 156 }]
        }
        // SPEC-0037 783 to 789 teleport 0 (kind 13): the value for eW 31, the leave to eW 44,
        // then 3 s.
        (783 | 786 | 787 | 788 | 789, 0) => {
            let (v, _) = eastside_interior(pack).unwrap_or_default();
            state.blackboard.insert((2, 31), v);
            vec![Action::LeaveInterior { slot: 44 }, wait(3.0)]
        }
        // SPEC-0044 1017 teleports 0 and 1: value 60 or 50 for eW 81 (type 2) and the exterior
        // load to eW 81 (pack 991).
        (1017, 0 | 1) => {
            state.blackboard.insert((2, 81), if door == 0 { 60 } else { 50 });
            vec![Action::ReplaceExterior { slot: 81, plane: -1 }]
        }
        // SPEC-0045 991 teleports 0 to 7: a type-2 value for eW 81 (10 to 80 by index), the
        // transition (interiors eW 84 to 87 and 89; exterior loads to eW 88, also posting the
        // value for eW 88, and to eW 63), then 3 s.
        (991, 0..=7) => {
            let v = 10 * (door as i32 + 1);
            state.blackboard.insert((2, 81), v);
            let go = match door {
                0..=3 => Action::EnterInterior { slot: 84 + door },
                4 | 5 => {
                    state.blackboard.insert((2, 88), v);
                    Action::ReplaceExterior { slot: 88, plane: -1 }
                }
                6 => Action::EnterInterior { slot: 89 },
                _ => Action::ReplaceExterior { slot: 63, plane: -1 },
            };
            vec![go, wait(3.0)]
        }
        // SPEC-0039 880 teleport 0 (handler 6): shut with line 20 until 296 (AI 21 to 23 dead);
        // then value 80 for eW 81 (type 2) and the replacement to eW 81 (pack 991); 3 s.
        (880, 0) if !state.flag(296) => vec![Action::Message { line: 20 }, Action::Sleep { ticks: 30 }, wait(3.0)],
        (880, 0) => {
            state.blackboard.insert((2, 81), 80);
            vec![Action::ReplaceExterior { slot: 81, plane: -1 }, wait(3.0)]
        }
        // SPEC-0045 1217 teleports (context 0, on contact): index 0 routine 13, value 50 for eW
        // 68 and the exterior replacement to eW 68 (pack 909); index 1 routine 14, value 2 for eW
        // 129 under types 2 and 3, and the replacement to eW 129 (pack 1184) with argument 2.
        (1217, 0) => {
            state.blackboard.insert((2, 68), 50);
            vec![Action::ReplaceExterior { slot: 68, plane: -1 }]
        }
        (1217, 1) => {
            state.blackboard.insert((2, 129), 2);
            state.blackboard.insert((3, 129), 2);
            vec![Action::ReplaceExterior { slot: 129, plane: 2 }]
        }
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
        // SPEC-0045 1023 teleports.
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
        // Index 0: the castle (eW 106, pack 1098) by flags 150, 11 and 15; index 9 the dark
        // village (eW 114) with flag 13; index 10 the sewer entrance (eW 109) with flag 35 and
        // the type-2 value 20 for eW 109 and eW 90. Each load is followed by Wait(3 s).
        (1023, 0) => {
            let (f150, f11, f15) = (state.flag(150), state.flag(11), state.flag(15));
            if (f150 && !f11) || (!f150 && f15) || (f150 && f11 && f15) {
                vec![Action::ReplaceExterior { slot: 106, plane: -1 }, wait(3.0)]
            } else if !f15 {
                vec![Action::Message { line: 23 }]
            } else {
                Vec::new()
            }
        }
        (1023, 9) if !state.flag(13) => vec![Action::Message { line: 3 }],
        (1023, 9) => vec![Action::ReplaceExterior { slot: 114, plane: -1 }, wait(3.0)],
        (1023, 10) if !state.flag(35) => vec![Action::Message { line: 6 }],
        (1023, 10) => {
            state.blackboard.insert((2, 109), 20);
            state.blackboard.insert((2, 90), 20);
            vec![Action::ReplaceExterior { slot: 109, plane: -1 }, wait(3.0)]
        }
        // SPEC-0045 1163: index 0 value 100 for eW 106 and the castle; index 1 unless s is 30,
        // value 50 for eW 92 and the courtyard, then Wait(3 s); with s 30 line 13.
        (1163, 0) => {
            state.blackboard.insert((2, 106), 100);
            vec![Action::ReplaceExterior { slot: 106, plane: -1 }]
        }
        (1163, 1) if castle_stage(state) == 30 => vec![Action::Message { line: 13 }],
        (1163, 1) => {
            state.blackboard.insert((2, 92), 50);
            vec![Action::ReplaceExterior { slot: 92, plane: -1 }, wait(3.0)]
        }
        // SPEC-0044 1056 teleports: index 1 to eW 122 (value 50) and index 0 to eW 106 (value
        // 40) unless flag 30 is set (line 71); index 2 to eW 106 with value 30.
        (1056, 0 | 1) if state.flag(30) => vec![Action::Message { line: 71 }],
        (1056, 1) => {
            state.blackboard.insert((2, 122), 50);
            vec![Action::ReplaceExterior { slot: 122, plane: -1 }]
        }
        (1056, 0) => {
            state.blackboard.insert((2, 106), 40);
            vec![Action::ReplaceExterior { slot: 106, plane: -1 }]
        }
        (1056, 2) => {
            state.blackboard.insert((2, 106), 30);
            vec![Action::ReplaceExterior { slot: 106, plane: -1 }]
        }
        // SPEC-0045 1098 transitions (s is the castle stage: the highest of flags 31 to 21 set).
        (1098, 0) => vec![Action::EnterInterior { slot: 121 }],
        (1098, 1) => vec![Action::EnterInterior { slot: 120 }],
        (1098, 3) => vec![Action::Message { line: 17 }],
        (1098, 2) if castle_stage(state) == 30 => vec![Action::Message { line: 17 }],
        (1098, 2) => {
            state.blackboard.insert((2, 119), 90);
            vec![Action::ReplaceExterior { slot: 119, plane: -1 }]
        }
        (1098, 4 | 7) if castle_stage(state) == 30 => vec![Action::Message { line: 16 }],
        (1098, 4 | 7) => {
            state.blackboard.insert((2, 92), if door == 4 { 30 } else { 40 });
            vec![Action::ReplaceExterior { slot: 92, plane: -1 }]
        }
        (1098, 5) => {
            state.blackboard.insert((2, 122), 100);
            vec![Action::ReplaceExterior { slot: 122, plane: -1 }]
        }
        (1098, 6) if castle_stage(state) == 30 || state.word(1098, 0) != 0 => vec![Action::Message { line: 16 }],
        (1098, 6) => vec![Action::ReplaceExterior { slot: 90, plane: -1 }],
        // SPEC-0032 955: message 40 for eW 68, leave to eW 5.
        (955, 0) => {
            state.blackboard.insert((2, 68), 40);
            vec![Action::LeaveInterior { slot: 5 }]
        }
        _ => Vec::new(),
    }
}

/// Whether a module's object calls are reproduced: its objects then follow the draw gate (hidden
/// until a script initialises them; SPEC-0028 section 10). Other packs draw their objects at
/// rest until their calls are built.
pub fn objects_scripted(pack: u32) -> bool {
    matches!(pack, 909 | 951 | 678 | 686 | 691 | 784 | 796 | 799 | 880 | 1173 | 886 | 1230 | 1369 | 1023 | 1133 | 1148 | 1150 | 1171 | 1172 | 1184 | 1056 | 1163 | 1244)
}

/// Run a pack's Init (once per load of the pack): the calls with world effects described so far.
pub fn on_init(pack: u32, state: &mut GameState) -> Vec<Action> {
    let reg = |index, routine| Action::RegisterTrigger { index, routine };
    match pack {
        // SPEC-0037 796 Init: objects 1, 0 and 17 off; S1 on trigger 1 and S0 on trigger 0.
        796 => vec![Action::InitObjOn { obj: 1, end: false }, Action::InitObjOn { obj: 0, end: false }, Action::InitObjOn { obj: 17, end: false }, reg(1, 1), reg(0, 0)],
        // SPEC-0037 1369 Init: objects 0 and 1 off, value 10 (type 2) for eW 157, T1 on trigger 1
        // and T0 on trigger 0.
        1369 => {
            state.blackboard.insert((2, 157), 10);
            vec![Action::InitObjOn { obj: 0, end: false }, Action::InitObjOn { obj: 1, end: false }, reg(1, 1), reg(0, 0)]
        }
        // SPEC-0037 1238 Init and thread T1: with flag 50 slot 0 (table A) loads set 3 and slot 1
        // (table B) set 1 (567 set, 568 clear) or set 2; without it slot 0 loads set 0; then with
        // 578 set and 579 clear slot 2 (table C) loads set 4 (region 1239 + set). Thread T2 is in
        // Update; the pickup callback is on_pickup.
        1238 => {
            let load = |slot: u8, set: u32| Action::LoadRegion { slot, region: Some(1239 + set) };
            let mut v = if state.flag(50) {
                vec![load(0, 3), load(1, if state.flag(567) && !state.flag(568) { 1 } else { 2 })]
            } else {
                vec![load(0, 0)]
            };
            if state.flag(578) && !state.flag(579) {
                v.push(load(2, 4));
            }
            v
        }
        // SPEC-0037 947 Init: thread G loads slot 0 with set 0 (region 948) while 375 is clear
        // (its AI-3 watch is in Update); S0, S1 and S2 on triggers 0 to 2; teleport 0 (kind 0)
        // does nothing. Thread H (flag 1: line 3 below 20 % hit points) and
        // RegisterMovableBlockSoundIndex are not reproduced.
        947 => {
            let mut v = vec![reg(0, 0), reg(1, 1), reg(2, 2)];
            if !state.flag(375) {
                v.push(Action::LoadRegion { slot: 0, region: Some(948) });
            }
            v
        }
        // SPEC-0037 1299 Init: S0 on trigger 0; the teleports and the arrival are fixed rules.
        1299 => vec![reg(0, 0)],
        // SPEC-0037 809 Init (play mode: the debug writes do not run): thread D loads slot 0 with
        // set 4 to 0 by the first set flag of 58, 57, 56, 55, 54 (region 810 + set).
        809 => [(58, 4), (57, 3), (56, 2), (55, 1), (54, 0)].into_iter().find(|&(f, _)| state.flag(f)).map(|(_, j)| Action::LoadRegion { slot: 0, region: Some(810 + j) }).into_iter().collect(),
        // SPEC-0043 1358 Init: the sign routine (20) on triggers 5 and 7, the slot swap (19) on 3
        // and 4; with 102 the latch lines (21, 22) on 1 and 2 and the prize wheel (56) on 0; the
        // slot thread loads slot 0 with set 1 (region 1360) under 102, else set 0 (1359); object 0
        // and the wheel's new pattern. Teleports, the proximity routines and the arrival are
        // fixed rules.
        1358 => {
            let mut v = vec![reg(5, 20), reg(7, 20), reg(3, 19), reg(4, 19)];
            if state.flag(102) {
                v.extend([reg(1, 21), reg(2, 22), reg(0, 56)]);
            }
            v.extend([
                Action::LoadRegion { slot: 0, region: Some(if state.flag(102) { 1360 } else { 1359 }) },
                Action::InitObjOn { obj: 0, end: false },
                Action::WheelPattern,
            ]);
            v
        }
        // SPEC-0044 745 Init: value 10 for eW 20; four or more of item 82 with 567 set sets 568;
        // the slot thread: with 76 set slot 1 loads set 1 (region 747) while 569 is clear; with 76
        // clear slot 0 loads set 0 (746) and slot 1 set 1 (while 569 is clear, as slot store B
        // exists only then); the normal camera.
        745 => {
            state.blackboard.insert((2, 20), 10);
            if state.flag(567) && state.items.get(&82).copied().unwrap_or(0) >= 4 {
                state.set(568, true);
            }
            let mut v = Vec::new();
            if !state.flag(76) {
                v.push(Action::LoadRegion { slot: 0, region: Some(746) });
            }
            if !state.flag(569) {
                v.push(Action::LoadRegion { slot: 1, region: Some(747) });
            }
            v.push(Action::ReturnCamera { preset: -1, release: true });
            v
        }
        // SPEC-0043 1173 Init: object 1 off; routine 2 on trigger 1 and routine 3 on trigger 0;
        // teleport 0 (kind 13), entity 0's routine and the arrival are fixed rules.
        1173 => vec![Action::InitObjOn { obj: 1, end: false }, reg(1, 2), reg(0, 3)],
        // SPEC-0039 1405, 1407, 1417, 1419 and 1425 Init: their blackboard values (1417: 20 for
        // eW 159; 1425: 50 for eW 160); 1407 sets flag 100 first; the thread loads slot 0 with set
        // 0 (region P + 1) and returns the camera under each pack's gate (104, always, 102, 100,
        // 106). The callbacks are registered under the same gates (gated in their routines).
        1405 | 1407 | 1417 | 1419 | 1425 => {
            match pack {
                1407 => state.set(100, true),
                1417 => {
                    state.blackboard.insert((2, 159), 20);
                }
                1425 => {
                    state.blackboard.insert((2, 160), 50);
                }
                _ => {}
            }
            if gate_1405(pack, state) {
                vec![Action::LoadRegion { slot: 0, region: Some(pack + 1) }, Action::ReturnCamera { preset: -1, release: true }]
            } else {
                Vec::new()
            }
        }
        // SPEC-0039 1139 Init: object 0 at its end (flag 35) or start; with 304 clear, 304 and 13
        // set; value 10 for eW 109. The proximity routines, the teleports (index 1's context by
        // flag 35) and the arrival are fixed rules.
        1139 => {
            if !state.flag(304) {
                state.set(304, true);
                state.set(13, true);
            }
            state.blackboard.insert((2, 109), 10);
            vec![Action::InitObjOn { obj: 0, end: state.flag(35) }]
        }
        // SPEC-0039 732 Init: handler 1 on the triggers of the kind-0 group 2 (0, 1 and 2); thread
        // 4 loads slot 0 with set 0 (region 733) when 561 is set and 562 clear, or set 1 (734)
        // when both are set; its watch of AI 0 is in Update.
        732 => {
            let mut v = vec![reg(0, 1), reg(1, 1), reg(2, 1)];
            match (state.flag(561), state.flag(562)) {
                (true, false) => v.push(Action::LoadRegion { slot: 0, region: Some(733) }),
                (true, true) => v.push(Action::LoadRegion { slot: 0, region: Some(734) }),
                _ => {}
            }
            v
        }
        // SPEC-0039 793 Init: thread 3 loads slot 0 with set 0 (region 794) under 417, else set 1
        // (795).
        793 => vec![Action::LoadRegion { slot: 0, region: Some(if state.flag(417) { 794 } else { 795 }) }],
        // SPEC-0039 874 Init: thread 3 (Medium in the spec) is taken as set 1 (region 876) under
        // flag 58, else set 0 (875); the further loads by 291 and 290 are not reproduced.
        874 => vec![Action::LoadRegion { slot: 0, region: Some(if state.flag(58) { 876 } else { 875 }) }],
        // SPEC-0041 742 Init: thread A loads slot 0 with set 0 (region 743), and with 76, 561 set
        // and 562 clear slot 1 with set 1 (744); thread B's watch of entity 6 is in Update; P7 is
        // a fixed rule.
        742 => {
            let mut v = vec![Action::LoadRegion { slot: 0, region: Some(743) }];
            if state.flag(76) && state.flag(561) && !state.flag(562) {
                v.push(Action::LoadRegion { slot: 1, region: Some(744) });
            }
            v
        }
        // SPEC-0041 1414 Init: the thread loads slot 0 with set 1 (region 1416) under 102 and
        // returns the camera; without 102 set 0 (1415), 3 s, entities 0 and 1 agro (their
        // DisableInvisibility calls are not built) and the camera. The four proximity routines
        // (registered under 102), the teleports and the arrival are fixed rules.
        1414 => {
            if state.flag(102) {
                vec![Action::LoadRegion { slot: 0, region: Some(1416) }, Action::ReturnCamera { preset: -1, release: true }]
            } else {
                vec![Action::LoadRegion { slot: 0, region: Some(1415) }, Action::WaitLoads, wait(3.0), Action::Agro { entity: 0 }, Action::Agro { entity: 1 }, Action::ReturnCamera { preset: -1, release: true }]
            }
        }
        // SPEC-0041 1409, 1421 and 1423 Init: their blackboard value (30 for world 160, 30 for
        // world 159, 40 for world 160); the callbacks are registered under flag 106 (1409, 1423)
        // or 102 (1421) and gated in their routines; 1409's thread loads slot 0 with set 0
        // (region 1410) under 106 and returns the camera; 1421's and 1423's thread loads are not
        // given (partly known), only their camera returns. The idle loops and the shared
        // entity-control routine are not reproduced.
        1409 | 1421 | 1423 => {
            let (w, v) = match pack {
                1421 => (159, 30),
                1423 => (160, 40),
                _ => (160, 30),
            };
            state.blackboard.insert((2, w), v);
            let mut out = Vec::new();
            if pack == 1409 && state.flag(106) {
                out.push(Action::LoadRegion { slot: 0, region: Some(1410) });
            }
            out.push(Action::ReturnCamera { preset: -1, release: true });
            out
        }
        // SPEC-0043 750 Init: the control thread loads slot 0 with set 0 (region 751) while flag
        // 76 is clear and returns the normal camera. Its blackboard message's values are not
        // given; the proximity and on-enter routines are fixed rules. The shared entity-control
        // routine (Update) is not reproduced.
        750 => {
            let mut v: Vec<Action> = (!state.flag(76)).then_some(Action::LoadRegion { slot: 0, region: Some(751) }).into_iter().collect();
            v.push(Action::ReturnCamera { preset: -1, release: true });
            v
        }
        // SPEC-0037 1225 and 1220 Init: thread A loads slot 0 with set 3, 2, 1 or 0 by the first
        // set flag of 84, 83, 82, 81 (region P + 1 + set), nothing otherwise; 1220 also
        // registers S0 on trigger 0.
        1225 | 1220 => {
            let mut v: Vec<Action> = [(84, 3), (83, 2), (82, 1), (81, 0)].into_iter().find(|&(f, _)| state.flag(f)).map(|(_, j)| Action::LoadRegion { slot: 0, region: Some(pack + 1 + j) }).into_iter().collect();
            if pack == 1220 {
                v.push(reg(0, 0));
            }
            v
        }
        // SPEC-0037 1259 Init: routine S on triggers 0 to 49 (the slot table's entries).
        1259 => (0..50).map(|i| reg(i, 0)).collect(),
        // SPEC-0037 1253 Init: S0 on trigger 0; the arrival is a fixed rule.
        1253 => vec![reg(0, 0)],
        // SPEC-0037 799 Init: objects 1, 2 and 0 off; S0, S1 (empty) and S3 on triggers 0, 1, 3.
        799 => vec![
            Action::InitObjOn { obj: 1, end: false },
            Action::InitObjOn { obj: 2, end: false },
            Action::InitObjOn { obj: 0, end: false },
            reg(0, 0),
            reg(3, 3),
        ],
        // SPEC-0037 1152 Init: thread S loads slot 0 with set 2 (flag 31), 1 (30) or 0 (region
        // 1153 + set); the proximity routine, teleports 0 and 1 (kind 13) and the arrival are
        // fixed rules.
        1152 => {
            let set = if state.flag(31) { 2 } else if state.flag(30) { 1 } else { 0 };
            vec![Action::LoadRegion { slot: 0, region: Some(1153 + set) }]
        }
        // SPEC-0037 784 Init: with 543 clear object 2 off and S1 on trigger 1, else object 3 off;
        // thread K: with 590 set and 591 clear slot 0 loads set 0 (region 785) and entity 0 is
        // undamageable (undamageable()).
        784 => {
            let mut v = if state.flag(543) { vec![Action::InitObjOn { obj: 3, end: false }] } else { vec![Action::InitObjOn { obj: 2, end: false }, reg(1, 1)] };
            if state.flag(590) && !state.flag(591) {
                v.push(Action::LoadRegion { slot: 0, region: Some(785) });
            }
            v
        }
        // SPEC-0037 1251 Init: the proximity and on-enter callbacks; thread A's slot load (with
        // activate false) and its ItemActivate calls are not reproduced (the item slot is
        // Medium in the spec); the item-ask callback never fires.
        1251 => Vec::new(),
        // SPEC-0037 855, 856, 857, 858 and 1429 Init: the entry thread's line (argument 0: in
        // entity 0's conversation frame; 857's argument -1: none) and the normal camera. 856's
        // item-ask callbacks never fire (SPEC-0030 row 257); the buy callbacks are on_buy.
        855 | 856 | 857 | 858 | 1429 => {
            let f = |n| state.flag(n);
            let line = match pack {
                855 if f(463) => 0,
                855 if f(462) && state.has_item(22) => 2,
                855 if f(462) => 3,
                855 if f(59) => 5,
                855 if f(56) => 6,
                855 => 8,
                1429 if f(93) => 0,
                1429 if f(92) => 3,
                1429 => 6,
                _ if f(59) => 0,
                _ if f(56) => 2,
                _ => 4,
            };
            let say = if pack == 857 { Action::Message { line } } else { Action::Converse { entity: 0, line } };
            vec![say, Action::ReturnCamera { preset: -1, release: true }]
        }
        // SPEC-0039 984 Init: the proximity and on-enter callbacks are fixed rules; the four
        // item-ask callbacks never fire (SPEC-0030 row 257); thread 7 returns the normal camera.
        984 => vec![Action::ReturnCamera { preset: -1, release: true }],
        // SPEC-0044 1017 Init: flag 85; thread K1 loads slot 0 with set 1 (flag 86) or 0 (region
        // 1018 + set); 297 set when clear (thread K2, the opening cutscene, is not reproduced:
        // R-0056; thread K3 neither). Proximity routines, teleports 0 and 1 (context 13) and the
        // arrival are fixed rules.
        1017 => {
            state.set(85, true);
            state.set(297, true);
            vec![Action::LoadRegion { slot: 0, region: Some(1018 + state.flag(86) as u32) }]
        }
        // SPEC-0045 991 Init: flag 85; trigger routine A (8) on triggers 4 and 5 and on the
        // kind-7 group 0's instance trigger (the crowd zones); thread 2 loads slot 0 with set 3
        // (region 995) under flag 86, else set 8 (1000). The teleports (kind 13) and the
        // proximity routines are fixed rules (teleport_context, proximity_routine).
        991 => {
            state.set(85, true);
            vec![
                reg(4, 8),
                reg(5, 8),
                Action::RegisterInstance { routine: 8, trigger: 0, group: 0 },
                Action::LoadRegion { slot: 0, region: Some(if state.flag(86) { 995 } else { 1000 }) },
            ]
        }
        // SPEC-0039 886 Init: handler 1 on trigger 5, 2 on 2, 4 on 0; object 3 off while 250 is
        // clear; 249 set when 248 is set, 249 clear and movie 30 not played (movies are not
        // built, so it counts as not played); thread 3 loads slot 0 with set 3, 2, 1 or 0 by
        // the first set flag of 52, 51, 50 and 49 (region 887 + set), and nothing otherwise.
        // SetWorldCollisMask is not built.
        886 => {
            let mut v = vec![reg(5, 1), reg(2, 2)];
            if !state.flag(250) {
                v.push(Action::InitObjOn { obj: 3, end: false });
            }
            if state.flag(248) && !state.flag(249) {
                state.set(249, true);
            }
            if let Some(set) = [(52, 3), (51, 2), (50, 1), (49, 0)].into_iter().find(|&(f, _)| state.flag(f)).map(|(_, j)| j) {
                v.push(Action::LoadRegion { slot: 0, region: Some(887 + set) });
            }
            v.push(reg(0, 4));
            v
        }
        // SPEC-0039 1230 Init: slot 0 and thread 1; handler 2 on triggers 0, 2 and 1, handler 3
        // on 5; with flag 20 or 19 handlers 5 and 6 on 3 and 4; object 0 off with 64 set and 248
        // clear. SetWorldCollisMask and SetWorldDisplayMask under 248 are not built.
        1230 => {
            let mut v = vec![reg(0, 2), reg(2, 2), reg(1, 2), reg(5, 3)];
            if state.flag(20) || state.flag(19) {
                v.extend([reg(3, 5), reg(4, 6)]);
            }
            if state.flag(64) && !state.flag(248) {
                v.push(Action::InitObjOn { obj: 0, end: false });
            }
            v.extend(slots_1230(state));
            v
        }
        // SPEC-0039 880 Init: objects 0 and 1 off; slots 0 (sets 0 to 4) and 1 (set 4); triggers
        // 5 to 7 handler 1, 0 handler 2, 1 handler 3, 2 handler 9; thread 4 (the flag-chosen
        // set); the instance trigger of group 2 (handler 5); teleport 0 (context 13) and the
        // map shop on entities 3 and 7 are fixed rules here (teleport_context,
        // proximity_routine). SetWorldCollisMask(1) is not built.
        880 => {
            let mut v = vec![Action::InitObjOn { obj: 0, end: false }, Action::InitObjOn { obj: 1, end: false }];
            v.extend([reg(5, 1), reg(6, 1), reg(7, 1), reg(0, 2), reg(1, 3), reg(2, 9)]);
            v.push(Action::RegisterInstance { routine: 5, trigger: 0, group: 2 });
            v.push(slots_880(state));
            v
        }
        // SPEC-0042 1171 Init: objects 0, 1, 2, 11, 13, 15, 17, 20, 23 and 26 off; the levers on
        // triggers 5 to 7 (routines 5 to 7), T0 and T1 on 0 and 1 (routines 0 and 1); the
        // switch handler on 29 to 32 (replaced below, never run); the drain-flag loop's
        // SetBreakableActive calls are not reproduced (breakables, R-0040); then group type 12
        // (which replaces triggers 29 to 32). AddAnimObjSound is not built.
        1171 => {
            let mut v: Vec<Action> = [0, 1, 2, 11, 13, 15, 17, 20, 23, 26].into_iter().map(|obj| Action::InitObjOn { obj, end: false }).collect();
            v.extend([reg(5, 5), reg(6, 6), reg(7, 7), reg(29, 29), reg(30, 29), reg(31, 29), reg(32, 29), reg(0, 0), reg(1, 1)]);
            v.push(Action::RegisterGroupType { kind: 12 });
            v
        }
        // SPEC-0039 1244 Init: SetGameState(271, 1); with 273 clear objects 5, 0, 1 and 4 off and
        // handlers 1 to 5 on triggers 0, 1, 2, 4 and 3.
        1244 => {
            state.set(271, true);
            if state.flag(273) {
                Vec::new()
            } else {
                let mut v: Vec<Action> = [5, 0, 1, 4].into_iter().map(|obj| Action::InitObjOn { obj, end: false }).collect();
                v.extend([reg(0, 1), reg(1, 2), reg(2, 3), reg(4, 4), reg(3, 5)]);
                v
            }
        }
        // SPEC-0045 909 Init: SetWhinnyPenmesh first (inner 0, entrance 5, locator 8); script
        // triggers 0 to 3 routine 10, 5 routine 11, 8 routine 7, and 7 routine 4 while 117 is
        // clear.
        909 => {
            let mut out = vec![Action::WhinnyPen { inner: 0, entrance: 5, locator: 8 }];
            // The object helper (every load): objects 18, 17, 20 and 19 and 7 and 8 at their
            // start pose, effects 3 and 4 stopped; by flag 122 objects 15 and 16 (start, with
            // object 4 and the word of Update's flag-122 step set; or end, with object 5); by
            // flag 110 objects 10 and 11 (start, with object 3; or end, with object 2 and the
            // word of Update's flag-110 step set, so that step does not run again).
            let at = |obj, end| Action::InitObjOn { obj, end };
            out.extend([at(18, false), at(17, false), at(20, false), at(19, false)]);
            out.extend([Action::StopEffect { slot: 3 }, Action::StopEffect { slot: 4 }, at(7, false), at(8, false)]);
            if state.flag(122) {
                out.extend([at(15, true), at(16, true), at(5, false)]);
                state.set_word(pack, 6, 0);
            } else {
                out.extend([at(15, false), at(16, false), at(4, false)]);
                state.set_word(pack, 6, 1);
            }
            if state.flag(110) {
                out.extend([at(10, true), at(11, true), at(2, false)]);
                state.set_word(pack, 5, 1);
            } else {
                out.extend([at(10, false), at(11, false), at(3, false)]);
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
            // Init step 4: object 0 at its start pose, or at its end with flag 121 set.
            let mut out = vec![Action::LockChest { id: 0, key: 15, message: 124 }, Action::InitObjOn { obj: 0, end: state.flag(121) }];
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
        1184 => woods_init(state),
        1098 => castle_init(state),
        1056 => yard_init(state),
        1163 => kitchen_init(state),
        1217 => burrows_init(state),
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
    // The prize wheel (routine 56) when flags 64 and 11 are clear (the walk-up greetings 37 to
    // 40 are not built).
    if !state.flag(64) && !state.flag(11) {
        out.extend([reg(37, 54), reg(39, 54), reg(38, 55), reg(40, 55), reg(36, 56)]);
    }
    // Objects: 0 always; 5 on (end pose) or off by flags 150, 10 and 15; with 64 set and 248
    // clear, 1, 2 and 3 off. Then the wheel's new pattern (the three shuffle entries).
    out.push(Action::InitObjOn { obj: 0, end: false });
    let (f150, f10, f15) = (state.flag(150), state.flag(10), state.flag(15));
    if (f150 && !f10) || (!f150 && f15) || (f150 && f10 && f15) {
        out.push(Action::InitObjOn { obj: 5, end: true });
    } else if !f15 && (!f150 || f10) {
        out.push(Action::InitObjOn { obj: 5, end: false });
    }
    if state.flag(64) && !state.flag(248) {
        out.extend([1, 2, 3].map(|obj| Action::InitObjOn { obj, end: false }));
    }
    out.push(Action::WheelPattern);
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
    // Object 4 is activated and played (speed 0.45, wait); its reset to time 0 at locator 498
    // (a MoveObj call) is not built.
    v.extend([MoveEntity { entity: 1, locator: 496, face: true }, InitObjOn { obj: 4, end: false }, PlayAnim { obj: 4, speed: 0.45, wait: true }]);
    v.extend([PlaySound { cue: 12, locator: None }, MoveHero { locator: 493 }]);
    v.extend(cut(17));
    v.push(wait(1.0));
    v.extend(cut(19));
    v.extend([Message { line: 25 }, cam(20), Converse { entity: 1, line: 26 }]);
    v.extend(cut(18));
    v.extend([MoveEntity { entity: 0, locator: 499, face: true }, wait(0.2), walk(0, 500), wait(0.6), DeactivateObj { obj: 4 }, cam(21), Converse { entity: 1, line: 27 }]);
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

/// The init callback of a registered script-group type for one entry (its objects in order).
pub fn group_init(pack: u32, kind: u32, objects: &[usize]) -> Vec<Action> {
    match (pack, kind, objects) {
        // SPEC-0048 / SPEC-0042 1171 init routine G: A off, its time r / 100, looped at 1.0.
        (1171, 12, &[a, ..]) => vec![Action::InitObjOn { obj: a, end: false }, Action::SetObjTimeRandom { obj: a }, Action::LoopAnim { obj: a, speed: 1.0 }],
        _ => Vec::new(),
    }
}

/// The run callback of a registered script-group type, for a fire of the entry's first trigger
/// (the callback thread is keyed by the trigger, so it starts on the entering fire).
pub fn group_run(pack: u32, kind: u32, call: TriggerCall, objects: &[usize]) -> Vec<Action> {
    match (pack, kind, objects) {
        // SPEC-0048 / SPEC-0042 1171 run routine R: A off; B shown and played (wait); one tick
        // at a time while the hero is inside; B off; C shown at time 0 and played (wait); C off;
        // A shown and looped at 1.0.
        (1171, 12, &[a, b, c, ..]) if call.state == 1 && call.object => vec![
            Action::DeactivateObj { obj: a },
            Action::InitObjOn { obj: b, end: false },
            Action::PlayAnim { obj: b, speed: 1.0, wait: true },
            Action::WaitInside { index: call.index },
            Action::DeactivateObj { obj: b },
            Action::InitObjOn { obj: c, end: false },
            Action::SetObjTime { obj: c, seconds: 0.0 },
            Action::PlayAnim { obj: c, speed: 1.0, wait: true },
            Action::DeactivateObj { obj: c },
            Action::InitObjOn { obj: a, end: false },
            Action::LoopAnim { obj: a, speed: 1.0 },
        ],
        _ => Vec::new(),
    }
}

/// SPEC-0039 880 thread 4: slot 0 loads the set of the first set flag of 52, 51, 50 and 49
/// (sets 3, 2, 1, 0), else set 0 (region P + 1 + set).
fn slots_880(state: &GameState) -> Action {
    let set = [(52, 3), (51, 2), (50, 1), (49, 0)].into_iter().find(|&(f, _)| state.flag(f)).map_or(0, |(_, j)| j);
    Action::LoadRegion { slot: 0, region: Some(881 + set) }
}

/// The treasure-map shops of SPEC-0039 880 and 1230: (map, prompt, price) in offer order.
fn map_shop(pack: u32) -> &'static [(u32, u16, i32)] {
    match pack {
        880 => &[(1, 6, 250), (8, 7, 250), (30, 8, 250), (27, 9, 300), (18, 10, 400), (3, 17, 300), (6, 18, 300), (11, 19, 400)],
        1230 => &[(1, 6, 250), (8, 7, 250), (30, 8, 250), (27, 9, 300), (18, 10, 500), (10, 17, 300)],
        _ => &[],
    }
}

/// SPEC-0039 1230 thread 1: slot 0 loads set 6 with flag 248; else set 3, 2 or 1 by flags 19,
/// 18 and 52, or set 0. With flag 20 set the set depends on two flags the spec does not name
/// (R-0054), and nothing is loaded.
fn slots_1230(state: &GameState) -> Option<Action> {
    let set = if state.flag(248) {
        6
    } else if state.flag(20) {
        return None;
    } else {
        [(19, 3), (18, 2), (52, 1)].into_iter().find(|&(f, _)| state.flag(f)).map_or(0, |(_, j)| j)
    };
    Some(Action::LoadRegion { slot: 0, region: Some(1231 + set) })
}

/// SPEC-0044 1017 dialogue sets. Set D (Guildmaster Marl, routine 0): the first set flag of 593,
/// 590, 589, 587, 586, 585, 299 picks its three lines in turn; 298 gives line 98 (its dialogue
/// command gives item 23) and sets 299; 297 lines 99 to 101; else line 102 and 297. Sets A to C
/// (routines 2, 3, 1) keep only their flag sets: which line each branch shows is not given
/// (R-0056), so they show none.
fn guild_talk(routine: u8, s: &mut GameState) -> Vec<Action> {
    let f = |n: u16| s.flag(n);
    match routine {
        0 => {
            let key = (1017, 0);
            let mut rot = s.rotations.get(&key).copied().unwrap_or_default();
            let triple = |a: u16| cycle(&[a, a + 1, a + 2]);
            let list = [(593, triple(77)), (590, triple(80)), (589, triple(83)), (587, triple(86)), (586, triple(89)), (585, triple(92)), (299, triple(95))];
            let line = if let Some(l) = cascade(s, &mut rot, &list) {
                l
            } else if f(298) {
                s.set(299, true);
                rot.pick(7, &Lines::One(98))
            } else if f(297) {
                rot.pick(8, &triple(99))
            } else {
                s.set(297, true);
                rot.pick(9, &Lines::One(102))
            };
            s.rotations.insert(key, rot);
            line.map(|line| vec![Action::Talk { line }]).unwrap_or_default()
        }
        2 if !f(593) && !f(590) && !f(589) && !f(587) && f(586) => {
            s.set(587, true);
            Vec::new()
        }
        3 if !f(593) && !f(590) && f(589) => {
            s.set(590, true);
            Vec::new()
        }
        1 if !f(593) && !f(590) && !f(589) && !f(587) && !f(586) && !f(585) && f(299) => {
            s.set(585, true);
            Vec::new()
        }
        _ => Vec::new(),
    }
}

/// SPEC-0045 991 proximity routines (routine = the entity, or the first of a shared group).
fn lakeside_talk(routine: u8, s: &mut GameState) -> Vec<Action> {
    let key = (991, routine);
    let mut rot = s.rotations.get(&key).copied().unwrap_or_default();
    let hero = s.hero.selected.min(1) as u16;
    // Three lines in turn for the active hero: Tor's from t, Roberta's from r.
    let pair = |rot: &mut Rotation, t: u16, r: u16| rot.pick(hero, &cycle(&[0, 1, 2].map(|k| if hero == 1 { r + k } else { t + k })));
    let talk = |line: Option<u16>| line.map(|line| vec![Action::Talk { line }]).unwrap_or_default();
    let out = match routine {
        8 => talk(pair(&mut rot, 11, 14)),
        5 => talk(pair(&mut rot, 17, 20)),
        4 => talk(pair(&mut rot, 23, 26)),
        7 => talk(pair(&mut rot, 29, 32)),
        6 => talk(pair(&mut rot, 35, 38)),
        19 => talk(pair(&mut rot, 41, 44)),
        10 => talk(pair(&mut rot, 47, 50)),
        20 => talk(pair(&mut rot, 104, 107)),
        11 => talk(pair(&mut rot, 119, 122)),
        18 => talk(pair(&mut rot, 143, 146)),
        9 if s.flag(86) => talk(pair(&mut rot, 56, 59)),
        9 => talk(rot.pick(2, &cycle(&[53, 54, 55]))),
        16 if s.flag(86) => talk(pair(&mut rot, 83, 86)),
        16 => talk(rot.pick(2, &cycle(&[80, 81, 82]))),
        15 if s.flag(86) => talk(pair(&mut rot, 113, 116)),
        15 => talk(rot.pick(2, &cycle(&[110, 111, 112]))),
        // Entity 14: 417 set line 6; 416 clear sets it, line 10; with item 12 one is taken, 417
        // set, line 7; else 8 and 9 in turn.
        14 if s.flag(417) => talk(Some(6)),
        14 if !s.flag(416) => {
            s.set(416, true);
            talk(Some(10))
        }
        14 if s.has_item(12) => {
            s.take(12, 1);
            s.set(417, true);
            talk(Some(7))
        }
        14 => talk(rot.pick(3, &cycle(&[8, 9]))),
        // Entities 2, 1 and 3 (667 clear): their three lines in turn, then the prompt.
        2 | 1 | 3 => {
            let (first, prompt) = match routine {
                2 => (74, 77),
                1 => (137, 140),
                _ => (161, 164),
            };
            let mut v = talk(rot.pick(0, &cycle(&[first, first + 1, first + 2])));
            v.push(Action::Prompt { line: prompt, routine });
            v
        }
        // Entity 17, the map seller: offers for held maps are skipped (C 1 with map 21, 2 with
        // 17, 3 with 35); C 0 line 89 and C = 1; C 1 to 3 prompts 90 to 92; C 4 line 93, C = 1.
        17 => {
            let mut c = s.word(991, 1);
            for (k, map) in [(1, 21), (2, 17), (3, 35)] {
                if c == k && s.maps.contains(&map) {
                    c = k + 1;
                }
            }
            match c {
                0 => {
                    s.set_word(991, 1, 1);
                    talk(Some(89))
                }
                1..=3 => {
                    s.set_word(991, 1, c);
                    vec![Action::Prompt { line: 89 + c as u16, routine: 17 }]
                }
                _ => {
                    s.set_word(991, 1, 1);
                    talk(Some(93))
                }
            }
        }
        _ => Vec::new(),
    };
    s.rotations.insert(key, rot);
    out
}

/// SPEC-0059 section 7, the 1358 delivery giver (entity 20): 604 first (line 32); a taken task's
/// reminder (4 + k); else an offer from the list of open tasks with their prerequisites (none
/// for 1 and 2; 100 for 3 and 4; 106 for 5 and 6; 104 for 7 and 8) by r = Random(1, n) counted
/// from 0, so entry 0 is never offered and r = n is no offer (the clean behaviour of the draw
/// defect: no prompt); prompt 12 + k with k kept in word 1; n = 0 gives 33, 34 or 35.
fn delivery_talk(s: &mut GameState) -> Vec<Action> {
    let talk = |line| vec![Action::Talk { line }];
    if !s.flag(604) {
        s.set(604, true);
        return talk(32);
    }
    if let Some(k) = (1..=8u16).find(|&k| s.flag(604 + k) && !s.flag(678 + k)) {
        return talk(4 + k);
    }
    let pre = |k: u16| match k {
        1 | 2 => true,
        3 | 4 => s.flag(100),
        5 | 6 => s.flag(106),
        _ => s.flag(104),
    };
    let list: Vec<u16> = (1..=8u16).filter(|&k| !s.flag(678 + k) && pre(k)).collect();
    if list.is_empty() {
        return talk(32 + s.talk_roll.clamp(1, 3) as u16);
    }
    let n = list.len() as u32;
    let r = if n == 1 { 1 } else { 1 + s.talk_raw % n } as usize;
    match list.get(r) {
        Some(&k) => {
            s.set_word(1358, 1, k as i32);
            vec![Action::Prompt { line: 12 + k, routine: 1 }]
        }
        None => Vec::new(),
    }
}

/// SPEC-0044 745's item exchange: items 5, 4 and 3 (one of each) for line 16, flag 260 (if clear),
/// and flags 76 and 264. None when an item is missing.
fn exchange_745(s: &mut GameState) -> Option<Vec<Action>> {
    if ![5, 4, 3].iter().all(|&i| s.has_item(i)) {
        return None;
    }
    for i in [5, 4, 3] {
        s.take(i, 1);
    }
    for f in [260, 76, 264] {
        s.set(f, true);
    }
    Some(vec![Action::Talk { line: 16 }])
}

/// SPEC-0044 745 routines (entity 3: A, 0: B, 1: C, 2: D). "Cycle" steps three lines with one
/// counter per routine (word 10 + routine).
fn apothecary_talk(entity: u8, s: &mut GameState) -> Vec<Action> {
    let word = 10 + entity;
    let cyc = |s: &mut GameState, a: u16| {
        let c = s.word(745, word).rem_euclid(3);
        s.set_word(745, word, (c + 1) % 3);
        vec![Action::Talk { line: a + c as u16 }]
    };
    let f = |s: &GameState, n| s.flag(n);
    match entity {
        3 if f(s, 569) => cyc(s, 8),
        3 if f(s, 568) => {
            s.set(569, true);
            s.remove_all(82);
            vec![Action::Talk { line: 11 }]
        }
        3 if f(s, 567) => cyc(s, 12),
        3 if f(s, 76) => {
            s.set(567, true);
            vec![Action::Talk { line: 15 }]
        }
        3 if f(s, 75) => match exchange_745(s) {
            Some(v) => v,
            None if [5, 4, 3].iter().any(|&i| s.has_item(i)) => cyc(s, 17),
            None => cyc(s, 20),
        },
        3 => {
            s.set(260, true);
            vec![Action::Talk { line: 23 }]
        }
        0 => cyc(s, if f(s, 76) { 24 } else { 27 }),
        1 => cyc(s, if f(s, 76) { 30 } else { 33 }),
        2 => cyc(s, if f(s, 569) { 36 } else if f(s, 76) { 39 } else { 42 }),
        _ => Vec::new(),
    }
}

/// The registration gate of SPEC-0039 1405, 1407, 1417, 1419 and 1425.
fn gate_1405(pack: u32, s: &GameState) -> bool {
    match pack {
        1405 => s.flag(104),
        1407 => true,
        1417 => s.flag(102),
        1419 => s.flag(100),
        1425 => s.flag(106),
        _ => false,
    }
}

/// Their entity-0 routines: 1407 in full (the hand-in, then 8 to 10 with 102 and 104, else 11 to
/// 13); for 1417, 1419 and 1425 the item-33 hand-ins only (their other branches are Medium in
/// the spec and show nothing here); 1405's routine is not reproduced (Medium).
fn hall_talk(pack: u32, s: &mut GameState) -> Vec<Action> {
    if !gate_1405(pack, s) {
        return Vec::new();
    }
    let (need, done, line) = match pack {
        1407 => (607, 681, 14),
        1417 => (606, 680, 10),
        1419 => (608, 682, 13),
        1425 => (610, 684, 10),
        _ => return Vec::new(),
    };
    if s.flag(need) && !s.flag(done) {
        s.remove_all(33);
        s.set(done, true);
        return vec![Action::Talk { line }];
    }
    if pack != 1407 {
        return Vec::new();
    }
    let key = (pack, 0);
    let mut rot = s.rotations.get(&key).copied().unwrap_or_default();
    let line = if s.flag(102) && s.flag(104) { rot.pick(0, &cycle(&[8, 9, 10])) } else { rot.pick(1, &cycle(&[11, 12, 13])) };
    s.rotations.insert(key, rot);
    line.map(|line| vec![Action::Talk { line }]).unwrap_or_default()
}

/// SPEC-0039 1139, 732, 793 and 874 proximity routines.
fn town_talk(pack: u32, routine: u8, s: &mut GameState) -> Vec<Action> {
    let key = (pack, routine);
    let mut rot = s.rotations.get(&key).copied().unwrap_or_default();
    let talk = |line: Option<u16>| line.map(|line| vec![Action::Talk { line }]).unwrap_or_default();
    let triple = |a: u16| cycle(&[a, a + 1, a + 2]);
    // 1139's three-way rotation by flags 319 and 305 (branches 0 and 1), else `rest`.
    let by_flags = |s: &GameState, rot: &mut Rotation, a: u16, b: u16| -> Option<Option<u16>> {
        if s.flag(319) {
            Some(rot.pick(0, &triple(a)))
        } else if s.flag(305) {
            Some(rot.pick(1, &triple(b)))
        } else {
            None
        }
    };
    let out = match (pack, routine) {
        (1139, 4) => talk(by_flags(s, &mut rot, 7, 4).unwrap_or_else(|| rot.pick(2, &triple(1)))),
        (1139, 5) => talk(by_flags(s, &mut rot, 43, 40).unwrap_or_else(|| rot.pick(2, &triple(37)))),
        (1139, 2) => talk(by_flags(s, &mut rot, 52, 49).unwrap_or_else(|| if s.flag(678) { rot.pick(2, &triple(46)) } else { Some(55) })),
        // Entity 0: with 678 clear it is set with line 36; with item 23 every one is taken, value
        // 10 for eW 109 and movie 49, whose return world is eW 109 (pack 1129): the movie is not
        // built, so the replacement load is made directly; else lines 27 to 29.
        (1139, 1) => match by_flags(s, &mut rot, 33, 30) {
            Some(l) => talk(l),
            None if !s.flag(678) => {
                s.set(678, true);
                talk(Some(36))
            }
            None if s.has_item(23) => {
                s.remove_all(23);
                s.blackboard.insert((2, 109), 10);
                vec![Action::ReplaceExterior { slot: 109, plane: -1 }]
            }
            None => talk(rot.pick(2, &triple(27))),
        },
        // Entity 2: the one-time sale (word 0 holds the offered item + 1; 0 is none): Random(1,
        // 5) picks item 50, 152, 54, 55 or 56 with prompt 10 to 14; afterwards the rotation.
        (1139, 3) if s.word(pack, 0) == 0 => {
            let k = (s.talk_five.clamp(1, 5) - 1) as usize;
            let item = [50, 152, 54, 55, 56][k];
            s.set_word(pack, 0, item as i32 + 1);
            vec![Action::Prompt { line: 10 + k as u16, routine: 3 }]
        }
        (1139, 3) => talk(by_flags(s, &mut rot, 24, 21).unwrap_or_else(|| rot.pick(2, &triple(18)))),
        // 732 entity 1: lines 0 to 2 in turn with 562, else line 4.
        (732, 3) if s.flag(562) => talk(rot.pick(0, &triple(0))),
        (732, 3) => talk(Some(4)),
        // 793 entity 0: line 16 with 417. Entities 1 and 2: 0 (417), 3 (416), 6 (414), 9 (412),
        // 12 (410; item 13 taken and 412 set first), else 13.
        (793, 1) if s.flag(417) => talk(Some(16)),
        (793, 2) => {
            let line = if s.flag(417) {
                0
            } else if s.flag(416) {
                3
            } else if s.flag(414) {
                6
            } else if s.flag(412) {
                9
            } else if s.flag(410) {
                s.take(13, 1);
                s.set(412, true);
                12
            } else {
                13
            };
            talk(Some(line))
        }
        // 874 entities 6 and 7: the line, a camera, the entity's animation 30 (wait) and walk away,
        // a fade out, flag 291 or 290, the slot reset, the normal camera and a fade in.
        (874, 1 | 2) => {
            let (entity, line, track, locator, flag) = if routine == 1 { (6, 1, 5, 3, 291) } else { (7, 0, 4, 1, 290) };
            let mut v = vec![Action::Converse { entity, line }, Action::PlayCamera { track, wait: false, end: 1, speed: 1.0 }];
            if routine == 2 {
                v.push(Action::MoveHero { locator: 2 });
            }
            v.extend([
                Action::EntityAnim { entity, anim: 30, looped: false, wait: true },
                Action::WalkEntity { entity, locator, wait: false, orient: false, stop: 1.1 },
                Action::FadeOut { seconds: 1.0, wait: true },
                Action::SetFlag { flag, on: true },
                Action::LoadRegion { slot: 0, region: None },
                Action::ReturnCamera { preset: -1, release: true },
                Action::FadeIn { seconds: 0.7, wait: false },
            ]);
            v
        }
        _ => Vec::new(),
    };
    s.rotations.insert(key, rot);
    out
}

/// SPEC-0041 1409, 1421 and 1423 proximity callbacks (each line in the conversation frame).
fn palace_talk(pack: u32, routine: u8, s: &mut GameState) -> Vec<Action> {
    let talk = |line| vec![Action::Talk { line }];
    match pack {
        // 742 P7: line 0 in the conversation frame with 562, else line 4 alone.
        742 if s.flag(562) => talk(0),
        742 => vec![Action::Message { line: 4 }],
        // 1414 (registered with 102): entity 6 line 0, 9 line 6, 8 line 12, 7 line 9.
        1414 if !s.flag(102) => Vec::new(),
        1414 => talk(match routine {
            6 => 0,
            9 => 6,
            8 => 12,
            _ => 9,
        }),
        // 1409 callbacks 1 and 2 (registered with 106): with 102, line 0/3 or 10/13 by 104.
        1409 if !s.flag(106) || !s.flag(102) => Vec::new(),
        1409 => {
            let base = if routine == 2 { 10 } else { 0 };
            talk(base + if s.flag(104) { 0 } else { 3 })
        }
        // 1421 CB2 (registered with 102): the Random(1, 3) draw first; 605 with 679 clear takes
        // item 33, sets 679, line 13; then draw 1 line 10; else with 100 line 4 (106) or 7.
        1421 if !s.flag(102) || !s.flag(605) => Vec::new(),
        1421 if !s.flag(679) => {
            s.take(33, 1);
            s.set(679, true);
            talk(13)
        }
        1421 if s.talk_roll == 1 => talk(10),
        1421 if s.flag(100) => talk(if s.flag(106) { 4 } else { 7 }),
        // 1423 CB2 (registered with 106): 609 with 683 clear takes item 33, sets 683, line 7;
        // then with 104 line 8 (102) or 11.
        1423 if !s.flag(106) || !s.flag(609) => Vec::new(),
        1423 if !s.flag(683) => {
            s.take(33, 1);
            s.set(683, true);
            talk(7)
        }
        1423 if s.flag(104) => talk(if s.flag(102) { 8 } else { 11 }),
        _ => Vec::new(),
    }
}

/// SPEC-0043 750's line cycling for routine r (1 to 4; 0 is the on-enter routine): one position
/// counter per routine (word 10 + r) shared by its cases, set back to the first line once, the
/// first time the top case is taken (word 20 + r).
fn cycle_750(s: &mut GameState, r: u8, top: bool, lines: &[u16]) -> Vec<Action> {
    if top && s.word(750, 20 + r) == 0 {
        s.set_word(750, 20 + r, 1);
        s.set_word(750, 10 + r, 0);
    }
    let c = s.word(750, 10 + r).max(0) as usize;
    s.set_word(750, 10 + r, c as i32 + 1);
    vec![Action::Talk { line: lines[c % lines.len()] }]
}

/// SPEC-0043 750 routines 1 to 4 (entities 3, 1, 0, 2).
fn talk_750(r: u8, s: &mut GameState) -> Vec<Action> {
    let (f420, f76) = (s.flag(420), s.flag(76));
    let three = |a: u16| [a, a + 1, a + 2];
    match r {
        1..=3 => {
            let base = 9 * (r as u16 - 1);
            let lines = if f420 { three(base) } else if f76 { three(base + 3) } else { three(base + 6) };
            cycle_750(s, r, f420, &lines)
        }
        4 if f420 => cycle_750(s, 4, true, &three(35)),
        4 if s.flag(419) => {
            s.set(420, true);
            vec![Action::Talk { line: 38 }]
        }
        4 => cycle_750(s, 4, false, &if f76 { three(39) } else { three(42) }),
        _ => Vec::new(),
    }
}

/// SPEC-0037 interiors of eW 44 (783, 786, 787, 788, 789): the type-2 value their teleport 0
/// posts for eW 31 and the locator their arrival uses for it.
fn eastside_interior(pack: u32) -> Option<(i32, usize)> {
    match pack {
        783 => Some((60, 0)),
        786 => Some((20, 1)),
        787 => Some((30, 0)),
        788 => Some((50, 0)),
        789 => Some((40, 1)),
        _ => None,
    }
}

/// Their proximity routine on entity 0 (each line in the conversation frame).
fn eastside_talk(pack: u32, s: &mut GameState) -> Vec<Action> {
    let talk = |line| vec![Action::Talk { line }];
    match pack {
        788 => talk(4),
        // 787: with 447 set, 500 Neopoints buy flag 448 (line 3; 448 is not tested) or line 4;
        // else lines 5 to 10 in turn (module counter, word 0).
        787 if s.flag(447) && s.neopoints >= 500 => {
            s.set(448, true);
            s.neopoints -= 500;
            talk(3)
        }
        787 if s.flag(447) => talk(4),
        787 => {
            let c = s.word(pack, 0).rem_euclid(6);
            s.set_word(pack, 0, (c + 1) % 6);
            talk(5 + c as u16)
        }
        789 if s.hero.selected == 1 => talk(10),
        789 => talk(if s.flag(86) { 4 } else { 7 }),
        783 if s.flag(421) => talk(9),
        783 if s.flag(420) => {
            s.set(421, true);
            talk(12)
        }
        783 if s.flag(419) => talk(13),
        783 => {
            s.set(419, true);
            talk(16)
        }
        786 if s.flag(425) => talk(10),
        786 if s.flag(424) => {
            s.set(425, true);
            talk(13)
        }
        786 if s.flag(423) => talk(14),
        786 if s.flag(76) => {
            s.set(423, true);
            talk(17)
        }
        786 => talk(18),
        _ => Vec::new(),
    }
}

/// SPEC-0037 855 to 858 and 1429 proximity routines on entity 0.
fn outskirts_talk(pack: u32, s: &mut GameState) -> Vec<Action> {
    let talk = |line| vec![Action::Talk { line }];
    let f = |s: &GameState, n| s.flag(n);
    match pack {
        855 if f(s, 463) => talk(10),
        855 if f(s, 462) && s.has_item(22) => {
            s.set(463, true);
            s.take(22, 1);
            s.give(122, 1);
            talk(13)
        }
        855 if f(s, 462) => talk(14),
        855 if f(s, 59) => {
            s.set(462, true);
            talk(17)
        }
        855 => talk(if f(s, 56) { 18 } else { 21 }),
        856 => talk(if f(s, 59) { 10 } else if f(s, 56) { 13 } else { 16 }),
        857 => talk(if f(s, 59) { 6 } else if f(s, 56) { 9 } else { 12 }),
        858 if f(s, 56) && !f(s, 512) => {
            s.set(512, true);
            talk(6)
        }
        858 if f(s, 56) && !f(s, 514) && s.has_item(73) => {
            s.take(73, 1);
            s.set(513, true);
            s.set(514, true);
            talk(7)
        }
        // The rotating sets (words 0 and 1: the last set and its counter, restarted when the set
        // changes): 8 to 10 (59), 11 to 13 (56), 14 to 16.
        858 => {
            let set = if f(s, 59) { 1 } else if f(s, 56) { 2 } else { 3 };
            if s.word(pack, 0) != set {
                s.set_word(pack, 0, set);
                s.set_word(pack, 1, 0);
            }
            let c = s.word(pack, 1);
            let first = 8 + 3 * (set as u16 - 1);
            let (k, next) = match c {
                0 => (0, 1),
                1 => (1, 2),
                _ => (2, 0),
            };
            s.set_word(pack, 1, next);
            talk(first + k)
        }
        // 1429: one counter for the three sets, never reset on a change of set: 11 to 13 (322),
        // 14 to 16 (236), 17 to 19.
        1429 => {
            let first = if f(s, 322) { 11 } else if f(s, 236) { 14 } else { 17 };
            let c = s.word(pack, 0).rem_euclid(3);
            s.set_word(pack, 0, (c + 1) % 3);
            talk(first + c as u16)
        }
        // 1152: 29 lines 1 to 3; 28 line 4 (the words unchanged); 27 lines 5 to 7; 23 and 633
        // with 167 lines 8 to 10, with 162 lines 11 to 13, else 162 set and line 14; otherwise
        // lines 15 to 17 (set word 0 and counter 1, restarted on a change of set).
        1152 => {
            let (set, first) = if f(s, 29) {
                (1, 1)
            } else if f(s, 28) {
                return talk(4);
            } else if f(s, 27) {
                (2, 5)
            } else if f(s, 23) && f(s, 633) {
                if f(s, 167) {
                    (3, 8)
                } else if f(s, 162) {
                    (4, 11)
                } else {
                    s.set(162, true);
                    return talk(14);
                }
            } else {
                (5, 15)
            };
            if s.word(pack, 0) != set {
                s.set_word(pack, 0, set);
                s.set_word(pack, 1, 0);
            }
            let c = s.word(pack, 1).rem_euclid(3);
            s.set_word(pack, 1, (c + 1) % 3);
            talk(first + c as u16)
        }
        // 1251: 585 without 586 takes every item 84, sets 586, line 28; 460: 29 (273), 30 (270)
        // or the pair 31, 32; 457 with item 49: 460 set, line 33 (270) or 34, every item 49
        // taken (the ItemDeactivate and ItemActivate calls are not reproduced); 457: lines 35 to
        // 37; else 457 set and line 38 (273), 39 (270) or 40. Both rotations share set number
        // 2: the counter restarts only when the set word is not 2, and the pair gives no line at
        // counter 2.
        1251 if f(s, 585) && !f(s, 586) => {
            s.remove_all(84);
            s.set(586, true);
            talk(28)
        }
        1251 if f(s, 460) && f(s, 273) => talk(29),
        1251 if f(s, 460) && f(s, 270) => talk(30),
        1251 if f(s, 457) && !f(s, 460) && s.has_item(49) => {
            s.set(460, true);
            let line = if f(s, 270) { 33 } else { 34 };
            s.remove_all(49);
            talk(line)
        }
        1251 if f(s, 460) || f(s, 457) => {
            if s.word(pack, 0) != 2 {
                s.set_word(pack, 0, 2);
                s.set_word(pack, 1, 0);
            }
            let c = s.word(pack, 1);
            if f(s, 460) {
                match c {
                    0 => {
                        s.set_word(pack, 1, 1);
                        talk(31)
                    }
                    1 => {
                        s.set_word(pack, 1, 0);
                        talk(32)
                    }
                    _ => Vec::new(),
                }
            } else {
                let c = c.rem_euclid(3);
                s.set_word(pack, 1, (c + 1) % 3);
                talk(35 + c as u16)
            }
        }
        1251 => {
            s.set(457, true);
            talk(if f(s, 273) { 38 } else if f(s, 270) { 39 } else { 40 })
        }
        // 784: line 0 with 591, else 591 set and line 3.
        784 if f(s, 591) => talk(0),
        784 => {
            s.set(591, true);
            talk(3)
        }
        _ => Vec::new(),
    }
}

/// The flag-67 line rotation of SPEC-0039 880 and 1230: lines 0 to 2 with flag 67 set, 3 to 5
/// clear, one counter (word 2) restarted the first time 67 is seen set (word 3).
fn rotation_67(pack: u32, state: &mut GameState) -> u16 {
    let set = state.flag(67);
    if set && state.word(pack, 3) == 0 {
        state.set_word(pack, 3, 1);
        state.set_word(pack, 2, 0);
    }
    let k = state.word(pack, 2).rem_euclid(3);
    state.set_word(pack, 2, k + 1);
    if set { k as u16 } else { 3 + k as u16 }
}

/// SPEC-0042 1171 counting helper: for each set flag of 637 to 641, in order, the count rises
/// by one and flag 162 + count is set; returns the count.
fn clogs_1171(state: &mut GameState) -> u16 {
    let mut n = 0;
    for f in 637..=641 {
        if state.flag(f) {
            n += 1;
            state.set(162 + n, true);
        }
    }
    n
}

/// Run a registered script trigger routine of a pack for one fire of its record.
pub fn on_trigger(pack: u32, routine: u8, call: TriggerCall, state: &mut GameState) -> TriggerOut {
    let mut out = TriggerOut::default();
    let first = call.state == 1 && call.object;
    match (pack, routine) {
        // SPEC-0045 991 trigger routine A (crowd zones 0 to 2: triggers 4, 5 and the group's
        // trigger), on an entering fire: the crowd set (2 with 86 clear; 1 with 86, 299 and 414
        // set and 417 clear; else 0) and the zone's region; none ends it. Thread 4: wait until
        // the crowd record (word 30) is free, mark it, reset slot 1 and load the region, wait
        // until the hero leaves the zone's trigger, reset the slot and free the record.
        (991, 8) if call.state == 1 && call.object => {
            let set = if !state.flag(86) {
                2
            } else if state.flag(299) && state.flag(414) && !state.flag(417) {
                1
            } else {
                0
            };
            let zone = match call.index {
                5 => 1,
                4 => 0,
                _ => 2,
            };
            let j: [[Option<u32>; 3]; 3] = [[Some(4), Some(6), Some(7)], [Some(5), Some(6), Some(7)], [Some(9), Some(10), None]];
            if let Some(j) = j[set][zone] {
                out.actions.extend([
                    Action::WaitWord { word: 30 },
                    Action::Bits { word: 30, set: 1, clear: 0 },
                    Action::LoadRegion { slot: 1, region: None },
                    Action::LoadRegion { slot: 1, region: Some(992 + j) },
                    Action::WaitInside { index: call.index },
                    Action::LoadRegion { slot: 1, region: None },
                    Action::Bits { word: 30, set: 0, clear: 1 },
                ]);
            }
        }
        // SPEC-0039 886 handler 1 (trigger 5), entered with 250 clear: line 6; handler 2
        // (trigger 2): the Square context 3, a press shows line 7; handler 4 (trigger 0),
        // entered with 252 clear: line 8.
        (886, 1) if call.state == 1 && call.object && !state.flag(250) => out.actions.push(Action::Message { line: 6 }),
        (886, 2) if call.state != 0 && call.object => {
            out.prompt = Some(3);
            if call.square && !state.talking {
                out.actions.push(Action::Message { line: 7 });
            }
        }
        (886, 4) if call.state == 1 && call.object && !state.flag(252) => out.actions.push(Action::Message { line: 8 }),
        // SPEC-0039 1230 handler 2 (triggers 0, 1, 2): while inside, the Square context 3; a
        // press shows line 18, 20 or 19.
        (1230, 2) if call.state != 0 => {
            out.prompt = Some(3);
            if call.square && !state.talking {
                out.actions.push(Action::Message { line: match call.index { 0 => 18, 2 => 19, _ => 20 } });
            }
        }
        // Handler 3 (trigger 5), with 64 set, 248 clear and the object present: line 21 (the
        // BlockPlayerControl and ReturnHeroAnimToIdle calls are not reproduced).
        (1230, 3) if call.object && call.state != 0 && state.flag(64) && !state.flag(248) && !state.talking => out.actions.push(Action::Message { line: 21 }),
        // Handlers 5 and 6 (triggers 3 and 4): entity 46's line once per pass of 4 then 3, by
        // the flag-67 rotation (as 880's handlers 2 and 3).
        (1230, 5) if call.object && call.state == 1 && state.word(pack, 1) == 0 => {
            state.set_word(pack, 1, 1);
            out.actions.push(Action::Converse { entity: 46, line: rotation_67(pack, state) });
        }
        (1230, 6) if call.object && call.state == 1 => state.set_word(pack, 1, 0),
        // SPEC-0039 880 handler 1 (triggers 5 to 7): while inside, the Square context 3; a press
        // shows line 21, 22 or 24.
        (880, 1) if call.state != 0 => {
            out.prompt = Some(3);
            if call.square && !state.talking {
                out.actions.push(Action::Message { line: match call.index { 5 => 21, 6 => 22, _ => 24 } });
            }
        }
        // Handler 2 (trigger 0), entered with the talked word (1) clear: set it, and entity 3's
        // line by the flag-67 rotation (0 to 2 with 67 set, 3 to 5 clear; one counter, word 2,
        // restarted the first time 67 is seen set, word 3). Handler 3 (trigger 1) clears it.
        (880, 2) if call.object && call.state == 1 && state.word(pack, 1) == 0 => {
            state.set_word(pack, 1, 1);
            out.actions.push(Action::Converse { entity: 3, line: rotation_67(pack, state) });
        }
        (880, 3) if call.object && call.state == 1 => state.set_word(pack, 1, 0),
        // Handler 9 (trigger 2), entered with flag 252 clear: line 25.
        (880, 9) if call.object && call.state == 1 && !state.flag(252) => out.actions.push(Action::Message { line: 25 }),
        // Handler 5 (group 2's instance trigger), entered: with 296 clear slot 0 is emptied and
        // slot 1 loads set 4; then, after the hero leaves, slot 1 is emptied and thread 4 runs
        // again.
        (880, 5) if call.state == 1 && call.object => {
            if !state.flag(296) {
                out.actions.extend([Action::LoadRegion { slot: 0, region: None }, Action::LoadRegion { slot: 1, region: Some(885) }]);
            }
            out.actions.extend([Action::WaitInside { index: call.index }, Action::LoadRegion { slot: 1, region: None }, slots_880(state)]);
        }
        // SPEC-0037 796 S1 (trigger 1): the Square context 4; a press walks the hero to locator 7,
        // plays object 0 (wait) and object 17, deactivates group 19, re-activates groups 20 and
        // 21, plays object 1, cameras 5 and 4 (wait) and returns the camera (word 0 keeps it
        // from starting twice while it runs). S0 (trigger 0, entering with the body): value 80
        // for eW 22, a 1 s fade out, the replacement to eW 22, a 1 s fade in (Medium order).
        (796, 1) if call.state != 0 && state.word(pack, 0) == 0 => {
            out.prompt = Some(4);
            if call.square && !state.talking {
                state.set_word(pack, 0, 1);
                out.actions.extend([
                    Action::HoldHero,
                    Action::WalkHero { locator: 7, wait: false, orient: true },
                    Action::PlayAnim { obj: 0, speed: 1.0, wait: true },
                    Action::PlayAnim { obj: 17, speed: 1.0, wait: false },
                    Action::DeactivateGroup { group: 19 },
                    Action::ReactivateGroup { group: 20 },
                    Action::ReactivateGroup { group: 21 },
                    Action::PlayAnim { obj: 1, speed: 1.0, wait: false },
                    Action::PlayCamera { track: 5, wait: true, end: 1, speed: 1.0 },
                    Action::PlayCamera { track: 4, wait: true, end: 1, speed: 1.0 },
                    Action::ReturnCamera { preset: -1, release: true },
                    Action::Bits { word: 0, set: 0, clear: 1 },
                ]);
            }
        }
        (796, 0) if call.object && call.state == 1 => {
            state.blackboard.insert((2, 22), 80);
            out.actions.extend([Action::FadeOut { seconds: 1.0, wait: false }, Action::ReplaceExterior { slot: 22, plane: -1 }, Action::FadeIn { seconds: 1.0, wait: false }]);
        }
        // SPEC-0037 1369 T1 (trigger 1, the body; no state test): the exterior replacement to eW
        // 157. T0 (trigger 0, the body): the Square context 4; a press walks the hero to locator 1,
        // plays hero animation 140, object 1 (wait), camera 4, object 0 (wait), 1 s and the normal
        // camera (word 0 keeps it from starting twice while it runs).
        (1369, 1) if call.object => out.actions.push(Action::ReplaceExterior { slot: 157, plane: -1 }),
        (1369, 0) if call.object && state.word(pack, 0) == 0 => {
            out.prompt = Some(4);
            if call.square && !state.talking {
                state.set_word(pack, 0, 1);
                out.actions.extend([
                    Action::HoldHero,
                    Action::WalkHero { locator: 1, wait: true, orient: true },
                    Action::HeroAnim { anim: 140, wait: false },
                    Action::PlayAnim { obj: 1, speed: 1.0, wait: true },
                    Action::PlayCamera { track: 4, wait: false, end: 1, speed: 1.0 },
                    Action::PlayAnim { obj: 0, speed: 1.0, wait: true },
                    wait(1.0),
                    Action::ReturnCamera { preset: -1, release: true },
                    Action::Bits { word: 0, set: 0, clear: 1 },
                ]);
            }
        }
        // SPEC-0043 1358 latch (trigger 1, entering with the body, latch word 2 clear): entity
        // 21's line 87 the first time, 88 the second (word 3 counts), then nothing; trigger 2
        // clears the latch.
        (1358, 21) if call.state == 1 && call.object && state.word(pack, 2) == 0 => {
            state.set_word(pack, 2, 1);
            let n = state.word(pack, 3);
            if n < 2 {
                state.set_word(pack, 3, n + 1);
                out.actions.push(Action::Converse { entity: 21, line: 87 + n as u16 });
            }
        }
        (1358, 22) if call.object => state.set_word(pack, 2, 0),
        // 1358 routine 19 (triggers 3 and 4, entering with the body, flag 102): the slot swap to
        // choice 0 (set 2, region 1361) or 1 (set 3, 1362) when it differs from the current one
        // (word 4 holds choice + 1).
        (1358, 19) if call.state == 1 && call.object && state.flag(102) => {
            let choice = if call.index == 3 { 0 } else { 1 };
            if state.word(pack, 4) != choice + 1 {
                state.set_word(pack, 4, choice + 1);
                out.actions.extend([Action::LoadRegion { slot: 1, region: None }, Action::LoadRegion { slot: 1, region: Some(1361 + choice as u32) }]);
            }
        }
        // 1358 routine 20, the signs (triggers 5 and 7): the Square context 3; a press shows line
        // 2 or 4.
        (1358, 20) if call.state != 0 && call.object => {
            out.prompt = Some(3);
            if call.square && !state.talking {
                out.actions.push(Action::Message { line: if call.index == 5 { 2 } else { 4 } });
            }
        }
        // SPEC-0043 1173 routine 2 (trigger 1; Medium-High): while object 1 is shown, the Square
        // context 4; on a press with 162 set and 167 clear the walk to locator 7, hero animation
        // 140, object 1 played (wait) and off, object 2 on, camera 4, sound 11, object 0 played,
        // about 1 s, flag 630 and the normal camera; else line 0. Word 0 keeps it from starting
        // twice; the object-1 test is taken from the word (set when the lever is used).
        (1173, 2) if call.state != 0 && state.word(pack, 0) == 0 => {
            out.prompt = Some(4);
            if call.square && !state.talking {
                if state.flag(162) && !state.flag(167) {
                    state.set_word(pack, 0, 1);
                    out.actions.extend([
                        Action::HoldHero,
                        Action::WalkHero { locator: 7, wait: true, orient: true },
                        Action::HeroAnim { anim: 140, wait: false },
                        Action::PlayAnim { obj: 1, speed: 1.0, wait: true },
                        Action::DeactivateObj { obj: 1 },
                        Action::InitObjOn { obj: 2, end: false },
                        Action::PlayCamera { track: 4, wait: false, end: 1, speed: 1.0 },
                        Action::PlaySound { cue: 11, locator: None },
                        Action::PlayAnim { obj: 0, speed: 1.0, wait: false },
                        wait(1.0),
                        Action::SetFlag { flag: 630, on: true },
                        Action::ReturnCamera { preset: -1, release: true },
                    ]);
                } else {
                    out.actions.push(Action::Message { line: 0 });
                }
            }
        }
        // 1173 routine 3 (trigger 0, the body; Medium): with 630 the Square context 13; a press
        // posts 120 for eW 124 and replaces the exterior with eW 124 (pack 1171).
        (1173, 3) if call.object && state.flag(630) => {
            out.prompt = Some(13);
            if call.square && !state.talking {
                state.blackboard.insert((2, 124), 120);
                out.actions.push(Action::ReplaceExterior { slot: 124, plane: -1 });
            }
        }
        // SPEC-0039 732 handler 1 (triggers 0 to 2, every fire with the body): value 70, 60 or 40
        // for eW 22, a 0.5 s fade out (wait) and the hero to locator 0 (no button needed). Word 0
        // keeps the exit from repeating while it runs.
        (732, 1) if call.object && state.word(pack, 0) == 0 => {
            state.set_word(pack, 0, 1);
            state.blackboard.insert((2, 22), match call.index { 0 => 70, 1 => 60, _ => 40 });
            out.actions.extend([Action::FadeOut { seconds: 0.5, wait: true }, Action::MoveHero { locator: 0 }, Action::Bits { word: 0, set: 0, clear: 1 }]);
        }
        // SPEC-0037 947 S2 (trigger 2, the body) arms word 0; S0 (trigger 0, entering with the
        // body) with the word armed and 375 clear shows line 0 and disarms it; S1 (trigger 1,
        // entering) replaces the exterior with eW 69 (plane 0).
        (947, 2) if call.object => state.set_word(pack, 0, 1),
        (947, 0) if call.object && call.state == 1 && state.word(pack, 0) != 0 && !state.flag(375) => {
            state.set_word(pack, 0, 0);
            out.actions.push(Action::Message { line: 0 });
        }
        (947, 1) if call.object && call.state == 1 => out.actions.push(Action::ReplaceExterior { slot: 69, plane: 0 }),
        // SPEC-0037 1299 S0 (trigger 0, entering with the body): sting stream 18 at 1.0.
        (1299, 0) if call.object && call.state == 1 => out.actions.push(Action::PlayStinger { id: 18, volume: 1.0 }),
        // SPEC-0037 1220 S0 (trigger 0, entering with the body): camera 5, the walk to locator
        // 26, a 1 s fade out (wait), the hero to locator 29 and value 1 (type 3) for eW 129.
        (1220, 0) if call.object && call.state == 1 => {
            state.blackboard.insert((3, 129), 1);
            out.actions.extend([
                Action::PlayCamera { track: 5, wait: false, end: 1, speed: 1.0 },
                Action::WalkHero { locator: 26, wait: false, orient: false },
                Action::FadeOut { seconds: 1.0, wait: true },
                Action::MoveHero { locator: 29 },
            ]);
        }
        // SPEC-0037 1259 S (triggers 0 to 49, with the body): slot 0 reset and loaded with the set
        // of the trigger's index (region 1260 + index).
        (1259, 0) if call.object && call.state == 1 => out.actions.extend([Action::LoadRegion { slot: 0, region: None }, Action::LoadRegion { slot: 0, region: Some(1260 + call.index as u32) }]),
        // SPEC-0037 1253 S0 (trigger 0, entering with the body): value 40 for eW 141 and the
        // exterior replacement to eW 141.
        (1253, 0) if call.object && call.state == 1 => {
            state.blackboard.insert((2, 141), 40);
            out.actions.push(Action::ReplaceExterior { slot: 141, plane: -1 });
        }
        // SPEC-0037 799 S0 (trigger 0; a hero shot is not modelled, so every fire runs it): the
        // Square context 6; a press with 416 set takes every item 16, opens the lever (object 0
        // off, objects 1 and 2 played forward, 2 with wait), sets 663, waits until the hero
        // leaves, 1.5 s, plays them backward and turns object 0 on; with 416 clear line 0. Word 0
        // keeps a running sequence from starting again.
        (799, 0) if call.state != 0 && state.word(pack, 0) == 0 => {
            out.prompt = Some(6);
            if call.square && !state.talking {
                if state.flag(416) {
                    state.set_word(pack, 0, 1);
                    state.remove_all(16);
                    out.actions.extend([
                        Action::DeactivateObj { obj: 0 },
                        Action::PlayAnim { obj: 1, speed: 1.0, wait: false },
                        Action::PlayAnim { obj: 2, speed: 1.0, wait: true },
                        Action::SetFlag { flag: 663, on: true },
                        Action::WaitInside { index: call.index },
                        wait(1.5),
                        Action::PlayAnim { obj: 1, speed: -1.0, wait: false },
                        Action::PlayAnim { obj: 2, speed: -1.0, wait: false },
                        Action::InitObjOn { obj: 0, end: false },
                        Action::Bits { word: 0, set: 0, clear: 1 },
                    ]);
                } else {
                    out.actions.push(Action::Message { line: 0 });
                }
            }
        }
        // 799 S3 (trigger 3, entering with the body, item 12 not held, 416 set): line 1 on every
        // second qualifying entry (word 1 alternates, from 0).
        (799, 3) if call.object && call.state == 1 && !state.has_item(12) && state.flag(416) => {
            let c = state.word(pack, 1);
            if c > 0 {
                out.actions.push(Action::Message { line: 1 });
            }
            state.set_word(pack, 1, (c + 1) % 2);
        }
        // SPEC-0037 784 S1 (trigger 1, every fire; a hero shot never fires it here, and the
        // attack word is taken as 0): without the button and with 543 clear the Square context
        // 4; with the button, 543 clear and the body: line 5 while 542 is clear, else the walk
        // to locator 2, hero animation 140, object 2 played, 0.6 s, 0.8 s, 15 ticks, line 4 and
        // 543 set (word 2 keeps it from starting twice).
        (784, 1) if !state.flag(543) => {
            if !call.square {
                out.prompt = Some(4);
            } else if call.object && !state.talking && state.word(pack, 2) == 0 {
                if !state.flag(542) {
                    out.actions.push(Action::Message { line: 5 });
                } else {
                    state.set_word(pack, 2, 1);
                    out.actions.extend([
                        Action::HoldHero,
                        Action::WalkHero { locator: 2, wait: true, orient: true },
                        Action::HeroAnim { anim: 140, wait: false },
                        Action::PlayAnim { obj: 2, speed: 1.0, wait: false },
                        Action::Sleep { ticks: 36 },
                        Action::Sleep { ticks: 48 },
                        Action::Sleep { ticks: 15 },
                        Action::Message { line: 4 },
                        Action::SetFlag { flag: 543, on: true },
                        release_camera(),
                    ]);
                }
            }
        }
        // SPEC-0042 1171 levers L5 to L7 (triggers 5 to 7), once per load (words 5 to 7): the
        // Square prompt (message 6) while inside; on Square the walk and orient to the walk
        // point, the first lever object off and the second shown and played, hero animation
        // 140, a 0.5 s fade out and in, the camera track, the lever's looping sound, the gate
        // object played, 1 s, the sound killed, a fade out, the normal camera, a fade in.
        (1171, 5..=7) if call.state != 0 && state.word(pack, routine) == 0 => {
            let (a, b, walk, track, sound, gate) = match routine {
                5 => (11, 12, 56, 4, 56, 2),
                6 => (13, 14, 57, 7, 57, 0),
                _ => (15, 16, 58, 6, 58, 1),
            };
            out.prompt = Some(6);
            if call.square && !state.talking {
                state.set_word(pack, routine, 1);
                out.actions.extend([
                    Action::HoldHero,
                    Action::WalkHero { locator: walk, wait: true, orient: true },
                    Action::DeactivateObj { obj: a },
                    Action::InitObjOn { obj: b, end: false },
                    Action::PlayAnim { obj: b, speed: 1.0, wait: false },
                    Action::HeroAnim { anim: 140, wait: false },
                    Action::FadeOut { seconds: 0.5, wait: true },
                    Action::FadeIn { seconds: 0.5, wait: true },
                    Action::PlayCamera { track, wait: false, end: 1, speed: 1.0 },
                    Action::LoopSound { cue: sound, locator: None },
                    Action::PlayAnim { obj: gate, speed: 1.0, wait: false },
                    wait(1.0),
                    Action::KillSound { cue: sound, locator: None },
                    Action::FadeOut { seconds: 0.5, wait: true },
                    Action::ReturnCamera { preset: -1, release: true },
                    Action::FadeIn { seconds: 0.5, wait: true },
                ]);
            }
        }
        // 1171 T0 (trigger 0): object present, state 1 and flag 33 clear: the clog count; below
        // 5 the conscience line of that count.
        (1171, 0) if call.object && call.state == 1 && !state.flag(33) => {
            let n = clogs_1171(state);
            if n < 5 {
                out.actions.push(Action::Message { line: n });
            }
        }
        // 1171 T1 (trigger 1): the exterior replacement load of eW 126. The blackboard value 120
        // for world 126 is not added: its message type is not given (R-0053).
        (1171, 1) if call.object && call.state == 1 => out.actions.push(Action::ReplaceExterior { slot: 126, plane: -1 }),
        // SPEC-0039 1244 trigger 0 (handler 1): the first fire with 272 clear in swap mode 2, the
        // story switch to Roberta at locator 16 and her scene; it ends locked to Roberta (mode
        // 1). NeedHeroAlive and the weapon sheathe sound are not reproduced.
        (1244, 1) if first && !state.flag(272) && state.hero.swap_mode == 2 => {
            out.actions.extend([
                Action::HoldHero,
                Action::Equip { on: false },
                Action::FadeOut { seconds: 1.0, wait: true },
                Action::ActivateHero { hero: 1, locator: 16, block: false, fade: false, cue: false },
                Action::Sleep { ticks: 30 },
                Action::FadeIn { seconds: 1.0, wait: false },
                Action::LetterBox { on: true, seconds: 0.0, wait: false },
                Action::PlayCamera { track: 4, wait: true, end: 1, speed: 0.3 },
            ]);
            out.actions.extend((0..=10).map(|line| Action::Message { line }));
            out.actions.extend([
                Action::MoveHero { locator: 14 },
                camera(11),
                Action::Message { line: 11 },
                Action::LetterBox { on: false, seconds: 0.0, wait: false },
                Action::ReturnCamera { preset: 19, release: true },
                Action::SwapMode { mode: 1 },
            ]);
        }
        // Triggers 1 and 2 (handlers 2 and 3), 273 clear: line 13 (12) and the walk to locator 13
        // (14). The hero is held by the walk and the box, so a stay inside does not repeat it.
        (1244, 2 | 3) if call.state != 0 && call.object && !state.flag(273) && !state.talking && !state.hero.occupied => {
            let (line, locator) = if routine == 2 { (13, 13) } else { (12, 14) };
            out.actions.extend([Action::Message { line }, Action::WalkHero { locator, wait: true, orient: false }]);
        }
        // Trigger 4 (handler 4), 272 and level state 0 clear: level state 0, Roberta's walk to
        // locator 20 with lines 14 to 16 under cameras 12, 13 and 16, and the walk to 21.
        (1244, 4) if call.state != 0 && !state.flag(272) && !state.level(0) => {
            state.set_level(0, true);
            out.actions.extend([
                Action::HoldHero,
                Action::LetterBox { on: true, seconds: 1.0, wait: false },
                Action::WalkHero { locator: 20, wait: false, orient: true },
                Action::PlayCamera { track: 12, wait: true, end: 1, speed: 1.0 },
                Action::Message { line: 14 },
                camera(13),
                Action::Message { line: 15 },
                camera(16),
                Action::Message { line: 16 },
                Action::WalkHero { locator: 21, wait: false, orient: false },
                Action::LetterBox { on: false, seconds: 1.0, wait: false },
                release_camera(),
            ]);
        }
        // Trigger 3 (handler 5), the first fire with 272 clear: the break scene (object 5 played
        // with wait, objects 5, 4 and 1 off, object 6 on and played; the shake and Break of
        // breakables 17 to 24 are not reproduced: R-0040), then the story switch back to Tor at
        // locator 12 in mode 0, and 272, which ends the module's triggers for good.
        (1244, 5) if first && !state.flag(272) => {
            out.actions.extend([
                Action::LetterBox { on: true, seconds: 0.0, wait: false },
                Action::HoldHero,
                Action::MoveHero { locator: 22 },
                Action::Equip { on: false },
                Action::Message { line: 17 },
                Action::PlayCamera { track: 14, wait: true, end: 1, speed: 1.0 },
                camera(16),
                Action::PlaySound { cue: 7, locator: None },
                Action::PlayAnim { obj: 5, speed: 1.0, wait: true },
                Action::KillSound { cue: 7, locator: None },
                Action::PlaySound { cue: 8, locator: None },
                Action::Sleep { ticks: 20 },
                camera(17),
                Action::HeroAnim { anim: 125, wait: true },
                Action::HoldHero,
                Action::Sleep { ticks: 10 },
                camera(16),
                Action::PlayHeroSound { hero: -1, cue: 1, locator: None, do_not_restart: false },
                Action::Message { line: 18 },
                Action::DeactivateObj { obj: 5 },
                Action::DeactivateObj { obj: 4 },
                Action::DeactivateObj { obj: 1 },
                Action::InitObjOn { obj: 6, end: false },
                Action::PlaySound { cue: 9, locator: None },
                Action::PlayAnim { obj: 6, speed: 1.0, wait: true },
                Action::KillSound { cue: 9, locator: None },
                camera(16),
                Action::Message { line: 19 },
                Action::Sleep { ticks: 1 },
                fade_out(),
                Action::MoveHero { locator: 12 },
                camera(5),
                Action::FadeIn { seconds: FADE, wait: false },
                Action::SwapMode { mode: 0 },
                Action::ActivateHero { hero: 0, locator: 12, block: false, fade: false, cue: false },
                Action::Message { line: 20 },
                Action::Message { line: 21 },
                Action::SetFlag { flag: 272, on: true },
                fade_out(),
                Action::LetterBox { on: false, seconds: 0.0, wait: false },
                release_camera(),
                Action::Sleep { ticks: 30 },
                Action::FadeIn { seconds: FADE, wait: false },
            ]);
        }
        // SPEC-0059 section 3, the 1023 walk-up greetings: trigger 37 (the arena master, entity
        // 66) and 39 (the wheel host, entity 65) run their greeting when their latch (words 24
        // and 25) is 0 and set it; 38 and 40 clear it. The arena master speaks while the arena
        // result word (26) is 0: the line of the highest arena flag (385 to 381: 28 to 32),
        // else 33 and 34 in turn with 380, else 35 and 36 in turn (word 27). The wheel host
        // greets twice per module life (word 28): 1016 (480), 1017 (384), 1018 (479), 1019
        // (478) or 1020, then 1021.
        (1023, 54) if first => {
            let latch = if call.index == 37 { 24 } else { 25 };
            if state.word(pack, latch) == 0 {
                state.set_word(pack, latch, 1);
                if call.index == 37 {
                    if state.word(pack, 26) == 0 {
                        let line = [(385u16, 28u16), (384, 29), (383, 30), (382, 31), (381, 32)].iter().find(|(f, _)| state.flag(*f)).map(|&(_, l)| l).unwrap_or_else(|| {
                            let k = state.word(pack, 27);
                            state.set_word(pack, 27, 1 - k);
                            if state.flag(380) { 33 + k as u16 } else { 35 + k as u16 }
                        });
                        out.actions.push(Action::Converse { entity: 66, line });
                    }
                } else {
                    let n = state.word(pack, 28);
                    if n < 2 {
                        state.set_word(pack, 28, n + 1);
                        let line = if n == 1 {
                            1021
                        } else {
                            [(480u16, 1016u16), (384, 1017), (479, 1018), (478, 1019)].iter().find(|(f, _)| state.flag(*f)).map_or(1020, |&(_, l)| l)
                        };
                        out.actions.push(Action::Converse { entity: 65, line });
                    }
                }
            }
        }
        (1023, 55) if first => {
            let latch = if call.index == 38 { 24 } else { 25 };
            state.set_word(pack, latch, 0);
        }
        // SPEC-0059 section 2, the 1023 prize wheel (trigger 36, routine 56): context 5 while
        // inside; on Square, the too-poor line below the fee, else hero control and the round
        // step (the music override is not played).
        (1023 | 1358, 56) if call.state != 0 => {
            if !call.square {
                out.prompt = Some(5);
            } else if !state.talking && !state.hero.occupied {
                let c = crate::wheel::copy_of(pack);
                if state.neopoints < c.fee {
                    out.actions.push(Action::Message { line: c.too_poor });
                } else {
                    out.actions.extend([Action::HoldHero, Action::WheelRound]);
                }
            }
        }
        // SPEC-0042 1184 trigger 0 (E0): with 145 set, the first time (645), line 14.
        (1184, 1) if first && state.flag(145) && !state.flag(645) => {
            state.set(645, true);
            out.actions.push(Action::Message { line: 14 });
        }
        // E7 (220 set, 277 clear): 277, the fade, movie 40 (not played), type 3 value 1 for eW
        // 131, the hero to locator 44.
        (1184, 3) if first && !state.flag(277) => {
            state.set(277, true);
            state.blackboard.insert((3, 131), 1);
            out.actions.extend([Action::FadeOut { seconds: 0.25, wait: true }, Action::MoveHero { locator: 44 }]);
        }
        // E9 (220 set): type 3 value 1 for eW 130 and the replacement load without a fade.
        (1184, 5) if first => {
            state.blackboard.insert((3, 130), 1);
            out.actions.push(Action::ReplaceExterior { slot: 130, plane: -1 });
        }
        // E11 (220 clear): type 3 value 0 for eW 130 and the replacement load.
        (1184, 6) if first => {
            state.blackboard.insert((3, 130), 0);
            out.actions.push(Action::ReplaceExterior { slot: 130, plane: -1 });
        }
        // SPEC-0045 1163 trigger 0: the square-button prompt (context 13); the press with 155
        // clear or s 30 shows line 12.
        (1163, 1) if call.state != 0 && call.object => {
            out.prompt = Some(13);
            if call.square && (!state.flag(155) || castle_stage(state) == 30) {
                out.actions.push(Action::Message { line: 12 });
            }
        }
        // SPEC-0044 1056 triggers 0 and 1 (inside, not occupied, not talking): line 77 with flag
        // 653 and line 78 with flag 652, once each.
        (1056, 1) if call.state != 0 && !state.talking && !state.hero.occupied && !state.flag(653) => {
            state.set(653, true);
            out.actions.push(Action::Message { line: 77 });
        }
        (1056, 2) if call.state != 0 && !state.talking && !state.hero.occupied && !state.flag(652) => {
            state.set(652, true);
            out.actions.push(Action::Message { line: 78 });
        }
        // The zone triggers: the hero-inside mark of this tick (module word 0, bit = index).
        (1056, 5) if call.state != 0 => {
            let w = state.word(1056, 0);
            state.set_word(1056, 0, w | (1 << call.index));
        }
        // SPEC-0045 1098 trigger 0, the rest prompt (context 32): on Square, the first case of
        // the story (152 with 22 clear: movie 7, regions 1104 and 1103, camera 9, line 50,
        // flags 22, 642 and 153; 191 with 27 clear: line 376, flags 27 and 193; 204 with 29
        // clear: flags 29, 205 and 63 and movie 10). The hit-point percentage it sets is not
        // given; the movies are not played.
        (1098, 1) if call.state != 0 && call.object => {
            out.prompt = Some(32);
            if call.square {
                out.actions.extend(castle_rest(state));
            }
        }
        // Trigger 1 (30 and 31 clear at load): line 51 when the hero is not talking.
        (1098, 2) if first && !state.talking => out.actions.push(Action::Message { line: 51 }),
        // Triggers 11 and 12 (30 and 31 clear): slot A 1101 and slot B 1112; slot A 1102.
        (1098, 11) if first && !state.flag(30) && !state.flag(31) => {
            out.actions.extend([Action::LoadRegion { slot: 0, region: Some(1101) }, Action::LoadRegion { slot: 1, region: Some(1112) }]);
        }
        (1098, 12) if first && !state.flag(30) && !state.flag(31) => out.actions.push(Action::LoadRegion { slot: 0, region: Some(1102) }),
        // Trigger 14: slot A 1105 or 1107 and slot B one of 1110, 1111, 1114 or 1115 by flags
        // 194 to 196 and 204 to 206; the table is not given (R-0037): 1105 with 1110 (entity
        // 35) before 194, 1111 (entity 36) after it, 1114 from 204, and 1115 from 205 here.
        (1098, 14) if first && !state.flag(30) && !state.flag(31) => {
            let b = if state.flag(205) || state.flag(206) {
                1115
            } else if state.flag(204) {
                1114
            } else if state.flag(194) || state.flag(195) || state.flag(196) || state.word(1098, 1) != 0 {
                1111
            } else {
                1110
            };
            out.actions.extend([Action::LoadRegion { slot: 0, region: Some(1105) }, Action::LoadRegion { slot: 1, region: Some(b) }]);
        }
        // SPEC-0045 1217 script trigger 0 (Act 1): the start bit.
        (1217, 1) if first => {
            let w = state.word(1217, 0);
            state.set_word(1217, 0, w | 1);
        }
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
                // Fade out, hero to locator 29, object 9 activated, camera 27, fade in, sound 9,
                // objects 18, 20 and 17 played, object 20 stopped and deactivated, object 19
                // played, effects 3 and 4 started, sound 10, line 166, object 9 deactivated,
                // line 162, camera returned (the shake is not reproduced; the plays wait in turn,
                // Medium in the spec).
                out.actions.extend([Action::HoldHero, fade_out(), Action::MoveHero { locator: 29 }, Action::InitObjOn { obj: 9, end: false }, camera(27), fade_in()]);
                out.actions.push(Action::PlaySound { cue: 9, locator: None });
                for obj in [18, 20, 17] {
                    out.actions.push(Action::PlayAnim { obj, speed: 1.0, wait: true });
                }
                out.actions.extend([Action::StopAnim { obj: 20 }, Action::DeactivateObj { obj: 20 }, Action::PlayAnim { obj: 19, speed: 1.0, wait: true }]);
                out.actions.extend([Action::StartEffect { slot: 3, camera: false }, Action::StartEffect { slot: 4, camera: false }]);
                out.actions.push(Action::PlaySound { cue: 10, locator: None });
                state.remove_all(6);
                out.actions.push(Action::Message { line: 166 });
                state.give(7, 1);
                out.actions.push(Action::DeactivateObj { obj: 9 });
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
        // SPEC-0042 1184 thread B: while 512 is set and 513 clear, holding item 73 sets 513.
        1184 => {
            if state.flag(512) && !state.flag(513) && state.has_item(73) {
                state.set(513, true);
            }
        }
        1217 => out = burrows_fight(state, dialog),
        // SPEC-0039 886 Update. The end helper: with 250 clear and AI 16, 17 and 18 dead, object
        // 3 off, 250, 6 s, a 1 s fade out (movie 32 is not built), 1 tick, a 1 s fade in. The
        // hint helper, while 250 is clear: a 300-tick counter (word 0), then once line 1 unless
        // hero 1 is active (word 1), then once, with hero 1 active, AI 16 and 17 dead and 18
        // alive, 2 s and line 3 (word 2).
        886 => {
            let dead = |e: usize| state.ai_dead.contains(&e);
            if !state.flag(250) && dead(16) && dead(17) && dead(18) {
                state.set(250, true);
                out.extend([Action::DeactivateObj { obj: 3 }, wait(6.0), Action::FadeOut { seconds: 1.0, wait: true }, Action::Sleep { ticks: 1 }, Action::FadeIn { seconds: 1.0, wait: false }]);
            } else if !state.flag(250) {
                let n = state.word(pack, 0);
                if n < 300 {
                    state.set_word(pack, 0, n + 1);
                } else if state.word(pack, 1) == 0 {
                    state.set_word(pack, 1, 1);
                    if state.hero.selected != 1 {
                        out.push(Action::Message { line: 1 });
                    }
                    out.push(Action::Sleep { ticks: 1 });
                } else if state.word(pack, 2) == 0 && state.hero.selected == 1 && dead(16) && dead(17) && !dead(18) {
                    state.set_word(pack, 2, 1);
                    out.extend([wait(2.0), Action::Message { line: 3 }]);
                }
            }
        }
        // SPEC-0037 1238 thread T2 (flag 50, while 567 is set): four or more of item 82 with 568
        // clear set 568.
        1238 => {
            if state.flag(50) && state.flag(567) && !state.flag(568) && state.items.get(&82).copied().unwrap_or(0) >= 4 {
                state.set(568, true);
            }
        }
        // SPEC-0037 947 thread G: while 375 is clear, AI 3 dead sets 375 and shows lines 1 and 2.
        947 => {
            if !state.flag(375) && state.ai_dead.contains(&3) {
                state.set(375, true);
                out.extend([Action::Message { line: 1 }, Action::Message { line: 2 }]);
            }
        }
        // SPEC-0039 1230 Update: object 0 off unless 64 is set and 248 clear.
        1230 => {
            if !(state.flag(64) && !state.flag(248)) {
                out.push(Action::DeactivateObj { obj: 0 });
            }
        }
        // SPEC-0039 732 thread 4: AI 0 dead (after the set-0 load) sets 562.
        732 => {
            if state.flag(561) && !state.flag(562) && state.ai_dead.contains(&0) {
                state.set(562, true);
            }
        }
        // SPEC-0041 742 thread B: entity 6 (the Giant Slorg) dead sets 562.
        742 => {
            if !state.flag(562) && state.ai_dead.contains(&6) {
                state.set(562, true);
            }
        }
        // SPEC-0039 880 Update: while 296 is clear, AI 21, 22 and 23 all dead set 296.
        880 => {
            if !state.flag(296) && [21, 22, 23].iter().all(|e| state.ai_dead.contains(e)) {
                state.set(296, true);
            }
        }
        1056 => out = yard_update(state, dialog),
        1163 => out = kitchen_update(state),
        // SPEC-0045 1098 Update: effects 0, 1 and 2 are started while 642 is clear and stopped
        // while it is set; with 642 set, effect 3 is started while 632 is clear and stopped
        // while it is set (the start's camera flag is not given; none is passed).
        1098 => {
            let start = |slot| Action::StartEffect { slot, camera: false };
            if !state.flag(642) {
                out.extend((0..3).map(start));
            } else {
                out.extend((0..3).map(|slot| Action::StopEffect { slot }));
                out.push(if state.flag(632) { Action::StopEffect { slot: 3 } } else { start(3) });
            }
        }
        // SPEC-0045 909 Update: the flag and item steps and the scenes.
        909 => {
            // The Act 1 start: 707 on and 1 off sets 1 and clears 706, fades out at once, makes
            // the act-start 1 call (SPEC-0053 3.7) and moves entity 31 to locator 69 (movies 88
            // and 1 are not played; camera 30 is left out: no return of it is described, R-0025).
            if state.flag(707) && !state.flag(1) {
                state.set(1, true);
                state.set(706, false);
                out.extend([Action::FadeOut { seconds: 0.0, wait: false }, Action::ActStart { stage: 1 }, Action::MoveEntity { entity: 31, locator: 69, face: true }]);
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
            // to locator 7, camera 7, sound 5, objects 10 and 11 played, object 3 deactivated and
            // object 2 activated. The keyed camera 7 is played with end mode 2, which frees the
            // hero at its end (R-0025).
            if state.flag(110) && state.word(pack, 5) == 0 {
                state.set_word(pack, 5, 1);
                out.extend([Action::HoldHero, Action::MoveHero { locator: 7 }, Action::PlayCamera { track: 7, wait: false, end: 2, speed: 1.0 }]);
                out.push(Action::PlaySound { cue: 5, locator: None });
                out.extend([Action::PlayAnim { obj: 10, speed: 1.0, wait: false }, Action::PlayAnim { obj: 11, speed: 1.0, wait: false }]);
                out.extend([Action::DeactivateObj { obj: 3 }, Action::InitObjOn { obj: 2, end: false }]);
            }
            // The flag-122 step (its word set by Init when 122 was clear at load): sound 5,
            // objects 15 and 16 played, object 4 deactivated, object 5 activated, word cleared.
            if state.word(pack, 6) != 0 && state.flag(122) {
                state.set_word(pack, 6, 0);
                out.push(Action::PlaySound { cue: 5, locator: None });
                out.extend([Action::PlayAnim { obj: 15, speed: 1.0, wait: false }, Action::PlayAnim { obj: 16, speed: 1.0, wait: false }]);
                out.extend([Action::DeactivateObj { obj: 4 }, Action::InitObjOn { obj: 5, end: false }]);
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
                out.extend([Action::PlayAnim { obj: 19, speed: 1.0, wait: false }, Action::StopEffect { slot: 3 }, Action::StopEffect { slot: 4 }]);
                out.extend([Action::WalkEntity { entity: 44, locator: 15, wait: true, orient: false, stop: 0.0 }, fade_out()]);
                out.extend([Action::LoadRegion { slot: 3, region: Some(926) }, Action::LoadRegion { slot: 3, region: Some(928) }, Action::WaitLoads]);
                state.set(119, true);
                out.extend([Action::DeactivateObj { obj: 8 }, Action::DeactivateObj { obj: 7 }, Action::InitObjOn { obj: 6, end: false }]);
                out.extend([Action::MoveEntity { entity: 45, locator: 21, face: true }, camera(15), fade_in()]);
                out.extend([Action::WalkEntity { entity: 45, locator: 23, wait: true, orient: false, stop: 0.0 }, Action::PlayAnim { obj: 6, speed: 1.0, wait: true }, fade_out()]);
                out.extend([Action::LoadRegion { slot: 3, region: None }, release_camera(), fade_in(), Action::InitObjOn { obj: 8, end: false }]);
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
                    out.extend([Action::InitObjOn { obj: 0, end: false }, Action::InitObjOn { obj: 1, end: false }]);
                }
            } else if state.word(pack, 1) == 1 && !state.flag(125) {
                let current = sum.unwrap_or(0);
                if state.word(pack, 8) - current >= 31 {
                    state.set_word(pack, 1, 2);
                    out = vec![Action::DeactivateObj { obj: 0 }, Action::DeactivateObj { obj: 1 }];
                    out.extend(road_end(pack));
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
/// The setup helper of a module's shared entity-control table (SPEC-0043, Shared entity-control
/// routine, Records per module), run at its Init: the table is reset and its records added
/// under their gates. Returns the adds' idle-loop requests. Modules whose records follow story
/// stages or are parked during their cutscenes (909, 951, 1098, 1150, 1163) are not set up here.
pub fn entity_setup(pack: u32, state: &mut GameState) -> Vec<crate::entityctl::Command> {
    use crate::entityctl::{Point, Table};
    let mut t = Table::default();
    let mut out = Vec::new();
    let std3 = [(40, 2.5), (96, 2.5), (41, 1.3)];
    let std4 = [(40, 2.5), (95, 2.5), (96, 2.5), (41, 1.3)];
    let mut add = |t: &mut Table, e: usize, idle: u32, anim: (f32, f32), dwell: (f32, f32), anims: &[(u32, f32)], points: &[(usize, bool, bool)]| {
        out.extend(t.add(e, idle, anim, dwell));
        for &(a, s) in anims {
            t.add_anim(e, a, s);
        }
        for &(l, d, o) in points {
            t.add_point(e, Point::new(l, d, o));
        }
    };
    let dwell = (3.0, 4.0);
    match pack {
        750 => add(&mut t, 2, 0, (2.0, 5.0), (5.0, 7.0), &std3, &[]),
        1008 => add(&mut t, 1, 0, (5.0, 8.0), dwell, &[(95, 2.5), (96, 2.5), (41, 2.5)], &[]),
        1011 => add(&mut t, 1, 0, (5.0, 8.0), dwell, &[(41, 2.5), (95, 2.5), (96, 2.5)], &[]),
        1014 => {
            add(&mut t, 2, 0, (5.0, 8.0), dwell, &[(36, 2.5), (26, 2.5), (27, 2.5)], &[(1, true, true), (2, true, true), (3, true, true)]);
            add(&mut t, 3, 0, (5.0, 8.0), dwell, &[(95, 2.5), (96, 2.5), (41, 2.5)], &[(4, true, true), (5, true, true)]);
        }
        1020 => add(&mut t, 1, 0, (5.0, 8.0), dwell, &[(95, 2.5), (96, 2.5), (41, 2.5)], &[(1, true, true), (2, true, true), (3, true, true)]),
        1409 if state.flag(106) => {
            for e in [0, 1] {
                add(&mut t, e, 0, (8.0, 15.0), (5.0, 7.0), &std4, &[]);
            }
        }
        1414 if state.flag(102) => {
            let anims = [(110, 2.5), (95, 2.5), (40, 2.5), (41, 1.3)];
            for (e, locs) in [(7usize, 0..=2usize), (8, 3..=6), (9, 7..=11)] {
                let points: Vec<(usize, bool, bool)> = locs.map(|l| (l, true, true)).collect();
                add(&mut t, e, 25, (5.0, 7.0), (8.0, 12.0), &anims, &points);
            }
        }
        1421 if state.flag(102) => add(&mut t, 0, 0, (8.0, 15.0), (5.0, 7.0), &std4, &[]),
        1405 if state.flag(104) => add(&mut t, 0, 0, (5.0, 8.0), (5.0, 7.0), &std3, &[]),
        745 => {
            add(&mut t, 0, 98, (2.0, 3.0), dwell, &[], &[]);
            add(&mut t, 2, 98, (2.0, 3.0), dwell, &[], &[]);
            add(&mut t, 1, 74, (5.0, 8.0), dwell, &[(146, 2.5), (147, 2.5)], &[]);
            add(&mut t, 3, 0, (2.0, 5.0), (5.0, 7.0), &[(51, 2.5), (147, 1.2)], &[(8, true, true), (9, true, true), (7, true, true)]);
        }
        1173 => {
            let points = [(3, true, true), (4, true, false), (5, false, false), (6, false, false)];
            add(&mut t, 0, 0, (1.0, 3.0), dwell, &[(54, 2.5), (49, 2.5), (52, 2.5), (62, 2.5), (55, 2.5)], &points);
        }
        1407 => add(&mut t, 0, 0, (8.0, 14.0), (5.0, 7.0), &std3, &[]),
        1417 if state.flag(102) => add(&mut t, 0, 0, (8.0, 15.0), (5.0, 7.0), &std4, &[]),
        1419 if state.flag(100) => add(&mut t, 0, 0, (8.0, 14.0), (5.0, 7.0), &std3, &[]),
        1425 if state.flag(106) => add(&mut t, 0, 0, (8.0, 15.0), (5.0, 7.0), &std4, &[]),
        1423 if state.flag(106) => add(&mut t, 0, 0, (8.0, 15.0), (5.0, 7.0), &std4, &[]),
        _ => return Vec::new(),
    }
    state.entity_tables.insert(pack, t);
    out
}

/// One tick of a module's shared entity-control table (its Update's call).
pub fn entity_step(pack: u32, state: &mut GameState, talking: &dyn Fn(usize) -> bool, walking: &dyn Fn(usize) -> bool, random: &mut dyn FnMut(i32, i32) -> i32) -> Vec<crate::entityctl::Command> {
    // SPEC-0044 745 calls it only while flag 264 is clear.
    if pack == 745 && state.flag(264) {
        return Vec::new();
    }
    state.entity_tables.get_mut(&pack).map(|t| t.step(talking, walking, random)).unwrap_or_default()
}

/// The hero pickup callback (RegisterHeroPickupItemCallback) of a pack, for a picked-up item.
pub fn on_pickup(pack: u32, item: u16, state: &mut GameState) -> Vec<Action> {
    // SPEC-0037 1238 pickup P: item 83 (Pink Posies) sets flag 579 once.
    if (pack, item) == (1238, 83) && !state.flag(579) {
        state.set(579, true);
    }
    Vec::new()
}

/// The on-enter callback (RegisterOnEnterCallback, SPEC-0030 row 255), started each time the
/// world becomes current, right after the activate-world callback.
pub fn on_enter(pack: u32, state: &mut GameState) -> Vec<Action> {
    match pack {
        // SPEC-0039 984 on-enter (handler 2): with 67 entity 0's line 0 in the conversation
        // frame; else line 2 (202), 4 (150) or 6.
        984 if state.flag(67) => vec![Action::Converse { entity: 0, line: 0 }],
        984 => vec![Action::Message { line: if state.flag(202) { 2 } else if state.flag(150) { 4 } else { 6 } }],
        // SPEC-0044 745 routine E (on-enter): two lines in turn (word 20) by 569 (0, 1), 567 (2,
        // 3), 76 (4, 5); with 75 set and 264 clear the item exchange when all three are held
        // (else nothing); otherwise 6, 7.
        745 => {
            let pair = |state: &mut GameState, a: u16| {
                let c = state.word(745, 20).rem_euclid(2);
                state.set_word(745, 20, (c + 1) % 2);
                vec![Action::Message { line: a + c as u16 }]
            };
            if state.flag(569) {
                pair(state, 0)
            } else if state.flag(567) {
                pair(state, 2)
            } else if state.flag(76) {
                pair(state, 4)
            } else if state.flag(75) && !state.flag(264) {
                exchange_745(state).unwrap_or_default()
            } else {
                pair(state, 6)
            }
        }
        // SPEC-0039 1417 on-enter (registered with 102): with 100, line 0 (106) or 2. 1419 (with
        // 100): line 0 with 102 and 104, else 2.
        1417 if state.flag(102) && state.flag(100) => vec![Action::Message { line: if state.flag(106) { 0 } else { 2 } }],
        1419 if state.flag(100) => vec![Action::Message { line: if state.flag(102) && state.flag(104) { 0 } else { 2 } }],
        // SPEC-0041 1421 CB1 (registered with 102): with 100, line 0 (106) or 2. 1423 CB1
        // (registered with 106): with 104, line 0 (102) or 3.
        1421 if state.flag(102) && state.flag(100) => vec![Action::Message { line: if state.flag(106) { 0 } else { 2 } }],
        1423 if state.flag(106) && state.flag(104) => vec![Action::Message { line: if state.flag(102) { 0 } else { 3 } }],
        // SPEC-0043 750 on-enter (Medium): two lines in turn per case, 27/28 (421), 29/30 (420),
        // 31/32 (76) or 33/34, with the routine's shared counter.
        750 => {
            let (f421, f420, f76) = (state.flag(421), state.flag(420), state.flag(76));
            let lines = if f421 { [27, 28] } else if f420 { [29, 30] } else if f76 { [31, 32] } else { [33, 34] };
            cycle_750(state, 0, f421, &lines)
        }
        // SPEC-0037 1251 on-enter E: 460: 18 (273) or 19; 457: 20 (item 49) or 21; else 22
        // (item 49) or 23; in entity 0's conversation frame.
        1251 => {
            let held = state.has_item(49);
            let line = if state.flag(460) {
                if state.flag(273) { 18 } else { 19 }
            } else if state.flag(457) {
                if held { 20 } else { 21 }
            } else if held {
                22
            } else {
                23
            };
            vec![Action::Converse { entity: 0, line }]
        }
        _ => Vec::new(),
    }
}

pub fn on_activate(pack: u32, state: &mut GameState) -> Vec<Action> {
    match pack {
        // SPEC-0043 1358 activate-world: the value for eW 159 (10 to 50 to locators 2 to 6, 60 to
        // 77); with any value the normal camera; the value reset.
        1358 => {
            let v = state.take_message(2, 159);
            let mut out = Vec::new();
            match v {
                10 | 20 | 30 | 40 | 50 => out.push(Action::MoveHero { locator: (v / 10 + 1) as usize }),
                60 => out.push(Action::MoveHero { locator: 77 }),
                _ => {}
            }
            if v != -1 {
                out.push(Action::ReturnCamera { preset: -1, release: true });
            }
            out
        }
        // SPEC-0043 1173 activate-world: the value for eW 126; 120 locator 2, 110 locator 0, 130
        // locator 1 and flag 319; the value reset and the normal camera in every case.
        1173 => {
            let mut out = match state.take_message(2, 126) {
                120 => vec![Action::MoveHero { locator: 2 }],
                110 => vec![Action::MoveHero { locator: 0 }],
                130 => {
                    state.set(319, true);
                    vec![Action::MoveHero { locator: 1 }]
                }
                _ => Vec::new(),
            };
            out.push(Action::ReturnCamera { preset: -1, release: true });
            out
        }
        // SPEC-0039 1139 activate-world: the value for eW 114; 10 places the hero at locator 1,
        // 20 at 0; the normal camera.
        1139 => {
            let mut out = match state.take_message(2, 114) {
                10 => vec![Action::MoveHero { locator: 1 }],
                20 => vec![Action::MoveHero { locator: 0 }],
                _ => Vec::new(),
            };
            out.push(Action::ReturnCamera { preset: -1, release: true });
            out
        }
        // SPEC-0039 732 activate-world: the value for eW 21 (10, 20, 30, 40, 60 give locators 1
        // to 5 and tracks 8, 7, 6, 5, 4): the hero to the locator, a 0.5 s fade in, the track
        // (PlayTrackHeroCamera, played as a track; StartHeroLongFall is not built) and 1.5 s;
        // in every case the value reset, the fade in and the normal camera.
        732 => {
            let pick = match state.take_message(2, 21) {
                10 => Some((1, 8)),
                20 => Some((2, 7)),
                30 => Some((3, 6)),
                40 => Some((4, 5)),
                60 => Some((5, 4)),
                _ => None,
            };
            let mut out: Vec<Action> = pick.map(|(locator, _)| Action::MoveHero { locator }).into_iter().collect();
            out.push(Action::FadeIn { seconds: 0.5, wait: false });
            if let Some((_, track)) = pick {
                out.extend([Action::PlayCamera { track, wait: false, end: 1, speed: 1.0 }, wait(1.5)]);
            }
            out.push(Action::ReturnCamera { preset: -1, release: true });
            out
        }
        // SPEC-0041 1414 activate-world: the value for eW 180; 50 places the hero at locator 13
        // with the normal camera, 10 at locator 12; the value is reset on every path.
        1414 => match state.take_message(2, 180) {
            50 => vec![Action::MoveHero { locator: 13 }, Action::ReturnCamera { preset: -1, release: true }],
            10 => vec![Action::MoveHero { locator: 12 }],
            _ => Vec::new(),
        },
        // SPEC-0037 1299 activate-world: the value for eW 150 is read and reset; any value but -1
        // returns the normal camera.
        1299 => {
            if state.take_message(2, 150) != -1 { vec![Action::ReturnCamera { preset: -1, release: true }] } else { Vec::new() }
        }
        // SPEC-0037 1220 activate-world A2 (Medium order): with value 1 (type 3) for eW 131 a
        // cut to black, 1 s, a 0.5 s fade in, the hero to locator 25, camera 4 (wait), 0.5 s and
        // the normal camera; value -1 for eW 132 (as the spec gives). StartHeroLongFall is not
        // built.
        1220 => {
            if state.take_message(3, 131) != 1 {
                return Vec::new();
            }
            state.blackboard.insert((3, 132), -1);
            vec![
                Action::FadeOut { seconds: 0.0, wait: false },
                wait(1.0),
                Action::FadeIn { seconds: 0.5, wait: false },
                Action::MoveHero { locator: 25 },
                Action::PlayCamera { track: 4, wait: true, end: 1, speed: 0.5 },
                wait(0.5),
                Action::ReturnCamera { preset: -1, release: true },
            ]
        }
        // SPEC-0037 1253 activate-world: every arrival, by the value for eW 143 (20: locator 1
        // and track 6; 30: 2 and 7; else 0 and 5): the hero to the locator, a 0.5 s fade in, the
        // track played with wait (PlayTrackHeroCamera, taken as a camera track here; the long
        // fall is not built), 1.5 s, the normal camera.
        1253 => {
            let (locator, track) = match state.take_message(2, 143) {
                20 => (1, 6),
                30 => (2, 7),
                _ => (0, 5),
            };
            vec![
                Action::MoveHero { locator },
                Action::FadeIn { seconds: 0.5, wait: false },
                Action::PlayCamera { track, wait: true, end: 1, speed: 1.0 },
                wait(1.5),
                Action::ReturnCamera { preset: -1, release: true },
            ]
        }
        // SPEC-0037 1152 activate-world: the value for eW 119; 110 places the hero at locator 1,
        // any other value at locator 0; the normal camera (10 s) in both cases.
        1152 => {
            let v = state.take_message(2, 119);
            vec![Action::MoveHero { locator: if v == 110 { 1 } else { 0 } }, Action::ReturnCamera { preset: -1, release: true }]
        }
        // SPEC-0037 783 to 789 activate-world: the value for eW 31; this interior's value moves
        // the hero to its locator; any value returns the normal camera; the value is reset.
        783 | 786 | 787 | 788 | 789 => {
            let (value, locator) = eastside_interior(pack).unwrap_or_default();
            let v = state.take_message(2, 31);
            let mut out = Vec::new();
            if v == value {
                out.push(Action::MoveHero { locator });
            }
            if v != -1 {
                out.push(Action::ReturnCamera { preset: -1, release: true });
            }
            out
        }
        // SPEC-0044 1017 activate-world: the value for eW 81 places the hero at locator 2 (50) or
        // 3 (60); the normal camera; the value reset.
        1017 => {
            let v = state.take_message(2, 81);
            let mut out = Vec::new();
            match v {
                50 => out.push(Action::MoveHero { locator: 2 }),
                60 => out.push(Action::MoveHero { locator: 3 }),
                _ => {}
            }
            if v != -1 {
                out.push(Action::ReturnCamera { preset: -1, release: true });
            }
            out
        }
        // SPEC-0045 991 activate-world: the type-2 value for eW 81 places the hero at the return
        // locator of the teleport used (10 -> 3 ... 80 -> 10); any value but -1 returns the
        // normal camera; the value is reset.
        991 => {
            let v = state.take_message(2, 81);
            let mut out = Vec::new();
            if (10..=80).contains(&v) && v % 10 == 0 {
                out.push(Action::MoveHero { locator: (v / 10 + 2) as usize });
            }
            if v != -1 {
                out.push(Action::ReturnCamera { preset: -1, release: true });
            }
            out
        }
        // SPEC-0039 880 activate-world (handler 8): the type-2 value for eW 81; 80 places the
        // hero at locator 2; any value but -1 returns the normal camera; the value is reset.
        880 => {
            let v = state.take_message(2, 81);
            let mut out = Vec::new();
            if v == 80 {
                out.push(Action::MoveHero { locator: 2 });
            }
            if v != -1 {
                out.push(Action::ReturnCamera { preset: -1, release: true });
            }
            out
        }
        1184 => woods_activate(state),
        1098 => castle_activate(state),
        1056 => yard_activate(state),
        1163 => kitchen_activate(state),
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
                    actions.extend([Action::WaitLoads, Action::DeactivateObj { obj: 8 }, Action::HoldHero, Action::MoveHero { locator: 13 }, camera(10)]);
                    actions.extend([Action::Converse { entity: 47, line: 168 }, Action::Converse { entity: 39, line: 169 }]);
                    for (entity, locator) in [(47, 10), (39, 11), (33, 9)] {
                        actions.push(Action::MoveEntity { entity, locator, face: true });
                    }
                    actions.extend([camera(11), Action::WalkHero { locator: 14, wait: true, orient: false }, fade_out(), Action::InitObjOn { obj: 8, end: false }, release_camera(), fade_in()]);
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

/// The templates: template C's flag-138 line first, then the fame rule, then the story chain.
fn village_talk(k: u8, s: &mut GameState, rot: &mut Rotation) -> Option<u16> {
    let (_, template, b) = *VILLAGERS.get(k as usize)?;
    let three = |o: u16| cycle(&[b + o, b + o + 1, b + o + 2]);
    if template <= 2 && !(template == 2 && s.flag(138) && !s.flag(141)) {
        if let Some(l) = fame(s, rot, b) {
            return l;
        }
    }
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

/// SPEC-0042 1184 Init: trigger 0 (and flag 661), the 220-dependent triggers 7 to 9 or 11, and
/// thread A's slot-B load by the story set (regions 1215, 1200 or 1201, 1192 or 1193). Thread
/// C's loads (triggers 2 to 6) read a 21-entry list in the module data that no spec gives
/// (R-0036), so those triggers are not registered. Update's display and collision masks and E8's
/// world-load override are not reproduced.
fn woods_init(state: &mut GameState) -> Vec<Action> {
    let reg = |index, routine| Action::RegisterTrigger { index, routine };
    let mut out = vec![reg(0, 1)];
    if !state.flag(661) {
        state.set(661, true);
    }
    if state.flag(220) {
        out.extend([reg(7, 3), reg(9, 5)]);
    } else {
        out.push(reg(11, 6));
    }
    let alt = state.flag(512) && !state.flag(513);
    let j = if state.flag(84) {
        Some(30)
    } else if state.flag(83) {
        None
    } else if state.flag(82) {
        Some(if alt { 16 } else { 15 })
    } else {
        Some(if alt { 8 } else { 7 })
    };
    if let Some(j) = j {
        out.push(Action::LoadRegion { slot: 1, region: Some(1184 + 1 + j) });
    }
    out
}

/// SPEC-0042 1184 activate-world A: the type-3 value for eW 129. 1 arrives at locator 45 and
/// walks to 46 under camera 10; 2 places the hero at locator 50 (both clear the values).
fn woods_activate(state: &mut GameState) -> Vec<Action> {
    match state.take_message(3, 129) {
        1 => vec![
            Action::MoveHero { locator: 45 },
            Action::FadeIn { seconds: 0.5, wait: false },
            Action::PlayCamera { track: 10, wait: false, end: 1, speed: 1.0 },
            Action::WalkHero { locator: 46, wait: true, orient: false },
            release_camera(),
        ],
        2 => {
            state.blackboard.remove(&(2, 129));
            vec![Action::Sleep { ticks: 1 }, Action::MoveHero { locator: 50 }, release_camera()]
        }
        _ => Vec::new(),
    }
}

/// The Miamice game start (and restart): level state 0 on and 1 to 5 off, InitMiamouseBehavior
/// with the field mesh 0 and the cage mesh 1, and the game word (module word 0).
fn mice_start(state: &mut GameState) -> Vec<Action> {
    state.set_word(1163, 0, 1);
    let mut v = vec![Action::LevelState { flag: 0, on: true }];
    v.extend((1..=5).map(|flag| Action::LevelState { flag, on: false }));
    v.push(Action::MiamouseInit { field: 0, cage: 1 });
    v
}

/// SPEC-0045 1163 Init: trigger 0 with 155 clear (or s 30) and the slot thread (slot word 1
/// with j 2, 1 or 0 by flags 31, 11 and 30; slot word 4 with j 5 while 29 is clear). The
/// shared entity-control records are not run.
fn kitchen_init(state: &mut GameState) -> Vec<Action> {
    let mut out = Vec::new();
    if !state.flag(155) || castle_stage(state) == 30 {
        out.extend([Action::InitObjOn { obj: 0, end: false }, Action::RegisterTrigger { index: 0, routine: 1 }]);
    }
    let j = if state.flag(31) || state.flag(11) {
        2
    } else if state.flag(30) {
        1
    } else {
        0
    };
    out.push(Action::LoadRegion { slot: 0, region: Some(1163 + 1 + j) });
    if !state.flag(29) {
        out.push(Action::LoadRegion { slot: 3, region: Some(1163 + 1 + 5) });
    }
    out
}

/// SPEC-0045 1163 activate-world: a game in progress (156 set; 30, 31 and 158 clear) restarts;
/// otherwise catching is turned off. Then the type-2 value for eW 122 places the hero.
fn kitchen_activate(state: &mut GameState) -> Vec<Action> {
    let mut out = if !state.flag(30) && !state.flag(31) && !state.flag(158) && state.flag(156) { mice_start(state) } else { vec![Action::MiamouseDisable] };
    match state.take_message(2, 122) {
        50 => out.extend([Action::MoveHero { locator: 15 }, release_camera()]),
        100 => out.extend([Action::MoveHero { locator: 0 }, release_camera()]),
        _ => {}
    }
    out
}

/// SPEC-0045 1163 Update while the game word is set: level state 5 wins (flags 158 and 23, the
/// keeper's line 4); level states 1 to 4 show lines 43 to 46 the first time each is seen on, in
/// order (the phase word, module word 1).
fn kitchen_update(state: &mut GameState) -> Vec<Action> {
    if state.word(1163, 0) == 0 {
        return Vec::new();
    }
    if state.level(5) {
        state.set(158, true);
        state.set(23, true);
        state.set_word(1163, 0, 0);
        return vec![Action::Converse { entity: 0, line: 4 }];
    }
    let phase = state.word(1163, 1);
    for n in 1..=4u16 {
        if state.level(n) && phase < n as i32 {
            state.set_word(1163, 1, n as i32);
            return vec![Action::Message { line: 42 + n }];
        }
    }
    Vec::new()
}

/// SPEC-0045 1163 proximity routines: the keeper (0), entity 7 (lines only; its serving scene
/// with flags 25 and 635 is not built) and Sir Harlag (15, with his cutscene as its lines).
fn kitchen_talk(r: u8, s: &mut GameState, rot: &mut Rotation) -> Vec<Action> {
    let three = |b: u16| cycle(&[b, b + 1, b + 2]);
    let talk = |line: Option<u16>| line.map(|line| vec![Action::Talk { line }]).unwrap_or_default();
    match r {
        0 => {
            if s.flag(30) {
                talk(Some(0))
            } else if s.flag(160) {
                talk(rot.pick(0, &three(1)))
            } else if s.flag(158) {
                talk(Some(4))
            } else if s.flag(156) {
                talk(rot.pick(1, &three(5)))
            } else if s.flag(632) {
                s.set(156, true);
                let mut v = mice_start(s);
                v.push(Action::Talk { line: 8 });
                v
            } else {
                talk(rot.pick(2, &three(9)))
            }
        }
        7 => talk(if s.flag(29) {
            rot.pick(0, &three(49))
        } else if s.flag(28) {
            Some(52)
        } else if s.flag(27) {
            rot.pick(1, &three(53))
        } else if s.flag(25) && s.flag(635) {
            if s.flag(185) {
                rot.pick(2, &three(59))
            } else if s.flag(183) {
                rot.pick(3, &three(62))
            } else {
                Some(65)
            }
        } else {
            rot.pick(4, &three(56))
        }),
        15 => {
            if s.flag(200) {
                talk(rot.pick(0, &cycle(&[85, 86])))
            } else if s.flag(195) {
                talk(rot.pick(1, &three(84)))
            } else if s.flag(194) {
                s.set(195, true);
                (21..=26).map(|line| Action::Converse { entity: 15, line }).collect()
            } else {
                talk(rot.pick(2, &three(87)))
            }
        }
        _ => Vec::new(),
    }
}

/// SPEC-0044 1056 Init: the lesson flags (23 from 158, 24 from 167, 26 from 180 or 185), triggers
/// 0 and 1 and the zone triggers 5, 6, 7, 9, 10, 11, 12 and 14, and the setup thread's slot
/// loads and trainer choice. The breakables, the other dialogue routines (their entities come
/// from a module table), the prompt routines and the shield lesson's launcher are not built.
fn yard_init(state: &mut GameState) -> Vec<Action> {
    let reg = |index, routine| Action::RegisterTrigger { index, routine };
    if state.flag(158) {
        state.set(23, true);
    }
    if state.flag(167) {
        state.set(24, true);
    }
    if state.flag(180) || state.flag(185) {
        state.set(26, true);
    }
    let mut out = vec![reg(0, 1), reg(1, 2)];
    out.extend([5, 6, 7, 9, 10, 11, 12, 14].map(|i| reg(i, 5)));
    let load = |slot, j: u32| Action::LoadRegion { slot, region: Some(1056 + 1 + j) };
    let (a, b): (Option<u32>, Option<u32>) = if state.flag(30) {
        (Some(6), None)
    } else if state.flag(191) && !state.flag(29) {
        (None, Some(7))
    } else if state.flag(26) {
        (Some(4), Some(7))
    } else if state.flag(24) {
        (Some(2), Some(if state.flag(174) { 7 } else { 8 }))
    } else if state.flag(23) {
        (Some(1), Some(7))
    } else {
        (state.flag(22).then_some(0), Some(7))
    };
    out.extend(a.map(|j| load(0, j)));
    out.extend(b.map(|j| load(1, j)));
    // The trainer per stage, stored +1 in module word 11 (0: none).
    let trainer = if state.flag(30) || state.flag(191) {
        0
    } else if state.flag(26) {
        25
    } else if state.flag(24) {
        13
    } else if state.flag(23) {
        11
    } else {
        6
    };
    state.set_word(1056, 11, trainer);
    out
}

/// SPEC-0044 1056 arrival: the type-2 value for eW 92 (30 locator 4, 40 locator 2, 50 locator
/// 3), reset to 0.
fn yard_activate(state: &mut GameState) -> Vec<Action> {
    let v = state.take_message(2, 92);
    state.blackboard.insert((2, 92), 0);
    match v {
        30 => vec![Action::MoveHero { locator: 4 }, release_camera()],
        40 => vec![Action::MoveHero { locator: 2 }, release_camera()],
        50 => vec![Action::MoveHero { locator: 3 }, release_camera()],
        _ => Vec::new(),
    }
}

/// The entities 1056's Update makes undamageable each tick (the lesson targets).
pub fn undamageable(pack: u32) -> &'static [usize] {
    match pack {
        1056 => &[4, 7, 8, 9, 20, 21, 14],
        // SPEC-0037 784 thread K's SetEntityUndamageable(entity 0) (taken for every load).
        784 => &[0],
        _ => &[],
    }
}

/// SPEC-0044 1056 trainer routine (lines 0 to 70; nothing while the hero talks). Each branch
/// sets its marker flag; "cycle" is three lines in turn.
fn trainer_talk(s: &mut GameState, rot: &mut Rotation) -> Vec<Action> {
    let trainer = s.word(1056, 11) - 1;
    if trainer < 0 || s.talking {
        return Vec::new();
    }
    let say = |line: Option<u16>| line.map(|line| vec![Action::Converse { entity: trainer as usize, line }]).unwrap_or_default();
    let three = |b: u16| cycle(&[b, b + 1, b + 2]);
    if s.flag(26) {
        if s.flag(190) {
            s.set(191, true);
            return say(rot.pick(0, &cycle(&[0, 1, 2])));
        }
        if s.flag(189) {
            return say(Some(3));
        }
        if s.flag(188) {
            return say(Some(4));
        }
        s.set(187, true);
        s.give(160, 20);
        s.set(188, true);
        return [say(Some(5)), say(Some(4))].concat();
    }
    if s.flag(25) {
        if s.flag(182) {
            s.set(635, true);
            return say(rot.pick(1, &three(65)));
        }
        s.set(181, true);
        return say(rot.pick(2, &three(68)));
    }
    if s.flag(24) {
        return if s.flag(179) {
            say(Some(64))
        } else if s.flag(174) {
            say(rot.pick(3, &three(61)))
        } else if s.flag(173) {
            s.set(634, true);
            say(rot.pick(4, &three(6)))
        } else if s.flag(658) {
            say(Some(9))
        } else if s.flag(172) && s.flag(171) {
            say(Some(10))
        } else if s.flag(171) {
            say(Some(11))
        } else if s.flag(170) {
            say(Some(12))
        } else {
            s.set(169, true);
            say(Some(13))
        };
    }
    if s.flag(23) {
        return if s.flag(161) {
            s.set(633, true);
            say(rot.pick(5, &three(14)))
        } else if s.flag(648) {
            s.set(633, true);
            say(rot.pick(6, &cycle(&[32, 15, 16])))
        } else {
            s.set(160, true);
            say(rot.pick(7, &three(33)))
        };
    }
    if s.flag(155) || s.flag(647) {
        s.set(632, true);
        return say(rot.pick(8, &three(36)));
    }
    s.set(154, true);
    say(rot.pick(9, &three(58)))
}

/// The remark cycles (remark 1 to 11).
const REMARKS: [[u16; 3]; 11] = [
    [54, 53, 52],
    [48, 47, 46],
    [51, 50, 49],
    [45, 44, 43],
    [42, 41, 40],
    [57, 56, 55],
    [28, 27, 26],
    [19, 18, 17],
    [25, 24, 23],
    [31, 30, 29],
    [22, 21, 20],
];

/// SPEC-0044 1056 Update (per tick): the melee edge (level states 2 and 3), the lesson of the
/// first set stage flag while 191 is clear (the chain, spin and mote lessons; the shield
/// lesson's launcher is not built), the remarks and the auto-talk. Module words: 0 the zone
/// marks of this tick, 1 the spin marks, 2 the previous sub-state, 3 the pending remark, 4 the
/// chain's hit on entity 4, 5 to 7 the last jab start ticks, 9 a pending auto-talk, 10 the
/// mote item's activation, 11 the trainer + 1.
fn yard_update(s: &mut GameState, dialog: bool) -> Vec<Action> {
    let w = |s: &GameState, k: u8| s.word(1056, k);
    let zones = w(s, 0);
    let inside = |i: u16| zones & (1 << i) != 0;
    let sub = s.hero.sub;
    let prev = w(s, 2) as u8;
    let started = sub != prev && sub != 0;
    let ended = prev != 0 && sub != prev;
    s.set_word(1056, 2, sub as i32);
    // The melee edge.
    if sub != 0 {
        let first = !s.level(2);
        s.set_level(3, first);
        s.set_level(2, true);
    } else {
        s.set_level(2, false);
        s.set_level(3, false);
    }
    let hit = |s: &GameState, e: usize| s.hero.hits.contains(&e);
    let mut talk_now = false;
    let mut remark = 0;
    if !s.flag(191) {
        if s.flag(26) {
            // The shield lesson: the trainer's first talk; the launcher is not built.
            if !s.flag(188) {
                talk_now = true;
            }
        } else if s.flag(24) {
            let fire = |m: Option<u16>| matches!(m, Some(160 | 161));
            if s.flag(173) {
            } else if s.flag(658) {
                s.set(173, true);
                talk_now = true;
            } else if !s.flag(169) {
                talk_now = true;
            } else {
                if !s.flag(170) && s.has_item(160) {
                    s.set(170, true);
                    talk_now = true;
                } else if s.flag(170) && !s.flag(171) && fire(s.hero.attack_mote) {
                    s.set(171, true);
                    s.set_word(1056, 9, 1);
                } else if s.flag(171) && !s.flag(172) && sub != 0 && hit(s, 14) {
                    s.set(172, true);
                    s.set_word(1056, 9, 1);
                } else if s.flag(171) && s.flag(172) && sub == 10 && hit(s, 14) {
                    s.set(658, true);
                    talk_now = true;
                }
                if s.flag(170) && !s.flag(658) && sub == 0 && !fire(s.hero.attack_mote) {
                    s.set(171, false);
                }
            }
        } else if s.flag(23) {
            if !s.flag(160) {
                talk_now = true;
            } else if !s.flag(161) {
                let spin = matches!(sub, 7 | 8);
                let mut marks = w(s, 1);
                if spin {
                    for (k, e) in [7, 8, 9].into_iter().enumerate() {
                        if hit(s, e) {
                            marks |= 1 << k;
                        }
                    }
                } else {
                    marks = 0;
                }
                s.set_word(1056, 1, marks);
                if marks == 7 {
                    s.set(161, true);
                    s.set_word(1056, 9, 1);
                } else if inside(6) {
                    remark = match sub {
                        4 if started => 8,
                        12 if started => 9,
                        9 if started => 10,
                        _ if ended && matches!(prev, 7 | 8) => 11,
                        0 if prev == 3 => 7,
                        _ => 0,
                    };
                }
            }
        } else if s.flag(22) {
            if !s.flag(154) {
                talk_now = true;
            } else if !s.flag(155) {
                if sub == 4 && hit(s, 4) {
                    s.set(155, true);
                    talk_now = true;
                } else if inside(5) {
                    if sub == 4 && s.hero.hits.contains(&4) {
                        s.set_word(1056, 4, 1);
                    }
                    remark = match sub {
                        _ if ended && prev == 4 && w(s, 4) == 0 => 5,
                        _ if started && matches!(sub, 7 | 8) => 2,
                        12 if started => 3,
                        9 if started => 4,
                        0 if prev == 3 => 1,
                        _ => 0,
                    };
                    if ended && prev == 4 {
                        s.set_word(1056, 4, 0);
                    }
                    // The jab tracker: three starts of attack state 2 within 3.0 s.
                    if started && sub == 2 {
                        let t = s.hero.tick as i32;
                        let (a, b) = (w(s, 5), w(s, 6));
                        s.set_word(1056, 5, b);
                        s.set_word(1056, 6, t);
                        if a != 0 && t - a <= 180 {
                            remark = 6;
                        }
                    }
                }
            }
        }
    }
    if remark != 0 {
        s.set_word(1056, 3, remark);
    }
    if w(s, 9) != 0 && sub == 0 {
        s.set_word(1056, 9, 0);
        talk_now = true;
    }
    let mut out = Vec::new();
    let trainer = w(s, 11) - 1;
    // The stage's talk zone: chain 10, spin 11, mote 12, shield 14.
    let zone = if s.flag(26) {
        14
    } else if s.flag(24) {
        12
    } else if s.flag(23) {
        11
    } else {
        10
    };
    let free = !dialog && !s.hero.occupied && !s.talking;
    if talk_now && trainer >= 0 && free && inside(zone) {
        let mut rot = s.rotations.get(&(1056, 0)).copied().unwrap_or_default();
        out = trainer_talk(s, &mut rot);
        s.rotations.insert((1056, 0), rot);
    } else if talk_now {
        // Not in the talk zone yet: the talk waits.
        s.set_word(1056, 9, 1);
    }
    // A pending remark is spoken on the first tick with no melee attack active.
    let pending = w(s, 3);
    if out.is_empty() && pending > 0 && sub == 0 && trainer >= 0 && free {
        s.set_word(1056, 3, 0);
        let mut rot = s.rotations.get(&(1056, 60)).copied().unwrap_or_default();
        let lines = REMARKS[(pending - 1) as usize];
        if let Some(line) = rot.pick(pending as u16, &cycle(&lines)) {
            out.push(Action::Converse { entity: trainer as usize, line });
        }
        s.rotations.insert((1056, 60), rot);
    }
    // The mote area: magic refilled; with no fire alignment and no item 160, 160 x100.
    if s.flag(24) && !s.flag(191) && inside(7) {
        out.push(Action::RefillMagic);
        if !matches!(s.hero.attack_mote, Some(160 | 161)) && !s.has_item(160) {
            s.give(160, 100);
        }
    }
    // The zone marks last one tick.
    s.set_word(1056, 0, 0);
    out
}

/// SPEC-0045 1098: the castle stage s, the highest of flags 31 down to 21 that is set.
fn castle_stage(s: &GameState) -> u16 {
    (21..=31).rev().find(|&f| s.flag(f)).unwrap_or(0)
}

/// SPEC-0045 1098 Init: the flag steps (21; 25 with 182; 202 and 28 with 201; 151), the rest
/// prompt (trigger 0), trigger 1 with 30 and 31 clear, the slot triggers 11, 12 and 14, the
/// slot thread's loads (31: j 1; 30: j 9 and j 0; 501 with item 72 not held and 503 and 504
/// clear: j 21), and the module word that blocks teleport 6 when s is 31. The other story
/// triggers (4 to 10, 17 to 20), the objects, effects, navmesh steps and the hero-death
/// callback are not built.
fn castle_init(state: &mut GameState) -> Vec<Action> {
    let reg = |index, routine| Action::RegisterTrigger { index, routine };
    state.set(21, true);
    if state.flag(182) {
        state.set(25, true);
    }
    if state.flag(201) {
        state.set(202, true);
        state.set(28, true);
    }
    if !state.flag(151) {
        state.set(151, true);
    }
    if castle_stage(state) == 31 {
        state.set_word(1098, 0, 1);
    }
    let mut out = vec![reg(0, 1)];
    let (f30, f31) = (state.flag(30), state.flag(31));
    if !f30 && !f31 {
        out.extend([reg(1, 2), reg(11, 11), reg(12, 12), reg(14, 14)]);
    }
    if f31 {
        out.push(Action::LoadRegion { slot: 4, region: Some(1098 + 1 + 1) });
    } else if f30 {
        out.extend([Action::LoadRegion { slot: 0, region: Some(1098 + 1 + 9) }, Action::LoadRegion { slot: 4, region: Some(1098 + 1) }]);
    } else if state.flag(501) && !state.has_item(72) && !state.flag(504) && !state.flag(503) {
        out.push(Action::LoadRegion { slot: 3, region: Some(1098 + 1 + 21) });
    }
    out
}

/// SPEC-0045 1098 activate-world: the type-2 value for eW 106 picks the return locator.
fn castle_activate(state: &mut GameState) -> Vec<Action> {
    let locator = match state.take_message(2, 106) {
        10 => 6,
        20 => 5,
        30 => 9,
        40 => 8,
        50 => 7,
        60 => 4,
        70 => 3,
        90 => 1,
        100 => 10,
        _ => return Vec::new(),
    };
    vec![Action::MoveHero { locator }, release_camera()]
}

/// The rest prompt's press (trigger 0).
fn castle_rest(state: &mut GameState) -> Vec<Action> {
    let mut v = vec![Action::HoldHero, Action::FadeOut { seconds: 1.5, wait: true }, Action::MoveHero { locator: 12 }];
    if !state.flag(22) && state.flag(152) {
        for f in [22, 642, 153] {
            state.set(f, true);
        }
        v.extend([Action::LoadRegion { slot: 0, region: Some(1098 + 1 + 5) }, Action::LoadRegion { slot: 1, region: Some(1098 + 1 + 4) }, Action::WaitLoads]);
        v.extend([camera(9), Action::FadeIn { seconds: 1.0, wait: false }, Action::Message { line: 50 }, Action::FadeOut { seconds: 1.0, wait: true }, release_camera()]);
    } else if !state.flag(27) && state.flag(191) {
        state.set(27, true);
        state.set(193, true);
        v.extend([Action::LoadRegion { slot: 0, region: Some(1098 + 1 + 5) }, Action::LoadRegion { slot: 1, region: Some(1098 + 1 + 4) }, Action::WaitLoads]);
        v.extend([camera(9), Action::FadeIn { seconds: 1.0, wait: false }, Action::Message { line: 376 }, Action::FadeOut { seconds: 1.0, wait: true }, release_camera()]);
    } else if !state.flag(29) && state.flag(204) {
        for f in [29, 205, 63] {
            state.set(f, true);
        }
    }
    v.push(Action::FadeIn { seconds: 1.0, wait: true });
    v
}

/// The template-W entities of 1098 and their first lines (SPEC-0045 1098 Proximity routines).
const CASTLE_W: [(&[usize], u16); 13] = [
    (&[44], 97),
    (&[47], 115),
    (&[38, 43], 133),
    (&[46], 151),
    (&[45], 169),
    (&[15], 205),
    (&[40, 42], 223),
    (&[32], 241),
    (&[41], 259),
    (&[48], 277),
    (&[16], 295),
    (&[37], 313),
    (&[50], 349),
];

/// The proximity routine of a 1098 entity (shared routines share one number).
fn castle_routine(e: usize) -> Option<u8> {
    Some(match e {
        39 => 0,
        35 | 36 => 1,
        21 => 2,
        20 => 3,
        24 => 4,
        25 => 5,
        49 => 6,
        34 => 7,
        19 | 22 | 31 => 8,
        23 | 33 => 9,
        17 | 18 => 10,
        _ => 20 + CASTLE_W.iter().position(|(es, _)| es.contains(&e))? as u8,
    })
}

/// SPEC-0045 1098 proximity routines. The two story scenes (entity 39's cameras 20 to 24 with
/// lines 30 to 41, and the trigger-10 scene of lines 19 to 23) are shown as their lines in
/// order, without their cameras.
fn castle_talk(r: u8, s: &mut GameState, rot: &mut Rotation) -> Vec<Action> {
    let three = |b: u16| cycle(&[b, b + 1, b + 2]);
    let talk = |line: Option<u16>| line.map(|line| vec![Action::Talk { line }]).unwrap_or_default();
    let one = |l: u16| vec![Action::Talk { line: l }];
    match r {
        0 if s.flag(205) && !s.flag(206) => {
            s.set(206, true);
            one(3)
        }
        0 if s.flag(196) => talk(rot.pick(0, &three(0))),
        0 => {
            s.set(196, true);
            (30..=41).map(|line| Action::Converse { entity: 39, line }).collect()
        }
        1 => {
            if s.flag(204) {
                one(4)
            } else if s.flag(203) {
                one(5)
            } else if s.flag(202) {
                one(6)
            } else if s.flag(28) {
                s.set(202, true);
                one(7)
            } else if s.flag(194) {
                talk(rot.pick(0, &three(8)))
            } else if s.flag(193) {
                s.set(194, true);
                (19..=23).map(|line| Action::Talk { line }).collect()
            } else if s.flag(152) {
                talk(rot.pick(1, &three(11)))
            } else {
                s.set(152, true);
                s.set(151, true);
                one(14)
            }
        }
        2 => one(if s.flag(204) { if s.flag(503) { 52 } else { 53 } } else if s.flag(203) { 54 } else { 55 }),
        3 => one(if s.flag(204) { if s.flag(504) { 56 } else { 57 } } else if s.flag(203) { 58 } else { 59 }),
        4 => one(if s.flag(204) { 60 } else if s.flag(203) { 61 } else { 62 }),
        5 if !s.flag(204) && !s.flag(203) => one(385),
        5 => Vec::new(),
        6 => {
            if s.flag(29) {
                talk(rot.pick(0, &three(63)))
            } else if s.flag(28) {
                one(66)
            } else if s.flag(27) {
                talk(rot.pick(1, &three(67)))
            } else if s.flag(24) && s.flag(634) {
                if s.flag(179) {
                    s.set(180, true);
                    s.set(185, true);
                    talk(rot.pick(2, &three(70)))
                } else if s.flag(174) {
                    talk(rot.pick(3, &three(73)))
                } else {
                    s.set(174, true);
                    one(76)
                }
            } else {
                talk(rot.pick(4, &three(77)))
            }
        }
        7 => {
            if s.flag(29) {
                talk(rot.pick(0, &three(80)))
            } else if s.flag(28) {
                one(83)
            } else if s.flag(27) {
                talk(rot.pick(1, &three(84)))
            } else if s.flag(25) && s.flag(635) {
                if s.flag(185) {
                    talk(rot.pick(2, &three(90)))
                } else if s.flag(183) {
                    talk(rot.pick(3, &three(93)))
                } else {
                    s.set(183, true);
                    one(96)
                }
            } else {
                talk(rot.pick(4, &three(87)))
            }
        }
        8 => talk(if s.flag(29) { rot.pick(0, &three(187)) } else if s.flag(204) { rot.pick(1, &three(190)) } else { rot.pick(2, &three(193)) }),
        9 if s.flag(200) => talk(rot.pick(0, &three(202))),
        9 => Vec::new(),
        10 => talk(if s.flag(29) {
            rot.pick(0, &three(367))
        } else if s.flag(28) {
            rot.pick(1, &three(370))
        } else if s.flag(195) {
            rot.pick(2, &three(373))
        } else if s.flag(193) {
            rot.pick(3, &Lines::FirstOnce(376, vec![377, 378, 379]))
        } else {
            rot.pick(4, &three(380))
        }),
        _ => {
            // Template W (18 lines from b).
            let Some(&(_, b)) = CASTLE_W.get((r as usize).wrapping_sub(20)) else { return Vec::new() };
            talk(if s.flag(29) {
                rot.pick(0, &three(b))
            } else if s.flag(28) {
                Some(b + 3)
            } else if s.flag(27) {
                rot.pick(1, &three(b + 4))
            } else if s.flag(153) {
                if s.talk_roll == 1 {
                    Some([(26, 7), (25, 17), (24, 8), (23, 9)].iter().find(|(f, _)| s.flag(*f)).map_or(b + 10, |&(_, o)| b + o))
                } else {
                    rot.pick(2, &three(b + 11))
                }
            } else {
                rot.pick(3, &three(b + 14))
            })
        }
    }
}

/// The 1217 stage word (module word 0): bit 0 start, 1 intro done, 2 boss dead, 3 outro done,
/// 4 to 9 the stage scenes (requested, done), 10 the stage-1 spawn, 11 to 13 the stage-start
/// requests; bit 15 marks a controller scene running (the controller is one thread).
const BUSY: i32 = 1 << 15;

/// SPEC-0045 1217 Init (Act 1, flag 83 clear): flags 81 (and 83 with 248), the fight-due test;
/// on the fight-due branch script trigger 0, thread routine 1 (region 1218 into the slot),
/// thread routine 12 (entity 3 loops animation 94) and the streaming filter of callback 6
/// (only embedded ordinal 3, Lucy, binds; the others wait for their explicit activation).
/// Callback 5 (not due) refuses every ordinal. On the fight-due branch the object records are
/// filled (`crate::burrows`). The Act 3 branch is not built.
fn burrows_init(state: &mut GameState) -> Vec<Action> {
    state.set(81, true);
    if state.flag(248) {
        state.set(83, true);
    }
    let act3 = state.flag(83);
    let due = if act3 { !state.flag(278) } else { state.flag(145) && !state.flag(82) };
    let off = |e| Action::DeactivateEntity { entity: e };
    let mut out = Vec::new();
    if due && !act3 {
        // The records (the main records, the gates and the Act 1 cage) and thread routine 12's
        // request 2 on the cage.
        let (mut b, fill) = crate::burrows::Burrows::act1();
        b.cages[0].request = 2;
        state.burrows = Some(b);
        out.extend(fill);
        out.push(Action::RegisterTrigger { index: 0, routine: 1 });
        out.extend([0, 1, 2, 4, 5, 6].map(off));
        out.push(Action::LoadRegion { slot: 0, region: Some(1218) });
        out.push(Action::EntityAnim { entity: 3, anim: 94, looped: true, wait: false });
    } else {
        state.burrows = None;
        out.extend((0..7).map(off));
    }
    out
}

/// The stage spawns of 1217: spawn 1 entity 0 at locator 13 (bit 10); spawn 2 entities 0 and 1
/// at 14 and 15 (level flag 4); spawn 3 entities 0 to 2 at 16 to 18 (level flag 9). Each place
/// is ActivateEntity, MoveEntity with facing, SetAI_Agro.
fn burrows_spawn(n: u8) -> Vec<Action> {
    let place = |e, k| [Action::ActivateEntity { entity: e }, Action::MoveEntity { entity: e, locator: k, face: true }, Action::Agro { entity: e }];
    match n {
        1 => {
            let mut v = place(0, 13).to_vec();
            v.push(Action::Bits { word: 0, set: 1 << 10, clear: 0 });
            v
        }
        2 => {
            let mut v: Vec<Action> = [place(0, 14), place(1, 15)].concat();
            v.push(Action::LevelState { flag: 4, on: true });
            v
        }
        _ => {
            let mut v: Vec<Action> = [place(0, 16), place(1, 17), place(2, 18)].concat();
            v.push(Action::LevelState { flag: 9, on: true });
            v
        }
    }
}

/// SPEC-0045 1217 fight controller (Act 1), one step per pass while no controller scene runs:
/// the intro A, the win with outro A, the stage starts, the stage scenes and the fight monitor.
/// The skip intro (button 5 at the intro's start) is not read; the main records' scenes are
/// not played (they are object animations), so the death step of the records counts as done
/// when the entities are dead.
fn burrows_fight(state: &mut GameState, dialog: bool) -> Vec<Action> {
    use Action::*;
    let w = state.word(1217, 0);
    if w & BUSY != 0 || dialog || state.flag(83) {
        return Vec::new();
    }
    let bit = |b: i32| w & (1 << b) != 0;
    let busy = |state: &mut GameState| state.set_word(1217, 0, w | BUSY);
    let done = |b: i32| Bits { word: 0, set: 1 << b, clear: BUSY };
    let cut = |track| [FadeOut { seconds: 0.7, wait: true }, PlayCamera { track, wait: false, end: 1, speed: 1.0 }, FadeIn { seconds: 0.4, wait: false }];
    let cam_wait = |track| PlayCamera { track, wait: true, end: 1, speed: 1.0 };
    let anim = |anim, wait| EntityAnim { entity: 7, anim, looped: false, wait };
    // 1. The intro (A).
    if !bit(1) && bit(0) {
        busy(state);
        let req = |kind, index, request| BurrowsRequest { kind, index, request, inner: -1 };
        let mut v = vec![req(1, 0, 1), req(1, 1, 1), req(2, 0, 2)];
        v.extend([HoldHero, FadeOut { seconds: 0.5, wait: true }, WaitLoads, ActivateEntity { entity: 7 }, LetterBox { on: true, seconds: 1.0, wait: false }]);
        v.extend([MoveHero { locator: 88 }, Equip { on: false }, FadeIn { seconds: 0.5, wait: false }, camera(4), BurrowsStagger, WalkHero { locator: 89, wait: true, orient: true }]);
        v.extend([Sleep { ticks: 30 }, cam_wait(7)]);
        v.extend(cut(10));
        v.extend([wait(3.0), cam_wait(8), Sleep { ticks: 60 }]);
        v.extend(cut(6));
        v.extend([Message { line: 6 }, MoveHero { locator: 90 }, Equip { on: true }]);
        v.extend(cut(9));
        v.extend([wait(0.5), anim(5, true), EntityAnim { entity: 7, anim: 2, looped: true, wait: false }, wait(1.0), cam_wait(12)]);
        v.extend([HeroAnim { anim: 158, wait: false }, Sleep { ticks: 120 }]);
        v.extend(cut(11));
        v.extend([Sleep { ticks: 60 }, anim(6, true), anim(7, false), ReleaseEntity { entity: 7 }]);
        v.extend(burrows_spawn(1));
        v.extend([LetterBox { on: false, seconds: 1.0, wait: false }, release_camera(), done(1)]);
        return v;
    }
    if !bit(1) || bit(3) {
        return Vec::new();
    }
    // 2. The win: flags 82 and 147, then outro A.
    if bit(2) {
        busy(state);
        state.set(82, true);
        state.set(147, true);
        let mut v = vec![HoldHero, FadeOut { seconds: 1.0, wait: true }, LetterBox { on: true, seconds: 1.0, wait: false }, camera(14), MoveHero { locator: 93 }];
        v.extend([FadeIn { seconds: 0.5, wait: false }, wait(0.5), anim(0, true), LevelState { flag: 18, on: true }, Sleep { ticks: 20 }, camera(10)]);
        // The eleven main records closed one at a time; gate 1 and the cage are closed after
        // them (their place in the outro is not given; Medium).
        v.extend((0..11u8).flat_map(|index| [BurrowsClose { index }, Sleep { ticks: 15 }]));
        v.extend([BurrowsRequest { kind: 1, index: 1, request: 4, inner: -1 }, BurrowsRequest { kind: 2, index: 0, request: 4, inner: -1 }, Sleep { ticks: 1 }, WaitCage { index: 0 }]);
        v.extend([wait(2.0), DeactivateEntity { entity: 7 }, camera(15), WalkHero { locator: 94, wait: true, orient: false }, wait(2.0)]);
        v.extend(cut(17));
        v.extend([MoveHero { locator: 92 }, MoveEntity { entity: 3, locator: 91, face: true }, Converse { entity: 3, line: 10 }, camera(16)]);
        v.extend([Converse { entity: 3, line: 11 }, camera(17), Converse { entity: 3, line: 12 }, Converse { entity: 3, line: 13 }]);
        v.extend([WalkEntity { entity: 3, locator: 95, wait: false, orient: false, stop: 1.1 }, wait(2.0)]);
        v.extend(cut(14));
        v.extend([wait(2.0), FadeOut { seconds: 0.7, wait: true }, DeactivateEntity { entity: 3 }, release_camera()]);
        v.extend([LetterBox { on: false, seconds: 0.0, wait: false }, FadeIn { seconds: 0.5, wait: false }, done(3)]);
        return v;
    }
    // A stage start: SetLevelState 1, 5 or 10 and the stage's scene bits.
    for (req, flag, bits) in [(11, 1, (4, 5)), (12, 5, (6, 7)), (13, 10, (8, 9))] {
        if bit(req) {
            state.set_word(1217, 0, (w | (1 << bits.0) | (1 << bits.1)) & !(1 << req));
            return vec![LevelState { flag, on: true }];
        }
    }
    // A stage scene (Act 1): fade, camera, letterbox, the line, then the stage start. Which of
    // cameras 10 and 6 each scene uses is not given (R-0036): 10 for the first, 6 after.
    for (req, partner, line, flag, track) in [(4, 5, 7, 1, 10), (6, 7, 8, 5, 6), (8, 9, 9, 10, 6)] {
        if bit(req) && !bit(partner) {
            busy(state);
            let mut v = vec![HoldHero, FadeOut { seconds: 0.7, wait: true }, camera(track), LetterBox { on: true, seconds: 1.0, wait: false }, FadeIn { seconds: 0.4, wait: false }];
            v.extend([Message { line }, LetterBox { on: false, seconds: 1.0, wait: false }, release_camera(), LevelState { flag, on: true }]);
            v.push(Bits { word: 0, set: 1 << partner, clear: BUSY });
            return v;
        }
    }
    // 3. The fight monitor.
    let mut v = Vec::new();
    let set = |state: &mut GameState, b: i32| {
        let w = state.word(1217, 0);
        state.set_word(1217, 0, w | (1 << b));
    };
    if state.level(14) {
        set(state, 2);
    }
    for (f, spawn, clears, records) in [(15u16, 1u8, &[2u16][..], &[0usize][..]), (16, 2, &[6, 7], &[1, 2]), (17, 3, &[11, 12, 13], &[3, 4, 5])] {
        if state.level(f) {
            state.set_level(f, false);
            for &c in clears {
                state.set_level(c, false);
            }
            v.extend(burrows_spawn(spawn));
            if let Some(b) = state.burrows.as_mut() {
                for &r in records {
                    b.mains[r].inner.request = 1;
                }
            }
        }
    }
    // The records: close (level 4 record 0; level 9 records 1 and 2), slam (the stage-locator
    // flags 2, 6, 7, 11, 12 and 13, and 8 for records 1 and 2) and, for the stage that runs,
    // the hit (request 3 in state 2) and the death step (inner request 4 unless 4 or 6).
    let stage = if !state.level(1) { 1 } else if !state.level(5) { 2 } else if !state.level(10) { 3 } else { 0 };
    let records: &[(usize, usize)] = match stage {
        1 => &[(0, 0)],
        2 => &[(1, 0), (2, 1)],
        3 => &[(3, 0), (4, 1), (5, 2)],
        _ => &[],
    };
    let hits = state.hero.hits.clone();
    let dead: Vec<bool> = (0..3).map(|e| state.entity_dead(e)).collect();
    let levels: Vec<bool> = (0..=13).map(|f| state.level(f)).collect();
    if let Some(b) = state.burrows.as_mut() {
        for (flag, rs) in [(4usize, &[0usize][..]), (9, &[1, 2])] {
            if levels.get(flag).copied().unwrap_or(false) {
                for &r in rs {
                    if !matches!(b.mains[r].state, 4 | 5) {
                        b.mains[r].request = 4;
                    }
                }
            }
        }
        for (flag, r) in [(2usize, 0usize), (6, 1), (7, 2), (11, 3), (12, 4), (13, 5), (8, 1), (8, 2)] {
            if levels[flag] && !matches!(b.mains[r].inner.state, 5 | 7) {
                b.mains[r].inner.request = 7;
            }
        }
        if bit(1) {
            for &(r, e) in records {
                if hits.contains(&e) && b.mains[r].state == 2 {
                    b.mains[r].request = 3;
                }
                if dead[e] && !matches!(b.mains[r].inner.state, 4 | 6) {
                    b.mains[r].inner.request = 4;
                }
            }
        }
    }
    // A stage is complete when its records are all in inner state 6 (without records, when its
    // entities are dead).
    let complete = |state: &GameState, rs: &[(usize, usize)]| match state.burrows.as_ref() {
        Some(b) => rs.iter().all(|&(r, _)| b.mains[r].inner.state == 6),
        None => rs.iter().all(|&(_, e)| state.entity_dead(e)),
    };
    if stage == 1 {
        if bit(10) && complete(state, records) {
            set(state, if bit(4) { 11 } else { 4 });
        }
    } else if stage == 2 {
        if state.level(3) && !state.level(4) {
            state.set_level(4, true);
            v.extend(burrows_spawn(2));
        } else if state.level(4) && complete(state, records) {
            set(state, if bit(6) { 12 } else { 6 });
        }
    } else if stage == 3 {
        if state.level(8) && !state.level(9) {
            state.set_level(9, true);
            v.extend(burrows_spawn(3));
        } else if state.level(9) && complete(state, records) {
            set(state, if bit(8) { 13 } else { 8 });
        }
    }
    v
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
        // SPEC-0039 880 map shop answers: declined line 16; too few Neopoints line 15; else the
        // map given (item 175 with the map number), the price taken and line 14.
        // SPEC-0045 991 entity 17 (the map seller): offer k = C - 1 (word 1); no shows 100 + k;
        // yes with too few Neopoints 103; yes gives map 21, 17 or 35 for 300, 350 or 400 with
        // line 97 + k; C then goes up by 1.
        (991, 17, yes) => {
            let c = s.word(pack, 1).clamp(1, 3);
            let k = (c - 1) as usize;
            let (map, price) = [(21, 300), (17, 350), (35, 400)][k];
            s.set_word(pack, 1, c + 1);
            if !yes {
                vec![Action::Talk { line: 100 + k as u16 }]
            } else if s.neopoints < price {
                vec![Action::Talk { line: 103 }]
            } else {
                s.neopoints -= price;
                vec![Action::GiveItem { item: TREASURE_MAP, count: map }, Action::Talk { line: 97 + k as u16 }]
            }
        }
        // 991 entities 1, 2 and 3 (flag 667 has no setter, so the clear halves): the yes and no
        // lines; a yes sets the entity's module word (read by nothing).
        (991, 1..=3, yes) => {
            let (y, n) = match routine {
                1 => (141, 142),
                2 => (78, 79),
                _ => (165, 166),
            };
            if yes {
                s.set_word(pack, 10 + routine, 1);
            }
            vec![Action::Talk { line: if yes { y } else { n } }]
        }
        // SPEC-0039 1139 entity 2's sale: declined 16; below 500 Neopoints 17; else the item,
        // 500 taken, line 15.
        (1139, 3, false) => vec![Action::Talk { line: 16 }],
        (1139, 3, true) if s.neopoints < 500 => vec![Action::Talk { line: 17 }],
        (1139, 3, true) => {
            s.neopoints -= 500;
            let item = (s.word(pack, 0) - 1).max(0) as u16;
            vec![Action::GiveItem { item, count: 1 }, Action::Talk { line: 15 }]
        }
        // SPEC-0059 section 7 answers: yes takes task k (its flag; accept line 20 + k, whose
        // dialogue command gives one item 33); no shows 29, 30 or 31 by Random(1, 3).
        (1358, 1, true) => {
            let k = s.word(pack, 1).clamp(1, 8) as u16;
            s.set(604 + k, true);
            vec![Action::Talk { line: 20 + k }]
        }
        (1358, 1, false) => vec![Action::Talk { line: 28 + s.talk_roll.clamp(1, 3) as u16 }],
        (880, 7, false) | (1230, 4, false) => vec![Action::Talk { line: 16 }],
        (880, 7, true) | (1230, 4, true) => {
            let shop = map_shop(pack);
            let (map, _, price) = shop[(s.word(pack, 4).clamp(0, shop.len() as i32 - 1)) as usize];
            if s.neopoints < price {
                vec![Action::Talk { line: 15 }]
            } else {
                s.neopoints -= price;
                vec![Action::GiveItem { item: TREASURE_MAP, count: map }, Action::Talk { line: 14 }]
            }
        }
        // SPEC-0059 section 2: the wheel's start prompt (pay the fee, the light show, a new
        // pattern, the spin) and go-on prompt r (the go-on line and the next spin, or the
        // cash-out line, pay-out r and the cash-out flag).
        (1023 | 1358, 60, true) => {
            let c = crate::wheel::copy_of(pack);
            s.neopoints -= c.fee;
            let mut v: Vec<Action> = (0..14).flat_map(|k| [Action::WheelLights { even: k % 2 == 0 }, Action::Sleep { ticks: 5 }]).collect();
            v.push(Action::WheelPattern);
            v.extend(wheel_spin());
            v
        }
        (1023 | 1358, 60, false) => vec![release_camera()],
        (1023 | 1358, 61, true) => {
            let mut w = wheel_of(pack, s);
            w.r += 1;
            set_wheel(pack, s, &w);
            let mut v = vec![Action::Message { line: crate::wheel::copy_of(pack).go_on_line }];
            v.extend(wheel_spin());
            v
        }
        (1023 | 1358, 61, false) => {
            let c = crate::wheel::copy_of(pack);
            let mut w = wheel_of(pack, s);
            let r = (w.r as usize).min(3);
            s.neopoints += c.payouts[r];
            if let Some(f) = c.flags {
                s.set(f[1], true);
            }
            w.g = 0;
            w.r = 0;
            set_wheel(pack, s, &w);
            vec![Action::Message { line: c.cash_out[r] }, release_camera()]
        }
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
    // P0 to P3: the fame rule, then the story chain.
    let b = [9u16, 38, 67, 96][r as usize];
    if let Some(l) = fame(s, rot, b) {
        return l;
    }
    let three = |o: u16| cycle(&[b + o, b + o + 1, b + o + 2]);
    cascade(s, rot, &[(202, three(26)), (193, three(23)), (150, three(20)), (142, three(17)), (138, three(14))]).flatten().or_else(|| rot.pick(9, &three(11)))
}

/// The fame rule (SPEC-0059 section 3; SPEC-0045 1023 arena-talk test): with A the count of
/// arena flags 381 to 385 set and d the talk's Random(1, 6), d < A gives the arena line by the
/// highest arena flag (381 b, 382 b + 1, 383 b + 2, 384 b + 3 and b + 4 in turn, 385 b + 5 to
/// b + 7 in turn); else flag 480 with the talk's Random(1, 3) at 1 gives the wheel cycle b + 8
/// to b + 10. None when neither applies.
fn fame(s: &GameState, rot: &mut Rotation, b: u16) -> Option<Option<u16>> {
    let a = (381..=385).filter(|&f| s.flag(f)).count() as i32;
    if s.talk_d < a {
        return Some(match (381u16..=385).rev().find(|&f| s.flag(f)) {
            Some(385) => rot.pick(21, &cycle(&[b + 5, b + 6, b + 7])),
            Some(384) => rot.pick(20, &cycle(&[b + 3, b + 4])),
            Some(f) => Some(b + (f - 381)),
            None => None,
        });
    }
    (s.flag(480) && s.talk_roll == 1).then(|| rot.pick(22, &cycle(&[b + 8, b + 9, b + 10])))
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
        // SPEC-0037 783, 786, 787, 788 and 789, 855 to 858, 1429, 1152, 1251 and 784: P on
        // entity 0.
        (783 | 786 | 787 | 788 | 789 | 855..=858 | 1429 | 1152 | 1251 | 784, 0) => Some(0),
        // SPEC-0043 1358: the delivery giver on entity 20 and line 90 on entity 21 (entities 16 to
        // 19 are Medium in the spec and not routed).
        (1358, 20) => Some(1),
        (1358, 21) => Some(2),
        // SPEC-0044 745: routines A to D on entities 3, 0, 1 and 2; SPEC-0043 1173: routine 4 on 0.
        (745, 0..=3) => Some(entity as u8),
        (1173, 0) => Some(4),
        // SPEC-0039 1405 to 1425: the entity-0 routine.
        (1405 | 1407 | 1417 | 1419 | 1425, 0) => Some(0),
        // SPEC-0039 1139: handlers 1 to 5 on entities 0 to 4; 732: handler 3 on entity 1; 793:
        // handler 1 on entity 0, handler 2 on 1 and 2; 874: handlers 1 and 2 on entities 6 and 7.
        (1139, 0..=4) => Some(entity as u8 + 1),
        (732, 1) => Some(3),
        (793, 0) => Some(1),
        (793, 1 | 2) => Some(2),
        (874, 6) => Some(1),
        (874, 7) => Some(2),
        // SPEC-0041 742: P7 on entity 7; 1414: callbacks 1 to 4 on entities 6, 9, 8 and 7.
        (742, 7) => Some(7),
        (1414, 6..=9) => Some(entity as u8),
        // SPEC-0041 1409: callbacks 1 and 2 on entities 0 and 1; 1421 and 1423: CB2 on entity 0.
        (1409, 0 | 1) => Some(entity as u8 + 1),
        (1421 | 1423, 0) => Some(2),
        // SPEC-0043 750: routines 1 to 4 on entities 3, 1, 0 and 2.
        (750, 3) => Some(1),
        (750, 1) => Some(2),
        (750, 0) => Some(3),
        (750, 2) => Some(4),
        // SPEC-0039 984: handler 1 on entity 0.
        (984, 0) => Some(1),
        // SPEC-0044 1017: set D on entities 0 and 4, C on 1 and 5, A on 2 and 6, B on 3 and 7.
        (1017, 0 | 4) => Some(0),
        (1017, 1 | 5) => Some(1),
        (1017, 2 | 6) => Some(2),
        (1017, 3 | 7) => Some(3),
        // SPEC-0045 991: one routine per entity, with 9/12/40, 10/13, 15/42 and 16/41 shared.
        (991, 9 | 12 | 40) => Some(9),
        (991, 10 | 13) => Some(10),
        (991, 15 | 42) => Some(15),
        (991, 16 | 41) => Some(16),
        (991, 1..=8 | 11 | 14 | 17..=20) => Some(entity as u8),
        // SPEC-0039 880: the map shop (handler 7) on entities 3 and 7; 1230: handler 4 on 46
        // and 84.
        (880, 3 | 7) => Some(7),
        (1230, 46 | 84) => Some(4),
        // SPEC-0042 1150: P0 to P4 on entities 0 to 4.
        (1150, 0..=4) => Some(entity as u8),
        // SPEC-0039 1133: entity 1 with flag 10 set, else entity 0.
        (1133, 0 | 1) => Some(0),
        (1023, 90) => Some(90),
        (1098, e) => castle_routine(e),
        // SPEC-0045 1163: the Miamice keeper (entities 0 and 10), entity 7 and Sir Harlag (15).
        (1163, 0 | 10) => Some(0),
        (1163, 7) => Some(7),
        (1163, 15) => Some(15),
        // SPEC-0044 1056: the trainer of the current stage (entity 5, 10, 12 or 24).
        (1056, 5 | 10 | 12 | 24) => Some(0),
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
    // SPEC-0039 880 handler 7 and 1230 handler 4, the map shop: the first map not held is
    // offered with its prompt (its index kept in word 4 for the answer); with every map held,
    // lines 11, 12 and 13 in rotation (word 5).
    if matches!((pack, routine), (880, 7) | (1230, 4)) {
        let shop = map_shop(pack);
        if let Some(k) = shop.iter().position(|m| !state.maps.contains(&m.0)) {
            state.set_word(pack, 4, k as i32);
            return vec![Action::Prompt { line: shop[k].1, routine }];
        }
        let r = state.word(pack, 5).rem_euclid(3);
        state.set_word(pack, 5, r + 1);
        return vec![Action::Talk { line: 11 + r as u16 }];
    }
    if pack == 991 {
        return lakeside_talk(routine, state);
    }
    if pack == 1017 {
        return guild_talk(routine, state);
    }
    if pack == 750 {
        return talk_750(routine, state);
    }
    if pack == 745 {
        return apothecary_talk(routine, state);
    }
    if (pack, routine) == (1358, 2) {
        return vec![Action::Talk { line: 90 }];
    }
    if (pack, routine) == (1358, 1) {
        return delivery_talk(state);
    }
    if (pack, routine) == (1173, 4) {
        return if state.flag(30) { vec![Action::Talk { line: 1 }] } else { vec![Action::Message { line: 4 }] };
    }
    if matches!(pack, 1405 | 1407 | 1417 | 1419 | 1425) && routine == 0 {
        return hall_talk(pack, state);
    }
    if matches!(pack, 1139 | 732 | 793 | 874) {
        return town_talk(pack, routine, state);
    }
    if matches!(pack, 742 | 1409 | 1414 | 1421 | 1423) {
        return palace_talk(pack, routine, state);
    }
    if eastside_interior(pack).is_some() && routine == 0 {
        return eastside_talk(pack, state);
    }
    if matches!(pack, 855..=858 | 1429 | 1152 | 1251 | 784) && routine == 0 {
        return outskirts_talk(pack, state);
    }
    // SPEC-0039 984 entity 0 (handler 1): with 67 line 12 in the conversation frame; else line
    // 15 (202), 18 (150) or 21 alone.
    if (pack, routine) == (984, 1) {
        return if state.flag(67) {
            vec![Action::Talk { line: 12 }]
        } else {
            vec![Action::Message { line: if state.flag(202) { 15 } else if state.flag(150) { 18 } else { 21 } }]
        };
    }
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
        (1163, r) => {
            let a = kitchen_talk(r, state, &mut rot);
            state.rotations.insert(key, rot);
            return a;
        }
        (1056, 0) => {
            let a = trainer_talk(state, &mut rot);
            state.rotations.insert(key, rot);
            return a;
        }
        (1098, r) => {
            let a = castle_talk(r, state, &mut rot);
            state.rotations.insert(key, rot);
            return a;
        }
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
    fn module_1358_tasks_and_wheel() {
        // SPEC-0059 vector: 604 and 100 set, tasks 1 to 4 open, n = 4; r = 2 offers task 3
        // (prompt 15), yes sets 607 with line 23.
        let mut s = GameState { talk_raw: 1, ..Default::default() };
        assert_eq!(on_talk(1358, 1, &mut s), vec![Action::Talk { line: 32 }]);
        s.set(100, true);
        assert_eq!(on_talk(1358, 1, &mut s), vec![Action::Prompt { line: 15, routine: 1 }]);
        assert_eq!(on_answer(1358, 1, true, &mut s), vec![Action::Talk { line: 23 }]);
        assert!(s.flag(607));
        // Task 3 taken: its reminder line 7.
        assert_eq!(on_talk(1358, 1, &mut s), vec![Action::Talk { line: 7 }]);
        // Only task 2 left (n = 1): r = 1 reads past the list, no offer.
        let mut t = GameState::default();
        t.set(604, true);
        t.set(679, true);
        assert!(on_talk(1358, 1, &mut t).is_empty());
        // The 1358 wheel copy: fee 100, too-poor line 109.
        let mut w = GameState { neopoints: 99, ..Default::default() };
        let call = TriggerCall { index: 0, state: 2, object: true, square: true };
        assert_eq!(on_trigger(1358, 56, call, &mut w).actions, vec![Action::Message { line: 109 }]);
        // Arrival 60 goes to locator 77.
        w.blackboard.insert((2, 159), 60);
        assert_eq!(on_activate(1358, &mut w)[0], Action::MoveHero { locator: 77 });
    }

    #[test]
    fn modules_745_1173() {
        let mut s = GameState::default();
        // 745 entity 3: 260 and line 23; with 75 and items 5, 4 and 3 the exchange (line 16).
        assert_eq!(on_talk(745, 3, &mut s), vec![Action::Talk { line: 23 }]);
        s.set(75, true);
        assert_eq!(on_talk(745, 3, &mut s), vec![Action::Talk { line: 20 }]);
        for i in [3, 4, 5] {
            s.give(i, 1);
        }
        assert_eq!(on_talk(745, 3, &mut s), vec![Action::Talk { line: 16 }]);
        assert!(s.flag(76) && s.flag(264) && !s.has_item(4));
        // 76 then 567 (line 15); entity 0 cycles 24, 25, 26.
        assert_eq!(on_talk(745, 3, &mut s), vec![Action::Talk { line: 15 }]);
        assert_eq!(on_talk(745, 0, &mut s), vec![Action::Talk { line: 24 }]);
        // While 264 is set the entity-control routine does not run.
        entity_setup(745, &mut s);
        assert!(entity_step(745, &mut s, &|_| false, &|_| false, &mut |lo, _| lo).is_empty());
        // 1173 arrival 130: locator 1 and flag 319.
        s.blackboard.insert((2, 126), 130);
        assert_eq!(on_activate(1173, &mut s)[0], Action::MoveHero { locator: 1 });
        assert!(s.flag(319));
    }

    #[test]
    fn halls_1405_to_1425() {
        let mut s = GameState::default();
        // 1407 sets 100 at Init, so its record and slot load always run.
        assert_eq!(on_init(1407, &mut s)[0], Action::LoadRegion { slot: 0, region: Some(1408) });
        assert_eq!(entity_setup(1407, &mut s).len(), 1);
        assert_eq!(on_talk(1407, 0, &mut s), vec![Action::Talk { line: 11 }]);
        s.set(607, true);
        s.give(33, 2);
        assert_eq!(on_talk(1407, 0, &mut s), vec![Action::Talk { line: 14 }]);
        assert!(!s.has_item(33) && s.flag(681));
        // 1425 without 106: nothing registered, no load, no record.
        assert!(on_init(1425, &mut s).is_empty() && entity_setup(1425, &mut s).is_empty());
        assert_eq!(s.blackboard.get(&(2, 160)), Some(&50));
        assert!(on_enter(1419, &mut s) == vec![Action::Message { line: 2 }]);
    }

    #[test]
    fn modules_1139_732_793_874() {
        let mut s = GameState { neopoints: 600, talk_five: 3, ..Default::default() };
        // 1139 entity 2: a one-time offer (draw 3: item 54, prompt 12), then its rotation.
        assert_eq!(on_talk(1139, 3, &mut s), vec![Action::Prompt { line: 12, routine: 3 }]);
        assert_eq!(on_answer(1139, 3, true, &mut s), vec![Action::GiveItem { item: 54, count: 1 }, Action::Talk { line: 15 }]);
        assert_eq!((s.neopoints, on_talk(1139, 3, &mut s)), (100, vec![Action::Talk { line: 18 }]));
        // Entity 0: 678 then line 36; with item 23 the way to pack 1129.
        assert_eq!(on_talk(1139, 1, &mut s), vec![Action::Talk { line: 36 }]);
        s.give(23, 1);
        assert_eq!(on_talk(1139, 1, &mut s), vec![Action::ReplaceExterior { slot: 109, plane: -1 }]);
        // Entity 1 with 678 set: 46, 47, 48 in turn; with 305: 49.
        assert_eq!(on_talk(1139, 2, &mut s), vec![Action::Talk { line: 46 }]);
        s.set(305, true);
        assert_eq!(on_talk(1139, 2, &mut s), vec![Action::Talk { line: 49 }]);
        // 732: arrival 30 -> locator 3 and track 6.
        s.blackboard.insert((2, 21), 30);
        let a = on_activate(732, &mut s);
        assert_eq!((a[0], a[2]), (Action::MoveHero { locator: 3 }, Action::PlayCamera { track: 6, wait: false, end: 1, speed: 1.0 }));
        // 793 entities 1 and 2: 410 takes item 13 and sets 412 with line 12, then line 9.
        s.set(410, true);
        s.give(13, 1);
        assert_eq!(on_talk(793, 2, &mut s), vec![Action::Talk { line: 12 }]);
        assert_eq!(on_talk(793, 2, &mut s), vec![Action::Talk { line: 9 }]);
        assert!(on_talk(874, 1, &mut s).contains(&Action::SetFlag { flag: 291, on: true }));
    }

    #[test]
    fn modules_742_1414() {
        let mut s = GameState::default();
        assert_eq!(on_talk(742, 7, &mut s), vec![Action::Message { line: 4 }]);
        s.ai_dead.insert(6);
        on_update(742, &mut s, false);
        assert!(s.flag(562));
        assert_eq!(on_talk(742, 7, &mut s), vec![Action::Talk { line: 0 }]);
        // 1414 vectors: teleport 0 posts 50 for eW 159; arrivals 50 and 10.
        assert_eq!(on_teleport(1414, 0, &mut s), vec![Action::ReplaceExterior { slot: 159, plane: -1 }]);
        assert_eq!(s.blackboard.get(&(2, 159)), Some(&50));
        s.blackboard.insert((2, 180), 10);
        assert_eq!(on_activate(1414, &mut s), vec![Action::MoveHero { locator: 12 }]);
        assert!(on_activate(1414, &mut s).is_empty());
        assert!(on_talk(1414, 9, &mut s).is_empty());
        s.set(102, true);
        assert_eq!(on_talk(1414, 9, &mut s), vec![Action::Talk { line: 6 }]);
    }

    #[test]
    fn palace_rooms() {
        // SPEC-0041 1409 vectors: 102 and 104 set: lines 0 and 10; 104 clear: 3 and 13; 102
        // clear: nothing.
        let mut s = GameState::default();
        s.set(106, true);
        assert!(on_talk(1409, 1, &mut s).is_empty());
        s.set(102, true);
        s.set(104, true);
        assert_eq!((on_talk(1409, 1, &mut s), on_talk(1409, 2, &mut s)), (vec![Action::Talk { line: 0 }], vec![Action::Talk { line: 10 }]));
        s.set(104, false);
        assert_eq!(on_talk(1409, 2, &mut s), vec![Action::Talk { line: 13 }]);
        assert!(on_init(1409, &mut s).contains(&Action::LoadRegion { slot: 0, region: Some(1410) }));
        // 1421: the item-33 hand-in once, then draw 1 gives line 10.
        s.set(605, true);
        s.give(33, 1);
        assert_eq!(on_talk(1421, 2, &mut s), vec![Action::Talk { line: 13 }]);
        s.talk_roll = 1;
        assert_eq!(on_talk(1421, 2, &mut s), vec![Action::Talk { line: 10 }]);
        // 1423 on-enter with 104 and 102: line 0.
        s.set(104, true);
        assert_eq!(on_enter(1423, &mut s), vec![Action::Message { line: 0 }]);
    }

    #[test]
    fn module_750_cycles() {
        let mut s = GameState::default();
        // Entity 3 (routine 1): lines 6, 7, 8 in turn; with 76 the shared counter goes on (5).
        let a: Vec<_> = (0..2).map(|_| on_talk(750, 1, &mut s)).collect();
        assert_eq!(a.concat(), vec![Action::Talk { line: 6 }, Action::Talk { line: 7 }]);
        s.set(76, true);
        assert_eq!(on_talk(750, 1, &mut s), vec![Action::Talk { line: 5 }]);
        // The first top case (420) restarts it at line 0.
        s.set(420, true);
        assert_eq!(on_talk(750, 1, &mut s), vec![Action::Talk { line: 0 }]);
        assert_eq!(on_talk(750, 1, &mut s), vec![Action::Talk { line: 1 }]);
        // Routine 4: with 419 set and 420 clear, 420 is set with line 38.
        let mut t = GameState::default();
        t.set(419, true);
        assert_eq!(on_talk(750, 4, &mut t), vec![Action::Talk { line: 38 }]);
        assert_eq!(on_talk(750, 4, &mut t), vec![Action::Talk { line: 35 }]);
        assert_eq!(proximity_routine(750, 2), Some(4));
    }

    #[test]
    fn module_796_lever_and_exit() {
        let mut s = GameState::default();
        let call = TriggerCall { index: 1, state: 2, object: true, square: true };
        let run = on_trigger(796, 1, call, &mut s).actions;
        assert!(run.contains(&Action::DeactivateGroup { group: 19 }) && run.contains(&Action::ReactivateGroup { group: 21 }));
        assert!(on_trigger(796, 1, call, &mut s).actions.is_empty());
        let exit = on_trigger(796, 0, TriggerCall { index: 0, state: 1, ..call }, &mut s).actions;
        assert_eq!(exit[1], Action::ReplaceExterior { slot: 22, plane: -1 });
        assert_eq!(s.blackboard.get(&(2, 22)), Some(&80));
    }

    #[test]
    fn module_1369_walk_and_exit() {
        let mut s = GameState::default();
        on_init(1369, &mut s);
        assert_eq!(s.blackboard.get(&(2, 157)), Some(&10));
        let call = TriggerCall { index: 0, state: 2, object: true, square: false };
        assert_eq!(on_trigger(1369, 0, call, &mut s).prompt, Some(4));
        let run = on_trigger(1369, 0, TriggerCall { square: true, ..call }, &mut s).actions;
        assert!(run.contains(&Action::PlayCamera { track: 4, wait: false, end: 1, speed: 1.0 }));
        assert_eq!(on_trigger(1369, 1, TriggerCall { index: 1, ..call }, &mut s).actions, vec![Action::ReplaceExterior { slot: 157, plane: -1 }]);
    }

    #[test]
    fn modules_1238_947_1299_809() {
        // SPEC-0037 vector: 1238 with 567 set and 568 clear, four of item 82 set 568.
        let mut s = GameState::default();
        s.set(50, true);
        s.set(567, true);
        assert_eq!(on_init(1238, &mut s), vec![Action::LoadRegion { slot: 0, region: Some(1242) }, Action::LoadRegion { slot: 1, region: Some(1240) }]);
        s.give(82, 4);
        on_update(1238, &mut s, false);
        assert!(s.flag(568));
        on_pickup(1238, 83, &mut s);
        assert!(s.flag(579));
        // 947: line 0 once per pass of trigger 2 then 0; AI 3 dead sets 375 with lines 1, 2.
        let call = |index| TriggerCall { index, state: 1, object: true, square: false };
        assert!(on_trigger(947, 0, call(0), &mut s).actions.is_empty());
        on_trigger(947, 2, call(2), &mut s);
        assert_eq!(on_trigger(947, 0, call(0), &mut s).actions, vec![Action::Message { line: 0 }]);
        s.ai_dead.insert(3);
        assert_eq!(on_update(947, &mut s, false).len(), 2);
        // 1299: index 1 with 248 clear sets 243 and goes to eW 154.
        assert_eq!(on_teleport(1299, 1, &mut s), vec![Action::ReplaceExterior { slot: 154, plane: -1 }]);
        assert!(s.flag(243));
        // 809: flag 56 (with 54) loads set 2.
        s.set(54, true);
        s.set(56, true);
        assert_eq!(on_init(809, &mut s), vec![Action::LoadRegion { slot: 0, region: Some(812) }]);
    }

    #[test]
    fn modules_1225_1259_1253_799() {
        let mut s = GameState::default();
        s.set(82, true);
        s.set(81, true);
        assert_eq!(on_init(1225, &mut s), vec![Action::LoadRegion { slot: 0, region: Some(1227) }]);
        // SPEC-0037 vector: 1259 trigger 2 with the body resets the slot and loads set 2.
        let call = TriggerCall { index: 2, state: 1, object: true, square: false };
        assert_eq!(on_trigger(1259, 0, call, &mut s).actions, vec![Action::LoadRegion { slot: 0, region: None }, Action::LoadRegion { slot: 0, region: Some(1262) }]);
        // 1253 vectors: value 30 -> locator 2, track 7; no message -> locator 0, track 5.
        s.blackboard.insert((2, 143), 30);
        let a = on_activate(1253, &mut s);
        assert_eq!((a[0], a[2]), (Action::MoveHero { locator: 2 }, Action::PlayCamera { track: 7, wait: true, end: 1, speed: 1.0 }));
        assert_eq!(on_activate(1253, &mut s)[0], Action::MoveHero { locator: 0 });
        // 799 vector: trigger 3 entered three times with 416 set and no item 12: no line, line
        // 1, no line.
        s.set(416, true);
        let enter = TriggerCall { index: 3, state: 1, object: true, square: false };
        let lines: Vec<usize> = (0..3).map(|_| on_trigger(799, 3, enter, &mut s).actions.len()).collect();
        assert_eq!(lines, vec![0, 1, 0]);
        let lever = TriggerCall { index: 0, state: 2, object: true, square: true };
        assert!(on_trigger(799, 0, lever, &mut s).actions.contains(&Action::SetFlag { flag: 663, on: true }));
    }

    #[test]
    fn modules_1152_1251_784() {
        // SPEC-0037 vector: 1152 with only 23 and 633 set: 162 set and line 14, then line 11.
        let mut s = GameState::default();
        s.set(23, true);
        s.set(633, true);
        assert_eq!(on_talk(1152, 0, &mut s), vec![Action::Talk { line: 14 }]);
        assert_eq!(on_talk(1152, 0, &mut s), vec![Action::Talk { line: 11 }]);
        s.blackboard.insert((2, 119), 110);
        assert_eq!(on_activate(1152, &mut s)[0], Action::MoveHero { locator: 1 });
        assert_eq!(on_activate(1152, &mut s)[0], Action::MoveHero { locator: 0 });
        // 1251 vectors: 585 clear, 460 set, 273 and 270 clear: 31, 32, 31; a new visit with
        // 460 clear, 457 set and no item 49: 35, 36, 37, 35.
        let mut s = GameState::default();
        s.set(460, true);
        let a: Vec<_> = (0..3).map(|_| on_talk(1251, 0, &mut s)).collect();
        assert_eq!(a.concat(), [31, 32, 31].map(|line| Action::Talk { line }).to_vec());
        let mut s = GameState::default();
        s.set(457, true);
        let b: Vec<_> = (0..4).map(|_| on_talk(1251, 0, &mut s)).collect();
        assert_eq!(b.concat(), [35, 36, 37, 35].map(|line| Action::Talk { line }).to_vec());
        // 784 vector: trigger 1 with 543 clear, 542 set and the button: the walk and line 4.
        let mut s = GameState::default();
        s.set(542, true);
        let call = TriggerCall { index: 1, state: 2, object: true, square: true };
        let out = on_trigger(784, 1, call, &mut s);
        assert!(out.actions.contains(&Action::WalkHero { locator: 2, wait: true, orient: true }) && out.actions.contains(&Action::Message { line: 4 }));
        assert_eq!(on_trigger(784, 1, TriggerCall { square: false, ..call }, &mut s).prompt, Some(4));
    }

    #[test]
    fn outskirts_interiors() {
        let mut s = GameState::default();
        // SPEC-0037 vector: 858 with 56 and 512 set, 514 clear, no item 73, 59 clear: four talks
        // give 11, 12, 13, 11; then 59 set gives 8.
        s.set(56, true);
        s.set(512, true);
        let lines: Vec<_> = (0..4).map(|_| on_talk(858, 0, &mut s)).collect();
        assert_eq!(lines.concat(), [11, 12, 13, 11].map(|line| Action::Talk { line }).to_vec());
        s.set(59, true);
        assert_eq!(on_talk(858, 0, &mut s), vec![Action::Talk { line: 8 }]);
        // 857 thread A with 59 clear and 56 set: line 2.
        s.set(59, false);
        assert_eq!(on_init(857, &mut s)[0], Action::Message { line: 2 });
        // 855: 59 then 462, then the hammer (item 22) for the armour (122).
        s.set(59, true);
        assert_eq!(on_talk(855, 0, &mut s), vec![Action::Talk { line: 17 }]);
        s.give(22, 1);
        assert_eq!(on_talk(855, 0, &mut s), vec![Action::Talk { line: 13 }]);
        assert!(s.has_item(122) && !s.has_item(22) && s.flag(463));
        // 1429: one counter across the sets.
        assert_eq!(on_talk(1429, 0, &mut s), vec![Action::Talk { line: 17 }]);
        s.set(236, true);
        assert_eq!(on_talk(1429, 0, &mut s), vec![Action::Talk { line: 15 }]);
        assert_eq!(on_buy(1429, 109, &s), vec![Action::Message { line: 20 }]);
    }

    #[test]
    fn eastside_interiors() {
        let mut s = GameState { neopoints: 600, ..Default::default() };
        // 787: lines 5 to 10 in turn; with 447, 500 Neopoints for line 3.
        let lines: Vec<_> = (0..7).map(|_| on_talk(787, 0, &mut s)).collect();
        assert_eq!((lines[0].clone(), lines[6].clone()), (vec![Action::Talk { line: 5 }], vec![Action::Talk { line: 5 }]));
        s.set(447, true);
        assert_eq!(on_talk(787, 0, &mut s), vec![Action::Talk { line: 3 }]);
        assert_eq!((s.neopoints, on_talk(787, 0, &mut s)), (100, vec![Action::Talk { line: 4 }]));
        // 783's ladder: 16 (sets 419), 13, then with 420: 12 (sets 421), then 9.
        assert_eq!(on_talk(783, 0, &mut s), vec![Action::Talk { line: 16 }]);
        assert_eq!(on_talk(783, 0, &mut s), vec![Action::Talk { line: 13 }]);
        s.set(420, true);
        assert_eq!(on_talk(783, 0, &mut s), vec![Action::Talk { line: 12 }]);
        assert_eq!(on_talk(783, 0, &mut s), vec![Action::Talk { line: 9 }]);
        // 786 teleport 0 posts 20 for eW 31 and leaves to eW 44; the arrival uses locator 1.
        assert_eq!(on_teleport(786, 0, &mut s)[0], Action::LeaveInterior { slot: 44 });
        assert_eq!(on_activate(786, &mut s)[0], Action::MoveHero { locator: 1 });
        assert_eq!(teleport_context(789, 0), Some(13));
    }

    #[test]
    fn module_984_lines() {
        let mut s = GameState::default();
        assert_eq!(on_enter(984, &mut s), vec![Action::Message { line: 6 }]);
        s.set(150, true);
        assert_eq!(on_talk(984, 1, &mut s), vec![Action::Message { line: 18 }]);
        s.set(67, true);
        assert_eq!(on_enter(984, &mut s), vec![Action::Converse { entity: 0, line: 0 }]);
        assert_eq!(on_talk(984, 1, &mut s), vec![Action::Talk { line: 12 }]);
    }

    #[test]
    fn module_1017_guildmaster() {
        let mut s = GameState::default();
        assert_eq!(on_init(1017, &mut s), vec![Action::LoadRegion { slot: 0, region: Some(1018) }]);
        // 297 is set at Init, so Marl's first lines are 99 to 101; then 298 gives 98 once.
        assert_eq!(on_talk(1017, 0, &mut s), vec![Action::Talk { line: 99 }]);
        assert_eq!(on_talk(1017, 0, &mut s), vec![Action::Talk { line: 100 }]);
        s.set(298, true);
        assert_eq!(on_talk(1017, 0, &mut s), vec![Action::Talk { line: 98 }]);
        assert!(s.flag(299));
        assert_eq!(on_talk(1017, 0, &mut s), vec![Action::Talk { line: 95 }]);
        // Set C sets 585 with 299 set; Marl then moves to 92.
        on_talk(1017, 1, &mut s);
        assert!(s.flag(585));
        assert_eq!(on_talk(1017, 0, &mut s), vec![Action::Talk { line: 92 }]);
        assert_eq!(on_teleport(1017, 1, &mut s)[0], Action::ReplaceExterior { slot: 81, plane: -1 });
        assert_eq!(on_activate(1017, &mut s)[0], Action::MoveHero { locator: 2 });
    }

    #[test]
    fn module_991_crowds_seller_and_teleports() {
        let mut s = GameState { neopoints: 1000, ..Default::default() };
        for f in [86, 299, 414] {
            s.set(f, true);
        }
        // SPEC-0045 vector: set 1, trigger 5 loads 998 into the crowd slot, guarded by word 30.
        let enter = TriggerCall { index: 5, state: 1, object: true, square: false };
        let a = on_trigger(991, 8, enter, &mut s).actions;
        assert_eq!((a[0], a[3], a[4]), (Action::WaitWord { word: 30 }, Action::LoadRegion { slot: 1, region: Some(998) }, Action::WaitInside { index: 5 }));
        assert!(on_trigger(991, 8, TriggerCall { index: 4, ..enter }, &mut s).actions.contains(&Action::LoadRegion { slot: 1, region: Some(997) }));
        // Set 2 has no region for zone 2 (the group trigger).
        s.set(86, false);
        assert!(on_trigger(991, 8, TriggerCall { index: 10, ..enter }, &mut s).actions.is_empty());
        // The map seller: line 89, then prompt 90 (map 21 for 300); held map 17 skips prompt 91.
        assert_eq!(on_talk(991, 17, &mut s), vec![Action::Talk { line: 89 }]);
        assert_eq!(on_talk(991, 17, &mut s), vec![Action::Prompt { line: 90, routine: 17 }]);
        assert_eq!(on_answer(991, 17, true, &mut s), vec![Action::GiveItem { item: TREASURE_MAP, count: 21 }, Action::Talk { line: 97 }]);
        s.maps.extend([21, 17]);
        assert_eq!(on_talk(991, 17, &mut s), vec![Action::Prompt { line: 92, routine: 17 }]);
        // Teleport 4: values 50 for eW 81 and 88, the replacement to eW 88; back: locator 7.
        assert_eq!(on_teleport(991, 4, &mut s)[0], Action::ReplaceExterior { slot: 88, plane: -1 });
        assert_eq!(s.blackboard.get(&(2, 88)), Some(&50));
        assert_eq!(on_activate(991, &mut s)[0], Action::MoveHero { locator: 7 });
        // Entity 14: line 10 first, then the item-12 hand-in with line 7.
        assert_eq!(on_talk(991, 14, &mut s), vec![Action::Talk { line: 10 }]);
        s.give(12, 1);
        assert_eq!(on_talk(991, 14, &mut s), vec![Action::Talk { line: 7 }]);
        assert!(s.flag(417) && !s.has_item(12));
    }

    #[test]
    fn module_886_hints_and_end() {
        let mut s = GameState::default();
        s.set(50, true);
        assert!(on_init(886, &mut s).contains(&Action::LoadRegion { slot: 0, region: Some(888) }));
        // The first hint after 300 ticks, then the hero-1 hint with 16 and 17 dead and 18 alive.
        let mut lines = Vec::new();
        for _ in 0..301 {
            lines.extend(on_update(886, &mut s, false));
        }
        assert_eq!(lines, vec![Action::Message { line: 1 }, Action::Sleep { ticks: 1 }]);
        s.hero.selected = 1;
        s.ai_dead.extend([16, 17]);
        assert_eq!(on_update(886, &mut s, false)[1], Action::Message { line: 3 });
        assert!(on_update(886, &mut s, false).is_empty());
        s.ai_dead.insert(18);
        assert_eq!(on_update(886, &mut s, false)[0], Action::DeactivateObj { obj: 3 });
        assert!(s.flag(250));
    }

    #[test]
    fn module_1230_shop_slots_and_lines() {
        let mut s = GameState { neopoints: 1000, ..Default::default() };
        s.maps.extend([1, 8, 30, 27]);
        assert_eq!(on_talk(1230, 4, &mut s), vec![Action::Prompt { line: 10, routine: 4 }]);
        assert_eq!(on_answer(1230, 4, true, &mut s)[0], Action::GiveItem { item: TREASURE_MAP, count: 18 });
        assert_eq!(s.neopoints, 500);
        // Thread 1: flag 18 gives set 2; flag 248 gives set 6; flag 20 alone loads nothing.
        s.set(18, true);
        assert!(on_init(1230, &mut s).contains(&Action::LoadRegion { slot: 0, region: Some(1233) }));
        s.set(18, false);
        s.set(20, true);
        assert!(!on_init(1230, &mut s).iter().any(|a| matches!(a, Action::LoadRegion { .. })));
        s.set(248, true);
        assert!(on_init(1230, &mut s).contains(&Action::LoadRegion { slot: 0, region: Some(1237) }));
        let call = TriggerCall { index: 2, state: 2, object: true, square: true };
        assert_eq!(on_trigger(1230, 2, call, &mut s).actions, vec![Action::Message { line: 19 }]);
    }

    #[test]
    fn module_880_map_shop_and_gate() {
        let mut s = GameState { neopoints: 600, ..Default::default() };
        assert_eq!(on_talk(880, 7, &mut s), vec![Action::Prompt { line: 6, routine: 7 }]);
        assert_eq!(on_answer(880, 7, true, &mut s), vec![Action::GiveItem { item: TREASURE_MAP, count: 1 }, Action::Talk { line: 14 }]);
        assert_eq!(s.neopoints, 350);
        s.maps.insert(1);
        assert_eq!(on_talk(880, 7, &mut s), vec![Action::Prompt { line: 7, routine: 7 }]);
        assert_eq!(on_answer(880, 7, false, &mut s), vec![Action::Talk { line: 16 }]);
        s.maps.extend([8, 30, 27, 18, 3, 6]);
        s.neopoints = 399;
        assert_eq!(on_talk(880, 7, &mut s), vec![Action::Prompt { line: 19, routine: 7 }]);
        assert_eq!(on_answer(880, 7, true, &mut s), vec![Action::Talk { line: 15 }]);
        s.maps.insert(11);
        let rot: Vec<_> = (0..4).map(|_| on_talk(880, 7, &mut s)).collect();
        assert_eq!(rot[0], vec![Action::Talk { line: 11 }]);
        assert_eq!(rot[3], vec![Action::Talk { line: 11 }]);
        // The way on to 991 opens once AI 21 to 23 are dead.
        assert_eq!(on_teleport(880, 0, &mut s)[0], Action::Message { line: 20 });
        s.ai_dead.extend([21, 22]);
        on_update(880, &mut s, false);
        assert!(!s.flag(296));
        s.ai_dead.insert(23);
        on_update(880, &mut s, false);
        assert!(s.flag(296));
        assert_eq!(on_teleport(880, 0, &mut s)[0], Action::ReplaceExterior { slot: 81, plane: -1 });
        assert_eq!(on_activate(880, &mut s), vec![Action::MoveHero { locator: 2 }, Action::ReturnCamera { preset: -1, release: true }]);
    }

    #[test]
    fn module_1171_levers_counts_and_floats() {
        let mut s = GameState::default();
        let init = on_init(1171, &mut s);
        assert!(init.contains(&Action::RegisterGroupType { kind: 12 }));
        // T0 with flag 638 set: flag 163 (count 1) and line 1; at count 5 nothing is said.
        s.set(638, true);
        let call = TriggerCall { index: 0, state: 1, object: true, square: false };
        assert_eq!(on_trigger(1171, 0, call, &mut s).actions, vec![Action::Message { line: 1 }]);
        assert!(s.flag(163) && !s.flag(33));
        for f in 637..=641 {
            s.set(f, true);
        }
        assert!(on_trigger(1171, 0, call, &mut s).actions.is_empty());
        // A lever prompts while inside, runs on Square, then never again this load.
        let lever = TriggerCall { index: 5, state: 2, object: true, square: false };
        let out = on_trigger(1171, 5, lever, &mut s);
        assert_eq!((out.prompt, out.actions.len()), (Some(6), 0));
        let run = on_trigger(1171, 5, TriggerCall { square: true, ..lever }, &mut s).actions;
        assert!(run.contains(&Action::PlayCamera { track: 4, wait: false, end: 1, speed: 1.0 }) && run.contains(&Action::PlayAnim { obj: 2, speed: 1.0, wait: false }));
        assert_eq!(on_trigger(1171, 5, TriggerCall { square: true, ..lever }, &mut s).prompt, None);
        // Entry 4's floating object (20, 21, 22): R on the entering fire only.
        let enter = TriggerCall { index: 30, state: 1, object: true, square: false };
        let r = group_run(1171, 12, enter, &[20, 21, 22]);
        assert_eq!((r[0], r[3], r[10]), (Action::DeactivateObj { obj: 20 }, Action::WaitInside { index: 30 }, Action::LoopAnim { obj: 20, speed: 1.0 }));
        assert!(group_run(1171, 12, TriggerCall { state: 2, ..enter }, &[20, 21, 22]).is_empty());
        assert_eq!(group_init(1171, 12, &[17, 18, 19])[1], Action::SetObjTimeRandom { obj: 17 });
    }

    #[test]
    fn treasure_map_give_marks_the_map() {
        // SPEC-0021 Treasure maps: the give's quantity is the treasure number; no item is kept.
        let mut s = GameState::default();
        s.give(TREASURE_MAP, 11);
        s.give(TREASURE_MAP, 11);
        assert_eq!(s.maps.iter().copied().collect::<Vec<_>>(), vec![11]);
        assert!(!s.items.contains_key(&TREASURE_MAP));
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
        assert_eq!(init, vec![Action::LockChest { id: 0, key: 15, message: 124 }, Action::InitObjOn { obj: 0, end: false }, Action::LoadRegion { slot: 0, region: Some(953) }]);
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
        // The fight start activates objects 0 and 1.
        assert_eq!(on_update(691, &mut s, false), vec![Action::InitObjOn { obj: 0, end: false }, Action::InitObjOn { obj: 1, end: false }]);
        hp(&mut s, [55, 55, 50]);
        assert!(on_update(691, &mut s, false).is_empty());
        assert!(!s.flag(125));
        hp(&mut s, [54, 55, 50]);
        let end = on_update(691, &mut s, false);
        assert!(end.contains(&Action::LoadRegion { slot: 0, region: Some(695) }));
        assert_eq!(end[..2], [Action::DeactivateObj { obj: 0 }, Action::DeactivateObj { obj: 1 }]);
        assert!(s.flag(125) && s.flag(126) && s.items[&170] == 100);
        // The intro does not run again; the next visit loads set 3 (thread A).
        assert!(on_trigger(691, 1, TriggerCall { index: 1, state: 1, object: true, square: false }, &mut s).actions.is_empty());
        s.reload_module(691);
        assert_eq!(on_init(691, &mut s).iter().filter(|a| matches!(a, Action::LoadRegion { .. })).collect::<Vec<_>>(), vec![&Action::LoadRegion { slot: 0, region: Some(695) }]);
    }

    /// SPEC-0059 section 3: the wheel host greets twice per module life, the arena master
    /// alternates 35 and 36, and a latch holds until the partner trigger.
    /// SPEC-0059 section 3: entity 69 (b 84) gives arena line 85 with 381 and 382 set and d 1,
    /// the 384 pair with 384 highest, the wheel cycle with 480 and e 1, else its story chain.
    #[test]
    fn fame_lines() {
        let mut s = GameState::default();
        let mut rot = Rotation::default();
        let k = villager(69).unwrap() as u8;
        s.set(381, true);
        s.set(382, true);
        s.talk_d = 1;
        assert_eq!(village_talk(k, &mut s, &mut rot), Some(85));
        s.talk_d = 2;
        s.set(7, true);
        assert_eq!(village_talk(k, &mut s, &mut rot), Some(95), "story chain 7: b + 11");
        s.set(480, true);
        s.talk_roll = 1;
        assert_eq!(village_talk(k, &mut s, &mut rot), Some(92));
        assert_eq!(village_talk(k, &mut s, &mut rot), Some(93));
        s.set(383, true);
        s.set(384, true);
        s.talk_d = 1;
        assert_eq!(village_talk(k, &mut s, &mut rot), Some(87));
        assert_eq!(village_talk(k, &mut s, &mut rot), Some(88));
    }

    #[test]
    fn village_greetings() {
        let mut s = GameState::default();
        let call = |index| TriggerCall { index, state: 1, object: true, square: false };
        let say = |s: &mut GameState, index| on_trigger(1023, if index % 2 == 1 { 54 } else { 55 }, call(index), s).actions;
        assert_eq!(say(&mut s, 39), vec![Action::Converse { entity: 65, line: 1020 }]);
        assert!(say(&mut s, 39).is_empty(), "latched");
        say(&mut s, 40);
        assert_eq!(say(&mut s, 39), vec![Action::Converse { entity: 65, line: 1021 }]);
        say(&mut s, 40);
        assert!(say(&mut s, 39).is_empty(), "two per module life");
        assert_eq!(say(&mut s, 37), vec![Action::Converse { entity: 66, line: 35 }]);
        say(&mut s, 38);
        assert_eq!(say(&mut s, 37), vec![Action::Converse { entity: 66, line: 36 }]);
        say(&mut s, 38);
        s.set(383, true);
        assert_eq!(say(&mut s, 37), vec![Action::Converse { entity: 66, line: 30 }]);
    }

    #[test]
    fn castle_effects_by_flags() {
        let mut s = GameState::default();
        let start = |slot| Action::StartEffect { slot, camera: false };
        assert_eq!(on_update(1098, &mut s, false), vec![start(0), start(1), start(2)]);
        s.set(642, true);
        let a = on_update(1098, &mut s, false);
        assert!(a.contains(&Action::StopEffect { slot: 0 }) && a.contains(&start(3)));
        s.set(632, true);
        assert!(on_update(1098, &mut s, false).contains(&Action::StopEffect { slot: 3 }));
    }

    #[test]
    fn hero_swap_scene_1244() {
        let mut s = GameState::default();
        let init = on_init(1244, &mut s);
        assert!(s.flag(271));
        assert_eq!(init.len(), 9, "four objects off, five registrations");
        let call = TriggerCall { index: 0, state: 1, object: true, square: false };
        // Not in swap mode 2: no scene.
        s.hero.swap_mode = 1;
        assert!(on_trigger(1244, 1, call, &mut s).actions.is_empty());
        s.hero.swap_mode = 2;
        let a = on_trigger(1244, 1, call, &mut s).actions;
        assert!(a.contains(&Action::ActivateHero { hero: 1, locator: 16, block: false, fade: false, cue: false }));
        assert_eq!(a.last(), Some(&Action::SwapMode { mode: 1 }));
        // The break scene ends on Tor in mode 0 and sets 272, which closes triggers 0, 3 and 4.
        let b = on_trigger(1244, 5, TriggerCall { index: 3, ..call }, &mut s).actions;
        let mode = b.iter().position(|x| *x == Action::SwapMode { mode: 0 }).unwrap();
        assert_eq!(b[mode + 1], Action::ActivateHero { hero: 0, locator: 12, block: false, fade: false, cue: false });
        assert!(b.contains(&Action::SetFlag { flag: 272, on: true }));
        s.set(272, true);
        assert!(on_trigger(1244, 1, call, &mut s).actions.is_empty());
        assert!(on_trigger(1244, 4, TriggerCall { index: 4, ..call }, &mut s).actions.is_empty());
        // Trigger 4 runs once per level-state 0.
        s.set(272, false);
        assert!(!on_trigger(1244, 4, TriggerCall { index: 4, ..call }, &mut s).actions.is_empty());
        assert!(on_trigger(1244, 4, TriggerCall { index: 4, ..call }, &mut s).actions.is_empty());
    }

    #[test]
    fn miamice_game() {
        // SPEC-0045 1163: the keeper with 632 starts the game (line 8, flag 156, level state 0,
        // the catch on); deliveries 1 to 4 show lines 43 to 46 once each; level state 5 wins
        // (158 and 23, the keeper's line 4). A return with 156 set and 158 clear restarts.
        let mut s = GameState::default();
        s.set(632, true);
        let r = proximity_routine(1163, 10).unwrap();
        let start = on_talk(1163, r, &mut s);
        assert!(start.contains(&Action::Talk { line: 8 }) && start.contains(&Action::MiamouseInit { field: 0, cage: 1 }) && s.flag(156));
        for n in 1..=4u16 {
            s.level_flags = [n].into();
            assert_eq!(on_update(1163, &mut s, false), vec![Action::Message { line: 42 + n }]);
            assert!(on_update(1163, &mut s, false).is_empty(), "once each");
        }
        s.level_flags = [5].into();
        assert_eq!(on_update(1163, &mut s, false), vec![Action::Converse { entity: 0, line: 4 }]);
        assert!(s.flag(158) && s.flag(23));
        let mut s = GameState::default();
        s.set(156, true);
        assert!(on_activate(1163, &mut s).contains(&Action::MiamouseInit { field: 0, cage: 1 }));
        s.set(158, true);
        assert!(on_activate(1163, &mut s).contains(&Action::MiamouseDisable));
        assert!(on_init(1163, &mut s).contains(&Action::LoadRegion { slot: 0, region: Some(1164) }));
    }

    #[test]
    fn courtyard_lessons() {
        // SPEC-0044 1056. Chain lesson: Init with 22 picks trainer entity 5 and loads 1057 and
        // 1064; in talk zone 10 the auto-talk shows lines 58 to 60 (flag 154).
        let mut s = GameState::default();
        s.set(22, true);
        let init = on_init(1056, &mut s);
        assert!(init.contains(&Action::LoadRegion { slot: 0, region: Some(1057) }) && init.contains(&Action::LoadRegion { slot: 1, region: Some(1064) }));
        let zone = |s: &mut GameState, i: u16| on_trigger(1056, 5, TriggerCall { index: i, state: 1, object: true, square: false }, s);
        zone(&mut s, 10);
        assert_eq!(on_update(1056, &mut s, false), vec![Action::Converse { entity: 5, line: 58 }]);
        assert!(s.flag(154));
        // In the chain area, a finisher (attack state 4) that misses entity 4 gives remark 5
        // (lines 42, 41, 40), spoken once no attack is active.
        zone(&mut s, 5);
        s.hero.sub = 4;
        on_update(1056, &mut s, false);
        zone(&mut s, 5);
        s.hero.sub = 0;
        assert_eq!(on_update(1056, &mut s, false), vec![Action::Converse { entity: 5, line: 42 }]);
        // A finisher hitting entity 4 sets 155; the talk waits for zone 10.
        s.hero.sub = 4;
        s.hero.hits = vec![4];
        assert!(on_update(1056, &mut s, false).is_empty() && s.flag(155));
        s.hero = HeroView::default();
        zone(&mut s, 10);
        assert_eq!(on_update(1056, &mut s, false), vec![Action::Converse { entity: 5, line: 36 }]);
        assert!(s.flag(632));
        // Spin lesson: one spin (state 7) marking entities 7, 8 and 9 sets 161.
        let mut s = GameState::default();
        for f in [22, 23, 160] {
            s.set(f, true);
        }
        on_init(1056, &mut s);
        for e in [7, 8, 9] {
            s.hero = HeroView { sub: 7, hits: vec![e], ..HeroView::default() };
            on_update(1056, &mut s, false);
        }
        assert!(s.flag(161));
        // Mote lesson: with 169 set, holding item 160 sets 170; a fire Mote in word 2 sets 171.
        let mut s = GameState::default();
        for f in [22, 24, 169] {
            s.set(f, true);
        }
        on_init(1056, &mut s);
        s.give(160, 100);
        on_update(1056, &mut s, false);
        assert!(s.flag(170));
        s.hero.attack_mote = Some(160);
        on_update(1056, &mut s, false);
        assert!(s.flag(171));
        assert_eq!(undamageable(1056), &[4, 7, 8, 9, 20, 21, 14]);
    }

    #[test]
    fn castle_story_steps() {
        // SPEC-0045 1098: Init sets 21 and 151; Master Torak (entity 35) sets 152 with line 14,
        // then rotates 11 to 13; the rest prompt's press with 152 sets 22, 642 and 153 and
        // shows line 50; the return locator for value 90 is 1.
        let mut s = GameState::default();
        on_init(1098, &mut s);
        assert!(s.flag(21) && s.flag(151));
        let r = proximity_routine(1098, 35).unwrap();
        assert_eq!(proximity_routine(1098, 36), Some(r));
        assert_eq!(on_talk(1098, r, &mut s), vec![Action::Talk { line: 14 }]);
        assert!(s.flag(152));
        let lines: Vec<_> = (0..4).map(|_| on_talk(1098, r, &mut s)).collect();
        assert_eq!(lines, [11, 12, 13, 11].map(|line| vec![Action::Talk { line }]));
        let call = TriggerCall { index: 0, state: 1, object: true, square: true };
        let out = on_trigger(1098, 1, call, &mut s);
        assert!(out.prompt == Some(32) && out.actions.contains(&Action::Message { line: 50 }));
        assert!(s.flag(22) && s.flag(642) && s.flag(153));
        s.blackboard.insert((2, 106), 90);
        assert_eq!(on_activate(1098, &mut s)[0], Action::MoveHero { locator: 1 });
        // Template W with 153 and a roll of 1: the first set of 26, 25, 24, 23 (none: b + 10).
        let w = proximity_routine(1098, 44).unwrap();
        s.talk_roll = 1;
        assert_eq!(on_talk(1098, w, &mut s), vec![Action::Talk { line: 107 }]);
        s.set(24, true);
        assert_eq!(on_talk(1098, w, &mut s), vec![Action::Talk { line: 105 }]);
    }

    #[test]
    fn burrows_fight_controller() {
        // SPEC-0045 1217, Act 1: Init sets 81, registers trigger 0 and loads region 1218 when
        // 145 is set and 82 clear; not due, every embedded ordinal is refused.
        let mut s = GameState::default();
        let init = on_init(1217, &mut s);
        assert!(s.flag(81) && !init.iter().any(|a| matches!(a, Action::RegisterTrigger { .. })));
        s.set(145, true);
        let init = on_init(1217, &mut s);
        assert!(init.contains(&Action::RegisterTrigger { index: 0, routine: 1 }) && init.contains(&Action::LoadRegion { slot: 0, region: Some(1218) }));
        // Trigger 0 sets the start bit; the next update runs intro A once (line 6, spawn 1).
        let call = TriggerCall { index: 0, state: 1, object: true, square: false };
        on_trigger(1217, 1, call, &mut s);
        let intro = on_update(1217, &mut s, false);
        assert!(intro.contains(&Action::Message { line: 6 }) && intro.contains(&Action::MoveEntity { entity: 0, locator: 13, face: true }));
        assert!(on_update(1217, &mut s, false).is_empty(), "one controller scene at a time");
        // The intro's last calls: bit 10 (spawn 1) and bit 1, clearing the busy mark.
        for a in &intro {
            if let Action::Bits { word, set, clear } = *a {
                let w = s.word(1217, word);
                s.set_word(1217, word, (w | set) & !clear);
            }
        }
        // Entity 0 dead before stage 1: the death step (record 0's inner request 4, stepped by
        // the host's record thread to inner state 6), the scene request (bit 4), then its scene
        // (line 7 and level state 1).
        s.entity_hp.insert(0, 0);
        assert!(on_update(1217, &mut s, false).is_empty());
        let mut b = s.burrows.take().unwrap();
        for t in [0, 1, 40] {
            b.step(&|_| false, t);
        }
        assert_eq!(b.mains[0].inner.state, 6);
        s.burrows = Some(b);
        assert!(on_update(1217, &mut s, false).is_empty());
        let scene = on_update(1217, &mut s, false);
        assert!(scene.contains(&Action::Message { line: 7 }) && scene.contains(&Action::LevelState { flag: 1, on: true }));
        for a in &scene {
            match *a {
                Action::Bits { word, set, clear } => {
                    let w = s.word(1217, word);
                    s.set_word(1217, word, (w | set) & !clear);
                }
                Action::LevelState { flag, on } => s.set_level(flag, on),
                _ => {}
            }
        }
        // Stage 1: the boss's level flag 3 runs spawn 2 once (level flag 4).
        s.set_level(3, true);
        let spawn = on_update(1217, &mut s, false);
        assert!(spawn.contains(&Action::MoveEntity { entity: 1, locator: 15, face: true }) && s.level(4));
        // The boss's death (level flag 14): flags 82 and 147 and outro A.
        s.set_level(14, true);
        on_update(1217, &mut s, false);
        let outro = on_update(1217, &mut s, false);
        assert!(s.flag(82) && s.flag(147) && !s.flag(5));
        assert!(outro.contains(&Action::Converse { entity: 3, line: 13 }) && outro.contains(&Action::LevelState { flag: 18, on: true }));
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
