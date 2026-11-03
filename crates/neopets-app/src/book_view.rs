//! The book reader's 3D parts (SPEC-0035 section 6.1 rules 11, 12 and 14): the flat page image on the curved page
//! sheet, and the book's pictures on the decoration mesh, tilted by the page timer and held in the camera's frame.
//! They draw with their own camera over the world, as the original turns the depth test off for them.

use bevy::camera::visibility::RenderLayers;
use bevy::mesh::{Indices, PrimitiveTopology};
use bevy::asset::RenderAssetUsages;
use bevy::camera::RenderTarget;
use bevy::prelude::*;
use neopets_data::model::ModelData;

use crate::menu_view::Menu;

const BOOK_LAYER: usize = 1;

#[derive(Component)]
pub struct BookCamera;

#[derive(Component)]
pub struct BookPart;

/// The book the parts were built for, and its index.
#[derive(Resource, Default)]
pub struct BookView {
    index: Option<usize>,
}

/// The book's camera exists only while a book is open: other systems take the first 3D camera they find as the view
/// (the effects' sprites, the item popups), so an idle one would turn them all to its own frame.
fn spawn_camera(commands: &mut Commands, target: Option<RenderTarget>, msaa: Msaa) {
    let mut e = commands.spawn((
        Camera3d::default(),
        Camera { order: 1, clear_color: ClearColorConfig::None, ..default() },
        bevy::core_pipeline::tonemapping::Tonemapping::None,
        Projection::Perspective(PerspectiveProjection { near: 0.02, far: 100.0, ..default() }),
        RenderLayers::layer(BOOK_LAYER),
        msaa,
        BookCamera,
    ));
    if let Some(t) = target {
        e.insert(t);
    }
}

fn part_mesh(part: &neopets_data::model::MeshPart) -> Mesh {
    let g = crate::gs_unit(part);
    let colors: Vec<[f32; 4]> = part.colors.iter().map(|c| [(c[0] * g).powf(2.2), (c[1] * g).powf(2.2), (c[2] * g).powf(2.2), c[3].min(1.0)]).collect();
    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default());
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, part.positions.clone());
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, part.normals.clone());
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, part.uvs.clone());
    mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors);
    mesh.insert_indices(Indices::U32(part.indices.clone()));
    mesh
}

/// Rule 11: the 448 x 448 flat page in the top left of the 512 x 512 target the sheet's texture coordinates address.
fn page_target(flat: &neopets_data::texture::Image) -> Image {
    const N: usize = 512;
    let mut rgba = vec![0u8; N * N * 4];
    let (w, h) = (flat.width as usize, flat.height as usize);
    for y in 0..h.min(N) {
        let (from, to) = (y * w * 4, y * N * 4);
        rgba[to..to + w.min(N) * 4].copy_from_slice(&flat.rgba[from..from + w.min(N) * 4]);
    }
    crate::rgba_image(N as u32, N as u32, rgba)
}

fn spawn_parts(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    images: &mut Assets<Image>,
    model: &ModelData,
    page: Option<&Handle<Image>>,
) {
    let handles: Vec<Handle<Image>> = model.textures.iter().map(|t| images.add(crate::rgba_image(t.image.width, t.image.height, t.image.rgba.clone()))).collect();
    for part in &model.parts {
        let texture = page.cloned().or_else(|| part.texture.and_then(|t| handles.get(t).cloned()));
        let material = materials.add(StandardMaterial {
            base_color_texture: texture,
            unlit: true,
            cull_mode: None,
            alpha_mode: if page.is_some() { AlphaMode::Opaque } else { AlphaMode::Blend },
            ..default()
        });
        commands.spawn((Mesh3d(meshes.add(part_mesh(part))), MeshMaterial3d(material), Transform::default(), RenderLayers::layer(BOOK_LAYER), BookPart));
    }
}

/// Rule 12: a model point (x, y, z) at page angle t, with C and S its cosine and sine, goes to (x, C y - S z, S y + C z),
/// scaled by 1.7 and placed along the camera's rows, its origin 1.0 ahead and below the view axis by 0.75 at focal 512.
fn page_matrix(eye: Vec3, forward: Vec3, up: Vec3, focal: f32, angle: f32) -> Mat4 {
    // The view's forward need not be a unit, nor its up square to it: the camera's own rows.
    let forward = forward.normalize_or_zero();
    let side = forward.cross(up).normalize_or_zero();
    let up = side.cross(forward);
    let (s, c) = angle.sin_cos();
    let origin = eye + forward - up * 0.75 * (512.0 / focal.max(1.0));
    // The model's +x runs to the screen's left (rule 12 leaves the sign open; the text reads the right way round so).
    Mat4::from_cols((-side * 1.7).extend(0.0), ((up * c + forward * s) * 1.7).extend(0.0), ((forward * c - up * s) * 1.7).extend(0.0), origin.extend(1.0))
}

#[allow(clippy::too_many_arguments)]
pub fn sync_book(
    mut commands: Commands,
    mut view: ResMut<BookView>,
    menu: Option<Res<Menu>>,
    play: Option<Res<crate::PlayState>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
    mut parts: Query<(Entity, &mut Transform), (With<BookPart>, Without<BookCamera>)>,
    mut cameras: Query<(Entity, &mut Transform), With<BookCamera>>,
    main: Query<(Option<&RenderTarget>, Option<&Msaa>), (With<Camera3d>, Without<BookCamera>)>,
) {
    let open = menu.as_ref().filter(|m| m.flow.current == crate::menus::BOOK).and_then(|m| m.flow.book.as_ref().map(|b| (m, b)));
    let Some((m, b)) = open else {
        if view.index.take().is_some() {
            for (e, _) in &parts {
                commands.entity(e).despawn();
            }
        }
        for (e, _) in &cameras {
            commands.entity(e).despawn();
        }
        return;
    };
    if cameras.is_empty() {
        let (target, msaa) = main.iter().next().map_or((None, Msaa::Off), |(t, a)| (t.cloned(), a.copied().unwrap_or(Msaa::Off)));
        spawn_camera(&mut commands, target, msaa);
    }
    if view.index != Some(b.index) {
        view.index = Some(b.index);
        for (e, _) in &parts {
            commands.entity(e).despawn();
        }
        let mask = m.flow.style_masks.get(b.book.style as usize).copied().unwrap_or(0);
        if let (Some((sheet, pictures)), Some(words)) = (neopets_data::book::models(&b.book.record, mask), m.flow.page_words.as_ref()) {
            info!("Book {}: page sheet {} parts {:?} to {:?}, pictures {} parts {:?} to {:?} (style {}, mask {mask:#x})", b.index, sheet.parts.len(), sheet.min, sheet.max, pictures.parts.len(), pictures.min, pictures.max, b.book.style);
            let page = images.add(page_target(&crate::menu_view::book_page(&b.book, words)));
            spawn_parts(&mut commands, &mut meshes, &mut materials, &mut images, &sheet, Some(&page));
            spawn_parts(&mut commands, &mut meshes, &mut materials, &mut images, &pictures, None);
        }
        return;
    }
    let Some(play) = play else { return };
    let (eye, forward, up, focal) = play.play.view();
    for (_, mut c) in &mut cameras {
        *c = Transform::from_translation(Vec3::from(eye)).looking_to(Vec3::from(forward), Vec3::from(up));
    }
    let t = Transform::from_matrix(page_matrix(Vec3::from(eye), Vec3::from(forward), Vec3::from(up), focal, b.timer.angle));
    for (_, mut part) in &mut parts {
        *part = t;
    }
}
