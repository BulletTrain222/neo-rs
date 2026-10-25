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
use neopets_sim::hero::{HeroInput, Locomotion};
use neopets_sim::level::{self, Action, GameState};
use neopets_sim::motion::HERO_HEIGHT;
use neopets_sim::travel::{self as buffers, Buffer};
use std::cell::Cell;
use std::collections::HashMap;

const UP: [f32; 3] = [0.0, 1.0, 0.0];
/// The slot number used here for a world's embedded region.
const EMBEDDED_SLOT: u8 = 255;
/// SPEC-0006 Default start: pack 691 in T and 909 in E, the exterior current.
const START_EXTERIOR: u32 = 909;
const START_TRANSITION: u32 = 691;

/// A world pack resident in a buffer: what the hero's update and the camera read of it.
pub struct World {
    pub pack: u32,
    pub kind: Kind,
    pub buffer: Buffer,
    ground: Option<Ground>,
    /// Rectangle primitives that push characters out (SPEC-0018 box push; new-game mask 0).
    boxes: Vec<BoxShape>,
    neighbours: Neighbours,
    /// Teleport planes (doors), the arrival-door list (tag 0x0E) and the entry records.
    pub planes: Vec<TeleportPlane>,
    arrivals: Neighbours,
    entries: Vec<([f32; 3], [f32; 3])>,
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
}

impl World {
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
            scripted_doors: boot.imports(&bytes, &chain.chunks, "RegisterTeleportCallback"),
            lines: neopets_data::dialogue::LineTable::of(&bytes, &chain.chunks),
            entities: neopets_data::travel::entity_table(&bytes, &chain.chunks),
            navmeshes: neopets_data::navmesh::navmeshes(&bytes, &chain.chunks)
                .into_iter()
                .map(|m| neopets_sim::nav::NavMesh { kind: m.kind, triangles: m.triangles, neighbours: m.neighbours, table: m.table })
                .collect(),
            pens: Vec::new(),
        };
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
        self.ground.as_ref().map_or(ProbeHit { height: ground::MISS, normal: UP, kind: None, found: false }, |g| g.probe_hit(x, z, c))
    }
}

/// Diagnostic route goals for the trace.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Goal {
    Swap,
    NewNeighbour,
    Door,
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
    /// The townsfolk routine (SPEC-0056) for class-6 characters; other classes only idle.
    pub townsfolk: Option<neopets_sim::npc::Townsfolk>,
    /// Class-7 Whinnies: their walker (motion record and navigation) and routine.
    pub walker: Option<neopets_sim::walker::Walker>,
    pub whinny: Option<neopets_sim::whinny::Whinny>,
    root_step: [f32; 3],
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
    /// The Talk label is shown this tick, and the talked character (the speaker of its lines).
    pub talk_prompt: bool,
    pub talk_partner: Option<String>,
    /// The character in the conversation (base state 25), by id.
    talk_with: Option<u64>,
    /// The shared dialogue box is open (SPEC-0022: it closes only on a Square press; the hero
    /// stays in the talking state meanwhile).
    pub dialogue_open: bool,
    /// The game's random generator (SPEC-0008): one stream, seeded once.
    pub rng: neopets_sim::rng::Rng,
    /// Region characters of the resident worlds, the region each (pack, slot) should hold, and
    /// slot loads waiting for the region loader.
    pub characters: Vec<Character>,
    slot_regions: HashMap<(u32, u8), u32>,
    pub region_wanted: Vec<(u32, u8, u32)>,
    next_character: u64,
    /// Set once the hero's meshes exist.
    pub spawned: bool,
}

impl Play {
    /// Load a hero outfit and a world pack's ground and spawn point. Pack 909 starts as the new
    /// game does, with 691 resident in T.
    pub fn load(disc: &mut Disc, index: &ArchiveIndex, pack: u32, outfit: u32) -> Result<Self, String> {
        let model = crate::archive_view::model(disc, index, outfit)?.ok_or("Outfit has no model")?;
        let hero = crate::archive_view::hero(disc, index, outfit)?.ok_or("Not a hero outfit (504–519)")?;
        let boot = BootTravel::from_disc(disc)?;
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
            talk_prompt: false,
            talk_partner: None,
            talk_with: None,
            dialogue_open: false,
            rng: neopets_sim::rng::Rng::seeded(clock_seed()),
            characters: Vec::new(),
            slot_regions: HashMap::new(),
            region_wanted: Vec::new(),
            next_character: 0,
            spawned: false,
        };
        // The start world's Init, then its activate-world routine (SPEC-0006 Make current),
        // which for 909 loads the farm family's slots.
        let init = level::on_init(play.world.pack);
        play.apply(&init);
        play.activated();
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

    /// A newly loaded other world runs its Init (its actions apply to that world).
    fn init_other(&mut self) {
        if let Some(o) = self.other.take() {
            let init = level::on_init(o.pack);
            let current = std::mem::replace(&mut self.world, o);
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
        let actions = level::on_activate(self.world.pack, &mut self.state);
        self.apply(&actions);
    }

    /// Carry out a level handler's actions (SPEC-0030 host calls), in order.
    fn apply(&mut self, actions: &[Action]) {
        for &a in actions {
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
                    let Some(o) = self.other.take() else { continue };
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
                    self.message = Some((line, self.ticks));
                    self.dialogue_open = self.world.lines.line(line as usize).is_some();
                }
                Action::Talk { line } => {
                    self.message = Some((line, self.ticks));
                    self.dialogue_open = self.world.lines.line(line as usize).is_some();
                }
                Action::WhinnyPen { inner, entrance, locator } => {
                    let point = self.world.entries.get(locator).map_or([0.0; 3], |e| e.0);
                    self.world.pens = vec![neopets_sim::whinny::Pen { inner, entrance, point, occupied: false }];
                }
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
            .filter(|(_, c)| c.pack == self.world.pack)
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
                let actions = level::on_talk(pack, routine, &mut self.state);
                self.apply(&actions);
            }
        }
        true
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
        loaded: Vec<(neopets_data::region::Placement, ModelData, HeroAnim, Option<neopets_data::monster::Row>)>,
    ) {
        if self.slot_regions.get(&(pack, slot)) != Some(&region) {
            return;
        }
        self.characters.retain(|c| !(c.pack == pack && c.slot == slot));
        for (pl, model, anim, row) in loaded {
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
                walker: row.filter(|r| r.class == Some(7)).map(|r| {
                    neopets_sim::walker::Walker::new(position, pl.facing, r.radius, r.weight.max(0.1), self.next_character as u32)
                }),
                whinny: row.filter(|r| r.class == Some(7)).map(|_| neopets_sim::whinny::Whinny::default()),
                root_step: [0.0; 3],
                row,
                anim,
                model,
            });
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
                let (l, g) = self.world.entries.first().copied().unwrap_or(([0.0; 3], [1.0, 0.0, 0.0]));
                let l = self.world.snap(l, 6.0);
                self.arrive((l, g));
                self.last_door = format!("entered interior {} at entry record 0", self.world.pack);
                self.activated();
            }
            Job::Replace(_, plane) => {
                self.world = world;
                self.other = None;
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
        // SPEC-0027 Update order: the camera runs first, from the hero's previous pose.
        let w = &self.world;
        self.camera.update(
            &followed(&self.loco, w),
            pad,
            |x, z, c| w.probe(x, z, c).height,
            |e| push_eye(w, e),
            |a, c| w.ground.as_ref().is_none_or(|g| g.segment_clear(a, c, true)),
        );
        let (forward, left) = self.camera.rows();
        // The interaction chooser shows the prompt posted on the previous tick and sets the
        // confirm flag on a Square press edge.
        // An open dialogue box takes Square to close it, and holds the hero in place.
        let mut input = *input;
        if self.dialogue_open {
            if input.square {
                self.dialogue_open = false;
                self.message = None;
                self.talk_with = None;
            }
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
        if self.prompt_shown.is_some() && input.square {
            self.confirm = true;
        }
        self.square_now = input.square;
        self.loco.update(&input, forward, left, self.hero.clip_state());
        self.pending_step();
        self.hero.play(self.loco.action());
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
        if !self.dialogue_open && !self.talk_step() {
            self.door_step();
        }
        self.prompt_shown = self.door_prompt.take();
        self.root_step = self.hero.tick();
        // Characters and slot choices of worlds that are no longer resident are released.
        let resident: Vec<u32> = std::iter::once(self.world.pack).chain(self.other.as_ref().map(|o| o.pack)).collect();
        self.characters.retain(|c| resident.contains(&c.pack));
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
        for c in self.characters.iter_mut().filter(|c| c.pack == world.pack) {
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
        for c in &mut self.characters {
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
                _ if c.whinny.is_some() => format!(
                    "Whinny [{:?} at {:.0},{:.0}]",
                    c.whinny.as_ref().map(|w| w.mode).unwrap(),
                    c.position[0],
                    c.position[2]
                ),
                Some(t) => format!("{} [mode {} id {}]", c.name, t.mode, c.anim.action()),
                None => match c.row {
                    Some(r) => format!("{} [class {} hp {}]", if c.name.is_empty() { "(unnamed)" } else { &c.name }, r.class.unwrap_or(-1), r.hit_points),
                    None => c.name.clone(),
                },
            })
            .collect();
        names.dedup();
        let roundup: Vec<u16> = (107..=110).filter(|&f| self.state.flag(f)).collect();
        if !roundup.is_empty() {
            names.push(format!("round-up flags {roundup:?}"));
        }
        format!(
            "WORLDS (SPEC-0006): current {} ({:?}), other {other}, swaps {}{prompt}{job} | {}\nCHARACTERS (townsfolk modes 0, 10 to 13; no walks or enemy AI): {}",
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
            "PLAY (M7): hero at ({:.1}, {:.1}, {:.1}), state {}, phase {}, action {}, rate {:.2}, gap {:.2}. \
             WASD move, Shift run (R1), Space jump (Cross), J/L I/K or right stick camera, R recentre (R3)",
            p[0], p[1], p[2], self.loco.state, self.loco.phase, self.loco.action(), self.loco.rate, self.loco.body.gap
        )
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
