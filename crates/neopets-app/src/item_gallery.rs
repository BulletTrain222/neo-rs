//! The item gallery: a test tool of this build, not part of the original game. Every item id with a name or a model, on grid
//! pages in front of a plain wall, each turning slowly and labelled, so that a person can look for missing textures.
//!
//! Models come from the disc at run time: resource 7 and the fixed item resources, resource 9, then the first world pack that
//! has a type-8 child under the id (SPEC-0021 Item model registry). Second-pass layers are not drawn, as for a world item.
//! F1's "Item gallery" opens it; PageUp and PageDown change the page and Home closes it. Each opening writes
//! private/items/gallery.txt, a line per item.

use std::collections::HashMap;

use bevy::prelude::*;
use neopets_data::model::ModelData;

use crate::dev_menu::DevMenu;
use crate::PlayState;

const COLUMNS: usize = 8;
const ROWS: usize = 4;
const SPACING: f32 = 5.0;
/// The longest side of a model on the page.
const FIT: f32 = 3.0;

#[derive(Clone)]
struct Row {
    id: u16,
    name: String,
    source: String,
    /// Parts and those with no texture; None with no model found.
    parts: Option<(usize, usize)>,
}

#[derive(Resource, Default)]
pub struct Gallery {
    rows: Vec<Row>,
    built: bool,
    page: usize,
    centre: Vec3,
    speed: usize,
    shown: bool,
}

#[derive(Component)]
pub struct GalleryPart;

#[derive(Component)]
pub struct GalleryItem(usize);

#[derive(Component)]
pub struct GalleryLabel(usize);

#[derive(Component)]
pub struct GalleryTitle;

/// Every model found, by item id, with where it came from.
fn find_models(archive: &mut crate::Archive, packs: &[u32]) -> HashMap<u16, (ModelData, String)> {
    let crate::Archive { disc, index } = archive;
    let mut out: HashMap<u16, (ModelData, String)> = HashMap::new();
    let mut boot = None;
    for id in (50..=59).chain(98..=190) {
        if let Ok(Some(m)) = crate::item_model(disc, index, &mut boot, id, 0) {
            out.insert(id as u16, (m, "fixed".into()));
        }
    }
    let mut scan = |out: &mut HashMap<u16, (ModelData, String)>, resource: u32, base: u16, source: String| {
        let Ok(bytes) = index.read(disc, resource) else { return };
        let Ok(chain) = neopets_data::resource::walk_chunks(&bytes, None) else { return };
        let (trees, _) = neopets_data::model::find_trees(&bytes, &chain.chunks);
        let mut roles: Vec<u16> = trees.iter().map(|t| t.role_id).collect();
        roles.sort_unstable();
        roles.dedup();
        for role in roles {
            let id = role.saturating_add(base);
            if id > 322 || out.contains_key(&id) {
                continue;
            }
            if let Ok(m) = neopets_data::model::build_roles(&bytes, &chain.chunks, None, Some(&[role])) {
                if !m.parts.is_empty() {
                    out.insert(id, (m, source.clone()));
                }
            }
        }
    };
    scan(&mut out, 9, 283, "resource 9".into());
    // The fixed and world-item slots' resources register under their own ids, the Rope Gun's among them.
    for resource in 565..=575 {
        scan(&mut out, resource, 0, format!("resource {resource}"));
    }
    for &pack in packs {
        scan(&mut out, pack, 0, format!("world {pack}"));
    }
    out
}

fn fit_scale(m: &ModelData) -> (f32, Vec3) {
    let (lo, hi) = (Vec3::from(m.min), Vec3::from(m.max));
    let size = (hi - lo).max_element().max(1e-3);
    (FIT / size, (lo + hi) * 0.5)
}

fn cell(centre: Vec3, k: usize) -> Vec3 {
    let k = k % (COLUMNS * ROWS);
    let (col, row) = ((k % COLUMNS) as f32, (k / COLUMNS) as f32);
    // The camera looks along +Z, so its right is -X.
    centre + Vec3::new(((COLUMNS - 1) as f32 / 2.0 - col) * SPACING, ((ROWS - 1) as f32 / 2.0 - row) * SPACING, 0.0)
}

#[allow(clippy::too_many_arguments, clippy::type_complexity)]
pub fn step(
    mut commands: Commands,
    mut gallery: ResMut<Gallery>,
    mut dev: ResMut<DevMenu>,
    state: Option<Res<PlayState>>,
    mut archive: Option<ResMut<crate::Archive>>,
    keys: Res<ButtonInput<KeyCode>>,
    time: Res<Time<bevy::time::Real>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
    mut items: Query<(&GalleryItem, &mut Transform, &mut Visibility), (Without<GalleryPart>, Without<GalleryLabel>, Without<GalleryTitle>)>,
    mut labels: Query<(&GalleryLabel, &mut Node, &mut Visibility), (Without<GalleryItem>, Without<GalleryPart>, Without<GalleryTitle>)>,
    mut title: Query<(&mut Text, &mut Visibility), (With<GalleryTitle>, Without<GalleryItem>, Without<GalleryLabel>, Without<GalleryPart>)>,
    mut parts: Query<&mut Visibility, (With<GalleryPart>, Without<GalleryItem>, Without<GalleryLabel>, Without<GalleryTitle>)>,
    cameras: Query<(&Camera, &GlobalTransform), (With<Camera3d>, Without<crate::water_reflect::MirrorCam>)>,
    windows: Query<&Window>,
) {
    let open = dev.debug.gallery;
    if !open {
        if gallery.shown {
            gallery.shown = false;
            dev.debug.hide_hud = false;
            dev.gallery_end(gallery.speed);
            for (_, _, mut v) in &mut items {
                *v = Visibility::Hidden;
            }
            for (_, _, mut v) in &mut labels {
                *v = Visibility::Hidden;
            }
            for (_, mut v) in &mut title {
                *v = Visibility::Hidden;
            }
            for mut v in &mut parts {
                *v = Visibility::Hidden;
            }
        }
        return;
    }
    let Some(state) = state else { return };
    if !gallery.built {
        let Some(archive) = archive.as_deref_mut() else { return };
        gallery.built = true;
        let mut packs: Vec<u32> = state.play.world_keys.0.iter().copied().filter(|&k| k > 0).collect();
        packs.sort_unstable();
        packs.dedup();
        packs.retain(|&p| p != state.play.world.pack);
        packs.insert(0, state.play.world.pack);
        let found = find_models(archive, &packs);
        gallery.centre = Vec3::from(state.play.player.loco.position()) + Vec3::new(0.0, 1500.0, 0.0);
        let mut report = String::new();
        let mut rows = Vec::new();
        for id in 0..=322u16 {
            let name = state.play.debug_item_name(id);
            let model = found.get(&id);
            if name.is_none() && model.is_none() {
                continue;
            }
            let row = Row {
                id,
                name: name.unwrap_or_else(|| "(no name)".into()),
                source: model.map_or("none".into(), |m| m.1.clone()),
                parts: model.map(|(m, _)| (m.parts.len(), m.parts.iter().filter(|p| p.texture.is_none()).count())),
            };
            report.push_str(&format!("{}\t{}\t{}\t{}\n", row.id, row.name, row.source, row.parts.map_or("NO MODEL".to_string(), |(n, t)| format!("{n} parts, {t} with no texture"))));
            rows.push(row);
        }
        let _ = std::fs::create_dir_all("private/items");
        let _ = std::fs::write("private/items/gallery.txt", report);
        // The wall behind, and one spot behind the labels.
        let wall = materials.add(StandardMaterial { base_color: Color::srgb(0.32, 0.34, 0.38), unlit: true, ..default() });
        let plane = meshes.add(Cuboid::new(400.0, 400.0, 1.0));
        commands.spawn((GalleryPart, Mesh3d(plane), MeshMaterial3d(wall), Transform::from_translation(gallery.centre + Vec3::new(0.0, 0.0, 8.0)), Visibility::Hidden, bevy::light::NotShadowCaster));
        for (k, row) in rows.iter().enumerate() {
            let at = cell(gallery.centre, k);
            if let Some((m, _)) = found.get(&row.id) {
                let (s, c) = fit_scale(m);
                let list = crate::part_assets(m, &mut meshes, &mut materials, &mut images);
                commands.spawn((GalleryItem(k), Transform::from_translation(at).with_scale(Vec3::splat(s)), Visibility::Hidden)).with_children(|ch| {
                    for (_, mesh, material) in list {
                        ch.spawn((Mesh3d(mesh), MeshMaterial3d(material), Transform::from_translation(-c), bevy::light::NotShadowCaster));
                    }
                });
            }
            let warn = match row.parts {
                None => Some(Color::srgb(1.0, 0.45, 0.45)),
                Some((_, t)) if t > 0 => Some(Color::srgb(1.0, 0.75, 0.3)),
                _ => None,
            };
            let note = match row.parts {
                None => "\nNO MODEL".to_string(),
                Some((n, t)) if t > 0 => format!("\n{t} of {n} untextured"),
                _ => String::new(),
            };
            commands.spawn((
                GalleryLabel(k),
                Text::new(format!("{} {}{note}", row.id, row.name)),
                TextFont { font_size: bevy::text::FontSize::Px(10.0), ..default() },
                TextColor(warn.unwrap_or(Color::WHITE)),
                TextLayout { justify: bevy::text::Justify::Center, ..default() },
                Node { position_type: PositionType::Absolute, width: px(190), ..default() },
                GlobalZIndex(4800),
                Visibility::Hidden,
            ));
        }
        commands.spawn((
            GalleryTitle,
            Text::new(""),
            TextFont { font_size: bevy::text::FontSize::Px(10.0), ..default() },
            TextColor(Color::srgb(1.0, 0.9, 0.5)),
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.65)),
            Node { position_type: PositionType::Absolute, left: px(8), top: px(8), padding: UiRect::all(px(5)), ..default() },
            GlobalZIndex(4900),
            Visibility::Hidden,
        ));
        gallery.rows = rows;
        gallery.page = std::env::var("NEOPETS_GALLERY_PAGE").ok().and_then(|v| v.parse().ok()).unwrap_or(0);
        return;
    }
    if !gallery.shown {
        gallery.shown = true;
        gallery.speed = dev.gallery_begin(gallery.centre + Vec3::new(0.0, 0.0, -26.0));
        dev.debug.hide_hud = true;
        for mut v in &mut parts {
            *v = Visibility::Inherited;
        }
    }
    let pages = gallery.rows.len().div_ceil(COLUMNS * ROWS).max(1);
    if keys.just_pressed(KeyCode::PageDown) {
        gallery.page = (gallery.page + 1) % pages;
    }
    if keys.just_pressed(KeyCode::PageUp) {
        gallery.page = (gallery.page + pages - 1) % pages;
    }
    if keys.just_pressed(KeyCode::Home) {
        dev.debug.gallery = false;
        return;
    }
    let page = gallery.page.min(pages - 1);
    let on_page = |k: usize| k / (COLUMNS * ROWS) == page;
    let angle = time.elapsed_secs() * 0.6;
    for (item, mut t, mut v) in &mut items {
        let want = if on_page(item.0) { Visibility::Visible } else { Visibility::Hidden };
        if *v != want {
            *v = want;
        }
        t.rotation = Quat::from_rotation_y(angle);
    }
    let cam = cameras.iter().next();
    for (label, mut node, mut v) in &mut labels {
        // As fractions of the camera's own viewport (it can be a render target of another size), then of the window.
        let at = cam.zip(windows.iter().next()).and_then(|((c, g), w)| {
            let p = c.world_to_viewport(g, cell(gallery.centre, label.0) + Vec3::new(0.0, -FIT * 0.55, 0.0)).ok()?;
            Some(p / c.logical_viewport_size()? * Vec2::new(w.width(), w.height()))
        });
        match at.filter(|_| on_page(label.0)) {
            Some(p) => {
                node.left = px(p.x - 95.0);
                node.top = px(p.y);
                *v = Visibility::Inherited;
            }
            None => *v = Visibility::Hidden,
        }
    }
    let missing = gallery.rows.iter().filter(|r| r.parts.is_none()).count();
    let bare = gallery.rows.iter().filter(|r| r.parts.is_some_and(|p| p.1 > 0)).count();
    for (mut text, mut v) in &mut title {
        text.0 = format!("Item gallery, page {} of {}: {} items, {missing} with no model, {bare} with an untextured part. PageUp and PageDown change page, Home closes", page + 1, pages, gallery.rows.len());
        *v = Visibility::Inherited;
    }
}
