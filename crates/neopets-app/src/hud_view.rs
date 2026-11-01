//! The in-play HUD drawn from the front-end database (SPEC-0023 HUD screen and art; SPEC-0035
//! Section J node transform, Sprite drawing, Predicate table): the bars group 0x31 of the HUD
//! page, laid out on the 512 x 448 canvas scaled to the window, with the bar widths, the right
//! caps, the fade alpha and the health frame's tint from `neopets_sim::hud`.
//! Drawn: the bars group (predicate slots 42 to 48), the monster bar, the button prompts, the
//! quick-item panels and the petpet panel (slot 38 and the meter fill; SPEC-0023 section 7).

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
/// SPEC-0023 section 11: the selected quick item panel, the equipment panels (weapon slot 2,
/// armour slot 0, shield slot 1), the right quick-item cross icon (its group is the cross), and
/// the item-icon elements, whose art is the child of the item-icon node (key 0x9F) at the item's
/// id (R-0020): the four cross icons show the quick-item list's entries 0 to 3 (right, up, left,
/// down) and 0xA0 and 0xA1 the selected item (the marker), each while its count is above 0.
const ITEM_PANEL: u32 = 0x30;
const EQUIP: [u32; 3] = [0x1C, 0x1D, 0x1E];
const CROSS_RIGHT: u32 = 0x9C;
const ITEM_ICONS: [u32; 6] = [0x9C, 0x9B, 0x9E, 0x9D, 0xA0, 0xA1];
/// SPEC-0023 sections 2 and 7: the petpet panel (the predicate-38 group above the portrait
/// 0x3C's group) and its meter fill 0x12.
const PET_PORTRAIT: u32 = 0x3C;
const JOURNAL_ICON: u32 = 0xDF;
const PIVOT: u32 = 0x29;
const PORTRAITS: [u32; 2] = [0x2A, 0x2B];
const DPAD_HINT: u32 = 0xBB;
const PET_FILL: u32 = 0x12;

#[derive(Resource)]
pub struct HudArt {
    pub f: Frontend,
    mem: GsMemory,
    images: HashMap<(usize, u32, u32), Handle<Image>>,
    /// The monster name's text image (name, image, baseline).
    name: Option<(String, Handle<Image>, u32, u32, i32)>,
    /// Counts the button-picture changes, so the dialogue box and the menus rebuild their icons.
    pub style: u32,
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
        Some(Self { f, mem, images: HashMap::new(), name: None, style: 0 })
    }

    /// Put art in place of sheet assets (the button pictures) and drop the cached sprites.
    pub fn set_overrides(&mut self, overrides: HashMap<u32, neopets_data::texture::Image>) {
        self.f.sprite_overrides = overrides;
        self.images.clear();
        self.style += 1;
    }
}

#[derive(Component)]
pub struct HudText;

#[derive(Component)]
pub struct HudCaption;

/// A line of the price caption (SPEC-0035 section 15.2): 0 the price or coin gain, 1 the
/// balance.
#[derive(Component)]
pub struct HudPrice(usize);

#[derive(Component)]
pub struct HudCanvas;

#[derive(Component)]
pub struct HudSprite(usize);

/// A value-text node (kind 6) of the panels, its number drawn in its record's font, or a label
/// node (kind 2), its binding's text.
#[derive(Component)]
pub struct HudValue(usize);

fn subtree(f: &Frontend, n: usize, out: &mut Vec<usize>) {
    out.push(n);
    for c in f.children(n) {
        subtree(f, c, out);
    }
}

/// Whether node `n` is node `root` or lies under it.
fn under_node(f: &Frontend, n: usize, root: usize) -> bool {
    let mut at = Some(n);
    while let Some(i) = at {
        if i == root {
            return true;
        }
        at = f.nodes[i].parent;
    }
    false
}

/// The panel root a node belongs to, if any.
fn panel_root(f: &Frontend, n: usize, roots: &[usize]) -> Option<usize> {
    let mut at = Some(n);
    while let Some(i) = at {
        if roots.contains(&i) {
            return Some(i);
        }
        at = f.nodes[i].parent;
    }
    None
}

/// The alpha last pushed to the fade group a node is under (0x30, 0x1C, 0x1D, 0x1E); the
/// cross has no fade group and stays at 128.
fn group_alpha(f: &Frontend, n: usize, pushed: &[u8; 4]) -> u8 {
    let groups = [ITEM_PANEL, EQUIP[0], EQUIP[1], EQUIP[2]].map(|k| f.key(k));
    let mut at = Some(n);
    while let Some(i) = at {
        if let Some(k) = groups.iter().position(|&g| g == Some(i)) {
            return pushed[k];
        }
        at = f.nodes[i].parent;
    }
    128
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
        // Slot 38 with the petpet panel's arguments 255, 6, 3, 1, 0 and 7: SPEC-0023 section 7
        // shows the group whose argument is the active petpet's kind (255 any kind); SPEC-0035
        // names the slot's value a music index instead (R-0094).
        38 => p.pet.is_some_and(|k| node.arg == 255 || node.arg == k),
        25 => p.square,
        // SPEC-0035 9c and SPEC-0023 section 4: the quick-item cross and the panels.
        32 => p.quick_open,
        11 => p.counts.get(p.selection).is_some_and(|&c| c > 0),
        28 => p.selection == node.arg as usize,
        29 => p.counts[match node.arg {
            0 => 1,
            1 => 0,
            2 => 3,
            _ => 2,
        }] > 0,
        30 => p.counts[0] > 0,
        33 => p.counts[1] > 0,
        34 => p.counts[2] > 0,
        36 => p.counts[3] > 0,
        49 => p.marker >= 0,
        51 => p.g1 != 0,
        53 => p.g[0] != 0 && p.weapon,
        54 => p.g[1] != 0,
        55 => p.g[2] != 0 && p.shield,
        56 => true,
        // SPEC-0035 9c: the active hero is Tor (14) or Roberta (52); the count of the item in
        // equipment word 2 (17), word 0 (19) or word 1 (20) is above 0.
        14 => p.hero == 0,
        52 => p.hero == 1,
        // 60: the swing's HUD word is set and game-state word +0x08 (the swap mode) is 2.
        60 => p.swing,
        // SPEC-0035 9c: 37 the Mote panel is open (the camera word is not modelled); 15 the
        // selection equals the argument (255 none); 16 the owned count of Mote 160 + argument.
        37 => p.mote_open,
        15 => node.arg < 100 && p.mote_selection == node.arg || node.arg == 255 && p.mote_selection == 255,
        16 => p.mote_counts.get(node.arg as usize).is_some_and(|&c| c > 0),
        62 => p.hint,
        65 => p.journal,
        57 => p.available[0],
        59 => p.available[1],
        58 => p.dead.get(node.arg as usize).copied().unwrap_or(false),
        17 => p.words[2] > 0,
        19 => p.words[0] > 0,
        20 => p.words[1] > 0,
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
/// The window-edge shift for a HUD piece at canvas coordinate `c` along an axis of canvas
/// length `len`: the first third keeps its distance from the window's near edge, the last third
/// from the far edge, and the middle stays centred (`margin` is the canvas's margin in the
/// window on that axis).
fn edge_shift(c: f32, len: f32, margin: f32) -> f32 {
    if c < len / 3.0 {
        -margin
    } else if c > 2.0 * len / 3.0 {
        margin
    } else {
        0.0
    }
}

fn place(f: &Frontend, n: usize, xs: &HashMap<usize, i32>, ws: &HashMap<usize, i32>, ov: &HashMap<usize, (i32, i32, f32)>) -> (f32, f32, f32, f32) {
    let node = f.nodes[n];
    let (px, py, psx, psy) = match node.parent {
        Some(p) if !node.absolute => place(f, p, xs, ws, ov),
        _ => (0.0, 0.0, 1.0, 1.0),
    };
    let x = *xs.get(&n).unwrap_or(&(node.x as i32)) as f32;
    let mut y = node.y as f32;
    if let Some(p) = node.parent {
        let sib = f.children(p);
        let i = sib.iter().position(|&s| s == n).unwrap_or(0);
        y += f.nodes[p].spacing as f32 * i as f32;
    }
    let (mut sx, mut sy) = (psx * node.scale_x as f32 / 4096.0, psy * node.scale_y as f32 / 4096.0);
    // A position and scale written at run time (the portrait swing; SPEC-0023 section 12).
    let (x, y) = match ov.get(&n) {
        Some(&(ox, oy, k)) => {
            (sx, sy) = (psx * k, psy * k);
            (ox as f32, oy as f32)
        }
        None => (x, y),
    };
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
    /// Predicate slot 62 (SPEC-0035 9c): no quick panel open, slot 41 false and the busy test
    /// false (the D-pad hint); slot 65: slot 62 and the journal-notification word.
    hint: bool,
    journal: bool,
    c: i32,
    g0: i32,
    mp_max: i32,
    health_dividers: i32,
    pool_dividers: i32,
    /// The monster bar's draw rule (T not 0 and the quick panel closed; SPEC-0023 10.2).
    monster: bool,
    /// The Square prompt slot is shown this pass (predicate slot 25).
    square: bool,
    /// The quick panel: open, its selection, its four entries' owned counts, the effective
    /// marker and its count; G1 and G2 to G4; whether Tor's weapon (category 1) and shield
    /// (category 2) entries are filled.
    quick_open: bool,
    selection: usize,
    counts: [i32; 4],
    marker: i32,
    marker_count: i32,
    g1: i32,
    g: [i32; 3],
    weapon: bool,
    shield: bool,
    /// The live petpet's kind while present (T at least 1).
    pet: Option<u8>,
    /// The active hero index (predicate slots 14 and 52).
    hero: usize,
    /// The owned counts of the Motes in the active hero's equipment words 0, 1 and 2
    /// (predicate slots 19, 20 and 17; SPEC-0023 section 5).
    words: [i32; 3],
    /// The portrait swing is active and the swap mode is 2 (predicate slot 60), each hero's
    /// available word (57 Tor, 59 Roberta: 0 draws the dimmed overlay) and dead test (58 with
    /// the hero as argument: the dead icon).
    swing: bool,
    available: [bool; 2],
    dead: [bool; 2],
    /// The Mote panel: open (slot 37), its selection (slot 15) and the owned counts of Motes 160
    /// to 173 (slot 16).
    mote_open: bool,
    mote_selection: u8,
    mote_counts: [i32; 14],
}

#[allow(clippy::too_many_arguments)]
pub fn sync_hud(
    mut commands: Commands,
    state: Option<ResMut<PlayState>>,
    art: Option<ResMut<HudArt>>,
    mut images: ResMut<Assets<Image>>,
    windows: Query<&Window>,
    mut canvas: Query<(Entity, &mut Node, &mut GlobalZIndex), With<HudCanvas>>,
    menu: Option<Res<crate::menu_view::Menu>>,
    mut sprites: Query<(&HudSprite, &mut Node, &mut ImageNode, &mut Visibility), (Without<HudCanvas>, Without<HudText>)>,
    mut texts: Query<(&mut Node, &mut ImageNode, &mut Visibility), (With<HudText>, Without<HudCanvas>, Without<HudSprite>, Without<HudCaption>)>,
    mut captions: Query<(&mut Node, &mut ImageNode, &mut Visibility), (With<HudCaption>, Without<HudCanvas>, Without<HudSprite>, Without<HudText>)>,
    mut values: Query<(&HudValue, &mut Node, &mut ImageNode, &mut Visibility), (Without<HudCaption>, Without<HudCanvas>, Without<HudSprite>, Without<HudText>)>,
    mut caption_image: Local<Option<(Vec<u8>, Handle<Image>, u32, u32, i32)>>,
    mut value_images: Local<HashMap<(usize, String), (Handle<Image>, u32, u32, i32)>>,
    mut prices: Query<(&HudPrice, &mut Node, &mut ImageNode, &mut Visibility), (Without<HudCanvas>, Without<HudSprite>, Without<HudText>, Without<HudCaption>, Without<HudValue>)>,
) {
    let _span = crate::perf::span("sync_hud");
    let (Some(mut state), Some(mut art)) = (state, art) else { return };
    let HudArt { f, mem, images: cache, name: name_image, .. } = &mut *art;
    let f: &Frontend = f;
    // SPEC-0023 section 11.1: the host's tween requests become tweens between two nodes' canvas
    // points: the quick fly-in moves 0xA1 from cross icon k, fading in; a Mote fly-in moves the
    // word's equipment icon (keys 164, 166 and 162 for words 0, 1 and 2) from the panel cell of
    // the selection (the handler-15 node with that argument under group 392).
    if !state.play.hud.tween_requests.is_empty() {
        let zero: HashMap<usize, i32> = HashMap::new();
        let none: HashMap<usize, (i32, i32, f32)> = HashMap::new();
        let canvas = |n: usize| {
            let (x, y, _, _) = place(f, n, &zero, &zero, &none);
            [x, y]
        };
        let requests = std::mem::take(&mut state.play.hud.tween_requests);
        for r in requests {
            let made = match r {
                neopets_sim::hud::TweenRequest::Quick { entry } => {
                    let from = [0x9C, 0x9B, 0x9E, 0x9D].get(entry).and_then(|&k| f.key(k));
                    from.zip(f.key(0xA1)).map(|(a, b)| neopets_sim::hud::Tween::new(b, canvas(a), canvas(b), neopets_sim::hud::FLY_IN, [128, 128, 128, 0], [128; 4]))
                }
                neopets_sim::hud::TweenRequest::Mote { selection, word } => {
                    let mut cells = Vec::new();
                    if f.nodes.len() > 392 {
                        subtree(f, 392, &mut cells);
                    }
                    let cell = cells.into_iter().find(|&n| f.nodes[n].handler == 15 && f.nodes[n].arg == selection);
                    let icon = [164u32, 166, 162].get(word).and_then(|&k| f.key(k));
                    cell.zip(icon).map(|(a, b)| neopets_sim::hud::Tween::new(b, canvas(a), canvas(b), neopets_sim::hud::FLY_IN, [128; 4], [128; 4]))
                }
            };
            if let Some(t) = made {
                state.play.hud.start_tween(t);
            }
        }
    }
    let state = state.into_inner();
    let Some(window) = windows.iter().next() else { return };
    let mut fill_points = [None; 2];
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
    // The panel roots: the cross (the topmost predicate-32 group above the right cross icon),
    // the item panel 0x30 and the equipment panels' group.
    let cross = f.key(CROSS_RIGHT).map(|n| {
        let (mut at, mut top) = (Some(n), n);
        while let Some(i) = at {
            if f.nodes[i].handler == 32 {
                top = i;
            }
            at = f.nodes[i].parent;
        }
        top
    });
    let item_panel = f.key(ITEM_PANEL);
    let equip = f.key(EQUIP[0]).and_then(|n| f.nodes[n].parent);
    let panels: Vec<usize> = [cross, item_panel, equip].into_iter().flatten().collect();
    let pet_panel = f.key(PET_PORTRAIT).and_then(|n| f.nodes[n].parent).and_then(|n| f.nodes[n].parent);
    // SPEC-0023 sections 8 and 9: the journal-notification icon (0xDF) and the D-pad hint (0xBB).
    let (journal_icon, dpad_icon) = (f.key(JOURNAL_ICON), f.key(DPAD_HINT));
    let mut nodes = Vec::new();
    for g in [journal_icon, dpad_icon].into_iter().flatten() {
        subtree(f, g, &mut nodes);
    }
    // SPEC-0021 / SPEC-0023 section 24: the Mote panel, group node 392.
    let mote_root = (f.nodes.len() > 392).then_some(392usize);
    if let Some(g) = mote_root {
        subtree(f, g, &mut nodes);
    }
    // SPEC-0023 section 12: the portrait swing's group (the pivot's parent).
    let swing_root = f.key(PIVOT).and_then(|n| f.nodes[n].parent);
    if let Some(g) = swing_root {
        subtree(f, g, &mut nodes);
    }
    subtree(f, bars, &mut nodes);
    if let Some(m) = monster {
        subtree(f, m, &mut nodes);
    }
    if let Some(g) = prompts {
        subtree(f, g, &mut nodes);
    }
    if let Some(g) = pet_panel {
        subtree(f, g, &mut nodes);
    }
    if std::env::var_os("NEOPETS_TRACE_HUD_HANDLERS").is_some() {
        let mut all = Vec::new();
        subtree(f, 322, &mut all);
        let mut hs: Vec<(i16, u8, u32)> = all.iter().map(|&n| (f.nodes[n].handler, f.nodes[n].mode, f.nodes[n].key as u32)).filter(|h| h.0 >= 0).collect();
        hs.sort();
        hs.dedup();
        println!("hud handlers {hs:?}");
    }
    let mut panel_nodes = Vec::new();
    for &r in &panels {
        subtree(f, r, &mut panel_nodes);
    }
    nodes.extend(panel_nodes.iter().copied());
    let Ok((root, mut root_node, mut root_z)) = canvas.single_mut() else {
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
        // Lines 0 and 1 are the price caption; 2 is the countdown timer text (element 0xD8).
        for line in 0..3 {
            let price = commands.spawn((Node { position_type: PositionType::Absolute, ..default() }, ImageNode::default(), Visibility::Hidden, HudPrice(line))).id();
            commands.entity(root).add_child(price);
        }
        // The Mote panel's label (node 393 under group 392, variable 11) is a value text too.
        let mut mote_texts = Vec::new();
        if f.nodes.len() > 392 {
            subtree(f, 392, &mut mote_texts);
        }
        for &n in panel_nodes.iter().chain(mote_texts.iter().filter(|&&n| f.nodes[n].kind == 6 && f.values.get(&f.nodes[n].data).is_some_and(|v| v.variable == 11))).filter(|&&n| matches!(f.nodes[n].kind, 2 | 6)) {
            let child = commands.spawn((Node { position_type: PositionType::Absolute, ..default() }, ImageNode::default(), Visibility::Hidden, HudValue(n))).id();
            commands.entity(root).add_child(child);
        }
        return;
    };
    let _ = root;
    root_node.left = Val::Px(ox);
    root_node.top = Val::Px(oy);
    root_node.width = Val::Px(512.0 * s);
    root_node.height = Val::Px(448.0 * s);
    let hud = &p.hud;
    let mb = &p.monster_bar;
    // The owned count of an item id (0 for -1 or an item not held).
    let count = |id: i32| u16::try_from(id).ok().and_then(|id| p.state.items.get(&id)).map_or(0, |&n| n as i32);
    // SPEC-0035 section 14 HUD lines: the item screen's tab states draw the HUD bars group too,
    // with a draw offset of (60, 23) on its top node, fully visible (they hold the timers at
    // 240); the HUD then draws over the item page.
    let items_open = p.spawned && menu.as_ref().is_some_and(|m| (m.flow.items()..m.flow.items() + 3).contains(&m.flow.current));
    *root_z = GlobalZIndex(if items_open { 100_000 } else { 10 });
    let in_bars = |n: usize| {
        let mut at = Some(n);
        while let Some(i) = at {
            if i == bars {
                return true;
            }
            at = f.nodes[i].parent;
        }
        false
    };
    // SPEC-0035 9c slot 62 (the busy test of SPEC-0023 section 4.1: the letterbox or a screen
    // fade is up).
    let busy = p.bars.height() > 0.0 || p.fade.moving() || p.fade.opacity() > 0.0;
    let hint_ok = !p.quick.open && !(mb.t != 0 && !p.quick.open) && !busy;
    let inputs = Inputs {
        c: if items_open { 240 } else { hud.c },
        g0: if items_open { 240 } else { hud.g0 },
        mp_max: p.hero_mp_max,
        health_dividers: hud.health.dividers,
        pool_dividers: hud.pool.dividers,
        monster: mb.t != 0 && !p.quick.open,
        square: p.square_caption().is_some(),
        quick_open: p.quick.open,
        selection: p.quick.selection,
        counts: p.quick.entries.map(|id| count(id)),
        marker: p.quick.effective(&|id| count(id) > 0),
        marker_count: count(p.quick.effective(&|id| count(id) > 0)),
        g1: hud.g1,
        g: hud.g,
        weapon: p.weapon.is_some(),
        shield: p.shield.is_some(),
        pet: p.pet_kind(),
        hero: p.hero_index,
        swing: hud.swing.active && p.swap_mode == 2,
        mote_open: p.mote_panel.open,
        mote_selection: p.mote_panel.selection,
        mote_counts: std::array::from_fn(|k| count(160 + k as i32)),
        available: hud.swing.available,
        dead: p.state.hero.dead,
        words: std::array::from_fn(|w| p.equipment.motes[p.hero_index.min(1)][w].map_or(-1, i32::from)).map(count),
        hint: hint_ok,
        journal: hint_ok && hud.journal,
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
        // SPEC-0023 section 7: W = round(A x 60 / B); below 1 the fill is hidden.
        (key(PET_FILL), p.petpet.as_ref().map_or(0, |pp| pp.meter())),
    ]
    .into_iter()
    .collect();
    // A live tween writes its moved node's canvas position (as an offset from the parent's
    // placement) and its colour on that node's subtree.
    let tweened: Vec<neopets_sim::hud::Tween> = hud.tweens.iter().filter(|t| t.live).copied().collect();
    // The swing's placements of the two portrait groups.
    let mut ov: HashMap<usize, (i32, i32, f32)> = HashMap::new();
    if let Some(pv) = f.key(PIVOT) {
        let q = PORTRAITS.map(|k| f.key(k).map_or((0, 0), |n| (f.nodes[n].pivot_x as i32, f.nodes[n].pivot_y as i32)));
        let placed = hud.swing.placement(f.nodes[pv].width as i32, (f.nodes[pv].pivot_x as i32, f.nodes[pv].pivot_y as i32), q);
        for (k, &key) in PORTRAITS.iter().enumerate() {
            if let Some(n) = f.key(key) {
                ov.insert(n, placed[k]);
            }
        }
    }
    for t in &tweened {
        let node = f.nodes[t.node];
        let (px, py, psx, psy) = match node.parent {
            Some(par) if !node.absolute => place(f, par, &HashMap::new(), &HashMap::new(), &ov),
            _ => (0.0, 0.0, 1.0, 1.0),
        };
        let lx = ((t.pos[0] as f32 - px) / psx.max(1e-6)).round() as i32;
        let ly = ((t.pos[1] as f32 - py) / psy.max(1e-6)).round() as i32;
        ov.insert(t.node, (lx, ly, node.scale_x as f32 / 4096.0));
    }
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
    // SPEC-0023 section 11: the item id each icon element shows, and its art node (the child of
    // key 0x9F at that id; none past the last child).
    let icon_children: Vec<usize> = f.key(0x9F).map(|g| f.children(g)).unwrap_or_default();
    let held = |id: i32| u16::try_from(id).ok().and_then(|id| p.state.items.get(&id)).is_some_and(|&n| n > 0);
    let icon_art: HashMap<usize, Option<usize>> = ITEM_ICONS
        .iter()
        .enumerate()
        .filter_map(|(k, &key)| {
            let id = if k < 4 { p.quick.entries[k] } else { p.quick.marker };
            let art = usize::try_from(id).ok().filter(|_| held(id)).and_then(|i| icon_children.get(i).copied());
            Some((f.key(key)?, art))
        })
        .collect();
    let mut wanted: Vec<(usize, u32, u32)> = Vec::new();
    for (sprite, mut node, mut image, mut vis) in &mut sprites {
        let n = sprite.0;
        // Visible when every node from the sprite up to the bars group passes its condition.
        let mut ok = shown || (items_open && in_bars(n));
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
        let panel = panel_root(f, n, &panels);
        let in_pet = |n: usize| {
            let mut at = Some(n);
            while let Some(i) = at {
                if Some(i) == pet_panel {
                    return true;
                }
                at = f.nodes[i].parent;
            }
            false
        };
        let under = |root: Option<usize>| {
            let mut at = Some(n);
            while let Some(i) = at {
                if Some(i) == root {
                    return true;
                }
                at = f.nodes[i].parent;
            }
            false
        };
        let (in_journal, in_dpad) = (under(journal_icon), under(dpad_icon));
        let in_swing = under(swing_root);
        let in_mote = under(mote_root);
        // Section 12: drawn while the swing is active and the swap mode is 2.
        if in_swing {
            ok &= hud.swing.active && p.swap_mode == 2;
        }
        // Both icons are hidden while the quick panel is open or the letterbox or a screen fade
        // is up (the busy test of SPEC-0023 section 4.1); the journal icon only shows while its
        // flag is set.
        if in_journal || in_dpad {
            let busy = p.bars.height() > 0.0 || p.fade.moving() || p.fade.opacity() > 0.0;
            ok &= !p.quick.open && !p.mote_panel.open && !busy && (!in_journal || hud.journal);
        }
        let top = if in_mote {
            mote_root.unwrap_or(bars)
        } else if in_swing {
            swing_root.unwrap_or(bars)
        } else if in_journal {
            journal_icon.unwrap_or(bars)
        } else if in_dpad {
            dpad_icon.unwrap_or(bars)
        } else if in_monster(n) {
            monster.unwrap_or(bars)
        } else if in_pet(n) {
            pet_panel.unwrap_or(bars)
        } else if in_prompts(n) {
            prompts.unwrap_or(bars)
        } else {
            panel.unwrap_or(bars)
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
        // An item icon element draws its item's art, or nothing.
        let art = icon_art.get(&n).copied().unwrap_or(Some(n));
        if !ok || w <= 0 || h <= 0 || art.is_none() {
            *vis = Visibility::Hidden;
            continue;
        }
        let art = art.unwrap_or(n);
        let (mut x, mut y, mut sx, sy) = place(f, n, &xs, &ws, &ov);
        if items_open {
            x += 60.0;
            y += 23.0;
        }
        if Some(n) == f.key(MONSTER_FILL) {
            // The fill is drawn with its x scale reset to 1.0.
            sx = 1.0;
        }
        // Each HUD group keeps its distance from its own window edge rather than from the
        // centred 4:3 canvas, so it stays at that side at any aspect ratio (this build's choice;
        // the canvas is the PS2's whole screen): the Mote panel and the button prompts stay
        // centred, the hero-swap portraits go right, the other named groups left, and anything
        // else (the bars group) by the third of the canvas its left edge lies in, so a long fill
        // stays with its frame.
        // Vertically the same way: the Mote panel stays centred, the button prompts, the D-pad
        // hint, the journal icon and the monster bar go to the bottom edge, the other named
        // groups to the top.
        let shift = if in_mote || top == prompts.unwrap_or(usize::MAX) {
            0.0
        } else if in_swing {
            ox
        } else if top != bars {
            -ox
        } else {
            edge_shift(x, 512.0, ox)
        };
        let shift_y = if in_mote {
            0.0
        } else if in_dpad || in_journal || in_monster(n) || top == prompts.unwrap_or(usize::MAX) {
            oy
        } else if top != bars {
            -oy
        } else {
            edge_shift(y, 448.0, oy)
        };
        // The health and pool fills' right edge and vertical centre, for the loss effects
        // (SPEC-0023 section 3), as fractions of the window.
        if let Some(k) = [HEALTH_FILL, POOL_FILL].iter().position(|&key| f.key(key) == Some(n)) {
            let (ww, wh) = (window.width().max(1.0), window.height().max(1.0));
            let ex = ox + shift + (x + w as f32 * sx) * s;
            let ey = oy + shift_y + (y + h as f32 * sy / 2.0) * s;
            fill_points[k] = Some(([ex / ww, ey / wh], ww / wh));
        }
        node.left = Val::Px(x * s + shift);
        node.top = Val::Px(y * s + shift_y);
        node.width = Val::Px(w as f32 * sx * s);
        node.height = Val::Px(h as f32 * sy * s);
        let base = f.sprite(n).map_or([128; 4], |sp| sp.colour);
        let rgb = if tinted.contains(&n) { [hud.tint[0], hud.tint[1], hud.tint[2]] } else { [base[0], base[1], base[2]] };
        let c = |v: u8| v as f32 / 128.0;
        let tween_alpha = tweened.iter().find(|t| under(Some(t.node))).map(|t| t.colour[3]);
        let a = if let Some(ta) = tween_alpha {
            ta
        } else if in_swing || in_mote {
            128
        } else if in_journal {
            hud.journal_alpha()
        } else if in_dpad {
            hud.dpad_alpha()
        } else if in_monster(n) {
            neopets_sim::hud::alpha(mb.t)
        } else if in_prompts(n) || in_pet(n) {
            128
        } else if panel.is_some() {
            group_alpha(f, n, &hud.panel_alpha)
        } else {
            if items_open { 128 } else { hud.bars_alpha }
        };
        image.color = Color::srgba(c(rgb[0]), c(rgb[1]), c(rgb[2]), c(a).min(1.0));
        match cache.get(&(art, w as u32, h as u32)) {
            Some(handle) => image.image = handle.clone(),
            None => wanted.push((art, w as u32, h as u32)),
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
                let (x, y, sx, sy) = place(f, nn, &xs, &ws, &ov);
                // With the monster bar at the window's left edge.
                node.left = Val::Px(x * s - ox);
                node.top = Val::Px((y - *base as f32 * sy) * s + oy);
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
                    // With the button prompts at the window's bottom edge.
                    node.top = Val::Px((395 - base) as f32 * s + oy);
                    node.width = Val::Px(*w as f32 * s);
                    node.height = Val::Px(*h as f32 * s);
                    image.image = handle.clone();
                    *vis = Visibility::Inherited;
                }
            }
            None => *vis = Visibility::Hidden,
        }
    }
    // SPEC-0035 section 15.2: the price caption, font 0, right edge at x 410, baselines 115 and
    // 140, while its counter runs and the quick panel is closed. Line 1: "Exorbitant!" for 32767,
    // else the nine-cell number in (160, 160, 90) or (128, 60, 90) above the balance; line 2 the
    // balance in (128, 128, 128).
    for (line, mut node, mut image, mut vis) in &mut prices {
        // SPEC-0023 section 13: element 0xD8 shows the countdown timer as M:SS:CC while it is
        // visible, right-aligned at x 436 with its baseline at y 77, in the element's value-text
        // font (font 0 when it has none) and the timer's colour word.
        if line.0 == 2 {
            if !p.state.timer.visible {
                *vis = Visibility::Hidden;
                continue;
            }
            let text = neopets_sim::timer::text(p.state.timer.value).into_bytes();
            let rec = f.key(0xD8).and_then(|n| f.values.get(&f.nodes[n].data).copied());
            let (font, spacing) = rec.map_or((0, 0), |v| (v.font.max(0) as usize, v.spacing as i32));
            let colour = p.state.timer.colour;
            let key = (9002, format!("{}{colour:?}", String::from_utf8_lossy(&text)));
            if !value_images.contains_key(&key) {
                if let Some((img, base)) = f.text_image(mem, font, &text, spacing, colour) {
                    let (w, h) = (img.width, img.height);
                    value_images.insert(key.clone(), (images.add(crate::rgba_image(w, h, img.rgba)), w, h, base));
                }
            }
            match value_images.get(&key) {
                Some((handle, w, h, base)) => {
                    // At the window's right edge, like the hero-swap portraits beside it.
                    node.left = Val::Px((436.0 - *w as f32) * s + ox);
                    node.top = Val::Px((77 - base) as f32 * s - oy);
                    node.width = Val::Px(*w as f32 * s);
                    node.height = Val::Px(*h as f32 * s);
                    image.image = handle.clone();
                    *vis = Visibility::Inherited;
                }
                None => *vis = Visibility::Hidden,
            }
            continue;
        }
        let (value, counter) = p.price_lines[line.0];
        if counter == 0 || p.quick.open {
            *vis = Visibility::Hidden;
            continue;
        }
        let (text, colour) = if line.0 == 1 {
            (neopets_sim::shop::number_field(value, 9), [128, 128, 128, 127])
        } else if value == 32767 {
            (p.exorbitant.clone(), [128, 60, 90, 127])
        } else if value > p.state.neopoints {
            (neopets_sim::shop::number_field(value, 9), [128, 60, 90, 127])
        } else {
            (neopets_sim::shop::number_field(value, 9), [160, 160, 90, 127])
        };
        let key = (9000 + line.0, format!("{}{colour:?}", String::from_utf8_lossy(&text)));
        if !value_images.contains_key(&key) {
            if let Some((img, base)) = f.text_image(mem, 0, &text, 0, colour) {
                let (w, h) = (img.width, img.height);
                value_images.insert(key.clone(), (images.add(crate::rgba_image(w, h, img.rgba)), w, h, base));
            }
        }
        match value_images.get(&key) {
            Some((handle, w, h, base)) => {
                let y = if line.0 == 0 { 115 } else { 140 };
                node.left = Val::Px((410.0 - *w as f32) * s + ox);
                node.top = Val::Px((y - base) as f32 * s - oy);
                node.width = Val::Px(*w as f32 * s);
                node.height = Val::Px(*h as f32 * s);
                image.image = handle.clone();
                *vis = Visibility::Inherited;
            }
            None => *vis = Visibility::Hidden,
        }
    }
    // SPEC-0023 sections 5 and 11, SPEC-0035 section K: the panels' counts, as plain decimal in
    // the record's font, spacing and colour, aligned by the node's width, baseline at its y.
    // Which number each shows follows its predicate (the cross entries' counts, the equipment
    // words' counts) or, ungated, the selected item's count; the variable table at 0x2DD200
    // holds run-time pointers and is not read.
    for (value, mut node, mut image, mut vis) in &mut values {
        let n = value.0;
        // A label draws its binding's text (SPEC-0035 section 12, Text node draw); a value text
        // its number (section K).
        let (text, font, spacing, colour, ok0) = if f.nodes[n].kind == 2 {
            match f.label(n).filter(|l| l.font >= 0) {
                Some(l) => (String::from_utf8_lossy(&l.text).into_owned(), l.font as usize, l.spacing as i32, l.colour, true),
                None => continue,
            }
        } else if under_node(f, n, 392) && f.values.get(&f.nodes[n].data).is_some_and(|v| v.variable == 11) {
            // SPEC-0021 Second quick panel item 5: the label shows the owned count of the selected
            // Mote, and nothing for "No Mote" (the name beside it is not built: its format in the
            // label is not given).
            let Some(v) = f.values.get(&f.nodes[n].data).copied() else { continue };
            let sel = inputs.mote_selection;
            let text = if sel == 255 { String::new() } else { inputs.mote_counts.get(sel as usize).copied().unwrap_or(0).to_string() };
            (text, v.font as usize, v.spacing as i32, v.colour, inputs.mote_open && sel != 255)
        } else {
            let Some(v) = f.values.get(&f.nodes[n].data).copied() else { continue };
            let number = match f.nodes[n].handler {
                30 => inputs.counts[0],
                33 => inputs.counts[1],
                34 => inputs.counts[2],
                36 => inputs.counts[3],
                17 => inputs.words[2],
                19 => inputs.words[0],
                20 => inputs.words[1],
                _ => inputs.marker_count,
            };
            (number.to_string(), v.font as usize, v.spacing as i32, v.colour, v.gate != 0 && v.format == 1)
        };
        let mut ok = shown && ok0;
        let top = if under_node(f, n, 392) { 392 } else { panel_root(f, n, &panels).unwrap_or(n) };
        let mut at = Some(n);
        while let Some(i) = at {
            ok &= condition(f, i, &inputs);
            if i == top {
                break;
            }
            at = f.nodes[i].parent;
        }
        if !ok {
            *vis = Visibility::Hidden;
            continue;
        }
        let key = (n, text.clone());
        if !value_images.contains_key(&key) {
            if let Some((img, base)) = f.text_image(mem, font, text.as_bytes(), spacing, colour) {
                let (w, h) = (img.width, img.height);
                value_images.insert(key.clone(), (images.add(crate::rgba_image(w, h, img.rgba)), w, h, base));
            }
        }
        let Some((handle, w, h, base)) = value_images.get(&key) else { continue };
        let (x, y, sx, sy) = place(f, n, &xs, &ws, &ov);
        // With its panel: the Mote panel's stays centred, a panel's goes to the left edge, and
        // one outside a panel by the third of the canvas it lies in.
        let (shift, shift_y) = if top == 392 {
            (0.0, 0.0)
        } else if top != n {
            (-ox, -oy)
        } else {
            (edge_shift(x, 512.0, ox), edge_shift(y, 448.0, oy))
        };
        node.left = Val::Px(x * s + shift);
        node.top = Val::Px((y - *base as f32 * sy) * s + shift_y);
        node.width = Val::Px(*w as f32 * sx * s);
        node.height = Val::Px(*h as f32 * sy * s);
        image.image = handle.clone();
        image.color = Color::srgba(1.0, 1.0, 1.0, (group_alpha(f, n, &hud.panel_alpha) as f32 / 128.0).min(1.0));
        *vis = Visibility::Inherited;
    }
    for (n, w, h) in wanted {
        if let Some(img) = f.sprite_image(mem, n, w, h) {
            let handle = images.add(crate::rgba_image(img.width, img.height, img.rgba));
            cache.insert((n, w, h), handle);
        }
    }
    state.play.hud_fill_points = fill_points;
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
    let _span = crate::perf::span("sync_textbox");
    let (Some(state), Some(mut art)) = (state, art) else { return };
    let Some(window) = windows.iter().next() else { return };
    let Ok(root) = canvas.single() else { return };
    if parts.is_empty() {
        // Above the front-end pages: a card message shows over the save and load screens.
        // Each part on its own layer, the frame lowest: parts sharing one global z-index have no
        // fixed order, and the frame was sometimes drawn over the name and the prompt icon.
        for (k, part) in [BoxPart::Frame, BoxPart::Name, BoxPart::Body, BoxPart::Icon, BoxPart::Label, BoxPart::Icon2, BoxPart::Label2].into_iter().enumerate() {
            let e = commands.spawn((Node { position_type: PositionType::Absolute, ..default() }, ImageNode::default(), Visibility::Hidden, part, GlobalZIndex(1000 + k as i32))).id();
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
            // SPEC-0022 Prompt row: a LevelPrompt's last page shows no "Finish" but the two
            // choices, Circle and Triangle, with the table-13 labels 0x21 "Yes" and 0x22 "No"
            // (every live call passes -1 for its labels, SPEC-0030 row 100).
            let prompt = if p.textbox.more() {
                Prompt::Continue(&p.box_labels.0)
            } else if p.textbox.choice {
                let label = |i: usize| p.labels.get(i).map_or(&[][..], |l| &l[..]);
                Prompt::Choice(label(0x21), label(0x22))
            } else {
                Prompt::Continue(&p.box_labels.1)
            };
            (&p.textbox, name, colour, prompt)
        }
        None => {
            for (_, _, _, mut vis) in &mut parts {
                *vis = Visibility::Hidden;
            }
            return;
        }
    };
    let s = (window.width() / 512.0).min(window.height() / 448.0);
    let art_style = art.style;
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
        (BoxPart::Icon, hash((right.map(|r| r.0), art_style))),
        (BoxPart::Label, hash(right.map(|r| r.1))),
        (BoxPart::Icon2, hash((left.map(|r| r.0), art_style))),
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
