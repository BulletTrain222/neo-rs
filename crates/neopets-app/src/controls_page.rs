//! The bindings page (a feature of this build): opened from the Options menu's "Keyboard &
//! Gamepad" row, with a Keyboard & Mouse tab (two keys or mouse buttons per input, and the mouse
//! camera settings) and a Gamepad tab (one button per PS2 button). It works with the keyboard,
//! a gamepad or the mouse, and its own navigation keys are fixed so a bad binding cannot lock
//! anyone out. The game's input is muted while it is up.

use crate::bindings::{self as b, Bindings, Input, PadMute};
use bevy::prelude::*;
use bevy::window::{CursorGrabMode, CursorOptions, PrimaryWindow};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Row {
    Header(&'static str),
    Bind(usize),
    MouseLook,
    Sensitivity,
    InvertY,
    /// The developer menu's controller chord (up to three buttons).
    DevChord,
    Reset,
    Back,
}

/// A tab's rows: the two columns, then the footer rows (across the page).
fn layout(tab: usize) -> ([Vec<Row>; 2], Vec<Vec<Row>>) {
    let binds = |r: std::ops::Range<usize>| r.map(Row::Bind).collect::<Vec<_>>();
    if tab == 0 {
        let mut left = vec![Row::Header("MOVE AND CAMERA")];
        left.extend(binds(0..b::FIRST_BUTTON));
        left.push(Row::Header("D-PAD"));
        left.extend(binds(b::UP..b::ACTIONS));
        let mut right = vec![Row::Header("BUTTONS")];
        right.extend(binds(b::FIRST_BUTTON..b::UP));
        ([left, right], vec![vec![Row::MouseLook, Row::Sensitivity, Row::InvertY], vec![Row::Reset, Row::Back]])
    } else {
        let mut left = vec![Row::Header("FACE AND SHOULDER BUTTONS")];
        left.extend(binds(b::FIRST_BUTTON..b::L3));
        let mut right = vec![Row::Header("STICK BUTTONS, START, SELECT, D-PAD")];
        right.extend(binds(b::L3..b::ACTIONS));
        ([left, right], vec![vec![Row::DevChord], vec![Row::Reset, Row::Back]])
    }
}

/// The chord row's capture marker in `ControlsPage::capture`.
const CHORD: usize = usize::MAX;

/// The rows the cursor can stand on, in order.
fn selectable(tab: usize) -> Vec<Row> {
    let (cols, footer) = layout(tab);
    cols.into_iter().flatten().chain(footer.into_iter().flatten()).filter(|r| !matches!(r, Row::Header(_))).collect()
}

#[derive(Resource, Default)]
pub struct ControlsPage {
    pub open: bool,
    tab: usize,
    cursor: usize,
    slot: usize,
    /// Waiting for an input for (action, slot); frames to skip first (the press that started
    /// it), and the seconds left.
    capture: Option<(usize, usize)>,
    capture_skip: u8,
    capture_left: f32,
    note: String,
    /// The tab the rows were built for.
    built: Option<usize>,
    /// Frames the game's input stays muted after the page closes.
    closing: u8,
    /// A mouse press on a row or slot this frame.
    clicked: Option<(usize, Option<usize>)>,
    clicked_tab: Option<usize>,
    /// The chord being recorded: the buttons in the order they went down.
    chord: Vec<GamepadButton>,
}

#[derive(Component)]
pub struct CtlWindow;
#[derive(Component)]
pub struct CtlBody;
#[derive(Component)]
pub struct CtlTab(usize);
#[derive(Component)]
pub struct CtlRow(usize);
#[derive(Component)]
pub struct CtlSlot(usize, usize);
#[derive(Component)]
pub struct CtlSlotText(usize, usize);
#[derive(Component)]
pub struct CtlValue(usize);
#[derive(Component)]
pub struct CtlNote;

const PANEL: Color = Color::srgb(0.05, 0.06, 0.10);
const ROW: Color = Color::srgba(1.0, 1.0, 1.0, 0.04);
const ROW_HOVER: Color = Color::srgba(1.0, 1.0, 1.0, 0.10);
const ROW_CURSOR: Color = Color::srgba(0.95, 0.75, 0.25, 0.22);
const SLOT: Color = Color::srgba(1.0, 1.0, 1.0, 0.08);
const SLOT_CURSOR: Color = Color::srgba(0.95, 0.75, 0.25, 0.65);
const SLOT_WAIT: Color = Color::srgba(0.45, 0.75, 0.95, 0.75);
const TAB: Color = Color::srgba(1.0, 1.0, 1.0, 0.06);
const TAB_ON: Color = Color::srgba(0.95, 0.75, 0.25, 0.45);
const GOLD: Color = Color::srgb(0.98, 0.80, 0.35);
const TEXT: Color = Color::srgb(0.92, 0.93, 0.96);
const DIM: Color = Color::srgb(0.60, 0.63, 0.70);
const ON: Color = Color::srgb(0.45, 0.90, 0.50);

const TABS: [&str; 2] = ["Keyboard & Mouse", "Gamepad"];

fn font(size: f32) -> TextFont {
    TextFont { font_size: bevy::text::FontSize::Px(size), ..default() }
}

pub fn spawn_page(mut commands: Commands) {
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: percent(50),
                top: px(16),
                margin: UiRect::left(px(-440)),
                width: px(880),
                max_height: percent(94),
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(px(14)),
                row_gap: px(3),
                overflow: Overflow::clip_y(),
                display: Display::None,
                ..default()
            },
            BackgroundColor(PANEL),
            GlobalZIndex(5003),
            CtlWindow,
        ))
        .with_children(|ui| {
            ui.spawn(Node { flex_direction: FlexDirection::Row, column_gap: px(8), align_items: AlignItems::Center, margin: UiRect::bottom(px(4)), ..default() }).with_children(|tabs| {
                tabs.spawn((Text::new("Controls"), font(22.0), TextColor(GOLD), Node { margin: UiRect::right(px(16)), ..default() }));
                for (k, name) in TABS.iter().enumerate() {
                    tabs.spawn((Button, Node { padding: UiRect::axes(px(14), px(5)), ..default() }, BackgroundColor(TAB), CtlTab(k))).with_children(|t| {
                        t.spawn((Text::new(*name), font(15.0), TextColor(TEXT)));
                    });
                }
            });
            ui.spawn((
                Text::new("Enter, A or a click changes a slot; then press the new key or button (Esc cancels). Delete or Y clears it.\nArrows or D-pad move, Tab or LB/RB switch tabs, Esc, Backspace or B go back."),
                font(12.0),
                TextColor(DIM),
            ));
            ui.spawn((Node { flex_direction: FlexDirection::Column, row_gap: px(3), margin: UiRect::top(px(4)), ..default() }, CtlBody));
            ui.spawn((Text::default(), font(14.0), TextColor(ON), Node { margin: UiRect::top(px(6)), min_height: px(18), ..default() }, CtlNote));
        });
}

/// Build the open tab's rows under the body node.
fn build(commands: &mut Commands, body: Entity, tab: usize) {
    commands.entity(body).despawn_related::<Children>();
    let ([left, right], footer) = layout(tab);
    let slots = if tab == 0 { 2 } else { 1 };
    let mut k = 0usize;
    let mut row = |ui: &mut ChildSpawnerCommands, r: Row, width: f32| {
        match r {
            Row::Header(t) => {
                ui.spawn((Text::new(t), font(12.0), TextColor(GOLD), Node { margin: UiRect { top: px(4), bottom: px(1), ..default() }, ..default() }));
                return;
            }
            _ => {}
        }
        let i = k;
        k += 1;
        ui.spawn((
            Button,
            Node { flex_direction: FlexDirection::Row, justify_content: JustifyContent::SpaceBetween, align_items: AlignItems::Center, padding: UiRect::axes(px(8), px(2)), min_height: px(21), width: px(width), ..default() },
            BackgroundColor(ROW),
            CtlRow(i),
        ))
        .with_children(|row| {
            let label = match r {
                Row::Bind(a) => b::NAMES[a].to_string(),
                Row::MouseLook => "Mouse camera".into(),
                Row::Sensitivity => "Mouse speed".into(),
                Row::InvertY => "Invert mouse Y".into(),
                Row::DevChord => "Developer menu (up to 3 buttons held together)".into(),
                Row::Reset => format!("Reset {} to defaults", TABS[tab]),
                Row::Back => "Back".into(),
                Row::Header(_) => String::new(),
            };
            row.spawn((Text::new(label), font(13.0), TextColor(TEXT), TextLayout::no_wrap()));
            match r {
                Row::Bind(_) => {
                    row.spawn(Node { flex_direction: FlexDirection::Row, column_gap: px(4), ..default() }).with_children(|s| {
                        for slot in 0..slots {
                            s.spawn((Button, Node { width: px(if slots == 2 { 92.0 } else { 150.0 }), height: px(18), justify_content: JustifyContent::Center, align_items: AlignItems::Center, ..default() }, BackgroundColor(SLOT), CtlSlot(i, slot)))
                                .with_children(|t| {
                                    t.spawn((Text::default(), font(13.0), TextColor(TEXT), TextLayout::no_wrap(), CtlSlotText(i, slot)));
                                });
                        }
                    });
                }
                Row::DevChord => {
                    row.spawn((Button, Node { width: px(360), height: px(18), justify_content: JustifyContent::Center, align_items: AlignItems::Center, ..default() }, BackgroundColor(SLOT), CtlSlot(i, 0))).with_children(|t| {
                        t.spawn((Text::default(), font(13.0), TextColor(TEXT), TextLayout::no_wrap(), CtlSlotText(i, 0)));
                    });
                }
                Row::MouseLook | Row::Sensitivity | Row::InvertY => {
                    row.spawn((Text::default(), font(14.0), TextColor(GOLD), CtlValue(i)));
                }
                _ => {}
            }
        });
    };
    commands.entity(body).with_children(|ui| {
        ui.spawn(Node { flex_direction: FlexDirection::Row, column_gap: px(16), ..default() }).with_children(|cols| {
            for col in [left, right] {
                cols.spawn(Node { flex_direction: FlexDirection::Column, row_gap: px(3), ..default() }).with_children(|c| {
                    for r in col {
                        row(c, r, 420.0);
                    }
                });
            }
        });
        for line in footer {
            let w = (852.0 - 12.0 * (line.len() as f32 - 1.0)) / line.len() as f32;
            ui.spawn(Node { flex_direction: FlexDirection::Row, column_gap: px(12), margin: UiRect::top(px(4)), ..default() }).with_children(|f| {
                for r in line {
                    row(f, r, w);
                }
            });
        }
        if tab == 1 {
            ui.spawn((Text::new("The left stick always moves and the right stick turns the camera."), font(12.0), TextColor(DIM), Node { margin: UiRect::top(px(4)), ..default() }));
        }
    });
}

/// The mouse on the page: hovering moves the cursor, a press on a row or slot is kept for
/// `page_input`, a press on a tab switches to it.
pub fn page_mouse(mut page: ResMut<ControlsPage>, rows: Query<(&Interaction, &CtlRow), Changed<Interaction>>, slots: Query<(&Interaction, &CtlSlot), Changed<Interaction>>, tabs: Query<(&Interaction, &CtlTab), Changed<Interaction>>) {
    if !page.open || page.capture.is_some() {
        return;
    }
    for (i, r) in &rows {
        match i {
            Interaction::Hovered => page.cursor = r.0,
            Interaction::Pressed => page.clicked = Some((r.0, None)),
            Interaction::None => {}
        }
    }
    for (i, s) in &slots {
        match i {
            Interaction::Hovered => {
                page.cursor = s.0;
                page.slot = s.1;
            }
            Interaction::Pressed => page.clicked = Some((s.0, Some(s.1))),
            Interaction::None => {}
        }
    }
    for (i, t) in &tabs {
        if *i == Interaction::Pressed {
            page.clicked_tab = Some(t.0);
        }
    }
}

/// The page's keys, buttons and clicks, before the game reads its input: opening from the
/// Options row, the tab and cursor moves, the captures and the setting changes.
#[allow(clippy::too_many_arguments)]
pub fn page_input(
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    gamepads: Query<&Gamepad>,
    menu: Option<ResMut<crate::menu_view::Menu>>,
    mut page: ResMut<ControlsPage>,
    mut bindings: ResMut<Bindings>,
    mut mute: ResMut<PadMute>,
) {
    if let Some(mut m) = menu {
        if std::mem::take(&mut m.flow.controls_request) {
            *page = ControlsPage { open: true, ..default() };
            // NEOPETS_TRACE_PAGE="pad@n": the Gamepad tab (diagnostic, for a screenshot).
            if std::env::var("NEOPETS_TRACE_PAGE").is_ok_and(|v| v.starts_with("pad@")) {
                page.tab = 1;
            }
        }
    }
    page.closing = page.closing.saturating_sub(1);
    mute.0 = page.open || page.closing > 0;
    if !page.open {
        return;
    }
    let pad = |btn: GamepadButton| gamepads.iter().any(|g| g.just_pressed(btn));
    let rows = selectable(page.tab);
    // A capture: the next key or mouse button (keyboard tab) or gamepad button (gamepad tab).
    if let Some((action, slot)) = page.capture {
        if page.capture_skip > 0 {
            page.capture_skip -= 1;
            return;
        }
        page.capture_left -= time.delta_secs();
        if keys.just_pressed(KeyCode::Escape) || page.capture_left <= 0.0 {
            page.capture = None;
            page.note = "Unchanged".into();
            return;
        }
        // The chord: buttons are added as they go down (up to three); it is kept when they are
        // all released again.
        if action == CHORD {
            let down: Vec<GamepadButton> = b::PAD_BUTTONS.iter().map(|e| e.0).filter(|&btn| pad(btn)).collect();
            for btn in down {
                if !page.chord.contains(&btn) && page.chord.len() < 3 {
                    page.chord.push(btn);
                }
            }
            let any_held = gamepads.iter().any(|g| b::PAD_BUTTONS.iter().any(|e| g.pressed(e.0)));
            if !page.chord.is_empty() && !any_held {
                let mut c = [None; 3];
                for (slot, &btn) in c.iter_mut().zip(page.chord.iter()) {
                    *slot = Some(btn);
                }
                bindings.dev_chord = c;
                page.note = format!("Developer menu: {}", b::chord_name(&c));
                page.capture = None;
                page.chord.clear();
                bindings.save();
            }
            return;
        }
        let name = b::NAMES[action];
        if page.tab == 0 {
            let input = keys.get_just_pressed().copied().find(|&k| Input::bindable_key(k)).map(Input::Key).or_else(|| mouse.get_just_pressed().copied().next().map(Input::Mouse));
            if let Some(i) = input {
                let taken = bindings.bind_key(action, slot, i);
                page.note = match taken {
                    Some(t) => format!("{name}: {} (taken from {})", i.name(), b::NAMES[t]),
                    None => format!("{name}: {}", i.name()),
                };
                page.capture = None;
                bindings.save();
            }
        } else if let Some(&(btn, _)) = b::PAD_BUTTONS.iter().find(|(btn, _)| pad(*btn)) {
            let swapped = bindings.bind_pad(action, btn);
            page.note = match swapped {
                Some(t) => format!("{name}: {} (swapped with {})", b::pad_name(btn), b::NAMES[t]),
                None => format!("{name}: {}", b::pad_name(btn)),
            };
            page.capture = None;
            bindings.save();
        }
        return;
    }
    let slots = if page.tab == 0 { 2 } else { 1 };
    // Tabs.
    let switch = page.clicked_tab.take().or_else(|| (keys.just_pressed(KeyCode::Tab) || pad(GamepadButton::LeftTrigger) || pad(GamepadButton::RightTrigger)).then_some(1 - page.tab));
    if let Some(t) = switch {
        if t != page.tab {
            page.tab = t;
            page.cursor = 0;
            page.slot = 0;
            page.note.clear();
        }
        return;
    }
    let close = |page: &mut ControlsPage| {
        page.open = false;
        page.closing = 3;
        page.built = None;
    };
    if keys.just_pressed(KeyCode::Escape) || keys.just_pressed(KeyCode::Backspace) || pad(GamepadButton::East) {
        close(&mut page);
        return;
    }
    let n = rows.len();
    if keys.just_pressed(KeyCode::ArrowDown) || pad(GamepadButton::DPadDown) {
        page.cursor = (page.cursor + 1) % n;
    }
    if keys.just_pressed(KeyCode::ArrowUp) || pad(GamepadButton::DPadUp) {
        page.cursor = (page.cursor + n - 1) % n;
    }
    let left = keys.just_pressed(KeyCode::ArrowLeft) || pad(GamepadButton::DPadLeft);
    let right = keys.just_pressed(KeyCode::ArrowRight) || pad(GamepadButton::DPadRight);
    let clicked = page.clicked.take();
    if let Some((r, s)) = clicked {
        page.cursor = r;
        if let Some(s) = s {
            page.slot = s;
        }
    }
    let apply = clicked.is_some() || keys.just_pressed(KeyCode::Enter) || keys.just_pressed(KeyCode::NumpadEnter) || keys.just_pressed(KeyCode::Space) || pad(GamepadButton::South);
    let clear = keys.just_pressed(KeyCode::Delete) || pad(GamepadButton::North);
    let cursor = page.cursor.min(n - 1);
    match rows[cursor] {
        Row::Bind(a) => {
            if left || right {
                page.slot = if slots == 2 { 1 - page.slot.min(1) } else { 0 };
            }
            let slot = page.slot.min(slots - 1);
            if clear {
                if page.tab == 0 {
                    bindings.keys[a][slot] = None;
                } else {
                    bindings.pad[a - b::FIRST_BUTTON] = None;
                }
                page.note = format!("{}: cleared", b::NAMES[a]);
                bindings.save();
            } else if apply {
                page.capture = Some((a, slot));
                // Skip the press that started it; a mouse click waits a few frames more.
                page.capture_skip = if clicked.is_some() { 6 } else { 2 };
                page.capture_left = 6.0;
                page.note = if page.tab == 0 { format!("Press a key or mouse button for {} (Esc cancels)", b::NAMES[a]) } else { format!("Press a gamepad button for {} (Esc cancels)", b::NAMES[a]) };
            }
        }
        Row::DevChord if clear => {
            bindings.dev_chord = [None; 3];
            page.note = "Developer menu chord cleared (F1 still opens it)".into();
            bindings.save();
        }
        Row::DevChord if apply => {
            page.capture = Some((CHORD, 0));
            page.chord.clear();
            page.capture_skip = if clicked.is_some() { 6 } else { 2 };
            page.capture_left = 8.0;
            page.note = "Hold up to three gamepad buttons together, then let go (Esc cancels)".into();
        }
        Row::MouseLook if apply || left || right => {
            bindings.mouse_look = !bindings.mouse_look;
            bindings.save();
        }
        Row::InvertY if apply || left || right => {
            bindings.invert_mouse_y = !bindings.invert_mouse_y;
            bindings.save();
        }
        Row::Sensitivity if apply || left || right => {
            let d: i32 = if left { -1 } else { 1 };
            let v = bindings.sensitivity as i32 + d;
            bindings.sensitivity = if apply && !right && v > 20 { 1 } else { v.clamp(1, 20) as u32 };
            bindings.save();
        }
        Row::Reset if apply => {
            let d = Bindings::default();
            if page.tab == 0 {
                bindings.keys = d.keys;
                bindings.mouse_look = d.mouse_look;
                bindings.sensitivity = d.sensitivity;
                bindings.invert_mouse_y = d.invert_mouse_y;
            } else {
                bindings.pad = d.pad;
                bindings.dev_chord = d.dev_chord;
            }
            bindings.save();
            page.note = format!("{} reset to defaults", TABS[page.tab]);
        }
        Row::Back if apply => close(&mut page),
        _ => {}
    }
}

/// Show the page: build the open tab's rows, then the texts and highlights.
#[allow(clippy::type_complexity)]
pub fn show_page(
    mut commands: Commands,
    mut page: ResMut<ControlsPage>,
    bindings: Res<Bindings>,
    mut windows: Query<&mut Node, With<CtlWindow>>,
    body: Query<Entity, With<CtlBody>>,
    mut tabs: Query<(&CtlTab, &mut BackgroundColor), (Without<CtlRow>, Without<CtlSlot>)>,
    mut rows: Query<(&Interaction, &CtlRow, &mut BackgroundColor), (Without<CtlTab>, Without<CtlSlot>)>,
    mut slots: Query<(&CtlSlot, &mut BackgroundColor), (Without<CtlTab>, Without<CtlRow>)>,
    mut slot_texts: Query<(&CtlSlotText, &mut Text), (Without<CtlValue>, Without<CtlNote>)>,
    mut values: Query<(&CtlValue, &mut Text), (Without<CtlSlotText>, Without<CtlNote>)>,
    mut notes: Query<&mut Text, (With<CtlNote>, Without<CtlSlotText>, Without<CtlValue>)>,
) {
    for mut n in &mut windows {
        n.display = if page.open { Display::Flex } else { Display::None };
    }
    if !page.open {
        return;
    }
    if page.built != Some(page.tab) {
        if let Ok(e) = body.single() {
            build(&mut commands, e, page.tab);
            page.built = Some(page.tab);
        }
        return;
    }
    let list = selectable(page.tab);
    for (t, mut bg) in &mut tabs {
        bg.0 = if t.0 == page.tab { TAB_ON } else { TAB };
    }
    for (i, r, mut bg) in &mut rows {
        bg.0 = if r.0 == page.cursor { ROW_CURSOR } else if *i == Interaction::Hovered { ROW_HOVER } else { ROW };
    }
    for (s, mut bg) in &mut slots {
        let here = s.0 == page.cursor && s.1 == page.slot;
        let waiting = page.capture.is_some() && here;
        bg.0 = if waiting { SLOT_WAIT } else if here { SLOT_CURSOR } else { SLOT };
    }
    for (s, mut t) in &mut slot_texts {
        let row = list.get(s.0).copied();
        if row == Some(Row::DevChord) {
            let text = if page.capture.is_some_and(|c| c.0 == CHORD) {
                if page.chord.is_empty() { "Hold the buttons...".to_string() } else { page.chord.iter().map(|&btn| b::pad_name(btn)).collect::<Vec<_>>().join(" + ") }
            } else {
                b::chord_name(&bindings.dev_chord)
            };
            if t.0 != text {
                t.0 = text;
            }
            continue;
        }
        let Some(Row::Bind(a)) = row else { continue };
        let text = if page.capture.is_some() && s.0 == page.cursor && s.1 == page.slot {
            "Press...".to_string()
        } else if page.tab == 0 {
            bindings.keys[a][s.1].map_or_else(|| "-".to_string(), Input::name)
        } else {
            bindings.pad[a - b::FIRST_BUTTON].map_or_else(|| "-".to_string(), b::pad_name)
        };
        if t.0 != text {
            t.0 = text;
        }
    }
    for (v, mut t) in &mut values {
        let on = |x: bool| if x { "ON".to_string() } else { "OFF".to_string() };
        let text = match list.get(v.0) {
            Some(Row::MouseLook) => on(bindings.mouse_look),
            Some(Row::InvertY) => on(bindings.invert_mouse_y),
            Some(Row::Sensitivity) => format!("< {} >", bindings.sensitivity),
            _ => String::new(),
        };
        if t.0 != text {
            t.0 = text;
        }
    }
    for mut t in &mut notes {
        if t.0 != page.note {
            t.0 = page.note.clone();
        }
    }
}

/// The mouse camera: in play (no front-end page, developer menu or bindings page up, the window
/// focused) with mouse look on, the cursor is locked and hidden and the mouse turns the camera.
pub fn grab_cursor(
    play: Option<Res<crate::PlayState>>,
    menu: Option<Res<crate::menu_view::Menu>>,
    dev: Res<crate::dev_menu::DevMenu>,
    page: Res<ControlsPage>,
    bindings: Res<Bindings>,
    mut look: ResMut<b::MouseLook>,
    mut windows: Query<(&Window, &mut CursorOptions), With<PrimaryWindow>>,
) {
    let in_play = play.is_some() && menu.as_ref().is_some_and(|m| matches!(m.flow.current, crate::menus::PLAY | crate::menus::BOX_HOST));
    let Ok((window, mut cursor)) = windows.single_mut() else { return };
    let on = bindings.mouse_look && in_play && window.focused && !dev.blocks_input() && !page.open && std::env::var_os("NEOPETS_NO_GRAB").is_none();
    look.0 = on;
    let mode = if on { CursorGrabMode::Locked } else { CursorGrabMode::None };
    if cursor.grab_mode != mode {
        cursor.grab_mode = mode;
        cursor.visible = !on;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tabs_list_every_input_once() {
        let binds = |tab: usize| selectable(tab).into_iter().filter_map(|r| if let Row::Bind(a) = r { Some(a) } else { None }).collect::<Vec<_>>();
        let mut all = binds(0);
        all.sort_unstable();
        assert_eq!(all, (0..b::ACTIONS).collect::<Vec<_>>());
        assert_eq!(binds(1), (b::FIRST_BUTTON..b::ACTIONS).collect::<Vec<_>>());
        assert_eq!(selectable(0).last(), Some(&Row::Back));
        assert!(selectable(1).contains(&Row::DevChord));
    }

    #[test]
    fn dev_chord_fires_and_round_trips() {
        use GamepadButton as G;
        let c = b::DEV_CHORD;
        // All three held, the last just pressed: fires; one missing or none new: does not.
        assert!(b::chord_fired(&c, |_| true, |x| x == G::South));
        assert!(!b::chord_fired(&c, |x| x != G::LeftThumb, |x| x == G::South));
        assert!(!b::chord_fired(&c, |_| true, |_| false));
        assert!(!b::chord_fired(&[None; 3], |_| true, |_| true));
        // Select alone no longer opens it.
        assert!(!b::chord_fired(&c, |x| x == G::Select, |x| x == G::Select));
        // The settings file keeps it.
        let mut bind = Bindings::default();
        bind.dev_chord = [Some(G::Select), Some(G::North), None];
        assert_eq!(Bindings::parse(&bind.text()).dev_chord, bind.dev_chord);
        assert_eq!(Bindings::parse(&Bindings::default().text()).dev_chord, c);
    }
}
