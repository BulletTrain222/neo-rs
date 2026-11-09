//! The hero's candidate list and lock (SPEC-0040 Behavior 20, items 1 to 5): up to 15 records
//! (character id, distance, in-range flag, alignment, engaged flag), the add, the refresh with
//! its skip after a removal, the engagement (alignment above 0.25 while the hero attacks; in
//! range while closer than 20.0), the in-range records moved to the front and their pairwise
//! order (the 1.0 swap quirk), and the lock (the head record while it is in range).
//!
//! Also the hero's target list (SPEC-0053 3.9, at +0xA70) and the attack turn toward its head
//! (SPEC-0040 Behavior 20 item 7, `aim`).
//!
//! Not covered: the forced and held targets of the lock, the world lock-out counter and the run
//! suppression of the target flag (item 6). The candidate and scan tests are the host's.

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
    /// actor-list order; `exists` says whether an id is still found (world list, petpet
    /// children, second world); `forward` is the hero's forward axis; `attacking` the engagement
    /// test. Returns the lock.
    ///
    /// SPEC-0040 Behavior 20 item 2 (corrected 2026-11-09): a record whose character is found and
    /// fails the candidate test is removed; one whose id is not found is left exactly as it is
    /// (not refreshed, its flags stale) and does not count as in range this tick. The refresh
    /// distance and alignment are horizontal; only the add stores a 3D distance.
    pub fn tick(&mut self, chars: &[(u64, Vec3)], exists: impl Fn(u64) -> bool, hero: Vec3, forward: Vec3, attacking: bool) -> Option<u64> {
        // 1. Adding.
        for &(id, p) in chars {
            if self.records.len() < RECORDS && !self.records.iter().any(|r| r.id == id) {
                self.records.push(Record { id, d: dist(p, hero), ..Record::default() });
            }
        }
        // 2. Refresh: a removal moves the later records up and the walk skips the one that
        // moved into the freed place.
        let mut refreshed: Vec<u64> = Vec::new();
        let mut i = 0;
        while i < self.records.len() {
            let id = self.records[i].id;
            match chars.iter().find(|c| c.0 == id) {
                None if exists(id) => {
                    self.records.remove(i);
                }
                None => {}
                Some(&(_, p)) => {
                    let u = [p[0] - hero[0], 0.0, p[2] - hero[2]];
                    let d = u[0].hypot(u[2]);
                    let l = d.max(1e-6);
                    let r = &mut self.records[i];
                    r.d = d;
                    r.a = (forward[0] * u[0] + forward[2] * u[2]) / l;
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
                    refreshed.push(id);
                }
            }
            i += 1;
        }
        // 4. Order: the in-range records to the front, then the pairwise pass.
        let mut front = 0;
        for k in 0..self.records.len() {
            if self.records[k].in_range && refreshed.contains(&self.records[k].id) {
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

/// The target list's range and cone by attack variant (0 Tor, 1 Roberta; boot data 0x2DD6A8 to
/// 0x2DD6B4), its rescan period (0x2DD6B8) and the near distance that keeps a hit target listed
/// as a candidate behind the hero.
pub const TARGET_RANGE: [f32; 2] = [20.0, 100.0];
pub const TARGET_CONE: f32 = 0.5;
pub const RESCAN: u32 = 90;
const NEAR: f32 = 6.0;

/// A target list record (SPEC-0053 Layout +0xA70): the character, its hit counter, distance,
/// alignment and candidate flag (in range, in front and with a clear path).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Target {
    pub id: u64,
    pub counter: u32,
    pub d: f32,
    pub dot: f32,
    pub flag: bool,
}

/// The hero's target list (SPEC-0053 3.9): up to 15 records; characters that pass the scan are
/// added on the rescan (every 90 ticks, and on the first tick after the list is emptied); each
/// tick every record is refreshed, a gone one dropped, and the candidates moved ahead.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct TargetList {
    pub records: Vec<Target>,
    clock: u32,
}

impl TargetList {
    /// The select step (SPEC-0053 step 9): empty the list.
    pub fn clear(&mut self) {
        *self = Self::default();
    }

    /// One tick. `chars` pass the scan's test (id, position); `variant` 0 Tor, 1 Roberta; `clear`
    /// is the clear-path test from the hero to a character.
    pub fn tick(&mut self, chars: &[(u64, Vec3)], hero: Vec3, forward: Vec3, variant: usize, clear: impl Fn(u64) -> bool) {
        let range = TARGET_RANGE[variant.min(1)];
        if self.clock == 0 {
            for &(id, p) in chars {
                if self.records.len() < RECORDS && !self.records.iter().any(|r| r.id == id) && dist(p, hero) <= range {
                    self.records.push(Target { id, ..Default::default() });
                }
            }
        }
        self.clock = (self.clock + 1) % RESCAN;
        self.records.retain(|r| chars.iter().any(|c| c.0 == r.id));
        for r in &mut self.records {
            let Some(&(_, p)) = chars.iter().find(|c| c.0 == r.id) else { continue };
            r.d = dist(p, hero);
            let to = [p[0] - hero[0], p[1] - hero[1], p[2] - hero[2]];
            let n = r.d.max(1e-6);
            r.dot = (forward[0] * to[0] + forward[1] * to[1] + forward[2] * to[2]) / n;
            r.flag = !((r.dot < TARGET_CONE && (r.counter < 1 || r.d > NEAR)) || r.d > range) && clear(r.id);
        }
        // Candidates ahead of non-candidates, each group in its order.
        let (mut a, b): (Vec<Target>, Vec<Target>) = self.records.iter().partition(|r| r.flag);
        a.extend(b);
        self.records = a;
    }

    /// Raise a listed character's counter (the hero's ten-entry list of hits, +0x400).
    pub fn hit(&mut self, id: u64) {
        if let Some(r) = self.records.iter_mut().find(|r| r.id == id) {
            r.counter += 1;
        }
    }
}

/// The talker list's range, cone and leader hold (SPEC-0022 Interaction range and target choice).
pub const TALK_RANGE: f32 = 20.0;
pub const TALK_CONE: f32 = 0.5;
pub const TALK_HOLD: u32 = 90;

#[derive(Clone, Copy, Debug, Default, PartialEq)]
struct Talker {
    id: u64,
    d: f32,
    eligible: bool,
}

/// The hero's talker list (SPEC-0022 Candidate lists): up to 15 townsfolk. A leader that stays
/// eligible is kept for up to 90 ticks even when a nearer one appears; otherwise the eligible
/// entries go first, in their pairwise order with the 1.0 swap quirk.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct TalkerList {
    records: Vec<Talker>,
    held: u32,
    leader: Option<u64>,
}

impl TalkerList {
    /// One tick; `chars` are the townsfolk (id, position) in character-list order. Returns the
    /// leader and its distance when it is eligible.
    pub fn tick(&mut self, chars: &[(u64, Vec3)], hero: Vec3, forward: Vec3) -> Option<(u64, f32)> {
        self.records.retain(|r| chars.iter().any(|c| c.0 == r.id));
        for &(id, _) in chars {
            if self.records.len() < RECORDS && !self.records.iter().any(|r| r.id == id) {
                self.records.push(Talker { id, ..Talker::default() });
            }
        }
        for r in &mut self.records {
            let Some(&(_, p)) = chars.iter().find(|c| c.0 == r.id) else { continue };
            r.d = dist(p, hero);
            let n = r.d.max(1e-6);
            let dot = (forward[0] * (p[0] - hero[0]) + forward[1] * (p[1] - hero[1]) + forward[2] * (p[2] - hero[2])) / n;
            r.eligible = r.d <= TALK_RANGE && dot >= TALK_CONE;
        }
        let kept = self.records.first().is_some_and(|r| r.eligible && Some(r.id) == self.leader) && self.held < TALK_HOLD;
        if kept {
            self.held += 1;
        } else {
            let (mut a, b): (Vec<Talker>, Vec<Talker>) = self.records.iter().partition(|r| r.eligible);
            for i in 0..a.len() {
                for j in i + 1..a.len() {
                    if a[j].d < a[i].d + 1.0 {
                        a.swap(i, j);
                    }
                }
            }
            a.extend(b);
            self.records = a;
            self.leader = self.records.first().map(|r| r.id);
            self.held = 0;
        }
        self.records.first().filter(|r| r.eligible).map(|r| (r.id, r.d))
    }
}

/// SPEC-0022 Talk candidate: the lock-on leader when it is eligible, unless the talker leader is
/// eligible and strictly nearer; when only one is eligible, that one.
pub fn talk_candidate(talker: Option<(u64, f32)>, lock_on: Option<(u64, f32)>) -> Option<u64> {
    match (talker, lock_on) {
        (Some(t), Some(l)) => Some(if t.1 < l.1 { t.0 } else { l.0 }),
        (Some(t), None) => Some(t.0),
        (None, Some(l)) => Some(l.0),
        (None, None) => None,
    }
}

/// The shared aim rule (SPEC-0040 Behavior 8) as the attack turn uses it (Behavior 20 item 7):
/// from facing `f` toward the horizontal direction to `to` from `at`, weight 1.0 under 4.0
/// (strictly) and 0.25 otherwise; when the two point opposite (dot below -0.99), 0.1 times the
/// body side axis is added. `ahead_only` (Tor) refuses the instant turn at a target not ahead.
/// None when there is no turn.
pub fn aim(f: Vec3, at: Vec3, to: Vec3, ahead_only: bool) -> Option<Vec3> {
    let (dx, dz) = (to[0] - at[0], to[2] - at[2]);
    let dd = dx.hypot(dz);
    if dd < 1e-6 {
        return None;
    }
    let d = [dx / dd, 0.0, dz / dd];
    let dot = f[0] * d[0] + f[2] * d[2];
    let w = if dd < 4.0 {
        if ahead_only && dot <= 0.0 {
            return None;
        }
        1.0
    } else {
        0.25
    };
    let mut v = [w * d[0] + (1.0 - w) * f[0], 0.0, w * d[2] + (1.0 - w) * f[2]];
    if dot < -0.99 {
        // The body side axis (right of the facing).
        v[0] += 0.1 * f[2];
        v[2] -= 0.1 * f[0];
    }
    let n = v[0].hypot(v[2]);
    (n > 1e-6).then(|| [v[0] / n, 0.0, v[2] / n])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn talker_leader_is_held_for_90_ticks() {
        let (hero, f) = ([0.0; 3], [0.0, 0.0, 1.0]);
        let mut l = TalkerList::default();
        let far = (1, [0.0, 0.0, 10.0]);
        assert_eq!(l.tick(&[far], hero, f), Some((1, 10.0)));
        // A nearer talker appears: the leader stays until the hold runs out.
        let near = (2, [0.0, 0.0, 3.0]);
        for _ in 0..TALK_HOLD {
            assert_eq!(l.tick(&[far, near], hero, f).unwrap().0, 1);
        }
        assert_eq!(l.tick(&[far, near], hero, f).unwrap().0, 2);
        // A leader that leaves the cone gives way at once; none eligible gives none.
        let behind = (2, [0.0, 0.0, -3.0]);
        assert_eq!(l.tick(&[far, behind], hero, f).unwrap().0, 1);
        let too_far = (1, [0.0, 0.0, 20.5]);
        assert_eq!(l.tick(&[too_far, behind], hero, f), None);
    }

    #[test]
    fn talker_reorder_swaps_within_one_unit() {
        let (hero, f) = ([0.0; 3], [0.0, 0.0, 1.0]);
        let mut l = TalkerList::default();
        // Entry 1 is up to 1.0 farther and still swaps ahead (the quirk).
        assert_eq!(l.tick(&[(1, [0.0, 0.0, 5.0]), (2, [0.0, 0.0, 5.5])], hero, f).unwrap().0, 2);
        let mut l = TalkerList::default();
        assert_eq!(l.tick(&[(1, [0.0, 0.0, 5.0]), (2, [0.0, 0.0, 6.5])], hero, f).unwrap().0, 1);
    }

    #[test]
    fn talk_candidate_lock_on_steal() {
        assert_eq!(talk_candidate(Some((1, 5.0)), Some((2, 5.0))), Some(2), "a tie goes to the lock-on leader");
        assert_eq!(talk_candidate(Some((1, 4.9)), Some((2, 5.0))), Some(1));
        assert_eq!(talk_candidate(Some((1, 6.0)), Some((2, 5.0))), Some(2));
        assert_eq!(talk_candidate(None, Some((2, 5.0))), Some(2));
        assert_eq!(talk_candidate(Some((1, 5.0)), None), Some(1));
        assert_eq!(talk_candidate(None, None), None);
    }

    #[test]
    fn engagement_range_and_order() {
        let mut c = Candidates::default();
        let fwd = [0.0, 0.0, 1.0];
        let chars = [(1, [0.0, 0.0, 10.0]), (2, [0.0, 0.0, 15.0]), (3, [0.0, 0.0, -5.0])];
        // Not engaged until the hero attacks while facing the candidate.
        assert_eq!(c.tick(&chars, |_| true, [0.0; 3], fwd, false), None);
        assert_eq!(c.tick(&chars, |_| true, [0.0; 3], fwd, true), Some(1));
        // The one behind (alignment -1) is never engaged; at 20.0 a record loses both flags.
        assert!(!c.records.iter().find(|r| r.id == 3).unwrap().engaged);
        let far = [(1, [0.0, 0.0, 20.0]), (2, [0.0, 0.0, 15.0]), (3, [0.0, 0.0, -5.0])];
        assert_eq!(c.tick(&far, |_| true, [0.0; 3], fwd, false), Some(2));
        // Two in range within 1.0 of each other swap every tick (the quirk).
        let close = [(1, [0.0, 0.0, 10.0]), (2, [0.0, 0.0, 10.5])];
        let mut c = Candidates::default();
        let a = c.tick(&close, |_| true, [0.0; 3], fwd, true);
        let b = c.tick(&close, |_| true, [0.0; 3], fwd, true);
        assert_ne!(a, b);
        // A removed candidate leaves the list; 15 records at most.
        let many: Vec<(u64, Vec3)> = (0..20).map(|k| (k, [0.0, 0.0, 30.0])).collect();
        let mut c = Candidates::default();
        c.tick(&many, |_| true, [0.0; 3], fwd, false);
        assert_eq!(c.records.len(), RECORDS);
        c.tick(&many[1..], |_| true, [0.0; 3], fwd, false);
        assert!(!c.records.iter().any(|r| r.id == 0));
        // A record whose character no longer exists is kept as it is.
        c.tick(&many[2..], |id| id != 1, [0.0; 3], fwd, false);
        assert!(c.records.iter().any(|r| r.id == 1));
        // Refresh distance is horizontal: a candidate 10 ahead and 30 up is at 10.
        let mut c = Candidates::default();
        c.tick(&[(7, [0.0, 30.0, 10.0])], |_| true, [0.0; 3], fwd, true);
        assert!((c.records[0].d - 10.0).abs() < 1e-5 && (c.records[0].a - 1.0).abs() < 1e-5);
    }

    #[test]
    fn attack_turn_vectors() {
        // SPEC-0040 vector 161: facing (0, 0, 1), target point (3, 5, 0): D = 3, instant turn.
        let f = aim([0.0, 0.0, 1.0], [0.0; 3], [3.0, 5.0, 0.0], false).unwrap();
        assert!((f[0] - 1.0).abs() < 1e-5 && f[2].abs() < 1e-5);
        // Target at (8, 0, 0): weight 0.25, (0.31623, 0, 0.94868).
        let f = aim([0.0, 0.0, 1.0], [0.0; 3], [8.0, 0.0, 0.0], false).unwrap();
        assert!((f[0] - 0.31623).abs() < 1e-4 && (f[2] - 0.94868).abs() < 1e-4);
        // Tor refuses the instant turn at a near target behind him.
        assert_eq!(aim([0.0, 0.0, 1.0], [0.0; 3], [0.0, 0.0, -3.0], true), None);
    }

    #[test]
    fn target_list_flags() {
        let mut t = TargetList::default();
        let fwd = [0.0, 0.0, 1.0];
        // Ahead within range: a candidate; behind: not; beyond 20: not listed.
        let chars = [(1, [0.0, 0.0, 10.0]), (2, [0.0, 0.0, -5.0]), (3, [0.0, 0.0, 30.0])];
        t.tick(&chars, [0.0; 3], fwd, 0, |_| true);
        assert_eq!(t.records.iter().map(|r| (r.id, r.flag)).collect::<Vec<_>>(), vec![(1, true), (2, false)]);
        // A hit target close behind stays a candidate; a blocked path clears the flag.
        t.hit(2);
        t.tick(&chars, [0.0; 3], fwd, 0, |id| id != 1);
        assert_eq!(t.records.iter().map(|r| (r.id, r.flag)).collect::<Vec<_>>(), vec![(2, true), (1, false)]);
    }
}
