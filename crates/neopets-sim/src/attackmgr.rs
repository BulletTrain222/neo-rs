//! The common enemies' attack manager (SPEC-0038 Behavior 45): one shared list of up to 20
//! enemies in agro, re-ordered each frame by distance to the selected hero, with a weight budget
//! from the hero's gear that decides which of them must wait in line.

/// The list capacity.
pub const MAX: usize = 20;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Entry {
    pub id: u64,
    pub dist: f32,
    pub wait: bool,
}

/// What the frame's passes read about one listed enemy (None: it no longer exists).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Member {
    /// In class state 19 and not dead.
    pub agro: bool,
    pub dist: f32,
    /// Eligible to move in the order pass: not in the sequencer's states 0 to 3, not blocking,
    /// not in the hit reaction.
    pub movable: bool,
    /// The row's Difficulty value (u16 0x3A).
    pub weight: i32,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Manager {
    pub entries: Vec<Entry>,
}

/// C = 3.0 x (w + a): w the weapon's place in its family (Tor's swords 98 to 105, Roberta's
/// wands 111 to 118), a the body item's (Tor's armour 119 to 125, Roberta's dresses 127 to 134;
/// item 126 gives 0 for either); an empty slot gives -1.
pub fn capacity(hero: usize, weapon: Option<u16>, body: Option<u16>) -> f32 {
    let (wb, bb) = if hero == 0 { (98, 119) } else { (111, 127) };
    let w = weapon.map_or(-1, |id| id as i32 - wb);
    let a = body.map_or(-1, |id| if id == 126 { 0 } else { id as i32 - bb });
    3.0 * (w + a) as f32
}

impl Manager {
    /// The grant on entering agro: refused (false) for an id already listed or a 21st enemy.
    pub fn grant(&mut self, id: u64) -> bool {
        if self.entries.len() >= MAX || self.entries.iter().any(|e| e.id == id) {
            return false;
        }
        self.entries.push(Entry { id, dist: -1.0, wait: false });
        true
    }

    /// An enemy waits when its entry's flag is set; one not on the list never waits.
    pub fn waits(&self, id: u64) -> bool {
        self.entries.iter().any(|e| e.id == id && e.wait)
    }

    /// Removal copies the last entry into the freed place; the pass then goes on with the next
    /// place, so the moved entry is skipped for that frame.
    fn remove(&mut self, k: usize) {
        self.entries.swap_remove(k);
    }

    /// One frame's three passes: release (gone, not in agro or dead; a kept entry stores its
    /// distance), the order pass (swap i and j when the enemy at i may move and distance j is
    /// below distance i + 5.0) and the budget (entry k waits when C < used and k > 0, then its
    /// weight is added; gone entries are removed).
    pub fn run(&mut self, capacity: f32, member: &dyn Fn(u64) -> Option<Member>) {
        let mut k = 0;
        while k < self.entries.len() {
            match member(self.entries[k].id) {
                Some(m) if m.agro => self.entries[k].dist = m.dist,
                _ => self.remove(k),
            }
            k += 1;
        }
        let n = self.entries.len();
        for i in 0..n.saturating_sub(1) {
            for j in i..n {
                let movable = member(self.entries[i].id).is_some_and(|m| m.movable);
                if movable && self.entries[j].dist < self.entries[i].dist + 5.0 {
                    self.entries.swap(i, j);
                }
            }
        }
        let mut used = 0.0;
        let mut k = 0;
        while k < self.entries.len() {
            match member(self.entries[k].id) {
                Some(m) => {
                    self.entries[k].wait = capacity < used && k > 0;
                    used += m.weight as f32;
                }
                None => self.remove(k),
            }
            k += 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn capacity_from_gear() {
        assert_eq!(capacity(0, None, None), -6.0);
        assert_eq!(capacity(0, Some(98), Some(119)), 0.0);
        assert_eq!(capacity(0, Some(105), Some(125)), 39.0);
        assert_eq!(capacity(1, Some(118), Some(134)), 42.0);
        assert_eq!(capacity(1, Some(111), Some(126)), 0.0);
    }

    #[test]
    fn budget_and_release() {
        let mut m = Manager::default();
        for id in 1..=21 {
            assert_eq!(m.grant(id), id <= 20);
        }
        assert!(!m.grant(1));
        let mut m = Manager::default();
        for id in 1..=5 {
            m.grant(id);
        }
        // Weight 8 each, far apart (no reordering), all busy: C 15 lets two attack.
        let info = |id: u64| Some(Member { agro: true, dist: 10.0 * id as f32, movable: false, weight: 8 });
        m.run(15.0, &info);
        let waits: Vec<bool> = (1..=5).map(|id| m.waits(id)).collect();
        assert_eq!(waits, [false, false, true, true, true]);
        // C negative: exactly one.
        m.run(-6.0, &info);
        assert_eq!((1..=5).filter(|&id| !m.waits(id)).count(), 1);
        // Enemy 1 leaves agro: released on the next frame, the last entry moved into its place.
        let info2 = |id: u64| Some(Member { agro: id != 1, dist: 10.0 * id as f32, movable: false, weight: 8 });
        m.run(15.0, &info2);
        assert_eq!(m.entries.iter().map(|e| e.id).collect::<Vec<_>>(), [5, 2, 3, 4]);
        assert!(!m.waits(1));
    }

    #[test]
    fn order_pass_brings_near_free_enemies_forward() {
        let mut m = Manager::default();
        for id in 1..=3 {
            m.grant(id);
        }
        // Distances 30, 20, 10, all free: the nearest comes first.
        let info = |id: u64| Some(Member { agro: true, dist: 40.0 - 10.0 * id as f32, movable: true, weight: 1 });
        m.run(100.0, &info);
        assert_eq!(m.entries[0].id, 3);
    }
}
