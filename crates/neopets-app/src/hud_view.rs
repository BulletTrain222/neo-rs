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

impl HudArt {
    pub fn new(f: Frontend) -> Option<Self> {
        let mem = f.memory()?;
        Some(Self { f, mem, images: HashMap::new(), name: None })
    }
}

#[derive(Component)]
pub struct HudText;

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
    mut texts: Query<(&mut Node, &mut ImageNode, &mut Visibility), (With<HudText>, Without<HudCanvas>, Without<HudSprite>)>,
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
    let mut nodes = Vec::new();
    subtree(f, bars, &mut nodes);
    if let Some(m) = monster {
        subtree(f, m, &mut nodes);
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
    let shown = p.spawned && !p.dialogue_open && p.death.page.is_none();
    let mut wanted: Vec<(usize, u32, u32)> = Vec::new();
    for (sprite, mut node, mut image, mut vis) in &mut sprites {
        let n = sprite.0;
        // Visible when every node from the sprite up to the bars group passes its condition.
        let mut ok = shown;
        let top = if in_monster(n) { monster.unwrap_or(bars) } else { bars };
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
        let a = if in_monster(n) { neopets_sim::hud::alpha(mb.t) } else { hud.bars_alpha };
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
    for (n, w, h) in wanted {
        if let Some(img) = f.sprite_image(mem, n, w, h) {
            let handle = images.add(crate::rgba_image(img.width, img.height, img.rgba));
            cache.insert((n, w, h), handle);
        }
    }
}
