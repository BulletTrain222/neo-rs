//! Books (SPEC-0035 section 6.1; SPEC-0028 Book station; SPEC-0022 the chooser's Read Book):
//! the station test, the pending book index, the book cursor and the saved book word, the
//! chooser's book step, the reader's index rule and the page timer.

/// A book station: a kind-3 volume whose flags have bit 6 set and bits 1, 2, 3 and 8 clear.
pub fn is_station(kind: u8, flags: u16) -> bool {
    kind == 3 && flags & 0x0040 != 0 && flags & 0x010E == 0
}

/// The pending book index (game-state +0x50), the 1-based book cursor (+0x54) and the saved
/// book word (+0xA8 of the settings block); -1 means none for each.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Books {
    pub pending: i32,
    pub cursor: i32,
    pub saved: i32,
}

impl Default for Books {
    fn default() -> Self {
        Self { pending: -1, cursor: -1, saved: -1 }
    }
}

impl Books {
    /// The chooser's book step: while a book is pending and none is open, the "Read Book"
    /// prompt shows and the pending index is cleared; a Square edge first copies it to the
    /// cursor. Returns (prompt shown, book taken).
    pub fn step(&mut self, square: bool) -> (bool, bool) {
        if self.pending < 0 || self.cursor >= 0 {
            return (false, false);
        }
        if square {
            self.cursor = self.pending;
        }
        self.pending = -1;
        (true, square)
    }

    /// Play's book gate: the cursor or the saved word is 0 or more.
    pub fn gate(&self) -> bool {
        self.cursor >= 0 || self.saved >= 0
    }

    /// The reader's index: cursor - 1, else saved - 1; none when both are negative.
    pub fn index(&self) -> Option<usize> {
        let i = if self.cursor - 1 >= 0 { self.cursor - 1 } else { self.saved - 1 };
        usize::try_from(i).ok()
    }
}

/// The page timer (the page angle in radians): from the start word down by (start - min) / 75
/// per page-driver call, clamped at the minimum.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PageTimer {
    pub angle: f32,
    step: f32,
    min: f32,
}

impl PageTimer {
    pub fn new(start: f32, min: f32) -> Self {
        Self { angle: start, step: (start - min) * (1.0 / 75.0), min }
    }

    pub fn tick(&mut self) {
        self.angle = (self.angle - self.step).max(self.min);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stations_steps_and_timer() {
        assert!(is_station(3, 0x0040) && is_station(3, 0x0041));
        assert!(!is_station(3, 0x0048) && !is_station(3, 0x0140) && !is_station(1, 0x0040));
        // A value-82 station: the prompt clears pending; a Square edge sets the cursor first.
        let mut b = Books { pending: 82, ..Default::default() };
        assert_eq!(b.step(false), (true, false));
        assert_eq!((b.pending, b.cursor), (-1, -1));
        b.pending = 82;
        assert_eq!(b.step(true), (true, true));
        assert_eq!((b.pending, b.cursor, b.gate(), b.index()), (-1, 82, true, Some(81)));
        // With a book open the step does nothing.
        b.pending = 5;
        assert_eq!(b.step(true), (false, false));
        // Cursor 0 with the saved word -1: no index.
        let b = Books { cursor: 0, ..Default::default() };
        assert_eq!((b.gate(), b.index()), (true, None));
        // The timer: 1.2, 1.1856 after one call, 0.12 after 75 and after 76.
        let mut t = PageTimer::new(1.2, 0.12);
        t.tick();
        assert!((t.angle - 1.1856).abs() < 1e-5);
        for _ in 0..74 {
            t.tick();
        }
        assert!((t.angle - 0.12).abs() < 1e-5);
        t.tick();
        assert_eq!(t.angle, 0.12);
    }
}
