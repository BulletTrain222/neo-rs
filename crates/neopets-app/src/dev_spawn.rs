//! The developer menu's character spawner, a test tool of this build and not part of the game: a searchable list of
//! the characters on the disc (read from the disc when first opened; nothing is stored), and the chosen one placed in
//! front of the hero.

use bevy::input::mouse::{AccumulatedMouseScroll, MouseScrollUnit};
use bevy::prelude::*;
use neopets_data::archive::ArchiveIndex;
use neopets_data::Disc;
use std::sync::Mutex;
use std::sync::mpsc::{Receiver, TryRecvError};

use crate::dev_menu::DevMenu;

/// One selectable character: where its record lies on the disc, and how the list shows it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SpawnEntry {
    pub name: String,
    pub group: Group,
    pub region: u32,
    pub ordinal: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Group {
    Enemy,
    Npc,
    Other,
}

impl Group {
    /// The monster row's behaviour class: 17 the common enemies, 6 the townsfolk (SPEC-0029 Layout 3); the rest are
    /// the bosses and the special creatures. A record with no row is listed with the townsfolk.
    fn of(class: Option<i16>) -> Self {
        match class {
            Some(17) => Self::Enemy,
            Some(6) | None => Self::Npc,
            _ => Self::Other,
        }
    }

    fn word(self) -> &'static str {
        match self {
            Self::Enemy => "Enemy",
            Self::Npc => "NPC",
            Self::Other => "Boss/other",
        }
    }
}

/// The list's filters: everything, then each group.
const TABS: [(&str, Option<Group>); 4] = [("All", None), ("Enemies", Some(Group::Enemy)), ("NPCs", Some(Group::Npc)), ("Bosses/other", Some(Group::Other))];
const ROWS: usize = 16;

/// The characters of one resource, each new (name, type, variant) once. Records with no place that are not reserved
/// are left out, as the region loader leaves them out.
fn resource_entries(disc: &mut Disc, index: &ArchiveIndex, table: &neopets_data::monster::MonsterTable, resource: u32, seen: &mut std::collections::HashSet<(String, u16, u16)>) -> Vec<SpawnEntry> {
    let Ok(file) = index.read(disc, resource) else { return Vec::new() };
    let Ok(chain) = neopets_data::resource::walk_chunks(&file, None) else { return Vec::new() };
    let world = chain.chunks.first().is_some_and(|f| f.kind == 31);
    let records = if world {
        neopets_data::region::embedded_records(&file, &chain.chunks)
    } else if chain.chunks.iter().any(|c| c.kind == neopets_data::region::RECORDS) {
        Some(neopets_data::region::Region::of(&file, &chain.chunks).records)
    } else {
        None
    };
    let mut out = Vec::new();
    for pl in records.iter().flat_map(|r| &r.placements).filter(|p| p.summon || p.position != [0.0; 3]) {
        if !seen.insert((pl.name.clone(), pl.monster_type, pl.variant)) {
            continue;
        }
        let name = if pl.name.is_empty() { format!("Unnamed, type {} variant {}", pl.monster_type, pl.variant) } else { pl.name.clone() };
        out.push(SpawnEntry { name, group: Group::of(table.row(pl.monster_type, pl.variant).and_then(|r| r.class)), region: resource, ordinal: pl.ordinal });
    }
    out
}

/// Read the whole list from the disc, handing each resource's entries to `found`.
pub fn scan(disc: &mut Disc, index: &ArchiveIndex, mut found: impl FnMut(Vec<SpawnEntry>)) {
    let table = match (index.read(disc, 530), index.read(disc, 531)) {
        (Ok(a), Ok(b)) => neopets_data::monster::MonsterTable::parse(&a, &b).unwrap_or_default(),
        _ => Default::default(),
    };
    let mut seen = Default::default();
    for resource in 0..index.len() as u32 {
        let list = resource_entries(disc, index, &table, resource, &mut seen);
        if !list.is_empty() {
            found(list);
        }
    }
}

#[derive(Resource, Default)]
pub struct SpawnPicker {
    pub open: bool,
    entries: Vec<SpawnEntry>,
    found: Option<Mutex<Receiver<Vec<SpawnEntry>>>>,
    scanned: bool,
    filter: String,
    tab: usize,
    /// The filtered list (indices into `entries`), the cursor in it and the first row shown.
    view: Vec<usize>,
    at: usize,
    top: usize,
    note: String,
    /// The window's "spawn=NAME" request, waiting for the list.
    wanted: Option<String>,
}

impl SpawnPicker {
    fn refilter(&mut self) {
        let word = self.filter.to_lowercase();
        let tab = TABS[self.tab].1;
        self.view = (0..self.entries.len()).filter(|&i| tab.is_none_or(|g| self.entries[i].group == g) && self.entries[i].name.to_lowercase().contains(&word)).collect();
        self.view.sort_by_cached_key(|&i| self.entries[i].name.to_lowercase());
        self.at = self.at.min(self.view.len().saturating_sub(1));
        self.keep_in_view();
    }

    fn keep_in_view(&mut self) {
        if self.at < self.top {
            self.top = self.at;
        } else if self.at >= self.top + ROWS {
            self.top = self.at + 1 - ROWS;
        }
        self.top = self.top.min(self.view.len().saturating_sub(ROWS));
    }

    fn step(&mut self, by: i32) {
        self.at = (self.at as i32 + by).clamp(0, self.view.len().saturating_sub(1) as i32) as usize;
        self.keep_in_view();
    }
}

#[derive(Component)]
pub struct PickWindow;
#[derive(Component)]
pub struct PickTitle;
#[derive(Component)]
pub struct PickSearch;
#[derive(Component)]
pub struct PickNote;
#[derive(Component)]
pub struct PickRow(usize);
#[derive(Component)]
pub struct PickName(usize);
#[derive(Component)]
pub struct PickGroup(usize);

const PANEL: Color = Color::srgba(0.05, 0.06, 0.10, 0.98);
const ROW: Color = Color::srgba(1.0, 1.0, 1.0, 0.04);
const ROW_HOVER: Color = Color::srgba(1.0, 1.0, 1.0, 0.10);
const ROW_CURSOR: Color = Color::srgba(0.95, 0.75, 0.25, 0.28);
const GOLD: Color = Color::srgb(0.98, 0.80, 0.35);
const TEXT: Color = Color::srgb(0.92, 0.93, 0.96);
const DIM: Color = Color::srgb(0.60, 0.63, 0.70);
const ON: Color = Color::srgb(0.45, 0.90, 0.50);

fn font(size: f32) -> TextFont {
    TextFont { font_size: bevy::text::FontSize::Px(size), ..default() }
}

pub fn spawn_panel(mut commands: Commands) {
    commands
        .spawn((
            Node { position_type: PositionType::Absolute, left: px(16), top: px(16), width: px(620), flex_direction: FlexDirection::Column, padding: UiRect::all(px(14)), row_gap: px(3), display: Display::None, ..default() },
            crate::ui_fit::FitToWindow::Left,
            BackgroundColor(PANEL),
            GlobalZIndex(5010),
            Interaction::default(),
            PickWindow,
        ))
        .with_children(|ui| {
            ui.spawn((Text::new("Spawn a character"), font(22.0), TextColor(GOLD)));
            ui.spawn((Text::new("A test tool of this build. Characters may not act as in their own level."), font(13.0), TextColor(DIM)));
            ui.spawn((
                Text::new("Type to search, Backspace to erase, Up/Down or wheel to move, Tab for the groups, Enter or click to spawn, Esc to go back\nPad: D-pad move, LB/RB groups, A spawn, B back"),
                font(12.0),
                TextColor(DIM),
                Node { margin: UiRect { top: px(4), bottom: px(4), ..default() }, ..default() },
            ));
            ui.spawn((Text::default(), font(14.0), TextColor(TEXT), PickSearch));
            ui.spawn((Text::default(), font(12.0), TextColor(DIM), PickTitle));
            for k in 0..ROWS {
                ui.spawn((
                    Button,
                    Node { flex_direction: FlexDirection::Row, justify_content: JustifyContent::SpaceBetween, padding: UiRect::axes(px(8), px(2)), ..default() },
                    BackgroundColor(ROW),
                    PickRow(k),
                ))
                .with_children(|row| {
                    row.spawn((Text::default(), font(14.0), TextColor(TEXT), TextLayout::no_wrap(), PickName(k)));
                    row.spawn((Text::default(), font(12.0), TextColor(DIM), TextLayout::no_wrap(), PickGroup(k)));
                });
            }
            ui.spawn((Text::default(), font(14.0), TextColor(ON), Node { margin: UiRect::top(px(6)), ..default() }, PickNote));
        });
}

/// A letter, digit or space for the search line.
fn typed(key: KeyCode) -> Option<char> {
    use KeyCode::*;
    const LETTERS: [KeyCode; 26] = [KeyA, KeyB, KeyC, KeyD, KeyE, KeyF, KeyG, KeyH, KeyI, KeyJ, KeyK, KeyL, KeyM, KeyN, KeyO, KeyP, KeyQ, KeyR, KeyS, KeyT, KeyU, KeyV, KeyW, KeyX, KeyY, KeyZ];
    const DIGITS: [KeyCode; 10] = [Digit0, Digit1, Digit2, Digit3, Digit4, Digit5, Digit6, Digit7, Digit8, Digit9];
    if key == Space {
        return Some(' ');
    }
    LETTERS.iter().position(|&k| k == key).map(|i| (b'a' + i as u8) as char).or_else(|| DIGITS.iter().position(|&k| k == key).map(|i| (b'0' + i as u8) as char))
}

/// The picker's keys, pad, mouse and the list arriving from the disc. Runs before the menu's own input, which is
/// held off while the picker is up (`DevMenu::picking`).
#[allow(clippy::too_many_arguments)]
pub fn picker_input(
    keys: Res<ButtonInput<KeyCode>>,
    gamepads: Query<&Gamepad>,
    scroll: Res<AccumulatedMouseScroll>,
    rows: Query<(&Interaction, &PickRow), Changed<Interaction>>,
    archive: Option<Res<crate::Archive>>,
    play: Option<ResMut<crate::PlayState>>,
    mut dev: ResMut<DevMenu>,
    mut pick: ResMut<SpawnPicker>,
) {
    let pick = &mut *pick;
    // The key that closed the picker is kept from the menu until the next frame.
    if dev.picking && !pick.open {
        dev.picking = false;
    }
    if std::mem::take(&mut dev.want_picker) {
        pick.open = true;
        dev.picking = true;
    }
    // NEOPETS_DEV="picker" and "spawn=NAME" (diagnostic, for pictures).
    static ASKED: std::sync::Once = std::sync::Once::new();
    ASKED.call_once(|| {
        let v = std::env::var("NEOPETS_DEV").unwrap_or_default();
        pick.wanted = v.split(',').find_map(|t| t.trim().strip_prefix("spawn=")).map(str::to_string);
        if v.split(',').any(|t| t.trim() == "picker") {
            dev.open = true;
            pick.open = true;
            dev.picking = true;
        }
    });
    // The list is read once, on its own disc handle, the first time it is needed.
    if (pick.open || pick.wanted.is_some()) && pick.found.is_none() && !pick.scanned {
        if let Some(a) = archive.as_deref() {
            match a.disc.reopen() {
                Ok(mut disc) => {
                    let index = a.index.clone();
                    let (send, found) = std::sync::mpsc::channel();
                    std::thread::spawn(move || scan(&mut disc, &index, |list| drop(send.send(list))));
                    pick.found = Some(Mutex::new(found));
                }
                Err(e) => {
                    pick.note = format!("The disc could not be read: {e}");
                    pick.scanned = true;
                }
            }
        }
    }
    let mut grew = false;
    if let Some(found) = pick.found.as_ref().and_then(|f| f.lock().ok()) {
        loop {
            match found.try_recv() {
                Ok(list) => {
                    pick.entries.extend(list);
                    grew = true;
                }
                Err(TryRecvError::Empty) => break,
                Err(TryRecvError::Disconnected) => {
                    pick.scanned = true;
                    break;
                }
            }
        }
    }
    if grew {
        pick.refilter();
    }
    let Some(mut play) = play else { return };
    if pick.scanned {
        if let Some(name) = pick.wanted.take() {
            if let Some(e) = pick.entries.iter().find(|e| e.name == name) {
                play.play.dev_spawn(e.region, e.ordinal);
            }
        }
    }
    if !pick.open {
        return;
    }
    let pad = |b: GamepadButton| gamepads.iter().any(|g| g.just_pressed(b));
    if keys.just_pressed(KeyCode::Escape) || pad(GamepadButton::East) {
        pick.open = false;
        return;
    }
    if keys.just_pressed(KeyCode::ArrowUp) || pad(GamepadButton::DPadUp) {
        pick.step(-1);
    }
    if keys.just_pressed(KeyCode::ArrowDown) || pad(GamepadButton::DPadDown) {
        pick.step(1);
    }
    if keys.just_pressed(KeyCode::PageUp) {
        pick.step(-(ROWS as i32));
    }
    if keys.just_pressed(KeyCode::PageDown) {
        pick.step(ROWS as i32);
    }
    let lines = match scroll.unit {
        MouseScrollUnit::Line => scroll.delta.y,
        MouseScrollUnit::Pixel => scroll.delta.y / crate::PIXELS_PER_LINE,
    };
    if lines != 0.0 {
        pick.step(-(lines.round() as i32).clamp(-5, 5) * 2);
    }
    let tab = if keys.just_pressed(KeyCode::Tab) || pad(GamepadButton::RightTrigger) {
        1
    } else if pad(GamepadButton::LeftTrigger) {
        3
    } else {
        0
    };
    let mut changed = tab != 0;
    pick.tab = (pick.tab + tab) % TABS.len();
    for key in keys.get_just_pressed() {
        if let Some(c) = typed(*key) {
            pick.filter.push(c);
            changed = true;
        }
    }
    if keys.just_pressed(KeyCode::Backspace) && pick.filter.pop().is_some() {
        changed = true;
    }
    if changed {
        pick.at = 0;
        pick.top = 0;
        pick.refilter();
    }
    let mut chosen = keys.just_pressed(KeyCode::Enter) || pad(GamepadButton::South);
    for (i, r) in &rows {
        if *i == Interaction::Pressed && pick.top + r.0 < pick.view.len() {
            pick.at = pick.top + r.0;
            chosen = true;
        }
    }
    if chosen {
        if let Some(e) = pick.view.get(pick.at).map(|&i| pick.entries[i].clone()) {
            play.play.dev_spawn(e.region, e.ordinal);
            pick.note = format!("{} is on the way (close the menu to see it)", e.name);
        }
    }
}

#[allow(clippy::type_complexity)]
pub fn picker_show(
    pick: Res<SpawnPicker>,
    mut window: Query<&mut Node, With<PickWindow>>,
    mut search: Query<&mut Text, (With<PickSearch>, Without<PickTitle>, Without<PickNote>, Without<PickName>, Without<PickGroup>)>,
    mut title: Query<&mut Text, (With<PickTitle>, Without<PickSearch>, Without<PickNote>, Without<PickName>, Without<PickGroup>)>,
    mut note: Query<&mut Text, (With<PickNote>, Without<PickSearch>, Without<PickTitle>, Without<PickName>, Without<PickGroup>)>,
    mut names: Query<(&mut Text, &PickName), (Without<PickSearch>, Without<PickTitle>, Without<PickNote>, Without<PickGroup>)>,
    mut groups: Query<(&mut Text, &PickGroup), (Without<PickSearch>, Without<PickTitle>, Without<PickNote>, Without<PickName>)>,
    mut rows: Query<(&Interaction, &PickRow, &mut BackgroundColor)>,
) {
    let shown = if pick.open { Display::Flex } else { Display::None };
    for mut n in &mut window {
        if n.display != shown {
            n.display = shown;
        }
    }
    if !pick.open {
        return;
    }
    let set = |t: &mut Text, s: String| {
        if t.0 != s {
            t.0 = s;
        }
    };
    for mut t in &mut search {
        set(&mut t, format!("Search: {}_", pick.filter));
    }
    for mut t in &mut title {
        let tabs: Vec<String> = TABS.iter().enumerate().map(|(i, (n, _))| if i == pick.tab { format!("[{n}]") } else { (*n).to_string() }).collect();
        let reading = if pick.scanned { "" } else { "; reading the disc" };
        set(&mut t, format!("{}   {} of {} characters{reading}", tabs.join("  "), pick.view.len(), pick.entries.len()));
    }
    for mut t in &mut note {
        set(&mut t, pick.note.clone());
    }
    let entry = |k: usize| pick.view.get(pick.top + k).map(|&i| &pick.entries[i]);
    for (mut t, n) in &mut names {
        set(&mut t, entry(n.0).map_or(String::new(), |e| e.name.clone()));
    }
    for (mut t, g) in &mut groups {
        set(&mut t, entry(g.0).map_or(String::new(), |e| e.group.word().to_string()));
    }
    for (i, r, mut bg) in &mut rows {
        bg.0 = if pick.top + r.0 == pick.at && entry(r.0).is_some() {
            ROW_CURSOR
        } else if *i == Interaction::Hovered {
            ROW_HOVER
        } else {
            ROW
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(name: &str, group: Group) -> SpawnEntry {
        SpawnEntry { name: name.to_string(), group, region: 1, ordinal: 0 }
    }

    #[test]
    fn the_groups_follow_the_behaviour_class() {
        assert_eq!(Group::of(Some(17)), Group::Enemy);
        assert_eq!(Group::of(Some(6)), Group::Npc);
        assert_eq!(Group::of(None), Group::Npc);
        assert_eq!(Group::of(Some(32)), Group::Other);
    }

    #[test]
    fn the_list_filters_sorts_and_keeps_the_cursor_in_view() {
        let mut p = SpawnPicker { entries: vec![entry("Zed", Group::Enemy), entry("alpha", Group::Npc), entry("Zebra", Group::Other)], ..Default::default() };
        p.refilter();
        assert_eq!(p.view, [1, 2, 0]);
        p.filter = "ze".into();
        p.refilter();
        assert_eq!(p.view, [2, 0]);
        p.tab = 1;
        p.refilter();
        assert_eq!(p.view, [0]);
        p.filter = "nobody".into();
        p.refilter();
        assert!(p.view.is_empty());
        assert_eq!(p.at, 0);
        p.step(5);
        assert_eq!(p.at, 0);

        p = SpawnPicker { entries: (0..40).map(|k| entry(&format!("{k:02}"), Group::Npc)).collect(), ..Default::default() };
        p.refilter();
        p.step(30);
        assert_eq!((p.at, p.top), (30, 15));
        p.step(-100);
        assert_eq!((p.at, p.top), (0, 0));
    }

    #[test]
    fn typing_makes_lower_case_letters_digits_and_spaces() {
        assert_eq!(typed(KeyCode::KeyA), Some('a'));
        assert_eq!(typed(KeyCode::KeyZ), Some('z'));
        assert_eq!(typed(KeyCode::Digit7), Some('7'));
        assert_eq!(typed(KeyCode::Space), Some(' '));
        assert_eq!(typed(KeyCode::Tab), None);
    }
}
