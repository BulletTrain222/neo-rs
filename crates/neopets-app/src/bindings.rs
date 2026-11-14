//! Control bindings (a feature of this build; the original reads only the PS2 pad): every PS2
//! pad button and stick direction the game reads is a virtual pad input, fed by up to two
//! keyboard keys or mouse buttons and one gamepad button each, changed on the Options menu's
//! "Keyboard and mouse" and "Gamepad" pages and kept in a settings file. The mouse can also
//! turn the camera (the right stick) while the hero is in play.
//!
//! Gameplay and the front end read the virtual pad (`Pad`) instead of the raw devices; the
//! gamepad's two sticks still go straight through.

use bevy::input::mouse::AccumulatedMouseMotion;
use bevy::prelude::*;

/// Where the bindings are kept (local, not committed).
pub const SETTINGS_FILE: &str = "private/controls.txt";

/// The virtual pad's inputs: the left stick's four directions (movement), the right stick's
/// four (camera), then the sixteen PS2 buttons.
pub const ACTIONS: usize = 24;
/// The first button input (inputs below it are stick directions, keyboard only).
pub const FIRST_BUTTON: usize = 8;

pub const MOVE_UP: usize = 0;
pub const MOVE_DOWN: usize = 1;
pub const MOVE_LEFT: usize = 2;
pub const MOVE_RIGHT: usize = 3;
pub const LOOK_UP: usize = 4;
pub const LOOK_DOWN: usize = 5;
pub const LOOK_LEFT: usize = 6;
pub const LOOK_RIGHT: usize = 7;
pub const CROSS: usize = 8;
pub const CIRCLE: usize = 9;
pub const SQUARE: usize = 10;
pub const TRIANGLE: usize = 11;
pub const L1: usize = 12;
pub const R1: usize = 13;
pub const L2: usize = 14;
pub const R2: usize = 15;
pub const L3: usize = 16;
pub const R3: usize = 17;
pub const START: usize = 18;
pub const SELECT: usize = 19;
pub const UP: usize = 20;
pub const DOWN: usize = 21;
pub const LEFT: usize = 22;
pub const RIGHT: usize = 23;

/// Each input's name on the binding pages: what it does, and the PS2 button it stands for.
pub const NAMES: [&str; ACTIONS] = [
    "Move forward",
    "Move back",
    "Move left",
    "Move right",
    "Camera up",
    "Camera down",
    "Camera left",
    "Camera right",
    "Jump, confirm (X)",
    "Attack, answer (O)",
    "Use, talk (Square)",
    "Item panel, back (Tri.)",
    "Guard, hero switch (L1)",
    "Run, hero switch (R1)",
    "Page left (L2)",
    "Attack, page right (R2)",
    "Show the HUD (L3)",
    "Centre camera (R3)",
    "Item screen (Start)",
    "Pause (Select)",
    "Quick item up",
    "Quick item down",
    "Map, quick item left",
    "Journal, quick item right",
];

/// One keyboard-side binding: a key or a mouse button.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Input {
    Key(KeyCode),
    Mouse(MouseButton),
}

/// The keys a binding may use, with their names in the settings file and on the pages. F1 to
/// F6 stay with the developer tools.
const KEYS: &[(KeyCode, &str)] = &[
    (KeyCode::KeyA, "A"), (KeyCode::KeyB, "B"), (KeyCode::KeyC, "C"), (KeyCode::KeyD, "D"), (KeyCode::KeyE, "E"), (KeyCode::KeyF, "F"), (KeyCode::KeyG, "G"),
    (KeyCode::KeyH, "H"), (KeyCode::KeyI, "I"), (KeyCode::KeyJ, "J"), (KeyCode::KeyK, "K"), (KeyCode::KeyL, "L"), (KeyCode::KeyM, "M"), (KeyCode::KeyN, "N"),
    (KeyCode::KeyO, "O"), (KeyCode::KeyP, "P"), (KeyCode::KeyQ, "Q"), (KeyCode::KeyR, "R"), (KeyCode::KeyS, "S"), (KeyCode::KeyT, "T"), (KeyCode::KeyU, "U"),
    (KeyCode::KeyV, "V"), (KeyCode::KeyW, "W"), (KeyCode::KeyX, "X"), (KeyCode::KeyY, "Y"), (KeyCode::KeyZ, "Z"),
    (KeyCode::Digit0, "0"), (KeyCode::Digit1, "1"), (KeyCode::Digit2, "2"), (KeyCode::Digit3, "3"), (KeyCode::Digit4, "4"),
    (KeyCode::Digit5, "5"), (KeyCode::Digit6, "6"), (KeyCode::Digit7, "7"), (KeyCode::Digit8, "8"), (KeyCode::Digit9, "9"),
    (KeyCode::ArrowUp, "Up"), (KeyCode::ArrowDown, "Down"), (KeyCode::ArrowLeft, "Left"), (KeyCode::ArrowRight, "Right"),
    (KeyCode::Space, "Space"), (KeyCode::Enter, "Enter"), (KeyCode::Tab, "Tab"), (KeyCode::Escape, "Esc"), (KeyCode::Backspace, "Backspace"),
    (KeyCode::ShiftLeft, "L Shift"), (KeyCode::ShiftRight, "R Shift"), (KeyCode::ControlLeft, "L Ctrl"), (KeyCode::ControlRight, "R Ctrl"),
    (KeyCode::AltLeft, "L Alt"), (KeyCode::AltRight, "R Alt"), (KeyCode::CapsLock, "Caps Lock"),
    (KeyCode::Numpad0, "Num 0"), (KeyCode::Numpad1, "Num 1"), (KeyCode::Numpad2, "Num 2"), (KeyCode::Numpad3, "Num 3"), (KeyCode::Numpad4, "Num 4"),
    (KeyCode::Numpad5, "Num 5"), (KeyCode::Numpad6, "Num 6"), (KeyCode::Numpad7, "Num 7"), (KeyCode::Numpad8, "Num 8"), (KeyCode::Numpad9, "Num 9"),
    (KeyCode::NumpadAdd, "Num +"), (KeyCode::NumpadSubtract, "Num -"), (KeyCode::NumpadMultiply, "Num *"), (KeyCode::NumpadDivide, "Num /"),
    (KeyCode::NumpadEnter, "Num Enter"), (KeyCode::NumpadDecimal, "Num ."),
    (KeyCode::Minus, "-"), (KeyCode::Equal, "="), (KeyCode::BracketLeft, "["), (KeyCode::BracketRight, "]"), (KeyCode::Backslash, "\\"),
    (KeyCode::Semicolon, ";"), (KeyCode::Quote, "'"), (KeyCode::Comma, ","), (KeyCode::Period, "."), (KeyCode::Slash, "/"), (KeyCode::Backquote, "`"),
    (KeyCode::Insert, "Insert"), (KeyCode::Delete, "Delete"), (KeyCode::Home, "Home"), (KeyCode::End, "End"), (KeyCode::PageUp, "Page Up"), (KeyCode::PageDown, "Page Down"),
    (KeyCode::F7, "F7"), (KeyCode::F8, "F8"), (KeyCode::F9, "F9"), (KeyCode::F10, "F10"), (KeyCode::F11, "F11"), (KeyCode::F12, "F12"),
];

const MICE: &[(MouseButton, &str)] = &[(MouseButton::Left, "Left Mouse"), (MouseButton::Right, "Right Mouse"), (MouseButton::Middle, "Middle Mouse"), (MouseButton::Back, "Mouse 4"), (MouseButton::Forward, "Mouse 5")];

/// The gamepad buttons a PS2 button may use, with their names (Xbox layout first).
pub const PAD_BUTTONS: &[(GamepadButton, &str)] = &[
    (GamepadButton::South, "A / Cross"),
    (GamepadButton::East, "B / Circle"),
    (GamepadButton::West, "X / Square"),
    (GamepadButton::North, "Y / Triangle"),
    (GamepadButton::LeftTrigger, "LB / L1"),
    (GamepadButton::RightTrigger, "RB / R1"),
    (GamepadButton::LeftTrigger2, "LT / L2"),
    (GamepadButton::RightTrigger2, "RT / R2"),
    (GamepadButton::LeftThumb, "L Stick / L3"),
    (GamepadButton::RightThumb, "R Stick / R3"),
    (GamepadButton::Start, "Start / Menu"),
    (GamepadButton::Select, "Select / View"),
    (GamepadButton::DPadUp, "D-pad Up"),
    (GamepadButton::DPadDown, "D-pad Down"),
    (GamepadButton::DPadLeft, "D-pad Left"),
    (GamepadButton::DPadRight, "D-pad Right"),
];

impl Input {
    pub fn name(self) -> String {
        match self {
            Input::Key(k) => KEYS.iter().find(|e| e.0 == k).map_or_else(|| format!("{k:?}"), |e| e.1.to_string()),
            Input::Mouse(m) => MICE.iter().find(|e| e.0 == m).map_or_else(|| format!("{m:?}"), |e| e.1.to_string()),
        }
    }

    fn parse(t: &str) -> Option<Self> {
        KEYS.iter().find(|e| e.1 == t).map(|e| Input::Key(e.0)).or_else(|| MICE.iter().find(|e| e.1 == t).map(|e| Input::Mouse(e.0)))
    }

    /// Whether a key may be bound (F1 to F6 and unlisted keys may not).
    pub fn bindable_key(k: KeyCode) -> bool {
        KEYS.iter().any(|e| e.0 == k)
    }
}

pub fn pad_name(b: GamepadButton) -> String {
    PAD_BUTTONS.iter().find(|e| e.0 == b).map_or_else(|| format!("{b:?}"), |e| e.1.to_string())
}

fn pad_parse(t: &str) -> Option<GamepadButton> {
    PAD_BUTTONS.iter().find(|e| e.1 == t).map(|e| e.0)
}

/// The bindings: two keyboard or mouse inputs per virtual pad input, one gamepad button per PS2
/// button, and the mouse camera settings.
#[derive(Resource, Clone, Debug, PartialEq)]
pub struct Bindings {
    pub keys: [[Option<Input>; 2]; ACTIONS],
    pub pad: [Option<GamepadButton>; ACTIONS - FIRST_BUTTON],
    /// The mouse turns the camera while the hero is in play (the cursor is then hidden).
    pub mouse_look: bool,
    /// Mouse camera speed, 1 to 20.
    pub sensitivity: u32,
    pub invert_mouse_y: bool,
    /// The controller chord that opens and closes the developer menu: up to three buttons held
    /// together (the last of them pressed). Default LB + L3 + A.
    pub dev_chord: [Option<GamepadButton>; 3],
}

/// The developer menu chord's default: LB (L1) + L3 + A (Cross).
pub const DEV_CHORD: [Option<GamepadButton>; 3] = [Some(GamepadButton::LeftTrigger), Some(GamepadButton::LeftThumb), Some(GamepadButton::South)];

/// The chord's text: its buttons joined by " + ", or "-" when empty.
pub fn chord_name(c: &[Option<GamepadButton>; 3]) -> String {
    let names: Vec<String> = c.iter().flatten().map(|&b| pad_name(b)).collect();
    if names.is_empty() { "-".to_string() } else { names.join(" + ") }
}

/// Whether a gamepad just completed the chord: every button of it held and at least one of them
/// pressed this frame. An empty chord never fires.
pub fn chord_fired(c: &[Option<GamepadButton>; 3], held: impl Fn(GamepadButton) -> bool, pressed: impl Fn(GamepadButton) -> bool) -> bool {
    let set: Vec<GamepadButton> = c.iter().flatten().copied().collect();
    !set.is_empty() && set.iter().all(|&b| held(b)) && set.iter().any(|&b| pressed(b))
}

impl Default for Bindings {
    fn default() -> Self {
        use Input::{Key, Mouse};
        use KeyCode as K;
        let one = |k: KeyCode| [Some(Key(k)), None];
        let two = |a: Input, b: Input| [Some(a), Some(b)];
        let mut keys = [[None; 2]; ACTIONS];
        keys[MOVE_UP] = one(K::KeyW);
        keys[MOVE_DOWN] = one(K::KeyS);
        keys[MOVE_LEFT] = one(K::KeyA);
        keys[MOVE_RIGHT] = one(K::KeyD);
        keys[LOOK_UP] = one(K::KeyI);
        keys[LOOK_DOWN] = one(K::KeyK);
        keys[LOOK_LEFT] = one(K::KeyJ);
        keys[LOOK_RIGHT] = one(K::KeyL);
        keys[CROSS] = two(Key(K::Space), Key(K::Enter));
        keys[CIRCLE] = two(Key(K::KeyF), Mouse(MouseButton::Left));
        keys[SQUARE] = two(Key(K::KeyE), Mouse(MouseButton::Right));
        keys[TRIANGLE] = two(Key(K::KeyT), Key(K::Backspace));
        keys[L1] = one(K::KeyQ);
        keys[R1] = two(Key(K::ShiftLeft), Key(K::ShiftRight));
        keys[L2] = one(K::KeyZ);
        keys[R2] = one(K::KeyC);
        keys[L3] = one(K::KeyX);
        keys[R3] = two(Key(K::KeyR), Mouse(MouseButton::Middle));
        keys[START] = one(K::Tab);
        keys[SELECT] = one(K::Escape);
        keys[UP] = two(Key(K::ArrowUp), Key(K::Numpad8));
        keys[DOWN] = two(Key(K::ArrowDown), Key(K::Numpad2));
        keys[LEFT] = two(Key(K::ArrowLeft), Key(K::Numpad4));
        keys[RIGHT] = two(Key(K::ArrowRight), Key(K::Numpad6));
        Self { keys, pad: Self::default_pad(), mouse_look: true, sensitivity: 6, invert_mouse_y: false, dev_chord: DEV_CHORD }
    }
}

impl Bindings {
    pub fn default_pad() -> [Option<GamepadButton>; ACTIONS - FIRST_BUTTON] {
        std::array::from_fn(|k| Some(PAD_BUTTONS[k].0))
    }

    /// The settings file's text: one "name = value" line per setting.
    pub fn text(&self) -> String {
        let mut s = String::new();
        let slot = |i: Option<Input>| i.map_or_else(|| "-".to_string(), Input::name);
        for (k, b) in self.keys.iter().enumerate() {
            s.push_str(&format!("key {k} = {} | {}\n", slot(b[0]), slot(b[1])));
        }
        for (k, b) in self.pad.iter().enumerate() {
            s.push_str(&format!("pad {} = {}\n", k + FIRST_BUTTON, b.map_or_else(|| "-".to_string(), pad_name)));
        }
        s.push_str(&format!("mouse look = {}\nsensitivity = {}\ninvert mouse y = {}\n", self.mouse_look, self.sensitivity, self.invert_mouse_y));
        s.push_str(&format!("dev menu = {}\n", chord_name(&self.dev_chord)));
        s
    }

    /// Read the settings file's text; unknown or missing lines keep their defaults.
    pub fn parse(text: &str) -> Self {
        let mut b = Self::default();
        for line in text.lines() {
            let Some((name, value)) = line.split_once('=') else { continue };
            let (name, value) = (name.trim(), value.trim());
            let slot = |t: &str| if t == "-" { None } else { Input::parse(t) };
            if let Some(k) = name.strip_prefix("key ").and_then(|k| k.parse::<usize>().ok()).filter(|&k| k < ACTIONS) {
                let (a, c) = value.split_once('|').unwrap_or((value, "-"));
                b.keys[k] = [slot(a.trim()), slot(c.trim())];
            } else if let Some(k) = name.strip_prefix("pad ").and_then(|k| k.parse::<usize>().ok()).filter(|&k| (FIRST_BUTTON..ACTIONS).contains(&k)) {
                b.pad[k - FIRST_BUTTON] = if value == "-" { None } else { pad_parse(value) };
            } else {
                match name {
                    "mouse look" => b.mouse_look = value == "true",
                    "sensitivity" => b.sensitivity = value.parse().unwrap_or(6).clamp(1, 20),
                    "invert mouse y" => b.invert_mouse_y = value == "true",
                    "dev menu" => {
                        let mut c = [None; 3];
                        if value != "-" {
                            for (slot, t) in c.iter_mut().zip(value.split('+')) {
                                *slot = pad_parse(t.trim());
                            }
                        }
                        b.dev_chord = c;
                    }
                    _ => {}
                }
            }
        }
        b
    }

    pub fn load() -> Self {
        std::fs::read_to_string(SETTINGS_FILE).map(|t| Self::parse(&t)).unwrap_or_default()
    }

    pub fn save(&self) {
        if let Some(dir) = std::path::Path::new(SETTINGS_FILE).parent() {
            let _ = std::fs::create_dir_all(dir);
        }
        if let Err(e) = std::fs::write(SETTINGS_FILE, self.text()) {
            warn!("control bindings not saved: {e}");
        }
    }

    /// Bind `input` to slot `slot` of `action`, taking it from any other slot that had it; the
    /// action it was taken from, if any.
    pub fn bind_key(&mut self, action: usize, slot: usize, input: Input) -> Option<usize> {
        let mut taken = None;
        for (k, b) in self.keys.iter_mut().enumerate() {
            for (s, i) in b.iter_mut().enumerate() {
                if *i == Some(input) && (k, s) != (action, slot) {
                    *i = None;
                    if k != action {
                        taken = Some(k);
                    }
                }
            }
        }
        self.keys[action][slot] = Some(input);
        taken
    }

    /// Bind gamepad button `button` to PS2 button `action` (FIRST_BUTTON or above); an action
    /// that had it gets this action's old button (a swap). The action swapped with, if any.
    pub fn bind_pad(&mut self, action: usize, button: GamepadButton) -> Option<usize> {
        let k = action - FIRST_BUTTON;
        let old = self.pad[k];
        let other = self.pad.iter().position(|&b| b == Some(button)).filter(|&j| j != k);
        if let Some(j) = other {
            self.pad[j] = old;
        }
        self.pad[k] = Some(button);
        other.map(|j| j + FIRST_BUTTON)
    }
}

/// The virtual pad of this frame: each input's held and pressed bits (the pressed bits without
/// the mouse buttons for the front end), the keyboard's move direction and the camera's (keys
/// and mouse).
#[derive(Resource, Default, Clone, Copy, Debug)]
pub struct Pad {
    pub held: u32,
    pub pressed: u32,
    pub pressed_no_mouse: u32,
    pub held_no_mouse: u32,
    pub movement: Vec2,
    pub look: Vec2,
}

impl Pad {
    pub fn held(&self, a: usize) -> bool {
        self.held & (1 << a) != 0
    }
    pub fn pressed(&self, a: usize) -> bool {
        self.pressed & (1 << a) != 0
    }
}

/// Whether the virtual pad is muted this frame (a binding page open or just closed).
#[derive(Resource, Default)]
pub struct PadMute(pub bool);

/// The controller player 2 has in local co-op (coop.rs): left out of player 1's pad.
#[derive(Resource, Default)]
pub struct PadExclude(pub Option<Entity>);

/// Whether the mouse turns the camera this frame (the cursor is grabbed in play).
#[derive(Resource, Default)]
pub struct MouseLook(pub bool);

/// Mouse motion in pixels to a right-stick value: the sensitivity scale, then a lift past the
/// PS2 dead zone (0.31 of the stick's travel) so slow motion still turns the camera.
pub fn mouse_stick(delta: f32, sensitivity: u32) -> f32 {
    let v = (delta * sensitivity as f32 * 0.012).clamp(-1.0, 1.0);
    if v.abs() < 1e-4 { 0.0 } else { v.signum() * (0.32 + 0.68 * v.abs()) }
}

/// Read the virtual pad from the keyboard, mouse and gamepads.
#[allow(clippy::too_many_arguments)]
#[allow(clippy::too_many_arguments)]
pub fn read_pad(keys: Res<ButtonInput<KeyCode>>, mouse: Res<ButtonInput<MouseButton>>, motion: Res<AccumulatedMouseMotion>, all_pads: Query<(Entity, &Gamepad)>, exclude: Res<PadExclude>, bindings: Res<Bindings>, mute: Res<PadMute>, look: Res<MouseLook>, mut pad: ResMut<Pad>, menu: Option<Res<crate::menu_view::Menu>>) {
    let gamepads: Vec<&Gamepad> = all_pads.iter().filter(|(e, _)| Some(*e) != exclude.0).map(|(_, g)| g).collect();
    let _span = crate::perf::span("read_pad");
    *pad = Pad::default();
    if mute.0 {
        return;
    }
    // A text row being typed in (the Multiplayer page's join address or password): the
    // keyboard types there, so its bound keys do nothing else (Backspace was Triangle, which went
    // back a page; the number pad's 8, 2, 4 and 6 moved the cursor), except the arrows, Enter,
    // Tab and Esc, which still move off the row and leave the page.
    let typing = menu.as_ref().is_some_and(|m| {
        m.flow.current == crate::menus::DISPLAY && m.flow.display_page == 2 && matches!(m.flow.display_focus, crate::display::ADDRESS_ROW | crate::display::PASSWORD_ROW)
    });
    let kept = |k: KeyCode| !typing || matches!(k, KeyCode::ArrowUp | KeyCode::ArrowDown | KeyCode::ArrowLeft | KeyCode::ArrowRight | KeyCode::Enter | KeyCode::Tab | KeyCode::Escape);
    for a in 0..ACTIONS {
        let bit = 1u32 << a;
        for i in bindings.keys[a].iter().flatten() {
            match *i {
                Input::Key(k) if !kept(k) => {}
                Input::Key(k) => {
                    if keys.pressed(k) {
                        pad.held |= bit;
                        pad.held_no_mouse |= bit;
                    }
                    if keys.just_pressed(k) {
                        pad.pressed |= bit;
                        pad.pressed_no_mouse |= bit;
                    }
                }
                Input::Mouse(m) => {
                    if mouse.pressed(m) {
                        pad.held |= bit;
                    }
                    if mouse.just_pressed(m) {
                        pad.pressed |= bit;
                    }
                }
            }
        }
        if let Some(b) = a.checked_sub(FIRST_BUTTON).and_then(|k| bindings.pad[k]) {
            if gamepads.iter().any(|g| g.pressed(b)) {
                pad.held |= bit;
                pad.held_no_mouse |= bit;
            }
            if gamepads.iter().any(|g| g.just_pressed(b)) {
                pad.pressed |= bit;
                pad.pressed_no_mouse |= bit;
            }
        }
    }
    // NEOPETS_DEV_KEYS="ms:BUTTON;..." (diagnostic, for screenshots of the menus): a press of
    // that virtual-pad button (CROSS, CIRCLE, SQUARE, TRIANGLE, START, SELECT, UP, DOWN, LEFT,
    // RIGHT, L1, R1, L2, R2) on the first frame at or past that many milliseconds from the
    // first read.
    if let Ok(v) = std::env::var("NEOPETS_DEV_KEYS") {
        static T0: std::sync::OnceLock<std::time::Instant> = std::sync::OnceLock::new();
        static DONE: std::sync::Mutex<Vec<usize>> = std::sync::Mutex::new(Vec::new());
        let ms = T0.get_or_init(std::time::Instant::now).elapsed().as_millis() as u64;
        for (k, e) in v.split(';').enumerate() {
            let Some((f, name)) = e.split_once(':') else { continue };
            let Ok(mut done) = DONE.lock() else { continue };
            if f.trim().parse::<u64>().map_or(true, |f| ms < f) || done.contains(&k) {
                continue;
            }
            done.push(k);
            let a = match name.trim() {
                "CROSS" => CROSS,
                "CIRCLE" => CIRCLE,
                "SQUARE" => SQUARE,
                "TRIANGLE" => TRIANGLE,
                "START" => START,
                "SELECT" => SELECT,
                "UP" => UP,
                "DOWN" => DOWN,
                "LEFT" => LEFT,
                "RIGHT" => RIGHT,
                "L1" => L1,
                "R1" => R1,
                "L2" => L2,
                "R2" => R2,
                _ => continue,
            };
            pad.held |= 1 << a;
            pad.pressed |= 1 << a;
            pad.held_no_mouse |= 1 << a;
            pad.pressed_no_mouse |= 1 << a;
        }
    }
    let held = *pad;
    let axis = |neg: usize, pos: usize| held.held(pos) as i32 as f32 - held.held(neg) as i32 as f32;
    pad.movement = Vec2::new(axis(MOVE_LEFT, MOVE_RIGHT), axis(MOVE_DOWN, MOVE_UP));
    pad.look = Vec2::new(axis(LOOK_LEFT, LOOK_RIGHT), axis(LOOK_DOWN, LOOK_UP));
    if look.0 {
        let d = motion.delta;
        let dy = if bindings.invert_mouse_y { d.y } else { -d.y };
        pad.look.x += mouse_stick(d.x, bindings.sensitivity);
        pad.look.y += mouse_stick(dy, bindings.sensitivity);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn settings_round_trip() {
        let mut b = Bindings::default();
        b.bind_key(CIRCLE, 1, Input::Key(KeyCode::KeyG));
        b.bind_pad(CROSS, GamepadButton::East);
        b.sensitivity = 11;
        b.invert_mouse_y = true;
        let t = b.text();
        assert_eq!(Bindings::parse(&t), b);
        assert_eq!(Bindings::parse(""), Bindings::default());
    }

    #[test]
    fn binding_takes_and_swaps() {
        let mut b = Bindings::default();
        // E is Square's; binding it to Cross's second slot takes it from Square.
        assert_eq!(b.bind_key(CROSS, 1, Input::Key(KeyCode::KeyE)), Some(SQUARE));
        assert_eq!(b.keys[SQUARE][0], None);
        assert_eq!(b.keys[CROSS][1], Some(Input::Key(KeyCode::KeyE)));
        // The east button is Circle's; binding it to Cross swaps: Circle gets South.
        assert_eq!(b.bind_pad(CROSS, GamepadButton::East), Some(CIRCLE));
        assert_eq!(b.pad[CIRCLE - FIRST_BUTTON], Some(GamepadButton::South));
        assert_eq!(b.pad[CROSS - FIRST_BUTTON], Some(GamepadButton::East));
    }

    #[test]
    fn menu_words_follow_the_bindings_and_skip_the_mouse() {
        use crate::menus;
        // Cross pressed by a key, Circle only by the mouse, the movement key for up.
        let bit = |a: usize| 1u32 << a;
        let pad = Pad { pressed: bit(CROSS) | bit(CIRCLE) | bit(MOVE_UP), pressed_no_mouse: bit(CROSS) | bit(MOVE_UP), held_no_mouse: bit(LEFT), ..Default::default() };
        let (edge, buttons) = crate::menu_view::menu_words(&pad);
        assert_eq!(buttons, menus::CROSS);
        assert_eq!(edge, menus::CROSS | menus::UP);
        assert_eq!(crate::menu_view::menu_held(&pad), menus::LEFT);
    }

    #[test]
    fn every_button_has_a_default_and_no_key_is_bound_twice() {
        let b = Bindings::default();
        for a in 0..ACTIONS {
            assert!(b.keys[a][0].is_some(), "{} has no key", NAMES[a]);
        }
        let all: Vec<Input> = b.keys.iter().flatten().flatten().copied().collect();
        for (i, x) in all.iter().enumerate() {
            assert!(!all[i + 1..].contains(x), "{} bound twice", x.name());
        }
        assert!(b.pad.iter().all(Option::is_some));
    }

    #[test]
    fn mouse_stick_lifts_past_the_dead_zone() {
        assert_eq!(mouse_stick(0.0, 6), 0.0);
        assert!(mouse_stick(1.0, 6) > 0.32);
        assert!(mouse_stick(-1.0, 6) < -0.32);
        assert_eq!(mouse_stick(10_000.0, 6), 1.0);
    }
}
