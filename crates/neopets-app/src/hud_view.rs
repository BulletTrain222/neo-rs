//! The in-play HUD drawn from the front-end database (SPEC-0023 HUD screen and art; SPEC-0035
//! Section J node transform, Sprite drawing, Predicate table): the bars group 0x31 of the HUD
//! page, laid out on the 512 x 448 canvas scaled to the window, with the bar widths, the right
//! caps, the fade alpha and the health frame's tint from `neopets_sim::hud`.
//! Only the bars group is drawn so far; its predicates are slots 42 to 48 (other slots read as
//! false), and value texts and labels are not drawn.

use crate::PlayState;
use bevy::prelude::*;
use neopets_data::frontend::Frontend;
use neopets_data::gsmem::GsMemory;
use std::collections::HashMap;

/// Section C keys of the HUD (SPEC-0023 HUD elements).
const BARS: u32 = 0x31;
const TINT: u32 = 0x3B;
const HEALTH_FILL: u32 = 0x01;
const POOL_FILL: u32 = 0x02;
const HEALTH_MIDDLE: u32 = 0x07;
const POOL_MIDDLE: u32 = 0x09;
const HEALTH_CAP: u32 = 0x06;
const POOL_CAP: u32 = 0x08;
/// The monster bar group and its pieces (SPEC-0023 section 10.5).
const MONSTER: u32 = 0x24;
const MONSTER_MIDDLE: u32 = 0x1F;
const MONSTER_CAP: u32 = 0x20;
const PLATE_CENTRE: u32 = 0x21;
const PLATE_RIGHT: u32 = 0x22;
const MONSTER_FILL: u32 = 0x23;
const MONSTER_NAME: u32 = 0x32;

#[derive(Resource)]
pub struct HudArt {
    pub f: Frontend,
    mem: GsMemory,
    images: HashMap<(usize, u32, u32), Handle<Image>>,
    /// The monster name's text image (name, image, baseline).
    name: Option<(String, Handle<Image>, u32, u32, i32)>,
}

/// Shared read access to the front-end database and its texture memory.
pub struct HudArtParts<'a> {
    pub f: &'a Frontend,
    pub mem: &'a GsMemory,
}

impl HudArt {
    pub fn parts(&self) -> HudArtParts<'_> {
        HudArtParts { f: &self.f, mem: &self.mem }
    }

    pub fn new(f: Frontend) -> Option<Self> {
        let mem = f.memory()?;
        Some(Self { f, mem, images: HashMap::new(), name: None })
    }
}

#[derive(Component)]
pub struct HudText;

#[derive(Component)]
pub struct HudCaption;

#[derive(Component)]
pub struct HudCanvas;

#[derive(Component)]
pub struct HudSprite(usize);

fn subtree(f: &Frontend, n: usize, out: &mut Vec<usize>) {
    out.push(n);
    for c in f.children(n) {
        subtree(f, c, out);
    }
}

/// The node's condition (mode 0 draws; 2 while the predicate is nonzero; 3 while it is zero).
fn condition(f: &Frontend, n: usize, p: &Inputs) -> bool {
    let node = f.nodes[n];
    let value = match node.handler {
        44 => p.c != 0,
        47 => p.c != 0 && p.mp_max > 0,
        48 => p.g0 != 0,
        42 => (node.arg as i32) < p.health_dividers,
        45 => (node.arg as i32) < p.pool_dividers,
        41 => p.monster,
        25 => p.square,
        _ => false,
    };
    match node.mode {
        0 => true,
        2 => value,
        3 => !value,
        _ => false,
    }
}

/// Canvas position and scale of a node (SPEC-0035 Node transform; row spacing over all
/// siblings, alignment by the node's width), with x and width overrides.
fn place(f: &Frontend, n: usize, xs: &HashMap<usize, i32>, ws: &HashMap<usize, i32>) -> (f32, f32, f32, f32) {
    let node = f.nodes[n];
    let (px, py, psx, psy) = match node.parent {
        Some(p) if !node.absolute => place(f, p, xs, ws),
        _ => (0.0, 0.0, 1.0, 1.0),
    };
    let x = *xs.get(&n).unwrap_or(&(node.x as i32)) as f32;
    let mut y = node.y as f32;
    if let Some(p) = node.parent {
        let sib = f.children(p);
        let i = sib.iter().position(|&s| s == n).unwrap_or(0);
        y += f.nodes[p].spacing as f32 * i as f32;
    }
    let (sx, sy) = (psx * node.scale_x as f32 / 4096.0, psy * node.scale_y as f32 / 4096.0);
    let w = *ws.get(&n).unwrap_or(&(node.width as i32)) as f32;
    let shift = match node.align {
        1 => w * sx / 2.0,
        2 => w * sx,
        _ => 0.0,
    };
    (px + x * psx - shift, py + y * psy, sx, sy)
}

/// The values the HUD's predicates read this frame.
struct Inputs {
    c: i32,
    g0: i32,
    mp_max: i32,
    health_dividers: i32,
    pool_dividers: i32,
    /// The monster bar's draw rule (T not 0 and the quick panel closed; SPEC-0023 10.2).
    monster: bool,
    /// The Square prompt slot is shown this pass (predicate slot 25).
    square: bool,
}

#[allow(clippy::too_many_arguments)]
pub fn sync_hud(
    mut commands: Commands,
    state: Option<Res<PlayState>>,
    art: Option<ResMut<HudArt>>,
    mut images: ResMut<Assets<Image>>,
    windows: Query<&Window>,
    mut canvas: Query<(Entity, &mut Node), With<HudCanvas>>,
    mut sprites: Query<(&HudSprite, &mut Node, &mut ImageNode, &mut Visibility), (Without<HudCanvas>, Without<HudText>)>,
    mut texts: Query<(&mut Node, &mut ImageNode, &mut Visibility), (With<HudText>, Without<HudCanvas>, Without<HudSprite>, Without<HudCaption>)>,
    mut captions: Query<(&mut Node, &mut ImageNode, &mut Visibility), (With<HudCaption>, Without<HudCanvas>, Without<HudSprite>, Without<HudText>)>,
    mut caption_image: Local<Option<(Vec<u8>, Handle<Image>, u32, u32, i32)>>,
) {
    let (Some(state), Some(mut art)) = (state, art) else { return };
    let HudArt { f, mem, images: cache, name: name_image } = &mut *art;
    let f: &Frontend = f;
    let Some(window) = windows.iter().next() else { return };
    let p = &state.play;
    let s = (window.width() / 512.0).min(window.height() / 448.0);
    let (ox, oy) = ((window.width() - 512.0 * s) / 2.0, (window.height() - 448.0 * s) / 2.0);
    let Some(bars) = f.key(BARS) else { return };
    let monster = f.key(MONSTER);
    // The button-prompt group: the parent of the HUD page's node gated by predicate slot 25.
    let mut page = Vec::new();
    if let Some(root) = f.screens.get(9).and_then(|s| s.node) {
        subtree(f, root, &mut page);
    }
    let prompts = page.iter().copied().find(|&n| f.nodes[n].handler == 25).and_then(|n| f.nodes[n].parent);
    let mut nodes = Vec::new();
    subtree(f, bars, &mut nodes);
    if let Some(m) = monster {
        subtree(f, m, &mut nodes);
    }
    if let Some(g) = prompts {
        subtree(f, g, &mut nodes);
    }
    let Ok((root, mut root_node)) = canvas.single_mut() else {
        let root = commands
            .spawn((Node { position_type: PositionType::Absolute, ..default() }, HudCanvas, GlobalZIndex(10)))
            .id();
        for &n in nodes.iter().filter(|&&n| f.sprite(n).is_some()) {
            let child = commands.spawn((Node { position_type: PositionType::Absolute, ..default() }, ImageNode::default(), Visibility::Hidden, HudSprite(n))).id();
            commands.entity(root).add_child(child);
        }
        let text = commands.spawn((Node { position_type: PositionType::Absolute, ..default() }, ImageNode::default(), Visibility::Hidden, HudText)).id();
        commands.entity(root).add_child(text);
        let caption = commands.spawn((Node { position_type: PositionType::Absolute, ..default() }, ImageNode::default(), Visibility::Hidden, HudCaption)).id();
        commands.entity(root).add_child(caption);
        return;
    };
    let _ = root;
    root_node.left = Val::Px(ox);
    root_node.top = Val::Px(oy);
    root_node.width = Val::Px(512.0 * s);
    root_node.height = Val::Px(448.0 * s);
    let hud = &p.hud;
    let mb = &p.monster_bar;
    let inputs = Inputs {
        c: hud.c,
        g0: hud.g0,
        mp_max: p.hero_mp_max,
        health_dividers: hud.health.dividers,
        pool_dividers: hud.pool.dividers,
        monster: mb.t != 0 && !p.quick.open,
        square: p.square_caption().is_some(),
    };
    let key = |k: u32| f.key(k).unwrap_or(usize::MAX);
    let node_x = |k: u32| f.key(k).map_or(0, |n| f.nodes[n].x as i32);
    let ws: HashMap<usize, i32> = [
        (key(HEALTH_FILL), hud.health.fill),
        (key(POOL_FILL), hud.pool.fill),
        (key(HEALTH_MIDDLE), hud.health.middle),
        (key(POOL_MIDDLE), hud.pool.middle),
        (key(MONSTER_MIDDLE), mb.frame),
        (key(PLATE_CENTRE), mb.name_width),
        (key(MONSTER_FILL), mb.fill),
    ]
    .into_iter()
    .collect();
    let xs: HashMap<usize, i32> = [
        (key(HEALTH_CAP), hud.health.cap_x),
        (key(POOL_CAP), hud.pool.cap_x),
        (key(MONSTER_CAP), node_x(MONSTER_MIDDLE) + mb.frame - 5),
        (key(PLATE_RIGHT), node_x(PLATE_CENTRE) + mb.name_width - 5),
    ]
    .into_iter()
    .collect();
    // The name plate group (the parent of the name text) shows only with a name.
    let plate = f.key(MONSTER_NAME).and_then(|n| f.nodes[n].parent);
    let in_monster = |n: usize| {
        let mut at = Some(n);
        while let Some(i) = at {
            if Some(i) == monster {
                return true;
            }
            at = f.nodes[i].parent;
        }
        false
    };
    let mut tinted = Vec::new();
    if let Some(t) = f.key(TINT) {
        subtree(f, t, &mut tinted);
    }
    let shown = p.spawned && !p.dialogue_open && p.death.page.is_none() && !p.menu_open;
    let mut wanted: Vec<(usize, u32, u32)> = Vec::new();
    for (sprite, mut node, mut image, mut vis) in &mut sprites {
        let n = sprite.0;
        // Visible when every node from the sprite up to the bars group passes its condition.
        let mut ok = shown;
        let in_prompts = |n: usize| {
            let mut at = Some(n);
            while let Some(i) = at {
                if Some(i) == prompts {
                    return true;
                }
                at = f.nodes[i].parent;
            }
            false
        };
        let top = if in_monster(n) {
            monster.unwrap_or(bars)
        } else if in_prompts(n) {
            prompts.unwrap_or(bars)
        } else {
            bars
        };
        let mut at = Some(n);
        while let Some(i) = at {
            ok &= condition(f, i, &inputs);
            ok &= !(Some(i) == plate && mb.name.is_empty());
            if i == top {
                break;
            }
            at = f.nodes[i].parent;
        }
        let w = *ws.get(&n).unwrap_or(&(f.nodes[n].width as i32));
        let h = f.nodes[n].height as i32;
        if !ok || w <= 0 || h <= 0 {
            *vis = Visibility::Hidden;
            continue;
        }
        let (x, y, mut sx, sy) = place(f, n, &xs, &ws);
        if Some(n) == f.key(MONSTER_FILL) {
            // The fill is drawn with its x scale reset to 1.0.
            sx = 1.0;
        }
        node.left = Val::Px(x * s);
        node.top = Val::Px(y * s);
        node.width = Val::Px(w as f32 * sx * s);
        node.height = Val::Px(h as f32 * sy * s);
        let base = f.sprite(n).map_or([128; 4], |sp| sp.colour);
        let rgb = if tinted.contains(&n) { [hud.tint[0], hud.tint[1], hud.tint[2]] } else { [base[0], base[1], base[2]] };
        let c = |v: u8| v as f32 / 128.0;
        let a = if in_monster(n) {
            neopets_sim::hud::alpha(mb.t)
        } else if in_prompts(n) {
            128
        } else {
            hud.bars_alpha
        };
        image.color = Color::srgba(c(rgb[0]), c(rgb[1]), c(rgb[2]), c(a).min(1.0));
        match cache.get(&(n, w as u32, h as u32)) {
            Some(handle) => image.image = handle.clone(),
            None => wanted.push((n, w as u32, h as u32)),
        }
        *vis = Visibility::Inherited;
    }
    // The monster's name: the value text of key 0x32 in its record's font and colour, placed
    // with its baseline at the node's y.
    if let (Ok((mut node, mut image, mut vis)), Some(nn)) = (texts.single_mut(), f.key(MONSTER_NAME)) {
        let visible = shown && inputs.monster && !mb.name.is_empty();
        *vis = if visible { Visibility::Inherited } else { Visibility::Hidden };
        if visible {
            if name_image.as_ref().is_none_or(|n| n.0 != mb.name) {
                let v = f.values.get(&f.nodes[nn].data).copied().unwrap_or_default();
                if let Some((img, base)) = f.text_image(mem, v.font as usize, mb.name.as_bytes(), v.spacing as i32, [v.colour[0], v.colour[1], v.colour[2], 128]) {
                    let (w, h) = (img.width, img.height);
                    *name_image = Some((mb.name.clone(), images.add(crate::rgba_image(w, h, img.rgba)), w, h, base));
                }
            }
            if let Some((_, handle, w, h, base)) = name_image.as_ref() {
                let (x, y, sx, sy) = place(f, nn, &xs, &ws);
                node.left = Val::Px(x * s);
                node.top = Val::Px((y - *base as f32 * sy) * s);
                node.width = Val::Px(*w as f32 * sx * s);
                node.height = Val::Px(*h as f32 * sy * s);
                image.image = handle.clone();
                image.color = Color::srgba(1.0, 1.0, 1.0, (neopets_sim::hud::alpha(mb.t) as f32 / 128.0).min(1.0));
            }
        }
    }
    // SPEC-0023 section 30: the Square caption at its slot cell (327, 395), in the default
    // caption colour (boot 0x315A38) and font 1 (the prompt font; R-0018), baseline at the cell.
    if let Ok((mut node, mut image, mut vis)) = captions.single_mut() {
        match p.square_caption().filter(|_| shown) {
            Some(text) => {
                if caption_image.as_ref().is_none_or(|c| c.0 != text) {
                    if let Some((img, base)) = f.text_image(mem, 1, text, 0, p.box_words.prompt_colour) {
                        let (w, h) = (img.width, img.height);
                        *caption_image = Some((text.to_vec(), images.add(crate::rgba_image(w, h, img.rgba)), w, h, base));
                    }
                }
                if let Some((_, handle, w, h, base)) = caption_image.as_ref() {
                    node.left = Val::Px(327.0 * s);
                    node.top = Val::Px((395 - base) as f32 * s);
                    node.width = Val::Px(*w as f32 * s);
                    node.height = Val::Px(*h as f32 * s);
                    image.image = handle.clone();
                    *vis = Visibility::Inherited;
                }
            }
            None => *vis = Visibility::Hidden,
        }
    }
    for (n, w, h) in wanted {
        if let Some(img) = f.sprite_image(mem, n, w, h) {
            let handle = images.add(crate::rgba_image(img.width, img.height, img.rgba));
            cache.insert((n, w, h), handle);
        }
    }
}

/// The parts of the dialogue box.
#[derive(Component, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BoxPart {
    Frame,
    Name,
    Body,
    /// The right prompt group (Square with "Next" or "Finish", or Circle with the first choice
    /// label) and the left one (Triangle with the second choice label).
    Icon,
    Label,
    Icon2,
    Label2,
}

/// The box's prompt row (SPEC-0022 Text window draw layout).
enum Prompt<'a> {
    None,
    Continue(&'a [u8]),
    Choice(&'a [u8], &'a [u8]),
}

/// The images the box shows, rebuilt when what they show changes.
#[derive(Default)]
pub struct BoxCache {
    key: Vec<(BoxPart, u64)>,
    images: HashMap<BoxPart, (Handle<Image>, u32, u32, i32)>,
}

fn hash<T: std::hash::Hash>(v: T) -> u64 {
    use std::hash::Hasher;
    let mut h = std::collections::hash_map::DefaultHasher::new();
    v.hash(&mut h);
    h.finish()
}

/// SPEC-0022 Dialogue box layout on the 512 x 448 canvas: with L wrapped lines of the page
/// (font 0, wrap 410), the base line yb = 345 - 10 L; the frame (a nine-slice of the 64 x 64 box
/// sprite, key 0x19, borders 15, 28, 14, 26) at (30, yb - 41), 446 wide and 21 L + 45 high, in
/// the speaker colour at alpha 45; the name (font 0, spacing -1) at x 70 on baseline yb - 30;
/// the body (font 0) at x 50 from baseline yb, 21 per line, revealed up to the visible index;
/// the prompt row once the page is complete: the Square icon (key 0x1B) and "Next" or "Finish"
/// (font 1, spacing -1) grouped about x 384 on baseline yb + 21 L + 1.
#[allow(clippy::too_many_arguments)]
pub fn sync_textbox(
    mut commands: Commands,
    state: Option<Res<PlayState>>,
    menu: Option<Res<crate::menu_view::Menu>>,
    art: Option<ResMut<HudArt>>,
    mut images: ResMut<Assets<Image>>,
    windows: Query<&Window>,
    canvas: Query<Entity, With<HudCanvas>>,
    mut parts: Query<(&BoxPart, &mut Node, &mut ImageNode, &mut Visibility), (Without<HudCanvas>, Without<HudSprite>, Without<HudText>, Without<HudCaption>)>,
    mut cache: Local<BoxCache>,
) {
    let (Some(state), Some(mut art)) = (state, art) else { return };
    let Some(window) = windows.iter().next() else { return };
    let Ok(root) = canvas.single() else { return };
    if parts.is_empty() {
        // Above the front-end pages: a card message shows over the save and load screens.
        for part in [BoxPart::Frame, BoxPart::Name, BoxPart::Body, BoxPart::Icon, BoxPart::Label, BoxPart::Icon2, BoxPart::Label2] {
            let e = commands.spawn((Node { position_type: PositionType::Absolute, ..default() }, ImageNode::default(), Visibility::Hidden, part, GlobalZIndex(1000))).id();
            commands.entity(root).add_child(e);
        }
        return;
    }
    let p = &state.play;
    // SPEC-0025 Card message records: a card box of the front end, else the dialogue box.
    let card = menu.as_deref().and_then(|m| m.flow.card.as_ref()).filter(|c| c.text.open);
    let (text, name, colour, prompt) = match card {
        Some(c) => {
            let prompt = if c.busy {
                Prompt::None
            } else if c.text.more() {
                Prompt::Continue(&p.box_labels.0)
            } else {
                Prompt::Choice(&c.labels[0], &c.labels[1])
            };
            (&c.text, &[][..], c.colour, prompt)
        }
        None if p.dialogue_open && p.textbox.open && !p.menu_open => {
            let (name, colour) = p.box_speaker();
            (&p.textbox, name, colour, Prompt::Continue(if p.textbox.more() { &p.box_labels.0 } else { &p.box_labels.1 }))
        }
        None => {
            for (_, _, _, mut vis) in &mut parts {
                *vis = Visibility::Hidden;
            }
            return;
        }
    };
    let s = (window.width() / 512.0).min(window.height() / 448.0);
    let HudArt { f, mem, .. } = &mut *art;
    let f: &Frontend = f;
    let w = p.box_words;
    let page = text.page();
    let lines = f.wrap(0, page, w.wrap, 0);
    let l = lines.len().max(1) as i32;
    let h = f.fonts.first().map_or(21, |ft| ft.line_height());
    let yb = 345 - 10 * l;
    let complete = text.complete();
    let visible = text.visible();
    let frame_h = (l * h + w.height_terms.1 + h + w.height_terms.0) as u32;
    let frame_w = (w.wrap + w.margin + w.width_term) as u32;
    // The prompt groups: (icon key, label, x) for the right and the left group.
    let (right, left): (Option<(u32, &[u8], i32)>, Option<(u32, &[u8], i32)>) = match prompt {
        Prompt::None => (None, None),
        Prompt::Continue(label) => (Some((0x1B, label, w.prompt_x.1)), None),
        Prompt::Choice(first, second) => (Some((0x79, first, w.prompt_x.1)), Some((0x78, second, w.prompt_x.0))),
    };
    // Rebuild the images whose content changed.
    let wanted = vec![
        (BoxPart::Frame, hash(frame_h)),
        (BoxPart::Name, hash(name)),
        (BoxPart::Body, hash((page, visible))),
        (BoxPart::Icon, hash(right.map(|r| r.0))),
        (BoxPart::Label, hash(right.map(|r| r.1))),
        (BoxPart::Icon2, hash(left.map(|r| r.0))),
        (BoxPart::Label2, hash(left.map(|r| r.1))),
    ];
    for (part, k) in &wanted {
        if cache.key.contains(&(*part, *k)) && cache.images.contains_key(part) {
            continue;
        }
        let icon = |g: Option<(u32, &[u8], i32)>| g.and_then(|(key, _, _)| f.key(key)).and_then(|n| f.sprite_image(mem, n, 32, 32)).map(|i| (i, 0));
        let label = |g: Option<(u32, &[u8], i32)>| g.and_then(|(_, t, _)| f.text_image(mem, 1, t, -1, w.prompt_colour));
        let made = match part {
            BoxPart::Frame => f.key(0x19).and_then(|n| f.nine_slice(mem, n, frame_w, frame_h, [15, 28, 14, 26])).map(|i| (i, 0)),
            BoxPart::Name => f.text_image(mem, 0, name, -1, w.name_colour),
            BoxPart::Body => f.draw_lines(mem, 0, page, &lines, h, visible, 0, w.body_colour),
            BoxPart::Icon => icon(right),
            BoxPart::Label => label(right),
            BoxPart::Icon2 => icon(left),
            BoxPart::Label2 => label(left),
        };
        match made {
            Some((img, base)) => {
                let (iw, ih) = (img.width, img.height);
                cache.images.insert(*part, (images.add(crate::rgba_image(iw, ih, img.rgba)), iw, ih, base));
            }
            None => {
                cache.images.remove(part);
            }
        }
        cache.key.retain(|(q, _)| q != part);
        cache.key.push((*part, *k));
    }
    let py = yb + h * l + 1;
    // A group's left gx = x - (tw + E/2) / 2 with E = 32; the icon is centred on (gx, y) (its
    // anchor is Medium in the spec; R-0018) and the text sits at gx + E/2.
    let group = |g: Option<(u32, &[u8], i32)>| {
        g.map(|(_, label, x)| {
            let tw = f.fonts.get(1).map_or(0, |ft| ft.measure(label, -1));
            let th = f.fonts.get(1).and_then(|ft| label.iter().rev().find_map(|&c| ft.glyph(c))).map_or(0, |g| g.height as i32);
            (x - (tw + 16) / 2, th, !label.is_empty())
        })
    };
    let (gr, gl) = (group(right), group(left));
    for (part, mut node, mut image, mut vis) in &mut parts {
        let Some((handle, iw, ih, base)) = cache.images.get(part) else {
            *vis = Visibility::Hidden;
            continue;
        };
        let (x, y, show) = match (part, gr, gl) {
            (BoxPart::Frame, _, _) => (w.anchor - w.margin, yb - w.margin - h, true),
            (BoxPart::Name, _, _) => (w.anchor + w.margin, yb - w.margin - h / 2 - base, !name.is_empty()),
            (BoxPart::Body, _, _) => (w.anchor, yb - base, true),
            (BoxPart::Icon, Some((gx, _, _)), _) | (BoxPart::Icon2, _, Some((gx, _, _))) => (gx - 16, py - 16, complete),
            (BoxPart::Label, Some((gx, th, any)), _) | (BoxPart::Label2, _, Some((gx, th, any))) => (gx + 16, py + 16 - th / 2 - base, complete && any),
            _ => (0, 0, false),
        };
        *vis = if show { Visibility::Inherited } else { Visibility::Hidden };
        node.left = Val::Px(x as f32 * s);
        node.top = Val::Px(y as f32 * s);
        node.width = Val::Px(*iw as f32 * s);
        node.height = Val::Px(*ih as f32 * s);
        image.image = handle.clone();
        image.color = match part {
            BoxPart::Frame => Color::srgba(colour[0] as f32 / 128.0, colour[1] as f32 / 128.0, colour[2] as f32 / 128.0, 45.0 / 128.0),
            _ => Color::WHITE,
        };
    }
}
