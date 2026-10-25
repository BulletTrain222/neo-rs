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
use neopets_sim::motion::HERO_HEIGHT;
use neopets_sim::travel::{self as buffers, Buffer};
use std::cell::Cell;
use std::collections::HashMap;

const UP: [f32; 3] = [0.0, 1.0, 0.0];
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
    /// radius plus 0.25.
    pub fn touched_door(&self, p: [f32; 3]) -> Option<usize> {
        self.door_within(p, contact::HERO_RADIUS + 0.25)
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
        Ok(Self {
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
            spawned: false,
        })
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
        let resident = self.world.pack == world.pack || self.other.as_ref().is_some_and(|o| o.pack == world.pack);
        if !resident && self.streams(world.kind) {
            self.other = Some(world);
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
        true
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

    /// The pack a pending door job wants loaded.
    pub fn job_pack(&self) -> Option<u32> {
        match self.job? {
            Job::FromExterior { pack: p, .. } | Job::Replace(p, _) | Job::ReloadTransition(p) => Some(p),
        }
    }

    /// Finish a door job with its loaded world (SPEC-0006 Interior entry steps, exterior
    /// replacement and the post-load rule). Fades are not drawn.
    pub fn finish_job(&mut self, world: World) {
        let Some(job) = self.job.take() else { return };
        self.kinds.insert(world.pack, world.kind);
        match job {
            Job::FromExterior { plane, reverse, .. } if world.kind == Kind::Transition => {
                // E current, D a transition: it takes T and the hero lands on its door.
                self.other = Some(world);
                let landed = self.land_in_other(plane, reverse, false);
                self.last_door = format!("landed in transition {} at its door {plane}: {landed}", self.world.pack);
            }
            Job::FromExterior { .. } => {
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
            self.last_door = format!("door {n}: a script door (needs the level-script runtime)");
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
        if self.prompt_shown.is_some() && input.square {
            self.confirm = true;
        }
        self.loco.update(input, forward, left, self.hero.clip_state());
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
            }
        }
        self.door_step();
        self.prompt_shown = self.door_prompt.take();
        self.root_step = self.hero.tick();
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
        let prompt = match self.prompt_shown {
            Some(13) => " | E (Square): Enter (prompt 13)".to_string(),
            Some(id) => format!(" | E (Square): Exit (prompt {id})"),
            None => String::new(),
        };
        let job = self.job.map_or(String::new(), |j| format!(" | loading {j:?}"));
        format!(
            "WORLDS (SPEC-0006): current {} ({:?}), other {other}, swaps {}{prompt}{job} | {}",
            self.world.pack, self.world.buffer, self.swaps, self.last_door
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
