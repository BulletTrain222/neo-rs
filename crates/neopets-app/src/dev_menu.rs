//! The developer menu: cheats and a free camera for testing. It is a tool of this build, not
//! part of the original game, and follows no spec.
//!
//! F1 opens and closes the menu; Up and Down pick an entry, Left and Right change its value (Shift
//! for steps of 10), Enter or Space applies it. F2 toggles the free camera: WASD or the arrows
//! move, E and Q rise and sink, J/L and I/K or a right-button drag look around, Shift is fast.
//! While the menu is open or the free camera is on, the hero and the game menus get no input.

use bevy::input::mouse::AccumulatedMouseMotion;
use bevy::prelude::*;

use crate::orbit::Orbit;
use crate::PlayState;

#[derive(Resource, Default)]
pub struct DevMenu {
    pub open: bool,
    cursor: usize,
    amount: usize,
    item: u16,
    flag: u16,
    pub free: Option<FreeCam>,
    note: String,
}

impl DevMenu {
    /// The hero and the game menus get no input this frame.
    pub fn blocks_input(&self) -> bool {
        self.open || self.free.is_some()
    }
}

/// The free camera's place and look angles (radians; yaw about +Y from +Z, pitch up).
#[derive(Clone, Copy, Debug)]
pub struct FreeCam {
    pos: Vec3,
    yaw: f32,
    pitch: f32,
}

impl FreeCam {
    fn from_transform(t: &Transform) -> Self {
        let f = t.forward().as_vec3();
        Self { pos: t.translation, yaw: f.x.atan2(f.z), pitch: f.y.clamp(-1.0, 1.0).asin() }
    }

    fn forward(&self) -> Vec3 {
        Vec3::new(self.yaw.sin() * self.pitch.cos(), self.pitch.sin(), self.yaw.cos() * self.pitch.cos())
    }
}

#[derive(Component)]
pub struct DevPanel;

#[derive(Component)]
pub struct DevText;

const AMOUNTS: [i32; 4] = [100, 1_000, 10_000, 100_000];

#[derive(Clone, Copy, PartialEq, Eq)]
enum Entry {
    God,
    Magic,
    Charge,
    FreeCam,
    Neopoints,
    Heal,
    Motes,
    Item,
    Hero,
    Flag,
}

const ENTRIES: [Entry; 10] = [Entry::God, Entry::Magic, Entry::Charge, Entry::FreeCam, Entry::Neopoints, Entry::Heal, Entry::Motes, Entry::Item, Entry::Hero, Entry::Flag];

pub fn spawn_panel(mut commands: Commands) {
    commands
        .spawn((
            Node { position_type: PositionType::Absolute, left: px(12), top: px(12), max_width: percent(60), padding: UiRect::all(px(12)), display: Display::None, ..default() },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.92)),
            GlobalZIndex(5000),
            DevPanel,
        ))
        .with_children(|ui| {
            ui.spawn((Text::default(), TextFont { font_size: bevy::text::FontSize::Px(16.0), ..default() }, TextColor(Color::srgb(0.9, 0.95, 0.75)), DevText));
        });
}

/// The menu's keys and actions, before the game reads its input.
pub fn dev_input(keys: Res<ButtonInput<KeyCode>>, mut dev: ResMut<DevMenu>, play: Option<ResMut<PlayState>>, cameras: Query<&Transform, With<Orbit>>, mut frames: Local<u32>) {
    let Some(mut play) = play else { return };
    let toggle_free = |dev: &mut DevMenu| {
        dev.free = match dev.free {
            Some(_) => None,
            None => cameras.iter().next().map(FreeCam::from_transform),
        };
    };
    // NEOPETS_DEV="menu,free": open the menu and/or start the free camera once play has run a
    // few frames (diagnostic, for screenshots); with "free" the camera also backs up and rises.
    *frames += 1;
    if *frames == 30 {
        let v = std::env::var("NEOPETS_DEV").unwrap_or_default();
        dev.open |= v.contains("menu");
        if v.contains("free") && dev.free.is_none() {
            toggle_free(&mut dev);
            if let Some(fc) = dev.free.as_mut() {
                let f = fc.forward();
                fc.pos += -f * 25.0 + Vec3::Y * 15.0;
                fc.pitch -= 0.35;
            }
        }
    }
    if keys.just_pressed(KeyCode::F1) {
        dev.open = !dev.open;
    }
    if keys.just_pressed(KeyCode::F2) {
        toggle_free(&mut dev);
    }
    if !dev.open {
        return;
    }
    let n = ENTRIES.len();
    if keys.just_pressed(KeyCode::ArrowDown) || keys.just_pressed(KeyCode::KeyS) {
        dev.cursor = (dev.cursor + 1) % n;
    }
    if keys.just_pressed(KeyCode::ArrowUp) || keys.just_pressed(KeyCode::KeyW) {
        dev.cursor = (dev.cursor + n - 1) % n;
    }
    let step: i32 = if keys.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight]) { 10 } else { 1 };
    let delta = if keys.just_pressed(KeyCode::ArrowRight) || keys.just_pressed(KeyCode::KeyD) {
        step
    } else if keys.just_pressed(KeyCode::ArrowLeft) || keys.just_pressed(KeyCode::KeyA) {
        -step
    } else {
        0
    };
    let entry = ENTRIES[dev.cursor];
    if delta != 0 {
        match entry {
            Entry::Neopoints => dev.amount = (dev.amount as i32 + delta.signum()).clamp(0, AMOUNTS.len() as i32 - 1) as usize,
            Entry::Item => dev.item = (dev.item as i32 + delta).clamp(0, 180) as u16,
            Entry::Flag => dev.flag = (dev.flag as i32 + delta).clamp(0, u16::MAX as i32) as u16,
            _ => {}
        }
    }
    if !(keys.just_pressed(KeyCode::Enter) || keys.just_pressed(KeyCode::NumpadEnter) || keys.just_pressed(KeyCode::Space)) {
        return;
    }
    let p = &mut play.play;
    let on = |b: bool| if b { "on" } else { "off" };
    dev.note = match entry {
        Entry::God => {
            p.cheats.god = !p.cheats.god;
            format!("God mode {}", on(p.cheats.god))
        }
        Entry::Magic => {
            p.cheats.magic = !p.cheats.magic;
            format!("Infinite magic {}", on(p.cheats.magic))
        }
        Entry::Charge => {
            p.cheats.charge = !p.cheats.charge;
            format!("Infinite Mote charge {}", on(p.cheats.charge))
        }
        Entry::FreeCam => {
            toggle_free(&mut dev);
            format!("Free camera {}", on(dev.free.is_some()))
        }
        Entry::Neopoints => {
            let a = AMOUNTS[dev.amount];
            p.state.neopoints = p.state.neopoints.saturating_add(a);
            format!("+{a} NP (now {})", p.state.neopoints)
        }
        Entry::Heal => {
            p.dev_heal();
            "Healed both heroes".into()
        }
        Entry::Motes => {
            for id in 160..=173 {
                p.give(id);
            }
            "Gave every Mote (charge 100 each)".into()
        }
        Entry::Item => {
            p.give(dev.item);
            format!("Gave item {} ({})", dev.item, p.item_label(dev.item))
        }
        Entry::Hero => {
            let before = p.hero_index;
            p.trace_select_hero(1 - before);
            if p.hero_index == before { "No other hero to switch to here".into() } else { format!("Now playing hero {}", p.hero_index) }
        }
        Entry::Flag => {
            let f = dev.flag;
            let v = !p.state.flag(f);
            p.state.set(f, v);
            format!("Story flag {f} {}", if v { "set" } else { "cleared" })
        }
    };
}

/// The free camera: moves and looks, then replaces the play camera's pose (after the play's
/// frame has set it).
pub fn free_camera(time: Res<Time>, keys: Res<ButtonInput<KeyCode>>, mouse: Res<ButtonInput<MouseButton>>, motion: Res<AccumulatedMouseMotion>, mut dev: ResMut<DevMenu>, mut cameras: Query<&mut Transform, With<Orbit>>) {
    let open = dev.open;
    let Some(fc) = dev.free.as_mut() else { return };
    let dt = time.delta_secs().min(0.1);
    if !open {
        let key = |k: &[KeyCode]| if keys.any_pressed(k.iter().copied()) { 1.0 } else { 0.0 };
        let turn = 1.6 * dt;
        fc.yaw += (key(&[KeyCode::KeyJ]) - key(&[KeyCode::KeyL])) * turn;
        fc.pitch += (key(&[KeyCode::KeyI]) - key(&[KeyCode::KeyK])) * turn;
        if mouse.pressed(MouseButton::Right) {
            fc.yaw -= motion.delta.x * 0.004;
            fc.pitch -= motion.delta.y * 0.004;
        }
        fc.pitch = fc.pitch.clamp(-1.5, 1.5);
        let speed = if keys.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight]) { 120.0 } else { 30.0 } * dt;
        let f = fc.forward();
        let right = Vec3::new(-fc.yaw.cos(), 0.0, fc.yaw.sin());
        let ahead = key(&[KeyCode::KeyW, KeyCode::ArrowUp]) - key(&[KeyCode::KeyS, KeyCode::ArrowDown]);
        let side = key(&[KeyCode::KeyD, KeyCode::ArrowRight]) - key(&[KeyCode::KeyA, KeyCode::ArrowLeft]);
        let up = key(&[KeyCode::KeyE]) - key(&[KeyCode::KeyQ]);
        fc.pos += (f * ahead + right * side + Vec3::Y * up) * speed;
    }
    let pose = Transform::from_translation(fc.pos).looking_to(fc.forward(), Vec3::Y);
    for mut t in &mut cameras {
        *t = pose;
    }
}

/// The panel's text: the entries with their values, the last result and the free camera's
/// keys.
pub fn show_panel(dev: Res<DevMenu>, play: Option<Res<PlayState>>, mut panels: Query<&mut Node, With<DevPanel>>, mut texts: Query<&mut Text, With<DevText>>) {
    let visible = dev.open || dev.free.is_some();
    for mut n in &mut panels {
        n.display = if visible { Display::Flex } else { Display::None };
    }
    if !visible {
        return;
    }
    let Some(play) = play else { return };
    let p = &play.play;
    let on = |b: bool| if b { "ON" } else { "off" };
    let mut s = String::new();
    if dev.open {
        s.push_str("DEV MENU (F1 closes) - a test tool of this build, not part of the game\n\n");
        for (k, e) in ENTRIES.iter().enumerate() {
            let line = match e {
                Entry::God => format!("God mode: {}", on(p.cheats.god)),
                Entry::Magic => format!("Infinite magic: {}", on(p.cheats.magic)),
                Entry::Charge => format!("Infinite Mote charge: {}", on(p.cheats.charge)),
                Entry::FreeCam => format!("Free camera (F2): {}", on(dev.free.is_some())),
                Entry::Neopoints => format!("Add Neopoints: < {} >  (have {})", AMOUNTS[dev.amount], p.state.neopoints),
                Entry::Heal => format!("Full heal  (hp {}/{}, mp {}/{})", p.hero_hp, p.hero_max, p.hero_mp, p.hero_mp_max),
                Entry::Motes => "Give every Mote".to_string(),
                Entry::Item => format!("Give item: < {} > {}", dev.item, p.item_label(dev.item)),
                Entry::Hero => format!("Switch hero  (now {})", if p.hero_index == 0 { "Tor" } else { "Roberta" }),
                Entry::Flag => format!("Toggle story flag: < {} > ({})", dev.flag, if p.state.flag(dev.flag) { "set" } else { "clear" }),
            };
            s.push_str(if k == dev.cursor { "> " } else { "  " });
            s.push_str(&line);
            s.push('\n');
        }
        s.push_str("\nUp/Down pick, Left/Right change (Shift: x10), Enter apply\n");
        if !dev.note.is_empty() {
            s.push_str(&format!("{}\n", dev.note));
        }
    }
    if let Some(fc) = dev.free {
        s.push_str(&format!(
            "FREE CAMERA (F2 ends) at ({:.1}, {:.1}, {:.1}): WASD move, E/Q up/down, J/L I/K or right-drag look, Shift fast",
            fc.pos.x, fc.pos.y, fc.pos.z
        ));
    }
    for mut t in &mut texts {
        *t = Text::new(s.clone());
    }
}
