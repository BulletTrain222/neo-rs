//! The movie skip rule (SPEC-0026 Movie skip). The check runs once per presented movie frame.
//! The dead second counter (newly pressed buttons, threshold 3) is not built, as the spec says.

/// Skip state for one movie; reset at the start of each movie.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Skip {
    armed: bool,
    count: u8,
    latched: bool,
}

/// Held frames that latch the skip.
pub const SKIP_FRAMES: u8 = 5;

impl Skip {
    /// One presented frame, with whether Cross or Start is held on the combined pads. Returns
    /// true when the movie ends now: on the check after the latching frame, so one more frame is
    /// shown after the latch. The pad is not read once latched, and nothing resets on release.
    pub fn frame(&mut self, held: bool) -> bool {
        if self.latched {
            return true;
        }
        if !self.armed {
            self.armed = !held;
            return false;
        }
        if held {
            self.count += 1;
            self.latched = self.count >= SKIP_FRAMES;
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn movie_skip_rule() {
        // Held from the start: never armed, never skips.
        let mut s = Skip::default();
        assert!((0..20).all(|_| !s.frame(true)));
        // Armed by one free frame; five held frames, not consecutive, latch on the fifth; the
        // next check ends the movie.
        let mut s = Skip::default();
        assert!(!s.frame(false));
        let held = [true, true, false, true, true, false, true];
        assert!(held.iter().all(|&h| !s.frame(h)));
        assert!(s.frame(false));
    }
}
