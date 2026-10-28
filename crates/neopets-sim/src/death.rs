//! Hero death flow (SPEC-0055 sections 4, 8 and 8A): the death-state handler (death clip by
//! ground, death mode, the partner test at clip progress 0.95), timers E (game over) and F
//! (deadly-ground respawn) with their 1.5 s rule, the deadly-ground kill test, the state-60
//! penalty, and the game-over page's pass counter and exit test.
//!
//! Death callbacks are level::on_hero_death. Not covered: the partner (Roberta is not
//! playable, so the switch request is always refused), the death camera (mode 8), effects and
//! sounds, the hurting surface and the deep overlay, and the front end behind the game-over page
//! (the card state, countdown, attract and title; the session snapshot of Continue).

use crate::hero::ClipState;

/// The single-precision tick length added by the game-state timers.
const DT: f32 = 1.0 / 60.0;

/// A game-state timer (SPEC-0055 Timers): off, or the accumulated time.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Timer(Option<f32>);

impl Timer {
    pub fn running(&self) -> bool {
        self.0.is_some()
    }

    pub fn start(&mut self) {
        self.0 = Some(0.0);
    }

    pub fn stop(&mut self) {
        self.0 = None;
    }

    /// One pass: add dt; true on the first pass where the time is more than 1.5 s.
    pub fn pass(&mut self) -> bool {
        match self.0.as_mut() {
            Some(t) => {
                *t += DT;
                *t > 1.5
            }
            None => false,
        }
    }
}

/// The ground under the hero as the death rules read it: a deadly entry (kind 2) with its
/// respawn entry index (0xFF none) and surface word.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Deadly {
    pub respawn: u8,
    pub surface: u16,
}

/// The deadly-ground kill test: kind-2 ground with a gap of 0.085 or less.
pub fn deadly_kill(deadly: Option<Deadly>, gap: f32) -> bool {
    deadly.is_some() && gap <= 0.085
}

/// The death clip and one-shot effect for the ground (section 8A, step 1 and 2).
pub fn death_clip(deadly: Option<Deadly>) -> (u32, Option<u32>) {
    match deadly.map(|d| d.surface) {
        None => (34, None),
        Some(2) => (173, Some(0x81)),
        Some(8) => (174, Some(0xBC)),
        Some(_) => (136, None),
    }
}

/// What a game-state pass did.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Pass {
    /// Timer E fired: flag E is set (sound event 5).
    pub game_over: bool,
    /// Timer F fired: respawn the selected hero at this entry record.
    pub respawn: Option<u8>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct DeathFlow {
    /// The hero's death mode: 0 none, 1 game over, 2 deadly-ground respawn.
    pub mode: u8,
    pub timer_e: Timer,
    pub timer_f: Timer,
    pub f_index: u8,
    /// Flag E (game over), and the game-over page's pass counter while the page is up.
    pub flag_e: bool,
    pub page: Option<u32>,
}

impl DeathFlow {
    pub fn new() -> Self {
        Self { f_index: 0xFF, ..Self::default() }
    }

    /// The status machine's pass, before the world update.
    pub fn pass(&mut self) -> Pass {
        let mut out = Pass::default();
        if self.timer_e.pass() {
            self.timer_e.stop();
            self.flag_e = true;
            out.game_over = true;
        }
        if self.timer_f.pass() {
            self.timer_f.stop();
            out.respawn = Some(self.f_index);
            self.f_index = 0xFF;
        }
        out
    }

    /// The death-state handler for one tick of state 16: returns the death clip to request.
    /// `partner_takes_over` is the partner test at progress 0.95 (alive and the switch accepted).
    pub fn handler(&mut self, deadly: Option<Deadly>, clip: ClipState, partner_takes_over: bool) -> u32 {
        self.mode = 0;
        let (id, _effect) = death_clip(deadly);
        match deadly {
            Some(d) if d.respawn != 0xFF => {
                self.mode = 2;
                if !self.flag_e && !self.timer_e.running() && !self.timer_f.running() {
                    self.timer_f.start();
                    self.f_index = d.respawn;
                }
            }
            Some(_) => self.mode = 1,
            None => {
                if clip.action == id && clip.progress >= 0.95 && !partner_takes_over {
                    self.mode = 1;
                }
            }
        }
        if self.mode == 1 && !self.flag_e && !self.timer_e.running() {
            self.timer_e.start();
        }
        id
    }

    /// The game-over page: the hand-over pass sets the counter to 0.
    pub fn hand_over(&mut self) {
        self.page = Some(0);
    }

    /// One page pass: the exit test (counter above 120 and Cross newly pressed), then the
    /// counter rises. True on the exit pass (flag E is cleared).
    pub fn page_pass(&mut self, cross_new: bool) -> bool {
        let Some(n) = self.page.as_mut() else { return false };
        let exit = *n > 120 && cross_new;
        *n = n.saturating_add(1).min(i32::MAX as u32);
        if exit {
            self.page = None;
            self.flag_e = false;
            self.timer_e.stop();
        }
        exit
    }
}

/// The state-60 penalty: D = min(5, health - 1), never killing.
pub fn respawn_penalty(health: i32) -> i32 {
    if health > 5 { 5 } else { (health - 1).max(0) }
}

/// SPEC-0057 7A rule 2 (SPEC-0055 section 9): a non-deadly ground entry with surface word 2
/// hurts while the gap is below 0.2, on each tick where clock - trunc(clock x 0.4) x 2.5 < dt
/// (once every 2.5 s of game clock, phased by the clock), for 10 HP.
pub fn hurting_tick(surface: Option<u16>, deadly: bool, gap: f32, clock: f32, dt: f32) -> bool {
    use ps2_float::fpu::{mul, sub};
    surface == Some(2) && !deadly && gap < 0.2 && sub(clock, mul((mul(clock, 0.4) as i32) as f32, 2.5)) < dt
}

/// The hurting surface's hit.
pub const HURT_DAMAGE: f32 = 10.0;

#[cfg(test)]
mod tests {
    use super::*;

    fn clip(action: u32, progress: f32) -> ClipState {
        ClipState { action, progress, finished: false }
    }

    #[test]
    fn timer_fires_on_the_ninety_first_addition() {
        let mut t = Timer::default();
        t.start();
        assert!((0..90).all(|_| !t.pass()));
        assert!(t.pass());
    }

    #[test]
    fn deadly_ground_with_an_index_respawns() {
        // Pass 1001: the handler plays 174, mode 2, timer F starts; 91 passes later: respawn.
        let mut d = DeathFlow::new();
        let ground = Some(Deadly { respawn: 3, surface: 8 });
        assert!(deadly_kill(ground, 0.05) && !deadly_kill(ground, 0.1));
        assert_eq!(d.handler(ground, clip(0, 0.0), false), 174);
        assert_eq!(d.mode, 2);
        for _ in 0..90 {
            assert_eq!(d.pass(), Pass::default());
            d.handler(ground, clip(174, 0.5), false);
        }
        assert_eq!(d.pass().respawn, Some(3));
        assert_eq!((d.f_index, d.timer_f.running()), (0xFF, false));
        assert_eq!(respawn_penalty(40), 5);
        assert_eq!(respawn_penalty(4), 3);
        assert_eq!(respawn_penalty(1), 0);
    }

    #[test]
    fn ordinary_ground_death_starts_game_over_at_progress_095() {
        let mut d = DeathFlow::new();
        assert_eq!(d.handler(None, clip(34, 0.9), false), 34);
        assert!(!d.timer_e.running());
        d.handler(None, clip(34, 0.95), true);
        assert!(!d.timer_e.running(), "an accepted switch starts no timer");
        d.handler(None, clip(34, 0.96), false);
        assert_eq!(d.mode, 1);
        let fired = (0..91).map(|_| d.pass().game_over).position(|g| g);
        assert_eq!(fired, Some(90));
        assert!(d.flag_e);
    }

    #[test]
    fn game_over_page_exit_needs_a_new_press_from_pass_122() {
        let mut d = DeathFlow::new();
        d.flag_e = true;
        d.hand_over();
        // Page passes 1 to 121 with Cross pressed on 121 only: no exit.
        for pass in 1..=121 {
            assert!(!d.page_pass(pass == 121));
        }
        // Newly pressed on page pass 122: exit.
        assert!(d.page_pass(true));
        assert!(!d.flag_e && d.page.is_none());
    }

    /// SPEC-0057 TV-G1 to G3 and G5: hits at the clock times 2.5 and 5.0 only, none above the
    /// 0.2 gap, none on deadly ground.
    #[test]
    fn hurting_surface_phase() {
        let dt = 1.0f32 / 60.0;
        let hits: Vec<u32> = (0..200u32).filter(|&t| hurting_tick(Some(2), false, 0.1, 2.49 + t as f32 * dt, dt)).collect();
        let times: Vec<f32> = hits.iter().map(|&t| 2.49 + t as f32 * dt).collect();
        assert_eq!(times.len(), 2, "{times:?}");
        assert!((times[0] - 2.5).abs() < dt && (times[1] - 5.0).abs() < dt);
        assert!(!hurting_tick(Some(2), false, 0.25, 2.5, dt));
        assert!(!hurting_tick(Some(2), true, 0.1, 2.5, dt));
        assert!(!hurting_tick(Some(0), false, 0.1, 2.5, dt));
    }
}
