mod archive_view;
mod character;
mod controls;
mod dev_menu;
mod display;
mod perf;
mod effect_view;
mod foliage_view;
mod orbit;
mod pattern;
mod audio_view;
mod hud_view;
mod menu_view;
mod menus;
mod play;

use bevy::asset::RenderAssetUsages;
use bevy::image::ImageSampler;
use bevy::input::mouse::{AccumulatedMouseMotion, AccumulatedMouseScroll, MouseScrollUnit};
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use controls::{Browser, axis, movement};
use neopets_data::archive::ArchiveIndex;
use neopets_data::{Disc, iso_path};
use neopets_sim::{FixedStep, PlayerInput, Simulation};
use orbit::Orbit;

/// Play mode's world and hero: the new-game exterior (SPEC-0006 Default start) and Tor's first
/// outfit.
const PLAY_PACK: u32 = 909;
const PLAY_OUTFIT: u32 = 504;

/// The clean side's own save file (SPEC-0025 Summary and Clean-side mapping of the card jobs):
/// one 0xC400-byte file of four slots, kept under private/ (never committed).
const SAVE_FILE: &str = "private/saves/savegame.bin";

/// The save scan's read: the file when it exists and is exactly 0xC400 bytes.
/// SPEC-0034 test vectors 70 to 72: one node of a definition run alone with every draw at its
/// midpoint, at the origin with an identity frame.
fn check_effects(disc: &mut Disc, index: &ArchiveIndex) -> Result<(), String> {
    use neopets_sim::effect::*;
    let bytes = index.read(disc, neopets_data::effect::RESOURCE)?;
    let chain = neopets_data::resource::walk_chunks(&bytes, None)?;
    let range = neopets_data::effect::table_in_chain(&bytes, &chain.chunks).ok_or("no effect table")?;
    let table = EffectTable::parse(&bytes[range])?;
    println!("Effect table: {} definitions, {} nodes; id 101 -> {:?}, 159 -> {:?}, 1201 -> {:?}, 938 -> {:?}, 4007 -> {:?}, 162 -> {:?}", table.definitions.len(), table.nodes.len(), table.find(101), table.find(159), table.find(1201), table.find(938), table.find(4007), table.find(162));
    // SPEC-0034 4.5 / SPEC-0046: light-emitter records, the definitions using one, and the
    // point lights effect 1300 publishes after 60 ticks.
    let lit = table.definitions.iter().filter(|d| d.nodes.iter().any(|&n| table.a[table.nodes[n].a].light)).count();
    let with_material = table.nodes.iter().filter(|n| table.a[n.a].light && n.material.is_some()).count();
    let light_nodes = table.nodes.iter().filter(|n| table.a[n.a].light).count();
    println!("Light emitters: {} A records, {lit} definitions, {light_nodes} nodes ({with_material} with a material)", table.a.iter().filter(|a| a.light).count());
    for id in [1300, 2202, 4012] {
        let mut fx = Effects::new();
        let h = fx.add_object(EffectObject::new(&table, id, [0.0; 3], IDENTITY));
        fx.update(&table, h, None, &mut Midpoint);
        for _ in 0..60 {
            fx.tick(&table, &mut Midpoint);
        }
        let l = fx.lights(&table);
        let first = l.first().map(|e| format!("colour ({:.1}, {:.1}, {:.1}) strength {:.3} radius {:.3}", e.colour[0], e.colour[1], e.colour[2], e.strength, e.radius));
        println!("  effect {id} after 60 ticks: {} listed lights, first {first:?}", l.len());
    }
    for (id, node, ticks) in [(1201, 414usize, 300u32), (159, 1371, 370), (400, 446, 70)] {
        let mut t = table.clone();
        let Some(d) = t.find(id) else { continue };
        t.definitions[d].nodes = vec![node];
        let n = &t.nodes[node];
        println!("Vector: id {id} node {node} (A {}, B {}, C {})", n.a, n.b, n.c);
        if std::env::var_os("NEOPETS_EFFECT_DUMP").is_some() {
            println!("  B {:?}
  A rates {:?} thresholds {:?} size {:?}", t.b[n.b], t.a[n.a].size_rates, t.a[n.a].thresholds, t.a[n.a].size);
        }
        let mut fx = Effects::new();
        let h = fx.add_object(EffectObject::new(&t, id, [0.0; 3], IDENTITY));
        fx.update(&t, h, None, &mut Midpoint);
        let first = |fx: &Effects| fx.particles.iter().flatten().next().cloned();
        let mut bursts = Vec::new();
        let mut last = 0;
        for tick in 1..=ticks {
            let before = fx.spawners.iter().flatten().map(|s| s.bursts).sum::<u32>();
            fx.tick(&t, &mut Midpoint);
            let after = fx.spawners.iter().flatten().map(|s| s.bursts).sum::<u32>();
            if after > before && bursts.len() < 5 {
                bursts.push(tick);
            }
            if [1, 30, 31, 32, 60, 61, 240].contains(&tick) {
                if let Some(p) = first(&fx) {
                    println!("  tick {tick}: P ({:.4}, {:.4}, {:.4}) v ({:.4}, {:.4}, {:.4}) colour {:?} bytes {:?} size {:.4} half {:?} elapsed {:.4}", p.position[0], p.position[1], p.position[2], p.velocity[0], p.velocity[1], p.velocity[2], p.colour.map(|c| (c * 10000.0).round() / 10000.0), p.bytes(), p.size[0], p.half.map(|h| (h * 10000.0).round() / 10000.0), p.elapsed);
                }
            }
            if fx.live_particles() > 0 {
                last = tick;
            }
            if tick == ticks {
                println!("  bursts on ticks {bursts:?}; live {} particles, {} spawners at tick {tick}; last live tick {last}", fx.live_particles(), fx.live_spawners());
            }
        }
    }
    Ok(())
}

fn read_save(path: &str) -> Option<neopets_sim::save::SaveFile> {
    std::fs::read(path).ok().and_then(|b| neopets_sim::save::SaveFile::parse(&b).ok())
}

fn write_save(path: &str, file: &neopets_sim::save::SaveFile) -> Result<(), String> {
    if let Some(dir) = std::path::Path::new(path).parent() {
        std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    }
    std::fs::write(path, file.to_bytes()).map_err(|e| e.to_string())
}

/// The save's date text from the system clock, in UTC: the console's time-zone and
/// daylight-saving offset have no counterpart here (a deviation; the original converts its
/// real-time clock to local time).
fn save_date() -> Vec<u8> {
    let secs = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(0, |d| d.as_secs()) as i64;
    let (days, rem) = (secs.div_euclid(86_400), secs.rem_euclid(86_400));
    // Civil date from days since 1970-01-01 (proleptic Gregorian).
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = yoe + era * 400 + (month <= 2) as i64;
    neopets_sim::save::date_text(month as u32, day as u32, year as u32, (rem / 3600) as u32, (rem % 3600 / 60) as u32).into_bytes()
}

/// A 16-bit PCM WAV file image.
fn wav(pcm: &[i16], channels: u16, rate: u32) -> Vec<u8> {
    let data = pcm.len() as u32 * 2;
    let mut w = Vec::with_capacity(44 + data as usize);
    w.extend(b"RIFF");
    w.extend((36 + data).to_le_bytes());
    w.extend(b"WAVEfmt ");
    w.extend(16u32.to_le_bytes());
    w.extend(1u16.to_le_bytes());
    w.extend(channels.to_le_bytes());
    w.extend(rate.to_le_bytes());
    w.extend((rate * 2 * channels as u32).to_le_bytes());
    w.extend((2 * channels).to_le_bytes());
    w.extend(16u16.to_le_bytes());
    w.extend(b"data");
    w.extend(data.to_le_bytes());
    for v in pcm {
        w.extend(v.to_le_bytes());
    }
    w
}

/// SPEC-0025 Loading: the game rebuilt in the slot's worlds and the slot applied. Key B (the
/// exterior) becomes the current world and key A (the transition) is put in T; with neither
/// found the current world is kept. The original posts the two loads and applies the slot over
/// the live objects instead of rebuilding them (a simplification here); key C is not read.
fn load_game(archive: &mut Archive, old: &play::Play, slot: &neopets_sim::save::Slot) -> Result<play::Play, String> {
    let (keys, ids) = &old.world_keys;
    let pack_of = |k: usize| {
        let key = slot.key(k);
        if key < 0 {
            return None;
        }
        keys.get(neopets_sim::save::reverse(ids, key as u32)).copied()
    };
    let (t, e) = (pack_of(0), pack_of(1));
    let current = e.or(t).unwrap_or(old.world.pack);
    let Archive { disc, index } = archive;
    let mut play = play::Play::load(disc, index, current, PLAY_OUTFIT)?;
    if let Some(t) = t.filter(|&t| t != current && play.other.as_ref().is_none_or(|o| o.pack != t)) {
        play.load_other(disc, index, t)?;
    }
    // The hero and the characters are drawn again from the new game (its hero may be the
    // other one, and its character ids start again).
    play.generation = old.generation + 1;
    play.cheats = old.cheats;
    play.snapshot = Some(slot.clone());
    play.apply_slot(slot);
    Ok(play)
}

/// The world teardown of Quit (SPEC-0026 Quit teardown; SPEC-0035 Behavior 2, state 7): both
/// world slots, both heroes, the sound and the world state are replaced by a new game's (909
/// in E with its Init; SPEC-0006 a new game), so a later New Game starts fresh and no session
/// snapshot (pending flag) is kept. The options block belongs to the front end and stays.
/// The developer menu's warp: a new game state in world pack `pack`, whose Init runs with the
/// current story flags, keeping the session (items, Neopoints, maps and collected treasures,
/// the transition map, books, both heroes' pools, the equipment, the active hero, the options
/// and the cheats). A test tool of this build; the original has no such travel.
fn warp(disc: &mut Disc, index: &ArchiveIndex, old: &play::Play, pack: u32) -> Result<play::Play, String> {
    let flags: Vec<u16> = old.state.flags.iter().copied().collect();
    let mut play = play::Play::load_with_flags(disc, index, pack, PLAY_OUTFIT, &flags)?;
    let s = &old.state;
    play.state.items = s.items.clone();
    play.state.neopoints = s.neopoints;
    play.state.maps = s.maps.clone();
    play.state.collected = s.collected.clone();
    play.state.transition_map = s.transition_map;
    play.equipment = old.equipment.clone();
    play.books = old.books;
    if old.hero_index != play.hero_index {
        play.trace_select_hero(old.hero_index);
    }
    play.hero_max = old.hero_max;
    play.hero_hp = old.hero_hp;
    play.hero_mp_max = old.hero_mp_max;
    play.hero_mp = old.hero_mp;
    play.generation = old.generation + 1;
    play.options = old.options;
    play.cheats = old.cheats;
    Ok(play)
}

fn new_game(disc: &mut Disc, index: &ArchiveIndex, old: &play::Play) -> Result<play::Play, String> {
    let mut play = play::Play::load(disc, index, PLAY_PACK, PLAY_OUTFIT)?;
    play.generation = old.generation + 1;
    play.options = old.options;
    play.cheats = old.cheats;
    Ok(play)
}

/// Play mode was asked for, in this world pack (`--pack`, default 909).
#[derive(Resource)]
struct PlayRequest(u32, Option<[f32; 2]>, Vec<u16>, bool, Vec<u16>, Option<[f32; 3]>);

/// Diagnostic: hold the stick forward at walk tilt in play mode (`--autowalk`), so screenshots
/// can show travel without a person at the controls; `--autojump` also presses Cross every 2 s.
#[derive(Resource)]
struct AutoWalk {
    jump: bool,
}

/// The play-mode hero entity; its children are the outfit's meshes.
#[derive(Component)]
struct HeroRoot;

#[derive(Resource)]
pub(crate) struct PlayState {
    pub(crate) play: play::Play,
    skinned: Vec<(Handle<Mesh>, usize)>,
}

/// Game logic runs a fixed 1/60 s step, one input snapshot per tick (SPEC-0007).
const GAME_TICK_HZ: u32 = 60;
/// Ticks run per rendered frame at most; the rest are dropped. The original never catches up
/// (an overlong pass slows the game), which `--original-pacing` reproduces with a cap of 1.
const MAX_CATCH_UP: u32 = 4;
/// Pixel-unit scroll (touchpads) per line-unit scroll step.
const PIXELS_PER_LINE: f32 = 40.0;

#[derive(Resource)]
struct DiscStatus(String);

/// Background loads for overworld streaming (SPEC-0006 Stream loads are asynchronous): a
/// thread with its own disc handle reads a requested pack, learns its kind, and builds its
/// scene when the buffer rule will stream it.
#[derive(Resource)]
struct Streamer {
    requests: std::sync::mpsc::Sender<(u32, bool, bool, bool)>,
    #[allow(clippy::type_complexity)]
    results: std::sync::Mutex<std::sync::mpsc::Receiver<(u32, bool, Result<(play::World, Option<neopets_data::model::ModelData>), String>)>>,
    in_flight: std::collections::HashSet<u32>,
    scenes: std::collections::HashMap<u32, neopets_data::model::ModelData>,
    regions: std::sync::mpsc::Sender<(u32, u8, u32)>,
    #[allow(clippy::type_complexity)]
    region_results: std::sync::Mutex<std::sync::mpsc::Receiver<(u32, u8, u32, Result<RegionLoad, String>)>>,
    /// Region loads sent and not yet received.
    regions_out: usize,
}

type RegionCharacter = (
    neopets_data::region::Placement,
    neopets_data::model::ModelData,
    character::HeroAnim,
    Option<neopets_data::monster::Row>,
    Option<neopets_data::monster::Tuning>,
);

/// A loaded region: its characters and its kind-8 item points.
type RegionLoad = (Vec<RegionCharacter>, Vec<neopets_data::region::ItemPoint>);

/// Read a region resource and build each placed character, and each summon-reserved record (its
/// summon byte set; kept by the host for spawn by key). Records with no position that are not
/// reserved are left out.
fn load_region(disc: &mut Disc, index: &ArchiveIndex, table: &neopets_data::monster::MonsterTable, region: u32) -> Result<RegionLoad, String> {
    let file = index.read(disc, region)?;
    let chain = neopets_data::resource::walk_chunks(&file, None)?;
    // A world pack's own characters are its embedded region: the chain inside its type-32
    // chunk, with offsets relative to that chunk's payload.
    let (bytes, chunks) = match chain.chunks.iter().find(|c| c.kind == neopets_data::region::EMBEDDED) {
        Some(c) if chain.chunks.first().is_some_and(|f| f.kind == 31) => {
            let payload = file[c.payload.clone()].to_vec();
            let nested = neopets_data::resource::walk_chunks(&payload, None)?;
            (payload, nested.chunks)
        }
        _ if chain.chunks.first().is_some_and(|f| f.kind == 31) => return Ok((Vec::new(), Vec::new())),
        _ => (file, chain.chunks),
    };
    let r = neopets_data::region::Region::of(&bytes, &chunks);
    let lists = std::sync::Arc::new(r.records.effect_lists.clone());
    let mut out = Vec::new();
    for pl in r.records.placements.iter().filter(|p| p.summon || p.position != [0.0; 3]) {
        match archive_view::region_character(disc, index, &bytes, &r, pl) {
            Ok((model, mut anim)) => {
                anim.set_effect_lists(lists.clone());
                out.push((pl.clone(), model, anim, table.row(pl.monster_type, pl.variant), table.tuning(pl.monster_type, pl.variant)))
            }
            Err(e) => eprintln!("Region {region}, {:?}: {e}", pl.name),
        }
    }
    Ok((out, r.records.items))
}

impl Streamer {
    fn start(mut disc: Disc, index: neopets_data::archive::ArchiveIndex, boot: neopets_data::travel::BootTravel) -> Self {
        // A second handle for the region loader.
        let second = disc.reopen();
        let region_index = index.clone();
        let (requests, jobs) = std::sync::mpsc::channel::<(u32, bool, bool, bool)>();
        let (done, results) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            // Each request: the pack, whether transitions (true) or other kinds (false) stream
            // now, whether streaming is possible at all, and whether a door job asked for it
            // (its scene is always built).
            for (pack, transitions_stream, any, job) in jobs {
                let result = play::World::load(&mut disc, &index, &boot, pack).and_then(|(world, bytes)| {
                    let streams = any && ((world.kind == neopets_data::travel::Kind::Transition) == transitions_stream)
                        && world.kind != neopets_data::travel::Kind::Interior;
                    let scene = if streams || job {
                        let chain = neopets_data::resource::walk_chunks(&bytes, None)?;
                        archive_view::world_scene(&mut disc, &index, pack, &bytes, &chain.chunks, false)?
                    } else {
                        None
                    };
                    Ok((world, scene))
                });
                if done.send((pack, job, result)).is_err() {
                    break;
                }
            }
        });
        let (regions, region_jobs) = std::sync::mpsc::channel::<(u32, u8, u32)>();
        let (region_done, region_results) = std::sync::mpsc::channel();
        if let Ok(mut disc) = second {
            std::thread::spawn(move || {
                // The monster table (SPEC-0029 section 4), read once.
                let table = match (region_index.read(&mut disc, 530), region_index.read(&mut disc, 531)) {
                    (Ok(a), Ok(b)) => neopets_data::monster::MonsterTable::parse(&a, &b).unwrap_or_default(),
                    _ => Default::default(),
                };
                for (pack, slot, region) in region_jobs {
                    let result = load_region(&mut disc, &region_index, &table, region);
                    if region_done.send((pack, slot, region, result)).is_err() {
                        break;
                    }
                }
            });
        }
        Self {
            requests,
            results: std::sync::Mutex::new(results),
            in_flight: Default::default(),
            scenes: Default::default(),
            regions,
            region_results: std::sync::Mutex::new(region_results),
            regions_out: 0,
        }
    }

    /// Send the pending step's and the door job's requests, install finished loads; returns a
    /// line per finished load.
    fn pump(&mut self, p: &mut play::Play) -> Vec<String> {
        let buffer = p.world.buffer;
        let transitions = buffer == neopets_sim::travel::Buffer::Exterior;
        let any = buffer != neopets_sim::travel::Buffer::Interior;
        if let Some(pack) = p.wanted.take() {
            if self.in_flight.insert(pack) {
                let _ = self.requests.send((pack, transitions, any, false));
            }
        }
        if let Some(pack) = p.job_pack() {
            if self.in_flight.insert(pack) {
                let _ = self.requests.send((pack, transitions, any, true));
            }
        }
        for (pack, slot, region) in p.region_wanted.drain(..) {
            if self.regions.send((pack, slot, region)).is_ok() {
                self.regions_out += 1;
            }
        }
        let mut log = Vec::new();
        let regions: Vec<_> = self.region_results.lock().map(|r| r.try_iter().collect()).unwrap_or_default();
        for (pack, slot, region, result) in regions {
            self.regions_out = self.regions_out.saturating_sub(1);
            match result {
                Ok((list, items)) => {
                    let names: Vec<String> = list.iter().map(|c| c.0.name.clone()).collect();
                    log.push(format!("region {region} for slot {slot} of {pack} on tick {}: {names:?}, {} item points", p.ticks, items.len()));
                    p.add_characters(pack, slot, region, list, items);
                }
                Err(e) => log.push(format!("region {region} failed: {e}")),
            }
        }
        p.loads_out = self.regions_out;
        let finished: Vec<_> = self.results.lock().map(|r| r.try_iter().collect()).unwrap_or_default();
        for (pack, job, result) in finished {
            self.in_flight.remove(&pack);
            match result {
                Ok((world, scene)) => {
                    log.push(format!("loaded {pack} on tick {} ({:?}, scene {}, door request {job})", p.ticks, world.kind, scene.is_some()));
                    if let Some(scene) = scene {
                        self.scenes.insert(pack, scene);
                    }
                    // A load of the job's pack finishes the job, whichever request read it.
                    if p.job_pack() == Some(pack) {
                        p.finish_job(world);
                    } else {
                        p.install(world);
                    }
                }
                Err(e) => log.push(format!("load of {pack} failed: {e}")),
            }
        }
        log
    }
}

/// Send the pending step's request to the loader and install finished loads.
fn stream_worlds(play: Option<ResMut<PlayState>>, streamer: Option<ResMut<Streamer>>) {
    let (Some(mut state), Some(mut streamer)) = (play, streamer) else { return };
    for line in streamer.pump(&mut state.play) {
        info!("{line}");
    }
}

/// The opened disc and its archive index, present only when the ISO validated.
#[derive(Resource)]
struct Archive {
    disc: Disc,
    index: ArchiveIndex,
}

/// World height of the texture quad; its width follows the texture's aspect ratio.
const QUAD_HEIGHT: f32 = 4.0;

#[derive(Component)]
struct TextureQuad;

/// The diagnostic grid and axes, hidden while a model is shown.
#[derive(Component)]
struct DiagnosticGrid;

/// Every entity spawned for the current model, despawned when the selection changes.
#[derive(Component)]
struct ModelPart;

/// The selected resource's model, and whether the viewer shows it or the texture quad.
#[derive(Resource, Default)]
struct ModelView {
    data: Option<neopets_data::model::ModelData>,
    show_model: bool,
    dirty: bool,
    /// Meshes of skinned parts (layout A) and their part index, for per-frame skinning.
    skinned: Vec<(Handle<Mesh>, usize)>,
    hero: Option<character::HeroAnim>,
}

impl ModelView {
    fn showing(&self) -> bool {
        self.show_model && self.data.is_some()
    }
}

/// Decoded textures of the selected resource, and which one the quad shows.
#[derive(Resource, Default)]
struct TextureView {
    images: Vec<(String, neopets_data::texture::Image)>,
    current: usize,
    dirty: bool,
}

pub(crate) fn rgba_image(width: u32, height: u32, rgba: Vec<u8>) -> Image {
    let mut image = Image::new(
        Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        rgba,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::default(),
    );
    image.sampler = ImageSampler::nearest();
    image
}

/// Summary text for the last resource described, so the disc is read once per selection.
#[derive(Resource, Default)]
struct BrowserView {
    shown: Option<u32>,
    text: String,
}

/// Host side of the input -> sim -> render loop. All game logic stays in neopets-sim.
#[derive(Resource)]
struct SimDriver {
    sim: Simulation,
    clock: FixedStep,
    input: PlayerInput,
    last_ticks: u32,
}

#[derive(Component)]
struct Hud;

/// Whether the HUD starts hidden (`--hide-hud`); Tab toggles it afterwards.
#[derive(Resource)]
struct HudHidden(bool);

#[derive(Component)]
struct SimText;

#[derive(Component)]
struct BrowserText;

fn main() {
    let mut args = std::env::args().skip(1);
    let mut explicit = None;
    let mut smoke_test = false;
    let mut screenshot = None;
    let mut hide_hud = false;
    // `--show-info`: in play, start with the developer info panel shown (F6 toggles it).
    let mut show_info = false;
    let mut catch_up = MAX_CATCH_UP;
    let mut play_mode = false;
    let mut play_pack = PLAY_PACK;
    let mut autowalk = None;
    let mut shot_at = 4.0f32;
    let mut start_resource = 0u32;
    let mut trace: Option<u64> = None;
    let mut trace_to: Option<[f32; 2]> = None;
    let mut flags: Vec<u16> = Vec::new();
    let mut look: Option<[f32; 2]> = None;
    let mut seed: Option<u32> = None;
    let mut give: Vec<u16> = Vec::new();
    // `--guard`: the fight trace holds L1 (the shield guard) while it is close to its enemy.
    let mut guard = false;
    // `--save-test`: at the end of a trace, save slot 0 to private/saves/trace.bin, read it back
    // and load it into a rebuilt game.
    let mut save_test = false;
    let mut render: Option<(u32, u32, u32, u32)> = None;
    let mut render_cue: Option<(u32, i32)> = None;
    // `--effects-check`: run SPEC-0034's per-tick vectors 70 to 72 on the disc's records.
    let mut effects_check = false;
    // `--title`: start at the front end's card state (logo, legal screen, title) instead of in
    // play.
    let mut title = false;
    let mut start: Option<[f32; 3]> = None;
    while let Some(arg) = args.next() {
        if arg == "--iso" {
            explicit = Some(
                args.next()
                    .unwrap_or_else(|| {
                        eprintln!("--iso requires a path");
                        std::process::exit(2)
                    })
                    .into(),
            );
        } else if arg == "--smoke-test" {
            smoke_test = true;
        } else if arg == "--resource" {
            start_resource = args
                .next()
                .and_then(|n| n.parse().ok())
                .unwrap_or_else(|| {
                    eprintln!("--resource requires a resource number");
                    std::process::exit(2)
                });
        } else if arg == "--autowalk" {
            autowalk = Some(AutoWalk { jump: false });
        } else if arg == "--autojump" {
            autowalk = Some(AutoWalk { jump: true });
        } else if arg == "--render-cue" {
            // `--render-cue BLOCK CUE`: render one sound-block cue for 3 s to
            // private/audio/cue-BLOCK-CUE.wav.
            let mut n = || args.next().and_then(|v| v.parse::<i32>().ok());
            render_cue = Some((n().unwrap_or(347) as u32, n().unwrap_or(20)));
        } else if arg == "--effects-check" {
            effects_check = true;
        } else if arg == "--render-music" {
            // `--render-music BANK SECONDS [G] [R2]`: render a bank's music offline to
            // private/audio/bank-BANK.wav (for checking the sequencer and synth).
            let mut n = || args.next().and_then(|v| v.parse::<u32>().ok());
            render = Some((n().unwrap_or(28), n().unwrap_or(20), n().unwrap_or(0), n().unwrap_or(0)));
        } else if arg == "--play-trace" {
            trace = Some(args.next().and_then(|v| v.parse().ok()).unwrap_or_else(|| {
                eprintln!("--play-trace requires a tick count");
                std::process::exit(2)
            }));
        } else if arg == "--pack" {
            play_pack = args.next().and_then(|v| v.parse().ok()).unwrap_or_else(|| {
                eprintln!("--pack requires a world pack resource number");
                std::process::exit(2)
            });
        } else if arg == "--trace-to" {
            trace_to = args.next().and_then(|v| {
                if v == "swap" {
                    return Some([f32::NAN, f32::NAN]);
                }
                if v == "new" {
                    return Some([f32::NAN, 0.0]);
                }
                if v == "door" {
                    return Some([f32::NAN, 1.0]);
                }
                if v == "herd" {
                    return Some([f32::NAN, 2.0]);
                }
                if v == "fight" {
                    return Some([f32::NAN, 3.0]);
                }
                let (x, z) = v.trim().split_once(',')?;
                Some([x.parse().ok()?, z.parse().ok()?])
            });
        } else if arg == "--look" {
            look = args.next().and_then(|v| {
                let (x, z) = v.trim().split_once(',')?;
                Some([x.parse::<f32>().ok()?, z.parse::<f32>().ok()?])
            });
        } else if arg == "--start" {
            // x,z or x,z,y: y is the height the ground probe starts from (default far above).
            start = args.next().and_then(|v| {
                let n: Vec<f32> = v.trim().split(',').map(|t| t.trim().parse::<f32>().ok()).collect::<Option<_>>()?;
                match n[..] {
                    [x, z] => Some([x, z, 1.0e6]),
                    [x, z, y] => Some([x, z, y]),
                    _ => None,
                }
            });
        } else if arg == "--give" {
            give.extend(args.next().and_then(|v| v.parse::<u16>().ok()));
        } else if arg == "--guard" {
            guard = true;
        } else if arg == "--save-test" {
            save_test = true;
        } else if arg == "--title" {
            title = true;
        } else if arg == "--seed" {
            seed = args.next().and_then(|v| v.parse::<u32>().ok());
        } else if arg == "--flag" {
            flags.extend(args.next().and_then(|v| v.parse::<u16>().ok()));
        } else if arg == "--play" {
            play_mode = true;
        } else if arg == "--original-pacing" {
            catch_up = 1;
        } else if arg == "--hide-hud" {
            hide_hud = true;
        } else if arg == "--show-info" {
            show_info = true;
        } else if arg == "--screenshot-at" {
            shot_at = args.next().and_then(|v| v.parse().ok()).unwrap_or_else(|| {
                eprintln!("--screenshot-at requires seconds");
                std::process::exit(2)
            });
        } else if arg == "--screenshot" {
            screenshot = Some(args.next().unwrap_or_else(|| {
                eprintln!("--screenshot requires an output .png path");
                std::process::exit(2)
            }));
        } else {
            eprintln!("Unknown argument: {arg}");
            std::process::exit(2);
        }
    }
    let root = std::env::current_dir().expect("Current directory");
    let result = iso_path(&root, explicit).and_then(|path| {
        let mut disc = Disc::open(&path)?;
        let name = disc.verify_neopets()?;
        let index = ArchiveIndex::from_disc(&mut disc)?;
        let status = format!(
            "{name}\n{} disc files indexed. Opened read-only. Boot fingerprint matches; {} archive resources.",
            disc.files().len(),
            index.len()
        );
        Ok((status, Archive { disc, index }))
    });
    let (status, archive) = match result {
        Ok((s, archive)) => (s, Some(archive)),
        Err(e) => (format!("ISO not ready\n{e}"), None),
    };
    if effects_check {
        let Some(mut archive) = archive else {
            eprintln!("{status}");
            std::process::exit(1)
        };
        let Archive { disc, index } = &mut archive;
        if let Err(e) = check_effects(disc, index) {
            eprintln!("{e}");
        }
        return;
    }
    if let Some((number, cue)) = render_cue {
        let Some(mut archive) = archive else {
            eprintln!("{status}");
            std::process::exit(1)
        };
        let Archive { disc, index } = &mut archive;
        let block = index.read(disc, number).and_then(|b| neopets_data::block::Block::parse(&b));
        match block {
            Ok(b) => {
                if std::env::var_os("NEOPETS_CUE_DUMP").is_some() {
                    if let Some(c) = b.cues.get(cue as usize) {
                        println!("cue {c:?}");
                        for cmd in &b.commands[c.first..c.first + c.count as usize] {
                            println!("  {cmd:?} voice {:?}", b.voices.get(&cmd.value()));
                        }
                    }
                }
                let def = std::sync::Arc::new(audio_view::block_def(number, &b));
                let mut fx = neopets_sim::cues::Effects::default();
                let started = fx.play(1, def, cue, 1024, None);
                let mut out = vec![0f32; 2 * 3 * neopets_sim::synth::RATE as usize];
                fx.render(&mut out);
                let peak = out.iter().fold(0f32, |a, &b| a.max(b.abs()));
                let last = out.chunks(2).rposition(|f| f[0] != 0.0 || f[1] != 0.0).unwrap_or(0);
                let pcm: Vec<i16> = out.iter().map(|&v| (v * 32767.0) as i16).collect();
                let path = format!("private/audio/cue-{number}-{cue}.wav");
                let _ = std::fs::create_dir_all("private/audio");
                let _ = std::fs::write(&path, wav(&pcm, 2, neopets_sim::synth::RATE));
                println!("Cue {cue} of block {number}: started {started}, peak {peak:.3}, sound until {:.3} s; {path}", last as f32 / neopets_sim::synth::RATE as f32);
            }
            Err(e) => eprintln!("Block {number}: {e}"),
        }
        return;
    }
    if let Some((bank, secs, excite, reg2)) = render {
        let Some(mut archive) = archive else {
            eprintln!("{status}");
            std::process::exit(1)
        };
        match audio_view::load_bank(&mut archive, bank) {
            Ok(mut engine) => {
                engine.volume = 93;
                engine.sequencer.excite = excite as u8;
                let frames = (secs * neopets_sim::synth::RATE) as usize;
                let mut out = vec![0f32; 2 * frames];
                let mut peak_voices = 0;
                for chunk in out.chunks_mut(2 * neopets_sim::sequencer::FRAMES_PER_TICK) {
                    engine.sequencer.registers[2] = reg2 as u8;
                    engine.render(chunk);
                    peak_voices = peak_voices.max(engine.synth.active());
                }
                let pcm: Vec<i16> = out.iter().map(|&v| (v * 32767.0) as i16).collect();
                let data = pcm.len() as u32 * 2;
                let mut w = Vec::new();
                w.extend(b"RIFF");
                w.extend((36 + data).to_le_bytes());
                w.extend(b"WAVEfmt ");
                w.extend(16u32.to_le_bytes());
                w.extend(1u16.to_le_bytes());
                w.extend(2u16.to_le_bytes());
                w.extend(neopets_sim::synth::RATE.to_le_bytes());
                w.extend((neopets_sim::synth::RATE * 4).to_le_bytes());
                w.extend(4u16.to_le_bytes());
                w.extend(16u16.to_le_bytes());
                w.extend(b"data");
                w.extend(data.to_le_bytes());
                for v in &pcm {
                    w.extend(v.to_le_bytes());
                }
                let path = format!("private/audio/bank-{bank}.wav");
                let _ = std::fs::create_dir_all("private/audio");
                match std::fs::write(&path, w) {
                    Ok(()) => println!("Rendered bank {bank} for {secs} s at G {excite}, register 2 {reg2}: {path}; peak voices {peak_voices}; playing children {:?}", engine.sequencer.playing()),
                    Err(e) => eprintln!("{path}: {e}"),
                }
            }
            Err(e) => eprintln!("Bank {bank}: {e}"),
        }
        return;
    }
    if let Some(ticks) = trace {
        let Some(mut archive) = archive else {
            eprintln!("{status}");
            std::process::exit(1)
        };
        play_trace(&mut archive, play_pack, ticks, autowalk.is_some_and(|a| a.jump), trace_to, &flags, seed, &give, start, (guard, save_test));
        return;
    }
    let driver = SimDriver {
        sim: Simulation::default(),
        clock: FixedStep::new(GAME_TICK_HZ, catch_up).expect("Positive tick rate"),
        input: PlayerInput::default(),
        last_ticks: 0,
    };
    let mut app = App::new();
    if let Some(archive) = archive {
        app.insert_resource(archive);
        if play_mode {
            app.insert_resource(PlayRequest(play_pack, look, give.clone(), title, flags.clone(), start));
        }
        if let Some(auto) = autowalk {
            app.insert_resource(auto);
        }
    }
    // In play the developer info panel starts hidden unless asked for.
    let hide_hud = hide_hud || (play_mode && !show_info);
    app.insert_resource(DiscStatus(status))
        .insert_resource(HudHidden(hide_hud))
        .insert_resource(driver)
        .insert_resource(Browser {
            selected: if play_mode { play_pack } else { start_resource },
            typing: None,
        })
        .init_resource::<BrowserView>()
        .init_resource::<TextureView>()
        .init_resource::<ModelView>()
        .insert_resource(ClearColor(Color::srgb(0.025, 0.035, 0.06)))
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Neopets • Rust rewrite workspace".into(),
                resolution: (1000, 600).into(),
                visible: !smoke_test,
                ..default()
            }),
            ..default()
        }))
        .init_resource::<dev_menu::DevMenu>()
        .init_resource::<perf::PerfOverlay>()
        .insert_resource(display::DisplaySettings::load())
        .init_resource::<display::SceneTarget>()
        .add_systems(Startup, (setup_viewer, setup_hud, setup_play, dev_menu::spawn_panel, perf::spawn_panel, display::setup))
        .add_systems(Update, (display::take_requests, display::apply_window, display::sync_scene).chain())
        .add_systems(Update, (audio_view::sync_music, audio_view::sync_bank, audio_view::sync_effects, audio_view::sync_stingers).after(step_sim))
        .add_systems(
            Update,
            (
                (dev_menu::dev_input, read_input, step_sim, stream_worlds).chain(),
                (browse_resources, cycle_textures, toggle_view, choose_action, orbit_camera, toggle_hud),
                (show_sim, show_browser),
                (apply_texture, apply_model, skin_hero, sync_worlds, sync_zones, sync_uv_scroll, (play_frame, sync_hero_items).chain(), (sync_characters, sync_attachments).chain(), sync_shots, effect_view::sync_effects, foliage_view::sync_bushes, sync_objects, sync_items, hud_view::sync_hud, hud_view::sync_textbox, menu_view::sync_menu, sync_fade, dev_menu::free_camera, dev_menu::show_panel, perf::perf_overlay).chain(),
            )
                .chain(),
        );
    app.add_systems(Update, sync_breakables.after(sync_objects));
    app.add_systems(Update, sync_blocks.after(sync_objects));
    app.add_systems(Update, send_rumble);
    audio_view::register(&mut app);
    if smoke_test {
        app.insert_resource(bevy::winit::WinitSettings::continuous());
        app.add_systems(Update, smoke_exit);
    } else if let Some(path) = screenshot {
        app.insert_resource(bevy::winit::WinitSettings::continuous())
            .insert_resource(ScreenshotPath(path, shot_at.max(1.0)))
            .add_systems(Update, screenshot_and_exit);
    }
    app.run();
}

#[derive(Resource)]
struct ScreenshotPath(String, f32);

/// Capture one frame once rendering has settled, then exit; for reviewing the window.
/// Waits on real time, not frames: render pipelines compile asynchronously after startup,
/// and meshes are simply not drawn until theirs are ready.
fn screenshot_and_exit(
    mut commands: Commands,
    time: Res<Time<Real>>,
    path: Res<ScreenshotPath>,
    mut taken: Local<bool>,
    mut exit: MessageWriter<AppExit>,
) {
    use bevy::render::view::screenshot::{Screenshot, save_to_disk};
    let elapsed = time.elapsed_secs();
    if !*taken && elapsed >= path.1 {
        *taken = true;
        commands
            .spawn(Screenshot::primary_window())
            .observe(save_to_disk(path.0.clone()));
    } else if elapsed >= path.1 + 2.0 {
        // Saving runs asynchronously; exiting too soon drops the capture.
        exit.write(AppExit::Success);
    }
}

fn smoke_exit(
    frame: Res<bevy::diagnostic::FrameCount>,
    driver: Res<SimDriver>,
    mut exit: MessageWriter<AppExit>,
) {
    if frame.0 >= 5 {
        info!(
            "Startup smoke check passed after five frames; scaffold sim at tick {}.",
            driver.sim.tick
        );
        exit.write(AppExit::Success);
    }
}

/// Orbit camera over a diagnostic grid and axes (X red, Y green, Z blue). None of this is
/// game content; decoded textures, models, and areas will replace it in M3/M4/M6.
fn setup_viewer(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
) {
    // The quad starts with the synthetic pattern; a selected resource's textures replace it.
    let texture = match pattern::rgba() {
        Ok(rgba) => Some(images.add(rgba_image(pattern::SIZE, pattern::SIZE, rgba))),
        Err(e) => {
            error!("Synthetic test pattern failed to decode: {e}");
            None
        }
    };
    commands.spawn((
        Mesh3d(meshes.add(Rectangle::new(QUAD_HEIGHT, QUAD_HEIGHT))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color_texture: texture,
            unlit: true,
            alpha_mode: AlphaMode::Blend,
            cull_mode: None,
            ..default()
        })),
        Transform::from_xyz(-3.0, QUAD_HEIGHT / 2.0 + 0.2, -3.0),
        TextureQuad,
    ));
    let orbit = Orbit::default();
    // No tone mapping: decoded textures should show their stored colours.
    commands.spawn((
        Camera3d::default(),
        bevy::core_pipeline::tonemapping::Tonemapping::None,
        Projection::Perspective(PerspectiveProjection {
            near: 0.02,
            far: 400_000.0,
            ..default()
        }),
        orbit.transform(),
        orbit,
    ));
    let mut unlit = |r: f32, g: f32, b: f32| {
        materials.add(StandardMaterial {
            base_color: Color::srgb(r, g, b),
            unlit: true,
            ..default()
        })
    };
    let grid = unlit(0.16, 0.20, 0.28);
    let axes = [
        (Vec3::new(5.0, 0.0, 0.0), Vec3::new(10.0, 0.06, 0.06), unlit(0.9, 0.25, 0.25)),
        (Vec3::new(0.0, 2.5, 0.0), Vec3::new(0.06, 5.0, 0.06), unlit(0.3, 0.85, 0.35)),
        (Vec3::new(0.0, 0.0, 5.0), Vec3::new(0.06, 0.06, 10.0), unlit(0.3, 0.5, 0.95)),
    ];
    const HALF: i32 = 10;
    let line_x = meshes.add(Cuboid::new(2.0 * HALF as f32, 0.02, 0.02));
    let line_z = meshes.add(Cuboid::new(0.02, 0.02, 2.0 * HALF as f32));
    for i in -HALF..=HALF {
        let at = i as f32;
        commands.spawn((
            Mesh3d(line_x.clone()),
            MeshMaterial3d(grid.clone()),
            Transform::from_xyz(0.0, 0.0, at),
            DiagnosticGrid,
        ));
        commands.spawn((
            Mesh3d(line_z.clone()),
            MeshMaterial3d(grid.clone()),
            Transform::from_xyz(at, 0.0, 0.0),
            DiagnosticGrid,
        ));
    }
    for (center, size, material) in axes {
        commands.spawn((
            Mesh3d(meshes.add(Cuboid::from_size(size))),
            MeshMaterial3d(material),
            Transform::from_translation(center),
            DiagnosticGrid,
        ));
    }
}

fn setup_hud(mut commands: Commands, status: Res<DiscStatus>, hidden: Res<HudHidden>) {
    let font = |px: f32| TextFont {
        font_size: bevy::text::FontSize::Px(px),
        ..default()
    };
    commands
        .spawn((
            Node {
                flex_direction: FlexDirection::Column,
                row_gap: px(18),
                margin: UiRect::all(px(32)),
                max_width: percent(90),
                display: if hidden.0 { Display::None } else { Display::Flex },
                padding: UiRect::all(px(20)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.02, 0.03, 0.05, 0.85)),
            Hud,
        ))
        .with_children(|ui| {
            ui.spawn((
                Text::new(format!(
                    "NEOPETS\nThe Darkest Faerie - rewrite workspace\n\n{}\n\nArea rendering and gameplay are not implemented yet. Original content comes from your ISO. No BIOS is used.",
                    // Bevy's built-in font has no bullet glyph.
                    status.0.replace(" • ", " | ")
                )),
                font(20.0),
                TextColor(Color::srgb(0.87, 0.91, 0.97)),
            ));
            ui.spawn((
                Text::default(),
                font(16.0),
                TextColor(Color::srgb(0.70, 0.85, 0.75)),
                SimText,
            ));
            ui.spawn((
                Text::default(),
                font(16.0),
                TextColor(Color::srgb(0.95, 0.82, 0.62)),
                BrowserText,
            ));
            ui.spawn((
                Text::new(
                    "VIEWER - diagnostic grid and axes. The quad shows the selected resource's textures, \
                     decoded from your ISO (or a synthetic pattern when there is no ISO)\n\
                     Drag with a mouse button to orbit | scroll to zoom | Tab hides this text",
                ),
                font(16.0),
                TextColor(Color::srgb(0.65, 0.72, 0.95)),
            ));
        });
}

fn read_input(
    keys: Res<ButtonInput<KeyCode>>,
    gamepads: Query<&Gamepad>,
    mut driver: ResMut<SimDriver>,
) {
    let held = |pair: [KeyCode; 2]| keys.any_pressed(pair);
    let keyboard = Vec2::new(
        axis(
            held([KeyCode::KeyA, KeyCode::ArrowLeft]),
            held([KeyCode::KeyD, KeyCode::ArrowRight]),
        ),
        axis(
            held([KeyCode::KeyS, KeyCode::ArrowDown]),
            held([KeyCode::KeyW, KeyCode::ArrowUp]),
        ),
    );
    driver.input = movement(keyboard, gamepads.iter().map(|pad| pad.left_stick()));
}

fn step_sim(
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    gamepads: Query<&Gamepad>,
    mut driver: ResMut<SimDriver>,
    mut models: ResMut<ModelView>,
    play: Option<ResMut<PlayState>>,
    autowalk: Option<Res<AutoWalk>>,
    mut menu: Option<ResMut<menu_view::Menu>>,
    art: Option<Res<hud_view::HudArt>>,
    mut archive: Option<ResMut<Archive>>,
    mut commands: Commands,
    mut dev: ResMut<dev_menu::DevMenu>,
) {
    let mut elapsed = time.delta().as_nanos().min(u64::MAX as u128) as u64;
    // The developer menu's game speed: the elapsed time scaled (0 while paused); a step while
    // paused runs exactly one tick's time.
    if dev.speed() != 1.0 {
        elapsed = (elapsed as f64 * dev.speed()) as u64;
    }
    if std::mem::take(&mut dev.step) {
        elapsed = 1_000_000_000 / 60 + 1;
    }
    let SimDriver {
        sim, clock, input, ..
    } = &mut *driver;
    let ticks = sim.run_for(clock, elapsed, *input);
    driver.last_ticks = ticks;
    if let Some(hero) = models.hero.as_mut() {
        for _ in 0..ticks {
            let _ = hero.tick();
        }
    }
    if let Some(mut play) = play {
        let i = driver.input;
        let mut input = neopets_sim::hero::HeroInput {
            side: -(i.move_x as f32) / i16::MAX as f32,
            forward: i.move_y as f32 / i16::MAX as f32,
            run: keys.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight])
                || gamepads.iter().any(|g| g.pressed(GamepadButton::RightTrigger)),
            jump: false,
            square: false,
            square_held: keys.pressed(KeyCode::KeyE) || gamepads.iter().any(|g| g.pressed(GamepadButton::West)),
            cross_held: keys.pressed(KeyCode::Space) || gamepads.iter().any(|g| g.pressed(GamepadButton::South)),
            // L1 (Q or the left bumper) holds the shield guard.
            guard: keys.pressed(KeyCode::KeyQ) || gamepads.iter().any(|g| g.pressed(GamepadButton::LeftTrigger)),
            attack_held: keys.pressed(KeyCode::KeyF)
                || mouse.pressed(MouseButton::Left)
                || gamepads.iter().any(|g| g.pressed(GamepadButton::East) || g.pressed(GamepadButton::RightTrigger2)),
            attack: false,
        };
        // The attack button (F, left mouse, Circle or R2): its press edge, kept for the next tick
        // when this frame runs none.
        play.play.attack_latch |= keys.just_pressed(KeyCode::KeyF)
            || mouse.just_pressed(MouseButton::Left)
            || gamepads.iter().any(|g| g.just_pressed(GamepadButton::East) || g.just_pressed(GamepadButton::RightTrigger2));
        // The quick-item panel (SPEC-0021): the D-pad or numpad 8/4/6/2 for its directions, T or
        // the north face button for Triangle; Cross and Circle are the jump and attack edges.
        {
            let pad = |b: GamepadButton| gamepads.iter().any(|g| g.just_pressed(b));
            let l = &mut play.play.panel_latch;
            l.up |= keys.just_pressed(KeyCode::Numpad8) || pad(GamepadButton::DPadUp);
            l.left |= keys.just_pressed(KeyCode::Numpad4) || pad(GamepadButton::DPadLeft);
            l.right |= keys.just_pressed(KeyCode::Numpad6) || pad(GamepadButton::DPadRight);
            l.down |= keys.just_pressed(KeyCode::Numpad2) || pad(GamepadButton::DPadDown);
            l.triangle |= keys.just_pressed(KeyCode::KeyT) || pad(GamepadButton::North);
        }
        // NEOPETS_TRACE_EFFECT="id": a diagnostic one-shot of that effect 3 units in front of
        // the hero every 120 play ticks.
        if let Some(id) = std::env::var("NEOPETS_TRACE_EFFECT").ok().and_then(|v| v.trim().parse::<i32>().ok()) {
            static LAST: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
            let n = play.play.ticks / 120;
            if LAST.swap(n, std::sync::atomic::Ordering::Relaxed) != n {
                play.play.diagnostic_effect(id);
            }
        }
        // NEOPETS_TRACE_TIMER="start": a diagnostic countdown timer from `start` seconds (limit 0),
        // shown at play tick 60.
        if let Some(start) = std::env::var("NEOPETS_TRACE_TIMER").ok().and_then(|v| v.trim().parse::<f32>().ok()) {
            static TIMER_DONE: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
            if play.play.ticks >= 60 && !TIMER_DONE.swap(true, std::sync::atomic::Ordering::Relaxed) {
                play.play.state.timer.activate(start, 0.0, false);
            }
        }
        // NEOPETS_TRACE_STATUS="code,seconds": a diagnostic status on the hero once, at play
        // tick 60.
        if let Some((code, secs)) = std::env::var("NEOPETS_TRACE_STATUS").ok().and_then(|v| {
            let (a, b) = v.split_once(',')?;
            Some((a.trim().parse::<usize>().ok()?, b.trim().parse::<i32>().ok()?))
        }) {
            static DONE: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
            if play.play.ticks >= 60 && !DONE.swap(true, std::sync::atomic::Ordering::Relaxed) {
                play.play.statuses.apply(code as i32, secs);
            }
        }
        // NEOPETS_TRACE_SWITCH also works in the window: the chord on those play ticks.
        if std::env::var("NEOPETS_TRACE_SWITCH").ok().is_some_and(|v| v.split(',').filter_map(|t| t.trim().parse::<u64>().ok()).any(|k| (k..k + 3).contains(&(play.play.ticks + 1)))) {
            play.play.l1_latch = true;
            play.play.r1_latch = true;
        }
        // The hero switch chord's press edges: L1 (Q or the left bumper) and R1 (Shift or the
        // right bumper), kept for the next tick (SPEC-0053 3.8 c).
        play.play.l1_latch |= keys.just_pressed(KeyCode::KeyQ) || gamepads.iter().any(|g| g.just_pressed(GamepadButton::LeftTrigger));
        play.play.r1_latch |= keys.any_just_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight]) || gamepads.iter().any(|g| g.just_pressed(GamepadButton::RightTrigger));
        // Square (E or the west face button) confirms a prompt; a press edge like Cross.
        play.play.square_latch |= keys.just_pressed(KeyCode::KeyE)
            || gamepads.iter().any(|g| g.just_pressed(GamepadButton::West));
        // Cross (Space or the south face button) is a press edge; keep it for the next tick
        // when this frame runs none.
        play.play.jump_latch |= keys.just_pressed(KeyCode::Space)
            || gamepads.iter().any(|g| g.just_pressed(GamepadButton::South));
        if let Some(auto) = autowalk.as_deref() {
            input.forward = 0.7;
            if auto.jump && ticks > 0 && play.play.ticks % 120 == 0 {
                play.play.jump_latch = true;
            }
        }
        // The right stick as PS2 pad bytes (X right 255, Y up 0); J/L and I/K stand in for it
        // on the keyboard, R for R3.
        let mut right = gamepads.iter().map(|g| g.right_stick()).find(|v| v.length() > 0.2).unwrap_or(Vec2::ZERO);
        let key = |k: KeyCode| keys.pressed(k) as i32 as f32;
        right.x += key(KeyCode::KeyL) - key(KeyCode::KeyJ);
        right.y += key(KeyCode::KeyI) - key(KeyCode::KeyK);
        let byte = |v: f32| (127.5 + 127.5 * v.clamp(-1.0, 1.0)).round() as u8;
        let r3_held = keys.pressed(KeyCode::KeyR) || gamepads.iter().any(|g| g.pressed(GamepadButton::RightThumb));
        play.play.r3_latch |= keys.just_pressed(KeyCode::KeyR)
            || gamepads.iter().any(|g| g.just_pressed(GamepadButton::RightThumb));
        // L3 (X or the left stick button) shows every HUD group again.
        play.play.l3_latch |= keys.just_pressed(KeyCode::KeyX)
            || gamepads.iter().any(|g| g.just_pressed(GamepadButton::LeftThumb));
        // SPEC-0035: the front end runs once per tick; outside play and the game-over page the
        // world is frozen and the hero's buttons are dropped.
        let (mut edge, mut buttons) = menu_view::menu_words(&keys, &gamepads);
        let mut held = menu_view::menu_held(&keys, &gamepads);
        // The left stick's direction bits join the rising-edge and held words (never the
        // button-only word). Edges seen on a frame that runs no tick are kept for the next
        // tick, so no press is lost when the frame rate is above the 60 Hz tick rate.
        if let Some(m) = menu.as_deref_mut() {
            let stick = menu_view::stick_bits(&gamepads);
            edge |= stick & !m.stick;
            held |= stick;
            m.stick = stick;
            edge |= m.latched.0;
            buttons |= m.latched.1;
            m.latched = (0, 0);
        }
        // A slot to apply after this frame's ticks (a load, or Continue with the snapshot).
        // The developer menu or free camera holds the hero's and the game menus' input.
        if dev.blocks_input() {
            input = neopets_sim::hero::HeroInput::default();
            right = Vec2::ZERO;
            edge = 0;
            buttons = 0;
            let p = &mut play.play;
            (p.jump_latch, p.square_latch, p.attack_latch, p.l1_latch, p.r1_latch, p.r3_latch, p.l3_latch) = (false, false, false, false, false, false, false);
            p.panel_latch = Default::default();
        }
        let r3_held = r3_held && !dev.blocks_input();
        // With no tick this frame the menu edges wait for the next one.
        if ticks == 0 {
            if let Some(m) = menu.as_deref_mut() {
                m.latched = (std::mem::take(&mut edge), std::mem::take(&mut buttons));
            }
        }
        let mut pending: Option<neopets_sim::save::Slot> = None;
        // Quit's world teardown, run after this frame's ticks.
        let mut quit = false;
        for _ in 0..ticks {
            if pending.is_some() || quit {
                break;
            }
            if let (Some(m), Some(a)) = (menu.as_deref_mut(), art.as_deref()) {
                // SPEC-0035 section 6: the map and journal shortcuts take the D-pad's Left and
                // Right edges while no quick panel is open or opening and no box is up.
                let l = play.play.panel_latch;
                let ctx = menus::Ctx {
                    game_over: play.play.death.page.is_some() || play.play.death.flag_e,
                    dpad: if l.left { menus::LEFT } else { 0 } | if l.right { menus::RIGHT } else { 0 },
                    shortcuts: !play.play.quick.open && !l.up && !l.down && !play.play.dialogue_open,
                    book_gate: play.play.books.gate(),
                    book_index: play.play.books.index(),
                    box_busy: play.play.dialogue_open && play.play.textbox.open,
                };
                m.flow.held = held;
                m.flow.view = play.play.item_view(&m.flow.grid_ids);
                m.flow.map_view = play.play.map_view();
                // SPEC-0025 Clean-side mapping of the card jobs: the scans read the save file,
                // the write job writes the whole file at once.
                match m.flow.tick(&a.f, std::mem::take(&mut edge), std::mem::take(&mut buttons), &ctx) {
                    Some(menus::Command::Scan) => m.flow.scanned(read_save(SAVE_FILE)),
                    Some(menus::Command::Rebuild(k)) => {
                        play.play.write_slot(&mut m.flow.picker.file.slots[k], &save_date());
                        // The snapshot and the pending flag are set at the confirm, before the
                        // write's question and result.
                        play.play.snapshot = Some(m.flow.picker.file.slots[k].clone());
                    }
                    Some(menus::Command::Write) => {
                        let ok = write_save(SAVE_FILE, &m.flow.picker.file);
                        if let Err(e) = &ok {
                            error!("Save failed: {e}");
                        }
                        m.flow.written(ok.is_ok());
                    }
                    Some(menus::Command::Load(k)) => pending = m.flow.picker.file.slot(k).cloned(),
                    Some(menus::Command::BookClose(saved)) => {
                        play.play.books.cursor = -1;
                        if saved {
                            play.play.books.saved = -1;
                        }
                    }
                    Some(menus::Command::Quit) => quit = true,
                    Some(c) => play.play.item_command(&c),
                    None => {}
                }
                // SPEC-0024 Book index: the reader's request; a new index replaces the loaded
                // book and starts its page timer, the loaded one is kept.
                if m.flow.current == menus::BOOK {
                    if let (Some(i), Some(archive)) = (play.play.books.index(), archive.as_deref_mut()) {
                        if m.flow.book.as_ref().map(|b| b.index) != Some(i) {
                            let crate::Archive { disc, index } = archive;
                            let book = play.play.book_entry(i).and_then(|e| index.read(disc, neopets_data::book::RESOURCE).ok().and_then(|bytes| neopets_data::book::read(&bytes, e)));
                            let (start, min) = m.flow.book_words;
                            m.flow.book = book.map(|book| menus::BookPage { index: i, book, timer: neopets_sim::book::PageTimer::new(start, min) });
                        }
                    }
                }
                // The box host sets the play-active word too, so the world runs under it.
                let frozen = !matches!(m.flow.current, menus::PLAY | menus::GAME_OVER | menus::BOX_HOST);
                // SPEC-0023 section 4.1: opening the pause page clears the HUD timers and the
                // monster bar timer, so the HUD stays hidden on return until a re-arm.
                if m.flow.current == menus::PAUSE && !play.play.menu_open {
                    play.play.hud.hide_all();
                    play.play.monster_bar.t = 0;
                    // SPEC-0031: pause and unpause cues (148, 149).
                    play.play.cue(148);
                } else if m.flow.current == menus::PLAY && play.play.menu_open && m.flow.stack_was_pause() {
                    play.play.cue(149);
                }
                // Menu-table sounds play at the fixed 1,024; the save-complete cue at the default.
                for c in std::mem::take(&mut m.flow.sounds) {
                    if c == 0xA9 {
                        play.play.cue(c);
                    } else {
                        play.play.cue_fixed(c);
                    }
                }
                // The front end owns the options; play reads them.
                play.play.options = m.flow.options;
                play.play.menu_open = frozen;
                if frozen {
                    // SPEC-0009 Pass gate: once the title has been shown the music controller
                    // also runs on front-end passes: the title's bank override 25, the pause
                    // page's level, the stream gate closed.
                    if m.flow.current == menus::TITLE {
                        m.flow.title_seen = true;
                    }
                    if m.flow.title_seen {
                        play.play.frontend_music(m.flow.bank_override, m.flow.current == menus::PAUSE);
                    }
                    play.play.jump_latch = false;
                    play.play.square_latch = false;
                    play.play.attack_latch = false;
                    play.play.panel_latch = Default::default();
                    play.play.l3_latch = false;
                    continue;
                }
            }
            input.jump = std::mem::take(&mut play.play.jump_latch);
            input.square = std::mem::take(&mut play.play.square_latch);
            input.attack = std::mem::take(&mut play.play.attack_latch);
            let pad = neopets_sim::camera::CameraPad {
                // SPEC-0035 Behavior 3: the invert flags complement the right-stick bytes.
                right_x: if play.play.options.invert.0 { 255 - byte(right.x) } else { byte(right.x) },
                right_y: if play.play.options.invert.1 { 255 - byte(-right.y) } else { byte(-right.y) },
                r3_held,
                r3_pressed: std::mem::take(&mut play.play.r3_latch),
            };
            play.play.tick(&input, pad);
            // A save point's Square press opens the save screen (a direct set of record 18).
            if std::mem::take(&mut play.play.save_open) {
                if let (Some(m), Some(a)) = (menu.as_deref_mut(), art.as_deref()) {
                    m.flow.open_save(&a.f);
                }
            }
            if std::mem::take(&mut play.play.reload_wanted) {
                pending = play.play.snapshot.clone();
            }
            // The game-over exit with nothing pending: the teardown and record 0.
            if std::mem::take(&mut play.play.teardown_wanted) {
                quit = true;
                if let (Some(m), Some(a)) = (menu.as_deref_mut(), art.as_deref()) {
                    m.flow.set(&a.f, menus::CARD);
                }
            }
            // NEOPETS_TRACE_QUICK=n: an Up press edge on play tick n (the quick-item panel opens).
            if std::env::var("NEOPETS_TRACE_QUICK").ok().and_then(|v| v.trim().parse::<u64>().ok()) == Some(play.play.ticks) {
                play.play.panel_latch.up = true;
            }
            // NEOPETS_TRACE_BOOK="value@n": the book cursor set to a station value on play tick n
            // (diagnostic: the reader opens on the next pass).
            if let Some((v, k)) = std::env::var("NEOPETS_TRACE_BOOK").ok().and_then(|v| v.split_once('@').and_then(|(a, b)| Some((a.trim().parse::<i32>().ok()?, b.trim().parse::<u64>().ok()?)))) {
                if k == play.play.ticks {
                    play.play.books.cursor = v;
                }
            }
            // NEOPETS_TRACE_PAGE="map@n", "journal@n" or "options@n": View Map, View Quests or
            // the Options page on play tick n
            // (diagnostic, for a screenshot).
            if let Some((page, k)) = std::env::var("NEOPETS_TRACE_PAGE").ok().and_then(|v| v.split_once('@').map(|(a, b)| (a.to_string(), b.trim().parse::<u64>().ok()))) {
                if k == Some(play.play.ticks) {
                    if let (Some(m), Some(a)) = (menu.as_deref_mut(), art.as_deref()) {
                        m.flow.map_view = play.play.map_view();
                        m.flow.trace_open(&a.f, &page);
                    }
                }
            }
            // NEOPETS_TRACE_QUITBOX=n: the pause page with its quit box open on play tick n
            // (diagnostic, for a screenshot).
            if std::env::var("NEOPETS_TRACE_QUITBOX").ok().and_then(|v| v.trim().parse::<u64>().ok()) == Some(play.play.ticks) {
                if let (Some(m), Some(a)) = (menu.as_deref_mut(), art.as_deref()) {
                    m.flow.set(&a.f, menus::PAUSE);
                    m.flow.open_quit_box();
                }
            }
            // NEOPETS_TRACE_QUIT=n also works in the window: Quit's teardown on play tick n of
            // the first game, then play again at once (the front end's pages are skipped).
            if play.play.generation == 0 && std::env::var("NEOPETS_TRACE_QUIT").ok().and_then(|v| v.trim().parse::<u64>().ok()) == Some(play.play.ticks) {
                quit = true;
                if let (Some(m), Some(a)) = (menu.as_deref_mut(), art.as_deref()) {
                    m.flow.set(&a.f, menus::PLAY);
                }
            }
        }
        if let (Some(pack), Some(archive)) = (dev.warp.take(), archive.as_deref_mut()) {
            match warp(&mut archive.disc, &archive.index, &play.play, pack) {
                Ok(new) => {
                    play.play = new;
                    match archive.disc.reopen() {
                        Ok(second) => commands.insert_resource(Streamer::start(second, archive.index.clone(), play.play.boot.clone())),
                        Err(e) => error!("No loader thread, streaming is off: {e}"),
                    }
                    info!("Warped to world pack {pack}");
                }
                Err(e) => error!("Warp to {pack} failed: {e}"),
            }
        }
        if let (true, Some(archive)) = (quit, archive.as_deref_mut()) {
            match new_game(&mut archive.disc, &archive.index, &play.play) {
                Ok(new) => {
                    play.play = new;
                    match archive.disc.reopen() {
                        Ok(second) => commands.insert_resource(Streamer::start(second, archive.index.clone(), play.play.boot.clone())),
                        Err(e) => error!("No loader thread, streaming is off: {e}"),
                    }
                    info!("Quit: the world was torn down; a new game waits in {}", play.play.world.pack);
                }
                Err(e) => error!("Quit teardown failed: {e}"),
            }
        }
        if let (Some(slot), Some(archive)) = (pending, archive.as_deref_mut()) {
            match load_game(archive, &play.play, &slot) {
                Ok(new) => {
                    play.play = new;
                    if let Some(m) = menu.as_deref_mut() {
                        m.flow.options = play.play.options;
                    }
                    match archive.disc.reopen() {
                        Ok(second) => commands.insert_resource(Streamer::start(second, archive.index.clone(), play.play.boot.clone())),
                        Err(e) => error!("No loader thread, streaming is off: {e}"),
                    }
                    info!("Loaded: {}", play.play.status());
                }
                Err(e) => error!("Load failed: {e}"),
            }
        }
    }
}

fn browse_resources(keys: Res<ButtonInput<KeyCode>>, mut browser: ResMut<Browser>) {
    const DIGITS: [(KeyCode, KeyCode, char); 10] = [
        (KeyCode::Digit0, KeyCode::Numpad0, '0'),
        (KeyCode::Digit1, KeyCode::Numpad1, '1'),
        (KeyCode::Digit2, KeyCode::Numpad2, '2'),
        (KeyCode::Digit3, KeyCode::Numpad3, '3'),
        (KeyCode::Digit4, KeyCode::Numpad4, '4'),
        (KeyCode::Digit5, KeyCode::Numpad5, '5'),
        (KeyCode::Digit6, KeyCode::Numpad6, '6'),
        (KeyCode::Digit7, KeyCode::Numpad7, '7'),
        (KeyCode::Digit8, KeyCode::Numpad8, '8'),
        (KeyCode::Digit9, KeyCode::Numpad9, '9'),
    ];
    for (main, pad, digit) in DIGITS {
        if keys.just_pressed(main) || keys.just_pressed(pad) {
            browser.type_digit(digit);
        }
    }
    if keys.just_pressed(KeyCode::Backspace) {
        browser.backspace();
    }
    if keys.just_pressed(KeyCode::Escape) {
        browser.cancel();
    }
    if keys.any_just_pressed([KeyCode::Enter, KeyCode::NumpadEnter]) {
        browser.commit();
    }
    let step: i64 = [
        (KeyCode::BracketLeft, -1),
        (KeyCode::BracketRight, 1),
        (KeyCode::PageDown, -100),
        (KeyCode::PageUp, 100),
    ]
    .into_iter()
    .filter(|(key, _)| keys.just_pressed(*key))
    .map(|(_, d)| d)
    .sum();
    if step != 0 {
        browser.step(step);
    }
}

fn orbit_camera(
    buttons: Res<ButtonInput<MouseButton>>,
    keys: Res<ButtonInput<KeyCode>>,
    motion: Res<AccumulatedMouseMotion>,
    scroll: Res<AccumulatedMouseScroll>,
    time: Res<Time<Real>>,
    models: Res<ModelView>,
    play: Option<Res<PlayState>>,
    mut cameras: Query<(&mut Orbit, &mut Transform)>,
) {
    // In play mode the follow camera owns the view.
    if play.is_some() {
        return;
    }
    let dragging = buttons.any_pressed([MouseButton::Left, MouseButton::Right, MouseButton::Middle]);
    let lines = match scroll.unit {
        MouseScrollUnit::Line => scroll.delta.y,
        MouseScrollUnit::Pixel => scroll.delta.y / PIXELS_PER_LINE,
    };
    // While a model is shown, WASD and Q/E move the point the camera orbits (a fly camera).
    let held = |k: KeyCode| keys.pressed(k) as i32 as f32;
    let fly = if models.showing() && play.is_none() {
        Vec3::new(
            held(KeyCode::KeyD) - held(KeyCode::KeyA),
            held(KeyCode::KeyE) - held(KeyCode::KeyQ),
            held(KeyCode::KeyS) - held(KeyCode::KeyW),
        )
    } else {
        Vec3::ZERO
    };
    if !(dragging && motion.delta != Vec2::ZERO) && lines == 0.0 && fly == Vec3::ZERO {
        return;
    }
    let boost = if keys.pressed(KeyCode::ShiftLeft) { 4.0 } else { 1.0 };
    for (mut orbit, mut transform) in &mut cameras {
        if dragging {
            // Screen y grows downward; dragging up should raise the camera.
            orbit.drag(Vec2::new(motion.delta.x, -motion.delta.y));
        }
        orbit.zoom(lines);
        let step = orbit.distance * 0.75 * boost * time.delta_secs();
        orbit.fly(fly * step);
        *transform = orbit.transform();
    }
}

/// F6 shows or hides the developer info panel (Tab is the game's Start key).
fn toggle_hud(keys: Res<ButtonInput<KeyCode>>, mut huds: Query<&mut Node, With<Hud>>) {
    if keys.just_pressed(KeyCode::F6) {
        for mut node in &mut huds {
            node.display = match node.display {
                Display::None => Display::Flex,
                _ => Display::None,
            };
        }
    }
}

fn show_sim(driver: Res<SimDriver>, play: Option<Res<PlayState>>, mut texts: Query<&mut Text, With<SimText>>) {
    let d = &*driver;
    let play_line = play
        .map(|p| format!("{}\n{}\n{}\n", p.play.status(), p.play.status_camera(), p.play.status_worlds()))
        .unwrap_or_default();
    for mut text in &mut texts {
        text.0 = play_line.clone() + &format!(
            "SIMULATION LOOP - fixed 1/60 s step (SPEC-0007); no gameplay systems yet\n\
             Tick {} at {} Hz | {} this frame | {} dropped | alpha {:.2}\n\
             Input: move ({}, {}) from WASD / arrows / left stick",
            d.sim.tick,
            d.clock.rate_hz(),
            d.last_ticks,
            d.clock.dropped(),
            d.clock.alpha(),
            d.input.move_x,
            d.input.move_y,
        );
    }
}

fn show_browser(
    mut browser: ResMut<Browser>,
    archive: Option<ResMut<Archive>>,
    mut view: ResMut<BrowserView>,
    mut textures: ResMut<TextureView>,
    mut models: ResMut<ModelView>,
    mut texts: Query<&mut Text, With<BrowserText>>,
    play_request: Option<Res<PlayRequest>>,
) {
    let Some(mut archive) = archive else {
        for mut text in &mut texts {
            text.0 = "RESOURCE BROWSER\nNo validated ISO, so the archive is unavailable.".into();
        }
        return;
    };
    let Archive { disc, index } = &mut *archive;
    browser.clamp(index.len() as u32);
    let n = browser.selected;
    if view.shown != Some(n) {
        view.shown = Some(n);
        view.text = archive_view::describe(disc, index, n)
            .unwrap_or_else(|e| format!("Resource #{n}: {e}"));
        let decoded = archive_view::textures(disc, index, n);
        if let Err(e) = &decoded {
            view.text.push_str(&format!("\nTextures could not be decoded: {e}"));
        }
        *textures = TextureView {
            images: decoded.unwrap_or_default(),
            current: 0,
            dirty: true,
        };
        // In play mode the resident worlds are drawn by `sync_worlds` instead.
        let model = if play_request.is_some() { Ok(None) } else { archive_view::model(disc, index, n) };
        if let Err(e) = &model {
            view.text.push_str(&format!("\nModel could not be built: {e}"));
        }
        let data = model.ok().flatten();
        let hero = match archive_view::hero(disc, index, n) {
            Ok(h) => h,
            Err(e) => {
                view.text.push_str(&format!("\nAnimation could not be loaded: {e}"));
                None
            }
        };
        *models = ModelView {
            show_model: data.is_some(),
            data,
            dirty: true,
            skinned: Vec::new(),
            hero,
        };
    }
    let model_line = match &models.data {
        Some(m) => {
            let verts: usize = m.parts.iter().map(|p| p.positions.len()).sum();
            let tris: usize = m.parts.iter().map(|p| p.indices.len() / 3).sum();
            let mut skipped = String::new();
            if !m.skipped_leaves.is_empty() {
                skipped += &format!(", {} leaves not drawn", m.skipped_leaves.len());
            }
            if m.unresolved_placements > 0 {
                skipped += &format!(", {} stale placements skipped", m.unresolved_placements);
            }
            format!(
                "Model (SPEC-0004): {} parts, {verts} vertices, {tris} triangles, {} textures{skipped}. \
                 Unlit. V switches model/texture view; WASD/QE fly",
                m.parts.len(),
                m.textures.len()
            )
        }
        None if play_request.is_some() => "Play mode draws the resident worlds (SPEC-0006).".into(),
        None => "No models in this resource.".into(),
    };
    let model_line = match &models.hero {
        Some(h) => format!("{model_line}\n{}", h.status()),
        None => model_line,
    };
    let texture_line = match textures.images.get(textures.current) {
        Some((label, image)) => format!(
            "Texture {}/{}: {label}, {}x{} (, and . to cycle)",
            textures.current + 1,
            textures.images.len(),
            image.width,
            image.height
        ),
        None => "No textures in this resource.".into(),
    };
    let entry = match &browser.typing {
        Some(t) => format!("Go to: {t}_   (Enter to jump, Esc to cancel)"),
        None => "Type a number and press Enter | [ ] step 1 | PgUp/PgDn step 100".into(),
    };
    for mut text in &mut texts {
        text.0 = format!(
            "RESOURCE BROWSER - archive index (SPEC-0001/0002), textures (SPEC-0003)\n{}\n{model_line}\n{texture_line}\n{entry}",
            view.text
        );
    }
}

fn cycle_textures(keys: Res<ButtonInput<KeyCode>>, mut textures: ResMut<TextureView>) {
    let len = textures.images.len();
    if len < 2 {
        return;
    }
    let forward = keys.just_pressed(KeyCode::Period) as usize;
    let back = keys.just_pressed(KeyCode::Comma) as usize * (len - 1);
    let step = (forward + back) % len;
    if step != 0 {
        textures.current = (textures.current + step) % len;
        textures.dirty = true;
    }
}

/// Put the current decoded texture on the quad, sized to its aspect ratio; hide the quad when
/// the selected resource has no textures.
fn apply_texture(
    mut textures: ResMut<TextureView>,
    models: Res<ModelView>,
    mut quads: Query<
        (&MeshMaterial3d<StandardMaterial>, &mut Transform, &mut Visibility),
        With<TextureQuad>,
    >,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
) {
    if !textures.dirty {
        return;
    }
    textures.dirty = false;
    for (material, mut transform, mut visibility) in &mut quads {
        let Some((_, image)) = textures.images.get(textures.current) else {
            *visibility = Visibility::Hidden;
            continue;
        };
        if models.showing() {
            *visibility = Visibility::Hidden;
            continue;
        }
        *visibility = Visibility::Inherited;
        transform.scale.x = image.width as f32 / image.height as f32;
        if let Some(mut m) = materials.get_mut(&material.0) {
            let handle = images.add(rgba_image(image.width, image.height, image.rgba.clone()));
            if let Some(old) = m.base_color_texture.replace(handle) {
                images.remove(&old);
            }
        }
    }
}

fn toggle_view(
    keys: Res<ButtonInput<KeyCode>>,
    mut models: ResMut<ModelView>,
    mut textures: ResMut<TextureView>,
) {
    if keys.just_pressed(KeyCode::KeyV) && models.data.is_some() {
        models.show_model = !models.show_model;
        models.dirty = true;
        textures.dirty = true;
    }
}

fn model_texture(t: &neopets_data::model::ModelTexture) -> Image {
    use bevy::image::{ImageAddressMode, ImageFilterMode, ImageSamplerDescriptor};
    let mut image = rgba_image(t.image.width, t.image.height, t.image.rgba.clone());
    let wrap = |repeat: bool| {
        if repeat { ImageAddressMode::Repeat } else { ImageAddressMode::ClampToEdge }
    };
    // Materials on disc magnify bilinearly (TEX1 MMAG = 1, SPEC-0003).
    image.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor {
        address_mode_u: wrap(t.repeat_u),
        address_mode_v: wrap(t.repeat_v),
        mag_filter: ImageFilterMode::Linear,
        min_filter: ImageFilterMode::Linear,
        ..default()
    });
    image
}

/// Spawn the selected model's parts (or remove them), and frame the camera on what is shown.
fn apply_model(
    mut commands: Commands,
    play_request: Option<Res<PlayRequest>>,
    mut models: ResMut<ModelView>,
    old: Query<Entity, With<ModelPart>>,
    mut grid: Query<&mut Visibility, With<DiagnosticGrid>>,
    mut cameras: Query<(&mut Orbit, &mut Transform)>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
) {
    if !models.dirty {
        return;
    }
    models.dirty = false;
    for e in &old {
        commands.entity(e).despawn();
    }
    let grid_visibility = if models.showing() || play_request.is_some() { Visibility::Hidden } else { Visibility::Inherited };
    for mut v in &mut grid {
        *v = grid_visibility;
    }
    let mut skinned = Vec::new();
    let framing = match (&models.data, models.show_model) {
        (Some(m), true) => {
            for (i, mesh, material) in part_assets(m, &mut meshes, &mut materials, &mut images) {
                if !m.parts[i].weights.is_empty() {
                    skinned.push((mesh.clone(), i));
                }
                commands.spawn((Mesh3d(mesh), MeshMaterial3d(material), Transform::IDENTITY, ModelPart));
            }
            let (lo, hi) = if m.world { m.dense_box(0.6) } else { (m.min, m.max) };
            Orbit::framing(Vec3::from(lo), Vec3::from(hi))
        }
        _ => Orbit::default(),
    };
    models.skinned = skinned;
    for (mut orbit, mut transform) in &mut cameras {
        *orbit = framing.clone();
        *transform = orbit.transform();
    }
}

/// A drawn piece of a resident world in play mode: a world pack's scene or a sky dome.
#[derive(Component, Clone, Copy, PartialEq, Eq, Hash, Debug)]
enum WorldPart {
    Scene(u32),
    Dome(u32),
    /// The world's foliage patches (SPEC-0060).
    Foliage(u32),
    /// The world's placed objects (type 29), each on its own transform (SPEC-0028 7.1), built
    /// with the world's display mask word.
    Objects(u32, u32),
}

/// A placed object's root entity: (pack, object index).
#[derive(Component, Clone, Copy)]
struct ObjectRoot(u32, usize);

/// Each frame: a placed object's root takes its drawn transform from the play, or hides.
/// SPEC-0033 Behavior 1C step 4: the motor send of the last tick goes to pad 0's large motor
/// (the first connected controller's strong motor) for its countdown, replacing what it played.
fn send_rumble(state: Option<ResMut<PlayState>>, gamepads: Query<Entity, With<Gamepad>>, mut out: MessageWriter<bevy::input::gamepad::GamepadRumbleRequest>) {
    let Some(mut state) = state else { return };
    let Some(send) = state.play.motor.take() else { return };
    let Some(gamepad) = gamepads.iter().min() else { return };
    out.write(bevy::input::gamepad::GamepadRumbleRequest::Stop { gamepad });
    if send.level > 0 && send.count > 0 {
        out.write(bevy::input::gamepad::GamepadRumbleRequest::Add {
            gamepad,
            duration: std::time::Duration::from_secs_f32(send.count as f32 / 60.0),
            intensity: bevy::input::gamepad::GamepadRumbleIntensity::strong_motor(send.level as f32 / 255.0),
        });
    }
}

/// A drawn movable block, by (pack, block index).
#[derive(Component)]
struct BlockRoot(u32, usize);

/// Movable blocks draw their mesh (local, about the base centre) at the block position with
/// axis B as its x and axis A as its z.
fn sync_blocks(state: Option<Res<PlayState>>, mut roots: Query<(&BlockRoot, &mut Transform)>) {
    let Some(state) = state else { return };
    for (b, mut t) in &mut roots {
        if let Some(m) = state.play.block_matrix(b.0, b.1) {
            *t = Transform::from_matrix(Mat4::from_cols_array(&m));
        }
    }
}

/// A drawn breakable, by (pack, entry index).
#[derive(Component)]
struct BreakableRoot(u32, usize);

/// Breakables are drawn in states 1 and 3 with their own matrix (SPEC-0048 Breakable draw).
fn sync_breakables(state: Option<Res<PlayState>>, mut roots: Query<(&BreakableRoot, &mut Visibility)>) {
    let Some(state) = state else { return };
    for (b, mut vis) in &mut roots {
        let want = if state.play.breakable_matrix(b.0, b.1).is_some() { Visibility::Visible } else { Visibility::Hidden };
        if *vis != want {
            *vis = want;
        }
    }
}

fn sync_objects(state: Option<Res<PlayState>>, mut roots: Query<(&ObjectRoot, &mut Transform, &mut Visibility)>) {
    let Some(state) = state else { return };
    for (o, mut t, mut vis) in &mut roots {
        match state.play.object_matrix(o.0, o.1) {
            Some(m) => {
                *t = Transform::from_matrix(Mat4::from_cols_array(&m));
                *vis = Visibility::Visible;
            }
            None => *vis = Visibility::Hidden,
        }
    }
}

/// Keep the drawn worlds in step with the resident buffers (SPEC-0006): the current world, the
/// other world when the neighbour-world test passes, and the current world's sky dome.
#[allow(clippy::too_many_arguments)]
fn sync_worlds(
    mut commands: Commands,
    play: Option<Res<PlayState>>,
    archive: Option<ResMut<Archive>>,
    mut streamer: Option<ResMut<Streamer>>,
    parts: Query<(Entity, &WorldPart)>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
    mut shown: Local<Vec<WorldPart>>,
) {
    let (Some(state), Some(mut archive)) = (play, archive) else { return };
    let p = &state.play;
    let mask = |pack: u32| p.display_mask.get(&pack).copied().unwrap_or(0);
    let mut want = vec![WorldPart::Scene(p.world.pack), WorldPart::Objects(p.world.pack, mask(p.world.pack))];
    if p.foliage.contains_key(&p.world.pack) {
        want.push(WorldPart::Foliage(p.world.pack));
    }
    if let (true, Some(o)) = (p.other_drawn(), &p.other) {
        want.push(WorldPart::Scene(o.pack));
        want.push(WorldPart::Objects(o.pack, mask(o.pack)));
        if p.foliage.contains_key(&o.pack) {
            want.push(WorldPart::Foliage(o.pack));
        }
    }
    let slot = p.boot.areas.slot_of(p.world.pack);
    want.push(WorldPart::Dome(neopets_data::area::dome_for_slot(slot)));
    if *shown == want {
        return;
    }
    for (e, part) in &parts {
        if !want.contains(part) {
            commands.entity(e).despawn();
        }
    }
    let Archive { disc, index } = &mut *archive;
    for part in &want {
        if shown.contains(part) {
            continue;
        }
        if let WorldPart::Objects(pack, layers) = *part {
            let world = [&p.world].into_iter().chain(p.other.as_ref()).find(|w| w.pack == pack);
            let built = index.read(disc, pack).and_then(|bytes| {
                let chain = neopets_data::resource::walk_chunks(&bytes, None)?;
                let mut out = Vec::new();
                let mut breakables = Vec::new();
                for (i, b) in world.map_or(&[][..], |w| &w.breakables[..]).iter().enumerate() {
                    let Some(tree) = b.mesh.clone() else { continue };
                    if let Ok(m) = neopets_data::model::build_tree(&bytes, &chain.chunks, tree, layers) {
                        if !m.parts.is_empty() {
                            breakables.push((i, b.matrix, m));
                        }
                    }
                }
                for (i, o) in world.map_or(&[][..], |w| &w.objects[..]).iter().enumerate() {
                    let Some(tree) = o.as_ref().and_then(|o| o.tree.clone()) else { continue };
                    if let Ok(m) = neopets_data::model::build_tree(&bytes, &chain.chunks, tree, layers) {
                        if !m.parts.is_empty() {
                            out.push((i, m));
                        }
                    }
                }
                // SPEC-0048 Movable block record +0x64: each block draws its inline mesh.
                let mut blocks = Vec::new();
                for (i, tree) in world.map_or(&[][..], |w| &w.block_meshes[..]).iter().enumerate() {
                    let Some(tree) = tree.clone() else { continue };
                    if let Ok(m) = neopets_data::model::build_tree(&bytes, &chain.chunks, tree, layers) {
                        if !m.parts.is_empty() {
                            if std::env::var_os("NEOPETS_TRACE_BLOCK").is_some() {
                                let pts = m.parts.iter().flat_map(|p| p.positions.iter());
                                let (lo, hi) = pts.fold(([f32::MAX; 3], [f32::MIN; 3]), |(lo, hi), q| (std::array::from_fn(|k| lo[k].min(q[k])), std::array::from_fn(|k| hi[k].max(q[k]))));
                                println!("      block {i} of {pack} mesh bounds {lo:?} to {hi:?}");
                            }
                            blocks.push((i, m));
                        }
                    }
                }
                Ok((out, breakables, blocks))
            });
            match built {
                Ok((list, breakables, blocks)) => {
                    for (i, m) in blocks {
                        let parts = part_assets(&m, &mut meshes, &mut materials, &mut images);
                        commands.spawn((Transform::IDENTITY, Visibility::Visible, BlockRoot(pack, i), *part)).with_children(|c| {
                            for (_, mesh, material) in parts {
                                c.spawn((Mesh3d(mesh), MeshMaterial3d(material), Transform::IDENTITY));
                            }
                        });
                    }
                    for (i, matrix, m) in breakables {
                        let parts = part_assets(&m, &mut meshes, &mut materials, &mut images);
                        commands.spawn((Transform::from_matrix(Mat4::from_cols_array(&matrix)), Visibility::Visible, BreakableRoot(pack, i), *part)).with_children(|c| {
                            for (_, mesh, material) in parts {
                                c.spawn((Mesh3d(mesh), MeshMaterial3d(material), Transform::IDENTITY));
                            }
                        });
                    }
                    for (i, m) in list {
                        let parts = part_assets(&m, &mut meshes, &mut materials, &mut images);
                        commands.spawn((Transform::IDENTITY, Visibility::Visible, ObjectRoot(pack, i), *part)).with_children(|c| {
                            for (_, mesh, material) in parts {
                                c.spawn((Mesh3d(mesh), MeshMaterial3d(material), Transform::IDENTITY));
                            }
                        });
                    }
                }
                Err(e) => error!("objects of {pack}: {e}"),
            }
            continue;
        }
        if let WorldPart::Foliage(pack) = *part {
            match foliage_view::models(disc, index, p, pack) {
                Ok(models) => {
                    let n = foliage_view::spawn(&mut commands, p, pack, &models, *part, &mut meshes, &mut materials, &mut images);
                    info!("foliage of {pack}: {n} plants of {} models", models.len());
                }
                Err(e) => error!("foliage of {pack}: {e}"),
            }
            continue;
        }
        let streamed = match (*part, streamer.as_deref_mut()) {
            (WorldPart::Scene(pack), Some(s)) => s.scenes.remove(&pack),
            _ => None,
        };
        let model = match *part {
            WorldPart::Scene(_) if streamed.is_some() => Ok(streamed),
            WorldPart::Scene(pack) => index
                .read(disc, pack)
                .and_then(|bytes| {
                    let chain = neopets_data::resource::walk_chunks(&bytes, None)?;
                    archive_view::world_scene(disc, index, pack, &bytes, &chain.chunks, false)
                }),
            WorldPart::Dome(d) => archive_view::dome(disc, index, d),
            WorldPart::Foliage(_) | WorldPart::Objects(..) => Ok(None),
        };
        match model {
            Ok(Some(m)) => {
                for (i, mesh, material) in part_assets(&m, &mut meshes, &mut materials, &mut images) {
                    let mut e = commands.spawn((Mesh3d(mesh), MeshMaterial3d(material), Transform::IDENTITY, *part));
                    // A part under a type-3 node, or of an instance or placed object, draws only
                    // while its zone column passes.
                    if let (WorldPart::Scene(pack), Some(z)) = (*part, m.parts[i].zone) {
                        e.insert(ZoneGate(pack, z));
                    }
                    if let Some(speed) = m.parts[i].scroll.filter(|_| m.parts[i].texture.is_some()) {
                        e.insert(UvScroll(speed));
                    }
                }
            }
            Ok(None) => {}
            Err(e) => error!("{part:?} could not be built: {e}"),
        }
    }
    *shown = want;
}

/// A world part's UV scroll speeds per pass (SPEC-0003 UV mode 2).
#[derive(Component, Clone, Copy)]
struct UvScroll([f32; 2]);

/// Each frame: a scrolling part's UV offset is frac(s x N) per axis, N the 16-bit pass counter
/// (the play tick).
fn sync_uv_scroll(state: Option<Res<PlayState>>, parts: Query<(&UvScroll, &MeshMaterial3d<StandardMaterial>)>, mut materials: ResMut<Assets<StandardMaterial>>) {
    let Some(state) = state else { return };
    let n = state.play.ticks as u16;
    for (s, m) in &parts {
        if let Some(mut mat) = materials.get_mut(&m.0) {
            let off = Vec2::new(neopets_data::material::scroll_offset(s.0[0], n), neopets_data::material::scroll_offset(s.0[1], n));
            mat.uv_transform = bevy::math::Affine2::from_translation(off);
        }
    }
}

/// A world part's zone gate: (world pack, zone column).
#[derive(Component, Clone, Copy)]
struct ZoneGate(u32, u8);

/// Each frame: a gated world part shows only while its zone query passes.
fn sync_zones(state: Option<Res<PlayState>>, mut parts: Query<(&ZoneGate, &mut Visibility)>) {
    let Some(state) = state else { return };
    for (g, mut v) in &mut parts {
        let want = if state.play.zone_visible(g.0, g.1) { Visibility::Inherited } else { Visibility::Hidden };
        if *v != want {
            *v = want;
        }
    }
}

/// A live enemy projectile's marker, by pool slot.
#[derive(Component)]
struct ShotMarker(usize);

/// Enemy projectiles are drawn as small plain markers at their positions: the flight effects
/// of SPEC-0034 are not implemented, so this is a diagnostic stand-in, not the game's visual.
fn sync_shots(
    mut commands: Commands,
    state: Option<Res<PlayState>>,
    mut markers: Query<(&ShotMarker, &mut Transform, &mut Visibility)>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut made: Local<bool>,
) {
    let Some(state) = state else { return };
    if !*made {
        *made = true;
        let mesh = meshes.add(Cuboid::new(0.4, 0.4, 0.4));
        let material = materials.add(StandardMaterial { base_color: Color::srgb(1.0, 0.85, 0.2), unlit: true, ..default() });
        // The pool's slots, then 8 for the Undead Draik's spit records.
        for i in 0..neopets_sim::projectile::SLOTS + 8 {
            commands.spawn((ShotMarker(i), Mesh3d(mesh.clone()), MeshMaterial3d(material.clone()), Transform::default(), Visibility::Hidden));
        }
        return;
    }
    let spits = state.play.spit_points();
    for (m, mut t, mut vis) in &mut markers {
        let at = match m.0.checked_sub(neopets_sim::projectile::SLOTS) {
            Some(k) => spits.get(k).copied(),
            None => state.play.shots.slots.get(m.0).copied().flatten().map(|s| s.position),
        };
        match at {
            Some(p) => {
                t.translation = Vec3::from(p);
                *vis = Visibility::Visible;
            }
            None => *vis = Visibility::Hidden,
        }
    }
}

/// A world item's marker, by (pack, pool slot).
#[derive(Component)]
struct ItemMarker(u32, usize);

/// An item id's registry model (SPEC-0021 Item model registry; SPEC-0004 Item model registry
/// key): ids 98 to 190 from resource 7's boot containers, by header u16 = id - 98 (the coins
/// 135 to 139, the Motes, the treasure map); the potions 50 to 59 from resource 515 + id, drawn
/// with resource 7's materials. Any other id has no model here (weapon and shield models are
/// registered only while equipped).
fn item_model(
    disc: &mut neopets_data::Disc,
    index: &neopets_data::archive::ArchiveIndex,
    boot: &mut Option<(Vec<u8>, Vec<neopets_data::resource::Chunk>)>,
    id: u32,
    pack: u32,
) -> Result<Option<neopets_data::model::ModelData>, String> {
    let m = match id {
        98..=190 => {
            if boot.is_none() {
                let b = index.read(disc, 7)?;
                let family = neopets_data::resource::classify(&b);
                let c = neopets_data::resource::walk_chunks(&b, neopets_data::resource::chunk_limit(7, family))?.chunks;
                *boot = Some((b, c));
            }
            let (b, c) = boot.as_ref().unwrap();
            neopets_data::model::build_roles(b, c, None, Some(&[(id - 98) as u16]))?
        }
        50..=59 => match archive_view::model(disc, index, 515 + id)? {
            Some(m) => m,
            None => return Ok(None),
        },
        // SPEC-0004 / SPEC-0006 type 8: a world pack registers each type-8 child under its role
        // id as a global model id, so other ids (the farm's bucket, 6) are the spawning world's
        // own child models.
        0..=322 => {
            let bytes = index.read(disc, pack)?;
            let chain = neopets_data::resource::walk_chunks(&bytes, None)?;
            neopets_data::model::build_roles(&bytes, &chain.chunks, None, Some(&[id as u16]))?
        }
        _ => return Ok(None),
    };
    Ok((!m.parts.is_empty()).then_some(m))
}

/// A world item's parts, each with its own material so the item's life value can fade it.
#[derive(Component)]
struct ItemPart;

/// World items are drawn with their registry model (`item_model`), turned by their spin angle,
/// scaled by their drawn scale and faded by their life value (SPEC-0021 Ring layout and item
/// draw). Items with no registry model here are drawn as small plain markers (blue for charms,
/// green for other items) as a diagnostic stand-in.
#[allow(clippy::too_many_arguments)]
fn sync_items(
    mut commands: Commands,
    state: Option<Res<PlayState>>,
    archive: Option<ResMut<Archive>>,
    mut markers: Query<(Entity, &ItemMarker, &mut Transform, &mut Visibility, Option<&Children>)>,
    parts: Query<&MeshMaterial3d<StandardMaterial>, With<ItemPart>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
    mut assets: Local<Option<(Handle<Mesh>, [Handle<StandardMaterial>; 3])>>,
    mut models: Local<std::collections::HashMap<(u32, u32), Option<Vec<(Handle<Mesh>, StandardMaterial)>>>>,
    mut boot: Local<Option<(Vec<u8>, Vec<neopets_data::resource::Chunk>)>>,
) {
    let Some(state) = state else { return };
    let p = &state.play;
    let (mesh, mats) = assets.get_or_insert_with(|| {
        let colour = |r, g, b| StandardMaterial { base_color: Color::srgb(r, g, b), unlit: true, ..default() };
        (
            meshes.add(Cuboid::new(0.5, 0.5, 0.15)),
            [materials.add(colour(1.0, 0.8, 0.1)), materials.add(colour(0.3, 0.7, 1.0)), materials.add(colour(0.3, 0.9, 0.3))],
        )
    });
    let mut seen = std::collections::HashSet::new();
    for (e, m, mut t, mut vis, children) in &mut markers {
        match p.pools.get(&m.0).and_then(|pool| pool.slots.get(m.1).copied().flatten()) {
            Some(it) if p.pack_drawn(m.0) => {
                t.translation = Vec3::from(it.position);
                t.rotation = Quat::from_rotation_y(it.angle);
                t.scale = Vec3::splat(it.drawn_scale().max(0.001));
                *vis = Visibility::Visible;
                seen.insert((m.0, m.1));
                let alpha = it.life.clamp(0.0, 1.0);
                for c in children.into_iter().flatten() {
                    if let Ok(h) = parts.get(*c) {
                        if let Some(mut mat) = materials.get_mut(&h.0) {
                            if (mat.base_color.alpha() - alpha).abs() > 1e-3 {
                                mat.base_color.set_alpha(alpha);
                                mat.alpha_mode = AlphaMode::Blend;
                            }
                        }
                    }
                }
            }
            Some(_) => {
                *vis = Visibility::Hidden;
                seen.insert((m.0, m.1));
            }
            None => commands.entity(e).despawn(),
        }
    }
    let mut archive = archive;
    for (&pack, pool) in &p.pools {
        for (i, it) in pool.slots.iter().enumerate() {
            let Some(it) = it else { continue };
            if seen.contains(&(pack, i)) {
                continue;
            }
            let id = it.id as u32;
            // Boot models and potions are global; other ids come from the spawning world.
            let key = (id, if matches!(id, 50..=59 | 98..=190) { 0 } else { pack });
            if !models.contains_key(&key) {
                let built = match archive.as_deref_mut() {
                    Some(Archive { disc, index }) => item_model(disc, index, &mut boot, id, pack).unwrap_or_else(|e| {
                        error!("item model {id}: {e}");
                        None
                    }),
                    None => None,
                };
                let list = built.map(|m| {
                    part_assets(&m, &mut meshes, &mut materials, &mut images)
                        .into_iter()
                        .map(|(_, mesh, mat)| (mesh, materials.get(&mat).cloned().unwrap_or_default()))
                        .collect::<Vec<_>>()
                });
                models.insert(key, list);
            }
            let root = (ItemMarker(pack, i), Transform::from_translation(Vec3::from(it.position)), Visibility::Hidden);
            match &models[&key] {
                Some(list) => {
                    commands.spawn(root).with_children(|c| {
                        for (mesh, mat) in list {
                            c.spawn((ItemPart, Mesh3d(mesh.clone()), MeshMaterial3d(materials.add(mat.clone())), Transform::IDENTITY));
                        }
                    });
                }
                None => {
                    let k = if neopets_sim::items::is_coin(it.id) {
                        0
                    } else if it.charm {
                        1
                    } else {
                        2
                    };
                    commands.spawn((root, Mesh3d(mesh.clone()), MeshMaterial3d(mats[k].clone())));
                }
            }
        }
    }
}

/// A focus-hero item's drawn root, by its resource.
#[derive(Component)]
struct HeroItemRoot(u32);

/// SPEC-0046 Held items, focus hero: the shield and weapon models (SPEC-0021 display slots,
/// resources 544 to 563) at the frames `Play::hero_items` gives, with the stowed rescales. The
/// box minimum and size are the model's vertex bounds (they match the root-node box, SPEC-0004
/// Bounds against vertices).
#[allow(clippy::too_many_arguments)]
fn sync_hero_items(
    mut commands: Commands,
    state: Option<Res<PlayState>>,
    archive: Option<ResMut<Archive>>,
    mut roots: Query<(Entity, &HeroItemRoot, &mut Transform, &mut Visibility)>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
    mut models: Local<std::collections::HashMap<u32, Option<([f32; 3], [f32; 3])>>>,
) {
    let Some(state) = state else { return };
    let items = state.play.hero_items();
    let mut archive = archive;
    let mut shown = std::collections::HashSet::new();
    for (res, frame, scale) in &items {
        if !models.contains_key(res) {
            let built = match archive.as_deref_mut() {
                Some(Archive { disc, index }) => archive_view::model(disc, index, *res).unwrap_or_else(|e| {
                    error!("hero item model {res}: {e}");
                    None
                }),
                None => None,
            };
            let bounds = built.as_ref().map(|m| (m.min, m.max));
            if let Some(m) = built {
                let parts = part_assets(&m, &mut meshes, &mut materials, &mut images);
                commands.spawn((HeroItemRoot(*res), Transform::IDENTITY, Visibility::Hidden)).with_children(|c| {
                    for (_, mesh, material) in parts {
                        c.spawn((Mesh3d(mesh), MeshMaterial3d(material), Transform::IDENTITY));
                    }
                });
            }
            models.insert(*res, bounds);
        }
        let Some(Some((min, max))) = models.get(res).copied() else { continue };
        let f = match scale {
            play::HeroItemScale::None => 1.0,
            play::HeroItemScale::Shield => 1.6 / (min[0] - (max[0] - min[0])).abs().max(1e-6),
            play::HeroItemScale::Weapon => (2.6 / (min[2] - (max[2] - min[2])).abs().max(1e-6)).min(0.6),
        };
        let c = |k: usize| Vec4::new(frame.cols[k][0] * f, frame.cols[k][1] * f, frame.cols[k][2] * f, 0.0);
        let m = Mat4::from_cols(c(0), c(1), c(2), Vec4::new(frame.t[0], frame.t[1], frame.t[2], 1.0));
        shown.insert(*res);
        for (_, root, mut t, mut vis) in &mut roots {
            if root.0 == *res {
                *t = Transform::from_matrix(m);
                *vis = Visibility::Visible;
            }
        }
    }
    for (_, root, _, mut vis) in &mut roots {
        if !shown.contains(&root.0) {
            *vis = Visibility::Hidden;
        }
    }
}

/// A region character's drawn root, by character id.
#[derive(Component)]
struct CharacterRoot(u64);

/// A character's attachment model root, by character id and slot.
#[derive(Component)]
struct AttachmentRoot(u64, u8);

/// SPEC-0046 Second walk and attachment slots: each character's attachment models at the frames
/// `Play::character_attachments` gives; hidden otherwise, despawned with their character.
fn sync_attachments(
    mut commands: Commands,
    state: Option<Res<PlayState>>,
    mut roots: Query<(Entity, &AttachmentRoot, &mut Transform, &mut Visibility)>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
    mut spawned: Local<std::collections::HashSet<(u64, u8)>>,
) {
    let Some(state) = state else { return };
    let p = &state.play;
    let frames: std::collections::HashMap<(u64, u8), neopets_data::anim::Affine> = p.character_attachments().into_iter().map(|(id, slot, f)| ((id, slot), f)).collect();
    let live: std::collections::HashSet<u64> = p.characters.iter().map(|c| c.id).collect();
    for (e, root, mut t, mut vis) in &mut roots {
        if !live.contains(&root.0) {
            commands.entity(e).despawn();
            spawned.remove(&(root.0, root.1));
            continue;
        }
        match frames.get(&(root.0, root.1)) {
            Some(f) => {
                let c = |k: usize| Vec4::new(f.cols[k][0], f.cols[k][1], f.cols[k][2], 0.0);
                *t = Transform::from_matrix(Mat4::from_cols(c(0), c(1), c(2), Vec4::new(f.t[0], f.t[1], f.t[2], 1.0)));
                *vis = Visibility::Visible;
            }
            None => *vis = Visibility::Hidden,
        }
    }
    for c in &p.characters {
        for (slot, m) in &c.model.attachments {
            if !spawned.insert((c.id, *slot)) {
                continue;
            }
            let parts = part_assets(m, &mut meshes, &mut materials, &mut images);
            commands.spawn((AttachmentRoot(c.id, *slot), Transform::IDENTITY, Visibility::Hidden)).with_children(|ch| {
                for (_, mesh, material) in parts {
                    ch.spawn((Mesh3d(mesh), MeshMaterial3d(material), Transform::IDENTITY));
                }
            });
        }
    }
}

/// Spawn, place, skin and relight the region characters of drawn worlds; despawn the rest.
#[allow(clippy::too_many_arguments)]
fn sync_characters(
    mut commands: Commands,
    state: Option<ResMut<PlayState>>,
    mut roots: Query<(Entity, &CharacterRoot, &mut Transform, &mut Visibility)>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
    mut skinned: Local<std::collections::HashMap<u64, Vec<(Handle<Mesh>, usize)>>>,
    mut generation: Local<u32>,
) {
    let Some(state) = state else { return };
    let p = &state.play;
    let live: std::collections::HashSet<u64> = p.characters.iter().map(|c| c.id).collect();
    // A rebuilt game (Quit or a load) draws every character again.
    let rebuilt = std::mem::replace(&mut *generation, p.generation) != p.generation;
    if rebuilt {
        skinned.clear();
    }
    for (e, root, mut t, mut vis) in &mut roots {
        if rebuilt {
            commands.entity(e).despawn();
            continue;
        }
        match p.characters.iter().find(|c| c.id == root.0) {
            None => {
                commands.entity(e).despawn();
                skinned.remove(&root.0);
            }
            Some(c) => {
                t.translation = Vec3::from(c.position);
                t.rotation = Quat::from_rotation_y(c.facing[0].atan2(c.facing[2]));
                *vis = if p.pack_drawn(c.pack) && !c.inactive { Visibility::Visible } else { Visibility::Hidden };
            }
        }
    }
    skinned.retain(|id, _| live.contains(id));
    for c in &p.characters {
        let rotation = Quat::from_rotation_y(c.facing[0].atan2(c.facing[2]));
        if !skinned.contains_key(&c.id) {
            let parts = part_assets(&c.model, &mut meshes, &mut materials, &mut images);
            let mut list = Vec::new();
            commands
                .spawn((Transform::from_translation(Vec3::from(c.position)).with_rotation(rotation), Visibility::Visible, CharacterRoot(c.id)))
                .with_children(|ch| {
                    for (i, mesh, material) in parts {
                        if !c.model.parts[i].weights.is_empty() {
                            list.push((mesh.clone(), i));
                        } else if let (Some(col), Some(mut m)) = (tinted(&c.model.parts[i], None, None, Some(&c.slots)), meshes.get_mut(&mesh)) {
                            // A rigid part keeps its build colours: the slot colour once.
                            m.insert_attribute(Mesh::ATTRIBUTE_COLOR, col);
                        }
                        ch.spawn((Mesh3d(mesh), MeshMaterial3d(material), Transform::IDENTITY));
                    }
                });
            skinned.insert(c.id, list);
        }
        if !p.pack_drawn(c.pack) {
            continue;
        }
        let matrices = c.anim.skin_matrices();
        let tint = p.status_tint(Some(c.id));
        let lights = p.lights(Some(c.id), c.position);
        for (handle, i) in &skinned[&c.id] {
            let part = &c.model.parts[*i];
            let (pos, nrm) = character::skin_part(&matrices, &part.positions, &part.normals, &part.weights, &part.bones);
            let slots = if c.flash > 0 { &play::FLASH_COLOUR } else { &c.slots };
            let colours = tinted(part, relight(part, &nrm, rotation, &lights), tint, Some(slots));
            if let Some(mut mesh) = meshes.get_mut(handle) {
                mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, pos);
                mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, nrm);
                if let Some(col) = colours {
                    mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, col);
                }
            }
        }
    }
}

/// Bevy meshes and materials for every part of a model: (part index, mesh, material). Parts are
/// unlit and double-sided; textured parts mask alpha at 0.5.
pub(crate) fn part_assets(
    m: &neopets_data::model::ModelData,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    images: &mut Assets<Image>,
) -> Vec<(usize, Handle<Mesh>, Handle<StandardMaterial>)> {
    use bevy::mesh::{Indices, PrimitiveTopology};
    let handles: Vec<Handle<StandardMaterial>> = m
        .textures
        .iter()
        .map(|t| {
            materials.add(StandardMaterial {
                base_color_texture: Some(images.add(model_texture(t))),
                unlit: true,
                alpha_mode: AlphaMode::Mask(0.5),
                cull_mode: None,
                ..default()
            })
        })
        .collect();
    let untextured = materials.add(StandardMaterial {
        unlit: true,
        cull_mode: None,
        ..default()
    });
    m.parts
        .iter()
        .enumerate()
        .map(|(i, part)| {
            // A scrolling part (level water, SPEC-0046) has its own blended material, whose UV
            // offset moves every frame.
            let scrolling = match (part.scroll, part.texture) {
                (Some(_), Some(t)) => Some(materials.add(StandardMaterial {
                    base_color_texture: Some(images.add(model_texture(&m.textures[t]))),
                    unlit: true,
                    alpha_mode: AlphaMode::Blend,
                    cull_mode: None,
                    ..default()
                })),
                _ => None,
            };
            let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default());
            // The GS modulates in gamma space; Bevy blends vertex colour in linear space.
            let g = gs_unit(part);
            let colors: Vec<[f32; 4]> = part
                .colors
                .iter()
                .map(|c| [(c[0] * g).powf(2.2), (c[1] * g).powf(2.2), (c[2] * g).powf(2.2), c[3].min(1.0)])
                .collect();
            mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, part.positions.clone());
            mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, part.normals.clone());
            mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, part.uvs.clone());
            mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors);
            mesh.insert_indices(Indices::U32(part.indices.clone()));
            let material = scrolling.unwrap_or_else(|| part.texture.map_or(untextured.clone(), |t| handles[t].clone()));
            (i, meshes.add(mesh), material)
        })
        .collect()
}

/// The vertex colours are kept as GS output / 128, the unit a texture is modulated by (128
/// leaves it unchanged). A part with no texture shows its output itself, on the GS's 0 to 255
/// scale, so its scale is 128 / 255: an untextured NPC part whose lit output is 248 is nearly
/// full colour, not twice as bright (SPEC-0046 Light slots and the lit draw; SPEC-0004 Colour).
fn gs_unit(part: &neopets_data::model::MeshPart) -> f32 {
    if part.texture.is_some() { 1.0 } else { 128.0 / 255.0 }
}

/// A part's linear vertex colours: lit (or stored), times the status tint and, for a region
/// character, times the colour of the part's slot (SPEC-0040 Behavior 26 item 9; the scale
/// applies in the GS's gamma space, so it is raised to 2.2 like the colours).
fn tinted(part: &neopets_data::model::MeshPart, lit: Option<Vec<[f32; 4]>>, tint: Option<[f32; 3]>, slots: Option<&[[f32; 3]; 15]>) -> Option<Vec<[f32; 4]>> {
    let base = lit.unwrap_or_else(|| part.colors.iter().map(|c| [c[0].powf(2.2), c[1].powf(2.2), c[2].powf(2.2), c[3].min(1.0)]).collect());
    let s = slots.map_or([1.0; 3], |s| s[part.colour_slot.min(14) as usize]);
    let t = tint.unwrap_or([1.0; 3]);
    let g = gs_unit(part);
    let t = [t[0] * s[0] * g, t[1] * s[1] * g, t[2] * s[2] * g].map(|c| c.max(0.0).powf(2.2));
    Some(base.into_iter().map(|c| [c[0] * t[0], c[1] * t[1], c[2] * t[2], c[3]]).collect())
}

/// Relight a lit part from its skinned normals turned into world space by `rotation`, with a
/// light block in world space (SPEC-0046), as linear vertex colours. None for unlit parts.
fn relight(part: &neopets_data::model::MeshPart, normals: &[[f32; 3]], rotation: Quat, lights: &neopets_data::light::Lights) -> Option<Vec<[f32; 4]>> {
    if !part.lit {
        return None;
    }
    let world: Vec<[f32; 3]> = normals.iter().map(|n| (rotation * Vec3::from(*n)).to_array()).collect();
    Some(
        neopets_data::model::lit_colours(&part.raw_colors, &world, lights)
            .into_iter()
            .map(|c| [c[0].powf(2.2), c[1].powf(2.2), c[2].powf(2.2), c[3].min(1.0)])
            .collect(),
    )
}

fn choose_action(keys: Res<ButtonInput<KeyCode>>, mut models: ResMut<ModelView>) {
    let Some(hero) = models.hero.as_mut() else { return };
    let a = hero.action();
    if keys.just_pressed(KeyCode::Equal) {
        hero.play((a + 1) % character::ACTIONS);
    } else if keys.just_pressed(KeyCode::Minus) {
        hero.play((a + character::ACTIONS - 1) % character::ACTIONS);
    }
}

/// Re-skin the hero's layout-A parts from its current pose (CPU skinning, every frame).
fn skin_hero(models: Res<ModelView>, mut meshes: ResMut<Assets<Mesh>>) {
    let (Some(hero), Some(data)) = (&models.hero, &models.data) else { return };
    if !models.showing() {
        return;
    }
    let matrices = hero.skin_matrices();
    for (handle, i) in &models.skinned {
        let part = &data.parts[*i];
        let (pos, nrm) = character::skin_part(&matrices, &part.positions, &part.normals, &part.weights, &part.bones);
        let colours = relight(part, &nrm, Quat::IDENTITY, &neopets_data::light::Lights::scene_default());
        if let Some(mut mesh) = meshes.get_mut(handle) {
            mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, pos);
            mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, nrm);
            if let Some(c) = colours {
                mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, c);
            }
        }
    }
}

/// Diagnostic (`--play-trace <ticks>`): run play mode without a window, walking forward along
/// the spawn facing (and jumping every 120 ticks with `--autojump`), and print each tick.
#[allow(clippy::too_many_arguments)]
fn play_trace(
    archive: &mut Archive,
    pack: u32,
    ticks: u64,
    jump: bool,
    to: Option<[f32; 2]>,
    flags: &[u16],
    seed: Option<u32>,
    give: &[u16],
    start: Option<[f32; 3]>,
    (guard, save_test): (bool, bool),
) {
    let Archive { disc, index } = &mut *archive;
    let mut play = match play::Play::load_with_flags(disc, index, pack, PLAY_OUTFIT, flags) {
        Ok(play) => play,
        Err(e) => {
            eprintln!("Play mode could not start: {e}");
            std::process::exit(1)
        }
    };
    // `--seed N`: a fixed seed for the random stream (SPEC-0008 developer option).
    if let Some(n) = seed {
        play.rng = neopets_sim::rng::Rng::seeded(n);
    }
    // `--give N` (repeatable): a diagnostic weapon or shield equipped at the start; `--start
    // x,z` places the hero.
    for &id in give {
        play.give(id);
    }
    // NEOPETS_TRACE_QUICK=id: that item is the selected quick item (trace only).
    if let Some(id) = std::env::var("NEOPETS_TRACE_QUICK").ok().and_then(|v| v.parse::<i32>().ok()) {
        play.quick.marker = id;
    }
    // NEOPETS_TRACE_ROOTS="a,b,...": each hero clip's key count and summed root records.
    if let Ok(v) = std::env::var("NEOPETS_TRACE_ROOTS") {
        for id in v.split(',').filter_map(|t| t.trim().parse::<u32>().ok()) {
            println!("      hero clip {id}: {:?}", play.hero.root_sum(id));
        }
    }
    // NEOPETS_TRACE_MOTES="w0,w1,w2": diagnostic Mote ids set into Tor's equipment words 0 to
    // 2 (each must be held; "-" leaves a word empty).
    if let Ok(m) = std::env::var("NEOPETS_TRACE_MOTES") {
        for (word, id) in m.split(',').enumerate().filter_map(|(k, t)| Some((k, t.trim().parse::<u16>().ok()?))) {
            play.item_command(&menus::Command::EquipMote { word, id });
        }
    }
    if let Some([x, z, y]) = start {
        play.place_at(x, z, y);
    }
    // NEOPETS_TRACE_MAPS="n,...": diagnostic treasure maps held from the start (SPEC-0051).
    for n in std::env::var("NEOPETS_TRACE_MAPS").unwrap_or_default().split(',').filter_map(|t| t.trim().parse::<u32>().ok()) {
        play.map_held(n);
    }
    // NEOPETS_TRACE_QUESTS=1: the quest rows (SPEC-0049 table 1) with their flags and texts.
    if std::env::var_os("NEOPETS_TRACE_QUESTS").is_some() {
        for (i, q) in play.map_tables().0.iter().enumerate() {
            println!("      quest {i}: kind {} start {} done {} stage {} at ({:.0}, {:.0}) {:?}: {:?}", q.kind, q.start, q.done, q.stage, q.x, q.z, String::from_utf8_lossy(&q.title), String::from_utf8_lossy(&q.objective));
        }
    }
    // NEOPETS_TRACE_REGION="slot,region": a diagnostic region load into a slot at the start.
    if let Some((sl, r)) = std::env::var("NEOPETS_TRACE_REGION").ok().and_then(|v| { let (a, b) = v.split_once(',')?; Some((a.trim().parse::<u8>().ok()?, b.trim().parse::<u32>().ok()?)) }) {
        play.diag_load_region(sl, r);
    }
    // NEOPETS_TRACE_ROBERTA=1: the trace starts as Roberta (diagnostic; her attack fires shots).
    if std::env::var("NEOPETS_TRACE_ROBERTA").is_ok_and(|v| v == "1") {
        play.trace_select_hero(1);
    }
    // NEOPETS_TRACE_TARGET=<name>: the fight trace fights only characters with that name.
    play.fight_target = std::env::var("NEOPETS_TRACE_TARGET").ok().filter(|n| !n.is_empty());
    // NEOPETS_TRACE_CHEATS="god,magic,charge": the developer menu's cheats from the start.
    if let Ok(v) = std::env::var("NEOPETS_TRACE_CHEATS") {
        play.cheats = play::Cheats { god: v.contains("god"), magic: v.contains("magic"), charge: v.contains("charge") };
    }
    // NEOPETS_TRACE_HP=n: the hero's starting hit points (diagnostic); a value above the
    // maximum raises the maximum too, so the boss-fight refills keep it.
    if let Some(hp) = std::env::var("NEOPETS_TRACE_HP").ok().and_then(|v| v.parse::<i32>().ok()) {
        play.hero_hp = hp;
        play.hero_max = play.hero_max.max(hp);
    }
    // NEOPETS_TRACE_PET="type,T": a diagnostic petpet activated by type at the start (F = T).
    if let Ok(v) = std::env::var("NEOPETS_TRACE_PET") {
        let mut it = v.split(',').filter_map(|t| t.trim().parse::<i32>().ok());
        if let (Some(ty), Some(t)) = (it.next(), it.next()) {
            play.activate_pet(ty, t, t);
        }
    }
    let mut streamer = match disc.reopen() {
        Ok(second) => Streamer::start(second, index.clone(), play.boot.clone()),
        Err(e) => {
            eprintln!("No loader thread: {e}");
            std::process::exit(1)
        }
    };
    println!("Grid bounds: {:?}; nearest swap point {:?}", play.bounds(), play.nearest_swap_point());
    for q in [[-1840.4f32, 48.8, 1125.2], [-1854.0, 49.0, 1144.0], [-1845.75, 49.7, 1143.8]] {
        let inside: Vec<usize> = (0..play.world.navmeshes.len()).filter(|&i| play.world.navmeshes[i].locate_full(q).is_some()).collect();
        println!("Meshes containing {q:?}: {inside:?}");
    }
    for (k, pl) in play.world.planes.iter().enumerate() {
        let g = play.world.ground_at(pl.p[0] + 0.5 * pl.w * pl.d[0], pl.p[2] + 0.5 * pl.w * pl.d[2], pl.p[1] + 10.0);
        println!("Door {k}: P {:?} F {:?} D {:?} Q {:?} w {} h {}; ground below its middle {g:?}", pl.p, pl.f, pl.d, pl.q, pl.w, pl.h);
    }
    // `--trace-to swap` follows a searched route to the nearest reachable swap point.
    let mut route: Vec<[f32; 2]> = Vec::new();
    let mut rerouted = false;
    let mut last_door = String::new();
    let mut hops = 0;
    let mut last_hit = String::new();
    let mut save_prompt = false;
    if let Some((line, at)) = play.message {
        let text = play.world.lines.line(line as usize).map(|l| format!("{}: {}", l.speaker, l.text));
        println!("Open at the start: LevelMsg line {line} on tick {at}: {text:?}");
    }
    println!("Save points of {}: {:?}", play.world.pack, play.world.save_points);
    println!("Book stations of {} (value, trigger index): {:?}", play.world.pack, play.book_stations());
    println!("Script triggers of {} (index, kind, middle): {:?}", play.world.pack, play.trigger_points());
    if std::env::var_os("NEOPETS_TRACE_LOCATORS").is_some() {
        for (k, e) in play.world.entries.iter().enumerate() {
            println!("      locator {k}: tag {:?} at {:?}", play.world.tags.get(k), e.0.map(|x| x.round() as i32));
        }
    }
    if std::env::var_os("NEOPETS_TRACE_TRIGGERS").is_some() {
        for v in &play.world.all_volumes {
            println!("      trigger {} kind {} flags {:#x} mask {:#x} {:?}", v.index, v.kind, v.flags, v.mask, v.shape);
        }
    }
    println!("Entity table of {} (entity: region j or 255 embedded, ordinal): {:?}", play.world.pack, play.world.entity_table());
    let placed: Vec<String> = play.world.placed_effects.iter().take(12).enumerate().map(|(k, p)| format!("{k}: id {} at {:.1},{:.1},{:.1}", p.id, p.position[0], p.position[1], p.position[2])).collect();
    println!("Placed effects of {} ({} records): {}", play.world.pack, play.world.placed_effects.len(), placed.join("; "));
    let trees: Vec<String> = play.world.patches.iter().filter(|p| p.key == 4).take(3).map(|p| format!("({:.1},{:.1})", (p.area[0] + p.area[1]) / 2.0, (p.area[2] + p.area[3]) / 2.0)).collect();
    println!("Foliage of {}: {} patches; key-4 tree patches at {}", play.world.pack, play.world.patches.len(), trees.join(" "));
    let kinds: Vec<String> = play.world.groups.iter().enumerate().map(|(i, g)| format!("{i}:k{} t{:?} o{:?}", g.kind, g.triggers.first(), g.objects.iter().take(2).collect::<Vec<_>>())).collect();
    println!("Script groups of {}: {}", play.world.pack, kinds.join(" "));
    println!("Script volumes of {} (index, a point inside): {:?}", play.world.pack, play.world.volume_points());
    if to.is_some_and(|t| t[0].is_nan()) {
        let goal = match to.map(|t| t[1]) {
            Some(g) if g == 0.0 => play::Goal::NewNeighbour,
            Some(g) if g == 1.0 => play::Goal::Door,
            _ => play::Goal::Swap,
        };
        route = if to.is_some_and(|t| t[1] >= 2.0) {
            Vec::new()
        } else if goal == play::Goal::Door {
            play.stand_before_door(0).map(|m| vec![m]).unwrap_or_default()
        } else {
            play.route_to(400_000, goal).unwrap_or_default()
        };
        println!("Route to a swap point: {} steps, ending at {:?}", route.len(), route.last());
    }
    // NEOPETS_TRACE_PATH="x,z;x,z;...": waypoints walked in turn along searched routes; each is
    // left once the hero has stood within 2 units of it for 30 ticks with no dialogue box up.
    // A leg may carry a third value, the height of its floor (stacked floors).
    let leg_list: Vec<([f32; 2], Option<f32>)> = std::env::var("NEOPETS_TRACE_PATH")
        .ok()
        .map(|v| {
            v.split(';')
                .filter_map(|w| {
                    let n: Vec<f32> = w.split(',').filter_map(|t| t.trim().parse().ok()).collect();
                    (n.len() >= 2).then(|| ([n[0], n[1]], n.get(2).copied()))
                })
                .collect()
        })
        .unwrap_or_default();
    let mut legs: Vec<[f32; 2]> = leg_list.iter().map(|l| l.0).collect();
    let mut leg_y: Vec<Option<f32>> = leg_list.iter().map(|l| l.1).collect();
    let walk_path = !legs.is_empty();
    // NEOPETS_NEOPOINTS=n: a diagnostic starting wallet.
    if let Some(n) = std::env::var("NEOPETS_NEOPOINTS").ok().and_then(|v| v.parse().ok()) {
        play.state.neopoints = n;
    }
    let mut dwell = 0;
    let mut leg_pack = play.world.pack;
    let mut scene = String::new();
    let mut last_hurt = String::new();
    let mut last_mote = String::new();
    let mut last_status = String::new();
    let mut last_pet = String::new();
    let mut last_boss = String::new();
    let mut last_summon = String::new();
    let mut last_swap = String::new();
    let mut last_fx = String::new();
    let mut last_block = String::new();
    let cross_at: Vec<u64> = std::env::var("NEOPETS_TRACE_CROSS").unwrap_or_default().split(',').filter_map(|t| t.trim().parse().ok()).collect();
    let switch_at: Vec<u64> = std::env::var("NEOPETS_TRACE_SWITCH").unwrap_or_default().split(',').filter_map(|t| t.trim().parse().ok()).collect();
    let mut flags_seen = play.state.flags.clone();
    // NEOPETS_TRACE_HOLD=n: the fight trace holds the attack n ticks (default 10; 50 or more
    // releases a power attack).
    let hold: u64 = std::env::var("NEOPETS_TRACE_HOLD").ok().and_then(|v| v.parse().ok()).unwrap_or(10).min(240);
    let mice = std::env::var_os("NEOPETS_TRACE_MICE").is_some();
    // NEOPETS_TRACE_PRESSES="a,b,...": attack press edges at these offsets of every 120 ticks
    // (the first held 10 ticks), for combos.
    let presses: Vec<u64> = std::env::var("NEOPETS_TRACE_PRESSES").unwrap_or_default().split(',').filter_map(|t| t.trim().parse().ok()).collect();
    // NEOPETS_TRACE_LEVEL="flag@tick,...": diagnostic level flags set on given ticks.
    let level_at: Vec<(u16, u64)> = std::env::var("NEOPETS_TRACE_LEVEL")
        .unwrap_or_default()
        .split(',')
        .filter_map(|t| {
            let (f, k) = t.trim().split_once('@')?;
            Some((f.parse().ok()?, k.parse().ok()?))
        })
        .collect();
    // NEOPETS_TRACE_ACT="stage@tick": a diagnostic TriggerActStart(stage) on the given tick.
    let act_at: Option<(i32, u64)> = std::env::var("NEOPETS_TRACE_ACT").ok().and_then(|v| {
        let (st, k) = v.trim().split_once('@')?;
        Some((st.parse().ok()?, k.parse().ok()?))
    });
    let mut last_pickup = String::new();
    let mut statuses_shown: Vec<usize> = Vec::new();
    // NEOPETS_TRACE_QUIT=n: Quit's world teardown on tick n (diagnostic; the pause page's Quit).
    let mut quit_at: Option<u64> = std::env::var("NEOPETS_TRACE_QUIT").ok().and_then(|v| v.trim().parse().ok());
    for _ in 0..ticks {
        // Once: the new game's tick count starts again. A game-over exit with nothing pending
        // tears the world down the same way.
        // NEOPETS_TRACE_KILL=n: the developer menu's enemy defeat on tick n.
        if std::env::var("NEOPETS_TRACE_KILL").ok().and_then(|v| v.trim().parse::<u64>().ok()) == Some(play.ticks) {
            let pk = play.world.pack;
            let alive: Vec<(String, bool)> = play.characters.iter().filter(|c| c.pack == pk && c.enemy.as_ref().is_some_and(|e| !e.dead)).map(|c| (c.name.clone(), c.inactive)).collect();
            println!("      enemies before (name, inactive): {alive:?}");
            let n = play.dev_kill_enemies();
            println!("      developer kill on tick {}: {n} enemies hit", play.ticks);
        }
        // NEOPETS_TRACE_RIG=Name: that character's main joint translations on tick 30.
        if play.ticks == 30 {
            if let Ok(name) = std::env::var("NEOPETS_TRACE_RIG") {
                if let Some(c) = play.characters.iter().find(|c| c.name == name) {
                    for (i, p, t, r) in c.anim.main_translations() {
                        println!("      {name} joint {i} parent {p:?}: sampled {t:?} rest {r:?}");
                    }
                }
            }
        }
        // NEOPETS_TRACE_BEND: every 20 ticks, the current world's most tilted bush (its up y and
        // position) and how many are tilted; on tick 1 the first bush positions.
        if std::env::var_os("NEOPETS_TRACE_BEND").is_some() {
            let bushes: Vec<([f32; 3], [f32; 3])> = play.foliage.get(&play.world.pack).into_iter().flatten().flat_map(|p| p.plants.iter().zip(&p.bushes).map(|(q, b)| (q.position, b.up))).collect();
            if play.ticks == 1 {
                let hero = play.loco.position();
                let mut near: Vec<_> = bushes.iter().map(|(q, _)| (((q[0] - hero[0]).hypot(q[2] - hero[2])), *q)).collect();
                near.sort_by(|a, b| a.0.total_cmp(&b.0));
                println!("      bushes {} nearest {:?}", bushes.len(), &near[..near.len().min(3)]);
            }
            if play.ticks % 20 == 0 {
                let tilted = bushes.iter().filter(|(_, u)| u[1] < 0.999).count();
                if let Some((q, u)) = bushes.iter().min_by(|a, b| a.1[1].total_cmp(&b.1[1])) {
                    println!("      bend tick {}: {tilted} tilted, most {:.3} at {:.0},{:.0}", play.ticks, u[1], q[0], q[2]);
                }
            }
        }
        // NEOPETS_TRACE_DEFENCE: on tick 60, each enemy's defence words (SPEC-0038 0xE0 to 0x10C).
        if play.ticks == 60 && std::env::var_os("NEOPETS_TRACE_DEFENCE").is_some() {
            for c in play.characters.iter().filter(|c| c.enemy.is_some()) {
                let d = c.enemy.as_ref().unwrap().p.defence;
                println!("      defence {} (pack {}): {d:?}", c.name, c.pack);
            }
        }
        // NEOPETS_TRACE_BREAK: on tick 60 the current world's breakables (count, kinds of
        // volume, the nearest three) and its switch groups.
        // NEOPETS_TRACE_NPCBLOCK=n: on tick n block 0 is moved onto the first walking character.
        if std::env::var("NEOPETS_TRACE_NPCBLOCK").ok().and_then(|v| v.trim().parse::<u64>().ok()) == Some(play.ticks) {
            println!("      block 0 moved onto character {:?} on tick {}", play.block_onto_npc(), play.ticks);
        }
        // NEOPETS_TRACE_BLOCK on tick 5: the current world's blocks and the volumes blocks fire.
        if play.ticks == 5 && std::env::var_os("NEOPETS_TRACE_BLOCK").is_some() {
            for line in play.block_survey() {
                println!("      {line}");
            }
        }
        // NEOPETS_TRACE_BLOCK: the hero's block state and the held block whenever they change.
        if std::env::var_os("NEOPETS_TRACE_BLOCK").is_some() {
            let now = play.block_trace();
            if now != last_block {
                println!("      tick {}: {now}", play.ticks);
                last_block = now;
            }
        }
        if play.ticks == 60 && std::env::var_os("NEOPETS_TRACE_BREAK").is_some() {
            let hero = play.loco.position();
            let mut near: Vec<(f32, usize, [f32; 3], Option<u8>, i32)> = play.world.breakables.iter().enumerate().map(|(i, b)| { let p = b.position(); (((p[0] - hero[0]).powi(2) + (p[2] - hero[2]).powi(2)).sqrt(), i, p, b.volume.map(|v| v.kind), b.state) }).collect();
            near.sort_by(|a, b| a.0.total_cmp(&b.0));
            println!("      breakables {} (meshes {}), nearest {:?}", play.world.breakables.len(), play.world.breakables.iter().filter(|b| b.mesh.is_some()).count(), &near[..near.len().min(3)]);
            // The switch groups (kind 4): each member with its position and state.
            for (gi, g) in play.world.groups.iter().enumerate().filter(|(_, g)| g.kind == 4) {
                let members: Vec<(i32, [f32; 3], i32)> = g.breakables.iter().map(|&m| (m, usize::try_from(m).ok().and_then(|m| play.world.breakables.get(m)).map_or([0.0; 3], |b| b.position()), usize::try_from(m).ok().and_then(|m| play.world.breakables.get(m)).map_or(-1, |b| b.state))).collect();
                println!("      switch group {gi}: {members:?}");
            }
        }
        // NEOPETS_TRACE_DROP=n: the developer menu's coin drop (with item 160) on tick n.
        if std::env::var("NEOPETS_TRACE_DROP").ok().and_then(|v| v.trim().parse::<u64>().ok()) == Some(play.ticks) {
            let n = play.dev_drop(160);
            println!("      developer drop on tick {}: {n} world items", play.ticks);
        }
        if quit_at.is_some_and(|k| k == play.ticks) || std::mem::take(&mut play.teardown_wanted) {
            quit_at = None;
            let held = |p: &play::Play| (p.world.pack, p.hero_index, p.hero_hp, p.state.flags.len(), p.state.neopoints, p.snapshot.is_some(), p.characters.len(), p.generation);
            println!("      Quit on tick {}: before (world, hero, hp, flags, NP, snapshot, characters, game) {:?}", play.ticks, held(&play));
            match new_game(&mut archive.disc, &archive.index, &play) {
                Ok(new) => {
                    play = new;
                    if let Ok(second) = archive.disc.reopen() {
                        streamer = Streamer::start(second, archive.index.clone(), play.boot.clone());
                    }
                    println!("      after {:?}; {}", held(&play), play.status());
                }
                Err(e) => println!("      Quit teardown failed: {e}"),
            }
        }
        // The trace waits for region loads (characters) before the next tick, up to a minute,
        // so that runs do not depend on the loader's speed.
        let waited = std::time::Instant::now();
        while streamer.regions_out > 0 && waited.elapsed().as_secs() < 60 {
            std::thread::sleep(std::time::Duration::from_millis(5));
            for line in streamer.pump(&mut play) {
                println!("      {line}");
            }
        }
        // `--trace-to x,z` steers the stick toward a point through the camera's rows.
        let (mut forward, mut side) = (0.7, 0.0);
        // After a swap along a `swap` route, go on to a cell that names a new neighbour.
        if play.swaps > 0 && to.is_some_and(|t| t[1].is_nan()) && route.len() <= 1 && !rerouted {
            rerouted = true;
            route = play.route_to(400_000, play::Goal::NewNeighbour).unwrap_or_default();
            println!("      rerouted on tick {} toward a new neighbour: {} steps", play.ticks, route.len());
        }
        if walk_path && play.job_pack().is_none() {
            // Arriving in another world ends the leg that led there (a door).
            if play.world.pack != leg_pack {
                leg_pack = play.world.pack;
                if legs.len() > 1 {
                    legs.remove(0);
                    if !leg_y.is_empty() {
                        leg_y.remove(0);
                    }
                }
                route.clear();
                println!("      waypoint left with the world on tick {}; {} left", play.ticks, legs.len() - 1);
            }
            let p = play.loco.position();
            if route.is_empty() && legs.len() > 1 {
                route = match leg_y.first().copied().flatten() {
                    Some(y) => play.route_to(200_000, play::Goal::PointAt([legs[0][0], legs[0][1], y])),
                    None => play.route_to(200_000, play::Goal::Point(legs[0])),
                }
                .unwrap_or_default();
                route.push(legs[0]);
            }
            while route.len() > 1 && (route[0][0] - p[0]).powi(2) + (route[0][1] - p[2]).powi(2) < 4.0 {
                route.remove(0);
            }
            let near = route.len() == 1 && (route[0][0] - p[0]).powi(2) + (route[0][1] - p[2]).powi(2) < 4.0;
            dwell = if near && !play.dialogue_open { dwell + 1 } else { 0 };
            if dwell >= 30 && legs.len() > 1 {
                dwell = 0;
                legs.remove(0);
                if !leg_y.is_empty() {
                    leg_y.remove(0);
                }
                route.clear();
                println!("      waypoint reached on tick {}; {} left", play.ticks, legs.len() - 1);
            }
            if route.is_empty() {
                route.push(legs[0]);
            }
        }
        while !walk_path && route.len() > 1 && {
            let p = play.loco.position();
            (route[0][0] - p[0]).powi(2) + (route[0][1] - p[2]).powi(2) < 4.0
        } {
            route.remove(0);
        }
        // NEOPETS_TRACE_FOLLOW=<name>: keep within 4.5 units of the character with that name.
        if let Ok(name) = std::env::var("NEOPETS_TRACE_FOLLOW") {
            // The name "wild" follows the first wild petpet, to within 3 units.
            let wild = name == "wild";
            if let Some(c) = play.characters.iter().find(|c| (c.name == name || wild && c.wild.is_some()) && c.pack == play.world.pack) {
                let p = play.loco.position();
                let t = [c.position[0], c.position[2]];
                if (t[0] - p[0]).hypot(t[1] - p[2]) <= if wild { 3.0 } else { 4.5 } {
                    route.clear();
                } else if route.last().is_none_or(|r| (r[0] - t[0]).hypot(r[1] - t[1]) > 4.0) {
                    // A searched route to the character, refreshed when it has moved on.
                    // NEOPETS_TRACE_Y=y: the target is on the floor near that height.
                    let goal = match std::env::var("NEOPETS_TRACE_Y").ok().and_then(|v| v.parse::<f32>().ok()) {
                        Some(y) => play::Goal::PointAt([t[0], t[1], y]),
                        None => play::Goal::Point(t),
                    };
                    route = play.route_to(200_000, goal).unwrap_or_default();
                    println!("      following {name} at {:?}: {} route steps", c.position, route.len());
                    route.push(t);
                }
            }
        }
        let herd = to.is_some_and(|t| t[1] == 2.0);
        if let Some((st, k)) = act_at.filter(|&(_, k)| play.ticks == k) {
            let before = (play.state.neopoints, play.state.items.len());
            play.act_start(st);
            let mut kit: Vec<(u16, u32)> = play.state.items.iter().map(|(&i, &n)| (i, n)).collect();
            kit.sort();
            println!("      diagnostic: act start {st} on tick {k}: hero {}, NP {} -> {}, party {} -> {kit:?}, stash {} items and {} NP", play.hero_index, before.0, play.state.neopoints, before.1, play.state.stash.len(), play.state.stash_neopoints);
        }
        for &(f, k) in &level_at {
            if play.ticks == k {
                play.state.set_level(f, true);
                println!("      diagnostic: level flag {f} set on tick {k}");
            }
        }
        let fight = to.is_some_and(|t| t[1] == 3.0).then(|| play.fight_goal()).flatten();
        // NEOPETS_TRACE_MICE: while carrying a Miamouse, walk to the cage's middle; otherwise
        // sneak (Square held) up behind the nearest catchable one.
        let cage = mice.then(|| play.cage_goal()).flatten();
        let stalk = if mice && cage.is_none() { play.mouse_goal() } else { None };
        let goal = if let Some(c) = cage {
            Some(c)
        } else if let Some(g) = stalk {
            Some(g)
        } else if herd {
            play.herd_goal()
        } else if let Some((g, close)) = fight {
            // Roberta keeps turning toward her target when close (a slow step) so her shots,
            // which fly along her facing, can reach it.
            (!close || play.hero_index == 1).then_some(g)
        } else {
            route.first().copied().or(to.filter(|t| !t[0].is_nan()))
        };
        if let Some([tx, tz]) = goal.filter(|g| {
            let p = play.loco.position();
            (route.len() > 1 && !walk_path) || to.is_some_and(|t| t[0].is_nan()) || (g[0] - p[0]).hypot(g[1] - p[2]) > if walk_path { 1.5 } else { 2.0 }
        }) {
            let p = play.loco.position();
            let (f, l) = play.camera.rows();
            let (dx, dz) = (tx - p[0], tz - p[2]);
            let len = (dx * dx + dz * dz).sqrt().max(1e-6);
            // Close, Roberta only turns: a slow step while her facing is off the target, none
            // once it is within about 8 degrees.
            let aligned = (dx * play.loco.facing[0] + dz * play.loco.facing[2]) / (dx * dx + dz * dz).sqrt().max(1e-6) > 0.99;
            let m = if fight.is_some_and(|f| f.1) && play.hero_index == 1 { if aligned { 0.0 } else { 0.15 } } else { 0.7 };
            forward = m * (dx * f[0] + dz * f[2]) / len / (f[0] * f[0] + f[2] * f[2]).sqrt().max(1e-6);
            side = m * (dx * l[0] + dz * l[2]) / len;
        }
        // NEOPETS_TRACE_PUSH="x,z": the stick points straight at the point (no route search),
        // except while climbing onto a block (centred) and on a ladder (pushed up).
        if let Some((x, z)) = std::env::var("NEOPETS_TRACE_PUSH").ok().and_then(|v| { let (a, b) = v.split_once(',')?; Some((a.trim().parse::<f32>().ok()?, b.trim().parse::<f32>().ok()?)) }) {
            (forward, side) = match play.loco.state {
                58 => (0.0, 0.0),
                41 => (1.0, 0.0),
                _ => play.stick_toward(x, z),
            };
        }
        // NEOPETS_TRACE_BLOCK: in the block state the stick points into the held face (centred
        // during the approach), so the hero pushes; the Square prompts are pressed as usual.
        if std::env::var_os("NEOPETS_TRACE_BLOCK").is_some() && play.loco.state == neopets_sim::hero::BLOCK {
            (forward, side) = play.block_trace_stick().unwrap_or((0.0, 0.0));
        }
        // NEOPETS_TRACE_PLACE="tick:x,z[,probe];...": the hero placed there on that tick
        // (diagnostic).
        if let Ok(v) = std::env::var("NEOPETS_TRACE_PLACE") {
            for e in v.split(';') {
                let Some((k, r)) = e.trim().split_once(':') else { continue };
                let n: Vec<f32> = r.split(',').filter_map(|t| t.trim().parse().ok()).collect();
                if k.parse::<u64>().ok() == Some(play.ticks) && n.len() >= 2 {
                    play.place_at(n[0], n[1], n.get(2).copied().unwrap_or(1.0e4));
                }
            }
        }
        // NEOPETS_TRACE_BOSSHP="tick:hp;...": the Gelert Assassin's HP set on that tick.
        if let Ok(v) = std::env::var("NEOPETS_TRACE_BOSSHP") {
            for e in v.split(';') {
                if let Some((k, h)) = e.trim().split_once(':') {
                    if k.parse::<u64>().ok() == Some(play.ticks) {
                        if let Ok(h) = h.parse::<i32>() {
                            play.diag_gelert_hp(h);
                        }
                    }
                }
            }
        }
        // NEOPETS_TRACE_SLIDE="tick:k:x,z;...": from that tick, block k slides to (x, z) at 0.2
        // units a tick, firing the volumes it touches (diagnostic).
        if let Ok(v) = std::env::var("NEOPETS_TRACE_SLIDE") {
            for e in v.split(';') {
                let parts: Vec<&str> = e.trim().split(':').collect();
                if let [t, k, xz] = parts[..] {
                    if t.parse::<u64>().ok() == Some(play.ticks) {
                        if let (Ok(k), Some((x, z))) = (k.parse::<usize>(), xz.split_once(',')) {
                            if let (Ok(x), Ok(z)) = (x.trim().parse::<f32>(), z.trim().parse::<f32>()) {
                                play.diag_slides.push((k, [x, z]));
                            }
                        }
                    }
                }
            }
        }
        // NEOPETS_TRACE_STICK="tick:f,s;tick:f,s;...": from each tick on, the raw stick values
        // (forward, side) replace the route's, until the next entry; "tick:-" returns to the
        // route. NEOPETS_TRACE_SQUARE="a,b,...": Square press edges on these ticks.
        if let Ok(v) = std::env::var("NEOPETS_TRACE_STICK") {
            let mut set: Option<(f32, f32)> = None;
            for e in v.split(';') {
                let Some((k, fs)) = e.trim().split_once(':') else { continue };
                if k.parse::<u64>().is_ok_and(|k| k <= play.ticks) {
                    set = fs.split_once(',').and_then(|(f, s)| Some((f.trim().parse().ok()?, s.trim().parse().ok()?)));
                }
            }
            if let Some(fs) = set {
                (forward, side) = fs;
            }
        }
        // Around a diagnostic switch the stick rests (the chord needs a standing hero).
        if switch_at.iter().any(|&k| play.ticks + 30 >= k && play.ticks <= k + 60) {
            (forward, side) = (0.0, 0.0);
        }
        let input = neopets_sim::hero::HeroInput {
            forward,
            side,
            // An open dialogue box gets a Square press every 10 ticks (skip, Next, Finish).
            // With NEOPETS_TRACE_MICE the drop prompt is pressed only inside the cage, and a
            // stalk starts the sneak with one press.
            square_held: stalk.is_some(),
            // NEOPETS_TRACE_CLIMB: the grab-block (10) and stop-climbing (18) prompts are not pressed, so the hero keeps
            // walking into the block.
            square: (play.prompt_shown.is_some() && !(mice && play.prompt_shown == Some(28) && !play.hero_in_cage()) && !(matches!(play.prompt_shown, Some(10 | 18)) && std::env::var_os("NEOPETS_TRACE_CLIMB").is_some()))
                || (stalk.is_some() && !matches!(play.loco.state, 22 | 40))
                || play.talk_prompt || (play.buyable.is_some() && play.ticks % 30 == 0) || (play.save_prompt && play.ticks % 10 == 0) || (play.dialogue_open && play.ticks % 10 == 0),
            guard: guard && fight.is_some_and(|f| f.1),
            // With `--autojump` the trace also presses attack every 120 ticks for 10 ticks.
            attack_held: ((jump || fight.is_some_and(|f| f.1)) && (30..30 + hold).contains(&(play.ticks % 120)))
                || presses.first().is_some_and(|&p| (p..p + 10).contains(&(play.ticks % 120))),
            // A question box gets Circle (yes) every 10 ticks; NEOPETS_TRACE_PRESSES adds press
            // edges at its tick offsets in every 120 (a combo).
            attack: ((jump || fight.is_some_and(|f| f.1)) && play.ticks % 120 == 30) || (play.textbox.asking() && play.ticks % 10 == 5) || presses.contains(&(play.ticks % 120)),
            // On the game-over page the trace taps Cross every 60 ticks.
            // NEOPETS_TRACE_CROSS="a,b,...": Cross press edges on these ticks.
            jump: (jump && play.ticks % 240 == 0 && play.ticks > 0) || (play.death.page.is_some() && play.ticks % 60 == 0) || cross_at.contains(&play.ticks),
            ..Default::default()
        };
        let mut input = input;
        if std::env::var("NEOPETS_TRACE_SQUARE").is_ok_and(|v| v.split(',').any(|t| t.trim().parse::<u64>().ok() == Some(play.ticks))) {
            input.square = true;
        }
        // In a fight below half health the trace drinks: it opens the quick panel, selects the
        // first entry it owns and presses Cross (or closes the panel with Triangle).
        let mut input = input;
        if fight.is_some() && play.hero_hp > 0 && play.hero_hp < play.hero_max / 2 {
            let owned = (0..4).find(|&k| {
                let id = play.quick.entries[k];
                id >= 0 && play.state.items.get(&(id as u16)).is_some_and(|&n| n > 0)
            });
            if !play.quick.open {
                play.panel_latch.up |= play.ticks % 20 == 0;
            } else {
                match owned {
                    Some(k) if play.quick.selection == k => input.jump = true,
                    Some(0) => play.panel_latch.right = true,
                    Some(1) => play.panel_latch.up = true,
                    Some(2) => play.panel_latch.left = true,
                    Some(_) => play.panel_latch.down = true,
                    None => play.panel_latch.triangle = true,
                }
            }
        }
        play.tick(&input, neopets_sim::camera::CameraPad::default());
        // Loads go through the same background loader as the window; the trace does not wait.
        for line in streamer.pump(&mut play) {
            println!("      {line}");
        }
        if let Some((line, at)) = play.message {
            if at == play.ticks {
                let text = play.world.lines.line(line as usize).map(|l| format!("{}: {}", play.talk_partner.as_deref().unwrap_or(&l.speaker), l.text));
                println!("      LevelMsg line {line} on tick {at}: {text:?}");
            }
        }
        let now_scene = play.scene_status();
        if now_scene != scene {
            println!("      scene on tick {}: {now_scene}", play.ticks);
            scene = now_scene;
        }
        let now: Vec<usize> = (0..14).filter(|&i| play.statuses.active(i)).collect();
        if now != statuses_shown {
            println!("      hero statuses {now:?} on tick {} (hp {}, mp {})", play.ticks, play.hero_hp, play.hero_mp);
            statuses_shown = now;
        }
        if !play.last_pickup.is_empty() && play.last_pickup != last_pickup {
            last_pickup = play.last_pickup.clone();
            println!("      {last_pickup}");
        }
        if !play.last_hurt.is_empty() && play.last_hurt != last_hurt {
            last_hurt = play.last_hurt.clone();
            println!("      hero {last_hurt} on tick {}", play.ticks);
        }
        if !play.last_hit.is_empty() && play.last_hit != last_hit {
            last_hit = play.last_hit.clone();
            println!("      {last_hit} on tick {}", play.ticks);
        }
        if !play.last_mote.is_empty() && play.last_mote != last_mote {
            last_mote = play.last_mote.clone();
            println!("      {last_mote}");
        }
        if !play.last_status.is_empty() && play.last_status != last_status {
            last_status = play.last_status.clone();
            println!("      {last_status}");
        }
        if !play.last_pet.is_empty() && play.last_pet != last_pet {
            last_pet = play.last_pet.clone();
            println!("      {last_pet}");
        }
        if play.state.flags != flags_seen {
            let on: Vec<u16> = play.state.flags.difference(&flags_seen).copied().collect();
            let off: Vec<u16> = flags_seen.difference(&play.state.flags).copied().collect();
            println!("      flags on {on:?} off {off:?} on tick {}", play.ticks);
            flags_seen = play.state.flags.clone();
        }
        if play.ticks == 2 || (play.ticks % 600 == 0 && play.ticks > 0) {
            let w = &play.world;
            let list: Vec<String> = w.object_anims.iter().enumerate().filter(|(_, o)| o.scripted).map(|(i, o)| format!("{i}:{}{}", o.state, if play.object_matrix(w.pack, i).is_some() { "" } else { " hidden" })).collect();
            let shown = (0..w.objects.len()).filter(|&i| play.object_matrix(w.pack, i).is_some()).count();
            println!("      objects of {} ({} in all, {shown} drawn) on tick {}: scripted [{}]", w.pack, w.objects.len(), play.ticks, list.join(", "));
            println!("      hero lights ({} light-set records): {}", w.lights.len(), play.light_summary());
            let on: Vec<usize> = play.foliage.get(&w.pack).map(|p| p.iter().map(|q| usize::from(q.on)).collect()).unwrap_or_default();
            println!("      placed records on: [{}]", play.placed_summary());
            let off = play.characters.iter().filter(|c| c.inactive && !c.deactivated).count();
            println!("      zones: grid {} row {} key {}; patches on {} of {}; characters {} ({off} cell-off)", w.zones.as_ref().map_or(0, |g| g.blocks.len()), play.zones.row, play.zones.key, on.iter().sum::<usize>(), on.len(), play.characters.len());
        }
        if play.state.timer.visible && play.ticks % 120 == 0 {
            println!("      timer on tick {}: {:.3} s ({}, limit {})", play.ticks, play.state.timer.value, if play.state.timer.up { "up" } else { "down" }, play.state.timer.limit);
        }
        if let Some(o) = play.shake_now.filter(|_| play.ticks % 10 == 0) {
            println!("      shake on tick {}: side {:.4} up {:.4} aim {:?} (amplitude {:.3})", play.ticks, o.side, o.up, o.aim, play.shake.amplitude);
        }
        if !play.last_fx.is_empty() && play.last_fx != last_fx {
            last_fx = play.last_fx.clone();
            println!("      {last_fx}");
        }
        if !play.last_swap.is_empty() && play.last_swap != last_swap {
            last_swap = play.last_swap.clone();
            println!("      {last_swap}");
        }
        // NEOPETS_TRACE_STATUS="code,seconds": the status on the hero at tick 60, and the hero's
        // colour-slot vector every 15 ticks while it is not white.
        if let Some((code, secs)) = std::env::var("NEOPETS_TRACE_STATUS").ok().and_then(|v| {
            let (a, b) = v.split_once(',')?;
            Some((a.trim().parse::<i32>().ok()?, b.trim().parse::<i32>().ok()?))
        }) {
            if play.ticks == 60 {
                play.statuses.apply(code, secs);
            }
            if let Some(t) = play.status_tint(None).filter(|_| play.ticks % 15 == 0) {
                println!("      hero tint ({:.3}, {:.3}, {:.3}) on tick {}", t[0], t[1], t[2], play.ticks);
            }
        }
        // NEOPETS_TRACE_SWITCH="tick,...": the chord (L1 held, R1 newly pressed) on those ticks.
        if switch_at.contains(&(play.ticks + 1)) {
            play.l1_latch = true;
            play.r1_latch = true;
        }
        if !play.last_summon.is_empty() && play.last_summon != last_summon {
            last_summon = play.last_summon.clone();
            println!("      {last_summon}");
        }
        if !play.last_boss.is_empty() && play.last_boss != last_boss {
            last_boss = play.last_boss.clone();
            println!("      {last_boss}");
        }
        if !play.last_door.is_empty() && play.last_door != last_door {
            last_door = play.last_door.clone();
            println!("      {} on tick {}", last_door, play.ticks);
            // A door trace goes on through door 0 of each world it arrives in, twice more.
            let arrived = last_door.starts_with("landed") || last_door.starts_with("entered") || last_door.contains("into the") || last_door.contains("left the");
            if arrived && to.is_some_and(|t| t[1] == 1.0) && hops < 2 {
                hops += 1;
                route = play.stand_before_door(0).map(|m| vec![m]).unwrap_or_default();
                println!("      hop {hops}: standing before door 0 of {}", play.world.pack);
            }
        }
        let (p, b) = (play.loco.position(), &play.loco.body);
        println!(
            "{:5} pos ({:9.3} {:8.3} {:9.3}) state {:2} phase {:2} action {:3} gap {:8.3} vy {:7.3} n ({:.2} {:.2} {:.2})",
            play.ticks, p[0], p[1], p[2], play.loco.state, play.loco.phase, play.loco.action(), b.gap, b.velocity[1],
            b.normal[0], b.normal[1], b.normal[2]
        );
        // NEOPETS_TRACE_FACING: the facing, the stick target and the body velocity too.
        if std::env::var_os("NEOPETS_TRACE_FACING").is_some() {
            println!("      facing {:?} target {:?} velocity {:?} root {:?} camera rows {:?}", play.loco.facing, play.loco.target, b.velocity, play.root_step, play.camera.rows()); println!("      eye {:?} focus {:?}", play.camera.eye, play.camera.focus);
        }
        if play.ticks % 60 == 0 {
            println!("      {}", play.status_camera());
            println!("      {}", play.status_worlds());
        }
        // The trace has no audio side: every slot's block counts as loaded, and the cue
        // requests are printed (one-shot plays only).
        let slots: Vec<u32> = play.sound.slots.iter().flatten().copied().collect();
        play.sound.loaded.extend(slots);
        let plays: Vec<String> = std::mem::take(&mut play.sound.requests)
            .into_iter()
            .filter_map(|r| match r {
                neopets_sim::sound::Request::Play { block, cue, volume, pan, .. } => Some(format!("{block}:{cue} at {volume}{}", pan.map_or(String::new(), |p| format!(" pan {p}")))),
                _ => None,
            })
            .collect();
        if !plays.is_empty() {
            println!("      cues {plays:?} on tick {}", play.ticks);
        }
        let emotes = std::mem::take(&mut play.emote_requests);
        if let Some(&(bank, start, len)) = emotes.first() {
            let Archive { disc, index } = &mut *archive;
            let clip = index.read(disc, bank).ok().and_then(|b| b.get(start as usize * 2048..(start + len) as usize * 2048).map(neopets_data::audio::vag));
            let info = match clip {
                Some(Ok((pcm, name))) => format!("{name:?}, {:.2} s", pcm.seconds()),
                Some(Err(e)) => e,
                None => "out of range".into(),
            };
            println!("      emote clips {emotes:?} on tick {}: {info}", play.ticks);
        }
        if play.dialogue_open && play.ticks % 15 == 0 {
            let partner = play.talk_partner_view();
            println!("      talking: hero main {} | partner {partner:?}", play.hero.action());
        }
        let stingers = std::mem::take(&mut play.stinger_requests);
        if !stingers.is_empty() {
            println!("      stingers {stingers:?} on tick {}", play.ticks);
        }
        play.stinger_playing = false;
        if play.save_prompt != save_prompt {
            save_prompt = play.save_prompt;
            println!("      save prompt {} on tick {}", if save_prompt { "shown" } else { "hidden" }, play.ticks);
        }
        if std::mem::take(&mut play.save_open) {
            println!("      save screen opened on tick {}; restart point {:?}", play.ticks, play.restart);
        }
    }
    let mut held: Vec<(u16, u32)> = play.state.items.iter().map(|(k, v)| (*k, *v)).collect();
    held.sort_unstable();
    println!("Held items at the end: {held:?}; {} NP; weapon {:?}, shield {:?}, entries {:?}", play.state.neopoints, play.weapon, play.shield, play.equipment.entries);
    if save_test {
        let path = "private/saves/trace.bin";
        let mut file = read_save(path).unwrap_or_default();
        play.write_slot(&mut file.slots[0], &save_date());
        let s = &file.slots[0];
        println!(
            "Saved slot 0: keys {:?}, location {:?}, date {:?}, tier {}, wallet {}, pools {:?}, maxima {:?}, entries {:?}",
            [s.key(0), s.key(1), s.key(2)],
            String::from_utf8_lossy(s.location()),
            String::from_utf8_lossy(s.date()),
            s.tier(),
            s.i32(neopets_sim::save::at::WALLET),
            s.pools(0),
            s.maximum_words(0),
            s.adds().iter().map(|a| (a.0, a.1, a.3, a.4)).collect::<Vec<_>>()
        );
        if let Err(e) = write_save(path, &file) {
            eprintln!("Save failed: {e}");
            std::process::exit(1);
        }
        let back = read_save(path).expect("the file just written reads back");
        assert_eq!(back, file);
        println!("Before: {}", play.status());
        match load_game(archive, &play, &back.slots[0]) {
            Ok(loaded) => {
                println!("Loaded: {}", loaded.status());
                println!("        {}", loaded.status_worlds());
                let summary = |p: &play::Play| {
                    let mut inv: Vec<_> = p.state.items.iter().map(|(k, v)| (*k, *v)).collect();
                    inv.sort();
                    format!("weapon {:?}, shield {:?}, items {inv:?}, flags {:?}, luck {}, statuses {:?}, restart {:?}, collected {:?}, maps {:?}", p.weapon, p.shield, p.state.flags, p.luck, p.statuses.timers, p.restart, p.state.collected, p.state.maps)
                };
                println!("Round trip before: {}", summary(&play));
                println!("Round trip after:  {}", summary(&loaded));
            }
            Err(e) => eprintln!("Load failed: {e}"),
        }
    }
}

fn setup_play(mut commands: Commands, request: Option<Res<PlayRequest>>, archive: Option<ResMut<Archive>>) {
    let (Some(request), Some(mut archive)) = (request, archive) else { return };
    let Archive { disc, index } = &mut *archive;
    // `--flag N` (repeatable): diagnostic story flags set before the start world's Init.
    match play::Play::load_with_flags(disc, index, request.0, PLAY_OUTFIT, &request.4) {
        Ok(mut play) => {
            // `--start x,z[,y]`: diagnostic start point; `--look x,z`: diagnostic start facing.
            if let Some([x, z, c]) = request.5 {
                play.place_at(x, z, c);
            }
            if let Some([x, z]) = request.1 {
                play.face_point(x, z);
            }
            // `--give N`: diagnostic equipment (no inventory yet).
            for &id in &request.2 {
                play.give(id);
            }
            // NEOPETS_TRACE_PET="type,T" and NEOPETS_TRACE_QUICK=id as in the trace (diagnostic).
            if let Ok(v) = std::env::var("NEOPETS_TRACE_PET") {
                let mut it = v.split(',').filter_map(|t| t.trim().parse::<i32>().ok());
                if let (Some(ty), Some(t)) = (it.next(), it.next()) {
                    play.activate_pet(ty, t, t);
                }
            }
            if let Some(id) = std::env::var("NEOPETS_TRACE_QUICK").ok().and_then(|v| v.parse::<i32>().ok()) {
                play.quick.marker = id;
            }
            // The HUD's art: the front-end database and texture sheet of resource 532.
            match index.read(disc, neopets_data::frontend::RESOURCE).and_then(|b| neopets_data::frontend::Frontend::parse(&b)) {
                Ok(f) => {
                    // The front-end flow: the card state with `--title`, else play directly.
                    let mut flow = menus::Flow::new(&f);
                    if !request.3 {
                        flow.set(&f, menus::PLAY);
                        // A direct play start counts as past the title for the music gate.
                        flow.title_seen = true;
                    }
                    // SPEC-0035 section 14.3: the row drawer's words (wrap 0x2DCDA8, pulse
                    // 0x2DCDB4 to 0x2DCDBC, bar width 0x2DCDC0, colours 0x2DCDC8 and 0x2DCDD0).
                    match neopets_data::archive::verified_boot_program(disc).and_then(|b| neopets_data::archive::boot_u32_table(&b, 0x002D_CDA8, 11)) {
                        Ok(w) => {
                            flow.rows = menus::RowWords {
                                wrap: w[0] as i32,
                                pulse_min: f32::from_bits(w[3]),
                                pulse_max: f32::from_bits(w[4]),
                                pulse_step: f32::from_bits(w[5]),
                                bar_width: w[6] as f32,
                                title_colour: w[8].to_le_bytes(),
                                text_colour: w[10].to_le_bytes(),
                            };
                        }
                        Err(e) => error!("No row drawer words: {e}"),
                    }
                    // SPEC-0025 Card message records: table 9 of resource 533, the substitution
                    // slots (0: card slot "1"; 1: the needed KB, 106 by the card side's rule) and
                    // the busy hold, trunc(seconds word 0x2DD410 x 60).
                    if let Ok(d) = index.read(disc, neopets_data::design::RESOURCE).and_then(|b| neopets_data::design::DesignTables::parse(&b)) {
                        flow.messages = (0..27).map(|i| d.message(i).unwrap_or_default()).collect();
                    }
                    flow.slots = vec![b"1".to_vec(), b"106".to_vec()];
                    // SPEC-0026 Credits: table 15's rows, the colour table at 0x2DCFA8 (two sets
                    // of 8 words) and the roll's words at 0x2DCF84 to 0x2DCFA0.
                    if let Ok(d) = index.read(disc, neopets_data::design::RESOURCE).and_then(|b| neopets_data::design::DesignTables::parse(&b)) {
                        flow.credit_rows = d.credits();
                    }
                    if let Ok(boot) = neopets_data::archive::verified_boot_program(disc) {
                        if let Ok(c) = neopets_data::archive::boot_u32_table(&boot, 0x002D_CFA8, 16) {
                            flow.credit_colours = c.iter().map(|w| w.to_le_bytes()).collect();
                        }
                        // SPEC-0035 section 14.1: the item screen's 20 grid ids at 0x2DCF28.
                        if let Ok(g) = neopets_data::archive::boot_u32_table(&boot, 0x002D_CF28, 20) {
                            for (k, id) in g.iter().enumerate() {
                                flow.grid_ids[k] = *id as i32;
                            }
                        }
                        if let Ok(w) = neopets_data::archive::boot_u32_table(&boot, 0x002D_CF84, 8) {
                            flow.credit_words = Some(neopets_sim::credits::Words {
                                pitch: f32::from_bits(w[0]),
                                slow: f32::from_bits(w[1]),
                                fast: f32::from_bits(w[2]),
                                drop: -(w[3] as i32),
                                offset: w[4] as i32,
                                fade_top: w[5] as i32,
                                fade_bottom: w[6] as i32,
                                ramp: f32::from_bits(w[7]),
                            });
                        }
                    }
                    if let Ok(w) = neopets_data::archive::verified_boot_program(disc).and_then(|b| neopets_data::archive::boot_u32_table(&b, 0x002D_D410, 1)) {
                        flow.busy_hold = (f32::from_bits(w[0]) * 60.0) as u32;
                    }
                    // SPEC-0051: the quest, treasure and area-name tables, the pulse words and
                    // the two label colours.
                    let (quests, treasures, names, collected_areas) = play.map_tables();
                    flow.map_data = menus::MapData { quests, treasures, names, collected_areas, ..Default::default() };
                    if let Ok(boot) = neopets_data::archive::verified_boot_program(disc) {
                        if let Ok(w) = neopets_data::archive::boot_u32_table(&boot, neopets_sim::mappage::PULSE_WORDS, 8) {
                            flow.map_data.pulse = neopets_sim::mappage::PulseWords::read(&w).unwrap_or_default();
                            flow.map_data.label_colour = w[2].to_le_bytes();
                        }
                        if let Ok(w) = neopets_data::archive::boot_u32_table(&boot, 0x002D_D010, 1) {
                            flow.map_data.name_colour = w[0].to_le_bytes();
                        }
                        // SPEC-0035 section 6.1: the page timer's start and minimum.
                        if let Ok(w) = neopets_data::archive::boot_u32_table(&boot, 0x002D_B2E0, 2) {
                            flow.book_words = (f32::from_bits(w[0]), f32::from_bits(w[1]));
                        }
                        // Rule 5: the block x, y and line-limit words and the text width.
                        if let Ok(w) = neopets_data::archive::boot_u32_table(&boot, neopets_sim::book::PAGE_WORDS, 9) {
                            flow.page_words = neopets_sim::book::PageWords::read(&w);
                        }
                    }
                    commands.insert_resource(menu_view::Menu::new(flow));
                    if let Some(art) = hud_view::HudArt::new(f) {
                        commands.insert_resource(art);
                    }
                }
                Err(e) => error!("No HUD art: {e}"),
            }
            match disc.reopen() {
                Ok(second) => commands.insert_resource(Streamer::start(second, index.clone(), play.boot.clone())),
                Err(e) => error!("No loader thread, streaming is off: {e}"),
            }
            commands.insert_resource(PlayState { play, skinned: Vec::new() })
        }
        Err(e) => error!("Play mode could not start: {e}"),
    }
}

/// Spawn the hero once, then each frame place it, skin it and keep the camera on it.
#[allow(clippy::too_many_arguments)]
fn play_frame(
    mut commands: Commands,
    play: Option<ResMut<PlayState>>,
    mut heroes: Query<&mut Transform, (With<HeroRoot>, Without<Orbit>)>,
    hero_roots: Query<Entity, With<HeroRoot>>,
    mut cameras: Query<(&mut Orbit, &mut Transform), Without<HeroRoot>>,
    mut projections: Query<&mut Projection>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
) {
    let Some(mut state) = play else { return };
    let p = state.play.loco.position();
    let f = state.play.loco.facing;
    // Characters face +Z in model space (SPEC-0004); turn that onto the facing.
    let rotation = Quat::from_rotation_y(f[0].atan2(f[2]));
    if !state.play.spawned {
        state.play.spawned = true;
        // A hero switch re-spawns the hero with the incoming model.
        for e in &hero_roots {
            commands.entity(e).despawn();
        }
        let parts = part_assets(&state.play.model, &mut meshes, &mut materials, &mut images);
        let mut skinned = Vec::new();
        commands
            .spawn((Transform::from_translation(Vec3::from(p)).with_rotation(rotation), Visibility::Visible, HeroRoot))
            .with_children(|c| {
                for (i, mesh, material) in parts {
                    if !state.play.model.parts[i].weights.is_empty() {
                        skinned.push((mesh.clone(), i));
                    }
                    c.spawn((Mesh3d(mesh), MeshMaterial3d(material), Transform::IDENTITY));
                }
            });
        state.skinned = skinned;
        // SPEC-0027 Projection: 50.2 degrees vertical (S 512 on 480 lines), near 1.0. The far
        // plane is SPEC-0046's widest (10560) so the sky dome is kept.
        for mut projection in &mut projections {
            *projection = Projection::Perspective(PerspectiveProjection {
                fov: neopets_sim::camera::vertical_fov(),
                near: neopets_sim::camera::NEAR,
                far: 10_560.0,
                ..default()
            });
        }
    }
    for mut t in &mut heroes {
        t.translation = Vec3::from(p);
        t.rotation = rotation;
    }
    let matrices = state.play.hero.skin_matrices();
    let tint = state.play.status_tint(None);
    let lights = state.play.lights(None, p);
    for (handle, i) in &state.skinned {
        let part = &state.play.model.parts[*i];
        let (pos, nrm) = character::skin_part(&matrices, &part.positions, &part.normals, &part.weights, &part.bones);
        let flash = (state.play.hero_flash > 0).then_some(&play::FLASH_COLOUR);
        let colours = tinted(part, relight(part, &nrm, rotation, &lights), tint, flash);
        if let Some(mut mesh) = meshes.get_mut(handle) {
            mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, pos);
            mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, nrm);
            if let Some(c) = colours {
                mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, c);
            }
        }
    }
    // The play's view: a camera play's track pose, else the follow camera (SPEC-0027). The
    // vertical field of view is 2 atan(240 / focal) on the 480-line screen.
    let (eye, forward, up, focal) = state.play.view();
    for (_, mut transform) in &mut cameras {
        *transform = Transform::from_translation(Vec3::from(eye)).looking_to(Vec3::from(forward), Vec3::from(up));
    }
    let fov = 2.0 * (neopets_sim::camera::SCREEN.1 / (2.0 * focal.max(1.0))).atan();
    for mut projection in &mut projections {
        if let Projection::Perspective(pp) = projection.as_mut() {
            if (pp.fov - fov).abs() > 1e-6 {
                pp.fov = fov;
            }
        }
    }
}

/// The screen effect's full-screen fade and letterbox bars (SPEC-0052 Screen effect, draw step
/// 5: the bars, then the fade on top), above the HUD and the menus.
#[derive(Component)]
struct FadeOverlay;

#[derive(Component)]
struct LetterBar;

fn sync_fade(
    mut commands: Commands,
    play: Option<Res<PlayState>>,
    mut overlay: Query<(&mut BackgroundColor, &mut Visibility), With<FadeOverlay>>,
    mut bars: Query<(&mut Node, &mut Visibility), (With<LetterBar>, Without<FadeOverlay>)>,
) {
    let Some(state) = play else { return };
    if bars.is_empty() {
        for top in [true, false] {
            let mut node = Node { position_type: PositionType::Absolute, width: Val::Percent(100.0), height: Val::Px(0.0), ..default() };
            if top {
                node.top = Val::Px(0.0);
            } else {
                node.bottom = Val::Px(0.0);
            }
            commands.spawn((node, BackgroundColor(Color::BLACK), Visibility::Hidden, GlobalZIndex(2999), LetterBar));
        }
    }
    let h = state.play.bars.height() / 448.0 * 100.0;
    for (mut node, mut vis) in &mut bars {
        node.height = Val::Percent(h);
        *vis = if h > 0.0 { Visibility::Inherited } else { Visibility::Hidden };
    }
    if overlay.is_empty() {
        commands.spawn((
            Node { position_type: PositionType::Absolute, width: Val::Percent(100.0), height: Val::Percent(100.0), ..default() },
            BackgroundColor(Color::NONE),
            Visibility::Hidden,
            GlobalZIndex(3000),
            FadeOverlay,
        ));
        return;
    }
    let f = &state.play.fade;
    for (mut colour, mut vis) in &mut overlay {
        let a = f.opacity();
        *vis = if a > 0.0 { Visibility::Inherited } else { Visibility::Hidden };
        let c = f.colour.map(|v| v as f32 / 255.0);
        colour.0 = Color::srgba(c[0], c[1], c[2], a);
    }
}
