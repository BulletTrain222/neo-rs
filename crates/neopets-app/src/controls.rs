//! Host input mapping and the resource-browser picker, kept free of ECS plumbing so they are
//! unit-testable. Which buttons the original reads, and their meaning, is not specified yet.
use bevy::prelude::*;
use neopets_sim::PlayerInput;

/// -1, 0, or 1 from a pair of opposing keys.
pub fn axis(negative: bool, positive: bool) -> f32 {
    positive as i32 as f32 - negative as i32 as f32
}

/// One stick axis through the PS2 pad's byte and the left-stick scaling (SPEC-0033 Behavior
/// 2): the byte 127.5 + 127.5 v, then (r - 167.5) / 87.5 above the dead zone, (r - 87.5) / 87.5
/// below it, and 0 for the bytes 88 to 167.
pub fn ps2_axis(v: f32) -> f32 {
    let r = (127.5 + 127.5 * v.clamp(-1.0, 1.0)).round();
    if r > 167.5 {
        (r - 167.5) / 87.5
    } else if r < 87.5 {
        (r - 87.5) / 87.5
    } else {
        0.0
    }
}

/// Movement from the keyboard direction and any gamepad sticks (each through the PS2 dead zone
/// and scale): the strongest source wins, scaled to the full i16 range. Buttons stay unmapped.
pub fn movement(keyboard: Vec2, sticks: impl IntoIterator<Item = Vec2>) -> PlayerInput {
    let mut best = keyboard;
    for stick in sticks {
        let stick = Vec2::new(ps2_axis(stick.x), ps2_axis(stick.y));
        if stick.length() > best.length() {
            best = stick;
        }
    }
    let scale = |v: f32| (v.clamp(-1.0, 1.0) * i16::MAX as f32) as i16;
    PlayerInput {
        buttons: 0,
        move_x: scale(best.x),
        move_y: scale(best.y),
    }
}

/// Resource number picked in the browser shell. Nothing is decoded until the archive specs
/// (R-0001, R-0002) are approved and implemented.
#[derive(Resource, Default, Debug, PartialEq, Eq)]
pub struct Browser {
    pub selected: u32,
    pub typing: Option<String>,
}

impl Browser {
    pub fn type_digit(&mut self, digit: char) {
        let typed = self.typing.get_or_insert_default();
        // Ten digits cover every u32; longer input could only overflow.
        if digit.is_ascii_digit() && typed.len() < 10 {
            typed.push(digit);
        }
    }

    pub fn backspace(&mut self) {
        if let Some(typed) = &mut self.typing {
            typed.pop();
        }
    }

    pub fn cancel(&mut self) {
        self.typing = None;
    }

    /// Jump to the typed number. Empty or out-of-range input is discarded.
    pub fn commit(&mut self) {
        if let Some(n) = self.typing.take().and_then(|t| t.parse().ok()) {
            self.selected = n;
        }
    }

    pub fn step(&mut self, delta: i64) {
        self.selected = (self.selected as i64 + delta).clamp(0, u32::MAX as i64) as u32;
    }

    /// Keep the selection inside an archive of `count` resources.
    pub fn clamp(&mut self, count: u32) {
        self.selected = self.selected.min(count.saturating_sub(1));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keyboard_axes_cancel_and_scale() {
        assert_eq!(axis(true, true), 0.0);
        assert_eq!(axis(false, true), 1.0);
        let input = movement(Vec2::new(axis(true, false), 1.0), []);
        assert_eq!((input.move_x, input.move_y), (-i16::MAX, i16::MAX));
        assert_eq!(input.buttons, 0);
    }

    #[test]
    fn ps2_stick_scaling() {
        // SPEC-0033 Behavior 2's bytes: 255 gives 1.0, 200 0.3714, 168 0.0057, 167 and 88 0,
        // 87 -0.0057, 40 -0.5429, 0 -1.0.
        let at = |r: f32| ps2_axis((r - 127.5) / 127.5);
        for (r, want) in [(255.0, 1.0), (200.0, 0.3714), (168.0, 0.0057), (167.0, 0.0), (88.0, 0.0), (87.0, -0.0057), (40.0, -0.5429), (0.0, -1.0)] {
            assert!((at(r) - want).abs() < 1e-4, "{r}: {}", at(r));
        }
    }

    #[test]
    fn strongest_stick_wins_and_is_clamped() {
        // A small push (0.2, byte 153) is inside the PS2 dead zone; -0.5 is byte 64.
        let input = movement(Vec2::ZERO, [Vec2::new(0.2, 0.0), Vec2::new(0.0, -0.5)]);
        assert_eq!((input.move_x, input.move_y), (0, ((64.0 - 87.5) / 87.5 * i16::MAX as f32) as i16));
        let input = movement(Vec2::new(0.0, 1.0), [Vec2::new(0.3, 0.3)]);
        assert_eq!(input.move_y, i16::MAX);
        let input = movement(Vec2::ZERO, [Vec2::new(4.0, -4.0)]);
        assert_eq!((input.move_x, input.move_y), (i16::MAX, -i16::MAX));
    }

    #[test]
    fn browser_typing_commits_cancels_and_bounds() {
        let mut b = Browser::default();
        for d in "0042".chars() {
            b.type_digit(d);
        }
        b.backspace();
        b.commit();
        assert_eq!((b.selected, b.typing.as_deref()), (4, None));
        b.type_digit('7');
        b.cancel();
        b.commit();
        assert_eq!(b.selected, 4);
        for d in "99999999999".chars() {
            b.type_digit(d);
        }
        assert_eq!(b.typing.as_deref(), Some("9999999999"));
        b.commit();
        assert_eq!(b.selected, 4, "values above u32::MAX are discarded");
        b.type_digit('x');
        assert_eq!(b.typing.as_deref(), Some(""));
        b.commit();
        assert_eq!(b.selected, 4);
    }

    #[test]
    fn browser_steps_saturate() {
        let mut b = Browser::default();
        b.step(-100);
        assert_eq!(b.selected, 0);
        b.step(101);
        assert_eq!(b.selected, 101);
        b.selected = u32::MAX - 1;
        b.step(100);
        assert_eq!(b.selected, u32::MAX);
        b.clamp(1434);
        assert_eq!(b.selected, 1433);
        b.clamp(0);
        assert_eq!(b.selected, 0);
    }
}
