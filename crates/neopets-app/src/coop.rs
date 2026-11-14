//! Local co-op (docs/MULTIPLAYER.md step 2; this build's feature, not in the original game):
//! the other players' controllers, their heroes on screen, and the split screen.
//!
//! Starting: the Multiplayer page's Local co-op row (Cross) adds players 2 to 4 beside player 1
//! (`Play::add_player`: player 2 is Roberta while she waits parked, else each is another Tor), as
//! many as its Players row asks for, and Cross again takes them out. A controller press never
//! starts it (the user's choice). Each player's controller is chosen on its row (Auto: player 1
//! keeps the keyboard and the first free controller; players 2 to 4 take the next free ones in
//! the order they were connected). NEOPETS_COOP=1 starts it when play starts, for testing.
//!
//! Split screen: two players share the window top and bottom (or side by side), three have
//! player 1 on the top half and players 2 and 3 on the bottom quarters, four have a quarter
//! each. Player 1's camera draws its part of the scene image; each other player has its own
//! camera and image, shown over its part.
use bevy::camera::{RenderTarget, Viewport};
use bevy::prelude::*;
use neopets_sim::camera::CameraPad;
use neopets_sim::hero::HeroInput;

use crate::{bindings, gpu_skin, PlayState};

/// The most players on one computer.
pub const MAX_PLAYERS: usize = 4;

/// One other player's controller and its input, built each frame; the presses are kept until a
/// tick takes them.
#[derive(Default, Clone)]
pub struct Seat {
    pub pad: Option<Entity>,
    pub input: HeroInput,
    pub cam: CameraPad,
    pub jump: bool,
    pub attack: bool,
    /// The Square press (doors), kept for the next tick.
    pub square: bool,
    /// The D-pad and Triangle presses for its quick-item and Mote panels.
    pub panel: neopets_sim::quick::PanelInput,
}

/// Players 2 to 4 (seats 0 to 2).
#[derive(Resource, Default)]
pub struct Coop {
    pub seats: [Seat; MAX_PLAYERS - 1],
    /// A player's Start in play (its seat): the pause page opens for it (step_sim).
    pub pause: Option<usize>,
    /// Online (net.rs): each joiner's input from its controller (by seat), in place of a local
    /// pad.
    pub remote: [Option<(HeroInput, CameraPad)>; MAX_PLAYERS - 1],
    /// The Local co-op row's press: start (true) or stop (false).
    pub request: Option<bool>,
    /// Local co-op is on: the other players are kept in play (added again after a load).
    pub wanted: bool,
    started: bool,
}

impl Coop {
    /// Seat `i`'s input for one tick (the jump, attack and Square presses are taken).
    pub fn take(&mut self, i: usize) -> (HeroInput, CameraPad) {
        // Online: the joiner's latest input, kept for the next tick with its presses taken.
        if let Some(r) = self.remote.get_mut(i).and_then(Option::as_mut) {
            let out = *r;
            (r.0.jump, r.0.attack, r.0.square) = (false, false, false);
            return out;
        }
        let Some(s) = self.seats.get_mut(i) else { return Default::default() };
        let mut input = s.input;
        input.jump = std::mem::take(&mut s.jump);
        input.attack = std::mem::take(&mut s.attack);
        input.square = std::mem::take(&mut s.square);
        (input, s.cam)
    }

    /// Local co-op ends (going online): the seats let go of their controllers.
    pub fn release_local(&mut self) {
        self.wanted = false;
        for s in &mut self.seats {
            s.pad = None;
        }
    }
}

/// Each player's controller from the rows' choices (0 Auto, k controller k in connection order)
/// and the connected controllers in that order: player 1's (with Auto, the first controller no
/// one chose), then players 2 to 4's (with Auto, the next free ones in order).
pub fn assign<T: Copy + PartialEq>(connected: &[T], choice: [u8; MAX_PLAYERS], players: usize) -> [Option<T>; MAX_PLAYERS] {
    let n = players.min(MAX_PLAYERS);
    let mut out = [None; MAX_PLAYERS];
    let mut used: Vec<T> = Vec::new();
    for k in 0..n {
        if let Some(&e) = (choice[k] as usize).checked_sub(1).and_then(|c| connected.get(c)) {
            if !used.contains(&e) {
                out[k] = Some(e);
                used.push(e);
            }
        }
    }
    for k in 0..n {
        if choice[k] == 0 {
            if let Some(&e) = connected.iter().find(|e| !used.contains(e)) {
                out[k] = Some(e);
                used.push(e);
            }
        }
    }
    out
}

/// Player `view`'s part of the window with `players` on screen, in half-window units (x, y,
/// width, height out of 2 by 2); `side` is the two-player side-by-side layout.
pub fn view_rect(players: usize, side: bool, view: usize) -> Option<[u32; 4]> {
    match (players, view) {
        (0 | 1, 0) => Some([0, 0, 2, 2]),
        (2, 0) => Some(if side { [0, 0, 1, 2] } else { [0, 0, 2, 1] }),
        (2, 1) => Some(if side { [1, 0, 1, 2] } else { [0, 1, 2, 1] }),
        (3, 0) => Some([0, 0, 2, 1]),
        (3, 1) => Some([0, 1, 1, 1]),
        (3, 2) => Some([1, 1, 1, 1]),
        (4.., 0..=3) => Some([view as u32 % 2, view as u32 / 2, 1, 1]),
        _ => None,
    }
}

/// A part of the window in half units as pixels of a `w` by `h` picture: (x, y, width, height).
pub fn rect_pixels(r: [u32; 4], w: u32, h: u32) -> (u32, u32, u32, u32) {
    let (x0, y0) = (w * r[0] / 2, h * r[1] / 2);
    let (x1, y1) = (w * (r[0] + r[2]) / 2, h * (r[1] + r[3]) / 2);
    (x0, y0, x1 - x0, y1 - y0)
}

/// How many players share this window's screen (online, each computer shows one).
pub fn on_screen(p: &crate::play::Play) -> usize {
    if p.net_follow || p.coop_remote { 1 } else { (1 + p.others.len()).min(MAX_PLAYERS) }
}

/// Each frame: starting and stopping, each player's controller, and the other players' input.
#[allow(clippy::too_many_arguments)]
pub fn read_coop(
    gamepads: Query<(Entity, &Gamepad)>,
    keys: Res<bindings::Bindings>,
    mute: Res<bindings::PadMute>,
    mut coop: ResMut<Coop>,
    mut exclude: ResMut<bindings::PadExclude>,
    play: Option<ResMut<PlayState>>,
    settings: Res<crate::display::DisplaySettings>,
    mut dev: ResMut<crate::dev_menu::DevMenu>,
    menu: Option<Res<crate::menu_view::Menu>>,
    net: Res<crate::net::Net>,
) {
    let in_play = menu.as_ref().is_none_or(|m| m.flow.current == crate::menus::PLAY);
    let mut connected: Vec<Entity> = gamepads.iter().map(|(e, _)| e).collect();
    connected.sort();
    // NEOPETS_COOP=n (diagnostic, 2 to 4): that many players, whatever the Players row says.
    let players = std::env::var("NEOPETS_COOP").ok().and_then(|v| v.trim().parse::<usize>().ok()).filter(|n| (2..=MAX_PLAYERS).contains(n)).unwrap_or(settings.coop_players).clamp(2, MAX_PLAYERS);
    let pads = assign(&connected, settings.coop_pads, players);
    let online = net.role != crate::net::Role::Off;
    if let Some(on) = coop.request.take() {
        if on && online {
            dev.toast = Some(("Local co-op is off while playing online.".into(), 240));
        } else {
            coop.wanted = on;
            dev.toast = Some((if on { format!("Local co-op: {players} players.") } else { "Local co-op stopped.".into() }, 240));
            info!("co-op: local co-op {}", if on { "started" } else { "stopped" });
        }
    }
    // NEOPETS_COOP=1 (diagnostic): local co-op on when play starts.
    if !coop.started && play.as_ref().is_some_and(|p| p.play.ticks > 0) {
        coop.started = true;
        if std::env::var_os("NEOPETS_COOP").is_some() {
            coop.wanted = true;
            info!("co-op: local co-op started (NEOPETS_COOP), {players} players, {} controllers", connected.len());
        }
    }
    let Some(mut play) = play else {
        for s in &mut coop.seats {
            s.pad = None;
        }
        *exclude = bindings::PadExclude::default();
        return;
    };
    // The Multiplayer page's options.
    // NEOPETS_NET_ALONE=1 or 0 (diagnostic): Doors take Only you or Both, whatever the settings
    // file says.
    let alone = std::env::var("NEOPETS_NET_ALONE").ok().map_or(settings.coop_doors_alone, |v| v.trim() != "0");
    if (play.play.coop_revive, play.play.coop_partner, play.play.coop_together, play.play.coop_doors_alone) != (settings.coop_revive, settings.coop_partner, settings.coop_together, alone) {
        (play.play.coop_revive, play.play.coop_partner, play.play.coop_together, play.play.coop_doors_alone) = (settings.coop_revive, settings.coop_partner, settings.coop_together, alone);
    }
    // Online, the other player is the joiner's: local co-op ends (its players go).
    let remote = play.play.net_follow || play.play.coop_remote;
    if online && coop.wanted {
        coop.release_local();
        if !remote {
            play.play.remove_player();
        }
        info!("co-op: local co-op ended for online play");
    }
    // The players asked for: added beside player 1 once play runs (and again after a new game
    // or load), and taken out when stopped or fewer are asked for.
    if !remote && !online && play.play.ticks > 0 {
        let want = if coop.wanted { players - 1 } else { 0 };
        while play.play.others.len() < want {
            if !play.play.add_player() {
                break;
            }
            info!("co-op: player {} joined", play.play.others.len() + 1);
        }
        if play.play.others.len() > want {
            play.play.keep_players(want);
            info!("co-op: {} players now", want + 1);
        }
    }
    let local = if remote { 0 } else { play.play.others.len() };
    for (i, s) in coop.seats.iter_mut().enumerate() {
        s.pad = if i < local { pads[i + 1] } else { None };
    }
    // In a menu every controller drives it; in play each other player's controller is its own
    // alone, and player 1 has the keyboard and its chosen controller (on Auto, every one left).
    *exclude = if in_play {
        bindings::PadExclude { only: pads[0].filter(|_| settings.coop_pads[0] != 0), others: coop.seats.iter().filter_map(|s| s.pad).collect() }
    } else {
        bindings::PadExclude::default()
    };
    let invert = play.play.options.invert;
    let walk = std::env::var_os("NEOPETS_COOP_WALK").is_some();
    let ticks = play.play.ticks;
    for i in 0..coop.seats.len() {
        let pad = coop.seats[i].pad.and_then(|e| gamepads.get(e).ok()).map(|(_, g)| g);
        let Some(g) = pad else {
            let s = &mut coop.seats[i];
            s.input = HeroInput::default();
            s.cam = CameraPad::default();
            // NEOPETS_COOP_WALK (diagnostic, with NEOPETS_COOP): a player with no controller
            // walks ahead, turning its camera slowly, and jumps every 1.5 seconds.
            if walk && i < local {
                s.input.forward = 1.0;
                s.cam.right_x = 170;
                s.jump |= ticks % 90 == 45;
                s.attack |= ticks % 90 == 20;
            }
            continue;
        };
        // In play: Start opens the pause page for this player.
        if in_play && !mute.0 && g.just_pressed(GamepadButton::Start) {
            coop.pause = Some(i);
        }
        let s = &mut coop.seats[i];
        let button = |a: usize| a.checked_sub(bindings::FIRST_BUTTON).and_then(|k| keys.pad[k]);
        let held = |a: usize| button(a).is_some_and(|b| g.pressed(b));
        let pressed = |a: usize| button(a).is_some_and(|b| g.just_pressed(b));
        let stick = g.left_stick();
        let (x, y) = (crate::controls::ps2_axis(stick.x), crate::controls::ps2_axis(stick.y));
        s.input = HeroInput {
            side: -x,
            forward: y,
            run: held(bindings::R1),
            square_held: held(bindings::SQUARE),
            cross_held: held(bindings::CROSS),
            guard: held(bindings::L1),
            attack_held: held(bindings::CIRCLE) || held(bindings::R2),
            r2_held: held(bindings::R2),
            circle_held: held(bindings::CIRCLE),
            l2: held(bindings::L2),
            ..Default::default()
        };
        s.jump |= pressed(bindings::CROSS);
        s.attack |= pressed(bindings::CIRCLE) || pressed(bindings::R2);
        s.square |= pressed(bindings::SQUARE);
        s.panel.up |= pressed(bindings::UP);
        s.panel.left |= pressed(bindings::LEFT);
        s.panel.right |= pressed(bindings::RIGHT);
        s.panel.down |= pressed(bindings::DOWN);
        s.panel.triangle |= pressed(bindings::TRIANGLE);
        let r = g.right_stick();
        let byte = |v: f32| (127.5 + 127.5 * v.clamp(-1.0, 1.0)).round() as u8;
        s.cam = CameraPad {
            right_x: if invert.0 { 255 - byte(r.x) } else { byte(r.x) },
            right_y: if invert.1 { 255 - byte(-r.y) } else { byte(-r.y) },
            r3_held: false,
            r3_pressed: false,
        };
    }
}

/// An other player's drawn hero: its root, its GPU joints and the model it was built from.
pub struct Drawn {
    root: Entity,
    joints: Vec<Entity>,
    who: usize,
    outfit: u32,
}

#[derive(Component)]
pub struct OtherHero;

/// Each frame: the other players' heroes, spawned from their models and skinned on the GPU from
/// their animation (as player 1's hero, `play_frame`), lit and tinted at their own place.
#[allow(clippy::too_many_arguments)]
pub fn draw_others(
    mut commands: Commands,
    play: Option<Res<PlayState>>,
    mut drawn: Local<Vec<Drawn>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut pass1_materials: ResMut<Assets<crate::pass1::Pass1Material>>,
    mut images: ResMut<Assets<Image>>,
    skin_assets: Res<gpu_skin::SkinAssets>,
    mut joint_query: Query<&mut GlobalTransform, With<gpu_skin::SkinJoint>>,
    mut roots: Query<&mut Transform, With<OtherHero>>,
) {
    let others = play.as_ref().map_or(&[][..], |p| &p.play.others[..]);
    // Gone or changed: despawned.
    while drawn.len() > others.len() || drawn.iter().zip(others).any(|(d, o)| d.who != o.who || d.outfit != o.outfit) {
        if let Some(d) = drawn.pop() {
            commands.entity(d.root).despawn();
        }
    }
    let Some(play) = play else { return };
    let p = &play.play;
    for (i, o) in p.others.iter().enumerate() {
        let at = o.loco.position();
        let f = o.loco.facing;
        let rotation = Quat::from_rotation_y(f[0].atan2(f[2]));
        if drawn.len() <= i {
            // The hero's program-6 parts are drawn solid, as player 1's (`play_frame`).
            let mut model = o.model.clone();
            for part in model.parts.iter_mut().filter(|q| q.program == Some(6) || (!q.colors.is_empty() && q.colors.iter().all(|c| c[3] < 0.5))) {
                for c in &mut part.colors {
                    c[3] = 1.0;
                }
            }
            let parts = crate::part_assets(&model, &mut meshes, &mut materials, &mut images);
            let second = crate::pass1_assets(&model, &mut meshes, &mut pass1_materials, &mut images);
            let bones = o.hero.skin_matrices().len();
            for (k, mesh) in second.iter().map(|(k, m, _, _)| (*k, m)).chain(parts.iter().map(|(k, m, _)| (*k, m))) {
                if !model.parts[k].weights.is_empty() {
                    if let Some(mut m) = meshes.get_mut(mesh) {
                        gpu_skin::prepare_mesh(&mut m, &model.parts[k], bones);
                    }
                }
            }
            let mut joints = Vec::new();
            let root = commands
                .spawn((Transform::from_translation(Vec3::from(at)).with_rotation(rotation), Visibility::Visible, OtherHero))
                .with_children(|c| {
                    joints = (0..bones + gpu_skin::DATA_JOINTS + 1).map(|_| c.spawn((GlobalTransform::IDENTITY, gpu_skin::SkinJoint)).id()).collect();
                    let skin = |k: usize| {
                        (!model.parts[k].weights.is_empty()).then(|| (bevy::mesh::skinning::SkinnedMesh { inverse_bindposes: skin_assets.bindposes.clone(), joints: joints.clone() }, bevy::camera::visibility::NoFrustumCulling))
                    };
                    for (k, mesh, material, scroll) in second {
                        let mut e = c.spawn((Mesh3d(mesh), MeshMaterial3d(material), Transform::IDENTITY));
                        if let Some(s) = scroll {
                            e.insert(crate::pass1::Pass1Scroll(s));
                        }
                        if let Some(s) = skin(k) {
                            e.insert(s);
                        }
                    }
                    for (k, mesh, material) in parts {
                        let mut e = c.spawn((Mesh3d(mesh), MeshMaterial3d(material), Transform::IDENTITY));
                        if let Some(s) = skin(k) {
                            e.insert(s);
                        }
                    }
                })
                .id();
            drawn.push(Drawn { root, joints, who: o.who, outfit: o.outfit });
        }
        let d = &drawn[i];
        if let Ok(mut t) = roots.get_mut(d.root) {
            let want = Transform::from_translation(Vec3::from(at)).with_rotation(rotation);
            if *t != want {
                *t = want;
            }
        }
        let lights = p.lights(None, at);
        let tint = p.draw_tint(None, at);
        let look = gpu_skin::Look { light_rotation: rotation, lights: &lights, tint: tint.unwrap_or([1.0; 3]), slots: if o.hero_flash > 0 { &crate::play::FLASH_COLOUR } else { &[[1.0; 3]; 15] }, ghost: 1.0 };
        let root = Transform::from_translation(Vec3::from(at)).with_rotation(rotation).compute_affine();
        for (e, m) in d.joints.iter().zip(gpu_skin::joint_matrices(&root, &o.hero.skin_matrices(), &look)) {
            if let Ok(mut g) = joint_query.get_mut(*e) {
                if *g != m {
                    *g = m;
                }
            }
        }
    }
}

/// Another player's camera (split screen; its seat) and the screen node that shows its picture.
#[derive(Component)]
pub struct PlayerCam(usize);
#[derive(Component)]
pub struct PlayerView(usize);

/// Another player's picture: its own image, the size it was made at and its colour-grade
/// material.
#[derive(Default)]
pub struct Picture {
    image: Option<Handle<Image>>,
    size: (u32, u32),
    grade: Option<Handle<crate::graphics::SceneGrade>>,
}

/// Each frame: with other players on this screen, player 1's camera draws its part of the scene
/// image (`view_rect`), and each other player's 3D camera, following that player's follow
/// camera, draws its own image, shown over its part of the game picture with the same colour
/// grade. Their own images (not parts of the scene image): Bevy's ambient occlusion read a camera
/// drawn at an offset in the target wrongly and left that part nearly black. With player 1
/// alone, its camera draws the whole image again.
#[allow(clippy::type_complexity, clippy::too_many_arguments)]
pub fn split_cameras(
    mut commands: Commands,
    play: Option<Res<PlayState>>,
    target: Res<crate::display::SceneTarget>,
    mut mains: Query<&mut Camera, (With<Camera3d>, With<crate::orbit::Orbit>, Without<PlayerCam>)>,
    mut seconds: Query<(Entity, &PlayerCam, &mut Transform)>,
    scene_views: Query<&Node, (With<crate::display::SceneView>, Without<PlayerView>)>,
    mut views: Query<(Entity, &PlayerView, &mut Node)>,
    mut images: ResMut<Assets<Image>>,
    mut grades: ResMut<Assets<crate::graphics::SceneGrade>>,
    mut pics: Local<[Picture; MAX_PLAYERS - 1]>,
    settings: Res<crate::display::DisplaySettings>,
) {
    let (w, h) = target.size;
    // Online (net.rs) each computer shows one player full screen: no split.
    let players = if w > 1 && h > 1 { play.as_ref().map_or(1, |p| on_screen(&p.play)) } else { 1 };
    let side = settings.coop_split == 1;
    // Cameras and nodes of players no longer on screen go.
    for (e, c, _) in &seconds {
        if c.0 + 1 >= players {
            commands.entity(e).despawn();
        }
    }
    for (e, v, _) in &views {
        if v.0 + 1 >= players {
            commands.entity(e).despawn();
        }
    }
    let main = (players > 1).then(|| view_rect(players, side, 0)).flatten().map(|r| rect_pixels(r, w, h));
    for mut c in &mut mains {
        let want = main.map(|(x, y, pw, ph)| (UVec2::new(x, y), UVec2::new(pw, ph)));
        if c.viewport.as_ref().map(|v| (v.physical_position, v.physical_size)) != want {
            c.viewport = want.map(|(p, s)| Viewport { physical_position: p, physical_size: s, ..default() });
        }
    }
    let Some(play) = play.filter(|_| players > 1) else { return };
    let scene = scene_views.iter().next().and_then(|n| match (n.left, n.top, n.width, n.height) {
        (Val::Px(l), Val::Px(t), Val::Px(sw), Val::Px(sh)) => Some((l, t, sw, sh)),
        _ => None,
    });
    for (i, o) in play.play.others.iter().enumerate().take(players - 1) {
        let Some(r) = view_rect(players, side, i + 1) else { continue };
        let (_, _, pw, ph) = rect_pixels(r, w, h);
        let size = (pw.max(1), ph.max(1));
        let pic = &mut pics[i];
        // Its image, remade when its size changes.
        let mut remade = false;
        if pic.image.is_none() || pic.size != size {
            let mut image = Image::new_target_texture(size.0, size.1, bevy::render::render_resource::TextureFormat::Rgba8UnormSrgb, None);
            image.sampler = bevy::image::ImageSampler::linear();
            let handle = images.add(image);
            if let Some(old) = pic.image.replace(handle.clone()) {
                images.remove(&old);
            }
            pic.size = size;
            remade = true;
            let existing = pic.grade.clone().filter(|g| grades.get(g).is_some());
            match existing {
                Some(g) => {
                    if let Some(mut m) = grades.get_mut(&g) {
                        m.image = handle;
                    }
                }
                None => {
                    let grade = grades.iter().next().map(|(_, g)| g.grade).unwrap_or_default();
                    pic.grade = Some(grades.add(crate::graphics::SceneGrade { grade, image: handle }));
                }
            }
        }
        let Some(image) = pic.image.clone() else { continue };
        let eye = Vec3::from(o.camera.eye);
        let focus = Vec3::from(o.camera.focus);
        let want = Transform::from_translation(eye).looking_to(focus - eye, Vec3::Y);
        match seconds.iter_mut().find(|(_, c, _)| c.0 == i) {
            Some((e, _, mut t)) => {
                if *t != want {
                    *t = want;
                }
                if remade {
                    commands.entity(e).insert(RenderTarget::from(image));
                }
            }
            None => {
                let fov = 2.0 * (neopets_sim::camera::SCREEN.1 / (2.0 * neopets_sim::camera::FOCAL)).atan();
                commands.spawn((
                    Camera3d::default(),
                    Camera { order: -2 - i as isize, ..default() },
                    RenderTarget::from(image),
                    bevy::core_pipeline::tonemapping::Tonemapping::None,
                    Projection::Perspective(PerspectiveProjection { fov, near: neopets_sim::camera::NEAR, far: 10_560.0, ..default() }),
                    want,
                    bevy::light::cluster::ClusterConfig::Single,
                    bevy::light::ShadowFilteringMethod::Gaussian,
                    PlayerCam(i),
                ));
            }
        }
        // The screen node: this player's part of the game picture (the scene view's rectangle).
        let Some((left, top, width, height)) = scene else { continue };
        let at = |k: u32, size: f32| size * k as f32 / 2.0;
        let want = (Val::Px(left + at(r[0], width)), Val::Px(top + at(r[1], height)), Val::Px(at(r[2], width)), Val::Px(at(r[3], height)));
        match views.iter_mut().find(|(_, v, _)| v.0 == i) {
            Some((_, _, mut n)) => {
                if (n.left, n.top, n.width, n.height) != want {
                    (n.left, n.top, n.width, n.height) = want;
                }
            }
            None => {
                if let Some(g) = pic.grade.clone() {
                    commands.spawn((
                        Node { position_type: PositionType::Absolute, left: want.0, top: want.1, width: want.2, height: want.3, ..default() },
                        bevy::ui_render::prelude::MaterialNode(g),
                        GlobalZIndex(i32::MIN + 2),
                        PlayerView(i),
                    ));
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn controllers_by_choice_and_auto() {
        // Auto everywhere: player 1 the first, then the next in order.
        assert_eq!(assign(&[10, 11, 12, 13], [0; 4], 4), [Some(10), Some(11), Some(12), Some(13)]);
        // Two players: the rest are not given out.
        assert_eq!(assign(&[10, 11, 12], [0; 4], 2), [Some(10), Some(11), None, None]);
        // Player 2 on controller 1: player 1 (Auto) takes the next free one.
        assert_eq!(assign(&[10, 11, 12], [0, 1, 0, 0], 3), [Some(11), Some(10), Some(12), None]);
        // A choice past the connected ones is none; the same controller twice goes to the first.
        assert_eq!(assign(&[10, 11], [0, 4, 1, 0], 3), [Some(11), None, Some(10), None]);
        // Fewer controllers than players: the last ones have none.
        assert_eq!(assign(&[10], [0; 4], 4), [Some(10), None, None, None]);
    }

    #[test]
    fn screen_parts() {
        assert_eq!(view_rect(1, false, 0), Some([0, 0, 2, 2]));
        assert_eq!(view_rect(2, false, 1), Some([0, 1, 2, 1]));
        assert_eq!(view_rect(2, true, 1), Some([1, 0, 1, 2]));
        assert_eq!(view_rect(3, false, 0), Some([0, 0, 2, 1]));
        assert_eq!(view_rect(3, true, 2), Some([1, 1, 1, 1]));
        assert_eq!(view_rect(4, false, 3), Some([1, 1, 1, 1]));
        assert_eq!(view_rect(4, false, 4), None);
        assert_eq!(view_rect(2, false, 2), None);
        // An odd size: the parts meet with no gap.
        assert_eq!(rect_pixels([0, 0, 1, 1], 1001, 601), (0, 0, 500, 300));
        assert_eq!(rect_pixels([1, 1, 1, 1], 1001, 601), (500, 300, 501, 301));
    }
}
