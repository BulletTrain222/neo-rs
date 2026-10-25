//! The front-end screens (SPEC-0035 Behavior 1b, 2, 4, 9, 9b, 9d, 12 and 13): the screen
//! stack with its parent-index and back rules, the generic menu step (Cross to the focused
//! item's target, Triangle back, Down and Up on list pages with wrap, off for a screen's first
//! two passes), the focus rules, the states of the card check (0), logo countdown (29),
//! attract (1), title (12), play (11), pause page (7), Resume (5) and Quit (6) focus states, the
//! game-over page (26), the load (21) and save (22) screens with their slot picker (section 7;
//! the card jobs mapped onto the clean side's own file, SPEC-0025), and the page draw list
//! (condition modes and predicates, the focused flag of kind-4 items, row spacing, alignment and
//! scale) with the picker's rows by the journal row drawer (section 14.3).
//!
//! Not covered: the memory card (the card state passes at once), movies (the attract goes
//! straight to the title), the quit box (Quit goes back to the card state without asking, and
//! the world is not torn down), the card message boxes (the overwrite and save questions, the
//! busy text, the failure and cancel boxes), options, credits, the item screens, journal and
//! maps (their pages open and Triangle leaves them), sounds, key masks and animation slots,
//! spatial pages, and the predicates other than those listed in `predicate`.

use neopets_data::frontend::Frontend;
use neopets_sim::save::{self, SaveFile};

pub const CARD: usize = 0;
pub const ATTRACT: usize = 1;
pub const PAUSE: usize = 2;
pub const PLAY: usize = 9;
pub const TITLE: usize = 10;
pub const GAME_OVER: usize = 23;
pub const LOGO: usize = 30;
/// The load prompt (record 17, state 21) and the save prompt (record 18, state 22).
pub const LOAD: usize = 17;
pub const SAVE: usize = 18;

/// Menu bits of the rising-edge word (SPEC-0035 section 4).
pub const UP: u16 = 0x1000;
pub const DOWN: u16 = 0x4000;
pub const CROSS: u16 = 0x0040;
pub const TRIANGLE: u16 = 0x0010;
pub const SELECT: u16 = 0x0100;

/// What the game around the front end must do.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Command {
    NewGame,
    Quit,
    /// Read the save file and report it with `Flow::scanned` (the load or the save scan).
    Scan,
    /// Rebuild slot k from the live game, write the file and report with `Flow::written`.
    Save(usize),
    /// Apply slot k of the scanned file.
    Load(usize),
}

/// The slot picker of the load and save screens (SPEC-0035 section 7): the stage, the cursor,
/// the scanned file, the selected row's pulse and its direction.
#[derive(Clone, Debug, PartialEq)]
pub struct Picker {
    pub stage: u8,
    pub cursor: usize,
    pub file: SaveFile,
    /// The scan found no readable file (the load screen leaves).
    failed: bool,
    pub pulse: f32,
    step: f32,
}

impl Default for Picker {
    fn default() -> Self {
        Self { stage: 0, cursor: 0, file: SaveFile::empty(), failed: false, pulse: 0.0, step: 0.0 }
    }
}

/// The row drawer's data words (SPEC-0035 section 14.3; boot 0x2DCDA8 to 0x2DCDD4): the pulse
/// bounds and step, the bar width, the objective wrap width, the title and objective colours.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct RowWords {
    pub pulse_min: f32,
    pub pulse_max: f32,
    pub pulse_step: f32,
    pub bar_width: f32,
    pub wrap: i32,
    pub title_colour: [u8; 4],
    pub text_colour: [u8; 4],
}

/// The save and load picker's bar colour (section 7.1; a constant of the row drawer, not a data
/// word).
const PICKER_BAR: [u8; 3] = [78, 119, 174];

/// One thing to draw on the canvas.
#[derive(Clone, Debug, PartialEq)]
pub enum Draw {
    /// A sprite node's quad: canvas rect, the node's colour, and the texels it samples (one
    /// per pixel before the node scale; a wider quad shows more texels, section 12).
    Sprite { node: usize, x: f32, y: f32, w: f32, h: f32, colour: [u8; 4], texels: [u32; 2] },
    /// A label node's text, with its baseline at y.
    Label { node: usize, x: f32, y: f32, scale: f32 },
    /// A run of text in a font, with its baseline at y.
    Text { font: usize, text: Vec<u8>, spacing: i32, x: f32, y: f32, colour: [u8; 4] },
}

#[derive(Clone, Debug, PartialEq)]
pub struct Flow {
    pub current: usize,
    stack: Vec<usize>,
    /// The current screen's pass count and focus index.
    pub pass: u32,
    pub focus: usize,
    attract_visits: u32,
    idle: u32,
    pub picker: Picker,
    pub rows: RowWords,
}

impl Flow {
    /// Front-end start-up: a direct set of record 0 (the card state).
    pub fn new(fe: &Frontend) -> Self {
        let mut f = Self { current: CARD, stack: vec![CARD], pass: 0, focus: 0, attract_visits: 0, idle: 0, picker: Picker::default(), rows: RowWords::default() };
        f.default_focus(fe, &Ctx::default());
        f
    }

    /// A direct screen set (record 0 at start-up, or play for a diagnostic start).
    pub fn set(&mut self, fe: &Frontend, r: usize) {
        self.stack = vec![r];
        self.current = r;
        self.pass = 0;
        self.default_focus(fe, &Ctx::default());
    }

    /// The save screen's opening from a save point (SPEC-0035 section 7.3): a direct set of
    /// record 18 with the stage and the slot cursor at 0.
    pub fn open_save(&mut self, fe: &Frontend) {
        self.set(fe, SAVE);
        self.picker.stage = 0;
        self.picker.cursor = 0;
    }

    /// The scan's result: the file read (`None`: missing, unreadable or not 0xC400 bytes). The
    /// save scan does not check its read, so it goes on with four empty slots.
    pub fn scanned(&mut self, file: Option<SaveFile>) {
        self.picker.failed = file.is_none();
        self.picker.file = file.unwrap_or_default();
        if self.current == LOAD {
            self.picker.cursor = save::load_start(self.picker.file.filled());
        }
    }

    /// The write job's result: written, or not (the save screen scans again).
    pub fn written(&mut self, fe: &Frontend, ok: bool) {
        if self.current != SAVE {
            return;
        }
        if ok {
            self.enter(fe, PLAY, &Ctx::default());
        } else {
            self.picker.stage = 0;
        }
    }

    /// The picker's pulse, once per pass: add the step, bounce at the bounds.
    fn pulse_tick(&mut self) {
        let (lo, hi) = (self.rows.pulse_min, self.rows.pulse_max);
        let p = &mut self.picker;
        if p.step == 0.0 {
            p.step = self.rows.pulse_step;
        }
        p.pulse += p.step;
        if p.pulse > hi {
            p.pulse = hi;
            p.step = -p.step;
        } else if p.pulse < lo {
            p.pulse = lo;
            p.step = -p.step;
        }
    }

    /// The picker's Down and Up (Down tested first); every move sets the pulse to its maximum.
    fn picker_moves(&mut self, edge: u16, load: bool) {
        let filled = self.picker.file.filled();
        let down = edge & DOWN != 0;
        if down || edge & UP != 0 {
            self.picker.cursor = save::cursor_step(self.picker.cursor, down, load, filled);
            self.picker.pulse = self.rows.pulse_max;
        }
    }

    fn back_code(fe: &Frontend, r: usize) -> i16 {
        fe.screens.get(r).map_or(-3, |s| s.back)
    }

    /// Enter a record by its parent-index rule (section 9), then make it current with its pass
    /// count at 0 and its default focus.
    fn enter(&mut self, fe: &Frontend, r: usize, ctx: &Ctx) {
        match Self::back_code(fe, r) {
            -1 => {
                if self.stack.last() != Some(&r) && self.stack.len() < 32 {
                    self.stack.push(r);
                }
            }
            -2 => self.stack = vec![r],
            -3 => {}
            p if p >= 0 => {
                let p = p as usize;
                match self.stack.iter().rposition(|&s| s == p) {
                    Some(i) => self.stack.truncate(i + 1),
                    None => self.stack = vec![p],
                }
                if self.stack.len() < 32 {
                    self.stack.push(r);
                }
            }
            _ => {}
        }
        self.current = r;
        self.pass = 0;
        self.default_focus(fe, ctx);
    }

    /// The back rule: pop and enter the new top (or the record's parent index when only the
    /// base entry was left).
    fn back(&mut self, fe: &Frontend, ctx: &Ctx) {
        let code = Self::back_code(fe, self.current);
        if self.stack.len() > 1 {
            self.stack.pop();
            let top = *self.stack.last().unwrap();
            self.current = top;
            self.pass = 0;
            self.default_focus(fe, ctx);
        } else if code >= 0 {
            self.enter(fe, code as usize, ctx);
        }
    }

    fn default_focus(&mut self, fe: &Frontend, ctx: &Ctx) {
        let items = items(fe, self.current);
        let page = page_of(fe, self.current);
        self.focus = page.map_or(0, |p| p.focus as usize);
        if items.get(self.focus).is_none_or(|&i| !item_ok(fe, i, self.current, ctx, self)) {
            self.focus = items.iter().position(|&i| item_ok(fe, i, self.current, ctx, self)).unwrap_or(0);
        }
    }

    /// One front-end pass. `edge` is the rising-edge menu word, `second` the button-only word.
    pub fn tick(&mut self, fe: &Frontend, edge: u16, second: u16, ctx: &Ctx) -> Option<Command> {
        let mut command = None;
        let state = fe.screens.get(self.current).map_or(0xFFFF, |s| s.state);
        // The screen state; 0 lets the generic step run, -4 keeps only the focus moves.
        let mut result: Option<usize> = None;
        let mut generic = true;
        match state {
            0 => result = Some(LOGO),
            29 if self.pass >= 300 => result = Some(ATTRACT),
            1 => {
                let wait = if self.attract_visits == 0 { 300 } else { 1 };
                if self.pass + 1 >= wait {
                    self.attract_visits += 1;
                    result = Some(TITLE);
                }
            }
            12 => {
                if self.pass >= 1 {
                    self.idle = if edge == 0 { self.idle + 1 } else { 0 };
                } else {
                    self.idle = 0;
                }
                if self.idle > 1800 {
                    result = Some(LOGO);
                } else if self.focus == 0 && self.pass >= 5 && edge & CROSS != 0 {
                    command = Some(Command::NewGame);
                    result = Some(PLAY);
                }
            }
            11 => {
                if ctx.game_over {
                    result = Some(GAME_OVER);
                } else if second & SELECT != 0 {
                    result = Some(PAUSE);
                }
                // Play's own buttons belong to the hero; the HUD page has no menu items.
                generic = false;
            }
            7 if second & SELECT != 0 => result = Some(PLAY),
            26 => {
                if !ctx.game_over {
                    result = Some(PLAY);
                }
                generic = false;
            }
            // SPEC-0035 section 7.2, the load screen: the scan (stages 0 and 1 keep Triangle
            // from acting), then the picker; Cross loads the chosen slot and returns to play.
            21 => {
                if self.pass == 0 {
                    self.picker.stage = 0;
                }
                match self.picker.stage {
                    0 => {
                        command = Some(Command::Scan);
                        self.picker.stage = 1;
                        generic = false;
                    }
                    1 => {
                        if self.picker.failed || !self.picker.file.filled().contains(&true) {
                            self.back(fe, ctx);
                            return command;
                        }
                        self.picker.stage = 2;
                        self.picker.pulse = self.rows.pulse_max;
                        generic = false;
                    }
                    _ => {
                        self.pulse_tick();
                        self.picker_moves(edge, true);
                        if edge & CROSS != 0 {
                            if self.picker.cursor < save::SLOTS {
                                command = Some(Command::Load(self.picker.cursor));
                                result = Some(PLAY);
                            }
                            generic = false;
                        }
                    }
                }
            }
            // Section 7.3, the save screen: scan, picker, write; Triangle in the picker returns
            // to play without writing.
            22 => {
                generic = false;
                match self.picker.stage {
                    0 => {
                        command = Some(Command::Scan);
                        self.picker.stage = 1;
                    }
                    1 => {
                        self.picker.stage = 2;
                        self.picker.pulse = self.rows.pulse_max;
                    }
                    2 => {
                        self.pulse_tick();
                        self.picker_moves(edge, false);
                        if edge & CROSS != 0 {
                            command = Some(Command::Save(self.picker.cursor.min(save::SLOTS - 1)));
                            self.picker.stage = 3;
                        } else if edge & TRIANGLE != 0 {
                            result = Some(PLAY);
                        }
                    }
                    _ => {}
                }
            }
            _ => {}
        }
        // The focused item: refocus to the first passing item when it fails; its focus state.
        let list = items(fe, self.current);
        if list.get(self.focus).is_none_or(|&i| !item_ok(fe, i, self.current, ctx, self)) {
            self.focus = list.iter().position(|&i| item_ok(fe, i, self.current, ctx, self)).unwrap_or(0);
        }
        if result.is_none() {
            let focus_state = list.get(self.focus).and_then(|i| fe.targets.get(i)).map_or(-1, |t| t.state);
            match focus_state {
                5 if second & CROSS != 0 => result = Some(PLAY),
                6 if second & CROSS != 0 => {
                    command = Some(Command::Quit);
                    result = Some(CARD);
                }
                _ => {}
            }
        }
        // The generic step, from the screen's third pass.
        if result.is_none() && generic && self.pass >= 2 {
            if edge & CROSS != 0 {
                if let Some(t) = list.get(self.focus).and_then(|i| fe.targets.get(i)) {
                    match t.target {
                        -3 => {}
                        -1 => {
                            self.back(fe, ctx);
                            return command;
                        }
                        r if r >= 0 => result = Some(r as usize),
                        _ => {}
                    }
                }
            } else if edge & TRIANGLE != 0 {
                let code = Self::back_code(fe, self.current);
                if code != -3 && code != -2 {
                    self.back(fe, ctx);
                    return command;
                }
            } else if page_of(fe, self.current).is_some_and(|p| p.list) && edge & (DOWN | UP) != 0 {
                let passing: Vec<usize> = (0..list.len()).filter(|&k| item_ok(fe, list[k], self.current, ctx, self)).collect();
                if let Some(at) = passing.iter().position(|&k| k == self.focus) {
                    let n = passing.len();
                    self.focus = passing[if edge & DOWN != 0 { (at + 1) % n } else { (at + n - 1) % n }];
                }
            }
        }
        match result {
            Some(r) => self.enter(fe, r, ctx),
            None => self.pass = (self.pass + 1).min(9999),
        }
        command
    }
}

/// What the predicates read from the game.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Ctx {
    pub game_over: bool,
}

/// Predicate table A (section 9c), the slots this build implements: 0, 2 and 56 always 1;
/// 12, 27, 39 and 40 always 0; 66 to 68 the option words (all 0 here); 72 the game-over page's
/// pass count above twice the tick rate. Other slots read 0.
fn predicate(slot: i16, _arg: u8, pass: u32) -> i32 {
    match slot {
        0 | 2 | 56 => 1,
        72 => (pass > 120) as i32,
        _ => 0,
    }
}

fn mode_passes(fe: &Frontend, n: usize, p: i32, focused: bool) -> bool {
    match fe.nodes[n].mode {
        0 => true,
        1 => focused,
        2 => p != 0,
        3 => p == 0,
        _ => false,
    }
}

/// The predicate value a node's own handler gives (1 with none).
fn own_predicate(fe: &Frontend, n: usize, pass: u32) -> i32 {
    let node = fe.nodes[n];
    if node.handler >= 0 { predicate(node.handler, node.arg, pass) } else { 1 }
}

/// A selectable item passes when the mode tests from its parent up to the record's node pass.
fn item_ok(fe: &Frontend, item: usize, record: usize, _ctx: &Ctx, flow: &Flow) -> bool {
    let top = fe.screens.get(record).and_then(|s| s.node);
    let mut at = fe.nodes[item].parent;
    while let Some(i) = at {
        let n = fe.nodes[i];
        let p = if n.handler >= 0 { predicate(n.handler, n.arg, flow.pass) } else { 1 };
        let ok = match n.mode {
            0 | 1 => true,
            2 => p != 0,
            3 => p == 0,
            _ => true,
        };
        if !ok {
            return false;
        }
        if Some(i) == top {
            break;
        }
        at = n.parent;
    }
    true
}

/// The kind-4 items of a record's tree in search order (their children are not searched).
pub fn items(fe: &Frontend, record: usize) -> Vec<usize> {
    fn walk(fe: &Frontend, n: usize, out: &mut Vec<usize>) {
        if fe.nodes[n].kind == 4 {
            out.push(n);
            return;
        }
        for c in fe.children(n) {
            walk(fe, c, out);
        }
    }
    let mut out = Vec::new();
    if let Some(root) = fe.screens.get(record).and_then(|s| s.node) {
        walk(fe, root, &mut out);
    }
    out
}

fn page_of(fe: &Frontend, record: usize) -> Option<neopets_data::frontend::Page> {
    let root = fe.screens.get(record)?.node?;
    let mut stack = vec![root];
    while let Some(n) = stack.pop() {
        if let Some(p) = fe.pages.get(&n) {
            return Some(*p);
        }
        stack.extend(fe.children(n));
    }
    None
}

/// The draw list of a record's tree (section 12): conditions with the nearest handler's
/// predicate, the focused flag under the focused kind-4 item, row spacing over visible
/// siblings, alignment by the node's width, and scale.
pub fn draw_list(fe: &Frontend, record: usize, focus: usize, pass: u32) -> Vec<Draw> {
    let mut out = Vec::new();
    let Some(root) = fe.screens.get(record).and_then(|s| s.node) else { return out };
    let mut counter = 0usize;
    draw_node(fe, root, (0.0, 0.0, 1.0, 1.0), false, focus, pass, &mut counter, &mut out);
    out
}

/// The slot picker's rows (sections 7.1 and 14.3): four rows by the journal row drawer, each at
/// offset p below the first; the cursor's row has its bar pulsed. Shown from the picker stage.
pub fn picker_draws(fe: &Frontend, flow: &Flow) -> Vec<Draw> {
    let mut out = Vec::new();
    if !matches!(flow.current, LOAD | SAVE) || flow.picker.stage < 2 {
        return out;
    }
    let w = &flow.rows;
    let node = |k: u32| fe.key(k);
    let (Some(left), Some(right), Some(bar), Some(icons), Some(title), Some(objective)) =
        (node(0xD3), node(0x37), node(0x36), node(0xDD), node(0x34), node(0x33))
    else {
        return out;
    };
    let font_of = |n: usize| fe.label(n).filter(|l| l.font >= 0).map(|l| (l.font as usize, l.spacing as i32));
    // The fonts come from the two label bindings; the objective binding's spacing word (9990
    // on disc) is not a letter spacing, so both texts are drawn with spacing 0 (R-0019).
    let (Some((title_font, _)), Some((text_font, _))) = (font_of(title), font_of(objective)) else { return out };
    let (title_spacing, text_spacing) = (0, 0);
    let line = fe.fonts.get(text_font).map_or(14, |f| f.line_height());
    let sprite = |n: usize, p: f32, w0: Option<f32>, colour: Option<[u8; 4]>| {
        let d = fe.nodes[n];
        let width = w0.unwrap_or(d.width as f32);
        Draw::Sprite {
            node: n,
            x: d.x as f32,
            y: d.y as f32 + p,
            w: width,
            h: d.height as f32,
            texels: [width.max(1.0) as u32, d.height.max(1) as u32],
            colour: colour.unwrap_or_else(|| fe.sprite(n).map_or([128; 4], |s| s.colour)),
        }
    };
    let mut p = 0.0f32;
    for (k, slot) in flow.picker.file.slots.iter().enumerate() {
        let (kind, line1, line2): (usize, &[u8], &[u8]) = if slot.is_empty() {
            (7, b"Empty", b"")
        } else {
            ((slot.tier() as i32 + 2).clamp(0, 7) as usize, slot.location(), slot.date())
        };
        let lines = fe.wrap(text_font, line2, w.wrap, text_spacing);
        let limit = ((360.0 - (128.0 + p)) / line as f32).trunc() as i32 + 2;
        let shown = lines.len().min(limit.max(0) as usize);
        let next = p + 79f32.max(75.0 + line as f32 * shown as f32);
        if next >= 360.0 {
            break;
        }
        let colour = if k == flow.picker.cursor {
            let c = |v: u8| (v as f32 * flow.picker.pulse).round().clamp(0.0, 255.0) as u8;
            [c(PICKER_BAR[0]), c(PICKER_BAR[1]), c(PICKER_BAR[2]), 128]
        } else {
            [PICKER_BAR[0], PICKER_BAR[1], PICKER_BAR[2], 128]
        };
        out.push(sprite(left, p, None, None));
        out.push(sprite(right, p, None, None));
        out.push(sprite(bar, p, Some(w.bar_width), Some(colour)));
        if let Some(&icon) = fe.children(icons).get(kind) {
            out.push(sprite(icon, p, None, None));
        }
        let (tx, ty) = (fe.nodes[title].x as f32, fe.nodes[title].y as f32 + p);
        out.push(Draw::Text { font: title_font, text: line1.to_vec(), spacing: title_spacing, x: tx, y: ty, colour: w.title_colour });
        let (ox, oy) = (fe.nodes[objective].x as f32, fe.nodes[objective].y as f32 + p);
        if limit != 0 {
            for (i, &(a, b)) in lines.iter().take(shown).enumerate() {
                out.push(Draw::Text { font: text_font, text: line2[a..b].to_vec(), spacing: text_spacing, x: ox, y: oy + (line as usize * i) as f32, colour: w.text_colour });
            }
        }
        p = next;
    }
    out
}

fn nearest_predicate(fe: &Frontend, n: usize, pass: u32) -> i32 {
    let mut at = Some(n);
    while let Some(i) = at {
        if fe.nodes[i].handler >= 0 {
            return own_predicate(fe, i, pass);
        }
        at = fe.nodes[i].parent;
    }
    1
}

#[allow(clippy::too_many_arguments)]
fn draw_node(fe: &Frontend, n: usize, t: (f32, f32, f32, f32), focused: bool, focus: usize, pass: u32, counter: &mut usize, out: &mut Vec<Draw>) {
    let node = fe.nodes[n];
    if node.kind != 4 && !mode_passes(fe, n, nearest_predicate(fe, n, pass), focused) {
        return;
    }
    let (px, py, psx, psy) = t;
    let (sx, sy) = (psx * node.scale_x as f32 / 4096.0, psy * node.scale_y as f32 / 4096.0);
    let (w, h) = (node.width as f32, node.height as f32);
    let x = px;
    let y = py;
    let mut child_focused = focused;
    match node.kind {
        1 => {
            if let Some(s) = fe.sprite(n) {
                out.push(Draw::Sprite { node: n, x, y, w: w * sx, h: h * sy, colour: s.colour, texels: [node.width.max(1) as u32, node.height.max(1) as u32] });
            }
        }
        2 => out.push(Draw::Label { node: n, x, y, scale: sx }),
        4 => {
            child_focused = *counter == focus;
            *counter += 1;
        }
        _ => {}
    }
    // Children: placed from this node's transform, row spacing over visible siblings.
    let children = fe.children(n);
    let mut visible_after_first = 0;
    for (i, &c) in children.iter().enumerate() {
        let cn = fe.nodes[c];
        let passes = cn.kind == 4 || mode_passes(fe, c, nearest_predicate(fe, c, pass), child_focused);
        if i > 0 && passes {
            visible_after_first += 1;
        }
        let rows = if i == 0 { 0 } else { visible_after_first };
        let (bx, by, bsx, bsy) = if cn.absolute { (0.0, 0.0, 1.0, 1.0) } else { (x, y, sx, sy) };
        let csx = bsx * cn.scale_x as f32 / 4096.0;
        let shift = match cn.align {
            1 => cn.width as f32 * csx / 2.0,
            2 => cn.width as f32 * csx,
            _ => 0.0,
        };
        let cx = bx + cn.x as f32 * bsx - shift;
        let cy = by + (cn.y as f32 + node.spacing as f32 * rows as f32) * bsy;
        draw_node(fe, c, (cx, cy, bsx, bsy), child_focused, focus, pass, counter, out);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use neopets_data::frontend::{Node, Page, Screen, Target};

    /// A two-screen front end: record 0 (state 0) and a title record 10 (state 12) whose page
    /// holds three items; item 1 targets record 11 (state none, back -1).
    fn fe() -> Frontend {
        let mut f = Frontend::default();
        let node = |kind: u8, parent: Option<usize>| Node { kind, parent, scale_x: 4096, scale_y: 4096, handler: -1, key: 0xFFFF, ..Node::default() };
        // 0: card root; 1: title container; 2: page root; 3, 4, 5: items; 6: options container.
        f.nodes = vec![node(0, None), node(0, None), node(3, Some(1)), node(4, Some(2)), node(4, Some(2)), node(4, Some(2)), node(0, None)];
        f.nodes[1].child = Some(2);
        f.nodes[2].child = Some(3);
        f.nodes[3].sibling = Some(4);
        f.nodes[4].sibling = Some(5);
        f.nodes[2].spacing = 31;
        f.screens = vec![Screen { node: Some(0), back: -2, state: 0 }; 31];
        f.screens[10] = Screen { node: Some(1), back: -2, state: 12 };
        f.screens[11] = Screen { node: Some(6), back: -1, state: 0xFFFF };
        f.screens[9] = Screen { node: Some(0), back: -2, state: 11 };
        f.screens[30] = Screen { node: Some(0), back: -2, state: 29 };
        f.screens[1] = Screen { node: Some(0), back: -2, state: 1 };
        f.pages.insert(2, Page { focus: 0, list: true });
        f.targets.insert(3, Target { target: -3, state: -1 });
        f.targets.insert(4, Target { target: 11, state: -1 });
        f
    }

    #[test]
    fn start_up_reaches_the_title_after_the_logo_and_attract() {
        let f = fe();
        let mut flow = Flow::new(&f);
        let ctx = Ctx::default();
        let mut ticks = 0;
        while flow.current != TITLE && ticks < 1000 {
            flow.tick(&f, 0, 0, &ctx);
            ticks += 1;
        }
        // Card 1 pass, logo 301 passes, attract 300 passes on the first visit.
        assert_eq!(ticks, 1 + 301 + 300);
    }

    #[test]
    fn title_menu_moves_opens_and_starts() {
        let f = fe();
        let mut flow = Flow::new(&f);
        let ctx = Ctx::default();
        flow.set(&f, TITLE);
        // The generic step is off for the first two passes.
        flow.tick(&f, DOWN, 0, &ctx);
        assert_eq!(flow.focus, 0);
        flow.tick(&f, 0, 0, &ctx);
        flow.tick(&f, DOWN, 0, &ctx);
        assert_eq!(flow.focus, 1);
        flow.tick(&f, UP, 0, &ctx);
        flow.tick(&f, UP, 0, &ctx);
        assert_eq!(flow.focus, 2, "Up wraps from the first item to the last");
        flow.tick(&f, DOWN, 0, &ctx);
        flow.tick(&f, DOWN, 0, &ctx);
        // Cross on item 1 opens record 11 (pushed); Triangle comes back.
        flow.tick(&f, CROSS, 0, &ctx);
        assert_eq!(flow.current, 11);
        flow.tick(&f, 0, 0, &ctx);
        flow.tick(&f, 0, 0, &ctx);
        flow.tick(&f, TRIANGLE, 0, &ctx);
        assert_eq!((flow.current, flow.focus), (TITLE, 0), "back to the title, default focus");
        // New Game: focus 0, pass 5 or more, Cross.
        for _ in 0..5 {
            flow.tick(&f, 0, 0, &ctx);
        }
        assert_eq!(flow.tick(&f, CROSS, 0, &ctx), Some(Command::NewGame));
        assert_eq!(flow.current, PLAY);
    }

    #[test]
    fn select_pauses_and_resumes() {
        let mut f = fe();
        f.screens[PAUSE] = Screen { node: Some(6), back: -1, state: 7 };
        let mut flow = Flow::new(&f);
        let ctx = Ctx::default();
        flow.set(&f, PLAY);
        flow.tick(&f, SELECT, SELECT, &ctx);
        assert_eq!(flow.current, PAUSE);
        flow.tick(&f, SELECT, SELECT, &ctx);
        assert_eq!(flow.current, PLAY);
        // A game over takes play to the game-over page, and its end back to play.
        f.screens[GAME_OVER] = Screen { node: Some(6), back: -2, state: 26 };
        flow.tick(&f, 0, 0, &Ctx { game_over: true });
        assert_eq!(flow.current, GAME_OVER);
        flow.tick(&f, 0, 0, &Ctx { game_over: false });
        assert_eq!(flow.current, PLAY);
    }

    #[test]
    fn load_and_save_pickers() {
        let mut f = fe();
        f.screens[LOAD] = Screen { node: Some(6), back: -1, state: 21 };
        f.screens[SAVE] = Screen { node: Some(6), back: -2, state: 22 };
        let ctx = Ctx::default();
        let mut flow = Flow::new(&f);
        flow.rows.pulse_max = 1.8;
        flow.set(&f, TITLE);
        // The load screen with no save data leaves at once.
        flow.enter(&f, LOAD, &ctx);
        assert_eq!(flow.tick(&f, 0, 0, &ctx), Some(Command::Scan));
        flow.scanned(None);
        flow.tick(&f, 0, 0, &ctx);
        assert_eq!(flow.current, TITLE);
        // With slots 1 and 3 filled the cursor starts on 1 and skips 2.
        let mut file = SaveFile::empty();
        file.slots[1].set_key(0, 11);
        file.slots[3].set_key(0, 602);
        flow.enter(&f, LOAD, &ctx);
        flow.tick(&f, 0, 0, &ctx);
        flow.scanned(Some(file.clone()));
        flow.tick(&f, 0, 0, &ctx);
        assert_eq!((flow.picker.stage, flow.picker.cursor), (2, 1));
        flow.tick(&f, DOWN, 0, &ctx);
        assert_eq!(flow.picker.cursor, 3);
        assert_eq!(flow.tick(&f, CROSS, CROSS, &ctx), Some(Command::Load(3)));
        assert_eq!(flow.current, PLAY);
        // The save screen: opened at slot 0, moves by one, Cross saves, a good write returns.
        flow.open_save(&f);
        assert_eq!(flow.tick(&f, 0, 0, &ctx), Some(Command::Scan));
        flow.scanned(Some(file));
        flow.tick(&f, 0, 0, &ctx);
        flow.tick(&f, DOWN, 0, &ctx);
        assert_eq!(flow.picker.cursor, 1);
        assert_eq!(flow.tick(&f, CROSS, CROSS, &ctx), Some(Command::Save(1)));
        flow.written(&f, false);
        assert_eq!((flow.current, flow.picker.stage), (SAVE, 0), "a failed write scans again");
        flow.tick(&f, 0, 0, &ctx);
        flow.tick(&f, 0, 0, &ctx);
        flow.tick(&f, CROSS, CROSS, &ctx);
        flow.written(&f, true);
        assert_eq!(flow.current, PLAY);
        // Triangle in the picker returns to play without writing.
        flow.open_save(&f);
        flow.tick(&f, 0, 0, &ctx);
        flow.scanned(None);
        flow.tick(&f, 0, 0, &ctx);
        assert_eq!(flow.tick(&f, TRIANGLE, TRIANGLE, &ctx), None);
        assert_eq!(flow.current, PLAY);
    }

    #[test]
    fn list_items_are_spaced_by_visible_rows() {
        let f = fe();
        let draws = draw_list(&f, TITLE, 0, 0);
        assert!(draws.is_empty(), "items have no sprites here");
        let mut g = fe();
        for i in [3, 4, 5] {
            g.nodes[i].child = None;
        }
        // Give each item a sprite child.
        for (k, item) in [3usize, 4, 5].iter().enumerate() {
            let mut s = Node { kind: 1, parent: Some(*item), scale_x: 4096, scale_y: 4096, width: 10, height: 10, handler: -1, data: 1000 + k as u32, ..Node::default() };
            s.key = 0xFFFF;
            g.nodes.push(s);
            let at = g.nodes.len() - 1;
            g.nodes[*item].child = Some(at);
            g.sprites.insert(1000 + k as u32, neopets_data::frontend::Sprite { colour: [128; 4], ..Default::default() });
        }
        let ys: Vec<f32> = draw_list(&g, TITLE, 0, 0)
            .iter()
            .filter_map(|d| match d {
                Draw::Sprite { y, .. } => Some(*y),
                _ => None,
            })
            .collect();
        assert_eq!(ys, vec![0.0, 31.0, 62.0]);
    }
}
