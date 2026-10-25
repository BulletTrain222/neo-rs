//! Townsfolk (behavior class 6) per-tick routine (SPEC-0056 Townsfolk routine, Modes, Greeting,
//! Mode from code, Animation ids by mode): the greeting entry pre-step, the mode set from the
//! behavior code, and the bodies of modes 0, 10, 11, 12 and 13.
//! Not covered: wander (5) and patrol (6) need the navigation mesh, its route table and the
//! game's random stream; in this build they stand idle. The villager pair, script walks,
//! navigation index updates and base states other than the conversation are not covered.

use crate::hero::{Vec3, steer};

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
}

/// What the routine reads of the world this tick.
#[derive(Clone, Copy, Debug)]
pub struct View {
    pub position: Vec3,
    pub hero: Vec3,
    /// The animation id playing and its clip progress.
    pub action: u32,
    pub progress: f32,
}

fn horizontal(a: Vec3, b: Vec3) -> (f32, Vec3) {
    let d = [b[0] - a[0], 0.0, b[2] - a[2]];
    let len = (d[0] * d[0] + d[2] * d[2]).sqrt();
    let unit = if len > 0.0 { d.map(|c| c / len) } else { d };
    (len, unit)
}

impl Townsfolk {
    pub fn new(code: i16, home_facing: Vec3) -> Self {
        Self { code, mode: 0, home_facing, counter: 0, done: false, pose: false }
    }

    /// One tick: returns the animation id to request (None: keep what is playing), and turns
    /// `facing` where the mode says so.
    pub fn tick(&mut self, facing: &mut Vec3, v: View) -> Option<u32> {
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
            // Wander and patrol are not implemented: stand.
            5 | 6 => Some(0),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn view(hero: Vec3, action: u32, progress: f32) -> View {
        View { position: [0.0; 3], hero, action, progress }
    }

    #[test]
    fn greeting_waits_180_ticks_then_plays_30_once() {
        let mut t = Townsfolk::new(0, [0.0, 0.0, 1.0]);
        let mut f = [0.0, 0.0, 1.0];
        // Hero 4 ahead: the greeting starts and restarts each tick, so it idles while the hero
        // stays; the counter only runs once the hero has left the 6.0 entry range but is
        // still within 12.0 and in front.
        assert_eq!(t.tick(&mut f, view([0.0, 0.0, 4.0], 0, 0.0)), Some(0));
        assert_eq!(t.mode, 13);
        let far = view([0.0, 0.0, 8.0], 0, 0.0);
        let mut ids = Vec::new();
        for _ in 0..181 {
            ids.push(t.tick(&mut f, far));
        }
        assert!(ids[..179].iter().all(|&i| i == Some(0)));
        assert_eq!(ids[180], Some(30));
        assert_eq!(t.tick(&mut f, view([0.0, 0.0, 8.0], 30, 0.995)), Some(0));
        // Beyond 12.0 the greeting ends; code 0 returns to mode 0.
        assert_eq!(t.tick(&mut f, view([0.0, 0.0, 13.0], 0, 0.0)), None);
        assert_eq!(t.mode, 0);
    }

    #[test]
    fn idle_code_0_turns_back_to_its_placement_facing() {
        let mut t = Townsfolk::new(0, [1.0, 0.0, 0.0]);
        let mut f = [0.0, 0.0, 1.0];
        let behind = view([0.0, 0.0, -20.0], 0, 0.0);
        t.tick(&mut f, behind);
        assert!(f[0] > 0.0 && f[0] < 0.1, "a 0.05 blend step: {f:?}");
        for _ in 0..200 {
            t.tick(&mut f, behind);
        }
        assert!(f[0] > 0.99);
    }

    #[test]
    fn poses_by_code_and_no_greeting_in_them() {
        // The greeting entry runs before mode-from-code, so start with the hero away.
        let mut t = Townsfolk::new(10, [0.0, 0.0, 1.0]);
        let mut f = [0.0, 0.0, 1.0];
        assert_eq!(t.tick(&mut f, view([0.0, 0.0, -30.0], 0, 0.0)), Some(65));
        assert_eq!(t.tick(&mut f, view([0.0, 0.0, 3.0], 65, 0.0)), Some(65), "no greeting in mode 10");
        assert_eq!(t.mode, 10);
    }
}
