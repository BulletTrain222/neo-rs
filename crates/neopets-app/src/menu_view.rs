//! The front-end screens in the window (SPEC-0035): the menu flow of `menus` as a resource, its
//! input word from the keyboard and pads, and the current page drawn from the disc tree with
//! its sprites, full-screen pictures (resources 577 to 677) and labels on the 512 x 448 canvas,
//! then the slot picker's rows on the load and save screens.

use crate::hud_view::HudArt;
use crate::menus::{self, Draw, Flow};
use crate::PlayState;
use bevy::prelude::*;
use neopets_data::frontend::Asset;
use std::collections::HashMap;

#[derive(Resource)]
pub struct Menu {
    pub flow: Flow,
}

#[derive(Component)]
pub struct MenuItem(usize);

/// A black backdrop behind front-end pages: the screen beyond the 4:3 canvas is empty, and the
/// world is not drawn under the menus (no world is loaded before New Game in the original).
#[derive(Component)]
pub struct MenuBackdrop;

/// The held menu word: Left and Right (arrows or A/D, the D-pad) for the sliders.
pub fn menu_held(keys: &ButtonInput<KeyCode>, gamepads: &Query<&Gamepad>) -> u16 {
    let pad = |b: GamepadButton| gamepads.iter().any(|g| g.pressed(b));
    let mut w = 0;
    if keys.pressed(KeyCode::ArrowLeft) || keys.pressed(KeyCode::KeyA) || pad(GamepadButton::DPadLeft) {
        w |= menus::LEFT;
    }
    if keys.pressed(KeyCode::ArrowRight) || keys.pressed(KeyCode::KeyD) || pad(GamepadButton::DPadRight) {
        w |= menus::RIGHT;
    }
    w
}

/// The menu words of this frame: the rising-edge word (stick and D-pad directions included) and
/// the button-only word (SPEC-0035 section 4).
pub fn menu_words(keys: &ButtonInput<KeyCode>, gamepads: &Query<&Gamepad>) -> (u16, u16) {
    let pad = |b: GamepadButton| gamepads.iter().any(|g| g.just_pressed(b));
    let mut buttons = 0u16;
    if keys.just_pressed(KeyCode::Space) || keys.just_pressed(KeyCode::Enter) || pad(GamepadButton::South) {
        buttons |= menus::CROSS;
    }
    if keys.just_pressed(KeyCode::KeyT) || keys.just_pressed(KeyCode::Backspace) || pad(GamepadButton::North) {
        buttons |= menus::TRIANGLE;
    }
    if keys.just_pressed(KeyCode::Escape) || pad(GamepadButton::Select) {
        buttons |= menus::SELECT;
    }
    // Circle (Y or F, the east face button) and Square (E, the west face button) answer and
    // page the card boxes.
    if keys.just_pressed(KeyCode::KeyY) || keys.just_pressed(KeyCode::KeyF) || pad(GamepadButton::East) {
        buttons |= menus::CIRCLE;
    }
    if keys.just_pressed(KeyCode::KeyE) || pad(GamepadButton::West) {
        buttons |= menus::SQUARE;
    }
    let mut edge = buttons;
    if keys.just_pressed(KeyCode::ArrowUp) || keys.just_pressed(KeyCode::KeyW) || pad(GamepadButton::DPadUp) {
        edge |= menus::UP;
    }
    if keys.just_pressed(KeyCode::ArrowDown) || keys.just_pressed(KeyCode::KeyS) || pad(GamepadButton::DPadDown) {
        edge |= menus::DOWN;
    }
    if keys.just_pressed(KeyCode::ArrowLeft) || keys.just_pressed(KeyCode::KeyA) || pad(GamepadButton::DPadLeft) {
        edge |= menus::LEFT;
    }
    if keys.just_pressed(KeyCode::ArrowRight) || keys.just_pressed(KeyCode::KeyD) || pad(GamepadButton::DPadRight) {
        edge |= menus::RIGHT;
    }
    (edge, buttons)
}

/// Decode a full-screen picture resource: its one texture chunk's indexed texture through its
/// palette.
fn picture(archive: &mut crate::Archive, resource: u32) -> Option<neopets_data::texture::Image> {
    let crate::Archive { disc, index } = archive;
    let bytes = index.read(disc, resource).ok()?;
    let chain = neopets_data::resource::walk_chunks(&bytes, Some(1)).ok()?;
    let c = chain.chunks.iter().find(|c| c.kind == 4)?;
    let tex = neopets_data::texture::TextureChunk::parse(&bytes[c.payload.clone()]).ok()?;
    let mem = tex.gs_memory();
    let t = tex.textures().next()?;
    let pal = tex.first_palette_for(t)?;
    tex.decode(&mem, t, pal).ok()
}

#[derive(Default)]
pub struct MenuCache {
    sprites: HashMap<(usize, u32, u32), Handle<Image>>,
    labels: HashMap<usize, (Handle<Image>, u32, u32, i32)>,
    texts: HashMap<(usize, Vec<u8>, i32, [u8; 4]), Option<(Handle<Image>, u32, u32, i32)>>,
    pictures: HashMap<u32, Option<neopets_data::texture::Image>>,
}

#[allow(clippy::too_many_arguments)]
pub fn sync_menu(
    mut commands: Commands,
    state: Option<Res<PlayState>>,
    menu: Option<Res<Menu>>,
    art: Option<Res<HudArt>>,
    mut archive: Option<ResMut<crate::Archive>>,
    mut images: ResMut<Assets<Image>>,
    windows: Query<&Window>,
    mut items: Query<(&MenuItem, &mut Node, &mut ImageNode, &mut Visibility), Without<MenuBackdrop>>,
    mut backdrop: Query<&mut Visibility, (With<MenuBackdrop>, Without<MenuItem>)>,
    mut cache: Local<MenuCache>,
    mut pool: Local<usize>,
) {
    let (Some(_state), Some(menu), Some(art)) = (state, menu, art) else { return };
    let Some(window) = windows.iter().next() else { return };
    let flow = &menu.flow;
    let mut draws = if flow.current == menus::PLAY { Vec::new() } else { menus::draw_list(&art.f, flow.current, flow.focus, flow.pc()) };
    draws.extend(menus::picker_draws(&art.f, flow));
    // Grow the pool of UI entities to the draw count.
    if backdrop.is_empty() {
        commands.spawn((
            Node { position_type: PositionType::Absolute, width: Val::Percent(100.0), height: Val::Percent(100.0), ..default() },
            BackgroundColor(Color::BLACK),
            Visibility::Hidden,
            MenuBackdrop,
            GlobalZIndex(29),
        ));
    }
    for mut v in &mut backdrop {
        *v = if draws.is_empty() { Visibility::Hidden } else { Visibility::Inherited };
    }
    if *pool < draws.len() {
        for i in *pool..draws.len() + 8 {
            // Each item is stacked by its draw index, so later draws cover earlier ones.
            commands.spawn((Node { position_type: PositionType::Absolute, ..default() }, ImageNode::default(), Visibility::Hidden, MenuItem(i), GlobalZIndex(30 + i as i32)));
        }
        *pool = draws.len() + 8;
        return;
    }
    let s = (window.width() / 512.0).min(window.height() / 448.0);
    let (ox, oy) = ((window.width() - 512.0 * s) / 2.0, (window.height() - 448.0 * s) / 2.0);
    let crate::hud_view::HudArtParts { f, mem } = art.parts();
    // Build what the draws need.
    let mut placed: Vec<Option<(Handle<Image>, f32, f32, f32, f32, Color)>> = Vec::with_capacity(draws.len());
    for d in &draws {
        placed.push(match d.clone() {
            Draw::Sprite { node, x, y, w, h, colour, texels } => {
                let [iw, ih] = texels;
                let key = (node, iw, ih);
                if !cache.sprites.contains_key(&key) {
                    let asset = f.sprite(node).map(|sp| sp.asset);
                    let img = match asset.and_then(|a| f.assets.get(a as usize).copied()) {
                        Some(Asset::Picture(v)) => {
                            let r = 575 + v;
                            if !cache.pictures.contains_key(&r) {
                                let p = archive.as_deref_mut().and_then(|a| picture(a, r));
                                cache.pictures.insert(r, p);
                            }
                            cache.pictures[&r].as_ref().map(|pic| {
                                let mut rgba = Vec::with_capacity((iw * ih * 4) as usize);
                                for yy in 0..ih {
                                    for xx in 0..iw {
                                        let (sx, sy) = (xx.min(pic.width - 1), yy.min(pic.height - 1));
                                        let i = ((sy * pic.width + sx) * 4) as usize;
                                        rgba.extend_from_slice(&pic.rgba[i..i + 4]);
                                    }
                                }
                                neopets_data::texture::Image { width: iw, height: ih, rgba }
                            })
                        }
                        _ => f.sprite_image(mem, node, iw, ih),
                    };
                    if let Some(img) = img {
                        cache.sprites.insert(key, images.add(crate::rgba_image(img.width, img.height, img.rgba)));
                    }
                }
                let c = |v: u8| v as f32 / 128.0;
                cache.sprites.get(&key).map(|h0| (h0.clone(), x, y, w, h, Color::srgba(c(colour[0]), c(colour[1]), c(colour[2]), c(colour[3]).min(1.0))))
            }
            Draw::Label { node, x, y, scale } => {
                if !cache.labels.contains_key(&node) {
                    if let Some(l) = f.label(node).filter(|l| l.font >= 0) {
                        if let Some((img, base)) = f.text_image(mem, l.font as usize, &l.text, l.spacing as i32, l.colour) {
                            let (w, h) = (img.width, img.height);
                            cache.labels.insert(node, (images.add(crate::rgba_image(w, h, img.rgba)), w, h, base));
                        }
                    }
                }
                cache.labels.get(&node).map(|(h0, w, h, base)| (h0.clone(), x, y - *base as f32 * scale, *w as f32 * scale, *h as f32 * scale, Color::WHITE))
            }
            Draw::Text { font, text, spacing, x, y, colour } => {
                let key = (font, text, spacing, colour);
                if !cache.texts.contains_key(&key) {
                    let made = f.text_image(mem, font, &key.1, spacing, colour).map(|(img, base)| {
                        let (w, h) = (img.width, img.height);
                        (images.add(crate::rgba_image(w, h, img.rgba)), w, h, base)
                    });
                    cache.texts.insert(key.clone(), made);
                }
                cache.texts[&key].as_ref().map(|(h0, w, h, base)| (h0.clone(), x, y - *base as f32, *w as f32, *h as f32, Color::WHITE))
            }
        });
    }
    for (item, mut node, mut image, mut vis) in &mut items {
        match placed.get(item.0).cloned().flatten() {
            Some((handle, x, y, w, h, colour)) => {
                node.left = Val::Px(ox + x * s);
                node.top = Val::Px(oy + y * s);
                node.width = Val::Px(w * s);
                node.height = Val::Px(h * s);
                image.image = handle;
                image.color = colour;
                *vis = Visibility::Inherited;
            }
            None => *vis = Visibility::Hidden,
        }
    }
}
