//! The hero's candidate list and lock (SPEC-0040 Behavior 20, items 1 to 5): up to 15 records
//! (character id, distance, in-range flag, alignment, engaged flag), the add, the refresh with
//! its skip after a removal, the engagement (alignment above 0.25 while the hero attacks; in
//! range while closer than 20.0), the in-range records moved to the front and their pairwise
//! order (the 1.0 swap quirk), and the lock (the head record while it is in range).
//!
//! Not covered: the forced and held targets, the world lock-out counter, the run suppression of
//! the target flag (item 6) and the attack turn (item 7). The candidate test is the host's.

use crate::hero::Vec3;

pub const RECORDS: usize = 15;
/// The in-range distance (data word 0x2DD6C0).
pub const RANGE: f32 = 20.0;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Record {
    pub id: u64,
    pub d: f32,
    pub in_range: bool,
    pub a: f32,
    pub engaged: bool,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Candidates {
    pub records: Vec<Record>,
}

fn dist(a: Vec3, b: Vec3) -> f32 {
    ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)).sqrt()
}

impl Candidates {
    /// One tick. `chars` are the characters passing the candidate test with their positions, in
    /// actor-list order; `forward` is the hero's forward axis; `attacking` the engagement test.
    /// Returns the lock.
    pub fn tick(&mut self, chars: &[(u64, Vec3)], hero: Vec3, forward: Vec3, attacking: bool) -> Option<u64> {
        // 1. Adding.
        for &(id, p) in chars {
            if self.records.len() < RECORDS && !self.records.iter().any(|r| r.id == id) {
                self.records.push(Record { id, d: dist(p, hero), ..Record::default() });
            }
        }
        // 2. Refresh: a removal moves the later records up and the walk skips the one that
        // moved into the freed place.
        let mut i = 0;
        while i < self.records.len() {
            match chars.iter().find(|c| c.0 == self.records[i].id) {
                None => {
                    self.records.remove(i);
                }
                Some(&(_, p)) => {
                    let d = dist(p, hero);
                    let u = [p[0] - hero[0], p[1] - hero[1], p[2] - hero[2]];
                    let l = d.max(1e-6);
                    let r = &mut self.records[i];
                    r.d = d;
                    r.a = (forward[0] * u[0] + forward[1] * u[1] + forward[2] * u[2]) / l;
                    // 3. Engagement.
                    if !r.engaged && r.a > 0.25 && attacking {
                        r.engaged = true;
                    }
                    if r.engaged {
                        if r.d < RANGE {
                            r.in_range = true;
                        } else {
                            r.engaged = false;
                            r.in_range = false;
                        }
                    }
                }
            }
            i += 1;
        }
        // 4. Order: the in-range records to the front, then the pairwise pass.
        let mut front = 0;
        for k in 0..self.records.len() {
            if self.records[k].in_range {
                self.records.swap(front, k);
                front += 1;
            }
        }
        for i in 0..front {
            for j in i + 1..front {
                if self.records[j].d < self.records[i].d + 1.0 {
                    self.records.swap(i, j);
                }
            }
        }
        // 5. The lock: the head record while one is in range and the head is closer than 20.
        self.records.first().filter(|r| front > 0 && r.d < RANGE).map(|r| r.id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn engagement_range_and_order() {
        let mut c = Candidates::default();
        let fwd = [0.0, 0.0, 1.0];
        let chars = [(1, [0.0, 0.0, 10.0]), (2, [0.0, 0.0, 15.0]), (3, [0.0, 0.0, -5.0])];
        // Not engaged until the hero attacks while facing the candidate.
        assert_eq!(c.tick(&chars, [0.0; 3], fwd, false), None);
        assert_eq!(c.tick(&chars, [0.0; 3], fwd, true), Some(1));
        // The one behind (alignment -1) is never engaged; at 20.0 a record loses both flags.
        assert!(!c.records.iter().find(|r| r.id == 3).unwrap().engaged);
        let far = [(1, [0.0, 0.0, 20.0]), (2, [0.0, 0.0, 15.0]), (3, [0.0, 0.0, -5.0])];
        assert_eq!(c.tick(&far, [0.0; 3], fwd, false), Some(2));
        // Two in range within 1.0 of each other swap every tick (the quirk).
        let close = [(1, [0.0, 0.0, 10.0]), (2, [0.0, 0.0, 10.5])];
        let mut c = Candidates::default();
        let a = c.tick(&close, [0.0; 3], fwd, true);
        let b = c.tick(&close, [0.0; 3], fwd, true);
        assert_ne!(a, b);
        // A removed candidate leaves the list; 15 records at most.
        let many: Vec<(u64, Vec3)> = (0..20).map(|k| (k, [0.0, 0.0, 30.0])).collect();
        let mut c = Candidates::default();
        c.tick(&many, [0.0; 3], fwd, false);
        assert_eq!(c.records.len(), RECORDS);
        c.tick(&many[1..], [0.0; 3], fwd, false);
        assert!(!c.records.iter().any(|r| r.id == 0));
    }
}
