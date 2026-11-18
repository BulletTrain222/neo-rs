//! Display settings (a feature of this build; the original has no display options, SPEC-0035
//! Behavior 3): window mode, resolution, aspect ratio, render scale and the upscale filter,
//! changed on the Options menu's Display page and kept in a settings file.
//!
//! The 3D scene is drawn into an offscreen image the size of the game viewport (the window
//! fitted to the chosen aspect ratio) times the render scale, and a window camera shows that
//! image in the viewport, scaled with a sharp or smooth filter. Menus and the HUD draw on top at
//! the window's full resolution.

use bevy::camera::RenderTarget;
use bevy::image::ImageSampler;
use bevy::prelude::*;
use bevy::render::render_resource::TextureFormat;
use bevy::window::{MonitorSelection, PrimaryWindow, VideoModeSelection, WindowMode};

/// Where the settings are kept (local, not committed).
/// The Button pictures choices (`DisplaySettings::buttons`).
pub const BUTTON_PICTURES: [&str; 4] = ["PlayStation", "Xbox", "Keyboard", "Auto"];

pub const SETTINGS_FILE: &str = "private/display.txt";

/// Render scales in percent of the viewport.
pub const SCALES: [u32; 8] = [50, 67, 75, 85, 100, 125, 150, 200];
/// Aspect ratios: a name and the width over height (None fills the window).
pub const ASPECTS: [(&str, Option<f32>); 5] = [("Fill window", None), ("4:3", Some(4.0 / 3.0)), ("16:9", Some(16.0 / 9.0)), ("16:10", Some(16.0 / 10.0)), ("21:9", Some(21.0 / 9.0))];
const MODES: [&str; 3] = ["Windowed", "Borderless", "Fullscreen"];
/// Window sizes offered when the monitor lists no video modes.
const FALLBACK_SIZES: [(u32, u32); 8] = [(3840, 2160), (2560, 1440), (1920, 1200), (1920, 1080), (1600, 900), (1280, 960), (1280, 720), (1024, 768)];
/// The Display page's rows.
pub const ROWS: usize = 53;
/// The rows on the Gameplay page, which shares the Display page's window and settings file; the
/// other rows are the Display page's.
pub const GAMEPLAY_ROWS: [usize; 5] = [17, 18, 19, 20, ATTACK_TURN_ROW];
/// The Gameplay page's Attack turn row (this build's option).
pub const ATTACK_TURN_ROW: usize = 44;
/// The rows on the Multiplayer page (docs/MULTIPLAYER.md; this build's feature), in the order
/// shown: the tabs' rows in turn (This computer, Online, Players).
pub const MULTI_ROWS: [usize; 26] = [COOP_ROW, PLAYERS_ROW, 26, PAD_ROWS[0], PAD_ROWS[1], PAD_ROWS[2], PAD_ROWS[3], HOST_ROW, REMOVE_ROW, UPNP_ROW, GAME_ROWS[0], GAME_ROWS[1], GAME_ROWS[2], GAME_ROWS[3], ADDRESS_ROW, JOIN_ROW, PASSWORD_ROW, PORT_ROW, DOORS_ROW, COLOUR_ROWS[0], COLOUR_ROWS[1], COLOUR_ROWS[2], COLOUR_ROWS[3], TAGS_ROW, 25, TOGETHER_ROW];
/// A page's tabs: each tab's name and its rows, in the order shown.
pub type Tabs = [(&'static str, &'static [usize])];
/// The Display page's tabs: the window and speed; the preset and the picture; shadows and how far
/// things are drawn.
pub static DISPLAY_TABS: [(&str, &[usize]); 3] = [
    ("Screen", &[0, 1, 2, 3, 4, 16, 22]),
    ("Graphics", &[5, 6, 7, 10, 9, 24]),
    ("Shadows & distance", &[12, 13, 14, 15, 11, 21]),
];
/// The Multiplayer page's tabs.
pub static MULTI_TABS: [(&str, &[usize]); 3] = [
    ("This computer", &[COOP_ROW, PLAYERS_ROW, 26, PAD_ROWS[0], PAD_ROWS[1], PAD_ROWS[2], PAD_ROWS[3]]),
    ("Online", &[HOST_ROW, REMOVE_ROW, UPNP_ROW, GAME_ROWS[0], GAME_ROWS[1], GAME_ROWS[2], GAME_ROWS[3], ADDRESS_ROW, JOIN_ROW, PASSWORD_ROW, PORT_ROW, DOORS_ROW]),
    ("Players", &[COLOUR_ROWS[0], COLOUR_ROWS[1], COLOUR_ROWS[2], COLOUR_ROWS[3], TAGS_ROW, 25, TOGETHER_ROW]),
];
/// The most tabs a page has (the tab bar's chips).
pub const MAX_TABS: usize = 3;
/// A page's tabs (none on the Gameplay page).
pub fn tabs(page: u8) -> &'static Tabs {
    match page {
        0 => &DISPLAY_TABS,
        2 => &MULTI_TABS,
        _ => &[],
    }
}
/// The headings over groups of a tabbed page's rows: (the row they come before, text).
pub const HEADINGS: [(usize, &str); 16] = [
    (0, "Window"),
    (3, "Picture"),
    (16, "Speed"),
    (5, "Preset"),
    (6, "Detail"),
    (9, "Look"),
    (12, "Shadows"),
    (11, "Distance"),
    (COOP_ROW, "Split screen co-op"),
    (PAD_ROWS[0], "Controllers"),
    (HOST_ROW, "Host"),
    (GAME_ROWS[0], "Games on this network"),
    (ADDRESS_ROW, "Join by address"),
    (PASSWORD_ROW, "Both"),
    (COLOUR_ROWS[0], "Look"),
    (25, "Rules"),
];
/// The rows drawn as a button (an action, not a choice): start or stop local co-op, host, join,
/// remove the last joiner.
pub const ACTION_ROWS: [usize; 8] = [COOP_ROW, HOST_ROW, JOIN_ROW, REMOVE_ROW, GAME_ROWS[0], GAME_ROWS[1], GAME_ROWS[2], GAME_ROWS[3]];
/// The game finder's rows (net.rs `Finder`): a game hosting on this home network each, a click
/// joining it; the first row says "Looking..." while none has answered.
pub const GAME_ROWS: [usize; 4] = [49, 50, 51, 52];
/// The rows typed into, drawn as a text box (the join address, the password, the port).
pub const TEXT_ROWS: [usize; 3] = [ADDRESS_ROW, PASSWORD_ROW, PORT_ROW];
/// The focus value of a tabbed page's tab bar (Left and Right change the tab).
pub const TAB_FOCUS: usize = ROWS + 1;

/// The rows a page shows, in order, with Back at the end; a tabbed page shows its tab bar first
/// and its current tab's rows, and the Multiplayer page a controller row only for as many players
/// as play.
pub fn visible_rows(page: u8, tab: u8, players: usize, found: usize) -> Vec<usize> {
    let t = tabs(page);
    let mut v = if !t.is_empty() {
        let mut v = vec![TAB_FOCUS];
        // A game finder row for each game found (the first shows while none has).
        let shown = |r: &usize| !PAD_ROWS.iter().position(|p| p == r).is_some_and(|k| k >= players) && !GAME_ROWS.iter().position(|g| g == r).is_some_and(|k| k > 0 && k >= found);
        v.extend(t[(tab as usize).min(t.len() - 1)].1.iter().copied().filter(shown));
        v
    } else {
        page_rows(page)
    };
    v.push(ROWS);
    v
}
/// Rows no page shows any more (27, the Player 2 partner choice: every player starts as Tor and
/// switches on its own; 8 and 23, Bloom and Light shafts, always off now).
#[cfg_attr(not(test), allow(dead_code))]
pub const RETIRED_ROWS: [usize; 3] = [27, 8, 23];
/// Local co-op on this computer: start or stop it.
pub const COOP_ROW: usize = 37;
/// How many players local co-op has (2 to 4).
pub const PLAYERS_ROW: usize = 38;
/// Each player's controller (players 1 to 4).
pub const PAD_ROWS: [usize; 4] = [39, 40, 41, 42];
/// Each player's colour (players 1 to 4; Tor's yellow fur recoloured; this build's option).
pub const COLOUR_ROWS: [usize; 4] = [45, 46, 47, 48];
/// Player tags (P1 to P4) over the other players.
pub const TAGS_ROW: usize = 43;
/// A controller choice: Auto, or controller 1 to this many (in the order they were connected).
pub const PAD_CHOICES: u8 = 4;
/// The Multiplayer page's online rows (net.rs): host, the join address, join.
pub const HOST_ROW: usize = 28;
pub const ADDRESS_ROW: usize = 29;
pub const JOIN_ROW: usize = 30;
/// The host sends player 2 away (it keeps hosting).
pub const REMOVE_ROW: usize = 31;
/// The game's password (none by default), typed with letters and digits.
pub const PASSWORD_ROW: usize = 32;
/// Ask the router to forward the port while hosting (UPnP).
pub const UPNP_ROW: usize = 33;
/// The port to host and join on (default 47817), typed with the number keys.
pub const PORT_ROW: usize = 34;
/// Co-op: player 2 kept near player 1, or free to wander the loaded area.
pub const TOGETHER_ROW: usize = 35;
/// Online: a door takes both players, or only the one who goes through it.
pub const DOORS_ROW: usize = 36;
/// Split screen layouts.
pub const SPLITS: [&str; 2] = ["Top and bottom", "Side by side"];

/// The rows of the Display page (0), the Gameplay page (1) or the Multiplayer page (2), in the
/// order shown.
pub fn page_rows(page: u8) -> Vec<usize> {
    match page {
        1 => GAMEPLAY_ROWS.to_vec(),
        2 => MULTI_ROWS.to_vec(),
        _ => DISPLAY_TABS.iter().flat_map(|t| t.1.iter().copied()).collect(),
    }
}
/// Character shadows: off, the original volumes as the game draws them (blocky hulls, a drop of
/// 8 of 255), or this build's detailed ones (each character's own shape, cast by the sun).
pub const CHAR_SHADOWS: [&str; 3] = ["Off", "Original", "Detailed"];

/// The sun shadow quality (shadow_view.rs: the map size and cascades of each level).
pub const SHADOW_QUALITY: [&str; 4] = ["Very Low", "Low", "Medium", "High"];
/// How far from the camera the sun's shadows reach, in world units (shadow_view.rs).
pub const SHADOW_DISTANCES: [f32; 5] = [50.0, 100.0, 150.0, 200.0, 300.0];
/// The frame rate limits, frames per second; 0 is uncapped.
pub const FPS_LIMITS: [u32; 4] = [30, 60, 120, 0];

/// The graphics APIs on offer (main.rs `render_settings`; Windows only, read at start).
pub const GRAPHICS_APIS: [&str; 2] = ["Vulkan", "DX12"];

/// The graphics presets (`DisplaySettings::apply_preset`).
pub const PRESETS: [&str; 3] = ["Low", "Medium", "High"];

#[derive(Resource, Clone, Debug, PartialEq)]
pub struct DisplaySettings {
    /// 0 windowed, 1 borderless fullscreen, 2 exclusive fullscreen.
    pub mode: usize,
    /// The window (or exclusive fullscreen) size in physical pixels.
    pub size: (u32, u32),
    /// Index into ASPECTS.
    pub aspect: usize,
    /// Index into SCALES.
    pub scale: usize,
    /// Smooth (linear) or sharp (nearest) upscaling.
    pub smooth: bool,
    /// Graphics options (graphics.rs): anti-aliasing (index into graphics::AA), sharper
    /// textures at a distance, bloom and colour grade (indices), ambient occlusion.
    pub aa: usize,
    pub sharp_textures: bool,
    pub bloom: usize,
    pub colour: usize,
    pub ao: bool,
    /// Distant foliage as the original draws it (SPEC-0060: far sprites, fades out by distance;
    /// SPEC-0046: instanced meshes hidden past their draw distance) instead of full models at
    /// every distance (this build's default).
    pub original_foliage: bool,
    /// Character shadows (index into CHAR_SHADOWS; SPEC-0046 Character shadow volumes).
    pub char_shadows: usize,
    /// Shadows cast by everything (trees, buildings, props): this build's option, not in the
    /// original.
    pub world_shadows: bool,
    /// The sun shadow quality (index into SHADOW_QUALITY): this build's option.
    pub shadow_quality: usize,
    /// How far the sun's shadows reach (index into SHADOW_DISTANCES): this build's option.
    pub shadow_distance: usize,
    /// The frame rate limit (index into FPS_LIMITS): this build's option.
    pub fps: usize,
    /// Movie cutscenes played (on) or passed over at once, as if skipped (this build's testing
    /// option, not in the original).
    pub movies: bool,
    /// Button pictures (`BUTTON_PICTURES`): the game's PlayStation icons (default), Xbox-style
    /// A, B, X and Y, keyboard keys, or Auto (Xbox after a gamepad press, keys after a key or
    /// click) (button_style.rs; this build's option).
    pub buttons: usize,
    /// The HUD's D-pad hint drawn (default) or hidden (this build's option, not in the original).
    pub dpad_hud: bool,
    /// Menu pictures carry their edges out to the window's sides (this build's option; off draws the original 512 x 448 frame).
    pub menu_edges: bool,
    /// The neighbouring world drawn in full (this build's default) or gated by the disc's zone
    /// tables as the original draws it (SPEC-0006 Zones), which hides some of its pieces from
    /// parts of the current world (user reports 112, 114 and 116).
    pub original_neighbour: bool,
    /// The graphics API on Windows (index into GRAPHICS_APIS), taken at the next start: this
    /// build's option. Vulkan is faster with the display on the discrete GPU; DX12 can be faster
    /// when a laptop's display hangs off its integrated GPU.
    pub graphics_api: usize,
    /// Light shafts and haze from the sun (godrays.rs): this build's option, not in the original.
    pub light_shafts: bool,
    /// Reflections on flat water (water_reflect.rs): this build's option, not in the original.
    pub water_reflections: bool,
    /// Local co-op: a player at 0 health is knocked down and can be revived (true), or the game is
    /// over at once (false).
    pub coop_revive: bool,
    /// Local co-op: the split screen's layout (`SPLITS`).
    pub coop_split: usize,
    /// Local co-op: player 2 is the partner hero when the story has one waiting (true), else
    /// always a second Tor.
    pub coop_partner: bool,
    /// Ask the router to forward the hosting port (UPnP; this build's option, off by default).
    pub upnp: bool,
    /// The UDP port to host on, and to join on when an address names none (default 47817).
    pub net_port: u16,
    /// Co-op: player 2 is brought back beside player 1 when far away (on by default).
    pub coop_together: bool,
    /// Online: a door takes only the player who goes through it (off by default: both).
    pub coop_doors_alone: bool,
    /// Local co-op: how many players (2 to 4).
    pub coop_players: usize,
    /// Each player's controller: 0 Auto, k controller k in the order they were connected.
    pub coop_pads: [u8; 4],
    /// A tag (P1 to P4) over each other player (on by default).
    pub coop_tags: bool,
    /// Each player's colour in multiplayer (`hero_colour::COLOURS`; 0 the original yellow).
    pub coop_colours: [u8; 4],
    /// Steady: an attack turns the hero toward the enemy's body; Original: toward its target
    /// point (SPEC-0040 Behavior 20 item 7, attachment point 7, which on some enemies swings
    /// with their limbs). This build's option, Steady by default (user report 150).
    pub steady_turn: bool,
}

impl Default for DisplaySettings {
    fn default() -> Self {
        Self { mode: 0, size: (1000, 600), aspect: 0, scale: 4, smooth: true, aa: 2, sharp_textures: true, bloom: 0, colour: 1, ao: true, original_foliage: false, char_shadows: 1, world_shadows: false, shadow_quality: 2, shadow_distance: 2, fps: 2, movies: true, buttons: 0, dpad_hud: true, menu_edges: true, original_neighbour: false, graphics_api: 0, light_shafts: false, water_reflections: false, coop_revive: true, coop_split: 0, coop_partner: true, upnp: false, net_port: crate::net::PORT, coop_together: true, coop_doors_alone: false, coop_players: 2, coop_pads: [0; 4], coop_tags: true, steady_turn: true, coop_colours: [0, 1, 2, 3] }
    }
}

impl DisplaySettings {
    /// Parse "key = value" lines; unknown keys and bad values keep the defaults.
    pub fn parse(text: &str) -> Self {
        let mut s = Self::default();
        for line in text.lines() {
            let Some((k, v)) = line.split_once('=') else { continue };
            let v = v.trim();
            match k.trim() {
                "mode" => s.mode = v.parse().unwrap_or(s.mode).min(MODES.len() - 1),
                "size" => {
                    if let Some((w, h)) = v.split_once('x').and_then(|(w, h)| Some((w.trim().parse().ok()?, h.trim().parse().ok()?))) {
                        s.size = (w, h);
                    }
                }
                "aspect" => s.aspect = v.parse().unwrap_or(s.aspect).min(ASPECTS.len() - 1),
                "scale" => {
                    if let Some(i) = v.parse::<u32>().ok().and_then(|p| SCALES.iter().position(|&x| x == p)) {
                        s.scale = i;
                    }
                }
                "filter" => s.smooth = v != "sharp",
                "antialiasing" => s.aa = v.parse().unwrap_or(s.aa).min(crate::graphics::AA.len() - 1),
                "sharp textures" => s.sharp_textures = v == "true",
                // Bloom and light shafts are gone (the user's choice: they did not suit this
                // game); a saved value is passed over and both stay off.
                "bloom" => {}
                "colour" => s.colour = v.parse().unwrap_or(s.colour).min(crate::graphics::COLOUR.len() - 1),
                "ambient occlusion" => s.ao = v == "true",
                "original foliage" => s.original_foliage = v == "true",
                "character shadows" => s.char_shadows = v.parse().unwrap_or(s.char_shadows).min(CHAR_SHADOWS.len() - 1),
                "world shadows" => s.world_shadows = v == "true",
                "shadow level" => s.shadow_quality = v.parse().unwrap_or(s.shadow_quality).min(SHADOW_QUALITY.len() - 1),
                // The older key counted from Low, before Very Low was added.
                "shadow quality" => s.shadow_quality = v.parse::<usize>().map_or(s.shadow_quality, |q| q + 1).min(SHADOW_QUALITY.len() - 1),
                "shadow distance" => s.shadow_distance = v.parse().unwrap_or(s.shadow_distance).min(SHADOW_DISTANCES.len() - 1),
                "frame rate" => s.fps = v.parse().unwrap_or(s.fps).min(FPS_LIMITS.len() - 1),
                "movies" => s.movies = v != "false",
                "button pictures" => s.buttons = BUTTON_PICTURES.iter().position(|b| b.eq_ignore_ascii_case(v)).unwrap_or(0),
                "dpad hud" => s.dpad_hud = v != "false",
                "menu edges" => s.menu_edges = v != "false",
                "original neighbour" => s.original_neighbour = v == "true",
                "light shafts" => {}
                "water reflections" => s.water_reflections = v == "true",
                "coop revive" => s.coop_revive = v == "true",
                "coop split" => s.coop_split = v.parse().unwrap_or(0).min(SPLITS.len() - 1),
                "coop partner" => s.coop_partner = v == "true",
                "upnp" => s.upnp = v == "true",
                "coop together" => s.coop_together = v == "true",
                "coop doors alone" => s.coop_doors_alone = v == "true",
                "coop tags" => s.coop_tags = v != "false",
                "attack turn" => s.steady_turn = v != "original",
                "coop colours" => {
                    for (k, n) in v.split(',').take(4).enumerate() {
                        s.coop_colours[k] = n.trim().parse::<u8>().unwrap_or(0).min(crate::hero_colour::COLOURS.len() as u8 - 1);
                    }
                }
                "coop players" => s.coop_players = v.parse().unwrap_or(2).clamp(2, 4),
                "coop pads" => {
                    for (k, n) in v.split(',').take(4).enumerate() {
                        s.coop_pads[k] = n.trim().parse::<u8>().unwrap_or(0).min(PAD_CHOICES);
                    }
                }
                "net port" => s.net_port = v.parse().ok().filter(|&p: &u16| p > 0).unwrap_or(crate::net::PORT),
                "graphics api" => s.graphics_api = GRAPHICS_APIS.iter().position(|a| a.eq_ignore_ascii_case(v)).unwrap_or(s.graphics_api),
                _ => {}
            }
        }
        s
    }

    pub fn text(&self) -> String {
        format!(
            "mode = {}\nsize = {}x{}\naspect = {}\nscale = {}\nfilter = {}\nantialiasing = {}\nsharp textures = {}\nbloom = {}\ncolour = {}\nambient occlusion = {}\noriginal foliage = {}\ncharacter shadows = {}\nworld shadows = {}\nshadow level = {}\nshadow distance = {}\nframe rate = {}\nmovies = {}\nbutton pictures = {}\ndpad hud = {}\nmenu edges = {}\noriginal neighbour = {}\ngraphics api = {}\nlight shafts = {}\nwater reflections = {}\ncoop revive = {}\ncoop split = {}\ncoop partner = {}\nupnp = {}\nnet port = {}\ncoop together = {}\ncoop doors alone = {}\ncoop players = {}\ncoop pads = {}\ncoop tags = {}\nattack turn = {}\ncoop colours = {}\n",
            self.mode,
            self.size.0,
            self.size.1,
            self.aspect,
            SCALES[self.scale],
            if self.smooth { "smooth" } else { "sharp" },
            self.aa,
            self.sharp_textures,
            self.bloom,
            self.colour,
            self.ao,
            self.original_foliage,
            self.char_shadows,
            self.world_shadows,
            self.shadow_quality,
            self.shadow_distance,
            self.fps,
            self.movies,
            BUTTON_PICTURES[self.buttons].to_lowercase(),
            self.dpad_hud,
            self.menu_edges,
            self.original_neighbour,
            GRAPHICS_APIS[self.graphics_api].to_lowercase(),
            self.light_shafts,
            self.water_reflections,
            self.coop_revive,
            self.coop_split,
            self.coop_partner,
            self.upnp,
            self.net_port,
            self.coop_together,
            self.coop_doors_alone,
            self.coop_players,
            self.coop_pads.map(|n| n.to_string()).join(","),
            self.coop_tags,
            if self.steady_turn { "steady" } else { "original" },
            self.coop_colours.map(|n| n.to_string()).join(",")
        )
    }

    pub fn load() -> Self {
        let mut s = std::fs::read_to_string(SETTINGS_FILE).map(|t| Self::parse(&t)).unwrap_or_default();
        // NEOPETS_GFX="aa,textures,bloom,colour,ao[,foliage]" (0/1 for the switches): the graphics options
        // for this run (diagnostic, for screenshot comparisons; not saved unless changed).
        // NEOPETS_SHADOWS="char,world[,quality]": the shadow settings for this run (diagnostic).
        if let Some(v) = std::env::var("NEOPETS_SHADOWS").ok() {
            let n: Vec<usize> = v.split(',').filter_map(|t| t.trim().parse().ok()).collect();
            if let Some(&c) = n.first() {
                s.char_shadows = c.min(CHAR_SHADOWS.len() - 1);
            }
            if let Some(&w) = n.get(1) {
                s.world_shadows = w != 0;
            }
            // An optional third value: the shadow quality (0 Very Low, 1 Low, 2 Medium, 3 High);
            // a fourth: the shadow distance index.
            if let Some(&q) = n.get(2) {
                s.shadow_quality = q.min(SHADOW_QUALITY.len() - 1);
            }
            if let Some(&d) = n.get(3) {
                s.shadow_distance = d.min(SHADOW_DISTANCES.len() - 1);
            }
        }
        // NEOPETS_FPS=index: the frame rate limit for this run (diagnostic).
        if let Some(i) = std::env::var("NEOPETS_FPS").ok().and_then(|v| v.trim().parse::<usize>().ok()) {
            s.fps = i.min(FPS_LIMITS.len() - 1);
        }
        // NEOPETS_SCALE=percent: the render scale for this run (diagnostic).
        if let Some(i) = std::env::var("NEOPETS_SCALE").ok().and_then(|v| v.trim().parse::<u32>().ok()).and_then(|p| SCALES.iter().position(|&x| x == p)) {
            s.scale = i;
        }
        // NEOPETS_BUTTONS=xbox, keyboard or auto: those button pictures for this run (diagnostic).
        if let Ok(v) = std::env::var("NEOPETS_BUTTONS") {
            s.buttons = BUTTON_PICTURES.iter().position(|b| b.eq_ignore_ascii_case(v.trim())).unwrap_or(0);
        }
        // NEOPETS_MOVIES=0 or 1: Movie cutscenes for this run (diagnostic).
        if let Ok(v) = std::env::var("NEOPETS_MOVIES") {
            s.movies = v.trim() != "0";
        }
        if let Some(v) = std::env::var("NEOPETS_GFX").ok() {
            let n: Vec<usize> = v.split(',').filter_map(|t| t.trim().parse().ok()).collect();
            if let [aa, tex, bloom, colour, ao, ref rest @ ..] = n[..] {
                (s.aa, s.sharp_textures, s.bloom, s.colour, s.ao) = (aa.min(3), tex != 0, bloom.min(2), colour.min(2), ao != 0);
                // An optional sixth value: 1 for the original distant foliage.
                if let Some(&f) = rest.first() {
                    s.original_foliage = f != 0;
                }
            }
        }
        s
    }

    pub fn save(&self) {
        if let Some(dir) = std::path::Path::new(SETTINGS_FILE).parent() {
            let _ = std::fs::create_dir_all(dir);
        }
        if let Err(e) = std::fs::write(SETTINGS_FILE, self.text()) {
            warn!("display settings not saved: {e}");
        }
    }

    /// A graphics preset's options: (anti-aliasing, bloom, ambient occlusion, world shadows,
    /// shadow quality). Low turns off the costly passes (FXAA, no bloom, no ambient occlusion,
    /// no world shadows, Low shadows); Medium is the new-player look (SMAA, bloom, ambient
    /// occlusion, Medium shadows, no world shadows); High adds world shadows at High quality.
    /// The other options (colour, textures, foliage, character shadows) are left as they are.
    fn preset_values(p: usize) -> (usize, usize, bool, bool, usize) {
        match p {
            0 => (3, 0, false, false, 1),
            1 => (2, 0, true, false, 2),
            _ => (2, 0, true, true, 3),
        }
    }

    /// The preset the options match, if any (the Display page shows Custom otherwise).
    pub fn preset(&self) -> Option<usize> {
        (0..PRESETS.len()).find(|&p| Self::preset_values(p) == (self.aa, self.bloom, self.ao, self.world_shadows, self.shadow_quality))
    }

    pub fn apply_preset(&mut self, p: usize) {
        (self.aa, self.bloom, self.ao, self.world_shadows, self.shadow_quality) = Self::preset_values(p);
    }

    /// The rows as (name, value) for the Display page.
    pub fn rows(&self) -> [(String, String); ROWS] {
        [
            ("Window".into(), MODES[self.mode].into()),
            ("Resolution".into(), if self.mode == 1 { "Desktop".into() } else { format!("{} x {}", self.size.0, self.size.1) }),
            ("Aspect ratio".into(), ASPECTS[self.aspect].0.into()),
            ("Render scale".into(), format!("{}%", SCALES[self.scale])),
            ("Upscale filter".into(), if self.smooth { "Smooth".into() } else { "Sharp".into() }),
            ("Graphics preset".into(), self.preset().map_or("Custom", |p| PRESETS[p]).into()),
            ("Anti-aliasing".into(), crate::graphics::AA[self.aa].into()),
            ("Distant textures".into(), if self.sharp_textures { "Sharp".into() } else { "Original".into() }),
            ("Bloom".into(), crate::graphics::BLOOM[self.bloom].into()),
            ("Colour".into(), crate::graphics::COLOUR[self.colour].into()),
            ("Ambient occlusion".into(), if self.ao { "On".into() } else { "Off".into() }),
            ("Distant foliage".into(), if self.original_foliage { "Original".into() } else { "Full".into() }),
            ("Character shadows".into(), CHAR_SHADOWS[self.char_shadows].into()),
            ("World shadows".into(), if self.world_shadows { "On".into() } else { "Off".into() }),
            ("Shadow quality".into(), SHADOW_QUALITY[self.shadow_quality].into()),
            ("Shadow distance".into(), format!("{}", SHADOW_DISTANCES[self.shadow_distance])),
            ("Frame rate limit".into(), match FPS_LIMITS[self.fps] { 0 => "Uncapped".into(), n => format!("{n}") }),
            ("Movie cutscenes".into(), if self.movies { "On".into() } else { "Off".into() }),
            ("Button pictures".into(), BUTTON_PICTURES[self.buttons].into()),
            ("D-pad HUD".into(), if self.dpad_hud { "On".into() } else { "Off".into() }),
            ("Menu edges".into(), if self.menu_edges { "Extended".into() } else { "Original".into() }),
            ("Neighbour world".into(), if self.original_neighbour { "Original".into() } else { "Full".into() }),
            ("Graphics API".into(), GRAPHICS_APIS[self.graphics_api].into()),
            ("Light shafts".into(), if self.light_shafts { "On".into() } else { "Off".into() }),
            ("Water reflections".into(), if self.water_reflections { "On".into() } else { "Off".into() }),
            ("Knocked out".into(), if self.coop_revive { "Revive".into() } else { "Game over".into() }),
            ("Split screen".into(), SPLITS[self.coop_split].into()),
            ("Player 2".into(), if self.coop_partner { "Partner".into() } else { "Second Tor".into() }),
            // The online rows' values come from the session (show_window).
            ("Host a game".into(), String::new()),
            ("Join address".into(), String::new()),
            ("Join a game".into(), String::new()),
            ("Remove a player".into(), String::new()),
            ("Password".into(), String::new()),
            ("Open the port (UPnP)".into(), if self.upnp { "On".into() } else { "Off".into() }),
            ("Port".into(), if self.net_port == crate::net::PORT { format!("{} (default)", self.net_port) } else { self.net_port.to_string() }),
            ("Stay together".into(), if self.coop_together { "On".into() } else { "Off".into() }),
            ("Doors take".into(), if self.coop_doors_alone { "Only you".into() } else { "Both".into() }),
            // The Local co-op row's value comes from the game (show_window).
            ("Local co-op".into(), String::new()),
            ("Players".into(), self.coop_players.to_string()),
            ("Player 1 controller".into(), pad_name(self.coop_pads[0])),
            ("Player 2 controller".into(), pad_name(self.coop_pads[1])),
            ("Player 3 controller".into(), pad_name(self.coop_pads[2])),
            ("Player 4 controller".into(), pad_name(self.coop_pads[3])),
            ("Player tags".into(), if self.coop_tags { "On".into() } else { "Off".into() }),
            ("Attack turn".into(), if self.steady_turn { "Steady".into() } else { "Original".into() }),
            ("Player 1 colour".into(), crate::hero_colour::COLOURS[self.coop_colours[0] as usize].into()),
            ("Player 2 colour".into(), crate::hero_colour::COLOURS[self.coop_colours[1] as usize].into()),
            ("Player 3 colour".into(), crate::hero_colour::COLOURS[self.coop_colours[2] as usize].into()),
            ("Player 4 colour".into(), crate::hero_colour::COLOURS[self.coop_colours[3] as usize].into()),
            // The game finder's rows (their words come from what it found: show_window).
            ("Game 1".into(), String::new()),
            ("Game 2".into(), String::new()),
            ("Game 3".into(), String::new()),
            ("Game 4".into(), String::new()),
        ]
    }

    /// Step row `row` by `delta` (Left -1, Right +1), wrapping; `sizes` are the resolutions on
    /// offer, largest first.
    pub fn change(&mut self, row: usize, delta: i32, sizes: &[(u32, u32)]) {
        let step = |i: usize, n: usize| (i as i32 + delta).rem_euclid(n as i32) as usize;
        match row {
            0 => self.mode = step(self.mode, MODES.len()),
            1 if !sizes.is_empty() => {
                // Larger first in the list, so Right (+1) moves to a smaller size: flip it. A size
                // not on the list steps to the nearest smaller (Right) or larger (Left) one.
                let area = |s: (u32, u32)| s.0 as u64 * s.1 as u64;
                self.size = match sizes.iter().position(|&s| s == self.size) {
                    Some(at) => sizes[(at as i32 - delta).rem_euclid(sizes.len() as i32) as usize],
                    None if delta > 0 => sizes.iter().copied().find(|&s| area(s) < area(self.size)).unwrap_or(sizes[0]),
                    None => sizes.iter().rev().copied().find(|&s| area(s) > area(self.size)).unwrap_or(sizes[sizes.len() - 1]),
                };
            }
            2 => self.aspect = step(self.aspect, ASPECTS.len()),
            3 => self.scale = (self.scale as i32 + delta).clamp(0, SCALES.len() as i32 - 1) as usize,
            4 => self.smooth = !self.smooth,
            5 => {
                let next = match self.preset() {
                    Some(p) => (p as i32 + delta).rem_euclid(PRESETS.len() as i32) as usize,
                    None if delta > 0 => 0,
                    None => PRESETS.len() - 1,
                };
                self.apply_preset(next);
            }
            6 => self.aa = step(self.aa, crate::graphics::AA.len()),
            7 => self.sharp_textures = !self.sharp_textures,
            8 => self.bloom = step(self.bloom, crate::graphics::BLOOM.len()),
            9 => self.colour = step(self.colour, crate::graphics::COLOUR.len()),
            10 => self.ao = !self.ao,
            11 => self.original_foliage = !self.original_foliage,
            12 => self.char_shadows = step(self.char_shadows, CHAR_SHADOWS.len()),
            13 => self.world_shadows = !self.world_shadows,
            14 => self.shadow_quality = step(self.shadow_quality, SHADOW_QUALITY.len()),
            15 => self.shadow_distance = step(self.shadow_distance, SHADOW_DISTANCES.len()),
            16 => self.fps = step(self.fps, FPS_LIMITS.len()),
            17 => self.movies = !self.movies,
            18 => self.buttons = step(self.buttons, BUTTON_PICTURES.len()),
            19 => self.dpad_hud = !self.dpad_hud,
            20 => self.menu_edges = !self.menu_edges,
            21 => self.original_neighbour = !self.original_neighbour,
            22 => self.graphics_api = step(self.graphics_api, GRAPHICS_APIS.len()),
            23 => self.light_shafts = !self.light_shafts,
            24 => self.water_reflections = !self.water_reflections,
            25 => self.coop_revive = !self.coop_revive,
            26 => self.coop_split = step(self.coop_split, SPLITS.len()),
            27 => self.coop_partner = !self.coop_partner,
            UPNP_ROW => self.upnp = !self.upnp,
            // Left and Right step the port by one (1024 to 65535).
            PORT_ROW => self.net_port = (self.net_port as i32 + delta).clamp(1024, 65535) as u16,
            TOGETHER_ROW => self.coop_together = !self.coop_together,
            DOORS_ROW => self.coop_doors_alone = !self.coop_doors_alone,
            TAGS_ROW => self.coop_tags = !self.coop_tags,
            ATTACK_TURN_ROW => self.steady_turn = !self.steady_turn,
            r if COLOUR_ROWS.contains(&r) => {
                let k = r - COLOUR_ROWS[0];
                self.coop_colours[k] = step(self.coop_colours[k] as usize, crate::hero_colour::COLOURS.len()) as u8;
            }
            PLAYERS_ROW => self.coop_players = 2 + step(self.coop_players.clamp(2, 4) - 2, 3),
            r if PAD_ROWS.contains(&r) => {
                let k = r - PAD_ROWS[0];
                self.coop_pads[k] = step(self.coop_pads[k] as usize, PAD_CHOICES as usize + 1) as u8;
            }
            _ => {}
        }
    }
}

/// A controller choice as shown: Auto, or Controller k.
pub fn pad_name(n: u8) -> String {
    if n == 0 { "Auto".into() } else { format!("Controller {n}") }
}

/// The frame rate limit: the frame waits out the rest of its time (the last millisecond by
/// spinning, as a sleep can overrun). Runs last in the frame; the game's ticks are timed apart
/// from it.
pub fn limit_frame(settings: Res<DisplaySettings>, mut windows: Query<&mut Window, With<PrimaryWindow>>, mut next: Local<Option<std::time::Instant>>) {
    let limit = FPS_LIMITS[settings.fps];
    // The limit is this timer alone, vsync off: measured exact at 30, 60 and 120, where vsync held
    // this laptop at about 30 whatever the limit (with its lid closed; not tested open).
    for mut w in &mut windows {
        if w.present_mode != bevy::window::PresentMode::AutoNoVsync {
            w.present_mode = bevy::window::PresentMode::AutoNoVsync;
        }
    }
    if limit == 0 || std::env::var_os("NEOPETS_PERF_NOVSYNC").is_some() {
        *next = None;
        return;
    }
    let period = std::time::Duration::from_secs_f64(1.0 / limit as f64);
    let now = std::time::Instant::now();
    let due = next.filter(|&d| d > now);
    if let Some(d) = due {
        if let Some(rest) = d.checked_duration_since(now + std::time::Duration::from_millis(1)) {
            std::thread::sleep(rest);
        }
        while std::time::Instant::now() < d {
            std::hint::spin_loop();
        }
    }
    *next = Some(due.unwrap_or(now) + period);
}

/// The game viewport inside a window of size (w, h): the largest rectangle of the aspect ratio,
/// centred (x, y, w, h), in the window's units.
pub fn viewport(w: f32, h: f32, aspect: Option<f32>) -> (f32, f32, f32, f32) {
    match aspect {
        None => (0.0, 0.0, w, h),
        Some(a) if w / h > a => {
            let vw = (h * a).round();
            (((w - vw) / 2.0).floor(), 0.0, vw, h)
        }
        Some(a) => {
            let vh = (w / a).round();
            (0.0, ((h - vh) / 2.0).floor(), w, vh)
        }
    }
}

/// The 3D camera's offscreen image and the UI node that shows it.
#[derive(Resource, Default)]
pub struct SceneTarget {
    pub(crate) image: Option<Handle<Image>>,
    pub(crate) size: (u32, u32),
    smooth: bool,
}

#[derive(Component)]
pub struct SceneView;

/// The window camera, which draws the scene image and the UI.
#[derive(Component)]
pub struct WindowCamera;

/// Spawn the window camera (the default UI camera) and the scene view node behind every UI.
pub fn setup(mut commands: Commands, mut grades: ResMut<Assets<crate::graphics::SceneGrade>>) {
    commands.spawn((
        Camera2d,
        Camera { order: 2, clear_color: ClearColorConfig::Custom(Color::BLACK), ..default() },
        bevy::core_pipeline::tonemapping::Tonemapping::None,
        IsDefaultUiCamera,
        WindowCamera,
    ));
    // The 3D picture, drawn with the colour grade (graphics.rs).
    let grade = grades.add(crate::graphics::SceneGrade { grade: Default::default(), image: Handle::default() });
    commands.spawn((Node { position_type: PositionType::Absolute, ..default() }, bevy::ui_render::prelude::MaterialNode(grade), GlobalZIndex(i32::MIN + 1), SceneView));
}

/// The resolutions on offer: the primary monitor's video mode sizes (largest first, at least
/// 640 x 480), else a fixed list.
pub fn sizes(monitors: &Query<&bevy::window::Monitor, With<bevy::window::PrimaryMonitor>>) -> Vec<(u32, u32)> {
    let mut v: Vec<(u32, u32)> = monitors
        .iter()
        .flat_map(|m| m.video_modes.iter().map(|vm| (vm.physical_size.x, vm.physical_size.y)))
        .filter(|&(w, h)| w >= 640 && h >= 480)
        .collect();
    v.sort_by(|a, b| (b.0 * b.1).cmp(&(a.0 * a.1)).then(b.0.cmp(&a.0)));
    v.dedup();
    if v.is_empty() {
        v = FALLBACK_SIZES.to_vec();
    }
    v
}

/// Apply the Display page's changes from the menu flow, and save them.
pub fn take_requests(
    menu: Option<ResMut<crate::menu_view::Menu>>,
    mut settings: ResMut<DisplaySettings>,
    monitors: Query<&bevy::window::Monitor, With<bevy::window::PrimaryMonitor>>,
    mut net: ResMut<crate::net::Net>,
    mut address: ResMut<crate::net::JoinAddress>,
    mut coop: ResMut<crate::coop::Coop>,
) {
    let _span = crate::perf::span("take_requests");
    let Some(mut menu) = menu else { return };
    // The Multiplayer page shows a controller row per player, and a game row per game found.
    let players = settings.coop_players.clamp(2, 4);
    if menu.flow.display_players != players {
        menu.flow.display_players = players;
    }
    let found = net.finder.found.len();
    if menu.flow.display_found != found {
        menu.flow.display_found = found;
    }
    if menu.flow.display_requests.is_empty() {
        return;
    }
    let sizes = sizes(&monitors);
    for (row, delta) in std::mem::take(&mut menu.flow.display_requests) {
        // The online rows act on the session (net.rs): host, join, or leave the one running.
        let leave = net.role != crate::net::Role::Off;
        match row {
            // A dimmed button does nothing: hosting while joined, or joining while hosting.
            HOST_ROW | JOIN_ROW if (row == HOST_ROW && net.role == crate::net::Role::Join) || (row == JOIN_ROW && net.role == crate::net::Role::Host) => {}
            HOST_ROW => net.request = Some(if leave && net.role == crate::net::Role::Host { crate::net::Request::Leave } else { crate::net::Request::Host }),
            JOIN_ROW => {
                net.request = Some(if leave && net.role == crate::net::Role::Join { crate::net::Request::Leave } else { crate::net::Request::Join(address.0.clone()) });
                let _ = std::fs::write("private/join.txt", &address.0);
            }
            ADDRESS_ROW => {}
            PASSWORD_ROW => {}
            // A game the finder found: its address goes in the Join address box, and it is joined.
            r if GAME_ROWS.contains(&r) => {
                let game = net.finder.found.get(r - GAME_ROWS[0]).filter(|g| g.joinable()).map(|g| g.join_text());
                if let (Some(a), crate::net::Role::Off) = (game, net.role) {
                    address.0 = a.clone();
                    let _ = std::fs::write("private/join.txt", &a);
                    net.request = Some(crate::net::Request::Join(a));
                }
            }
            // Local co-op: Cross (or Left and Right) starts it, or stops it while on.
            COOP_ROW => coop.request = Some(!coop.wanted),
            REMOVE_ROW => {
                if net.role == crate::net::Role::Host && net.any_peer() {
                    net.request = Some(crate::net::Request::Remove);
                }
            }
            _ => settings.change(row, delta, &sizes),
        }
    }
    settings.save();
}

/// How long the Multiplayer page waits for a controller's button, in seconds, and how long it
/// passes over buttons at first (the press that started the wait).
const LISTEN_SECONDS: f32 = 10.0;
const LISTEN_GRACE: f32 = 0.25;

/// The Multiplayer page waits for a button on a controller (Cross or a click on a Player N
/// controller row): the first controller pressed is given to that player. A player who had that
/// controller chosen takes the waiting player's old choice. Esc or Backspace on the keyboard,
/// a click, or ten seconds cancel.
pub fn listen_pads(menu: Option<ResMut<crate::menu_view::Menu>>, mut settings: ResMut<DisplaySettings>, pads: Query<(Entity, &Gamepad)>, keys: Res<ButtonInput<KeyCode>>, time: Res<Time>, mut since: Local<f32>) {
    let Some(mut menu) = menu else { return };
    let flow = &mut menu.flow;
    let Some(k) = flow.display_listen else {
        *since = 0.0;
        return;
    };
    if flow.current != crate::menus::DISPLAY || flow.display_page != 2 || k >= PAD_ROWS.len() {
        flow.display_listen = None;
        return;
    }
    *since += time.delta_secs();
    let cancel = *since > LISTEN_SECONDS || keys.just_pressed(KeyCode::Escape) || keys.just_pressed(KeyCode::Backspace);
    let mut connected: Vec<(Entity, &Gamepad)> = pads.iter().collect();
    connected.sort_by_key(|(e, _)| *e);
    let pressed = if *since < LISTEN_GRACE { None } else { connected.iter().position(|(_, g)| g.get_just_pressed().next().is_some()) };
    if let Some(n) = pressed {
        let choice = n as u8 + 1;
        settings.coop_pads = give_pad(settings.coop_pads, k, choice);
        settings.save();
        flow.sounds.push(0x9A);
    }
    if cancel || pressed.is_some() {
        // The press that ended the wait does not also act on the page.
        flow.display_listen = None;
        flow.pass = 0;
        *since = 0.0;
    }
}

/// Give controller `choice` (1 to 4) to player `k`; a player who had it chosen takes player `k`'s
/// old choice.
pub fn give_pad(mut pads: [u8; 4], k: usize, choice: u8) -> [u8; 4] {
    let old = pads[k];
    for (j, p) in pads.iter_mut().enumerate() {
        if j != k && *p == choice {
            *p = old;
        }
    }
    pads[k] = choice;
    pads
}

/// Apply the window mode and size when the settings change.
pub fn apply_window(
    mut settings: ResMut<DisplaySettings>,
    mut windows: Query<&mut Window, With<PrimaryWindow>>,
    monitors: Query<&bevy::window::Monitor, With<bevy::window::PrimaryMonitor>>,
    mut last: Local<Option<DisplaySettings>>,
) {
    let _span = crate::perf::span("apply_window");
    let Ok(mut window) = windows.single_mut() else { return };
    // The first run keeps the window as built unless a settings file asked for something; the
    // size then starts as the window's own size in physical pixels (a scaled display makes the
    // built 1000 x 600 larger).
    if last.is_none() {
        let default = DisplaySettings::default();
        // NEOPETS_WINDOW (diagnostic) keeps the window as built over the settings file.
        if settings.mode == default.mode && settings.size == default.size || std::env::var_os("NEOPETS_WINDOW").is_some() {
            settings.mode = 0;
            settings.size = (window.physical_width(), window.physical_height());
            *last = Some(settings.clone());
            return;
        }
    }
    let changed = last.as_ref().is_none_or(|l| l.mode != settings.mode || l.size != settings.size);
    *last = Some(settings.clone());
    if !changed {
        return;
    }
    let (w, h) = settings.size;
    window.mode = match settings.mode {
        1 => WindowMode::BorderlessFullscreen(MonitorSelection::Current),
        2 => {
            // Exclusive fullscreen needs one of the monitor's own modes: the highest refresh
            // rate at this size, else the current mode.
            let mode = monitors
                .iter()
                .flat_map(|m| m.video_modes.iter())
                .filter(|vm| vm.physical_size == UVec2::new(w, h))
                .max_by_key(|vm| (vm.refresh_rate_millihertz, vm.bit_depth))
                .copied();
            WindowMode::Fullscreen(MonitorSelection::Current, mode.map_or(VideoModeSelection::Current, VideoModeSelection::Specific))
        }
        _ => WindowMode::Windowed,
    };
    if settings.mode != 1 {
        window.resolution.set_physical_resolution(w, h);
    }
}

/// Each frame: fit the viewport to the window and the aspect ratio, keep the 3D camera's
/// image at the viewport times the render scale, and place the scene view over the viewport.
#[allow(clippy::too_many_arguments)]
pub fn sync_scene(
    mut commands: Commands,
    settings: Res<DisplaySettings>,
    windows: Query<&Window, With<PrimaryWindow>>,
    cameras: Query<Entity, (With<Camera3d>, Without<crate::water_reflect::MirrorCam>)>,
    mut views: Query<(&mut Node, &bevy::ui_render::prelude::MaterialNode<crate::graphics::SceneGrade>), With<SceneView>>,
    mut grades: ResMut<Assets<crate::graphics::SceneGrade>>,
    mut images: ResMut<Assets<Image>>,
    mut target: ResMut<SceneTarget>,
) {
    let _span = crate::perf::span("sync_scene");
    let Ok(window) = windows.single() else { return };
    let (pw, ph) = (window.physical_width() as f32, window.physical_height() as f32);
    if pw < 1.0 || ph < 1.0 {
        return;
    }
    let (vx, vy, vw, vh) = viewport(pw, ph, ASPECTS[settings.aspect].1);
    let s = SCALES[settings.scale] as f32 / 100.0;
    let size = (((vw * s).round() as u32).clamp(16, 8192), ((vh * s).round() as u32).clamp(16, 8192));
    if target.image.is_none() || target.size != size || target.smooth != settings.smooth {
        let mut image = Image::new_target_texture(size.0, size.1, TextureFormat::Rgba8UnormSrgb, None);
        image.sampler = if settings.smooth { ImageSampler::linear() } else { ImageSampler::nearest() };
        let handle = images.add(image);
        if let Some(old) = target.image.replace(handle.clone()) {
            images.remove(&old);
        }
        target.size = size;
        target.smooth = settings.smooth;
        for e in &cameras {
            commands.entity(e).insert(RenderTarget::from(handle.clone()));
        }
        for (_, m) in &views {
            if let Some(mut g) = grades.get_mut(&m.0) {
                g.image = handle.clone();
            }
        }
    }
    // The node is in logical pixels.
    let f = window.scale_factor();
    for (mut node, _) in &mut views {
        let want = (px(vx / f), px(vy / f), px(vw / f), px(vh / f));
        if (node.left, node.top, node.width, node.height) != want {
            (node.left, node.top, node.width, node.height) = want;
        }
    }
}

/// The Display page's window (this build), in the bindings page's look: a row per setting with
/// its value between < and > buttons, a Back row, and a line about the focused setting. The
/// front end's Display page (menus.rs) keeps the cursor and the changes; the window shows them
/// and takes the mouse.
#[derive(Component)]
pub struct DispWindow;
#[derive(Component)]
pub struct DispTitle;
#[derive(Component)]
pub struct DispRow(usize);
#[derive(Component)]
pub struct DispLabel(usize);
#[derive(Component)]
pub struct DispValue(usize);
#[derive(Component)]
pub struct DispStep(usize, i32);
#[derive(Component)]
pub struct DispHelp;
/// A tabbed page's tab bar, its tabs and their names, and a heading over a group of rows.
#[derive(Component)]
pub struct DispTabs;
#[derive(Component)]
pub struct DispTab(u8);
#[derive(Component)]
pub struct DispTabText(u8);
/// An action button's text (its colour follows the button's state).
#[derive(Component)]
pub struct DispActionText;
/// The keys line under the title.
#[derive(Component)]
pub struct DispKeys;
#[derive(Component)]
pub struct DispHeading(usize);
/// An action row's button.
#[derive(Component)]
pub struct DispAction(usize);
/// A typing row's text box.
#[derive(Component)]
pub struct DispInput(usize);

const PANEL: Color = Color::srgb(0.05, 0.06, 0.10);
const ROW: Color = Color::srgba(1.0, 1.0, 1.0, 0.04);
const ROW_HOVER: Color = Color::srgba(1.0, 1.0, 1.0, 0.10);
const ROW_CURSOR: Color = Color::srgba(0.95, 0.75, 0.25, 0.22);
const ARROW: Color = Color::srgba(1.0, 1.0, 1.0, 0.08);
const ARROW_HOVER: Color = Color::srgba(0.95, 0.75, 0.25, 0.65);
const GOLD: Color = Color::srgb(0.98, 0.80, 0.35);
const TEXT: Color = Color::srgb(0.92, 0.93, 0.96);
const DIM: Color = Color::srgb(0.60, 0.63, 0.70);
/// Headings over groups of rows: a muted gold, under the title's.
const ACCENT: Color = Color::srgb(0.80, 0.66, 0.40);
/// A tab: not chosen, chosen, and chosen with the cursor on the tab bar.
const TAB: Color = Color::srgba(1.0, 1.0, 1.0, 0.05);
const TAB_ON: Color = Color::srgba(0.95, 0.75, 0.25, 0.22);
const TAB_ON_HOT: Color = Color::srgba(0.95, 0.75, 0.25, 0.45);
/// An action button (square, in the menu's gold): its fill, edge and text when ready; selected or
/// hovered; when it stops something (a muted red); and when not available now (grey).
const ACTION: (Color, Color, Color) = (Color::srgba(0.95, 0.75, 0.25, 0.12), Color::srgba(0.98, 0.80, 0.35, 0.55), GOLD);
const ACTION_HOT: (Color, Color, Color) = (Color::srgba(0.95, 0.75, 0.25, 0.38), GOLD, Color::srgb(1.0, 0.96, 0.85));
const ACTION_STOP: (Color, Color, Color) = (Color::srgba(0.70, 0.25, 0.20, 0.22), Color::srgba(0.85, 0.42, 0.35, 0.60), Color::srgb(0.95, 0.68, 0.60));
const ACTION_STOP_HOT: (Color, Color, Color) = (Color::srgba(0.70, 0.25, 0.20, 0.50), Color::srgb(0.92, 0.50, 0.42), Color::srgb(1.0, 0.90, 0.86));
const ACTION_OFF: (Color, Color, Color) = (Color::srgba(1.0, 1.0, 1.0, 0.03), Color::srgba(1.0, 1.0, 1.0, 0.14), Color::srgb(0.45, 0.47, 0.52));
/// A text box: its field and edge, and the edge while it is being typed into or hovered.
const INPUT: Color = Color::srgb(0.03, 0.03, 0.04);
const INPUT_EDGE: Color = Color::srgba(1.0, 1.0, 1.0, 0.30);
const INPUT_HOT: Color = Color::srgb(0.98, 0.80, 0.35);

/// What each setting does, shown under the rows.
const HELP: [&str; ROWS + 1] = [
    "Windowed, borderless or full screen.",
    "The window size, or the screen mode in full screen.",
    "The shape of the game picture; bars fill the rest of the window.",
    "How sharp the 3D world is drawn. Higher looks better and costs speed.",
    "How the world picture is scaled to the window: Sharp keeps pixels crisp, Smooth blends them.",
    "Sets the costly options on every tab at once. Low runs best on slower computers; High adds shadows from everything. Changing one of them shows Custom.",
    "Smooths jagged edges. SMAA also smooths leaves and fences; MSAA turns to SMAA while ambient occlusion is on.",
    "Keeps ground and walls crisp at a distance and at an angle, without shimmer.",
    "A soft glow around bright light, magic and sky.",
    "Vivid and Rich lift colour and contrast a little; Original is the game's own colour.",
    "Soft shade where walls meet floors and in corners.",
    "Full keeps trees, bushes and props in 3D at any distance; Original turns far trees into sprites, fades distant plants out and hides props past their draw distance, as the PS2 game.",
    "Original is the game's own faint, blocky hero shadow; Detailed casts every character's real shape from the area's main light.",
    "Shadows cast by everything, trees and buildings too, from the area's main light. Not in the original game.",
    "How sharp the shadows from the area's main light are (World shadows and Detailed character shadows). Lower is faster.",
    "How far from the camera those shadows reach. Shorter is faster; shadows end at this distance.",
    "The most frames drawn each second; Uncapped draws as fast as it can. Exclusive full screen may tear. The game's speed does not change.",
    "Off passes over every movie cutscene at once, as if skipped (for testing). Not in the original game.",
    "The button pictures in boxes and menus: the game's PlayStation ones, Xbox A, B, X and Y, your keyboard keys, or Auto (Xbox after a gamepad press, keys after a key or click).",
    "The D-pad picture on the HUD during play. Off hides it; the D-pad still works.",
    "How menus fill a wide window. Extended carries each picture's edge colours out to the sides; Original keeps the 512 x 448 frame with bars.",
    "Full always draws the whole of the next area beside you; Original hides some of its pieces from parts of this area, as the PS2 game does.",
    "Vulkan runs fastest on most computers. DX12 can be faster on a laptop whose screen is wired to its built-in graphics (for example a gaming laptop in its hybrid or battery GPU mode). Takes effect after a restart.",
    "Beams of sunlight through trees and a soft haze toward the sun. Shafts need World shadows on. Not in the original game.",
    "Flat water mirrors the scenery and sky around it, more at low angles. Not in the original game.",
    "Revive: a fallen player gets up when the other stands beside them. Game over: any fall ends the game.",
    "How two players share the screen: one above the other, or side by side. Three or four players always have quarters.",
    "Partner: player 2 is Roberta when the story has her waiting. Second Tor: always Tor. Players 3 and 4 are always Tor.",
    "Play with a friend on your home network. Start a game first, then host; your friend joins with the address shown.",
    "The host's address (shown on their Host row). Type it with the number keys; Backspace deletes.",
    "Join the host's game. Each player needs their own copy of the game disc.",
    "Hosting: sends the last player who joined back to their title screen. You keep hosting, and they can join again.",
    "Optional. The host types one before hosting; a friend must type the same before joining. Letters and digits; Backspace deletes.",
    "On: while you host, your router is asked to let a friend on the internet in (UPnP). It closes when you stop hosting. Some routers have UPnP off.",
    "The UDP port to host and join on (47817 by default; both computers should match). Type it, or step it with Left and Right.",
    "On: other players are brought back beside player 1 when more than 60 units away. Off: everyone roams the area; doors and loads still take everyone.",
    "Both: a door anyone takes moves everyone. Only you: a joiner can go through a door alone into a world of its own, and comes back when they walk into the host's world.",
    "Starts (or stops) play on this computer: players 2 to 4 join beside player 1, each with a part of the screen. Not while online.",
    "Players on this computer: 2, 3 or 4. Three or four share the screen in quarters (with three, player 1 has the top half).",
    "Player 1's controller: Cross or a click, then press a button on it. Auto: the keyboard and every controller no other player has. Player 1 always has the keyboard.",
    "Player 2's controller: Cross or a click, then press a button on it. Left and right step through Auto and Controllers 1 to 4.",
    "Player 3's controller: Cross or a click, then press a button on it. Auto: the next free controller.",
    "Player 4's controller: Cross or a click, then press a button on it. Auto: the next free controller.",
    "On: a coloured tag (P1 to P4) over each other player, so you can tell who is who. Not in the original game.",
    "Steady: an attack turns your hero toward the enemy's body. Original: toward the game's aim point, which on some enemies (the Swamp Tentacle) swings with them and can turn a swing away.",
    "Player 1's colour in multiplayer: Tor's fur or Roberta's skin (Original keeps them). The host's choices count online.",
    "Player 2's colour in multiplayer: Tor's yellow fur or Roberta's blue skin in another colour. Original keeps them.",
    "Player 3's colour in multiplayer: Tor's yellow fur or Roberta's blue skin in another colour. Original keeps them.",
    "Player 4's colour in multiplayer: Tor's yellow fur or Roberta's blue skin in another colour. Original keeps them.",
    "Games hosting on this home network, found by themselves while this tab shows. Press or click a game to join it.",
    "Games hosting on this home network. Press or click a game to join it.",
    "Games hosting on this home network. Press or click a game to join it.",
    "Games hosting on this home network. Press or click a game to join it.",
    "Back to Options.",
];

fn font(size: f32) -> TextFont {
    TextFont { font_size: bevy::text::FontSize::Px(size), ..default() }
}

pub fn spawn_window(mut commands: Commands) {
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: percent(50),
                top: px(16),
                margin: UiRect::left(px(-300)),
                width: px(600),
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(px(14)),
                row_gap: px(3),
                display: Display::None,
                ..default()
            },
            crate::ui_fit::FitToWindow::Centre,
            BackgroundColor(PANEL),
            GlobalZIndex(5003),
            DispWindow,
        ))
        .with_children(|ui| {
            ui.spawn((Text::new("Display"), font(22.0), TextColor(GOLD), DispTitle));
            ui.spawn((Text::default(), font(12.0), TextColor(DIM), Node { margin: UiRect::bottom(px(8)), ..default() }, DispKeys));
            // A tabbed page's tabs (L1 and R1, or Left and Right on the bar, or a click); the
            // chosen tab is gold with a gold line under it.
            ui.spawn((Node { flex_direction: FlexDirection::Row, column_gap: px(4), margin: UiRect::bottom(px(2)), display: Display::None, ..default() }, DispTabs)).with_children(|bar| {
                for t in 0..MAX_TABS as u8 {
                    bar.spawn((Button, Node { flex_grow: 1.0, justify_content: JustifyContent::Center, padding: UiRect::axes(px(8), px(5)), border: UiRect::bottom(px(2)), ..default() }, BackgroundColor(TAB), BorderColor::all(Color::NONE), DispTab(t))).with_children(|b| {
                        b.spawn((Text::default(), font(15.0), TextColor(TEXT), TextLayout::no_wrap(), DispTabText(t)));
                    });
                }
            });
            let order: Vec<usize> = (0..3u8).flat_map(page_rows).chain([ROWS]).collect();
            for k in order {
                let back = k == ROWS;
                if let Some(&(_, text)) = HEADINGS.iter().find(|h| h.0 == k) {
                    ui.spawn((Text::new(text.to_uppercase()), font(12.0), TextColor(ACCENT), Node { margin: UiRect { top: px(8), bottom: px(1), left: px(4), ..default() }, display: Display::None, ..default() }, DispHeading(k)));
                }
                ui.spawn((
                    Button,
                    Node { flex_direction: FlexDirection::Row, justify_content: JustifyContent::SpaceBetween, align_items: AlignItems::Center, padding: UiRect::axes(px(10), px(3)), min_height: px(25), margin: UiRect::top(px(if back { 8.0 } else { 0.0 })), ..default() },
                    BackgroundColor(ROW),
                    DispRow(k),
                ))
                .with_children(|row| {
                    row.spawn((Text::new(if back { "Back" } else { "" }), font(15.0), TextColor(TEXT), TextLayout::no_wrap(), DispLabel(k)));
                    if back {
                        return;
                    }
                    if TEXT_ROWS.contains(&k) {
                        row.spawn((
                            Button,
                            Node { width: px(290), height: px(28), justify_content: JustifyContent::FlexStart, align_items: AlignItems::Center, padding: UiRect::horizontal(px(10)), border: UiRect::all(px(1)), ..default() },
                            BackgroundColor(INPUT),
                            BorderColor::all(INPUT_EDGE),
                            DispInput(k),
                        ))
                        .with_children(|b| {
                            b.spawn((Text::default(), font(15.0), TextColor(TEXT), TextLayout::no_wrap(), DispValue(k)));
                        });
                        return;
                    }
                    if ACTION_ROWS.contains(&k) {
                        row.spawn((
                            Button,
                            Node { width: px(290), height: px(28), justify_content: JustifyContent::Center, align_items: AlignItems::Center, border: UiRect::all(px(1)), ..default() },
                            BackgroundColor(ACTION.0),
                            BorderColor::all(ACTION.1),
                            DispAction(k),
                        ))
                        .with_children(|b| {
                            b.spawn((Text::default(), font(15.0), TextColor(ACTION.2), TextLayout::no_wrap(), DispValue(k), DispActionText));
                        });
                        return;
                    }
                    row.spawn(Node { flex_direction: FlexDirection::Row, align_items: AlignItems::Center, column_gap: px(6), ..default() }).with_children(|r| {
                        let arrow = |r: &mut ChildSpawnerCommands, d: i32| {
                            r.spawn((Button, Node { width: px(24), height: px(22), justify_content: JustifyContent::Center, align_items: AlignItems::Center, ..default() }, BackgroundColor(ARROW), DispStep(k, d))).with_children(|b| {
                                b.spawn((Text::new(if d < 0 { "<" } else { ">" }), font(15.0), TextColor(TEXT)));
                            });
                        };
                        arrow(r, -1);
                        r.spawn(Node { width: px(230), justify_content: JustifyContent::Center, ..default() }).with_children(|v| {
                            v.spawn((Text::default(), font(15.0), TextColor(GOLD), TextLayout::no_wrap(), DispValue(k)));
                        });
                        arrow(r, 1);
                    });
                });
            }
            ui.spawn((Text::default(), font(13.0), TextColor(TEXT), Node { margin: UiRect::top(px(10)), min_height: px(54), ..default() }, DispHelp));
        });
}

/// The mouse on the Display window: hovering a row focuses it, a click on < or > changes it (a
/// click on the row steps it forward), and Back returns to Options.
pub fn window_mouse(menu: Option<ResMut<crate::menu_view::Menu>>, rows: Query<(&Interaction, &DispRow), Changed<Interaction>>, steps: Query<(&Interaction, &DispStep), Changed<Interaction>>, (tabs, actions, inputs): (Query<(&Interaction, &DispTab), Changed<Interaction>>, Query<(&Interaction, &DispAction), Changed<Interaction>>, Query<(&Interaction, &DispInput), Changed<Interaction>>)) {
    let _span = crate::perf::span("window_mouse");
    let Some(mut menu) = menu else { return };
    let flow = &mut menu.flow;
    if flow.current != crate::menus::DISPLAY {
        return;
    }
    // Waiting for a controller's button: a click anywhere cancels.
    if flow.display_listen.is_some() {
        if rows.iter().any(|(i, _)| *i == Interaction::Pressed) || steps.iter().any(|(i, _)| *i == Interaction::Pressed) || tabs.iter().any(|(i, _)| *i == Interaction::Pressed) || actions.iter().any(|(i, _)| *i == Interaction::Pressed) || inputs.iter().any(|(i, _)| *i == Interaction::Pressed) {
            flow.display_listen = None;
            flow.pass = 0;
        }
        return;
    }
    let mut stepped_action = false;
    for (i, t) in &tabs {
        if *i == Interaction::Pressed {
            flow.display_tab = t.0;
            flow.display_focus = TAB_FOCUS;
        }
    }
    for (i, a) in &actions {
        if *i == Interaction::Pressed {
            flow.display_focus = a.0;
            flow.display_requests.push((a.0, 1));
            stepped_action = true;
        }
    }
    for (i, t) in &inputs {
        if *i == Interaction::Pressed {
            flow.display_focus = t.0;
            stepped_action = true;
        }
    }
    let mut stepped = stepped_action;
    for (i, s) in &steps {
        if *i == Interaction::Pressed {
            flow.display_focus = s.0;
            flow.display_requests.push((s.0, s.1));
            stepped = true;
        }
    }
    for (i, r) in &rows {
        match i {
            Interaction::Hovered => flow.display_focus = r.0,
            Interaction::Pressed if !stepped => {
                flow.display_focus = r.0;
                if TEXT_ROWS.contains(&r.0) {
                    // A typing row is clicked to type into it (before, a click stepped the port).
                } else if PAD_ROWS.contains(&r.0) {
                    // A controller row waits for a button on the controller to give that player.
                    flow.display_listen = Some(r.0 - PAD_ROWS[0]);
                } else if r.0 == ROWS {
                    flow.display_back = true;
                } else {
                    flow.display_requests.push((r.0, 1));
                }
            }
            _ => {}
        }
    }
}

/// Show the Display window while the front end is on the Display page.
#[allow(clippy::type_complexity)]
pub fn show_window(
    menu: Option<Res<crate::menu_view::Menu>>,
    settings: Res<DisplaySettings>,
    mut windows: Query<&mut Node, With<DispWindow>>,
    mut rows: Query<(&Interaction, &DispRow, &mut BackgroundColor, &mut Node), (Without<DispStep>, Without<DispWindow>)>,
    mut steps: Query<(&Interaction, &mut BackgroundColor), (With<DispStep>, Without<DispRow>)>,
    mut labels: Query<(&DispLabel, &mut Text, &mut TextColor), (Without<DispValue>, Without<DispHelp>, Without<DispTitle>)>,
    mut values: Query<(&DispValue, &mut Text), (Without<DispLabel>, Without<DispHelp>, Without<DispTitle>)>,
    mut help: Query<&mut Text, (With<DispHelp>, Without<DispLabel>, Without<DispValue>, Without<DispTitle>)>,
    mut titles: Query<&mut Text, (With<DispTitle>, Without<DispHelp>, Without<DispLabel>, Without<DispValue>)>,
    (net, address, password): (Res<crate::net::Net>, Res<crate::net::JoinAddress>, Res<crate::net::NetPassword>),
    (coop, play, gamepads): (Res<crate::coop::Coop>, Option<Res<crate::PlayState>>, Query<Entity, With<Gamepad>>),
    mut tab_bar: Query<&mut Node, (With<DispTabs>, Without<DispWindow>, Without<DispRow>, Without<DispHeading>)>,
    mut tab_chips: Query<(&DispTab, &Interaction, &mut BackgroundColor, &mut BorderColor), (Without<DispRow>, Without<DispStep>, Without<DispAction>, Without<DispInput>)>,
    mut headings: Query<(&DispHeading, &mut Node), (Without<DispWindow>, Without<DispRow>, Without<DispTabs>)>,
    mut actions: Query<(&DispAction, &Interaction, &mut BackgroundColor, &mut BorderColor), (Without<DispRow>, Without<DispStep>, Without<DispTab>, Without<DispInput>)>,
    (mut boxes, mut action_text, mut tab_text, mut keys): (
        Query<(&DispInput, &Interaction, &mut BorderColor), (Without<DispAction>, Without<DispTab>)>,
        Query<(&DispValue, &mut TextColor), (With<DispActionText>, Without<DispLabel>, Without<DispTabText>)>,
        Query<(&DispTabText, &mut Text, &mut TextColor), (Without<DispLabel>, Without<DispValue>, Without<DispHelp>, Without<DispTitle>, Without<DispKeys>)>,
        Query<&mut Text, (With<DispKeys>, Without<DispLabel>, Without<DispValue>, Without<DispHelp>, Without<DispTitle>)>,
    ),
) {
    let _span = crate::perf::span("show_window");
    let flow = menu.as_ref().map(|m| &m.flow);
    let open = flow.is_some_and(|f| f.current == crate::menus::DISPLAY);
    for mut n in &mut windows {
        let d = if open { Display::Flex } else { Display::None };
        if n.display != d {
            n.display = d;
        }
    }
    let Some(flow) = flow.filter(|_| open) else { return };
    let focus = if flow.display_focus == TAB_FOCUS { TAB_FOCUS } else { flow.display_focus.min(ROWS) };
    let mut shown = settings.rows();
    let mut action_colours: Vec<(usize, Color)> = Vec::new();
    {
        use crate::net::Role;
        shown[HOST_ROW].1 = match net.role {
            Role::Host => "Stop hosting".into(),
            Role::Join => "Leave first to host".into(),
            _ => "Host".into(),
        };
        shown[ADDRESS_ROW].1 = match (address.0.is_empty(), focus == ADDRESS_ROW) {
            (true, false) => "Click to type an address".into(),
            (_, f) => format!("{}{}", address.0, if f { "_" } else { "" }),
        };
        shown[JOIN_ROW].1 = match net.role {
            Role::Join if net.following() => "Leave".into(),
            Role::Join => "Joining... (cancel)".into(),
            Role::Host => "Stop hosting to join".into(),
            _ => "Join".into(),
        };
        shown[REMOVE_ROW].1 = if net.role == Role::Host && net.any_peer() { format!("Remove Player {}", net.peers() + 1) } else { "No one online".into() };
        if focus == PORT_ROW {
            shown[PORT_ROW].1 = format!("{}_", if settings.net_port == 0 { String::new() } else { settings.net_port.to_string() });
        }
        // Local co-op: on or off, and each controller row's controller as it is now.
        let here = play.as_ref().map_or(0, |p| if p.play.net_follow || p.play.coop_remote { 0 } else { p.play.others.len() });
        shown[COOP_ROW].1 = if net.role != Role::Off {
            "Not while online".into()
        } else if coop.wanted {
            if here == 0 { "Stop (starts in play)".into() } else { format!("Stop ({} playing)", here + 1) }
        } else {
            "Start".into()
        };
        // The action buttons: gold ready, red when it stops something, grey when not available.
        let state = |row: usize| -> u8 {
            match row {
                COOP_ROW if net.role != Role::Off => 0,
                COOP_ROW if coop.wanted => 2,
                HOST_ROW if net.role == Role::Join => 0,
                HOST_ROW if net.role == Role::Host => 2,
                JOIN_ROW if net.role == Role::Host => 0,
                JOIN_ROW if net.role == Role::Join => 2,
                REMOVE_ROW if !(net.role == Role::Host && net.any_peer()) => 0,
                REMOVE_ROW => 2,
                r if GAME_ROWS.contains(&r) => net.finder.found.get(r - GAME_ROWS[0]).is_some_and(|g| g.joinable() && net.role == Role::Off) as u8,
                _ => 1,
            }
        };
        for (t, i, mut edge) in &mut boxes {
            let want = BorderColor::all(if t.0 == focus || *i != Interaction::None { INPUT_HOT } else { INPUT_EDGE });
            if *edge != want {
                *edge = want;
            }
        }
        for (a, i, mut bg, mut edge) in &mut actions {
            let hot = a.0 == focus || *i != Interaction::None;
            let want = match (state(a.0), hot) {
                (0, _) => ACTION_OFF,
                (2, false) => ACTION_STOP,
                (2, true) => ACTION_STOP_HOT,
                (_, false) => ACTION,
                (_, true) => ACTION_HOT,
            };
            if bg.0 != want.0 {
                bg.0 = want.0;
            }
            let e = BorderColor::all(want.1);
            if *edge != e {
                *edge = e;
            }
            action_colours.push((a.0, want.2));
        }
        let mut connected: Vec<Entity> = gamepads.iter().collect();
        connected.sort();
        // While on, as many as are playing (a test switch can ask for another number).
        let count = if coop.wanted && here > 0 { here + 1 } else { settings.coop_players.clamp(2, 4) };
        let pads = crate::coop::assign(&connected, settings.coop_pads, count);
        for (k, &row) in PAD_ROWS.iter().enumerate() {
            let number = pads[k].and_then(|e| connected.iter().position(|&c| c == e)).map(|n| n + 1);
            let choice = settings.coop_pads[k];
            shown[row].1 = match (k, choice, number) {
                _ if k >= count => format!("{} (not playing)", pad_name(choice)),
                (0, 0, _) => "Auto (keyboard + rest)".into(),
                (_, 0, Some(n)) => format!("Auto (Controller {n})"),
                // A player with no controller: Cross or a click, then a button on one, gives it.
                (_, _, None) => "Press a button".into(),
                (_, c, Some(_)) => pad_name(c),
            };
        }
        // The game finder's rows: each game's address and players, and what its button does.
        for (k, &row) in GAME_ROWS.iter().enumerate() {
            shown[row] = match net.finder.found.get(k) {
                Some(g) => (
                    format!("Game at {}", g.join_text()),
                    if !g.same_build {
                        "Another version".into()
                    } else if !g.ready {
                        "Not playing yet".into()
                    } else if g.players >= g.max {
                        format!("Full ({} of {})", g.players, g.max)
                    } else {
                        format!("Join ({} of {}{})", g.players, g.max, if g.password { ", password" } else { "" })
                    },
                ),
                None if net.role != Role::Off => ("Games found".into(), "Not while online".into()),
                None => ("Games found".into(), "Looking...".into()),
            };
        }
        if let Some(k) = flow.display_listen.filter(|&k| k < PAD_ROWS.len()) {
            shown[PAD_ROWS[k]].1 = "Waiting for a button...".into();
        }
        shown[PASSWORD_ROW].1 = match (password.0.is_empty(), focus == PASSWORD_ROW) {
            (true, false) => "None (click to type one)".into(),
            (true, true) => "_".into(),
            (false, f) => "*".repeat(password.0.len()) + if f { "_" } else { "" },
        };
    }
    let page = visible_rows(flow.display_page, flow.display_tab, flow.display_players, flow.display_found);
    for (i, r, mut bg, mut node) in &mut rows {
        bg.0 = if r.0 == focus { ROW_CURSOR } else if *i == Interaction::Hovered { ROW_HOVER } else { ROW };
        let d = if page.contains(&r.0) { Display::Flex } else { Display::None };
        if node.display != d {
            node.display = d;
        }
    }
    // A tabbed page's tab bar and headings.
    let page_tabs = tabs(flow.display_page);
    let tabbed = !page_tabs.is_empty();
    for mut n in &mut tab_bar {
        let d = if tabbed { Display::Flex } else { Display::None };
        if n.display != d {
            n.display = d;
        }
    }
    for (t, i, mut bg, mut edge) in &mut tab_chips {
        let on = t.0 == flow.display_tab;
        let want = if on {
            if focus == TAB_FOCUS { TAB_ON_HOT } else { TAB_ON }
        } else if *i == Interaction::Hovered {
            ROW_HOVER
        } else {
            TAB
        };
        if bg.0 != want {
            bg.0 = want;
        }
        let e = BorderColor::all(if on { GOLD } else { Color::NONE });
        if *edge != e {
            *edge = e;
        }
    }
    for (t, mut text, mut colour) in &mut tab_text {
        let name = page_tabs.get(t.0 as usize).map_or("", |p| p.0);
        if text.0 != name {
            text.0 = name.to_string();
        }
        colour.0 = if t.0 == flow.display_tab { GOLD } else { DIM };
    }
    let keys_line = if tabbed { "Up/down pick, left/right change, L1/R1 switch tabs, Esc or B go back." } else { "Up and down pick, left and right or a click change. Esc or B go back." };
    for mut t in &mut keys {
        if t.0 != keys_line {
            t.0 = keys_line.to_string();
        }
    }
    for (h, mut n) in &mut headings {
        let d = if tabbed && page.contains(&h.0) { Display::Flex } else { Display::None };
        if n.display != d {
            n.display = d;
        }
    }
    let title = ["Display", "Gameplay", "Multiplayer"][flow.display_page.min(2) as usize];
    for mut t in &mut titles {
        if t.0 != title {
            t.0 = title.to_string();
        }
    }
    for (i, mut bg) in &mut steps {
        bg.0 = if *i == Interaction::None { ARROW } else { ARROW_HOVER };
    }
    for (l, mut t, mut c) in &mut labels {
        let s = shown.get(l.0).map_or("Back".to_string(), |r| r.0.clone());
        if t.0 != s {
            t.0 = s;
        }
        c.0 = if l.0 == focus { GOLD } else { TEXT };
    }
    for (v, mut t) in &mut values {
        let s = shown.get(v.0).map_or(String::new(), |r| r.1.clone());
        if t.0 != s {
            t.0 = s;
        }
    }
    // An action button's text: gold when ready, light while selected, grey when not available.
    for (v, mut c) in &mut action_text {
        if let Some(&(_, want)) = action_colours.iter().find(|a| a.0 == v.0) {
            if c.0 != want {
                c.0 = want;
            }
        }
    }
    // On the Multiplayer page the session's status follows the help line.
    let listen_help = flow.display_listen.map(|k| format!("Press any button on the controller Player {} should use. Esc or a click cancels.", k + 1));
    let help_text = match (focus == TAB_FOCUS, flow.display_page) {
        _ if listen_help.is_some() => listen_help.as_deref().unwrap_or(""),
        (true, 0) => "Screen: the window, picture size and frame rate. Graphics: the preset and the picture's detail and look. Shadows & distance: shadows and how far things are drawn.",
        (true, _) => "This computer: split screen and controllers. Online: host or join. Players: colours and the rules for everyone.",
        _ => HELP[focus],
    };
    let line = if flow.display_page == 2 && !net.status.is_empty() && flow.display_tab == 1 { format!("{}\n{}", help_text, net.status_line()) } else { help_text.to_string() };
    for mut t in &mut help {
        if t.0 != line {
            t.0 = line.clone();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn viewport_fits_the_aspect() {
        assert_eq!(viewport(1920.0, 1080.0, None), (0.0, 0.0, 1920.0, 1080.0));
        // The two pages share the rows between them.
        assert_eq!(page_rows(1), [17, 18, 19, 20, 44]);
        assert_eq!(page_rows(2), [37, 38, 26, 39, 40, 41, 42, 28, 31, 33, 49, 50, 51, 52, 29, 30, 32, 34, 36, 45, 46, 47, 48, 43, 25, 35]);
        // Every Multiplayer row is in one tab; two players show two controller rows.
        let mut all: Vec<usize> = MULTI_TABS.iter().flat_map(|t| t.1.iter().copied()).collect();
        all.sort();
        let mut want = page_rows(2);
        want.sort();
        assert_eq!(all, want);
        assert_eq!(visible_rows(2, 0, 2, 0), [TAB_FOCUS, 37, 38, 26, 39, 40, ROWS]);
        assert_eq!(visible_rows(2, 0, 4, 0).len(), 9);
        // The game finder: one row while none is found, a row a game after.
        assert_eq!(visible_rows(2, 1, 2, 0).iter().filter(|r| GAME_ROWS.contains(r)).count(), 1);
        assert_eq!(visible_rows(2, 1, 2, 3).iter().filter(|r| GAME_ROWS.contains(r)).count(), 3);
        // The Display page's tabs hold every row no other page has, once each.
        let mut display = page_rows(0);
        display.sort();
        assert_eq!(display, (0..ROWS).filter(|k| !GAMEPLAY_ROWS.contains(k) && !MULTI_ROWS.contains(k) && !RETIRED_ROWS.contains(k)).collect::<Vec<_>>());
        assert_eq!(visible_rows(0, 1, 2, 0), [TAB_FOCUS, 5, 6, 7, 10, 9, 24, ROWS]);
        assert!(tabs(1).is_empty() && visible_rows(1, 0, 2, 0)[0] == 17);
        // Every heading comes before a row of a tabbed page.
        assert!(HEADINGS.iter().all(|h| page_rows(0).contains(&h.0) || page_rows(2).contains(&h.0)));
        // 4:3 in a 16:9 window: pillarbox 1440 wide.
        assert_eq!(viewport(1920.0, 1080.0, Some(4.0 / 3.0)), (240.0, 0.0, 1440.0, 1080.0));
        // 21:9 in a 16:9 window: letterbox 823 high.
        assert_eq!(viewport(1920.0, 1080.0, Some(21.0 / 9.0)), (0.0, 128.0, 1920.0, 823.0));
    }

    #[test]
    fn graphics_presets() {
        let sizes = [(1920, 1080)];
        let mut s = DisplaySettings::default();
        // The new-player options are the Medium preset.
        assert_eq!((s.preset(), s.rows()[5].1.as_str()), (Some(1), "Medium"));
        s.change(5, 1, &sizes);
        assert_eq!((s.preset(), s.aa, s.bloom, s.ao, s.world_shadows, s.shadow_quality), (Some(2), 2, 0, true, true, 3));
        s.change(5, 1, &sizes);
        assert_eq!((s.preset(), s.aa, s.bloom, s.ao, s.world_shadows, s.shadow_quality), (Some(0), 3, 0, false, false, 1));
        // Changing an option below shows Custom; Right from Custom goes to Low.
        s.change(10, 1, &sizes);
        assert_eq!(s.rows()[5].1, "Custom");
        s.change(5, 1, &sizes);
        assert_eq!(s.preset(), Some(0));
    }

    #[test]
    fn settings_round_trip_and_rows() {
        let mut s = DisplaySettings::default();
        let sizes = [(1920, 1080), (1280, 720), (1000, 600)];
        s.change(0, 1, &sizes);
        s.change(0, 1, &sizes);
        assert_eq!(s.mode, 2);
        s.change(1, 1, &sizes);
        assert_eq!(s.size, (1280, 720), "Right steps to the next smaller size");
        s.change(2, -1, &sizes);
        assert_eq!(ASPECTS[s.aspect].0, "21:9", "Left wraps");
        s.change(3, 1, &sizes);
        s.change(3, 1, &sizes);
        assert_eq!(SCALES[s.scale], 150);
        s.change(4, 1, &sizes);
        assert!(!s.smooth);
        s.change(18, 1, &sizes);
        assert!(s.buttons == 1 && s.rows()[18].1 == "Xbox");
        s.change(17, 1, &sizes);
        assert!(!s.movies && s.rows()[17].1 == "Off", "Movie cutscenes off");
        s.change(19, 1, &sizes);
        assert!(!s.dpad_hud && s.rows()[19].1 == "Off", "D-pad HUD off");
        assert!(!DisplaySettings::parse(&s.text()).dpad_hud, "the choice is saved");
        // The frame rate limit wraps 120, Uncapped, 30; the distance steps down from 150 to 100.
        assert_eq!(s.rows()[16].1, "120");
        s.change(16, 1, &sizes);
        assert_eq!(s.rows()[16].1, "Uncapped");
        s.change(16, 1, &sizes);
        assert_eq!(s.rows()[16].1, "30");
        assert_eq!(s.rows()[15].1, "150");
        s.change(15, -1, &sizes);
        assert_eq!(s.rows()[15].1, "100");
        assert_eq!(s.rows()[14].1, "Medium");
        s.change(14, 1, &sizes);
        s.change(14, 1, &sizes);
        assert_eq!((s.shadow_quality, s.rows()[14].1.as_str()), (0, "Very Low"), "wraps to Very Low");
        assert_eq!(DisplaySettings::parse(&s.text()), s);
        // Local co-op: 2 to 4 players (wrapping), each controller Auto or 1 to 4, saved.
        s.change(PLAYERS_ROW, 1, &sizes);
        s.change(PLAYERS_ROW, 1, &sizes);
        assert_eq!(s.rows()[PLAYERS_ROW].1, "4");
        s.change(PLAYERS_ROW, 1, &sizes);
        assert_eq!(s.coop_players, 2, "wraps");
        s.change(PAD_ROWS[1], -1, &sizes);
        assert_eq!(s.rows()[PAD_ROWS[1]].1, "Controller 4", "Left from Auto wraps");
        s.change(PAD_ROWS[3], 1, &sizes);
        assert_eq!(DisplaySettings::parse(&s.text()), s);
        assert_eq!(DisplaySettings::parse("coop pads = 9,x,2").coop_pads, [4, 0, 2, 0]);
        // A pressed controller goes to the waiting player; one who had it takes the old choice.
        assert_eq!(give_pad([0, 0, 0, 0], 1, 2), [0, 2, 0, 0]);
        assert_eq!(give_pad([0, 3, 2, 0], 1, 2), [0, 2, 3, 0]);
        assert_eq!(give_pad([1, 0, 0, 0], 2, 1), [0, 0, 1, 0]);
        // A settings file from before Very Low keeps its level.
        assert_eq!(SHADOW_QUALITY[DisplaySettings::parse("shadow quality = 1").shadow_quality], "Medium");
        assert_eq!(SHADOW_QUALITY[DisplaySettings::parse("shadow quality = 0").shadow_quality], "Low");
        assert_eq!(s.rows()[3].1, "150%");
        assert_eq!(DisplaySettings::parse("scale = 33\nmode = 9\n").mode, 2, "bad values clamp or keep");
        // A size off the list steps to its neighbours.
        let mut t = DisplaySettings { size: (2000, 1200), ..Default::default() };
        t.change(1, 1, &sizes);
        assert_eq!(t.size, (1920, 1080));
        let mut t = DisplaySettings { size: (1100, 650), ..Default::default() };
        t.change(1, -1, &sizes);
        assert_eq!(t.size, (1280, 720));
    }
}
