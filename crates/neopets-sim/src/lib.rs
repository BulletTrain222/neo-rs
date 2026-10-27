//! Engine-independent simulation boundary. No recovered gameplay is implemented yet.
#![forbid(unsafe_code)]
pub use ps2_float::{F, Vf};

pub mod attack;
pub mod camera;
pub mod chain;
pub mod climb;
pub mod ledge;
pub mod block;
pub mod book;
pub mod burrows;
pub mod charlight;
pub mod charm;
pub mod clip;
pub mod combat;
pub mod contact;
pub mod credits;
pub mod effect;
pub mod cues;
pub mod death;
pub mod emote;
pub mod gesture;
pub mod darkest;
pub mod draik;
pub mod enemy;
pub mod entityctl;
pub mod foliage;
pub mod hero;
pub mod lock;
pub mod hud;
pub mod inventory;
pub mod items;
pub mod ladder;
pub mod ixi;
pub mod fyora;
pub mod gnarfas;
pub mod juppie;
pub mod gelert;
pub mod knight;
pub mod sisters;
pub mod summon;
pub mod werelupe;
pub mod miamouse;
pub mod level;
pub mod mappage;
pub mod mortog;
pub mod mote;
pub mod motion;
pub mod music;
pub mod nav;
pub mod projectile;
pub mod quick;
pub mod npc;
pub mod object;
pub mod options;
pub mod petpet;
pub mod rng;
pub mod rumble;
pub mod roberta;
pub mod save;
pub mod scene;
pub mod sound;
pub mod sequencer;
pub mod serpent;
pub mod shake;
pub mod shop;
pub mod synth;
pub mod status;
pub mod swap;
mod step;
pub mod textbox;
pub mod timer;
pub mod travel;
pub mod trail;
pub mod trigger;
pub mod walker;
pub mod wheel;
pub mod whinny;
pub mod zone;
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
