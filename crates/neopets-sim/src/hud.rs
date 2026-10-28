//! The in-play HUD's bar rules (SPEC-0023 Behavior 1 to 4): the change detector C and the
//! group timers G0 (bars 0x31), G1 (quick-item panel 0x30) and G2 to G4 (equipment panels 0x1C,
//! 0x1D and 0x1E) with their alpha pushes and re-arms, the bar update with its watched
//! values, the bar widths L = trunc(max x K) and F = trunc(current x K), the frame layout, the
//! upgrade dividers, alpha(t), and the red and green tint pulses of the health frame (0x3B).
//!
//! The monster health bar (section 10): the arm with its 600-tick timer, the zero-health cut,
//! the frame width from the maximum and the name width, the fill width.
//!
//! The D-pad hint's timer D and alpha (section 9) and the journal-notification icon's flag,
//! phase and alpha (section 8).
//!
//! Not covered: the loss effects, the petpet meter, the portrait swing and the countdown texts.

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
    /// G2, G3 and G4: the equipment panels 0x1C (weapon slot 2), 0x1D (slot 0) and 0x1E (slot 1).
    pub g: [i32; 3],
    /// The alpha last pushed to groups 0x30, 0x1C, 0x1D and 0x1E.
    pub panel_alpha: [u8; 4],
    watched: [i32; 4],
    seen: Option<Values>,
    pulse: Option<Pulse>,
    /// The health frame's colour (element 0x3B), the alpha last pushed to group 0x31, and the
    /// bars' layout.
    pub tint: [u8; 4],
    pub bars_alpha: u8,
    pub health: Bar,
    pub pool: Bar,
    /// The D-pad hint's timer D (element 0xBB; section 9).
    pub d: i32,
    /// The journal-notification flag and its phase (element 0xDF; section 8).
    pub journal: bool,
    pub journal_phase: f32,
}

impl Default for Hud {
    fn default() -> Self {
        // The full reset after a load: all timers 0, C = -1.
        Self { c: -1, g0: 0, g1: 0, g: [0; 3], panel_alpha: [128; 4], watched: [-1; 4], seen: None, pulse: None, tint: [128; 4], bars_alpha: 128, health: Bar::default(), pool: Bar::default(), d: 0, journal: false, journal_phase: 0.0 }
    }
}

/// Round half away from zero.
fn round_away(x: f32) -> i32 {
    if x < 0.0 { -((-x + 0.5).floor()) as i32 } else { (x + 0.5).floor() as i32 }
}

impl Hud {
    /// Section 9: 128 while D >= 30, else round(D x (128 - 32) x 0.033333335 + 32).
    pub fn dpad_alpha(&self) -> u8 {
        if self.d >= 30 { 128 } else { round_away(self.d as f32 * 96.0 * 0.033_333_335 + 32.0).clamp(0, 255) as u8 }
    }

    /// Section 8: y = -|phase| / 2 pi, t = |(round(y) - y) - 0.5| - 0.25, alpha =
    /// round(64 (1 + P(t))) with the truncated sine series (it is inaccurate as t nears 0.75, so
    /// the bottom of each cycle stays near 14 to 25).
    pub fn journal_alpha(&self) -> u8 {
        let y = -self.journal_phase.abs() / std::f32::consts::TAU;
        let t = ((y.round() - y) - 0.5).abs() - 0.25;
        let t2 = t * t;
        let (c1, c3, c5, c7, c9) = (6.283185f32, 41.34167f32, 81.60223f32, 76.57496f32, 39.71066f32);
        let p = t * (c1 - t2 * (c3 - t2 * (c5 - t2 * (c7 - t2 * c9))));
        round_away(64.0 * (1.0 + p)).clamp(0, 255) as u8
    }

    /// Section 8: a progress flag with a journal record set for the first time.
    pub fn notify_journal(&mut self) {
        self.journal = true;
        self.journal_phase = 0.0;
    }

    /// Per tick: D lowered by 1 while above 0, and the journal phase advanced by 0.15 (boot
    /// 0x2DCF18) while the flag is set.
    pub fn tick_icons(&mut self) {
        if self.d > 0 {
            self.d -= 1;
        }
        if self.journal {
            self.journal_phase += 0.15;
        }
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

    /// L3 (and a hero switch with the extra flag): C and G0 to G4 to 240.
    pub fn rearm_all(&mut self) {
        (self.c, self.g0, self.g1) = (240, 240, 240);
        self.g = [240; 3];
    }

    /// Equipping Mote word k, or spending charge from the Mote in it (section 4 re-arms): word 0
    /// re-arms G3 (panel 0x1D), word 1 G4 (0x1E) and word 2 G2 (0x1C).
    pub fn rearm_word(&mut self, word: usize) {
        match word {
            0 => self.g[1] = 240,
            1 => self.g[2] = 240,
            2 => self.g[0] = 240,
            _ => {}
        }
    }

    /// Selecting or using a quick item: G0, G1 and C to 240.
    pub fn rearm_quick(&mut self) {
        (self.c, self.g0, self.g1) = (240, 240, 240);
    }

    /// Opening the pause page or the item screen (section 4.1): G0 to G4 and C to 0.
    pub fn hide_all(&mut self) {
        (self.c, self.g0, self.g1) = (0, 0, 0);
        self.g = [0; 3];
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
        // The group fades, in order (0x30, G1), (0x1C, G2), (0x1D, G3), (0x1E, G4), (0x31, G0).
        if self.g1 > 0 {
            self.panel_alpha[0] = alpha(self.g1);
            self.g1 -= 1;
        }
        for k in 0..3 {
            if self.g[k] > 0 {
                self.panel_alpha[k + 1] = alpha(self.g[k]);
                self.g[k] -= 1;
            }
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

    #[test]
    fn dpad_and_journal_icons() {
        // Section 9: full alpha for 210 ticks after L3, then down to 32.
        let mut h = Hud { d: 240, ..Default::default() };
        assert_eq!(h.dpad_alpha(), 128);
        h.d = 15;
        assert_eq!(h.dpad_alpha(), 80);
        h.d = 0;
        assert_eq!(h.dpad_alpha(), 32);
        // Section 8: phase 0 gives P(0.25) = 1 (alpha 128); the flag's ticks advance the phase.
        h.notify_journal();
        assert_eq!(h.journal_alpha(), 128);
        h.tick_icons();
        assert!((h.journal_phase - 0.15).abs() < 1e-6 && h.journal_alpha() < 128);
        // The series quirk: where t wraps to 0.75 the alpha runs 25, 18, 15, 14, 17.
        let seq: Vec<u8> = (0..44).map(|_| { h.tick_icons(); h.journal_alpha() }).collect();
        assert!(seq.windows(5).any(|w| w == [25, 18, 15, 14, 17]), "{seq:?}");
    }

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

    #[test]
    fn mote_word_rearms() {
        // SPEC-0023 section 4: word 2 G2, word 1 G4, word 0 G3; C, G0 and G1 untouched.
        let mut h = Hud::default();
        h.rearm_word(2);
        assert_eq!((h.g, h.c, h.g0, h.g1), ([240, 0, 0], -1, 0, 0));
        h.rearm_word(1);
        h.rearm_word(0);
        assert_eq!(h.g, [240; 3]);
    }

    #[test]
    fn panel_timers_fade_and_rearm() {
        let w = Words { half: 20.0, red: [200, 0, 0, 128], green: [0, 200, 0, 128], cap: -9, inset: 6, base: (150, 90), step: (25, 15) };
        let v = Values { hp: 150, hp_max: 150, mp: 90, mp_max: 90, poisoned: false };
        let mut h = Hud::default();
        h.tick(v, false, &w);
        assert_eq!(h.g, [0; 3]);
        // L3: every group at 240; full alpha for 210 ticks, then the fade, hidden at 0.
        h.rearm_all();
        for _ in 0..215 {
            h.tick(v, false, &w);
        }
        assert_eq!((h.g, h.panel_alpha), ([25; 3], [110; 4]), "pushed at 26 before the decrement");
        for _ in 0..25 {
            h.tick(v, false, &w);
        }
        assert_eq!(h.g, [0; 3]);
        assert_eq!(h.panel_alpha[1], 4);
        // A quick item re-arms G0, G1 and C only; the pause page clears them all.
        h.rearm_quick();
        assert_eq!((h.g0, h.g1, h.c, h.g), (240, 240, 240, [0; 3]));
        h.hide_all();
        assert_eq!((h.g0, h.g1, h.c), (0, 0, 0));
    }
}
