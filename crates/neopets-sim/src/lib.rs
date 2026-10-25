//! Engine-independent simulation boundary. No recovered gameplay is implemented yet.
#![forbid(unsafe_code)]
pub use ps2_float::{F, Vf};

pub mod attack;
pub mod camera;
pub mod clip;
pub mod combat;
pub mod contact;
pub mod credits;
pub mod cues;
pub mod death;
pub mod emote;
pub mod enemy;
pub mod hero;
pub mod hud;
pub mod items;
pub mod level;
pub mod motion;
pub mod music;
pub mod nav;
pub mod projectile;
pub mod quick;
pub mod npc;
pub mod options;
pub mod rng;
pub mod save;
pub mod scene;
pub mod sound;
pub mod sequencer;
pub mod shop;
pub mod synth;
pub mod status;
mod step;
pub mod textbox;
pub mod travel;
pub mod trigger;
pub mod walker;
pub mod whinny;
pub use step::FixedStep;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PlayerInput {
    pub buttons: u32,
    pub move_x: i16,
    pub move_y: i16,
}

/// Recovered systems will own their state here; input/time arrive from the app.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Simulation {
    pub tick: u64,
}

impl Simulation {
    /// Scaffold clock only. This is not Neopets movement, physics, or game timing proof.
    pub fn advance(&mut self, _input: PlayerInput) {
        self.tick = self.tick.wrapping_add(1);
    }

    /// Run as many fixed ticks as `elapsed_ns` of host time allows, all with the same input.
    /// Returns the number of ticks run.
    pub fn run_for(&mut self, clock: &mut FixedStep, elapsed_ns: u64, input: PlayerInput) -> u32 {
        let ticks = clock.advance(elapsed_ns);
        for _ in 0..ticks {
            self.advance(input);
        }
        ticks
    }
}
