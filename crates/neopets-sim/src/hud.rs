//! The in-play HUD's bar rules (SPEC-0023 Behavior 1 to 4): the change detector C and the
//! group timers G0 (bars 0x31) and G1 (quick-item panel 0x30), the bar update with its watched
//! values, the bar widths L = trunc(max x K) and F = trunc(current x K), the frame layout, the
//! upgrade dividers, alpha(t), and the red and green tint pulses of the health frame (0x3B).
//!
//! The monster health bar (section 10): the arm with its 600-tick timer, the zero-health cut,
//! the frame width from the maximum and the name width, the fill width.
//!
//! Not covered: the equipment panels and their timers G2 to G4, the D-pad hint, the loss
//! effects, the petpet meter, the journal icon, the portrait swing and the countdown texts.

/// Bar scale K (float32 0.6666667).
pub const K: f32 = 0.666_666_7;

/// alpha(t): 128 when t < 0 or t >= 30, else trunc(t x 4.266667).
pub fn alpha(t: i32) -> u8 {
    if !(0..30).contains(&t) { 128 } else { (t as f32 * 4.266_667) as u8 }
}

/// Bar width in pixels for a value.
pub fn width(v: i32) -> i32 {
    (v as f32 * K) as i32
}

/// Boot words the HUD reads (SPEC-0023 Layout): the pulse half-length, the red and green tints,
/// the right-cap offset A and frame inset B, and the divider bases and steps of the active hero
/// (health, pool).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Words {
    pub half: f32,
    pub red: [u8; 4],
    pub green: [u8; 4],
    pub cap: i32,
    pub inset: i32,
    pub base: (i32, i32),
    pub step: (i32, i32),
}

/// The values the bar update watches, and whether the hero is poisoned.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Values {
    pub hp: i32,
    pub hp_max: i32,
    pub mp: i32,
    pub mp_max: i32,
    pub poisoned: bool,
}

/// One bar's layout relative to its bar origin: the frame middle's width, the right cap's x,
/// the fill width and the number of dividers shown.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Bar {
    pub middle: i32,
    pub cap_x: i32,
    pub fill: i32,
    pub dividers: i32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct Pulse {
    target: [u8; 4],
    t: f32,
    rising: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Hud {
    pub c: i32,
    pub g0: i32,
    pub g1: i32,
    watched: [i32; 4],
    seen: Option<Values>,
    pulse: Option<Pulse>,
    /// The health frame's colour (element 0x3B), the alpha last pushed to group 0x31, and the
    /// bars' layout.
    pub tint: [u8; 4],
    pub bars_alpha: u8,
    pub health: Bar,
    pub pool: Bar,
}

impl Default for Hud {
    fn default() -> Self {
        // The full reset after a load: all timers 0, C = -1.
        Self { c: -1, g0: 0, g1: 0, watched: [-1; 4], seen: None, pulse: None, tint: [128; 4], bars_alpha: 128, health: Bar::default(), pool: Bar::default() }
    }
}

impl Hud {
    /// Group 0x31 is drawn while G0 is not 0; its health bar group while C is not 0; the pool
    /// bar group also needs a pool maximum above 0.
    pub fn bars_drawn(&self) -> bool {
        self.g0 != 0
    }

    pub fn health_drawn(&self) -> bool {
        self.c != 0
    }

    pub fn pool_drawn(&self, v: &Values) -> bool {
        self.c != 0 && v.mp_max > 0
    }

    fn start_pulse(&mut self, target: [u8; 4], w: &Words) {
        self.pulse = Some(Pulse { target, t: 0.0, rising: true });
        self.advance_pulse(w);
    }

    fn advance_pulse(&mut self, w: &Words) {
        let Some(p) = self.pulse.as_mut() else { return };
        let step = 1.0 / w.half;
        if p.rising {
            p.t += step;
            if p.t >= 1.0 {
                p.t = 1.0;
                p.rising = false;
            }
        } else {
            p.t -= step;
        }
        let t = p.t.max(0.0);
        self.tint = std::array::from_fn(|k| (128.0 + t * (p.target[k] as f32 - 128.0) + 0.5) as u8);
        if !p.rising && p.t <= 0.0 {
            self.pulse = None;
        }
    }

    fn bar(max: i32, cur: i32, base: i32, step: i32, w: &Words) -> Bar {
        let l = width(max);
        Bar { middle: l - w.inset, cap_x: l + w.inset + w.cap, fill: width(cur), dividers: if step > 0 { ((max - base) / step).max(0) } else { 0 } }
    }

    /// One in-play tick. `hold` keeps G0, G1 and C at 240 (the quick panel is open).
    pub fn tick(&mut self, v: Values, hold: bool, w: &Words) {
        // A health or pool change through the character accessors re-arms C.
        if self.seen.is_some_and(|s| (s.hp, s.hp_max, s.mp, s.mp_max) != (v.hp, v.hp_max, v.mp, v.mp_max)) {
            self.c = 240;
        }
        self.seen = Some(v);
        if hold {
            (self.c, self.g0, self.g1) = (240, 240, 240);
        }
        if self.c != 0 {
            let now = [v.hp, v.hp_max, v.mp, v.mp_max];
            if now != self.watched {
                (self.c, self.g0, self.g1) = (240, 240, 240);
            }
            let health_fell = v.hp < self.watched[0] && self.watched[0] >= 0;
            self.watched = now;
            if health_fell {
                self.start_pulse(w.red, w);
            } else if v.poisoned && self.pulse.is_none() {
                self.start_pulse(w.green, w);
            } else if self.pulse.is_some() {
                self.advance_pulse(w);
            }
            self.health = Self::bar(v.hp_max, v.hp, w.base.0, w.step.0, w);
            if v.mp_max >= 1 {
                self.pool = Self::bar(v.mp_max, v.mp, w.base.1, w.step.1, w);
            }
            self.bars_alpha = alpha(self.c);
            if self.c > 0 {
                self.c -= 1;
            }
        }
        if self.g1 > 0 {
            self.g1 -= 1;
        }
        if self.g0 > 0 {
            self.bars_alpha = alpha(self.g0);
            self.g0 -= 1;
        }
    }
}

/// The monster bar's tuning words (SPEC-0023 Layout): frame divisor, maximum and minimum,
/// name margin, and the two piece offsets.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MonsterWords {
    pub divisor: f32,
    pub max: i32,
    pub min: i32,
    pub margin: i32,
    pub cap: i32,
    pub plate: i32,
}

/// SPEC-0023 section 10: one bar for the monster armed most recently.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct MonsterBar {
    pub t: i32,
    pub target: Option<u64>,
    pub name: String,
    /// The name's measured width N (10 when empty), the frame width B and the fill width.
    pub name_width: i32,
    pub frame: i32,
    pub fill: i32,
}

impl MonsterBar {
    /// Arm: target, name (first 31 characters), T = trunc(60 x 10.0) = 600, then the layout.
    pub fn arm(&mut self, target: u64, name: &str, name_width: i32, hp: i32, max: i32, w: &MonsterWords) {
        self.target = Some(target);
        self.name = name.chars().take(31).collect();
        self.name_width = if self.name.is_empty() { 10 } else { name_width };
        self.t = 600;
        self.layout(hp, max, w);
    }

    fn layout(&mut self, hp: i32, max: i32, w: &MonsterWords) {
        let m = max.max(1) as f32;
        let b = ((max as f32 / w.divisor + 1.0) as i32).clamp(w.min, w.max).max(self.name_width + w.margin);
        self.frame = b;
        self.fill = ((hp as f32 * (b as f32 / m) + 1.0) as i32).min(b);
        if hp == 0 && self.t as f32 > 60.0 {
            self.t = 60;
        }
    }

    /// Each in-play tick while T is not 0: T - 1, then the layout with the zero-health cut.
    pub fn tick(&mut self, hp: i32, max: i32, w: &MonsterWords) {
        if self.t != 0 {
            self.t -= 1;
            self.layout(hp, max, w);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const MW: MonsterWords = MonsterWords { divisor: 1.5, max: 250, min: 64, margin: 24, cap: -5, plate: -5 };

    #[test]
    fn monster_bar_widths_and_timer() {
        // Section 10.3 examples.
        let mut b = MonsterBar::default();
        b.arm(1, "", 0, 150, 150, &MW);
        assert_eq!((b.t, b.frame, b.fill), (600, 101, 101));
        b.arm(1, "", 0, 60, 60, &MW);
        assert_eq!(b.frame, 64);
        b.arm(1, "", 0, 600, 600, &MW);
        assert_eq!(b.frame, 250);
        b.arm(1, "Spyder", 60, 60, 60, &MW);
        assert_eq!(b.frame, 84);
        // A kill: the cut to 60 on the next tick, the 1-pixel fill, hidden after 60 more.
        b.arm(2, "", 0, 16, 16, &MW);
        b.tick(0, 16, &MW);
        assert_eq!((b.t, b.fill), (60, 1));
        for _ in 0..60 {
            b.tick(0, 16, &MW);
        }
        assert_eq!(b.t, 0);
    }

    const W: Words = Words { half: 20.0, red: [200, 0, 0, 128], green: [0, 200, 0, 128], cap: -9, inset: 6, base: (150, 90), step: (25, 15) };

    #[test]
    fn widths_and_layout() {
        // Health maximum 150 gives L = 100; pool maximum 90 gives L = 60.
        assert_eq!((width(150), width(90)), (100, 60));
        let b = Hud::bar(150, 75, 150, 25, &W);
        assert_eq!(b, Bar { middle: 94, cap_x: 97, fill: 50, dividers: 0 });
        assert_eq!(Hud::bar(200, 200, 150, 25, &W).dividers, 2);
        assert_eq!((alpha(-1), alpha(30), alpha(29), alpha(1)), (128, 128, 123, 4));
    }

    #[test]
    fn change_shows_bars_then_fades() {
        let mut h = Hud::default();
        let v = Values { hp: 150, hp_max: 150, mp: 90, mp_max: 90, poisoned: false };
        h.tick(v, false, &W);
        assert!(h.bars_drawn() && h.health_drawn() && h.pool_drawn(&v), "the first update after the reset");
        assert_eq!(h.c, 239, "a change counted once: C shows 239");
        let mut ticks = 1;
        while h.bars_drawn() {
            h.tick(v, false, &W);
            ticks += 1;
        }
        assert_eq!(ticks, 240, "hidden 240 ticks after the last re-arm");
        // A health loss re-arms and starts the red pulse (peak after 20 ticks, 40 in all).
        let hit = Values { hp: 120, ..v };
        h.tick(hit, false, &W);
        assert!(h.bars_drawn() && h.tint != [128; 4]);
        let mut peak = h.tint;
        let mut n = 1;
        while h.tint != [128; 4] {
            h.tick(hit, false, &W);
            if h.tint[1] < peak[1] {
                peak = h.tint;
            }
            n += 1;
        }
        assert_eq!(peak, [200, 0, 0, 128]);
        assert!((38..=41).contains(&n), "{n}");
    }
}
