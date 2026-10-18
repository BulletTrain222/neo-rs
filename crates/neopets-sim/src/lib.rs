//! Engine-independent simulation boundary. No recovered gameplay is implemented yet.
#![forbid(unsafe_code)]
pub use ps2_float::{F, Vf};

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
}
