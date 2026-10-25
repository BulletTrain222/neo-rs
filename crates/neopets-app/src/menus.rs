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
//! the world is not torn down), options, credits, the item screens, journal and maps (their
//! pages open and Triangle leaves them), sounds (the menu moves, the save-complete cue), key masks
//! and animation slots, spatial pages, and the predicates other than those listed in
//! `predicate`. The card boxes cover the clean side's jobs only: no card, format or card-gone
//! records.

use neopets_data::frontend::Frontend;
use neopets_sim::save::{self, SaveFile};
use neopets_sim::textbox::{self, TextBox};

pub const CARD: usize = 0;
pub const ATTRACT: usize = 1;
pub const PAUSE: usize = 2;
pub const PLAY: usize = 9;
pub const TITLE: usize = 10;
pub const GAME_OVER: usize = 23;
pub const LOGO: usize = 30;
/// The load prompt (record 17, state 21) and the save prompt (record 18, state 22).
pub const LOAD: usize = 17;
/// The credits roll (record 14, state 18) and the Options page (record 11).
pub const CREDITS: usize = 14;
pub const SAVE: usize = 18;
/// The item screen's tabs for Tor: records 6, 7 and 8 (states 8, 9 and 10; codes 0x78, 0x8C,
/// 0xA0), whose page pictures show Tor; Roberta's are records 3 to 5 (codes 0x3C, 0x50, 0x64).
/// The spec gives the six codes by tab and hero without the order (R-0031).
pub const ITEMS: usize = 6;

/// Menu bits of the rising-edge word (SPEC-0035 section 4).
pub const UP: u16 = 0x1000;
pub const DOWN: u16 = 0x4000;
pub const CROSS: u16 = 0x0040;
pub const TRIANGLE: u16 = 0x0010;
pub const CIRCLE: u16 = 0x0020;
pub const RIGHT: u16 = 0x2000;
pub const LEFT: u16 = 0x8000;
pub const SQUARE: u16 = 0x0080;
pub const SELECT: u16 = 0x0100;
pub const START: u16 = 0x0800;
pub const L1: u16 = 0x0004;
pub const R1: u16 = 0x0008;

/// What the game around the front end must do.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Command {
    NewGame,
    Quit,
    /// Read the save file and report it with `Flow::scanned` (the load or the save scan).
    Scan,
    /// Rebuild slot k of the scanned file from the live game and set the session snapshot.
    Rebuild(usize),
    /// Write the scanned file (with its rebuilt slot) and report with `Flow::written`.
    Write,
    /// Apply slot k of the scanned file.
    Load(usize),
    /// Item screen (SPEC-0035 section 14.2): use the item, set the quick-item selection marker
    /// (-1 clears it), equip a Mote into equipment word k, or swap the Tattered Cloak.
    UseItem(u16),
    Marker(i32),
    EquipMote { word: usize, id: u16 },
    Cloak,
}

/// The slot picker of the load and save screens (SPEC-0035 section 7): the stage, the cursor,
/// the scanned file, the selected row's pulse and its direction.
#[derive(Clone, Debug, PartialEq)]
pub struct Picker {
    pub stage: u8,
    pub cursor: usize,
    pub file: SaveFile,
    /// The scan found no readable file (the load screen shows record 14).
    failed: bool,
    /// The rows are up (from the end of a scan until the next scan).
    pub rows: bool,
    /// Passes counted by a waiting stage.
    wait: u32,
    pub pulse: f32,
    step: f32,
}

impl Default for Picker {
    fn default() -> Self {
        Self { stage: 0, cursor: 0, file: SaveFile::empty(), failed: false, rows: false, wait: 0, pulse: 0.0, step: 0.0 }
    }
}

/// A card message in the shared box (SPEC-0025 Card message records): a table-9 record, its
/// labels and colour; a question waits for Circle (1) or Triangle (0) and returns two passes
/// after the answer; a busy record has no labels and takes no input.
#[derive(Clone, Debug, PartialEq)]
pub struct CardBox {
    pub record: usize,
    pub text: TextBox,
    pub labels: [Vec<u8>; 2],
    pub colour: [u8; 3],
    pub busy: bool,
    passes: u32,
    answer: Option<i32>,
}

/// Busy box: 3 passes, then trunc(2.0 x 60) = 120 more before the card operation runs (the
/// seconds word at 0x2DD410 is read at start-up into `Flow::busy_hold`).
const BUSY_FIRST: u32 = 3;

// Load screen stages (state 21).
const L_BUSY: u8 = 0;
const L_READ: u8 = 1;
const L_PICK: u8 = 2;
const L_FAILED: u8 = 3;
const L_CANCEL: u8 = 4;
const L_AFTER: u8 = 5;
// Save screen stages (state 22).
const S_SCAN: u8 = 0;
const S_SCANNED: u8 = 1;
const S_PICK: u8 = 2;
const S_ASK: u8 = 3;
const S_BUSY: u8 = 4;
const S_WRITE: u8 = 5;
const S_DONE: u8 = 6;
const S_FAILED: u8 = 7;
const S_CANCEL: u8 = 8;

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
    /// The card message on screen, the table-9 records, the substitution slots (0: the card
    /// slot "1"; 1: the needed KB) and the busy hold in passes.
    pub card: Option<CardBox>,
    pub messages: Vec<neopets_data::design::Message>,
    pub slots: Vec<Vec<u8>>,
    pub busy_hold: u32,
    /// Front-end sounds this pass, as main-block cue ids (SPEC-0031 cue use table; SPEC-0035
    /// menu sounds: cursor move 0x9A, select 0x98, back 0x99, save complete 0xA9).
    pub sounds: Vec<i32>,
    /// The screen current before the last screen change.
    came_from: Option<usize>,
    /// The live options (SPEC-0035 Behavior 3) and the held menu word of this pass (sliders act
    /// on held Left and Right).
    pub options: neopets_sim::options::Options,
    pub held: u16,
    /// The title has been shown (the music controller's front-end pass gate).
    pub title_seen: bool,
    /// The music bank override the front end holds (SPEC-0035 Behavior 3a): the title sets 25,
    /// the credits 8 (it stays on the Options page); the pause page and play clear it.
    pub bank_override: Option<i32>,
    /// The credits (SPEC-0026 Credits): the rows (text, style, colour set), the colour table
    /// (two sets of 8 words), the roll's words, the roll and the rows shown this pass.
    pub credit_rows: Vec<(Option<Vec<u8>>, u16, u16)>,
    pub credit_colours: Vec<[u8; 4]>,
    pub credit_words: Option<neopets_sim::credits::Words>,
    pub roll: Option<neopets_sim::credits::Roll>,
    pub credits_shown: Vec<neopets_sim::credits::Shown>,
    /// The item screen (SPEC-0035 section 14.1): the tab (kept for the session), the Items-tab
    /// cursor (kept between visits), the Charms-tab and Quest-tab cursors, the grid ids (boot
    /// 0x2DCF28) and the last item view the host gave.
    pub item_tab: usize,
    pub item_cursor: usize,
    pub charm_cursor: usize,
    pub quest_cursor: usize,
    pub grid_ids: [i32; 20],
    pub view: ItemView,
}

/// What the predicates read: the pass count, the option words of slots 66 to 68 (Vibration,
/// Invert Left-Right, Invert Up-Down), the knob positions the last apply set, and the item
/// screen's view.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Pc {
    pub pass: u32,
    pub words: [u32; 3],
    pub knobs: Option<[i32; 3]>,
    pub items: ItemPc,
}

/// The item screen's predicate inputs: the tab, the cursor, the Items-tab grid ids (0 for a cell
/// with nothing held), the selected entry's id and usability, the game's item view.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ItemPc {
    pub tab: usize,
    pub cursor: usize,
    pub grid: [i32; 20],
    pub selected: i32,
    pub view: ItemView,
}

/// The game's side of the item screen, filled by the host each pass: the counts of the grid ids,
/// the owned charms (category 8) and quest items (category 7) in list order (-1 past the end),
/// the quick-item selection marker, the active hero, its Mote words (-1 empty), which of its
/// weapon, body and shield entries is present, its cloak entry, and the counts of Motes 160 to
/// 173.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ItemView {
    pub grid_counts: [u32; 20],
    pub charms: [i32; 14],
    pub quests: [i32; 60],
    pub marker: i32,
    pub hero: usize,
    pub motes: [i32; 3],
    pub equipped: [bool; 3],
    pub cloak: bool,
    pub mote_counts: [u32; 14],
}

impl Default for ItemView {
    fn default() -> Self {
        Self { grid_counts: [0; 20], charms: [-1; 14], quests: [-1; 60], marker: -1, hero: 0, motes: [-1; 3], equipped: [false; 3], cloak: false, mote_counts: [0; 14] }
    }
}

impl Flow {
    /// Front-end start-up: a direct set of record 0 (the card state).
    pub fn new(fe: &Frontend) -> Self {
        let mut f = Self { current: CARD, stack: vec![CARD], pass: 0, focus: 0, attract_visits: 0, idle: 0, picker: Picker::default(), rows: RowWords::default(), card: None, messages: Vec::new(), slots: Vec::new(), busy_hold: 120, sounds: Vec::new(), came_from: None, options: Default::default(), held: 0, title_seen: false, bank_override: None, credit_rows: Vec::new(), credit_colours: Vec::new(), credit_words: None, roll: None, credits_shown: Vec::new(), item_tab: 0, item_cursor: 0, charm_cursor: 0, quest_cursor: 0, grid_ids: [-1; 20], view: ItemView::default() };
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

    /// The predicates' inputs of this pass.
    pub fn pc(&self) -> Pc {
        let b = self.options.block;
        Pc { pass: self.pass, words: [b.vibration, b.invert_lr, b.invert_ud], knobs: self.options.knobs, items: self.item_pc() }
    }

    /// The item screen's predicate inputs for this pass.
    fn item_pc(&self) -> ItemPc {
        let v = &self.view;
        let grid = std::array::from_fn(|k| if v.grid_counts[k] > 0 { self.grid_ids[k] } else { 0 });
        let tab = self.current.saturating_sub(ITEMS).min(2);
        let (cursor, selected) = match tab {
            0 => (self.item_cursor, if v.grid_counts[self.item_cursor] > 0 { self.grid_ids[self.item_cursor] } else { -1 }),
            1 => (self.charm_cursor, v.charms[self.charm_cursor.min(13)]),
            _ => (self.quest_cursor, v.quests[self.quest_cursor.min(59)]),
        };
        ItemPc { tab, cursor, grid, selected, view: *v }
    }

    /// One pass of the item screen's tab states 8, 9 and 10 (SPEC-0035 sections 14.1 and 14.2).
    fn item_screen(&mut self, tab: usize, edge: u16, second: u16, command: &mut Option<Command>) -> Option<usize> {
        let buttons = edge | second;
        let v = self.view;
        if buttons & (TRIANGLE | START) != 0 {
            self.sounds.push(0x95);
            return Some(PLAY);
        }
        if buttons & L1 != 0 {
            self.sounds.push(0x98);
            self.item_tab = (tab + 2) % 3;
            return Some(ITEMS + self.item_tab);
        }
        if buttons & R1 != 0 {
            self.sounds.push(0x99);
            self.item_tab = (tab + 1) % 3;
            return Some(ITEMS + self.item_tab);
        }
        let (up, down, left, right) = (edge & UP != 0, edge & DOWN != 0, edge & LEFT != 0, edge & RIGHT != 0);
        match tab {
            0 => {
                self.item_cursor = neopets_sim::inventory::grid_move(self.item_cursor, up, down, left, right);
                let k = self.item_cursor;
                let id = self.grid_ids[k];
                let held = v.grid_counts[k] > 0 && id >= 0;
                if buttons & CROSS != 0 {
                    if held {
                        *command = Some(Command::UseItem(id as u16));
                    } else {
                        self.sounds.push(0xA2);
                    }
                } else if buttons & CIRCLE != 0 && held {
                    self.sounds.push(0x9A);
                    *command = Some(Command::Marker(if v.marker == id { -1 } else { id }));
                } else if buttons & SQUARE != 0 && v.cloak {
                    *command = Some(Command::Cloak);
                }
            }
            1 => {
                // The Mote picker over the owned charms: Left and Right step through the list
                // (with at least two entries); Cross equips word 0, Square word 1 (with a shield)
                // and Circle word 2 (with a weapon).
                let n = v.charms.iter().take_while(|&&c| c >= 0).count();
                if n >= 2 && (left || up) {
                    self.charm_cursor = (self.charm_cursor + n - 1) % n;
                } else if n >= 2 && (right || down) {
                    self.charm_cursor = (self.charm_cursor + 1) % n;
                }
                self.charm_cursor = self.charm_cursor.min(n.saturating_sub(1));
                let id = v.charms[self.charm_cursor];
                let word = if buttons & CROSS != 0 {
                    Some(0)
                } else if buttons & SQUARE != 0 && v.equipped[2] {
                    Some(1)
                } else if buttons & CIRCLE != 0 && v.equipped[0] {
                    Some(2)
                } else {
                    None
                };
                if let (Some(word), true) = (word, id >= 0) {
                    *command = Some(Command::EquipMote { word, id: id as u16 });
                }
            }
            _ => {
                let n = v.quests.iter().take_while(|&&c| c >= 0).count();
                if n >= 2 {
                    self.quest_cursor = neopets_sim::inventory::grid_move(self.quest_cursor, up, down, left, right).min(n - 1);
                }
                if buttons & (CROSS | CIRCLE) != 0 {
                    self.sounds.push(0xA2);
                }
            }
        }
        None
    }

    /// Whether play was entered from the pause page on this pass (the unpause cue).
    pub fn stack_was_pause(&self) -> bool {
        self.came_from == Some(PAUSE)
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

    /// The write job's result: written (the box clears and two passes later play resumes), or
    /// not (record 6, Save failed: Retry or Cancel save).
    pub fn written(&mut self, ok: bool) {
        if self.current != SAVE || self.picker.stage != S_WRITE {
            return;
        }
        if ok {
            self.sounds.push(0xA9);
            self.card = None;
            self.picker.stage = S_DONE;
            self.picker.wait = 0;
        } else {
            self.show(6, false);
            self.picker.stage = S_FAILED;
        }
    }

    /// Open a card message: a busy record, or a question.
    fn show(&mut self, record: usize, busy: bool) {
        let m = self.messages.get(record).cloned().unwrap_or_default();
        let mut text = TextBox::default();
        let body = textbox::substitute(&m.text, &self.slots);
        if busy {
            text.open(&body);
        } else {
            text.open_choice(&body);
        }
        let [a, b] = m.labels;
        self.card = Some(CardBox { record, text, labels: [a.unwrap_or_default(), b.unwrap_or_default()], colour: m.colour, busy, passes: 0, answer: None });
    }

    /// One pass of the card box: the reveal and paging, then the answer (returned two passes
    /// after it was given); a busy box counts its passes.
    fn card_pass(&mut self, edge: u16) -> Option<i32> {
        let c = self.card.as_mut()?;
        if c.busy {
            c.text.update(false, false);
            c.passes += 1;
            return None;
        }
        if let Some(a) = c.answer {
            c.passes += 1;
            if c.passes >= 2 {
                self.card = None;
                return Some(a);
            }
            return None;
        }
        c.text.update(false, edge & SQUARE != 0);
        if let Some(a) = c.text.choose(edge & CIRCLE != 0, edge & TRIANGLE != 0) {
            c.answer = Some(a);
            self.sounds.push(if a == 1 { 0x98 } else { 0x99 });
            c.passes = 0;
        }
        None
    }

    /// The busy box has been up for its minimum (3 + the hold passes).
    fn busy_done(&self) -> bool {
        self.card.as_ref().is_some_and(|c| c.busy && c.passes >= BUSY_FIRST + self.busy_hold)
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
            self.sounds.push(0x9A);
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
        self.came_from = Some(self.current);
        if matches!(r, PLAY | PAUSE) {
            self.bank_override = None;
        }
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
                self.bank_override = Some(25);
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
                } else if (edge | second) & START != 0 {
                    // SPEC-0035: Start opens the item screen on its current tab (Tor's records).
                    result = Some(ITEMS + self.item_tab);
                }
                // Play's own buttons belong to the hero; the HUD page has no menu items.
                generic = false;
            }
            7 if second & SELECT != 0 => result = Some(PLAY),
            8..=10 => {
                if self.pass == 0 && self.current == ITEMS && self.view.grid_counts[self.item_cursor] == 0 {
                    self.item_cursor = 0;
                }
                if self.pass == 0 && self.current == ITEMS + 2 {
                    self.quest_cursor = 0;
                }
                result = self.item_screen(state as usize - 8, edge, second, &mut command);
                generic = false;
            }
            26 => {
                if !ctx.game_over {
                    result = Some(PLAY);
                }
                generic = false;
            }
            // SPEC-0026 Credits (state 18): the roll from pass 0; the bank override 8 each pass;
            // the end returns "back" (no cue) to the Options page. Triangle leaves early through
            // the generic back rule from pass 2.
            18 => {
                self.bank_override = Some(8);
                if self.pass == 0 {
                    self.roll = self.credit_words.map(|w| neopets_sim::credits::Roll::new(448, &w));
                }
                let styles: Vec<(bool, u16)> = self.credit_rows.iter().map(|r| (r.0.is_some(), r.1)).collect();
                match (self.roll.as_mut(), self.credit_words) {
                    (Some(roll), Some(w)) => {
                        self.credits_shown = roll.step(&styles, 448, &w);
                        if roll.ended(styles.len()) {
                            self.credits_shown.clear();
                            self.roll = None;
                            self.back(fe, ctx);
                            return command;
                        }
                    }
                    _ => self.credits_shown.clear(),
                }
            }
            // SPEC-0035 section 7.2 and SPEC-0025 Load scan, mapped onto the clean side's file:
            // the busy record 15 for its minimum, the read, then the picker (Cross loads the
            // chosen slot); a missing or bad file shows record 14 (Retry, or Cancel load and
            // record 17); no filled slot leaves at once. Only the picker lets Triangle back out.
            21 => {
                if self.pass == 0 {
                    self.picker.stage = L_BUSY;
                    self.card = None;
                }
                generic = false;
                let answer = self.card_pass(edge);
                match self.picker.stage {
                    L_BUSY => {
                        self.picker.rows = false;
                        self.show(15, true);
                        self.picker.stage = L_READ;
                    }
                    L_READ if self.busy_done() => {
                        command = Some(Command::Scan);
                        self.card = None;
                        self.picker.stage = L_AFTER;
                        self.picker.wait = 0;
                    }
                    L_AFTER => {
                        self.picker.wait += 1;
                        if self.picker.wait >= 2 {
                            if self.picker.failed {
                                self.show(14, false);
                                self.picker.stage = L_FAILED;
                            } else if !self.picker.file.filled().contains(&true) {
                                self.back(fe, ctx);
                                return command;
                            } else {
                                self.picker.stage = L_PICK;
                                self.picker.rows = true;
                                self.picker.pulse = self.rows.pulse_max;
                            }
                        }
                    }
                    L_PICK => {
                        generic = true;
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
                    L_FAILED => match answer {
                        Some(1) => self.picker.stage = L_BUSY,
                        Some(_) => {
                            self.show(17, false);
                            self.picker.stage = L_CANCEL;
                        }
                        None => {}
                    },
                    L_CANCEL => match answer {
                        Some(1) => {
                            self.back(fe, ctx);
                            return command;
                        }
                        Some(_) => self.picker.stage = L_BUSY,
                        None => {}
                    },
                    _ => {}
                }
            }
            // Section 7.3 and SPEC-0025 Save write, mapped onto the clean side's file: scan,
            // picker; Cross rebuilds the slot and asks record 7 (overwrite) or 10 (save to the
            // card); Yes holds the busy record 5, then writes the whole file; No scans again. A
            // failed write shows record 6 (Retry scans again; Cancel save asks record 9, whose
            // Yes returns to play without writing: the original's save-complete cue on that path
            // is a quirk this build drops). Triangle in the picker returns to play.
            22 => {
                generic = false;
                let answer = self.card_pass(edge);
                match self.picker.stage {
                    S_SCAN => {
                        self.picker.rows = false;
                        command = Some(Command::Scan);
                        self.picker.stage = S_SCANNED;
                    }
                    S_SCANNED => {
                        self.picker.stage = S_PICK;
                        self.picker.rows = true;
                        self.picker.pulse = self.rows.pulse_max;
                    }
                    S_PICK => {
                        self.pulse_tick();
                        self.picker_moves(edge, false);
                        if edge & CROSS != 0 {
                            let k = self.picker.cursor.min(save::SLOTS - 1);
                            let present = self.picker.file.slots[k].present();
                            command = Some(Command::Rebuild(k));
                            self.show(if present { 7 } else { 10 }, false);
                            self.picker.stage = S_ASK;
                        } else if edge & TRIANGLE != 0 {
                            result = Some(PLAY);
                        }
                    }
                    S_ASK => match answer {
                        Some(1) => {
                            self.show(5, true);
                            self.picker.stage = S_BUSY;
                        }
                        Some(_) => self.picker.stage = S_SCAN,
                        None => self.pulse_tick(),
                    },
                    S_BUSY if self.busy_done() => {
                        command = Some(Command::Write);
                        self.picker.stage = S_WRITE;
                    }
                    S_DONE => {
                        self.picker.wait += 1;
                        if self.picker.wait >= 2 {
                            result = Some(PLAY);
                        }
                    }
                    S_FAILED => match answer {
                        Some(1) => self.picker.stage = S_SCAN,
                        Some(_) => {
                            self.show(9, false);
                            self.picker.stage = S_CANCEL;
                        }
                        None => {}
                    },
                    S_CANCEL => match answer {
                        Some(1) => result = Some(PLAY),
                        Some(_) => self.picker.stage = S_SCAN,
                        None => {}
                    },
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
            // SPEC-0035 Behavior 3: the option rows. A toggle changes once per Left or Right
            // press; an invert row and a slider run the apply rule on every pass; sliders act on
            // held Left and Right. Option changes are silent.
            use neopets_sim::options as opt;
            let press = edge & (LEFT | RIGHT) != 0;
            let (hl, hr) = (self.held & LEFT != 0, self.held & RIGHT != 0);
            let b = &mut self.options.block;
            match focus_state {
                13 if press => b.vibration = opt::toggle(b.vibration),
                14 => {
                    if press {
                        if self.focus == 1 {
                            b.invert_lr = opt::toggle(b.invert_lr);
                        } else {
                            b.invert_ud = opt::toggle(b.invert_ud);
                        }
                    }
                    self.options.apply();
                }
                15..=17 => {
                    let v = match focus_state {
                        15 => &mut b.music,
                        16 => &mut b.effects,
                        _ => &mut b.voice,
                    };
                    *v = opt::slide(*v, hl, hr);
                    self.options.apply();
                }
                _ => {}
            }
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
                    if t.target != -3 {
                        self.sounds.push(0x98);
                    }
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
                    self.sounds.push(0x99);
                    self.back(fe, ctx);
                    return command;
                }
            } else if page_of(fe, self.current).is_some_and(|p| p.list) && edge & (DOWN | UP) != 0 {
                let passing: Vec<usize> = (0..list.len()).filter(|&k| item_ok(fe, list[k], self.current, ctx, self)).collect();
                if let Some(at) = passing.iter().position(|&k| k == self.focus) {
                    self.sounds.push(0x9A);
                    let n = passing.len();
                    self.focus = passing[if edge & DOWN != 0 { (at + 1) % n } else { (at + n - 1) % n }];
                }
            }
        }
        match result {
            Some(r) => {
                self.card = None;
                self.enter(fe, r, ctx);
            }
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
/// 12, 27, 39 and 40 always 0; 66 to 68 the option words (Vibration, Invert Left-Right, Invert
/// Up-Down); 72 the game-over page's pass count above twice the tick rate. Other slots read 0.
fn predicate(slot: i16, arg: u8, pc: Pc) -> i32 {
    let it = &pc.items;
    let v = &it.view;
    let a = arg as usize;
    let count_of = |id: i32| -> u32 { if (160..=173).contains(&id) { v.mote_counts[(id - 160) as usize] } else { 0 } };
    match slot {
        // SPEC-0035 section 9c, the item screen's slots.
        1 => (it.tab == 0 && a < 20 && it.grid[a] != 0 && it.grid[a] == v.marker) as i32,
        3 => (a == it.cursor) as i32,
        4 => if it.tab == 0 && a < 20 { it.grid[a] } else { 0 },
        5 | 7 => (it.selected >= 0 && it.tab != 2) as i32,
        6 => (it.selected >= 0 && it.selected == v.marker) as i32,
        8 => v.cloak as i32,
        9 => if a == 1 { (!v.cloak) as i32 } else { 1 },
        13 => (a < 3 && v.equipped[a]) as i32,
        14 => (v.hero == 0) as i32,
        52 => (v.hero == 1) as i32,
        15 => (it.tab == 1 && v.charms.get(it.cursor).copied().unwrap_or(-1) == 160 + a as i32 % 100) as i32,
        16 => if a < 14 { v.mote_counts[a] as i32 } else { 0 },
        17 => (count_of(v.motes[2]) > 0) as i32,
        19 => (count_of(v.motes[0]) > 0) as i32,
        20 => (count_of(v.motes[1]) > 0) as i32,
        0 | 2 | 56 => 1,
        66 => pc.words[0] as i32,
        67 => pc.words[1] as i32,
        68 => pc.words[2] as i32,
        72 => (pc.pass > 120) as i32,
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
fn own_predicate(fe: &Frontend, n: usize, pc: Pc) -> i32 {
    let node = fe.nodes[n];
    if node.handler >= 0 { predicate(node.handler, node.arg, pc) } else { 1 }
}

/// A selectable item passes when the mode tests from its parent up to the record's node pass.
fn item_ok(fe: &Frontend, item: usize, record: usize, _ctx: &Ctx, flow: &Flow) -> bool {
    let top = fe.screens.get(record).and_then(|s| s.node);
    let mut at = fe.nodes[item].parent;
    while let Some(i) = at {
        let n = fe.nodes[i];
        let p = if n.handler >= 0 { predicate(n.handler, n.arg, flow.pc()) } else { 1 };
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
pub fn draw_list(fe: &Frontend, record: usize, focus: usize, pc: Pc) -> Vec<Draw> {
    let mut out = Vec::new();
    let Some(root) = fe.screens.get(record).and_then(|s| s.node) else { return out };
    let mut counter = 0usize;
    draw_node(fe, root, (0.0, 0.0, 1.0, 1.0), false, focus, pc, &mut counter, &mut out);
    out
}

/// The credits rows of this pass: centred at x 256 in font 0 (the spec names no font; R-0023),
/// in the colour of the row's style and set, with the roll's alpha.
pub fn credits_draws(fe: &Frontend, flow: &Flow) -> Vec<Draw> {
    if flow.current != CREDITS {
        return Vec::new();
    }
    let font = 0usize;
    flow.credits_shown
        .iter()
        .filter_map(|s| {
            let (text, style, set) = flow.credit_rows.get(s.row)?;
            let text = text.as_ref()?;
            let c = flow.credit_colours.get(*set as usize * 8 + *style as usize).copied().unwrap_or([128; 4]);
            let w = fe.fonts.get(font).map_or(0, |f| f.measure(text, 0));
            Some(Draw::Text { font, text: text.clone(), spacing: 0, x: 256.0 - (w / 2) as f32, y: s.y, colour: [c[0], c[1], c[2], s.alpha as u8] })
        })
        .collect()
}

/// The item screen's two text lines (SPEC-0035 section 14.1): the selected entry's name line
/// and its description, at the two value-text nodes of the record's group at (63, 388), in each
/// node's font and colour. The lines' texts come from the host (`lines`).
pub fn item_draws(fe: &Frontend, flow: &Flow, lines: &[Vec<u8>; 2]) -> Vec<Draw> {
    if !(ITEMS..ITEMS + 3).contains(&flow.current) {
        return Vec::new();
    }
    // The group with key 0xC2 (cells, frames, counts and the action box) is a library group of
    // record 3's tree under the slot-12 gate; the tab states of either hero draw it themselves
    // (section 9c). The two text lines are the value-text nodes beside it.
    let Some(g) = fe.key(0xC2) else { return Vec::new() };
    let mut out = Vec::new();
    // The Items tab draws the 0xC2 group; the Charms tab draws the Mote panel (key 0xC0) on its
    // own; the Quest-items tab draws neither here.
    let tab = flow.current - ITEMS;
    let group = match tab {
        0 => Some(g),
        1 => fe.key(0xC0),
        _ => None,
    };
    if let Some(n) = group {
        let (x, y) = node_xy(fe, n);
        let mut counter = 0;
        draw_node(fe, n, (x, y, 1.0, 1.0), false, flow.focus, flow.pc(), &mut counter, &mut out);
    }
    // SPEC-0035 section 14.1: a cell with an owned count of 0 is hidden. The icon layer's
    // predicate (slot 2) is always 1, so the rule is applied here (R-0031).
    let pc = flow.pc();
    out.retain(|d| match d {
        Draw::Sprite { node, .. } => {
            let n = fe.nodes[*node];
            !(n.handler == 2 && pc.items.tab == 0 && (n.arg as usize) < 20 && pc.items.grid[n.arg as usize] == 0)
        }
        _ => true,
    });
    let mut nodes = Vec::new();
    fn walk(fe: &Frontend, n: usize, out: &mut Vec<usize>) {
        if fe.nodes[n].kind == 6 {
            out.push(n);
        }
        for c in fe.children(n) {
            walk(fe, c, out);
        }
    }
    if let Some(parent) = fe.nodes[g].parent {
        for c in fe.children(parent).into_iter().filter(|&c| c != g) {
            walk(fe, c, &mut nodes);
        }
    }
    // The name and description lines are the last two (the pair at (63, 388)).
    let pair = &nodes[nodes.len().saturating_sub(2)..];
    out.extend(pair.iter().zip(lines.iter()).filter_map(|(&n, text)| {
        let v = fe.values.get(&fe.nodes[n].data)?;
        let (x, y) = node_xy(fe, n);
        Some(Draw::Text { font: v.font as usize, text: text.clone(), spacing: v.spacing as i32, x, y, colour: v.colour })
    }));
    out
}

/// A node's canvas position: its offset plus its parents' up to an absolute node or the root
/// (the item screen's nodes have scale 1).
fn node_xy(fe: &Frontend, n: usize) -> (f32, f32) {
    let (mut x, mut y) = (0.0, 0.0);
    let mut at = Some(n);
    while let Some(i) = at {
        let d = fe.nodes[i];
        x += d.x as f32;
        y += d.y as f32;
        if d.absolute {
            break;
        }
        at = d.parent;
    }
    (x, y)
}

/// The slot picker's rows (sections 7.1 and 14.3): four rows by the journal row drawer, each at
/// offset p below the first; the cursor's row has its bar pulsed. Shown from the picker stage.
pub fn picker_draws(fe: &Frontend, flow: &Flow) -> Vec<Draw> {
    let mut out = Vec::new();
    if !matches!(flow.current, LOAD | SAVE) || !flow.picker.rows {
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

fn nearest_predicate(fe: &Frontend, n: usize, pc: Pc) -> i32 {
    let mut at = Some(n);
    while let Some(i) = at {
        if fe.nodes[i].handler >= 0 {
            return own_predicate(fe, i, pc);
        }
        at = fe.nodes[i].parent;
    }
    1
}

#[allow(clippy::too_many_arguments)]
fn draw_node(fe: &Frontend, n: usize, t: (f32, f32, f32, f32), focused: bool, focus: usize, pc: Pc, counter: &mut usize, out: &mut Vec<Draw>) {
    let node = fe.nodes[n];
    if node.kind != 4 && !mode_passes(fe, n, nearest_predicate(fe, n, pc), focused) {
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
        let passes = cn.kind == 4 || mode_passes(fe, c, nearest_predicate(fe, c, pc), child_focused);
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
        // A slider knob (elements 0xCE to 0xD0) sits at the x the last apply wrote.
        let x0 = match (cn.key, pc.knobs) {
            (0xCE..=0xD0, Some(k)) => k[(cn.key - 0xCE) as usize] as f32,
            _ => cn.x as f32,
        };
        let cx = bx + x0 * bsx - shift;
        let cy = by + (cn.y as f32 + node.spacing as f32 * rows as f32) * bsy;
        draw_node(fe, c, (cx, cy, bsx, bsy), child_focused, focus, pc, counter, out);
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

    /// Run passes until the flow emits a command (or a limit).
    fn until(flow: &mut Flow, f: &Frontend, ctx: &Ctx, limit: usize) -> (Option<Command>, usize) {
        for n in 1..=limit {
            if let Some(c) = flow.tick(f, 0, 0, ctx) {
                return (Some(c), n);
            }
        }
        (None, limit)
    }

    /// Reveal the open card box and answer it (Circle 1 or Triangle 0), then the two passes.
    fn answer(flow: &mut Flow, f: &Frontend, ctx: &Ctx, circle: bool) {
        for _ in 0..400 {
            if flow.card.as_ref().is_some_and(|c| c.text.asking()) {
                break;
            }
            flow.tick(f, SQUARE, 0, ctx);
        }
        let b = if circle { CIRCLE } else { TRIANGLE };
        flow.tick(f, b, b, ctx);
        flow.tick(f, 0, 0, ctx);
        flow.tick(f, 0, 0, ctx);
    }

    #[test]
    fn load_and_save_pickers() {
        let mut f = fe();
        f.screens[LOAD] = Screen { node: Some(6), back: -1, state: 21 };
        f.screens[SAVE] = Screen { node: Some(6), back: -2, state: 22 };
        let ctx = Ctx::default();
        let mut flow = Flow::new(&f);
        flow.rows.pulse_max = 1.8;
        flow.messages = vec![neopets_data::design::Message { text: b"Message".to_vec(), labels: [Some(b"Yes".to_vec()), Some(b"No".to_vec())], colour: [128; 3] }; 27];
        flow.set(&f, TITLE);
        // The load screen: busy record 15 for 123 passes, then the read.
        flow.enter(&f, LOAD, &ctx);
        let (c, n) = until(&mut flow, &f, &ctx, 400);
        assert_eq!((c, n), (Some(Command::Scan), 124), "the busy box holds 3 + 120 passes first");
        assert_eq!(flow.card.as_ref().map(|c| c.record), None);
        // A missing file: record 14; Cancel load, then Yes on record 17 leaves.
        flow.scanned(None);
        flow.tick(&f, 0, 0, &ctx);
        flow.tick(&f, 0, 0, &ctx);
        assert_eq!(flow.card.as_ref().map(|c| c.record), Some(14));
        answer(&mut flow, &f, &ctx, false);
        assert_eq!(flow.card.as_ref().map(|c| c.record), Some(17));
        answer(&mut flow, &f, &ctx, true);
        assert_eq!(flow.current, TITLE);
        // With slots 1 and 3 filled the cursor starts on 1 and skips 2; Cross loads.
        let mut file = SaveFile::empty();
        file.slots[1].set_key(0, 11);
        file.slots[3].set_key(0, 602);
        flow.enter(&f, LOAD, &ctx);
        until(&mut flow, &f, &ctx, 400);
        flow.scanned(Some(file.clone()));
        flow.tick(&f, 0, 0, &ctx);
        flow.tick(&f, 0, 0, &ctx);
        assert_eq!((flow.picker.stage, flow.picker.cursor, flow.picker.rows), (L_PICK, 1, true));
        flow.tick(&f, DOWN, 0, &ctx);
        assert_eq!(flow.picker.cursor, 3);
        assert_eq!(flow.tick(&f, CROSS, CROSS, &ctx), Some(Command::Load(3)));
        assert_eq!(flow.current, PLAY);
        // The save screen: opened at slot 0; Cross on filled slot 1 rebuilds it and asks
        // record 7; Yes holds busy record 5, then writes; a good write returns to play.
        flow.open_save(&f);
        assert_eq!(flow.tick(&f, 0, 0, &ctx), Some(Command::Scan));
        flow.scanned(Some(file.clone()));
        flow.tick(&f, 0, 0, &ctx);
        flow.tick(&f, DOWN, 0, &ctx);
        assert_eq!(flow.picker.cursor, 1);
        assert_eq!(flow.tick(&f, CROSS, CROSS, &ctx), Some(Command::Rebuild(1)));
        assert_eq!(flow.card.as_ref().map(|c| c.record), Some(7));
        answer(&mut flow, &f, &ctx, true);
        assert_eq!(flow.card.as_ref().map(|c| (c.record, c.busy)), Some((5, true)));
        let (c, _) = until(&mut flow, &f, &ctx, 400);
        assert_eq!(c, Some(Command::Write));
        flow.written(true);
        flow.tick(&f, 0, 0, &ctx);
        flow.tick(&f, 0, 0, &ctx);
        assert_eq!(flow.current, PLAY);
        // An empty slot asks record 10; No scans again with the cursor kept.
        flow.open_save(&f);
        flow.tick(&f, 0, 0, &ctx);
        flow.scanned(Some(file.clone()));
        flow.tick(&f, 0, 0, &ctx);
        flow.tick(&f, DOWN, 0, &ctx);
        flow.tick(&f, DOWN, 0, &ctx);
        assert_eq!(flow.tick(&f, CROSS, CROSS, &ctx), Some(Command::Rebuild(2)));
        assert_eq!(flow.card.as_ref().map(|c| c.record), Some(10));
        answer(&mut flow, &f, &ctx, false);
        assert_eq!(flow.tick(&f, 0, 0, &ctx), Some(Command::Scan));
        assert_eq!(flow.picker.cursor, 2);
        // A failed write: record 6; Cancel save, then Yes on record 9 returns to play.
        flow.scanned(Some(file));
        flow.tick(&f, 0, 0, &ctx);
        flow.tick(&f, CROSS, CROSS, &ctx);
        answer(&mut flow, &f, &ctx, true);
        until(&mut flow, &f, &ctx, 400);
        flow.written(false);
        assert_eq!(flow.card.as_ref().map(|c| c.record), Some(6));
        answer(&mut flow, &f, &ctx, false);
        assert_eq!(flow.card.as_ref().map(|c| c.record), Some(9));
        answer(&mut flow, &f, &ctx, true);
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
    fn option_rows() {
        // A page of three items with focus states 15 (Music), 14 (Invert) and 13 (Vibration).
        let mut f = fe();
        f.targets.insert(3, Target { target: -3, state: 15 });
        f.targets.insert(4, Target { target: -3, state: 14 });
        f.targets.insert(5, Target { target: -3, state: 13 });
        let ctx = Ctx::default();
        let mut flow = Flow::new(&f);
        flow.set(&f, TITLE);
        assert_eq!(flow.options.gains, [1.0; 3], "1.0 until the first apply");
        // The Music row applies on every pass, held or not: 0.75 from the defaults; held Right
        // adds 1.0 per pass.
        flow.tick(&f, 0, 0, &ctx);
        assert_eq!(flow.options.gains, [0.75; 3]);
        flow.held = RIGHT;
        for _ in 0..10 {
            flow.tick(&f, 0, 0, &ctx);
        }
        assert_eq!(flow.options.block.music, 85.0);
        flow.held = 0;
        // Focus 1 is the invert row: a Left press toggles Invert Left-Right.
        flow.focus = 1;
        flow.tick(&f, LEFT, 0, &ctx);
        assert_eq!((flow.options.block.invert_lr, flow.options.invert.0), (1, true));
        // The Vibration row toggles once per press.
        flow.focus = 2;
        flow.tick(&f, RIGHT, 0, &ctx);
        assert_eq!(flow.options.block.vibration, 0);
        assert_eq!(flow.pc().words, [0, 1, 0]);
    }

    #[test]
    fn list_items_are_spaced_by_visible_rows() {
        let f = fe();
        let draws = draw_list(&f, TITLE, 0, Pc::default());
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
        let ys: Vec<f32> = draw_list(&g, TITLE, 0, Pc::default())
            .iter()
            .filter_map(|d| match d {
                Draw::Sprite { y, .. } => Some(*y),
                _ => None,
            })
            .collect();
        assert_eq!(ys, vec![0.0, 31.0, 62.0]);
    }
}
