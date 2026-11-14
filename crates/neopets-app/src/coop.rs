//! Local co-op (docs/MULTIPLAYER.md step 2; this build's feature, not in the original game):
//! player 2's controller, player 2's hero on screen, and the split screen.
//!
//! Joining: Start on a second controller (not the first one connected, which stays player 1's)
//! adds player 2 beside player 1 (`Play::add_player`: Roberta while she waits parked, else a
//! second Tor); Select on player 2's controller takes player 2 out. That controller then drives
//! only player 2. NEOPETS_COOP=1 adds player 2 when play starts (on the second controller when
//! there is one), for testing.
//!
//! Split screen: player 1's camera draws the top half of the scene image and a second camera,
//! following player 2's follow camera, the bottom half.
use bevy::camera::{RenderTarget, Viewport};
use bevy::prelude::*;
use neopets_sim::camera::CameraPad;
use neopets_sim::hero::HeroInput;

use crate::{bindings, gpu_skin, PlayState};

/// Player 2's controller and its input, built each frame; the jump press is kept until a tick
/// takes it.
#[derive(Resource, Default)]
pub struct Coop {
    pub pad: Option<Entity>,
    pub input: HeroInput,
    pub cam: CameraPad,
    pub jump: bool,
    pub attack: bool,
    /// Player 2's D-pad and Triangle presses for its quick-item and Mote panels.
    pub panel: neopets_sim::quick::PanelInput,
    /// Online (net.rs): player 2's input from the joiner's controller, in place of a local pad.
    pub remote: Option<(HeroInput, CameraPad)>,
    started: bool,
}

impl Coop {
    /// Player 2's input for one tick (the jump and attack presses are taken).
    pub fn take(&mut self) -> (HeroInput, CameraPad) {
        if let Some(r) = self.remote.take() {
            return r;
        }
        let mut i = self.input;
        i.jump = std::mem::take(&mut self.jump);
        i.attack = std::mem::take(&mut self.attack);
        (i, self.cam)
    }
}

/// Each frame: joining and leaving, and player 2's input from its controller.
#[allow(clippy::too_many_arguments)]
pub fn read_coop(gamepads: Query<(Entity, &Gamepad)>, keys: Res<bindings::Bindings>, mute: Res<bindings::PadMute>, mut coop: ResMut<Coop>, mut exclude: ResMut<bindings::PadExclude>, play: Option<ResMut<PlayState>>, settings: Res<crate::display::DisplaySettings>) {
    let Some(mut play) = play else {
        coop.pad = None;
        exclude.0 = None;
        return;
    };
    // The Multiplayer page's options.
    if (play.play.coop_revive, play.play.coop_partner) != (settings.coop_revive, settings.coop_partner) {
        (play.play.coop_revive, play.play.coop_partner) = (settings.coop_revive, settings.coop_partner);
    }
    let first = gamepads.iter().map(|(e, _)| e).min();
    if play.play.others.is_empty() {
        coop.pad = None;
    }
    if !coop.started && play.play.ticks > 0 {
        coop.started = true;
        if std::env::var_os("NEOPETS_COOP").is_some() && play.play.add_player() {
            coop.pad = gamepads.iter().map(|(e, _)| e).filter(|e| Some(*e) != first).min();
            info!("co-op: player 2 added at start (NEOPETS_COOP), on controller {:?} of {}", coop.pad, gamepads.iter().count());
        }
    }
    if coop.pad.is_none() && !mute.0 {
        if let Some((e, _)) = gamepads.iter().find(|(e, g)| Some(*e) != first && g.just_pressed(GamepadButton::Start)) {
            if play.play.add_player() {
                coop.pad = Some(e);
                info!("co-op: player 2 joined");
            }
        }
    } else if let Some(g) = coop.pad.and_then(|e| gamepads.get(e).ok()).map(|(_, g)| g) {
        if g.just_pressed(GamepadButton::Select) {
            play.play.remove_player();
            coop.pad = None;
            info!("co-op: player 2 left");
        }
    }
    exclude.0 = coop.pad;
    let Some(g) = coop.pad.and_then(|e| gamepads.get(e).ok()).map(|(_, g)| g) else {
        coop.input = HeroInput::default();
        coop.cam = CameraPad::default();
        // NEOPETS_COOP_WALK (diagnostic, with NEOPETS_COOP and no second controller): player 2
        // walks ahead, turning the camera slowly, and jumps every 1.5 seconds.
        if std::env::var_os("NEOPETS_COOP_WALK").is_some() && !play.play.others.is_empty() {
            coop.input.forward = 1.0;
            coop.cam.right_x = 170;
            coop.jump |= play.play.ticks % 90 == 45;
            coop.attack |= play.play.ticks % 90 == 20;
        }
        return;
    };
    let button = |a: usize| a.checked_sub(bindings::FIRST_BUTTON).and_then(|k| keys.pad[k]);
    let held = |a: usize| button(a).is_some_and(|b| g.pressed(b));
    let pressed = |a: usize| button(a).is_some_and(|b| g.just_pressed(b));
    let stick = g.left_stick();
    let (x, y) = (crate::controls::ps2_axis(stick.x), crate::controls::ps2_axis(stick.y));
    coop.input = HeroInput {
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
    coop.jump |= pressed(bindings::CROSS);
    coop.attack |= pressed(bindings::CIRCLE) || pressed(bindings::R2);
    coop.panel.up |= pressed(bindings::UP);
    coop.panel.left |= pressed(bindings::LEFT);
    coop.panel.right |= pressed(bindings::RIGHT);
    coop.panel.down |= pressed(bindings::DOWN);
    coop.panel.triangle |= pressed(bindings::TRIANGLE);
    let r = g.right_stick();
    let byte = |v: f32| (127.5 + 127.5 * v.clamp(-1.0, 1.0)).round() as u8;
    let invert = play.play.options.invert;
    coop.cam = CameraPad {
        right_x: if invert.0 { 255 - byte(r.x) } else { byte(r.x) },
        right_y: if invert.1 { 255 - byte(-r.y) } else { byte(-r.y) },
        r3_held: false,
        r3_pressed: false,
    };
}

/// An other player's drawn hero: its root, its GPU joints and the model it was built from.
pub struct Drawn {
    root: Entity,
    joints: Vec<Entity>,
    who: usize,
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
    while drawn.len() > others.len() || drawn.iter().zip(others).any(|(d, o)| d.who != o.who) {
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
            drawn.push(Drawn { root, joints, who: o.who });
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

/// Player 2's camera (split screen, bottom half) and the screen node that shows its picture.
#[derive(Component)]
pub struct Player2Cam;
#[derive(Component)]
pub struct Player2View;

/// Player 2's picture: its own image, the size it was made at and its colour-grade material.
#[derive(Default)]
pub struct Picture {
    image: Option<Handle<Image>>,
    size: (u32, u32),
    grade: Option<Handle<crate::graphics::SceneGrade>>,
}

/// Each frame: with player 2 in, player 1's camera draws the top half of the scene image, and a
/// second 3D camera following player 2's follow camera draws its own image, shown over the
/// bottom half of the game picture with the same colour grade. Its own image (not the scene
/// image's bottom half): Bevy's ambient occlusion read a camera drawn at an offset in the
/// target wrongly and left that half nearly black. Without player 2, player 1's camera draws
/// the whole image again.
#[allow(clippy::type_complexity, clippy::too_many_arguments)]
pub fn split_cameras(
    mut commands: Commands,
    play: Option<Res<PlayState>>,
    target: Res<crate::display::SceneTarget>,
    mut mains: Query<&mut Camera, (With<Camera3d>, With<crate::orbit::Orbit>, Without<Player2Cam>)>,
    mut seconds: Query<(Entity, &mut Transform), With<Player2Cam>>,
    scene_views: Query<&Node, (With<crate::display::SceneView>, Without<Player2View>)>,
    mut views: Query<(Entity, &mut Node), With<Player2View>>,
    mut images: ResMut<Assets<Image>>,
    mut grades: ResMut<Assets<crate::graphics::SceneGrade>>,
    mut pic: Local<Picture>,
    settings: Res<crate::display::DisplaySettings>,
) {
    // Online (net.rs) each computer shows one player full screen: no split.
    let other = play.as_ref().filter(|p| !p.play.net_follow && !p.play.coop_remote).and_then(|p| p.play.others.first());
    // The Multiplayer page's layout: one above the other, or side by side.
    let side = settings.coop_split == 1;
    let (w, h) = target.size;
    let Some(o) = other.filter(|_| w > 0 && h > 1) else {
        for (e, _) in &seconds {
            commands.entity(e).despawn();
        }
        for (e, _) in &views {
            commands.entity(e).despawn();
        }
        for mut c in &mut mains {
            if c.viewport.is_some() {
                c.viewport = None;
            }
        }
        return;
    };
    let half = if side { (w - w / 2, h) } else { (w, h - h / 2) };
    for mut c in &mut mains {
        let top = (UVec2::ZERO, if side { UVec2::new(w / 2, h) } else { UVec2::new(w, h / 2) });
        if c.viewport.as_ref().map(|v| (v.physical_position, v.physical_size)) != Some(top) {
            c.viewport = Some(Viewport { physical_position: top.0, physical_size: top.1, ..default() });
        }
    }
    // Player 2's image, remade when the scene image's size changes.
    let mut remade = false;
    if pic.image.is_none() || pic.size != half {
        let mut image = Image::new_target_texture(half.0, half.1, bevy::render::render_resource::TextureFormat::Rgba8UnormSrgb, None);
        image.sampler = bevy::image::ImageSampler::linear();
        let handle = images.add(image);
        if let Some(old) = pic.image.replace(handle.clone()) {
            images.remove(&old);
        }
        pic.size = half;
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
    let Some(image) = pic.image.clone() else { return };
    let eye = Vec3::from(o.camera.eye);
    let focus = Vec3::from(o.camera.focus);
    let want = Transform::from_translation(eye).looking_to(focus - eye, Vec3::Y);
    match seconds.single_mut() {
        Ok((e, mut t)) => {
            if *t != want {
                *t = want;
            }
            if remade {
                commands.entity(e).insert(RenderTarget::from(image));
            }
        }
        Err(_) => {
            let fov = 2.0 * (neopets_sim::camera::SCREEN.1 / (2.0 * neopets_sim::camera::FOCAL)).atan();
            commands.spawn((
                Camera3d::default(),
                Camera { order: -2, ..default() },
                RenderTarget::from(image),
                bevy::core_pipeline::tonemapping::Tonemapping::None,
                Projection::Perspective(PerspectiveProjection { fov, near: neopets_sim::camera::NEAR, far: 10_560.0, ..default() }),
                want,
                bevy::light::cluster::ClusterConfig::Single,
                bevy::light::ShadowFilteringMethod::Gaussian,
                Player2Cam,
            ));
        }
    }
    // The screen node: the bottom half of the game picture (the scene view's rectangle).
    let Some(scene) = scene_views.iter().next() else { return };
    let (Val::Px(left), Val::Px(top), Val::Px(width), Val::Px(height)) = (scene.left, scene.top, scene.width, scene.height) else { return };
    let want = if side {
        (Val::Px(left + width / 2.0), Val::Px(top), Val::Px(width / 2.0), Val::Px(height))
    } else {
        (Val::Px(left), Val::Px(top + height / 2.0), Val::Px(width), Val::Px(height / 2.0))
    };
    match views.single_mut() {
        Ok((_, mut n)) => {
            if (n.left, n.top, n.width, n.height) != want {
                (n.left, n.top, n.width, n.height) = want;
            }
        }
        Err(_) => {
            if let Some(g) = pic.grade.clone() {
                commands.spawn((
                    Node { position_type: PositionType::Absolute, left: want.0, top: want.1, width: want.2, height: want.3, ..default() },
                    bevy::ui_render::prelude::MaterialNode(g),
                    GlobalZIndex(i32::MIN + 2),
                    Player2View,
                ));
            }
        }
    }
}
