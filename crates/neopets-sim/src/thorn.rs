//! The Faerie Sisters' thorn hazards (SPEC-0011 Main-loop launches, Thorn hits; SPEC-0029
//! Layout 1a): five records, each placed where a thorn shot ended. A hazard hurts the hero on
//! contact (then waits 120 ticks), grows for 60 ticks, and is removed by the third hero melee
//! swing that reaches it after the growth (each swing counts once).

pub type Vec3 = [f32; 3];

/// The pool size (SPEC-0029 Layout 1a).
pub const RECORDS: usize = 5;
/// Ticks of growth before swings count.
pub const GROW: i32 = 60;
/// Ticks after a contact hit before the next.
pub const REHIT: i32 = 120;
/// The contact radius word (0x2DCB20, 1.0).
pub const RADIUS_WORD: u32 = 0x002D_CB20;
/// The thorn shot's projectile kind (SPEC-0011 Projectile kinds).
pub const SHOT_KIND: i32 = 16;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Hazard {
    pub position: Vec3,
    pub age: i32,
    pub wait: i32,
    pub strikes: i32,
    /// The serial of the last swing counted.
    pub swing: Option<u32>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Pool {
    pub records: [Option<Hazard>; RECORDS],
}

/// What a hazard tick reports.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Event {
    /// The hero touched hazard `i` and takes the thorn damage.
    Contact(usize),
    /// Hazard `i` took its last strike and was removed.
    Removed(usize),
}

impl Pool {
    /// Claims the first free record at `at`; false (nothing spawned) when all five are in use.
    pub fn spawn(&mut self, at: Vec3) -> bool {
        let Some(r) = self.records.iter_mut().find(|r| r.is_none()) else { return false };
        *r = Some(Hazard { position: at, age: 0, wait: 0, strikes: 0, swing: None });
        true
    }

    pub fn reset(&mut self) {
        self.records = [None; RECORDS];
    }

    pub fn live(&self) -> impl Iterator<Item = &Hazard> {
        self.records.iter().flatten()
    }

    /// One tick: `touches(p)` is the hero body test against a hazard at `p`; `swing` is the
    /// current swing's serial and `reaches(p)` its weapon volume test (None without a live
    /// swing window); `max_hits` is the strike count that removes a hazard.
    pub fn tick(&mut self, touches: &dyn Fn(Vec3) -> bool, swing: Option<(u32, &dyn Fn(Vec3) -> bool)>, max_hits: i32) -> Vec<Event> {
        let mut out = Vec::new();
        for (i, slot) in self.records.iter_mut().enumerate() {
            let Some(h) = slot.as_mut() else { continue };
            h.age += 1;
            if h.wait > 0 {
                h.wait -= 1;
            } else if touches(h.position) {
                h.wait = REHIT;
                out.push(Event::Contact(i));
            }
            if h.age > GROW {
                if let Some((serial, reaches)) = swing {
                    if h.swing != Some(serial) && reaches(h.position) {
                        h.swing = Some(serial);
                        h.strikes += 1;
                        if h.strikes >= max_hits {
                            *slot = None;
                            out.push(Event::Removed(i));
                        }
                    }
                }
            }
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn three_swings_remove_after_growth() {
        let mut p = Pool::default();
        assert!(p.spawn([0.0; 3]));
        let no = |_: Vec3| false;
        let yes = |_: Vec3| true;
        // A swing during the growth does not count.
        p.tick(&no, Some((1, &yes)), 3);
        for _ in 0..GROW {
            p.tick(&no, None, 3);
        }
        // Two separate swings, each held for several ticks, count twice.
        for serial in [2, 3] {
            for _ in 0..4 {
                assert!(p.tick(&no, Some((serial, &yes)), 3).is_empty());
            }
        }
        assert_eq!(p.live().next().map(|h| h.strikes), Some(2));
        assert_eq!(p.tick(&no, Some((4, &yes)), 3), vec![Event::Removed(0)]);
        assert_eq!(p.live().count(), 0);
    }

    #[test]
    fn contact_waits_and_pool_fills() {
        let mut p = Pool::default();
        p.spawn([0.0; 3]);
        let yes = |_: Vec3| true;
        let hits = (0..=REHIT + 1).filter(|_| !p.tick(&yes, None, 3).is_empty()).count();
        assert_eq!(hits, 2);
        for _ in 1..RECORDS {
            assert!(p.spawn([1.0; 3]));
        }
        assert!(!p.spawn([2.0; 3]));
    }
}
