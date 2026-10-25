//! The shared dialogue box (SPEC-0022 Reveal and paging, Choices input and release, Speaker
//! name and colour rule 5): pages split at type-4 stops, the reveal at 10 characters per
//! progress unit with a progress step of 1/30 per update (x 5 while Cross is held), control
//! codes run once as the reveal reaches them and take no time, the Square skip, and the Square
//! advance ("Next") or close ("Finish") once a page is complete; choice mode (Circle answers 1,
//! Triangle 0, on the complete last page) and the substitution codes (Substitution codes).
//!
//! Not covered: the instant flag, the box owner's action beyond the override clear, the
//! same-text reopen, and the emote, give-items and progress-flag codes (they are stepped over).

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
    /// Choice mode, and the result: -1 after a new text, 1 (Circle) or 0 (Triangle) after a
    /// choice.
    pub choice: bool,
    pub result: i32,
    /// Emote requests the reveal reached (kind, emote id), for the caller to take.
    pub emotes: Vec<(u8, i32)>,
}

/// Replace each substitution code (run-pass mask V - 2 with bit 5 set) by the text of slot k,
/// k being its first argument; other codes are stepped over whole (Substitution codes).
pub fn substitute(text: &[u8], slots: &[Vec<u8>]) -> Vec<u8> {
    let mut out = Vec::with_capacity(text.len());
    let mut i = 0;
    while i < text.len() {
        match code(text, i) {
            Some((len, _)) => {
                let v = text[i + 4];
                if v >= 2 && (v - 2) & 32 != 0 {
                    let k = decode_args(&text[i + 5..i + len]).first().copied().unwrap_or(0) as usize;
                    out.extend_from_slice(slots.get(k).map_or(&[][..], |s| &s[..]));
                } else {
                    out.extend_from_slice(&text[i..i + len]);
                }
                i += len;
            }
            None => {
                out.push(text[i]);
                i += 1;
            }
        }
    }
    out
}

/// The escape rule for argument bytes: 0xFD then 0x40 to 0x60 is the value b - 0x40, 0xFD then
/// 0x02 is 0xFD, 0xFD then any other byte is 0; other bytes stand for themselves.
fn decode_args(a: &[u8]) -> Vec<u8> {
    let mut out = Vec::new();
    let mut i = 0;
    while i < a.len() {
        if a[i] == 0xFD {
            let b = a.get(i + 1).copied().unwrap_or(0);
            out.push(match b {
                0x40..=0x60 => b - 0x40,
                0x02 => 0xFD,
                _ => 0,
            });
            i += 2;
        } else {
            out.push(a[i]);
            i += 1;
        }
    }
    out
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
        *self = Self { text: text.to_vec(), open: true, result: -1, ..Self::default() };
        self.page_from(0);
    }

    /// Open a text in choice mode (LevelPrompt and the card questions).
    pub fn open_choice(&mut self, text: &[u8]) {
        self.open(text);
        self.choice = true;
    }

    /// Whether the box waits for a choice now: choice mode on the complete last page.
    pub fn asking(&self) -> bool {
        self.open && self.choice && self.next.is_none() && self.complete()
    }

    /// Choices, input and release, item 3: on the complete last page in choice mode, Circle
    /// sets the result to 1 and Triangle to 0 (Circle wins), and the box closes.
    pub fn choose(&mut self, circle: bool, triangle: bool) -> Option<i32> {
        if !self.asking() || !(circle || triangle) {
            return None;
        }
        self.result = circle as i32;
        self.open = false;
        Some(self.result)
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
                    self.run(t, &page[i + 5..i + len]);
                    i += len;
                }
                None => i += 1,
            }
        }
        let mut n = to.max(i);
        while let Some((len, t)) = code(&page, n) {
            self.run(t, &page[n + 5..n + len]);
            n += len;
        }
        n
    }

    fn run(&mut self, t: u8, args: &[u8]) {
        match t {
            32 => self.override_hero = true,
            34 => self.override_hero = false,
            // Emote requests (SPEC-0022 Dialogue control records): the first argument group.
            0x10 | 0x16 | 0x17 | 0x18 => {
                let a = decode_args(args);
                if a.len() >= 4 {
                    self.emotes.push((t, i32::from_le_bytes([a[0], a[1], a[2], a[3]])));
                }
            }
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
            // In choice mode the last page has no "Finish": only Circle or Triangle closes it.
            if square && !(self.choice && self.next.is_none()) {
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
    fn substitution_and_choice() {
        // "slot ", a substitution code (n 3, L 13, type 5, V 34, four escape pairs for 0), ".".
        let code = |k: u8| {
            let mut c = vec![0x01, 3, 13, 5, 34];
            for v in [k, 0, 0, 0] {
                c.extend([0xFD, 0x40 + v]);
            }
            c
        };
        let mut t = b"slot ".to_vec();
        t.extend(code(0));
        t.push(b'.');
        let slots = vec![b"1".to_vec(), b"96".to_vec()];
        assert_eq!(substitute(&t, &slots), b"slot 1.");
        let mut two = code(1);
        two.extend(b" KB, ");
        two.extend(code(0));
        assert_eq!(substitute(&two, &slots), b"96 KB, 1");
        // A colour code is kept whole.
        let colour = [0x01, 3, 9, 2, 3, 1, 2, 3, 4];
        assert_eq!(substitute(&colour, &slots), colour);
        // Choice mode: Square does not close the last page; Circle answers 1, Circle wins.
        let mut b = TextBox::default();
        b.open_choice(b"Overwrite?");
        assert_eq!(b.result, -1);
        assert_eq!(b.choose(true, false), None, "not complete yet");
        while !b.complete() {
            b.update(true, false);
        }
        assert_eq!(b.update(false, true), None);
        assert!(b.open && b.asking());
        assert_eq!(b.choose(true, true), Some(1));
        assert!(!b.open);
        b.open_choice(b"Cancel save?");
        while !b.complete() {
            b.update(true, false);
        }
        assert_eq!(b.choose(false, true), Some(0));
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

    #[test]
    fn emote_requests_run_when_the_reveal_reaches_them() {
        // "Hi" then a partner body emote with id 51 (one group: 0x33 0 0 0, zeros escaped).
        let mut t = b"Hi".to_vec();
        t.extend([0x01, 3, 13, 0x10, 4, 0x33, 0xFD, 0x40, 0xFD, 0x40, 0xFD, 0x40]);
        t.extend(b"!");
        let mut b = TextBox::default();
        b.open(&t);
        assert!(b.emotes.is_empty());
        while !b.complete() {
            b.update(false, false);
        }
        assert_eq!(b.emotes, vec![(0x10, 51)]);
    }
}
