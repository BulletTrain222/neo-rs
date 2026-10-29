//! Fixed-size UI windows (the developer menu, the Display page) scaled down to fit a small game
//! window (this build's UI; not a spec rule).
use bevy::prelude::*;

/// The window's margin to the game window's edges, in logical pixels, and the smallest scale.
const EDGE: f32 = 16.0;
const MIN_SCALE: f32 = 0.35;

/// A window laid out at its full size and then scaled down as one piece to fit the game window
/// (never up). Its top edge stays put; `centred` keeps it centred across, else its left edge
/// stays put. Bevy's UI picking follows the transform, so clicks land on the scaled rows.
#[derive(Component, Clone, Copy)]
#[require(UiTransform)]
pub struct FitToWindow {
    pub centred: bool,
}

/// After UI layout: each FitToWindow node's scale and the shift that keeps its anchor.
pub fn fit_to_window(windows: Query<&Window, With<bevy::window::PrimaryWindow>>, mut nodes: Query<(&ComputedNode, &FitToWindow, &mut UiTransform)>) {
    let Ok(win) = windows.single() else { return };
    for (node, fit, mut t) in &mut nodes {
        let size = node.size() * node.inverse_scale_factor();
        if size.x <= 0.0 || size.y <= 0.0 {
            continue;
        }
        let room = Vec2::new(win.width() - 2.0 * EDGE, win.height() - 2.0 * EDGE).max(Vec2::ONE);
        let s = (room.x / size.x).min(room.y / size.y).clamp(MIN_SCALE, 1.0);
        // UiTransform scales about the node's centre: shift it so the anchor stays put.
        let shift = -(1.0 - s) * size / 2.0;
        let want = UiTransform { translation: Val2::px(if fit.centred { 0.0 } else { shift.x }, shift.y), scale: Vec2::splat(s), ..UiTransform::IDENTITY };
        if *t != want {
            *t = want;
        }
    }
}
