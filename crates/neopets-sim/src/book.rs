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

/// The page layout words (boot data 0x2DB2B4 to 0x2DB2D4; SPEC-0035 6.1 rule 5): block x values
/// (20, 234), block y values (54, 256, 300), line limits (19, 7, 5) and the text width (192).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PageWords {
    pub x: [i32; 2],
    pub y: [i32; 3],
    pub limit: [i32; 3],
    pub width: i32,
}

impl PageWords {
    /// From the nine words at 0x2DB2B4.
    pub fn read(w: &[u32]) -> Option<Self> {
        let i = |k: usize| w.get(k).map(|&v| v as i32);
        Some(Self { x: [i(0)?, i(1)?], y: [i(2)?, i(3)?, i(4)?], limit: [i(5)?, i(6)?, i(7)?], width: i(8)? })
    }
}

pub const PAGE_WORDS: u32 = 0x002D_B2B4;
/// SPEC-0035 section 6.1 rule 14: the 23 style masks, by the book's style index.
pub const STYLE_MASKS: u32 = 0x0031_2D88;

/// The 23 template rows (rule 9): for blocks A and B, which x, y and line-limit word each uses
/// (None: the block is unused).
const ROWS: [[Option<(u8, u8, u8)>; 2]; 23] = [
    [Some((0, 0, 0)), Some((1, 0, 0))],
    [Some((0, 1, 1)), Some((1, 0, 0))],
    [Some((0, 0, 0)), Some((1, 1, 1))],
    [Some((0, 0, 1)), Some((1, 0, 0))],
    [Some((0, 0, 0)), Some((1, 0, 1))],
    [Some((0, 2, 2)), Some((1, 2, 2))],
    [Some((1, 0, 0)), None],
    [Some((0, 0, 0)), None],
    [Some((0, 1, 1)), Some((1, 1, 1))],
    [Some((0, 0, 1)), Some((1, 0, 1))],
    [Some((0, 1, 1)), Some((1, 0, 1))],
    [Some((0, 0, 1)), Some((1, 1, 1))],
    [Some((0, 1, 1)), Some((1, 0, 0))],
    [Some((0, 0, 0)), Some((1, 1, 1))],
    [Some((0, 0, 1)), Some((1, 0, 0))],
    [Some((0, 0, 0)), Some((1, 0, 1))],
    [Some((1, 0, 0)), None],
    [Some((0, 0, 0)), None],
    [Some((0, 1, 1)), Some((1, 1, 1))],
    [Some((0, 0, 1)), Some((1, 0, 1))],
    [Some((0, 1, 1)), Some((1, 0, 1))],
    [Some((0, 0, 1)), Some((1, 1, 1))],
    [Some((0, 0, 2)), Some((1, 0, 2))],
];

/// One laid-out line (rules 9 and 10): the body bytes start..end, the extra pixels on every
/// glyph's advance and on each break candidate, and the line's x and baseline y on the flat page.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Line {
    pub start: usize,
    pub end: usize,
    pub glyph_extra: f32,
    pub space_extra: f32,
    pub x: i32,
    pub y: i32,
}

fn candidate(c: u8) -> bool {
    c == b' ' || c == b'_'
}

/// Rule 10's line breaking at `width`: (start, end, justified extras) for every line of the body,
/// in order. `advance` gives a byte's glyph advance (None: no glyph, adds nothing).
pub fn wrap(body: &[u8], width: i32, advance: &dyn Fn(u8) -> Option<i32>) -> Vec<(usize, usize, f32, f32)> {
    let space = advance(b' ').unwrap_or(0) as f32;
    let mut out = Vec::new();
    let mut i = 0;
    while i < body.len() {
        let (mut w, mut last): (i32, Option<(usize, i32)>) = (0, None);
        let mut j = i;
        let mut done = None;
        while j < body.len() {
            let c = body[j];
            if c == b'\n' {
                done = Some((j, j + 1, false, w));
                break;
            }
            if candidate(c) {
                last = Some((j, w));
            }
            w += advance(c).unwrap_or(0);
            if w > width {
                done = Some(match last {
                    Some((k, wb)) => (k, k + 1, true, wb),
                    // No candidate: the overflowing byte stays in the line (SPEC-0035 rule 10).
                    None => (j + 1, j + 1, false, w),
                });
                break;
            }
            j += 1;
        }
        let (end, next, at_candidate, lw) = done.unwrap_or((body.len(), body.len(), false, w));
        // Justify a line that ended at a candidate and holds at least one candidate inside it.
        let inside = body[i..end].iter().filter(|&&c| candidate(c)).count();
        let (e, s) = if at_candidate && inside > 0 {
            let l = (width - lw) as f32;
            let n = (end - i).max(1) as f32;
            let e = (l / n).min(2.0);
            let rest = l - e * n;
            (e, if rest != 0.0 { (rest / inside as f32).min(2.5 * space) } else { 0.0 })
        } else {
            (0.0, 0.0)
        };
        out.push((i, end, e, s));
        i = next;
    }
    out
}

/// Rules 9 and 10: the lines of a book's body on its flat page by its style's template row,
/// block A first, then block B; lines past both limits are not drawn.
pub fn layout(body: &[u8], style: u32, w: &PageWords, pitch: i32, advance: &dyn Fn(u8) -> Option<i32>) -> Vec<Line> {
    let lines = wrap(body, w.width, advance);
    let row = ROWS.get(style as usize).copied().unwrap_or(ROWS[0]);
    let mut out = Vec::new();
    let mut it = lines.into_iter();
    for (xi, yi, li) in row.into_iter().flatten() {
        let (x, y, limit) = (w.x[xi as usize], w.y[yi as usize], w.limit[li as usize]);
        for j in 0..limit.max(0) {
            let Some((start, end, e, s)) = it.next() else { return out };
            out.push(Line { start, end, glyph_extra: e, space_extra: s, x, y: y + pitch * j });
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    const WORDS: PageWords = PageWords { x: [20, 234], y: [54, 256, 300], limit: [19, 7, 5], width: 192 };

    #[test]
    fn wrap_breaks_and_justifies() {
        // Every byte 10 wide, the space 5: "aaaaaaaaaa aaaaaaaaaa ..." breaks at the last space
        // before the width passes 192.
        let adv = |c: u8| Some(if c == b' ' { 5 } else { 10 });
        let body = b"aaaaaaaaaa aaaaaaaa bbb\n\nccc";
        let l = wrap(body, 192, &adv);
        // Line 0: 10 + 1 + 8 bytes = 185 wide, then " bbb" passes 192 at the second b: it breaks
        // at the space after the 8 a's (width 185), which is consumed.
        assert_eq!((l[0].0, l[0].1), (0, 19));
        // Justified: L = 7, n = 19, e = 7 / 19; no remainder.
        assert!((l[0].2 - 7.0 / 19.0).abs() < 1e-6 && l[0].3.abs() < 1e-4);
        // "bbb" ends at a newline (not stretched), the blank line, then "ccc".
        assert_eq!(l[1..].iter().map(|x| (x.0, x.1, x.2)).collect::<Vec<_>>(), [(20, 23, 0.0), (24, 24, 0.0), (25, 28, 0.0)]);
        // No candidate: the overflowing byte stays in the line.
        let long = [b'x'; 25];
        let l = wrap(&long, 192, &adv);
        assert_eq!((l[0].0, l[0].1, l[1].0), (0, 20, 20));
    }

    #[test]
    fn template_rows_place_blocks() {
        // Style 12: block A at x 20, y 256, 7 lines; block B at x 234, y 54, 19 lines.
        let body: Vec<u8> = (0..30).flat_map(|_| b"a\n".iter().copied()).collect();
        let adv = |_| Some(5);
        let l = layout(&body, 12, &WORDS, 18, &adv);
        assert_eq!(l.len(), 26, "7 + 19 lines fit; the rest are not drawn");
        assert_eq!((l[0].x, l[0].y, l[6].y), (20, 256, 256 + 6 * 18));
        assert_eq!((l[7].x, l[7].y, l[25].y), (234, 54, 54 + 18 * 18));
        // Style 6: only block B, on the right page.
        let l = layout(&body, 6, &WORDS, 18, &adv);
        assert_eq!((l.len(), l[0].x), (19, 234));
    }

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
