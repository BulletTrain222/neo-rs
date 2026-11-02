//! Townsfolk (behavior class 6) per-tick routine (SPEC-0056 Townsfolk routine, Modes, Greeting,
//! Mode from code, Animation ids by mode): the greeting entry pre-step, the mode set from the
//! behavior code, and the bodies of modes 0, 10, 11, 12 and 13, and the patrol (mode 6) along
//! the placement's path (SPEC-0056 Patrol; the host moves the character toward `walk_to`), and
//! the wander (mode 5; SPEC-0056 Wander). The host walks them over their navigation mesh. The
//! villager pair (never formed on disc), navigation index updates and base states other than
//! the conversation are not covered.

use crate::hero::{Vec3, steer};
use crate::nav::NavMesh;
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
    /// The wander of mode 5.
    pub wander: Option<Wander>,
}

/// The townsfolk wander (mode 5; SPEC-0056 Wander): the placement's radius (+0xBC), pause range
/// (+0xC0, +0xC4) and pause chance (+0xC8).
#[derive(Clone, Debug, PartialEq)]
pub struct Wander {
    home: Vec3,
    radius: f32,
    pause_range: (i32, i32),
    chance: i32,
    point: Option<Vec3>,
    pause: i32,
    fidget: i32,
    asked: bool,
    /// This tick the character walks.
    pub walking: bool,
    /// Ticks on one point (this build's guard: 600 ticks count as reached, so a character whose
    /// route is blocked by something the mesh does not hold does not walk in place for ever).
    spent: u32,
}

impl Wander {
    pub fn new(home: Vec3, words: [u32; 8]) -> Self {
        let (a, b) = (words[2] as i32, words[3] as i32);
        Self { home, radius: f32::from_bits(words[1]), pause_range: (a.min(b), a.max(b)), chance: words[4] as i32, point: None, pause: 0, fidget: 0, asked: false, walking: false, spent: 0 }
    }

    /// Form H of the spawn point search (SPEC-0056): a whole-number point in the square of side
    /// r about home, moved halfway toward the character while off the mesh; the 10th retry gives
    /// home. With r below 0.1, a random cell's centroid.
    fn pick(&self, m: &NavMesh, p: Vec3, rng: &mut Rng) -> Vec3 {
        if self.radius < 0.1 {
            return m.centroid(rng.index(m.len() as u32) as usize);
        }
        let n = self.radius as i32;
        let draw = |rng: &mut Rng| if n > 0 { rng.range(0, n - 1) as f32 } else { 0.0 };
        let (a, b) = (draw(rng), draw(rng));
        let mut q = [self.home[0] + a - self.radius / 2.0, self.home[1], self.home[2] + b - self.radius / 2.0];
        for _ in 0..10 {
            if m.first_holding(q).is_some() {
                return q;
            }
            q = std::array::from_fn(|i| (q[i] + p[i]) / 2.0);
        }
        self.home
    }

    /// One tick on the character's mesh: the id to request and the point to walk to.
    fn tick(&mut self, v: &View, mesh: Option<&NavMesh>, rng: &mut Rng) -> (Option<u32>, Option<Vec3>) {
        self.walking = false;
        // A character outside every kind-1 mesh never walks.
        let Some(m) = mesh.filter(|m| !m.is_empty()) else {
            return (Some(0), None);
        };
        if self.pause > 0 {
            self.pause -= 1;
        }
        if self.point.is_none() {
            self.point = Some(self.pick(m, v.position, rng));
            self.spent = 0;
        }
        if self.pause < 1 {
            let point = self.point.unwrap_or(self.home);
            self.spent += 1;
            if horizontal(v.position, point).0 < 1.5 || v.arrived || self.spent >= 600 {
                self.point = None;
                if rng.range(1, 100) < self.chance {
                    self.pause = rng.range(self.pause_range.0, self.pause_range.1);
                    self.fidget = rng.range(self.pause_range.0, self.pause_range.1);
                    self.asked = false;
                    return (Some(0), None);
                }
                return (Some(1), None);
            }
            self.walking = true;
            return (Some(1), Some(point));
        }
        if self.asked {
            if v.action == 2 && v.progress >= 0.99 {
                self.fidget = rng.range(self.pause_range.0, self.pause_range.1);
                self.asked = false;
                return (Some(0), None);
            }
            return (None, None);
        }
        if self.fidget >= 1 {
            if v.wander_fidget {
                self.fidget -= 1;
            }
            return (Some(0), None);
        }
        self.asked = true;
        (Some(2), None)
    }
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
    /// The route's points.
    pub fn points(&self) -> &[Vec3] {
        &self.points
    }

    /// `points` are the route's positions; `words` the placement's +0xB8 to +0xD4.
    pub fn new(points: Vec<Vec3>, words: [u32; 8]) -> Self {
        let w = |i: usize| words[i] as i32;
        Self { points, looping: w(1) == 0, hold: (w(3), w(4)), chance: w(5), stop: (w(6), w(7)), index: 0, forward: true, reached: false, started: false, pause: 0, fidget: 0, fidget_asked: false }
    }

    /// The common-enemy form of the route (SPEC-0038 state 6): the route starts at point 1, a
    /// point counts as reached within 0.5 and starts a pause of int[+0xC4, +0xC8] ticks; while
    /// walking, `may_stop` (the tick the awareness countdown is 0) allows the roll below +0xCC
    /// that stops for int[+0xD0, +0xD4] ticks. Returns the point to walk to (None while paused)
    /// and, on the tick a pause starts, its range (for the fidget counter).
    pub fn enemy_step(&mut self, position: Vec3, may_stop: bool, rng: &mut Rng) -> (Option<Vec3>, Option<(i32, i32)>) {
        if self.points.is_empty() {
            return (None, None);
        }
        let draw = |r: (i32, i32), rng: &mut Rng| rng.range(r.0.min(r.1), r.0.max(r.1));
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
        if self.pause >= 1 {
            return (None, None);
        }
        let target = self.points[self.index];
        if horizontal(position, target).0 < 0.5 {
            self.reached = true;
            self.pause = draw(self.hold, rng);
            return (None, Some(self.hold));
        }
        if may_stop && rng.range(1, 100) < self.chance {
            self.pause = draw(self.stop, rng);
            return (None, Some(self.stop));
        }
        (Some(target), None)
    }

    /// This build's guard: the point cannot be reached, so go on to the next.
    pub fn point_stuck(&mut self) {
        self.reached = true;
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
    /// The animation table has an entry for the wander fidget (id 2).
    pub wander_fidget: bool,
    /// The last navigation step arrived (the settled arrival, or a fresh block).
    pub arrived: bool,
}

fn horizontal(a: Vec3, b: Vec3) -> (f32, Vec3) {
    let d = [b[0] - a[0], 0.0, b[2] - a[2]];
    let len = (d[0] * d[0] + d[2] * d[2]).sqrt();
    let unit = if len > 0.0 { d.map(|c| c / len) } else { d };
    (len, unit)
}

impl Townsfolk {
    pub fn new(code: i16, home_facing: Vec3) -> Self {
        Self { code, mode: 0, home_facing, counter: 0, done: false, pose: false, patrol: None, walk_to: None, wander: None }
    }

    /// The same with the patrol of mode 6 (the route the placement names, if any).
    pub fn with_patrol(self, patrol: Option<Patrol>) -> Self {
        Self { patrol, ..self }
    }

    /// One tick with no navigation mesh (a wander then stands).
    pub fn tick(&mut self, facing: &mut Vec3, v: View, rng: &mut Rng) -> Option<u32> {
        self.tick_on(facing, v, rng, None)
    }

    /// One tick: returns the animation id to request (None: keep what is playing), and turns
    /// `facing` where the mode says so.
    pub fn tick_on(&mut self, facing: &mut Vec3, v: View, rng: &mut Rng, mesh: Option<&NavMesh>) -> Option<u32> {
        self.walk_to = None;
        if let Some(w) = self.wander.as_mut() {
            w.walking = false;
        }
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
            // A character outside every kind-1 mesh never walks (SPEC-0056 Walker interruptions).
            6 => match self.patrol.as_mut().filter(|_| mesh.is_some()) {
                Some(p) => {
                    let (id, to) = p.tick(&v, rng);
                    self.walk_to = to;
                    id
                }
                None => Some(0),
            },
            5 => match self.wander.as_mut() {
                Some(w) => {
                    let (id, to) = w.tick(&v, mesh, rng);
                    self.walk_to = to;
                    id
                }
                None => Some(0),
            },
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn view(hero: Vec3, action: u32, progress: f32) -> View {
        View { position: [0.0; 3], hero, action, progress, fidget_entry: true, wander_fidget: true, arrived: false }
    }

    #[test]
    fn wander_walks_pauses_and_stays_near_home() {
        // One large triangle about the origin.
        let mesh = NavMesh { kind: 1, triangles: vec![[[-60.0, 0.0, -60.0], [60.0, 0.0, -60.0], [0.0, 0.0, 60.0]]], neighbours: vec![[-1; 3]], table: Vec::new() };
        // Radius 10, pause 5 to 5, chance 101 (always pauses).
        let mut words = [0u32; 8];
        words[1] = 10.0f32.to_bits();
        (words[2], words[3], words[4]) = (5, 5, 101);
        let mut t = Townsfolk::new(5, [0.0, 0.0, 1.0]);
        t.wander = Some(Wander::new([0.0; 3], words));
        let (mut f, mut r) = ([0.0, 0.0, 1.0], Rng::seeded(1));
        let far = [100.0, 0.0, 100.0];
        let mut p = [0.0; 3];
        let (mut walked, mut paused) = (0, 0);
        for _ in 0..3000 {
            let id = t.tick_on(&mut f, View { position: p, hero: far, action: 1, progress: 0.0, fidget_entry: true, wander_fidget: true, arrived: false }, &mut r, Some(&mesh));
            if let Some(d) = t.walk_to {
                walked += 1;
                assert_eq!(id, Some(1), "walking asks for the walk id");
                let (_, u) = horizontal(p, d);
                p = [p[0] + 0.1 * u[0], 0.0, p[2] + 0.1 * u[2]];
                assert!(d[0].abs() <= 5.0 && d[2].abs() <= 5.0, "the point stays in the square of side r");
            } else {
                paused += 1;
            }
        }
        assert!(walked > 100 && paused > 10);
        // Outside every mesh: stands, and never walks.
        let mut t = Townsfolk::new(5, [0.0, 0.0, 1.0]);
        t.wander = Some(Wander::new([500.0, 0.0, 0.0], words));
        let v = View { position: [500.0, 0.0, 0.0], hero: far, action: 0, progress: 0.0, fidget_entry: true, wander_fidget: true, arrived: false };
        assert_eq!(t.tick_on(&mut f, v, &mut r, None), Some(0));
        assert_eq!(t.walk_to, None);
    }

    #[test]
    fn patrol_targets_point_1_first_and_pauses_on_arrival() {
        let mesh = NavMesh { kind: 1, triangles: vec![[[-60.0, 0.0, -60.0], [60.0, 0.0, -60.0], [0.0, 0.0, 60.0]]], neighbours: vec![[-1; 3]], table: Vec::new() };
        // Loop mode, point pause 3 to 3, no random stop.
        let pts = vec![[0.0, 0.0, 0.0], [0.0, 0.0, 10.0], [5.0, 0.0, 10.0]];
        let mut t = Townsfolk::new(6, [0.0, 0.0, 1.0]);
        t.patrol = Some(Patrol::new(pts, [0, 0, 0, 3, 3, 0, 0, 0]));
        let (mut f, mut r) = ([0.0, 0.0, 1.0], Rng::seeded(1));
        let far = [100.0, 0.0, -100.0];
        let at = |p: Vec3| View { position: p, hero: far, action: 8, progress: 0.0, fidget_entry: true, wander_fidget: true, arrived: false };
        assert_eq!(t.tick_on(&mut f, at([0.0; 3]), &mut r, Some(&mesh)), Some(8));
        assert_eq!(t.walk_to, Some([0.0, 0.0, 10.0]), "the first target is point 1");
        // Arriving (below 0.5): idle, no walk; three pause ticks, then on to point 2.
        assert_eq!(t.tick_on(&mut f, at([0.0, 0.0, 9.6]), &mut r, Some(&mesh)), Some(0));
        assert_eq!(t.walk_to, None);
        let mut walked = 0;
        for _ in 0..3 {
            t.tick_on(&mut f, at([0.0, 0.0, 9.6]), &mut r, Some(&mesh));
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
