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
    /// Rising edges seen on frames that ran no tick, kept for the next tick (edge word,
    /// button word), and last frame's stick direction bits.
    pub latched: (u16, u16),
    pub stick: u16,
}

impl Menu {
    pub fn new(flow: Flow) -> Self {
        Self { flow, latched: (0, 0), stick: 0 }
    }
}

/// The left stick's direction bits (SPEC-0035 section 4, analog-to-digital): a byte below 0x20
/// sets Up (vertical) or Left (horizontal), above 0xE0 Down or Right. Bytes as the pad gives
/// them: X right 255, Y up 0.
pub fn stick_bits(gamepads: &Query<&Gamepad>) -> u16 {
    let byte = |v: f32| (127.5 + 127.5 * v.clamp(-1.0, 1.0)).round() as i32;
    let mut w = 0;
    for g in gamepads.iter() {
        let s = g.left_stick();
        let (x, y) = (byte(s.x), byte(-s.y));
        if y < 0x20 {
            w |= menus::UP;
        }
        if y > 0xE0 {
            w |= menus::DOWN;
        }
        if x < 0x20 {
            w |= menus::LEFT;
        }
        if x > 0xE0 {
            w |= menus::RIGHT;
        }
    }
    w
}

#[derive(Component)]
pub struct MenuItem(usize);

/// A black backdrop behind front-end pages: the screen beyond the 4:3 canvas is empty, and the
/// world is not drawn under the menus (no world is loaded before New Game in the original).
#[derive(Component)]
pub struct MenuBackdrop;

/// The held menu word: Left and Right (arrows or A/D, the D-pad) for the sliders; the stick's
/// bits are added by the caller.
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
    // Start (Tab or the Start button) opens the item screen; L1 and R1 (Z and C, or the
    // shoulder buttons) change its tab.
    if keys.just_pressed(KeyCode::Tab) || pad(GamepadButton::Start) {
        buttons |= menus::START;
    }
    if keys.just_pressed(KeyCode::KeyZ) || pad(GamepadButton::LeftTrigger) {
        buttons |= menus::L1;
    }
    if keys.just_pressed(KeyCode::KeyC) || pad(GamepadButton::RightTrigger) {
        buttons |= menus::R1;
    }
    // L2 and R2 (V and B, or the lower triggers) turn the item screen, map and journal ring.
    if keys.just_pressed(KeyCode::KeyV) || pad(GamepadButton::LeftTrigger2) {
        buttons |= menus::L2;
    }
    if keys.just_pressed(KeyCode::KeyB) || pad(GamepadButton::RightTrigger2) {
        buttons |= menus::R2;
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
    /// The map marker's image, redrawn each frame at the marker's angle.
    marker: Option<Handle<Image>>,
    /// The open book's flat page image, by book index.
    book: Option<(usize, Handle<Image>)>,
}

/// SPEC-0035 6.1 rules 10 and 11: the book's flat page, 448 x 448: the 128 x 128 paper picture
/// over the whole page, then each laid-out line in the book's glyph sheet, colour (100, 100,
/// 100, 128), every glyph's advance plus the line's extra and each candidate's further extra.
fn book_page(b: &neopets_data::book::Book, words: &neopets_sim::book::PageWords) -> neopets_data::texture::Image {
    const N: u32 = 448;
    let mut rgba = vec![0u8; (N * N * 4) as usize];
    if let Some(p) = &b.paper {
        for y in 0..N {
            for x in 0..N {
                let (sx, sy) = (x * p.width / N, y * p.height / N);
                let si = ((sy * p.width + sx) * 4) as usize;
                let di = ((y * N + x) * 4) as usize;
                rgba[di..di + 4].copy_from_slice(&p.rgba[si..si + 4]);
            }
        }
    }
    if let (Some(font), Some(sheet)) = (&b.font, &b.sheet) {
        let advance = |c: u8| font.glyph(c).map(|g| g.advance as i32);
        for line in neopets_sim::book::layout(&b.body, b.style, words, font.line_height(), &advance) {
            let mut pen = 0.0f32;
            let mut glyphs = Vec::new();
            for &c in &b.body[line.start..line.end] {
                let Some(g) = font.glyph(c) else { continue };
                glyphs.push((pen, g));
                pen += g.advance as f32 + line.glyph_extra + if c == b' ' || c == b'_' { line.space_extra } else { 0.0 };
            }
            let (img, up) = neopets_data::frontend::render_glyphs(sheet, font.line_height(), &glyphs, [100, 100, 100, 128]);
            let (ox, oy) = (line.x, line.y - up);
            for y in 0..img.height as i32 {
                for x in 0..img.width as i32 {
                    let (dx, dy) = (ox + x, oy + y);
                    if dx < 0 || dy < 0 || dx >= N as i32 || dy >= N as i32 {
                        continue;
                    }
                    let si = ((y as u32 * img.width + x as u32) * 4) as usize;
                    let a = img.rgba[si + 3] as u32;
                    if a == 0 {
                        continue;
                    }
                    let di = ((dy as u32 * N + dx as u32) * 4) as usize;
                    for k in 0..3 {
                        rgba[di + k] = ((img.rgba[si + k] as u32 * a + rgba[di + k] as u32 * (255 - a)) / 255) as u8;
                    }
                    rgba[di + 3] = 255;
                }
            }
        }
    }
    neopets_data::texture::Image { width: N, height: N, rgba }
}

#[allow(clippy::too_many_arguments)]
pub fn sync_menu(
    mut commands: Commands,
    state: Option<Res<PlayState>>,
    menu: Option<Res<Menu>>,
    art: Option<Res<HudArt>>,
    display: Option<Res<crate::display::DisplaySettings>>,
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
    // Play and the box host draw over the running world (the box itself is the HUD's).
    let mut draws = if matches!(flow.current, menus::PLAY | menus::BOX_HOST) { Vec::new() } else { menus::draw_list(&art.f, flow.current, flow.focus, flow.pc()) };
    draws.extend(menus::picker_draws(&art.f, flow));
    draws.extend(menus::credits_draws(&art.f, flow));
    let lines = _state.play.item_lines(flow.pc().items.selected);
    draws.extend(menus::item_draws(&art.f, flow, &lines));
    draws.extend(menus::map_draws(&art.f, flow));
    draws.extend(menus::journal_draws(&art.f, flow));
    draws.extend(menus::book_draws(&art.f, flow));
    let rows = display.map(|d| d.rows().to_vec()).unwrap_or_default();
    draws.extend(menus::display_draws(&art.f, flow, &rows));
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
            Draw::BookPage { x, y } => match (menu.flow.book.as_ref(), menu.flow.page_words.as_ref()) {
                (Some(b), Some(words)) => {
                    if cache.book.as_ref().is_none_or(|(i, _)| *i != b.index) {
                        let img = book_page(&b.book, words);
                        cache.book = Some((b.index, images.add(crate::rgba_image(img.width, img.height, img.rgba))));
                    }
                    cache.book.as_ref().map(|(_, h0)| (h0.clone(), x, y, 448.0, 448.0, Color::WHITE))
                }
                _ => None,
            },
            Draw::Marker { node, x, y, angle, colour } => {
                // SPEC-0051 section 7: each corner offset (x, y) of the 32 x 32 quad goes to the
                // screen offset ((6/7)(C x - S y), S x + C y); drawn into a 48 x 48 image by the
                // inverse map, nearest texel.
                const N: i32 = 48;
                let src = f.sprite_image(mem, node, 32, 32);
                let (c, s) = (angle.cos(), angle.sin());
                let mut rgba = vec![0u8; (N * N * 4) as usize];
                if let Some(src) = src {
                    for oy in 0..N {
                        for ox in 0..N {
                            let (sx, sy) = (ox as f32 + 0.5 - N as f32 / 2.0, oy as f32 + 0.5 - N as f32 / 2.0);
                            let u = sx * 7.0 / 6.0;
                            let (lx, ly) = (c * u + s * sy, -s * u + c * sy);
                            let (tx, ty) = ((lx + 16.0).floor() as i32, (ly + 16.0).floor() as i32);
                            if (0..32).contains(&tx) && (0..32).contains(&ty) {
                                let i = ((ty * 32 + tx) * 4) as usize;
                                let o = ((oy * N + ox) * 4) as usize;
                                rgba[o..o + 4].copy_from_slice(&src.rgba[i..i + 4]);
                            }
                        }
                    }
                }
                let image = crate::rgba_image(N as u32, N as u32, rgba);
                let handle = match cache.marker.as_ref() {
                    Some(h0) => {
                        if let Some(mut img) = images.get_mut(h0) {
                            *img = image;
                        }
                        h0.clone()
                    }
                    None => {
                        let h0 = images.add(image);
                        cache.marker = Some(h0.clone());
                        h0
                    }
                };
                let k = |v: u8| v as f32 / 128.0;
                Some((handle, (x - N as f32 / 2.0).round(), (y - N as f32 / 2.0).round(), N as f32, N as f32, Color::srgba(k(colour[0]), k(colour[1]), k(colour[2]), k(colour[3]).min(1.0))))
            }
            Draw::Text { font, text, spacing, x, y, colour } => {
                // The alpha is applied to the drawn image, so a fading row reuses its image.
                let alpha = colour[3];
                let key = (font, text, spacing, [colour[0], colour[1], colour[2], 128]);
                if !cache.texts.contains_key(&key) {
                    let made = f.text_image(mem, font, &key.1, spacing, key.3).map(|(img, base)| {
                        let (w, h) = (img.width, img.height);
                        (images.add(crate::rgba_image(w, h, img.rgba)), w, h, base)
                    });
                    cache.texts.insert(key.clone(), made);
                }
                cache.texts[&key].as_ref().map(|(h0, w, h, base)| (h0.clone(), x, y - *base as f32, *w as f32, *h as f32, Color::srgba(1.0, 1.0, 1.0, (alpha as f32 / 128.0).min(1.0))))
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
