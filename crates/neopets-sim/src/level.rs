//! Level behaviour written from the per-level specs (SPEC-0030 section 7: each level as data
//! plus rules in the host-API vocabulary, never by running module code). This holds the game
//! state the handlers read and write (story flags, the blackboard, held items), the actions they
//! return for the host to carry out, and the handlers described so far:
//! - 909 (SPEC-0045): teleport callbacks 0 and 1 (interior entries to slots 72 and 73), the
//!   activate-world routine's flag reset and blackboard placement, and its activation thread's
//!   slot loads by story stage (the farm family and the fourth slot).
//! - 951 (SPEC-0041): teleport callback 0 (a line, or leave to slot 5 with message 10).
//! - 955 (SPEC-0032): teleport callback 0 (message 40, leave to slot 5).
//! Not covered: every other level, and in these levels the triggers, proximity talk, threads,
//! slot loads, objects, cutscenes, Update and the camera calls.

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
    /// SlotLoad into one of the level's slots: the region resource to load, or None to empty
    /// the slot (its characters leave).
    LoadRegion { slot: u8, region: Option<u32> },
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
        _ => None,
    };
    state.rotations.insert(key, rot);
    line.map(|line| vec![Action::Talk { line }]).unwrap_or_default()
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
        assert_eq!(on_activate(909, &mut s), vec![load(0, Some(910)), load(1, Some(914)), load(2, Some(920)), load(3, Some(926))]);
        // SPEC-0045 test vector: flags 1 and 122 set, no entry value 10: 910 or 911 by flag
        // 110, P 916, Q 922, R 930.
        s.set(1, true);
        s.set(122, true);
        // A finished round-up (107 to 110 all set) is kept, so flag 110 picks 911.
        for f in 107..=110 {
            s.set(f, true);
        }
        assert_eq!(on_activate(909, &mut s), vec![load(0, Some(911)), load(1, Some(916)), load(2, Some(922)), load(3, Some(930))]);
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
