//! Townsfolk (behavior class 6) per-tick routine (SPEC-0056 Townsfolk routine, Modes, Greeting,
//! Mode from code, Animation ids by mode): the greeting entry pre-step, the mode set from the
//! behavior code, and the bodies of modes 0, 10, 11, 12 and 13, and the patrol (mode 6) along
//! the placement's path (SPEC-0056 Patrol; the host moves the character toward `walk_to`).
//! Not covered: wander (5) needs the navigation mesh; in this build it stands idle. The
//! villager pair (never formed on disc), navigation index updates and base states other than
//! the conversation are not covered.

use crate::hero::{Vec3, steer};
use crate::rng::Rng;

/// Animation ids that mark a pose for the greeting (SPEC-0056 Greeting).
const POSES: [u32; 13] = [65, 66, 67, 68, 69, 70, 71, 72, 73, 74, 78, 97, 98];
const GREET_RANGE: f32 = 6.0;

#[derive(Clone, Debug, PartialEq)]
pub struct Townsfolk {
    /// Behavior code (placement +0x84) and the current mode.
    pub code: i16,
    pub mode: i16,
    /// Placement facing (mode 0 with code 0 turns back toward it).
    pub home_facing: Vec3,
    counter: u32,
    done: bool,
    pose: bool,
    /// The patrol of mode 6, when the placement names a path of the area's path list.
    pub patrol: Option<Patrol>,
    /// This tick's walk target (mode 6 walking), for the host to move toward.
    pub walk_to: Option<Vec3>,
}

/// The townsfolk patrol (mode 6; SPEC-0056 Patrol (mode 6)): the route's points and the
/// placement's loop mode (+0xBC) and timing values (+0xC4 to +0xD4).
#[derive(Clone, Debug, PartialEq)]
pub struct Patrol {
    points: Vec<Vec3>,
    looping: bool,
    hold: (i32, i32),
    chance: i32,
    stop: (i32, i32),
    index: usize,
    forward: bool,
    reached: bool,
    started: bool,
    pause: i32,
    fidget: i32,
    fidget_asked: bool,
}

impl Patrol {
    /// `points` are the route's positions; `words` the placement's +0xB8 to +0xD4.
    pub fn new(points: Vec<Vec3>, words: [u32; 8]) -> Self {
        let w = |i: usize| words[i] as i32;
        Self { points, looping: w(1) == 0, hold: (w(3), w(4)), chance: w(5), stop: (w(6), w(7)), index: 0, forward: true, reached: false, started: false, pause: 0, fidget: 0, fidget_asked: false }
    }

    /// The route point now targeted.
    pub fn target(&self) -> Option<Vec3> {
        self.points.get(self.index).copied()
    }

    /// The path step: loop mode wraps after the last point; back-and-forth runs to the last
    /// point, back to point 0 and forward again.
    fn advance(&mut self) {
        let n = self.points.len();
        if self.looping {
            self.index = (self.index + 1) % n;
        } else if self.forward {
            if self.index + 1 < n {
                self.index += 1;
            } else {
                self.forward = false;
                self.index = self.index.saturating_sub(1);
            }
        } else if self.index > 0 {
            self.index -= 1;
        } else {
            self.forward = true;
            self.index = 1.min(n - 1);
        }
    }

    /// One tick: the animation id to request and the point to walk to (None: stand).
    fn tick(&mut self, v: &View, rng: &mut Rng) -> (Option<u32>, Option<Vec3>) {
        if self.points.is_empty() {
            return (Some(0), None);
        }
        // Step 1: the route starts once with the reached flag set, so the first step (on the
        // start tick) targets point 1.
        if !self.started {
            self.started = true;
            self.reached = true;
        }
        if self.pause > 0 {
            self.pause -= 1;
        }
        if self.reached {
            self.advance();
            self.reached = false;
        }
        let target = self.points[self.index];
        if self.pause < 1 {
            // Step 3: arrival below 0.5 draws both timers from the point-pause range; else a
            // roll from 1 to 100 below the chance draws them from the random-stop range.
            if horizontal(v.position, target).0 < 0.5 {
                self.reached = true;
                self.pause = rng.range(self.hold.0, self.hold.1);
                self.fidget = rng.range(self.hold.0, self.hold.1);
                return (Some(0), None);
            }
            if rng.range(1, 100) < self.chance {
                self.pause = rng.range(self.stop.0, self.stop.1);
                self.fidget = rng.range(self.stop.0, self.stop.1);
            }
            return (Some(8), Some(target));
        }
        // Step 4: the pause, as wander step 4 with fidget id 11.
        if self.fidget >= 1 {
            if v.fidget_entry {
                self.fidget -= 1;
            }
            (Some(0), None)
        } else if !self.fidget_asked {
            self.fidget_asked = true;
            (Some(11), None)
        } else if v.action == 11 && v.progress >= 0.99 {
            self.fidget = rng.range(self.hold.0, self.hold.1);
            self.fidget_asked = false;
            (Some(0), None)
        } else {
            (None, None)
        }
    }
}

/// The script path walk (class 35; SPEC-0056 Path walk): the route's points from point 0, a
/// point reached below 0.5 horizontally, the class ending on the tick after the last point.
#[derive(Clone, Debug, PartialEq)]
pub struct PathWalk {
    points: Vec<Vec3>,
    index: usize,
    reached: bool,
}

/// One tick of a path walk.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PathStep {
    /// Walk toward the point this tick, then call `moved`.
    Walk(Vec3),
    /// The step to the next point (the walk resumes on the next tick).
    Next,
    /// The class ends (no animation request).
    End,
}

impl PathWalk {
    pub fn new(points: Vec<Vec3>) -> Self {
        Self { points, index: 0, reached: false }
    }

    /// The start of a tick.
    pub fn begin(&mut self) -> PathStep {
        if self.reached {
            if self.index + 1 >= self.points.len() {
                return PathStep::End;
            }
            self.index += 1;
            self.reached = false;
            return PathStep::Next;
        }
        self.points.get(self.index).map_or(PathStep::End, |&p| PathStep::Walk(p))
    }

    /// After the navigation step: the point is reached below 0.5.
    pub fn moved(&mut self, position: Vec3) {
        if let Some(&p) = self.points.get(self.index) {
            self.reached = horizontal(position, p).0 < 0.5;
        }
    }
}

/// What the routine reads of the world this tick.
#[derive(Clone, Copy, Debug)]
pub struct View {
    pub position: Vec3,
    pub hero: Vec3,
    /// The animation id playing and its clip progress.
    pub action: u32,
    pub progress: f32,
    /// The animation table has an entry for the patrol fidget (id 11).
    pub fidget_entry: bool,
}

fn horizontal(a: Vec3, b: Vec3) -> (f32, Vec3) {
    let d = [b[0] - a[0], 0.0, b[2] - a[2]];
    let len = (d[0] * d[0] + d[2] * d[2]).sqrt();
    let unit = if len > 0.0 { d.map(|c| c / len) } else { d };
    (len, unit)
}

impl Townsfolk {
    pub fn new(code: i16, home_facing: Vec3) -> Self {
        Self { code, mode: 0, home_facing, counter: 0, done: false, pose: false, patrol: None, walk_to: None }
    }

    /// The same with the patrol of mode 6 (the route the placement names, if any).
    pub fn with_patrol(self, patrol: Option<Patrol>) -> Self {
        Self { patrol, ..self }
    }

    /// One tick: returns the animation id to request (None: keep what is playing), and turns
    /// `facing` where the mode says so.
    pub fn tick(&mut self, facing: &mut Vec3, v: View, rng: &mut Rng) -> Option<u32> {
        self.walk_to = None;
        let (d, to_hero) = horizontal(v.position, v.hero);
        let front = to_hero[0] * facing[0] + to_hero[2] * facing[2];
        // Pre-step: the greeting entry in every mode except 10 to 12 (it repeats each tick the
        // conditions hold, restarting the counter); then mode 0 takes the code.
        if !matches!(self.mode, 10..=12) && d < GREET_RANGE && front > 0.0 {
            self.mode = 13;
            self.counter = 0;
            self.done = false;
            self.pose = POSES.contains(&v.action);
        }
        if self.mode == 0 {
            self.mode = self.code;
        }
        match self.mode {
            0 => {
                if self.code == 0 {
                    *facing = steer(*facing, self.home_facing, 0.05);
                }
                Some(0)
            }
            10 => Some(65),
            11 => Some(97),
            12 => Some(98),
            13 => {
                if d > 2.0 * GREET_RANGE || front < -0.1 {
                    self.mode = 0;
                    return None;
                }
                if self.pose {
                    None
                } else if self.counter < 180 {
                    self.counter += 1;
                    Some(0)
                } else if !self.done {
                    if v.action == 30 && v.progress >= 0.99 {
                        self.done = true;
                        Some(0)
                    } else {
                        Some(30)
                    }
                } else {
                    Some(0)
                }
            }
            6 => match self.patrol.as_mut() {
                Some(p) => {
                    let (id, to) = p.tick(&v, rng);
                    self.walk_to = to;
                    id
                }
                None => Some(0),
            },
            // Wander is not implemented: stand.
            5 => Some(0),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn view(hero: Vec3, action: u32, progress: f32) -> View {
        View { position: [0.0; 3], hero, action, progress, fidget_entry: true }
    }

    #[test]
    fn patrol_targets_point_1_first_and_pauses_on_arrival() {
        // Loop mode, point pause 3 to 3, no random stop.
        let pts = vec![[0.0, 0.0, 0.0], [0.0, 0.0, 10.0], [5.0, 0.0, 10.0]];
        let mut t = Townsfolk::new(6, [0.0, 0.0, 1.0]);
        t.patrol = Some(Patrol::new(pts, [0, 0, 0, 3, 3, 0, 0, 0]));
        let (mut f, mut r) = ([0.0, 0.0, 1.0], Rng::seeded(1));
        let far = [100.0, 0.0, -100.0];
        let at = |p: Vec3| View { position: p, hero: far, action: 8, progress: 0.0, fidget_entry: true };
        assert_eq!(t.tick(&mut f, at([0.0; 3]), &mut r), Some(8));
        assert_eq!(t.walk_to, Some([0.0, 0.0, 10.0]), "the first target is point 1");
        // Arriving (below 0.5): idle, no walk; three pause ticks, then on to point 2.
        assert_eq!(t.tick(&mut f, at([0.0, 0.0, 9.6]), &mut r), Some(0));
        assert_eq!(t.walk_to, None);
        let mut walked = 0;
        for _ in 0..3 {
            t.tick(&mut f, at([0.0, 0.0, 9.6]), &mut r);
            walked += t.walk_to.is_some() as u32;
        }
        assert_eq!(walked, 1);
        assert_eq!(t.walk_to, Some([5.0, 0.0, 10.0]));
    }

    #[test]
    fn path_walk_starts_at_point_0_and_ends_a_tick_after_the_last() {
        let mut w = PathWalk::new(vec![[0.0; 3], [0.0, 0.0, 2.0]]);
        assert_eq!(w.begin(), PathStep::Walk([0.0; 3]), "point 0 first");
        // TV-0056-3: 0.49 is reached, 0.50 is not.
        w.moved([0.5, 0.0, 0.0]);
        assert_eq!(w.begin(), PathStep::Walk([0.0; 3]));
        w.moved([0.49, 0.0, 0.0]);
        assert_eq!(w.begin(), PathStep::Next);
        assert_eq!(w.begin(), PathStep::Walk([0.0, 0.0, 2.0]));
        w.moved([0.0, 0.0, 2.0]);
        assert_eq!(w.begin(), PathStep::End);
    }

    #[test]
    fn patrol_back_and_forth_and_loop_orders() {
        let pts = vec![[0.0; 3], [1.0, 0.0, 0.0], [2.0, 0.0, 0.0]];
        let order = |looping: u32| {
            let mut p = Patrol::new(pts.clone(), [0, looping, 0, 0, 0, 0, 0, 0]);
            (0..6)
                .map(|_| {
                    p.advance();
                    p.index
                })
                .collect::<Vec<_>>()
        };
        assert_eq!(order(0), [1, 2, 0, 1, 2, 0]);
        assert_eq!(order(1), [1, 2, 1, 0, 1, 2]);
    }

    #[test]
    fn greeting_waits_180_ticks_then_plays_30_once() {
        let mut t = Townsfolk::new(0, [0.0, 0.0, 1.0]);
        let mut f = [0.0, 0.0, 1.0];
        // Hero 4 ahead: the greeting starts and restarts each tick, so it idles while the hero
        // stays; the counter only runs once the hero has left the 6.0 entry range but is
        // still within 12.0 and in front.
        assert_eq!(t.tick(&mut f, view([0.0, 0.0, 4.0], 0, 0.0), &mut Rng::seeded(1)), Some(0));
        assert_eq!(t.mode, 13);
        let far = view([0.0, 0.0, 8.0], 0, 0.0);
        let mut ids = Vec::new();
        for _ in 0..181 {
            ids.push(t.tick(&mut f, far, &mut Rng::seeded(1)));
        }
        assert!(ids[..179].iter().all(|&i| i == Some(0)));
        assert_eq!(ids[180], Some(30));
        assert_eq!(t.tick(&mut f, view([0.0, 0.0, 8.0], 30, 0.995), &mut Rng::seeded(1)), Some(0));
        // Beyond 12.0 the greeting ends; code 0 returns to mode 0.
        assert_eq!(t.tick(&mut f, view([0.0, 0.0, 13.0], 0, 0.0), &mut Rng::seeded(1)), None);
        assert_eq!(t.mode, 0);
    }

    #[test]
    fn idle_code_0_turns_back_to_its_placement_facing() {
        let mut t = Townsfolk::new(0, [1.0, 0.0, 0.0]);
        let mut f = [0.0, 0.0, 1.0];
        let behind = view([0.0, 0.0, -20.0], 0, 0.0);
        t.tick(&mut f, behind, &mut Rng::seeded(1));
        assert!(f[0] > 0.0 && f[0] < 0.1, "a 0.05 blend step: {f:?}");
        for _ in 0..200 {
            t.tick(&mut f, behind, &mut Rng::seeded(1));
        }
        assert!(f[0] > 0.99);
    }

    #[test]
    fn poses_by_code_and_no_greeting_in_them() {
        // The greeting entry runs before mode-from-code, so start with the hero away.
        let mut t = Townsfolk::new(10, [0.0, 0.0, 1.0]);
        let mut f = [0.0, 0.0, 1.0];
        assert_eq!(t.tick(&mut f, view([0.0, 0.0, -30.0], 0, 0.0), &mut Rng::seeded(1)), Some(65));
        assert_eq!(t.tick(&mut f, view([0.0, 0.0, 3.0], 65, 0.0), &mut Rng::seeded(1)), Some(65), "no greeting in mode 10");
        assert_eq!(t.mode, 10);
    }
}
