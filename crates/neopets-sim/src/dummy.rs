//! The Target Dummy, behaviour class 36 (SPEC-0029 Behavior 29; SPEC-0038 Behavior 28): it never
//! attacks and cannot die. A hit plays animation 3, restores its hit points, sets the twitch
//! counter to 1 and draws a new interval in [400, 800] ticks. Every other tick, except while
//! animation 3 is still playing, the counter goes up by 1; a multiple of the interval plays the
//! twitch (1), and otherwise the idle (0) unless the twitch is still playing.

use crate::rng::Rng;

const INTERVAL: (i32, i32) = (400, 800);

/// The clip the routine looks at: the current animation id and whether it has finished.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Clip {
    pub action: u32,
    pub done: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Dummy {
    counter: i32,
    interval: i32,
}

impl Dummy {
    pub fn new(rng: &mut Rng) -> Self {
        Self { counter: 1, interval: rng.range(INTERVAL.0, INTERVAL.1) }
    }

    /// One tick: the animation to request, if any. `hit` is the pending hit flag (consumed here);
    /// the host restores the hit points on it.
    pub fn tick(&mut self, hit: bool, clip: Clip, rng: &mut Rng) -> Option<u32> {
        if hit {
            self.counter = 1;
            self.interval = rng.range(INTERVAL.0, INTERVAL.1);
            return Some(3);
        }
        if clip.action == 3 && !clip.done {
            return None;
        }
        let twitch = self.counter % self.interval == 0;
        self.counter += 1;
        if twitch {
            Some(1)
        } else if clip.action != 1 || clip.done {
            Some(0)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const IDLE: Clip = Clip { action: 0, done: false };

    #[test]
    fn twitches_at_each_multiple_and_idles_between() {
        let mut r = Rng::seeded(1);
        let mut d = Dummy::new(&mut r);
        assert!((400..=800).contains(&d.interval));
        d.interval = 3;
        // Counter 1, 2: idle; 3: the twitch; while it plays nothing is requested.
        assert_eq!(d.tick(false, IDLE, &mut r), Some(0));
        assert_eq!(d.tick(false, IDLE, &mut r), Some(0));
        assert_eq!(d.tick(false, IDLE, &mut r), Some(1));
        assert_eq!(d.tick(false, Clip { action: 1, done: false }, &mut r), None);
        assert_eq!(d.counter, 5, "the twitch still counts");
        assert_eq!(d.tick(false, Clip { action: 1, done: true }, &mut r), Some(0));
    }

    #[test]
    fn a_hit_resets_and_its_clip_stops_the_count() {
        let mut r = Rng::seeded(2);
        let mut d = Dummy::new(&mut r);
        d.counter = 50;
        assert_eq!(d.tick(true, IDLE, &mut r), Some(3));
        assert_eq!(d.counter, 1);
        assert!((400..=800).contains(&d.interval));
        assert_eq!(d.tick(false, Clip { action: 3, done: false }, &mut r), None);
        assert_eq!(d.counter, 1, "no count while animation 3 plays");
        assert_eq!(d.tick(false, Clip { action: 3, done: true }, &mut r), Some(0));
        assert_eq!(d.counter, 2);
    }
}
