//! Level behaviour written from the per-level specs (SPEC-0030 section 7: each level as data
//! plus rules in the host-API vocabulary, never by running module code). This holds the game
//! state the handlers read and write (story flags, the blackboard, held items), the actions they
//! return for the host to carry out, and the handlers described so far:
//! - 909 (SPEC-0045): teleport callbacks 0 and 1 (interior entries to slots 72 and 73) and the
//!   activate-world routine's flag reset and blackboard placement.
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
}

/// Story flags (QueryGameState and SetGameState), the blackboard (AddBlackboardMessage, read by
/// the activate-world routines) and the hero's items.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct GameState {
    pub flags: BTreeSet<u16>,
    /// (message type, eW slot) to value; a read resets it to -1 (absent).
    pub blackboard: HashMap<(u8, usize), i32>,
    pub items: HashMap<u16, u32>,
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
            let locator = match state.take_message(2, 68) {
                10 => Some(2),
                30 => Some(1),
                40 => Some(3),
                50 if state.stage() == Some(6) && !state.has_item(36) => Some(28),
                50 => Some(4),
                _ => None,
            };
            locator.map(|locator| vec![Action::MoveHero { locator }]).unwrap_or_default()
        }
        _ => Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn farm_doors_and_returns() {
        let mut s = GameState::default();
        assert_eq!(on_teleport(909, 1, &mut s), vec![Action::EnterInterior { slot: 73 }]);
        // Leaving 955 posts 40 for eW 68; 909's activation then moves the hero to locator 3 and
        // resets the message.
        assert_eq!(on_teleport(955, 0, &mut s), vec![Action::LeaveInterior { slot: 5 }]);
        assert_eq!(on_activate(909, &mut s), vec![Action::MoveHero { locator: 3 }]);
        assert_eq!(on_activate(909, &mut s), vec![]);
        // 951 with state 119 set and 122 clear shows line 126 and posts nothing.
        s.set(119, true);
        assert_eq!(on_teleport(951, 0, &mut s), vec![Action::Message { line: 126 }]);
        s.set(122, true);
        assert_eq!(on_teleport(951, 0, &mut s), vec![Action::LeaveInterior { slot: 5 }]);
        assert_eq!(on_activate(909, &mut s), vec![Action::MoveHero { locator: 2 }]);
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
        assert_eq!(on_activate(909, &mut s), vec![Action::MoveHero { locator: 28 }]);
        s.items.insert(36, 1);
        s.blackboard.insert((2, 68), 50);
        assert_eq!(on_activate(909, &mut s), vec![Action::MoveHero { locator: 4 }]);
    }
}
