mod archive_view;
mod bindings;
mod coop;
mod net;
mod button_style;
mod controls_page;
mod character;
mod controls;
mod debug_tools;
mod item_gallery;
mod dev_menu;
mod dev_spawn;
mod stage_view;
mod story_run;
mod states_menu;
mod ui_fit;
mod display;
mod draw_order;
mod perf;
mod ground_shade;
mod prepass_normals;
mod scene_merge;
mod debris_view;
mod effect_view;
mod foliage_view;
mod fast_lit;
mod godrays;
mod water_reflect;
mod gpu_skin;
mod ui_lazy;
mod graphics;
mod pass1;
mod shadow_view;
mod orbit;
mod pattern;
mod audio_view;
mod hud_view;
mod book_view;
mod menu_view;
mod pick_view;
mod menus;
mod movie_view;
mod placeholder;
mod play;

use bevy::asset::RenderAssetUsages;
use bevy::image::ImageSampler;
use bevy::input::mouse::{AccumulatedMouseMotion, AccumulatedMouseScroll, MouseScrollUnit};
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use controls::{Browser, movement};
use neopets_data::archive::ArchiveIndex;
use neopets_data::{Disc, iso_path};
use neopets_sim::{FixedStep, PlayerInput, Simulation};
use orbit::Orbit;

/// Play mode's world and hero: the new-game exterior (SPEC-0006 Default start) and Tor's first
/// outfit.
const PLAY_PACK: u32 = 909;
const PLAY_OUTFIT: u32 = 504;

/// The clean side's own save file (SPEC-0025 Summary and Clean-side mapping of the card jobs):
/// one file of eight 0x3100-byte slots (the original has four; a four-slot 0xC400-byte file
/// still loads), kept under private/ (never committed).
const SAVE_FILE: &str = "private/saves/savegame.bin";

/// The save scan's read: the file when it exists and is the eight- or four-slot size.
/// SPEC-0034 test vectors 70 to 72: one node of a definition run alone with every draw at its
/// midpoint, at the origin with an identity frame.
fn check_effects(disc: &mut Disc, index: &ArchiveIndex) -> Result<(), String> {
    use neopets_sim::effect::*;
    let bytes = index.read(disc, neopets_data::effect::RESOURCE)?;
    let chain = neopets_data::resource::walk_chunks(&bytes, None)?;
    let range = neopets_data::effect::table_in_chain(&bytes, &chain.chunks).ok_or("no effect table")?;
    let table = EffectTable::parse(&bytes[range.clone()])?;
    // SPEC-0034 2.1: the 50 child chunks; the save point's ray models (children 8 and 9) against
    // resource-9 models 8 and 9, by part count and height.
    let size = |m: &neopets_data::model::ModelData| {
        let ys = m.parts.iter().flat_map(|p| p.positions.iter().map(|q| q[1]));
        let (lo, hi) = ys.fold((f32::MAX, f32::MIN), |(l, h), y| (l.min(y), h.max(y)));
        format!("{} parts, y {lo:.2} to {hi:.2}", m.parts.len())
    };
    println!("Effect child chunks: {}", table.child_chunks.len());
    // SPEC-0004 Which models carry animated materials: 4 of the 38 container models scroll
    // (roles 2, 30, 31 and 33) and 9 of the 50 children use scrolled or flipbook materials.
    let animated = |m: &neopets_data::model::ModelData| m.parts.iter().any(|p| p.scroll.is_some() || p.flipbook.is_some());
    let roles: Vec<u16> = (0..38u16).filter(|&r| neopets_data::model::build_roles(&bytes, &chain.chunks, None, Some(&[r])).is_ok_and(|m| animated(&m))).collect();
    let kids: Vec<usize> = (0..table.child_chunks.len()).filter(|&k| neopets_data::model::build_child(&bytes, &chain.chunks, range.start + table.child_chunks[k]).is_ok_and(|m| animated(&m))).collect();
    println!("Animated effect models: container roles {roles:?}; child chunks {kids:?}");
    for k in [8usize, 9] {
        let child = table.child_chunks.get(k).map(|&o| neopets_data::model::build_child(&bytes, &chain.chunks, range.start + o).map(|m| size(&m)));
        let role = neopets_data::model::build_roles(&bytes, &chain.chunks, None, Some(&[k as u16])).map(|m| size(&m));
        println!("  model {k}: child chunk {child:?}; resource-9 model {role:?}");
    }
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
    // The bars' loss effects (SPEC-0023 section 3): started as the loss step starts them, the
    // spawners and particles live after each tick count.
    // NEOPETS_FX_IDS="a,b,...": more effect ids to run the same way (diagnostic).
    let extra: Vec<i32> = std::env::var("NEOPETS_FX_IDS").map(|v| v.split(',').filter_map(|t| t.trim().parse().ok()).collect()).unwrap_or_default();
    // NEOPETS_FX_IDS also dumps each extra definition's gates and emitters (diagnostic).
    for &id in &extra {
        let Some(d) = table.find(id) else { continue };
        let def = &table.definitions[d];
        println!("  definition {id}: threshold {} cull {} cone {} multi {} scale {}", def.threshold, def.cull, def.cone, def.multi, def.scale);
        for &n in &def.nodes {
            let node = &table.nodes[n];
            let a = &table.a[node.a];
            println!("    node {n}: rates {:?} thresholds {:?} B {:?} gate {} size {:?} per_burst {} period {} cycles {} life {} looped {} material {:?} child {} factors {:?} ring {} fill {} preset {} birth {} transform {:?} c {:?}", a.size_rates, a.thresholds, table.b[node.b], a.gate, a.size, a.per_burst, a.period, a.cycles, a.life, a.looped, node.material, node.child, node.factors, node.ring, node.ring_fill, node.preset, a.birth_mode, node.transform, table.c[node.c]);
        }
    }
    for id in [0xFA2, 0xFA3].into_iter().chain(extra) {
        let mut fx = Effects::new();
        let h = fx.add_object(EffectObject::new(&table, id, [0.0; 3], IDENTITY));
        fx.update(&table, h, None, &mut Midpoint);
        let mut seen = Vec::new();
        for t in 1..=1200 {
            fx.tick(&table, &mut Midpoint);
            if matches!(t, 1 | 10 | 30 | 60 | 120 | 300 | 600 | 1200) {
                let size = fx.particles.iter().flatten().map(|p| p.size[0]).fold(0.0f32, f32::max);
                seen.push(format!("t{t}: {} spawners {} particles, largest size {size:.2}", fx.live_spawners(), fx.live_particles()));
            }
        }
        println!("  loss effect {id:#x} (definition {:?}): {}", table.find(id), seen.join(", "));
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

/// The save file's path: SAVE_FILE, or NEOPETS_SAVE_FILE (diagnostic, so a test file can be
/// used without touching the player's saves).
fn save_file() -> String {
    std::env::var("NEOPETS_SAVE_FILE").unwrap_or_else(|_| SAVE_FILE.to_string())
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
    let Archive { disc, index } = archive;
    load_game_from(disc, index, old, slot)
}

fn load_game_from(disc: &mut Disc, index: &ArchiveIndex, old: &play::Play, slot: &neopets_sim::save::Slot) -> Result<play::Play, String> {
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
    if std::env::var_os("NEOPETS_TRACE_BIND").is_some() {
        println!("      save load: keys {} and {}, T {t:?}, E {e:?}", slot.key(0), slot.key(1));
    }
    // SPEC-0026 Load game and SPEC-0025 Loading: the slot's progress flags are restored first,
    // then T and E load and each runs its Init at load completion, so both Inits see the saved
    // flags (before, E's Init ran with a new game's flags and T's never ran, so the road
    // module of 691 never loaded its bandits).
    let saved: Vec<u16> = (0..neopets_sim::save::FLAGS).filter(|&n| slot.flag(n)).collect();
    let mut play = play::Play::load_with_flags(disc, index, current, PLAY_OUTFIT, &saved)?;
    if let Some(t) = t.filter(|&t| t != current && play.other.as_ref().is_none_or(|o| o.pack != t)) {
        play.load_other(disc, index, t)?;
    }
    // Flags the Inits set themselves (not the new-game marker 707) outlive the slot's restore.
    let by_init: Vec<u16> = play.state.flags.iter().copied().filter(|&f| f != 707 && !slot.flag(f)).collect();
    // The hero and the characters are drawn again from the new game (its hero may be the
    // other one, and its character ids start again).
    play.generation = old.generation + 1;
    play.cheats = old.cheats;
    play.snapshot = Some(slot.clone());
    play.apply_slot(slot);
    for f in by_init {
        play.state.flags.insert(f);
    }
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
    play.player.hero_max = old.player.hero_max;
    play.player.hero_hp = old.player.hero_hp;
    play.player.hero_mp_max = old.player.hero_mp_max;
    play.player.hero_mp = old.player.hero_mp;
    play.generation = old.generation + 1;
    play.options = old.options;
    play.cheats = old.cheats;
    Ok(play)
}

/// Developer menu boss fight: a new game in the boss's world with its start flags (the stage
/// rows of its start index and its walkthrough flags), then the stage grants, gear and placement.
fn boss_fight(disc: &mut Disc, index: &ArchiveIndex, old: &play::Play, b: &play::BossStart) -> Result<play::Play, String> {
    let flags = old.boss_flags(b);
    let mut play = play::Play::load_with_flags(disc, index, b.pack, PLAY_OUTFIT, &flags)?;
    play.generation = old.generation + 1;
    play.options = old.options;
    play.cheats = old.cheats;
    play.boss_setup(b);
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
pub(crate) struct HeroRoot;

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
                        archive_view::world_scene(&mut disc, &index, pack, &bytes, &chain.chunks, false, 0)?
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
    let _span = crate::perf::span("stream_worlds");
    let (Some(mut state), Some(mut streamer)) = (play, streamer) else { return };
    for line in streamer.pump(&mut state.play) {
        info!("{line}");
    }
}

/// The opened disc and its archive index, present only when the ISO validated.
#[derive(Resource)]
pub(crate) struct Archive {
    pub(crate) disc: Disc,
    pub(crate) index: ArchiveIndex,
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

/// The graphics API on Windows: Vulkan, with DX12 as the fallback, unless WGPU_BACKEND names one.
/// Measured on the user's laptop (RX 6700S; performance passes of 2026-11-12, views 24, 25, 27
/// and 29 at 1920 x 1080, vsync off):
/// - With the display on the RX 6700S (Armoury Crate GPU mode Ultimate): Vulkan 4.6 to 5.9 ms a
///   frame (World shadows 5.2 to 6.5), DX12 9.5 to 13.7 (World shadows 17.7 to 22.1).
/// - With the display on the integrated GPU (Standard or Optimized), Vulkan's present copies
///   every frame across (about 6.5 ms at 1920 x 1080), and DX12 was 1 to 3 ms faster there
///   with World shadows off, slower with them on.
/// On DX12 wgpu lacks buffer binding arrays, so Bevy cannot draw the standard material
/// bindless: each of the world's 300 to 400 materials is its own batch with its own buffer
/// writes in every view. Vulkan is the better choice everywhere but that one case, so it is the
/// default; the Display page's Graphics API row (saved settings, read here at start) chooses DX12
/// instead, with Vulkan then the fallback.
fn render_settings() -> bevy::render::settings::WgpuSettings {
    use bevy::render::settings::Backends;
    let mut s = bevy::render::settings::WgpuSettings::default();
    if cfg!(target_os = "windows") && std::env::var_os("WGPU_BACKEND").is_none() {
        let dx12 = display::GRAPHICS_APIS[display::DisplaySettings::load().graphics_api] == "DX12";
        s.backends = Some(if dx12 { Backends::DX12 } else { Backends::VULKAN | Backends::DX12 });
    }
    s
}

fn main() {
    // A crash's message, place and backtrace are also appended to private/crash.log (with the
    // time), so a crash in a window started without a console can be read afterwards.
    let default_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let when = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(0, |d| d.as_secs());
        let thread = std::thread::current().name().unwrap_or("?").to_string();
        let trace = std::backtrace::Backtrace::force_capture();
        let text = format!("--- crash at unix time {when}, thread {thread}
{info}
{trace}
");
        if std::fs::create_dir_all("private").is_ok() {
            use std::io::Write;
            if let Ok(mut f) = std::fs::OpenOptions::new().create(true).append(true).open("private/crash.log") {
                let _ = f.write_all(text.as_bytes());
            }
        }
        default_hook(info);
    }));
    let mut args = std::env::args().skip(1);
    let mut explicit = None;
    let mut smoke_test = false;
    let mut screenshot = None;
    let mut hide_hud = false;
    // `--show-info`: in play, start with the developer info panel shown (F6 toggles it).
    let mut show_info = false;
    let mut catch_up = MAX_CATCH_UP;
    let mut play_mode = false;
    let mut goal_start = None;
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
    let mut autoplay = false;
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
        } else if arg == "--autoplay" {
            autoplay = true;
        } else if arg == "--seed" {
            seed = args.next().and_then(|v| v.parse::<u32>().ok());
        } else if arg == "--flag" {
            flags.extend(args.next().and_then(|v| v.parse::<u16>().ok()));
        } else if arg == "--play" {
            play_mode = true;
        } else if arg == "--goal" {
            play_mode = true;
            goal_start = args.next().and_then(|v| v.parse::<u32>().ok()).or(Some(1));
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
        // NEOPETS_BOSS_BOX: the F6 boss box starts on (diagnostic, for screenshots).
        .insert_resource(BossBoxOn(std::env::var_os("NEOPETS_BOSS_BOX").is_some()))
        // `--autoplay`: the autopilot turns on once play has started.
        .insert_resource(AutoplayStart(autoplay))
        .init_resource::<BrowserView>()
        .init_resource::<SceneSigs>()
        .init_resource::<TextureView>()
        .init_resource::<ModelView>()
        .insert_resource(ClearColor(Color::srgb(0.025, 0.035, 0.06)))
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Neopets • Rust rewrite workspace".into(),
                // NEOPETS_WINDOW=WxH: the starting window size (diagnostic; 1000 x 600 by default).
                resolution: std::env::var("NEOPETS_WINDOW")
                    .ok()
                    .and_then(|v| {
                        let (w, h) = v.split_once('x')?;
                        Some((w.trim().parse::<u32>().ok()?, h.trim().parse::<u32>().ok()?))
                    })
                    .unwrap_or((1000, 600))
                    .into(),
                visible: !smoke_test,
                // NEOPETS_PERF_NOVSYNC: no vsync, to measure the real frame cost (diagnostic).
                present_mode: if std::env::var_os("NEOPETS_PERF_NOVSYNC").is_some() { bevy::window::PresentMode::AutoNoVsync } else { bevy::window::PresentMode::AutoVsync },
                ..default()
            }),
            ..default()
        }).set(bevy::render::RenderPlugin { render_creation: render_settings().into(), ..default() }))
        .init_resource::<dev_menu::DevMenu>()
        .init_resource::<states_menu::StatesMenu>()
        .init_resource::<dev_spawn::SpawnPicker>()
        .insert_resource(bindings::Bindings::load())
        .init_resource::<bindings::Pad>()
        .init_resource::<bindings::PadMute>()
        .init_resource::<bindings::PadExclude>()
        .init_resource::<coop::Coop>()
        .init_resource::<net::Net>()
        .init_resource::<net::JoinAddress>()
        .init_resource::<net::NetPassword>()
        .init_resource::<bindings::MouseLook>()
        .init_resource::<controls_page::ControlsPage>()
        .init_resource::<perf::PerfOverlay>()
        .insert_resource(display::DisplaySettings::load())
        .add_plugins(graphics::GraphicsPlugin)
        .add_plugins(fast_lit::FastLitPlugin)
        .add_plugins(gpu_skin::GpuSkinPlugin)
        .add_plugins(godrays::GodRaysPlugin)
        .add_plugins(water_reflect::WaterReflectPlugin)
        .add_plugins(scene_merge::OverlayPlugin)
        .add_plugins(prepass_normals::PrepassNormalsPlugin)
        .add_plugins(pass1::Pass1Plugin)
        .add_plugins(draw_order::DrawOrderPlugin)
        .add_plugins(shadow_view::ShadowPlugin)
        .init_resource::<display::SceneTarget>()
        .add_systems(Startup, no_gpu_clustering)
        .add_systems(Startup, (setup_viewer, setup_hud, setup_play, dev_menu::spawn_panel, display::setup))
        // The developer and settings windows are made while open (ui_lazy.rs).
        .add_systems(First, ui_lazy::sync_windows)
        .add_systems(Update, (display::take_requests, display::apply_window, display::sync_scene).chain())
        .add_systems(Update, debug_tools::trace_assets)
        .add_systems(Update, (audio_view::sync_music, audio_view::sync_bank, audio_view::sync_effects, audio_view::sync_stingers).after(step_sim))
        .add_systems(
            Update,
            (
                (controls_page::page_mouse, controls_page::page_input, controls_page::grab_cursor, bindings::read_pad, dev_spawn::picker_input, dev_menu::dev_mouse, dev_menu::dev_input, read_input, coop::read_coop, net::type_address, net::net_step, step_sim, stream_worlds).chain(),
                (browse_resources, cycle_textures, toggle_view, choose_action, orbit_camera, toggle_hud, toggle_autopilot, sync_haze, sync_atmos_overlay),
                (show_sim, show_browser),
                (apply_texture, apply_model, skin_hero, sync_worlds, sync_zones, sync_uv_scroll, (play_frame, coop::draw_others, coop::split_cameras, sync_hero_items, sync_hero_glows, sync_effect_models, debris_view::sync_debris).chain(), (sync_characters, sync_attachments).chain(), (sync_shots, sync_aim_marker, sync_cloud_fx, stage_view::drift_wisps, shadow_view::sync_shadows, shadow_view::sync_sun, shadow_view::mark_faint, shadow_view::sync_casters), (effect_view::sync_effects, effect_view::sync_trails), foliage_view::sync_bushes, sync_objects, sync_items, (hud_view::sync_hud::<0>, hud_view::sync_hud::<1>).chain(), hud_view::sync_textbox, menu_view::sync_menu, sync_fade, dev_menu::free_camera, (dev_menu::show_panel, dev_menu::show_toast, show_boss_box, show_auto_box), perf::perf_overlay).chain(),
            )
                .chain(),
        );
    app.add_systems(Update, sync_breakables.after(sync_objects));
    app.add_systems(Update, sync_blocks.after(sync_objects));
    app.add_systems(Update, send_rumble);
    app.add_systems(Update, sync_uv_flip);
    app.add_systems(Startup, (setup_movies, movie_view::spawn_screen));
    app.init_resource::<HeroItemFrames>();
    app.add_systems(Update, (movie_view::collect, movie_view::sync_screen, movie_view::sync_skip, movie_view::sync_sound, movie_view::hush).chain().after(step_sim));
    app.add_systems(Update, (display::window_mouse.before(step_sim), display::show_window.after(step_sim)));
    app.add_systems(Update, controls_page::show_page.after(dev_menu::show_panel));
    app.add_systems(Update, dev_spawn::picker_show);
    app.init_resource::<book_view::BookView>();
    app.add_systems(Update, book_view::sync_book.after(play_frame));
    app.add_systems(Update, pick_view::pick.after(play_frame));
    app.init_resource::<debug_tools::DebugState>();
    app.add_plugins(debug_tools::WireframeTool);
    app.init_resource::<item_gallery::Gallery>();
    app.add_systems(Update, item_gallery::step.after(play_frame));
    app.add_systems(Update, (debug_tools::track_events, debug_tools::save_report, debug_tools::note_input, states_menu::states_menu, debug_tools::show_panel, debug_tools::hide_hud, debug_tools::show_lines).after(play_frame));
    app.insert_resource(goal_start.map_or_else(story_run::GoalWalk::default, story_run::GoalWalk::open));
    app.add_systems(Startup, story_run::spawn_goal_panel);
    app.add_systems(Update, (story_run::goal_walk_step.before(play_frame), story_run::show_goal_panel));
    app.add_systems(PostUpdate, ui_fit::fit_to_window.after(bevy::ui::UiSystems::Layout));
    app.add_systems(Update, button_style::apply);
    app.add_systems(Last, display::limit_frame);
    // NEOPETS_PERF_GPU: Bevy's render diagnostics (CPU and GPU time per render pass), logged
    // every 4 seconds (diagnostic).
    if std::env::var_os("NEOPETS_PERF_GPU").is_some() {
        app.add_plugins((bevy::render::diagnostic::RenderDiagnosticsPlugin, bevy::diagnostic::LogDiagnosticsPlugin { wait_duration: std::time::Duration::from_secs(4), ..default() }));
    }
    if perf::perf_enabled() {
        app.init_resource::<perf::PerfLog>().add_systems(First, perf::perf_first).add_systems(Last, (perf::perf_last, perf_kinds, perf::perf_text));
        perf::add_render_timing(&mut app);
    }
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
static SERIES_TAKEN: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

/// Waits on real time, not frames: render pipelines compile asynchronously after startup,
/// and meshes are simply not drawn until theirs are ready.
fn screenshot_and_exit(
    mut commands: Commands,
    time: Res<Time<Real>>,
    path: Res<ScreenshotPath>,
    state: Option<Res<PlayState>>,
    mut taken: Local<bool>,
    mut seen: Local<Option<f32>>,
    mut exit: MessageWriter<AppExit>,
) {
    use bevy::render::view::screenshot::{Screenshot, save_to_disk};
    let elapsed = time.elapsed_secs();
    // NEOPETS_SHOT_ON="text:ms" (diagnostic): the capture is taken ms after the boss or pickup
    // line first contains the text, instead of at the fixed time (which then only bounds the
    // wait at 120 s).
    let on = std::env::var("NEOPETS_SHOT_ON").ok().and_then(|v| {
        let (text, ms) = v.rsplit_once(':')?;
        Some((text.to_string(), ms.trim().parse::<f32>().ok()? / 1000.0))
    });
    let at = match &on {
        Some((text, delay)) => {
            if seen.is_none() && state.as_ref().is_some_and(|s| s.play.last_mote.contains(text.as_str()) || s.play.last_boss.contains(text.as_str()) || s.play.last_pickup.contains(text.as_str())) {
                *seen = Some(elapsed);
            }
            seen.map_or(120.0, |t| t + delay)
        }
        None => path.1,
    };
    let path = ScreenshotPath(path.0.clone(), at);
    // NEOPETS_SHOT_SERIES=n: n captures on consecutive frames, saved as <path>.<k>.png
    // (diagnostic, for flicker).
    let series = std::env::var("NEOPETS_SHOT_SERIES").ok().and_then(|v| v.parse::<usize>().ok());
    if let Some(n) = series {
        if elapsed >= path.1 && !*taken {
            let k = SERIES_TAKEN.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            if k < n {
                commands.spawn(Screenshot::primary_window()).observe(save_to_disk(format!("{}.{k}.png", path.0)));
                return;
            }
            *taken = true;
        }
        if *taken && elapsed >= path.1 + 2.0 {
            exit.write(AppExit::Success);
        }
        return;
    }
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
        // One light cluster instead of Bevy's grid: the game has no point lights, spotlights,
        // light probes or decals (the sun and the ambient light are global, and the disc's light
        // blocks are applied in gpu_skin.rs and on the CPU), so the per-frame grid was empty work
        // (performance pass of 2026-11-12). ClusterConfig::None would skip it entirely, but Bevy
        // 0.19's GPU clustering then makes a texture of width 0 and the renderer stops. A point
        // light or spotlight added later still lights correctly, only less efficiently.
        bevy::light::cluster::ClusterConfig::Single,
        // NEOPETS_SHADOW_FILTER=0/1/2: hardware 2x2, Gaussian, temporal (diagnostic).
        match std::env::var("NEOPETS_SHADOW_FILTER").ok().as_deref() {
            Some("0") => bevy::light::ShadowFilteringMethod::Hardware2x2,
            Some("2") => bevy::light::ShadowFilteringMethod::Temporal,
            _ => bevy::light::ShadowFilteringMethod::Gaussian,
        },
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
    pad: Res<bindings::Pad>,
    mute: Res<bindings::PadMute>,
    gamepads: Query<(Entity, &Gamepad)>,
    exclude: Res<bindings::PadExclude>,
    mut driver: ResMut<SimDriver>,
) {
    let _span = crate::perf::span("read_input");
    // The bound movement keys, and the gamepads' left sticks (not while the bindings page is up).
    // Player 2's controller (local co-op) moves only player 2.
    let sticks: Vec<Vec2> = if mute.0 { Vec::new() } else { gamepads.iter().filter(|(e, _)| Some(*e) != exclude.0).map(|(_, g)| g.left_stick()).collect() };
    driver.input = movement(pad.movement, sticks);
}

fn step_sim(
    time: Res<Time>,
    vpad: Res<bindings::Pad>,
    mute: Res<bindings::PadMute>,
    gamepads: Query<(Entity, &Gamepad)>,
    mut driver: ResMut<SimDriver>,
    mut models: ResMut<ModelView>,
    mut play: Option<ResMut<PlayState>>,
    autowalk: Option<Res<AutoWalk>>,
    mut menu: Option<ResMut<menu_view::Menu>>,
    art: Option<Res<hud_view::HudArt>>,
    mut archive: Option<ResMut<Archive>>,
    mut commands: Commands,
    mut dev: ResMut<dev_menu::DevMenu>,
    mut movies: Option<ResMut<movie_view::Movies>>,
    (exclude, mut coop, menu_pads, mut netr): (Res<bindings::PadExclude>, ResMut<coop::Coop>, Query<&Gamepad>, ResMut<net::Net>),
) {
    let _span = crate::perf::span("step_sim");
    let mut elapsed = time.delta().as_nanos().min(u64::MAX as u128) as u64;
    // SPEC-0026 Movies: the movies queued last frame play now, and while one plays the whole
    // game loop waits (no ticks, no input to the game).
    // SPEC-0026 hand-off step 1 (added 2026-11-09): while a world load is in progress no queued
    // movie starts; the queue is kept and the game runs on until the load completes.
    let loading = play.as_deref().is_some_and(|p| p.play.job_pack().is_some());
    if let Some(mv) = movies.as_deref_mut().filter(|mv| !(loading && mv.idle())) {
        let playing = mv.step(elapsed, vpad.held(bindings::CROSS) || vpad.held(bindings::START), archive.as_deref_mut().map(|a| &mut a.disc));
        // The played word is set as each movie starts, whoever queued it.
        if let (Some(m), Some(p)) = (mv.started.take(), play.as_deref_mut()) {
            p.play.state.movies.insert(m);
        }
        if std::mem::take(&mut mv.ended) {
            if let Some(p) = play.as_deref_mut() {
                p.play.after_movies();
            }
        }
        if playing {
            driver.last_ticks = 0;
            return;
        }
    }
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
            // The virtual pad (bindings.rs): R1 runs, L1 holds the shield guard, Circle or R2
            // attacks.
            run: vpad.held(bindings::R1),
            jump: false,
            square: false,
            square_held: vpad.held(bindings::SQUARE),
            cross_held: vpad.held(bindings::CROSS),
            guard: vpad.held(bindings::L1),
            attack_held: vpad.held(bindings::CIRCLE) || vpad.held(bindings::R2),
            attack: false,
            r2_held: vpad.held(bindings::R2),
            circle_held: vpad.held(bindings::CIRCLE),
            l2: vpad.held(bindings::L2),
        };
        // The attack button (Circle or R2): its press edge, kept for the next tick when this
        // frame runs none.
        play.play.attack_latch |= vpad.pressed(bindings::CIRCLE) || vpad.pressed(bindings::R2);
        // The quick-item panel (SPEC-0021): the D-pad for its directions and Triangle; Cross
        // and Circle are the jump and attack edges.
        {
            let l = &mut play.play.panel_latch;
            l.up |= vpad.pressed(bindings::UP);
            l.left |= vpad.pressed(bindings::LEFT);
            l.right |= vpad.pressed(bindings::RIGHT);
            l.down |= vpad.pressed(bindings::DOWN);
            l.triangle |= vpad.pressed(bindings::TRIANGLE);
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
        // NEOPETS_TRACE_POPUP=id: the small pickup popup started every 150 ticks (diagnostic, for pictures).
        if let Some(id) = std::env::var("NEOPETS_TRACE_POPUP").ok().and_then(|v| v.trim().parse::<u16>().ok()) {
            if play.play.ticks % 150 == 1 {
                play.play.diag_popup(id);
            }
        }
        // NEOPETS_TRACE_SHOWCASE="tick:id" in the window too: the item showcase started once.
        if let Some((k, id)) = std::env::var("NEOPETS_TRACE_SHOWCASE").ok().and_then(|v| {
            let (a, b) = v.split_once(':')?;
            Some((a.trim().parse::<u64>().ok()?, b.trim().parse::<u16>().ok()?))
        }) {
            static SHOWN: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
            if play.play.ticks >= k && !SHOWN.swap(true, std::sync::atomic::Ordering::Relaxed) {
                play.play.diag_showcase(id);
            }
        }
        // NEOPETS_TRACE_MOVIE="index[,index...]": diagnostic PlayFMV calls at play tick 60.
        if let Ok(v) = std::env::var("NEOPETS_TRACE_MOVIE") {
            static MOVIE_DONE: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
            if play.play.ticks >= 60 && !MOVIE_DONE.swap(true, std::sync::atomic::Ordering::Relaxed) {
                for m in v.split(',').filter_map(|t| t.trim().parse::<u16>().ok()) {
                    play.play.state.queue_movie(m);
                }
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
                play.play.player.statuses.apply(code as i32, secs);
            }
        }
        // NEOPETS_TRACE_DOWN, NEOPETS_TRACE_LEFT and NEOPETS_TRACE_CIRCLE="tick,...": a D-pad Down
        // or Left edge, or a Circle edge, once,
        // on the first frame at or past each tick (diagnostic; Down opens the Mote panel).
        {
            static FIRED: std::sync::Mutex<Vec<(u8, u64)>> = std::sync::Mutex::new(Vec::new());
            for (dir, var) in [(0u8, "NEOPETS_TRACE_DOWN"), (1, "NEOPETS_TRACE_LEFT"), (2, "NEOPETS_TRACE_CIRCLE")] {
                for k in std::env::var(var).unwrap_or_default().split(',').filter_map(|t| t.trim().parse::<u64>().ok()) {
                    let mut fired = FIRED.lock().unwrap();
                    if play.play.ticks >= k && !fired.contains(&(dir, k)) {
                        fired.push((dir, k));
                        match dir {
                            0 => play.play.panel_latch.down = true,
                            1 => play.play.panel_latch.left = true,
                            _ => play.play.attack_latch = true,
                        }
                    }
                }
            }
        }
        // NEOPETS_TRACE_SWITCH also works in the window: the chord on those play ticks.
        if std::env::var("NEOPETS_TRACE_SWITCH").ok().is_some_and(|v| v.split(',').filter_map(|t| t.trim().parse::<u64>().ok()).any(|k| (k..k + 3).contains(&(play.play.ticks + 1)))) {
            play.play.l1_latch = true;
            play.play.r1_latch = true;
        }
        // The hero switch chord's press edges: L1 (Q or the left bumper) and R1 (Shift or the
        // right bumper), kept for the next tick (SPEC-0053 3.8 c).
        play.play.l1_latch |= vpad.pressed(bindings::L1);
        play.play.r1_latch |= vpad.pressed(bindings::R1);
        // Square confirms a prompt; a press edge like Cross.
        play.play.square_latch |= vpad.pressed(bindings::SQUARE);
        // Cross is a press edge; keep it for the next tick when this frame runs none.
        play.play.jump_latch |= vpad.pressed(bindings::CROSS);
        if let Some(auto) = autowalk.as_deref() {
            // NEOPETS_AUTO_STAND: --autojump presses only, the hero stands (diagnostic).
            input.forward = if std::env::var_os("NEOPETS_AUTO_STAND").is_some() { 0.0 } else { 0.7 };
            if auto.jump && ticks > 0 && play.play.ticks % 120 == 0 {
                play.play.jump_latch = true;
            }
        }
        // The right stick as PS2 pad bytes (X right 255, Y up 0); the bound camera keys and the
        // mouse (bindings.rs) add to it.
        let mut right = if mute.0 { Vec2::ZERO } else { gamepads.iter().filter(|(e, _)| Some(*e) != exclude.0).map(|(_, g)| g.right_stick()).find(|v| v.length() > 0.2).unwrap_or(Vec2::ZERO) };
        right += vpad.look;
        let byte = |v: f32| (127.5 + 127.5 * v.clamp(-1.0, 1.0)).round() as u8;
        let r3_held = vpad.held(bindings::R3);
        play.play.r3_latch |= vpad.pressed(bindings::R3);
        // L3 shows every HUD group again.
        play.play.l3_latch |= vpad.pressed(bindings::L3);
        // SPEC-0035: the front end runs once per tick; outside play and the game-over page the
        // world is frozen and the hero's buttons are dropped.
        let (mut edge, mut buttons) = menu_view::menu_words(&vpad);
        let mut held = menu_view::menu_held(&vpad);
        // The left stick's direction bits join the rising-edge and held words (never the
        // button-only word). Edges seen on a frame that runs no tick are kept for the next
        // tick, so no press is lost when the frame rate is above the 60 Hz tick rate.
        if let Some(m) = menu.as_deref_mut() {
            let stick = if mute.0 { 0 } else { menu_view::stick_bits(&menu_pads) };
            edge |= stick & !m.stick;
            held |= stick;
            m.stick = stick;
            edge |= m.latched.0;
            buttons |= m.latched.1;
            m.latched = (0, 0);
        }
        // --autojump also presses Square every 10 ticks while a dialogue box is open, as the
        // trace does (diagnostic, for screenshots past a scene's lines).
        let shot_lines = std::env::var_os("NEOPETS_SHOT_BOSS").is_some() && play.play.ticks > 120;
        if (autowalk.as_deref().is_some_and(|a| a.jump) || shot_lines) && ticks > 0 && play.play.dialogue_open && play.play.ticks % 10 == 0 {
            play.play.square_latch = true;
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
        // Online (net.rs): a joined, left or lost line for the player; the host paused; a
        // joiner whose host left goes back to the title screen.
        if let Some(n) = netr.notice.take() {
            dev.toast = Some((n, 300));
        }
        if netr.following() && netr.peer_paused && dev.toast.as_ref().is_none_or(|t| t.1 < 20) {
            let line = if netr.peer_page == 2 { "Game over. The host chooses what happens next." } else { "The host paused the game." };
            dev.toast = Some((line.into(), 20));
        }
        let net_ended = std::mem::take(&mut netr.ended);
        // Online (net.rs): the host's game, sent at joining and at each of its world changes.
        let net_load = netr.load.take();
        if net_load.is_some() {
            pending = net_load.clone();
        }
        // NEOPETS_LOAD_SLOT=k (diagnostic): load slot k (1 to 4) of the save file once, on the
        // first play frame, as the Load page would.
        static SLOT_LOADED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
        if let Some(k) = std::env::var("NEOPETS_LOAD_SLOT").ok().and_then(|v| v.parse::<usize>().ok()) {
            if !SLOT_LOADED.swap(true, std::sync::atomic::Ordering::Relaxed) {
                pending = read_save(&save_file()).and_then(|f| f.slot(k.saturating_sub(1)).cloned());
                if pending.is_none() {
                    error!("NEOPETS_LOAD_SLOT: slot {k} is empty or the save file is unreadable");
                }
            }
        }
        // Quit's world teardown, run after this frame's ticks.
        let mut quit = false;
        if net_ended {
            quit = true;
            if let (Some(m), Some(a)) = (menu.as_deref_mut(), art.as_deref()) {
                m.flow.set(&a.f, menus::CARD);
            }
        }
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
                    shortcuts: !play.play.quick.open && !play.play.mote_panel.open && !l.up && !l.down && !play.play.dialogue_open,
                    book_gate: play.play.books.gate(),
                    book_index: play.play.books.index(),
                    box_busy: play.play.dialogue_open && play.play.textbox.open,
                };
                m.flow.held = held;
                // Back in play the menu is player 1's again (player 2's Start sets it to 2's).
                if m.flow.current == menus::PLAY {
                    play.play.menu_player = 0;
                }
                // A joiner's menus are player 2's (its own hero is player 2 here).
                if play.play.net_follow {
                    play.play.menu_player = 1;
                }
                let grid = m.flow.grid_ids;
                m.flow.view = play.play.as_menu_player(|p| p.item_view(&grid));
                m.flow.map_view = play.play.map_view();
                // SPEC-0025 Clean-side mapping of the card jobs: the scans read the save file,
                // the write job writes the whole file at once.
                match m.flow.tick(&a.f, std::mem::take(&mut edge), std::mem::take(&mut buttons), &ctx) {
                    Some(menus::Command::Scan) => {
                        m.flow.scanned(read_save(&save_file()));
                        // NEOPETS_TRACE_PICKER_CURSOR=k: the picker's cursor on slot k after the
                        // scan (diagnostic, for a screenshot of the scrolled list).
                        if let Some(k) = std::env::var("NEOPETS_TRACE_PICKER_CURSOR").ok().and_then(|v| v.trim().parse::<usize>().ok()) {
                            m.flow.picker.cursor = k.min(neopets_sim::save::SLOTS - 1);
                        }
                    }
                    Some(menus::Command::Rebuild(k)) => {
                        play.play.write_slot(&mut m.flow.picker.file.slots[k], &save_date());
                        // The snapshot and the pending flag are set at the confirm, before the
                        // write's question and result.
                        play.play.snapshot = Some(m.flow.picker.file.slots[k].clone());
                    }
                    Some(menus::Command::Write) => {
                        let ok = write_save(&save_file(), &m.flow.picker.file);
                        if let Err(e) = &ok {
                            error!("Save failed: {e}");
                        }
                        m.flow.written(ok.is_ok());
                        // Online, hosting: the joiner is told (it keeps no save of its own).
                        if ok.is_ok() && netr.role == net::Role::Host && netr.peer.is_some() {
                            netr.push_event(net::Event::Notice(net::NOTICE_SAVED));
                        }
                    }
                    Some(menus::Command::Load(k)) => pending = m.flow.picker.file.slot(k).cloned(),
                    Some(menus::Command::BookClose(saved)) => {
                        play.play.books.cursor = -1;
                        if saved {
                            play.play.books.saved = -1;
                        }
                    }
                    Some(menus::Command::Quit) => {
                        quit = true;
                        // Online (net.rs): quitting ends the session (the host's joiner goes back
                        // to the title screen; a joiner leaves the host's game).
                        if netr.role != net::Role::Off {
                            netr.request = Some(net::Request::Leave);
                        }
                    }
                    // Online, joining: the command goes to the host (its game is the one kept).
                    Some(c) if netr.following() && play.play.net_follow => {
                        if let Some(k) = net::cmd_of(&c) {
                            netr.cmds.push_back(k);
                            info!("net: item-screen command {c:?} sent to the host");
                        }
                    }
                    Some(c) => {
                        play.play.as_menu_player(|p| p.item_command(&c));
                        if play.play.menu_player != 0 && std::env::var_os("NEOPETS_TRACE_COOPMENU").is_some() {
                            println!("COOPMENU tick {}: player 2's item command {c:?}; p2 hp {:?}", play.play.ticks, play.play.others.first().map(|o| o.hero_hp));
                        }
                    }
                    None => {}
                }
                // NEOPETS_SHOT_BOOK=value: the window opens that book at once (diagnostic, to picture it).
                static SHOT_BOOK: std::sync::Once = std::sync::Once::new();
                SHOT_BOOK.call_once(|| {
                    if let Some(v) = std::env::var("NEOPETS_SHOT_BOOK").ok().and_then(|v| v.trim().parse::<i32>().ok()) {
                        play.play.books.cursor = v;
                    }
                });
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
                // SPEC-0023 section 8: opening the quest journal clears the notification.
                if m.flow.current == menus::JOURNAL {
                    play.play.hud.journal = false;
                }
                // SPEC-0021 Panel visibility item 5 (R-0195): the Select and Start openers run the HUD reset.
                if frozen && !play.play.menu_open {
                    play.play.hud_reset();
                }
                if m.flow.current == menus::PAUSE && !play.play.menu_open {
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
            // The autopilot (F7; this build's test tool) plays in the player's place.
            play.play.autopilot_input(&mut input);
            // NEOPETS_DEV_SCRIPT=1 (diagnostic): the trace's NEOPETS_TRACE_STICK, _FACE, _CROSS and _SQUARE
            // scripts drive the window's hero by play tick, so a traced moment can be pictured.
            if std::env::var_os("NEOPETS_DEV_SCRIPT").is_some() {
                dev_script(&mut play.play, &mut input);
            }
            // NEOPETS_SHOT_BOSS's companions: NEOPETS_TRACE_AUTO, _BOSSHP and _LEVEL, as in the
            // trace, on those play ticks (diagnostic).
            if std::env::var_os("NEOPETS_SHOT_BOSS").is_some() {
                play.play.cheats.god |= std::env::var_os("NEOPETS_TRACE_GOD").is_some();
                if play.play.ticks == 5 && std::env::var_os("NEOPETS_TRACE_AUTO").is_some() && play.play.autopilot.is_none() {
                    play.play.toggle_autopilot();
                }
                for (k, h) in std::env::var("NEOPETS_TRACE_BOSSHP").unwrap_or_default().split(';').filter_map(|e| e.split_once(':')) {
                    if k.trim().parse::<u64>().ok() == Some(play.play.ticks) {
                        if let Ok(h) = h.trim().parse::<i32>() {
                            play.play.diag_gelert_hp(h);
                        }
                    }
                }
                // NEOPETS_SHOT_BREAK="first:every": from that tick, on each period, the hero is put beside
                // the nearest intact breakable and it breaks (diagnostic, for the debris).
                if let Some((a, b)) = std::env::var("NEOPETS_SHOT_BREAK").ok().as_deref().and_then(|v| v.split_once(':')).and_then(|(a, b)| Some((a.trim().parse::<u64>().ok()?, b.trim().parse::<u64>().ok()?))) {
                    if play.play.ticks >= a && (play.play.ticks - a) % b.max(1) == 0 {
                        play.play.diag_break_nearest();
                    }
                }
                // NEOPETS_SHOT_WARP="tick:x,z,y": the hero placed on the ground below y.
                if let Some((k, r)) = std::env::var("NEOPETS_SHOT_WARP").ok().as_deref().and_then(|v| v.split_once(':')).map(|(a, b)| (a.to_string(), b.to_string())) {
                    let n: Vec<f32> = r.split(',').filter_map(|t| t.trim().parse().ok()).collect();
                    if k.trim().parse::<u64>().ok() == Some(play.play.ticks) && n.len() == 3 {
                        let y = play.play.world.ground_at(n[0], n[1], n[2]).unwrap_or(n[2]);
                        let f = play.play.player.loco.facing;
                        play.play.player.loco.teleport([n[0], y, n[1]], f);
                    }
                }
                for (f, k) in std::env::var("NEOPETS_TRACE_LEVEL").unwrap_or_default().split(',').filter_map(|e| e.split_once('@')) {
                    if k.trim().parse::<u64>().ok() == Some(play.play.ticks) {
                        if let Ok(f) = f.trim().parse() {
                            play.play.state.set_level(f, true);
                        }
                    }
                }
            }
            debug_tools::window_replay(&mut play.play, &mut input);
            // NEOPETS_NET_GAMEOVER=T (diagnostic, not on a joiner): every player falls on play
            // tick T (the game-over page); NEOPETS_NET_CONTINUE: Cross on the page every second.
            if !play.play.net_follow {
                if std::env::var("NEOPETS_NET_GAMEOVER").ok().and_then(|v| v.trim().parse::<u64>().ok()) == Some(play.play.ticks) {
                    // NEOPETS_NET_SNAPSHOT: as if the game had been saved just before (Continue
                    // then reloads it instead of going to the title screen).
                    if std::env::var_os("NEOPETS_NET_SNAPSHOT").is_some() {
                        let mut slot = neopets_sim::save::Slot::empty();
                        play.play.write_slot(&mut slot, b"DIAG");
                        play.play.snapshot = Some(slot);
                    }
                    play.play.diag_game_over();
                    info!("net: game over forced on tick {}", play.play.ticks);
                }
                if let Some(t) = std::env::var("NEOPETS_NET_GAMEOVER").ok().and_then(|v| v.trim().parse::<u64>().ok()).filter(|&t| play.play.ticks > t && (play.play.ticks - t) % 120 == 0) {
                    let p = &play.play;
                    info!("net: {} ticks after the fall: state {} hero {} page {:?} flag E {} mode {} timer E {} clip {:?} hp {} others down {:?} job {} dialogue {}", p.ticks - t, p.player.loco.state, p.hero_index, p.death.page, p.death.flag_e, p.death.mode, p.death.timer_e.running(), p.player.hero.clip_state(), p.player.hero_hp, p.others.iter().map(|o| o.down).collect::<Vec<_>>(), p.job.is_some(), p.dialogue_open);
                }
                // NEOPETS_NET_MOVIE="T:m": movie m queued on play tick T, as a script would.
                if let Some((t, m)) = std::env::var("NEOPETS_NET_MOVIE").ok().as_deref().and_then(|v| v.split_once(':')).and_then(|(a, b)| Some((a.trim().parse::<u64>().ok()?, b.trim().parse::<u16>().ok()?))) {
                    if t == play.play.ticks {
                        // Queued before this tick's count of new movies: sent here.
                        play.play.state.movie_queue.push(m);
                        if netr.peer.is_some() {
                            netr.push_event(net::Event::Movie(m));
                        }
                    }
                }
                // NEOPETS_NET_P2HP="T:hp": player 2's health set on play tick T.
                if let Some((t, hp)) = std::env::var("NEOPETS_NET_P2HP").ok().as_deref().and_then(|v| v.split_once(':')).and_then(|(a, b)| Some((a.trim().parse::<u64>().ok()?, b.trim().parse::<i32>().ok()?))) {
                    if t == play.play.ticks {
                        if let Some(o) = play.play.others.first_mut() {
                            o.hero_hp = hp;
                            info!("net: player 2's health set to {hp}");
                        }
                    }
                }
                // NEOPETS_NET_PET="T:type": a petpet of that type activated on play tick T (after
                // any fresh game the start makes).
                if let Some((t, ty)) = std::env::var("NEOPETS_NET_PET").ok().as_deref().and_then(|v| v.split_once(':')).and_then(|(a, b)| Some((a.trim().parse::<u64>().ok()?, b.trim().parse::<i32>().ok()?))) {
                    if t == play.play.ticks {
                        play.play.activate_pet(ty, 30000, 30000);
                        info!("net: petpet {ty} activated on tick {t}");
                    }
                    if play.play.ticks > t && (play.play.ticks - t) % 300 == 0 {
                        let p = &play.play;
                        info!("net: petpet {} ({}) character {:?} found {} at {:?}", ty, p.last_pet, p.pet_char, p.pet_char.is_some_and(|id| p.characters.iter().any(|c| c.id == id)), p.pet_char.and_then(|id| p.characters.iter().find(|c| c.id == id)).map(|c| (c.pack, c.slot, c.ordinal, c.inactive)));
                    }
                }
                // NEOPETS_NET_BOSSHP="T:hp[;T:hp...]": every boss of the world set to hp on tick T
                // (as hits would; the monster bar follows).
                for (t, hp) in std::env::var("NEOPETS_NET_BOSSHP").unwrap_or_default().split(';').filter_map(|e| e.split_once(':')).filter_map(|(a, b)| Some((a.trim().parse::<u64>().ok()?, b.trim().parse::<i32>().ok()?))) {
                    if t == play.play.ticks {
                        play.play.diag_gelert_hp(hp);
                        info!("net: bosses set to {hp} on tick {t}");
                    }
                }
                // NEOPETS_TRACE_NETCLIP: the host's own hero's clip every 30 ticks (to compare).
                if std::env::var_os("NEOPETS_TRACE_NETCLIP").is_some() && play.play.ticks % 30 == 0 {
                    println!("NETCLIP host tick {}: hero state {} clip {:?} rate {:.2}", play.play.ticks, play.play.player.loco.state, play.play.player.hero.clip_state(), play.play.player.loco.rate);
                }
                // NEOPETS_NET_STOPAT=T: hosting (or joining) stopped on play tick T.
                if std::env::var("NEOPETS_NET_STOPAT").ok().and_then(|v| v.trim().parse::<u64>().ok()) == Some(play.play.ticks) {
                    netr.request = Some(net::Request::Leave);
                }
                // NEOPETS_NET_REMOVE=T: the Multiplayer page's Remove player 2 on play tick T.
                if std::env::var("NEOPETS_NET_REMOVE").ok().and_then(|v| v.trim().parse::<u64>().ok()) == Some(play.play.ticks) && netr.peer.is_some() {
                    netr.request = Some(net::Request::Remove);
                }
                // NEOPETS_NET_NOTICE=T: the saved notice sent on play tick T, as a save would.
                if std::env::var("NEOPETS_NET_NOTICE").ok().and_then(|v| v.trim().parse::<u64>().ok()) == Some(play.play.ticks) && netr.peer.is_some() {
                    netr.push_event(net::Event::Notice(net::NOTICE_SAVED));
                }
                if std::env::var_os("NEOPETS_NET_CONTINUE").is_some() && play.play.death.page.is_some() && play.play.ticks % 60 == 0 {
                    input.jump = true;
                    info!("net: Cross on the game-over page on tick {}", play.play.ticks);
                }
            }
            // NEOPETS_TRACE_SETFLAG="tick:flag[,...]" in a window too (diagnostic, not on a
            // joiner): a story flag set on those play ticks, as in the trace.
            if !play.play.net_follow {
                for g in std::env::var("NEOPETS_TRACE_SETFLAG").unwrap_or_default().split(',') {
                    let n: Vec<u64> = g.split(':').filter_map(|x| x.trim().parse().ok()).collect();
                    if let [t, flag] = n[..] {
                        if t == play.play.ticks {
                            play.play.state.set(flag as u16, true);
                        }
                    }
                }
            }
            if netr.following() && play.play.net_follow {
                // Online, joining (net.rs): this side's controller goes to the host as player 2's;
                // this side's game runs without it and is then put where the host's is.
                // NEOPETS_NET_WALK (diagnostic): this side walks ahead, jumping every 1.5 seconds.
                if std::env::var_os("NEOPETS_NET_WALK").is_some() {
                    input.forward = 1.0;
                    input.jump = play.play.ticks % 90 == 45;
                }
                // This side's D-pad and Triangle go to the host for player 2's panels (this side's
                // own game opens none).
                let pl = std::mem::take(&mut play.play.panel_latch);
                for (k, on) in [pl.up, pl.left, pl.right, pl.down, pl.triangle].into_iter().enumerate() {
                    netr.dpad[k] = netr.dpad[k].wrapping_add(on as u8);
                }
                let c = &mut netr.counts;
                c.0 = c.0.wrapping_add(input.jump as u8);
                // NEOPETS_TRACE_NETINPUT (diagnostic): each jump press as it leaves this side.
                if input.jump && std::env::var_os("NEOPETS_TRACE_NETINPUT").is_some() {
                    info!("net: jump {} pressed", c.0);
                }
                c.1 = c.1.wrapping_add(input.attack as u8);
                c.2 = c.2.wrapping_add(input.square as u8);
                let counts = netr.counts;
                let generation = netr.loaded;
                let ack = (netr.applied, netr.story_held.as_ref().map_or(0, |h| h.0));
                let mut out = net::Input::from_hero(&input, &pad, generation, counts, ack);
                out.dpad = netr.dpad;
                out.world = netr.world_held.as_ref().map_or(0, |h| h.0);
                out.seen = netr.seen_tick;
                // NEOPETS_NET_ITEMCMD="T:kind:a:b" (diagnostic): an item-screen command from this
                // side on play tick T (net::cmd_of's numbering).
                if let Some(v) = std::env::var("NEOPETS_NET_ITEMCMD").ok() {
                    let n: Vec<u64> = v.split(':').filter_map(|x| x.trim().parse().ok()).collect();
                    if let [t, k, a, b] = n[..] {
                        if t == play.play.ticks {
                            netr.cmds.push_back((k as u8, a as u16, b as u16));
                            info!("net: item-screen command {k} {a} {b} sent to the host (diagnostic)");
                        }
                    }
                }
                if let Some(&(k, a, b)) = netr.cmds.front() {
                    out.cmd = (netr.cmds_done.wrapping_add(1), k, a, b);
                }
                netr.outgoing = Some(out);
                let sounds = play.play.sound.requests.len();
                let clips = (play.play.stinger_requests.len(), play.play.emote_requests.len());
                let movies = play.play.state.movie_queue.len();
                play.play.effects.share.mute = true;
                play.play.effects.share.log = None;
                // NEOPETS_TRACE_NETOWN (diagnostic): what this side's own game changes of the
                // shared story in its tick (the host's comes with its states).
                let own = std::env::var_os("NEOPETS_TRACE_NETOWN").is_some().then(|| {
                    let s = &play.play.state;
                    (s.flags.clone(), s.items.clone(), s.neopoints, s.collected.clone(), s.maps.clone(), play.play.job.is_some(), play.play.world.pack)
                });
                play.play.tick(&neopets_sim::hero::HeroInput::default(), neopets_sim::camera::CameraPad::default());
                if let Some((flags, items, np, collected, maps, job, pack)) = own {
                    let s = &play.play.state;
                    let added: Vec<u16> = s.flags.difference(&flags).copied().collect();
                    let removed: Vec<u16> = flags.difference(&s.flags).copied().collect();
                    let mut changed: Vec<(u16, u32, u32)> = s.items.iter().filter(|(k, v)| items.get(k) != Some(v)).map(|(&k, &v)| (k, items.get(&k).copied().unwrap_or(0), v)).chain(items.iter().filter(|(k, _)| !s.items.contains_key(k)).map(|(&k, &v)| (k, v, 0))).collect();
                    changed.sort_unstable();
                    let coll: Vec<u32> = s.collected.symmetric_difference(&collected).copied().collect();
                    let mp: Vec<u32> = s.maps.symmetric_difference(&maps).copied().collect();
                    let load = !job && play.play.job.is_some();
                    if !added.is_empty() || !removed.is_empty() || !changed.is_empty() || s.neopoints != np || !coll.is_empty() || !mp.is_empty() || load || play.play.world.pack != pack {
                        println!("NETOWN joiner tick {}: flags +{:?} -{:?}, items {:?}, NP {} -> {}, collected {:?}, maps {:?}, load started {} ({}), world {} -> {}", play.play.ticks, added, removed, changed, np, s.neopoints, coll, mp, load, play.play.last_door, pack, play.play.world.pack);
                    }
                }
                // The joiner hears the host's sounds, not its own game's (net.rs events).
                // NEOPETS_TRACE_NETEV: the events played so far, every 300 ticks (diagnostic).
                if std::env::var_os("NEOPETS_TRACE_NETEV").is_some() && play.play.ticks % 300 == 0 {
                    let cpu = std::mem::take(&mut netr.cpu_us);
                    println!("NETEV joiner tick {}: {} host events played, {} own sound requests dropped this tick; {} states since the last line, largest gap {} ms, {} ticks with no new state; the host's hero stalled {} times, largest step {:.2}; buffer {} (most {}, holding {}); apply {:.1} us a tick", play.play.ticks, netr.applied, play.play.sound.requests.len() - sounds, netr.gap_states, netr.gap_max, netr.repeats, netr.smooth.2, netr.smooth.3, netr.queue.len(), netr.queue_most, netr.hold, cpu as f32 / 300.0);
                    netr.queue_most = 0;
                    (netr.gap_states, netr.gap_max, netr.repeats) = (0, 0, 0);
                    (netr.smooth.2, netr.smooth.3) = (0, 0.0);
                }
                play.play.sound.requests.truncate(sounds);
                play.play.stinger_requests.truncate(clips.0);
                play.play.emote_requests.truncate(clips.1);
                play.play.state.movie_queue.truncate(movies);
                play.play.motor = None;
                let table = play.play.effect_table.clone();
                // The joiner hears from its own view (player 2's camera, or the host's cutscene).
                let ears = {
                    let (eye, forward, _, _) = play.play.view();
                    net::listener_of(eye, forward)
                };
                // The placed host voices' volume and pan from this side's view, each tick.
                {
                    let ids: Vec<usize> = netr.voices.keys().copied().collect();
                    for id in ids {
                        let r = net::hear(neopets_sim::sound::Request::Set { id, volume: 0, pan: None }, &mut netr.voices, &ears);
                        play.play.sound.requests.push(r);
                    }
                }
                for e in std::mem::take(&mut netr.incoming) {
                    match e {
                        net::Event::Voice { id, emitter, base, r, r0 } => {
                            netr.voices.insert(id as usize, (emitter, base, r, r0));
                        }
                        net::Event::Sound(r) => {
                            let r = net::hear(r, &mut netr.voices, &ears);
                            play.play.sound.requests.push(r);
                        }
                        net::Event::Fx(op) => {
                            let p = &mut play.play;
                            p.effects.apply(&table, &op, &mut p.net_fx, &mut p.rng);
                        }
                        net::Event::Stinger(k) => play.play.stinger_requests.push(k),
                        net::Event::Emote(b, s, l) => play.play.emote_requests.push((b, s, l)),
                        net::Event::Movie(m) => play.play.state.movie_queue.push(m),
                        net::Event::Rumble(level, count) => play.play.motor = Some(neopets_sim::rumble::Send { level, count }),
                        net::Event::Notice(n) => {
                            info!("net: the host's notice {n}");
                            netr.notice = net::notice_text(n).map(String::from);
                        }
                    }
                }
                // Only a state of the world loaded here (one from before a world change waits).
                netr.queue_most = netr.queue_most.max(netr.queue.len());
                if let Some(s) = netr.next_state().filter(|s| s.generation == netr.loaded) {
                    let started = std::time::Instant::now();
                    play.play.apply_net_state(&s);
                    // NEOPETS_TRACE_NETCLIP (diagnostic): the host's hero's clip here every 30 ticks.
                    if std::env::var_os("NEOPETS_TRACE_NETCLIP").is_some() && play.play.ticks % 30 == 0 {
                        println!("NETCLIP joiner tick {}: host hero state {} clip {:?}", play.play.ticks, play.play.player.loco.state, play.play.player.hero.clip_state());
                    }
                    netr.cpu_us = netr.cpu_us.saturating_add(started.elapsed().as_micros() as u64);
                    // NEOPETS_TRACE_NETEV: the host's hero here, a tick with no movement after one
                    // with (a stall) and the largest one-tick step, counted for the trace line.
                    {
                        let at = play.play.player.loco.position();
                        let d = netr.smooth.0.map_or(0.0, |b: [f32; 3]| ((at[0] - b[0]).powi(2) + (at[2] - b[2]).powi(2)).sqrt());
                        if d < 0.001 && netr.smooth.1 > 0.05 {
                            netr.smooth.2 += 1;
                        }
                        if d < 2.0 {
                            netr.smooth.3 = netr.smooth.3.max(d);
                        }
                        netr.smooth.0 = Some(at);
                        netr.smooth.1 = d;
                    }
                    // The host walked on into a world this side's game did not: after 1.5 seconds
                    // (no load running here) ask for a new welcome.
                    if s.pack != play.play.world.pack && play.play.job.is_none() {
                        netr.mismatch += 1;
                    } else {
                        netr.mismatch = 0;
                    }
                    if netr.mismatch > 90 {
                        netr.mismatch = 0;
                        netr.ask_resync();
                    }
                }
                if std::env::var_os("NEOPETS_TRACE_NETEV").is_some() && play.play.ticks % 300 == 0 {
                    let p = &play.play;
                    println!("NETFX joiner tick {} (missing {:?}) (told {:?}): {} particles, {} spawners, {} objects, {} shots, music {:?} bank {:?}, fade {}, bars {:.2}, message {:?}, flags {} items {} NP {}, world items {} broken {} hidden {}, objects {:?}, monster bar {:?}, boss hp {:?}, pet {:?}", p.ticks, netr.world_held.as_ref().map(|h| h.1.chars.iter().filter(|k| !p.characters.iter().any(|c| (c.pack, c.slot, c.ordinal as u32) == **k)).count()), netr.world_held.as_ref().map(|h| (h.0, h.1.items.len(), h.1.items.iter().filter(|e| e.0 == p.world.pack || p.other.as_ref().is_some_and(|o| o.pack == e.0)).count(), h.1.pack)), p.effects.live_particles(), p.effects.live_spawners(), p.effects.objects.iter().flatten().count(), p.shots.slots.iter().flatten().count(), p.music.channels.map(|c| (c.stream, c.volume)), p.bank_music.playing, p.fade.alpha, p.bars.progress, p.message.map(|m| m.0), p.state.flags.len(), p.state.items.len(), p.state.neopoints, p.pools.iter().filter(|(k, _)| **k == p.world.pack || p.other.as_ref().is_some_and(|o| o.pack == **k)).map(|(_, pl)| pl.live()).sum::<usize>(), p.world.breakables.iter().filter(|b| b.state == 2).count(), p.characters.iter().filter(|c| c.inactive).count(), p.world.object_anims.iter().enumerate().filter(|(_, o)| o.state != 3).map(|(i, o)| (i, o.state, o.drawn)).collect::<Vec<_>>(), p.monster_bar.target.map(|_| (p.monster_bar.name.clone(), p.monster_bar.fill)), p.characters.iter().filter(|c| c.boss.is_some() || c.gnarfas.is_some() || c.knight.is_some() || c.gelert.is_some() || c.ixi.is_some() || c.serpent.is_some()).map(|c| c.hp).collect::<Vec<_>>(), p.pet_char.and_then(|id| p.characters.iter().find(|c| c.id == id)).map(|c| (c.pack, c.slot, c.ordinal, c.position.map(|v| (v * 10.0).round() / 10.0))));
                }
            } else {
                let sounds = play.play.sound.requests.len();
                let clips = (play.play.stinger_requests.len(), play.play.emote_requests.len());
                let movies = play.play.state.movie_queue.len();
                let swaps_before = play.play.swaps;
                // Online, hosting: the gameplay effects are logged for the joiner (net.rs).
                let hosting = netr.role == net::Role::Host && netr.peer.is_some();
                play.play.effects.share.mute = false;
                if hosting {
                    play.play.effects.share.log.get_or_insert_with(Vec::new);
                } else {
                    play.play.effects.share.log = None;
                }
                play.play.tick(&input, pad);
                // Local co-op (coop.rs): player 2's tick after player 1's; online, player 2's input
                // comes from the joiner (net.rs).
                play.play.coop_remote = netr.role == net::Role::Host && netr.peer.is_some();
                // A game started afresh while hosting (a developer warp) brings player 2 back.
                if play.play.coop_remote && play.play.others.is_empty() {
                    play.play.add_player();
                }
                if !play.play.others.is_empty() {
                    let p2 = coop.take();
                    let pl = std::mem::take(&mut coop.panel);
                    if let Some(o) = play.play.others.first_mut() {
                        let q = &mut o.dpad;
                        (q.up, q.left, q.right, q.down, q.triangle) = (q.up | pl.up, q.left | pl.left, q.right | pl.right, q.down | pl.down, q.triangle | pl.triangle);
                    }
                    play.play.coop_step(&[p2]);
                }
                // NEOPETS_NET_BREAK="first:every" (diagnostic, hosting): from that tick, on each
                // period, the hero breaks the nearest intact breakable and drops coins (so the
                // joiner's copy can be checked).
                if play.play.coop_remote {
                    if let Some((a, b)) = std::env::var("NEOPETS_NET_BREAK").ok().as_deref().and_then(|v| v.split_once(':')).and_then(|(a, b)| Some((a.trim().parse::<u64>().ok()?, b.trim().parse::<u64>().ok()?))) {
                        if play.play.ticks >= a && (play.play.ticks - a) % b.max(1) == 0 {
                            play.play.diag_break_nearest();
                            let n = play.play.dev_drop(160, 6.0);
                            info!("net test: broke the nearest breakable and dropped coins ({n} world items)");
                        }
                    }
                }
                // Online, hosting: the state after this tick, and a new welcome after a world change.
                if play.play.coop_remote {
                    // A world change, or a new game or load here (Continue after a game over): a
                    // new welcome. Walking into the loaded neighbour needs none (the joiner's own
                    // game makes the same move; it asks for a welcome if it ends up elsewhere).
                    let walked = play.play.swaps != swaps_before && play.play.job.is_none() && play.play.generation == netr.welcome_game;
                    if walked && play.play.world.pack != netr.welcome_pack && !netr.resync {
                        netr.welcome_pack = play.play.world.pack;
                        info!("net: world {} by walking; the joiner follows without a welcome", play.play.world.pack);
                    }
                    if std::mem::take(&mut netr.resync) {
                        netr.welcome_pack = u32::MAX;
                    }
                    if play.play.world.pack != netr.welcome_pack || play.play.generation != netr.welcome_game {
                        let mut slot = neopets_sim::save::Slot::empty();
                        play.play.write_slot(&mut slot, b"NETPLAY");
                        let (pack, game) = (play.play.world.pack, play.play.generation);
                        let fx = play.play.effects.snapshot();
                        netr.rewelcome(&slot, pack, game, &fx);
                    }
                    for r in play.play.sound.requests[sounds.min(play.play.sound.requests.len())..].to_vec() {
                        // A voice with a place: where, so the joiner hears it from its own player
                        // (and works out its volume and pan each tick itself: the host's changes
                        // for it are not sent).
                        match r {
                            neopets_sim::sound::Request::Play { id, .. } => {
                                if let Some((emitter, base, rr, r0)) = play.play.sound.source(id) {
                                    netr.push_event(net::Event::Voice { id: id as u32, emitter, base, r: rr, r0 });
                                    netr.placed.insert(id);
                                }
                            }
                            neopets_sim::sound::Request::Set { id, .. } if netr.placed.contains(&id) => continue,
                            neopets_sim::sound::Request::Stop { id } => {
                                netr.placed.remove(&id);
                            }
                            _ => {}
                        }
                        netr.push_event(net::Event::Sound(r));
                    }
                    for op in play.play.effects.share.log.as_mut().map(std::mem::take).unwrap_or_default() {
                        netr.push_event(net::Event::Fx(op));
                    }
                    for k in play.play.stinger_requests[clips.0.min(play.play.stinger_requests.len())..].to_vec() {
                        netr.push_event(net::Event::Stinger(k));
                    }
                    for (b, s, l) in play.play.emote_requests[clips.1.min(play.play.emote_requests.len())..].to_vec() {
                        netr.push_event(net::Event::Emote(b, s, l));
                    }
                    for m in play.play.state.movie_queue[movies.min(play.play.state.movie_queue.len())..].to_vec() {
                        netr.push_event(net::Event::Movie(m));
                    }
                    if let Some(s) = play.play.others.first_mut().and_then(|o| o.motor.take()) {
                        netr.push_event(net::Event::Rumble(s.level, s.count));
                    }
                    let started = std::time::Instant::now();
                    let s = play.play.net_state(netr.generation);
                    netr.send_state(&s);
                    netr.cpu_us = netr.cpu_us.saturating_add(started.elapsed().as_micros() as u64);
                    if std::env::var_os("NEOPETS_TRACE_NETEV").is_some() && play.play.ticks % 300 == 0 {
                        let p = &play.play;
                        if let Some(o) = p.others.first() {
                            println!("NETP2 host tick {}: p2 quick open {} mote open {} selection {}", p.ticks, o.quick.open, o.mote_panel.open, o.mote_panel.selection);
                        }
                        println!("NETFX host tick {}: {} sound voices; {} particles, {} spawners, {} objects, {} shots, music {:?} bank {:?}, fade {}, bars {:.2}, message {:?}, flags {} items {} NP {}, world items {} broken {} hidden {}, objects {:?}, monster bar {:?}, boss hp {:?}, pet {:?}", p.ticks, p.sound.voices(), p.effects.live_particles(), p.effects.live_spawners(), p.effects.objects.iter().flatten().count(), p.shots.slots.iter().flatten().count(), p.music.channels.map(|c| (c.stream, c.volume)), p.bank_music.playing, p.fade.alpha, p.bars.progress, p.message.map(|m| m.0), p.state.flags.len(), p.state.items.len(), p.state.neopoints, p.pools.iter().filter(|(k, _)| **k == p.world.pack || p.other.as_ref().is_some_and(|o| o.pack == **k)).map(|(_, pl)| pl.live()).sum::<usize>(), p.world.breakables.iter().filter(|b| b.state == 2).count(), p.characters.iter().filter(|c| c.inactive).count(), p.world.object_anims.iter().enumerate().filter(|(_, o)| o.state != 3).map(|(i, o)| (i, o.state, o.drawn)).collect::<Vec<_>>(), p.monster_bar.target.map(|_| (p.monster_bar.name.clone(), p.monster_bar.fill)), p.characters.iter().filter(|c| c.boss.is_some() || c.gnarfas.is_some() || c.knight.is_some() || c.gelert.is_some() || c.ixi.is_some() || c.serpent.is_some()).map(|c| c.hp).collect::<Vec<_>>(), p.pet_char.and_then(|id| p.characters.iter().find(|c| c.id == id)).map(|c| (c.pack, c.slot, c.ordinal, c.position.map(|v| (v * 10.0).round() / 10.0))));
                    }
                }
            }
            // A save point's Square press opens the save screen (a direct set of record 18).
            if std::mem::take(&mut play.play.save_open) {
                if let (Some(m), Some(a)) = (menu.as_deref_mut(), art.as_deref()) {
                    m.flow.open_save(&a.f);
                }
            }
            if std::mem::take(&mut play.play.reload_wanted) {
                pending = play.play.snapshot.clone();
            }
            // SPEC-0026 Credits: the gate on story bit 368 opens record 14 with the pending flag.
            if std::mem::take(&mut play.play.credits_wanted) {
                if let (Some(m), Some(a)) = (menu.as_deref_mut(), art.as_deref()) {
                    m.flow.open_credits_from_play(&a.f);
                }
            }
            // The game-over exit with nothing pending: the teardown and record 0.
            if std::mem::take(&mut play.play.teardown_wanted) {
                quit = true;
                if netr.role != net::Role::Off {
                    netr.request = Some(net::Request::Leave);
                }
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
                    // "save@n": the save screen, as a save point's Square press opens it.
                    if page == "save" {
                        play.play.save_open = true;
                    } else if let (Some(m), Some(a)) = (menu.as_deref_mut(), art.as_deref()) {
                        m.flow.map_view = play.play.map_view();
                        m.flow.trace_open(&a.f, &page);
                    }
                }
            }
            // Co-op on one PC: player 2's Start opens the pause page.
            if std::mem::take(&mut coop.pause) {
                if let (Some(m), Some(a)) = (menu.as_deref_mut(), art.as_deref()) {
                    if m.flow.current == menus::PLAY {
                        m.flow.set(&a.f, menus::PAUSE);
                        play.play.menu_player = 1;
                        info!("co-op: player 2 opened the pause page");
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
                if netr.role != net::Role::Off {
                    netr.request = Some(net::Request::Leave);
                }
                if let (Some(m), Some(a)) = (menu.as_deref_mut(), art.as_deref()) {
                    m.flow.set(&a.f, menus::PLAY);
                }
            }
        }
        // The developer menu's warp list names: each world pack's map area (design table 5).
        if let Some(archive) = archive.as_deref_mut() {
            for pack in dev_menu::warp_names_wanted(&dev, &play.play) {
                let area = archive.index.read(&mut archive.disc, pack).ok().and_then(|b| {
                    let chain = neopets_data::resource::walk_chunks(&b, None).ok()?;
                    neopets_data::travel::map_area(&b, &chain.chunks)
                });
                dev.warp_names.insert(pack, area.and_then(|m| play.play.area_name(m)).unwrap_or_default());
            }
        }
        // SPEC-0021 Outfit items: the active hero wears the outfit its body entry (or the worn
        // cloak) names; a change loads that outfit's model and rig from the disc.
        if let Some(archive) = archive.as_deref_mut() {
            let h = play.play.hero_index.min(1);
            let want = play.play.wanted_outfit(h);
            if want != play.play.outfits[h] {
                match (archive_view::model(&mut archive.disc, &archive.index, want), archive_view::hero(&mut archive.disc, &archive.index, want)) {
                    (Ok(Some(m)), Ok(Some(a))) => {
                        info!("Outfit {want} for hero {h}");
                        play.play.set_outfit(want, m, a);
                    }
                    _ => {
                        error!("Outfit {want} could not be loaded");
                        play.play.outfits[h] = want;
                    }
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
                    // A cut world with no entry record: the hero by hand (SPEC-0066).
                    if let Some([x, y, z]) = dev.warp_point.take() {
                        play.play.place_at(x, z, y + 8.0);
                    }
                    // A cut character group into its owner world.
                    if let Some(region) = dev.group.take() {
                        play.play.dev_load_group(region);
                    }
                    info!("Warped to world pack {pack}");
                }
                Err(e) => {
                    dev.warp_point = None;
                    dev.group = None;
                    error!("Warp to {pack} failed: {e}");
                }
            }
        }
        // NEOPETS_SHOT_BOSS=n (diagnostic): the window starts as the developer menu's boss fight n,
        // god mode with NEOPETS_TRACE_GOD, so a traced moment can be pictured.
        static SHOT_BOSS: std::sync::Once = std::sync::Once::new();
        SHOT_BOSS.call_once(|| {
            if let Some(n) = std::env::var("NEOPETS_SHOT_BOSS").ok().and_then(|v| v.trim().parse::<usize>().ok()).filter(|&n| n < play::BOSSES.len()) {
                dev.boss = Some(n);
            }
        });
        if let (Some(i), Some(archive)) = (dev.boss.take(), archive.as_deref_mut()) {
            let b = play::BOSSES[i];
            match boss_fight(&mut archive.disc, &archive.index, &play.play, &b) {
                Ok(new) => {
                    play.play = new;
                    match archive.disc.reopen() {
                        Ok(second) => commands.insert_resource(Streamer::start(second, archive.index.clone(), play.play.boot.clone())),
                        Err(e) => error!("No loader thread, streaming is off: {e}"),
                    }
                    info!("Boss fight: {} in {}", b.name, b.pack);
                }
                Err(e) => error!("Boss fight {} failed: {e}", b.name),
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
                    // Online (net.rs): the joiner follows the host's game from here, in play.
                    if net_load.is_some() {
                        play.play.net_follow = true;
                        // The host's effects running at the welcome.
                        let table = play.play.effect_table.clone();
                        for op in std::mem::take(&mut netr.load_fx) {
                            let p = &mut play.play;
                            p.effects.apply(&table, &op, &mut p.net_fx, &mut p.rng);
                        }
                        info!("net: the welcome's game loaded with {} running effects", play.play.net_fx.len());
                        if let (Some(m), Some(a)) = (menu.as_deref_mut(), art.as_deref()) {
                            m.flow.set(&a.f, menus::PLAY);
                            m.flow.title_seen = true;
                        }
                    }
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
    let _span = crate::perf::span("browse_resources");
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
    let _span = crate::perf::span("orbit_camera");
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

/// The save states' folder (this build's tool): each state is `<id>.bin`, a one-slot save
/// file written as a save point would, and `<id>.txt` beside it: the world, place and facing
/// of the hero, then the player's name for it, then the date. NEOPETS_STATES_DIR moves it
/// (diagnostic, so a trace never touches the player's states).
fn states_dir() -> String {
    std::env::var("NEOPETS_STATES_DIR").unwrap_or_else(|_| "private/saves/states".to_string())
}

/// A save state under a new id, named `name`; returns its id.
fn save_state(play: &play::Play, name: &str) -> Result<String, String> {
    let dir = states_dir();
    let secs = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(0, |d| d.as_secs());
    let mut id = format!("s{secs}");
    let mut k = 1;
    while std::path::Path::new(&format!("{dir}/{id}.bin")).exists() {
        k += 1;
        id = format!("s{secs}-{k}");
    }
    let mut file = neopets_sim::save::SaveFile::default();
    let date = save_date();
    play.write_slot(&mut file.slots[0], &date);
    let (p, f) = (play.player.loco.position(), play.player.loco.facing);
    let name = name.replace(['\n', '\r'], " ");
    let place = format!("{} {} {} {} {} {} {}\n{}\n{}\n", play.world.pack, p[0], p[1], p[2], f[0], f[1], f[2], name.trim(), String::from_utf8_lossy(&date));
    write_save(&format!("{dir}/{id}.bin"), &file)?;
    std::fs::write(format!("{dir}/{id}.txt"), place).map_err(|e| e.to_string())?;
    Ok(id)
}

/// Save state `id` loaded as the menu's Load loads a slot, then the hero put back at its
/// place when the loaded world is the saved one. None when the state is missing.
fn load_state(disc: &mut Disc, index: &ArchiveIndex, old: &play::Play, id: &str) -> Option<Result<play::Play, String>> {
    let dir = states_dir();
    let file = read_save(&format!("{dir}/{id}.bin"))?;
    let text = std::fs::read_to_string(format!("{dir}/{id}.txt")).unwrap_or_default();
    let place: Vec<f32> = text.lines().next().unwrap_or("").split_whitespace().filter_map(|v| v.parse().ok()).collect();
    Some(load_game_from(disc, index, old, &file.slots[0]).map(|mut new| {
        if let [pack, x, y, z, fx, fy, fz] = place[..] {
            if new.world.pack == pack as u32 {
                new.state_place([x, y, z], [fx, fy, fz]);
            }
        }
        new
    }))
}

/// `--autoplay`: turn the autopilot on once play has started.
#[derive(Resource)]
struct AutoplayStart(bool);

/// F7 turns the autopilot (this build's test tool) on or off.
fn toggle_autopilot(keys: Res<ButtonInput<KeyCode>>, play: Option<ResMut<PlayState>>, mut start: ResMut<AutoplayStart>) {
    let Some(mut play) = play else { return };
    if keys.just_pressed(KeyCode::F7) || std::mem::take(&mut start.0) {
        play.play.toggle_autopilot();
        info!("Autopilot {}", if play.play.autopilot.is_some() { "on" } else { "off" });
    }
}

/// The autopilot's line at the bottom left while it plays.
#[derive(Component)]
struct AutoBox;

fn show_auto_box(mut commands: Commands, play: Option<Res<PlayState>>, mut boxes: Query<(Entity, &mut Text), With<AutoBox>>) {
    let line = play.as_ref().and_then(|p| p.play.autopilot_line());
    match (line, boxes.iter_mut().next()) {
        (Some(l), Some((_, mut t))) => {
            if t.0 != l {
                t.0 = l;
            }
        }
        (Some(l), None) => {
            commands.spawn((
                Text::new(l),
                TextFont { font_size: bevy::text::FontSize::Px(16.0), ..default() },
                TextColor(Color::srgb(0.7, 1.0, 0.75)),
                BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.55)),
                Node { position_type: PositionType::Absolute, bottom: px(12), left: px(12), padding: UiRect::all(px(6)), ..default() },
                GlobalZIndex(1001),
                AutoBox,
            ));
        }
        (None, Some((e, _))) => commands.entity(e).despawn(),
        (None, None) => {}
    }
}

/// The F6 boss box (this build's tool): a small box at the top right with the boss's short
/// readout, while a boss controller writes one.
#[derive(Component)]
struct BossBox;

/// Whether the F6 boss box is on.
#[derive(Resource, Default)]
struct BossBoxOn(bool);

fn show_boss_box(mut commands: Commands, on: Res<BossBoxOn>, play: Option<Res<PlayState>>, mut boxes: Query<(Entity, &mut Text), With<BossBox>>) {
    let lines = play.as_ref().filter(|_| on.0).map(|p| p.play.boss_lines().join("\n")).filter(|l| !l.is_empty());
    match (lines, boxes.iter_mut().next()) {
        (Some(l), Some((_, mut t))) => {
            if t.0 != l {
                t.0 = l;
            }
        }
        (Some(l), None) => {
            commands.spawn((
                Text::new(l),
                TextFont { font_size: bevy::text::FontSize::Px(17.0), ..default() },
                TextColor(Color::srgb(1.0, 0.92, 0.6)),
                BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.55)),
                Node { position_type: PositionType::Absolute, top: px(12), right: px(12), max_width: px(560), padding: UiRect::all(px(8)), ..default() },
                GlobalZIndex(1001),
                BossBox,
            ));
        }
        (None, Some((e, _))) => commands.entity(e).despawn(),
        (None, None) => {}
    }
}

/// F6 shows or hides the developer info panel (Tab is the game's Start key); during a boss
/// fight it toggles the small boss box instead.
fn toggle_hud(keys: Res<ButtonInput<KeyCode>>, mut huds: Query<&mut Node, With<Hud>>, play: Option<Res<PlayState>>, mut boss_box: ResMut<BossBoxOn>) {
    let _span = crate::perf::span("toggle_hud");
    if keys.just_pressed(KeyCode::F6) && play.as_ref().is_some_and(|p| !p.play.boss_lines().is_empty()) {
        boss_box.0 = !boss_box.0;
        return;
    }
    if keys.just_pressed(KeyCode::F6) {
        for mut node in &mut huds {
            node.display = match node.display {
                Display::None => Display::Flex,
                _ => Display::None,
            };
        }
    }
}

/// Whether the developer info panel is shown (F6). Its texts are written only then: a changed
/// text is laid out and measured again every frame, hidden or not (performance pass of
/// 2026-11-12: the two texts changed on every frame of play while the panel was hidden).
fn hud_shown(huds: &Query<&Node, With<Hud>>) -> bool {
    huds.iter().any(|n| n.display != Display::None)
}

/// Writes a text only when it differs (a write marks it changed even with the same words).
fn set_text(text: &mut Mut<Text>, s: String) {
    if text.0 != s {
        text.0 = s;
    }
}

fn show_sim(driver: Res<SimDriver>, play: Option<Res<PlayState>>, mut texts: Query<&mut Text, With<SimText>>, huds: Query<&Node, With<Hud>>) {
    let _span = crate::perf::span("show_sim");
    if !hud_shown(&huds) {
        return;
    }
    let d = &*driver;
    let play_line = play
        .map(|p| format!("{}\n{}\n{}\n", p.play.status(), p.play.status_camera(), p.play.status_worlds()))
        .unwrap_or_default();
    for mut text in &mut texts {
        set_text(&mut text, play_line.clone() + &format!(
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
        ));
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
    huds: Query<&Node, With<Hud>>,
) {
    let _span = crate::perf::span("show_browser");
    let Some(mut archive) = archive else {
        for mut text in &mut texts {
            set_text(&mut text, "RESOURCE BROWSER\nNo validated ISO, so the archive is unavailable.".into());
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
    // The view above is loaded whatever the panel shows (the viewer draws its model); the
    // panel's text is built only while it is shown.
    if !hud_shown(&huds) {
        return;
    }
    let entry = match &browser.typing {
        Some(t) => format!("Go to: {t}_   (Enter to jump, Esc to cancel)"),
        None => "Type a number and press Enter | [ ] step 1 | PgUp/PgDn step 100".into(),
    };
    for mut text in &mut texts {
        set_text(&mut text, format!("RESOURCE BROWSER - archive index (SPEC-0001/0002), textures (SPEC-0003)\n{}\n{model_line}\n{texture_line}\n{entry}", view.text));
    }
}

fn cycle_textures(keys: Res<ButtonInput<KeyCode>>, mut textures: ResMut<TextureView>) {
    let _span = crate::perf::span("cycle_textures");
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
/// the selected resource has no textures. The quad is the resource viewer's: in play it stays
/// hidden (it stood at (-3, 2, -3) in every world, showing the pack's first texture, and in the
/// middle of the Juppie arena, 1217, it could be seen).
fn apply_texture(
    play: Option<Res<PlayState>>,
    mut textures: ResMut<TextureView>,
    models: Res<ModelView>,
    mut quads: Query<
        (&MeshMaterial3d<StandardMaterial>, &mut Transform, &mut Visibility),
        With<TextureQuad>,
    >,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
) {
    let _span = crate::perf::span("apply_texture");
    if play.is_some() {
        for (_, _, mut visibility) in &mut quads {
            if *visibility != Visibility::Hidden {
                *visibility = Visibility::Hidden;
            }
        }
        return;
    }
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
    let _span = crate::perf::span("toggle_view");
    if keys.just_pressed(KeyCode::KeyV) && models.data.is_some() {
        models.show_model = !models.show_model;
        models.dirty = true;
        textures.dirty = true;
    }
}

/// Performance: whether a textured part can draw Opaque instead of Mask(0.5) with the same
/// result: no fragment is cut when the texture's lowest alpha times the part's lowest vertex
/// alpha is at least 0.5. Opaque keeps early depth rejection and draws the shadow and depth
/// passes without the texture.
fn mask_never_cuts(t: &neopets_data::model::ModelTexture, part: &neopets_data::model::MeshPart) -> bool {
    // NEOPETS_NO_OPAQUE_TWIN: every textured part keeps Mask(0.5) (diagnostic).
    if std::env::var_os("NEOPETS_NO_OPAQUE_TWIN").is_some() {
        return false;
    }
    let tex = t.image.rgba.chunks_exact(4).map(|p| p[3]).min().unwrap_or(255) as f32 / 255.0;
    let vertex = part.colors.iter().map(|c| c[3].min(1.0)).fold(1.0f32, f32::min);
    tex * vertex >= 0.5
}

/// The indices a Mask(0.5) part draws: a triangle whose three vertex alphas are all below 0.5
/// is cut at every pixel (the interpolated vertex alpha stays below 0.5 and the texture alpha
/// is at most 1), so it is left out. Bevy's depth prepass tests only the material and texture
/// alpha, so such a triangle would otherwise write depth and leave a black hole where the main
/// pass draws nothing (seen on Roberta's head with the sun or ambient occlusion on).
fn drawn_indices(part: &neopets_data::model::MeshPart) -> Vec<u32> {
    // Only a textured part draws Mask(0.5); an untextured one draws solid whatever its vertex
    // alpha, so it keeps every triangle.
    if part.colors.is_empty() || part.texture.is_none() || std::env::var_os("NEOPETS_KEEP_CUT").is_some() {
        return part.indices.clone();
    }
    let cut = |i: u32| part.colors.get(i as usize).is_none_or(|c| c[3].min(1.0) < 0.5);
    part.indices.chunks_exact(3).filter(|t| !(cut(t[0]) && cut(t[1]) && cut(t[2]))).flatten().copied().collect()
}

/// The opaque twin of a Mask(0.5) material (see `mask_never_cuts`).
/// A textured part whose vertex alpha fades: some vertices below 0.5 and some at or above it
/// (path edges laid over the grass). Drawn with Mask(0.5) its cut would follow the vertex alpha,
/// which Bevy's depth prepass does not see, so with the sun or ambient occlusion on the prepass
/// wrote depth where the main pass drew nothing: black holes in the ground (user's F8 view 3 in
/// 1230). Such a part is drawn blended with its vertex alpha instead (`blend_twin`), outside
/// the prepass.
fn vertex_fades(part: &neopets_data::model::MeshPart) -> bool {
    std::env::var_os("NEOPETS_NO_FADE_BLEND").is_none()
        && part.texture.is_some()
        && part.colors.iter().any(|c| c[3] < 0.5)
        && part.colors.iter().any(|c| c[3] >= 0.5)
}

/// The cut core of a soft material: Mask at the alpha test's reference, 120 of 128.
fn cut_twin(materials: &mut Assets<StandardMaterial>, mask: &Handle<StandardMaterial>) -> Handle<StandardMaterial> {
    let mut m = materials.get(mask).cloned().unwrap_or_default();
    m.alpha_mode = AlphaMode::Mask(120.0 / 128.0);
    materials.add(m)
}

/// The blended twin of a Mask(0.5) material (see `vertex_fades`).
/// It is a world surface, so it turns lit with the solid ones when world shadows or ambient
/// occlusion are on (`graphics::SCENE_BLENDS`).
fn blend_twin(materials: &mut Assets<StandardMaterial>, mask: &Handle<StandardMaterial>) -> Handle<StandardMaterial> {
    let mut m = materials.get(mask).cloned().unwrap_or_default();
    m.alpha_mode = AlphaMode::Blend;
    let h = materials.add(m);
    if let Ok(mut set) = graphics::SCENE_BLENDS.lock() {
        set.get_or_insert_with(Default::default).insert(h.id());
    }
    h
}

fn opaque_twin(materials: &mut Assets<StandardMaterial>, mask: &Handle<StandardMaterial>) -> Handle<StandardMaterial> {
    let mut m = materials.get(mask).cloned().unwrap_or_default();
    m.alpha_mode = AlphaMode::Opaque;
    materials.add(m)
}

fn model_texture(t: &neopets_data::model::ModelTexture) -> Image {
    use bevy::image::{ImageAddressMode, ImageFilterMode, ImageSamplerDescriptor};
    // With mipmaps for the Display page's distant-texture option (graphics.rs).
    let mut image = graphics::with_mipmaps(rgba_image(t.image.width, t.image.height, t.image.rgba.clone()));
    let wrap = |repeat: bool| {
        if repeat { ImageAddressMode::Repeat } else { ImageAddressMode::ClampToEdge }
    };
    // Materials on disc magnify bilinearly (TEX1 MMAG = 1, SPEC-0003).
    image.sampler = graphics::world_sampler(ImageSamplerDescriptor {
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
    let _span = crate::perf::span("apply_model");
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

/// NEOPETS_PERF: every 240 frames, the meshes by owner (scene, objects, foliage, dome,
/// characters and the rest), counting a mesh by its nearest tagged ancestor.
fn perf_kinds(meshes: Query<Entity, With<Mesh3d>>, parents: Query<&ChildOf>, parts: Query<&WorldPart>, chars: Query<(), Or<(With<CharacterRoot>, With<HeroRoot>)>>, mut frames: Local<u32>, mut events: MessageReader<AssetEvent<Mesh>>, assets: Res<Assets<Mesh>>, mut changed: Local<(usize, usize)>, users: Query<(Entity, &Mesh3d)>, mut by: Local<std::collections::HashMap<AssetId<Mesh>, usize>>, names: Query<&Name>, mut mat_events: MessageReader<AssetEvent<StandardMaterial>>, mut img_events: MessageReader<AssetEvent<Image>>, mut other: Local<(usize, usize)>, mut added: Local<(usize, usize, usize)>) {
    *frames += 1;
    other.0 += mat_events.read().filter(|e| matches!(e, AssetEvent::Modified { .. })).count();
    other.1 += img_events.read().filter(|e| matches!(e, AssetEvent::Modified { .. })).count();
    for e in events.read() {
        if let AssetEvent::Added { id } = e {
            added.0 += 1;
            added.1 += assets.get(*id).map_or(0, |m| m.count_vertices());
        }
        if let AssetEvent::Removed { .. } = e {
            added.2 += 1;
        }
        if let AssetEvent::Modified { id } = e {
            changed.0 += 1;
            changed.1 += assets.get(*id).map_or(0, |m| m.count_vertices());
            *by.entry(*id).or_default() += 1;
        }
    }
    if *frames % 240 != 0 {
        return;
    }
    // The owners of the meshes changed in the last frame window, by kind.
    info!("perf: meshes modified per frame {:.1}, vertices {:.0}; added {:.1} ({:.0} vertices), removed {:.1}; materials {:.1}, images {:.1}", changed.0 as f32 / 240.0, changed.1 as f32 / 240.0, added.0 as f32 / 240.0, added.1 as f32 / 240.0, added.2 as f32 / 240.0, other.0 as f32 / 240.0, other.1 as f32 / 240.0);
    *added = (0, 0, 0);
    *other = (0, 0);
    *changed = (0, 0);
    let mut owners: std::collections::BTreeMap<String, usize> = Default::default();
    for (e, m) in &users {
        let Some(&n) = by.get(&m.0.id()) else { continue };
        let root = std::iter::successors(Some(e), |&x| parents.get(x).ok().map(|p| p.parent())).last().unwrap_or(e);
        let kind = if chars.contains(root) { "characters".to_string() } else { names.get(root).map_or_else(|_| format!("{root:?}").split('v').next().unwrap_or("").to_string(), |n| n.to_string()) };
        let kind = if kind == "characters" { kind } else if parts.get(root).is_ok() { "world".into() } else { format!("other {}", if names.get(root).is_ok() { kind } else { String::new() }) };
        *owners.entry(kind).or_default() += n;
    }
    by.clear();
    info!("perf: mesh modifications by owner over 240 frames {owners:?}");
    let mut n: std::collections::BTreeMap<&str, usize> = Default::default();
    for e in &meshes {
        let kind = std::iter::successors(Some(e), |&x| parents.get(x).ok().map(|p| p.parent())).find_map(|x| {
            parts.get(x).ok().map(|p| match p {
                WorldPart::Scene(..) => "scene",
                WorldPart::Dome(_) => "dome",
                WorldPart::Foliage(..) => "foliage",
                WorldPart::Objects(..) => "objects",
                WorldPart::Stage => "stage",
                WorldPart::Field => "field",
                WorldPart::MaskTree(..) => "mask tree",
            }).or_else(|| chars.contains(x).then_some("characters"))
        });
        *n.entry(kind.unwrap_or("other")).or_default() += 1;
    }
    info!("perf: meshes by owner {n:?}");
}

/// Performance: write a component's value only when it differs (a write through `Mut` marks
/// the component changed, and Bevy then propagates and prepares the entity again).
fn set_if<T: PartialEq, D: std::ops::DerefMut<Target = T>>(d: &mut D, v: T) {
    if **d != v {
        **d = v;
    }
}

/// A drawn piece of a resident world in play mode: a world pack's scene or a sky dome.
#[derive(Component, Clone, Copy, PartialEq, Eq, Hash, Debug)]
enum WorldPart {
    /// The world's static scene with its world layer mask (SPEC-0046 Culling).
    Scene(u32, u32),
    Dome(u32),
    /// The world's foliage patches (SPEC-0060), and whether static patches are merged (the
    /// Display page's Full distant foliage): a change of that setting builds them again.
    Foliage(u32, bool),
    /// The world's placed objects (type 29), each on its own transform (SPEC-0028 7.1), built
    /// with the world's display mask word.
    Objects(u32, u32),
    /// The developer test stage's floor and ring (stage_view.rs; this build's own art).
    Stage,
    /// The reed field's plain floor (stage_view.rs; this build's own art).
    Field,
    /// Pack 1299's display node 401 (the role-401 tree of its type-8 container), drawn with the
    /// looping-door puzzle's own mask word, not the display word or the story layers
    /// (SPEC-0046 The pack 1299 mask tree; SPEC-0028).
    MaskTree(u32, u32),
}

/// A placed object's root entity: (pack, object index).
#[derive(Component, Clone, Copy)]
struct ObjectRoot(u32, usize);

/// Each frame: a placed object's root takes its drawn transform from the play, or hides.
/// SPEC-0033 Behavior 1C step 4: the motor send of the last tick goes to pad 0's large motor
/// (the first connected controller's strong motor) for its countdown, replacing what it played.
fn send_rumble(state: Option<ResMut<PlayState>>, gamepads: Query<Entity, With<Gamepad>>, mut out: MessageWriter<bevy::input::gamepad::GamepadRumbleRequest>, coop: Res<coop::Coop>) {
    let _span = crate::perf::span("send_rumble");
    let Some(mut state) = state else { return };
    // Co-op on one PC: player 2's send goes to its own controller.
    if let (Some(send), Some(gamepad)) = (state.play.others.first_mut().and_then(|o| o.motor.take()), coop.pad) {
        out.write(bevy::input::gamepad::GamepadRumbleRequest::Stop { gamepad });
        if send.level > 0 && send.count > 0 {
            out.write(bevy::input::gamepad::GamepadRumbleRequest::Add {
                gamepad,
                duration: std::time::Duration::from_secs_f32(send.count as f32 / 60.0),
                intensity: bevy::input::gamepad::GamepadRumbleIntensity::strong_motor(send.level as f32 / 255.0),
            });
        }
    }
    let Some(send) = state.play.motor.take() else { return };
    let Some(gamepad) = gamepads.iter().filter(|g| Some(*g) != coop.pad).min() else { return };
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
    let _span = crate::perf::span("sync_blocks");
    let Some(state) = state else { return };
    for (b, mut t) in &mut roots {
        if let Some(m) = state.play.block_matrix(b.0, b.1) {
            set_if(&mut t, Transform::from_matrix(Mat4::from_cols_array(&m)));
        }
    }
}

/// A drawn breakable, by (pack, entry index).
#[derive(Component)]
struct BreakableRoot(u32, usize);

/// Breakables are drawn in states 1 and 3 with their own matrix (SPEC-0048 Breakable draw).
fn sync_breakables(state: Option<Res<PlayState>>, mut roots: Query<(&BreakableRoot, &mut Visibility)>) {
    let _span = crate::perf::span("sync_breakables");
    let Some(state) = state else { return };
    for (b, mut vis) in &mut roots {
        let want = if state.play.breakable_matrix(b.0, b.1).is_some() { Visibility::Visible } else { Visibility::Hidden };
        if *vis != want {
            *vis = want;
        }
    }
}

fn sync_objects(state: Option<Res<PlayState>>, mut roots: Query<(&ObjectRoot, &mut Transform, &mut Visibility)>) {
    let _span = crate::perf::span("sync_objects");
    let Some(state) = state else { return };
    for (o, mut t, mut vis) in &mut roots {
        match state.play.object_matrix(o.0, o.1) {
            Some(m) => {
                set_if(&mut t, Transform::from_matrix(Mat4::from_cols_array(&m)));
                set_if(&mut vis, Visibility::Visible);
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
    mut pass1_materials: ResMut<Assets<pass1::Pass1Material>>,
    mut images: ResMut<Assets<Image>>,
    mut shown: Local<Vec<WorldPart>>,
    mut built: Local<std::collections::HashMap<WorldPart, scene_merge::Built>>,
    mut overlays: ResMut<scene_merge::Overlays>,
    mut ext_materials: ResMut<Assets<scene_merge::OverlayMaterial>>,
    mut sigs: ResMut<SceneSigs>,
    mut pending_fresh: Local<Vec<WorldPart>>,
    settings: Res<display::DisplaySettings>,
) {
    let _span = crate::perf::span("sync_worlds");
    let (Some(state), Some(mut archive)) = (play, archive) else { return };
    let p = &state.play;
    let mask = |pack: u32| p.display_mask.get(&pack).copied().unwrap_or(0);
    // NEOPETS_SCENE_MASK=n: the world layer mask forced (diagnostic).
    let forced = std::env::var("NEOPETS_SCENE_MASK").ok().and_then(|v| v.parse::<u32>().ok());
    let layers = |pack: u32| forced.unwrap_or(mask(pack) | p.story_layers());
    if std::env::var_os("NEOPETS_SCENE_MASK_LOG").is_some() {
        info!("scene layers of {}: display {} story {:#x}", p.world.pack, mask(p.world.pack), p.story_layers());
    }
    let mut want = vec![WorldPart::Scene(p.world.pack, layers(p.world.pack)), WorldPart::Objects(p.world.pack, mask(p.world.pack))];
    // NEOPETS_PERF_NOFOLIAGE: no foliage drawn (diagnostic, to measure its cost).
    let no_foliage = std::env::var_os("NEOPETS_PERF_NOFOLIAGE").is_some();
    let merge_foliage = !settings.original_foliage && foliage_view::merge_enabled();
    if p.foliage.contains_key(&p.world.pack) && !no_foliage {
        want.push(WorldPart::Foliage(p.world.pack, merge_foliage));
    }
    if let (true, Some(o)) = (p.other_drawn(), &p.other) {
        want.push(WorldPart::Scene(o.pack, layers(o.pack)));
        want.push(WorldPart::Objects(o.pack, mask(o.pack)));
        if p.foliage.contains_key(&o.pack) && !no_foliage {
            want.push(WorldPart::Foliage(o.pack, merge_foliage));
        }
    }
    if let Some((pack, m)) = p.mask_tree() {
        want.push(WorldPart::MaskTree(pack, m));
    }
    let slot = p.boot.areas.slot_of(p.world.pack);
    // NEOPETS_PERF_NODOME: no sky dome (diagnostic, to measure its cost).
    if std::env::var_os("NEOPETS_PERF_NODOME").is_none() {
        want.push(WorldPart::Dome(neopets_data::area::dome_for_slot(slot)));
    }
    // The test stage has no disc scene: its own floor under the default sky dome.
    if p.on_test_stage() {
        // The reed field's world is a plain floor; the test stage has the arena.
        let floor = if p.world.pack == play::FIELD_PACK { WorldPart::Field } else { WorldPart::Stage };
        want = vec![floor, WorldPart::Dome(neopets_data::area::dome_for_slot(None))];
        // The reed field's plants.
        if p.foliage.get(&p.world.pack).is_some_and(|v| !v.is_empty()) && !no_foliage {
            want.push(WorldPart::Foliage(p.world.pack, merge_foliage));
        }
    }
    // NEOPETS_NO_OBJECTS / NEOPETS_NO_MASKTREE: those world parts left out (diagnostic).
    if std::env::var_os("NEOPETS_NO_OBJECTS").is_some() {
        want.retain(|w| !matches!(w, WorldPart::Objects(..)));
    }
    if std::env::var_os("NEOPETS_NO_MASKTREE").is_some() {
        want.retain(|w| !matches!(w, WorldPart::MaskTree(..)));
    }
    if *shown == want {
        return;
    }
    for (e, part) in &parts {
        if !want.contains(part) {
            commands.entity(e).despawn();
        }
    }
    built.retain(|k, _| want.contains(k));
    pending_fresh.retain(|k| want.contains(k));
    let mut fresh: Vec<WorldPart> = std::mem::take(&mut *pending_fresh);
    let Archive { disc, index } = &mut *archive;
    // World parts are built until this frame's budget is spent and the rest on later frames, so
    // a load is several short frames rather than one long one.
    let started = std::time::Instant::now();
    let mut done: Vec<WorldPart> = shown.iter().copied().filter(|k| want.contains(k)).collect();
    let mut deferred = false;
    for part in &want {
        if shown.contains(part) {
            continue;
        }
        if started.elapsed().as_secs_f32() * 1000.0 > WORLD_BUILD_BUDGET_MS {
            deferred = true;
            break;
        }
        done.push(*part);
        if let WorldPart::Objects(pack, layers) = *part {
            let world = [&p.world].into_iter().chain(p.other.as_ref()).find(|w| w.pack == pack);
            let _load = crate::perf::span("load:objects");
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
                        let parts = flip_parts(&m, part_assets(&m, &mut meshes, &mut materials, &mut images), &mut materials);
                        let second = pass1_assets(&m, &mut meshes, &mut pass1_materials, &mut images);
                        commands.spawn((Transform::IDENTITY, Visibility::Visible, BlockRoot(pack, i), *part)).with_children(|c| {
                            for (_, mesh, material, flip) in parts {
                                let mut e = c.spawn((Mesh3d(mesh), MeshMaterial3d(material), Transform::IDENTITY));
                                if let Some(f) = flip {
                                    e.insert(UvFlip(f));
                                }
                            }
                            for (_, mesh, material, scroll) in second {
                                let mut e = c.spawn((Mesh3d(mesh), MeshMaterial3d(material), Transform::IDENTITY));
                                if let Some(s) = scroll {
                                    e.insert(pass1::Pass1Scroll(s));
                                }
                            }
                        });
                    }
                    for (i, matrix, m) in breakables {
                        let parts = flip_parts(&m, part_assets(&m, &mut meshes, &mut materials, &mut images), &mut materials);
                        let second = pass1_assets(&m, &mut meshes, &mut pass1_materials, &mut images);
                        commands.spawn((Transform::from_matrix(Mat4::from_cols_array(&matrix)), Visibility::Visible, BreakableRoot(pack, i), *part)).with_children(|c| {
                            for (_, mesh, material, flip) in parts {
                                let mut e = c.spawn((Mesh3d(mesh), MeshMaterial3d(material), Transform::IDENTITY));
                                if let Some(f) = flip {
                                    e.insert(UvFlip(f));
                                }
                            }
                            for (_, mesh, material, scroll) in second {
                                let mut e = c.spawn((Mesh3d(mesh), MeshMaterial3d(material), Transform::IDENTITY));
                                if let Some(s) = scroll {
                                    e.insert(pass1::Pass1Scroll(s));
                                }
                            }
                        });
                    }
                    for (i, m) in list {
                        let parts = flip_parts(&m, part_assets(&m, &mut meshes, &mut materials, &mut images), &mut materials);
                        let second = pass1_assets(&m, &mut meshes, &mut pass1_materials, &mut images);
                        commands.spawn((Transform::IDENTITY, Visibility::Visible, ObjectRoot(pack, i), *part)).with_children(|c| {
                            for (_, mesh, material, flip) in parts {
                                let mut e = c.spawn((Mesh3d(mesh), MeshMaterial3d(material), Transform::IDENTITY));
                                if let Some(f) = flip {
                                    e.insert(UvFlip(f));
                                }
                            }
                            for (_, mesh, material, scroll) in second {
                                let mut e = c.spawn((Mesh3d(mesh), MeshMaterial3d(material), Transform::IDENTITY));
                                if let Some(s) = scroll {
                                    e.insert(pass1::Pass1Scroll(s));
                                }
                            }
                        });
                    }
                }
                Err(e) => error!("objects of {pack}: {e}"),
            }
            continue;
        }
        if let WorldPart::Stage = *part {
            stage_view::spawn(&mut commands, &mut meshes, &mut materials, &mut images, *part);
            continue;
        }
        if let WorldPart::Field = *part {
            stage_view::spawn_floor(&mut commands, &mut meshes, &mut materials, &mut images, *part);
            continue;
        }
        if let WorldPart::Foliage(pack, merge) = *part {
            let _load = crate::perf::span("load:foliage");
            match foliage_view::models(disc, index, p, pack) {
                Ok(models) => {
                    let n = foliage_view::spawn(&mut commands, p, pack, &models, *part, &mut meshes, &mut materials, &mut pass1_materials, &mut images, merge);
                    info!("foliage of {pack}: {n} plants of {} models", models.len());
                }
                Err(e) => error!("foliage of {pack}: {e}"),
            }
            continue;
        }
        // SPEC-0046 Sky domes, step 6: the dome's role-10 layer, drawn while the atmosphere
        // blend factor b is above 0 with b as its opacity (sync_haze); play had none, so a gated
        // purple zone such as pack 1230's showed no haze (user's report 71).
        if let WorldPart::Dome(d) = *part {
            let haze = archive_view::dome_haze(disc, index, d);
            if std::env::var_os("NEOPETS_TRACE_ATMOS").is_some() {
                eprintln!("ATMOS dome {d} role-10 layer: {:?}", haze.as_ref().map(|m| m.as_ref().map(|m| (m.parts.len(), m.min, m.max))));
            }
            if let Ok(Some(mut m)) = haze {
                // The dome draws first (SPEC-0046 Frame order); Bevy sorts the blended draws by
                // their origin's distance, and the layer's origin at the world origin tied with
                // the scene's, so it drew over the blended road parts of Meridell (user's report
                // 74). Its origin goes far above the world with the mesh moved back, so it always
                // sorts farthest and draws first.
                const FAR: [f32; 3] = [0.0, 100_000.0, 0.0];
                for part in &mut m.parts {
                    for p in &mut part.positions {
                        p[1] -= FAR[1];
                    }
                }
                for (_, mesh, material) in part_assets(&m, &mut meshes, &mut materials, &mut images) {
                    let mut own = materials.get(&material).cloned().unwrap_or_default();
                    own.alpha_mode = AlphaMode::Blend;
                    own.unlit = true;
                    let h = materials.add(own);
                    commands.spawn((Mesh3d(mesh), MeshMaterial3d(h.clone()), Transform::from_translation(Vec3::from(FAR)), *part, Visibility::Hidden, HazeLayer(h), bevy::light::NotShadowCaster, bevy::light::NotShadowReceiver, draw_order::DrawRank { pack: None, rank: 150_000 }));
                }
            }
        }
        let streamed = match (*part, streamer.as_deref_mut()) {
            // The streamed scene is built with mask 0.
            (WorldPart::Scene(pack, 0), Some(s)) => s.scenes.remove(&pack),
            _ => None,
        };
        let model = match *part {
            WorldPart::Scene(..) if streamed.is_some() => Ok(streamed),
            WorldPart::Scene(pack, layers) => {
                let _load = crate::perf::span("load:scene_read");
                index.read(disc, pack).and_then(|bytes| {
                    let chain = neopets_data::resource::walk_chunks(&bytes, None)?;
                    archive_view::world_scene(disc, index, pack, &bytes, &chain.chunks, false, layers)
                })
            }
            WorldPart::Dome(d) => archive_view::dome(disc, index, d),
            WorldPart::MaskTree(pack, m) => index.read(disc, pack).and_then(|bytes| {
                let chain = neopets_data::resource::walk_chunks(&bytes, None)?;
                let data = neopets_data::model::build_masked(&bytes, &chain.chunks, None, Some(&[401]), Some(m))?;
                // NEOPETS_TRACE_MASKTREE: the parts the mask keeps and their bounds (diagnostic).
                if std::env::var_os("NEOPETS_TRACE_MASKTREE").is_some() {
                    let (lo, hi) = data.parts.iter().flat_map(|p| p.positions.iter()).fold(([f32::MAX; 3], [f32::MIN; 3]), |(lo, hi), q| (std::array::from_fn(|k| lo[k].min(q[k])), std::array::from_fn(|k| hi[k].max(q[k]))));
                    println!("MASKTREE {pack} mask {m:#x}: {} parts, box {lo:?}..{hi:?}", data.parts.len());
                }
                Ok((!data.parts.is_empty()).then_some(data))
            }),
            WorldPart::Foliage(..) | WorldPart::Objects(..) | WorldPart::Stage | WorldPart::Field => Ok(None),
        };
        match model {
            // A scene is kept and spawned below, merged (scene_merge.rs), once the twins of the
            // other drawn scenes are known.
            Ok(Some(m)) if matches!(*part, WorldPart::Scene(..)) => {
                if let WorldPart::Scene(pack, _) = *part {
                    let _load = crate::perf::span("load:scene_textures");
                    built.insert(*part, scene_merge::Built::new(m, pack, &mut materials, &mut images));
                    fresh.push(*part);
                }
            }
            Ok(Some(m)) => {
                // An environment-mapped pass 0 is drawn by the pass-1 shader with view-generated
                // UVs (pass1.rs), its texture shared per model texture.
                let mut env_textures: std::collections::HashMap<usize, Handle<Image>> = std::collections::HashMap::new();
                let ranks = draw_order::ranks(&m);
                for (i, mesh, material) in part_assets(&m, &mut meshes, &mut materials, &mut images) {
                    let env = m.parts[i].texture.filter(|_| m.parts[i].env_map && std::env::var_os("NEOPETS_NO_PASS1").is_none());
                    let mut e = match env {
                        Some(t) => {
                            if std::env::var_os("NEOPETS_TRACE_PASS1").is_some() {
                                let q = &m.parts[i].positions;
                                let n = q.len().max(1) as f32;
                                let c = q.iter().fold([0.0f32; 3], |a, p| [a[0] + p[0] / n, a[1] + p[1] / n, a[2] + p[2] / n]);
                                println!("env pass 0 part {i}: middle ({:.0}, {:.0}, {:.0}), {} vertices", c[0], c[1], c[2], q.len());
                            }
                            let image = env_textures.entry(t).or_insert_with(|| images.add(model_texture(&m.textures[t]))).clone();
                            commands.spawn((Mesh3d(mesh), MeshMaterial3d(pass1_materials.add(pass1::env_base_for(image, m.parts[i].alpha))), Transform::IDENTITY, *part))
                        }
                        None => commands.spawn((Mesh3d(mesh), MeshMaterial3d(material.clone()), Transform::IDENTITY, *part)),
                    };
                    // The sky dome neither casts nor takes the world shadows.
                    if matches!(*part, WorldPart::Dome(_)) {
                        e.insert((bevy::light::NotShadowCaster, bevy::light::NotShadowReceiver, draw_order::DrawRank { pack: None, rank: ranks[i] }));
                    }
                    // A part under a type-3 node, or of an instance or placed object, draws only
                    // while its zone column passes.
                    if let (WorldPart::Scene(pack, _), Some(z)) = (*part, m.parts[i].zone) {
                        e.insert(ZoneGate(pack, z));
                    }
                    if let WorldPart::Scene(pack, _) = *part {
                        e.insert(part_signature(pack, &m.parts[i], false));
                    }
                    // SPEC-0046 Instanced meshes: an instance's parts draw only within the
                    // record's draw distance D1 of the eye (F is 0 on disc: no fade), with
                    // Distant foliage set to Original.
                    if let Some((at, d1, _)) = m.parts[i].cull {
                        e.insert(DrawCull(Vec3::from(at), d1));
                    }
                    if let Some(speed) = m.parts[i].scroll.filter(|_| m.parts[i].texture.is_some()) {
                        e.insert(UvScroll(speed));
                    }
                    // A variant-1 flipbook steps its own copy of the material through the cells.
                    if let (Some(fb), None) = (m.parts[i].flipbook, env) {
                        if std::env::var_os("NEOPETS_TRACE_PASS1").is_some() {
                            let q = &m.parts[i].positions;
                            let n = q.len().max(1) as f32;
                            let c = q.iter().fold([0.0f32; 3], |a, p| [a[0] + p[0] / n, a[1] + p[1] / n, a[2] + p[2] / n]);
                            println!("flipbook part {i}: middle ({:.0}, {:.0}, {:.0}), {:?}", c[0], c[1], c[2], fb);
                        }
                        if let Some(own) = materials.get(&material).cloned() {
                            e.insert((MeshMaterial3d(materials.add(own)), UvFlip(fb)));
                        }
                    }
                }
                // Render pass 1 of the two-pass parts (pass1.rs), gated like their pass 0; its
                // signature is kept apart so the duplicate test never hides it as pass 0's twin.
                for (i, mesh, material, scroll) in pass1_assets(&m, &mut meshes, &mut pass1_materials, &mut images) {
                    let mut e = commands.spawn((Mesh3d(mesh), MeshMaterial3d(material), Transform::IDENTITY, *part));
                    if let (WorldPart::Scene(pack, _), Some(z)) = (*part, m.parts[i].zone) {
                        e.insert(ZoneGate(pack, z));
                    }
                    if let WorldPart::Scene(pack, _) = *part {
                        e.insert(part_signature(pack, &m.parts[i], true));
                    }
                    if matches!(*part, WorldPart::Dome(_)) {
                        e.insert(draw_order::DrawRank { pack: None, rank: ranks[i] + 1 });
                    }
                    if let Some(speed) = scroll {
                        e.insert(pass1::Pass1Scroll(speed));
                    }
                }
            }
            Ok(None) => {}
            Err(e) => error!("{part:?} could not be built: {e}"),
        }
    }
    // The scenes wait until every wanted part is built, so a scene is not spawned once and again
    // when its neighbour's twins arrive a frame later.
    if deferred {
        *shown = done;
        *pending_fresh = fresh;
        return;
    }
    // Spawn the new scenes, and spawn again an older scene in which a new scene found twins (its
    // twins must be separate entities for the duplicate rule of sync_zones).
    let keys: Vec<WorldPart> = built.keys().copied().collect();
    for k in keys {
        let WorldPart::Scene(pack, _) = k else { continue };
        let twins = {
            let others: Vec<&[PartSig]> = built.iter().filter(|(o, _)| matches!(o, WorldPart::Scene(op, _) if *op != pack)).map(|(_, b)| &b.sigs[..]).collect();
            built[&k].twins(&others)
        };
        let is_fresh = fresh.contains(&k);
        let b = built.get_mut(&k).expect("kept scene");
        let grew = twins.iter().zip(&b.shared).any(|(t, s)| *t && !*s);
        if !is_fresh && !grew {
            continue;
        }
        if !is_fresh {
            for (e, p) in &parts {
                if *p == k {
                    commands.entity(e).despawn();
                }
            }
        }
        for (s, t) in b.shared.iter_mut().zip(twins) {
            *s |= t;
        }
        let _load = crate::perf::span("load:scene_spawn");
        let started = std::time::Instant::now();
        let n = scene_merge::spawn(&mut commands, k, pack, b, &mut meshes, &mut materials, &mut pass1_materials, &mut images, &mut overlays, &mut ext_materials);
        // NEOPETS_TRACE_LAYER: also the time each scene took to spawn (diagnostic).
        if std::env::var_os("NEOPETS_TRACE_LAYER").is_some() {
            println!("SPAWN {pack} {:.1} ms", started.elapsed().as_secs_f64() * 1000.0);
        }
        sigs.0.insert(pack, b.sigs.clone());
        info!("scene of {pack}: {} parts in {n} entities ({} twins)", b.model.parts.len(), b.shared.iter().filter(|&&s| s).count());
    }
    *shown = want;
}

/// How long one frame may spend building world parts, in milliseconds (a part in progress
/// finishes).
const WORLD_BUILD_BUDGET_MS: f32 = 6.0;

/// The parts with their flipbooks: a flipbook part gets its own copy of the material, which
/// the flipbook steps.
fn flip_parts(
    m: &neopets_data::model::ModelData,
    parts: Vec<(usize, Handle<Mesh>, Handle<StandardMaterial>)>,
    materials: &mut Assets<StandardMaterial>,
) -> Vec<(usize, Handle<Mesh>, Handle<StandardMaterial>, Option<neopets_data::material::Flipbook>)> {
    parts
        .into_iter()
        .map(|(i, mesh, material)| match m.parts[i].flipbook {
            Some(f) => {
                let own = materials.get(&material).cloned().map_or(material, |c| materials.add(c));
                (i, mesh, own, Some(f))
            }
            None => (i, mesh, material, None),
        })
        .collect()
}

/// A world part's variant-1 flipbook (SPEC-0003 Sub-texture frame addressing).
#[derive(Component, Clone, Copy)]
struct UvFlip(neopets_data::material::Flipbook);

/// Each frame: a flipbook part's UV offset is its cell at the render frame counter (the play
/// tick, which holds under a pause as the cells do).
fn sync_uv_flip(state: Option<Res<PlayState>>, parts: Query<(&UvFlip, &MeshMaterial3d<StandardMaterial>)>, mut materials: ResMut<Assets<StandardMaterial>>) {
    let _span = crate::perf::span("sync_uv_flip");
    if std::env::var_os("NEOPETS_PERF_NOSCROLL").is_some() {
        return;
    }
    let Some(state) = state else { return };
    let n = state.play.ticks as u16;
    // NEOPETS_TRACE_FLIP: the flipbook parts and their offsets every 60 ticks (diagnostic).
    if std::env::var_os("NEOPETS_TRACE_FLIP").is_some() && n % 60 == 0 {
        println!("flipbooks on tick {n}: {} parts, offsets {:?}", parts.iter().count(), parts.iter().map(|(f, _)| f.0.offset(n)).collect::<Vec<_>>());
    }
    // Performance: only a changed offset is written (a get_mut marks the material changed,
    // which the renderer prepares again).
    for (f, m) in &parts {
        let off = f.0.offset(n);
        let want = bevy::math::Affine2::from_translation(Vec2::new(off[0], off[1]));
        if materials.get(&m.0).is_some_and(|mat| mat.uv_transform != want) {
            if let Some(mut mat) = materials.get_mut(&m.0) {
                mat.uv_transform = want;
            }
        }
    }
}

/// A world part's UV scroll speeds per pass (SPEC-0003 UV mode 2).
#[derive(Component, Clone, Copy)]
pub(crate) struct UvScroll([f32; 2]);

/// Each frame: a scrolling part's UV offset is frac(s x N) per axis, N the 16-bit pass counter
/// at 60 passes a second of real time: the scroll never reads game time, so it runs on during a
/// pause or a cutscene and does not slow with a time scale (SPEC-0003, Speed per second).
fn sync_uv_scroll(state: Option<Res<PlayState>>, time: Res<Time<bevy::time::Real>>, parts: Query<(&UvScroll, &MeshMaterial3d<StandardMaterial>)>, mut materials: ResMut<Assets<StandardMaterial>>) {
    let _span = crate::perf::span("sync_uv_scroll");
    if std::env::var_os("NEOPETS_PERF_NOSCROLL").is_some() {
        return;
    }
    if state.is_none() {
        return;
    }
    let n = (time.elapsed_secs_f64() * 60.0) as u64 as u16;
    for (s, m) in &parts {
        let off = Vec2::new(neopets_data::material::scroll_offset(s.0[0], n), neopets_data::material::scroll_offset(s.0[1], n));
        let want = bevy::math::Affine2::from_translation(off);
        if materials.get(&m.0).is_some_and(|mat| mat.uv_transform != want) {
            if let Some(mut mat) = materials.get_mut(&m.0) {
                mat.uv_transform = want;
            }
        }
    }
}

/// A world part's zone gate: (world pack, zone column).
#[derive(Component, Clone, Copy)]
struct ZoneGate(u32, u8);

/// A type-20 instance part's translation and draw distance D1 (SPEC-0046 Instanced meshes).
#[derive(Component)]
struct DrawCull(Vec3, f32);

/// NEOPETS_DUP_LEVEL: how many of the wider duplicate rules apply: 0 exact repeats only, 1 the
/// near-copies of large parts, 2 (default) and the 40 percent box overlap, 3 and the 90 percent
/// containment, 4 and the cover by several parts. Levels 3 and 4 hid 696's own canyon walls
/// beside 1023 (the user's F8 views 1 and 2), so they are off.
pub fn dup_level() -> u8 {
    // Read once: the part comparisons call this for every pair, hundreds of thousands of times
    // when a world loads beside another.
    static LEVEL: std::sync::OnceLock<u8> = std::sync::OnceLock::new();
    *LEVEL.get_or_init(|| std::env::var("NEOPETS_DUP_LEVEL").ok().and_then(|v| v.parse().ok()).unwrap_or(2))
}

/// Every built scene's part signatures by pack, merged parts included (the duplicate rule of
/// `sync_zones` compares the other world's parts with all of the current world's).
#[derive(Resource, Default)]
struct SceneSigs(std::collections::HashMap<u32, Vec<PartSig>>);

/// A world scene part's pack and geometry signature: whether it is a second-pass part, its
/// vertex count and its bounds.
#[derive(Component, Clone, Copy)]
pub struct PartSig {
    pub pack: u32,
    pass1: bool,
    pub count: usize,
    pub lo: [f32; 3],
    hi: [f32; 3],
}

fn part_signature(pack: u32, p: &neopets_data::model::MeshPart, pass1: bool) -> PartSig {
    let (lo, hi) = p.positions.iter().fold(([f32::MAX; 3], [f32::MIN; 3]), |(lo, hi), q| (std::array::from_fn(|k| lo[k].min(q[k])), std::array::from_fn(|k| hi[k].max(q[k]))));
    PartSig { pack, pass1, count: p.positions.len(), lo, hi }
}

impl PartSig {
    /// The same geometry: the same kind and vertex count, every bound within half a unit (the
    /// two packs place a shared mesh through their own matrices, which differ in the last
    /// digits).
    /// A large part (over 200 units across) also repeats one whose bounds all lie within 5
    /// percent of its size, and a mountain-sized one (over 800) within 25 percent, whatever
    /// the vertex count: 696 carries lower-detail copies of 1023's ring of mountains (900 to
    /// 1700 units across, bounds 18 to 250 apart) that overlap them and fight for depth (user
    /// report: the mountains flicker; gone with the second world hidden). A part over 100 units
    /// whose box overlaps by 40 percent or more (intersection over union) is a copy too: the
    /// cliffs and castle rock behind the town (696 parts 64 to 71) and the north cliffs. And a
    /// part whose box lies at least 90 percent inside this one's: 696 holds a low-detail copy
    /// of the whole town (rocks, bushes, ground pieces) inside 1023's own geometry.
    pub fn diff(&self, o: &PartSig) -> f32 {
        (0..3).map(|k| (self.lo[k] - o.lo[k]).abs().max((self.hi[k] - o.hi[k]).abs())).fold(0.0f32, f32::max)
    }

    pub fn size(&self) -> f32 {
        (0..3).map(|k| self.hi[k] - self.lo[k]).fold(0.0f32, f32::max)
    }

    fn same(&self, o: &PartSig) -> bool {
        if self.pass1 != o.pass1 {
            return false;
        }
        let diff = (0..3).map(|k| (self.lo[k] - o.lo[k]).abs().max((self.hi[k] - o.hi[k]).abs())).fold(0.0f32, f32::max);
        if self.count == o.count && diff <= 0.5 {
            return true;
        }
        let size = (0..3).map(|k| self.hi[k] - self.lo[k]).fold(0.0f32, f32::max);
        // NEOPETS_DUP_LEVEL=k (diagnostic): only the first k of the wider rules below.
        let level = dup_level();
        (level >= 1 && ((size > 200.0 && diff <= 0.05 * size) || (size > 800.0 && diff <= 0.25 * size)))
            // Only 696 and 1023 hold such low-detail copies: elsewhere overlapping boxes are different
            // pieces, and hiding them cut holes in 1066's mountains (user's F8 view 13).
            || (matches!((self.pack, o.pack), (696, 1023) | (1023, 696)) && ((level >= 2 && size > 100.0 && self.overlap(o) >= 0.4) || (level >= 3 && self.contains(o) >= 0.9)))
    }

    /// The part of `o`'s box that lies inside this one's (each extent at least 1 unit).
    pub fn contains(&self, o: &PartSig) -> f32 {
        let vol = |lo: [f32; 3], hi: [f32; 3]| (0..3).map(|k| (hi[k] - lo[k]).max(1.0)).product::<f32>();
        let (lo, hi): ([f32; 3], [f32; 3]) = (std::array::from_fn(|k| self.lo[k].max(o.lo[k])), std::array::from_fn(|k| self.hi[k].min(o.hi[k])));
        if (0..3).any(|k| hi[k] < lo[k]) {
            return 0.0;
        }
        vol(lo, hi) / vol(o.lo, o.hi)
    }

    /// The bounding boxes' intersection over their union (each extent at least 1 unit).
    pub fn overlap(&self, o: &PartSig) -> f32 {
        let vol = |lo: [f32; 3], hi: [f32; 3]| (0..3).map(|k| (hi[k] - lo[k]).max(1.0)).product::<f32>();
        let (lo, hi): ([f32; 3], [f32; 3]) = (std::array::from_fn(|k| self.lo[k].max(o.lo[k])), std::array::from_fn(|k| self.hi[k].min(o.hi[k])));
        if (0..3).any(|k| hi[k] <= lo[k]) {
            return 0.0;
        }
        let i = vol(lo, hi);
        i / (vol(self.lo, self.hi) + vol(o.lo, o.hi) - i)
    }
}

/// Each frame: a gated world part shows only while its zone query passes. A part of the other
/// drawn world that repeats a part of the current world exactly (the same vertex count and
/// bounds within half a unit; 696 and 1023 share their castle and mountain parts) is hidden:
/// drawn twice in one place the two copies fight for depth and flicker (this build's draw
/// rule, not a spec rule).
fn sync_zones(
    state: Option<Res<PlayState>>,
    mut parts: Query<(Entity, Option<&ZoneGate>, Option<&PartSig>, Option<&DrawCull>, &mut Visibility), Or<(With<ZoneGate>, With<PartSig>, With<DrawCull>)>>,
    cameras: Query<&GlobalTransform, (With<Camera3d>, Without<crate::water_reflect::MirrorCam>)>,
    settings: Res<display::DisplaySettings>,
    mut toggles: Local<(std::collections::HashMap<Entity, (u32, String)>, u32)>,
    sigs: Res<SceneSigs>,
    mut dups: Local<(u32, usize, std::collections::HashMap<Entity, bool>)>,
) {
    let _span = crate::perf::span("sync_zones");
    let Some(state) = state else { return };
    // The instance draw distance applies only with Distant foliage set to Original; Full (the
    // default) draws every instance at any distance.
    let eye = cameras.iter().next().map(|g| g.translation()).filter(|_| settings.original_foliage);
    let current = state.play.world.pack;
    let mine: &[PartSig] = sigs.0.get(&current).map_or(&[], |v| &v[..]);
    // The duplicate test is cached per entity until the current world or its scene changes.
    let key = (current, mine.len() + sigs.0.len());
    if (dups.0, dups.1) != key {
        *dups = (key.0, key.1, std::collections::HashMap::new());
    }
    // NEOPETS_TRACE_VIS: parts whose visibility changes, counted and printed every 300 frames
    // (diagnostic, for flicker).
    let trace = std::env::var_os("NEOPETS_TRACE_VIS").is_some();
    let no_zones = std::env::var_os("NEOPETS_NO_ZONES").is_some();
    for (e, g, s, cull, mut v) in &mut parts {
        // NEOPETS_NO_ZONES: every zone-gated part drawn (diagnostic).
        // The neighbouring world's parts skip the zone gate unless its Original setting is on.
        let zone_ok = g.is_none_or(|g| no_zones || (g.0 != current && !settings.original_neighbour) || state.play.zone_visible(g.0, g.1));
        // A repeat of one current part, or a part whose box the current parts' boxes cover by
        // 90 percent together (the sum of the covered parts, overlaps counted twice): 696's town
        // copy straddles several of 1023's parts.
        let dup = s.is_some_and(|s| {
            s.pack != current && *dups.2.entry(e).or_insert_with(|| mine.iter().any(|m| m.same(s)) || (dup_level() >= 4 && mine.iter().map(|m| m.contains(s)).sum::<f32>() >= 0.9))
        });
        // Skipped at D1 or more (squared values compared).
        let near = cull.is_none_or(|c| eye.is_none_or(|e| e.distance_squared(c.0) < c.1 * c.1));
        let want = if zone_ok && !dup && near { Visibility::Inherited } else { Visibility::Hidden };
        if *v != want {
            if trace {
                let t = (*toggles).0.entry(e).or_insert((0, String::new()));
                t.0 += 1;
                t.1 = format!("zone {:?} sig {:?} zone_ok {zone_ok} dup {dup} near {near}", g.map(|g| (g.0, g.1)), s.map(|s| (s.pack, s.count, s.lo, s.hi)));
            }
            *v = want;
        }
    }
    if trace {
        (*toggles).1 += 1;
        if (*toggles).1 % 300 == 0 {
            let mut list: Vec<_> = (*toggles).0.iter().filter(|(_, t)| t.0 > 2).map(|(e, t)| (t.0, *e, t.1.clone())).collect();
            list.sort_by(|a, b| b.0.cmp(&a.0));
            println!("vis frame {}: {} parts toggled more than twice", (*toggles).1, list.len());
            for (n, e, d) in list.iter().take(10) {
                println!("  {e:?} toggled {n}: {d}");
            }
        }
    }
}

/// A live enemy projectile's marker, by pool slot.
#[derive(Component)]
struct ShotMarker(usize);

/// The Rope Gun's strand from the muzzle to the rope's endpoint (SPEC-0033 State 27). The disc
/// draws repeated segments of resource 7's model key 159, whose draw size and joins the spec
/// leaves open; this is this build's stand-in: one thin gold cylinder.
#[derive(Component)]
struct RopeStrand;

/// The aim-view marker (SPEC-0033 section 23; R-0280 resolved): registry model 312, the role 29
/// child of resource 9's model container, a flat eight-pointed star outline with a white texture,
/// drawn unlit with ordinary alpha blend and depth test, tinted by the object's colour vector as
/// the draw colour scale (dark red (0.33, 0, 0, 1) while aiming). It stands 10 units ahead of the
/// camera, faces it, turns by the spin angle about the forward row and is scaled by M. When the
/// model cannot be read, this build's own star outline stands in. Hidden with the developer
/// menu's "Aim marker" switch.
#[derive(Component)]
struct AimMarker;

/// A flat star outline in the xy plane: `points` points at radius `outer`, notches at `inner`,
/// drawn as a band `width` wide centred on the outline (this build's own geometry).
fn aim_star_mesh(points: usize, outer: f32, inner: f32, width: f32) -> Mesh {
    let n = points * 2;
    let corner = |k: usize| {
        let a = k as f32 * std::f32::consts::TAU / n as f32 + std::f32::consts::FRAC_PI_2;
        let r = if k % 2 == 0 { outer } else { inner };
        Vec2::new(r * a.cos(), r * a.sin())
    };
    let mut positions = Vec::with_capacity(n * 2);
    for k in 0..n {
        let (p, a, b) = (corner(k), corner((k + n - 1) % n), corner((k + 1) % n));
        // The mitred offset of the corner, half the band each side, its length capped.
        let n1 = (p - a).perp().normalize();
        let n2 = (b - p).perp().normalize();
        let miter = (n1 + n2).normalize();
        let len = (0.5 * width / miter.dot(n1)).min(2.0 * width);
        for q in [p + miter * len, p - miter * len] {
            positions.push([q.x, q.y, 0.0]);
        }
    }
    let mut indices = Vec::with_capacity(n * 6);
    for k in 0..n {
        let (a, b) = (2 * k as u32, 2 * ((k + 1) % n) as u32);
        indices.extend_from_slice(&[a, a + 1, b, b, a + 1, b + 1]);
    }
    let normals = vec![[0.0, 0.0, 1.0]; positions.len()];
    Mesh::new(bevy::mesh::PrimitiveTopology::TriangleList, bevy::asset::RenderAssetUsages::default())
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
        .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
        .with_inserted_indices(bevy::mesh::Indices::U32(indices))
}

fn sync_aim_marker(
    mut commands: Commands,
    state: Option<Res<PlayState>>,
    dev: Res<dev_menu::DevMenu>,
    mut root: Query<(&mut Transform, &mut Visibility), With<AimMarker>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
    archive: Option<ResMut<Archive>>,
    mut material: Local<Option<(Handle<StandardMaterial>, bool)>>,
) {
    let Some(state) = state else { return };
    let Some((handle, disc_model)) = material.clone() else {
        // Model 312: role 29 of resource 9's container (R-0280).
        let model = archive.and_then(|mut a| {
            let a = &mut *a;
            let b = a.index.read(&mut a.disc, neopets_data::effect::RESOURCE).ok()?;
            let c = neopets_data::resource::walk_chunks(&b, None).ok()?.chunks;
            neopets_data::model::build_roles(&b, &c, None, Some(&[312 - 283])).ok().filter(|m| !m.parts.is_empty())
        });
        let base = StandardMaterial { base_color: Color::srgba(0.33, 0.0, 0.0, 1.0), unlit: true, alpha_mode: AlphaMode::Blend, cull_mode: None, ..default() };
        let mut parts: Vec<(Handle<Mesh>, Handle<StandardMaterial>)> = Vec::new();
        let from_disc = model.is_some();
        if let Some(m) = model {
            info!("aim marker: model 312 from resource 9 ({} parts)", m.parts.len());
            let mat = materials.add(base.clone());
            for (_, mesh, own) in part_assets(&m, &mut meshes, &mut materials, &mut images) {
                // Keep the part's texture (opaque white), take the marker's tint and blend.
                let tex = materials.get(&own).and_then(|o| o.base_color_texture.clone());
                if let Some(mut mm) = materials.get_mut(&mat) {
                    mm.base_color_texture = mm.base_color_texture.clone().or(tex);
                }
                parts.push((mesh, mat.clone()));
            }
        } else {
            warn!("aim marker: model 312 not read; drawing the stand-in star");
            parts.push((meshes.add(aim_star_mesh(8, 1.0, 0.78, 0.07)), materials.add(base)));
        }
        *material = Some((parts[0].1.clone(), from_disc));
        commands
            .spawn((AimMarker, Transform::default(), Visibility::Hidden))
            .with_children(|c| {
                for (mesh, m) in parts {
                    c.spawn((Mesh3d(mesh), MeshMaterial3d(m), Transform::default(), bevy::light::NotShadowCaster));
                }
            });
        return;
    };
    let Ok((mut t, mut vis)) = root.single_mut() else { return };
    match state.play.aim_marker().filter(|_| !dev.debug.no_aim_marker) {
        Some((at, forward, spin, scale, rgba)) => {
            let f = Vec3::from(forward).normalize_or_zero();
            let look = Transform::from_translation(Vec3::from(at)).looking_to(f, Vec3::Y);
            set_if(&mut t, look.with_rotation(look.rotation * Quat::from_rotation_z(spin)).with_scale(Vec3::splat(scale)));
            set_if(&mut vis, Visibility::Visible);
            if let Some(mut m) = materials.get_mut(&handle) {
                // The colour vector is the draw colour scale (R-0280); the stand-in doubles it.
                let k = if disc_model { 1.0 } else { 2.0 };
                let c = Color::srgba((k * rgba[0]).min(1.0), (k * rgba[1]).min(1.0), (k * rgba[2]).min(1.0), rgba[3].clamp(0.0, 1.0));
                if m.base_color != c {
                    m.base_color = c;
                }
            }
        }
        None => set_if(&mut vis, Visibility::Hidden),
    }
}

/// A pool shot's trail model part (SPEC-0029 Visual update): pool slot, owner and table entry.
#[derive(Component)]
struct ShotTrail(usize, u64, i32);

/// Enemy projectiles of visual kind below 5 draw their trail model part, the owner's extra
/// model part of the shot's table entry, at the shot position with the shot's frame (its
/// forward axis and the world up vector); a shot with flight effects is drawn by them. A shot
/// of visual kind below 5 with neither is drawn as a small plain marker, a diagnostic stand-in
/// (on disc it draws nothing).
#[allow(clippy::too_many_arguments)]
fn sync_shots(
    mut commands: Commands,
    state: Option<Res<PlayState>>,
    mut markers: Query<(&ShotMarker, &mut Transform, &mut Visibility), (Without<RopeStrand>, Without<ShotTrail>)>,
    mut strand: Query<(&mut Transform, &mut Visibility), (With<RopeStrand>, Without<ShotMarker>, Without<ShotTrail>)>,
    mut trails: Query<(Entity, &ShotTrail, &mut Transform, &mut Visibility), (Without<RopeStrand>, Without<ShotMarker>)>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
    mut made: Local<bool>,
) {
    let _span = crate::perf::span("sync_shots");
    let Some(state) = state else { return };
    if !*made {
        *made = true;
        let mesh = meshes.add(Cuboid::new(0.4, 0.4, 0.4));
        let material = materials.add(StandardMaterial { base_color: Color::srgb(1.0, 0.85, 0.2), unlit: true, ..default() });
        // The pool's slots, then 8 for the Undead Draik's spit records.
        for i in 0..neopets_sim::projectile::SLOTS + 8 {
            commands.spawn((ShotMarker(i), Mesh3d(mesh.clone()), MeshMaterial3d(material.clone()), Transform::default(), Visibility::Hidden, bevy::light::NotShadowCaster));
        }
        let rope = meshes.add(Cylinder::new(0.06, 1.0));
        let gold = materials.add(StandardMaterial { base_color: Color::srgb(0.85, 0.68, 0.36), perceptual_roughness: 0.8, ..default() });
        commands.spawn((RopeStrand, Mesh3d(rope), MeshMaterial3d(gold), Transform::default(), Visibility::Hidden, bevy::light::NotShadowCaster));
        return;
    }
    for (mut t, mut vis) in &mut strand {
        match state.play.rope_line() {
            Some((a, b)) => {
                let (a, b) = (Vec3::from(a), Vec3::from(b));
                let d = b - a;
                let l = d.length();
                if l < 1e-3 {
                    set_if(&mut vis, Visibility::Hidden);
                    continue;
                }
                set_if(&mut t, Transform::from_translation((a + b) * 0.5).with_rotation(Quat::from_rotation_arc(Vec3::Y, d / l)).with_scale(Vec3::new(1.0, l, 1.0)));
                set_if(&mut vis, Visibility::Visible);
            }
            None => *vis = Visibility::Hidden,
        }
    }
    // The trail parts: (slot, owner, entry, position, velocity) of each live shot that has one.
    let p = &state.play;
    let part = |owner: u64, entry: i32| p.characters.iter().find(|c| c.id == owner).and_then(|c| c.model.attachments.iter().find(|(k, _)| *k as i32 == entry)).map(|(_, m)| m);
    let wanted: Vec<(usize, u64, i32, [f32; 3], [f32; 3])> = p
        .shots
        .slots
        .iter()
        .enumerate()
        .filter_map(|(i, s)| {
            let s = s.as_ref()?;
            let owner = s.s.owner?;
            (s.s.visual < 5 && s.s.trail >= 0 && part(owner, s.s.trail).is_some()).then_some((i, owner, s.s.trail, s.position, s.velocity))
        })
        .collect();
    let frame = |at: [f32; 3], v: [f32; 3]| {
        let fwd = Vec3::from(v).normalize_or_zero();
        let t = Transform::from_translation(Vec3::from(at));
        if fwd.length_squared() == 0.0 { t } else { t.looking_to(-fwd, Vec3::Y) }
    };
    for (e, tr, mut t, mut vis) in &mut trails {
        match wanted.iter().find(|w| (w.0, w.1, w.2) == (tr.0, tr.1, tr.2)) {
            Some(&(_, _, _, at, v)) => {
                set_if(&mut t, frame(at, v));
                set_if(&mut vis, Visibility::Visible);
            }
            None => commands.entity(e).despawn(),
        }
    }
    for &(i, owner, entry, at, v) in wanted.iter().filter(|w| !trails.iter().any(|(_, tr, ..)| (tr.0, tr.1, tr.2) == (w.0, w.1, w.2))) {
        let Some(m) = part(owner, entry) else { continue };
        let parts = part_assets(m, &mut meshes, &mut materials, &mut images);
        commands.spawn((ShotTrail(i, owner, entry), frame(at, v), Visibility::Visible)).with_children(|c| {
            for (_, mesh, material) in parts {
                c.spawn((Mesh3d(mesh), MeshMaterial3d(material), Transform::IDENTITY, bevy::light::NotShadowCaster));
            }
        });
    }
    for (m, mut t, mut vis) in &mut markers {
        // A shot drawn by its trail part has no marker.
        if wanted.iter().any(|w| w.0 == m.0) {
            set_if(&mut vis, Visibility::Hidden);
            continue;
        }
        // A shot drawn by its flight effects, and the spits (drawn by their effects), have no
        // marker. SPEC-0029 Visual update: a shot draws only its flight effects and, for visual
        // kinds below 5, a trail model part, so one of visual kind 5 or more with both flight
        // ids -1 draws nothing (the Spore Monster's lobbed seed, kind 8, visual 23). The marker
        // stays for a visual kind below 5 with no flight effect (its trail part is not drawn).
        let at = match m.0.checked_sub(neopets_sim::projectile::SLOTS) {
            Some(_) => None,
            None if state.play.pool_slot_has_fx(m.0) => None,
            // SPEC-0029 Visual update: a null part pointer draws nothing, so a shot with no effect and no part is unseen (the
            // Plant Gunner's: user's report of yellow dots; R-0240). NEOPETS_SHOT_MARKERS shows them (diagnostic).
            None if std::env::var_os("NEOPETS_SHOT_MARKERS").is_none() => None,
            None => state.play.shots.slots.get(m.0).copied().flatten().filter(|s| s.s.visual < 5).map(|s| s.position),
        };
        match at {
            Some(p) => {
                t.translation = Vec3::from(p);
                set_if(&mut vis, Visibility::Visible);
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
        // SPEC-0021 Item model registry: on disc each of resources 544 to 564 registers under its own item id.
        98..=105 | 111..=118 | 106..=110 => {
            let resource = match id {
                98..=105 => 544 + id - 98,
                111..=118 => 552 + id - 111,
                _ => 560 + id - 106,
            };
            match archive_view::model(disc, index, resource)? {
                Some(m) => m,
                None => return Ok(None),
            }
        }
        98..=190 => {
            if boot.is_none() {
                let b = index.read(disc, 7)?;
                let family = neopets_data::resource::classify(&b);
                let c = neopets_data::resource::walk_chunks(&b, neopets_data::resource::chunk_limit(7, family))?.chunks;
                *boot = Some((b, c));
            }
            let (b, c) = boot.as_ref().unwrap();
            let m = neopets_data::model::build_roles(b, c, None, Some(&[(id - 98) as u16]))?;
            if m.parts.is_empty() {
                // Not in resource 7: a world pack's type-8 child under this id (SPEC-0021 Item model registry).
                let bytes = index.read(disc, pack)?;
                let chain = neopets_data::resource::walk_chunks(&bytes, None)?;
                neopets_data::model::build_roles(&bytes, &chain.chunks, None, Some(&[id as u16]))?
            } else {
                m
            }
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
    mut markers: Query<(Entity, &ItemMarker, &mut Transform, &mut Visibility, Option<&Children>), Without<ShowcaseMarker>>,
    mut showcases: Query<(Entity, &ShowcaseMarker, &mut Transform, &mut Visibility, Option<&ChildOf>), Without<ItemMarker>>,
    cameras: Query<(Entity, &Camera, &GlobalTransform), (With<Camera3d>, Without<crate::water_reflect::MirrorCam>)>,
    windows: Query<&Window>,
    parts: Query<&MeshMaterial3d<StandardMaterial>, With<ItemPart>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
    mut assets: Local<Option<(Handle<Mesh>, [Handle<StandardMaterial>; 3])>>,
    mut models: Local<std::collections::HashMap<(u32, u32), Option<Vec<(Handle<Mesh>, StandardMaterial, Option<neopets_data::material::Flipbook>)>>>>,
    mut boot: Local<Option<(Vec<u8>, Vec<neopets_data::resource::Chunk>)>>,
    time: Res<Time>,
    mut coin_time: Local<f32>,
) {
    let _span = crate::perf::span("sync_items");
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
                // SPEC-0021 Spawning step 11 and Per-tick step 5: the axis rows, a non-coin's mirrored ones included, times the scale.
                let [ax, ay, az] = it.axes().map(Vec3::from);
                set_if(&mut t, Transform::from_matrix(Mat4::from_cols(ax.extend(0.0), ay.extend(0.0), az.extend(0.0), Vec3::from(it.position).extend(1.0))).with_scale(Vec3::splat(it.drawn_scale().max(0.001)) * Transform::from_matrix(Mat4::from_cols(ax.extend(0.0), ay.extend(0.0), az.extend(0.0), Vec4::W)).scale));
                set_if(&mut vis, Visibility::Visible);
                seen.insert((m.0, m.1));
                let alpha = it.life.clamp(0.0, 1.0);
                for c in children.into_iter().flatten() {
                    if let Ok(h) = parts.get(*c) {
                        if materials.get(&h.0).is_some_and(|mat| (mat.base_color.alpha() - alpha).abs() > 1e-3) {
                            if let Some(mut mat) = materials.get_mut(&h.0) {
                                mat.base_color.set_alpha(alpha);
                                mat.alpha_mode = AlphaMode::Blend;
                            }
                        }
                    }
                }
            }
            Some(_) => {
                set_if(&mut vis, Visibility::Hidden);
                seen.insert((m.0, m.1));
            }
            None => commands.entity(e).despawn(),
        }
    }
    let mut archive = archive;
    // SPEC-0021 Item showcase, mode 4: the item model floating above the hero. Modes 1 to 3
    // (Medium): anchored to the screen, at E + z F + 640 (x - 0.5)(z / 512) R + 480 (y - 0.5)
    // (z / 512) U (the camera's forward, right and up; the PS2 rows point left and down, so the
    // spec's minus signs become plus here), turned by the angle, then the camera's orientation,
    // scaled by s. A screen-anchored model is a child of the camera, placed in the camera's axes: taken from the camera's
    // GlobalTransform it lagged a frame behind, and shook whenever the camera moved (user's report).
    let camera = cameras.iter().next().map(|(e, ..)| e);
    let show: Option<(u16, u32, Transform, bool)> = p.showcase_item().map(|(id, pack, at, angle)| (id, pack, Transform::from_translation(Vec3::from(at)).with_rotation(Quat::from_rotation_y(-angle)), false)).or_else(|| {
        let c = p.corner?;
        let k = c.z / 512.0;
        let origin = Vec3::new(640.0 * (c.x - 0.5) * k, 480.0 * (c.y - 0.5) * k, -c.z);
        Some((c.id, c.pack, Transform { translation: origin, rotation: Quat::from_rotation_y(std::f32::consts::PI - c.angle), scale: Vec3::splat(c.s.max(1e-4)) }, true))
    });
    // SPEC-0035 section 15.2 (R-0155): while price caption line 1 shows (its counter above 0, both
    // quick panels closed) the one-Neopoint coin model (item 135) is drawn centred at canvas
    // (332.8, 107.5), 6.0 ahead of the camera, scale 0.5886, depth tested, turning at 1 rad/s about
    // the camera's up axis; the angle moves only on drawn frames and is never reset. In a window
    // wider than the canvas it stays with the caption at the window's top-right edge (hud_view;
    // this build's choice, the canvas being the PS2's whole screen).
    let caption_coin: Option<Transform> = (p.price_lines[0].1 > 0 && !p.quick.open && !p.mote_panel.open && !p.menu_open)
        .then(|| {
            let (_, cam, g) = cameras.iter().next()?;
            let w = windows.iter().next()?;
            let (ww, wh) = (w.width(), w.height());
            let sc = (ww / 512.0).min(wh / 448.0);
            let ox = (ww - 512.0 * sc) / 2.0;
            // As fractions of the window, then of the camera's own viewport (which can be a
            // render target of another size).
            let at = Vec2::new((2.0 * ox + 332.8 * sc) / ww, 107.5 * sc / wh) * cam.logical_viewport_size()?;
            let ray = cam.viewport_to_world(g, at).ok()?;
            let depth = 6.0 / ray.direction.dot(g.forward().into()).max(1e-3);
            *coin_time = (*coin_time - time.delta_secs()) % std::f32::consts::TAU;
            Some(Transform { translation: g.affine().inverse().transform_point3(ray.get_point(depth)), rotation: Quat::from_rotation_y(*coin_time), scale: Vec3::splat(0.5886) })
        })
        .flatten();
    // (marker key, item model id, pack, transform); the caption coin's key is not an item id.
    let wanted: Vec<(u16, u16, u32, Transform, bool)> = show.map(|(id, pack, t, anchored)| (id, id, pack, t, anchored)).into_iter().chain(caption_coin.map(|t| (CAPTION_COIN, 135, 0, t, true))).collect();
    for (e, s, mut t, mut vis, parent) in &mut showcases {
        match wanted.iter().find(|w| w.0 == s.0) {
            Some(&(_, _, _, tr, anchored)) => {
                set_if(&mut t, tr);
                set_if(&mut vis, Visibility::Visible);
                match (anchored, parent, camera) {
                    (true, None, Some(c)) => {
                        commands.entity(e).insert(ChildOf(c));
                    }
                    (false, Some(_), _) => {
                        commands.entity(e).remove::<ChildOf>();
                    }
                    _ => {}
                }
            }
            None => commands.entity(e).despawn(),
        }
    }
    for &(marker, id, pack, at, anchored) in wanted.iter().filter(|w| !showcases.iter().any(|(_, s, ..)| s.0 == w.0)) {
        let id = id as u32;
        let key = (id, if matches!(id, 50..=59 | 98..=190) { 0 } else { pack });
        if !models.contains_key(&key) {
            let built = match archive.as_deref_mut() {
                Some(Archive { disc, index }) => item_model(disc, index, &mut boot, id, pack).unwrap_or(None),
                None => None,
            };
            let list = built.map(|m| {
                part_assets(&m, &mut meshes, &mut materials, &mut images)
                    .into_iter()
                    .map(|(k, mesh, mat)| (mesh, materials.get(&mat).cloned().unwrap_or_default(), m.parts[k].flipbook))
                    .collect::<Vec<_>>()
            });
            models.insert(key, list);
        }
        if let Some(list) = &models[&key] {
            let mut root = commands.spawn((ShowcaseMarker(marker), at, Visibility::Visible));
            if let (true, Some(c)) = (anchored, camera) {
                root.insert(ChildOf(c));
            }
            root.with_children(|c| {
                for (mesh, mat, flip) in list {
                    let mut e = c.spawn((Mesh3d(mesh.clone()), MeshMaterial3d(materials.add(mat.clone())), Transform::IDENTITY));
                    if let Some(f) = flip {
                        e.insert(UvFlip(*f));
                    }
                }
            });
        }
    }
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
                        .map(|(k, mesh, mat)| (mesh, materials.get(&mat).cloned().unwrap_or_default(), m.parts[k].flipbook))
                        .collect::<Vec<_>>()
                });
                models.insert(key, list);
            }
            let root = (ItemMarker(pack, i), Transform::from_translation(Vec3::from(it.position)), Visibility::Hidden);
            match &models[&key] {
                Some(list) => {
                    commands.spawn(root).with_children(|c| {
                        for (mesh, mat, flip) in list {
                            let mut e = c.spawn((ItemPart, Mesh3d(mesh.clone()), MeshMaterial3d(materials.add(mat.clone())), Transform::IDENTITY));
                            if let Some(f) = flip {
                                e.insert(UvFlip(*f));
                            }
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

/// The showcase marker key of the coin beside the price caption.
const CAPTION_COIN: u16 = u16::MAX;

/// The floating item of the full showcase (SPEC-0021 Item showcase), by item id, or the coin
/// beside the price caption (`CAPTION_COIN`).
#[derive(Component)]
struct ShowcaseMarker(u16);

/// A focus-hero item's drawn root, by its resource.
#[derive(Component)]
pub(crate) struct HeroItemRoot(u32);

/// The focus hero's held items' drawn matrices this frame, by display resource (the saved
/// weapon and shield transforms that the Mote glow takes; SPEC-0046 Hero-item effect matrices).
#[derive(Resource, Default)]
struct HeroItemFrames(std::collections::HashMap<u32, Mat4>);

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
    mut pass1_materials: ResMut<Assets<pass1::Pass1Material>>,
    mut images: ResMut<Assets<Image>>,
    mut models: Local<std::collections::HashMap<u32, Option<([f32; 3], [f32; 3])>>>,
    mut frames: ResMut<HeroItemFrames>,
    mut lit: Local<std::collections::HashMap<u32, (neopets_data::model::ModelData, Vec<(Handle<Mesh>, usize)>)>>,
    mut spawned: Local<std::collections::HashSet<u32>>,
) {
    let _span = crate::perf::span("sync_hero_items");
    frames.0.clear();
    let Some(state) = state else { return };
    // Player 1's held items by their display resource, then each other player's (local co-op,
    // coop.rs) under its own key (the resource with the player's number in the top byte), so
    // that the same sword in two hands is two models.
    let mut items: Vec<(u32, u32, neopets_data::anim::Affine, play::HeroItemScale)> = state.play.hero_items().into_iter().map(|(r, f, s)| (r, r, f, s)).collect();
    for (k, o) in state.play.others.iter().enumerate() {
        items.extend(o.items_drawn.iter().map(|&(r, f, s)| (r | ((k as u32 + 1) << 24), r, f, s)));
    }
    let mut archive = archive;
    let mut shown = std::collections::HashSet::new();
    // SPEC-0046 Light slots and the lit draw: the focus hero's held items are lit from his own
    // light state, following their rotation.
    for (key, res, frame, _) in &items {
        if let (true, Some((m, handles))) = (key == res, lit.get(res)) {
            relight_model(m, handles, frame, &state.play.lights(None, frame.t), &mut meshes);
        }
    }
    for (key, res, frame, scale) in &items {
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
                if std::env::var_os("NEOPETS_TRACE_PASS1").is_some() {
                    println!("hero item model {res}: bounds {:?} to {:?}, {} parts, {} two-pass, {} env pass 0, {} flipbook", m.min, m.max, m.parts.len(), m.parts.iter().filter(|p| p.pass1.is_some()).count(), m.parts.iter().filter(|p| p.env_map).count(), m.parts.iter().filter(|p| p.flipbook.is_some()).count());
                }
                lit.insert(*res, (m, Vec::new()));
            }
            models.insert(*res, bounds);
        }
        if !spawned.contains(key) {
            if let Some((m, handles)) = lit.get_mut(res) {
                spawned.insert(*key);
                let parts = part_assets(m, &mut meshes, &mut materials, &mut images);
                // The held model's render pass 1 (the blade and metal shine; pass1.rs).
                let second = pass1_assets(m, &mut meshes, &mut pass1_materials, &mut images);
                if key == res {
                    *handles = parts.iter().filter(|(k, ..)| m.parts[*k].lit).map(|(k, mesh, _)| (mesh.clone(), *k)).collect();
                }
                commands.spawn((HeroItemRoot(*key), Transform::IDENTITY, Visibility::Hidden)).with_children(|c| {
                    for (_, mesh, material) in parts {
                        c.spawn((Mesh3d(mesh), MeshMaterial3d(material), Transform::IDENTITY));
                    }
                    for (_, mesh, material, scroll) in second {
                        let mut e = c.spawn((Mesh3d(mesh), MeshMaterial3d(material), Transform::IDENTITY));
                        if let Some(s) = scroll {
                            e.insert(pass1::Pass1Scroll(s));
                        }
                    }
                });
            }
        }
        let Some(Some((min, max))) = models.get(res).copied() else { continue };
        let f = match scale {
            play::HeroItemScale::None => 1.0,
            play::HeroItemScale::Shield => 1.6 / (min[0] - (max[0] - min[0])).abs().max(1e-6),
            play::HeroItemScale::Weapon => (2.6 / (min[2] - (max[2] - min[2])).abs().max(1e-6)).min(0.6),
        };
        let c = |k: usize| Vec4::new(frame.cols[k][0] * f, frame.cols[k][1] * f, frame.cols[k][2] * f, 0.0);
        let m = Mat4::from_cols(c(0), c(1), c(2), Vec4::new(frame.t[0], frame.t[1], frame.t[2], 1.0));
        if key == res {
            frames.0.insert(*res, m);
        }
        shown.insert(*key);
        for (_, root, mut t, mut vis) in &mut roots {
            if root.0 == *key {
                set_if(&mut t, Transform::from_matrix(m));
                set_if(&mut vis, Visibility::Visible);
            }
        }
    }
    for (_, root, _, mut vis) in &mut roots {
        if !shown.contains(&root.0) {
            set_if(&mut vis, Visibility::Hidden);
        }
    }
}

/// A Mote glow model's drawn root, by registry model id, with its parts' materials.
#[derive(Component)]
struct GlowRoot(u32, Vec<Handle<StandardMaterial>>);

/// SPEC-0046 Hero-item effect matrices and SPEC-0003 CPU vertex animator 3: Tor's Mote glow
/// models (resource 9 children 16 to 28, ids 299 to 311) at the held item's drawn matrix,
/// unlit and additive, coloured with the Mote's RGB (vertex colour 128 = 1.0). Each drawn frame
/// the animator's U phase advances by 0.06 and its alpha phase p by 0.2, with alpha
/// a = 64 + 186 (1 + sin p) / 2 (both phases are advanced once per frame here, not per mesh).
#[allow(clippy::too_many_arguments)]
fn sync_hero_glows(
    mut commands: Commands,
    state: Option<Res<PlayState>>,
    archive: Option<ResMut<Archive>>,
    mut roots: Query<(&GlowRoot, &mut Transform, &mut Visibility)>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
    mut models: Local<std::collections::HashMap<u32, bool>>,
    mut phase: Local<(f32, f32)>,
    frames: Res<HeroItemFrames>,
) {
    let _span = crate::perf::span("sync_hero_glows");
    let Some(state) = state else { return };
    // NEOPETS_NO_GLOW: the glow off (diagnostic).
    let glows = if std::env::var_os("NEOPETS_NO_GLOW").is_some() { Vec::new() } else { state.play.hero_glows() };
    let mut archive = archive;
    if !glows.is_empty() {
        phase.0 = (phase.0 + 0.06).rem_euclid(1.0);
        phase.1 += 0.2;
    }
    let alpha = (64.0 + 186.0 * (1.0 + phase.1.sin()) / 2.0 + 0.5).floor();
    let mut shown = std::collections::HashSet::new();
    for (id, item, rgb) in &glows {
        if !models.contains_key(id) {
            let built = archive.as_deref_mut().and_then(|Archive { disc, index }| {
                let b = index.read(disc, neopets_data::effect::RESOURCE).ok()?;
                let c = neopets_data::resource::walk_chunks(&b, None).ok()?.chunks;
                let m = neopets_data::model::build_roles(&b, &c, None, Some(&[(*id - 283) as u16]));
                if std::env::var_os("NEOPETS_TRACE_GLOW").is_some() {
                    match &m {
                        Ok(m) => println!("glow model {id}: {} parts, {} trees, skipped {:?}, bounds {:?} to {:?}, first {:?}", m.parts.len(), m.trees, m.skipped_leaves, m.min, m.max, m.parts.first().map(|p| p.positions.iter().take(4).collect::<Vec<_>>())),
                        Err(e) => println!("glow model {id}: {e}"),
                    }
                }
                m.ok()
            });
            let built = built.filter(|m| !m.parts.is_empty());
            let ok = built.is_some();
            if let Some(m) = built {
                let parts = part_assets(&m, &mut meshes, &mut materials, &mut images);
                let mut handles = Vec::new();
                let mut children = Vec::new();
                for (_, mesh, material) in parts {
                    let mut own = materials.get(&material).cloned().unwrap_or_default();
                    own.unlit = true;
                    own.alpha_mode = AlphaMode::Add;
                    own.cull_mode = None;
                    let h = materials.add(own);
                    handles.push(h.clone());
                    children.push((mesh, h));
                }
                commands.spawn((GlowRoot(*id, handles), Transform::IDENTITY, Visibility::Hidden)).with_children(|c| {
                    for (mesh, h) in children {
                        c.spawn((Mesh3d(mesh), MeshMaterial3d(h), Transform::IDENTITY, bevy::light::NotShadowCaster));
                    }
                });
            }
            models.insert(*id, ok);
        }
        // The glow takes the held item's drawn matrix (its frame with the stowed rescale).
        let (Some(true), Some(&m)) = (models.get(id).copied(), frames.0.get(item)) else { continue };
        shown.insert(*id);
        let k = alpha / 128.0;
        let colour = Color::linear_rgb(rgb[0] as f32 / 128.0 * k, rgb[1] as f32 / 128.0 * k, rgb[2] as f32 / 128.0 * k);
        for (root, mut t, mut vis) in &mut roots {
            if root.0 == *id {
                set_if(&mut t, Transform::from_matrix(m));
                set_if(&mut vis, Visibility::Visible);
                let uv = bevy::math::Affine2::from_translation(Vec2::new(phase.0, 0.0));
                for h in &root.1 {
                    if materials.get(h).is_some_and(|mat| mat.base_color != colour || mat.uv_transform != uv) {
                        if let Some(mut mat) = materials.get_mut(h) {
                            mat.base_color = colour;
                            mat.uv_transform = uv;
                        }
                    }
                }
            }
        }
    }
    for (root, _, mut vis) in &mut roots {
        if !shown.contains(&root.0) {
            set_if(&mut vis, Visibility::Hidden);
        }
    }
}

/// The atmosphere overlay (SPEC-0046 step 10, SPEC-0028 section 11 vector B).
#[derive(Component)]
struct AtmosOverlay;

/// SPEC-0046 step 10: when vector B's alpha byte is nonzero, its RGB is drawn over the frame.
/// The spec's blend is (1 - Ad) x Cs + Ad x Cd with the destination alpha Ad, which this build
/// has no frame alpha for; this build's choice (R-0260) weights the colour by the alpha byte /
/// 128 (the gated zones carry 28, Meridell's ungated one 10), under the HUD (user's report 71:
/// the purple haze of the gated zones was missing).
fn sync_atmos_overlay(mut commands: Commands, state: Option<Res<PlayState>>, mut overlays: Query<(&mut BackgroundColor, &mut Visibility), With<AtmosOverlay>>) {
    let out = state.as_ref().map(|s| s.play.atmos_out);
    // Drawn whenever vector B's alpha byte is above 0 (SPEC-0046 step 10, draw 2). A gated zone
    // with its blend factor above 0 keeps the user-tuned weight below; an ungated zone (blend
    // 0, such as the 1335 palace's (160, 84, 171) at 16: user report 108, the purple was
    // missing) takes the spec's weight alpha / 128. It had been left out by the user's choice
    // for Meridell's warm (255, 214, 128) at 10, when its weight was far stronger.
    // Meridell's warm ungated zone (255, 214, 128) stays undrawn by the user's choice: they found
    // it wrong as an 8 percent wash and again at the spec's weight (user report 134, "really bad
    // golden haze"). Other ungated zones (the 1335 palace's purple, report 108) are drawn.
    let warm = out.is_some_and(|o| o.blend <= 0.0 && o.b[0].round() == 255.0 && o.b[1].round() == 214.0 && o.b[2].round() == 128.0);
    let byte = if warm { 0.0 } else { out.map_or(0.0, |o| o.b[3].round().clamp(0.0, 255.0)) };
    // The weight is 1 - Ad with Ad the frame alpha an opaque draw leaves, 0x80, normalized over
    // 255: about 0.5 (the source alpha byte is only the gate). NEOPETS_ATMOS_WEIGHT overrides it
    // (diagnostic). The alpha-byte / 128 weight (0.22) left the land with no haze (user's report 72).
    let w = std::env::var("NEOPETS_ATMOS_WEIGHT").ok().and_then(|v| v.trim().parse::<f32>().ok()).unwrap_or(1.0 - 128.0 / 255.0);
    let gate = out.map_or(0.0, |o| o.blend.min(1.0));
    // The GS blends in gamma space; the UI blends in linear light, where the same weight darkens
    // far less (a bright roof lost a third, not half: user's report 73). Over a dark overlay
    // colour, out = (1 - w) x Cd in gamma is (1 - w)^2.2 x Cd in linear, so the linear alpha is
    // 1 - (1 - w)^2.2 (about 0.78 for w = 0.5).
    // The user found the full gamma-space weight (0.78) too strong and 0.5 too weak; this build
    // uses the exponent 1.4 (about 0.62), between the two (R-0260).
    // The ungated weight a = alpha / 128 is a gamma-space blend toward B. The UI blends in linear
    // light, where a brighter overlay lifts dark pixels far more (the 1344 cellar went solid
    // purple); 0.6 a matches the gamma-space result for mid-tones (encoded 50 to 100) over
    // (160, 84, 171): a = 0.125 gives about 0.075.
    let spec_w = out.map_or(0.0, |o| 0.6 * o.b[3].clamp(0.0, 128.0) / 128.0);
    // The gated haze is then scaled to 0.7 by the user's choice (user report 140: the night
    // swamp 722 went nearly black under it; "lighter everywhere"), about 0.43 at full gate.
    // NEOPETS_HAZE_SCALE overrides the scale (diagnostic).
    let scale = std::env::var("NEOPETS_HAZE_SCALE").ok().and_then(|v| v.trim().parse::<f32>().ok()).unwrap_or(0.7);
    let wl = if gate > 0.0 { scale * (1.0 - (1.0 - (w * gate).clamp(0.0, 1.0)).powf(1.4)) } else { spec_w };
    let colour = out.map(|o| Color::srgba(o.b[0].clamp(0.0, 255.0) / 255.0, o.b[1].clamp(0.0, 255.0) / 255.0, o.b[2].clamp(0.0, 255.0) / 255.0, wl));
    match overlays.iter_mut().next() {
        Some((mut bg, mut vis)) => {
            set_if(&mut vis, if byte > 0.0 && std::env::var_os("NEOPETS_NO_ATMOS").is_none() { Visibility::Visible } else { Visibility::Hidden });
            if let Some(c) = colour {
                if bg.0 != c {
                    bg.0 = c;
                }
            }
        }
        None if state.is_some() => {
            commands.spawn((
                Node { position_type: PositionType::Absolute, left: px(0), right: px(0), top: px(0), bottom: px(0), ..default() },
                BackgroundColor(colour.unwrap_or(Color::NONE)),
                Visibility::Hidden,
                GlobalZIndex(1),
                AtmosOverlay,
            ));
        }
        None => {}
    }
}

/// A part of the sky dome's role-10 layer and its own material (SPEC-0046 Sky domes).
#[derive(Component)]
struct HazeLayer(Handle<StandardMaterial>);

/// SPEC-0046 Sky domes, step 6: the role-10 layer shows while the atmosphere blend factor b is
/// above 0, with b as its master opacity.
fn sync_haze(state: Option<Res<PlayState>>, mut layers: Query<(&HazeLayer, &mut Visibility)>, mut materials: ResMut<Assets<StandardMaterial>>) {
    let b = state.map_or(0.0, |s| s.play.atmos_out.blend).clamp(0.0, 1.0);
    for (layer, mut vis) in &mut layers {
        set_if(&mut vis, if b > 0.0 { Visibility::Visible } else { Visibility::Hidden });
        if materials.get(&layer.0).is_some_and(|m| (m.base_color.alpha() - b).abs() > 1e-3) {
            if let Some(mut m) = materials.get_mut(&layer.0) {
                m.base_color.set_alpha(b);
            }
        }
    }
}

/// One pooled model particle's drawn root: (resource-9 model, pool slot) and its materials.
#[derive(Component)]
struct EffectModelRoot(u16, usize, Vec<Handle<StandardMaterial>>);

/// SPEC-0034 4.7: each live model particle draws its node's child model with the particle's draw
/// frame, unlit, with colour channel 1 as its colour. A root is made per model the first time
/// that many of it are live, then reused.
#[allow(clippy::too_many_arguments)]
fn sync_effect_models(
    mut commands: Commands,
    state: Option<Res<PlayState>>,
    archive: Option<ResMut<Archive>>,
    mut roots: Query<(&EffectModelRoot, &mut Transform, &mut Visibility)>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
    mut built: Local<std::collections::HashMap<u16, Option<neopets_data::model::ModelData>>>,
    mut made: Local<std::collections::HashSet<(u16, usize)>>,
) {
    let _span = crate::perf::span("sync_effect_models");
    let Some(state) = state else { return };
    let play = &state.play;
    let mut archive = archive;
    let mut counts: std::collections::HashMap<u16, usize> = std::collections::HashMap::new();
    let mut shown: std::collections::HashMap<(u16, usize), (Mat4, [f32; 4])> = std::collections::HashMap::new();
    for (model, at, rows, colour) in play.effects.models(&play.effect_table) {
        let slot = counts.entry(model).or_insert(0);
        let v = |r: [f32; 3], w: f32| Vec4::new(r[0], r[1], r[2], w);
        shown.insert((model, *slot), (Mat4::from_cols(v(rows[0], 0.0), v(rows[1], 0.0), v(rows[2], 0.0), v(at, 1.0)), colour));
        *slot += 1;
    }
    // NEOPETS_TRACE_FXMODELS: the live model particles, by model, with their colours, every 120 ticks (diagnostic).
    if std::env::var_os("NEOPETS_TRACE_FXMODELS").is_some() && play.ticks % 120 == 0 && !shown.is_empty() {
        let mut list: Vec<String> = shown.iter().map(|(k, v)| format!("{}:{} {:.2?}", k.0, k.1, v.1)).collect();
        list.sort();
        println!("fx models on tick {}: {} live: {}", play.ticks, list.len(), list.join("; "));
    }
    for &(model, slot) in shown.keys() {
        if made.contains(&(model, slot)) {
            continue;
        }
        let data = built.entry(model).or_insert_with(|| {
            archive.as_deref_mut().and_then(|Archive { disc, index }| {
                let b = index.read(disc, neopets_data::effect::RESOURCE).ok()?;
                let c = neopets_data::resource::walk_chunks(&b, None).ok()?.chunks;
                // SPEC-0034 2.1: a child-chunk model is the effect group's own type-12 chunk k,
                // not resource-9 model k (user's reports 66, 69 and 94: the save point's rays drew
                // container models 8 and 9, tall shafts the original does not show).
                if let Some(k) = model.checked_sub(neopets_sim::effect::CHILD_MODEL) {
                    let table = neopets_data::effect::table_in_chain(&b, &c)?;
                    let at = table.start + *play.effect_table.child_chunks.get(k as usize)?;
                    return neopets_data::model::build_child(&b, &c, at).ok().filter(|m| !m.parts.is_empty());
                }
                neopets_data::model::build_roles(&b, &c, None, Some(&[model])).ok().filter(|m| !m.parts.is_empty())
            })
        });
        made.insert((model, slot));
        if std::env::var_os("NEOPETS_TRACE_FX").is_some() {
            println!("effect model {model} slot {slot}: {:?} parts", data.as_ref().map(|m| m.parts.len()));
        }
        let Some(m) = data.as_ref() else { continue };
        let parts = part_assets(m, &mut meshes, &mut materials, &mut images);
        let mut handles = Vec::new();
        let mut children = Vec::new();
        for (i, mesh, material) in parts {
            let mut own = materials.get(&material).cloned().unwrap_or_default();
            own.unlit = true;
            own.cull_mode = None;
            if own.alpha_mode == AlphaMode::Opaque {
                own.alpha_mode = AlphaMode::Blend;
            }
            let h = materials.add(own);
            handles.push(h.clone());
            // SPEC-0003 Effect model particles: an effect model's scrolled or flipbook material
            // runs like any mesh's (the lightning columns, bolts and the flame sheet).
            let anim = (m.parts[i].scroll.filter(|_| m.parts[i].texture.is_some()), m.parts[i].flipbook);
            children.push((mesh, h, anim));
        }
        commands.spawn((EffectModelRoot(model, slot, handles), Transform::IDENTITY, Visibility::Hidden)).with_children(|c| {
            for (mesh, h, (scroll, flip)) in children {
                let mut e = c.spawn((Mesh3d(mesh), MeshMaterial3d(h), Transform::IDENTITY, bevy::light::NotShadowCaster));
                if let Some(s) = scroll {
                    e.insert(UvScroll(s));
                }
                if let Some(f) = flip {
                    e.insert(UvFlip(f));
                }
            }
        });
    }
    for (root, mut t, mut vis) in &mut roots {
        // NEOPETS_HIDE_FXMODEL="a,b": those effect models are never shown (diagnostic).
        let hidden = std::env::var("NEOPETS_HIDE_FXMODEL").is_ok_and(|v| v.split(',').any(|t| t.trim().parse::<usize>().ok() == Some(root.0 as usize)));
        let Some(&(m, c)) = shown.get(&(root.0, root.1)).filter(|_| !hidden) else {
            set_if(&mut vis, Visibility::Hidden);
            continue;
        };
        set_if(&mut t, Transform::from_matrix(m));
        set_if(&mut vis, Visibility::Visible);
        // SPEC-0034 4.7 (SPEC-0046 draw colour scale): with r, g and b all within 0.49 to 0.51 no
        // colour command is sent and the scale is (1, 1, 1, a'), a' = 0.008 + 0.992 x round(256 x
        // clamp(a, 0.008, 1)) / 256; otherwise (2r, 2g, 2b, a), so 0.5 is neutral. The scale is
        // in the GS's gamma space, so the RGB goes to linear like the vertex colours. (Before the
        // child-chunk models were drawn, the wrong save point models glared and this build used
        // c^2.2; reports 66, 69 and 94.)
        let neutral = c[..3].iter().all(|v| (0.49..=0.51).contains(v));
        let colour = if neutral {
            let a = 0.008 + 0.992 * (256.0 * c[3].clamp(0.008, 1.0)).round() / 256.0;
            Color::linear_rgba(1.0, 1.0, 1.0, a)
        } else {
            let g = |v: f32| (2.0 * v.clamp(0.0, 2.0)).powf(2.2);
            Color::linear_rgba(g(c[0]), g(c[1]), g(c[2]), c[3])
        };
        for h in &root.2 {
            if materials.get(h).is_some_and(|mat| mat.base_color != colour) {
                if let Some(mut mat) = materials.get_mut(h) {
                    mat.base_color = colour;
                }
            }
        }
    }
}

/// A region character's drawn root, by character id.
#[derive(Component)]
pub(crate) struct CharacterRoot(u64);

/// A character's attachment model root, by character id and slot.
#[derive(Component)]
pub(crate) struct AttachmentRoot(u64, u8);

/// SPEC-0046 Second walk and attachment slots: each character's attachment models at the frames
/// `Play::character_attachments` gives; hidden otherwise, despawned with their character.
fn sync_attachments(
    mut commands: Commands,
    state: Option<Res<PlayState>>,
    mut roots: Query<(Entity, &AttachmentRoot, &mut Transform, &mut Visibility)>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut pass1_materials: ResMut<Assets<pass1::Pass1Material>>,
    mut images: ResMut<Assets<Image>>,
    mut spawned: Local<std::collections::HashSet<(u64, u8)>>,
    mut lit: Local<std::collections::HashMap<u64, Vec<(Handle<Mesh>, usize)>>>,
) {
    let _span = crate::perf::span("sync_attachments");
    let Some(state) = state else { return };
    let p = &state.play;
    let frames: std::collections::HashMap<(u64, u8), neopets_data::anim::Affine> = p.character_attachments().into_iter().map(|(id, slot, f)| ((id, slot), f)).collect();
    let live: std::collections::HashSet<u64> = p.characters.iter().map(|c| c.id).collect();
    lit.retain(|id, _| live.contains(id));
    // SPEC-0046 Light slots and the lit draw: a world-table character's main attachment (slot 0)
    // is lit from the character's own light state; the other slots pass no context and keep the
    // scene default lights.
    for c in &p.characters {
        if let (Some(handles), Some(f), Some((_, m))) = (lit.get(&c.id), frames.get(&(c.id, 0)), c.model.attachments.iter().find(|(slot, _)| *slot == 0)) {
            relight_model(m, handles, f, &p.lights(Some(c.id), f.t), &mut meshes);
        }
    }
    for (e, root, mut t, mut vis) in &mut roots {
        if !live.contains(&root.0) {
            commands.entity(e).despawn();
            spawned.remove(&(root.0, root.1));
            continue;
        }
        match frames.get(&(root.0, root.1)) {
            Some(f) => {
                let c = |k: usize| Vec4::new(f.cols[k][0], f.cols[k][1], f.cols[k][2], 0.0);
                set_if(&mut t, Transform::from_matrix(Mat4::from_cols(c(0), c(1), c(2), Vec4::new(f.t[0], f.t[1], f.t[2], 1.0))));
                set_if(&mut vis, Visibility::Visible);
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
            let second = pass1_assets(m, &mut meshes, &mut pass1_materials, &mut images);
            if *slot == 0 {
                lit.insert(c.id, parts.iter().filter(|(k, ..)| m.parts[*k].lit).map(|(k, mesh, _)| (mesh.clone(), *k)).collect());
            }
            commands.spawn((AttachmentRoot(c.id, *slot), Transform::IDENTITY, Visibility::Hidden)).with_children(|ch| {
                for (_, mesh, material) in parts {
                    ch.spawn((Mesh3d(mesh), MeshMaterial3d(material), Transform::IDENTITY));
                }
                for (_, mesh, material, scroll) in second {
                    let mut e = ch.spawn((Mesh3d(mesh), MeshMaterial3d(material), Transform::IDENTITY));
                    if let Some(s) = scroll {
                        e.insert(pass1::Pass1Scroll(s));
                    }
                }
            });
        }
    }
}

/// Spawn, place, skin and relight the region characters of drawn worlds; despawn the rest.
#[allow(clippy::too_many_arguments)]
/// A character root's transform: its position and facing (or the carry rows while carried),
/// and a Cloud Beast stand-in's pose (bob, lean and scale; the disc has no clips for them).
fn character_root_transform(p: &play::Play, c: &play::Character) -> Transform {
    let mut want = Transform::from_translation(Vec3::from(c.position)).with_rotation(Quat::from_rotation_y(c.facing[0].atan2(c.facing[2])));
    if let Some([x, y, z]) = c.carry_rows {
        want.rotation = Quat::from_mat3(&Mat3::from_cols(Vec3::from(x), Vec3::from(y), Vec3::from(z)));
    }
    if let Some((bob, lean, s)) = p.cloud_pose(c) {
        want.translation.y += bob;
        want.rotation *= Quat::from_rotation_x(lean);
        want.scale = Vec3::splat(s);
    }
    want
}

fn sync_characters(
    mut commands: Commands,
    state: Option<ResMut<PlayState>>,
    mut roots: Query<(Entity, &CharacterRoot, &mut Transform, &mut Visibility)>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut pass1_materials: ResMut<Assets<pass1::Pass1Material>>,
    mut images: ResMut<Assets<Image>>,
    mut skinned: Local<std::collections::HashMap<u64, Vec<(Handle<Mesh>, usize)>>>,
    mut generation: Local<u32>,
    mut rigid: Local<std::collections::HashMap<u64, (Vec<(Handle<Mesh>, usize)>, bool)>>,
    mut kids: Local<std::collections::HashMap<u64, (Vec<Entity>, Option<(u64, bool)>)>>,
    seen: Query<&ViewVisibility>,
    cameras: Query<&GlobalTransform, (With<Camera3d>, Without<gpu_skin::SkinJoint>, Without<crate::water_reflect::MirrorCam>)>,
    (skin_assets, mut joint_query, mut gpu_joints): (Res<gpu_skin::SkinAssets>, Query<&mut GlobalTransform, With<gpu_skin::SkinJoint>>, Local<std::collections::HashMap<u64, Vec<Entity>>>),
) {
    let _span = crate::perf::span("sync_characters");
    let Some(state) = state else { return };
    let p = &state.play;
    let eye = cameras.iter().next().map(|g| g.translation());
    let live: std::collections::HashSet<u64> = p.characters.iter().map(|c| c.id).collect();
    // A rebuilt game (Quit or a load) draws every character again.
    let rebuilt = std::mem::replace(&mut *generation, p.generation) != p.generation;
    if rebuilt {
        skinned.clear();
        rigid.clear();
        kids.clear();
        gpu_joints.clear();
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
                set_if(&mut t, character_root_transform(p, c));
                // SPEC-0046 Drawable test: the visibility predicate hides a waiting enemy with cue
                // word 0 and an invisible kind in its default state or state 21 before clip 82.
                // SPEC-0010 Vanishing: the Cloud Beast's bodies during the gust's travel.
                let hidden = c.enemy.as_ref().is_some_and(|e| e.lock_vetoed(c.anim.clip_state().action)) || p.cloud_hidden(c);
                set_if(&mut vis, if p.pack_drawn(c.pack) && !c.inactive && !hidden { Visibility::Visible } else { Visibility::Hidden });
            }
        }
    }
    skinned.retain(|id, _| live.contains(id));
    gpu_joints.retain(|id, _| live.contains(id));
    rigid.retain(|id, _| live.contains(id));
    kids.retain(|id, _| live.contains(id));
    for c in &p.characters {
        let rotation = Quat::from_rotation_y(c.facing[0].atan2(c.facing[2]));
        if !skinned.contains_key(&c.id) {
            if std::env::var_os("NEOPETS_TRACE_CUT").is_some() {
                if let Some(e) = &c.enemy {
                    println!("spawn {}: default state {}, words {:?}", c.name, e.spawn.default_state, e.spawn.words);
                }
            }
            // NEOPETS_TRACE_CUT: each new character's triangles whose three vertex alphas are below
            // 0.5, textured and untextured (diagnostic).
            if std::env::var_os("NEOPETS_TRACE_CUT").is_some() {
                let (mut tex, mut plain, mut all) = (0, 0, 0);
                for part in &c.model.parts {
                    let cut = |i: u32| part.colors.get(i as usize).is_some_and(|q| q[3] < 0.5);
                    for t in part.indices.chunks_exact(3) {
                        all += 1;
                        if t.iter().all(|&i| cut(i)) {
                            if part.texture.is_some() { tex += 1 } else { plain += 1 }
                        }
                    }
                }
                println!("cut {}: {all} triangles, {tex} textured and {plain} untextured fully below 0.5", c.name);
            }
            let parts = part_assets(&c.model, &mut meshes, &mut materials, &mut images);
            // SPEC-0003 How a pass is drawn: the second-pass layer of two-pass parts, skinned with
            // pass 0 (its mesh shares the part's vertices, so the skinning loop writes both), and
            // an environment-mapped pass 0 drawn by the pass-1 shader as on the scenes.
            let second = pass1_assets(&c.model, &mut meshes, &mut pass1_materials, &mut images);
            let no_pass1 = std::env::var_os("NEOPETS_NO_PASS1").is_some();
            let mut env_textures: std::collections::HashMap<usize, Handle<Image>> = std::collections::HashMap::new();
            let env: std::collections::HashMap<usize, Handle<pass1::Pass1Material>> = c
                .model
                .parts
                .iter()
                .enumerate()
                .filter(|(_, q)| q.env_map && !no_pass1)
                .filter_map(|(i, q)| {
                    let t = q.texture?;
                    let tex = c.model.textures.get(t)?;
                    let image = env_textures.entry(t).or_insert_with(|| images.add(model_texture(tex))).clone();
                    Some((i, pass1_materials.add(pass1::env_base_for(image, q.alpha))))
                })
                .collect();
            let mut list = Vec::new();
            let mut spawned = Vec::new();
            // GPU skinning (gpu_skin.rs): the skinned parts keep their rest vertices and draw from
            // joint entities written each tick.
            let bones = c.anim.skin_matrices().len();
            let gpu = gpu_skin::enabled() && gpu_skin::fits(bones);
            if gpu {
                let skinned_parts = second.iter().map(|(i, m, _, _)| (*i, m)).chain(parts.iter().map(|(i, m, _)| (*i, m)));
                for (i, mesh) in skinned_parts {
                    let part = &c.model.parts[i];
                    if !part.weights.is_empty() {
                        if let Some(mut m) = meshes.get_mut(mesh) {
                            gpu_skin::prepare_mesh(&mut m, part, bones);
                        }
                    }
                }
            }
            let mut joints = Vec::new();
            let skin = gpu.then(|| skin_assets.bindposes.clone());
            commands
                .spawn((Transform::from_translation(Vec3::from(c.position)).with_rotation(rotation), Visibility::Visible, CharacterRoot(c.id)))
                .with_children(|ch| {
                    if gpu {
                        joints = (0..bones + gpu_skin::DATA_JOINTS + 1).map(|_| ch.spawn((GlobalTransform::IDENTITY, gpu_skin::SkinJoint)).id()).collect();
                    }
                    let skin_of = |joints: &Vec<Entity>| skin.clone().map(|h| (bevy::mesh::skinning::SkinnedMesh { inverse_bindposes: h, joints: joints.clone() }, bevy::camera::visibility::NoFrustumCulling));
                    for (i, mesh, material, scroll) in second {
                        let skinned_part = !c.model.parts[i].weights.is_empty();
                        if skinned_part && !gpu {
                            list.push((mesh.clone(), i));
                        }
                        let mut e = ch.spawn((Mesh3d(mesh), MeshMaterial3d(material), Transform::IDENTITY));
                        if let Some(s) = scroll {
                            e.insert(pass1::Pass1Scroll(s));
                        }
                        if let Some(k) = skin_of(&joints).filter(|_| skinned_part) {
                            e.insert(k);
                        }
                        spawned.push(e.id());
                    }
                    for (i, mesh, material) in parts {
                        let skinned_part = !c.model.parts[i].weights.is_empty();
                        if skinned_part {
                            if !gpu {
                                list.push((mesh.clone(), i));
                            }
                        } else if let (Some(col), Some(mut m)) = (tinted(&c.model.parts[i], None, None, Some(&c.slots)), meshes.get_mut(&mesh)) {
                            // A rigid part keeps its build colours: the slot colour once.
                            m.insert_attribute(Mesh::ATTRIBUTE_COLOR, col);
                            // The Cloud Beast stand-ins have no skinned parts; their rigid parts
                            // take the hit flash instead.
                            if p.cloud_pose(c).is_some() {
                                rigid.entry(c.id).or_default().0.push((mesh.clone(), i));
                            }
                        }
                        let mut e = match env.get(&i) {
                            Some(m) => ch.spawn((Mesh3d(mesh), MeshMaterial3d(m.clone()), Transform::IDENTITY)),
                            None => {
                                // A ghost's parts blend at its level (R-0202).
                                let material = if c.ghost.is_some() { blend_twin(&mut materials, &material) } else { material };
                                ch.spawn((Mesh3d(mesh), MeshMaterial3d(material), Transform::IDENTITY))
                            }
                        };
                        if let Some(k) = skin_of(&joints).filter(|_| skinned_part) {
                            e.insert(k);
                        }
                        spawned.push(e.id());
                    }
                });
            if gpu {
                gpu_joints.insert(c.id, joints);
            }
            skinned.insert(c.id, list);
            kids.insert(c.id, (spawned, None));
        }
        if !p.pack_drawn(c.pack) {
            continue;
        }
        if let Some((parts, was)) = rigid.get_mut(&c.id) {
            let flashing = c.flash > 0;
            if flashing != *was {
                *was = flashing;
                let slots = if flashing { &play::FLASH_COLOUR } else { &c.slots };
                for (handle, i) in parts.iter() {
                    if let (Some(col), Some(mut m)) = (tinted(&c.model.parts[*i], None, None, Some(slots)), meshes.get_mut(handle)) {
                        m.insert_attribute(Mesh::ATTRIBUTE_COLOR, col);
                    }
                }
            }
        }
        // Performance: the skin is rebuilt only for a drawn character (not inactive or
        // hiding), when one of its meshes was on screen last frame (or it has never been
        // skinned), and when a game tick has run or its flash changed since the last skin.
        let hidden = c.inactive || c.enemy.as_ref().is_some_and(|e| e.lock_vetoed(c.anim.clip_state().action));
        let Some((children, stamp)) = kids.get_mut(&c.id) else { continue };
        let now = (p.ticks, c.flash > 0);
        let on_screen = children.iter().any(|&k| seen.get(k).is_ok_and(|v| v.get()));
        // NEOPETS_PERF_NOSKIP: skin every character every frame, as before (diagnostic).
        static NOSKIP: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
        let noskip = *NOSKIP.get_or_init(|| std::env::var_os("NEOPETS_PERF_NOSKIP").is_some());
        // A far character re-skins less often: every tick within 25 units of the camera, every
        // second tick within 60, every fourth beyond (staggered by character); a flash change
        // re-skins at once.
        let d = eye.map_or(0.0, |e| e.distance(Vec3::from(c.position)));
        let every: u64 = if d < 25.0 { 1 } else if d < 60.0 { 2 } else { 4 };
        let due = stamp.is_none_or(|(t, f)| f != now.1 || (p.ticks >= t + every && (p.ticks + c.id) % every == 0) || p.ticks >= t + 2 * every);
        // NEOPETS_PERF_FREEZE: characters skinned once only (diagnostic, to measure uploads).
        static FREEZE: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
        let freeze = *FREEZE.get_or_init(|| std::env::var_os("NEOPETS_PERF_FREEZE").is_some());
        // GPU skinning: the joints are written on every tick (or flash change) of a drawn
        // character, near or far, on screen or not; only the matrices go to the GPU.
        if let Some(js) = gpu_joints.get(&c.id) {
            if hidden || *stamp == Some(now) || (freeze && stamp.is_some()) {
                continue;
            }
            *stamp = Some(now);
            let lights = p.lights(Some(c.id), c.position);
            let look = gpu_skin::Look {
                light_rotation: rotation,
                lights: &lights,
                tint: p.draw_tint(Some(c.id), c.position).unwrap_or([1.0; 3]),
                slots: if c.flash > 0 { &play::FLASH_COLOUR } else { &c.slots },
                // SPEC-0029 Behavior 33 (R-0202): a ghost blends at its level / 128.
                ghost: c.ghost.as_ref().map_or(1.0, |g| ((g.opacity * 128.0) as i32).clamp(0, 128) as f32 / 128.0),
            };
            let root = character_root_transform(p, c).compute_affine();
            for (e, m) in js.iter().zip(gpu_skin::joint_matrices(&root, &c.anim.skin_matrices(), &look)) {
                if let Ok(mut g) = joint_query.get_mut(*e) {
                    set_if(&mut g, m);
                }
            }
            continue;
        }
        if (freeze && stamp.is_some()) || (!noskip && (hidden || (stamp.is_some() && (!on_screen || !due)))) {
            continue;
        }
        *stamp = Some(now);
        let matrices = c.anim.skin_matrices();
        let tint = p.draw_tint(Some(c.id), c.position);
        let lights = p.lights(Some(c.id), c.position);
        for (handle, i) in &skinned[&c.id] {
            let part = &c.model.parts[*i];
            let (pos, nrm) = character::skin_part(&matrices, &part.positions, &part.normals, &part.weights, &part.bones);
            let slots = if c.flash > 0 { &play::FLASH_COLOUR } else { &c.slots };
            let mut colours = tinted(part, relight(part, &nrm, rotation, &lights), tint, Some(slots));
            // SPEC-0029 Behavior 33 (R-0202): a ghost blends at its level / 128 (level = opacity x 128).
            if let (Some(g), Some(col)) = (c.ghost.as_ref(), colours.as_mut()) {
                let level = ((g.opacity * 128.0) as i32).clamp(0, 128) as f32 / 128.0;
                col.iter_mut().for_each(|q| q[3] *= level);
            }
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

/// One drawn piece of the Cloud Beast's stand-in effects (index ranges in `CloudPiece`).
#[derive(Component)]
struct CloudFx(usize);

/// The pieces: the gust column and its swirl rings, the jagged bolt segments and glow of each
/// beam, the balls, the shock dome, the bursts, the cloud puffs and the companion's charge glow.
struct CloudPiece;

impl CloudPiece {
    const GUST: usize = 0;
    const RINGS: std::ops::Range<usize> = 1..5;
    /// Beam b's segments: BOLTS.start + b * SEGMENTS + i; its glow GLOWS.start + b.
    const SEGMENTS: usize = 8;
    const BOLTS: std::ops::Range<usize> = 5..21;
    const GLOWS: std::ops::Range<usize> = 21..23;
    const BALLS: std::ops::Range<usize> = 23..28;
    const SHOCK: usize = 28;
    const BURSTS: std::ops::Range<usize> = 29..45;
    const PUFFS: std::ops::Range<usize> = 45..61;
    const ORB: usize = 61;
    const COUNT: usize = 62;
}

/// A hash of three integers to 0..1 (the bolts' jitter; this build's own, no game data).
fn jitter(a: u32, b: u32, c: u32) -> f32 {
    let mut h = a.wrapping_mul(0x9E37_79B1) ^ b.wrapping_mul(0x85EB_CA77) ^ c.wrapping_mul(0xC2B2_AE3D);
    h ^= h >> 15;
    h = h.wrapping_mul(0x2C1B_3C6D);
    h ^= h >> 12;
    (h & 0xFFFF) as f32 / 65535.0
}

/// SPEC-0010's effect definitions 447 to 457 are not on the disc; these are this build's own
/// stand-ins (not the original's look): the gust as a pale column with swirl rings; each beam as
/// a jagged bolt of eight thin segments re-drawn every 3 ticks inside a soft glow, yellow for the
/// first beam and pale cyan for the second (the reflected or wall-bounced beam); glowing balls; a
/// pulsing dome for the shock; short sparks for bursts; sixteen soft cloud puffs drifting round
/// the body (the kind-340 cloud emitters, hidden during the gust's travel as the spec stops
/// them); and a flickering glow on the companion while the lightning winds up and fires.
fn sync_cloud_fx(
    mut commands: Commands,
    state: Option<Res<PlayState>>,
    mut pieces: Query<(&CloudFx, &mut Transform, &mut Visibility, &MeshMaterial3d<StandardMaterial>)>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let _span = crate::perf::span("sync_cloud_fx");
    let Some(state) = state else { return };
    let p = &state.play;
    let fx = p.cloud_fx();
    if fx.is_none() && p.cloud_bursts.is_empty() && pieces.is_empty() {
        return;
    }
    if pieces.is_empty() {
        let unit_cyl = meshes.add(Cylinder::new(1.0, 1.0));
        let ball = meshes.add(Sphere::new(1.0).mesh().uv(16, 10));
        let ring = meshes.add(Torus::new(0.85, 1.0));
        let mat = |m: &mut Assets<StandardMaterial>, c: Color, add: bool| {
            m.add(StandardMaterial { base_color: c, unlit: true, alpha_mode: if add { AlphaMode::Add } else { AlphaMode::Blend }, cull_mode: None, depth_bias: 1.0, ..default() })
        };
        for k in 0..CloudPiece::COUNT {
            let (mesh, colour, add) = match k {
                CloudPiece::GUST => (unit_cyl.clone(), Color::srgba(0.85, 0.93, 1.0, 0.25), false),
                k if CloudPiece::RINGS.contains(&k) => (ring.clone(), Color::srgba(0.9, 0.96, 1.0, 0.35), false),
                k if CloudPiece::BOLTS.contains(&k) => {
                    let second = k >= CloudPiece::BOLTS.start + CloudPiece::SEGMENTS;
                    (unit_cyl.clone(), if second { Color::srgba(0.75, 0.95, 1.0, 1.0) } else { Color::srgba(1.0, 1.0, 0.85, 1.0) }, true)
                }
                k if CloudPiece::GLOWS.contains(&k) => {
                    let second = k > CloudPiece::GLOWS.start;
                    (unit_cyl.clone(), if second { Color::srgba(0.3, 0.75, 1.0, 0.35) } else { Color::srgba(1.0, 0.8, 0.2, 0.35) }, true)
                }
                k if CloudPiece::BALLS.contains(&k) => (ball.clone(), Color::srgba(1.0, 0.92, 0.35, 0.9), false),
                CloudPiece::SHOCK => (ball.clone(), Color::srgba(0.55, 0.8, 1.0, 0.22), false),
                k if CloudPiece::BURSTS.contains(&k) => (ball.clone(), Color::srgba(1.0, 0.95, 0.5, 0.8), true),
                k if CloudPiece::PUFFS.contains(&k) => (ball.clone(), Color::srgba(0.86, 0.86, 0.95, 0.42), false),
                _ => (ball.clone(), Color::srgba(1.0, 0.9, 0.35, 0.6), true),
            };
            commands.spawn((CloudFx(k), Mesh3d(mesh), MeshMaterial3d(mat(&mut materials, colour, add)), Transform::IDENTITY, Visibility::Hidden, bevy::light::NotShadowCaster));
        }
        return;
    }
    // A column from a to b of radius r (the unit cylinder is along +Y, centred).
    let column = |a: Vec3, b: Vec3, r: f32| -> Option<Transform> {
        let d = b - a;
        let l = d.length();
        (l > 1e-3).then(|| Transform { translation: (a + b) * 0.5, rotation: Quat::from_rotation_arc(Vec3::Y, d / l), scale: Vec3::new(r, l, r) })
    };
    // The jagged bolt: point i of SEGMENTS + 1 from a to b, pushed off the line by up to 0.6 x
    // a sine taper (0 at both ends), re-drawn every 3 ticks.
    let bolt_point = |a: Vec3, b: Vec3, beam: u32, i: usize| -> Vec3 {
        let d = b - a;
        let l = d.length().max(1e-3);
        let dir = d / l;
        let side = dir.cross(Vec3::Y).try_normalize().unwrap_or(Vec3::X);
        let up = side.cross(dir);
        let f = i as f32 / CloudPiece::SEGMENTS as f32;
        let taper = (f * std::f32::consts::PI).sin();
        let frame = (p.ticks / 3) as u32;
        let (u, v) = (jitter(frame, beam, i as u32) - 0.5, jitter(frame, beam + 7, i as u32 + 31) - 0.5);
        let amp = (0.08 * l).min(1.2) * taper;
        a + d * f + (side * u + up * v) * 2.0 * amp
    };
    let draw = fx;
    let t = p.ticks as f32 / 60.0;
    for (piece, mut tr, mut vis, mat) in &mut pieces {
        let k = piece.0;
        let placed = match (k, draw.as_ref()) {
            (CloudPiece::GUST, Some(d)) => d.gust.and_then(|(o, e, fade)| {
                if let Some(mut m) = materials.get_mut(&mat.0) {
                    m.base_color = Color::srgba(0.85, 0.93, 1.0, 0.1 + 0.18 * (128 - fade.clamp(0, 128)) as f32 / 128.0);
                }
                column(Vec3::from(o), Vec3::from(e), 1.8)
            }),
            (k, Some(d)) if CloudPiece::RINGS.contains(&k) => d.gust.and_then(|(o, e, _)| {
                let (o, e) = (Vec3::from(o), Vec3::from(e));
                let axis = (e - o).try_normalize()?;
                let i = (k - CloudPiece::RINGS.start) as f32;
                let f = ((i + t * 2.5) / 4.0).fract();
                Some(Transform { translation: o + (e - o) * f, rotation: Quat::from_rotation_arc(Vec3::Y, axis) * Quat::from_rotation_y(t * 6.0 + i), scale: Vec3::splat(1.6 + 0.6 * (f * std::f32::consts::PI).sin()) })
            }),
            (k, Some(d)) if CloudPiece::BOLTS.contains(&k) => {
                let n = k - CloudPiece::BOLTS.start;
                let (b, i) = (n / CloudPiece::SEGMENTS, n % CloudPiece::SEGMENTS);
                d.beams.iter().find(|x| x.2 == (b == 1)).and_then(|&(a, e, _)| {
                    let (a, e) = (Vec3::from(a), Vec3::from(e));
                    column(bolt_point(a, e, b as u32, i), bolt_point(a, e, b as u32, i + 1), 0.09)
                })
            }
            (k, Some(d)) if CloudPiece::GLOWS.contains(&k) => {
                let b = k - CloudPiece::GLOWS.start;
                d.beams.iter().find(|x| x.2 == (b == 1)).and_then(|&(a, e, _)| column(Vec3::from(a), Vec3::from(e), 0.5 + 0.15 * (t * 30.0 + k as f32).sin()))
            }
            (k, Some(d)) if CloudPiece::BALLS.contains(&k) => d.balls.get(k - CloudPiece::BALLS.start).map(|&b| Transform::from_translation(Vec3::from(b)).with_scale(Vec3::splat(0.7 + 0.1 * (t * 20.0).sin()))),
            (CloudPiece::SHOCK, Some(d)) => d.shock.map(|(c, r)| Transform::from_translation(Vec3::from(c)).with_scale(Vec3::new(r, 0.35 * r, r) * (0.92 + 0.08 * (t * 18.0).sin()))),
            (k, _) if CloudPiece::BURSTS.contains(&k) => p.cloud_bursts.get(k - CloudPiece::BURSTS.start).map(|&(c, left)| Transform::from_translation(Vec3::from(c)).with_scale(Vec3::splat(0.4 + 0.12 * (20 - left.min(20)) as f32))),
            (k, Some(d)) if CloudPiece::PUFFS.contains(&k) && !d.hidden => {
                // The cloud darkens toward storm grey as the stages rise (this build's choice).
                let c = match d.stage {
                    1 => Color::srgba(0.86, 0.86, 0.95, 0.42),
                    2 => Color::srgba(0.62, 0.62, 0.72, 0.5),
                    _ => Color::srgba(0.38, 0.38, 0.48, 0.58),
                };
                if materials.get(&mat.0).is_some_and(|m| m.base_color != c) {
                    if let Some(mut m) = materials.get_mut(&mat.0) {
                        m.base_color = c;
                    }
                }
                // Puff i drifts round the body on its own orbit, height and breathing.
                let i = (k - CloudPiece::PUFFS.start) as f32;
                let a = i * 2.399 + t * (0.25 + 0.05 * (i % 3.0));
                let r = d.radius * (0.85 + 0.25 * (i * 1.7).sin()) + 0.3 * (t * 0.9 + i).sin();
                let h = -1.6 + 3.2 * ((i * 0.618).fract()) + 0.25 * (t * 1.3 + i).sin();
                let s = d.radius * (0.32 + 0.08 * (i * 2.3).cos()) * (1.0 + 0.1 * (t * 1.1 + i * 0.7).sin());
                Some(Transform::from_translation(Vec3::from(d.body) + Vec3::new(r * a.cos(), h, r * a.sin())).with_scale(Vec3::new(s, 0.75 * s, s)))
            }
            (CloudPiece::ORB, Some(d)) if !d.hidden => d.orb.zip(d.charge).map(|(o, ph)| {
                // Wind-up: a flicker that grows; fire: a steady bright glow.
                let flicker = 0.75 + 0.25 * jitter((p.ticks / 2) as u32, 3, 9);
                let s = if ph == 0 { 0.8 + 0.6 * flicker } else { 1.5 + 0.2 * (t * 40.0).sin() };
                Transform::from_translation(Vec3::from(o)).with_scale(Vec3::splat(s))
            }),
            _ => None,
        };
        match placed {
            Some(x) => {
                set_if(&mut tr, x);
                set_if(&mut vis, Visibility::Visible);
            }
            None => *vis = Visibility::Hidden,
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
    let mut opaque: std::collections::HashMap<usize, Handle<StandardMaterial>> = std::collections::HashMap::new();
    let mut additive: std::collections::HashMap<usize, Handle<StandardMaterial>> = std::collections::HashMap::new();
    m.parts
        .iter()
        .enumerate()
        .map(|(i, part)| {
            // SPEC-0046 (R-0128): program 6 draws with texture function highlight 2, so its
            // vertex alpha is an additive highlight, never the opacity: a program-6 part is solid
            // wherever its texture is (the Black Knight's armour and legs had alpha 0 and were
            // cut away). The highlight itself is not drawn.
            let solid;
            let part = if part.program == Some(6) && part.colors.iter().any(|c| c[3] < 1.0) {
                let mut q = part.clone();
                for c in &mut q.colors {
                    c[3] = 1.0;
                }
                solid = q;
                &solid
            } else {
                part
            };
            // A scrolling part (level water, SPEC-0046) has its own blended material, whose UV
            // offset moves every frame.
            // An additive pass (the effect lightning and bolts, SPEC-0003 Effect model particles)
            // keeps its additive blend while it scrolls.
            let scroll_adds = part.alpha.is_some() && crate::pass1::blend_of(part.alpha).0 < 2;
            let scrolling = match (part.scroll, part.texture) {
                (Some(_), Some(t)) => Some(materials.add(StandardMaterial {
                    base_color_texture: Some(images.add(model_texture(&m.textures[t]))),
                    unlit: true,
                    alpha_mode: if scroll_adds { AlphaMode::Add } else { AlphaMode::Blend },
                    cull_mode: None,
                    ..default()
                })),
                _ => None,
            };
            let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default());
            // The GS modulates in gamma space; Bevy blends vertex colour in linear space.
            let g = gs_unit(part);
            // An additive part's colour is capped at 1.0: the GS clamps the modulated colour at
            // 255 and the frame at full white (public GS documentation, TFX modulate and the
            // colour clamp), so in the original added light never passes white; uncapped, the
            // 2x vertex colours and the bloom made the save point's rays glare (user's report 66).
            let cap = if part.texture.is_some() && part.alpha.is_some() && crate::pass1::blend_of(part.alpha).0 < 2 { 1.0 } else { f32::MAX };
            // The add-by-FIX form (Cs x FIX + Cd) scales the added colour by FIX / 128; it was
            // drawn at full strength and the save point's rays stayed too strong (user's report 69).
            let fix = match crate::pass1::blend_of(part.alpha) {
                (1, f) if part.alpha.is_some() && part.texture.is_some() => f.min(1.0),
                _ => 1.0,
            };
            let colors: Vec<[f32; 4]> = part
                .colors
                .iter()
                .map(|c| [((c[0] * g).min(cap) * fix).powf(2.2), ((c[1] * g).min(cap) * fix).powf(2.2), ((c[2] * g).min(cap) * fix).powf(2.2), c[3].min(1.0)])
                .collect();
            // NEOPETS_TRACE_ADDITIVE: each additive part's blend register and colour range (diagnostic).
            if cap == 1.0 && std::env::var_os("NEOPETS_TRACE_ADDITIVE").is_some() {
                let (lo, hi) = part.colors.iter().fold((f32::MAX, 0.0f32), |(l, h), c| (l.min(c[3]), h.max(c[0].max(c[1]).max(c[2]))));
                println!("additive part {i}: alpha {:#x} fix {fix} vertex alpha min {lo:.2} colour max {hi:.2} vertices {}", part.alpha.unwrap_or(0), part.positions.len());
            }
            mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, part.positions.clone());
            mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, part.normals.clone());
            mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, part.uvs.clone());
            mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors);
            // A blended (scrolling) part keeps every triangle; a masked one drops the cut ones.
            mesh.insert_indices(Indices::U32(if scrolling.is_some() || (part.texture.is_some() && part.alpha.is_some() && crate::pass1::blend_of(part.alpha).0 < 2) { part.indices.clone() } else { drawn_indices(part) }));
            // SPEC-0003 How a pass is drawn: a pass-0 ALPHA of the additive family (Cs x As + Cd,
            // or Cs x FIX + Cd) adds the texture to the frame; drawn with the 0.5 mask it was a
            // grey panel (the sealed chest's light rays, user's report 62). The FIX scale is not
            // applied.
            let adds = part.texture.is_some() && crate::pass1::blend_of(part.alpha).0 < 2 && part.alpha.is_some();
            let material = scrolling.unwrap_or_else(|| match part.texture {
                Some(t) if adds => additive
                    .entry(t)
                    .or_insert_with(|| {
                        let mut own = materials.get(&handles[t]).cloned().unwrap_or_default();
                        own.alpha_mode = AlphaMode::Add;
                        materials.add(own)
                    })
                    .clone(),
                None => untextured.clone(),
                Some(t) if part.flipbook.is_none() && mask_never_cuts(&m.textures[t], part) => opaque.entry(t).or_insert_with(|| opaque_twin(materials, &handles[t])).clone(),
                Some(t) => handles[t].clone(),
            });
            (i, meshes.add(mesh), material)
        })
        .collect()
}

/// Render pass 1 of a model's two-pass parts (SPEC-0003 How a pass is drawn): per part, a
/// mesh with pass 0's positions, normals and colours and the pass-1 UVs, its material and,
/// for a scroll pass, the speeds.
pub(crate) fn pass1_assets(
    m: &neopets_data::model::ModelData,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<pass1::Pass1Material>,
    images: &mut Assets<Image>,
) -> Vec<(usize, Handle<Mesh>, Handle<pass1::Pass1Material>, Option<[f32; 2]>)> {
    use bevy::mesh::{Indices, PrimitiveTopology};
    let mut textures: std::collections::HashMap<usize, Handle<Image>> = std::collections::HashMap::new();
    let mut out = Vec::new();
    // NEOPETS_NO_PASS1: pass 1 is not drawn (diagnostic, for before-and-after comparisons).
    if std::env::var_os("NEOPETS_NO_PASS1").is_some() {
        return out;
    }
    for (i, part) in m.parts.iter().enumerate() {
        let Some(p1) = part.pass1.as_ref() else { continue };
        let Some(t) = m.textures.get(p1.texture) else { continue };
        let image = textures.entry(p1.texture).or_insert_with(|| images.add(model_texture(t))).clone();
        let (kind, fix) = pass1::blend_of(p1.alpha);
        let material = materials.add(pass1::Pass1Material {
            params: pass1::Pass1Params { mode: Vec4::new(if p1.uv_mode == 1 { 1.0 } else { 0.0 }, kind as f32, fix, 0.0), offset: Vec4::ZERO },
            texture: image,
            blend: kind == 2,
            mask: false,
        });
        let g = gs_unit(part);
        let colors: Vec<[f32; 4]> = part.colors.iter().map(|c| [(c[0] * g).powf(2.2), (c[1] * g).powf(2.2), (c[2] * g).powf(2.2), c[3].min(1.0)]).collect();
        let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default());
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, part.positions.clone());
        mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, part.normals.clone());
        mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, p1.uvs.clone());
        mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors);
        mesh.insert_indices(Indices::U32(part.indices.clone()));
        // NEOPETS_TRACE_PASS1: each pass-1 part's middle, UV mode and blend (diagnostic).
        if std::env::var_os("NEOPETS_TRACE_PASS1").is_some() {
            let n = part.positions.len().max(1) as f32;
            let c = part.positions.iter().fold([0.0f32; 3], |a, q| [a[0] + q[0] / n, a[1] + q[1] / n, a[2] + q[2] / n]);
            println!("pass1 part {i}: middle ({:.0}, {:.0}, {:.0}), {} vertices, uv mode {}, blend {:?}", c[0], c[1], c[2], part.positions.len(), p1.uv_mode, p1.alpha.map(|a| a & 0xFF));
        }
        out.push((i, meshes.add(mesh), material, (p1.uv_mode == 2).then_some(p1.scroll)));
    }
    out
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
/// A held model's lit parts recoloured from a light state at its frame's rotation.
fn relight_model(m: &neopets_data::model::ModelData, handles: &[(Handle<Mesh>, usize)], frame: &neopets_data::anim::Affine, lights: &neopets_data::light::Lights, meshes: &mut Assets<Mesh>) {
    let col = |k: usize| Vec3::from(frame.cols[k]).normalize_or_zero();
    let rotation = Quat::from_mat3(&Mat3::from_cols(col(0), col(1), col(2)));
    for (handle, k) in handles {
        let part = &m.parts[*k];
        if let (Some(colours), Some(mut mesh)) = (relight(part, &part.normals, rotation, lights), meshes.get_mut(handle)) {
            mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colours);
        }
    }
}

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
    let _span = crate::perf::span("choose_action");
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
    let _span = crate::perf::span("skin_hero");
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
    // NEOPETS_TRACE_REGIONS="a-b": the character names of regions a to b (diagnostic).
    if let Some((a, b)) = std::env::var("NEOPETS_TRACE_REGIONS").ok().and_then(|v| {
        let (a, b) = v.split_once('-')?;
        Some((a.trim().parse::<u32>().ok()?, b.trim().parse::<u32>().ok()?))
    }) {
        let table = match (index.read(disc, 530), index.read(disc, 531)) {
            (Ok(x), Ok(y)) => neopets_data::monster::MonsterTable::parse(&x, &y).unwrap_or_default(),
            _ => Default::default(),
        };
        for r in a..=b {
            match load_region(disc, index, &table, r) {
                Ok((list, _)) => println!("      region {r}: {:?}", list.iter().map(|c| match c.3.map_or(0, |row| row.can_fly) { 0 => c.0.name.clone(), f => format!("{} (fly {f})", c.0.name) }).collect::<Vec<_>>()),
                Err(e) => println!("      region {r}: {e}"),
            }
        }
    }
    let mut play = match play::Play::load_with_flags(disc, index, pack, PLAY_OUTFIT, flags) {
        Ok(play) => play,
        Err(e) => {
            eprintln!("Play mode could not start: {e}");
            std::process::exit(1)
        }
    };
    // NEOPETS_LOAD_SLOT=k: the trace starts from slot k (1 to 4) of the save file, as the
    // Load page would load it (diagnostic, for a player's save).
    if let Some(k) = std::env::var("NEOPETS_LOAD_SLOT").ok().and_then(|v| v.trim().parse::<usize>().ok()) {
        match read_save(&save_file()).and_then(|f| f.slot(k.saturating_sub(1)).cloned()) {
            Some(slot) => match load_game_from(disc, index, &play, &slot) {
                Ok(p) => {
                    play = p;
                    println!("      loaded slot {k}: world {}, hero at {:?}", play.world.pack, play.player.loco.position());
                }
                Err(e) => eprintln!("Slot {k} could not load: {e}"),
            },
            None => eprintln!("Slot {k} is empty or the save file is unreadable"),
        }
    }
    // NEOPETS_TRACE_GOD: the developer menu's god mode from the start (diagnostic).
    if std::env::var_os("NEOPETS_TRACE_GOD").is_some() {
        play.cheats.god = true;
    }
    // NEOPETS_TRACE_AUTO: the autopilot plays the trace (its line every 300 ticks).
    if std::env::var_os("NEOPETS_TRACE_AUTO").is_some() {
        play.toggle_autopilot();
    }
    // NEOPETS_TRACE_DUEL="limit": the duel trace, each enemy given at most `limit` ticks
    // (default 2400); its report is printed at the end.
    if let Ok(v) = std::env::var("NEOPETS_TRACE_DUEL") {
        play.fight_duel = Some(play::Duel { limit: v.trim().parse().unwrap_or(2400), ..Default::default() });
    }
    // NEOPETS_TRACE_BOSS=n: the trace starts as the developer menu's boss fight n does (its
    // world, start flags, stage grants, gear and placement; `play::BOSSES`), so a boss's own
    // timeline can be traced headlessly (diagnostic).
    if let Some(b) = std::env::var("NEOPETS_TRACE_BOSS").ok().and_then(|v| v.trim().parse::<usize>().ok()).and_then(|n| play::BOSSES.get(n).copied()) {
        match boss_fight(disc, index, &play, &b) {
            Ok(p) => {
                play = p;
                println!("      boss fight {} in {} (start index {})", b.name, b.pack, b.start);
                play.cheats.god |= std::env::var_os("NEOPETS_TRACE_GOD").is_some();
            }
            Err(e) => eprintln!("Boss fight {} could not start: {e}", b.name),
        }
    }
    // NEOPETS_TRACE_QUADS="x,z,r": the world scene's mesh parts of at most 8 vertices whose
    // centre is within r of (x, z), with their texture, zone and vertices.
    if let Some((x, z, r)) = std::env::var("NEOPETS_TRACE_QUADS").ok().and_then(|v| {
        let n: Vec<f32> = v.split(',').filter_map(|t| t.trim().parse().ok()).collect();
        (n.len() == 3).then(|| (n[0], n[1], n[2]))
    }) {
        let bytes = index.read(disc, pack).unwrap_or_default();
        if let Ok(chain) = neopets_data::resource::walk_chunks(&bytes, None) {
            for layers in [0u32, u32::MAX] {
                if let Ok(Some(m)) = archive_view::world_scene(disc, index, pack, &bytes, &chain.chunks, false, layers) {
                    for (i, p) in m.parts.iter().enumerate() {
                        let n = p.positions.len().max(1) as f32;
                        let c = p.positions.iter().fold([0.0f32; 3], |a, q| [a[0] + q[0] / n, a[1] + q[1] / n, a[2] + q[2] / n]);
                        if p.positions.len() <= 8 && (c[0] - x).hypot(c[2] - z) <= r {
                            println!("      layers {layers:#x} part {i}: centre ({:.1}, {:.1}, {:.1}) texture {:?} zone {:?} env {} cull {:?} positions {:?}", c[0], c[1], c[2], p.texture, p.zone, p.env_map, p.cull, p.positions.iter().map(|q| q.map(|v| (v * 10.0).round() / 10.0)).collect::<Vec<_>>());
                        }
                    }
                }
            }
        }
    }
    // NEOPETS_TRACE_NP=n: the wallet starts at n Neopoints (diagnostic).
    if let Some(n) = std::env::var("NEOPETS_TRACE_NP").ok().and_then(|v| v.trim().parse::<i32>().ok()) {
        play.state.neopoints = n;
    }
    // NEOPETS_TRACE_BOSS=n: start as the developer menu's boss fight n instead (trace only).
    if let Some(b) = std::env::var("NEOPETS_TRACE_BOSS").ok().and_then(|v| v.parse::<usize>().ok()).and_then(|i| play::BOSSES.get(i)) {
        match boss_fight(disc, index, &play, b) {
            Ok(p) => play = p,
            Err(e) => eprintln!("Boss fight {} failed: {e}", b.name),
        }
    }
    // `--seed N`: a fixed seed for the random stream (SPEC-0008 developer option).
    if let Some(n) = seed {
        play.rng = neopets_sim::rng::Rng::seeded(n);
    }
    // NEOPETS_TRACE_ENTRY=k: start at the pack's entry record k (diagnostic, for sweeps from
    // the places a player arrives); the count is printed either way.
    println!("Start points: {}", play.dev_start_points());
    if let Some(k) = std::env::var("NEOPETS_TRACE_ENTRY").ok().and_then(|v| v.trim().parse::<usize>().ok()) {
        play.dev_go_start(k);
    }
    // NEOPETS_LIST_WORLDS: print the world packs of the save's world-key list and stop
    // (diagnostic, for sweeps over every area).
    if std::env::var_os("NEOPETS_LIST_WORLDS").is_some() {
        let mut v: Vec<i32> = play.world_keys.0.iter().copied().filter(|&k| k > 0).map(|k| k as i32).collect();
        v.sort_unstable();
        v.dedup();
        println!("WORLDS {}", v.iter().map(|k| k.to_string()).collect::<Vec<_>>().join(" "));
        return;
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
    // NEOPETS_TRACE_CLOAK: the hero starts wearing the Tattered Cloak (trace only).
    if std::env::var_os("NEOPETS_TRACE_CLOAK").is_some() {
        play.cloak_worn = true;
        play.state.cloak_worn = true;
    }
    // NEOPETS_TRACE_ROOTS="a,b,...": each hero clip's key count and summed root records.
    if let Ok(v) = std::env::var("NEOPETS_TRACE_ROOTS") {
        for id in v.split(',').filter_map(|t| t.trim().parse::<u32>().ok()) {
            println!("      hero clip {id}: {:?}", play.player.hero.root_sum(id));
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
    // NEOPETS_TRACE_CEILINGS: the world's ceiling panels with their middle points and links (diagnostic).
    if std::env::var_os("NEOPETS_TRACE_CEILINGS").is_some() {
        for (k, r) in play.world.panels.iter().enumerate().filter(|(_, r)| r.normal[1] < -0.99) {
            let mid: [f32; 3] = std::array::from_fn(|j| -r.normal[3] * r.normal[j] + (0.5 * r.lu - r.u[3]) * r.u[j] + (0.5 * r.lv - r.v[3]) * r.v[j]);
            println!("      ceiling {k}: middle {mid:?} U {:?} LU {} LV {} links {:?}", r.u, r.lu, r.lv, r.links);
        }
    }
    // NEOPETS_TRACE_ITEMLIST=1: every item of the item table with its type and label (diagnostic).
    if std::env::var_os("NEOPETS_TRACE_ITEMLIST").is_some() {
        for i in &play.items.items {
            println!("ITEM {} kind {} {}", i.id, i.kind, play.item_label(i.id));
        }
    }
    // NEOPETS_TRACE_OBJAUDIT=1: objects still in their disc state 3 or 4 that this build treats as drawn, with their wall counts (diagnostic).
    if std::env::var_os("NEOPETS_TRACE_OBJAUDIT").is_some() {
        println!("OBJAUDIT {} {:?}", play.world.pack, play.undrawn_but_shown());
        println!("RAILOWNERS {} {:?}", play.world.pack, play.rail_owners());
        println!("PANELOWNERS {} {:?} rails {}", play.world.pack, play.panel_owners(), play.world.rails.len());
        println!("CLIMBDEBUG {}", play.climb_debug());
        println!("GATED {}", play.gated_boxes());
        for (k, m) in play.world.navmeshes.iter().enumerate() {
            let (lo, hi) = m.triangles.iter().flatten().fold(([f32::MAX; 3], [f32::MIN; 3]), |(lo, hi), p| (std::array::from_fn(|i| lo[i].min(p[i])), std::array::from_fn(|i| hi[i].max(p[i]))));
            println!("NAVMESH {k}: kind {} triangles {} bounds {:?} to {:?}", m.kind, m.triangles.len(), lo.map(|v| v.round()), hi.map(|v| v.round()));
        }
    }
    // NEOPETS_TRACE_CUE_RECORDS=1: the type-4 and type-5 records of all sound blocks, against SPEC-0031's counts (diagnostic).
    if std::env::var_os("NEOPETS_TRACE_CUE_RECORDS").is_some() {
        if let Ok(mut second) = disc.reopen() {
            let (mut four, mut five, mut targets, mut shapes) = (0, 0, [0; 7], [0; 6]);
            for number in 347..=490 {
                let Ok(b) = index.read(&mut second, number).and_then(|b| neopets_data::block::Block::parse(&b)) else { continue };
                four += b.commands.iter().filter(|c| c.kind == 4).count();
                five += b.commands.iter().filter(|c| c.kind == 5).count();
                for c in b.commands.iter().filter(|c| c.kind == 4) {
                    if let Some(r) = b.records.get(&c.value()) {
                        targets[(r[1] as usize).min(6)] += 1;
                        shapes[(r[3] as usize).min(5)] += 1;
                    }
                }
            }
            println!("CUE RECORDS type 4 {four}, type 5 {five}; targets 0 to 6 {targets:?}; shapes 0 to 5 {shapes:?}");
        }
    }
    // NEOPETS_TRACE_DOORS=1: this world's doors with their script context, or none where a scripted door has no rule (diagnostic).
    if std::env::var_os("NEOPETS_TRACE_DOORS").is_some() {
        let doors: Vec<Option<u16>> = (0..play.world.planes.len()).map(|n| neopets_sim::level::teleport_context(play.world.pack, n)).collect();
        println!("DOORS {} scripted {} {:?}", play.world.pack, play.world.scripted_doors, doors);
    }
    // NEOPETS_TRACE_CATALOG=1: the developer menu's character list, one line each (diagnostic).
    if std::env::var_os("NEOPETS_TRACE_CATALOG").is_some() {
        if let Ok(mut second) = disc.reopen() {
            dev_spawn::scan(&mut second, index, |list| {
                for e in list {
                    println!("CATALOG {:?} {} {} {:?}", e.group, e.region, e.ordinal, e.name);
                }
            });
        }
    }
    // NEOPETS_TRACE_GROUP=region: the developer menu's cut group loaded in the trace's world (diagnostic).
    if let Some(r) = std::env::var("NEOPETS_TRACE_GROUP").ok().and_then(|v| v.parse::<u32>().ok()) {
        play.dev_load_group(r);
    }
    // NEOPETS_TRACE_ROBERTA=1: the trace starts as Roberta (diagnostic; her attack fires shots).
    if std::env::var("NEOPETS_TRACE_ROBERTA").is_ok_and(|v| v == "1") {
        play.trace_select_hero(1);
    }
    // NEOPETS_TRACE_TARGET=<name>: the fight trace fights only characters with that name.
    play.fight_target = std::env::var("NEOPETS_TRACE_TARGET").ok().filter(|n| !n.is_empty());
    // NEOPETS_TRACE_CHEATS="god,magic,charge": the developer menu's cheats from the start.
    if let Ok(v) = std::env::var("NEOPETS_TRACE_CHEATS") {
        play.cheats = play::Cheats { god: v.contains("god"), magic: v.contains("magic"), charge: v.contains("charge"), noclip: v.contains("noclip"), haste: 0, swarm: v.contains("swarm"), spin: v.contains("spin") };
    }
    // NEOPETS_TRACE_HASTE=k: the developer menu's Hero Haste slider at step k (17 is 10x; diagnostic).
    if let Some(k) = std::env::var("NEOPETS_TRACE_HASTE").ok().and_then(|v| v.parse::<u8>().ok()) {
        play.cheats.haste = k;
    }
    // NEOPETS_TRACE_HP=n: the hero's starting hit points (diagnostic); a value above the
    // maximum raises the maximum too, so the boss-fight refills keep it.
    if let Some(hp) = std::env::var("NEOPETS_TRACE_HP").ok().and_then(|v| v.parse::<i32>().ok()) {
        play.player.hero_hp = hp;
        play.player.hero_max = play.player.hero_max.max(hp);
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
    println!("Location: {}", String::from_utf8_lossy(&play.location_name()));
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
    // NEOPETS_TRACE_BREAKABLES: each breakable's position, state and drop (placement ordinal).
    if std::env::var_os("NEOPETS_TRACE_BREAKABLES").is_some() {
        for (i, b) in play.world.breakables.iter().enumerate() {
            let p = b.position();
            println!("      breakable {i}: at ({:.1}, {:.1}, {:.1}) state {} drop {} script {}", p[0], p[1], p[2], b.state, b.drop, b.script);
        }
    }
    // NEOPETS_TRACE_PATCHES: the world's foliage patches (key, family, box) (diagnostic).
    if std::env::var_os("NEOPETS_TRACE_PATCHES").is_some() {
        for (i, p) in play.world.patches.iter().enumerate() {
            println!("      patch {i}: key {} {:?} box {:?}", p.key, neopets_sim::foliage::kind(p.key).map(|k| k.family), p.area);
        }
    }
    // NEOPETS_TRACE_LIMITS: each named item's text category and carry limit (diagnostic).
    if std::env::var_os("NEOPETS_TRACE_LIMITS").is_some() {
        for line in play.item_limits() {
            println!("      {line}");
        }
    }
    println!("Book stations of {} (value, trigger index): {:?}", play.world.pack, play.book_stations());
    println!("Script triggers of {} (index, kind, middle): {:?}", play.world.pack, play.trigger_points());
    if std::env::var_os("NEOPETS_TRACE_LOCATORS").is_some() {
        for (k, e) in play.world.entries.iter().enumerate() {
            println!("      locator {k}: tag {:?} at {:?} facing {:?}", play.world.tags.get(k), e.0.map(|x| (x * 100.0).round() / 100.0), e.1.map(|x| (x * 100.0).round() / 100.0));
        }
    }
    // NEOPETS_TRACE_NEAR="x,z,r": the placed objects whose rest origin is within r of (x, z).
    if let Some((x, z, r)) = std::env::var("NEOPETS_TRACE_NEAR").ok().and_then(|v| {
        let n: Vec<f32> = v.split(',').filter_map(|t| t.trim().parse().ok()).collect();
        (n.len() == 3).then(|| (n[0], n[1], n[2]))
    }) {
        for (k, o) in play.world.objects.iter().enumerate() {
            let Some(o) = o else { continue };
            let t = [o.rest[12], o.rest[13], o.rest[14]];
            if (t[0] - x).hypot(t[2] - z) <= r {
                println!("      object {k}: at ({:.1}, {:.1}, {:.1}) scale {:?} state {} keys {} zone {} tree {} set {} header {} anim {:?}", t[0], t[1], t[2], o.scale, o.state, o.keys, o.zone, o.tree.is_some(), o.model_set.is_some(), o.header.is_some(), play.world.object_anims.get(k).map(|a| (a.state, a.visible())));
            }
        }
    }
    if std::env::var_os("NEOPETS_TRACE_TRIGGERS").is_some() {
        for v in &play.world.all_volumes {
            println!("      trigger {} kind {} flags {:#x} mask {:#x} {:?}", v.index, v.kind, v.flags, v.mask, v.shape);
        }
    }
    println!("Entity table of {} (entity: region j or 255 embedded, ordinal): {:?}", play.world.pack, play.world.entity_table());
    let placed: Vec<String> = play.world.placed_effects.iter().take(40).enumerate().map(|(k, p)| format!("{k}: id {} at {:.1},{:.1},{:.1}", p.id, p.position[0], p.position[1], p.position[2])).collect();
    println!("Placed effects of {} ({} records): {}", play.world.pack, play.world.placed_effects.len(), placed.join("; "));
    if std::env::var_os("NEOPETS_TRACE_PATCHES").is_some() {
        for p in &play.world.patches {
            println!("      PATCH key {} area {:?} height {} ground {:?}", p.key, p.area, p.height, play.world.ground_at((p.area[0] + p.area[1]) / 2.0, (p.area[2] + p.area[3]) / 2.0, 1.0e4));
        }
    }
    // NEOPETS_TRACE_ATMOS=1: the type-41 atmosphere zones of the pack (diagnostic).
    if std::env::var_os("NEOPETS_TRACE_ATMOS").is_some() {
        if let Ok(bytes) = archive.index.read(&mut archive.disc, play.world.pack) {
            if let Ok(chain) = neopets_data::resource::walk_chunks(&bytes, None) {
                for (k, z) in neopets_data::atmosphere::zones(&bytes, &chain.chunks).iter().enumerate() {
                    println!("ATMOS {} zone {k}: key {} A {:?} B {:?} depth {} centre {:?} radius {} points {} closed {}", play.world.pack, z.key, z.a, z.b, z.depth, z.centre, z.radius, z.points.len(), z.closed);
                }
            }
        }
    }
    // NEOPETS_TRACE_INSTANCES=1: the type-20 instances of this world: tree, translation, axis lengths, zone, D1, the ground under it (diagnostic).
    if std::env::var_os("NEOPETS_TRACE_INSTANCES").is_some() {
        if let Ok(bytes) = archive.index.read(&mut archive.disc, play.world.pack) {
            if let Ok(chain) = neopets_data::resource::walk_chunks(&bytes, None) {
                for t in neopets_data::world::scene(&bytes, &chain.chunks).trees.iter().filter(|t| t.source == neopets_data::world::INSTANCES) {
                    let Some(m) = t.matrix else { continue };
                    let len = |k: usize| (m[k] * m[k] + m[k + 1] * m[k + 1] + m[k + 2] * m[k + 2]).sqrt();
                    println!("      INSTANCE tree {:#x} at {:.1},{:.1},{:.1} axes {:.2},{:.2},{:.2} zone {:?} cull {:?} ground {:?}", t.payload.start, m[12], m[13], m[14], len(0), len(4), len(8), t.zone, t.cull, play.world.ground_at(m[12], m[14], 1.0e4));
                }
            }
        }
    }
    // NEOPETS_TRACE_PARTS="x0,z0,x1,z1": the scene parts of this world inside that box (diagnostic).
    if let Some(v) = std::env::var("NEOPETS_TRACE_PARTS").ok().map(|v| v.split(',').filter_map(|t| t.trim().parse::<f32>().ok()).collect::<Vec<f32>>()).filter(|v| v.len() == 4) {
        if let Ok(bytes) = archive.index.read(&mut archive.disc, play.world.pack) {
            if let Ok(chain) = neopets_data::resource::walk_chunks(&bytes, None) {
                if let Ok(m) = neopets_data::model::build_world(&bytes, &chain.chunks, u32::MAX) {
                    for (i, q) in m.parts.iter().enumerate() {
                        let (lo, hi) = q.positions.iter().fold(([f32::MAX; 3], [f32::MIN; 3]), |(lo, hi), p| (std::array::from_fn(|k| lo[k].min(p[k])), std::array::from_fn(|k| hi[k].max(p[k]))));
                        if hi[0] >= v[0] && lo[0] <= v[2] && hi[2] >= v[1] && lo[2] <= v[3] && hi[0] - lo[0] < 150.0 && hi[2] - lo[2] < 150.0 {
                            let g = play.world.ground_at(0.5 * (lo[0] + hi[0]), 0.5 * (lo[2] + hi[2]), hi[1] + 8.0);
                            println!("      PART {i} tree {} verts {} lo {lo:?} hi {hi:?} zone {:?} group {:?} tex {:?} lit {} pass {} bucket {:?} ground {g:?}", q.tree, q.positions.len(), q.zone, q.group, q.texture, q.lit, q.pass, q.bucket);
                        }
                    }
                }
            }
        }
    }
    // NEOPETS_TRACE_PART_VERTS=n: the scene parts of this world with n vertices: their vertex alpha, in tenths, and material (diagnostic).
    if let Some(n) = std::env::var("NEOPETS_TRACE_PART_VERTS").ok().and_then(|v| v.parse::<usize>().ok()) {
        if let Ok(bytes) = archive.index.read(&mut archive.disc, play.world.pack) {
            if let Ok(chain) = neopets_data::resource::walk_chunks(&bytes, None) {
                if let Ok(m) = neopets_data::model::build_world(&bytes, &chain.chunks, u32::MAX) {
                    for (i, q) in m.parts.iter().enumerate().filter(|(_, q)| q.positions.len() == n) {
                        let mut tenths = [0usize; 13];
                        for c in &q.colors {
                            tenths[((c[3] * 10.0) as usize).min(12)] += 1;
                        }
                        let tex = q.texture.and_then(|t| m.textures.get(t)).map(|t| { let a: Vec<u8> = t.image.rgba.chunks_exact(4).map(|p| p[3]).collect(); (t.image.width, t.image.height, a.iter().copied().min(), a.iter().copied().max()) });
                        println!("      PARTV {i} group {:?} program {:?} zone {:?} pass {} bucket {:?} scroll {:?} flip {} env {} lit {} pass1 {} alpha tenths {tenths:?} tex {:?} fades {}", q.group, q.program, q.zone, q.pass, q.bucket, q.scroll, q.flipbook.is_some(), q.env_map, q.lit, q.pass1.is_some(), tex, vertex_fades(q));
                    }
                }
            }
        }
    }
    let trees: Vec<String> = play.world.patches.iter().filter(|p| p.key == 4).take(3).map(|p| format!("({:.1},{:.1})", (p.area[0] + p.area[1]) / 2.0, (p.area[2] + p.area[3]) / 2.0)).collect();
    println!("Foliage of {}: {} patches; key-4 tree patches at {}", play.world.pack, play.world.patches.len(), trees.join(" "));
    let kinds: Vec<String> = play.world.groups.iter().enumerate().map(|(i, g)| format!("{i}:k{} t{:?} o{:?}", g.kind, g.triggers.first(), g.objects.iter().take(2).collect::<Vec<_>>())).collect();
    println!("Script groups of {}: {}", play.world.pack, kinds.join(" "));
    println!("Script volumes of {} (index, a point inside): {:?}", play.world.pack, play.world.volume_points());
    let solid_kinds: Vec<usize> = [1u8, 3, 4, 7].iter().map(|k| play.world.solids.iter().filter(|v| v.kind == *k).count()).collect();
    let hero = play.player.loco.position();
    let mut near: Vec<(f32, u8, [f32; 3])> = play.world.solids.iter().map(|v| {
        let at = match v.shape {
            neopets_sim::trigger::Shape::Sphere { centre, .. } => centre,
            neopets_sim::trigger::Shape::Cylinder { base, .. } => base,
            neopets_sim::trigger::Shape::Rectangle { origin, .. } => origin,
        };
        (((at[0] - hero[0]).powi(2) + (at[2] - hero[2]).powi(2)).sqrt(), v.kind, at)
    }).collect();
    near.sort_by(|a, b| a.0.total_cmp(&b.0));
    println!("Solid volumes of {} (kinds 1, 3, 4, 7): {:?}; nearest {:?}", play.world.pack, solid_kinds, near.iter().take(3).map(|n| (n.1, n.2.map(|x| x.round()))).collect::<Vec<_>>());
    if to.is_some_and(|t| t[0].is_nan()) {
        let goal = match to.map(|t| t[1]) {
            Some(g) if g == 0.0 => play::Goal::NewNeighbour,
            Some(g) if g == 1.0 => play::Goal::Door,
            _ => play::Goal::Swap,
        };
        route = if to.is_some_and(|t| t[1] >= 2.0) {
            Vec::new()
        } else if goal == play::Goal::Door {
            // NEOPETS_TRACE_DOOR=k: the first door walked through is door k (default 0).
            let k = std::env::var("NEOPETS_TRACE_DOOR").ok().and_then(|v| v.trim().parse().ok()).unwrap_or(0);
            println!("      doors of {}: {}", play.world.pack, play.world.planes.len());
            play.stand_before_door(k).map(|m| vec![m]).unwrap_or_default()
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
    let mut moving_before: Vec<bool> = Vec::new();
    let cross_at: Vec<u64> = std::env::var("NEOPETS_TRACE_CROSS").unwrap_or_default().split(',').filter_map(|t| t.trim().parse().ok()).collect();
    let mut replay = debug_tools::Replay::from_env();
    let mut last_sun = String::new();
    let switch_at: Vec<u64> = std::env::var("NEOPETS_TRACE_SWITCH").unwrap_or_default().split(',').filter_map(|t| t.trim().parse().ok()).collect();
    let mut flags_seen = play.state.flags.clone();
    // NEOPETS_TRACE_HOLD=n: the fight trace holds the attack n ticks (default 10; 50 or more
    // releases a power attack).
    let hold: u64 = std::env::var("NEOPETS_TRACE_HOLD").ok().and_then(|v| v.parse().ok()).unwrap_or(10).min(240);
    let mice = std::env::var_os("NEOPETS_TRACE_MICE").is_some();
    let mut faerie_code = 0u8;
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
    // NEOPETS_TRACE_STORY=path: the story checker's goal file, run through in order (scripts/story_check.py).
    let mut story = std::env::var("NEOPETS_TRACE_STORY").ok().and_then(|p| story_run::StoryRun::from_file(&p));
    let mut story_order = story_run::Order::default();
    // NEOPETS_TRACE_QUIT=n: Quit's world teardown on tick n (diagnostic; the pause page's Quit).
    let mut quit_at: Option<u64> = std::env::var("NEOPETS_TRACE_QUIT").ok().and_then(|v| v.trim().parse().ok());
    for _ in 0..ticks {
        // The duel trace ends once every enemy it found has been fought (after a 600-tick
        // grace for regions to load).
        if play.ticks > 600 && play.fight_duel.as_ref().is_some_and(|d| d.done) {
            break;
        }
        // Once: the new game's tick count starts again. A game-over exit with nothing pending
        // tears the world down the same way.
        // NEOPETS_TRACE_CEILING="tick:k": the hero grabs ceiling panel k on that tick (diagnostic).
        if let Some((t, k)) = std::env::var("NEOPETS_TRACE_CEILING").ok().and_then(|v| Some((v.split(':').next()?.parse::<u64>().ok()?, v.split(':').nth(1)?.parse::<usize>().ok()?))) {
            if t == play.ticks {
                play.diag_ceiling(k);
            }
        }
        // NEOPETS_TRACE_CHEST="tick:group": chest group pays out with the hero at its locator; its drops every 30 ticks.
        if let Some((k, g)) = std::env::var("NEOPETS_TRACE_CHEST").ok().as_deref().and_then(|v| v.split_once(':')).and_then(|(a, b)| Some((a.trim().parse::<u64>().ok()?, b.trim().parse::<usize>().ok()?))) {
            if play.ticks == k {
                play.diag_chest(g);
            }
            if play.ticks >= k && (play.ticks - k) % 30 == 0 {
                println!("DROPS {}: {}", play.ticks, play.diag_drops());
            }
        }
        // NEOPETS_TRACE_WORDS="pack:a,b,...": those module words of the pack, every 30 ticks (diagnostic).
        if let Some((pk, ws)) = std::env::var("NEOPETS_TRACE_WORDS").ok().as_deref().and_then(|v| v.split_once(':')).map(|(a, b)| (a.trim().parse::<u32>().unwrap_or(0), b.split(',').filter_map(|w| w.trim().parse::<usize>().ok()).collect::<Vec<_>>())) {
            if play.ticks % if std::env::var_os("NEOPETS_TRACE_WORDS_EVERY").is_some() { 1 } else { 30 } == 0 {
                println!("WORDS tick {} pack {pk}: {:?} shots {} at {:?}", play.ticks, ws.iter().map(|&w| play.state.words.get(&(pk, w as u8)).copied().unwrap_or(0)).collect::<Vec<_>>(), play.shots.slots.iter().flatten().count(), play.shots.slots.iter().flatten().map(|s| s.position.map(|v| v.round() as i32)).collect::<Vec<_>>());
            }
        }
        // NEOPETS_TRACE_PLACED="pack:a-b": how many of those placed effects are active, every 100 ticks.
        if let Some((pack, range)) = std::env::var("NEOPETS_TRACE_PLACED").ok().as_deref().and_then(|v| v.split_once(':')).map(|(a, b)| (a.to_string(), b.to_string())) {
            if let (Ok(pack), Some((a, b))) = (pack.parse::<u32>(), range.split_once('-')) {
                if play.ticks % 100 == 0 {
                    println!("PLACED tick {} world {}: {} active", play.ticks, play.world.pack, play.diag_placed(pack, a.parse().unwrap_or(0), b.parse().unwrap_or(0)));
                }
            }
        }
        // NEOPETS_TRACE_LOCMESH="tick:a-b": which walking maps hold locators a to b of the current world.
        if let Some((k, range)) = std::env::var("NEOPETS_TRACE_LOCMESH").ok().as_deref().and_then(|v| v.split_once(':')).map(|(a, b)| (a.to_string(), b.to_string())) {
            if k.parse::<u64>().ok() == Some(play.ticks) {
                if let Some((a, b)) = range.split_once('-') {
                    play.diag_locator_meshes(a.parse().unwrap_or(0), b.parse().unwrap_or(0));
                }
            }
        }
        // NEOPETS_TRACE_WILD=tick: the hero placed beside the first wild petpet on that tick; the petpets every 60 ticks.
        if let Some(k) = std::env::var("NEOPETS_TRACE_WILD").ok().and_then(|v| v.trim().parse::<u64>().ok()) {
            if play.ticks == k || play.ticks % 60 == 0 {
                let tick = play.ticks;
                println!("WILD {}: {}", tick, play.diag_wild(tick == k));
            }
        }
        // NEOPETS_TRACE_CHESTDROP="tick:group": the hero dropped onto the chest's body; its state every 30 ticks.
        if let Some((k, g)) = std::env::var("NEOPETS_TRACE_CHESTDROP").ok().as_deref().and_then(|v| v.split_once(':')).and_then(|(a, b)| Some((a.trim().parse::<u64>().ok()?, b.trim().parse::<usize>().ok()?))) {
            if play.ticks % 30 == 0 || play.ticks == k {
                let tick = play.ticks;
                println!("CHESTBODY {tick}: {:?}", play.diag_chest_body(g, tick == k));
            }
        }
        // NEOPETS_TRACE_MODELS=tick: on that tick the live items with no model, and each named character's model parts.
        if std::env::var("NEOPETS_TRACE_MODELS").ok().and_then(|v| v.trim().parse::<u64>().ok()) == Some(play.ticks) {
            let mut boot = None;
            let mut ids: Vec<(u32, u16)> = play.pools.iter().flat_map(|(&pk, pool)| pool.slots.iter().flatten().map(move |it| (pk, it.id))).collect();
            ids.extend([151u16, 154, 174].map(|id| (play.world.pack, id)));
            ids.sort_unstable();
            ids.dedup();
            for (pk, id) in ids {
                let parts = item_model(&mut archive.disc, &archive.index, &mut boot, id as u32, pk).ok().flatten().map(|m| m.parts.len());
                println!("MODEL item {id} of {pk}: parts {parts:?}");
            }
            let mut seen = std::collections::HashSet::new();
            for c in &play.characters {
                if seen.insert(c.name.clone()) {
                    println!("MODEL {:?}: attachment slots {:?}", c.name, c.model.attachments.iter().map(|(k, _)| *k).collect::<Vec<_>>());
                }
            }
        }
        // NEOPETS_TRACE_OWNED="tick:x,z,r,top": the object-owned ground patches near (x, z) (diagnostic).
        if let Some((k, n)) = std::env::var("NEOPETS_TRACE_OWNED").ok().as_deref().and_then(|v| v.split_once(':')).map(|(a, b)| (a.trim().parse::<u64>().unwrap_or(0), b.split(',').filter_map(|s| s.trim().parse::<f32>().ok()).collect::<Vec<_>>())) {
            if k == play.ticks && n.len() == 4 {
                play.diag_owned(n[0], n[1], n[2], n[3]);
            }
        }
        // NEOPETS_TRACE_RAILS=x,y,z: every ladder's grab distances from that point, on tick 10.
        if play.ticks == 10 {
            if let Some(at) = std::env::var("NEOPETS_TRACE_RAILS").ok().and_then(|v| { let n: Vec<f32> = v.split(',').filter_map(|s| s.trim().parse().ok()).collect(); (n.len() == 3).then(|| [n[0], n[1], n[2]]) }) {
                play.diag_rails(at);
            }
        }
        // NEOPETS_TRACE_EYE=n: the camera every n ticks.
        if std::env::var("NEOPETS_TRACE_EYE").ok().and_then(|v| v.trim().parse::<u64>().ok()).is_some_and(|n| play.ticks % n == 0) {
            let h = play.player.loco.position();
            println!("EYE {} pack {} hero {:.2},{:.2},{:.2} state {} facing {:.2},{:.2} eye {:.2},{:.2},{:.2} focus {:.2},{:.2},{:.2} polar {:?}", play.ticks, play.world.pack, h[0], h[1], h[2], play.player.loco.state, play.player.loco.facing[0], play.player.loco.facing[2], play.player.camera.eye[0], play.player.camera.eye[1], play.player.camera.eye[2], play.player.camera.focus[0], play.player.camera.focus[1], play.player.camera.focus[2], play.player.camera.current);
        }
        // NEOPETS_DEV_OBJEND="tick:a,b": as in the window, in the trace (diagnostic).
        if let Some((k, objs)) = std::env::var("NEOPETS_DEV_OBJEND").ok().as_deref().and_then(|v| v.split_once(':')).map(|(a, b)| (a.trim().parse::<u64>().unwrap_or(0), b.split(',').filter_map(|s| s.trim().parse::<usize>().ok()).collect::<Vec<_>>())) {
            if k == play.ticks {
                let acts: Vec<neopets_sim::level::Action> = objs.into_iter().map(|obj| neopets_sim::level::Action::InitObjOn { obj, end: true }).collect();
                play.apply(&acts);
            }
        }
        // NEOPETS_TRACE_OBJS="tick:a-b": placed objects a to b on that tick.
        if let Some((k, a, b)) = std::env::var("NEOPETS_TRACE_OBJS").ok().and_then(|v| { let (k, r) = v.split_once(':')?; let (a, b) = r.split_once('-')?; Some((k.trim().parse::<u64>().ok()?, a.trim().parse::<usize>().ok()?, b.trim().parse::<usize>().ok()?)) }) {
            if play.ticks == k {
                play.diag_objects(a, b);
                println!("OBJECT SOLIDS {}", play.debug_object_solids());
            }
        }
        // NEOPETS_TRACE_BREAKLIST=tick: every breakable's state on that tick.
        if std::env::var("NEOPETS_TRACE_BREAKLIST").ok().and_then(|v| v.trim().parse::<u64>().ok()) == Some(play.ticks) {
            play.diag_breakables();
        }
        // NEOPETS_TRACE_CATCH=tick: the hero stood beside the first Miamouse on that tick; its carry state every 60 ticks.
        if let Some(k) = std::env::var("NEOPETS_TRACE_CATCH").ok().and_then(|v| v.trim().parse::<u64>().ok()) {
            if play.ticks == k {
                play.diag_catch();
            }
            if play.ticks % 60 == 0 {
                println!("CATCH {}: {}", play.ticks, play.debug_carry());
            }
        }
        // NEOPETS_TRACE_SUN: the sun's direction whenever it or the world changes.
        if std::env::var_os("NEOPETS_TRACE_SUN").is_some() {
            let now = format!("world {} lights {} sun {:?}", play.world.pack, play.world.lights.len(), play.sun_direction());
            if last_sun != now {
                println!("SUN tick {}: {now}", play.ticks);
                last_sun = now;
            }
        }
        // NEOPETS_TRACE_SETFLAG="tick:flag[,tick:flag...]": a story flag set on those ticks.
        if let Ok(v) = std::env::var("NEOPETS_TRACE_SETFLAG") {
            for g in v.split(',') {
                let n: Vec<u64> = g.split(':').filter_map(|x| x.trim().parse().ok()).collect();
                if let [t, flag] = n[..] {
                    if t == play.ticks {
                        play.state.set(flag as u16, true);
                    }
                }
            }
        }
        // NEOPETS_TRACE_GIVE="tick:id:n[,tick:id:n...]": the party add on those ticks.
        if let Ok(v) = std::env::var("NEOPETS_TRACE_GIVE") {
            for g in v.split(',') {
                let n: Vec<i64> = g.split(':').filter_map(|x| x.trim().parse().ok()).collect();
                if let [t, id, count] = n[..] {
                    if t as u64 == play.ticks {
                        play.diag_give(id as u16, count as i32);
                    }
                }
            }
        }
        // NEOPETS_TRACE_USE="tick:id[,tick:id...]": the item use step on those ticks.
        if let Ok(v) = std::env::var("NEOPETS_TRACE_USE") {
            for g in v.split(',') {
                let n: Vec<u64> = g.split(':').filter_map(|x| x.trim().parse().ok()).collect();
                if let [t, id] = n[..] {
                    if t == play.ticks {
                        play.diag_use(id as u16);
                    }
                }
            }
        }
        // NEOPETS_TRACE_CASTERS: the characters casting a shadow on tick 1000.
        if std::env::var_os("NEOPETS_TRACE_CASTERS").is_some() && play.ticks == 1000 {
            println!("CASTERS {:?}", play.diag_casters());
        }
        // NEOPETS_TRACE_ROUTINES: every pack and entity with a talk routine (for a check against the specs).
        if std::env::var_os("NEOPETS_TRACE_ROUTINES").is_some() && play.ticks == 1 {
            for pack in 600..1500u32 {
                let v: Vec<usize> = (0..256).filter(|&e| neopets_sim::level::proximity_routine(pack, e).is_some()).collect();
                if !v.is_empty() {
                    println!("ROUTINES {pack} {v:?}");
                }
            }
        }
        // NEOPETS_TRACE_INITS: the triggers each pack's Init registers, with no flags and with all set.
        if std::env::var_os("NEOPETS_TRACE_INITS").is_some() && play.ticks == 1 {
            for pack in 600..1500u32 {
                let mut got = std::collections::BTreeSet::new();
                for all in [false, true] {
                    let mut st = neopets_sim::level::GameState::default();
                    if all {
                        for f in 0..800 {
                            st.set(f, true);
                        }
                    }
                    for a in neopets_sim::level::on_init(pack, &mut st) {
                        if let neopets_sim::level::Action::RegisterTrigger { index, .. } = a {
                            got.insert(index);
                        }
                    }
                }
                println!("INITS {pack} {got:?}");
            }
        }
        // NEOPETS_TRACE_STATUPS: the items the stat-up rule accepts.
        if std::env::var_os("NEOPETS_TRACE_STATUPS").is_some() && play.ticks == 1 {
            println!("STATUPS {:?}", play.stat_up_list());
        }
        // NEOPETS_TRACE_NPCWALK="n": the townsfolk walking audit's report on tick n.
        if std::env::var("NEOPETS_TRACE_NPCWALK").ok().and_then(|v| v.trim().parse::<u64>().ok()) == Some(play.ticks) {
            play.npc_audit_report();
        }
        // NEOPETS_TRACE_SPAWN="tick:region:ordinal;...": the developer menu's character spawn on those ticks.
        for e in std::env::var("NEOPETS_TRACE_SPAWN").unwrap_or_default().split(';') {
            let mut it = e.split(':').filter_map(|v| v.trim().parse::<u64>().ok());
            if let (Some(t), Some(r), Some(o)) = (it.next(), it.next(), it.next()) {
                if t == play.ticks {
                    play.dev_spawn(r as u32, o as usize);
                }
            }
        }
        // NEOPETS_TRACE_KILL="n[,n...]": the developer menu's enemy defeat on those ticks.
        // NEOPETS_DEV_KILL_ENTITY="tick:e,e,...": those entities of the current world killed on that tick (diagnostic).
        if let Some((k, list)) = std::env::var("NEOPETS_DEV_KILL_ENTITY").ok().as_deref().and_then(|v| v.split_once(':')).map(|(a, b)| (a.trim().parse::<u64>().unwrap_or(0), b.split(',').filter_map(|s| s.trim().parse::<usize>().ok()).collect::<Vec<_>>())) {
            if k == play.ticks {
                let (tick, n) = (play.ticks, play.dev_kill_entities(&list));
                println!("      developer kill of entities {list:?} on tick {tick}: {n} hit");
            }
        }
        if std::env::var("NEOPETS_TRACE_KILL").is_ok_and(|v| v.split(',').any(|t| t.trim().parse::<u64>().ok() == Some(play.ticks))) {
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
                let hero = play.player.loco.position();
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
        // NEOPETS_TRACE_LEDGES: on tick 5 the jump-to-ledge candidates of the world, and each
        // ledge-state tick (diagnostic).
        if play.ticks == 5 && std::env::var_os("NEOPETS_TRACE_LEDGES").is_some() {
            for line in play.ledge_survey() {
                println!("      {line}");
            }
            for (k, l) in play.world.grab_lines.iter().enumerate() {
                println!("      grab line {k}: A {:?} B {:?} L {} r {} mask {}", l.a, l.b(), l.length, l.radius, l.mask);
                let hs: Vec<String> = (0..=30).step_by(3).map(|s| {
                    let q: [f32; 3] = std::array::from_fn(|i| l.a[i] + s as f32 * l.u[i]);
                    let off = [-l.u[2], 0.0, l.u[0]];
                    let g = |d: f32| play.world.ground_at(q[0] + d * off[0], q[2] + d * off[2], q[1]).map_or("-".to_string(), |h| format!("{:.1}", q[1] - h));
                    format!("s{s}:{}/{}/{}", g(-3.0), g(0.0), g(3.0))
                }).collect();
                println!("        line height above ground (left/under/right): {}", hs.join(" "));
            }
            for (k, r) in play.world.rails.iter().enumerate() {
                println!("      rail {k}: normal {:?} u {:?} v {:?} lu {} lv {}", r.normal, r.u, r.v, r.lu, r.lv);
            }
        }
        if play.ticks == 5 && std::env::var_os("NEOPETS_TRACE_BLOCK").is_some() {
            for line in play.block_survey() {
                println!("      {line}");
            }
        }
        // NEOPETS_TRACE_BLOCK: each block's centre when a move ends, at full precision.
        if std::env::var_os("NEOPETS_TRACE_BLOCK").is_some() {
            let now: Vec<bool> = play.world.blocks.iter().map(|b| b.moving).collect();
            for (k, b) in play.world.blocks.iter().enumerate() {
                if moving_before.get(k) == Some(&true) && !b.moving {
                    println!("      block {k} rests at ({:.4}, {:.4}) on tick {}", b.position[0], b.position[2], play.ticks);
                }
            }
            moving_before = now;
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
            let hero = play.player.loco.position();
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
        if std::env::var_os("NEOPETS_TRACE_POOL").is_some() && play.ticks % 20 == 0 {
            let live: Vec<String> = play.pools.get(&play.world.pack).map(|p| p.slots.iter().flatten().map(|it| format!("{}:{:.1}/{:.1}", it.id, it.age, it.lifetime)).collect()).unwrap_or_default();
            println!("      pool on tick {} (hero state {} dialogue {}): {}", play.ticks, play.player.loco.state, play.dialogue_open, live.join(" "));
        }
        // NEOPETS_TRACE_DROP_ITEM=id picks the item; NEOPETS_TRACE_DROP takes a list of ticks.
        if std::env::var("NEOPETS_TRACE_DROP").unwrap_or_default().split(',').filter_map(|t| t.trim().parse::<u64>().ok()).any(|k| k == play.ticks) {
            let id = std::env::var("NEOPETS_TRACE_DROP_ITEM").ok().and_then(|v| v.parse().ok()).unwrap_or(160);
            let n = play.dev_drop(id, 6.0);
            println!("      developer drop on tick {}: {n} world items", play.ticks);
        }
        if quit_at.is_some_and(|k| k == play.ticks) || std::mem::take(&mut play.teardown_wanted) {
            quit_at = None;
            let held = |p: &play::Play| (p.world.pack, p.hero_index, p.player.hero_hp, p.state.flags.len(), p.state.neopoints, p.snapshot.is_some(), p.characters.len(), p.generation);
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
        if let Some(run) = story.as_mut() {
            story_order = run.step(&mut play);
            if story_order.finished {
                println!("STORY finished on tick {}, world {}", play.ticks, play.world.pack);
                break;
            }
            if let Some(pack) = story_order.warp {
                match warp(&mut archive.disc, &archive.index, &play, pack) {
                    Ok(new) => {
                        play = new;
                        if let Ok(second) = archive.disc.reopen() {
                            streamer = Streamer::start(second, archive.index.clone(), play.boot.clone());
                        }
                    }
                    Err(e) => println!("STORY warp to {pack} failed: {e}"),
                }
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
        // `--trace-to x,z` steers the stick toward a point through the camera's rows; without
        // one the stick is held forward at 0.7, or left centred with NEOPETS_TRACE_STAND.
        let (mut forward, mut side) = (if std::env::var_os("NEOPETS_TRACE_STAND").is_some() && !story_order.walk { 0.0 } else { 0.7 }, 0.0);
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
            let p = play.player.loco.position();
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
            let p = play.player.loco.position();
            (route[0][0] - p[0]).powi(2) + (route[0][1] - p[2]).powi(2) < 4.0
        } {
            route.remove(0);
        }
        // NEOPETS_TRACE_FOLLOW=<name>: keep within 4.5 units of the character with that name.
        if let Ok(name) = std::env::var("NEOPETS_TRACE_FOLLOW") {
            // The name "wild" follows the first wild petpet, to within 3 units.
            let wild = name == "wild";
            if let Some(c) = play.characters.iter().find(|c| (c.name == name || wild && c.wild.is_some()) && c.pack == play.world.pack) {
                let p = play.player.loco.position();
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
        // NEOPETS_TRACE_WARP="tick:x,z,y": the hero placed on the ground found below y at (x, z)
        // on that tick, facing kept (diagnostic: reaching ground that is drawn only later).
        if let Ok(v) = std::env::var("NEOPETS_TRACE_WARP") {
            if let Some((t, p)) = v.split_once(':') {
                let n: Vec<f32> = p.split(',').filter_map(|x| x.trim().parse().ok()).collect();
                if t.trim().parse::<u64>().ok() == Some(play.ticks) && n.len() == 3 {
                    let y = play.world.ground_at(n[0], n[1], n[2]).unwrap_or(n[2]);
                    let f = play.player.loco.facing;
                    play.player.loco.teleport([n[0], y, n[1]], f);
                    println!("      hero warped to ({:.2}, {y:.2}, {:.2}) on tick {}", n[0], n[1], play.ticks);
                }
            }
        }
        // NEOPETS_BOSS_HP="tick:hp;tick:hp": every boss of the current world set to that health on
        // that tick (diagnostic: reaching the health stages of a fight).
        if let Ok(v) = std::env::var("NEOPETS_BOSS_HP") {
            for e in v.split(';') {
                let Some((t, h)) = e.split_once(':') else { continue };
                if let (Ok(t), Ok(h)) = (t.trim().parse::<u64>(), h.trim().parse::<i32>()) {
                    if t == play.ticks {
                        let pack = play.world.pack;
                        for c in play.characters.iter_mut().filter(|c| c.pack == pack && play::Play::is_boss(c)) {
                            c.hp = h;
                        }
                        println!("      boss health set to {h} on tick {t}");
                    }
                }
            }
        }
        // NEOPETS_BOSS_DEFEAT="tick" (or "tick,every" to repeat every 300 ticks after it): the
        // developer menu's defeat of every enemy and boss on that tick; from then on, and with
        // NEOPETS_TRACE_SCENE=n every n ticks from the start, a line of what could hold the
        // player (diagnostic: soft locks after a fight).
        if let Ok(v) = std::env::var("NEOPETS_BOSS_DEFEAT") {
            let mut it = v.split(',');
            if let Some(k) = it.next().and_then(|t| t.trim().parse::<u64>().ok()) {
                let every = it.next().is_some_and(|t| t.trim() == "every");
                if play.ticks == k || (every && play.ticks > k && (play.ticks - k) % 300 == 0) {
                    let n = play.diag_defeat_bosses();
                    println!("      DEFEAT on tick {}: {n} acted on", play.ticks);
                }
                if play.ticks >= k && (play.ticks - k) % 120 == 0 {
                    println!("      SCENE {}", play.diag_scene_report());
                }
            }
        }
        // NEOPETS_TRACE_FACING: each character's heading (atan2(x, z), degrees) on ticks 1 and 120.
        if (play.ticks == 1 || play.ticks == 120) && std::env::var_os("NEOPETS_TRACE_FACING").is_some() {
            let h: Vec<String> = play.characters.iter().filter(|c| c.pack == play.world.pack).map(|c| format!("{} {:.0}", c.name, c.facing[0].atan2(c.facing[2]).to_degrees())).collect();
            println!("      FACING tick {}: {}", play.ticks, h.join(", "));
        }
        // NEOPETS_TRACE_BOX=n: the F6 boss box's lines every n ticks.
        if let Some(n) = std::env::var("NEOPETS_TRACE_BOX").ok().and_then(|v| v.trim().parse::<u64>().ok()).filter(|&n| n > 0) {
            if play.ticks % n == 0 {
                let lines = play.boss_lines();
                if !lines.is_empty() {
                    println!("      BOX tick {}: {}", play.ticks, lines.join(" | "));
                }
            }
        }
        if let Some(n) = std::env::var("NEOPETS_TRACE_SCENE").ok().and_then(|v| v.trim().parse::<u64>().ok()).filter(|&n| n > 0) {
            if play.ticks % n == 0 {
                println!("      SCENE {}", play.diag_scene_report());
            }
        }
        // NEOPETS_TRACE_DUEL="limit": the duel trace's plan replaces the fight goal and buttons.
        let duel_plan = play.duel_step().filter(|_| std::env::var_os("NEOPETS_TRACE_DUEL_PASSIVE").is_none());
        let fight = if play.fight_duel.is_some() { duel_plan.map(|p| (p.goal, p.close)) } else { to.is_some_and(|t| t[1] == 3.0).then(|| play.fight_goal()).flatten() };
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
            let p = play.player.loco.position();
            (route.len() > 1 && !walk_path) || to.is_some_and(|t| t[0].is_nan()) || (g[0] - p[0]).hypot(g[1] - p[2]) > if walk_path { 1.5 } else { 2.0 }
        }) {
            let p = play.player.loco.position();
            let (f, l) = play.player.camera.rows();
            let (dx, dz) = (tx - p[0], tz - p[2]);
            let len = (dx * dx + dz * dz).sqrt().max(1e-6);
            // Close, Roberta only turns: a slow step while her facing is off the target, none
            // once it is within about 8 degrees.
            let aligned = (dx * play.player.loco.facing[0] + dz * play.player.loco.facing[2]) / (dx * dx + dz * dz).sqrt().max(1e-6) > 0.99;
            let m = if fight.is_some_and(|f| f.1) && play.hero_index == 1 { if aligned { 0.0 } else { 0.15 } } else { 0.7 };
            forward = m * (dx * f[0] + dz * f[2]) / len / (f[0] * f[0] + f[2] * f[2]).sqrt().max(1e-6);
            side = m * (dx * l[0] + dz * l[2]) / len;
        }
        // NEOPETS_TRACE_PUSH="x,z": the stick points straight at the point (no route search),
        // except while climbing onto a block (centred) and on a ladder (pushed up).
        // NEOPETS_TRACE_PUSH_PACK=p: the push only while the hero is in pack p.
        let push_here = std::env::var("NEOPETS_TRACE_PUSH_PACK").ok().and_then(|v| v.trim().parse::<u32>().ok()).is_none_or(|k| k == play.world.pack);
        if let Some((x, z)) = std::env::var("NEOPETS_TRACE_PUSH").ok().filter(|_| push_here).and_then(|v| { let (a, b) = v.split_once(',')?; Some((a.trim().parse::<f32>().ok()?, b.trim().parse::<f32>().ok()?)) }) {
            (forward, side) = match play.player.loco.state {
                58 => (0.0, 0.0),
                41 => (1.0, 0.0),
                _ => play.stick_toward(x, z),
            };
        }
        // NEOPETS_TRACE_BLOCK: in the block state the stick points into the held face (centred
        // during the approach), so the hero pushes; the Square prompts are pressed as usual.
        if std::env::var_os("NEOPETS_TRACE_BLOCK").is_some() && play.player.loco.state == neopets_sim::hero::BLOCK {
            // NEOPETS_TRACE_BLOCK_MOVE="tick:t;tick:t": from each tick on the stick turns t
            // quarter turns from the held face (0 push, 2 pull, 1 and 3 the side moves; "-"
            // centred).
            let mut turn = Some(0);
            if let Ok(v) = std::env::var("NEOPETS_TRACE_BLOCK_MOVE") {
                for e in v.split(';') {
                    let Some((k, t)) = e.trim().split_once(':') else { continue };
                    if k.parse::<u64>().is_ok_and(|k| k <= play.ticks) {
                        turn = t.trim().parse::<i32>().ok();
                    }
                }
            }
            (forward, side) = turn.and_then(|t| play.block_trace_stick_toward(t)).unwrap_or((0.0, 0.0));
        }
        // NEOPETS_TRACE_BLOCK_AT="tick:x,z": block 0 moved to (x, z) (diagnostic).
        if let Some((k, x, z)) = std::env::var("NEOPETS_TRACE_BLOCK_AT").ok().and_then(|v| {
            let (k, r) = v.split_once(':')?;
            let (x, z) = r.split_once(',')?;
            Some((k.parse::<u64>().ok()?, x.parse::<f32>().ok()?, z.parse::<f32>().ok()?))
        }) {
            if k == play.ticks {
                println!("      block 0 moved to ({x}, {z}), top {:?}", play.diag_block_at(x, z));
            }
        }
        if play.ticks == 2 && std::env::var_os("NEOPETS_TRACE_GATED").is_some() {
            for line in play.gated_walls() {
                println!("      gated wall: {line}");
            }
        }
        // NEOPETS_TRACE_SHOWCASE="tick:id": the item showcase started (diagnostic).
        if let Some((k, id)) = std::env::var("NEOPETS_TRACE_SHOWCASE").ok().and_then(|v| {
            let (k, id) = v.split_once(':')?;
            Some((k.parse::<u64>().ok()?, id.parse::<u16>().ok()?))
        }) {
            if k == play.ticks {
                play.diag_showcase(id);
            }
        }
        // NEOPETS_TRACE_REPLACE="tick,slot": the exterior replacement load to eW slot on that
        // tick, as the arena master's yes asks for it (diagnostic).
        if let Some((t, sl)) = std::env::var("NEOPETS_TRACE_REPLACE").ok().and_then(|v| { let (a, b) = v.split_once(',')?; Some((a.trim().parse::<u64>().ok()?, b.trim().parse::<usize>().ok()?)) }) {
            if t == play.ticks {
                play.diag_replace_exterior(sl, 0);
            }
        }
        // NEOPETS_TRACE_LOS="tick,name": the world scene parts that cross the line from the
        // drawn camera's eye to the named character's chest on that tick (diagnostic).
        if let Some((t, name)) = std::env::var("NEOPETS_TRACE_LOS").ok().and_then(|v| v.split_once(',').map(|(a, b)| (a.trim().parse::<u64>().ok(), b.to_string()))) {
            if t == Some(play.ticks) {
                let (mut eye, fwd, up, focal) = play.view();
                // "@x,y,z,x,y,z": an explicit segment instead of the camera and a character.
                let seg: Vec<f32> = name.strip_prefix('@').map(|r| r.split(',').filter_map(|v| v.parse().ok()).collect()).unwrap_or_default();
                let target = if seg.len() == 6 {
                    eye = [seg[0], seg[1], seg[2]];
                    Some([seg[3], seg[4], seg[5]])
                } else {
                    play.characters.iter().find(|c| c.name == name).map(|c| [c.position[0], c.position[1] + 1.5, c.position[2]])
                };
                if let Some(to) = target {
                    let d = [to[0] - eye[0], to[1] - eye[1], to[2] - eye[2]];
                    let l = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt().max(1e-6);
                    let angle = ((d[0] * fwd[0] + d[1] * fwd[1] + d[2] * fwd[2]) / l).clamp(-1.0, 1.0).acos().to_degrees();
                    println!("      los tick {t:?}: eye {eye:?} forward {fwd:?} up {up:?} focal {focal} to {name} {to:?}, {angle:.0} degrees off the view axis; camera {}", play.status_camera());
                    // The drawn objects whose origin lies nearest the line (objects are not in the
                    // scene model).
                    let mut objs: Vec<(f32, f32, usize, i32, bool)> = (0..play.world.objects.len())
                        .filter_map(|i| {
                            let m = play.object_matrix(play.world.pack, i)?;
                            let o = [m[12], m[13], m[14]];
                            let v = [o[0] - eye[0], o[1] - eye[1], o[2] - eye[2]];
                            let s = ((v[0] * d[0] + v[1] * d[1] + v[2] * d[2]) / (l * l)).clamp(0.0, 1.0);
                            let q = [eye[0] + d[0] * s - o[0], eye[1] + d[1] * s - o[1], eye[2] + d[2] * s - o[2]];
                            let a = play.world.object_anims.get(i);
                            Some(((q[0] * q[0] + q[1] * q[1] + q[2] * q[2]).sqrt(), s * l, i, a.map_or(-1, |a| a.state as i32), a.is_some_and(|a| a.scripted)))
                        })
                        .collect();
                    objs.sort_by(|a, b| a.0.total_cmp(&b.0));
                    for (off, along, i, st, scripted) in objs.iter().take(12) {
                        println!("      los object {i}: {off:.1} from the line, {along:.1} along it, state {st}, scripted {scripted}");
                    }
                    let Archive { disc, index } = &mut *archive;
                    let bytes = index.read(disc, play.world.pack).unwrap_or_default();
                    if let Ok(chain) = neopets_data::resource::walk_chunks(&bytes, None) {
                        if let Ok(Some(m)) = archive_view::world_scene(disc, index, play.world.pack, &bytes, &chain.chunks, false, u32::MAX) {
                            let sub = |a: [f32; 3], b: [f32; 3]| [a[0] - b[0], a[1] - b[1], a[2] - b[2]];
                            let cross = |a: [f32; 3], b: [f32; 3]| [a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0]];
                            let dot = |a: [f32; 3], b: [f32; 3]| a[0] * b[0] + a[1] * b[1] + a[2] * b[2];
                            let d = sub(to, eye);
                            for (i, p) in m.parts.iter().enumerate() {
                                let mut hits = 0;
                                for t in p.indices.chunks_exact(3) {
                                    let (a, b, c) = (p.positions[t[0] as usize], p.positions[t[1] as usize], p.positions[t[2] as usize]);
                                    let (e1, e2) = (sub(b, a), sub(c, a));
                                    let h = cross(d, e2);
                                    let det = dot(e1, h);
                                    if det.abs() < 1e-8 {
                                        continue;
                                    }
                                    let s = sub(eye, a);
                                    let u = dot(s, h) / det;
                                    let q = cross(s, e1);
                                    let v = dot(d, q) / det;
                                    let w = dot(e2, q) / det;
                                    if (0.0..=1.0).contains(&u) && v >= 0.0 && u + v <= 1.0 && (0.0..=1.0).contains(&w) {
                                        hits += 1;
                                    }
                                }
                                if hits > 0 {
                                    let n = p.positions.len().max(1) as f32;
                                    let ce = p.positions.iter().fold([0.0f32; 3], |a, q| [a[0] + q[0] / n, a[1] + q[1] / n, a[2] + q[2] / n]);
                                    let alpha = p.colors.iter().map(|c| c[3]).fold(0.0f32, f32::max);
                                    println!("      los part {i}: {hits} hits, {} vertices, centre ({:.1}, {:.1}, {:.1}), texture {:?}, zone {:?}, env {}, cull {:?}, max vertex alpha {alpha:.2}", p.positions.len(), ce[0], ce[1], ce[2], p.texture, p.zone, p.env_map, p.cull);
                                }
                            }
                        }
                    }
                }
            }
        }
        // NEOPETS_TRACE_OBJON="tick,obj": InitObjOn for that object on that tick (diagnostic).
        if let Some((t, o)) = std::env::var("NEOPETS_TRACE_OBJON").ok().and_then(|v| { let (a, b) = v.split_once(',')?; Some((a.trim().parse::<u64>().ok()?, b.trim().parse::<usize>().ok()?)) }) {
            if t == play.ticks {
                play.diag_object_on(o);
            }
        }
        // NEOPETS_TRACE_OBJS="tick,a-b": objects a to b of the current world on that tick: drawn
        // matrix origin, animation state, and the probed ground from 10 above (diagnostic).
        if let Some((t, a, b)) = std::env::var("NEOPETS_TRACE_OBJS").ok().and_then(|v| {
            let (t, r) = v.split_once(',')?;
            let (a, b) = r.split_once('-')?;
            Some((t.trim().parse::<u64>().ok()?, a.trim().parse::<usize>().ok()?, b.trim().parse::<usize>().ok()?))
        }) {
            // Tick 0: every 10 ticks, with the hero's position.
            if t == 0 && play.ticks % 10 == 0 {
                let h = play.player.loco.position();
                println!("      hero tick {}: ({:.2}, {:.2}, {:.2}) ground under {:?}", play.ticks, h[0], h[1], h[2], play.world.ground_at(h[0], h[2], h[1] + 3.0));
            }
            if t == play.ticks {
                let sets = play.world.object_sets();
                for i in a..=b {
                    let m = play.object_matrix(play.world.pack, i);
                    let st = play.world.object_anims.get(i).map(|o| (o.state, o.scripted, o.drawn));
                    let set = sets.iter().find(|(_, o)| *o == i).map(|(id, _)| *id);
                    let g = m.and_then(|m| play.world.ground_at(m[12], m[14], m[13] + 10.0));
                    let walls = m.map_or(0, |m| play.world.object_wall_count(m[12], m[14]));
                    println!("      obj {i}: drawn at {:?}, anim {st:?}, patch set {set:?}, owner pose {}, ground from 10 above {g:?}, object walls in its cell {walls}", m.map(|m| [m[12], m[13], m[14]]), play.world.owner_poses.contains_key(&i));
                }
            }
        }
        // NEOPETS_TRACE_PROBE="tick,x0,z0,x1,z1,step,ceiling": the ground probe on a grid of the
        // current world on that tick: each point's height (or "-" for none) (diagnostic).
        if let Some(v) = std::env::var("NEOPETS_TRACE_PROBE").ok().map(|v| v.split(',').filter_map(|t| t.trim().parse::<f32>().ok()).collect::<Vec<f32>>()).filter(|v| v.len() == 7) {
            if v[0] as u64 == play.ticks {
                let mut z = v[2];
                while z <= v[4] {
                    let mut row = format!("      probe z {z:7.1}:");
                    let mut x = v[1];
                    while x <= v[3] {
                        let h = play.world.ground_at(x, z, v[6]);
                        row.push_str(&h.map_or("      -".to_string(), |h| format!(" {h:6.1}")));
                        x += v[5];
                    }
                    println!("{row}");
                    z += v[5];
                }
            }
        }
        // NEOPETS_TRACE_BONES="tick,name": that character's posed bones (model space) and its
        // bind pose, on that tick (diagnostic).
        if let Some((t, name)) = std::env::var("NEOPETS_TRACE_BONES").ok().and_then(|v| v.split_once(',').map(|(a, b)| (a.trim().parse::<u64>().ok(), b.trim().to_string()))) {
            if t == Some(play.ticks) {
                if let Some(c) = play.characters.iter().find(|c| c.name == name) {
                    let st = c.anim.clip_state();
                    println!("      bones of {name}: action {} progress {:.2}", st.action, st.progress);
                    for (k, f) in c.anim.bone_frames().iter().enumerate() {
                        let rest = c.anim.bones().rest_position.get(k).copied();
                        println!("        bone {k}: parent {:?} posed {:?} rest {:?}", c.anim.bones().parents.get(k), f.t.map(|v| (v * 100.0).round() / 100.0), rest.map(|r| r.map(|v| (v * 100.0).round() / 100.0)));
                    }
                }
            }
        }
        // NEOPETS_TRACE_FACING=name: that character's position, facing angle and clip on every tick (diagnostic).
        if let Ok(name) = std::env::var("NEOPETS_TRACE_FACING") {
            for c in play.characters.iter().filter(|c| c.name.contains(&name)) {
                let clip = c.anim.clip_state();
                println!("      FACING {} {:.2} {:.2} {:.1} clip {} world {} current {}", play.ticks, c.position[0], c.position[2], c.facing[2].atan2(c.facing[0]).to_degrees(), clip.action, c.pack, play.world.pack);
                if play.ticks % 200 == 0 {
                    let to = c.townsfolk.as_ref().and_then(|t| t.walk_to);
                    let wk = c.walker.as_ref();
                    let m = wk.and_then(|w| w.mesh).and_then(|i| play.world.navmeshes.get(i));
                    println!("      ROUTE {:?} code {:?}", c.townsfolk.as_ref().and_then(|t| t.patrol.as_ref().map(|p| p.points().to_vec())), c.townsfolk.as_ref().map(|t| t.code));
                    println!("      WALKSTATE {} walk_to {to:?} mesh {:?} arrived {:?} blocked {:?} target in mesh {:?} here in mesh {:?}", play.ticks, wk.map(|w| w.mesh), wk.map(|w| w.arrived), wk.map(|w| w.blocked), to.zip(m).map(|(t, m)| m.locate_full(t)), m.map(|m| m.locate_full(c.position)));
                }
            }
        }
        // NEOPETS_TRACE_CHARS=tick: each character's name, position and activity (diagnostic).
        if std::env::var("NEOPETS_TRACE_CHARS").ok().and_then(|v| v.parse::<u64>().ok()) == Some(play.ticks) {
            for c in &play.characters {
                let world = [&play.world].into_iter().chain(play.other.as_ref()).find(|w| w.pack == c.pack);
                let ground = world.and_then(|w| w.ground_at(c.position[0], c.position[2], c.position[1] + 5.0));
                println!("      char {} {:?} entity {:?} at ({:.1}, {:.1}, {:.1}) inactive {} deactivated {} enemy {} ground {:?} fly {}", c.name, c.pack, c.entity, c.position[0], c.position[1], c.position[2], c.inactive, c.deactivated, c.enemy.is_some(), ground.map(|g| (g * 10.0).round() / 10.0), c.row.map_or(0, |r| r.can_fly));
            }
        }
        // NEOPETS_TRACE_REPORT=n (diagnostic): every n ticks, the text a bug report writes (not its
        // shadow part), built and dropped, so a panic in it shows in a trace.
        if let Some(n) = std::env::var("NEOPETS_TRACE_REPORT").ok().and_then(|v| v.parse::<u64>().ok()).filter(|&n| n > 0) {
            if play.ticks % n == 0 {
                let mut len = 0;
                for part in [play.debug_inspect(), play.scene_status(), play.status(), play.objects_near(), play.placed_summary(), play.debug_nearby(100.0)] {
                    len += part.len();
                }
                let (a, b) = play.recorder.export();
                len += a.len() + b.len() + play.recorder.history_csv().len();
                println!("      report text tick {}: {len} bytes", play.ticks);
            }
        }
        // NEOPETS_TRACE_BLOCKERS=tick: what pushes the hero on that tick (diagnostic).
        if std::env::var("NEOPETS_TRACE_BLOCKERS").ok().and_then(|v| v.parse::<u64>().ok()) == Some(play.ticks) {
            for line in play.blockers() {
                println!("      blocker: {line}");
            }
        }
        // NEOPETS_TRACE_LADDER="tick:k:up": the hero put on ladder k, `up` above its base
        // (diagnostic).
        if let Ok(v) = std::env::var("NEOPETS_TRACE_LADDER") {
            let f: Vec<&str> = v.split(':').collect();
            if f.len() == 3 && f[0].parse::<u64>().ok() == Some(play.ticks) {
                play.diag_ladder(f[1].parse().unwrap_or(0), f[2].parse().unwrap_or(0.0));
            }
        }
        // NEOPETS_TRACE_WALKERS=1: the characters on a scripted walk with their clips, every 15 ticks (diagnostic).
        if std::env::var_os("NEOPETS_TRACE_WALKERS").is_some() && play.ticks % 15 == 0 && !play.walkers().is_empty() {
            println!("      WALKERS tick {}: {:?}", play.ticks, play.walkers());
        }
        // NEOPETS_TRACE_SPRITES=tick: the live sprite particles of the effects on that tick (diagnostic).
        if std::env::var("NEOPETS_TRACE_SPRITES").ok().and_then(|v| v.parse::<u64>().ok()) == Some(play.ticks) {
            for (p, group) in play.effects.sprites(&play.effect_table).take(24) {
                println!("      SPRITE group {group} at {:?} spin {:.3} half {:?} node {}", p.position, p.spin, p.half, p.node);
            }
        }
        // NEOPETS_TRACE_LINES="a,b": those lines of the current world's table with their control bytes (diagnostic).
        if play.ticks == 2 {
            if let Ok(v) = std::env::var("NEOPETS_TRACE_LINES") {
                for n in v.split(',').filter_map(|t| t.trim().parse::<usize>().ok()) {
                    println!("      LINE {n}: {:?} gives {}", play.world.lines.line(n).map(|l| l.text.clone()), play.world.lines.line(n).is_some_and(|l| l.raw.windows(4).any(|w| w[0] == 1 && w[3] == 0x13)));
                }
            }
        }
        // NEOPETS_TRACE_SAY="tick:line": that line of the current world opens on that tick (diagnostic).
        if let Some((k, n)) = std::env::var("NEOPETS_TRACE_SAY").ok().as_deref().and_then(|v| v.split_once(':')).and_then(|(a, b)| Some((a.parse::<u64>().ok()?, b.parse::<u16>().ok()?))) {
            if k == play.ticks {
                play.diag_say(n);
            }
        }
        // NEOPETS_TRACE_HELD=id: each change in how many of that item the hero holds (diagnostic).
        if let Some(id) = std::env::var("NEOPETS_TRACE_HELD").ok().and_then(|v| v.trim().parse::<u16>().ok()) {
            static LAST: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
            let now = play.state.items.get(&id).copied().unwrap_or(0);
            if LAST.swap(now, std::sync::atomic::Ordering::Relaxed) != now {
                println!("      HELD {id}: {now} on tick {} in {}", play.ticks, play.world.pack);
            }
        }
        // NEOPETS_TRACE_PLACE="tick:x,z[,probe[,y]];...": the hero placed there on that tick
        // (diagnostic).
        if let Ok(v) = std::env::var("NEOPETS_TRACE_PLACE") {
            for e in v.split(';') {
                let Some((k, r)) = e.trim().split_once(':') else { continue };
                let n: Vec<f32> = r.split(',').filter_map(|t| t.trim().parse().ok()).collect();
                if k.parse::<u64>().ok() == Some(play.ticks) && n.len() >= 2 {
                    if n.len() >= 4 {
                        // A fourth value: the height itself, with no probe (in the air).
                        play.place_exact([n[0], n[3], n[1]]);
                    } else {
                            play.place_at(n[0], n[1], n.get(2).copied().unwrap_or(1.0e4));
                    }
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
        // NEOPETS_TRACE_FACE_NAME="name@n": from tick n the hero is turned each tick to face the
        // nearest character of that name in the current world (diagnostic).
        if let Some(name) = std::env::var("NEOPETS_TRACE_FACE_NAME").ok().and_then(|v| {
            let (name, from) = v.rsplit_once('@').unwrap_or((v.as_str(), "0"));
            (from.trim().parse::<u64>().ok()? <= play.ticks).then(|| name.to_string())
        }) {
            let h = play.player.loco.position();
            let pack = play.world.pack;
            if let Some(c) = play.characters.iter().filter(|c| c.pack == pack && c.name == name && !c.inactive).min_by(|a, b| {
                let d = |c: &&play::Character| (c.position[0] - h[0]).hypot(c.position[2] - h[2]);
                d(a).total_cmp(&d(b))
            }) {
                let (dx, dz) = (c.position[0] - h[0], c.position[2] - h[2]);
                let l = dx.hypot(dz).max(1e-6);
                play.player.loco.facing = [dx / l, 0.0, dz / l];
            }
        }
        // NEOPETS_TRACE_FAERIE_DRIVE: when the Darkest Faerie's absorb window starts (code 7) the
        // hero is put 10 units west of her, and when she has landed (code 10) on her spot
        // (diagnostic; once per entry of each code).
        if std::env::var_os("NEOPETS_TRACE_FAERIE_DRIVE").is_some() {
            let pack = play.world.pack;
            let f = play.characters.iter().find(|c| c.pack == pack && c.faerie.is_some()).map(|c| (c.position, c.faerie.as_ref().map_or(0, |f| f.code)));
            if let Some((at, code)) = f {
                if code != faerie_code {
                    match code {
                        7 => play.place_at(at[0] - 10.0, at[2], at[1] + 6.0),
                        10 => play.place_at(at[0], at[2] + 2.0, at[1] + 6.0),
                        _ => {}
                    }
                    faerie_code = code;
                }
            }
        }
        // NEOPETS_TRACE_SISTERS: the Faerie Sisters' places every 150 ticks (diagnostic).
        if std::env::var_os("NEOPETS_TRACE_SISTERS").is_some() && play.ticks % 150 == 0 {
            println!("      SISTERS tick {}: {}", play.ticks, play.sister_places());
        }
        // NEOPETS_TRACE_FRAMES="tick:x,z,r": the frame records of the cell at (x, z) within r of it, with
        // their type, gate, start, face, top, length and height (diagnostic).
        if let Some((k, v)) = std::env::var("NEOPETS_TRACE_FRAMES").ok().as_deref().and_then(|v| v.split_once(':')).map(|(a, b)| (a.trim().parse::<u64>().unwrap_or(0), b.split(',').filter_map(|s| s.trim().parse::<f32>().ok()).collect::<Vec<_>>())) {
            if k == play.ticks && v.len() == 3 {
                for line in play.frames_near(v[0], v[1], v[2]) {
                    println!("      {line}");
                }
            }
        }
        // NEOPETS_TRACE_COLLIS="tick:mask": SetWorldCollisMask(mask) on that tick (diagnostic).
        if let Some((k, m)) = std::env::var("NEOPETS_TRACE_COLLIS").ok().as_deref().and_then(|v| v.split_once(':')).and_then(|(a, b)| Some((a.trim().parse::<u64>().ok()?, b.trim().parse::<u16>().ok()?))) {
            if play.ticks == k {
                play.world.set_collis_mask(m);
            }
        }
        // NEOPETS_TRACE_STATE="save:name@t,load:id@t": a save state named `name`, or the load of
        // state `id`, on those ticks (diagnostic; use NEOPETS_STATES_DIR to keep the player's).
        if let Ok(v) = std::env::var("NEOPETS_TRACE_STATE") {
            for e in v.split(',') {
                let Some((what, k)) = e.trim().split_once('@') else { continue };
                if k.trim().parse::<u64>().ok() != Some(play.ticks) {
                    continue;
                }
                if let Some(name) = what.strip_prefix("save:") {
                    println!("      save state on tick {}: ready {} {:?} at {:?}", play.ticks, play.state_ready(), save_state(&play, name), play.player.loco.position());
                } else if let Some(r) = what.strip_prefix("load:").and_then(|id| load_state(&mut archive.disc, &archive.index, &play, id)) {
                    match r {
                        Ok(p) => {
                            play = p;
                            println!("      state load on tick {}: world {} hero at {:?} hp {}/{}", play.ticks, play.world.pack, play.player.loco.position(), play.player.hero_hp, play.player.hero_max);
                        }
                        Err(e) => println!("      state load failed: {e}"),
                    }
                }
            }
        }
        // NEOPETS_TRACE_ATMOS: the atmosphere zone output every 120 ticks (diagnostic).
        if std::env::var_os("NEOPETS_TRACE_ATMOS").is_some() && play.ticks % 120 == 0 {
            println!("      ATMOS out tick {}: {:?} ({} zones)", play.ticks, play.atmos_out, play.world.atmos.len());
        }
        // NEOPETS_TRACE_FACE="tick:x,z;...": the hero turned to face (x, z) on that tick
        // (diagnostic).
        if let Ok(v) = std::env::var("NEOPETS_TRACE_FACE") {
            for e in v.split(';') {
                let Some((t, p)) = e.split_once(':') else { continue };
                let n: Vec<f32> = p.split(',').filter_map(|t| t.trim().parse().ok()).collect();
                if t.trim().parse::<u64>().ok() == Some(play.ticks) && n.len() >= 2 {
                    let h = play.player.loco.position();
                    let (dx, dz) = (n[0] - h[0], n[1] - h[2]);
                    let l = dx.hypot(dz).max(1e-6);
                    play.player.loco.facing = [dx / l, 0.0, dz / l];
                }
            }
        }
        // NEOPETS_TRACE_MOTE_AT="tick:word:id;...": a held Mote equipped into Tor's equipment
        // word on that tick (diagnostic).
        if let Ok(v) = std::env::var("NEOPETS_TRACE_MOTE_AT") {
            for e in v.split(';') {
                let n: Vec<u64> = e.split(':').filter_map(|t| t.trim().parse().ok()).collect();
                if let [t, word, id] = n[..] {
                    if t == play.ticks {
                        play.item_command(&menus::Command::EquipMote { word: word as usize, id: id as u16 });
                    }
                }
            }
        }
        // NEOPETS_TRACE_SMASH="tick:k;...": breakable k of the current world broken on that tick
        // (its drop runs; diagnostic).
        if let Ok(v) = std::env::var("NEOPETS_TRACE_SMASH") {
            for (t, k) in v.split(';').filter_map(|e| e.split_once(':')).filter_map(|(a, b)| Some((a.trim().parse::<u64>().ok()?, b.trim().parse::<usize>().ok()?))) {
                if t == play.ticks {
                    play.diag_break(k);
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
        // NEOPETS_TRACE_TOWARD="tick:x,z": from that tick the stick points at (x, z), through the camera (diagnostic).
        if let Some((k, x, z)) = std::env::var("NEOPETS_TRACE_TOWARD").ok().as_deref().and_then(|v| {
            let (k, p) = v.split_once(':')?;
            let (x, z) = p.split_once(',')?;
            Some((k.trim().parse::<u64>().ok()?, x.trim().parse::<f32>().ok()?, z.trim().parse::<f32>().ok()?))
        }) {
            if play.ticks >= k {
                let h = play.player.loco.position();
                let (e, c) = (play.player.camera.eye, play.player.camera.focus);
                let (fx, fz) = (c[0] - e[0], c[2] - e[2]);
                let l = fx.hypot(fz).max(1e-6);
                let (fx, fz) = (fx / l, fz / l);
                let (dx, dz) = (x - h[0], z - h[2]);
                let d = dx.hypot(dz).max(1e-6);
                (forward, side) = ((dx * fx + dz * fz) / d, (dx * fz - dz * fx) / d);
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
            // NEOPETS_TRACE_L2=n: L2 held from tick n (diagnostic).
            l2: std::env::var("NEOPETS_TRACE_L2").ok().and_then(|v| v.trim().parse::<u64>().ok()).is_some_and(|n| play.ticks >= n),
            // An open dialogue box gets a Square press every 10 ticks (skip, Next, Finish).
            // With NEOPETS_TRACE_MICE the drop prompt is pressed only inside the cage, and a
            // stalk starts the sneak with one press.
            square_held: stalk.is_some(),
            // NEOPETS_TRACE_CLIMB: the grab-block (10) and stop-climbing (18) prompts are not pressed, so the hero keeps
            // walking into the block.
            // NEOPETS_TRACE_NOAUTO: prompts are not pressed (only NEOPETS_TRACE_SQUARE presses).
            square: (std::env::var_os("NEOPETS_TRACE_NOAUTO").is_none() && !story_order.quiet && play.prompt_shown.is_some() && !(mice && play.prompt_shown == Some(28) && !play.hero_in_cage()) && !(matches!(play.prompt_shown, Some(10 | 18)) && std::env::var_os("NEOPETS_TRACE_CLIMB").is_some()) && !(play.player.loco.state == neopets_sim::hero::BLOCK && std::env::var_os("NEOPETS_TRACE_BLOCK").is_some()))
                || (stalk.is_some() && !matches!(play.player.loco.state, 22 | 40))
                || play.talk_prompt || (play.buyable.is_some() && play.ticks % 30 == 0) || (play.save_prompt && play.ticks % 10 == 0) || (play.dialogue_open && play.ticks % 10 == 0 && std::env::var_os("NEOPETS_COOP_BOX").is_none()),
            // NEOPETS_TRACE_GUARD=n (or "n-m"): the guard is held on every tick from tick n (up to
            // tick m) (diagnostic).
            // NEOPETS_TRACE_GUARD_ORB: the guard is held while one of the Darkest Faerie's
            // projectiles is within 30 units of the hero.
            guard: (guard && fight.is_some_and(|f| f.1))
                || std::env::var("NEOPETS_TRACE_GUARD").ok().is_some_and(|v| {
                    let (a, b) = v.split_once('-').unwrap_or((v.as_str(), ""));
                    a.trim().parse::<u64>().is_ok_and(|n| play.ticks >= n) && b.trim().parse::<u64>().map_or(true, |m| play.ticks <= m)
                })
                || (std::env::var_os("NEOPETS_TRACE_GUARD_ORB").is_some() && {
                    let h = play.player.loco.position();
                    play.characters.iter().filter_map(|c| c.faerie.as_ref()).flat_map(|f| f.slots.iter().flatten()).any(|s| (0..3).map(|k| (s.position[k] - h[k]).powi(2)).sum::<f32>() < 900.0)
                }),
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
        if play.autopilot.is_some() {
            play.autopilot_input(&mut input);
            if play.ticks % 300 == 0 {
                if let Some(l) = play.autopilot_line() {
                    println!("      AUTO tick {} world {}: {l}", play.ticks, play.world.pack);
                }
            }
        }
        if let Some(p) = duel_plan {
            input.guard = p.guard;
            input.attack = p.attack;
            input.attack_held = p.attack_held;
            input.jump = p.jump || input.jump;
            input.run = p.run;
        }
        if std::env::var("NEOPETS_TRACE_SQUARE").is_ok_and(|v| v.split(',').any(|t| t.trim().parse::<u64>().ok() == Some(play.ticks))) {
            input.square = true;
        }
        // NEOPETS_TRACE_ATTACK=t,t,...: the attack pressed on those ticks (diagnostic).
        if std::env::var("NEOPETS_TRACE_ATTACK").is_ok_and(|v| v.split(',').any(|t| t.trim().parse::<u64>().ok() == Some(play.ticks))) {
            input.attack = true;
        }
        input.square |= story_order.square;
        input.jump |= story_order.jump;
        input.attack |= story_order.attack;
        // In a fight below half health the trace drinks: it opens the quick panel, selects the
        // first entry it owns and presses Cross (or closes the panel with Triangle).
        let mut input = input;
        if fight.is_some() && play.player.hero_hp > 0 && play.player.hero_hp < play.player.hero_max / 2 {
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
        // NEOPETS_TRACE_ROPE="k@t": from tick t the aim scan looks straight at aim target k, the
        // right stick is pushed up on tick t (aim mode) and R2 is held from t + 5 to t + 15.
        let mut pad = neopets_sim::camera::CameraPad::default();
        if let Some((k, t)) = std::env::var("NEOPETS_TRACE_ROPE").ok().and_then(|v| {
            let (k, t) = v.split_once('@')?;
            Some((k.trim().parse::<usize>().ok()?, t.trim().parse::<u64>().ok()?))
        }) {
            if play.ticks >= t {
                play.rope_aim = Some(k);
            }
            if play.ticks == t {
                pad.right_y = 0;
            }
            input.r2_held = (t + 5..t + 15).contains(&play.ticks);
        }
        // NEOPETS_REPLAY=dir: a bug report's start state, then its input, tick by tick (debug_tools; close, not exact: enemies
        // and scripts are not in the snapshot).
        if let Some(r) = replay.as_mut() {
            r.apply(&mut play, &mut input);
        }
        play.tick(&input, pad);
        // NEOPETS_COOP=1 (diagnostic): player 2 joins on tick 1 and walks ahead, jumping every 90
        // ticks; its position is printed every 30 ticks (docs/MULTIPLAYER.md step 2).
        if std::env::var_os("NEOPETS_COOP").is_some() {
            if play.ticks == 1 {
                // NEOPETS_COOP_NOREVIVE: the Multiplayer page's "Game over" (diagnostic).
                play.coop_revive = std::env::var_os("NEOPETS_COOP_NOREVIVE").is_none();
                // NEOPETS_COOP_TOR: player 2 is a second Tor (the Multiplayer page's choice).
                if std::env::var_os("NEOPETS_COOP_TOR").is_some() {
                    play.coop_partner = false;
                }
                println!("COOP player 2 joined: {}", play.add_player());
                // NEOPETS_COOP_P1HP=n: player 1 starts with that much health (diagnostic).
                if let Some(h) = std::env::var("NEOPETS_COOP_P1HP").ok().and_then(|v| v.parse::<i32>().ok()) {
                    play.player.hero_hp = h;
                }
                // NEOPETS_COOP_HP=n: player 2 starts with that much health (diagnostic).
                if let (Some(h), Some(o)) = (std::env::var("NEOPETS_COOP_HP").ok().and_then(|v| v.parse::<i32>().ok()), play.others.first_mut()) {
                    o.hero_hp = h;
                    // 0: player 2 starts down.
                    if h <= 0 {
                        o.down = Some(0);
                        o.loco.state = neopets_sim::hero::DEAD;
                    }
                }
            }
            // NEOPETS_COOP_ATTACK: player 2 stands and attacks every 40 ticks instead (with
            // NEOPETS_TRACE_GOD player 1 is not hit).
            // NEOPETS_COOP_DROP=tick: the developer coin drop 6 units ahead of player 2 on that tick.
            if std::env::var("NEOPETS_COOP_DROP").ok().and_then(|v| v.parse::<u64>().ok()) == Some(play.ticks) && !play.others.is_empty() {
                play.swap_player(0);
                let n = play.dev_drop(160, 6.0);
                play.swap_player(0);
                println!("COOP drop ahead of player 2 on tick {}: {n} world items, NP {}", play.ticks, play.state.neopoints);
            }
            // NEOPETS_COOP_STILL: player 2 stands still.
            // NEOPETS_COOP_WALKFROM=n: player 2 stands until tick n, then walks.
            let wait = std::env::var("NEOPETS_COOP_WALKFROM").ok().and_then(|v| v.parse::<u64>().ok()).is_some_and(|n| play.ticks < n);
            let p2 = if wait || std::env::var_os("NEOPETS_COOP_STILL").is_some() {
                neopets_sim::hero::HeroInput::default()
            } else if let Some(t) = std::env::var("NEOPETS_COOP_POWER").ok().and_then(|v| v.trim().parse::<u64>().ok()) {
                // NEOPETS_COOP_POWER=t: player 2 presses attack on tick t and holds it 80 ticks
                // (a power attack with a Mote in word 2).
                let held = play.ticks >= t && play.ticks < t + 80;
                if play.ticks % 10 == 0 {
                    println!("COOP {} power: p2 attack {:?} mp {:?} | {}", play.ticks, play.others.first().and_then(|o| o.attack.as_ref().map(|a| (a.sub, a.f))), play.others.first().map(|o| o.hero_mp), play.last_mote);
                }
                neopets_sim::hero::HeroInput { attack: play.ticks == t, attack_held: held, ..Default::default() }
            } else if std::env::var("NEOPETS_COOP_CLIMB").is_ok_and(|v| v.trim().parse::<u64>().map_or(true, |n| n <= 1 || play.ticks < n)) {
                // NEOPETS_COOP_CLIMB: player 2 pushes the stick forward (no jumps); "=n" (above 1)
                // lets go from tick n.
                neopets_sim::hero::HeroInput { forward: 1.0, ..Default::default() }
            } else if std::env::var_os("NEOPETS_COOP_GUARD").is_some() {
                // NEOPETS_COOP_GUARD: player 2 holds the guard button.
                neopets_sim::hero::HeroInput { guard: true, ..Default::default() }
            } else if std::env::var_os("NEOPETS_COOP_ATTACK").is_some() {
                // NEOPETS_COOP_TURN=n: first n ticks pushing the stick back (turning player 2 round).
                let turn = std::env::var("NEOPETS_COOP_TURN").ok().and_then(|v| v.parse::<u64>().ok()).unwrap_or(0);
                if play.ticks <= turn {
                    neopets_sim::hero::HeroInput { forward: -1.0, ..Default::default() }
                } else {
                    neopets_sim::hero::HeroInput { attack: play.ticks % 40 == 20, attack_held: play.ticks % 40 >= 20 && play.ticks % 40 < 26, ..Default::default() }
                }
            } else {
                neopets_sim::hero::HeroInput { forward: 1.0, jump: play.ticks % 90 == 45, ..Default::default() }
            };
            let mut p2 = p2;
            // NEOPETS_COOP_SWITCH=t: on tick t the story switches player 1 to the other hero; the
            // heroes of both players are printed every 30 ticks after.
            if let Some(t) = std::env::var("NEOPETS_COOP_SWITCH").ok().and_then(|v| v.trim().parse::<u64>().ok()) {
                if play.ticks == t {
                    play.diag_story_switch();
                    println!("COOP switch requested on tick {t}: {}", play.last_swap);
                }
                if play.ticks > t && play.ticks % 30 == 0 {
                    println!("COOP {} heroes: player 1 hero {}, player 2 hero {:?} hp {:?}, swap running {}", play.ticks, play.hero_index, play.others.first().map(|o| o.who), play.others.first().map(|o| (o.hero_hp, o.hero_max)), play.swap.is_some());
                }
            }
            // NEOPETS_COOP_AT="x,z,y@t": player 2 put there on tick t, facing as player 1, and its
            // state printed every 10 ticks after.
            if let Some((at, t)) = std::env::var("NEOPETS_COOP_AT").ok().and_then(|v| {
                let (a, t) = v.split_once('@')?;
                let n: Vec<f32> = a.split(',').filter_map(|x| x.trim().parse().ok()).collect();
                Some(([*n.first()?, *n.get(2)?, *n.get(1)?], t.trim().parse::<u64>().ok()?))
            }) {
                let f = play.player.loco.facing;
                if let Some(o) = play.others.first_mut() {
                    if play.ticks == t {
                        o.loco.teleport(at, f);
                    }
                    if play.ticks > t && play.ticks % 10 == 0 {
                        let q = o.loco.position();
                        println!("COOP {} p2 at ({:.2}, {:.2}, {:.2}) state {} ladder {} ledge {} hang {}", play.ticks, q[0], q[1], q[2], o.loco.state, o.ladder.is_some(), o.ledge.is_some(), o.hang.is_some());
                    }
                }
            }
            // NEOPETS_COOP_CHEST="g@t": on tick t player 2 is put at script group g's locator 0
            // (a chest's) with player 1 twenty units back, and presses Square every 20 ticks
            // after; held items and the group's state every 20 ticks.
            if let Some((g, t)) = std::env::var("NEOPETS_COOP_CHEST").ok().as_deref().and_then(|v| v.split_once('@')).and_then(|(a, b)| Some((a.trim().parse::<usize>().ok()?, b.trim().parse::<u64>().ok()?))) {
                if play.ticks == t {
                    if let Some((at, d)) = play.diag_group_spot(g) {
                        play.player.loco.teleport([at[0] - d[0] * 20.0, at[1] + 1.0, at[2] - d[2] * 20.0], d);
                        if let Some(o) = play.others.first_mut() {
                            o.loco.teleport([at[0], at[1] + 0.5, at[2]], d);
                        }
                        println!("COOP chest: player 2 put at group {g}'s locator {at:?}");
                    }
                }
                if play.ticks > t && play.ticks % 20 == 0 {
                    p2.square = true;
                    let mut held: Vec<(u16, u32)> = play.state.items.iter().map(|(k, v)| (*k, *v)).collect();
                    held.sort_unstable();
                    println!("COOP {} chest: p2 caption {:?} items {:?} {}", play.ticks, play.others.first().map(|o| o.captions.0), held, play.last_door);
                }
            }
            // NEOPETS_COOP_MENUUSE="t:id": on tick t player 2's item screen uses item id (the
            // item screen's command, run as the menu's player 2), and both players' health then.
            if let Some((t, id)) = std::env::var("NEOPETS_COOP_MENUUSE").ok().as_deref().and_then(|v| v.split_once(':')).and_then(|(a, b)| Some((a.trim().parse::<u64>().ok()?, b.trim().parse::<u16>().ok()?))) {
                if play.ticks == t {
                    play.menu_player = 1;
                    play.as_menu_player(|p| p.item_command(&menus::Command::UseItem(id)));
                    play.menu_player = 0;
                    println!("COOPMENU {}: player 2's item screen used {id}: p1 hp {} p2 hp {:?}", play.ticks, play.player.hero_hp, play.others.first().map(|o| o.hero_hp));
                }
            }
            // NEOPETS_COOP_DEADLY=t: on tick t player 2 is dropped onto the deadly ground nearest
            // player 1 (within 300), and its place and health every 10 ticks after.
            if let Some(t) = std::env::var("NEOPETS_COOP_DEADLY").ok().and_then(|v| v.trim().parse::<u64>().ok()) {
                if play.ticks == t {
                    let a = play.player.loco.position();
                    let spot = play.diag_deadly_near(a[0], a[2], 300.0);
                    println!("COOP deadly: nearest deadly ground {spot:?} (player 1 at {a:?})");
                    if let (Some(at), Some(o)) = (spot, play.others.first_mut()) {
                        let f = o.loco.facing;
                        o.loco.teleport([at[0], at[1] + 1.0, at[2]], f);
                    }
                }
                if play.ticks > t && play.ticks <= t + 120 && play.ticks % 10 == 0 {
                    if let Some(o) = play.others.first() {
                        let q = o.loco.position();
                        println!("COOP {} deadly: p2 at {:?} hp {} probe {} | {}", play.ticks, q.map(|v| v.round()), o.hero_hp, play.diag_probe(q[0], q[2], q[1] + 2.5), play.last_hurt);
                    }
                }
            }
            // NEOPETS_COOP_HURT1=t: player 1 takes a hit of 9,999 on tick t; both players' downs,
            // player 1's state and the game-over page every 20 ticks after.
            if let Some(t) = std::env::var("NEOPETS_COOP_HURT1").ok().and_then(|v| v.trim().parse::<u64>().ok()) {
                if play.ticks == t {
                    play.diag_hurt(9999.0);
                }
                if play.ticks > t && play.ticks % 20 == 0 {
                    println!("COOP {} downs: p1 {:?} state {} p2 {:?}; page {:?} flag E {}", play.ticks, play.player.down, play.player.loco.state, play.others.first().map(|o| o.down), play.death.page, play.death.flag_e);
                }
            }
            // NEOPETS_COOP_SQUARE="t[,t...]": player 2 presses Square on those ticks, and its
            // caption is printed then.
            if std::env::var("NEOPETS_COOP_SQUARE").is_ok_and(|v| v.split(',').any(|t| t.trim().parse::<u64>().ok() == Some(play.ticks))) {
                p2.square = true;
                println!("COOP {} square: p2 caption {:?}", play.ticks, play.others.first().map(|o| o.captions.0));
            }
            // NEOPETS_COOP_BOX: player 2 presses Square in talk boxes every 10 ticks instead of
            // player 1 (player 2 moves the boxes on).
            if std::env::var_os("NEOPETS_COOP_BOX").is_some() && play.dialogue_open && play.ticks % 10 == 5 {
                p2.square = true;
            }
            // NEOPETS_COOP_DOOR="n@t[;n@t...]": on tick t player 2 is put in door plane n of the
            // world then current (player 1 fifteen units back; the planes are listed then) and
            // presses Square every 20 ticks after the first; player 2's caption, the door line and
            // the world every 10 ticks.
            if let Ok(v) = std::env::var("NEOPETS_COOP_DOOR") {
                let doors: Vec<(usize, u64)> = v.split(';').filter_map(|e| {
                    let (n, t) = e.split_once('@')?;
                    Some((n.trim().parse().ok()?, t.trim().parse().ok()?))
                }).collect();
                for &(n, t) in &doors {
                    if play.ticks != t {
                        continue;
                    }
                    for (k, pl) in play.world.planes.iter().enumerate() {
                        println!("COOP door plane {k} of {}: at ({:.1}, {:.1}, {:.1}) normal ({:.2}, {:.2}, {:.2}) width {:.1} height {:.1}", play.world.pack, pl.p[0], pl.p[1], pl.p[2], pl.f[0], pl.f[1], pl.f[2], pl.w, pl.h);
                    }
                    if let Some(pl) = play.world.planes.get(n).copied() {
                        let at = [pl.p[0] + pl.d[0] * pl.w * 0.5 + pl.q[0] * 0.5, pl.p[1] + pl.d[1] * pl.w * 0.5 + pl.q[1] * 0.5, pl.p[2] + pl.d[2] * pl.w * 0.5 + pl.q[2] * 0.5];
                        let face = [-pl.f[0], 0.0, -pl.f[2]];
                        play.player.loco.teleport([at[0] + pl.f[0] * 15.0, at[1] + 2.0, at[2] + pl.f[2] * 15.0], face);
                        if let Some(o) = play.others.first_mut() {
                            o.loco.teleport(at, face);
                        }
                        println!("COOP door: player 2 put in door {n} of {} at ({:.1}, {:.1}, {:.1})", play.world.pack, at[0], at[1], at[2]);
                    }
                }
                let first = doors.iter().map(|d| d.1).min().unwrap_or(u64::MAX);
                if play.ticks > first && play.ticks % 20 == 0 {
                    p2.square = true;
                }
                if play.ticks > first && play.ticks % 10 == 0 {
                    let q = play.others.first().map(|o| (o.loco.position(), o.captions.0));
                    let a = play.player.loco.position();
                    println!("COOP {} door: world {} job {:?} p2 {:?} | p1 ({:.1}, {:.1}, {:.1}) | {}", play.ticks, play.world.pack, play.job, q, a[0], a[1], a[2], play.last_door);
                }
            }
            // NEOPETS_COOP_DRINK: below half health player 2 opens its own quick panel, moves to
            // the first entry it owns and presses Cross (as the trace's own drink above).
            if std::env::var_os("NEOPETS_COOP_DRINK").is_some() {
                if let Some(o) = play.others.first_mut() {
                    if o.hero_hp > 0 && o.hero_hp < o.hero_max / 2 {
                        let owned = (0..4).find(|&k| {
                            let id = o.quick.entries[k];
                            id >= 0 && play.state.items.get(&(id as u16)).is_some_and(|&n| n > 0)
                        });
                        if !o.quick.open {
                            o.dpad.up |= play.ticks % 20 == 0;
                        } else {
                            match owned {
                                Some(k) if o.quick.selection == k => p2.jump = true,
                                Some(0) => o.dpad.right = true,
                                Some(1) => o.dpad.up = true,
                                Some(2) => o.dpad.left = true,
                                Some(_) => o.dpad.down = true,
                                None => o.dpad.triangle = true,
                            }
                        }
                        if play.ticks % 10 == 0 {
                            println!("COOP {} drink: p2 hp {} panel open {} entries {:?} selection {} owned {:?}; p1 panel open {}", play.ticks, o.hero_hp, o.quick.open, o.quick.entries, o.quick.selection, owned, play.quick.open);
                        }
                    }
                }
            }
            play.coop_step(&[(p2, neopets_sim::camera::CameraPad::default())]);
            if play.ticks % 30 == 0 {
                if let Some(o) = play.others.first() {
                    let q = o.loco.position();
                    let a = play.player.loco.position();
                    println!("COOP {} p2 ({:.1}, {:.1}, {:.1}) state {} clip {} hp {} down {:?} | p1 ({:.1}, {:.1}, {:.1}) hp {} down {:?} state {} | p2 eye ({:.1}, {:.1}, {:.1}) | p2 line {:?}", play.ticks, q[0], q[1], q[2], o.loco.state, o.hero.clip_state().action, o.hero_hp, o.down, a[0], a[1], a[2], play.player.hero_hp, play.player.down, play.player.loco.state, o.camera.eye[0], o.camera.eye[1], o.camera.eye[2], o.down_line.as_deref().map(String::from_utf8_lossy));
                    let near: Vec<String> = play.characters.iter().filter(|c| c.enemy.is_some() && (c.position[0] - q[0]).hypot(c.position[2] - q[2]) < 15.0).map(|c| format!("{} hp {} at {:.1}", c.name, c.hp, (c.position[0] - q[0]).hypot(c.position[2] - q[2]))).collect();
                    println!("COOP {} enemies near p2: {} | p2 lock {:?} targets {} p1 lock {:?} targets {}", play.ticks, near.join(", "), o.lock, o.targets.records.len(), play.player.lock, play.player.targets.records.len());
                    println!("COOP {} p2 hud c {} g0 {} health {:?} | p1 hud c {} g0 {} health {:?}", play.ticks, o.hud.c, o.hud.g0, o.hud.health, play.hud.c, play.hud.g0, play.hud.health);
                }
            }
        }
        // NEOPETS_TRACE_CHECKSUM: the state checksum after each tick (docs/MULTIPLAYER.md step 1).
        if std::env::var_os("NEOPETS_TRACE_CHECKSUM").is_some() {
            println!("CHECKSUM {} {:016x}", play.ticks, play.checksum());
        }
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
        let now: Vec<usize> = (0..14).filter(|&i| play.player.statuses.active(i)).collect();
        if now != statuses_shown {
            println!("      hero statuses {now:?} on tick {} (hp {}, mp {})", play.ticks, play.player.hero_hp, play.player.hero_mp);
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
            println!("      zones: grid {} row {} key {}; patches on {} of {}; characters {} ({off} cell-off), enemies {:?}", w.zones.as_ref().map_or(0, |g| g.blocks.len()), play.zones.row, play.zones.key, on.iter().sum::<usize>(), on.len(), play.characters.len(), play.characters.iter().filter(|c| c.enemy.is_some()).map(|c| (c.pack, c.name.as_str(), c.inactive)).collect::<Vec<_>>());
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
                play.player.statuses.apply(code, secs);
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
        let (p, b) = (play.player.loco.position(), &play.player.loco.body);
        println!(
            "{:5} pos ({:9.3} {:8.3} {:9.3}) state {:2} phase {:2} action {:3} gap {:8.3} vy {:7.3} n ({:.2} {:.2} {:.2})",
            play.ticks, p[0], p[1], p[2], play.player.loco.state, play.player.loco.phase, play.player.loco.action(), b.gap, b.velocity[1],
            b.normal[0], b.normal[1], b.normal[2]
        );
        // NEOPETS_TRACE_FACING: the facing, the stick target and the body velocity too.
        if std::env::var_os("NEOPETS_TRACE_FACING").is_some() {
            println!("      facing {:?} target {:?} velocity {:?} root {:?} camera rows {:?}", play.player.loco.facing, play.player.loco.target, b.velocity, play.player.root_step, play.player.camera.rows()); println!("      eye {:?} focus {:?}", play.player.camera.eye, play.player.camera.focus);
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
            println!("      talking: hero main {} | partner {partner:?}", play.player.hero.action());
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
    for line in play.duel_report() {
        println!("{line}");
    }
    println!("Held items at the end: {held:?}; {} NP; weapon {:?}, shield {:?}, entries {:?}", play.state.neopoints, play.weapon, play.shield, play.equipment.entries);
    if save_test {
        let path = "private/saves/trace.bin";
        let mut file = read_save(path).unwrap_or_default();
        // NEOPETS_TRACE_SAVE_SLOT=k: write slot k of the trace file instead of slot 0.
        let k = std::env::var("NEOPETS_TRACE_SAVE_SLOT").ok().and_then(|v| v.trim().parse::<usize>().ok()).unwrap_or(0).min(neopets_sim::save::SLOTS - 1);
        play.write_slot(&mut file.slots[k], &save_date());
        file.slots.swap(0, k);
        let s = &file.slots[0];
        println!(
            "Saved slot 0: keys {:?}, location {:?}, date {:?}, tier {}, wallet {}, pools {:?}, maxima {:?}, motes {:?}, entries {:?}",
            [s.key(0), s.key(1), s.key(2)],
            String::from_utf8_lossy(s.location()),
            String::from_utf8_lossy(s.date()),
            s.tier(),
            s.i32(neopets_sim::save::at::WALLET),
            s.pools(0),
            s.maximum_words(0),
            (0..2).map(|h| (0..3).map(|i| s.mote(h, i)).collect::<Vec<_>>()).collect::<Vec<_>>(),
            s.adds().iter().map(|a| (a.0, a.1, a.3, a.4)).collect::<Vec<_>>()
        );
        file.slots.swap(0, k);
        if let Err(e) = write_save(path, &file) {
            eprintln!("Save failed: {e}");
            std::process::exit(1);
        }
        let mut back = read_save(path).expect("the file just written reads back");
        assert_eq!(back, file);
        back.slots.swap(0, k);
        file.slots.swap(0, k);
        assert_eq!(back, file);
        println!("Before: {}", play.status());
        match load_game(archive, &play, &back.slots[0]) {
            Ok(loaded) => {
                println!("Loaded: {}", loaded.status());
                println!("        {}", loaded.status_worlds());
                let summary = |p: &play::Play| {
                    let mut inv: Vec<_> = p.state.items.iter().map(|(k, v)| (*k, *v)).collect();
                    inv.sort();
                    format!("weapon {:?}, shield {:?}, items {inv:?}, flags {:?}, luck {}, statuses {:?}, restart {:?}, collected {:?}, maps {:?}", p.weapon, p.shield, p.state.flags, p.luck, p.player.statuses.timers, p.restart, p.state.collected, p.state.maps)
                };
                println!("Round trip before: {}", summary(&play));
                println!("Round trip after:  {}", summary(&loaded));
            }
            Err(e) => eprintln!("Load failed: {e}"),
        }
    }
}

/// The movie table and headers (SPEC-0026, SPEC-0008), read once from the disc.
fn setup_movies(mut commands: Commands, archive: Option<ResMut<Archive>>) {
    let Some(mut archive) = archive else { return };
    match movie_view::Movies::load(&mut archive.disc) {
        Ok(m) => commands.insert_resource(m),
        Err(e) => eprintln!("movies: {e}"),
    }
}

/// Bevy's GPU light clustering off (its CPU path instead): with no point lights, spotlights,
/// light probes or decals in the game (see the camera's ClusterConfig), the GPU path still set up
/// buffers and ran its compute passes every frame, about 0.35 ms of setup and 0.18 ms of
/// command recording on the render thread (performance pass of 2026-11-12). The CPU path has
/// nothing to cluster. NEOPETS_GPU_CLUSTERING keeps the GPU path (diagnostic).
fn no_gpu_clustering(settings: Option<ResMut<bevy::light::cluster::GlobalClusterSettings>>) {
    if std::env::var_os("NEOPETS_GPU_CLUSTERING").is_some() {
        return;
    }
    if let Some(mut s) = settings {
        s.gpu_clustering = None;
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
            // NEOPETS_TRACE_MOTES="w0,w1,w2" as in the trace: held Motes equipped (diagnostic).
            if let Ok(m) = std::env::var("NEOPETS_TRACE_MOTES") {
                for (word, id) in m.split(',').enumerate().filter_map(|(k, t)| Some((k, t.trim().parse::<u16>().ok()?))) {
                    play.item_command(&menus::Command::EquipMote { word, id });
                }
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
            // NEOPETS_TRACE_ROBERTA=1: start as Roberta (diagnostic).
            if std::env::var("NEOPETS_TRACE_ROBERTA").is_ok_and(|v| v == "1") {
                play.trace_select_hero(1);
            }
            // NEOPETS_REED_FIELD=1: start on the reed field (the developer menu's test world).
            if std::env::var_os("NEOPETS_REED_FIELD").is_some() {
                let line = play.enter_reed_field();
                info!("{line}");
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
                        if let Ok(w) = neopets_data::archive::boot_u32_table(&boot, neopets_sim::book::STYLE_MASKS, 23) {
                            flow.style_masks = w;
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
    mut pass1_materials: ResMut<Assets<pass1::Pass1Material>>,
    mut images: ResMut<Assets<Image>>,
    mut hero_stamp: Local<Option<(u64, bool, usize, usize)>>,
    (skin_assets, mut joint_query, mut hero_joints): (Res<gpu_skin::SkinAssets>, Query<&mut GlobalTransform, With<gpu_skin::SkinJoint>>, Local<Vec<Entity>>),
) {
    let _span = crate::perf::span("play_frame");
    let Some(mut state) = play else { return };
    let p = state.play.player.loco.position();
    let f = state.play.player.loco.facing;
    // Characters face +Z in model space (SPEC-0004); turn that onto the facing.
    let rotation = Quat::from_rotation_y(f[0].atan2(f[2]));
    if !state.play.spawned {
        state.play.spawned = true;
        // A hero switch re-spawns the hero with the incoming model.
        for e in &hero_roots {
            commands.entity(e).despawn();
        }
        // SPEC-0046 (R-0128): program 6 draws with texture function highlight 2, so its vertex
        // alpha is an additive highlight, never the opacity: every program-6 part of the hero
        // is drawn solid (Roberta's head dome among them, whose base alpha is 0). The additive
        // highlight itself is not drawn.
        {
            for part in state.play.player.model.parts.iter_mut().filter(|p| p.program == Some(6) || (!p.colors.is_empty() && p.colors.iter().all(|c| c[3] < 0.5))) {
                for c in &mut part.colors {
                    c[3] = 1.0;
                }
            }
        }
        let parts = part_assets(&state.play.player.model, &mut meshes, &mut materials, &mut images);
        // SPEC-0003 How a pass is drawn: two-pass parts' second layer, skinned with pass 0.
        let second = pass1_assets(&state.play.player.model, &mut meshes, &mut pass1_materials, &mut images);
        let mut skinned = Vec::new();
        // GPU skinning (gpu_skin.rs), as for the world's characters.
        let bones = state.play.player.hero.skin_matrices().len();
        let gpu = gpu_skin::enabled() && gpu_skin::fits(bones);
        if gpu {
            for (i, mesh) in second.iter().map(|(i, m, _, _)| (*i, m)).chain(parts.iter().map(|(i, m, _)| (*i, m))) {
                let part = &state.play.player.model.parts[i];
                if !part.weights.is_empty() {
                    if let Some(mut m) = meshes.get_mut(mesh) {
                        gpu_skin::prepare_mesh(&mut m, part, bones);
                    }
                }
            }
        }
        let skin = gpu.then(|| skin_assets.bindposes.clone());
        let mut joints = Vec::new();
        commands
            .spawn((Transform::from_translation(Vec3::from(p)).with_rotation(rotation), Visibility::Visible, HeroRoot))
            .with_children(|c| {
                if gpu {
                    joints = (0..bones + gpu_skin::DATA_JOINTS + 1).map(|_| c.spawn((GlobalTransform::IDENTITY, gpu_skin::SkinJoint)).id()).collect();
                }
                let skin_of = |joints: &Vec<Entity>| skin.clone().map(|h| (bevy::mesh::skinning::SkinnedMesh { inverse_bindposes: h, joints: joints.clone() }, bevy::camera::visibility::NoFrustumCulling));
                for (i, mesh, material, scroll) in second {
                    let skinned_part = !state.play.player.model.parts[i].weights.is_empty();
                    if skinned_part && !gpu {
                        skinned.push((mesh.clone(), i));
                    }
                    let mut e = c.spawn((Mesh3d(mesh), MeshMaterial3d(material), Transform::IDENTITY));
                    if let Some(s) = scroll {
                        e.insert(pass1::Pass1Scroll(s));
                    }
                    if let Some(k) = skin_of(&joints).filter(|_| skinned_part) {
                        e.insert(k);
                    }
                }
                for (i, mesh, material) in parts {
                    let skinned_part = !state.play.player.model.parts[i].weights.is_empty();
                    if skinned_part && !gpu {
                        skinned.push((mesh.clone(), i));
                    }
                    let mut e = c.spawn((Mesh3d(mesh), MeshMaterial3d(material), Transform::IDENTITY));
                    if let Some(k) = skin_of(&joints).filter(|_| skinned_part) {
                        e.insert(k);
                    }
                }
            });
        state.skinned = skinned;
        *hero_joints = joints;
        *hero_stamp = None;
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
    // Performance: the hero's skin is rebuilt once per game tick (and when its flash or model
    // changes), not on every drawn frame.
    let stamp = (state.play.ticks, state.play.player.hero_flash > 0, state.play.hero_index, state.skinned.len());
    let fresh = std::mem::replace(&mut *hero_stamp, Some(stamp)) != Some(stamp);
    let matrices = if fresh { state.play.player.hero.skin_matrices() } else { Vec::new() };
    let tint = state.play.draw_tint(None, p);
    let lights = state.play.lights(None, p);
    if fresh && !hero_joints.is_empty() {
        let look = gpu_skin::Look { light_rotation: rotation, lights: &lights, tint: tint.unwrap_or([1.0; 3]), slots: if state.play.player.hero_flash > 0 { &play::FLASH_COLOUR } else { &[[1.0; 3]; 15] }, ghost: 1.0 };
        let root = Transform::from_translation(Vec3::from(p)).with_rotation(rotation).compute_affine();
        for (e, m) in hero_joints.iter().zip(gpu_skin::joint_matrices(&root, &matrices, &look)) {
            if let Ok(mut g) = joint_query.get_mut(*e) {
                set_if(&mut g, m);
            }
        }
    }
    for (handle, i) in state.skinned.iter().filter(|_| fresh) {
        let part = &state.play.player.model.parts[*i];
        let (pos, nrm) = character::skin_part(&matrices, &part.positions, &part.normals, &part.weights, &part.bones);
        let flash = (state.play.player.hero_flash > 0).then_some(&play::FLASH_COLOUR);
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
    let _span = crate::perf::span("sync_fade");
    let Some(state) = play else { return };
    if bars.is_empty() {
        for top in [true, false] {
            let mut node = Node { position_type: PositionType::Absolute, width: Val::Percent(100.0), height: Val::Px(0.0), ..default() };
            if top {
                node.top = Val::Px(0.0);
            } else {
                node.bottom = Val::Px(0.0);
            }
            commands.spawn((node, BackgroundColor(Color::BLACK), Visibility::Hidden, GlobalZIndex(4), LetterBar));
        }
    }
    let h = state.play.bars.height() / 448.0 * 100.0;
    // Written only on change: a changed node makes Bevy lay out the whole UI again.
    for (mut node, mut vis) in &mut bars {
        if node.height != Val::Percent(h) {
            node.height = Val::Percent(h);
        }
        set_if(&mut vis, if h > 0.0 { Visibility::Inherited } else { Visibility::Hidden });
    }
    if overlay.is_empty() {
        commands.spawn((
            Node { position_type: PositionType::Absolute, width: Val::Percent(100.0), height: Val::Percent(100.0), ..default() },
            BackgroundColor(Color::NONE),
            Visibility::Hidden,
            // SPEC-0046 frame steps 11 and 12: the HUD (10) and the menus draw on top of the fade and the bars.
            GlobalZIndex(5),
            FadeOverlay,
        ));
        return;
    }
    let f = &state.play.fade;
    for (mut colour, mut vis) in &mut overlay {
        let a = f.opacity();
        set_if(&mut vis, if a > 0.0 { Visibility::Inherited } else { Visibility::Hidden });
        let c = f.colour.map(|v| v as f32 / 255.0);
        let want = Color::srgba(c[0], c[1], c[2], a);
        if colour.0 != want {
            colour.0 = want;
        }
    }
}

/// The window's dev script (NEOPETS_DEV_SCRIPT): the trace's stick (raw forward and side, from
/// each listed tick on), facing (turned toward a point on a listed tick), Cross and Square press
/// (on listed ticks) scripts, applied to this tick's input.
fn dev_script(play: &mut play::Play, input: &mut neopets_sim::hero::HeroInput) {
    let t = play.ticks;
    // NEOPETS_DEV_CARRY=tick: the hero picks up the first Miamouse, for a picture.
    if std::env::var("NEOPETS_DEV_CARRY").ok().and_then(|v| v.trim().parse::<u64>().ok()) == Some(t) {
        play.diag_pick_up();
    }
    // NEOPETS_DEV_OBJEND="tick:a,b,...": those objects of the current world put at the end of their animation (diagnostic).
    if let Some((k, objs)) = std::env::var("NEOPETS_DEV_OBJEND").ok().as_deref().and_then(|v| v.split_once(':')).map(|(a, b)| (a.trim().parse::<u64>().unwrap_or(0), b.split(',').filter_map(|s| s.trim().parse::<usize>().ok()).collect::<Vec<_>>())) {
        if k == t {
            let acts: Vec<neopets_sim::level::Action> = objs.into_iter().map(|obj| neopets_sim::level::Action::InitObjOn { obj, end: true }).collect();
            play.apply(&acts);
        }
    }
    // NEOPETS_DEV_HP="tick:hp:max": the hero's health and maximum on that tick, for a picture of the bar.
    if let Some(v) = std::env::var("NEOPETS_DEV_HP").ok() {
        let n: Vec<i32> = v.split(':').filter_map(|s| s.trim().parse().ok()).collect();
        if n.len() == 3 && n[0] as u64 == t {
            play.player.hero_max = n[2];
            play.player.hero_hp = n[1];
        }
    }
    // NEOPETS_DEV_ITEM="tick:id": that item placed ahead of the hero, for a picture.
    if let Some((k, id)) = std::env::var("NEOPETS_DEV_ITEM").ok().as_deref().and_then(|v| v.split_once(':')).and_then(|(a, b)| Some((a.trim().parse::<u64>().ok()?, b.trim().parse::<u16>().ok()?))) {
        if k == t {
            play.diag_place_item(id);
        }
    }
    // NEOPETS_TRACE_PLACE="tick:x,z,y[;...]": the hero placed there, as in the trace.
    for e in std::env::var("NEOPETS_TRACE_PLACE").unwrap_or_default().split(';') {
        let Some((k, r)) = e.trim().split_once(':') else { continue };
        let n: Vec<f32> = r.split(',').filter_map(|v| v.trim().parse().ok()).collect();
        if k.parse::<u64>().ok() == Some(t) && n.len() >= 2 {
            play.place_at(n[0], n[1], n.get(2).copied().unwrap_or(1.0e4));
        }
    }
    // NEOPETS_DEV_ATTACK="from:to": the attack pressed on that tick and held to the other, with _CHEATS, _GIVE,
    // _MOTE_AT, _ROBERTA and _CEILING as in the trace, so a move can be pictured (diagnostic).
    if let Some((a, b)) = std::env::var("NEOPETS_DEV_ATTACK").ok().as_deref().and_then(|v| v.split_once(':')).and_then(|(a, b)| Some((a.parse::<u64>().ok()?, b.parse::<u64>().ok()?))) {
        input.attack = t == a;
        input.attack_held = (a..=b).contains(&t);
    }
    if t == 5 {
        if let Ok(v) = std::env::var("NEOPETS_TRACE_CHEATS") {
            play.cheats = play::Cheats { god: v.contains("god"), magic: v.contains("magic"), charge: v.contains("charge"), noclip: v.contains("noclip"), haste: play.cheats.haste, swarm: v.contains("swarm"), spin: v.contains("spin") };
        }
        if let Some(k) = std::env::var("NEOPETS_TRACE_HASTE").ok().and_then(|v| v.parse::<u8>().ok()) {
            play.cheats.haste = k;
        }
        if std::env::var("NEOPETS_TRACE_ROBERTA").is_ok_and(|v| v == "1") {
            play.trace_select_hero(1);
        }
    }
    for g in std::env::var("NEOPETS_TRACE_GIVE").unwrap_or_default().split(',') {
        let n: Vec<i64> = g.split(':').filter_map(|x| x.trim().parse().ok()).collect();
        if let [k, id, count] = n[..] {
            if k as u64 == t {
                play.diag_give(id as u16, count as i32);
            }
        }
    }
    for e in std::env::var("NEOPETS_TRACE_MOTE_AT").unwrap_or_default().split(';') {
        let n: Vec<u64> = e.split(':').filter_map(|x| x.trim().parse().ok()).collect();
        if let [k, word, id] = n[..] {
            if k == t {
                play.item_command(&menus::Command::EquipMote { word: word as usize, id: id as u16 });
            }
        }
    }
    if let Some((k, c)) = std::env::var("NEOPETS_TRACE_CEILING").ok().as_deref().and_then(|v| v.split_once(':')).and_then(|(a, b)| Some((a.parse::<u64>().ok()?, b.parse::<usize>().ok()?))) {
        if k == t {
            play.diag_ceiling(c);
        }
    }
    if let Ok(v) = std::env::var("NEOPETS_TRACE_STICK") {
        let mut set = None;
        for e in v.split(';') {
            let Some((k, fs)) = e.trim().split_once(':') else { continue };
            if k.parse::<u64>().is_ok_and(|k| k <= t) {
                set = fs.split_once(',').and_then(|(f, s)| Some((f.trim().parse::<f32>().ok()?, s.trim().parse::<f32>().ok()?)));
            }
        }
        if let Some((f, s)) = set {
            input.forward = f / 87.5;
            input.side = s / 87.5;
        }
    }
    if let Ok(v) = std::env::var("NEOPETS_TRACE_FACE") {
        for e in v.split(';') {
            let Some((k, p)) = e.split_once(':') else { continue };
            let n: Vec<f32> = p.split(',').filter_map(|x| x.trim().parse().ok()).collect();
            if k.trim().parse::<u64>().ok() == Some(t) && n.len() >= 2 {
                let h = play.player.loco.position();
                let (dx, dz) = (n[0] - h[0], n[1] - h[2]);
                let l = dx.hypot(dz).max(1e-6);
                play.player.loco.facing = [dx / l, 0.0, dz / l];
            }
        }
    }
    if std::env::var("NEOPETS_TRACE_CROSS").is_ok_and(|v| v.split(',').any(|x| x.trim().parse::<u64>().ok() == Some(t))) {
        input.jump = true;
    }
    if std::env::var("NEOPETS_TRACE_SQUARE").is_ok_and(|v| v.split(',').any(|x| x.trim().parse::<u64>().ok() == Some(t))) {
        input.square = true;
    }
}
