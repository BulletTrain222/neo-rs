//! The developer and settings windows made only while needed (a performance step of this build;
//! not a spec rule). A hidden window's nodes still cost Bevy's UI layout, stacking and extraction
//! every frame: about 1,000 UI nodes in play, 670 of them in these windows, measured at about
//! 0.3 ms of the main thread a frame (performance pass of 2026-11-12, NEOPETS_PERF_NOPANELS). Each
//! window is spawned when it opens and despawned once it has stayed closed for IDLE frames, so a
//! quick close and reopen keeps it. Its update systems already fill it from their state each frame
//! it is open (rows, values, texts), so a fresh window shows the same page on its first frame.
use bevy::ecs::system::SystemId;
use bevy::prelude::*;

/// Frames a closed window is kept before it is despawned.
const IDLE: u32 = 120;

/// One window: its root marker's despawn and its spawn system.
fn keep<M: Component>(commands: &mut Commands, open: bool, roots: &Query<Entity, With<M>>, idle: &mut u32, spawn: &mut Option<SystemId>, make: fn(Commands)) {
    if open {
        *idle = 0;
        if roots.is_empty() {
            let id = *spawn.get_or_insert_with(|| commands.register_system(make));
            commands.run_system(id);
        }
        return;
    }
    *idle = idle.saturating_add(1);
    if *idle >= IDLE {
        for e in roots {
            commands.entity(e).despawn();
        }
    }
}

#[derive(Default)]
pub struct Idle {
    frames: [u32; 5],
    spawn: [Option<SystemId>; 5],
}

#[allow(clippy::too_many_arguments)]
pub fn sync_windows(
    mut commands: Commands,
    dev: Res<crate::dev_menu::DevMenu>,
    pick: Res<crate::dev_spawn::SpawnPicker>,
    overlay: Res<crate::perf::PerfOverlay>,
    controls: Res<crate::controls_page::ControlsPage>,
    menu: Option<Res<crate::menu_view::Menu>>,
    roots: (
        Query<Entity, With<crate::dev_menu::DevWindow>>,
        Query<Entity, With<crate::dev_spawn::PickWindow>>,
        Query<Entity, With<crate::perf::PerfPanel>>,
        Query<Entity, With<crate::controls_page::CtlWindow>>,
        Query<Entity, With<crate::display::DispWindow>>,
    ),
    mut state: Local<Idle>,
) {
    let _span = crate::perf::span("ui_lazy");
    let Idle { frames, spawn } = &mut *state;
    let display = menu.as_ref().is_some_and(|m| m.flow.current == crate::menus::DISPLAY);
    keep(&mut commands, dev.open, &roots.0, &mut frames[0], &mut spawn[0], crate::dev_menu::spawn_window);
    keep(&mut commands, pick.open, &roots.1, &mut frames[1], &mut spawn[1], crate::dev_spawn::spawn_panel);
    keep(&mut commands, overlay.on, &roots.2, &mut frames[2], &mut spawn[2], crate::perf::spawn_panel);
    keep(&mut commands, controls.open, &roots.3, &mut frames[3], &mut spawn[3], crate::controls_page::spawn_page);
    keep(&mut commands, display, &roots.4, &mut frames[4], &mut spawn[4], crate::display::spawn_window);
}
