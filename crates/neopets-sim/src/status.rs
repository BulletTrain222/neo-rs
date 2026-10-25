//! Hero statuses (SPEC-0036 Status timers, Status effects, Status apply, Regeneration): the 14
//! timers, the hero's apply method with the immunities, the per-tick routine (stun, poison with
//! its 1 HP floor, slow and speed scales, the regeneration statuses with the timer-12 quirk), the
//! low-level trickle regeneration of health and magic, and the curse and strength damage scale.
//!
//! The status display (SPEC-0036 Status display and cues; SPEC-0034 5.4): the shown status, its
//! 4 s rotation over codes 0 to 7, the effect ids and the steady tint.
//!
//! Not covered: the display's cues and the pair managers of its emitters.

pub const STUN: usize = 0;
pub const CURSE: usize = 1;
pub const POISON: usize = 2;
pub const SLOW: usize = 3;
pub const STRENGTH: usize = 4;
pub const SPEED: usize = 5;
pub const FIRE: usize = 7;
pub const IMMUNE_CURSE: usize = 8;
pub const IMMUNE_POISON: usize = 9;
pub const IMMUNE_SLOW: usize = 10;
pub const HEALTH_REGEN: usize = 12;
pub const MAGIC_REGEN: usize = 13;

/// Default durations in seconds by status (0 where the hero's method applies nothing).
const DEFAULTS: [i32; 14] = [5, 60, 120, 60, 60, 60, 0, 5, 30, 30, 30, 0, 60, 60];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Statuses {
    pub timers: [i32; 14],
}

impl Default for Statuses {
    fn default() -> Self {
        Self { timers: [-1; 14] }
    }
}

/// What the hero's per-tick status routine asks of the game.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Tick {
    /// Stun: the action state is set to 1 and the speed scale to 0.
    pub stunned: bool,
    /// The speed scale P after slow and speed.
    pub scale: f32,
    /// A poison hit of this many HP (attacker -2, blocked flag 1).
    pub poison: Option<i32>,
    /// Health and magic gained from the regeneration statuses.
    pub hp: i32,
    pub mp: i32,
}

impl Statuses {
    pub fn active(&self, i: usize) -> bool {
        self.timers.get(i).is_some_and(|&t| t >= 1)
    }

    /// Death and a scene setup: every timer to -1.
    pub fn clear_all(&mut self) {
        self.timers = [-1; 14];
    }

    /// Clear one status when it is active.
    pub fn clear(&mut self, i: usize) {
        if self.active(i) {
            self.timers[i] = -1;
        }
    }

    /// The hero's status-apply method: `seconds` -1 takes the default. Returns false when the
    /// apply was blocked (an immunity, or a code the hero's method ignores).
    pub fn apply(&mut self, code: i32, seconds: i32) -> bool {
        if code == -1 {
            self.clear_all();
            return true;
        }
        let Ok(i) = usize::try_from(code) else { return false };
        if i >= 14 || matches!(i, 6 | 11) {
            return false;
        }
        let blocked = match i {
            CURSE => self.active(IMMUNE_CURSE),
            POISON => self.active(IMMUNE_POISON),
            SLOW => self.active(IMMUNE_SLOW),
            _ => false,
        };
        if blocked {
            return false;
        }
        let s = if seconds == -1 { DEFAULTS[i] } else { seconds };
        if s < 0 {
            return false;
        }
        let ticks = s * 60;
        if (IMMUNE_CURSE..=IMMUNE_SLOW).contains(&i) {
            if ticks < 1 {
                return false;
            }
            self.clear(i - 7);
        }
        self.timers[i] = ticks;
        true
    }

    /// The per-tick routine. `moving` is the moving-state test, `attacking` the in-attack test.
    pub fn tick(&mut self, health: i32, moving: bool, attacking: bool) -> Tick {
        for t in &mut self.timers {
            if *t >= 1 {
                *t -= 1;
            }
        }
        let mut out = Tick { scale: 1.0, ..Tick::default() };
        if self.active(STUN) {
            out.stunned = true;
            out.scale = 0.0;
        }
        if self.active(POISON) && self.timers[POISON] % (4 * 60) == 0 {
            let a = 2;
            out.poison = Some(if health - a < 2 { (health - 1).max(0) } else { a });
        }
        if self.active(SLOW) && (moving || attacking) {
            out.scale *= 0.75;
        }
        if self.active(SPEED) && moving {
            out.scale *= 1.85;
        }
        // A timer of -1 (cleared) is no beat; one that ran down to 0 beats on every tick.
        let beat = self.timers[HEALTH_REGEN] % 60 == 0;
        if self.active(HEALTH_REGEN) && beat {
            out.hp = 2;
        }
        // Quirk: the magic regeneration reads the status-12 timer.
        if self.active(MAGIC_REGEN) && beat {
            out.mp = 2;
        }
        out
    }

    /// The attacker's damage scale G: curse 0.75, strength 1.5.
    pub fn damage_scale(&self) -> f32 {
        let mut g = 1.0;
        if self.active(CURSE) {
            g *= 0.75;
        }
        if self.active(STRENGTH) {
            g *= 1.5;
        }
        g
    }
}

/// SPEC-0036 Regeneration, the trickle toward int(max x 0.1) + 1: the gain this tick (frame
/// counter `frame`; magic every 59 ticks at 1.0/s, health every 29 ticks at 2.0/s).
pub fn trickle(current: i32, max: i32, frame: u64, health: bool) -> i32 {
    let target = (max as f32 * 0.1) as i32 + 1;
    if current >= target {
        return 0;
    }
    let rate: f32 = if health { 2.0 } else { 1.0 };
    let x = rate * (1.0 / 60.0);
    let f = x - x.trunc();
    let m = (1.0 / f) as u64;
    let gain = if m > 0 && frame % m == 0 { (x + 1.0).trunc() as i32 } else { x.trunc() as i32 };
    gain.min(target - current)
}

/// The effect of a shown status (0xCC stun, 0xBE curse, 0xC8 poison, 0xCB slow, 0xBF strength,
/// 0xC0 speed, 0xC9 confusion, 0xCA fire); codes 8 to 10 play the one-shots 0xC2 to 0xC4.
pub fn display_effect(code: i32) -> Option<i32> {
    Some(match code {
        0 => 0xCC,
        1 => 0xBE,
        2 => 0xC8,
        3 => 0xCB,
        4 => 0xBF,
        5 => 0xC0,
        6 => 0xC9,
        7 => 0xCA,
        8 => 0xC2,
        9 => 0xC3,
        10 => 0xC4,
        _ => return None,
    })
}

/// The one-shot when a stun ends.
pub const STUN_END_EFFECT: i32 = 0xCD;

/// The steady tint of a shown status for hero slot `hero` (curse differs for Roberta): each
/// factor channel capped at 1.0; None when all three come out 1.0 (the tint is cleared) or the
/// status sets no tint.
pub fn tint(code: i32, hero: usize) -> Option<[f32; 3]> {
    let f = match code {
        1 if hero == 1 => [0.5, 0.0, 0.4],
        1 => [0.4, 0.0, 0.5],
        2 => [0.2, 1.5, 0.6],
        3 => [0.0, 0.5, 1.5],
        4 => [1.8, 0.3, 0.2],
        5 => [2.0, 2.0, 0.3],
        7 => [1.5, 1.5, 1.0],
        _ => return None,
    };
    let t = f.map(|c: f32| c.min(1.0));
    (t != [1.0; 3]).then_some(t)
}

/// A character's status display: the shown status (-1 none) and the 240-tick rotation timer.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Display {
    pub shown: i32,
    timer: i32,
}

impl Default for Display {
    fn default() -> Self {
        Self { shown: -1, timer: 0 }
    }
}

impl Display {
    /// The display start of an apply: a code other than the shown one becomes shown with the
    /// timer reset. Returns whether the shown status changed.
    pub fn start(&mut self, code: i32) -> bool {
        if code == self.shown {
            return false;
        }
        self.shown = code;
        self.timer = 240;
        true
    }

    /// The next active status after the shown one among codes 0 to 7, wrapping round.
    fn next(&self, active: &dyn Fn(i32) -> bool) -> i32 {
        let from = self.shown.clamp(-1, 7);
        (1..=8).map(|k| (from + k).rem_euclid(8)).find(|&c| active(c)).unwrap_or(-1)
    }

    /// One tick: a shown status no longer active gives way to the next active one (or none);
    /// otherwise the timer drops and at 0 the next active status is shown and the timer
    /// re-armed. Returns whether the shown status changed.
    pub fn tick(&mut self, active: &dyn Fn(i32) -> bool) -> bool {
        let before = self.shown;
        if self.shown >= 0 && !active(self.shown) {
            self.shown = self.next(active);
            self.timer = 240;
        } else if self.shown >= 0 {
            self.timer -= 1;
            if self.timer <= 0 {
                self.shown = self.next(active);
                self.timer = 240;
            }
        }
        self.shown != before
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn immunities_block_and_clear() {
        let mut s = Statuses::default();
        assert!(s.apply(POISON as i32, -1));
        assert_eq!(s.timers[POISON], 7200);
        assert!(s.apply(IMMUNE_POISON as i32, -1), "the immunity clears an active poison");
        assert!(!s.active(POISON) && s.timers[IMMUNE_POISON] == 1800);
        assert!(!s.apply(POISON as i32, -1), "poison is blocked while immune");
        assert!(!s.apply(6, -1) && !s.apply(11, -1), "confusion and poison attack are ignored");
        s.apply(-1, -1);
        assert_eq!(s, Statuses::default());
    }

    #[test]
    fn poison_ticks_with_its_floor() {
        let mut s = Statuses::default();
        s.apply(POISON as i32, 8);
        let mut hits = Vec::new();
        let mut hp = 5;
        for t in 1..=480 {
            if let Some(d) = s.tick(hp, false, false).poison {
                hits.push((t, d));
                hp -= d;
            }
        }
        // Timer 480: a multiple of 240 at 240 remaining (tick 240); at 0 it is no longer active.
        assert_eq!(hits, vec![(240, 2)]);
        assert_eq!(hp, 3);
        let mut s = Statuses::default();
        s.apply(POISON as i32, 5);
        assert_eq!((1..=240).find_map(|_| s.tick(2, false, false).poison), Some(1), "health - 2 < 2: D = health - 1");
    }

    #[test]
    fn regeneration_quirk_and_scales() {
        let mut s = Statuses::default();
        s.apply(MAGIC_REGEN as i32, -1);
        assert!((0..120).all(|_| s.tick(10, false, false).mp == 0), "magic reads the status-12 timer, which is off");
        s.apply(HEALTH_REGEN as i32, 2);
        let gains: i32 = (0..120).map(|_| s.tick(10, false, false)).map(|t| t.hp + t.mp).sum();
        // Timer 60 gives +2 HP and +2 MP; at 0 status 12 has ended but magic still beats (the
        // SPEC-0036 Edge cases quirk: +2 MP on every tick while timer 12 is 0).
        assert_eq!(gains, 6);
        let mut s = Statuses::default();
        s.apply(SLOW as i32, -1);
        s.apply(SPEED as i32, -1);
        assert!((s.tick(10, true, false).scale - 0.75 * 1.85).abs() < 1e-6);
        assert_eq!(s.tick(10, false, false).scale, 1.0);
        s.apply(STUN as i32, -1);
        let t = s.tick(10, true, false);
        assert!(t.stunned && t.scale == 0.0);
        s.apply(CURSE as i32, -1);
        s.apply(STRENGTH as i32, -1);
        assert!((s.damage_scale() - 1.125).abs() < 1e-6);
    }

    #[test]
    fn trickle_periods() {
        // Magic: +1 every 59 ticks toward 10 for a maximum of 90; health every 29 ticks.
        let gains: Vec<u64> = (1..=200u64).filter(|&f| trickle(5, 90, f, false) > 0).collect();
        assert_eq!(gains, vec![59, 118, 177]);
        let gains: Vec<u64> = (1..=60u64).filter(|&f| trickle(5, 150, f, true) > 0).collect();
        assert_eq!(gains, vec![29, 58]);
        assert_eq!(trickle(16, 150, 29, true), 0, "at the target 16 nothing is gained");
    }

    /// SPEC-0036 Status display test vectors: slow tints (0, 0.5, 1.0), fire clears the tint,
    /// speed (1.0, 1.0, 0.3); the rotation shows the next active status after 240 ticks.
    #[test]
    fn display_tint_and_rotation() {
        assert_eq!(tint(3, 0), Some([0.0, 0.5, 1.0]));
        assert_eq!(tint(7, 0), None);
        assert_eq!(tint(5, 0), Some([1.0, 1.0, 0.3]));
        assert_eq!((tint(1, 0), tint(1, 1)), (Some([0.4, 0.0, 0.5]), Some([0.5, 0.0, 0.4])));
        let mut d = Display::default();
        let active = |c: i32| c == 2 || c == 5;
        assert!(d.start(2));
        assert!(!d.start(2));
        for _ in 0..239 {
            assert!(!d.tick(&active));
        }
        assert!(d.tick(&active));
        assert_eq!(d.shown, 5);
        // The shown status ends: the next active one, or none.
        let only = |c: i32| c == 2;
        assert!(d.tick(&only));
        assert_eq!(d.shown, 2);
        assert!(d.tick(&|_| false));
        assert_eq!(d.shown, -1);
    }
}
