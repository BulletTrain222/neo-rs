//! The shared dialogue box (SPEC-0022 Reveal and paging, Choices input and release, Speaker
//! name and colour rule 5): pages split at type-4 stops, the reveal at 10 characters per
//! progress unit with a progress step of 1/30 per update (x 5 while Cross is held), control
//! codes run once as the reveal reaches them and take no time, the Square skip, and the Square
//! advance ("Next") or close ("Finish") once a page is complete.
//!
//! Not covered: choices (LevelPrompt), substitution codes, the instant flag, the box owner's
//! action beyond the override clear, and the emote, give-items and progress-flag codes (they are
//! stepped over).

const RATE: f32 = 10.0;
const STEP: f32 = 1.0 / 30.0;

/// What an update did.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BoxEvent {
    /// Square on a complete page with a stop pending: the next page (cue 0x98).
    Advanced,
    /// Square on the complete last page: the box closes.
    Closed,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct TextBox {
    pub text: Vec<u8>,
    /// The page: bytes [start, end); `next` is where the following page starts (after a stop).
    pub start: usize,
    pub end: usize,
    next: Option<usize>,
    pub p: f32,
    pub open: bool,
    /// The speaker override (type 32 sets it, type 34 clears it).
    pub override_hero: bool,
}

/// The length of a control code at `i` (0 when the byte is not a marker), and its type.
fn code(text: &[u8], i: usize) -> Option<(usize, u8)> {
    if text.get(i) != Some(&0x01) {
        return None;
    }
    let len = *text.get(i + 2)? as usize;
    (len >= 5 && i + len <= text.len()).then(|| (len, text[i + 3]))
}

impl TextBox {
    /// Open a text at its first page.
    pub fn open(&mut self, text: &[u8]) {
        *self = Self { text: text.to_vec(), open: true, ..Self::default() };
        self.page_from(0);
    }

    fn page_from(&mut self, start: usize) {
        self.start = start;
        self.p = 0.0;
        let mut i = start;
        while i < self.text.len() {
            match code(&self.text, i) {
                Some((len, 4)) => {
                    self.end = i;
                    self.next = Some(i + len);
                    return;
                }
                Some((len, _)) => i += len,
                None if self.text[i] == 0x01 => break,
                None => i += 1,
            }
        }
        self.end = i;
        self.next = None;
    }

    /// The page bytes.
    pub fn page(&self) -> &[u8] {
        &self.text[self.start..self.end]
    }

    /// The visible index n within the page.
    pub fn visible(&self) -> usize {
        (self.p * RATE + 0.001).floor() as usize
    }

    pub fn complete(&self) -> bool {
        self.visible() >= self.page().len()
    }

    /// A stop is pending (the prompt shows "Next"; otherwise "Finish").
    pub fn more(&self) -> bool {
        self.next.is_some()
    }

    /// Run the codes that start in [from, to) of the page, then those at the frontier, which are
    /// stepped over; returns the new frontier.
    fn run_codes(&mut self, from: usize, to: usize) -> usize {
        let page = self.page().to_vec();
        let mut i = from;
        while i < to.min(page.len()) {
            match code(&page, i) {
                Some((len, t)) => {
                    self.run(t);
                    i += len;
                }
                None => i += 1,
            }
        }
        let mut n = to.max(i);
        while let Some((len, t)) = code(&page, n) {
            self.run(t);
            n += len;
        }
        n
    }

    fn run(&mut self, t: u8) {
        match t {
            32 => self.override_hero = true,
            34 => self.override_hero = false,
            _ => {}
        }
    }

    /// One box update: input is read only when the page was already complete; otherwise the
    /// skip, then the progress step and the code runs.
    pub fn update(&mut self, cross_held: bool, square: bool) -> Option<BoxEvent> {
        if !self.open {
            return None;
        }
        if self.complete() {
            if square {
                return Some(match self.next {
                    Some(k) => {
                        self.override_hero = false;
                        self.page_from(k);
                        BoxEvent::Advanced
                    }
                    None => {
                        self.open = false;
                        BoxEvent::Closed
                    }
                });
            }
            return None;
        }
        let m = self.visible();
        if square && self.p > 0.0 {
            self.p = self.page().len() as f32 / RATE;
        } else {
            self.p += STEP * if cross_held { 5.0 } else { 1.0 };
        }
        let n = self.run_codes(m, self.visible());
        self.p = self.p.max(n as f32 / RATE);
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn five_characters_reveal_in_sixteen_updates() {
        // SPEC-0022 timing: a page of 5 plain characters is complete in update 15 (the 16th):
        // `complete()` after update i is what update i + 1 starts with.
        let mut b = TextBox::default();
        b.open(b"Hello");
        let n = (0..40).position(|_| {
            b.update(false, false);
            b.complete()
        });
        assert_eq!(n.map(|i| i + 1), Some(15));
        // With Cross held from the start it is complete in update 3.
        b.open(b"Hello");
        let n = (0..40).position(|_| {
            b.update(true, false);
            b.complete()
        });
        assert_eq!(n.map(|i| i + 1), Some(3));
    }

    #[test]
    fn pages_skip_advance_and_close() {
        let mut text = b"ab".to_vec();
        text.extend([0x01, 2, 5, 4, 6]);
        text.extend(b"cd");
        let mut b = TextBox::default();
        b.open(&text);
        assert_eq!(b.page(), b"ab");
        assert!(b.more());
        // The first update cannot skip (p is 0); the second Square press skips.
        b.update(false, true);
        assert!(!b.complete());
        b.update(false, true);
        assert!(b.complete());
        assert_eq!(b.update(false, true), Some(BoxEvent::Advanced));
        assert_eq!(b.page(), b"cd");
        assert!(!b.more());
        b.update(false, false);
        b.update(false, true);
        assert_eq!(b.update(false, true), Some(BoxEvent::Closed));
        assert!(!b.open);
    }

    #[test]
    fn codes_take_no_time_and_set_the_override() {
        let mut text = vec![0x01, 2, 5, 32, 4];
        text.extend(b"Hi");
        let mut b = TextBox::default();
        b.open(&text);
        b.update(false, false);
        assert!(b.override_hero, "the type-32 code at the start runs at once");
        assert!(b.visible() >= 5, "and is stepped over");
    }
}
