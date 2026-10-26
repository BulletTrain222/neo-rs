//! Play mode (M7): a hero outfit placed at a world pack's spawn (SPEC-0006), moving by the
//! locomotion of neopets-sim (SPEC-0033 tiers and turn, SPEC-0040 root-step travel) over the
//! ground probe (SPEC-0018), animated by its action's clips (SPEC-0005), followed by the
//! SPEC-0027 camera, and travelling between areas by SPEC-0006 streaming and world swaps.
use crate::character::HeroAnim;
use neopets_data::archive::ArchiveIndex;
use neopets_data::ground::{self, CellOrder, Ground, ProbeHit, TeleportPlane};
use neopets_data::model::ModelData;
use neopets_data::travel::{BootTravel, Kind, Neighbours};
use neopets_data::{Disc, resource};
use neopets_sim::camera::{self, CameraPad, FollowCamera, Followed};
use neopets_sim::contact::{self, BoxShape, WallEdge};
use neopets_sim::combat::Cylinder;
use neopets_sim::enemy::{self, Enemy};
use neopets_sim::hero::{self as hero_states, HeroInput, Locomotion};
use neopets_sim::level::{self, Action, GameState};
use neopets_sim::motion::HERO_HEIGHT;
use neopets_sim::travel::{self as buffers, Buffer};
use std::cell::Cell;
use std::collections::HashMap;

const UP: [f32; 3] = [0.0, 1.0, 0.0];
/// The slot number used here for a world's embedded region.
const EMBEDDED_SLOT: u8 = 255;
/// The region slot the live petpet's package loads into (SPEC-0047 Activation).
const PET_SLOT: u8 = 254;
/// The slot a forced bind reloads the embedded region into (SPEC-0045 1217 Stage spawns).
const REBIND_SLOT: u8 = 253;
/// SPEC-0006 Default start: pack 691 in T and 909 in E, the exterior current.
const START_EXTERIOR: u32 = 909;
const START_TRANSITION: u32 = 691;

/// A world pack resident in a buffer: what the hero's update and the camera read of it.
pub struct World {
    pub pack: u32,
    pub kind: Kind,
    pub buffer: Buffer,
    /// The placed effects (type 39), by slot.
    pub placed_effects: Vec<neopets_data::effect::Placed>,
    /// The placed objects (type 29) with their motion streams and script states (SPEC-0028
    /// section 7.1), by object index.
    pub objects: Vec<Option<neopets_data::world::Object>>,
    pub motions: Vec<neopets_sim::object::Motion>,
    pub object_anims: Vec<neopets_sim::object::ObjAnim>,
    /// The foliage patches (type 16) and the registry ids of the pack's type-8 children.
    pub patches: Vec<neopets_data::foliage::Patch>,
    /// The area's character light sets (type 30, SPEC-0028 section 2), in table order.
    pub lights: Vec<neopets_sim::charlight::Record>,
    /// The world id (the area table's id of this pack; -1 when it has none) and its binary
    /// grid (type 35) with block 0 keyed by that id (SPEC-0006 Binary grids).
    pub world_id: i32,
    pub zones: Option<neopets_sim::zone::Grid>,
    pub children: Vec<u16>,
    ground: Option<Ground>,
    /// Rectangle primitives that push characters out (SPEC-0018 box push; new-game mask 0).
    boxes: Vec<BoxShape>,
    neighbours: Neighbours,
    /// Teleport planes (doors), the arrival-door list (tag 0x0E) and the entry records.
    pub planes: Vec<TeleportPlane>,
    arrivals: Neighbours,
    entries: Vec<([f32; 3], [f32; 3])>,
    /// The entry records' locator tags (u16 +0x1E), by list position.
    tags: Vec<u16>,
    /// The module imports RegisterTeleportCallback: its doors are script doors, which need the
    /// level-script runtime (not implemented), so they do nothing here.
    pub scripted_doors: bool,
    /// The dialogue line table (type 1000) that LevelMsg reads.
    pub lines: neopets_data::dialogue::LineTable,
    /// The entity-id table (tag 0x12): (region byte, ordinal) per entity id.
    entities: Vec<(u8, u8)>,
    /// Navigation meshes (type 21) and the Whinny pens the level sets.
    pub navmeshes: Vec<neopets_sim::nav::NavMesh>,
    pens: Vec<neopets_sim::whinny::Pen>,
    /// The save points of the pack's own region (kind-4 records, at most 8; SPEC-0025) and its
    /// map area id (type 1001, SPEC-0006 Area names).
    pub save_points: Vec<neopets_sim::save::SavePoint>,
    pub map_area: Option<i32>,
    /// The type-1001 record's music key and selector (SPEC-0009 Area to music row).
    pub music: Option<(i32, u32)>,
    /// Placed sound emitters (SPEC-0031): the fourth list's ambient loops and adventure zones
    /// with their start-delay counters (cleared at load), and the sphere emitters.
    pub ambients: Vec<neopets_data::ground::Ambient>,
    ambient_counters: Vec<u8>,
    pub spheres: Vec<neopets_data::ground::SphereEmitter>,
    /// Script triggers (SPEC-0030 Triggers; SPEC-0028 section 13): the trigger capacity, the
    /// volumes that carry an index, the routine registered per index and the records; the script
    /// groups (type 40), the placement id table, the embedded region's item points, the lock
    /// table and the groups opened in this load.
    capacity: usize,
    volumes: Vec<neopets_sim::trigger::Volume>,
    routines: HashMap<u16, u8>,
    /// Script-group type registrations: trigger index to its owner group (SPEC-0048 Script
    /// groups, type registration); a later RegisterTrigger on the index replaces it.
    group_routes: HashMap<u16, usize>,
    records: HashMap<u16, neopets_sim::trigger::Record>,
    pub groups: Vec<neopets_data::trigger::ScriptGroup>,
    placements: Vec<(u8, u8)>,
    item_points: Vec<neopets_data::region::ItemPoint>,
    locks: Vec<(i32, i32, i32)>,
    opened: std::collections::BTreeSet<usize>,
    /// Camera tracks (type 28; SPEC-0052), by track id.
    pub tracks: Vec<neopets_sim::scene::Track>,
}

impl World {
    /// The script volumes' trigger indices and a point inside each (diagnostic).
    pub fn volume_points(&self) -> Vec<(u16, [i32; 3])> {
        use neopets_sim::trigger::Shape;
        self.volumes
            .iter()
            .map(|v| {
                let p = match v.shape {
                    Shape::Sphere { centre, .. } => centre,
                    Shape::Cylinder { base, axis, length, .. } => std::array::from_fn(|k| base[k] + 0.5 * length * axis[k]),
                    Shape::Rectangle { origin, axes, extents } => std::array::from_fn(|k| origin[k] + 0.5 * (extents[0] * axes[0][k] + extents[1] * axes[1][k])),
                };
                (v.index, p.map(|c| c as i32))
            })
            .collect()
    }

    /// The entity table: (region j or 0xFF for the embedded region, ordinal) by entity id.
    pub fn entity_table(&self) -> Vec<(u8, u16)> {
        self.entities.iter().map(|&(r, o)| (r, o as u16)).collect()
    }

    pub fn load(disc: &mut Disc, index: &ArchiveIndex, boot: &BootTravel, pack: u32) -> Result<(Self, Vec<u8>), String> {
        let bytes = index.read(disc, pack)?;
        let chain = resource::walk_chunks(&bytes, None)?;
        let kind = boot.kind(pack, &bytes, &chain.chunks);
        let buffer = match kind {
            Kind::Transition => Buffer::Transition,
            Kind::Exterior => Buffer::Exterior,
            Kind::Interior => Buffer::Interior,
        };
        let world = Self {
            pack,
            kind,
            buffer,
            placed_effects: neopets_data::effect::placed(&bytes, &chain.chunks),
            patches: neopets_data::foliage::patches(&bytes, &chain.chunks),
            lights: neopets_data::light::light_sets(&bytes, &chain.chunks).iter().map(light_record).collect(),
            world_id: world_id(boot, pack),
            zones: neopets_data::zone::zone_grid(&bytes, &chain.chunks).map(|blocks| neopets_sim::zone::Grid {
                blocks: blocks
                    .into_iter()
                    .enumerate()
                    .map(|(i, b)| neopets_sim::zone::Block { key: if i == 0 { world_id(boot, pack) } else { b.key }, width: b.width, height: b.height, cells: b.cells, after: b.after })
                    .collect(),
            }),
            objects: Vec::new(),
            motions: Vec::new(),
            object_anims: Vec::new(),
            children: neopets_data::model::find_trees(&bytes, &chain.chunks).0.iter().filter(|t| t.depth > 0).map(|t| t.role_id).collect(),
            ground: Ground::of(&bytes, &chain.chunks, CellOrder::XMajor),
            boxes: ground::rectangles(&bytes, &chain.chunks)
                .into_iter()
                .filter(|r| r.pushes(0))
                .map(|r| BoxShape { origin: r.origin, axes: r.axes, extents: r.extents })
                .collect(),
            neighbours: Neighbours::of(&bytes, &chain.chunks),
            planes: ground::teleport_planes(&bytes, &chain.chunks),
            arrivals: Neighbours::arrivals(&bytes, &chain.chunks),
            entries: (0..4096).map_while(|i| ground::entry_record(&bytes, &chain.chunks, i)).collect(),
            tags: (0..4096).map_while(|i| ground::entry_tag(&bytes, &chain.chunks, i)).collect(),
            scripted_doors: boot.imports(&bytes, &chain.chunks, "RegisterTeleportCallback"),
            lines: neopets_data::dialogue::LineTable::of(&bytes, &chain.chunks),
            entities: neopets_data::travel::entity_table(&bytes, &chain.chunks),
            navmeshes: neopets_data::navmesh::navmeshes(&bytes, &chain.chunks)
                .into_iter()
                .map(|m| neopets_sim::nav::NavMesh { kind: m.kind, triangles: m.triangles, neighbours: m.neighbours, table: m.table })
                .collect(),
            pens: Vec::new(),
            save_points: neopets_data::region::embedded_records(&bytes, &chain.chunks)
                .map(|r| r.save_points.iter().map(|s| neopets_sim::save::SavePoint { position: s.position, flag: s.flag }).collect())
                .unwrap_or_default(),
            map_area: neopets_data::travel::map_area(&bytes, &chain.chunks),
            music: neopets_data::audio::pack_record(&bytes, &chain.chunks),
            ambients: Vec::new(),
            ambient_counters: Vec::new(),
            spheres: Vec::new(),
            capacity: 0,
            volumes: Vec::new(),
            routines: HashMap::new(),
            group_routes: HashMap::new(),
            records: HashMap::new(),
            groups: neopets_data::trigger::script_groups(&bytes, &chain.chunks),
            placements: neopets_data::travel::placement_table(&bytes, &chain.chunks),
            item_points: neopets_data::region::embedded_records(&bytes, &chain.chunks).map(|r| r.items).unwrap_or_default(),
            locks: Vec::new(),
            opened: Default::default(),
            tracks: neopets_data::track::camera_tracks(&bytes, &chain.chunks)
                .into_iter()
                .map(|t| {
                    let key = |k: neopets_data::track::Key| neopets_sim::scene::Key { q: k.q, p: k.p, focal: k.focal };
                    neopets_sim::scene::Track { head: key(t.head), dt: t.dt, per_key_focal: t.per_key_focal, keys: t.keys.into_iter().map(key).collect() }
                })
                .collect(),
        };
        let mut world = world;
        world.objects = neopets_data::world::objects(&bytes, &chain.chunks);
        world.motions = world
            .objects
            .iter()
            .map(|o| o.as_ref().and_then(|o| neopets_sim::object::Motion::parse(&bytes[o.stream.clone()?], o.keys as usize)).unwrap_or_default())
            .collect();
        world.object_anims = vec![Default::default(); world.objects.len()];
        let (ambients, spheres) = ground::sound_emitters(&bytes, &chain.chunks);
        world.ambient_counters = vec![0; ambients.len()];
        world.ambients = ambients;
        world.spheres = spheres;
        let (capacity, volumes) = neopets_data::trigger::volumes(&bytes, &chain.chunks);
        world.capacity = capacity;
        world.volumes = volumes
            .into_iter()
            .map(|v| {
                use neopets_data::trigger::Shape as D;
                use neopets_sim::trigger::Shape as S;
                let shape = match v.shape {
                    D::Sphere { centre, radius } => S::Sphere { centre, radius },
                    D::Cylinder { base, axis, length, radius } => S::Cylinder { base, axis, length, radius },
                    D::Rectangle { origin, axes, extents } => S::Rectangle { origin, axes, extents },
                };
                neopets_sim::trigger::Volume { kind: v.kind, flags: v.flags, mask: v.mask, index: v.index, shape }
            })
            .filter(|v| v.fires_on_contact(0))
            .collect();
        Ok((world, bytes))
    }

    /// H(x, z, c) of SPEC-0006: the probe height, or the start height when nothing is sampled.
    fn snap(&self, l: [f32; 3], above: f32) -> [f32; 3] {
        let h = self.probe(l[0], l[2], l[1] + above);
        [l[0], if h.found { h.height } else { l[1] }, l[2]]
    }

    /// SPEC-0006 Teleport landing on plane k: refused when k is out of range; a flat plane
    /// (|F.y| > 0.85) lands at P + h·Q + w·D, 5 lower, unprobed; an upright one at
    /// P + 0.5·w·D + 5·normxz(F − Q), probed from 6 above (`upright` forces this form). The
    /// facing is F, or −F with the reverse flag. In an interior, entry record 0 replaces both.
    fn landing(&self, k: i32, reverse: bool, upright: bool) -> Option<([f32; 3], [f32; 3])> {
        let pl = self.planes.get(usize::try_from(k).ok()?)?;
        if self.buffer == Buffer::Interior {
            if let Some(&e) = self.entries.first() {
                return Some(e);
            }
        }
        let add = |a: [f32; 3], b: [f32; 3], s: f32| [a[0] + s * b[0], a[1] + s * b[1], a[2] + s * b[2]];
        let l = if pl.f[1].abs() > 0.85 && !upright {
            let l = add(add(pl.p, pl.q, pl.h), pl.d, pl.w);
            [l[0], l[1] - 5.0, l[2]]
        } else {
            let n = [pl.f[0] - pl.q[0], 0.0, pl.f[2] - pl.q[2]];
            let len = (n[0] * n[0] + n[2] * n[2]).sqrt().max(f32::MIN_POSITIVE);
            let l = add(add(pl.p, pl.d, 0.5 * pl.w), [n[0] / len, 0.0, n[2] / len], 5.0);
            self.snap(l, 6.0)
        };
        let g = if reverse { pl.f.map(|c| -c) } else { pl.f };
        Some((l, g))
    }

    /// The door the hero touches, if any. The hero's own door test is only partly specified
    /// (R-0012): here its body (feet to 3.5 above) must overlap the door rectangle's height,
    /// its centre lie within the width, and its distance to the plane be at most the body
    /// radius plus 1.5 (951's exit door stands 2.66 behind the wall that stops the hero).
    pub fn touched_door(&self, p: [f32; 3]) -> Option<usize> {
        self.door_within(p, contact::HERO_RADIUS + 1.5)
    }

    fn door_within(&self, p: [f32; 3], reach: f32) -> Option<usize> {
        self.planes.iter().position(|pl| {
            let d = [p[0] - pl.p[0], p[1] + 1.75 - pl.p[1], p[2] - pl.p[2]];
            let dot = |v: [f32; 3]| v[0] * d[0] + v[1] * d[1] + v[2] * d[2];
            let (a, s, u) = (dot(pl.f), dot(pl.d), dot(pl.q));
            a.abs() <= reach && (0.0..=pl.w).contains(&s) && u + 1.75 >= 0.0 && u - 1.75 <= pl.h
        })
    }

    /// Diagnostic: the probed ground height at (x, z) under ceiling c, when found.
    pub fn ground_at(&self, x: f32, z: f32, c: f32) -> Option<f32> {
        let h = self.probe(x, z, c);
        h.found.then_some(h.height)
    }

    fn probe(&self, x: f32, z: f32, c: f32) -> ProbeHit {
        self.ground.as_ref().map_or(ProbeHit { height: ground::MISS, normal: UP, kind: None, found: false, respawn: None, surface: None, zone: None }, |g| g.probe_hit(x, z, c))
    }
}

/// Diagnostic route goals for the trace.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Goal {
    Swap,
    NewNeighbour,
    Door,
    /// Within 3 units of a point (x, z) of the current world.
    Point([f32; 2]),
}

/// A world load that a door started (SPEC-0006 default door rule), serviced by the loader.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Job {
    /// Go through a door from E to pack D, whose kind is learnt when it loads: a transition is
    /// landed on at door `plane` (reversed by `reverse`); anything else is entered as an
    /// interior (no plane, entry record 0).
    FromExterior { pack: u32, plane: i32, reverse: bool },
    /// Replace the exterior with D, arriving on plane p.
    Replace(u32, i32),
    /// Reload T with the pack it held when the interior was entered.
    ReloadTransition(u32),
    /// A script's StartEnterInterior: enter the pack as an interior at entry record 0.
    EnterInterior(u32),
}

/// A region character standing in a world (SPEC-0006 kind-3 placement): drawn at its
/// placement and playing its idle action. No AI, talk, streaming by distance or kill memory.
pub struct Character {
    pub id: u64,
    pub pack: u32,
    pub slot: u8,
    /// Entity id (tag 0x12 reverse rule), when the pack's table names this character.
    pub entity: Option<usize>,
    pub name: String,
    pub position: [f32; 3],
    pub facing: [f32; 3],
    pub anim: HeroAnim,
    pub model: ModelData,
    /// The monster row of the placement (SPEC-0029 section 4).
    pub row: Option<neopets_data::monster::Row>,
    /// Hit points (from the row), and the death countdown once they reach 0.
    pub hp: i32,
    pub dying: Option<u32>,
    /// The action whose sound word was last seen, and the creature blocks the main table names.
    sound_action: Option<u32>,
    sound_blocks: Vec<u32>,
    /// The character's emote queues (cleared at spawn).
    emotes: neopets_sim::emote::Queues<(u32, u32, u32)>,
    /// The townsfolk routine (SPEC-0056) for class-6 characters; other classes only idle.
    pub townsfolk: Option<neopets_sim::npc::Townsfolk>,
    /// Class-7 Whinnies: their walker (motion record and navigation) and routine.
    pub walker: Option<neopets_sim::walker::Walker>,
    pub whinny: Option<neopets_sim::whinny::Whinny>,
    /// Class-17 common enemies (SPEC-0038) that do not fly: the AI, and the five weapon
    /// volumes (radius, length) of the row (SPEC-0029 Behavior 10.2).
    pub enemy: Option<Enemy>,
    weapons: [(f32, f32); 5],
    /// The placement's colour bytes (the talk partner's frame colour).
    colour: [u8; 3],
    /// The kind-8 record its drop placement ordinal names (SPEC-0038 Behavior 34, part B).
    drop: Option<neopets_data::region::ItemPoint>,
    root_step: [f32; 3],
    /// A script walk to a point (SPEC-0030 rows 106 and 108; SPEC-0056 classes 23 and 25), with
    /// the final facing of the orienting form.
    script_walk: Option<([f32; 3], Option<[f32; 3]>, f32)>,
    /// Under script control (PlayEntityAnim, LoopEntityAnim, script walks): its AI does not
    /// run until a release (SPEC-0030 section 4).
    pub held: bool,
    /// Not drawn, not run, not hit: deactivated (DeactivateEntity, until ActivateEntity) or
    /// with its cell flag 0 this tick (SPEC-0006 Cell flag).
    pub inactive: bool,
    pub deactivated: bool,
    /// The Juppie boss controller (class 18, SPEC-0015).
    pub boss: Option<neopets_sim::juppie::Juppie>,
    /// The Miamouse routine (class 21, SPEC-0029 Behavior 28).
    pub miamouse: Option<neopets_sim::miamouse::Miamouse>,
}

/// The parked hero (SPEC-0053 3.9): its model, animation and pools, kept while the other hero
/// is selected (the original reloads the shared hero pool on each swap; this build keeps both).
pub struct Parked {
    pub anim: HeroAnim,
    pub model: ModelData,
    pub hp: i32,
    pub mp: i32,
    pub hp_max: i32,
    pub mp_max: i32,
}

/// A built foliage patch (SPEC-0060 Behavior 1 and 2): its family, the model drawn (the near
/// model, else the far model), its centre, its placed plants and, for bushes, their states.
#[derive(Clone, Debug)]
pub struct LivePatch {
    pub kind: neopets_sim::foliage::Kind,
    pub model: u16,
    pub centre: [f32; 3],
    pub plants: Vec<neopets_sim::foliage::Placed>,
    pub bushes: Vec<neopets_sim::foliage::Bush>,
    /// The zone number (+0x20, low byte) and whether this pass's zone query passed.
    pub zone: u8,
    pub on: bool,
}

/// A script thread (SPEC-0030 section 4; SPEC-0052 Threads and waiting): the calls it has left
/// and what its blocking call waits for.
struct Thread {
    /// The world the thread's script belongs to (its entities and locators).
    pack: u32,
    calls: std::collections::VecDeque<Action>,
    wait: Option<Wait>,
}

/// What a blocking script call waits for.
#[derive(Clone, Copy, Debug, PartialEq)]
enum Wait {
    Ticks(u32),
    Fade,
    Camera,
    HeroWalk,
    EntityWalk(u32, usize),
    EntityWalks(u32, [usize; 3]),
    EntityAnim(u32, usize, u32),
    Bars,
    Dialog,
    Loads,
    /// The hero within a horizontal distance of a point.
    HeroNear([f32; 3], f32),
    /// A LevelPrompt's answer, then `on_answer` of the pack's routine.
    Choice(u8),
    /// PlayAnim with WAIT: while the object is in play-once state.
    ObjAnim(u32, usize),
    /// A wheel spin until it stops.
    WheelSpin,
    /// While a 1217 cage record is in state 4.
    Cage(usize),
    /// PlayHeroAnim with WAIT: while the hero stays in the one-shot state.
    HeroOneShot,
    /// While the current world's trigger record stays non-zero.
    Inside(u16),
    /// Until a module word of a pack is 0.
    Word(u32, u8),
}

pub struct Play {
    pub loco: Locomotion,
    pub camera: FollowCamera,
    pub hero: HeroAnim,
    pub model: ModelData,
    pub boot: BootTravel,
    /// The current world and the other resident buffer (E while T is current, T while E is).
    pub world: World,
    pub other: Option<World>,
    /// A pack the pending step wants resident and whose kind is not known yet, or which must be
    /// loaded: serviced by the app between ticks (`stream`).
    pub wanted: Option<u32>,
    kinds: HashMap<u32, Kind>,
    root_step: [f32; 3],
    /// A Cross press waiting for the next tick.
    pub jump_latch: bool,
    /// An R3 press waiting for the next tick.
    pub r3_latch: bool,
    /// An L3 press waiting for the next tick (it re-arms every HUD group; SPEC-0023 section 4).
    pub l3_latch: bool,
    /// Ticks run so far.
    pub ticks: u64,
    /// Number of world swaps so far.
    pub swaps: u32,
    /// Door prompt id posted by this tick's door step, and the one the chooser shows (posted on
    /// the previous tick); the door-confirm flag (SPEC-0006 Door prompt and confirm).
    door_prompt: Option<u16>,
    pub prompt_shown: Option<u16>,
    confirm: bool,
    /// A Square press waiting for the next tick.
    pub square_latch: bool,
    /// A door's world load in progress, and the pack T held when an interior was entered.
    pub job: Option<Job>,
    interior_t: Option<u32>,
    /// What the last door did.
    pub last_door: String,
    /// Story flags, blackboard and items read by the level handlers.
    pub state: GameState,
    /// The dialogue line a handler showed, and the tick it was shown.
    pub message: Option<(u16, u64)>,
    /// Square was pressed this tick (a script door starts only on a press edge).
    square_now: bool,
    /// Square's press edge as the script button query reads it (also while the hero is held).
    button_square: bool,
    /// The Talk label is shown this tick, and the talked character (the speaker of its lines).
    pub talk_prompt: bool,
    pub talk_partner: Option<String>,
    /// The character in the conversation (base state 25), by id.
    talk_with: Option<u64>,
    /// The shared dialogue box is open (SPEC-0022: it closes only on a Square press; the hero
    /// stays in the talking state meanwhile).
    pub dialogue_open: bool,
    /// Scene staging (SPEC-0052): the script threads, the camera play, the screen fade, the
    /// hero's scripted walk (target, final facing, last distance), the script hold, and the
    /// region loads the loader still has out.
    threads: Vec<Thread>,
    pub cut: Option<neopets_sim::scene::CameraPlay>,
    pub fade: neopets_sim::scene::Fade,
    pub bars: neopets_sim::scene::Bars,
    hero_walk: Option<([f32; 3], Option<[f32; 3]>, f32)>,
    pub held: bool,
    pub loads_out: usize,
    /// The pack last made current (the "last active world" of the next activation).
    seen_pack: u32,
    /// Shops (SPEC-0050 sections 3 and 9): the buyable item this tick (its pool index), the
    /// "Buy <name>" caption, the price caption's two lines (value, shown counter), and the boot
    /// texts (the "Buy %s" template at 0x319528 and "Exorbitant!" at 0x315A48).
    pub buyable: Option<usize>,
    buy_caption: Option<Vec<u8>>,
    pub price_lines: [(i32, u32); 2],
    buy_template: Vec<u8>,
    pub exorbitant: Vec<u8>,
    /// Tor's attack step (SPEC-0036), while state 13 runs.
    pub attack: Option<neopets_sim::attack::TorAttack>,
    /// An attack press waiting for the next tick.
    pub attack_latch: bool,
    /// The equipped weapon (item id), the item table, the hero's hit list and the weapon axis
    /// of the previous tick (D2 of the swept test).
    /// The heroes' family entries and Mote words (SPEC-0021 Party inventory lists, Equipment
    /// slots); `weapon` and `shield` mirror Tor's entries for the combat code.
    pub equipment: neopets_sim::inventory::Equipment,
    /// The Tattered Cloak is worn (the cloak swap; SPEC-0021 Tattered Cloak, partly).
    pub cloak_worn: bool,
    pub weapon: Option<u16>,
    /// The equipped shield (item id; Tor's shields are type 5, SPEC-0036).
    pub shield: Option<u16>,
    pub items: neopets_data::item::ItemTable,
    reach_tor: Vec<f32>,
    /// The Magic table's cost and power words (SPEC-0036 Magic).
    magic: neopets_sim::mote::Magic,
    /// The last Mote charge or magic payment, for the status line.
    pub last_mote: String,
    /// The last status applied to an enemy, for the status line.
    pub last_status: String,
    /// The stick gesture machine and its words (SPEC-0033 Behavior 52).
    gesture: neopets_sim::gesture::Gesture,
    gesture_words: neopets_sim::gesture::Words,
    /// The Miamouse catch switch, meshes and count, and the carried Miamouse (character id).
    pub catch: neopets_sim::miamouse::Catch,
    carried: Option<u64>,
    /// Hero switching (SPEC-0053): the selected hero (0 Tor, 1 Roberta), the parked hero, the
    /// swap mode, the running swap task and the L1 and R1 press edges kept for the next tick.
    pub hero_index: usize,
    pub parked: Option<Parked>,
    pub swap_mode: u32,
    pub swap: Option<neopets_sim::swap::Task>,
    pub l1_latch: bool,
    pub r1_latch: bool,
    pub last_swap: String,
    /// Roberta's attack step, her six shot slots and the shot rows (SPEC-0036).
    r_attack: Option<neopets_sim::roberta::RobertaAttack>,
    pub hero_shots: Vec<neopets_sim::roberta::Shot>,
    shot_rows: neopets_sim::roberta::Rows,
    /// Forced binds waiting for the embedded region: (pack, ordinal).
    rebind: Vec<(u32, usize)>,
    /// Entities deactivated before their characters loaded.
    pending_off: std::collections::HashSet<(u32, usize)>,
    /// The Juppie tuning words and the boss hits of this tick's swing (character id, damage).
    juppie_words: neopets_sim::juppie::Words,
    boss_hits: Vec<(u64, i32)>,
    pub last_boss: String,
    /// The live petpet (SPEC-0047 class 34), its character once its package has loaded, the
    /// pending save values, the bonus words and the low-time mark.
    pub petpet: Option<neopets_sim::petpet::Live>,
    pet_char: Option<u64>,
    pet_spawn: Option<[f32; 3]>,
    pub pet_pending: neopets_sim::petpet::Pending,
    pet_bonus: neopets_sim::petpet::Bonus,
    pet_low: i32,
    pub last_pet: String,
    /// The live Mote power-attack objects and their boot words (SPEC-0036 Charm attack hits).
    mote_objects: Vec<neopets_sim::charm::Object>,
    charm_words: neopets_sim::charm::Words,
    hit_list: Vec<u64>,
    last_axis: Option<[f32; 3]>,
    /// The last hit, for the status line.
    pub last_hit: String,
    /// The hero's health and maximum (SPEC-0036 Start values), the band factor tables
    /// (SPEC-0038 Behavior 18) and the last hit taken.
    pub hero_hp: i32,
    pub hero_max: i32,
    bands: (Vec<f32>, Vec<f32>),
    pub last_hurt: String,
    /// The enemy projectile pool (SPEC-0029 Behavior 9).
    pub shots: neopets_sim::projectile::Pool,
    /// World items (SPEC-0021): one pool per resident world (the wallet and the party's item
    /// counts are the level state's `neopoints` and `items`), the once-per-area pickups (id, world), the luck value,
    /// the coin denominations, ids and motion words, enemy payouts due this tick, and the last
    /// pickup.
    pub pools: HashMap<u32, neopets_sim::items::ItemPool>,
    once: std::collections::HashSet<(u16, u32)>,
    pub luck: i32,
    coins: (Vec<i32>, Vec<u16>, Option<neopets_sim::items::CoinWords>),
    payouts: Vec<(u32, [f32; 3], Option<neopets_data::region::ItemPoint>)>,
    pub last_pickup: String,
    /// The quick-item panel (SPEC-0021), its candidate rows and the heal tables; the hero's
    /// magic (pool B); the panel's button edges waiting for the next tick.
    pub quick: neopets_sim::quick::QuickPanel,
    quick_rows: [[i32; 8]; 4],
    heal_tables: (Vec<i32>, Vec<i32>),
    pub hero_mp: i32,
    pub hero_mp_max: i32,
    /// The hero's 14 status timers (SPEC-0036).
    pub statuses: neopets_sim::status::Statuses,
    /// The in-play HUD's bar rules (SPEC-0023) and their boot words; the monster bar, its words,
    /// and the font and letter spacing of its name text (the value text of key 0x32).
    pub hud: neopets_sim::hud::Hud,
    hud_words: neopets_sim::hud::Words,
    pub monster_bar: neopets_sim::hud::MonsterBar,
    monster_words: neopets_sim::hud::MonsterWords,
    name_font: Option<(neopets_data::frontend::Font, i32)>,
    pub panel_latch: neopets_sim::quick::PanelInput,
    /// The shared dialogue box (SPEC-0022) with its speaker name and frame colour, the hero
    /// name colours (design table 10), the "Next" and "Finish" labels (table 13) and the box's
    /// boot words.
    pub textbox: neopets_sim::textbox::TextBox,
    pub speaker: (Vec<u8>, [u8; 3]),
    hero_colour: [u8; 3],
    pub box_labels: (Vec<u8>, Vec<u8>),
    /// The action labels of design table 13 (the Square captions).
    pub labels: Vec<Vec<u8>>,
    pub box_words: BoxWords,
    /// The death flow (SPEC-0055): death mode, timers E and F, flag E and the game-over page;
    /// the state-60 penalty due on the next tick; a Continue that wants the placeholder restart.
    pub death: neopets_sim::death::DeathFlow,
    penalty_due: bool,
    pub continued: bool,
    /// The game's random generator (SPEC-0008): one stream, seeded once.
    pub rng: neopets_sim::rng::Rng,
    /// Effects (SPEC-0034): the resource-9 definition table, its material images, the shared
    /// pools and the footstep dust table (28 ids at 0x3150A8).
    pub effect_table: std::sync::Arc<neopets_sim::effect::EffectTable>,
    pub effect_images: std::sync::Arc<Vec<neopets_data::effect::GroupImage>>,
    pub effects: neopets_sim::effect::Effects,
    dust_ids: Vec<i32>,
    /// The flight effect objects (ids 0 and 1) of each projectile slot, with the shot's start
    /// point that tells a new shot in the same slot.
    shot_fx: Vec<Option<([f32; 3], [Option<usize>; 2])>>,
    /// Each character's running clip effects: its list index and an effect object per entry.
    clip_fx: HashMap<u64, (Option<usize>, Vec<Option<usize>>)>,
    pub last_fx: String,
    /// The save-point glow (SPEC-0034 section 36): the effect objects of 400 and 404 and the
    /// world they belong to.
    save_glow: Option<(u32, [usize; 2])>,
    /// Placed effect objects made by StartEffect, by (pack, slot).
    placed_fx: HashMap<(u32, usize), usize>,
    /// Roberta's hold effect and leap glow (SPEC-0036): their timers and effect objects.
    r_displays: neopets_sim::roberta::Displays,
    r_fx: [Option<usize>; 2],
    /// Character light states (SPEC-0046 Character lights): the hero's (key None) and each
    /// region character's, by id.
    char_lights: HashMap<Option<u64>, neopets_sim::charlight::State>,
    /// The drop direction of a dig reward while it pays out (SPEC-0051 section 10 step 11).
    dig_drop: Option<[f32; 3]>,
    /// Entities that died in a resident world this load (pack, entity id): the kill memory the
    /// IsAI_Dead test falls back on (cleared when the world is released).
    killed: std::collections::HashSet<(u32, usize)>,
    /// Placed records with their active flag set (the switch path; SPEC-0028 section 9).
    placed_on: std::collections::HashSet<(u32, usize)>,
    /// The zone manager (SPEC-0006 Zone manager); its grid is the current world's.
    pub zones: neopets_sim::zone::Zones,
    /// The prize wheel's running spin (its pack) and each world's display mask word
    /// (SetWorldDisplayMask; the display word stores 2M).
    wheel_spin: Option<(u32, neopets_sim::wheel::Spin)>,
    pub display_mask: HashMap<u32, u32>,
    /// Status displays by character (the hero is u64::MAX): the display, its effect object and
    /// last tick's active statuses.
    displays: HashMap<u64, (neopets_sim::status::Display, Option<usize>, [bool; 14])>,
    /// Colour-override slot 0 per display (SPEC-0046 draw step 6; SPEC-0034 section 21).
    overrides: HashMap<u64, neopets_sim::status::Override>,
    /// The resident worlds' foliage (SPEC-0060), by pack.
    pub foliage: HashMap<u32, Vec<LivePatch>>,
    /// Region characters of the resident worlds, the region each (pack, slot) should hold, and
    /// slot loads waiting for the region loader.
    pub characters: Vec<Character>,
    slot_regions: HashMap<(u32, u8), u32>,
    pub region_wanted: Vec<(u32, u8, u32)>,
    next_character: u64,
    /// Set once the hero's meshes exist.
    pub spawned: bool,
    /// A front-end page other than play is up (the world is frozen and the HUD hidden).
    pub menu_open: bool,
    /// The save game (SPEC-0025): the world key table (keys, saved ids), the growth words, the
    /// item text and location tables, the save-point state, whether the "Save Game" prompt
    /// shows this tick, a save-screen opening for the front end, the hero's restart point, the
    /// last named map area, the session snapshot and a Continue that wants it applied.
    pub world_keys: (Vec<u32>, Vec<u32>),
    pub growth: neopets_sim::save::Growth,
    design: neopets_data::design::DesignTables,
    save_points: neopets_sim::save::SavePoints,
    pub save_prompt: bool,
    pub save_open: bool,
    pub restart: Option<([f32; 3], [f32; 3])>,
    last_area: Option<i32>,
    pub snapshot: Option<neopets_sim::save::Slot>,
    pub reload_wanted: bool,
    /// The music controller's stream channels (SPEC-0009 Streams) and this tick's selection.
    pub music: neopets_sim::music::Streams,
    pub music_selection: Option<neopets_sim::music::Selection>,
    /// The bank music's game side and the intensity inputs (SPEC-0009).
    pub bank_music: neopets_sim::music::BankMusic,
    pub intensity: neopets_sim::music::Intensity,
    /// The cue call's game side (SPEC-0031): slots, voices and requests for the audio side;
    /// the instances the driver finished since the last pass; the footstep rule's remembered
    /// (clip, frame) and the last tick's hero state (for landings).
    pub sound: neopets_sim::sound::SoundManager,
    pub sound_finished: Vec<usize>,
    foot_frame: Option<(usize, i32)>,
    last_state: u8,
    /// Stingers (SPEC-0009 Effect and stinger index): requests for the audio side, whether one
    /// is playing (set by the audio side), the tick stinger 4 last started, and the progress
    /// flags of the last tick (a rising flag plays its table-2 stinger).
    pub stinger_requests: Vec<i32>,
    /// Emote clips to play: (bank resource, start and length in 2,048-byte units).
    pub emote_requests: Vec<(u32, u32, u32)>,
    /// The hero's emote queues (SPEC-0040 Behavior 22); each character holds its own.
    hero_emotes: neopets_sim::emote::Queues<(u32, u32, u32)>,
    pub stinger_playing: bool,
    stinger4: u64,
    /// The live options and gains (SPEC-0035 Behavior 3), kept in step with the front end.
    pub options: neopets_sim::options::Options,
    flags_seen: std::collections::BTreeSet<u16>,
}

/// The footstep surface offset o(k) by ground material code (SPEC-0031 Footsteps).
fn surface_offset(k: u32) -> Option<i32> {
    match k {
        0 | 18 => Some(0),
        1 => Some(2),
        3 => Some(3),
        4 => Some(4),
        5 => Some(9),
        6 => Some(1),
        7 => Some(8),
        8 | 11 => Some(7),
        9 => Some(5),
        10 => Some(6),
        _ => None,
    }
}

/// The landing cue by ground material code (SPEC-0031 Landings).
fn landing_cue(k: u32) -> Option<i32> {
    match k {
        0 => Some(66),
        1 => Some(65),
        3 => Some(62),
        4 => Some(63),
        5 => Some(67),
        6 => Some(60),
        7 => Some(68),
        8 | 11 => Some(64),
        9 => Some(61),
        10 => Some(69),
        _ => None,
    }
}

/// The five stat-up items with a once-per-area set, in slot order (SPEC-0025 Once-per-area sets).
const ONCE_ITEMS: [u16; 5] = [19, 20, 151, 174, 154];

impl Play {
    /// Load a hero outfit and a world pack's ground and spawn point. Pack 909 starts as the new
    /// game does, with 691 resident in T.
    pub fn load(disc: &mut Disc, index: &ArchiveIndex, pack: u32, outfit: u32) -> Result<Self, String> {
        Self::load_with_flags(disc, index, pack, outfit, &[])
    }

    /// As `load`, with story flags set before the start world's Init (a diagnostic stand-in for
    /// a start point's flags).
    pub fn load_with_flags(disc: &mut Disc, index: &ArchiveIndex, pack: u32, outfit: u32, flags: &[u16]) -> Result<Self, String> {
        let model = crate::archive_view::model(disc, index, outfit)?.ok_or("Outfit has no model")?;
        let hero = crate::archive_view::hero(disc, index, outfit)?.ok_or("Not a hero outfit (504–519)")?;
        // Roberta (outfit 512, her first; which outfit the save selects is not read) waits parked
        // with her start pools (SPEC-0036 Start values; the growth words' starts).
        let roberta = match (crate::archive_view::model(disc, index, 512), crate::archive_view::hero(disc, index, 512)) {
            (Ok(Some(m)), Ok(Some(a))) => Some((m, a)),
            _ => None,
        };
        let boot = BootTravel::from_disc(disc)?;
        let items = neopets_data::item::ItemTable::parse(&index.read(disc, neopets_data::item::ITEM_CONFIG)?)?;
        let boot_bytes = neopets_data::archive::verified_boot_program(disc)?;
        let reach_tor = neopets_data::archive::boot_u32_table(&boot_bytes, neopets_data::item::REACH_TOR, 11)?
            .into_iter()
            .map(f32::from_bits)
            .collect();
        let hero_max = neopets_data::archive::boot_u32_table(&boot_bytes, neopets_data::item::HEALTH_TOR, 1)?[0] as i32;
        let floats = |at: u32| -> Result<Vec<f32>, String> {
            Ok(neopets_data::archive::boot_u32_table(&boot_bytes, at, 7)?.into_iter().map(f32::from_bits).collect())
        };
        let bands = (floats(neopets_data::monster::BAND_F1)?, floats(neopets_data::monster::BAND_F2)?);
        let words = |at: u32, n: usize| neopets_data::archive::boot_u32_table(&boot_bytes, at, n);
        let rows = words(neopets_data::item::QUICK_ROWS, 32)?;
        let quick_rows: [[i32; 8]; 4] = std::array::from_fn(|r| std::array::from_fn(|k| rows[8 * r + k] as i32));
        let heal_tables = (
            words(neopets_data::item::HEAL_A, 4)?.into_iter().map(|w| w as i32).collect(),
            words(neopets_data::item::HEAL_B, 2)?.into_iter().map(|w| w as i32).collect(),
        );
        let hero_mp_max = words(neopets_data::item::MAGIC_TOR, 1)?[0] as i32;
        // SPEC-0023 Layout: pulse half-length, red and green tints, bar offset tables A and B,
        // and the hero divider table (bases, then steps).
        let w1 = |at: u32| words(at, 1).map(|v| v[0]);
        let dividers = words(neopets_data::item::HEALTH_TOR, 8)?;
        let float = |at: u32| words(at, 1).map(|v| f32::from_bits(v[0]));
        let charm_words = {
            use neopets_sim::charm::*;
            Words {
                orbit_radius: float(ORBIT_RADIUS)?,
                orbit_seconds: float(ORBIT_SECONDS)?,
                nova_radius: float(NOVA_RADIUS)?,
                wind_radius: float(WIND_RADIUS)?,
                wind_height: float(WIND_HEIGHT)?,
            }
        };
        let bw = words(neopets_sim::petpet::BONUS_AT, 4)?;
        let pet_bonus = neopets_sim::petpet::Bonus { damage: f32::from_bits(bw[0]), intake: f32::from_bits(bw[1]), hp: f32::from_bits(bw[2]), mp: f32::from_bits(bw[3]) };
        let pet_low = (float(neopets_sim::petpet::LOW_TIME_WORD)? * 3600.0) as i32;
        let shot_rows = neopets_sim::roberta::Rows::from_words(&words(neopets_sim::roberta::ROWS_AT, 55)?);
        let gesture_words = {
            use neopets_sim::gesture::*;
            let int = |at: u32| words(at, 1).map(|v| v[0]);
            Words { n: int(TOR_CCW_STEPS)?, w: int(TOR_SPIN_HOLD)?, flick: int(FLICK_HOLD)? }
        };
        let juppie_words = {
            use neopets_sim::juppie::*;
            let int = |at: u32| words(at, 1).map(|v| v[0] as i32);
            let cd = words(COOLDOWNS, 6)?;
            let counts = words(COUNTS, 3)?;
            let table = words(STUN_TABLE, 3)?;
            Words {
                slam_min: float(SLAM_MIN)?,
                slam_max: float(SLAM_MAX)?,
                slam_chance: int(SLAM_CHANCE)?,
                slam_damage: int(SLAM_DAMAGE)?,
                stun_min: float(STUN_MIN)?,
                stun_max: float(STUN_MAX)?,
                counts: std::array::from_fn(|k| counts[k] as i32),
                spit_distance: float(SPIT_DISTANCE)?,
                spit_chance: int(SPIT_CHANCE)?,
                cooldown: std::array::from_fn(|k| (f32::from_bits(cd[2 * k]), f32::from_bits(cd[2 * k + 1]))),
                impact: float(IMPACT)?,
                push: float(SLAM_PUSH)?,
                cone: float(LOCATOR_CONE)?,
                locator_distance: float(LOCATOR_DISTANCE)?,
                stun_table: std::array::from_fn(|k| table[k] as i32),
            }
        };
        let (effect_table, effect_images) = {
            let bytes = index.read(disc, neopets_data::effect::RESOURCE)?;
            let chain = neopets_data::resource::walk_chunks(&bytes, None)?;
            let range = neopets_data::effect::table_in_chain(&bytes, &chain.chunks).ok_or("Resource 9 has no effect table")?;
            let table = neopets_sim::effect::EffectTable::parse(&bytes[range])?;
            (std::sync::Arc::new(table), std::sync::Arc::new(neopets_data::effect::group_images(&bytes, &chain.chunks)?))
        };
        let dust_ids: Vec<i32> = words(0x0031_50A8, 28)?.into_iter().map(|w| w as i32).collect();
        let magic = neopets_sim::mote::Magic::new(words(neopets_sim::mote::MAGIC_AT, neopets_sim::mote::MAGIC_WORDS)?.into_iter().map(|w| w as i32).collect());
        let design = index.read(disc, neopets_data::design::RESOURCE).ok().and_then(|b| neopets_data::design::DesignTables::parse(&b).ok()).unwrap_or_default();
        let hero_colour = design.hero_colour(0).unwrap_or([128; 3]);
        let box_labels = (design.label(0x10).unwrap_or_default(), design.label(0x11).unwrap_or_default());
        let labels: Vec<Vec<u8>> = (0..36).map(|i| design.label(i).unwrap_or_default()).collect();
        let colour = |at: u32| words(at, 1).map(|v| v[0].to_le_bytes());
        let prompts = w1(0x002D_CC40)?;
        let box_words = BoxWords {
            margin: w1(0x002D_CC18)? as i32,
            width_term: w1(0x002D_CC1C)? as i32,
            height_terms: (w1(0x002D_CC20)? as i32, w1(0x002D_CC24)? as i32),
            prompt_x: ((prompts & 0xFFFF) as i32, (prompts >> 16) as i32),
            wrap: w1(0x002D_CC44)? as i32,
            anchor: w1(0x002D_CC48)? as i32,
            name_colour: colour(0x0031_59F0)?,
            body_colour: colour(0x0031_59F8)?,
            prompt_colour: colour(0x0031_5A38)?,
        };
        let monster_words = neopets_sim::hud::MonsterWords {
            divisor: f32::from_bits(w1(0x002D_D01C)?),
            max: w1(0x002D_D020)? as i32,
            min: w1(0x002D_D024)? as i32,
            margin: w1(0x002D_D028)? as i32,
            cap: w1(0x002D_D02C)? as i32,
            plate: w1(0x002D_D030)? as i32,
        };
        let name_font = index.read(disc, neopets_data::frontend::RESOURCE).ok().and_then(|b| neopets_data::frontend::Frontend::parse(&b).ok()).and_then(|f| {
            let node = f.nodes.get(f.key(0x32)?)?;
            let v = f.values.get(&node.data)?;
            Some((f.fonts.get(v.font as usize)?.clone(), v.spacing as i32))
        });
        let hud_words = neopets_sim::hud::Words {
            half: f32::from_bits(w1(0x002D_CDDC)?),
            red: w1(0x002D_CDE8)?.to_le_bytes(),
            green: w1(0x002D_CDE0)?.to_le_bytes(),
            cap: w1(0x002D_CE00)? as u16 as i16 as i32,
            inset: w1(0x002D_CE08)? as u16 as i16 as i32,
            base: (dividers[0] as i32, dividers[2] as i32),
            step: (dividers[4] as i32, dividers[6] as i32),
        };
        let coins = (
            words(neopets_data::item::COIN_DENOMINATIONS, 5)?.into_iter().map(|w| w as i32).collect(),
            words(neopets_data::item::COIN_IDS, 5)?.into_iter().map(|w| w as u16).collect(),
            neopets_sim::items::CoinWords::from_words(&words(neopets_data::item::COIN_WORDS, 8)?.into_iter().map(f32::from_bits).collect::<Vec<_>>()),
        );
        let world_keys = (words(0x0031_EFA8, 193)?, words(0x0031_F2B0, 193)?);
        let growth = neopets_sim::save::Growth::from_words(&words(neopets_data::item::HEALTH_TOR, 12)?).unwrap_or_default();
        let parked = roberta.map(|(model, anim)| Parked { anim, model, hp: growth.start[1], mp: growth.start[3], hp_max: growth.start[1], mp_max: growth.start[3] });
        let (world, bytes) = World::load(disc, index, &boot, pack)?;
        let chain = resource::walk_chunks(&bytes, None)?;
        let (spawn, facing) = ground::spawn(&bytes, &chain.chunks).unwrap_or(([0.0; 3], [1.0, 0.0, 0.0]));
        let y = world.ground.as_ref().map_or(spawn[1], |g| ground::spawn_height(g, spawn));
        let other = if pack == START_EXTERIOR {
            Some(World::load(disc, index, &boot, START_TRANSITION)?.0)
        } else if world.kind == Kind::Interior {
            // Diagnostic start in an interior: the exterior its cells name is put in E, as if
            // the hero had entered from it.
            let v = world.ground.as_ref().map_or(-1, |g| g.cell_value(spawn[0], spawn[2]));
            let id = world.neighbours.get(v);
            let slot = boot.areas.world_ids.iter().position(|&w| id >= 0 && w == id as u32);
            match slot.map(|s| boot.areas.packs[s]) {
                Some(e) => Some(World::load(disc, index, &boot, e)?.0),
                None => None,
            }
        } else {
            None
        };
        let loco = Locomotion::spawn([spawn[0], y, spawn[2]], facing);
        let camera = FollowCamera::attach(
            &followed(&loco, &world),
            |x, z, c| world.probe(x, z, c).height,
            |e| push_eye(&world, e),
        );
        let mut kinds = HashMap::new();
        kinds.insert(world.pack, world.kind);
        if let Some(o) = &other {
            kinds.insert(o.pack, o.kind);
        }
        let mut play = Self {
            loco,
            camera,
            hero,
            model,
            boot,
            world,
            other,
            wanted: None,
            kinds,
            root_step: [0.0; 3],
            jump_latch: false,
            r3_latch: false,
            l3_latch: false,
            ticks: 0,
            swaps: 0,
            door_prompt: None,
            prompt_shown: None,
            confirm: false,
            square_latch: false,
            job: None,
            interior_t: None,
            last_door: String::new(),
            state: GameState::default(),
            message: None,
            square_now: false,
            button_square: false,
            talk_prompt: false,
            talk_partner: None,
            talk_with: None,
            dialogue_open: false,
            threads: Vec::new(),
            cut: None,
            fade: Default::default(),
            bars: Default::default(),
            hero_walk: None,
            held: false,
            loads_out: 0,
            seen_pack: 0,
            equipment: Default::default(),
            cloak_worn: false,
            buyable: None,
            buy_caption: None,
            price_lines: [(0, 0); 2],
            buy_template: boot_string(&boot_bytes, 0x0031_9528),
            exorbitant: boot_string(&boot_bytes, 0x0031_5A48),
            rng: neopets_sim::rng::Rng::seeded(clock_seed()),
            effect_table,
            effect_images,
            effects: neopets_sim::effect::Effects::new(),
            dust_ids,
            shot_fx: vec![None; neopets_sim::projectile::SLOTS],
            clip_fx: HashMap::new(),
            last_fx: String::new(),
            save_glow: None,
            placed_fx: HashMap::new(),
            r_displays: Default::default(),
            r_fx: [None; 2],
            char_lights: HashMap::new(),
            dig_drop: None,
            placed_on: Default::default(),
            killed: Default::default(),
            zones: Default::default(),
            wheel_spin: None,
            display_mask: HashMap::new(),
            displays: HashMap::new(),
            overrides: HashMap::new(),
            foliage: HashMap::new(),
            attack: None,
            attack_latch: false,
            weapon: None,
            shield: None,
            items,
            reach_tor,
            magic,
            last_mote: String::new(),
            mote_objects: Vec::new(),
            charm_words,
            last_status: String::new(),
            rebind: Vec::new(),
            r_attack: None,
            hero_shots: Vec::new(),
            shot_rows,
            hero_index: 0,
            parked,
            swap_mode: neopets_sim::swap::rule_m(flags.contains(&220), flags.contains(&248)),
            swap: None,
            l1_latch: false,
            r1_latch: false,
            last_swap: String::new(),
            catch: Default::default(),
            carried: None,
            gesture: Default::default(),
            gesture_words,
            pending_off: Default::default(),
            juppie_words,
            boss_hits: Vec::new(),
            last_boss: String::new(),
            petpet: None,
            pet_char: None,
            pet_spawn: None,
            pet_pending: Default::default(),
            pet_bonus,
            pet_low,
            last_pet: String::new(),
            hit_list: Vec::new(),
            last_axis: None,
            last_hit: String::new(),
            hero_hp: hero_max,
            hero_max,
            bands,
            last_hurt: String::new(),
            shots: Default::default(),
            pools: HashMap::new(),
            once: Default::default(),
            luck: 0,
            coins,
            payouts: Vec::new(),
            last_pickup: String::new(),
            quick: Default::default(),
            quick_rows,
            heal_tables,
            hero_mp: hero_mp_max,
            hero_mp_max,
            statuses: Default::default(),
            hud: Default::default(),
            hud_words,
            monster_bar: Default::default(),
            monster_words,
            name_font,
            panel_latch: Default::default(),
            textbox: Default::default(),
            speaker: (Vec::new(), [128; 3]),
            hero_colour,
            box_labels,
            labels,
            box_words,
            death: neopets_sim::death::DeathFlow::new(),
            penalty_due: false,
            continued: false,
            characters: Vec::new(),
            slot_regions: HashMap::new(),
            region_wanted: Vec::new(),
            next_character: 0,
            spawned: false,
            menu_open: false,
            world_keys,
            growth,
            design,
            save_points: Default::default(),
            save_prompt: false,
            save_open: false,
            restart: None,
            last_area: None,
            snapshot: None,
            reload_wanted: false,
            music: Default::default(),
            music_selection: None,
            bank_music: Default::default(),
            intensity: Default::default(),
            sound: Default::default(),
            sound_finished: Vec::new(),
            foot_frame: None,
            last_state: 1,
            stinger_requests: Vec::new(),
            emote_requests: Vec::new(),
            hero_emotes: Default::default(),
            stinger_playing: false,
            stinger4: 0,
            options: Default::default(),
            flags_seen: Default::default(),
        };
        // The start world's Init, then its activate-world routine (SPEC-0006 Make current),
        // which for 909 loads the farm family's slots.
        // SPEC-0049 New game at start index 0: no stage rows, then flag 707 (a loaded save
        // replaces the flags later).
        play.state.set(707, true);
        for &f in flags {
            play.state.set(f, true);
        }
        let mut init = level::on_init(play.world.pack, &mut play.state);
        init.extend(play.group_setup());
        play.apply(&init);
        play.activated();
        // Flags already set when play starts are not risings (no stingers for them).
        play.flags_seen = play.state.flags.clone();
        Ok(play)
    }

    /// Grid bounds (x min, z min, x max, z max) of the current and other worlds.
    pub fn bounds(&self) -> Vec<(u32, [f32; 4])> {
        std::iter::once(&self.world)
            .chain(self.other.as_ref())
            .filter_map(|w| Some((w.pack, w.ground.as_ref()?.bounds())))
            .collect()
    }

    /// Diagnostic: the nearest point (8-unit lattice) to the hero where the current world has
    /// no ground and the other world has, that is, where a walk would swap worlds.
    pub fn nearest_swap_point(&self) -> Option<[f32; 2]> {
        let o = self.other.as_ref()?;
        let [x0, z0, x1, z1] = o.ground.as_ref()?.bounds();
        let p = self.loco.position();
        let mut best: Option<([f32; 2], f32)> = None;
        let mut z = z0;
        while z < z1 {
            let mut x = x0;
            while x < x1 {
                let d = (x - p[0]).powi(2) + (z - p[2]).powi(2);
                if best.is_none_or(|b| d < b.1) && !self.world.probe(x, z, 1.0e6).found && o.probe(x, z, 1.0e6).found {
                    best = Some(([x, z], d));
                }
                x += 8.0;
            }
            z += 8.0;
        }
        best.map(|b| b.0)
    }

    /// Diagnostic: a walkable route (4-unit lattice, breadth first) from the hero to the nearest
    /// reachable point where the current world has no ground and the other world has. A step
    /// is walkable when both ends have ground in either world, the ground rises at most 1.5
    /// and no frame record of the current world crosses it 1.5 above the ground.
    pub fn route_to(&self, max_nodes: usize, goal: Goal) -> Option<Vec<[f32; 2]>> {
        let o = self.other.as_ref();
        let p = self.loco.position();
        let step = 4.0;
        let key = |x: f32, z: f32| ((x / step).round() as i32, (z / step).round() as i32);
        let ground = |x: f32, z: f32, y: f32| {
            let h = self.world.probe(x, z, y + 2.5);
            if h.found {
                return Some((h.height, false));
            }
            let h = o?.probe(x, z, y + 2.5);
            h.found.then_some((h.height, true))

        };
        let start = key(p[0], p[2]);
        let mut came = HashMap::new();
        let mut queue = std::collections::VecDeque::new();
        came.insert(start, (start, p[1]));
        queue.push_back(start);
        while let Some(k) = queue.pop_front() {
            if came.len() > max_nodes {
                eprintln!("route search: node limit {max_nodes} reached");
                return None;
            }
            let (_, y) = came[&k];
            let (x, z) = (k.0 as f32 * step, k.1 as f32 * step);
            for (dx, dz) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
                let n = (k.0 + dx, k.1 + dz);
                if came.contains_key(&n) {
                    continue;
                }
                let (nx, nz) = (n.0 as f32 * step, n.1 as f32 * step);
                let Some((h, other)) = ground(nx, nz, y) else { continue };
                if h > y + 1.5 {
                    continue;
                }
                let a = [x, y + 1.5, z];
                let b = [nx, h + 1.5, nz];
                let blocked = self.world.ground.as_ref().is_some_and(|g| {
                    g.frames_at(x, z).iter().chain(g.frames_at(nx, nz).iter()).any(|f| f.crossed_by(a, b))
                });
                if blocked {
                    continue;
                }
                came.insert(n, (k, h));
                let reached = match goal {
                    Goal::NewNeighbour => !other && self.neighbour_at(nx, nz).is_some_and(|t| !self.resident(t)),
                    Goal::Swap => other,
                    Goal::Door => !other && !self.world.scripted_doors && self.world.door_within([nx, h, nz], 3.0).is_some(),
                    Goal::Point(t) => !other && (t[0] - nx).hypot(t[1] - nz) <= 3.0,
                };
                if reached {
                    let mut route = vec![[nx, nz]];
                    // A door route ends on the middle of the door's foot.
                    if let (Goal::Door, Some(k)) = (goal, self.world.door_within([nx, h, nz], 3.0)) {
                        let pl = self.world.planes[k];
                        route.insert(0, [pl.p[0] + 0.5 * pl.w * pl.d[0], pl.p[2] + 0.5 * pl.w * pl.d[2]]);
                    }
                    let mut c = k;
                    while c != start {
                        route.push([c.0 as f32 * step, c.1 as f32 * step]);
                        c = came[&c].0;
                    }
                    route.reverse();
                    return Some(route);
                }
                queue.push_back(n);
            }
        }
        None
    }

    /// Diagnostic: put the hero 3 units in front of door k of the current world (on the side
    /// with ground), facing it; returns the middle of the door's foot to walk to.
    pub fn stand_before_door(&mut self, k: usize) -> Option<[f32; 2]> {
        let pl = *self.world.planes.get(k)?;
        let m = [pl.p[0] + 0.5 * pl.w * pl.d[0], pl.p[1], pl.p[2] + 0.5 * pl.w * pl.d[2]];
        for side in [1.0f32, -1.0] {
            let (x, z) = (m[0] + 3.0 * side * pl.f[0], m[2] + 3.0 * side * pl.f[2]);
            if let Some(y) = self.world.ground_at(x, z, m[1] + 5.0) {
                self.arrive(([x, y, z], [-side * pl.f[0], 0.0, -side * pl.f[2]]));
                return Some([m[0], m[2]]);
            }
        }
        None
    }

    /// Whether the other world is drawn with the current one (SPEC-0006 neighbour-world test
    /// with q = the other world's pack).
    pub fn other_drawn(&self) -> bool {
        self.other.as_ref().is_some_and(|o| self.boot.neighbour_ok(self.world.pack, self.world.kind, o.pack))
    }

    /// Whether a loaded world of this kind would be streamed into the other buffer now.
    pub fn streams(&self, kind: Kind) -> bool {
        let buffer = match kind {
            Kind::Transition => Buffer::Transition,
            Kind::Exterior => Buffer::Exterior,
            Kind::Interior => Buffer::Interior,
        };
        buffers::stream_buffer(self.world.buffer, kind == Kind::Transition) == Some(buffer)
    }

    /// Take a loaded world (from `stream` or a loader thread): remember its kind and make it
    /// the other world when the buffer rule streams it and it is not already resident.
    pub fn install(&mut self, world: World) {
        self.kinds.insert(world.pack, world.kind);
        self.state.reload_module(world.pack);
        let resident = self.world.pack == world.pack || self.other.as_ref().is_some_and(|o| o.pack == world.pack);
        if !resident && self.streams(world.kind) {
            self.other = Some(world);
            self.init_other();
        }
    }

    /// The neighbour pack the cell under (x, z) names (tag 0x0B, tables B and A), unless it is
    /// paired with the current pack.
    fn neighbour_at(&self, x: f32, z: f32) -> Option<u32> {
        let v = self.world.ground.as_ref().map_or(-1, |g| g.cell_value(x, z));
        let id = self.world.neighbours.get(v);
        if id < 0 {
            return None;
        }
        let areas = &self.boot.areas;
        let slot = areas.world_ids.iter().position(|&w| w == id as u32)?;
        let target = areas.packs[slot];
        (!self.boot.paired(self.world.pack, target)).then_some(target)
    }

    fn resident(&self, pack: u32) -> bool {
        self.world.pack == pack || self.other.as_ref().is_some_and(|o| o.pack == pack)
    }

    /// SPEC-0006 Pending step: the cell value under the hero names a neighbour (tag 0x0B, then
    /// tables B and A); a pair with the current pack stops it; otherwise it is streamed into
    /// the other buffer when the buffer rule allows and it is not resident there already.
    fn pending_step(&mut self) {
        let p = self.loco.position();
        let Some(target) = self.neighbour_at(p[0], p[2]) else { return };
        if self.resident(target) {
            return;
        }
        match self.kinds.get(&target) {
            Some(&k) => {
                if buffers::stream_buffer(self.world.buffer, k == Kind::Transition).is_some() {
                    self.wanted = Some(target);
                }
            }
            None => self.wanted = Some(target),
        }
    }

    /// The neighbour pack the cell under the hero names, pairs included (the door rule's D).
    fn door_target(&self) -> Option<u32> {
        let p = self.loco.position();
        let v = self.world.ground.as_ref().map_or(-1, |g| g.cell_value(p[0], p[2]));
        let id = self.world.neighbours.get(v);
        let slot = self.boot.areas.world_ids.iter().position(|&w| id >= 0 && w == id as u32)?;
        Some(self.boot.areas.packs[slot])
    }

    /// Move the hero to a landing in the current world and put the camera behind it.
    fn arrive(&mut self, (l, g): ([f32; 3], [f32; 3])) {
        self.loco.teleport(l, g);
        let f = followed(&self.loco, &self.world);
        self.camera.place_behind(&f);
    }

    /// Make the other world current and land on its plane k (teleport landing).
    fn land_in_other(&mut self, k: i32, reverse: bool, upright: bool) -> bool {
        let Some(landing) = self.other.as_ref().and_then(|o| o.landing(k, reverse, upright)) else { return false };
        if let Some(o) = self.other.take() {
            self.other = Some(std::mem::replace(&mut self.world, o));
        }
        self.arrive(landing);
        self.activated();
        true
    }

    /// The current world's Init (a world loaded by a door job).
    fn run_init(&mut self) {
        let mut init = level::on_init(self.world.pack, &mut self.state);
        init.extend(self.group_setup());
        self.apply(&init);
    }

    /// SPEC-0048 set-up of the current world's type-40 groups at its load: a kind-1 chest's
    /// object 0 is drawn at its start (state 9), or at its last frame (finished) when its
    /// collected entry is set.
    fn group_setup(&self) -> Vec<Action> {
        let mut out = Vec::new();
        for g in &self.world.groups {
            let Some(obj) = g.objects.first().and_then(|&o| usize::try_from(o).ok()) else { continue };
            match g.kind {
                1 => out.push(Action::InitObjOn { obj, end: self.state.collected.contains(&g.key) }),
                // Kind 5 (a reaction object): object 0 on and looped at 1.0.
                5 => out.extend([Action::InitObjOn { obj, end: false }, Action::LoopAnim { obj, speed: 1.0 }]),
                // Kind 6 (a dig site, SPEC-0051 section 10 Site set-up): undug, object 0 at its
                // start and the marker on exactly when the map is held; dug, object 2 at its
                // start and object 1 at its end, the marker off.
                6 if g.objects.len() >= 3 => {
                    let o = |k: usize| usize::try_from(g.objects[k]).ok();
                    let marker = g.effects.first().and_then(|&e| usize::try_from(e).ok());
                    if self.state.collected.contains(&g.key) {
                        out.extend(o(2).map(|obj| Action::InitObjOn { obj, end: false }));
                        out.extend(o(1).map(|obj| Action::InitObjOn { obj, end: true }));
                        out.extend(marker.map(|slot| Action::SwitchEffect { slot, on: false }));
                    } else {
                        out.push(Action::InitObjOn { obj, end: false });
                        let held = g.key.checked_sub(200).is_some_and(|i| self.state.maps.contains(&i));
                        out.extend(marker.map(|slot| Action::SwitchEffect { slot, on: held }));
                    }
                }
                _ => {}
            }
        }
        out
    }

    /// A newly loaded other world runs its Init (its actions apply to that world).
    fn init_other(&mut self) {
        if let Some(o) = self.other.take() {
            let current = std::mem::replace(&mut self.world, o);
            let mut init = level::on_init(self.world.pack, &mut self.state);
            init.extend(self.group_setup());
            self.apply(&init);
            self.other = Some(std::mem::replace(&mut self.world, current));
        }
    }

    /// SPEC-0006 Make current: the new world's script runs its activate-world routine.
    fn activated(&mut self) {
        // The world's own region (its embedded type-32 characters), slot 255, once per load.
        let pack = self.world.pack;
        if !self.slot_regions.contains_key(&(pack, EMBEDDED_SLOT)) {
            self.slot_regions.insert((pack, EMBEDDED_SLOT), pack);
            self.region_wanted.push((pack, EMBEDDED_SLOT, pack));
        }
        if self.seen_pack != self.world.pack {
            self.state.last_pack = Some(self.seen_pack).filter(|&p| p != 0);
            self.seen_pack = self.world.pack;
        }
        let actions = level::on_activate(self.world.pack, &mut self.state);
        self.apply(&actions);
        // RegisterOnEnterCallback (SPEC-0030 row 255): right after the activate-world callback.
        let actions = level::on_enter(self.world.pack, &mut self.state);
        self.apply(&actions);
        // SPEC-0047 Scene start restore.
        match self.pet_pending.scene_start() {
            neopets_sim::petpet::Restore::Spawn { type_id, t, f } => self.activate_pet(type_id, t, f),
            neopets_sim::petpet::Restore::Deactivate => self.deactivate_pet(),
            neopets_sim::petpet::Restore::Nothing => {}
        }
        // The pool's slots are reset when a world starts (here: when a world is made current).
        self.shots.reset();
        // SPEC-0025: the region reset clears the remembered glow indices; a named map area is
        // the location a save names.
        self.save_points = Default::default();
        if let Some(m) = self.world.map_area {
            self.last_area = Some(m);
        }
    }

    /// Carry out a level handler's actions (SPEC-0030 host calls), in order.
    /// Start a handler's calls as a script thread and run it until it waits (SPEC-0030 section
    /// 4): calls without a wait take effect at once.
    fn apply(&mut self, actions: &[Action]) {
        if actions.is_empty() {
            return;
        }
        self.threads.push(Thread { pack: self.world.pack, calls: actions.iter().copied().collect(), wait: None });
        let i = self.threads.len() - 1;
        self.step_thread(i);
    }

    /// Run thread i until a call waits or its calls run out.
    fn step_thread(&mut self, i: usize) {
        loop {
            if let Some(w) = self.threads[i].wait {
                if !self.wait_done(w) {
                    return;
                }
                self.threads[i].wait = None;
                // A prompt's answer continues the thread with the routine's answer calls.
                if let Wait::Choice(routine) = w {
                    let pack = self.threads[i].pack;
                    let answer = level::on_answer(pack, routine, self.textbox.result == 1, &mut self.state);
                    for a in answer.into_iter().rev() {
                        self.threads[i].calls.push_front(a);
                    }
                }
            }
            let Some(a) = self.threads[i].calls.pop_front() else { return };
            let pack = self.threads[i].pack;
            // The wheel's round step continues this thread with its calls (SPEC-0059).
            if a == Action::WheelRound {
                for x in level::wheel_round(pack, &mut self.state).into_iter().rev() {
                    self.threads[i].calls.push_front(x);
                }
                continue;
            }
            if let Some(w) = self.execute(a, pack) {
                self.threads[i].wait = Some(w);
            }
        }
    }

    fn wait_done(&self, w: Wait) -> bool {
        match w {
            Wait::Ticks(n) => n == 0,
            Wait::Fade => !self.fade.moving(),
            Wait::Camera => self.cut.is_none_or(|c| !c.waiting()),
            Wait::HeroWalk => self.hero_walk.is_none(),
            Wait::EntityWalk(pack, e) => !self.characters.iter().any(|c| c.pack == pack && c.entity == Some(e) && c.script_walk.is_some()),
            Wait::EntityWalks(pack, es) => !self.characters.iter().any(|c| c.pack == pack && c.entity.is_some_and(|e| es.contains(&e)) && c.script_walk.is_some()),
            Wait::EntityAnim(pack, e, anim) => self.characters.iter().find(|c| c.pack == pack && c.entity == Some(e)).is_none_or(|c| {
                let k = c.anim.clip_state();
                k.action != anim || k.progress >= 0.99
            }),
            Wait::Bars => self.bars.done,
            Wait::Choice(_) => !self.dialogue_open,
            Wait::HeroNear(t, d) => {
                let p = self.loco.position();
                (t[0] - p[0]).hypot(t[2] - p[2]) <= d
            }
            Wait::Dialog => !self.dialogue_open,
            Wait::Loads => self.region_wanted.is_empty() && self.loads_out == 0,
            Wait::WheelSpin => self.wheel_spin.is_none(),
            Wait::HeroOneShot => self.loco.state != neopets_sim::hero::ONE_SHOT,
            Wait::Inside(i) => self.world.records.get(&i).is_none_or(|r| r.state == 0),
            Wait::Word(pack, w) => self.state.word_of(pack, w) == 0,
            Wait::Cage(i) => self.state.burrows.as_ref().and_then(|b| b.cages.get(i)).is_none_or(|c| c.state != 4),
            Wait::ObjAnim(pack, i) => [&self.world].into_iter().chain(self.other.as_ref()).find(|w| w.pack == pack).and_then(|w| w.object_anims.get(i)).is_none_or(|a| a.state != 2),
        }
    }

    /// The script step: each thread's sleep counts down, then every thread runs on until it
    /// waits; finished threads are freed.
    fn run_threads(&mut self) {
        for t in &mut self.threads {
            if let Some(Wait::Ticks(n)) = t.wait.as_mut() {
                *n = n.saturating_sub(1);
            }
        }
        let mut i = 0;
        while i < self.threads.len() {
            self.step_thread(i);
            i += 1;
        }
        self.threads.retain(|t| t.wait.is_some() || !t.calls.is_empty());
        // With no scene left the hero is not held by a thread that never released it.
        if self.threads.is_empty() && self.cut.is_none() && self.hero_walk.is_none() {
            self.held = false;
        }
    }

    /// The character of world `pack` with entity id e.
    fn entity_mut(&mut self, pack: u32, e: usize) -> Option<&mut Character> {
        self.characters.iter_mut().find(|c| c.pack == pack && c.entity == Some(e))
    }

    /// Entry (locator) record k of resident world `pack`.
    fn locator(&self, pack: u32, k: usize) -> Option<([f32; 3], [f32; 3])> {
        [&self.world].into_iter().chain(self.other.as_ref()).find(|w| w.pack == pack)?.entries.get(k).copied()
    }

    /// The ground height under (x, z) in resident world `pack`, probed from c.
    fn ground_in(&self, pack: u32, x: f32, z: f32, c: f32) -> Option<f32> {
        [&self.world].into_iter().chain(self.other.as_ref()).find(|w| w.pack == pack)?.ground_at(x, z, c)
    }

    /// Carry out one script call of a thread of world `pack`; a blocking call returns what it
    /// waits for.
    fn execute(&mut self, a: Action, pack: u32) -> Option<Wait> {
        {
            match a {
                Action::EnterInterior { slot } if self.world.buffer == Buffer::Exterior => {
                    if let Some(&pack) = self.boot.areas.packs.get(slot) {
                        self.job = Some(Job::EnterInterior(pack));
                        self.last_door = format!("script: entering slot {slot} (pack {pack}) as an interior");
                    }
                }
                Action::LeaveInterior { slot } if self.world.buffer == Buffer::Interior => {
                    // E becomes current at its entry record 0 (no probe); T is reloaded with
                    // the slot's pack; E's activate-world routine runs.
                    let Some(o) = self.other.take() else { return None };
                    let landing = o.entries.first().copied().unwrap_or(([0.0; 3], [0.0, 0.0, 1.0]));
                    let interior = std::mem::replace(&mut self.world, o);
                    drop(interior);
                    self.interior_t = None;
                    self.arrive(landing);
                    if let Some(&pack) = self.boot.areas.packs.get(slot) {
                        self.job = Some(Job::ReloadTransition(pack));
                    }
                    self.last_door = format!("script: left the interior to {}, reloading slot {slot}", self.world.pack);
                    self.activated();
                }
                Action::MoveHero { locator } => {
                    if let Some(&e) = self.world.entries.get(locator) {
                        self.arrive(e);
                        self.last_door = format!("script: hero moved to locator {locator} of {}", self.world.pack);
                    }
                }
                Action::Message { line } => {
                    self.talk_partner = None;
                    self.talk_with = None;
                    self.open_line(line);
                    return Some(Wait::Dialog);
                }
                Action::Talk { line } => {
                    self.open_line(line);
                    return Some(Wait::Dialog);
                }
                Action::Converse { entity, line } => {
                    let partner = self.entity_mut(pack, entity).map(|c| (c.id, c.name.clone()));
                    self.talk_with = partner.as_ref().map(|p| p.0);
                    self.talk_partner = partner.map(|p| p.1);
                    self.open_line(line);
                    return Some(Wait::Dialog);
                }
                Action::FadeOut { seconds, wait } => {
                    self.fade.out([0; 3], neopets_sim::scene::frames(seconds));
                    return wait.then_some(Wait::Fade);
                }
                Action::FadeIn { seconds, wait } => {
                    self.fade.fade_in([0; 3], neopets_sim::scene::frames(seconds));
                    return wait.then_some(Wait::Fade);
                }
                Action::PlayCamera { track, wait, end, speed } => {
                    // A track outside the list does nothing; WAIT waits only on a keyed track.
                    let keyed = !self.world.tracks.get(track)?.keys.is_empty();
                    self.cut = Some(neopets_sim::scene::CameraPlay::new(track, end, speed));
                    self.held = true;
                    return (wait && keyed).then_some(Wait::Camera);
                }
                Action::ReturnCamera { preset, release } => {
                    // The follow camera takes over behind the hero (the preset's pull-in to a
                    // locator is not reproduced).
                    let _ = preset;
                    self.cut = None;
                    let f = followed(&self.loco, &self.world);
                    self.camera.place_behind(&f);
                    if release {
                        self.held = false;
                        self.hero_walk = None;
                    }
                }
                Action::HoldHero => self.held = true,
                Action::SwapMode { mode } => {
                    self.swap_mode = mode;
                    self.last_swap = format!("script: swap mode {mode} on tick {}", self.ticks);
                }
                Action::SwitchHero { hero, effects, flash } => self.script_switch(hero, effects, flash),
                Action::ActivateHero { hero, locator, block, fade, cue } => {
                    if let Some(&e) = self.world.entries.get(locator) {
                        self.arrive(e);
                    }
                    self.script_switch(hero, fade, cue);
                    if block {
                        self.held = true;
                    }
                }
                Action::ActStart { stage } => self.act_start(stage),
                Action::StartEffect { slot, camera } => self.start_placed(pack, slot, camera),
                Action::SwitchEffect { slot, on } => self.switch_placed(pack, slot, on),
                Action::InitObjOn { obj, .. } | Action::PlayAnim { obj, .. } | Action::LoopAnim { obj, speed: _ } | Action::StopAnim { obj } | Action::DeactivateObj { obj }
                    if [&self.world].into_iter().chain(self.other.as_ref()).find(|w| w.pack == pack).is_some_and(|w| obj < w.object_anims.len()) =>
                {
                    let w = if self.world.pack == pack { &mut self.world } else { self.other.as_mut().unwrap() };
                    let (m, o) = (&w.motions[obj], &mut w.object_anims[obj]);
                    match a {
                        Action::InitObjOn { end, .. } => o.init_on(m, end),
                        Action::PlayAnim { speed, wait, .. } => {
                            o.play(m, speed);
                            if wait {
                                return Some(Wait::ObjAnim(pack, obj));
                            }
                        }
                        Action::LoopAnim { speed, .. } => o.looped(speed),
                        Action::StopAnim { .. } => o.stop(),
                        _ => o.deactivate(),
                    }
                }
                Action::InitObjOn { .. } | Action::PlayAnim { .. } | Action::LoopAnim { .. } | Action::StopAnim { .. } | Action::DeactivateObj { .. } => {}
                Action::WheelLights { even } => {
                    let mut w = level::wheel_of(pack, &self.state);
                    w.light_show(if even { 0 } else { 5 });
                    level::set_wheel(pack, &mut self.state, &w);
                    self.display_mask.insert(pack, 2 * w.mask(neopets_sim::wheel::VILLAGE.shift));
                }
                Action::WheelPattern => {
                    let mut w = level::wheel_of(pack, &self.state);
                    let rng = &mut self.rng;
                    w.new_pattern(&mut || rng.range(0, 7));
                    level::set_wheel(pack, &mut self.state, &w);
                    self.display_mask.insert(pack, 2 * w.mask(neopets_sim::wheel::VILLAGE.shift));
                }
                Action::WheelSpin => {
                    self.wheel_spin = Some((pack, neopets_sim::wheel::Spin::default()));
                    return Some(Wait::WheelSpin);
                }
                Action::WheelResult => {
                    let mut w = level::wheel_of(pack, &self.state);
                    let won = w.result();
                    level::set_wheel(pack, &mut self.state, &w);
                    self.display_mask.insert(pack, 2 * w.mask(neopets_sim::wheel::VILLAGE.shift));
                    let (win, loss) = neopets_sim::wheel::VILLAGE.sounds;
                    self.apply(&[Action::PlaySound { cue: if won { win } else { loss }, locator: None }]);
                    self.last_fx = format!("wheel stopped at {:.1} on cell {}: {} on tick {}", w.angle, w.cell(), if won { "win" } else { "loss" }, self.ticks);
                }
                Action::WheelRound => {}
                Action::BurrowsRequest { .. } | Action::BurrowsClose { .. } => level::burrows_request(&mut self.state, a),
                Action::BurrowsStagger => self.apply(&level::burrows_stagger()),
                Action::ChestReward { group } => self.chest_reward(group as usize),
                Action::DigStep { group, step } => self.dig_thread_step(group as usize, step),
                Action::WaitCage { index } => return Some(Wait::Cage(index as usize)),
                Action::SleepRandom { lo, hi } => return Some(Wait::Ticks(self.rng.range(lo as i32, hi as i32).max(0) as u32)),
                Action::MoveEffect { slot, locator } => {
                    if let Some((at, _)) = self.locator(pack, locator) {
                        let h = self.placed_object(pack, slot);
                        if let Some(h) = h {
                            self.effects.set_position(h, at);
                            if let Some(Some(o)) = self.effects.objects.get_mut(h) {
                                o.position = at;
                            }
                        }
                    }
                }
                Action::StopEffect { slot } => {
                    let table = self.effect_table.clone();
                    if let Some(&h) = self.placed_fx.get(&(pack, slot)) {
                        self.effects.stop(&table, h);
                    }
                }
                Action::WalkHero { locator, wait, orient } => {
                    let (t, g) = *self.world.entries.get(locator)?;
                    self.hero_walk = Some((t, orient.then_some(g), f32::MAX));
                    self.held = true;
                    return wait.then_some(Wait::HeroWalk);
                }
                Action::MoveEntity { entity, locator, face } => {
                    let (l, g) = self.locator(pack, locator)?;
                    let y = self.ground_in(pack, l[0], l[2], l[1] + 4.0).unwrap_or(l[1]);
                    let c = self.entity_mut(pack, entity)?;
                    c.position = [l[0], y, l[2]];
                    c.script_walk = None;
                    c.held = true;
                    if face {
                        c.facing = g;
                    }
                    if let Some(wk) = c.walker.as_mut() {
                        wk.body.position = c.position;
                        wk.facing = c.facing;
                    }
                }
                Action::WalkEntity { entity, locator, wait, orient, stop } => {
                    let (l, g) = self.locator(pack, locator)?;
                    let y = self.ground_in(pack, l[0], l[2], l[1] + 6.0).unwrap_or(l[1]);
                    let face = orient.then(|| {
                        let len = g[0].hypot(g[2]).max(1e-6);
                        [g[0] / len, 0.0, g[2] / len]
                    });
                    let c = self.entity_mut(pack, entity)?;
                    c.script_walk = Some(([l[0], y, l[2]], face, if stop > 0.0 { stop } else { 1.0 }));
                    c.held = true;
                    return wait.then_some(Wait::EntityWalk(pack, entity));
                }
                Action::WaitWalks { entities } => return Some(Wait::EntityWalks(pack, entities)),
                Action::SetFlag { flag, on } => self.state.set(flag, on),
                Action::GiveItem { item, count } if item == level::TREASURE_MAP => self.map_held(count),
                Action::GiveItem { item, count } => self.state.give(item, count),
                Action::WaitHeroNear { locator, distance } => {
                    let (l, _) = self.locator(pack, locator)?;
                    return Some(Wait::HeroNear(l, distance));
                }
                Action::Prompt { line, routine } => {
                    self.talk_partner = None;
                    self.talk_with = None;
                    self.open_line(line);
                    if let Some(l) = self.world.lines.line(line as usize) {
                        self.textbox.open_choice(&l.raw);
                    }
                    return Some(Wait::Choice(routine));
                }
                Action::ScalePrices { factor } => {
                    if let Some(pool) = self.pools.get_mut(&pack) {
                        for it in pool.slots.iter_mut().flatten().filter(|it| it.flag == 1) {
                            it.price = neopets_sim::shop::scaled(it.price, factor);
                        }
                    }
                }
                Action::GiveWordItem { word, default } => {
                    let v = self.state.words.get(&(pack, word)).copied().unwrap_or(0);
                    let item = if v > 0 { v as u16 } else { default };
                    self.state.give(item, 1);
                }
                Action::RandomLoop { entity, anims } => {
                    let a = anims[(self.rng.range(1, 2) - 1).clamp(0, 1) as usize];
                    let c = self.entity_mut(pack, entity)?;
                    c.held = true;
                    c.anim.play(a);
                }
                Action::EntityAnim { entity, anim, looped: _, wait } => {
                    // The clip plays as the character's animation (loops repeat as its clip
                    // does); the entity is under script control.
                    let c = self.entity_mut(pack, entity)?;
                    c.held = true;
                    c.script_walk = None;
                    c.anim.play(anim);
                    return wait.then_some(Wait::EntityAnim(pack, entity, anim));
                }
                Action::ReleaseEntity { entity } => {
                    let c = self.entity_mut(pack, entity)?;
                    c.held = false;
                    c.script_walk = None;
                    c.anim.play(0);
                }
                Action::Agro { entity } => {
                    let hero = self.loco.position();
                    let rng = &mut self.rng;
                    let c = self.characters.iter_mut().find(|c| c.pack == pack && c.entity == Some(entity))?;
                    c.held = false;
                    if let Some(id) = c.enemy.as_mut().and_then(|e| e.agro_on(hero, rng)) {
                        c.anim.play(id);
                    }
                }
                Action::DeactivateEntity { entity } => {
                    // An entity whose region has not loaded yet is deactivated when it arrives
                    // (an Init's deactivations and streaming refusals run before the load).
                    let Some(c) = self.entity_mut(pack, entity) else {
                        self.pending_off.insert((pack, entity));
                        return None;
                    };
                    c.inactive = true;
                    c.deactivated = true;
                    c.script_walk = None;
                }
                Action::ActivateEntity { entity } => {
                    self.pending_off.remove(&(pack, entity));
                    if let Some(c) = self.characters.iter_mut().find(|c| c.pack == pack && c.entity == Some(entity) && c.dying.is_none() && c.hp > 0 && !c.enemy.as_ref().is_some_and(|e| e.dead)) {
                        c.deactivated = false;
                        c.inactive = false;
                        return None;
                    }
                    // A dead or released embedded entity is bound again from its record (the
                    // forced bind of SPEC-0045 1217): the embedded region is read again and the
                    // thread waits for it.
                    let world = [&self.world].into_iter().chain(self.other.as_ref()).find(|w| w.pack == pack)?;
                    let &(r, o) = world.entities.get(entity)?;
                    if r != 0xFF {
                        return None;
                    }
                    self.characters.retain(|c| !(c.pack == pack && c.entity == Some(entity)));
                    self.rebind.push((pack, o as usize));
                    self.slot_regions.insert((pack, REBIND_SLOT), pack);
                    self.region_wanted.push((pack, REBIND_SLOT, pack));
                    return Some(Wait::Loads);
                }
                Action::LevelState { flag, on } => self.state.set_level(flag, on),
                Action::RefillMagic => self.hero_mp = self.hero_mp_max,
                Action::MiamouseInit { field, cage } => self.catch.init(field, cage),
                Action::MiamouseDisable => self.catch.on = false,
                Action::Bits { word, set, clear } => {
                    let w = self.state.word_of(pack, word);
                    self.state.set_word(pack, word, (w | set) & !clear);
                }
                // PlayHeroAnim and the equipment display are not reproduced.
                // PlayHeroAnim: the one-shot state 51 (the clip restarted); the call yields one
                // tick, and with WAIT while the hero stays in state 51. The scripts' CTL is taken
                // as 1 (the hero returns to state 1).
                Action::HeroAnim { anim, wait } => {
                    self.loco.play_one_shot(anim, true);
                    self.hero.request(anim, true);
                    return Some(if wait { Wait::HeroOneShot } else { Wait::Ticks(1) });
                }
                Action::Equip { .. } => {}
                Action::ReplaceExterior { slot, plane } => {
                    if let Some(&p) = self.boot.areas.packs.get(slot) {
                        self.job = Some(Job::Replace(p, plane));
                        self.last_door = format!("script: replacing the exterior with slot {slot} (pack {p}), plane {plane}");
                    }
                }
                Action::HitPointsPercent { entity, percent } => {
                    let c = self.entity_mut(pack, entity)?;
                    let max = c.row.map_or(1, |r| r.hit_points as i32);
                    c.hp = (max as f32 * percent / 100.0) as i32;
                }
                Action::LetterBox { on, seconds, wait } => {
                    let n = neopets_sim::scene::frames(seconds);
                    if on {
                        self.bars.on(n);
                    } else {
                        self.bars.off(n);
                    }
                    return wait.then_some(Wait::Bars);
                }
                Action::Sleep { ticks } => return Some(Wait::Ticks(ticks)),
                Action::WaitLoads => return Some(Wait::Loads),
                Action::WaitDialog => return Some(Wait::Dialog),
                Action::RegisterTrigger { index, routine } => {
                    let _ = pack;
                    if (index as usize) < self.world.capacity {
                        self.world.routines.insert(index, routine);
                        self.world.group_routes.remove(&index);
                    }
                }
                Action::RegisterGroupType { kind } => {
                    // Each entry of the kind: its first trigger to the run callback (replacing an
                    // earlier registration), then the init callback at once.
                    let entries: Vec<(usize, Option<u16>, Vec<usize>)> = self
                        .world
                        .groups
                        .iter()
                        .enumerate()
                        .filter(|(_, g)| g.kind == kind)
                        .map(|(i, g)| (i, g.triggers.first().and_then(|&t| u16::try_from(t).ok()), g.objects.iter().filter_map(|&o| usize::try_from(o).ok()).collect()))
                        .collect();
                    for (i, trigger, objects) in entries {
                        if let Some(t) = trigger.filter(|&t| (t as usize) < self.world.capacity) {
                            self.world.routines.remove(&t);
                            self.world.group_routes.insert(t, i);
                        }
                        let init = level::group_init(self.world.pack, kind, &objects);
                        self.apply(&init);
                    }
                }
                Action::SetObjTime { obj, seconds } => {
                    let w = if self.world.pack == pack { Some(&mut self.world) } else { self.other.as_mut().filter(|o| o.pack == pack) };
                    if let Some(w) = w.filter(|w| obj < w.object_anims.len()) {
                        w.object_anims[obj].set_time(&w.motions[obj], seconds);
                    }
                }
                Action::SetObjTimeRandom { obj } => {
                    // Random(0, 100) / 100, computed in double precision and stored as a float.
                    let r = self.rng.range(0, 100);
                    self.apply(&[Action::SetObjTime { obj, seconds: (r as f64 / 100.0) as f32 }]);
                }
                Action::WaitInside { index } => return Some(Wait::Inside(index)),
                Action::WaitWord { word } => return Some(Wait::Word(pack, word)),
                Action::RegisterInstance { routine, trigger, group } => {
                    let triggers: Vec<u16> = match self.world.groups.get(group) {
                        Some(g) if g.kind == 7 => g.triggers.iter().filter_map(|&t| u16::try_from(t).ok()).collect(),
                        Some(_) => vec![trigger],
                        None => Vec::new(),
                    };
                    for t in triggers.into_iter().filter(|&t| (t as usize) < self.world.capacity) {
                        self.world.routines.insert(t, routine);
                        self.world.group_routes.remove(&t);
                    }
                }
                Action::LockChest { id, key, message } => {
                    if self.world.locks.len() < 2 {
                        self.world.locks.push((id, key, message));
                    }
                }
                Action::WhinnyPen { inner, entrance, locator } => {
                    let point = self.world.entries.get(locator).map_or([0.0; 3], |e| e.0);
                    self.world.pens = vec![neopets_sim::whinny::Pen { inner, entrance, point, occupied: false }];
                }
                Action::PlaySound { cue, locator } | Action::LoopSound { cue, locator } | Action::VisitLoopSound { cue, locator } | Action::KillSound { cue, locator } => {
                    use neopets_sim::sound::{Play as Cue, AREA};
                    let at = locator.and_then(|i| self.world.entries.get(i)).map(|e| e.0);
                    match a {
                        Action::PlaySound { .. } => {
                            self.sound.play(AREA, cue, Cue { emitter: at, ..Default::default() });
                        }
                        Action::LoopSound { .. } => {
                            self.sound.play(AREA, cue, Cue { emitter: at, keep_alive: true, ..Default::default() });
                        }
                        Action::VisitLoopSound { .. } => self.sound.refresh(AREA, cue, at),
                        _ => self.sound.stop(AREA, cue, at),
                    }
                }
                Action::PlayHeroSound { hero, cue, locator, do_not_restart } => {
                    // Only for -1 or the selected hero's flag (0, Tor); at the locator or the hero.
                    if hero == -1 || hero == 0 {
                        let at = locator.and_then(|i| self.world.entries.get(i)).map(|e| e.0).unwrap_or(self.loco.position());
                        self.sound.play(neopets_sim::sound::VOICE, cue, neopets_sim::sound::Play { emitter: Some(at), do_not_restart, ..Default::default() });
                    }
                }
                Action::PlayStinger { id, volume } => {
                    if volume > 0.0 {
                        self.stinger(id);
                    }
                }
                Action::SetExcite { level } => {
                    if (0..100).contains(&level) {
                        self.intensity.e = level;
                    }
                }
                Action::SetAdventure { level } => self.intensity.adventure(level),
                Action::LoadRegion { slot, region } => {
                    let pack = self.world.pack;
                    self.characters.retain(|c| !(c.pack == pack && c.slot == slot));
                    match region {
                        Some(r) => {
                            self.slot_regions.insert((pack, slot), r);
                            self.region_wanted.push((pack, slot, r));
                        }
                        None => {
                            self.slot_regions.remove(&(pack, slot));
                        }
                    }
                }
                _ => self.last_door = format!("script: {a:?} refused in the {:?} buffer", self.world.buffer),
            }
        }
        None
    }

    /// Diagnostic: the scene state (camera play, fade, hold, scripted walk, threads).
    pub fn scene_status(&self) -> String {
        let walking = self.characters.iter().filter(|c| c.script_walk.is_some()).filter_map(|c| c.entity).collect::<Vec<_>>();
        format!(
            "camera {:?} fade to {:.0} bars {} held {} hero walk {} entity walks {walking:?} threads {}",
            self.cut.map(|c| (c.track, c.state)),
            self.fade.target,
            self.bars.target,
            self.held,
            self.hero_walk.is_some(),
            self.threads.len()
        )
    }

    /// The view the window draws: the camera play's pose (while it plays or holds its last
    /// pose) or the follow camera; eye, forward, up and focal length (SPEC-0027 Projection).
    pub fn view(&self) -> ([f32; 3], [f32; 3], [f32; 3], f32) {
        match self.cut.filter(|c| c.state != 0).and_then(|c| c.last) {
            Some(p) => (p.eye, p.rows[2], p.rows[1], p.focal),
            None => {
                let (e, f) = (self.camera.eye, self.camera.focus);
                ([e[0], e[1], e[2]], [f[0] - e[0], f[1] - e[1], f[2] - e[2]], [0.0, 1.0, 0.0], neopets_sim::camera::FOCAL)
            }
        }
    }

    /// SPEC-0006 Doors, default rule, for door n of the current world (already confirmed).
    fn default_door(&mut self, n: usize) {
        let e = self.world.arrivals.get(n as i32);
        let plane = self.world.planes[n];
        let f = self.loco.facing;
        let reverse = plane.f[0] * f[0] + plane.f[1] * f[1] + plane.f[2] * f[2] >= 0.0;
        let target = self.door_target();
        self.last_door = match self.world.buffer {
            Buffer::Exterior => {
                let Some(d) = target else {
                    self.last_door = format!("door {n}: the cell names no world");
                    return;
                };
                if self.other.as_ref().is_some_and(|o| o.pack == d && o.kind == Kind::Transition) {
                    if self.land_in_other(e, reverse, false) {
                        format!("door {n}: into transition {d} at its door {e}")
                    } else {
                        format!("door {n}: transition {d} has no door {e}")
                    }
                } else if !self.boot.paired(self.world.pack, d) {
                    self.job = Some(Job::FromExterior { pack: d, plane: e, reverse });
                    format!("door {n}: loading {d} to enter it")
                } else {
                    self.job = Some(Job::Replace(d, e));
                    format!("door {n}: replacing the exterior with {d}, arriving at door {e}")
                }
            }
            Buffer::Transition => {
                if self.land_in_other(e, reverse, false) {
                    format!("door {n}: into the exterior at its door {e}")
                } else {
                    format!("door {n}: no exterior door {e} to land on")
                }
            }
            Buffer::Interior => {
                // Leave-interior job: E becomes current, the upright landing on plane e (always
                // probed), facing F; then T is reloaded with the pack it held.
                if self.land_in_other(e, false, true) {
                    if let Some(t) = self.interior_t.take() {
                        self.job = Some(Job::ReloadTransition(t));
                    }
                    format!("door {n}: left the interior to its exterior's door {e}")
                } else {
                    format!("door {n}: no exterior resident to leave to")
                }
            }
        };
    }

    /// SPEC-0006 reverse id: the first entity-table entry whose region byte is the character's
    /// region (0xFF for the embedded region, else j of region P + 1 + j) and whose ordinal is
    /// its kind-3 ordinal.
    fn entity_of(&self, pack: u32, region: u32, ordinal: usize) -> Option<usize> {
        let world = [&self.world].into_iter().chain(self.other.as_ref()).find(|w| w.pack == pack)?;
        let j = if region == pack { 0xFF } else { u8::try_from(region.checked_sub(pack + 1)?).ok()? };
        world.entities.iter().position(|&(r, o)| r == j && o as usize == ordinal)
    }

    /// SPEC-0050 sections 3 and 9 and SPEC-0022 chooser step 1: the buyable item (it beats
    /// talk). Without a press the caption is "Buy" with the item name and the price caption shows
    /// the price and the balance; with Square the buy action runs. Returns whether it claimed.
    fn buy_step(&mut self) -> bool {
        self.buy_caption = None;
        self.buyable = None;
        let pack = self.world.pack;
        let Some(pool) = self.pools.get(&pack) else { return false };
        let slots: Vec<(usize, neopets_sim::items::WorldItem)> = pool.slots.iter().enumerate().filter_map(|(i, s)| Some((i, (*s)?))).collect();
        let cands: Vec<neopets_sim::shop::Candidate> = slots.iter().map(|(_, it)| neopets_sim::shop::Candidate { position: it.position, flag: it.flag, ready: it.ready() }).collect();
        let Some(k) = neopets_sim::shop::buyable(&cands, self.loco.position(), self.loco.facing) else { return false };
        let (i, it) = slots[k];
        self.buyable = Some(i);
        if !self.square_now {
            let name = self.design.item_name(it.id).unwrap_or_else(|| it.id.to_string().into_bytes());
            let mut text = Vec::new();
            match self.buy_template.windows(2).position(|w| w == b"%s") {
                Some(at) => {
                    text.extend_from_slice(&self.buy_template[..at]);
                    text.extend_from_slice(&name);
                    text.extend_from_slice(&self.buy_template[at + 2..]);
                }
                None => text.extend_from_slice(&name),
            }
            self.buy_caption = Some(text);
            self.price_lines[1] = (self.state.neopoints, 1);
            self.price_lines[0] = (it.price as i32, 1);
            return true;
        }
        self.buy(i, it);
        true
    }

    /// SPEC-0050 section 3, the buy action on pool item i: the buy hook's line (handled 0, so
    /// the purchase goes on), the price tests, the add with the carry limit, the charge, and the
    /// take-or-copy and sold-out rules of section 8.
    fn buy(&mut self, i: usize, it: neopets_sim::items::WorldItem) {
        let pack = self.world.pack;
        let hook = level::on_buy(pack, it.id, &self.state);
        self.apply(&hook);
        // A buy callback that writes the record's flag word cancels the buy (SPEC-0030 row
        // 260); the hooks here return their line only then.
        if !hook.is_empty() {
            self.last_pickup = format!("buy of item {} cancelled by the level script on tick {}", it.id, self.ticks);
            return;
        }
        let limit = self.design.carry_limit(it.id).unwrap_or(0);
        let held = self.state.items.get(&it.id).copied().unwrap_or(0);
        let add_ok = limit == 0 || held < limit;
        match neopets_sim::shop::decide(it.price, self.state.neopoints, add_ok) {
            neopets_sim::shop::Buy::NeedMore(n) => {
                // The need-more text of SPEC-0050 (its currency glyph is not given; R-0030).
                let text = format!("I need another {n}!").into_bytes();
                self.open_text(b"Tor", &text);
                self.last_pickup = format!("buy refused: need {n} more NP for item {} on tick {}", it.id, self.ticks);
            }
            neopets_sim::shop::Buy::Bought => {
                self.state.neopoints -= it.price as i32;
                self.state.neopoints = self.state.neopoints.max(0);
                if it.id == level::TREASURE_MAP {
                    // A bought map is map +0x2C plus 1 (SPEC-0021 Treasure maps).
                    self.map_held(it.map as u32 + 1);
                } else {
                    self.state.give(it.id, if neopets_sim::mote::is_mote(it.id) { 100 } else { 1 });
                }
                let taken = neopets_sim::shop::takes(it.id, self.item_kind(it.id));
                if let Some(pool) = self.pools.get_mut(&pack) {
                    if taken {
                        pool.take(i);
                    }
                    // Sold out: holding the carry limit removes every world item of that id.
                    if limit > 0 && held + 1 >= limit {
                        for s in pool.slots.iter_mut() {
                            if s.is_some_and(|w| w.id == it.id) {
                                *s = None;
                            }
                        }
                    }
                }
                if matches!(it.id, 19 | 20 | 151 | 154 | 174) {
                    self.once.insert((it.id, pack));
                }
                self.last_pickup = format!("bought item {} for {} NP (wallet {}) on tick {}", it.id, it.price, self.state.neopoints, self.ticks);
            }
            other => {
                // The never-affordable and can't-buy-now texts are not given (R-0030).
                self.last_pickup = format!("buy refused ({other:?}) for item {} on tick {}", it.id, self.ticks);
            }
        }
    }

    /// Open the box on a text that is not a level line (the shop's refusal), with a speaker.
    fn open_text(&mut self, speaker: &[u8], text: &[u8]) {
        self.message = None;
        self.speaker = (speaker.to_vec(), self.hero_colour);
        self.textbox.open(text);
        self.dialogue_open = true;
    }

    /// SPEC-0022 Interaction range and target choice, simplified: the nearest character of the
    /// current world with a registered talk routine, at most 20.0 away and within the hero's
    /// facing cone (cosine 0.5). The 90-tick leader hold, the reorder rule and the lock-on
    /// list are not reproduced. Returns (index, distance, routine).
    fn talk_candidate(&self) -> Option<(usize, f32, u8)> {
        let p = self.loco.position();
        let f = self.loco.facing;
        self.characters
            .iter()
            .enumerate()
            .filter(|(_, c)| c.pack == self.world.pack && !c.inactive)
            .filter_map(|(i, c)| {
                let routine = level::proximity_routine(c.pack, c.entity?)?;
                let d = [c.position[0] - p[0], c.position[1] - p[1], c.position[2] - p[2]];
                let len = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
                let cos = (d[0] * f[0] + d[2] * f[2]) / (d[0] * d[0] + d[2] * d[2]).sqrt().max(1e-6);
                (len <= 20.0 && cos >= 0.5).then_some((i, len, routine))
            })
            .min_by(|a, b| a.1.total_cmp(&b.1))
    }

    /// The talk entry (SPEC-0022): within 5.8 the "Talk" label (8) is shown and Square starts
    /// the routine; a candidate with a routine claims the press at any range. Returns whether
    /// the press was claimed.
    fn talk_step(&mut self) -> bool {
        let Some((i, d, routine)) = self.talk_candidate() else { return false };
        if d < 5.8 {
            self.talk_prompt = true;
            if self.square_now {
                let pack = self.characters[i].pack;
                self.talk_partner = Some(self.characters[i].name.clone());
                self.talk_with = Some(self.characters[i].id);
                // SetConversationTarget clears both characters' emote queues.
                self.hero_emotes.clear();
                self.characters[i].emotes.clear();
                self.state.talk_d = self.rng.range(1, 6);
                self.state.talk_roll = self.rng.range(1, 3);
                let actions = level::on_talk(pack, routine, &mut self.state);
                self.apply(&actions);
            }
        }
        true
    }

    /// SPEC-0036 Attack input (Roberta), Charge completion and Attack router: the attack start
    /// by gesture mode, the sub-state step with its requests, the power-ready check from F = 7,
    /// and the launcher: on the release frame (event bit 0x20), unfired, with at least 1 MP, the
    /// sub-state's burst into free shot slots (a full release pays its cost and word-2 charge at
    /// the first shot, or at release for ids 162, 169 and 171).
    fn roberta_step(&mut self, input: &HeroInput) {
        use neopets_sim::roberta::*;
        if self.r_attack.is_none() && input.attack && (1..=6).contains(&self.loco.state) {
            self.r_attack = Some(RobertaAttack::start_with(self.gesture.mode));
        }
        let mote = self.equipment.motes[1][2];
        let Some(mut a) = self.r_attack.take() else { return };
        let c = self.hero.clip_state();
        let before = a.sub;
        match a.tick(input.attack_held, input.attack, mote, Clip { action: c.action, progress: c.progress }) {
            Some((action, restart)) => {
                self.loco.set_attack(Some(action));
                if restart {
                    self.hero.request(action, true);
                }
            }
            None => {
                self.loco.set_attack(None);
                return;
            }
        }
        if a.cleared {
            a.cleared = false;
            self.hit_list.clear();
        }
        // Sub-state 14 (ids 162, 169 and 171) pays at release.
        if before != 14 && a.sub == 14 {
            let (cost, q) = self.magic.roberta_full(mote);
            if let Some(id) = mote {
                let s = self.weapon_stat_b();
                self.spend_charge(2, id, charge_full(q, s));
            }
            self.pay_mp(cost);
        }
        // The power-ready check from F = 7 with a Mote in word 2.
        if a.sub == 1 && a.f >= 7 {
            if let Some(id) = mote {
                let held = self.state.items.get(&id).copied().unwrap_or(0) > 0;
                if !(self.weapon.is_some() && held && self.hero_mp >= self.magic.roberta_full(mote).0) {
                    a.f = 0;
                }
            }
        }
        // The launcher.
        let release = self.hero.frame_bits() & 0x20 != 0;
        if !a.fired && release && self.hero_mp >= 1 {
            a.fired = true;
            self.roberta_launch(&a, mote);
        }
        self.r_attack = Some(a);
    }

    /// The launch of one burst for the attack's sub-state (SPEC-0036 Roberta shot projectile).
    fn roberta_launch(&mut self, a: &neopets_sim::roberta::RobertaAttack, mote: Option<u16>) {
        use neopets_sim::roberta::*;
        let (class, sub, held) = (a.class(), a.sub, a.held);
        let d = a.step_factor();
        let (row, count) = self.shot_rows.launch(class, sub, mote);
        let full = class == 3;
        let p = self.loco.position();
        let f = self.loco.facing;
        let anchor = anchor(p, f);
        let dir = launch_dir(anchor, default_aim(p, f));
        // S = A x D x B x G x E; T is 1, or the attack power table value for a full release.
        let weapon_a = self.weapon.and_then(|i| self.items.get(i)).map_or(0.0, |i| i.stats[0] as f32);
        let b = mote.and_then(|id| self.items.get(id)).map_or(1.0, |i| neopets_sim::mote::one_plus_percent(i.stats[5] as f32));
        let e = self.pet_bonus.damage_factor(self.petpet.as_ref().map(|p| p.kind), self.petpet.as_ref().map_or(0, |p| p.t));
        let scale = neopets_sim::charm::scale(weapon_a, d, b, self.statuses.damage_scale(), e);
        let t = if full { mote.and_then(|id| neopets_sim::mote::power_index(1, id)).and_then(|k| self.design.attack_power(k)).unwrap_or(1.0) } else { 1.0 };
        let status = (full && row.reaction != -1)
            .then(|| mote.and_then(|id| neopets_sim::mote::status_request(1, id, |k| self.design.attack_power(k))).map(|r| [row.reaction, r[1], r[2], r[3]]).unwrap_or([row.reaction, -1, -1, -1]));
        let mut launched = 0;
        for _ in 0..count {
            if self.hero_shots.len() >= 6 {
                break;
            }
            // The first full-release shot pays the cost and the word-2 charge.
            if full && launched == 0 {
                let (cost, q) = self.magic.roberta_full(mote);
                if let Some(id) = mote {
                    let s = self.weapon_stat_b();
                    self.spend_charge(2, id, charge_full(q, s));
                }
                self.pay_mp(cost);
            }
            self.hero_shots.push(Shot {
                position: anchor,
                dir,
                speed: row.speed,
                radius: row.radius,
                life: row.life,
                homing: row.homing,
                target: None,
                element: neopets_sim::mote::element(mote),
                scale,
                t,
                class,
                status,
            });
            launched += 1;
        }
        self.last_hit = format!("Roberta launched {launched} shot(s) (class {class}, held {held}) on tick {}", self.ticks);
    }

    /// SPEC-0036 Roberta full-charge hold effect and Timed glow: effect 239 at the hero's frame
    /// (started, then updated there each pass) and effect 0xF1 at the hero + (0, 0.5, 0) on its
    /// flat facing (started when not live, then moved); each stops when its timer says so.
    fn roberta_displays_step(&mut self) {
        let (sub, f) = match (self.hero_index, self.r_attack.as_ref()) {
            (1, Some(a)) => (a.sub, a.f),
            _ => (0, 0),
        };
        let shown = self.r_displays.step(sub, f);
        let table = self.effect_table.clone();
        let p = self.loco.position();
        let fw = unit([self.loco.facing[0], 0.0, self.loco.facing[2]]);
        let frame = [[fw[2], 0.0, -fw[0]], [0.0, 1.0, 0.0], fw];
        for (k, (show, id, at)) in [(shown.0, neopets_sim::roberta::HOLD_EFFECT, p), (shown.1, neopets_sim::roberta::GLOW_EFFECT, [p[0], p[1] + 0.5, p[2]])].into_iter().enumerate() {
            match (show, self.r_fx[k]) {
                (true, None) => {
                    let h = self.effects.add_object(neopets_sim::effect::EffectObject::new(&table, id, at, frame));
                    self.effects.update(&table, h, None, &mut self.rng);
                    self.r_fx[k] = Some(h);
                    self.last_fx = format!("Roberta effect {id} started (sub-state {sub}, F {f}) on tick {}", self.ticks);
                }
                (true, Some(h)) => {
                    self.effects.set_position(h, at);
                    if k == 0 {
                        self.effects.update(&table, h, None, &mut self.rng);
                    }
                }
                (false, Some(h)) => {
                    self.effects.stop(&table, h);
                    self.effects.release(h);
                    self.r_fx[k] = None;
                }
                (false, None) => {}
            }
        }
    }

    /// SPEC-0036 Roberta shot projectile, per tick: each shot steps (life, homing, move), ends
    /// below the ground under it, then tests the hittable characters' bone capsules against its
    /// sphere: a hit deals trunc(max(0, S x C x T - defence)), pays a quick shot's cost and
    /// word-2 charge before the take-hit, sends a full release's status request, and disarms.
    fn shots_step(&mut self) {
        let pack = self.world.pack;
        let hero = self.loco.position();
        let mut shots = std::mem::take(&mut self.hero_shots);
        let mut keep = Vec::new();
        for mut s in shots.drain(..) {
            let aim = s.target.and_then(|id| self.characters.iter().find(|c| c.id == id && c.dying.is_none())).map(|c| [c.position[0], c.position[1] + 1.5, c.position[2]]);
            if !s.step(aim) {
                continue;
            }
            let g = self.world.ground_at(s.position[0], s.position[2], s.position[1] + 3.0);
            if g.is_some_and(|h| s.position[1] < h) {
                continue;
            }
            // The first character whose bone capsule meets the shot sphere.
            let hit = self.characters.iter().enumerate().filter(|(_, c)| c.pack == pack && !c.inactive && c.dying.is_none() && !c.enemy.as_ref().is_some_and(|e| e.dead) && c.townsfolk.is_none() && c.row.is_some()).find_map(|(i, c)| {
                let place = placement(c.position, c.facing);
                let frames = c.anim.bone_frames();
                let touches = c.anim.bones().hurt.iter().zip(&frames).any(|(&(base, length, radius), f)| {
                    if length <= 0.0 {
                        return false;
                    }
                    let w = place.mul(f);
                    let p0 = w.apply(base);
                    let axis = unit(w.cols[1]);
                    let v = sub_v(s.position, p0);
                    let t = (v[0] * axis[0] + v[1] * axis[1] + v[2] * axis[2]).clamp(0.0, length);
                    let q = add(p0, scale(axis, t));
                    let d = sub_v(s.position, q);
                    (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt() <= radius + s.radius
                });
                touches.then_some(i)
            });
            let Some(i) = hit else {
                keep.push(s);
                continue;
            };
            let (t_type, defence) = self.characters[i].row.map_or((0, 0.0), |r| (r.target_type as i32, r.defence));
            let mote = self.equipment.motes[1][2];
            let gst = mote.and_then(|id| self.items.get(id)).map_or(0.0, |i| i.stats[6] as f32);
            let c = if neopets_sim::mote::matches(s.element, t_type) { neopets_sim::mote::one_plus_percent(gst) } else { 1.0 };
            // trunc(max(0, S x C x T - defence)), absorbed when S x C x T > 0 and the result <= 0.
            let (damage, absorbed) = neopets_sim::charm::damage(neopets_sim::charm::scale(s.scale, c, s.t, 1.0, 1.0), 1.0, defence, 1.0);
            if s.class < 3 {
                // The quick-shot payment: the class's cost and exactly Q of word-2 charge.
                let (cost, q) = self.magic.roberta_quick(s.class, mote);
                self.pay_mp(cost);
                if let Some(id) = mote {
                    self.spend_charge(2, id, q);
                }
            }
            if damage > 0 || absorbed {
                self.character_take_hit(i, damage, hero, "Roberta's shot ");
            }
            if let Some(r) = s.status {
                self.character_status(i, r);
            }
        }
        self.hero_shots = keep;
    }

    /// SPEC-0029 Behavior 9 Visual update: a shot's flight effects (ids 0 and 1) are effect
    /// objects placed at the shot each tick and updated with no listener; they stop when the
    /// shot ends.
    fn shot_effects_step(&mut self) {
        let table = self.effect_table.clone();
        for k in 0..neopets_sim::projectile::SLOTS {
            let shot = self.shots.slots[k];
            let same = matches!((&self.shot_fx[k], shot), (Some((start, _)), Some(s)) if *start == s.s.start);
            if !same {
                if let Some((_, objects)) = self.shot_fx[k].take() {
                    for h in objects.into_iter().flatten() {
                        self.effects.stop(&table, h);
                        self.effects.release(h);
                    }
                }
                if let Some(s) = shot {
                    let objects = [0, 1].map(|j| {
                        (s.s.effects[j] != -1).then(|| self.effects.add_object(neopets_sim::effect::EffectObject::new(&table, s.s.effects[j], s.position, neopets_sim::effect::IDENTITY)))
                    });
                    self.shot_fx[k] = Some((s.s.start, objects));
                }
            }
            if let (Some(s), Some((_, objects))) = (shot, self.shot_fx[k]) {
                for h in objects.into_iter().flatten() {
                    self.effects.set_position(h, s.position);
                    self.effects.update(&table, h, None, &mut self.rng);
                }
            }
        }
    }

    /// SPEC-0034 1.2 and 5.1: each character's clip effect list. With t the layer-0 cursor, D
    /// the frame count and L the speed, an entry starts its effect while t is in [s, s + L)
    /// (s = start x D, or L for a start of 0) and stops it while t is in [e, e + L) (e = end x
    /// D, at most D - 1); a new list stops every effect of the old one. The point is attach
    /// slot A's frame applied to the entry's offset, with the ground rules of +0x3C and +0x3D;
    /// running effects follow it. Matrix slot k is taken as attach point k (R-0044). Trails,
    /// the positional sound, shake and rumble are not reproduced.
    fn clip_effects_step(&mut self) {
        let table = self.effect_table.clone();
        let live: std::collections::HashSet<u64> = self.characters.iter().map(|c| c.id).collect();
        let gone: Vec<u64> = self.clip_fx.keys().filter(|id| !live.contains(id)).copied().collect();
        for id in gone {
            if let Some((_, handles)) = self.clip_fx.remove(&id) {
                for h in handles.into_iter().flatten() {
                    self.effects.stop(&table, h);
                    self.effects.release(h);
                }
            }
        }
        for i in 0..self.characters.len() {
            let c = &self.characters[i];
            if c.inactive {
                continue;
            }
            let list = c.anim.effect_list().map(|(k, l)| (k, l.to_vec()));
            let window = c.anim.layer0_window();
            let key = list.as_ref().map(|l| l.0);
            let id = c.id;
            let entry = self.clip_fx.entry(id).or_insert((None, Vec::new()));
            if entry.0 != key {
                let old = std::mem::take(&mut entry.1);
                entry.0 = key;
                entry.1 = vec![None; list.as_ref().map_or(0, |l| l.1.len())];
                for h in old.into_iter().flatten() {
                    self.effects.stop(&table, h);
                    self.effects.release(h);
                }
            }
            let (Some((_, entries)), Some((t, d, l))) = (list, window) else { continue };
            for (j, e) in entries.iter().enumerate() {
                if e.id == -1 || e.trail {
                    continue;
                }
                let s = if e.start * d == 0.0 { l } else { e.start * d };
                let end = if e.end * d >= d { d - 1.0 } else { e.end * d };
                let handle = self.clip_fx.get(&id).and_then(|x| x.1.get(j).copied().flatten());
                let (point, frame) = self.clip_point(i, e);
                if handle.is_none() && t >= s && t < s + l {
                    let h = self.effects.add_object(neopets_sim::effect::EffectObject::new(&table, e.id, point, frame));
                    self.last_fx = format!("clip effect {} (list {:?} entry {j}, action {}) on {} on tick {}", e.id, self.clip_fx.get(&id).and_then(|x| x.0), self.characters[i].anim.action(), self.characters[i].name, self.ticks);
                    self.effects.update(&table, h, None, &mut self.rng);
                    if let Some(x) = self.clip_fx.get_mut(&id).and_then(|x| x.1.get_mut(j)) {
                        *x = Some(h);
                    }
                } else if let Some(h) = handle {
                    if t >= end && t < end + l {
                        self.effects.stop(&table, h);
                        self.effects.release(h);
                        if let Some(x) = self.clip_fx.get_mut(&id).and_then(|x| x.1.get_mut(j)) {
                            *x = None;
                        }
                    } else {
                        self.effects.set_position(h, point);
                    }
                }
            }
        }
    }

    /// An entry's point 1 and frame on character i (SPEC-0034 5.1 and 1.1).
    fn clip_point(&self, i: usize, e: &neopets_data::region::EffectEntry) -> ([f32; 3], neopets_sim::effect::Frame) {
        let c = &self.characters[i];
        let place = placement(c.position, c.facing);
        let m = e
            .slot_a
            .and_then(|k| c.anim.bones().attach.get(k as usize).copied())
            .and_then(|(bone, angles, offset)| Some(place.mul(c.anim.bone_frames().get(bone?)?).mul(&attach_rotation(offset, angles))))
            .unwrap_or(place);
        let mut p = m.apply(e.offset_a);
        let world = [&self.world].into_iter().chain(self.other.as_ref()).find(|w| w.pack == c.pack);
        if e.owner_ground {
            p[1] = c.position[1];
        } else if e.probe {
            if let Some(h) = world.and_then(|w| w.ground_at(p[0], p[2], p[1])).filter(|&h| h >= p[1] - 8.0) {
                p[1] = h;
            }
        }
        let frame = if e.full_frame { [m.cols[0], m.cols[1], m.cols[2]] } else { neopets_sim::effect::IDENTITY };
        (p, frame)
    }

    /// A diagnostic one-shot of effect `id` 3 units in front of the hero, 1.5 above its feet.
    pub fn diagnostic_effect(&mut self, id: i32) {
        let (p, f) = (self.loco.position(), self.loco.facing);
        let table = self.effect_table.clone();
        self.effects.one_shot(&table, id, [p[0] + 3.0 * f[0], p[1] + 1.5, p[2] + 3.0 * f[2]], neopets_sim::effect::IDENTITY, &mut self.rng);
    }

    /// The selected hero's name for the speaker rule (SPEC-0022).
    pub fn hero_name(&self) -> &'static [u8] {
        if self.hero_index == 1 { b"Roberta" } else { b"Tor" }
    }

    /// SPEC-0053 3.2: a switch request to the other hero. Refused by the mode, a dead partner
    /// (a parked hero with no hit points) or a running task. Returns whether it was accepted.
    pub fn request_switch(&mut self, extra: bool, effects: bool, flash: bool) -> bool {
        let Some(parked) = self.parked.as_ref() else { return false };
        if self.swap.is_some() || !neopets_sim::swap::allowed(self.swap_mode, self.hero_index) || parked.hp <= 0 {
            self.last_swap = format!("switch refused (mode {}) on tick {}", self.swap_mode, self.ticks);
            return false;
        }
        let dead = self.loco.state == hero_states::DEAD;
        self.swap = Some(neopets_sim::swap::Task::new(1 - self.hero_index, extra, effects, flash, self.loco.state, dead));
        self.last_swap = format!("switch to hero {} accepted on tick {}", 1 - self.hero_index, self.ticks);
        true
    }

    /// SwitchHero(h, a, b) (SPEC-0053 3.2): no request when h is -1 or selected; otherwise a
    /// request with extra flag 0 and the two flags.
    fn script_switch(&mut self, hero: i32, effects: bool, flash: bool) {
        if hero < 0 || hero as usize == self.hero_index {
            return;
        }
        self.request_switch(false, effects, flash);
    }

    /// TriggerActStart(stage) (SPEC-0053 3.7): the swap mode, the hero (stage 1 by a normal
    /// switch request, stages 2 and 3 selected directly with no task and both heroes placed at
    /// the world's entry record 0), the story flag and the heal of the heroes in the mask (hit
    /// points to the maximum, status timers cleared). The Act 2 stash copy and Roberta's fresh
    /// kit (SPEC-0025) are not reproduced (R-0042).
    pub fn act_start(&mut self, stage: i32) {
        let (mode, select, flag, heal) = neopets_sim::swap::act_start(stage);
        self.swap_mode = mode;
        match select {
            Some((h, true)) => {
                if h != self.hero_index {
                    self.request_switch(false, false, false);
                }
            }
            Some((h, false)) => {
                self.swap = None;
                if h != self.hero_index {
                    self.exchange_heroes();
                    self.loco.state = 1;
                }
                if let Some(&e) = self.world.entries.first() {
                    self.arrive(e);
                }
            }
            None => {}
        }
        if let Some(f) = flag {
            self.state.set(f, true);
        }
        for hero in 0..2 {
            if heal & (1 << hero) == 0 {
                continue;
            }
            if hero == self.hero_index {
                self.hero_hp = self.hero_max;
                self.statuses.clear_all();
            } else if let Some(p) = self.parked.as_mut() {
                p.hp = p.hp_max;
            }
        }
        self.last_swap = format!("act start {stage}: mode {mode}, hero {} on tick {}", self.hero_index, self.ticks);
    }

    /// The exchange of SPEC-0053 3.4 steps 6 to 9: the incoming hero takes the outgoing hero's
    /// place and facing (the locomotion record stays), its model and pools come in, its status
    /// timers are cleared, and the renderer re-spawns the hero.
    fn exchange_heroes(&mut self) {
        let Some(mut p) = self.parked.take() else { return };
        std::mem::swap(&mut self.hero, &mut p.anim);
        std::mem::swap(&mut self.model, &mut p.model);
        std::mem::swap(&mut self.hero_hp, &mut p.hp);
        std::mem::swap(&mut self.hero_mp, &mut p.mp);
        std::mem::swap(&mut self.hero_max, &mut p.hp_max);
        std::mem::swap(&mut self.hero_mp_max, &mut p.mp_max);
        self.parked = Some(p);
        self.hero_index = 1 - self.hero_index;
        self.statuses.clear_all();
        self.attack = None;
        self.hero_colour = self.design.hero_colour(self.hero_index).unwrap_or([128; 3]);
        self.weapon = self.equipment.entries[0][self.hero_index];
        self.shield = self.equipment.entries[1][self.hero_index];
        self.spawned = false;
        self.last_swap = format!("hero {} selected on tick {}", self.hero_index, self.ticks);
    }

    /// SPEC-0053 3.8 c and 3.12: the L1 and R1 chord with the switch-ready gate, then the swap
    /// task's step of this tick. Returns whether a swap is in progress (game-state code 5).
    fn swap_step(&mut self, input: &HeroInput) -> bool {
        let (l1, r1) = (std::mem::take(&mut self.l1_latch), std::mem::take(&mut self.r1_latch));
        if self.swap.is_none() && neopets_sim::swap::chord(input.guard, l1, input.run, r1) {
            let r = neopets_sim::swap::Ready {
                dialogue: self.dialogue_open,
                quick_open: self.quick.open,
                code_zero: !self.dialogue_open,
                state: self.loco.state,
                height: self.loco.body.gap.max(0.0),
                grounded: self.loco.body.gap <= 0.15,
                v: neopets_sim::hero::speed_value(input),
                stunned: self.statuses.active(neopets_sim::status::STUN),
                fade_moving: self.fade.alpha != self.fade.target,
            };
            if neopets_sim::swap::ready(&r) {
                self.request_switch(true, true, true);
            }
        }
        let Some(mut task) = self.swap.take() else { return false };
        let done = self.fade.alpha == self.fade.target;
        match task.tick(done) {
            Some(neopets_sim::swap::Step::Start { flash }) => {
                // Step 2: the hero attack pools are reset.
                self.mote_objects.clear();
                self.hero_shots.clear();
                self.r_attack = None;
                if flash {
                    self.fade.out([255; 3], 37);
                }
            }
            Some(neopets_sim::swap::Step::Exchange) => self.exchange_heroes(),
            Some(neopets_sim::swap::Step::FadeIn) => self.fade.fade_in([255; 3], 8),
            Some(neopets_sim::swap::Step::Carry { state, extra }) => {
                self.loco.state = state;
                if extra {
                    self.hud.rearm_all();
                }
            }
            Some(neopets_sim::swap::Step::Done) => {
                self.last_swap = format!("switch done on tick {}", self.ticks);
                return false;
            }
            None => {}
        }
        self.swap = Some(task);
        true
    }

    /// SPEC-0029 Behavior 28 and SPEC-0033 Behavior 18: the "Catch Miamouse" prompt (label 27)
    /// for the first catchable Miamouse within 4.0 in front of a free hero, and the "Drop
    /// Miamouse" prompt (label 28) while carrying; Square catches (state 57) or drops.
    fn catch_step(&mut self) -> bool {
        if self.loco.state == hero_states::CARRY {
            if matches!(self.loco.carry, 1 | 2) && self.carried.is_some() {
                self.door_prompt = Some(28);
                if self.square_now {
                    self.loco.drop_carry();
                }
                return true;
            }
            return false;
        }
        if neopets_sim::hero::busy_state(self.loco.state) || !self.catch.on {
            return false;
        }
        let (hero, f) = (self.loco.position(), self.loco.facing);
        let pack = self.world.pack;
        let Some(id) = self
            .characters
            .iter()
            .filter(|c| c.pack == pack && !c.inactive)
            .find(|c| c.miamouse.as_ref().is_some_and(|m| neopets_sim::miamouse::catchable(&self.catch, m, c.position, hero, f)))
            .map(|c| c.id)
        else {
            return false;
        };
        self.door_prompt = Some(27);
        if self.square_now {
            self.loco.start_carry();
            self.carried = Some(id);
        }
        true
    }

    /// Diagnostic: a point 2.5 behind the nearest catchable Miamouse while catching is on (the
    /// mouse itself once the hero is behind it).
    pub fn mouse_goal(&self) -> Option<[f32; 2]> {
        if !self.catch.on || self.carried.is_some() {
            return None;
        }
        let p = self.loco.position();
        let c = self
            .characters
            .iter()
            .filter(|c| c.pack == self.world.pack && c.miamouse.as_ref().is_some_and(|m| m.catchable))
            .min_by(|a, b| (a.position[0] - p[0]).hypot(a.position[2] - p[2]).total_cmp(&(b.position[0] - p[0]).hypot(b.position[2] - p[2])))?;
        let f = c.facing;
        let behind = [c.position[0] - 2.5 * f[0], c.position[2] - 2.5 * f[2]];
        let d = (behind[0] - p[0]).hypot(behind[1] - p[2]);
        Some(if d < 1.0 { [c.position[0], c.position[2]] } else { behind })
    }

    /// Diagnostic: the cage mesh's middle while a Miamouse is carried.
    pub fn cage_goal(&self) -> Option<[f32; 2]> {
        self.carried?;
        let m = self.world.navmeshes.get(usize::try_from(self.catch.cage).ok()?)?;
        let n = m.len();
        let c = (0..n).map(|t| m.centroid(t)).fold([0.0f32; 3], |a, b| [a[0] + b[0], a[1] + b[1], a[2] + b[2]]);
        // First a point 11 north of the cage's middle (round the posts), then straight south at
        // it, so the hero faces the cage at the fence.
        let (x, z) = (c[0] / n.max(1) as f32, c[2] / n.max(1) as f32);
        let p = self.loco.position();
        let near = (p[0] - x).abs() < 3.0 && p[2] < z + 12.5;
        Some(if near { [x, z] } else { [x + 1.5, z + 11.0] })
    }

    /// Diagnostic: the release point (the held point + 1.0 forward) is in the cage zone.
    pub fn hero_in_cage(&self) -> bool {
        let (h, f) = (self.held_point(), self.loco.facing);
        neopets_sim::miamouse::Catch::in_mesh(&self.world.navmeshes, self.catch.cage, [h[0] + f[0], h[1], h[2] + f[2]])
    }

    /// The point a carried Miamouse is held at: the hero's attach point 0 (SPEC-0005 Attach
    /// points and carrying; the damage-bit choice of points 1 to 4 and the Miamouse's optional
    /// offset are not applied).
    fn held_point(&self) -> [f32; 3] {
        let (p, f) = (self.loco.position(), self.loco.facing);
        let frames = self.hero.bone_frames();
        self.hero
            .bones()
            .attach
            .first()
            .and_then(|&(bone, angles, offset)| Some(placement(p, f).mul(frames.get(bone?)?).mul(&attach_rotation(offset, angles)).t))
            .unwrap_or([p[0] + 0.8 * f[0], p[1] + 1.2, p[2] + 0.8 * f[2]])
    }

    /// The Miamice per tick: the carry hold (from the pick-up clip's second tick, through the put
    /// down's first tick), the release on the tick after clip 181 starts (or at once when state
    /// 57 is left another way), then each free Miamouse's routine and walk, and the delivery
    /// step on the level states.
    fn mice_step(&mut self) {
        let hclip = self.hero.clip_state();
        let held_at = self.held_point();
        let forward = self.loco.facing;
        let carrying = self.loco.state == hero_states::CARRY;
        if let Some(id) = self.carried {
            let release = !carrying || (self.loco.carry == 3 && hclip.action == 181);
            if let Some(c) = self.characters.iter_mut().find(|c| c.id == id) {
                if let (Some(m), Some(wk)) = (c.miamouse.as_mut(), c.walker.as_mut()) {
                    if release {
                        m.release(wk, &self.world.navmeshes, &self.catch, held_at, forward);
                        c.anim.play(neopets_sim::miamouse::CLIP_RELEASED);
                        c.position = wk.position();
                        self.carried = None;
                    } else if !(self.loco.carry == 0 && hclip.action != 178) {
                        wk.place(held_at);
                        c.position = held_at;
                        c.facing = forward;
                        if c.anim.clip_state().action != neopets_sim::miamouse::CLIP_HELD {
                            c.anim.play(neopets_sim::miamouse::CLIP_HELD);
                        }
                    }
                }
            }
        }
        let hero = self.loco.position();
        let hero_state = self.loco.state;
        let pack = self.world.pack;
        let carried = self.carried;
        let mut delivered = 0;
        let world = &self.world;
        for c in self.characters.iter_mut().filter(|c| c.pack == pack && !c.inactive && Some(c.id) != carried) {
            let (Some(m), Some(wk)) = (c.miamouse.as_mut(), c.walker.as_mut()) else { continue };
            // Its own navmesh: the first mesh holding its position, once.
            if wk.mesh.is_none() && m.state != 3 {
                let pick = world.navmeshes.iter().position(|n| n.locate_full(wk.position()).is_some());
                wk.assign_mesh(pick, &world.navmeshes);
            }
            let clip = c.anim.clip_state();
            let v = neopets_sim::miamouse::View { hero, hero_state, clip: clip.action, progress: clip.progress, gap: wk.body.gap };
            let id = m.tick(wk, &world.navmeshes, &self.catch, &v, &mut self.rng);
            if m.delivered {
                delivered += 1;
            }
            if clip.action != id {
                c.anim.play(id);
            }
            let ground = &world.ground;
            wk.motion_tick(c.root_step, &world.navmeshes, |x, z, cl| ground.as_ref().map_or((0.0, UP), |g| g.probe_normal(x, z, cl)));
            c.position = wk.position();
            c.facing = wk.facing;
        }
        for _ in 0..delivered {
            let flags = self.state.level_flags.clone();
            let level = |k: u16| flags.contains(&k);
            let state = &mut self.state;
            let mut set = |k: u16, on: bool| state.set_level(k, on);
            self.catch.deliver(&level, &mut set);
            self.last_pickup = format!("Miamouse delivered ({} in the cage) on tick {}", self.catch.count, self.ticks);
        }
    }

    /// The weapon reach W (SPEC-0021): Tor's table value for swords 98 to 108, 1.5 unarmed.
    fn weapon_reach(&self) -> f32 {
        self.weapon.and_then(|id| self.reach_tor.get(id.checked_sub(98)? as usize).copied()).unwrap_or(1.5)
    }

    /// SPEC-0036 Hero swing, per tick while the attack window (bit 0x1) is on: the weapon
    /// cylinder from attach point 0 (radius 1, length L = W + 2, base P0 = A + 0.25 D1) against
    /// each bone capsule of every hittable character within 65.0 (static overlap, then the swept
    /// test), skipping the hit list; a hit deals the melee damage of SPEC-0036 (no Mote, no
    /// petpet or status factors). Uses this tick's poses (the original reads the previous pass).
    fn hero_swing(&mut self) {
        let (Some(a), true) = (self.attack.as_ref(), self.hero.frame_bits() & 1 != 0) else {
            self.last_axis = None;
            return;
        };
        let sub = a.sub;
        let p = self.loco.position();
        let place = placement(p, self.loco.facing);
        let frames = self.hero.bone_frames();
        let Some(&(Some(bone), angles, offset)) = self.hero.bones().attach.first() else { return };
        let Some(bw) = frames.get(bone) else { return };
        let w = place.mul(bw).mul(&attach_rotation(offset, angles));
        let (anchor, axis) = (w.t, unit(w.cols[2]));
        let l = self.weapon_reach() + 2.0;
        let p0 = add(anchor, scale(axis, 0.25));
        let weapon = neopets_sim::combat::Cylinder { center: add(p0, scale(axis, 0.5 * l)), axis, radius: 1.0, length: l };
        let d2 = self.last_axis.replace(axis);
        let pack = self.world.pack;
        let stat_a = self.weapon.and_then(|id| self.items.get(id)).filter(|i| i.is_weapon()).map_or(0, |i| i.stats[0]);
        let g = self.statuses.damage_scale();
        // SPEC-0036 Damage: the attack Mote in word 2 gives B and C; P is the attack power table
        // value only in the power step (sub-state 10).
        let mote_id = self.equipment.motes[0][2];
        let mote = mote_id.and_then(|id| self.items.get(id)).map(|i| neopets_sim::mote::AttackMote { f: i.stats[5] as f32, g: i.stats[6] as f32, kind: neopets_sim::mote::element(mote_id) });
        let power = match mote_id {
            Some(id) if sub == 10 => neopets_sim::mote::power_index(0, id).and_then(|k| self.design.attack_power(k)).unwrap_or(1.0),
            _ => 1.0,
        };
        let mut landed = 0;
        let hero_at = self.loco.position();
        // Factor E: the live petpet of kind 0 or 7 (SPEC-0047 Passive bonuses).
        let e_factor = self.pet_bonus.damage_factor(self.petpet.as_ref().map(|p| p.kind), self.petpet.as_ref().map_or(0, |p| p.t));
        let mut hits = Vec::new();
        for (i, c) in self.characters.iter_mut().enumerate().filter(|(_, c)| c.pack == pack && !c.inactive && c.dying.is_none() && !c.enemy.as_ref().is_some_and(|e| e.dead)) {
            if self.hit_list.contains(&c.id) {
                continue;
            }
            let d = sub_v(c.position, p);
            if (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt() > 65.0 {
                continue;
            }
            let cplace = placement(c.position, c.facing);
            let cframes = c.anim.bone_frames();
            let hit = c.anim.bones().hurt.iter().zip(&cframes).any(|(&(base, length, radius), f)| {
                if length <= 0.0 {
                    return false;
                }
                let wf = cplace.mul(f);
                let cap = neopets_sim::combat::Cylinder::from_base(wf.apply(base), unit(wf.cols[1]), radius, length);
                neopets_sim::combat::overlap(&weapon, &cap)
                    || d2.is_some_and(|d2| {
                        let (b, cc) = (add(p0, scale(axis, l)), add(p0, scale(d2, l)));
                        b != cc && neopets_sim::combat::swept(p0, b, cc, &cap)
                    })
            });
            if !hit {
                continue;
            }
            self.hit_list.push(c.id);
            // SPEC-0036 Damage: x = P A B C D E (P, B, C, E are 1 without a Mote or petpet), y = x
            // − defence, truncated; D by sub-state.
            let d_factor = match sub {
                4 | 12 => 2.0,
                5 | 6 => 0.75,
                _ => 1.0,
            };
            // y = (x - defence) x G, the attacker's status scale (curse 0.75, strength 1.5); E is
            // 1 (no petpet).
            let t = c.row.map_or(0, |r| r.target_type as i32);
            let (damage, absorbed) = neopets_sim::mote::melee(power, stat_a as f32, mote, t, d_factor, e_factor, c.row.map_or(0.0, |r| r.defence), g);
            if damage > 0 || absorbed {
                landed += 1;
            }
            // A boss takes its hits through its own controller (SPEC-0015 counted hits).
            if c.boss.is_some() {
                self.boss_hits.push((c.id, damage));
                continue;
            }
            hits.push((i, damage));
        }
        // SPEC-0036 Damage, take-hit: in the power step a status request of the word-2 Mote
        // also goes to the target (SPEC-0029 Behavior 27.1).
        let request = mote_id.filter(|_| sub == 10).and_then(|id| neopets_sim::mote::status_request(0, id, |k| self.design.attack_power(k)));
        for &(i, _) in &hits {
            if let Some(e) = self.characters.get(i).and_then(|c| c.entity) {
                self.state.hero.hits.push(e);
            }
        }
        for (i, damage) in hits {
            self.character_take_hit(i, damage, hero_at, "");
            if let Some(r) = request {
                self.character_status(i, r);
            }
        }
        // A landed Tor hit outside the power step pays the level-0 cost, then uses word-2 charge.
        if sub != 10 {
            for _ in 0..landed {
                let (cost, q) = self.magic.tor_swing(mote_id);
                self.pay_mp(cost);
                if let Some(id) = mote_id {
                    let n = neopets_sim::mote::charge_tor(q, self.weapon_stat_b());
                    self.spend_charge(2, id, n);
                }
            }
        }
    }

    /// A hit of `damage` from the hero at `from` on character i: SPEC-0038 Behavior 37, the
    /// common enemy's take-hit hook (doubling on entering agro, the hit reaction, facing the
    /// attacker), then the death hook at 0 HP; SPEC-0056, a townsfolk take-hit returns at once.
    fn character_take_hit(&mut self, i: usize, damage: i32, from: [f32; 3], by: &str) {
        let hero_at = from;
        // The script's undamageable entities (SPEC-0044 1056 Update) take no damage.
        let pack = self.world.pack;
        let damage = if self.characters.get(i).and_then(|c| c.entity).is_some_and(|e| level::undamageable(pack).contains(&e)) { 0 } else { damage };
        let Some(c) = self.characters.get_mut(i) else { return };
        {
            if let Some(e) = c.enemy.as_mut() {
                // SPEC-0023 10.1: the take-hit arms the monster bar before the damage.
                if !e.dead {
                    let width = self.name_font.as_ref().map_or(10, |(f, sp)| f.measure(c.name.as_bytes(), *sp));
                    let max = c.row.map_or(1, |r| r.hit_points as i32);
                    self.monster_bar.arm(c.id, &c.name, width, c.hp, max, &self.monster_words);
                }
                let (applied, toward) = e.take_hit(damage, hero_at, &mut self.rng);
                if let Some(w) = c.walker.as_mut() {
                    w.steer_to(toward, 1.0);
                }
                c.hp -= applied;
                if c.hp <= 0 && !e.dead {
                    c.hp = 0;
                    e.die();
                    self.payouts.push((c.pack, c.position, c.drop));
                }
            } else if damage > 0 && c.townsfolk.is_none() {
            // SPEC-0056: the take-hit of a townsfolk character returns at once.
                c.hp -= damage;
                if c.hp <= 0 {
                    c.dying = Some(c.row.map_or(60, |r| r.disappear.max(1) as u32));
                }
            }
            self.last_hit = format!("{by}hit {} for {damage} (hp {})", if c.name.is_empty() { "(unnamed)" } else { &c.name }, c.hp.max(0));
        }
    }

    /// A status request to character i: the enemy's apply (SPEC-0029 Behavior 27.1); other
    /// characters ignore it.
    fn character_status(&mut self, i: usize, [code, d, a, t]: [i32; 4]) {
        let Some(c) = self.characters.get_mut(i) else { return };
        if let Some(e) = c.enemy.as_mut() {
            if e.apply_status(code, d, a, t) {
                self.last_status = format!("status {code} on {} ({d} s, {a}, {t}) on tick {}", c.name, self.ticks);
            }
        }
    }

    /// SPEC-0036 Charm attack hits, per tick: each live Mote object updates, then walks the
    /// hittable characters of the world against their body cylinders; a target not on its
    /// list (and with a free slot) is listed and takes trunc((scale x C - defence) x P).
    fn mote_objects_step(&mut self) {
        let hero = self.loco.position();
        let pack = self.world.pack;
        let mut objects = std::mem::take(&mut self.mote_objects);
        objects.retain_mut(|o| o.tick(hero));
        for o in objects.iter_mut() {
            let p = neopets_sim::mote::power_index(0, o.id).and_then(|k| self.design.attack_power(k)).unwrap_or(1.0);
            let g = self.items.get(o.id).map_or(0.0, |i| i.stats[6] as f32);
            let mut hits = Vec::new();
            for (i, c) in self.characters.iter().enumerate().filter(|(_, c)| c.pack == pack && !c.inactive && c.dying.is_none() && !c.enemy.as_ref().is_some_and(|e| e.dead)) {
                let Some(r) = c.row else { continue };
                if !o.touches(c.position, r.radius, r.height) || !o.list(c.id) {
                    continue;
                }
                let t = r.target_type as i32;
                let cf = if neopets_sim::mote::matches(neopets_sim::mote::element(Some(o.id)), t) { neopets_sim::mote::one_plus_percent(g) } else { 1.0 };
                let (damage, _) = neopets_sim::charm::damage(o.scale, cf, r.defence, p);
                hits.push((i, damage));
            }
            // Tor's Lava orbit sends fire with Tor's Lava values (SPEC-0029 Behavior 27.1).
            let request = (o.id == 161).then(|| neopets_sim::mote::status_request(0, 161, |k| self.design.attack_power(k))).flatten();
            for (i, damage) in hits {
                self.character_take_hit(i, damage, hero, &format!("Mote {} object ", o.id));
                if let Some(r) = request {
                    self.character_status(i, r);
                }
            }
        }
        self.mote_objects = objects;
    }

    /// Stat B (S) of Tor's weapon entry; 0 with no weapon.
    fn weapon_stat_b(&self) -> f32 {
        self.weapon.and_then(|id| self.items.get(id)).map_or(0.0, |i| i.stats[1] as f32)
    }

    /// Spending more magic than the hero has sets it to 0 (SPEC-0036 Magic).
    fn pay_mp(&mut self, cost: i32) {
        if cost > 0 {
            self.hero_mp = (self.hero_mp - cost).max(0);
        }
    }

    /// SPEC-0036 Mote charge use: a use of n below the remaining charge subtracts n; otherwise
    /// the Mote leaves the inventory and every equipment word holding it is cleared.
    fn spend_charge(&mut self, word: usize, id: u16, n: i32) {
        if n <= 0 {
            return;
        }
        // SPEC-0023 section 4: the use re-arms the panel of the word it spends from.
        self.hud.rearm_word(word);
        let left = self.state.items.get(&id).copied().unwrap_or(0);
        if (n as u32) < left {
            self.state.items.insert(id, left - n as u32);
        } else {
            self.state.items.remove(&id);
            for w in self.equipment.motes.iter_mut().flatten() {
                if *w == Some(id) {
                    *w = None;
                }
            }
        }
        self.last_mote = format!("Mote {id} used {n} charge ({} left), mp {} on tick {}", self.state.items.get(&id).copied().unwrap_or(0), self.hero_mp, self.ticks);
    }

    /// SPEC-0036 Power-ready check, each held tick from F = 50: with a Mote in word 2 the hero
    /// needs a weapon, the Mote held with charge above 0 and MP at least the release cost; a
    /// failure returns F to 0 (the HUD timers of the failure are not driven).
    fn power_ready_step(&mut self) {
        if !self.attack.as_ref().is_some_and(|a| a.sub == 1 && a.f >= 50) {
            return;
        }
        let Some(id) = self.equipment.motes[0][2] else { return };
        let held = self.state.items.get(&id).copied().unwrap_or(0) > 0;
        let ready = self.mote_objects.is_empty() && self.weapon.is_some() && held && self.hero_mp >= self.magic.tor_power(Some(id)).0;
        if !ready {
            if let Some(a) = self.attack.as_mut() {
                a.f = 0;
            }
            self.last_mote = format!("power attack not ready (Mote {id}, mp {}) on tick {}", self.hero_mp, self.ticks);
        }
    }

    /// SPEC-0036 Power attack and full release, Tor: at release, word-2 charge (level 1), then
    /// the MP cost; no damage at release (the Mote power-attack object is not spawned).
    fn power_release(&mut self) {
        let mote = self.equipment.motes[0][2];
        let (cost, q) = self.magic.tor_power(mote);
        if let Some(id) = mote {
            let n = neopets_sim::mote::charge_tor(q, self.weapon_stat_b());
            self.spend_charge(2, id, n);
            // The object of the word-2 Mote at the hero position plus facing x reach, storing
            // the scale A x D x B x G x E (D = 1 at sub-state 10; E = 1, no petpet).
            let hero = self.loco.position();
            let f = self.loco.facing;
            let w = self.weapon_reach();
            let at = [hero[0] + f[0] * w, hero[1], hero[2] + f[2] * w];
            let a = self.weapon.and_then(|i| self.items.get(i)).filter(|i| i.is_weapon()).map_or(0.0, |i| i.stats[0] as f32);
            let b = neopets_sim::mote::one_plus_percent(self.items.get(id).map_or(0.0, |i| i.stats[5] as f32));
            let e = self.pet_bonus.damage_factor(self.petpet.as_ref().map(|p| p.kind), self.petpet.as_ref().map_or(0, |p| p.t));
            let scale = neopets_sim::charm::scale(a, 1.0, b, self.statuses.damage_scale(), e);
            if let Some(o) = neopets_sim::charm::spawn_tor(id, &self.charm_words, at, hero, f, scale) {
                self.last_mote = format!("Mote {id} power-attack object (scale {scale}, {} ticks) on tick {}", o.life, self.ticks);
                self.mote_objects.push(o);
            }
        }
        self.pay_mp(cost);
    }

    /// SPEC-0036 Enemy attack damage to a hero (shared enemy class): the shield block (with the
    /// shield Mote's charge and cost), the armour Mote and the body item, for an attack of
    /// damage d0, element `element` and the attacker's status scale G, with the petpet factor.
    fn enemy_damage(&mut self, d0: f32, blocked: bool, element: i32, g: f32) -> f32 {
        use neopets_sim::mote;
        let stat = |p: &Self, id: u16, k: usize| p.items.get(id).map_or(0.0, |i| i.stats[k] as f32);
        let mut b = 0.0;
        if let (true, Some(shield)) = (blocked, self.equipment.entries[1][0]) {
            let m1 = self.equipment.motes[0][1];
            b = mote::shield_block(stat(self, shield, 0), m1.is_some());
            if let Some(m1) = m1.filter(|&m| mote::element_passes(element, mote::element(Some(m)))) {
                // s is the shield's stat B for type 5 (Tor's shields), stat D for type 3.
                let s = match self.item_kind(shield) {
                    5 => stat(self, shield, 1),
                    3 => stat(self, shield, 3),
                    _ => 0.0,
                };
                let (cost, q) = mote::shield_cost(m1);
                self.spend_charge(1, m1, mote::charge_tor(q, s));
                self.pay_mp(cost);
            }
        }
        let mut d = mote::less_block(d0, b);
        if let Some(body) = self.equipment.entries[2][0] {
            if let Some(m0) = self.equipment.motes[0][0] {
                let (cost, q0) = self.magic.armour(0, Some(m0));
                let charge = self.state.items.get(&m0).copied().unwrap_or(0) as i32;
                if q0 <= charge && self.hero_mp >= cost && mote::element_passes(element, mote::element(Some(m0))) {
                    d = mote::armour_mote(d, stat(self, m0, 7));
                    self.spend_charge(0, m0, mote::charge_armour(q0, stat(self, body, 1)));
                    self.pay_mp(cost);
                }
            }
            d = mote::armour_body(d, stat(self, body, 0));
        }
        // The petpet factor 0.9 for kinds 1 and 7 (SPEC-0047 Passive bonuses).
        d = mote::scaled(d, self.pet_bonus.intake_factor(self.petpet.as_ref().map(|p| p.kind), self.petpet.as_ref().map_or(0, |p| p.t)));
        mote::taken(d, g) as f32
    }

    /// Diagnostic fight goal: a stunned boss, else the nearest living active class-17 character
    /// of the current world, and
    /// whether it is within 4.5 units.
    pub fn fight_goal(&self) -> Option<([f32; 2], bool)> {
        let p = self.loco.position();
        // A Juppie in a stage with no living enemy about: stand on the stage's first unused
        // locator so its slam reaches it (diagnostic only).
        let enemies = self.characters.iter().any(|c| c.pack == self.world.pack && !c.inactive && c.enemy.as_ref().is_some_and(|e| !e.dead) && c.dying.is_none());
        if let Some((c, b)) = self.characters.iter().filter(|c| c.pack == self.world.pack && !c.held).find_map(|c| Some((c, c.boss.as_ref()?))) {
            let stunned = matches!(b.top, neopets_sim::juppie::Top::Slam(2 | 3));
            if stunned {
                // The stunned head (bone 15) lies on the ground: go for it.
                let head = c.anim.bone_frames().get(neopets_sim::juppie::HEAD_BONE).map(|f| placement(c.position, c.facing).mul(f).t);
                if let Some(h) = head {
                    let d = (h[0] - p[0]).hypot(h[2] - p[2]);
                    return Some((if d < 9.0 { [p[0], p[2]] } else { [h[0], h[2]] }, d < 9.0));
                }
            }
            if !enemies && !stunned && b.stage > 0 {
                let w = &self.world;
                let spot = neopets_sim::juppie::stage_locators(b.stage)
                    .iter()
                    .filter(|(_, f)| !self.state.level(*f))
                    .find_map(|&(k, _)| w.tags.iter().position(|&t| t as usize == k).and_then(|i| w.entries.get(i)));
                if let Some((at, _)) = spot {
                    return Some(([at[0], at[2]], false));
                }
            }
        }
        let c = self
            .characters
            .iter()
            .filter(|c| c.pack == self.world.pack && c.dying.is_none() && !c.inactive && !c.enemy.as_ref().is_some_and(|e| e.dead))
            // A stunned boss is fought first; otherwise the nearest class-17 enemy.
            .filter(|c| c.row.is_some_and(|r| r.class == Some(17)) || c.boss.as_ref().is_some_and(|b| matches!(b.top, neopets_sim::juppie::Top::Slam(2 | 3))))
            .min_by(|a, b| {
                let d = |c: &Character| if c.boss.is_some() { 0.0 } else { (c.position[0] - p[0]).hypot(c.position[2] - p[2]) };
                d(a).total_cmp(&d(b))
            })?;
        let d = (c.position[0] - p[0]).hypot(c.position[2] - p[2]);
        Some(([c.position[0], c.position[2]], d < 4.5))
    }

    /// Diagnostic herding goal: 3.5 behind the nearest Whinny that is not penned, on the far
    /// side from the pen point.
    pub fn herd_goal(&self) -> Option<[f32; 2]> {
        let pen = self.world.pens.first()?.point;
        let p = self.loco.position();
        let w = self
            .characters
            .iter()
            .filter(|c| c.pack == self.world.pack && c.whinny.as_ref().is_some_and(|w| !w.penned()))
            .min_by(|a, b| {
                let d = |c: &Character| (c.position[0] - p[0]).hypot(c.position[2] - p[2]);
                d(a).total_cmp(&d(b))
            })?;
        let (dx, dz) = (w.position[0] - pen[0], w.position[2] - pen[2]);
        let len = dx.hypot(dz).max(1e-6);
        Some([w.position[0] + 3.5 * dx / len, w.position[2] + 3.5 * dz / len])
    }

    /// Diagnostic: put the hero at (x, z) on the current world's ground, probed from height c.
    pub fn place_at(&mut self, x: f32, z: f32, c: f32) {
        let y = self.world.ground_at(x, z, c).unwrap_or(self.loco.position()[1]);
        let f = self.loco.facing;
        self.arrive(([x, y, z], f));
    }

    /// Diagnostic: turn the hero toward a point and put the camera behind it.
    pub fn face_point(&mut self, x: f32, z: f32) {
        let p = self.loco.position();
        self.arrive((p, [x - p[0], 0.0, z - p[2]]));
    }

    /// Take a loaded region's characters, when the slot still wants that region.
    #[allow(clippy::type_complexity)]
    pub fn add_characters(
        &mut self,
        pack: u32,
        slot: u8,
        region: u32,
        loaded: Vec<(neopets_data::region::Placement, ModelData, HeroAnim, Option<neopets_data::monster::Row>, Option<neopets_data::monster::Tuning>)>,
        items: Vec<neopets_data::region::ItemPoint>,
    ) {
        if self.slot_regions.get(&(pack, slot)) != Some(&region) {
            return;
        }
        if slot != REBIND_SLOT {
            self.region_items(pack, &items);
            self.characters.retain(|c| !(c.pack == pack && c.slot == slot));
        }
        if slot == PET_SLOT {
            self.pet_loaded(pack, loaded);
            return;
        }
        // A forced bind: only the wanted embedded records come in, as embedded characters.
        let wanted: Vec<usize> = if slot == REBIND_SLOT {
            let w = self.rebind.iter().filter(|(p, _)| *p == pack).map(|&(_, o)| o).collect();
            self.rebind.retain(|(p, _)| *p != pack);
            w
        } else {
            Vec::new()
        };
        let loaded: Vec<_> = if slot == REBIND_SLOT { loaded.into_iter().filter(|(pl, ..)| wanted.contains(&pl.ordinal)).collect() } else { loaded };
        let (slot, region) = if slot == REBIND_SLOT {
            self.slot_regions.remove(&(pack, REBIND_SLOT));
            (EMBEDDED_SLOT, pack)
        } else {
            (slot, region)
        };
        for (pl, model, anim, row, tuning) in loaded {
            self.next_character += 1;
            // Characters have motion records and settle on the ground under their placement
            // (the ground step probes 2.5 above; SPEC-0018). The fall itself is not simulated.
            let world = [&self.world].into_iter().chain(self.other.as_ref()).find(|w| w.pack == pack);
            let position = match world.and_then(|w| w.ground_at(pl.position[0], pl.position[2], pl.position[1] + 2.5)) {
                Some(h) => [pl.position[0], h, pl.position[2]],
                None => pl.position,
            };
            self.characters.push(Character {
                id: self.next_character,
                pack,
                slot,
                entity: self.entity_of(pack, region, pl.ordinal),
                name: pl.name,
                position,
                facing: pl.facing,
                townsfolk: row.and_then(|r| r.class).filter(|&c| c == 6).map(|_| neopets_sim::npc::Townsfolk::new(pl.ai_code, pl.facing)),
                walker: row.filter(|r| matches!(r.class, Some(7 | 21)) || (r.class == Some(17) && r.can_fly == 0 && tuning.is_some())).map(|r| {
                    neopets_sim::walker::Walker::new(position, pl.facing, r.radius, r.weight.max(0.1), self.next_character as u32)
                }),
                whinny: row.filter(|r| r.class == Some(7)).map(|_| neopets_sim::whinny::Whinny::default()),
                enemy: match (row, tuning) {
                    (Some(r), Some(t)) if r.class == Some(17) && r.can_fly == 0 => Some(Enemy::new(
                        params(&r, &t),
                        enemy::Spawn { default_state: pl.ai_code as u16, home: position, emerge_distance: pl.emerge_distance, selector: pl.selector },
                        &mut self.rng,
                    )),
                    _ => None,
                },
                weapons: tuning.map_or([(0.0, 0.0); 5], |t| t.weapons),
                drop: usize::try_from(pl.drop).ok().and_then(|i| items.get(i).copied()),
                colour: pl.colour,
                root_step: [0.0; 3],
                script_walk: None,
                held: false,
                inactive: self.entity_of(pack, region, pl.ordinal).is_some_and(|e| self.pending_off.contains(&(pack, e))),
                deactivated: self.entity_of(pack, region, pl.ordinal).is_some_and(|e| self.pending_off.contains(&(pack, e))),
                boss: row.filter(|r| r.class == Some(18)).map(|_| neopets_sim::juppie::Juppie::default()),
                miamouse: row.filter(|r| r.class == Some(21)).map(|_| neopets_sim::miamouse::Miamouse::new(position)),
                hp: row.map_or(1, |r| r.hit_points as i32),
                dying: None,
                sound_action: None,
                emotes: Default::default(),
                sound_blocks: anim.sound_blocks(),
                row,
                anim,
                model,
            });
        }
    }

    /// The petpet package's character record ordinal 0 becomes the live petpet's character at
    /// the spawn point, playing animation 6 (SPEC-0047 Spawn).
    #[allow(clippy::type_complexity)]
    fn pet_loaded(&mut self, pack: u32, loaded: Vec<(neopets_data::region::Placement, ModelData, HeroAnim, Option<neopets_data::monster::Row>, Option<neopets_data::monster::Tuning>)>) {
        let (Some(_), Some(at), Some((pl, model, mut anim, row, _))) = (self.petpet.as_ref(), self.pet_spawn.take(), loaded.into_iter().next()) else { return };
        self.characters.retain(|c| Some(c.id) != self.pet_char);
        self.next_character += 1;
        let position = match self.world.ground_at(at[0], at[2], at[1] + 2.5) {
            Some(h) => [at[0], h, at[2]],
            None => at,
        };
        let facing = self.loco.facing;
        let (radius, weight) = row.map_or((0.5, 1.0), |r| (r.radius, r.weight.max(0.1)));
        anim.play(6);
        self.pet_char = Some(self.next_character);
        self.characters.push(Character {
            id: self.next_character,
            pack,
            slot: PET_SLOT,
            entity: None,
            name: pl.name,
            position,
            facing,
            townsfolk: None,
            walker: Some(neopets_sim::walker::Walker::new(position, facing, radius, weight, self.next_character as u32)),
            whinny: None,
            enemy: None,
            weapons: [(0.0, 0.0); 5],
            drop: None,
            colour: pl.colour,
            root_step: [0.0; 3],
            script_walk: None,
            held: false,
            inactive: false,
            deactivated: false,
            boss: None,
            miamouse: None,
            hp: row.map_or(1, |r| r.hit_points as i32),
            dying: None,
            sound_action: None,
            emotes: Default::default(),
            sound_blocks: anim.sound_blocks(),
            row,
            anim,
            model,
        });
        self.last_pet = format!("petpet {} (kind {}) spawned at {position:?} on tick {}", self.petpet.as_ref().map_or(-1, |p| p.type_id), self.petpet.as_ref().map_or(0, |p| p.kind), self.ticks);
    }

    /// SPEC-0047 Activation by type (fresh): the current petpet is deactivated, then the kind's
    /// package (resource 534 + kind) loads and the petpet spawns 7 units ahead of the hero with
    /// T and F. A type with no kind spawns nothing.
    pub fn activate_pet(&mut self, type_id: i32, t: i32, f: i32) {
        let Some(live) = neopets_sim::petpet::Live::new(type_id, t, f) else { return };
        self.deactivate_pet();
        let region = neopets_sim::petpet::package(live.kind);
        self.petpet = Some(live);
        self.pet_spawn = Some(neopets_sim::petpet::spawn_point(self.loco.position(), self.loco.facing));
        let pack = self.world.pack;
        self.slot_regions.insert((pack, PET_SLOT), region);
        self.region_wanted.push((pack, PET_SLOT, region));
    }

    /// Deactivate: the petpet and its character are removed.
    pub fn deactivate_pet(&mut self) {
        self.petpet = None;
        if let Some(id) = self.pet_char.take() {
            self.characters.retain(|c| c.id != id);
        }
        self.slot_regions.retain(|(_, s), _| *s != PET_SLOT);
    }

    /// The live petpet's present kind (T at least 1).
    pub fn pet_kind(&self) -> Option<u8> {
        self.petpet.as_ref().and_then(|p| p.present())
    }

    /// One petpet tick (SPEC-0047 class 34): the controller, its animation request and walk; T
    /// counts down only in the in-play state (no dialogue box). The petpet follows the hero
    /// from world to world: it is moved to the current world and, when it is more than 60 units
    /// away (a new world), placed 7 units ahead of the hero (unverified; R-0035).
    fn pet_step(&mut self) {
        let Some(id) = self.pet_char else { return };
        let hero = self.loco.position();
        let hf = self.loco.facing;
        let idle = matches!(self.loco.state, hero_states::STANCE);
        let counting = !self.dialogue_open;
        let pack = self.world.pack;
        let Some(c) = self.characters.iter_mut().find(|c| c.id == id) else { return };
        let Some(live) = self.petpet.as_mut() else { return };
        if c.pack != pack || neopets_sim::petpet::Live::far(c.position, hero) {
            c.pack = pack;
            let at = neopets_sim::petpet::spawn_point(hero, hf);
            let y = self.world.ground_at(at[0], at[2], at[1] + 2.5).unwrap_or(hero[1]);
            c.position = [at[0], y, at[2]];
            c.walker = Some(neopets_sim::walker::Walker::new(c.position, hf, c.row.map_or(0.5, |r| r.radius), c.row.map_or(1.0, |r| r.weight.max(0.1)), id as u32));
        }
        let clip = c.anim.clip_state();
        let v = neopets_sim::petpet::View {
            position: c.position,
            facing: c.facing,
            hero,
            hero_facing: hf,
            hero_right: [hf[2], 0.0, -hf[0]],
            hero_idle: idle,
            counting,
            contact: false,
            clip_done: clip.progress >= 0.99,
        };
        let act = live.tick(&v, self.pet_low, &mut self.rng);
        if let Some(a) = act.anim {
            if clip.action != a {
                c.anim.play(a);
            }
        }
        if let Some(wk) = c.walker.as_mut() {
            if let Some((t, w)) = act.toward {
                wk.steer_to(t, w);
            }
            let world = &self.world;
            let ground = &world.ground;
            wk.motion_tick(c.root_step, &world.navmeshes, |x, z, cl| ground.as_ref().map_or((0.0, UP), |g| g.probe_normal(x, z, cl)));
            c.position = wk.position();
            c.facing = wk.facing;
        }
        if act.leave {
            self.last_pet = format!("petpet left on tick {}", self.ticks);
            self.deactivate_pet();
        }
    }

    /// SPEC-0015 per tick for each Juppie of the current world that the script does not hold:
    /// its controller with the head bone (15), the boss-side action-frame query, the level
    /// flags, the stage locators and this tick's swing hits; then its animation, turn, slam hit
    /// on the hero (raw, with the push along the reverse of the hero's facing), HP loss, level
    /// flags, death sequence and the head cylinder blocker.
    fn boss_step(&mut self) {
        let hits = std::mem::take(&mut self.boss_hits);
        let pack = self.world.pack;
        let hero = self.loco.position();
        let hf = self.loco.facing;
        let ids: Vec<u64> = self.characters.iter().filter(|c| c.pack == pack && c.boss.is_some() && !c.held && !c.inactive).map(|c| c.id).collect();
        for id in ids {
            let Some(i) = self.characters.iter().position(|c| c.id == id) else { continue };
            let level_flags = self.state.level_flags.clone();
            let level = |f: u16| level_flags.contains(&f);
            let world = &self.world;
            // The locator lookup by tag: the first entry record with the tag.
            let locator = |k: usize| world.tags.iter().position(|&t| t as usize == k).and_then(|i| world.entries.get(i)).map(|e| e.0);
            let c = &mut self.characters[i];
            let place = placement(c.position, c.facing);
            let frames = c.anim.bone_frames();
            let head = frames.get(neopets_sim::juppie::HEAD_BONE).map_or([c.position[0], c.position[1] + 2.0, c.position[2]], |f| place.mul(f).t);
            let clip = c.anim.clip_state();
            let v = neopets_sim::juppie::View {
                position: c.position,
                facing: c.facing,
                head,
                hero,
                clip: (clip.action, clip.progress, clip.progress >= 0.99),
                can_act: c.anim.frame_bits() & 0x1F != 0,
                hp: c.hp,
                max_hp: c.row.map_or(1, |r| r.hit_points as i32),
                level: &level,
                locator: &locator,
                hero_hit: hits.iter().find(|(h, _)| *h == id).map(|&(_, d)| d),
            };
            let Some(boss) = c.boss.as_mut() else { continue };
            let before = boss.top;
            let act = boss.tick(&v, &self.juppie_words, &mut self.rng);
            let after = boss.top;
            if let Some(a) = act.anim {
                if clip.action != a || (clip.progress >= 0.99 && before != after) {
                    c.anim.play(a);
                }
            }
            if let Some(rate) = act.face {
                let to = [hero[0] - c.position[0], 0.0, hero[2] - c.position[2]];
                c.facing = neopets_sim::hero::steer(c.facing, to, rate);
            }
            c.hp -= act.hp_loss;
            if act.death {
                c.hp = 1;
            }
            if act.hp_loss > 0 {
                self.last_boss = format!("Juppie lost {} (hp {}) on tick {}", act.hp_loss, c.hp, self.ticks);
            }
            if before != after {
                // With the stage locators' horizontal distance and facing dot from the boss
                // and their distance from the head, for the slam's stage events.
                let locs: Vec<String> = (101..=106)
                    .filter_map(|k| {
                        let p = locator(k)?;
                        let (dx, dz) = (p[0] - c.position[0], p[2] - c.position[2]);
                        let l = dx.hypot(dz).max(1e-6);
                        let dot = (dx * c.facing[0] + dz * c.facing[2]) / l;
                        let h = ((head[0] - p[0]).powi(2) + (head[1] - p[1]).powi(2) + (head[2] - p[2]).powi(2)).sqrt();
                        Some(format!("{k}: {l:.1} dot {dot:.2} head {h:.1}"))
                    })
                    .collect();
                self.last_boss = format!("Juppie {:?} -> {:?} (stage {}, hp {}) on tick {}; at {:.1},{:.1}; locators [{}]", before, after, boss.stage, c.hp, self.ticks, c.position[0], c.position[2], locs.join(", "));
            }
            for &f in &act.set_level {
                self.state.set_level(f, true);
            }
            for &f in &act.clear_level {
                self.state.set_level(f, false);
            }
            if act.death {
                // Every other living non-townsfolk character takes 10000.
                for o in self.characters.iter_mut().filter(|o| o.id != id && o.pack == pack && o.townsfolk.is_none() && o.dying.is_none()) {
                    if let Some(e) = o.enemy.as_mut() {
                        if !e.dead {
                            o.hp = 0;
                            e.die();
                        }
                    } else if o.boss.is_none() && o.whinny.is_none() && o.row.is_some() {
                        o.hp -= 10000;
                    }
                }
            }
            if let Some((damage, push)) = act.hero_damage {
                self.hero_hit(damage as f32, push, false, [-hf[0], 0.0, -hf[2]], "the Juppie's head slam".into());
                // The shared hit flash 222, detached, at the impact point (SPEC-0015; SPEC-0029).
                let table = self.effect_table.clone();
                self.effects.one_shot(&table, 222, head, neopets_sim::effect::IDENTITY, &mut self.rng);
            }
            if let Some(p) = act.blocker {
                // The head cylinder (radius 4.0, length 10.0) pushes the hero's body out.
                let pos = self.loco.position();
                let (dx, dz) = (pos[0] - p[0], pos[2] - p[2]);
                let d = (dx * dx + dz * dz).sqrt();
                let r = 4.0 + contact::HERO_RADIUS;
                if d < r && d > 1e-4 && (pos[1] - p[1]).abs() <= 5.0 {
                    let k = (r - d) / d;
                    let b = &mut self.loco.body.position;
                    b[0] += dx * k;
                    b[2] += dz * k;
                }
            }
        }
    }

    /// Whether a world's characters are drawn: the current world's, and the other world's when
    /// it is drawn.
    pub fn pack_drawn(&self, pack: u32) -> bool {
        self.world.pack == pack || (self.other_drawn() && self.other.as_ref().is_some_and(|o| o.pack == pack))
    }

    /// The pack a pending door job wants loaded.
    pub fn job_pack(&self) -> Option<u32> {
        match self.job? {
            Job::FromExterior { pack: p, .. } | Job::Replace(p, _) | Job::ReloadTransition(p) | Job::EnterInterior(p) => Some(p),
        }
    }

    /// Finish a door job with its loaded world (SPEC-0006 Interior entry steps, exterior
    /// replacement and the post-load rule). Fades are not drawn.
    pub fn finish_job(&mut self, world: World) {
        let Some(job) = self.job.take() else { return };
        self.kinds.insert(world.pack, world.kind);
        self.state.reload_module(world.pack);
        match job {
            Job::FromExterior { plane, reverse, .. } if world.kind == Kind::Transition => {
                // E current, D a transition: it takes T and the hero lands on its door.
                self.other = Some(world);
                let landed = self.land_in_other(plane, reverse, false);
                self.last_door = format!("landed in transition {} at its door {plane}: {landed}", self.world.pack);
                self.activated();
            }
            Job::FromExterior { .. } | Job::EnterInterior(_) => {
                // T is released and its pack remembered; E stays resident as the other world.
                if self.other.as_ref().is_some_and(|o| o.buffer == Buffer::Transition) {
                    self.interior_t = self.other.take().map(|o| o.pack);
                }
                let exterior = std::mem::replace(&mut self.world, world);
                self.other = Some(exterior);
                self.run_init();
                let (l, g) = self.world.entries.first().copied().unwrap_or(([0.0; 3], [1.0, 0.0, 0.0]));
                let l = self.world.snap(l, 6.0);
                self.arrive((l, g));
                self.last_door = format!("entered interior {} at entry record 0", self.world.pack);
                self.activated();
            }
            Job::Replace(_, plane) => {
                self.world = world;
                self.other = None;
                self.run_init();
                let landing = self.world.landing(plane, false, false).or_else(|| {
                    let (l, g) = self.world.entries.first().copied()?;
                    Some((self.world.snap(l, 8.0), g))
                });
                self.arrive(landing.unwrap_or(([0.0; 3], [1.0, 0.0, 0.0])));
                self.last_door = format!("replaced the exterior with {}, arriving at door {plane}", self.world.pack);
                self.activated();
            }
            Job::ReloadTransition(_) => {
                if self.world.buffer == Buffer::Exterior {
                    self.other = Some(world);
                }
            }
        }
    }

    /// SPEC-0028 Trigger pass for the selected hero's body (the grid cell lists are not
    /// modelled: every firing volume of the current world is tested), then the script step: the
    /// registered routine or the lock chest of each fired index runs, then the records decay.
    fn trigger_step(&mut self) {
        let frame = self.ticks as i64;
        let p = self.loco.position();
        let mut fired: Vec<u16> = self.world.volumes.iter().filter(|v| v.touches(p)).map(|v| v.index).collect();
        fired.sort_unstable();
        fired.dedup();
        for &i in &fired {
            self.world.records.entry(i).or_default().fire(frame, true, -1);
        }
        // The callbacks read the pad's newly-pressed word themselves; an open box keeps it.
        let square = self.square_now && !self.dialogue_open;
        for i in fired {
            let rec = self.world.records.get(&i).copied().unwrap_or_default();
            if let Some(g) = self.world.groups.iter().position(|g| g.kind == 1 && g.triggers.first() == Some(&(i as i32))) {
                self.chest_step(g, rec, square);
                continue;
            }
            if let Some(g) = self.world.groups.iter().position(|g| g.kind == 6 && g.triggers.first() == Some(&(i as i32))) {
                self.dig_step(g, rec);
                continue;
            }
            // SPEC-0048 Reaction object (kind 5): on a fire of trigger 0 while no run is going on,
            // object 0 hidden, object 1 shown, played once and waited for, set to time 0 and
            // hidden, then object 0 shown and looped again.
            if let Some(g) = self.world.groups.iter().find(|g| g.kind == 5 && g.triggers.first() == Some(&(i as i32))) {
                let objs: Vec<usize> = g.objects.iter().filter_map(|&o| usize::try_from(o).ok()).collect();
                if let (1, [a, b, ..]) = (rec.state, objs.as_slice()) {
                    if self.world.object_anims.get(*b).is_some_and(|o| !o.drawn) {
                        let (a, b) = (*a, *b);
                        self.apply(&[
                            Action::DeactivateObj { obj: a },
                            Action::InitObjOn { obj: b, end: false },
                            Action::PlayAnim { obj: b, speed: 1.0, wait: true },
                            Action::InitObjOn { obj: b, end: false },
                            Action::DeactivateObj { obj: b },
                            Action::InitObjOn { obj: a, end: false },
                            Action::LoopAnim { obj: a, speed: 1.0 },
                        ]);
                    }
                }
                continue;
            }
            if let Some(&gi) = self.world.group_routes.get(&i) {
                let Some(g) = self.world.groups.get(gi) else { continue };
                let objects: Vec<usize> = g.objects.iter().filter_map(|&o| usize::try_from(o).ok()).collect();
                let call = level::TriggerCall { index: i, state: rec.state, object: rec.object, square };
                let out = level::group_run(self.world.pack, g.kind, call, &objects);
                self.apply(&out);
                continue;
            }
            let Some(&routine) = self.world.routines.get(&i) else { continue };
            let call = level::TriggerCall { index: i, state: rec.state, object: rec.object, square };
            self.state.talking = self.dialogue_open;
            let out = level::on_trigger(self.world.pack, routine, call, &mut self.state);
            if out.prompt.is_some() && self.door_prompt.is_none() && !self.talk_prompt {
                self.door_prompt = out.prompt;
            }
            self.apply(&out.actions);
        }
        for r in self.world.records.values_mut() {
            r.decay(frame);
        }
    }

    /// SPEC-0048 Lock chests, the press rule for script group g (kind 1): the gate (the chest
    /// object drawn, not animating and not finished), the Open prompt, the press, the lock, and
    /// the open sequence as a script thread (camera 0, the hero to locator 0, the chest's open
    /// clip, effect 790 at B facing d, cue 142, hero animation 162 (not played: PlayHeroAnim is
    /// not built), the reward, a 1 s wait, camera 1, a 1.5 s wait, the normal camera).
    fn chest_step(&mut self, g: usize, rec: neopets_sim::trigger::Record, square: bool) {
        let Some(group) = self.world.groups.get(g).cloned() else { return };
        let obj = group.objects.first().and_then(|&o| usize::try_from(o).ok()).filter(|&o| o < self.world.object_anims.len());
        let ready = match obj.map(|o| self.world.object_anims[o]) {
            Some(a) => a.drawn && !a.animating() && !matches!(a.state, 4 | 6 | 8),
            None => !self.world.opened.contains(&g) && !self.state.collected.contains(&group.key),
        };
        if rec.state == 0 || !rec.object || !ready {
            return;
        }
        // The Square caption's shown count must be zero.
        if self.door_prompt.is_some() || self.talk_prompt {
            return;
        }
        self.door_prompt = Some(6);
        if !square {
            return;
        }
        match level::chest_press(&self.world.locks, g as i32, &mut self.state) {
            level::ChestPress::Locked { line } => self.apply(&[Action::Message { line }]),
            level::ChestPress::Open => {
                self.world.opened.insert(g);
                let locator = group.locators.first().and_then(|&k| usize::try_from(k).ok());
                let camera = |k: usize| group.cameras.get(k).and_then(|&c| usize::try_from(c).ok()).map(|track| Action::PlayCamera { track, wait: false, end: 1, speed: 1.0 });
                let mut v: Vec<Action> = vec![Action::HoldHero];
                v.extend(camera(0));
                v.extend(locator.map(|locator| Action::MoveHero { locator }));
                if let Some(obj) = obj {
                    v.extend([Action::InitObjOn { obj, end: false }, Action::PlayAnim { obj, speed: 1.0, wait: false }]);
                }
                // Effect 790 at B = X + 2.9 d, facing d, and cue 142 on the main slot.
                let (x, d) = locator.and_then(|k| self.world.entries.get(k).copied()).unwrap_or((self.loco.position(), self.loco.facing));
                let b = [x[0] + 2.9 * d[0], x[1] + 2.9 * d[1], x[2] + 2.9 * d[2]];
                let f = unit([d[0], 0.0, d[2]]);
                let table = self.effect_table.clone();
                self.effects.one_shot(&table, 790, b, [[f[2], 0.0, -f[0]], [0.0, 1.0, 0.0], f], &mut self.rng);
                self.cue(142);
                v.extend([Action::HeroAnim { anim: 162, wait: true }, Action::ChestReward { group: g as u16 }, Action::Sleep { ticks: 60 }]);
                v.extend(camera(1));
                v.extend([Action::Sleep { ticks: 90 }, Action::WaitDialog, Action::ReturnCamera { preset: -1, release: true }]);
                self.apply(&v);
            }
        }
    }

    /// SPEC-0051 section 10, when a dig starts: a repeat fire (state 2) with a nonzero object
    /// value, object 0 active, the treasure not collected and its map held; then the dig
    /// sequence as a script thread. Not built: SetHeroEquippedState 0 and the HUD reset (step 2)
    /// and the camera shake (step 6).
    fn dig_step(&mut self, g: usize, rec: neopets_sim::trigger::Record) {
        let Some(group) = self.world.groups.get(g).cloned() else { return };
        let objs: Vec<usize> = group.objects.iter().filter_map(|&o| usize::try_from(o).ok()).filter(|&o| o < self.world.object_anims.len()).collect();
        let [o0, o1, o2, ..] = objs.as_slice() else { return };
        let (o0, o1, o2) = (*o0, *o1, *o2);
        let a = self.world.object_anims[o0];
        let held = group.key.checked_sub(200).is_some_and(|i| self.state.maps.contains(&i));
        // A started dig is marked for this load (clean side: the sequence is not started twice
        // while the hero stays in the sphere).
        if rec.state != 2 || !rec.object || !a.drawn || a.state == 4 || self.state.collected.contains(&group.key) || !held || self.world.opened.contains(&g) {
            return;
        }
        self.world.opened.insert(g);
        let Some((p, d)) = group.locators.first().and_then(|&k| usize::try_from(k).ok()).and_then(|k| self.world.entries.get(k).copied()) else { return };
        // Step 1: the hero at P - 2.2 D, facing D.
        self.loco.teleport([p[0] - 2.2 * d[0], p[1] - 2.2 * d[1], p[2] - 2.2 * d[2]], d);
        // Steps 3 to 5: camera [0], the marker off, effect 4036 at P + 2.9 D with no rotation.
        let camera = |k: usize| group.cameras.get(k).and_then(|&c| usize::try_from(c).ok()).map(|track| Action::PlayCamera { track, wait: false, end: 0, speed: 1.0 });
        let table = self.effect_table.clone();
        self.effects.one_shot(&table, 4036, [p[0] + 2.9 * d[0], p[1] + 2.9 * d[1], p[2] + 2.9 * d[2]], neopets_sim::effect::IDENTITY, &mut self.rng);
        let mut v: Vec<Action> = vec![Action::HoldHero];
        v.extend(camera(0));
        v.extend(group.effects.first().and_then(|&e| usize::try_from(e).ok()).map(|slot| Action::SwitchEffect { slot, on: false }));
        // Step 7: object 0 played once and waited for, deactivated; objects 2 and 1 at time 0;
        // object 1 played once and waited for.
        v.extend([
            Action::PlayAnim { obj: o0, speed: 1.0, wait: true },
            Action::DeactivateObj { obj: o0 },
            Action::InitObjOn { obj: o2, end: false },
            Action::InitObjOn { obj: o1, end: false },
            Action::PlayAnim { obj: o1, speed: 1.0, wait: true },
            Action::Sleep { ticks: 1 },
            Action::DigStep { group: g as u16, step: 8 },
            Action::Sleep { ticks: 60 },
        ]);
        v.extend(camera(1));
        v.extend([
            Action::HeroAnim { anim: 186, wait: false },
            Action::DigStep { group: g as u16, step: 11 },
            Action::Sleep { ticks: 120 },
            Action::ReturnCamera { preset: -1, release: true },
            Action::DigStep { group: g as u16, step: 13 },
            Action::Sleep { ticks: 1 },
        ]);
        self.last_door = format!("dig started at group {g} (key {})", group.key);
        self.apply(&v);
    }

    /// The dig's thread steps 8, 11 and 13 (SPEC-0051 section 10).
    fn dig_thread_step(&mut self, g: usize, step: u8) {
        let Some(group) = self.world.groups.get(g).cloned() else { return };
        let Some((p, d)) = group.locators.first().and_then(|&k| usize::try_from(k).ok()).and_then(|k| self.world.entries.get(k).copied()) else { return };
        match step {
            8 => {
                let f = unit([d[0], 0.0, d[2]]);
                let table = self.effect_table.clone();
                self.effects.one_shot(&table, 790, [p[0] + 2.9 * d[0], p[1] + 2.9 * d[1], p[2] + 2.9 * d[2]], [[f[2], 0.0, -f[0]], [0.0, 1.0, 0.0], f], &mut self.rng);
            }
            11 => {
                // The drop action on reward [0] at P + 2.2 D with direction -D (each coin 0.4 s
                // after the one before); reward [1]'s character activation is not used on disc.
                let point = group
                    .drops
                    .first()
                    .and_then(|&id| usize::try_from(id).ok())
                    .and_then(|id| self.world.placements.get(id))
                    .filter(|(region, _)| *region == 0xFF)
                    .and_then(|&(_, ord)| self.world.item_points.get(ord as usize).copied());
                let pack = self.world.pack;
                let at = [p[0] + 2.2 * d[0], p[1] + 2.2 * d[1], p[2] + 2.2 * d[2]];
                self.dig_drop = Some([-d[0], -d[1], -d[2]]);
                self.pay_out(pack, at, point);
                self.dig_drop = None;
                self.last_door = format!("dig reward of group {g} (key {}): {:?} at {at:?}", group.key, point.map(|q| (q.key, q.group, q.count)));
            }
            13 => {
                self.state.collected.insert(group.key);
                self.last_door = format!("dig of group {g} done: entry {} set, maps held {:?}", group.key, self.state.maps);
            }
            _ => {}
        }
    }

    /// Mark map n held (SPEC-0021 Treasure maps): cue 159 (the stinger request is dropped), and
    /// a dig site of the current world with key 200 + n turns its marker on at once.
    pub fn map_held(&mut self, n: u32) {
        if !self.state.maps.insert(n) {
            return;
        }
        self.cue(159);
        // The site with key 200 + n, current world first (the switch setter's search).
        let site = [&self.world].into_iter().chain(self.other.as_ref()).find_map(|w| {
            let g = w.groups.iter().find(|g| g.kind == 6 && g.key == 200 + n)?;
            Some((w.pack, usize::try_from(*g.effects.first()?).ok()?))
        });
        if let Some((pack, slot)) = site {
            self.switch_placed(pack, slot, true);
        }
    }

    /// SPEC-0048 Lock chests step 6: the drop action on reward [0] (a placement id through the
    /// placement id table; the embedded region only) at A = X + 2.4 d + 0.4 up, and the
    /// collected entry (step 7's mark, set here with the reward).
    fn chest_reward(&mut self, g: usize) {
        let Some(group) = self.world.groups.get(g).cloned() else { return };
        let locator = group.locators.first().and_then(|&k| usize::try_from(k).ok());
        let (x, d) = locator.and_then(|k| self.world.entries.get(k).copied()).unwrap_or((self.loco.position(), self.loco.facing));
        let a = [x[0] + 2.4 * d[0], x[1] + 2.4 * d[1] + 0.4, x[2] + 2.4 * d[2]];
        let point = group
            .drops
            .first()
            .and_then(|&id| usize::try_from(id).ok())
            .and_then(|id| self.world.placements.get(id))
            .filter(|(region, _)| *region == 0xFF)
            .and_then(|&(_, ord)| self.world.item_points.get(ord as usize).copied());
        let pack = self.world.pack;
        self.pay_out(pack, a, point);
        self.state.collected.insert(group.key);
        self.last_door = format!("chest group {g} opened (key {}), reward {:?}", group.key, point.map(|p| p.key));
    }

    /// SPEC-0006 door step after the hero's motion (its trigger step): a touched door of the
    /// current world posts its prompt until confirmed, then travels by the default rule.
    fn door_step(&mut self) {
        if self.job.is_some() {
            return;
        }
        let p = self.loco.position();
        let Some(n) = self.world.touched_door(p) else { return };
        if self.world.scripted_doors {
            // SPEC-0006 Doors, item 5: the registered callback decides. Context 0 starts on
            // contact; otherwise the door posts the context as its prompt and the callback starts
            // on a Square press edge with the confirm flag set, which it never clears.
            match level::teleport_context(self.world.pack, n) {
                None => self.last_door = format!("door {n}: script door of {} not described yet", self.world.pack),
                Some(0) => {
                    let actions = level::on_teleport(self.world.pack, n, &mut self.state);
                    self.apply(&actions);
                }
                Some(c) => {
                    self.door_prompt = Some(c);
                    if self.square_now && self.confirm {
                        let actions = level::on_teleport(self.world.pack, n, &mut self.state);
                        self.apply(&actions);
                    }
                }
            }
            return;
        }
        if self.world.ground.as_ref().is_none_or(|g| g.cell_value(p[0], p[2]) < 0) {
            return;
        }
        if self.confirm {
            self.confirm = false;
            self.default_door(n);
        } else {
            self.door_prompt = Some(if self.world.buffer == Buffer::Exterior { 13 } else { 26 });
        }
    }

    /// One game tick in the hero order of SPEC-0040 Behavior 3: the camera, the hero update
    /// (stick, turn, tiers, jumps and air phases, the pending step), the motion-record tick
    /// (ground step with the world swap search, root travel, constraint, gravity, contacts,
    /// integrator), then the animation pass, which stores the next root step.
    pub fn tick(&mut self, input: &HeroInput, pad: CameraPad) {
        self.ticks += 1;
        self.equipment_step();
        // SPEC-0053: while a swap runs (game-state code 5) the hero takes no input.
        let swapping = self.swap_step(input);
        let held_input;
        let input = if swapping {
            held_input = HeroInput::default();
            &held_input
        } else {
            input
        };
        // SPEC-0035 section 15.2: each drawn pass (quick panels closed) lowers the caption
        // lines' counters.
        if !self.quick.open {
            for l in &mut self.price_lines {
                l.1 = l.1.saturating_sub(1);
            }
        }
        // SPEC-0052 Screen effect: the bars' and the fade's per-pass update.
        self.bars.update();
        self.fade.update();
        // SPEC-0055: the game-state pass runs the timers before the world update; while the
        // game-over page is up the world is frozen and only the page's exit test runs.
        if self.death.page.is_some() {
            if self.death.page_pass(input.jump) {
                // SPEC-0025 Session snapshot: with a save or load in the session, Continue
                // applies the snapshot; otherwise the placeholder for the title flow.
                if self.snapshot.is_some() {
                    self.reload_wanted = true;
                } else {
                    self.continue_placeholder();
                }
            }
            return;
        }
        if self.death.flag_e {
            self.death.hand_over();
            return;
        }
        let pass = self.death.pass();
        if let Some(i) = pass.respawn {
            // Timer F: the start pose becomes entry record i; the respawn at the start pose,
            // in state 60 for death mode 2.
            let pose = self.world.entries.get(i as usize).copied().unwrap_or((self.loco.position(), self.loco.facing));
            self.arrive(pose);
            if self.death.mode == 2 {
                self.loco.set_end_state(hero_states::RESPAWN);
                self.penalty_due = true;
            }
            self.death.mode = 0;
        }
        if std::mem::take(&mut self.penalty_due) {
            // State 60's first tick: the plain deduct of min(5, health - 1).
            self.hero_hp -= neopets_sim::death::respawn_penalty(self.hero_hp);
        }
        // SPEC-0027 Update order: the camera runs first, from the hero's previous pose.
        let w = &self.world;
        self.camera.update(
            &followed(&self.loco, w),
            pad,
            |x, z, c| w.probe(x, z, c).height,
            |e| push_eye(w, e),
            |a, c| w.ground.as_ref().is_none_or(|g| g.segment_clear(a, c, true)),
        );
        // SPEC-0027 Scripted cameras: the play is sampled and its end mode acts.
        if let Some(cut) = self.cut.as_mut() {
            if let neopets_sim::scene::PlayUpdate::Ended { idle, release } = cut.update(&self.world.tracks) {
                if release {
                    self.held = false;
                }
                if idle {
                    self.cut = None;
                }
            }
        }
        let (forward, left) = self.camera.rows();
        // The interaction chooser shows the prompt posted on the previous tick and sets the
        // confirm flag on a Square press edge.
        // An open dialogue box takes Square to close it, and holds the hero in place.
        let mut input = *input;
        if self.dialogue_open {
            // SPEC-0022: the box takes Cross (held, faster reveal) and Square (skip, Next,
            // Finish); the hero stays in place meanwhile.
            // SPEC-0022 Choices: Circle (yes) or Triangle (no) answers a question box.
            if self.textbox.asking() && self.textbox.choose(input.attack, std::mem::take(&mut self.panel_latch.triangle)).is_some() {
                self.cue(0x97);
                self.dialogue_open = false;
                self.message = None;
                self.talk_with = None;
            }
            match self.textbox.update(input.cross_held, input.square) {
                // SPEC-0022: an advance plays 0x98; the box host plays 0x97 when the box closes.
                Some(neopets_sim::textbox::BoxEvent::Closed) => {
                    self.cue(0x97);
                    self.dialogue_open = false;
                    self.message = None;
                    self.talk_with = None;
                }
                Some(neopets_sim::textbox::BoxEvent::Advanced) => self.cue(0x98),
                None => {}
            }
            self.emote_requests();
            self.emote_consumers();
            input = HeroInput::default();
            // SPEC-0056 Talk facing: in the conversation the hero turns toward its partner
            // with weight 0.1 and the partner toward the hero with weight 0.3.
            if let Some(c) = self.talk_with.and_then(|id| self.characters.iter_mut().find(|c| c.id == id)) {
                let h = self.loco.position();
                let to_hero = [h[0] - c.position[0], 0.0, h[2] - c.position[2]];
                c.facing = neopets_sim::hero::steer(c.facing, to_hero, 0.3);
                self.loco.steer_toward(to_hero.map(|v| -v), 0.1);
            }
        }
        // SPEC-0052 Control locks: the script hold, a running camera play and a moving fade
        // take the hero's input; a scripted walk (SPEC-0033 4A.4 and 4A.5) steers it instead.
        // The script button query (IsButtonPressed) reads the pad, not the hero's input.
        self.button_square = input.square;
        if self.held || self.cut.is_some_and(|c| c.state == 1) || self.fade.moving() {
            input = HeroInput::default();
        }
        if let Some((t, face, last)) = self.hero_walk {
            let p = self.loco.position();
            let (dx, dz) = (t[0] - p[0], t[2] - p[2]);
            let d = dx.hypot(dz);
            // Arrival within 0.25 (or once the distance grows inside 1.0: the stick walk
            // cannot always land within 0.25).
            if d < 0.25 || (d < 1.0 && d > last) {
                match face {
                    Some(g) if self.loco.facing[0] * g[0] + self.loco.facing[2] * g[2] < 0.99 => {
                        self.loco.steer_toward([g[0], 0.0, g[2]], 0.1);
                        self.hero_walk = Some((t, face, 0.0));
                    }
                    _ => self.hero_walk = None,
                }
            } else {
                let (f, l) = (forward, left);
                let fl = (f[0] * f[0] + f[2] * f[2]).sqrt().max(1e-6);
                input.forward = 0.6 * (dx * f[0] + dz * f[2]) / d / fl;
                input.side = 0.6 * (dx * l[0] + dz * l[2]) / d;
                self.hero_walk = Some((t, face, d));
            }
        }
        if self.prompt_shown.is_some() && input.square {
            self.confirm = true;
        }
        self.square_now = input.square;
        self.save_point_step(input.square);
        self.music_step();
        // The attack press edge enters state 13 from the ground states (SPEC-0036); the step
        // then runs every tick and decides the clip.
        // SPEC-0021 Quick-item panel: it opens on an Up edge (not in action state 0x19) and,
        // while open, takes Cross and Circle; Triangle with it closed uses the marker.
        let mut panel = std::mem::take(&mut self.panel_latch);
        if self.quick.open {
            panel.cross |= input.jump;
            panel.circle |= input.attack;
        }
        let inv = &self.state.items;
        let owns = |id: i32| id >= 0 && inv.get(&(id as u16)).is_some_and(|&n| n > 0);
        let was_open = self.quick.open;
        let marker = self.quick.marker;
        let out = self.quick.tick(panel, self.loco.state != 0x19, &self.quick_rows, &owns);
        // SPEC-0023 section 4: selecting or using a quick item re-arms G0, G1 and C; L3 re-arms
        // every group.
        if out.use_item.is_some() || self.quick.marker != marker {
            self.hud.rearm_quick();
        }
        if std::mem::take(&mut self.l3_latch) {
            self.hud.rearm_all();
        }
        if out.consumed || was_open {
            input.jump = false;
            input.attack = false;
        }
        if let Some(id) = out.use_item {
            self.use_item(id as u16);
        }
        if matches!(self.loco.state, hero_states::DEAD | hero_states::HURT | hero_states::LONG_FALL) {
            input = HeroInput { side: input.side, forward: input.forward, ..HeroInput::default() };
        }
        // SPEC-0033 Behavior 52: the stick gesture machine (reset while busy or in a box), in
        // the dead-zoned stick units (87.5 at a full push).
        if neopets_sim::hero::busy_state(self.loco.state) || self.dialogue_open {
            self.gesture.reset();
        } else {
            self.gesture.tick(input.forward * 87.5, input.side * 87.5, &self.gesture_words);
        }
        if self.hero_index == 1 {
            self.roberta_step(&input);
        }
        self.roberta_displays_step();
        if self.hero_index == 0 && self.attack.is_none() && input.attack && (1..=6).contains(&self.loco.state) {
            self.attack = Some(neopets_sim::attack::TorAttack::start_with(self.gesture.mode));
        }
        let mut force = false;
        if let Some(a) = self.attack.as_mut() {
            let c = self.hero.clip_state();
            match a.tick(input.attack_held, input.attack, neopets_sim::attack::Clip { action: c.action, progress: c.progress }) {
                Some(r) => {
                    self.loco.set_attack(Some(r.action));
                    force = r.restart;
                }
                None => {
                    self.attack = None;
                    self.loco.set_attack(None);
                }
            }
        }
        self.power_ready_step();
        if let Some(sub) = self.attack.as_ref().filter(|a| a.cleared).map(|a| a.sub) {
            if sub == 10 {
                self.power_release();
            }
            self.hit_list.clear();
            if let Some(a) = self.attack.as_mut() {
                a.cleared = false;
            }
            // SPEC-0036 Attack router, Tor: into 2 and from 2 into 3, weapon 3 and voice 5; into
            // 4, weapon 4 and voice 6; entering 6 and each repeat, weapon 9 and voice 4; into 10,
            // voice 6, then voice 8.
            use neopets_sim::sound::{Play as Cue, VOICE, WEAPON};
            let plays: &[(usize, i32)] = match sub {
                2 | 3 => &[(WEAPON, 3), (VOICE, 5)],
                4 => &[(WEAPON, 4), (VOICE, 6)],
                6 => &[(WEAPON, 9), (VOICE, 4)],
                7 | 8 => &[(WEAPON, 7)],
                9 => &[(WEAPON, 8)],
                10 => &[(VOICE, 6), (VOICE, 8)],
                _ => &[],
            };
            for &(slot, cue) in plays {
                self.sound.play(slot, cue, Cue::default());
            }
        }
        let hits = self.hit_list.len();
        self.hero_swing();
        self.mote_objects_step();
        self.shots_step();
        // A swing hit: power cue 2 in sub-state 10, else weapon cue 14 (no material impact).
        if self.hit_list.len() > hits {
            let power = self.attack.as_ref().is_some_and(|a| a.sub == 10);
            let (slot, cue) = if power { (neopets_sim::sound::POWER, 2) } else { (neopets_sim::sound::WEAPON, 14) };
            self.sound.play(slot, cue, neopets_sim::sound::Play::default());
        }
        self.loco.shield = self.shield.is_some();
        self.status_step();
        self.loco.update(&input, forward, left, self.hero.clip_state());
        self.pending_step();
        // While a conversation runs the emote consumer owns the hero's main id.
        if !self.dialogue_open {
            self.hero.request(self.loco.action(), force);
        }
        self.hero.set_rate(self.loco.rate);
        let (world, other) = (&self.world, self.other.as_ref());
        let search_other = buffers::may_swap(world.buffer, self.boot.neighbour_ok(world.pack, world.kind, world.pack));
        let swap = Cell::new(false);
        self.loco.record_tick(
            self.root_step,
            |x, z, c| {
                let hit = world.probe(x, z, c);
                if !hit.found && search_other {
                    if let Some(o) = other {
                        let there = o.probe(x, z, c);
                        if there.found {
                            swap.set(true);
                            return (there.height, there.normal);
                        }
                    }
                }
                (hit.height, hit.normal)
            },
            |p| {
                // SPEC-0040 Behavior 14A driver branch 0, in order: the cell's frame records
                // (walls), then the box primitives, each with the hero radius. Not implemented:
                // cell lists A and B as such, cylinders, spheres, the ledge tests, the shared
                // contact response (pushes move the position only) and recovery.
                if let Some(g) = &world.ground {
                    let walls: Vec<WallEdge> = g
                        .frames_at(p[0], p[2])
                        .into_iter()
                        .map(|f| WallEdge { face: f.face, run: f.run, top: f.top, length: f.length, height: f.height })
                        .collect();
                    contact::resolve_walls(p, &walls, contact::HERO_RADIUS);
                }
                contact::resolve(p, &world.boxes, contact::HERO_RADIUS);
            },
        );
        // SPEC-0055 8A: the deadly-ground kill (kind 2, gap 0.085 or less; no health change),
        // then the death-state handler of state 16.
        let hp = {
            let p = self.loco.position();
            self.world.probe(p[0], p[2], p[1] + neopets_sim::motion::PROBE_CEILING)
        };
        let deadly = (hp.found && hp.kind == Some(2))
            .then(|| neopets_sim::death::Deadly { respawn: hp.respawn.unwrap_or(0xFF), surface: hp.surface.unwrap_or(0) });
        if neopets_sim::death::deadly_kill(deadly, self.loco.body.gap) {
            // The death effect of the surface (0x81 hot burst, 0xBC water splash; SPEC-0055 8A,
            // SPEC-0034 section 37) when the hero enters the death state.
            if self.loco.state != hero_states::DEAD {
                if let Some(e) = neopets_sim::death::death_clip(deadly).1 {
                    let table = self.effect_table.clone();
                    self.effects.one_shot(&table, e as i32, self.loco.position(), neopets_sim::effect::IDENTITY, &mut self.rng);
                }
            }
            self.default_death();
        } else if self.loco.state != hero_states::DEAD
            && neopets_sim::death::hurting_tick(hp.found.then_some(hp.surface).flatten(), deadly.is_some(), self.loco.body.gap, self.ticks as f32 / 60.0, 1.0 / 60.0)
        {
            // SPEC-0057 7A rule 2: a normal hit of 10 with no knockback (overlay rule 3 is not
            // built: the overlay entry's point test is not read).
            let f = self.loco.facing;
            self.hero_hit(neopets_sim::death::HURT_DAMAGE, 0.0, false, f, "a hurting surface".into());
        }
        if self.loco.state == hero_states::DEAD {
            // SPEC-0053 3.8 d: with the death clip at 0.95, a switch to the partner (both flags
            // and the extra flag set) is requested every tick; a running or accepted switch
            // keeps the game going, a refusal or a dead partner leads to game over.
            let clip = self.hero.clip_state();
            let at_end = deadly.is_none() && clip.progress >= 0.95;
            let takes_over = at_end && (self.swap.is_some() || self.request_switch(true, true, true));
            let id = self.death.handler(deadly, clip, takes_over);
            self.loco.set_death_clip(id);
        }
        // World swap: a hit in the other buffer makes it current (the original waits for the
        // next frame; here it takes effect before the next tick).
        if swap.get() {
            if let Some(o) = self.other.take() {
                self.other = Some(std::mem::replace(&mut self.world, o));
                self.swaps += 1;
                self.activated();
            }
        }
        // The interaction chooser: a talk candidate claims Square before the doors.
        self.talk_prompt = false;
        if !self.dialogue_open && !self.buy_step() && !self.talk_step() && !self.catch_step() {
            self.door_step();
            // The chooser's last fallback: Square with no target enters the sneak.
            if self.square_now && self.door_prompt.is_none() && self.prompt_shown.is_none() {
                self.loco.sneak();
            }
        }
        if self.job.is_none() {
            self.state.hero.swap_mode = self.swap_mode;
            self.state.hero.selected = self.hero_index;
            let parked_dead = self.parked.as_ref().is_some_and(|p| p.hp <= 0);
            let live_dead = self.loco.state == hero_states::DEAD;
            self.state.hero.dead = if self.hero_index == 0 { [live_dead, parked_dead] } else { [parked_dead, live_dead] };
            self.state.hero.swapping = self.swap.is_some();
            self.trigger_step();
            let pack = self.world.pack;
            self.state.entity_hp = self.characters.iter().filter(|c| c.pack == pack).filter_map(|c| Some((c.entity?, c.hp.max(0)))).collect();
            self.state.hero.sub = self.attack.as_ref().map_or(0, |a| a.sub);
            self.state.hero.occupied = neopets_sim::hero::busy_state(self.loco.state) && self.loco.state != hero_states::ATTACK || self.held;
            self.state.hero.attack_mote = self.equipment.motes[0][2];
            self.state.hero.shield_mote = self.equipment.motes[0][1];
            self.state.hero.tick = self.ticks;
            let actions = level::on_update(self.world.pack, &mut self.state, self.dialogue_open);
            self.state.hero.hits.clear();
            self.apply(&actions);
        }
        self.run_threads();
        // SPEC-0045 1217: the record thread steps the object records every pass.
        if self.world.pack == 1217 {
            if let Some(mut b) = self.state.burrows.take() {
                let anims = &self.world.object_anims;
                let actions = b.step(&|o| anims.get(o).is_some_and(|a| a.animating()), self.ticks);
                self.state.burrows = Some(b);
                for a in actions {
                    self.execute(a, 1217);
                }
            }
        }
        // The wheel's spin (SPEC-0059 section 2): the stop prompt (context 4) at full rate and
        // the Square press; the spin ends after the tick the press was seen.
        if let Some((pack, mut spin)) = self.wheel_spin.take() {
            let mut w = level::wheel_of(pack, &self.state);
            if spin.stopped {
                level::set_wheel(pack, &mut self.state, &w);
            } else {
                if spin.tick(&mut w, self.button_square) {
                    self.door_prompt = Some(4);
                }
                level::set_wheel(pack, &mut self.state, &w);
                self.wheel_spin = Some((pack, spin));
            }
        }
        self.prompt_shown = self.door_prompt.take();
        self.root_step = self.hero.tick();
        self.sound_step();
        self.clip_effects_step();
        // SPEC-0028 7.1: the resident worlds' animated objects step before the hero update.
        for w in [&mut self.world].into_iter().chain(self.other.as_mut()) {
            for (o, m) in w.object_anims.iter_mut().zip(&w.motions) {
                o.tick(m);
            }
        }
        self.status_display_step();
        self.zone_step();
        self.foliage_step();
        self.placed_release_step();
        self.placed_zone_step();
        let table = self.effect_table.clone();
        self.effects.tick(&table, &mut self.rng);
        self.light_step();
        // IsAI_Dead (SPEC-0030 row 45): a bound character's dead test, else the kill memory of
        // this load for its entity id.
        for c in self.characters.iter().filter(|c| c.dying.is_some() || c.enemy.as_ref().is_some_and(|e| e.dead)) {
            if let Some(e) = c.entity {
                self.killed.insert((c.pack, e));
            }
        }
        let pack = self.world.pack;
        self.state.ai_dead = self.killed.iter().filter(|(p, _)| *p == pack).map(|&(_, e)| e).collect();
        self.state.ai_dead.retain(|&e| !self.characters.iter().any(|c| c.pack == pack && c.entity == Some(e) && c.dying.is_none() && !c.enemy.as_ref().is_some_and(|x| x.dead) && c.hp > 0));
        // Dead characters disappear when their countdown ends.
        for c in &mut self.characters {
            if let Some(t) = c.dying.as_mut() {
                *t = t.saturating_sub(1);
            }
        }
        self.characters.retain(|c| c.dying != Some(0) && !c.enemy.as_ref().is_some_and(|e| e.gone));
        // Characters and slot choices of worlds that are no longer resident are released.
        let resident: Vec<u32> = std::iter::once(self.world.pack).chain(self.other.as_ref().map(|o| o.pack)).collect();
        let pet = self.pet_char;
        self.characters.retain(|c| resident.contains(&c.pack) || Some(c.id) == pet);
        self.pending_off.retain(|(p, _)| resident.contains(p));
        self.pools.retain(|pack, _| resident.contains(pack));
        self.slot_regions.retain(|(pack, _), _| resident.contains(pack));
        let hero = self.loco.position();
        let talking = if self.dialogue_open { self.talk_with } else { None };
        // Whinnies: routine, group count after each member (SPEC-0049), motion-record tick.
        let world = &mut self.world;
        let (mut members, mut penned_now) = (0, 0);
        for c in self.characters.iter().filter(|c| c.pack == world.pack && c.whinny.is_some()) {
            members += 1;
            penned_now += c.whinny.as_ref().is_some_and(|w| w.penned()) as usize;
        }
        for c in self.characters.iter_mut().filter(|c| c.pack == world.pack && !c.held && !c.inactive) {
            let (Some(wk), Some(wh)) = (c.walker.as_mut(), c.whinny.as_mut()) else { continue };
            let was = wh.penned();
            let id = wh.tick(wk, &world.navmeshes, &mut world.pens, hero, &mut self.rng);
            penned_now = penned_now + wh.penned() as usize - was as usize;
            neopets_sim::whinny::group_count(&mut self.state, members, penned_now);
            c.anim.play(id);
            let ground = &world.ground;
            wk.motion_tick(c.root_step, &world.navmeshes, |x, z, cl| {
                ground.as_ref().map_or((0.0, UP), |g| g.probe_normal(x, z, cl))
            });
            c.position = wk.position();
            c.facing = wk.facing;
        }
        self.pet_step();
        self.boss_step();
        self.mice_step();
        self.enemies_step();
        self.items_step();
        // SPEC-0023: the in-play HUD steps only while the in-play state runs, so not while a
        // dialogue box is up.
        if !self.dialogue_open {
            let v = neopets_sim::hud::Values {
                hp: self.hero_hp,
                hp_max: self.hero_max,
                mp: self.hero_mp,
                mp_max: self.hero_mp_max,
                poisoned: self.statuses.active(neopets_sim::status::POISON),
            };
            self.hud.tick(v, self.quick.open, &self.hud_words);
            let target = self.monster_bar.target.and_then(|id| self.characters.iter().find(|c| c.id == id));
            let (hp, max) = target.map_or((0, 1), |c| (c.hp, c.row.map_or(1, |r| r.hit_points as i32)));
            self.monster_bar.tick(hp, max, &self.monster_words);
        }
        let world = &self.world;
        for c in &mut self.characters {
            // SPEC-0056 Walk-to-point (classes 23 and 25), simplified: straight toward the
            // target at the townsfolk walk speed (0.08337 a tick, move type 0) on the ground,
            // steering with weight 0.3, walk clip 1; arrival within 1.0, then the turn to the
            // final facing at 0.1 until the dot is 0.99, then idle (0). The navigation mesh, the
            // hold for a near hero and root-motion travel are not used.
            if c.inactive {
                continue;
            }
            if let Some((t, face, stop)) = c.script_walk {
                let (dx, dz) = (t[0] - c.position[0], t[2] - c.position[2]);
                let d = dx.hypot(dz);
                if d >= stop {
                    c.facing = neopets_sim::hero::steer(c.facing, [dx, 0.0, dz], 0.3);
                    let s = 0.083_37 / d;
                    let (x, z) = (c.position[0] + dx * s, c.position[2] + dz * s);
                    let y = if c.pack == world.pack { world.ground_at(x, z, c.position[1] + 2.5) } else { None };
                    c.position = [x, y.unwrap_or(c.position[1]), z];
                    if c.anim.clip_state().action != 1 {
                        c.anim.play(1);
                    }
                } else if let Some(g) = face.filter(|g| c.facing[0] * g[0] + c.facing[2] * g[2] < 0.99) {
                    c.facing = neopets_sim::hero::steer(c.facing, g, 0.1);
                } else {
                    c.script_walk = None;
                    c.anim.play(0);
                }
                if let Some(wk) = c.walker.as_mut() {
                    wk.body.position = c.position;
                    wk.facing = c.facing;
                }
                c.root_step = c.anim.tick();
                continue;
            }
            if c.held {
                c.root_step = c.anim.tick();
                continue;
            }
            // Base state 25 (conversation) freezes the routine.
            if let (Some(t), false) = (c.townsfolk.as_mut(), talking == Some(c.id)) {
                let clip = c.anim.clip_state();
                let view = neopets_sim::npc::View { position: c.position, hero, action: clip.action, progress: clip.progress };
                if let Some(id) = t.tick(&mut c.facing, view) {
                    c.anim.play(id);
                }
            }
            c.root_step = c.anim.tick();
        }
    }

    /// The common enemies of the current world (SPEC-0038): their view of the hero, the think
    /// loop, movement by the walker (navigation and root-step travel), and their strikes on the
    /// hero (SPEC-0036 Enemy attack damage to a hero, Taking a hit, Knockback, Health and death).
    fn enemies_step(&mut self) {
        let hero = self.loco.position();
        let hero_facing = self.loco.facing;
        let hplace = placement(hero, hero_facing);
        let hframes = self.hero.bone_frames();
        let caps: Vec<Cylinder> = self
            .hero
            .bones()
            .hurt
            .iter()
            .zip(&hframes)
            .filter(|(h, _)| h.1 > 0.0)
            .map(|(&(base, length, radius), f)| {
                let w = hplace.mul(f);
                Cylinder::from_base(w.apply(base), unit(w.cols[1]), radius, length)
            })
            .collect();
        let bands = enemy::band_factors(self.loco.state, &self.bands.0, &self.bands.1);
        let guarding = self.loco.state == hero_states::GUARD;
        let attackable = self.loco.state != hero_states::DEAD && !self.dialogue_open;
        let world = &self.world;
        let mut strikes = Vec::new();
        let mut dots = Vec::new();
        for c in self.characters.iter_mut().filter(|c| c.pack == world.pack && !c.held && !c.inactive) {
            let Character { id, enemy: Some(e), walker: Some(wk), anim, row: Some(row), weapons, position, facing, root_step, .. } = c else { continue };
            let p = wk.position();
            if e.dead {
                let clip = anim.clip_state();
                let has = |id: u32| anim.has(id);
                let v = enemy_view(p, wk, hero, hero_facing, bands, attackable, true, (false, false), false, 0.0, clip, &has);
                let act = e.tick(&v, &mut self.rng);
                if let Some(id) = act.anim {
                    anim.request(id, false);
                }
                wk.body.position[1] -= act.sink;
                *position = wk.position();
                continue;
            }
            // The navmesh pick: the first ground mesh holding the enemy (SPEC-0038 Behavior 6).
            if wk.mesh.and_then(|i| world.navmeshes.get(i)).is_none_or(|m| m.locate_full(p).is_none()) {
                let pick = world.navmeshes.iter().position(|m| m.kind == 1 && m.locate_full(p).is_some());
                if pick.is_some() {
                    wk.assign_mesh(pick, &world.navmeshes);
                }
            }
            let mesh = wk.mesh.and_then(|i| world.navmeshes.get(i));
            let (route, on_mesh) = match mesh {
                Some(m) => {
                    let (a, b) = (m.locate_full(p), m.locate_full(hero));
                    (a.zip(b).is_some_and(|(a, b)| a == b || m.route(a, b).is_some()), b.is_some())
                }
                None => (true, true),
            };
            // The sight line between sampled points (half the body heights; R-0015).
            let eye = [p[0], p[1] + 0.5 * row.height, p[2]];
            let at = [hero[0], hero[1] + 0.5 * HERO_HEIGHT, hero[2]];
            let sight = world.ground.as_ref().is_none_or(|g| g.segment_clear(eye, at, true));
            // The strike test: weapon N while event bit 2^(N-1) is set, a cylinder centred on
            // attach frame N-1's anchor along its Z row, against the hero's bone capsules.
            let bits = anim.frame_bits();
            let place = placement(p, wk.facing);
            let frames = anim.bone_frames();
            let contact = (0..5).any(|n| {
                if bits & (1 << n) == 0 {
                    return false;
                }
                let Some(&(Some(bone), angles, offset)) = anim.bones().attach.get(n) else { return false };
                let Some(bw) = frames.get(bone) else { return false };
                let w = place.mul(bw).mul(&attach_rotation(offset, angles));
                let (radius, length) = weapons[n];
                let weapon = Cylinder { center: w.t, axis: unit(w.cols[2]), radius, length };
                caps.iter().any(|cap| neopets_sim::combat::overlap(&weapon, cap))
            });
            let clip = anim.clip_state();
            let has = |id: u32| anim.has(id);
            let speed = (root_step[0].powi(2) + root_step[2].powi(2)).sqrt();
            let mut v = enemy_view(p, wk, hero, hero_facing, bands, attackable, sight, (route, on_mesh), contact, speed, clip, &has);
            v.radii = row.radius + contact::HERO_RADIUS;
            v.spawn_bits = bits;
            // SPEC-0036 Guard timer and per-attacker block: the guard predicate (state 20; the
            // 40-tick threat-sense timer is not implemented) and dot(hero facing, attacker
            // facing) at most -0.5.
            let wf = wk.facing;
            v.hero_blocks = guarding && hero_facing[0] * wf[0] + hero_facing[1] * wf[1] + hero_facing[2] * wf[2] <= -0.5;
            let act = e.tick(&v, &mut self.rng);
            // A missile slot's spawn event: the spawner from the emit point of the event bit
            // (attach frame n; the body middle when the frame has no bone; R-0016).
            if let Some(req) = act.shot {
                let launch = anim
                    .bones()
                    .attach
                    .get(req.emit)
                    .and_then(|&(bone, angles, offset)| Some(place.mul(frames.get(bone?)?).mul(&attach_rotation(offset, angles)).t))
                    .unwrap_or([p[0], p[1] + 0.5 * row.height, p[2]]);
                let cam = &self.camera;
                let fwd = unit(sub_v(cam.focus, cam.eye));
                let to = unit(sub_v(p, cam.eye));
                let out_of_cone = fwd[0] * to[0] + fwd[1] * to[1] + fwd[2] * to[2] < 0.85;
                let slot = &e.p.slots[req.slot];
                if let Some(mut setup) = neopets_sim::projectile::spawn(&slot.missile, e.p.element, *id, launch, hero, out_of_cone, &mut self.rng) {
                    // Behavior 13: the damage times the enemy damage scale (curse, strength).
                    setup.damage = (setup.damage * e.shot_scale()).max(0.0);
                    self.shots.submit(setup);
                }
            }
            if let Some(id) = act.anim {
                anim.request(id, act.force);
            }
            anim.set_rate(act.rate.unwrap_or(1.0));
            match act.go {
                Some(enemy::Go::Toward(t)) => {
                    wk.nav_step(&world.navmeshes, t, hero);
                }
                Some(enemy::Go::Facing(t)) => wk.steer_to(t, 0.3),
                None => {}
            }
            if let Some((t, w)) = act.face {
                wk.steer_to(t, w.clamp(0.0, 1.0));
            }
            let step = if act.stop { [0.0; 3] } else { *root_step };
            let ground = &world.ground;
            wk.motion_tick(step, &world.navmeshes, |x, z, cl| ground.as_ref().map_or((0.0, UP), |g| g.probe_normal(x, z, cl)));
            *position = wk.position();
            *facing = wk.facing;
            if let Some(s) = act.strike {
                let m = e.p.slots[s.slot].missile;
                strikes.push((*id, s, wk.facing, (m.status, m.status_chance, m.status_range), e.strike_scale(), e.status(11)));
            }
            for amount in act.status_hits.into_iter().flatten() {
                dots.push((*id, amount));
            }
        }
        // SPEC-0029 Behavior 27.2 steps 3 and 8: each poison or fire hit is a take-hit.
        for (id, amount) in dots {
            if let Some(i) = self.characters.iter().position(|c| c.id == id) {
                self.character_take_hit(i, amount, hero, "status ");
            }
        }
        // The projectile pool's update, after the region passes (SPEC-0029 Behavior 9).
        let owners: HashMap<u64, ([f32; 3], f32, f32)> =
            self.characters.iter().map(|c| (c.id, (c.position, c.row.map_or(1.0, |r| r.radius), c.row.map_or(3.0, |r| r.height)))).collect();
        let gap = self.loco.body.gap;
        let shield = self.shield.is_some();
        let events = {
            let world = &self.world;
            let owner_exists = |id: u64| owners.contains_key(&id);
            let owner_body = |id: u64| owners.get(&id).copied();
            let ground = |x: f32, z: f32, c: f32| world.ground_at(x, z, c);
            let wall = |a: [f32; 3], b: [f32; 3]| world.ground.as_ref().is_some_and(|g| !g.segment_clear(a, b, true));
            let env = neopets_sim::projectile::Env {
                hero,
                hero_radius: contact::HERO_RADIUS,
                hero_height: HERO_HEIGHT,
                hero_gap: gap,
                hero_facing,
                guard: guarding,
                shield,
                owner_exists: &owner_exists,
                owner_body: &owner_body,
                ground: &ground,
                wall: &wall,
            };
            self.shots.tick(&env)
        };
        for ev in events {
            match ev {
                neopets_sim::projectile::Event::Hero { damage, blocked, element, .. } => {
                    // SPEC-0029 Behavior 9: D = max(0, F(damage - S)) with the shield block, the
                    // armour and the body item of SPEC-0036 (the curse and strength scale was
                    // applied at spawn), then the blocked quarter.
                    let d = self.enemy_damage(damage, blocked, element, 1.0);
                    self.hero_hit(neopets_sim::projectile::taken(d, blocked), 0.0, blocked, hero_facing, "a shot".into());
                }
                neopets_sim::projectile::Event::Reflected { element } => {
                    // The shield Mote's charge for a reflection (no MP; R-0034).
                    if let (Some(shield), Some(m1)) = (self.equipment.entries[1][0], self.equipment.motes[0][1]) {
                        if neopets_sim::mote::element_passes(element, neopets_sim::mote::element(Some(m1))) {
                            let s = if self.item_kind(shield) == 5 { self.items.get(shield).map_or(0.0, |i| i.stats[1] as f32) } else { 0.0 };
                            self.spend_charge(1, m1, neopets_sim::mote::charge_tor(1, s));
                        }
                    }
                }
                neopets_sim::projectile::Event::Owner { id, damage } => {
                    // A reflected shot: the owner's take-hit, then its death hook at 0 HP.
                    if let Some(c) = self.characters.iter_mut().find(|c| c.id == id) {
                        if let Some(e) = c.enemy.as_mut() {
                            let (applied, _) = e.take_hit(damage.trunc() as i32, hero, &mut self.rng);
                            c.hp -= applied;
                            if c.hp <= 0 && !e.dead {
                                c.hp = 0;
                                e.die();
                                self.payouts.push((c.pack, c.position, c.drop));
                            }
                            self.last_hit = format!("reflected shot hit {} for {applied} (hp {})", c.name, c.hp);
                        }
                    }
                }
                neopets_sim::projectile::Event::Status { code, chance, seconds } => {
                    // SPEC-0029 Behavior 9: int[0, 100] at most the chance; no stun while stunned.
                    let r = self.rng.range(0, 100);
                    if r <= chance && !(code == 0 && self.statuses.active(neopets_sim::status::STUN)) {
                        self.statuses.apply(code, seconds);
                    }
                }
                neopets_sim::projectile::Event::Summon { .. } => {}
                neopets_sim::projectile::Event::Effect { id, at } => {
                    let table = self.effect_table.clone();
                    self.effects.one_shot(&table, id, at, neopets_sim::effect::IDENTITY, &mut self.rng);
                }
            }
        }
        self.shot_effects_step();
        for (at, s, f, status, g, poison_attack) in strikes {
            // SPEC-0036 Enemy attack damage to a hero: the shield block, the armour Mote and the
            // body item, by the attack's damage alignment (no petpet).
            let element = self.characters.iter().find(|c| c.id == at).and_then(|c| c.enemy.as_ref()).map_or(0, |e| e.p.slots[s.slot].alignment);
            let damage = self.enemy_damage(s.damage, s.blocked, element, g);
            // Status 11 (poison attack): a landed melee hit also poisons the hero at the default
            // (SPEC-0029 Behavior 27.1), through the hero's apply.
            if poison_attack && !s.blocked {
                self.statuses.apply(neopets_sim::status::POISON as i32, -1);
            }
            let died = self.hero_hit(damage, s.knockback, s.blocked, f, format!("slot {}{}", s.slot, if s.blocked { ", blocked" } else { "" }));
            // SPEC-0038 Behavior 13 strike test: the status roll int[0, 100] strictly below the
            // chance, for int[int(range max), int(range min)] / 60 whole seconds.
            if status.0 != -1 && !s.blocked {
                let r = self.rng.range(0, 100);
                if (r as f32) < status.1 {
                    let ticks = self.rng.range(status.2 .1 as i32, status.2 .0 as i32);
                    self.statuses.apply(status.0, ticks / 60);
                }
            }
            if died {
                // A hit that leaves the hero at 0 HP: the attacker's victory pose (state 10).
                if let Some(c) = self.characters.iter_mut().find(|c| c.id == at) {
                    let Character { enemy: Some(e), walker: Some(wk), anim, .. } = c else { continue };
                    let has = |id: u32| anim.has(id);
                    let v = enemy_view(wk.position(), wk, hero, hero_facing, bands, false, true, (true, true), false, 0.0, anim.clip_state(), &has);
                    let act = e.hero_down(&v, &mut self.rng);
                    if let Some(id) = act.anim {
                        anim.request(id, false);
                    }
                }
            }
        }
    }

    /// Open a table line in the shared box (SPEC-0022 Showing text, Speaker name and colour): a
    /// "Hero" or "[Hero]" speaker (any case) or one starting with a control code names the hero
    /// with its table-10 colour; otherwise a talk partner's own name and colour; otherwise the
    /// record's.
    fn open_line(&mut self, line: u16) {
        let Some(l) = self.world.lines.line(line as usize) else { return };
        self.message = Some((line, self.ticks));
        let s = &l.raw_speaker;
        let hero = s.eq_ignore_ascii_case(b"Hero") || s.eq_ignore_ascii_case(b"[Hero]") || s.first() == Some(&0x01);
        let partner = self.talk_with.and_then(|id| self.characters.iter().find(|c| c.id == id));
        self.speaker = if hero {
            (self.hero_name().to_vec(), self.hero_colour)
        } else if let Some(c) = partner {
            (c.name.as_bytes().to_vec(), c.colour)
        } else {
            (l.speaker.as_bytes().to_vec(), l.colour)
        };
        self.textbox.open(&l.raw);
        self.dialogue_open = true;
    }

    /// The Square caption shown this pass (SPEC-0023 section 33; SPEC-0022 Interaction prompts):
    /// the talk prompt (label 8), else the door prompt the chooser shows (13 Enter, 26 Exit or a
    /// script door's context); none while a box or a quick panel is up.
    pub fn square_caption(&self) -> Option<&[u8]> {
        if self.dialogue_open || self.quick.open {
            return None;
        }
        // The "Save Game" caption (label 25) is set first in the pass, so the chooser's talk
        // and door captions replace it.
        if let Some(c) = self.buy_caption.as_deref() {
            return Some(c);
        }
        let id = if self.talk_prompt {
            Some(8)
        } else if self.prompt_shown.is_some() {
            self.prompt_shown
        } else if self.save_prompt {
            Some(25)
        } else {
            None
        };
        self.labels.get(id? as usize).map(Vec::as_slice).filter(|l| !l.is_empty())
    }

    /// The name and frame colour shown now (the type-32 override names the hero).
    pub fn box_speaker(&self) -> (&[u8], [u8; 3]) {
        if self.textbox.override_hero { (self.hero_name(), self.hero_colour) } else { (&self.speaker.0, self.speaker.1) }
    }

    /// SPEC-0036 Status effects and Regeneration, run in the hero update before its state
    /// routine: the timers, stun (state 1, speed scale 0), the poison hit (attacker -2, blocked
    /// flag 1), slow and speed, the regeneration statuses, then the trickle toward 10 % + 1.
    fn status_step(&mut self) {
        use neopets_sim::status as st;
        let state = self.loco.state;
        let moving = (2..=6).contains(&state) || state == 32;
        let attacking = self.attack.as_ref().is_some_and(|a| (2..=12).contains(&a.sub));
        let t = self.statuses.tick(self.hero_hp, moving, attacking);
        if t.stunned && state != hero_states::DEAD {
            self.attack = None;
            self.loco.set_end_state(hero_states::STANCE);
        }
        self.loco.status_scale = t.scale;
        if let Some(d) = t.poison {
            let f = self.loco.facing;
            if state != hero_states::DEAD {
                self.hero_hit(d as f32, 0.0, true, f, "poison".into());
            }
        }
        self.hero_hp = (self.hero_hp + t.hp).min(self.hero_max);
        self.hero_mp = (self.hero_mp + t.mp).min(self.hero_mp_max);
        let poisoned = self.statuses.active(st::POISON);
        // SPEC-0047 Passive bonuses: the petpet point is added to the item trickle of the same
        // tick before the cap, with no alive or poison gate.
        let (kind, t) = (self.petpet.as_ref().map(|p| p.kind), self.petpet.as_ref().map_or(0, |p| p.t));
        let (mut hp, mut mp) = (self.pet_bonus.hp_point(kind, t), self.pet_bonus.mp_point(kind, t));
        if !poisoned {
            mp += neopets_sim::status::trickle(self.hero_mp, self.hero_mp_max, self.ticks, false);
            if self.hero_hp > 0 && self.loco.state != hero_states::DEAD {
                hp += neopets_sim::status::trickle(self.hero_hp, self.hero_max, self.ticks, true);
            }
        }
        if hp > 0 {
            self.hero_hp = (self.hero_hp + hp).min(self.hero_max);
        }
        if mp > 0 {
            self.hero_mp = (self.hero_mp + mp).min(self.hero_mp_max);
        }
    }

    /// The quick panel's use step (forced): refused for a dead hero; a consumable heals pool A
    /// (health) and pool B (magic) by SPEC-0021 Consumable healing and is consumed when a pool
    /// changes. Charms run the equipped slot routine and the status and revive items their own
    /// effects, which are not implemented: their use is refused.
    fn use_item(&mut self, id: u16) {
        if self.loco.state == hero_states::DEAD {
            return;
        }
        // SPEC-0053 3.8 b: the Potion of Revival (item 50), in mode 2 with the partner dead,
        // heals the partner to its maximum and requests a switch to it (extra flag set).
        if id == 50 {
            let revive = self.swap_mode == 2 && self.parked.as_ref().is_some_and(|p| p.hp <= 0);
            if revive {
                if let Some(p) = self.parked.as_mut() {
                    p.hp = p.hp_max;
                }
                if self.request_switch(true, true, true) {
                    let left = {
                        let n = self.state.items.entry(id).or_insert(0);
                        *n = n.saturating_sub(1);
                        *n
                    };
                    self.quick.used(id as i32, left);
                    self.last_pickup = format!("used item 50 on tick {}: the partner is revived", self.ticks);
                }
            } else {
                self.cue(0xA2);
            }
            return;
        }
        let Some(item) = self.items.get(id).copied() else { return };
        use neopets_sim::status as st;
        // SPEC-0021 Consumable effects by item id: the buffs and immunities apply only while
        // inactive; the cure needs one of curse, poison or slow; 58 and 59 apply their
        // regeneration before the heal check.
        let buff = match id {
            51 => Some(st::IMMUNE_POISON),
            52 => Some(st::IMMUNE_CURSE),
            53 => Some(st::IMMUNE_SLOW),
            55 => Some(st::STRENGTH),
            56 => Some(st::SPEED),
            _ if !(1..=6).contains(&item.kind) && item.stats[3] > 0 => Some(st::IMMUNE_POISON),
            _ => None,
        };
        let applied = match (id, buff) {
            (_, Some(s)) => {
                let ok = !self.statuses.active(s) && self.statuses.apply(s as i32, -1);
                Some(ok)
            }
            (54, _) => {
                let any = [st::CURSE, st::POISON, st::SLOW].iter().any(|&s| self.statuses.active(s));
                if any {
                    for s in [st::CURSE, st::POISON, st::SLOW] {
                        self.statuses.clear(s);
                    }
                }
                Some(any)
            }
            _ => None,
        };
        if let Some(ok) = applied {
            if !ok {
                self.cue(162);
                self.last_pickup = format!("use of item {id} refused on tick {}", self.ticks);
                return;
            }
            // A buff or immunity plays 0xA0; the cure 0x9F.
            self.cue(if id == 54 { 159 } else { 160 });
            let left = {
                let n = self.state.items.entry(id).or_insert(0);
                *n = n.saturating_sub(1);
                *n
            };
            self.quick.used(id as i32, left);
            self.last_pickup = format!("used item {id} on tick {} ({left} left)", self.ticks);
            return;
        }
        match id {
            58 => {
                self.statuses.apply(st::HEALTH_REGEN as i32, -1);
            }
            59 => {
                self.statuses.apply(st::MAGIC_REGEN as i32, -1);
            }
            _ => {}
        }
        let (mut pa, mut pb) = (item.stats[4] as i32, item.stats[5] as i32);
        if (1..=6).contains(&item.kind) || item.kind == 17 {
            (pa, pb) = (0, 0);
        }
        match id {
            58 => pa = 100,
            59 => pb = 100,
            _ => {}
        }
        let ta = self.heal_tables.0.first().copied().unwrap_or(500);
        let tb = self.heal_tables.1.first().copied().unwrap_or(300);
        let a = neopets_sim::quick::heal(ta, pa, self.hero_hp, self.hero_max, true);
        let b = neopets_sim::quick::heal(tb, pb, self.hero_mp, self.hero_mp_max, true);
        if a.is_none() && b.is_none() {
            self.cue(162);
            self.last_pickup = format!("use of item {id} refused on tick {}", self.ticks);
            return;
        }
        // A heal plays 0x9F (0xA0 for 58 and 59), then 0x9B or 0x9C above 10 percent (pool B's
        // replaces pool A's).
        self.cue(if matches!(id, 58 | 59) { 160 } else { 159 });
        if pb > 10 && b.is_some() {
            self.cue(156);
        } else if pa > 10 && a.is_some() {
            self.cue(155);
        }
        if let Some(hp) = a {
            self.hero_hp = hp;
        }
        if let Some(mp) = b {
            self.hero_mp = mp;
        }
        let left = {
            let n = self.state.items.entry(id).or_insert(0);
            *n = n.saturating_sub(1);
            *n
        };
        self.quick.used(id as i32, left);
        self.last_pickup = format!("used item {id} on tick {}: hp {}, mp {} ({left} left)", self.ticks, self.hero_hp, self.hero_mp);
    }

    /// The item type of an id (config type; -1 when the table has no record).
    fn item_kind(&self, id: u16) -> i16 {
        self.items.get(id).map_or(-1, |i| i.kind)
    }

    /// SPEC-0021 Pickup dispatch, region start-up: for each kind-8 point, the luck gate (one
    /// draw r = int[0, 100] when its gate field is set), for keys 0 or more the equipment veto
    /// and the once-per-area and held vetoes, then the flag test (0 or 1 spawn, placed, with the
    /// key as the id: a negative key is refused by the spawn).
    fn region_items(&mut self, pack: u32, items: &[neopets_data::region::ItemPoint]) {
        for p in items {
            if p.luck != 0 {
                let r = self.rng.range(0, 100);
                if !neopets_sim::items::luck_passes(self.luck, r) {
                    continue;
                }
            }
            if p.key >= 0 && self.vetoed(p.key as u16, pack) {
                continue;
            }
            // A map placement is not spawned when its map (+0x2C plus 1) is held.
            if p.key == level::TREASURE_MAP as i16 && self.state.maps.contains(&(p.map as u32 + 1)) {
                continue;
            }
            if p.flags > 1 || p.key < 0 {
                continue;
            }
            let id = p.key as u16;
            let spawn = neopets_sim::items::Spawn {
                id,
                point: p.position,
                placed: true,
                charm: self.item_kind(id) == 17,
                s0: 1.0,
                s1: 1.0,
                delay: 0.0,
                larg: -1.0,
                direction: Some(p.facing),
                flag: p.flags,
                map: p.map,
                price: p.count,
            };
            self.spawn_item(pack, spawn);
        }
    }

    /// The equipment veto (types 1 to 6; Tor's entries for 4 to 6) and the quest-style vetoes
    /// (once-per-area ids 19, 20, 151, 154 and 174; item 126 when held). Map 175's veto needs
    /// the placement's map byte and is applied by the callers.
    fn vetoed(&self, id: u16, pack: u32) -> bool {
        let kind = self.item_kind(id);
        if (4..=6).contains(&kind) {
            let equipped = [self.weapon, self.shield].into_iter().flatten().find(|&e| self.item_kind(e) == kind);
            if equipped.is_some_and(|e| id <= e) {
                return true;
            }
        }
        match id {
            19 | 20 | 151 | 154 | 174 => self.once.contains(&(id, pack)),
            126 => self.state.items.get(&126).is_some_and(|&n| n > 0),
            _ => false,
        }
    }

    fn spawn_item(&mut self, pack: u32, s: neopets_sim::items::Spawn) -> bool {
        let world = [&self.world].into_iter().chain(self.other.as_ref()).find(|w| w.pack == pack);
        let ground = |x: f32, z: f32, c: f32| world.and_then(|w| w.ground_at(x, z, c));
        let rng = &mut self.rng;
        let mut draw = |lo: f32, hi: f32| rng.frange(lo, hi);
        self.pools.entry(pack).or_default().spawn(s, &ground, &mut draw)
    }

    /// SPEC-0038 Behavior 34 parts B and D for a first death tick (no direct-item word and no
    /// carried Neopoints yet): the drop placement's key spawns that item; a key of -1 runs the
    /// drop action (money roll, coin split or item roll; SPEC-0021 Drop and money flow).
    fn pay_out(&mut self, pack: u32, at: [f32; 3], drop: Option<neopets_data::region::ItemPoint>) {
        let Some(p) = drop else { return };
        let direction = self.dig_drop;
        let drop_spawn = |id: u16, charm: bool| neopets_sim::items::Spawn {
            id,
            point: at,
            placed: false,
            charm,
            s0: 0.25,
            s1: 1.0,
            delay: 0.0,
            larg: -1.0,
            direction,
            flag: p.flags,
            map: p.map,
            price: 0,
        };
        if p.key >= 0 {
            let id = p.key as u16;
            let charm = self.item_kind(id) == 17;
            self.spawn_item(pack, drop_spawn(id, charm));
            return;
        }
        let group = usize::try_from(p.group).ok().and_then(|g| self.items.groups.get(g)).map(|g| neopets_sim::items::Group {
            chance: g.chance as i32,
            min: g.min as i32,
            max: g.max as i32,
            entries: g.entries.clone(),
        });
        let roll = {
            let rng = std::cell::RefCell::new(&mut self.rng);
            let mut index = |n: u32| rng.borrow_mut().index(n);
            let mut range = |a: i32, b: i32| rng.borrow_mut().range(a, b);
            neopets_sim::items::drop_roll(group.as_ref(), &self.items.sub_tables, p.count, &mut index, &mut range)
        };
        match roll {
            neopets_sim::items::Drop::Coins(m) => {
                // With a direction (a dig reward) each coin starts 0.4 s after the one before.
                let step = if direction.is_some() { 0.4 } else { 0.0 };
                for (k, id) in neopets_sim::items::coin_split(m, &self.coins.0, &self.coins.1).into_iter().enumerate() {
                    self.spawn_item(pack, neopets_sim::items::Spawn { delay: step * k as f32, ..drop_spawn(id, false) });
                }
            }
            neopets_sim::items::Drop::Item(id) => {
                let charm = self.item_kind(id) == 17;
                self.spawn_item(pack, drop_spawn(id, charm));
            }
            neopets_sim::items::Drop::Nothing => {}
        }
    }

    /// The world items' per-tick step for the resident worlds' pools, the enemy payouts due,
    /// and the hero's pickup in the current world (SPEC-0021 Pickup, Auto-collect, Coin pickups).
    fn items_step(&mut self) {
        for (pack, at, drop) in std::mem::take(&mut self.payouts) {
            self.pay_out(pack, at, drop);
        }
        let Some(coins) = self.coins.2 else { return };
        let packs: Vec<u32> = self.pools.keys().copied().collect();
        for pack in packs {
            let world = [&self.world].into_iter().chain(self.other.as_ref()).find(|w| w.pack == pack);
            let ground = |x: f32, z: f32, c: f32| world.and_then(|w| w.ground_at(x, z, c));
            let rng = &mut self.rng;
            let mut draw = |lo: f32, hi: f32| rng.frange(lo, hi);
            if let Some(pool) = self.pools.get_mut(&pack) {
                pool.tick(&coins, &ground, &mut draw);
            }
        }
        let p = self.loco.position();
        let pack = self.world.pack;
        let base = self.items.get(146).map_or(1.5, |i| i.pickup_base);
        let Some(i) = self.pools.get(&pack).and_then(|pool| pool.collectable(p, base, contact::HERO_RADIUS)) else { return };
        let Some(it) = self.pools.get(&pack).and_then(|pool| pool.slots[i]) else { return };
        let coin = neopets_sim::items::is_coin(it.id);
        let mote = (160..=173).contains(&it.id);
        let state = self.loco.state;
        if !coin && (matches!(state, 0x19 | 0x21 | 0x33 | 0x37) || (!mote && (7..=0x3C).contains(&state))) {
            return;
        }
        if it.id >= 191 && !it.charm {
            return;
        }
        if it.id == level::TREASURE_MAP && self.state.maps.contains(&(it.map as u32 + 1)) {
            return;
        }
        if let Some(pool) = self.pools.get_mut(&pack) {
            pool.take(i);
        }
        if coin {
            let value = self.coins.1.iter().position(|&c| c == it.id).map_or(0, |k| self.coins.0[k]);
            if value > 0 {
                self.state.neopoints = self.state.neopoints.checked_add(value).unwrap_or(0).max(0);
            }
            // SPEC-0035 section 15.2: line 1 shows the gain, or the running total within 3 s.
            let l1 = &mut self.price_lines[0];
            l1.0 = if l1.1 > 0 { l1.0 + value } else { value };
            l1.1 = 180;
            self.cue(165);
            self.last_pickup = format!("picked up coin {} (+{value} NP, wallet {}) on tick {}", it.id, self.state.neopoints, self.ticks);
        } else {
            if it.id == level::TREASURE_MAP {
                // A map pickup's number is the placement byte +0x2C plus 1; the pickup is
                // vetoed (handled before the take) when that map is held.
                self.map_held(it.map as u32 + 1);
                self.last_pickup = format!("picked up map {} on tick {}", it.map as u32 + 1, self.ticks);
                return;
            }
            let count = if it.charm { 100 } else { 1 };
            *self.state.items.entry(it.id).or_insert(0) += count;
            if matches!(it.id, 19 | 20 | 151 | 154 | 174) {
                self.once.insert((it.id, pack));
            }
            self.last_pickup = format!("picked up item {} (held {}) on tick {}", it.id, self.state.items[&it.id], self.ticks);
            let hook = level::on_pickup(pack, it.id, &mut self.state);
            self.apply(&hook);
        }
    }

    /// A common-enemy strike on the hero: SPEC-0036 Enemy attack damage to a hero (no guard,
    /// so no block; no armour, Motes or petpet; status scale 1), Taking a hit (the dialogue box
    /// discards hits; a dead hero takes none; Tor's charge sub-states 1 and 10 are busy), the
    /// deduct and the default death, and the knockback. Returns whether the hero died.
    fn hero_hit(&mut self, damage: f32, knockback: f32, blocked: bool, attacker_facing: [f32; 3], what: String) -> bool {
        if self.dialogue_open || self.loco.state == hero_states::DEAD {
            return false;
        }
        let damage = damage.max(0.0).trunc() as i32;
        // A hit on a living hero ends an active stun.
        self.statuses.clear(neopets_sim::status::STUN);
        let busy = self.attack.as_ref().is_some_and(|a| matches!(a.sub, 1 | 10));
        if !busy && blocked {
            // The shield slot's "block small".
            self.sound.play(neopets_sim::sound::SHIELD, 0, neopets_sim::sound::Play::default());
            self.loco.block();
        } else if !busy {
            self.attack = None;
            self.loco.hurt(self.weapon.is_some());
        }
        self.hero_hp -= damage;
        self.last_hurt = format!("took {damage} ({what}) on tick {}, hp {}", self.ticks, self.hero_hp.max(0));
        // SPEC-0031 Hero impact: melee or projectile with no armour plays 170; the voice slot
        // plays hit big (1) on a fair draw of 0, else hit small (0).
        self.cue(170);
        let big = self.rng.next_u32() % 2 == 0;
        self.sound.play(neopets_sim::sound::VOICE, if big { 1 } else { 0 }, neopets_sim::sound::Play::default());
        if self.hero_hp <= 0 {
            self.hero_hp = 0;
            self.default_death();
            return true;
        }
        if damage > 0 && knockback > 0.0 {
            let d = unit([9.0 * attacker_facing[0], 9.0 * attacker_facing[1] + 12.0, 9.0 * attacker_facing[2]]);
            self.attack = None;
            self.loco.knock(scale(d, knockback));
        }
        false
    }

    /// SPEC-0036 Health and death, step 3 (no death callback is registered by any level here):
    /// state 16, the melee hit list cleared.
    fn default_death(&mut self) {
        // SPEC-0055 Death cue test: sound event 3 (stinger 3, "Character Dies") while the other
        // hero is not dead (Roberta is not in this build and counts as alive).
        self.stinger(3);
        // The "knocked out" voice cue, at the hero, do-not-restart.
        let p = self.loco.position();
        self.sound.play(neopets_sim::sound::VOICE, 2, neopets_sim::sound::Play { emitter: Some(p), do_not_restart: true, ..Default::default() });
        self.attack = None;
        self.hit_list.clear();
        self.statuses.clear_all();
        self.loco.die();
    }

    /// SPEC-0025 Save points, per tick for the current world's own region: the nearest point,
    /// the prompt within 5 units, and on a Square press edge with the hero not busy the restart
    /// point (zero-flag branch) and the save screen's opening. The gate is game-state code 0:
    /// no door job and no death or game-over state, then the glow (SPEC-0034 section 36).
    fn save_point_step(&mut self, square: bool) {
        let gate = self.job.is_none() && self.death.page.is_none() && !self.death.flag_e;
        let busy = hero_states::busy_state(self.loco.state);
        let r = self.save_points.step(&self.world.save_points, self.loco.position(), gate, square, busy);
        self.save_prompt = r.prompt;
        self.save_glow_step(r.glow_at);
        // Cue 0xA8 is kept alive within 15 units, with no emitter (the zero-flag branch's quirk).
        if r.cue {
            self.sound.keep_alive(neopets_sim::sound::MAIN, 0xA8, None);
        }
        if r.open {
            if r.restart {
                self.restart = Some((self.loco.position(), self.loco.facing));
            }
            self.save_open = true;
        }
    }

    /// SPEC-0060: build each resident world's patches once (a family whose model the pack does
    /// not register builds nothing; grass's model 283 is resource 9's), drop the patches of a
    /// world that left, and run the bush logic of the patches within 100 units of the camera eye:
    /// the weapon test within 5 units in the attack window (a capsule of radius 0.5 and length
    /// 6.0 up from the base against the weapon segment), the shot test (each armed shot within
    /// its radius + 1.5 of the base; not for key 5), the cut's drop roll and the sparkle 139.
    fn foliage_step(&mut self) {
        let resident: Vec<u32> = [&self.world].into_iter().chain(self.other.as_ref()).map(|w| w.pack).collect();
        self.foliage.retain(|p, _| resident.contains(p));
        for pack in resident {
            if self.foliage.contains_key(&pack) {
                continue;
            }
            let Some(world) = [&self.world].into_iter().chain(self.other.as_ref()).find(|w| w.pack == pack) else { continue };
            let mut rng = neopets_sim::rng::Rng::seeded(pack);
            let mut built = Vec::new();
            for p in &world.patches {
                let Some(kind) = neopets_sim::foliage::kind(p.key) else { continue };
                let Some(model) = kind.near.or(kind.far) else { continue };
                if !((283..=320).contains(&model) || world.children.contains(&model)) {
                    continue;
                }
                let j = neopets_sim::foliage::jitter(kind.family, p.jitter, p.area[3] - p.area[2], p.rows.len());
                let ceiling = neopets_sim::foliage::probe_ceiling(&kind).unwrap_or(neopets_sim::motion::PROBE_CEILING);
                let ground = |x: f32, z: f32| world.ground_at(x, z, p.height + ceiling);
                let plants: Vec<_> = p
                    .rows
                    .iter()
                    .flat_map(|row| row.plants.iter().map(move |q| (row.offset, q)))
                    .filter_map(|(row, q)| neopets_sim::foliage::place(&kind, p.area, p.height, row, q.offset, j, q.colour, &ground, &mut rng))
                    .collect();
                let bushes = if kind.family == neopets_sim::foliage::Family::Bush { plants.iter().map(|q| neopets_sim::foliage::Bush::new(q.scale, &mut rng)).collect() } else { Vec::new() };
                let centre = [(p.area[0] + p.area[1]) / 2.0, p.height, (p.area[2] + p.area[3]) / 2.0];
                built.push(LivePatch { kind, model, centre, plants, bushes, zone: p.zone as u8, on: true });
            }
            self.foliage.insert(pack, built);
        }
        // The gate's zone query: key -1 for the current world, the world's id otherwise.
        for (&pack, patches) in self.foliage.iter_mut() {
            let key = if pack == self.world.pack { -1 } else { self.other.as_ref().map_or(-1, |o| o.world_id) };
            for p in patches.iter_mut() {
                p.on = self.zones.query(self.world.zones.as_ref(), p.zone as i32, key);
            }
        }
        // The bush logic.
        let eye = self.camera.eye;
        let hero = self.loco.position();
        let weapon = self.weapon_segment();
        let shots: Vec<([f32; 3], f32)> = self.hero_shots.iter().map(|s| (s.position, s.radius)).collect();
        let table = self.effect_table.clone();
        let mut drops: Vec<(u32, [f32; 3], Vec<u16>)> = Vec::new();
        let mut sparkles = Vec::new();
        let current = self.world.pack;
        for (&pack, patches) in self.foliage.iter_mut() {
            for p in patches.iter_mut().filter(|p| !p.bushes.is_empty() && p.on) {
                let d = [p.centre[0] - eye[0], p.centre[1] - eye[1], p.centre[2] - eye[2]];
                if d[0] * d[0] + d[1] * d[1] + d[2] * d[2] >= 100.0 * 100.0 {
                    continue;
                }
                let negg = neopets_sim::foliage::negg_of(p.model);
                let negg_open = negg.is_none_or(|n| !self.once.contains(&(n, current)));
                let may_sparkle = p.model == 262 || (negg.is_some() && negg_open);
                for (q, b) in p.plants.iter().zip(p.bushes.iter_mut()) {
                    let mut hits = 0;
                    if b.hittable() {
                        let near = (q.position[0] - hero[0]).powi(2) + (q.position[1] - hero[1]).powi(2) + (q.position[2] - hero[2]).powi(2) < 25.0;
                        if let (true, Some(w)) = (near, weapon.as_ref()) {
                            let bush = neopets_sim::combat::Cylinder::from_base(q.position, [0.0, 1.0, 0.0], 0.5, 6.0);
                            if neopets_sim::combat::overlap(w, &bush) {
                                hits += 1;
                            }
                        }
                        if p.kind.near != Some(198) {
                            hits += shots.iter().filter(|(s, r)| ((s[0] - q.position[0]).powi(2) + (s[1] - q.position[1]).powi(2) + (s[2] - q.position[2]).powi(2)).sqrt() < r + 1.5).count() as u32;
                        }
                    }
                    let step = b.step(hits, may_sparkle, &mut self.rng);
                    for _ in 0..step.cuts {
                        let r = self.rng.range(0, 100);
                        drops.push((pack, q.position, neopets_sim::foliage::bush_drop(p.model, r, self.luck, negg_open)));
                    }
                    if step.sparkle {
                        sparkles.push(q.position);
                    }
                }
            }
        }
        for at in sparkles {
            self.effects.one_shot(&table, 139, at, neopets_sim::effect::IDENTITY, &mut self.rng);
        }
        // Behavior 8: a key-4 patch within 100 units of the eye moves its solid cylinder (height
        // 25, radius 5.5 x the tree's scale) to each tree in turn and tests the hero against it;
        // here as a horizontal push of the hero's body out of the cylinder (the shared cylinder
        // contact response of SPEC-0018 is not built).
        let pack = self.world.pack;
        let trunks: Vec<([f32; 3], f32)> = self
            .foliage
            .get(&pack)
            .into_iter()
            .flatten()
            .filter(|p| p.kind.family == neopets_sim::foliage::Family::Tree && p.kind.near == Some(200) && p.kind.far.is_none())
            .filter(|p| (p.centre[0] - eye[0]).powi(2) + (p.centre[1] - eye[1]).powi(2) + (p.centre[2] - eye[2]).powi(2) < 100.0 * 100.0)
            .flat_map(|p| p.plants.iter().map(|q| (q.position, 5.5 * q.scale)))
            .collect();
        for (base, radius) in trunks {
            let pos = self.loco.position();
            let (dx, dz) = (pos[0] - base[0], pos[2] - base[2]);
            let d = (dx * dx + dz * dz).sqrt();
            let r = radius + contact::HERO_RADIUS;
            if d < r && d > 1e-4 && pos[1] >= base[1] - 1.0 && pos[1] <= base[1] + 25.0 {
                let k = (r - d) / d;
                let b = &mut self.loco.body.position;
                b[0] += dx * k;
                b[2] += dz * k;
            }
        }
        for (pack, at, ids) in drops {
            self.last_fx = format!("bush cut at {:.1},{:.1}: drop {ids:?} on tick {}", at[0], at[2], self.ticks);
            for (k, id) in ids.into_iter().enumerate() {
                // The coin bush's coins after the first are offset by [0.2, 0.7] in x and z.
                let point = if k == 0 { at } else { [at[0] + self.rng.frange(0.2, 0.7), at[1], at[2] + self.rng.frange(0.2, 0.7)] };
                let charm = self.item_kind(id) == 17;
                let spawn = neopets_sim::items::Spawn { id, point, placed: false, charm, s0: 0.25, s1: 1.0, delay: 0.0, larg: -1.0, direction: None, flag: 0, map: 0, price: 0 };
                self.spawn_item(pack, spawn);
            }
        }
    }

    /// The hero's weapon segment as a thin cylinder while the attack window is open (frame bit
    /// 1; SPEC-0036), from the weapon attach point along its axis over the weapon reach.
    fn weapon_segment(&self) -> Option<neopets_sim::combat::Cylinder> {
        if self.attack.is_none() || self.hero.frame_bits() & 1 == 0 {
            return None;
        }
        let place = placement(self.loco.position(), self.loco.facing);
        let frames = self.hero.bone_frames();
        let &(Some(bone), angles, offset) = self.hero.bones().attach.first()? else { return None };
        let w = place.mul(frames.get(bone)?).mul(&attach_rotation(offset, angles));
        let axis = unit(w.cols[2]);
        let l = self.weapon_reach() + 2.0;
        let p0 = add(w.t, scale(axis, 0.25));
        Some(neopets_sim::combat::Cylinder { center: add(p0, scale(axis, 0.5 * l)), axis, radius: 0.0, length: l })
    }

    /// A placed object's drawn transform (SPEC-0028 7.1 step 6) as a column-major matrix, or
    /// None while it is hidden. In a pack whose object calls are not reproduced yet, an object no
    /// script call has touched is drawn at its rest block (a deviation: the original hides it
    /// until InitObjOn).
    pub fn object_matrix(&self, pack: u32, i: usize) -> Option<[f32; 16]> {
        let w = [&self.world].into_iter().chain(self.other.as_ref()).find(|w| w.pack == pack)?;
        let obj = w.objects.get(i)?.as_ref()?;
        let a = w.object_anims.get(i)?;
        // An object a type-40 group names (chests and the other group kinds, whose setup is not
        // built) is also drawn at rest until a script touches it.
        let grouped = w.groups.iter().any(|g| g.objects.contains(&(i as i32)));
        let gated = level::objects_scripted(pack) || (pack == 1217 && neopets_sim::burrows::named(i));
        if !a.scripted && (!gated || grouped) {
            return Some(obj.rest);
        }
        // SPEC-0059 Layout: the 1023 wheel is object 0 turned to A about its depth axis.
        if pack == 1023 && i == 0 && a.visible() {
            let angle = level::wheel_of(pack, &self.state).angle.to_radians();
            let (s, c) = angle.sin_cos();
            let m = obj.rest;
            let (x, y) = ([m[0], m[1], m[2]], [m[4], m[5], m[6]]);
            let rx: [f32; 3] = std::array::from_fn(|k| c * x[k] + s * y[k]);
            let ry: [f32; 3] = std::array::from_fn(|k| -s * x[k] + c * y[k]);
            return Some([rx[0], rx[1], rx[2], 0.0, ry[0], ry[1], ry[2], 0.0, m[8], m[9], m[10], 0.0, m[12], m[13], m[14], 1.0]);
        }
        if !a.visible() {
            return None;
        }
        let (q, root, scale) = w.motions.get(i)?.sample(a.time);
        let r = neopets_sim::object::rotation(q);
        let t = [obj.rest[12] + root[0], obj.rest[13] + root[1], obj.rest[14] + root[2]];
        let c = |k: usize| r[k].map(|v| v * scale[k]);
        let (c0, c1, c2) = (c(0), c(1), c(2));
        Some([c0[0], c0[1], c0[2], 0.0, c1[0], c1[1], c1[2], 0.0, c2[0], c2[1], c2[2], 0.0, t[0], t[1], t[2], 1.0])
    }

    /// The id the status displays use for the selected hero.
    const HERO_DISPLAY: u64 = u64::MAX;

    /// SPEC-0036 Status display and cues for the hero and the common enemies: a status that
    /// became active this tick runs the display start (codes 8 to 10 play their one-shot), the
    /// rotation steps, the shown status's effect follows the character (curse 1.5 above), and a
    /// stun that ended plays 0xCD. The effects sit at the character's origin: the main attach
    /// point is not identified (R-0045).
    fn status_display_step(&mut self) {
        let table = self.effect_table.clone();
        let mut who: Vec<(u64, [bool; 14], [f32; 3])> = vec![(Self::HERO_DISPLAY, std::array::from_fn(|c| self.statuses.active(c)), self.loco.position())];
        for c in &self.characters {
            if let Some(e) = c.enemy.as_ref().filter(|e| !e.dead && !c.inactive) {
                who.push((c.id, std::array::from_fn(|k| e.status(k)), c.position));
            }
        }
        let live: std::collections::HashSet<u64> = who.iter().map(|w| w.0).collect();
        let gone: Vec<u64> = self.displays.keys().filter(|k| !live.contains(k)).copied().collect();
        for k in gone {
            self.overrides.remove(&k);
            if let Some((_, Some(h), _)) = self.displays.remove(&k) {
                self.effects.stop(&table, h);
                self.effects.release(h);
            }
        }
        for (id, active, at) in who {
            let (mut d, mut h, prev) = self.displays.get(&id).copied().unwrap_or_default();
            let mut changed = false;
            for code in 0..14 {
                if active[code] && !prev[code] {
                    if (8..=10).contains(&code) {
                        if let Some(e) = neopets_sim::status::display_effect(code as i32) {
                            self.effects.one_shot(&table, e, at, neopets_sim::effect::IDENTITY, &mut self.rng);
                        }
                    }
                    changed |= d.start(code as i32);
                }
            }
            if prev[0] && !active[0] {
                self.effects.one_shot(&table, neopets_sim::status::STUN_END_EFFECT, at, neopets_sim::effect::IDENTITY, &mut self.rng);
            }
            changed |= d.tick(&|c: i32| active.get(c as usize).copied().unwrap_or(false));
            // Colour-override slot 0: cleared when the shown status changes; the stun start sets
            // it once in mode 0; the tint step of codes 1 to 5 and 7 sets it in mode 1 (cleared
            // when the capped colour is white); one blend tick, with the region character's
            // per-tick colour push (the hero has none).
            let o = self.overrides.entry(id).or_default();
            if changed {
                o.clear();
                // Deviation (R-0052): the hero's colour is restored to white at the clear;
                // read literally, it would keep the drifted colour, as it has no push.
                if id == Self::HERO_DISPLAY {
                    o.colour = [1.0; 3];
                }
            }
            if active[0] && !prev[0] {
                o.set(neopets_sim::status::STUN_TINT, 0);
            }
            if matches!(d.shown, 1..=5 | 7) {
                match neopets_sim::status::tint(d.shown, if id == Self::HERO_DISPLAY { self.hero_index } else { 0 }) {
                    Some(t) => o.set(t, 1),
                    None => o.clear(),
                }
            }
            o.tick((id != Self::HERO_DISPLAY).then_some([1.0; 3]));
            let point = if d.shown == 1 { [at[0], at[1] + 1.5, at[2]] } else { at };
            if changed {
                if let Some(old) = h.take() {
                    self.effects.stop(&table, old);
                    self.effects.release(old);
                }
                if let Some(e) = neopets_sim::status::display_effect(d.shown).filter(|_| (0..=7).contains(&d.shown)) {
                    let o = self.effects.add_object(neopets_sim::effect::EffectObject::new(&table, e, point, neopets_sim::effect::IDENTITY));
                    self.effects.update(&table, o, None, &mut self.rng);
                    h = Some(o);
                }
            } else if let Some(o) = h {
                self.effects.set_position(o, point);
            }
            self.displays.insert(id, (d, h, active));
        }
    }

    /// SPEC-0046 Character lights, once per pass after the effect pools: for the hero and each
    /// region character standing on a ground entry whose zone passes the zone query (with the
    /// character's world id), the walk over the current area's light set (entry zone gate: the
    /// query with key -1) and the pass's dynamic point lights from the light emitters, and its
    /// colour fill; a character that fails keeps its state. States of released characters are
    /// dropped.
    fn light_step(&mut self) {
        let grid = self.world.zones.as_ref();
        let zones = self.zones;
        let entry_zone = |z: u8| zones.query(grid, z as i32, -1);
        let dynamic = self.effects.lights(&self.effect_table);
        let mut walk = |key: Option<u64>, at: [f32; 3], world: Option<&World>| {
            let Some(w) = world else { return };
            let hit = w.probe(at[0], at[2], at[1] + 2.5);
            let Some(z) = hit.zone.filter(|_| hit.found) else { return };
            if !zones.query(grid, z as i32, w.world_id) {
                return;
            }
            let s = self.char_lights.entry(key).or_default();
            s.walk(&self.world.lights, &dynamic, at, &entry_zone);
            s.fill();
        };
        walk(None, self.loco.position(), Some(&self.world));
        for c in &self.characters {
            walk(Some(c.id), c.position, [&self.world].into_iter().chain(self.other.as_ref()).find(|w| w.pack == c.pack));
        }
        let live: std::collections::HashSet<u64> = self.characters.iter().map(|c| c.id).collect();
        self.char_lights.retain(|k, _| k.is_none_or(|id| live.contains(&id)));
    }

    /// The light block of the hero (None) or a character for a model at `at`: its light state
    /// (SPEC-0046 Light slots and the lit draw), or the scene default lights before its first
    /// walk. Directions are toward the light (the negated travel direction).
    pub fn lights(&self, id: Option<u64>, at: [f32; 3]) -> neopets_data::light::Lights {
        let Some(s) = self.char_lights.get(&id) else { return neopets_data::light::Lights::scene_default() };
        let (ambient, block) = s.block(at);
        let toward = |d: [f32; 3]| {
            let l = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
            if l > 0.0 { d.map(|c| -c / l) } else { [0.0; 3] }
        };
        neopets_data::light::Lights { ambient, directional: block.map(|(d, c)| (toward(d), c)) }
    }

    /// Diagnostic: the hero's light slots (entry type, flag, weight and stored colour).
    pub fn light_summary(&self) -> String {
        let Some(s) = self.char_lights.get(&None) else { return "none".into() };
        s.slots
            .iter()
            .map(|x| match x.entry {
                Some(e) => format!("[t{} f{} w{:.3} ({:.1}, {:.1}, {:.1})]", e.kind, e.flag, x.weight, x.stored[0], x.stored[1], x.stored[2]),
                None => "[-]".into(),
            })
            .collect::<Vec<_>>()
            .join(" ")
    }

    /// The camera's zone update (SPEC-0006 Zone manager): K = the current world's id and R =
    /// the zone of the hero's ground entry (camera mode 0 follows the hero); the active block is
    /// block 0 for the current world's passes.
    fn zone_step(&mut self) {
        let p = self.loco.position();
        let hit = self.world.probe(p[0], p[2], p[1] + 2.5);
        self.zones.active = Some(0);
        self.zones.update(self.world.zones.as_ref(), self.world.world_id, hit.zone.filter(|_| hit.found));
        // The cell flag of each region character: 1 without a ground entry, else the zone query
        // with the entry's zone and the character's world id. The live petpet is not a region
        // slot and is not gated.
        let (grid, zones, pet) = (self.world.zones.as_ref(), self.zones, self.pet_char);
        for c in &mut self.characters {
            let Some(w) = [&self.world].into_iter().chain(self.other.as_ref()).find(|w| w.pack == c.pack) else { continue };
            let hit = w.probe(c.position[0], c.position[2], c.position[1] + 2.5);
            let cell = Some(c.id) == pet || hit.zone.filter(|_| hit.found).is_none_or(|z| zones.query(grid, z as i32, w.world_id));
            c.inactive = c.deactivated || !cell;
        }
    }

    /// Diagnostic: the active placed records and the live spawners of each.
    pub fn placed_summary(&self) -> String {
        let mut on: Vec<(u32, usize)> = self.placed_on.iter().copied().collect();
        on.sort_unstable();
        on.iter()
            .map(|&(p, k)| {
                let live = self.placed_fx.get(&(p, k)).and_then(|&h| self.effects.objects.get(h)).and_then(|o| o.as_ref()).map_or(0, |o| o.slots.iter().flatten().count());
                format!("{p}:{k} ({live} live)")
            })
            .collect::<Vec<_>>()
            .join(", ")
    }

    /// Whether patch `i` of world `pack` passed its zone gate this pass.
    pub fn patch_on(&self, pack: u32, i: usize) -> bool {
        self.foliage.get(&pack).and_then(|p| p.get(i)).is_none_or(|p| p.on)
    }

    /// The colour-slot vector of the hero (None) or a character (SPEC-0034 5.4 and section 21):
    /// its colour-override slot 0's stored colour, None while it is white.
    pub fn status_tint(&self, id: Option<u64>) -> Option<[f32; 3]> {
        let o = self.overrides.get(&id.unwrap_or(Self::HERO_DISPLAY))?;
        (o.colour != [1.0; 3]).then_some(o.colour)
    }

    /// StartEffect (SPEC-0030 row 6; SPEC-0034 section 6): placed record `slot` of world `pack`
    /// is an effect object bound to its definition (its own cull kept when positive or -1); the
    /// call runs one update, with the camera as the listener when `camera` is set. An index
    /// outside the table does nothing.
    fn start_placed(&mut self, pack: u32, slot: usize, camera: bool) {
        let table = self.effect_table.clone();
        let Some(h) = self.placed_object(pack, slot) else { return };
        let listener = camera.then(|| {
            let (eye, focus) = (self.camera.eye, self.camera.focus);
            (eye, unit([focus[0] - eye[0], focus[1] - eye[1], focus[2] - eye[2]]))
        });
        self.effects.update(&table, h, listener, &mut self.rng);
    }

    /// Placed record `slot` of world `pack` as an effect object (made on first use): bound to its
    /// definition, with the record's own cull kept when positive or -1. None for an index
    /// outside the table.
    fn placed_object(&mut self, pack: u32, slot: usize) -> Option<usize> {
        if let Some(&h) = self.placed_fx.get(&(pack, slot)) {
            return Some(h);
        }
        let table = self.effect_table.clone();
        let r = [&self.world].into_iter().chain(self.other.as_ref()).find(|w| w.pack == pack).and_then(|w| w.placed_effects.get(slot)).copied()?;
        let mut o = neopets_sim::effect::EffectObject::new(&table, r.id, r.position, r.frame);
        if r.cull > 0.0 || r.cull == -1.0 {
            o.cull = Some(r.cull);
        }
        let h = self.effects.add_object(o);
        self.placed_fx.insert((pack, slot), h);
        Some(h)
    }

    /// The switch setter's effect on placed record `slot` of world `pack` (SPEC-0030 section
    /// 2.1): on sets the active flag; off clears it and drains the record.
    fn switch_placed(&mut self, pack: u32, slot: usize, on: bool) {
        if on {
            if [&self.world].into_iter().chain(self.other.as_ref()).any(|w| w.pack == pack && slot < w.placed_effects.len()) {
                self.placed_on.insert((pack, slot));
            }
            return;
        }
        self.placed_on.remove(&(pack, slot));
        if let Some(&h) = self.placed_fx.get(&(pack, slot)) {
            let table = self.effect_table.clone();
            self.effects.stop(&table, h);
        }
    }

    /// The per-pass zone gate of the placed records (SPEC-0028 section 9; SPEC-0034 section 6):
    /// every active record of the current and the second world is queried with its zone (+0x44,
    /// low byte) and that world's id; it is updated with the camera as the listener when the
    /// query passes and stopped when it fails.
    fn placed_zone_step(&mut self) {
        let table = self.effect_table.clone();
        let (eye, focus) = (self.camera.eye, self.camera.focus);
        let listener = (eye, unit([focus[0] - eye[0], focus[1] - eye[1], focus[2] - eye[2]]));
        let mut on: Vec<(u32, usize)> = self.placed_on.iter().copied().collect();
        on.sort_unstable();
        for (pack, slot) in on {
            let Some(w) = [&self.world].into_iter().chain(self.other.as_ref()).find(|w| w.pack == pack) else { continue };
            let Some(r) = w.placed_effects.get(slot) else { continue };
            let visible = self.zones.query(self.world.zones.as_ref(), (r.zone & 0xFF) as i32, w.world_id);
            let Some(h) = self.placed_object(pack, slot) else { continue };
            if visible {
                self.effects.update(&table, h, Some(listener), &mut self.rng);
            } else {
                self.effects.stop(&table, h);
            }
        }
    }

    /// World reset of the placed effects (SPEC-0034 section 6): a world no longer resident
    /// drains and releases its started records.
    fn placed_release_step(&mut self) {
        let table = self.effect_table.clone();
        let resident: Vec<u32> = [&self.world].into_iter().chain(self.other.as_ref()).map(|w| w.pack).collect();
        self.placed_on.retain(|(p, _)| resident.contains(p));
        self.killed.retain(|(p, _)| resident.contains(p));
        let gone: Vec<(u32, usize)> = self.placed_fx.keys().filter(|(p, _)| !resident.contains(p)).copied().collect();
        for k in gone {
            if let Some(h) = self.placed_fx.remove(&k) {
                self.effects.stop(&table, h);
                self.effects.release(h);
            }
        }
    }

    /// SPEC-0034 section 36 and SPEC-0025 Save points step 2: the glow (definitions 400 and
    /// 404) is stopped, moved to a newly chosen save point and started again, updated every tick
    /// with the camera as the listener, and stopped when its world is left (region reset).
    fn save_glow_step(&mut self, moved_to: Option<usize>) {
        let table = self.effect_table.clone();
        let pack = self.world.pack;
        if let Some((p, objects)) = self.save_glow {
            if p != pack {
                for h in objects {
                    self.effects.stop(&table, h);
                    self.effects.release(h);
                }
                self.save_glow = None;
            }
        }
        if let Some(sp) = moved_to.and_then(|k| self.world.save_points.get(k)).copied() {
            let objects = match self.save_glow {
                Some((_, o)) => o,
                None => {
                    let o = [400, 404].map(|id| self.effects.add_object(neopets_sim::effect::EffectObject::new(&table, id, sp.position, neopets_sim::effect::IDENTITY)));
                    self.save_glow = Some((pack, o));
                    o
                }
            };
            for h in objects {
                self.effects.stop(&table, h);
                self.effects.set_position(h, sp.position);
            }
        }
        if let Some((_, objects)) = self.save_glow {
            let (eye, focus) = (self.camera.eye, self.camera.focus);
            let forward = unit([focus[0] - eye[0], focus[1] - eye[1], focus[2] - eye[2]]);
            for h in objects {
                self.effects.update(&table, h, Some((eye, forward)), &mut self.rng);
            }
        }
    }

    /// SPEC-0031 per pass: the slots' blocks (Tor's voice, weapon, shield and power blocks, the
    /// area block of the music row), the listener from the camera, the footsteps and landings of
    /// the grounded hero, the low-health cue, then the voice update.
    fn sound_step(&mut self) {
        use neopets_sim::sound::{self as snd, Play};
        let s = &mut self.sound;
        s.slots[snd::VOICE] = Some(348);
        s.slots[snd::WEAPON] = Some(snd::weapon_block(false, self.weapon));
        s.slots[snd::SHIELD] = Some(snd::shield_block(false, self.shield));
        s.slots[snd::POWER] = Some(snd::power_block(None));
        if let Some(sel) = self.music_selection {
            match sel.block {
                Some(b) => s.slots[snd::AREA] = Some(b),
                None => {
                    let r = self.design.music_row(self.world.music.and_then(|(k, _)| self.world_keys.1.iter().position(|&v| v as i32 == k)).unwrap_or(193));
                    if r.block < -1 {
                        s.slots[snd::AREA] = None;
                    }
                }
            }
        }
        // The listener: the camera position plus 5 x its forward axis; rows right, up, forward.
        let (eye, focus) = (self.camera.eye, self.camera.focus);
        let f = unit([focus[0] - eye[0], focus[1] - eye[1], focus[2] - eye[2]]);
        let r = unit([f[1] * UP[2] - f[2] * UP[1], f[2] * UP[0] - f[0] * UP[2], f[0] * UP[1] - f[1] * UP[0]]);
        let u = [r[1] * f[2] - r[2] * f[1], r[2] * f[0] - r[0] * f[2], r[0] * f[1] - r[1] * f[0]];
        s.listener = snd::Listener::from_camera(eye, [r, u, f]);
        let p = self.loco.position();
        let state = self.loco.state;
        let grounded = self.loco.body.gap < 0.1;
        let code = self.world.probe(p[0], p[2], p[1] + neopets_sim::motion::PROBE_CEILING).kind;
        // Footsteps: a new layer-0 frame with bit 0x80 or 0x100 while grounded; gait by state.
        let frame = self.hero.layer0();
        if frame.map(|f| f.0) != self.foot_frame.map(|f| f.0) {
            self.foot_frame = None;
        }
        if grounded && self.hero.frame_bits() & 0x180 != 0 && frame != self.foot_frame {
            self.foot_frame = frame;
            let gait = match state {
                4 => 40,
                3 => 20,
                _ => 0,
            };
            if let Some(o) = code.and_then(surface_offset) {
                self.sound.play(snd::MAIN, gait + o, Play { emitter: Some(p), ..Default::default() });
            }
            // SPEC-0034 3.16 footstep dust: entry 1 dirt (codes 0 and 18), 2 grass (1), 5 muck
            // (5), 8 water (8 and 11), 9 wood (9), at joint 28 on a 0x80 frame, else joint 31.
            let entry = match code {
                Some(0 | 18) => Some(1),
                Some(1) => Some(2),
                Some(5) => Some(5),
                Some(8 | 11) => Some(8),
                Some(9) => Some(9),
                _ => None,
            };
            if let Some(&id) = entry.and_then(|e| self.dust_ids.get(e)) {
                let joint = if self.hero.frame_bits() & 0x80 != 0 { 28 } else { 31 };
                let frames = self.hero.bone_frames();
                let at = frames.get(joint).map_or(p, |f| placement(p, self.loco.facing).mul(f).t);
                let table = self.effect_table.clone();
                self.effects.one_shot(&table, id, at, neopets_sim::effect::IDENTITY, &mut self.rng);
            }
        }
        // Landing: the hero leaves a jump or fall state onto the ground.
        let airborne = |s: u8| (hero_states::JUMP..=hero_states::JUMP_RUN).contains(&s) || s == 11 || s == 12 || s == hero_states::LONG_FALL;
        if airborne(self.last_state) && !airborne(state) && grounded {
            if let Some(c) = code.and_then(landing_cue) {
                self.sound.play(snd::MAIN, c, Play { emitter: Some(p), ..Default::default() });
            }
        }
        // Jump take-off: voice cue 10.
        if !airborne(self.last_state) && (hero_states::JUMP..=hero_states::JUMP_RUN).contains(&state) {
            self.sound.play(snd::VOICE, 10, Play::default());
        }
        self.last_state = state;
        // Low health: cue 157 every tick below 0.15 of the maximum, with do-not-restart.
        if self.hero_hp > 0 && (self.hero_hp as f32) < 0.15 * self.hero_max as f32 {
            self.sound.play(snd::MAIN, 157, Play { do_not_restart: true, ..Default::default() });
        }
        self.creature_sounds();
        self.emitters();
        let finished = std::mem::take(&mut self.sound_finished);
        self.sound.update(&finished);
    }

    /// Creature blocks and animation sounds (SPEC-0031): the blocks the resident characters'
    /// main tables name are held in slots 6 to 24 while referenced; a character whose animation
    /// changes to one with a sound word plays its cue once at its position, or keeps a
    /// keep-alive voice refreshed while that animation stays current.
    fn creature_sounds(&mut self) {
        use neopets_sim::sound::{Play, SLOTS};
        let mut wanted: Vec<u32> = self.characters.iter().flat_map(|c| c.sound_blocks.iter().copied()).collect();
        wanted.sort_unstable();
        wanted.dedup();
        for k in 6..SLOTS {
            if self.sound.slots[k].is_some_and(|b| !wanted.contains(&b)) {
                self.sound.slots[k] = None;
            }
        }
        for b in wanted {
            if !self.sound.slots.contains(&Some(b)) {
                if let Some(k) = (6..SLOTS).find(|&k| self.sound.slots[k].is_none()) {
                    self.sound.slots[k] = Some(b);
                }
            }
        }
        let mut plays = Vec::new();
        for c in &mut self.characters {
            if c.dying.is_some() {
                continue;
            }
            let action = c.anim.action();
            let changed = c.sound_action != Some(action);
            c.sound_action = Some(action);
            let Some(w) = c.anim.sound_word(action) else { continue };
            let (Some(block), Some((cue, keep))) = (crate::character::sound_block(w), crate::character::sound_cue(w)) else { continue };
            if changed || keep {
                plays.push((block, cue, keep, c.position));
            }
        }
        for (block, cue, keep, at) in plays {
            let Some(slot) = self.sound.slots.iter().position(|&s| s == Some(block)) else { continue };
            if keep {
                self.sound.keep_alive(slot, cue, Some(at));
            } else {
                self.sound.play(slot, cue, Play { emitter: Some(at), ..Default::default() });
            }
        }
    }

    /// Placed sound emitters and adventure zones of the current world (SPEC-0031 Placed sound
    /// emitters; SPEC-0009 Intensity inputs). Every record is tested (the original walks the
    /// camera cell's list A; the sphere gates decide the same).
    fn emitters(&mut self) {
        use neopets_sim::sound::{Play, AREA};
        let l = self.sound.listener.point;
        let dist = |c: [f32; 3]| ((c[0] - l[0]).powi(2) + (c[1] - l[1]).powi(2) + (c[2] - l[2]).powi(2)).sqrt();
        let hero = self.loco.position();
        let hero_d = |c: [f32; 3]| ((c[0] - hero[0]).powi(2) + (c[1] - hero[1]).powi(2) + (c[2] - hero[2]).powi(2)).sqrt();
        let default = self.sound.default_volume;
        for i in 0..self.world.ambients.len() {
            let a = self.world.ambients[i];
            if a.subkind != 0 {
                // Adventure zone: the listener inside the sphere widened by 1; f from the hero.
                if dist(a.centre) < a.radius + 1.0 {
                    let d = hero_d(a.centre);
                    let f = if d <= a.inner || d >= a.radius { 1.0 } else { 1.0 - (d - a.inner) / (a.radius - a.inner) };
                    self.intensity.adventure(f);
                }
                continue;
            }
            if dist(a.centre) >= a.radius {
                self.world.ambient_counters[i] = 0;
                continue;
            }
            if a.factor < 0.0 || a.cue < 0 {
                continue;
            }
            let emitter = a.positioned.then_some(a.centre);
            if self.sound.has(AREA, a.cue, emitter) {
                self.sound.keep_alive(AREA, a.cue, emitter);
                self.world.ambient_counters[i] = 0;
            } else if self.world.ambient_counters[i] < 6 {
                self.world.ambient_counters[i] += 1;
                if self.world.ambient_counters[i] >= 3 {
                    let volume = (default as f32 * a.factor) as i32;
                    self.sound.play(AREA, a.cue, Play { emitter, do_not_restart: true, keep_alive: true, volume: Some(volume), radius: Some((a.radius, a.inner)), ..Default::default() });
                }
            }
        }
        for s in self.world.spheres.clone() {
            if dist(s.centre) >= s.radius + 1.0 {
                continue;
            }
            if self.sound.has(AREA, s.cue, Some(s.centre)) {
                self.sound.keep_alive(AREA, s.cue, Some(s.centre));
            } else {
                self.sound.play(AREA, s.cue, Play { emitter: Some(s.centre), do_not_restart: true, keep_alive: true, radius: Some((s.hearing.max(30.0), 0.0)), ..Default::default() });
            }
        }
    }

    /// The emote requests the box's reveal reached (SPEC-0022; SPEC-0009 Emote clip lookup): each
    /// looks up its row and directory position now and queues (clip, emote id) on its actor
    /// (SPEC-0040 Behavior 22): hero kinds on the hero's queues, partner kinds on the talk
    /// partner's (none without a partner). A row with no clip still queues its gesture.
    fn emote_requests(&mut self) {
        use neopets_sim::emote;
        // SPEC-0022 Dialogue control records 0x11 to 0x13: set or clear progress flags (the range
        // rule) and give items (a count of exactly 18 gives 1; item 135 is a Neopoint reward).
        for (kind, a0, a1) in std::mem::take(&mut self.textbox.commands) {
            let range = |a0: i32, a1: i32| if a1 < 1 || a1 <= a0 { a0..=a0 } else { a0..=a1 };
            match kind {
                0x11 | 0x12 => {
                    for f in range(a0, a1) {
                        if let Ok(f) = u16::try_from(f) {
                            self.state.set(f, kind == 0x11);
                        }
                    }
                }
                _ if a0 == 135 => self.state.neopoints += a1.max(0),
                _ => {
                    if let Ok(item) = u16::try_from(a0) {
                        self.state.give(item, if a1 == 18 { 1 } else { a1.max(0) as u32 });
                    }
                }
            }
        }
        for (kind, id) in std::mem::take(&mut self.textbox.emotes) {
            let partner = matches!(kind, emote::PARTNER_BODY | emote::PARTNER_FACE);
            let who = if partner {
                let Some(k) = self.talk_with.and_then(|t| self.characters.iter().position(|c| c.id == t)) else { continue };
                Some(k)
            } else {
                None
            };
            let voice_set = who.map_or(-1, |k| self.characters[k].row.map_or(-1, |r| r.voice_set));
            let row = self.design.emote(!partner, id).map(|r| emote::Row { clip: r.clip, chance: r.chance, variance: r.variance });
            let rng = &mut self.rng;
            let pos = emote::position(kind, row, voice_set, false, &mut |n| rng.next_u32() % n.max(1));
            let (bank, dir) = if partner { (144, 0) } else { (145, 1) };
            let clip = pos.and_then(|p| self.design.paths.get(dir).and_then(|d| emote::playable(p, d))).map(|(start, len)| (bank, start, len));
            let entry = emote::Entry { clip, id };
            match who {
                Some(k) => self.characters[k].emotes.push(kind, entry),
                None => self.hero_emotes.push(kind, entry),
            }
        }
    }

    /// SPEC-0040 Behavior 22, the consumer, once per tick while the conversation runs: the hero
    /// (body emotes on) and the talk partner (body emotes off in a seated or lying pose: main id
    /// 65 to 74, 78, 97, 98, 146 or 147). A main request sets the face override and requests main
    /// id B; otherwise only the face override is set. A clip starts only then, and is refused
    /// while the stinger handle is busy, the bank music is not active, or the voice volume is 0.
    fn emote_consumers(&mut self) {
        type Clip = (u32, u32, u32);
        let mut clips: Vec<Clip> = Vec::new();
        let apply = |anim: &mut crate::character::HeroAnim, out: neopets_sim::emote::Consumed<Clip>| {
            anim.set_face(u32::try_from(out.face).ok());
            if let Some(b) = out.request_main {
                anim.request(b.max(0) as u32, false);
            }
            out.clip
        };
        let view = self.hero.emote_view();
        let out = self.hero_emotes.consume(view, true);
        clips.extend(apply(&mut self.hero, out));
        if let Some(c) = self.talk_with.and_then(|t| self.characters.iter_mut().find(|c| c.id == t)) {
            let view = c.anim.emote_view();
            let busy = matches!(view.main, 65..=74 | 78 | 97 | 98 | 146 | 147);
            let out = c.emotes.consume(view, !busy);
            clips.extend(apply(&mut c.anim, out));
        }
        for (bank, start, len) in clips {
            if !self.stinger_playing && self.bank_music.playing.is_some() && self.emote_volume() > 0 {
                self.stinger_playing = true;
                self.emote_requests.push((bank, start, len));
            }
        }
    }

    /// The talk partner's main id and face override, for the trace.
    pub fn talk_partner_view(&self) -> Option<(String, i32, i32)> {
        let c = self.talk_with.and_then(|t| self.characters.iter().find(|c| c.id == t))?;
        let v = c.anim.emote_view();
        Some((c.name.clone(), v.main, v.face))
    }

    /// A stinger request (SPEC-0009): indices 0 to 18 only; one at a time; stinger 4 not within
    /// 40 s of its last start (from clock 0).
    pub fn stinger(&mut self, k: i32) {
        if !(0..19).contains(&k) || self.stinger_playing || self.stinger_volume() <= 0 {
            return;
        }
        if k == 4 {
            if self.ticks < self.stinger4 + 40 * 60 {
                return;
            }
            self.stinger4 = self.ticks;
        }
        self.stinger_playing = true;
        self.stinger_requests.push(k);
    }

    /// A cue on the main slot with no position (menus, items, the HUD).
    /// A front-end menu sound (the menu sound table): always at the fixed volume 1,024.
    pub fn cue_fixed(&mut self, cue: i32) {
        self.sound.play(neopets_sim::sound::MAIN, cue, neopets_sim::sound::Play { volume: Some(1024), ..Default::default() });
    }

    /// The music controller on a front-end pass (SPEC-0009 Pass gate; SPEC-0035 Behaviors 3a and
    /// 3b): a bank override (the title's 25) replaces the row's bank offset, unshifted by the
    /// flag; the pause page's one-tick level is trunc(400 x Music gain); the stream gate is closed,
    /// so the stream step is skipped. The default cue volume and the voice update still run.
    pub fn frontend_music(&mut self, bank_override: Option<i32>, pause: bool) {
        let (key, selector) = self.world.music.unwrap_or((-1, 0));
        let row = self.world_keys.1.iter().position(|&v| v as i32 == key).unwrap_or(193);
        let r = self.design.music_row(row);
        let state = &self.state;
        let flag = neopets_sim::music::runtime_flag(selector, &|n| state.flag(n));
        let bank = bank_override.unwrap_or(r.bank + if flag { 67 } else { 0 });
        let duck = if self.stinger_playing { 0.7 } else { 1.0 };
        if pause {
            let level = (400.0 * self.options.music()) as i32;
            self.bank_music.tick(bank, level, duck);
        } else {
            self.bank_music.tick(bank, r.level, self.options.music() * duck);
        }
        self.sound.default_volume = (r.scale as f32 * self.options.effects()) as i32;
        let finished = std::mem::take(&mut self.sound_finished);
        self.sound.update(&finished);
    }

    /// The stinger volume, trunc(1024 x Sound-effects gain), and the emote-clip volume,
    /// trunc(1024 x Voice gain).
    pub fn stinger_volume(&self) -> i32 {
        (1024.0 * self.options.effects()) as i32
    }

    pub fn emote_volume(&self) -> i32 {
        (1024.0 * self.options.voice()) as i32
    }

    pub fn cue(&mut self, cue: i32) {
        self.sound.play(neopets_sim::sound::MAIN, cue, neopets_sim::sound::Play::default());
    }

    /// SPEC-0009 Per-tick controller, the parts this build has: the current world's music row
    /// (its type-1001 key looked up in the value table; 193, the default row, when absent), the
    /// run-time flag, and the stream step with the ceiling trunc(Cap x Sound-effects gain). The
    /// gains are 1.0 until options are applied (not implemented).
    fn music_step(&mut self) {
        let (key, selector) = self.world.music.unwrap_or((-1, 0));
        let row = self.world_keys.1.iter().position(|&v| v as i32 == key).unwrap_or(193);
        let r = self.design.music_row(row);
        let state = &self.state;
        let flag = neopets_sim::music::runtime_flag(selector, &|n| state.flag(n));
        let s = neopets_sim::music::select(r.bank, r.stream, r.block, flag);
        self.music.tick(s.stream, s.stop, r.cap, self.options.effects());
        // The default cue volume: trunc(row Scale x Sound-effects gain).
        self.sound.default_volume = (r.scale as f32 * self.options.effects()) as i32;
        self.music_selection = Some(s);
        // The bank: the row's offset with the flag's +67 (added to negative offsets too).
        let duck = if self.stinger_playing { 0.7 } else { 1.0 };
        self.bank_music.tick(r.bank + if flag { 67 } else { 0 }, r.level, self.options.music() * duck);
        // A progress flag that rose since the last tick plays its table-2 stinger (v - 1).
        let risen: Vec<u16> = self.state.flags.difference(&self.flags_seen).copied().collect();
        self.flags_seen = self.state.flags.clone();
        for n in risen {
            if let Some((_, v)) = self.design.flag_cue(n) {
                if v >= 1 {
                    self.stinger(v - 1);
                }
            }
        }
        // Intensity: k counts the characters in AI state 19 (agro) that are not dead.
        let k = self.characters.iter().filter(|c| c.enemy.as_ref().is_some_and(|e| e.state == 19 && !e.dead)).count();
        self.intensity.tick(k, self.hero_hp, self.hero_max);
    }

    /// The location a save names: the current world's map area, else the last named one, else
    /// "Unknown Location" (SPEC-0025 Location, date and progress).
    pub fn location_name(&self) -> Vec<u8> {
        self.world
            .map_area
            .or(self.last_area)
            .and_then(|m| self.design.location_name(m as usize))
            .unwrap_or_else(|| b"Unknown Location".to_vec())
    }

    fn world_index(&self, pack: u32) -> usize {
        neopets_sim::save::world_index(&self.world_keys.0, &self.world_keys.1, pack as i32)
    }

    /// Rebuild a slot from the live game (SPEC-0025 Saving, step 3). Only Tor is in this build:
    /// Roberta's words hold her start pools, her timers are inactive and her entries empty;
    /// the stash, petpet, movies, per-world tables and count table have no live
    /// state yet and are written empty; the options block holds the start-up defaults.
    pub fn write_slot(&self, s: &mut neopets_sim::save::Slot, date: &[u8]) {
        use neopets_sim::save::{self as sv, Group, at};
        let p = self.loco.position();
        let f = self.loco.facing;
        s.set_vec4(at::POSITION, [p[0], p[1], p[2], 1.0]);
        s.set_vec4(at::FACING, [f[0], f[1], f[2], 0.0]);
        s.set_u32(at::ACTIVE_HERO, self.hero_index as u32);
        // Keys A, B and C: the worlds in T, E and I as saved ids (0xFFFF for an empty buffer,
        // like a lookup miss, so the slot stays present).
        let mut packs = [None; 3];
        for w in std::iter::once(&self.world).chain(self.other.as_ref()) {
            let k = match w.buffer {
                Buffer::Transition => 0,
                Buffer::Exterior => 1,
                Buffer::Interior => 2,
            };
            packs[k] = Some(w.pack);
        }
        if packs[0].is_none() {
            packs[0] = self.interior_t;
        }
        let (keys, ids) = &self.world_keys;
        for (k, pack) in packs.iter().enumerate() {
            s.set_key(k, pack.map_or(0xFFFF, |pk| sv::forward(keys, ids, pk as i32) as i32));
        }
        s.clear_flags();
        for &n in &self.state.flags {
            s.set_flag(n, true);
        }
        s.0[at::MOVIES..at::MOVIES + 368].fill(0);
        s.set_i32(at::WALLET, self.state.neopoints);
        s.set_i32(at::STASH, 0);
        s.set_i32(at::LUCK, self.luck);
        // Each hero's maxima, by index (the parked hero's from its record).
        let parked = self.parked.as_ref().map_or((self.growth.start[1], self.growth.start[3], self.growth.start[1], self.growth.start[3]), |p| (p.hp, p.mp, p.hp_max, p.mp_max));
        let live = (self.hero_hp, self.hero_mp, self.hero_max, self.hero_mp_max);
        let by_hero = if self.hero_index == 0 { [live, parked] } else { [parked, live] };
        for (h, v) in by_hero.iter().enumerate() {
            s.set_maximum_words(h, v.2, v.3);
        }
        // SPEC-0047 Save: the live petpet's type, T and F (-1 with no pet or T below 1).
        let (ty, t, f) = neopets_sim::petpet::save_fields(self.petpet.as_ref().map(|p| (p.type_id, p.t, p.f)));
        s.set_i32(at::PET_TYPE, ty);
        s.set_i32(at::PET_TIME, t);
        s.set_i32(at::PET_FULL, f);
        // Container A from the party's counts by text category, in id order; the equipped
        // weapon and shield are Tor's entries (entry 0) of the weapon and shield families.
        let mut lists: HashMap<&str, Vec<(u16, u32)>> = HashMap::new();
        let mut held: Vec<(&u16, &u32)> = self.state.items.iter().filter(|(_, n)| **n > 0).collect();
        held.sort();
        for (&id, &n) in held {
            // The equipment families and the tail are written from the equipped items below;
            // the cloak (category 9) is carried by the cloak flags.
            let category = self.design.item_category(id);
            let g = match category.map(Group::of_category) {
                Some(Some(Group::Quest)) => "quest",
                Some(Some(Group::Charm)) => "charm",
                Some(Some(Group::General)) | None => "general",
                Some(_) => continue,
            };
            lists.entry(g).or_default().push((id, n));
        }
        let family = |e: Option<u16>| e.map(|id| vec![(id, 1u32)]).unwrap_or_default();
        for g in Group::ORDER {
            let list = match g {
                Group::General => lists.remove("general").unwrap_or_default(),
                Group::Quest => lists.remove("quest").unwrap_or_default(),
                Group::Charm => lists.remove("charm").unwrap_or_default(),
                Group::Weapon => family(self.weapon),
                Group::Shield => family(self.shield),
                Group::Tail => Vec::new(),
            };
            for i in 0..g.capacity() {
                match list.get(i) {
                    Some(&(id, n)) => s.set_entry(0, g, i, Some(id), n as i32),
                    None => s.set_entry(0, g, i, None, 0),
                }
                s.set_entry(1, g, i, None, 0);
            }
        }
        for h in 0..2 {
            s.set_pools(h, by_hero[h].0, by_hero[h].1);
            for i in 0..14 {
                let ticks = if h == self.hero_index { self.statuses.timers[i] } else { -1 };
                s.set_status(h, i, sv::status_seconds(ticks));
            }
            for k in 0..4 {
                s.set_status_word(h, k, -1);
            }
            for i in 0..3 {
                s.set_mote(h, i, -1);
            }
        }
        s.0[at::T1..at::ONCE].fill(0);
        for set in 0..5 {
            for w in 0..sv::WORLDS {
                s.set_once(set, w, false);
            }
        }
        for &(id, pack) in &self.once {
            if let Some(set) = ONCE_ITEMS.iter().position(|&i| i == id) {
                s.set_once(set, self.world_index(pack), true);
            }
        }
        s.0[at::TREASURE..at::LOCATION].fill(0);
        s.set_treasures(&self.state.collected, &self.state.maps);
        s.set_location(&self.location_name());
        s.set_date(date);
        s.0[at::TIER] = sv::tier(|n| self.state.flag(n));
        s.0[at::COUNTS..at::COUNTS + sv::COUNTS].fill(0);
        s.set_u32(at::SWAP_MODE, self.swap_mode);
        let cloak = self.state.items.get(&126).is_some_and(|&n| n > 0);
        s.0[at::CLOAK..at::CLOAK + 3].copy_from_slice(&[cloak as u8, 0, 0]);
        // A copy of the live options block (SPEC-0025 Options block).
        s.0[at::OPTIONS..at::OPTIONS + 32].copy_from_slice(&self.options.block.to_bytes());
    }

    /// Apply a slot to this freshly loaded game (SPEC-0025 Loading, apply steps 2, 7 to 12, 15 and
    /// the end-of-pass apply), for Tor: progress flags, wallet, the party's entries re-added
    /// (carry limit on the general and quest lists), the cloak, the once-per-area sets, the
    /// status restore, luck, maxima and current pools, the restart point and the hero's
    /// placement at vectors A and B, and the options with their apply rule. Not applied:
    /// Roberta, the stash, the petpet, movies, per-world tables, the count table, swap mode and
    /// Motes.
    pub fn apply_slot(&mut self, s: &neopets_sim::save::Slot) {
        use neopets_sim::save::{self as sv, Group, at};
        self.state.flags.clear();
        for n in 0..sv::FLAGS {
            if s.flag(n) {
                self.state.flags.insert(n);
            }
        }
        self.state.items.clear();
        self.weapon = None;
        self.shield = None;
        self.state.neopoints = s.i32(at::WALLET);
        for (c, g, i, id, v) in s.adds() {
            let Ok(id) = u16::try_from(id) else { continue };
            if c != 0 {
                continue;
            }
            match g {
                Group::Weapon | Group::Shield if i == 0 => self.give(id),
                Group::General | Group::Quest => {
                    let limit = self.design.carry_limit(id).unwrap_or(0);
                    let n = if limit > 0 { (v as u32).min(limit) } else { v as u32 };
                    *self.state.items.entry(id).or_insert(0) += n;
                }
                Group::Charm => *self.state.items.entry(id).or_insert(0) += v as u32,
                _ => {}
            }
        }
        if s.0[at::CLOAK] != 0 {
            self.state.items.insert(126, 1);
        }
        self.once.clear();
        for (set, &id) in ONCE_ITEMS.iter().enumerate() {
            for w in 0..sv::WORLDS {
                if s.once(set, w) {
                    if let Some(&pack) = self.world_keys.0.get(w) {
                        self.once.insert((id, pack));
                    }
                }
            }
        }
        // Treasure sets (Loading step 11): the collected rows and held maps, then the dig-site
        // switch of each map row in the resident worlds (current first): a site is switched on
        // for a held map and off otherwise, without testing the collected set.
        let (rows, maps) = s.treasures();
        self.state.collected.retain(|&k| k == 0 || k >= sv::COLLECTED_ROWS as u32);
        self.state.collected.extend(rows);
        self.state.maps = maps.into_iter().collect();
        for i in 0..sv::MAP_ROWS as u32 {
            let found = [&self.world].into_iter().chain(self.other.as_ref()).find_map(|w| {
                let g = w.groups.iter().find(|g| g.kind == 6 && g.key == 200 + i)?;
                Some((w.pack, usize::try_from(*g.effects.first()?).ok()?))
            });
            if let Some((pack, slot)) = found {
                let on = self.state.maps.contains(&i);
                self.switch_placed(pack, slot, on);
            }
        }
        self.statuses.clear_all();
        for i in 0..14 {
            self.statuses.apply(i as i32, sv::restore_seconds(s.status(0, i)));
        }
        self.luck = sv::restore_luck(s.i32(at::LUCK));
        // SPEC-0047: the save load fills the pending petpet values for the next scene start.
        self.pet_pending = neopets_sim::petpet::Pending::load(s.i32(at::PET_TYPE), s.i32(at::PET_TIME), s.i32(at::PET_FULL));
        let m = sv::maxima(s, &self.growth);
        // Tor's pools come in live and Roberta's parked; the slot's active hero is then
        // selected (SPEC-0053), and the swap mode comes from save word 0x307C.
        if self.hero_index == 1 {
            self.exchange_heroes();
        }
        self.hero_max = m[0][0];
        self.hero_mp_max = m[0][1];
        let (hp, mp) = s.pools(0);
        self.hero_hp = hp.min(self.hero_max);
        self.hero_mp = mp.min(self.hero_mp_max);
        if let Some(p) = self.parked.as_mut() {
            let (hp, mp) = s.pools(1);
            (p.hp_max, p.mp_max) = (m[1][0], m[1][1]);
            (p.hp, p.mp) = (hp.min(p.hp_max), mp.min(p.mp_max));
        }
        self.swap_mode = s.u32(at::SWAP_MODE);
        self.swap = None;
        if s.u32(at::ACTIVE_HERO) == 1 {
            self.exchange_heroes();
        }
        let (a, b) = (s.vec4(at::POSITION), s.vec4(at::FACING));
        let pose = ([a[0], a[1], a[2]], [b[0], b[1], b[2]]);
        self.restart = Some(pose);
        self.arrive(pose);
        self.flags_seen = self.state.flags.clone();
        // Options (Loading, step 15): the slot's block, then the apply rule.
        self.options.block = neopets_sim::options::Block::from_bytes(&s.0[at::OPTIONS..at::OPTIONS + 32]);
        self.options.apply();
    }

    /// Load a world into the other buffer (a load's key A world in T).
    pub fn load_other(&mut self, disc: &mut Disc, index: &ArchiveIndex, pack: u32) -> Result<(), String> {
        let (w, _) = World::load(disc, index, &self.boot, pack)?;
        self.kinds.insert(w.pack, w.kind);
        self.other = Some(w);
        Ok(())
    }

    /// Give an item diagnostically: a weapon (types 3 and 6) or one of Tor's shields (type 5) is
    /// equipped; every item is also counted as held.
    pub fn give(&mut self, id: u16) {
        // The add puts equipment into its family entry (equipment_step); a Mote adds its
        // charge of 100 (SPEC-0021 Inventory).
        let n = if neopets_sim::mote::is_mote(id) { 100 } else { 1 };
        *self.state.items.entry(id).or_insert(0) += n;
        self.equipment_step();
    }

    /// SPEC-0021 Party inventory lists, categories 1, 2 and 5: each held equipment item goes to
    /// its family entry by the add rule (empty, or a greater id replaces the entry; the replaced
    /// item leaves the inventory); an entry whose item is no longer held is emptied. Tor's weapon
    /// and shield entries are the combat code's weapon and shield.
    fn equipment_step(&mut self) {
        let e = &mut self.equipment;
        for f in e.entries.iter_mut().flatten() {
            if f.is_some_and(|id| !self.state.items.get(&id).is_some_and(|&n| n > 0)) {
                *f = None;
            }
        }
        let mut held: Vec<u16> = self.state.items.iter().filter(|e| *e.1 > 0).map(|e| *e.0).collect();
        held.sort_unstable();
        for id in held {
            let kind = self.items.get(id).map_or(-1, |i| i.kind);
            if neopets_sim::inventory::family(kind).is_none() {
                continue;
            }
            match self.equipment.add(id, kind) {
                Ok(Some(old)) => {
                    self.state.items.remove(&old);
                }
                Ok(None) => {}
                // A lesser item than the entry is refused by the add.
                Err(()) if self.equipment.entries.iter().flatten().all(|&f| f != Some(id)) => {
                    self.state.items.remove(&id);
                }
                Err(()) => {}
            }
        }
        self.weapon = self.equipment.entries[0][self.hero_index];
        self.shield = self.equipment.entries[1][self.hero_index];
    }

    /// The item screen's view of the party (SPEC-0035 section 14.1) for the grid ids.
    pub fn item_view(&self, grid: &[i32; 20]) -> crate::menus::ItemView {
        let count = |id: i32| u16::try_from(id).ok().and_then(|id| self.state.items.get(&id)).copied().unwrap_or(0);
        let mut v = crate::menus::ItemView { marker: self.quick.marker, ..Default::default() };
        v.grid_counts = grid.map(count);
        let mut held: Vec<u16> = self.state.items.iter().filter(|e| *e.1 > 0).map(|e| *e.0).collect();
        held.sort_unstable();
        for (slot, id) in held.iter().filter(|&&id| self.design.item_category(id) == Some(8)).take(14).enumerate() {
            v.charms[slot] = *id as i32;
        }
        for (slot, id) in held.iter().filter(|&&id| self.design.item_category(id) == Some(7)).take(60).enumerate() {
            v.quests[slot] = *id as i32;
        }
        v.motes = self.equipment.motes[self.hero_index].map(|m| m.map_or(-1, |id| id as i32));
        let e = &self.equipment.entries;
        v.equipped = [e[0][0].is_some(), e[2][0].is_some(), e[1][0].is_some()];
        v.cloak = count(126) > 0;
        v.mote_counts = std::array::from_fn(|k| count(160 + k as i32));
        v
    }

    /// The item screen's two text lines for the selected entry (SPEC-0035 section 14.1); the
    /// times-obtained count is not kept, so it reads 0.
    pub fn item_lines(&self, selected: i32) -> [Vec<u8>; 2] {
        let Ok(id) = u16::try_from(selected) else { return [b" ".to_vec(), b" ".to_vec()] };
        let held = self.state.items.get(&id).copied().unwrap_or(0);
        let name = self.design.item_name(id);
        let line = neopets_sim::inventory::name_line(name.as_deref(), self.design.carry_limit(id).unwrap_or(0), 0, held);
        let desc = if held > 0 { self.design.item_description(id).map(|d| d.into_iter().take(255).collect()).unwrap_or_else(|| b" ".to_vec()) } else { b" ".to_vec() };
        [line, desc]
    }

    /// An item-screen command (SPEC-0035 section 14.2; SPEC-0021 Using an equipped item, Equip
    /// and unequip rules).
    pub fn item_command(&mut self, c: &crate::menus::Command) {
        use crate::menus::Command;
        match *c {
            // A charm is never consumed: the use fails (0xA2); others take the consumable use.
            Command::UseItem(id) if (160..=173).contains(&id) => self.cue(0xA2),
            Command::UseItem(id) => self.use_item(id),
            Command::Marker(m) => {
                self.quick.marker = m;
                if m >= 0 {
                    self.cue(0xA3);
                }
            }
            Command::EquipMote { word, id } => {
                if self.equipment.set_mote(self.hero_index, word, id) {
                    self.cue(0xA3);
                    self.hud.rearm_word(word);
                }
            }
            Command::Cloak => self.cloak_worn = !self.cloak_worn,
            _ => {}
        }
    }

    /// The game-over page's quit exit. With no save or load in the session the game tears the
    /// world down and goes to the card state and the title flow, which this build does not have:
    /// as a placeholder (not the game's behaviour) the hero comes back at the current world's
    /// entry record 0 with full health in state 59, and `continued` is set.
    fn continue_placeholder(&mut self) {
        let pose = self.world.entries.first().copied().unwrap_or((self.loco.position(), self.loco.facing));
        self.arrive(pose);
        self.hero_hp = self.hero_max;
        self.loco.set_end_state(hero_states::ARRIVAL);
        self.continued = true;
        self.last_hurt = format!("continued on tick {} (placeholder for the title flow)", self.ticks);
    }

    pub fn status_camera(&self) -> String {
        let c = &self.camera;
        let p = camera::polar(std::array::from_fn(|k| c.eye[k] - c.focus[k]));
        format!(
            "CAMERA (SPEC-0027 mode 0): profile distance {:.0}, distance {:.2}, yaw {:.1} deg, elevation {:.3}, spin {:.4}",
            c.profile.distance,
            p[0],
            p[1].to_degrees(),
            p[2],
            c.spin
        )
    }

    pub fn status_worlds(&self) -> String {
        let other = self.other.as_ref().map_or("none".into(), |o| format!("{} ({:?})", o.pack, o.buffer));
        let prompt = match self.prompt_shown.or(self.talk_prompt.then_some(8)) {
            Some(8) => " | E (Square): Talk (label 8)".to_string(),
            Some(13) => " | E (Square): Enter (prompt 13)".to_string(),
            Some(26) => " | E (Square): Exit (prompt 26)".to_string(),
            Some(id) => format!(" | E (Square): prompt {id}"),
            None => String::new(),
        };
        let job = self.job.map_or(String::new(), |j| format!(" | loading {j:?}"));
        let ch = self.music.channels;
        let job = format!(
            "{job} | MUSIC (SPEC-0009): {:?}, streams [{:?} at {}/1024, {:?} at {}/1024], bank {:?} at {} (sent {}), E {} register 2 {} G {}",
            self.music_selection.map(|s| (s.bank, s.stream, s.block)),
            ch[0].stream, ch[0].volume, ch[1].stream, ch[1].volume,
            self.bank_music.playing, self.bank_music.volume, self.bank_music.sent(),
            self.intensity.e, self.intensity.register2, self.intensity.excite
        );
        let prompt = match self.message {
            Some((line, _)) if self.dialogue_open => match self.world.lines.line(line as usize) {
                // SPEC-0022 Speaker rule: the hero's talk partner names the speaker.
                Some(l) => format!("{prompt}\nLINE {line}: {}: {}", self.talk_partner.as_deref().unwrap_or(&l.speaker), l.text),
                None => format!("{prompt} | LevelMsg line {line} (no such line)"),
            },
            _ => prompt,
        };
        let mut names: Vec<String> = self
            .characters
            .iter()
            .filter(|c| self.pack_drawn(c.pack))
            .map(|c| match &c.townsfolk {
                _ if c.miamouse.is_some() => {
                    let m = c.miamouse.as_ref().unwrap();
                    format!("Miamouse [state {} sub {} catchable {} at {:.1},{:.1}; catching {} count {}]", m.state, m.sub, m.catchable, c.position[0], c.position[2], self.catch.on, self.catch.count)
                }
                _ if c.whinny.is_some() => format!(
                    "Whinny [{:?} at {:.0},{:.0}]",
                    c.whinny.as_ref().map(|w| w.mode).unwrap(),
                    c.position[0],
                    c.position[2]
                ),
                Some(t) => format!("{} [mode {} id {}]", c.name, t.mode, c.anim.action()),
                None if c.enemy.is_some() => {
                    let e = c.enemy.as_ref().unwrap();
                    let what = if e.dead { "dead".to_string() } else { format!("state {} outer {} inner {} seq {}", e.state, e.outer, e.inner, e.seq) };
                    format!(
                        "{} [{what}, hp {}, id {} at {:.0},{:.1},{:.0}]",
                        if c.name.is_empty() { "(unnamed)" } else { &c.name },
                        c.hp,
                        c.anim.action(),
                        c.position[0],
                        c.position[1],
                        c.position[2]
                    )
                }
                None => match c.row {
                    Some(r) => format!("{} [class {} hp {}]", if c.name.is_empty() { "(unnamed)" } else { &c.name }, r.class.unwrap_or(-1), r.hit_points),
                    None => c.name.clone(),
                },
            })
            .collect();
        names.dedup();
        if self.shots.live() > 0 {
            names.push(format!("enemy shots {}", self.shots.live()));
        }
        if let Some(pool) = self.pools.get(&self.world.pack).filter(|p| p.live() > 0) {
            let ids: Vec<u16> = pool.slots.iter().flatten().map(|i| i.id).collect();
            names.push(format!("world items {ids:?}"));
        }
        if let (Some(p), Some(c)) = (self.petpet.as_ref(), self.pet_char.and_then(|id| self.characters.iter().find(|c| c.id == id))) {
            names.push(format!("petpet [kind {} (present {:?}) state {} T {} anim {} at {:.1},{:.1}]", p.kind, self.pet_kind(), p.state, p.t, c.anim.action(), c.position[0], c.position[2]));
        }
        if self.effects.live_spawners() > 0 || self.effects.live_particles() > 0 {
            names.push(format!("effects [{} spawners, {} particles]", self.effects.live_spawners(), self.effects.live_particles()));
        }
        let roundup: Vec<u16> = (107..=110).filter(|&f| self.state.flag(f)).collect();
        if !roundup.is_empty() {
            names.push(format!("round-up flags {roundup:?}"));
        }
        format!(
            "WORLDS (SPEC-0006): current {} ({:?}), other {other}, swaps {}{prompt}{job} | {}\nCHARACTERS (townsfolk modes 0, 10 to 13; common enemies SPEC-0038, partial): {}",
            self.world.pack,
            self.world.buffer,
            self.swaps,
            self.last_door,
            names.join(", ")
        )
    }

    pub fn status(&self) -> String {
        let p = self.loco.position();
        format!(
            "PLAY (M7): hero at ({:.1}, {:.1}, {:.1}), state {}, phase {}, action {}, rate {:.2}, gap {:.2}, hp {}/{}, mp {}/{}, {} NP{}. \
             WASD move, Shift run (R1), Space jump (Cross), J/L I/K or right stick camera, R recentre (R3)",
            p[0],
            p[1],
            p[2],
            self.loco.state,
            self.loco.phase,
            self.loco.action(),
            self.loco.rate,
            self.loco.body.gap,
            self.hero_hp,
            self.hero_max,
            self.hero_mp,
            self.hero_mp_max,
            self.state.neopoints,
            if self.last_hurt.is_empty() { String::new() } else { format!(" ({})", self.last_hurt) }
        ) + &if self.quick.open {
            format!("\nQUICK ITEMS (SPEC-0021): entries Right/Up/Left/Down {:?}, selected {}; Space uses, F sets, T closes", self.quick.entries, self.quick.selection)
        } else if self.quick.marker >= 0 {
            format!(" Quick item {} (T uses).", self.quick.marker)
        } else {
            String::new()
        } + &match (self.death.page, self.death.flag_e) {
            (Some(n), _) => format!("\nGAME OVER page (SPEC-0055; the front end's sprites are not drawn): Continue with Space (Cross) from page pass 122, now {n}"),
            (None, true) => "\nGAME OVER".to_string(),
            _ => String::new(),
        }
    }
}

fn add(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

fn sub_v(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn scale(a: [f32; 3], s: f32) -> [f32; 3] {
    a.map(|c| c * s)
}

fn unit(a: [f32; 3]) -> [f32; 3] {
    let l = (a[0] * a[0] + a[1] * a[1] + a[2] * a[2]).sqrt();
    if l > 0.0 { scale(a, 1.0 / l) } else { a }
}

/// A character's placement: translation to its position, turned so +Z is its facing.
fn placement(p: [f32; 3], f: [f32; 3]) -> neopets_data::anim::Affine {
    let th = f[0].atan2(f[2]);
    let (s, c) = th.sin_cos();
    neopets_data::anim::Affine { cols: [[c, 0.0, -s], [0.0, 1.0, 0.0], [s, 0.0, c]], t: p }
}

/// SPEC-0005 attach point transform: Translate(t) × Rz(rz) × Ry(ry) × Rx(rx); angles of
/// magnitude 0.001 or less are identity.
fn attach_rotation(t: [f32; 3], angles: [f32; 3]) -> neopets_data::anim::Affine {
    let rot = |axis: usize, a: f32| {
        if a.abs() <= 0.001 {
            return neopets_data::anim::Affine::IDENTITY;
        }
        let (s, c) = a.sin_cos();
        let cols = match axis {
            0 => [[1.0, 0.0, 0.0], [0.0, c, s], [0.0, -s, c]],
            1 => [[c, 0.0, -s], [0.0, 1.0, 0.0], [s, 0.0, c]],
            _ => [[c, s, 0.0], [-s, c, 0.0], [0.0, 0.0, 1.0]],
        };
        neopets_data::anim::Affine { cols, t: [0.0; 3] }
    };
    let tr = neopets_data::anim::Affine { cols: neopets_data::anim::Affine::IDENTITY.cols, t };
    tr.mul(&rot(2, angles[2])).mul(&rot(1, angles[1])).mul(&rot(0, angles[0]))
}

/// The dialogue box's boot words (SPEC-0022 Box geometry constants; Dialogue box layout).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct BoxWords {
    pub margin: i32,
    pub width_term: i32,
    pub height_terms: (i32, i32),
    /// The left (Triangle) and right (Circle or continue) prompt x positions.
    pub prompt_x: (i32, i32),
    pub wrap: i32,
    pub anchor: i32,
    pub name_colour: [u8; 4],
    pub body_colour: [u8; 4],
    pub prompt_colour: [u8; 4],
}

/// The sim's monster-row values from the disc row (SPEC-0038 Layout section 2).
fn params(r: &neopets_data::monster::Row, t: &neopets_data::monster::Tuning) -> enemy::Params {
    let slot = |s: &neopets_data::monster::AttackSlot| enemy::Slot {
        kind: s.kind,
        orient: s.orient,
        trigger_max: s.trigger_max,
        trigger_min: s.trigger_min,
        chance: s.chance,
        special: s.special,
        wind_speed: s.wind_speed,
        strike_speed: s.strike_speed,
        recover_speed: s.recover_speed,
        arc: s.arc,
        range: s.range,
        delay_max: s.delay_max,
        delay_min: s.delay_min,
        charge: s.charge,
        charge_max: s.charge_max,
        charge_min: s.charge_min,
        damage: s.damage,
        knockback: s.knockback,
        flags: s.flags,
        spawn_cap: s.spawn_cap,
        loopable: s.loopable,
        loop_min: s.loop_min,
        loop_max: s.loop_max,
        loop_hold: s.loop_hold,
        blocked_charge: s.blocked_charge,
        stun_max: s.stun_max,
        stun_min: s.stun_min,
        stun_type: s.stun_type,
        alignment: s.alignment,
        spawn_count: s.spawn_count,
        missile: neopets_sim::projectile::Missile {
            flight: s.flight,
            speed: s.speed,
            end_speed: s.end_speed,
            radius: s.radius,
            range: s.missile_range,
            inaccuracy: s.inaccuracy,
            spin: s.spin,
            damage: s.damage,
            band: (s.trigger_min, s.trigger_max),
            flags: s.flags,
            visual: s.visual,
            area_duration: s.area_duration,
            area_radius: s.area_radius,
            key: s.spawn_key,
            status: s.status,
            status_chance: s.status_chance,
            status_range: (s.status_range.0, s.status_range.1),
            effects: s.effects,
        },
    };
    enemy::Params {
        approach: t.approach,
        hostile: r.hostile,
        display_default: t.display_default,
        can_fly: r.can_fly,
        search_max: t.search_max,
        search_min: t.search_min,
        attack_timer_max: t.attack_timer_max,
        attack_timer_min: t.attack_timer_min,
        approach_timer_max: t.approach_timer_max,
        approach_timer_min: t.approach_timer_min,
        turn: t.turn,
        distance_hide: t.distance_hide,
        flying_hit: t.flying_hit,
        anger: t.anger,
        view_angle: t.view_angle,
        awareness: t.awareness,
        awareness_timer: t.awareness_timer,
        chance_far: t.chance_far,
        chance_near: t.chance_near,
        range_far: t.range_far,
        range_near: t.range_near,
        always_sight: t.always_sight,
        rush: t.rush,
        desired: t.desired,
        retreat: t.retreat,
        strafe: t.strafe,
        backbiter: t.backbiter,
        disappear: r.disappear as i32,
        element: t.element as i32,
        slots: t.slots.each_ref().map(slot),
    }
}

#[allow(clippy::too_many_arguments)]
fn enemy_view<'a>(
    p: [f32; 3],
    wk: &neopets_sim::walker::Walker,
    hero: [f32; 3],
    hero_facing: [f32; 3],
    bands: (f32, f32),
    attackable: bool,
    sight: bool,
    (route, hero_on_mesh): (bool, bool),
    contact: bool,
    speed: f32,
    clip: neopets_sim::hero::ClipState,
    has: &'a dyn Fn(u32) -> bool,
) -> enemy::View<'a> {
    enemy::View {
        position: p,
        facing: wk.facing,
        hero,
        hero_facing,
        hero_attackable: attackable,
        bands,
        sight,
        route,
        hero_on_mesh,
        waypoint: wk.waypoint(),
        blocked: wk.blocked,
        speed,
        contact,
        hero_blocks: false,
        spawn_bits: 0,
        radii: 0.0,
        clip: enemy::Clip { action: clip.action, progress: clip.progress, finished: clip.finished },
        has,
    }
}

/// SPEC-0008: the seed from the local clock (seconds, minutes, hours, day, month, year).
fn clock_seed() -> u32 {
    let secs = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(0, |d| d.as_secs());
    let (s, m, h) = ((secs % 60) as u8, (secs / 60 % 60) as u8, (secs / 3600 % 24) as u8);
    let days = secs / 86_400;
    neopets_sim::rng::Rng::clock_seed(s, m, h, (days % 28 + 1) as u8, (days / 30 % 12 + 1) as u8, (1970 + days / 365 % 100) as u8)
}

/// The hero as the camera sees it, with the ground kind of the patch under its feet (the same
/// probe as its ground step, 2.5 above the feet).
fn followed(loco: &Locomotion, world: &World) -> Followed {
    let p = loco.position();
    let ground_kind = world.probe(p[0], p[2], p[1] + 2.5).kind;
    Followed { position: p, facing: loco.facing, height: HERO_HEIGHT, moving: loco.moved(), ground_kind }
}

/// The eye wall push (SPEC-0027 Walls): the cell's frame records other than type 6, then the
/// box primitives, each with the eye radius 1.25. The frame records use the characters'
/// two-sided correction; the camera's single-margin plane test is only partly specified
/// (Medium). Moving objects, cylinders, planes and characters are not pushed against yet.
fn push_eye(world: &World, e: &mut [f32; 3]) {
    if let Some(g) = &world.ground {
        let walls: Vec<WallEdge> = g
            .frames_at(e[0], e[2])
            .into_iter()
            .filter(|f| f.kind != 6)
            .map(|f| WallEdge { face: f.face, run: f.run, top: f.top, length: f.length, height: f.height })
            .collect();
        contact::resolve_walls(e, &walls, camera::EYE_RADIUS);
    }
    contact::resolve(e, &world.boxes, camera::EYE_RADIUS);
}

/// A NUL-terminated string of the boot program's data at a virtual address (at most 64 bytes).
fn boot_string(boot: &[u8], va: u32) -> Vec<u8> {
    let words = neopets_data::archive::boot_u32_table(boot, va, 16).unwrap_or_default();
    let bytes: Vec<u8> = words.iter().flat_map(|w| w.to_le_bytes()).collect();
    bytes.into_iter().take_while(|&b| b != 0).collect()
}

/// A light-set record of the disc as the sim's record (SPEC-0028 section 2).
fn light_record(r: &neopets_data::light::LightRecord) -> neopets_sim::charlight::Record {
    let e = |x: &neopets_data::light::LightEntry| neopets_sim::charlight::Entry {
        position: x.position,
        direction: x.direction,
        colour: x.colour,
        kind: x.kind,
        strength: x.strength,
        flag: x.flag,
        radius: x.radius,
        radius_sq: x.radius_sq,
        zone: x.zone,
    };
    neopets_sim::charlight::Record { gate: r.gate != 0, a: r.a.iter().map(e).collect(), b: r.b.iter().map(e).collect(), c: r.c.iter().map(e).collect() }
}

/// The world id of `pack` in the area table (SPEC-0006), or -1.
fn world_id(boot: &BootTravel, pack: u32) -> i32 {
    boot.areas.packs.iter().position(|&p| p == pack).and_then(|s| boot.areas.world_ids.get(s)).map_or(-1, |&w| w as i32)
}
