//! The credits roll (SPEC-0026 Credits; SPEC-0024 table 15): rows from the top-row index at y,
//! a pitch of 24, edge fades over 32 px, a speed easing by 1/512 per pass toward its target (1.5
//! after a style-5 row is walked, 1.0 after a style-6 row), y falling by the speed, and at most
//! one top row dropped per pass once it is above -20. Float32 throughout, as the spec's timing.

/// The roll's tuning words (boot 0x2DCF84 to 0x2DCFA0): pitch, slow and fast speeds, the speed
/// ramp, the fade distances, the drop threshold and the start offset.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Words {
    pub pitch: f32,
    pub slow: f32,
    pub fast: f32,
    pub ramp: f32,
    pub fade_top: i32,
    pub fade_bottom: i32,
    pub drop: i32,
    pub offset: i32,
}

/// The roll state.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Roll {
    pub top: usize,
    pub y: f32,
    pub speed: f32,
    pub target: f32,
}

/// A row drawn this pass: its index, y and alpha (0 to 128).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Shown {
    pub row: usize,
    pub y: f32,
    pub alpha: i32,
}

impl Roll {
    /// Pass 0: top row 0 at y = H + offset, speed and target slow.
    pub fn new(h: i32, w: &Words) -> Self {
        Self { top: 0, y: (h + w.offset) as f32, speed: w.slow, target: w.slow }
    }

    pub fn ended(&self, rows: usize) -> bool {
        self.top >= rows
    }

    /// One pass over `styles` (one per row; None for a row with no text): the rows to draw, then
    /// the speed, the scroll and the drop.
    pub fn step(&mut self, styles: &[(bool, u16)], h: i32, w: &Words) -> Vec<Shown> {
        let mut shown = Vec::new();
        let mut y = self.y;
        for (k, &(text, style)) in styles.iter().enumerate().skip(self.top) {
            let a = if y < w.fade_top as f32 {
                128.0 * y / w.fade_top as f32
            } else if y > (h - w.fade_bottom) as f32 {
                128.0 * (h as f32 - y) / w.fade_bottom as f32
            } else {
                128.0
            };
            let alpha = (a as i32).clamp(0, 128);
            if text {
                shown.push(Shown { row: k, y, alpha });
            }
            match style {
                5 => self.target = w.fast,
                6 => self.target = w.slow,
                _ => {}
            }
            if y > h as f32 {
                break;
            }
            y += w.pitch;
        }
        if self.speed < self.target {
            self.speed = (self.speed + w.ramp).min(self.target);
        } else if self.speed > self.target {
            self.speed = (self.speed - w.ramp).max(self.target);
        }
        self.y -= self.speed;
        if self.y < w.drop as f32 {
            self.y += w.pitch;
            self.top += 1;
        }
        shown
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spec_timing() {
        let w = Words { pitch: 24.0, slow: 1.0, fast: 1.5, ramp: 1.0 / 512.0, fade_top: 32, fade_bottom: 32, drop: -20, offset: 1 };
        // 623 rows: style 5 at rows 214 and 524, style 6 at 335 and 557, text elsewhere.
        let styles: Vec<(bool, u16)> = (0..623)
            .map(|k| match k {
                214 | 524 => (false, 5),
                335 | 557 => (false, 6),
                _ => (true, 3),
            })
            .collect();
        let h = 448;
        let mut r = Roll::new(h, &w);
        assert_eq!(r.y, 449.0);
        let (mut first_drop, mut changes, mut pass) = (None, Vec::new(), 0u32);
        let mut target = r.target;
        while !r.ended(styles.len()) {
            let before = r.top;
            r.step(&styles, h, &w);
            if r.top > before && first_drop.is_none() {
                first_drop = Some(pass);
            }
            if r.target != target {
                changes.push(pass);
                target = r.target;
            }
            pass += 1;
        }
        assert_eq!(first_drop, Some(469));
        assert_eq!(changes, vec![5113, 7092, 11564, 12135]);
        assert_eq!(pass, 14123, "the last row drops on pass 14,122");
        // Alpha: a row at y 16 is 64; at H - 8 it is 32.
        let mut r = Roll { top: 0, y: 16.0, speed: 1.0, target: 1.0 };
        let shown = r.step(&[(true, 3)], h, &w);
        assert_eq!(shown[0].alpha, 64);
        let mut r = Roll { top: 0, y: (h - 8) as f32, speed: 1.0, target: 1.0 };
        assert_eq!(r.step(&[(true, 3)], h, &w)[0].alpha, 32);
    }
}
