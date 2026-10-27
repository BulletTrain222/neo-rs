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
        Self { keys, pad: Self::default_pad(), mouse_look: true, sensitivity: 6, invert_mouse_y: false }
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
pub fn read_pad(keys: Res<ButtonInput<KeyCode>>, mouse: Res<ButtonInput<MouseButton>>, motion: Res<AccumulatedMouseMotion>, gamepads: Query<&Gamepad>, bindings: Res<Bindings>, mute: Res<PadMute>, look: Res<MouseLook>, mut pad: ResMut<Pad>) {
    *pad = Pad::default();
    if mute.0 {
        return;
    }
    for a in 0..ACTIONS {
        let bit = 1u32 << a;
        for i in bindings.keys[a].iter().flatten() {
            match *i {
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
    fn mouse_stick_lifts_past_the_dead_zone() {
        assert_eq!(mouse_stick(0.0, 6), 0.0);
        assert!(mouse_stick(1.0, 6) > 0.32);
        assert!(mouse_stick(-1.0, 6) < -0.32);
        assert_eq!(mouse_stick(10_000.0, 6), 1.0);
    }
}
